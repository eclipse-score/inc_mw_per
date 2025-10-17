use rust_kvs::prelude::{JsonBackend, Kvs, SnapshotId};
use std::path::PathBuf;

pub mod kvs_instance;
pub mod kvs_parameters;

/// Helper function to convert `Debug`-typed value to `String`.
pub(crate) fn to_str<T: std::fmt::Debug>(value: &T) -> String {
    format!("{value:?}")
}

/// Helper function to get `JsonBackend` from KVS object.
pub(crate) fn json_backend(kvs: &Kvs) -> &JsonBackend {
    let backend = &kvs.parameters().backend;
    let cast_result = backend.as_any().downcast_ref::<JsonBackend>();
    cast_result.expect("Failed to cast backend to JsonBackend")
}

/// Helper function to get KVS and hash file paths from KVS instance.
pub(crate) fn kvs_hash_paths(kvs: &Kvs, snapshot_id: SnapshotId) -> (PathBuf, PathBuf) {
    let backend = json_backend(kvs);
    let instance_id = kvs.parameters().instance_id;
    let kvs_path = backend.kvs_file_path(instance_id, snapshot_id);
    let hash_path = backend.hash_file_path(instance_id, snapshot_id);

    (kvs_path, hash_path)
}
