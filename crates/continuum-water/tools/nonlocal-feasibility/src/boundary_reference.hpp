#pragma once

#include <string>

namespace nextengine::nonlocal::fcr {

struct SplitBoundaryReport {
    bool passed = false;
    std::string json;
};

SplitBoundaryReport run_split_static_boundary_controls();
SplitBoundaryReport run_boundary_composition_smoke_controls();
SplitBoundaryReport run_boundary_reaction_accuracy_controls();
SplitBoundaryReport run_displacement_ownership_controls();

} // namespace nextengine::nonlocal::fcr
