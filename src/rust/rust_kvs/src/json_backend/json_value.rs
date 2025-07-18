// Abstraction for JSON handling in KVS
// This module provides a trait for JSON operations and an implementation using tinyjson

use std::collections::HashMap;

use crate::kvs_backend::KvsBackendError;

pub type JsonValue = tinyjson::JsonValue;

// Implementation for tinyjson
use tinyjson::{JsonGenerateError, JsonParseError};

#[derive(Default)]
pub struct TinyJson;

impl KvsJson for TinyJson {
    type Value = JsonValue;
    fn parse(s: &str) -> Result<Self::Value, KvsBackendError> {
        s.parse()
            .map_err(|e: JsonParseError| KvsBackendError::Json(format!("parse error: {e:?}")))
    }
    fn stringify(val: &Self::Value) -> Result<String, KvsBackendError> {
        val.stringify().map_err(|e: JsonGenerateError| {
            KvsBackendError::Json(format!("stringify: {}", e.message()))
        })
    }
    fn get_object(val: &Self::Value) -> Option<&HashMap<String, Self::Value>> {
        val.get::<HashMap<String, JsonValue>>()
    }
    fn get_array(val: &Self::Value) -> Option<&Vec<Self::Value>> {
        val.get::<Vec<JsonValue>>()
    }
    fn get_f64(val: &Self::Value) -> Option<f64> {
        val.get::<f64>().copied()
    }
    fn get_bool(val: &Self::Value) -> Option<bool> {
        val.get::<bool>().copied()
    }
    fn get_string(val: &Self::Value) -> Option<&str> {
        val.get::<String>().map(|s| s.as_str())
    }
    fn is_null(val: &Self::Value) -> bool {
        matches!(val, JsonValue::Null)
    }
    fn to_kvs_value(val: Self::Value) -> KvsValue {
        KvsValue::from(&val)
    }
    fn from_kvs_value(val: &KvsValue) -> Self::Value {
        JsonValue::from(val)
    }
}

// Conversion between KvsValue and JsonValue
use crate::kvs_value::KvsValue;

pub trait KvsJson {
    type Value;
    fn parse(s: &str) -> Result<Self::Value, KvsBackendError>;
    fn stringify(val: &Self::Value) -> Result<String, KvsBackendError>;
    fn get_object(val: &Self::Value) -> Option<&HashMap<String, Self::Value>>;
    fn get_array(val: &Self::Value) -> Option<&Vec<Self::Value>>;
    fn get_f64(val: &Self::Value) -> Option<f64>;
    fn get_bool(val: &Self::Value) -> Option<bool>;
    fn get_string(val: &Self::Value) -> Option<&str>;
    fn is_null(val: &Self::Value) -> bool;
    fn to_kvs_value(val: Self::Value) -> KvsValue;
    /// Convert a &KvsValue into the backend's JSON value type
    fn from_kvs_value(val: &KvsValue) -> Self::Value;
}
