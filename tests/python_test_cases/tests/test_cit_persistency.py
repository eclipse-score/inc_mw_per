from pathlib import Path
from typing import Any, Generator
import pytest
from common import CommonScenario, ResultCode, temp_dir_common
from testing_utils import ScenarioResult, LogContainer

pytestmark = pytest.mark.parametrize("version", ["rust"], scope="class")


class PersistencyScenario(CommonScenario):
    """
    Common base implementation for persistency tests.
    """

    def instance_id(self) -> int:
        return 2

    @pytest.fixture(scope="class")
    def temp_dir(
        self, tmp_path_factory: pytest.TempPathFactory, version: str
    ) -> Generator[Path, None, None]:
        yield from temp_dir_common(tmp_path_factory, self.__class__.__name__, version)


@pytest.mark.PartiallyVerifies([])
@pytest.mark.FullyVerifies(["comp_req__persistency__persist_data_store_com"])
@pytest.mark.Description(
    "Verifies that disabling flush on exit but manually flushing ensures data is persisted correctly."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("requirements-based")
class TestExplicitFlush(PersistencyScenario):
    NUM_VALUES = 5

    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.persistency.explicit_flush"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path) -> dict[str, Any]:
        return {
            "kvs_parameters": {
                "instance_id": self.instance_id(),
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
class TestFlushOnExitEnabled(PersistencyScenario):
    NUM_VALUES = 5

    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.persistency.flush_on_exit"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path) -> dict[str, Any]:
        return {
            "kvs_parameters": {
                "instance_id": self.instance_id(),
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
class TestFlushOnExitDisabled(PersistencyScenario):
    NUM_VALUES = 5

    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.persistency.flush_on_exit"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path) -> dict[str, Any]:
        return {
            "kvs_parameters": {
                "instance_id": self.instance_id(),
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
