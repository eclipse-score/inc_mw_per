//! KVS instance test helpers.

use crate::helpers::kvs_parameters::KvsParameters;
use rust_kvs::prelude::{ErrorCode, Kvs, KvsApi, KvsBuilder};

/// Create KVS instance based on provided parameters.
pub fn kvs_instance(kvs_parameters: KvsParameters) -> Result<Kvs, ErrorCode> {
    let mut builder = KvsBuilder::new(kvs_parameters.instance_id);

    if let Some(flag) = kvs_parameters.defaults {
        builder = builder.defaults(flag);
    }

    if let Some(flag) = kvs_parameters.kvs_load {
        builder = builder.kvs_load(flag);
    }

    if let Some(dir) = kvs_parameters.dir {
        builder = builder.dir(dir.to_string_lossy().to_string());
    }

    let kvs: Kvs = builder.build()?;

    if let Some(flag) = kvs_parameters.flush_on_exit {
        kvs.set_flush_on_exit(flag);
    }

    Ok(kvs)
}
