"""
Smoke test for Rust-C++ tests.
"""

from typing import Any
import pytest
from testing_utils import LogContainer, ScenarioResult
from common import CommonScenario, ResultCode


@pytest.mark.parametrize("version", ["cpp", "rust"], scope="class")
class TestBasic(CommonScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self, *_, **__) -> str:
        return "basic.basic"

    @pytest.fixture(scope="class")
    def test_config(self, *_, **__) -> dict[str, Any]:
        return {"kvs_parameters": {"instance_id": 2, "flush_on_exit": False}}

    def test_returncode_ok(self, results: ScenarioResult):
        assert results.return_code == ResultCode.SUCCESS

    def test_trace_ok(self, logs_target: LogContainer):
        lc = logs_target.get_logs("example_key", value="example_value")
        assert len(lc) == 1
