[package]
name = "zeta-reticula"
version = "0.1.0"
edition = "2021"
description = "A k8s-native library for distributed AI inference and storage in Zeta Reticula"
license = "Apache-2.0"
repository = "https://github.com/xAI/zeta-reticula"

[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
thiserror = "1"
log = "0.4"
parking_lot = "0.12"
uuid = { version = "1", features = ["v4"] }
rand = "0.8"
ndarray = "0.15"
chrono = "0.4"

[dependencies.p2pstore]
path = "src/p2pstore"
version = "0.1.0"

[dependencies.zeta-vault-synergy]
path = "src/zeta_vault_synergy"
version = "0.1.0"

[dependencies.client]
path = "src/client"
version = "0.1.0"

[dependencies.master-service]
path = "src/master_service"
version = "0.1.0"

[dependencies.attention-store]
path = "src/attention_store"
version = "0.1.0"

[dependencies.llm-rs]
path = "src/llm_rs"
version = "0.1.0"

[dependencies.quantize]
path = "src/quantize"
version = "0.1.0"

[dependencies.agentflow]
path = "src/agentflow"
version = "0.1.0"#!/bin/bash
set -e

# Change to the project root
cd "$(dirname "$0")"

# Build shared first
echo "Building shared..."
cargo build -p shared

# Build kvquant-rs next
echo "Building kvquant-rs..."
cargo build -p kvquant-rs

# Build ns-router-rs
echo "Building ns-router-rs..."
# Temporarily modify ns-router-rs to not depend on salience-engine
if grep -q 'salience-engine' ns-router-rs/Cargo.toml; then
    sed -i.bak 's/^salience-engine/# salience-engine/' ns-router-rs/Cargo.toml
    cargo build -p ns-router-rs
    mv ns-router-rs/Cargo.toml.bak ns-router-rs/Cargo.toml  # Restore original
else
    cargo build -p ns-router-rs
fi

# Build llm-rs
echo "Building llm-rs..."
cargo build -p llm-rs

# Build salience-engine
echo "Building salience-engine..."
cargo build -p salience-engine

echo "All crates built successfully!"
