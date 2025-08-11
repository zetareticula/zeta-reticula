#!/bin/bash

# Simple Zeta Reticula Setup Check
# This script performs basic checks for the Zeta Reticula setup

echo "🔍 Checking Zeta Reticula setup..."

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed"
    exit 1
else
    echo "✅ Docker is installed"
    
    if ! docker info &> /dev/null; then
        echo "❌ Docker daemon is not running"
        exit 1
    else
        echo "✅ Docker daemon is running"
    fi
fi

# Check Docker Compose
if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose is not installed"
    exit 1
else
    echo "✅ Docker Compose is installed"
fi

# Check required directories
for dir in models attention-data vault-data quantization-results monitoring; do
    if [ -d "$dir" ]; then
        echo "✅ Directory '$dir' exists"
    else
        echo "⚠️  Directory '$dir' is missing"
        mkdir -p "$dir"
        echo "   Created directory '$dir'"
    fi
done

# Check required files
for file in "docker-compose.full.yml" "Dockerfile.zeta-vault" "monitoring/prometheus.yml"; do
    if [ -f "$file" ]; then
        echo "✅ File '$file' exists"
    else
        echo "❌ Required file '$file' is missing"
        exit 1
    fi
done

echo "\n📊 System Resources:"

# CPU cores
if [ "$(uname)" = "Darwin" ]; then
    CPU_CORES=$(sysctl -n hw.ncpu)
else
    CPU_CORES=$(nproc)
fi
echo "CPU Cores: $CPU_CORES"

# Memory
if [ "$(uname)" = "Darwin" ]; then
    TOTAL_MEM_GB=$(($(sysctl -n hw.memsize) / 1024 / 1024 / 1024))
else
    TOTAL_MEM_GB=$(($(grep MemTotal /proc/meminfo | awk '{print $2}') / 1024 / 1024))
fi
echo "Total Memory: ${TOTAL_MEM_GB}GB"

# Disk space
echo "Available Disk Space: $(df -h . | tail -1 | awk '{print $4}')"

# Check GPU
if command -v nvidia-smi &> /dev/null; then
    echo "\n✅ NVIDIA GPU detected:"
    nvidia-smi --query-gpu=name,memory.total --format=csv
else
    echo "\n⚠️  No NVIDIA GPU detected. System will run on CPU only."
fi

echo "\n✅ Basic setup check completed successfully!"
