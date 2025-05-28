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

//! # Verify KVS Base Functionality without Defaults

use rust_kvs::{ErrorCode, InstanceId, KvsBuilder, KvsValue};
use std::collections::HashMap;

mod common;
use crate::common::TempDir;

/// Create a key-value-storage without defaults
#[test]
fn kvs_new_wo_defaults() -> Result<(), ErrorCode> {
    let dir = TempDir::create()?;
    dir.set_current_dir()?;

    let kvs = KvsBuilder::new(InstanceId::new(0))
        .need_defaults(false)
        .need_kvs(false)
        .build()?;

    kvs.set_value("number", 123.0)?;
    kvs.set_value("bool", true)?;
    kvs.set_value("string", "Hello".to_string())?;
    kvs.set_value("null", ())?;
    kvs.set_value(
        "array",
        vec![
            KvsValue::from(456.0),
            false.into(),
            "Bye".to_string().into(),
        ],
    )?;
    kvs.set_value(
        "object",
        HashMap::from([
            (String::from("sub-number"), KvsValue::from(789.0)),
            ("sub-bool".into(), true.into()),
            ("sub-string".into(), "Hi".to_string().into()),
            ("sub-null".into(), ().into()),
            (
                "sub-array".into(),
                KvsValue::from(vec![
                    KvsValue::from(1246.0),
                    false.into(),
                    "Moin".to_string().into(),
                ]),
            ),
        ]),
    )?;

    // drop the current instance with flush-on-exit enabled and reopen storage
    drop(kvs);

    let kvs = KvsBuilder::new(InstanceId::new(0))
        .need_defaults(false)
        .need_kvs(true)
        .build()?;

    assert_eq!(kvs.get_value("number")?, KvsValue::Number(123.0));
    assert_eq!(kvs.get_value("bool")?, KvsValue::Boolean(true));
    assert_eq!(kvs.get_value("string")?, KvsValue::String("Hello".into()));
    assert_eq!(kvs.get_value("null")?, KvsValue::Null);

    let json_array = kvs.get_value("array")?;
    assert_eq!(json_array[0].get(), Some(&456.0));
    assert_eq!(json_array[1].get(), Some(&false));
    assert_eq!(json_array[2].get(), Some(&"Bye".to_string()));

    let json_map = kvs.get_value("object")?;
    let json_map = json_map.get::<HashMap<String, KvsValue>>().unwrap();
    assert_eq!(json_map["sub-number"].get(), Some(&789.0));
    assert_eq!(json_map["sub-bool"].get(), Some(&true));
    assert_eq!(json_map["sub-string"].get(), Some(&"Hi".to_string()));
    assert_eq!(json_map["sub-null"].get(), Some(&()));

    let json_sub_array = &json_map["sub-array"].get::<Vec<KvsValue>>().unwrap();
    assert_eq!(json_sub_array[0].get(), Some(&1246.0));
    assert_eq!(json_sub_array[1].get(), Some(&false));
    assert_eq!(json_sub_array[2].get(), Some(&"Moin".to_string()));

    // test for non-existent values
    assert_eq!(
        kvs.get_value("non-existent").err(),
        Some(ErrorCode::KeyNotFound)
    );

    Ok(())
}
