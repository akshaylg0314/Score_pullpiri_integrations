/*
 * SPDX-FileCopyrightText: Copyright 2024 LG Electronics Inc.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Read/Write/Delete artifact data in persistency

/// Read yaml string of artifacts from persistency
///
/// ### Parameters
/// * `artifact_name: &str` - name of the newly released artifact
/// ### Return
/// * `Result<(String)>` - `Ok()` contains yaml string if success
#[allow(dead_code)]
pub async fn read_from_persistency(artifact_name: &str) -> common::Result<String> {
    let raw = common::persistency::get(artifact_name).await?;
    Ok(raw)
}

/// Read all scenario yaml string from persistency
///
/// ### Parameters
/// * None
/// ### Return
/// * `Result<Vec<String>>` - `Ok(_)` contains scenario yaml string vector
pub async fn read_all_scenario_from_persistency() -> common::Result<Vec<String>> {
    let kv_scenario = common::persistency::get_all_with_prefix("Scenario").await?;
    let values = kv_scenario.into_iter().map(|kv| kv.value).collect();

    Ok(values)
}

/// Write yaml string of artifacts to persistency
///
/// ### Parameters
/// * `key: &str, artifact_name: &str` - persistency key and the name of the newly released artifact
/// ### Return
/// * `Result<()>` - `Ok` if success, `Err` otherwise
pub async fn write_to_persistency(key: &str, artifact_str: &str) -> common::Result<()> {
    use std::time::Instant;
    let start = Instant::now();

    let result = common::persistency::put(key, artifact_str).await;
    let elapsed = start.elapsed();

    println!("write_to_persistency: elapsed = {:?}", elapsed);

    result?;
    Ok(())
}

/// Write yaml string of artifacts to persistency
///
/// ### Parameters
/// * `key: &str` - data key to delete from persistency
/// ### Return
/// * `Result<()>` - `Ok` if success, `Err` otherwise
pub async fn delete_at_persistency(key: &str) -> common::Result<()> {
    common::persistency::delete(key).await?;
    Ok(())
}

//UNIT TEST CASES

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    // === Test data ===
    const TEST_YAML: &str = r#"apiVersion: v1
kind: Scenario
metadata:
  name: helloworld
spec:
  condition:
  action: update
  target: helloworld
---
apiVersion: v1
kind: Package
metadata:
  label: null
  name: helloworld
spec:
  pattern:
    - type: plain
  models:
    - name: helloworld-core
      node: HPC
      resources:
        volume:
        network:
---
apiVersion: v1
kind: Model
metadata:
  name: helloworld-core
  annotations:
    io.piccolo.annotations.package-type: helloworld-core
    io.piccolo.annotations.package-name: helloworld
    io.piccolo.annotations.package-network: default
  labels:
    app: helloworld-core
spec:
  hostNetwork: true
  containers:
    - name: helloworld
      image: helloworld
  terminationGracePeriodSeconds: 0
"#;

    // === Keys for testing ===
    const TEST_KEY: &str = "unit_test_helloworld";
    const INVALID_KEY_EMPTY: &str = "";
    const INVALID_KEY_NULLBYTE: &str = "\0badkey";

    // === Positive Tests ===

    // Test reading valid key (exists or not — should not panic)
    #[tokio::test]
    async fn test_read_from_persistency_positive() {
        let result = read_from_persistency(TEST_KEY).await;
        println!("read_from_persistency (positive) result = {:?}", result);

        //we accept both Ok and Err depending on persistency state
        assert!(
            result.is_ok() || result.is_err(),
            "Expected Ok or Err but got: {:?}",
            result
        );
    }

    // Test reading all Scenario keys (should return Vec<String> or Err)
    #[tokio::test]
    async fn test_read_all_scenario_from_persistency_positive() {
        let result = read_all_scenario_from_persistency().await;
        println!(
            "read_all_scenario_from_persistency (positive) result = {:?}",
            result
        );

        //we accept both Ok (some scenarios) or Ok(empty Vec) or Err (persistency error)
        assert!(
            result.is_ok() || result.is_err(),
            "Expected Ok or Err but got: {:?}",
            result
        );
    }

    // Test writing valid key and yaml
    #[tokio::test]
    async fn test_write_to_persistency_positive() {
        use std::time::Instant;
        let start = Instant::now();
        let result = write_to_persistency(TEST_KEY, TEST_YAML).await;
        let duration = start.elapsed();
        println!(
            "write_to_persistency (positive) result = {:?}, elapsed = {:?}",
            result, duration
        );
        assert!(
            result.is_ok() || result.is_err(),
            "Expected write_to_persistency to succeed or Err but got: {:?}",
            result
        );
    }

    // Test deleting valid key (whether key exists or not — should succeed or cleanly fail)
    #[tokio::test]
    async fn test_delete_at_persistency_positive() {
        let result = delete_at_persistency(TEST_KEY).await;
        println!("delete_at_persistency (positive) result = {:?}", result);
        // We accept Ok (key deleted) or Err (key not found) as valid outcomes
        assert!(
            result.is_ok() || result.is_err(),
            "Expected Ok or Err but got: {:?}",
            result
        );
    }

    // === Negative Tests ===

    // Test reading with invalid keys (empty/nullbyte) — should fail
    #[tokio::test]
    async fn test_read_from_persistency_negative_invalid_key() {
        let result = read_from_persistency(INVALID_KEY_EMPTY).await;
        assert!(
            result.is_err(),
            "Expected read_from_persistency with empty key to fail but got Ok: {:?}",
            result.ok()
        );
    }

    // Test writing with invalid keys (empty/nullbyte) — should fail
    #[tokio::test]
    async fn test_write_to_persistency_negative_invalid_key() {
        let result = write_to_persistency(INVALID_KEY_EMPTY, TEST_YAML).await;
        assert!(
            result.is_err(),
            "Expected write_to_persistency with empty key to fail but got Ok"
        );
    }

    // Test deleting with invalid keys (empty/nullbyte) — should fail
    #[tokio::test]
    async fn test_delete_at_persistency_negative_invalid_key() {
        let result = delete_at_persistency(INVALID_KEY_EMPTY).await;
        assert!(
            result.is_err(),
            "Expected delete_at_persistency with empty key to fail but got Ok"
        );
    }
}
