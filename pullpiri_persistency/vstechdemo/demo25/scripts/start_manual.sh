#!/bin/bash
# Stop other driving modes if running
systemctl stop emergency_driving.service 2>/dev/null || true
systemctl stop autonomous_driving.service 2>/dev/null || true

# Start the DDS manual app (with REST API on port 9081)
/home/acrn/new_ak/demo25/dds_console_app/dds_manual_app/target/debug/dds_manual_app > /tmp/manual_dds.log 2>&1 &
DDS_PID=$!

# Start the React dashboard
cd /home/acrn/new_ak/demo25/Dashboards/Manual_app
npm run dev > /tmp/manual_dashboard.log 2>&1 &
DASHBOARD_PID=$!

# Store PIDs for cleanup
echo $DDS_PID > /tmp/manual_dds.pid
echo $DASHBOARD_PID > /tmp/manual_dashboard.pid

echo "Manual driving mode started - DDS API: http://localhost:9081/data"
echo "DDS PID: $DDS_PID, Dashboard PID: $DASHBOARD_PID"

# Wait for either process to exit
wait $DDS_PID $DASHBOARD_PID
