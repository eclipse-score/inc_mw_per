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
filegroup(
    name = "kvs_srcs",
    srcs = glob([
        "src/*.cpp",
        "inc/**/*.hpp",
    ]),
    visibility = [
        "//src/cpp/tests:__pkg__",
    ],
)

cc_library(
    name = "kvs_cpp",
    srcs = glob(["src/*.cpp"]),
    hdrs = glob([
        "inc/**/*.hpp",
    ]),
    includes = ["inc"],
    visibility = ["//:__pkg__"],
    deps = [
        "@score-baselibs//score/filesystem:filesystem",
        "@score-baselibs//score/json:interface",
        "@score-baselibs//score/json:json_parser",
        "@score-baselibs//score/result:result",
    ],
)
