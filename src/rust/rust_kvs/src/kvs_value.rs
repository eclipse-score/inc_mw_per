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

use std::collections::HashMap;
use std::ops::Index;

/// Key-value storage map type
pub type KvsMap = std::collections::HashMap<String, KvsValue>;

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

use std::convert::TryFrom;

macro_rules! impl_tryfrom_kvs_value_to_t {
    ($to:ty, $item:ident) => {
        impl<'a> TryFrom<&'a KvsValue> for $to {
            type Error = ();
            fn try_from(val: &'a KvsValue) -> Result<$to, Self::Error> {
                if let KvsValue::$item(ref v) = val {
                    Ok(v.clone())
                } else {
                    Err(())
                }
            }
        }
    };
}

impl_tryfrom_kvs_value_to_t!(f64, Number);
impl_tryfrom_kvs_value_to_t!(bool, Boolean);
impl_tryfrom_kvs_value_to_t!(String, String);
impl_tryfrom_kvs_value_to_t!(Vec<KvsValue>, Array);
impl_tryfrom_kvs_value_to_t!(HashMap<String, KvsValue>, Object);

impl<'a> TryFrom<&'a KvsValue> for () {
    type Error = ();
    fn try_from(val: &'a KvsValue) -> Result<(), Self::Error> {
        if let KvsValue::Null = val {
            Ok(())
        } else {
            Err(())
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

    #[test]
    fn test_get_inner_value() {
        let value = KvsValue::Number(42.0);
        let inner = f64::get_inner_value(&value);
        assert_eq!(inner, Some(&42.0), "Expected to get inner f64 value");
    }
}
