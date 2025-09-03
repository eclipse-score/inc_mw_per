# Key-Value-Storage

## License

```text
Copyright (c) 2025 Contributors to the Eclipse Foundation

See the NOTICE file(s) distributed with this work for additional
information regarding copyright ownership.

This program and the accompanying materials are made available under the
terms of the Apache License Version 2.0 which is available at
https://www.apache.org/licenses/LICENSE-2.0

SPDX-License-Identifier: Apache-2.0
```

## Setup

> [!NOTE]
> This repository offers a [DevContainer](https://containers.dev/).
> For setting this up and enabling code completion read [eclipse-score/devcontainer/README.md#inside-the-container](https://github.com/eclipse-score/devcontainer/blob/main/README.md#inside-the-container).

If you chose not to use the devcontainer you have to setup the following dependencies yourself.

### System dependencies

```bash
sudo apt-get update
sudo apt-get install -y curl build-essential protobuf-compiler libclang-dev
```

### Rust installation

[Install Rust using rustup](https://www.rust-lang.org/tools/install)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

### Bazel installation

[Install Bazel using Bazelisk](https://bazel.build/install/bazelisk)

```bash
curl --proto '=https' -sSfOL https://github.com/bazelbuild/bazelisk/releases/download/v1.26.0/bazelisk-amd64.deb
dpkg -i bazelisk-amd64.deb
rm bazelisk-amd64.deb
```

Correct Bazel version will be installed on first run, based on `bazelversion` file.

## Build

List all targets:

```bash
bazel query //...
```

List curated available targets with description:

```bash
bazel run //:help
```

Build selected target:

```bash
bazel build <TARGET_NAME>
```

Build all targets:

```bash
bazel build //...
```

## Run

List all rust library targets:

```bash
bazel query 'kind(rust_library, //src/...)'
```

Run selected target:

```bash
bazel run <TARGET_NAME>
```

## Test

List all test targets:

```bash
bazel query 'kind(rust_test, //...)'
```

Run all tests:

```bash
bazel test //...
```

Run Component Integration Tests (grouped into single Test Suite):

```bash
bazel test //src/rust/rust_kvs:cit
```

Run selected test target:

```bash
bazel test <TARGET_NAME>
```

## Cargo-based operations

Please use Bazel whenever possible.

### Build with Cargo

Build using `cargo` directly:

```bash
cargo build
```

### Run CLI tool with Cargo

```bash
cargo run --help
```

```text
---------------------------------------
KVS Tool - Command Line Interface
---------------------------------------

Version 0.1.0

---------------------------------------

Options:
-h, --help          Show this help message and exit
-o, --operation     Specify the operation to perform (setkey, getkey, removekey, 
                    listkeys, reset, snapshotcount, snapshotmaxcount, snapshotrestore, 
                    getkvsfilename, gethashfilename, createtestdata)
-k, --key           Specify the key to operate on (for key operations)
-p, --payload       Specify the value to write (for set operations)
-s, --snapshotid    Specify the snapshot ID for Snapshot operations
-d, --directory     Specify the directory of the Key-Files (default is current directory)

---------------------------------------

Usage Examples:

Read a Key and show value:
    kvs_tool -o getkey -k MyKey

Write a Key and use the <payload> as the data source:
    (automatically detects following types: Number, Boolean, String, Null, Object, Array)
    kvs_tool -o setkey  -k MyKey -p 'Hello World' 
    kvs_tool -o setkey  -k MyKey -p 'true'
    kvs_tool -o setkey  -k MyKey -p 15
    kvs_tool -o setkey  -k MyKey -p '[456,false,"Second"]'
    kvs_tool -o setkey  -k MyKey -p '{"sub-number":789,"sub-array":[1246,false,"Fourth"]}'

Delete a key:
    kvs_tool -o removekey -k MyKey

List Keys:
    kvs_tool -o listkeys

Reset KVS:
    kvs_tool -o reset

Snapshot Count:
    kvs_tool -o snapshotcount

Snapshot Restore:
    kvs_tool -o snapshotrestore -s 1

Get KVS Filename:
    kvs_tool -o getkvsfilename -s 1

Get Hash Filename:
    kvs_tool -o gethashfilename -s 1

---------------------------------------

Create Test Data:
    kvs_tool -o createtestdata (Creates Data provided by the example code in the KVS API)

---------------------------------------
```

### Run tests with Cargo

Using `cargo test`:

```bash
cargo test
```

### Run examples with Cargo

Examples can be run `cargo run`.
Following examples are available: `basic`, `defaults`, `flush`, `snapshots`.

```bash
cargo run --example <EXAMPLE_NAME>
```

Basic example command:

```bash
cargo run --example basic
```
