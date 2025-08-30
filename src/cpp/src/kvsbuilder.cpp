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
#include "kvsbuilder.hpp"

namespace score::mw::per::kvs {

//Definition of static member variables
std::unordered_map<size_t, std::shared_ptr<Kvs>> KvsBuilder::kvs_instances;
KvsBuilder* KvsBuilder::latest_instance = nullptr;
std::mutex                  KvsBuilder::kvs_instances_mutex;                 ///< Mutex for synchronizing access to KVS instances
std::mutex                  KvsBuilder::latest_instance_mutex;               ///< Mutex for synchronizing access to KVS Builder

/*********************** KVS Builder Implementation *********************/
KvsBuilder::KvsBuilder(const InstanceId& instance_id)
    : instance_id(instance_id)
    , need_defaults(false)
    , need_kvs(false)
    , directory("./data_folder/") /* Default Directory */
{
    std::lock_guard<std::mutex> lock(latest_instance_mutex);
    latest_instance = this;
}

KvsBuilder::~KvsBuilder() {
    std::lock(latest_instance_mutex, kvs_instances_mutex);
    std::lock_guard<std::mutex> latest_lock(latest_instance_mutex, std::adopt_lock);
    std::lock_guard<std::mutex> kvs_instances_lock(kvs_instances_mutex, std::adopt_lock);
    if (latest_instance == this) {
       kvs_instances.clear();
    }
}

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


score::Result<std::shared_ptr<Kvs>> KvsBuilder::build() {

    /* Use current directory if empty */
    if (directory.empty()) {
        directory = "./";
    }

    //Lock the mutex before accessing the cache
    std::lock_guard<std::mutex> lock(kvs_instances_mutex);

    auto it = kvs_instances.find(instance_id.id);
    if(it != kvs_instances.end()) {
        /* Return existing instance */
        return score::Result<std::shared_ptr<Kvs>>(it->second);
    }

    auto opened_kvs = Kvs::open(
        instance_id,
        need_defaults ? OpenNeedDefaults::Required : OpenNeedDefaults::Optional,
        need_kvs      ? OpenNeedKvs::Required      : OpenNeedKvs::Optional,
        std::move(directory)
    );

    if(opened_kvs.has_value()) {
        auto kvs_ptr = std::make_shared<Kvs>(std::move(opened_kvs.value()));
        kvs_instances.insert({instance_id.id, kvs_ptr});
        return score::Result<std::shared_ptr<Kvs>>(kvs_ptr);
    }
    else {
        return score::Result<std::shared_ptr<Kvs>>(score::Unexpected(opened_kvs.error()));
    }

}

void KvsBuilder::clear_cache() {
    std::lock_guard<std::mutex> lock(kvs_instances_mutex);
    kvs_instances.clear();
}

} /* namespace score::mw::per::kvs */
