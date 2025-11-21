#!/bin/bash

# Copy emergency thresholds to active thresholds file
THRESHOLD_DIR="/home/acrn/new_ak/demo25/new_demo25/dds_console_app/dds_vehicle_app"
if [ -f "$THRESHOLD_DIR/thresholds_emergency.json" ]; then
    cp "$THRESHOLD_DIR/thresholds_emergency.json" "$THRESHOLD_DIR/thresholds.json"
    echo "🚨 Emergency thresholds loaded"
else
    echo "⚠️  Warning: thresholds_emergency.json not found, using existing thresholds.json"
fi

