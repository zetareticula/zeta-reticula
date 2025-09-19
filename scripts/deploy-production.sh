#!/bin/bash
set -euo pipefail

# Production Deployment Script for Zeta Reticula
# Usage: ./scripts/deploy-production.sh [environment]

# Configuration
ENVIRONMENT=${1:-staging}
CLUSTER_NAME="zeta-reticula-prod"
REGION="us-west-2"
NAMESPACE="zeta-prod"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🚀 Starting Zeta Reticula Production Deployment (${ENVIRONMENT})${NC}"

# Check for required tools
check_tools() {
    local tools=("kubectl" "helm" "aws" "terraform")
    for tool in "${tools[@]}"; do
        if ! command -v $tool &> /dev/null; then
            echo "Error: $tool is not installed"
            exit 1
        fi
    done
}

# Authenticate with the Kubernetes cluster
authenticate() {
    echo -e "\n${GREEN}🔐 Authenticating with EKS cluster...${NC}"
    aws eks --region $REGION update-kubeconfig --name $CLUSTER_NAME
    kubectl config set-context --current --namespace=$NAMESPACE
}

# Create namespace if it doesn't exist
setup_namespace() {
    echo -e "\n${GREEN}📦 Setting up Kubernetes namespace...${NC}"
    if ! kubectl get namespace $NAMESPACE &> /dev/null; then
        kubectl create namespace $NAMESPACE
        kubectl label namespace $NAMESPACE environment=production
    fi
}

# Deploy secrets
deploy_secrets() {
    echo -e "\n${GREEN}🔑 Deploying secrets...${NC}"
    # In production, use a secrets manager or sealed secrets
    kubectl create secret generic zeta-secrets \
        --from-literal=database-url=postgresql://user:${DB_PASSWORD}@postgres:5432/zetadb \
        --from-literal=redis-password=${REDIS_PASSWORD} \
        --dry-run=client -o yaml | kubectl apply -f -
}

# Deploy monitoring stack
deploy_monitoring() {
    echo -e "\n${GREEN}📊 Deploying monitoring stack...${NC}"
    helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
    helm repo update
    
    helm upgrade --install prometheus prometheus-community/kube-prometheus-stack \
        --namespace monitoring \
        --create-namespace \
        --values k8s/production/monitoring-values.yaml
}

# Deploy Redis cache
deploy_redis() {
    echo -e "\n${GREEN}🔴 Deploying Redis...${NC}"
    helm repo add bitnami https://charts.bitnami.com/bitnami
    helm upgrade --install zeta-redis bitnami/redis \
        --namespace $NAMESPACE \
        --values k8s/production/redis-values.yaml
}

# Deploy PostgreSQL
deploy_postgres() {
    echo -e "\n${GREEN}🐘 Deploying PostgreSQL...${NC}"
    helm repo add bitnami https://charts.bitnami.com/bitnami
    helm upgrade --install postgres bitnami/postgresql \
        --namespace $NAMESPACE \
        --values k8s/production/postgresql-values.yaml
}

# Deploy Zeta Reticula services
deploy_services() {
    echo -e "\n${GREEN}🚀 Deploying Zeta Reticula services...${NC}"
    
    # Build and push Docker images
    echo -e "\n${GREEN}🐳 Building and pushing Docker images...${NC}"
    docker build -t zeta-reticula/api:latest -f Dockerfile.api .
    docker build -t zeta-reticula/worker:latest -f Dockerfile.worker .
    
    # Apply Kubernetes manifests
    echo -e "\n${GREEN}🛠️  Applying Kubernetes manifests...${NC}"
    kubectl apply -f k8s/production/namespace.yaml
    kubectl apply -f k8s/production/configmap.yaml
    kubectl apply -f k8s/production/secrets.yaml
    kubectl apply -f k8s/production/deployment-api.yaml
    kubectl apply -f k8s/production/deployment-worker.yaml
    kubectl apply -f k8s/production/service.yaml
    kubectl apply -f k8s/production/ingress.yaml
    
    # Wait for deployments to be ready
    echo -e "\n${GREEN}⏳ Waiting for deployments to be ready...${NC}"
    kubectl rollout status deployment/zeta-api -n $NAMESPACE
    kubectl rollout status deployment/zeta-worker -n $NAMESPACE
}

# Verify deployment
verify_deployment() {
    echo -e "\n${GREEN}✅ Verifying deployment...${NC}"
    
    # Check pod status
    kubectl get pods -n $NAMESPACE
    
    # Test API endpoint
    API_URL=$(kubectl get svc zeta-api -n $NAMESPACE -o jsonpath='{.status.loadBalancer.ingress[0].hostname}')
    echo -e "\n${GREEN}🌐 Testing API endpoint: ${API_URL}${NC}"
    curl -s $API_URL/health | jq
    
    echo -e "\n${GREEN}🎉 Deployment successful!${NC}"
}

# Main deployment flow
main() {
    check_tools
    authenticate
    setup_namespace
    deploy_secrets
    deploy_monitoring
    deploy_redis
    deploy_postgres
    deploy_services
    verify_deployment
}

# Run the main function
main
