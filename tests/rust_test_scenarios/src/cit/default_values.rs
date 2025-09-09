use crate::helpers::kvs_instance::kvs_instance;
use crate::helpers::kvs_parameters::KvsParameters;
use crate::helpers::to_str;
use rust_kvs::prelude::*;
use test_scenarios_rust::scenario::{Scenario, ScenarioGroup, ScenarioGroupImpl};
use tracing::info;

/// Common test for default values.
struct DefaultValues;

impl Scenario for DefaultValues {
    fn name(&self) -> &str {
        "default_values"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        // Key used for tests.
        let key = "test_number";

        // Create KVS instance with provided params.
        let input_string = input.as_ref().expect("Test input is expected");
        let params = KvsParameters::from_json(input_string).expect("Failed to parse parameters");
        {
            let kvs = kvs_instance(params.clone()).expect("Failed to create KVS instance");

            // Get current value parameters.
            let value_is_default = to_str(&kvs.is_value_default(key));
            let default_value = to_str(&kvs.get_default_value(key));
            let current_value = to_str(&kvs.get_value(key));

            info!(key, value_is_default, default_value, current_value);

            // Set value and check value parameters.
            kvs.set_value(key, 432.1).expect("Failed to set value");
        }

        // Flush and reopen KVS instance to ensure persistency.
        {
            let kvs = kvs_instance(params).expect("Failed to create KVS instance");

            // Get current value parameters.
            let value_is_default = to_str(&kvs.is_value_default(key));
            let default_value = to_str(&kvs.get_default_value(key));
            let current_value = to_str(&kvs.get_value(key));

            info!(key, value_is_default, default_value, current_value);
        }

        Ok(())
    }
}

/// Remove key.
struct RemoveKey;

impl Scenario for RemoveKey {
    fn name(&self) -> &str {
        "remove_key"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        // Key used for tests.
        let key = "test_number";

        // Create KVS instance with provided params.
        let input_string = input.as_ref().expect("Test input is expected");
        let params = KvsParameters::from_json(input_string).expect("Failed to parse parameters");
        {
            let kvs = kvs_instance(params.clone()).expect("Failed to create KVS instance");

            // Get value parameters before set.
            let value_is_default = to_str(&kvs.is_value_default(key));
            let default_value = to_str(&kvs.get_default_value(key));
            let current_value = to_str(&kvs.get_value(key));

            info!(key, value_is_default, default_value, current_value);

            // Get value parameters after set.
            kvs.set_value(key, 432.1).expect("Failed to set value");

            // Get current value parameters.
            let value_is_default = to_str(&kvs.is_value_default(key));
            let default_value = to_str(&kvs.get_default_value(key));
            let current_value = to_str(&kvs.get_value(key));

            info!(key, value_is_default, default_value, current_value);

            // Get value parameters after remove.
            kvs.remove_key(key).expect("Failed to remove key");
            let value_is_default = to_str(&kvs.is_value_default(key));
            let default_value = to_str(&kvs.get_default_value(key));
            let current_value = to_str(&kvs.get_value(key));

            info!(key, value_is_default, default_value, current_value);
        }

        Ok(())
    }
}

/// Reset all keys.
struct ResetAllKeys;

impl Scenario for ResetAllKeys {
    fn name(&self) -> &str {
        "reset_all_keys"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let num_values = 5;

        // Create KVS instance with provided params.
        let input_string = input.as_ref().expect("Test input is expected");
        let params = KvsParameters::from_json(input_string).expect("Failed to parse parameters");
        {
            let kvs = kvs_instance(params.clone()).expect("Failed to create KVS instance");

            // List of keys and corresponding values.
            let mut key_values = Vec::new();
            for i in 0..num_values {
                let key = format!("test_number_{i}");
                let value = 123.4 * i as f64;
                key_values.push((key, value));
            }

            // Set non-default values.
            for (key, value) in key_values.iter() {
                // Get value parameters before set.
                let value_is_default = kvs
                    .is_value_default(key)
                    .expect("Failed to check if default value");
                let current_value = kvs.get_value_as::<f64>(key).expect("Failed to read value");

                info!(key = key, value_is_default, current_value);

                // Set value.
                kvs.set_value(key.clone(), *value)
                    .expect("Failed to set value");

                // Get value parameters after set.
                let value_is_default = kvs
                    .is_value_default(key)
                    .expect("Failed to check if default value");
                let current_value = kvs.get_value_as::<f64>(key).expect("Failed to read value");

                info!(key, value_is_default, current_value);
            }

            // Reset.
            kvs.reset().expect("Failed to reset KVS instance");

            // Get value parameters after reset.
            for (key, _) in key_values.iter() {
                let value_is_default = kvs
                    .is_value_default(key)
                    .expect("Failed to check if default value");
                let current_value = kvs.get_value_as::<f64>(key).expect("Failed to read value");

                info!(key, value_is_default, current_value);
            }
        }

        Ok(())
    }
}

/// Reset single key.
struct ResetSingleKey;

impl Scenario for ResetSingleKey {
    fn name(&self) -> &str {
        "reset_single_key"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let num_values = 5;
        let reset_index = 2;

        // Create KVS instance with provided params.
        let input_string = input.as_ref().expect("Test input is expected");
        let params = KvsParameters::from_json(input_string).expect("Failed to parse parameters");
        {
            let kvs = kvs_instance(params.clone()).expect("Failed to create KVS instance");

            // List of keys and corresponding values.
            let mut key_values = Vec::new();
            for i in 0..num_values {
                let key = format!("test_number_{i}");
                let value = 123.4 * i as f64;
                key_values.push((key, value));
            }

            // Set non-default values.
            for (key, value) in key_values.iter() {
                // Get value parameters before set.
                let value_is_default = kvs
                    .is_value_default(key)
                    .expect("Failed to check if default value");
                let current_value = kvs.get_value_as::<f64>(key).expect("Failed to read value");

                info!(key = key, value_is_default, current_value);

                // Set value.
                kvs.set_value(key.clone(), *value)
                    .expect("Failed to set value");

                // Get value parameters after set.
                let value_is_default = kvs
                    .is_value_default(key)
                    .expect("Failed to check if default value");
                let current_value = kvs.get_value_as::<f64>(key).expect("Failed to read value");

                info!(key, value_is_default, current_value);
            }

            // Reset single key.
            kvs.reset_key(&key_values.get(reset_index).expect("Failed to read value").0)
                .expect("Failed to reset key");

            // Get value parameters after reset.
            for (key, _) in key_values.iter() {
                let value_is_default = kvs
                    .is_value_default(key)
                    .expect("Failed to check if default value");
                let current_value = kvs.get_value_as::<f64>(key).expect("Failed to read value");

                info!(key, value_is_default, current_value);
            }
        }

        Ok(())
    }
}

/// Check hash file is created when KVS is init with defaults.
struct Checksum;

impl Scenario for Checksum {
    fn name(&self) -> &str {
        "checksum"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        // Create KVS instance with provided params.
        let input_string = input.as_ref().expect("Test input is expected");
        let params = KvsParameters::from_json(input_string).expect("Failed to parse parameters");
        let kvs_path;
        let hash_path;
        {
            // Create instance, flush, store paths to files, close instance.
            let kvs = kvs_instance(params.clone()).expect("Failed to create KVS instance");
            kvs.flush().expect("Failed to flush KVS instance");
            kvs_path = kvs
                .get_kvs_filename(SnapshotId(0))
                .expect("Failed to get KVS file path");
            hash_path = kvs
                .get_hash_filename(SnapshotId(0))
                .expect("Failed to get hash file path");
        }
        info!(
            kvs_path = kvs_path.display().to_string(),
            hash_path = hash_path.display().to_string()
        );

        Ok(())
    }
}

pub fn default_values_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "default_values",
        vec![
            Box::new(DefaultValues),
            Box::new(RemoveKey),
            Box::new(ResetAllKeys),
            Box::new(ResetSingleKey),
            Box::new(Checksum),
        ],
        vec![],
    ))
}
