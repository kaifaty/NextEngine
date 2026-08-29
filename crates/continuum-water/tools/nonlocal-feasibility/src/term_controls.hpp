#pragma once

#include <string>

namespace nextengine::nonlocal::npr1 {

struct TermControlReport {
    bool passed = false;
    std::string json;
};

TermControlReport run_term_controls();

} // namespace nextengine::nonlocal::npr1
