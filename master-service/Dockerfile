# Stage 1: Build environment
FROM --platform=linux/amd64 rust:1.82-slim-bookworm AS builder

# Install build dependencies
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

WORKDIR /app

# Copy only the necessary files for building
COPY Cargo.toml Cargo.lock build.rs ./
COPY src/ ./src/
COPY proto/ ./proto/

# Build the application
RUN cargo build --release && \
    mv target/release/master-service /master-service

# Stage 2: Runtime image
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder
COPY --from=builder /master-service /usr/local/bin/

# Default environment variables
ENV RUST_LOG=info
ENV BIND_ADDR=0.0.0.0:50051
ENV NODE_TIMEOUT_SECONDS=300

# Expose gRPC port
EXPOSE 50051

# Health check (using gRPC health check)
HEALTHCHECK --interval=30s --timeout=3s \
  CMD grpc_health_probe -addr=:50051 || exit 1

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/master-service"]
