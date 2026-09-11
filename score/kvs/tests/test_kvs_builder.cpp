/********************************************************************************
 * Copyright (c) 2025 Contributors to the Eclipse Foundation
 *
 * See the NOTICE file(s) distributed with this work for additional
 * information regarding copyright ownership.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Apache License Version 2.0 which is available at
 * https://www.apache.org/licenses/LICENSE-2.0
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/
#include "test_kvs_general.hpp"

TEST(kvs_kvsbuilder, kvsbuilder_build)
{
    /* This test also checks the kvs open function with the KvsBuilder */

    /* Test the KvsBuilder constructor */
    KvsBuilder builder(instance_id);
    EXPECT_EQ(builder.instance_id.id, instance_id.id);
    EXPECT_EQ(builder.need_defaults, false);
    EXPECT_EQ(builder.need_kvs, false);

    /* Test the KvsBuilder methods */
    builder.need_defaults_flag(true);
    EXPECT_EQ(builder.need_defaults, true);
    builder.need_kvs_flag(true);
    EXPECT_EQ(builder.need_kvs, true);
    builder.dir("./kvsbuilder/");
    EXPECT_EQ(builder.directory, "./kvsbuilder/");

    /* Test the KvsBuilder build method */
    /* We want to check, if OpenNeedDefaults::Required and OpenNeedKvs::Required is passed correctly
    open() function should return an error, since the files are not available */
    auto result_build = builder.build();
    ASSERT_FALSE(result_build);
    EXPECT_EQ(static_cast<ErrorCode>(*result_build.error()),
              ErrorCode::KvsFileReadError); /* This error occurs in open_json and is passed through
                                               open()*/
    builder.need_defaults_flag(false);
    result_build = builder.build();
    ASSERT_FALSE(result_build);
    EXPECT_EQ(static_cast<ErrorCode>(*result_build.error()),
              ErrorCode::KvsFileReadError); /* This error occurs in open_json and is passed through
                                               open()*/
    builder.need_kvs_flag(false);
    result_build = builder.build();
    EXPECT_TRUE(result_build);
    EXPECT_EQ(result_build.value().filename_prefix.CStr(), "./kvsbuilder/kvs_" + std::to_string(instance_id.id));
}

TEST(kvs_kvsbuilder, kvsbuilder_directory_check)
{
    /* Test the KvsBuilder with all configurations for the current working directory */
    KvsBuilder builder(instance_id);
    builder.dir("");
    auto result_build = builder.build();
    EXPECT_TRUE(result_build);
    EXPECT_EQ(result_build.value().filename_prefix.CStr(), "./kvs_" + std::to_string(instance_id.id));

    builder.dir("./");
    result_build = builder.build();
    EXPECT_TRUE(result_build);
    EXPECT_EQ(result_build.value().filename_prefix.CStr(), "./kvs_" + std::to_string(instance_id.id));

    builder.dir(".");
    result_build = builder.build();
    EXPECT_TRUE(result_build);
    EXPECT_EQ(result_build.value().filename_prefix.CStr(), "./kvs_" + std::to_string(instance_id.id));
}

TEST(kvs_kvsbuilder, kvsbuilder_snapshot_max_count_default)
{
    /* comp_req__kvs__snapshot_max_num:
       An unconfigured builder must keep the compile-time default, so that existing
       users observe no behavior change. */
    KvsBuilder builder(instance_id);
    EXPECT_EQ(builder.max_snapshots, KVS_DEFAULT_MAX_SNAPSHOTS);

    builder.dir(std::string(data_dir));
    auto result_build = builder.build();
    ASSERT_TRUE(result_build);
    EXPECT_EQ(result_build.value().snapshot_max_count(), KVS_DEFAULT_MAX_SNAPSHOTS);

    cleanup_environment();
}

TEST(kvs_kvsbuilder, kvsbuilder_snapshot_max_count_configured)
{
    /* comp_req__kvs__snapshot_max_num:
       The configured value must reach the built KVS instance. Without the plumbing
       through build()/open() the setter would be silently ignored. */
    const std::size_t configured_max = 7U;

    KvsBuilder builder(instance_id);
    builder.snapshot_max_count(configured_max);
    EXPECT_EQ(builder.max_snapshots, configured_max);

    /* The setter must be chainable like every other builder option */
    EXPECT_EQ(&builder.snapshot_max_count(configured_max), &builder);

    builder.dir(std::string(data_dir));
    auto result_build = builder.build();
    ASSERT_TRUE(result_build);
    EXPECT_EQ(result_build.value().snapshot_max_count(), configured_max);

    cleanup_environment();
}
