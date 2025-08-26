#!/bin/bash

# Script to manage Airflow DAGs (validate, test, deploy)

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

# Function to validate DAG files
validate_dags() {
    local dags_dir="./dags"
    local has_errors=false
    
    echo -e "${YELLOW}Validating DAG files in $dags_dir...${NC}"
    
    for dag_file in "$dags_dir"/*.py; do
        if [ -f "$dag_file" ]; then
            echo -n "Validating $(basename "$dag_file")... "
            
            # Check Python syntax
            if ! python3 -m py_compile "$dag_file"; then
                echo -e "${RED}✗ Syntax error${NC}"
                has_errors=true
                continue
            fi
            
            # Check for common DAG issues
            local dag_id=$(basename "$dag_file" .py)
            local validation_output
            validation_output=$(python3 -c "
import os
import sys
from airflow.models import DagBag

dag_bag = DagBag('$dags_dir')
if '$dag_id' not in dag_bag.dags:
    print('DAG $dag_id not found in DAG bag')
    sys.exit(1)

dag = dag_bag.get_dag('$dag_id')
if not dag:
    print('Failed to load DAG $dag_id')
    sys.exit(1)

# Check for cycles
from airflow.utils.dag_cycle_tester import test_cycle
try:
    test_cycle(dag)
except Exception as e:
    print(f'Cycle detected: {str(e)}')
    sys.exit(1)

print('OK')
sys.exit(0)
" 2>&1)
            
            if [ $? -ne 0 ]; then
                echo -e "${RED}✗ ${validation_output}${NC}"
                has_errors=true
            else
                echo -e "${GREEN}✓ Valid${NC}"
            fi
        fi
    done
    
    if [ "$has_errors" = true ]; then
        echo -e "${RED}DAG validation failed${NC}"
        return 1
    else
        echo -e "${GREEN}All DAGs validated successfully${NC}"
        return 0
    fi
}

# Function to test a specific DAG
test_dag() {
    local dag_id=$1
    
    if [ -z "$dag_id" ]; then
        echo "Error: DAG ID is required"
        echo "Usage: $0 test-dag <dag_id>"
        exit 1
    fi
    
    echo -e "${YELLOW}Testing DAG: $dag_id${NC}"
    
    # Test DAG loading
    local load_output
    load_output=$(docker-compose exec -T airflow-webserver airflow dags list | grep "$dag_id" || true)
    
    if [ -z "$load_output" ]; then
        echo -e "${RED}Error: DAG '$dag_id' not found${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}DAG loaded successfully${NC}"
    
    # Test tasks
    echo "Testing tasks..."
    if ! docker-compose exec -T airflow-webserver airflow tasks test "$dag_id" list; then
        echo -e "${RED}Error: Failed to list tasks for DAG '$dag_id'${NC}"
        exit 1
    fi
    
    # Test task rendering
    echo -e "\nTesting task rendering..."
    if ! docker-compose exec -T airflow-webserver airflow tasks render "$dag_id"; then
        echo -e "${RED}Error: Failed to render tasks for DAG '$dag_id'${NC}"
        exit 1
    fi
    
    echo -e "\n${GREEN}DAG test completed successfully${NC}"
}

# Function to deploy DAGs
deploy_dags() {
    local dags_dir="./dags"
    local target_dir="/opt/airflow/dags"
    
    echo -e "${YELLOW}Deploying DAGs to Airflow...${NC}"
    
    # Validate DAGs before deployment
    if ! validate_dags; then
        echo -e "${RED}Deployment aborted: DAG validation failed${NC}"
        return 1
    fi
    
    # Copy DAGs to the DAGs folder
    echo "Copying DAGs to $target_dir..."
    
    # Create necessary directories
    docker-compose exec -T airflow-webserver mkdir -p "$target_dir"
    
    # Copy files
    for file in "$dags_dir"/*; do
        if [ -f "$file" ]; then
            local filename=$(basename "$file")
            echo "Deploying $filename..."
            docker cp "$file" "$(docker-compose ps -q airflow-webserver):$target_dir/"
        fi
    done
    
    # Set permissions
    docker-compose exec -T airflow-webserver chown -R airflow:airflow "$target_dir"
    
    echo -e "\n${GREEN}DAGs deployed successfully!${NC}"
    echo "Airflow will automatically detect and load the new DAGs."
    echo "You can check the status in the Airflow UI or with 'docker-compose logs -f airflow-webserver'"
}

# Function to list all DAGs
list_dags() {
    echo -e "${YELLOW}Listing all DAGs:${NC}"
    docker-compose exec -T airflow-webserver airflow dags list
}

# Function to get DAG details
dag_details() {
    local dag_id=$1
    
    if [ -z "$dag_id" ]; then
        echo "Error: DAG ID is required"
        echo "Usage: $0 dag-details <dag_id>"
        exit 1
    fi
    
    echo -e "${YELLOW}Details for DAG: $dag_id${NC}"
    docker-compose exec -T airflow-webserver airflow dags show "$dag_id"
}

# Function to pause/unpause a DAG
toggle_dag() {
    local dag_id=$1
    local action=$2
    
    if [ -z "$dag_id" ] || [ -z "$action" ]; then
        echo "Error: DAG ID and action are required"
        echo "Usage: $0 toggle-dag <dag_id> <pause|unpause>"
        exit 1
    fi
    
    case $action in
        pause|unpause)
            echo -e "${YELLOW}${action}ing DAG: $dag_id${NC}"
            docker-compose exec -T airflow-webserver airflow dags "$action" "$dag_id"
            ;;
        *)
            echo "Error: Invalid action. Must be 'pause' or 'unpause'"
            exit 1
            ;;
    esac
}

# Main script
case "$1" in
    validate)
        validate_dags
        ;;
    test-dag)
        test_dag "$2"
        ;;
    deploy)
        deploy_dags
        ;;
    list)
        list_dags
        ;;
    dag-details)
        dag_details "$2"
        ;;
    toggle-dag)
        toggle_dag "$2" "$3"
        ;;
    *)
        echo "Usage: $0 {validate|test-dag|deploy|list|dag-details|toggle-dag}"
        echo "  validate                  Validate all DAGs in the dags/ directory"
        echo "  test-dag <dag_id>         Test a specific DAG"
        echo "  deploy                    Deploy DAGs to Airflow"
        echo "  list                      List all DAGs"
        echo "  dag-details <dag_id>      Show details for a specific DAG"
        echo "  toggle-dag <dag_id> <pause|unpause>  Pause or unpause a DAG"
        exit 1
        ;;
esac
