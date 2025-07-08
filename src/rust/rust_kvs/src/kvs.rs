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

//core and alloc libs
use core::fmt;

//std dependencies
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{self, AtomicBool};
use std::sync::Mutex;

//external crates
use adler32::RollingAdler32;
use tinyjson::{JsonGenerator, JsonValue};

use crate::error_code::ErrorCode;
use crate::kvs_api::KvsApi;
use crate::kvs_value::KvsValue;

/// Maximum number of snapshots
///
/// Feature: `FEAT_REQ__KVS__snapshots`
const KVS_MAX_SNAPSHOTS: usize = 3;

/// Instance ID
#[derive(Clone, Debug, PartialEq)]
pub struct InstanceId(usize);

/// Snapshot ID
#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotId(usize);

/// Key-value-storage data
pub struct Kvs {
    /// Storage data
    ///
    /// Feature: `FEAT_REQ__KVS__thread_safety` (Mutex)
    kvs: Mutex<HashMap<String, KvsValue>>,

    /// Optional default values
    ///
    /// Feature: `FEAT_REQ__KVS__default_values`
    default: HashMap<String, KvsValue>,

    /// Filename prefix
    filename_prefix: String,

    /// Flush on exit flag
    flush_on_exit: AtomicBool,
}

/// Need-Defaults flag
pub enum OpenNeedDefaults {
    /// Optional: Open defaults only if available
    Optional,

    /// Required: Defaults must be available
    Required,
}

/// Need-KVS flag
pub enum OpenNeedKvs {
    /// Optional: Use an empty KVS if no KVS is available
    Optional,

    /// Required: KVS must be already exist
    Required,
}

/// Need-File flag
#[derive(PartialEq)]
enum OpenJsonNeedFile {
    /// Optional: If the file doesn't exist, start with empty data
    Optional,

    /// Required: The file must already exist
    Required,
}

impl From<bool> for OpenNeedDefaults {
    fn from(flag: bool) -> OpenNeedDefaults {
        if flag {
            OpenNeedDefaults::Required
        } else {
            OpenNeedDefaults::Optional
        }
    }
}

impl From<bool> for OpenNeedKvs {
    fn from(flag: bool) -> OpenNeedKvs {
        if flag {
            OpenNeedKvs::Required
        } else {
            OpenNeedKvs::Optional
        }
    }
}

impl From<OpenNeedDefaults> for OpenJsonNeedFile {
    fn from(val: OpenNeedDefaults) -> OpenJsonNeedFile {
        match val {
            OpenNeedDefaults::Optional => OpenJsonNeedFile::Optional,
            OpenNeedDefaults::Required => OpenJsonNeedFile::Required,
        }
    }
}

impl From<OpenNeedKvs> for OpenJsonNeedFile {
    fn from(val: OpenNeedKvs) -> OpenJsonNeedFile {
        match val {
            OpenNeedKvs::Optional => OpenJsonNeedFile::Optional,
            OpenNeedKvs::Required => OpenJsonNeedFile::Required,
        }
    }
}

/// Verify-Hash flag
#[derive(PartialEq)]
enum OpenJsonVerifyHash {
    /// No: Parse the file without the hash
    No,

    /// Yes: Parse the file with the hash
    Yes,
}

impl fmt::Display for InstanceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for SnapshotId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl InstanceId {
    /// Create a new instance ID
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

impl SnapshotId {
    /// Create a new Snapshot ID
    pub fn new(id: usize) -> Self {
        SnapshotId(id)
    }
}

impl Kvs {
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
    ///   * `Ok`: KVS data as `HashMap<String, KvsValue>`
    ///   * `ErrorCode::ValidationFailed`: KVS hash validation failed
    ///   * `ErrorCode::JsonParserError`: JSON parser error
    ///   * `ErrorCode::KvsFileReadError`: KVS file read error
    ///   * `ErrorCode::KvsHashFileReadError`: KVS hash file read error
    ///   * `ErrorCode::UnmappedError`: Generic error
    fn open_json<T>(
        filename_prefix: &str,
        need_file: T,
        verify_hash: OpenJsonVerifyHash,
    ) -> Result<HashMap<String, KvsValue>, ErrorCode>
    where
        T: Into<OpenJsonNeedFile>,
    {
        let filename_json = format!("{filename_prefix}.json");
        let filename_hash = format!("{filename_prefix}.hash");
        match fs::read_to_string(&filename_json) {
            Ok(data) => {
                if verify_hash == OpenJsonVerifyHash::Yes {
                    // data exists, read hash file
                    match fs::read(&filename_hash) {
                        Ok(hash) => {
                            let hash_kvs = RollingAdler32::from_buffer(data.as_bytes()).hash();
                            if u32::from_be_bytes(hash.try_into()?) != hash_kvs {
                                eprintln!(
                                    "error: KVS data corrupted ({filename_json}, {filename_hash})"
                                );
                                Err(ErrorCode::ValidationFailed)
                            } else {
                                println!("JSON data has valid hash");
                                let data: JsonValue = data.parse()?;
                                println!("parsing file {filename_json}");
                                Ok(data
                                    .get::<HashMap<_, _>>()
                                    .ok_or(ErrorCode::JsonParserError)?
                                    .iter()
                                    .map(|(key, value)| (key.clone(), value.into()))
                                    .collect())
                            }
                        }
                        Err(err) => {
                            eprintln!(
                                "error: hash file {filename_hash} could not be read: {err:#?}"
                            );
                            Err(ErrorCode::KvsHashFileReadError)
                        }
                    }
                } else {
                    Ok(data
                        .parse::<JsonValue>()?
                        .get::<HashMap<_, _>>()
                        .ok_or(ErrorCode::JsonParserError)?
                        .iter()
                        .map(|(key, value)| (key.clone(), value.into()))
                        .collect())
                }
            }
            Err(err) => {
                if need_file.into() == OpenJsonNeedFile::Required {
                    eprintln!("error: file {filename_json} could not be read: {err:#?}");
                    Err(ErrorCode::KvsFileReadError)
                } else {
                    println!("file {filename_json} not found, using empty data");
                    Ok(HashMap::new())
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
            let hash_old = format!("{}_{}.hash", self.filename_prefix, idx - 1);
            let hash_new = format!("{}_{}.hash", self.filename_prefix, idx);
            let snap_old = format!("{}_{}.json", self.filename_prefix, idx - 1);
            let snap_new = format!("{}_{}.json", self.filename_prefix, idx);

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

impl KvsApi for Kvs {
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
    ///   * `ErrorCode::JsonParserError`: JSON parser error
    ///   * `ErrorCode::KvsFileReadError`: KVS file read error
    ///   * `ErrorCode::KvsHashFileReadError`: KVS hash file read error
    ///   * `ErrorCode::UnmappedError`: Generic error
    fn open(
        instance_id: InstanceId,
        need_defaults: OpenNeedDefaults,
        need_kvs: OpenNeedKvs,
        dir: Option<String>,
    ) -> Result<Kvs, ErrorCode> {
        let dir = if let Some(dir) = dir {
            format!("{dir}/")
        } else {
            "".to_string()
        };
        let filename_default = format!("{dir}kvs_{instance_id}_default");
        let filename_prefix = format!("{dir}kvs_{instance_id}");
        let filename_kvs = format!("{filename_prefix}_0");

        let default = Self::open_json(&filename_default, need_defaults, OpenJsonVerifyHash::No)?;
        let kvs = Self::open_json(&filename_kvs, need_kvs, OpenJsonVerifyHash::Yes)?;

        println!("opened KVS: instance '{instance_id}'");
        println!("max snapshot count: {KVS_MAX_SNAPSHOTS}");

        Ok(Self {
            kvs: Mutex::new(kvs),
            default,
            filename_prefix,
            flush_on_exit: AtomicBool::new(true),
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

    /// Reset a key-value pair in the storage to its default state
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

        if self.default.get(key).is_none() {
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
    fn set_value<S: Into<String>, J: Into<KvsValue>>(
        &self,
        key: S,
        value: J,
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
        let json: HashMap<String, JsonValue> = self
            .kvs
            .lock()?
            .iter()
            .map(|(key, value)| (key.clone(), value.into()))
            .collect();
        let json = JsonValue::from(json);
        let mut buf = Vec::new();
        let mut gen = JsonGenerator::new(&mut buf).indent("  ");
        gen.generate(&json)?;

        self.snapshot_rotate()?;

        let hash = RollingAdler32::from_buffer(&buf).hash();

        let filename_json = format!("{}_0.json", self.filename_prefix);
        let data = String::from_utf8(buf)?;
        fs::write(filename_json, &data)?;

        let filename_hash = format!("{}_0.hash", self.filename_prefix);
        fs::write(filename_hash, hash.to_be_bytes()).ok();

        Ok(())
    }

    /// Get the count of snapshots
    ///
    /// # Return Values
    ///   * usize: Count of found snapshots
    fn snapshot_count(&self) -> usize {
        let mut count = 0;

        for idx in 0..KVS_MAX_SNAPSHOTS {
            let snapshot_path = PathBuf::from(format!("{}_{}.json", self.filename_prefix, idx));
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

        let kvs = Self::open_json(
            &format!("{}_{}", self.filename_prefix, id.0),
            OpenJsonNeedFile::Required,
            OpenJsonVerifyHash::Yes,
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
        let path = PathBuf::from(format!("{}_{}.json", self.filename_prefix, id));
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
        let path = PathBuf::from(format!("{}_{}.hash", self.filename_prefix, id));
        if !path.exists() {
            Err(ErrorCode::FileNotFound)
        } else {
            Ok(path)
        }
    }
}

impl Drop for Kvs {
    fn drop(&mut self) {
        if self.flush_on_exit.load(atomic::Ordering::Relaxed) {
            self.flush().ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kvs_builder::KvsBuilder;
    use tempfile::tempdir;

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

    impl<'a> TryFrom<&'a KvsValue> for u64 {
        type Error = ErrorCode;

        fn try_from(_val: &'a KvsValue) -> Result<u64, Self::Error> {
            Err(ErrorCode::ConversionFailed)
        }
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
        let _ = kvs.set_value("reset", KvsValue::Number(1.0));
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
        let _ = kvs.set_value("bar", KvsValue::Number(2.0));
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
        let _ = kvs.set_value("snap", KvsValue::Number(3.0));
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

    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_kvs_reset_single() {
        let instance_id = InstanceId::new(0);
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        std::fs::copy(
            "tests/kvs_0_default.json",
            format!("{}/kvs_0_default.json", dir_path.clone()),
        )
        .unwrap();
        let kvs = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path.clone())
            .need_defaults(true)
            .build()
            .unwrap();

        let _ = kvs.set_value("number1", KvsValue::Number(987f64));
        let _ = kvs.reset_key("number1");
        assert_eq!(kvs.get_value_as::<f64>("number1").unwrap(), 987f64);

        let _ = kvs.set_value("string1", KvsValue::String("Testing".to_string()));
        let _ = kvs.reset_key("string1");
        assert_eq!(kvs.get_value_as::<String>("string1").unwrap(), "Hello");

        let _ = kvs.set_value("bool", KvsValue::Boolean(true));
        assert!(matches!(
            kvs.reset_key("bool"),
            Err(ErrorCode::KeyDefaultNotFound)
        ));

        let _ = kvs.reset_key("bool1");
        assert_eq!(kvs.get_value_as::<bool>("bool1").unwrap(), false);

        assert!(matches!(
            kvs.reset_key("fail"),
            Err(ErrorCode::KeyDefaultNotFound)
        ));
    }
}
