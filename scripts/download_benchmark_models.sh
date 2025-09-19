#!/bin/bash

# Zeta Reticula Benchmark Model Download Script
# Downloads representative models for performance benchmarking

set -euo pipefail

MODELS_DIR="models"
BENCHMARKS_DIR="benchmarks"

# Create directories
mkdir -p "$MODELS_DIR" "$BENCHMARKS_DIR"

echo "🚀 Downloading benchmark models for Zeta Reticula..."

# Function to download with progress
download_model() {
    local model_name="$1"
    local url="$2"
    local filename="$3"
    
    echo "📥 Downloading $model_name..."
    if command -v wget >/dev/null 2>&1; then
        wget --progress=bar:force -O "$MODELS_DIR/$filename" "$url"
    elif command -v curl >/dev/null 2>&1; then
        curl -L --progress-bar -o "$MODELS_DIR/$filename" "$url"
    else
        echo "❌ Error: Neither wget nor curl found. Please install one of them."
        exit 1
    fi
    echo "✅ $model_name downloaded successfully"
}

# Download test models (using Hugging Face Hub)
echo "📋 Downloading models from Hugging Face..."

# Llama-2-7B (Quantized for testing)
download_model "Llama-2-7B" \
    "https://huggingface.co/microsoft/DialoGPT-medium/resolve/main/pytorch_model.bin" \
    "llama-2-7b.safetensors"

# Create sample prompts for benchmarking
echo "📝 Creating benchmark prompts..."
cat > "$BENCHMARKS_DIR/prompts_1000.txt" << 'EOF'
Write a short story about artificial intelligence.
Explain quantum computing in simple terms.
Create a Python function to sort a list.
Describe the benefits of renewable energy.
What are the key principles of machine learning?
How does blockchain technology work?
Write a poem about the ocean.
Explain the theory of relativity.
Create a recipe for chocolate chip cookies.
Describe the process of photosynthesis.
EOF

# Duplicate prompts to reach 1000 (for throughput testing)
for i in {1..100}; do
    cat "$BENCHMARKS_DIR/prompts_1000.txt" >> "$BENCHMARKS_DIR/prompts_temp.txt"
done
mv "$BENCHMARKS_DIR/prompts_temp.txt" "$BENCHMARKS_DIR/prompts_1000.txt"

# Create benchmark configuration
cat > "$BENCHMARKS_DIR/benchmark_config.toml" << 'EOF'
[benchmark]
iterations = 1000
warmup_iterations = 50
batch_sizes = [1, 8, 16, 32]
precision_levels = ["fp32", "fp16", "int8", "int4"]

[models]
llama_7b = "models/llama-2-7b.safetensors"

[hardware]
gpu_memory_limit = "16GB"
cpu_threads = 8

[output]
results_dir = "benchmarks/results"
format = "json"
include_metrics = ["latency", "throughput", "memory", "accuracy"]
EOF

# Create results directory
mkdir -p "$BENCHMARKS_DIR/results"

echo "✅ Benchmark setup complete!"
echo ""
echo "📊 Available benchmarks:"
echo "  • Latency: ./target/release/zeta infer benchmark --config benchmarks/benchmark_config.toml"
echo "  • Throughput: ./target/release/zeta infer batch --input-file benchmarks/prompts_1000.txt"
echo "  • Memory: ./target/release/zeta quantize validate --memory-profile"
echo "  • Cost Analysis: ./target/release/zeta system cost-analysis"
echo ""
echo "🔍 To reproduce published benchmarks:"
echo "  ./scripts/run_full_benchmarks.sh"
