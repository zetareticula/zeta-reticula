#!/bin/bash
set -e

# Zeta Reticula Quantization Engine Deployment Script
# Integrates ns-router-rs, agentflow-rs, salience-engine, and kvquant

# Default values
ENV="development"
IMAGE_TAG="latest"
REGISTRY="docker.io/zetareticula"
SKIP_BUILD=false
SKIP_PUSH=false
ENABLE_GPU=false
MEMORY_LIMIT="8g"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  key="$1"
  case $key in
    -e|--environment)
      ENV="$2"
      shift 2
      ;;
    -t|--tag)
      IMAGE_TAG="$2"
      shift 2
      ;;
    -r|--registry)
      REGISTRY="$2"
      shift 2
      ;;
    --skip-build)
      SKIP_BUILD=true
      shift
      ;;
    --skip-push)
      SKIP_PUSH=true
      shift
      ;;
    --enable-gpu)
      ENABLE_GPU=true
      shift
      ;;
    --memory-limit)
      MEMORY_LIMIT="$2"
      shift 2
      ;;
    -h|--help)
      echo "Usage: $0 [OPTIONS]"
      echo "Options:"
      echo "  -e, --environment ENV     Deployment environment (development/staging/production)"
      echo "  -t, --tag TAG            Docker image tag"
      echo "  -r, --registry REGISTRY  Docker registry"
      echo "  --skip-build             Skip Docker build"
      echo "  --skip-push              Skip Docker push"
      echo "  --enable-gpu             Enable GPU support"
      echo "  --memory-limit LIMIT     Memory limit for containers"
      echo "  -h, --help               Show this help"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

echo "🚀 Deploying Zeta Reticula Quantization Engine"
echo "Environment: $ENV"
echo "Image Tag: $IMAGE_TAG"
echo "Registry: $REGISTRY"
echo "GPU Enabled: $ENABLE_GPU"
echo "Memory Limit: $MEMORY_LIMIT"

# Build Docker image
if [ "$SKIP_BUILD" = false ]; then
    echo "📦 Building Docker image..."
    
    # Build context includes all necessary components
    docker build \
        --build-arg ENABLE_GPU=$ENABLE_GPU \
        -t $REGISTRY/zeta-quantize:$IMAGE_TAG \
        -f Dockerfile \
        .
    
    echo "✅ Docker image built successfully"
fi

# Push to registry
if [ "$SKIP_PUSH" = false ]; then
    echo "📤 Pushing to registry..."
    docker push $REGISTRY/zeta-quantize:$IMAGE_TAG
    echo "✅ Image pushed successfully"
fi

# Deploy based on environment
case $ENV in
    "development")
        echo "🔧 Deploying to development environment..."
        docker-compose -f docker-compose.dev.yml up -d
        ;;
    "staging")
        echo "🎭 Deploying to staging environment..."
        kubectl apply -f k8s/staging/
        ;;
    "production")
        echo "🏭 Deploying to production environment..."
        kubectl apply -f k8s/production/
        ;;
    *)
        echo "❌ Unknown environment: $ENV"
        exit 1
        ;;
esac

echo "✅ Deployment completed successfully!"
echo "🔍 To check status:"
echo "  Development: docker-compose -f docker-compose.dev.yml ps"
echo "  Staging/Production: kubectl get pods -l app=zeta-quantize"
