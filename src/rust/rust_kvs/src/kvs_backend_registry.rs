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
use crate::kvs_backend::KvsBackendFactory;
use crate::kvs_value::{KvsMap, KvsValue};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard, PoisonError};

/// Function providing backend factory.
type KvsBackendFactoryFn = fn() -> Box<dyn KvsBackendFactory>;

/// Map containing names as strings and factory-creating functions as values.
type BackendMap = HashMap<String, KvsBackendFactoryFn>;

/// Provide map containing default backends.
fn default_backends() -> BackendMap {
    let mut backends: BackendMap = HashMap::new();
    // Register JSON backend.
    {
        use crate::json_backend::JsonBackendFactory;
        backends.insert("json".to_string(), || Box::new(JsonBackendFactory));
    }

    backends
}

/// Pool containing registered backend factories.
static REGISTERED_BACKENDS: LazyLock<Mutex<BackendMap>> =
    LazyLock::new(|| Mutex::new(default_backends()));

impl From<PoisonError<MutexGuard<'_, BackendMap>>> for ErrorCode {
    fn from(_cause: PoisonError<MutexGuard<'_, BackendMap>>) -> Self {
        ErrorCode::MutexLockFailed
    }
}

/// KVS backend registry.
pub struct KvsBackendRegistry;

impl KvsBackendRegistry {
    /// Get registered backend using name.
    pub(crate) fn from_name(name: &str) -> Result<Box<dyn KvsBackendFactory>, ErrorCode> {
        let registered_backends = REGISTERED_BACKENDS.lock()?;

        match registered_backends.get(name) {
            Some(backend_factory_fn) => Ok(backend_factory_fn()),
            None => Err(ErrorCode::UnknownBackend),
        }
    }

    /// Get registered backend using 'name' field from parameters.
    pub(crate) fn from_parameters(
        parameters: &KvsMap,
    ) -> Result<Box<dyn KvsBackendFactory>, ErrorCode> {
        let name = match parameters.get("name") {
            Some(value) => match value {
                KvsValue::String(name) => Ok(name),
                _ => Err(ErrorCode::InvalidBackendParameters),
            },
            None => Err(ErrorCode::KeyNotFound),
        }?;

        Self::from_name(name)
    }

    /// Register new backend factory.
    pub fn register(name: &str, backend_factory_fn: KvsBackendFactoryFn) -> Result<(), ErrorCode> {
        let mut registered_backends = REGISTERED_BACKENDS.lock()?;

        // Check backend factory already registered.
        if registered_backends.contains_key(name) {
            return Err(ErrorCode::BackendAlreadyRegistered);
        }

        // Insert backend factory.
        registered_backends.insert(name.to_string(), backend_factory_fn);
        Ok(())
    }
}

#[cfg(test)]
mod registry_tests {
    use crate::error_code::ErrorCode;
    use crate::kvs_api::{InstanceId, SnapshotId};
    use crate::kvs_backend::{KvsBackend, KvsBackendFactory};
    use crate::kvs_backend_registry::{default_backends, KvsBackendRegistry, REGISTERED_BACKENDS};
    use crate::kvs_value::{KvsMap, KvsValue};
    use std::ops::DerefMut;
    use std::sync::{LazyLock, Mutex, MutexGuard};

    /// Serial test execution mutex.
    static SERIAL_TEST: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    /// Execute test serially with registry initialized to default.
    fn lock_and_reset<'a>() -> MutexGuard<'a, ()> {
        // Tests in this group must be executed serially.
        let serial_lock: MutexGuard<'a, ()> = SERIAL_TEST.lock().unwrap();

        // Reset `REGISTERED_BACKENDS` state to default.
        // This is to mitigate `BackendAlreadyRegistered` errors between tests.
        let mut registry = REGISTERED_BACKENDS.lock().unwrap();
        *registry.deref_mut() = default_backends();

        serial_lock
    }

    /// Mock backend.
    struct MockBackend {
        parameters: KvsMap,
    }

    impl KvsBackend for MockBackend {
        /// `load_kvs` is reused to access parameters used by factory.
        fn load_kvs(
            &self,
            _instance_id: InstanceId,
            _snapshot_id: SnapshotId,
        ) -> Result<KvsMap, ErrorCode> {
            Ok(self.parameters.clone())
        }

        fn load_defaults(&self, _instance_id: InstanceId) -> Result<KvsMap, ErrorCode> {
            unimplemented!()
        }

        fn flush(&self, _instance_id: InstanceId, _kvs_map: &KvsMap) -> Result<(), ErrorCode> {
            unimplemented!()
        }

        fn snapshot_count(&self, _instance_id: InstanceId) -> usize {
            unimplemented!()
        }

        fn snapshot_max_count(&self) -> usize {
            unimplemented!()
        }

        fn snapshot_restore(
            &self,
            _instance_id: InstanceId,
            _snapshot_id: SnapshotId,
        ) -> Result<KvsMap, ErrorCode> {
            unimplemented!()
        }
    }

    /// Mock backend factory.
    struct MockBackendFactory;

    impl KvsBackendFactory for MockBackendFactory {
        fn create(&self, parameters: &KvsMap) -> Result<Box<dyn KvsBackend>, ErrorCode> {
            Ok(Box::new(MockBackend {
                parameters: parameters.clone(),
            }))
        }
    }

    #[test]
    fn test_from_name_ok() {
        let _lock = lock_and_reset();

        let result = KvsBackendRegistry::from_name("json");
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_name_unknown() {
        let _lock = lock_and_reset();

        let result = KvsBackendRegistry::from_name("unknown");
        assert!(result.is_err_and(|e| e == ErrorCode::UnknownBackend));
    }

    #[test]
    fn test_from_parameters_ok() {
        let _lock = lock_and_reset();

        let params = KvsMap::from([("name".to_string(), KvsValue::String("json".to_string()))]);
        let result = KvsBackendRegistry::from_parameters(&params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_parameters_unknown() {
        let _lock = lock_and_reset();

        let params = KvsMap::from([("name".to_string(), KvsValue::String("unknown".to_string()))]);
        let result = KvsBackendRegistry::from_parameters(&params);
        assert!(result.is_err_and(|e| e == ErrorCode::UnknownBackend));
    }

    #[test]
    fn test_from_parameters_invalid_type() {
        let _lock = lock_and_reset();

        let params = KvsMap::from([("name".to_string(), KvsValue::I64(123))]);
        let result = KvsBackendRegistry::from_parameters(&params);
        assert!(result.is_err_and(|e| e == ErrorCode::InvalidBackendParameters));
    }

    #[test]
    fn test_from_parameters_missing_name() {
        let _lock = lock_and_reset();

        let params = KvsMap::new();
        let result = KvsBackendRegistry::from_parameters(&params);
        assert!(result.is_err_and(|e| e == ErrorCode::KeyNotFound));
    }

    #[test]
    fn test_register_ok() {
        let _lock = lock_and_reset();

        let result = KvsBackendRegistry::register("mock", || Box::new(MockBackendFactory));
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_already_registered() {
        let _lock = lock_and_reset();

        KvsBackendRegistry::register("mock", || Box::new(MockBackendFactory)).unwrap();
        let result = KvsBackendRegistry::register("mock", || Box::new(MockBackendFactory));
        assert!(result.is_err_and(|e| e == ErrorCode::BackendAlreadyRegistered))
    }
}
