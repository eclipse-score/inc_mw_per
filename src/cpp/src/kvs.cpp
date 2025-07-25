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

#include <fstream>
#include <iostream>
#include <sstream>
#include "kvs.hpp"
#include "internal/kvs_helper.hpp"


//TODO Default Value Handling TBD
//TODO Add Score Logging
//TODO Replace std libs with baselibs
//TODO String Handling in set_value TBD: (e.g. set_value("key", "value") is not correct, since KvsValue() expects a string ptr and not a string)

using namespace std;

/*********************** Hash Functions *********************/
/*Adler 32 checksum algorithm*/ 
// Optimized version: processes data in blocks to reduce modulo operations
uint32_t calculate_hash_adler32(const string& data) {
    constexpr size_t ADLER32_NMAX = 5552;
    constexpr uint32_t ADLER32_BASE = 65521;
    uint32_t a = 1, b = 0;
    size_t len = data.size();
    size_t i = 0;

    // Process in blocks of 5552 bytes (as recommended for Adler-32)
    while (len > 0) {
        size_t tlen = len > ADLER32_NMAX ? ADLER32_NMAX : len;
        len -= tlen;
        for (size_t j = 0; j < tlen; ++j, ++i) {
            a += static_cast<unsigned char>(data[i]);
            b += a;
        }
        a %= ADLER32_BASE;
        b %= ADLER32_BASE;
    }
    return (b << 16) | a;
}

/*Parse Adler32 checksum Byte-Array to uint32 */ 
uint32_t parse_hash_adler32(std::istream& in)
{
    std::array<uint8_t,4> buf{};
    in.read(reinterpret_cast<char*>(buf.data()), buf.size());

    uint32_t value = (uint32_t(buf[0]) << 24)
                   | (uint32_t(buf[1]) << 16)
                   | (uint32_t(buf[2]) <<  8)
                   |  uint32_t(buf[3]);
    return value;
}

/* Split uint32 checksum in bytes for writing*/ 
std::array<uint8_t,4> get_hash_bytes_adler32(uint32_t hash)
{
    std::array<uint8_t, 4> value = {
        uint8_t((hash >> 24) & 0xFF),
        uint8_t((hash >> 16) & 0xFF),
        uint8_t((hash >>  8) & 0xFF),
        uint8_t((hash      ) & 0xFF)
    };
    return value;
}

/***** Wrapper Functions for Hash *****/
/* Wrapper Functions should isolate the Hash Algorithm, so that the algorithm can be easier replaced*/

/* Wrapper Function to get checksum in bytes*/
std::array<uint8_t,4> get_hash_bytes(const string& data)
{
    uint32_t hash = calculate_hash_adler32(data);
    std::array<uint8_t, 4> value = get_hash_bytes_adler32(hash);
    return value;
}

/* Wrapper Function to check, if Hash is valid*/
bool check_hash(const string& data_calculate, std::istream& data_parse){
    bool result;
    uint32_t calculated_hash = calculate_hash_adler32(data_calculate);
    uint32_t parsed_hash = parse_hash_adler32(data_parse);
    if(calculated_hash == parsed_hash){
        result = true;
    }else{
        result = false;
    }

    return result; 
}

/*********************** Standalone Helper Functions *********************/

/* Helper Function for Any -> KVSValue conversion */
score::Result<KvsValue> any_to_kvsvalue(const score::json::Any& any){
    score::Result<KvsValue> result = score::MakeUnexpected(MyErrorCode::UnmappedError);
    if (auto o = any.As<score::json::Object>(); o.has_value()) {
        const auto& objAny = o.value().get();
        auto type  = objAny.find("type");
        auto value = objAny.find("value");
        if (type != objAny.end() && value != objAny.end()) {
            if (auto typeStr = type->second.As<std::string>(); typeStr.has_value()) {
                const std::string_view typeStrV = typeStr.value().get();
                const score::json::Any& valueAny = value->second;

                if (typeStrV == "I32") {
                    if (auto n = valueAny.As<int32_t>(); n.has_value()){
                        result = KvsValue(static_cast<int32_t>(n.value()));
                    }
                    else {
                        result = score::MakeUnexpected(MyErrorCode::InvalidValueType);
                    }
                }
                else if (typeStrV == "U32") {
                    if (auto n = valueAny.As<uint32_t>(); n.has_value()) {
                        result = KvsValue(static_cast<uint32_t>(n.value()));
                    }
                    else {
                        result = score::MakeUnexpected(MyErrorCode::InvalidValueType);
                    }
                }
                else if (typeStrV == "I64") {
                    if (auto n = valueAny.As<int64_t>(); n.has_value()) {
                        result = KvsValue(static_cast<int64_t>(n.value()));
                    }
                    else {
                        result = score::MakeUnexpected(MyErrorCode::InvalidValueType);
                    }
                }
                else if (typeStrV == "U64") {
                    if (auto n = valueAny.As<uint64_t>(); n.has_value()) {
                        result = KvsValue(static_cast<uint64_t>(n.value()));
                    }
                    else {
                        result = score::MakeUnexpected(MyErrorCode::InvalidValueType);
                    }
                }
                else if (typeStrV == "F64") {
                    if (auto n = valueAny.As<double>(); n.has_value()) {
                        result = KvsValue(n.value());
                    }
                    else {
                        result = score::MakeUnexpected(MyErrorCode::InvalidValueType);
                    }
                }
                else if (typeStrV == "Boolean") {
                    if (auto b = valueAny.As<bool>(); b.has_value()) {
                        result = KvsValue(b.value());
                    }
                    else {
                        result = score::MakeUnexpected(MyErrorCode::InvalidValueType);
                    }
                }
                else if (typeStrV == "String") {
                    if (auto s = valueAny.As<std::string>(); s.has_value()) {
                        result = KvsValue(s.value().get());
                    }
                    else {
                        result = score::MakeUnexpected(MyErrorCode::InvalidValueType);
                    }
                }
                else if (typeStrV == "Null") {
                    if (valueAny.As<score::json::Null>().has_value()) {
                        result = KvsValue(nullptr);
                    }
                    else {
                        result = score::MakeUnexpected(MyErrorCode::InvalidValueType);
                    }
                }
                else if (typeStrV == "Array") {
                    if (auto l = valueAny.As<score::json::List>(); l.has_value()) {
                        KvsValue::Array arr;
                        bool error = false;
                        for (auto const& elem : l.value().get()) {
                            auto conv = any_to_kvsvalue(elem);
                            if (!conv) {
                                error = true;
                                result = score::MakeUnexpected(MyErrorCode::InvalidValueType);
                                break;
                            }
                            arr.emplace_back(std::move(conv.value()));
                        }
                        if (!error){
                            result = KvsValue(std::move(arr));
                        }
                    } else {
                        result = score::MakeUnexpected(MyErrorCode::InvalidValueType);
                    }
                }
                else if (typeStrV == "Object") {
                    if (auto obj = valueAny.As<score::json::Object>(); obj.has_value()) {
                        KvsValue::Object map;
                        bool error = false;
                        for (auto const& [key, valAny] : obj.value().get()) {
                            auto conv = any_to_kvsvalue(valAny);
                            if (!conv) {
                                error = true;
                                result = score::MakeUnexpected(MyErrorCode::InvalidValueType);
                                break;
                            }
                            map.emplace(key.GetAsStringView().to_string(), std::move(conv.value()));
                        }
                        if (!error) {
                            result = KvsValue(std::move(map));
                        }
                    } else {
                        result = score::MakeUnexpected(MyErrorCode::InvalidValueType);
                    }
                } else {
                    result = score::MakeUnexpected(MyErrorCode::InvalidValueType);
                }
            } else {
                result = score::MakeUnexpected(MyErrorCode::InvalidValueType);
            }
        } else {
            result = score::MakeUnexpected(MyErrorCode::InvalidValueType);
        }
    } else {
        result = score::MakeUnexpected(MyErrorCode::InvalidValueType);
    }

    return result;
}

/* Helper Function for KVSValue -> Any conversion */
score::Result<score::json::Any> kvsvalue_to_any(const KvsValue& kv) {
    score::Result<score::json::Any> result = score::MakeUnexpected(MyErrorCode::UnmappedError);
    bool error = false;
    score::json::Object obj;
    switch (kv.getType()) {
        case KvsValue::Type::I32: {
            obj.emplace("type", score::json::Any(std::string("I32")));
            obj.emplace("value", score::json::Any(static_cast<int32_t>(std::get<int32_t>(kv.getValue()))));
            break;
        }
        case KvsValue::Type::U32: {
            obj.emplace("type", score::json::Any(std::string("U32")));
            obj.emplace("value", score::json::Any(static_cast<uint32_t>(std::get<uint32_t>(kv.getValue()))));
            break;
        }
        case KvsValue::Type::I64: {
            obj.emplace("type", score::json::Any(std::string("I64")));
            obj.emplace("value", score::json::Any(static_cast<int64_t>(std::get<int64_t>(kv.getValue()))));
            break;
        }
        case KvsValue::Type::U64: {
            obj.emplace("type", score::json::Any(std::string("U64")));
            obj.emplace("value", score::json::Any(static_cast<uint64_t>(std::get<uint64_t>(kv.getValue()))));
            break;
        }
        case KvsValue::Type::F64: {
            obj.emplace("type", score::json::Any(std::string("F64")));
            obj.emplace("value", score::json::Any(std::get<double>(kv.getValue())));
            break;
        }
        case KvsValue::Type::Boolean: {
            obj.emplace("type", score::json::Any(std::string("Boolean")));
            obj.emplace("value", score::json::Any(std::get<bool>(kv.getValue())));
            break;
        }
        case KvsValue::Type::String: {
            obj.emplace("type", score::json::Any(std::string("String")));
            obj.emplace("value", score::json::Any(std::get<std::string>(kv.getValue())));
            break;
        }
        case KvsValue::Type::Null: {
            obj.emplace("type", score::json::Any(std::string("Null")));
            obj.emplace("value", score::json::Any(score::json::Null{}));
            break;
        }
        case KvsValue::Type::Array: {
            obj.emplace("type", score::json::Any(std::string("Array")));
            score::json::List list;
            for (auto& elem : std::get<KvsValue::Array>(kv.getValue())) {
                auto conv = kvsvalue_to_any(elem);
                if (!conv) {
                    result = score::MakeUnexpected(MyErrorCode::InvalidValueType);
                    error = true;
                    break;
                }
                list.push_back(std::move(conv.value()));
            }
            if (!error) {
                obj.emplace("value", score::json::Any(std::move(list)));
            }
            break;
        }
        case KvsValue::Type::Object: {
            obj.emplace("type", score::json::Any(std::string("Object")));
            score::json::Object inner_obj;
            for (auto& [key, value] : std::get<KvsValue::Object>(kv.getValue())) {
                auto conv = kvsvalue_to_any(value);
                if (!conv) {
                    result = score::MakeUnexpected(MyErrorCode::InvalidValueType);
                    error = true;
                    break;
                }
                inner_obj.emplace(key, std::move(conv.value()));
            }
            if (!error) {
                obj.emplace("value", score::json::Any(std::move(inner_obj)));
            }
            break;
        }
        default: {
            result = score::MakeUnexpected(MyErrorCode::InvalidValueType);
            error = true;
            break;
        }
    }

    if (!error) {
        result = score::json::Any(std::move(obj));
    }

    return result;
}


/*********************** Error Implementation *********************/
std::string_view MyErrorDomain::MessageFor(score::result::ErrorCode const& code) const noexcept
{
    std::string_view msg;
    switch (static_cast<MyErrorCode>(code))
    {
        case MyErrorCode::UnmappedError:
            msg = "Error that was not yet mapped";
            break;
        case MyErrorCode::FileNotFound:
            msg = "File not found";
            break;
        case MyErrorCode::KvsFileReadError:
            msg = "KVS file read error";
            break;
        case MyErrorCode::KvsHashFileReadError:
            msg = "KVS hash file read error";
            break;
        case MyErrorCode::JsonParserError:
            msg = "JSON parser error";
            break;
        case MyErrorCode::JsonGeneratorError:
            msg = "JSON generator error";
            break;
        case MyErrorCode::PhysicalStorageFailure:
            msg = "Physical storage failure";
            break;
        case MyErrorCode::IntegrityCorrupted:
            msg = "Integrity corrupted";
            break;
        case MyErrorCode::ValidationFailed:
            msg = "Validation failed";
            break;
        case MyErrorCode::EncryptionFailed:
            msg = "Encryption failed";
            break;
        case MyErrorCode::ResourceBusy:
            msg = "Resource is busy";
            break;
        case MyErrorCode::OutOfStorageSpace:
            msg = "Out of storage space";
            break;
        case MyErrorCode::QuotaExceeded:
            msg = "Quota exceeded";
            break;
        case MyErrorCode::AuthenticationFailed:
            msg = "Authentication failed";
            break;
        case MyErrorCode::KeyNotFound:
            msg = "Key not found";
            break;
        case MyErrorCode::KeyDefaultNotFound:
            msg = "Key default value not found";
            break;
        case MyErrorCode::SerializationFailed:
            msg = "Serialization failed";
            break;
        case MyErrorCode::InvalidSnapshotId:
            msg = "Invalid snapshot ID";
            break;
        case MyErrorCode::ConversionFailed:
            msg = "Conversion failed";
            break;
        case MyErrorCode::MutexLockFailed:
            msg = "Mutex failed";
            break;
        case MyErrorCode::InvalidValueType:
            msg = "Invalid value type";
            break;
        default:
            msg = "Unknown Error!";
            break;
    }

    return msg;
}

score::result::Error MakeError(MyErrorCode code, std::string_view user_message) noexcept
{
    return {static_cast<score::result::ErrorCode>(code), my_error_domain, user_message};
}


/*********************** KVS Builder Implementation *********************/
KvsBuilder::KvsBuilder(const InstanceId& instance_id)
    : instance_id(instance_id)
    , need_defaults(false)
    , need_kvs(false)
    , directory("./data_folder/") /* Default Directory */
{}

KvsBuilder& KvsBuilder::need_defaults_flag(bool flag) {
    need_defaults = flag;
    return *this;
}

KvsBuilder& KvsBuilder::need_kvs_flag(bool flag) {
    need_kvs = flag;
    return *this;
}

KvsBuilder& KvsBuilder::dir(std::string&& dir_path) {
    this->directory = std::move(dir_path);
    return *this;
}


score::Result<Kvs> KvsBuilder::build() const {
    score::Result<Kvs> result = score::MakeUnexpected(MyErrorCode::UnmappedError);

    /* Check if directory argument is valid */
    if (directory.empty() || directory.back() != '/') {
        result = score::MakeUnexpected(MyErrorCode::InvalidArgument);
    } else {
        result = Kvs::open(
            instance_id,
            need_defaults ? OpenNeedDefaults::Required : OpenNeedDefaults::Optional,
            need_kvs      ? OpenNeedKvs::Required      : OpenNeedKvs::Optional,
            std::move(directory)
        );
    }
    
    return result;
}


/*********************** KVS Implementation *********************/
Kvs::~Kvs(){
    if (flush_on_exit.load(std::memory_order_relaxed)) {
        (void)flush();
    }
}

Kvs::Kvs()
    : flush_on_exit(false)
{
}

Kvs::Kvs(Kvs&& other) noexcept
    : filename_prefix(std::move(other.filename_prefix))
    , flush_on_exit(other.flush_on_exit.load(std::memory_order_relaxed))
{
    {
        std::lock_guard<std::mutex> lock(other.kvs_mutex);
        kvs = std::move(other.kvs);
    }
    
    default_values = std::move(other.default_values);
    
    /* Disable flush in source to avoid double flush on destruction */
    other.flush_on_exit.store(false, std::memory_order_relaxed);
}

Kvs& Kvs::operator=(Kvs&& other) noexcept
{
    if (this != &other) {
        {
            std::lock_guard<std::mutex> lock_this(kvs_mutex);
            kvs.clear();
        }
        default_values.clear();
        filename_prefix = std::move(other.filename_prefix);
        
        /* Disable flush in source to avoid double flush on destruction */
        bool flag = other.flush_on_exit.load(std::memory_order_relaxed);
        flush_on_exit.store(flag, std::memory_order_relaxed);
        other.flush_on_exit.store(false, std::memory_order_relaxed);

        {
            std::lock_guard<std::mutex> lock_other(other.kvs_mutex);
            std::lock_guard<std::mutex> lock_this(kvs_mutex);
            kvs = std::move(other.kvs);
        }
        default_values = std::move(other.default_values);
    }
    return *this;
}

/* Helper Function to parse JSON data for open_json*/
score::Result<std::unordered_map<std::string, KvsValue>> parse_json_data(const std::string& data) {
    
    score::Result<unordered_map<std::string, KvsValue>> result = score::MakeUnexpected(MyErrorCode::UnmappedError);
    score::json::JsonParser parser;
    auto any_res = parser.FromBuffer(data);

    if (!any_res) {
        result = score::MakeUnexpected(MyErrorCode::JsonParserError);
    }else{
        score::json::Any root = std::move(any_res).value();
        std::unordered_map<std::string, KvsValue> result_value;

        if (auto obj = root.As<score::json::Object>(); obj.has_value()) {
            bool error = false;
            for (const auto& element : obj.value().get()) {
                auto sv = element.first.GetAsStringView();
                std::string key(sv.data(), sv.size());

                auto conv = any_to_kvsvalue(element.second);
                if (!conv) {
                    result = score::MakeUnexpected(static_cast<MyErrorCode>(*conv.error()));
                    error = true;
                    break;
                }else{
                    result_value.emplace(std::move(key), std::move(conv.value()));
                }
            }
            if (!error) {
                result = std::move(result_value);
            }
        } else {
            result = score::MakeUnexpected(MyErrorCode::JsonParserError);
        }
    }

    return result;
}

/* Open and read JSON File */
score::Result<std::unordered_map<string, KvsValue>> open_json(const string& prefix, OpenJsonNeedFile need_file)
{   
    string json_file = prefix + ".json";
    string hash_file = prefix + ".hash";
    string data;
    bool error = false; /* Error flag */
    bool new_kvs = false; /* Flag to check if new KVS file is created*/
    score::Result<std::unordered_map<string, KvsValue>> result = score::MakeUnexpected(MyErrorCode::UnmappedError);
    
    /* Read JSON file */
    ifstream in(json_file);
    if (!in) {
        if (need_file == OpenJsonNeedFile::Required) {
            cerr << "error: file " << json_file << " could not be read" << endl;
            error = true;
            result = score::MakeUnexpected(MyErrorCode::KvsFileReadError);
        }else{
            cout << "file " << json_file << " not found, using empty data" << endl;
            new_kvs = true;
            result = score::Result<std::unordered_map<string,KvsValue>>({});
        }
    }else{
        ostringstream ss;
        ss << in.rdbuf();
        data = ss.str();
    }

    /* Verify JSON Hash */
    if((!error) && (!new_kvs)){
        ifstream hin(hash_file, ios::binary);
        if (!hin) {
            cerr << "error: hash file " << hash_file << " could not be read" << endl;
            error = true;
            result = score::MakeUnexpected(MyErrorCode::KvsHashFileReadError);
            
        }else{
            bool valid_hash = check_hash(data, hin);
            if(!valid_hash){
                cerr << "error: KVS data corrupted (" << json_file << ", " << hash_file << ")" << endl;
                error = true;
                result = score::MakeUnexpected(MyErrorCode::ValidationFailed);
            }else{
                cout << "JSON data has valid hash" << endl;
            }
        }
    }
    
    /* Parse JSON Data */
    if((!error) && (!new_kvs)){
        auto parse_res = parse_json_data(data);
        if (!parse_res) {
            cerr << "error: parsing JSON data failed" << endl;
            error = true;
            result = score::MakeUnexpected(static_cast<MyErrorCode>(*parse_res.error()));
        }else{
            result = std::move(parse_res.value());
        }
    }

    return result;
}

/* Open KVS Instance */
score::Result<Kvs> Kvs::open(const InstanceId& instance_id, OpenNeedDefaults need_defaults, OpenNeedKvs need_kvs, const std::string&& dir)
{
    const string filename_default = dir + "kvs_" + to_string(instance_id.id) + "_default";
    const string filename_prefix = dir + "kvs_" + to_string(instance_id.id);
    const string filename_kvs = filename_prefix + "_0";

    score::Result<Kvs> result = score::MakeUnexpected(MyErrorCode::UnmappedError); /* Redundant initialization needed, since Resul<KVS> would call the implicitly-deleted default constructor of KVS */
    
    
    auto default_res = open_json(
        filename_default,
        need_defaults == OpenNeedDefaults::Required ? OpenJsonNeedFile::Required : OpenJsonNeedFile::Optional);
    if (!default_res){
        result = score::MakeUnexpected(static_cast<MyErrorCode>(*default_res.error())); /* Dereferences the Error class to its underlying code -> error.h*/
    }
    else{
        auto kvs_res = open_json(
            filename_kvs,
            need_kvs == OpenNeedKvs::Required ? OpenJsonNeedFile::Required : OpenJsonNeedFile::Optional);
        if (!kvs_res){
            result = score::MakeUnexpected(static_cast<MyErrorCode>(*kvs_res.error()));
        }else{ 
            cout << "opened KVS: instance '" << instance_id.id << "'" << endl;
            cout << "max snapshot count: " << KVS_MAX_SNAPSHOTS << endl;
            
            Kvs kvs;
            kvs.kvs = std::move(kvs_res.value());
            kvs.default_values = std::move(default_res.value());
            kvs.filename_prefix = filename_prefix;
            kvs.flush_on_exit.store(true, std::memory_order_relaxed);
            result = std::move(kvs);
        }
    }

    return result;
}

/* Set flush on exit flag*/
void Kvs::set_flush_on_exit(bool flush) {
    flush_on_exit.store(flush, std::memory_order_relaxed);
    return;
}

/* Reset KVS to initial state*/
score::ResultBlank Kvs::reset() {
    score::ResultBlank result = score::MakeUnexpected(MyErrorCode::UnmappedError);
    std::unique_lock<std::mutex> lock(kvs_mutex, std::try_to_lock);
    if (lock.owns_lock()) {
        kvs.clear();
        result = score::ResultBlank{};
    }else{
        result = score::MakeUnexpected(MyErrorCode::MutexLockFailed);
    }

    return result;
}

/* Retrieve all keys in the KVS*/
score::Result<std::vector<std::string_view>> Kvs::get_all_keys() {
    score::Result<std::vector<std::string_view>> result = score::MakeUnexpected(MyErrorCode::UnmappedError);
    std::unique_lock<std::mutex> lock(kvs_mutex, std::try_to_lock);
    if (lock.owns_lock()) {
        std::vector<std::string_view> keys;
        keys.reserve(kvs.size());
        for (const auto& [key, _] : kvs) {
            keys.emplace_back(key);
        }
        result = std::move(keys);
    }else{
        result = score::MakeUnexpected(MyErrorCode::MutexLockFailed);
    }

    return result;
}

/* Check if a key exists*/
score::Result<bool> Kvs::key_exists(const string_view key) {
    score::Result<bool> result = score::MakeUnexpected(MyErrorCode::UnmappedError);
    std::unique_lock<std::mutex> lock(kvs_mutex, std::try_to_lock);
    if (lock.owns_lock()) {
        auto search = kvs.find(std::string(key)); /* unordered_map find() needs string and doesnt work with string_view, workaround for c++20: heterogeneous lookup (applies to more functions) */
        if (search != kvs.end()) {
            result = true;
        } else {
            result = false;
        }
    }else{
        result = score::MakeUnexpected(MyErrorCode::MutexLockFailed);
    }

    return result;
}

/* Retrieve the value associated with a key*/
score::Result<KvsValue> Kvs::get_value(const std::string_view key) {
    score::Result<KvsValue> result = score::MakeUnexpected(MyErrorCode::UnmappedError);
    std::unique_lock<std::mutex> lock_kvs(kvs_mutex, std::try_to_lock);
    if (lock_kvs.owns_lock()){
        auto search_kvs = kvs.find(std::string(key));
        if (search_kvs != kvs.end()) {
            result = search_kvs->second;
        } else {
            auto search_default = default_values.find(std::string(key));
            if (search_default != default_values.end()) {
                result = search_default->second;
            } else {
                result = score::MakeUnexpected(MyErrorCode::KeyNotFound);
            }
        }
    }
    else{
        result = score::MakeUnexpected(MyErrorCode::MutexLockFailed);
    }

    return result;
}

/*Retrieve the default value associated with a key*/
score::Result<KvsValue> Kvs::get_default_value(const std::string_view key) {
    score::Result<KvsValue> result = score::MakeUnexpected(MyErrorCode::UnmappedError);

    auto search = default_values.find(std::string(key));
    if (search != default_values.end()) {
        result = search->second;
    } else {
        result = score::MakeUnexpected(MyErrorCode::KeyNotFound);
    }

    return result;
}

/* Resets a Key to its default value (Deletes written key if default is available) */
score::ResultBlank Kvs::reset_key(const std::string_view key)
{
    score::ResultBlank result = score::MakeUnexpected(MyErrorCode::UnmappedError);
    std::unique_lock<std::mutex> lock_kvs(kvs_mutex, std::try_to_lock);
    if (!lock_kvs.owns_lock()) {
        result = score::MakeUnexpected(MyErrorCode::MutexLockFailed);
    }
    else {
        auto search_default = default_values.find(std::string(key));
        if (search_default == default_values.end()) {
            result = score::MakeUnexpected(MyErrorCode::KeyDefaultNotFound);
        }
        else {
            auto search_kvs = kvs.find(std::string(key));
            if (search_kvs != kvs.end()) {
                (void)kvs.erase(std::string(key)); /* Return Value ignored, since its already secured, that the key exists*/
                result = score::ResultBlank{};
            }else{
                result = score::ResultBlank{};
            }
        }
    }

    return result;
}

/* Check if a key has a default value*/
score::Result<bool> Kvs::has_default_value(const std::string_view key) {
    score::Result<bool> result = score::MakeUnexpected(MyErrorCode::UnmappedError);

    auto search = default_values.find(std::string(key)); /* unordered_map find() needs string and doesnt work with string_view, workaround for c++20: heterogeneous lookup (applies to more functions) */
    if (search != default_values.end()) {
        result = true;
    } else {
        result = false;
    }

    return result;
}

/* Set the value for a key*/
score::ResultBlank Kvs::set_value(const std::string_view key, const KvsValue& value) {
    score::ResultBlank result = score::MakeUnexpected(MyErrorCode::UnmappedError);
    std::unique_lock<std::mutex> lock(kvs_mutex, std::try_to_lock);
    if (lock.owns_lock()) {
        kvs.insert_or_assign(std::string(key), value);
        result = score::ResultBlank{};
    }else{
        result = score::MakeUnexpected(MyErrorCode::MutexLockFailed);
    }
    
    return result;
}

/* Remove a key-value pair*/
score::ResultBlank Kvs::remove_key(const string_view key) {
    score::ResultBlank result = score::MakeUnexpected(MyErrorCode::UnmappedError);
    std::unique_lock<std::mutex> lock(kvs_mutex, std::try_to_lock);
    if (lock.owns_lock()) {
        const auto erased = kvs.erase(std::string(key));
        if (erased > 0U) {
            result = score::ResultBlank{};
        } else {
            result = score::MakeUnexpected(MyErrorCode::KeyNotFound);
        }
    }else{
        result = score::MakeUnexpected(MyErrorCode::MutexLockFailed);
    }

    return result;
}

/* Helper Function to write JSON data to a file for flush process (also adds Hash file)*/
score::ResultBlank write_json_data(const std::string& filename_prefix, const std::string& buf)
{
    score::ResultBlank result = score::MakeUnexpected(MyErrorCode::UnmappedError);
    const std::string fn_json = filename_prefix + "_0.json";
    score::filesystem::Path json_path{fn_json};
    score::filesystem::Path dir = json_path.ParentPath();
    if  (!dir.Empty()) {
        score::filesystem::Filesystem fs = score::filesystem::FilesystemFactory{}.CreateInstance(); /* noexcept call*/
        const auto create_path_res = fs.standard->CreateDirectories(dir);
        if(!create_path_res.has_value()) {
            result = score::MakeUnexpected(MyErrorCode::PhysicalStorageFailure);
        } else {
            std::ofstream out(fn_json, std::ios::binary);
            if (!out.write(buf.data(), buf.size())) {
                result = score::MakeUnexpected(MyErrorCode::PhysicalStorageFailure);
            } else {
                /* Write Hash File */
                std::array<uint8_t, 4> hash_bytes = get_hash_bytes(buf);
                const std::string fn_hash = filename_prefix + "_0.hash";
                std::ofstream hout(fn_hash, std::ios::binary);
                if (!hout.write(reinterpret_cast<const char*>(hash_bytes.data()), hash_bytes.size())) {
                    result = score::MakeUnexpected(MyErrorCode::PhysicalStorageFailure);
                } else {
                    result = score::ResultBlank{};
                }
            }
        }
    } else {
        std::cerr << "Failed to create directory for KVS file '" << fn_json << "'\n";
        result = score::MakeUnexpected(MyErrorCode::UnmappedError); /* Unmapped, because this error shall not occur on runtime but only in unittests since the argument is passed by a const variable*/
    }

    return result;
}

/* Flush the key-value store*/
score::ResultBlank Kvs::flush() {
    score::ResultBlank result = score::MakeUnexpected(MyErrorCode::UnmappedError);
    /* Create JSON Object */
    score::json::Object root_obj;
    bool error = false;
    {
        std::unique_lock<std::mutex> lock(kvs_mutex, std::try_to_lock);
        if (lock.owns_lock()) {
            for (auto const& [key, value] : kvs) {
                auto conv = kvsvalue_to_any(value);
                if (!conv) {
                    result = score::MakeUnexpected(static_cast<MyErrorCode>(*conv.error()));
                    error = true;
                    break;
                }else{
                    root_obj.emplace(
                        key,
                        std::move(conv.value()) /*emplace in map uses move operator*/
                    );
                }
            }
        } else {
            result = score::MakeUnexpected(MyErrorCode::MutexLockFailed);
            error = true;
        }
    }
    
    if(!error){
        /* Serialize Buffer */
        score::json::JsonWriter writer;
        auto buf_res = writer.ToBuffer(root_obj);
        if (!buf_res) {
            result = score::MakeUnexpected(MyErrorCode::JsonGeneratorError);
        }else{
            /* Rotate Snapshots */
            auto rotate_result = snapshot_rotate();
            if (!rotate_result) {
                result = rotate_result;
            }else{
                /* Write JSON Data */
                std::string buf = std::move(buf_res.value());
                result = write_json_data(filename_prefix, buf);
            }
        }
    }
    
    return result;
}

/* Retrieve the snapshot count*/
size_t Kvs::snapshot_count() const {
    size_t count = 0;
    for (size_t idx = 1; idx <= KVS_MAX_SNAPSHOTS; ++idx) {
        const string fname = filename_prefix + "_" + to_string(idx) + ".json";
        if (!std::ifstream(fname)) {
            break;
        }
        count = idx;
    }
    return count;
}

/* Retrieve the max snapshot count*/
size_t Kvs::snapshot_max_count() const {
    return KVS_MAX_SNAPSHOTS;
}

/* Rotate Snapshots */
score::ResultBlank Kvs::snapshot_rotate() {
    score::ResultBlank result = score::MakeUnexpected(MyErrorCode::UnmappedError);
    std::unique_lock<std::mutex> lock(kvs_mutex, std::try_to_lock);
    if (lock.owns_lock()) {
        bool error = false;
        for (size_t idx = KVS_MAX_SNAPSHOTS; idx > 0; --idx) {
            const string hash_old = filename_prefix + "_" + to_string(idx - 1) + ".hash";
            const string hash_new = filename_prefix + "_" + to_string(idx)     + ".hash";
            const string snap_old = filename_prefix + "_" + to_string(idx - 1) + ".json";
            const string snap_new = filename_prefix + "_" + to_string(idx)     + ".json";

            cout << "rotating: " << snap_old << " -> " << snap_new << endl;
            /* Rename hash */
            int32_t hash_rename = std::rename(hash_old.c_str(), hash_new.c_str());
            if (0 != hash_rename) {
                if (errno != ENOENT) {
                    error = true;
                    cout << "error: could not rename hash file " << snap_old << ". Rename Errorcode " << errno << endl;
                    result = score::MakeUnexpected(MyErrorCode::PhysicalStorageFailure);
                }
            }
            if(!error){
                /* Rename snapshot */
                int32_t snap_rename = std::rename(snap_old.c_str(), snap_new.c_str());
                if (0 != snap_rename) {
                    if (errno != ENOENT) {
                        error = true;
                        cout << "error: could not rename snapshot file " << snap_old << ". Rename Errorcode " << errno << endl;
                        result = score::MakeUnexpected(MyErrorCode::PhysicalStorageFailure);
                    }
                }
            }
            if(error){
                break;
            }
        }
        if(!error){
            result = score::ResultBlank{};
        }
    } else {
        result = score::MakeUnexpected(MyErrorCode::MutexLockFailed);
    }

    return result;
}

/* Restore the key-value store from a snapshot*/
score::ResultBlank Kvs::snapshot_restore(const SnapshotId& snapshot_id) {
    score::ResultBlank result = score::MakeUnexpected(MyErrorCode::UnmappedError);
    std::unique_lock<std::mutex> lock(kvs_mutex, std::try_to_lock);
    if (lock.owns_lock()) {
        /* Fail if the snapshot ID is the current KVS */
        if (0 == snapshot_id.id) {
            result = score::MakeUnexpected(MyErrorCode::InvalidSnapshotId);
        }else if (snapshot_count() < snapshot_id.id) {
            result = score::MakeUnexpected(MyErrorCode::InvalidSnapshotId);
        }else{
            auto data_res = open_json(
                filename_prefix + "_" + to_string(snapshot_id.id),
                OpenJsonNeedFile::Required);
            if (!data_res) {
                result = score::MakeUnexpected(static_cast<MyErrorCode>(*data_res.error()));
            }else{
                kvs = std::move(data_res.value());
                result = score::ResultBlank{};
            }
        }
    }else{
        result = score::MakeUnexpected(MyErrorCode::MutexLockFailed);
    }
    
    return result;
}

/* Get the filename for a snapshot*/
std::string Kvs::get_kvs_filename(const SnapshotId& snapshot_id) const {
    std::string filename = filename_prefix + "_" + std::to_string(snapshot_id.id) + ".json";
    return filename;
}

/* Get the hash filename for a snapshot*/
std::string Kvs::get_kvs_hash_filename(const SnapshotId& snapshot_id) const {
    std::string filename = filename_prefix + "_" + std::to_string(snapshot_id.id) + ".hash";
    return filename;
}
