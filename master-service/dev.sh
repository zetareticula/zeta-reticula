#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
ACTION="help"
ENV_FILE=".env"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  key="$1"
  case $key in
    build)
      ACTION="build"
      shift
      ;;
    test)
      ACTION="test"
      shift
      ;;
    run)
      ACTION="run"
      shift
      ;;
    docker-build)
      ACTION="docker-build"
      shift
      ;;
    docker-run)
      ACTION="docker-run"
      shift
      ;;
    --env)
      ENV_FILE="$2"
      shift
      shift
      ;;
    -h|--help|help)
      ACTION="help"
      shift
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  escapedone

# Load environment variables if file exists
if [ -f "$ENV_FILE" ]; then
  echo -e "${YELLOW}Loading environment variables from $ENV_FILE${NC}"
  export $(grep -v '^#' "$ENV_FILE" | xargs)
fi

# Help message
show_help() {
  cat << EOF
Usage: $0 <command> [options]

Commands:
  build         Build the project
  test          Run tests
  run           Run the service locally
  docker-build  Build Docker image
  docker-run    Run Docker container
  help          Show this help message

Options:
  --env FILE    Load environment variables from FILE (default: .env)
  -h, --help    Show this help message

Examples:
  $0 build
  $0 test
  $0 run
  $0 docker-build
  $0 docker-run --env .env.local
"""
}

# Build the project
build() {
  echo -e "${GREEN}Building project...${NC}"
  cargo build
}

# Run tests
test() {
  echo -e "${GREEN}Running tests...${NC}"
  cargo test -- --nocapture
}

# Run the service locally
run() {
  echo -e "${GREEN}Starting master service...${NC}"
  cargo run --release
}

# Build Docker image
docker_build() {
  echo -e "${GREEN}Building Docker image...${NC}"
  docker build -t zetareticula/master-service:latest .
}

# Run Docker container
docker_run() {
  echo -e "${GREEN}Starting Docker container...${NC}"
  docker run -it --rm \
    -p 50051:50051 \
    -e RUST_LOG=info \
    -e BIND_ADDR=0.0.0.0:50051 \
    -e NODE_TIMEOUT_SECONDS=300 \
    zetareticula/master-service:latest
}

# Main command dispatcher
case $ACTION in
  build)
    build
    ;;
  test)
    test
    ;;
  run)
    run
    ;;
  docker-build)
    docker_build
    ;;
  docker-run)
    docker_run
    ;;
  help|*)
    show_help
    ;;
esac

echo -e "${GREEN}Done!${NC}"
