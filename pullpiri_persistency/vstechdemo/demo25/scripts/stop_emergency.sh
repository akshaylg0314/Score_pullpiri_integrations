#!/bin/bash
# Read PIDs
DDS_PID=$(cat /tmp/emergency_dds.pid 2>/dev/null)
DASHBOARD_PID=$(cat /tmp/emergency_dashboard.pid 2>/dev/null)

echo "Stopping emergency driving services..."

# Kill dashboard first (gracefully)
if [ ! -z "$DASHBOARD_PID" ]; then
  echo "Stopping dashboard (PID: $DASHBOARD_PID)"
  kill $DASHBOARD_PID 2>/dev/null
fi

# Kill DDS app (gracefully)
if [ ! -z "$DDS_PID" ]; then
  echo "Stopping DDS app (PID: $DDS_PID)"
  kill $DDS_PID 2>/dev/null
fi


# Force kill if still running
pkill -f dds_emergency_app 2>/dev/null || true
pkill -f "npm.*Emergency_app" 2>/dev/null || true

# Clean up PID files
rm -f /tmp/emergency_dds.pid /tmp/emergency_dashboard.pid

echo "Emergency driving services stopped"

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
