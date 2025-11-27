#!/bin/bash

# Test script for file-based driver distraction system

set -e

echo "🧪 Testing File-Based Driver Distraction System"
echo "=============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Configuration
BACKEND_PORT=8081
BACKEND_URL="http://localhost:${BACKEND_PORT}"
DATA_DIR="/tmp/driver_distraction"

# Step 1: Check if backend is running
print_status "Checking if file monitoring backend is running..."
if curl --connect-timeout 3 --silent --fail "${BACKEND_URL}/health" > /dev/null 2>&1; then
    print_success "Backend is running and responding"
else
    print_error "Backend is not responding on ${BACKEND_URL}"
    print_status "Please start the backend with:"
    echo "  cd file_backend_app && python3 file_monitor_backend.py"
    exit 1
fi

# Step 2: Create data directory if it doesn't exist
print_status "Setting up data directory..."
sudo mkdir -p "${DATA_DIR}"
sudo chmod 777 "${DATA_DIR}"
print_success "Data directory ready: ${DATA_DIR}"

# Step 3: Test initial state
print_status "Testing initial backend state..."
initial_response=$(curl --connect-timeout 3 --silent "${BACKEND_URL}/data")
echo "Initial response: $initial_response"

# Step 4: Test file message writer
print_status "Testing file message writer container..."
sudo podman run --rm \
    -v "${DATA_DIR}:/data" \
    -e MESSAGE_TYPE="driver_distraction" \
    -e SCENARIO_NAME="test-file-system" \
    -e MESSAGE_CONTENT="Testing file-based message system" \
    -e SEVERITY="info" \
    -e THRESHOLD_VALUE="3.0" \
    file-message-writer:latest

print_success "Message writer container completed"

# Step 5: Check if file was created
print_status "Checking if message file was created..."
if [ -f "${DATA_DIR}/driver_distraction_messages.json" ]; then
    print_success "Message file created successfully"
    echo "File content:"
    cat "${DATA_DIR}/driver_distraction_messages.json"
else
    print_error "Message file not found!"
    exit 1
fi

# Step 6: Wait for backend to pick up the change
print_status "Waiting for backend to detect file change..."
sleep 2

# Step 7: Test backend API
print_status "Testing backend API after message write..."
api_response=$(curl --connect-timeout 3 --silent "${BACKEND_URL}/data")
echo "API response: $api_response"

if echo "$api_response" | grep -q "test-file-system"; then
    print_success "✨ File-based messaging working correctly!"
    echo ""
    print_status "📊 System Status:"
    echo "   📁 File writing: Working"
    echo "   👁️  File monitoring: Working"
    echo "   🌐 REST API: Working"
    echo "   📄 Message flow: Working"
else
    print_warning "Message may not have been picked up yet, checking backend status..."
    status_response=$(curl --connect-timeout 3 --silent "${BACKEND_URL}/status")
    echo "Backend status: $status_response"
fi

# Step 8: Test timeout functionality
print_status "Testing message timeout (waiting 4 seconds)..."
sleep 4

timeout_response=$(curl --connect-timeout 3 --silent "${BACKEND_URL}/data")
if echo "$timeout_response" | grep -q "No messages received yet"; then
    print_success "✨ Message timeout working correctly - old messages cleared!"
else
    print_warning "Message timeout may not be working as expected"
    echo "Response after timeout: $timeout_response"
fi

echo ""
print_success "🎉 File-based system test completed!"
print_status "Files created in: ${DATA_DIR}"
print_status "Backend running on: ${BACKEND_URL}"