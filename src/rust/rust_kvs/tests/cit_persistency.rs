//! Persistency tests.
//!
//! Requirements verified:
//! - Persistency (feat_req__persistency__persistency)
//!   The KVS system shall persist stored data and provide an API to explicitly trigger persistence.
//! - Store persistent data (feat_req__persistency__persist_data)
//!   The KVS shall support storing and loading its data to and from persistent storage.

mod common;
use common::compare_kvs_values;
use rust_kvs::prelude::*;
use std::collections::HashMap;
use tempfile::tempdir;

/// Flush on exit is enabled by default.
/// Data will be flushed on `kvs` being dropped.
#[test]
fn cit_persistency_flush_on_exit_enabled() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_path = dir.path().to_path_buf();

    // Values of each type.
    let mut kv_values: HashMap<String, KvsValue> = HashMap::new();
    kv_values.insert("number".to_string(), KvsValue::from(123.4));
    kv_values.insert("bool".to_string(), KvsValue::from(true));
    kv_values.insert("str".to_string(), KvsValue::from("abcd".to_string()));
    kv_values.insert("null".to_string(), KvsValue::from(()));
    let hashmap = HashMap::from([("sub-number".to_string(), KvsValue::from(789.0))]);
    kv_values.insert("obj".to_string(), KvsValue::from(hashmap.clone()));
    let array = vec![
        KvsValue::from(321.0),
        KvsValue::from(false),
        KvsValue::from("example_string".to_string()),
        KvsValue::from(()),
        KvsValue::from(vec![]),
        KvsValue::from(hashmap),
    ];
    kv_values.insert("array".to_string(), KvsValue::from(array));

    {
        let mut kvs_provider = KvsProvider::new(dir_path.clone());
        // First KVS run.
        let kvs = kvs_provider.init(KvsParameters::new(InstanceId(0)))?;

        // Set values.
        for (key, value) in kv_values.iter() {
            kvs.set_value(key, value.clone())?;
        }
    }

    // Assertions.
    {
        let mut kvs_provider = KvsProvider::new(dir_path);
        // Second KVS run.
        // KVS file is expected to exist.
        let kvs =
            kvs_provider.init(KvsParameters::new(InstanceId(0)).kvs_load(KvsLoad::Required))?;

        // Compare values.
        for (key, expected_value) in kv_values.iter() {
            let actual_value = kvs.get_value(key).unwrap();
            assert!(compare_kvs_values(expected_value, &actual_value))
        }
    }

    Ok(())
}

#[test]
fn cit_persistency_flush_on_exit_disabled_drop_data() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_path = dir.path().to_path_buf();
    let mut kvs_provider = KvsProvider::new(dir_path);

    // Values of each type.
    let mut kv_values: HashMap<String, KvsValue> = HashMap::new();
    kv_values.insert("number".to_string(), KvsValue::from(123.4));
    kv_values.insert("bool".to_string(), KvsValue::from(true));
    kv_values.insert("str".to_string(), KvsValue::from("abcd".to_string()));
    kv_values.insert("null".to_string(), KvsValue::from(()));
    let hashmap = HashMap::from([("sub-number".to_string(), KvsValue::from(789.0))]);
    kv_values.insert("obj".to_string(), KvsValue::from(hashmap.clone()));
    let array = vec![
        KvsValue::from(321.0),
        KvsValue::from(false),
        KvsValue::from("example_string".to_string()),
        KvsValue::from(()),
        KvsValue::from(vec![]),
        KvsValue::from(hashmap),
    ];
    kv_values.insert("array".to_string(), KvsValue::from(array));

    {
        // First KVS run.
        let kvs =
            kvs_provider.init(KvsParameters::new(InstanceId(0)).flush_on_exit(FlushOnExit::No))?;

        // Set values.
        for (key, value) in kv_values.iter() {
            kvs.set_value(key, value.clone())?;
        }
    }

    // Assert file is not filled - no flush happened.
    let exp_json_path = dir.path().join("kvs_0_0.json");
    assert!(!exp_json_path.exists());

    Ok(())
}

#[test]
fn cit_persistency_flush_on_exit_disabled_manual_flush() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_path = dir.path().to_path_buf();

    // Values of each type.
    let mut kv_values: HashMap<String, KvsValue> = HashMap::new();
    kv_values.insert("number".to_string(), KvsValue::from(123.4));
    kv_values.insert("bool".to_string(), KvsValue::from(true));
    kv_values.insert("str".to_string(), KvsValue::from("abcd".to_string()));
    kv_values.insert("null".to_string(), KvsValue::from(()));
    let hashmap = HashMap::from([("sub-number".to_string(), KvsValue::from(789.0))]);
    kv_values.insert("obj".to_string(), KvsValue::from(hashmap.clone()));
    let array = vec![
        KvsValue::from(321.0),
        KvsValue::from(false),
        KvsValue::from("example_string".to_string()),
        KvsValue::from(()),
        KvsValue::from(vec![]),
        KvsValue::from(hashmap),
    ];
    kv_values.insert("array".to_string(), KvsValue::from(array));

    {
        let mut kvs_provider = KvsProvider::new(dir_path.clone());
        // First KVS run.
        let kvs =
            kvs_provider.init(KvsParameters::new(InstanceId(0)).flush_on_exit(FlushOnExit::No))?;

        // Set values.
        for (key, value) in kv_values.iter() {
            kvs.set_value(key, value.clone())?;
        }

        // Explicitly flush.
        kvs.flush()?;
    }

    // Assertions.
    {
        let mut kvs_provider = KvsProvider::new(dir_path);
        // Second KVS run.
        // KVS file is expected to exist.
        let kvs = kvs_provider.init(
            KvsParameters::new(InstanceId(0))
                .flush_on_exit(FlushOnExit::No)
                .kvs_load(KvsLoad::Required),
        )?;

        // Compare values.
        for (key, expected_value) in kv_values.iter() {
            let actual_value = kvs.get_value(key).unwrap();
            assert!(compare_kvs_values(expected_value, &actual_value))
        }
    }

    Ok(())
}

#[test]
fn cit_persistency_multiple_instances_shared_data() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_path = dir.path().to_path_buf();
    let mut kvs_provider = KvsProvider::new(dir_path);

    // Initialize first KVS instance.
    let kvs1 =
        kvs_provider.init(KvsParameters::new(InstanceId(0)).flush_on_exit(FlushOnExit::No))?;
    {
        kvs1.set_value("k1", KvsValue::from("v1".to_string()))?;
    }

    // Initialize seconds KVS instance.
    let kvs2 = kvs_provider.get(InstanceId(0))?;
    {
        kvs2.set_value("k2", KvsValue::from("v2".to_string()))?;
    }

    // Assert data is shared between objects of same InstanceID.
    assert_eq!(kvs1.get_value_as::<String>("k1")?, "v1".to_string());
    assert_eq!(kvs1.get_value_as::<String>("k2")?, "v2".to_string());
    assert_eq!(kvs2.get_value_as::<String>("k2")?, "v2".to_string());
    assert_eq!(kvs2.get_value_as::<String>("k1")?, "v1".to_string());

    Ok(())
}
