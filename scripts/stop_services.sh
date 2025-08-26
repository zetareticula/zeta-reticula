#!/bin/bash
set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to stop a service
stop_service() {
    local name=$1
    local pid_file="../.pids/$name.pid"
    
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if ps -p "$pid" > /dev/null; then
            echo -e "${YELLOW}Stopping $name (PID: $pid)...${NC}"
            kill "$pid" 2>/dev/null || true
            
            # Wait for the process to terminate
            local count=0
            while ps -p "$pid" > /dev/null && [ "$count" -lt 10 ]; do
                sleep 1
                count=$((count + 1))
            done
            
            # Force kill if still running
            if ps -p "$pid" > /dev/null; then
                echo -e "${RED}Force stopping $name (PID: $pid)...${NC}"
                kill -9 "$pid" 2>/dev/null || true
            fi
            
            echo -e "${GREEN}Stopped $name${NC}"
        fi
        rm -f "$pid_file"
    else
        echo -e "${YELLOW}$name is not running (no PID file found)${NC}"
    fi
}

# Change to project root
cd "$(dirname "$0")/.."

echo -e "${YELLOW}=== Stopping Services ===${NC}"

# Stop services in reverse order of dependencies
stop_service "salience-engine"
stop_service "agentflow"
stop_service "master-service"
stop_service "distributed-store"

echo -e "\n${GREEN}All services have been stopped.${NC}"
