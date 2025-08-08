#pragma once

#include <optional>
#include <string>

#include "scenario.hpp"

class BasicScenario final : public Scenario {
   public:
    ~BasicScenario() final = default;

    std::string name() const final;

    void run(const std::optional<std::string>& input) const final;
};
