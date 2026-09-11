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
from typing import Any, Generator

import pytest
from common import CommonScenario, ResultCode, temp_dir_common
from test_properties import add_test_properties
from testing_utils import LogContainer, ScenarioResult

pytestmark = pytest.mark.parametrize("version", ["rust", "cpp"], scope="class")


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
        yield from temp_dir_common(tmp_path_factory, self.__class__.__name__, version, str(snapshot_max_count))


@add_test_properties(
    partially_verifies=["comp_req__kvs__snapshot_creation"],
    test_type="requirements-based",
    derivation_technique="requirements-analysis",
)
@pytest.mark.parametrize("snapshot_max_count", [0, 1, 3, 10], scope="class")
class TestSnapshotCountFirstFlush(MaxSnapshotsScenario):
    """Verifies that a snapshot is only created after the first flush, and not before."""

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


@add_test_properties(
    partially_verifies=["comp_req__kvs__snapshot_creation"],
    test_type="requirements-based",
    derivation_technique="requirements-analysis",
)
class TestSnapshotCountFull(TestSnapshotCountFirstFlush):
    """ "Checks that the snapshot count increases with each flush, up to the maximum allowed count."""

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


@add_test_properties(
    partially_verifies=["comp_req__kvs__snapshot_max_num"],
    test_type="inspection",
    derivation_technique="boundary-values",
)
@pytest.mark.parametrize("snapshot_max_count", [0, 1, 3, 10], scope="class")
class TestSnapshotMaxCount(MaxSnapshotsScenario):
    """Verifies that the KVS instance reports the configured maximum number of snapshots."""

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
        assert logs_info_level.find_log("max_count", value=snapshot_max_count) is not None


@add_test_properties(
    fully_verifies=["comp_req__kvs__snapshot_restore"],
    partially_verifies=[
        "comp_req__kvs__snapshot_creation",
        "comp_req__kvs__snapshot_rotate",
    ],
    test_type="control-flow-analysis",
    derivation_technique="requirements-analysis",
)
@pytest.mark.parametrize("snapshot_max_count", [3, 10], scope="class")
class TestSnapshotRestorePrevious(MaxSnapshotsScenario):
    """Verifies restoring to a previous snapshot returns the expected value."""

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


@add_test_properties(
    partially_verifies=["comp_req__kvs__snapshot_creation"],
    test_type="fault-injection",
    derivation_technique="requirements-analysis",
)
class TestSnapshotRestoreCurrent(CommonScenario):
    """Checks that restoring the current snapshot ID fails with InvalidSnapshotId error."""

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

    def test_error(
        self,
        results: ScenarioResult,
        logs_info_level: LogContainer,
        version: str,
    ):
        assert results.return_code == ResultCode.SUCCESS

        if version == "rust":
            assert "Restoring current KVS snapshot is not allowed" in results.stdout

        result_log = logs_info_level.find_log("result")
        assert result_log is not None
        assert result_log.result == "Err(InvalidSnapshotId)"


@add_test_properties(
    partially_verifies=[
        "comp_req__kvs__snapshot_creation",
        "comp_req__kvs__snapshot_restore",
    ],
    test_type="fault-injection",
    derivation_technique="requirements-analysis",
)
class TestSnapshotRestoreNonexistent(CommonScenario):
    """Checks that restoring a non-existing snapshot fails with InvalidSnapshotId error."""

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

    def test_error(self, results: ScenarioResult, logs_info_level: LogContainer, version: str):
        assert results.return_code == ResultCode.SUCCESS

        if version == "rust":
            assert "Unable to restore non-existing snapshot" in results.stdout

        result_log = logs_info_level.find_log("result")
        assert result_log is not None
        assert result_log.result == "Err(InvalidSnapshotId)"


@add_test_properties(
    partially_verifies=["comp_req__kvs__snapshot_creation"],
    test_type="interface-test",
    derivation_technique="requirements-analysis",
)
class TestSnapshotPathsExist(CommonScenario):
    """Verifies that the KVS and hash filenames for an existing snapshot is generated correctly."""

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
        assert paths_log.kvs_path == f"{temp_dir}/kvs_1_1.json"
        assert Path(paths_log.kvs_path).exists()
        assert paths_log.hash_path == f"{temp_dir}/kvs_1_1.hash"
        assert Path(paths_log.hash_path).exists()


@add_test_properties(
    partially_verifies=["comp_req__kvs__snapshot_creation"],
    test_type="fault-injection",
    derivation_technique="requirements-analysis",
)
class TestSnapshotPathsNonexistent(CommonScenario):
    """Checks that requesting the KVS and hash filenames for a non-existing snapshot returns FileNotFound error."""

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
        assert paths_log.kvs_path == f"{temp_dir}/kvs_1_2.json"
        assert not Path(paths_log.kvs_path).exists()
        assert paths_log.hash_path == f"{temp_dir}/kvs_1_2.hash"
        assert not Path(paths_log.hash_path).exists()
