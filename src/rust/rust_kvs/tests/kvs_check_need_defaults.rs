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

//! # Verify File Check for non-existing Defaults File

use rust_kvs::{ErrorCode, InstanceId, Kvs, KvsBuilder};
use std::env::set_current_dir;
use tempfile::tempdir;

/// Start with no KVS and check if the `need_defaults` flag is working
#[test]
fn kvs_check_needs_defaults() -> Result<(), ErrorCode> {
    let dir = tempdir()?;
    set_current_dir(dir.path())?;

    let kvs = KvsBuilder::<Kvs>::new(InstanceId::new(0))
        .need_defaults(true)
        .need_kvs(false)
        .build();

    assert_eq!(kvs.err(), Some(ErrorCode::KvsFileReadError));

    Ok(())
}
