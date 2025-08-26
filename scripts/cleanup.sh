#!/bin/bash
set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to confirm before proceeding
confirm() {
    read -r -p "${1:-Are you sure?} [y/N] " response
    case "$response" in
        [yY][eE][sS]|[yY]) 
            true
            ;;
        *)
            false
            ;;
    esac
}

# Change to project root
cd "$(dirname "$0")/.."

echo -e "${YELLOW}=== Cleaning Up Zeta Reticula System ===${NC}\n"

# Stop all running services first
if [ -d ".pids" ]; then
    echo -e "${YELLOW}Stopping all running services...${NC}"
    ./scripts/stop_services.sh
    echo
fi

# Remove PID directory
if [ -d ".pids" ]; then
    echo -e "${YELLOW}Removing PID files...${NC}"
    rm -rf .pids
    echo -e "${GREEN}✓ Removed PID files${NC}"
fi

# Clean up build artifacts
echo -e "\n${YELLOW}Cleaning build artifacts...${NC}"

# Clean Rust build artifacts
if [ -d "target" ]; then
    echo -e "${YELLOW}Removing Rust target directories...${NC}"
    find . -name 'target' -type d -prune -exec rm -rf {} +
    echo -e "${GREEN}✓ Removed Rust target directories${NC}"
fi

# Clean Python cache files
echo -e "\n${YELLOW}Removing Python cache files...${NC}"
find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
find . -type f -name "*.py[co]" -delete
find . -type d -name "*.egg-info" -exec rm -rf {} + 2>/dev/null || true
find . -type d -name ".pytest_cache" -exec rm -rf {} + 2>/dev/null || true
echo -e "${GREEN}✓ Removed Python cache files${NC}"

# Clean log files
if [ -d "logs" ]; then
    if confirm "Do you want to remove all log files? (y/N)"; then
        echo -e "\n${YELLOW}Removing log files...${NC}"
        rm -rf logs/*
        echo -e "${GREEN}✓ Removed log files${NC}"
    fi
fi

# Clean data directories
if [ -d "data" ]; then
    if confirm "Do you want to remove all data files? This will delete all stored models and caches. (y/N)"; then
        echo -e "\n${YELLOW}Removing data files...${NC}"
        rm -rf data/*
        echo -e "${GREEN}✓ Removed data files${NC}"
    fi
fi

# Clean temporary files
echo -e "\n${YELLOW}Removing temporary files...${NC}"
find . -type f -name "*.log" -delete
find . -type f -name "*.tmp" -delete
find . -type f -name "*.swp" -delete
find . -type f -name "*.swo" -delete
find . -type f -name "*.bak" -delete
echo -e "${GREEN}✓ Removed temporary files${NC}"

# Clean Docker resources
if command -v docker >/dev/null 2>&1; then
    if confirm "Do you want to clean up Docker resources? (stop and remove containers, networks, volumes) (y/N)"; then
        echo -e "\n${YELLOW}Cleaning up Docker resources...${NC}"
        
        # Stop and remove all containers
        if [ "$(docker ps -aq)" ]; then
            echo -e "${YELLOW}Stopping and removing containers...${NC}"
            docker stop $(docker ps -aq) 2>/dev/null || true
            docker rm $(docker ps -aq) 2>/dev/null || true
        fi
        
        # Remove unused networks
        echo -e "${YELLOW}Removing unused networks...${NC}"
        docker network prune -f 2>/dev/null || true
        
        # Remove unused volumes
        echo -e "${YELLOW}Removing unused volumes...${NC}"
        docker volume prune -f 2>/dev/null || true
        
        # Remove unused images
        if confirm "Do you want to remove unused Docker images? (y/N)"; then
            echo -e "${YELLOW}Removing unused images...${NC}"
            docker image prune -f 2>/dev/null || true
        fi
        
        echo -e "${GREEN}✓ Docker resources cleaned up${NC}"
    fi
fi

echo -e "\n${GREEN}=== Cleanup Complete ===${NC}"
