/********************************************************************************
 * Copyright (c) 2025 Contributors to the Eclipse Foundation
 *
 * See the NOTICE file(s) distributed with this work for additional
 * information regarding copyright ownership.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Apache License Version 2.0 which is available at
 * https://www.apache.org/licenses/LICENSE-2.0
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

//! Messages
//!
//! This module contains the definition of messages
//! to be used within this example.

#[cfg(feature = "recording")]
use feo::{recording::registry::TypeRegistry, register_type, register_types};
#[cfg(feature = "recording")]
use postcard::experimental::max_size::MaxSize;
#[cfg(feature = "recording")]
use serde::{Deserialize, Serialize};
use dust_dds::topic_definition::type_support::DdsType;

/// VehicleData
///
/// Vehicle driving mode parameters
#[derive(DdsType, Clone, Debug, Default)]
#[repr(C)]
pub struct VehicleData {
    pub vehicle_speed: f64,        // km/h
    pub lane_position: f64,        // meters (-1.0 to 1.0, 0 = center)
    pub obstacle_detected: bool,   // true if obstacle detected
    pub obstacle_distance: f64,    // meters
    pub traffic_signal: String,    // "green", "yellow", "red", "stop"
    pub steering_angle: f64,       // degrees (-45 to 45)
    pub brake_force: f64,         // percentage (0-100)
    pub acceleration: f64,        // m/s²
    pub weather_condition: String, // "clear", "rain", "snow", "fog"
    pub road_condition: String,   // "dry", "wet", "icy", "gravel"
    pub timestamp: i64,           // Unix timestamp in milliseconds
    pub is_valid: bool,           // Data validity flag
    pub collision_risk: f64,        // percentage (0-100)
}

/// DashboardData
///
/// Combined DMS + LKAS + Vehicle data for UI dashboard display
/// Replaces VehicleData for the main dashboard publisher
#[derive(DdsType, Clone, Debug, Default)]
#[repr(C)]
pub struct DashboardData {
    // DMS Fields
    pub distraction_duration: f64,      // Current continuous distraction (seconds)
    pub gaze_direction: String,         // "Forward", "Left", "Right", "Down"
    pub head_yaw: f64,                  // degrees (-30 to 30)
    pub head_pitch: f64,                // degrees (-20 to 20)
    pub head_roll: f64,                 // degrees (-15 to 15)
    pub drowsiness_score: f64,          // 0-40% (kept low)
    pub attention_level: f64,           // 0-100%
    pub driver_status: String,          // "Active", "Distracted", "Drowsy"
    
    // LKAS Fields
    pub current_lane: i32,              // 1-4
    pub lane_position_offset: f64,      // meters from lane center (-1.0 to 1.0)
    pub lkas_status: String,            // "Active", "Standby", "PullingOver"
    pub left_lane_distance: f64,        // meters to left lane boundary
    pub right_lane_distance: f64,       // meters to right lane boundary
    
    // Vehicle Metrics (from original VehicleData)
    pub vehicle_speed: f64,             // km/h
    pub obstacle_distance: f64,         // meters
    pub obstacle_detected: bool,
    pub steering_angle: f64,            // degrees
    
    pub timestamp: i64,                 // Unix timestamp (ms)
    pub is_valid: bool,
}

/// DistractionMonitor
///
/// Published via DDS to trigger external alarms
#[derive(DdsType, Clone, Debug, Default)]
#[repr(C)]
pub struct DistractionMonitor {
    pub dms_val: i32,  // Current distraction duration in SECONDS (IDL long = i32)
}

/// DmsState
///
/// Internal state from Driver Monitoring System
/// Shared via FEO internal topics (not DDS)
/// NOTE: Cannot use String types - FEO internal communication requires zero-copy compatible types
#[cfg_attr(feature = "recording", derive(Serialize, Deserialize, MaxSize))]
#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct DmsState {
    pub distraction_duration: i64,      // Current continuous distraction (ms) - internal tracking
    pub gaze_direction: [u8; 16],       // Fixed-size string: "Forward", "Left", "Right", "Down"
    pub head_yaw: f64,                  // degrees
    pub head_pitch: f64,                // degrees
    pub head_roll: f64,                 // degrees
    pub drowsiness_score: f64,          // 0-40%
    pub attention_level: f64,           // 0-100%
    pub driver_status: [u8; 16],        // Fixed-size string: "Active", "Distracted", "Drowsy"
}

/// LkasState
///
/// Internal state from Lane Keep Assistance System
/// Shared via FEO internal topics (not DDS)
/// NOTE: Cannot use String types - FEO internal communication requires zero-copy compatible types
#[cfg_attr(feature = "recording", derive(Serialize, Deserialize, MaxSize))]
#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct LkasState {
    pub current_lane: i32,              // 1-4
    pub lane_position_offset: f64,      // meters from lane center
    pub steering_angle: f64,            // degrees
    pub lkas_status: [u8; 16],          // Fixed-size string: "Active", "Standby", "PullingOver", "LaneChange"
    pub left_lane_distance: f64,        // meters
    pub right_lane_distance: f64,       // meters
}

/// LaneChangeCommand
///
/// Command to change lanes (internal FEO message)
/// NOTE: Cannot use String types - FEO internal communication requires zero-copy compatible types
#[cfg_attr(feature = "recording", derive(Serialize, Deserialize, MaxSize))]
#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct LaneChangeCommand {
    pub target_lane: i32,       // 1-4
    pub reason: [u8; 32],       // Fixed-size string: "ObstacleAvoidance", "Pullover", "Normal"
    pub urgency: f64,           // 0.0-1.0
}

// Helper function to convert &str to fixed-size array
pub fn str_to_fixed<const N: usize>(s: &str) -> [u8; N] {
    let mut arr = [0u8; N];
    let bytes = s.as_bytes();
    let len = bytes.len().min(N - 1); // Leave room for null terminator
    arr[..len].copy_from_slice(&bytes[..len]);
    arr
}

// Helper function to convert fixed-size array to String
pub fn fixed_to_string<const N: usize>(arr: &[u8; N]) -> String {
    let len = arr.iter().position(|&b| b == 0).unwrap_or(N);
    String::from_utf8_lossy(&arr[..len]).to_string()
}

/// Camera image
///
/// A neural network could detect the number of people,
/// number of cars and the distance to the closest obstacle.
/// Given that we do not have a real neural network,
/// we already include information to be dummy inferred.
#[cfg_attr(feature = "recording", derive(Serialize, Deserialize, MaxSize))]
#[derive(Debug, Default)]
#[repr(C)]
pub struct CameraImage {
    pub num_people: usize,
    pub num_cars: usize,
    pub distance_obstacle: f64,
}

/// Radar scan
///
/// With post-processing, we could detect the closest object
/// from a real radar scan. In this example,
/// the message type already carries the information to be dummy extracted.
#[cfg_attr(feature = "recording", derive(Serialize, Deserialize, MaxSize))]
#[derive(Debug, Default)]
#[repr(C)]
pub struct RadarScan {
    pub distance_obstacle: f64,
    pub error_margin: f64,
}

/// Scene
///
/// The scene is the result of fusing the camera image and the radar scan
/// with a neural network. In our example, we just extract the information.
#[cfg_attr(feature = "recording", derive(Serialize, Deserialize, MaxSize))]
#[derive(Debug, Default)]
#[repr(C)]
pub struct Scene {
    pub num_people: usize,
    pub num_cars: usize,
    pub distance_obstacle: f64,
    pub distance_left_lane: f64,
    pub distance_right_lane: f64,
}

/// Brake instruction
///
/// This is an instruction whether to engage the brakes and at which level.
#[cfg_attr(feature = "recording", derive(Serialize, Deserialize, MaxSize))]
#[derive(Debug, Default)]
#[repr(C)]
pub struct BrakeInstruction {
    pub active: bool,
    pub level: f64,
}

/// Steering
///
/// This carries the angle of steering.
#[cfg_attr(feature = "recording", derive(Serialize, Deserialize, MaxSize))]
#[derive(Debug, Default)]
#[repr(C)]
pub struct Steering {
    pub angle: f64,
}

/// Return a type registry containing the types defined in this file
#[cfg(feature = "recording")]
pub fn type_registry() -> TypeRegistry {
    use core::fmt;
    use feo_com::interface::ActivityInput;

    #[cfg(feature = "com_iox2")]
    use feo_com::iox2::Iox2Input;

    #[cfg(feature = "com_linux_shm")]
    use feo_com::linux_shm::LinuxShmInput;

    fn activity_input<T>(topic: &str) -> Box<dyn ActivityInput<T>>
    where
        T: fmt::Debug + 'static,
    {
        #[cfg(feature = "com_iox2")]
        return Box::new(Iox2Input::new(topic));

        #[cfg(feature = "com_linux_shm")]
        Box::new(LinuxShmInput::new(topic))
    }

    let mut registry = TypeRegistry::default();
    register_types!(
        registry;
        CameraImage, |topic: &str| activity_input(topic);
        RadarScan, |topic: &str| activity_input(topic);
        Scene, |topic: &str| activity_input(topic);
        BrakeInstruction, |topic: &str| activity_input(topic);
        Steering, |topic: &str| activity_input(topic);
        VehicleData, |topic: &str| activity_input(topic);
        DmsState, |topic: &str| activity_input(topic);
        LkasState, |topic: &str| activity_input(topic);
        LaneChangeCommand, |topic: &str| activity_input(topic)
    );
    registry
}
