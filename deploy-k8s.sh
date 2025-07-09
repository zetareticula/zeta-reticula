#!/bin/bash
set -e

# Colors for output
cyan=\033[0;36m
red=\033[0;31m
green=\033[0;32m
yellow=\033[0;33m
reset=\033[0m

function log() {
    echo -e "${cyan}==> $1${reset}"
}

function error() {
    echo -e "${red}==> $1${reset}"
    exit 1
}

function success() {
    echo -e "${green}==> $1${reset}"
}

function warning() {
    echo -e "${yellow}==> $1${reset}"
}

function check_prerequisites() {
    log "Checking prerequisites..."
    
    # Check if minikube is installed
    if ! command -v minikube &> /dev/null; then
        error "Minikube is not installed. Please install it first: brew install minikube"
    fi

    # Check if helm is installed
    if ! command -v helm &> /dev/null; then
        error "Helm is not installed. Please install it first: brew install helm"
    fi

    # Check if docker is installed
    if ! command -v docker &> /dev/null; then
        error "Docker is not installed. Please install Docker Desktop from https://www.docker.com/products/docker-desktop/"
    fi
}

function start_minikube() {
    log "Starting Minikube..."
    
    # Start Minikube with recommended resources
    minikube start --driver=docker --cpus=4 --memory=8192 --disk-size=40g || {
        error "Failed to start Minikube. Please check your system resources and try again."
    }

    # Wait for Minikube to be ready
    log "Waiting for Minikube to be ready..."
    minikube status --wait=all
}

function switch_to_minikube_docker() {
    log "Switching to Minikube's Docker environment..."
    eval $(minikube docker-env)
}

function build_images() {
    log "Building Docker images..."
    
    # List of services to build
    local SERVICES=("agentflow-rs" "llm-rs" "ns-router-rs" "salience-engine" "kvquant-rs" "api" "zeta-sidecar")

    # Build each service
    for SERVICE in "${SERVICES[@]}"; do
        log "Building $SERVICE..."
        docker build -t "zetareticula/$SERVICE:local" -f Dockerfile --build-arg SERVICE=$SERVICE .
    done
}

function deploy_to_kubernetes() {
    log "Deploying to Kubernetes..."
    
    # Create local values file if it doesn't exist
    if [ ! -f "charts/zeta-reticula/values.local.yaml" ]; then
        cat > "charts/zeta-reticula/values.local.yaml" << EOF
image:
  repository: zetareticula
  tag: "local"
  pullPolicy: Never

service:
  type: NodePort

ingress:
  enabled: false
EOF
    fi

    # Deploy the chart
    helm upgrade --install zeta-reticula ./charts/zeta-reticula -f ./charts/zeta-reticula/values.local.yaml
}

function verify_deployment() {
    log "Verifying deployment..."
    
    # Wait for pods to be ready
    log "Waiting for pods to be ready..."
    kubectl wait --for=condition=ready pod --all --timeout=300s

    # Show pod status
    log "Pod status:"
    kubectl get pods

    # Show services
    log "Services:"
    kubectl get svc
}

function main() {
    check_prerequisites
    start_minikube
    switch_to_minikube_docker
    build_images
    deploy_to_kubernetes
    verify_deployment
    
    success "Deployment completed successfully!"
}

main "$@"
