use dust_dds::domain::domain_participant_factory::DomainParticipantFactory;
use dust_dds::infrastructure::listeners::NoOpListener;
use dust_dds::infrastructure::qos::{DataWriterQos, QosKind};
use dust_dds::infrastructure::qos_policy::{
    DurabilityQosPolicy, DurabilityQosPolicyKind, ReliabilityQosPolicy,
    ReliabilityQosPolicyKind,
};
use dust_dds::infrastructure::status::NO_STATUS;
use dust_dds::infrastructure::time::{Duration, DurationKind};
use dust_dds::topic_definition::type_support::DdsType;
use serde::{Deserialize, Serialize};
use std::env;
use chrono::Utc;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Starting DDS Message Sender (Rust)");
    
    // Get configuration from environment variables
    let message_type = env::var("MESSAGE_TYPE").unwrap_or_else(|_| "driver_distraction_alert".to_string());
    let scenario_name = env::var("SCENARIO_NAME").unwrap_or_else(|_| "unknown_scenario".to_string());
    let dds_topic = env::var("DDS_TOPIC").unwrap_or_else(|_| "DriverDistractionEvents".to_string());
    let message_content = env::var("MESSAGE_CONTENT").unwrap_or_else(|_| "Distraction detected".to_string());
    let severity = env::var("SEVERITY").unwrap_or_else(|_| "warning".to_string());
    let threshold_value = env::var("THRESHOLD_VALUE")
        .unwrap_or_else(|_| "5.0".to_string())
        .parse::<f64>()
        .unwrap_or(5.0);
    
    println!("📋 Configuration:");
    println!("   Scenario: {}", scenario_name);
    println!("   Message Type: {}", message_type);
    println!("   DDS Topic: {}", dds_topic);
    println!("   Severity: {}", severity);
    println!("   Threshold: {}s", threshold_value);
    
    // Create DDS message
    let event_id = format!("{}_{}", scenario_name, chrono::Utc::now().timestamp());
    let message = DriverDistractionEvent {
        timestamp: Utc::now().to_rfc3339(),
        message_type: message_type.clone(),
        scenario_name: scenario_name.clone(),
        content: message_content.clone(),
        severity: severity.clone(),
        source: "pullpiri_rust_container".to_string(),
        event_id: event_id.clone(),
        distraction_duration: threshold_value,
        threshold_exceeded: true,
    };
    
    println!("📡 Created DDS message:");
    println!("   Event ID: {}", event_id);
    println!("   Content: {}", message_content);
    
    // Send DDS message
    match send_dds_message(&dds_topic, &message).await {
        Ok(_) => {
            println!("✅ DDS message sent successfully");
            // Also write to shared file for backward compatibility
            if let Err(e) = write_message_to_file(&message).await {
                println!("⚠️  Failed to write to shared file: {}", e);
            }
            println!("🎉 Container task completed successfully");
        }
        Err(e) => {
            eprintln!("❌ Failed to send DDS message: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

async fn send_dds_message(topic_name: &str, message: &DriverDistractionEvent) -> anyhow::Result<()> {
    let domain_id = 100; // Match the domain ID from your existing setup
    
    println!("🔗 Setting up DDS publisher...");
    
    // Create DDS participant
    let participant_factory = DomainParticipantFactory::get_instance();
    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .map_err(|e| anyhow::anyhow!("Failed to create DDS participant: {:?}", e))?;
    
    println!("   ✓ DDS participant created (domain {})", domain_id);
    
    // Create publisher
    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
        .map_err(|e| anyhow::anyhow!("Failed to create DDS publisher: {:?}", e))?;
    
    println!("   ✓ DDS publisher created");
    
    // Create topic
    let topic = participant
        .create_topic(
            topic_name,
            "DriverDistractionEvent",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .map_err(|e| anyhow::anyhow!("Failed to create DDS topic '{}': {:?}", topic_name, e))?;
    
    println!("   ✓ DDS topic '{}' created", topic_name);
    
    // Configure DataWriter QoS for best effort delivery (match subscriber)
    let writer_qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort, // Match subscriber BestEffort QoS
            max_blocking_time: DurationKind::Finite(Duration::new(1, 0)),
        },
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal, // Keep message for late joiners
        },
        history: dust_dds::infrastructure::qos_policy::HistoryQosPolicy {
            kind: dust_dds::infrastructure::qos_policy::HistoryQosPolicyKind::KeepLast(5),
        },
        ..Default::default()
    };
    
    // Create DataWriter
    let writer = publisher
        .create_datawriter::<DriverDistractionEvent>(&topic, QosKind::Specific(writer_qos), NoOpListener::new(), NO_STATUS)
        .map_err(|e| anyhow::anyhow!("Failed to create DDS DataWriter: {:?}", e))?;
    
    println!("   ✓ DDS DataWriter created with best effort QoS");
    
    // Wait longer for discovery and retry if needed
    println!("⏳ Waiting for subscriber discovery...");
    let mut attempts = 0;
    let max_attempts = 10;
    
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        attempts += 1;
        
        let matched_status = writer.get_publication_matched_status().unwrap_or_default();
        println!("   📊 Attempt {}/{}: Matched subscribers: {}", attempts, max_attempts, matched_status.current_count);
        
        if matched_status.current_count > 0 {
            println!("   ✅ Subscriber found! Proceeding with message publishing...");
            break;
        }
        
        if attempts >= max_attempts {
            println!("   ⚠️  No subscribers found after {} attempts, publishing anyway...", max_attempts);
            break;
        }
    }
    
    // Write the message
    println!("📤 Publishing DDS message...");
    writer
        .write(message, None)
        .map_err(|e| anyhow::anyhow!("Failed to write DDS message: {:?}", e))?;
    
    println!("   ✓ Message published to DDS topic '{}'", topic_name);
    
    // Wait a bit to ensure delivery
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Check final status
    let final_status = writer.get_publication_matched_status().unwrap_or_default();
    println!("   📊 Final matched subscribers: {}", final_status.current_count);
    
    Ok(())
}

async fn write_message_to_file(message: &DriverDistractionEvent) -> anyhow::Result<()> {
    use std::path::Path;
    
    let file_path = "/data/dds_messages.json";
    
    // Ensure directory exists
    if let Some(parent) = Path::new(file_path).parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    
    // Read existing messages
    let mut messages: Vec<DriverDistractionEvent> = if tokio::fs::metadata(file_path).await.is_ok() {
        let content = tokio::fs::read_to_string(file_path).await.unwrap_or_else(|_| "[]".to_string());
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    // Add new message
    messages.push(message.clone());

    // Keep only last 100 messages
    let len = messages.len();
    if len > 100 {
        messages = messages.into_iter().skip(len - 100).collect();
    }

    // Write back to file
    let json_content = serde_json::to_string_pretty(&messages)?;
    tokio::fs::write(file_path, json_content).await?;

    println!("   ✓ Message also written to shared file: {}", file_path);

    Ok(())
}