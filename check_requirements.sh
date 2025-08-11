#!/bin/bash

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    echo "   Visit: https://docs.docker.com/get-docker/"
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose is not installed. Please install Docker Compose."
    echo "   Visit: https://docs.docker.com/compose/install/"
    exit 1
fi

# Check available memory (in MB)
MEMORY_GB=$(($(sysctl -n hw.memsize 2>/dev/null || echo 0) / 1024 / 1024 / 1024))
MIN_MEMORY_GB=32

if [ "$MEMORY_GB" -lt "$MIN_MEMORY_GB" ]; then
    echo "⚠️  Warning: Your system has ${MEMORY_GB}GB of RAM."
    echo "   For optimal performance, at least ${MIN_MEMORY_GB}GB of RAM is recommended."
    read -p "   Do you want to continue anyway? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check disk space (in GB)
DISK_SPACE_GB=$(($(df -k . | tail -n1 | awk '{print $4}') / 1024 / 1024))
MIN_DISK_SPACE_GB=100

if [ "$DISK_SPACE_GB" -lt "$MIN_DISK_SPACE_GB" ]; then
    echo "⚠️  Warning: Low disk space available (${DISK_SPACE_GB}GB)."
    echo "   At least ${MIN_DISK_SPACE_GB}GB of free space is recommended."
    read -p "   Do you want to continue anyway? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check CPU cores
CPU_CORES=$(sysctl -n hw.ncpu 2>/dev/null || echo 4)
MIN_CPU_CORES=4

if [ "$CPU_CORES" -lt "$MIN_CPU_CORES" ]; then
    echo "⚠️  Warning: Your system has only ${CPU_CORES} CPU cores."
    echo "   For better performance, at least ${MIN_CPU_CORES} CPU cores are recommended."
    read -p "   Do you want to continue anyway? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check NVIDIA GPU (optional but recommended)
if ! command -v nvidia-smi &> /dev/null; then
    echo "ℹ️  NVIDIA GPU not detected or NVIDIA drivers not installed."
    echo "   GPU acceleration will not be available. Running on CPU only."
    echo "   For better performance, install NVIDIA drivers and NVIDIA Container Toolkit."
    read -p "   Continue without GPU support? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
    export DOCKER_RUNTIME=""
else
    echo "✅ NVIDIA GPU detected."
    export DOCKER_RUNTIME="--gpus all"
fi

echo "✅ All requirements met. You can now start the system with:"
echo "   docker-compose -f docker-compose.full.yml up -d"

exit 0
