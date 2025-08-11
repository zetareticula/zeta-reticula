# Just commands for Zeta Reticula development

# Default target - show available commands
default:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Available commands:"
    just --list

# Build the project in release mode
build:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Building in release mode..."
    RUSTFLAGS="-C target-cpu=native" cargo build --release

# Run the salience engine
run:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Starting salience-engine..."
    RUST_LOG=info cargo run --release --bin salience-engine

# Run tests
test:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Running tests..."
    RUST_BACKTRACE=1 cargo test -- --nocapture

# Clean build artifacts
clean:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Cleaning..."
    cargo clean
    find . -name "target" -type d -exec rm -rf {} + 2>/dev/null || true
    find . -name "Cargo.lock" -delete

# Format code
fmt:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Formatting code..."
    cargo fmt --all

# Check code style
clippy:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Running clippy..."
    cargo clippy -- -D warnings

# Run in development mode with watch
dev:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Starting development server with watch..."
    cargo watch -x 'run --bin salience-engine'
