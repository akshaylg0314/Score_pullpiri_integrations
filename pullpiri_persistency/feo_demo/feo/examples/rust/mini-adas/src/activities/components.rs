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
use std::sync::{Arc, OnceLock};
use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::qos::QosKind,
    publication::data_writer::DataWriter,
    domain::domain_participant::DomainParticipant,
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
use feo_tracing::instrument;
use std::hash::RandomState;
use std::thread;

// Shared DDS domain participant to prevent memory corruption
static DDS_PARTICIPANT: OnceLock<Arc<DomainParticipant>> = OnceLock::new();

fn get_dds_participant() -> Arc<DomainParticipant> {
    DDS_PARTICIPANT.get_or_init(|| {
        let factory = DomainParticipantFactory::get_instance();
        let participant = factory
            .create_participant(100, QosKind::Default, None, &[])
            .expect("Failed to create DDS participant");
        Arc::new(participant)
    }).clone()
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

    // Local state for pseudo-random output generation
    num_people: usize,
    num_cars: usize,
    distance_obstacle: f64,
}

impl Camera {
    pub fn build(activity_id: ActivityId, image_topic: &str) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            output_image: activity_output(image_topic),
            num_people: 4,
            num_cars: 10,
            distance_obstacle: 40.0,
        })
    }

    fn get_image(&mut self) -> CameraImage {
        const PEOPLE_CHANGE_PROP: f64 = 0.8;
        const CAR_CHANGE_PROP: f64 = 0.8;
        const DISTANCE_CHANGE_PROP: f64 = 1.0;

        self.num_people = random_walk_integer(self.num_people, PEOPLE_CHANGE_PROP, 1);
        self.num_cars = random_walk_integer(self.num_people, CAR_CHANGE_PROP, 2);
        let sample = random_walk_float(self.distance_obstacle, DISTANCE_CHANGE_PROP, 5.0);
        self.distance_obstacle = sample.clamp(20.0, 50.0);

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
            distance_obstacle: 40.0,
        })
    }

    fn get_scan(&mut self) -> RadarScan {
        const DISTANCE_CHANGE_PROP: f64 = 1.0;

        let sample = random_walk_float(self.distance_obstacle, DISTANCE_CHANGE_PROP, 6.0);
        self.distance_obstacle = sample.clamp(16.0, 60.0);

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

    // Local state for mode calculation
    current_mode: String,
    obstacle_threshold: f64,
    emergency_threshold: f64,
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
            obstacle_threshold: 10.0, // meters
            emergency_threshold: 5.0, // meters
        })
    }
}

impl Activity for CarModeCalculator {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "CarModeCalculator startup")]
    fn startup(&mut self) {
        info!("🚗 CarModeCalculator starting up - autonomous mode for safe conditions, manual for complex traffic");
    }

    #[instrument(name = "CarModeCalculator step")]
    fn step(&mut self) {
        if let Ok(scene) = self.input_scene.read() {
            debug!("CarModeCalculator processing scene: people={}, cars={}, obstacle_distance={}", 
                   scene.num_people, scene.num_cars, scene.distance_obstacle);

            // Determine driving mode based on scene conditions
            // Autonomous mode for safe conditions, manual for complex traffic
            let new_mode = if scene.distance_obstacle < self.emergency_threshold {
                "emergency".to_string()
            } else if scene.distance_obstacle < self.obstacle_threshold 
                || scene.num_people > 2 
                || scene.num_cars > 3 {
                "manual".to_string()    // Complex traffic - driver control
            } else {
                "autonomous".to_string()  // Safe conditions - system takes control
            };

            // Only update if mode changed
            if new_mode != self.current_mode {
                info!("🚦 CarModeCalculator mode change: {} -> {}", self.current_mode, new_mode);
                self.current_mode = new_mode.clone();
            }

            // Create and publish car data
            if let Ok(car_data_output) = self.output_car_data.write_uninit() {
                let car_data = CarData {
                    driving_mode: self.current_mode.clone(),
                };
                
                let car_data_output = car_data_output.write_payload(car_data);
                car_data_output.send().unwrap();
                info!("📤 CarModeCalculator published mode: {} (distance: {:.1}m, people: {}, cars: {})", 
                    self.current_mode, scene.distance_obstacle, scene.num_people, scene.num_cars);
            }
        } else {
            debug!("CarModeCalculator: No scene data available");
        }

        sleep_random();
    }

    #[instrument(name = "CarModeCalculator shutdown")]
    fn shutdown(&mut self) {}
}

/// Car Data Publisher activity
///
/// This activity publishes CarData to DDS for external systems like pullpiri
/// to receive the current driving mode.
pub struct CarDataPublisher {
    activity_id: ActivityId,
    input_car_data: Box<dyn ActivityInput<CarData>>,
    writer: Option<DataWriter<CarData>>,
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
        })
    }
}

impl Activity for CarDataPublisher {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    fn startup(&mut self) {
        info!("🚀 CarDataPublisher started, initializing DDS for pullpiri integration...");
        
        // Use shared DDS participant to prevent memory corruption
        let participant = get_dds_participant();

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

        let writer = publisher
            .create_datawriter::<CarData>(
                &topic,
                QosKind::Default,
                None,
                &[],
            )
            .unwrap();

        self.writer = Some(writer);
        thread::sleep(Duration::from_millis(100)); // minimal discovery time
        info!("✅ CarDataPublisher DDS setup complete - ready to publish CarData");
    }

    fn step(&mut self) {
        if let Ok(car_data) = self.input_car_data.read() {
            info!("📡 Publishing CarData to DDS for pullpiri: mode = {}", car_data.driving_mode);

            if let Some(writer) = &mut self.writer {
                let dds_car_data = CarData {
                    driving_mode: car_data.driving_mode.clone(),
                };

                info!("🌐 [DDS] Sending CarData for pullpiri integration: {:?}", dds_car_data);
                writer.write(&dds_car_data, None).unwrap();
                info!("✅ [DDS] CarData successfully published to topic 'CarData'");
            }
        } else {
            debug!("CarDataPublisher: No car data available to publish");
        }

        sleep_random();
    }

    fn shutdown(&mut self) {
        self.writer = None;
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
        })
    }
}

impl Activity for AutonomousModePublisher {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    fn startup(&mut self) {
        debug!("AutonomousModePublisher started, initializing DDS...");
        
        // Use shared DDS participant to prevent memory corruption
        let participant = get_dds_participant();

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

        let writer = publisher
            .create_datawriter::<AutonomousCarData>(
                &topic,
                QosKind::Default,
                None,
                &[],
            )
            .unwrap();

        self.writer = Some(writer);
        thread::sleep(Duration::from_millis(100)); // minimal discovery time
    }

    fn step(&mut self) {
        if let (Ok(car_data), Ok(scene)) = (self.input_car_data.read(), self.input_scene.read()) {
            if car_data.driving_mode == "autonomous" {
                debug!("Publishing autonomous car data");

                if let Some(writer) = &mut self.writer {
                    let current_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as i64;

                    let autonomous_data = AutonomousCarData {
                        vehicle_speed: gen_random_in_range(40..80) as f64, // km/h
                        lane_position: gen_random_in_range(-50..50) as f64 / 100.0, // -0.5 to 0.5
                        obstacle_detected: scene.distance_obstacle < 30.0,
                        obstacle_distance: scene.distance_obstacle,
                        traffic_signal: if scene.num_cars > 5 { "red".to_string() } else { "green".to_string() },
                        steering_angle: gen_random_in_range(-10..10) as f64, // degrees
                        brake_force: if scene.distance_obstacle < 20.0 { 
                            (30.0 - scene.distance_obstacle) * 5.0 
                        } else { 0.0 },
                        acceleration: if scene.distance_obstacle > 30.0 { 2.0 } else { -1.0 },
                        weather_condition: "clear".to_string(),
                        road_condition: "dry".to_string(),
                        timestamp: current_time,
                        is_valid: true,
                    };

                    debug!("[DDS] Sending autonomous car data: {:?}", autonomous_data);
                    writer.write(&autonomous_data, None).unwrap();
                }
            }
        }

        sleep_random();
    }

    fn shutdown(&mut self) {
        self.writer = None;
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
        })
    }
}

impl Activity for ManualModePublisher {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    fn startup(&mut self) {
        debug!("ManualModePublisher started, initializing DDS...");
        
        // Use shared DDS participant to prevent memory corruption
        let participant = get_dds_participant();

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

        let writer = publisher
            .create_datawriter::<ManualCarData>(
                &topic,
                QosKind::Default,
                None,
                &[],
            )
            .unwrap();

        self.writer = Some(writer);
        thread::sleep(Duration::from_millis(100)); // minimal discovery time
    }

    fn step(&mut self) {
        if let (Ok(car_data), Ok(scene)) = (self.input_car_data.read(), self.input_scene.read()) {
            if car_data.driving_mode == "manual" {
                debug!("Publishing manual car data");

                if let Some(writer) = &mut self.writer {
                    let current_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as i64;

                    let manual_data = ManualCarData {
                        vehicle_speed: gen_random_in_range(30..70) as f64, // km/h
                        steering_angle: gen_random_in_range(-15..15) as f64, // degrees
                        brake_force: if scene.distance_obstacle < 25.0 { 
                            (25.0 - scene.distance_obstacle) * 3.0 
                        } else { 0.0 },
                        acceleration: gen_random_in_range(-2..3) as f64, // m/s²
                        weather_condition: "clear".to_string(),
                        road_condition: "dry".to_string(),
                        driver_alertness: true, // assume driver is alert
                        throttle_position: gen_random_in_range(20..80) as f64, // percentage
                        timestamp: current_time,
                        is_valid: true,
                    };

                    debug!("[DDS] Sending manual car data: {:?}", manual_data);
                    writer.write(&manual_data, None).unwrap();
                }
            }
        }

        sleep_random();
    }

    fn shutdown(&mut self) {
        self.writer = None;
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
        })
    }
}

impl Activity for EmergencyModePublisher {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    fn startup(&mut self) {
        debug!("EmergencyModePublisher started, initializing DDS...");
        
        // Use shared DDS participant to prevent memory corruption
        let participant = get_dds_participant();

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

        let writer = publisher
            .create_datawriter::<EmergencyModeData>(
                &topic,
                QosKind::Default,
                None,
                &[],
            )
            .unwrap();

        self.writer = Some(writer);
        thread::sleep(Duration::from_millis(100)); // minimal discovery time
    }

    fn step(&mut self) {
        if let (Ok(car_data), Ok(scene)) = (self.input_car_data.read(), self.input_scene.read()) {
            if car_data.driving_mode == "emergency" {
                debug!("Publishing emergency car data with safety features activated");

                if let Some(writer) = &mut self.writer {
                    let current_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as i64;

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
                        vehicle_speed: gen_random_in_range(10..40) as f64, // km/h - reduced speed
                        steering_angle: gen_random_in_range(-25..25) as f64, // degrees - wider range for evasion
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
                    debug!("[DDS] Sending emergency car data: {:?}", emergency_data);
                    writer.write(&emergency_data, None).unwrap();
                }
            }
        }

        sleep_random();
    }

    fn shutdown(&mut self) {
        self.writer = None;
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
