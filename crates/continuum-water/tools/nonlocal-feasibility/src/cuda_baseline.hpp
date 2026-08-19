#pragma once

#include "oracle.hpp"
#include "profiles.hpp"

#include <string>

namespace nextengine::nonlocal {

struct CommandReport {
    bool passed = false;
    std::string json;
};

enum class AccumulationMode {
    SourceAtomicV0,
    GatherDirectedR0,
};

const char* accumulation_identity(AccumulationMode mode);
AccumulationMode parse_accumulation_identity(const std::string& identity);

CommandReport run_cuda_self_test(
    AccumulationMode mode = AccumulationMode::SourceAtomicV0);
CommandReport run_cuda_check(
    const Profile& profile,
    int iterations,
    AccumulationMode mode = AccumulationMode::SourceAtomicV0);
CommandReport run_cuda_repeatability(
    const Profile& profile,
    int iterations,
    int runs,
    AccumulationMode mode);
CommandReport run_cuda_benchmark(
    const Profile& profile,
    int warmup,
    int runs,
    AccumulationMode mode = AccumulationMode::SourceAtomicV0);

} // namespace nextengine::nonlocal
