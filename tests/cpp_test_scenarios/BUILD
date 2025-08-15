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

cc_binary(
    name = "cpp_test_scenarios",
    srcs = [
        "src/main.cpp",
        "src/test_basic.cpp",
        "src/test_basic.hpp",
    ],
    copts = [
        "-g",
    ],
    visibility = ["//visibility:private"],
    deps = [
        "//kvs_cpp",
        "@testing-utils//test_scenarios_cpp:test_scenarios_cpp",
    ],
)
