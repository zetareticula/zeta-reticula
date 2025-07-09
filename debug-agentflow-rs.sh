#!/bin/bash
set -e

# Colors for output
cyan="\033[0;36m"
red="\033[0;31m"
green="\033[0;32m"
yellow="\033[0;33m"
reset="\033[0m"

function log() {
    echo -e "${cyan}==> $1${reset}"
}

function warning() {
    echo -e "${yellow}Warning: $1${reset}"
}

function error() {
    echo -e "${red}==> $1${reset}"
    exit 1
}

function success() {
    echo -e "${green}==> $1${reset}"
}

function check_rust_environment() {
    log "Checking Rust environment..."
    
    # Check if Rust is installed
    if ! command -v rustc &> /dev/null; then
        error "Rust is not installed. Please install Rust first."
    fi

    # Get Rust version
    RUST_VERSION=$(rustc --version)
    log "Rust version: ${RUST_VERSION}"

    # Check Cargo
    if ! command -v cargo &> /dev/null; then
        error "Cargo is not installed. Please install Cargo first."
    fi

    # Check Rust toolchain
    TOOLCHAIN=$(rustup show active-toolchain)
    log "Active toolchain: ${TOOLCHAIN}"
}

function analyze_code_structure() {
    log "Analyzing code structure..."
    
    # Check for main components
    if [ -f "src/main.rs" ]; then
        log "Found main.rs"
    else
        warning "No main.rs found"
    fi

    if [ -f "src/lib.rs" ]; then
        log "Found lib.rs"
    else
        warning "No lib.rs found"
    fi

    # Check for tests
    if [ -d "tests" ]; then
        log "Found tests directory"
        TEST_FILES=$(find tests -name "*.rs")
        if [ -z "${TEST_FILES}" ]; then
            warning "No test files found in tests directory"
        fi
    else
        warning "No tests directory found"
    fi

    # Check for examples
    if [ -d "examples" ]; then
        log "Found examples directory"
        EXAMPLE_FILES=$(find examples -name "*.rs")
        if [ -z "${EXAMPLE_FILES}" ]; then
            warning "No example files found in examples directory"
        fi
    fi
}

function check_dependencies() {
    log "Checking dependencies..."
    
    # Check Cargo.toml
    if [ -f "Cargo.toml" ]; then
        log "Found Cargo.toml"
        
        # Parse dependencies
        log "Dependencies:"
        cat Cargo.toml | grep -E "^name|^version|^dependencies|\[features\]"
        
        # Check for common dependencies
        COMMON_DEPS=$(cat Cargo.toml | grep -E "serde|thiserror|anyhow|tokio|tonic")
        if [ -z "${COMMON_DEPS}" ]; then
            warning "Missing common dependencies"
        fi
    else
        error "No Cargo.toml found"
    fi

    # Check for build script
    if [ -f "build.rs" ]; then
        log "Found build.rs"
        
        # Check if build script is declared in Cargo.toml
        if ! grep -q "build = " Cargo.toml; then
            warning "build.rs found but not declared in Cargo.toml"
        fi
    fi
}

function check_code_quality() {
    log "Checking code quality..."
    
    # Check for common Rust patterns
    if [ -f "src/lib.rs" ]; then
        # Check for Debug derive
        if ! grep -q "derive(Debug)" src/lib.rs; then
            warning "No Debug derive found in lib.rs"
        fi

        # Check for Clone derive
        if ! grep -q "derive(Clone)" src/lib.rs; then
            warning "No Clone derive found in lib.rs"
        fi

        # Check for async/await usage
        if ! grep -q "async fn" src/lib.rs; then
            warning "No async functions found"
        fi
    fi

    # Check for unsafe code
    if [ -f "src/lib.rs" ]; then
        if grep -q "unsafe" src/lib.rs; then
            log "Unsafe code found"
            grep -n "unsafe" src/lib.rs
        fi
    fi
}

function run_tests() {
    log "Running tests..."
    
    # Run tests with verbose output
    cargo test --verbose 2>&1 | grep -E "^test|running|running.*test|ok|FAILED|error"
}

function check_build() {
    log "Checking build..."
    
    # Check release build
    cargo build --release --verbose 2>&1 | grep -E "error|warning"
    
    # Check debug build
    cargo build --verbose 2>&1 | grep -E "error|warning"
}

function main() {
    log "Starting agentflow-rs debugging..."
    
    # Change to agentflow-rs directory
    if [ ! -d "agentflow-rs" ]; then
        error "agentflow-rs directory not found"
    fi
    
    cd agentflow-rs
    
    # Check Rust environment
    check_rust_environment
    
    # Analyze code structure
    analyze_code_structure
    
    # Check dependencies
    check_dependencies
    
    # Check code quality
    check_code_quality
    
    # Run tests
    run_tests
    
    # Check build
    check_build

    success "agentflow-rs debugging completed!"
}

main "$@"
