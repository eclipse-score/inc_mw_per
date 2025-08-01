# Rust-C++ tests

For general information check [main README.md file](../README.md).

## Setup

Create `venv`, activate and install dependencies:

```bash
python -m venv <REPO_ROOT>/.venv
source <REPO_ROOT>/.venv/bin/activate
pip install -r <REPO_ROOT>/tests/python_test_cases/requirements.txt
```

## Usage

Set current working directory to the following:

```bash
cd <REPO_ROOT>/tests/python_test_cases
```

### Run tests

Basic run:

```bash
pytest .
```

Run with additional flags:

```bash
pytest -vsx . -k <PATTERN> --build-scenarios
```

- `-v` - increase verbosity.
- `-s` - show logs (disable capture).
- `-k <PATTERN>` - run tests matching the pattern.
- `--build-scenarios` - build Rust test scenarios before execution.

Run tests repeatedly:

```bash
pytest . --count <VALUE> --repeat-scope session -x
```

- `--count <VALUE>` - number of repeats.
- `--repeat-scope session` - scope of repeat.
- `-x` - exit on first error.

Refer to `pytest` manual for `pytest` specific options.
Refer to `conftest.py` for test suite specific options.

### Create HTML report

To generate HTML report use:

```bash
pytest -v . --build-scenarios --self-contained-html --html report.html --traces <VALUE>
```

- `--self-contained-html` - generate self contained HTML file.
- `--html report.html` - HTML report output path.
- `--traces <VALUE>` - verbosity of traces in output and HTML report - "none", "bin" or "all".

> Traces are collected using `stdout`.
> Setting `--capture` flag (including `-s`) might cause traces to be missing from HTML report.

## Standalone execution of test scenarios

Test scenarios can be run independently from `pytest`.

CLI interface is same for both Rust and C++ implementation.
Executables are interactive by default and prompt will require test input to be provided.
E.g., `{"kvs_parameters":{"instance_id":0,"flush_on_exit":false}}`.

Type of input can be selected with `--input-type`.
`arg` and `stdin` are accepted.

### Rust

Set current working directory to the following:

```bash
cd <REPO_ROOT>/tests/rust_test_scenarios
```

List all available scenarios:

```bash
cargo run -- --list-scenarios
```

Run specific test scenario with test input from `stdin`:

```bash
cargo run -- --name <TEST_GROUP>.<TEST_SCENARIO> <<< <TEST_INPUT>
```

Example:

```bash
cargo run -- --name basic.basic <<< '{"kvs_parameters":{"instance_id":0,"flush_on_exit":false}}'
```

Run specific test scenario with test input from argument:

```bash
cargo run -- --name <TEST_GROUP>.<TEST_SCENARIO> --input-type arg --input <TEST_INPUT>
```

Example:

```bash
cargo run -- --name basic.basic --input-type arg --input '{"kvs_parameters":{"instance_id":0,"flush_on_exit":false}}'
```

Run test scenario executable directly:

```bash
<REPO_ROOT>/target/debug/rust_test_scenarios --name basic.basic <<< '{"kvs_parameters":{"instance_id":0,"flush_on_exit":false}}'
```

### C++

Set current working directory to the following:

```bash
cd <REPO_ROOT>/tests/cpp_test_scenarios
```

List all available scenarios:

```bash
bazel run //tests/cpp_test_scenarios:cpp_test_scenarios -- --list-scenarios
```

Run specific test scenario with test input from `stdin`:

```bash
bazel run //tests/cpp_test_scenarios:cpp_test_scenarios -- --name <TEST_GROUP>.<TEST_SCENARIO> <<< <TEST_INPUT>
```

Example:

```bash
bazel run //tests/cpp_test_scenarios:cpp_test_scenarios -- --name basic.basic <<< '{"kvs_parameters":{"instance_id":0,"flush_on_exit":false}}'
```

Run specific test scenario with test input from argument:

```bash
bazel run //tests/cpp_test_scenarios:cpp_test_scenarios -- --name <TEST_GROUP>.<TEST_SCENARIO> --input-type arg --input <TEST_INPUT>
```

Example:

```bash
bazel run //tests/cpp_test_scenarios:cpp_test_scenarios -- --name basic.basic --input-type arg --input '{"kvs_parameters":{"instance_id":0,"flush_on_exit":false}}'
```

Run test scenario executable directly:

```bash
<REPO_ROOT>/bazel-bin/tests/cpp_test_scenarios/cpp_test_scenarios --name basic.basic <<< '{"kvs_parameters":{"instance_id":0,"flush_on_exit":false}}'
```

Run with GDB:

```bash
bazel build //tests/cpp_test_scenarios:cpp_test_scenarios -c dbg --strip never
gdb --args <REPO_ROOT>/bazel-bin/tests/cpp_test_scenarios/cpp_test_scenarios --name <TEST_GROUP>.<TEST_SCENARIO> --input-type arg --input '{"kvs_parameters":{"instance_id":0,"flush_on_exit":false}}'
```
