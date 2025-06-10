# Base image for Rust
FROM rust:1.78 AS builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies (cache optimization)
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Copy source code
COPY . .

# Build the application
RUN cargo build --release --bin <service-name> # Replace <service-name> with actual binary (e.g., llm-rs)

# Runtime image
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/<service-name> /usr/local/bin/<service-name>

CMD ["<service-name>"]