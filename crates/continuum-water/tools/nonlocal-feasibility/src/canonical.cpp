#include "canonical.hpp"

#include "sha256.hpp"

#include <algorithm>
#include <cmath>
#include <cstring>
#include <limits>
#include <string_view>
#include <utility>

namespace nextengine::nonlocal::canonical {
namespace {

#if defined(__SIZEOF_INT128__)
__extension__ typedef unsigned __int128 uint128_t;
#else
#error "NPR1 canonical publication requires a checked unsigned 128-bit integer"
#endif

constexpr const char* NONFINITE = "NONLOCAL_NONFINITE_VALUE";
constexpr const char* OVERFLOW = "NONLOCAL_NUMERIC_OVERFLOW";
constexpr const char* POSITION_RANGE = "NONLOCAL_POSITION_OUT_OF_RANGE";
constexpr const char* VELOCITY_RANGE = "NONLOCAL_VELOCITY_OUT_OF_RANGE";
constexpr const char* SAMPLE_CAPACITY = "NONLOCAL_SAMPLE_CAPACITY_EXCEEDED";
constexpr const char* DUPLICATE_ID = "NONLOCAL_DUPLICATE_SAMPLE_ID";
constexpr const char* SAMPLE_ORDER = "NONLOCAL_SAMPLE_ORDER_INVALID";
constexpr const char* ROOT_INVALID = "NONLOCAL_ROOT_INVALID";
constexpr char FRAME_DOMAIN[] = "nextengine.nonlocal.canonical-frame.v1\0";
constexpr char TRAJECTORY_DOMAIN[] = "nextengine.nonlocal.canonical-trajectory.v1\0";

std::vector<std::uint8_t> decode_root(const std::string& hex) {
    if (hex.size() != 64U) {
        throw Error(ROOT_INVALID, "SHA-256 root must contain exactly 64 hex digits");
    }
    const auto nibble = [](char value) -> std::uint8_t {
        if (value >= '0' && value <= '9') {
            return static_cast<std::uint8_t>(value - '0');
        }
        if (value >= 'a' && value <= 'f') {
            return static_cast<std::uint8_t>(value - 'a' + 10);
        }
        if (value >= 'A' && value <= 'F') {
            return static_cast<std::uint8_t>(value - 'A' + 10);
        }
        throw Error(ROOT_INVALID, "SHA-256 root contains a non-hex digit");
    };
    std::vector<std::uint8_t> bytes(32U);
    for (std::size_t index = 0; index < bytes.size(); ++index) {
        bytes[index] = static_cast<std::uint8_t>(
            (nibble(hex[2U * index]) << 4U) | nibble(hex[2U * index + 1U]));
    }
    return bytes;
}

void append_bytes(std::string& output, const std::vector<std::uint8_t>& bytes) {
    output.append(reinterpret_cast<const char*>(bytes.data()), bytes.size());
}

void append_u32(std::string& output, std::uint32_t value) {
    for (int shift = 0; shift < 32; shift += 8) {
        output.push_back(static_cast<char>((value >> shift) & 0xffU));
    }
}

void append_i64(std::string& output, std::int64_t value) {
    const std::uint64_t bits = static_cast<std::uint64_t>(value);
    for (int shift = 0; shift < 64; shift += 8) {
        output.push_back(static_cast<char>((bits >> shift) & 0xffU));
    }
}

uint128_t round_divide_power_of_two(uint128_t numerator, unsigned shift) {
    if (shift > 128U) {
        return 0;
    }
    if (shift == 128U) {
        const uint128_t half = uint128_t(1) << 127U;
        return numerator > half ? uint128_t(1) : uint128_t(0);
    }
    const uint128_t divisor = uint128_t(1) << shift;
    const uint128_t quotient = numerator / divisor;
    const uint128_t remainder = numerator % divisor;
    const uint128_t half = divisor >> 1U;
    if (remainder > half || (remainder == half && (quotient & 1U) != 0U)) {
        return quotient + 1U;
    }
    return quotient;
}

std::int64_t signed_i64(bool negative, uint128_t magnitude) {
    const uint128_t negative_limit = uint128_t(1) << 63U;
    const uint128_t positive_limit = negative_limit - 1U;
    if (negative) {
        if (magnitude > negative_limit) {
            throw Error(OVERFLOW, "negative publication result is below i64 minimum");
        }
        if (magnitude == negative_limit) {
            return std::numeric_limits<std::int64_t>::min();
        }
        return -static_cast<std::int64_t>(static_cast<std::uint64_t>(magnitude));
    }
    if (magnitude > positive_limit) {
        throw Error(OVERFLOW, "publication result exceeds i64 maximum");
    }
    return static_cast<std::int64_t>(static_cast<std::uint64_t>(magnitude));
}

void validate_strict_order(const std::vector<Sample>& samples) {
    for (std::size_t index = 1; index < samples.size(); ++index) {
        if (samples[index - 1U].sample_id >= samples[index].sample_id) {
            throw Error(SAMPLE_ORDER,
                "canonical samples must be strictly ordered by SampleId");
        }
    }
}

} // namespace

Error::Error(std::string code, std::string message)
    : std::runtime_error(std::move(message)), code_(std::move(code)) {}

const std::string& Error::code() const noexcept { return code_; }

void validate_sample_count(std::size_t count, std::size_t maximum) {
    if (count == 0U || count > maximum) {
        throw Error(SAMPLE_CAPACITY, "canonical sample count is outside the admitted range");
    }
}

std::int64_t quantize_scaled(double value, std::uint64_t scale) {
    if (!std::isfinite(value)) {
        throw Error(NONFINITE, "cannot publish a nonfinite binary64 value");
    }
    if (scale == 0U) {
        throw Error(OVERFLOW, "publication scale must be positive");
    }
    std::uint64_t bits = 0;
    static_assert(sizeof(bits) == sizeof(value));
    std::memcpy(&bits, &value, sizeof(bits));
    const bool negative = (bits >> 63U) != 0U;
    const int raw_exponent = static_cast<int>((bits >> 52U) & 0x7ffU);
    const std::uint64_t fraction = bits & 0x000fffffffffffffULL;
    if (raw_exponent == 0 && fraction == 0U) {
        return 0;
    }
    const std::uint64_t significand = raw_exponent == 0
        ? fraction : ((std::uint64_t(1) << 52U) | fraction);
    const int exponent = raw_exponent == 0 ? -1074 : raw_exponent - 1023 - 52;
    const uint128_t scaled = uint128_t(significand) * uint128_t(scale);
    uint128_t magnitude = 0;
    if (exponent >= 0) {
        const unsigned shift = static_cast<unsigned>(exponent);
        const uint128_t maximum = ~uint128_t(0);
        if (shift >= 128U || scaled > (maximum >> shift)) {
            throw Error(OVERFLOW, "publication left shift overflow");
        }
        magnitude = scaled << shift;
    } else {
        magnitude = round_divide_power_of_two(
            scaled, static_cast<unsigned>(-exponent));
    }
    return signed_i64(negative, magnitude);
}

std::int64_t quantize_position(double value) {
    const std::int64_t result = quantize_scaled(value, 1000000U);
    if (result < -POSITION_LIMIT_UM || result > POSITION_LIMIT_UM) {
        throw Error(POSITION_RANGE, "published position exceeds the lab bound");
    }
    return result;
}

std::int64_t quantize_velocity(double value) {
    const std::int64_t result = quantize_scaled(value, 1000000U);
    if (result < -VELOCITY_LIMIT_UM_S || result > VELOCITY_LIMIT_UM_S) {
        throw Error(VELOCITY_RANGE, "published velocity exceeds the lab bound");
    }
    return result;
}

std::int64_t quantize_ppb(double value) {
    return quantize_scaled(value, 1000000000U);
}

Frame publish_frame(
    const std::string& profile_sha256,
    const std::string& scenario_sha256,
    std::uint32_t step,
    std::vector<FloatSample> samples) {
    validate_sample_count(samples.size());
    std::sort(samples.begin(), samples.end(), [](const FloatSample& left,
                                                 const FloatSample& right) {
        return left.sample_id < right.sample_id;
    });
    Frame frame;
    frame.step = step;
    frame.samples.reserve(samples.size());
    for (const FloatSample& input : samples) {
        if (!frame.samples.empty()
            && frame.samples.back().sample_id == input.sample_id) {
            throw Error(DUPLICATE_ID, "duplicate SampleId at canonical publication");
        }
        Sample output;
        output.sample_id = input.sample_id;
        for (std::size_t component = 0; component < 3U; ++component) {
            output.position_um[component] = quantize_position(input.position_m[component]);
            output.velocity_um_s[component] =
                quantize_velocity(input.velocity_m_s[component]);
        }
        frame.samples.push_back(output);
    }
    frame.root_sha256 =
        frame_root(profile_sha256, scenario_sha256, step, frame.samples);
    return frame;
}

std::string frame_root(
    const std::string& profile_sha256,
    const std::string& scenario_sha256,
    std::uint32_t step,
    const std::vector<Sample>& samples) {
    validate_sample_count(samples.size());
    validate_strict_order(samples);
    std::string bytes(FRAME_DOMAIN, sizeof(FRAME_DOMAIN) - 1U);
    append_bytes(bytes, decode_root(profile_sha256));
    append_bytes(bytes, decode_root(scenario_sha256));
    append_u32(bytes, step);
    append_u32(bytes, static_cast<std::uint32_t>(samples.size()));
    for (const Sample& sample : samples) {
        append_u32(bytes, sample.sample_id);
        for (std::int64_t value : sample.position_um) {
            append_i64(bytes, value);
        }
        for (std::int64_t value : sample.velocity_um_s) {
            append_i64(bytes, value);
        }
    }
    return sha256_hex(bytes);
}

std::string trajectory_root(
    const std::string& profile_sha256,
    const std::string& scenario_sha256,
    const std::vector<std::string>& frame_roots) {
    if (frame_roots.empty()
        || frame_roots.size() > std::numeric_limits<std::uint32_t>::max()) {
        throw Error(ROOT_INVALID, "trajectory frame-root count is outside u32 range");
    }
    std::string bytes(TRAJECTORY_DOMAIN, sizeof(TRAJECTORY_DOMAIN) - 1U);
    append_bytes(bytes, decode_root(profile_sha256));
    append_bytes(bytes, decode_root(scenario_sha256));
    append_u32(bytes, static_cast<std::uint32_t>(frame_roots.size()));
    for (const std::string& root : frame_roots) {
        append_bytes(bytes, decode_root(root));
    }
    return sha256_hex(bytes);
}

} // namespace nextengine::nonlocal::canonical
