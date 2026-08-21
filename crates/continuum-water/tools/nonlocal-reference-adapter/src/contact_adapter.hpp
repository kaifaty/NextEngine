#pragma once

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

AdapterRun run_contact_adapter(PreflightMutation mutation);
AdapterRun reject_unknown_argument();
std::string process_preflight_failure();

} // namespace nextengine::nonlocal_reference
