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
#include "internal/kvs_helper.hpp"
#include "kvs.hpp"

//TODO Default Value Handling TBD
//TODO Add Score Logging
//TODO Replace std libs with baselibs
//TODO String Handling in set_value TBD: (e.g. set_value("key", "value") is not correct, since KvsValue() expects a string ptr and not a string)

using namespace std;

namespace score::mw::pers::kvs {

/*********************** KVS Implementation *********************/
Kvs::~Kvs(){
    if (flush_on_exit.load(std::memory_order_relaxed)) {
        (void)flush();
    }
}

Kvs::Kvs()
    : flush_on_exit(false)
    , filesystem(std::make_unique<score::filesystem::Filesystem>(score::filesystem::FilesystemFactory{}.CreateInstance())) /* Create Filesystem instance, noexcept call */
    , parser(std::make_unique<score::json::JsonParser>())
    , writer(std::make_unique<score::json::JsonWriter>())
{
}

Kvs::Kvs(Kvs&& other) noexcept
    : filename_prefix(std::move(other.filename_prefix))
    , flush_on_exit(other.flush_on_exit.load(std::memory_order_relaxed))
    , filesystem(std::move(other.filesystem))
    , parser(std::move(other.parser)) /* Not absolutely necessary, because a new JSON writer/parser object would also be okay*/
    , writer(std::move(other.writer))
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

        filesystem = std::move(other.filesystem);
        /* Transfer ownership of JSON parser and writer
            Not absolutely necessary, because a new JSON writer/parser object would also be okay*/
        parser = std::move(other.parser);
        writer = std::move(other.writer);
    }
    return *this;
}

/* Helper Function to parse JSON data for open_json*/
score::Result<std::unordered_map<std::string, KvsValue>> Kvs::parse_json_data(const std::string& data) {
    
    score::Result<unordered_map<std::string, KvsValue>> result = score::MakeUnexpected(ErrorCode::UnmappedError);
    auto any_res = parser->FromBuffer(data);

    if (!any_res) {
        result = score::MakeUnexpected(ErrorCode::JsonParserError);
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
                    result = score::MakeUnexpected(static_cast<ErrorCode>(*conv.error()));
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
            result = score::MakeUnexpected(ErrorCode::JsonParserError);
        }
    }

    return result;
}

/* Open and read JSON File */
score::Result<std::unordered_map<string, KvsValue>> Kvs::open_json(const score::filesystem::Path& prefix, OpenJsonNeedFile need_file)
{   
    score::filesystem::Path json_file = prefix.Native() + ".json";
    score::filesystem::Path hash_file = prefix.Native() + ".hash";
    std::string data;
    bool error = false; /* Error flag */
    bool new_kvs = false; /* Flag to check if new KVS file is created*/
    score::Result<std::unordered_map<string, KvsValue>> result = score::MakeUnexpected(ErrorCode::UnmappedError);
    
    /* Read JSON file */
    ifstream in(json_file.CStr());
    if (!in) {
        if (need_file == OpenJsonNeedFile::Required) {
            cerr << "error: file " << json_file << " could not be read" << endl;
            error = true;
            result = score::MakeUnexpected(ErrorCode::KvsFileReadError);
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
        ifstream hin(hash_file.CStr(), ios::binary);
        if (!hin) {
            cerr << "error: hash file " << hash_file << " could not be read" << endl;
            error = true;
            result = score::MakeUnexpected(ErrorCode::KvsHashFileReadError);
            
        }else{
            bool valid_hash = check_hash(data, hin);
            if(!valid_hash){
                cerr << "error: KVS data corrupted (" << json_file << ", " << hash_file << ")" << endl;
                error = true;
                result = score::MakeUnexpected(ErrorCode::ValidationFailed);
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
            result = score::MakeUnexpected(static_cast<ErrorCode>(*parse_res.error()));
        }else{
            result = std::move(parse_res.value());
        }
    }

    return result;
}

/* Open KVS Instance */
score::Result<Kvs> Kvs::open(const InstanceId& instance_id, OpenNeedDefaults need_defaults, OpenNeedKvs need_kvs, const std::string&& dir)
{
    score::Result<Kvs> result = score::MakeUnexpected(ErrorCode::UnmappedError); /* Redundant initialization needed, since Resul<KVS> would call the implicitly-deleted default constructor of KVS */

    score::filesystem::Path base_path(dir);
    score::filesystem::Path filename_prefix = base_path / ("kvs_" + std::to_string(instance_id.id));
    const score::filesystem::Path filename_default = filename_prefix.Native() + "_default";
    const score::filesystem::Path filename_kvs = filename_prefix.Native() + "_0";
    
    Kvs kvs; /* Create KVS instance */
    auto default_res = kvs.open_json(
        filename_default,
        need_defaults == OpenNeedDefaults::Required ? OpenJsonNeedFile::Required : OpenJsonNeedFile::Optional);
    if (!default_res){
        result = score::MakeUnexpected(static_cast<ErrorCode>(*default_res.error())); /* Dereferences the Error class to its underlying code -> error.h*/
    }
    else{
        auto kvs_res = kvs.open_json(
            filename_kvs,
            need_kvs == OpenNeedKvs::Required ? OpenJsonNeedFile::Required : OpenJsonNeedFile::Optional);
        if (!kvs_res){
            result = score::MakeUnexpected(static_cast<ErrorCode>(*kvs_res.error()));
        }else{ 
            cout << "opened KVS: instance '" << instance_id.id << "'" << endl;
            cout << "max snapshot count: " << KVS_MAX_SNAPSHOTS << endl;
            
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
    score::ResultBlank result = score::MakeUnexpected(ErrorCode::UnmappedError);
    std::unique_lock<std::mutex> lock(kvs_mutex, std::try_to_lock);
    if (lock.owns_lock()) {
        kvs.clear();
        result = score::ResultBlank{};
    }else{
        result = score::MakeUnexpected(ErrorCode::MutexLockFailed);
    }

    return result;
}

/* Retrieve all keys in the KVS*/
score::Result<std::vector<std::string_view>> Kvs::get_all_keys() {
    score::Result<std::vector<std::string_view>> result = score::MakeUnexpected(ErrorCode::UnmappedError);
    std::unique_lock<std::mutex> lock(kvs_mutex, std::try_to_lock);
    if (lock.owns_lock()) {
        std::vector<std::string_view> keys;
        keys.reserve(kvs.size());
        for (const auto& [key, _] : kvs) {
            keys.emplace_back(key);
        }
        result = std::move(keys);
    }else{
        result = score::MakeUnexpected(ErrorCode::MutexLockFailed);
    }

    return result;
}

/* Check if a key exists*/
score::Result<bool> Kvs::key_exists(const std::string_view key) {
    score::Result<bool> result = score::MakeUnexpected(ErrorCode::UnmappedError);
    std::unique_lock<std::mutex> lock(kvs_mutex, std::try_to_lock);
    if (lock.owns_lock()) {
        auto search = kvs.find(std::string(key)); /* unordered_map find() needs string and doesnt work with string_view, workaround for c++20: heterogeneous lookup (applies to more functions) */
        if (search != kvs.end()) {
            result = true;
        } else {
            result = false;
        }
    }else{
        result = score::MakeUnexpected(ErrorCode::MutexLockFailed);
    }

    return result;
}

/* Retrieve the value associated with a key*/
score::Result<KvsValue> Kvs::get_value(const std::string_view key) {
    score::Result<KvsValue> result = score::MakeUnexpected(ErrorCode::UnmappedError);
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
                result = score::MakeUnexpected(ErrorCode::KeyNotFound);
            }
        }
    }
    else{
        result = score::MakeUnexpected(ErrorCode::MutexLockFailed);
    }

    return result;
}

/*Retrieve the default value associated with a key*/
score::Result<KvsValue> Kvs::get_default_value(const std::string_view key) {
    score::Result<KvsValue> result = score::MakeUnexpected(ErrorCode::UnmappedError);

    auto search = default_values.find(std::string(key));
    if (search != default_values.end()) {
        result = search->second;
    } else {
        result = score::MakeUnexpected(ErrorCode::KeyNotFound);
    }

    return result;
}

/* Resets a Key to its default value (Deletes written key if default is available) */
score::ResultBlank Kvs::reset_key(const std::string_view key)
{
    score::ResultBlank result = score::MakeUnexpected(ErrorCode::UnmappedError);
    std::unique_lock<std::mutex> lock_kvs(kvs_mutex, std::try_to_lock);
    if (!lock_kvs.owns_lock()) {
        result = score::MakeUnexpected(ErrorCode::MutexLockFailed);
    }
    else {
        auto search_default = default_values.find(std::string(key));
        if (search_default == default_values.end()) {
            result = score::MakeUnexpected(ErrorCode::KeyDefaultNotFound);
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
    score::Result<bool> result = score::MakeUnexpected(ErrorCode::UnmappedError);

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
    score::ResultBlank result = score::MakeUnexpected(ErrorCode::UnmappedError);
    std::unique_lock<std::mutex> lock(kvs_mutex, std::try_to_lock);
    if (lock.owns_lock()) {
        kvs.insert_or_assign(std::string(key), value);
        result = score::ResultBlank{};
    }else{
        result = score::MakeUnexpected(ErrorCode::MutexLockFailed);
    }
    
    return result;
}

/* Remove a key-value pair*/
score::ResultBlank Kvs::remove_key(const std::string_view key) {
    score::ResultBlank result = score::MakeUnexpected(ErrorCode::UnmappedError);
    std::unique_lock<std::mutex> lock(kvs_mutex, std::try_to_lock);
    if (lock.owns_lock()) {
        const auto erased = kvs.erase(std::string(key));
        if (erased > 0U) {
            result = score::ResultBlank{};
        } else {
            result = score::MakeUnexpected(ErrorCode::KeyNotFound);
        }
    }else{
        result = score::MakeUnexpected(ErrorCode::MutexLockFailed);
    }

    return result;
}

/* Helper Function to write JSON data to a file for flush process (also adds Hash file)*/
score::ResultBlank Kvs::write_json_data(const std::string& buf)
{
    score::ResultBlank result = score::MakeUnexpected(ErrorCode::UnmappedError);
    score::filesystem::Path json_path{filename_prefix.Native() + "_0.json"};
    score::filesystem::Path dir = json_path.ParentPath();
    if  (!dir.Empty()) {
        const auto create_path_res = filesystem->standard->CreateDirectories(dir);
        if(!create_path_res.has_value()) {
            result = score::MakeUnexpected(ErrorCode::PhysicalStorageFailure);
        } else {
            std::ofstream out(json_path.CStr(), std::ios::binary);
            if (!out.write(buf.data(), buf.size())) {
                result = score::MakeUnexpected(ErrorCode::PhysicalStorageFailure);
            } else {
                /* Write Hash File */
                std::array<uint8_t, 4> hash_bytes = get_hash_bytes(buf);
                score::filesystem::Path fn_hash = filename_prefix.Native() + "_0.hash";
                std::ofstream hout(fn_hash.CStr(), std::ios::binary);
                if (!hout.write(reinterpret_cast<const char*>(hash_bytes.data()), hash_bytes.size())) {
                    result = score::MakeUnexpected(ErrorCode::PhysicalStorageFailure);
                } else {
                    result = score::ResultBlank{};
                }
            }
        }
    } else {
        std::cerr << "Failed to create directory for KVS file '" << json_path.CStr() << "'\n";
        result = score::MakeUnexpected(ErrorCode::PhysicalStorageFailure);
    }

    return result;
}

/* Flush the key-value store*/
score::ResultBlank Kvs::flush() {
    score::ResultBlank result = score::MakeUnexpected(ErrorCode::UnmappedError);
    /* Create JSON Object */
    score::json::Object root_obj;
    bool error = false;
    {
        std::unique_lock<std::mutex> lock(kvs_mutex, std::try_to_lock);
        if (lock.owns_lock()) {
            for (auto const& [key, value] : kvs) {
                auto conv = kvsvalue_to_any(value);
                if (!conv) {
                    result = score::MakeUnexpected(static_cast<ErrorCode>(*conv.error()));
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
            result = score::MakeUnexpected(ErrorCode::MutexLockFailed);
            error = true;
        }
    }
    
    if(!error){
        /* Serialize Buffer */
        auto buf_res = writer->ToBuffer(root_obj);
        if (!buf_res) {
            result = score::MakeUnexpected(ErrorCode::JsonGeneratorError);
        }else{
            /* Rotate Snapshots */
            auto rotate_result = snapshot_rotate();
            if (!rotate_result) {
                result = rotate_result;
            }else{
                /* Write JSON Data */
                std::string buf = std::move(buf_res.value());
                result = write_json_data(buf);
            }
        }
    }
    
    return result;
}

/* Retrieve the snapshot count*/
score::Result<size_t> Kvs::snapshot_count() const {
    score::Result<size_t> result = score::MakeUnexpected(ErrorCode::UnmappedError);
    size_t count = 0;
    bool error = false;
    for (size_t idx = 1; idx <= KVS_MAX_SNAPSHOTS; ++idx) {
        const score::filesystem::Path fname = filename_prefix.Native() + "_" + to_string(idx) + ".json";
        const auto fname_exists_res = filesystem->standard->Exists(fname);
        if (fname_exists_res) {
            if(false == fname_exists_res.value()) {
                break;
            }
        } else{
            error = true;
            break;
        }
        count = idx;
    }
    if (error) {
        result = score::MakeUnexpected(ErrorCode::PhysicalStorageFailure);
    } else {
        result = count;
    }

    return result;
}

/* Retrieve the max snapshot count*/
size_t Kvs::snapshot_max_count() const {
    return KVS_MAX_SNAPSHOTS;
}

/* Rotate Snapshots */
score::ResultBlank Kvs::snapshot_rotate() {
    score::ResultBlank result = score::MakeUnexpected(ErrorCode::UnmappedError);
    std::unique_lock<std::mutex> lock(kvs_mutex, std::try_to_lock);
    if (lock.owns_lock()) {
        bool error = false;
        for (size_t idx = KVS_MAX_SNAPSHOTS; idx > 0; --idx) {
            score::filesystem::Path hash_old = filename_prefix.Native() + "_" + to_string(idx - 1) + ".hash";
            score::filesystem::Path hash_new = filename_prefix.Native() + "_" + to_string(idx)     + ".hash";
            score::filesystem::Path snap_old = filename_prefix.Native() + "_" + to_string(idx - 1) + ".json";
            score::filesystem::Path snap_new = filename_prefix.Native() + "_" + to_string(idx)     + ".json";

            cout << "rotating: " << snap_old << " -> " << snap_new << endl;
            /* Rename hash */
            int32_t hash_rename = std::rename(hash_old.CStr(), hash_new.CStr());
            if (0 != hash_rename) {
                if (errno != ENOENT) {
                    error = true;
                    cout << "error: could not rename hash file " << snap_old << ". Rename Errorcode " << errno << endl;
                    result = score::MakeUnexpected(ErrorCode::PhysicalStorageFailure);
                }
            }
            if(!error){
                /* Rename snapshot */
                int32_t snap_rename = std::rename(snap_old.CStr(), snap_new.CStr());
                if (0 != snap_rename) {
                    if (errno != ENOENT) {
                        error = true;
                        cout << "error: could not rename snapshot file " << snap_old << ". Rename Errorcode " << errno << endl;
                        result = score::MakeUnexpected(ErrorCode::PhysicalStorageFailure);
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
        result = score::MakeUnexpected(ErrorCode::MutexLockFailed);
    }

    return result;
}

/* Restore the key-value store from a snapshot*/
score::ResultBlank Kvs::snapshot_restore(const SnapshotId& snapshot_id) {
    score::ResultBlank result = score::MakeUnexpected(ErrorCode::UnmappedError);
    std::unique_lock<std::mutex> lock(kvs_mutex, std::try_to_lock);
    if (lock.owns_lock()) {
        auto snapshot_count_res = snapshot_count();
        if (!snapshot_count_res) {
            result = score::MakeUnexpected(static_cast<ErrorCode>(*snapshot_count_res.error()));
        }else{
            /* Fail if the snapshot ID is the current KVS */
            if (0 == snapshot_id.id) {
                result = score::MakeUnexpected(ErrorCode::InvalidSnapshotId);
            }else if (snapshot_count_res.value() < snapshot_id.id) {
                result = score::MakeUnexpected(ErrorCode::InvalidSnapshotId);
            }else{
                score::filesystem::Path restore_path = filename_prefix.Native() + "_" + to_string(snapshot_id.id);
                auto data_res = open_json(
                    restore_path,
                    OpenJsonNeedFile::Required);
                if (!data_res) {
                    result = score::MakeUnexpected(static_cast<ErrorCode>(*data_res.error()));
                }else{
                    kvs = std::move(data_res.value());
                    result = score::ResultBlank{};
                }
            }
        }
    } else {
        result = score::MakeUnexpected(ErrorCode::MutexLockFailed);
    }
    
    return result;
}

/* Get the filename for a snapshot*/
score::Result<score::filesystem::Path> Kvs::get_kvs_filename(const SnapshotId& snapshot_id) const {
    score::filesystem::Path filename = filename_prefix.Native() + "_" + std::to_string(snapshot_id.id) + ".json";
    score::Result<score::filesystem::Path> result = score::MakeUnexpected(ErrorCode::UnmappedError);

    const auto fname_exists_res = filesystem->standard->Exists(filename);
    if (fname_exists_res) {
        if (false == fname_exists_res.value()) {
            result = score::MakeUnexpected(ErrorCode::FileNotFound);
        } else {
            result = filename;
        }
    } else {
        result = score::MakeUnexpected(static_cast<ErrorCode>(*fname_exists_res.error()));
    }
    return result;
}

/* Get the hash filename for a snapshot*/
score::Result<score::filesystem::Path> Kvs::get_hash_filename(const SnapshotId& snapshot_id) const {
    score::filesystem::Path filename = filename_prefix.Native() + "_" + std::to_string(snapshot_id.id) + ".hash";
    score::Result<score::filesystem::Path> result = score::MakeUnexpected(ErrorCode::UnmappedError);

    const auto fname_exists_res = filesystem->standard->Exists(filename);
    if (fname_exists_res) {
        if (false == fname_exists_res.value()) {
            result = score::MakeUnexpected(ErrorCode::FileNotFound);
        } else {
            result = filename;
        }
    } else {
        result = score::MakeUnexpected(static_cast<ErrorCode>(*fname_exists_res.error()));
    }
    return result;
}

} /* namespace score::mw::pers::kvs */
