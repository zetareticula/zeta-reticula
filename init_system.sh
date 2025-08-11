#!/bin/bash
set -e

# Create required directories
echo "Creating required directories..."
mkdir -p models attention-data vault-data quantization-results monitoring

# Set proper permissions
echo "Setting up permissions..."
chmod -R 777 models attention-data vault-data quantization-results

# Create a sample model configuration
echo "Creating sample model configuration..."
cat > models/model_config.json <<EOL
{
  "model_name": "llama2-7b",
  "model_path": "/app/models/llama2-7b",
  "quantization_bits": [1, 2, 4, 8, 16],
  "max_sequence_length": 4096,
  "attention_heads": 32,
  "hidden_size": 4096,
  "num_hidden_layers": 32,
  "vocab_size": 32000
}
EOL

echo "Initialization complete!"
echo "You can now start the system with: docker-compose -f docker-compose.full.yml up -d"
