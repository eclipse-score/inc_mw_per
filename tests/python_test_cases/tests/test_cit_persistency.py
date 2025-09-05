from pathlib import Path
from typing import Any
import pytest
from common import CommonScenario, ResultCode
from testing_utils import ScenarioResult, LogContainer

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
                "flush_on_exit": False,
            }
        }

    def test_data_stored(self, results: ScenarioResult, logs_info_level: LogContainer):
        assert results.return_code == ResultCode.SUCCESS

        for i in range(self.NUM_VALUES):
            log = logs_info_level.find_log("key", value=f"test_number_{i}")
            assert log is not None
            assert log.value == f"Ok(F64({12.3 * i}))"


@pytest.mark.PartiallyVerifies([])
@pytest.mark.FullyVerifies(["comp_req__persistency__persist_data_store_com"])
@pytest.mark.Description(
    "Verifies that data is automatically flushed and persisted when the KVS instance is dropped, with flush on exit enabled."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("requirements-based")
class TestFlushOnExitEnabled(CommonScenario):
    NUM_VALUES = 5

    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.persistency.flush_on_exit"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path) -> dict[str, Any]:
        return {
            "kvs_parameters": {
                "instance_id": 2,
                "dir": str(temp_dir),
                "flush_on_exit": True,
            }
        }

    def test_data_stored(self, results: ScenarioResult, logs_info_level: LogContainer):
        assert results.return_code == ResultCode.SUCCESS

        for i in range(self.NUM_VALUES):
            log = logs_info_level.find_log("key", value=f"test_number_{i}")
            assert log is not None
            assert log.value == f"Ok(F64({12.3 * i}))"


@pytest.mark.PartiallyVerifies([])
@pytest.mark.FullyVerifies(["comp_req__persistency__persist_data_store_com"])
@pytest.mark.Description(
    "Checks that disabling flush on exit causes data to be dropped and not persisted after the KVS instance is dropped."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("requirements-based")
class TestFlushOnExitDisabled(CommonScenario):
    NUM_VALUES = 5

    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.persistency.flush_on_exit"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path) -> dict[str, Any]:
        return {
            "kvs_parameters": {
                "instance_id": 2,
                "dir": str(temp_dir),
                "flush_on_exit": False,
            }
        }

    def test_data_dropped(self, results: ScenarioResult, logs_info_level: LogContainer):
        assert results.return_code == ResultCode.SUCCESS

        for i in range(self.NUM_VALUES):
            log = logs_info_level.find_log("key", value=f"test_number_{i}")
            assert log is not None
            assert log.value == "Err(KeyNotFound)"
