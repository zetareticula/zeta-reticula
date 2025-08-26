#!/bin/bash

# Load environment variables
set -a
source .env
set +a

# Create necessary directories
mkdir -p ./logs
mkdir -p ./data/kvstore
mkdir -p ./models

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check for required commands
for cmd in cargo docker docker-compose; do
    if ! command_exists "$cmd"; then
        echo "Error: $cmd is not installed"
        exit 1
    fi
done

# Build all Rust components
echo "Building Rust components..."
cd kvquant_rs && cargo build --release && cd ..
cd distributed-store && cargo build --release && cd ..
cd llm-rs && cargo build --release && cd ..
cd salience-engine && cargo build --release && cd ..
cd agentflow-rs && cargo build --release && cd ..
cd quantize-cli && cargo build --release && cd ..
cd master-service && cargo build --release && cd ..

# Start the distributed store
echo "Starting distributed store..."
cd distributed-store
cargo run --release -- \
    --host $DISTRIBUTED_STORE_HOST \
    --port $DISTRIBUTED_STORE_PORT \
    --replication $DISTRIBUTED_STORE_REPLICATION_FACTOR \
    --data-dir ../data/kvstore &
DISTRIBUTED_STORE_PID=$!
cd ..

# Wait for distributed store to be ready
sleep 5

# Start the master service
echo "Starting master service..."
cd master-service
cargo run --release -- \
    --host $MASTER_SERVICE_HOST \
    --port $MASTER_SERVICE_PORT \
    --distributed-store-url "http://$DISTRIBUTED_STORE_HOST:$DISTRIBUTED_STORE_PORT" &
MASTER_SERVICE_PID=$!
cd ..

# Wait for master service to be ready
sleep 3

# Start AgentFlow
echo "Starting AgentFlow..."
cd agentflow-rs
cargo run --release -- \
    --host $AGENTFLOW_HOST \
    --port $AGENTFLOW_PORT \
    --master-service-url "http://$MASTER_SERVICE_HOST:$MASTER_SERVICE_PORT" &
AGENTFLOW_PID=$!
cd ..

# Quantize the model if needed
if [ ! -d "$QUANTIZED_MODEL_PATH" ]; then
    echo "Quantizing model..."
    cd quantize-cli
    cargo run --release -- \
        --model-path "../$MODEL_PATH" \
        --output-path "../$QUANTIZED_MODEL_PATH" \
        --bits $QUANTIZE_BITS \
        --group-size $QUANTIZE_GROUP_SIZE \
        --act-order $QUANTIZE_ACT_ORDER
    cd ..
fi

# Start the salience engine
echo "Starting salience engine..."
cd salience-engine
cargo run --release -- \
    --host $SALIENCE_ENGINE_HOST \
    --port $SALIENCE_ENGINE_PORT \
    --model-path "../$QUANTIZED_MODEL_PATH" \
    --model-type $MODEL_TYPE \
    --distributed-store-url "http://$DISTRIBUTED_STORE_HOST:$DISTRIBUTED_STORE_PORT" \
    --master-service-url "http://$MASTER_SERVICE_HOST:$MASTER_SERVICE_PORT" \
    --num-workers $NUM_WORKERS \
    --token-chunk-size $TOKEN_CHUNK_SIZE &
SALIENCE_ENGINE_PID=$!
cd ..

echo "All components started successfully!"
echo "- Distributed Store: http://$DISTRIBUTED_STORE_HOST:$DISTRIBUTED_STORE_PORT"
echo "- Master Service: http://$MASTER_SERVICE_HOST:$MASTER_SERVICE_PORT"
echo "- AgentFlow: http://$AGENTFLOW_HOST:$AGENTFLOW_PORT"
echo "- Salience Engine: http://$SALIENCE_ENGINE_HOST:$SALIENCE_ENGINE_PORT"

# Cleanup function
cleanup() {
    echo "Shutting down components..."
    kill $SALIENCE_ENGINE_PID $AGENTFLOW_PID $MASTER_SERVICE_PID $DISTRIBUTED_STORE_PID 2>/dev/null
    exit 0
}

# Set up trap to catch termination signals
trap cleanup INT TERM

# Keep the script running
wait

# If we get here, something went wrong
echo "Error: One or more components failed to start"
cleanup
