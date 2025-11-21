#!/bin/bash
# Read PIDs
DDS_PID=$(cat /tmp/manual_dds.pid 2>/dev/null)
DASHBOARD_PID=$(cat /tmp/manual_dashboard.pid 2>/dev/null)

echo "Stopping manual driving services..."

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

sleep 1

# Force kill if still running
pkill -f dds_manual_app 2>/dev/null || true
pkill -f "npm.*Manual_app" 2>/dev/null || true

# Clean up PID files
rm -f /tmp/manual_dds.pid /tmp/manual_dashboard.pid

echo "Manual driving services stopped"
