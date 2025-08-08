#include <iostream>
#include <memory>
#include <string>
#include <vector>

#include "cli.hpp"
#include "scenario.hpp"
#include "test_basic.hpp"
#include "test_context.hpp"

int main(int argc, char** argv) {
    try {
        std::vector<std::string> raw_arguments{argv, argv + argc};

        // Basic group.
        Scenario::Ptr basic_scenario{new BasicScenario{}};
        ScenarioGroup::Ptr basic_group{new ScenarioGroupImpl{"basic", {basic_scenario}, {}}};

        // Root group.
        ScenarioGroup::Ptr root_group{new ScenarioGroupImpl{"root", {}, {basic_group}}};

        // Run.
        TestContext test_context{root_group};
        run_cli_app(raw_arguments, test_context);
    } catch (const std::exception& ex) {
        std::cerr << ex.what() << std::endl;
        return 1;
    }
}
