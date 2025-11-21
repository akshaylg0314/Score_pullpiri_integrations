#!/usr/bin/env python3
import sys
import json
import shutil
import os
import argparse
from datetime import datetime

def main():
    parser = argparse.ArgumentParser(description='Autonomous Driving Threshold Manager')
    parser.add_argument('--obstacle', type=float, required=True, help='Obstacle detection threshold (meters)')
    parser.add_argument('--risk', type=float, required=True, help='Risk assessment threshold (percentage)')
    parser.add_argument('--brake', type=float, required=True, help='Brake force threshold (percentage)')
    parser.add_argument('--speed-max', type=float, default=120.0, help='Maximum speed threshold (km/h)')
    parser.add_argument('--speed-min', type=float, default=10.0, help='Minimum speed threshold (km/h)')
    parser.add_argument('--steering', type=float, default=30.0, help='Maximum steering angle (degrees)')
    
    args = parser.parse_args()
    
    # Create thresholds dictionary in backend-compatible format
    thresholds = {
        "current_mode_thresholds": {
            "obstacle_distance_min": args.obstacle,
            "collision_risk_max": args.risk,
            "vehicle_speed_max": args.speed_max,
            "vehicle_speed_min": args.speed_min,
            "brake_force_max": args.brake,
            "steering_angle_max": args.steering
        },
        "stability": {
            "mode_change_cooldown_ms": 30000  # 30 seconds cooldown
        },
        "metadata": {
            "mode": "autonomous",
            "timestamp": datetime.now().isoformat(),
            "container_managed": True
        }
    }
    
    # Ensure data directory exists
    data_dir = "/data"
    os.makedirs(data_dir, exist_ok=True)
    
    # Save directly to thresholds.json (what backend watches)
    active_path = os.path.join(data_dir, "thresholds.json")
    with open(active_path, "w") as f:
        json.dump(thresholds, f, indent=2)
    
    print(f"✅ Backend-compatible thresholds saved to {active_path}")
    print(f"📊 Backend Format: {json.dumps(thresholds, indent=2)}")
    
    # Also save mode-specific backup for reference
    autonomous_path = os.path.join(data_dir, "thresholds_autonomous.json")
    with open(autonomous_path, "w") as f:
        json.dump(thresholds, f, indent=2)
    
    print(f"📁 Backup saved to {autonomous_path}")
    
    # Verify files exist and are readable
    if os.path.exists(active_path):
        print("✅ Backend-compatible threshold file created successfully")
        
        # Display file contents for verification
        with open(active_path, 'r') as f:
            active_content = json.load(f)
        print(f"📁 Backend will load: {json.dumps(active_content, indent=2)}")
    else:
        print("❌ Error: thresholds.json not created")
        sys.exit(1)
    
    print("🎯 Autonomous driving threshold manager completed successfully")

if __name__ == "__main__":
    main()
