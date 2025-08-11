#!/bin/bash
set -euo pipefail

# Load configuration
source "$(dirname "$0")/deploy_config.sh"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to print section headers
section() {
    echo -e "\n${GREEN}=== $1 ===${NC}"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check for required tools
section "Checking dependencies"
for cmd in docker docker-compose cargo; do
    if ! command_exists "$cmd"; then
        echo -e "${RED}Error: $cmd is not installed${NC}"
        exit 1
    fi
done

# Build the project
section "Building project"
./scripts/optimized_build.sh

# Build Docker image
section "Building Docker image"
docker build \
    -f Dockerfile.optimized \
    -t "${DOCKER_IMAGE}:${DOCKER_TAG}" \
    .

# Push to registry if specified
if [ "$PUSH_TO_REGISTRY" = true ]; then
    section "Pushing to registry"
    docker push "${DOCKER_IMAGE}:${DOCKER_TAG}"
fi

# Deploy to Kubernetes if enabled
if [ "$DEPLOY_K8S" = true ]; then
    section "Deploying to Kubernetes"
    if ! command_exists kubectl; then
        echo -e "${YELLOW}Warning: kubectl not found, skipping Kubernetes deployment${NC}"
    else
        # Update the image in the Kubernetes deployment
        kubectl set image deployment/zeta-reticula \
            "zeta-reticula=${DOCKER_IMAGE}:${DOCKER_TAG}" \
            --record
        
        # Wait for rollout to complete
        kubectl rollout status deployment/zeta-reticula
    fi
fi

# Print deployment summary
section "Deployment complete"
echo -e "${GREEN}✓ Successfully deployed ${DOCKER_IMAGE}:${DOCKER_TAG}${NC}"
echo -e "\n${YELLOW}Next steps:${NC}"
echo "1. Verify the deployment:"
echo "   kubectl get pods -l app=zeta-reticula"
echo "2. Check logs:"
echo "   kubectl logs -f deployment/zeta-reticula"
echo "3. Access the service:"
echo "   kubectl port-forward svc/zeta-reticula 8080:80"
