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

////////////////////////////////////////////////////////////////////////////////
/* The test_kvs_general files provide configuration data and methods needed for KVS tests */
////////////////////////////////////////////////////////////////////////////////

#include <cstdio>
#include <cstdlib>
#include <filesystem>
#include <fstream>
#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <string>
#include <unistd.h>

/* Change Private Members and final to public to allow access to member variables (kvs and kvsbuilder) and derive from kvsvalue in unittests*/
#define private public
#define final 
#include "kvsbuilder.hpp"
#undef private
#undef final
#include "internal/kvs_helper.hpp"
#include "score/json/i_json_parser_mock.h"
#include "score/json/i_json_writer_mock.h"
#include "score/filesystem/filesystem_mock.h"
using namespace score::mw::per::kvs;

////////////////////////////////////////////////////////////////////////////////

uint32_t adler32(const std::string& data);
void prepare_environment();
void cleanup_environment();

////////////////////////////////////////////////////////////////////////////////
/* Default data used in unittests*/
////////////////////////////////////////////////////////////////////////////////

const std::uint32_t instance = 123;
const InstanceId instance_id{instance};

/* Notice: score::filesystem::Path could be constructed implicitly from std::string, but for readability,
           the explicit construction from those strings are used in the testcode */
const std::string data_dir = "./data_folder/";
const std::string default_prefix = data_dir + "kvs_"+std::to_string(instance)+"_default";
const std::string kvs_prefix     = data_dir + "kvs_"+std::to_string(instance)+"_0";
const std::string filename_prefix = data_dir + "kvs_"+std::to_string(instance);

const std::string default_json = R"({
    "default": {
        "t": "i32",
        "v": 5
    }
})";
const std::string kvs_json = R"({
    "kvs": {
        "t": "i32",
        "v": 2
    }
})";

////////////////////////////////////////////////////////////////////////////////
/* Mock KvsValue for testing purposes (kvsvalue_to_any) */
////////////////////////////////////////////////////////////////////////////////

class BrokenKvsValue : public KvsValue {
public:
    BrokenKvsValue() : KvsValue(nullptr) {
        /* Intentionally break the type by assigning an invalid value */
        *(Type*)&this->type = static_cast<Type>(999);
    }
};

////////////////////////////////////////////////////////////////////////////////
