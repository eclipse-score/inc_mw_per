use crate::helpers::kvs_instance::kvs_instance;
use crate::helpers::kvs_parameters::KvsParameters;
use rust_kvs::prelude::*;
use std::collections::HashMap;
use test_scenarios_rust::scenario::{Scenario, ScenarioGroup, ScenarioGroupImpl};
use tinyjson::JsonValue;
use tracing::info;

struct SupportedDatatypesKeys;

impl Scenario for SupportedDatatypesKeys {
    fn name(&self) -> &str {
        "keys"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let input_string = input.as_ref().expect("Test input is expected");
        let params = KvsParameters::from_json(input_string).expect("Failed to parse parameters");
        let kvs = kvs_instance(params).expect("Failed to create KVS instance");

        // Set key-value pairs. Unit type is used for value - only key is used later on.
        let keys_to_check = vec![
            String::from("example"),
            String::from("emoji ✅❗😀"),
            String::from("greek ημα"),
        ];
        for key in keys_to_check {
            kvs.set_value(key, ()).expect("Failed to set value");
        }

        // Get and print all keys.
        let keys_in_kvs = kvs.get_all_keys().expect("Failed to read all keys");
        for key in keys_in_kvs {
            info!(key);
        }

        Ok(())
    }
}

struct SupportedDatatypesValues {
    value: KvsValue,
}

impl Scenario for SupportedDatatypesValues {
    fn name(&self) -> &str {
        // Set scenario name based on provided value.
        match self.value {
            KvsValue::I32(_) => "i32",
            KvsValue::U32(_) => "u32",
            KvsValue::I64(_) => "i64",
            KvsValue::U64(_) => "u64",
            KvsValue::F64(_) => "f64",
            KvsValue::Boolean(_) => "bool",
            KvsValue::String(_) => "str",
            KvsValue::Null => "null",
            KvsValue::Array(_) => "arr",
            KvsValue::Object(_) => "obj",
        }
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let input_string = input.as_ref().expect("Test input is expected");
        let params = KvsParameters::from_json(input_string).expect("Failed to parse parameters");
        let kvs = kvs_instance(params).expect("Failed to create KVS instance");

        kvs.set_value(self.name(), self.value.clone())
            .expect("Failed to set value");

        let kvs_value = kvs.get_value(self.name()).expect("Failed to read value");
        let json_value = JsonValue::from(kvs_value);
        let json_str = json_value.stringify().expect("Failed to stringify JSON");

        info!(key = self.name(), value = json_str);

        Ok(())
    }
}

fn supported_datatypes_i32() -> Box<dyn Scenario> {
    Box::new(SupportedDatatypesValues {
        value: KvsValue::I32(-321),
    })
}

fn supported_datatypes_u32() -> Box<dyn Scenario> {
    Box::new(SupportedDatatypesValues {
        value: KvsValue::U32(1234),
    })
}

fn supported_datatypes_i64() -> Box<dyn Scenario> {
    Box::new(SupportedDatatypesValues {
        value: KvsValue::I64(-123456789),
    })
}

fn supported_datatypes_u64() -> Box<dyn Scenario> {
    Box::new(SupportedDatatypesValues {
        value: KvsValue::U64(123456789),
    })
}

fn supported_datatypes_f64() -> Box<dyn Scenario> {
    Box::new(SupportedDatatypesValues {
        value: KvsValue::F64(-5432.1),
    })
}

fn supported_datatypes_bool() -> Box<dyn Scenario> {
    Box::new(SupportedDatatypesValues {
        value: KvsValue::Boolean(true),
    })
}

fn supported_datatypes_string() -> Box<dyn Scenario> {
    Box::new(SupportedDatatypesValues {
        value: KvsValue::String("example".to_string()),
    })
}

fn supported_datatypes_array() -> Box<dyn Scenario> {
    let hashmap = HashMap::from([("sub-number".to_string(), KvsValue::from(789.0))]);
    let array = vec![
        KvsValue::from(321.5),
        KvsValue::from(false),
        KvsValue::from("hello".to_string()),
        KvsValue::from(()),
        KvsValue::from(vec![]),
        KvsValue::from(hashmap),
    ];
    Box::new(SupportedDatatypesValues {
        value: KvsValue::Array(array),
    })
}

fn supported_datatypes_object() -> Box<dyn Scenario> {
    let hashmap = HashMap::from([("sub-number".to_string(), KvsValue::from(789.0))]);
    Box::new(SupportedDatatypesValues {
        value: KvsValue::Object(hashmap),
    })
}

fn value_types_group() -> Box<dyn ScenarioGroup> {
    let group = ScenarioGroupImpl::new(
        "values",
        vec![
            supported_datatypes_i32(),
            supported_datatypes_u32(),
            supported_datatypes_i64(),
            supported_datatypes_u64(),
            supported_datatypes_f64(),
            supported_datatypes_bool(),
            supported_datatypes_string(),
            supported_datatypes_array(),
            supported_datatypes_object(),
        ],
        vec![],
    );
    Box::new(group)
}

pub fn supported_datatypes_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "supported_datatypes",
        vec![Box::new(SupportedDatatypesKeys)],
        vec![value_types_group()],
    ))
}
