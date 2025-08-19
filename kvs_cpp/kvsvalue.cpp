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
#include "kvsvalue.hpp"

namespace score::mw::per::kvs {

KvsValue::KvsValue(const Array& array){
    Array shared_array;
    shared_array.reserve(array.size());
    for (const auto& item : array) {
        shared_array.push_back(std::make_shared<KvsValue>(*item));
    }
    value = std::move(shared_array);
    type = Type::Array;
}

KvsValue::KvsValue(const Object& object) {
    Object shared_object;
    for (const auto& [key, value] : object) {
        shared_object[key] = std::make_shared<KvsValue>(*value);
    }
    value = std::move(shared_object);
    type = Type::Object;
}

KvsValue::KvsValue(const std::vector<KvsValue>& array) {
    Array shared_array;
    shared_array.reserve(array.size());  // Reserve space for N elements
    for (const auto& item : array) {
        shared_array.emplace_back(std::make_shared<KvsValue>(item));
    }
    value = std::move(shared_array);
    type = Type::Array;
}

KvsValue::KvsValue(const std::unordered_map<std::string, KvsValue>& object) {
    Object shared_object;
    for (const auto& [key, value] : object) {
        shared_object[key] = std::make_shared<KvsValue>(value);
    }
    value = std::move(shared_object);
    type = Type::Object;
}

/* copy constructor */
KvsValue::KvsValue(const KvsValue& other) : type(other.type) {
    switch(other.type){
        case Type::Array:{
            const Array& otherArray = std::get<Array>(other.value);
            Array copiedArray;
            copiedArray.reserve(otherArray.size());
            for (const auto& item : otherArray) {
                copiedArray.push_back(std::make_shared<KvsValue>(*item));
            }
            value = std::move(copiedArray);
            break;
        }
        case Type::Object:{
            const Object& otherObject = std::get<Object>(other.value);
            Object copiedObject;
            for (const auto& [key, value] : otherObject) {
                copiedObject[key] = std::make_shared<KvsValue>(*value);
            }
            value = std::move(copiedObject);
            break;
        }
        default:
            value = other.value; // For other types, just copy the value
            break;
    }
}

/* copy Assignment Operator */
KvsValue& KvsValue::operator=(const KvsValue& other) {
    if (this != &other) {
        KvsValue temp(other); // deep copy
        std::swap(value, temp.value);
        type = other.type;
    }
    return *this;
}

/* move Assignment Operator */
KvsValue& KvsValue::operator=(KvsValue&& other) noexcept {
    if (this != &other) {
        value = std::move(other.value);
        type = other.type;
    }
    return *this;
}

} /* end namespace score::mw::per::kvs */