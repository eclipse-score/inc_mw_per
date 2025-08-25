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

use rust_kvs::prelude::*;
use tempfile::tempdir;

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__multi_instance", "comp_req__persistency__concurrency"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Verifies that multiple KVS instances with different IDs store and retrieve independent values without interference.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
fn cit_persistency_multiple_instances() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Values.
    let keyname = "number".to_string();
    let value1 = 111.1;
    let value2 = 222.2;

    {
        // Create first KVS instance.
        let kvs1 = Kvs::open(
            InstanceId(0),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;
        // Create second KVS instance.
        let kvs2 = Kvs::open(
            InstanceId(1),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;

        // Set values to both KVS instances.
        kvs1.set_value(&keyname, value1)?;
        kvs2.set_value(&keyname, value2)?;
    }

    // Assertions.
    {
        // Second KVS run.
        let kvs1 = Kvs::open(
            InstanceId(0),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;
        let kvs2 = Kvs::open(
            InstanceId(1),
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;

        // Compare values, ensure they are not mixed up.
        assert_eq!(
            kvs1.get_value_as::<f64>(&keyname)?,
            value1,
            "kvs1: key '{keyname}' should have value1 {value1}"
        );
        assert_ne!(
            kvs1.get_value_as::<f64>(&keyname)?,
            value2,
            "kvs1: key '{keyname}' should not have value2 {value2}"
        );

        assert_eq!(
            kvs2.get_value_as::<f64>(&keyname)?,
            value2,
            "kvs2: key '{keyname}' should have value2 {value2}"
        );
        assert_ne!(
            kvs2.get_value_as::<f64>(&keyname)?,
            value1,
            "kvs2: key '{keyname}' should not have value1 {value1}"
        );
    }

    Ok(())
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__multi_instance", "comp_req__persistency__concurrency"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Checks that multiple KVS instances with the same ID and key maintain consistent values across instances.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
fn cit_persistency_multiple_instances_same_id_common_value() -> Result<(), ErrorCode> {
    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Values.
    let common_keyname = "number".to_string();
    let common_value = 100.0;

    let instance_id = InstanceId(0);
    {
        // Create first KVS instance.
        let kvs1 = Kvs::open(
            instance_id,
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;
        // Create second KVS instance.
        let kvs2 = Kvs::open(
            instance_id,
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;

        // Set values to both KVS instances.
        kvs1.set_value(&common_keyname, common_value)?;
        kvs2.set_value(&common_keyname, common_value)?;
    }

    // Assertions.
    {
        // Second KVS run.
        let kvs1 = Kvs::open(
            instance_id,
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;
        let kvs2 = Kvs::open(
            instance_id,
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;

        assert_eq!(
            kvs1.get_value_as::<f64>(&common_keyname)?,
            common_value,
            "kvs1: key '{common_keyname}' should have common_value {common_value}"
        );
        assert_eq!(
            kvs2.get_value_as::<f64>(&common_keyname)?,
            common_value,
            "kvs2: key '{common_keyname}' should have common_value {common_value}"
        );
    }

    Ok(())
}

#[test]
// #[record_property("PartiallyVerifies", ["comp_req__persistency__multi_instance", "comp_req__persistency__concurrency"])]
// #[record_property("FullyVerifies", [""])]
// #[record_property("Description", "Verifies that changes in one KVS instance with a shared ID and key are reflected in another instance, demonstrating interference.")]
// #[record_property("TestType", "requirements-based")]
// #[record_property("DerivationTechnique", "requirements-based")]
#[ignore = "https://github.com/qorix-group/inc_mw_per/issues/20"]
fn cit_persistency_multiple_instances_same_id_interfere() -> Result<(), ErrorCode> {
    // TODO: https://github.com/qorix-group/inc_mw_per/issues/20

    // Temp directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Values.
    let keyname = "number1".to_string();
    let value1 = 111.1;
    let value2 = 222.2;

    let instance_id = InstanceId(0);
    {
        // Create first KVS instance.
        let kvs1 = Kvs::open(
            instance_id,
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;
        // Create second KVS instance.
        let kvs2 = Kvs::open(
            instance_id,
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;

        // Set values to both KVS instances.
        kvs1.set_value(&keyname, value1)?;
        kvs2.set_value(&keyname, value1)?;
    }

    // Assertions.
    {
        // Second KVS run.
        let kvs1 = Kvs::open(
            instance_id,
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;
        let kvs2 = Kvs::open(
            instance_id,
            OpenNeedDefaults::Optional,
            OpenNeedKvs::Optional,
            Some(dir_string.clone()),
        )?;

        // Change value in first KVS instance.
        // This should affect the second KVS instance as well,
        // since they share the same instance ID and key.
        kvs1.set_value(&keyname, value2)?;
        kvs1.flush()?;

        assert_eq!(
            kvs1.get_value_as::<f64>(&keyname)?,
            value2,
            "kvs1: key '{keyname}' should have value {value2}"
        );
        assert_eq!(
            kvs2.get_value_as::<f64>(&keyname)?,
            value2,
            "kvs2: key '{keyname}' should have value {value2}"
        );
    }

    Ok(())
}
