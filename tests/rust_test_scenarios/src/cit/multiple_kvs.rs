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
        let v: Value = serde_json::from_str(input.as_ref().unwrap()).unwrap();
        let params1 = KvsParameters::from_value(&v["kvs_parameters_1"]).unwrap();
        let params2 = KvsParameters::from_value(&v["kvs_parameters_2"]).unwrap();
        {
            // Create first KVS instance.
            let kvs1 = kvs_instance(params1.clone()).unwrap();

            // Create seconds KVS instance.
            let kvs2 = kvs_instance(params2.clone()).unwrap();

            // Set values to both KVS instances.
            kvs1.set_value(&keyname, value1).unwrap();
            kvs2.set_value(&keyname, value2).unwrap();
        }

        {
            // Second KVS run.
            let kvs1 = kvs_instance(params1).unwrap();
            let kvs2 = kvs_instance(params2).unwrap();

            let value1 = kvs1.get_value_as::<f64>(&keyname).unwrap();
            info!(instance = "kvs1", key = keyname, value = value1);
            let value2 = kvs2.get_value_as::<f64>(&keyname).unwrap();
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
        let params = KvsParameters::from_json(input.as_ref().unwrap()).unwrap();
        {
            // Create first KVS instance.
            let kvs1 = kvs_instance(params.clone()).unwrap();

            // Create seconds KVS instance.
            let kvs2 = kvs_instance(params.clone()).unwrap();

            // Set values to both KVS instances.
            kvs1.set_value(&keyname, value).unwrap();
            kvs2.set_value(&keyname, value).unwrap();
        }

        {
            // Second KVS run.
            let kvs1 = kvs_instance(params.clone()).unwrap();
            let kvs2 = kvs_instance(params).unwrap();

            let value1 = kvs1.get_value_as::<f64>(&keyname).unwrap();
            info!(instance = "kvs1", key = keyname, value = value1);
            let value2 = kvs2.get_value_as::<f64>(&keyname).unwrap();
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
        let params = KvsParameters::from_json(input.as_ref().unwrap()).unwrap();
        {
            // Create first KVS instance.
            let kvs1 = kvs_instance(params.clone()).unwrap();

            // Create seconds KVS instance.
            let kvs2 = kvs_instance(params.clone()).unwrap();

            // Set values to both KVS instances.
            kvs1.set_value(&keyname, value1).unwrap();
            kvs2.set_value(&keyname, value2).unwrap();
        }

        {
            // Second KVS run.
            let kvs1 = kvs_instance(params.clone()).unwrap();
            let kvs2 = kvs_instance(params).unwrap();

            let value1 = kvs1.get_value_as::<f64>(&keyname).unwrap();
            info!(instance = "kvs1", key = keyname, value = value1);
            let value2 = kvs2.get_value_as::<f64>(&keyname).unwrap();
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
