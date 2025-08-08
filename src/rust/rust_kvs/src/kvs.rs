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
        let mut kvs = self.kvs.lock()?;

        if !self.default.contains_key(key) {
            eprintln!("error: resetting key without a default value");
            return Err(ErrorCode::KeyDefaultNotFound);
        }

        let _ = kvs.remove(key);
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
mod kvs_tests {
    use crate::error_code::ErrorCode;
    use crate::json_backend::JsonBackend;
    use crate::kvs::{GenericKvs, KVS_MAX_SNAPSHOTS};
    use crate::kvs_api::{InstanceId, KvsApi, OpenNeedDefaults, OpenNeedKvs, SnapshotId};
    use crate::kvs_backend::KvsBackend;
    use crate::kvs_value::{KvsMap, KvsValue};
    use std::path::PathBuf;
    use std::sync::{atomic, Mutex};
    use tempfile::tempdir;

    /// Most tests can be performed with mocked backend.
    /// Only those with file handling must use concrete implementation.
    struct MockBackend;

    impl KvsBackend for MockBackend {
        fn load_kvs(
            _source_path: PathBuf,
            _verify_hash: bool,
            _hash_source: Option<PathBuf>,
        ) -> Result<KvsMap, ErrorCode> {
            Ok(KvsMap::new())
        }

        fn save_kvs(
            _kvs: &KvsMap,
            _destination_path: PathBuf,
            _add_hash: bool,
        ) -> Result<(), ErrorCode> {
            Ok(())
        }
    }

    fn get_kvs<B: KvsBackend>(
        working_dir: PathBuf,
        kvs_map: KvsMap,
        defaults_map: KvsMap,
    ) -> GenericKvs<B> {
        let instance_id = InstanceId(1);
        let mut kvs = GenericKvs::<B>::open(
            instance_id,
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(working_dir.display().to_string()),
        )
        .unwrap();

        kvs.kvs = Mutex::new(kvs_map);
        kvs.default = defaults_map;
        kvs
    }

    #[test]
    fn test_new_ok() {
        // Check only if panic happens.
        get_kvs::<MockBackend>(PathBuf::new(), KvsMap::new(), KvsMap::new());
    }

    #[test]
    fn test_reset() {
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
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
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
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
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
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
        let kvs = get_kvs::<MockBackend>(PathBuf::new(), KvsMap::new(), KvsMap::new());

        let keys = kvs.get_all_keys().unwrap();
        assert_eq!(keys.len(), 0);
    }

    #[test]
    fn test_key_exists_found() {
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
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
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
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
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
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
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
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
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
            KvsMap::from([("example2".to_string(), KvsValue::from(true))]),
            KvsMap::from([("example1".to_string(), KvsValue::from("default_value"))]),
        );

        assert!(kvs
            .get_value("invalid_key")
            .is_err_and(|e| e == ErrorCode::KeyNotFound));
    }

    #[test]
    fn test_get_value_as_found() {
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
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
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
            KvsMap::from([("example2".to_string(), KvsValue::from(true))]),
            KvsMap::from([("example1".to_string(), KvsValue::from("default_value"))]),
        );

        let value = kvs.get_value_as::<String>("example1").unwrap();
        assert_eq!(value, "default_value");
    }

    #[test]
    fn test_get_value_as_not_found() {
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
            KvsMap::from([("example2".to_string(), KvsValue::from(true))]),
            KvsMap::from([("example1".to_string(), KvsValue::from("default_value"))]),
        );

        assert!(kvs
            .get_value_as::<String>("invalid_key")
            .is_err_and(|e| e == ErrorCode::KeyNotFound));
    }

    #[test]
    fn test_get_value_as_invalid_type() {
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
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
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
            KvsMap::from([("example2".to_string(), KvsValue::from(true))]),
            KvsMap::from([("example1".to_string(), KvsValue::from("default_value"))]),
        );

        assert!(kvs
            .get_value_as::<f64>("example1")
            .is_err_and(|e| e == ErrorCode::ConversionFailed));
    }

    #[test]
    fn test_get_default_value_found() {
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
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
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
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
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
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
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
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
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
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
        let kvs = get_kvs::<MockBackend>(PathBuf::new(), KvsMap::new(), KvsMap::new());

        kvs.set_value("key", "value").unwrap();
        assert_eq!(kvs.get_value_as::<String>("key").unwrap(), "value");
    }

    #[test]
    fn test_set_value_exists() {
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
            KvsMap::from([("key".to_string(), KvsValue::from("old_value"))]),
            KvsMap::new(),
        );

        kvs.set_value("key", "new_value").unwrap();
        assert_eq!(kvs.get_value_as::<String>("key").unwrap(), "new_value");
    }

    #[test]
    fn test_remove_key_found() {
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
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
        let kvs = get_kvs::<MockBackend>(
            PathBuf::new(),
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
        let kvs = get_kvs::<MockBackend>(PathBuf::new(), KvsMap::new(), KvsMap::new());

        assert!(kvs.flush_on_exit.load(atomic::Ordering::Relaxed));
        kvs.flush_on_exit(false);
        assert!(!kvs.flush_on_exit.load(atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_flush() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs::<JsonBackend>(
            dir_path,
            KvsMap::from([("key".to_string(), KvsValue::from("value"))]),
            KvsMap::new(),
        );

        kvs.flush().unwrap();
        let snapshot_id = SnapshotId(0);
        // Functions below check if file exist.
        kvs.get_kvs_filename(snapshot_id.clone()).unwrap();
        kvs.get_hash_filename(snapshot_id).unwrap();
    }

    #[test]
    fn test_snapshot_count_zero() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs::<JsonBackend>(dir_path, KvsMap::new(), KvsMap::new());
        assert_eq!(kvs.snapshot_count(), 0);
    }

    #[test]
    fn test_snapshot_count_to_one() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs::<JsonBackend>(dir_path, KvsMap::new(), KvsMap::new());
        kvs.flush().unwrap();
        assert_eq!(kvs.snapshot_count(), 1);
    }

    #[test]
    fn test_snapshot_count_to_max() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs::<JsonBackend>(dir_path, KvsMap::new(), KvsMap::new());
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
        let kvs = get_kvs::<JsonBackend>(dir_path, KvsMap::new(), KvsMap::new());
        for i in 1..=KVS_MAX_SNAPSHOTS {
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
        let kvs = get_kvs::<JsonBackend>(dir_path, KvsMap::new(), KvsMap::new());
        for i in 1..=KVS_MAX_SNAPSHOTS {
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
        let kvs = get_kvs::<JsonBackend>(dir_path, KvsMap::new(), KvsMap::new());
        for i in 1..=KVS_MAX_SNAPSHOTS {
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
        let kvs = get_kvs::<JsonBackend>(dir_path, KvsMap::new(), KvsMap::new());
        for i in 1..=2 {
            kvs.set_value("counter", KvsValue::I32(i)).unwrap();
            kvs.flush().unwrap();
        }

        assert!(kvs
            .snapshot_restore(SnapshotId(3))
            .is_err_and(|e| e == ErrorCode::InvalidSnapshotId));
    }

    #[test]
    fn test_get_kvs_file_path_found() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs::<JsonBackend>(dir_path, KvsMap::new(), KvsMap::new());

        kvs.flush().unwrap();
        kvs.flush().unwrap();
        let kvs_path = kvs.get_kvs_filename(SnapshotId(1)).unwrap();
        let kvs_name = kvs_path.file_name().unwrap().to_str().unwrap();
        assert_eq!(kvs_name, "kvs_1_1.json");
    }

    #[test]
    fn test_get_kvs_file_path_not_found() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs::<JsonBackend>(dir_path, KvsMap::new(), KvsMap::new());

        assert!(kvs
            .get_kvs_filename(SnapshotId(1))
            .is_err_and(|e| e == ErrorCode::FileNotFound));
    }

    #[test]
    fn test_get_hash_file_path_found() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs::<JsonBackend>(dir_path, KvsMap::new(), KvsMap::new());

        kvs.flush().unwrap();
        kvs.flush().unwrap();
        let hash_path = kvs.get_hash_filename(SnapshotId(1)).unwrap();
        let hash_name = hash_path.file_name().unwrap().to_str().unwrap();
        assert_eq!(hash_name, "kvs_1_1.hash");
    }

    #[test]
    fn test_get_hash_file_path_not_found() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs = get_kvs::<JsonBackend>(dir_path, KvsMap::new(), KvsMap::new());

        assert!(kvs
            .get_hash_filename(SnapshotId(1))
            .is_err_and(|e| e == ErrorCode::FileNotFound));
    }

    #[test]
    fn test_drop() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        {
            let kvs = get_kvs::<JsonBackend>(dir_path.clone(), KvsMap::new(), KvsMap::new());
            kvs.flush_on_exit(true);
            kvs.set_value("key", "value").unwrap();
        }

        let kvs_path = dir_path.join(format!("kvs_{}_{}.json", InstanceId(1), SnapshotId(0)));
        assert!(kvs_path.exists());
        let hash_path = dir_path.join(format!("kvs_{}_{}.hash", InstanceId(1), SnapshotId(0)));
        assert!(hash_path.exists());
    }
}
