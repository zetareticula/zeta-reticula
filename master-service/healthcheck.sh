#!/bin/bash
set -e

# Default values
HEALTH_CHECK_TIMEOUT=60
SERVICE_ADDR="localhost:50051"
HEALTH_CHECK_INTERVAL=5
VERBOSE=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  key="$1"
  case $key in
    --address)
      SERVICE_ADDR="$2"
      shift # past argument
      shift # past value
      ;;
    --timeout)
      HEALTH_CHECK_TIMEOUT="$2"
      shift # past argument
      shift # past value
      ;;
    --interval)
      HEALTH_CHECK_INTERVAL="$2"
      shift # past argument
      shift # past value
      ;;
    -v|--verbose)
      VERBOSE=true
      shift # past argument
      ;;
    -h|--help)
      echo "Health check script for Master Service"
      echo ""
      echo "Usage: $0 [options]"
      echo "  --address ADDR   Service address (default: localhost:50051)"
      echo "  --timeout SEC    Maximum time to wait in seconds (default: 60)"
      echo "  --interval SEC   Time between retries in seconds (default: 5)"
      echo "  -v, --verbose    Enable verbose output"
      echo "  -h, --help       Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if grpcurl is installed
if ! command -v grpcurl &> /dev/null; then
  echo -e "${YELLOW}grpcurl is not installed. Installing...${NC}"
  if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    curl -sSL https://github.com/fullstorydev/grpcurl/releases/latest/download/grpcurl_$(uname -m)_linux.tar.gz | tar xz
    chmod +x grpcurl
    sudo mv grpcurl /usr/local/bin/
  elif [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    brew install grpcurl
  else
    echo -e "${RED}Unsupported OS. Please install grpcurl manually.${NC}"
    exit 1
  fi
fi

# Function to check service health
check_health() {
  local status
  status=$(grpcurl -plaintext -d '{"service": "grpc.health.v1.Health"}' \
    -connect-timeout 3 \
    -max-time 5 \
    $SERVICE_ADDR grpc.health.v1.Health/Check 2>&1 | grep -o '"status":"[A-Z_]*"' | cut -d'"' -f4)
  
  if [ "$status" = "SERVING" ]; then
    echo -e "${GREEN}Service is healthy${NC}"
    return 0
  else
    if [ "$VERBOSE" = true ]; then
      echo -e "${YELLOW}Service not ready: $status${NC}"
    fi
    return 1
  fi
}

# Main health check loop
start_time=$(date +%s)
timeout_time=$((start_time + HEALTH_CHECK_TIMEOUT))

while true; do
  current_time=$(date +%s)
  
  if [ $current_time -ge $timeout_time ]; then
    echo -e "${RED}Health check timed out after $HEALTH_CHECK_TIMEOUT seconds${NC}"
    exit 1
  fi
  
  if check_health; then
    exit 0
  fi
  
  sleep $HEALTH_CHECK_INTERVAL
done
