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

load("@score_docs_as_code//:docs.bzl", "docs")
load("@score_tooling//:defs.bzl", "cli_helper", "copyright_checker", "dash_license_checker", "setup_starpls", "use_format_targets")
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
    config = "@score_tooling//cr_checker/resources:config",
    template = "@score_tooling//cr_checker/resources:templates",
    visibility = ["//visibility:public"],
)

# Needed for Dash tool to check python dependency licenses.
# This is a workaround to filter out local packages from the Cargo.lock file.
# The tool is intended for third-party content.
genrule(
    name = "filtered_cargo_lock",
    srcs = ["Cargo.lock"],
    outs = ["Cargo.lock.filtered"],
    cmd = """
    awk '
    BEGIN { skip = 0; data = "" }
    /^\\[\\[package\\]\\]/ {
        if (data != "" && !skip) print data;
        skip = 1;
        data = $$0;
        next;
    }
    data != "" { data = data "\\n" $$0 }
    # any package that has a "source = " line will not be skipped.
    /^source = / { skip = 0 }
    END { if (data != "" && !skip) print data }
    ' $(location Cargo.lock) > $@
    """,
)

dash_license_checker(
    src = ":filtered_cargo_lock",
    file_type = "",  # let it auto-detect based on project_config
    project_config = PROJECT_CONFIG,
    visibility = ["//visibility:public"],
)

cli_helper(
    name = "cli-help",
    visibility = ["//visibility:public"],
)

# Add target for formatting checks
use_format_targets()

alias(
    name = "kvs_cpp",
    actual = "//src/cpp/src:kvs_cpp",
    tags = ["cli_help=Build KVS CPP [build]"],
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
