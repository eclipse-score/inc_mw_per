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

use rust_kvs::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use tempfile::tempdir;
use tinyjson::{JsonGenerator, JsonValue};

fn write_defaults_file(
    dir_string: &Path,
    data: HashMap<String, JsonValue>,
    instance: InstanceId,
) -> Result<(), ErrorCode> {
    let filepath = dir_string.join(format!("kvs_{instance}_default.json"));

    // Convert HashMap<String, JsonValue> to t-tagged format
    let mut tagged_map = HashMap::new();
    for (k, v) in data.into_iter() {
        let t = match &v {
            JsonValue::Number(_) => "f64", // always treat as f64 for compatibility
            JsonValue::Boolean(_) => "bool",
            JsonValue::String(_) => "str",
            JsonValue::Array(_) => "arr",
            JsonValue::Object(_) => "obj",
            JsonValue::Null => "null",
        };
        let mut tagged = HashMap::new();
        tagged.insert("v".to_string(), v);
        tagged.insert("t".to_string(), JsonValue::String(t.to_string()));
        tagged_map.insert(k, JsonValue::Object(tagged));
    }
    let json = JsonValue::Object(tagged_map);
    let mut buf = Vec::new();
    let mut gen = JsonGenerator::new(&mut buf).indent("  ");
    gen.generate(&json)?;

    let data = String::from_utf8(buf)?;
    std::fs::write(filepath, &data)?;
    Ok(())
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__value_default", "comp_req__persistency__default_value_config", "comp_req__persistency__default_value_types", "comp_req__persistency__default_value_query"])]
// #[record_property("FullyVerifies", [])]
// #[record_property("Description", "Verifies default value loading, querying, and override behavior for KVS instances with and without defaults.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
fn cit_persistency_default_values() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Values.
    let keyname = "test_number".to_string();
    let default_value = 111.1;
    let non_default_value = 333.3;

    // Create defaults file for instance 0.
    let default_id = InstanceId(0);
    let non_default_id = InstanceId(1);
    write_defaults_file(
        dir.path(),
        HashMap::from([(keyname.clone(), JsonValue::from(default_value))]),
        default_id,
    )?;

    // Assertions.
    {
        // KVS instance with defaults.
        let kvs_with_defaults = Kvs::open(
            default_id,
            OpenNeedDefaults::Required,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;
        // KVS instance without defaults.
        let kvs_without_defaults = Kvs::open(
            non_default_id,
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;
        // Check defaults.
        assert!(
            kvs_with_defaults.is_value_default(&keyname)?,
            "kvs_with_defaults: key '{keyname}' should be default"
        );
        assert_eq!(
            kvs_with_defaults.get_default_value(&keyname)?,
            KvsValue::from(default_value),
            "kvs_with_defaults: key '{keyname}' should have default value {default_value}"
        );
        assert_eq!(
            kvs_without_defaults.is_value_default(&keyname).unwrap_err(),
            ErrorCode::KeyNotFound,
            "kvs_without_defaults: key '{keyname}' should not exist and return KeyNotFound"
        );
        assert_eq!(
            kvs_without_defaults
                .get_default_value(&keyname)
                .unwrap_err(),
            ErrorCode::KeyNotFound,
            "kvs_without_defaults: key '{keyname}' should not exist and return KeyNotFound"
        );

        // Check values.
        assert_eq!(
            kvs_with_defaults.get_value_as::<f64>(&keyname)?,
            default_value,
            "kvs_with_defaults: key '{keyname}' should have default value {default_value}"
        );
        assert_eq!(
            kvs_without_defaults
                .get_value_as::<f64>(&keyname)
                .unwrap_err(),
            ErrorCode::KeyNotFound,
            "kvs_without_defaults: key '{keyname}' should not exist and return KeyNotFound"
        );
        // Set non-default value to both KVS instances.
        kvs_with_defaults.set_value(&keyname, non_default_value)?;
        kvs_without_defaults.set_value(&keyname, non_default_value)?;
        // Check that the value is non-default.
        assert!(
            !kvs_with_defaults.is_value_default(&keyname)?,
            "kvs_with_defaults: key '{keyname}' should NOT be default after set"
        );
        assert!(
            !kvs_without_defaults.is_value_default(&keyname)?,
            "kvs_without_defaults: key '{keyname}' should NOT be default after set"
        );
    }
    // Flush and reopen KVS instances to ensure persistency.
    {
        // KVS instance with defaults.
        let kvs_with_defaults = Kvs::open(
            default_id,
            OpenNeedDefaults::Required,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;
        // KVS instance without defaults.
        let kvs_without_defaults = Kvs::open(
            non_default_id,
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;
        // Check that the value is still non-default.
        assert_eq!(
            kvs_with_defaults.get_value_as::<f64>(&keyname)?,
            non_default_value,
            "kvs_with_defaults: key '{keyname}' should persist non-default value {non_default_value} after reopen"
        );
        assert_eq!(
            kvs_without_defaults.get_value_as::<f64>(&keyname)?,
            non_default_value,
            "kvs_without_defaults: key '{keyname}' should persist non-default value {non_default_value} after reopen"
        );
    }

    Ok(())
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__value_default", "comp_req__persistency__default_value_config", "comp_req__persistency__default_value_types"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Checks that KVS loads default values when defaults are present and OpenNeedDefaults is set to Optional.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
fn cit_persistency_default_values_optional() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir().unwrap();
    let dir_string = dir.path().to_string_lossy().to_string();

    // Values.
    let keyname = "test_number".to_string();
    let default_value = 111.1;

    // Create defaults file for instance 0.
    let default_id = InstanceId(0);
    write_defaults_file(
        dir.path(),
        HashMap::from([(keyname.clone(), JsonValue::from(default_value))]),
        default_id,
    )
    .unwrap();

    // Assertions.
    {
        // KVS instance with present defaults file and optional defaults setting
        // (should load defaults).
        let kvs_optional_defaults = Kvs::open(
            default_id,
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;

        // Check defaults.
        assert!(
            kvs_optional_defaults.is_value_default(&keyname)?,
            "kvs_optional_defaults: key '{keyname}' should be default"
        );
        assert_eq!(
            kvs_optional_defaults.get_value_as::<f64>(&keyname)?,
            default_value,
            "kvs_optional_defaults: key '{keyname}' should have default value {default_value}"
        );
    }

    Ok(())
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__value_default", "comp_req__persistency__default_value_config", "comp_req__persistency__default_value_types"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Tests removal of values in KVS with defaults enabled, ensuring keys revert to their default values.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
fn cit_persistency_defaults_enabled_values_removal() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Values.
    let keyname = "test_number".to_string();
    let default_value = 111.1;
    let non_default_value = 333.3;

    // Create defaults file for instance 0.
    let default_id = InstanceId(0);
    write_defaults_file(
        dir.path(),
        HashMap::from([(keyname.clone(), JsonValue::from(default_value))]),
        default_id,
    )?;

    // Assertions.
    {
        // KVS instance with defaults.
        let kvs_with_defaults = Kvs::open(
            default_id,
            OpenNeedDefaults::Required,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;
        // Check default value.
        assert_eq!(
            kvs_with_defaults.get_value_as::<f64>(&keyname)?,
            default_value,
            "kvs_with_defaults: key '{keyname}' should have default value {default_value}"
        );

        // Set non-default value and check it.
        kvs_with_defaults.set_value(&keyname, non_default_value)?;
        assert_eq!(
            kvs_with_defaults.get_value_as::<f64>(&keyname)?,
            non_default_value,
            "kvs_with_defaults: key '{keyname}' should have non-default value {non_default_value} after set"
        );

        // Remove key and check that the value is back to default.
        kvs_with_defaults.remove_key(&keyname)?;
        assert_eq!(
            kvs_with_defaults.get_value_as::<f64>(&keyname)?,
            default_value,
            "kvs_with_defaults: key '{keyname}' should revert to default value {default_value} after remove"
        );
        assert!(
            kvs_with_defaults.is_value_default(&keyname)?,
            "kvs_with_defaults: key '{keyname}' should be default after remove"
        );
    }

    Ok(())
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__value_default", "comp_req__persistency__default_value_types"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Tests removal of values in KVS without defaults, ensuring removed keys are not found and return KeyNotFound.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
fn cit_persistency_defaults_disabled_values_removal() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Values.
    let keyname = "test_number".to_string();
    let non_default_value = 333.3;

    // Assertions.
    {
        // KVS instance with defaults.
        let kvs_without_defaults = Kvs::open(
            InstanceId(0),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;
        // Set non-default value and check it.
        kvs_without_defaults.set_value(&keyname, non_default_value)?;
        assert_eq!(
            kvs_without_defaults.get_value_as::<f64>(&keyname)?,
            non_default_value,
            "kvs_without_defaults: key '{keyname}' should have non-default value {non_default_value} after set"
        );

        // Remove key and check that KeyNotFound is raised.
        kvs_without_defaults.remove_key(&keyname)?;
        assert_eq!(
            kvs_without_defaults.is_value_default(&keyname).unwrap_err(),
            ErrorCode::KeyNotFound,
            "kvs_without_defaults: key '{keyname}' should not exist and return KeyNotFound"
        );
    }

    Ok(())
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__value_default", "comp_req__persistency__default_value_config", "comp_req__persistency__default_value_types"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Verifies that KVS fails to open when the defaults file contains invalid JSON.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
fn cit_persistency_invalid_default_values() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Write invalid JSON directly
    let keyname = "test_bool";
    let default_id = InstanceId(0);
    let filename = dir.path().join(format!("kvs_{default_id}_default.json"));
    let invalid_json = format!(r#"{{"{keyname}": True}}"#);
    std::fs::write(&filename, invalid_json)?;

    // Assertions: opening should fail due to invalid JSON
    let kvs = Kvs::open(
        default_id,
        OpenNeedDefaults::Required,
        OpenNeedKvs::Optional,
        Some(dir_string.clone()),
    );
    assert!(
        kvs.is_err(),
        "Kvs::open should fail with invalid JSON in defaults file"
    );

    Ok(())
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__value_default", "comp_req__persistency__default_value_config", "comp_req__persistency__default_value_types"])]
// #[record_property("FullyVerifies", ["comp_req__persistency__value_reset"])]
// #[record_property("Description", "Checks that resetting KVS restores all keys to their default values.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
fn cit_persistency_reset_all_default_values() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Values.
    let keyname1 = "test_number1".to_string();
    let keyname2 = "test_number2".to_string();
    let default_value: f64 = 111.1;
    let non_default_value = 333.3;

    // Create defaults file for instance 0.
    let default_id = InstanceId(0);
    write_defaults_file(
        dir.path(),
        HashMap::from([
            (keyname1.clone(), JsonValue::from(default_value)),
            (keyname2.clone(), JsonValue::from(default_value)),
        ]),
        default_id,
    )?;

    // Assertions.
    {
        // KVS instance with defaults.
        let kvs_with_defaults = Kvs::open(
            default_id,
            OpenNeedDefaults::Required,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;

        // Check defaults.
        assert!(
            kvs_with_defaults.is_value_default(&keyname1)?,
            "kvs_with_defaults: key '{keyname1}' should be default"
        );
        assert!(
            kvs_with_defaults.is_value_default(&keyname2)?,
            "kvs_with_defaults: key '{keyname2}' should be default"
        );

        // Set non-default value
        kvs_with_defaults.set_value(&keyname1, non_default_value)?;
        kvs_with_defaults.set_value(&keyname2, non_default_value)?;
        // Check that the value is non-default.
        assert!(
            !kvs_with_defaults.is_value_default(&keyname1)?,
            "kvs_with_defaults: key '{keyname1}' should NOT be default after set"
        );
        assert!(
            !kvs_with_defaults.is_value_default(&keyname2)?,
            "kvs_with_defaults: key '{keyname2}' should NOT be default after set"
        );

        // Reset the KVS instance - all keys should revert to default values.
        kvs_with_defaults.reset()?;
        // Check that the value is default again.
        assert!(
            kvs_with_defaults.is_value_default(&keyname1)?,
            "kvs_with_defaults: key '{keyname1}' should be default"
        );
        assert!(
            kvs_with_defaults.is_value_default(&keyname2)?,
            "kvs_with_defaults: key '{keyname2}' should be default"
        );
    }

    Ok(())
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__value_default", "comp_req__persistency__default_value_config"])]
// #[record_property("FullyVerifies", ["comp_req__persistency__default_value_checksum"])]
// #[record_property("Description", "Ensures that a checksum file is created when opening KVS with defaults.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
fn cit_persistency_defaults_checksum() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Values.
    let keyname = "test_number".to_string();
    let default_value = 111.1;
    // Create defaults file for instance 0.
    let default_id = InstanceId(0);
    write_defaults_file(
        dir.path(),
        HashMap::from([(keyname.clone(), JsonValue::from(default_value))]),
        default_id,
    )?;

    let hash_file = dir.path().join(format!("kvs_{default_id}_0.hash"));
    assert!(
        !hash_file.exists(),
        "Hash file should NOT exist before opening KVS with defaults"
    );

    {
        // KVS instance with defaults.
        let kvs_with_defaults = Kvs::open(
            default_id.clone(),
            OpenNeedDefaults::Required,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;
    }

    assert!(
        hash_file.exists(),
        "Hash file should exist after opening KVS with defaults"
    );

    Ok(())
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__value_default", "comp_req__persistency__default_value_config"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Placeholder for future test: should verify resetting a single key to its default value.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
#[ignore = "not implemented yet"]
fn cit_persistency_reset_single_default_value() -> Result<(), ErrorCode> {
    // TODO: This test is not implemented yet.
    // API supports resetting only all keys.
    Ok(())
}
