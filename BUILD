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

load("@score_cr_checker//:cr_checker.bzl", "copyright_checker")
load("@score_dash_license_checker//:dash.bzl", "dash_license_checker")
load("@score_docs_as_code//:docs.bzl", "docs")
load("@score_format_checker//:macros.bzl", "use_format_targets")
load("@score_starpls_lsp//:starpls.bzl", "setup_starpls")
load("//:project_config.bzl", "PROJECT_CONFIG")

# Creates all documentation targets:
# - `:docs` for building documentation at build-time
docs(
    data = [
        "@score_platform//:needs_json",
        "@score_process//:needs_json",
    ],
    source_dir = "docs",
)

setup_starpls(
    name = "starpls_server",
    visibility = ["//visibility:public"],
)

copyright_checker(
    name = "copyright",
    srcs = [
        "src",
        "//:BUILD",
        "//:MODULE.bazel",
    ],
    config = "@score_cr_checker//resources:config",
    template = "@score_cr_checker//resources:templates",
    visibility = ["//visibility:public"],
)

dash_license_checker(
    src = ":cargo_lock",
    file_type = "",  # let it auto-detect based on project_config
    project_config = PROJECT_CONFIG,
    visibility = ["//visibility:public"],
)

# Add target for formatting checks
use_format_targets()

alias(
    name = "kvs_cpp",
    actual = "//src/cpp/src:kvs_cpp",
    visibility = ["//visibility:public"],
)

test_suite(
    name = "test_kvs_cpp",
    tests = ["//src/cpp/tests:test_kvs_cpp"],
    visibility = ["//visibility:public"],
)

test_suite(
    name = "bm_kvs_cpp",
    tests = ["//src/cpp/tests:bm_kvs_cpp"],
    visibility = ["//visibility:public"],
)
