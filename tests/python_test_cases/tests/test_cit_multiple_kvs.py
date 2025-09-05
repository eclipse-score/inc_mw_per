from pathlib import Path
from typing import Any
import pytest

from common import CommonScenario, ResultCode
from testing_utils import ScenarioResult, LogContainer


pytestmark = pytest.mark.parametrize("version", ["rust"], scope="class")


@pytest.mark.PartiallyVerifies(
    ["comp_req__persistency__multi_instance", "comp_req__persistency__concurrency"]
)
@pytest.mark.FullyVerifies([""])
@pytest.mark.Description(
    "Verifies that multiple KVS instances with different IDs store and retrieve independent values without interference."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("requirements-based")
class TestMultipleInstanceIds(CommonScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.multiple_kvs.multiple_instance_ids"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path) -> dict[str, Any]:
        return {
            "kvs_parameters_1": {
                "kvs_parameters": {"instance_id": 1, "dir": str(temp_dir)}
            },
            "kvs_parameters_2": {
                "kvs_parameters": {"instance_id": 2, "dir": str(temp_dir)}
            },
        }

    def test_ok(self, results: ScenarioResult, logs_info_level: LogContainer):
        assert results.return_code == ResultCode.SUCCESS

        key = "number"
        log1 = logs_info_level.find_log("instance", value="kvs1")
        assert log1 is not None
        assert log1.key == key
        assert log1.value == 111.1

        log2 = logs_info_level.find_log("instance", value="kvs2")
        assert log2 is not None
        assert log2.key == key
        assert log2.value == 222.2


@pytest.mark.PartiallyVerifies(
    ["comp_req__persistency__multi_instance", "comp_req__persistency__concurrency"]
)
@pytest.mark.FullyVerifies([""])
@pytest.mark.Description(
    "Checks that multiple KVS instances with the same ID and key maintain consistent values across instances."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("requirements-based")
class TestSameInstanceIdSameValue(CommonScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.multiple_kvs.same_instance_id_same_value"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path) -> dict[str, Any]:
        return {"kvs_parameters": {"instance_id": 1, "dir": str(temp_dir)}}

    def test_ok(self, results: ScenarioResult, logs_info_level: LogContainer):
        assert results.return_code == ResultCode.SUCCESS

        key = "number"
        log1 = logs_info_level.find_log("instance", value="kvs1")
        assert log1 is not None
        assert log1.key == key
        assert log1.value == 111.1

        log2 = logs_info_level.find_log("instance", value="kvs2")
        assert log2 is not None
        assert log2.key == key
        assert log2.value == 111.1

        assert log1.value == log2.value


@pytest.mark.PartiallyVerifies(
    ["comp_req__persistency__multi_instance", "comp_req__persistency__concurrency"]
)
@pytest.mark.FullyVerifies([""])
@pytest.mark.Description(
    "Verifies that changes in one KVS instance with a shared ID and key are reflected in another instance, demonstrating interference."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("requirements-based")
class TestSameInstanceIdDifferentValue(CommonScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.multiple_kvs.same_instance_id_diff_value"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path) -> dict[str, Any]:
        return {"kvs_parameters": {"instance_id": 1, "dir": str(temp_dir)}}

    def test_ok(self, results: ScenarioResult, logs_info_level: LogContainer):
        assert results.return_code == ResultCode.SUCCESS

        # Assertions are same as in 'TestSameInstanceIdSameValue'.
        # Test scenario behavior differs underneath.
        key = "number"
        log1 = logs_info_level.find_log("instance", value="kvs1")
        assert log1 is not None
        assert log1.key == key
        assert log1.value == 111.1

        log2 = logs_info_level.find_log("instance", value="kvs2")
        assert log2 is not None
        assert log2.key == key
        assert log2.value == 111.1

        assert log1.value == log2.value
