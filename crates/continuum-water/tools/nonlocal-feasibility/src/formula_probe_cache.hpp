#pragma once

#include "formula_probe_api.hpp"

#include <string>

namespace nextengine::nonlocal::fcr {

void write_formula_probe_parent_fixture(
    const std::string& path, const FormulaProbeParentFixture& fixture);
FormulaProbeParentFixture read_formula_probe_parent_fixture(
    const std::string& path);

} // namespace nextengine::nonlocal::fcr
