#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Running integration tests...${NC}"

# Build all required components
echo "Building components..."
cargo build --all-targets

# Run the integration test
echo -e "\nRunning attention-vault integration test..."
RUST_LOG=debug cargo test --test integration_test -- --nocapture

if [ $? -eq 0 ]; then
    echo -e "\n${GREEN}All integration tests passed!${NC}"
else
    echo -e "\n${RED}Integration tests failed${NC}"
    exit 1
fi
