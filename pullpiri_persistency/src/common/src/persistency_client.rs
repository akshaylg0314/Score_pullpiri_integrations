/*
 * SPDX-FileCopyrightText: Copyright 2024 LG Electronics Inc.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Persistency Service Client
//!
//! This module provides a client interface to the persistency service,
//! replacing direct PERSISTENCY usage with gRPC calls to the persistency service.

use crate::persistency_proto::{
    persistency_service_client::PersistencyServiceClient, GetAllWithPrefixRequest,
    GetValueRequest, KvsValue, RemoveKeyRequest, ResetRequest, SetValueRequest,
    FlushRequest,
};
use tonic::transport::{Channel, Error as TonicError};
use tonic::Status;

/// Key-Value pair for compatibility with existing code
#[derive(Debug, Clone)]
pub struct KV {
    pub key: String,
    pub value: String,
}

/// Client for the persistency service
pub struct PersistencyClient {
    client: PersistencyServiceClient<Channel>,
}

/// Custom error type for persistency operations
#[derive(Debug)]
pub enum PersistencyError {
    Transport(TonicError),
    Grpc(Status),
    Conversion(String),
    NotFound,
    InvalidArgs(String),
}

impl From<TonicError> for PersistencyError {
    fn from(err: TonicError) -> Self {
        PersistencyError::Transport(err)
    }
}

impl From<Status> for PersistencyError {
    fn from(err: Status) -> Self {
        PersistencyError::Grpc(err)
    }
}

impl std::fmt::Display for PersistencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistencyError::Transport(e) => write!(f, "Transport error: {}", e),
            PersistencyError::Grpc(e) => write!(f, "gRPC error: {}", e),
            PersistencyError::Conversion(e) => write!(f, "Conversion error: {}", e),
            PersistencyError::NotFound => write!(f, "Key not found"),
            PersistencyError::InvalidArgs(e) => write!(f, "Invalid arguments: {}", e),
        }
    }
}

impl std::error::Error for PersistencyError {}

impl PersistencyClient {
    /// Create a new persistency client
    pub async fn new() -> Result<Self, PersistencyError> {
        let endpoint = crate::persistency_proto::connect_server();
        let client = PersistencyServiceClient::connect(endpoint).await?;
        
        Ok(Self { client })
    }

    /// Helper function to convert string to KvsValue
    fn string_to_kvs_value(value: &str) -> KvsValue {
        KvsValue {
            value: Some(crate::persistency_proto::kvs_value::Value::StringValue(value.to_string())),
        }
    }

    /// Helper function to convert KvsValue to string
    fn kvs_value_to_string(value: &KvsValue) -> Result<String, PersistencyError> {
        match &value.value {
            Some(crate::persistency_proto::kvs_value::Value::StringValue(s)) => Ok(s.clone()),
            Some(crate::persistency_proto::kvs_value::Value::I32Value(v)) => Ok(v.to_string()),
            Some(crate::persistency_proto::kvs_value::Value::U32Value(v)) => Ok(v.to_string()),
            Some(crate::persistency_proto::kvs_value::Value::I64Value(v)) => Ok(v.to_string()),
            Some(crate::persistency_proto::kvs_value::Value::U64Value(v)) => Ok(v.to_string()),
            Some(crate::persistency_proto::kvs_value::Value::F64Value(v)) => Ok(v.to_string()),
            Some(crate::persistency_proto::kvs_value::Value::BooleanValue(v)) => Ok(v.to_string()),
            Some(crate::persistency_proto::kvs_value::Value::NullValue(_)) => Ok("null".to_string()),
            Some(crate::persistency_proto::kvs_value::Value::ArrayValue(_)) => {
                // For arrays and objects, we'll serialize to JSON string
                Err(PersistencyError::Conversion("Complex types not supported in string conversion".to_string()))
            }
            Some(crate::persistency_proto::kvs_value::Value::ObjectValue(_)) => {
                // For arrays and objects, we'll serialize to JSON string
                Err(PersistencyError::Conversion("Complex types not supported in string conversion".to_string()))
            }
            None => Err(PersistencyError::Conversion("Empty value".to_string())),
        }
    }

    /// Set a key-value pair
    pub async fn put(&mut self, key: &str, value: &str) -> Result<(), PersistencyError> {
        // Validate key similar to original implementation
        if key.len() > 1024 {
            return Err(PersistencyError::InvalidArgs(
                "Key exceeds maximum allowed length of 1024 characters".to_string(),
            ));
        }

        if key.contains(['<', '>', '?', '{', '}']) {
            return Err(PersistencyError::InvalidArgs(
                "Key contains invalid special characters".to_string(),
            ));
        }

        let request = SetValueRequest {
            key: key.to_string(),
            value: Some(Self::string_to_kvs_value(value)),
        };

        let response = self.client.set_value(request).await?;
        let response = response.into_inner();
        
        if response.success {
            Ok(())
        } else {
            Err(PersistencyError::InvalidArgs(response.error_message))
        }
    }

    /// Get a value by key
    pub async fn get(&mut self, key: &str) -> Result<String, PersistencyError> {
        // Validate key similar to original implementation
        if key.is_empty() {
            return Err(PersistencyError::InvalidArgs("Key cannot be empty".to_string()));
        }

        if key.len() > 1024 {
            return Err(PersistencyError::InvalidArgs(
                "Key exceeds maximum allowed length of 1024 characters".to_string(),
            ));
        }

        if key.contains(['<', '>', '?', '{', '}']) {
            return Err(PersistencyError::InvalidArgs(
                "Key contains invalid special characters".to_string(),
            ));
        }

        let request = GetValueRequest {
            key: key.to_string(),
        };

        let response = self.client.get_value(request).await?;
        let response = response.into_inner();
        
        if response.success {
            if let Some(value) = response.value {
                Self::kvs_value_to_string(&value)
            } else {
                Err(PersistencyError::NotFound)
            }
        } else {
            Err(PersistencyError::NotFound)
        }
    }

    /// Get all key-value pairs with a given prefix
    pub async fn get_all_with_prefix(&mut self, prefix: &str) -> Result<Vec<KV>, PersistencyError> {
        let request = GetAllWithPrefixRequest {
            prefix: prefix.to_string(),
        };

        let response = self.client.get_all_with_prefix(request).await?;
        let response = response.into_inner();
        
        if response.success {
            let mut kv_pairs = Vec::new();
            for (key, value) in response.key_values {
                match Self::kvs_value_to_string(&value) {
                    Ok(value_str) => {
                        kv_pairs.push(KV {
                            key,
                            value: value_str,
                        });
                    }
                    Err(_) => {
                        // Skip complex values that can't be converted to strings
                        continue;
                    }
                }
            }
            Ok(kv_pairs)
        } else {
            Err(PersistencyError::InvalidArgs(response.error_message))
        }
    }

    /// Delete a key
    pub async fn delete(&mut self, key: &str) -> Result<(), PersistencyError> {
        // Validate key similar to original implementation
        if key.len() > 1024 {
            return Err(PersistencyError::InvalidArgs(
                "Key exceeds maximum allowed length of 1024 characters".to_string(),
            ));
        }

        if key.contains(['<', '>', '?', '{', '}']) {
            return Err(PersistencyError::InvalidArgs(
                "Key contains invalid special characters".to_string(),
            ));
        }

        let request = RemoveKeyRequest {
            key: key.to_string(),
        };

        let response = self.client.remove_key(request).await?;
        let response = response.into_inner();
        
        if response.success {
            Ok(())
        } else {
            Err(PersistencyError::InvalidArgs(response.error_message))
        }
    }

    /// Delete all keys with a given prefix
    pub async fn delete_all_with_prefix(&mut self, prefix: &str) -> Result<(), PersistencyError> {
        // First get all keys with the prefix
        let kv_pairs = self.get_all_with_prefix(prefix).await?;
        
        // Then delete each key individually
        for kv in kv_pairs {
            self.delete(&kv.key).await?;
        }
        
        Ok(())
    }

    /// Reset all data (for testing/development)
    pub async fn reset(&mut self) -> Result<(), PersistencyError> {
        let request = ResetRequest {};
        let response = self.client.reset(request).await?;
        let response = response.into_inner();
        
        if response.success {
            Ok(())
        } else {
            Err(PersistencyError::InvalidArgs(response.error_message))
        }
    }

    /// Flush data to persistent storage
    pub async fn flush(&mut self) -> Result<(), PersistencyError> {
        let request = FlushRequest {};
        let response = self.client.flush(request).await?;
        let response = response.into_inner();
        
        if response.success {
            Ok(())
        } else {
            Err(PersistencyError::InvalidArgs(response.error_message))
        }
    }
}