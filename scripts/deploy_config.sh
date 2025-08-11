# Docker configuration
export DOCKER_IMAGE="zetareticula/zeta-reticula"
export DOCKER_TAG="latest"

# Registry configuration
export PUSH_TO_REGISTRY=false

# Kubernetes deployment configuration
export DEPLOY_K8S=true
export K8S_NAMESPACE="default"

# Resource limits
export CPU_LIMIT="2000m"
export MEMORY_LIMIT="4Gi"

# Environment variables
export RUST_LOG="info"
export RUST_BACKTRACE="1"

# Feature flags
export ENABLE_CUDA=false
export ENABLE_OPENCL=false
export ENABLE_MKL=true

# Performance tuning
export RAYON_NUM_THREADS="$(nproc)"
export TOKENIZERS_PARALLELISM="true"

# Model configuration
export MODEL_PATH="/models/llama-2-7b"
export QUANTIZATION_BITS=4

# API configuration
export API_HOST="0.0.0.0"
export API_PORT=8080

# Monitoring
export ENABLE_METRICS=true
export METRICS_PORT=9090

# Logging
export LOG_LEVEL="info"
export LOG_FORMAT="json"
