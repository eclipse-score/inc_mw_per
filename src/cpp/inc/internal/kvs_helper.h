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

#include <sstream>
#include <string>

/* 
 * This header defines helper functions used internally by the Key-Value Store (KVS) implementation.
 * It exists to allow unit tests to access these internal functions.
 */

uint32_t parse_hash_adler32(std::istream& in);
uint32_t calculate_hash_adler32(const std::string& data);
std::array<uint8_t,4> get_hash_bytes_adler32(uint32_t hash);
std::array<uint8_t,4> get_hash_bytes(const std::string& data);
score::Result<std::unordered_map<std::string, KvsValue>> parse_json_data(const std::string& data);
score::Result<std::unordered_map<std::string, KvsValue>> open_json(const std::string& prefix, OpenJsonNeedFile need_file);
score::ResultBlank write_json_data(const std::string& filename_prefix, const std::string& buf);
score::Result<KvsValue> any_to_kvsvalue(const score::json::Any& any);
score::Result<score::json::Any> kvsvalue_to_any(const KvsValue& kv);
