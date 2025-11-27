#!/bin/bash

echo "🧪 Testing DDS Communication Between Sender and Receiver"
echo "=================================================="

# Check if both applications are built
echo "📋 Checking if applications are built..."

if [ ! -f "./dds_backend_app/target/debug/dds_backend_receiver" ]; then
    echo "❌ Backend receiver not built. Building..."
    cd ./dds_backend_app
    cargo build
    if [ $? -ne 0 ]; then
        echo "❌ Failed to build backend receiver"
        exit 1
    fi
    cd ..
fi

if [ ! -f "./container-app/dds-message-sender/target/debug/dds_message_sender" ]; then
    echo "❌ Message sender not built. Building..."
    cd ./container-app/dds-message-sender
    cargo build
    if [ $? -ne 0 ]; then
        echo "❌ Failed to build message sender"
        exit 1
    fi
    cd ../..
fi

echo "✅ Both applications are built"
echo ""

# Start backend receiver in background

echo "🚀 Step 1: Starting DDS Backend Receiver..."

# Allow custom REST API address/port for backend
REST_API_ADDR=${REST_API_ADDR:-0.0.0.0}
REST_API_PORT=${REST_API_PORT:-8080}

cd ./dds_backend_app
REST_API_ADDR="$REST_API_ADDR" REST_API_PORT="$REST_API_PORT" ./target/debug/dds_backend_receiver &
BACKEND_PID=$!
cd ..

echo "   ✓ Backend receiver started (PID: $BACKEND_PID)"
echo "   🕐 Waiting 3 seconds for backend to initialize..."
sleep 3

# Test if backend is responding
echo "🌐 Step 2: Testing Backend REST API..."
if curl -s http://localhost:${REST_API_PORT}/health > /dev/null; then
    echo "   ✓ Backend REST API is responding"
else
    echo "   ⚠️  Backend REST API not yet responding, continuing anyway..."
fi

echo ""
echo "📤 Step 3: Starting DDS Message Sender..."

cd ./container-app/dds-message-sender

# Set environment variables for testing
export SCENARIO_NAME="test_scenario"
export MESSAGE_TYPE="test_driver_distraction"
export MESSAGE_CONTENT="Test message from manual sender"
export SEVERITY="info"
export THRESHOLD_VALUE="3.0"

echo "   📋 Test configuration:"
echo "      Scenario: $SCENARIO_NAME"
echo "      Message: $MESSAGE_CONTENT"
echo "      Severity: $SEVERITY"
echo ""

# Run the sender
./target/debug/dds_message_sender
cd ../..

echo ""
echo "🔍 Step 4: Checking if message was received..."
sleep 2

# Check backend for received messages
echo "📊 Backend Status:"
curl -s http://localhost:${REST_API_PORT}/status | jq '.' 2>/dev/null || echo "Could not query backend status"

echo ""
echo "📬 Latest Messages:"
curl -s http://localhost:${REST_API_PORT}/messages/latest | jq '.' 2>/dev/null || echo "Could not query latest messages"

echo ""
echo "🛑 Step 5: Stopping Backend Receiver..."
kill $BACKEND_PID 2>/dev/null
wait $BACKEND_PID 2>/dev/null

echo "✅ Test completed!"
echo ""
echo "💡 If you saw messages being received, DDS communication is working!"
echo "💡 If no messages were received, there may be a DDS networking issue."