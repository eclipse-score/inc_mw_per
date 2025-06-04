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

//! # Key-Value-Storage API and Implementation
//!
//! ## Introduction
//!
//! This crate provides a Key-Value-Store using [TinyJSON](https://crates.io/crates/tinyjson) to
//! persist the data. To validate the stored data a hash is build and verified using the
//! [Adler32](https://crates.io/crates/adler32) crate. No other direct dependencies are used
//! besides the Rust `std` library.
//!
//! The key-value-storage is opened or initialized with [`KvsBuilder::new`] where various settings
//! can be applied before the KVS instance is created.
//!
//! Without configuration the KVS is flushed on exit by default. This can be controlled by
//! [`Kvs::flush_on_exit`]. It is possible to manually flush the KVS by calling [`Kvs::flush`].
//!
//! All `TinyJSON` provided datatypes can be used:
//!   * `Number`: `f64`
//!   * `Boolean`: `bool`
//!   * `String`: `String`
//!   * `Null`: `()`
//!   * `Array`: `Vec<KvsValue>`
//!   * `Object`: `HashMap<String, KvsValue>`
//!
//! Note: JSON arrays are not restricted to only contain values of the same type.
//!
//! Writing a value to the KVS can be done by calling [`Kvs::set_value`] with the `key` as first
//! and a `KvsValue` as second parameter. Either `KvsValue::Number(123.0)` or `123.0` can be
//! used as there will be an auto-Into performed when calling the function.
//!
//! To read a `KvsValue` value call [`Kvs::get_value`](Kvs::get_value) with the `key` as first
//! parameter.
//!
//! If a `key` isn't available in the KVS a lookup into the defaults storage will be performed and
//! if the `value` is found the default will be returned. The default value isn't stored when
//! [`Kvs::flush`] is called unless it's explicitly written with [`Kvs::set_value`]. So when
//! defaults change always the latest values will be returned. If that is an unwanted behaviour
//! it's better to remove the default value and write the value permanently when the KVS is
//! initialized. To check whether a value has a default call [`Kvs::get_default_value`] and to
//! see if the value contains the default call
//! [`Kvs::has_default_value`].
//!
//!
//! ## Example Usage
//!
//! ```
//! use rust_kvs::{ErrorCode, InstanceId, KvsBuilder, KvsValue};
//! use std::collections::HashMap;
//! use tempdir::TempDir;
//!
//! fn main() -> Result<(), ErrorCode> {
//!     let temp_dir = TempDir::new("").unwrap();
//!     let temp_path = temp_dir.path().display().to_string();
//!     let kvs = KvsBuilder::new(InstanceId::new(0)).dir(temp_path).build()?;
//!
//!     kvs.set_value("number", 123.0)?;
//!     kvs.set_value("bool", true)?;
//!     kvs.set_value("string", "First".to_string())?;
//!     kvs.set_value("null", ())?;
//!     kvs.set_value(
//!         "array",
//!         vec![
//!             KvsValue::from(456.0),
//!             false.into(),
//!             "Second".to_string().into(),
//!         ],
//!     )?;
//!     kvs.set_value(
//!         "object",
//!         HashMap::from([
//!             (String::from("sub-number"), KvsValue::from(789.0)),
//!             ("sub-bool".into(), true.into()),
//!             ("sub-string".into(), "Third".to_string().into()),
//!             ("sub-null".into(), ().into()),
//!             (
//!                 "sub-array".into(),
//!                 KvsValue::from(vec![
//!                     KvsValue::from(1246.0),
//!                     false.into(),
//!                     "Fourth".to_string().into(),
//!                 ]),
//!             ),
//!         ]),
//!     )?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Feature Coverage
//!
//! Feature and requirement definition:
//!   * [Features/Persistency/Key-Value-Storage](https://github.com/eclipse-score/score/blob/ulhu_persistency_kvs/docs/features/persistency/key-value-storage/index.rst#specification)
//!   * [Requirements/Stakeholder](https://github.com/eclipse-score/score/blob/ulhu_persistency_kvs/docs/requirements/stakeholder/index.rst)
//!
//! Supported features and requirements:
//!   * `FEAT_REQ__KVS__thread_safety`
//!   * `FEAT_REQ__KVS__supported_datatypes_keys`
//!   * `FEAT_REQ__KVS__supported_datatypes_values`
//!   * `FEAT_REQ__KVS__default_values`
//!   * `FEAT_REQ__KVS__update_mechanism`: JSON format-flexibility
//!   * `FEAT_REQ__KVS__snapshots`
//!   * `FEAT_REQ__KVS__default_value_reset`
//!   * `FEAT_REQ__KVS__default_value_retrieval`
//!   * `FEAT_REQ__KVS__persistency`
//!   * `FEAT_REQ__KVS__integrity_check`
//!   * `STKH_REQ__30`: JSON storage format
//!   * `STKH_REQ__8`: Defaults stored in JSON format
//!   * `STKH_REQ__12`: Support storing data on non-volatile memory
//!   * `STKH_REQ__13`: POSIX portability
//!
//! Currently unsupported features:
//!   * `FEAT_REQ__KVS__maximum_size`
//!   * `FEAT_REQ__KVS__cpp_rust_interoperability`
//!   * `FEAT_REQ__KVS__versioning`: JSON version ID
//!   * `FEAT_REQ__KVS__tooling`: Get/set CLI, JSON editor
//!   * `STKH_REQ__350`: Safe key-value-store
//!
//! Additional info:
//!   * Feature `FEAT_REQ__KVS__supported_datatypes_keys` is matched by the Rust standard which
//!     defines that `String` and `str` are always valid UTF-8.
//!   * Feature `FEAT_REQ__KVS__supported_datatypes_values` is matched by using the same types that
//!     the IPC will use for the Rust implementation.
#![forbid(unsafe_code)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use adler32::RollingAdler32;
use std::array::TryFromSliceError;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::ops::Index;
use std::path::Path;
use std::string::FromUtf8Error;
use std::sync::{
    atomic::{self, AtomicBool},
    Mutex, MutexGuard, PoisonError,
};
use tinyjson::{JsonGenerateError, JsonGenerator, JsonParseError, JsonValue};

/// Maximum number of snapshots
///
/// Feature: `FEAT_REQ__KVS__snapshots`
const KVS_MAX_SNAPSHOTS: usize = 3;

/// Instance ID
#[derive(Clone, Debug, PartialEq)]
pub struct InstanceId(usize);

/// Snapshot ID
#[derive(PartialEq, Debug)]
pub struct SnapshotId(usize);

/// Runtime Error Codes
#[derive(Debug, PartialEq)]
pub enum ErrorCode {
    /// Error that was not yet mapped
    UnmappedError,

    /// File not found
    FileNotFound,

    /// KVS file read error
    KvsFileReadError,

    /// KVS hash file read error
    KvsHashFileReadError,

    /// JSON parser error
    JsonParserError,

    /// JSON generator error
    JsonGeneratorError,

    /// Physical storage failure
    PhysicalStorageFailure,

    /// Integrity corrupted
    IntegrityCorrupted,

    /// Validation failed
    ValidationFailed,

    /// Encryption failed
    EncryptionFailed,

    /// Resource is busy
    ResourceBusy,

    /// Out of storage space
    OutOfStorageSpace,

    /// Quota exceeded
    QuotaExceeded,

    /// Authentication failed
    AuthenticationFailed,

    /// Key not found
    KeyNotFound,

    /// Key default value not found
    KeyDefaultNotFound,

    /// Serialization failed
    SerializationFailed,

    /// Invalid snapshot ID
    InvalidSnapshotId,

    /// Conversion failed
    ConversionFailed,

    /// Mutex failed
    MutexLockFailed,

    /// Invalid value type
    InvalidValueType,
}

/// Key-value-storage builder
pub struct KvsBuilder {
    /// Instance ID
    instance_id: InstanceId,

    /// Need-defaults flag
    need_defaults: bool,

    /// Need-KVS flag
    need_kvs: bool,

    /// Working directory
    dir: Option<String>,
}

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

/// Key-value-storage value
#[derive(Clone, Debug, PartialEq)]
pub enum KvsValue {
    /// Number
    Number(f64),

    /// Boolean
    Boolean(bool),

    /// String
    String(String),

    /// Null
    Null,

    /// Array
    Array(Vec<KvsValue>),

    /// Object
    Object(HashMap<String, KvsValue>),
}

/// Need-Defaults flag
enum OpenNeedDefaults {
    /// Optional: Open defaults only if available
    Optional,

    /// Required: Defaults must be available
    Required,
}

/// Need-KVS flag
enum OpenNeedKvs {
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

impl From<std::io::Error> for ErrorCode {
    fn from(cause: std::io::Error) -> Self {
        let kind = cause.kind();
        match kind {
            std::io::ErrorKind::NotFound => ErrorCode::FileNotFound,
            _ => {
                eprintln!("error: unmapped error: {kind}");
                ErrorCode::UnmappedError
            }
        }
    }
}

impl From<JsonParseError> for ErrorCode {
    fn from(cause: JsonParseError) -> Self {
        eprintln!(
            "error: JSON parser error: line = {}, column = {}",
            cause.line(),
            cause.column()
        );
        ErrorCode::JsonParserError
    }
}

impl From<JsonGenerateError> for ErrorCode {
    fn from(cause: JsonGenerateError) -> Self {
        eprintln!("error: JSON generator error: msg = {}", cause.message());
        ErrorCode::JsonGeneratorError
    }
}

impl From<FromUtf8Error> for ErrorCode {
    fn from(cause: FromUtf8Error) -> Self {
        eprintln!("error: UTF-8 conversion failed: {cause:#?}");
        ErrorCode::ConversionFailed
    }
}

impl From<TryFromSliceError> for ErrorCode {
    fn from(cause: TryFromSliceError) -> Self {
        eprintln!("error: try_into from slice failed: {cause:#?}");
        ErrorCode::ConversionFailed
    }
}

impl From<Vec<u8>> for ErrorCode {
    fn from(cause: Vec<u8>) -> Self {
        eprintln!("error: try_into from u8 vector failed: {cause:#?}");
        ErrorCode::ConversionFailed
    }
}

impl From<PoisonError<MutexGuard<'_, HashMap<std::string::String, KvsValue>>>> for ErrorCode {
    fn from(cause: PoisonError<MutexGuard<'_, HashMap<std::string::String, KvsValue>>>) -> Self {
        eprintln!("error: Mutex locking failed: {cause:#?}");
        ErrorCode::MutexLockFailed
    }
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

impl From<usize> for SnapshotId {
    fn from(val: usize) -> SnapshotId {
        SnapshotId(val)
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

impl KvsBuilder {
    /// Create a builder to open the key-value-storage
    ///
    /// Only the instance ID must be set. All other settings are using default values until changed
    /// via the builder API.
    ///
    /// # Parameters
    ///   * `instance_id`: Instance ID
    ///
    /// # Return Values
    ///   * KvsBuilder instance
    pub fn new(instance_id: InstanceId) -> Self {
        Self {
            instance_id,
            need_defaults: false,
            need_kvs: false,
            dir: None,
        }
    }

    /// Configure if defaults must exist when opening the KVS
    ///
    /// # Parameters
    ///   * `flag`: Yes = `true`, no = `false` (default)
    ///
    /// # Return Values
    ///   * KvsBuilder instance
    pub fn need_defaults(mut self, flag: bool) -> KvsBuilder {
        self.need_defaults = flag;
        self
    }

    /// Configure if KVS must exist when opening the KVS
    ///
    /// # Parameters
    ///   * `flag`: Yes = `true`, no = `false` (default)
    ///
    /// # Return Values
    ///   * KvsBuilder instance
    pub fn need_kvs(mut self, flag: bool) -> KvsBuilder {
        self.need_kvs = flag;
        self
    }

    /// Set the key-value-storage permanent storage directory
    ///
    /// # Parameters
    ///   * `dir`: Path to permanent storage
    ///
    /// # Return Values
    pub fn dir<P: Into<String>>(mut self, dir: P) -> KvsBuilder {
        self.dir = Some(dir.into());
        self
    }

    /// Finalize the builder and open the key-value-storage
    ///
    /// Calls `Kvs::open` with the configured settings.
    ///
    /// # Features
    ///   * `FEAT_REQ__KVS__default_values`
    ///   * `FEAT_REQ__KVS__multiple_kvs`
    ///   * `FEAT_REQ__KVS__integrity_check`
    ///
    /// # Return Values
    ///   * Ok: KVS instance
    ///   * `ErrorCode::ValidationFailed`: KVS hash validation failed
    ///   * `ErrorCode::JsonParserError`: JSON parser error
    ///   * `ErrorCode::KvsFileReadError`: KVS file read error
    ///   * `ErrorCode::KvsHashFileReadError`: KVS hash file read error
    ///   * `ErrorCode::UnmappedError`: Generic error
    pub fn build(self) -> Result<Kvs, ErrorCode> {
        Kvs::open(
            self.instance_id,
            self.need_defaults.into(),
            self.need_kvs.into(),
            self.dir,
        )
    }
}

impl Kvs {
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

        let default = Self::open_json(&filename_default, need_defaults)?;
        let kvs = Self::open_json(&filename_kvs, need_kvs)?;

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
    pub fn flush_on_exit(&self, flush_on_exit: bool) {
        self.flush_on_exit
            .store(flush_on_exit, atomic::Ordering::Relaxed);
    }

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
    ) -> Result<HashMap<String, KvsValue>, ErrorCode>
    where
        T: Into<OpenJsonNeedFile>,
    {
        let filename_json = format!("{filename_prefix}.json");
        let filename_hash = format!("{filename_prefix}.hash");
        match fs::read_to_string(&filename_json) {
            Ok(data) => {
                let hash_kvs = RollingAdler32::from_buffer(data.as_bytes()).hash();

                // data exists, read hash file
                match fs::read(&filename_hash) {
                    Ok(hash) => {
                        let hash_calc = u32::from_be_bytes(hash.try_into()?);
                        if hash_calc != hash_kvs {
                            eprintln!(
                                "error: KVS data corrupted {filename_json}: calculated = {hash_calc:#x}, stored = {hash_kvs:#x}"
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
                                .map(|(key, value)| Ok((key.clone(), value.try_into()?)))
                                .collect::<Result<HashMap<String, KvsValue>, ErrorCode>>()?)
                        }
                    }
                    Err(err) => {
                        eprintln!("error: hash file {filename_hash} could not be read: {err:#?}");
                        eprintln!("error: required hash file content: {hash_kvs:#x}");
                        Err(ErrorCode::KvsHashFileReadError)
                    }
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

    /// Resets a key-value-storage to its initial state
    ///
    /// # Return Values
    ///   * Ok: Reset of the KVS was successful
    ///   * `ErrorCode::MutexLockFailed`: Mutex locking failed
    pub fn reset(&self) -> Result<(), ErrorCode> {
        *self.kvs.lock()? = HashMap::new();
        Ok(())
    }

    /// Get list of all keys
    ///
    /// # Return Values
    ///   * Ok: List of all keys
    ///   * `ErrorCode::MutexLockFailed`: Mutex locking failed
    pub fn get_all_keys(&self) -> Result<Vec<String>, ErrorCode> {
        Ok(self.kvs.lock()?.keys().map(|x| x.to_string()).collect())
    }

    /// Reports if a manually stored value exists for a given key in the KVS
    ///
    /// If the key was never written it will always return false even if a default value for the
    /// key is available.
    ///
    /// # Parameters
    ///   * `key`: Key to check for existence
    ///
    /// # Return Values
    ///   * Ok(`true`): Key exists
    ///   * Ok(`false`): Key doesn't exist
    ///   * `ErrorCode::MutexLockFailed`: Mutex locking failed
    pub fn key_exists(&self, key: &str) -> Result<bool, ErrorCode> {
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
    ///   * `ErrorCode::ConversionFailed`: Type conversion failed
    ///   * `ErrorCode::KeyNotFound`: Key wasn't found in KVS nor in defaults
    pub fn get_value(&self, key: &str) -> Result<KvsValue, ErrorCode> {
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
    pub fn get_default_value(&self, key: &str) -> Result<KvsValue, ErrorCode> {
        if let Some(value) = self.default.get(key) {
            Ok(value.clone())
        } else {
            Err(ErrorCode::KeyNotFound)
        }
    }

    /// Return if the value is equal to its default
    ///
    /// If no default value for a given key is available or the current key value doesn't match the
    /// default the function will return `false`. Otherwise it will return `true`.
    ///
    /// # Features
    ///   * `FEAT_REQ__KVS__default_values`
    ///
    /// # Parameters
    ///   * `key`: Key to check if a default exists
    ///
    /// # Return Values
    ///   * true: Key contains default value
    ///   * false: Key hasn't a default or value varies from default
    pub fn has_default_value(&self, key: &str) -> bool {
        if let Some(default) = self.default.get(key) {
            if let Ok(kvs) = self.kvs.lock() {
                if let Some(current) = kvs.get(key) {
                    if default == current {
                        // values match => value is default
                        return true;
                    }
                } else {
                    // value isn't set but default found
                    return true;
                }
            }
        }

        false
    }

    /// Set the key value to its default value
    ///
    /// Overwrite the current value of the key with its default value. If the value has a default
    /// but wasn't written yet it will fixate the current default value even if the default changes
    /// later until it is rewritten. If no default exist a `ErrorCode::KeyDefaultNotFound` error is
    /// returned.
    ///
    /// # Parameters
    ///   * `key`: Key to write default value to
    ///
    /// # Return Values
    ///   * Ok: Default value was assigned to key
    ///   * `ErrorCode::MutexLockFailed`: Mutex locking failed
    ///   * `ErrorCode::KeyDefaultNotFound`: Default value not found for key
    pub fn set_default_value<S: Into<String>>(&self, key: S) -> Result<(), ErrorCode> {
        let key = key.into();

        if let Some(default) = self.default.get(&key) {
            self.kvs.lock()?.insert(key, default.clone());
            Ok(())
        } else {
            Err(ErrorCode::KeyDefaultNotFound)
        }
    }

    /// Assign a value to a given key
    ///
    /// # Parameters
    ///   * `key`: Key to set value
    ///   * `value`: Value to be set
    ///   * `key`: Key to write default value to
    ///
    /// # Return Values
    ///   * Ok: Value was assigned to key
    ///   * `ErrorCode::MutexLockFailed`: Mutex locking failed
    pub fn set_value<S: Into<String>, J: Into<KvsValue>>(
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
    pub fn remove_key(&self, key: &str) -> Result<(), ErrorCode> {
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
    pub fn flush(&self) -> Result<(), ErrorCode> {
        let json: HashMap<String, JsonValue> = self
            .kvs
            .lock()?
            .iter()
            .map(|(key, value)| Ok((key.clone(), value.try_into()?)))
            .collect::<Result<HashMap<String, JsonValue>, ErrorCode>>()?;
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
    pub fn snapshot_count(&self) -> usize {
        let mut count = 0;

        for idx in 0..=KVS_MAX_SNAPSHOTS {
            if !Path::new(&format!("{}_{}.json", self.filename_prefix, idx)).exists() {
                break;
            }

            // skip current KVS but make sure it exists before search for snapshots
            if idx == 0 {
                continue;
            }

            count = idx;
        }

        count
    }

    /// Return maximum snapshot count
    ///
    /// # Return Values
    ///   * usize: Maximum count of snapshots
    pub fn snapshot_max_count() -> usize {
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
    pub fn snapshot_restore(&self, id: SnapshotId) -> Result<(), ErrorCode> {
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
        )?;
        *self.kvs.lock()? = kvs;

        Ok(())
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

    /// Return the KVS-filename for a given snapshot ID
    ///
    /// # Parameters
    ///   * `id`: Snapshot ID to get the filename for
    ///
    /// # Return Values
    ///   * String: Filename for ID
    pub fn get_kvs_filename(&self, id: SnapshotId) -> String {
        format!("{}_{}.json", self.filename_prefix, id)
    }

    /// Return the hash-filename for a given snapshot ID
    ///
    /// # Parameters
    ///   * `id`: Snapshot ID to get the hash filename for
    ///
    /// # Return Values
    ///   * String: Hash filename for ID
    pub fn get_hash_filename(&self, id: SnapshotId) -> String {
        format!("{}_{}.hash", self.filename_prefix, id)
    }
}

impl Drop for Kvs {
    fn drop(&mut self) {
        if self.flush_on_exit.load(atomic::Ordering::Relaxed) {
            self.flush().ok();
        }
    }
}

impl TryFrom<&JsonValue> for KvsValue {
    type Error = ErrorCode;

    fn try_from(val: &JsonValue) -> Result<Self, Self::Error> {
        Ok(match val {
            JsonValue::Number(val) => KvsValue::Number(*val),
            JsonValue::Boolean(val) => KvsValue::Boolean(*val),
            JsonValue::String(val) => KvsValue::String(val.clone()),
            JsonValue::Null => KvsValue::Null,
            JsonValue::Array(val) => KvsValue::Array(
                val.iter()
                    .map(|x| x.try_into())
                    .collect::<Result<Vec<KvsValue>, ErrorCode>>()?,
            ),
            JsonValue::Object(val) => KvsValue::Object(
                val.iter()
                    .map(|(x, y)| Ok((x.clone(), y.try_into()?)))
                    .collect::<Result<HashMap<String, KvsValue>, ErrorCode>>()?,
            ),
        })
    }
}

impl TryFrom<&KvsValue> for JsonValue {
    type Error = ErrorCode;

    fn try_from(val: &KvsValue) -> Result<Self, Self::Error> {
        Ok(match val {
            KvsValue::Number(val) => JsonValue::Number(*val),
            KvsValue::Boolean(val) => JsonValue::Boolean(*val),
            KvsValue::String(val) => JsonValue::String(val.clone()),
            KvsValue::Null => JsonValue::Null,
            KvsValue::Array(val) => JsonValue::Array(
                val.iter()
                    .map(|x| x.try_into())
                    .collect::<Result<Vec<JsonValue>, ErrorCode>>()?,
            ),
            KvsValue::Object(val) => JsonValue::Object(
                val.iter()
                    .map(|(x, y)| Ok((x.clone(), y.try_into()?)))
                    .collect::<Result<HashMap<String, JsonValue>, ErrorCode>>()?,
            ),
        })
    }
}

macro_rules! impl_from_t_for_kvs_value {
    ($from:ty, $item:ident) => {
        impl From<$from> for KvsValue {
            fn from(val: $from) -> KvsValue {
                KvsValue::$item(val)
            }
        }
    };
}

impl_from_t_for_kvs_value!(f64, Number);
impl_from_t_for_kvs_value!(bool, Boolean);
impl_from_t_for_kvs_value!(String, String);
impl_from_t_for_kvs_value!(Vec<KvsValue>, Array);
impl_from_t_for_kvs_value!(HashMap<String, KvsValue>, Object);

impl From<()> for KvsValue {
    fn from(_data: ()) -> KvsValue {
        KvsValue::Null
    }
}

macro_rules! impl_from_kvs_value_to_t {
    ($to:ty, $item:ident) => {
        impl<'a> TryFrom<&'a KvsValue> for $to {
            type Error = ErrorCode;

            fn try_from(val: &'a KvsValue) -> Result<$to, Self::Error> {
                if let KvsValue::$item(val) = val {
                    Ok(val.clone())
                } else {
                    Err(ErrorCode::InvalidValueType)
                }
            }
        }
    };
}

impl_from_kvs_value_to_t!(f64, Number);
impl_from_kvs_value_to_t!(bool, Boolean);
impl_from_kvs_value_to_t!(String, String);
impl_from_kvs_value_to_t!(Vec<KvsValue>, Array);
impl_from_kvs_value_to_t!(HashMap<String, KvsValue>, Object);

impl<'a> TryFrom<&'a KvsValue> for () {
    type Error = ErrorCode;

    fn try_from(val: &'a KvsValue) -> Result<(), Self::Error> {
        if let KvsValue::Null = val {
            Ok(())
        } else {
            Err(ErrorCode::InvalidValueType)
        }
    }
}

// Note: The following logic was copied and adapted from TinyJSON.

pub trait KvsValueGet {
    fn get_inner_value(val: &KvsValue) -> Option<&Self>;
}

impl KvsValue {
    pub fn get<T: KvsValueGet>(&self) -> Option<&T> {
        T::get_inner_value(self)
    }
}

macro_rules! impl_kvs_get_inner_value {
    ($to:ty, $pat:pat => $val:expr) => {
        impl KvsValueGet for $to {
            fn get_inner_value(v: &KvsValue) -> Option<&$to> {
                use KvsValue::*;
                match v {
                    $pat => Some($val),
                    _ => None,
                }
            }
        }
    };
}

impl_kvs_get_inner_value!(f64, Number(n) => n);
impl_kvs_get_inner_value!(bool, Boolean(b) => b);
impl_kvs_get_inner_value!(String, String(s) => s);
impl_kvs_get_inner_value!((), Null => &());
impl_kvs_get_inner_value!(Vec<KvsValue>, Array(a) => a);
impl_kvs_get_inner_value!(HashMap<String, KvsValue>, Object(h) => h);

impl Index<usize> for KvsValue {
    type Output = KvsValue;

    fn index(&self, index: usize) -> &'_ Self::Output {
        let array = match self {
            KvsValue::Array(a) => a,
            _ => panic!(
                "Attempted to access to an array with index {index} but actually the value was {self:?}",
            ),
        };
        &array[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use tempdir::TempDir;

    #[must_use]
    fn test_dir() -> (TempDir, String) {
        let temp_dir = TempDir::new("").unwrap();
        let temp_path = temp_dir.path().display().to_string();
        (temp_dir, temp_path)
    }

    #[test]
    fn test_new_kvs_builder() {
        let instance_id = InstanceId::new(0);
        let builder = KvsBuilder::new(instance_id.clone()).dir(test_dir().1);

        assert_eq!(builder.instance_id, instance_id);
        assert!(!builder.need_defaults);
        assert!(!builder.need_kvs);
    }

    #[test]
    fn test_need_defaults() {
        let instance_id = InstanceId::new(0);
        let builder = KvsBuilder::new(instance_id.clone())
            .dir(test_dir().1)
            .need_defaults(true);

        assert!(builder.need_defaults);
    }

    #[test]
    fn test_need_kvs() {
        let instance_id = InstanceId::new(0);
        let builder = KvsBuilder::new(instance_id.clone())
            .dir(test_dir().1)
            .need_kvs(true);

        assert!(builder.need_kvs);
    }

    #[test]
    fn test_build() {
        let instance_id = InstanceId::new(0);
        let builder = KvsBuilder::new(instance_id.clone()).dir(test_dir().1);

        builder.build().unwrap();
    }

    #[test]
    fn test_build_with_defaults() {
        let instance_id = InstanceId::new(0);
        let builder = KvsBuilder::new(instance_id.clone())
            .dir(test_dir().1)
            .need_defaults(true);

        assert!(builder.build().is_err());
    }

    #[test]
    fn test_build_with_kvs() {
        let instance_id = InstanceId::new(0);
        let temp_dir = test_dir();

        // negative
        let builder = KvsBuilder::new(instance_id.clone())
            .dir(temp_dir.1.clone())
            .need_kvs(true);
        assert!(builder.build().is_err());

        KvsBuilder::new(instance_id.clone())
            .dir(temp_dir.1.clone())
            .build()
            .unwrap();

        // positive
        let builder = KvsBuilder::new(instance_id).dir(temp_dir.1).need_kvs(true);
        builder.build().unwrap();
    }

    #[test]
    fn test_unknown_error_code_from_io_error() {
        let error = std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid input provided");
        assert_eq!(ErrorCode::from(error), ErrorCode::UnmappedError);
    }

    #[test]
    fn test_unknown_error_code_from_json_parse_error() {
        let error = tinyjson::JsonParser::new("[1, 2, 3".chars())
            .parse()
            .unwrap_err();
        assert_eq!(ErrorCode::from(error), ErrorCode::JsonParserError);
    }

    #[test]
    fn test_unknown_error_code_from_json_generate_error() {
        let data: JsonValue = JsonValue::Number(f64::INFINITY);
        let error = data.stringify().unwrap_err();
        assert_eq!(ErrorCode::from(error), ErrorCode::JsonGeneratorError);
    }

    #[test]
    fn test_conversion_failed_from_utf8_error() {
        // test from: https://doc.rust-lang.org/std/string/struct.FromUtf8Error.html
        let bytes = vec![0, 159];
        let error = String::from_utf8(bytes).unwrap_err();
        assert_eq!(ErrorCode::from(error), ErrorCode::ConversionFailed);
    }

    #[test]
    fn test_conversion_failed_from_slice_error() {
        let bytes = [0x12, 0x34, 0x56, 0x78, 0xab];
        let bytes_ptr: &[u8] = &bytes;
        let error = TryInto::<[u8; 8]>::try_into(bytes_ptr).unwrap_err();
        assert_eq!(ErrorCode::from(error), ErrorCode::ConversionFailed);
    }

    #[test]
    fn test_conversion_failed_from_vec_u8() {
        let bytes: Vec<u8> = vec![];
        assert_eq!(ErrorCode::from(bytes), ErrorCode::ConversionFailed);
    }

    #[test]
    fn test_mutex_lock_failed_from_poison_error() {
        let mutex: Arc<Mutex<HashMap<String, KvsValue>>> = Arc::default();

        // test from: https://doc.rust-lang.org/std/sync/struct.PoisonError.html
        let c_mutex = Arc::clone(&mutex);
        let _ = thread::spawn(move || {
            let _unused = c_mutex.lock().unwrap();
            panic!();
        })
        .join();

        let error = mutex.lock().unwrap_err();
        assert_eq!(ErrorCode::from(error), ErrorCode::MutexLockFailed);
    }

    #[test]
    fn test_flush_on_exit() {
        let instance_id = InstanceId::new(0);
        let temp_dir = test_dir();
        let kvs = KvsBuilder::new(instance_id.clone())
            .dir(temp_dir.1.clone())
            .build()
            .unwrap();
        kvs.flush_on_exit(true);
    }

    #[test]
    fn test_reset() {
        let instance_id = InstanceId::new(0);
        let temp_dir = test_dir();
        let kvs = KvsBuilder::new(instance_id.clone())
            .dir(temp_dir.1.clone())
            .build()
            .unwrap();
        let _ = kvs.reset();
    }

    #[test]
    fn test_get_all_keys() {
        let instance_id = InstanceId::new(0);
        let temp_dir = test_dir();
        let kvs = KvsBuilder::new(instance_id.clone())
            .dir(temp_dir.1.clone())
            .build()
            .unwrap();
        let _ = kvs.set_value("test", KvsValue::Number(1.0));
        let _ = kvs.get_all_keys();
    }

    #[test]
    fn test_key_exists() {
        let instance_id = InstanceId::new(0);
        let temp_dir = test_dir();
        let kvs = KvsBuilder::new(instance_id.clone())
            .dir(temp_dir.1.clone())
            .build()
            .unwrap();
        let _ = kvs.key_exists("test");
    }

    #[test]
    fn test_get_default_value() {
        let instance_id = InstanceId::new(0);
        let temp_dir = test_dir();

        std::fs::copy(
            "tests/kvs_0_default.json",
            format!("{}/kvs_0_default.json", temp_dir.1.clone()),
        )
        .unwrap();

        std::fs::copy(
            "tests/kvs_0_default.hash",
            format!("{}/kvs_0_default.hash", temp_dir.1.clone()),
        )
        .unwrap();

        let kvs = KvsBuilder::new(instance_id.clone())
            .dir(temp_dir.1.clone())
            .need_defaults(true)
            .build()
            .unwrap();

        // negative
        let _ = kvs.get_default_value("test");

        // positive
        let _ = kvs.get_default_value("bool1");
    }

    #[test]
    fn test_has_default_value() {
        let instance_id = InstanceId::new(0);
        let temp_dir = test_dir();
        let kvs = KvsBuilder::new(instance_id.clone())
            .dir(temp_dir.1.clone())
            .build()
            .unwrap();
        let _ = kvs.has_default_value("test");
    }

    #[test]
    fn test_remove_key() {
        let instance_id = InstanceId::new(0);
        let temp_dir = test_dir();
        let kvs = KvsBuilder::new(instance_id.clone())
            .dir(temp_dir.1.clone())
            .build()
            .unwrap();
        let _ = kvs.remove_key("test");
        let _ = kvs.set_value("test", KvsValue::Number(1.0));
        let _ = kvs.remove_key("test");
    }

    #[test]
    fn test_restore_snapshot() {
        let instance_id = InstanceId::new(0);
        let temp_dir = test_dir();
        let kvs = KvsBuilder::new(instance_id.clone())
            .dir(temp_dir.1.clone())
            .build()
            .unwrap();

        assert!(kvs.flush().is_ok());
        assert!(kvs.snapshot_restore(SnapshotId::new(1)).is_err());
        assert!(kvs.flush().is_ok());
        assert!(kvs.snapshot_restore(SnapshotId::new(1)).is_ok());
        assert!(kvs.snapshot_restore(SnapshotId::new(0)).is_err());

        let snapshot = format!("{}/kvs_0_1.hash", temp_dir.1.clone());
        let _ = std::fs::remove_file(snapshot.clone());

        assert!(kvs.snapshot_restore(SnapshotId::new(1)).is_err());
    }

    #[test]
    fn test_rotate_snapshot() {
        let instance_id = InstanceId::new(0);
        let temp_dir = test_dir();
        let kvs = KvsBuilder::new(instance_id.clone())
            .dir(temp_dir.1.clone())
            .build()
            .unwrap();
        let _ = kvs.flush();

        let metadata = fs::metadata(temp_dir.1.clone()).unwrap();
        let mut permissions = metadata.permissions();

        permissions.set_readonly(true);
        fs::set_permissions(temp_dir.1.clone(), permissions.clone()).unwrap();
        assert!(kvs.snapshot_rotate().is_err());

        permissions.set_readonly(false);
        fs::set_permissions(temp_dir.1.clone(), permissions).unwrap();
        assert!(kvs.snapshot_rotate().is_ok());

        assert!(fs::remove_file(format!("{}/kvs_0_1.json", temp_dir.1.clone())).is_ok());
        assert!(kvs.snapshot_rotate().is_err());
    }

    #[test]
    fn test_get_filename() {
        let instance_id = InstanceId::new(0);
        let temp_dir = test_dir();
        let kvs = KvsBuilder::new(instance_id.clone())
            .dir(temp_dir.1.clone())
            .build()
            .unwrap();
        let _ = kvs.get_kvs_filename(SnapshotId::new(0));
    }

    #[test]
    fn test_get_value() {
        let instance_id = InstanceId::new(0);
        let temp_dir = test_dir();
        let kvs = Arc::new(
            KvsBuilder::new(instance_id.clone())
                .dir(temp_dir.1.clone())
                .build()
                .unwrap(),
        );
        let _ = kvs.set_value("test", KvsValue::Number(123.0));

        let c_kvs = kvs.clone();
        let _ = thread::spawn(move || {
            let _ = c_kvs.get_value("test");
            panic!();
        })
        .join();
    }

    #[test]
    #[should_panic]
    fn test_kvs_value_array_index_fail() {
        let _ = KvsValue::Number(123.0)[10];
    }

    #[test]
    fn test_from_kvs_value_f64_null() {
        let res = f64::try_from(&KvsValue::Null);
        assert!(res.is_err());
    }

    #[test]
    fn test_get_inner_value() {
        let value = KvsValue::Null;
        let _ = f64::get_inner_value(&value);
    }

    #[test]
    fn test_drop() {
        let instance_id = InstanceId::new(0);
        let temp_dir = test_dir();
        let kvs = Arc::new(
            KvsBuilder::new(instance_id.clone())
                .dir(temp_dir.1.clone())
                .build()
                .unwrap(),
        );
        kvs.flush_on_exit(false);
    }

    impl<'a> TryFrom<&'a KvsValue> for u64 {
        type Error = ErrorCode;

        fn try_from(_val: &'a KvsValue) -> Result<u64, Self::Error> {
            Err(ErrorCode::ConversionFailed)
        }
    }

    #[test]
    fn test_get_value_try_from_error() {
        let instance_id = InstanceId::new(0);
        let temp_dir = test_dir();

        std::fs::copy(
            "tests/kvs_0_default.json",
            format!("{}/kvs_0_default.json", temp_dir.1.clone()),
        )
        .unwrap();

        std::fs::copy(
            "tests/kvs_0_default.hash",
            format!("{}/kvs_0_default.hash", temp_dir.1.clone()),
        )
        .unwrap();

        let kvs = Arc::new(
            KvsBuilder::new(instance_id.clone())
                .dir(temp_dir.1.clone())
                .need_defaults(true)
                .build()
                .unwrap(),
        );
        let _ = kvs.set_value("test", KvsValue::Number(123.0));

        // stored value
        let _ = kvs.get_value("test");

        // default value
        let _ = kvs.get_value("bool1");
    }

    #[test]
    fn test_snapshot_id() {
        let snapshot_id: SnapshotId = 123.into();
        assert_eq!(snapshot_id, SnapshotId(123));
    }

    #[test]
    fn test_set_default_value() {
        let instance_id = InstanceId::new(0);
        let temp_dir = test_dir();

        std::fs::copy(
            "tests/kvs_0_default.json",
            format!("{}/kvs_0_default.json", temp_dir.1.clone()),
        )
        .unwrap();

        std::fs::copy(
            "tests/kvs_0_default.hash",
            format!("{}/kvs_0_default.hash", temp_dir.1.clone()),
        )
        .unwrap();

        let kvs = Arc::new(
            KvsBuilder::new(instance_id.clone())
                .dir(temp_dir.1.clone())
                .need_defaults(true)
                .build()
                .unwrap(),
        );

        assert_eq!(
            kvs.get_value("string1"),
            Ok(KvsValue::String("Hello".to_string()))
        );
        assert!(kvs
            .set_value("string1", KvsValue::String("Bye".to_string()))
            .is_ok());
        assert_eq!(
            kvs.get_value("string1"),
            Ok(KvsValue::String("Bye".to_string()))
        );
        assert!(kvs.set_default_value("string1").is_ok());
        assert_eq!(
            kvs.get_value("string1"),
            Ok(KvsValue::String("Hello".to_string()))
        );
        assert_eq!(
            kvs.set_default_value("string2").err(),
            Some(ErrorCode::KeyDefaultNotFound)
        );
    }

    #[test]
    fn test_has_default_value_with_mutex_lock_fail() {
        let instance_id = InstanceId::new(0);
        let temp_dir = test_dir();

        std::fs::copy(
            "tests/kvs_0_default.json",
            format!("{}/kvs_0_default.json", temp_dir.1.clone()),
        )
        .unwrap();

        std::fs::copy(
            "tests/kvs_0_default.hash",
            format!("{}/kvs_0_default.hash", temp_dir.1.clone()),
        )
        .unwrap();

        let kvs = Arc::new(
            KvsBuilder::new(instance_id.clone())
                .dir(temp_dir.1.clone())
                .need_defaults(true)
                .build()
                .unwrap(),
        );

        // test from: https://doc.rust-lang.org/std/sync/struct.PoisonError.html
        let c_kvs = kvs.clone();
        let _ = thread::spawn(move || {
            let _unused = c_kvs.kvs.lock().unwrap();
            panic!();
        })
        .join();

        assert!(!kvs.has_default_value("string1"));
    }

    #[test]
    fn test_try_from_kvs_value() {
        assert!(f64::try_from(&KvsValue::Number(123.0)).is_ok());
        assert!(<()>::try_from(&KvsValue::Null).is_ok());
        assert_eq!(
            <()>::try_from(&KvsValue::Number(123.0)),
            Err(ErrorCode::InvalidValueType)
        );
        assert_eq!(
            u64::try_from(&KvsValue::Number(123.0)),
            Err(ErrorCode::ConversionFailed)
        );
    }
}
