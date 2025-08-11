#!/bin/bash

# Zeta Reticula Setup Verification
# This script verifies that all required components are properly installed

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

echo -e "${YELLOW}🔍 Verifying Zeta Reticula setup...${NC}"

# Check Docker
if command_exists docker; then
    echo -e "${GREEN}✅ Docker is installed${NC}"
    
    # Check Docker daemon
    if docker info >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Docker daemon is running${NC}
    else
        echo -e "${RED}❌ Docker daemon is not running${NC}"
        echo -e "   Please start Docker and try again"
        exit 1
    fi
else
    echo -e "${RED}❌ Docker is not installed${NC}"
    echo -e "   Please install Docker from: https://docs.docker.com/get-docker/"
    exit 1
fi

# Check Docker Compose
if command_exists docker-compose; then
    echo -e "${GREEN}✅ Docker Compose is installed${NC}
else
    echo -e "${RED}❌ Docker Compose is not installed${NC}"
    echo -e "   Please install Docker Compose from: https://docs.docker.com/compose/install/"
    exit 1
fi

# Check required directories
REQUIRED_DIRS=("models" "attention-data" "vault-data" "quantization-results" "monitoring")
for dir in "${REQUIRED_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        echo -e "${GREEN}✅ Directory '$dir' exists${NC}"
    else
        echo -e "${YELLOW}⚠️  Directory '$dir' is missing, creating...${NC}"
        mkdir -p "$dir"
        chmod 777 "$dir"
    fi
done

# Check required files
REQUIRED_FILES=("docker-compose.full.yml" "Dockerfile.zeta-vault" "monitoring/prometheus.yml")
for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo -e "${GREEN}✅ File '$file' exists${NC}"
    else
        echo -e "${RED}❌ Required file '$file' is missing${NC}"
        exit 1
    fi
done

# Check ports
PORTS=("3000" "8080" "9090" "9091" "9092" "9093" "9094" "9095")
for port in "${PORTS[@]}"; do
    if lsof -i ":$port" > /dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  Port $port is already in use${NC}"
    else
        echo -e "${GREEN}✅ Port $port is available${NC}"
    fi
done

# Check model files
MODEL_DIR="models/llama2-7b"
REQUIRED_MODEL_FILES=("config.json" "tokenizer.json" "model.safetensors")

if [ -d "$MODEL_DIR" ]; then
    echo -e "${GREEN}✅ Model directory '$MODEL_DIR' exists${NC}"
    
    all_model_files_exist=true
    for file in "${REQUIRED_MODEL_FILES[@]}"; do
        if [ -f "$MODEL_DIR/$file" ]; then
            echo -e "${GREEN}✅ Model file '$file' exists${NC}"
        else
            echo -e "${YELLOW}⚠️  Model file '$file' is missing${NC}"
            all_model_files_exist=false
        fi
    done
    
    if [ "$all_model_files_exist" = true ]; then
        echo -e "${GREEN}✅ All required model files are present${NC}"
    else
        echo -e "${YELLOW}⚠️  Some model files are missing. You can download them using:${NC}"
        echo -e "   ${YELLOW}./download_model.sh${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Model directory '$MODEL_DIR' not found${NC}"
    echo -e "   Please download the model files using: ${YELLOW}./download_model.sh${NC}"
fi

# Check system resources
echo -e "\n${YELLOW}📊 System Resources:${NC}"

# CPU cores
if [ "$(uname)" = "Darwin" ]; then
    CPU_CORES=$(sysctl -n hw.ncpu)
else
    CPU_CORES=$(nproc)
fi
echo -e "CPU Cores: ${GREEN}${CPU_CORES}${NC}"

# Memory
if [ "$(uname)" = "Darwin" ]; then
    TOTAL_MEM_GB=$(($(sysctl -n hw.memsize) / 1024 / 1024 / 1024))
else
    TOTAL_MEM_GB=$(($(grep MemTotal /proc/meminfo | awk '{print $2}') / 1024 / 1024))
fi
echo -e "Total Memory: ${GREEN}${TOTAL_MEM_GB}GB${NC}"

# Disk space
DISK_SPACE=$(df -h . | tail -1 | awk '{print $4}')
echo -e "Available Disk Space: ${GREEN}${DISK_SPACE}${NC}"

# Check GPU
if command_exists nvidia-smi; then
    echo -e "\n${GREEN}✅ NVIDIA GPU detected:${NC}"
    nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv
else
    echo -e "\n${YELLOW}⚠️  No NVIDIA GPU detected. System will run on CPU only.${NC}"
    echo -e "   For better performance, consider using a system with an NVIDIA GPU"
fi

# Final check
echo -e "\n${GREEN}========================================${NC}"
if [ "$all_model_files_exist" = true ]; then
    echo -e "${GREEN}✅ Zeta Reticula is ready to start!${NC}"
    echo -e "\nTo start the system, run: ${YELLOW}./launch.sh start${NC}"
else
    echo -e "${YELLOW}⚠️  Zeta Reticula setup is almost complete${NC}"
    echo -e "   Please download the required model files first: ${YELLOW}./download_model.sh${NC}"
    echo -e "   Then start the system with: ${YELLOW}./launch.sh start${NC}"
fi
echo -e "${GREEN}========================================${NC}"
