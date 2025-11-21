use dust_dds::domain::domain_participant_factory::DomainParticipantFactory;
use dust_dds::infrastructure::listeners::NoOpListener;
use dust_dds::infrastructure::qos::{DataReaderQos, QosKind};
use dust_dds::infrastructure::qos_policy::{
    DurabilityQosPolicy, DurabilityQosPolicyKind, ReliabilityQosPolicy,
    ReliabilityQosPolicyKind,
};
use dust_dds::infrastructure::status::{StatusKind, NO_STATUS};
use dust_dds::infrastructure::time::{Duration, DurationKind};
use dust_dds::infrastructure::wait_set::{Condition, WaitSet};
use dust_dds::subscription::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE};
use dust_dds::topic_definition::type_support::DdsType;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use tokio::signal;
use warp::Filter;

#[derive(DdsType, Clone, Debug, Serialize, Deserialize)]
pub struct AutonomousCarData {
    pub vehicle_speed: f64,
    pub lane_position: f64,
    pub obstacle_detected: bool,
    pub obstacle_distance: f64,
    pub traffic_signal: String,
    pub steering_angle: f64,
    pub brake_force: f64,
    pub acceleration: f64,
    pub weather_condition: String,
    pub road_condition: String,
    pub timestamp: i64,
    pub is_valid: bool,
}

#[tokio::main]
async fn main() {
    // Shared state for latest data
    let latest_data = Arc::new(Mutex::new(None::<AutonomousCarData>));
    let latest_data_filter = warp::any().map({
        let latest_data = latest_data.clone();
        move || latest_data.clone()
    });

    // REST endpoint: GET /data
    let get_data = warp::path("data")
        .and(warp::get())
        .and(latest_data_filter.clone())
        .map(|latest_data: Arc<Mutex<Option<AutonomousCarData>>>| {
            let data = latest_data.lock().unwrap();
            let response = if let Some(ref d) = *data {
                warp::reply::json(d)
            } else {
                warp::reply::json(&serde_json::json!({"error": "No autonomous car data available yet"}))
            };
            
            warp::reply::with_header(
                warp::reply::with_header(
                    warp::reply::with_header(response, "Access-Control-Allow-Origin", "*"),
                    "Access-Control-Allow-Methods", "GET, POST, OPTIONS"
                ),
                "Access-Control-Allow-Headers", "Content-Type"
            )
        });

    // OPTIONS handler for CORS preflight
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

    let api = get_data.or(options_data);

    // Spawn REST API server in background (port 9083 for autonomous)
    let rest_handle = tokio::spawn(async move {
        println!("Autonomous Car Data REST API running on http://localhost:9083/data");
        warp::serve(api).run(([0, 0, 0, 0], 9083)).await;
    });

    // Spawn DDS subscriber in background task
    let latest_data_sub = latest_data.clone();
    let dds_handle = tokio::spawn(async move {
        let domain_id = 100;
        let topic_name = "AutonomousCarData";
        let type_name = "AutonomousCarData";

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
            .create_datareader::<AutonomousCarData>(&topic, QosKind::Specific(reader_qos), NoOpListener::new(), NO_STATUS)
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

        println!("Autonomous DDS Subscriber ready - waiting for data...");
        
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
                                    println!("📡 Received historical autonomous data: speed={}, distance={:.1}m", 
                                        data.vehicle_speed, data.obstacle_distance);
                                    *latest_data_sub.lock().unwrap() = Some(data.clone());
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
                                        println!("📡 Received historical autonomous data (retry): speed={}, distance={:.1}m", 
                                            data.vehicle_speed, data.obstacle_distance);
                                        *latest_data_sub.lock().unwrap() = Some(data.clone());
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
                            println!("📡 Received fresh autonomous data: speed={}, distance={:.1}m", 
                                data.vehicle_speed, data.obstacle_distance);
                            // Update shared state for REST API
                            *latest_data_sub.lock().unwrap() = Some(data.clone());
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
