#pragma once

#include <string>

namespace nextengine::nonlocal::fcr {

struct ReferenceAttestationReport {
    bool passed = false;
    std::string json;
};

ReferenceAttestationReport run_reference_attestation_controls();

} // namespace nextengine::nonlocal::fcr

