use rust_kvs::prelude::{InstanceId, SnapshotId};
use std::path::{Path, PathBuf};

pub mod kvs_instance;
pub mod kvs_parameters;

/// Helper function to convert `Debug`-typed value to `String`.
pub(crate) fn to_str<T: std::fmt::Debug>(value: &T) -> String {
    format!("{value:?}")
}

/// Helper function to get expected KVS and hash file paths.
pub(crate) fn kvs_hash_paths(
    working_dir: &Path,
    instance_id: InstanceId,
    snapshot_id: SnapshotId,
) -> (PathBuf, PathBuf) {
    let kvs_path = working_dir.join(format!("kvs_{instance_id}_{snapshot_id}.json"));
    let hash_path = working_dir.join(format!("kvs_{instance_id}_{snapshot_id}.hash"));

    (kvs_path, hash_path)
}
