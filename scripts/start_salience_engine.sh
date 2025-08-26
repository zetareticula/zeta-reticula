#!/bin/bash
set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Load environment variables
if [ -f "../.env" ]; then
    source ../.env
else
    echo -e "${RED}Error: .env file not found in the project root${NC}"
    exit 1
fi

# Function to check if a service is running
is_running() {
    local name=$1
    local port=$2
    if nc -z 127.0.0.1 "$port" &>/dev/null; then
        echo -e "${GREEN}✓ $name is running on port $port${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠ $name is not running on port $port${NC}"
        return 1
    fi
}

# Function to start a service in the background
start_service() {
    local name=$1
    local cmd=$2
    local log_file="../logs/$name.log"
    
    echo -e "${YELLOW}Starting $name...${NC}"
    mkdir -p "$(dirname "$log_file")"
    
    # Start the service in the background
    eval "$cmd" >> "$log_file" 2>&1 &
    local pid=$!
    
    # Save the PID to a file
    mkdir -p ../.pids
    echo $pid > "../.pids/$name.pid"
    
    echo -e "${GREEN}Started $name (PID: $pid, Log: $log_file)${NC}"
    return 0
}

# Change to project root
cd "$(dirname "$0")/.."

# Create necessary directories
mkdir -p logs
mkdir -p .pids

# Check if services are already running
echo -e "${YELLOW}=== Checking Services ===${NC}"

is_running "Distributed Store" "$DISTRIBUTED_STORE_PORT" || {
    # Start distributed store
    cd distributed-store
    start_service "distributed-store" \
        "cargo run --release -- \
        --host $DISTRIBUTED_STORE_HOST \
        --port $DISTRIBUTED_STORE_PORT \
        --replication $DISTRIBUTED_STORE_REPLICATION_FACTOR \
        --data-dir ../data/kvstore"
    cd ..
    
    # Wait for the distributed store to be ready
    echo -e "${YELLOW}Waiting for Distributed Store to be ready...${NC}"
    sleep 5
}

is_running "Master Service" "$MASTER_SERVICE_PORT" || {
    # Start master service
    cd master-service
    start_service "master-service" \
        "cargo run --release -- \
        --host $MASTER_SERVICE_HOST \
        --port $MASTER_SERVICE_PORT \
        --distributed-store-url http://$DISTRIBUTED_STORE_HOST:$DISTRIBUTED_STORE_PORT"
    cd ..
    
    # Wait for the master service to be ready
    echo -e "${YELLOW}Waiting for Master Service to be ready...${NC}"
    sleep 3
}

is_running "AgentFlow" "$AGENTFLOW_PORT" || {
    # Start AgentFlow
    cd agentflow-rs
    start_service "agentflow" \
        "cargo run --release -- \
        --host $AGENTFLOW_HOST \
        --port $AGENTFLOW_PORT \
        --master-service-url http://$MASTER_SERVICE_HOST:$MASTER_SERVICE_PORT"
    cd ..
    
    # Wait for AgentFlow to be ready
    echo -e "${YELLOW}Waiting for AgentFlow to be ready...${NC}"
    sleep 3
}

is_running "Salience Engine" "$SALIENCE_ENGINE_PORT" || {
    # Start Salience Engine
    cd salience-engine
    start_service "salience-engine" \
        "cargo run --release -- \
        --host $SALIENCE_ENGINE_HOST \
        --port $SALIENCE_ENGINE_PORT \
        --model-path $MODEL_PATH \
        --model-type $MODEL_TYPE \
        --distributed-store-url http://$DISTRIBUTED_STORE_HOST:$DISTRIBUTED_STORE_PORT \
        --master-service-url http://$MASTER_SERVICE_HOST:$MASTER_SERVICE_PORT \
        --num-workers $NUM_WORKERS \
        --token-chunk-size $TOKEN_CHUNK_SIZE"
    cd ..
}

# Print status
echo -e "\n${GREEN}=== System Status ===${NC}"
is_running "Distributed Store" "$DISTRIBUTED_STORE_PORT"
is_running "Master Service" "$MASTER_SERVICE_PORT"
is_running "AgentFlow" "$AGENTFLOW_PORT"
is_running "Salience Engine" "$SALIENCE_ENGINE_PORT"

echo -e "\n${GREEN}=== Access Information ===${NC}"
echo "- Distributed Store: http://$DISTRIBUTED_STORE_HOST:$DISTRIBUTED_STORE_PORT"
echo "- Master Service: http://$MASTER_SERVICE_HOST:$MASTER_SERVICE_PORT"
echo "- AgentFlow: http://$AGENTFLOW_HOST:$AGENTFLOW_PORT"
echo "- Salience Engine: http://$SALIENCE_ENGINE_HOST:$SALIENCE_ENGINE_PORT"
echo -e "\n${YELLOW}Logs are available in the 'logs' directory.${NC}"

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}Shutting down services...${NC}"
    
    # Kill all services
    if [ -d ".pids" ]; then
        for pid_file in .pids/*.pid; do
            if [ -f "$pid_file" ]; then
                local service_name=$(basename "$pid_file" .pid)
                local pid=$(cat "$pid_file")
                if ps -p "$pid" > /dev/null; then
                    echo -e "${YELLOW}Stopping $service_name (PID: $pid)...${NC}"
                    kill "$pid" 2>/dev/null || true
                fi
                rm -f "$pid_file"
            fi
        done
    fi
    
    echo -e "${GREEN}All services have been stopped.${NC}"
}

# Set up trap to catch termination signals
trap cleanup INT TERM

# Keep the script running
wait
