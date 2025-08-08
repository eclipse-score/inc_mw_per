load("@rules_cc//cc:cc_binary.bzl", "cc_binary")

cc_binary(
    name = "cpp_test_scenarios",
    srcs = [
        "src/main.cpp",
        "src/test_basic.cpp",
        "src/test_basic.hpp",
    ],
    copts = [
        "-g",
    ],
    visibility = ["//visibility:private"],
    deps = [
        "//src/cpp/src:kvs_cpp",
        "@testing-utils//test_scenarios_cpp:test_scenarios_cpp",
    ],
)
