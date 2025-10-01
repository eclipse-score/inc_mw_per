..
   # *******************************************************************************
   # Copyright (c) 2024 Contributors to the Eclipse Foundation
   #
   # See the NOTICE file(s) distributed with this work for additional
   # information regarding copyright ownership.
   #
   # This program and the accompanying materials are made available under the
   # terms of the Apache License Version 2.0 which is available at
   # https://www.apache.org/licenses/LICENSE-2.0
   #
   # SPDX-License-Identifier: Apache-2.0
   # *******************************************************************************

Key-Value-Storage (rust_kvs) Documentation
==========================================

This documentation describes the `rust_kvs` crate, which provides a key-value storage implementation with JSON-like persistence using Rust.

.. contents:: Table of Contents
   :depth: 2
   :local:

Summary
-------

**Crate:** `rust_kvs`

**Purpose:** Key-Value-Storage API and Implementation

**Description:**  
This crate provides a Key-Value-Store using TinyJSON to persist the data. It uses the Adler32 crate to validate stored data and depends only on the Rust standard library.

Introduction
------------

The key-value store is initialized with `Kvs::open` and can be flushed with `Kvs::flush`.

All TinyJSON-supported datatypes are available:

- `Number`: `f64`
- `Boolean`: `bool`
- `String`: `String`
- `Null`: `()`
- `Array`: `Vec<KvsValue>`
- `Object`: `HashMap<String, KvsValue>`

JSON arrays can hold mixed types.

Usage Notes:

- Use `Kvs::set_value(key, value)` to write.
- Use `Kvs::get_value::<T>(key)` to read.
- If a key is missing, the store checks for a default and returns it.
- Defaults are not flushed unless explicitly written.

To check for defaults:

- `Kvs::get_default_value`
- `Kvs::is_value_default`

Example
-------

.. code-block:: rust

   use rust_kvs::{ErrorCode, InstanceId, Kvs, OpenNeedDefaults, OpenNeedKvs, KvsValue};
   use std::collections::HashMap;

   fn main() -> Result<(), ErrorCode> {
       let kvs = Kvs::open(
           InstanceId::new(0),
           OpenNeedDefaults::Optional,
           OpenNeedKvs::Optional)?;

       kvs.set_value("number", 123.0)?;
       kvs.set_value("bool", true)?;
       kvs.set_value("string", "First".to_string())?;
       kvs.set_value("null", ())?;
       kvs.set_value(
           "array",
           vec![
               KvsValue::from(456.0),
               false.into(),
               "Second".to_string().into(),
           ],
       )?;
       kvs.set_value(
           "object",
           HashMap::from([
               ("sub-number".into(), KvsValue::from(789.0)),
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
       )?;

       Ok(())
   }

Related Requirements
====================

This crate addresses or is influenced by the following specification requirements from the system architecture.

General Capabilities
--------------------

- :need:`S-CORE_feat_req__persistency__cpp_rust_interop`
- :need:`S-CORE_feat_req__persistency__multiple_kvs`
- :need:`S-CORE_feat_req__persistency__tooling`
- :need:`S-CORE_feat_req__persistency__variant_management`
- :need:`S-CORE_feat_req__persistency__eng_mode`
- :need:`S-CORE_feat_req__persistency__async_api`
- :need:`S-CORE_feat_req__persistency__access_control`
- :need:`S-CORE_feat_req__persistency__events`
- :need:`S-CORE_feat_req__persistency__fast_access`
- :need:`S-CORE_feat_req__persistency__intra_process_comm`

Data Storage and Persistency
----------------------------

- :need:`S-CORE_feat_req__persistency__default_values`
- :need:`S-CORE_feat_req__persistency__default_value_get`
- :need:`S-CORE_feat_req__persistency__default_value_reset`
- :need:`S-CORE_feat_req__persistency__persistency`
- :need:`S-CORE_feat_req__persistency__integrity_check`
- :need:`S-CORE_feat_req__persistency__versioning`
- :need:`S-CORE_feat_req__persistency__update_mechanism`
- :need:`S-CORE_feat_req__persistency__snapshots`
- :need:`S-CORE_feat_req__persistency__persist_data`

Datatypes and Interfaces
------------------------

- :need:`S-CORE_feat_req__persistency__support_datatype_keys`
- :need:`S-CORE_feat_req__persistency__support_datatype_value`

Configuration Support
---------------------

- :need:`S-CORE_feat_req__persistency__default_value_file`
- :need:`S-CORE_feat_req__persistency__config_file`

Limitations and Future Work
---------------------------

- :need:`S-CORE_feat_req__persistency__maximum_size` *(currently unsupported)*



