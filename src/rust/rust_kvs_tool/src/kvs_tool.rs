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

//! # Key-Value Handling Command Line Tool based on the KVS API
//!
//! ## Introduction
//!
//! This Command Line Tool provides Key-Value using the KVS API (`FEAT_REQ__KVS__tooling`).
//! For Command Line Argument parsing the crate pico_args [pico_args](https://docs.rs/pico-args/latest/pico_args/) is used.
//! For JSON parsing the crate tinyjson [tinyjson](https://docs.rs/tinyjson/latest/tinyjson/) is used, which is also used in the KVS itself.
//! No other direct dependencies are used besides the Rust `std` library.
//!
//! ## Usage
//!
//! ```text
//!
//!    Options:
//!    -h, --help          Show this help message and exit
//!    -o, --operation     Specify the operation to perform (setkey, getkey, removekey, listkeys, reset, snapshotcount, snapshotmaxcount, snapshotrestore, getkvsfilename, gethashfilename, createtestdata)
//!    -k, --key           Specify the key to operate on (for key operations)
//!    -p, --payload       Specify the value to write (for set operations)
//!    -s, --snapshotid    Specify the snapshot ID for Snapshot operations
//!    -d, --directory     Specify the directory of the Key-Files (default is current directory)
//!
//!    ---------------------------------------
//!
//!    Usage Examples:
//!
//!    Read a Key and show value:
//!        kvs_tool -o getkey -k MyKey
//!
//!    Write a Key and use the <payload> as the data source:
//!        kvs_tool -o setkey  -k MyKey -p 'Hello World' (automatically detects following types: Number, Boolean, String, Null, Object, Array)
//!        kvs_tool -o setkey  -k MyKey -p 'true'
//!        kvs_tool -o setkey  -k MyKey -p 15
//!        kvs_tool -o setkey  -k MyKey -p '[456,false,"Second"]'
//!        kvs_tool -o setkey  -k MyKey -p '{"sub-number":789,"sub-array":[1246,false,"Fourth"]}'
//!
//!    Delete a key:
//!        kvs_tool -o removekey -k MyKey
//!
//!    List Keys:
//!        kvs_tool -o listkeys
//!
//!    Reset KVS:
//!        kvs_tool -o reset
//!
//!    Snapshot Count:
//!        kvs_tool -o snapshotcount
//!
//!    Snapshot Restore:
//!        kvs_tool -o snapshotrestore -s 1
//!
//!    Get KVS Filename:
//!        kvs_tool -o getkvsfilename -s 1
//!
//!    Get Hash Filename:
//!        kvs_tool -o gethashfilename -s 1
//!
//!    ---------------------------------------
//!
//!    Create Test Data:
//!        kvs_tool -o createtestdata (Creates Data provided by the example code in the KVS API)
//!
//! ```
//!

use pico_args::Arguments;
use rust_kvs::prelude::*;
use std::collections::HashMap;
use tinyjson::JsonValue;

/// Defines the available operation modes for key and file management.
enum OperationMode {
    Invalid,
    SetKey,
    GetKey,
    RemoveKey,
    ListKeys,
    Reset,
    SnapshotCount,
    SnapshotMaxCount,
    SnapshotRestore,
    GetKvsFilename,
    GetHashFilename,
    CreateTestData,
}

/// Converts a TinyJSON value to a KVS value.
fn from_tinyjson(value: &JsonValue) -> KvsValue {
    match value {
        JsonValue::Number(n) => KvsValue::F64(*n),
        JsonValue::Boolean(b) => KvsValue::Boolean(*b),
        JsonValue::String(s) => KvsValue::String(s.clone()),
        JsonValue::Null => KvsValue::Null,
        JsonValue::Array(arr) => {
            let v = arr.iter().map(from_tinyjson).collect();
            KvsValue::Array(v)
        }
        JsonValue::Object(obj) => {
            let map = obj
                .iter()
                .map(|(k, v)| (k.clone(), from_tinyjson(v)))
                .collect();
            KvsValue::Object(map)
        }
    }
}

/// Gets the key-value pair from the KVS and prints it to the console.
/// This function checks if the key exists and if it is a default value.
/// It also prints the default value.
fn _getkey(kvs: Kvs, mut args: Arguments) -> Result<(), ErrorCode> {
    println!("----------------------");
    kvs.set_flush_on_exit(FlushOnExit::No);

    let key: String = match args.opt_value_from_str("--key") {
        Ok(Some(val)) => val,
        Ok(None) | Err(_) => match args.opt_value_from_str("-k") {
            Ok(Some(val)) => val,
            _ => {
                eprintln!("Error: Key (-k or --key) needs to be specified!");
                return Err(ErrorCode::UnmappedError);
            }
        },
    };
    println!("Read Key {}", &key);

    let key_exist = kvs.key_exists(&key).map_err(|e| {
        eprintln!("KVS get:key_exists failed: {e:?}");
        e
    })?;

    let is_default = kvs.is_value_default(&key).map_err(|e| {
        eprintln!("KVS get:is_value_default failed: {e:?}");
        e
    })?;

    if key_exist {
        println!("Key '{key}' exists!");
        match kvs.get_value(&key) {
            Ok(value) => {
                println!("Key Value: {value:?}");
            }
            Err(e) => {
                eprintln!("Get Key Error: {e:?}");
            }
        };
    } else {
        println!("Key '{key}' does not exist!");
        if is_default {
            println!("Key is default value!");
        } else {
            println!("Key is not default value!");
            return Err(ErrorCode::KeyNotFound);
        }
    }

    match kvs.get_default_value(&key) {
        Ok(value) => {
            println!("Default Value: {value:?}");
        }
        Err(e) => {
            eprintln!("Default Value Error: {e:?}");
        }
    };

    println!("----------------------");
    Ok(())
}

/// Sets a key-value pair in the KVS.
/// It converts the value to the appropriate type based on the provided payload.
/// If the payload is a valid JSON string, it will be parsed and stored as a KVSValue.
/// If the payload is not provided, it will store a null value.
/// If the payload is not a valid JSON string, it will be stored as a string.
fn _setkey(kvs: Kvs, mut args: Arguments) -> Result<(), ErrorCode> {
    println!("----------------------");
    println!("Set Key");
    kvs.set_flush_on_exit(FlushOnExit::Yes);
    let key: String = match args.opt_value_from_str("--key") {
        Ok(Some(val)) => val,
        Ok(None) | Err(_) => match args.opt_value_from_str("-k") {
            Ok(Some(val)) => val,
            _ => {
                eprintln!("Error: Key (-k or --key) needs to be specified!");
                return Err(ErrorCode::UnmappedError);
            }
        },
    };

    let value_str: Option<String> = match args.opt_value_from_str("-p") {
        Ok(Some(val)) => Some(val),
        Ok(None) | Err(_) => match args.opt_value_from_str("--payload") {
            Ok(Some(val)) => Some(val),
            _ => None,
        },
    };

    match value_str {
        Some(value) => {
            if let Ok(json_val) = value.parse::<JsonValue>() {
                let kvs_val = from_tinyjson(&json_val);
                println!("Key:'{}' \nParsed as JSON Value: {:?}", &key, kvs_val);
                kvs.set_value(key, kvs_val).map_err(|e| {
                    eprintln!("KVS set failed: {e:?}");
                    e
                })?;
            } else {
                println!("Key:'{}' \nParsed as String Value: {}", &key, value);
                kvs.set_value(key, KvsValue::String(value)).map_err(|e| {
                    eprintln!("KVS set failed: {e:?}");
                    e
                })?;
            }
        }
        None => {
            kvs.set_value(key, KvsValue::Null).map_err(|e| {
                eprintln!("KVS set failed: {e:?}");
                e
            })?;
        }
    }
    println!("----------------------");
    Ok(())
}

/// Removes a key-value pair from the KVS.
fn _removekey(kvs: Kvs, mut args: Arguments) -> Result<(), ErrorCode> {
    println!("----------------------");
    kvs.set_flush_on_exit(FlushOnExit::Yes);
    let key: String = match args.opt_value_from_str("--key") {
        Ok(Some(val)) => val,
        Ok(None) | Err(_) => match args.opt_value_from_str("-k") {
            Ok(Some(val)) => val,
            _ => {
                eprintln!("Error: Key (-k or --key) needs to be specified!");
                return Err(ErrorCode::UnmappedError);
            }
        },
    };
    println!("Remove Key {}", &key);
    kvs.remove_key(&key).map_err(|e| {
        eprintln!("KVS remove failed: {e:?}");
        e
    })?;
    println!("----------------------");
    Ok(())
}

/// Lists all keys in the KVS.
/// It retrieves all keys and prints them to the console.
fn _listkeys(kvs: Kvs) -> Result<(), ErrorCode> {
    println!("----------------------");
    println!("List Keys");
    kvs.set_flush_on_exit(FlushOnExit::No);
    let keys = kvs.get_all_keys().map_err(|e| {
        eprintln!("KVS list failed: {e:?}");
        e
    })?;

    for key in keys {
        println!("{key}");
    }

    println!("----------------------");
    Ok(())
}

/// Resets the KVS by removing all keys and values.
fn _reset(kvs: Kvs) -> Result<(), ErrorCode> {
    println!("----------------------");
    println!("Reset KVS");
    kvs.set_flush_on_exit(FlushOnExit::Yes);
    kvs.reset().map_err(|e| {
        eprintln!("KVS set failed: {e:?}");
        e
    })?;
    println!("----------------------");
    Ok(())
}

/// Retrieves the snapshot count from the KVS.
fn _snapshotcount(kvs: Kvs) -> Result<(), ErrorCode> {
    println!("----------------------");
    println!("Snapshot Count");
    kvs.set_flush_on_exit(FlushOnExit::No);
    let count = kvs.snapshot_count();
    println!("Snapshot Count: {count}");
    println!("----------------------");
    Ok(())
}

/// Retrieves the maximum snapshot count from the KVS.
fn _snapshotmaxcount(kvs: Kvs) -> Result<(), ErrorCode> {
    println!("----------------------");
    println!("Snapshots Max Count");
    kvs.set_flush_on_exit(FlushOnExit::No);
    let max = Kvs::snapshot_max_count();
    println!("Snapshots Maximum Count: {max}");
    println!("----------------------");
    Ok(())
}

/// Restores a snapshot in the KVS.
/// It takes a snapshot ID as an argument and restores the KVS to that snapshot.
fn _snapshotrestore(kvs: Kvs, mut args: Arguments) -> Result<(), ErrorCode> {
    println!("----------------------");
    println!("Snapshot Restore");
    kvs.set_flush_on_exit(FlushOnExit::Yes);

    let snapshot_id: u32 = match args.opt_value_from_str("--snapshotid") {
        Ok(Some(val)) => val,
        Ok(None) | Err(_) => match args.opt_value_from_str("-s") {
            Ok(Some(val)) => val,
            _ => {
                eprintln!("Error: Snapshot ID (-s or --snapshotid) needs to be specified!");
                return Err(ErrorCode::UnmappedError);
            }
        },
    };
    println!("Restore Snapshot {}", &snapshot_id);
    let snapshot_id = SnapshotId(snapshot_id as usize);
    kvs.snapshot_restore(snapshot_id).map_err(|e| {
        eprintln!("KVS restore failed: {e:?}");
        e
    })?;
    println!("----------------------");
    Ok(())
}

/// Retrieves the KVS filename for a given snapshot ID.
fn _getkvsfilename(kvs: Kvs, mut args: Arguments) -> Result<(), ErrorCode> {
    println!("----------------------");
    println!("Get KVS Filename");
    kvs.set_flush_on_exit(FlushOnExit::No);
    let snapshot_id: u32 = match args.opt_value_from_str("--snapshotid") {
        Ok(Some(val)) => val,
        Ok(None) | Err(_) => match args.opt_value_from_str("-s") {
            Ok(Some(val)) => val,
            _ => {
                eprintln!("Error: Snapshot ID (-s or --snapshotid) needs to be specified!");
                return Err(ErrorCode::UnmappedError);
            }
        },
    };
    let snapshot_id = SnapshotId(snapshot_id as usize);
    let filename = kvs.get_kvs_filename(snapshot_id)?;
    println!("KVS Filename: {}", filename.display());
    println!("----------------------");
    Ok(())
}

/// Retrieves the hash filename for a given snapshot ID.
fn _gethashfilename(kvs: Kvs, mut args: Arguments) -> Result<(), ErrorCode> {
    println!("----------------------");
    println!("Get Hash Filename");
    kvs.set_flush_on_exit(FlushOnExit::No);

    let snapshot_id: u32 = match args.opt_value_from_str("--snapshotid") {
        Ok(Some(val)) => val,
        Ok(None) | Err(_) => match args.opt_value_from_str("-s") {
            Ok(Some(val)) => val,
            _ => {
                eprintln!("Error: Snapshot ID (-s or --snapshotid) needs to be specified!");
                return Err(ErrorCode::UnmappedError);
            }
        },
    };
    let snapshot_id = SnapshotId(snapshot_id as usize);
    let filename = kvs.get_hash_filename(snapshot_id);
    println!("Hash Filename: {}", filename?.display());
    println!("----------------------");
    Ok(())
}

/// Creates test data in the KVS based on the example code from the KVS.
fn _createtestdata(kvs: Kvs) -> Result<(), ErrorCode> {
    println!("----------------------");
    println!("Create Test Data");
    kvs.set_flush_on_exit(FlushOnExit::Yes);

    kvs.set_value("number", 123.0).map_err(|e| {
        eprintln!("KVS Create Test Data Error (number): {e:?}");
        e
    })?;
    kvs.set_value("bool", true).map_err(|e| {
        eprintln!("KVS Create Test Data Error (bool): {e:?}");
        e
    })?;
    kvs.set_value("string", "First".to_string()).map_err(|e| {
        eprintln!("KVS Create Test Data Error (string): {e:?}");
        e
    })?;
    kvs.set_value("null", ()).map_err(|e| {
        eprintln!("KVS Create Test Data Error (null): {e:?}");
        e
    })?;
    kvs.set_value(
        "array",
        vec![
            KvsValue::from(456.0),
            false.into(),
            "Second".to_string().into(),
        ],
    )
    .map_err(|e| {
        eprintln!("KVS Create Test Data Error (array): {e:?}");
        e
    })?;
    kvs.set_value(
        "object",
        HashMap::from([
            (String::from("sub-number"), KvsValue::from(789.0)),
            ("sub-bool".into(), true.into()),
            ("sub-string".into(), "Third".to_string().into()),
            ("sub-null".into(), ().into()),
            (
                "sub-array".into(),
                KvsValue::from(vec![
                    KvsValue::from(1246.0),
                    false.into(),
                    "Fourth".to_string().into(),
                ]),
            ),
        ]),
    )
    .map_err(|e| {
        eprintln!("KVS Create Test Data Error (object): {e:?}");
        e
    })?;
    println!("Done!");
    println!("----------------------");
    Ok(())
}

/// Main function to run the KVS tool command line interface.
fn main() -> Result<(), ErrorCode> {
    let mut args = Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        const HELP: &str = r#"

        ---------------------------------------
        KVS Tool - Command Line Interface
        ---------------------------------------

        Version 0.1.0

        ---------------------------------------

        Options:
        -h, --help          Show this help message and exit
        -o, --operation     Specify the operation to perform (setkey, getkey, removekey, 
                            listkeys, reset, snapshotcount, snapshotmaxcount, snapshotrestore, 
                            getkvsfilename, gethashfilename, createtestdata)
        -k, --key           Specify the key to operate on (for key operations)
        -p, --payload       Specify the value to write (for set operations)
        -s, --snapshotid    Specify the snapshot ID for Snapshot operations
        -d, --directory     Specify the directory of the Key-Files (default is current directory)

        ---------------------------------------

        Usage Examples:

        Read a Key and show value:
            kvs_tool -o getkey -k MyKey

        Write a Key and use the <payload> as the data source:
            (automatically detects following types: Number, Boolean, String, Null, Object, Array)
            kvs_tool -o setkey  -k MyKey -p 'Hello World'
            kvs_tool -o setkey  -k MyKey -p 'true'
            kvs_tool -o setkey  -k MyKey -p 15
            kvs_tool -o setkey  -k MyKey -p '[456,false,"Second"]'
            kvs_tool -o setkey  -k MyKey -p '{"sub-number":789,"sub-array":[1246,false,"Fourth"]}'

        Delete a key:
            kvs_tool -o removekey -k MyKey

        List Keys:
            kvs_tool -o listkeys

        Reset KVS:
            kvs_tool -o reset

        Snapshot Count:
            kvs_tool -o snapshotcount

        Snapshot Restore:
            kvs_tool -o snapshotrestore -s 1

        Get KVS Filename:
            kvs_tool -o getkvsfilename -s 1

        Get Hash Filename:
            kvs_tool -o gethashfilename -s 1

        ---------------------------------------

        Create Test Data:
            kvs_tool -o createtestdata (Creates Data provided by the example code in the KVS API)

        ---------------------------------------

        "#;
        println!("{HELP}");
        return Ok(());
    }
    let directory: Option<String> = match args.opt_value_from_str("--directory") {
        Ok(Some(val)) => Some(val),
        Ok(None) | Err(_) => match args.opt_value_from_str("-d") {
            Ok(Some(val)) => Some(val),
            _ => None,
        },
    };

    let builder = KvsBuilder::new(InstanceId(0))
        .need_defaults(false)
        .need_kvs(false);

    let builder = if let Some(dir) = directory {
        builder.dir(dir)
    } else {
        builder
    };

    let kvs = match builder.build() {
        Ok(kvs) => kvs,
        Err(e) => {
            eprintln!("Error opening KVS: {e:?}");
            return Err(e);
        }
    };

    let operation: Option<String> = match args.opt_value_from_str("--operation") {
        Ok(Some(val)) => Some(val),
        Ok(None) | Err(_) => match args.opt_value_from_str("-o") {
            Ok(Some(val)) => Some(val),
            _ => {
                eprintln!(
                    "Error: No operation specified. Use -o or --operation followed by a value."
                );
                return Err(ErrorCode::UnmappedError);
            }
        },
    };
    let op_mode = match operation {
        Some(op) => match op.as_str() {
            "getkey" => OperationMode::GetKey,
            "setkey" => OperationMode::SetKey,
            "removekey" => OperationMode::RemoveKey,
            "listkeys" => OperationMode::ListKeys,
            "reset" => OperationMode::Reset,
            "createtestdata" => OperationMode::CreateTestData,
            "snapshotcount" => OperationMode::SnapshotCount,
            "snapshotmaxcount" => OperationMode::SnapshotMaxCount,
            "snapshotrestore" => OperationMode::SnapshotRestore,
            "getkvsfilename" => OperationMode::GetKvsFilename,
            "gethashfilename" => OperationMode::GetHashFilename,
            _ => OperationMode::Invalid,
        },
        None => OperationMode::Invalid,
    };

    match op_mode {
        OperationMode::GetKey => {
            _getkey(kvs, args)?;
            Ok(())
        }
        OperationMode::SetKey => {
            _setkey(kvs, args)?;
            Ok(())
        }
        OperationMode::RemoveKey => {
            _removekey(kvs, args)?;
            Ok(())
        }
        OperationMode::ListKeys => {
            _listkeys(kvs)?;
            Ok(())
        }
        OperationMode::Reset => {
            _reset(kvs)?;
            Ok(())
        }
        OperationMode::SnapshotCount => {
            _snapshotcount(kvs)?;
            Ok(())
        }
        OperationMode::SnapshotMaxCount => {
            _snapshotmaxcount(kvs)?;
            Ok(())
        }
        OperationMode::SnapshotRestore => {
            _snapshotrestore(kvs, args)?;
            Ok(())
        }
        OperationMode::GetKvsFilename => {
            _getkvsfilename(kvs, args)?;
            Ok(())
        }
        OperationMode::GetHashFilename => {
            _gethashfilename(kvs, args)?;
            Ok(())
        }
        OperationMode::CreateTestData => {
            _createtestdata(kvs)?;
            Ok(())
        }
        OperationMode::Invalid => {
            println!("----------------------");
            eprintln!("Invalid operation specified. Use -o or --operation to specify a valid operation. (See -h or --help for more information)");
            println!("----------------------");
            Err(ErrorCode::UnmappedError)
        }
    }
}
