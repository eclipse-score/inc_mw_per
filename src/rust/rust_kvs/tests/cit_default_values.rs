//! Persistency tests.
//!
//! Requirements verified:
//! - Persistency (feat_req__persistency__multiple_kvs)
//! The KVS system shall allow instantiating multiple independent stores per software architecture element.
//! - Persistency (feat_req__persistency__default_values)
//! The KVS system shall support predefined default values for keys.
//! - Persistency (feat_req__persistency__default_value_get)
//! The KVS system shall support retrieving the default value associated with a key.
//! - Persistency (feat_req__persistency__default_value_file)
//! The KVS shall support the configuration of default key values using an external file.
//!
use rust_kvs::{ErrorCode, InstanceId, Kvs, KvsApi, OpenNeedDefaults, OpenNeedKvs};
use std::collections::HashMap;
use tempfile::tempdir;
use tinyjson::{JsonGenerator, JsonValue};

fn write_defaults_file(
    dir_path: &str,
    data: HashMap<String, JsonValue>,
    instance: InstanceId,
) -> Result<(), ErrorCode> {
    let filename = format!("kvs_{}_default.json", instance);

    let json = JsonValue::from(data);
    let mut buf = Vec::new();
    let mut gen = JsonGenerator::new(&mut buf).indent("  ");
    gen.generate(&json)?;

    let data = String::from_utf8(buf)?;
    std::fs::write(format!("{}/{}", dir_path, filename), &data)?;
    Ok(())
}

#[test]
fn cit_persistency_default_values() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_path = dir.path();

    // Values.
    let keyname = "test_number".to_string();
    let default_value = 111.1;
    let non_default_value = 333.3;

    // Create defaults file for instance 0.
    let default_id = InstanceId::new(0);
    let non_default_id = InstanceId::new(1);
    write_defaults_file(
        &dir_path.to_string_lossy().to_string(),
        HashMap::from([(keyname.clone(), JsonValue::from(default_value))]),
        default_id.clone(),
    )?;

    // Assertions.
    {
        // KVS instance with defaults.
        let kvs_with_defaults = Kvs::open(
            default_id.clone(),
            dir_path.to_path_buf(),
            OpenNeedDefaults::Required,
            OpenNeedKvs::Optional,
        )?;
        // KVS instance without defaults.
        let kvs_without_defaults = Kvs::open(
            non_default_id.clone(),
            dir_path.to_path_buf(),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
        )?;
        // Check defaults.
        assert!(
            kvs_with_defaults.is_value_default(&keyname)?,
            "kvs_with_defaults: key '{}' should be default",
            keyname
        );
        assert_eq!(
            kvs_without_defaults.is_value_default(&keyname).unwrap_err(),
            ErrorCode::KeyNotFound,
            "kvs_without_defaults: key '{}' should not exist and return KeyNotFound",
            keyname
        );

        // Check values.
        assert_eq!(
            kvs_with_defaults.get_value::<f64>(&keyname)?,
            default_value,
            "kvs_with_defaults: key '{}' should have default value {}",
            keyname,
            default_value
        );
        assert_eq!(
            kvs_without_defaults.get_value::<f64>(&keyname).unwrap_err(),
            ErrorCode::KeyNotFound,
            "kvs_without_defaults: key '{}' should not exist and return KeyNotFound",
            keyname
        );
        // Set non-default value to both KVS instances.
        kvs_with_defaults.set_value(&keyname, non_default_value)?;
        kvs_without_defaults.set_value(&keyname, non_default_value)?;
        // Check that the value is non-default.
        assert!(
            !kvs_with_defaults.is_value_default(&keyname)?,
            "kvs_with_defaults: key '{}' should NOT be default after set",
            keyname
        );
        assert!(
            !kvs_without_defaults.is_value_default(&keyname)?,
            "kvs_without_defaults: key '{}' should NOT be default after set",
            keyname
        );
    }
    // Flush and reopen KVS instances to ensure persistency.
    {
        // KVS instance with defaults.
        let kvs_with_defaults = Kvs::open(
            default_id.clone(),
            dir_path.to_path_buf(),
            OpenNeedDefaults::Required,
            OpenNeedKvs::Optional,
        )?;
        // KVS instance without defaults.
        let kvs_without_defaults = Kvs::open(
            non_default_id.clone(),
            dir_path.to_path_buf(),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
        )?;
        // Check that the value is still non-default.
        assert_eq!(
            kvs_with_defaults.get_value::<f64>(&keyname)?,
            non_default_value,
            "kvs_with_defaults: key '{}' should persist non-default value {} after reopen",
            keyname,
            non_default_value
        );
        assert_eq!(
            kvs_without_defaults.get_value::<f64>(&keyname)?,
            non_default_value,
            "kvs_without_defaults: key '{}' should persist non-default value {} after reopen",
            keyname,
            non_default_value
        );
    }

    Ok(())
}

#[test]
#[should_panic(expected = "KeyNotFound")]
fn cit_persistency_default_values_unused() -> () {
    // Temp directory.
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    // Values.
    let keyname = "test_number".to_string();
    let default_value = 111.1;

    // Create defaults file for instance 0.
    let default_id = InstanceId::new(0);
    write_defaults_file(
        &dir_path.to_string_lossy().to_string(),
        HashMap::from([(keyname.clone(), JsonValue::from(default_value))]),
        default_id.clone(),
    )
    .unwrap();

    // Assertions.
    {
        // KVS instance with present defaults file but not used.
        let kvs_without_defaults = Kvs::open(
            default_id.clone(),
            dir_path.to_path_buf(),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
        )
        .unwrap();

        // Check defaults.
        let _ = kvs_without_defaults.is_value_default(&keyname);
    }

    ()
}

#[test]
fn cit_persistency_default_values_removal() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_path = dir.path();

    // Values.
    let keyname = "test_number".to_string();
    let default_value = 111.1;
    let non_default_value = 333.3;

    // Create defaults file for instance 0.
    let default_id = InstanceId::new(0);
    write_defaults_file(
        &dir_path.to_string_lossy().to_string(),
        HashMap::from([(keyname.clone(), JsonValue::from(default_value))]),
        default_id.clone(),
    )?;

    // Assertions.
    {
        // KVS instance with defaults.
        let kvs_with_defaults = Kvs::open(
            default_id.clone(),
            dir_path.to_path_buf(),
            OpenNeedDefaults::Required,
            OpenNeedKvs::Optional,
        )?;
        // Check default value.
        assert_eq!(
            kvs_with_defaults.get_value::<f64>(&keyname)?,
            default_value,
            "kvs_with_defaults: key '{}' should have default value {}",
            keyname,
            default_value
        );

        // Set non-default value and check it.
        kvs_with_defaults.set_value(&keyname, non_default_value)?;
        assert_eq!(
            kvs_with_defaults.get_value::<f64>(&keyname)?,
            non_default_value,
            "kvs_with_defaults: key '{}' should have non-default value {} after set",
            keyname,
            non_default_value
        );

        // Remove key and check that the value is back to default.
        kvs_with_defaults.remove_key(&keyname)?;
        assert_eq!(
            kvs_with_defaults.get_value::<f64>(&keyname)?,
            default_value,
            "kvs_with_defaults: key '{}' should revert to default value {} after remove",
            keyname,
            default_value
        );
        assert!(
            kvs_with_defaults.is_value_default(&keyname)?,
            "kvs_with_defaults: key '{}' should be default after remove",
            keyname
        );
    }

    Ok(())
}

#[test]
fn cit_persistency_invalid_default_values() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_path = dir.path();

    // Write invalid JSON directly
    let keyname = "test_bool";
    let default_id = InstanceId::new(0);
    let filename = format!(
        "{}/kvs_{}_default.json",
        dir_path.to_string_lossy().to_string(),
        default_id
    );
    let invalid_json = format!(r#"{{"{}": True}}"#, keyname);
    std::fs::write(&filename, invalid_json)?;

    // Assertions: opening should fail due to invalid JSON
    let kvs = Kvs::open(
        default_id.clone(),
        dir_path.to_path_buf(),
        OpenNeedDefaults::Required,
        OpenNeedKvs::Optional,
    );
    assert!(
        kvs.is_err(),
        "Kvs::open should fail with invalid JSON in defaults file"
    );

    Ok(())
}

#[test]
fn cit_persistency_reset_all_default_values() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_path = dir.path();

    // Values.
    let keyname1 = "test_number1".to_string();
    let keyname2 = "test_number2".to_string();
    let default_value: f64 = 111.1;
    let non_default_value = 333.3;

    // Create defaults file for instance 0.
    let default_id = InstanceId::new(0);
    write_defaults_file(
        &dir_path.to_string_lossy().to_string(),
        HashMap::from([
            (keyname1.clone(), JsonValue::from(default_value)),
            (keyname2.clone(), JsonValue::from(default_value)),
        ]),
        default_id.clone(),
    )?;

    // Assertions.
    {
        // KVS instance with defaults.
        let kvs_with_defaults = Kvs::open(
            default_id.clone(),
            dir_path.to_path_buf(),
            OpenNeedDefaults::Required,
            OpenNeedKvs::Optional,
        )?;

        // Check defaults.
        assert!(
            kvs_with_defaults.is_value_default(&keyname1)?,
            "kvs_with_defaults: key '{}' should be default",
            keyname1
        );
        assert!(
            kvs_with_defaults.is_value_default(&keyname2)?,
            "kvs_with_defaults: key '{}' should be default",
            keyname2
        );

        // Set non-default value
        kvs_with_defaults.set_value(&keyname1, non_default_value)?;
        kvs_with_defaults.set_value(&keyname2, non_default_value)?;
        // Check that the value is non-default.
        assert!(
            !kvs_with_defaults.is_value_default(&keyname1)?,
            "kvs_with_defaults: key '{}' should NOT be default after set",
            keyname1
        );
        assert!(
            !kvs_with_defaults.is_value_default(&keyname2)?,
            "kvs_with_defaults: key '{}' should NOT be default after set",
            keyname2
        );

        // Reset the KVS instance - all keys should revert to default values.
        kvs_with_defaults.reset()?;
        // Check that the value is default again.
        assert!(
            kvs_with_defaults.is_value_default(&keyname1)?,
            "kvs_with_defaults: key '{}' should be default",
            keyname1
        );
        assert!(
            kvs_with_defaults.is_value_default(&keyname2)?,
            "kvs_with_defaults: key '{}' should be default",
            keyname2
        );
    }

    Ok(())
}

#[test]
#[ignore]
fn cit_persistency_reset_single_default_value() -> Result<(), ErrorCode> {
    // TODO: This test is not implemented yet.
    // API supports resettinng only all keys.
    Ok(())
}
