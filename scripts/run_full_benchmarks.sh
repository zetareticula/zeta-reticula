#!/bin/bash

# Zeta Reticula Full Benchmark Suite
# Reproduces all performance benchmarks published in README.md

set -euo pipefail

RESULTS_DIR="benchmarks/results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
BENCHMARK_RUN="benchmark_run_$TIMESTAMP"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}"
    exit 1
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    if ! command -v cargo >/dev/null 2>&1; then
        error "Rust/Cargo not found. Please install Rust."
    fi
    
    if [ ! -f "target/release/zeta" ]; then
        warning "Zeta CLI not found in release mode. Building..."
        cargo build --release --bin zeta || error "Failed to build Zeta CLI"
    fi
    
    if [ ! -d "models" ] || [ ! -f "benchmarks/prompts_1000.txt" ]; then
        warning "Benchmark models not found. Downloading..."
        ./scripts/download_benchmark_models.sh || error "Failed to download benchmark models"
    fi
    
    success "Prerequisites check complete"
}

# System information
collect_system_info() {
    log "Collecting system information..."
    
    mkdir -p "$RESULTS_DIR/$BENCHMARK_RUN"
    
    cat > "$RESULTS_DIR/$BENCHMARK_RUN/system_info.json" << EOF
{
    "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "hostname": "$(hostname)",
    "os": "$(uname -s)",
    "kernel": "$(uname -r)",
    "architecture": "$(uname -m)",
    "cpu_info": "$(lscpu | grep 'Model name' | cut -d: -f2 | xargs || echo 'N/A')",
    "memory_gb": "$(free -g | awk '/^Mem:/{print $2}' || echo 'N/A')",
    "gpu_info": "$(nvidia-smi --query-gpu=name --format=csv,noheader,nounits 2>/dev/null || echo 'N/A')",
    "rust_version": "$(rustc --version)",
    "zeta_version": "$(./target/release/zeta --version 2>/dev/null || echo '1.0.0')"
}
EOF
    
    success "System info collected"
}

# Latency benchmarks
run_latency_benchmarks() {
    log "Running latency benchmarks..."
    
    local models=("llama-2-7b.safetensors")
    local precisions=("int8" "int4" "fp16")
    
    for model in "${models[@]}"; do
        for precision in "${precisions[@]}"; do
            log "Benchmarking $model with $precision precision..."
            
            ./target/release/zeta infer benchmark \
                --model "models/$model" \
                --iterations 1000 \
                --warmup 50 \
                --precision "$precision" \
                --output "$RESULTS_DIR/$BENCHMARK_RUN/latency_${model%.*}_${precision}.json" \
                2>&1 | tee "$RESULTS_DIR/$BENCHMARK_RUN/latency_${model%.*}_${precision}.log"
            
            success "Completed latency benchmark for $model ($precision)"
        done
    done
}

# Throughput benchmarks
run_throughput_benchmarks() {
    log "Running throughput benchmarks..."
    
    local batch_sizes=(1 8 16 32)
    local precisions=("int8" "fp16")
    
    for batch_size in "${batch_sizes[@]}"; do
        for precision in "${precisions[@]}"; do
            log "Benchmarking throughput with batch size $batch_size, precision $precision..."
            
            ./target/release/zeta infer batch \
                --model "models/llama-2-7b.safetensors" \
                --input-file "benchmarks/prompts_1000.txt" \
                --output-file "$RESULTS_DIR/$BENCHMARK_RUN/throughput_batch${batch_size}_${precision}.txt" \
                --batch-size "$batch_size" \
                --precision "$precision" \
                2>&1 | tee "$RESULTS_DIR/$BENCHMARK_RUN/throughput_batch${batch_size}_${precision}.log"
            
            success "Completed throughput benchmark (batch: $batch_size, precision: $precision)"
        done
    done
}

# Memory benchmarks
run_memory_benchmarks() {
    log "Running memory benchmarks..."
    
    local precisions=("fp32" "fp16" "int8" "int4" "int2" "int1")
    
    for precision in "${precisions[@]}"; do
        log "Analyzing memory usage for $precision precision..."
        
        ./target/release/zeta quantize validate \
            --model "models/llama-2-7b.safetensors" \
            --precision "$precision" \
            --memory-profile \
            --output "$RESULTS_DIR/$BENCHMARK_RUN/memory_${precision}.json" \
            2>&1 | tee "$RESULTS_DIR/$BENCHMARK_RUN/memory_${precision}.log"
        
        success "Completed memory analysis for $precision"
    done
}

# Salience optimization benchmarks
run_salience_benchmarks() {
    log "Running salience optimization benchmarks..."
    
    local thresholds=("0.9" "0.8" "0.7" "0.6")
    
    for threshold in "${thresholds[@]}"; do
        log "Testing salience threshold $threshold..."
        
        ./target/release/zeta salience analyze \
            --input "$(head -n 10 benchmarks/prompts_1000.txt | tr '\n' ' ')" \
            --threshold "$threshold" \
            --output "$RESULTS_DIR/$BENCHMARK_RUN/salience_${threshold}.json" \
            2>&1 | tee "$RESULTS_DIR/$BENCHMARK_RUN/salience_${threshold}.log"
        
        success "Completed salience benchmark for threshold $threshold"
    done
}

# Cache efficiency benchmarks
run_cache_benchmarks() {
    log "Running cache efficiency benchmarks..."
    
    local policies=("lru" "lfu" "salience-based")
    
    for policy in "${policies[@]}"; do
        log "Testing cache policy: $policy..."
        
        ./target/release/zeta cache config --eviction-policy "$policy" --max-size 10000
        
        # Run inference with cache
        ./target/release/zeta infer batch \
            --model "models/llama-2-7b.safetensors" \
            --input-file "benchmarks/prompts_1000.txt" \
            --output-file "$RESULTS_DIR/$BENCHMARK_RUN/cache_${policy}.txt" \
            --use-cache \
            2>&1 | tee "$RESULTS_DIR/$BENCHMARK_RUN/cache_${policy}.log"
        
        # Export cache stats
        ./target/release/zeta cache stats > "$RESULTS_DIR/$BENCHMARK_RUN/cache_stats_${policy}.txt"
        
        success "Completed cache benchmark for $policy"
    done
}

# Cost analysis
run_cost_analysis() {
    log "Running cost analysis..."
    
    ./target/release/zeta system cost-analysis \
        --benchmark-results "$RESULTS_DIR/$BENCHMARK_RUN" \
        --cloud-provider aws \
        --region us-west-2 \
        --output "$RESULTS_DIR/$BENCHMARK_RUN/cost_analysis.json" \
        2>&1 | tee "$RESULTS_DIR/$BENCHMARK_RUN/cost_analysis.log"
    
    success "Cost analysis complete"
}

# Generate summary report
generate_summary() {
    log "Generating benchmark summary..."
    
    cat > "$RESULTS_DIR/$BENCHMARK_RUN/README.md" << EOF
# Zeta Reticula Benchmark Results

**Run ID:** $BENCHMARK_RUN  
**Timestamp:** $(date -u +"%Y-%m-%d %H:%M:%S UTC")  
**System:** $(hostname)

## Files Generated

- \`system_info.json\` - System specifications
- \`latency_*.json\` - Latency benchmark results
- \`throughput_*.txt\` - Throughput benchmark results  
- \`memory_*.json\` - Memory usage analysis
- \`salience_*.json\` - Salience optimization results
- \`cache_*.txt\` - Cache efficiency results
- \`cost_analysis.json\` - Cost savings analysis

## Reproduction

To reproduce these results:

\`\`\`bash
./scripts/run_full_benchmarks.sh
\`\`\`

## Analysis

Compare results with published benchmarks in the main README.md.
Results may vary based on hardware configuration and system load.
EOF
    
    success "Summary report generated"
}

# Main execution
main() {
    log "Starting Zeta Reticula benchmark suite..."
    log "Results will be saved to: $RESULTS_DIR/$BENCHMARK_RUN"
    
    check_prerequisites
    collect_system_info
    run_latency_benchmarks
    run_throughput_benchmarks
    run_memory_benchmarks
    run_salience_benchmarks
    run_cache_benchmarks
    run_cost_analysis
    generate_summary
    
    success "Benchmark suite completed successfully!"
    log "Results available in: $RESULTS_DIR/$BENCHMARK_RUN"
    log "View summary: cat $RESULTS_DIR/$BENCHMARK_RUN/README.md"
}

# Run main function
main "$@"
