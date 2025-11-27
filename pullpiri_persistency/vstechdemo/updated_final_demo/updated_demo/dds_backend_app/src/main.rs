use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        qos::QosKind,
        status::NO_STATUS,
    },
    subscription::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    topic_definition::type_support::DdsType,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, Duration};
use std::fs;
use std::path::Path;
use warp::Filter;

// DashboardData struct matching mini-adas definition
#[derive(Debug, Default, Clone, Serialize, Deserialize, DdsType)]
#[repr(C)]
pub struct DashboardData {
    // DMS Fields
    pub distraction_duration: f64,
    pub gaze_direction: String,
    pub head_yaw: f64,
    pub head_pitch: f64,
    pub head_roll: f64,
    pub drowsiness_score: f64,
    pub attention_level: f64,
    pub driver_status: String,
    
    // LKAS Fields
    pub current_lane: i32,
    pub lane_position_offset: f64,
    pub lkas_status: String,
    pub left_lane_distance: f64,
    pub right_lane_distance: f64,
    
    // Vehicle Metrics
    pub vehicle_speed: f64,
    pub obstacle_distance: f64,
    pub obstacle_detected: bool,
    pub steering_angle: f64,
    
    pub timestamp: i64,
    pub is_valid: bool,
}

// Struct to hold data with timestamp for timeout management
#[derive(Clone, Debug)]
pub struct TimestampedData {
    pub data: DashboardData,
    pub received_at: Instant,
}

// Alert message from file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertMessage {
    pub scenario_name: String,
    pub content: String,
    #[serde(skip)]
    pub received_at: Option<Instant>,
}

// Shared application state
#[derive(Default)]
pub struct AppState {
    pub telemetry: Option<TimestampedData>,
    pub alert: Option<AlertMessage>,
}

#[tokio::main]
async fn main() {
    // Shared state for REST API
    let app_state = Arc::new(Mutex::new(AppState::default()));
    
    let state_filter = warp::any().map({
        let state = app_state.clone();
        move || state.clone()
    });

    // REST endpoint: GET /data
    let get_data = warp::path("data")
        .and(warp::get())
        .and(state_filter.clone())
        .map(|state: Arc<Mutex<AppState>>| {
            let mut guard = state.lock().unwrap();
            
            // 1. Process Telemetry (Timeout: 3s)
            let telemetry_data = if let Some(ref t_data) = guard.telemetry {
                if t_data.received_at.elapsed().as_secs() > 3 {
                    guard.telemetry = None;
                    None
                } else {
                    Some(t_data.data.clone())
                }
            } else {
                None
            };

            // 2. Process Alert (Timeout: 5s)
            let alert_data = if let Some(ref a_data) = guard.alert {
                if let Some(received_at) = a_data.received_at {
                    if received_at.elapsed().as_secs() > 5 {
                        // println!("⏰ Alert expired, clearing");
                        guard.alert = None;
                        None
                    } else {
                        Some(a_data.clone())
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // 3. Construct Merged Response
            let response = serde_json::json!({
                "telemetry": telemetry_data,
                "alert": alert_data
            });
            
            warp::reply::with_header(
                warp::reply::with_header(
                    warp::reply::with_header(warp::reply::json(&response), "Access-Control-Allow-Origin", "*"),
                    "Access-Control-Allow-Methods", "GET, POST, OPTIONS"
                ),
                "Access-Control-Allow-Headers", "Content-Type"
            )
        });

    // OPTIONS handler for CORS
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

    // Health endpoint
    let health = warp::path("health")
        .and(warp::get())
        .map(|| {
            let response = serde_json::json!({
                "status": "healthy",
                "service": "dds-backend-receiver-rust",
                "version": "2.0.0"
            });
            warp::reply::json(&response)
        });

    let api = get_data.or(options_data).or(health);

    // Spawn REST API server
    tokio::spawn(async move {
    println!("🌐 DDS Backend REST API running on:");
    println!("   - http://0.0.0.0:8089/data (Merged Telemetry + Alerts)");
    warp::serve(api).run(([0, 0, 0, 0], 8089)).await;
    });

    // --- FILE MONITOR THREAD WITH PRIORITY ---
    let file_monitor_state = app_state.clone();
    std::thread::spawn(move || {
        let file_path_default = Path::new("/tmp/driver_distraction/driver_distraction_messages.json");
        let file_path_10sec = Path::new("/tmp/driver_distraction/driver_distraction_over10sec_messages.json");
        let mut last_modified_default: Option<SystemTime> = None;
        let mut last_modified_10sec: Option<SystemTime> = None;
        let mut over10sec_loaded_at: Option<Instant> = None;

        println!("📁 Started monitoring files: {:?} and {:?}", file_path_default, file_path_10sec);

        loop {
            std::thread::sleep(Duration::from_millis(500));

            let mut alert_to_set: Option<AlertMessage> = None;

            // 1. Check over10sec file first (priority)
            if file_path_10sec.exists() {
                if let Ok(metadata) = fs::metadata(file_path_10sec) {
                    if let Ok(modified) = metadata.modified() {
                        if last_modified_10sec != Some(modified) {
                            last_modified_10sec = Some(modified);
                            if let Ok(content) = fs::read_to_string(file_path_10sec) {
                                if !content.trim().is_empty() {
                                    if let Ok(mut alert) = serde_json::from_str::<AlertMessage>(&content) {
                                        alert.received_at = Some(Instant::now());
                                        let mut guard = file_monitor_state.lock().unwrap();
                                        guard.alert = Some(alert.clone());
                                        over10sec_loaded_at = Some(Instant::now());
                                    }
                                }
                            }
                        } else if let Some(loaded_at) = over10sec_loaded_at {
                            // If already loaded, check if 5s passed
                            if loaded_at.elapsed().as_secs() > 5 {
                                // Clear the file and also clear alert from state
                                let _ = fs::remove_file(file_path_10sec);
                                println!("🧹 Cleared over10sec alert file after 5s");
                                over10sec_loaded_at = None;
                                last_modified_10sec = None;
                                // Clear alert from state so default is not shown for 2-3s
                                let mut guard = file_monitor_state.lock().unwrap();
                                guard.alert = None;
                                continue;
                            } else {
                                // If still active, do not check default file
                                if let Some(loaded_at) = over10sec_loaded_at {
                                    if loaded_at.elapsed().as_secs() <= 5 {
                                        // Set the alert from previous load if not already set
                                        if alert_to_set.is_none() {
                                            // Try to reload the content if file still exists
                                            if let Ok(content) = fs::read_to_string(file_path_10sec) {
                                                if !content.trim().is_empty() {
                                                    if let Ok(mut alert) = serde_json::from_str::<AlertMessage>(&content) {
                                                        alert.received_at = Some(Instant::now());
                                                        alert_to_set = Some(alert);
                                                    }
                                                }
                                            }
                                        }
                                        // Skip default file
                                        let mut guard = file_monitor_state.lock().unwrap();
                                        if let Some(alert) = alert_to_set {
                                            guard.alert = Some(alert);
                                        }
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 2. If no over10sec alert, check default file
            else if file_path_default.exists() {
                if let Ok(metadata) = fs::metadata(file_path_default) {
                    if let Ok(modified) = metadata.modified() {
                        if last_modified_default != Some(modified) {
                            last_modified_default = Some(modified);
                            if let Ok(content) = fs::read_to_string(file_path_default) {
                                if !content.trim().is_empty() {
                                    if let Ok(mut alert) = serde_json::from_str::<AlertMessage>(&content) {
                                        alert.received_at = Some(Instant::now());
                                        // Set alert in state immediately
                                        let mut guard = file_monitor_state.lock().unwrap();
                                        guard.alert = Some(alert.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    // --- DDS SUBSCRIBER THREAD ---
    let dds_state = app_state.clone();
    
    // Create participant in MAIN thread
    let domain_id = 100;
    let participant_factory = DomainParticipantFactory::get_instance();
    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
        .expect("Failed to create participant");

    std::thread::spawn(move || {
        let topic_name = "DashboardData";
        let type_name = "DashboardData";

        let subscriber = participant
            .create_subscriber(QosKind::Default, None, NO_STATUS)
            .expect("Failed to create subscriber");

        let topic = participant
            .create_topic::<DashboardData>(
                topic_name,
                type_name,
                QosKind::Default,
                None,
                NO_STATUS,
            )
            .expect("Failed to create topic");

        let reader = subscriber
            .create_datareader::<DashboardData>(&topic, QosKind::Default, None, NO_STATUS)
            .expect("Failed to create datareader");

        println!("DDS Subscriber ready - waiting for DashboardData...");
        
        let mut publisher_discovered = false;
        loop {
            // Poll every 100ms
            std::thread::sleep(Duration::from_millis(100));

            let status = reader.get_subscription_matched_status().unwrap_or_default();
            if status.current_count > 0 {
                if !publisher_discovered {
                    println!("✅ Publisher discovered! {} publisher(s) matched", status.current_count);
                    publisher_discovered = true;
                }
            } else {
                 if publisher_discovered {
                    println!("⚠️  Publisher disconnected");
                    publisher_discovered = false;
                 }
            }

            // Try to read fresh data
            if let Ok(samples) = reader.take(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE) {
                for sample in samples {
                    if let Ok(data) = sample.data() {
                        // println!("📡 Received data: speed={:.1}", data.vehicle_speed);
                        let mut guard = dds_state.lock().unwrap();
                        guard.telemetry = Some(TimestampedData {
                            data: data.clone(),
                            received_at: Instant::now(),
                        });
                    }
                }
            }
        }
    });

    // Keep main thread alive
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

