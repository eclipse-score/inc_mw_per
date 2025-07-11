//! Supported datatypes tests.
//!
//! Requirements verified:
//! - Supported Datatypes (Keys) (feat_req__persistency__support_datatype_keys)
//! The KVS system shall support UTF-8 encoded strings as valid key types.
//! - Supported Datatypes (Values) (feat_req__persistency__support_datatype_value)
//! The KVS system shall support storing both primitive and non-primitive datatypes as values.
//! The supported datatypes shall match those used by the IPC feature.

mod common;
use common::compare_kvs_values;
use rust_kvs::{ErrorCode, InstanceId, Kvs, KvsApi, KvsBuilder, KvsValue};
use std::collections::HashMap;
use tempfile::tempdir;

#[test]
fn cit_supported_datatypes_keys() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_path = dir.path().to_string_lossy().to_string();

    let kvs: Kvs = KvsBuilder::new(InstanceId::new(0)).dir(dir_path).build()?;
    // `str` and `String` are guaranteed to be valid UTF-8.
    kvs.set_value("k1", ())?;
    kvs.set_value(String::from("k2"), ())?;

    assert_eq!(kvs.get_value_as::<()>("k1")?, ());
    assert_eq!(kvs.get_value_as::<()>("k2")?, ());

    Ok(())
}

fn supported_datatypes_common_impl(data: HashMap<&str, KvsValue>) -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_path = dir.path().to_string_lossy().to_string();

    // Initialize KVS and load data.
    let kvs: Kvs = KvsBuilder::new(InstanceId::new(0)).dir(dir_path).build()?;
    for (k, v) in data.iter() {
        kvs.set_value(k.to_string(), v.clone())?;
    }

    // Assert.
    for (key, expected_value) in data.iter() {
        let actual_value = kvs.get_value(key)?;
        assert!(compare_kvs_values(&actual_value, expected_value));
    }

    Ok(())
}

#[test]
fn cit_supported_datatypes_number() -> Result<(), ErrorCode> {
    let data = HashMap::from([
        ("number1", KvsValue::from(123.0)),
        ("number2", KvsValue::from(-345.6)),
    ]);
    supported_datatypes_common_impl(data)
}

#[test]
fn cit_supported_datatypes_boolean() -> Result<(), ErrorCode> {
    let data = HashMap::from([("bool", KvsValue::from(true))]);
    supported_datatypes_common_impl(data)
}

#[test]
fn cit_supported_datatypes_string() -> Result<(), ErrorCode> {
    let data = HashMap::from([("string", KvsValue::from("example".to_string()))]);
    supported_datatypes_common_impl(data)
}

#[test]
fn cit_supported_datatypes_null() -> Result<(), ErrorCode> {
    let data = HashMap::from([("null", KvsValue::from(()))]);
    supported_datatypes_common_impl(data)
}

#[test]
fn cit_supported_datatypes_array() -> Result<(), ErrorCode> {
    let hashmap = HashMap::from([("sub-number".to_string(), KvsValue::from(789.0))]);
    let data = HashMap::from([(
        "array",
        KvsValue::from(vec![
            KvsValue::from(321.0),
            KvsValue::from(false),
            KvsValue::from("dbca".to_string()),
            KvsValue::from(()),
            KvsValue::from(vec![]),
            KvsValue::from(hashmap),
        ]),
    )]);
    supported_datatypes_common_impl(data)
}

#[test]
fn cit_supported_datatypes_object() -> Result<(), ErrorCode> {
    let hashmap = HashMap::from([("sub-number".to_string(), KvsValue::from(789.0))]);
    let data = HashMap::from([("object", KvsValue::from(hashmap))]);
    supported_datatypes_common_impl(data)
}
