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

function debug_service() {
    local SERVICE=$1
    local SERVICE_DIR="${SERVICE}"
    
    if [ ! -d "${SERVICE_DIR}" ]; then
        warning "Service directory ${SERVICE_DIR} not found"
        return
    fi

    log "Debugging ${SERVICE}..."
    
    # Check Cargo.toml
    if [ -f "${SERVICE_DIR}/Cargo.toml" ]; then
        log "Dependencies for ${SERVICE}:"
        cat "${SERVICE_DIR}/Cargo.toml" | grep -E "^name|^version|^dependencies|\[features\]"
        
        # Check for common dependency issues
        local MISSING_DEPS=$(grep -E "^dependencies" "${SERVICE_DIR}/Cargo.toml" && \
                            ! grep -qE "serde|thiserror|anyhow|tokio|tonic" "${SERVICE_DIR}/Cargo.toml")
        if [ -n "${MISSING_DEPS}" ]; then
            warning "Missing common dependencies in ${SERVICE_DIR}/Cargo.toml"
        fi
    else
        warning "No Cargo.toml found in ${SERVICE_DIR}"
    fi

    # Check source files
    local SRC_FILES=""
    if [ -f "${SERVICE_DIR}/src/main.rs" ]; then
        SRC_FILES="${SERVICE_DIR}/src/main.rs"
        log "Found main.rs in ${SERVICE_DIR}"
    elif [ -f "${SERVICE_DIR}/src/lib.rs" ]; then
        SRC_FILES="${SERVICE_DIR}/src/lib.rs"
        log "Found lib.rs in ${SERVICE_DIR}"
    else
        warning "No main.rs or lib.rs found in ${SERVICE_DIR}"
    fi

    if [ -n "${SRC_FILES}" ]; then
        # Check for common Rust patterns
        if grep -q "derive.*Debug" "${SRC_FILES}" && grep -q "derive.*Clone" "${SRC_FILES}"; then
            log "Found Debug and Clone derives in ${SRC_FILES}"
        else
            warning "Missing Debug/Clone derives in ${SRC_FILES}"
        fi

        # Check for duplicate definitions
        local DUPLICATES=$(grep -o "struct [A-Za-z0-9_]*" "${SRC_FILES}" | sort | uniq -d)
        if [ -n "${DUPLICATES}" ]; then
            error "Found duplicate definitions in ${SRC_FILES}: ${DUPLICATES}"
        fi

        # Check for unresolved imports
        local UNRESOLVED_IMPORTS=$(grep "use crate::" "${SRC_FILES}" | grep -v "use crate::prelude" | grep -v "use crate::types")
        if [ -n "${UNRESOLVED_IMPORTS}" ]; then
            warning "Potential unresolved imports found in ${SRC_FILES}"
            echo "${UNRESOLVED_IMPORTS}"
        fi
    fi

    # Check for build.rs
    if [ -f "${SERVICE_DIR}/build.rs" ]; then
        log "Found build.rs in ${SERVICE_DIR}"
        # Check for build script dependencies
        if ! grep -q "build.rs" "${SERVICE_DIR}/Cargo.toml"; then
            warning "build.rs found but not declared in Cargo.toml"
        fi
    fi

    # Check for tests
    if [ -d "${SERVICE_DIR}/tests" ]; then
        log "Found tests directory in ${SERVICE_DIR}"
        # Check if tests are ignored
        if [ -f "${SERVICE_DIR}/tests/.gitignore" ]; then
            warning "Tests directory is ignored in ${SERVICE_DIR}"
        fi
    fi

    # Check for feature flags
    if [ -f "${SERVICE_DIR}/Cargo.toml" ]; then
        local FEATURES=$(grep "\[features\]" "${SERVICE_DIR}/Cargo.toml" -A 10)
        if [ -n "${FEATURES}" ]; then
            log "Feature flags found:"
            echo "${FEATURES}"
            # Check for unused features
            local UNUSED_FEATURES=$(grep -o "[^\[]*" <<< "${FEATURES}" | grep -v "default" | grep -v "server" | grep -v "wasm")
            if [ -n "${UNUSED_FEATURES}" ]; then
                warning "Potential unused features found: ${UNUSED_FEATURES}"
            fi
        fi
    fi

    # Run cargo check with detailed output
    log "Running cargo check for ${SERVICE}..."
    pushd "${SERVICE_DIR}" > /dev/null
    cargo check 2>&1 | grep -E "error|warning" || true
    popd > /dev/null
}

function main() {
    log "Starting Rust app debugging..."
    
    # List of main services to debug
    local SERVICES=(
        "agentflow-rs"
        "llm-rs"
        "ns-router-rs"
        "salience-engine"
        "kvquant-rs"
        "api"
    )

    # Debug each service
    for SERVICE in "${SERVICES[@]}"; do
        echo -e "\n${green}=== Debugging ${SERVICE} ===${reset}"
        debug_service "${SERVICE}"
    done

    # Check for common issues
    log "\n=== Common Issues Check ==="
    
    # Check for missing dependencies
    log "Checking for missing dependencies..."
    local MISSING_DEPS=$(cargo check 2>&1 | grep -i "error" | grep -i "unresolved import")
    if [ -n "${MISSING_DEPS}" ]; then
        error "Missing dependencies found:"
        echo "${MISSING_DEPS}"
    fi

    # Check for duplicate definitions
    log "Checking for duplicate definitions..."
    local DUPLICATES=$(cargo check 2>&1 | grep -i "error" | grep -i "defined multiple times")
    if [ -n "${DUPLICATES}" ]; then
        error "Duplicate definitions found:"
        echo "${DUPLICATES}"
    fi

    # Check for feature flag conflicts
    log "Checking feature flags..."
    local FEATURE_ERRORS=$(cargo build --features "server" --release 2>&1 | grep -i "error" | grep -i "feature")
    if [ -n "${FEATURE_ERRORS}" ]; then
        error "Feature flag issues found:"
        echo "${FEATURE_ERRORS}"
    fi

    # Check for WASM build issues
    log "Checking WASM build..."
    local WASM_ERRORS=$(cargo build --target wasm32-unknown-unknown --features "wasm" 2>&1 | grep -i "error")
    if [ -n "${WASM_ERRORS}" ]; then
        error "WASM build issues found:"
        echo "${WASM_ERRORS}"
        # Check for common WASM issues
        if echo "${WASM_ERRORS}" | grep -q "unsupported by mio"; then
            warning "Net feature not disabled for WASM target"
        fi
    fi

    # Check for missing derive macros
    log "Checking for missing derive macros..."
    local MISSING_MACROS=$(cargo check 2>&1 | grep -i "error" | grep -i "derive")
    if [ -n "${MISSING_MACROS}" ]; then
        error "Missing derive macros found:"
        echo "${MISSING_MACROS}"
    fi

    # Check for unresolved types
    log "Checking for unresolved types..."
    local UNRESOLVED_TYPES=$(cargo check 2>&1 | grep -i "error" | grep -i "cannot find type")
    if [ -n "${UNRESOLVED_TYPES}" ]; then
        error "Unresolved types found:"
        echo "${UNRESOLVED_TYPES}"
    fi

    # Check for implementation conflicts
    log "Checking for implementation conflicts..."
    local IMPL_CONFLICTS=$(cargo check 2>&1 | grep -i "error" | grep -i "conflicting implementations")
    if [ -n "${IMPL_CONFLICTS}" ]; then
        error "Implementation conflicts found:"
        echo "${IMPL_CONFLICTS}"
    fi

    # Check for mismatched types
    log "Checking for type mismatches..."
    local TYPE_MISMATCHES=$(cargo check 2>&1 | grep -i "error" | grep -i "mismatched types")
    if [ -n "${TYPE_MISMATCHES}" ]; then
        error "Type mismatches found:"
        echo "${TYPE_MISMATCHES}"
    fi
}

main "$@"
