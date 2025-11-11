
# Mini-ADAS Example: Activity Relations & Process Overview

## 1. Overview & Use Case

This Mini-ADAS (Advanced Driver Assistance System) example demonstrates a complete autonomous driving pipeline using the FEO framework. The system simulates a vehicle with multiple sensors, processes the data to understand the environment, makes driving decisions, and controls the vehicle accordingly.

### Real-World Scenario
The system simulates a car driving on a road with:
- **Camera**: Detects people, cars, and obstacles
- **Radar**: Measures distance to obstacles with error margins
- **Neural Network**: Fuses sensor data to create a complete scene understanding
- **Decision Engine**: Determines appropriate driving mode based on safety conditions
- **Control Systems**: Executes steering, braking, and acceleration commands

### Data Flow Pipeline
```
Sensors → Data Fusion → Scene Analysis → Mode Decision → Vehicle Control → DDS Publication
```

The system operates in three distinct modes:
1. **Autonomous Mode**: System takes control (safe conditions, clear roads)
2. **Manual Mode**: Driver controls the vehicle (complex traffic scenarios)
3. **Emergency Mode**: Emergency braking and safety systems activated (immediate danger)

## 2. Process Architecture

The system is split into two processes for safety-critical separation:

### Primary Process (`adas_primary`)
**Purpose**: Perception, decision-making, and high-level control

**Agent 100 (Workers 40, 41)**: Sensor simulation
- **Worker 40**:
  - **Camera (Activity A0)**: Simulates front camera sensor
    - Generates pseudo-random data: number of people, cars, obstacle distance
    - Publishes CameraImage messages via shared memory
- **Worker 41**:
  - **Radar (Activity A1)**: Simulates front radar sensor  
    - Generates distance measurements with error margins
    - Publishes RadarScan messages via shared memory

**Agent 101 (Worker 42)**: Data fusion and decision-making
- **Worker 42** (All activities on single worker):
  - **NeuralNet (Activity A2)**: Data fusion engine
    - Combines camera and radar data into unified scene understanding
    - Calculates final obstacle distance, lane distances
    - Publishes Scene messages to all dependent activities
  - **EnvironmentRenderer (Activity A3)**: Visualization
    - Consumes scene data for display/debugging purposes
  - **CarModeCalculator (Activity A4)**: Driving mode decision engine
    - Analyzes scene conditions (obstacle distance, people count, car count)
    - Determines appropriate driving mode using safety thresholds
    - Publishes CarData with current mode decision
  - **CarDataPublisher (Activity A5)**: DDS integration gateway
    - Receives internal CarData messages
    - Publishes to external DDS topic "CarData" for pullpiri integration
    - Enables external systems to monitor current driving mode
  - **AutonomousModePublisher (Activity A6)**: Autonomous control publisher
    - Active only when in autonomous mode
    - Publishes comprehensive vehicle control data via DDS
    - Includes speed, steering, braking, traffic signals, timestamps
  - **ManualModePublisher (Activity A7)**: Manual control publisher
    - Active only when in manual mode
    - Publishes driver-assisted control data via DDS
    - Includes throttle position, driver alertness, manual inputs
  - **EmergencyModePublisher (Activity A8)**: Emergency control publisher
    - Active only when in emergency mode
    - Publishes safety-critical emergency data via DDS
    - Includes emergency braking, seatbelt tightening, airbag status

### Secondary Processes (`adas_secondary`)
**Purpose**: Safety-critical control and actuation

The secondary system is designed to run as separate processes for enhanced safety isolation:

#### Secondary Process 1 (`adas_secondary 1`)
**Agent 102 (Worker 44)**: Vehicle control systems
- **Worker 44** (Control activities):
  - **LaneAssist (Activity A9)**: C++ lane keeping system
    - Consumes scene data to calculate steering corrections
    - Implements lane departure prevention algorithms
    - Publishes steering commands for lane centering
  - **SteeringController (Activity A10)**: Steering actuator
    - Receives steering commands from LaneAssist
    - Simulates physical steering system control
    - Executes steering angle adjustments

#### Secondary Process 2 (`adas_secondary 2`) 
**Agent 102 (Worker 44)**: Visualization and monitoring
- **Worker 44** (Visualization activities):
  - **TrajectoryVisualizer (Activity A11)**: C++ trajectory planner
    - Visualizes planned vehicle trajectory
    - Consumes steering data for path prediction
    - Provides trajectory feedback for debugging

#### Process Separation Benefits
- **Safety Isolation**: Critical steering control (Process 1) separated from visualization (Process 2)
- **Fault Tolerance**: Trajectory visualization failure doesn't affect steering control
- **Performance**: Control and visualization can run on different CPU cores/priorities
- **Development**: Visualization can be disabled/restarted without affecting vehicle control

## 3. Activity Dependencies & Data Flow

### Dependency Chain
```
Camera (A0) ──┐
              ├─→ NeuralNet (A2) ─┬─→ EnvironmentRenderer (A3)
Radar (A1) ───┘                   ├─→ CarModeCalculator (A4) ─→ CarDataPublisher (A5) ─┬─→ AutonomousModePublisher (A6)
                                  │                                                     ├─→ ManualModePublisher (A7)
                                  │                                                     └─→ EmergencyModePublisher (A8)
                                  └─→ LaneAssist (A9) ─┬─→ SteeringController (A10)
                                                       └─→ TrajectoryVisualizer (A11)
```

### Activity Assignment Table
| Activity | Name                    | Process      | Agent | Worker | Description |
|----------|------------------------|--------------|-------|--------|-------------|
| **A0**   | Camera                 | Primary      | 100   | 40     | Front camera sensor simulation |
| **A1**   | Radar                  | Primary      | 100   | 41     | Front radar sensor simulation |
| **A2**   | NeuralNet              | Primary      | 101   | 42     | Sensor data fusion engine |
| **A3**   | EnvironmentRenderer    | Primary      | 101   | 42     | Scene visualization |
| **A4**   | CarModeCalculator      | Primary      | 101   | 42     | Driving mode decision engine |
| **A5**   | CarDataPublisher       | Primary      | 101   | 42     | DDS gateway for mode data |
| **A6**   | AutonomousModePublisher| Primary      | 101   | 42     | Autonomous control data publisher |
| **A7**   | ManualModePublisher    | Primary      | 101   | 42     | Manual control data publisher |
| **A8**   | EmergencyModePublisher | Primary      | 101   | 42     | Emergency control data publisher |
| **A9**   | LaneAssist             | Secondary 1  | 102   | 44     | C++ lane keeping system |
| **A10**  | SteeringController     | Secondary 1  | 102   | 44     | Steering actuator control |
| **A11**  | TrajectoryVisualizer   | Secondary 2  | 102   | 44     | C++ trajectory planning |

### Process Communication
- **Primary Process**: Handles A0-A8 (sensor simulation, data fusion, mode decisions, DDS publishing)
- **Secondary Process 1**: Handles A9-A10 (safety-critical steering control)
- **Secondary Process 2**: Handles A11 (trajectory visualization and monitoring)
- **Inter-Process**: Scene data flows from Primary A2 → Secondary 1 A9 via FEO shared memory

### Implementation Note
**Config File Mapping**: Due to historical reasons, the `config.rs` file uses non-sequential activity IDs:
- Logical A4 (CarModeCalculator) = Config ID 9
- Logical A5 (CarDataPublisher) = Config ID 10  
- Logical A6 (AutonomousModePublisher) = Config ID 11
- Logical A7 (ManualModePublisher) = Config ID 12
- Logical A8 (EmergencyModePublisher) = Config ID 13
- Logical A9 (LaneAssist) = Config ID 5
- Logical A10 (SteeringController) = Config ID 7
- Logical A11 (TrajectoryVisualizer) = Config ID 8

**Process Arguments**: 
- Run primary: `cargo run --bin adas_primary 9000`
- Run secondary 1: `cargo run --bin adas_secondary 1` (steering control)
- Run secondary 2: `cargo run --bin adas_secondary 2` (visualization)

The logical numbering (A0-A11) provides clearer documentation, while config IDs maintain backward compatibility.

### Communication Mechanisms
- **Internal FEO**: Shared memory communication between activities within each process
- **Inter-Process**: Scene data from Primary NeuralNet → Secondary LaneAssist via FEO
- **External DDS**: Mode publishers → External systems (pullpiri) via DDS topics

## 4. Car Mode Logic & Decision Making

The system continuously monitors the environment and switches between three driving modes based on safety-critical thresholds:

### Mode Decision Algorithm
```rust
if obstacle_distance < 5.0 meters {
    mode = Emergency    // Immediate danger - emergency braking
} else if obstacle_distance < 10.0 || people > 2 || cars > 3 {
    mode = Manual       // Complex traffic - driver control
} else {
    mode = Autonomous   // Safe conditions - system takes control
}
```

### Mode Characteristics
- **Autonomous Mode**: 
  - Conditions: obstacle > 10m, people ≤ 2, cars ≤ 3 (safe conditions)
  - System has full control
  - Optimal for highway cruising, clear roads, predictable scenarios
  - Vehicle speed: 40-80 km/h
  - Publishes lane position, traffic signals, obstacle detection

- **Manual Mode**: 
  - Conditions: obstacle < 10m OR people > 2 OR cars > 3 (complex traffic)
  - Driver has full control with system assistance
  - Complex intersections, heavy traffic, unpredictable scenarios
  - Vehicle speed: 30-70 km/h
  - Publishes driver input data, throttle position, alertness status

- **Emergency Mode**: 
  - Conditions: obstacle < 5m
  - Immediate threat detected
  - Emergency braking activated (up to 100% force)
  - Seatbelt tightening, airbag systems primed
  - Vehicle speed: 10-40 km/h (reduced)
  - Emergency lights activated

## 5. Detailed Data Flow & Integration

### Internal Data Pipeline (FEO Shared Memory)
1. **Sensor Simulation**: Camera and Radar generate pseudo-realistic sensor data
2. **Data Fusion**: NeuralNet combines sensor inputs into unified scene understanding
3. **Parallel Processing**: Scene data flows to visualization, mode calculation, and control
4. **Mode Decision**: CarModeCalculator analyzes scene and determines driving mode
5. **Control Publication**: Appropriate mode publisher activates based on current mode

### External Integration (DDS)
- **CarData Topic**: Basic mode information for scenario engines (pullpiri)
- **AutonomousCarData Topic**: Detailed autonomous vehicle telemetry  
- **ManualCarData Topic**: Driver assistance and manual control data
- **EmergencyModeData Topic**: Safety-critical emergency system status

### Real-Time Operation
- All activities run in parallel with configurable timing
- Mode publishers operate at different frequencies based on criticality
- Emergency mode has highest priority and fastest response time
- DDS topics enable real-time external monitoring and control

## 6. Technical Implementation Details

### Agent/Worker Assignment Strategy
- **Agent 100**: Sensor simulation (Workers 40, 41)
  - Isolated sensor activities for performance 
  - Independent random data generation
- **Agent 101**: Core processing (Worker 42)  
  - Data fusion, decision making, and mode publishing
  - Centralized logic for consistency
- **Agent 102**: Safety-critical control (Worker 44)
  - Steering and trajectory control
  - Separated for safety isolation

### Domain Configuration
- **DDS Domain ID**: 100 (configurable in `get_dds_participant()`)
- **Topic Names**: CarData, AutonomousCarData, ManualCarData, EmergencyModeData
- **QoS Settings**: Default reliability and durability

### Performance Characteristics  
- **Sensor Frequency**: Random timing (10-45ms intervals)
- **Decision Latency**: Real-time mode switching based on scene changes
- **Control Response**: Immediate activation of safety systems in emergency mode
- **Memory Management**: Shared DDS participant prevents memory corruption

---

**File References**: 
- Process configuration: `src/config.rs`
- Activity implementations: `src/activities/components.rs`  
- Message definitions: `src/activities/messages.rs`
- Build configuration: `Cargo.toml`, `BUILD.bazel`
