// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// <https://www.apache.org/licenses/LICENSE-2.0>
//
// SPDX-License-Identifier: Apache-2.0

mod common;
use common::compare_kvs_values;
use rust_kvs::prelude::*;
use std::collections::HashMap;
use tempfile::tempdir;

/// Flush on exit is enabled by default.
/// Data will be flushed on `kvs` being dropped.
#[test]
// #[record_property("PartiallyVerifies", [])]
// #[record_property("FullyVerifies", ["comp_req__persistency__persist_data_store_com"])]
// #[record_property("Description", "Verifies that data is automatically flushed and persisted when the KVS instance is dropped, with flush on exit enabled.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
fn cit_persistency_flush_on_exit_enabled() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

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
        KvsValue::from("dbca".to_string()),
        KvsValue::from(()),
        KvsValue::from(vec![]),
        KvsValue::from(hashmap),
    ];
    kv_values.insert("array".to_string(), KvsValue::from(array));

    {
        // First KVS run.
        let kvs = Kvs::open(
            InstanceId(0),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;

        // Set values.
        for (key, value) in kv_values.iter() {
            kvs.set_value(key, value.clone())?;
        }
    }

    // Assertions.
    {
        // Second KVS run.
        // KVS file is expected to exist.
        let kvs = Kvs::open(
            InstanceId(0),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Required,
            Some(dir_string),
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
// #[record_property("PartiallyVerifies", [])]
// #[record_property("FullyVerifies", ["comp_req__persistency__persist_data_store_com"])]
// #[record_property("Description", "Checks that disabling flush on exit causes data to be dropped and not persisted after the KVS instance is dropped.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
fn cit_persistency_flush_on_exit_disabled_drop_data() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

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
        KvsValue::from("dbca".to_string()),
        KvsValue::from(()),
        KvsValue::from(vec![]),
        KvsValue::from(hashmap),
    ];
    kv_values.insert("array".to_string(), KvsValue::from(array));

    {
        // First KVS run.
        let kvs = Kvs::open(
            InstanceId(0),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;
        kvs.set_flush_on_exit(FlushOnExit::No);

        // Set values.
        for (key, value) in kv_values.iter() {
            kvs.set_value(key, value.clone())?;
        }
    }

    // Assertions.
    {
        // Second KVS run.
        // KVS file is expected to not to exist.
        let kvs = Kvs::open(
            InstanceId(0),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_string),
        )?;

        // Make sure no keys are defined.
        assert!(kvs.get_all_keys()?.is_empty());
    }

    Ok(())
}

#[test]
// #[record_property("PartiallyVerifies", [])]
// #[record_property("FullyVerifies", ["comp_req__persistency__persist_data_store_com"])]
// #[record_property("Description", "Verifies that disabling flush on exit but manually flushing ensures data is persisted correctly.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
fn cit_persistency_flush_on_exit_disabled_manual_flush() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

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
        KvsValue::from("dbca".to_string()),
        KvsValue::from(()),
        KvsValue::from(vec![]),
        KvsValue::from(hashmap),
    ];
    kv_values.insert("array".to_string(), KvsValue::from(array));

    {
        // First KVS run.
        let kvs = Kvs::open(
            InstanceId(0),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;
        kvs.set_flush_on_exit(FlushOnExit::No);

        // Set values.
        for (key, value) in kv_values.iter() {
            kvs.set_value(key, value.clone())?;
        }

        // Explicitly flush.
        kvs.flush()?;
    }

    // Assertions.
    {
        // Second KVS run.
        // KVS file is expected to exist.
        let kvs = Kvs::open(
            InstanceId(0),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Required,
            Some(dir_string),
        )?;

        // Compare values.
        for (key, expected_value) in kv_values.iter() {
            let actual_value = kvs.get_value(key).unwrap();
            assert!(compare_kvs_values(expected_value, &actual_value))
        }
    }

    Ok(())
}
