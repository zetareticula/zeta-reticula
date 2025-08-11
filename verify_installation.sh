#!/bin/bash

# Zeta Reticula Installation Verification Script
# This script verifies that all components are properly installed and configured

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check if a Docker container is running
container_running() {
    docker ps --format '{{.Names}}' | grep -q "$1"
}

# Function to check if a port is in use
port_in_use() {
    lsof -i ":$1" > /dev/null 2>&1
}

echo -e "${YELLOW}🔍 Verifying Zeta Reticula installation...${NC}"

# Check Docker
if ! command_exists docker; then
    echo -e "${RED}❌ Docker is not installed. Please install Docker first.${NC}"
    exit 1
else
    echo -e "${GREEN}✅ Docker is installed.${NC}"
fi

# Check Docker Compose
if ! command_exists docker-compose; then
    echo -e "${RED}❌ Docker Compose is not installed. Please install Docker Compose.${NC}"
    exit 1
else
    echo -e "${GREEN}✅ Docker Compose is installed.${NC}"
fi

# Check Docker daemon
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}❌ Docker daemon is not running. Please start Docker.${NC}"
    exit 1
else
    echo -e "${GREEN}✅ Docker daemon is running.${NC}"
fi

# Check required directories
REQUIRED_DIRS=("models" "attention-data" "vault-data" "quantization-results" "monitoring")
for dir in "${REQUIRED_DIRS[@]}"; do
    if [ ! -d "$dir" ]; then
        echo -e "${YELLOW}⚠️  Directory '$dir' is missing. Creating it now...${NC}"
        mkdir -p "$dir"
    else
        echo -e "${GREEN}✅ Directory '$dir' exists.${NC}"
    fi
    
    # Check directory permissions
    if [ ! -w "$dir" ]; then
        echo -e "${YELLOW}⚠️  No write permissions for directory '$dir'. Fixing permissions...${NC}"
        chmod -R 777 "$dir"
    fi
done

# Check required files
REQUIRED_FILES=("docker-compose.full.yml" "Dockerfile.zeta-vault" "monitoring/prometheus.yml")
for file in "${REQUIRED_FILES[@]}"; do
    if [ ! -f "$file" ]; then
        echo -e "${RED}❌ Required file '$file' is missing.${NC}"
        exit 1
    else
        echo -e "${GREEN}✅ File '$file' exists.${NC}"
    fi
done

# Check if services are already running
SERVICES=("salience-engine" "ns-router" "agentflow" "llm-service" "kv-quant" "attention-store" "zeta-vault" "api-gateway")
RUNNING_SERVICES=0

for service in "${SERVICES[@]}"; do
    if container_running "$service"; then
        echo -e "${YELLOW}⚠️  Service '$service' is already running.${NC}"
        RUNNING_SERVICES=$((RUNNING_SERVICES + 1))
    fi
done

if [ "$RUNNING_SERVICES" -gt 0 ]; then
    echo -e "${YELLOW}ℹ️  Some services are already running. You may want to stop them first.${NC}"
    read -p "Do you want to stop all running services and continue? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        ./launch.sh stop
    fi
fi

# Check required ports
PORTS=("3000" "8080" "9090" "9091" "9092" "9093" "9094" "9095")
for port in "${PORTS[@]}"; do
    if port_in_use "$port"; then
        echo -e "${YELLOW}⚠️  Port $port is already in use. This may cause issues.${NC}"
    else
        echo -e "${GREEN}✅ Port $port is available.${NC}"
    fi
done

# Check for model files
if [ ! -d "models/llama2-7b" ]; then
    echo -e "${YELLOW}⚠️  Model directory 'models/llama2-7b' not found.${NC}"
    echo -e "${YELLOW}   Please place your model files in the 'models/llama2-7b' directory.${NC}"
    echo -e "${YELLOW}   Required files: config.json, tokenizer.json, model.safetensors${NC}"
    echo -e "${YELLOW}   You can download the model using the download_model.sh script.${NC}"
    ALL_FILES_EXIST=false
else
    echo -e "${GREEN}✅ Model directory 'models/llama2-7b' exists.${NC}
    
    # Check for required model files
    REQUIRED_MODEL_FILES=("config.json" "tokenizer.json" "model.safetensors")
    ALL_FILES_EXIST=true
    
    for file in "${REQUIRED_MODEL_FILES[@]}"; do
        if [ ! -f "models/llama2-7b/$file" ]; then
            echo -e "${RED}❌ Required model file '$file' is missing.${NC}"
            ALL_FILES_EXIST=false
        fi
    done
    
    if [ "$ALL_FILES_EXIST" = true ]; then
        echo -e "${GREEN}✅ All required model files are present.${NC}"
    else
        echo -e "${RED}❌ Some required model files are missing. Please check the model directory.${NC}"
        echo -e "${YELLOW}   You can download the model using: ${NC}./download_model.sh"
        ALL_FILES_EXIST=false
    fi
fi

# Check Docker Compose configuration
echo -e "\n${YELLOW}🔧 Verifying Docker Compose configuration...${NC}"
if ! docker-compose -f docker-compose.full.yml config -q; then
    echo -e "${RED}❌ Docker Compose configuration is invalid.${NC}"
    exit 1
else
    echo -e "${GREEN}✅ Docker Compose configuration is valid.${NC}"
fi

# Check available resources
echo -e "\n${YELLOW}📊 Checking system resources...${NC}"

# Check CPU cores
CPU_CORES=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo "unknown")
echo -e "CPU Cores: ${GREEN}${CPU_CORES}${NC}"

# Check memory (in GB)
if [ "$(uname)" = "Darwin" ]; then
    # macOS
    TOTAL_MEM=$(($(sysctl -n hw.memsize) / 1024 / 1024 / 1024))
else
    # Linux
    TOTAL_MEM=$(($(grep MemTotal /proc/meminfo | awk '{print $2}') / 1024 / 1024))
fi
echo -e "Total Memory: ${GREEN}${TOTAL_MEM}GB${NC}"

# Check disk space (in GB)
DISK_SPACE=$(df -h . | tail -1 | awk '{print $4}')
echo -e "Available Disk Space: ${GREEN}${DISK_SPACE}${NC}"

# Check GPU
if command_exists nvidia-smi; then
    echo -e "${GREEN}✅ NVIDIA GPU detected.${NC}"
    nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv
else
    echo -e "${YELLOW}⚠️  No NVIDIA GPU detected. Running on CPU only.${NC}"
fi

# Final check
echo -e "\n${GREEN}========================================${NC}"
if [ "$ALL_FILES_EXIST" = true ]; then
    echo -e "${GREEN}✅ Zeta Reticula is ready to start!${NC}"
    echo -e "\nTo start the system, run: ${YELLOW}./launch.sh start${NC}"
else
    echo -e "${YELLOW}⚠️  Zeta Reticula is almost ready, but some components are missing.${NC}"
    echo -e "   Please check the warnings above and then run: ${YELLOW}./launch.sh start${NC}"
fi
echo -e "${GREEN}========================================${NC}"

exit 0
