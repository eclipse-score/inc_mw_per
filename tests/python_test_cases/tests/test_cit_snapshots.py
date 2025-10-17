from pathlib import Path
from typing import Any, Generator
import pytest
from .common import CommonScenario, ResultCode, temp_dir_common
from testing_utils import ScenarioResult, LogContainer

pytestmark = pytest.mark.parametrize("version", ["rust"], scope="class")


class MaxSnapshotsScenario(CommonScenario):
    """
    Common base implementation for snapshots tests.
    """

    @pytest.fixture(scope="class")
    def temp_dir(
        self,
        tmp_path_factory: pytest.TempPathFactory,
        version: str,
        snapshot_max_count: int,
    ) -> Generator[Path, None, None]:
        """
        Create temporary directory and remove it after test.
        """
        yield from temp_dir_common(
            tmp_path_factory, self.__class__.__name__, version, str(snapshot_max_count)
        )


@pytest.mark.PartiallyVerifies(["comp_req__persistency__snapshot_creation"])
@pytest.mark.FullyVerifies([])
@pytest.mark.Description(
    "Verifies that a snapshot is only created after the first flush, and not before."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("requirements-based")
@pytest.mark.parametrize("snapshot_max_count", [0, 1, 3, 10], scope="class")
class TestSnapshotCountFirstFlush(MaxSnapshotsScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.snapshots.count"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path, snapshot_max_count: int) -> dict[str, Any]:
        return {
            "kvs_parameters": {
                "instance_id": 1,
                "dir": str(temp_dir),
                "snapshot_max_count": snapshot_max_count,
            },
            "count": 1,
        }

    def test_ok(
        self,
        test_config: dict[str, Any],
        results: ScenarioResult,
        logs_info_level: LogContainer,
        snapshot_max_count: int,
    ):
        assert results.return_code == ResultCode.SUCCESS

        count = test_config["count"]
        logs = logs_info_level.get_logs("snapshot_count")
        assert len(logs) == count + 1
        for i in range(count):
            expected = min(i, snapshot_max_count)
            assert logs[i].snapshot_count == expected

        assert logs[-1].snapshot_count == min(count, snapshot_max_count)


@pytest.mark.PartiallyVerifies(["comp_req__persistency__snapshot_creation"])
@pytest.mark.FullyVerifies([])
@pytest.mark.Description(
    "Checks that the snapshot count increases with each flush, up to the maximum allowed count."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("requirements-based")
class TestSnapshotCountFull(TestSnapshotCountFirstFlush):
    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path, snapshot_max_count: int) -> dict[str, Any]:
        return {
            "kvs_parameters": {
                "instance_id": 1,
                "dir": str(temp_dir),
                "snapshot_max_count": snapshot_max_count,
            },
            "count": snapshot_max_count + 1,
        }


@pytest.mark.PartiallyVerifies(["comp_req__persistency__snapshot_max_num"])
@pytest.mark.FullyVerifies([])
@pytest.mark.Description(
    "Verifies that the maximum number of snapshots is a constant value."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("inspection")
@pytest.mark.parametrize("snapshot_max_count", [0, 1, 3, 10], scope="class")
class TestSnapshotMaxCount(MaxSnapshotsScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.snapshots.max_count"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path, snapshot_max_count: int) -> dict[str, Any]:
        return {
            "kvs_parameters": {
                "instance_id": 1,
                "dir": str(temp_dir),
                "snapshot_max_count": snapshot_max_count,
            }
        }

    def test_ok(
        self,
        results: ScenarioResult,
        logs_info_level: LogContainer,
        snapshot_max_count: int,
    ):
        assert results.return_code == ResultCode.SUCCESS
        assert (
            logs_info_level.find_log("max_count", value=snapshot_max_count) is not None
        )


@pytest.mark.PartiallyVerifies(
    [
        "comp_req__persistency__snapshot_creation",
        "comp_req__persistency__snapshot_rotate",
    ]
)
@pytest.mark.FullyVerifies(["comp_req__persistency__snapshot_restore"])
@pytest.mark.Description(
    "Verifies restoring to a previous snapshot returns the expected value."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("control-flow-analysis")
@pytest.mark.parametrize("snapshot_max_count", [3, 10], scope="class")
class TestSnapshotRestorePrevious(MaxSnapshotsScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.snapshots.restore"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path, snapshot_max_count: int) -> dict[str, Any]:
        return {
            "kvs_parameters": {
                "instance_id": 1,
                "dir": str(temp_dir),
                "snapshot_max_count": snapshot_max_count,
            },
            "snapshot_id": 1,
            "count": 3,
        }

    def test_ok(
        self,
        results: ScenarioResult,
        logs_info_level: LogContainer,
    ):
        assert results.return_code == ResultCode.SUCCESS

        result_log = logs_info_level.find_log("result")
        assert result_log is not None
        assert result_log.result == "Ok(())"

        value_log = logs_info_level.find_log("value")
        assert value_log is not None
        assert value_log.value == 1


@pytest.mark.PartiallyVerifies(["comp_req__persistency__snapshot_creation"])
@pytest.mark.FullyVerifies([])
@pytest.mark.Description(
    "Checks that restoring the current snapshot ID fails with InvalidSnapshotId error."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("fault-injection")
class TestSnapshotRestoreCurrent(CommonScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.snapshots.restore"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path) -> dict[str, Any]:
        return {
            "kvs_parameters": {"instance_id": 1, "dir": str(temp_dir)},
            "snapshot_id": 0,
            "count": 3,
        }

    def capture_stderr(self) -> bool:
        return True

    def test_error(
        self,
        results: ScenarioResult,
        logs_info_level: LogContainer,
    ):
        assert results.return_code == ResultCode.SUCCESS

        assert results.stderr is not None
        assert "error: tried to restore current KVS as snapshot" in results.stderr

        result_log = logs_info_level.find_log("result")
        assert result_log is not None
        assert result_log.result == "Err(InvalidSnapshotId)"


@pytest.mark.PartiallyVerifies(
    [
        "comp_req__persistency__snapshot_creation",
        "comp_req__persistency__snapshot_restore",
    ]
)
@pytest.mark.FullyVerifies([])
@pytest.mark.Description(
    "Checks that restoring a non-existing snapshot fails with InvalidSnapshotId error."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("fault-injection")
class TestSnapshotRestoreNonexistent(CommonScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.snapshots.restore"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path) -> dict[str, Any]:
        return {
            "kvs_parameters": {"instance_id": 1, "dir": str(temp_dir)},
            "snapshot_id": 2,
            "count": 1,
        }

    def capture_stderr(self) -> bool:
        return True

    def test_error(
        self,
        results: ScenarioResult,
        logs_info_level: LogContainer,
    ):
        assert results.return_code == ResultCode.SUCCESS

        assert results.stderr is not None
        assert "error: tried to restore a non-existing snapshot" in results.stderr

        result_log = logs_info_level.find_log("result")
        assert result_log is not None
        assert result_log.result == "Err(InvalidSnapshotId)"


@pytest.mark.PartiallyVerifies(["comp_req__persistency__snapshot_creation"])
@pytest.mark.FullyVerifies([])
@pytest.mark.Description(
    "Verifies that the KVS and hash filenames for an existing snapshot is generated correctly."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("interface-test")
class TestSnapshotPathsExist(CommonScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.snapshots.paths"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path) -> dict[str, Any]:
        return {
            "kvs_parameters": {"instance_id": 1, "dir": str(temp_dir)},
            "snapshot_id": 1,
            "count": 3,
        }

    def test_ok(
        self,
        temp_dir: Path,
        results: ScenarioResult,
        logs_info_level: LogContainer,
    ):
        assert results.return_code == ResultCode.SUCCESS

        paths_log = logs_info_level.find_log("kvs_path")
        assert paths_log is not None
        assert paths_log.kvs_path == f'"{temp_dir}/kvs_1_1.json"'
        assert paths_log.kvs_path_exists
        assert paths_log.hash_path == f'"{temp_dir}/kvs_1_1.hash"'
        assert paths_log.hash_path_exists


@pytest.mark.PartiallyVerifies(["comp_req__persistency__snapshot_creation"])
@pytest.mark.FullyVerifies([])
@pytest.mark.Description(
    "Checks that requesting the KVS and hash filenames for a non-existing snapshot returns FileNotFound error."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("fault-injection")
class TestSnapshotPathsNonexistent(CommonScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.snapshots.paths"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path) -> dict[str, Any]:
        return {
            "kvs_parameters": {"instance_id": 1, "dir": str(temp_dir)},
            "snapshot_id": 2,
            "count": 1,
        }

    def test_error(
        self,
        temp_dir: Path,
        results: ScenarioResult,
        logs_info_level: LogContainer,
    ):
        assert results.return_code == ResultCode.SUCCESS

        paths_log = logs_info_level.find_log("kvs_path")
        assert paths_log is not None
        assert paths_log.kvs_path == f'"{temp_dir}/kvs_1_2.json"'
        assert not paths_log.kvs_path_exists
        assert paths_log.hash_path == f'"{temp_dir}/kvs_1_2.hash"'
        assert not paths_log.hash_path_exists
