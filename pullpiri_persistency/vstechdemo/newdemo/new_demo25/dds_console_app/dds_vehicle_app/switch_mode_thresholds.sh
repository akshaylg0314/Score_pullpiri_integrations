#!/bin/bash
# Mode Threshold Switcher Script
# This script demonstrates how an external service should update thresholds.json
# when the driving mode changes

MODE=$1

if [ -z "$MODE" ]; then
    echo "Usage: $0 <autonomous|manual|emergency>"
    echo ""
    echo "Current mode status:"
    curl -s http://localhost:9083/mode-status | jq '.'
    exit 1
fi

case "$MODE" in
    autonomous)
        echo "🤖 Switching to AUTONOMOUS mode thresholds..."
        cp thresholds_autonomous.json thresholds.json
        echo "✅ Thresholds updated for autonomous mode"
        echo "   - Obstacle distance min: 25.0m (strict)"
        echo "   - Collision risk max: 50.0% (conservative)"
        echo "   - Speed max: 90 km/h"
        echo "   - Cooldown: 30 seconds"
        ;;
    manual)
        echo "👤 Switching to MANUAL mode thresholds..."
        cp thresholds_manual.json thresholds.json 2>/dev/null || cat > thresholds.json << 'EOF'
{
  "current_mode_thresholds": {
    "obstacle_distance_min": 15.0,
    "collision_risk_max": 70.0,
    "vehicle_speed_max": 100.0,
    "vehicle_speed_min": 0.0,
    "brake_force_max": 85.0,
    "steering_angle_max": 35.0,
    "max_people_in_scene": 8
  },
  "stability": {
    "confirmation_window": 5,
    "mode_change_cooldown_ms": 30000
  }
}
EOF
        echo "✅ Thresholds updated for manual mode"
        echo "   - Obstacle distance min: 15.0m (moderate)"
        echo "   - Collision risk max: 70.0% (moderate)"
        echo "   - Speed max: 100 km/h"
        echo "   - Cooldown: 30 seconds"
        ;;
    emergency)
        echo "🚨 Switching to EMERGENCY mode thresholds..."
        cp thresholds_emergency.json thresholds.json
        echo "✅ Thresholds updated for emergency mode"
        echo "   - Obstacle distance min: 5.0m (critical only)"
        echo "   - Collision risk max: 90.0% (recovery focus)"
        echo "   - Speed max: 120 km/h"
        echo "   - Cooldown: 15 seconds (faster recovery)"
        ;;
    *)
        echo "❌ Invalid mode: $MODE"
        echo "Valid modes: autonomous, manual, emergency"
        exit 1
        ;;
esac

echo ""
echo "📁 File watcher will reload thresholds within 2 seconds"
echo ""
echo "Current thresholds:"
cat thresholds.json | jq '.'
