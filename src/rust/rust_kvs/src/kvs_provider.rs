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
use crate::kvs::GenericKvs;
use crate::kvs_api::{Defaults, FlushOnExit, InstanceId, KvsLoad, SnapshotId};
use crate::kvs_backend::{KvsBackend, KvsPathResolver};
use crate::kvs_value::KvsMap;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// Maximum number of instances.
const KVS_MAX_INSTANCES: usize = 10;

/// Key-value-storage parameters builder.
#[derive(Clone)]
pub struct KvsParameters {
    /// Instance ID.
    instance_id: InstanceId,

    /// Defaults handling mode.
    defaults: Defaults,

    /// KVS load mode.
    kvs_load: KvsLoad,

    /// Flush on exit mode.
    flush_on_exit: FlushOnExit,
}

impl KvsParameters {
    pub fn new(instance_id: InstanceId) -> Self {
        Self {
            instance_id,
            defaults: Defaults::Optional,
            kvs_load: KvsLoad::Optional,
            flush_on_exit: FlushOnExit::Yes,
        }
    }

    pub fn defaults(mut self, flag: Defaults) -> Self {
        self.defaults = flag;
        self
    }

    pub fn kvs_load(mut self, flag: KvsLoad) -> Self {
        self.kvs_load = flag;
        self
    }

    pub fn flush_on_exit(mut self, flag: FlushOnExit) -> Self {
        self.flush_on_exit = flag;
        self
    }
}

pub(crate) struct GenericKvsInner {
    /// Storage data.
    pub(crate) kvs_map: KvsMap,

    /// Optional default values.
    pub(crate) defaults_map: KvsMap,

    /// Flush on exit mode.
    pub(crate) flush_on_exit: FlushOnExit,
}

impl From<PoisonError<MutexGuard<'_, GenericKvsInner>>> for ErrorCode {
    fn from(cause: PoisonError<MutexGuard<'_, GenericKvsInner>>) -> Self {
        eprintln!("error: Pool mutex locking failed: {cause:#?}");
        ErrorCode::MutexLockFailed
    }
}

pub struct GenericKvsProvider<Backend: KvsBackend + KvsPathResolver> {
    /// KVS pool.
    kvs_pool: [Option<Arc<Mutex<GenericKvsInner>>>; KVS_MAX_INSTANCES],

    /// Working directory.
    working_dir: PathBuf,

    /// Marker for `Backend`.
    _backend_marker: PhantomData<Backend>,
}

/// KVS provider.
/// Initializes and provides KVS objects.
/// Changes are committed on `flush`.
impl<Backend: KvsBackend + KvsPathResolver> GenericKvsProvider<Backend> {
    pub fn new(working_dir: PathBuf) -> Self {
        // Initialize array of empty entries.
        let kvs_pool = [const { None }; KVS_MAX_INSTANCES];

        Self {
            kvs_pool,
            working_dir,
            _backend_marker: PhantomData,
        }
    }

    pub fn max_instances() -> usize {
        KVS_MAX_INSTANCES
    }

    /// Initialize KVS instance.
    /// On success returns initialized instance.
    pub fn init(&mut self, params: KvsParameters) -> Result<GenericKvs<Backend>, ErrorCode> {
        let instance_id = params.instance_id;
        let instance_id_index: usize = instance_id.clone().into();

        // Check given instance ID is in range, but instance is not initialized yet.
        match self.kvs_pool.get(instance_id_index) {
            Some(entry) => {
                if entry.is_some() {
                    return Err(ErrorCode::InstanceAlreadyInitialized);
                }
            }
            None => return Err(ErrorCode::InvalidInstanceId),
        }

        // Create backend object.
        let backend = Backend::new(instance_id, &self.working_dir);

        // Initialize with provided parameters.
        // Load file containing defaults.
        let defaults_map = match params.defaults {
            Defaults::Ignored => KvsMap::new(),
            Defaults::Optional => {
                if backend.defaults_file_path().exists() {
                    backend.load_defaults()?
                } else {
                    KvsMap::new()
                }
            }
            Defaults::Required => backend.load_defaults()?,
        };

        // Load KVS and hash files.
        let snapshot_id = SnapshotId(0);
        let kvs_map = match params.kvs_load {
            KvsLoad::Ignored => KvsMap::new(),
            KvsLoad::Optional => {
                if backend.kvs_file_path(&snapshot_id).exists() {
                    backend.load_kvs(&snapshot_id)?
                } else {
                    KvsMap::new()
                }
            }
            KvsLoad::Required => backend.load_kvs(&snapshot_id)?,
        };

        // Initialize entry in pool.
        let kvs_pool_entry = match self.kvs_pool.get_mut(instance_id_index) {
            Some(entry) => entry,
            None => return Err(ErrorCode::InvalidInstanceId),
        };

        let kvs_inner = kvs_pool_entry.get_or_insert(Arc::new(Mutex::new(GenericKvsInner {
            kvs_map,
            defaults_map,
            flush_on_exit: params.flush_on_exit,
        })));

        Ok(GenericKvs::new(kvs_inner.clone(), backend))
    }

    pub fn get(&self, instance_id: InstanceId) -> Result<GenericKvs<Backend>, ErrorCode> {
        let instance_id_index: usize = instance_id.clone().into();

        // Check given instance ID is in range and instance is initialized.
        match self.kvs_pool.get(instance_id_index) {
            Some(entry) => match entry {
                Some(kvs_inner) => Ok(GenericKvs::new(
                    kvs_inner.clone(),
                    Backend::new(instance_id, &self.working_dir.clone()),
                )),
                None => Err(ErrorCode::InstanceNotInitialized),
            },
            None => Err(ErrorCode::InvalidInstanceId),
        }
    }
}

#[cfg(test)]
mod kvs_parameters_tests {
    use crate::kvs_api::{Defaults, FlushOnExit, InstanceId, KvsLoad};
    use crate::kvs_provider::KvsParameters;

    #[test]
    fn test_default_values() {
        let instance_id = InstanceId(42);
        let params = KvsParameters::new(instance_id);
        assert_eq!(params.instance_id, InstanceId(42));
        assert_eq!(params.defaults, Defaults::Optional);
        assert_eq!(params.kvs_load, KvsLoad::Optional);
        assert_eq!(params.flush_on_exit, FlushOnExit::Yes);
    }

    #[test]
    fn test_defaults() {
        let instance_id = InstanceId(1);
        let params = KvsParameters::new(instance_id).defaults(Defaults::Required);
        assert_eq!(params.defaults, Defaults::Required);
    }

    #[test]
    fn test_kvs_load() {
        let instance_id = InstanceId(1);
        let params = KvsParameters::new(instance_id).kvs_load(KvsLoad::Required);
        assert_eq!(params.kvs_load, KvsLoad::Required);
    }

    #[test]
    fn test_flush_on_exit() {
        let instance_id = InstanceId(1);
        let params = KvsParameters::new(instance_id).flush_on_exit(FlushOnExit::No);
        assert_eq!(params.flush_on_exit, FlushOnExit::No);
    }

    #[test]
    fn test_chained() {
        let instance_id = InstanceId(1);
        let params = KvsParameters::new(instance_id)
            .defaults(Defaults::Ignored)
            .kvs_load(KvsLoad::Ignored)
            .flush_on_exit(FlushOnExit::No);
        assert_eq!(params.instance_id, InstanceId(1));
        assert_eq!(params.defaults, Defaults::Ignored);
        assert_eq!(params.kvs_load, KvsLoad::Ignored);
        assert_eq!(params.flush_on_exit, FlushOnExit::No);
    }
}

#[cfg(test)]
mod kvs_provider_tests {
    use crate::error_code::ErrorCode;
    use crate::json_backend::JsonBackend;
    use crate::kvs_api::{Defaults, FlushOnExit, InstanceId, KvsApi, KvsLoad, SnapshotId};
    use crate::kvs_backend::KvsPathResolver;
    use crate::kvs_provider::{GenericKvsProvider, KvsParameters, KVS_MAX_INSTANCES};
    use crate::kvs_value::KvsMap;
    use crate::KvsProvider;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;
    use tinyjson::{JsonGenerator, JsonValue};

    /// Tests reuse JSON backend.
    /// This is to ensure valid load/save behavior.
    type TestBackend = JsonBackend;

    /// KVS provider type used for tests.
    type TestKvsProvider = GenericKvsProvider<TestBackend>;

    #[test]
    fn test_new_empty_path() {
        let dir_path = PathBuf::new();
        let kvs_provider = TestKvsProvider::new(dir_path.clone());
        assert_eq!(kvs_provider.working_dir, dir_path);
    }

    #[test]
    fn test_new_defined_path() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let kvs_provider = TestKvsProvider::new(dir_path.clone());
        assert_eq!(kvs_provider.working_dir, dir_path);
    }

    #[test]
    fn test_new_empty_pool() {
        let dir_path = PathBuf::new();
        let kvs_provider = TestKvsProvider::new(dir_path.clone());
        let pool = kvs_provider.kvs_pool;
        assert!(pool.iter().all(|x| x.is_none()));
        assert_eq!(pool.len(), KVS_MAX_INSTANCES);
    }

    #[test]
    fn test_max_instances() {
        assert_eq!(TestKvsProvider::max_instances(), KVS_MAX_INSTANCES);
    }

    #[test]
    fn test_init_ok() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters = KvsParameters::new(instance_id.clone());
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());

        kvs_provider.init(kvs_parameters).unwrap();
        assert!(kvs_provider
            .kvs_pool
            .get(usize::from(instance_id))
            .is_some());
    }

    #[test]
    fn test_init_instance_already_initialized() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters = KvsParameters::new(instance_id);
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());
        kvs_provider.init(kvs_parameters.clone()).unwrap();

        let result = kvs_provider.init(kvs_parameters);
        assert!(result.is_err_and(|e| e == ErrorCode::InstanceAlreadyInitialized));
    }

    #[test]
    fn test_init_instance_id_out_of_range() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(100);
        let kvs_parameters = KvsParameters::new(instance_id);
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());

        let result = kvs_provider.init(kvs_parameters);
        assert!(result.is_err_and(|e| e == ErrorCode::InvalidInstanceId))
    }

    /// Generate and store file containing example default values.
    fn create_defaults_file(working_dir: &Path, instance_id: &InstanceId) {
        let defaults_file_path =
            TestBackend::new(instance_id.clone(), working_dir).defaults_file_path();

        let content: HashMap<String, JsonValue> = HashMap::from([
            (
                "number1".to_string(),
                JsonValue::Object(HashMap::from([
                    ("t".to_string(), JsonValue::String("f64".to_string())),
                    ("v".to_string(), JsonValue::Number(123.0)),
                ])),
            ),
            (
                "bool1".to_string(),
                JsonValue::Object(HashMap::from([
                    ("t".to_string(), JsonValue::String("bool".to_string())),
                    ("v".to_string(), JsonValue::Boolean(true)),
                ])),
            ),
            (
                "string1".to_string(),
                JsonValue::Object(HashMap::from([
                    ("t".to_string(), JsonValue::String("str".to_string())),
                    ("v".to_string(), JsonValue::String("Hello".to_string())),
                ])),
            ),
        ]);

        let json_value = JsonValue::Object(content);
        let mut buffer = Vec::new();
        let mut json_generator = JsonGenerator::new(&mut buffer).indent("  ");
        json_generator.generate(&json_value).unwrap();

        let data = String::from_utf8(buffer).unwrap();
        std::fs::write(defaults_file_path, &data).unwrap();
    }

    /// Generate and store files containing example KVS and hash data.
    fn create_kvs_files(working_dir: &Path, instance_id: &InstanceId) {
        // KVS itself is used to generate those files.
        let mut kvs_provider = KvsProvider::new(working_dir.to_path_buf());
        let kvs_parameters = KvsParameters::new(instance_id.clone()).flush_on_exit(FlushOnExit::No);
        let kvs = kvs_provider.init(kvs_parameters).unwrap();

        // Set values.
        kvs.set_value("number1", 123.0).unwrap();
        kvs.set_value("bool1", true).unwrap();
        kvs.set_value("string1", "Hello").unwrap();

        // Explicit flush.
        kvs.flush().unwrap();
    }

    #[test]
    fn test_init_defaults_ignored() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters = KvsParameters::new(instance_id).defaults(Defaults::Ignored);
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());

        kvs_provider.init(kvs_parameters).unwrap();
        let kvs_inner_option = kvs_provider.kvs_pool.get(1).cloned().unwrap();
        let kvs_inner = kvs_inner_option.unwrap();
        assert_eq!(kvs_inner.lock().unwrap().defaults_map, KvsMap::new());
    }

    #[test]
    fn test_init_defaults_optional_not_provided() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters = KvsParameters::new(instance_id).defaults(Defaults::Optional);
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());

        kvs_provider.init(kvs_parameters).unwrap();
        let kvs_inner_option = kvs_provider.kvs_pool.get(1).cloned().unwrap();
        let kvs_inner = kvs_inner_option.unwrap();
        assert_eq!(kvs_inner.lock().unwrap().defaults_map, KvsMap::new());
    }

    #[test]
    fn test_init_defaults_optional_provided() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters = KvsParameters::new(instance_id.clone()).defaults(Defaults::Optional);
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());
        create_defaults_file(&dir_path, &instance_id);

        kvs_provider.init(kvs_parameters).unwrap();
        let kvs_inner_option = kvs_provider.kvs_pool.get(1).cloned().unwrap();
        let kvs_inner = kvs_inner_option.unwrap();
        assert_eq!(kvs_inner.lock().unwrap().defaults_map.len(), 3);
    }

    #[test]
    fn test_init_defaults_required_not_provided() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters = KvsParameters::new(instance_id).defaults(Defaults::Required);
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());

        let result = kvs_provider.init(kvs_parameters);
        assert!(result.is_err_and(|e| e == ErrorCode::FileNotFound));
    }

    #[test]
    fn test_init_defaults_required_provided() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters = KvsParameters::new(instance_id.clone()).defaults(Defaults::Required);
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());
        create_defaults_file(&dir_path, &instance_id);

        kvs_provider.init(kvs_parameters).unwrap();
        let kvs_inner_option = kvs_provider.kvs_pool.get(1).cloned().unwrap();
        let kvs_inner = kvs_inner_option.unwrap();
        assert_eq!(kvs_inner.lock().unwrap().defaults_map.len(), 3);
    }

    #[test]
    fn test_init_kvs_load_ignored() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters = KvsParameters::new(instance_id).kvs_load(KvsLoad::Ignored);
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());

        kvs_provider.init(kvs_parameters).unwrap();
        let kvs_inner_option = kvs_provider.kvs_pool.get(1).cloned().unwrap();
        let kvs_inner = kvs_inner_option.unwrap();
        assert_eq!(kvs_inner.lock().unwrap().kvs_map, KvsMap::new());
    }

    #[test]
    fn test_init_kvs_load_optional_not_provided() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters = KvsParameters::new(instance_id).kvs_load(KvsLoad::Optional);
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());

        kvs_provider.init(kvs_parameters).unwrap();
        let kvs_inner_option = kvs_provider.kvs_pool.get(1).cloned().unwrap();
        let kvs_inner = kvs_inner_option.unwrap();
        assert_eq!(kvs_inner.lock().unwrap().kvs_map, KvsMap::new());
    }

    #[test]
    fn test_init_kvs_load_optional_kvs_provided_hash_not_provided() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters = KvsParameters::new(instance_id.clone()).kvs_load(KvsLoad::Optional);
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());
        create_kvs_files(&dir_path, &instance_id);
        std::fs::remove_file(
            TestBackend::new(instance_id, &dir_path).kvs_file_path(&SnapshotId(0)),
        )
        .unwrap();

        kvs_provider.init(kvs_parameters).unwrap();
        let kvs_inner_option = kvs_provider.kvs_pool.get(1).cloned().unwrap();
        let kvs_inner = kvs_inner_option.unwrap();
        assert_eq!(kvs_inner.lock().unwrap().kvs_map, KvsMap::new());
    }

    #[test]
    fn test_init_kvs_load_optional_kvs_not_provided_hash_provided() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters = KvsParameters::new(instance_id.clone()).kvs_load(KvsLoad::Optional);
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());
        create_kvs_files(&dir_path, &instance_id);
        std::fs::remove_file(
            TestBackend::new(instance_id, &dir_path).hash_file_path(&SnapshotId(0)),
        )
        .unwrap();

        let result = kvs_provider.init(kvs_parameters);
        assert!(result.is_err_and(|e| e == ErrorCode::KvsHashFileReadError));
    }

    #[test]
    fn test_init_kvs_load_optional_provided() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters = KvsParameters::new(instance_id.clone()).kvs_load(KvsLoad::Optional);
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());
        create_kvs_files(&dir_path, &instance_id);

        kvs_provider.init(kvs_parameters).unwrap();
        let kvs_inner_option = kvs_provider.kvs_pool.get(1).cloned().unwrap();
        let kvs_inner = kvs_inner_option.unwrap();
        assert_eq!(kvs_inner.lock().unwrap().kvs_map.len(), 3);
    }

    #[test]
    fn test_init_kvs_load_required_not_provided() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters = KvsParameters::new(instance_id).kvs_load(KvsLoad::Required);
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());

        let result = kvs_provider.init(kvs_parameters);
        assert!(result.is_err_and(|e| e == ErrorCode::FileNotFound));
    }

    #[test]
    fn test_init_kvs_load_required_kvs_provided_hash_not_provided() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters = KvsParameters::new(instance_id.clone()).kvs_load(KvsLoad::Required);
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());
        create_kvs_files(&dir_path, &instance_id);
        std::fs::remove_file(
            TestBackend::new(instance_id, &dir_path).kvs_file_path(&SnapshotId(0)),
        )
        .unwrap();

        let result = kvs_provider.init(kvs_parameters);
        assert!(result.is_err_and(|e| e == ErrorCode::FileNotFound));
    }

    #[test]
    fn test_init_kvs_load_required_kvs_not_provided_hash_provided() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters = KvsParameters::new(instance_id.clone()).kvs_load(KvsLoad::Required);
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());
        create_kvs_files(&dir_path, &instance_id);
        std::fs::remove_file(
            TestBackend::new(instance_id, &dir_path).hash_file_path(&SnapshotId(0)),
        )
        .unwrap();

        let result = kvs_provider.init(kvs_parameters);
        assert!(result.is_err_and(|e| e == ErrorCode::KvsHashFileReadError));
    }

    #[test]
    fn test_init_kvs_load_required_provided() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters = KvsParameters::new(instance_id.clone()).kvs_load(KvsLoad::Required);
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());
        create_kvs_files(&dir_path, &instance_id);

        kvs_provider.init(kvs_parameters).unwrap();
        let kvs_inner_option = kvs_provider.kvs_pool.get(1).cloned().unwrap();
        let kvs_inner = kvs_inner_option.unwrap();
        assert_eq!(kvs_inner.lock().unwrap().kvs_map.len(), 3);
    }

    #[test]
    fn test_init_flush_on_exit_no() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters = KvsParameters::new(instance_id).flush_on_exit(FlushOnExit::No);
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());

        let kvs = kvs_provider.init(kvs_parameters).unwrap();
        assert!(kvs.flush_on_exit().is_ok_and(|v| v == FlushOnExit::No));
    }

    #[test]
    fn test_init_flush_on_exit_yes() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters = KvsParameters::new(instance_id).flush_on_exit(FlushOnExit::Yes);
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());

        let kvs = kvs_provider.init(kvs_parameters).unwrap();
        assert!(kvs.flush_on_exit().is_ok_and(|v| v == FlushOnExit::Yes));
    }

    #[test]
    fn test_get_ok() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters = KvsParameters::new(instance_id.clone());
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());
        kvs_provider.init(kvs_parameters).unwrap();

        kvs_provider.get(instance_id).unwrap();
    }

    #[test]
    fn test_get_instance_not_initialized() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_provider = TestKvsProvider::new(dir_path.clone());

        let result = kvs_provider.get(instance_id);
        assert!(result.is_err_and(|e| e == ErrorCode::InstanceNotInitialized));
    }

    #[test]
    fn test_get_instance_id_out_of_range() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(100);
        let kvs_provider = TestKvsProvider::new(dir_path.clone());

        let result = kvs_provider.get(instance_id);
        assert!(result.is_err_and(|e| e == ErrorCode::InvalidInstanceId));
    }

    #[test]
    fn test_get_flush_on_exit_no() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters = KvsParameters::new(instance_id.clone()).flush_on_exit(FlushOnExit::No);
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());
        kvs_provider.init(kvs_parameters).unwrap();

        let kvs = kvs_provider.get(instance_id).unwrap();
        assert!(kvs.flush_on_exit().is_ok_and(|v| v == FlushOnExit::No));
    }

    #[test]
    fn test_get_flush_on_exit_yes() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let instance_id = InstanceId(1);
        let kvs_parameters =
            KvsParameters::new(instance_id.clone()).flush_on_exit(FlushOnExit::Yes);
        let mut kvs_provider = TestKvsProvider::new(dir_path.clone());
        kvs_provider.init(kvs_parameters).unwrap();

        let kvs = kvs_provider.get(instance_id).unwrap();
        assert!(kvs.flush_on_exit().is_ok_and(|v| v == FlushOnExit::Yes));
    }
}
