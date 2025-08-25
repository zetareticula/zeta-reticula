#!/bin/bash
set -e

echo "=== Cleaning up test resources ==="

# Delete test certificate
kubectl delete certificate test-certificate -n default --ignore-not-found
kubectl delete secret test-tls -n default --ignore-not-found

echo "Test resources have been cleaned up."
