#!/bin/bash
set -e

# Set environment variables
export DOCKER_BUILDKIT=1
IMAGE_NAME="zeta-reticula-master"
TAG="latest"

# Build the Docker image from the master-service directory
echo "Building $IMAGE_NAME:$TAG from master-service directory..."
cd "$(dirname "$0")/../master-service"

docker build \
    -t $IMAGE_NAME:$TAG \
    -f Dockerfile \
    ..

echo "Successfully built $IMAGE_NAME:$TAG"

# If running in Minikube, load the image
if command -v minikube &> /dev/null && minikube status &> /dev/null; then
    echo "Loading image into Minikube..."
    minikube image load $IMAGE_NAME:$TAG
    echo "Image loaded into Minikube"
fi
