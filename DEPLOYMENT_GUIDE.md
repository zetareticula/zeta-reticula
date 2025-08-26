# Zeta Reticula Deployment Guide

This guide provides step-by-step instructions for deploying and running the Zeta Reticula system, including the salience-engine, distributed-store, kvquant, llm-rs, and other components.

## Prerequisites

- Rust (latest stable version)
- Cargo (Rust's package manager)
- Docker and Docker Compose
- Python 3.8+ (for some utilities)
- At least 16GB RAM (32GB recommended for production)
- NVIDIA GPU with CUDA support (recommended for optimal performance)

## Quick Start

1. **Clone the repository**
   ```bash
   git clone https://github.com/zetareticula/zeta-reticula.git
   cd zeta-reticula
   ```

2. **Set up environment variables**
   ```bash
   cp .env.example .env
   # Edit the .env file with your configuration
   ```

3. **Build all components**
   ```bash
   ./scripts/build.sh
   ```

4. **Run the system**
   ```bash
   ./run.sh
   ```

## Component Details

### 1. Distributed Store with KV-Cache

The distributed store provides a distributed key-value store with caching capabilities.

**Configuration**: `distributed-store/config.toml`

**Key Features**:
- Distributed key-value storage
- Raft consensus protocol
- Sharding and replication
- Monitoring and metrics

### 2. Master Service

The master service coordinates between different components and manages the cluster.

**Configuration**: `master-service/config.toml`

**Key Features**:
- Service discovery
- Load balancing
- Health checking
- Metrics collection

### 3. AgentFlow

AgentFlow manages the execution of AI agents and workflows.

**Configuration**: `agentflow-rs/config.toml`

**Key Features**:
- Agent lifecycle management
- Workflow orchestration
- Resource allocation
- Performance monitoring

### 4. Salience Engine

The salience engine handles model inference and KV-cache management.

**Configuration**: `salience-engine/config.toml`

**Key Features**:
- Model inference with llm-rs
- KV-cache optimization
- Distributed inference
- Performance optimization

### 5. KVQuant

KVQuant provides quantization for the KV-cache to reduce memory usage.

**Configuration**: Through environment variables and runtime parameters

**Key Features**:
- 4-bit and 8-bit quantization
- Group-wise quantization
- Activation order preservation

## Advanced Configuration

### Model Configuration

1. Place your model files in the `models` directory
2. Update the `MODEL_PATH` in `.env`
3. Configure model-specific parameters in the respective component configs

### Scaling

To scale the system:

1. **Horizontal Scaling**:
   - Add more nodes to the distributed store
   - Deploy multiple instances of the salience engine
   - Use a load balancer in front of the services

2. **Vertical Scaling**:
   - Increase CPU/memory allocation
   - Use GPUs for model inference
   - Tune cache sizes and batch sizes

## Monitoring and Logging

- **Metrics**: Available on ports 9100-9102 (Prometheus format)
- **Logs**: Stored in the `logs` directory
- **Health Checks**:
  - Distributed Store: `http://localhost:50051/health`
  - Master Service: `http://localhost:50052/health`
  - AgentFlow: `http://localhost:50053/health`
  - Salience Engine: `http://localhost:50054/health`

## Troubleshooting

### Common Issues

1. **Port Conflicts**:
   - Check if the required ports are available
   - Update the configuration files if needed

2. **Model Loading Issues**:
   - Verify the model path and permissions
   - Check for missing model files
   - Ensure the model format is supported

3. **Performance Problems**:
   - Check system resource usage
   - Tune batch sizes and cache settings
   - Enable GPU acceleration if available

## Production Deployment

For production deployments, consider:

1. **Security**:
   - Enable TLS for all services
   - Set up authentication and authorization
   - Use secure API keys

2. **High Availability**:
   - Deploy multiple instances of each service
   - Use a service mesh for service discovery
   - Implement proper backup and recovery procedures

3. **Monitoring**:
   - Set up Prometheus and Grafana for monitoring
   - Configure alerts for critical metrics
   - Log aggregation with ELK or similar

## Support

For support, please open an issue on GitHub or join our community forum.
