# File-Based Driver Distraction Event System

This document describes the file-based driver distraction event system integrated with the Pullpiri orchestration platform. When Pullpiri YAML conditions are matched, the system launches Rust containers that write messages to shared files, and a Python backend monitors these files to expose REST endpoints for UI polling.

## 🏗️ System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Pullpiri Orchestration                      │
│              (Monitors DistractionMonitor.dms_val)             │
└─────────────────────────┬───────────────────────────────────────┘
                          │ YAML Condition Matched (gt 5 or gt 10)
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│              Rust File Message Writer Container                │
│                   (file-message-writer:latest)                 │
│                                                                 │
│  Triggered by: driver-distraction-5sec.yaml OR                 │
│                driver-distraction-10sec.yaml                   │
│                                                                 │
│  Actions:                                                       │
│  1. Reads environment variables for configuration               │
│  2. Creates DriverDistractionEvent JSON                        │
│  3. Writes to /data/driver_distraction_messages.json           │
│  4. Saves history to /data/history/message_{id}.json           │
│  5. Exits after successful file write                          │
└─────────────────────────┬───────────────────────────────────────┘
                          │ File System (Shared Volume)
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                Python File Monitor Backend                     │
│              (file_monitor_backend.py)                         │
│        REST API always runs on 127.0.0.1:8081                 │
│                                                                 │
│  File Monitor:                                                  │
│  - Monitors /tmp/driver_distraction/ directory                 │
│  - Detects file changes every 500ms                            │
│  - Auto-expires messages after 3 seconds                      │
│  - Thread-safe message state management                        │
│                                                                 │
│  REST API Endpoints:                                            │
│  - GET /data                - Latest file message received     │
│  - GET /health              - Health check with file status    │
│  - GET /status              - Detailed system information      │
│  - POST /clear              - Manual message clear (testing)   │
└─────────────────────────┬───────────────────────────────────────┘
                          │ HTTP REST API
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Dashboard UI                               │
│       (Polls REST endpoints every 1-2s for real-time UI)      │
└─────────────────────────────────────────────────────────────────┘
```

## 📁 File Structure

```
updated_demo/
├── build_file_system.sh              # Build script for file-based system
├── test_file_system.sh               # Test script for validation
├── File_Based_System_README.md       # This documentation
├── container-app/
│   └── file-message-writer/          # Rust container for file writing
│       ├── Cargo.toml
│       ├── Dockerfile
│       └── src/
│           └── main.rs
├── file_backend_app/                 # Python file monitoring backend
│   ├── file_monitor_backend.py       # Main Python backend script
│   └── requirements.txt              # Python dependencies
└── pullpiri/
    └── examples/
        └── resources/
            ├── driver-distraction-5sec.yaml   # 5-second scenario
            └── driver-distraction-10sec.yaml  # 10-second scenario
```

## 🎯 Key Features

### ✅ **No DDS Required**
- Pure file-based messaging system
- No network dependencies or discovery issues
- Simple shared volume mounting

### ⚡ **Real-Time File Monitoring**
- Python backend monitors file changes every 500ms
- Instant detection of new messages from containers
- Thread-safe state management

### ⏰ **Automatic Message Expiration**
- Messages automatically expire after 3 seconds
- Ensures UI alarms turn off when distraction stops
- Configurable timeout settings

### 📦 **Container Integration**
- Rust containers write JSON messages to shared volumes
- Clean container exit after message write
- Volume mounting handles file persistence

### 🌐 **REST API Compatibility**
- Same endpoints as DDS version (`/data`, `/health`, `/status`)
- UI code requires no changes
- CORS support for web dashboards

## 🚀 Installation and Setup

### Prerequisites

```bash
# System requirements
- Podman container runtime
- Python 3.9+ with pip
- Rust 1.83+ (for building containers)
- Pullpiri orchestration platform
```

### Step 1: Build the File-Based System

```bash
# Make build script executable
chmod +x build_file_system.sh

# Build the file message writer container
./build_file_system.sh
```

**Expected Output:**
```
🔧 Building File-Based Driver Distraction System...
==================================================
ℹ️  Building file-message-writer container...
✅ file-message-writer:latest built successfully
✅ 🎉 File-based system containers built successfully!
```

### Step 2: Setup Python Backend

```bash
# Navigate to backend directory
cd file_backend_app

# Install Python dependencies
pip3 install -r requirements.txt

# Create shared data directory
sudo mkdir -p /tmp/driver_distraction
sudo chmod 777 /tmp/driver_distraction
```

### Step 3: Start the Backend

```bash
# Start the file monitoring backend
python3 file_monitor_backend.py
```

**Expected Output:**
```
🚀 Starting File Monitoring Backend
📁 Monitoring file: /tmp/driver_distraction/driver_distraction_messages.json
⏱️  Check interval: 0.5s
⏰ Message timeout: 3.0s
🌐 REST API will be available on http://127.0.0.1:8081
📁 Started monitoring file: /tmp/driver_distraction/driver_distraction_messages.json
 * Running on http://127.0.0.1:8081
```

### Step 4: Test the System

```bash
# Run comprehensive test
./test_file_system.sh
```

## 📋 Message Format

### Container Environment Variables
```bash
MESSAGE_TYPE="driver_distraction_5sec_alert"    # Event type
SCENARIO_NAME="driver_distraction_5sec"         # Scenario identifier
MESSAGE_CONTENT="Keep Your Eyes On the Road Ahead"  # User message
SEVERITY="warning"                              # warning/critical
THRESHOLD_VALUE="5.0"                          # Threshold exceeded
```

### JSON File Structure
```json
{
  "timestamp": "2025-11-25T10:30:00.000Z",
  "message_type": "driver_distraction_5sec_alert",
  "scenario_name": "driver_distraction_5sec",
  "content": "Keep Your Eyes On the Road Ahead",
  "severity": "warning",
  "source": "pullpiri_rust_container",
  "event_id": "driver_distraction_5sec_1732512600",
  "distraction_duration": 5.0,
  "threshold_exceeded": true
}
```

## 🌐 REST API Reference

### GET /data
Returns the latest message from the monitored file.

**Response (when message available):**
```json
{
  "timestamp": "2025-11-25T10:30:00.000Z",
  "message_type": "driver_distraction_5sec_alert",
  "scenario_name": "driver_distraction_5sec",
  "content": "Keep Your Eyes On the Road Ahead",
  "severity": "warning",
  "source": "pullpiri_rust_container",
  "event_id": "driver_distraction_5sec_1732512600",
  "distraction_duration": 5.0,
  "threshold_exceeded": true
}
```

**Response (when no message available or expired):**
```json
{
  "error": "No messages received yet"
}
```

### GET /health
Returns backend health status with file monitoring information.

**Response:**
```json
{
  "status": "healthy",
  "service": "file-monitoring-backend",
  "file_path": "/tmp/driver_distraction/driver_distraction_messages.json",
  "file_exists": true,
  "data_available": true,
  "data_age_seconds": 1.2,
  "timeout_threshold_seconds": 3.0,
  "last_check": "2025-11-25T10:30:01.000Z"
}
```

### GET /status
Returns detailed system status and configuration.

**Response:**
```json
{
  "monitoring": true,
  "file_path": "/tmp/driver_distraction/driver_distraction_messages.json",
  "file_exists": true,
  "current_message": { /* latest message object or null */ },
  "check_interval": 0.5,
  "message_timeout": 3.0,
  "system_time": "2025-11-25T10:30:01.000Z"
}
```

### POST /clear
Manually clears the current message (useful for testing).

**Response:**
```json
{
  "status": "cleared",
  "message": "Current message cleared successfully"
}
```

## 🧪 Testing Scenarios

### Manual Container Test
```bash
# Test 5-second distraction message
sudo podman run --rm \
    -v /tmp/driver_distraction:/data \
    -e MESSAGE_TYPE="driver_distraction" \
    -e SCENARIO_NAME="test-5sec" \
    -e MESSAGE_CONTENT="Keep Your Eyes On the Road Ahead" \
    -e SEVERITY="warning" \
    -e THRESHOLD_VALUE="5.0" \
    file-message-writer:latest

# Check if file was created
cat /tmp/driver_distraction/driver_distraction_messages.json

# Test API response
curl http://localhost:8081/data
```

### Test 10-Second Emergency Message
```bash
# Test 10-second emergency message
sudo podman run --rm \
    -v /tmp/driver_distraction:/data \
    -e MESSAGE_TYPE="driver_distraction" \
    -e SCENARIO_NAME="test-10sec" \
    -e MESSAGE_CONTENT="Emergency: Moving Car to Safe Area" \
    -e SEVERITY="critical" \
    -e THRESHOLD_VALUE="10.0" \
    file-message-writer:latest

# Check API response
curl http://localhost:8081/data
```

### Test Message Timeout
```bash
# Send a message, then wait for timeout
sudo podman run --rm -v /tmp/driver_distraction:/data \
    -e MESSAGE_CONTENT="Test timeout message" \
    file-message-writer:latest

# Immediately check API
curl http://localhost:8081/data  # Should return message

# Wait 4 seconds and check again
sleep 4
curl http://localhost:8081/data  # Should return "No messages received yet"
```

## 🔗 Pullpiri Integration

### YAML Configuration Changes

The system uses updated YAML files with volume mounts instead of DDS configuration:

**driver-distraction-5sec.yaml:**
```yaml
spec:
  hostNetwork: true
  containers:
    - name: file-writer-5sec
      image: file-message-writer:latest
      env:
        - name: MESSAGE_CONTENT
          value: "Keep Your Eyes On the Road Ahead"
        - name: SEVERITY
          value: "warning"
      volumeMounts:
        - name: message-data
          mountPath: /data
  volumes:
    - name: message-data
      hostPath:
        path: /tmp/driver_distraction
        type: DirectoryOrCreate
```

### Deployment Commands
```bash
# Deploy 5-second scenario
curl -X POST http://pullpiri-server:47099/api/artifact \
     -H 'Content-Type: text/plain' \
     -d @pullpiri/examples/resources/driver-distraction-5sec.yaml

# Deploy 10-second scenario  
curl -X POST http://pullpiri-server:47099/api/artifact \
     -H 'Content-Type: text/plain' \
     -d @pullpiri/examples/resources/driver-distraction-10sec.yaml
```

## 🎮 Dashboard UI Integration

### JavaScript Example
```javascript
// File-based system uses same API as DDS version
async function updateDistractionStatus() {
    try {
        const response = await fetch('http://localhost:8081/data');
        const data = await response.json();
        
        if (data.error) {
            // No active distraction - clear UI
            updateUI({ 
                status: 'safe', 
                message: 'No active distraction detected' 
            });
        } else {
            // Active distraction - show alert
            updateUI({
                status: 'alert',
                scenario: data.scenario_name,
                severity: data.severity,
                message: data.content,
                duration: data.distraction_duration,
                timestamp: data.timestamp
            });
        }
    } catch (error) {
        console.error('Backend connection error:', error);
        updateUI({ 
            status: 'error', 
            message: 'Backend connection failed' 
        });
    }
}

// Poll every 1 second for real-time updates
setInterval(updateDistractionStatus, 1000);
```

### React Hook Example
```javascript
import { useState, useEffect } from 'react';

function useDistractionStatus() {
    const [status, setStatus] = useState(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const fetchStatus = async () => {
            try {
                const response = await fetch('http://localhost:8081/data');
                const data = await response.json();
                setStatus(data.error ? null : data);
                setLoading(false);
            } catch (error) {
                console.error('Fetch error:', error);
                setLoading(false);
            }
        };

        // Initial fetch
        fetchStatus();
        
        // Poll every 1 second
        const interval = setInterval(fetchStatus, 1000);
        return () => clearInterval(interval);
    }, []);

    return { status, loading };
}
```

## 🔧 Troubleshooting

### Common Issues

1. **Backend not responding**
   ```bash
   # Check if backend is running
   curl http://localhost:8081/health
   
   # Check Python process
   ps aux | grep file_monitor_backend
   ```

2. **File not being created**
   ```bash
   # Check directory permissions
   ls -la /tmp/driver_distraction/
   
   # Check container logs
   sudo podman logs <container_id>
   ```

3. **Messages not expiring**
   ```bash
   # Check backend logs for timeout messages
   # Look for "⏰ Message expired" in output
   
   # Manually clear message for testing
   curl -X POST http://localhost:8081/clear
   ```

4. **Container permission issues**
   ```bash
   # Ensure data directory is writable
   sudo chmod 777 /tmp/driver_distraction/
   
   # Check SELinux context (if applicable)
   ls -Z /tmp/driver_distraction/
   ```

### Debug Commands

```bash
# Check file monitoring in real-time
watch -n 1 'ls -la /tmp/driver_distraction/'

# Monitor backend logs
python3 file_monitor_backend.py  # Watch console output

# Test file write manually
echo '{"test": "message"}' > /tmp/driver_distraction/driver_distraction_messages.json

# Check API responses
curl -s http://localhost:8081/status | python3 -m json.tool
```

## 📊 Performance Characteristics

### File System Performance
- **File Check Frequency**: 500ms (configurable)
- **Message Timeout**: 3 seconds (configurable)
- **File Size**: ~300-500 bytes per message
- **History Retention**: All messages saved to history/ subdirectory

### Memory Usage
- **Backend Memory**: ~20-30MB Python process
- **Container Memory**: ~5-10MB per execution
- **File Storage**: Minimal (KB per message)

### Throughput
- **Message Processing**: Near-instantaneous file write
- **API Response Time**: <10ms for cached data
- **Container Startup**: ~1-2 seconds from trigger to file write

## 🔒 Security Considerations

### File Permissions
```bash
# Recommended directory setup
sudo mkdir -p /tmp/driver_distraction/history
sudo chmod 755 /tmp/driver_distraction
sudo chmod 755 /tmp/driver_distraction/history

# Files are created with standard permissions
# Backend runs as regular user (not root)
```

### API Security
- REST API binds to localhost (127.0.0.1) only
- No authentication required (demo/internal use)
- CORS enabled for web dashboard integration
- No sensitive data in message content

### Container Security
- Containers run as non-root user
- Minimal runtime image (Debian slim)
- Read-only container filesystem except /data mount
- Container exits immediately after task completion

## 🚀 Production Deployment

### Systemd Service Example
```ini
# /etc/systemd/system/distraction-monitor.service
[Unit]
Description=File-Based Distraction Monitor Backend
After=network.target

[Service]
Type=simple
User=distraction
Group=distraction
WorkingDirectory=/opt/distraction-system/file_backend_app
ExecStart=/usr/bin/python3 file_monitor_backend.py
Restart=always
RestartSec=5
Environment=PYTHONPATH=/opt/distraction-system

[Install]
WantedBy=multi-user.target
```

### Docker Compose Alternative
```yaml
version: '3.8'
services:
  distraction-backend:
    build: ./file_backend_app
    ports:
      - "127.0.0.1:8081:8081"
    volumes:
      - /tmp/driver_distraction:/data
    restart: unless-stopped
    environment:
      - MESSAGE_TIMEOUT=3.0
      - CHECK_INTERVAL=0.5
```

## 📈 Future Enhancements

### Planned Features
- **Multiple Message Types**: Support for different event categories
- **Message Prioritization**: Critical messages override warnings
- **File Rotation**: Automatic cleanup of old history files
- **WebSocket Support**: Real-time push notifications to UI
- **Message Persistence**: Optional database storage for analytics

### Configuration Options
```python
# Configurable settings in backend
MESSAGE_TIMEOUT = float(os.getenv('MESSAGE_TIMEOUT', 3.0))
CHECK_INTERVAL = float(os.getenv('CHECK_INTERVAL', 0.5))
MAX_HISTORY_FILES = int(os.getenv('MAX_HISTORY_FILES', 1000))
ENABLE_WEBSOCKETS = os.getenv('ENABLE_WEBSOCKETS', 'false').lower() == 'true'
```

---

## 📞 Support

For issues with the file-based driver distraction system:

1. **Check system status**: `curl http://localhost:8081/status`
2. **Review backend logs**: Look for error messages in console output
3. **Verify file permissions**: Ensure containers can write to shared directory
4. **Test manually**: Use test scripts to isolate issues
5. **Check Pullpiri integration**: Verify YAML deployment and container execution

The file-based system provides a robust, simple alternative to DDS messaging with the same UI compatibility and real-time performance characteristics.