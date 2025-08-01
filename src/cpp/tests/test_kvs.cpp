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

#include <cstdio>
#include <cstdlib>
#include <filesystem>
#include <fstream>
#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <string>
#include <unistd.h>

/* Change Private Members and final to public to allow access to member variables and derive from kvsvalue in unittests*/
#define private public
#define final 
#include "kvs.hpp"
#undef private
#undef final
#include "internal/kvs_helper.hpp"
#include "score/json/i_json_parser_mock.h"
#include "score/json/i_json_writer_mock.h"
#include "score/filesystem/filesystem_mock.h"

////////////////////////////////////////////////////////////////////////////////
/* Test Environment Setup - Standard Variables for tests*/

const std::uint32_t instance = 123;
const InstanceId instance_id{instance};

/* Notice: score::filesystem::Path could be constructed implicitly from std::string, but for readability, 
           we mostly use explicit construction from those strings in the testcode */
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

////////////////////////////////////////////////////////////////////////////////

TEST(kvs_calculate_hash_adler32, calculate_hash_adler32) {
    /* Test the adler32 hash calculation 
    Parsing test is done automatically by the open tests */

    std::string test_data = "Hello, World!";
    uint32_t calculated_hash = adler32(test_data);
    EXPECT_EQ(calculated_hash, calculate_hash_adler32(test_data));

    std::array<uint8_t,4> buf{};
    std::istringstream stream(test_data);
    stream.read(reinterpret_cast<char*>(buf.data()), buf.size());

    std::array<uint8_t, 4> value = {
        uint8_t((calculated_hash >> 24) & 0xFF),
        uint8_t((calculated_hash >> 16) & 0xFF),
        uint8_t((calculated_hash >>  8) & 0xFF),
        uint8_t((calculated_hash      ) & 0xFF)
    };
    EXPECT_EQ(value, get_hash_bytes(test_data));

}

TEST(kvs_calculate_hash_adler32, calculate_hash_adler32_large_data) {
    // Create Teststring with more than 5552 characters to ensure that the hash is calculated correctly
    std::string large_data(6000, 'A');
    uint32_t hash = calculate_hash_adler32(large_data);

    EXPECT_EQ(adler32(large_data), hash);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_bool) {
    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("bool")));
    obj.emplace("v", score::json::Any(true));
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    ASSERT_TRUE(result);
    EXPECT_EQ(result.value().getType(), KvsValue::Type::Boolean);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_i32) {
    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("i32")));
    obj.emplace("v", score::json::Any(42.0));
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    ASSERT_TRUE(result);
    EXPECT_EQ(result.value().getType(), KvsValue::Type::i32);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_u32) {
    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("u32")));
    obj.emplace("v", score::json::Any(42.0));
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    ASSERT_TRUE(result);
    EXPECT_EQ(result.value().getType(), KvsValue::Type::u32);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_i64) {
    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("i64")));
    obj.emplace("v", score::json::Any(42.0));
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    ASSERT_TRUE(result);
    EXPECT_EQ(result.value().getType(), KvsValue::Type::i64);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_u64) {
    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("u64")));
    obj.emplace("v", score::json::Any(42.0));
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    ASSERT_TRUE(result);
    EXPECT_EQ(result.value().getType(), KvsValue::Type::u64);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_f64) {
    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("f64")));
    obj.emplace("v", score::json::Any(42.0));
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    ASSERT_TRUE(result);
    EXPECT_EQ(result.value().getType(), KvsValue::Type::f64);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_string) {
    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("str")));
    obj.emplace("v", score::json::Any(std::string("test")));
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    ASSERT_TRUE(result);
    EXPECT_EQ(result.value().getType(), KvsValue::Type::String);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_null) {
    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("null")));
    obj.emplace("v", score::json::Any(score::json::Null()));
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    ASSERT_TRUE(result);
    EXPECT_EQ(result.value().getType(), KvsValue::Type::Null);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_array) {
    score::json::List list;
    score::json::Object inner_obj_1;
    inner_obj_1.emplace("t", score::json::Any(std::string("bool")));
    inner_obj_1.emplace("v", score::json::Any(true));
    list.push_back(score::json::Any(std::move(inner_obj_1)));

    score::json::Object inner_obj_2;
    inner_obj_2.emplace("t", score::json::Any(std::string("f64")));
    inner_obj_2.emplace("v", score::json::Any(1.1));
    list.push_back(score::json::Any(std::move(inner_obj_2)));

    score::json::Object inner_obj_3;
    inner_obj_3.emplace("t", score::json::Any(std::string("str")));
    inner_obj_3.emplace("v", score::json::Any(std::string("test")));
    list.push_back(score::json::Any(std::move(inner_obj_3)));

    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("arr")));
    obj.emplace("v", score::json::Any(std::move(list)));
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    ASSERT_TRUE(result);
    EXPECT_EQ(result.value().getType(), KvsValue::Type::Array);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_object) {
    score::json::Object inner_obj_1;
    inner_obj_1.emplace("t", score::json::Any(std::string("bool")));
    inner_obj_1.emplace("v", score::json::Any(true));

    score::json::Object inner_obj_2;
    inner_obj_2.emplace("t", score::json::Any(std::string("f64")));
    inner_obj_2.emplace("v", score::json::Any(42.0));

    score::json::Object combined_obj;
    combined_obj.emplace("flag", score::json::Any(std::move(inner_obj_1)));
    combined_obj.emplace("count", score::json::Any(std::move(inner_obj_2)));

    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("obj")));
    obj.emplace("v", score::json::Any(std::move(combined_obj)));

    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    ASSERT_TRUE(result);
    EXPECT_EQ(result.value().getType(), KvsValue::Type::Object);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_format_invalid) {
    score::json::Object obj_type;
    obj_type.emplace("invalid", score::json::Any(std::string("bool")));
    obj_type.emplace("v", score::json::Any(true));
    score::json::Any any_obj_type(std::move(obj_type));
    auto result = any_to_kvsvalue(any_obj_type);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);

    score::json::Object obj_value;
    obj_value.emplace("t", score::json::Any(std::string("bool")));
    obj_value.emplace("invalid", score::json::Any(true));
    score::json::Any any_obj_value(std::move(obj_value));
    result = any_to_kvsvalue(any_obj_value);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_no_object) {
    score::json::Any any_bool(true);
    auto result = any_to_kvsvalue(any_bool);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_type_no_string) {
    score::json::Object obj;
    obj.emplace("t", score::json::Any(42.0)); // Not a string
    obj.emplace("v", score::json::Any(true));
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_type_invalid) {
    score::json::Object obj;
    obj.emplace("t", "invalid");
    obj.emplace("v", score::json::Any(true));
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_no_string_type) {
    score::json::Object obj;
    obj.emplace("t", score::json::Any(42.0)); // Not a string
    obj.emplace("v", score::json::Any(true));
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_invalid_i32){
    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("i32")));
    obj.emplace("v", score::json::Any("invalid")); // Invalid value for I32
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_invalid_u32){
    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("u32")));
    obj.emplace("v", score::json::Any("invalid")); // Invalid value for U32
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_invalid_i64){
    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("i64")));
    obj.emplace("v", score::json::Any("invalid")); // Invalid value for I64
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_invalid_u64){
    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("u64")));
    obj.emplace("v", score::json::Any("invalid")); // Invalid value for U64
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_invalid_f64){
    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("f64")));
    obj.emplace("v", score::json::Any("invalid")); // Invalid value for F64
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_invalid_boolean){
    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("bool")));
    obj.emplace("v", score::json::Any(42.0)); // Invalid value for Boolean
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_invalid_string){
    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("str")));
    obj.emplace("v", score::json::Any(42.0)); // Invalid value for String
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_invalid_null){
    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("null")));
    obj.emplace("v", score::json::Any(42.0)); // Invalid value for Null
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_invalid_array) {
    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("arr")));
    obj.emplace("v", score::json::Any(42.0)); // Invalid value for Array
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_invalid_object) {
    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("obj")));
    obj.emplace("v", score::json::Any(42.0)); // Invalid value for Object
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_array_with_invalid_element) {
    score::json::List list;

    score::json::Object inner_obj1;
    inner_obj1.emplace("t", score::json::Any(std::string("bool")));
    inner_obj1.emplace("v", score::json::Any(true));
    list.push_back(score::json::Any(std::move(inner_obj1)));

    score::json::Object inner_obj2;
    inner_obj2.emplace("t", score::json::Any(std::string("InvalidType")));
    inner_obj2.emplace("v", score::json::Any(std::string("test")));
    list.push_back(score::json::Any(std::move(inner_obj2)));

    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("arr")));
    obj.emplace("v", score::json::Any(std::move(list)));
    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);
}

TEST(kvs_any_to_kvsvalue, any_to_kvsvalue_object_with_invalid_value) {
    score::json::Object inner_obj1;
    inner_obj1.emplace("t", score::json::Any(std::string("bool")));
    inner_obj1.emplace("v", score::json::Any(true));

    score::json::Object inner_obj2;
    inner_obj2.emplace("t", score::json::Any(std::string("InvalidType")));
    inner_obj2.emplace("v", score::json::Any(42.0));

    score::json::Object value_obj;
    value_obj.emplace("flag", score::json::Any(std::move(inner_obj1)));
    value_obj.emplace("count", score::json::Any(std::move(inner_obj2)));

    score::json::Object obj;
    obj.emplace("t", score::json::Any(std::string("obj")));
    obj.emplace("v", score::json::Any(std::move(value_obj)));

    score::json::Any any_obj(std::move(obj));
    auto result = any_to_kvsvalue(any_obj);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);
}

/* Mock KvsValue for testing purposes (kvsvalue_to_any) */
class BrokenKvsValue : public KvsValue {
public:
    BrokenKvsValue() : KvsValue(nullptr) {
        /* Intentionally break the type by assigning an invalid value */
        *(Type*)&this->type = static_cast<Type>(999);
    }
};

TEST(kvs_kvsvalue_to_any, kvsvalue_to_any_null) {
    KvsValue null_val(nullptr);
    auto result = kvsvalue_to_any(null_val);
    ASSERT_TRUE(result);
    const auto& obj = result.value().As<score::json::Object>().value().get();
    EXPECT_EQ(obj.at("t").As<std::string>().value().get(), "null");
    EXPECT_TRUE(obj.at("v").As<score::json::Null>().has_value());
}

TEST(kvs_kvsvalue_to_any, kvsvalue_to_any_boolean) {
    KvsValue bool_val(true);
    auto result = kvsvalue_to_any(bool_val);
    ASSERT_TRUE(result);
    const auto& obj = result.value().As<score::json::Object>().value().get();
    EXPECT_EQ(obj.at("t").As<std::string>().value().get(), "bool");
    EXPECT_EQ(obj.at("v").As<bool>().value(), true);
}

TEST(kvs_kvsvalue_to_any, kvsvalue_to_any_i32) {
    KvsValue i32_val(static_cast<int32_t>(42));
    auto result = kvsvalue_to_any(i32_val);
    ASSERT_TRUE(result);
    const auto& obj = result.value().As<score::json::Object>().value().get();
    EXPECT_EQ(obj.at("t").As<std::string>().value().get(), "i32");
    EXPECT_EQ(obj.at("v").As<int32_t>().value(), 42.0);
}

TEST(kvs_kvsvalue_to_any, kvsvalue_to_any_u32) {
    KvsValue u32_val(static_cast<uint32_t>(42));
    auto result = kvsvalue_to_any(u32_val);
    ASSERT_TRUE(result);
    const auto& obj = result.value().As<score::json::Object>().value().get();
    EXPECT_EQ(obj.at("t").As<std::string>().value().get(), "u32");
    EXPECT_EQ(obj.at("v").As<uint32_t>().value(), 42.0);
}

TEST(kvs_kvsvalue_to_any, kvsvalue_to_any_i64) {
    KvsValue i64_val(static_cast<int64_t>(42));
    auto result = kvsvalue_to_any(i64_val);
    ASSERT_TRUE(result);
    const auto& obj = result.value().As<score::json::Object>().value().get();
    EXPECT_EQ(obj.at("t").As<std::string>().value().get(), "i64");
    EXPECT_EQ(obj.at("v").As<int64_t>().value(), 42.0);
}

TEST(kvs_kvsvalue_to_any, kvsvalue_to_any_u64) {
    KvsValue u64_val(static_cast<uint64_t>(42));
    auto result = kvsvalue_to_any(u64_val);
    ASSERT_TRUE(result);
    const auto& obj = result.value().As<score::json::Object>().value().get();
    EXPECT_EQ(obj.at("t").As<std::string>().value().get(), "u64");
    EXPECT_EQ(obj.at("v").As<uint64_t>().value(), 42.0);
}

TEST(kvs_kvsvalue_to_any, kvsvalue_to_any_f64) {
    KvsValue f64_val(42.0);
    auto result = kvsvalue_to_any(f64_val);
    ASSERT_TRUE(result);
    const auto& obj = result.value().As<score::json::Object>().value().get();
    EXPECT_EQ(obj.at("t").As<std::string>().value().get(), "f64");
    EXPECT_EQ(obj.at("v").As<double>().value(), 42.0);
}

TEST(kvs_kvsvalue_to_any, kvsvalue_to_any_string) {
    std::string str = "test";
    KvsValue string_val(str);
    auto result = kvsvalue_to_any(string_val);
    ASSERT_TRUE(result);
    const auto& obj = result.value().As<score::json::Object>().value().get();
    EXPECT_EQ(obj.at("t").As<std::string>().value().get(), "str");
    EXPECT_EQ(obj.at("v").As<std::string>().value().get(), "test");
}

TEST(kvs_kvsvalue_to_any, kvsvalue_to_any_array) {
    KvsValue::Array array;
    array.push_back(KvsValue(true));
    array.push_back(KvsValue(1.1));
    array.push_back(KvsValue(std::string("test")));
    KvsValue array_val(array);
    auto result = kvsvalue_to_any(array_val);
    ASSERT_TRUE(result);
    const auto& obj = result.value().As<score::json::Object>().value().get();
    EXPECT_EQ(obj.at("t").As<std::string>().value().get(), "arr");
    const auto& list = obj.at("v").As<score::json::List>().value().get();
    EXPECT_EQ(list.size(), 3);

    const auto& elem0 = list[0].As<score::json::Object>().value().get();
    EXPECT_EQ(elem0.at("t").As<std::string>().value().get(), "bool");
    EXPECT_EQ(elem0.at("v").As<bool>().value(), true);

    const auto& elem1 = list[1].As<score::json::Object>().value().get();
    EXPECT_EQ(elem1.at("t").As<std::string>().value().get(), "f64");
    EXPECT_EQ(elem1.at("v").As<double>().value(), 1.1);

    const auto& elem2 = list[2].As<score::json::Object>().value().get();
    EXPECT_EQ(elem2.at("t").As<std::string>().value().get(), "str");
    EXPECT_EQ(elem2.at("v").As<std::string>().value().get(), "test");
}

TEST(kvs_kvsvalue_to_any, kvsvalue_to_any_object) {
    KvsValue::Object obj;
    obj.emplace("flag", KvsValue(true)); // Boolean
    obj.emplace("count", KvsValue(42.0)); // F64
    KvsValue obj_val(obj);

    auto result = kvsvalue_to_any(obj_val);
    ASSERT_TRUE(result);

    const auto& outer_obj = result.value().As<score::json::Object>().value().get();
    EXPECT_EQ(outer_obj.at("t").As<std::string>().value().get(), "obj");

    const auto& inner_obj = outer_obj.at("v").As<score::json::Object>().value().get();

    const auto& flag_entry = inner_obj.at("flag").As<score::json::Object>().value().get();
    EXPECT_EQ(flag_entry.at("t").As<std::string>().value().get(), "bool");
    EXPECT_EQ(flag_entry.at("v").As<bool>().value(), true);

    const auto& count_entry = inner_obj.at("count").As<score::json::Object>().value().get();
    EXPECT_EQ(count_entry.at("t").As<std::string>().value().get(), "f64");
    EXPECT_EQ(count_entry.at("v").As<double>().value(), 42.0);
}

TEST(kvs_kvsvalue_to_any, kvsvalue_to_any_invalid) {
    /* InvalidType */
    BrokenKvsValue invalid;
    auto result = kvsvalue_to_any(invalid);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);

    /* Invalid values in array and object */
    KvsValue::Array array;
    array.push_back(KvsValue(42.0));
    array.push_back(invalid);
    KvsValue array_invalid(array);
    result = kvsvalue_to_any(array_invalid);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);

    KvsValue::Object obj;
    obj.emplace("valid", KvsValue(42.0));
    obj.emplace("invalid", invalid);
    KvsValue obj_invalid(obj);
    result = kvsvalue_to_any(obj_invalid);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType);
}

TEST(kvs_MessageFor, MessageFor) {
   struct {
        MyErrorCode code;
        std::string_view expected_message;
    } test_cases[] = {
        {MyErrorCode::UnmappedError,          "Error that was not yet mapped"},
        {MyErrorCode::FileNotFound,           "File not found"},
        {MyErrorCode::KvsFileReadError,       "KVS file read error"},
        {MyErrorCode::KvsHashFileReadError,   "KVS hash file read error"},
        {MyErrorCode::JsonParserError,        "JSON parser error"},
        {MyErrorCode::JsonGeneratorError,     "JSON generator error"},
        {MyErrorCode::PhysicalStorageFailure, "Physical storage failure"},
        {MyErrorCode::IntegrityCorrupted,     "Integrity corrupted"},
        {MyErrorCode::ValidationFailed,       "Validation failed"},
        {MyErrorCode::EncryptionFailed,       "Encryption failed"},
        {MyErrorCode::ResourceBusy,           "Resource is busy"},
        {MyErrorCode::OutOfStorageSpace,      "Out of storage space"},
        {MyErrorCode::QuotaExceeded,          "Quota exceeded"},
        {MyErrorCode::AuthenticationFailed,   "Authentication failed"},
        {MyErrorCode::KeyNotFound,            "Key not found"},
        {MyErrorCode::KeyDefaultNotFound,     "Key default value not found"},
        {MyErrorCode::SerializationFailed,    "Serialization failed"},
        {MyErrorCode::InvalidSnapshotId,      "Invalid snapshot ID"},
        {MyErrorCode::ConversionFailed,       "Conversion failed"},
        {MyErrorCode::MutexLockFailed,        "Mutex failed"},
        {MyErrorCode::InvalidValueType,       "Invalid value type"},
    };
    for (const auto& test : test_cases) {
        SCOPED_TRACE(static_cast<int>(test.code));
        score::result::ErrorCode code = static_cast<score::result::ErrorCode>(test.code);
        EXPECT_EQ(my_error_domain.MessageFor(code), test.expected_message);
    }

    score::result::ErrorCode invalid_code = static_cast<score::result::ErrorCode>(9999);
    EXPECT_EQ(my_error_domain.MessageFor(invalid_code), "Unknown Error!");

}

TEST(kvs_kvsbuilder, kvsbuilder_build) {
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
    EXPECT_EQ(static_cast<MyErrorCode>(*result_build.error()), MyErrorCode::KvsFileReadError); /* This error occurs in open_json and is passed through open()*/
    builder.need_defaults_flag(false);
    result_build = builder.build();
    ASSERT_FALSE(result_build);
    EXPECT_EQ(static_cast<MyErrorCode>(*result_build.error()), MyErrorCode::KvsFileReadError); /* This error occurs in open_json and is passed through open()*/
    builder.need_kvs_flag(false);
    result_build = builder.build();
    EXPECT_TRUE(result_build);
    result_build.value().flush_on_exit = false;
    EXPECT_EQ(result_build.value().filename_prefix.CStr(), "./kvsbuilder/kvs_"+std::to_string(instance_id.id));
}

TEST(kvs_kvsbuilder, kvsbuilder_directory_check) {

    /* Test the KvsBuilder with all configurations for the current working directory */    
    KvsBuilder builder(instance_id);
    builder.dir("");
    auto result_build = builder.build();
    EXPECT_TRUE(result_build);
    EXPECT_EQ(result_build.value().filename_prefix.CStr(), "./kvs_"+std::to_string(instance_id.id));

    builder.dir("./");
    result_build = builder.build();
    EXPECT_TRUE(result_build);
    EXPECT_EQ(result_build.value().filename_prefix.CStr(), "./kvs_"+std::to_string(instance_id.id));

    builder.dir(".");
    result_build = builder.build();
    EXPECT_TRUE(result_build);
    EXPECT_EQ(result_build.value().filename_prefix.CStr(), "./kvs_"+std::to_string(instance_id.id));

}

TEST(kvs_constructor, move_constructor) {
    /* Also checks set_flush_on_exit */
    const std::uint32_t instance_b = 5;
    
    /* create object A */
    auto result_a = Kvs::open(InstanceId(instance_b), OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(result_a);
    Kvs kvs_a = std::move(result_a.value());
    kvs_a.flush_on_exit = false;

    /* create object B */
    auto result_b = Kvs::open(instance_id , OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(result_b);
    Kvs kvs_b = std::move(result_b.value());
    kvs_a.flush_on_exit = true;

    /* Create Test Data*/
    kvs_b.kvs.insert({ "test_kvs", KvsValue(42.0) });
    kvs_b.default_values.insert({ "test_default", KvsValue(true) });

    /* Move assignment operator */
    kvs_a = std::move(kvs_b);

    /* Ignore Compiler Warning "Wself-move" for GCC and CLANG*/
    #if defined(__clang__) 
    #pragma clang diagnostic push
    #pragma clang diagnostic ignored "-Wself-move"
    #elif defined(__GNUC__)
    #pragma GCC diagnostic push
    #pragma GCC diagnostic ignored "-Wself-move"
    #endif
    kvs_a = std::move(kvs_a);  /* Intentional self-move */
    #if defined(__clang__)
    #pragma clang diagnostic pop
    #elif defined(__GNUC__)
    #pragma GCC diagnostic pop
    #endif

    
    /*Expectations:
    - kvs_a now contains data from the former kvs_b (default JSON data)
    - flush_on_exit of kvs_a should be true (was copied) -> flush_on_exit of kvs_b MUST be false */

    EXPECT_EQ(kvs_a.flush_on_exit, true);
    ASSERT_EQ(kvs_b.flush_on_exit, false);

    EXPECT_EQ(kvs_a.filename_prefix.CStr(), data_dir + "kvs_"+std::to_string(instance));

    EXPECT_TRUE(kvs_a.kvs.count("test_kvs"));
    EXPECT_TRUE(kvs_a.default_values.count("test_default"));

    const auto& val = kvs_a.kvs.at("test_kvs");
    EXPECT_EQ(val.getType(), KvsValue::Type::f64);
    EXPECT_EQ(std::get<double>(val.getValue()), 42.0);

    const auto& def = kvs_a.default_values.at("test_default");
    EXPECT_EQ(def.getType(), KvsValue::Type::Boolean);
    EXPECT_EQ(std::get<bool>(def.getValue()), true);

    EXPECT_TRUE(kvs_b.kvs.empty());
    EXPECT_TRUE(kvs_b.default_values.empty());

    kvs_a.flush_on_exit = false;
    cleanup_environment();
}

TEST(kvs_TEST, parse_json_data_sucess) {
    /* Success */
    prepare_environment();
    
    auto kvs = Kvs::open(InstanceId(instance_id), OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(kvs);  

    auto mock_parser = std::make_unique<score::json::IJsonParserMock>();
    score::json::Object obj;
    score::json::Object inner_obj;
    inner_obj.emplace("t", score::json::Any(std::string("i32")));
    inner_obj.emplace("v", score::json::Any(42));
    obj.emplace("kvs", score::json::Any(std::move(inner_obj)));
    score::json::Any any_obj(std::move(obj));

    EXPECT_CALL(*mock_parser, FromBuffer(::testing::_))
        .WillOnce(::testing::Return(score::Result<score::json::Any>(std::move(any_obj))));

    kvs->parser = std::move(mock_parser);

    auto result = kvs->parse_json_data("data_not_used_in_mocking");
    EXPECT_TRUE(result);

    cleanup_environment();
}

TEST(kvs_TEST, parse_json_data_failure) {

    prepare_environment();

    /* Json Parser Failure */

    auto kvs = Kvs::open(InstanceId(instance_id), OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(kvs);  

    auto mock_parser = std::make_unique<score::json::IJsonParserMock>();
    EXPECT_CALL(*mock_parser, FromBuffer(::testing::_))
        .WillOnce(::testing::Return(score::Result<score::json::Any>(score::MakeUnexpected(score::json::Error::kInvalidFilePath))));

    kvs->parser = std::move(mock_parser);

    auto result = kvs->parse_json_data("data_not_used_in_mocking");
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::JsonParserError);


    /* No Object returned Failure */

    mock_parser = std::make_unique<score::json::IJsonParserMock>();
    score::json::Any JsonParserReturnValue(42.0);

    EXPECT_CALL(*mock_parser, FromBuffer(::testing::_))
        .WillOnce(::testing::Return(score::Result<score::json::Any>(std::move(JsonParserReturnValue))));

    kvs->parser = std::move(mock_parser);

    result = kvs->parse_json_data("data_not_used_in_mocking");
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::JsonParserError);


    /* any_to_kvsvalue error */

    mock_parser = std::make_unique<score::json::IJsonParserMock>();
    score::json::Object obj;
    score::json::Object inner_obj;
    inner_obj.emplace("t", score::json::Any(std::string("invalid")));
    inner_obj.emplace("v", score::json::Any(42));
    obj.emplace("kvs", score::json::Any(std::move(inner_obj)));
    score::json::Any any_obj(std::move(obj));

    EXPECT_CALL(*mock_parser, FromBuffer(::testing::_))
        .WillOnce(::testing::Return(score::Result<score::json::Any>(std::move(any_obj))));

    kvs->parser = std::move(mock_parser);

    result = kvs->parse_json_data("data_not_used_in_mocking");
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), MyErrorCode::InvalidValueType); //error is passed from any_to_kvsvalue

    cleanup_environment();
}

TEST(kvs_open_json, open_json_success) {
    
    prepare_environment();

    auto kvs = Kvs::open(InstanceId(instance_id), OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(kvs);

    auto result = kvs->open_json(score::filesystem::Path(kvs_prefix), OpenJsonNeedFile::Required);
    EXPECT_TRUE(result);
    result = kvs->open_json(score::filesystem::Path(kvs_prefix), OpenJsonNeedFile::Optional);
    EXPECT_TRUE(result);

    cleanup_environment();
}

TEST(kvs_open_json, open_json_json_invalid) {
    
    prepare_environment();
    /* JSON Data invalid (parse_json_data Failure) */
    std::string invalid_json = "{ invalid json }";
    std::ofstream invalid_json_file(kvs_prefix + ".json");
    invalid_json_file << invalid_json;
    invalid_json_file.close();

    uint32_t kvs_hash = adler32(invalid_json); /* Still valid Hash*/
    std::ofstream kvs_hash_file(kvs_prefix + ".hash", std::ios::binary);
    kvs_hash_file.put((kvs_hash >> 24) & 0xFF);
    kvs_hash_file.put((kvs_hash >> 16) & 0xFF);
    kvs_hash_file.put((kvs_hash >> 8)  & 0xFF);
    kvs_hash_file.put(kvs_hash & 0xFF);
    kvs_hash_file.close();

    Kvs kvs = Kvs(); /* Create Kvs instance without any data (this constructor is normally private) */

    /* Make JSON Parser Fail */
    auto mock_parser = std::make_unique<score::json::IJsonParserMock>();
    EXPECT_CALL(*mock_parser, FromBuffer(::testing::_))
        .WillRepeatedly([](auto) {
            return score::Result<score::json::Any>(score::MakeUnexpected(score::json::Error::kInvalidFilePath));
        });
    kvs.parser = std::move(mock_parser);

    auto result = kvs.open_json(score::filesystem::Path(kvs_prefix), OpenJsonNeedFile::Required);
    ASSERT_FALSE(result);
    EXPECT_EQ(static_cast<MyErrorCode>(*result.error()), MyErrorCode::JsonParserError); /* Errorcode passed by parse json function*/

    /* JSON not existing */
    system(("rm -rf " + kvs_prefix + ".json").c_str());
    result = kvs.open_json(score::filesystem::Path(kvs_prefix), OpenJsonNeedFile::Required);
    ASSERT_FALSE(result);
    EXPECT_EQ(static_cast<MyErrorCode>(*result.error()), MyErrorCode::KvsFileReadError);

    cleanup_environment();
}

TEST(kvs_open_json, open_json_hash_invalid) {
    
    prepare_environment();
    /* Hash corrupted */
    std::fstream corrupt_default_hash_file(kvs_prefix + ".hash", std::ios::in | std::ios::out | std::ios::binary);
    corrupt_default_hash_file.seekp(0);
    corrupt_default_hash_file.put(0xFF);
    corrupt_default_hash_file.close();

    Kvs kvs = Kvs(); /* Create Kvs instance without any data (this constructor is normally private) */

    auto result = kvs.open_json(score::filesystem::Path(kvs_prefix), OpenJsonNeedFile::Optional);
    ASSERT_FALSE(result);
    EXPECT_EQ(static_cast<MyErrorCode>(*result.error()), MyErrorCode::ValidationFailed);

    /* Hash not existing */
    system(("rm -rf " + kvs_prefix + ".hash").c_str());
    result = kvs.open_json(score::filesystem::Path(kvs_prefix), OpenJsonNeedFile::Optional);
    ASSERT_FALSE(result);
    EXPECT_EQ(static_cast<MyErrorCode>(*result.error()), MyErrorCode::KvsHashFileReadError);

    /* JSON not existing */
    system(("rm -rf " + kvs_prefix + ".json").c_str());
    result = kvs.open_json(score::filesystem::Path(kvs_prefix), OpenJsonNeedFile::Required);
    ASSERT_FALSE(result);
    EXPECT_EQ(static_cast<MyErrorCode>(*result.error()), MyErrorCode::KvsFileReadError);

    cleanup_environment();
}

TEST(kvs_set_flush_on_exit, set_flush_on_exit) {
   
    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = true;
    result.value().set_flush_on_exit(false);
    ASSERT_FALSE(result.value().flush_on_exit);
    result.value().set_flush_on_exit(true);
    ASSERT_TRUE(result.value().flush_on_exit);

    result.value().flush_on_exit = false;

    cleanup_environment();

}

TEST(kvs_reset, reset_success){
    
    prepare_environment();
    auto result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    /* Check Data existing */
    EXPECT_FALSE(result.value().kvs.empty());

    /* Reset KVS */
    auto reset_result = result.value().reset();
    EXPECT_TRUE(reset_result);
    EXPECT_TRUE(result.value().kvs.empty());

    cleanup_environment();
}

TEST(kvs_reset, reset_failure){
    
    prepare_environment();

    /* Mutex locked */
    auto result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    std::unique_lock<std::mutex> lock(result.value().kvs_mutex);
    auto reset_result = result.value().reset();
    EXPECT_FALSE(reset_result);
    EXPECT_EQ(static_cast<MyErrorCode>(*reset_result.error()), MyErrorCode::MutexLockFailed);

    cleanup_environment();
}

TEST(kvs_get_all_keys, get_all_keys_success){
    
    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    /* Check Data existing */
    EXPECT_FALSE(result.value().kvs.empty());
    
    /* Check if all keys are returned */
    auto get_all_keys_result = result.value().get_all_keys();
    ASSERT_TRUE(get_all_keys_result);
    EXPECT_FALSE(get_all_keys_result.value().empty());

    auto search = std::find(get_all_keys_result.value().begin(), get_all_keys_result.value().end(), "kvs");
    EXPECT_TRUE(search != get_all_keys_result.value().end());

    /* Check if empty keys are returned */
    result.value().kvs.clear();
    get_all_keys_result = result.value().get_all_keys();
    EXPECT_TRUE(get_all_keys_result.value().empty());

    cleanup_environment();
}

TEST(kvs_get_all_keys, get_all_keys_failure){
   
    prepare_environment();

    /* Mutex locked */
    auto result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;
    std::unique_lock<std::mutex> lock(result.value().kvs_mutex);

    auto get_all_keys_result = result.value().get_all_keys();
    EXPECT_FALSE(get_all_keys_result);
    EXPECT_EQ(static_cast<MyErrorCode>(*get_all_keys_result.error()), MyErrorCode::MutexLockFailed);

    cleanup_environment();
}

TEST(kvs_key_exists, key_exists_success){
    
    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    /* Check Data existing */
    EXPECT_FALSE(result.value().kvs.empty());
    
    /* Check if key exists */
    auto exists_result = result.value().key_exists("kvs");
    EXPECT_TRUE(exists_result);
    EXPECT_TRUE(exists_result.value());
    /* Check if non-existing key returns false */
    exists_result = result.value().key_exists("non_existing_key");
    EXPECT_TRUE(exists_result);
    EXPECT_FALSE(exists_result.value());

    cleanup_environment();
}

TEST(kvs_key_exists, key_exists_failure){

    prepare_environment();

    /* Mutex locked */
    auto result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    std::unique_lock<std::mutex> lock(result.value().kvs_mutex);
    auto exists_result = result.value().key_exists("kvs");
    EXPECT_FALSE(exists_result);
    EXPECT_EQ(static_cast<MyErrorCode>(*exists_result.error()), MyErrorCode::MutexLockFailed);

    cleanup_environment();
}

TEST(kvs_get_value, get_value_success){
    
    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    /* Check Data existing */
    EXPECT_FALSE(result.value().kvs.empty());
    
    /* Check if value is returned */
    auto get_value_result = result.value().get_value("kvs");
    ASSERT_TRUE(get_value_result);
    EXPECT_EQ(get_value_result.value().getType(), KvsValue::Type::i32);
    EXPECT_EQ(std::get<int32_t>(get_value_result.value().getValue()), 2);

    /* Check if default value is returned when no written key exists */
    result.value().kvs.clear();
    ASSERT_TRUE(result.value().kvs.empty()); // Make sure kvs is empty and it uses the default value
    int32_t default_value(42);
    result.value().default_values.insert_or_assign(
        "kvs", 
        KvsValue(default_value)
    );
    get_value_result = result.value().get_value("kvs");
    ASSERT_TRUE(get_value_result);
    EXPECT_EQ(get_value_result.value().getType(), KvsValue::Type::i32);
    EXPECT_EQ(std::get<int32_t>(get_value_result.value().getValue()), 42);

    cleanup_environment();
}

TEST(kvs_get_value, get_value_failure){

    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    /* Check if non-existing key returns error */
    auto get_value_result = result.value().get_value("non_existing_key");
    EXPECT_FALSE(get_value_result);
    EXPECT_EQ(get_value_result.error(), MyErrorCode::KeyNotFound);

    /* Mutex locked */
    result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;
    std::unique_lock<std::mutex> lock(result.value().kvs_mutex);
    get_value_result = result.value().get_value("kvs");
    EXPECT_FALSE(get_value_result);
    EXPECT_EQ(static_cast<MyErrorCode>(*get_value_result.error()), MyErrorCode::MutexLockFailed);

    cleanup_environment();
}

TEST(kvs_get_default_value, get_default_value_success){
    
    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    /* Check Data existing */
    int32_t default_value(42);
    result.value().default_values.insert_or_assign(
        "kvs", 
        KvsValue(default_value)
    );

    /* Check if default value is returned */
    auto get_def_value_result = result.value().get_default_value("kvs");
    ASSERT_TRUE(get_def_value_result);
    EXPECT_EQ(get_def_value_result.value().getType(), KvsValue::Type::i32);
    EXPECT_EQ(std::get<int32_t>(get_def_value_result.value().getValue()), 42);

    cleanup_environment();
}

TEST(kvs_get_default_value, get_default_value_failure){
    
    prepare_environment();
    
    auto result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    /* Check if non-existing key returns error */
    auto get_def_value_result = result.value().get_default_value("non_existing_key");
    EXPECT_FALSE(get_def_value_result);
    EXPECT_EQ(get_def_value_result.error(), MyErrorCode::KeyNotFound);

    cleanup_environment();
}

TEST(kvs_reset_key, reset_key_success){
    
    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;
    ASSERT_TRUE(result.value().kvs.count("kvs")); /* Check Data existing */

    result.value().default_values.insert_or_assign( /* Create default Value for "kvs"-key */
        "kvs", 
        KvsValue(42.0)
    ); 
    /* Reset a key */
    auto reset_key_result = result.value().reset_key("kvs");
    EXPECT_TRUE(reset_key_result);
    EXPECT_FALSE(result.value().kvs.count("kvs"));
    EXPECT_TRUE(result.value().default_values.count("kvs"));

    /* Reset a non written key which has default value */
    result.value().default_values.insert_or_assign(
        "default", 
        KvsValue(42.0)
    ); 
    reset_key_result = result.value().reset_key("default");
    EXPECT_TRUE(reset_key_result);

    cleanup_environment();
}

TEST(kvs_reset_key, reset_key_failure){

    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    /* Reset a non-existing key */
    auto reset_key_result = result.value().reset_key("non_existing_key");
    EXPECT_FALSE(reset_key_result);
    EXPECT_EQ(reset_key_result.error(), MyErrorCode::KeyDefaultNotFound);

    /* Reset a key without default value */
    result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;
    result.value().default_values.clear(); // Clear default values to ensure no default value exists for "kvs"
    reset_key_result = result.value().reset_key("kvs");
    EXPECT_FALSE(reset_key_result);
    EXPECT_EQ(reset_key_result.error(), MyErrorCode::KeyDefaultNotFound);

    /* Mutex locked */
    result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;
    std::unique_lock<std::mutex> lock(result.value().kvs_mutex);
    reset_key_result = result.value().reset_key("kvs");
    EXPECT_FALSE(reset_key_result);
    EXPECT_EQ(static_cast<MyErrorCode>(*reset_key_result.error()), MyErrorCode::MutexLockFailed);

    cleanup_environment();
}

TEST(kvs_has_default_value, has_default_value){
    
    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    /* Create Test Default Data */
    result.value().default_values.insert_or_assign(
        "default", 
        KvsValue(42.0)
    ); 
    
    /* Check if default value exists */
    auto has_default_result = result.value().has_default_value("default");
    EXPECT_TRUE(has_default_result);
    EXPECT_TRUE(has_default_result.value());

    /* Check if non-existing key returns false */
    has_default_result = result.value().has_default_value("non_existing_key");
    EXPECT_TRUE(has_default_result);
    EXPECT_FALSE(has_default_result.value());
    
    cleanup_environment();
}

TEST(kvs_set_value, set_value_success){

    prepare_environment();
    auto result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;
    
    /* Set a new value */
    auto set_value_result = result.value().set_value("new_key", KvsValue(3.14));
    EXPECT_TRUE(set_value_result);
    EXPECT_TRUE(result.value().kvs.count("new_key"));
    EXPECT_EQ(result.value().kvs.at("new_key").getType(), KvsValue::Type::f64);
    EXPECT_DOUBLE_EQ(std::get<double>(result.value().kvs.at("new_key").getValue()), 3.14);

    /* Set a value with an existing key */
    set_value_result = result.value().set_value("kvs", KvsValue(2.718));
    EXPECT_TRUE(set_value_result);
    EXPECT_EQ(result.value().kvs.at("kvs").getType(), KvsValue::Type::f64);
    EXPECT_DOUBLE_EQ(std::get<double>(result.value().kvs.at("kvs").getValue()), 2.718);

    cleanup_environment();
}

TEST(kvs_set_value, set_value_failure){

    prepare_environment();

    /* Mutex locked */
    auto result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    std::unique_lock<std::mutex> lock(result.value().kvs_mutex);
    auto set_value_result = result.value().set_value("new_key", KvsValue(3.0));
    EXPECT_FALSE(set_value_result);
    EXPECT_EQ(static_cast<MyErrorCode>(*set_value_result.error()), MyErrorCode::MutexLockFailed);

    cleanup_environment();
}

TEST(kvs_remove_key, remove_key_success){
    
    prepare_environment();
    auto result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    /* Check Data existing */
    EXPECT_TRUE(result.value().kvs.count("kvs"));
    
    /* Remove an existing key */
    auto remove_key_result = result.value().remove_key("kvs");
    EXPECT_TRUE(remove_key_result);
    EXPECT_FALSE(result.value().kvs.count("kvs"));

    cleanup_environment();
}

TEST(kvs_remove_key, remove_key_failure){

    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    /* Remove a non-existing key */
    auto remove_key_result = result.value().remove_key("non_existing_key");
    EXPECT_FALSE(remove_key_result);
    EXPECT_EQ(remove_key_result.error(), MyErrorCode::KeyNotFound);

    /* Mutex locked */
    result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;
    std::unique_lock<std::mutex> lock(result.value().kvs_mutex);
    remove_key_result = result.value().remove_key("kvs");
    EXPECT_FALSE(remove_key_result);
    EXPECT_EQ(static_cast<MyErrorCode>(*remove_key_result.error()), MyErrorCode::MutexLockFailed);

    cleanup_environment();
}

TEST(kvs_write_json_data, write_json_data_success){
    
    prepare_environment();
    /* Test writing valid JSON data, also checks get_hash_bytes_adler32 and get_hash_bytes*/
    const char* json_test_data = R"({
        "booltest": {
            "t": "bool",
            "v": 1
        },
    })";
    system(("rm -rf " + kvs_prefix + ".json").c_str());
    system(("rm -rf " + kvs_prefix + ".hash").c_str());

    auto kvs = Kvs::open(InstanceId(instance_id), OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(kvs);

    kvs->filename_prefix = score::filesystem::Path(filename_prefix); /* Set the filename prefix to the test prefix */
    auto result = kvs->write_json_data(json_test_data);
    EXPECT_TRUE(result);
    EXPECT_TRUE(std::filesystem::exists(kvs_prefix + ".json"));
    EXPECT_TRUE(std::filesystem::exists(kvs_prefix + ".hash"));

    /* Check if the content and hash is correct */
    std::ifstream in_content(kvs_prefix + ".json", std::ios::binary);
    std::string file_content((std::istreambuf_iterator<char>(in_content)), std::istreambuf_iterator<char>());
    in_content.close();
    EXPECT_EQ(file_content, json_test_data);
    std::ifstream in_hash(kvs_prefix + ".hash", std::ios::binary);
    std::string hash_content((std::istreambuf_iterator<char>(in_hash)), std::istreambuf_iterator<char>());
    in_hash.close();
    uint32_t hash = adler32(json_test_data);
    std::array<uint8_t, 4> hash_bytes = {
        uint8_t((hash >> 24) & 0xFF),
        uint8_t((hash >> 16) & 0xFF),
        uint8_t((hash >>  8) & 0xFF),
        uint8_t((hash      ) & 0xFF)
    };
    std::string expected_hash_string(reinterpret_cast<const char*>(hash_bytes.data()), hash_bytes.size());
    EXPECT_EQ(hash_content, expected_hash_string);

    cleanup_environment();
}

TEST(kvs_write_json_data, write_json_data_filesystem_failure){

    prepare_environment();
    /* Test CreateDirectories fails */
    system(("rm -rf " + kvs_prefix + ".json").c_str());
    system(("rm -rf " + kvs_prefix + ".hash").c_str());

    auto kvs = Kvs::open(InstanceId(instance_id), OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(kvs);

    /* Mock Filesystem */
    score::filesystem::Filesystem mock_filesystem = score::filesystem::CreateMockFileSystem();
    auto standard_mock = std::dynamic_pointer_cast<score::filesystem::StandardFilesystemMock>(mock_filesystem.standard);
    ASSERT_NE(standard_mock, nullptr);
    EXPECT_CALL(*standard_mock, CreateDirectories(::testing::_))
        .WillOnce(::testing::Return(score::ResultBlank(score::MakeUnexpected(score::filesystem::ErrorCode::kCouldNotCreateDirectory))));
    kvs.value().filesystem = std::make_unique<score::filesystem::Filesystem>(std::move(mock_filesystem));

    auto result = kvs->write_json_data(kvs_json);
    EXPECT_FALSE(result);
    EXPECT_EQ(result.error(), MyErrorCode::PhysicalStorageFailure);

   
    /* Test if path argument is missing parent path (will only occur if semantic errors will be done in flush() )*/
    kvs->filename_prefix = score::filesystem::Path("no_parent_path"); /* Set the filename prefix to the test prefix */
    result = kvs->write_json_data(kvs_json);
    EXPECT_FALSE(result);
    EXPECT_EQ(result.error(), MyErrorCode::PhysicalStorageFailure);

    cleanup_environment();
}
TEST(kvs_write_json_data, write_json_data_permissions_failure){
    prepare_environment();
    /* Test CreateDirectories fails */
    system(("rm -rf " + kvs_prefix + ".json").c_str());
    system(("rm -rf " + kvs_prefix + ".hash").c_str());

    auto kvs = Kvs::open(InstanceId(instance_id), OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(kvs);

     /* Test writing to a non-writable hash file */
    std::ofstream out_hash(kvs_prefix + ".hash");
    out_hash << "data";
    out_hash.close();
    std::filesystem::permissions(kvs_prefix + ".hash", std::filesystem::perms::owner_read, std::filesystem::perm_options::replace);
    kvs->filename_prefix = score::filesystem::Path(filename_prefix); /* Reset the filename prefix to the test prefix */
    auto result = kvs->write_json_data(kvs_json);
    EXPECT_FALSE(result);
    EXPECT_EQ(result.error(), MyErrorCode::PhysicalStorageFailure);

    /* Test writing to a non-writable kvs file */
    std::ofstream out_json(kvs_prefix + ".json");
    out_json << "data";
    out_json.close();
    std::filesystem::permissions(kvs_prefix + ".json", std::filesystem::perms::owner_read, std::filesystem::perm_options::replace);
    kvs->filename_prefix = score::filesystem::Path(filename_prefix); /* Reset the filename prefix to the test prefix */
    result = kvs->write_json_data(kvs_json);
    EXPECT_FALSE(result);
    EXPECT_EQ(result.error(), MyErrorCode::PhysicalStorageFailure);

    cleanup_environment();

}

TEST(kvs_snapshot_rotate, snapshot_rotate_success){
    
    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;
    
    /* Create empty Test-Snapshot Files */
    for (size_t i = 1; i < KVS_MAX_SNAPSHOTS; i++) {
        std::ofstream(filename_prefix + "_" + std::to_string(i) + ".json") << "{}";
        std::ofstream(filename_prefix + "_" + std::to_string(i) + ".hash") << "{}";
        EXPECT_EQ(result.value().snapshot_count().value(), i);
    }
    ASSERT_FALSE(std::filesystem::exists(filename_prefix + "_" + std::to_string(KVS_MAX_SNAPSHOTS) + ".json"));
    ASSERT_FALSE(std::filesystem::exists(filename_prefix + "_" + std::to_string(KVS_MAX_SNAPSHOTS) + ".hash"));
    
    /* Rotate Snapshots */
    auto rotate_result = result.value().snapshot_rotate();
    ASSERT_TRUE(rotate_result);
    
    /* Check if the snapshot ids are rotated and no ID 0 exists */
    EXPECT_TRUE(std::filesystem::exists(filename_prefix + "_" + std::to_string(KVS_MAX_SNAPSHOTS) + ".json"));
    EXPECT_TRUE(std::filesystem::exists(filename_prefix + "_" + std::to_string(KVS_MAX_SNAPSHOTS) + ".hash"));
    EXPECT_FALSE(std::filesystem::exists(filename_prefix + "_" + std::to_string(0) + ".json"));
    EXPECT_FALSE(std::filesystem::exists(filename_prefix + "_" + std::to_string(0) + ".hash"));

    cleanup_environment();
}

TEST(kvs_snapshot_rotate, snapshot_rotate_max_snapshots){

    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;
    
    /* Create empty Test-Snapshot Files */
    for (size_t i = 1; i < KVS_MAX_SNAPSHOTS; i++) {
        std::ofstream(filename_prefix + "_" + std::to_string(i) + ".json") << "{}";
        std::ofstream(filename_prefix + "_" + std::to_string(i) + ".hash") << "{}";
        EXPECT_EQ(result.value().snapshot_count().value(), i);
    }
    ASSERT_FALSE(std::filesystem::exists(filename_prefix + "_" + std::to_string(KVS_MAX_SNAPSHOTS) + ".json"));
    ASSERT_FALSE(std::filesystem::exists(filename_prefix + "_" + std::to_string(KVS_MAX_SNAPSHOTS) + ".hash"));

    /* Check if no ID higher than KVS_MAX_SNAPSHOTS exists */
    auto rotate_result = result.value().snapshot_rotate();
    ASSERT_TRUE(rotate_result);
    EXPECT_FALSE(std::filesystem::exists(filename_prefix + "_" + std::to_string(KVS_MAX_SNAPSHOTS + 1) + ".json"));
    EXPECT_FALSE(std::filesystem::exists(filename_prefix + "_" + std::to_string(KVS_MAX_SNAPSHOTS + 1) + ".hash"));

    cleanup_environment();
}
    
TEST(kvs_snapshot_rotate, snapshot_rotate_failure_renaming_json){

    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    /* Create empty Test-Snapshot Files */
    for (size_t i = 1; i < KVS_MAX_SNAPSHOTS; i++) {
        std::ofstream(filename_prefix + "_" + std::to_string(i) + ".json") << "{}";
        std::ofstream(filename_prefix + "_" + std::to_string(i) + ".hash") << "{}";
        EXPECT_EQ(result.value().snapshot_count().value(), i);
    }

    /* Snapshot (JSON) Renaming failed (Create directorys instead of json files to trigger rename error)*/
    std::filesystem::create_directory(filename_prefix + "_" + std::to_string(KVS_MAX_SNAPSHOTS) + ".json");
    auto rotate_result = result.value().snapshot_rotate();
    EXPECT_FALSE(rotate_result);
    EXPECT_EQ(static_cast<MyErrorCode>(*rotate_result.error()), MyErrorCode::PhysicalStorageFailure);

    cleanup_environment();
}

TEST(kvs_snapshot_rotate, snapshot_rotate_failure_renaming_hash){

    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    /* Create empty Test-Snapshot Files */
    for (size_t i = 1; i < KVS_MAX_SNAPSHOTS; i++) {
        std::ofstream(filename_prefix + "_" + std::to_string(i) + ".json") << "{}";
        std::ofstream(filename_prefix + "_" + std::to_string(i) + ".hash") << "{}";
        EXPECT_EQ(result.value().snapshot_count().value(), i);
    }

    /* Hash Renaming failed (Create directorys instead of json files to trigger rename error)*/
    std::filesystem::create_directory(filename_prefix + "_" + std::to_string(KVS_MAX_SNAPSHOTS) + ".hash");
    auto rotate_result = result.value().snapshot_rotate();
    EXPECT_FALSE(rotate_result);
    EXPECT_EQ(static_cast<MyErrorCode>(*rotate_result.error()), MyErrorCode::PhysicalStorageFailure);
    
    cleanup_environment();
}

TEST(kvs_snapshot_rotate, snapshot_rotate_failure_mutex){
    
    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    /* Mutex locked */
    std::unique_lock<std::mutex> lock(result.value().kvs_mutex);
    auto rotate_result = result.value().snapshot_rotate();
    EXPECT_FALSE(rotate_result);
    EXPECT_EQ(static_cast<MyErrorCode>(*rotate_result.error()), MyErrorCode::MutexLockFailed);

    cleanup_environment();
}

TEST(kvs_flush, flush_success_data){
    
    prepare_environment();
    /* Test flush with valid data */
    system(("rm -rf " + kvs_prefix + ".json").c_str());
    system(("rm -rf " + kvs_prefix + ".hash").c_str());

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    result.value().kvs.clear(); /* Clear KVS to ensure no data is written */
    std::string value = "value1";
    result.value().kvs.insert({"key1", KvsValue(value)});
    auto flush_result = result.value().flush();
    ASSERT_TRUE(flush_result);

    /* Check if files were created correctly */
    EXPECT_TRUE(std::filesystem::exists(kvs_prefix + ".json"));
    EXPECT_TRUE(std::filesystem::exists(kvs_prefix + ".hash"));
    EXPECT_FALSE(std::filesystem::exists(filename_prefix + "_1.json"));
    EXPECT_FALSE(std::filesystem::exists(filename_prefix + "_1.hash"));

    cleanup_environment();
}

TEST(kvs_flush, flush_success_snapshot_rotate){
    
    prepare_environment();

    /* Test flush with valid data */
    system(("rm -rf " + kvs_prefix + ".json").c_str());
    system(("rm -rf " + kvs_prefix + ".hash").c_str());

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;
    EXPECT_FALSE(std::filesystem::exists(filename_prefix + "_1.json"));
    EXPECT_FALSE(std::filesystem::exists(filename_prefix + "_1.hash"));

    result.value().flush(); /* Initial Flush -> SnapshotID 0 */

    /* Check if snapshot_rotate was triggered on second flush --> one snapshot should be available afterwards */
    auto flush_result = result.value().flush();
    ASSERT_TRUE(flush_result);
    EXPECT_TRUE(std::filesystem::exists(filename_prefix + "_1.json"));
    EXPECT_TRUE(std::filesystem::exists(filename_prefix + "_1.hash"));

    cleanup_environment();
}

TEST(kvs_flush, flush_failure_mutex){

    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    std::unique_lock<std::mutex> lock(result.value().kvs_mutex);
    auto flush_result = result.value().flush();
    EXPECT_FALSE(flush_result);
    EXPECT_EQ(static_cast<MyErrorCode>(*flush_result.error()), MyErrorCode::MutexLockFailed);

    cleanup_environment();
}

TEST(kvs_flush, flush_failure_rotate_snapshots){

    prepare_environment();
    /* Test Folder for permission handling */
    std::string permissions_dir = data_dir + "permissions/";
    std::filesystem::create_directories(permissions_dir);
    auto result = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(permissions_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    std::filesystem::permissions(permissions_dir, std::filesystem::perms::owner_read, std::filesystem::perm_options::replace);
    auto flush_result = result.value().flush();

    EXPECT_FALSE(flush_result);
    EXPECT_EQ(static_cast<MyErrorCode>(*flush_result.error()), MyErrorCode::PhysicalStorageFailure);

    cleanup_environment();
}

TEST(kvs_flush, flush_failure_kvsvalue_invalid){

    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    BrokenKvsValue invalid;
    result.value().kvs.insert({"invalid_key", invalid});

    auto flush_result_invalid = result.value().flush();
    EXPECT_FALSE(flush_result_invalid);
    EXPECT_EQ(flush_result_invalid.error(), MyErrorCode::InvalidValueType);

    cleanup_environment();
}

TEST(kvs_flush, flush_failure_json_writer){

    prepare_environment();

    auto kvs = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(kvs);
    kvs.value().flush_on_exit = false;

    auto mock_writer = std::make_unique<score::json::IJsonWriterMock>(); /* Force error in writer.ToBuffer */
    EXPECT_CALL(*mock_writer, ToBuffer(::testing::A<const score::json::Object&>()))
        .WillOnce(::testing::Return(score::Result<std::string>(score::MakeUnexpected(score::json::Error::kUnknownError))));

    kvs->writer = std::move(mock_writer);
    auto result = kvs.value().flush();
    EXPECT_FALSE(result);
    EXPECT_EQ(result.error(), MyErrorCode::JsonGeneratorError);

    cleanup_environment();
}

TEST(kvs_snapshot_count, snapshot_count_success){
    
    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    /* Create empty Test-Snapshot Files */
    for (size_t i = 1; i <= KVS_MAX_SNAPSHOTS; i++) {
        std::ofstream(filename_prefix + "_" + std::to_string(i) + ".json") << "{}";
        auto count = result.value().snapshot_count();
        EXPECT_TRUE(count);
        EXPECT_EQ(count.value(), i);
    }
    /* Test maximum capacity */
    std::ofstream(filename_prefix + "_" + std::to_string(KVS_MAX_SNAPSHOTS + 1) + ".json") << "{}";
    auto count = result.value().snapshot_count();
    EXPECT_TRUE(count);
    EXPECT_EQ(count.value(), KVS_MAX_SNAPSHOTS);

    cleanup_environment();
}

TEST(kvs_snapshot_count, snapshot_count_invalid){
    prepare_environment();

    auto kvs = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(kvs);
    kvs.value().flush_on_exit = false;

    /* Mock Filesystem */
    score::filesystem::Filesystem mock_filesystem = score::filesystem::CreateMockFileSystem();
    auto standard_mock = std::dynamic_pointer_cast<score::filesystem::StandardFilesystemMock>(mock_filesystem.standard);
    ASSERT_NE(standard_mock, nullptr);
    EXPECT_CALL(*standard_mock, Exists(::testing::_))
        .WillOnce(::testing::Return(score::Result<bool>(score::MakeUnexpected(score::filesystem::ErrorCode::kCouldNotRetrieveStatus))));
    kvs.value().filesystem = std::make_unique<score::filesystem::Filesystem>(std::move(mock_filesystem));

    auto result = kvs.value().snapshot_count();
    EXPECT_FALSE(result);
    EXPECT_EQ(static_cast<MyErrorCode>(*result.error()), MyErrorCode::PhysicalStorageFailure);
}


TEST(kvs_snapshot_restore, snapshot_restore_success){

    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    /* Create empty Test-Snapshot Files -> Data received by the JsonParser should be the data listed below */
    ASSERT_FALSE(result.value().kvs.empty());
    ASSERT_FALSE(result.value().kvs.count("key1"));
    ASSERT_TRUE(result.value().kvs.count("kvs"));

    /* Create dummy data */
    const std::string json_data = R"({
        "kvs_old": {
            "t": "i32",
            "v": 42
        }
    })";
    /* Create Hash Data for json_data*/
    std::ofstream out(filename_prefix + "_1.json", std::ios::binary);
    out.write(json_data.data(), json_data.size());
    out.close();
    uint32_t hash = adler32(json_data);
    std::array<uint8_t, 4> hash_bytes = {
        uint8_t((hash >> 24) & 0xFF),
        uint8_t((hash >> 16) & 0xFF),
        uint8_t((hash >>  8) & 0xFF),
        uint8_t((hash      ) & 0xFF)
    };
    std::ofstream hash_out(filename_prefix + "_1.hash", std::ios::binary);
    hash_out.write(reinterpret_cast<const char*>(hash_bytes.data()), hash_bytes.size());
    hash_out.close();

    auto restore_result = result.value().snapshot_restore(1);
    EXPECT_TRUE(restore_result);
    EXPECT_TRUE(result.value().kvs.count("kvs_old"));

    cleanup_environment();

}

TEST(kvs_snapshot_restore, snapshot_restore_failure_invalid_snapshot_id){

    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    /* Restore Snapshot ID 0 -> Current KVS*/
    auto restore_result = result.value().snapshot_restore(0);
    ASSERT_FALSE(restore_result);
    EXPECT_EQ(static_cast<MyErrorCode>(*restore_result.error()), MyErrorCode::InvalidSnapshotId);

    /* Restore Snapshot ID higher than snapshot_count (e.g. KVS_MAX_SNAPSHOTS +1) */
    restore_result = result.value().snapshot_restore(KVS_MAX_SNAPSHOTS + 1);
    ASSERT_FALSE(restore_result);
    EXPECT_EQ(static_cast<MyErrorCode>(*restore_result.error()), MyErrorCode::InvalidSnapshotId);

    cleanup_environment();
}

TEST(kvs_snapshot_restore, snapshot_restore_failure_open_json){

    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    /* Create empty Test-Snapshot Files */
    std::ofstream(filename_prefix + "_1.json") << "{}"; /* Empty JSON */
    std::ofstream(filename_prefix + "_1.hash") << "invalid_hash"; /* Invalid Hash -> Trigger open_json error */

    auto restore_result = result.value().snapshot_restore(1);
    EXPECT_FALSE(restore_result);
    EXPECT_EQ(static_cast<MyErrorCode>(*restore_result.error()), MyErrorCode::ValidationFailed); /* passed by open_json*/

    cleanup_environment();
}

TEST(kvs_snapshot_restore, snapshot_restore_failure_mutex){

    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    std::unique_lock<std::mutex> lock(result.value().kvs_mutex);
    auto restore_result = result.value().snapshot_restore(1);
    EXPECT_FALSE(restore_result);
    EXPECT_EQ(static_cast<MyErrorCode>(*restore_result.error()), MyErrorCode::MutexLockFailed);

    cleanup_environment();
}

TEST(kvs_snapshot_restore, snapshot_restore_failure_snapshot_count){

    prepare_environment();

    auto kvs = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(kvs);
    kvs.value().flush_on_exit = false;

    /* Mock Filesystem */
    score::filesystem::Filesystem mock_filesystem = score::filesystem::CreateMockFileSystem();
    auto standard_mock = std::dynamic_pointer_cast<score::filesystem::StandardFilesystemMock>(mock_filesystem.standard);
    ASSERT_NE(standard_mock, nullptr);
    EXPECT_CALL(*standard_mock, Exists(::testing::_))
        .WillOnce(::testing::Return(score::Result<bool>(score::MakeUnexpected(score::filesystem::ErrorCode::kCouldNotRetrieveStatus))));
    kvs.value().filesystem = std::make_unique<score::filesystem::Filesystem>(std::move(mock_filesystem));

    auto result = kvs.value().snapshot_restore(1);
    EXPECT_FALSE(result);

    cleanup_environment();
}

TEST(kvs_snapshot_max_count, snapshot_max_count){
    
    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(result);
    EXPECT_EQ(result.value().snapshot_max_count(), KVS_MAX_SNAPSHOTS);
    result.value().flush_on_exit = false;

    cleanup_environment();
}

TEST(kvs_get_filename, get_kvs_filename_success){
    
    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    /* Generate Testfiles */
    for (size_t i = 0; i < KVS_MAX_SNAPSHOTS; i++) {
        std::ofstream(filename_prefix + "_" + std::to_string(i) + ".json") << "{}";
    }

    for(int i = 0; i< KVS_MAX_SNAPSHOTS; i++) {
        auto filename = result.value().get_kvs_filename(SnapshotId(i));
        EXPECT_TRUE(filename);
        EXPECT_EQ(filename.value().CStr(), filename_prefix + "_" + std::to_string(i) + ".json");
    }
    
    cleanup_environment();
}

TEST(kvs_get_filename, get_kvs_filename_failure){
    
    prepare_environment();

    auto kvs = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(kvs);
    kvs.value().flush_on_exit = false;

    /* Testfiles not available */
    auto result = kvs.value().get_kvs_filename(SnapshotId(1));
    EXPECT_FALSE(result);
    EXPECT_EQ(static_cast<MyErrorCode>(*result.error()), MyErrorCode::FileNotFound);

    /* Filesystem exists error */
    /* Mock Filesystem */
    score::filesystem::Filesystem mock_filesystem = score::filesystem::CreateMockFileSystem();
    auto standard_mock = std::dynamic_pointer_cast<score::filesystem::StandardFilesystemMock>(mock_filesystem.standard);
    ASSERT_NE(standard_mock, nullptr);
    EXPECT_CALL(*standard_mock, Exists(::testing::_))
        .WillOnce(::testing::Return(score::Result<bool>(score::MakeUnexpected(score::filesystem::ErrorCode::kCouldNotRetrieveStatus))));
    kvs.value().filesystem = std::make_unique<score::filesystem::Filesystem>(std::move(mock_filesystem));

    result = kvs.value().get_kvs_filename(SnapshotId(1));
    EXPECT_FALSE(result);

    cleanup_environment();
}

TEST(kvs_get_filename, get_hashname_success){
    
    prepare_environment();

    auto result = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;

    /* Generate Testfiles */
    for (size_t i = 0; i < KVS_MAX_SNAPSHOTS; i++) {
        std::ofstream(filename_prefix + "_" + std::to_string(i) + ".hash") << "{}";
    }

    for(int i = 0; i< KVS_MAX_SNAPSHOTS; i++) {
        auto hashname = result.value().get_hash_filename(SnapshotId(i));
        system("ls -l ./data_folder/");
        EXPECT_TRUE(hashname);
        EXPECT_EQ(hashname.value().CStr(), filename_prefix + "_" + std::to_string(i) + ".hash");
    }
    
    cleanup_environment();
}

TEST(kvs_get_filename, get_hashname_failure){
    
    prepare_environment();

    auto kvs = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(kvs);
    kvs.value().flush_on_exit = false;

    /* Testfiles not available */
    auto result = kvs.value().get_hash_filename(SnapshotId(1));
    EXPECT_FALSE(result);
    EXPECT_EQ(static_cast<MyErrorCode>(*result.error()), MyErrorCode::FileNotFound);

    /* Filesystem exists error */
    /* Mock Filesystem */
    score::filesystem::Filesystem mock_filesystem = score::filesystem::CreateMockFileSystem();
    auto standard_mock = std::dynamic_pointer_cast<score::filesystem::StandardFilesystemMock>(mock_filesystem.standard);
    ASSERT_NE(standard_mock, nullptr);
    EXPECT_CALL(*standard_mock, Exists(::testing::_))
        .WillOnce(::testing::Return(score::Result<bool>(score::MakeUnexpected(score::filesystem::ErrorCode::kCouldNotRetrieveStatus))));
    kvs.value().filesystem = std::make_unique<score::filesystem::Filesystem>(std::move(mock_filesystem));

    result = kvs.value().get_hash_filename(SnapshotId(1));
    EXPECT_FALSE(result);

    cleanup_environment();
}



TEST(kvs_destructor, destructor) {

    prepare_environment();

    {
    auto result = Kvs::open(instance_id, OpenNeedDefaults::Optional, OpenNeedKvs::Optional, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = true;
    }
    /* Since flush and snapshot is already tested, we let flush() be called by the destructor and check if the snapshot files (ID=1) are created */
    EXPECT_TRUE(std::filesystem::exists(filename_prefix + "_1.json"));

    cleanup_environment();
}
