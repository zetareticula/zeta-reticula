# Master Service

The Master Service is a critical component of the Zeta Reticula distributed AI system. It serves as the central coordinator for managing worker nodes, handling service discovery, and maintaining cluster state.

## Features

- **Service Discovery**: Track and manage worker nodes in the cluster
- **Health Monitoring**: Monitor node health with configurable timeouts
- **Load Balancing**: Distribute work across available nodes
- **High Availability**: Designed for fault tolerance and high availability
- **gRPC API**: Efficient communication using Protocol Buffers and gRPC

## Prerequisites

- Rust 1.60+ (for building from source)
- Docker & Docker Compose (for containerized deployment)
- protoc (for protocol buffer compilation)

## Development

### Prerequisites

- Rust 1.60+ (for building from source)
- protobuf-compiler (for protocol buffer compilation)
- Docker & Docker Compose (for containerized development)

### Development Workflow

#### Using Makefile (Recommended)

A `Makefile` is provided to simplify common development tasks:

```bash
# Install development dependencies
make install-deps

# Build the project
make build

# Run tests
make test

# Run the service locally
make run

# In another terminal, check service health
./healthcheck.sh --verbose

# Build Docker image
make docker-build

# Run Docker container
make docker-run

# Deploy to staging
make deploy-staging

# Deploy to production (with confirmation)
make deploy-prod

# Check code style and formatting
make check-style check-format

# Format code
make format

# Clean build artifacts
make clean

# Show available commands
make help
```

#### Using Development Scripts

Alternatively, you can use the individual development scripts:

```bash
# Make the scripts executable
chmod +x dev.sh healthcheck.sh

# Build the project
./dev.sh build

# Run tests
./dev.sh test

# Run the service locally
./dev.sh run

# Check service health
./healthcheck.sh --verbose

# Build Docker image
./dev.sh docker-build

# Run Docker container
./dev.sh docker-run
```

### Health Check

The health check script verifies that the master service is running and responding to gRPC health checks:

```bash
# Basic health check (default: localhost:50051)
./healthcheck.sh

# Custom service address
./healthcheck.sh --address myservice:50051

# Increase timeout and set check interval
./healthcheck.sh --timeout 120 --interval 10

# Verbose output
./healthcheck.sh --verbose

# Show help
./healthcheck.sh --help
```

The health check script will:
1. Check if `grpcurl` is installed and install it if missing (on Linux/macOS)
2. Poll the gRPC health check endpoint
3. Exit with status 0 when the service is healthy
4. Exit with status 1 if the service doesn't become healthy within the timeout period

### Local Build

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt-get update && sudo apt-get install -y protobuf-compiler

# Build the project
cargo build --release
```

### Docker Build

```bash
docker build -t zeta-reticula/master-service:latest .
```

## Running

### Local Execution

```bash
# Set environment variables
export RUST_LOG=info
export BIND_ADDR=0.0.0.0:50051
export NODE_TIMEOUT_SECONDS=300

# Run the service
cargo run --release
```

### Docker Compose

```bash
docker-compose up -d
```

## Kubernetes Deployment

The master service can be deployed to a Kubernetes cluster using the provided Kubernetes manifests and deployment script.

### Prerequisites

- Kubernetes cluster (v1.19+)
- `kubectl` configured to communicate with your cluster
- `kustomize` (included with `kubectl` v1.14+)
- Docker or another container runtime
- Container registry (e.g., Docker Hub, ECR, GCR)

### Deployment Script

A deployment script is provided to simplify the deployment process:

```bash
# Make the script executable
chmod +x deploy.sh

# Deploy to staging environment (default)
./deploy.sh --environment staging --tag latest-staging

# Deploy to production
./deploy.sh --environment production --tag v1.0.0

# Custom registry and image tag
./deploy.sh --environment production --tag v1.0.0 --registry your-registry.com/your-org

# Skip build and push (for CI/CD)
./deploy.sh --environment production --tag v1.0.0 --skip-build --skip-push
```

### Manual Deployment

If you prefer to deploy manually:

```bash
# Build and push the Docker image
docker build -t your-registry.com/your-org/master-service:tag .
docker push your-registry.com/your-org/master-service:tag

# Apply Kubernetes manifests
kubectl apply -k k8s/overlays/staging  # For staging
# or
kubectl apply -k k8s/overlays/production  # For production

# Verify the deployment
kubectl get pods -n <namespace>
kubectl logs -f deployment/master-service -n <namespace>
```

### Environment Configuration

The following environment variables can be configured:

| Variable | Description | Default |
|----------|-------------|---------|
| `BIND_ADDR` | Address and port to bind the gRPC server to | `0.0.0.0:50051` |
| `NODE_TIMEOUT_SECONDS` | Seconds before an inactive node is removed | `300` |
| `RUST_LOG` | Logging level (error, warn, info, debug, trace) | `info` |
| `GRPC_MAX_CONNECTION_AGE` | Maximum connection age in seconds | `3600` |
| `GRPC_MAX_CONNECTION_AGE_GRACE` | Grace period for connection draining | `300` |
| `METRICS_ENABLED` | Enable Prometheus metrics endpoint | `false` |
| `METRICS_PORT` | Port for metrics endpoint | `9090` |

## Configuration

| Environment Variable    | Default Value    | Description                                  |
|-------------------------|------------------|----------------------------------------------|
| `BIND_ADDR`            | `0.0.0.0:50051`  | Address and port to bind the gRPC server to  |
| `NODE_TIMEOUT_SECONDS`  | `300`            | Seconds before an inactive node is removed   |
| `RUST_LOG`             | `info`           | Logging level (error, warn, info, debug, trace) |

## API Documentation

The Master Service exposes the following gRPC endpoints:

### Register

Register a new node with the master service.

```protobuf
rpc Register(RegisterRequest) returns (RegisterResponse);

message RegisterRequest {
  string node_id = 1;
  map<string, string> metadata = 2;
}

message RegisterResponse {
  string node_id = 1;
}
```

### Heartbeat

Send a heartbeat to indicate the node is still alive.

```protobuf
rpc Heartbeat(HeartbeatRequest) returns (HeartbeatResponse);

message HeartbeatRequest {
  string node_id = 1;
}

message HeartbeatResponse {
  bool success = 1;
}
```

### GetNodes

Get information about all registered nodes.

```protobuf
rpc GetNodes(GetNodesRequest) returns (GetNodesResponse);

message GetNodesRequest {}

message GetNodesResponse {
  repeated NodeInfo nodes = 1;
}

message NodeInfo {
  string id = 1;
  int64 last_seen = 2;
  map<string, string> metadata = 3;
}
```

## Monitoring

The service exposes Prometheus metrics on `/metrics` (HTTP) and supports gRPC health checks.

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
