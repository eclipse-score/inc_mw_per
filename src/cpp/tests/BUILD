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

cc_test(
    name = "test_kvs_cpp",
    size = "small",
    srcs = [
        "test_kvs.cpp",
        "test_kvs_builder.cpp",
        "test_kvs_error.cpp",
        "test_kvs_general.cpp",
        "test_kvs_general.hpp",
        "test_kvs_helper.cpp",
    ],
    visibility = ["//:__pkg__"],
    deps = [
        "//:kvs_cpp",
        "//src/cpp/src/internal:kvs_helper",
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
        "bm_kvs.cpp",
    ],
    visibility = ["//:__pkg__"],
    deps = [
        "//:kvs_cpp",
        "//src/cpp/src/internal:kvs_helper",
        "@google_benchmark//:benchmark",
        "@score-baselibs//score/filesystem:filesystem",
        "@score-baselibs//score/json",
        "@score-baselibs//score/result:result",
    ],
)
