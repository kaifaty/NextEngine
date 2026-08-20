#pragma once

#include <string>

namespace nextengine::nonlocal::fcr {

struct SplitBoundaryReport {
    bool passed = false;
    std::string json;
};

SplitBoundaryReport run_split_static_boundary_controls();

} // namespace nextengine::nonlocal::fcr
