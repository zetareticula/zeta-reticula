#!/bin/bash

# Zeta Reticula Model Downloader
# This script helps download the required model files

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Model configuration
MODEL_NAME="llama2-7b"
MODEL_DIR="models/${MODEL_NAME}"
REQUIRED_FILES=("config.json" "tokenizer.json" "model.safetensors")

# Create model directory if it doesn't exist
mkdir -p "${MODEL_DIR}"

# Function to check if a file exists
file_exists() {
    [ -f "$1" ]
}

# Function to download a file with progress
download_file() {
    local url="$1"
    local output="$2"
    
    echo -e "${YELLOW}Downloading ${url}...${NC}"
    
    # Use curl if available, otherwise use wget
    if command -v curl >/dev/null 2>&1; then
        curl -L -o "${output}" "${url}" --progress-bar
    elif command -v wget >/dev/null 2>&1; then
        wget -O "${output}" "${url}"
    else
        echo -e "${RED}❌ Neither curl nor wget found. Please install one of them.${NC}"
        exit 1
    fi
}

# Check if all required files already exist
all_files_exist=true
for file in "${REQUIRED_FILES[@]}"; do
    if ! file_exists "${MODEL_DIR}/${file}"; then
        all_files_exist=false
        break
    fi
done

if [ "$all_files_exist" = true ]; then
    echo -e "${GREEN}✅ All model files already exist in ${MODEL_DIR}${NC}"
    exit 0
fi

echo -e "${YELLOW}⚠️  Model files not found in ${MODEL_DIR}${NC}"
echo -e "${YELLOW}   This script will help you download the required model files.${NC}"
echo -e "${YELLOW}   Note: The model files are large (several GB).${NC}"

# Check available disk space (in GB)
AVAILABLE_SPACE=$(($(df -k . | tail -1 | awk '{print $4}') / 1024 / 1024))
MIN_SPACE_REQUIRED=15  # GB

if [ "$AVAILABLE_SPACE" -lt "$MIN_SPACE_REQUIRED" ]; then
    echo -e "${RED}❌ Not enough disk space available.${NC}"
    echo -e "   Required: ${MIN_SPACE_REQUIRED}GB, Available: ${AVAILABLE_SPACE}GB"
    echo -e "   Please free up some space and try again."
    exit 1
fi

# Ask for confirmation
read -p "Do you want to download the model files? [y/N] " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}Download canceled.${NC}"
    exit 0
fi

# Download model files
BASE_URL="https://huggingface.co/meta-llama/Llama-2-7b/resolve/main"

echo -e "${YELLOW}Downloading model files to ${MODEL_DIR}...${NC}"

# Download each required file
for file in "${REQUIRED_FILES[@]}"; do
    if ! file_exists "${MODEL_DIR}/${file}"; then
        download_file "${BASE_URL}/${file}" "${MODEL_DIR}/${file}"
        
        # Verify download
        if [ $? -ne 0 ]; then
            echo -e "${RED}❌ Failed to download ${file}${NC}"
            exit 1
        fi
        
        echo -e "${GREEN}✅ Downloaded ${file}${NC}"
    else
        echo -e "${GREEN}✅ ${file} already exists, skipping...${NC}"
    fi
done

# Verify all files were downloaded
all_files_exist=true
for file in "${REQUIRED_FILES[@]}"; do
    if ! file_exists "${MODEL_DIR}/${file}"; then
        echo -e "${RED}❌ Missing file: ${file}${NC}"
        all_files_exist=false
    fi
done

if [ "$all_files_exist" = true ]; then
    echo -e "\n${GREEN}✅ Successfully downloaded all model files to ${MODEL_DIR}${NC}"
    echo -e "\nYou can now start the Zeta Reticula system with: ${YELLOW}./launch.sh start${NC}"
else
    echo -e "\n${RED}❌ Some files failed to download. Please check your internet connection and try again.${NC}"
    exit 1
fi

exit 0
