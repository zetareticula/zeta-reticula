## 🆕 Recent Updates (Sep 2025)

- Client crate
  - Introduced a local `mooncake` gRPC stub in `client/src/client.rs` to unblock builds without generated protobufs. This will be swapped for real generated types in a future step.
  - Made `Client::new` async and wrapped the internal gRPC client with `RwLock` for safe concurrent access.
  - Exposed helpers used by the ping task: `ping_master()`, `reconnect_master()`, `client_id()`, and `is_running()`.

- Quantize CLI
  - Removed legacy duplicate CLI implementation and unified on `src/app.rs` + `src/cli/mod.rs`.
  - Updated to current APIs: `NSRouter::new()`, `MesolimbicSystem::with_config(Default::default())`, and `KVQuantizer::new(KVQuantConfig::default())`.

- CI/CD
  - Enhanced `.github/workflows/ci-cd.yaml` to:
    - Run `cargo fmt --check`, `cargo clippy -D warnings`, full workspace build and tests.
    - Include additional crates in path filters: `client/`, `attention-store/`, and `p2pstore/`.
  - Existing Docker build and EKS deploy stages preserved.

> Note: `attention-store` is being refactored to remove serialization of `Instant`, avoid non-object-safe async traits, and decouple segment management. Remaining work will land in follow-up commits.

## 🧑‍💻 Developer Notes

- Full workspace build
  - `cargo build --workspace --all-features`
  - `cargo test --workspace --all-features`

- Linting & formatting
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-features -- -D warnings`

<div align="center">
  <a href="https://github.com/zetareticula/zeta-reticula">
    <img src="assets/blob.png" alt="Zeta Reticula Logo" width="400">
  </a>
  
  <h1>Zeta Reticula</h1>
  
  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
  [![Rust](https://github.com/zetareticula/zeta-reticula/actions/workflows/rust.yml/badge.svg)](https://github.com/zetareticula/zeta-reticula/actions)
  [![Docker](https://img.shields.io/docker/pulls/zetareticula/salience-engine)](https://hub.docker.com/r/zetareticula/salience-engine)
  [![Crates.io](https://img.shields.io/crates/v/llm-rs)](https://crates.io/crates/llm-rs)
  [![Documentation](https://docs.rs/llm-rs/badge.svg)](https://docs.rs/llm-rs)
</div>

> "Precision-engineered intelligence for the next generation of AI applications."

## 🚀 Overview

Zeta Reticula is a high-performance, open-source framework for optimizing large language model (LLM) inference through advanced quantization techniques. Built in Rust for maximum performance and safety, it provides fine-grained control over numerical precision to balance model accuracy, memory usage, and computational efficiency.

### System Architecture

Zeta Reticula follows a modular, microservices-based architecture with the following core components:

1. **AgentFlow-RS**: Core orchestration and workflow management
2. **Attention-Store**: Manages attention mechanisms and KV cache
3. **KVQuant-RS**: Handles model quantization and optimization
4. **LLM-RS**: Core language model inference engine
5. **API Layer**: Next.js-based REST API for model serving
6. **Sidecar Services**: gRPC services for low-level operations

## ✨ Features

### 🎯 Core Capabilities
- **Multiple Precision Levels**: 1-bit, 2-bit, 4-bit, 8-bit, 16-bit (fp16), and 32-bit (fp32) support
- **Dynamic Quantization**: On-the-fly precision adjustment based on model requirements
- **Salience-Based Processing**: Intelligent token prioritization for efficient inference
- **Model Parallelism**: Distributed model execution across multiple devices
- **Hardware Acceleration**: Optimized for modern CPUs and GPUs (NVIDIA/AMD/Intel)
- **Memory Efficiency**: Up to 32x memory reduction with minimal accuracy loss
- **Low-Latency Inference**: Sub-millisecond token generation for real-time applications

### 🛠️ Advanced Features
- **Attention Management**: Efficient KV cache with layer-wise preloading
- **Role-Based Inference**: Dynamic model routing based on input characteristics
- **Secure Deployment**: mTLS for service communication and RBAC
- **Observability**: Built-in metrics collection and distributed tracing
- **Efficient KV Caching**: Smart eviction policies and distributed caching
- **High Throughput**: Optimized for batch processing and concurrent requests

### 🚀 Performance Characteristics
- **Hardware Acceleration**: Optimized for modern CPUs and GPUs (NVIDIA/AMD/Intel)
- **Memory Efficiency**: Up to 32x memory reduction with minimal accuracy loss
- **Low-Latency Inference**: Sub-millisecond token generation for real-time applications
- **Efficient KV Caching**: Smart eviction policies and distributed caching
- **High Throughput**: Optimized for batch processing and concurrent requests
- **Resource Scaling**: Automatic scaling based on workload demands

### 🛠️ Developer Experience
- **Rust-Powered**: Memory safety without garbage collection
- **Simple API**: Easy integration into existing pipelines
- **Comprehensive Metrics**: Detailed performance and accuracy tracking

## 🛠️ Technical Architecture

### Core Components
- **llm-rs**: Core LLM functionality with support for multiple model architectures
- **kvquant-rs**: Advanced quantization with salience-based processing
- **agentflow-rs**: Workflow orchestration with role-based access control
- **attention-store**: Distributed attention mechanism management
- **distributed-store**: Scalable key-value storage for model parameters

### Infrastructure
- **APIs**: Next.js 13+ with TypeScript for type safety
- **gRPC Services**: High-performance inter-service communication
- **Containerization**: Multi-stage Docker builds for optimized images
## 🚀 Getting Started

### Prerequisites

- Rust toolchain (1.70+)
- Node.js 18+ (for API and web components)
- Docker & Kubernetes (for containerized deployment)
- CUDA Toolkit (for GPU acceleration, optional)
- OpenBLAS or Intel MKL (for CPU acceleration)

### Build Status ✅

**Latest Update (January 2025)**: All compilation errors have been resolved across the entire workspace!

- ✅ **master-service**: Fixed protobuf build and ownership issues
- ✅ **salience-engine**: Resolved type visibility and method signatures
- ✅ **ns-router-rs**: Major fixes to routing logic, symbolic reasoning, and type compatibility
- ✅ **shared**: Added FP16 precision support
- ✅ **quantize-cli**: Modular structure with successful compilation
- ✅ **llm-rs**: Core inference engine building successfully

**Workspace Build**: `cargo build --workspace` ✅ **SUCCESS**

### Quick Start

1. **Clone and Build**
   ```bash
   git clone https://github.com/zetareticula/zeta-reticula.git
   cd zeta-reticula
   cargo build --workspace --release
   ```

2. **Start Services**
   ```bash
   # Start all services in development mode
   docker-compose up -d
   
   # Or deploy to Kubernetes
   kubectl apply -k k8s/overlays/dev
   ```

3. **Verify Installation**
   ```bash
   # Check API health
   curl http://localhost:3000/api/health
   
   # Run tests
   cargo test --all-features
   ```

## 🛠️ Core Components

### AgentFlow-RS
Orchestrates agent workflows and manages the execution pipeline.

```rust
// Example: Initializing AgentFlow
let config = AgentFlowConfig {
    max_concurrent_tasks: 8,
    cache_size_mb: 2048,
    ..Default::default()
};
let agent_flow = initialize_agent_flow(config);
```

### Attention-Store
Manages attention mechanisms and KV cache with efficient storage.

```rust
// Example: Initializing AttentionStore
let attention_store = AttentionStore::new(
    vault,
    transfer_engine,
    client,
    master_service
)?;
```

### KVQuant-RS
Handles model quantization and optimization.

```yaml
# Example: KVQuant Configuration
quantization:
  block_size: 1024
  precision: int8
  use_mixed_precision: true
  salience_threshold: 0.8
```

### LLM-RS
Core language model inference engine with support for multiple model architectures.

## 🚀 Deployment

### Prerequisites

- Kubernetes cluster (v1.24+)
- `kubectl` and `kustomize` installed
- Container registry access
- Sufficient resources (CPU/GPU, memory)

### 1. Initialize Models Directory

```bash
# Initialize models directory with a sample model
chmod +x scripts/init_models.sh
./scripts/init_models.sh
```

### 2. Deploy NS Router

```bash
# Deploy NS Router to Kubernetes
chmod +x scripts/deploy_ns_router.sh
./scripts/deploy_ns_router.sh
```

### 3. Quantize Models

```bash
# Quantize models using kvquant_rs and store in p2pstore
chmod +x scripts/quantize_models.sh
./scripts/quantize_models.sh
```

### 4. Verify Deployment

```bash
# Verify all components are running
chmod +x scripts/verify_deployment.sh
./scripts/verify_deployment.sh
```

### 5. Configure AgentFlow Semaphores

Create or update `agentflow-rs/config/semaphore.toml`:

```toml
[components]
attention_store = { max_concurrent = 5, timeout_secs = 30 }
llm_rs = { max_concurrent = 3, timeout_secs = 60 }
zeta_vault = { max_concurrent = 2, timeout_secs = 120 }
```

## 🔄 Component Integration

### Kubernetes Deployment

#### Prerequisites
- Kubernetes cluster (v1.24+)
- `kubectl` and `kustomize`
- Container registry access
- Sufficient resources (CPU/GPU, memory)

#### Deployment Steps

1. **Configure Environment**
   ```bash
   # Set environment variables
   export NAMESPACE=zeta-reticula
   export REGISTRY=your-registry
   export TAG=latest
   ```

2. **Deploy Dependencies**
   ```bash
   # Create namespace
   kubectl create namespace $NAMESPACE
   
   # Deploy monitoring stack
   helm install prometheus prometheus-community/kube-prometheus-stack \
     -n $NAMESPACE \
     --set prometheus.prometheusSpec.serviceMonitorSelectorNilUsesHelmValues=false
   ```

3. **Deploy Zeta Reticula**
   ```bash
   # Apply base configuration
   kubectl apply -k k8s/base
   
   # Deploy with production settings
   kubectl apply -k k8s/overlays/prod
   ```

### Docker Compose (Development)

```yaml
version: '3.8'

services:
  api:
    build: .
    ports:
      - "3000:3000"
    environment:
      - RUST_LOG=info
    volumes:
      - .:/app
    depends_on:
      - redis
      - postgres

  redis:
    image: redis:alpine
    ports:
      - "6379:6379"

  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_PASSWORD: example
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

## 📊 Performance Tuning

### KV Cache Optimization

```yaml
# config/production.yaml
kv_cache:
  block_size: 1024
  max_blocks: 1024
  eviction_policy: lru
  compression: zstd
```

### Resource Management

```bash
# Monitor resource usage
kubectl top pods -n zeta-reticula

# Adjust resource limits
kubectl edit deployment/api -n zeta-reticula
```

## 🔄 Basic Usage

### Model Quantization

Quantize models to various precision levels:

```bash
cargo run --release --bin quantize-cli -- quantize \
    --input model.bin \
    --output model_quantized.bin \
    --precision int4  # Options: int1, int2, int4, int8, fp16, fp32
```

### Running Inference

Perform inference with optimized models:

```bash
cargo run --release --bin quantize-cli -- infer \
    --model model_quantized.bin \
    --input "Your prompt here" \
    --precision int4  # Match the precision used during quantization
```

### Integration with LLMs

Zeta Reticula supports various open-source LLMs:

```rust
// Example: Using with a custom model
let model = LLMModel::load("path/to/model.bin")?;
let config = InferenceConfig {
    max_tokens: 512,
    temperature: 0.7,
    ..Default::default()
};

let output = model.generate("Your prompt here", &config)?;
println!("Generated: {}", output);
```

### Testing

Run the full test suite:

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_tests -- --nocapture

# Performance benchmarks
cargo bench
```

## 📞 Support

For support, please open an issue or join our [Discord community](https://discord.gg/zetareticula).

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📚 Resources

- [API Documentation](https://docs.zeta-reticula.dev/api)
- [Architecture Guide](docs/ARCHITECTURE.md)
- [Performance Benchmarks](docs/BENCHMARKS.md)
- [Contributing Guide](CONTRIBUTING.md)

## 📊 Monitoring & Observability

### Metrics
Zeta Reticua exposes Prometheus metrics at `/metrics`:
- Request latency
- Error rates
- Resource utilization
- Cache hit/miss ratios

### Logging
Structured JSON logging with the following fields:
- `timestamp`
- `level` (info, warn, error, debug)
- `target` (module path)
- `message`
- `request_id` (for request tracing)

### Distributed Tracing
Supports OpenTelemetry for end-to-end request tracing across services.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

   git clone https://github.com/zetareticula/zeta-reticula.git
   cd zeta-reticula
   cargo build --release
   ```

2. **Run with Docker**
   ```bash
   docker-compose up --build
   ```
   Access the API at `http://localhost:8080`

### 🚀 Production Deployment

#### Kubernetes (Helm)

```bash
# Add Helm repo
helm repo add zeta https://charts.zeta-reticula.ai

# Install chart
helm install zeta zeta/zeta-reticula -n zeta --create-namespace
```

## 📚 Documentation

- [API Reference](https://docs.zeta-reticula.ai/api)
- [Deployment Guide](https://docs.zeta-reticula.ai/deployment)
- [Developer Guide](https://docs.zeta-reticula.ai/development)

## 🤝 Contributing

We welcome contributions! Please read our [Contributing Guide](CONTRIBUTING.md) to get started.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🌐 Community

- [Discord](https://discord.gg/your-invite)
- [Twitter](https://twitter.com/zetareticula)
- [Blog](https://blog.zeta-reticula.ai)

---

<div align="center">
  Made with ❤️ by the Zeta Reticula Team
</div>


   ```

3. **Set Up the Front-End**

   ```bash
   cd app
   npm install
   npm start
   ```

Visit `http://localhost:3000` to explore the dashboard and begin your journey into optimized inference!

### Troubleshooting

#### Docker Build Issues

- **Missing Dependencies**: Ensure all build dependencies are installed in the Dockerfile.
  ```dockerfile
  RUN apt-get update && apt-get install -y \
      pkg-config \
      libssl-dev \
      build-essential \
      cmake \
      curl \
      git \
      clang \
      lld \
      protobuf-compiler \
      libprotobuf-dev \
      && rm -rf /var/lib/apt/lists/*
  ```

- **Rust Version Mismatch**: Ensure the Rust version in the Dockerfile matches the required version for all dependencies.
  ```dockerfile
  FROM --platform=linux/amd64 rust:1.82-slim-bookworm AS builder
  ```

#### Kubernetes Issues

- **Image Pull Errors**: Ensure the image is available in your cluster. For local development, use `kind` to load the image:
  ```bash
  kind load docker-image zeta-salience/salience-engine:local --name your-cluster-name
  ```

- **Service Not Accessible**: Check if the service is running and the ports are correctly exposed:
  ```bash
  kubectl -n zeta get svc,pods
  kubectl -n zeta logs -l app=zeta-reticula,component=salience-engine
  ```

#### Common Build Errors

- **Protoc Not Found**: Ensure `protobuf-compiler` is installed:
  ```bash
  sudo apt-get install -y protobuf-compiler
  ```

- **Rust Toolchain Issues**: Ensure the correct Rust toolchain is installed:
  ```bash
  rustup update
  rustup default stable
  ```

For additional help, please open an issue on our [GitHub repository](https://github.com/your-org/zeta-reticula/issues).

---

## Directory Structure

```
zeta-reticula/
├── app/              # React-based front-end UI/UX
├── api/              # Rust-based API server
├── llm-rs/           # Core inference engine
├── salience-engine/  # Salience-driven quantization
├── ns-router-rs/     # Neural network routing
├── kvquant-rs/       # KV cache quantization
├── quantize-cli/     # Command-line interface
├── agentflow-rs/     # Federated learning framework
├── README.md         # This file
└── LICENSE           # Open-source license (e.g., MIT)
```

---

## Contributing

As we venture into this new epoch of artificial intelligence, we invite bold pioneers to contribute. Fork the repository, submit pull requests, and join our community to shape the future of inference quantization. Issues and feature requests are welcome—let’s build a Time Machine for the mind together!

- **Issues**: Report bugs or suggest enhancements [here](https://github.com/your-org/zeta-reticula/issues).
- **Code Style**: Adhere to Rust and JavaScript best practices.
- **Communication**: Engage with us via our [Discord server](https://discord.gg/your-invite-link).

---

## Roadmap

- **Q3 2025**: Integrate WebSockets for real-time metric streaming.
- **Q4 2025**: Expand support for homomorphic encryption and dynamic client allocation.
- **Q1 2026**: Launch enterprise-grade features like multi-tenant support and advanced visualization tools.

---

## License

This project is licensed under the MIT License—free to use, modify, and distribute, as we propel humanity into the stars of computational innovation.

---

## Contact

Embark on this odyssey with us! Reach out at [karl@zetareticula.com](mailto:karl@zetareticula.com) or follow our journey on [Twitter](https://twitter.com/ZetaReticulaAI).

"Into the abyss of the future we go, where machines dream and humanity ascends!" — H.G. Wells, rekindled.

🌠 **Zeta Reticula: Quantizing the Infinite, Today!** 🌠
