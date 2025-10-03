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
use crate::kvs_value::KvsMap;
use std::any::Any;

pub trait DynEq: Any {
    fn dyn_eq(&self, other: &dyn Any) -> bool;
    fn as_any(&self) -> &dyn Any;
}

impl<T: PartialEq + Any> DynEq for T
where
    T: KvsBackend,
{
    fn dyn_eq(&self, other: &dyn Any) -> bool {
        if let Some(other) = other.downcast_ref::<T>() {
            self == other
        } else {
            false
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// KVS backend interface.
pub trait KvsBackend: DynEq + Sync + Send {
    /// Load KVS content.
    fn load_kvs(
        &self,
        instance_id: InstanceId,
        snapshot_id: SnapshotId,
    ) -> Result<KvsMap, ErrorCode>;

    /// Load default values.
    fn load_defaults(&self, instance_id: InstanceId) -> Result<KvsMap, ErrorCode>;

    /// Flush KvsMap to persistent storage.
    /// Snapshots are rotated and current state is stored as first (0).
    fn flush(&self, instance_id: InstanceId, kvs_map: &KvsMap) -> Result<(), ErrorCode>;

    /// Count available snapshots.
    fn snapshot_count(&self, instance_id: InstanceId) -> usize;

    /// Max number of snapshots.
    fn snapshot_max_count(&self) -> usize;

    /// Restore snapshot with given ID.
    fn snapshot_restore(
        &self,
        instance_id: InstanceId,
        snapshot_id: SnapshotId,
    ) -> Result<KvsMap, ErrorCode>;
}
