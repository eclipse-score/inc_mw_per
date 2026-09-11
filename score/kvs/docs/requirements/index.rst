..
   # *******************************************************************************
   # Copyright (c) 2025 Contributors to the Eclipse Foundation
   #
   # See the NOTICE file(s) distributed with this work for additional
   # information regarding copyright ownership.
   #
   # This program and the accompanying materials are made available under the
   # terms of the Apache License Version 2.0 which is available at
   # https://www.apache.org/licenses/LICENSE-2.0
   #
   # SPDX-License-Identifier: Apache-2.0
   # *******************************************************************************

Requirements
############

.. document:: KVS Requirements
   :id: doc__kvs_requirements
   :status: valid
   :version: 1
   :safety: ASIL_B
   :security: NO
   :realizes: wp__requirements_comp[version==1]

Component Requirements
----------------------

.. comp_req:: Key Naming
   :id: comp_req__kvs__key_naming
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__support_datatype_keys[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall accept keys that consist solely of alphanumeric characters, underscores, or dashes.

.. comp_req:: Key Encoding
   :id: comp_req__kvs__key_encoding
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__support_datatype_keys[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall encode each key as valid UTF-8.

.. comp_req:: Key Uniqueness
   :id: comp_req__kvs__key_uniqueness
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__support_datatype_keys[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall guarantee that each key is unique.

.. comp_req:: Key Length
   :id: comp_req__kvs__key_length
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__support_datatype_keys[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall limit the maximum length of a key to 32 bytes.

.. comp_req:: Value Data Types
   :id: comp_req__kvs__value_data_types
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__support_datatype_value[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall accept only values of the following data types: Number,
   String, Null, Array[Value], or Dictionary{Key:Value}.

.. comp_req:: Value Serialization
   :id: comp_req__kvs__value_serialize
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__support_datatype_value[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall serialize and deserialize all values to and from JSON.

.. comp_req:: Value Length
   :id: comp_req__kvs__value_length
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__support_datatype_value[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall limit the maximum length of a value to 1024 bytes.

.. comp_req:: Value Default
   :id: comp_req__kvs__value_default
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__support_datatype_value[version==1],feat_req__persistency__default_values[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall support unset values and shall provide a default value
   when a value is unset.

.. comp_req:: Value Reset
   :id: comp_req__kvs__value_reset
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__support_datatype_value[version==1],feat_req__persistency__default_values[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall allow resetting a value to its default if a default is
   defined.

.. comp_req:: Default Value Datatypes
   :id: comp_req__kvs__default_value_types
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__default_values[version==1],feat_req__persistency__default_value_get[version==1],feat_req__persistency__reset_to_default[version==1],feat_req__persistency__default_value_file[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall accept default values of only permitted value data
   types.

.. comp_req:: Default Value Query
   :id: comp_req__kvs__default_value_query
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__default_values[version==1],feat_req__persistency__default_value_get[version==1],feat_req__persistency__reset_to_default[version==1],feat_req__persistency__default_value_file[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall provide an API to retrieve default values.

.. comp_req:: Default Value Config
   :id: comp_req__kvs__default_value_cfg
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__default_values[version==1],feat_req__persistency__default_value_get[version==1],feat_req__persistency__reset_to_default[version==1],feat_req__persistency__default_value_file[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall allow configuration of default values in code or in a
   separate configuration file.

.. comp_req:: Default Value Checksum
   :id: comp_req__kvs__default_val_chksum
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__default_values[version==1],feat_req__persistency__default_value_get[version==1],feat_req__persistency__reset_to_default[version==1],feat_req__persistency__default_value_file[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall secure the configuration file for default values with an
   associated checksum file when default values are stored in a file.

.. comp_req:: Constraint Configuration
   :id: comp_req__kvs__constraints
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__cfg[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall allow configuration of KVS constraints at compile-time
   using source code constants or at runtime using a configuration file.

.. comp_req:: Concurrency
   :id: comp_req__kvs__concurrency
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__concurrency[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall implement thread-safe mechanisms to enable concurrent
   access to data without data races.

.. comp_req:: Multi-Instance
   :id: comp_req__kvs__multi_instance
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__multiple_kvs[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall manage all runtime variables within an instance to
   enable creation and use of multiple KVS instances concurrently within a
   single software architecture element.

.. comp_req:: Persistent Data Storage Components
   :id: comp_req__kvs__persist_data_com
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__integrity_check[version==1],feat_req__persistency__store_data[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall use the file API and the JSON data format to persist data.

.. comp_req:: Persistent Data Storage Checksum Write
   :id: comp_req__kvs__pers_data_csum
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__integrity_check[version==1],feat_req__persistency__store_data[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall generate a checksum for each data file and shall store
   it alongside the data.

.. comp_req:: Persistent Data Storage Checksum Verify
   :id: comp_req__kvs__pers_data_csum_vrfy
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__integrity_check[version==1],feat_req__persistency__load_data[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall verify the checksum when loading data.

.. comp_req:: Persistent Data Storage Backend
   :id: comp_req__kvs__pers_data_store_bnd
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__integrity_check[version==1],feat_req__persistency__store_data[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall use the file API to persist data.

.. comp_req:: Persistent Data Storage Format
   :id: comp_req__kvs__pers_data_store_fmt
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__integrity_check[version==1],feat_req__persistency__store_data[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall use the JSON data format to persist data.

.. comp_req:: Persistent Data Versioning
   :id: comp_req__kvs__pers_data_version
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__versioning[version==1],feat_req__persistency__update_mechanism[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall not provide built-in versioning.

.. comp_req:: Persistent Data Schema
   :id: comp_req__kvs__pers_data_schema
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__versioning[version==1],feat_req__persistency__update_mechanism[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall use a JSON file storage format that enables the
   application to implement versioning, including upgrade and downgrade paths,
   as needed.

.. comp_req:: Snapshot maximum number
   :id: comp_req__kvs__snapshot_max_num_api
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__cfg[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: valid

   The component shall maintain a configurable maximum number of snapshots. The maximum number shall be in the range <0..3>.
   A value of zero means that snapshot operations are disabled, while a non-zero value specifies the maximum number of snapshots.

.. comp_req:: Snapshot create API
   :id: comp_req__kvs__snapshot_create_api
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :status: valid
   :version: 1
   :derived_from: feat_req__persistency__snapshot_create[version==1]
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: valid

   The component shall provide API for creation of snapshots with an argument that selects a dedicated snapshot slot by snapshot index.

   .. note::

      A snapshot is a point-in-time, frozen view on all values in a key-value storage.

.. comp_req:: Snapshot create
   :id: comp_req__kvs__snapshot_create
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :status: valid
   :version: 1
   :derived_from: feat_req__persistency__snapshot_create[version==1]
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: valid

   The component shall create a new snapshot in the selected snapshot slot when the slot is empty.

.. comp_req:: Snapshot overwrite
   :id: comp_req__kvs__snapshot_overwrite
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :status: valid
   :version: 1
   :derived_from: feat_req__persistency__snapshot_create[version==1]
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: valid

   The component shall overwrite the selected snapshot slot when the slot is occupied.

.. comp_req:: Snapshot restore API
   :id: comp_req__kvs__snapshot_restore_api
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :status: valid
   :version: 1
   :derived_from: feat_req__persistency__snapshot_restore[version==1]
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: valid

   The component shall provide API for restoration of snapshots with an argument that selects a dedicated snapshot slot by snapshot index.
   The function shall return an error, when referenced snapshot slot is free.

.. comp_req:: Snapshot remove API
   :id: comp_req__kvs__snapshot_remove_api
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :status: valid
   :version: 1
   :derived_from: feat_req__persistency__snapshot_remove[version==1]
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: valid

   The component shall provide API for removing of snapshots with an argument that selects a dedicated snapshot slot by snapshot index.
   The function shall return an error, when referenced slot is free.

.. comp_req:: Explicit snapshot operations
   :id: comp_req__kvs__explicit_snapshot_operations
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :status: valid
   :version: 1
   :derived_from: feat_req__persistency__snapshot_create[version==1], feat_req__persistency__snapshot_restore[version==1], feat_req__persistency__snapshot_remove[version==1]
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: valid

   The component shall perform snapshot creation, restoration, and deletion only when explicitly triggered by the user through the corresponding APIs.

.. comp_req:: Snapshot slot indexing
   :id: comp_req__kvs__snapshot_id_api
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__snapshot_create[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: valid

   The component shall identify snapshot slots by a zero-based index, where the first slot has index 0, the second slot has index 1, and so on.

.. comp_req:: Snapshot data source
   :id: comp_req__kvs__snapshot_source
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :status: valid
   :version: 1
   :derived_from: feat_req__persistency__snapshot_create[version==1]
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: valid

   The component shall use the live values that were set by the user, regardless of whether the values were flushed to disk.

.. comp_req:: Snapshot slot free query API
   :id: comp_req__kvs__snapshot_slot_free_api
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :status: valid
   :version: 1
   :derived_from: feat_req__persistency__snapshot_create[version==1], feat_req__persistency__snapshot_remove[version==1], feat_req__persistency__cfg[version==1]
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: valid

   The component shall provide an API to check whether a slot identified by a snapshot index is free or occupied.

.. comp_req:: Snapshot Creation
   :id: comp_req__kvs__snapshot_creation
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__snapshot_create[version==1]
   :status: invalid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall create a snapshot each time data is stored.

   .. note::

      This requirement is invalid after change of snapshot handling (see <ticket-id>)

.. comp_req:: Snapshot Maximum Number
   :id: comp_req__kvs__snapshot_max_num
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__cfg[version==1]
   :status: invalid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall maintain a configurable maximum number of snapshots.

   .. note::

      This requirement is invalid after change of snapshot handling (see <ticket-id>)

.. comp_req:: Snapshot IDs
   :id: comp_req__kvs__snapshot_id
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__snapshot_create[version==1]
   :status: invalid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall assign the ID 1 to the newest snapshot and shall increment the IDs of older snapshots accordingly.

     .. note::

      This requirement is invalid after change of snapshot handling (see <ticket-id>)

.. comp_req:: Snapshot Rotation
   :id: comp_req__kvs__snapshot_rotate
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__snapshot_remove[version==1],feat_req__persistency__snapshot_restore[version==1]
   :status: invalid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall rotate and delete the oldest snapshot when the maximum number is reached.

   .. note::

      This requirement is invalid after change of snapshot handling (see <ticket-id>)

.. comp_req:: Snapshot Restore
   :id: comp_req__kvs__snapshot_restore
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__snapshot_restore[version==1]
   :status: invalid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall allow restoration of a snapshot by its ID.

   .. note::

      This requirement is invalid after change of snapshot handling (see <ticket-id>)

.. comp_req:: Snapshot Deletion
   :id: comp_req__kvs__snapshot_delete
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__snapshot_remove[version==1]
   :status: invalid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall allow deletion of individual snapshots.

   .. note::

      This requirement is invalid after change of snapshot handling (see <ticket-id>)

.. comp_req:: Engineering Mode
   :id: comp_req__kvs__eng_mode
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__dev_mode[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall provide an engineering mode that can be enabled during
   build time to display debugging and internal information.

.. comp_req:: Field Mode
   :id: comp_req__kvs__field_mode
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__prod_mode[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall provide a field mode that can be enabled during build
   time to restrict access as much as possible.

.. comp_req:: Async API
   :id: comp_req__kvs__async_api
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__async_api[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall provide an asynchronous API in addition to the standard API.

.. comp_req:: Permission Control
   :id: comp_req__kvs__permission_control
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__access_control[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall rely on the underlying filesystem for access and
   permission management and shall not implement its own access or permission
   controls.

.. comp_req:: Permission Error Handling
   :id: comp_req__kvs__permission_err_hndl
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__access_control[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall report any access or permission errors encountered at
   the filesystem level to the application.

.. comp_req:: Callback Support
   :id: comp_req__kvs__callback_support
   :reqtype: Functional
   :security: NO
   :safety: ASIL_B
   :derived_from: feat_req__persistency__async_api[version==1],feat_req__persistency__async_completion[version==1]
   :status: valid
   :version: 1
   :satisfied_by: comp__persistency_kvs[version==1]
   :tags: inspected

   The component shall provide an API for registering callbacks that are triggered by data change events.


Assumption of Use Requirements
------------------------------

none

Environmental Requirements
--------------------------

none


.. needextend:: c.this_doc() and docname is not None
   :+tags: kvs
