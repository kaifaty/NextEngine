#pragma once

#include "contact_adapter.hpp"

#include <array>
#include <string>
#include <string_view>
#include <vector>

namespace nextengine::nonlocal_reference {

AdapterRun run_r1c_manifest_preflight(bool force_manifest_mismatch);

struct R1CScenarioData {
    std::string id;
    std::string manifest;
    double x_max;
    bool orifice;
    std::vector<std::array<double, 3>> fluid_positions;
    std::vector<std::array<double, 3>> boundary_positions;
};

R1CScenarioData build_r1c_scenario(std::string_view scenario_id);
std::string_view r1c_profile_identity_projection();

} // namespace nextengine::nonlocal_reference
