
# File-Based Driver Distraction Event System

This document describes the file-based driver distraction event system integrated with the Pullpiri orchestration platform. When Pullpiri YAML conditions are matched, the system launches Rust containers that write messages to shared files, and a Python backend monitors these files to expose REST endpoints for UI polling.

## File Structure (updated_demo)

```
updated_demo/
├── build_file_system.sh
├── Driver_Distraction_System_README.md
├── test_file_system.sh
├── container-app/
│   └── file-message-writer/
│       ├── Cargo.toml
│       ├── Dockerfile
│       └── src/
│           └── main.rs
├── file_backend_app/
│   ├── file_monitor_backend.py
│   └── requirements.txt
├── pullpiri/
│   └── ... (Pullpiri source, scenarios, docs, etc.)
```

## Overview

The Rust DDS Driver Distraction Event System consists of:

1. **Rust DDS Message Sender Container**: Triggered by Pullpiri when YAML conditions match - sends DDS events and exits
2. **Rust DDS Backend Receiver**: Continuously subscribes to DDS messages and exposes REST endpoints for UI polling (fixed port 8081)

This architecture uses pure Rust DDS communication (dust_dds library) with simplified state management for high performance event-driven messaging triggered by Pullpiri scenario conditions.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Pullpiri Orchestration                      │
│                  (Monitors DDS Conditions)                     │
└─────────────────────────┬───────────────────────────────────────┘
                          │ YAML Condition Matched
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│              Rust DDS Message Sender Container                 │
│                   (dds-message-sender-rust)                    │
│                                                                 │
│  Triggered by: driver-distraction-5sec.yaml OR                 │
│                driver-distraction-10sec.yaml                   │
│                                                                 │
│  Actions:                                                       │
│  1. Reads environment variables for configuration               │
│  2. Creates DriverDistractionEvent struct                      │
│  3. Publishes to DDS topic "DriverDistractionEvents"           │
│  4. Exits after successful message delivery                     │
│                                                                 │
│  DDS Configuration:                                             │
│  - Domain ID: 100                                               │
│  - Reliable QoS with TransientLocal durability                 │
│  - Uses dust_dds library                                        │
└─────────────────────────┬───────────────────────────────────────┘
                          │ DDS Messages (Domain 100)
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│               Rust DDS Backend Receiver                        │
│                (dds-backend-receiver-rust)                     │
│        REST API always runs on 127.0.0.1:8081                 │
│                                                                 │
│  DDS Subscriber:                                                │
│  - Subscribes to "DriverDistractionEvents" topic               │
│  - Domain ID: 100, BestEffort QoS for compatibility            │
│  - Simple Arc<Mutex> state management                          │
│  - Stores latest DDS message in memory                         │
│                                                                 │
│  REST API Endpoints:                                            │
│  - GET /data                - Latest DDS message received      │
│  - GET /health              - Health check endpoint            │
│                                                                 │
│  Architecture: Simplified VehicleData-style pattern            │
└─────────────────────────┬───────────────────────────────────────┘
                          │ HTTP REST API
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Dashboard UI                               │
│              (Polls REST endpoints every 1-2s)                 │
└─────────────────────────────────────────────────────────────────┘
```

## Components

### 1. Rust DDS Message Sender Container

#### DDS Message Sender (Rust)
- **Location**: `container-apps/dds-message-sender/`
- **Image**: `dds-message-sender:latest`
- **Language**: Rust with dust_dds library (same as your existing DDS apps)
- **Trigger**: Pullpiri YAML condition matches
- **Execution**: Run-once container (sends message and exits)

**Features:**
- Event-driven DDS message publishing
- Configurable via environment variables
- Reliable DDS communication with TransientLocal durability
- Automatic publisher discovery and message delivery
- Fast execution and clean exit
- Uses same DDS patterns as `dds_autonomous_app`

**DDS Message Structure:**
```rust
#[derive(DdsType, Clone, Debug, Serialize, Deserialize)]
pub struct DriverDistractionEvent {
    pub timestamp: String,
    pub message_type: String,
    pub scenario_name: String,
    pub content: String,
    pub severity: String,
    pub source: String,
    pub event_id: String,
    pub distraction_duration: f64,
    pub threshold_exceeded: bool,
}
```

**Configuration (Environment Variables):**
- `MESSAGE_TYPE`: Type of distraction event
- `SCENARIO_NAME`: Pullpiri scenario identifier  
- `DDS_TOPIC`: DDS topic name (default: "DriverDistractionEvents")
- `MESSAGE_CONTENT`: Human-readable event description
- `SEVERITY`: Event severity level (warning/critical)
- `THRESHOLD_VALUE`: Duration threshold that was exceeded

### 2. Rust DDS Backend Receiver Application

#### DDS Backend Receiver (Rust)
- **Location**: `dds_backend_app/`
- **Image**: `dds-backend-receiver-rust:latest` 
- **REST API**: Always runs on `127.0.0.1:8081`
- **Language**: Rust with dust_dds + warp web framework
- **Function**: Subscribes to DDS messages and exposes REST API

**Features:**
- Real-time DDS message subscription using dust_dds
- Simplified Arc<Mutex<Option<DriverDistractionEvent>>> state management
- Multi-threaded architecture (DDS subscriber + REST server)
- REST API for UI polling with CORS support
- Latest message storage (no complex history tracking)
- Health monitoring and status reporting
- High performance Rust implementation with VehicleData-style architecture

### 2. Pullpiri Integration

#### YAML Scenario Files
Located in `pullpiri/examples/resources/`:

1. **driver-distraction-5sec.yaml**
   - Scenario trigger: DDS condition `gt 5 seconds`
   - Container deployment with port mapping
   - Volume mounts for alert data persistence

2. **driver-distraction-10sec.yaml**
   - Scenario trigger: DDS condition `gt 10 seconds`
   - Emergency container with vehicle control capabilities
   - Enhanced monitoring and safety protocols

## Installation and Setup

### Prerequisites
- Podman container runtime
- Pullpiri orchestration platform
- Python 3.9+ (for container images)
- Flask and requests Python packages

### Building Container Images

1. **Make build script executable:**
   ```bash
  chmod +x build_dds_system.sh
   ```

2. **Build both container images:**
   ```bash
  ./build_dds_system.sh
   ```

This script will:
- Build `driver-distraction-5sec:latest` image
- Build `driver-distraction-10sec:latest` image
- Display available images
- Provide next steps and testing commands

### Deployment via Pullpiri

1. **Deploy 5-second alert monitor:**
   ```bash
   curl -X POST http://rhivos-ip:8080/api/scenarios \
        -H "Content-Type: application/yaml" \
        -d @pullpiri/examples/resources/driver-distraction-5sec.yaml
   ```

2. **Deploy 10-second emergency manager:**
   ```bash
   curl -X POST http://rhivos-ip:8080/api/scenarios \
        -H "Content-Type: application/yaml" \
        -d @pullpiri/examples/resources/driver-distraction-10sec.yaml
   ```

## API Reference

### DDS Backend Receiver (Port 8081)

#### GET /data
Returns the latest DDS message received by the backend.

**Response (when message available):**
```json
{
  "timestamp": "2025-11-24T10:30:00.000Z",
  "message_type": "driver_distraction",
  "scenario_name": "driver-distraction-5sec",
  "content": "Driver distraction detected for 5 seconds",
  "severity": "warning",
  "source": "pullpiri-container",
  "event_id": "evt_123456",
  "distraction_duration": 5.0,
  "threshold_exceeded": true
}
```

**Response (when no message available):**
```json
{
  "error": "No DDS messages received yet"
}
```

#### GET /health
Returns health check for the backend service.

**Response:**
```json
{
  "status": "healthy",
  "service": "dds-backend-receiver"
}
```

## Usage Examples

### Dashboard UI Integration

The dashboard UI should poll the DDS backend endpoint regularly (recommended: every 1-2s) for real-time updates:

```javascript
// Poll DDS backend for latest driver distraction events
async function updateDistractionStatus() {
    try {
        const response = await fetch('http://localhost:8081/data');
        const data = await response.json();
        
        if (data.error) {
            console.log('No DDS messages received yet');
            updateUI({ status: 'waiting', message: 'Waiting for distraction events...' });
        } else {
            console.log('Latest distraction event:', data);
            updateUI({
                status: 'active',
                scenario: data.scenario_name,
                severity: data.severity,
                duration: data.distraction_duration,
                content: data.content,
                timestamp: data.timestamp
            });
        }
    } catch (error) {
        console.error('DDS backend connection error:', error);
        updateUI({ status: 'error', message: 'Backend connection failed' });
    }
}

// Check backend health
async function checkBackendHealth() {
    try {
        const response = await fetch('http://localhost:8081/health');
        const health = await response.json();
        console.log('Backend health:', health.status);
        return health.status === 'healthy';
    } catch (error) {
        console.error('Health check failed:', error);
        return false;
    }
}

// Set up polling
setInterval(updateDistractionStatus, 2000);
setInterval(checkBackendHealth, 10000);
```

### Testing Scenarios

#### Build and Test the Complete System:
```bash
# Build the DDS system containers
./build_dds_system.sh

# Start the backend receiver (in terminal 1)
cd dds_backend_app
sudo cargo run

# Send test DDS message (in terminal 2)
cd container-app/dds-message-sender
podman run --rm --network=host \
  -e MESSAGE_TYPE="driver_distraction" \
  -e SCENARIO_NAME="test-scenario" \
  -e MESSAGE_CONTENT="Test distraction event" \
  -e SEVERITY="warning" \
  dds-message-sender:latest

# Check if message was received via REST API
curl http://localhost:8081/data

# Check backend health
curl http://localhost:8081/health
```

#### Test Container Communication:
```bash
# Run DDS sender container with debug output
podman run --rm --network=host \
  -e MESSAGE_TYPE="driver_distraction" \
  -e SCENARIO_NAME="driver-distraction-5sec" \
  -e MESSAGE_CONTENT="Keep Your Eyes On the Road Ahead" \
  -e SEVERITY="warning" \
  -e THRESHOLD_VALUE="5.0" \
  dds-message-sender:latest

#OR
podman run --rm --network=host \
  -e MESSAGE_TYPE="driver_distraction" \
  -e SCENARIO_NAME="driver-distraction-10sec" \
  -e MESSAGE_CONTENT="Moving Car to Safe Area" \
  -e SEVERITY="critical" \
  -e THRESHOLD_VALUE="10.0" \
  dds-message-sender:latest

# Poll backend for new messages
curl http://localhost:8081/data
```

## Integration with Existing Systems

### DDS Integration
The system is designed to integrate with DDS (Data Distribution Service) for real-time distraction data:

- **DDS Topic**: `DriverDistraction`
- **Data Source**: `DistractionMonitor`
- **Condition Monitoring**: Automatic threshold-based triggering

### Pullpiri Orchestration
- **ActionController**: Manages scenario deployment and lifecycle
- **NodeAgent**: Handles container deployment on target nodes
- **BlueChi**: Coordinates multi-node systemd service management
- **Quadlet**: Generates systemd services from container specifications

### Vehicle Control Systems
- **LKAS Integration**: Lane Keeping Assist System activation
- **Safe Area Navigation**: Autonomous movement to safe stopping areas
- **Emergency Protocols**: Hazard light activation, gradual deceleration
- **Manual Override**: Safe resumption of driver control


### Log Monitoring
Monitor system components for detailed status information:
```bash
# View DDS backend receiver logs
cd dds_backend_app
sudo cargo run  # View real-time logs

# View DDS sender container logs (if running as container)
sudo podman logs dds-message-sender

# Check container status
sudo podman ps -a
```

### Common Issues

1. **DDS messages not received**: Check QoS alignment (both use BestEffort), ensure host networking
2. **REST API not responding**: Verify backend is running on port 8081, check firewall
3. **Container not starting**: Check Rust 1.83 availability, build images with correct dependencies
4. **Discovery issues**: Ensure both sender and receiver use domain 100, check multicast networking

### Debugging Commands
```bash
# Check if backend is listening
sudo netstat -tlnp | grep 8081

# Test backend availability
curl --connect-timeout 5 http://localhost:8081/health

# Reset backend state (restart the backend)
cd dds_backend_app
sudo cargo clean && sudo cargo run
```

## File Structure

See the top of this README for the updated_demo directory structure.

## Security Considerations

- REST APIs run on localhost by default
- No authentication required for demo purposes
- Container isolation provides security boundaries
- Vehicle control commands require safety checks
- Emergency protocols cannot be overridden during active distraction

## Future Enhancements

- **Machine Learning Integration**: Advanced distraction detection algorithms
- **Multi-Camera Support**: 360-degree driver monitoring
- **Biometric Integration**: Heart rate and stress level monitoring
- **Predictive Analytics**: Proactive distraction prevention
- **Fleet Management**: Multi-vehicle monitoring dashboard
- **Compliance Reporting**: Driver behavior analytics and reporting

## Support and Maintenance

For issues or questions regarding the Driver Distraction Monitoring System:

1. Check container health status
2. Review API logs for error messages
3. Verify Pullpiri scenario deployment status
4. Test with simulation endpoints
5. Reset system state if needed

The system is designed for high availability and automatic recovery from transient failures.