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

use rust_kvs::{ErrorCode, InstanceId, Kvs, KvsBuilder, KvsValue, OpenNeedDefaults, OpenNeedKvs};
use std::collections::HashMap;

mod common;
use crate::common::TempDir;

/// Create a key-value-storage without defaults via builder
#[test]
fn kvs_without_defaults_builder() -> Result<(), ErrorCode> {
    let dir = TempDir::create()?;
    dir.set_current_dir()?;

    let kvs = Kvs::open(
        InstanceId::new(0),
        OpenNeedDefaults::Optional,
        OpenNeedKvs::Optional,
    )?;

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

    let builder = KvsBuilder::new(InstanceId::new(0));
    let builder = builder.need_defaults(false);
    let builder = builder.need_kvs(true);
    let kvs = builder.build()?;

    assert_eq!(kvs.get_value::<f64>("number")?, 123.0);
    assert!(kvs.get_value::<bool>("bool")?);
    assert_eq!(kvs.get_value::<String>("string")?, "Hello");
    assert_eq!(kvs.get_value::<()>("null"), Ok(()));

    let json_array = kvs.get_value::<Vec<KvsValue>>("array")?;
    assert_eq!(json_array[0].get(), Some(&456.0));
    assert_eq!(json_array[1].get(), Some(&false));
    assert_eq!(json_array[2].get(), Some(&"Bye".to_string()));

    let json_map = kvs.get_value::<HashMap<String, KvsValue>>("object")?;
    assert_eq!(json_map["sub-number"].get(), Some(&789.0));
    assert_eq!(json_map["sub-bool"].get(), Some(&true));
    assert_eq!(json_map["sub-string"].get(), Some(&"Hi".to_string()));
    assert_eq!(json_map["sub-null"].get(), Some(&()));

    let json_sub_array = &json_map["sub-array"];
    assert_eq!(json_sub_array[0].get(), Some(&1246.0));
    assert_eq!(json_sub_array[1].get(), Some(&false));
    assert_eq!(json_sub_array[2].get(), Some(&"Moin".to_string()));

    // test for non-existent values
    assert_eq!(
        kvs.get_value::<String>("non-existent").err(),
        Some(ErrorCode::KeyNotFound)
    );

    Ok(())
}
