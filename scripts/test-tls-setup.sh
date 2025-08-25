#!/bin/bash
set -e

# Test cert-manager installation
echo "=== Testing cert-manager installation ==="
kubectl get pods -n cert-manager
kubectl get crd | grep cert-manager.io

echo -e "\n=== Testing ClusterIssuers ==="
kubectl get clusterissuers

# Create a test certificate to verify the setup
echo -e "\n=== Creating test certificate ==="
cat <<EOF | kubectl apply -f -
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: test-certificate
  namespace: default
spec:
  secretName: test-tls
  issuerRef:
    name: letsencrypt-staging
    kind: ClusterIssuer
  commonName: test.example.com
  dnsNames:
    - test.example.com
EOF

echo -e "\n=== Waiting for certificate to be ready (this may take a few minutes) ==="
for i in {1..12}; do
  echo "Checking certificate status (attempt $i/12)..."
  STATUS=$(kubectl get certificate test-certificate -n default -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}' 2>/dev/null || echo "Unknown")
  
  if [ "$STATUS" == "True" ]; then
    echo "Certificate is ready!"
    kubectl get certificate test-certificate -n default
    exit 0
  elif [ "$STATUS" == "False" ]; then
    echo "Certificate failed to issue. Check cert-manager logs for details."
    kubectl describe certificate test-certificate -n default
    exit 1
  fi
  
  sleep 5
done

echo "Timed out waiting for certificate to be ready. Check cert-manager logs for details." 
echo "Current certificate status:"
kubectl describe certificate test-certificate -n default
exit 1
