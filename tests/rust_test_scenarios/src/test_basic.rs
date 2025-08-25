use rust_kvs::{kvs_api::FlushOnExit, prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use test_scenarios_rust::scenario::Scenario;
use tracing::info;

#[derive(Serialize, Deserialize, Debug)]
struct InputParameters {
    instance_id: usize,
    need_defaults: Option<bool>,
    need_kvs: Option<bool>,
    dir: Option<String>,
    flush_on_exit: Option<bool>,
}

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
        eprintln!("{}", input.clone().unwrap());

        let v: Value = serde_json::from_str(input.as_deref().unwrap()).unwrap();
        let input_parameters: InputParameters =
            serde_json::from_value(v["kvs_parameters"].clone()).unwrap();

        // Set KVS parameters.
        let instance_id = InstanceId(input_parameters.instance_id);
        let mut kvs_parameters = KvsParameters::new(instance_id);
        if let Some(flag) = input_parameters.need_defaults {
            kvs_parameters = kvs_parameters.defaults(if flag {
                Defaults::Required
            } else {
                Defaults::Optional
            });
        }
        if let Some(flag) = input_parameters.need_kvs {
            kvs_parameters = kvs_parameters.kvs_load(if flag {
                KvsLoad::Required
            } else {
                KvsLoad::Optional
            });
        }
        let working_dir = match input_parameters.dir {
            Some(p) => PathBuf::from(p),
            None => PathBuf::new(),
        };

        // Create KVS.
        let mut provider = KvsProvider::new(working_dir);
        let kvs = provider.init(kvs_parameters).unwrap();
        if let Some(flag) = input_parameters.flush_on_exit {
            kvs.set_flush_on_exit(if flag {
                FlushOnExit::Yes
            } else {
                FlushOnExit::No
            })
            .unwrap();
        }

        // Simple set/get.
        let key = "example_key";
        let value = "example_value".to_string();
        kvs.set_value(key, value).unwrap();
        let value_read = kvs.get_value_as::<String>(key).unwrap();

        // Trace.
        info!(example_key = value_read);

        Ok(())
    }
}
