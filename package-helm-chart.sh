#!/bin/bash
set -e

echo "📦 Packaging Zeta Reticula Helm Chart"
echo "====================================="

# Check if Helm is installed
if ! command -v helm &> /dev/null; then
    echo "❌ Helm is not installed. Please install Helm first."
    exit 1
fi

echo "✅ Helm is installed: $(helm version --short)"

# Create output directory
mkdir -p ./dist

# Package the chart
echo ""
echo "📦 Packaging chart..."
helm package ./charts/zeta-reticula -d ./dist

# Show the package
echo ""
echo "📋 Package created:"
ls -la ./dist/zeta-reticula-*.tgz

# Extract and validate
echo ""
echo "🔍 Validating package..."
helm lint ./dist/zeta-reticula-*.tgz

# Show package contents
echo ""
echo "📊 Package contents:"
tar -tzf ./dist/zeta-reticula-*.tgz | head -20

# Test installation (dry run)
echo ""
echo "🧪 Testing installation (dry run)..."
helm install zeta-reticula-test ./dist/zeta-reticula-*.tgz \
  --dry-run \
  --debug > /tmp/helm-install-test.yaml

echo "✅ Dry run completed successfully"

# Show generated resources
echo ""
echo "📋 Resources in dry run:"
grep -c "kind:" /tmp/helm-install-test.yaml

echo ""
echo "🎯 Installation command:"
echo "helm install zeta-reticula ./dist/zeta-reticula-*.tgz"
echo ""
echo "🗑️  Cleanup command:"
echo "helm uninstall zeta-reticula-test"

# Clean up test file
rm /tmp/helm-install-test.yaml

echo ""
echo "✅ Helm chart packaging completed successfully!"
echo ""
echo "🚀 Next steps:"
echo "1. Upload package to Helm repository: helm push ./dist/zeta-reticula-*.tgz <repo>"
echo "2. Install on cluster: helm install zeta-reticula ./dist/zeta-reticula-*.tgz"
echo "3. Test deployment: kubectl get pods -l app.kubernetes.io/name=zeta-reticula"
