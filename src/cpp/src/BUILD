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
        "//:__pkg__",
        "//src/cpp/src/internal:__pkg__",
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
        "//src/cpp/src/internal:kvs_helper",
    ],
    includes = ["."],
    visibility = ["//:__pkg__"],
    deps = [
        ":kvsvalue",
        "//src/cpp/src/internal:error",
        "@score-baselibs//score/filesystem:filesystem",
        "@score-baselibs//score/json",
        "@score-baselibs//score/result:result",
    ],
)
