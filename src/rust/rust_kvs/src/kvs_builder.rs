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
use crate::kvs_api::{InstanceId, KvsApi, KvsDefaults, KvsLoad};

/// Key-value-storage builder
pub struct KvsBuilder<T: KvsApi> {
    /// Instance ID
    instance_id: InstanceId,

    /// Defaults handling mode.
    defaults: KvsDefaults,

    /// KVS load mode.
    kvs_load: KvsLoad,

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
            defaults: KvsDefaults::Optional,
            kvs_load: KvsLoad::Optional,
            dir: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Configure defaults handling mode.
    ///
    /// # Parameters
    ///   * `mode`: defaults handling mode (default: [`KvsDefaults::Optional`](KvsDefaults::Optional))
    ///
    /// # Return Values
    ///   * KvsBuilder instance
    pub fn defaults(mut self, mode: KvsDefaults) -> Self {
        self.defaults = mode;
        self
    }

    /// Configure KVS load mode.
    ///
    /// # Parameters
    ///   * `mode`: KVS load mode (default: [`KvsLoad::Optional`](KvsLoad::Optional))
    ///
    /// # Return Values
    ///   * KvsBuilder instance
    pub fn kvs_load(mut self, mode: KvsLoad) -> Self {
        self.kvs_load = mode;
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
        T::open(self.instance_id, self.defaults, self.kvs_load, self.dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kvs_api::{KvsDefaults, KvsLoad, SnapshotId};
    use crate::kvs_value::KvsValue;
    use std::path::PathBuf;

    /// Empty KVS stub with exposed builder parameters.
    struct StubKvs {
        instance_id: InstanceId,
        defaults: KvsDefaults,
        kvs_load: KvsLoad,
        dir: Option<String>,
    }

    impl StubKvs {
        fn instance_id(&self) -> &InstanceId {
            &self.instance_id
        }

        fn defaults(&self) -> &KvsDefaults {
            &self.defaults
        }

        fn kvs_load(&self) -> &KvsLoad {
            &self.kvs_load
        }

        fn dir(&self) -> &Option<String> {
            &self.dir
        }
    }

    impl KvsApi for StubKvs {
        fn open(
            instance_id: InstanceId,
            defaults: KvsDefaults,
            kvs_load: KvsLoad,
            dir: Option<String>,
        ) -> Result<Self, ErrorCode>
        where
            Self: Sized,
        {
            Ok(Self {
                instance_id,
                defaults,
                kvs_load,
                dir,
            })
        }

        fn reset(&self) -> Result<(), ErrorCode> {
            unimplemented!()
        }

        fn reset_key(&self, _key: &str) -> Result<(), ErrorCode> {
            unimplemented!()
        }

        fn get_all_keys(&self) -> Result<Vec<String>, ErrorCode> {
            unimplemented!()
        }

        fn key_exists(&self, _key: &str) -> Result<bool, ErrorCode> {
            unimplemented!()
        }

        fn get_value(&self, _key: &str) -> Result<crate::prelude::KvsValue, ErrorCode> {
            unimplemented!()
        }

        fn get_value_as<T>(&self, _key: &str) -> Result<T, ErrorCode>
        where
            for<'a> T: TryFrom<&'a KvsValue> + Clone,
            for<'a> <T as TryFrom<&'a KvsValue>>::Error: std::fmt::Debug,
        {
            unimplemented!()
        }

        fn get_default_value(&self, _key: &str) -> Result<crate::prelude::KvsValue, ErrorCode> {
            unimplemented!()
        }

        fn is_value_default(&self, _key: &str) -> Result<bool, ErrorCode> {
            unimplemented!()
        }

        fn set_value<S: Into<String>, J: Into<crate::prelude::KvsValue>>(
            &self,
            _key: S,
            _value: J,
        ) -> Result<(), ErrorCode> {
            unimplemented!()
        }

        fn remove_key(&self, _key: &str) -> Result<(), ErrorCode> {
            unimplemented!()
        }

        fn flush(&self) -> Result<(), ErrorCode> {
            unimplemented!()
        }

        fn snapshot_count(&self) -> usize {
            unimplemented!()
        }

        fn snapshot_max_count() -> usize
        where
            Self: Sized,
        {
            unimplemented!()
        }

        fn snapshot_restore(&self, _id: SnapshotId) -> Result<(), ErrorCode> {
            unimplemented!()
        }

        fn get_kvs_filename(&self, _id: SnapshotId) -> Result<PathBuf, ErrorCode> {
            unimplemented!()
        }

        fn get_hash_filename(&self, _id: SnapshotId) -> Result<PathBuf, ErrorCode> {
            unimplemented!()
        }
    }

    #[test]
    fn test_builder_only_instance_id() {
        let instance_id = InstanceId(42);
        let builder = KvsBuilder::<StubKvs>::new(instance_id);
        let kvs = builder.build().unwrap();
        assert_eq!(kvs.instance_id().clone(), instance_id);
        assert_eq!(kvs.defaults().clone(), KvsDefaults::Optional);
        assert_eq!(kvs.kvs_load().clone(), KvsLoad::Optional);
        assert!(kvs.dir().is_none());
    }

    #[test]
    fn test_builder_defaults() {
        let instance_id = InstanceId(1);
        let builder = KvsBuilder::<StubKvs>::new(instance_id).defaults(KvsDefaults::Required);
        let kvs = builder.build().unwrap();
        assert_eq!(kvs.instance_id().clone(), instance_id);
        assert_eq!(kvs.defaults().clone(), KvsDefaults::Required);
        assert_eq!(kvs.kvs_load().clone(), KvsLoad::Optional);
        assert!(kvs.dir().is_none());
    }

    #[test]
    fn test_builder_kvs_load() {
        let instance_id = InstanceId(1);
        let builder = KvsBuilder::<StubKvs>::new(instance_id).kvs_load(KvsLoad::Required);
        let kvs = builder.build().unwrap();
        assert_eq!(kvs.instance_id().clone(), instance_id);
        assert_eq!(kvs.defaults().clone(), KvsDefaults::Optional);
        assert_eq!(kvs.kvs_load().clone(), KvsLoad::Required);
        assert!(kvs.dir().is_none());
    }

    #[test]
    fn test_builder_dir() {
        let instance_id = InstanceId(1);
        let dir = "/tmp/test_kvs".to_string();
        let builder = KvsBuilder::<StubKvs>::new(instance_id).dir(dir.clone());
        let kvs = builder.build().unwrap();
        assert_eq!(kvs.instance_id().clone(), instance_id);
        assert_eq!(kvs.defaults().clone(), KvsDefaults::Optional);
        assert_eq!(kvs.kvs_load().clone(), KvsLoad::Optional);
        assert!(kvs.dir().clone().is_some_and(|p| p == dir));
    }

    #[test]
    fn test_builder_chained() {
        let instance_id = InstanceId(1);
        let dir = "/tmp/test_kvs".to_string();
        let builder = KvsBuilder::<StubKvs>::new(instance_id)
            .defaults(KvsDefaults::Required)
            .kvs_load(KvsLoad::Required)
            .dir(dir.clone());
        let kvs = builder.build().unwrap();
        assert_eq!(kvs.instance_id().clone(), instance_id);
        assert_eq!(kvs.defaults().clone(), KvsDefaults::Required);
        assert_eq!(kvs.kvs_load().clone(), KvsLoad::Required);
        assert!(kvs.dir().clone().is_some_and(|p| p == dir));
    }
}
