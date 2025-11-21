# Intelligent Mode Transition System - Documentation

## Overview
This system implements an intelligent driving mode transition system with three modes:
- **Autonomous**: Self-driving with strict safety thresholds
- **Manual**: Human driver control with moderate monitoring
- **Emergency**: Critical intervention mode with relaxed thresholds for recovery

## Key Features

### 1. **Mode-Specific Thresholds**
Each mode has its own threshold configuration that is dynamically loaded from `thresholds.json`. Your background service should update this file when modes change.

### 2. **Stability Mechanisms**
- **Confirmation Window**: Requires 3-5 consecutive samples before mode transitions (prevents oscillation)
- **Cooldown Period**: 
  - Autonomous/Manual: 30 seconds (meets avg 30s requirement)
  - Emergency: 15 seconds (within 10-20s range)
- **Hysteresis**: Different thresholds for entering vs. exiting modes

### 3. **Staged Recovery**
- Emergency → Manual (automatic after cooldown)
- Manual → Autonomous (requires all conditions clear)
- **Never** Emergency → Autonomous directly (safety measure)

### 4. **Weather/Road Adaptation**
Dynamic threshold multipliers based on conditions:
- **Rain/Wet**: 1.3x distance, 0.8x speed
- **Snow/Icy**: 1.5x distance, 0.6x speed  
- **Fog**: Forces manual mode (visibility issue)
- **Gravel**: 1.1x distance, 0.85x speed

### 5. **Transition Tracking**
Global state tracks:
- `current_mode`: Current driving mode
- `previous_mode`: Last mode before transition
- `transition_reason`: Detailed explanation of why mode changed
- `timestamp`: When transition occurred

## Threshold Configuration Files

### `thresholds.json` (Active - Manual Mode Default)
```json
{
  "current_mode_thresholds": {
    "obstacle_distance_min": 15.0,      // meters
    "collision_risk_max": 70.0,         // percentage
    "vehicle_speed_max": 100.0,         // km/h
    "vehicle_speed_min": 0.0,           // km/h
    "brake_force_max": 85.0,            // percentage
    "steering_angle_max": 35.0,         // degrees
    "max_people_in_scene": 8            // count
  },
  "stability": {
    "confirmation_window": 5,           // samples needed
    "mode_change_cooldown_ms": 30000    // 30 seconds
  }
}
```

### `thresholds_autonomous.json` (Template - Most Restrictive)
- Higher safety margins
- `obstacle_distance_min: 25.0m` (vs 15.0m manual)
- `collision_risk_max: 50.0%` (vs 70.0% manual)
- `steering_angle_max: 25.0°` (vs 35.0° manual)
- 30s cooldown for stability

### `thresholds_emergency.json` (Template - Recovery Mode)
- Relaxed thresholds for system recovery
- `obstacle_distance_min: 5.0m` (critical proximity)
- `collision_risk_max: 90.0%` (critical risk)
- 15s cooldown (faster recovery)

## Mode Transition Logic

### From Autonomous Mode
```
Autonomous → Manual: Threshold violations detected
Autonomous → Emergency: Critical violations (obstacle <10m, risk >85%, brake >80%)
```

### From Manual Mode  
```
Manual → Autonomous: All conditions clear for 5 consecutive samples + cooldown expired
Manual → Emergency: Critical violations (obstacle <5m, risk >90%, brake >85%)
```

### From Emergency Mode
```
Emergency → Manual: Conditions improve + 15s cooldown (staged recovery)
Emergency → Autonomous: NOT ALLOWED (must go through Manual first)
```

## REST API Endpoints

### `GET /data`
Returns latest vehicle data from DDS topic

### `GET /mode-status`
Returns mode transition information:
```json
{
  "current_mode": "manual",
  "previous_mode": "autonomous",
  "transition_reason": "Obstacle too close: 12.3m < 25.0m (adjusted for clear/dry)",
  "timestamp": 1732022400000
}
```

## How to Update Thresholds (External Service)

Your background service should:

1. **Monitor mode changes** via DDS `CarMode` topic
2. **Copy appropriate template** when mode changes:
   ```bash
   # On transition to autonomous
   cp thresholds_autonomous.json thresholds.json
   
   # On transition to manual  
   cp thresholds_manual.json thresholds.json
   
   # On transition to emergency
   cp thresholds_emergency.json thresholds.json
   ```
3. **File watcher** automatically reloads within 2 seconds

## Threshold Tuning Guidelines

Based on vehicle data generator analysis:

### Vehicle Data Ranges
- `obstacle_distance`: 16.0 - 60.0m
- `vehicle_speed`: 0 - 80 km/h
- `collision_risk`: 0 - 100%
- `brake_force`: 0 - 100%

### Recommended Thresholds

**Autonomous (Conservative)**:
- Keep obstacle distance >30% of max range (25m minimum)
- Collision risk <50% for safety margin
- Speed limits conservative (90 km/h max)

**Manual (Moderate)**:
- Obstacle distance >25% of min range (15m minimum)
- Collision risk <70% (human can react)
- Speed limits moderate (100 km/h max)

**Emergency (Recovery)**:
- Obstacle distance at critical level (5m minimum)
- Collision risk very high (90% max)
- Focus on recovery, not prevention

## Testing Validation

### Stability Checks
1. ✅ Mode transitions require confirmation window (no single-sample flips)
2. ✅ Cooldown periods enforced (30s for autonomous/manual, 15s for emergency)
3. ✅ Staged recovery prevents Emergency → Autonomous jumps
4. ✅ Weather multipliers applied correctly
5. ✅ Transition reasons tracked for UI feedback

### Expected Behavior
- **Average mode duration**: ~30s for autonomous/manual (cooldown enforces minimum)
- **Emergency duration**: 10-20s (15s cooldown + recovery time)
- **No rapid oscillations**: Confirmation window prevents mode spam
- **Graceful degradation**: Autonomous → Manual → Emergency path
- **Safe recovery**: Emergency → Manual → Autonomous path

## Running the System

```bash
cd /home/acrn/new_ak/demo25/new_demo25/dds_console_app/dds_vehicle_app
cargo run
```

System will:
1. Start in **manual** mode (failsafe default)
2. Load thresholds from `thresholds.json`
3. Monitor vehicle data via DDS
4. Publish mode changes to `CarMode` DDS topic
5. Expose REST APIs on port 9083

## Integration Points

### For UI
- Poll `GET /mode-status` to show current mode and transition reason
- Display weather-adjusted thresholds to driver
- Show cooldown countdown timer

### For External Service  
- Subscribe to `CarMode` DDS topic
- Update `thresholds.json` based on mode
- Monitor transition frequency for tuning

### For Vehicle Data Generator
- Publishes to `VehicleData` DDS topic
- Includes weather/road conditions
- Real-time data at 10-45ms intervals

## Failsafe Behaviors

1. **Invalid data** (`is_valid = false`): Continue with last valid data
2. **Fog weather**: Force manual mode immediately
3. **Unknown mode**: Default to manual (safest)
4. **File watch failure**: Continue with loaded thresholds
5. **No data received**: Maintain current mode (no panic transitions)
