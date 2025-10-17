// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// <https://www.apache.org/licenses/LICENSE-2.0>
//
// SPDX-License-Identifier: Apache-2.0

use crate::error_code::ErrorCode;
use crate::kvs_api::{InstanceId, KvsApi, KvsDefaults, KvsLoad, SnapshotId};
use crate::kvs_backend::KvsBackend;
use crate::kvs_builder::KvsData;
use crate::kvs_value::{KvsMap, KvsValue};
use std::sync::{Arc, Mutex};

/// KVS instance parameters.
#[derive(Debug, PartialEq)]
pub struct KvsParameters {
    /// Instance ID.
    pub instance_id: InstanceId,

    /// Defaults handling mode.
    pub defaults: KvsDefaults,

    /// KVS load mode.
    pub kvs_load: KvsLoad,

    /// Backend parameters.
    pub backend_parameters: KvsMap,
}

/// Key-value-storage data
pub struct Kvs {
    /// KVS instance data.
    data: Arc<Mutex<KvsData>>,

    /// KVS instance parameters.
    parameters: Arc<KvsParameters>,

    /// Backend.
    backend: Box<dyn KvsBackend>,
}

impl Kvs {
    pub(crate) fn new(
        data: Arc<Mutex<KvsData>>,
        parameters: Arc<KvsParameters>,
        backend: Box<dyn KvsBackend>,
    ) -> Self {
        Self {
            data,
            parameters,
            backend,
        }
    }

    /// KVS instance parameters.
    pub fn parameters(&self) -> &KvsParameters {
        &self.parameters
    }
}

impl KvsApi for Kvs {
    /// Resets a key-value-storage to its initial state
    ///
    /// # Return Values
    ///   * Ok: Reset of the KVS was successful
    ///   * `ErrorCode::MutexLockFailed`: Mutex locking failed
    fn reset(&self) -> Result<(), ErrorCode> {
        let mut data = self.data.lock()?;
        data.kvs_map = KvsMap::new();
        Ok(())
    }

    /// Reset a key-value pair in the storage to its initial state
    ///
    /// # Parameters
    ///    * 'key': Key being reset to default
    ///
    /// # Return Values
    ///    * Ok: Reset of the key-value pair was successful
    ///    * `ErrorCode::MutexLockFailed`: Mutex locking failed
    ///    * `ErrorCode::KeyDefaultNotFound`: Key has no default value
    fn reset_key(&self, key: &str) -> Result<(), ErrorCode> {
        let mut data = self.data.lock()?;
        if !data.defaults_map.contains_key(key) {
            eprintln!("error: resetting key without a default value");
            return Err(ErrorCode::KeyDefaultNotFound);
        }

        let _ = data.kvs_map.remove(key);
        Ok(())
    }

    /// Get list of all keys
    ///
    /// # Return Values
    ///   * Ok: List of all keys
    ///   * `ErrorCode::MutexLockFailed`: Mutex locking failed
    fn get_all_keys(&self) -> Result<Vec<String>, ErrorCode> {
        let data = self.data.lock()?;
        Ok(data.kvs_map.keys().map(|x| x.to_string()).collect())
    }

    /// Check if a key exists
    ///
    /// # Parameters
    ///   * `key`: Key to check for existence
    ///
    /// # Return Values
    ///   * Ok(`true`): Key exists
    ///   * Ok(`false`): Key doesn't exist
    ///   * `ErrorCode::MutexLockFailed`: Mutex locking failed
    fn key_exists(&self, key: &str) -> Result<bool, ErrorCode> {
        let data = self.data.lock()?;
        Ok(data.kvs_map.contains_key(key))
    }

    /// Get the assigned value for a given key
    ///
    /// # Features
    ///   * `FEAT_REQ__KVS__default_values`
    ///
    /// # Parameters
    ///   * `key`: Key to retrieve the value from
    ///
    /// # Return Value
    ///   * Ok: Type specific value if key was found
    ///   * `ErrorCode::MutexLockFailed`: Mutex locking failed
    ///   * `ErrorCode::KeyNotFound`: Key wasn't found in KVS nor in defaults
    fn get_value(&self, key: &str) -> Result<KvsValue, ErrorCode> {
        let data = self.data.lock()?;
        if let Some(value) = data.kvs_map.get(key) {
            Ok(value.clone())
        } else if let Some(value) = data.defaults_map.get(key) {
            Ok(value.clone())
        } else {
            eprintln!("error: get_value could not find key: {key}");
            Err(ErrorCode::KeyNotFound)
        }
    }

    /// Get the assigned value for a given key
    ///
    /// See [Variants](https://docs.rs/tinyjson/latest/tinyjson/enum.JsonValue.html#variants) for
    /// supported value types.
    ///
    /// # Features
    ///   * `FEAT_REQ__KVS__default_values`
    ///
    /// # Parameters
    ///   * `key`: Key to retrieve the value from
    ///
    /// # Return Value
    ///   * Ok: Type specific value if key was found
    ///   * `ErrorCode::MutexLockFailed`: Mutex locking failed
    ///   * `ErrorCode::ConversionFailed`: Type conversion failed
    ///   * `ErrorCode::KeyNotFound`: Key wasn't found in KVS nor in defaults
    fn get_value_as<T>(&self, key: &str) -> Result<T, ErrorCode>
    where
        for<'a> T: TryFrom<&'a KvsValue> + std::clone::Clone,
        for<'a> <T as TryFrom<&'a KvsValue>>::Error: std::fmt::Debug,
    {
        let data = self.data.lock()?;
        if let Some(value) = data.kvs_map.get(key) {
            match T::try_from(value) {
                Ok(value) => Ok(value),
                Err(err) => {
                    eprintln!(
                        "error: get_value could not convert KvsValue from KVS store: {err:#?}"
                    );
                    Err(ErrorCode::ConversionFailed)
                }
            }
        } else if let Some(value) = data.defaults_map.get(key) {
            // check if key has a default value
            match T::try_from(value) {
                Ok(value) => Ok(value),
                Err(err) => {
                    eprintln!(
                        "error: get_value could not convert KvsValue from default store: {err:#?}"
                    );
                    Err(ErrorCode::ConversionFailed)
                }
            }
        } else {
            eprintln!("error: get_value could not find key: {key}");

            Err(ErrorCode::KeyNotFound)
        }
    }

    /// Get default value for a given key
    ///
    /// # Features
    ///   * `FEAT_REQ__KVS__default_values`
    ///   * `FEAT_REQ__KVS__default_value_retrieval`
    ///
    /// # Parameters
    ///   * `key`: Key to get the default for
    ///
    /// # Return Values
    ///   * Ok: `KvsValue` for the key
    ///   * `ErrorCode::KeyNotFound`: Key not found in defaults
    fn get_default_value(&self, key: &str) -> Result<KvsValue, ErrorCode> {
        let data = self.data.lock()?;
        if let Some(value) = data.defaults_map.get(key) {
            Ok(value.clone())
        } else {
            Err(ErrorCode::KeyNotFound)
        }
    }

    /// Return if the value wasn't set yet and uses its default value
    ///
    /// # Features
    ///   * `FEAT_REQ__KVS__default_values`
    ///
    /// # Parameters
    ///   * `key`: Key to check if a default exists
    ///
    /// # Return Values
    ///   * Ok(true): Key currently returns the default value
    ///   * Ok(false): Key returns the set value
    ///   * `ErrorCode::MutexLockFailed`: Mutex locking failed
    ///   * `ErrorCode::KeyNotFound`: Key wasn't found
    fn is_value_default(&self, key: &str) -> Result<bool, ErrorCode> {
        let data = self.data.lock()?;
        if data.kvs_map.contains_key(key) {
            Ok(false)
        } else if data.defaults_map.contains_key(key) {
            Ok(true)
        } else {
            Err(ErrorCode::KeyNotFound)
        }
    }

    /// Assign a value to a given key
    ///
    /// # Parameters
    ///   * `key`: Key to set value
    ///   * `value`: Value to be set
    ///
    /// # Return Values
    ///   * Ok: Value was assigned to key
    ///   * `ErrorCode::MutexLockFailed`: Mutex locking failed
    fn set_value<S: Into<String>, V: Into<KvsValue>>(
        &self,
        key: S,
        value: V,
    ) -> Result<(), ErrorCode> {
        let mut data = self.data.lock()?;
        data.kvs_map.insert(key.into(), value.into());
        Ok(())
    }

    /// Remove a key
    ///
    /// # Parameters
    ///   * `key`: Key to remove
    ///
    /// # Return Values
    ///   * Ok: Key removed successfully
    ///   * `ErrorCode::MutexLockFailed`: Mutex locking failed
    ///   * `ErrorCode::KeyNotFound`: Key not found
    fn remove_key(&self, key: &str) -> Result<(), ErrorCode> {
        let mut data = self.data.lock()?;
        if data.kvs_map.remove(key).is_some() {
            Ok(())
        } else {
            Err(ErrorCode::KeyNotFound)
        }
    }

    /// Flush the in-memory key-value-storage to the persistent storage
    ///
    /// # Features
    ///   * `FEAT_REQ__KVS__snapshots`
    ///   * `FEAT_REQ__KVS__persistency`
    ///   * `FEAT_REQ__KVS__integrity_check`
    ///
    /// # Return Values
    ///   * Ok: Flush successful
    ///   * `ErrorCode::MutexLockFailed`: Mutex locking failed
    ///   * `ErrorCode::JsonGeneratorError`: Failed to serialize to JSON
    ///   * `ErrorCode::ConversionFailed`: JSON could not serialize into String
    ///   * `ErrorCode::UnmappedError`: Unmapped error
    fn flush(&self) -> Result<(), ErrorCode> {
        if self.snapshot_max_count() == 0 {
            eprintln!("warn: snapshot_max_count == 0, flush ignored");
            return Ok(());
        }

        let data = self.data.lock()?;
        self.backend
            .flush(self.parameters.instance_id, &data.kvs_map)
    }

    /// Get the count of snapshots
    ///
    /// # Return Values
    ///   * usize: Count of found snapshots
    fn snapshot_count(&self) -> usize {
        self.backend.snapshot_count(self.parameters.instance_id)
    }

    /// Return maximum number of snapshots to store.
    ///
    /// # Return Values
    ///   * usize: Maximum count of snapshots
    fn snapshot_max_count(&self) -> usize {
        self.backend.snapshot_max_count()
    }

    /// Recover key-value-storage from snapshot
    ///
    /// Restore a previously created KVS snapshot.
    ///
    /// # Features
    ///   * `FEAT_REQ__KVS__snapshots`
    ///
    /// # Parameters
    ///   * `id`: Snapshot ID
    ///
    /// # Return Values
    ///   * `Ok`: Snapshot restored
    ///   * `ErrorCode::InvalidSnapshotId`: Invalid snapshot ID
    ///   * `ErrorCode::ValidationFailed`: KVS hash validation failed
    ///   * `ErrorCode::JsonParserError`: JSON parser error
    ///   * `ErrorCode::KvsFileReadError`: KVS file not found
    ///   * `ErrorCode::KvsHashFileReadError`: KVS hash file read error
    ///   * `ErrorCode::UnmappedError`: Generic error
    fn snapshot_restore(&self, snapshot_id: SnapshotId) -> Result<(), ErrorCode> {
        let mut data = self.data.lock()?;
        data.kvs_map = self
            .backend
            .snapshot_restore(self.parameters.instance_id, snapshot_id)?;
        Ok(())
    }
}

#[cfg(test)]
mod kvs_tests {
    use crate::error_code::ErrorCode;
    use crate::json_backend::JsonBackendBuilder;
    use crate::kvs::{Kvs, KvsParameters};
    use crate::kvs_api::{InstanceId, KvsApi, KvsDefaults, KvsLoad, SnapshotId};
    use crate::kvs_backend::KvsBackend;
    use crate::kvs_builder::KvsData;
    use crate::kvs_value::{KvsMap, KvsValue};
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;

    /// Most tests can be performed with mocked backend.
    /// Only those with file handling must use concrete implementation.
    #[derive(PartialEq)]
    struct MockBackend;

    impl KvsBackend for MockBackend {
        fn load_kvs(
            &self,
            _instance_id: InstanceId,
            _snapshot_id: SnapshotId,
        ) -> Result<KvsMap, ErrorCode> {
            unimplemented!()
        }

        fn load_defaults(&self, _instance_id: InstanceId) -> Result<KvsMap, ErrorCode> {
            unimplemented!()
        }

        fn flush(&self, _instance_id: InstanceId, _kvs_map: &KvsMap) -> Result<(), ErrorCode> {
            unimplemented!()
        }

        fn snapshot_count(&self, _instance_id: InstanceId) -> usize {
            unimplemented!()
        }

        fn snapshot_max_count(&self) -> usize {
            unimplemented!()
        }

        fn snapshot_restore(
            &self,
            _instance_id: InstanceId,
            _snapshot_id: SnapshotId,
        ) -> Result<KvsMap, ErrorCode> {
            unimplemented!()
        }
    }

    fn get_kvs(backend: Box<dyn KvsBackend>, kvs_map: KvsMap, defaults_map: KvsMap) -> Kvs {
        let instance_id = InstanceId(1);
        let data = Arc::new(Mutex::new(KvsData {
            kvs_map,
            defaults_map,
        }));
        let parameters = Arc::new(KvsParameters {
            instance_id,
            defaults: KvsDefaults::Optional,
            kvs_load: KvsLoad::Optional,
            backend_parameters: KvsMap::new(),
        });
        Kvs::new(data, parameters, backend)
    }

    #[test]
    fn test_new_ok() {
        // Check only if panic happens.
        get_kvs(Box::new(MockBackend), KvsMap::new(), KvsMap::new());
    }

    #[test]
    fn test_parameters_ok() {
        let kvs = get_kvs(Box::new(MockBackend), KvsMap::new(), KvsMap::new());

        // Assert params as expected.
        let expected_parameters = KvsParameters {
            instance_id: InstanceId(1),
            defaults: KvsDefaults::Optional,
            kvs_load: KvsLoad::Optional,
            backend_parameters: KvsMap::new(),
        };

        assert_eq!(*kvs.parameters(), expected_parameters);
    }

    #[test]
    fn test_reset() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([
                ("example1".to_string(), KvsValue::from("explicit_value")),
                ("example2".to_string(), KvsValue::from(true)),
            ]),
            KvsMap::from([("example1".to_string(), KvsValue::from("default_value"))]),
        );

        kvs.reset().unwrap();
        assert_eq!(kvs.get_all_keys().unwrap().len(), 0);
        assert_eq!(
            kvs.get_value_as::<String>("example1").unwrap(),
            "default_value"
        );
        assert!(kvs
            .get_value_as::<bool>("example2")
            .is_err_and(|e| e == ErrorCode::KeyNotFound));
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_reset_key() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([
                ("example1".to_string(), KvsValue::from("explicit_value")),
                ("example2".to_string(), KvsValue::from(true)),
            ]),
            KvsMap::from([("example1".to_string(), KvsValue::from("default_value"))]),
        );

        kvs.reset_key("example1").unwrap();
        assert_eq!(
            kvs.get_value_as::<String>("example1").unwrap(),
            "default_value"
        );

        // TODO: determine why resetting entry without default value is an error.
        assert!(kvs
            .reset_key("example2")
            .is_err_and(|e| e == ErrorCode::KeyDefaultNotFound));
    }

    #[test]
    fn test_get_all_keys_some() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([
                ("example1".to_string(), KvsValue::from("value")),
                ("example2".to_string(), KvsValue::from(true)),
            ]),
            KvsMap::new(),
        );

        let mut keys = kvs.get_all_keys().unwrap();
        keys.sort();
        assert_eq!(keys, vec!["example1", "example2"]);
    }

    #[test]
    fn test_get_all_keys_empty() {
        let kvs = get_kvs(Box::new(MockBackend), KvsMap::new(), KvsMap::new());

        let keys = kvs.get_all_keys().unwrap();
        assert_eq!(keys.len(), 0);
    }

    #[test]
    fn test_key_exists_found() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([
                ("example1".to_string(), KvsValue::from("value")),
                ("example2".to_string(), KvsValue::from(true)),
            ]),
            KvsMap::new(),
        );

        assert!(kvs.key_exists("example1").unwrap());
        assert!(kvs.key_exists("example2").unwrap());
    }

    #[test]
    fn test_key_exists_not_found() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([
                ("example1".to_string(), KvsValue::from("value")),
                ("example2".to_string(), KvsValue::from(true)),
            ]),
            KvsMap::new(),
        );

        assert!(!kvs.key_exists("invalid_key").unwrap());
    }

    #[test]
    fn test_get_value_found() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([
                ("example1".to_string(), KvsValue::from("value")),
                ("example2".to_string(), KvsValue::from(true)),
            ]),
            KvsMap::new(),
        );

        let value = kvs.get_value("example1").unwrap();
        assert_eq!(value, KvsValue::String("value".to_string()));
    }

    #[test]
    fn test_get_value_available_default() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([("example2".to_string(), KvsValue::from(true))]),
            KvsMap::from([("example1".to_string(), KvsValue::from("default_value"))]),
        );

        assert_eq!(
            kvs.get_value("example1").unwrap(),
            KvsValue::String("default_value".to_string())
        );
    }

    #[test]
    fn test_get_value_not_found() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([("example2".to_string(), KvsValue::from(true))]),
            KvsMap::from([("example1".to_string(), KvsValue::from("default_value"))]),
        );

        assert!(kvs
            .get_value("invalid_key")
            .is_err_and(|e| e == ErrorCode::KeyNotFound));
    }

    #[test]
    fn test_get_value_as_found() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([
                ("example1".to_string(), KvsValue::from("value")),
                ("example2".to_string(), KvsValue::from(true)),
            ]),
            KvsMap::new(),
        );

        let value = kvs.get_value_as::<String>("example1").unwrap();
        assert_eq!(value, "value");
    }

    #[test]
    fn test_get_value_as_available_default() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([("example2".to_string(), KvsValue::from(true))]),
            KvsMap::from([("example1".to_string(), KvsValue::from("default_value"))]),
        );

        let value = kvs.get_value_as::<String>("example1").unwrap();
        assert_eq!(value, "default_value");
    }

    #[test]
    fn test_get_value_as_not_found() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([("example2".to_string(), KvsValue::from(true))]),
            KvsMap::from([("example1".to_string(), KvsValue::from("default_value"))]),
        );

        assert!(kvs
            .get_value_as::<String>("invalid_key")
            .is_err_and(|e| e == ErrorCode::KeyNotFound));
    }

    #[test]
    fn test_get_value_as_invalid_type() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([
                ("example1".to_string(), KvsValue::from("value")),
                ("example2".to_string(), KvsValue::from(true)),
            ]),
            KvsMap::new(),
        );

        assert!(kvs
            .get_value_as::<f64>("example1")
            .is_err_and(|e| e == ErrorCode::ConversionFailed));
    }

    #[test]
    fn test_get_value_as_default_invalid_type() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([("example2".to_string(), KvsValue::from(true))]),
            KvsMap::from([("example1".to_string(), KvsValue::from("default_value"))]),
        );

        assert!(kvs
            .get_value_as::<f64>("example1")
            .is_err_and(|e| e == ErrorCode::ConversionFailed));
    }

    #[test]
    fn test_get_default_value_found() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([
                ("example1".to_string(), KvsValue::from("value")),
                ("example2".to_string(), KvsValue::from(true)),
            ]),
            KvsMap::from([("example3".to_string(), KvsValue::from("default"))]),
        );

        let value = kvs.get_default_value("example3").unwrap();
        assert_eq!(value, KvsValue::String("default".to_string()));
    }

    #[test]
    fn test_get_default_value_not_found() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([
                ("example1".to_string(), KvsValue::from("value")),
                ("example2".to_string(), KvsValue::from(true)),
            ]),
            KvsMap::from([("example3".to_string(), KvsValue::from("default"))]),
        );

        assert!(kvs
            .get_default_value("invalid_key")
            .is_err_and(|e| e == ErrorCode::KeyNotFound));
    }

    #[test]
    fn test_is_value_default_false() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([
                ("example1".to_string(), KvsValue::from("value")),
                ("example2".to_string(), KvsValue::from(true)),
            ]),
            KvsMap::from([("example1".to_string(), KvsValue::from("default"))]),
        );

        assert!(!kvs.is_value_default("example1").unwrap());
    }

    #[test]
    fn test_is_value_default_true() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([
                ("example1".to_string(), KvsValue::from("value")),
                ("example2".to_string(), KvsValue::from(true)),
            ]),
            KvsMap::from([("example3".to_string(), KvsValue::from("default"))]),
        );

        assert!(kvs.is_value_default("example3").unwrap());
    }

    #[test]
    fn test_is_value_default_not_found() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([
                ("example1".to_string(), KvsValue::from("value")),
                ("example2".to_string(), KvsValue::from(true)),
            ]),
            KvsMap::from([("example1".to_string(), KvsValue::from("default"))]),
        );

        assert!(kvs
            .is_value_default("invalid_key")
            .is_err_and(|e| e == ErrorCode::KeyNotFound));
    }

    #[test]
    fn test_set_value_new() {
        let kvs = get_kvs(Box::new(MockBackend), KvsMap::new(), KvsMap::new());

        kvs.set_value("key", "value").unwrap();
        assert_eq!(kvs.get_value_as::<String>("key").unwrap(), "value");
    }

    #[test]
    fn test_set_value_exists() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([("key".to_string(), KvsValue::from("old_value"))]),
            KvsMap::new(),
        );

        kvs.set_value("key", "new_value").unwrap();
        assert_eq!(kvs.get_value_as::<String>("key").unwrap(), "new_value");
    }

    #[test]
    fn test_remove_key_found() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([
                ("example1".to_string(), KvsValue::from("value")),
                ("example2".to_string(), KvsValue::from(true)),
            ]),
            KvsMap::new(),
        );

        kvs.remove_key("example1").unwrap();
        assert!(!kvs.key_exists("example1").unwrap());
    }

    #[test]
    fn test_remove_key_not_found() {
        let kvs = get_kvs(
            Box::new(MockBackend),
            KvsMap::from([
                ("example1".to_string(), KvsValue::from("value")),
                ("example2".to_string(), KvsValue::from(true)),
            ]),
            KvsMap::new(),
        );

        assert!(kvs
            .remove_key("invalid_key")
            .is_err_and(|e| e == ErrorCode::KeyNotFound));
    }

    #[test]
    fn test_flush() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let backend = Box::new(JsonBackendBuilder::new().working_dir(dir_path).build());
        let kvs = get_kvs(
            backend.clone(),
            KvsMap::from([("key".to_string(), KvsValue::from("value"))]),
            KvsMap::new(),
        );

        kvs.flush().unwrap();

        // Functions below check if file exist.
        let instance_id = kvs.parameters().instance_id;
        let snapshot_id = SnapshotId(0);
        assert!(backend.kvs_file_path(instance_id, snapshot_id).exists());
        assert!(backend.hash_file_path(instance_id, snapshot_id).exists());
    }

    #[test]
    fn test_flush_snapshot_max_count_zero() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        const MAX_COUNT: usize = 0;
        let kvs = get_kvs(
            Box::new(
                JsonBackendBuilder::new()
                    .working_dir(dir_path)
                    .snapshot_max_count(MAX_COUNT)
                    .build(),
            ),
            KvsMap::new(),
            KvsMap::new(),
        );

        // Flush several times.
        for _ in 0..MAX_COUNT + 1 {
            kvs.flush().unwrap();
        }

        assert_eq!(kvs.snapshot_count(), MAX_COUNT);
    }

    #[test]
    fn test_flush_snapshot_max_count_one() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        const MAX_COUNT: usize = 1;
        let kvs = get_kvs(
            Box::new(
                JsonBackendBuilder::new()
                    .working_dir(dir_path)
                    .snapshot_max_count(MAX_COUNT)
                    .build(),
            ),
            KvsMap::new(),
            KvsMap::new(),
        );

        // Flush several times.
        for _ in 0..MAX_COUNT + 1 {
            kvs.flush().unwrap();
        }

        assert_eq!(kvs.snapshot_count(), MAX_COUNT);
    }

    #[test]
    fn test_flush_snapshot_max_count_default() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        const EXPECTED_MAX_COUNT: usize = 3;
        let kvs = get_kvs(
            Box::new(JsonBackendBuilder::new().working_dir(dir_path).build()),
            KvsMap::new(),
            KvsMap::new(),
        );

        // Flush several times.
        for _ in 0..EXPECTED_MAX_COUNT + 1 {
            kvs.flush().unwrap();
        }

        assert_eq!(kvs.snapshot_count(), EXPECTED_MAX_COUNT);
    }

    #[test]
    fn test_snapshot_count_zero() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs(
            Box::new(JsonBackendBuilder::new().working_dir(dir_path).build()),
            KvsMap::new(),
            KvsMap::new(),
        );
        assert_eq!(kvs.snapshot_count(), 0);
    }

    #[test]
    fn test_snapshot_count_to_one() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs(
            Box::new(JsonBackendBuilder::new().working_dir(dir_path).build()),
            KvsMap::new(),
            KvsMap::new(),
        );
        kvs.flush().unwrap();
        assert_eq!(kvs.snapshot_count(), 1);
    }

    #[test]
    fn test_snapshot_count_to_max() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs(
            Box::new(JsonBackendBuilder::new().working_dir(dir_path).build()),
            KvsMap::new(),
            KvsMap::new(),
        );
        for i in 1..=kvs.snapshot_max_count() {
            kvs.flush().unwrap();
            assert_eq!(kvs.snapshot_count(), i);
        }
        kvs.flush().unwrap();
        kvs.flush().unwrap();
        assert_eq!(kvs.snapshot_count(), kvs.snapshot_max_count());
    }

    #[test]
    fn test_snapshot_max_count() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs(
            Box::new(JsonBackendBuilder::new().working_dir(dir_path).build()),
            KvsMap::new(),
            KvsMap::new(),
        );
        assert_eq!(kvs.snapshot_max_count(), 3);
    }

    #[test]
    fn test_snapshot_restore_ok() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs(
            Box::new(JsonBackendBuilder::new().working_dir(dir_path).build()),
            KvsMap::new(),
            KvsMap::new(),
        );
        for i in 1..=kvs.snapshot_max_count() {
            kvs.set_value("counter", KvsValue::I32(i as i32)).unwrap();
            kvs.flush().unwrap();
        }

        kvs.snapshot_restore(SnapshotId(1)).unwrap();
        assert_eq!(kvs.get_value_as::<i32>("counter").unwrap(), 2);
    }

    #[test]
    fn test_snapshot_restore_invalid_id() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs(
            Box::new(JsonBackendBuilder::new().working_dir(dir_path).build()),
            KvsMap::new(),
            KvsMap::new(),
        );
        for i in 1..=kvs.snapshot_max_count() {
            kvs.set_value("counter", KvsValue::I32(i as i32)).unwrap();
            kvs.flush().unwrap();
        }

        assert!(kvs
            .snapshot_restore(SnapshotId(123))
            .is_err_and(|e| e == ErrorCode::InvalidSnapshotId));
    }

    #[test]
    fn test_snapshot_restore_current_id() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs(
            Box::new(JsonBackendBuilder::new().working_dir(dir_path).build()),
            KvsMap::new(),
            KvsMap::new(),
        );
        for i in 1..=kvs.snapshot_max_count() {
            kvs.set_value("counter", KvsValue::I32(i as i32)).unwrap();
            kvs.flush().unwrap();
        }

        assert!(kvs
            .snapshot_restore(SnapshotId(0))
            .is_err_and(|e| e == ErrorCode::InvalidSnapshotId));
    }

    #[test]
    fn test_snapshot_restore_not_available() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs(
            Box::new(JsonBackendBuilder::new().working_dir(dir_path).build()),
            KvsMap::new(),
            KvsMap::new(),
        );
        for i in 1..=2 {
            kvs.set_value("counter", KvsValue::I32(i)).unwrap();
            kvs.flush().unwrap();
        }

        assert!(kvs
            .snapshot_restore(SnapshotId(3))
            .is_err_and(|e| e == ErrorCode::InvalidSnapshotId));
    }
}
