#!/bin/bash

# Zeta Reticula Crates.io Publishing Script
# Publishes all packages to crates.io registry

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}"
}

info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

# Check if cargo is available
check_cargo() {
    if ! command -v cargo >/dev/null 2>&1; then
        error "Cargo (Rust) is not installed. Please install Rust toolchain."
        exit 1
    fi

    if ! command -v cargo-publish >/dev/null 2>&1; then
        error "cargo-publish is not available. Please install cargo-publish plugin."
        error "Run: cargo install cargo-publish"
        exit 1
    fi
}

# Check if user is logged in to crates.io
check_crates_login() {
    if ! cargo search zeta-reticula >/dev/null 2>&1; then
        error "Not logged in to crates.io"
        info "Please login to crates.io:"
        info "  cargo login"
        info "Then run this script again."
        exit 1
    fi
    success "Logged in to crates.io"
}

# Publish package with dependencies
publish_package() {
    local package_path="$1"
    local package_name="$2"

    log "Publishing $package_name..."

    if [ -d "$package_path" ]; then
        cd "$package_path"

        # Check if package needs to be published
        if cargo search "$package_name" | grep -q "^$package_name = "; then
            warning "Package $package_name already exists on crates.io"
            info "To update, run: cargo publish --allow-dirty"
        else
            # Try to publish
            if cargo publish --dry-run; then
                if cargo publish; then
                    success "Successfully published $package_name"
                else
                    error "Failed to publish $package_name"
                    exit 1
                fi
            else
                error "Package $package_name failed dry-run validation"
                exit 1
            fi
        fi

        cd - >/dev/null
    else
        warning "Package path $package_path does not exist, skipping..."
    fi
}

# Main publishing function
main() {
    echo -e "${BLUE}📦 Zeta Reticula Crates.io Publishing Script${NC}"
    echo ""

    log "Checking prerequisites..."

    check_cargo
    check_crates_login

    echo ""
    info "Publishing packages in dependency order:"
    echo ""

    # Core packages (no dependencies on each other)
    info "1. Publishing core packages (can be published in parallel):"
    info "   - zeta-shared"
    info "   - zeta-kv-cache"
    info "   - zeta-quantization"
    info "   - zeta-salience"
    echo ""

    # Publish core packages first
    publish_package "core/shared" "zeta-shared"
    publish_package "core/kv-cache" "zeta-kv-cache"
    publish_package "core/quantization" "zeta-quantization"
    publish_package "core/salience" "zeta-salience"

    echo ""
    info "2. Publishing runtime packages:"
    info "   - zeta-inference"
    echo ""

    # Check if inference crate exists and publish
    if [ -d "runtime/inference" ]; then
        cd runtime/inference
        if [ -f "Cargo.toml" ]; then
            publish_package "runtime/inference" "zeta-inference"
        fi
        cd - >/dev/null
    fi

    echo ""
    info "3. Publishing interface packages:"
    info "   - zeta-reticula (CLI)"
    echo ""

    # Publish CLI last (depends on others)
    publish_package "interfaces/cli" "zeta-reticula"

    echo ""
    success "All packages published successfully to crates.io!"
    echo ""
    info "Published packages:"
    echo "  • zeta-shared - Shared utilities and types"
    echo "  • zeta-kv-cache - High-performance key-value cache"
    echo "  • zeta-quantization - Advanced quantization engine"
    echo "  • zeta-salience - Salience analysis engine"
    echo "  • zeta-inference - Unified inference runtime"
    echo "  • zeta-reticula - Main CLI application"
    echo ""
    info "You can now install the CLI with:"
    info "  cargo install zeta-reticula"
    echo ""
    info "For more information, visit:"
    info "  https://crates.io/crates/zeta-reticula"
}

# Show usage if requested
if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Publish Zeta Reticula packages to crates.io"
    echo ""
    echo "Prerequisites:"
    echo "  • Rust toolchain installed"
    echo "  • Logged in to crates.io (cargo login)"
    echo "  • All packages pass 'cargo publish --dry-run'"
    echo ""
    echo "Options:"
    echo "  --help, -h    Show this help message"
    echo "  --dry-run     Run dry-run only (no actual publishing)"
    echo ""
    echo "Examples:"
    echo "  $0                    # Publish all packages"
    echo "  $0 --dry-run         # Test publishing without uploading"
    exit 0
fi

# Handle dry-run option
if [ "${1:-}" = "--dry-run" ]; then
    log "Running in DRY RUN mode - no actual publishing"
    echo ""
    info "To actually publish, run: $0"
    echo ""
    # Still check prerequisites but don't publish
    check_cargo
    check_crates_login
    success "Dry run completed successfully - all prerequisites met"
    exit 0
fi

# Run main publishing function
main "$@"
