#!/bin/bash

# Zeta Reticula Kubernetes Validation Script
# Validates the Kubernetes manifests for deployment readiness

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

# Check if kustomize is available
check_kustomize() {
    if ! command -v kustomize >/dev/null 2>&1; then
        error "kustomize not found. Please install kustomize."
        exit 1
    fi
    success "kustomize $(kustomize version) is available"
}

# Check if kubectl is available
check_kubectl() {
    if ! command -v kubectl >/dev/null 2>&1; then
        warning "kubectl not found. Cannot validate against live cluster."
        return 1
    fi
    success "kubectl $(kubectl version --client --short | head -1) is available"
    return 0
}

# Validate base configuration
validate_base() {
    log "Validating base configuration..."
    if kustomize build k8s/base > /dev/null; then
        success "Base configuration is valid"
    else
        error "Base configuration has errors"
        return 1
    fi
}

# Validate development overlay
validate_dev() {
    log "Validating development overlay..."
    if kustomize build k8s/overlays/dev > /dev/null; then
        success "Development overlay is valid"
    else
        error "Development overlay has errors"
        return 1
    fi
}

# Validate production overlay
validate_prod() {
    log "Validating production overlay..."
    if kustomize build k8s/overlays/prod > /dev/null; then
        success "Production overlay is valid"
    else
        error "Production overlay has errors"
        return 1
    fi
}

# Check resource allocations
check_resources() {
    log "Checking resource allocations..."

    # Check if GPU resources are properly allocated
    if kustomize build k8s/overlays/prod | grep -q "nvidia.com/gpu"; then
        success "GPU resources are properly allocated"
    else
        warning "No GPU resources found in production configuration"
    fi

    # Check memory allocations
    local total_memory=$(kustomize build k8s/overlays/prod | grep -o "memory: \"[^\"]*\"" | sort | uniq -c)
    info "Memory allocations found:"
    echo "$total_memory"
}

# Check security policies
check_security() {
    log "Checking security policies..."

    if kustomize build k8s/base | grep -q "NetworkPolicy"; then
        success "Network policies are configured"
    else
        warning "No network policies found"
    fi

    if kustomize build k8s/base | grep -q "kind: Ingress"; then
        success "Ingress configuration is present"
    else
        warning "No ingress configuration found"
    fi
}

# Generate deployment summary
generate_summary() {
    log "Generating deployment summary..."

    echo ""
    info "📊 Kubernetes Deployment Summary"
    echo ""

    # Count resources by type
    local resources=$(kustomize build k8s/overlays/prod 2>/dev/null || echo "")
    if [ -n "$resources" ]; then
        echo "Base Resources:"
        echo "$resources" | grep "kind:" | sort | uniq -c
        echo ""

        # Show replica counts
        info "Replica Counts:"
        echo "$resources" | grep "replicas:" | sort | uniq -c
        echo ""
    fi

    # Show environment differences
    info "Environment Comparison:"
    echo "Development:"
    kustomize build k8s/overlays/dev 2>/dev/null | grep "replicas:" | head -3
    echo ""
    echo "Production:"
    kustomize build k8s/overlays/prod 2>/dev/null | grep "replicas:" | head -3
    echo ""
}

# Main validation function
main() {
    echo -e "${BLUE}🐳 Zeta Reticula Kubernetes Validation${NC}"
    echo ""

    local all_passed=true

    # Run all checks
    check_kustomize

    if ! check_kubectl; then
        warning "Skipping cluster-specific validations"
    fi

    if ! validate_base; then all_passed=false; fi
    if ! validate_dev; then all_passed=false; fi
    if ! validate_prod; then all_passed=false; fi

    check_resources
    check_security
    generate_summary

    echo ""
    if [ "$all_passed" = true ]; then
        success "All validations passed! ✅"
        info "Ready for deployment:"
        echo "  1. Apply base configuration: kubectl apply -k k8s/base"
        echo "  2. Apply development overlay: kubectl apply -k k8s/overlays/dev"
        echo "  3. Or apply production overlay: kubectl apply -k k8s/overlays/prod"
    else
        error "Some validations failed. Please fix the issues above."
        exit 1
    fi
}

# Show usage if requested
if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Validate Zeta Reticula Kubernetes configuration"
    echo ""
    echo "Options:"
    echo "  --help, -h    Show this help message"
    echo ""
    echo "This script validates:"
    echo "  - Base Kubernetes configuration"
    echo "  - Development and production overlays"
    echo "  - Resource allocations"
    echo "  - Security policies"
    echo "  - Network configuration"
    exit 0
fi

# Run main validation
main "$@"
