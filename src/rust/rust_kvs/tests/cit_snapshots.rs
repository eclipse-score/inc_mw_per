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

use rust_kvs::prelude::*;
use std::cmp::min;
use std::path::PathBuf;
use tempfile::tempdir;

/// Initialize KVS object with set number of snapshots.
fn init_kvs(
    instance_id: InstanceId,
    dir_string: String,
    num_snapshots: usize,
) -> Result<Kvs, ErrorCode> {
    let kvs = Kvs::open(
        instance_id,
        OpenNeedDefaults::Optional,
        OpenNeedKvs::Optional,
        Some(dir_string),
    )?;

    // Add snapshots.
    for i in 1..=num_snapshots {
        kvs.set_value("counter", i as f64)?;
        kvs.flush()?;

        assert_eq!(kvs.snapshot_count(), min(i, Kvs::snapshot_max_count()));
    }
    assert!(kvs.snapshot_count() <= Kvs::snapshot_max_count());

    Ok(kvs)
}
#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__snapshot_creation"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Verifies that a snapshot is only created after the first flush, and not before.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
fn cit_snapshots_snapshot_count_first_flush() -> Result<(), ErrorCode> {
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    let kvs = Kvs::open(
        InstanceId(0),
        OpenNeedDefaults::Optional,
        OpenNeedKvs::Optional,
        Some(dir_string),
    )?;
    kvs.set_value("counter", 1.0)?;

    // Not flushed yet - no snapshots.
    assert_eq!(kvs.snapshot_count(), 0);

    // Flush.
    kvs.flush()?;

    // Flushed once - one snapshot.
    assert_eq!(kvs.snapshot_count(), 1);

    Ok(())
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__snapshot_creation"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Checks that the snapshot count increases with each flush, up to the maximum allowed count.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
fn cit_snapshots_snapshot_count_full() -> Result<(), ErrorCode> {
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Create snapshots - one more than max count.
    for counter in 0..=Kvs::snapshot_max_count() {
        let kvs = Kvs::open(
            InstanceId(0),
            OpenNeedDefaults::Optional,
            if counter == 0 {
                OpenNeedKvs::Optional
            } else {
                OpenNeedKvs::Required
            },
            Some(dir_string.clone()),
        )?;
        kvs.set_value("counter", counter as f64)?;

        assert_eq!(kvs.snapshot_count(), counter);
    }

    // Check if at max.
    {
        let kvs = Kvs::open(
            InstanceId(0),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Required,
            Some(dir_string),
        )?;
        assert_eq!(kvs.snapshot_count(), Kvs::snapshot_max_count());
    }

    Ok(())
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__snapshot_max_num"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Verifies that the maximum number of snapshots is a constant value.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "inspection")]
fn cit_snapshots_snapshot_max_count() -> Result<(), ErrorCode> {
    // Value is constant.
    assert_eq!(Kvs::snapshot_max_count(), 3);
    Ok(())
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__snapshot_creation", "comp_req__persistency__snapshot_rotate"])]
// #[record_property("FullyVerifies", ["comp_req__persistency__snapshot_restore"])]
// #[record_property("Description", "Verifies restoring to a previous snapshot returns the expected value.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "control-flow-analysis")]
fn cit_snapshots_snapshot_restore_previous_snapshot() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Arrange.
    let instance_id = InstanceId(0);
    let num_snapshots = 4;
    let kvs = init_kvs(instance_id, dir_string, num_snapshots)?;

    // Assert.
    kvs.snapshot_restore(SnapshotId(3))?;
    assert_eq!(kvs.get_value_as::<f64>("counter")?, 1.0);
    Ok(())
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__snapshot_creation"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Checks that restoring the current snapshot ID fails with InvalidSnapshotId error.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "fault-injection")]
fn cit_snapshots_snapshot_restore_current_snapshot() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Arrange.
    let instance_id = InstanceId(0);
    let num_snapshots = 2;
    let kvs = init_kvs(instance_id, dir_string.clone(), num_snapshots)?;

    // Assert.
    let result = kvs.snapshot_restore(SnapshotId(0));
    assert!(result.is_err_and(|e| e == ErrorCode::InvalidSnapshotId));
    Ok(())
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__snapshot_creation", "comp_req__persistency__snapshot_restore"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Checks that restoring a non-existing snapshot fails with InvalidSnapshotId error.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "fault-injection")]
fn cit_snapshots_snapshot_restore_nonexisting_snapshot() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Arrange.
    let instance_id = InstanceId(0);
    let num_snapshots = 2;
    let kvs = init_kvs(instance_id, dir_string.clone(), num_snapshots)?;

    // Assert.
    let result = kvs.snapshot_restore(SnapshotId(3));
    assert!(result.is_err_and(|e| e == ErrorCode::InvalidSnapshotId));
    Ok(())
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__snapshot_creation"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Verifies that the filename for an existing snapshot is generated correctly.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "interface-test")]
fn cit_snapshots_get_kvs_filename_existing_snapshot() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Arrange.
    let instance_id = InstanceId(0);
    let num_snapshots = 2;
    let kvs = init_kvs(instance_id, dir_string.clone(), num_snapshots)?;

    // Assert.
    let last_snapshot_index = num_snapshots - 1;
    let expected = PathBuf::from(dir_string).join(format!(
        "kvs_{}_{}.json",
        instance_id.clone(),
        last_snapshot_index
    ));
    let actual = kvs.get_kvs_filename(SnapshotId(last_snapshot_index))?;
    assert_eq!(expected, actual);
    Ok(())
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__snapshot_creation"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Checks that requesting the filename for a non-existing snapshot returns FileNotFound error.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "fault-injection")]
fn cit_snapshots_get_kvs_filename_nonexisting_snapshot() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Arrange.
    let instance_id = InstanceId(0);
    let num_snapshots = 2;
    let kvs = init_kvs(instance_id, dir_string.clone(), num_snapshots)?;

    // Assert.
    let invalid_snapshot_index = num_snapshots;
    let result = kvs.get_kvs_filename(SnapshotId(invalid_snapshot_index));
    assert!(result.is_err_and(|e| e == ErrorCode::FileNotFound));
    Ok(())
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__snapshot_creation"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Verifies that the hash filename for an existing snapshot is generated correctly.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "interface-test")]
fn cit_snapshots_get_hash_filename_existing_snapshot() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Arrange.
    let instance_id = InstanceId(0);
    let num_snapshots = 2;
    let kvs = init_kvs(instance_id, dir_string.clone(), num_snapshots)?;

    // Assert.
    let last_snapshot_index = num_snapshots - 1;
    let expected = PathBuf::from(dir_string).join(format!(
        "kvs_{}_{}.hash",
        instance_id.clone(),
        last_snapshot_index
    ));
    let actual = kvs.get_hash_filename(SnapshotId(last_snapshot_index))?;
    assert_eq!(expected, actual);
    Ok(())
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__snapshot_creation"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Checks that requesting the hash filename for a non-existing snapshot returns FileNotFound error.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "fault-injection")]
fn cit_snapshots_get_hash_filename_nonexisting_snapshot() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Arrange.
    let instance_id = InstanceId(0);
    let num_snapshots = 2;
    let kvs = init_kvs(instance_id, dir_string.clone(), num_snapshots)?;

    // Assert.
    let invalid_snapshot_index = num_snapshots;
    let result = kvs.get_hash_filename(SnapshotId(invalid_snapshot_index));
    assert!(result.is_err_and(|e| e == ErrorCode::FileNotFound));
    Ok(())
}
