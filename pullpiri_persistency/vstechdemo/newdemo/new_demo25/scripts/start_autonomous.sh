#!/bin/bash

# Copy autonomous thresholds to active thresholds file
THRESHOLD_DIR="/home/acrn/new_ak/demo25/new_demo25/dds_console_app/dds_vehicle_app"
if [ -f "$THRESHOLD_DIR/thresholds_autonomous.json" ]; then
    cp "$THRESHOLD_DIR/thresholds_autonomous.json" "$THRESHOLD_DIR/thresholds.json"
    echo "✅ Autonomous thresholds loaded"
else
    echo "⚠️  Warning: thresholds_autonomous.json not found, using existing thresholds.json"
fi

