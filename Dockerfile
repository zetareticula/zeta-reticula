# Stage 1: Build environment
FROM --platform=linux/amd64 rust:1.78-slim-bullseye as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Create a dummy project to cache dependencies
RUN USER=root cargo new --bin zeta-reticula
WORKDIR /app/zeta-reticula

# Copy build files
COPY Cargo.toml Cargo.lock ./

# Build dependencies (cached unless Cargo.toml changes)
RUN cargo build --release \
    && find target -name "*.rlib" -delete \
    && find target -name "*.d" -delete

# Copy source code
COPY . .

# Build the application
RUN cargo build --release --bins

# Stage 2: Runtime image
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder
COPY --from=builder /app/zeta-reticula/target/release/salience-engine /usr/local/bin/

# Default command (can be overridden)
CMD ["salience-engine"]