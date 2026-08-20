#pragma once

#include <array>
#include <cstddef>
#include <cstdint>
#include <stdexcept>
#include <string>
#include <vector>

namespace nextengine::nonlocal::canonical {

constexpr std::size_t MAXIMUM_SAMPLES = 50000U;
constexpr std::int64_t POSITION_LIMIT_UM = 16000000;
constexpr std::int64_t VELOCITY_LIMIT_UM_S = 64000000;

class Error final : public std::runtime_error {
public:
    Error(std::string code, std::string message);

    const std::string& code() const noexcept;

private:
    std::string code_;
};

struct FloatSample {
    std::uint32_t sample_id = 0;
    std::array<double, 3> position_m{};
    std::array<double, 3> velocity_m_s{};
};

struct Sample {
    std::uint32_t sample_id = 0;
    std::array<std::int64_t, 3> position_um{};
    std::array<std::int64_t, 3> velocity_um_s{};
};

struct Frame {
    std::uint32_t step = 0;
    std::vector<Sample> samples;
    std::string root_sha256;
};

void validate_sample_count(std::size_t count, std::size_t maximum = MAXIMUM_SAMPLES);
std::int64_t quantize_scaled(double value, std::uint64_t scale);
std::int64_t quantize_position(double value);
std::int64_t quantize_velocity(double value);
std::int64_t quantize_ppb(double value);

Frame publish_frame(
    const std::string& profile_sha256,
    const std::string& scenario_sha256,
    std::uint32_t step,
    std::vector<FloatSample> samples);

std::string frame_root(
    const std::string& profile_sha256,
    const std::string& scenario_sha256,
    std::uint32_t step,
    const std::vector<Sample>& samples);

std::string trajectory_root(
    const std::string& profile_sha256,
    const std::string& scenario_sha256,
    const std::vector<std::string>& frame_roots);

} // namespace nextengine::nonlocal::canonical
