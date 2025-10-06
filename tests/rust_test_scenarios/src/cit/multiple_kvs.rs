use crate::helpers::{kvs_instance::kvs_instance, kvs_parameters::KvsParameters};
use rust_kvs::prelude::KvsApi;
use serde_json::Value;
use test_scenarios_rust::scenario::{Scenario, ScenarioGroup, ScenarioGroupImpl};
use tracing::info;

struct MultipleInstanceIds;

impl Scenario for MultipleInstanceIds {
    fn name(&self) -> &str {
        "multiple_instance_ids"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        // Values.
        let keyname = "number".to_string();
        let value1 = 111.1;
        let value2 = 222.2;

        // Parameters.
        let input_string = input.as_ref().expect("Test input is expected");
        let v: Value = serde_json::from_str(input_string).expect("Failed to parse input string");
        let params1 =
            KvsParameters::from_value(&v["kvs_parameters_1"]).expect("Failed to parse parameters");
        let params2 =
            KvsParameters::from_value(&v["kvs_parameters_2"]).expect("Failed to parse parameters");
        {
            // Create first KVS instance.
            let kvs1 = kvs_instance(params1.clone()).expect("Failed to create KVS instance");

            // Create seconds KVS instance.
            let kvs2 = kvs_instance(params2.clone()).expect("Failed to create KVS instance");

            // Set values to both KVS instances.
            kvs1.set_value(&keyname, value1)
                .expect("Failed to set value");
            kvs2.set_value(&keyname, value2)
                .expect("Failed to set value");

            // Flush KVS.
            kvs1.flush().expect("Failed to flush first instance");
            kvs2.flush().expect("Failed to flush second instance");
        }

        {
            // Second KVS run.
            let kvs1 = kvs_instance(params1).expect("Failed to create KVS instance");
            let kvs2 = kvs_instance(params2).expect("Failed to create KVS instance");

            let value1 = kvs1
                .get_value_as::<f64>(&keyname)
                .expect("Failed to read value");
            info!(instance = "kvs1", key = keyname, value = value1);
            let value2 = kvs2
                .get_value_as::<f64>(&keyname)
                .expect("Failed to read value");
            info!(instance = "kvs2", key = keyname, value = value2);
        }

        Ok(())
    }
}

struct SameInstanceIdSameValue;

impl Scenario for SameInstanceIdSameValue {
    fn name(&self) -> &str {
        "same_instance_id_same_value"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        // Values.
        let keyname = "number".to_string();
        let value = 111.1;

        // Parameters.
        let input_string = input.as_ref().expect("Test input is expected");
        let params = KvsParameters::from_json(input_string).expect("Failed to parse parameters");
        {
            // Create first KVS instance.
            let kvs1 = kvs_instance(params.clone()).expect("Failed to create KVS instance");

            // Create seconds KVS instance.
            let kvs2 = kvs_instance(params.clone()).expect("Failed to create KVS instance");

            // Set values to both KVS instances.
            kvs1.set_value(&keyname, value)
                .expect("Failed to set value");
            kvs2.set_value(&keyname, value)
                .expect("Failed to set value");

            // Flush KVS.
            kvs1.flush().expect("Failed to flush first instance");
            kvs2.flush().expect("Failed to flush second instance");
        }

        {
            // Second KVS run.
            let kvs1 = kvs_instance(params.clone()).expect("Failed to create KVS instance");
            let kvs2 = kvs_instance(params).expect("Failed to create KVS instance");

            let value1 = kvs1
                .get_value_as::<f64>(&keyname)
                .expect("Failed to read value");
            info!(instance = "kvs1", key = keyname, value = value1);
            let value2 = kvs2
                .get_value_as::<f64>(&keyname)
                .expect("Failed to read value");
            info!(instance = "kvs2", key = keyname, value = value2);
        }

        Ok(())
    }
}

struct SameInstanceIdDifferentValue;

impl Scenario for SameInstanceIdDifferentValue {
    fn name(&self) -> &str {
        "same_instance_id_diff_value"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        // Values.
        let keyname = "number".to_string();
        let value1 = 111.1;
        let value2 = 222.2;

        // Parameters.
        let input_string = input.as_ref().expect("Test input is expected");
        let params = KvsParameters::from_json(input_string).expect("Failed to parse parameters");
        {
            // Create first KVS instance.
            let kvs1 = kvs_instance(params.clone()).expect("Failed to create KVS instance");

            // Create seconds KVS instance.
            let kvs2 = kvs_instance(params.clone()).expect("Failed to create KVS instance");

            // Set values to both KVS instances.
            kvs1.set_value(&keyname, value1)
                .expect("Failed to set value");
            kvs2.set_value(&keyname, value2)
                .expect("Failed to set value");

            // Flush KVS.
            kvs1.flush().expect("Failed to flush first instance");
            kvs2.flush().expect("Failed to flush second instance");
        }

        {
            // Second KVS run.
            let kvs1 = kvs_instance(params.clone()).expect("Failed to create KVS instance");
            let kvs2 = kvs_instance(params).expect("Failed to create KVS instance");

            let value1 = kvs1
                .get_value_as::<f64>(&keyname)
                .expect("Failed to read value");
            info!(instance = "kvs1", key = keyname, value = value1);
            let value2 = kvs2
                .get_value_as::<f64>(&keyname)
                .expect("Failed to read value");
            info!(instance = "kvs2", key = keyname, value = value2);
        }

        Ok(())
    }
}

pub fn multiple_kvs_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "multiple_kvs",
        vec![
            Box::new(MultipleInstanceIds),
            Box::new(SameInstanceIdSameValue),
            Box::new(SameInstanceIdDifferentValue),
        ],
        vec![],
    ))
}
