use crate::kvs_value::{KvsMap, KvsValue};

use crate::error_code::ErrorCode;

use crate::json_backend::json_value::{KvsJson, TinyJson};

use std::fs;

// Unified error type for all backend and JSON errors
#[derive(Debug)]
pub enum KvsBackendError {
    Io(std::io::Error),
    Json(String),
    JsonParserError,
    ValidationFailed,
    KvsHashFileReadError,
    MutexLockFailed,
    KeyNotFound,
    ConversionFailed,
    UnmappedError,
}

impl std::fmt::Display for KvsBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KvsBackendError::Io(e) => write!(f, "IO error: {e}"),
            KvsBackendError::Json(e) => write!(f, "JSON error: {e}"),
            KvsBackendError::JsonParserError => write!(f, "JSON parser error"),
            KvsBackendError::ValidationFailed => write!(f, "Validation failed"),
            KvsBackendError::KvsHashFileReadError => write!(f, "KVS hash file read error"),
            KvsBackendError::MutexLockFailed => write!(f, "Mutex lock failed"),
            KvsBackendError::KeyNotFound => write!(f, "Key not found"),
            KvsBackendError::ConversionFailed => write!(f, "Conversion failed"),
            KvsBackendError::UnmappedError => write!(f, "Unmapped error"),
        }
    }
}

impl std::error::Error for KvsBackendError {}

impl From<std::io::Error> for KvsBackendError {
    fn from(e: std::io::Error) -> Self {
        KvsBackendError::Io(e)
    }
}

// Trait for persisting and loading KvsMap from storage
pub trait PersistKvs {
    fn get_kvs_from_file(
        filename: &str,
        kvs: &mut KvsMap,
        verify_hash: bool,
        hash_filename: &str,
    ) -> Result<(), KvsBackendError>;
    fn persist_kvs_to_file(
        kvs: &KvsMap,
        filename: &str,
        add_hash: bool,
    ) -> Result<(), KvsBackendError>;
}

#[derive(Default)]
pub struct DefaultPersistKvs<J: KvsJson = TinyJson> {
    _marker: std::marker::PhantomData<J>,
}

impl<J: KvsJson> PersistKvs for DefaultPersistKvs<J> {
    fn get_kvs_from_file(
        filename: &str,
        kvs: &mut KvsMap,
        verify_hash: bool,
        hash_filename: &str,
    ) -> Result<(), KvsBackendError> {
        let filename = &format!("{filename}.json");
        let data = fs::read_to_string(filename).map_err(KvsBackendError::Io)?;
        let json_val = J::parse(&data).map_err(|e| KvsBackendError::Json(e.to_string()))?;
        // Hash check logic (use parsed data)
        if verify_hash && !hash_filename.is_empty() {
            match fs::read(hash_filename) {
                Ok(hash_bytes) => {
                    let hash_kvs = adler32::RollingAdler32::from_buffer(data.as_bytes()).hash();
                    if hash_bytes.len() == 4 {
                        let file_hash = u32::from_be_bytes([
                            hash_bytes[0],
                            hash_bytes[1],
                            hash_bytes[2],
                            hash_bytes[3],
                        ]);
                        if hash_kvs != file_hash {
                            return Err(KvsBackendError::ValidationFailed);
                        }
                    } else {
                        return Err(KvsBackendError::ValidationFailed);
                    }
                }
                Err(_) => {
                    return Err(KvsBackendError::KvsHashFileReadError);
                }
            }
        }
        let kvs_val = J::to_kvs_value(json_val);
        if let KvsValue::Object(map) = kvs_val {
            kvs.clear();
            kvs.extend(map);
            Ok(())
        } else {
            Err(KvsBackendError::JsonParserError)
        }
    }

    fn persist_kvs_to_file(
        kvs: &KvsMap,
        filename: &str,
        add_hash: bool,
    ) -> Result<(), KvsBackendError> {
        let filename = &format!("{}_0.json", filename);
        let kvs_val = KvsValue::Object(kvs.clone());
        let json_val = J::from_kvs_value(&kvs_val);
        let json_str = J::stringify(&json_val).map_err(|e| KvsBackendError::Json(e.to_string()))?;
        fs::write(filename, &json_str).map_err(KvsBackendError::from)?;
        if add_hash {
            // Compute hash and write to hash file
            let hash = adler32::RollingAdler32::from_buffer(json_str.as_bytes()).hash();
            // If filename ends with .json, replace with .hash
            let filename_hash = if filename.ends_with(".json") {
                filename.trim_end_matches(".json").to_string() + ".hash"
            } else {
                filename.to_string() + ".hash"
            };
            fs::write(&filename_hash, hash.to_be_bytes()).map_err(KvsBackendError::from)?;
        }
        Ok(())
    }
}

impl From<KvsBackendError> for ErrorCode {
    fn from(e: KvsBackendError) -> Self {
        match e {
            KvsBackendError::Io(_) => ErrorCode::KvsFileReadError,
            KvsBackendError::Json(_) => ErrorCode::JsonParserError,
            KvsBackendError::JsonParserError => ErrorCode::JsonParserError,
            KvsBackendError::ValidationFailed => ErrorCode::ValidationFailed,
            KvsBackendError::KvsHashFileReadError => ErrorCode::KvsHashFileReadError,
            KvsBackendError::MutexLockFailed => ErrorCode::MutexLockFailed,
            KvsBackendError::KeyNotFound => ErrorCode::KeyNotFound,
            KvsBackendError::ConversionFailed => ErrorCode::ConversionFailed,
            KvsBackendError::UnmappedError => ErrorCode::UnmappedError,
        }
    }
}
