use crate::error_code::ErrorCode;
use crate::kvs_backend::KvsBackend;
use crate::kvs_value::{KvsMap, KvsValue};
use std::fs;
use std::path::PathBuf;

use tinyjson::{JsonGenerateError, JsonParseError, JsonValue};

/// Backend-specific JsonValue -> KvsValue conversion.
impl From<JsonValue> for KvsValue {
    fn from(val: JsonValue) -> KvsValue {
        match val {
            JsonValue::Number(n) => KvsValue::Number(n),
            JsonValue::Boolean(b) => KvsValue::Boolean(b),
            JsonValue::String(s) => KvsValue::String(s),
            JsonValue::Null => KvsValue::Null,
            JsonValue::Array(arr) => KvsValue::Array(arr.into_iter().map(KvsValue::from).collect()),
            JsonValue::Object(obj) => KvsValue::Object(
                obj.into_iter()
                    .map(|(k, v)| (k.clone(), KvsValue::from(v)))
                    .collect(),
            ),
        }
    }
}

/// Backend-specific KvsValue -> JsonValue conversion.
impl From<KvsValue> for JsonValue {
    fn from(val: KvsValue) -> JsonValue {
        match val {
            KvsValue::Number(n) => JsonValue::Number(n),
            KvsValue::Boolean(b) => JsonValue::Boolean(b),
            KvsValue::String(s) => JsonValue::String(s),
            KvsValue::Null => JsonValue::Null,
            KvsValue::Array(arr) => {
                JsonValue::Array(arr.into_iter().map(JsonValue::from).collect())
            }
            KvsValue::Object(map) => JsonValue::Object(
                map.into_iter()
                    .map(|(k, v)| (k.clone(), JsonValue::from(v)))
                    .collect(),
            ),
        }
    }
}

/// tinyjson::JsonParseError -> ErrorCode::JsonParseError
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

/// tinyjson::JsonGenerateError -> ErrorCode::JsonGenerateError
impl From<JsonGenerateError> for ErrorCode {
    fn from(cause: JsonGenerateError) -> Self {
        eprintln!("error: JSON generator error: msg = {}", cause.message());
        ErrorCode::JsonGeneratorError
    }
}

/// KVS backend implementation based on TinyJSON.
pub struct JsonBackend;

impl JsonBackend {
    fn parse(s: &str) -> Result<JsonValue, ErrorCode> {
        s.parse()
            .map_err(|_e: JsonParseError| crate::error_code::ErrorCode::JsonParserError)
    }

    fn stringify(val: &JsonValue) -> Result<String, ErrorCode> {
        val.stringify()
            .map_err(|_e: JsonGenerateError| crate::error_code::ErrorCode::JsonParserError)
    }
}

impl KvsBackend for JsonBackend {
    fn load_kvs(
        source_path: PathBuf,
        verify_hash: bool,
        hash_source: Option<PathBuf>,
    ) -> Result<KvsMap, ErrorCode> {
        let filename = source_path.with_extension("json");
        let data = fs::read_to_string(&filename).map_err(|_| ErrorCode::KvsFileReadError)?;
        let json_value = Self::parse(&data).map_err(|_| ErrorCode::JsonParserError)?;

        // Hash check logic (use parsed data)
        if verify_hash {
            if let Some(hash_filename) = hash_source {
                if !hash_filename.as_os_str().is_empty() {
                    match fs::read(&hash_filename) {
                        Ok(hash_bytes) => {
                            let hash_kvs =
                                adler32::RollingAdler32::from_buffer(data.as_bytes()).hash();
                            if hash_bytes.len() == 4 {
                                let file_hash = u32::from_be_bytes([
                                    hash_bytes[0],
                                    hash_bytes[1],
                                    hash_bytes[2],
                                    hash_bytes[3],
                                ]);
                                if hash_kvs != file_hash {
                                    return Err(ErrorCode::ValidationFailed);
                                }
                            } else {
                                return Err(ErrorCode::ValidationFailed);
                            }
                        }
                        Err(_) => {
                            return Err(ErrorCode::KvsHashFileReadError);
                        }
                    }
                }
            }
        }

        let kvs_value = KvsValue::from(json_value);
        if let KvsValue::Object(kvs_map) = kvs_value {
            Ok(kvs_map)
        } else {
            Err(ErrorCode::JsonParserError)
        }
    }

    fn save_kvs(kvs: &KvsMap, destination_path: PathBuf, add_hash: bool) -> Result<(), ErrorCode> {
        let filename = destination_path
            .with_extension("json")
            .with_file_name(format!("{}_0.json", destination_path.display()));

        let kvs_value = KvsValue::Object(kvs.clone());
        let json_value = JsonValue::from(kvs_value);

        let json_str = Self::stringify(&json_value).map_err(|_| ErrorCode::JsonParserError)?;
        fs::write(&filename, &json_str).map_err(|_| ErrorCode::KvsFileReadError)?;

        if add_hash {
            // Compute hash and write to hash file
            let hash = adler32::RollingAdler32::from_buffer(json_str.as_bytes()).hash();
            // If filename ends with .json, replace with .hash
            let filename_hash = if let Some(stem) = filename.file_stem() {
                let mut hash_path = filename.clone();
                hash_path.set_file_name(format!("{}.hash", stem.to_string_lossy()));
                hash_path
            } else {
                let mut hash_path = filename.clone();
                hash_path.set_extension("hash");
                hash_path
            };
            fs::write(&filename_hash, hash.to_be_bytes())
                .map_err(|_| ErrorCode::KvsFileReadError)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
