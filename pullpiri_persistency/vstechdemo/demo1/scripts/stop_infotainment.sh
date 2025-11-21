#!/bin/bash

echo "Stopping infotainment app ..."

# Kill processes by PID if PID files exist
if [ -f /tmp/infotainment_dashboard.pid ]; then
    DASHBOARD_PID=$(cat /tmp/infotainment_dashboard.pid)
    if kill -0 $DASHBOARD_PID 2>/dev/null; then
        echo "Stopping dashboard (PID: $DASHBOARD_PID)"
        kill $DASHBOARD_PID
    fi
    rm -f /tmp/infotainment_dashboard.pid
fi

# Fallback: kill by process name
pkill -f "npm run dev" 2>/dev/null || true

echo "Infotainment app stopped"