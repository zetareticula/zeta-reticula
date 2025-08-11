# Optimized Build and Deployment

This document outlines the optimized build and deployment process for Zeta Reticula, focusing on performance, efficiency, and reliability.

## Prerequisites

- Rust (latest stable)
- Docker and Docker Compose
- Kubernetes (optional, for cluster deployment)
- UPX (for binary compression)

## Quick Start

1. **Build the project**:
   ```bash
   ./scripts/optimized_build.sh
   ```

2. **Deploy using Docker**:
   ```bash
   docker build -f Dockerfile.optimized -t zeta-reticula:latest .
   docker run -p 8080:8080 zeta-reticula:latest
   ```

3. **Or use the deployment script**:
   ```bash
   ./scripts/deploy.sh
   ```

## Build Configuration

Edit `scripts/build_config.toml` to customize build settings:

```toml
[build]
mode = "release"
target = "x86_64-unknown-linux-gnu"
parallel_jobs = 0  # 0 = auto-detect
opt_level = 3
lto = true
```

## Deployment Configuration

Edit `scripts/deploy_config.sh` to configure deployment settings:

```bash
# Docker configuration
export DOCKER_IMAGE="zetareticula/zeta-reticula"
export DOCKER_TAG="latest"

# Kubernetes
export DEPLOY_K8S=true
export K8S_NAMESPACE="default"
```

## Performance Tuning

### Environment Variables

- `RAYON_NUM_THREADS`: Controls the number of threads used by Rayon
- `TOKENIZERS_PARALLELISM`: Enables parallel tokenization
- `RUST_LOG`: Controls logging verbosity

### Feature Flags

Enable/disable features in `scripts/build_config.toml`:

```toml
[features]
cuda = false
opencl = false
mkl = true
```

## Monitoring

Metrics are exposed on port 9090 when `ENABLE_METRICS=true`.

## Troubleshooting

1. **Build failures**:
   - Ensure all dependencies are installed
   - Check the Rust version (`rustc --version`)
   - Clean the target directory: `cargo clean`

2. **Performance issues**:
   - Enable CPU-specific optimizations in `build_config.toml`
   - Adjust resource limits in `deploy_config.sh`

## License

Apache 2.0
