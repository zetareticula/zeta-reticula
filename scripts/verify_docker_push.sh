#!/bin/bash

# Script to verify Docker image push to Docker Hub for Zeta Reticula
# Monitors CI/CD pipeline and validates Docker Hub image availability

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DOCKER_REGISTRY="docker.io"
DOCKER_REPO="zetareticula/zeta-reticula"
CURRENT_COMMIT=$(git rev-parse HEAD)
SHORT_COMMIT=${CURRENT_COMMIT:0:7}

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

# Function to check if Docker image exists on Docker Hub
check_docker_hub_image() {
    local tag="$1"
    local repo="$2"
    
    log "Checking Docker Hub for image: $repo:$tag"
    
    # Use Docker Hub API to check if image exists
    local api_url="https://hub.docker.com/v2/repositories/$repo/tags/$tag"
    
    if command -v curl >/dev/null 2>&1; then
        local response=$(curl -s -o /dev/null -w "%{http_code}" "$api_url")
        if [ "$response" = "200" ]; then
            return 0
        else
            return 1
        fi
    elif command -v wget >/dev/null 2>&1; then
        if wget -q --spider "$api_url" 2>/dev/null; then
            return 0
        else
            return 1
        fi
    else
        error "Neither curl nor wget found. Cannot check Docker Hub API."
        return 1
    fi
}

# Function to get image metadata from Docker Hub
get_image_metadata() {
    local tag="$1"
    local repo="$2"
    
    local api_url="https://hub.docker.com/v2/repositories/$repo/tags/$tag"
    
    if command -v curl >/dev/null 2>&1; then
        curl -s "$api_url" | python3 -m json.tool 2>/dev/null || echo "Failed to parse JSON"
    else
        echo "curl not available for metadata retrieval"
    fi
}

# Function to check GitHub Actions status
check_github_actions() {
    local repo_url=$(git config --get remote.origin.url | sed 's/\.git$//')
    local actions_url="${repo_url}/actions"
    
    log "GitHub Actions Status:"
    echo "  📋 Actions URL: $actions_url"
    echo "  🔍 Look for workflow run with commit: $SHORT_COMMIT"
    echo ""
    
    info "Expected workflow jobs:"
    echo "    1. Run Tests"
    echo "    2. Validate Kubernetes Manifests"
    echo "    3. Build and Push Docker Images ← This pushes to Docker Hub"
    echo "    4. Deploy to EKS (main branch only)"
    echo "    5. Notify Status"
    echo ""
}

# Function to wait for image with timeout
wait_for_image() {
    local tag="$1"
    local repo="$2"
    local timeout_minutes="${3:-30}"
    local check_interval="${4:-60}"
    
    local timeout_seconds=$((timeout_minutes * 60))
    local elapsed=0
    
    log "Waiting for Docker image $repo:$tag (timeout: ${timeout_minutes}m)"
    
    while [ $elapsed -lt $timeout_seconds ]; do
        if check_docker_hub_image "$tag" "$repo"; then
            success "Docker image found: $repo:$tag"
            return 0
        fi
        
        echo -n "."
        sleep $check_interval
        elapsed=$((elapsed + check_interval))
    done
    
    echo ""
    error "Timeout waiting for Docker image: $repo:$tag"
    return 1
}

# Function to verify image can be pulled
verify_image_pullable() {
    local tag="$1"
    local repo="$2"
    
    log "Verifying image can be pulled: $repo:$tag"
    
    if command -v docker >/dev/null 2>&1; then
        if docker pull "$repo:$tag" >/dev/null 2>&1; then
            success "Image successfully pulled: $repo:$tag"
            
            # Get image info
            log "Image information:"
            docker inspect "$repo:$tag" --format='{{.Created}}' | head -1 | xargs -I {} echo "  Created: {}"
            docker inspect "$repo:$tag" --format='{{.Size}}' | head -1 | xargs -I {} echo "  Size: {} bytes"
            docker inspect "$repo:$tag" --format='{{.Architecture}}' | head -1 | xargs -I {} echo "  Architecture: {}"
            
            return 0
        else
            error "Failed to pull image: $repo:$tag"
            return 1
        fi
    else
        warning "Docker not available locally. Cannot verify image pull."
        return 0
    fi
}

# Main verification function
main() {
    echo -e "${BLUE}🐳 Zeta Reticula Docker Hub Verification${NC}"
    echo ""
    
    log "Current commit: $CURRENT_COMMIT"
    log "Expected Docker tags:"
    echo "  • $DOCKER_REPO:$CURRENT_COMMIT (full commit SHA)"
    echo "  • $DOCKER_REPO:latest (if main branch)"
    echo ""
    
    # Check GitHub Actions status
    check_github_actions
    
    # Check for images with different tag strategies
    local tags_to_check=("$CURRENT_COMMIT" "latest" "$SHORT_COMMIT")
    local found_image=false
    
    for tag in "${tags_to_check[@]}"; do
        log "Checking for tag: $tag"
        
        if check_docker_hub_image "$tag" "$DOCKER_REPO"; then
            success "Found image: $DOCKER_REPO:$tag"
            found_image=true
            
            # Get metadata
            log "Image metadata:"
            get_image_metadata "$tag" "$DOCKER_REPO" | head -20
            echo ""
            
            # Verify pullable
            verify_image_pullable "$tag" "$DOCKER_REPO"
            echo ""
            
        else
            warning "Image not found: $DOCKER_REPO:$tag"
        fi
    done
    
    if [ "$found_image" = false ]; then
        echo ""
        warning "No Docker images found yet. This could mean:"
        echo "  1. CI/CD pipeline is still running"
        echo "  2. Build job hasn't reached the push stage"
        echo "  3. There was a build failure"
        echo "  4. Docker Hub credentials are not configured"
        echo ""
        
        info "To wait for the image to be pushed, run:"
        echo "  $0 --wait"
        echo ""
        
        info "To check CI/CD status, run:"
        echo "  ./scripts/check_ci_status.sh"
        
    else
        success "Docker image verification completed successfully!"
        echo ""
        info "You can now pull and use the image:"
        echo "  docker pull $DOCKER_REPO:latest"
        echo "  docker run $DOCKER_REPO:latest system status"
    fi
}

# Handle command line arguments
case "${1:-}" in
    --wait)
        wait_for_image "$CURRENT_COMMIT" "$DOCKER_REPO" 30 60
        ;;
    --check-latest)
        check_docker_hub_image "latest" "$DOCKER_REPO" && success "Latest image available" || warning "Latest image not found"
        ;;
    --metadata)
        get_image_metadata "${2:-latest}" "$DOCKER_REPO"
        ;;
    --help)
        echo "Usage: $0 [OPTIONS]"
        echo ""
        echo "Options:"
        echo "  --wait          Wait up to 30 minutes for image to be pushed"
        echo "  --check-latest  Check if latest tag is available"
        echo "  --metadata TAG  Get metadata for specific tag"
        echo "  --help          Show this help message"
        echo ""
        echo "Examples:"
        echo "  $0                    # Check current status"
        echo "  $0 --wait            # Wait for image push"
        echo "  $0 --metadata latest # Get latest image metadata"
        ;;
    *)
        main
        ;;
esac
