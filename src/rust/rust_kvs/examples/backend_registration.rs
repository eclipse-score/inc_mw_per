//! Example for custom backend registration.
//! - Implementation of `KvsBackend` traits.
//! - Registration of custom backend.
//! - Creation of KVS instance utilizing custom backend.

use rust_kvs::prelude::*;
use tempfile::tempdir;

/// Mock backend implementation.
/// Only `load_kvs` is implemented.
struct MockBackend;

impl KvsBackend for MockBackend {
    fn load_kvs(
        &self,
        _instance_id: InstanceId,
        _snapshot_id: SnapshotId,
    ) -> Result<KvsMap, ErrorCode> {
        println!("`load_kvs` used");
        Ok(KvsMap::new())
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

/// Mock backend factory implementation.
struct MockBackendFactory;

impl KvsBackendFactory for MockBackendFactory {
    fn create(&self, _parameters: &KvsMap) -> Result<Box<dyn KvsBackend>, ErrorCode> {
        Ok(Box::new(MockBackend))
    }
}

fn main() -> Result<(), ErrorCode> {
    // Temporary directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Register `MockBackendFactory`.
    KvsBackendRegistry::register("mock", || Box::new(MockBackendFactory))?;

    // Build KVS instance with mock backend.
    {
        let instance_id = InstanceId(0);
        let parameters = KvsMap::from([("name".to_string(), KvsValue::String("mock".to_string()))]);
        let builder = KvsBuilder::new(instance_id)
            .backend_parameters(parameters)
            .defaults(KvsDefaults::Ignored);
        let kvs = builder.build()?;

        println!(
            "KVS instance with mock backend - parameters: {:?}",
            kvs.parameters()
        );
    }

    // Build KVS instance with JSON backend - default parameters.
    {
        let instance_id = InstanceId(1);
        let builder = KvsBuilder::new(instance_id).defaults(KvsDefaults::Ignored);
        let kvs = builder.build()?;

        println!(
            "KVS instance with default JSON backend - parameters: {:?}",
            kvs.parameters()
        );
    }

    // Build KVS instance with JSON backend - `working_dir` set.
    {
        let instance_id = InstanceId(2);
        let parameters = KvsMap::from([
            ("name".to_string(), KvsValue::String("json".to_string())),
            ("working_dir".to_string(), KvsValue::String(dir_string)),
        ]);
        let builder = KvsBuilder::new(instance_id)
            .backend_parameters(parameters)
            .defaults(KvsDefaults::Ignored);
        let kvs = builder.build()?;

        println!(
            "KVS instance with JSON backend - parameters: {:?}",
            kvs.parameters()
        );
    }

    Ok(())
}
