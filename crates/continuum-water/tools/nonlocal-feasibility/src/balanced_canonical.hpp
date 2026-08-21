#pragma once

#include "canonical.hpp"

#include <array>
#include <cstddef>
#include <cstdint>
#include <string>
#include <vector>

namespace nextengine::nonlocal::balanced_canonical {

struct ComponentReport {
    std::int64_t exact_target = 0;
    std::int64_t published_sum = 0;
    std::int64_t correction = 0;
    std::size_t corrected_samples = 0;
    long double maximum_local_error_units = 0.0L;
    long double aggregate_error_units = 0.0L;
    long double signed_aggregate_error_units = 0.0L;
    bool local_bound_exact = false;
    bool aggregate_bound_exact = false;
    bool target_exact = false;
    bool corrections_unique = false;
    bool exhaustive_optimal = false;
    bool nearest_exact_when_balanced = false;
};

struct PublishResult {
    canonical::Frame frame;
    std::array<ComponentReport, 3> position;
    std::array<ComponentReport, 3> velocity;
};

PublishResult publish_frame(
    const std::string& profile_sha256,
    const std::string& scenario_sha256,
    std::uint32_t step,
    std::vector<canonical::FloatSample> samples);

void require_unique_correction_capacity(
    std::uint64_t correction_magnitude, std::size_t sample_count);

} // namespace nextengine::nonlocal::balanced_canonical
