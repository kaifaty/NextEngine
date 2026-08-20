#pragma once

#include <string>

namespace nextengine::nonlocal::fcr {

struct DiscriminatorReport {
    bool passed = false;
    std::string json;
};

DiscriminatorReport run_pair_pressure_discriminator();

} // namespace nextengine::nonlocal::fcr
