#include "canonical.hpp"

#include "sha256.hpp"
#include "term_controls.hpp"

#include <cmath>
#include <cstring>
#include <iostream>
#include <limits>
#include <stdexcept>
#include <string>
#include <vector>

namespace {

using nextengine::nonlocal::canonical::Error;

constexpr const char* PROFILE_SHA256 =
    "624678f6ad4dbf2d3657b30880ad1cff2445d0348194549b37c71c1e55804327";
constexpr const char* SCENARIO_SHA256 =
    "1111111111111111111111111111111111111111111111111111111111111111";
constexpr const char* FRAME_GOLDEN =
    "07cb5e151046c6e51c6c680d146bb430a6ecdac91715d8ba07daa502e453d075";
constexpr const char* TRAJECTORY_GOLDEN =
    "2791ab24ef76a6466b8548c1a0b3e39076146609d0678914c3c46a2570976bfd";

void require(bool condition, const char* message) {
    if (!condition) {
        throw std::runtime_error(message);
    }
}

template <typename Function>
void require_error(Function function, const char* code) {
    try {
        function();
    } catch (const Error& error) {
        require(error.code() == code, "canonical error code mismatch");
        return;
    }
    throw std::runtime_error("expected canonical error was not raised");
}

double from_bits(std::uint64_t bits) {
    double value = 0.0;
    std::memcpy(&value, &bits, sizeof(value));
    return value;
}

std::string run_self_test() {
    using namespace nextengine::nonlocal::canonical;

    require(quantize_scaled(0.5, 1) == 0, "0.5 tie did not round even");
    require(quantize_scaled(1.5, 1) == 2, "1.5 tie did not round even");
    require(quantize_scaled(2.5, 1) == 2, "2.5 tie did not round even");
    require(quantize_scaled(3.5, 1) == 4, "3.5 tie did not round even");
    require(quantize_scaled(-1.5, 1) == -2, "negative tie did not round even");
    require(quantize_scaled(-2.5, 1) == -2, "negative even tie changed");
    require(quantize_scaled(-0.0, 1000000U) == 0, "negative zero not normalized");
    require(quantize_scaled(from_bits(1U), 1000000000U) == 0,
        "minimum subnormal should quantize to zero");

    require_error([] { quantize_scaled(std::numeric_limits<double>::quiet_NaN(), 1); },
        "NONLOCAL_NONFINITE_VALUE");
    require_error([] { quantize_scaled(std::numeric_limits<double>::infinity(), 1); },
        "NONLOCAL_NONFINITE_VALUE");
    require_error([] { quantize_scaled(std::numeric_limits<double>::max(),
                         std::numeric_limits<std::uint64_t>::max()); },
        "NONLOCAL_NUMERIC_OVERFLOW");

    const double two_pow_63 = from_bits(std::uint64_t(1023 + 63) << 52U);
    require(quantize_scaled(-two_pow_63, 1)
            == std::numeric_limits<std::int64_t>::min(),
        "negative i64 boundary rejected");
    require_error([&] { quantize_scaled(two_pow_63, 1); },
        "NONLOCAL_NUMERIC_OVERFLOW");
    require(quantize_position(16.0000004) == POSITION_LIMIT_UM,
        "position bound tie-safe value rejected");
    require_error([] { quantize_position(16.000001); },
        "NONLOCAL_POSITION_OUT_OF_RANGE");
    require(quantize_velocity(-64.0000004) == -VELOCITY_LIMIT_UM_S,
        "velocity bound tie-safe value rejected");
    require_error([] { quantize_velocity(64.000001); },
        "NONLOCAL_VELOCITY_OUT_OF_RANGE");

    validate_sample_count(MAXIMUM_SAMPLES - 1U);
    validate_sample_count(MAXIMUM_SAMPLES);
    require_error([] { validate_sample_count(MAXIMUM_SAMPLES + 1U); },
        "NONLOCAL_SAMPLE_CAPACITY_EXCEEDED");

    std::vector<FloatSample> unordered = {
        {9U, {-0.000007, 0.000008, -0.000009},
            {0.000010, -0.000011, 0.000012}},
        {2U, {0.000001, -0.000002, 0.000003},
            {0.000004, -0.000005, 0.000006}},
    };
    const Frame published = publish_frame(PROFILE_SHA256, SCENARIO_SHA256, 7U, unordered);
    require(published.samples[0].sample_id == 2U
            && published.samples[1].sample_id == 9U,
        "publication did not canonicalize SampleId order");
    require(published.root_sha256 == FRAME_GOLDEN, "frame golden root mismatch");

    std::vector<FloatSample> duplicate = {unordered[0], unordered[0]};
    require_error([&] { publish_frame(PROFILE_SHA256, SCENARIO_SHA256, 7U, duplicate); },
        "NONLOCAL_DUPLICATE_SAMPLE_ID");
    std::vector<Sample> reverse = {published.samples[1], published.samples[0]};
    require_error([&] { frame_root(PROFILE_SHA256, SCENARIO_SHA256, 7U, reverse); },
        "NONLOCAL_SAMPLE_ORDER_INVALID");

    const std::string second_root(64U, '2');
    const std::string trajectory = trajectory_root(
        PROFILE_SHA256, SCENARIO_SHA256, {published.root_sha256, second_root});
    require(trajectory == TRAJECTORY_GOLDEN, "trajectory golden root mismatch");
    require_error([&] { trajectory_root(PROFILE_SHA256, SCENARIO_SHA256, {}); },
        "NONLOCAL_ROOT_INVALID");
    require_error([&] { trajectory_root("bad", SCENARIO_SHA256, {FRAME_GOLDEN}); },
        "NONLOCAL_ROOT_INVALID");

    const std::string result_material = std::string(PROFILE_SHA256) + '|'
        + FRAME_GOLDEN + '|' + TRAJECTORY_GOLDEN
        + "|ties|subnormal|nonfinite|overflow|bounds|capacity|order|duplicate";
    return std::string("{\"schema\":\"nextengine.nonlocal.npr1_canonical_self_test.v1\"")
        + ",\"status\":\"PASS\",\"profile_sha256\":\"" + PROFILE_SHA256
        + "\",\"numeric_profile\":{\"precision\":\"ieee754_binary64\""
          ",\"rounding\":\"nearest_ties_to_even\",\"fma\":\"forbidden\"}"
          ",\"cases\":{\"ties\":true,\"negative_zero\":true"
          ",\"subnormal\":true,\"nonfinite\":true,\"overflow\":true"
          ",\"lab_bounds\":true,\"sample_capacity\":true"
          ",\"sample_order\":true,\"duplicate_id\":true"
          ",\"root_hex_validation\":true}"
          ",\"frame_golden_sha256\":\"" + FRAME_GOLDEN
        + "\",\"trajectory_golden_sha256\":\"" + TRAJECTORY_GOLDEN
        + "\",\"runtime_authority\":false,\"npr2_authorized\":false"
          ",\"result_sha256\":\""
        + nextengine::nonlocal::sha256_hex(result_material) + "\"}";
}

} // namespace

int main(int argc, char** argv) {
    try {
        if (argc != 2) {
            std::cerr << "usage: nonlocal-npr1-canonical "
                         "--self-test|--term-self-test\n";
            return 2;
        }
        const std::string command = argv[1];
        if (command == "--self-test") {
            std::cout << run_self_test() << '\n';
            return 0;
        }
        if (command == "--term-self-test") {
            const nextengine::nonlocal::npr1::TermControlReport report =
                nextengine::nonlocal::npr1::run_term_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        std::cerr << "usage: nonlocal-npr1-canonical "
                     "--self-test|--term-self-test\n";
        return 2;
    } catch (const std::exception& error) {
        std::cerr << "nonlocal-npr1-canonical: " << error.what() << '\n';
        return 1;
    }
}
