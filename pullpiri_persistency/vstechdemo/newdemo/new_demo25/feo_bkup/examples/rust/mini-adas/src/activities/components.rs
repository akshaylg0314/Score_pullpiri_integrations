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

use crate::activities::messages::{CameraImage, RadarScan, Scene, Steering, VehicleData};
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
use std::hash::RandomState;
use std::thread;
use std::time::SystemTime;

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
    writer: Option<DataWriter<VehicleData>>,
    participant: Option<DomainParticipant>,
    // Internal state for vehicle simulation
    vehicle_speed: f64,        // km/h
    lane_position: f64,        // -1.0 to 1.0
    last_obstacle_distance: f64, // meters - for smooth transitions
}

impl VehiclePublisher {
    pub fn build(
        activity_id: ActivityId,
        scene_topic: &str,
        steering_topic: &str,
    ) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            input_scene: activity_input(scene_topic),
            input_steering: activity_input(steering_topic),
            writer: None,
            participant: None,
            vehicle_speed: 60.0,           // Start at 60 km/h
            lane_position: 0.0,            // Start at center
            last_obstacle_distance: 50.0,  // Start with clear road
        })
    }
}

impl Activity for VehiclePublisher {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    fn startup(&mut self) {
        info!("🤖 VehiclePublisher started with INDIVIDUAL participant - publishes IMMEDIATELY when Vehicle mode is active");
        
        // Create individual DDS participant for this component - prevents shared state issues  
        let participant = create_dds_participant();

        let topic = participant
            .create_topic::<VehicleData>(
                "VehicleData",
                "VehicleData",
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
            .create_datawriter::<VehicleData>(
                &topic,
                QosKind::Specific(writer_qos),
                None,
                &[],
            )
            .unwrap();

        self.writer = Some(writer);
        self.participant = Some(participant); // Store for clean shutdown
        
        // Longer initial discovery time to ensure subscribers are detected after restarts
        info!("🤖 VehiclePublisher waiting for subscriber discovery...");
        thread::sleep(Duration::from_millis(200));
        info!("🤖 VehiclePublisher discovery period complete");
    }

    fn step(&mut self) {
        debug!("Stepping VehiclePublisher");
        sleep_random();

        let scene = self.input_scene.read();
        let steering = self.input_steering.read();

        if let Some(ref writer) = self.writer {
            // Create VehicleData from available inputs
            let mut vehicle_data = VehicleData::default();
            
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
                vehicle_data.obstacle_detected = scene_data.distance_obstacle < 50.0;
                vehicle_data.obstacle_distance = scene_data.distance_obstacle;
                
                // Calculate collision risk based on speed and distance
                // Risk increases with speed and decreases with distance
                if scene_data.distance_obstacle < 30.0 {
                    let distance_factor = (30.0 - scene_data.distance_obstacle) / 30.0;
                    let speed_factor = self.vehicle_speed / 80.0; // Normalize by max speed
                    vehicle_data.collision_risk = (distance_factor * 70.0 + speed_factor * 30.0).clamp(0.0, 100.0);
                } else {
                    vehicle_data.collision_risk = 0.0;
                };
                
                // Update lane position based on scene
                self.lane_position = (scene_data.distance_left_lane - scene_data.distance_right_lane) / 10.0;
                vehicle_data.lane_position = self.lane_position.clamp(-1.0, 1.0);
            }

            // Realistic vehicle physics based on obstacle distance
            const MAX_SPEED: f64 = 80.0;           // km/h
            const NATURAL_DECEL: f64 = 0.5;        // km/h per step (friction/drag)
            const ACCEL_RATE: f64 = 1.5;           // km/h per step
            const EMERGENCY_BRAKE_DECEL: f64 = 8.0; // km/h per step
            
            let (acceleration_mps2, brake_force_pct) = if self.last_obstacle_distance > 35.0 {
                // Clear road - accelerate to max speed
                if self.vehicle_speed < MAX_SPEED {
                    self.vehicle_speed = (self.vehicle_speed + ACCEL_RATE).min(MAX_SPEED);
                    (1.5, 0.0) // Positive acceleration, no braking
                } else {
                    // At max speed, maintain with slight deceleration from drag
                    self.vehicle_speed = (self.vehicle_speed - NATURAL_DECEL * 0.3).max(0.0);
                    (0.0, 0.0) // Neutral
                }
            } else if self.last_obstacle_distance > 25.0 && self.last_obstacle_distance <= 35.0 {
                // Moderate distance - cautious acceleration or maintain speed
                if self.vehicle_speed < MAX_SPEED * 0.6 {
                    // Slow acceleration when obstacle at moderate distance (even from stopped)
                    self.vehicle_speed = (self.vehicle_speed + ACCEL_RATE * 0.5).min(MAX_SPEED * 0.6);
                    (0.75, 0.0)
                } else {
                    // Maintain speed with natural deceleration
                    self.vehicle_speed = (self.vehicle_speed - NATURAL_DECEL).max(0.0);
                    (-0.5, 15.0) // Light braking to maintain safe distance
                }
            } else if self.last_obstacle_distance > 15.0 && self.last_obstacle_distance <= 25.0 {
                // Close - apply braking if moving, otherwise stay stopped
                if self.vehicle_speed > 0.0 {
                    // Moving - moderate braking to slow down
                    let decel = 3.0;
                    self.vehicle_speed = (self.vehicle_speed - decel).max(0.0);
                    (-3.0, 40.0 + (25.0 - self.last_obstacle_distance) * 3.0)
                } else {
                    // Already stopped and obstacle still too close - stay stopped
                    (0.0, 0.0)
                }
            } else if self.last_obstacle_distance > 10.0 && self.last_obstacle_distance <= 15.0 {
                // Very close - strong braking if moving, stay stopped if not
                if self.vehicle_speed > 0.0 {
                    let decel = 6.0;
                    self.vehicle_speed = (self.vehicle_speed - decel).max(0.0);
                    (-6.0, 70.0 + (15.0 - self.last_obstacle_distance) * 4.0)
                } else {
                    // Already stopped - stay stopped
                    (0.0, 0.0)
                }
            } else {
                // Critical distance (<10m) - emergency braking if moving, stay stopped if not
                if self.vehicle_speed > 0.0 {
                    self.vehicle_speed = (self.vehicle_speed - EMERGENCY_BRAKE_DECEL).max(0.0);
                    (-8.0, 95.0)
                } else {
                    // Already stopped - stay stopped
                    (0.0, 0.0)
                }
            };

            vehicle_data.acceleration = acceleration_mps2;
            vehicle_data.brake_force = brake_force_pct.clamp(0.0, 100.0);

            // Update from steering data if available
            if let Ok(steering_data) = steering {
                vehicle_data.steering_angle = steering_data.angle.clamp(-45.0, 45.0);
            }

            // Set other vehicle data
            vehicle_data.vehicle_speed = self.vehicle_speed;
            vehicle_data.traffic_signal = "green".to_string();
            vehicle_data.weather_condition = "clear".to_string();
            vehicle_data.road_condition = "dry".to_string();
            vehicle_data.timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;
            vehicle_data.is_valid = true;

            // Publish vehicle data
            match writer.write(&vehicle_data, None) {
                Ok(_) => {
                    debug!("Published VehicleData: speed={:.1} km/h, brake={:.1}%, dist={:.1}m, risk={:.1}%", 
                           vehicle_data.vehicle_speed, 
                           vehicle_data.brake_force,
                           vehicle_data.obstacle_distance,
                           vehicle_data.collision_risk);
                },
                Err(e) => {
                    debug!("Failed to publish VehicleData: {:?}", e);
                }
            }
        }
    }

    fn shutdown(&mut self) {
        info!("🔄 VehiclePublisher shutting down - cleaning up individual DDS participant");
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
