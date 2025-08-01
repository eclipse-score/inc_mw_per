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
    EXPECT_EQ(result.error(), ErrorCode::JsonParserError);


    /* No Object returned Failure */

    mock_parser = std::make_unique<score::json::IJsonParserMock>();
    score::json::Any JsonParserReturnValue(42.0);

    EXPECT_CALL(*mock_parser, FromBuffer(::testing::_))
        .WillOnce(::testing::Return(score::Result<score::json::Any>(std::move(JsonParserReturnValue))));

    kvs->parser = std::move(mock_parser);

    result = kvs->parse_json_data("data_not_used_in_mocking");
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), ErrorCode::JsonParserError);


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
    EXPECT_EQ(result.error(), ErrorCode::InvalidValueType); //error is passed from any_to_kvsvalue

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
    EXPECT_EQ(static_cast<ErrorCode>(*result.error()), ErrorCode::JsonParserError); /* Errorcode passed by parse json function*/

    /* JSON not existing */
    system(("rm -rf " + kvs_prefix + ".json").c_str());
    result = kvs.open_json(score::filesystem::Path(kvs_prefix), OpenJsonNeedFile::Required);
    ASSERT_FALSE(result);
    EXPECT_EQ(static_cast<ErrorCode>(*result.error()), ErrorCode::KvsFileReadError);

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
    EXPECT_EQ(static_cast<ErrorCode>(*result.error()), ErrorCode::ValidationFailed);

    /* Hash not existing */
    system(("rm -rf " + kvs_prefix + ".hash").c_str());
    result = kvs.open_json(score::filesystem::Path(kvs_prefix), OpenJsonNeedFile::Optional);
    ASSERT_FALSE(result);
    EXPECT_EQ(static_cast<ErrorCode>(*result.error()), ErrorCode::KvsHashFileReadError);

    /* JSON not existing */
    system(("rm -rf " + kvs_prefix + ".json").c_str());
    result = kvs.open_json(score::filesystem::Path(kvs_prefix), OpenJsonNeedFile::Required);
    ASSERT_FALSE(result);
    EXPECT_EQ(static_cast<ErrorCode>(*result.error()), ErrorCode::KvsFileReadError);

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
    EXPECT_EQ(static_cast<ErrorCode>(*reset_result.error()), ErrorCode::MutexLockFailed);

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
    EXPECT_EQ(static_cast<ErrorCode>(*get_all_keys_result.error()), ErrorCode::MutexLockFailed);

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
    EXPECT_EQ(static_cast<ErrorCode>(*exists_result.error()), ErrorCode::MutexLockFailed);

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
    EXPECT_EQ(get_value_result.error(), ErrorCode::KeyNotFound);

    /* Mutex locked */
    result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;
    std::unique_lock<std::mutex> lock(result.value().kvs_mutex);
    get_value_result = result.value().get_value("kvs");
    EXPECT_FALSE(get_value_result);
    EXPECT_EQ(static_cast<ErrorCode>(*get_value_result.error()), ErrorCode::MutexLockFailed);

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
    EXPECT_EQ(get_def_value_result.error(), ErrorCode::KeyNotFound);

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
    EXPECT_EQ(reset_key_result.error(), ErrorCode::KeyDefaultNotFound);

    /* Reset a key without default value */
    result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;
    result.value().default_values.clear(); // Clear default values to ensure no default value exists for "kvs"
    reset_key_result = result.value().reset_key("kvs");
    EXPECT_FALSE(reset_key_result);
    EXPECT_EQ(reset_key_result.error(), ErrorCode::KeyDefaultNotFound);

    /* Mutex locked */
    result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;
    std::unique_lock<std::mutex> lock(result.value().kvs_mutex);
    reset_key_result = result.value().reset_key("kvs");
    EXPECT_FALSE(reset_key_result);
    EXPECT_EQ(static_cast<ErrorCode>(*reset_key_result.error()), ErrorCode::MutexLockFailed);

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
    EXPECT_EQ(static_cast<ErrorCode>(*set_value_result.error()), ErrorCode::MutexLockFailed);

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
    EXPECT_EQ(remove_key_result.error(), ErrorCode::KeyNotFound);

    /* Mutex locked */
    result = Kvs::open(instance_id, OpenNeedDefaults::Required, OpenNeedKvs::Required, std::string(data_dir));
    ASSERT_TRUE(result);
    result.value().flush_on_exit = false;
    std::unique_lock<std::mutex> lock(result.value().kvs_mutex);
    remove_key_result = result.value().remove_key("kvs");
    EXPECT_FALSE(remove_key_result);
    EXPECT_EQ(static_cast<ErrorCode>(*remove_key_result.error()), ErrorCode::MutexLockFailed);

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
    EXPECT_EQ(result.error(), ErrorCode::PhysicalStorageFailure);

   
    /* Test if path argument is missing parent path (will only occur if semantic errors will be done in flush() )*/
    kvs->filename_prefix = score::filesystem::Path("no_parent_path"); /* Set the filename prefix to the test prefix */
    result = kvs->write_json_data(kvs_json);
    EXPECT_FALSE(result);
    EXPECT_EQ(result.error(), ErrorCode::PhysicalStorageFailure);

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
    EXPECT_EQ(result.error(), ErrorCode::PhysicalStorageFailure);

    /* Test writing to a non-writable kvs file */
    std::ofstream out_json(kvs_prefix + ".json");
    out_json << "data";
    out_json.close();
    std::filesystem::permissions(kvs_prefix + ".json", std::filesystem::perms::owner_read, std::filesystem::perm_options::replace);
    kvs->filename_prefix = score::filesystem::Path(filename_prefix); /* Reset the filename prefix to the test prefix */
    result = kvs->write_json_data(kvs_json);
    EXPECT_FALSE(result);
    EXPECT_EQ(result.error(), ErrorCode::PhysicalStorageFailure);

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
    EXPECT_EQ(static_cast<ErrorCode>(*rotate_result.error()), ErrorCode::PhysicalStorageFailure);

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
    EXPECT_EQ(static_cast<ErrorCode>(*rotate_result.error()), ErrorCode::PhysicalStorageFailure);
    
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
    EXPECT_EQ(static_cast<ErrorCode>(*rotate_result.error()), ErrorCode::MutexLockFailed);

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
    EXPECT_EQ(static_cast<ErrorCode>(*flush_result.error()), ErrorCode::MutexLockFailed);

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
    EXPECT_EQ(static_cast<ErrorCode>(*flush_result.error()), ErrorCode::PhysicalStorageFailure);

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
    EXPECT_EQ(flush_result_invalid.error(), ErrorCode::InvalidValueType);

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
    EXPECT_EQ(result.error(), ErrorCode::JsonGeneratorError);

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
    EXPECT_EQ(static_cast<ErrorCode>(*result.error()), ErrorCode::PhysicalStorageFailure);
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
    EXPECT_EQ(static_cast<ErrorCode>(*restore_result.error()), ErrorCode::InvalidSnapshotId);

    /* Restore Snapshot ID higher than snapshot_count (e.g. KVS_MAX_SNAPSHOTS +1) */
    restore_result = result.value().snapshot_restore(KVS_MAX_SNAPSHOTS + 1);
    ASSERT_FALSE(restore_result);
    EXPECT_EQ(static_cast<ErrorCode>(*restore_result.error()), ErrorCode::InvalidSnapshotId);

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
    EXPECT_EQ(static_cast<ErrorCode>(*restore_result.error()), ErrorCode::ValidationFailed); /* passed by open_json*/

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
    EXPECT_EQ(static_cast<ErrorCode>(*restore_result.error()), ErrorCode::MutexLockFailed);

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
    EXPECT_EQ(static_cast<ErrorCode>(*result.error()), ErrorCode::FileNotFound);

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
    EXPECT_EQ(static_cast<ErrorCode>(*result.error()), ErrorCode::FileNotFound);

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
