# DDS Console Apps with REST API Endpoints

## 📋 Overview

All three DDS console applications now have REST API endpoints that your React frontend can call to get the latest data from each driving mode.

## 🚀 Applications & Endpoints

### 1. Manual Driving App
- **DDS Subscriber**: Receives `ManualCarData` from topic `"ManualCarData"`
- **REST API**: `http://localhost:9081/data`
- **Binary**: `/home/acrn/new_ak/dds_console_app/dds_manual_app/target/debug/dds_manual_app`

### 2. Emergency Driving App  
- **DDS Subscriber**: Receives `EmergencyModeData` from topic `"EmergencyModeData"`
- **REST API**: `http://localhost:9082/data`
- **Binary**: `/home/acrn/new_ak/dds_console_app/dds_emergency_app/target/debug/dds_emergency_app`

### 3. Autonomous Driving App
- **DDS Subscriber**: Receives `AutonomousCarData` from topic `"AutonomousCarData"`
- **REST API**: `http://localhost:9083/data`
- **Binary**: `/home/acrn/new_ak/dds_console_app/dds_autonomous_app/target/debug/dds_autonomous_app`

## 📊 Data Structures

### ManualCarData (Port 9081)
```json
{
  "vehicle_speed": 65.5,
  "steering_angle": 12.3,
  "brake_force": 15.0,
  "acceleration": 2.1,
  "weather_condition": "clear",
  "road_condition": "dry",
  "driver_alertness": true,
  "throttle_position": 45.0,
  "timestamp": 1699123456789,
  "is_valid": true
}
```

### EmergencyModeData (Port 9082)
```json
{
  "vehicle_speed": 45.0,
  "steering_angle": -25.0,
  "brake_force": 85.0,
  "obstacle_detected": true,
  "obstacle_distance": 15.5,
  "collision_risk": 75.0,
  "stability_control": true,
  "traffic_signal": "red",
  "seatbelt_tightened": true,
  "emergency_lights": true,
  "emergency_type": "collision_avoidance",
  "emergency_brake_force": 100.0,
  "airbag_ready": true,
  "timestamp": 1699123456789,
  "is_valid": true
}
```

### AutonomousCarData (Port 9083)
```json
{
  "vehicle_speed": 55.0,
  "lane_position": 1.2,
  "obstacle_detected": false,
  "obstacle_distance": 100.0,
  "traffic_signal": "green",
  "steering_angle": 5.0,
  "brake_force": 0.0,
  "acceleration": 1.5,
  "weather_condition": "clear",
  "road_condition": "dry",
  "timestamp": 1699123456789,
  "is_valid": true
}
```

## 🛠️ How to Use

### 1. Start a DDS App
```bash
# Manual mode
sudo /home/acrn/new_ak/dds_console_app/dds_manual_app/target/debug/dds_manual_app

# Emergency mode  
sudo /home/acrn/new_ak/dds_console_app/dds_emergency_app/target/debug/dds_emergency_app

# Autonomous mode
sudo /home/acrn/new_ak/dds_console_app/dds_autonomous_app/target/debug/dds_autonomous_app
```

### 2. Call REST API from Frontend
```javascript
// React example
const fetchManualData = async () => {
  try {
    const response = await fetch('http://localhost:9081/data');
    const data = await response.json();
    console.log('Manual car data:', data);
    return data;
  } catch (error) {
    console.error('Error fetching manual data:', error);
  }
};

const fetchEmergencyData = async () => {
  const response = await fetch('http://localhost:9082/data');
  return await response.json();
};

const fetchAutonomousData = async () => {
  const response = await fetch('http://localhost:9083/data');
  return await response.json();
};
```

### 3. Test REST APIs
```bash
# Test manual data endpoint
curl http://localhost:9081/data

# Test emergency data endpoint  
curl http://localhost:9082/data

# Test autonomous data endpoint
curl http://localhost:9083/data
```

## 🔧 Integration with Systemd Scripts

### Updated systemd scripts should include both DDS subscriber and React dashboard:

#### `/usr/local/bin/start_manual.sh`
```bash
#!/bin/bash
# Stop other driving modes if running
systemctl stop emergency_driving.service
systemctl stop autonomous_driving.service

# Start the DDS manual app (with REST API on port 9081)
nohup /home/acrn/new_ak/dds_console_app/dds_manual_app/target/debug/dds_manual_app > /tmp/manual_dds.log 2>&1 &

# Start the React dashboard
cd /path/to/your/react/dashboard
nohup npm run dev > /tmp/manual_dashboard.log 2>&1 &

echo "Manual driving mode started - DDS API: http://localhost:9081/data"
```

#### `/usr/local/bin/start_emergency.sh`
```bash
#!/bin/bash
# Stop other driving modes if running  
systemctl stop manual_driving.service
systemctl stop autonomous_driving.service

# Start the DDS emergency app (with REST API on port 9082)
nohup /home/acrn/new_ak/dds_console_app/dds_emergency_app/target/debug/dds_emergency_app > /tmp/emergency_dds.log 2>&1 &

# Start the React dashboard
cd /path/to/your/react/dashboard  
nohup npm run dev > /tmp/emergency_dashboard.log 2>&1 &

echo "Emergency driving mode started - DDS API: http://localhost:9082/data"
```

#### `/usr/local/bin/start_autonomous.sh`
```bash
#!/bin/bash
# Stop other driving modes if running
systemctl stop manual_driving.service  
systemctl stop emergency_driving.service

# Start the DDS autonomous app (with REST API on port 9083)
nohup /home/acrn/new_ak/dds_console_app/dds_autonomous_app/target/debug/dds_autonomous_app > /tmp/autonomous_dds.log 2>&1 &

# Start the React dashboard
cd /path/to/your/react/dashboard
nohup npm run dev > /tmp/autonomous_dashboard.log 2>&1 &

echo "Autonomous driving mode started - DDS API: http://localhost:9083/data"
```

## 📝 CORS Configuration

All REST APIs are configured with proper CORS support:
- **GET** `/data` - Returns latest DDS data with CORS headers
- **OPTIONS** `/data` - Handles CORS preflight requests
- Headers set:
  - `Access-Control-Allow-Origin: *`
  - `Access-Control-Allow-Methods: GET, POST, OPTIONS`
  - `Access-Control-Allow-Headers: Content-Type`

This allows your React frontend to call the APIs from any domain without CORS errors.

## 🏗️ Architecture

Each app now runs with **concurrent tasks**:
1. **REST API Server Task**: Handles HTTP requests on dedicated port
2. **DDS Subscriber Task**: Receives data from DDS publisher and updates shared state

This ensures:
- Regular DDS data reception (no blocking)
- Responsive REST API (independent of DDS operations)
- Real-time data updates in shared state

## 🐛 Error Responses

If no DDS data has been received yet, the APIs return:
```json
{
  "error": "No [manual/emergency/autonomous] data available yet"
}
```

## 🔍 Logs

Each app will print:
- DDS subscription status (waiting for publisher, data received)
- REST API server startup: `"[Mode] Data REST API running on http://localhost:[port]/data"`
- Received DDS data in console output

## 🎯 Summary

**When Pullpiri launches any driving service:**
1. **DDS App starts** → Subscribes to DDS topic → Exposes REST API
2. **React Dashboard starts** → Calls REST API → Displays live vehicle data  
3. **Frontend gets real-time data** → Updates dashboard continuously

**Your React dashboard can now fetch live vehicle data from whichever driving mode is currently active!**