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

# The filegroup is used to collect all source files for the tests.

cc_library(
    name = "kvsvalue",
    srcs = [
        "kvsvalue.cpp",
    ],
    hdrs = ["kvsvalue.hpp"],
    includes = ["."],
    visibility = [
        "//kvs_cpp/internal:__pkg__",
    ],
)

cc_library(
    name = "kvs_cpp",
    srcs = [
        "kvs.cpp",
        "kvsbuilder.cpp",
    ],
    hdrs = [
        "kvs.hpp",
        "kvsbuilder.hpp",
    ],
    implementation_deps = [
        "//kvs_cpp/internal:kvs_helper",
    ],
    includes = ["."],
    visibility = [
        "//:__pkg__",
        "//tests/cpp_test_scenarios:__pkg__",
    ],
    deps = [
        ":kvsvalue",
        "//kvs_cpp/internal:error",
        "@score-baselibs//score/filesystem:filesystem",
        "@score-baselibs//score/json",
        "@score-baselibs//score/result:result",
    ],
)

# tests
cc_test(
    name = "test_kvs_cpp",
    size = "small",
    srcs = [
        "tests/test_kvs.cpp",
        "tests/test_kvs_builder.cpp",
        "tests/test_kvs_error.cpp",
        "tests/test_kvs_general.cpp",
        "tests/test_kvs_general.hpp",
        "tests/test_kvs_helper.cpp",
    ],
    visibility = ["//:__pkg__"],
    deps = [
        ":kvs_cpp",
        "//kvs_cpp/internal:kvs_helper",
        "@googletest//:gtest_main",
        "@score-baselibs//score/filesystem:filesystem",
        "@score-baselibs//score/filesystem:mock",
        "@score-baselibs//score/json:mock",
        "@score-baselibs//score/result:result",
    ],
)

cc_test(
    name = "bm_kvs_cpp",
    size = "small",
    srcs = [
        "tests/bm_kvs.cpp",
    ],
    visibility = ["//:__pkg__"],
    deps = [
        ":kvs_cpp",
        "//kvs_cpp/internal:kvs_helper",
        "@google_benchmark//:benchmark",
        "@score-baselibs//score/filesystem:filesystem",
        "@score-baselibs//score/json",
        "@score-baselibs//score/result:result",
    ],
)
