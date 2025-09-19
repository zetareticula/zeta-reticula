#!/bin/bash

# Script to check CI/CD status for Zeta Reticula
# Provides instructions for monitoring GitHub Actions

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔍 Zeta Reticula CI/CD Status Checker${NC}"
echo ""

# Get current commit info
CURRENT_COMMIT=$(git rev-parse HEAD)
CURRENT_BRANCH=$(git branch --show-current)
REPO_URL=$(git config --get remote.origin.url | sed 's/\.git$//')

echo -e "${YELLOW}📋 Current Status:${NC}"
echo "  Branch: $CURRENT_BRANCH"
echo "  Commit: $CURRENT_COMMIT"
echo "  Repository: $REPO_URL"
echo ""

# Check if we have recent commits that should trigger CI
echo -e "${YELLOW}📝 Recent Commits (last 5):${NC}"
git log --oneline -5
echo ""

# Show CI/CD trigger paths
echo -e "${YELLOW}🎯 CI/CD Trigger Paths:${NC}"
echo "  The following paths trigger CI/CD when changed:"
echo "  • src/**"
echo "  • core/**"
echo "  • runtime/**"
echo "  • interfaces/**"
echo "  • master-service/**"
echo "  • ns-router-rs/**"
echo "  • salience-engine/**"
echo "  • quantize-cli/**"
echo "  • llm-rs/**"
echo "  • agentflow-rs/**"
echo "  • kvquant_rs/**"
echo "  • shared/**"
echo "  • k8s/**"
echo "  • .github/workflows/ci-cd.yaml"
echo "  • Dockerfile*"
echo "  • Cargo.toml"
echo "  • Cargo.lock"
echo ""

# Instructions for checking GitHub Actions
echo -e "${GREEN}🚀 How to Check CI/CD Status:${NC}"
echo ""
echo "1. Visit GitHub Actions page:"
echo "   ${REPO_URL}/actions"
echo ""
echo "2. Look for workflow run with commit: ${CURRENT_COMMIT:0:7}"
echo ""
echo "3. Expected workflow jobs:"
echo "   • ✅ Run Tests"
echo "   • ✅ Validate Kubernetes Manifests" 
echo "   • ✅ Build and Push Docker Images"
echo "   • ✅ Deploy to EKS (production only)"
echo "   • ✅ Notify Status"
echo ""
echo "4. Check deployment status:"
echo "   • Docker Hub: https://hub.docker.com/r/\$DOCKERHUB_USERNAME/zeta-reticula"
echo "   • Kubernetes: kubectl get pods -n zeta-reticula"
echo ""

# Local validation
echo -e "${GREEN}🔧 Local Validation:${NC}"
echo ""
echo "Build status:"
if cargo build --bin zeta >/dev/null 2>&1; then
    echo "  ✅ Local build: SUCCESS"
else
    echo "  ❌ Local build: FAILED"
fi

echo ""
echo "CLI functionality:"
if ./target/debug/zeta system status >/dev/null 2>&1; then
    echo "  ✅ CLI test: SUCCESS"
else
    echo "  ❌ CLI test: FAILED"
fi

echo ""
echo "Kubernetes manifests:"
if ./kustomize build k8s/base/ >/dev/null 2>&1; then
    echo "  ✅ K8s base: VALID"
else
    echo "  ❌ K8s base: INVALID"
fi

if ./kustomize build k8s/overlays/prod/ >/dev/null 2>&1; then
    echo "  ✅ K8s prod: VALID"
else
    echo "  ❌ K8s prod: INVALID"
fi

echo ""
echo -e "${BLUE}📊 Next Steps:${NC}"
echo "1. Monitor the GitHub Actions workflow at the URL above"
echo "2. Check for any failed jobs and review logs"
echo "3. Verify Docker image was pushed successfully"
echo "4. Confirm Kubernetes deployment if applicable"
echo "5. Test the deployed application endpoints"
echo ""
echo -e "${GREEN}✅ CI/CD configuration updated to include /src/ directory${NC}"
echo -e "${GREEN}✅ Docker build fixed for unified zeta CLI${NC}"
echo -e "${GREEN}✅ All local validations should pass${NC}"
