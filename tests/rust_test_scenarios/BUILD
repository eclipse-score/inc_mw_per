load("@rules_rust//rust:defs.bzl", "rust_binary")
load("@score_persistency_crates//:defs.bzl", "all_crate_deps")

rust_binary(
    name = "rust_test_scenarios",
    srcs = glob(["src/**/*.rs"]),
    visibility = ["//tests/python_test_cases:__pkg__"],
    deps =
        [
            "//src/rust/rust_kvs",
            "@score_persistency_crates//:tinyjson",
        ] + all_crate_deps(normal = True),
)
