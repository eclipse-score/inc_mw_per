use crate::helpers::{kvs_instance::kvs_instance, kvs_parameters::KvsParameters};
use rust_kvs::{
    prelude::{KvsApi, SnapshotId},
    Kvs,
};
use serde_json::Value;
use test_scenarios_rust::scenario::{Scenario, ScenarioGroup, ScenarioGroupImpl};
use tracing::info;

struct SnapshotCount;

impl Scenario for SnapshotCount {
    fn name(&self) -> &str {
        "count"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let input_string = input.as_ref().unwrap();
        let v: Value = serde_json::from_str(input_string).unwrap();
        let count = serde_json::from_value(v["count"].clone()).unwrap();
        let params = KvsParameters::from_value(&v).unwrap();

        // Create snapshots.
        for i in 0..count {
            let kvs = kvs_instance(params.clone()).unwrap();
            kvs.set_value("counter", i).unwrap();
            info!(snapshot_count = kvs.snapshot_count());
        }

        {
            let kvs = kvs_instance(params).unwrap();
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

    fn run(&self, _input: Option<String>) -> Result<(), String> {
        info!(max_count = Kvs::snapshot_max_count());
        Ok(())
    }
}

struct SnapshotRestore;

impl Scenario for SnapshotRestore {
    fn name(&self) -> &str {
        "restore"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let input_string = input.as_ref().unwrap();
        let v: Value = serde_json::from_str(input_string).unwrap();
        let count = serde_json::from_value(v["count"].clone()).unwrap();
        let snapshot_id = serde_json::from_value(v["snapshot_id"].clone()).unwrap();
        let params = KvsParameters::from_value(&v).unwrap();

        // Create snapshots.
        for i in 0..count {
            let kvs = kvs_instance(params.clone()).unwrap();
            kvs.set_value("counter", i).unwrap();
        }

        {
            let kvs = kvs_instance(params).unwrap();

            let result = kvs.snapshot_restore(SnapshotId(snapshot_id));
            info!(result = format!("{result:?}"));
            if let Ok(()) = result {
                info!(value = kvs.get_value_as::<i32>("counter").unwrap());
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
        let input_string = input.as_ref().unwrap();
        let v: Value = serde_json::from_str(input_string).unwrap();
        let count = serde_json::from_value(v["count"].clone()).unwrap();
        let snapshot_id = serde_json::from_value(v["snapshot_id"].clone()).unwrap();
        let params = KvsParameters::from_value(&v).unwrap();

        // Create snapshots.
        for i in 0..count {
            let kvs = kvs_instance(params.clone()).unwrap();
            kvs.set_value("counter", i).unwrap();
        }

        {
            let kvs = kvs_instance(params).unwrap();

            let kvs_path_result = kvs.get_kvs_filename(SnapshotId(snapshot_id));
            let hash_path_result = kvs.get_hash_filename(SnapshotId(snapshot_id));
            info!(
                kvs_path = format!("{kvs_path_result:?}"),
                hash_path = format!("{hash_path_result:?}")
            )
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
