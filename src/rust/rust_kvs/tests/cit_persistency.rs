//! Persistency tests.
//!
//! Requirements verified:
//! - Persistency (feat_req__persistency__persistency)
//! The KVS system shall persist stored data and provide an API to explicitly trigger persistence.
//! - Store persistent data (feat_req__persistency__persist_data)
//! The KVS shall support storing and loading its data to and from persistent storage.

use rust_kvs::{ErrorCode, InstanceId, Kvs, KvsApi, KvsValue, OpenNeedDefaults, OpenNeedKvs};
use std::collections::HashMap;
use std::iter::zip;
use tempfile::tempdir;

fn cmp_match(left: &KvsValue, right: &KvsValue) -> bool {
    return match (left, right) {
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
                if !cmp_match(lv, rv) {
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
                if !cmp_match(&l[k], &r[k]) {
                    return false;
                }
            }

            true
        }
        (_, _) => false,
    };
}

fn cmp_object(left: HashMap<String, KvsValue>, right: HashMap<String, KvsValue>) -> bool {
    cmp_match(&KvsValue::from(left), &KvsValue::from(right))
}

fn cmp_array(left: Vec<KvsValue>, right: Vec<KvsValue>) -> bool {
    cmp_match(&KvsValue::from(left), &KvsValue::from(right))
}

/// Flush on exit is enabled by default.
/// Data will be flushed on `kvs` being dropped.
#[test]
fn cit_persistency_flush_on_exit_enabled() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_path = dir.path().to_path_buf();

    // Values.
    let kv_number = ("number", 123.4);
    let kv_bool = ("bool", true);
    let kv_str = ("str", "abcd".to_string());
    let kv_null = ("null", ());
    let hashmap = HashMap::from([("sub-number".to_string(), KvsValue::from(789.0))]);
    let kv_obj = ("obj", hashmap.clone());
    let array = vec![
        KvsValue::from(321.0),
        KvsValue::from(false),
        KvsValue::from("dbca".to_string()),
        KvsValue::from(()),
        KvsValue::from(vec![]),
        KvsValue::from(hashmap),
    ];
    let kv_array = ("array", array);

    {
        // First KVS run.
        let kvs = Kvs::open(
            InstanceId::new(0),
            dir_path.clone(),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
        )?;

        // Set value of each type.
        kvs.set_value(kv_number.0, kv_number.1)?;
        kvs.set_value(kv_bool.0, kv_bool.1)?;
        kvs.set_value(kv_str.0, kv_str.1.clone())?;
        kvs.set_value(kv_null.0, kv_null.1)?;
        kvs.set_value(kv_obj.0, kv_obj.1.clone())?;
        kvs.set_value(kv_array.0, kv_array.1.clone())?;
    }

    // Assertions.
    {
        // Second KVS run.
        // KVS file is expected to exist.
        let kvs = Kvs::open(
            InstanceId::new(0),
            dir_path,
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Required,
        )?;

        // Compare each value.
        assert_eq!(kvs.get_value::<f64>(kv_number.0)?, kv_number.1);
        assert_eq!(kvs.get_value::<bool>(kv_bool.0)?, kv_bool.1);
        assert_eq!(kvs.get_value::<String>(kv_str.0)?, kv_str.1);
        assert_eq!(kvs.get_value::<()>(kv_null.0)?, kv_null.1);
        assert!(cmp_object(
            kvs.get_value::<HashMap<String, KvsValue>>(kv_obj.0)?,
            kv_obj.1
        ));
        assert!(cmp_array(
            kvs.get_value::<Vec<KvsValue>>(kv_array.0)?,
            kv_array.1
        ));
    }

    Ok(())
}

#[test]
fn cit_persistency_flush_on_exit_disabled_drop_data() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_path = dir.path().to_path_buf();

    // Values.
    let kv_number = ("number", 123.4);
    let kv_bool = ("bool", true);
    let kv_str = ("str", "abcd".to_string());
    let kv_null = ("null", ());
    let hashmap = HashMap::from([("sub-number".to_string(), KvsValue::from(789.0))]);
    let kv_obj = ("obj", hashmap.clone());
    let array = vec![
        KvsValue::from(321.0),
        KvsValue::from(false),
        KvsValue::from("dbca".to_string()),
        KvsValue::from(()),
        KvsValue::from(vec![]),
        KvsValue::from(hashmap),
    ];
    let kv_array = ("array", array);

    {
        // First KVS run.
        let kvs = Kvs::open(
            InstanceId::new(0),
            dir_path.clone(),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
        )?;
        kvs.flush_on_exit(false);

        // Set value of each type.
        kvs.set_value(kv_number.0, kv_number.1)?;
        kvs.set_value(kv_bool.0, kv_bool.1)?;
        kvs.set_value(kv_str.0, kv_str.1.clone())?;
        kvs.set_value(kv_null.0, kv_null.1)?;
        kvs.set_value(kv_obj.0, kv_obj.1.clone())?;
        kvs.set_value(kv_array.0, kv_array.1.clone())?;
    }

    // Assertions.
    {
        // Second KVS run.
        // KVS file is expected to not to exist.
        let kvs = Kvs::open(
            InstanceId::new(0),
            dir_path,
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
        )?;

        // Make sure no keys are defined.
        assert!(kvs.get_all_keys()?.is_empty());
    }

    Ok(())
}

#[test]
fn cit_persistency_flush_on_exit_disabled_manual_flush() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_path = dir.path().to_path_buf();

    // Values.
    let kv_number = ("number", 123.4);
    let kv_bool = ("bool", true);
    let kv_str = ("str", "abcd".to_string());
    let kv_null = ("null", ());
    let hashmap = HashMap::from([("sub-number".to_string(), KvsValue::from(789.0))]);
    let kv_obj = ("obj", hashmap.clone());
    let array = vec![
        KvsValue::from(321.0),
        KvsValue::from(false),
        KvsValue::from("dbca".to_string()),
        KvsValue::from(()),
        KvsValue::from(vec![]),
        KvsValue::from(hashmap),
    ];
    let kv_array = ("array", array);

    {
        // First KVS run.
        let kvs = Kvs::open(
            InstanceId::new(0),
            dir_path.clone(),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
        )?;
        kvs.flush_on_exit(false);

        // Set value of each type.
        kvs.set_value(kv_number.0, kv_number.1)?;
        kvs.set_value(kv_bool.0, kv_bool.1)?;
        kvs.set_value(kv_str.0, kv_str.1.clone())?;
        kvs.set_value(kv_null.0, kv_null.1)?;
        kvs.set_value(kv_obj.0, kv_obj.1.clone())?;
        kvs.set_value(kv_array.0, kv_array.1.clone())?;

        // Explicitly flush.
        kvs.flush()?;
    }

    // Assertions.
    {
        // Second KVS run.
        // KVS file is expected to exist.
        let kvs = Kvs::open(
            InstanceId::new(0),
            dir_path,
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Required,
        )?;

        // Compare each value.
        assert_eq!(kvs.get_value::<f64>(kv_number.0)?, kv_number.1);
        assert_eq!(kvs.get_value::<bool>(kv_bool.0)?, kv_bool.1);
        assert_eq!(kvs.get_value::<String>(kv_str.0)?, kv_str.1);
        assert_eq!(kvs.get_value::<()>(kv_null.0)?, kv_null.1);
        assert!(cmp_object(
            kvs.get_value::<HashMap<String, KvsValue>>(kv_obj.0)?,
            kv_obj.1
        ));
        assert!(cmp_array(
            kvs.get_value::<Vec<KvsValue>>(kv_array.0)?,
            kv_array.1
        ));
    }

    Ok(())
}
