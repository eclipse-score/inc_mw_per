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

/* adler32 control instance */
uint32_t adler32(const std::string& data) {
    const uint32_t mod = 65521;
    uint32_t a = 1, b = 0;
    for (unsigned char c : data) {
        a = (a + c) % mod;
        b = (b + a) % mod;
    }
    return (b << 16) | a;
}

/* Create Test environment with default data, which is needed in most testcases */
void prepare_environment(){
    /* Prepare the test environment */
    mkdir(data_dir.c_str(), 0777);

    std::ofstream default_json_file(default_prefix + ".json");
    default_json_file << default_json;
    default_json_file.close();

    std::ofstream kvs_json_file(kvs_prefix + ".json");
    kvs_json_file << kvs_json;
    kvs_json_file.close();

    uint32_t default_hash = adler32(default_json);
    uint32_t kvs_hash = adler32(kvs_json);

    std::ofstream default_hash_file(default_prefix + ".hash", std::ios::binary);
    default_hash_file.put((default_hash >> 24) & 0xFF);
    default_hash_file.put((default_hash >> 16) & 0xFF);
    default_hash_file.put((default_hash >> 8)  & 0xFF);
    default_hash_file.put(default_hash & 0xFF);
    default_hash_file.close();

    std::ofstream kvs_hash_file(kvs_prefix + ".hash", std::ios::binary);
    kvs_hash_file.put((kvs_hash >> 24) & 0xFF);
    kvs_hash_file.put((kvs_hash >> 16) & 0xFF);
    kvs_hash_file.put((kvs_hash >> 8)  & 0xFF);
    kvs_hash_file.put(kvs_hash & 0xFF);
    kvs_hash_file.close();
}

void cleanup_environment() {
    /* Cleanup the test environment */
    if (std::filesystem::exists(data_dir)) {
        for (auto& p : std::filesystem::recursive_directory_iterator(data_dir)) {
            std::filesystem::permissions(p,
                std::filesystem::perms::owner_all | std::filesystem::perms::group_all | std::filesystem::perms::others_all,
                std::filesystem::perm_options::replace);
        }
        std::filesystem::remove_all(data_dir);
    }
}
