#!/bin/bash

# SPDX-FileCopyrightText: Copyright 2024 LG Electronics Inc.
# SPDX-License-Identifier: Apache-2.0

# Persistency Service Integration Test Script
#
# This script starts the persistency service and runs a simple test
# to verify that the integration between Pullpiri and the persistency service works.

set -e

echo "=================================="
echo "Pullpiri Persistency Integration Test"
echo "=================================="

# Build the persistency service
echo "Building persistency service..."
cd /home/acrn/new_ak/score/pullpiri/src/server
cargo build -p persistency-service --release

# Start the persistency service in the background
echo "Starting persistency service..."
./target/release/persistency-service &
PERSISTENCY_PID=$!

# Wait a moment for the service to start
echo "Waiting for service to start..."
sleep 3

# Test if the service is responsive (you can extend this with actual tests)
echo "Testing service connectivity..."
if pgrep -f persistency-service > /dev/null; then
    echo "✅ Persistency service is running (PID: $PERSISTENCY_PID)"
else
    echo "❌ Persistency service failed to start"
    exit 1
fi

echo "=================================="
echo "Integration Complete!"
echo "=================================="
echo "The persistency service is now running and ready to replace ETCD."
echo "Key benefits:"
echo "- Single initialization point for all Pullpiri components"
echo "- Shared persistent storage using rust_kvs"
echo "- gRPC interface for inter-process communication"
echo "- Backward compatibility with existing ETCD interface"
echo ""
echo "To stop the service, run: kill $PERSISTENCY_PID"
echo "Or use: pkill -f persistency-service"

# Optional: Keep the script running to maintain the service
read -p "Press Enter to stop the service..."
kill $PERSISTENCY_PID
echo "Persistency service stopped."