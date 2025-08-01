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
    name = "error",
    srcs = [
        "error.cpp",
    ],
    hdrs = [
        "error.hpp",
    ],
    visibility = [
        "//src/cpp/src:__pkg__",
        "//src/cpp/tests:__pkg__",
    ],
    deps = [
        "@score-baselibs//score/result:result",
    ],
)

cc_library(
    name = "kvs_helper",
    srcs = [
        "kvs_helper.cpp",
    ],
    hdrs = [
        "kvs_helper.hpp",
    ],
    visibility = [
        "//src/cpp/src:__pkg__",
        "//src/cpp/tests:__pkg__",
    ],
    deps = [
        ":error",
        "//src/cpp/src:kvsvalue",
        "@score-baselibs//score/json",
    ],
)
