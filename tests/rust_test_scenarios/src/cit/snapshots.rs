use crate::helpers::kvs_instance::kvs_instance;
use crate::helpers::kvs_parameters::KvsParameters;
use crate::helpers::{kvs_hash_paths, to_str};
use rust_kvs::prelude::{KvsApi, SnapshotId};
use serde_json::Value;
use test_scenarios_rust::scenario::{Scenario, ScenarioGroup, ScenarioGroupImpl};
use tracing::info;

struct SnapshotCount;

impl Scenario for SnapshotCount {
    fn name(&self) -> &str {
        "count"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let input_string = input.as_ref().expect("Test input is expected");
        let v: Value = serde_json::from_str(input_string).expect("Failed to parse input string");
        let count =
            serde_json::from_value(v["count"].clone()).expect("Failed to parse \"count\" field");
        let params = KvsParameters::from_value(&v).expect("Failed to parse parameters");

        // Create snapshots.
        for i in 0..count {
            let kvs = kvs_instance(params.clone()).expect("Failed to create KVS instance");
            kvs.set_value("counter", i).expect("Failed to set value");
            info!(snapshot_count = kvs.snapshot_count());

            // Flush KVS.
            kvs.flush().expect("Failed to flush");
        }

        {
            let kvs = kvs_instance(params).expect("Failed to create KVS instance");
            info!(snapshot_count = kvs.snapshot_count());
        }

        Ok(())
    }
}

struct SnapshotMaxCount;

impl Scenario for SnapshotMaxCount {
    fn name(&self) -> &str {
        "max_count"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let input_string = input.as_ref().expect("Test input is expected");
        let v: Value = serde_json::from_str(input_string).expect("Failed to parse input string");
        let params = KvsParameters::from_value(&v).expect("Failed to parse parameters");

        let kvs = kvs_instance(params.clone()).expect("Failed to create KVS instance");
        info!(max_count = kvs.snapshot_max_count());
        Ok(())
    }
}

struct SnapshotRestore;

impl Scenario for SnapshotRestore {
    fn name(&self) -> &str {
        "restore"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let input_string = input.as_ref().expect("Test input is expected");
        let v: Value = serde_json::from_str(input_string).expect("Failed to parse input string");
        let count =
            serde_json::from_value(v["count"].clone()).expect("Failed to parse \"count\" field");
        let snapshot_id = serde_json::from_value(v["snapshot_id"].clone())
            .expect("Failed to parse \"snapshot_id\" field");
        let params = KvsParameters::from_value(&v).expect("Failed to parse parameters");

        // Create snapshots.
        for i in 0..count {
            let kvs = kvs_instance(params.clone()).expect("Failed to create KVS instance");
            kvs.set_value("counter", i).expect("Failed to set value");

            // Flush KVS.
            kvs.flush().expect("Failed to flush");
        }

        {
            let kvs = kvs_instance(params).expect("Failed to create KVS instance");

            let result = kvs.snapshot_restore(SnapshotId(snapshot_id));
            info!(result = format!("{result:?}"));
            if let Ok(()) = result {
                let value = kvs
                    .get_value_as::<i32>("counter")
                    .expect("Failed to read value");
                info!(value);
            }
        }

        Ok(())
    }
}

struct SnapshotPaths;

impl Scenario for SnapshotPaths {
    fn name(&self) -> &str {
        "paths"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let input_string = input.as_ref().expect("Test input is expected");
        let v: Value = serde_json::from_str(input_string).expect("Failed to parse input string");
        let count =
            serde_json::from_value(v["count"].clone()).expect("Failed to parse \"count\" field");
        let snapshot_id = serde_json::from_value(v["snapshot_id"].clone())
            .expect("Failed to parse \"snapshot_id\" field");
        let params = KvsParameters::from_value(&v).expect("Failed to parse parameters");

        // Create snapshots.
        for i in 0..count {
            let kvs = kvs_instance(params.clone()).expect("Failed to create KVS instance");
            kvs.set_value("counter", i).expect("Failed to set value");

            // Flush KVS.
            kvs.flush().expect("Failed to flush");
        }

        {
            let kvs = kvs_instance(params).expect("Failed to create KVS instance");
            let (kvs_path, hash_path) = kvs_hash_paths(&kvs, SnapshotId(snapshot_id));
            info!(
                kvs_path = to_str(&kvs_path),
                kvs_path_exists = kvs_path.exists(),
                hash_path = to_str(&hash_path),
                hash_path_exists = hash_path.exists(),
            );
        }

        Ok(())
    }
}

pub fn snapshots_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "snapshots",
        vec![
            Box::new(SnapshotCount),
            Box::new(SnapshotMaxCount),
            Box::new(SnapshotRestore),
            Box::new(SnapshotPaths),
        ],
        vec![],
    ))
}
