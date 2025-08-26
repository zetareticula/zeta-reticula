#!/bin/bash

# Setup script for Zeta Reticula Airflow
set -e

# Set environment variables
export AIRFLOW_UID=50000
export AIRFLOW_PROJ_DIR="$(pwd)"

# Create necessary directories
mkdir -p ./dags ./logs ./plugins ./config
echo "Created required directories"

# Set the right permissions for Airflow
chmod -R 777 ./logs
chmod -R 777 ./dags
chmod -R 777 ./plugins

echo "Setting up Airflow with Docker Compose..."

# Initialize the database
echo "Initializing Airflow database..."
docker-compose up airflow-init

# Start all services
echo "Starting Airflow services..."
docker-compose up -d

echo "Waiting for Airflow to be ready..."
until curl -s -f "http://localhost:8080/health" >/dev/null 2>&1; do
  echo -n "."
  sleep 5
done

echo -e "\nAirflow is now ready!"
echo "- Web UI: http://localhost:8080"
echo "- Username: admin"
echo "- Password: admin"

# Create Kubernetes connection in Airflow
echo "Setting up Kubernetes connection in Airflow..."
KUBE_CONFIG="${HOME}/.kube/config"
if [ -f "$KUBE_CONFIG" ]; then
    docker-compose exec -T airflow-webserver airflow connections add \
        'kubernetes_default' \
        --conn-type 'kubernetes' \
        --conn-extra "$(jq -n --arg kubeconfig "$(base64 < "$KUBE_CONFIG" | tr -d '\n')" '{"kube_config": $kubeconfig}' | base64)" \
        --conn-description 'Kubernetes connection for running pods in the cluster'
    echo "Kubernetes connection 'kubernetes_default' created successfully!"
else
    echo "Warning: Kubernetes config file not found at $KUBE_CONFIG"
    echo "Please create the Kubernetes connection manually in the Airflow UI"
fi

echo "\nSetup complete!"
