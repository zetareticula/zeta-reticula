#!/bin/bash

# Quantize models using p2pstore and kvquant_rs

set -e

# Configuration
MODELS_DIR="./models"
QUANTIZED_DIR="./quantized_models"
PRECISION="int8"  # Options: int4, int8, fp16, etc.

# Create output directory if it doesn't exist
mkdir -p "$QUANTIZED_DIR"

# Function to quantize a single model
quantize_model() {
    local model_path="$1"
    local model_name="$(basename "$model_path")"
    local output_path="${QUANTIZED_DIR}/${model_name%.*}_quantized.${model_name##*.}"
    
    echo "Quantizing $model_name..."
    
    # Use kvquant_rs to quantize the model
    cargo run --release --bin kvquant-cli -- quantize \
        --input "$model_path" \
        --output "$output_path" \
        --precision "$PRECISION"
    
    echo "Quantized model saved to $output_path"
    
    # Store the quantized model in p2pstore
    echo "Storing in p2pstore..."
    cargo run --bin p2pstore -- store "$output_path"
}

export -f quantize_model

# Process all models in the models directory
find "$MODELS_DIR" -type f \( -name "*.pt" -o -name "*.pth" -o -name "*.onnx" -o -name "*.safetensors" \) | \
    xargs -I {} -P 4 bash -c 'quantize_model "$@"' _ {}

echo "All models have been quantized and stored in p2pstore!"
