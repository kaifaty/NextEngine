#pragma once

#include <string>

namespace nextengine::nonlocal::fcr {

struct ReferenceSolverReport {
    bool passed = false;
    std::string json;
};

ReferenceSolverReport run_reference_solver_controls();
ReferenceSolverReport run_conditioning_controls();
ReferenceSolverReport run_sissm_controls();

} // namespace nextengine::nonlocal::fcr
