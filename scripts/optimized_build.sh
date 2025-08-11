#!/bin/bash
set -euo pipefail

# Detect OS and architecture
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

# Set platform-specific variables
case "${OS}-${ARCH}" in
    "darwin-x86_64")
        TARGET="x86_64-apple-darwin"
        PARALLEL_JOBS=$(sysctl -n hw.ncpu)
        export MACOSX_DEPLOYMENT_TARGET="10.15"
        ;;
    "darwin-arm64")
        TARGET="aarch64-apple-darwin"
        PARALLEL_JOBS=$(sysctl -n hw.ncpu)
        export MACOSX_DEPLOYMENT_TARGET="11.0"
        ;;
    *)
        TARGET="x86_64-unknown-linux-gnu"
        PARALLEL_JOBS=$(nproc 2>/dev/null || echo 4)
        ;;
esac

# Configuration
BUILD_MODE="release"

# Configuration
FEATURES="default"
OUTPUT_DIR="./target/optimized"

# Ensure we have the target installed
rustup target add $TARGET

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create output directory if it doesn't exist
mkdir -p "$OUTPUT_DIR"

# Function to print section headers
section() {
    echo -e "\n${GREEN}=== $1 ===${NC}"
}

# Clean previous build artifacts
section "Cleaning previous build"
cargo clean --target-dir="$OUTPUT_DIR"

# Configure build settings
export RUSTFLAGS="-C target-cpu=native -C codegen-units=1"
export CARGO_BUILD_JOBS="$PARALLEL_JOBS"
export CARGO_INCREMENTAL=1

# Build the project
section "Building project"
time cargo build \
    --target "$TARGET" \
    --features "$FEATURES" \
    --$BUILD_MODE \
    --target-dir="$OUTPUT_DIR" \
    --workspace \
    --exclude=api \
    --exclude=app \
    --exclude=client

# Optimize binaries
section "Optimizing binaries"
for bin in zeta-infer quantize-cli; do
    if [ -f "$OUTPUT_DIR/$TARGET/$BUILD_MODE/$bin" ]; then
        strip --strip-all "$OUTPUT_DIR/$TARGET/$BUILD_MODE/$bin"
        upx --best --lzma "$OUTPUT_DIR/$TARGET/$BUILD_MODE/$bin"
    fi
done

# Print build summary
section "Build complete"
du -sh "$OUTPUT_DIR/$TARGET/$BUILD_MODE/"* | sort -hr

# Copy necessary files to dist directory
section "Creating distribution"
DIST_DIR="./dist"
mkdir -p "$DIST_DIR/bin"
cp -v "$OUTPUT_DIR/$TARGET/$BUILD_MODE/zeta-infer" "$DIST_DIR/bin/"
cp -v "$OUTPUT_DIR/$TARGET/$BUILD_MODE/quantize-cli" "$DIST_DIR/bin/"

# Create a simple launcher script
cat > "$DIST_DIR/run.sh" << 'EOF'
#!/bin/bash
set -euo pipefail

export RUST_LOG=info
BIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

exec "${BIN_DIR}/zeta-infer" "$@"
EOF

chmod +x "$DIST_DIR/run.sh"

section "Distribution created in $DIST_DIR"
ls -lh "$DIST_DIR"

echo -e "\n${GREEN}✓ Build completed successfully!${NC}"
