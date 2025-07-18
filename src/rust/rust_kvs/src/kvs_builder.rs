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

use super::Kvs;
use crate::error_code::ErrorCode;
use crate::kvs::InstanceId;
use crate::kvs_api::KvsApi;
/// Key-value-storage builder
pub struct KvsBuilder<T: KvsApi = Kvs> {
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

impl<T> KvsBuilder<T>
where
    T: KvsApi,
{
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
    use crate::kvs::SnapshotId;
    use crate::kvs_value::KvsValue;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[test]
    fn test_new_kvs_builder() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let builder = KvsBuilder::<Kvs>::new(instance_id.clone()).dir(dir_path);

        assert_eq!(builder.instance_id, instance_id);
        assert!(!builder.need_defaults);
        assert!(!builder.need_kvs);
    }

    #[test]
    fn test_need_defaults() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let builder = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path)
            .need_defaults(true);

        assert!(builder.need_defaults);
    }

    #[test]
    fn test_need_kvs() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let builder = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path)
            .need_kvs(true);

        assert!(builder.need_kvs);
    }

    #[test]
    fn test_build() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let builder = KvsBuilder::<Kvs>::new(instance_id.clone()).dir(dir_path);

        builder.build().unwrap();
    }

    #[test]
    fn test_build_with_defaults() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let builder = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path)
            .need_defaults(true);

        assert!(builder.build().is_err());
    }

    #[test]
    fn test_build_with_kvs() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);

        // negative
        let builder = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path.clone())
            .need_kvs(true);
        assert!(builder.build().is_err());

        KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path.clone())
            .build()
            .unwrap();

        // positive
        let builder = KvsBuilder::<Kvs>::new(instance_id)
            .dir(dir_path)
            .need_kvs(true);
        builder.build().unwrap();
    }

    #[test]
    fn test_flush_on_exit() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let kvs = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path.clone())
            .build()
            .unwrap();
        kvs.flush_on_exit(true);
    }

    #[test]
    fn test_reset() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let kvs = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path.clone())
            .build()
            .unwrap();
        let _ = kvs.set_value("test", 1.0);
        let result = kvs.reset();
        assert!(result.is_ok(), "Expected Ok for reset");
    }

    #[test]
    fn test_get_all_keys() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let kvs = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path.clone())
            .build()
            .unwrap();
        let _ = kvs.set_value("test", 1.0);
        let keys = kvs.get_all_keys();
        assert!(keys.is_ok(), "Expected Ok for get_all_keys");
        let keys = keys.unwrap();
        assert!(
            keys.contains(&"test".to_string()),
            "Expected 'test' key in get_all_keys"
        );
    }

    #[test]
    fn test_key_exists() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let kvs = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path.clone())
            .build()
            .unwrap();
        let exists = kvs.key_exists("test");
        assert!(exists.is_ok(), "Expected Ok for key_exists");
        assert!(!exists.unwrap(), "Expected 'test' key to not exist");
        let _ = kvs.set_value("test", 1.0);
        let exists = kvs.key_exists("test");
        assert!(exists.is_ok(), "Expected Ok for key_exists after set");
        assert!(exists.unwrap(), "Expected 'test' key to exist after set");
    }

    #[test]
    fn test_get_filename_before_flush() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let kvs = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path)
            .build()
            .unwrap();
        let result = kvs.get_kvs_filename(SnapshotId::new(0));
        assert!(result.is_err_and(|e| e == ErrorCode::FileNotFound));
    }

    #[test]
    fn test_get_filename() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let kvs = KvsBuilder::<Kvs>::new(instance_id.clone())
            .dir(dir_path.clone())
            .build()
            .unwrap();
        kvs.flush().unwrap();
        let filename = kvs.get_kvs_filename(SnapshotId::new(0)).unwrap();
        let filename_str = filename.to_string_lossy().to_string();
        assert!(
            filename_str.ends_with("_0.json"),
            "Expected filename to end with _0.json: {}",
            filename.display()
        );
    }

    #[test]
    fn test_get_value() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let kvs = Arc::new(
            KvsBuilder::<Kvs>::new(instance_id.clone())
                .dir(dir_path.clone())
                .build()
                .unwrap(),
        );
        let _ = kvs.set_value("test", 123.0);
        let value = kvs.get_value("test").unwrap();
        assert_eq!(
            *value.get::<f64>().unwrap(),
            123.0,
            "Expected to retrieve the inserted value"
        );
    }

    #[test]
    fn test_get_value_as() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);
        let kvs = Arc::new(
            KvsBuilder::<Kvs>::new(instance_id.clone())
                .dir(dir_path.clone())
                .build()
                .unwrap(),
        );
        let _ = kvs.set_value("test", 123.0);
        let value = kvs.get_value_as::<f64>("test");
        assert_eq!(
            value.unwrap(),
            123.0,
            "Expected to retrieve the inserted value"
        );
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_get_value_try_from_error() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_string_lossy().to_string();

        let instance_id = InstanceId::new(0);

        // Create the test JSON file locally instead of copying
        let test_json_path = format!("{}/kvs_0_default.json", dir_path.clone());
        let test_json_content = r#"{
            "bool1": { "type": "Boolean", "value": true },
            "test": { "type": "F64", "value": 123.0 }
        }"#;
        std::fs::write(&test_json_path, test_json_content).unwrap();

        let kvs = Arc::new(
            KvsBuilder::<Kvs>::new(instance_id.clone())
                .dir(dir_path.clone())
                .need_defaults(true)
                .build()
                .unwrap(),
        );

        let _ = kvs.set_value("test", 123.0f64);

        // stored value: should return ConversionFailed
        let result = kvs.get_value_as::<u64>("test");
        assert!(
            matches!(result, Err(ErrorCode::ConversionFailed)),
            "Expected ConversionFailed for stored value"
        );

        // default value: should return ConversionFailed
        let result = kvs.get_value_as::<u64>("bool1");
        assert!(
            matches!(result, Err(ErrorCode::ConversionFailed)),
            "Expected ConversionFailed for default value"
        );
    }
}
