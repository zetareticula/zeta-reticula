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

WORKDIR /app/zeta-reticula

# Copy full workspace (required so Cargo can resolve workspace members)
COPY . .

# Build only the salience-engine package (avoid other heavy deps)
RUN cargo build --release -p salience-engine --features server --bin salience-engine \
    && find target -name "*.rlib" -delete \
    && find target -name "*.d" -delete

# Stage 2: Runtime image
FROM debian:bullseye-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the salience-engine binary from the builder
COPY --from=builder /app/zeta-reticula/target/release/salience-engine /usr/local/bin/

# Default command (can be overridden)
ENV PORT=8080
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/salience-engine"]