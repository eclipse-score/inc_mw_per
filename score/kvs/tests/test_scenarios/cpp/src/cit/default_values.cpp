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

#include "default_values.hpp"
#include "helpers/helpers.hpp"
#include "helpers/kvs_instance.hpp"
#include "helpers/kvs_parameters.hpp"
#include "tracing.hpp"
#include <sstream>

using namespace score::mw::per::kvs;

namespace
{
const std::string kTargetName{"cpp_test_scenarios::cit::default_values"};

/**
 * Helper to log key/value state in a format parsable by Python tests.
 *
 * @param key The key being queried or modified in the KVS.
 * @param value_is_default String encoding whether the current value matches the
 * default ("Ok(true)", "Ok(false)", or error string).
 * @param default_value String encoding the default value for the key (e.g.,
 * "Ok(F64(...))" or error string).
 * @param current_value String encoding the current value for the key (e.g.,
 * "Ok(F64(...))" or error string).
 *
 * This function emits logs in a structured format so that the Python test suite
 * can parse and validate scenario output.
 */
static void info_log(const std::string& key,
                     const std::string& value_is_default,
                     const std::string& default_value,
                     const std::string& current_value)
{
    TRACING_INFO(kTargetName,
                 std::pair{std::string{"key"}, key},
                 std::pair{std::string{"value_is_default"}, value_is_default},
                 std::pair{std::string{"default_value"}, default_value},
                 std::pair{std::string{"current_value"}, current_value});
}

/**
 * Overload of info_log for logging key/value state without a default_value field.
 *
 * @param key The key being queried or modified in the KVS.
 * @param value_is_default String encoding whether the current value matches the default.
 * @param current_value String encoding the current value for the key (e.g.,
 * "Ok(F64(...))" or an error string such as "Err(KeyNotFound)").
 *
 * This function emits logs in a structured format so that the Python test suite
 * can parse and validate scenario output.
 */
static void info_log(const std::string& key, const std::string& value_is_default, const std::string& current_value)
{
    TRACING_INFO(kTargetName,
                 std::pair{std::string{"key"}, key},
                 std::pair{std::string{"value_is_default"}, value_is_default},
                 std::pair{std::string{"current_value"}, current_value});
}

std::string get_value_is_default(const Kvs& kvs, const std::string& key)
{
    auto result{kvs.is_value_default(key)};
    if (result.has_value())
    {
        if (result.value())
        {
            return std::string{"Ok(true)"};
        }
        else
        {
            return std::string{"Ok(false)"};
        }
    }
    else
    {
        return std::string{"Err(KeyNotFound)"};
    }
}

std::string get_default_value(Kvs& kvs, const std::string& key)
{
    auto result{kvs.get_default_value(key)};
    if (result.has_value() && result.value().getType() == KvsValue::Type::f64)
    {
        std::ostringstream oss;
        oss.precision(1);
        oss << std::fixed << std::get<double>(result.value().getValue());
        return std::string{"Ok(F64(" + oss.str() + "))"};
    }
    else
    {
        return std::string{"Err(KeyNotFound)"};
    }
}

std::string get_current_value(Kvs& kvs, const std::string& key)
{
    auto result{kvs.get_value(key)};
    if (result.has_value() && result.value().getType() == KvsValue::Type::f64)
    {
        std::ostringstream oss;
        oss.precision(1);
        oss << std::fixed << std::get<double>(result.value().getValue());
        return std::string{"Ok(F64(" + oss.str() + "))"};
    }
    else
    {
        return std::string{"Err(KeyNotFound)"};
    }
}

}  // namespace

class DefaultValues final : public Scenario
{
  public:
    ~DefaultValues() final = default;

    std::string name() const final
    {
        return "default_values";
    }

    void run(const std::string& input) const final
    {
        // Key used for tests.
        std::string key{"test_number"};

        // Create KVS instance with provided params.
        auto params{KvsParameters::from_json(input)};
        {
            auto kvs{kvs_instance(params)};

            // Get current value parameters.
            std::string value_is_default{get_value_is_default(kvs, key)};
            std::string default_value{get_default_value(kvs, key)};
            std::string current_value{get_current_value(kvs, key)};

            info_log(key, value_is_default, default_value, current_value);

            // Set value and check value parameters.
            auto set_result{kvs.set_value(key, KvsValue{432.1})};
            if (!set_result)
            {
                throw std::runtime_error{"Failed to set value"};
            }

            // Flush KVS.
            auto flush_result{kvs.flush()};
            if (!flush_result)
            {
                throw std::runtime_error{"Failed to flush"};
            }
        }

        // Flush and reopen KVS instance to ensure persistency.
        {
            auto kvs{kvs_instance(params)};

            // Get current value parameters.
            std::string value_is_default{get_value_is_default(kvs, key)};
            std::string default_value{get_default_value(kvs, key)};
            std::string current_value{get_current_value(kvs, key)};

            info_log(key, value_is_default, default_value, current_value);
        }
    }
};

class RemoveKey final : public Scenario
{
  public:
    ~RemoveKey() final = default;

    std::string name() const final
    {
        return "remove_key";
    }

    void run(const std::string& input) const final
    {
        // Key used for tests.
        std::string key{"test_number"};

        // Create KVS instance with provided params.
        auto params{KvsParameters::from_json(input)};
        auto kvs{kvs_instance(params)};

        // Get value parameters before set.
        {
            std::string value_is_default{get_value_is_default(kvs, key)};
            std::string default_value{get_default_value(kvs, key)};
            std::string current_value{get_current_value(kvs, key)};

            info_log(key, value_is_default, default_value, current_value);
        }

        // Get value parameters after set.
        auto set_result{kvs.set_value(key, KvsValue{432.1})};
        if (!set_result)
        {
            throw std::runtime_error{"Failed to set value"};
        }

        // Get current value parameters.
        {
            std::string value_is_default{get_value_is_default(kvs, key)};
            std::string default_value{get_default_value(kvs, key)};
            std::string current_value{get_current_value(kvs, key)};

            info_log(key, value_is_default, default_value, current_value);
        }

        // Get value parameters after remove.
        {
            auto remove_result{kvs.remove_key(key)};
            if (!remove_result)
            {
                throw std::runtime_error{"Failed to remove key"};
            }

            std::string value_is_default{get_value_is_default(kvs, key)};
            std::string default_value{get_default_value(kvs, key)};
            std::string current_value{get_current_value(kvs, key)};

            info_log(key, value_is_default, default_value, current_value);
        }
    }
};

class ResetAllKeys final : public Scenario
{
  public:
    ~ResetAllKeys() final = default;

    std::string name() const final
    {
        return "reset_all_keys";
    }

    void run(const std::string& input) const final
    {
        const int num_values{5};

        // Create KVS instance with provided params.
        auto params{KvsParameters::from_json(input)};
        auto kvs{kvs_instance(params)};

        // List of keys and corresponding values.
        std::vector<std::pair<std::string, double>> key_values;
        for (int i{0}; i < num_values; ++i)
        {
            key_values.emplace_back("test_number_" + std::to_string(i), 123.4 * i);
        }

        // Set non-default values.
        for (const auto& [key, value] : key_values)
        {
            // Get value before set.
            {
                const std::string value_is_default{get_value_is_default(kvs, key)};
                const std::string current_value{get_current_value(kvs, key)};

                info_log(key, value_is_default, current_value);
            }

            // Set value.
            auto set_result{kvs.set_value(key, KvsValue{value})};
            if (!set_result)
            {
                throw std::runtime_error{"Failed to set value"};
            }

            // Get value after set.
            {
                const std::string value_is_default{get_value_is_default(kvs, key)};
                const std::string current_value{get_current_value(kvs, key)};

                info_log(key, value_is_default, current_value);
            }
        }

        // Reset.
        auto reset_result{kvs.reset()};
        if (!reset_result)
        {
            throw std::runtime_error{"Failed to reset KVS instance"};
        }

        // Get value parameters after reset.
        for (const auto& [key, _] : key_values)
        {
            const std::string value_is_default{get_value_is_default(kvs, key)};
            const std::string current_value{get_current_value(kvs, key)};

            info_log(key, value_is_default, current_value);
        }
    }
};

class ResetSingleKey final : public Scenario
{
  public:
    ~ResetSingleKey() final = default;

    std::string name() const final
    {
        return "reset_single_key";
    }

    void run(const std::string& input) const final
    {
        const int num_values{5};
        const int reset_index{2};

        // Create KVS instance with provided params.
        auto params{KvsParameters::from_json(input)};
        auto kvs{kvs_instance(params)};

        // List of keys and corresponding values.
        std::vector<std::pair<std::string, double>> key_values;
        for (int i{0}; i < num_values; ++i)
        {
            key_values.emplace_back("test_number_" + std::to_string(i), 123.4 * i);
        }

        // Set non-default values.
        for (const auto& [key, value] : key_values)
        {
            // Get value parameters before set.
            {
                const std::string value_is_default{get_value_is_default(kvs, key)};
                const std::string current_value{get_current_value(kvs, key)};

                info_log(key, value_is_default, current_value);
            }

            // Set value.
            auto set_result{kvs.set_value(key, KvsValue{value})};
            if (!set_result)
            {
                throw std::runtime_error{"Failed to set value"};
            }

            // Get value parameters after set.
            {
                const std::string value_is_default{get_value_is_default(kvs, key)};
                const std::string current_value{get_current_value(kvs, key)};

                info_log(key, value_is_default, current_value);
            }
        }

        // Reset single key.
        auto reset_result{kvs.reset_key(key_values[reset_index].first)};
        if (!reset_result)
        {
            throw std::runtime_error{"Failed to reset key"};
        }

        // Use KVS APIs to get value_is_default and current_value after reset
        for (const auto& [key, value] : key_values)
        {
            const std::string value_is_default{get_value_is_default(kvs, key)};
            const std::string current_value{get_current_value(kvs, key)};
            info_log(key, value_is_default, current_value);
        }
    }
};

class Checksum final : public Scenario
{
  public:
    ~Checksum() final = default;

    std::string name() const final
    {
        return "checksum";
    }

    void run(const std::string& input) const final
    {
        // Create KVS instance with provided params.
        auto params{KvsParameters::from_json(input)};
        auto working_dir{*params.dir};
        std::string kvs_path, hash_path;
        {
            // Create instance, flush, store paths to files, close instance.
            auto kvs{kvs_instance(params)};
            auto flush_result{kvs.flush()};
            if (!flush_result)
            {
                throw std::runtime_error{"Failed to flush"};
            }
            std::tie(kvs_path, hash_path) = kvs_hash_paths(working_dir, params.instance_id, SnapshotId{0});
        }

        TRACING_INFO(
            kTargetName, std::pair{std::string{"kvs_path"}, kvs_path}, std::pair{std::string{"hash_path"}, hash_path});
    }
};

// Default values group
ScenarioGroup::Ptr default_values_group()
{
    return ScenarioGroup::Ptr{new ScenarioGroupImpl{"default_values",
                                                    {std::make_shared<DefaultValues>(),
                                                     std::make_shared<RemoveKey>(),
                                                     std::make_shared<ResetAllKeys>(),
                                                     std::make_shared<ResetSingleKey>(),
                                                     std::make_shared<Checksum>()},
                                                    {}}};
}
