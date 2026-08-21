#pragma once

#include <string>
#include <string_view>

namespace nextengine::nonlocal_reference_reader {

struct ReaderRun {
    bool passed = false;
    std::string report;
};

ReaderRun run_profile_self_test();
ReaderRun run_attestation(std::string_view artifact_root);
ReaderRun reject_unknown_argument();

} // namespace nextengine::nonlocal_reference_reader
