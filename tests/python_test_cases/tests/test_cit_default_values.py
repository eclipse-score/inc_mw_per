import json
from pathlib import Path
import re
from typing import Any, Generator
import pytest
from common import CommonScenario, ResultCode, temp_dir_common
from testing_utils import ScenarioResult, LogContainer

pytestmark = pytest.mark.parametrize("version", ["rust"], scope="class")

# Type tag and value pair.
TaggedValue = tuple[str, Any]


def create_defaults_json(values: dict[str, TaggedValue]) -> str:
    """
    Create JSON string containing default values.
    """
    # Create defaults.
    json_value = dict()
    for key, tagged_value in values.items():
        type_tag, value = tagged_value
        json_value[key] = {"t": type_tag, "v": value}

    return json.dumps(json_value)


def create_defaults_file(
    dir_path: Path, instance_id: int, values: dict[str, TaggedValue]
) -> Path:
    """
    Create file containing default values.
    """
    # Path to expected defaults file.
    # E.g., `/tmp/xyz/kvs_0_default.json`.
    defaults_file_path = dir_path / f"kvs_{instance_id}_default.json"

    # Create JSON string containing default values.
    json_str = create_defaults_json(values)

    # Save to file.
    with open(defaults_file_path, mode="w", encoding="UTF-8") as file:
        file.write(json_str)

    return defaults_file_path


class DefaultValuesScenario(CommonScenario):
    """
    Common base implementation for default values tests.
    """

    def instance_id(self) -> int:
        return 1

    @pytest.fixture(scope="class")
    def temp_dir(
        self, tmp_path_factory: pytest.TempPathFactory, version: str, defaults: str
    ) -> Generator[Path, None, None]:
        """
        Create temporary directory and remove it after test.
        """
        yield from temp_dir_common(
            tmp_path_factory, self.__class__.__name__, version, defaults
        )


@pytest.mark.PartiallyVerifies(
    [
        "comp_req__persistency__value_default",
        "comp_req__persistency__default_value_config",
        "comp_req__persistency__default_value_types",
        "comp_req__persistency__default_value_query",
    ]
)
@pytest.mark.FullyVerifies([])
@pytest.mark.Description(
    "Verifies default value loading, querying, and override behavior for KVS instances with and without defaults."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("requirements-based")
@pytest.mark.parametrize("defaults", ["optional", "required", "without"], scope="class")
class TestDefaultValues(DefaultValuesScenario):
    KEY = "test_number"
    VALUE = 111.1

    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.default_values.default_values"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path, defaults: str) -> dict[str, Any]:
        # Use 'optional' for no defaults file to allow init.
        if defaults == "without":
            defaults = "optional"

        return {
            "kvs_parameters": {
                "instance_id": self.instance_id(),
                "dir": str(temp_dir),
                "defaults": defaults,
            }
        }

    @pytest.fixture(scope="class")
    def defaults_file(self, temp_dir: Path, defaults: str) -> Path | None:
        assert defaults in ("optional", "required", "without")
        if defaults == "without":
            return None

        return create_defaults_file(
            temp_dir, self.instance_id(), {self.KEY: ("f64", self.VALUE)}
        )

    def test_valid(
        self,
        defaults_file: Path | None,
        results: ScenarioResult,
        logs_info_level: LogContainer,
    ) -> None:
        assert results.return_code == ResultCode.SUCCESS

        logs = logs_info_level.get_logs("key", value=self.KEY)
        assert len(logs) == 2

        if defaults_file is not None:
            # Check values before change.
            assert logs[0].value_is_default == "Ok(true)"
            assert logs[0].default_value == f"Ok(F64({self.VALUE}))"
            assert logs[0].current_value == f"Ok(F64({self.VALUE}))"
            # Check values after change.
            assert logs[1].value_is_default == "Ok(false)"
            assert logs[1].default_value == f"Ok(F64({self.VALUE}))"
            assert logs[1].current_value == "Ok(F64(432.1))"

        else:
            # Check values before change.
            assert logs[0].value_is_default == "Err(KeyNotFound)"
            assert logs[0].default_value == "Err(KeyNotFound)"
            assert logs[0].current_value == "Err(KeyNotFound)"
            # Check values after change.
            assert logs[1].value_is_default == "Ok(false)"
            assert logs[1].default_value == "Err(KeyNotFound)"
            assert logs[1].current_value == "Ok(F64(432.1))"


@pytest.mark.PartiallyVerifies(
    [
        "comp_req__persistency__value_default",
        "comp_req__persistency__default_value_config",
        "comp_req__persistency__default_value_types",
    ]
)
@pytest.mark.FullyVerifies([])
@pytest.mark.Description(
    "Tests removal of values in KVS with defaults enabled, ensuring keys revert to their default values."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("requirements-based")
@pytest.mark.parametrize("defaults", ["optional", "required", "without"], scope="class")
class TestRemoveKey(DefaultValuesScenario):
    KEY = "test_number"
    VALUE = 111.1

    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.default_values.remove_key"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path, defaults: str) -> dict[str, Any]:
        # Use 'optional' for no defaults file to allow init.
        if defaults == "without":
            defaults = "optional"

        return {
            "kvs_parameters": {
                "instance_id": self.instance_id(),
                "dir": str(temp_dir),
                "defaults": defaults,
            }
        }

    @pytest.fixture(scope="class")
    def defaults_file(self, temp_dir: Path, defaults: str) -> Path | None:
        assert defaults in ("optional", "required", "without")
        if defaults == "without":
            return None

        return create_defaults_file(
            temp_dir, self.instance_id(), {self.KEY: ("f64", self.VALUE)}
        )

    def test_valid(
        self,
        defaults_file: Path | None,
        results: ScenarioResult,
        logs_info_level: LogContainer,
    ) -> None:
        assert results.return_code == ResultCode.SUCCESS

        logs = logs_info_level.get_logs("key", value=self.KEY)
        assert len(logs) == 3

        if defaults_file is not None:
            # Check values before set.
            assert logs[0].value_is_default == "Ok(true)"
            assert logs[0].default_value == f"Ok(F64({self.VALUE}))"
            assert logs[0].current_value == f"Ok(F64({self.VALUE}))"
            # Check values after set.
            assert logs[1].value_is_default == "Ok(false)"
            assert logs[1].default_value == f"Ok(F64({self.VALUE}))"
            assert logs[1].current_value == "Ok(F64(432.1))"
            # Check values after remove.
            assert logs[2].value_is_default == "Ok(true)"
            assert logs[2].default_value == f"Ok(F64({self.VALUE}))"
            assert logs[2].current_value == f"Ok(F64({self.VALUE}))"

        else:
            # Check values before set.
            assert logs[0].value_is_default == "Err(KeyNotFound)"
            assert logs[0].default_value == "Err(KeyNotFound)"
            assert logs[0].current_value == "Err(KeyNotFound)"
            # Check values after set.
            assert logs[1].value_is_default == "Ok(false)"
            assert logs[1].default_value == "Err(KeyNotFound)"
            assert logs[1].current_value == "Ok(F64(432.1))"
            # Check values after remove.
            assert logs[2].value_is_default == "Err(KeyNotFound)"
            assert logs[2].default_value == "Err(KeyNotFound)"
            assert logs[2].current_value == "Err(KeyNotFound)"


@pytest.mark.PartiallyVerifies(
    [
        "comp_req__persistency__value_default",
        "comp_req__persistency__default_value_config",
        "comp_req__persistency__default_value_types",
    ]
)
@pytest.mark.FullyVerifies([])
@pytest.mark.Description(
    "Verifies that KVS fails to open when the defaults file contains invalid JSON."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("requirements-based")
@pytest.mark.parametrize("defaults", ["optional", "required"], scope="class")
class TestMalformedDefaultsFile(DefaultValuesScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.default_values.default_values"

    def capture_stderr(self) -> bool:
        return True

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path, defaults: str) -> dict[str, Any]:
        return {
            "kvs_parameters": {
                "instance_id": self.instance_id(),
                "dir": str(temp_dir),
                "defaults": defaults,
            }
        }

    @pytest.fixture(scope="class")
    def defaults_file(self, temp_dir: Path, defaults: str) -> Path | None:
        assert defaults in ("optional", "required")

        # Create malformed JSON string by removing last characters.
        key = "test_number"
        value = 111.1
        json_str = create_defaults_json({key: ("f64", value)})[:-2]

        defaults_file_path = temp_dir / f"kvs_{self.instance_id()}_default.json"
        with open(defaults_file_path, mode="w", encoding="UTF-8") as file:
            file.write(json_str)

        return defaults_file_path

    def test_invalid(
        self,
        defaults_file: Path | None,
        results: ScenarioResult,
    ) -> None:
        assert defaults_file is not None
        assert results.return_code == ResultCode.PANIC
        assert results.stderr is not None
        pattern = r'error: file ".*" could not be read: JsonParserError'
        assert re.findall(pattern, results.stderr) is not None


@pytest.mark.PartiallyVerifies(
    [
        "comp_req__persistency__value_default",
        "comp_req__persistency__default_value_config",
        "comp_req__persistency__default_value_types",
    ]
)
@pytest.mark.FullyVerifies([])
@pytest.mark.Description(
    "Verifies that KVS fails to open when the defaults file is missing."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("requirements-based")
@pytest.mark.parametrize("defaults", ["required"], scope="class")
class TestMissingDefaultsFile(DefaultValuesScenario):
    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.default_values.default_values"

    def capture_stderr(self) -> bool:
        return True

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path, defaults: str) -> dict[str, Any]:
        return {
            "kvs_parameters": {
                "instance_id": self.instance_id(),
                "dir": str(temp_dir),
                "defaults": defaults,
            }
        }

    def test_invalid(self, results: ScenarioResult) -> None:
        assert results.return_code == ResultCode.PANIC
        assert results.stderr is not None
        pattern = r'error: file ".*" could not be read: KvsFileReadError'
        assert re.findall(pattern, results.stderr) is not None


@pytest.mark.PartiallyVerifies(
    [
        "comp_req__persistency__value_default",
        "comp_req__persistency__default_value_config",
        "comp_req__persistency__default_value_types",
    ]
)
@pytest.mark.FullyVerifies(["comp_req__persistency__value_reset"])
@pytest.mark.Description(
    "Checks that resetting KVS restores all keys to their default values."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("requirements-based")
@pytest.mark.parametrize("defaults", ["optional", "required"], scope="class")
class TestResetAllKeys(DefaultValuesScenario):
    NUM_VALUES = 5

    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.default_values.reset_all_keys"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path, defaults: str) -> dict[str, Any]:
        return {
            "kvs_parameters": {
                "instance_id": self.instance_id(),
                "dir": str(temp_dir),
                "defaults": defaults,
            }
        }

    @pytest.fixture(scope="class")
    def defaults_file(self, temp_dir: Path, defaults: str) -> Path | None:
        assert defaults in ("optional", "required")

        values = {}
        for i in range(self.NUM_VALUES):
            values[f"test_number_{i}"] = ("f64", 432.1 * i)

        return create_defaults_file(temp_dir, self.instance_id(), values)

    def test_valid(
        self,
        defaults_file: Path | None,
        results: ScenarioResult,
        logs_info_level: LogContainer,
    ):
        assert defaults_file is not None
        assert results.return_code == ResultCode.SUCCESS

        for i in range(self.NUM_VALUES):
            logs = logs_info_level.get_logs("key", value=f"test_number_{i}")

            # Check values before set.
            assert logs[0].value_is_default
            assert logs[0].current_value == 432.1 * i

            # Check values after set.
            assert not logs[1].value_is_default
            assert logs[1].current_value == 123.4 * i

            # Check values after reset.
            assert logs[2].value_is_default
            assert logs[2].current_value == 432.1 * i


@pytest.mark.PartiallyVerifies(
    [
        "comp_req__persistency__value_default",
        "comp_req__persistency__default_value_config",
    ]
)
@pytest.mark.FullyVerifies([])
@pytest.mark.Description(
    "Checks that resetting single key restores it to its default value."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("requirements-based")
@pytest.mark.parametrize("defaults", ["optional", "required"], scope="class")
class TestResetSingleKey(DefaultValuesScenario):
    NUM_VALUES = 5
    RESET_INDEX = 2

    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.default_values.reset_single_key"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path, defaults: str) -> dict[str, Any]:
        return {
            "kvs_parameters": {
                "instance_id": self.instance_id(),
                "dir": str(temp_dir),
                "defaults": defaults,
            }
        }

    @pytest.fixture(scope="class")
    def defaults_file(self, temp_dir: Path, defaults: str) -> Path | None:
        assert defaults in ("optional", "required")

        values = {}
        for i in range(self.NUM_VALUES):
            values[f"test_number_{i}"] = ("f64", 432.1 * i)

        return create_defaults_file(temp_dir, self.instance_id(), values)

    def test_valid(
        self,
        defaults_file: Path | None,
        results: ScenarioResult,
        logs_info_level: LogContainer,
    ):
        assert defaults_file is not None
        assert results.return_code == ResultCode.SUCCESS

        for i in range(self.NUM_VALUES):
            logs = logs_info_level.get_logs("key", value=f"test_number_{i}")

            if i == self.RESET_INDEX:
                # Check values before set.
                assert logs[0].value_is_default
                assert logs[0].current_value == 432.1 * i

                # Check values after set.
                assert not logs[1].value_is_default
                assert logs[1].current_value == 123.4 * i

                # Check values after reset.
                assert logs[2].value_is_default
                assert logs[2].current_value == 432.1 * i

            else:
                # Check values before set.
                assert logs[0].value_is_default
                assert logs[0].current_value == 432.1 * i

                # Check values after set.
                assert not logs[1].value_is_default
                assert logs[1].current_value == 123.4 * i

                # Check values after reset.
                assert not logs[2].value_is_default
                assert logs[2].current_value == 123.4 * i


@pytest.mark.PartiallyVerifies(
    [
        "comp_req__persistency__value_default",
        "comp_req__persistency__default_value_config",
    ]
)
@pytest.mark.FullyVerifies(["comp_req__persistency__default_value_checksum"])
@pytest.mark.Description(
    "Ensures that a checksum file is created when opening KVS with defaults."
)
@pytest.mark.TestType("requirements-based")
@pytest.mark.DerivationTechnique("requirements-based")
@pytest.mark.parametrize("defaults", ["optional", "required"], scope="class")
class TestChecksumOnProvidedDefaults(DefaultValuesScenario):
    KEY = "test_number"
    VALUE = 111.1

    @pytest.fixture(scope="class")
    def scenario_name(self) -> str:
        return "cit.default_values.checksum"

    @pytest.fixture(scope="class")
    def test_config(self, temp_dir: Path, defaults: str) -> dict[str, Any]:
        return {
            "kvs_parameters": {
                "instance_id": self.instance_id(),
                "dir": str(temp_dir),
                "defaults": defaults,
            }
        }

    @pytest.fixture(scope="class")
    def defaults_file(self, temp_dir: Path, defaults: str) -> Path | None:
        assert defaults in ("optional", "required", "without")
        if defaults == "without":
            return None

        return create_defaults_file(
            temp_dir, self.instance_id(), {self.KEY: ("f64", self.VALUE)}
        )

    def test_valid(
        self,
        defaults_file: Path | None,
        results: ScenarioResult,
        logs_info_level: LogContainer,
    ) -> None:
        assert defaults_file is not None
        assert results.return_code == ResultCode.SUCCESS

        logs = logs_info_level.get_logs(field="kvs_path")
        assert len(logs) == 1

        kvs_path = Path(logs[0].kvs_path)
        assert kvs_path.is_file()
        hash_path = Path(logs[0].hash_path)
        assert hash_path.is_file()
