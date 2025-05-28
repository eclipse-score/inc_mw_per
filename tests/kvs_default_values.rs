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

//! # Verify KVS Default Value Functionality

use adler32::RollingAdler32;
use rust_kvs::{ErrorCode, InstanceId, KvsBuilder, KvsValue};
use std::collections::HashMap;
use tinyjson::{JsonGenerator, JsonValue};

mod common;
use crate::common::TempDir;

/// Test default values
///   * Default file must exist
///   * Default value must be returned when key isn't set
///   * Key must report that default is used
///   * Key must be returned when it was written and report it
///   * Change in default must be returned when key isn't set
///   * Change in default must be ignored when key was once set
#[test]
fn kvs_default_values() -> Result<(), ErrorCode> {
    let dir = TempDir::create()?;
    dir.set_current_dir()?;

    // create defaults file
    let defaults: HashMap<String, KvsValue> = HashMap::from([
        ("number1".to_string(), KvsValue::from(123.0)),
        ("bool1".to_string(), true.into()),
        ("string1".to_string(), "Hello".to_string().into()),
    ]);

    let json = KvsValue::from(defaults);
    let json = JsonValue::try_from(&json)?;
    let mut buf = Vec::new();
    let mut gen = JsonGenerator::new(&mut buf).indent("  ");
    gen.generate(&json)?;

    let data = String::from_utf8(buf)?;
    let hash = RollingAdler32::from_buffer(data.as_bytes()).hash();
    std::fs::write("kvs_0_default.json", &data)?;
    std::fs::write("kvs_0_default.hash", hash.to_be_bytes())?;

    // create KVS
    let kvs = KvsBuilder::new(InstanceId::new(0))
        .need_defaults(true)
        .need_kvs(false)
        .build()?;

    kvs.set_value("number2", 345.0)?;
    kvs.set_value("bool2", false)?;
    kvs.set_value("string2", "Ola".to_string())?;

    assert_eq!(kvs.get_value("number1")?, KvsValue::Number(123.0));
    assert_eq!(kvs.get_value("number2")?, KvsValue::Number(345.0));

    assert_eq!(kvs.get_value("bool1")?, KvsValue::Boolean(true));
    assert_eq!(kvs.get_value("bool2")?, KvsValue::Boolean(false));

    assert_eq!(kvs.get_value("string1")?, KvsValue::String("Hello".into()));
    assert_eq!(kvs.get_value("string2")?, KvsValue::String("Ola".into()));

    assert!(kvs.has_default_value("number1"));
    assert!(!kvs.has_default_value("number2"));

    assert!(kvs.has_default_value("bool1"));
    assert!(!kvs.has_default_value("bool2"));

    assert!(kvs.has_default_value("string1"));
    assert!(!kvs.has_default_value("string2"));

    // write same-as-default-value into `bool1`
    kvs.set_value("bool1", true)?;

    // write not-same-as-default into `string1`
    kvs.set_value("string1", "Bonjour".to_string())?;

    // drop the current instance with flush-on-exit enabled and reopen storage
    drop(kvs);

    let kvs = KvsBuilder::new(InstanceId::new(0))
        .need_defaults(false)
        .need_kvs(true)
        .build()?;

    assert_eq!(kvs.get_value("bool1")?, KvsValue::Boolean(true));
    assert!(kvs.has_default_value("bool1"));

    assert_eq!(
        kvs.get_value("string1")?,
        KvsValue::String("Bonjour".into())
    );
    assert!(!kvs.has_default_value("string1"));

    // drop the current instance with flush-on-exit enabled and reopen storage
    drop(kvs);

    // change default of `number1` and `bool1`
    let defaults: HashMap<String, KvsValue> = HashMap::from([
        ("number1".to_string(), KvsValue::from(987.0)),
        ("bool1".to_string(), false.into()),
        ("string1".to_string(), "Hello".to_string().into()),
    ]);

    let json = KvsValue::from(defaults);
    let json = JsonValue::try_from(&json)?;
    let mut buf = Vec::new();
    let mut gen = JsonGenerator::new(&mut buf).indent("  ");
    gen.generate(&json)?;

    let data = String::from_utf8(buf)?;
    let hash = RollingAdler32::from_buffer(data.as_bytes()).hash();
    std::fs::write("kvs_0_default.json", &data)?;
    std::fs::write("kvs_0_default.hash", hash.to_be_bytes())?;

    let kvs = KvsBuilder::new(InstanceId::new(0))
        .need_defaults(false)
        .need_kvs(true)
        .build()?;

    assert_eq!(kvs.get_value("number1")?, KvsValue::Number(987.0));
    assert!(kvs.has_default_value("number1"));

    assert_eq!(kvs.get_value("bool1")?, KvsValue::Boolean(true));
    assert!(!kvs.has_default_value("bool1"));

    Ok(())
}
