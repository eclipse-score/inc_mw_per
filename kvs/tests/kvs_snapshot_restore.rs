//! # Verify Snapshot Recovery
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

mod common;
use crate::common::TempDir;

/// Test snapshot recovery
#[test]
fn kvs_snapshot_restore() -> Result<(), RtErrorCodes> {
    let dir = TempDir::create()?;
    dir.set_current_dir()?;

    let max_count = RtKvs::snapshot_max_count()?;
    let mut kvs = RtKvs::open(RtInstanceId::new(0), RtSacId::new(0), false, false)?;

    // we need a double zero here because after the first sync no snapshot is created
    // and the max count is also added twice to make sure we rotate once
    let mut cnts: Vec<usize> = vec![0];
    let mut cnts_mid: Vec<usize> = (0..=max_count).collect();
    let mut cnts_post: Vec<usize> = vec![max_count];
    cnts.append(&mut cnts_mid);
    cnts.append(&mut cnts_post);

    let mut counter = 0;
    for (idx, _) in cnts.into_iter().enumerate() {
        counter = idx;
        kvs.set_value("counter", counter as f64)?;

        // drop the current instance with sync-on-exit enabled and re-open it
        drop(kvs);
        kvs = RtKvs::open(RtInstanceId::new(0), RtSacId::new(0), false, true)?;
    }

    // restore snapshots and check `counter` value
    for idx in 1..=max_count {
        kvs.snapshot_restore(RtSnapshotId::new(idx))?;
        assert_eq!(kvs.get_value::<f64>("counter")?, (counter - idx) as f64);
    }

    Ok(())
}
