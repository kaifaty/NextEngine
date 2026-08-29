#pragma once

#include <string>
#include <string_view>

namespace nextengine::nonlocal_reference_slice {

struct SliceRun {
    bool passed = false;
    std::string report;
};

SliceRun run_first_output(std::string_view artifact_root);
SliceRun run_initial_binary64(std::string_view artifact_root);
SliceRun reject_unknown_argument();

} // namespace nextengine::nonlocal_reference_slice
