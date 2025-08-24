//! Snapshots tests.
//!
//! Requirements verified:
//! - Snapshots (feat_req__persistency__snapshots)
//!   The KVS system shall support explicit creation of snapshots identified by unique IDs and allow rollback to previous snapshots.
//!   Snapshots shall also be deletable.

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
    let kvs = KvsBuilder::new(instance_id).dir(dir_string).build()?;

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
fn cit_snapshots_snapshot_count_first_flush() -> Result<(), ErrorCode> {
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    let kvs = KvsBuilder::new(InstanceId(0)).dir(dir_string).build()?;
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
fn cit_snapshots_snapshot_count_full() -> Result<(), ErrorCode> {
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Create snapshots - one more than max count.
    for counter in 0..=Kvs::snapshot_max_count() {
        let kvs = KvsBuilder::new(InstanceId(0))
            .dir(dir_string.clone())
            .kvs_load(if counter == 0 {
                KvsLoad::Optional
            } else {
                KvsLoad::Required
            })
            .build()?;
        kvs.set_value("counter", counter as f64)?;

        assert_eq!(kvs.snapshot_count(), counter);
    }

    // Check if at max.
    {
        let kvs = KvsBuilder::new(InstanceId(0))
            .dir(dir_string.clone())
            .kvs_load(KvsLoad::Required)
            .build()?;
        assert_eq!(kvs.snapshot_count(), Kvs::snapshot_max_count());
    }

    Ok(())
}

#[test]
fn cit_snapshots_snapshot_max_count() -> Result<(), ErrorCode> {
    // Value is constant.
    assert_eq!(Kvs::snapshot_max_count(), 3);
    Ok(())
}

#[test]
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
fn cit_snapshots_get_kvs_file_path_existing_snapshot() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Arrange.
    let instance_id = InstanceId(0);
    let num_snapshots = 2;
    let kvs = init_kvs(instance_id, dir_string.clone(), num_snapshots)?;

    // Assert.
    let last_snapshot_index = num_snapshots - 1;
    let expected =
        PathBuf::from(dir_string).join(format!("kvs_{instance_id}_{last_snapshot_index}.json"));
    let actual = kvs.get_kvs_file_path(SnapshotId(last_snapshot_index))?;
    assert_eq!(expected, actual);
    Ok(())
}

#[test]
fn cit_snapshots_get_kvs_file_path_nonexisting_snapshot() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Arrange.
    let instance_id = InstanceId(0);
    let num_snapshots = 2;
    let kvs = init_kvs(instance_id, dir_string.clone(), num_snapshots)?;

    // Assert.
    let invalid_snapshot_index = num_snapshots;
    let result = kvs.get_kvs_file_path(SnapshotId(invalid_snapshot_index));
    assert!(result.is_err_and(|e| e == ErrorCode::FileNotFound));
    Ok(())
}

#[test]
fn cit_snapshots_get_hash_file_path_existing_snapshot() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Arrange.
    let instance_id = InstanceId(0);
    let num_snapshots = 2;
    let kvs = init_kvs(instance_id, dir_string.clone(), num_snapshots)?;

    // Assert.
    let last_snapshot_index = num_snapshots - 1;
    let expected =
        PathBuf::from(dir_string).join(format!("kvs_{instance_id}_{last_snapshot_index}.hash"));
    let actual = kvs.get_hash_file_path(SnapshotId(last_snapshot_index))?;
    assert_eq!(expected, actual);
    Ok(())
}

#[test]
fn cit_snapshots_get_hash_file_path_nonexisting_snapshot() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Arrange.
    let instance_id = InstanceId(0);
    let num_snapshots = 2;
    let kvs = init_kvs(instance_id, dir_string.clone(), num_snapshots)?;

    // Assert.
    let invalid_snapshot_index = num_snapshots;
    let result = kvs.get_hash_file_path(SnapshotId(invalid_snapshot_index));
    assert!(result.is_err_and(|e| e == ErrorCode::FileNotFound));
    Ok(())
}
