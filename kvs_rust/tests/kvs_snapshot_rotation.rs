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

//! # Verify Snapshot Rotation

use rust_kvs::{ErrorCode, InstanceId, Kvs, KvsValue, OpenNeedDefaults, OpenNeedKvs, SnapshotId};
use std::collections::HashMap;

mod common;
use crate::common::TempDir;

/// Test snapshot rotation
#[test]
fn kvs_snapshot_rotation() -> Result<(), ErrorCode> {
    let dir = TempDir::create()?;
    dir.set_current_dir()?;

    let max_count = Kvs::snapshot_max_count();
    let mut kvs = create_kvs()?;

    // we need a double zero here because after the first flush no snapshot is created
    // and the max count is also added twice to make sure we rotate once
    let mut cnts: Vec<usize> = vec![0];
    let mut cnts_mid: Vec<usize> = (0..=max_count).collect();
    let mut cnts_post: Vec<usize> = vec![max_count];
    cnts.append(&mut cnts_mid);
    cnts.append(&mut cnts_post);

    // make sure the snapshot count is 0, 0, .., max_count, max_count (rotation)
    for cnt in cnts {
        assert_eq!(kvs.snapshot_count(), cnt);

        // drop the current instance with flush-on-exit enabled and re-open it
        drop(kvs);
        kvs = Kvs::open(
            InstanceId::new(0),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Required,
        )?;
    }

    // restore the oldest snapshot
    assert!(kvs.snapshot_restore(SnapshotId::new(max_count)).is_ok());

    // try to restore a snapshot behind the last one
    assert_eq!(
        kvs.snapshot_restore(SnapshotId::new(max_count + 1)).err(),
        Some(ErrorCode::InvalidSnapshotId)
    );

    Ok(())
}

/// Create an example KVS
fn create_kvs() -> Result<Kvs, ErrorCode> {
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

    Ok(kvs)
}
