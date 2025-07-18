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
use crate::kvs_value::KvsMap;

use std::path::PathBuf;

/// KVS backend interface.
pub trait KvsBackend {
    /// Load KvsMap from given file.
    fn load_kvs(
        source_path: PathBuf,
        verify_hash: bool,
        hash_source: Option<PathBuf>,
    ) -> Result<KvsMap, ErrorCode>;

    /// Store KvsMap at given file path.
    fn save_kvs(kvs: &KvsMap, destination_path: PathBuf, add_hash: bool) -> Result<(), ErrorCode>;
}
