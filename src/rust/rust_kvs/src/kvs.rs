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
use crate::kvs_api::{FlushOnExit, KvsApi, SnapshotId};
use crate::kvs_backend::KvsBackend;
use crate::kvs_provider::GenericKvsInner;
use crate::kvs_value::{KvsMap, KvsValue};
use std::sync::{Arc, Mutex};

/// Key-value-storage data
pub struct GenericKvs<Backend: KvsBackend> {
    /// Inner storage data representation.
    kvs_inner: Arc<Mutex<GenericKvsInner>>,

    /// Backend object.
    backend: Backend,
}

impl<Backend: KvsBackend> GenericKvs<Backend> {
    pub(crate) fn new(kvs_inner: Arc<Mutex<GenericKvsInner>>, backend: Backend) -> Self {
        Self { kvs_inner, backend }
    }

    pub fn backend(&self) -> &Backend {
        &self.backend
    }
}

impl<Backend: KvsBackend> KvsApi for GenericKvs<Backend> {
    /// Get current flush on exit behavior.
    ///
    /// # Return Values
    ///    * Ok: Flush on exit behavior was successful
    ///    * `ErrorCode::MutexLockFailed`: Mutex locking failed
    fn flush_on_exit(&self) -> Result<FlushOnExit, ErrorCode> {
        let kvs_inner = self.kvs_inner.lock()?;
        Ok(kvs_inner.flush_on_exit.clone())
    }

    /// Control the flush on exit behavior
    ///
    /// # Parameters
    ///   * `flush_on_exit`: Flag to control flush-on-exit behavior
    ///
    /// # Return Values
    ///    * Ok: Changing flush on exit behavior was successful
    ///    * `ErrorCode::MutexLockFailed`: Mutex locking failed
    fn set_flush_on_exit(&self, flush_on_exit: FlushOnExit) -> Result<(), ErrorCode> {
        let mut kvs_inner = self.kvs_inner.lock()?;
        kvs_inner.flush_on_exit = flush_on_exit;
        Ok(())
    }

    /// Resets a key-value-storage to its initial state
    ///
    /// # Return Values
    ///   * Ok: Reset of the KVS was successful
    ///   * `ErrorCode::MutexLockFailed`: Mutex locking failed
    fn reset(&self) -> Result<(), ErrorCode> {
        let mut kvs_inner = self.kvs_inner.lock()?;
        kvs_inner.kvs_map = KvsMap::new();
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
        let mut kvs_inner = self.kvs_inner.lock()?;
        if !kvs_inner.defaults_map.contains_key(key) {
            eprintln!("error: resetting key without a default value");
            return Err(ErrorCode::KeyDefaultNotFound);
        }

        let _ = kvs_inner.kvs_map.remove(key);
        Ok(())
    }

    /// Get list of all keys
    ///
    /// # Return Values
    ///   * Ok: List of all keys
    ///   * `ErrorCode::MutexLockFailed`: Mutex locking failed
    fn get_all_keys(&self) -> Result<Vec<String>, ErrorCode> {
        let kvs_inner = self.kvs_inner.lock()?;
        Ok(kvs_inner.kvs_map.keys().map(|x| x.to_string()).collect())
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
        let kvs_inner = self.kvs_inner.lock()?;
        Ok(kvs_inner.kvs_map.contains_key(key))
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
        let kvs_inner = self.kvs_inner.lock()?;
        if let Some(value) = kvs_inner.kvs_map.get(key) {
            Ok(value.clone())
        } else if let Some(value) = kvs_inner.defaults_map.get(key) {
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
        let kvs_inner = self.kvs_inner.lock()?;
        if let Some(value) = kvs_inner.kvs_map.get(key) {
            match T::try_from(value) {
                Ok(value) => Ok(value),
                Err(err) => {
                    eprintln!(
                        "error: get_value could not convert KvsValue from KVS store: {err:#?}"
                    );
                    Err(ErrorCode::ConversionFailed)
                }
            }
        } else if let Some(value) = kvs_inner.defaults_map.get(key) {
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
        let kvs_inner = self.kvs_inner.lock()?;
        if let Some(value) = kvs_inner.defaults_map.get(key) {
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
        let kvs_inner = self.kvs_inner.lock()?;
        if kvs_inner.kvs_map.contains_key(key) {
            Ok(false)
        } else if kvs_inner.defaults_map.contains_key(key) {
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
        let mut kvs_inner = self.kvs_inner.lock()?;
        kvs_inner.kvs_map.insert(key.into(), value.into());
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
        let mut kvs_inner = self.kvs_inner.lock()?;
        if kvs_inner.kvs_map.remove(key).is_some() {
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
        let kvs_inner = self.kvs_inner.lock()?;
        self.backend.flush(&kvs_inner.kvs_map)
    }

    /// Get the count of snapshots
    ///
    /// # Return Values
    ///   * usize: Count of found snapshots
    fn snapshot_count(&self) -> usize {
        self.backend.snapshot_count()
    }

    /// Return maximum snapshot count
    ///
    /// # Return Values
    ///   * usize: Maximum count of snapshots
    fn snapshot_max_count() -> usize {
        Backend::snapshot_max_count()
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
    fn snapshot_restore(&self, snapshot_id: &SnapshotId) -> Result<(), ErrorCode> {
        let mut kvs_inner = self.kvs_inner.lock()?;
        kvs_inner.kvs_map = self.backend.snapshot_restore(snapshot_id)?;
        Ok(())
    }
}

impl<Backend: KvsBackend> Drop for GenericKvs<Backend> {
    fn drop(&mut self) {
        if self.flush_on_exit().unwrap() == FlushOnExit::Yes {
            if let Err(e) = self.flush() {
                eprintln!("GenericKvs::flush() failed in Drop: {e:?}");
            }
        }
    }
}

#[cfg(test)]
mod kvs_tests {
    // TODO: add lock tests.
    // TODO: check function comments.
    // TODO: check coverage of unit tests.

    use crate::error_code::ErrorCode;
    use crate::json_backend::JsonBackend;
    use crate::kvs::GenericKvs;
    use crate::kvs_api::{FlushOnExit, InstanceId, KvsApi, SnapshotId};
    use crate::kvs_backend::{KvsBackend, KvsPathResolver};
    use crate::kvs_provider::GenericKvsInner;
    use crate::kvs_value::KvsMap;
    use crate::prelude::KvsValue;
    use crate::KVS_MAX_SNAPSHOTS;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;

    /// Most tests can be performed with mocked backend.
    /// Only those with file handling must use concrete implementation.
    struct MockBackend;

    impl KvsBackend for MockBackend {
        fn load_kvs(&self, _snapshot_id: &SnapshotId) -> Result<KvsMap, ErrorCode> {
            unimplemented!()
        }

        fn load_defaults(&self) -> Result<KvsMap, ErrorCode> {
            unimplemented!()
        }

        fn flush(&self, _kvs_map: &KvsMap) -> Result<(), ErrorCode> {
            unimplemented!()
        }

        fn snapshot_count(&self) -> usize {
            unimplemented!()
        }

        fn snapshot_max_count() -> usize {
            KVS_MAX_SNAPSHOTS
        }

        fn snapshot_restore(&self, _snapshot_id: &SnapshotId) -> Result<KvsMap, ErrorCode> {
            unimplemented!()
        }
    }

    impl KvsPathResolver for MockBackend {
        fn new(_instance_id: InstanceId, _working_dir: &Path) -> Self {
            Self
        }

        fn kvs_file_name(&self, _snapshot_id: &SnapshotId) -> String {
            unimplemented!()
        }

        fn kvs_file_path(&self, _snapshot_id: &SnapshotId) -> PathBuf {
            unimplemented!()
        }

        fn hash_file_name(&self, _snapshot_id: &SnapshotId) -> String {
            unimplemented!()
        }

        fn hash_file_path(&self, _snapshot_id: &SnapshotId) -> PathBuf {
            unimplemented!()
        }

        fn defaults_file_name(&self) -> String {
            unimplemented!()
        }

        fn defaults_file_path(&self) -> PathBuf {
            unimplemented!()
        }
    }

    fn get_kvs<B: KvsBackend>(backend: B, kvs_map: KvsMap, defaults_map: KvsMap) -> GenericKvs<B> {
        let kvs_inner = Arc::new(Mutex::new(GenericKvsInner {
            kvs_map,
            defaults_map,
            flush_on_exit: FlushOnExit::No,
        }));
        GenericKvs::<B>::new(kvs_inner, backend)
    }

    #[test]
    fn test_new() {
        get_kvs(
            MockBackend::new(InstanceId(1), &PathBuf::new()),
            KvsMap::new(),
            KvsMap::new(),
        );
    }

    #[test]
    fn test_reset() {
        let kvs = get_kvs(
            MockBackend::new(InstanceId(1), &PathBuf::new()),
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
            MockBackend::new(InstanceId(1), &PathBuf::new()),
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
            MockBackend::new(InstanceId(1), &PathBuf::new()),
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
        let kvs = get_kvs(
            MockBackend::new(InstanceId(1), &PathBuf::new()),
            KvsMap::new(),
            KvsMap::new(),
        );

        let keys = kvs.get_all_keys().unwrap();
        assert_eq!(keys.len(), 0);
    }

    #[test]
    fn test_key_exists_found() {
        let kvs = get_kvs(
            MockBackend::new(InstanceId(1), &PathBuf::new()),
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
            MockBackend::new(InstanceId(1), &PathBuf::new()),
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
            MockBackend::new(InstanceId(1), &PathBuf::new()),
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
            MockBackend::new(InstanceId(1), &PathBuf::new()),
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
            MockBackend::new(InstanceId(1), &PathBuf::new()),
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
            MockBackend::new(InstanceId(1), &PathBuf::new()),
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
            MockBackend::new(InstanceId(1), &PathBuf::new()),
            KvsMap::from([("example2".to_string(), KvsValue::from(true))]),
            KvsMap::from([("example1".to_string(), KvsValue::from("default_value"))]),
        );

        let value = kvs.get_value_as::<String>("example1").unwrap();
        assert_eq!(value, "default_value");
    }

    #[test]
    fn test_get_value_as_not_found() {
        let kvs = get_kvs(
            MockBackend::new(InstanceId(1), &PathBuf::new()),
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
            MockBackend::new(InstanceId(1), &PathBuf::new()),
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
            MockBackend::new(InstanceId(1), &PathBuf::new()),
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
            MockBackend::new(InstanceId(1), &PathBuf::new()),
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
            MockBackend::new(InstanceId(1), &PathBuf::new()),
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
            MockBackend::new(InstanceId(1), &PathBuf::new()),
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
            MockBackend::new(InstanceId(1), &PathBuf::new()),
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
            MockBackend::new(InstanceId(1), &PathBuf::new()),
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
        let kvs = get_kvs(
            MockBackend::new(InstanceId(1), &PathBuf::new()),
            KvsMap::new(),
            KvsMap::new(),
        );

        kvs.set_value("key", "value").unwrap();
        assert_eq!(kvs.get_value_as::<String>("key").unwrap(), "value");
    }

    #[test]
    fn test_set_value_exists() {
        let kvs = get_kvs(
            MockBackend::new(InstanceId(1), &PathBuf::new()),
            KvsMap::from([("key".to_string(), KvsValue::from("old_value"))]),
            KvsMap::new(),
        );

        kvs.set_value("key", "new_value").unwrap();
        assert_eq!(kvs.get_value_as::<String>("key").unwrap(), "new_value");
    }

    #[test]
    fn test_remove_key_found() {
        let kvs = get_kvs(
            MockBackend::new(InstanceId(1), &PathBuf::new()),
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
            MockBackend::new(InstanceId(1), &PathBuf::new()),
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
    fn test_flush_on_exit() {
        let kvs = get_kvs(
            MockBackend::new(InstanceId(1), &PathBuf::new()),
            KvsMap::new(),
            KvsMap::new(),
        );

        assert_eq!(kvs.flush_on_exit().unwrap(), FlushOnExit::No);
    }

    #[test]
    fn test_set_flush_on_exit() {
        let kvs = get_kvs(
            MockBackend::new(InstanceId(1), &PathBuf::new()),
            KvsMap::new(),
            KvsMap::new(),
        );

        kvs.set_flush_on_exit(FlushOnExit::Yes).unwrap();
        assert_eq!(kvs.flush_on_exit().unwrap(), FlushOnExit::Yes);
        kvs.set_flush_on_exit(FlushOnExit::No).unwrap();
        assert_eq!(kvs.flush_on_exit().unwrap(), FlushOnExit::No);
    }

    #[test]
    fn test_flush() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs(
            JsonBackend::new(InstanceId(1), &dir_path),
            KvsMap::from([("key".to_string(), KvsValue::from("value"))]),
            KvsMap::new(),
        );

        kvs.flush().unwrap();

        // Check files exist.
        let snapshot_id = SnapshotId(0);
        assert!(kvs.backend().kvs_file_path(&snapshot_id).exists());
        assert!(kvs.backend().hash_file_path(&snapshot_id).exists());
    }

    #[test]
    fn test_snapshot_count_zero() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs(
            JsonBackend::new(InstanceId(1), &dir_path),
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
            JsonBackend::new(InstanceId(1), &dir_path),
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
            JsonBackend::new(InstanceId(1), &dir_path),
            KvsMap::new(),
            KvsMap::new(),
        );
        for i in 1..=KVS_MAX_SNAPSHOTS {
            kvs.flush().unwrap();
            assert_eq!(kvs.snapshot_count(), i);
        }
        kvs.flush().unwrap();
        kvs.flush().unwrap();
        assert_eq!(kvs.snapshot_count(), KVS_MAX_SNAPSHOTS);
    }

    #[test]
    fn test_snapshot_max_count() {
        assert_eq!(
            GenericKvs::<MockBackend>::snapshot_max_count(),
            KVS_MAX_SNAPSHOTS
        );
    }

    #[test]
    fn test_snapshot_restore_ok() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs(
            JsonBackend::new(InstanceId(1), &dir_path),
            KvsMap::new(),
            KvsMap::new(),
        );
        for i in 1..=KVS_MAX_SNAPSHOTS {
            kvs.set_value("counter", KvsValue::I32(i as i32)).unwrap();
            kvs.flush().unwrap();
        }

        kvs.snapshot_restore(&SnapshotId(1)).unwrap();
        assert_eq!(kvs.get_value_as::<i32>("counter").unwrap(), 2);
    }

    #[test]
    fn test_snapshot_restore_invalid_id() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs(
            JsonBackend::new(InstanceId(1), &dir_path),
            KvsMap::new(),
            KvsMap::new(),
        );
        for i in 1..=KVS_MAX_SNAPSHOTS {
            kvs.set_value("counter", KvsValue::I32(i as i32)).unwrap();
            kvs.flush().unwrap();
        }

        assert!(kvs
            .snapshot_restore(&SnapshotId(123))
            .is_err_and(|e| e == ErrorCode::InvalidSnapshotId));
    }

    #[test]
    fn test_snapshot_restore_current_id() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs(
            JsonBackend::new(InstanceId(1), &dir_path),
            KvsMap::new(),
            KvsMap::new(),
        );
        for i in 1..=KVS_MAX_SNAPSHOTS {
            kvs.set_value("counter", KvsValue::I32(i as i32)).unwrap();
            kvs.flush().unwrap();
        }

        assert!(kvs
            .snapshot_restore(&SnapshotId(0))
            .is_err_and(|e| e == ErrorCode::InvalidSnapshotId));
    }

    #[test]
    fn test_snapshot_restore_not_available() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs(
            JsonBackend::new(InstanceId(1), &dir_path),
            KvsMap::new(),
            KvsMap::new(),
        );
        for i in 1..=2 {
            kvs.set_value("counter", KvsValue::I32(i)).unwrap();
            kvs.flush().unwrap();
        }

        assert!(kvs
            .snapshot_restore(&SnapshotId(3))
            .is_err_and(|e| e == ErrorCode::InvalidSnapshotId));
    }

    #[test]
    fn test_drop() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        {
            let kvs = get_kvs(
                JsonBackend::new(InstanceId(1), &dir_path),
                KvsMap::new(),
                KvsMap::new(),
            );
            kvs.set_flush_on_exit(FlushOnExit::Yes).unwrap();
            kvs.set_value("key", "value").unwrap();
        }

        let backend = JsonBackend::new(InstanceId(1), &dir_path);
        let kvs_path = backend.kvs_file_path(&SnapshotId(0));
        assert!(kvs_path.exists());
        let hash_path = backend.hash_file_path(&SnapshotId(0));
        assert!(hash_path.exists());
    }
}
