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
load("@crates//:defs.bzl", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "rust_kvs",
    srcs = glob(["src/**/*.rs"]),
    visibility = ["//visibility:public"],
    deps = all_crate_deps(
        normal = True,
    ),
)

rust_test(
    name = "tests",
    crate = "rust_kvs",
    tags = [
        "unit_tests",
        "ut",
    ],
    deps = all_crate_deps(
        normal = True,
        normal_dev = True,
    ),
)

__cit_deps = [
    ":rust_kvs",
] + all_crate_deps(
    normal = True,
    normal_dev = True,
)

__cit_tags = [
    "cit",
    "component_tests",
]

rust_test(
    name = "cit_default_values",
    srcs = ["tests/cit_default_values.rs"],
    tags = __cit_tags,
    deps = __cit_deps,
)

rust_test(
    name = "cit_multiple_kvs",
    srcs = ["tests/cit_multiple_kvs.rs"],
    tags = __cit_tags,
    deps = __cit_deps,
)

rust_test(
    name = "cit_persistency",
    srcs = [
        "tests/cit_persistency.rs",
        "tests/common/mod.rs",
    ],
    tags = __cit_tags,
    deps = __cit_deps,
)

rust_test(
    name = "cit_snapshots",
    srcs = ["tests/cit_snapshots.rs"],
    tags = __cit_tags,
    deps = __cit_deps,
)

rust_test(
    name = "cit_supported_datatypes",
    srcs = [
        "tests/cit_supported_datatypes.rs",
        "tests/common/mod.rs",
    ],
    tags = __cit_tags,
    deps = __cit_deps,
)

test_suite(
    name = "cit",
    tests = [
        ":cit_default_values",
        ":cit_multiple_kvs",
        ":cit_persistency",
        ":cit_snapshots",
        ":cit_supported_datatypes",
    ],
)
