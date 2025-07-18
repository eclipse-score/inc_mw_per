//! Common test utilities.

use rust_kvs::kvs_value::KvsValue;
use std::iter::zip;

/// Compare `KvsValue` objects.
///
/// # Parameters
///   * `left`: left value
///   * `right`: right value
///
/// # Return Value
///   * `true` if provided values are same.
pub fn compare_kvs_values(left: &KvsValue, right: &KvsValue) -> bool {
    match (left, right) {
        (KvsValue::I32(i32_val_l), KvsValue::I32(i32_val_r)) => i32_val_l == i32_val_r,
        (KvsValue::U32(u32_val_l), KvsValue::U32(u32_val_r)) => u32_val_l == u32_val_r,
        (KvsValue::I64(i64_val_l), KvsValue::I64(i64_val_r)) => i64_val_l == i64_val_r,
        (KvsValue::U64(u64_val_l), KvsValue::U64(u64_val_r)) => u64_val_l == u64_val_r,
        (KvsValue::F64(f64_val_l), KvsValue::F64(f64_val_r)) => f64_val_l == f64_val_r,
        (KvsValue::Boolean(bool_val_l), KvsValue::Boolean(bool_val_r)) => bool_val_l == bool_val_r,
        (KvsValue::String(string_val_l), KvsValue::String(string_val_r)) => {
            string_val_l == string_val_r
        }
        (KvsValue::Null, KvsValue::Null) => true,
        (KvsValue::Array(array_l), KvsValue::Array(array_r)) => {
            if array_l.len() != array_r.len() {
                return false;
            }
            for (elem_l, elem_r) in zip(array_l, array_r) {
                if !compare_kvs_values(elem_l, elem_r) {
                    return false;
                }
            }
            true
        }
        (KvsValue::Object(object_l), KvsValue::Object(object_r)) => {
            if object_l.len() != object_r.len() {
                return false;
            }
            if object_l.keys().ne(object_r.keys()) {
                return false;
            }
            for k in object_l.keys() {
                if !compare_kvs_values(&object_l[k], &object_r[k]) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}
