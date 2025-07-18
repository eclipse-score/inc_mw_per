//! Serialization and deserialization logic for KvsValue <-> JsonValue

use super::json_value::JsonValue;
use crate::kvs_value::KvsValue;
use std::collections::HashMap;

impl From<&JsonValue> for KvsValue {
    fn from(val: &JsonValue) -> KvsValue {
        match val {
            JsonValue::Object(obj) => {
                // Type-tagged: { "type": ..., "value": ... }
                if let (Some(JsonValue::String(type_str)), Some(value)) =
                    (obj.get("type"), obj.get("value"))
                {
                    match type_str.as_str() {
                        "I32" => {
                            if let JsonValue::Number(num) = value {
                                return KvsValue::I32(*num as i32);
                            } else {
                                return KvsValue::Null; // type mismatch
                            }
                        }
                        "U32" => {
                            if let JsonValue::Number(num) = value {
                                return KvsValue::U32(*num as u32);
                            } else {
                                return KvsValue::Null;
                            }
                        }
                        "I64" => {
                            if let JsonValue::Number(num) = value {
                                return KvsValue::I64(*num as i64);
                            } else {
                                return KvsValue::Null;
                            }
                        }
                        "U64" => {
                            if let JsonValue::Number(num) = value {
                                return KvsValue::U64(*num as u64);
                            } else {
                                return KvsValue::Null;
                            }
                        }
                        "F64" => {
                            if let JsonValue::Number(num) = value {
                                return KvsValue::F64(*num);
                            } else {
                                return KvsValue::Null;
                            }
                        }
                        "Boolean" => {
                            if let JsonValue::Boolean(bv) = value {
                                return KvsValue::Boolean(*bv);
                            } else {
                                return KvsValue::Null;
                            }
                        }
                        "String" => {
                            if let JsonValue::String(sv) = value {
                                return KvsValue::String(sv.clone());
                            } else {
                                return KvsValue::Null;
                            }
                        }
                        "Null" => {
                            if let JsonValue::Null = value {
                                return KvsValue::Null;
                            } else {
                                return KvsValue::Null;
                            }
                        }
                        "Array" => {
                            if let JsonValue::Array(vec) = value {
                                return KvsValue::Array(vec.iter().map(KvsValue::from).collect());
                            } else {
                                return KvsValue::Null;
                            }
                        }
                        "Object" => {
                            if let JsonValue::Object(hm) = value {
                                return KvsValue::Object(
                                    hm.iter()
                                        .map(|(k, v)| (k.clone(), KvsValue::from(v)))
                                        .collect(),
                                );
                            } else {
                                return KvsValue::Null;
                            }
                        }
                        _ => {
                            return KvsValue::Null;
                        }
                    }
                }
                // fallback: treat as object of kvs values
                KvsValue::Object(
                    obj.iter()
                        .map(|(k, v)| (k.clone(), KvsValue::from(v)))
                        .collect(),
                )
            }
            JsonValue::Number(n) => KvsValue::F64(*n),
            JsonValue::Boolean(b) => KvsValue::Boolean(*b),
            JsonValue::String(s) => KvsValue::String(s.clone()),
            JsonValue::Null => KvsValue::Null,
            JsonValue::Array(arr) => KvsValue::Array(arr.iter().map(KvsValue::from).collect()),
        }
    }
}

impl From<&KvsValue> for JsonValue {
    fn from(val: &KvsValue) -> JsonValue {
        let mut obj = HashMap::new();
        match val {
            KvsValue::I32(n) => {
                obj.insert("type".to_string(), JsonValue::String("I32".to_string()));
                obj.insert("value".to_string(), JsonValue::Number(*n as f64));
            }
            KvsValue::U32(n) => {
                obj.insert("type".to_string(), JsonValue::String("U32".to_string()));
                obj.insert("value".to_string(), JsonValue::Number(*n as f64));
            }
            KvsValue::I64(n) => {
                obj.insert("type".to_string(), JsonValue::String("I64".to_string()));
                obj.insert("value".to_string(), JsonValue::Number(*n as f64));
            }
            KvsValue::U64(n) => {
                obj.insert("type".to_string(), JsonValue::String("U64".to_string()));
                obj.insert("value".to_string(), JsonValue::Number(*n as f64));
            }
            KvsValue::F64(n) => {
                obj.insert("type".to_string(), JsonValue::String("F64".to_string()));
                obj.insert("value".to_string(), JsonValue::Number(*n));
            }
            KvsValue::Boolean(b) => {
                obj.insert("type".to_string(), JsonValue::String("Boolean".to_string()));
                obj.insert("value".to_string(), JsonValue::Boolean(*b));
            }
            KvsValue::String(s) => {
                obj.insert("type".to_string(), JsonValue::String("String".to_string()));
                obj.insert("value".to_string(), JsonValue::String(s.clone()));
            }
            KvsValue::Null => {
                obj.insert("type".to_string(), JsonValue::String("Null".to_string()));
                obj.insert("value".to_string(), JsonValue::Null);
            }
            KvsValue::Array(arr) => {
                obj.insert("type".to_string(), JsonValue::String("Array".to_string()));
                obj.insert(
                    "value".to_string(),
                    JsonValue::Array(arr.iter().map(JsonValue::from).collect()),
                );
            }
            KvsValue::Object(map) => {
                obj.insert("type".to_string(), JsonValue::String("Object".to_string()));
                obj.insert(
                    "value".to_string(),
                    JsonValue::Object(
                        map.iter()
                            .map(|(k, v)| (k.clone(), JsonValue::from(v)))
                            .collect(),
                    ),
                );
            }
        }
        JsonValue::Object(obj)
    }
}
