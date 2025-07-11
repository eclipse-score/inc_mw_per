//! Common test utilities.

use rust_kvs::KvsValue;
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
        (KvsValue::Number(l), KvsValue::Number(r)) => l == r,
        (KvsValue::Boolean(l), KvsValue::Boolean(r)) => l == r,
        (KvsValue::String(l), KvsValue::String(r)) => l == r,
        (KvsValue::Null, KvsValue::Null) => true,
        (KvsValue::Array(l), KvsValue::Array(r)) => {
            // Check size.
            if l.len() != r.len() {
                return false;
            }

            // Iterate over elements.
            for (lv, rv) in zip(l, r) {
                if !compare_kvs_values(lv, rv) {
                    return false;
                }
            }

            true
        }
        (KvsValue::Object(l), KvsValue::Object(r)) => {
            // Check size.
            if l.len() != r.len() {
                return false;
            }

            // Check keys.
            if l.keys().ne(r.keys()) {
                return false;
            }

            // Iterate over elements.
            let keys = l.keys();
            for k in keys {
                if !compare_kvs_values(&l[k], &r[k]) {
                    return false;
                }
            }

            true
        }
        (_, _) => false,
    }
}
