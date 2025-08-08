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
load("@rules_rust//rust:defs.bzl", "rust_binary")
load("@score_persistency_crates//:defs.bzl", "all_crate_deps")

rust_binary(
    name = "kvs_tool",
    srcs = [
        "src/kvs_tool.rs",
    ],
    crate_name = "rust_kvs_tool",
    visibility = ["//visibility:public"],
    deps = all_crate_deps(
        normal = True,
    ) + [
        "//src/rust/rust_kvs",
    ],
)
