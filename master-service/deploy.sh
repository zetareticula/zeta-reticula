#!/bin/bash
set -e

# Default values
ENV="staging"
IMAGE_TAG="latest"
REGISTRY="docker.io/zetareticula"
SKIP_BUILD=false
SKIP_PUSH=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  key="$1"
  case $key in
    -e|--environment)
      ENV="$2"
      shift # past argument
      shift # past value
      ;;
    -t|--tag)
      IMAGE_TAG="$2"
      shift # past argument
      shift # past value
      ;;
    -r|--registry)
      REGISTRY="$2"
      shift # past argument
      shift # past value
      ;;
    --skip-build)
      SKIP_BUILD=true
      shift # past argument
      ;;
    --skip-push)
      SKIP_PUSH=true
      shift # past argument
      ;;
    -h|--help)
      echo "Deploy the master service to Kubernetes"
      echo ""
      echo "Usage: $0 [options]"
      echo "  -e, --environment ENV   Deployment environment (staging|production), default: staging"
      echo "  -t, --tag TAG           Docker image tag, default: latest"
      echo "  -r, --registry REGISTRY  Docker registry, default: docker.io/zetareticula"
      echo "  --skip-build            Skip building the Docker image"
      echo "  --skip-push             Skip pushing the Docker image to the registry"
      echo "  -h, --help              Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Validate environment
if [[ "$ENV" != "staging" && "$ENV" != "production" ]]; then
  echo "Error: Environment must be either 'staging' or 'production'"
  exit 1
fi

# Set image name and tag
IMAGE_NAME="${REGISTRY}/master-service:${IMAGE_TAG}"

# Build the Docker image
if [ "$SKIP_BUILD" = false ]; then
  echo "Building Docker image: $IMAGE_NAME"
  docker build -t "$IMAGE_NAME" .
  
  if [ $? -ne 0 ]; then
    echo "Error: Docker build failed"
    exit 1
  fi
  
  echo "Successfully built $IMAGE_NAME"
fi

# Push the Docker image
if [ "$SKIP_PUSH" = false ]; then
  echo "Pushing Docker image to registry..."
  docker push "$IMAGE_NAME"
  
  if [ $? -ne 0 ]; then
    echo "Error: Docker push failed"
    exit 1
  fi
  
  echo "Successfully pushed $IMAGE_NAME"
fi

# Apply Kubernetes manifests
echo "Deploying to $ENV environment..."

# Create namespace if it doesn't exist
kubectl create namespace $ENV --dry-run=client -o yaml | kubectl apply -f -

# Apply the Kubernetes manifests using kustomize
kubectl kustomize "k8s/overlays/$ENV" | \
  sed "s|docker.io/zetareticula/master-service:.*|$IMAGE_NAME|" | \
  kubectl apply -f -

# Wait for deployment to complete
echo "Waiting for deployment to complete..."
kubectl rollout status deployment/master-service -n $ENV --timeout=300s

if [ $? -eq 0 ]; then
  echo "Deployment to $ENV environment completed successfully!"
  
  # Get service URL
  if [ "$ENV" = "production" ]; then
    echo ""
    echo "Service endpoints:"
    kubectl get svc master-service -n $ENV -o jsonpath='{.status.loadBalancer.ingress[0].hostname}' 2>/dev/null || \
      kubectl get svc master-service -n $ENV -o jsonpath='{.status.loadBalancer.ingress[0].ip}'
  fi
else
  echo "Error: Deployment failed"
  exit 1
fi
