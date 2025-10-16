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
from pathlib import Path
from typing import Any

import pytest
from testing_utils import LogContainer, ScenarioResult

from .common import CommonScenario, ResultCode

pytestmark = pytest.mark.parametrize("version", ["rust"], scope="class")


@pytest.mark.PartiallyVerifies([])
@pytest.mark.FullyVerifies(["comp_req__persistency__persist_data_store_com"])
@pytest.mark.Description(
    "Verifies that disabling flush on exit but manually flushing ensures data is persisted correctly."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("requirements-based")
class TestExplicitFlush(CommonScenario):
    NUM_VALUES = 5

    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.persistency.explicit_flush"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path) -> dict[str, Any]:
        return {
            "kvs_parameters": {
                "instance_id": 2,
                "dir": str(temp_dir),
            }
        }

    def test_data_stored(self, results: ScenarioResult, logs_info_level: LogContainer):
        assert results.return_code == ResultCode.SUCCESS

        for i in range(self.NUM_VALUES):
            log = logs_info_level.find_log("key", value=f"test_number_{i}")
            assert log is not None
            assert log.value == f"Ok(F64({12.3 * i}))"
