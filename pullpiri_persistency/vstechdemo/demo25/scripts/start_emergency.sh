#!/bin/bash
# Stop other driving modes if running
systemctl stop manual_driving.service 2>/dev/null || true
systemctl stop autonomous_driving.service 2>/dev/null || true

# Start the DDS emergency app (with REST API on port 9082)
/home/acrn/new_ak/demo25/dds_console_app/dds_emergency_app/target/debug/dds_emergency_app > /tmp/emergency_dds.log 2>&1 &
DDS_PID=$!

# Start the React dashboard
cd /home/acrn/new_ak/demo25/Dashboards/Emergency_app
npm run dev > /tmp/emergency_dashboard.log 2>&1 &
DASHBOARD_PID=$!

# Store PIDs for cleanup
echo $DDS_PID > /tmp/emergency_dds.pid
echo $DASHBOARD_PID > /tmp/emergency_dashboard.pid

echo "Emergency driving mode started - DDS API: http://localhost:9082/data"
echo "DDS PID: $DDS_PID, Dashboard PID: $DASHBOARD_PID"

# Wait for either process to exit
wait $DDS_PID $DASHBOARD_PID
