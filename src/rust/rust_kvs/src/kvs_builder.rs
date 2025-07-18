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
use crate::kvs_api::{InstanceId, KvsApi};

/// Key-value-storage builder
pub struct KvsBuilder<T: KvsApi> {
    /// Instance ID
    instance_id: InstanceId,

    /// Need-defaults flag
    need_defaults: bool,

    /// Need-KVS flag
    need_kvs: bool,

    /// Working directory
    dir: Option<String>,

    /// Phantom data for drop check
    _phantom: std::marker::PhantomData<T>,
}

impl<T: KvsApi> KvsBuilder<T> {
    /// Create a builder to open the key-value-storage
    ///
    /// Only the instance ID must be set. All other settings are using default values until changed
    /// via the builder API.
    ///
    /// # Parameters
    ///   * `instance_id`: Instance ID
    ///
    /// # Return Values
    ///   * KvsBuilder instance
    pub fn new(instance_id: InstanceId) -> Self {
        Self {
            instance_id,
            need_defaults: false,
            need_kvs: false,
            dir: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Configure if defaults must exist when opening the KVS
    ///
    /// # Parameters
    ///   * `flag`: Yes = `true`, no = `false` (default)
    ///
    /// # Return Values
    ///   * KvsBuilder instance
    pub fn need_defaults(mut self, flag: bool) -> Self {
        self.need_defaults = flag;
        self
    }

    /// Configure if KVS must exist when opening the KVS
    ///
    /// # Parameters
    ///   * `flag`: Yes = `true`, no = `false` (default)
    ///
    /// # Return Values
    ///   * KvsBuilder instance
    pub fn need_kvs(mut self, flag: bool) -> Self {
        self.need_kvs = flag;
        self
    }

    /// Set the key-value-storage permanent storage directory
    ///
    /// # Parameters
    ///   * `dir`: Path to permanent storage
    ///
    /// # Return Values
    pub fn dir<P: Into<String>>(mut self, dir: P) -> Self {
        self.dir = Some(dir.into());
        self
    }

    /// Finalize the builder and open the key-value-storage
    ///
    /// Calls `Kvs::open` with the configured settings.
    ///
    /// # Features
    ///   * `FEAT_REQ__KVS__default_values`
    ///   * `FEAT_REQ__KVS__multiple_kvs`
    ///   * `FEAT_REQ__KVS__integrity_check`
    ///
    /// # Return Values
    ///   * Ok: KVS instance
    ///   * `ErrorCode::ValidationFailed`: KVS hash validation failed
    ///   * `ErrorCode::JsonParserError`: JSON parser error
    ///   * `ErrorCode::KvsFileReadError`: KVS file read error
    ///   * `ErrorCode::KvsHashFileReadError`: KVS hash file read error
    ///   * `ErrorCode::UnmappedError`: Generic error
    pub fn build(self) -> Result<T, ErrorCode> {
        T::open(
            self.instance_id,
            self.need_defaults.into(),
            self.need_kvs.into(),
            self.dir,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kvs_mock::MockKvs;

    #[test]
    fn test_builder_new_sets_instance_id() {
        let instance_id = InstanceId::new(42);
        let builder = KvsBuilder::<MockKvs>::new(instance_id.clone());
        let kvs = builder.build();
        assert!(kvs.is_ok());
    }

    #[test]
    fn test_builder_need_defaults() {
        let builder = KvsBuilder::<MockKvs>::new(InstanceId::new(1)).need_defaults(true);
        let kvs = builder.build();
        assert!(kvs.is_ok());
    }

    #[test]
    fn test_builder_need_kvs() {
        let builder = KvsBuilder::<MockKvs>::new(InstanceId::new(1)).need_kvs(true);
        let kvs = builder.build();
        assert!(kvs.is_ok());
    }

    #[test]
    fn test_builder_dir() {
        let builder = KvsBuilder::<MockKvs>::new(InstanceId::new(1)).dir("/tmp/test_kvs");
        let kvs = builder.build();
        assert!(kvs.is_ok());
    }

    #[test]
    fn test_builder_chained() {
        let builder = KvsBuilder::<MockKvs>::new(InstanceId::new(1))
            .need_defaults(true)
            .need_kvs(true)
            .dir("/tmp/test_kvs2");
        let kvs = builder.build();
        assert!(kvs.is_ok());
    }
}
