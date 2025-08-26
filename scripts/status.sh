#!/bin/bash
set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to check if a service is running
check_service() {
    local name=$1
    local port=$2
    local pid_file="../.pids/$name.pid"
    
    # Check if service is running on the port
    if nc -z 127.0.0.1 "$port" &>/dev/null; then
        echo -e "${GREEN}✓ $name is running on port $port${NC}"
        
        # Get process info if available
        if [ -f "$pid_file" ]; then
            local pid=$(cat "$pid_file" 2>/dev/null)
            if [ -n "$pid" ] && ps -p "$pid" > /dev/null; then
                echo -e "  Process ID: $pid"
                echo -e "  Memory usage: $(ps -o rss= -p "$pid" | awk '{printf "%.2f MB", $1/1024}')"
                echo -e "  CPU usage: $(ps -o %cpu= -p "$pid")%"
                
                # Check service health if there's a health endpoint
                if [ "$name" = "distributed-store" ] || [ "$name" = "master-service" ] || 
                   [ "$name" = "agentflow" ] || [ "$name" = "salience-engine" ]; then
                    local health_url="http://127.0.0.1:$port/health"
                    if curl -s -f "$health_url" >/dev/null 2>&1; then
                        local health_status=$(curl -s "$health_url" | jq -r '.status // .' 2>/dev/null || echo "unknown")
                        echo -e "  Health: ${GREEN}$health_status${NC}"
                    fi
                fi
            fi
        fi
        return 0
    else
        echo -e "${RED}✗ $name is not running on port $port${NC}"
        return 1
    fi
}

# Change to project root
cd "$(dirname "$0")/.."

# Source environment variables if available
if [ -f ".env" ]; then
    source .env
else
    echo -e "${YELLOW}Warning: .env file not found. Using default ports.${NC}"
    DISTRIBUTED_STORE_PORT=50051
    MASTER_SERVICE_PORT=50052
    AGENTFLOW_PORT=50053
    SALIENCE_ENGINE_PORT=50054
fi

echo -e "${YELLOW}=== Service Status ===${NC}\n"

# Check each service
check_service "Distributed Store" "$DISTRIBUTED_STORE_PORT"
echo
check_service "Master Service" "$MASTER_SERVICE_PORT"
echo
check_service "AgentFlow" "$AGENTFLOW_PORT"
echo
check_service "Salience Engine" "$SALIENCE_ENGINE_PORT"

# Check disk space
echo -e "\n${YELLOW}=== Disk Space ===${NC}"
df -h . | grep -v "^Filesystem" | awk '{print $4 " free on " $6}' | while read -r line; do
    echo -e "  ${GREEN}✓ $line${NC}"
done

# Check memory usage
echo -e "\n${YELLOW}=== Memory Usage ===${NC}"
free -h | grep -v "^ " | while read -r line; do
    echo -e "  ${GREEN}✓ $line${NC}"
done

# Check running processes
echo -e "\n${YELLOW}=== Running Processes ===${NC}"
if [ -d ".pids" ]; then
    for pid_file in .pids/*.pid; do
        if [ -f "$pid_file" ]; then
            local service_name=$(basename "$pid_file" .pid)
            local pid=$(cat "$pid_file" 2>/dev/null)
            if [ -n "$pid" ] && ps -p "$pid" > /dev/null; then
                echo -e "  ${GREEN}✓ $service_name (PID: $pid) - $(ps -o %cpu= -o %mem= -p "$pid" | awk '{print "CPU: " $1 "%, MEM: " $2 "%"}')${NC}"
            fi
        fi
    done
fi
