#!/bin/bash
# Start the React dashboard
cd /home/acrn/new_ak/demo25/Dashboards/Infotainment_app
npm run dev > /tmp/infotainment_dashboard.log 2>&1 &
DASHBOARD_PID=$!

# Store PIDs for cleanup
echo $DASHBOARD_PID > /tmp/infotainment_dashboard.pid

echo "Dashboard PID: $DASHBOARD_PID"

# Wait for process to exit
wait $DASHBOARD_PID