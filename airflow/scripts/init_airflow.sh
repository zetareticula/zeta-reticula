#!/bin/bash

# Initialize Airflow database and set up Kubernetes executor
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Load environment variables
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
fi

# Check if Airflow is already initialized
if [ -f "airflow/airflow-webserver.pid" ]; then
    echo -e "${YELLOW}Airflow appears to be already initialized.${NC}"
    read -p "Do you want to re-initialize the database? This will delete all data. [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}Aborting.${NC}"
        exit 0
    fi
    
    echo -e "${YELLOW}Stopping Airflow services...${NC}"
    docker-compose down -v
    
    echo -e "${YELLOW}Removing existing database...${NC}"
    docker volume rm -f $(docker volume ls -q | grep airflow_postgres) 2>/dev/null || true
    
    echo -e "${YELLOW}Removing existing logs...${NC}"
    rm -rf logs/*
fi

# Start PostgreSQL
echo -e "\n${YELLOW}Starting PostgreSQL...${NC}"
docker-compose up -d postgres

# Wait for PostgreSQL to be ready
echo -e "\n${YELLOW}Waiting for PostgreSQL to be ready...${NC}"
until docker-compose exec -T postgres pg_isready -U airflow; do
    sleep 1
done

# Initialize the database
echo -e "\n${YELLOW}Initializing Airflow database...${NC}"
docker-compose run --rm airflow-webserver airflow db init

# Create admin user
echo -e "\n${YELLOW}Creating admin user...${NC}"
docker-compose run --rm airflow-webserver airflow users create \
    --username admin \
    --firstname Admin \
    --lastname User \
    --role Admin \
    --email admin@example.com \
    --password admin

# Set up connections
echo -e "\n${YELLOW}Setting up connections...${NC}"

# Kubernetes connection
KUBE_CONFIG="${HOME}/.kube/config"
if [ -f "$KUBE_CONFIG" ]; then
    echo "Setting up Kubernetes connection..."
    docker-compose exec -T airflow-webserver airflow connections add \
        'kubernetes_default' \
        --conn-type 'kubernetes' \
        --conn-extra "$(jq -n --arg kubeconfig "$(base64 < "$KUBE_CONFIG" | tr -d '\n')" '{"kube_config": $kubeconfig}' | base64)" \
        --conn-description 'Kubernetes connection for running pods in the cluster'
    echo -e "${GREEN}✓ Kubernetes connection created${NC}"
else
    echo -e "${YELLOW}Warning: Kubernetes config file not found at $KUBE_CONFIG${NC}"
    echo "Please create the Kubernetes connection manually in the Airflow UI"
fi

# PostgreSQL connection
echo "Setting up PostgreSQL connection..."
docker-compose exec -T airflow-webserver airflow connections add \
    'postgres_default' \
    --conn-type 'postgres' \
    --conn-login 'airflow' \
    --conn-password 'airflow' \
    --conn-host 'postgres' \
    --conn-port '5432' \
    --conn-schema 'airflow'
echo -e "${GREEN}✓ PostgreSQL connection created${NC}"

# Start all services
echo -e "\n${YELLOW}Starting all Airflow services...${NC}"
docker-compose up -d

# Wait for Airflow web server to be ready
echo -e "\n${YELLOW}Waiting for Airflow web server to be ready...${NC}"
until curl -s -f "http://localhost:8080/health" >/dev/null 2>&1; do
    echo -n "."
    sleep 5
done

# Import variables if sample file exists
if [ -f "config/sample_variables.json" ]; then
    echo -e "\n${YELLOW}Importing variables...${NC}"
    docker cp "config/sample_variables.json" "$(docker-compose ps -q airflow-webserver):/tmp/variables.json"
    docker-compose exec -T airflow-webserver airflow variables import /tmp/variables.json
    docker-compose exec -T airflow-webserver rm /tmp/variables.json
    echo -e "${GREEN}✓ Variables imported${NC}"
fi

# Print success message
echo -e "\n${GREEN}✓ Airflow initialization complete!${NC}"
echo -e "\n${YELLOW}Airflow is now running:${NC}"
echo -e "- Web UI: ${GREEN}http://localhost:8080${NC}"
echo -e "- Username: ${GREEN}admin${NC}"
echo -e "- Password: ${GREEN}admin${NC}"
echo -e "\n${YELLOW}To stop Airflow, run:${NC} docker-compose down"
echo -e "${YELLOW}To view logs, run:${NC} docker-compose logs -f"

# Print next steps
echo -e "\n${YELLOW}Next steps:${NC}"
echo "1. Log in to the Airflow UI"
echo "2. Configure any additional connections in Admin > Connections"
echo "3. Unpause your DAGs to start scheduling"
echo -e "4. Run ${GREEN}make test${NC} to run tests"
echo -e "5. Run ${GREEN}make lint${NC} to check code quality"

# Make the script executable
chmod +x "$0"
