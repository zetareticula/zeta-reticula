#!/bin/bash
set -e

echo "🧪 Testing Zeta Reticula Helm Chart Locally"
echo "=========================================="

# Check if Helm is installed
if ! command -v helm &> /dev/null; then
    echo "❌ Helm is not installed. Please install Helm first."
    exit 1
fi

echo "✅ Helm is installed: $(helm version --short)"

# Validate the Helm chart
echo ""
echo "🔍 Validating Helm chart..."
helm lint ./charts/zeta-reticula

# Test template rendering
echo ""
echo "🎨 Testing template rendering..."
helm template zeta-reticula-test ./charts/zeta-reticula \
  --values ./charts/zeta-reticula/values.yaml \
  --debug > /tmp/zeta-reticula-test.yaml

echo "✅ Template rendered successfully"

# Show key resources
echo ""
echo "📋 Generated resources:"
grep -c "kind:" /tmp/zeta-reticula-test.yaml

echo ""
echo "📊 Summary:"
echo "- Deployment: $(grep -c "kind: Deployment" /tmp/zeta-reticula-test.yaml)"
echo "- Service: $(grep -c "kind: Service" /tmp/zeta-reticula-test.yaml)"
echo "- ConfigMap: $(grep -c "kind: ConfigMap" /tmp/zeta-reticula-test.yaml)"
echo "- Secret: $(grep -c "kind: Secret" /tmp/zeta-reticula-test.yaml)"

# Clean up
rm /tmp/zeta-reticula-test.yaml

echo ""
echo "✅ Helm chart test completed successfully!"
echo ""
echo "🚀 To install locally:"
echo "   helm install zeta-reticula ./charts/zeta-reticula"
echo ""
echo "📦 To install with production values:"
echo "   helm install zeta-reticula ./charts/zeta-reticula -f ./charts/zeta-reticula/values-production.yaml"
echo ""
echo "🎯 To view available values:"
echo "   helm show values ./charts/zeta-reticula"
