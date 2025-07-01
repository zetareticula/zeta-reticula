#!/bin/bash
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
