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

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__key_encoding", "comp_req__persistency__value_data_types"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Verifies that KVS supports UTF-8 string keys for storing and retrieving values.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "interface-test")]
fn cit_supported_datatypes_keys() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    let kvs: Kvs = KvsBuilder::new(InstanceId(0)).dir(dir_string).build()?;
    // `str` and `String` are guaranteed to be valid UTF-8.
    kvs.set_value("k1", ())?;
    kvs.set_value(String::from("k2"), ())?;

    assert!(kvs.get_value_as::<()>("k1").is_ok());
    assert!(kvs.get_value_as::<()>("k2").is_ok());

    Ok(())
}

fn supported_datatypes_common_impl(data: HashMap<&str, KvsValue>) -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Initialize KVS and load data.
    let kvs: Kvs = KvsBuilder::new(InstanceId(0)).dir(dir_string).build()?;
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
// #[record_property("PartiallyVerifies", ["comp_req__persistency__key_naming", "comp_req__persistency__key_encoding", "comp_req__persistency__value_data_types"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Checks that KVS can store and retrieve numeric values correctly.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
fn cit_supported_datatypes_number() -> Result<(), ErrorCode> {
    let data = HashMap::from([
        ("number1", KvsValue::from(123.0)),
        ("number2", KvsValue::from(-345.6)),
    ]);
    supported_datatypes_common_impl(data)
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__key_naming", "comp_req__persistency__key_encoding", "comp_req__persistency__value_data_types"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Checks that KVS can store and retrieve boolean values correctly.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
fn cit_supported_datatypes_boolean() -> Result<(), ErrorCode> {
    let data = HashMap::from([("bool", KvsValue::from(true))]);
    supported_datatypes_common_impl(data)
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__key_naming", "comp_req__persistency__key_encoding", "comp_req__persistency__value_data_types"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Checks that KVS can store and retrieve string values correctly.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
fn cit_supported_datatypes_string() -> Result<(), ErrorCode> {
    let data = HashMap::from([("string", KvsValue::from("example".to_string()))]);
    supported_datatypes_common_impl(data)
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__key_naming", "comp_req__persistency__key_encoding", "comp_req__persistency__value_data_types"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Checks that KVS can store and retrieve null/unit values correctly.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
fn cit_supported_datatypes_null() -> Result<(), ErrorCode> {
    let data = HashMap::from([("null", KvsValue::from(()))]);
    supported_datatypes_common_impl(data)
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__key_naming", "comp_req__persistency__key_encoding", "comp_req__persistency__value_data_types"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Checks that KVS can store and retrieve array values correctly.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
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
// #[record_property("PartiallyVerifies", ["comp_req__persistency__key_naming", "comp_req__persistency__key_encoding", "comp_req__persistency__value_data_types"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Checks that KVS can store and retrieve object/map values correctly.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
fn cit_supported_datatypes_object() -> Result<(), ErrorCode> {
    let hashmap = HashMap::from([("sub-number".to_string(), KvsValue::from(789.0))]);
    let data = HashMap::from([("object", KvsValue::from(hashmap))]);
    supported_datatypes_common_impl(data)
}
