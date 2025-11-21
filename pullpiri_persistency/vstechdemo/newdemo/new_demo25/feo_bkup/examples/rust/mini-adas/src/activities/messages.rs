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
        VehicleData, |topic: &str| activity_input(topic)
    );
    registry
}
