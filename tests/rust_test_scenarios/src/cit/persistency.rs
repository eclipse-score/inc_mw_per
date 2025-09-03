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
        let params = KvsParameters::from_json(input.as_ref().unwrap()).unwrap();
        {
            // First KVS instance object - used for setting and flushing data.
            let kvs = kvs_instance(params.clone()).unwrap();

            // Set values.
            for (key, value) in key_values.iter() {
                kvs.set_value(key, *value).unwrap();
            }

            // Explicit flush.
            kvs.flush().unwrap();
        }

        {
            // Second KVS instance object - used for flush check.
            let kvs = kvs_instance(params).unwrap();

            // Get values.
            for (key, _) in key_values.iter() {
                let value = kvs.get_value(key);
                info!(key, value = to_str(&value));
            }
        }

        Ok(())
    }
}

struct TestFlushOnExit;

impl Scenario for TestFlushOnExit {
    fn name(&self) -> &str {
        "flush_on_exit"
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
        let params = KvsParameters::from_json(input.as_ref().unwrap()).unwrap();
        {
            // First KVS instance object - used for setting and flushing data.
            let kvs = kvs_instance(params.clone()).unwrap();

            // Set values.
            for (key, value) in key_values.iter() {
                kvs.set_value(key, *value).unwrap();
            }

            // Flush happens on `kvs` going out of scope and `flush_on_exit` enabled.
        }

        {
            // Second KVS instance object - used for flush check.
            let kvs = kvs_instance(params).unwrap();

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
        vec![Box::new(ExplicitFlush), Box::new(TestFlushOnExit)],
        vec![],
    ))
}
