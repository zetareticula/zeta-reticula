#!/bin/bash
set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
MODEL_NAME="llama-2-7b"
MODEL_DIR="./models"
QUANTIZE=false
BITS=4
GROUP_SIZE=64
ACT_ORDER=true

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --model)
      MODEL_NAME="$2"
      shift 2
      ;;
    --dir)
      MODEL_DIR="$2"
      shift 2
      ;;
    --quantize)
      QUANTIZE=true
      shift
      ;;
    --bits)
      BITS="$2"
      shift 2
      ;;
    --group-size)
      GROUP_SIZE="$2"
      shift 2
      ;;
    --act-order)
      ACT_ORDER=true
      shift
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Create model directory if it doesn't exist
mkdir -p "$MODEL_DIR"

# Function to download model
download_model() {
  local model_name=$1
  local target_dir="$MODEL_DIR/$model_name"
  
  echo -e "${GREEN}Downloading model: $model_name${NC}"
  
  # Check if model already exists
  if [ -d "$target_dir" ]; then
    echo -e "${YELLOW}Model $model_name already exists at $target_dir${NC}"
    return 0
  fi
  
  # Create a temporary directory
  local temp_dir=$(mktemp -d)
  
  # Download the model (replace with actual download command)
  # This is a placeholder - you'll need to implement the actual download logic
  # based on your model hosting solution
  echo -e "${YELLOW}Downloading model files...${NC}"
  # Example: huggingface-cli download $model_name --local-dir "$temp_dir" --local-dir-use-symlinks False
  
  # Move to target directory
  mv "$temp_dir" "$target_dir"
  
  echo -e "${GREEN}Model downloaded to $target_dir${NC}" 
}

# Function to quantize model
quantize_model() {
  local model_name=$1
  local bits=$2
  local group_size=$3
  local act_order=$4
  
  local source_dir="$MODEL_DIR/$model_name"
  local target_dir="$MODEL_DIR/${model_name}-q${bits}-g${group_size}"
  
  if [ "$act_order" = true ]; then
    target_dir="${target_dir}-act"
  fi
  
  # Check if quantized model already exists
  if [ -d "$target_dir" ]; then
    echo -e "${YELLOW}Quantized model already exists at $target_dir${NC}"
    return 0
  fi
  
  echo -e "${GREEN}Quantizing model to ${bits} bits with group size ${group_size}...${NC}"
  
  # Build quantize-cli if not already built
  if [ ! -f "target/release/quantize-cli" ]; then
    echo -e "${YELLOW}Building quantize-cli...${NC}"
    cd quantize-cli
    cargo build --release
    cd ..
  fi
  
  # Run quantization
  ./quantize-cli/target/release/quantize-cli \
    --model-path "$source_dir" \
    --output-path "$target_dir" \
    --bits "$bits" \
    --group-size "$group_size" \
    $([ "$act_order" = true ] && echo "--act-order")
  
  echo -e "${GREEN}Quantized model saved to $target_dir${NC}"
}

# Main script
main() {
  # Download the model
  download_model "$MODEL_NAME"
  
  # Quantize if requested
  if [ "$QUANTIZE" = true ]; then
    quantize_model "$MODEL_NAME" "$BITS" "$GROUP_SIZE" "$ACT_ORDER"
    
    # Update .env with the quantized model path
    QUANTIZED_MODEL_PATH="$MODEL_DIR/${MODEL_NAME}-q${BITS}-g${GROUP_SIZE}$([ "$ACT_ORDER" = true ] && echo "-act")"
    sed -i '' "s|^QUANTIZED_MODEL_PATH=.*|QUANTIZED_MODEL_PATH=\"$QUANTIZED_MODEL_PATH\"|" .env
    echo -e "${GREEN}Updated .env with QUANTIZED_MODEL_PATH=$QUANTIZED_MODEL_PATH${NC}"
  fi
}

# Run the main function
main "$@"
