#!/bin/bash

# Stop Emergency Service - Simple version without logs or PID files

EMERGENCY_FLAG_DIR="${EMERGENCY_FLAG_DIR:-/tmp/emergency}"
EMERGENCY_FLAG="$EMERGENCY_FLAG_DIR/emergency_active"

# Kill node server if running on port 9085
pkill -f "node.*mode_server.js" 2>/dev/null || true

# Clear emergency flag
if [ -f "$EMERGENCY_FLAG" ]; then
    rm -f "$EMERGENCY_FLAG"
    echo "Emergency flag cleared"
fi

echo "Emergency service stopped"