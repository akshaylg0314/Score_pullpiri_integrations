#!/bin/bash

echo "Stopping autonomous driving services..."

# Kill processes by PID if PID files exist
if [ -f /tmp/autonomous_dds.pid ]; then
    DDS_PID=$(cat /tmp/autonomous_dds.pid)
    if kill -0 $DDS_PID 2>/dev/null; then
        echo "Stopping DDS app (PID: $DDS_PID)"
        kill $DDS_PID
    fi
    rm -f /tmp/autonomous_dds.pid
fi

if [ -f /tmp/autonomous_dashboard.pid ]; then
    DASHBOARD_PID=$(cat /tmp/autonomous_dashboard.pid)
    if kill -0 $DASHBOARD_PID 2>/dev/null; then
        echo "Stopping dashboard (PID: $DASHBOARD_PID)"
        kill $DASHBOARD_PID
    fi
    rm -f /tmp/autonomous_dashboard.pid
fi

# Fallback: kill by process name
pkill -f "dds_autonomous_app" 2>/dev/null || true
pkill -f "npm run dev" 2>/dev/null || true

echo "Autonomous driving services stopped"
