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

// This file exposes common logging interface for different front-ends.

// Compile error if both `logging` and `score-log` features are enabled.
#[cfg(all(feature = "logging", feature = "score-log"))]
compile_error!("`logging` and `score-log` cannot be enabled at the same time.");

#[allow(unused_imports)]
#[cfg(feature = "logging")]
pub use log::{debug, error, info, trace, warn};

#[allow(unused_imports)]
#[cfg(feature = "score-log")]
pub use mw_log::{debug, error, info, trace, warn};
