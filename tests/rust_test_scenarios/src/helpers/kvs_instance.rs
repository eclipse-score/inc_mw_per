//! KVS instance test helpers.

use crate::helpers::kvs_parameters::KvsParameters;
use rust_kvs::prelude::{ErrorCode, Kvs, KvsBuilder, KvsMap, KvsValue};

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

    // Set `backend_parameters`.
    let mut backend_parameters =
        KvsMap::from([("name".to_string(), KvsValue::String("json".to_string()))]);
    let mut set_backend = false;

    // Set working directory.
    if let Some(dir) = kvs_parameters.dir {
        backend_parameters.insert(
            "working_dir".to_string(),
            KvsValue::String(dir.to_string_lossy().to_string()),
        );
        set_backend = true;
    }

    // Set max number of snapshots.
    if let Some(snapshot_max_count) = kvs_parameters.snapshot_max_count {
        backend_parameters.insert(
            "snapshot_max_count".to_string(),
            KvsValue::U64(snapshot_max_count as u64),
        );
        set_backend = true;
    }

    // Set backend, if backend parameters were provided.
    if set_backend {
        kvs_builder = kvs_builder.backend_parameters(backend_parameters);
    }

    let kvs: Kvs = kvs_builder.build()?;

    Ok(kvs)
}
