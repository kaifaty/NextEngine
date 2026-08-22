#include "slice.hpp"

#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cerrno>
#include <cfenv>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <fcntl.h>
#include <iomanip>
#include <limits>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <sys/stat.h>
#include <unistd.h>
#include <utility>
#include <vector>

namespace nextengine::nonlocal_reference_slice {
namespace {

constexpr std::string_view SCHEMA =
    "nextengine.nonlocal.nsr3b4e2r-first-output-reference-slice.v1";
constexpr std::string_view IDENTITY =
    "681e6e2aab130e0a461dadf575caac754b0931668027db911512a1edac2cccf3";
constexpr std::string_view IDENTITY_PROJECTION =
    "nextengine.nonlocal.nsr3b4e2r-first-output-reference-slice|v1|"
    "parent=9cf5fc571fee7bc0be5585d9b467f90cd8a27f40d9cf39d6be999b0c466cbccc:"
    "60e5575b5e6f5cb332cedda4d59c8ded30960cd5a07a14ccec4cd10ea6c6630e|"
    "alignment=c53a112830cb94c4139da75c2044d28f61673cb1aa05c116098967aee41f7bbe:"
    "79a8932136a64c1cfa953caafe323cb9e860cb134c1d4dc0907be7f40891587a|"
    "payload=ba34b4e3b12986ebc831320d6811551d5311a6774a64f079aabe3a5eaa6bb746;"
    "dam:a0b030805034538420e0a5d90390f7117e645119013e9223ff1b334564cbcc13;"
    "hydro:6d6b70c3feb6fd583d8740a901cc2577de7e3744d4b6c8dbd76a73d2a2710c2f|"
    "slice=dam:step4;hydro:step24;frame1;stable-id6000|"
    "parser=standalone-cxx17;no-generator-reader-reuse;openat-nofollow;"
    "full-hash-before-parse;max67108864|canonical=ties-even-micrometre;"
    "sample-root;sum-position;sum-velocity;q99-rank5940|controls=step,id,"
    "position-um,q99-synthetic,file-final-byte|runs=2-builds;2-processes;"
    "byte-exact|trajectory=none|timing=none|"
    "credit=b4e2d-contract-research-only";
constexpr std::string_view PROFILE =
    "ba34b4e3b12986ebc831320d6811551d5311a6774a64f079aabe3a5eaa6bb746";
constexpr std::string_view R1D_PROFILE_PROJECTION =
    "nextengine.nonlocal.nsr3b4dr1d-full-generation|v1|"
    "parent=3f5a73693f05daf487898fd692160357c1775652a1c11e65205356243c57386f|"
    "schedule=hydro:0..1200/every24;dam:0..720/every4;"
    "orifice:0..720/every4|format=CWREFV2;frame=unchanged|"
    "summary=nearest-rank-q99-x-f64bits;nearest-rank-q99-y-f64bits;"
    "receiver-u32;domain-separated-sha256|"
    "publication=external-content-addressed;regular-file;no-tmp;"
    "verified-copy-only|runs=2-fresh-byte-exact-per-scenario|"
    "parallel=max3;omp1|report-order=hydro,dam,orifice|pressure-max=300|"
    "all-else=r1c5|credit=new-root-only";
constexpr std::string_view MAGIC{"CWREFV2\0", 8U};
constexpr std::size_t MAXIMUM_BYTES = 64U * 1024U * 1024U;
constexpr std::uint32_t SAMPLE_COUNT = 6'000U;
constexpr std::size_t FEATURE_COUNT = 25U;
constexpr std::size_t FRAME_BYTES = 312'156U;
constexpr std::size_t Q99_INDEX = 5'939U;

constexpr std::string_view DAM_MANIFEST = R"(B4DR1D_SCENARIO_V1_BEGIN
scenario_id=CW-DAMBREAK-001
kind=dam-break
box_um=0,0,0;4000000,1000000,1000000
fluid=20,15,20;first=(25000,25000,25000);velocity=(0,0,0);id=iy-iz-ix
fluid_root=9c12e445666c7b0eada3e6e2c258c733323e4eb8ca6474a6f3d5b863f1566e76
boundary=two-layer-outer-complement
boundary_count=16384
boundary_root=1cf0fd172dcb321e995f372119ea956d1e376b8a409b804bc07e31a729aa830d
steps=720
outputs=0..720/every=4
B4DR1D_SCENARIO_V1_END
)";
constexpr std::string_view HYDRO_MANIFEST = R"(B4DR1D_SCENARIO_V1_BEGIN
scenario_id=CW-HYDRO-001
kind=hydrostatic-cube
box_um=0,0,0;1000000,1000000,1000000
fluid=20,15,20;first=(25000,25000,25000);velocity=(0,0,0);id=iy-iz-ix
fluid_root=7d4e661d08de08b18d43a76342329b51f6ae98bca9baee3d850e0f403eae5606
boundary=two-layer-outer-complement
boundary_count=5824
boundary_root=25de85b5eeec041c12bbb5de10e00b8374457b4cf09cb61d99dfc4d5511d8d62
steps=1200
outputs=0..1200/every=24
B4DR1D_SCENARIO_V1_END
)";

struct ReferenceSpec {
    std::string_view scenario;
    std::string_view manifest;
    std::size_t bytes;
    std::string_view file_sha256;
    std::uint32_t total_steps;
    std::uint32_t stride;
    std::uint32_t frames;
};

constexpr std::array<ReferenceSpec, 2> REFERENCES = {{
    {
        "CW-DAMBREAK-001", DAM_MANIFEST, 56'501'239U,
        "a0b030805034538420e0a5d90390f7117e645119013e9223ff1b334564cbcc13",
        720U, 4U, 181U,
    },
    {
        "CW-HYDRO-001", HYDRO_MANIFEST, 15'920'965U,
        "6d6b70c3feb6fd583d8740a901cc2577de7e3744d4b6c8dbd76a73d2a2710c2f",
        1'200U, 24U, 51U,
    },
}};

class Descriptor {
public:
    Descriptor() = default;
    explicit Descriptor(int value) : value_(value) {}
    Descriptor(const Descriptor&) = delete;
    Descriptor& operator=(const Descriptor&) = delete;
    Descriptor(Descriptor&& other) noexcept : value_(other.value_) {
        other.value_ = -1;
    }
    Descriptor& operator=(Descriptor&& other) noexcept {
        if (this != &other) {
            reset();
            value_ = other.value_;
            other.value_ = -1;
        }
        return *this;
    }
    ~Descriptor() { reset(); }
    int get() const { return value_; }
    explicit operator bool() const { return value_ >= 0; }

private:
    void reset() {
        if (value_ >= 0) {
            ::close(value_);
            value_ = -1;
        }
    }
    int value_ = -1;
};

Descriptor open_absolute_directory(std::string_view path) {
    if (path.empty() || path.front() != '/') {
        errno = EINVAL;
        return Descriptor();
    }
    Descriptor current(::open("/", O_RDONLY | O_DIRECTORY | O_CLOEXEC));
    if (!current) {
        return Descriptor();
    }
    std::size_t cursor = 1U;
    while (cursor < path.size()) {
        while (cursor < path.size() && path[cursor] == '/') {
            ++cursor;
        }
        if (cursor == path.size()) {
            break;
        }
        const std::size_t end = path.find('/', cursor);
        const std::size_t component_end = end == std::string_view::npos
            ? path.size() : end;
        const std::string_view component = path.substr(
            cursor, component_end - cursor);
        if (component.empty() || component == "." || component == "..") {
            errno = EINVAL;
            return Descriptor();
        }
        const std::string owned(component);
        Descriptor next(::openat(current.get(), owned.c_str(),
            O_RDONLY | O_DIRECTORY | O_CLOEXEC | O_NOFOLLOW));
        if (!next) {
            return Descriptor();
        }
        current = std::move(next);
        cursor = component_end;
    }
    return current;
}

void append_u32(std::string& output, std::uint32_t value) {
    for (unsigned int shift = 0U; shift < 32U; shift += 8U) {
        output.push_back(static_cast<char>(value >> shift));
    }
}

void append_u64(std::string& output, std::uint64_t value) {
    for (unsigned int shift = 0U; shift < 64U; shift += 8U) {
        output.push_back(static_cast<char>(value >> shift));
    }
}

void append_i64(std::string& output, std::int64_t value) {
    append_u64(output, static_cast<std::uint64_t>(value));
}

std::string bytes_hash(const std::vector<std::uint8_t>& bytes) {
    return nextengine::nonlocal::sha256_hex(std::string_view(
        reinterpret_cast<const char*>(bytes.data()), bytes.size()));
}

std::string domain_hash(std::string_view domain, const std::string& bytes) {
    std::string projection(domain);
    projection.push_back('\0');
    projection.append(bytes);
    return nextengine::nonlocal::sha256_hex(projection);
}

bool same_stat(const struct stat& before, const struct stat& after) {
    return before.st_dev == after.st_dev && before.st_ino == after.st_ino
        && before.st_mode == after.st_mode && before.st_size == after.st_size
        && before.st_mtim.tv_sec == after.st_mtim.tv_sec
        && before.st_mtim.tv_nsec == after.st_mtim.tv_nsec;
}

class Parser {
public:
    explicit Parser(const std::vector<std::uint8_t>& input) : input_(input) {}

    std::uint32_t u32() {
        require(4U);
        std::uint32_t value = 0U;
        for (unsigned int shift = 0U; shift < 32U; shift += 8U) {
            value |= static_cast<std::uint32_t>(input_[cursor_++]) << shift;
        }
        return value;
    }

    std::uint64_t u64() {
        require(8U);
        std::uint64_t value = 0U;
        for (unsigned int shift = 0U; shift < 64U; shift += 8U) {
            value |= static_cast<std::uint64_t>(input_[cursor_++]) << shift;
        }
        return value;
    }

    double finite_f64() {
        const std::uint64_t bits = u64();
        double value = 0.0;
        std::memcpy(&value, &bits, sizeof(value));
        if (!std::isfinite(value)) {
            throw std::runtime_error("NONFINITE_BINARY64");
        }
        return value;
    }

    std::string_view raw(std::size_t count) {
        require(count);
        const char* begin = reinterpret_cast<const char*>(
            input_.data() + cursor_);
        cursor_ += count;
        return std::string_view(begin, count);
    }

    std::size_t cursor() const { return cursor_; }

private:
    void require(std::size_t count) const {
        if (cursor_ > input_.size() || input_.size() - cursor_ < count) {
            throw std::runtime_error("TRUNCATED_PAYLOAD");
        }
    }
    const std::vector<std::uint8_t>& input_;
    std::size_t cursor_ = 0U;
};

struct CanonicalSample {
    std::uint32_t id = 0U;
    std::array<std::int64_t, 3> position{};
    std::array<std::int64_t, 3> velocity{};
};

struct SliceValue {
    std::uint32_t step = 0U;
    std::uint32_t pressure_iterations = 0U;
    std::uint32_t divergence_iterations = 0U;
    std::uint32_t receiver_count = 0U;
    double density_min = 0.0;
    double density_max = 0.0;
    double maximum_speed = 0.0;
    std::vector<CanonicalSample> samples;
    std::string sample_root;
    std::string aggregate_root;
    std::array<std::int64_t, 3> position_sum{};
    std::array<std::int64_t, 3> velocity_sum{};
    std::int64_t q99_x = 0;
    std::int64_t q99_y = 0;
};

struct FileResult {
    const ReferenceSpec* spec = nullptr;
    bool passed = false;
    std::string failure;
    bool path_admitted = false;
    bool regular_file = false;
    bool capacity_admitted = false;
    bool read_exact = false;
    bool file_hash_exact = false;
    bool manifest_exact = false;
    bool layout_exact = false;
    bool frame_zero_exact = false;
    bool selected_frame_exact = false;
    bool serialized_mutation_rejected = false;
    bool step_mutation_rejected = false;
    bool id_mutation_rejected = false;
    bool position_mutation_rejected = false;
    std::string observed_sha256;
    SliceValue slice;
};

#if defined(__SIZEOF_INT128__)
__extension__ typedef unsigned __int128 WideUnsigned;
#else
#error "B4E2R canonical extraction requires checked unsigned 128-bit integer"
#endif

WideUnsigned round_divide_power_of_two(
    WideUnsigned numerator,
    unsigned int shift) {
    if (shift > 128U) {
        return 0U;
    }
    if (shift == 128U) {
        const WideUnsigned half = WideUnsigned(1U) << 127U;
        return numerator > half ? WideUnsigned(1U) : WideUnsigned(0U);
    }
    const WideUnsigned divisor = WideUnsigned(1U) << shift;
    const WideUnsigned quotient = numerator / divisor;
    const WideUnsigned remainder = numerator % divisor;
    const WideUnsigned half = divisor >> 1U;
    return remainder > half
            || (remainder == half && (quotient & 1U) != 0U)
        ? quotient + 1U : quotient;
}

std::int64_t signed_i64(bool negative, WideUnsigned magnitude) {
    const WideUnsigned negative_limit = WideUnsigned(1U) << 63U;
    const WideUnsigned positive_limit = negative_limit - 1U;
    if (negative) {
        if (magnitude > negative_limit) {
            throw std::runtime_error("CANONICAL_RANGE");
        }
        if (magnitude == negative_limit) {
            return std::numeric_limits<std::int64_t>::min();
        }
        return -static_cast<std::int64_t>(
            static_cast<std::uint64_t>(magnitude));
    }
    if (magnitude > positive_limit) {
        throw std::runtime_error("CANONICAL_RANGE");
    }
    return static_cast<std::int64_t>(
        static_cast<std::uint64_t>(magnitude));
}

std::int64_t quantize_micrometres(double value) {
    if (!std::isfinite(value) || std::fegetround() != FE_TONEAREST) {
        throw std::runtime_error("CANONICAL_ROUNDING_ENVIRONMENT");
    }
    std::uint64_t bits = 0U;
    std::memcpy(&bits, &value, sizeof(bits));
    const bool negative = (bits >> 63U) != 0U;
    const int raw_exponent = static_cast<int>((bits >> 52U) & 0x7ffU);
    const std::uint64_t fraction = bits & 0x000fffffffffffffULL;
    if (raw_exponent == 0 && fraction == 0U) {
        return 0;
    }
    const std::uint64_t significand = raw_exponent == 0
        ? fraction : ((std::uint64_t(1U) << 52U) | fraction);
    const int exponent = raw_exponent == 0
        ? -1074 : raw_exponent - 1023 - 52;
    const WideUnsigned scaled = WideUnsigned(significand)
        * WideUnsigned(1'000'000U);
    WideUnsigned magnitude = 0U;
    if (exponent >= 0) {
        const unsigned int shift = static_cast<unsigned int>(exponent);
        const WideUnsigned maximum = ~WideUnsigned(0U);
        if (shift >= 128U || scaled > (maximum >> shift)) {
            throw std::runtime_error("CANONICAL_RANGE");
        }
        magnitude = scaled << shift;
    } else {
        magnitude = round_divide_power_of_two(
            scaled, static_cast<unsigned int>(-exponent));
    }
    return signed_i64(negative, magnitude);
}

bool add_checked(std::int64_t& target, std::int64_t value) {
    if ((value > 0
            && target > std::numeric_limits<std::int64_t>::max() - value)
        || (value < 0
            && target < std::numeric_limits<std::int64_t>::min() - value)) {
        return false;
    }
    target += value;
    return true;
}

std::string sample_root(
    std::string_view scenario,
    std::uint32_t step,
    const std::vector<CanonicalSample>& samples) {
    std::string bytes;
    bytes.reserve(12U + scenario.size() + samples.size() * 52U);
    append_u32(bytes, static_cast<std::uint32_t>(scenario.size()));
    bytes.append(scenario);
    append_u32(bytes, step);
    append_u32(bytes, static_cast<std::uint32_t>(samples.size()));
    for (const CanonicalSample& sample : samples) {
        append_u32(bytes, sample.id);
        for (const std::int64_t value : sample.position) {
            append_i64(bytes, value);
        }
        for (const std::int64_t value : sample.velocity) {
            append_i64(bytes, value);
        }
    }
    return domain_hash(
        "nextengine.nonlocal.nsr3b4e2r-canonical-samples.v1", bytes);
}

void finalize_slice(std::string_view scenario, SliceValue& value) {
    std::vector<std::int64_t> x;
    std::vector<std::int64_t> y;
    x.reserve(value.samples.size());
    y.reserve(value.samples.size());
    for (const CanonicalSample& sample : value.samples) {
        for (std::size_t axis = 0U; axis < 3U; ++axis) {
            if (!add_checked(value.position_sum[axis], sample.position[axis])
                || !add_checked(
                    value.velocity_sum[axis], sample.velocity[axis])) {
                throw std::runtime_error("CANONICAL_SUM_OVERFLOW");
            }
        }
        x.push_back(sample.position[0]);
        y.push_back(sample.position[1]);
    }
    if (x.size() != SAMPLE_COUNT) {
        throw std::runtime_error("CANONICAL_SAMPLE_COUNT");
    }
    std::sort(x.begin(), x.end());
    std::sort(y.begin(), y.end());
    value.q99_x = x[Q99_INDEX];
    value.q99_y = y[Q99_INDEX];
    value.sample_root = sample_root(scenario, value.step, value.samples);

    std::string aggregate;
    append_u32(aggregate, static_cast<std::uint32_t>(scenario.size()));
    aggregate.append(scenario);
    append_u32(aggregate, value.step);
    append_u32(aggregate, SAMPLE_COUNT);
    append_u32(aggregate, static_cast<std::uint32_t>(value.sample_root.size()));
    aggregate.append(value.sample_root);
    for (const std::int64_t item : value.position_sum) {
        append_i64(aggregate, item);
    }
    for (const std::int64_t item : value.velocity_sum) {
        append_i64(aggregate, item);
    }
    append_i64(aggregate, value.q99_x);
    append_i64(aggregate, value.q99_y);
    value.aggregate_root = domain_hash(
        "nextengine.nonlocal.nsr3b4e2r-aggregate.v1", aggregate);
}

SliceValue parse_frame(
    Parser& parser,
    const ReferenceSpec& spec,
    std::uint32_t frame_index,
    bool capture) {
    const std::size_t begin = parser.cursor();
    SliceValue result;
    result.step = parser.u32();
    result.pressure_iterations = parser.u32();
    result.divergence_iterations = parser.u32();
    result.receiver_count = parser.u32();
    const std::uint64_t pressure_error_bits = parser.u64();
    const std::uint64_t divergence_error_bits = parser.u64();
    result.density_min = parser.finite_f64();
    result.density_max = parser.finite_f64();
    result.maximum_speed = parser.finite_f64();
    double pressure_error = 0.0;
    double divergence_error = 0.0;
    std::memcpy(&pressure_error, &pressure_error_bits, sizeof(pressure_error));
    std::memcpy(
        &divergence_error, &divergence_error_bits, sizeof(divergence_error));
    if (!std::isfinite(pressure_error) || !std::isfinite(divergence_error)) {
        throw std::runtime_error("NONFINITE_SOLVER_ERROR");
    }
    const std::uint32_t expected_step = frame_index * spec.stride;
    if (result.step != expected_step || result.receiver_count > SAMPLE_COUNT) {
        throw std::runtime_error("FRAME_HEADER_MISMATCH");
    }
    if (frame_index == 0U) {
        if (result.pressure_iterations != 0U
            || result.divergence_iterations != 0U
            || result.receiver_count != 0U || pressure_error_bits != 0U
            || divergence_error_bits != 0U || result.density_min != 0.0
            || result.density_max != 0.0 || result.maximum_speed != 0.0) {
            throw std::runtime_error("INITIAL_DIAGNOSTICS_MISMATCH");
        }
    } else if (result.pressure_iterations < 2U
        || result.pressure_iterations > 300U
        || result.divergence_iterations < 1U
        || result.divergence_iterations > 100U
        || result.density_min > result.density_max
        || result.maximum_speed < 0.0) {
        throw std::runtime_error("FRAME_DIAGNOSTICS_MISMATCH");
    }
    for (std::size_t feature = 0U; feature < FEATURE_COUNT; ++feature) {
        const std::uint32_t count = parser.u32();
        if ((frame_index == 0U && count != 0U) || count > SAMPLE_COUNT) {
            throw std::runtime_error("FEATURE_COUNT_MISMATCH");
        }
    }
    if (capture) {
        result.samples.reserve(SAMPLE_COUNT);
    }
    for (std::uint32_t id = 0U; id < SAMPLE_COUNT; ++id) {
        if (parser.u32() != id) {
            throw std::runtime_error("STABLE_ID_MISMATCH");
        }
        CanonicalSample sample;
        sample.id = id;
        for (std::size_t axis = 0U; axis < 3U; ++axis) {
            const double decoded = parser.finite_f64();
            if (capture) {
                sample.position[axis] = quantize_micrometres(decoded);
            }
        }
        for (std::size_t axis = 0U; axis < 3U; ++axis) {
            const double decoded = parser.finite_f64();
            if (capture) {
                sample.velocity[axis] = quantize_micrometres(decoded);
            }
        }
        if (capture) {
            result.samples.push_back(sample);
        }
    }
    if (parser.cursor() - begin != FRAME_BYTES) {
        throw std::runtime_error("FRAME_WIDTH_MISMATCH");
    }
    if (capture) {
        finalize_slice(spec.scenario, result);
    }
    return result;
}

bool q99_control_passed() {
    std::vector<std::int64_t> values(SAMPLE_COUNT);
    for (std::size_t index = 0U; index < values.size(); ++index) {
        values[index] = static_cast<std::int64_t>(index);
    }
    std::sort(values.begin(), values.end());
    const bool boundary = values[Q99_INDEX]
        == static_cast<std::int64_t>(Q99_INDEX);
    values[Q99_INDEX] = static_cast<std::int64_t>(SAMPLE_COUNT);
    std::sort(values.begin(), values.end());
    return boundary && values[Q99_INDEX]
        == static_cast<std::int64_t>(Q99_INDEX + 1U);
}

bool ties_to_even_control_passed() {
    return round_divide_power_of_two(WideUnsigned(1U), 1U) == 0U
        && round_divide_power_of_two(WideUnsigned(3U), 1U) == 2U
        && round_divide_power_of_two(WideUnsigned(5U), 1U) == 2U
        && round_divide_power_of_two(WideUnsigned(7U), 1U) == 4U;
}

bool stable_ids_exact(const std::vector<CanonicalSample>& samples) {
    if (samples.size() != SAMPLE_COUNT) {
        return false;
    }
    for (std::size_t index = 0U; index < samples.size(); ++index) {
        if (samples[index].id != index) {
            return false;
        }
    }
    return true;
}

std::string expected_manifest(const ReferenceSpec& spec) {
    std::string result(R1D_PROFILE_PROJECTION);
    result.push_back('\n');
    result.append(spec.manifest);
    return result;
}

FileResult inspect_file(int root_descriptor, const ReferenceSpec& spec) {
    FileResult result;
    result.spec = &spec;
    Descriptor profile(::openat(root_descriptor, std::string(PROFILE).c_str(),
        O_RDONLY | O_DIRECTORY | O_CLOEXEC | O_NOFOLLOW));
    if (!profile) {
        result.failure = "PROFILE_OPEN_REJECTED";
        return result;
    }
    Descriptor scenario(::openat(profile.get(),
        std::string(spec.scenario).c_str(),
        O_RDONLY | O_DIRECTORY | O_CLOEXEC | O_NOFOLLOW));
    if (!scenario) {
        result.failure = "SCENARIO_OPEN_REJECTED";
        return result;
    }
    const std::string filename = std::string(spec.file_sha256) + ".cwrefv2";
    Descriptor file(::openat(scenario.get(), filename.c_str(),
        O_RDONLY | O_CLOEXEC | O_NOFOLLOW));
    if (!file) {
        result.failure = errno == ELOOP
            ? "SYMLINK_REJECTED" : "FILE_OPEN_REJECTED";
        return result;
    }
    result.path_admitted = true;
    struct stat before {};
    if (::fstat(file.get(), &before) != 0) {
        result.failure = "FSTAT_BEFORE_FAILED";
        return result;
    }
    result.regular_file = S_ISREG(before.st_mode);
    if (!result.regular_file) {
        result.failure = "NOT_REGULAR_FILE";
        return result;
    }
    if (before.st_size < 0
        || static_cast<std::uintmax_t>(before.st_size) > MAXIMUM_BYTES) {
        result.failure = "REFERENCE_INPUT_CAPACITY_EXCEEDED";
        return result;
    }
    result.capacity_admitted = true;
    if (static_cast<std::uintmax_t>(before.st_size) != spec.bytes) {
        result.failure = "TOTAL_SIZE_MISMATCH";
        return result;
    }

    std::vector<std::uint8_t> bytes(spec.bytes);
    std::size_t offset = 0U;
    while (offset < bytes.size()) {
        const ssize_t count = ::read(
            file.get(), bytes.data() + offset, bytes.size() - offset);
        if (count < 0) {
            if (errno == EINTR) {
                continue;
            }
            result.failure = "READ_FAILED";
            return result;
        }
        if (count == 0) {
            result.failure = "TRUNCATED_DURING_READ";
            return result;
        }
        offset += static_cast<std::size_t>(count);
    }
    std::uint8_t trailing = 0U;
    const ssize_t trailing_count = ::read(file.get(), &trailing, 1U);
    struct stat after {};
    result.read_exact = trailing_count == 0 && ::fstat(file.get(), &after) == 0
        && same_stat(before, after);
    if (!result.read_exact) {
        result.failure = "FILE_CHANGED_DURING_READ";
        return result;
    }
    result.observed_sha256 = bytes_hash(bytes);
    result.file_hash_exact = result.observed_sha256 == spec.file_sha256;
    if (!result.file_hash_exact) {
        result.failure = "COMPLETE_FILE_HASH_MISMATCH";
        return result;
    }
    bytes.back() ^= 0x01U;
    result.serialized_mutation_rejected =
        bytes_hash(bytes) != spec.file_sha256;
    bytes.back() ^= 0x01U;
    if (!result.serialized_mutation_rejected) {
        result.failure = "SERIALIZED_MUTATION_NOT_REJECTED";
        return result;
    }

    try {
        Parser parser(bytes);
        if (parser.raw(MAGIC.size()) != MAGIC) {
            throw std::runtime_error("MAGIC_MISMATCH");
        }
        const std::uint32_t manifest_bytes = parser.u32();
        const std::string expected = expected_manifest(spec);
        if (manifest_bytes != expected.size()
            || parser.raw(manifest_bytes) != expected) {
            throw std::runtime_error("MANIFEST_MISMATCH");
        }
        result.manifest_exact = true;
        const std::uint32_t samples = parser.u32();
        const std::uint32_t frames = parser.u32();
        if (samples != SAMPLE_COUNT || frames != spec.frames
            || (spec.frames - 1U) * spec.stride != spec.total_steps
            || bytes.size() != 12U + expected.size() + 8U
                + FRAME_BYTES * spec.frames) {
            throw std::runtime_error("FINAL_LAYOUT_MISMATCH");
        }
        result.layout_exact = true;
        const SliceValue initial = parse_frame(parser, spec, 0U, false);
        result.frame_zero_exact = initial.step == 0U;
        if (!result.frame_zero_exact) {
            throw std::runtime_error("INITIAL_FRAME_MISMATCH");
        }
        result.slice = parse_frame(parser, spec, 1U, true);
        result.selected_frame_exact = result.slice.step == spec.stride
            && result.slice.samples.size() == SAMPLE_COUNT
            && !result.slice.sample_root.empty()
            && !result.slice.aggregate_root.empty();
        if (!result.selected_frame_exact) {
            throw std::runtime_error("SELECTED_FRAME_MISMATCH");
        }
        SliceValue step_mutation = result.slice;
        ++step_mutation.step;
        result.step_mutation_rejected = step_mutation.step != spec.stride;
        std::vector<CanonicalSample> id_mutation = result.slice.samples;
        std::swap(id_mutation[0].id, id_mutation[1].id);
        result.id_mutation_rejected = stable_ids_exact(result.slice.samples)
            && !stable_ids_exact(id_mutation);

        std::vector<CanonicalSample> mutated = result.slice.samples;
        if (mutated.front().position[0]
            == std::numeric_limits<std::int64_t>::max()) {
            throw std::runtime_error("POSITION_MUTATION_RANGE");
        }
        ++mutated.front().position[0];
        SliceValue mutated_slice = result.slice;
        mutated_slice.samples = std::move(mutated);
        mutated_slice.sample_root.clear();
        mutated_slice.aggregate_root.clear();
        mutated_slice.position_sum = {};
        mutated_slice.velocity_sum = {};
        finalize_slice(spec.scenario, mutated_slice);
        result.position_mutation_rejected =
            mutated_slice.sample_root != result.slice.sample_root
            && mutated_slice.aggregate_root != result.slice.aggregate_root;
        if (!result.step_mutation_rejected || !result.id_mutation_rejected
            || !result.position_mutation_rejected) {
            throw std::runtime_error("MUTATION_CONTROL_FAILED");
        }
        result.passed = true;
    } catch (const std::exception& error) {
        result.failure = error.what();
    }
    return result;
}

void append_triplet(
    std::ostringstream& output,
    const std::array<std::int64_t, 3>& values) {
    output << values[0] << ',' << values[1] << ',' << values[2];
}

void append_file(std::ostringstream& output, const FileResult& value) {
    output << "file." << value.spec->scenario << '='
           << (value.passed ? "PASS" : "FAIL")
           << ";failure=" << value.failure
           << ";bytes=" << value.spec->bytes
           << ";sha256=" << value.observed_sha256
           << ";path=" << value.path_admitted
           << ";regular=" << value.regular_file
           << ";capacity=" << value.capacity_admitted
           << ";read=" << value.read_exact
           << ";hash=" << value.file_hash_exact
           << ";manifest=" << value.manifest_exact
           << ";layout=" << value.layout_exact
           << ";frame0=" << value.frame_zero_exact
           << ";selected=" << value.selected_frame_exact
           << ";step=" << value.slice.step
           << ";sample_root=" << value.slice.sample_root
           << ";position_sum_um=";
    append_triplet(output, value.slice.position_sum);
    output << ";velocity_sum_um_s=";
    append_triplet(output, value.slice.velocity_sum);
    output << ";q99_x_um=" << value.slice.q99_x
           << ";q99_y_um=" << value.slice.q99_y
           << ";aggregate_root=" << value.slice.aggregate_root
           << ";pressure_iterations=" << value.slice.pressure_iterations
           << ";divergence_iterations=" << value.slice.divergence_iterations
           << ";receiver_count=" << value.slice.receiver_count
           << ";density_min=" << value.slice.density_min
           << ";density_max=" << value.slice.density_max
           << ";maximum_speed=" << value.slice.maximum_speed
           << ";controls=" << value.serialized_mutation_rejected << ','
           << value.step_mutation_rejected << ','
           << value.id_mutation_rejected << ','
           << value.position_mutation_rejected << '\n';
}

SliceRun failure_report(std::string_view failure) {
    std::ostringstream output;
    output << "schema=" << SCHEMA << '\n'
           << "identity=" << IDENTITY << '\n'
           << "status=FAIL\n"
           << "first_failure=" << failure << '\n'
           << "trajectory_started=false\n"
           << "timing_admitted=false\n"
           << "b4e2d_contract_research_authorized=false\n"
           << "runtime_authority=false\n"
           << "production_authority=false\n";
    return {false, output.str()};
}

} // namespace

SliceRun run_first_output(std::string_view artifact_root) {
    if (nextengine::nonlocal::sha256_hex(IDENTITY_PROJECTION) != IDENTITY) {
        return failure_report("IDENTITY_MISMATCH");
    }
    if (artifact_root.empty() || artifact_root.front() != '/') {
        return failure_report("ABSOLUTE_ROOT_REQUIRED");
    }
    if (artifact_root == "/tmp"
        || artifact_root.substr(0U, 5U) == "/tmp/") {
        return failure_report("TMP_ROOT_REJECTED");
    }
    if (std::fesetround(FE_TONEAREST) != 0
        || std::fegetround() != FE_TONEAREST) {
        return failure_report("ROUNDING_ENVIRONMENT");
    }
    Descriptor root = open_absolute_directory(artifact_root);
    if (!root) {
        return failure_report("ROOT_OPEN_REJECTED");
    }
    std::array<FileResult, REFERENCES.size()> files;
    const bool q99_control = q99_control_passed();
    const bool ties_control = ties_to_even_control_passed();
    bool passed = q99_control && ties_control;
    std::string first_failure;
    if (!q99_control) {
        first_failure = "Q99_CONTROL";
    } else if (!ties_control) {
        first_failure = "TIES_TO_EVEN_CONTROL";
    }
    for (std::size_t index = 0U; index < REFERENCES.size(); ++index) {
        files[index] = inspect_file(root.get(), REFERENCES[index]);
        if (!files[index].passed && first_failure.empty()) {
            first_failure = std::string(REFERENCES[index].scenario)
                + ':' + files[index].failure;
        }
        passed = passed && files[index].passed;
    }

    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
             << IDENTITY << '|' << q99_control << ':' << ties_control;
    for (const FileResult& value : files) {
        semantic << '|' << value.spec->scenario << ':' << value.slice.step
                 << ':' << value.slice.sample_root << ':'
                 << value.slice.aggregate_root;
        for (const std::int64_t item : value.slice.position_sum) {
            semantic << ':' << item;
        }
        for (const std::int64_t item : value.slice.velocity_sum) {
            semantic << ':' << item;
        }
        semantic << ':' << value.slice.q99_x << ':' << value.slice.q99_y
                 << ':' << value.serialized_mutation_rejected
                 << ':' << value.step_mutation_rejected
                 << ':' << value.id_mutation_rejected
                 << ':' << value.position_mutation_rejected;
    }
    const std::string result_sha256 =
        nextengine::nonlocal::sha256_hex(semantic.str());
    std::ostringstream report;
    report << std::boolalpha << std::setprecision(17)
           << "schema=" << SCHEMA << '\n'
           << "identity=" << IDENTITY << '\n'
           << "status=" << (passed ? "PASS" : "FAIL") << '\n'
           << "first_failure=" << first_failure << '\n'
           << "q99_rank_5940_control=" << q99_control << '\n'
           << "ties_to_even_control=" << ties_control << '\n';
    for (const FileResult& value : files) {
        append_file(report, value);
    }
    report << "trajectory_started=false\n"
           << "timing_admitted=false\n"
           << "speedup_claim=false\n"
           << "first_output_reference_slice_selected=" << passed << '\n'
           << "b4e2d_contract_research_authorized=" << passed << '\n'
           << "hydro_trajectory_authorized=false\n"
           << "broad_corpus_authorized=false\n"
           << "runtime_authority=false\n"
           << "production_authority=false\n"
           << "result_sha256=" << result_sha256 << '\n';
    return {passed, report.str()};
}

SliceRun reject_unknown_argument() {
    return failure_report("UNKNOWN_ARGUMENT");
}

} // namespace nextengine::nonlocal_reference_slice
