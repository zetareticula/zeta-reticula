#!/bin/bash
set -e

# List of main Rust services (binaries)
SERVICES=(
  "agentflow-rs"
  "llm-rs"
  "ns-router-rs"
  "salience-engine"
  "kvquant-rs"
  "api"
)

# Path to the root Dockerfile template
DOCKERFILE="Dockerfile"

for SERVICE in "${SERVICES[@]}"; do
  echo "\n=== Building Docker image for $SERVICE ==="
  # Copy Dockerfile and replace <service-name> with actual binary name
  TMP_DOCKERFILE="Dockerfile.$SERVICE.tmp"
  sed "s/<service-name>/$SERVICE/g" "$DOCKERFILE" > "$TMP_DOCKERFILE"
  docker build -f "$TMP_DOCKERFILE" -t "$SERVICE:latest" .
  rm "$TMP_DOCKERFILE"
done

echo "\nAll Docker images built successfully."
