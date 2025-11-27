use dust_dds::domain::domain_participant_factory::DomainParticipantFactory;
use dust_dds::infrastructure::listeners::NoOpListener;
use dust_dds::infrastructure::qos::{DataReaderQos, QosKind};
use dust_dds::infrastructure::qos_policy::{
    DurabilityQosPolicy, DurabilityQosPolicyKind, ReliabilityQosPolicy,
    ReliabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
};
use dust_dds::infrastructure::status::{StatusKind, NO_STATUS};
use dust_dds::infrastructure::time::{Duration, DurationKind};
use dust_dds::infrastructure::wait_set::{Condition, WaitSet};
use dust_dds::subscription::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE};
use dust_dds::topic_definition::type_support::DdsType;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use warp::Filter;

#[derive(DdsType, Clone, Debug, Serialize, Deserialize)]
pub struct DriverDistractionEvent {
    pub timestamp: String,
    pub message_type: String,
    pub scenario_name: String,
    pub content: String,
    pub severity: String,
    pub source: String,
    pub event_id: String,
    pub distraction_duration: f64,
    pub threshold_exceeded: bool,
}

// Struct to hold data with timestamp for timeout management
#[derive(Clone, Debug)]
pub struct TimestampedData {
    pub data: DriverDistractionEvent,
    pub received_at: Instant,
}

#[tokio::main]
async fn main() {
    // For REST API, we need shared state access for latest data with timestamp
    let latest_data_for_api = Arc::new(Mutex::new(None::<TimestampedData>));
    
    let latest_data_filter = warp::any().map({
        let latest_data = latest_data_for_api.clone();
        move || latest_data.clone()
    });

    // REST endpoint: GET /data
    let get_data = warp::path("data")
        .and(warp::get())
        .and(latest_data_filter.clone())
        .map(|latest_data: Arc<Mutex<Option<TimestampedData>>>| {
            let mut data_guard = latest_data.lock().unwrap();
            
            // Check if data is older than 3 seconds and clear it if so
            let response = if let Some(ref timestamped_data) = *data_guard {
                let age = timestamped_data.received_at.elapsed();
                if age.as_secs() > 3 {
                    println!("⏰ Data expired ({}s old), clearing for UI alarm reset", age.as_secs());
                    *data_guard = None;
                    warp::reply::json(&serde_json::json!({"error": "No DDS messages received yet"}))
                } else {
                    warp::reply::json(&timestamped_data.data)
                }
            } else {
                warp::reply::json(&serde_json::json!({"error": "No DDS messages received yet"}))
            };
            
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

    // Health endpoint
    let health = warp::path("health")
        .and(warp::get())
        .and(latest_data_filter.clone())
        .map(|latest_data: Arc<Mutex<Option<TimestampedData>>>| {
            let data_guard = latest_data.lock().unwrap();
            let (has_data, data_age) = if let Some(ref timestamped_data) = *data_guard {
                let age = timestamped_data.received_at.elapsed();
                (true, age.as_secs())
            } else {
                (false, 0)
            };

            let response = serde_json::json!({
                "status": "healthy",
                "service": "dds-backend-receiver-rust",
                "data_available": has_data,
                "data_age_seconds": data_age,
                "timeout_threshold_seconds": 3
            });

            warp::reply::with_header(
                warp::reply::with_header(
                    warp::reply::with_header(warp::reply::json(&response), "Access-Control-Allow-Origin", "*"),
                    "Access-Control-Allow-Methods", "GET, POST, OPTIONS"
                ),
                "Access-Control-Allow-Headers", "Content-Type"
            )
        });

    let api = get_data.or(options_data).or(health);

    // Spawn REST API server in background (port 8081 for DDS Backend)
    let rest_handle = tokio::spawn(async move {
        println!("🌐 DDS Backend REST API running on:");
        println!("   - http://127.0.0.1:8081/data (latest DDS message)");
        warp::serve(api).run(([127, 0, 0, 1], 8081)).await;
    });

    // Spawn DDS subscriber in background task
    let latest_data_sub = latest_data_for_api.clone();
    let dds_handle = tokio::spawn(async move {
        let domain_id = 100;
        let topic_name = "DriverDistractionEvents";
        let type_name = "DriverDistractionEvent";

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
                max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
            },
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal, // Keep for historical data
            },
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast(5),
            },
            ..Default::default()
        };

        let reader = subscriber
            .create_datareader::<DriverDistractionEvent>(&topic, QosKind::Specific(reader_qos), NoOpListener::new(), NO_STATUS)
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

        println!("DDS Subscriber ready - waiting for DriverDistractionEvents...");
        
        let mut publisher_discovered = false;
        loop {
            // Wait for discovery or data
            match wait_set.wait(Duration::new(5, 0)) {
                Ok(_) => {
                    // Check subscription status
                    let subscription_matched_status = reader.get_subscription_matched_status().unwrap_or_default();
                    if subscription_matched_status.current_count > 0 && !publisher_discovered {
                        println!("✅ Publisher discovered! {} publisher(s) matched", subscription_matched_status.current_count);
                        publisher_discovered = true;
                        
                        // Wait for transient local data to be available
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        
                        // Try to read any historical data
                        let historical_samples = reader
                            .take(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
                            .unwrap_or_default();
                        
                        if !historical_samples.is_empty() {
                            println!("📜 Found {} historical DDS messages", historical_samples.len());
                            for sample in historical_samples {
                                if let Ok(data) = sample.data() {
                                    println!("📡 Received historical message: {} - {}", 
                                        data.scenario_name, data.content);
                                    
                                    // Update API data with timestamp
                                    *latest_data_sub.lock().unwrap() = Some(TimestampedData {
                                        data: data.clone(),
                                        received_at: Instant::now(),
                                    });
                                }
                            }
                        } else {
                            println!("📜 No historical data found - waiting for fresh messages");
                        }
                    }
                    
                    // Try to read fresh data
                    let samples = reader
                        .take(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
                        .unwrap_or_default();

                    for sample in samples {
                        if let Ok(data) = sample.data() {
                            println!("📡 Received fresh DDS message: {} - {} ({})", 
                                data.scenario_name, data.content, data.severity);
                            
                            // Update shared state for REST API with current timestamp
                            *latest_data_sub.lock().unwrap() = Some(TimestampedData {
                                data: data.clone(),
                                received_at: Instant::now(),
                            });
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

