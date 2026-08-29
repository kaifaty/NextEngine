#pragma once

#include <string>

namespace nextengine::nonlocal {

struct CpuBoundaryDiscriminatorReport {
    bool passed = false;
    std::string json;
};

CpuBoundaryDiscriminatorReport run_cpu_boundary_discriminator();

} // namespace nextengine::nonlocal
