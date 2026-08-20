#pragma once

#include <string>

namespace nextengine::nonlocal {

struct CpuTinyCorpusReport {
    bool passed = false;
    std::string json;
};

CpuTinyCorpusReport run_cpu_tiny_physical_corpus();
CpuTinyCorpusReport run_cpu_hydro_remediation();

} // namespace nextengine::nonlocal
