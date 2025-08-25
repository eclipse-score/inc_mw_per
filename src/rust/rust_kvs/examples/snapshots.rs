//! Example for snapshots handling.
//! - Snapshot count and max count.
//! - Snapshot restore.

use rust_kvs::prelude::*;
use tempfile::tempdir;

fn main() -> Result<(), ErrorCode> {
    // Temporary directory.
    let dir = tempdir()?;
    let dir_path = dir.path().to_path_buf();

    // Instance ID for KVS object instances.
    let instance_id = InstanceId(0);

    {
        println!("-> `snapshot_count` and `snapshot_max_count` usage");

        // Build KVS instance for given instance ID and temporary directory.
        let mut provider = KvsProvider::new(dir_path.clone());
        let params = KvsParameters::new(instance_id.clone());
        let kvs = provider.init(params)?;
        kvs.set_flush_on_exit(FlushOnExit::No)?;

        let max_count = Kvs::snapshot_max_count() as u32;
        println!("Max snapshot count: {max_count:?}");

        // Snapshots are created and rotated on flush.
        let counter_key = "counter";
        for index in 0..max_count {
            kvs.set_value(counter_key, index)?;
            kvs.flush()?;
            println!("Snapshot count: {:?}", kvs.snapshot_count());
        }

        println!();
    }

    {
        println!("-> `snapshot_restore` usage");

        // Build KVS instance for given instance ID and temporary directory.
        let mut provider = KvsProvider::new(dir_path.clone());
        let params = KvsParameters::new(instance_id.clone());
        let kvs = provider.init(params)?;
        kvs.set_flush_on_exit(FlushOnExit::No)?;

        let max_count = Kvs::snapshot_max_count() as u32;
        let counter_key = "counter";
        for index in 0..max_count {
            kvs.set_value(counter_key, index)?;
            kvs.flush()?;
        }

        // Print current counter value, then restore oldest snapshot.
        println!("{counter_key} = {:?}", kvs.get_value(counter_key)?);
        kvs.snapshot_restore(&SnapshotId(2))?;
        println!("{counter_key} = {:?}", kvs.get_value(counter_key)?);

        println!();
    }

    Ok(())
}
