#include "balanced_canonical.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstring>
#include <limits>
#include <numeric>
#include <stdexcept>
#include <utility>
#include <vector>

namespace nextengine::nonlocal::balanced_canonical {
namespace {

#if defined(__SIZEOF_INT128__)
__extension__ typedef unsigned __int128 uint128_t;
#else
#error "balanced canonical research requires a checked unsigned 128-bit integer"
#endif

constexpr std::size_t LIMBS = 20U;
constexpr unsigned DENOMINATOR_SHIFT = 1074U;
constexpr std::uint64_t SCALE = 1000000U;
constexpr const char* NONFINITE = "NONLOCAL_NONFINITE_VALUE";
constexpr const char* POSITION_RANGE = "NONLOCAL_POSITION_OUT_OF_RANGE";
constexpr const char* VELOCITY_RANGE = "NONLOCAL_VELOCITY_OUT_OF_RANGE";
constexpr const char* DUPLICATE_ID = "NONLOCAL_DUPLICATE_SAMPLE_ID";
constexpr const char* INFEASIBLE =
    "NONLOCAL_CONSERVATION_APPORTIONMENT_INFEASIBLE";

using Magnitude = std::array<std::uint64_t, LIMBS>;

struct SignedMagnitude {
    bool negative = false;
    Magnitude magnitude{};
};

bool is_zero(const Magnitude& value) {
    return std::all_of(value.begin(), value.end(),
        [](std::uint64_t limb) { return limb == 0U; });
}

int compare_magnitude(const Magnitude& lhs, const Magnitude& rhs) {
    for (std::size_t i = LIMBS; i-- > 0U;) {
        if (lhs[i] != rhs[i]) {
            return lhs[i] < rhs[i] ? -1 : 1;
        }
    }
    return 0;
}

Magnitude add_magnitude(const Magnitude& lhs, const Magnitude& rhs) {
    Magnitude result{};
    uint128_t carry = 0;
    for (std::size_t i = 0; i < LIMBS; ++i) {
        const uint128_t sum = static_cast<uint128_t>(lhs[i])
            + static_cast<uint128_t>(rhs[i]) + carry;
        result[i] = static_cast<std::uint64_t>(sum);
        carry = sum >> 64U;
    }
    if (carry != 0U) {
        throw canonical::Error(INFEASIBLE,
            "exact aggregate superaccumulator overflow");
    }
    return result;
}

Magnitude subtract_magnitude(
    const Magnitude& larger, const Magnitude& smaller) {
    Magnitude result{};
    std::uint64_t borrow = 0U;
    for (std::size_t i = 0; i < LIMBS; ++i) {
        const uint128_t subtrahend =
            static_cast<uint128_t>(smaller[i]) + borrow;
        const uint128_t minuend = static_cast<uint128_t>(larger[i]);
        result[i] = static_cast<std::uint64_t>(minuend - subtrahend);
        borrow = minuend < subtrahend ? 1U : 0U;
    }
    if (borrow != 0U) {
        throw canonical::Error(INFEASIBLE,
            "exact aggregate superaccumulator underflow");
    }
    return result;
}

SignedMagnitude normalized(SignedMagnitude value) {
    if (is_zero(value.magnitude)) {
        value.negative = false;
    }
    return value;
}

SignedMagnitude add_signed(
    const SignedMagnitude& lhs, const SignedMagnitude& rhs) {
    if (lhs.negative == rhs.negative) {
        return normalized({lhs.negative,
            add_magnitude(lhs.magnitude, rhs.magnitude)});
    }
    const int comparison = compare_magnitude(
        lhs.magnitude, rhs.magnitude);
    if (comparison == 0) {
        return {};
    }
    if (comparison > 0) {
        return normalized({lhs.negative,
            subtract_magnitude(lhs.magnitude, rhs.magnitude)});
    }
    return normalized({rhs.negative,
        subtract_magnitude(rhs.magnitude, lhs.magnitude)});
}

SignedMagnitude negate(SignedMagnitude value) {
    if (!is_zero(value.magnitude)) {
        value.negative = !value.negative;
    }
    return value;
}

int compare_signed(
    const SignedMagnitude& lhs, const SignedMagnitude& rhs) {
    if (lhs.negative != rhs.negative) {
        return lhs.negative ? -1 : 1;
    }
    const int comparison = compare_magnitude(
        lhs.magnitude, rhs.magnitude);
    return lhs.negative ? -comparison : comparison;
}

void place_word(Magnitude& result, std::uint64_t word, unsigned shift) {
    if (word == 0U) {
        return;
    }
    const std::size_t limb = shift / 64U;
    const unsigned offset = shift % 64U;
    if (limb >= LIMBS) {
        throw canonical::Error(INFEASIBLE,
            "exact dyadic value exceeds the fixed superaccumulator");
    }
    result[limb] |= word << offset;
    if (offset != 0U) {
        if (limb + 1U >= LIMBS) {
            throw canonical::Error(INFEASIBLE,
                "exact dyadic value exceeds the fixed superaccumulator");
        }
        result[limb + 1U] |= word >> (64U - offset);
    }
}

SignedMagnitude exact_scaled(double value) {
    if (!std::isfinite(value)) {
        throw canonical::Error(NONFINITE,
            "cannot apportion a nonfinite binary64 value");
    }
    std::uint64_t bits = 0U;
    static_assert(sizeof(bits) == sizeof(value));
    std::memcpy(&bits, &value, sizeof(bits));
    const bool negative = (bits >> 63U) != 0U;
    const int raw_exponent = static_cast<int>(
        (bits >> 52U) & 0x7ffU);
    const std::uint64_t fraction =
        bits & 0x000fffffffffffffULL;
    if (raw_exponent == 0 && fraction == 0U) {
        return {};
    }
    const std::uint64_t significand = raw_exponent == 0
        ? fraction : ((std::uint64_t(1) << 52U) | fraction);
    const int exponent = raw_exponent == 0
        ? -1074 : raw_exponent - 1023 - 52;
    const int shift = exponent + static_cast<int>(DENOMINATOR_SHIFT);
    if (shift < 0) {
        throw canonical::Error(INFEASIBLE,
            "binary64 dyadic denominator exceeds the fixed denominator");
    }
    const uint128_t product = static_cast<uint128_t>(significand)
        * static_cast<uint128_t>(SCALE);
    SignedMagnitude result;
    result.negative = negative;
    place_word(result.magnitude,
        static_cast<std::uint64_t>(product),
        static_cast<unsigned>(shift));
    place_word(result.magnitude,
        static_cast<std::uint64_t>(product >> 64U),
        static_cast<unsigned>(shift) + 64U);
    return normalized(result);
}

SignedMagnitude exact_integer(std::int64_t value) {
    SignedMagnitude result;
    result.negative = value < 0;
    const std::uint64_t magnitude = value < 0
        ? std::uint64_t(0) - static_cast<std::uint64_t>(value)
        : static_cast<std::uint64_t>(value);
    place_word(result.magnitude, magnitude, DENOMINATOR_SHIFT);
    return normalized(result);
}

bool any_bits_below(const Magnitude& value, unsigned bit) {
    const std::size_t complete_limbs = bit / 64U;
    for (std::size_t i = 0; i < complete_limbs; ++i) {
        if (value[i] != 0U) {
            return true;
        }
    }
    const unsigned remaining = bit % 64U;
    if (remaining == 0U || complete_limbs >= LIMBS) {
        return false;
    }
    const std::uint64_t mask = (std::uint64_t(1) << remaining) - 1U;
    return (value[complete_limbs] & mask) != 0U;
}

bool bit_at(const Magnitude& value, unsigned bit) {
    const std::size_t limb = bit / 64U;
    return limb < LIMBS
        && ((value[limb] >> (bit % 64U)) & 1U) != 0U;
}

std::uint64_t extract_u64(const Magnitude& value, unsigned shift) {
    const std::size_t limb = shift / 64U;
    const unsigned offset = shift % 64U;
    if (limb >= LIMBS) {
        return 0U;
    }
    std::uint64_t result = value[limb] >> offset;
    if (offset != 0U && limb + 1U < LIMBS) {
        result |= value[limb + 1U] << (64U - offset);
    }
    return result;
}

bool any_bits_at_or_above(const Magnitude& value, unsigned bit) {
    const std::size_t limb = bit / 64U;
    const unsigned offset = bit % 64U;
    if (limb < LIMBS && offset != 0U
        && (value[limb] >> offset) != 0U) {
        return true;
    }
    const std::size_t start = limb + (offset == 0U ? 0U : 1U);
    for (std::size_t i = start; i < LIMBS; ++i) {
        if (value[i] != 0U) {
            return true;
        }
    }
    return false;
}

std::int64_t round_exact(const SignedMagnitude& value) {
    if (any_bits_at_or_above(
            value.magnitude, DENOMINATOR_SHIFT + 63U)) {
        throw canonical::Error(INFEASIBLE,
            "exact aggregate target exceeds signed i64");
    }
    std::uint64_t quotient = extract_u64(
        value.magnitude, DENOMINATOR_SHIFT);
    const bool half = bit_at(
        value.magnitude, DENOMINATOR_SHIFT - 1U);
    const bool below_half = any_bits_below(
        value.magnitude, DENOMINATOR_SHIFT - 1U);
    if (half && (below_half || (quotient & 1U) != 0U)) {
        ++quotient;
    }
    const std::uint64_t negative_limit = std::uint64_t(1) << 63U;
    if (value.negative) {
        if (quotient > negative_limit) {
            throw canonical::Error(INFEASIBLE,
                "negative aggregate target exceeds signed i64");
        }
        if (quotient == negative_limit) {
            return std::numeric_limits<std::int64_t>::min();
        }
        return -static_cast<std::int64_t>(quotient);
    }
    if (quotient >= negative_limit) {
        throw canonical::Error(INFEASIBLE,
            "positive aggregate target exceeds signed i64");
    }
    return static_cast<std::int64_t>(quotient);
}

Magnitude power_of_two(unsigned bit) {
    Magnitude result{};
    const std::size_t limb = bit / 64U;
    if (limb >= LIMBS) {
        throw canonical::Error(INFEASIBLE,
            "fixed superaccumulator power exceeds capacity");
    }
    result[limb] = std::uint64_t(1) << (bit % 64U);
    return result;
}

long double to_units(const SignedMagnitude& value) {
    long double result = 0.0L;
    for (std::size_t i = LIMBS; i-- > 0U;) {
        result = std::ldexp(result, 64)
            + static_cast<long double>(value.magnitude[i]);
    }
    result = std::ldexp(result,
        -static_cast<int>(DENOMINATOR_SHIFT));
    return value.negative ? -result : result;
}

struct ScalarInput {
    std::uint32_t sample_id = 0U;
    double value = 0.0;
};

struct ScalarResult {
    std::vector<std::int64_t> values;
    ComponentReport report;
};

bool subset_optimal(
    const std::vector<SignedMagnitude>& residuals,
    const std::vector<bool>& corrected,
    std::int64_t correction) {
    if (residuals.size() > 8U) {
        return true;
    }
    const std::size_t count = static_cast<std::size_t>(
        correction < 0 ? -correction : correction);
    SignedMagnitude selected;
    for (std::size_t i = 0; i < residuals.size(); ++i) {
        if (corrected[i]) {
            selected = add_signed(selected, residuals[i]);
        }
    }
    const std::size_t combinations = std::size_t(1) << residuals.size();
    for (std::size_t mask = 0; mask < combinations; ++mask) {
        if (static_cast<std::size_t>(__builtin_popcountll(mask)) != count) {
            continue;
        }
        SignedMagnitude candidate;
        for (std::size_t i = 0; i < residuals.size(); ++i) {
            if ((mask & (std::size_t(1) << i)) != 0U) {
                candidate = add_signed(candidate, residuals[i]);
            }
        }
        const int comparison = compare_signed(candidate, selected);
        if ((correction > 0 && comparison < 0)
            || (correction < 0 && comparison > 0)) {
            return false;
        }
    }
    return true;
}

ScalarResult apportion(
    const std::vector<ScalarInput>& samples,
    bool position) {
    ScalarResult result;
    result.values.resize(samples.size());
    std::vector<SignedMagnitude> exact(samples.size());
    std::vector<SignedMagnitude> residuals(samples.size());
    SignedMagnitude exact_sum;
    std::int64_t nearest_sum = 0;
    for (std::size_t i = 0; i < samples.size(); ++i) {
        result.values[i] = position
            ? canonical::quantize_position(samples[i].value)
            : canonical::quantize_velocity(samples[i].value);
        exact[i] = exact_scaled(samples[i].value);
        exact_sum = add_signed(exact_sum, exact[i]);
        if ((result.values[i] > 0
                && nearest_sum > std::numeric_limits<std::int64_t>::max()
                    - result.values[i])
            || (result.values[i] < 0
                && nearest_sum < std::numeric_limits<std::int64_t>::min()
                    - result.values[i])) {
            throw canonical::Error(INFEASIBLE,
                "nearest aggregate sum exceeds signed i64");
        }
        nearest_sum += result.values[i];
        residuals[i] = add_signed(
            exact_integer(result.values[i]), negate(exact[i]));
    }
    result.report.exact_target = round_exact(exact_sum);
    result.report.correction = result.report.exact_target - nearest_sum;
    const std::uint64_t magnitude = result.report.correction < 0
        ? std::uint64_t(0)
            - static_cast<std::uint64_t>(result.report.correction)
        : static_cast<std::uint64_t>(result.report.correction);
    require_unique_correction_capacity(magnitude, samples.size());
    std::vector<std::size_t> order(samples.size());
    std::iota(order.begin(), order.end(), 0U);
    if (result.report.correction > 0) {
        std::sort(order.begin(), order.end(),
            [&](std::size_t lhs, std::size_t rhs) {
                const int comparison = compare_signed(
                    residuals[lhs], residuals[rhs]);
                return comparison != 0 ? comparison < 0
                    : samples[lhs].sample_id < samples[rhs].sample_id;
            });
    } else if (result.report.correction < 0) {
        std::sort(order.begin(), order.end(),
            [&](std::size_t lhs, std::size_t rhs) {
                const int comparison = compare_signed(
                    residuals[lhs], residuals[rhs]);
                return comparison != 0 ? comparison > 0
                    : samples[lhs].sample_id < samples[rhs].sample_id;
            });
    }
    std::vector<bool> corrected(samples.size(), false);
    for (std::size_t slot = 0;
         slot < static_cast<std::size_t>(magnitude); ++slot) {
        const std::size_t index = order[slot];
        result.values[index] += result.report.correction > 0 ? 1 : -1;
        corrected[index] = true;
    }
    result.report.corrected_samples = static_cast<std::size_t>(magnitude);
    result.report.published_sum = std::accumulate(
        result.values.begin(), result.values.end(), std::int64_t(0));
    result.report.target_exact =
        result.report.published_sum == result.report.exact_target;
    result.report.corrections_unique =
        static_cast<std::size_t>(std::count(
            corrected.begin(), corrected.end(), true)) == magnitude;
    result.report.exhaustive_optimal = subset_optimal(
        residuals, corrected, result.report.correction);
    result.report.nearest_exact_when_balanced =
        result.report.correction != 0 || result.values == [&]() {
            std::vector<std::int64_t> nearest(samples.size());
            for (std::size_t i = 0; i < samples.size(); ++i) {
                nearest[i] = position
                    ? canonical::quantize_position(samples[i].value)
                    : canonical::quantize_velocity(samples[i].value);
            }
            return nearest;
        }();
    const Magnitude unit = power_of_two(DENOMINATOR_SHIFT);
    const Magnitude half = power_of_two(DENOMINATOR_SHIFT - 1U);
    result.report.local_bound_exact = true;
    for (std::size_t i = 0; i < samples.size(); ++i) {
        const SignedMagnitude error = add_signed(
            exact_integer(result.values[i]), negate(exact[i]));
        result.report.maximum_local_error_units = std::max(
            result.report.maximum_local_error_units,
            std::fabs(to_units(error)));
        result.report.local_bound_exact =
            result.report.local_bound_exact
            && compare_magnitude(error.magnitude, unit) < 0;
    }
    const SignedMagnitude aggregate_error = add_signed(
        exact_integer(result.report.published_sum), negate(exact_sum));
    result.report.signed_aggregate_error_units = to_units(aggregate_error);
    result.report.aggregate_error_units = std::fabs(
        result.report.signed_aggregate_error_units);
    result.report.aggregate_bound_exact = compare_magnitude(
        aggregate_error.magnitude, half) <= 0;
    const std::int64_t low = position
        ? -canonical::POSITION_LIMIT_UM : -canonical::VELOCITY_LIMIT_UM_S;
    const std::int64_t high = position
        ? canonical::POSITION_LIMIT_UM : canonical::VELOCITY_LIMIT_UM_S;
    for (std::int64_t value : result.values) {
        if (value < low || value > high) {
            throw canonical::Error(position ? POSITION_RANGE : VELOCITY_RANGE,
                "balanced canonical result exceeds the lab bound");
        }
    }
    return result;
}

} // namespace

void require_unique_correction_capacity(
    std::uint64_t correction_magnitude, std::size_t sample_count) {
    if (correction_magnitude > sample_count) {
        throw canonical::Error(INFEASIBLE,
            "aggregate correction requires repeated sample adjustment");
    }
}

PublishResult publish_frame(
    const std::string& profile_sha256,
    const std::string& scenario_sha256,
    std::uint32_t step,
    std::vector<canonical::FloatSample> samples) {
    canonical::validate_sample_count(samples.size());
    std::sort(samples.begin(), samples.end(),
        [](const canonical::FloatSample& lhs,
           const canonical::FloatSample& rhs) {
            return lhs.sample_id < rhs.sample_id;
        });
    for (std::size_t i = 1; i < samples.size(); ++i) {
        if (samples[i - 1U].sample_id == samples[i].sample_id) {
            throw canonical::Error(DUPLICATE_ID,
                "duplicate SampleId at balanced canonical publication");
        }
    }
    PublishResult result;
    result.frame.step = step;
    result.frame.samples.resize(samples.size());
    for (std::size_t component = 0; component < 3U; ++component) {
        std::vector<ScalarInput> position;
        std::vector<ScalarInput> velocity;
        position.reserve(samples.size());
        velocity.reserve(samples.size());
        for (const canonical::FloatSample& sample : samples) {
            position.push_back({sample.sample_id,
                sample.position_m[component]});
            velocity.push_back({sample.sample_id,
                sample.velocity_m_s[component]});
        }
        const ScalarResult position_result = apportion(position, true);
        const ScalarResult velocity_result = apportion(velocity, false);
        result.position[component] = position_result.report;
        result.velocity[component] = velocity_result.report;
        for (std::size_t i = 0; i < samples.size(); ++i) {
            result.frame.samples[i].sample_id = samples[i].sample_id;
            result.frame.samples[i].position_um[component] =
                position_result.values[i];
            result.frame.samples[i].velocity_um_s[component] =
                velocity_result.values[i];
        }
    }
    result.frame.root_sha256 = canonical::frame_root(
        profile_sha256, scenario_sha256, step, result.frame.samples);
    return result;
}

} // namespace nextengine::nonlocal::balanced_canonical
