#!/bin/bash

# Initialize the models directory with a sample model if it doesn't exist

set -e

MODELS_DIR="./models"
SAMPLE_MODEL_URL="https://huggingface.co/facebook/opt-125m/resolve/main/pytorch_model.bin"

# Create models directory if it doesn't exist
mkdir -p "$MODELS_DIR"

# Check if models directory is empty
if [ -z "$(ls -A $MODELS_DIR)" ]; then
    echo "No models found in $MODELS_DIR"
    echo "Downloading sample model..."
    
    # Download a small sample model
    wget -O "${MODELS_DIR}/opt-125m.bin" "$SAMPLE_MODEL_URL"
    
    if [ $? -eq 0 ]; then
        echo "Sample model downloaded successfully to ${MODELS_DIR}/opt-125m.bin"
    else
        echo "Failed to download sample model. Please add your models to the $MODELS_DIR directory."
        exit 1
    fi
else
    echo "Models directory already contains files. Skipping sample model download."
    echo "Found the following models:"
    ls -lh "$MODELS_DIR"
fi

echo "Models directory is ready!"
