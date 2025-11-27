#!/bin/bash

# Build script for File-Based Driver Distraction System

set -e

echo "🔧 Building File-Based Driver Distraction System..."
echo "=================================================="

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

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Build Rust file message writer container
print_status "Building file-message-writer container..."
cd "${SCRIPT_DIR}/container-app/file-message-writer"
sudo podman build -t file-message-writer:latest .
if [ $? -eq 0 ]; then
    print_success "file-message-writer:latest built successfully"
else
    print_error "Failed to build file-message-writer"
    exit 1
fi

echo ""
print_success "🎉 File-based system containers built successfully!"
echo ""
print_status "📦 Available images:"
sudo podman images | grep -E "(file-message-writer)"

echo ""
print_status "🚀 Usage Instructions:"
print_status "======================"
echo ""
print_status "1. 🐍 Start File Monitoring Backend:"
echo "   cd file_backend_app"
echo "   pip3 install -r requirements.txt"
echo "   sudo mkdir -p /tmp/driver_distraction"
echo "   sudo chmod 777 /tmp/driver_distraction"
echo "   python3 file_monitor_backend.py"
echo "   # Backend will run on http://127.0.0.1:8081"
echo ""
print_status "2. 📄 Deploy Pullpiri scenarios:"
echo "   curl -X POST http://pullpiri-server:8080/api/scenarios \\"
echo "        -H 'Content-Type: application/yaml' \\"
echo "        -d @pullpiri/examples/resources/driver-distraction-5sec.yaml"
echo ""
print_status "3. 🧪 Test file message writer manually:"
echo "   sudo mkdir -p /tmp/driver_distraction"
echo "   sudo podman run --rm \\"
echo "     -v /tmp/driver_distraction:/data \\"
echo "     -e MESSAGE_TYPE='driver_distraction' \\"
echo "     -e SCENARIO_NAME='test-scenario' \\"
echo "     -e MESSAGE_CONTENT='Test file-based message' \\"
echo "     -e SEVERITY='warning' \\"
echo "     file-message-writer:latest"
echo ""
print_status "4. 📡 Check REST API endpoints:"
echo "   curl http://localhost:8081/data     # Latest file message"
echo "   curl http://localhost:8081/health   # Health check"
echo "   curl http://localhost:8081/status   # Detailed status"
echo ""
print_status "5. 📁 Check written files:"
echo "   cat /tmp/driver_distraction/driver_distraction_messages.json"
echo "   ls /tmp/driver_distraction/history/"
echo ""
print_status "📝 File-based system ready! No DDS required."