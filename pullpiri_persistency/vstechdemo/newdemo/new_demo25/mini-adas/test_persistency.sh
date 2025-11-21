#!/bin/bash

echo "=== Testing ADAS Persistency ==="
echo "1. Starting adas_primary (agent 100) in background..."
cd /home/lg/work/s-core/modules/feo

# Clean up any existing data
rm -rf examples/rust/mini-adas/adas_data

# Start adas_primary in background
./target/debug/adas_primary 100 &
PRIMARY_PID=$!
echo "Primary started with PID: $PRIMARY_PID"

sleep 2

echo "2. Starting adas_secondary agent 101 in background..."
./target/debug/adas_secondary 1 &
SECONDARY1_PID=$!
echo "Secondary 1 started with PID: $SECONDARY1_PID"

sleep 2

echo "3. Starting adas_secondary agent 102 (with EmergencyBraking) in background..."
./target/debug/adas_secondary 2 &
SECONDARY2_PID=$!
echo "Secondary 2 started with PID: $SECONDARY2_PID"

echo "4. Waiting 15 seconds for persistency to trigger..."
sleep 15

echo "5. Checking for adas_data directory and files in mini-adas:"
ls -la examples/rust/mini-adas/adas_data/ 2>/dev/null || echo "No adas_data directory found"

echo "6. Checking for KVS files:"
find examples/rust/mini-adas/ -name "kvs_*.json" -o -name "kvs_*.hash" 2>/dev/null || echo "No KVS files found"

echo "7. Stopping all processes..."
kill $PRIMARY_PID $SECONDARY1_PID $SECONDARY2_PID 2>/dev/null
wait

echo "=== Test Complete ==="