use crate::helpers::kvs_parameters::KvsParameters;
use rust_kvs::prelude::*;
use test_scenarios_rust::scenario::Scenario;
use tracing::info;

fn _error_code_to_string(e: ErrorCode) -> String {
    format!("ErrorCode::{e:?}")
}

pub struct BasicScenario;

/// Checks (almost) empty program with only shutdown
impl Scenario for BasicScenario {
    fn name(&self) -> &'static str {
        "basic"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        // Print and parse parameters.
        let input_string = input.as_ref().expect("Test input is expected");
        eprintln!("{input_string}");

        let params = KvsParameters::from_json(input_string).expect("Failed to parse parameters");

        // Set builder parameters.
        let mut builder = KvsBuilder::new(params.instance_id);
        if let Some(flag) = params.defaults {
            builder = builder.defaults(flag);
        }
        if let Some(flag) = params.kvs_load {
            builder = builder.kvs_load(flag);
        }
        if let Some(dir) = params.dir {
            builder = builder.dir(dir.to_string_lossy().to_string());
        }

        // Create KVS.
        let kvs: Kvs = builder.build().expect("Failed to build KVS instance");

        // Simple set/get.
        let key = "example_key";
        let value = "example_value".to_string();
        kvs.set_value(key, value).expect("Failed to set value");
        let value_read = kvs
            .get_value_as::<String>(key)
            .expect("Failed to read value");

        // Trace.
        info!(example_key = value_read);

        Ok(())
    }
}
