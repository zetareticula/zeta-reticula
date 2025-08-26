#!/bin/bash

# Verify the deployment of Zeta Reticula components

set -e

echo "🔍 Verifying Zeta Reticula deployment..."

# Check if kubectl is installed
if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl is not installed. Please install it first."
    exit 1
fi

# Check Kubernetes cluster status
echo "\n🌐 Checking Kubernetes cluster status..."
kubectl cluster-info

# Check NS Router deployment
echo "\n🔄 Checking NS Router deployment..."
kubectl get deployment -n zeta-reticula ns-router

# Check pods
echo "\n📦 Checking pods..."
kubectl get pods -n zeta-reticula

# Check services
echo "\n🔌 Checking services..."
kubectl get svc -n zeta-reticula

# Verify p2pstore is running
echo "\n📚 Verifying p2pstore..."
if [ -f "target/release/p2pstore" ]; then
    echo "✅ p2pstore is built and ready"
else
    echo "⚠️  p2pstore not found. Building..."
    cargo build --release --bin p2pstore
fi

echo "\n✅ Verification complete!"
echo "\nNext steps:"
echo "1. Add your models to the 'models' directory"
echo "2. Run './scripts/quantize_models.sh' to quantize models"
echo "3. Access the NS Router at: $(kubectl get svc -n zeta-reticula ns-router -o jsonpath='{.status.loadBalancer.ingress[0].ip}')"
