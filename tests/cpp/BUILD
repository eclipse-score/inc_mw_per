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

cc_library(
    name = "json_stubs",
    hdrs = glob(["stubs/score/json/**/*.h"]),
    includes = ["stubs"],
    visibility = [
        "//tests/cpp:__pkg__",
    ],
)

cc_test(
    name = "test_kvs_cpp",
    srcs = [
        "test_kvs.cpp",
        "//src/cpp:kvs_srcs",
    ],
    copts = [
        "-Isrc/cpp/inc",
    ],
    visibility = ["//visibility:public"],
    deps = [
        ":json_stubs",
        "@googletest//:gtest_main",
        "@score-baselibs//score/filesystem:filesystem",
        "@score-baselibs//score/memory:string_comparison_adaptor",
        "@score-baselibs//score/result:result",
    ],
)
