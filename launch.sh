#!/bin/bash

# Zeta Reticula Launch Script
# This script helps manage the Zeta Reticula system

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to display help
show_help() {
    echo -e "${YELLOW}Zeta Reticula Management Script${NC}"
    echo "Usage: $0 [command] [options]"
    echo ""
    echo "Commands:"
    echo "  start       Start all services"
    echo "  stop        Stop all services"
    echo "  restart     Restart all services"
    echo "  status      Show status of services"
    echo "  logs        View logs of services"
    echo "  update      Update the system"
    echo "  help        Show this help message"
    echo ""
    echo "Options:"
    echo "  -b, --build     Rebuild containers before starting"
    echo "  -f, --force     Force operation without confirmation"
    echo "  -s, --service   Specify a specific service to manage"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check requirements
check_requirements() {
    echo -e "${YELLOW}Checking system requirements...${NC}"
    
    # Check Docker
    if ! command_exists docker; then
        echo -e "${RED}❌ Docker is not installed. Please install Docker first.${NC}"
        exit 1
    fi
    
    # Check Docker Compose
    if ! command_exists docker-compose; then
        echo -e "${RED}❌ Docker Compose is not installed. Please install Docker Compose.${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ All requirements are met.${NC}"
}

# Function to start services
start_services() {
    echo -e "${YELLOW}Starting Zeta Reticula services...${NC}"
    
    # Check if already running
    if [ "$(docker-compose -f docker-compose.full.yml ps -q | wc -l)" -gt 0 ]; then
        echo -e "${YELLOW}Services are already running. Use 'restart' to restart them.${NC}"
        exit 1
    fi
    
    # Build if requested
    if [ "$BUILD" = true ]; then
        echo -e "${YELLOW}Building containers...${NC}"
        docker-compose -f docker-compose.full.yml build
    fi
    
    # Start services
    docker-compose -f docker-compose.full.yml up -d
    
    echo -e "${GREEN}✅ Zeta Reticula services started successfully!${NC}"
    echo -e "\n${YELLOW}Access the following services:${NC}"
    echo -e "- API Gateway: ${GREEN}http://localhost:3000${NC}"
    echo -e "- Prometheus:  ${GREEN}http://localhost:9090${NC}"
    echo -e "- Grafana:     ${GREEN}http://localhost:3001${NC} (admin/admin)"
    echo -e "- Salience:    ${GREEN}http://localhost:8080${NC}"
}

# Function to stop services
stop_services() {
    if [ "$FORCE" != true ]; then
        read -p "Are you sure you want to stop all services? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 0
        fi
    fi
    
    echo -e "${YELLOW}Stopping Zeta Reticula services...${NC}"
    docker-compose -f docker-compose.full.yml down
    echo -e "${GREEN}✅ Services stopped.${NC}"
}

# Function to restart services
restart_services() {
    echo -e "${YELLOW}Restarting Zeta Reticula services...${NC}"
    docker-compose -f docker-compose.full.yml restart
    echo -e "${GREEN}✅ Services restarted.${NC}"
}

# Function to show service status
show_status() {
    echo -e "${YELLOW}Zeta Reticula Service Status:${NC}"
    docker-compose -f docker-compose.full.yml ps
}

# Function to show logs
show_logs() {
    if [ -z "$SERVICE" ]; then
        docker-compose -f docker-compose.full.yml logs -f
    else
        docker-compose -f docker-compose.full.yml logs -f "$SERVICE"
    fi
}

# Function to update the system
update_system() {
    echo -e "${YELLOW}Updating Zeta Reticula system...${NC}"
    
    # Pull latest changes
    git pull
    
    # Rebuild and restart
    docker-compose -f docker-compose.full.yml down
    docker-compose -f docker-compose.full.yml pull
    docker-compose -f docker-compose.full.yml build --no-cache
    docker-compose -f docker-compose.full.yml up -d
    
    echo -e "${GREEN}✅ System updated successfully!${NC}"
}

# Parse command line arguments
COMMAND=""
BUILD=false
FORCE=false
SERVICE=""

while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        start|stop|restart|status|logs|update|help)
            COMMAND="$1"
            shift
            ;;
        -b|--build)
            BUILD=true
            shift
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        -s|--service)
            SERVICE="$2"
            shift 2
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# Default to help if no command provided
if [ -z "$COMMAND" ]; then
    show_help
    exit 0
fi

# Execute the requested command
case "$COMMAND" in
    start)
        check_requirements
        start_services
        ;;
    stop)
        stop_services
        ;;
    restart)
        restart_services
        ;;
    status)
        show_status
        ;;
    logs)
        show_logs
        ;;
    update)
        update_system
        ;;
    help)
        show_help
        ;;
    *)
        echo -e "${RED}Unknown command: $COMMAND${NC}"
        show_help
        exit 1
        ;;
esac

exit 0
