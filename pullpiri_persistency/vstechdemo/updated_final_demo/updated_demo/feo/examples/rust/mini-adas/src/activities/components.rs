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

use crate::activities::messages::{
    BrakeInstruction, CameraImage, DashboardData, DistractionMonitor, DmsState, 
    LaneChangeCommand, LkasState, RadarScan, Scene, Steering,
    str_to_fixed, fixed_to_string
};
use core::fmt;
use core::hash::{BuildHasher as _, Hasher as _};
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut, Range};
use core::time::Duration;
use feo::activity::Activity;
use feo::ids::ActivityId;
use feo_com::interface::{ActivityInput, ActivityOutput};
#[cfg(feature = "com_iox2")]
use feo_com::iox2::{Iox2Input, Iox2Output};
#[cfg(feature = "com_linux_shm")]
use feo_com::linux_shm::{LinuxShmInput, LinuxShmOutput};
use feo_log::{debug, info};
use feo_tracing::instrument;
use rust_kvs::prelude::*;
use std::hash::RandomState;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

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
    fn shutdown(&mut self) {
        debug!("Shutting down Camera activity {}", self.activity_id);
    }
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
    fn shutdown(&mut self) {
        debug!("Shutting down Radar activity {}", self.activity_id);
    }
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
    fn shutdown(&mut self) {
        debug!("Shutting down NeuralNet activity {}", self.activity_id);
    }
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
    fn shutdown(&mut self) {
        debug!(
            "Shutting down EnvironmentRenderer activity {}",
            self.activity_id
        );
    }
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
    fn shutdown(&mut self) {
        debug!(
            "Shutting down SteeringController activity {}",
            self.activity_id
        );
    }
}

pub struct VehiclePublisher {
    activity_id: ActivityId,
    input_scene: Box<dyn ActivityInput<Scene>>,
    input_steering: Box<dyn ActivityInput<Steering>>,
    input_dms: Box<dyn ActivityInput<DmsState>>,
    input_lkas: Box<dyn ActivityInput<LkasState>>,
    writer: Option<DataWriter<DashboardData>>,
    participant: Option<DomainParticipant>,
    // Internal state for vehicle simulation
    vehicle_speed: f64,        // km/h
    current_lane: i32,         // 1-4
    last_obstacle_distance: f64, // meters - for smooth transitions
}

impl VehiclePublisher {
    pub fn build(
        activity_id: ActivityId,
        scene_topic: &str,
        steering_topic: &str,
        dms_topic: &str,
        lkas_topic: &str,
    ) -> Box<dyn Activity> {
        info!("🤖 DashboardPublisher initializing with INDIVIDUAL DDS participant");
        
        Box::new(Self {
            activity_id,
            input_scene: activity_input(scene_topic),
            input_steering: activity_input(steering_topic),
            input_dms: activity_input(dms_topic),
            input_lkas: activity_input(lkas_topic),
            writer: None,
            participant: None,
            vehicle_speed: 60.0,
            current_lane: 2,  // Start in lane 2
            last_obstacle_distance: 50.0,
        })
    }
}

impl Activity for VehiclePublisher {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    fn startup(&mut self) {
        info!("🤖 DashboardPublisher started - creating DDS writer for DashboardData");
        
        // Create individual DDS participant for this component - prevents shared state issues  
        let participant = create_dds_participant();

        let topic = participant
            .create_topic::<DashboardData>(
                "DashboardData",
                "DashboardData",
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
                kind: ReliabilityQosPolicyKind::BestEffort,
                max_blocking_time: dust_dds::infrastructure::time::DurationKind::Finite(
                    dust_dds::infrastructure::time::Duration::new(0, 100_000_000)
                ),
            },
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal,
            },
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast(5),
            },
            ..Default::default()
        };
 

        let writer = publisher
            .create_datawriter::<DashboardData>(
                &topic,
                QosKind::Specific(writer_qos),
                None,
                &[],
            )
            .unwrap();

        self.writer = Some(writer);
        self.participant = Some(participant);
        
        info!("✅ DashboardPublisher DDS writer ready");
        thread::sleep(Duration::from_millis(200));
    }

    fn step(&mut self) {
        debug!("Stepping DashboardPublisher");
        sleep_random();

        debug!("📍 Reading scene...");
        let scene = self.input_scene.read();
        debug!("📍 Scene read complete");
        
        debug!("📍 Reading steering...");
        let steering = self.input_steering.read();
        debug!("📍 Steering read complete");
        
        debug!("📍 Reading DMS...");
        let dms = self.input_dms.read();
        debug!("📍 DMS read complete");
        
        debug!("📍 Reading LKAS...");
        let lkas = self.input_lkas.read();
        debug!("📍 LKAS read complete");

        if let Some(ref writer) = self.writer {
            debug!("📍 Creating DashboardData...");
            // Create DashboardData from available inputs
            let mut dashboard_data = DashboardData::default();
            debug!("📍 DashboardData created with default()");
            
            // Get current obstacle distance (default to far if no scene data)
            let obstacle_distance = if let Ok(ref scene_data) = scene {
                scene_data.distance_obstacle
            } else {
                50.0 // Default to clear road
            };
            
            // Smooth the obstacle distance to prevent sudden changes
            self.last_obstacle_distance = self.last_obstacle_distance * 0.7 + obstacle_distance * 0.3;

            // Update from scene data if available
            if let Ok(scene_data) = scene {
                dashboard_data.obstacle_detected = scene_data.distance_obstacle < 50.0;
                dashboard_data.obstacle_distance = scene_data.distance_obstacle;
            }
            debug!("📍 Scene data updated");

            // Update from LKAS data if available
            if let Ok(lkas_data) = lkas {
                self.current_lane = lkas_data.current_lane;
                dashboard_data.current_lane = lkas_data.current_lane;
                dashboard_data.lane_position_offset = lkas_data.lane_position_offset;
                dashboard_data.steering_angle = lkas_data.steering_angle; // Use LKAS steering
                debug!("📍 Before converting lkas_status");
                dashboard_data.lkas_status = fixed_to_string(&lkas_data.lkas_status);
                debug!("📍 After converting lkas_status");
                dashboard_data.left_lane_distance = lkas_data.left_lane_distance;
                dashboard_data.right_lane_distance = lkas_data.right_lane_distance;
            } else {
                // Fallback if LKAS not available yet
                dashboard_data.current_lane = self.current_lane;
                dashboard_data.lane_position_offset = 0.0;
                dashboard_data.steering_angle = 0.0;
                dashboard_data.lkas_status = "Active".to_string();
                dashboard_data.left_lane_distance = 1.75;
                dashboard_data.right_lane_distance = 1.75;
            }
            debug!("📍 LKAS data updated");

            // Update from DMS data if available
            debug!("📍 Checking DMS data availability");
            if let Ok(dms_data) = dms {
                debug!("📍 DMS data is OK, starting field copy");
                dashboard_data.distraction_duration = dms_data.distraction_duration as f64 / 1000.0;  // Convert ms to seconds
                debug!("📍 Copied distraction_duration");
                dashboard_data.gaze_direction = fixed_to_string(&dms_data.gaze_direction);
                debug!("📍 Copied gaze_direction");
                dashboard_data.head_yaw = dms_data.head_yaw;
                dashboard_data.head_pitch = dms_data.head_pitch;
                dashboard_data.head_roll = dms_data.head_roll;
                dashboard_data.drowsiness_score = dms_data.drowsiness_score;
                dashboard_data.attention_level = dms_data.attention_level;
                debug!("📍 Before cloning driver_status");
                dashboard_data.driver_status = fixed_to_string(&dms_data.driver_status);
                debug!("📍 Copied driver_status");
                // Note: Pullover logic moved to LaneKeepAssistSystem
            } else {
                debug!("📍 DMS data not available, using fallback");
                // Fallback if DMS not available yet - initialize ALL fields
                dashboard_data.distraction_duration = 0.0;  // f64 seconds
                dashboard_data.gaze_direction = "Forward".to_string();
                dashboard_data.head_yaw = 0.0;
                dashboard_data.head_pitch = 0.0;
                dashboard_data.head_roll = 0.0;
                dashboard_data.drowsiness_score = 10.0;
                dashboard_data.attention_level = 95.0;
                dashboard_data.driver_status = "Active".to_string();
            }
            debug!("📍 DMS data section completed");

            // Vehicle Physics & Speed Control
            const MAX_SPEED: f64 = 80.0;
            const ACCEL_RATE: f64 = 1.0;
            const DECEL_RATE: f64 = 2.0;
            const NATURAL_DECEL: f64 = 0.5;
            const EMERGENCY_BRAKE_DECEL: f64 = 10.0;

            if dashboard_data.lkas_status == "PullingOver" {
                // Decelerate to stop
                if self.vehicle_speed > 0.0 {
                    self.vehicle_speed = (self.vehicle_speed - DECEL_RATE).max(0.0);
                }
            } else {
                // Normal driving or Lane Change
                let should_change_lane = dashboard_data.lkas_status == "LaneChange";
                // We assume the obstacle is primarily in Lane 2. If we are not in Lane 2, we are safe.
                let is_safe_from_obstacle = self.last_obstacle_distance > 35.0 || self.current_lane != 2;

                if should_change_lane {
                     // Maintain or slight accel
                     if self.vehicle_speed < MAX_SPEED * 0.8 {
                        self.vehicle_speed = (self.vehicle_speed + ACCEL_RATE * 0.8).min(MAX_SPEED * 0.8);
                     }
                } else if is_safe_from_obstacle {
                    // Accelerate
                    if self.vehicle_speed < MAX_SPEED {
                        self.vehicle_speed = (self.vehicle_speed + ACCEL_RATE).min(MAX_SPEED);
                    } else {
                         self.vehicle_speed = (self.vehicle_speed - NATURAL_DECEL * 0.3).max(0.0);
                    }
                } else if self.last_obstacle_distance > 25.0 {
                    // Moderate distance
                    if self.vehicle_speed < MAX_SPEED * 0.6 {
                         self.vehicle_speed = (self.vehicle_speed + ACCEL_RATE * 0.5).min(MAX_SPEED * 0.6);
                    } else {
                         self.vehicle_speed = (self.vehicle_speed - NATURAL_DECEL).max(0.0);
                    }
                } else if self.last_obstacle_distance > 15.0 {
                    // Brake
                    if self.vehicle_speed > 0.0 {
                        self.vehicle_speed = (self.vehicle_speed - 3.0).max(0.0);
                    }
                } else if self.last_obstacle_distance > 10.0 {
                    // Strong Brake
                     if self.vehicle_speed > 0.0 {
                        self.vehicle_speed = (self.vehicle_speed - 6.0).max(0.0);
                    }
                } else {
                    // Emergency Brake
                    if self.vehicle_speed > 0.0 {
                        self.vehicle_speed = (self.vehicle_speed - EMERGENCY_BRAKE_DECEL).max(0.0);
                    }
                }
            }
            
            // Update from steering data if available (override if needed, but we prefer LKAS steering)
            // If LKAS is active (LaneChange/PullingOver), use LKAS steering.
            // If manual driving (Active), maybe use steering input?
            // For now, LKAS steering is authoritative for the demo.
            
            // Set vehicle metrics
            dashboard_data.vehicle_speed = self.vehicle_speed;
            dashboard_data.obstacle_distance = self.last_obstacle_distance;
            dashboard_data.timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;
            dashboard_data.is_valid = true;

            // Debug: Log before attempting DDS write
            debug!("📋 Attempting to publish DashboardData: lane={}, gaze={}, lkas_status={}", 
                   dashboard_data.current_lane,
                   dashboard_data.gaze_direction,
                   dashboard_data.lkas_status);

            // Publish dashboard data
            match writer.write(&dashboard_data, None) {
                Ok(_) => {
                    debug!("📡 Published DashboardData: speed={:.1}km/h, lane={}, distraction={:.1}s, status={}", 
                           dashboard_data.vehicle_speed,
                           dashboard_data.current_lane,
                           dashboard_data.distraction_duration,
                           dashboard_data.driver_status);
                },
                Err(e) => {
                    debug!("Failed to publish DashboardData: {:?}", e);
                }
            }
        }
    }

    fn shutdown(&mut self) {
        info!("🔄 DashboardPublisher shutting down - cleaning up individual DDS participant");
        self.writer = None;
        self.participant = None;
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

/// Emergency braking activity
///
/// This component emulates an emergency braking function
/// which sends instructions to activate the brakes
/// if the distance to the closest obstacle becomes too small.
/// The level of brake engagement depends on the distance.
pub struct EmergencyBraking {
    /// ID of the activity
    activity_id: ActivityId,
    /// Scene input
    input_scene: Box<dyn ActivityInput<Scene>>,
    /// Brake instruction output
    output_brake_instruction: Box<dyn ActivityOutput<BrakeInstruction>>,
    /// KVS instance for persistency
    kvs: Option<Kvs>,
    /// Last time data was persisted
    last_persist_time: SystemTime,
    /// Emergency braking statistics
    total_activations: u64,
    last_activation_time: Option<SystemTime>,
}

impl EmergencyBraking {
    pub fn build(
        activity_id: ActivityId,
        scene_topic: &str,
        brake_instruction_topic: &str,
    ) -> Box<dyn Activity> {
        // Initialize KVS for persistency
        info!("🔧 Initializing EmergencyBraking with persistency support");
        
        // Ensure directory exists
        if let Err(e) = std::fs::create_dir_all(crate::config::ADAS_DATA_DIR) {
            info!("❌ Failed to create directory {}: {:?}", crate::config::ADAS_DATA_DIR, e);
        } else {
            info!("✅ Directory {} created/verified", crate::config::ADAS_DATA_DIR);
        }
        
        let kvs = match KvsBuilder::new(InstanceId(0))
            .dir(crate::config::ADAS_DATA_DIR)
            .kvs_load(KvsLoad::Optional)
            .build() {
            Ok(kvs_instance) => {
                info!("✅ KVS successfully initialized for EmergencyBraking");
                Some(kvs_instance)
            },
            Err(e) => {
                info!("❌ Failed to create KVS: {:?}", e);
                None
            }
        };

        Box::new(Self {
            activity_id,
            input_scene: activity_input(scene_topic),
            output_brake_instruction: activity_output(brake_instruction_topic),
            kvs,
            last_persist_time: SystemTime::now(),
            total_activations: 0,
            last_activation_time: None,
        })
    }

    fn persist_data(&mut self, current_time: SystemTime, current_status_active: bool) {
        if let Some(ref kvs) = self.kvs {
            // Format timestamp for JSON
            let timestamp = current_time
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let last_activation_timestamp = self.last_activation_time
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            // Create JSON data structure
            let json_data = format!(
                r#"{{
    "timestamp": {},
    "emergency_braking_events": {{
        "total_activations": {},
        "last_activation": {},
        "current_status": "{}",
        "confidence_score": 0.98
    }}
}}"#,
                timestamp,
                self.total_activations,
                last_activation_timestamp,
                if current_status_active { "active" } else { "inactive" }
            );

            // Create timestamped key
            let key = format!("adas_emergency_data_{}", timestamp);

            info!("💾 Persisting emergency braking data: key={}, activations={}", key, self.total_activations);

            // Store in KVS
            if let Err(e) = kvs.set_value(key.clone(), json_data) {
                info!("❌ Failed to persist emergency braking data: {:?}", e);
            } else {
                // Flush to ensure data is written to disk
                if let Err(e) = kvs.flush() {
                    info!("❌ Failed to flush KVS data: {:?}", e);
                } else {
                    info!("✅ Successfully persisted and flushed emergency braking data at timestamp {}", timestamp);
                }
            }
        } else {
            info!("⚠️  KVS not initialized, skipping persistence");
        }
    }
}

impl fmt::Debug for EmergencyBraking {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EmergencyBraking")
            .field("activity_id", &self.activity_id)
            .field("kvs", &self.kvs.is_some())
            .field("last_persist_time", &self.last_persist_time)
            .field("total_activations", &self.total_activations)
            .field("last_activation_time", &self.last_activation_time)
            .finish()
    }
}

impl Activity for EmergencyBraking {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    #[instrument(name = "EmergencyBraking startup")]
    fn startup(&mut self) {
        debug!("EmergencyBraking component starting up with persistency");
    }

    #[instrument(name = "EmergencyBraking")]
    fn step(&mut self) {
        debug!("Stepping EmergencyBraking");
        sleep_random();

        let scene = self.input_scene.read();
        let brake_instruction = self.output_brake_instruction.write_uninit();

        if let (Ok(scene), Ok(brake_instruction)) = (scene, brake_instruction) {
            const ENGAGE_DISTANCE: f64 = 35.0;
            const MAX_BRAKE_DISTANCE: f64 = 15.0;

            let current_time = SystemTime::now();
            let emergency_activated = scene.distance_obstacle < ENGAGE_DISTANCE;
            
            debug!("EmergencyBraking: distance={}, threshold={}, emergency_activated={}", 
                   scene.distance_obstacle, ENGAGE_DISTANCE, emergency_activated);

            if emergency_activated {
                // Map distances ENGAGE_DISTANCE..MAX_BRAKE_DISTANCE to intensities 0.0..1.0
                let level = f64::min(
                    1.0,
                    (ENGAGE_DISTANCE - scene.distance_obstacle)
                        / (ENGAGE_DISTANCE - MAX_BRAKE_DISTANCE),
                );

                // Update statistics
                self.total_activations += 1;
                self.last_activation_time = Some(current_time);
                
                debug!("🚨 EMERGENCY BRAKING ACTIVATED! Level: {}, Total activations: {}", 
                       level, self.total_activations);

                let brake_instruction = brake_instruction.write_payload(BrakeInstruction {
                    active: true,
                    level,
                });
                brake_instruction.send().unwrap();
            } else {
                debug!("Emergency braking not needed (distance {} > {})", 
                       scene.distance_obstacle, ENGAGE_DISTANCE);
                let brake_instruction = brake_instruction.write_payload(BrakeInstruction {
                    active: false,
                    level: 0.0,
                });
                brake_instruction.send().unwrap();
            }

            // Persist data every 10 seconds
            let elapsed = current_time.duration_since(self.last_persist_time).unwrap_or_default();
            
            if elapsed >= Duration::from_secs(10) {
                info!("⏰ 10 seconds elapsed, persisting emergency braking data");
                self.persist_data(current_time, emergency_activated);
                self.last_persist_time = current_time;
            }
        }
    }

    #[instrument(name = "EmergencyBraking shutdown")]
    fn shutdown(&mut self) {
        debug!("Shutting down EmergencyBraking activity {}", self.activity_id);
    }
}

/// Brake controller activity
///
/// This activity emulates the braking system that receives
/// brake instructions from the EmergencyBraking component
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

/// Driver Monitoring System activity
///
/// Simulates driver monitoring with realistic distraction patterns
/// Tracks gaze, head pose, drowsiness, and attention level
pub struct DriverMonitoringSystem {
    activity_id: ActivityId,
    output_dms: Box<dyn ActivityOutput<DmsState>>,
    
    // Internal state
    distraction_duration: i64,      // Continuous distraction time (ms)
    gaze_direction: String,
    head_yaw: f64,
    head_pitch: f64,
    head_roll: f64,
    drowsiness_score: f64,
    attention_level: f64,
    is_distracted: bool,
    cycles_in_current_state: u32,
}

impl DriverMonitoringSystem {
    pub fn build(activity_id: ActivityId, dms_topic: &str) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            output_dms: activity_output(dms_topic),
            distraction_duration: 0,
            gaze_direction: "Forward".to_string(),
            head_yaw: 0.0,
            head_pitch: 0.0,
            head_roll: 0.0,
            drowsiness_score: 10.0,
            attention_level: 95.0,
            is_distracted: false,
            cycles_in_current_state: 0,
        })
    }
    
  fn update_state(&mut self) {
        const DISTRACTION_PROBABILITY: f64 = 0.70; // 70% chance to trigger distraction
      const MAX_DISTRACTION_MS: i64 = 15000;     // 15 seconds max
      const MIN_DISTRACTION_MS: i64 = 3000;      // 3 seconds min for distraction events
      const CYCLE_TIME_MS: i64 = 400;            // Approximate cycle time
      
      self.cycles_in_current_state += 1;
      
      // Decide state transitions
      if !self.is_distracted {
          // Currently attentive - check if should become distracted
          let rand_val = gen_random_in_range(0..100) as f64 / 100.0;
          
          if rand_val < DISTRACTION_PROBABILITY {
              // Start distraction
              self.is_distracted = true;
              self.distraction_duration = 0;
              self.cycles_in_current_state = 0;
              
              // Set distracted gaze
              let gaze_choice = gen_random_in_range(0..100);
              self.gaze_direction = if gaze_choice < 30 {
                  "Left".to_string()
              } else if gaze_choice < 60 {
                  "Right".to_string()
              } else {
                  "Down".to_string()  // Phone, controls, etc.
              };
              
              debug!("🚨 DMS: Driver became distracted (gaze: {})", self.gaze_direction);
          } else {
              // Stay attentive
              self.gaze_direction = "Forward".to_string();
              self.distraction_duration = 0;
          }
      } else {
          // Currently distracted - accumulate distraction time
          self.distraction_duration += CYCLE_TIME_MS;
          
          // Check if should return to attentive state
          // Reduced recovery chance to 2% per cycle to allow longer distractions
          let should_reset = self.distraction_duration >= MAX_DISTRACTION_MS ||
                            (self.distraction_duration > MIN_DISTRACTION_MS && 
                             gen_random_in_range(0..100) < 2); // 2% chance to recover after min duration
          
          if should_reset {
                // Return to attentive
                self.is_distracted = false;
                self.distraction_duration = 0;
                self.gaze_direction = "Forward".to_string();
                self.cycles_in_current_state = 0;
                debug!("✅ DMS: Driver returned to attentive state");
            }
        }
        
        // Update head pose with small random walks
        if self.is_distracted {
            // More head movement when distracted
            self.head_yaw = random_walk_float(self.head_yaw, 0.8, 3.0).clamp(-30.0, 30.0);
            self.head_pitch = random_walk_float(self.head_pitch, 0.8, 2.0).clamp(-20.0, 20.0);
            self.head_roll = random_walk_float(self.head_roll, 0.6, 1.5).clamp(-15.0, 15.0);
        } else {
            // Minimal head movement when attentive
            self.head_yaw = random_walk_float(self.head_yaw, 0.4, 1.0).clamp(-10.0, 10.0);
            self.head_pitch = random_walk_float(self.head_pitch, 0.4, 0.8).clamp(-8.0, 8.0);
            self.head_roll = random_walk_float(self.head_roll, 0.3, 0.5).clamp(-5.0, 5.0);
        }
        
        // Update drowsiness (increases during distraction, stays low otherwise)
        if self.is_distracted {
            self.drowsiness_score = (self.drowsiness_score + 0.5).clamp(0.0, 40.0);
        } else {
            self.drowsiness_score = (self.drowsiness_score - 1.0).clamp(5.0, 40.0);
        }
        
        // Update attention level (inverse of drowsiness + distraction)
        if self.is_distracted {
            self.attention_level = (100.0 - self.drowsiness_score - 
                                   (self.distraction_duration as f64 / 100.0)).clamp(30.0, 100.0);
        } else {
            self.attention_level = (100.0 - self.drowsiness_score).clamp(60.0, 100.0);
        }
    }
    
    fn get_driver_status(&self) -> String {
        if self.drowsiness_score > 30.0 {
            "Drowsy".to_string()
        } else if self.is_distracted {
            "Distracted".to_string()
        } else {
            "Active".to_string()
        }
    }
}

impl fmt::Debug for DriverMonitoringSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DriverMonitoringSystem")
            .field("activity_id", &self.activity_id)
            .field("distraction_duration", &self.distraction_duration)
            .field("is_distracted", &self.is_distracted)
            .finish()
    }
}

impl Activity for DriverMonitoringSystem {
    fn id(&self) -> ActivityId {
        self.activity_id
    }
    
    #[instrument(name = "DMS startup")]
    fn startup(&mut self) {
        info!("🤖 Driver Monitoring System started");
    }
    
    #[instrument(name = "DMS")]
    fn step(&mut self) {
        debug!("Stepping DMS");
        sleep_random();
        
        // Update internal state
        self.update_state();
        
        // Publish DMS state
        if let Ok(dms_output) = self.output_dms.write_uninit() {
            let state = DmsState {
                distraction_duration: self.distraction_duration,
                gaze_direction: str_to_fixed(&self.gaze_direction),
                head_yaw: self.head_yaw,
                head_pitch: self.head_pitch,
                head_roll: self.head_roll,
                drowsiness_score: self.drowsiness_score,
                attention_level: self.attention_level,
                driver_status: str_to_fixed(&self.get_driver_status()),
            };
            
            let driver_status_str = fixed_to_string(&state.driver_status);
            debug!("DMS: distraction={}ms, gaze={}, status={}", 
                   self.distraction_duration, self.gaze_direction, driver_status_str);
            
            let dms_output = dms_output.write_payload(state);
            dms_output.send().unwrap();
        }
    }
    
    #[instrument(name = "DMS shutdown")]
    fn shutdown(&mut self) {
        info!("Shutting down DMS activity {}", self.activity_id);
    }
}

/// Lane Keep Assistance System activity
///
/// Manages lane tracking and generates lane change commands
pub struct LaneKeepAssistSystem {
    activity_id: ActivityId,
    input_scene: Box<dyn ActivityInput<Scene>>,
    input_dms: Box<dyn ActivityInput<DmsState>>,
    output_lkas: Box<dyn ActivityOutput<LkasState>>,
    output_lane_change: Box<dyn ActivityOutput<LaneChangeCommand>>,
    
    // Internal state
    current_lane: i32,              // 1-4
    target_lane: i32,               // For gradual lane changes
    lane_position_offset: f64,
    steering_angle: f64,
    lkas_status: String,
    is_changing_lane: bool,
    pullover_requested: bool,
}

impl LaneKeepAssistSystem {
    pub fn build(
        activity_id: ActivityId, 
        scene_topic: &str,
        dms_topic: &str,
        lkas_topic: &str,
        lane_change_topic: &str
    ) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            input_scene: activity_input(scene_topic),
            input_dms: activity_input(dms_topic),
            output_lkas: activity_output(lkas_topic),
            output_lane_change: activity_output(lane_change_topic),
            current_lane: 2,  // Start in lane 2
            target_lane: 2,
            lane_position_offset: 0.0,
            steering_angle: 0.0,
            lkas_status: "Active".to_string(),
            is_changing_lane: false,
            pullover_requested: false,
        })
    }
    
    fn update_lane_position(&mut self) {
        // Gradual lane change
        if self.is_changing_lane {
            if self.current_lane < self.target_lane {
                self.current_lane += 1;
                info!("🚗 LKAS: Changed to lane {}", self.current_lane);
            } else if self.current_lane > self.target_lane {
                self.current_lane -= 1;
                info!("🚗 LKAS: Changed to lane {}", self.current_lane);
            }
            
            if self.current_lane == self.target_lane {
                self.is_changing_lane = false;
                if self.pullover_requested && self.current_lane == 4 {
                    self.lkas_status = "Standby".to_string();
                    info!("✅ LKAS: Pullover complete, vehicle in safe lane");
                } else {
                    self.lkas_status = "Active".to_string();
                }
            }
        }
        
        // Small random walk for lane position offset
        self.lane_position_offset = random_walk_float(
            self.lane_position_offset, 0.7, 0.1
        ).clamp(-0.3, 0.3);
    }
}

impl fmt::Debug for LaneKeepAssistSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LaneKeepAssistSystem")
            .field("activity_id", &self.activity_id)
            .field("current_lane", &self.current_lane)
            .field("lkas_status", &self.lkas_status)
            .finish()
    }
}

impl Activity for LaneKeepAssistSystem {
    fn id(&self) -> ActivityId {
        self.activity_id
    }
    
    #[instrument(name = "LKAS startup")]
    fn startup(&mut self) {
        info!("🤖 Lane Keep Assistance System started (lane {})", self.current_lane);
    }
    
    #[instrument(name = "LKAS")]
fn step(&mut self) {
    debug!("Stepping LKAS");
    sleep_random();
    
    // Read inputs
    let scene = self.input_scene.read();
    let dms = self.input_dms.read();
    
    // --- DECISION LOGIC ---
    
    // 1. Check Pullover Condition (High Priority)
    if let Ok(dms_data) = dms {
        // Distraction > 10s triggers pullover
        if dms_data.distraction_duration > 10000 && !self.pullover_requested {
            info!("🚨 LKAS: Pullover requested due to distraction > 10s");
            self.pullover_requested = true;
            self.target_lane = 3; // Target rightmost lane (Lane 3)
            self.lkas_status = "PullingOver".to_string();
            
            if self.current_lane != self.target_lane {
                self.is_changing_lane = true;
            }
        }
    }
    
    // 2. Check Obstacle Avoidance (Low Priority, only if not pulling over)
    if !self.pullover_requested && !self.is_changing_lane {
        if let Ok(scene_data) = scene {
            if scene_data.distance_obstacle < 35.0 {
                info!("⚠️ LKAS: Obstacle detected ({:.1}m), initiating lane change", scene_data.distance_obstacle);
                // Move Left (Lane - 1) to avoid, clamped to Lane 1
                let new_lane = (self.current_lane - 1).max(1);
                if new_lane != self.current_lane {
                    self.target_lane = new_lane;
                    self.is_changing_lane = true;
                    self.lkas_status = "LaneChange".to_string();
                }
            }
        }
    }
    
    // --- PHYSICS & EXECUTION ---
    
    if self.is_changing_lane {
        const LANE_WIDTH: f64 = 3.5;
        const LATERAL_SPEED: f64 = 0.4; // Meters per cycle (~1 m/s)
        
        // Determine direction
        // Lane 1 is Left, Lane 3 is Right
        // If target < current, moving Left (Negative offset)
        // If target > current, moving Right (Positive offset)
        
        let direction = if self.target_lane < self.current_lane { -1.0 } else { 1.0 };
        
        // Update offset
        // We simulate moving from 0.0 to +/- 3.5
        // Actually, we should track absolute lateral position or just relative offset accumulation
        // Simplification: We are moving 'away' from center of current lane towards the other lane
        // When offset exceeds LANE_WIDTH/2, we switch lane ID and flip offset
        
        // Let's use a simpler model:
        // We move lane_position_offset. When it reaches +/- 3.5, we snap to new lane.
        
        self.lane_position_offset += direction * LATERAL_SPEED;
        
        // Steering Angle Logic (Simple Model)
        // -15 deg for Left, +15 deg for Right
        self.steering_angle = direction * 15.0;
        
        // Check if lane change complete (crossed 3.5m threshold)
        if self.lane_position_offset.abs() >= LANE_WIDTH {
            info!("✅ LKAS: Lane change complete (Lane {} -> {})", self.current_lane, self.target_lane);
            self.current_lane = self.target_lane;
            self.lane_position_offset = 0.0;
            self.steering_angle = 0.0;
            
            // If pulling over and not yet at rightmost lane, keep going?
            // For now, assume single step lane changes.
            // If we need to go 1 -> 3, we do 1->2 then 2->3.
            
            if self.pullover_requested {
                if self.current_lane < 3 {
                    // Need to move further right
                    self.target_lane = 3;
                    self.is_changing_lane = true; // Keep changing
                    // Offset is 0, ready for next segment
                } else {
                    // Reached Lane 3
                    self.is_changing_lane = false;
                    // Keep status as PullingOver
                }
            } else {
                // Normal lane change done
                self.is_changing_lane = false;
                self.lkas_status = "Active".to_string();
            }
        }
    } else {
        // Lane Keeping - small random corrections
        self.steering_angle = random_walk_float(self.steering_angle, 0.5, 1.0).clamp(-2.0, 2.0);
        self.lane_position_offset = random_walk_float(self.lane_position_offset, 0.05, 0.2).clamp(-0.3, 0.3);
    }
    
    // Publish LKAS state
    if let Ok(lkas_output) = self.output_lkas.write_uninit() {
        let state = LkasState {
            current_lane: self.current_lane,
            lane_position_offset: self.lane_position_offset,
            steering_angle: self.steering_angle,
            lkas_status: str_to_fixed(&self.lkas_status),
            left_lane_distance: 1.75 - self.lane_position_offset,
            right_lane_distance: 1.75 + self.lane_position_offset,
        };
        
        debug!("LKAS: lane={}, status={}, offset={:.2}m, steer={:.1}", 
               self.current_lane, self.lkas_status, self.lane_position_offset, self.steering_angle);
        
        let lkas_output = lkas_output.write_payload(state);
        lkas_output.send().unwrap();
    }
    
    // Publish lane change commands if changing
    if self.is_changing_lane {
        if let Ok(cmd_output) = self.output_lane_change.write_uninit() {
            let cmd = LaneChangeCommand {
                target_lane: self.target_lane,
                reason: if self.pullover_requested { 
                    str_to_fixed("Pullover")
                } else { 
                    str_to_fixed("ObstacleAvoidance")
                },
                urgency: if self.pullover_requested { 1.0 } else { 0.7 },
            };
            let cmd_output = cmd_output.write_payload(cmd);
            cmd_output.send().unwrap();
        }
    }
}
    
    #[instrument(name = "LKAS shutdown")]
    fn shutdown(&mut self) {
        info!("Shutting down LKAS activity {}", self.activity_id);
    }
}

// Make LKAS methods public for external control (from VehiclePublisher)
impl LaneKeepAssistSystem {
    pub fn request_lane_change(&mut self, target: i32, reason: &str) {
        if target >= 1 && target <= 4 && target != self.current_lane {
            self.target_lane = target;
            self.is_changing_lane = true;
            self.lkas_status = "LaneChange".to_string();
            
            if reason == "Pullover" {
                self.pullover_requested = true;
                self.lkas_status = "PullingOver".to_string();
            }
            
            info!("🚗 LKAS: Lane change requested from {} to {} ({})", 
                  self.current_lane, target, reason);
        }
    }
}

/// Distraction Publisher activity
///
/// DDS publisher for distraction monitoring (triggers external alarm)
pub struct DistractionPublisher {
    activity_id: ActivityId,
    input_dms: Box<dyn ActivityInput<DmsState>>,
    writer: Option<DataWriter<DistractionMonitor>>,
    participant: Option<DomainParticipant>,
}

impl DistractionPublisher {
    pub fn build(activity_id: ActivityId, dms_topic: &str) -> Box<dyn Activity> {
        info!("🤖 DistractionPublisher initializing");
        
        Box::new(Self {
            activity_id,
            input_dms: activity_input(dms_topic),
            writer: None,
            participant: None,
        })
    }
}

impl fmt::Debug for DistractionPublisher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DistractionPublisher")
            .field("activity_id", &self.activity_id)
            .finish()
    }
}

impl Activity for DistractionPublisher {
    fn id(&self) -> ActivityId {
        self.activity_id
    }
    
    fn startup(&mut self) {
        info!("🤖 DistractionPublisher started - creating DDS writer");
        
        let participant = create_dds_participant();
        
        let topic = participant
            .create_topic::<DistractionMonitor>(
                "DistractionMonitor",
                "DistractionMonitor",
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
                kind: ReliabilityQosPolicyKind::BestEffort,
                max_blocking_time: dust_dds::infrastructure::time::DurationKind::Finite(
                    dust_dds::infrastructure::time::Duration::new(0, 100_000_000)
                ),
            },
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal,
            },
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast(1),  // Only keep last sample
            },
            ..Default::default()
        };
        
        let writer = publisher
            .create_datawriter::<DistractionMonitor>(
                &topic,
                QosKind::Specific(writer_qos),
                None,
                &[],
            )
            .unwrap();
        
        self.writer = Some(writer);
        self.participant = Some(participant);
        
        info!("✅ DistractionPublisher DDS writer ready");
        thread::sleep(Duration::from_millis(200));
    }
    
    fn step(&mut self) {
        debug!("Stepping DistractionPublisher");
        sleep_random();
        
        let dms_state = self.input_dms.read();
        
        if let (Some(ref writer), Ok(dms)) = (&self.writer, dms_state) {
            let distraction_data = DistractionMonitor {
                dms_val: (dms.distraction_duration as f64 / 1000.0) as i32,  // Convert ms to seconds (i32)
            };
            
            match writer.write(&distraction_data, None) {
                Ok(_) => {
                    debug!("📡 Published DistractionMonitor: dms_val={}s", distraction_data.dms_val);
                },
                Err(e) => {
                    debug!("Failed to publish DistractionMonitor: {:?}", e);
                }
            }
        }
    }
    
    fn shutdown(&mut self) {
        info!("🔄 DistractionPublisher shutting down");
        self.writer = None;
        self.participant = None;
    }
}
