#!/bin/bash
# Build script for Rust DDS system containers

set -e

echo "🔧 Building Simplified Rust DDS System Container Images..."
echo "============================================================="

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Build Rust DDS message sender container
echo "Building dds-message-sender container..."
cd "${SCRIPT_DIR}/container-app/dds-message-sender"
podman build -t dds-message-sender:latest .
if [ $? -eq 0 ]; then
    echo "✅ dds-message-sender:latest built successfully"
else
    echo "❌ Failed to build dds-message-sender"
    exit 1
fi

# Build Rust DDS backend receiver container (optional - mainly for testing)
echo "Building dds-backend-receiver container..."
cd "${SCRIPT_DIR}/dds_backend_app"
podman build -t dds-backend-receiver:latest .
if [ $? -eq 0 ]; then
    echo "✅ dds-backend-receiver:latest built successfully"
else
    echo "❌ Failed to build dds-backend-receiver"
    exit 1
fi

echo ""
echo "🎉 All Simplified DDS containers built successfully!"
echo ""
echo "📦 Available images:"
sudo podman images | grep -E "(dds-message-sender|dds-backend-receiver)"

echo ""
echo "🚀 Usage Instructions:"
echo "======================"
echo ""
echo "1. 🔧 Start DDS Backend Receiver (recommended - run directly):"
echo "   cd dds_backend_app"
echo "   sudo cargo run"
echo "   # Backend will run on http://127.0.0.1:8081"
echo ""
echo "2. 🐳 Alternative: Run backend as container:"
echo "   podman run -d --name dds-backend --network=host dds-backend-receiver:latest"
echo ""
echo "3. 📄 Deploy Pullpiri scenarios:"
echo "   curl -X POST http://pullpiri-server:8080/api/scenarios \\"
echo "        -H 'Content-Type: application/yaml' \\"
echo "        -d @pullpiri/examples/resources/driver-distraction-5sec.yaml"
echo ""
echo "4. 🧪 Test DDS message sender manually:"
echo "   podman run --rm --network=host \\"
echo "     -e MESSAGE_TYPE='driver_distraction' \\"
echo "     -e SCENARIO_NAME='test-scenario' \\"
echo "     -e MESSAGE_CONTENT='Test distraction event' \\"
echo "     -e SEVERITY='warning' \\"
echo "     dds-message-sender:latest"
echo ""
echo "5. 📡 Check REST API endpoints:"
echo "   curl http://localhost:8081/data     # Latest DDS message"
echo "   curl http://localhost:8081/health   # Health check"
echo ""
echo "📝 Run './test_simplified_backend.sh' for comprehensive testing!"