use rust_kvs::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use test_scenarios_rust::scenario::Scenario;
use tracing::info;

#[derive(Serialize, Deserialize, Debug)]
struct KvsParameters {
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
        let params: KvsParameters = serde_json::from_value(v["kvs_parameters"].clone()).unwrap();

        // Set builder parameters.
        let instance_id = InstanceId(params.instance_id);
        let mut builder = KvsBuilder::new(instance_id);
        if let Some(flag) = params.need_defaults {
            builder = builder.need_defaults(flag);
        }
        if let Some(flag) = params.need_kvs {
            builder = builder.need_kvs(flag);
        }
        if let Some(dir) = params.dir {
            builder = builder.dir(dir);
        }

        // Create KVS.
        let kvs: Kvs = builder.build().unwrap();
        if let Some(flag) = params.flush_on_exit {
            kvs.flush_on_exit(flag);
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
