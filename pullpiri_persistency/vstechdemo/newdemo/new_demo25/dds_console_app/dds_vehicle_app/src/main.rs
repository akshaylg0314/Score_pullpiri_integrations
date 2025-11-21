use dust_dds::domain::domain_participant_factory::DomainParticipantFactory;
use dust_dds::infrastructure::listeners::NoOpListener;
use dust_dds::infrastructure::qos::{DataReaderQos, QosKind};
use dust_dds::infrastructure::qos_policy::{
    DurabilityQosPolicy, DurabilityQosPolicyKind, ReliabilityQosPolicy,
    ReliabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
};
use dust_dds::publication::data_writer::DataWriter;
use dust_dds::domain::domain_participant::DomainParticipant;
use dust_dds::infrastructure::status::{StatusKind, NO_STATUS};
use dust_dds::infrastructure::time::{Duration, DurationKind};
use dust_dds::infrastructure::wait_set::{Condition, WaitSet};
use dust_dds::subscription::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE};
use dust_dds::topic_definition::type_support::DdsType;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, Condvar};
use std::time::SystemTime;
use std::fs;
use warp::Filter;

#[derive(DdsType, Clone, Debug, Serialize, Deserialize)]
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

#[derive(DdsType, Clone, Debug, Serialize, Deserialize)]
pub struct CarMode {
    pub driving_mode: String,
}

// Mode transition tracking for UI notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeTransition {
    pub current_mode: String,
    pub previous_mode: String,
    pub transition_reason: String,
    pub timestamp: i64,
    pub transition_state: String, // "active", "pending", "waiting_for_threshold_update", "timeout"
}

impl ModeTransition {
    fn new(current: &str, previous: &str, reason: &str) -> Self {
        Self {
            current_mode: current.to_string(),
            previous_mode: previous.to_string(),
            transition_reason: reason.to_string(),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
            transition_state: "active".to_string(),
        }
    }
    
    fn new_pending(target: &str, current: &str, reason: &str) -> Self {
        Self {
            current_mode: target.to_string(),
            previous_mode: current.to_string(),
            transition_reason: reason.to_string(),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
            transition_state: "waiting_for_threshold_update".to_string(),
        }
    }
}

// Weather/road condition multipliers
#[derive(Debug, Clone)]
struct EnvironmentMultipliers {
    distance_multiplier: f64,
    speed_multiplier: f64,
    force_manual: bool,
}

impl EnvironmentMultipliers {
    fn from_conditions(weather: &str, road: &str) -> Self {
        let mut multipliers = Self {
            distance_multiplier: 1.0,
            speed_multiplier: 1.0,
            force_manual: false,
        };

        // Weather adjustments
        match weather.to_lowercase().as_str() {
            "rain" => {
                multipliers.distance_multiplier = 1.3;
                multipliers.speed_multiplier = 0.8;
            }
            "snow" => {
                multipliers.distance_multiplier = 1.5;
                multipliers.speed_multiplier = 0.6;
            }
            "fog" => {
                multipliers.force_manual = true; // Fog forces manual mode
            }
            _ => {} // "clear" uses defaults
        }

        // Road condition adjustments (compound with weather)
        match road.to_lowercase().as_str() {
            "wet" => {
                multipliers.distance_multiplier *= 1.2;
                multipliers.speed_multiplier *= 0.9;
            }
            "icy" => {
                multipliers.distance_multiplier *= 1.5;
                multipliers.speed_multiplier *= 0.6;
            }
            "gravel" => {
                multipliers.distance_multiplier *= 1.1;
                multipliers.speed_multiplier *= 0.85;
            }
            _ => {} // "dry" uses defaults
        }

        multipliers
    }
}

// Threshold set for a specific mode
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ThresholdSet {
    pub obstacle_distance_min: f64,
    pub collision_risk_max: f64,
    pub vehicle_speed_max: f64,
    pub vehicle_speed_min: f64,
    pub brake_force_max: f64,
    pub steering_angle_max: f64,
}

// Complete threshold configuration - dynamically loaded per mode
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Thresholds {
    pub current_mode_thresholds: ThresholdSet,
    pub stability: StabilityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StabilityConfig {
    pub mode_change_cooldown_ms: u64,    // Cooldown period between mode changes
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            current_mode_thresholds: ThresholdSet {
                obstacle_distance_min: 25.0,
                collision_risk_max: 50.0,
                vehicle_speed_max: 90.0,
                vehicle_speed_min: 0.0,
                brake_force_max: 80.0,
                steering_angle_max: 25.0,
            },
            stability: StabilityConfig {
                mode_change_cooldown_ms: 30000, // 30 seconds default
            },
        }
    }
}

struct SharedState {
    latest_data: Option<VehicleData>,
    data_updated: bool,
    current_mode: String,
    pending_mode: Option<String>,          // Target mode waiting for threshold update
    mode_transition: ModeTransition,
    violation_count: usize,           // Consecutive violations for confirmation window
    last_mode_change_time: i64,       // Timestamp of last mode change for cooldown
    pending_transition_time: Option<i64>, // Timestamp when transition was initiated (for timeout)
}

impl SharedState {
    fn new() -> Self {
        let initial_transition = ModeTransition::new("manual", "none", "System startup - default to manual mode");
        Self {
            latest_data: None,
            data_updated: false,
            current_mode: "manual".to_string(),
            pending_mode: None,
            mode_transition: initial_transition,
            violation_count: 0,
            last_mode_change_time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
            pending_transition_time: None,
        }
    }
}

struct CarModePublisher {
    writer: Option<DataWriter<CarMode>>,
    participant: Option<DomainParticipant>,
    last_published_mode: Option<String>,
}

impl CarModePublisher {
    fn new() -> Self {
        Self {
            writer: None,
            participant: None,
            last_published_mode: None,
        }
    }

    fn startup(&mut self) {
        println!("🚀 CarModePublisher started, initializing DDS with INDIVIDUAL participant for clean restart...");
        
        // Create individual DDS participant for this component
        let participant_factory = DomainParticipantFactory::get_instance();
        let participant = participant_factory
            .create_participant(100, QosKind::Default, NoOpListener::new(), NO_STATUS)
            .expect("Failed to create participant for CarMode publisher");

        let topic = participant
            .create_topic(
                "CarMode",
                "CarMode", 
                QosKind::Default,
                NoOpListener::new(),
                NO_STATUS,
            )
            .unwrap();

        let publisher = participant
            .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
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
            .create_datawriter::<CarMode>(
                &topic,
                QosKind::Specific(writer_qos),
                NoOpListener::new(),
                NO_STATUS,
            )
            .unwrap();

        println!("📡 CarModePublisher setup with INDIVIDUAL participant - Enhanced QoS & clean restart support");
        
        self.writer = Some(writer);
        self.participant = Some(participant); // Store for clean shutdown
        
        std::thread::sleep(std::time::Duration::from_millis(200));
        println!("✅ CarModePublisher DDS setup complete - ready to publish CarMode with enhanced reliability");
    }

    fn publish_car_mode(&mut self, car_mode: &str, force: bool) {
        // Check if this is a new/different mode compared to what we last published
        let should_publish = force || match &self.last_published_mode {
            Some(last_mode) => last_mode != car_mode,
            None => true, // First publish
        };
        
        if should_publish {
            if force {
                println!("📡 FORCED PUBLISH - Publishing CarMode to DDS for pullpiri: {}", car_mode);
            } else {
                println!("📡 NEW MODE DETECTED - Publishing CarMode to DDS for pullpiri: {} → {}", 
                    self.last_published_mode.as_deref().unwrap_or("none"), car_mode);
            }

            if let Some(writer) = &mut self.writer {
                let dds_car_mode = CarMode {
                    driving_mode: car_mode.to_string(),
                };

                println!("🌐 [DDS] Sending CarMode for pullpiri integration: {:?}", dds_car_mode);
                
                // Retry logic with exponential backoff
                let mut retry_count = 0;
                const MAX_RETRIES: u32 = 3;
                let mut success = false;
                
                while retry_count <= MAX_RETRIES && !success {
                    match writer.write(&dds_car_mode, None) {
                        Ok(_) => {
                            println!("✅ [DDS] CarMode successfully published to topic 'CarMode'{}", 
                                if retry_count > 0 { format!(" (retry {})", retry_count) } else { String::new() });
                            self.last_published_mode = Some(car_mode.to_string());
                            success = true;
                        },
                        Err(e) => {
                            if retry_count < MAX_RETRIES {
                                let backoff_ms = 100 * (2_u64.pow(retry_count));
                                println!("⚠️  [DDS] Write failed (attempt {}), retrying in {}ms...", 
                                    retry_count + 1, backoff_ms);
                                std::thread::sleep(std::time::Duration::from_millis(backoff_ms));
                                retry_count += 1;
                            } else {
                                println!("❌ [DDS] All {} write attempts failed: {:?}", MAX_RETRIES + 1, e);
                                // Still update last_published_mode - data is in TransientLocal
                                println!("📝 [DDS] CarMode cached in TransientLocal durability (late joiners will receive)");
                                self.last_published_mode = Some(car_mode.to_string());
                                break;
                            }
                        },
                    }
                }
            }
        } else {
            println!("📡 CarModePublisher: Mode unchanged ({}), skipping DDS publish to prevent spam", car_mode);
        }
    }
}

fn load_thresholds(file_path: &str) -> Thresholds {
    match fs::read_to_string(file_path) {
        Ok(content) => {
            match serde_json::from_str::<Thresholds>(&content) {
                Ok(thresholds) => {
                    println!("📁 Loaded thresholds from {}", file_path);
                    thresholds
                }
                Err(e) => {
                    println!("⚠️  Failed to parse thresholds file: {}. Using defaults.", e);
                    Thresholds::default()
                }
            }
        }
        Err(_) => {
            println!("📁 Thresholds file not found, creating default one at {}", file_path);
            let default_thresholds = Thresholds::default();
            if let Ok(json_content) = serde_json::to_string_pretty(&default_thresholds) {
                let _ = fs::write(file_path, json_content);
            }
            default_thresholds
        }
    }
}

fn get_file_modified_time(file_path: &str) -> Option<SystemTime> {
    fs::metadata(file_path)
        .and_then(|metadata| metadata.modified())
        .ok()
}

fn spawn_file_watcher(thresholds: Arc<Mutex<Thresholds>>, shared_state: Arc<Mutex<SharedState>>) {
    std::thread::spawn(move || {
        let file_path = "thresholds.json";
        let mut last_modified = get_file_modified_time(file_path);
        
        println!("🔍 File watcher started for {}", file_path);
        
        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
            
            if let Some(current_modified) = get_file_modified_time(file_path) {
                if last_modified.is_none() || last_modified.unwrap() != current_modified {
                    println!("📝 Thresholds file changed, reloading...");
                    let new_thresholds = load_thresholds(file_path);
                    *thresholds.lock().unwrap() = new_thresholds;
                    last_modified = Some(current_modified);
                    
                    // Check if there's a pending mode transition waiting for this update
                    let mut state = shared_state.lock().unwrap();
                    if let Some(pending_mode) = state.pending_mode.clone() {
                        println!("✅ Thresholds updated! Completing mode transition: {} → {}", 
                            state.current_mode, pending_mode);
                        
                        // Complete the transition
                        let previous_mode = state.current_mode.clone();
                        state.current_mode = pending_mode.clone();
                        state.pending_mode = None;
                        state.pending_transition_time = None;
                        state.last_mode_change_time = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as i64;
                        
                        // Update transition to active state
                        state.mode_transition = ModeTransition::new(
                            &pending_mode,
                            &previous_mode,
                            "Threshold update complete - mode transition activated"
                        );
                        
                        println!("🔄 MODE CHANGE COMPLETE: {} → {} | New thresholds active", 
                            previous_mode, pending_mode);
                    }
                }
            }
        }
    });
}

fn spawn_timeout_checker(shared_state: Arc<Mutex<SharedState>>, car_publisher: Arc<Mutex<CarModePublisher>>) {
    std::thread::spawn(move || {
        const TIMEOUT_MS: i64 = 5000; // 5 seconds timeout
        
        println!("⏰ Timeout checker started ({}s timeout for pending transitions)", TIMEOUT_MS / 1000);
        
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            
            let mut state = shared_state.lock().unwrap();
            
            if let (Some(pending_mode), Some(pending_time)) = (&state.pending_mode, state.pending_transition_time) {
                let current_time = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i64;
                
                let elapsed_ms = current_time - pending_time;
                
                if elapsed_ms >= TIMEOUT_MS {
                    println!("❌ TIMEOUT: Threshold update not received within {}s", TIMEOUT_MS / 1000);
                    println!("❌ Cancelling pending transition: {} → {}", 
                        state.current_mode, pending_mode);
                    
                    // Cancel the pending transition
                    state.pending_mode = None;
                    state.pending_transition_time = None;
                    
                    // Update transition state to timeout
                    let mut timeout_transition = state.mode_transition.clone();
                    timeout_transition.transition_state = "timeout".to_string();
                    timeout_transition.transition_reason = format!(
                        "Timeout: External service failed to update thresholds within {}s - transition cancelled",
                        TIMEOUT_MS / 1000
                    );
                    state.mode_transition = timeout_transition;
                    
                    drop(state); // Release lock before publisher operation
                    
                    // Clear last_published_mode to allow fresh publish on next attempt
                    let mut publisher = car_publisher.lock().unwrap();
                    publisher.last_published_mode = None;
                    println!("🔄 Cleared last_published_mode to allow fresh publish on retry");
                    
                    println!("⚠️  Remaining in mode: {} (external service may be unavailable)", 
                        shared_state.lock().unwrap().current_mode);
                }
            }
        }
    });
}

fn spawn_condition_checker(
    shared_state: Arc<Mutex<SharedState>>,
    data_condvar: Arc<Condvar>,
    thresholds: Arc<Mutex<Thresholds>>,
    car_publisher: Arc<Mutex<CarModePublisher>>,
) {
    std::thread::spawn(move || {
        println!("🔧 Intelligent condition checker started with mode transition tracking");
        
        loop {
            // Wait for new data signal
            let mut state = shared_state.lock().unwrap();
            while !state.data_updated {
                state = data_condvar.wait(state).unwrap();
            }
            
            // Reset the flag and get the data
            state.data_updated = false;
            
            // Check if transition is pending - if so, skip condition checking
            if state.pending_mode.is_some() {
                if let Some(pending_time) = state.pending_transition_time {
                    let current_time = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as i64;
                    let elapsed_ms = current_time - pending_time;
                    
                    println!("⏳ Mode transition PENDING ({:.1}s elapsed) - waiting for threshold update, skipping condition checks", 
                        elapsed_ms as f64 / 1000.0);
                }
                continue;
            }
            
            if let Some(ref data) = state.latest_data {
                let data_clone = data.clone();
                let current_mode = state.current_mode.clone();
                let last_change_time = state.last_mode_change_time;
                drop(state); // Release the lock
                
                // Get current thresholds
                let thresholds_guard = thresholds.lock().unwrap();
                let current_thresholds = thresholds_guard.clone();
                drop(thresholds_guard);
                
                // Check conditions and determine new mode with transition tracking
                let result = check_conditions_and_get_mode(
                    &data_clone, 
                    &current_thresholds, 
                    &current_mode,
                    last_change_time
                );
                
                // If mode should change, initiate pending transition
                if result.new_mode != current_mode {
                    let mut state = shared_state.lock().unwrap();
                    
                    // Set pending transition
                    state.pending_mode = Some(result.new_mode.clone());
                    state.pending_transition_time = Some(SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as i64);
                    
                    // Update transition to pending state
                    state.mode_transition = ModeTransition::new_pending(
                        &result.new_mode,
                        &current_mode,
                        &result.transition.transition_reason
                    );
                    
                    drop(state);
                    
                    println!("🔄 Mode transition INITIATED: {} → {} | Reason: {}", 
                        current_mode,
                        result.new_mode,
                        result.transition.transition_reason);
                    println!("⏳ Waiting for external service to update thresholds.json...");
                    
                    // Publish new car mode via DDS to trigger external service
                    // Use force=true to ensure publish even if transitioning to same mode as before
                    let mut publisher = car_publisher.lock().unwrap();
                    publisher.publish_car_mode(&result.new_mode, true); // FORCE publish for pending transitions
                } else {
                    // Mode unchanged, update violation count for confirmation window
                    let mut state = shared_state.lock().unwrap();
                    state.violation_count = result.violation_count;
                }
            }
        }
    });
}

struct ModeCheckResult {
    new_mode: String,
    transition: ModeTransition,
    violation_count: usize,
}

fn check_conditions_and_get_mode(
    data: &VehicleData, 
    thresholds: &Thresholds,
    current_mode: &str,
    last_mode_change_time: i64,
) -> ModeCheckResult {
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    
    // Calculate time since last mode change
    let time_since_change_ms = current_time - last_mode_change_time;
    
    // Check if still in cooldown period
    let in_cooldown = time_since_change_ms < thresholds.stability.mode_change_cooldown_ms as i64;
    
    // Get environment multipliers based on weather and road conditions
    let env_multipliers = EnvironmentMultipliers::from_conditions(
        &data.weather_condition,
        &data.road_condition
    );
    
    // Apply weather/road multipliers to thresholds
    let adjusted_thresholds = apply_environment_multipliers(&thresholds.current_mode_thresholds, &env_multipliers);
    
    // Check for fog condition - forces manual mode immediately
    if env_multipliers.force_manual && current_mode != "manual" {
        return ModeCheckResult {
            new_mode: "manual".to_string(),
            transition: ModeTransition::new(
                "manual",
                current_mode,
                "Weather condition (fog) requires manual control"
            ),
            violation_count: 0,
        };
    }
    
    // Collect violations with detailed reasons
    let mut violations = Vec::new();
    let mut critical_violations = Vec::new();
    
    // Check obstacle distance (adjusted for weather/road)
    if data.obstacle_distance < adjusted_thresholds.obstacle_distance_min {
        let reason = format!("Obstacle too close: {:.1}m < {:.1}m (adjusted for {}/{})", 
            data.obstacle_distance, 
            adjusted_thresholds.obstacle_distance_min,
            data.weather_condition,
            data.road_condition);
        violations.push(reason.clone());
        
        // Critical if obstacle very close
        if data.obstacle_distance < adjusted_thresholds.obstacle_distance_min * 0.5 {
            critical_violations.push(reason);
        }
    }
    
    // Check collision risk
    if data.collision_risk > adjusted_thresholds.collision_risk_max {
        let reason = format!("Collision risk too high: {:.1}% > {:.1}%", 
            data.collision_risk, 
            adjusted_thresholds.collision_risk_max);
        violations.push(reason.clone());
        
        // Critical if collision risk very high
        if data.collision_risk > 90.0 {
            critical_violations.push(reason);
        }
    }
    
    // Check speed violations (adjusted for weather/road)
    if data.vehicle_speed > adjusted_thresholds.vehicle_speed_max {
        let reason = format!("Speed too high: {:.1} km/h > {:.1} km/h (adjusted for {}/{})", 
            data.vehicle_speed, 
            adjusted_thresholds.vehicle_speed_max,
            data.weather_condition,
            data.road_condition);
        violations.push(reason.clone());
        
        // Critical if significantly overspeeding
        if data.vehicle_speed > adjusted_thresholds.vehicle_speed_max * 1.2 {
            critical_violations.push(reason);
        }
    }
    
    if data.vehicle_speed < adjusted_thresholds.vehicle_speed_min {
        violations.push(format!("Speed too low: {:.1} km/h < {:.1} km/h", 
            data.vehicle_speed, 
            adjusted_thresholds.vehicle_speed_min));
    }
    
    // Check brake force
    if data.brake_force > adjusted_thresholds.brake_force_max {
        let reason = format!("Brake force too high: {:.1}% > {:.1}%", 
            data.brake_force, 
            adjusted_thresholds.brake_force_max);
        violations.push(reason.clone());
        
        // Critical if emergency braking
        if data.brake_force > 90.0 {
            critical_violations.push(reason);
        }
    }
    
    // Check steering angle
    if data.steering_angle.abs() > adjusted_thresholds.steering_angle_max {
        violations.push(format!("Steering angle too sharp: {:.1}° > {:.1}°", 
            data.steering_angle.abs(), 
            adjusted_thresholds.steering_angle_max));
    }
    
    // Determine mode based on current mode and violations (staged recovery logic)
    let (new_mode, reason) = match current_mode {
        "autonomous" => {
            // From autonomous: can degrade to manual or emergency
            if !critical_violations.is_empty() {
                ("emergency".to_string(), format!("CRITICAL: {}", critical_violations.join("; ")))
            } else if !violations.is_empty() {
                ("manual".to_string(), format!("Threshold violations: {}", violations.join("; ")))
            } else {
                (current_mode.to_string(), "All conditions nominal".to_string())
            }
        },
        "manual" => {
            // From manual: can go to emergency (critical) or autonomous (all clear with confirmation)
            if !critical_violations.is_empty() {
                ("emergency".to_string(), format!("CRITICAL: {}", critical_violations.join("; ")))
            } else if violations.is_empty() && !in_cooldown {
                // All clear - but need confirmation window before returning to autonomous
                ("autonomous".to_string(), "All conditions clear - returning to autonomous mode".to_string())
            } else if in_cooldown {
                (current_mode.to_string(), format!("In cooldown period ({:.1}s remaining)", 
                    (thresholds.stability.mode_change_cooldown_ms as i64 - time_since_change_ms) as f64 / 1000.0))
            } else {
                (current_mode.to_string(), format!("Conditions not clear: {}", violations.join("; ")))
            }
        },
        "emergency" => {
            // From emergency: can only go to manual (staged recovery), never directly to autonomous
            if !in_cooldown {
                if critical_violations.is_empty() && violations.len() < 3 {
                    // Conditions improved - return to manual (not autonomous, staged recovery)
                    ("manual".to_string(), "Emergency conditions resolved - transitioning to manual mode".to_string())
                } else {
                    (current_mode.to_string(), format!("Emergency conditions persist: {}", 
                        if !critical_violations.is_empty() { critical_violations.join("; ") } 
                        else { violations.join("; ") }))
                }
            } else {
                (current_mode.to_string(), format!("Emergency cooldown active ({:.1}s remaining)", 
                    (thresholds.stability.mode_change_cooldown_ms as i64 - time_since_change_ms) as f64 / 1000.0))
            }
        },
        _ => {
            // Unknown mode, default to manual (failsafe)
            ("manual".to_string(), "Unknown mode detected - failsafe to manual".to_string())
        }
    };
    
    // Log the results
    if new_mode == current_mode {
        if violations.is_empty() {
            println!("✅ Mode: {} | All conditions OK - Speed: {:.1} km/h, Distance: {:.1}m, Risk: {:.1}%", 
                current_mode, data.vehicle_speed, data.obstacle_distance, data.collision_risk);
        } else {
            println!("⚠️  Mode: {} | Violations detected but staying in current mode: {}", 
                current_mode, reason);
        }
    }
    
    ModeCheckResult {
        new_mode: new_mode.clone(),
        transition: ModeTransition::new(&new_mode, current_mode, &reason),
        violation_count: if !violations.is_empty() { 1 } else { 0 },
    }
}

fn apply_environment_multipliers(
    thresholds: &ThresholdSet,
    multipliers: &EnvironmentMultipliers
) -> ThresholdSet {
    ThresholdSet {
        obstacle_distance_min: thresholds.obstacle_distance_min * multipliers.distance_multiplier,
        collision_risk_max: thresholds.collision_risk_max, // Risk threshold not affected by weather
        vehicle_speed_max: thresholds.vehicle_speed_max * multipliers.speed_multiplier,
        vehicle_speed_min: thresholds.vehicle_speed_min,
        brake_force_max: thresholds.brake_force_max,
        steering_angle_max: thresholds.steering_angle_max,
    }
}

#[tokio::main]
async fn main() {
    // Shared state for latest data and condition checking with mode tracking
    let shared_state = Arc::new(Mutex::new(SharedState::new()));
    let data_condvar = Arc::new(Condvar::new());
    
    // Shared thresholds
    let thresholds = Arc::new(Mutex::new(load_thresholds("thresholds.json")));
    
    // Start file watcher thread (now with shared_state access)
    spawn_file_watcher(thresholds.clone(), shared_state.clone());
    
    // Initialize CarMode publisher
    let car_publisher = Arc::new(Mutex::new(CarModePublisher::new()));
    {
        let mut publisher = car_publisher.lock().unwrap();
        publisher.startup();
    }
    
    // Start timeout checker thread (now with car_publisher access)
    spawn_timeout_checker(shared_state.clone(), car_publisher.clone());
    
    // Start condition checker thread
    spawn_condition_checker(shared_state.clone(), data_condvar.clone(), thresholds.clone(), car_publisher.clone());
    
    // For REST API, we need shared state access for latest data and mode status
    let latest_data_for_api = Arc::new(Mutex::new(None::<VehicleData>));
    let mode_state_for_api = shared_state.clone();
    
    let latest_data_filter = warp::any().map({
        let latest_data = latest_data_for_api.clone();
        move || latest_data.clone()
    });
    
    let mode_state_filter = warp::any().map({
        let mode_state = mode_state_for_api.clone();
        move || mode_state.clone()
    });
    
    let thresholds_filter = warp::any().map({
        let thresholds = thresholds.clone();
        move || thresholds.clone()
    });

    // REST endpoint: GET /data
    let get_data = warp::path("data")
        .and(warp::get())
        .and(latest_data_filter.clone())
        .map(|latest_data: Arc<Mutex<Option<VehicleData>>>| {
            let data = latest_data.lock().unwrap();
            let response = if let Some(ref d) = *data {
                warp::reply::json(d)
            } else {
                warp::reply::json(&serde_json::json!({"error": "No vehicle data available yet"}))
            };
            
            warp::reply::with_header(
                warp::reply::with_header(
                    warp::reply::with_header(response, "Access-Control-Allow-Origin", "*"),
                    "Access-Control-Allow-Methods", "GET, POST, OPTIONS"
                ),
                "Access-Control-Allow-Headers", "Content-Type"
            )
        });
    
    // NEW REST endpoint: GET /mode-status
    let get_mode_status = warp::path("mode-status")
        .and(warp::get())
        .and(mode_state_filter.clone())
        .map(|mode_state: Arc<Mutex<SharedState>>| {
            let state = mode_state.lock().unwrap();
            let response = warp::reply::json(&state.mode_transition);
            
            warp::reply::with_header(
                warp::reply::with_header(
                    warp::reply::with_header(response, "Access-Control-Allow-Origin", "*"),
                    "Access-Control-Allow-Methods", "GET, POST, OPTIONS"
                ),
                "Access-Control-Allow-Headers", "Content-Type"
            )
        });
    
    // NEW REST endpoint: GET /thresholds
    let get_thresholds = warp::path("thresholds")
        .and(warp::get())
        .and(thresholds_filter.clone())
        .map(|thresholds: Arc<Mutex<Thresholds>>| {
            let thresh = thresholds.lock().unwrap();
            let response = warp::reply::json(&*thresh);
            
            warp::reply::with_header(
                warp::reply::with_header(
                    warp::reply::with_header(response, "Access-Control-Allow-Origin", "*"),
                    "Access-Control-Allow-Methods", "GET, POST, OPTIONS"
                ),
                "Access-Control-Allow-Headers", "Content-Type"
            )
        });

    // OPTIONS handler for CORS preflight for /data
    let options_data = warp::path("data")
        .and(warp::options())
        .map(|| {
            warp::reply::with_header(
                warp::reply::with_header(
                    warp::reply::with_header(warp::reply(), "Access-Control-Allow-Origin", "*"),
                    "Access-Control-Allow-Methods", "GET, POST, OPTIONS"
                ),
                "Access-Control-Allow-Headers", "Content-Type"
            )
        });
    
    // OPTIONS handler for CORS preflight for /mode-status
    let options_mode_status = warp::path("mode-status")
        .and(warp::options())
        .map(|| {
            warp::reply::with_header(
                warp::reply::with_header(
                    warp::reply::with_header(warp::reply(), "Access-Control-Allow-Origin", "*"),
                    "Access-Control-Allow-Methods", "GET, POST, OPTIONS"
                ),
                "Access-Control-Allow-Headers", "Content-Type"
            )
        });
    
    // OPTIONS handler for CORS preflight for /thresholds
    let options_thresholds = warp::path("thresholds")
        .and(warp::options())
        .map(|| {
            warp::reply::with_header(
                warp::reply::with_header(
                    warp::reply::with_header(warp::reply(), "Access-Control-Allow-Origin", "*"),
                    "Access-Control-Allow-Methods", "GET, POST, OPTIONS"
                ),
                "Access-Control-Allow-Headers", "Content-Type"
            )
        });

    let api = get_data.or(get_mode_status).or(get_thresholds).or(options_data).or(options_mode_status).or(options_thresholds);

    // Spawn REST API server in background (port 9083 for Vehicle)
    let rest_handle = tokio::spawn(async move {
        println!("🌐 Vehicle Data REST API running on:");
        println!("   - http://localhost:9083/data (vehicle data)");
        println!("   - http://localhost:9083/mode-status (mode transition info)");
        println!("   - http://localhost:9083/thresholds (active thresholds)");
        warp::serve(api).run(([0, 0, 0, 0], 9083)).await;
    });

    // Spawn DDS subscriber in background task
    let latest_data_sub = latest_data_for_api.clone();
    let shared_state_dds = shared_state.clone();
    let data_condvar_dds = data_condvar.clone();
    let dds_handle = tokio::spawn(async move {
        let domain_id = 100;
        let topic_name = "VehicleData";
        let type_name = "VehicleData";

        let participant_factory = DomainParticipantFactory::get_instance();
        let participant = participant_factory
            .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
            .expect("Failed to create participant");

        let subscriber = participant
        .create_subscriber(QosKind::Default, NoOpListener::new(), NO_STATUS)
        .expect("Failed to create subscriber");

        let topic = participant
        .create_topic(
            topic_name,
            type_name,
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .expect("Failed to create topic");

         let reader_qos = DataReaderQos {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort, // Match publisher BestEffort QoS
                max_blocking_time: DurationKind::Finite(Duration::new(1, 0)), // Reduced timeout
            },
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal, // Keep for historical data
            },
            history: dust_dds::infrastructure::qos_policy::HistoryQosPolicy {
                kind: dust_dds::infrastructure::qos_policy::HistoryQosPolicyKind::KeepLast(5), // Reduced history
            },
            ..Default::default()
        };

        let reader = subscriber
            .create_datareader::<VehicleData>(&topic, QosKind::Specific(reader_qos), NoOpListener::new(), NO_STATUS)
            .expect("Failed to create datareader");

        // Wait for publisher discovery and data
        let reader_cond = reader.get_statuscondition().expect("Failed to get status condition");
        reader_cond
            .set_enabled_statuses(&[StatusKind::SubscriptionMatched, StatusKind::DataAvailable])
            .expect("Failed to set enabled statuses");
        
        let mut wait_set = WaitSet::new();
        wait_set
            .attach_condition(Condition::StatusCondition(reader_cond.clone()))
            .expect("Failed to attach condition");

        println!("Vehicle DDS Subscriber ready - waiting for data...");
        
        let mut publisher_discovered = false;
        loop {
            // Wait for discovery or data with longer timeout for better discovery
            match wait_set.wait(Duration::new(5, 0)) { // Increased from 2 to 5 seconds
                Ok(_) => {
                    // Check subscription status
                    let subscription_matched_status = reader.get_subscription_matched_status().unwrap_or_default();
                    if subscription_matched_status.current_count > 0 && !publisher_discovered {
                        println!("✅ Publisher discovered! {} publisher(s) matched", subscription_matched_status.current_count);
                        publisher_discovered = true;
                        
                        // Wait longer for transient local data to be available
                        std::thread::sleep(std::time::Duration::from_millis(500)); // Increased from 100ms to 500ms
                        
                        // Immediately try to read any historical data
                        let historical_samples = reader
                            .take(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
                            .unwrap_or_default();
                        
                        if !historical_samples.is_empty() {
                            println!("📜 Found {} historical samples from TransientLocal durability", historical_samples.len());
                            for sample in historical_samples {
                                if let Ok(data) = sample.data() {
                                    println!("📡 Received historical Vehicle data: speed={}, distance={:.1}m", 
                                        data.vehicle_speed, data.obstacle_distance);
                                    
                                    // Update API data
                                    *latest_data_sub.lock().unwrap() = Some(data.clone());
                                    
                                    // Update shared state and signal condition checker
                                    {
                                        let mut state = shared_state_dds.lock().unwrap();
                                        state.latest_data = Some(data.clone());
                                        state.data_updated = true;
                                    }
                                    data_condvar_dds.notify_one();
                                }
                            }
                        } else {
                            println!("📜 No historical data found - waiting for fresh data");
                            
                            // Try to read again after another brief wait
                            std::thread::sleep(std::time::Duration::from_millis(300));
                            let retry_samples = reader
                                .take(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
                                .unwrap_or_default();
                            
                            if !retry_samples.is_empty() {
                                println!("📜 Found {} historical samples on retry", retry_samples.len());
                                for sample in retry_samples {
                                    if let Ok(data) = sample.data() {
                                        println!("📡 Received historical Vehicle data (retry): speed={}, distance={:.1}m", 
                                            data.vehicle_speed, data.obstacle_distance);
                                        
                                        // Update API data
                                        *latest_data_sub.lock().unwrap() = Some(data.clone());
                                        
                                        // Update shared state and signal condition checker
                                        {
                                            let mut state = shared_state_dds.lock().unwrap();
                                            state.latest_data = Some(data.clone());
                                            state.data_updated = true;
                                        }
                                        data_condvar_dds.notify_one();
                                    }
                                }
                            }
                        }
                    }
                    
                    // Try to read fresh data
                    let samples = reader
                        .take(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
                        .unwrap_or_default();

                    for sample in samples {
                        if let Ok(data) = sample.data() {
                            println!("📡 Received fresh VehicleData: speed={}, distance={:.1}m", 
                                data.vehicle_speed, data.obstacle_distance);
                            
                            // Update shared state for REST API
                            *latest_data_sub.lock().unwrap() = Some(data.clone());
                            
                            // Update shared state and signal condition checker
                            {
                                let mut state = shared_state_dds.lock().unwrap();
                                state.latest_data = Some(data.clone());
                                state.data_updated = true;
                            }
                            data_condvar_dds.notify_one();
                        }
                    }
                }
                Err(_) => {
                    // Timeout - check if we still have publishers
                    let status = reader.get_subscription_matched_status().unwrap_or_default();
                    if status.current_count == 0 && publisher_discovered {
                        println!("⚠️  Publisher disconnected, waiting for reconnection...");
                        publisher_discovered = false;
                    }
                }
            }
        }
    });

    // Wait for both tasks to complete
    let _ = tokio::join!(rest_handle, dds_handle);
}
