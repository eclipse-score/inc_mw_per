//! Example for migrations between backends.

use rust_kvs::prelude::*;

/// Example custom backend.
/// Returns some data on `load_kvs`.
struct FromBackend;

impl KvsBackend for FromBackend {
    fn load_kvs(
        &self,
        _instance_id: InstanceId,
        _snapshot_id: SnapshotId,
    ) -> Result<KvsMap, ErrorCode> {
        Ok(KvsMap::from([(
            "example_key".to_string(),
            KvsValue::String("example_value".to_string()),
        )]))
    }

    fn load_defaults(&self, _instance_id: InstanceId) -> Result<KvsMap, ErrorCode> {
        unimplemented!()
    }

    fn flush(&self, _instance_id: InstanceId, _kvs_map: &KvsMap) -> Result<(), ErrorCode> {
        unimplemented!()
    }

    fn snapshot_count(&self, _instance_id: InstanceId) -> usize {
        unimplemented!()
    }

    fn snapshot_max_count(&self) -> usize {
        unimplemented!()
    }

    fn snapshot_restore(
        &self,
        _instance_id: InstanceId,
        _snapshot_id: SnapshotId,
    ) -> Result<KvsMap, ErrorCode> {
        unimplemented!()
    }
}

/// Example custom backend.
/// Prints provided data to stdout.
struct ToBackend;

impl KvsBackend for ToBackend {
    fn load_kvs(
        &self,
        _instance_id: InstanceId,
        _snapshot_id: SnapshotId,
    ) -> Result<KvsMap, ErrorCode> {
        unimplemented!()
    }

    fn load_defaults(&self, _instance_id: InstanceId) -> Result<KvsMap, ErrorCode> {
        unimplemented!()
    }

    fn flush(&self, _instance_id: InstanceId, kvs_map: &KvsMap) -> Result<(), ErrorCode> {
        println!("{kvs_map:?}");
        Ok(())
    }

    fn snapshot_count(&self, _instance_id: InstanceId) -> usize {
        unimplemented!()
    }

    fn snapshot_max_count(&self) -> usize {
        unimplemented!()
    }

    fn snapshot_restore(
        &self,
        _instance_id: InstanceId,
        _snapshot_id: SnapshotId,
    ) -> Result<KvsMap, ErrorCode> {
        unimplemented!()
    }
}

fn main() -> Result<(), ErrorCode> {
    // Load `KvsMap` from first backend.
    // `instance_id` and `snapshot_id` normally must be provided, but aren't used in this example.
    let from_backend = FromBackend;
    let instance_id = InstanceId(0);
    let snapshot_id = SnapshotId(0);
    let data = from_backend.load_kvs(instance_id, snapshot_id).unwrap();

    // Save `KvsMap` in second backend.
    let to_backend = ToBackend;
    to_backend.flush(instance_id, &data).unwrap();

    Ok(())
}
