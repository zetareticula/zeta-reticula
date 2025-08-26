#!/bin/bash
set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to display usage
usage() {
    echo -e "${YELLOW}Usage: $0 [command]${NC}"
    echo ""
    echo "Available commands:"
    echo "  start         Start all services"
    echo "  stop          Stop all services"
    echo "  restart       Restart all services"
    echo "  status        Show status of all services"
    echo "  logs [service] View logs for a specific service or all services"
    echo "  clean         Clean up temporary files and resources"
    echo "  update        Update the system (git pull and rebuild)"
    echo "  help          Show this help message"
    echo ""
    echo "Services: distributed-store, master-service, agentflow, salience-engine"
    exit 1
}

# Function to display header
header() {
    echo -e "\n${BLUE}=== Zeta Reticula Management System ===${NC}\n"
}

# Change to project root
cd "$(dirname "$0")/.."

# Check if .env exists
if [ ! -f ".env" ]; then
    echo -e "${RED}Error: .env file not found. Please create one from .env.example${NC}"
    exit 1
fi

# Source environment variables
source .env

# Command line arguments
COMMAND="${1:-help}"
SERVICE="${2:-}"

case "$COMMAND" in
    start)
        header
        echo -e "${GREEN}Starting Zeta Reticula system...${NC}"
        ./scripts/start_salience_engine.sh
        ;;
        
    stop)
        header
        echo -e "${YELLOW}Stopping Zeta Reticula system...${NC}"
        ./scripts/stop_services.sh
        ;;
        
    restart)
        header
        echo -e "${YELLOW}Restarting Zeta Reticula system...${NC}"
        ./scripts/stop_services.sh
        sleep 2
        ./scripts/start_salience_engine.sh
        ;;
        
    status)
        header
        ./scripts/status.sh
        ;;
        
    logs)
        header
        if [ -z "$SERVICE" ]; then
            echo -e "${YELLOW}Showing logs for all services:${NC}\n"
            for log_file in logs/*.log; do
                if [ -f "$log_file" ]; then
                    service_name=$(basename "$log_file" .log)
                    echo -e "${BLUE}=== $service_name logs ===${NC}"
                    tail -n 20 "$log_file"
                    echo ""
                fi
            done
        else
            log_file="logs/$SERVICE.log"
            if [ -f "$log_file" ]; then
                echo -e "${BLUE}=== $SERVICE logs ===${NC}"
                tail -f "$log_file"
            else
                echo -e "${RED}Error: No log file found for service '$SERVICE'${NC}"
                exit 1
            fi
        fi
        ;;
        
    clean)
        header
        ./scripts/cleanup.sh
        ;;
        
    update)
        header
        echo -e "${GREEN}Updating Zeta Reticula system...${NC}\n"
        
        # Pull latest changes
        echo -e "${YELLOW}Pulling latest changes...${NC}"
        git pull
        
        # Rebuild all components
        echo -e "\n${YELLOW}Rebuilding components...${NC}"
        ./scripts/build.sh
        
        echo -e "\n${GREEN}Update complete!${NC}"
        ;;
        
    help|*)
        header
        usage
        ;;
esac

echo -e "\n${GREEN}Done!${NC}"
