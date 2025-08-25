//! Example for flush.
//! - Behavior for flush enabled and disabled.
//! - Underlying storage file access.

use rust_kvs::prelude::*;
use std::path::PathBuf;
use tempfile::tempdir;

/// Utility function returning KVS file path pair - KVS and hash.
fn get_kvs_path(dir_path: PathBuf, instance_id: InstanceId) -> (PathBuf, PathBuf) {
    let snapshot_id = SnapshotId(0);
    let kvs_name = format!("kvs_{instance_id}_{snapshot_id}.json");
    let kvs_path = dir_path.join(kvs_name);
    let hash_name = format!("kvs_{instance_id}_{snapshot_id}.hash");
    let hash_path = dir_path.join(hash_name);

    (kvs_path, hash_path)
}

fn main() -> Result<(), ErrorCode> {
    // Temporary directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Instance ID for KVS object instances.
    let instance_id = InstanceId(0);

    // KVS and hash file paths.
    let (kvs_path, hash_path) = get_kvs_path(dir.path().to_path_buf(), instance_id);

    {
        println!("-> disabled `flush_on_exit`");

        {
            // Build KVS instance for given instance ID and temporary directory.
            let builder = KvsBuilder::<Kvs>::new(instance_id).dir(dir_string.clone());
            let kvs = builder.build()?;

            // Disable flush on exit.
            kvs.set_flush_on_exit(FlushOnExit::No);

            // Set value - will be dropped on going out of scope.
            kvs.set_value("k1", "v1")?;
        }
        if !kvs_path.exists() && !hash_path.exists() {
            println!("KVS files not flushed");
        }

        println!();
    }

    {
        println!("-> enabled `flush_on_exit`");

        {
            // Build KVS instance for given instance ID and temporary directory.
            let builder = KvsBuilder::<Kvs>::new(instance_id).dir(dir_string.clone());
            let kvs = builder.build()?;

            // Explicitly enable flush on exit - this is the default.
            kvs.set_flush_on_exit(FlushOnExit::Yes);

            // Set value.
            kvs.set_value("k1", "v1")?;
        }
        if kvs_path.exists() && hash_path.exists() {
            println!("KVS files flushed");
        }

        println!();
    }

    {
        println!("-> `flush` usage");

        {
            // Build KVS instance for given instance ID and temporary directory.
            let builder = KvsBuilder::<Kvs>::new(instance_id).dir(dir_string.clone());
            let kvs = builder.build()?;

            // Disable flush on exit.
            kvs.set_flush_on_exit(FlushOnExit::No);

            // Set value.
            kvs.set_value("k2", "v2")?;

            // Explicitly flush content.
            kvs.flush()?;
            println!("KVS explicitly flushed")
        }

        {
            // Build KVS instance to check current state.
            let builder = KvsBuilder::<Kvs>::new(instance_id)
                .dir(dir_string)
                .kvs_load(KvsLoad::Required);
            let kvs = builder.build()?;
            kvs.set_flush_on_exit(FlushOnExit::No);

            let k1_key = "k1";
            let k1_value = kvs.get_value(k1_key)?;
            let k2_key = "k2";
            let k2_value = kvs.get_value(k2_key)?;
            println!("Content of new KVS instance:");
            println!("{k1_key:?} = {k1_value:?}");
            println!("{k2_key:?} = {k2_value:?}");
        }

        println!();
    }

    Ok(())
}
