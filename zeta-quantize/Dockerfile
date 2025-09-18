# Multi-stage build for optimized production image
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Copy local dependencies
COPY ../ns-router-rs/ ../ns-router-rs/
COPY ../agentflow-rs/ ../agentflow-rs/
COPY ../salience-engine/ ../salience-engine/
COPY ../kvquant_rs/ ../kvquant_rs/
COPY ../shared/ ../shared/

# Build the application in release mode
RUN cargo build --release --bin zeta-quantize

# Production stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false -m -d /app quantize

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/zeta-quantize /usr/local/bin/zeta-quantize

# Create directories for models and output
RUN mkdir -p /app/models /app/output && \
    chown -R quantize:quantize /app

# Switch to non-root user
USER quantize

# Set environment variables
ENV RUST_LOG=info
ENV MODEL_PATH=/app/models
ENV OUTPUT_PATH=/app/output

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD zeta-quantize validate --model-path /app/models/test.safetensors || exit 1

# Default command
ENTRYPOINT ["zeta-quantize"]
CMD ["--help"]
