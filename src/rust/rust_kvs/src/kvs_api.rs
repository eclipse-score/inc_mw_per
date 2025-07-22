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

use std::path::PathBuf;

//core and alloc libs
use core::fmt;

use crate::error_code::ErrorCode;
use crate::kvs_value::KvsValue;

/// Instance ID
#[derive(Clone, Debug, PartialEq)]
pub struct InstanceId(usize);

/// Snapshot ID
#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotId(pub usize);

impl fmt::Display for InstanceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for SnapshotId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl InstanceId {
    /// Create a new instance ID
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

impl SnapshotId {
    /// Create a new Snapshot ID
    pub fn new(id: usize) -> Self {
        SnapshotId(id)
    }
}

/// Need-Defaults flag
pub enum OpenNeedDefaults {
    /// Optional: Open defaults only if available
    Optional,

    /// Required: Defaults must be available
    Required,
}

/// Need-KVS flag
pub enum OpenNeedKvs {
    /// Optional: Use an empty KVS if no KVS is available
    Optional,

    /// Required: KVS must be already exist
    Required,
}

impl From<bool> for OpenNeedDefaults {
    fn from(flag: bool) -> OpenNeedDefaults {
        if flag {
            OpenNeedDefaults::Required
        } else {
            OpenNeedDefaults::Optional
        }
    }
}

impl From<bool> for OpenNeedKvs {
    fn from(flag: bool) -> OpenNeedKvs {
        if flag {
            OpenNeedKvs::Required
        } else {
            OpenNeedKvs::Optional
        }
    }
}

pub trait KvsApi {
    fn open(
        instance_id: InstanceId,
        need_defaults: OpenNeedDefaults,
        need_kvs: OpenNeedKvs,
        dir: Option<String>,
    ) -> Result<Self, ErrorCode>
    where
        Self: Sized;

    fn reset(&self) -> Result<(), ErrorCode>;
    fn reset_key(&self, key: &str) -> Result<(), ErrorCode>;
    fn get_all_keys(&self) -> Result<Vec<String>, ErrorCode>;
    fn key_exists(&self, key: &str) -> Result<bool, ErrorCode>;
    fn get_value(&self, key: &str) -> Result<KvsValue, ErrorCode>;
    fn get_value_as<T>(&self, key: &str) -> Result<T, ErrorCode>
    where
        for<'a> T: TryFrom<&'a KvsValue> + Clone,
        for<'a> <T as TryFrom<&'a KvsValue>>::Error: std::fmt::Debug;
    fn get_default_value(&self, key: &str) -> Result<KvsValue, ErrorCode>;
    fn is_value_default(&self, key: &str) -> Result<bool, ErrorCode>;
    fn set_value<S: Into<String>, J: Into<KvsValue>>(
        &self,
        key: S,
        value: J,
    ) -> Result<(), ErrorCode>;
    fn remove_key(&self, key: &str) -> Result<(), ErrorCode>;
    fn flush_on_exit(&self, flush_on_exit: bool);
    fn flush(&self) -> Result<(), ErrorCode>;
    fn snapshot_count(&self) -> usize;
    fn snapshot_max_count() -> usize
    where
        Self: Sized;
    fn snapshot_restore(&self, id: SnapshotId) -> Result<(), ErrorCode>;
    fn get_kvs_filename(&self, id: SnapshotId) -> Result<PathBuf, ErrorCode>;
    fn get_hash_filename(&self, id: SnapshotId) -> Result<PathBuf, ErrorCode>;
}
