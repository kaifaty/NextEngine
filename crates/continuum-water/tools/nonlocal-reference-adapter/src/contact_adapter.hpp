#pragma once

#include <array>
#include <cstdint>
#include <string>

namespace nextengine::nonlocal_reference {

enum class PreflightMutation {
    None,
    RoundDown,
    FtzOn,
};

struct AdapterRun {
    bool passed;
    std::string report;
};

struct R1CContactProjection {
    std::array<double, 3> velocity;
    std::array<double, 3> position;
    std::array<std::uint32_t, 25> feature_counts;
};

AdapterRun run_contact_adapter(PreflightMutation mutation);
AdapterRun reject_unknown_argument();
std::string process_preflight_failure();
R1CContactProjection project_r1c_contact(
    const std::array<double, 3> &start,
    const std::array<double, 3> &velocity,
    double x_max,
    bool orifice);

} // namespace nextengine::nonlocal_reference
