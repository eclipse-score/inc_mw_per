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

//! # Verify Snapshot Recovery

use rust_kvs::{ErrorCode, InstanceId, Kvs, KvsApi, KvsBuilder, SnapshotId};
use std::env::set_current_dir;
use tempfile::tempdir;

/// Test snapshot recovery
#[test]
fn kvs_snapshot_restore() -> Result<(), ErrorCode> {
    let dir = tempdir()?;
    set_current_dir(dir.path())?;

    let max_count = Kvs::snapshot_max_count();
    let mut kvs = KvsBuilder::<Kvs>::new(InstanceId::new(0))
        .need_defaults(false)
        .need_kvs(false)
        .build()?;

    // we need a double zero here because after the first flush no snapshot is created
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

        // drop the current instance with flush-on-exit enabled and re-open it
        drop(kvs);
        kvs = KvsBuilder::new(InstanceId::new(0))
            .need_defaults(false)
            .need_kvs(true)
            .build()?;
    }

    // restore snapshots and check `counter` value
    for idx in 1..=max_count {
        kvs.snapshot_restore(SnapshotId::new(idx))?;
        assert_eq!(kvs.get_value::<f64>("counter")?, (counter - idx) as f64);
    }

    Ok(())
}
