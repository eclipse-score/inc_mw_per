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
//! The key-value-storage is opened or initialized with [`KvsBuilder::<Kvs>::new`] where various settings
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
//! To read a value call [`Kvs::get_value`](Kvs::get_value) or [`Kvs::get_value_as::<T>`](Kvs::get_value_as)
//! with the `key` as first parameter. `T` represents the type to read and can be `f64`, `bool`, `String`, `()`,
//! `Vec<KvsValue>`, `HashMap<String, KvsValue>` or `KvsValue`.
//! Also `let value: f64 = kvs.get_value_as()` can be used.
//!
//! If a `key` isn't available in the KVS a lookup into the defaults storage will be performed and
//! if the `value` is found the default will be returned. The default value isn't stored when
//! [`Kvs::flush`] is called unless it's explicitly written with [`Kvs::set_value`]. So when
//! defaults change always the latest values will be returned. If that is an unwanted behaviour
//! it's better to remove the default value and write the value permanently when the KVS is
//! initialized. To check whether a value has a default call [`Kvs::get_default_value`] and to
//! see if the value wasn't written yet and will return the default call
//! [`Kvs::is_value_default`].
//!
//!
//! ## Example Usage
//!
//! ```
//! use rust_kvs::{ErrorCode, InstanceId,Kvs,KvsApi, KvsBuilder, KvsValue};
//! use std::collections::HashMap;
//!
//! fn main() -> Result<(), ErrorCode> {
//!    
//!     
//!     let kvs = KvsBuilder::<Kvs>::new(InstanceId::new(0)).dir("").build()?;
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

extern crate alloc;

//core and alloc libs
use alloc::string::FromUtf8Error;
use core::array::TryFromSliceError;
use core::fmt;
use core::ops::Index;

//std libs
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{
    atomic::{self, AtomicBool},
    Mutex, MutexGuard, PoisonError,
};

//external libs
use adler32::RollingAdler32;
use tinyjson::{JsonGenerateError, JsonGenerator, JsonParseError, JsonValue};

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

    /// Serialization failed
    SerializationFailed,

    /// Invalid snapshot ID
    InvalidSnapshotId,

    /// Conversion failed
    ConversionFailed,

    /// Mutex failed
    MutexLockFailed,
}

/// Key-value-storage builder
pub struct KvsBuilder<T: KvsApi = Kvs> {
    /// Instance ID
    instance_id: InstanceId,

    /// Need-defaults flag
    need_defaults: bool,

    /// Need-KVS flag
    need_kvs: bool,

    /// Working directory
    dir: Option<String>,

    /// Phantom data for drop check
    _phantom: std::marker::PhantomData<T>,
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
#[derive(Clone, Debug)]
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

pub trait KvsApi {
    fn open(
        instance_id: InstanceId,
        need_defaults: OpenNeedDefaults,
        need_kvs: OpenNeedKvs,
        dir: Option<String>,
    ) -> Result<Self, ErrorCode>
    where
        Self: Sized;

    fn reset(&self) -> Result<(), ErrorCode>;
    fn get_all_keys(&self) -> Result<Vec<String>, ErrorCode>;
    fn key_exists(&self, key: &str) -> Result<bool, ErrorCode>;
    fn get_value(&self, key: &str) -> Result<KvsValue, ErrorCode>;
    fn get_value_as<T>(&self, key: &str) -> Result<T, ErrorCode>
    where
        for<'a> T: TryFrom<&'a KvsValue> + Clone,
        for<'a> <T as TryFrom<&'a KvsValue>>::Error: std::fmt::Debug;
    fn get_default_value(&self, key: &str) -> Result<KvsValue, ErrorCode>;
    fn is_value_default(&self, key: &str) -> Result<bool, ErrorCode>;
    fn set_value<S: Into<String>, J: Into<KvsValue>>(
        &self,
        key: S,
        value: J,
    ) -> Result<(), ErrorCode>;
    fn remove_key(&self, key: &str) -> Result<(), ErrorCode>;
    fn flush_on_exit(&self, flush_on_exit: bool);
    fn flush(&self) -> Result<(), ErrorCode>;
    fn snapshot_count(&self) -> usize;
    fn snapshot_max_count() -> usize
    where
        Self: Sized;
    fn snapshot_restore(&self, id: SnapshotId) -> Result<(), ErrorCode>;
    fn get_kvs_filename(&self, id: SnapshotId) -> Result<PathBuf, ErrorCode>;
    fn get_hash_filename(&self, id: SnapshotId) -> Result<PathBuf, ErrorCode>;
}

impl<T> KvsBuilder<T>
where
    T: KvsApi,
{
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
            _phantom: std::marker::PhantomData,
        }
    }

    /// Configure if defaults must exist when opening the KVS
    ///
    /// # Parameters
    ///   * `flag`: Yes = `true`, no = `false` (default)
    ///
    /// # Return Values
    ///   * KvsBuilder instance
    pub fn need_defaults(mut self, flag: bool) -> Self {
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
    pub fn need_kvs(mut self, flag: bool) -> Self {
        self.need_kvs = flag;
        self
    }

    /// Set the key-value-storage permanent storage directory
    ///
    /// # Parameters
    ///   * `dir`: Path to permanent storage
    ///
    /// # Return Values
    pub fn dir<P: Into<String>>(mut self, dir: P) -> Self {
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
    pub fn build(self) -> Result<T, ErrorCode> {
        T::open(
            self.instance_id,
            self.need_defaults.into(),
            self.need_kvs.into(),
            self.dir,
        )
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

impl From<&JsonValue> for KvsValue {
    fn from(val: &JsonValue) -> KvsValue {
        match val {
            JsonValue::Number(val) => KvsValue::Number(*val),
            JsonValue::Boolean(val) => KvsValue::Boolean(*val),
            JsonValue::String(val) => KvsValue::String(val.clone()),
            JsonValue::Null => KvsValue::Null,
            JsonValue::Array(val) => KvsValue::Array(val.iter().map(|x| x.into()).collect()),
            JsonValue::Object(val) => {
                KvsValue::Object(val.iter().map(|(x, y)| (x.clone(), y.into())).collect())
            }
        }
    }
}

impl From<&KvsValue> for JsonValue {
    fn from(val: &KvsValue) -> JsonValue {
        match val {
            KvsValue::Number(val) => JsonValue::Number(*val),
            KvsValue::Boolean(val) => JsonValue::Boolean(*val),
            KvsValue::String(val) => JsonValue::String(val.clone()),
            KvsValue::Null => JsonValue::Null,
            KvsValue::Array(val) => JsonValue::Array(val.iter().map(|x| x.into()).collect()),
            KvsValue::Object(val) => {
                JsonValue::Object(val.iter().map(|(x, y)| (x.clone(), y.into())).collect())
            }
        }
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
        impl<'a> From<&'a KvsValue> for $to {
            fn from(val: &'a KvsValue) -> $to {
                if let KvsValue::$item(val) = val {
                    return val.clone();
                }

                panic!("Invalid KvsValue type");
            }
        }
    };
}

impl_from_kvs_value_to_t!(f64, Number);
impl_from_kvs_value_to_t!(bool, Boolean);
impl_from_kvs_value_to_t!(String, String);
impl_from_kvs_value_to_t!(Vec<KvsValue>, Array);
impl_from_kvs_value_to_t!(HashMap<String, KvsValue>, Object);

impl<'a> From<&'a KvsValue> for () {
    fn from(val: &'a KvsValue) {
        if let KvsValue::Null = val {
            return;
        }

        panic!("Invalid KvsValue type for ()");
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
    use tempfile::tempdir;

    #[test]
    fn test_new_kvs_builder() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let builder = KvsBuilder::<Kvs>::new(instance_id.clone()).dir(dir_path);

        assert_eq!(builder.instance_id, instance_id);
        assert!(!builder.need_defaults);
        assert!(!builder.need_kvs);
    }

    #[test]
    fn test_need_defaults() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let builder = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path)
            .need_defaults(true);

        assert!(builder.need_defaults);
    }

    #[test]
    fn test_need_kvs() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let builder = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path)
            .need_kvs(true);

        assert!(builder.need_kvs);
    }

    #[test]
    fn test_build() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let builder = KvsBuilder::<Kvs>::new(instance_id.clone()).dir(dir_path);

        builder.build().unwrap();
    }

    #[test]
    fn test_build_with_defaults() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let builder = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path)
            .need_defaults(true);

        assert!(builder.build().is_err());
    }

    #[test]
    fn test_build_with_kvs() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);

        // negative
        let builder = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path.clone())
            .need_kvs(true);
        assert!(builder.build().is_err());

        KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path.clone())
            .build()
            .unwrap();

        // positive
        let builder = KvsBuilder::<Kvs>::new(instance_id)
            .dir(dir_path)
            .need_kvs(true);
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
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let kvs = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path.clone())
            .build()
            .unwrap();
        kvs.flush_on_exit(true);
    }

    #[test]
    fn test_reset() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let kvs = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path.clone())
            .build()
            .unwrap();
        let _ = kvs.set_value("test", KvsValue::Number(1.0));
        let result = kvs.reset();
        assert!(result.is_ok(), "Expected Ok for reset");
    }

    #[test]
    fn test_get_all_keys() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let kvs = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path.clone())
            .build()
            .unwrap();
        let _ = kvs.set_value("test", KvsValue::Number(1.0));
        let keys = kvs.get_all_keys();
        assert!(keys.is_ok(), "Expected Ok for get_all_keys");
        let keys = keys.unwrap();
        assert!(
            keys.contains(&"test".to_string()),
            "Expected 'test' key in get_all_keys"
        );
    }

    #[test]
    fn test_key_exists() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let kvs = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path.clone())
            .build()
            .unwrap();
        let exists = kvs.key_exists("test");
        assert!(exists.is_ok(), "Expected Ok for key_exists");
        assert!(!exists.unwrap(), "Expected 'test' key to not exist");
        let _ = kvs.set_value("test", KvsValue::Number(1.0));
        let exists = kvs.key_exists("test");
        assert!(exists.is_ok(), "Expected Ok for key_exists after set");
        assert!(exists.unwrap(), "Expected 'test' key to exist after set");
    }

    #[test]
    fn test_get_filename_before_flush() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let kvs = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path)
            .build()
            .unwrap();
        let result = kvs.get_kvs_filename(SnapshotId::new(0));
        assert!(result.is_err_and(|e| e == ErrorCode::FileNotFound));
    }

    #[test]
    fn test_get_filename() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let kvs = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path.clone())
            .build()
            .unwrap();
        kvs.flush().unwrap();
        let filename = kvs.get_kvs_filename(SnapshotId::new(0)).unwrap();
        let filename_str = filename.to_string_lossy().to_string();
        assert!(
            filename_str.ends_with("_0.json"),
            "Expected filename to end with _0.json: {}",
            filename.display()
        );
    }

    #[test]
    fn test_get_value() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let kvs = Arc::new(
            KvsBuilder::<Kvs>::new(instance_id.clone())
                .dir(dir_path.clone())
                .build()
                .unwrap(),
        );
        let _ = kvs.set_value("test", KvsValue::Number(123.0));
        let value = kvs.get_value("test").unwrap();
        assert_eq!(
            *value.get::<f64>().unwrap(),
            123.0,
            "Expected to retrieve the inserted value"
        );
    }

    #[test]
    fn test_get_value_as() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let kvs = Arc::new(
            KvsBuilder::<Kvs>::new(instance_id.clone())
                .dir(dir_path.clone())
                .build()
                .unwrap(),
        );
        let _ = kvs.set_value("test", KvsValue::Number(123.0));
        let value = kvs.get_value_as::<f64>("test");
        assert_eq!(
            value.unwrap(),
            123.0,
            "Expected to retrieve the inserted value"
        );
    }

    #[test]
    fn test_get_inner_value() {
        let value = KvsValue::Number(42.0);
        let inner = f64::get_inner_value(&value);
        assert_eq!(inner, Some(&42.0), "Expected to get inner f64 value");
    }

    #[test]
    fn test_drop() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let kvs = Arc::new(
            KvsBuilder::<Kvs>::new(instance_id.clone())
                .dir(dir_path.clone())
                .build()
                .unwrap(),
        );
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

    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_get_value_try_from_error() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);

        std::fs::copy(
            "tests/kvs_0_default.json",
            format!("{}/kvs_0_default.json", dir_path.clone()),
        )
        .unwrap();

        let kvs = Arc::new(
            KvsBuilder::<Kvs>::new(instance_id.clone())
                .dir(dir_path.clone())
                .need_defaults(true)
                .build()
                .unwrap(),
        );

        let _ = kvs.set_value("test", KvsValue::Number(123.0f64));

        // stored value: should return ConversionFailed
        let result = kvs.get_value_as::<u64>("test");
        assert!(
            matches!(result, Err(ErrorCode::ConversionFailed)),
            "Expected ConversionFailed for stored value"
        );

        // default value: should return ConversionFailed
        let result = kvs.get_value_as::<u64>("bool1");
        assert!(
            matches!(result, Err(ErrorCode::ConversionFailed)),
            "Expected ConversionFailed for default value"
        );
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
}
