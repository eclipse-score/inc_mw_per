//! # Verify KVS Open with missing Checksum
//!
//! ## License
//!
//! Copyright (c) 2025 Qorix GmbH
//!
//! This program and the accompanying materials are made available under the terms of the Apache
//! License, Version 2.0 which is available at https://www.apache.org/licenses/LICENSE-2.0.
//!
//! SPDX-License-Identifier: Apache-2.0

use rust_kvs::{RtErrorCodes, RtInstanceId, RtKvs, RtSacId, RtSnapshotId};
use tinyjson::JsonValue;

mod common;
use crate::common::TempDir;

/// Create a KVS, close it, delete checksum and try to reopen it.
#[test]
fn kvs_checksum_missing() -> Result<(), RtErrorCodes> {
    let dir = TempDir::create()?;
    dir.set_current_dir()?;

    let kvs = RtKvs::open(RtInstanceId::new(0), RtSacId::new(0), false, false)?;

    kvs.set_value("number", 123.0)?;
    kvs.set_value("bool", true)?;
    kvs.set_value("string", "Hello".to_string())?;
    kvs.set_value("null", ())?;
    kvs.set_value(
        "array",
        vec![
            JsonValue::from(456.0),
            false.into(),
            "Bye".to_string().into(),
        ],
    )?;

    // remember hash filename
    let hash_filename = kvs.get_hash_filename(RtSnapshotId::new(0));

    // drop the current instance with sync-on-exit enabled and reopen storage
    drop(kvs);

    // delete the checksum
    std::fs::remove_file(hash_filename)?;

    // opening must fail because of the missing checksum file
    let kvs = RtKvs::open(RtInstanceId::new(0), RtSacId::new(0), false, true);
    assert_eq!(kvs.err(), Some(RtErrorCodes::FileNotFound));

    Ok(())
}
