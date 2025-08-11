#!/bin/bash

# Set environment variables
export RUST_LOG=info
export PORT=8080
MODEL_PATH="./models/example.gguf"
QUANTIZED_MODEL="./models/example_q8_0.gguf"
API_URL="http://localhost:${PORT}"

# Create models directory if it doesn't exist
mkdir -p ./models

# Function to check if a service is running
wait_for_service() {
    echo "Waiting for $1 to be ready..."
    until curl -s "$API_URL/health" >/dev/null 2>&1; do
        sleep 2
    done
    echo "$1 is ready!"
}

# 1. Start the inference server in the background
echo "Starting inference server..."
cargo run --bin zeta-infer -- infer --model "$MODEL_PATH" --port "$PORT" 2>&1 > server.log &
INFERENCE_PID=$!

# Wait for server to start with timeout
wait_for_service "Inference server"

# 2. Check if we need to download a test model
if [ ! -f "$MODEL_PATH" ]; then
    echo "Downloading test model..."
    mkdir -p models
    # Download a small test model (adjust URL as needed)
    curl -L -o "$MODEL_PATH" "https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q8_0.gguf"
fi

# 3. Quantize the model (if not already quantized)
if [ ! -f "$QUANTIZED_MODEL" ]; then
    echo "Quantizing model..."
    cargo run --bin zeta-infer -- quantize \
        --model "$MODEL_PATH" \
        --bits 8 \
        --chunk 64 \
        --output "$QUANTIZED_MODEL"
fi

# 4. Test API endpoints
echo -e "\nTesting API endpoints..."

# Health check
echo -e "\n[TEST] Health check:"
curl -s "$API_URL/health" | jq '.'

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo "jq is not installed. Installing..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        brew install jq
    else
        sudo apt-get update && sudo apt-get install -y jq
    fi
fi

# Register a test user (if endpoint exists)
echo -e "\n[TEST] User registration:"
REGISTER_RESPONSE=$(curl -s -X POST "$API_URL/auth/register" \
    -H "Content-Type: application/json" \
    -d '{"username":"testuser", "password":"testpass"}' 2>/dev/null || echo '{"error":"Endpoint not available"}')
echo $REGISTER_RESPONSE | jq '.'

# Extract token if registration was successful
TOKEN=$(echo $REGISTER_RESPONSE | jq -r '.token // empty')

if [ -z "$TOKEN" ]; then
    echo "Using mock token for testing"
    TOKEN="mock_token_$(uuidgen)"
fi

# Make an inference request
echo -e "\n[TEST] Making inference request:"
INFERENCE_RESPONSE=$(curl -s -X POST "$API_URL/infer" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $TOKEN" \
    -d '{"prompt": "Explain how attention works in one sentence", "max_tokens": 100, "temperature": 0.7}' 2>/dev/null || echo '{"error":"Inference endpoint not available"}')
echo $INFERENCE_RESPONSE | jq '.'

# Extract request ID if available
REQUEST_ID=$(echo $INFERENCE_RESPONSE | jq -r '.request_id // empty')

if [ ! -z "$REQUEST_ID" ]; then
    # Check agentflow status
    echo -e "\n[TEST] Agentflow status:"
    curl -s "$API_URL/agentflow/status/$REQUEST_ID" -H "Authorization: Bearer $TOKEN" 2>/dev/null | jq '.' || echo "Agentflow status endpoint not available"
    
    # Get attention patterns
    echo -e "\n[TEST] Attention patterns:"
    curl -s "$API_URL/attention/patterns/$REQUEST_ID" -H "Authorization: Bearer $TOKEN" 2>/dev/null | jq '.' || echo "Attention patterns endpoint not available"
fi

# Get salience metrics
echo -e "\n[TEST] Salience metrics:"
curl -s "$API_URL/salience/metrics" -H "Authorization: Bearer $TOKEN" 2>/dev/null | jq '.' || echo "Salience metrics endpoint not available"

# 5. Clean up
echo -e "\nCleaning up..."
kill $INFERENCE_PID 2>/dev/null

echo -e "\nTest completed! Check server.log for detailed logs."
