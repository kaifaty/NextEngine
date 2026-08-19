#pragma once

#include "oracle.hpp"
#include "profiles.hpp"

#include <string>

namespace nextengine::nonlocal {

struct CommandReport {
    bool passed = false;
    std::string json;
};

CommandReport run_cuda_self_test();
CommandReport run_cuda_check(const Profile& profile, int iterations);
CommandReport run_cuda_benchmark(const Profile& profile, int warmup, int runs);

} // namespace nextengine::nonlocal
