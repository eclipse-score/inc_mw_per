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
use crate::kvs_api::{InstanceId, SnapshotId};
use crate::kvs_backend::{KvsBackend, KvsPathResolver};
use crate::kvs_value::{KvsMap, KvsValue};
use crate::KVS_MAX_SNAPSHOTS;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tinyjson::{JsonGenerateError, JsonParseError, JsonValue};

// Example of how KvsValue is stored in the JSON file (t-tagged format):
// {
//   "my_int": { "t": "i32", "v": 42 },
//   "my_float": { "t": "f64", "v": 3.1415 },
//   "my_bool": { "t": "bool", "v": true },
//   "my_string": { "t": "str", "v": "hello" },
//   "my_array": { "t": "arr", "v": [ ... ] },
//   "my_object": { "t": "obj", "v": { ... } },
//   "my_null": { "t": "null", "v": null }
// }

/// Backend-specific JsonValue -> KvsValue conversion.
impl From<JsonValue> for KvsValue {
    fn from(val: JsonValue) -> KvsValue {
        match val {
            JsonValue::Object(mut obj) => {
                // Type-tagged: { "t": ..., "v": ... }
                if let (Some(JsonValue::String(type_str)), Some(value)) =
                    (obj.remove("t"), obj.remove("v"))
                {
                    return match (type_str.as_str(), value) {
                        ("i32", JsonValue::Number(v)) => KvsValue::I32(v as i32),
                        ("u32", JsonValue::Number(v)) => KvsValue::U32(v as u32),
                        ("i64", JsonValue::Number(v)) => KvsValue::I64(v as i64),
                        ("u64", JsonValue::Number(v)) => KvsValue::U64(v as u64),
                        ("f64", JsonValue::Number(v)) => KvsValue::F64(v),
                        ("bool", JsonValue::Boolean(v)) => KvsValue::Boolean(v),
                        ("str", JsonValue::String(v)) => KvsValue::String(v),
                        ("null", JsonValue::Null) => KvsValue::Null,
                        ("arr", JsonValue::Array(v)) => {
                            KvsValue::Array(v.into_iter().map(KvsValue::from).collect())
                        }
                        ("obj", JsonValue::Object(v)) => KvsValue::Object(
                            v.into_iter().map(|(k, v)| (k, KvsValue::from(v))).collect(),
                        ),
                        // Remaining types can be handled with Null.
                        _ => KvsValue::Null,
                    };
                }
                // If not a t-tagged object, treat as a map of key-value pairs (KvsMap)
                let map: KvsMap = obj
                    .into_iter()
                    .map(|(k, v)| (k, KvsValue::from(v)))
                    .collect();
                KvsValue::Object(map)
            }
            // Remaining types can be handled with Null.
            _ => KvsValue::Null,
        }
    }
}

/// Backend-specific KvsValue -> JsonValue conversion.
impl From<KvsValue> for JsonValue {
    fn from(val: KvsValue) -> JsonValue {
        let mut obj = HashMap::new();
        match val {
            KvsValue::I32(n) => {
                obj.insert("t".to_string(), JsonValue::String("i32".to_string()));
                obj.insert("v".to_string(), JsonValue::Number(n as f64));
            }
            KvsValue::U32(n) => {
                obj.insert("t".to_string(), JsonValue::String("u32".to_string()));
                obj.insert("v".to_string(), JsonValue::Number(n as f64));
            }
            KvsValue::I64(n) => {
                obj.insert("t".to_string(), JsonValue::String("i64".to_string()));
                obj.insert("v".to_string(), JsonValue::Number(n as f64));
            }
            KvsValue::U64(n) => {
                obj.insert("t".to_string(), JsonValue::String("u64".to_string()));
                obj.insert("v".to_string(), JsonValue::Number(n as f64));
            }
            KvsValue::F64(n) => {
                obj.insert("t".to_string(), JsonValue::String("f64".to_string()));
                obj.insert("v".to_string(), JsonValue::Number(n));
            }
            KvsValue::Boolean(b) => {
                obj.insert("t".to_string(), JsonValue::String("bool".to_string()));
                obj.insert("v".to_string(), JsonValue::Boolean(b));
            }
            KvsValue::String(s) => {
                obj.insert("t".to_string(), JsonValue::String("str".to_string()));
                obj.insert("v".to_string(), JsonValue::String(s));
            }
            KvsValue::Null => {
                obj.insert("t".to_string(), JsonValue::String("null".to_string()));
                obj.insert("v".to_string(), JsonValue::Null);
            }
            KvsValue::Array(arr) => {
                obj.insert("t".to_string(), JsonValue::String("arr".to_string()));
                obj.insert(
                    "v".to_string(),
                    JsonValue::Array(arr.into_iter().map(JsonValue::from).collect()),
                );
            }
            KvsValue::Object(map) => {
                obj.insert("t".to_string(), JsonValue::String("obj".to_string()));
                obj.insert(
                    "v".to_string(),
                    JsonValue::Object(
                        map.into_iter()
                            .map(|(k, v)| (k, JsonValue::from(v)))
                            .collect(),
                    ),
                );
            }
        }
        JsonValue::Object(obj)
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
pub struct JsonBackend {
    instance_id: InstanceId,
    working_dir: PathBuf,
}

impl JsonBackend {
    fn parse(s: &str) -> Result<JsonValue, ErrorCode> {
        s.parse().map_err(ErrorCode::from)
    }

    fn stringify(val: &JsonValue) -> Result<String, ErrorCode> {
        val.stringify().map_err(ErrorCode::from)
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
        for idx in (1..=Self::snapshot_max_count()).rev() {
            let old_snapshot_id = SnapshotId(idx - 1);
            let new_snapshot_id = SnapshotId(idx);

            let hash_path_old = self.hash_file_path(&old_snapshot_id);
            let hash_path_new = self.hash_file_path(&new_snapshot_id);
            let snap_name_old = self.kvs_file_name(&old_snapshot_id);
            let snap_path_old = self.kvs_file_path(&old_snapshot_id);
            let snap_name_new = self.kvs_file_name(&new_snapshot_id);
            let snap_path_new = self.kvs_file_path(&new_snapshot_id);

            println!("rotating: {snap_name_old} -> {snap_name_new}");

            let res = fs::rename(hash_path_old, hash_path_new);
            if let Err(err) = res {
                if err.kind() != std::io::ErrorKind::NotFound {
                    return Err(err.into());
                } else {
                    continue;
                }
            }

            let res = fs::rename(snap_path_old, snap_path_new);
            if let Err(err) = res {
                return Err(err.into());
            }
        }

        Ok(())
    }

    /// Check path have correct extension.
    fn check_extension(path: &Path, extension: &str) -> bool {
        let ext = path.extension();
        ext.is_some_and(|ep| ep.to_str().is_some_and(|es| es == extension))
    }

    /// Load KvsMap from given file.
    fn load(path: &Path, hash_path: Option<&PathBuf>) -> Result<KvsMap, ErrorCode> {
        if !Self::check_extension(path, "json") {
            return Err(ErrorCode::KvsFileReadError);
        }
        if hash_path.is_some_and(|p| !Self::check_extension(p, "hash")) {
            return Err(ErrorCode::KvsHashFileReadError);
        }

        // Load KVS file and parse from string to `JsonValue`.
        let json_str = fs::read_to_string(path)?;
        let json_value = Self::parse(&json_str)?;

        // Perform hash check.
        if let Some(hash_path) = hash_path {
            match fs::read(hash_path) {
                Ok(hash_bytes) => {
                    let hash_kvs = adler32::RollingAdler32::from_buffer(json_str.as_bytes()).hash();
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
                Err(_) => return Err(ErrorCode::KvsHashFileReadError),
            };
        }

        // Cast from `JsonValue` to `KvsValue`.
        let kvs_value = KvsValue::from(json_value);
        if let KvsValue::Object(kvs_map) = kvs_value {
            Ok(kvs_map)
        } else {
            Err(ErrorCode::JsonParserError)
        }
    }

    /// Store KvsMap at given file path.
    fn save(kvs_map: &KvsMap, path: &Path, hash_path: Option<&PathBuf>) -> Result<(), ErrorCode> {
        // Validate extensions.
        if !Self::check_extension(path, "json") {
            return Err(ErrorCode::KvsFileReadError);
        }
        if hash_path.is_some_and(|p| !Self::check_extension(p, "hash")) {
            return Err(ErrorCode::KvsHashFileReadError);
        }

        // Cast from `KvsValue` to `JsonValue`.
        let kvs_value = KvsValue::Object(kvs_map.clone());
        let json_value = JsonValue::from(kvs_value);

        // Stringify `JsonValue` and save to KVS file.
        let json_str = Self::stringify(&json_value)?;
        fs::write(path, &json_str)?;

        // Generate hash and save to hash file.
        if let Some(hash_path) = hash_path {
            let hash = adler32::RollingAdler32::from_buffer(json_str.as_bytes()).hash();
            fs::write(hash_path, hash.to_be_bytes())?
        }

        Ok(())
    }
}

impl KvsBackend for JsonBackend {
    fn load_kvs(&self, snapshot_id: &SnapshotId) -> Result<KvsMap, ErrorCode> {
        let kvs_path = self.kvs_file_path(snapshot_id);
        let hash_path = self.hash_file_path(snapshot_id);
        Self::load(&kvs_path, Some(&hash_path))
    }

    fn load_defaults(&self) -> Result<KvsMap, ErrorCode> {
        let defaults_path = self.defaults_file_path();
        Self::load(&defaults_path, None)
    }

    fn flush(&self, kvs_map: &KvsMap) -> Result<(), ErrorCode> {
        // Rotate previous snapshots.
        self.snapshot_rotate()?;

        // Save new snapshot with snapshot ID = 0.
        let snapshot_id = SnapshotId(0);
        let kvs_path = self.kvs_file_path(&snapshot_id);
        let hash_path = self.hash_file_path(&snapshot_id);
        Self::save(kvs_map, &kvs_path, Some(&hash_path))?;

        Ok(())
    }

    fn snapshot_count(&self) -> usize {
        let mut count = 0;

        for idx in 0..Self::snapshot_max_count() {
            let snapshot_id = SnapshotId(idx);
            let snapshot_path = self.kvs_file_path(&snapshot_id);
            if !snapshot_path.exists() {
                break;
            }

            count += 1;
        }

        count
    }

    fn snapshot_max_count() -> usize {
        KVS_MAX_SNAPSHOTS
    }

    fn snapshot_restore(&self, snapshot_id: &SnapshotId) -> Result<KvsMap, ErrorCode> {
        // Fail if given snapshot ID is for current KVS.
        if *snapshot_id == SnapshotId(0) {
            eprintln!("error: tried to restore current KVS as snapshot");
            return Err(ErrorCode::InvalidSnapshotId);
        }

        // Fail if snapshot doesn't exist.
        if self.snapshot_count() < snapshot_id.0 {
            eprintln!("error: tried to restore a non-existing snapshot");
            return Err(ErrorCode::InvalidSnapshotId);
        }

        self.load_kvs(snapshot_id)
    }
}

/// KVS backend path resolver for `JsonBackend`.
impl KvsPathResolver for JsonBackend {
    fn new(instance_id: InstanceId, working_dir: &Path) -> Self {
        Self {
            instance_id,
            working_dir: working_dir.to_path_buf(),
        }
    }

    fn kvs_file_name(&self, snapshot_id: &SnapshotId) -> String {
        format!("kvs_{}_{snapshot_id}.json", self.instance_id)
    }

    fn kvs_file_path(&self, snapshot_id: &SnapshotId) -> PathBuf {
        self.working_dir.join(self.kvs_file_name(snapshot_id))
    }

    fn hash_file_name(&self, snapshot_id: &SnapshotId) -> String {
        format!("kvs_{}_{snapshot_id}.hash", self.instance_id)
    }

    fn hash_file_path(&self, snapshot_id: &SnapshotId) -> PathBuf {
        self.working_dir.join(self.hash_file_name(snapshot_id))
    }

    fn defaults_file_name(&self) -> String {
        format!("kvs_{}_default.json", self.instance_id)
    }

    fn defaults_file_path(&self) -> PathBuf {
        self.working_dir.join(self.defaults_file_name())
    }
}

#[cfg(test)]
mod json_value_to_kvs_value_conversion_tests {
    use std::collections::HashMap;
    use tinyjson::JsonValue;

    use crate::prelude::{KvsMap, KvsValue};

    #[test]
    fn test_i32_ok() {
        let jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("i32".to_string())),
            ("v".to_string(), JsonValue::Number(-123.0)),
        ]));
        let kv = KvsValue::from(jv);
        assert_eq!(kv, KvsValue::I32(-123));
    }

    #[test]
    fn test_i32_invalid_type() {
        let jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("i32".to_string())),
            ("v".to_string(), JsonValue::String("-123.0".to_string())),
        ]));
        let kv = KvsValue::from(jv);
        assert_eq!(kv, KvsValue::Null);
    }

    #[test]
    fn test_u32_ok() {
        let jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("u32".to_string())),
            ("v".to_string(), JsonValue::Number(123.0)),
        ]));
        let kv = KvsValue::from(jv);
        assert_eq!(kv, KvsValue::U32(123));
    }

    #[test]
    fn test_u32_invalid_type() {
        let jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("u32".to_string())),
            ("v".to_string(), JsonValue::String("123.0".to_string())),
        ]));
        let kv = KvsValue::from(jv);
        assert_eq!(kv, KvsValue::Null);
    }

    #[test]
    fn test_i64_ok() {
        let jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("i64".to_string())),
            ("v".to_string(), JsonValue::Number(-123.0)),
        ]));
        let kv = KvsValue::from(jv);
        assert_eq!(kv, KvsValue::I64(-123));
    }

    #[test]
    fn test_i64_invalid_type() {
        let jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("i64".to_string())),
            ("v".to_string(), JsonValue::String("-123.0".to_string())),
        ]));
        let kv = KvsValue::from(jv);
        assert_eq!(kv, KvsValue::Null);
    }

    #[test]
    fn test_u64_ok() {
        let jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("u64".to_string())),
            ("v".to_string(), JsonValue::Number(123.0)),
        ]));
        let kv = KvsValue::from(jv);
        assert_eq!(kv, KvsValue::U64(123));
    }

    #[test]
    fn test_u64_invalid_type() {
        let jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("u64".to_string())),
            ("v".to_string(), JsonValue::String("123.0".to_string())),
        ]));
        let kv = KvsValue::from(jv);
        assert_eq!(kv, KvsValue::Null);
    }

    #[test]
    fn test_f64_ok() {
        let jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("f64".to_string())),
            ("v".to_string(), JsonValue::Number(-432.1)),
        ]));
        let kv = KvsValue::from(jv);
        assert_eq!(kv, KvsValue::F64(-432.1));
    }

    #[test]
    fn test_f64_invalid_type() {
        let jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("f64".to_string())),
            ("v".to_string(), JsonValue::String("-432.1".to_string())),
        ]));
        let kv = KvsValue::from(jv);
        assert_eq!(kv, KvsValue::Null);
    }

    #[test]
    fn test_bool_ok() {
        let jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("bool".to_string())),
            ("v".to_string(), JsonValue::Boolean(true)),
        ]));
        let kv = KvsValue::from(jv);
        assert_eq!(kv, KvsValue::Boolean(true));
    }

    #[test]
    fn test_bool_invalid_type() {
        let jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("bool".to_string())),
            ("v".to_string(), JsonValue::String("true".to_string())),
        ]));
        let kv = KvsValue::from(jv);
        assert_eq!(kv, KvsValue::Null);
    }

    #[test]
    fn test_string_ok() {
        let jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("str".to_string())),
            ("v".to_string(), JsonValue::String("example".to_string())),
        ]));
        let kv = KvsValue::from(jv);
        assert_eq!(kv, KvsValue::String("example".to_string()));
    }

    #[test]
    fn test_string_invalid_type() {
        let jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("str".to_string())),
            ("v".to_string(), JsonValue::Number(123.4)),
        ]));
        let kv = KvsValue::from(jv);
        assert_eq!(kv, KvsValue::Null);
    }

    #[test]
    fn test_null_ok() {
        let jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("null".to_string())),
            ("v".to_string(), JsonValue::Null),
        ]));
        let kv = KvsValue::from(jv);
        assert_eq!(kv, KvsValue::Null);
    }

    #[test]
    fn test_null_invalid_type() {
        let jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("null".to_string())),
            ("v".to_string(), JsonValue::Number(123.4)),
        ]));
        let kv = KvsValue::from(jv);
        assert_eq!(kv, KvsValue::Null);
    }

    #[test]
    fn test_array_ok() {
        let entry1 = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("i32".to_string())),
            ("v".to_string(), JsonValue::Number(-123.0)),
        ]));
        let entry2 = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("f64".to_string())),
            ("v".to_string(), JsonValue::Number(555.5)),
        ]));
        let jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("arr".to_string())),
            ("v".to_string(), JsonValue::Array(vec![entry1, entry2])),
        ]));
        let kv = KvsValue::from(jv);
        assert_eq!(
            kv,
            KvsValue::Array(vec![KvsValue::I32(-123), KvsValue::F64(555.5)])
        );
    }

    #[test]
    fn test_array_invalid_type() {
        let jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("arr".to_string())),
            ("v".to_string(), JsonValue::String("example".to_string())),
        ]));
        let kv = KvsValue::from(jv);
        assert_eq!(kv, KvsValue::Null);
    }

    #[test]
    fn test_object_ok() {
        let entry1 = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("i32".to_string())),
            ("v".to_string(), JsonValue::Number(-123.0)),
        ]));
        let entry2 = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("f64".to_string())),
            ("v".to_string(), JsonValue::Number(555.5)),
        ]));
        let jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("obj".to_string())),
            (
                "v".to_string(),
                JsonValue::Object(HashMap::from([
                    ("entry1".to_string(), entry1.clone()),
                    ("entry2".to_string(), entry2.clone()),
                ])),
            ),
        ]));
        let kv = KvsValue::from(jv);
        assert_eq!(
            kv,
            KvsValue::Object(KvsMap::from([
                ("entry1".to_string(), KvsValue::from(entry1)),
                ("entry2".to_string(), KvsValue::from(entry2))
            ]))
        );
    }

    #[test]
    fn test_object_invalid_type() {
        let jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("obj".to_string())),
            ("v".to_string(), JsonValue::String("example".to_string())),
        ]));
        let kv = KvsValue::from(jv);
        assert_eq!(kv, KvsValue::Null);
    }

    #[test]
    fn test_non_json_value_object() {
        let jv = JsonValue::Number(123.0);
        let kv = KvsValue::from(jv);
        assert_eq!(kv, KvsValue::Null);
    }
}

#[cfg(test)]
mod kvs_value_to_json_value_conversion_tests {
    use crate::kvs_value::{KvsMap, KvsValue};
    use std::collections::HashMap;
    use tinyjson::JsonValue;

    #[test]
    fn test_i32_ok() {
        let kv = KvsValue::I32(-123);
        let jv = JsonValue::from(kv);

        assert_eq!(
            jv,
            JsonValue::Object(HashMap::from([
                ("t".to_string(), JsonValue::String("i32".to_string())),
                ("v".to_string(), JsonValue::Number(-123.0))
            ]))
        );
    }

    #[test]
    fn test_u32_ok() {
        let kv = KvsValue::U32(123);
        let jv = JsonValue::from(kv);

        assert_eq!(
            jv,
            JsonValue::Object(HashMap::from([
                ("t".to_string(), JsonValue::String("u32".to_string())),
                ("v".to_string(), JsonValue::Number(123.0))
            ]))
        );
    }

    #[test]
    fn test_i64_ok() {
        let kv = KvsValue::I64(-123);
        let jv = JsonValue::from(kv);

        assert_eq!(
            jv,
            JsonValue::Object(HashMap::from([
                ("t".to_string(), JsonValue::String("i64".to_string())),
                ("v".to_string(), JsonValue::Number(-123.0)),
            ]))
        );
    }

    #[test]
    fn test_u64_ok() {
        let kv = KvsValue::U64(123);
        let jv = JsonValue::from(kv);

        assert_eq!(
            jv,
            JsonValue::Object(HashMap::from([
                ("t".to_string(), JsonValue::String("u64".to_string())),
                ("v".to_string(), JsonValue::Number(123.0))
            ]))
        );
    }

    #[test]
    fn test_f64_ok() {
        let kv = KvsValue::F64(-432.1);
        let jv = JsonValue::from(kv);

        assert_eq!(
            jv,
            JsonValue::Object(HashMap::from([
                ("t".to_string(), JsonValue::String("f64".to_string())),
                ("v".to_string(), JsonValue::Number(-432.1)),
            ]))
        );
    }

    #[test]
    fn test_bool_ok() {
        let kv = KvsValue::Boolean(true);
        let jv = JsonValue::from(kv);

        assert_eq!(
            jv,
            JsonValue::Object(HashMap::from([
                ("t".to_string(), JsonValue::String("bool".to_string())),
                ("v".to_string(), JsonValue::Boolean(true)),
            ]))
        );
    }

    #[test]
    fn test_string_ok() {
        let kv = KvsValue::String("example".to_string());
        let jv = JsonValue::from(kv);

        assert_eq!(
            jv,
            JsonValue::Object(HashMap::from([
                ("t".to_string(), JsonValue::String("str".to_string())),
                ("v".to_string(), JsonValue::String("example".to_string())),
            ]))
        );
    }

    #[test]
    fn test_null_ok() {
        let kv = KvsValue::Null;
        let jv = JsonValue::from(kv);

        assert_eq!(
            jv,
            JsonValue::Object(HashMap::from([
                ("t".to_string(), JsonValue::String("null".to_string())),
                ("v".to_string(), JsonValue::Null),
            ]))
        );
    }

    #[test]
    fn test_array_ok() {
        let kv = KvsValue::Array(vec![KvsValue::I32(-123), KvsValue::F64(555.5)]);
        let jv = JsonValue::from(kv);

        let exp_entry1 = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("i32".to_string())),
            ("v".to_string(), JsonValue::Number(-123.0)),
        ]));
        let exp_entry2 = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("f64".to_string())),
            ("v".to_string(), JsonValue::Number(555.5)),
        ]));
        let exp_jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("arr".to_string())),
            (
                "v".to_string(),
                JsonValue::Array(vec![exp_entry1, exp_entry2]),
            ),
        ]));
        assert_eq!(jv, exp_jv);
    }

    #[test]
    fn test_object_ok() {
        let entry1 = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("i32".to_string())),
            ("v".to_string(), JsonValue::Number(-123.0)),
        ]));
        let entry2 = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("f64".to_string())),
            ("v".to_string(), JsonValue::Number(555.5)),
        ]));

        let kv = KvsValue::Object(KvsMap::from([
            ("entry1".to_string(), KvsValue::from(entry1.clone())),
            ("entry2".to_string(), KvsValue::from(entry2.clone())),
        ]));
        let jv = JsonValue::from(kv);

        let exp_jv = JsonValue::from(HashMap::from([
            ("t".to_string(), JsonValue::String("obj".to_string())),
            (
                "v".to_string(),
                JsonValue::Object(HashMap::from([
                    ("entry1".to_string(), entry1),
                    ("entry2".to_string(), entry2),
                ])),
            ),
        ]));
        assert_eq!(jv, exp_jv);
    }
}

#[cfg(test)]
mod error_code_tests {
    use crate::error_code::ErrorCode;
    use tinyjson::JsonValue;

    #[test]
    fn test_from_json_parse_error_to_json_parser_error() {
        let error = tinyjson::JsonParser::new("[1, 2, 3".chars())
            .parse()
            .unwrap_err();
        assert_eq!(ErrorCode::from(error), ErrorCode::JsonParserError);
    }

    #[test]
    fn test_from_json_generate_error_to_json_generate_error() {
        let data: JsonValue = JsonValue::Number(f64::INFINITY);
        let error = data.stringify().unwrap_err();
        assert_eq!(ErrorCode::from(error), ErrorCode::JsonGeneratorError);
    }
}

#[cfg(test)]
mod backend_tests {
    use crate::error_code::ErrorCode;
    use crate::json_backend::JsonBackend;
    use crate::kvs_value::{KvsMap, KvsValue};
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    fn create_kvs_files(working_dir: &Path) -> (PathBuf, PathBuf) {
        let kvs_map = KvsMap::from([
            ("k1".to_string(), KvsValue::from("v1")),
            ("k2".to_string(), KvsValue::from(true)),
            ("k3".to_string(), KvsValue::from(123.4)),
        ]);
        let kvs_path = working_dir.join("kvs.json");
        let hash_path = working_dir.join("kvs.hash");
        JsonBackend::save(&kvs_map, &kvs_path, Some(&hash_path)).unwrap();
        (kvs_path, hash_path)
    }

    #[test]
    fn test_load_kvs_ok() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let (kvs_path, _hash_path) = create_kvs_files(&dir_path);

        let kvs_map = JsonBackend::load(&kvs_path, None).unwrap();
        assert_eq!(kvs_map.len(), 3);
    }

    #[test]
    fn test_load_kvs_not_found() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs_path = dir_path.join("kvs.json");

        assert!(JsonBackend::load(&kvs_path, None).is_err_and(|e| e == ErrorCode::FileNotFound));
    }

    #[test]
    fn test_load_kvs_invalid_extension() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs_path = dir_path.join("kvs.invalid_ext");

        assert!(JsonBackend::load(&kvs_path, None).is_err_and(|e| e == ErrorCode::KvsFileReadError));
    }

    #[test]
    fn test_load_kvs_malformed_json() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs_path = dir_path.join("kvs.json");
        std::fs::write(kvs_path.clone(), "{\"malformed_json\"}").unwrap();

        assert!(JsonBackend::load(&kvs_path, None).is_err_and(|e| e == ErrorCode::JsonParserError));
    }

    #[test]
    fn test_load_kvs_invalid_data() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs_path = dir_path.join("kvs.json");
        std::fs::write(kvs_path.clone(), "[123.4, 567.8]").unwrap();

        assert!(JsonBackend::load(&kvs_path, None).is_err_and(|e| e == ErrorCode::JsonParserError));
    }

    #[test]
    fn test_load_kvs_hash_path_some_ok() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let (kvs_path, hash_path) = create_kvs_files(&dir_path);

        let kvs_map = JsonBackend::load(&kvs_path, Some(&hash_path)).unwrap();
        assert_eq!(kvs_map.len(), 3);
    }

    #[test]
    fn test_load_kvs_hash_path_some_invalid_extension() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let (kvs_path, hash_path) = create_kvs_files(&dir_path);
        let new_hash_path = hash_path.with_extension("invalid_ext");
        std::fs::rename(hash_path, new_hash_path.clone()).unwrap();

        assert!(JsonBackend::load(&kvs_path, Some(&new_hash_path))
            .is_err_and(|e| e == ErrorCode::KvsHashFileReadError));
    }

    #[test]
    fn test_load_kvs_hash_path_some_not_found() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let (kvs_path, hash_path) = create_kvs_files(&dir_path);
        std::fs::remove_file(hash_path.clone()).unwrap();

        assert!(JsonBackend::load(&kvs_path, Some(&hash_path))
            .is_err_and(|e| e == ErrorCode::KvsHashFileReadError));
    }

    #[test]
    fn test_load_kvs_invalid_hash_content() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let (kvs_path, hash_path) = create_kvs_files(&dir_path);
        std::fs::write(hash_path.clone(), vec![0x12, 0x34, 0x56, 0x78]).unwrap();

        assert!(JsonBackend::load(&kvs_path, Some(&hash_path))
            .is_err_and(|e| e == ErrorCode::ValidationFailed));
    }

    #[test]
    fn test_load_kvs_invalid_hash_len() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let (kvs_path, hash_path) = create_kvs_files(&dir_path);
        std::fs::write(hash_path.clone(), vec![0x12, 0x34, 0x56]).unwrap();

        assert!(JsonBackend::load(&kvs_path, Some(&hash_path))
            .is_err_and(|e| e == ErrorCode::ValidationFailed));
    }

    #[test]
    fn test_save_kvs_ok() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();

        let kvs_map = KvsMap::from([
            ("k1".to_string(), KvsValue::from("v1")),
            ("k2".to_string(), KvsValue::from(true)),
            ("k3".to_string(), KvsValue::from(123.4)),
        ]);
        let kvs_path = dir_path.join("kvs.json");
        JsonBackend::save(&kvs_map, &kvs_path, None).unwrap();

        assert!(kvs_path.exists());
    }

    #[test]
    fn test_save_kvs_invalid_extension() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();

        let kvs_map = KvsMap::new();
        let kvs_path = dir_path.join("kvs.invalid_ext");
        assert!(JsonBackend::save(&kvs_map, &kvs_path, None)
            .is_err_and(|e| e == ErrorCode::KvsFileReadError));
    }

    #[test]
    fn test_save_kvs_hash_path_some_ok() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();

        let kvs_map = KvsMap::from([
            ("k1".to_string(), KvsValue::from("v1")),
            ("k2".to_string(), KvsValue::from(true)),
            ("k3".to_string(), KvsValue::from(123.4)),
        ]);
        let kvs_path = dir_path.join("kvs.json");
        let hash_path = dir_path.join("kvs.hash");
        JsonBackend::save(&kvs_map, &kvs_path, Some(&hash_path)).unwrap();

        assert!(kvs_path.exists());
        assert!(hash_path.exists());
    }

    #[test]
    fn test_save_kvs_hash_path_some_invalid_extension() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();

        let kvs_map = KvsMap::new();
        let kvs_path = dir_path.join("kvs.json");
        let hash_path = dir_path.join("kvs.invalid_ext");
        assert!(JsonBackend::save(&kvs_map, &kvs_path, Some(&hash_path))
            .is_err_and(|e| e == ErrorCode::KvsHashFileReadError));
    }

    #[test]
    fn test_save_kvs_impossible_str() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();

        let kvs_map = KvsMap::from([("inf".to_string(), KvsValue::from(f64::INFINITY))]);
        let kvs_path = dir_path.join("kvs.json");
        assert!(JsonBackend::save(&kvs_map, &kvs_path, None)
            .is_err_and(|e| e == ErrorCode::JsonGeneratorError));
    }
}

#[cfg(test)]
mod path_resolver_tests {
    use crate::json_backend::JsonBackend;
    use crate::kvs_api::{InstanceId, SnapshotId};
    use crate::kvs_backend::KvsPathResolver;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_kvs_file_name() {
        let instance_id = InstanceId(123);
        let snapshot_id = SnapshotId(2);
        let backend = JsonBackend::new(instance_id.clone(), &PathBuf::new());
        let exp_name = format!("kvs_{instance_id}_{snapshot_id}.json");
        let act_name = backend.kvs_file_name(&snapshot_id);
        assert_eq!(exp_name, act_name);
    }

    #[test]
    fn test_kvs_file_path() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path();

        let instance_id = InstanceId(123);
        let snapshot_id = SnapshotId(2);
        let backend = JsonBackend::new(instance_id.clone(), dir_path);
        let exp_name = dir_path.join(format!("kvs_{instance_id}_{snapshot_id}.json"));
        let act_name = backend.kvs_file_path(&snapshot_id);
        assert_eq!(exp_name, act_name);
    }
    #[test]
    fn test_hash_file_name() {
        let instance_id = InstanceId(123);
        let snapshot_id = SnapshotId(2);
        let backend = JsonBackend::new(instance_id.clone(), &PathBuf::new());
        let exp_name = format!("kvs_{instance_id}_{snapshot_id}.hash");
        let act_name = backend.hash_file_name(&snapshot_id);
        assert_eq!(exp_name, act_name);
    }

    #[test]
    fn test_hash_file_path() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path();

        let instance_id = InstanceId(123);
        let snapshot_id = SnapshotId(2);
        let backend = JsonBackend::new(instance_id.clone(), dir_path);
        let exp_name = dir_path.join(format!("kvs_{instance_id}_{snapshot_id}.hash"));
        let act_name = backend.hash_file_path(&snapshot_id);
        assert_eq!(exp_name, act_name);
    }

    #[test]
    fn test_defaults_file_name() {
        let instance_id = InstanceId(123);
        let backend = JsonBackend::new(instance_id.clone(), &PathBuf::new());
        let exp_name = format!("kvs_{instance_id}_default.json");
        let act_name = backend.defaults_file_name();
        assert_eq!(exp_name, act_name);
    }

    #[test]
    fn test_defaults_file_path() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path();

        let instance_id = InstanceId(123);
        let backend = JsonBackend::new(instance_id.clone(), dir_path);
        let exp_name = dir_path.join(format!("kvs_{instance_id}_default.json"));
        let act_name = backend.defaults_file_path();
        assert_eq!(exp_name, act_name);
    }
}
