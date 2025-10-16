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
"""
Smoke test for Rust-C++ tests.
"""

from typing import Any

import pytest
from testing_utils import LogContainer, ScenarioResult

from .common import CommonScenario, ResultCode


@pytest.mark.parametrize("version", ["cpp", "rust"], scope="class")
class TestBasic(CommonScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self, *_, **__) -> str:
        return "basic.basic"

    @pytest.fixture(scope="class")
    def test_config(self, *_, **__) -> dict[str, Any]:
        return {"kvs_parameters": {"instance_id": 2}}

    def test_returncode_ok(self, results: ScenarioResult):
        assert results.return_code == ResultCode.SUCCESS

    def test_trace_ok(self, logs_target: LogContainer):
        lc = logs_target.get_logs("example_key", value="example_value")
        assert len(lc) == 1
