#include "test_basic.hpp"

#include <cassert>
#include <iostream>
#include <kvs.hpp>
#include <kvsbuilder.hpp>
#include <unordered_map>

#include "score/json/json_parser.h"
#include "score/json/json_writer.h"
#include "score/result/result.h"
#include "tracing.hpp"

namespace {

struct KvsParameters {
    uint64_t instance_id;
    std::optional<bool> need_defaults;
    std::optional<bool> need_kvs;
    std::optional<std::string> dir;
};

KvsParameters map_to_params(const std::string& data) {
    using namespace score::json;

    JsonParser parser;
    auto any_res{parser.FromBuffer(data)};
    if (!any_res) {
        throw std::runtime_error{"Failed to parse JSON data"};
    }
    const auto& map_root{any_res.value().As<Object>().value().get().at("kvs_parameters")};
    const auto& obj_root{map_root.As<Object>().value().get()};

    KvsParameters params;
    params.instance_id = obj_root.at("instance_id").As<double>().value();
    if (obj_root.find("need_defaults") != obj_root.end()) {
        params.need_defaults = obj_root.at("need_defaults").As<bool>().value();
    }
    if (obj_root.find("need_kvs") != obj_root.end()) {
        params.need_kvs = obj_root.at("need_kvs").As<bool>().value();
    }
    if (obj_root.find("dir") != obj_root.end()) {
        params.dir = obj_root.at("dir").As<std::string>().value();
    }

    return params;
}

const std::string kTargetName{"cpp_test_scenarios::basic::basic"};

}  // namespace

std::string BasicScenario::name() const { return "basic"; }

void BasicScenario::run(const std::optional<std::string>& input) const {
    using namespace score::mw::per::kvs;

    // Print and parse parameters.
    std::cerr << *input << std::endl;

    auto params{map_to_params(*input)};

    // Set builder parameters.
    InstanceId instance_id{params.instance_id};
    KvsBuilder builder{instance_id};
    if (params.need_defaults.has_value()) {
        builder = builder.need_defaults_flag(*params.need_defaults);
    }
    if (params.need_kvs.has_value()) {
        builder = builder.need_kvs_flag(*params.need_kvs);
    }
    // TODO: handle dir?

    // Create KVS.
    Kvs kvs{*builder.build()};

    // Simple set/get.
    std::string key{"example_key"};
    std::string value{"example_value"};
    auto set_value_result{kvs.set_value(key, KvsValue{value})};
    if (!set_value_result) {
        throw std::runtime_error("Failed to set value");
    }

    auto get_value_result{kvs.get_value(key)};
    if (!get_value_result) {
        throw std::runtime_error{"Failed to get value"};
    }
    auto stored_kvs_value{get_value_result.value()};
    if (stored_kvs_value.getType() != KvsValue::Type::String) {
        throw std::runtime_error{"Invalid value type"};
    }

    auto stored_value{std::get<std::string>(stored_kvs_value.getValue())};
    if (stored_value.compare(value) != 0) {
        throw std::runtime_error("Value mismatch");
    }

    // Trace.
    TRACING_INFO(kTargetName, std::pair{std::string{"example_key"}, stored_value});
}
