use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::Write;
use chrono::Utc;

#[derive(Clone, Debug, Serialize, Deserialize)]
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting File Message Writer (Rust)");
    
    // Get configuration from environment variables
    let message_type = env::var("MESSAGE_TYPE").unwrap_or_else(|_| "driver_distraction_alert".to_string());
    let scenario_name = env::var("SCENARIO_NAME").unwrap_or_else(|_| "unknown_scenario".to_string());
    let message_content = env::var("MESSAGE_CONTENT").unwrap_or_else(|_| "Distraction detected".to_string());
    let severity = env::var("SEVERITY").unwrap_or_else(|_| "warning".to_string());
    let threshold_value = env::var("THRESHOLD_VALUE")
        .unwrap_or_else(|_| "5.0".to_string())
        .parse::<f64>()
        .unwrap_or(5.0);
    
    println!("📋 Configuration:");
    println!("   Scenario: {}", scenario_name);
    println!("   Message Type: {}", message_type);
    println!("   Severity: {}", severity);
    println!("   Threshold: {}s", threshold_value);
    println!("   Content: {}", message_content);
    
    // Create message
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
    
    println!("📄 Created message:");
    println!("   Event ID: {}", event_id);
    println!("   Content: {}", message_content);
    
    // Write message to file
    match write_message_to_file(&message).await {
        Ok(_) => {
            println!("✅ Message written to file successfully");
            println!("🎉 Container task completed successfully");
        }
        Err(e) => {
            eprintln!("❌ Failed to write message to file: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

async fn write_message_to_file(message: &DriverDistractionEvent) -> Result<(), Box<dyn std::error::Error>> {
    // Use a different file if threshold is greater than or equal to 10
    let file_path = if message.distraction_duration >= 10.0 {
        "/data/driver_distraction_over10sec_messages.json"
    } else {
        "/data/driver_distraction_messages.json"
    };

    println!("📁 Writing to file: {}", file_path);

    // Ensure directory exists
    if let Some(parent_dir) = std::path::Path::new(file_path).parent() {
        fs::create_dir_all(parent_dir)?;
    }

    // Write the latest message (replace existing content)
    let json_content = serde_json::to_string_pretty(message)?;

    let mut file = fs::File::create(file_path)?;
    file.write_all(json_content.as_bytes())?;
    file.flush()?;

    println!("   ✓ Latest message written to: {}", file_path);

    // Also write to a timestamped history file
    let history_file = format!("/data/history/message_{}.json", message.event_id);
    if let Some(history_dir) = std::path::Path::new(&history_file).parent() {
        fs::create_dir_all(history_dir)?;
    }

    let mut history = fs::File::create(&history_file)?;
    history.write_all(json_content.as_bytes())?;
    history.flush()?;

    println!("   ✓ History saved to: {}", history_file);

    Ok(())
}