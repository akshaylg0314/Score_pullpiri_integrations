#!/usr/bin/env python3
import sys
import json
import shutil
import os
import argparse
from datetime import datetime

def main():
    parser = argparse.ArgumentParser(description='Manual Driving Threshold Manager')
    parser.add_argument('--obstacle', type=float, required=True, help='Manual obstacle detection threshold (meters)')
    parser.add_argument('--risk', type=float, required=True, help='Manual risk assessment threshold (percentage)')
    parser.add_argument('--brake', type=float, required=True, help='Manual brake force threshold (percentage)')
    parser.add_argument('--speed-max', type=float, default=100.0, help='Maximum speed threshold for manual (km/h)')
    parser.add_argument('--speed-min', type=float, default=0.0, help='Minimum speed threshold for manual (km/h)')
    parser.add_argument('--steering', type=float, default=40.0, help='Maximum steering angle for manual (degrees)')
    
    args = parser.parse_args()
    
    # Create thresholds dictionary in backend-compatible format (balanced for manual)
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
            "mode_change_cooldown_ms": 25000  # Medium cooldown for manual (25 seconds)
        },
        "metadata": {
            "mode": "manual",
            "timestamp": datetime.now().isoformat(),
            "container_managed": True,
            "safety_level": "standard"
        }
    }
    
    # Ensure data directory exists
    data_dir = "/data"
    os.makedirs(data_dir, exist_ok=True)
    
    # Save directly to thresholds.json (what backend watches)
    active_path = os.path.join(data_dir, "thresholds.json")
    with open(active_path, "w") as f:
        json.dump(thresholds, f, indent=2)
    
    print(f"✅ Manual backend-compatible thresholds saved to {active_path}")
    print(f"📊 Manual Format: {json.dumps(thresholds, indent=2)}")
    
    # Also save mode-specific backup for reference
    manual_path = os.path.join(data_dir, "thresholds_manual.json")
    with open(manual_path, "w") as f:
        json.dump(thresholds, f, indent=2)
    
    print(f"📁 Manual backup saved to {manual_path}")
    
    # Verify files exist and are readable
    if os.path.exists(active_path):
        print("✅ Manual backend-compatible threshold file created successfully")
        
        # Display file contents for verification
        with open(active_path, 'r') as f:
            active_content = json.load(f)
        print(f"📁 Backend will load: {json.dumps(active_content, indent=2)}")
    else:
        print("❌ Error: thresholds.json not created")
        sys.exit(1)
    
    print("🚗 Manual driving threshold manager completed successfully")

if __name__ == "__main__":
    main()