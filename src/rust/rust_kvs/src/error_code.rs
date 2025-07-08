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

extern crate alloc;

use alloc::string::FromUtf8Error;
use core::array::TryFromSliceError;

use crate::kvs_value::KvsValue;
use std::collections::HashMap;
use std::sync::{MutexGuard, PoisonError};

use tinyjson::{JsonGenerateError, JsonParseError};

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

    // Key has no default value
    KeyDefaultNotFound,

    /// Serialization failed
    SerializationFailed,

    /// Invalid snapshot ID
    InvalidSnapshotId,

    /// Conversion failed
    ConversionFailed,

    /// Mutex failed
    MutexLockFailed,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use tinyjson::JsonValue;

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
}
