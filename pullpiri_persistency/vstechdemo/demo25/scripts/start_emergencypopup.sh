#!/bin/bash

# Simple Emergency Service - Start node server and create emergency flag

# Use system directories for generic deployment
EMERGENCY_FLAG_DIR="${EMERGENCY_FLAG_DIR:-/tmp/emergency}"
NODE_SERVER_DIR="${NODE_SERVER_DIR:-/home/acrn/new_ak/demo25/dds_console_app/driving_mode}"
NODE_SERVER_PORT="${NODE_SERVER_PORT:-9085}"

# Create emergency flag directory if it doesn't exist
mkdir -p "$EMERGENCY_FLAG_DIR"

# Start node server if port not in use
if ! netstat -tuln | grep -q ":$NODE_SERVER_PORT "; then
    echo "Starting node server on port $NODE_SERVER_PORT..."
    cd "$NODE_SERVER_DIR"
    node mode_server.js &
fi

# Create emergency flag in system temp directory
echo "$(date)" > "$EMERGENCY_FLAG_DIR/emergency_active"
echo "Emergency activated! Flag created at: $EMERGENCY_FLAG_DIR/emergency_active"