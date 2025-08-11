#!/bin/bash
set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print section headers
section() {
    echo -e "\n${GREEN}=== $1 ===${NC}"
}

# Detect system configuration
section "Detecting system configuration"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
CPU_CORES=$(sysctl -n hw.ncpu 2>/dev/null || echo 4)

echo "OS: $OS"
echo "Architecture: $ARCH"
echo "CPU Cores: $CPU_CORES"

# Configuration
BUILD_MODE="${BUILD_MODE:-release}"
TARGET="${TARGET:-$ARCH-apple-darwin}"
FEATURES="${FEATURES:-default}"

# Set up build environment
export RUSTFLAGS="-C target-cpu=native -C codegen-units=1"
export CARGO_BUILD_JOBS=$CPU_CORES
export CARGO_INCREMENTAL=1

# Function to build a single crate
build_crate() {
    local crate=$1
    section "Building $crate"
    
    (
        cd "$crate"
        cargo build --target "$TARGET" --features "$FEATURES" --$BUILD_MODE
    )
}

# Function to build all crates
build_all() {
    section "Building all crates in workspace"
    
    # Build in dependency order
    local crates=(
        "protos"
        "shared"
        "attention-store"
        "kvquant-rs"
        "llm-rs"
        "ns-router-rs"
        "salience-engine"
        "agentflow-rs"
        "quantize-cli"
        "api"
    )
    
    for crate in "${crates[@]}"; do
        if [ -d "$crate" ]; then
            build_crate "$crate"
        fi
    done
}

# Main build process
section "Starting build process"

if [ $# -gt 0 ]; then
    # Build specific crates
    for crate in "$@"; do
        if [ -d "$crate" ]; then
            build_crate "$crate"
        else
            echo -e "${YELLOW}Warning: Crate '$crate' not found${NC}"
        fi
    done
else
    # Build all crates
    build_all
fi

section "Build complete"
echo -e "${GREEN}✓ Build completed successfully!${NC}"
