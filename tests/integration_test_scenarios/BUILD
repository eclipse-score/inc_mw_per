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

load("@score_toolchains_qnx//rules/fs:ifs.bzl", "qnx_ifs")

config_setting(
    name = "is_qnx_x86_64",
    constraint_values = [
        "@platforms//cpu:x86_64",
        "@platforms//os:qnx",
    ],
)

config_setting(
    name = "is_qnx_aarch64",
    constraint_values = [
        "@platforms//cpu:aarch64",
        "@platforms//os:qnx",
    ],
)

# Build artifact comes from //src/rust/rust_kvs_tool:kvs_tool (built for QNX by --platforms)
genrule(
    name = "stage_kvs_tool",
    srcs = ["//src/rust/rust_kvs_tool:kvs_tool"],
    outs = ["install/usr/bin/kvs_tool"],
    cmd = """
      mkdir -p $(RULEDIR)/install/usr/bin
      cp $(location //src/rust/rust_kvs_tool:kvs_tool) $@
      chmod +x $@
    """,
    visibility = ["//visibility:public"],
)

# Stage the integration test script that exercises kvs_tool.
# Put the script next to this BUILD file as kvs_test.sh (see snippet below).
genrule(
    name = "stage_kvs_test_sh",
    srcs = ["kvs_test.sh"],
    outs = ["install/usr/bin/kvs_test.sh"],
    cmd = """
      mkdir -p $(RULEDIR)/install/usr/bin
      cp $(location kvs_test.sh) $@
      chmod +x $@
    """,
)

# state the tests too
#TODO: enable when rust_test_scenarios is built for QNX
#genrule(
#    name = "stage_rust_test_scenarios",
#    srcs = ["//tests/rust_test_scenarios:rust_test_scenarios"],
#    outs = ["install/usr/bin/rust_test_scenarios"],
#    cmd = """
#      mkdir -p $(RULEDIR)/install/usr/bin
#      cp $(location //tests/rust_test_scenarios:rust_test_scenarios) $@
#      chmod +x $@
#    """,
#)

filegroup(
    name = "overlay_tree",
    srcs = glob(["install/**"]),
    visibility = ["//visibility:public"],
)

qnx_ifs(
    name = "init",
    # Only include the install tree as inputs on aarch64
    srcs = select({
        ":is_qnx_aarch64": [
            ":stage_kvs_tool",
            ":stage_kvs_test_sh",
            #            ":stage_rust_test_scenarios",
            ":overlay_tree",
        ],
        "//conditions:default": [],
    }),
    out = select({
        ":is_qnx_x86_64": "init_x86_64.ifs",
        ":is_qnx_aarch64": "init_aarch64.ifs",
    }),
    build_file = select({
        ":is_qnx_x86_64": "init_x86_64.build",
        ":is_qnx_aarch64": "init_rpi4.build",
    }),

    # Only pass -r on aarch64 (mkifs will receive: -r install)
    search_roots = select({
        ":is_qnx_aarch64": ["install"],  # relative to the .build file’s dir
        "//conditions:default": [],
    }),
)

sh_binary(
    name = "run_qemu",
    srcs = ["run_qemu.sh"],
    args = [
        "$(location @toolchains_qnx_sdp//:host_dir)",
        "$(location :init)",
        "$(location @custom_qemu//:qemu_bin)",
        "$(location @custom_qemu//:rpi4_dtb)",
    ],
    data = [
        ":init",
        "@custom_qemu//:qemu_bin",
        "@custom_qemu//:rpi4_dtb",
        "@toolchains_qnx_sdp//:host_all",
        "@toolchains_qnx_sdp//:host_dir",
    ],
)
