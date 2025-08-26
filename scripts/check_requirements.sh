#!/bin/bash
set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to check if a command exists
command_exists() {
    command -v "$1" &> /dev/null
}

# Function to check version requirements
check_version() {
    local cmd=$1
    local min_version=$2
    local version_cmd=$3
    local version_pattern=$4
    
    if ! command_exists "$cmd"; then
        echo -e "${RED}✗ $cmd is not installed${NC}"
        return 1
    fi
    
    local version=$($version_cmd | grep -oP "$version_pattern" | head -1)
    if [ "$(printf '%s\n' "$min_version" "$version" | sort -V | head -n1)" = "$min_version" ]; then
        echo -e "${GREEN}✓ $cmd $version (>= $min_version)${NC}"
        return 0
    else
        echo -e "${RED}✗ $cmd $version is installed but version >= $min_version is required${NC}"
        return 1
    fi
}

# Check system requirements
echo -e "${YELLOW}=== Checking System Requirements ===${NC}"

# Check Rust
if command_exists "rustc"; then
    rust_version=$(rustc --version | grep -oP '\d+\.\d+\.\d+')
    required_rust="1.70.0"
    if [ "$(printf '%s\n' "$required_rust" "$rust_version" | sort -V | head -n1)" = "$required_rust" ]; then
        echo -e "${GREEN}✓ Rust $rust_version (>= $required_rust)${NC}"
    else
        echo -e "${RED}✗ Rust $rust_version is installed but version >= $required_rust is required${NC}"
    fi
else
    echo -e "${RED}✗ Rust is not installed${NC}"
fi

# Check Cargo
if command_exists "cargo"; then
    cargo_version=$(cargo --version | grep -oP '\d+\.\d+\.\d+')
    echo -e "${GREEN}✓ Cargo $cargo_version${NC}"
else
    echo -e "${RED}✗ Cargo is not installed${NC}"
fi

# Check Docker
if command_exists "docker"; then
    docker_version=$(docker --version | grep -oP '\d+\.\d+\.\d+')
    required_docker="20.10.0"
    if [ "$(printf '%s\n' "$required_docker" "$docker_version" | sort -V | head -n1)" = "$required_docker" ]; then
        echo -e "${GREEN}✓ Docker $docker_version (>= $required_docker)${NC}"
    else
        echo -e "${YELLOW}⚠ Docker $docker_version is installed but version >= $required_docker is recommended${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Docker is not installed (required for some components)${NC}"
fi

# Check Docker Compose
if command_exists "docker-compose"; then
    docker_compose_version=$(docker-compose --version | grep -oP '\d+\.\d+\.\d+')
    required_docker_compose="1.29.0"
    if [ "$(printf '%s\n' "$required_docker_compose" "$docker_compose_version" | sort -V | head -n1)" = "$required_docker_compose" ]; then
        echo -e "${GREEN}✓ Docker Compose $docker_compose_version (>= $required_docker_compose)${NC}"
    else
        echo -e "${YELLOW}⚠ Docker Compose $docker_compose_version is installed but version >= $required_docker_compose is recommended${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Docker Compose is not installed (required for some components)${NC}"
fi

# Check Python
if command_exists "python3"; then
    python_version=$(python3 --version | grep -oP '\d+\.\d+\.\d+')
    required_python="3.8.0"
    if [ "$(printf '%s\n' "$required_python" "$python_version" | sort -V | head -n1)" = "$required_python" ]; then
        echo -e "${GREEN}✓ Python $python_version (>= $required_python)${NC}"
    else
        echo -e "${YELLOW}⚠ Python $python_version is installed but version >= $required_python is recommended${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Python 3 is not installed (required for some utilities)${NC}"
fi

# Check CUDA (if available)
if command_exists "nvidia-smi"; then
    cuda_version=$(nvcc --version | grep -oP 'release \K\d+\.\d+' || echo "Not found")
    if [ "$cuda_version" != "Not found" ]; then
        echo -e "${GREEN}✓ CUDA $cuda_version${NC}"
        
        # Check GPU memory
        gpu_memory=$(nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits | head -n1)
        if [ "$gpu_memory" -lt 8000 ]; then
            echo -e "${YELLOW}⚠ Low GPU memory: ${gpu_memory}MB (8GB or more recommended)${NC}"
        else
            echo -e "${GREEN}✓ GPU memory: $((gpu_memory / 1024))GB${NC}"
        fi
    else
        echo -e "${YELLOW}⚠ CUDA toolkit is not properly installed${NC}"
    fi
else
    echo -e "${YELLOW}⚠ NVIDIA drivers not found. GPU acceleration will not be available.${NC}"
fi

# Check system resources
echo -e "\n${YELLOW}=== System Resources ===${NC}"

# Check CPU cores
cpu_cores=$(sysctl -n hw.ncpu 2>/dev/null || echo "4")
if [ "$cpu_cores" -lt 4 ]; then
    echo -e "${YELLOW}⚠ Low number of CPU cores: $cpu_cores (4 or more recommended)${NC}"
else
    echo -e "${GREEN}✓ CPU cores: $cpu_cores${NC}"
fi

# Check RAM
if [ "$(uname -s)" = "Darwin" ]; then
    ram_mb=$(($(sysctl -n hw.memsize) / 1024 / 1024))
else
    ram_mb=$(($(grep MemTotal /proc/meminfo | awk '{print $2}') / 1024))
fi

if [ "$ram_mb" -lt 16000 ]; then
    echo -e "${YELLOW}⚠ Low system memory: $((ram_mb / 1024))GB (16GB or more recommended)${NC}"
else
    echo -e "${GREEN}✓ System memory: $((ram_mb / 1024))GB${NC}"
fi

# Check disk space
disk_space=$(df -h . | awk 'NR==2 {print $4}')
echo -e "${GREEN}✓ Available disk space: $disk_space${NC}"

# Check environment variables
echo -e "\n${YELLOW}=== Environment Variables ===${NC}"

# Check for CUDA_HOME
if [ -z "${CUDA_HOME:-}" ]; then
    echo -e "${YELLOW}⚠ CUDA_HOME is not set${NC}"
else
    echo -e "${GREEN}✓ CUDA_HOME is set to $CUDA_HOME${NC}"
fi

# Check for LD_LIBRARY_PATH
if [ -z "${LD_LIBRARY_PATH:-}" ]; then
    echo -e "${YELLOW}⚠ LD_LIBRARY_PATH is not set${NC}"
else
    echo -e "${GREEN}✓ LD_LIBRARY_PATH is set${NC}"
fi

# Check for required environment variables from .env
if [ -f ".env" ]; then
    echo -e "\n${YELLOW}=== Required Environment Variables ===${NC}"
    
    required_vars=(
        "MODEL_PATH"
        "QUANTIZED_MODEL_PATH"
        "DISTRIBUTED_STORE_HOST"
        "DISTRIBUTED_STORE_PORT"
        "MASTER_SERVICE_HOST"
        "MASTER_SERVICE_PORT"
        "AGENTFLOW_HOST"
        "AGENTFLOW_PORT"
        "SALIENCE_ENGINE_HOST"
        "SALIENCE_ENGINE_PORT"
    )
    
    for var in "${required_vars[@]}"; do
        if grep -q "^$var=" .env; then
            echo -e "${GREEN}✓ $var is set${NC}"
        else
            echo -e "${RED}✗ $var is not set in .env${NC}"
        fi
    done
else
    echo -e "${RED}✗ .env file not found. Please create one from .env.example${NC}"
fi

echo -e "\n${YELLOW}=== Summary ===${NC}"

# Check if all requirements are met
if [ -f ".env" ] && \
   command_exists "rustc" && \
   command_exists "cargo" && \
   command_exists "docker" && \
   command_exists "docker-compose"; then
    echo -e "${GREEN}✓ All requirements are met! You can proceed with the setup.${NC}"
else
    echo -e "${YELLOW}⚠ Some requirements are missing or need attention. Please address the issues above before proceeding.${NC}"
fi

echo -e "\n${YELLOW}Next steps:${NC}"
echo "1. Install any missing dependencies"
echo "2. Configure your .env file"
echo "3. Run './scripts/download_model.sh' to download the required models"
echo "4. Run './run.sh' to start the system"
