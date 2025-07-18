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

//std dependencies
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{self, AtomicBool};
use std::sync::Mutex;

use crate::error_code::ErrorCode;
use crate::kvs_api::{InstanceId, KvsApi, SnapshotId};
use crate::kvs_api::{OpenNeedDefaults, OpenNeedKvs};
use crate::kvs_backend::KvsBackend;
use crate::kvs_value::{KvsMap, KvsValue};

/// Maximum number of snapshots
///
/// Feature: `FEAT_REQ__KVS__snapshots`
const KVS_MAX_SNAPSHOTS: usize = 3;

/// Key-value-storage data
pub struct GenericKvs<J: KvsBackend> {
    /// Storage data
    ///
    /// Feature: `FEAT_REQ__KVS__thread_safety` (Mutex)
    kvs: Mutex<KvsMap>,

    /// Optional default values
    ///
    /// Feature: `FEAT_REQ__KVS__default_values`
    default: KvsMap,

    /// Filename prefix
    filename_prefix: PathBuf,

    /// Flush on exit flag
    flush_on_exit: AtomicBool,

    _backend: std::marker::PhantomData<J>,
}

/// Need-File flag
#[derive(PartialEq)]
enum OpenKvsNeedFile {
    /// Optional: If the file doesn't exist, start with empty data
    Optional,

    /// Required: The file must already exist
    Required,
}

impl From<OpenNeedDefaults> for OpenKvsNeedFile {
    fn from(val: OpenNeedDefaults) -> OpenKvsNeedFile {
        match val {
            OpenNeedDefaults::Optional => OpenKvsNeedFile::Optional,
            OpenNeedDefaults::Required => OpenKvsNeedFile::Required,
        }
    }
}

impl From<OpenNeedKvs> for OpenKvsNeedFile {
    fn from(val: OpenNeedKvs) -> OpenKvsNeedFile {
        match val {
            OpenNeedKvs::Optional => OpenKvsNeedFile::Optional,
            OpenNeedKvs::Required => OpenKvsNeedFile::Required,
        }
    }
}

/// Verify-Hash flag
#[derive(PartialEq)]
enum OpenKvsVerifyHash {
    /// No: Parse the file without the hash
    No,

    /// Yes: Parse the file with the hash
    Yes,
}

impl<J: KvsBackend> GenericKvs<J> {
    /// Open and parse a JSON file
    ///
    /// Return an empty hash when no file was found.
    ///
    /// # Features
    ///   * `FEAT_REQ__KVS__integrity_check`
    ///
    /// # Parameters
    ///   * `need_file`: fail if file doesn't exist
    ///   * `verify_hash`: content is verified against a hash file
    ///
    /// # Return Values
    ///   * Ok: KVS data as `HashMap<String, KvsValue>`
    ///   * `ErrorCode::ValidationFailed`: KVS hash validation failed
    ///   * `ErrorCode::JsonParserError`: JSON parser error (invalid JSON or type error)
    ///   * `ErrorCode::KvsFileReadError`: KVS file read error (I/O error)
    ///   * `ErrorCode::KvsHashFileReadError`: KVS hash file read error (hash file missing or unreadable)
    ///   * `ErrorCode::UnmappedError`: Generic error
    fn open_kvs<T>(
        filename: &PathBuf,
        need_file: T,
        verify_hash: OpenKvsVerifyHash,
        hash_filename: Option<&PathBuf>,
    ) -> Result<KvsMap, ErrorCode>
    where
        T: Into<OpenKvsNeedFile>,
    {
        let do_hash = matches!(verify_hash, OpenKvsVerifyHash::Yes);
        let filename_path = filename.clone();
        let hash_filename_path = hash_filename.cloned();
        match J::load_kvs(filename_path.clone(), do_hash, hash_filename_path.clone()) {
            Ok(_) => {
                let map = J::load_kvs(filename_path, do_hash, hash_filename_path).map_err(|e| {
                    eprintln!("error: {e:?}");
                    e
                })?;
                Ok(map)
            }
            Err(e) => {
                if need_file.into() == OpenKvsNeedFile::Required {
                    eprintln!("error: file {filename:?} could not be read: {e:?}");
                    Err(e)
                } else {
                    println!("file {filename:?} not found, using empty data");
                    Ok(KvsMap::new())
                }
            }
        }
    }

    /// Rotate snapshots
    ///
    /// # Features
    ///   * `FEAT_REQ__KVS__snapshots`
    ///
    /// # Return Values
    ///   * Ok: Rotation successful, also if no rotation was needed
    ///   * `ErrorCode::UnmappedError`: Unmapped error
    fn snapshot_rotate(&self) -> Result<(), ErrorCode> {
        for idx in (1..=KVS_MAX_SNAPSHOTS).rev() {
            let hash_old = format!("{}_{}.hash", self.filename_prefix.display(), idx - 1);
            let hash_new = format!("{}_{}.hash", self.filename_prefix.display(), idx);
            let snap_old = format!("{}_{}.json", self.filename_prefix.display(), idx - 1);
            let snap_new = format!("{}_{}.json", self.filename_prefix.display(), idx);

            println!("rotating: {snap_old} -> {snap_new}");

            let res = fs::rename(hash_old, hash_new);
            if let Err(err) = res {
                if err.kind() != std::io::ErrorKind::NotFound {
                    return Err(err.into());
                } else {
                    continue;
                }
            }

            let res = fs::rename(snap_old, snap_new);
            if let Err(err) = res {
                return Err(err.into());
            }
        }

        Ok(())
    }
}

impl<J: KvsBackend> KvsApi for GenericKvs<J> {
    /// Open the key-value-storage
    ///
    /// Checks and opens a key-value-storage. Flush on exit is enabled by default and can be
    /// controlled with [`flush_on_exit`](Self::flush_on_exit).
    ///
    /// # Features
    ///   * `FEAT_REQ__KVS__default_values`
    ///   * `FEAT_REQ__KVS__multiple_kvs`
    ///   * `FEAT_REQ__KVS__integrity_check`
    ///
    /// # Parameters
    ///   * `instance_id`: Instance ID
    ///   * `need_defaults`: Fail when no default file was found
    ///   * `need_kvs`: Fail when no KVS file was found
    ///
    /// # Return Values
    ///   * Ok: KVS instance
    ///   * `ErrorCode::ValidationFailed`: KVS hash validation failed
    ///   * `ErrorCode::JsonParserError`: JSON parser error (invalid JSON or type error)
    ///   * `ErrorCode::KvsFileReadError`: KVS file read error (I/O error)
    ///   * `ErrorCode::KvsHashFileReadError`: KVS hash file read error (hash file missing or unreadable)
    ///   * `ErrorCode::UnmappedError`: Generic error
    fn open(
        instance_id: InstanceId,
        need_defaults: OpenNeedDefaults,
        need_kvs: OpenNeedKvs,
        dir: Option<String>,
    ) -> Result<GenericKvs<J>, ErrorCode> {
        let dir = if let Some(dir) = dir {
            format!("{dir}/")
        } else {
            "".to_string()
        };
        let filename_default = PathBuf::from(format!("{dir}kvs_{instance_id}_default"));
        let filename_prefix = PathBuf::from(format!("{dir}kvs_{instance_id}"));
        let filename_kvs =
            filename_prefix.with_file_name(format!("{}_0", filename_prefix.display()));

        let default = GenericKvs::<J>::open_kvs(
            &filename_default,
            need_defaults,
            OpenKvsVerifyHash::No,
            None,
        )?;
        // Use hash checking for the main KVS file
        let hash_path =
            filename_prefix.with_file_name(format!("{}_0.hash", filename_prefix.display()));
        let kvs = GenericKvs::<J>::open_kvs(
            &filename_kvs,
            need_kvs,
            OpenKvsVerifyHash::Yes,
            Some(&hash_path),
        )?;

        println!("opened KVS: instance '{instance_id}'");
        println!("max snapshot count: {KVS_MAX_SNAPSHOTS}");

        Ok(GenericKvs {
            kvs: Mutex::new(kvs),
            default,
            filename_prefix,
            flush_on_exit: AtomicBool::new(true),
            _backend: std::marker::PhantomData,
        })
    }

    /// Control the flush on exit behaviour
    ///
    /// # Parameters
    ///   * `flush_on_exit`: Flag to control flush-on-exit behaviour
    fn flush_on_exit(&self, flush_on_exit: bool) {
        self.flush_on_exit
            .store(flush_on_exit, atomic::Ordering::Relaxed);
    }

    /// Resets a key-value-storage to its initial state
    ///
    /// # Return Values
    ///   * Ok: Reset of the KVS was successful
    ///   * `ErrorCode::MutexLockFailed`: Mutex locking failed
    fn reset(&self) -> Result<(), ErrorCode> {
        *self.kvs.lock()? = HashMap::new();
        Ok(())
    }

    /// Get list of all keys
    ///
    /// # Return Values
    ///   * Ok: List of all keys
    ///   * `ErrorCode::MutexLockFailed`: Mutex locking failed
    fn get_all_keys(&self) -> Result<Vec<String>, ErrorCode> {
        Ok(self.kvs.lock()?.keys().map(|x| x.to_string()).collect())
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
        Ok(self.kvs.lock()?.contains_key(key))
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
        let kvs = self.kvs.lock()?;

        if let Some(value) = kvs.get(key) {
            Ok(value.clone())
        } else if let Some(value) = self.default.get(key) {
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
        let kvs = self.kvs.lock()?;

        if let Some(value) = kvs.get(key) {
            match T::try_from(value) {
                Ok(value) => Ok(value),
                Err(err) => {
                    eprintln!(
                        "error: get_value could not convert KvsValue from KVS store: {err:#?}"
                    );
                    Err(ErrorCode::ConversionFailed)
                }
            }
        } else if let Some(value) = self.default.get(key) {
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
        if let Some(value) = self.default.get(key) {
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
        if self.kvs.lock()?.contains_key(key) {
            Ok(false)
        } else if self.default.contains_key(key) {
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
        self.kvs.lock()?.insert(key.into(), value.into());
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
        if self.kvs.lock()?.remove(key).is_some() {
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
        self.snapshot_rotate().map_err(|e| {
            eprintln!("error: snapshot_rotate failed: {e:?}");
            e
        })?;
        let kvs = self.kvs.lock().map_err(|e| {
            eprintln!("error: Mutex lock failed: {e:?}");
            ErrorCode::MutexLockFailed
        })?;
        J::save_kvs(&kvs, self.filename_prefix.clone(), true).map_err(|e| {
            eprintln!("error: save_kvs failed: {e:?}");
            e
        })?;
        Ok(())
    }

    /// Get the count of snapshots
    ///
    /// # Return Values
    ///   * usize: Count of found snapshots
    fn snapshot_count(&self) -> usize {
        let mut count = 0;

        for idx in 0..KVS_MAX_SNAPSHOTS {
            let snapshot_path = self.filename_prefix.with_file_name(format!(
                "{}_{}.json",
                self.filename_prefix.display(),
                idx
            ));
            if !snapshot_path.exists() {
                break;
            }

            count += 1;
        }

        count
    }

    /// Return maximum snapshot count
    ///
    /// # Return Values
    ///   * usize: Maximum count of snapshots
    fn snapshot_max_count() -> usize {
        KVS_MAX_SNAPSHOTS
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
    fn snapshot_restore(&self, id: SnapshotId) -> Result<(), ErrorCode> {
        // fail if the snapshot ID is the current KVS
        if id.0 == 0 {
            eprintln!("error: tried to restore current KVS as snapshot");
            return Err(ErrorCode::InvalidSnapshotId);
        }

        if self.snapshot_count() < id.0 {
            eprintln!("error: tried to restore a non-existing snapshot");
            return Err(ErrorCode::InvalidSnapshotId);
        }

        let snap_path = PathBuf::from(format!("{}_{}", self.filename_prefix.display(), id.0));
        let kvs = Self::open_kvs(
            &snap_path,
            OpenKvsNeedFile::Required,
            OpenKvsVerifyHash::Yes,
            None,
        )?;
        *self.kvs.lock()? = kvs;

        Ok(())
    }

    /// Return the KVS-filename for a given snapshot ID
    ///
    /// # Parameters
    ///   * `id`: Snapshot ID to get the filename for
    ///
    /// # Return Values
    ///   * `Ok`: Filename for ID
    ///   * `ErrorCode::FileNotFound`: KVS file for snapshot ID not found
    fn get_kvs_filename(&self, id: SnapshotId) -> Result<PathBuf, ErrorCode> {
        let path = self.filename_prefix.with_file_name(format!(
            "{}_{}.json",
            self.filename_prefix.display(),
            id
        ));
        if !path.exists() {
            Err(ErrorCode::FileNotFound)
        } else {
            Ok(path)
        }
    }

    /// Return the hash-filename for a given snapshot ID
    ///
    /// # Parameters
    ///   * `id`: Snapshot ID to get the hash filename for
    ///
    /// # Return Values
    ///   * `Ok`: Hash filename for ID
    ///   * `ErrorCode::FileNotFound`: Hash file for snapshot ID not found
    fn get_hash_filename(&self, id: SnapshotId) -> Result<PathBuf, ErrorCode> {
        let path = self.filename_prefix.with_file_name(format!(
            "{}_{}.hash",
            self.filename_prefix.display(),
            id
        ));
        if !path.exists() {
            Err(ErrorCode::FileNotFound)
        } else {
            Ok(path)
        }
    }
}

impl<J: KvsBackend> Drop for GenericKvs<J> {
    fn drop(&mut self) {
        if self.flush_on_exit.load(atomic::Ordering::Relaxed) {
            if let Err(e) = self.flush() {
                eprintln!("GenericKvs::flush() failed in Drop: {e:?}");
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::Kvs;
    use tempfile::tempdir;

    mod mock_backend {
        use super::*;
        use crate::kvs_backend::KvsBackend;

        #[derive(Default, Clone)]
        pub struct KvsMockBackend {}

        impl KvsBackend for KvsMockBackend {
            fn load_kvs(
                source_path: PathBuf,
                _verify_hash: bool,
                _hash_source: Option<PathBuf>,
            ) -> Result<KvsMap, ErrorCode> {
                let mut map = KvsMap::new();
                let fname = source_path.display().to_string();
                if fname.ends_with("default") {
                    map.insert("mock_default_key".to_string(), KvsValue::from(111.0));
                } else {
                    map.insert("mock_key".to_string(), KvsValue::from(123.0));
                }
                Ok(map)
            }

            fn save_kvs(
                _kvs: &KvsMap,
                _destination_path: PathBuf,
                _add_hash: bool,
            ) -> Result<(), ErrorCode> {
                Ok(())
            }
        }

        #[derive(Default, Clone)]
        pub struct KvsMockBackendFail;

        impl KvsBackend for KvsMockBackendFail {
            fn load_kvs(
                _source_path: PathBuf,
                _verify_hash: bool,
                _hash_source: Option<PathBuf>,
            ) -> Result<KvsMap, ErrorCode> {
                Err(ErrorCode::UnmappedError)
            }

            fn save_kvs(
                _kvs: &KvsMap,
                _destination_path: PathBuf,
                _add_hash: bool,
            ) -> Result<(), ErrorCode> {
                Err(ErrorCode::UnmappedError)
            }
        }
    }

    use mock_backend::KvsMockBackend;
    use mock_backend::KvsMockBackendFail;

    fn new_kvs_with_mock() -> GenericKvs<KvsMockBackend> {
        let instance_id = InstanceId::new(100);
        // Directory is not used by mock backend
        GenericKvs::<KvsMockBackend>::open(
            instance_id,
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            None,
        )
        .unwrap()
    }

    fn new_kvs_with_mock_required() -> GenericKvs<KvsMockBackend> {
        let instance_id = InstanceId::new(101);
        GenericKvs::<KvsMockBackend>::open(
            instance_id,
            OpenNeedDefaults::Required,
            OpenNeedKvs::Required,
            None,
        )
        .unwrap()
    }

    fn new_kvs_with_mock_required_fail() -> Result<GenericKvs<KvsMockBackendFail>, ErrorCode> {
        let instance_id = InstanceId::new(102);
        GenericKvs::<KvsMockBackendFail>::open(
            instance_id,
            OpenNeedDefaults::Required,
            OpenNeedKvs::Required,
            None,
        )
    }

    #[test]
    fn test_open_kvs_with_mock_required_success() {
        let kvs = new_kvs_with_mock_required();
        // Should contain the mock_key from mock backend
        assert!(kvs.key_exists("mock_key").unwrap());
        let value = kvs.get_value("mock_key").unwrap();
        assert_eq!(*value.get::<f64>().unwrap(), 123.0);
    }

    #[test]
    fn test_open_kvs_with_mock_required_fail() {
        let res = new_kvs_with_mock_required_fail();
        assert!(res.is_err());
        // Should be ErrorCode::UnmappedError
        assert!(matches!(res, Err(ErrorCode::UnmappedError)));
    }

    #[test]
    fn test_open_kvs_with_mock_backend() {
        let kvs = new_kvs_with_mock();
        // Should contain the mock_key from mock backend
        assert!(kvs.key_exists("mock_key").unwrap());
        let value = kvs.get_value("mock_key").unwrap();
        assert_eq!(*value.get::<f64>().unwrap(), 123.0);
    }

    #[test]
    fn test_set_and_get_value() {
        let kvs = new_kvs_with_mock();
        kvs.set_value("foo", 42.0).unwrap();
        let value = kvs.get_value("foo").unwrap();
        assert_eq!(*value.get::<f64>().unwrap(), 42.0);
    }

    #[test]
    fn test_get_value_as() {
        let kvs = new_kvs_with_mock();
        kvs.set_value("bar", 99.0).unwrap();
        let v: f64 = kvs.get_value_as("bar").unwrap();
        assert_eq!(v, 99.0);
    }

    #[test]
    fn test_get_default_value_and_is_value_default() {
        let kvs = new_kvs_with_mock_required();
        println!("{:?}", kvs.default);
        // mock_key is always present as default in mock backend
        assert!(kvs.is_value_default("mock_default_key").unwrap());
        let def = kvs.get_default_value("mock_default_key").unwrap();
        assert_eq!(*def.get::<f64>().unwrap(), 111.0);

        // mock_key is always present as default in mock backend
        assert!(!kvs.is_value_default("mock_key").unwrap());
    }

    #[test]
    fn test_key_exists_and_get_all_keys() {
        let kvs = new_kvs_with_mock();
        assert!(kvs.key_exists("mock_key").unwrap());
        kvs.set_value("foo", true).unwrap();
        assert!(kvs.key_exists("foo").unwrap());
        let keys = kvs.get_all_keys().unwrap();
        assert!(keys.contains(&"foo".to_string()));
        assert!(keys.contains(&"mock_key".to_string()));
    }

    #[test]
    fn test_remove_key() {
        let kvs = new_kvs_with_mock();
        kvs.set_value("baz", 1.0).unwrap();
        assert!(kvs.key_exists("baz").unwrap());
        kvs.remove_key("baz").unwrap();
        assert!(!kvs.key_exists("baz").unwrap());
    }

    #[test]
    fn test_reset() {
        let kvs = new_kvs_with_mock();
        kvs.set_value("reset_me", 5.0).unwrap();
        assert!(kvs.key_exists("reset_me").unwrap());
        kvs.reset().unwrap();
        assert!(!kvs.key_exists("reset_me").unwrap());
    }

    #[test]
    fn test_flush_with_mock_backend() {
        let kvs = new_kvs_with_mock();
        // Should not error, even though nothing is written
        kvs.flush().unwrap();
    }

    #[test]
    fn test_snapshot_count_and_max_count() {
        let kvs = new_kvs_with_mock();
        // No real snapshots, so count should be 0
        assert_eq!(kvs.snapshot_count(), 0);
        assert_eq!(GenericKvs::<KvsMockBackend>::snapshot_max_count(), 3);
    }

    #[test]
    fn test_snapshot_restore_invalid_id() {
        let kvs = new_kvs_with_mock();
        // id 1 is always invalid in this mock (no real snapshots)
        let res = kvs.snapshot_restore(SnapshotId::new(1));
        assert!(res.is_err());
    }

    #[test]
    fn test_get_kvs_filename_and_hash_filename() {
        let kvs = new_kvs_with_mock();
        // No files exist, so should return FileNotFound
        let res = kvs.get_kvs_filename(SnapshotId::new(1));
        assert!(matches!(res, Err(ErrorCode::FileNotFound)));
        let res = kvs.get_hash_filename(SnapshotId::new(1));
        assert!(matches!(res, Err(ErrorCode::FileNotFound)));
    }

    #[test]
    fn test_get_value_error_cases() {
        let kvs = new_kvs_with_mock();
        // Key does not exist
        let res = kvs.get_value("not_found");
        assert!(matches!(res, Err(ErrorCode::KeyNotFound)));
    }

    #[test]
    fn test_get_value_as_conversion_error() {
        let kvs = new_kvs_with_mock();
        kvs.set_value("str_key", "string".to_string()).unwrap();
        // Try to get as f64, should fail
        let res: Result<f64, _> = kvs.get_value_as("str_key");
        assert!(matches!(res, Err(ErrorCode::ConversionFailed)));
    }

    #[test]
    fn test_is_value_default_error() {
        let kvs = new_kvs_with_mock();
        let res = kvs.is_value_default("not_found");
        assert!(matches!(res, Err(ErrorCode::KeyNotFound)));
    }

    #[test]
    fn test_kvs_open_and_set_get_value() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(42);
        let kvs = Kvs::open(
            instance_id.clone(),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_path.clone()),
        )
        .unwrap();
        let _ = kvs.set_value("direct", KvsValue::String("abc".to_string()));
        let value = kvs.get_value("direct").unwrap();
        assert_eq!(value.get::<String>().unwrap(), "abc");
    }

    #[test]
    fn test_kvs_reset() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(43);
        let kvs = Kvs::open(
            instance_id.clone(),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_path.clone()),
        )
        .unwrap();
        let _ = kvs.set_value("reset", 1.0);
        assert!(kvs.get_value("reset").is_ok());
        kvs.reset().unwrap();
        assert!(matches!(
            kvs.get_value("reset"),
            Err(ErrorCode::KeyNotFound)
        ));
    }

    #[test]
    fn test_kvs_key_exists_and_get_all_keys() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(44);
        let kvs = Kvs::open(
            instance_id.clone(),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_path.clone()),
        )
        .unwrap();
        assert!(!kvs.key_exists("foo").unwrap());
        let _ = kvs.set_value("foo", KvsValue::Boolean(true));
        assert!(kvs.key_exists("foo").unwrap());
        let keys = kvs.get_all_keys().unwrap();
        assert!(keys.contains(&"foo".to_string()));
    }

    #[test]
    fn test_kvs_remove_key() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(45);
        let kvs = Kvs::open(
            instance_id.clone(),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_path.clone()),
        )
        .unwrap();
        let _ = kvs.set_value("bar", 2.0);
        assert!(kvs.key_exists("bar").unwrap());
        kvs.remove_key("bar").unwrap();
        assert!(!kvs.key_exists("bar").unwrap());
    }

    #[test]
    fn test_kvs_flush_and_snapshot() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(46);
        let kvs = Kvs::open(
            instance_id.clone(),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_path.clone()),
        )
        .unwrap();
        let _ = kvs.set_value("snap", 3.0);
        // Before flush, snapshot count should be 0 (no snapshots yet)
        assert_eq!(kvs.snapshot_count(), 0);
        kvs.flush().unwrap();
        // After flush, snapshot count should be 1
        assert_eq!(kvs.snapshot_count(), 1);
        // Call flush again to rotate and create a snapshot
        kvs.flush().unwrap();
        assert_eq!(kvs.snapshot_count(), 2);
        // Restore from snapshot if available
        if kvs.snapshot_count() > 0 {
            kvs.snapshot_restore(SnapshotId::new(1)).unwrap();
        }
    }

    #[test]
    fn test_drop() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);

        let kvs = Kvs::open(
            instance_id.clone(),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_path.clone()),
        )
        .unwrap();

        kvs.flush_on_exit(false);
        // Drop is called automatically, but we can check that flush_on_exit is set to false
        assert!(
            !kvs.flush_on_exit.load(std::sync::atomic::Ordering::Relaxed),
            "Expected flush_on_exit to be false"
        );
    }
}
