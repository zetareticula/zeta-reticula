#!/bin/bash
set -e

# Base URL of the API
BASE_URL="http://localhost:3000/api"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test function with status check and response display
test_endpoint() {
  local method=$1
  local endpoint=$2
  local data=$3
  local token=$4
  
  echo -e "\n${YELLOW}Testing ${method} ${endpoint}${NC}"
  
  local response_file=$(mktemp)
  local status_code
  
  # Build curl command
  local cmd="curl -s -o ${response_file} -w '%{http_code}'"
  cmd+=" -X ${method}"
  cmd+=" -H 'Content-Type: application/json'"
  
  # Add token if provided
  if [ -n "$token" ]; then
    cmd+=" -H 'Authorization: Bearer ${token}'"
  fi
  
  # Add data for POST/PUT requests
  if [ -n "$data" ]; then
    cmd+=" -d '${data}'"
  fi
  
  # Add URL
  cmd+=" ${BASE_URL}${endpoint}"
  
  # Execute and get status code
  status_code=$(eval $cmd)
  
  # Print result
  if [[ $status_code == 2* ]] || [[ $status_code == 3* ]]; then
    echo -e "${GREEN}✅ Success (${status_code})${NC}"
    echo "Response:"
    cat $response_file | jq . 2>/dev/null || cat $response_file
  else
    echo -e "${RED}❌ Failed (${status_code})${NC}"
    echo "Response:"
    cat $response_file | jq . 2>/dev/null || cat $response_file
    echo -e "\nCommand: ${cmd}"
    rm -f $response_file
    return 1
  fi
  
  rm -f $response_file
}

# Check if jq is installed
if ! command -v jq &> /dev/null; then
  echo "jq is not installed. Installing..."
  brew install jq
fi

echo "=== Testing API Endpoints ==="

# Test health endpoint
echo -e "\n${YELLOW}=== Testing Health Endpoint ===${NC}"
test_endpoint "GET" "/health"

# Test model endpoints
echo -e "\n${YELLOW}=== Testing Model Endpoints ===${NC}"
test_endpoint "GET" "/models"
test_endpoint "GET" "/models/123" ""

# Test inference endpoints
echo -e "\n${YELLOW}=== Testing Inference Endpoints ===${NC}"
test_endpoint "POST" "/inference" '{"prompt": "Hello, world!"}'
test_endpoint "GET" "/inference/usage"

# Test authentication endpoints
echo -e "\n${YELLOW}=== Testing Authentication ===${NC}"
echo -e "${YELLOW}Note: Authentication tests will fail without a valid JWT token${NC}"
test_endpoint "POST" "/auth/login" '{"username": "test", "password": "test"}'

echo -e "\n${YELLOW}=== Test Summary ===${NC}"
echo "Note: Some endpoints may require authentication to work properly."
echo "To test authenticated endpoints, you need to obtain a JWT token first."
echo "You can do this by implementing the authentication flow or using a test token."
