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
load("@pip_score_venv_test//:requirements.bzl", "all_requirements")
load("@rules_python//python:pip.bzl", "compile_pip_requirements")
load("@score_python_basics//:defs.bzl", "score_py_pytest", "score_virtualenv")

# Additional requirements for the tests
compile_pip_requirements(
    name = "requirements",
    # requirements_txt = "requirements.txt",
    src = "//tests/python_test_cases:pyproject.toml",
    tags = [
        "manual",
    ],
)

score_virtualenv(
    name = "python_tc_venv",
    reqs = all_requirements,
    venv_name = ".python_tc_venv",
)

# Tests targets
score_py_pytest(
    name = "cit_cpp",
    srcs = glob(["tests/**/*.py"]),
    args = [
        "-m cpp",
        "--cpp-target-path",
        "$(rootpath //tests/cpp_test_scenarios)",
    ],
    data = [
        ":python_tc_venv",
        "//tests/cpp_test_scenarios",
        "//tests/rust_test_scenarios",
    ],
    env = {
        "RUST_BACKTRACE": "1",
    },
    pytest_ini = "//tests/python_test_cases:pytest.ini",
    tags = [
        "manual",
    ],
    deps = all_requirements,
)

score_py_pytest(
    name = "cit_rust",
    srcs = glob(["tests/**/*.py"]),
    args = [
        "-m rust",
        "--rust-target-path",
        "$(rootpath //tests/rust_test_scenarios)",
    ],
    data = [
        ":python_tc_venv",
        "//tests/cpp_test_scenarios",
        "//tests/rust_test_scenarios",
    ],
    env = {
        "RUST_BACKTRACE": "1",
    },
    pytest_ini = "//tests/python_test_cases:pytest.ini",
    tags = [
        "manual",
    ],
    deps = all_requirements,
)

test_suite(
    name = "cit",
    tests = [
        ":cit_cpp",
        ":cit_rust",
    ],
    visibility = ["//visibility:public"],
)
