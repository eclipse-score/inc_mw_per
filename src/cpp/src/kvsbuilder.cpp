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

namespace score::mw::per::kvs
{

    // Definition of static member variable
    std::unordered_map<size_t, std::shared_ptr<Kvs>> KvsBuilder::kvs_map;
    KvsBuilder* KvsBuilder::latest_instance = nullptr;

    /*********************** KVS Builder Implementation *********************/
    KvsBuilder::KvsBuilder(const InstanceId &instance_id)
        : instance_id(instance_id), need_defaults(false), need_kvs(false), directory("./data_folder/") /* Default Directory */
    {
        latest_instance = this;
    }

    KvsBuilder::~KvsBuilder()
    {
        if(this == latest_instance) {
            kvs_map.clear();
        }
    }

    KvsBuilder &KvsBuilder::need_defaults_flag(bool flag)
    {
        need_defaults = flag;
        return *this;
    }

    KvsBuilder &KvsBuilder::need_kvs_flag(bool flag)
    {
        need_kvs = flag;
        return *this;
    }

    KvsBuilder &KvsBuilder::dir(std::string &&dir_path)
    {
        this->directory = std::move(dir_path);
        return *this;
    }

    score::Result<std::shared_ptr<Kvs>> KvsBuilder::build()
    {

        /* Use current directory if empty */
        if (directory.empty())
        {
            directory = "./";
        }

        // Check if we already have this KVS instance cached
        auto it = kvs_map.find(instance_id.id);
        if (it != kvs_map.end())
        {
            // Return the cached shared_ptr
            return score::Result<std::shared_ptr<Kvs>>(it->second);
        }

        // Open new KVS
        auto opened_kvs = Kvs::open(
            instance_id,
            need_defaults ? OpenNeedDefaults::Required : OpenNeedDefaults::Optional,
            need_kvs ? OpenNeedKvs::Required : OpenNeedKvs::Optional,
            std::move(directory));

        if (opened_kvs.has_value())
        {
            // Create shared_ptr by moving the opened KVS
            auto kvs_ptr = std::make_shared<Kvs>(std::move(opened_kvs.value()));
            kvs_map.insert({instance_id.id, kvs_ptr});
            return score::Result<std::shared_ptr<Kvs>>(kvs_ptr);
        }
        else
        {
            // Return the error wrapped in the expected Result type
            return score::Result<std::shared_ptr<Kvs>>(score::Unexpected{opened_kvs.error()});
        }
    }

} /* namespace score::mw::per::kvs */