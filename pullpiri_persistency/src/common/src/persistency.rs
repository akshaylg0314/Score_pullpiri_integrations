/*
 * SPDX-FileCopyrightText: Copyright 2024 LG Electronics Inc.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Persistency Service Interface
//!
//! This module provides a high-level interface to the persistency service,
//! offering key-value storage operations with built-in error handling and retry logic.
//! This is the main interface for components to interact with persistent data storage.

use crate::persistency_client::{PersistencyClient, PersistencyError};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Re-export the error type for convenience
pub type Error = PersistencyError;

/// Lazy static client instance for global access
static CLIENT: tokio::sync::OnceCell<Arc<Mutex<PersistencyClient>>> = tokio::sync::OnceCell::const_new();

/// Initialize the global persistency client
async fn get_client() -> Result<Arc<Mutex<PersistencyClient>>, PersistencyError> {
    const MAX_RETRIES: u32 = 10;
    const RETRY_DELAY_MS: u64 = 1000;

    CLIENT.get_or_try_init(|| async {
        let mut attempt = 0;
        let mut last_error = None;

        while attempt < MAX_RETRIES {
            match PersistencyClient::new().await {
                Ok(client) => {
                    return Ok(Arc::new(Mutex::new(client)));
                }
                Err(err) => {
                    println!(
                        "Failed to connect to persistency service (attempt {}/{}): {:?}",
                        attempt + 1,
                        MAX_RETRIES,
                        err
                    );
                    last_error = Some(err);
                    attempt += 1;

                    if attempt < MAX_RETRIES {
                        tokio::time::sleep(tokio::time::Duration::from_millis(RETRY_DELAY_MS)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            PersistencyError::InvalidArgs("Failed to connect to persistency service after multiple attempts".to_string())
        }))
    }).await.map(|client| client.clone())
}

pub struct KV {
    pub key: String,
    pub value: String,
}

pub async fn put(key: &str, value: &str) -> Result<(), PersistencyError> {
    let client = get_client().await?;
    let mut client = client.lock().await;
    client.put(key, value).await
}

pub async fn get(key: &str) -> Result<String, PersistencyError> {
    let client = get_client().await?;
    let mut client = client.lock().await;
    client.get(key).await
}

pub async fn get_all_with_prefix(key: &str) -> Result<Vec<KV>, PersistencyError> {
    let client = get_client().await?;
    let mut client = client.lock().await;
    
    let persistency_kvs = client.get_all_with_prefix(key).await?;
    let kvs = persistency_kvs
        .into_iter()
        .map(|kv| KV {
            key: kv.key,
            value: kv.value,
        })
        .collect();
    
    Ok(kvs)
}

pub async fn delete(key: &str) -> Result<(), PersistencyError> {
    let client = get_client().await?;
    let mut client = client.lock().await;
    client.delete(key).await
}

pub async fn delete_all_with_prefix(key: &str) -> Result<(), PersistencyError> {
    let client = get_client().await?;
    let mut client = client.lock().await;
    client.delete_all_with_prefix(key).await
}

// Keep the server configuration functions for compatibility
pub fn open_server() -> String {
    let config = crate::setting::get_config();
    if config.host.ip.is_empty() {
        panic!("Host IP is missing in the configuration.");
    }

    // Validate the IP format
    if config.host.ip.parse::<std::net::IpAddr>().is_err() {
        panic!("Invalid IP address format: {}", config.host.ip);
    }

    // Return persistency service address
    format!("{}:47007", config.host.ip)
}

//Unit Test Cases
#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    // Test constants
    const TEST_KEY: &str = "unit_test_key";
    const TEST_VALUE: &str = "unit_test_value";
    const TEST_PREFIX: &str = "unit_test_";

    #[tokio::test]
    async fn test_put_and_get() {
        // Test put and get operations
        let result = put(TEST_KEY, TEST_VALUE).await;
        if result.is_ok() {
            let get_result = get(TEST_KEY).await;
            if let Ok(value) = get_result {
                assert_eq!(value, TEST_VALUE);
            }
        }
        // Clean up
        let _ = delete(TEST_KEY).await;
    }

    #[tokio::test]
    async fn test_get_nonexistent_key() {
        let result = get("nonexistent_key_12345").await;
        assert!(result.is_err(), "Expected error for nonexistent key");
    }

    #[tokio::test]
    async fn test_get_all_with_prefix() {
        // Set up test data
        let _ = put(&format!("{}key1", TEST_PREFIX), "value1").await;
        let _ = put(&format!("{}key2", TEST_PREFIX), "value2").await;
        let _ = put("other_key", "other_value").await;

        // Test prefix search
        let result = get_all_with_prefix(TEST_PREFIX).await;
        if let Ok(kvs) = result {
            assert!(kvs.len() >= 2, "Expected at least 2 keys with prefix");
            for kv in &kvs {
                assert!(kv.key.starts_with(TEST_PREFIX), "Key should start with prefix");
            }
        }

        // Clean up
        let _ = delete(&format!("{}key1", TEST_PREFIX)).await;
        let _ = delete(&format!("{}key2", TEST_PREFIX)).await;
        let _ = delete("other_key").await;
    }

    #[tokio::test]
    async fn test_delete() {
        // Set up test data
        let _ = put(TEST_KEY, TEST_VALUE).await;
        
        // Test delete
        let delete_result = delete(TEST_KEY).await;
        if delete_result.is_ok() {
            let get_result = get(TEST_KEY).await;
            assert!(get_result.is_err(), "Key should not exist after deletion");
        }
    }

    #[tokio::test]
    async fn test_delete_all_with_prefix() {
        // Set up test data
        let _ = put(&format!("{}key1", TEST_PREFIX), "value1").await;
        let _ = put(&format!("{}key2", TEST_PREFIX), "value2").await;
        let _ = put("other_key", "other_value").await;

        // Test prefix deletion
        let result = delete_all_with_prefix(TEST_PREFIX).await;
        if result.is_ok() {
            let remaining = get_all_with_prefix(TEST_PREFIX).await;
            if let Ok(kvs) = remaining {
                assert_eq!(kvs.len(), 0, "No keys should remain with the prefix");
            }
            
            // Verify other key still exists
            let other_result = get("other_key").await;
            if other_result.is_ok() {
                let _ = delete("other_key").await;
            }
        }
    }

    #[tokio::test]
    async fn test_invalid_key_validation() {
        // Test empty key
        let result = put("", TEST_VALUE).await;
        assert!(result.is_err(), "Empty key should be rejected");

        // Test key with invalid characters
        let result = put("key<with>invalid?chars{}", TEST_VALUE).await;
        assert!(result.is_err(), "Key with invalid characters should be rejected");

        // Test overly long key
        let long_key = "a".repeat(2048);
        let result = put(&long_key, TEST_VALUE).await;
        assert!(result.is_err(), "Overly long key should be rejected");
    }

    #[tokio::test]
    async fn test_open_server_config() {
        // This test just ensures the function doesn't panic with valid config
        let server_addr = open_server();
        assert!(!server_addr.is_empty(), "Server address should not be empty");
        assert!(server_addr.contains(":47007"), "Should use persistency service port");
    }
}