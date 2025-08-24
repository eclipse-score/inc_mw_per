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
use std::sync::{Arc, LazyLock, Mutex, MutexGuard, PoisonError};

/// Maximum number of instances.
const KVS_MAX_INSTANCES: usize = 10;

/// KVS instance parameters.
/// Expected to be cloned into new KVS instances.
#[derive(Clone, PartialEq)]
pub(crate) struct KvsParameters {
    /// Instance ID.
    pub(crate) instance_id: InstanceId,

    /// Defaults handling mode.
    pub(crate) defaults: Defaults,

    /// KVS load mode.
    pub(crate) kvs_load: KvsLoad,

    /// Flush on exit mode.
    pub(crate) flush_on_exit: FlushOnExit,

    /// Working directory.
    pub(crate) working_dir: PathBuf,
}

/// KVS instance data.
/// Expected to be shared between instance pool and instances.
pub(crate) struct KvsData {
    /// Storage data.
    pub(crate) kvs_map: KvsMap,

    /// Optional default values.
    pub(crate) defaults_map: KvsMap,
}

impl From<PoisonError<MutexGuard<'_, KvsData>>> for ErrorCode {
    fn from(_cause: PoisonError<MutexGuard<'_, KvsData>>) -> Self {
        ErrorCode::MutexLockFailed
    }
}

/// KVS instance inner representation.
pub(crate) struct KvsInner {
    /// KVS instance parameters.
    pub(crate) parameters: KvsParameters,

    /// KVS instance data.
    pub(crate) data: Arc<Mutex<KvsData>>,
}

static KVS_POOL: LazyLock<Mutex<[Option<KvsInner>; KVS_MAX_INSTANCES]>> =
    LazyLock::new(|| Mutex::new([const { None }; KVS_MAX_INSTANCES]));

impl From<PoisonError<MutexGuard<'_, [Option<KvsInner>; KVS_MAX_INSTANCES]>>> for ErrorCode {
    fn from(_cause: PoisonError<MutexGuard<'_, [Option<KvsInner>; KVS_MAX_INSTANCES]>>) -> Self {
        ErrorCode::MutexLockFailed
    }
}

/// Key-value-storage builder.
pub struct GenericKvsBuilder<Backend: KvsBackend, PathResolver: KvsPathResolver = Backend> {
    /// KVS instance parameters.
    parameters: KvsParameters,

    /// Marker for `Backend`.
    _backend_marker: PhantomData<Backend>,

    /// Marker for `PathResolver`.
    _path_resolver_marker: PhantomData<PathResolver>,
}

impl<Backend: KvsBackend, PathResolver: KvsPathResolver> GenericKvsBuilder<Backend, PathResolver> {
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
        let parameters = KvsParameters {
            instance_id,
            defaults: Defaults::Optional,
            kvs_load: KvsLoad::Optional,
            flush_on_exit: FlushOnExit::Yes,
            working_dir: PathBuf::new(),
        };

        Self {
            parameters,
            _backend_marker: PhantomData,
            _path_resolver_marker: PhantomData,
        }
    }

    /// Configure defaults handling mode.
    ///
    /// # Parameters
    ///   * `mode`: defaults handling mode (default: [`Defaults::Optional`](Defaults::Optional))
    ///
    /// # Return Values
    ///   * KvsBuilder instance
    pub fn defaults(mut self, mode: Defaults) -> Self {
        self.parameters.defaults = mode;
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
        self.parameters.kvs_load = mode;
        self
    }

    /// Set the key-value-storage permanent storage directory
    ///
    /// # Parameters
    ///   * `dir`: Path to permanent storage
    ///
    /// # Return Values
    pub fn dir<P: Into<String>>(mut self, dir: P) -> Self {
        self.parameters.working_dir = PathBuf::from(dir.into());
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
    pub fn build(self) -> Result<GenericKvs<Backend, PathResolver>, ErrorCode> {
        let instance_id = self.parameters.clone().instance_id;
        let instance_id_index: usize = instance_id.into();
        let working_dir = self.parameters.clone().working_dir;

        // Check if instance already exists.
        {
            let kvs_pool = KVS_POOL.lock()?;
            let kvs_inner_option = match kvs_pool.get(instance_id_index) {
                Some(kvs_pool_entry) => match kvs_pool_entry {
                    // If instance exists then parameters must match.
                    Some(kvs_inner) => {
                        if kvs_inner.parameters == self.parameters {
                            Ok(Some(kvs_inner))
                        } else {
                            Err(ErrorCode::InstanceParametersMismatch)
                        }
                    }
                    // Instance not found - not an error, will initialize later.
                    None => Ok(None),
                },
                // Instance ID out of range.
                None => Err(ErrorCode::InvalidInstanceId),
            }?;

            // Return existing instance if initialized.
            if let Some(kvs_inner) = kvs_inner_option {
                return Ok(GenericKvs::<Backend, PathResolver>::new(
                    kvs_inner.data.clone(),
                    kvs_inner.parameters.clone(),
                ));
            }
        }

        // Initialize KVS instance with provided parameters.
        // Load file containing defaults.
        let defaults_path = PathResolver::defaults_file_path(&working_dir, instance_id);
        let defaults_map = match self.parameters.defaults {
            Defaults::Ignored => KvsMap::new(),
            Defaults::Optional => {
                if defaults_path.exists() {
                    Backend::load_kvs(&defaults_path, None)?
                } else {
                    KvsMap::new()
                }
            }
            Defaults::Required => Backend::load_kvs(&defaults_path, None)?,
        };

        // Load KVS and hash files.
        let snapshot_id = SnapshotId(0);
        let kvs_path = PathResolver::kvs_file_path(&working_dir, instance_id, snapshot_id);
        let hash_path = PathResolver::hash_file_path(&working_dir, instance_id, snapshot_id);
        let kvs_map = match self.parameters.kvs_load {
            KvsLoad::Ignored => KvsMap::new(),
            KvsLoad::Optional => {
                if kvs_path.exists() {
                    Backend::load_kvs(&kvs_path, Some(&hash_path))?
                } else {
                    KvsMap::new()
                }
            }
            KvsLoad::Required => Backend::load_kvs(&kvs_path, Some(&hash_path))?,
        };

        // Shared object containing data.
        let data = Arc::new(Mutex::new(KvsData {
            kvs_map,
            defaults_map,
        }));

        // Initialize entry in pool and return new KVS instance.
        {
            let mut kvs_pool = KVS_POOL.lock()?;
            let kvs_pool_entry = match kvs_pool.get_mut(instance_id_index) {
                Some(entry) => entry,
                None => return Err(ErrorCode::InvalidInstanceId),
            };

            let _ = kvs_pool_entry.insert(KvsInner {
                parameters: self.parameters.clone(),
                data: data.clone(),
            });
        }

        Ok(GenericKvs::new(data, self.parameters))
    }
}

#[cfg(test)]
mod kvs_builder_tests {
    use crate::error_code::ErrorCode;
    use crate::kvs_api::{Defaults, InstanceId, KvsLoad, SnapshotId};
    use crate::kvs_backend::{KvsBackend, KvsPathResolver};
    use crate::kvs_builder::GenericKvsBuilder;
    use crate::kvs_value::KvsMap;
    use std::path::PathBuf;

    struct StubBackend;

    impl KvsBackend for StubBackend {
        fn load_kvs(
            _kvs_path: &std::path::Path,
            _hash_path: Option<&PathBuf>,
        ) -> Result<KvsMap, ErrorCode> {
            Ok(KvsMap::new())
        }

        fn save_kvs(
            _kvs_map: &KvsMap,
            _kvs_path: &std::path::Path,
            _hash_path: Option<&PathBuf>,
        ) -> Result<(), ErrorCode> {
            Ok(())
        }
    }

    impl KvsPathResolver for StubBackend {
        fn kvs_file_name(_instance_id: InstanceId, _snapshot_id: SnapshotId) -> String {
            String::new()
        }

        fn kvs_file_path(
            _working_dir: &std::path::Path,
            _instance_id: InstanceId,
            _snapshot_id: SnapshotId,
        ) -> PathBuf {
            PathBuf::new()
        }

        fn hash_file_name(_instance_id: InstanceId, _snapshot_id: SnapshotId) -> String {
            String::new()
        }

        fn hash_file_path(
            _working_dir: &std::path::Path,
            _instance_id: InstanceId,
            _snapshot_id: SnapshotId,
        ) -> PathBuf {
            PathBuf::new()
        }

        fn defaults_file_name(_instance_id: InstanceId) -> String {
            String::new()
        }

        fn defaults_file_path(
            _working_dir: &std::path::Path,
            _instance_id: InstanceId,
        ) -> PathBuf {
            PathBuf::new()
        }
    }

    #[test]
    fn test_builder_only_instance_id() {
        let instance_id = InstanceId(1);
        let builder = GenericKvsBuilder::<StubBackend>::new(instance_id);
        let kvs = builder.build().unwrap();
        assert_eq!(kvs.parameters().instance_id, instance_id);
        assert_eq!(kvs.parameters().defaults, Defaults::Optional);
        assert_eq!(kvs.parameters().kvs_load, KvsLoad::Optional);
        assert_eq!(kvs.parameters().working_dir, PathBuf::new());
    }

    #[test]
    fn test_builder_defaults() {
        let instance_id = InstanceId(1);
        let builder =
            GenericKvsBuilder::<StubBackend>::new(instance_id).defaults(Defaults::Required);
        let kvs = builder.build().unwrap();
        assert_eq!(kvs.parameters().instance_id, instance_id);
        assert_eq!(kvs.parameters().defaults, Defaults::Required);
        assert_eq!(kvs.parameters().kvs_load, KvsLoad::Optional);
        assert_eq!(kvs.parameters().working_dir, PathBuf::new());
    }

    #[test]
    fn test_builder_kvs_load() {
        let instance_id = InstanceId(1);
        let builder =
            GenericKvsBuilder::<StubBackend>::new(instance_id).kvs_load(KvsLoad::Required);
        let kvs = builder.build().unwrap();
        assert_eq!(kvs.parameters().instance_id, instance_id);
        assert_eq!(kvs.parameters().defaults, Defaults::Optional);
        assert_eq!(kvs.parameters().kvs_load, KvsLoad::Required);
        assert_eq!(kvs.parameters().working_dir, PathBuf::new());
    }

    #[test]
    fn test_builder_dir() {
        let instance_id = InstanceId(1);
        let dir = "/tmp/test_kvs".to_string();
        let builder = GenericKvsBuilder::<StubBackend>::new(instance_id).dir(dir.clone());
        let kvs = builder.build().unwrap();
        assert_eq!(kvs.parameters().instance_id, instance_id);
        assert_eq!(kvs.parameters().defaults, Defaults::Optional);
        assert_eq!(kvs.parameters().kvs_load, KvsLoad::Optional);
        assert_eq!(kvs.parameters().working_dir, PathBuf::from(dir));
    }

    #[test]
    fn test_builder_chained() {
        let instance_id = InstanceId(1);
        let dir = "/tmp/test_kvs".to_string();
        let builder = GenericKvsBuilder::<StubBackend>::new(instance_id)
            .defaults(Defaults::Required)
            .kvs_load(KvsLoad::Required)
            .dir(dir.clone());
        let kvs = builder.build().unwrap();
        assert_eq!(kvs.parameters().instance_id, instance_id);
        assert_eq!(kvs.parameters().defaults, Defaults::Required);
        assert_eq!(kvs.parameters().kvs_load, KvsLoad::Required);
        assert_eq!(kvs.parameters().working_dir, PathBuf::from(dir));
    }
}
