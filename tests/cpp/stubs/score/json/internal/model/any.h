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
// Difference to the original file:
// - Added global control flags to control the stub behavior
// - Stubbed NULL and Number classes inside the Any.h
// - Added simpler constructors for Any class to handle different JSON types sufficiently for unittests
// - Any is copyable to allow comparisons in unittests
////////////////////////////////////////////////////////////////////////////////

#ifndef SCORE_LIB_JSON_INTERNAL_MODEL_ANY_H
#define SCORE_LIB_JSON_INTERNAL_MODEL_ANY_H

#include <cstddef>
#include <functional>
#include <map>
#include <string>
#include <variant>
#include <vector>
#include "score/json/internal/model/error.h"
#include "score/memory/string_comparison_adaptor.h"
#include "score/result/result.h"
#include <score/string_view.hpp>


namespace score {
namespace json {

//Conntrol flags for As Behaviour
extern bool g_AnyNumberAsShouldFail; // Global flag to control failure of As methods

////////////////////////////////////////////////////////////////////////////////
// Null stub
////////////////////////////////////////////////////////////////////////////////
struct Null {};

////////////////////////////////////////////////////////////////////////////////
// Number stub: holds a double, supports As<T>() for arithmetic T ≠ bool
////////////////////////////////////////////////////////////////////////////////
class Number
{
public:
  explicit Number(double v) noexcept
    : value_{v}
  {}

  template<typename T,
           typename = std::enable_if_t<std::is_arithmetic_v<T> && !std::is_same_v<T,bool>, bool>>
  score::Result<T> As() const noexcept
  {
    if(g_AnyNumberAsShouldFail) {
        return score::MakeUnexpected(json::Error::kWrongType);
    }else{
        return static_cast<T>(value_);
    }
  }

private:
  double value_;
};

////////////////////////////////////////////////////////////////////////////////
// Forward declarations & aliases
////////////////////////////////////////////////////////////////////////////////
class Any;
using List   = std::vector<Any>;
using Object = std::map<score::memory::StringComparisonAdaptor, Any>;

////////////////////////////////////////////////////////////////////////////////
// Any stub
//
// Holds one of: Null, bool, Number, std::string, List, Object
////////////////////////////////////////////////////////////////////////////////
class Any
{
public:
    // Default  (→ Null)
    Any() noexcept = default;

    // Copy/move -> Copy is allowed in Unittest to compare values 
    Any(const Any&) = default;
    Any& operator=(const Any&) = default;
    Any(Any&&) noexcept = default;
    Any& operator=(Any&&) noexcept = default;

    ~Any() = default;

    // constructors for each JSON type
    explicit Any(Null) noexcept                      : value_(Null{}) {}
    explicit Any(bool b) noexcept                    : value_(b) {}
    explicit Any(double d) noexcept                  : value_(Number(d)) {}
    explicit Any(Number n) noexcept                  : value_(std::move(n)) {}
    explicit Any(const std::string& s)               : value_(s) {}
    explicit Any(std::string&& s)                    : value_(std::move(s)) {}
    explicit Any(const char* s)                      : value_(std::string(s)) {}
    explicit Any(const List& l)                      : value_(l) {}
    explicit Any(List&& l)                           : value_(std::move(l)) {}
    explicit Any(const Object& o)                    : value_(o) {}
    explicit Any(Object&& o)                         : value_(std::move(o)) {}

    // As<Null>
    template<typename T = Null>
    std::enable_if_t<std::is_same_v<T, Null>, score::Result<Null>>
    As() const noexcept
    {
        if (auto p = std::get_if<Null>(&value_))
        return *p;
        return score::MakeUnexpected(json::Error::kWrongType);
    }

    // As<bool>
    template<typename T = bool>
    std::enable_if_t<std::is_same_v<T, bool>, score::Result<bool>>
    As() const noexcept
    {
        if (auto p = std::get_if<bool>(&value_))
        return *p;
        return score::MakeUnexpected(json::Error::kWrongType);
    }

    // As<Number>
    template<typename T = Number>
    std::enable_if_t<std::is_same_v<T, Number>, score::Result<Number>>
    As() const noexcept
    {
        if (auto p = std::get_if<Number>(&value_))
        return *p;
        return score::MakeUnexpected(json::Error::kWrongType);
    }

    // As<T> for arithmetic T ≠ bool
    template<typename T,
            typename = std::enable_if_t<std::is_arithmetic_v<T> && !std::is_same_v<T,bool>, bool>>
    score::Result<T> As() const noexcept
    {
        if (auto p = std::get_if<Number>(&value_))
        return p->As<T>();
        return score::MakeUnexpected(json::Error::kWrongType);
    }

    // As<std::string>
    template<typename T = std::string>
    std::enable_if_t<std::is_same_v<T, std::string>, score::Result<std::reference_wrapper<const std::string>>>
    As() const noexcept
    {
        if (auto p = std::get_if<std::string>(&value_))
        return *p;
        return score::MakeUnexpected(json::Error::kWrongType);
    }

    // As<List>
    template<typename T = List>
    std::enable_if_t<std::is_same_v<T, List>, score::Result<std::reference_wrapper<const List>>>
    As() const noexcept
    {
        if (auto p = std::get_if<List>(&value_))
        return *p;
        return score::MakeUnexpected(json::Error::kWrongType);
    }
    template<typename T = List>
    std::enable_if_t<std::is_same_v<T, List>, score::Result<std::reference_wrapper<List>>>
    As() noexcept
    {
        if (auto p = std::get_if<List>(&value_))
        return *p;
        return score::MakeUnexpected(json::Error::kWrongType);
    }

    // As<Object>
    template<typename T = Object>
    std::enable_if_t<std::is_same_v<T, Object>, score::Result<std::reference_wrapper<const Object>>>
    As() const noexcept
    {
        if (auto p = std::get_if<Object>(&value_))
        return *p;
        return score::MakeUnexpected(json::Error::kWrongType);
    }
    template<typename T = Object>
    std::enable_if_t<std::is_same_v<T, Object>, score::Result<std::reference_wrapper<Object>>>
    As() noexcept
    {
        if (auto p = std::get_if<Object>(&value_))
        return *p;
        return score::MakeUnexpected(json::Error::kWrongType);
    }

    private:
    std::variant<
        Null,
        bool,
        Number,
        std::string,
        List,
        Object
    > value_{Null{}};
};

} // namespace json
} // namespace score

#endif // SCORE_LIB_JSON_INTERNAL_MODEL_ANY_H
