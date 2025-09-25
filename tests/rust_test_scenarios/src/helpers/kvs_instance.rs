//! KVS instance test helpers.

use crate::helpers::kvs_parameters::KvsParameters;
use rust_kvs::prelude::{ErrorCode, JsonBackendBuilder, Kvs, KvsBuilder};

/// Create KVS instance based on provided parameters.
pub fn kvs_instance(kvs_parameters: KvsParameters) -> Result<Kvs, ErrorCode> {
    let mut kvs_builder = KvsBuilder::new(kvs_parameters.instance_id);

    // Set `defaults` mode.
    if let Some(flag) = kvs_parameters.defaults {
        kvs_builder = kvs_builder.defaults(flag);
    }

    // Set `kvs_load` mode.
    if let Some(flag) = kvs_parameters.kvs_load {
        kvs_builder = kvs_builder.kvs_load(flag);
    }

    // Set working directory - part of backend.
    let mut backend_builder = JsonBackendBuilder::new();
    let mut set_backend = false;
    if let Some(dir) = kvs_parameters.dir {
        backend_builder = backend_builder.working_dir(dir);
        set_backend = true;
    }

    // Set max number of snapshots - part of backend.
    if let Some(snapshot_max_count) = kvs_parameters.snapshot_max_count {
        backend_builder = backend_builder.snapshot_max_count(snapshot_max_count);
        set_backend = true;
    }

    // Set backend, if backend parameters were provided.
    if set_backend {
        kvs_builder = kvs_builder.backend(Box::new(backend_builder.build()));
    }

    let kvs: Kvs = kvs_builder.build()?;

    Ok(kvs)
}
