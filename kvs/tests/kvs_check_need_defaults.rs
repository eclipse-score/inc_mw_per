//! # Verify File Check for non-existing Defaults File
//!
//! ## License
//!
//! Copyright (c) 2025 Qorix GmbH
//!
//! This program and the accompanying materials are made available under the terms of the Apache
//! License, Version 2.0 which is available at https://www.apache.org/licenses/LICENSE-2.0.
//!
//! SPDX-License-Identifier: Apache-2.0

use rust_kvs::{RtErrorCodes, RtInstanceId, RtKvs, RtSacId};

mod common;
use crate::common::TempDir;

/// Start with no KVS and check if the `need_defaults` flag is working
#[test]
fn kvs_check_needs_kvs() -> Result<(), RtErrorCodes> {
    let dir = TempDir::create()?;
    dir.set_current_dir()?;

    let kvs = RtKvs::open(RtInstanceId::new(0), RtSacId::new(0), true, false);

    assert_eq!(kvs.err(), Some(RtErrorCodes::FileNotFound));

    Ok(())
}
