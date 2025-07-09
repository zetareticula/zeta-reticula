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

function check_go_environment() {
    log "Checking Go environment..."
    
    # Check if Go is installed
    if ! command -v go &> /dev/null; then
        error "Go is not installed. Please install Go first."
    fi

    # Get Go version
    GO_VERSION=$(go version)
    log "Go version: ${GO_VERSION}"

    # Check GOPATH
    GOPATH=$(go env GOPATH)
    log "GOPATH: ${GOPATH}"

    # Check GOROOT
    GOROOT=$(go env GOROOT)
    log "GOROOT: ${GOROOT}"
}

function debug_go_module() {
    local MODULE_DIR=$1
    
    if [ ! -d "${MODULE_DIR}" ]; then
        warning "Module directory ${MODULE_DIR} not found"
        return
    fi

    log "Debugging Go module in ${MODULE_DIR}..."
    
    # Check for go.mod
    if [ -f "${MODULE_DIR}/go.mod" ]; then
        log "Found go.mod"
        
        # Check module name and version
        MODULE_INFO=$(cat "${MODULE_DIR}/go.mod" | grep -E "^module|^go")
        log "Module info:"
        echo "${MODULE_INFO}"
        
        # Check dependencies
        pushd "${MODULE_DIR}" > /dev/null
        log "Dependencies:"
        go list -m all | grep -v "^go\.example\.com" | grep -v "^golang\.org/x"
        
        # Check for indirect dependencies
        INDIRECT=$(go list -m all | grep "indirect")
        if [ -n "${INDIRECT}" ]; then
            log "Indirect dependencies:"
            echo "${INDIRECT}"
        fi
        popd > /dev/null
    else
        warning "No go.mod found in ${MODULE_DIR}"
    fi

    # Check for main.go
    if [ -f "${MODULE_DIR}/main.go" ]; then
        log "Found main.go"
        
        # Check package declaration
        PACKAGE=$(head -n 1 "${MODULE_DIR}/main.go" | grep "package")
        if [ -z "${PACKAGE}" ]; then
            warning "No package declaration found in main.go"
        fi
    fi

    # Check for tests
    if [ -f "${MODULE_DIR}/_test.go" ] || [ -d "${MODULE_DIR}/internal" ]; then
        log "Found tests"
        
        # Run tests with coverage
        pushd "${MODULE_DIR}" > /dev/null
        go test -v -cover ./... 2>&1 | grep -E "^===|FAIL|PASS|coverage"
        popd > /dev/null
    fi

    # Check for build issues
    log "Checking build..."
    pushd "${MODULE_DIR}" > /dev/null
    go build -v ./... 2>&1 | grep -E "^#|error"
    popd > /dev/null

    # Check for linting issues
    if command -v golangci-lint &> /dev/null; then
        log "Running linter..."
        pushd "${MODULE_DIR}" > /dev/null
        golangci-lint run 2>&1 | grep -E "error|warning"
        popd > /dev/null
    else
        warning "golangci-lint not found. Skipping linting check."
    fi
}

function check_common_issues() {
    log "Checking common Go issues..."
    
    # Check for common dependency issues
    log "Checking for deprecated dependencies..."
    DEPRECATED=$(go list -m all | grep "deprecated")
    if [ -n "${DEPRECATED}" ]; then
        warning "Deprecated dependencies found:"
        echo "${DEPRECATED}"
    fi

    # Check for version conflicts
    log "Checking for version conflicts..."
    VERSIONS=$(go list -m all | sort | uniq -d)
    if [ -n "${VERSIONS}" ]; then
        warning "Version conflicts found:"
        echo "${VERSIONS}"
    fi

    # Check for missing modules
    log "Checking for missing modules..."
    go mod tidy 2>&1 | grep -E "error|warning"
}

function main() {
    log "Starting Go app debugging..."
    
    # List of Go modules to debug
    local MODULES=(
        "zeta-sidecar"
    )

    # Check Go environment
    check_go_environment

    # Debug each module
    for MODULE in "${MODULES[@]}"; do
        echo -e "\n${green}=== Debugging ${MODULE} ===${reset}"
        debug_go_module "${MODULE}"
    done

    # Check common issues
    check_common_issues

    success "Go app debugging completed!"
}

main "$@"
