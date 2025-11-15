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

//! Mini-ADAS Activity Components
//! 
//! This module implements a safety-first driving mode decision system:
//! 
//! ## Driving Mode Logic (Safety-First Approach):
//! 
//! **AUTONOMOUS MODE**: Activated in safe, predictable conditions where the system
//! can operate reliably:
//! - Clear roads with minimal traffic (≤2 people, ≤3 cars)  
//! - Safe obstacle distances (>10m)
//! - Highway cruising, rural roads, low-complexity scenarios
//! - System confidence is high, minimal variables to handle
//! 
//! **MANUAL MODE**: Activated in complex traffic scenarios where human experience
//! and intuition are preferred:
//! - Dense traffic (>2 people, >3 cars detected)
//! - Close obstacles (5-10m) requiring nuanced decisions
//! - City intersections, construction zones, school areas
//! - Unpredictable scenarios where human judgment excels
//! 
//! **EMERGENCY MODE**: Activated for immediate danger requiring instant protective action:
//! - Critical obstacles (<5m) with collision risk
//! - Emergency braking, seatbelt tightening, airbag preparation
//! - System override for maximum safety response
//! 
//! This approach ensures autonomous systems operate only when confident, while
//! complex scenarios are handled by experienced human drivers. It builds trust
//! through gradual capability demonstration and maintains safety through human
//! oversight in unpredictable situations.

use crate::activities::messages::{
    BrakeInstruction, CameraImage, RadarScan, Scene, Steering,
    CarData, AutonomousCarData, ManualCarData, EmergencyModeData,
};
 use feo_log::info;
use core::fmt;
use core::hash::{BuildHasher as _, Hasher as _};
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut, Range};

use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::qos::QosKind,
    publication::data_writer::DataWriter,
    domain::domain_participant::DomainParticipant,
    infrastructure::qos_policy::{
        DurabilityQosPolicy, DurabilityQosPolicyKind, ReliabilityQosPolicy,
        ReliabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
    },
};
use core::time::Duration;
use feo::activity::Activity;
use feo::ids::ActivityId;
use feo_com::interface::{ActivityInput, ActivityOutput};
#[cfg(feature = "com_iox2")]
use feo_com::iox2::{Iox2Input, Iox2Output};
#[cfg(feature = "com_linux_shm")]
use feo_com::linux_shm::{LinuxShmInput, LinuxShmOutput};
use feo_log::debug;
use feo_log::warn;
use feo_tracing::instrument;
use std::hash::RandomState;
use std::thread;

// Create individual DDS participants per component to prevent state sharing issues
// This ensures each component has its own clean DDS context and prevents
// shared state corruption when components restart
fn create_dds_participant() -> DomainParticipant {
    let factory = DomainParticipantFactory::get_instance();
    factory
        .create_participant(100, QosKind::Default, None, &[])
        .expect("Failed to create DDS participant")
}

const SLEEP_RANGE: Range<i64> = 10..45;

/// Camera activity
///
/// This activity emulates a camera generating a [CameraImage].
#[derive(Debug)]
pub struct Camera {
    /// ID of the activity
    activity_id: ActivityId,
    /// Image output
    output_image: Box<dyn ActivityOutput<CameraImage>>,

    // Local state for realistic scenario generation
    num_people: usize,
    num_cars: usize,
    distance_obstacle: f64,
    
    // Scenario state tracking for realistic traffic patterns
    scenario_timer: usize,
    current_scenario: String, // "highway", "city", "suburban", "emergency_test"
    scenario_duration: usize,
}

impl Camera {
    pub fn build(activity_id: ActivityId, image_topic: &str) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            output_image: activity_output(image_topic),
            num_people: 2,      // Start with moderate count
            num_cars: 3,        // Start with moderate count  
            distance_obstacle: 10.0,  // Start in mid-range to allow all modes
            
            // Initialize scenario tracking for realistic patterns
            scenario_timer: 0,
            current_scenario: "highway".to_string(), // Start with highway scenario
            scenario_duration: 40, // Steps before first scenario change
        })
    }

    fn get_image(&mut self) -> CameraImage {
        // Update scenario timer and potentially change scenario
        self.scenario_timer += 1;
        
        if self.scenario_timer >= self.scenario_duration {
            // Change to a new scenario
            self.scenario_timer = 0;
            self.current_scenario = match self.current_scenario.as_str() {
                "highway" => {
                    self.scenario_duration = 30; // Shorter for testing
                    "city".to_string()
                },
                "city" => {
                    self.scenario_duration = 25; // Shorter for testing  
                    "suburban".to_string()
                },
                "suburban" => {
                    self.scenario_duration = 15; // Short emergency test
                    "emergency_test".to_string()
                },
                "emergency_test" => {
                    self.scenario_duration = 40; // Back to highway
                    "highway".to_string()
                },
                _ => "highway".to_string(),
            };
            info!("🎬 Camera: Switching to {} scenario for {} steps", self.current_scenario, self.scenario_duration);
        }

        // Generate realistic data based on current scenario
        match self.current_scenario.as_str() {
            "highway" => {
                // Highway: Moderate cars that can trigger manual mode sometimes
                self.num_people = random_walk_integer(self.num_people, 0.3, 1).clamp(0, 3); // Very few people on highway
                self.num_cars = random_walk_integer(self.num_cars, 0.7, 2).clamp(2, 8);     // Can trigger manual with >5 cars
                self.distance_obstacle = random_walk_float(self.distance_obstacle, 0.8, 8.0).clamp(4.0, 25.0); // Can be closer sometimes
                debug!("🛣️  Highway scenario: people={}, cars={}, distance={:.1}m", self.num_people, self.num_cars, self.distance_obstacle);
            },
            "city" => {
                // City: High chance of manual mode - many pedestrians, many cars, closer distances
                self.num_people = random_walk_integer(self.num_people, 0.9, 3).clamp(3, 9); // Often >4 people
                self.num_cars = random_walk_integer(self.num_cars, 0.9, 3).clamp(4, 10);    // Often >5 cars
                self.distance_obstacle = random_walk_float(self.distance_obstacle, 1.0, 6.0).clamp(3.0, 12.0); // Often <6m
                debug!("🏙️  City scenario: people={}, cars={}, distance={:.1}m", self.num_people, self.num_cars, self.distance_obstacle);
            },
            "suburban" => {
                // Suburban: Mixed conditions - sometimes manual, sometimes autonomous
                self.num_people = random_walk_integer(self.num_people, 0.6, 2).clamp(2, 6); // Sometimes >4 people
                self.num_cars = random_walk_integer(self.num_cars, 0.6, 2).clamp(3, 7);     // Sometimes >5 cars
                self.distance_obstacle = random_walk_float(self.distance_obstacle, 0.7, 4.0).clamp(4.0, 18.0); // Mixed distances
                debug!("🏘️  Suburban scenario: people={}, cars={}, distance={:.1}m", self.num_people, self.num_cars, self.distance_obstacle);
            },
            "emergency_test" => {
                // Emergency: Close obstacles to test emergency mode
                self.num_people = random_walk_integer(self.num_people, 0.4, 1).clamp(0, 3); // Few people during emergency
                self.num_cars = random_walk_integer(self.num_cars, 0.4, 1).clamp(1, 4);     // Light traffic
                self.distance_obstacle = random_walk_float(self.distance_obstacle, 1.0, 2.0).clamp(1.5, 5.0); // Close obstacles for emergency
                debug!("🚨 Emergency test scenario: people={}, cars={}, distance={:.1}m", self.num_people, self.num_cars, self.distance_obstacle);
            },
            _ => {
                // Default fallback - balanced for all modes
                self.num_people = random_walk_integer(self.num_people, 0.7, 3).clamp(0, 8);
                self.num_cars = random_walk_integer(self.num_cars, 0.7, 3).clamp(1, 9);
                self.distance_obstacle = random_walk_float(self.distance_obstacle, 0.8, 8.0).clamp(2.0, 25.0);
            }
        }

        CameraImage {
            num_people: self.num_people,
            num_cars: self.num_cars,
            distance_obstacle: self.distance_obstacle,
        }
    }
}

impl Activity for Camera {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "Camera startup")]
    fn startup(&mut self) {}

    #[instrument(name = "Camera")]
    fn step(&mut self) {
        debug!("Stepping Camera");
        sleep_random();

        if let Ok(camera) = self.output_image.write_uninit() {
            let image = self.get_image();
            debug!("Sending image: {image:?}");
            let camera = camera.write_payload(image);
            camera.send().unwrap();
        }
    }

    #[instrument(name = "Camera shutdown")]
    fn shutdown(&mut self) {}
}

/// Radar activity
///
/// This component emulates are radar generating a [RadarScan].
#[derive(Debug)]
pub struct Radar {
    /// ID of the activity
    activity_id: ActivityId,
    /// Radar scan output
    output_scan: Box<dyn ActivityOutput<RadarScan>>,

    // Local state for pseudo-random output generation
    distance_obstacle: f64,
}

impl Radar {
    pub fn build(activity_id: ActivityId, radar_topic: &str) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            output_scan: activity_output(radar_topic),
            distance_obstacle: 10.0,  // Start in mid-range to allow all modes
        })
    }

    fn get_scan(&mut self) -> RadarScan {
        const DISTANCE_CHANGE_PROP: f64 = 1.0;

        let sample = random_walk_float(self.distance_obstacle, DISTANCE_CHANGE_PROP, 10.0); // Wider range
        self.distance_obstacle = sample.clamp(1.5, 30.0);  // Lower minimum for emergency scenarios

        let error_margin = gen_random_in_range(-10..10) as f64 / 10.0;

        RadarScan {
            distance_obstacle: self.distance_obstacle,
            error_margin,
        }
    }
}

impl Activity for Radar {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "Radar startup")]
    fn startup(&mut self) {}

    #[instrument(name = "Radar")]
    fn step(&mut self) {
        debug!("Stepping Radar");
        sleep_random();

        if let Ok(radar) = self.output_scan.write_uninit() {
            let scan = self.get_scan();
            debug!("Sending scan: {scan:?}");
            let radar = radar.write_payload(scan);
            radar.send().unwrap();
        }
    }

    #[instrument(name = "Radar shutdown")]
    fn shutdown(&mut self) {}
}

/// Neural network activity
///
/// This component emulates a neural network
/// pseudo-inferring a [Scene] output
/// from the provided [Camera] and [Radar] inputs.
#[derive(Debug)]
pub struct NeuralNet {
    /// ID of the activity
    activity_id: ActivityId,
    /// Image input
    input_image: Box<dyn ActivityInput<CameraImage>>,
    /// Radar scan input
    input_scan: Box<dyn ActivityInput<RadarScan>>,
    /// Scene output
    output_scene: Box<dyn ActivityOutput<Scene>>,
}

impl NeuralNet {
    pub fn build(
        activity_id: ActivityId,
        image_topic: &str,
        scan_topic: &str,
        scene_topic: &str,
    ) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            input_image: activity_input(image_topic),
            input_scan: activity_input(scan_topic),
            output_scene: activity_output(scene_topic),
        })
    }

    fn infer(image: &CameraImage, radar: &RadarScan, scene: &mut MaybeUninit<Scene>) {
        let CameraImage {
            num_people,
            num_cars,
            distance_obstacle,
        } = *image;

        let distance_obstacle = distance_obstacle.min(radar.distance_obstacle);
        let distance_left_lane = gen_random_in_range(5..10) as f64 / 10.0;
        let distance_right_lane = gen_random_in_range(5..10) as f64 / 10.0;

        // Get raw pointer to payload within `MaybeUninit`.
        let scene_ptr = scene.as_mut_ptr();

        // Safety: `scene_ptr` was create from a `MaybeUninit` of the right type and size.
        // The underlying type `Scene` has `repr(C)` and can be populated field by field.
        unsafe {
            (*scene_ptr).num_people = num_people;
            (*scene_ptr).num_cars = num_cars;
            (*scene_ptr).distance_obstacle = distance_obstacle;
            (*scene_ptr).distance_left_lane = distance_left_lane;
            (*scene_ptr).distance_right_lane = distance_right_lane;
        }
    }
}

impl Activity for NeuralNet {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "NeuralNet startup")]
    fn startup(&mut self) {}

    #[instrument(name = "NeuralNet")]
    fn step(&mut self) {
        debug!("Stepping NeuralNet");
        sleep_random();

        let camera = self.input_image.read();
        let radar = self.input_scan.read();
        let scene = self.output_scene.write_uninit();

        if let (Ok(camera), Ok(radar), Ok(mut scene)) = (camera, radar, scene) {
            debug!("Inferring scene with neural network");

            Self::infer(camera.deref(), radar.deref(), scene.deref_mut());
            // Safety: `Scene` has `repr(C)` and was fully initialized by `Self::infer` above.
            let scene = unsafe { scene.assume_init() };
            debug!("Sending Scene {:?}", scene.deref());
            scene.send().unwrap();
        }
    }

    #[instrument(name = "NeuralNet shutdown")]
    fn shutdown(&mut self) {}
}

/// Emergency braking activity
///
/// This component emulates an emergency braking function
/// which sends instructions to activate the brakes
/// if the distance to the closest obstacle becomes too small.
/// The level of brake engagement depends on the distance.
#[derive(Debug)]
pub struct EmergencyBraking {
    /// ID of the activity
    activity_id: ActivityId,
    /// Scene input
    input_scene: Box<dyn ActivityInput<Scene>>,
    /// Brake instruction output
    output_brake_instruction: Box<dyn ActivityOutput<BrakeInstruction>>,
}

impl EmergencyBraking {
    pub fn build(
        activity_id: ActivityId,
        scene_topic: &str,
        brake_instruction_topic: &str,
    ) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            input_scene: activity_input(scene_topic),
            output_brake_instruction: activity_output(brake_instruction_topic),
        })
    }
}

impl Activity for EmergencyBraking {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "EmergencyBraking startup")]
    fn startup(&mut self) {}

    #[instrument(name = "EmergencyBraking")]
    fn step(&mut self) {
        debug!("Stepping EmergencyBraking");
        sleep_random();

        let scene = self.input_scene.read();
        let brake_instruction = self.output_brake_instruction.write_uninit();

        if let (Ok(scene), Ok(brake_instruction)) = (scene, brake_instruction) {
            const ENGAGE_DISTANCE: f64 = 30.0;
            const MAX_BRAKE_DISTANCE: f64 = 15.0;

            if scene.distance_obstacle < ENGAGE_DISTANCE {
                // Map distances ENGAGE_DISTANCE..MAX_BRAKE_DISTANCE to intensities 0.0..1.0
                let level = f64::min(
                    1.0,
                    (ENGAGE_DISTANCE - scene.distance_obstacle)
                        / (ENGAGE_DISTANCE - MAX_BRAKE_DISTANCE),
                );

                let brake_instruction = brake_instruction.write_payload(BrakeInstruction {
                    active: true,
                    level,
                });
                brake_instruction.send().unwrap();
            } else {
                let brake_instruction = brake_instruction.write_payload(BrakeInstruction {
                    active: false,
                    level: 0.0,
                });
                brake_instruction.send().unwrap();
            }
        }
    }

    #[instrument(name = "EmergencyBraking shutdown")]
    fn shutdown(&mut self) {}
}

/// Brake controller activity
///
/// This component emulates a brake controller
/// which triggers the brakes based on an instruction
/// and therefore might run in a separate process
/// with only other ASIL-D activities.
#[derive(Debug)]
pub struct BrakeController {
    /// ID of the activity
    activity_id: ActivityId,
    /// Brake instruction input
    input_brake_instruction: Box<dyn ActivityInput<BrakeInstruction>>,
}

impl BrakeController {
    pub fn build(activity_id: ActivityId, brake_instruction_topic: &str) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            input_brake_instruction: activity_input(brake_instruction_topic),
        })
    }
}

impl Activity for BrakeController {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "BrakeController startup")]
    fn startup(&mut self) {}

    #[instrument(name = "BrakeController")]
    fn step(&mut self) {
        debug!("Stepping BrakeController");
        sleep_random();

        if let Ok(brake_instruction) = self.input_brake_instruction.read() {
            if brake_instruction.active {
                debug!(
                    "BrakeController activating brakes with level {:.3}",
                    brake_instruction.level
                )
            }
        }
    }

    #[instrument(name = "BrakeController shutdown")]
    fn shutdown(&mut self) {}
}

/// Environment renderer activity
///
/// This component emulates a renderer to display a scene
/// in the infotainment display.
/// In this example, it does not do anything with the scene input.
#[derive(Debug)]
pub struct EnvironmentRenderer {
    /// ID of the activity
    activity_id: ActivityId,
    /// Scene input
    input_scene: Box<dyn ActivityInput<Scene>>,
}

impl EnvironmentRenderer {
    pub fn build(activity_id: ActivityId, scene_topic: &str) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            input_scene: activity_input(scene_topic),
        })
    }
}

impl Activity for EnvironmentRenderer {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "EnvironmentRenderer startup")]
    fn startup(&mut self) {}

    #[instrument(name = "EnvironmentRenderer")]
    fn step(&mut self) {
        debug!("Stepping EnvironmentRenderer");
        sleep_random();

        if let Ok(_scene) = self.input_scene.read() {
            debug!("Rendering scene");
        }
    }

    #[instrument(name = "EnvironmentRenderer shutdown")]
    fn shutdown(&mut self) {}
}

/// Steering controller activity
///
/// This component emulates a steering controller
/// which adjusts the steering angle to control the heading of the car.
/// Therefore, it might run in a separate process
/// with only other ASIL-D activities.
#[derive(Debug)]
pub struct SteeringController {
    /// ID of the activity
    activity_id: ActivityId,
    /// Steering input
    input_steering: Box<dyn ActivityInput<Steering>>,
}

impl SteeringController {
    pub fn build(activity_id: ActivityId, steering_topic: &str) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            input_steering: activity_input(steering_topic),
        })
    }
}

impl Activity for SteeringController {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "SteeringController startup")]
    fn startup(&mut self) {}

    #[instrument(name = "SteeringController")]
    fn step(&mut self) {
        debug!("Stepping SteeringController");
        sleep_random();

        if let Ok(steering) = self.input_steering.read() {
            debug!(
                "SteeringController adjusting angle to {:.3}",
                steering.angle
            )
        }
    }

    #[instrument(name = "SteeringController shutdown")]
    fn shutdown(&mut self) {}
}

/// Car Mode Calculator activity
///
/// This activity analyzes scene data and determines the appropriate driving mode
/// based on environmental conditions:
/// - AUTONOMOUS: Safe conditions (clear roads, low traffic) - system control
/// - MANUAL: Complex traffic scenarios (many people/cars, obstacles) - driver control  
/// - EMERGENCY: Immediate danger (very close obstacles) - emergency systems
///
/// This approach ensures autonomous systems operate only in predictable, safe
/// conditions while handing control back to human drivers in complex situations.
#[derive(Debug)]
pub struct CarModeCalculator {
    /// ID of the activity
    activity_id: ActivityId,
    /// Scene input
    input_scene: Box<dyn ActivityInput<Scene>>,
    /// Car data output
    output_car_data: Box<dyn ActivityOutput<CarData>>,

    // Local state for mode calculation and smooth transitions
    current_mode: String,
    previous_published_mode: String,
    last_mode_change_time: Option<std::time::Instant>,
    obstacle_threshold: f64,
    emergency_threshold: f64,
    
    // Vehicle state for realistic behavior
    current_speed: f64,
    target_speed: f64,
    steering_angle: f64,
    brake_force: f64,
    
    // Minimum time between mode changes (1 minute)
    mode_change_cooldown: Duration,
}

impl CarModeCalculator {
    pub fn build(
        activity_id: ActivityId,
        scene_topic: &str,
        car_data_topic: &str,
    ) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            input_scene: activity_input(scene_topic),
            output_car_data: activity_output(car_data_topic),
            current_mode: "manual".to_string(),
            previous_published_mode: "".to_string(), // Initialize as empty to force first publish
            last_mode_change_time: None,
            obstacle_threshold: 6.0, // meters - manual mode threshold  
            emergency_threshold: 4.0, // meters - emergency mode threshold
            
            // Initialize vehicle state for realistic behavior
            current_speed: 50.0,  // Start at moderate highway speed
            target_speed: 50.0,
            steering_angle: 0.0,
            brake_force: 0.0,
            
            mode_change_cooldown: Duration::from_secs(15), // 15 seconds for testing (change to 60 for production)
        })
    }
}

impl Activity for CarModeCalculator {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "CarModeCalculator startup")]
    fn startup(&mut self) {
        info!("🚗 CarModeCalculator starting up - smooth mode transitions enabled");
        info!("⏱️ Mode change cooldown: {}s (minimum time between mode changes)", self.mode_change_cooldown.as_secs());
        info!("🎯 Current thresholds: emergency <{}m, manual <{}m or >4 people or >5 cars", 
              self.emergency_threshold, self.obstacle_threshold);
    }

    #[instrument(name = "CarModeCalculator step")]
    fn step(&mut self) {
        if let Ok(scene) = self.input_scene.read() {
            debug!("CarModeCalculator processing scene: people={}, cars={}, obstacle_distance={:.1}", 
                   scene.num_people, scene.num_cars, scene.distance_obstacle);

            // Show detailed condition evaluation
            let emergency_cond = scene.distance_obstacle < self.emergency_threshold;
            let manual_distance_cond = scene.distance_obstacle < self.obstacle_threshold;
            let manual_people_cond = scene.num_people > 4;
            let manual_cars_cond = scene.num_cars > 5;
            
            debug!("🔍 Conditions: Emergency({}<{}): {}, Manual distance({}<{}): {}, people({}): {}, cars({}): {}", 
                scene.distance_obstacle, self.emergency_threshold, emergency_cond,
                scene.distance_obstacle, self.obstacle_threshold, manual_distance_cond,
                scene.num_people, manual_people_cond,
                scene.num_cars, manual_cars_cond);

            // Determine potential new driving mode based on scene conditions
            let potential_new_mode = if emergency_cond {
                "emergency".to_string()
            } else if manual_distance_cond || manual_people_cond || manual_cars_cond {
                "manual".to_string()
            } else {
                "autonomous".to_string()
            };
            
            debug!("🎯 Potential new mode: {} (current: {})", potential_new_mode, self.current_mode);

            // Check if enough time has passed since last mode change (except for emergency)
            let can_change_mode = if potential_new_mode == "emergency" {
                info!("🚨 Emergency mode requested - bypassing cooldown");
                true // Emergency mode can always be activated immediately
            } else if let Some(last_change) = self.last_mode_change_time {
                let elapsed = last_change.elapsed();
                let can_change = elapsed >= self.mode_change_cooldown;
                if !can_change {
                    let remaining = (self.mode_change_cooldown.as_secs_f64() - elapsed.as_secs_f64()).max(0.0);
                    debug!("⏱️ Mode change cooldown active: {:.1}s remaining", remaining);
                }
                can_change
            } else {
                info!("🔄 First mode change - allowing immediate transition");
                true // First time, allow change
            };

            // Update mode only if conditions allow
            let should_change_mode = potential_new_mode != self.current_mode && can_change_mode;
            
            if should_change_mode {
                info!("🔄 Mode transition APPROVED: {} → {} (conditions: distance={:.1}m, people={}, cars={})", 
                    self.current_mode, potential_new_mode, scene.distance_obstacle, scene.num_people, scene.num_cars);
                
                self.current_mode = potential_new_mode.clone();
                self.last_mode_change_time = Some(std::time::Instant::now());
                info!("🕐 Next mode change allowed in {:.0} seconds", self.mode_change_cooldown.as_secs_f64());
                
                // Update vehicle behavior based on new mode
                let mode_for_behavior = self.current_mode.clone();
                self.update_vehicle_behavior(&mode_for_behavior, &scene);
            } else if potential_new_mode != self.current_mode && !can_change_mode {
                let remaining = if let Some(last_change) = self.last_mode_change_time {
                    (self.mode_change_cooldown.as_secs_f64() - last_change.elapsed().as_secs_f64()).max(0.0)
                } else { 0.0 };
                debug!("⏱️ Mode change BLOCKED: {} → {} (cooling down for {:.1}s more)", 
                    self.current_mode, potential_new_mode, remaining);
            }

            // Gradually adjust current speed towards target based on mode
            self.adjust_vehicle_dynamics();

            // Always publish car data so other components can read current mode
            // Only log mode changes to prevent spam, but always provide data
            if self.current_mode != self.previous_published_mode {
                // Log current conditions and mode with clear reasoning (only on actual change)
                match self.current_mode.as_str() {
                    "emergency" => info!("🚨 EMERGENCY MODE: Critical distance {:.1}m < {}m - speed reduced to {:.0} km/h!", 
                                       scene.distance_obstacle, self.emergency_threshold, self.current_speed),
                    "manual" => {
                        if scene.distance_obstacle < self.obstacle_threshold {
                            info!("👤 MANUAL MODE: Close obstacle {:.1}m - human control, speed {:.0} km/h", 
                                 scene.distance_obstacle, self.current_speed);
                        } else if scene.num_people > 4 {
                            info!("👤 MANUAL MODE: Heavy pedestrian traffic ({}) - speed {:.0} km/h", 
                                 scene.num_people, self.current_speed);
                        } else if scene.num_cars > 5 {
                            info!("👤 MANUAL MODE: Dense vehicle traffic ({}) - speed {:.0} km/h", 
                                 scene.num_cars, self.current_speed);
                        }
                    },
                    "autonomous" => info!("🤖 AUTONOMOUS MODE: Safe conditions - cruising at {:.0} km/h (distance {:.1}m)", 
                                        self.current_speed, scene.distance_obstacle),
                    _ => {}
                }
                
                // Update the last published mode for logging purposes
                self.previous_published_mode = self.current_mode.clone();
            } else {
                debug!("📝 Mode unchanged ({}), continuing to publish data for other components", self.current_mode);
            }

            // ALWAYS publish car data so other components can read current mode (even during cooldown)
            if let Ok(car_data_output) = self.output_car_data.write_uninit() {
                let car_data = CarData {
                    driving_mode: self.current_mode.clone(),
                };
                
                let car_data_output = car_data_output.write_payload(car_data);
                car_data_output.send().unwrap();
                debug!("📤 CarModeCalculator published CarData: {} (speed: {:.0} km/h)", 
                    self.current_mode, self.current_speed);
            }
        } else {
            debug!("CarModeCalculator: No scene data available");
        }

        sleep_random();
    }

    #[instrument(name = "CarModeCalculator shutdown")]
    fn shutdown(&mut self) {}
}

impl CarModeCalculator {
    /// Update vehicle behavior parameters based on driving mode
    fn update_vehicle_behavior(&mut self, mode: &str, scene: &Scene) {
        match mode {
            "emergency" => {
                // Emergency: immediate speed reduction, heavy braking
                self.target_speed = (self.current_speed * 0.3).max(10.0); // Reduce to 30% or minimum 10 km/h
                self.brake_force = 80.0; // Heavy braking
                info!("🚨 Emergency behavior: Target speed reduced to {:.0} km/h, brake force {:.0}%", 
                    self.target_speed, self.brake_force);
            },
            "manual" => {
                // Manual: variable speed based on traffic conditions
                if scene.distance_obstacle < 10.0 {
                    self.target_speed = 30.0 + (scene.distance_obstacle * 3.0); // 30-60 km/h based on distance
                } else {
                    self.target_speed = 40.0 + random_walk_float(0.0, 0.3, 15.0); // 40-55 km/h variable
                }
                self.target_speed = self.target_speed.clamp(25.0, 70.0);
                self.brake_force = random_walk_float(0.0, 0.2, 20.0).clamp(0.0, 40.0); // Light to moderate braking
                info!("👤 Manual behavior: Target speed {:.0} km/h, variable driving", self.target_speed);
            },
            "autonomous" => {
                // Autonomous: steady, efficient cruising speed
                let base_speed = 65.0; // Higher base speed for autonomous
                self.target_speed = base_speed + random_walk_float(0.0, 0.1, 5.0); // Very stable speed
                self.target_speed = self.target_speed.clamp(60.0, 75.0);
                self.brake_force = random_walk_float(0.0, 0.1, 5.0).clamp(0.0, 15.0); // Minimal braking
                info!("🤖 Autonomous behavior: Steady cruising at {:.0} km/h", self.target_speed);
            },
            _ => {}
        }
    }

    /// Gradually adjust vehicle dynamics towards target values
    fn adjust_vehicle_dynamics(&mut self) {
        // Smooth speed adjustment (not instant)
        let speed_diff = self.target_speed - self.current_speed;
        self.current_speed += speed_diff * 0.1; // 10% adjustment per step for smooth transition
        
        // Clamp to realistic ranges
        self.current_speed = self.current_speed.clamp(5.0, 120.0);
        
        // Adjust steering angle slightly for realism
        self.steering_angle = random_walk_float(self.steering_angle, 0.1, 2.0).clamp(-10.0, 10.0);
    }
}

/// Car Data Publisher activity
///
/// This activity publishes CarData to DDS for external systems like pullpiri
/// to receive the current driving mode.
pub struct CarDataPublisher {
    activity_id: ActivityId,
    input_car_data: Box<dyn ActivityInput<CarData>>,
    writer: Option<DataWriter<CarData>>,
    participant: Option<DomainParticipant>, // Own participant for clean restart
    last_published_mode: Option<String>, // Track last published mode to prevent spam
}

impl CarDataPublisher {
    pub fn build(
        activity_id: ActivityId,
        car_data_topic: &str,
    ) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            input_car_data: activity_input(car_data_topic),
            writer: None,
            participant: None, // Will be created in startup
            last_published_mode: None, // Initialize as None
        })
    }
}

impl Activity for CarDataPublisher {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    fn startup(&mut self) {
        info!("🚀 CarDataPublisher started, initializing DDS with INDIVIDUAL participant for clean restart...");
        
        // Create individual DDS participant for this component - prevents shared state issues
        let participant = create_dds_participant();

        let topic = participant
            .create_topic::<CarData>(
                "CarData",
                "CarData", 
                QosKind::Default,
                None,
                &[],
            )
            .unwrap();

        let publisher = participant
            .create_publisher(QosKind::Default, None, &[])
            .unwrap();

        // Configure QoS for reliable, persistent delivery with late-joiner support
        let writer_qos = dust_dds::infrastructure::qos::DataWriterQos {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort, // BestEffort doesn't wait for subscriber ACK
                max_blocking_time: dust_dds::infrastructure::time::DurationKind::Finite(
                    dust_dds::infrastructure::time::Duration::new(0, 100_000_000) // 100ms timeout
                ),
            },
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal, // Keep for late-joining subscribers
            },
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast(1), // Reduced to prevent memory buildup
            },
            ..Default::default()
        };
 

        let writer = publisher
            .create_datawriter::<CarData>(
                &topic,
                QosKind::Specific(writer_qos),
                None,
                &[],
            )
            .unwrap();

        info!("📡 CarDataPublisher setup with INDIVIDUAL participant - Enhanced QoS & clean restart support");
        
        self.writer = Some(writer);
        self.participant = Some(participant); // Store for clean shutdown
        
        thread::sleep(Duration::from_millis(200));
        info!("✅ CarDataPublisher DDS setup complete - ready to publish CarData with enhanced reliability");
    }

    fn step(&mut self) {
        if let Ok(car_data) = self.input_car_data.read() {
            // Check if this is a new/different mode compared to what we last published
            let should_publish = match &self.last_published_mode {
                Some(last_mode) => last_mode != &car_data.driving_mode,
                None => true, // First publish
            };
            
            if should_publish {
                info!("📡 NEW MODE DETECTED - Publishing CarData to DDS for pullpiri: {} → {}", 
                    self.last_published_mode.as_deref().unwrap_or("none"), car_data.driving_mode);

                if let Some(writer) = &mut self.writer {
                    let dds_car_data = CarData {
                        driving_mode: car_data.driving_mode.clone(),
                    };

                    info!("🌐 [DDS] Sending NEW CarData for pullpiri integration: {:?}", dds_car_data);
                    
                    // Attempt write without waiting for subscribers
                    // Using _ as instance handle means we don't wait for acknowledgment
                    match writer.write(&dds_car_data, None) {
                        Ok(_) => {
                            info!("✅ [DDS] NEW CarData successfully published to topic 'CarData'");
                            self.last_published_mode = Some(car_data.driving_mode.clone());
                        },
                        Err(_) => {
                            // Silently handle - data is stored in TransientLocal durability anyway
                            // Subscribers will get it when they join
                            debug!("📝 [DDS] CarData cached in TransientLocal (no active subscribers)");
                            self.last_published_mode = Some(car_data.driving_mode.clone());
                        },
                    }
                }
            } else {
                debug!("📡 CarDataPublisher: Mode unchanged ({}), skipping DDS publish to prevent spam", car_data.driving_mode);
            }
        } else {
            debug!("CarDataPublisher: No car data available to publish");
        }

        sleep_random();
    }

    fn shutdown(&mut self) {
        info!("🔄 CarDataPublisher shutting down - cleaning up individual DDS participant");
        self.writer = None;
        self.participant = None; // Clean shutdown of individual participant
    }
}

/// Autonomous Mode Publisher activity
///
/// This activity publishes comprehensive autonomous driving data with DDS output
/// when the car is in autonomous mode.
pub struct AutonomousModePublisher {
    activity_id: ActivityId,
    input_car_data: Box<dyn ActivityInput<CarData>>,
    input_scene: Box<dyn ActivityInput<Scene>>,
    writer: Option<DataWriter<AutonomousCarData>>,
    participant: Option<DomainParticipant>, // Own participant for clean restart
    discovery_counter: u32, // Counter for periodic subscriber re-discovery
}

impl AutonomousModePublisher {
    pub fn build(
        activity_id: ActivityId,
        car_data_topic: &str,
        scene_topic: &str,
        _autonomous_data_topic: &str,
    ) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            input_car_data: activity_input(car_data_topic),
            input_scene: activity_input(scene_topic),
            writer: None,
            participant: None, // Will be created in startup
            discovery_counter: 0, // Initialize discovery counter
        })
    }
}

impl Activity for AutonomousModePublisher {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    fn startup(&mut self) {
        info!("🤖 AutonomousModePublisher started with INDIVIDUAL participant - publishes IMMEDIATELY when autonomous mode is active");
        
        // Create individual DDS participant for this component - prevents shared state issues  
        let participant = create_dds_participant();

        let topic = participant
            .create_topic::<AutonomousCarData>(
                "AutonomousCarData",
                "AutonomousCarData",
                QosKind::Default,
                None,
                &[],
            )
            .unwrap();

        let publisher = participant
            .create_publisher(QosKind::Default, None, &[])
            .unwrap();

        let writer_qos = dust_dds::infrastructure::qos::DataWriterQos {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort, // BestEffort doesn't wait for subscriber ACK
                max_blocking_time: dust_dds::infrastructure::time::DurationKind::Finite(
                    dust_dds::infrastructure::time::Duration::new(0, 100_000_000) // 100ms timeout
                ),
            },
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal, // Keep for late-joining subscribers
            },
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast(5), // Reduced to prevent memory buildup
            },
            ..Default::default()
        };
 

        let writer = publisher
            .create_datawriter::<AutonomousCarData>(
                &topic,
                QosKind::Specific(writer_qos),
                None,
                &[],
            )
            .unwrap();

        self.writer = Some(writer);
        self.participant = Some(participant); // Store for clean shutdown
        
        // Longer initial discovery time to ensure subscribers are detected after restarts
        info!("🤖 AutonomousModePublisher waiting for subscriber discovery...");
        thread::sleep(Duration::from_millis(200));
        info!("🤖 AutonomousModePublisher discovery period complete");
    }

    fn step(&mut self) {
        // Increment discovery counter for periodic subscriber re-discovery
        self.discovery_counter += 1;
        
        // Every 100 steps (~10 seconds), add a brief pause to help with subscriber discovery after restarts
        if self.discovery_counter % 100 == 0 {
            debug!("🤖 AutonomousModePublisher: Periodic discovery refresh (step {})", self.discovery_counter);
            thread::sleep(Duration::from_millis(50)); // Brief pause for discovery refresh
        }
        
        // Try to read both inputs
        let car_data_result = self.input_car_data.read();
        let scene_result = self.input_scene.read();
        
        // Check if we're in autonomous mode either from car_data or by calculating from scene
        let is_autonomous_mode = if let Ok(car_data) = &car_data_result {
            car_data.driving_mode == "autonomous"
        } else if let Ok(scene) = &scene_result {
            // Calculate mode directly from scene if car_data is not available
            let emergency_cond = scene.distance_obstacle < 4.0;
            let manual_cond = scene.distance_obstacle < 6.0 || scene.num_people > 4 || scene.num_cars > 5;
            !emergency_cond && !manual_cond // autonomous if not emergency and not manual
        } else {
            false
        };
        
        if is_autonomous_mode {
            info!("🤖 AUTONOMOUS MODE ACTIVE - Publishing detailed data immediately (no delay)");
            
            // Use scene data (required) and car_data (optional)
            if let Ok(scene) = scene_result {
                if let Some(writer) = &mut self.writer {
                    let current_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as i64;

                    // Calculate realistic autonomous driving values based on actual scene conditions
                    let optimal_speed = if scene.distance_obstacle > 20.0 { 
                        70.0 // Highway cruising speed when clear
                    } else if scene.distance_obstacle > 15.0 { 
                        60.0 // Moderate speed with some obstacles
                    } else { 
                        50.0 // Slower when obstacles present
                    };
                    
                    // Autonomous systems maintain steady, efficient speeds
                    let realistic_speed = optimal_speed - (scene.num_cars as f64 * 0.5); // Slight reduction for traffic
                    let final_speed = realistic_speed.clamp(45.0, 75.0);
                    
                    // Autonomous systems make precise, minimal steering adjustments
                    let realistic_steering = if scene.distance_left_lane < scene.distance_right_lane {
                        2.0 // Small adjustment right
                    } else if scene.distance_right_lane < scene.distance_left_lane {
                        -2.0 // Small adjustment left  
                    } else {
                        0.0 // Stay centered
                    };
                    
                    let realistic_brake = if scene.distance_obstacle < 20.0 { 
                        (30.0 - scene.distance_obstacle) * 5.0 
                    } else { 0.0 };
                    
                    let realistic_acceleration = if scene.distance_obstacle > 30.0 { 
                        1.5 // Smooth acceleration when clear
                    } else if scene.distance_obstacle < 15.0 { 
                        -1.0 // Smooth deceleration when obstacles
                    } else { 
                        0.0 // Maintain speed
                    };

                    let autonomous_data = AutonomousCarData {
                        vehicle_speed: final_speed, // Real speed based on conditions
                        lane_position: (realistic_steering as f64 / 10.0).clamp(-0.5f64, 0.5f64), // Real lane position
                        obstacle_detected: scene.distance_obstacle < 30.0,
                        obstacle_distance: scene.distance_obstacle,
                        traffic_signal: if scene.num_cars > 5 { "red".to_string() } else { "green".to_string() },
                        steering_angle: realistic_steering, // Real precise steering
                        brake_force: realistic_brake, // Real braking based on distance
                        acceleration: realistic_acceleration, // Real smooth acceleration
                        weather_condition: "clear".to_string(),
                        road_condition: "dry".to_string(),
                        timestamp: current_time,
                        is_valid: true,
                    };

                    // Check subscriber count for better restart detection
                    let subscriber_count = writer.get_publication_matched_status()
                        .map(|status| status.current_count)
                        .unwrap_or(0);
                    
                    debug!("[DDS] Sending autonomous car data (subscribers: {}): {:?}", subscriber_count, autonomous_data);
                    match writer.write(&autonomous_data, None) {
                        Ok(_) => {
                            if subscriber_count > 0 {
                                debug!("✅ [DDS] Autonomous car data published successfully to {} subscriber(s)", subscriber_count);
                            } else {
                                debug!("✅ [DDS] Autonomous car data published and cached (no subscribers detected yet)");
                            }
                        },
                        Err(_) => {
                            // Silently handle - data is stored in TransientLocal durability anyway
                            // Subscribers will get it when they join
                            debug!("📝 [DDS] AutonomousCarData cached in TransientLocal (no active subscribers detected)");
                        },
                    }
                }
            } else {
                debug!("AutonomousModePublisher: No scene data available");
            }
        }

        sleep_random();
    }

    fn shutdown(&mut self) {
        info!("🔄 AutonomousModePublisher shutting down - cleaning up individual DDS participant");
        self.writer = None;
        self.participant = None; // Clean shutdown of individual participant
    }
}

/// Manual Mode Publisher activity
///
/// This activity publishes comprehensive manual driving data with DDS output
/// when the car is in manual mode.
pub struct ManualModePublisher {
    activity_id: ActivityId,
    input_car_data: Box<dyn ActivityInput<CarData>>,
    input_scene: Box<dyn ActivityInput<Scene>>,
    writer: Option<DataWriter<ManualCarData>>,
    participant: Option<DomainParticipant>, // Own participant for clean restart
    discovery_counter: u32, // Counter for periodic subscriber re-discovery
}

impl ManualModePublisher {
    pub fn build(
        activity_id: ActivityId,
        car_data_topic: &str,
        scene_topic: &str,
        _manual_data_topic: &str,
    ) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            input_car_data: activity_input(car_data_topic),
            input_scene: activity_input(scene_topic),
            writer: None,
            participant: None, // Will be created in startup
            discovery_counter: 0, // Initialize discovery counter
        })
    }
}

impl Activity for ManualModePublisher {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    fn startup(&mut self) {
        info!("👤 ManualModePublisher started with INDIVIDUAL participant - publishes IMMEDIATELY when manual mode is active");
        
        // Create individual DDS participant for this component - prevents shared state issues
        let participant = create_dds_participant();

        let topic = participant
            .create_topic::<ManualCarData>(
                "ManualCarData",
                "ManualCarData",
                QosKind::Default,
                None,
                &[],
            )
            .unwrap();

        let publisher = participant
            .create_publisher(QosKind::Default, None, &[])
            .unwrap();
        
        let writer_qos = dust_dds::infrastructure::qos::DataWriterQos {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort, // BestEffort doesn't wait for subscriber ACK
                max_blocking_time: dust_dds::infrastructure::time::DurationKind::Finite(
                    dust_dds::infrastructure::time::Duration::new(0, 100_000_000) // 100ms timeout
                ),
            },
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal, // Keep for late-joining subscribers
            },
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast(5), // Reduced to prevent memory buildup
            },
            ..Default::default()
        };

        let writer = publisher
            .create_datawriter::<ManualCarData>(
                &topic,
                QosKind::Specific(writer_qos),
                None,
                &[],
            )
            .unwrap();

        self.writer = Some(writer);
        self.participant = Some(participant); // Store for clean shutdown
        
        // Longer initial discovery time to ensure subscribers are detected after restarts
        info!("👤 ManualModePublisher waiting for subscriber discovery...");
        thread::sleep(Duration::from_millis(200));
        info!("👤 ManualModePublisher discovery period complete");
    }

    fn step(&mut self) {
        // Increment discovery counter for periodic subscriber re-discovery
        self.discovery_counter += 1;
        
        // Every 100 steps (~10 seconds), add a brief pause to help with subscriber discovery after restarts
        if self.discovery_counter % 100 == 0 {
            debug!("👤 ManualModePublisher: Periodic discovery refresh (step {})", self.discovery_counter);
            thread::sleep(Duration::from_millis(50)); // Brief pause for discovery refresh
        }
        
        // Try to read both inputs
        let car_data_result = self.input_car_data.read();
        let scene_result = self.input_scene.read();
        
        // Check if we're in manual mode either from car_data or by calculating from scene
        let is_manual_mode = if let Ok(car_data) = &car_data_result {
            car_data.driving_mode == "manual"
        } else if let Ok(scene) = &scene_result {
            // Calculate mode directly from scene if car_data is not available
            let emergency_cond = scene.distance_obstacle < 4.0;
            let manual_cond = scene.distance_obstacle < 6.0 || scene.num_people > 4 || scene.num_cars > 5;
            !emergency_cond && manual_cond // manual if not emergency but meets manual conditions
        } else {
            false
        };
        
        if is_manual_mode {
            info!("👤 MANUAL MODE ACTIVE - Publishing detailed data immediately (no delay)");
            
            // Use scene data (required) and car_data (optional)
            if let Ok(scene) = scene_result {
                if let Some(writer) = &mut self.writer {
                    let current_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as i64;

                    // Calculate realistic manual driving values based on actual scene conditions
                    let base_speed = if scene.distance_obstacle < 10.0 {
                        30.0 + (scene.distance_obstacle * 3.0) // 30-60 km/h based on distance
                    } else {
                        45.0 // Moderate city/suburban speed
                    };
                    
                    let traffic_penalty = (scene.num_people as f64 * 2.0) + (scene.num_cars as f64 * 1.5);
                    let realistic_speed = (base_speed - traffic_penalty).clamp(25.0, 65.0);
                    
                    let realistic_steering = if scene.num_cars > 6 { 
                        // More steering corrections in heavy traffic
                        (scene.num_cars as f64 - 6.0) * 2.0 
                    } else { 
                        1.0 
                    }.clamp(0.0, 12.0);
                    
                    let realistic_brake = if scene.distance_obstacle < 25.0 { 
                        (25.0 - scene.distance_obstacle) * 3.0 
                    } else { 0.0 };
                    
                    let realistic_acceleration = if scene.distance_obstacle > 15.0 { 
                        1.0 // Can accelerate when clear
                    } else if scene.distance_obstacle < 8.0 { 
                        -2.0 // Decelerate when close
                    } else { 
                        0.0 // Maintain speed
                    };

                    let manual_data = ManualCarData {
                        vehicle_speed: realistic_speed, // Real speed based on conditions
                        steering_angle: realistic_steering, // Real steering based on traffic
                        brake_force: realistic_brake, // Real braking based on obstacles
                        acceleration: realistic_acceleration, // Real acceleration based on distance
                        weather_condition: "clear".to_string(),
                        road_condition: "dry".to_string(),
                        driver_alertness: true, // assume driver is alert
                        throttle_position: (realistic_speed / 65.0 * 100.0).clamp(20.0, 80.0), // Realistic throttle
                        timestamp: current_time,
                        is_valid: true,
                    };

                    // Check subscriber count for better restart detection
                    let subscriber_count = writer.get_publication_matched_status()
                        .map(|status| status.current_count)
                        .unwrap_or(0);
                    
                    debug!("[DDS] Sending manual car data (subscribers: {}): {:?}", subscriber_count, manual_data);
                    match writer.write(&manual_data, None) {
                        Ok(_) => {
                            if subscriber_count > 0 {
                                debug!("✅ [DDS] Manual car data published successfully to {} subscriber(s)", subscriber_count);
                            } else {
                                debug!("✅ [DDS] Manual car data published and cached (no subscribers detected yet)");
                            }
                        },
                        Err(_) => {
                            // Silently handle - data is stored in TransientLocal durability anyway
                            // Subscribers will get it when they join
                            debug!("📝 [DDS] ManualCarData cached in TransientLocal (no active subscribers detected)");
                        },
                    }
                }
            } else {
                debug!("ManualModePublisher: No scene data available");
            }
        }

        sleep_random();
    }

    fn shutdown(&mut self) {
        info!("🔄 ManualModePublisher shutting down - cleaning up individual DDS participant");
        self.writer = None;
        self.participant = None; // Clean shutdown of individual participant
    }
}

/// Emergency Mode Publisher activity
///
/// This activity publishes comprehensive emergency driving data with DDS output
/// when the car is in emergency mode, including seatbelt tightening and safety features.
pub struct EmergencyModePublisher {
    activity_id: ActivityId,
    input_car_data: Box<dyn ActivityInput<CarData>>,
    input_scene: Box<dyn ActivityInput<Scene>>,
    writer: Option<DataWriter<EmergencyModeData>>,
    participant: Option<DomainParticipant>, // Own participant for clean restart
    discovery_counter: u32, // Counter for periodic subscriber re-discovery
}

impl EmergencyModePublisher {
    pub fn build(
        activity_id: ActivityId,
        car_data_topic: &str,
        scene_topic: &str,
        _emergency_data_topic: &str,
    ) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            input_car_data: activity_input(car_data_topic),
            input_scene: activity_input(scene_topic),
            writer: None,
            participant: None, // Will be created in startup
            discovery_counter: 0, // Initialize discovery counter
        })
    }
}

impl Activity for EmergencyModePublisher {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    fn startup(&mut self) {
        info!("🚨 EmergencyModePublisher started with INDIVIDUAL participant - publishes IMMEDIATELY when emergency mode is active");
        
        // Create individual DDS participant for this component - prevents shared state issues
        let participant = create_dds_participant();

        let topic = participant
            .create_topic::<EmergencyModeData>(
                "EmergencyModeData",
                "EmergencyModeData",
                QosKind::Default,
                None,
                &[],
            )
            .unwrap();

        let publisher = participant
            .create_publisher(QosKind::Default, None, &[])
            .unwrap();

       let writer_qos = dust_dds::infrastructure::qos::DataWriterQos {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort, // BestEffort doesn't wait for subscriber ACK
                max_blocking_time: dust_dds::infrastructure::time::DurationKind::Finite(
                    dust_dds::infrastructure::time::Duration::new(0, 100_000_000) // 100ms timeout
                ),
            },
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal, // Keep for late-joining subscribers
            },
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast(5), // Reduced to prevent memory buildup
            },
            ..Default::default()
        };
 

        let writer = publisher
            .create_datawriter::<EmergencyModeData>(
                &topic,
                QosKind::Specific(writer_qos),
                None,
                &[],
            )
            .unwrap();

        self.writer = Some(writer);
        self.participant = Some(participant); // Store for clean shutdown
        
        // Longer initial discovery time to ensure subscribers are detected after restarts
        info!("🚨 EmergencyModePublisher waiting for subscriber discovery...");
        thread::sleep(Duration::from_millis(200));
        info!("🚨 EmergencyModePublisher discovery period complete");
    }

    fn step(&mut self) {
        // Increment discovery counter for periodic subscriber re-discovery
        self.discovery_counter += 1;
        
        // Every 100 steps (~10 seconds), add a brief pause to help with subscriber discovery after restarts
        if self.discovery_counter % 100 == 0 {
            debug!("🚨 EmergencyModePublisher: Periodic discovery refresh (step {})", self.discovery_counter);
            thread::sleep(Duration::from_millis(50)); // Brief pause for discovery refresh
        }
        
        // Try to read both inputs
        let car_data_result = self.input_car_data.read();
        let scene_result = self.input_scene.read();
        
        // Check if we're in emergency mode either from car_data or by calculating from scene
        let is_emergency_mode = if let Ok(car_data) = &car_data_result {
            car_data.driving_mode == "emergency"
        } else if let Ok(scene) = &scene_result {
            // Calculate mode directly from scene if car_data is not available
            scene.distance_obstacle < 4.0 // emergency if very close obstacle
        } else {
            false
        };
        
        if is_emergency_mode {
            info!("🚨 EMERGENCY MODE ACTIVE - Publishing detailed safety data immediately (no delay)");
            
            // Use scene data (required) and car_data (optional)
            if let Ok(scene) = scene_result {
                if let Some(writer) = &mut self.writer {
                    let current_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as i64;

                    // Calculate realistic emergency values based on actual scene conditions
                    let emergency_speed = if scene.distance_obstacle < 2.0 { 
                        5.0 // Nearly stopped for imminent collision
                    } else if scene.distance_obstacle < 3.0 { 
                        15.0 // Very slow emergency speed
                    } else { 
                        25.0 // Reduced emergency speed
                    };
                    
                    // Emergency steering - wider range for evasive maneuvers
                    let emergency_steering = if scene.num_people > 0 && scene.distance_obstacle < 5.0 {
                        // Evasive steering to avoid pedestrians
                        if scene.distance_left_lane > scene.distance_right_lane { -20.0 } else { 20.0 }
                    } else if scene.distance_obstacle < 3.0 {
                        // Sharp steering for obstacle avoidance
                        if scene.distance_left_lane > scene.distance_right_lane { -15.0 } else { 15.0 }
                    } else {
                        // Moderate steering correction
                        if scene.distance_left_lane > scene.distance_right_lane { -8.0 } else { 8.0 }
                    };

                    // Calculate emergency parameters based on scene
                    let collision_risk = if scene.distance_obstacle < 5.0 { 
                        100.0 
                    } else { 
                        (10.0 - scene.distance_obstacle) * 10.0 
                    };
                    
                    let emergency_brake_force = if scene.distance_obstacle < 3.0 { 
                        100.0 
                    } else { 
                        (5.0 - scene.distance_obstacle) * 20.0 
                    };

                    let emergency_type = if scene.num_people > 0 && scene.distance_obstacle < 5.0 {
                        "collision_avoidance".to_string()
                    } else if scene.distance_obstacle < 3.0 {
                        "obstacle".to_string()
                    } else {
                        "system_failure".to_string()
                    };

                    let emergency_data = EmergencyModeData {
                        vehicle_speed: emergency_speed, // Real emergency speed based on danger level
                        steering_angle: emergency_steering, // Real emergency steering based on obstacles
                        brake_force: emergency_brake_force.max(0.0).min(100.0),
                        obstacle_detected: true,
                        obstacle_distance: scene.distance_obstacle,
                        collision_risk: collision_risk.max(0.0).min(100.0),
                        stability_control: true, // always active in emergency
                        traffic_signal: "emergency".to_string(), // emergency override
                        seatbelt_tightened: true, // ⚠️ SEATBELT TIGHTENING ACTIVATED
                        emergency_lights: true, // hazard lights on
                        emergency_type,
                        emergency_brake_force: emergency_brake_force.max(0.0).min(100.0),
                        airbag_ready: true, // airbag systems primed
                        timestamp: current_time,
                        is_valid: true,
                    };

                    debug!("[DDS] 🚨 EMERGENCY MODE: Seatbelts tightened, airbags ready, emergency braking: {:.1}%", 
                           emergency_data.emergency_brake_force);
                    
                    // Check subscriber count for better restart detection
                    let subscriber_count = writer.get_publication_matched_status()
                        .map(|status| status.current_count)
                        .unwrap_or(0);
                    
                    debug!("[DDS] Sending emergency car data (subscribers: {}): {:?}", subscriber_count, emergency_data);
                    match writer.write(&emergency_data, None) {
                        Ok(_) => {
                            if subscriber_count > 0 {
                                debug!("✅ [DDS] Emergency car data published successfully to {} subscriber(s)", subscriber_count);
                            } else {
                                debug!("✅ [DDS] Emergency car data published and cached (no subscribers detected yet)");
                            }
                        },
                        Err(_) => {
                            // Silently handle - data is stored in TransientLocal durability anyway
                            // Subscribers will get it when they join
                            debug!("📝 [DDS] EmergencyModeData cached in TransientLocal (no active subscribers detected)");
                        },
                    }
                }
            } else {
                warn!("🚨 Emergency mode active but no scene data available for emergency publishing");
            }
        }

        sleep_random();
    }

    fn shutdown(&mut self) {
        info!("🔄 EmergencyModePublisher shutting down - cleaning up individual DDS participant");
        self.writer = None;
        self.participant = None; // Clean shutdown of individual participant
    }
}

/// Create an activity input.
fn activity_input<T>(topic: &str) -> Box<dyn ActivityInput<T>>
where
    T: fmt::Debug + 'static,
{
    #[cfg(feature = "com_iox2")]
    return Box::new(Iox2Input::new(topic));
    #[cfg(feature = "com_linux_shm")]
    return Box::new(LinuxShmInput::new(topic));
}

/// Create an activity output.
fn activity_output<T>(topic: &str) -> Box<dyn ActivityOutput<T>>
where
    T: fmt::Debug + 'static,
{
    #[cfg(feature = "com_iox2")]
    return Box::new(Iox2Output::new(topic));
    #[cfg(feature = "com_linux_shm")]
    return Box::new(LinuxShmOutput::new(topic));
}

/// Generate a pseudo-random number in the specified range.
fn gen_random_in_range(range: Range<i64>) -> i64 {
    let rand = RandomState::new().build_hasher().finish();
    let rand = (rand % (i64::MAX as u64)) as i64;
    rand % (range.end - range.start + 1) + range.start
}

/// Random walk from `previous` with a probability of `change_prop` in a range of +/-`max_delta`
fn random_walk_float(previous: f64, change_prop: f64, max_delta: f64) -> f64 {
    if gen_random_in_range(0..100) as f64 / 100.0 < change_prop {
        const SCALE_FACTOR: f64 = 1000.0;

        // Scale delta to work in integers
        let scaled_max_delta = (max_delta * SCALE_FACTOR) as i64;
        let scaled_delta = gen_random_in_range(-scaled_max_delta..scaled_max_delta) as f64;

        return previous + (scaled_delta / SCALE_FACTOR);
    }

    previous
}

/// Random walk from `previous` with a probability of `change_prop` in a range of +/-`max_delta`
fn random_walk_integer(previous: usize, change_prop: f64, max_delta: usize) -> usize {
    let max_delta = max_delta as i64;

    if gen_random_in_range(0..100) as f64 / 100.0 < change_prop {
        let delta = gen_random_in_range(-max_delta..max_delta);

        return i64::max(0, previous as i64 + delta) as usize;
    }

    previous
}

/// Sleep for a random amount of time
fn sleep_random() {
    thread::sleep(Duration::from_millis(
        gen_random_in_range(SLEEP_RANGE) as u64
    ));
}
