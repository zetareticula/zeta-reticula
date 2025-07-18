#!/bin/bash
set -e

# Build the project
cargo build --release

# Build Docker image
docker build -t zetareticula/p2pstore:latest .

# Push to container registry (if needed)
# docker push zetareticula/p2pstore:latest

echo "Build complete. To run locally:"
echo "docker run -p 50051:50051 zetareticula/p2pstore:latest"
