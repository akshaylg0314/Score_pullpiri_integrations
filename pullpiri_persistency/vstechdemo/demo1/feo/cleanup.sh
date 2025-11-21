#!/bin/bash
# FEO System Cleanup Script

echo "Cleaning up FEO system..."

# Kill any lingering FEO processes
echo "Killing FEO processes..."
sudo pkill -f adas || true
sudo pkill -f feo || true
sudo pkill -f tracer || true

# Wait a moment for processes to die
sleep 2

# Clean shared memory segments
echo "Cleaning shared memory..."
sudo rm -rf /dev/shm/iceoryx* /dev/shm/roudi* /dev/shm/dds* /dev/shm/feo* 2>/dev/null || true

# Clean socket files
echo "Cleaning socket files..."
sudo rm -f /tmp/feo_listener*.socket 2>/dev/null || true

# Clean any leftover lock files
echo "Cleaning lock files..."
sudo rm -f /tmp/.feo* 2>/dev/null || true

echo "Cleanup complete. System ready for restart."