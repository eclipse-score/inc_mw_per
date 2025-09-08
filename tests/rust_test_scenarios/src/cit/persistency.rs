use crate::helpers::kvs_instance::kvs_instance;
use crate::helpers::kvs_parameters::KvsParameters;
use crate::helpers::to_str;
use rust_kvs::prelude::*;
use test_scenarios_rust::scenario::{Scenario, ScenarioGroup, ScenarioGroupImpl};
use tracing::info;

struct ExplicitFlush;

impl Scenario for ExplicitFlush {
    fn name(&self) -> &str {
        "explicit_flush"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        // List of keys and corresponding values.
        let num_values = 5;
        let mut key_values = Vec::new();
        for i in 0..num_values {
            let key = format!("test_number_{i}");
            let value = 12.3 * i as f64;
            key_values.push((key, value));
        }

        // Check parameters.
        let input_string = input.as_ref().expect("Test input is expected");
        let params = KvsParameters::from_json(input_string).expect("Failed to parse parameters");
        {
            // First KVS instance object - used for setting and flushing data.
            let kvs = kvs_instance(params.clone()).expect("Failed to create KVS instance");

            // Set values.
            for (key, value) in key_values.iter() {
                kvs.set_value(key, *value).expect("Failed to set value");
            }

            // Explicit flush.
            kvs.flush().expect("Failed to flush");
        }

        {
            // Second KVS instance object - used for flush check.
            let kvs = kvs_instance(params).expect("Failed to create KVS instance");

            let snapshot_id = SnapshotId(0);
            let kvs_path_result = kvs.get_kvs_filename(snapshot_id);
            let hash_path_result = kvs.get_hash_filename(snapshot_id);
            info!(
                kvs_path = format!("{kvs_path_result:?}"),
                hash_path = format!("{hash_path_result:?}")
            );

            // Get values.
            for (key, _) in key_values.iter() {
                let value = kvs.get_value(key);
                info!(key, value = to_str(&value));
            }
        }

        Ok(())
    }
}

pub fn persistency_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "persistency",
        vec![Box::new(ExplicitFlush)],
        vec![],
    ))
}
