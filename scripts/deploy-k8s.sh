#!/bin/bash

# Exit on error
set -e

# Default values
ENV="dev"
NAMESPACE="zeta-reticula"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  key="$1"
  case $key in
    -e|--env)
      ENV="$2"
      shift # past argument
      shift # past value
      ;;
    -n|--namespace)
      NAMESPACE="$2"
      shift # past argument
      shift # past value
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Validate environment
if [[ ! -d "k8s/overlays/$ENV" ]]; then
  echo "Error: Environment '$ENV' not found in k8s/overlays/"
  exit 1
fi

echo "Deploying Zeta Reticula to $ENV environment in namespace $NAMESPACE..."

# Create namespace if it doesn't exist
kubectl create namespace $NAMESPACE --dry-run=client -o yaml | kubectl apply -f -

# Apply the configuration
kubectl apply -k k8s/overlays/$ENV

# Wait for deployments to be ready
echo "Waiting for deployments to be ready..."
kubectl -n $NAMESPACE wait --for=condition=available --timeout=300s deployment/master-service
echo "Master service is ready!"

kubectl -n $NAMESPACE wait --for=condition=available --timeout=300s deployment/worker
echo "Worker nodes are ready!"

kubectl -n $NAMESPACE wait --for=condition=available --timeout=300s deployment/api-service
echo "API service is ready!"

echo "\nZeta Reticula has been successfully deployed to the $ENV environment!"
echo "You can access the API service at: $(kubectl -n $NAMESPACE get svc api-service -o jsonpath='{.status.loadBalancer.ingress[0].ip}')"
