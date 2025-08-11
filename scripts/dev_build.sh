#!/bin/bash
set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Function to print section headers
section() {
    echo -e "\n${GREEN}=== $1 ===${NC}"
}

# Configuration
BUILD_MODE="${1:-release}"  # Default to release mode
TARGET="$(uname -m)-apple-darwin"
FEATURES="${FEATURES:-default}"
PARALLEL_JOBS=$(sysctl -n hw.ncpu 2>/dev/null || echo 4)

# Export build settings
export RUSTFLAGS="-C target-cpu=native"
export CARGO_BUILD_JOBS=$PARALLEL_JOBS
export CARGO_INCREMENTAL=1

# Build function
build_crate() {
    local crate=$1
    section "Building $crate"
    
    if [ -d "$crate" ]; then
        (
            cd "$crate"
            echo "Building in $(pwd)"
            if [ "$BUILD_MODE" = "release" ]; then
                cargo build --release --features "$FEATURES"
            else
                cargo build --features "$FEATURES"
            fi
        )
    else
        echo -e "${YELLOW}Warning: $crate not found, skipping...${NC}"
    fi
}

# Main build process
section "Starting build process"
echo "Build mode: $BUILD_MODE"
echo "Target: $TARGET"
echo "Features: $FEATURES"
echo "Parallel jobs: $PARALLEL_JOBS"

# Build in dependency order
build_crate "kvquant-rs"
build_crate "llm-rs"
build_crate "agentflow-rs"
build_crate "quantize-cli"

section "Build complete"
echo -e "${GREEN}✓ Build completed successfully!${NC}"
