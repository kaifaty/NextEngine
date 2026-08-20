#pragma once

#include <string>

namespace nextengine::nonlocal::fcr {

struct FormulaReport {
    bool passed = false;
    std::string json;
};

FormulaReport run_formula_controls();

} // namespace nextengine::nonlocal::fcr
