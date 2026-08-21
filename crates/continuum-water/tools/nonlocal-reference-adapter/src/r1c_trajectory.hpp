#pragma once

#include "contact_adapter.hpp"

#include <string_view>

namespace nextengine::nonlocal_reference {

AdapterRun run_r1c_trajectory(std::string_view scenario_id, std::string_view output_dir);
AdapterRun run_r1c_trajectory_diagnostic(
    std::string_view scenario_id,
    std::string_view output_dir);

} // namespace nextengine::nonlocal_reference
