#!/bin/bash
# Stop other driving modes if running
systemctl stop manual_driving.service 2>/dev/null || true
systemctl stop emergency_driving.service 2>/dev/null || true

# Start the DDS autonomous app (with REST API on port 9083)
/home/acrn/new_ak/demo25/dds_console_app/dds_autonomous_app/target/debug/dds_autonomous_app &
DDS_PID=$!

# Start the React dashboard
cd /home/acrn/new_ak/demo25/Dashboards/Autonomous_app
npm run dev > /tmp/autonomous_dashboard.log 2>&1 &
DASHBOARD_PID=$!

# Store PIDs for cleanup
echo $DDS_PID > /tmp/autonomous_dds.pid
echo $DASHBOARD_PID > /tmp/autonomous_dashboard.pid

echo "Autonomous driving mode started - DDS API: http://localhost:9083/data"
echo "DDS PID: $DDS_PID, Dashboard PID: $DASHBOARD_PID"

# Wait for either process to exit
wait $DDS_PID $DASHBOARD_PID
