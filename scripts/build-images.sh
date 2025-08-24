#!/bin/bash

# Exit on error
set -e

# Load environment variables
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
else
    echo "Warning: .env file not found. Using default values."
    export CONTAINER_REGISTRY="ghcr.io/zetareticula"
    export IMAGE_TAG="latest"
fi

# Build and push master service
echo "Building master-service image..."
docker build -t ${CONTAINER_REGISTRY}/master-service:${IMAGE_TAG} -f Dockerfile.master .

echo "Building worker image..."
docker build -t ${CONTAINER_REGISTRY}/worker:${IMAGE_TAG} -f Dockerfile.worker .

# If running in Minikube, load images directly to avoid pushing to a registry
if command -v minikube &> /dev/null && minikube status &> /dev/null; then
    echo "Loading images into Minikube..."
    minikube image load ${CONTAINER_REGISTRY}/master-service:${IMAGE_TAG}
    minikube image load ${CONTAINER_REGISTRY}/worker:${IMAGE_TAG}
    
    # Tag images with the default Minikube registry
    docker tag ${CONTAINER_REGISTRY}/master-service:${IMAGE_TAG} master-service:${IMAGE_TAG}
    docker tag ${CONTAINER_REGISTRY}/worker:${IMAGE_TAG} worker:${IMAGE_TAG}
else
    echo "Pushing images to container registry..."
    # Uncomment these lines if you want to push to a remote registry
    # docker push ${CONTAINER_REGISTRY}/master-service:${IMAGE_TAG}
    # docker push ${CONTAINER_REGISTRY}/worker:${IMAGE_TAG}
fi

echo "Build complete!"
