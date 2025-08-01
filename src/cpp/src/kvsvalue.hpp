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
#ifndef SCORE_LIB_KVS_KVSVALUE_HPP
#define SCORE_LIB_KVS_KVSVALUE_HPP

#include <optional>
#include <stdexcept>
#include <string>
#include <unordered_map>
#include <vector>

namespace score::mw::pers::kvs {

/* Define the KvsValue class*/
/**
 * @class KvsValue
 * @brief Represents a flexible value type that can hold various data types, 
 *        including numbers, booleans, strings, null, arrays, and objects.
 * 
 * The KvsValue class provides a type-safe way to store and retrieve values of 
 * different types. It uses a std::variant to hold the underlying value and an 
 * enum to track the type of the value.
 * 
 * ## Supported Types:
 * - Number (double)
 * - Boolean (bool)
 * - String (std::string)
 * - Null (std::nullptr_t)
 * - Array (std::vector<KvsValue>)
 * - Object (std::unordered_map<std::string, KvsValue>)
 * 
 * ## Public Methods:
 * - `KvsValue(double number)`: Constructs a KvsValue holding a number.
 * - `KvsValue(bool boolean)`:
 * - Access the underlying value using `getValue()` and `std::get`.
 *
 * ## Example:
 * @code
 * KvsValue numberValue(42.0);
 * KvsValue stringValue("Hello, World!");
 * KvsValue arrayValue(KvsValue::Array{numberValue, stringValue});
 *
 * if (numberValue.getType() == KvsValue::Type::Number) {
 *     double number = std::get<double>(numberValue.getValue());
 * }
 * @endcode
 */

class KvsValue final{
public:
    /* Define the possible types for KvsValue*/
    using Array = std::vector<KvsValue>;
    using Object = std::unordered_map<std::string, KvsValue>;

    /* Enum to represent the type of the value*/
    enum class Type {
        i32,
        u32,
        i64,
        u64,
        f64,
        Boolean,
        String,
        Null,
        Array,
        Object
    };

    /* Constructors for each type*/
    explicit KvsValue(int32_t number) : value(number), type(Type::i32) {}
    explicit KvsValue(uint32_t number) : value(number), type(Type::u32) {}
    explicit KvsValue(int64_t number) : value(number), type(Type::i64) {}
    explicit KvsValue(uint64_t number) : value(number), type(Type::u64) {}
    explicit KvsValue(double number) : value(number), type(Type::f64) {}
    explicit KvsValue(bool boolean) : value(boolean), type(Type::Boolean) {}
    explicit KvsValue(const std::string& str) : value(str), type(Type::String) {}
    explicit KvsValue(std::nullptr_t) : value(nullptr), type(Type::Null) {}
    explicit KvsValue(const Array& array) : value(array), type(Type::Array) {}
    explicit KvsValue(const Object& object) : value(object), type(Type::Object) {}

    /* Get the type of the value*/
    Type getType() const { return type; }

    /* Access the underlying value (use std::get to retrieve the value)*/
    const std::variant<int32_t, uint32_t, int64_t, uint64_t, double, bool, std::string, std::nullptr_t, Array, Object>& getValue() const {
        return value;
    }

private:
    /* The underlying value*/
    std::variant<int32_t, uint32_t, int64_t, uint64_t, double, bool, std::string, std::nullptr_t, Array, Object> value;

    /* The type of the value*/
    Type type;
};

} /* namespace score::mw::pers::kvs */

#endif /* SCORE_LIB_KVS_KVSVALUE_HPP */
