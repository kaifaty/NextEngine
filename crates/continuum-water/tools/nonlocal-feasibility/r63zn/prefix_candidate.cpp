#include <quadmath.h>

#include "../r63zm/fixed_sha256.hpp"

#include <array>
#include <cfenv>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstring>

namespace {

__extension__ using Q = __float128;

constexpr std::size_t CACHE_BYTES = 1033625U;
constexpr std::size_t ARTIFACT_BYTES = 12916U;
constexpr std::size_t AUDIT_BYTES = 420U;
constexpr std::size_t N = 102U;
constexpr std::size_t FACTOR_N = N * N;
constexpr std::size_t ROLE2_VALUE_OFFSET = 5436U;
constexpr const char* CACHE_SHA256 =
    "23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84";
constexpr const char* ARTIFACT_SHA256 =
    "ac6946e872799baef366d8a6648e7bb5cd70c6f2acc326747fdf153c471f0b87";
constexpr const char* AUDIT_SHA256 =
    "fd4bcf0094e9f6ab8c64080c3a22566e3a23d2544e187641075350519a281f80";
constexpr const char* ARTIFACT_RESULT_SHA256 =
    "7eaedbd3775bac423c4e5dfadc25c18f0ffe2ffe181e5080675ff0a457a37da3";
constexpr const char* PRODUCT_SET_SHA256 =
    "6ac66c134a3b8eea010de7e66bab209aa995071089eb8b96bd14804e2722a06b";
constexpr const char* ROLE2_VALUE_SHA256 =
    "8b6373db6132ee119eff020cb53c01c7287d3d49e70a2d6ad7c387b7dd37dcce";
constexpr const char* CHECKER_ROOT_SHA256 =
    "b4a7969c6676b30e87fff470f95e9c098f8951b2a477314213c359c6164c80b4";
enum CandidateWorkSlot : std::size_t {
    RoundingModeSetCalls,
    RoundingModeChecks,
    InputReadAttempts,
    InputOpenCalls,
    InputSeekCalls,
    InputTellCalls,
    InputReadCalls,
    InputTrailingChecks,
    InputCloseCalls,
    InputReadBytes,
    InputHashCalls,
    InputHashBytes,
    CacheTakeCalls,
    CacheTakeBytes,
    CachePredicates,
    ParentArtifactChecks,
    ParentAuditChecks,
    FactorDecodes,
    PermutationDecodes,
    QuadDecodes,
    FactorFiniteChecks,
    PermutationRangeChecks,
    PermutationUniquenessChecks,
    QuadFiniteChecks,
    InversePositiveChecks,
    FactorSolves,
    FactorTerms,
    FactorDivisions,
    SolveFiniteChecks,
    BaselineComparisons,
    ResidualUpdates,
    ResidualDotCalls,
    ResidualDotTerms,
    RhoDotCalls,
    RhoDotTerms,
    TwoProducts,
    TwoSums,
    DotExactChecks,
    DotUnderflowChecks,
    PositivityChecks,
    CanonicalRootCalls,
    CanonicalRootBytes,
    Role2ValueRootComparisons,
    StructuralRootCalls,
    StructuralRootBytes,
    EventRootCalls,
    EventRootBytes,
    TraceRootCalls,
    TraceRootBytes,
    ResultRootCalls,
    ResultRootBytes,
    ReceiptFieldsWritten,
    ReceiptOpenCalls,
    ReceiptWriteCalls,
    ReceiptCloseCalls,
    ReceiptWriteBytes,
    RouteDecisions,
    ControlSelectorChecks,
    ControlSelectorBytes,
    ControlFixtureMutations,
    ControlSmallSolveCalls,
    ControlSmallSolveTerms,
    ControlSmallSolveDivisions,
    ControlWitnessOperations,
    ControlWitnessChecks,
    ReceiptZeroFillBytes,
    PackageAllocations,
    CandidateWorkSlotCount,
};
constexpr std::size_t WORK_COUNTERS = CandidateWorkSlotCount;
constexpr std::size_t EVENTS = 7U;
constexpr std::size_t RECEIPT_BYTES = 1368U;
constexpr std::size_t RECEIPT_VERSION_OFFSET = 8U;
constexpr std::size_t RECEIPT_SIZE_OFFSET = 12U;
constexpr std::size_t RECEIPT_ROUTE_OFFSET = 20U;
constexpr std::size_t RECEIPT_FLAGS_OFFSET = 24U;
constexpr std::size_t RECEIPT_CACHE_SIZE_OFFSET = 32U;
constexpr std::size_t RECEIPT_CACHE_ROOT_OFFSET = 40U;
constexpr std::size_t RECEIPT_ARTIFACT_SIZE_OFFSET = 72U;
constexpr std::size_t RECEIPT_ARTIFACT_ROOT_OFFSET = 80U;
constexpr std::size_t RECEIPT_AUDIT_SIZE_OFFSET = 112U;
constexpr std::size_t RECEIPT_AUDIT_ROOT_OFFSET = 120U;
constexpr std::size_t RECEIPT_BUNDLE_ROOT_OFFSET = 152U;
constexpr std::size_t RECEIPT_FACTOR_ROOT_OFFSET = 184U;
constexpr std::size_t RECEIPT_PERMUTATION_ROOT_OFFSET = 216U;
constexpr std::size_t RECEIPT_INVERSE_ROOT_OFFSET = 248U;
constexpr std::size_t RECEIPT_RHS_ROOT_OFFSET = 280U;
constexpr std::size_t RECEIPT_BASELINE0_ROOT_OFFSET = 312U;
constexpr std::size_t RECEIPT_X0_ROOT_OFFSET = 344U;
constexpr std::size_t RECEIPT_HX0_ROOT_OFFSET = 376U;
constexpr std::size_t RECEIPT_RESIDUAL_ROOT_OFFSET = 408U;
constexpr std::size_t RECEIPT_RESIDUAL_BOUND_ROOT_OFFSET = 440U;
constexpr std::size_t RECEIPT_Z0_ROOT_OFFSET = 472U;
constexpr std::size_t RECEIPT_RHO_ROOT_OFFSET = 504U;
constexpr std::size_t RECEIPT_WORK_OFFSET = 536U;
constexpr std::size_t RECEIPT_EVENT_COUNT_OFFSET = 1072U;
constexpr std::size_t RECEIPT_EVENTS_OFFSET = 1080U;
constexpr std::size_t RECEIPT_TRACE_ROOT_OFFSET = 1304U;
constexpr std::size_t RECEIPT_RESULT_ROOT_OFFSET = 1336U;
static_assert(8U == RECEIPT_VERSION_OFFSET);
static_assert(RECEIPT_VERSION_OFFSET + 4U == RECEIPT_SIZE_OFFSET);
static_assert(RECEIPT_SIZE_OFFSET + 8U == RECEIPT_ROUTE_OFFSET);
static_assert(RECEIPT_ROUTE_OFFSET + 4U == RECEIPT_FLAGS_OFFSET);
static_assert(RECEIPT_FLAGS_OFFSET + 8U == RECEIPT_CACHE_SIZE_OFFSET);
static_assert(RECEIPT_CACHE_SIZE_OFFSET + 8U == RECEIPT_CACHE_ROOT_OFFSET);
static_assert(RECEIPT_CACHE_ROOT_OFFSET + 32U
    == RECEIPT_ARTIFACT_SIZE_OFFSET);
static_assert(RECEIPT_ARTIFACT_SIZE_OFFSET + 8U
    == RECEIPT_ARTIFACT_ROOT_OFFSET);
static_assert(RECEIPT_ARTIFACT_ROOT_OFFSET + 32U
    == RECEIPT_AUDIT_SIZE_OFFSET);
static_assert(RECEIPT_AUDIT_SIZE_OFFSET + 8U
    == RECEIPT_AUDIT_ROOT_OFFSET);
static_assert(RECEIPT_AUDIT_ROOT_OFFSET + 32U
    == RECEIPT_BUNDLE_ROOT_OFFSET);
static_assert(RECEIPT_BUNDLE_ROOT_OFFSET + 32U
    == RECEIPT_FACTOR_ROOT_OFFSET);
static_assert(RECEIPT_FACTOR_ROOT_OFFSET + 32U
    == RECEIPT_PERMUTATION_ROOT_OFFSET);
static_assert(RECEIPT_PERMUTATION_ROOT_OFFSET + 32U
    == RECEIPT_INVERSE_ROOT_OFFSET);
static_assert(RECEIPT_INVERSE_ROOT_OFFSET + 32U
    == RECEIPT_RHS_ROOT_OFFSET);
static_assert(RECEIPT_RHS_ROOT_OFFSET + 32U
    == RECEIPT_BASELINE0_ROOT_OFFSET);
static_assert(RECEIPT_BASELINE0_ROOT_OFFSET + 32U
    == RECEIPT_X0_ROOT_OFFSET);
static_assert(RECEIPT_X0_ROOT_OFFSET + 32U == RECEIPT_HX0_ROOT_OFFSET);
static_assert(RECEIPT_HX0_ROOT_OFFSET + 32U
    == RECEIPT_RESIDUAL_ROOT_OFFSET);
static_assert(RECEIPT_RESIDUAL_ROOT_OFFSET + 32U
    == RECEIPT_RESIDUAL_BOUND_ROOT_OFFSET);
static_assert(RECEIPT_RESIDUAL_BOUND_ROOT_OFFSET + 32U
    == RECEIPT_Z0_ROOT_OFFSET);
static_assert(RECEIPT_Z0_ROOT_OFFSET + 32U == RECEIPT_RHO_ROOT_OFFSET);
static_assert(RECEIPT_RHO_ROOT_OFFSET + 32U == RECEIPT_WORK_OFFSET);
static_assert(RECEIPT_WORK_OFFSET + WORK_COUNTERS * 8U
    == RECEIPT_EVENT_COUNT_OFFSET);
static_assert(RECEIPT_EVENT_COUNT_OFFSET + 8U == RECEIPT_EVENTS_OFFSET);
static_assert(RECEIPT_EVENTS_OFFSET + EVENTS * 32U
    == RECEIPT_TRACE_ROOT_OFFSET);
static_assert(RECEIPT_TRACE_ROOT_OFFSET + 32U == RECEIPT_RESULT_ROOT_OFFSET);
static_assert(RECEIPT_RESULT_ROOT_OFFSET + 32U == RECEIPT_BYTES);

enum class Control : std::uint32_t {
    None = 0U,
    X0Mismatch = 1U,
    PrefixUnderflow = 2U,
    PrefixNonfinite = 3U,
    RhoNonpositive = 4U,
    SmallSolve = 5U,
    StateDifference = 6U,
    Invalid = 7U,
};

struct ControlChoice final {
    Control value = Control::None;
    std::uint64_t selector_checks = 0U;
    std::uint64_t selector_bytes = 0U;
};

ControlChoice parse_control(int argc, const char* text) noexcept {
    ControlChoice choice;
    if (argc == 5) return choice;
    const std::size_t length = std::strlen(text);
    choice.selector_bytes = length + 1U;
    auto matches = [&](const char* expected,
        std::size_t expected_length) noexcept {
        ++choice.selector_checks;
        if (length != expected_length) return false;
        choice.selector_bytes += length;
        return std::memcmp(text, expected, length) == 0;
    };
    if (matches("r63zn-x0-mismatch-v1",
            sizeof("r63zn-x0-mismatch-v1") - 1U)) {
        choice.value = Control::X0Mismatch;
        return choice;
    }
    if (matches("r63zn-prefix-underflow-v1",
            sizeof("r63zn-prefix-underflow-v1") - 1U)) {
        choice.value = Control::PrefixUnderflow;
        return choice;
    }
    if (matches("r63zn-prefix-nonfinite-v1",
            sizeof("r63zn-prefix-nonfinite-v1") - 1U)) {
        choice.value = Control::PrefixNonfinite;
        return choice;
    }
    if (matches("r63zn-rho-nonpositive-v1",
            sizeof("r63zn-rho-nonpositive-v1") - 1U)) {
        choice.value = Control::RhoNonpositive;
        return choice;
    }
    if (matches("r63zn-small-solve-v1",
            sizeof("r63zn-small-solve-v1") - 1U)) {
        choice.value = Control::SmallSolve;
        return choice;
    }
    if (matches("r63zn-state-difference-v1",
            sizeof("r63zn-state-difference-v1") - 1U)) {
        choice.value = Control::StateDifference;
        return choice;
    }
    choice.value = Control::Invalid;
    return choice;
}

struct ByteSpan {
    const std::uint8_t* data = nullptr;
    std::size_t size = 0U;
};

struct VectorSpan {
    ByteSpan bytes;
    std::uint64_t count = 0U;
};

class LittleReader final {
public:
    LittleReader(const std::uint8_t* data, std::size_t size) noexcept
        : begin_(data), cursor_(data), end_(data + size) {}

    ByteSpan bytes(std::size_t count) noexcept {
        if (!valid_ || count > static_cast<std::size_t>(end_ - cursor_)) {
            valid_ = false;
            return {};
        }
        const ByteSpan result{cursor_, count};
        cursor_ += count;
        ++reads_;
        bytes_ += count;
        return result;
    }

    std::uint8_t u8() noexcept {
        const ByteSpan value = bytes(1U);
        return value.data == nullptr ? 0U : value.data[0U];
    }

    std::uint32_t u32() noexcept {
        const ByteSpan value = bytes(4U);
        if (value.data == nullptr) return 0U;
        return static_cast<std::uint32_t>(value.data[0U])
            | (static_cast<std::uint32_t>(value.data[1U]) << 8U)
            | (static_cast<std::uint32_t>(value.data[2U]) << 16U)
            | (static_cast<std::uint32_t>(value.data[3U]) << 24U);
    }

    std::uint64_t u64() noexcept {
        const ByteSpan value = bytes(8U);
        if (value.data == nullptr) return 0U;
        std::uint64_t result = 0U;
        for (std::size_t index = 0U; index < 8U; ++index)
            result |= static_cast<std::uint64_t>(value.data[index])
                << (8U * index);
        return result;
    }

    bool boolean() noexcept {
        const std::uint8_t value = u8();
        require(value <= 1U);
        return value != 0U;
    }

    ByteSpan text() noexcept {
        const std::uint64_t count = u64();
        require(count <= CACHE_BYTES);
        return valid_ ? bytes(static_cast<std::size_t>(count)) : ByteSpan{};
    }

    VectorSpan vector(std::size_t width) noexcept {
        const std::uint64_t count = u64();
        require(width != 0U && count <= CACHE_BYTES / width);
        if (!valid_ || count > static_cast<std::uint64_t>(
                static_cast<std::size_t>(end_ - cursor_) / width))
            return fail_vector();
        return {bytes(static_cast<std::size_t>(count) * width), count};
    }

    void require(bool condition) noexcept {
        ++predicates_;
        valid_ = valid_ && condition;
    }

    bool complete() noexcept {
        ++predicates_;
        return valid_ && cursor_ == end_;
    }
    std::uint64_t reads() const noexcept { return reads_; }
    std::uint64_t byte_count() const noexcept { return bytes_; }
    std::uint64_t predicates() const noexcept { return predicates_; }

private:
    VectorSpan fail_vector() noexcept {
        valid_ = false;
        return {};
    }

    const std::uint8_t* begin_;
    const std::uint8_t* cursor_;
    const std::uint8_t* end_;
    bool valid_ = true;
    std::uint64_t reads_ = 0U;
    std::uint64_t bytes_ = 0U;
    std::uint64_t predicates_ = 0U;
};

void skip_certificate(LittleReader& reader) noexcept {
    static_cast<void>(reader.boolean());
    static_cast<void>(reader.boolean());
    static_cast<void>(reader.bytes(3U * 8U));
    static_cast<void>(reader.bytes(8U));
    for (std::size_t index = 0U; index < 3U; ++index)
        static_cast<void>(reader.text());
}

void skip_certificates(LittleReader& reader) noexcept {
    const std::uint64_t count = reader.u64();
    reader.require(count <= 64U);
    for (std::uint64_t index = 0U; index < count; ++index)
        skip_certificate(reader);
}

void skip_profile(LittleReader& reader) noexcept {
    for (std::size_t index = 0U; index < 3U; ++index)
        static_cast<void>(reader.boolean());
    static_cast<void>(reader.bytes(6U * 8U));
    for (std::size_t index = 0U; index < 4U; ++index)
        static_cast<void>(reader.vector(8U));
    static_cast<void>(reader.bytes(3U * 8U));
    for (std::size_t index = 0U; index < 4U; ++index)
        static_cast<void>(reader.text());
}

struct CacheSelection {
    bool exact = false;
    std::uint64_t dimension = 0U;
    std::uint64_t columns = 0U;
    VectorSpan baseline0;
    VectorSpan factor;
    VectorSpan permutation;
    ByteSpan inverse;
    VectorSpan original_rhs;
    std::uint64_t parse_reads = 0U;
    std::uint64_t parse_bytes = 0U;
    std::uint64_t parse_predicates = 0U;
};

CacheSelection parse_cache(
    const std::array<std::uint8_t, CACHE_BYTES>& cache) noexcept {
    constexpr std::array<std::uint8_t, 8U> magic{
        'N','E','F','P','R','B','1',0U};
    LittleReader reader(cache.data(), cache.size());
    CacheSelection result;
    const ByteSpan observed_magic = reader.bytes(magic.size());
    reader.require(observed_magic.data != nullptr
        && std::memcmp(observed_magic.data, magic.data(), magic.size()) == 0);
    reader.require(reader.u32() == 1U);
    reader.require(reader.u32() == 0x01020304U);
    reader.require(reader.u32() == 8U);
    reader.require(reader.u32() == 16U);
    static_cast<void>(reader.boolean());
    static_cast<void>(reader.text());
    result.dimension = reader.u64();
    result.columns = reader.u64();
    for (std::size_t index = 0U; index < 9U; ++index)
        static_cast<void>(reader.text());
    reader.require(reader.u64() == 3U);
    result.baseline0 = reader.vector(16U);
    static_cast<void>(reader.vector(16U));
    static_cast<void>(reader.vector(16U));
    skip_certificates(reader);
    static_cast<void>(reader.vector(16U));
    static_cast<void>(reader.bytes(16U));
    static_cast<void>(reader.text());
    result.factor = reader.vector(8U);
    const std::uint64_t permutation_count = reader.u64();
    reader.require(permutation_count <= 4096U);
    result.permutation = {
        reader.bytes(static_cast<std::size_t>(permutation_count) * 8U),
        permutation_count};
    result.inverse = reader.bytes(16U);
    for (std::size_t index = 0U; index < 3U; ++index)
        static_cast<void>(reader.text());
    result.original_rhs = reader.vector(16U);
    static_cast<void>(reader.vector(16U));
    static_cast<void>(reader.bytes(16U));
    static_cast<void>(reader.text());
    static_cast<void>(reader.text());
    static_cast<void>(reader.vector(8U));
    for (std::size_t index = 0U; index < 3U; ++index)
        static_cast<void>(reader.text());
    skip_profile(reader);
    static_cast<void>(reader.text());
    static_cast<void>(reader.text());
    reader.require(reader.u64() == 3U);
    static_cast<void>(reader.vector(16U));
    static_cast<void>(reader.vector(16U));
    static_cast<void>(reader.vector(16U));
    skip_certificates(reader);
    for (std::size_t index = 0U; index < 3U; ++index)
        static_cast<void>(reader.text());
    result.exact = reader.complete();
    result.parse_reads = reader.reads();
    result.parse_bytes = reader.byte_count();
    result.parse_predicates = reader.predicates();
    return result;
}

struct Accumulator {
    Q sum = static_cast<Q>(0.0);
    Q correction = static_cast<Q>(0.0);

    void add(Q value) noexcept {
        const Q next = sum + value;
        correction += fabsq(sum) >= fabsq(value)
            ? (sum - next) + value
            : (value - next) + sum;
        sum = next;
    }

    Q value() const noexcept { return sum + correction; }
};

bool normal_or_zero(Q value) noexcept {
    return finiteq(value) != 0
        && (value == static_cast<Q>(0.0)
            || fabsq(value) >= ldexpq(static_cast<Q>(1.0), -16382));
}

Q up_add(Q left, Q right) noexcept {
    if (left == static_cast<Q>(0.0) && right == static_cast<Q>(0.0))
        return static_cast<Q>(0.0);
    return nextafterq(left + right, HUGE_VALQ);
}

Q up_multiply(Q left, Q right) noexcept {
    if (left == static_cast<Q>(0.0) || right == static_cast<Q>(0.0))
        return static_cast<Q>(0.0);
    return nextafterq(left * right, HUGE_VALQ);
}

Q up_divide(Q numerator, Q denominator) noexcept {
    if (numerator == static_cast<Q>(0.0)) return static_cast<Q>(0.0);
    return nextafterq(numerator / denominator, HUGE_VALQ);
}

struct Pair {
    bool exact = false;
    bool no_underflow = false;
    Q high = static_cast<Q>(0.0);
    Q low = static_cast<Q>(0.0);
};

Pair two_sum(Q left, Q right) noexcept {
    Pair result;
    result.high = left + right;
    const Q virtual_right = result.high - left;
    result.low = (left - (result.high - virtual_right))
        + (right - virtual_right);
    result.no_underflow = normal_or_zero(left) && normal_or_zero(right)
        && normal_or_zero(result.high) && normal_or_zero(virtual_right)
        && normal_or_zero(result.low);
    result.exact = finiteq(result.high) != 0 && finiteq(result.low) != 0;
    return result;
}

Pair two_product(Q left, Q right) noexcept {
    Pair result;
    result.high = left * right;
    result.low = fmaq(left, right, -result.high);
    const bool product_not_lost = left == static_cast<Q>(0.0)
        || right == static_cast<Q>(0.0)
        || result.high != static_cast<Q>(0.0)
        || result.low != static_cast<Q>(0.0);
    result.no_underflow = product_not_lost && normal_or_zero(left)
        && normal_or_zero(right) && normal_or_zero(result.high)
        && normal_or_zero(result.low);
    result.exact = finiteq(result.high) != 0 && finiteq(result.low) != 0;
    return result;
}

struct Dot2 {
    bool exact = false;
    bool no_underflow = false;
    Q value = static_cast<Q>(0.0);
    Q bound = static_cast<Q>(0.0);
    Q absolute_products = static_cast<Q>(0.0);
    std::size_t products = 0U;
    std::size_t sums = 0U;
};

Dot2 dot2(const Q* left, const Q* right, std::size_t count) noexcept {
    Dot2 result;
    if (count == 0U) return result;
    const Pair first = two_product(left[0U], right[0U]);
    Q primary = first.high;
    Q correction = first.low;
    bool exact = first.exact;
    bool no_underflow = first.no_underflow;
    result.absolute_products = up_multiply(fabsq(left[0U]), fabsq(right[0U]));
    result.products = 1U;
    for (std::size_t index = 1U; index < count; ++index) {
        const Pair product = two_product(left[index], right[index]);
        const Pair sum = two_sum(primary, product.high);
        const Q local = sum.low + product.low;
        correction = correction + local;
        primary = sum.high;
        exact = exact && product.exact && sum.exact;
        no_underflow = no_underflow && product.no_underflow
            && sum.no_underflow && normal_or_zero(local)
            && normal_or_zero(correction);
        result.absolute_products = up_add(result.absolute_products,
            up_multiply(fabsq(left[index]), fabsq(right[index])));
        ++result.products;
        ++result.sums;
    }
    result.value = primary + correction;
    no_underflow = no_underflow && normal_or_zero(result.value);
    const Q one = static_cast<Q>(1.0);
    const Q unit = ldexpq(one, -112);
    const Q count_unit = static_cast<Q>(count) * unit;
    const Q gamma = up_divide(count_unit, one - count_unit);
    const Q numerator = up_add(up_multiply(unit, fabsq(result.value)),
        up_multiply(up_multiply(gamma, gamma), result.absolute_products));
    result.bound = up_divide(numerator, one - unit);
    result.no_underflow = no_underflow;
    result.exact = exact && finiteq(result.value) != 0
        && finiteq(result.bound) != 0 && result.bound >= static_cast<Q>(0.0);
    return result;
}

struct Solve {
    bool exact = false;
    std::size_t forward_terms = 0U;
    std::size_t backward_terms = 0U;
    std::size_t divisions = 0U;
    std::size_t finite_checks = 0U;
    std::array<Q, N> scaled{};
    std::array<Q, N> intermediate{};
    std::array<Q, N> permuted{};
    std::array<Q, N> solution{};
};

Solve solve(const std::array<Q, FACTOR_N>& upper,
    const std::array<std::uint64_t, N>& permutation,
    const std::array<Q, N>& rhs, Q inverse) noexcept {
    Solve result;
    bool finite = finiteq(inverse) != 0 && inverse > static_cast<Q>(0.0);
    ++result.finite_checks;
    for (std::size_t row = 0U; row < N; ++row) {
        finite = finite && permutation[row] < N;
        ++result.finite_checks;
        if (!finite) break;
        result.scaled[row] = rhs[permutation[row]] * inverse;
        finite = finite && finiteq(result.scaled[row]) != 0;
        ++result.finite_checks;
        Accumulator dot;
        for (std::size_t column = 0U; column < row; ++column) {
            dot.add(upper[column * N + row] * result.intermediate[column]);
            ++result.forward_terms;
        }
        const Q diagonal = upper[row * N + row];
        finite = finite && finiteq(diagonal) != 0
            && diagonal != static_cast<Q>(0.0);
        ++result.finite_checks;
        if (!finite) break;
        result.intermediate[row] = (result.scaled[row] - dot.value()) / diagonal;
        finite = finite && finiteq(result.intermediate[row]) != 0;
        ++result.finite_checks;
        ++result.divisions;
    }
    if (finite) {
        for (std::size_t reverse = N; reverse-- > 0U;) {
            Accumulator dot;
            for (std::size_t column = reverse + 1U; column < N; ++column) {
                dot.add(upper[reverse * N + column] * result.permuted[column]);
                ++result.backward_terms;
            }
            const Q diagonal = upper[reverse * N + reverse];
            finite = finite && finiteq(diagonal) != 0
                && diagonal != static_cast<Q>(0.0);
            ++result.finite_checks;
            if (!finite) break;
            result.permuted[reverse] =
                (result.intermediate[reverse] - dot.value()) / diagonal;
            finite = finite && finiteq(result.permuted[reverse]) != 0;
            ++result.finite_checks;
            ++result.divisions;
        }
    }
    for (std::size_t index = 0U; index < N; ++index)
        if (permutation[index] < N)
            result.solution[permutation[index]] = result.permuted[index];
    result.exact = finite && result.forward_terms == 5151U
        && result.backward_terms == 5151U && result.divisions == 204U;
    return result;
}

struct ControlAccounting final {
    std::uint64_t fixture_mutations = 0U;
    std::uint64_t small_solve_calls = 0U;
    std::uint64_t small_solve_terms = 0U;
    std::uint64_t small_solve_divisions = 0U;
    std::uint64_t witness_operations = 0U;
    std::uint64_t witness_checks = 0U;
};

bool run_small_solve_control(ControlAccounting& accounting) noexcept {
    ++accounting.small_solve_calls;
    constexpr std::size_t count = 3U;
    const std::array<Q, count * count> upper{{
        static_cast<Q>(2.0), static_cast<Q>(1.0), static_cast<Q>(-1.0),
        static_cast<Q>(0.0), static_cast<Q>(3.0), static_cast<Q>(2.0),
        static_cast<Q>(0.0), static_cast<Q>(0.0), static_cast<Q>(4.0)}};
    const std::array<Q, count> rhs{{static_cast<Q>(-6.0),
        static_cast<Q>(-3.0), static_cast<Q>(51.0)}};
    const std::array<Q, count> expected{{static_cast<Q>(1.0),
        static_cast<Q>(-2.0), static_cast<Q>(3.0)}};
    std::array<Q, count> forward{};
    for (std::size_t row = 0U; row < count; ++row) {
        Q sum = static_cast<Q>(0.0);
        for (std::size_t column = 0U; column < row; ++column) {
            sum += upper[column * count + row] * forward[column];
            ++accounting.small_solve_terms;
        }
        forward[row] = (rhs[row] - sum) / upper[row * count + row];
        ++accounting.small_solve_divisions;
    }
    std::array<Q, count> solution{};
    for (std::size_t remaining = count; remaining > 0U;) {
        const std::size_t row = --remaining;
        Q sum = static_cast<Q>(0.0);
        for (std::size_t column = row + 1U; column < count; ++column) {
            sum += upper[row * count + column] * solution[column];
            ++accounting.small_solve_terms;
        }
        solution[row] = (forward[row] - sum) / upper[row * count + row];
        ++accounting.small_solve_divisions;
    }
    bool exact = true;
    for (std::size_t index = 0U; index < count; ++index) {
        exact = exact && std::memcmp(&solution[index], &expected[index],
            sizeof(Q)) == 0;
        ++accounting.witness_checks;
    }
    return exact && accounting.small_solve_terms == 6U
        && accounting.small_solve_divisions == 6U;
}

bool run_state_difference_control(ControlAccounting& accounting) noexcept {
    const Q x0 = static_cast<Q>(1.0);
    const Q direction = static_cast<Q>(1.0)
        * ldexpq(static_cast<Q>(1.0), -113);
    ++accounting.witness_operations;
    const Q x1 = x0 + direction;
    ++accounting.witness_operations;
    const Q state_difference = x1 - x0;
    ++accounting.witness_operations;
    const Q reconstructed = state_difference / static_cast<Q>(1.0);
    ++accounting.witness_operations;
    bool valid = std::memcmp(&x1, &x0, sizeof(Q)) == 0;
    ++accounting.witness_checks;
    valid = valid && direction != static_cast<Q>(0.0);
    ++accounting.witness_checks;
    valid = valid && state_difference == static_cast<Q>(0.0);
    ++accounting.witness_checks;
    valid = valid && reconstructed == static_cast<Q>(0.0);
    ++accounting.witness_checks;
    return valid;
}

template <std::size_t Size>
struct FileObservation final {
    std::array<std::uint8_t, Size> bytes{};
    std::uint64_t observed_size = 0U;
    std::uint64_t open_calls = 0U;
    std::uint64_t seek_calls = 0U;
    std::uint64_t tell_calls = 0U;
    std::uint64_t read_calls = 0U;
    std::uint64_t trailing_checks = 0U;
    std::uint64_t close_calls = 0U;
    std::uint64_t bytes_read = 0U;
    r63zm::Digest root{};
    bool exact = false;
};

template <std::size_t Size>
void read_file(const char* path, FileObservation<Size>& observation) noexcept {
    ++observation.open_calls;
    std::FILE* file = std::fopen(path, "rb");
    if (file == nullptr) return;
    ++observation.seek_calls;
    if (std::fseek(file, 0L, SEEK_END) != 0) {
        ++observation.close_calls;
        static_cast<void>(std::fclose(file));
        return;
    }
    ++observation.tell_calls;
    const long size = std::ftell(file);
    if (size < 0L) {
        ++observation.close_calls;
        static_cast<void>(std::fclose(file));
        return;
    }
    ++observation.seek_calls;
    if (std::fseek(file, 0L, SEEK_SET) != 0) {
        ++observation.close_calls;
        static_cast<void>(std::fclose(file));
        return;
    }
    observation.observed_size = static_cast<std::uint64_t>(size);
    if (observation.observed_size != Size) {
        ++observation.close_calls;
        static_cast<void>(std::fclose(file));
        return;
    }
    ++observation.read_calls;
    const std::size_t count = std::fread(observation.bytes.data(), 1U,
        observation.bytes.size(), file);
    observation.bytes_read = count;
    ++observation.trailing_checks;
    const int trailing = std::fgetc(file);
    ++observation.close_calls;
    const int close = std::fclose(file);
    observation.exact = count == observation.bytes.size() && trailing == EOF
        && close == 0;
    if (observation.exact) observation.root = r63zm::sha256(observation.bytes);
}

std::uint8_t hex_nibble(char value) noexcept {
    if (value >= '0' && value <= '9')
        return static_cast<std::uint8_t>(value - '0');
    if (value >= 'a' && value <= 'f')
        return static_cast<std::uint8_t>(value - 'a' + 10);
    return 0xffU;
}

bool digest_equal_hex(const r63zm::Digest& digest, const char* expected) noexcept {
    for (std::size_t index = 0U; index < digest.size(); ++index) {
        const std::uint8_t high = hex_nibble(expected[2U * index]);
        const std::uint8_t low = hex_nibble(expected[2U * index + 1U]);
        if (high > 0x0fU || low > 0x0fU
            || digest[index] != static_cast<std::uint8_t>((high << 4U) | low))
            return false;
    }
    return expected[64U] == '\0';
}

bool bytes_equal_hex(const std::uint8_t* bytes, const char* expected,
    std::size_t count) noexcept {
    for (std::size_t index = 0U; index < count; ++index) {
        const std::uint8_t high = hex_nibble(expected[2U * index]);
        const std::uint8_t low = hex_nibble(expected[2U * index + 1U]);
        if (high > 0x0fU || low > 0x0fU
            || bytes[index] != static_cast<std::uint8_t>((high << 4U) | low))
            return false;
    }
    return expected[2U * count] == '\0';
}

std::uint32_t be32(const std::uint8_t* bytes) noexcept {
    return (static_cast<std::uint32_t>(bytes[0U]) << 24U)
        | (static_cast<std::uint32_t>(bytes[1U]) << 16U)
        | (static_cast<std::uint32_t>(bytes[2U]) << 8U)
        | static_cast<std::uint32_t>(bytes[3U]);
}

std::uint64_t be64(const std::uint8_t* bytes) noexcept {
    std::uint64_t value = 0U;
    for (std::size_t index = 0U; index < 8U; ++index)
        value = (value << 8U) | bytes[index];
    return value;
}

bool parent_artifact_valid(
    const std::array<std::uint8_t, ARTIFACT_BYTES>& bytes,
    const r63zm::Digest& root, std::uint64_t& checks) noexcept {
    constexpr std::array<std::uint8_t, 8U> magic{
        'N','E','R','6','3','Z','M','1'};
    constexpr std::size_t role2 = 1236U + 2U * 1944U;
    bool valid = true;
    auto check = [&](bool condition) noexcept {
        ++checks;
        valid = valid && condition;
    };
    check(digest_equal_hex(root, ARTIFACT_SHA256));
    check(std::memcmp(bytes.data(), magic.data(), magic.size()) == 0);
    check(be32(bytes.data() + 8U) == 3U);
    check(be64(bytes.data() + 12U) == ARTIFACT_BYTES);
    check(be32(bytes.data() + 60U) == 0U);
    check(bytes_equal_hex(bytes.data() + 128U, PRODUCT_SET_SHA256, 32U));
    check(bytes_equal_hex(bytes.data() + 484U, ARTIFACT_RESULT_SHA256, 32U));
    check(be32(bytes.data() + 1228U) == 6U);
    check(bytes[role2] == 1U && bytes[role2 + 1U] == 1U);
    check(bytes[role2 + 2U] == 2U);
    check(be64(bytes.data() + role2 + 304U) == N);
    check(bytes_equal_hex(bytes.data() + role2 + 144U + 64U,
        ROLE2_VALUE_SHA256, 32U));
    return valid;
}

bool parent_audit_valid(
    const std::array<std::uint8_t, AUDIT_BYTES>& bytes,
    const r63zm::Digest& root, std::uint64_t& checks) noexcept {
    constexpr std::array<std::uint8_t, 8U> magic{
        'N','E','R','6','3','Q','C','1'};
    bool valid = true;
    auto check = [&](bool condition) noexcept {
        ++checks;
        valid = valid && condition;
    };
    check(digest_equal_hex(root, AUDIT_SHA256));
    check(std::memcmp(bytes.data(), magic.data(), magic.size()) == 0);
    check(be32(bytes.data() + 8U) == 3U);
    check(be64(bytes.data() + 12U) == AUDIT_BYTES);
    check(be32(bytes.data() + 20U) == 0U);
    check(bytes[24U] == 1U && bytes[25U] == 1U && bytes[26U] == 1U);
    check(bytes[27U] == 0U);
    check(bytes_equal_hex(bytes.data() + 388U, CHECKER_ROOT_SHA256, 32U));
    return valid;
}

void hash_be64(r63zm::Sha256& hash, std::uint64_t value) noexcept {
    std::array<std::uint8_t, 8U> bytes{};
    for (std::size_t index = 0U; index < bytes.size(); ++index)
        bytes[index] = static_cast<std::uint8_t>(
            value >> (56U - 8U * index));
    hash.update(bytes);
}

void hash_tlv_header(r63zm::Sha256& hash, std::uint8_t tag,
    std::uint64_t length) noexcept {
    hash.update_byte(tag);
    hash_be64(hash, length);
}

void hash_tlv(r63zm::Sha256& hash, std::uint8_t tag,
    const std::uint8_t* data, std::size_t size) noexcept {
    hash_tlv_header(hash, tag, size);
    hash.update(std::span<const std::uint8_t>(data, size));
}

void hash_tlv_text(r63zm::Sha256& hash, std::uint8_t tag,
    const char* text) noexcept {
    hash_tlv(hash, tag, reinterpret_cast<const std::uint8_t*>(text),
        std::strlen(text));
}

void hash_tlv_u64(r63zm::Sha256& hash, std::uint8_t tag,
    std::uint64_t value) noexcept {
    hash_tlv_header(hash, tag, 8U);
    hash_be64(hash, value);
}

void hash_tlv_digest(r63zm::Sha256& hash, std::uint8_t tag,
    const r63zm::Digest& value) noexcept {
    hash_tlv(hash, tag, value.data(), value.size());
}

std::array<std::uint8_t, 16U> canonical_q(Q value) noexcept {
    std::array<std::uint8_t, 16U> native{};
    std::array<std::uint8_t, 16U> canonical{};
    std::memcpy(native.data(), &value, native.size());
    for (std::size_t index = 0U; index < canonical.size(); ++index)
        canonical[index] = native[canonical.size() - 1U - index];
    return canonical;
}

template <std::size_t Count>
r63zm::Digest q_vector_root(const std::array<Q, Count>& values) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U,
        "nextengine.nonlocal.r63zn.binary128-vector.v1");
    hash_tlv_u64(hash, 2U, Count);
    hash_tlv_header(hash, 3U, Count * 16U);
    for (Q value : values) hash.update(canonical_q(value));
    return hash.finish();
}

template <std::size_t Count>
r63zm::Digest parent_q_vector_root(
    const std::array<Q, Count>& values) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U,
        "nextengine.nonlocal.r63zm.binary128-vector.v1");
    hash_tlv_u64(hash, 2U, Count);
    hash_tlv_header(hash, 3U, Count * 16U);
    for (Q value : values) hash.update(canonical_q(value));
    return hash.finish();
}

r63zm::Digest binary64_factor_root(ByteSpan values,
    std::uint64_t count) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U,
        "nextengine.nonlocal.r63zn.binary64-factor.v1");
    hash_tlv_u64(hash, 2U, N);
    hash_tlv_u64(hash, 3U, count);
    hash_tlv_header(hash, 4U, count * 8U);
    for (std::size_t index = 0U; index < count; ++index)
        for (std::size_t byte = 0U; byte < 8U; ++byte)
            hash.update_byte(values.data[index * 8U + 7U - byte]);
    return hash.finish();
}

r63zm::Digest permutation_root(
    const std::array<std::uint64_t, N>& values) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U,
        "nextengine.nonlocal.r63zn.permutation.v1");
    hash_tlv_u64(hash, 2U, values.size());
    hash_tlv_header(hash, 3U, values.size() * 8U);
    for (std::uint64_t value : values) hash_be64(hash, value);
    return hash.finish();
}

r63zm::Digest q_scalar_root(const char* domain, Q value) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U, domain);
    const std::array<std::uint8_t, 16U> bytes = canonical_q(value);
    hash_tlv(hash, 2U, bytes.data(), bytes.size());
    return hash.finish();
}

r63zm::Digest rho_root(const Dot2& value) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U, "nextengine.nonlocal.r63zn.rho0.v1");
    hash_tlv_u64(hash, 2U, value.exact ? 1U : 0U);
    hash_tlv_u64(hash, 3U, value.no_underflow ? 1U : 0U);
    const std::array<std::uint8_t, 16U> scalar = canonical_q(value.value);
    const std::array<std::uint8_t, 16U> bound = canonical_q(value.bound);
    const std::array<std::uint8_t, 16U> absolute =
        canonical_q(value.absolute_products);
    hash_tlv(hash, 4U, scalar.data(), scalar.size());
    hash_tlv(hash, 5U, bound.data(), bound.size());
    hash_tlv(hash, 6U, absolute.data(), absolute.size());
    return hash.finish();
}

template <std::size_t Count>
r63zm::Digest digest_set_root(const char* domain,
    const std::array<r63zm::Digest, Count>& values) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U, domain);
    hash_tlv_u64(hash, 2U, values.size());
    hash_tlv_header(hash, 3U, values.size() * 32U);
    for (const r63zm::Digest& value : values) hash.update(value);
    return hash.finish();
}

r63zm::Digest work_root(
    const std::array<std::uint64_t, WORK_COUNTERS>& values) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U, "nextengine.nonlocal.r63zn.candidate-work.v1");
    hash_tlv_u64(hash, 2U, values.size());
    hash_tlv_header(hash, 3U, values.size() * 8U);
    for (std::uint64_t value : values) hash_be64(hash, value);
    return hash.finish();
}

r63zm::Digest event_root(std::uint64_t ordinal,
    const r63zm::Digest& left, const r63zm::Digest& right) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U, "nextengine.nonlocal.r63zn.event.v1");
    hash_tlv_u64(hash, 2U, ordinal);
    hash_tlv_digest(hash, 3U, left);
    hash_tlv_digest(hash, 4U, right);
    return hash.finish();
}

r63zm::Digest trace_root(std::uint64_t route,
    const std::array<r63zm::Digest, EVENTS>& events,
    std::size_t event_count) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U, "nextengine.nonlocal.r63zn.trace.v1");
    hash_tlv_u64(hash, 2U, route);
    hash_tlv_u64(hash, 3U, event_count);
    hash_tlv_header(hash, 4U, event_count * 32U);
    for (std::size_t index = 0U; index < event_count; ++index)
        hash.update(events[index]);
    return hash.finish();
}

r63zm::Digest result_root(std::uint64_t route,
    const std::array<r63zm::Digest, 3U>& inputs,
    const r63zm::Digest& trace, const r63zm::Digest& work) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U, "nextengine.nonlocal.r63zn.result.v1");
    hash_tlv_u64(hash, 2U, 3U);
    hash_tlv_u64(hash, 3U, route);
    hash_tlv_header(hash, 4U, inputs.size() * 32U);
    for (const r63zm::Digest& input : inputs) hash.update(input);
    hash_tlv_digest(hash, 5U, trace);
    hash_tlv_digest(hash, 6U, work);
    hash_tlv_u64(hash, 7U, RECEIPT_BYTES);
    return hash.finish();
}

void put_be32(std::uint8_t* target, std::uint32_t value) noexcept {
    for (std::size_t index = 0U; index < 4U; ++index)
        target[index] = static_cast<std::uint8_t>(
            value >> (24U - 8U * index));
}

void put_be64(std::uint8_t* target, std::uint64_t value) noexcept {
    for (std::size_t index = 0U; index < 8U; ++index)
        target[index] = static_cast<std::uint8_t>(
            value >> (56U - 8U * index));
}

void put_digest(std::uint8_t* target, const r63zm::Digest& value) noexcept {
    std::memcpy(target, value.data(), value.size());
}

template <std::size_t Size>
bool write_file(const char* path,
    const std::array<std::uint8_t, Size>& bytes) noexcept {
    std::FILE* file = std::fopen(path, "wb");
    if (file == nullptr) return false;
    const std::size_t count = std::fwrite(bytes.data(), 1U, bytes.size(), file);
    const int close = std::fclose(file);
    return count == bytes.size() && close == 0;
}

Q little_q(ByteSpan bytes, std::size_t index) noexcept {
    Q value{};
    if (bytes.data != nullptr && (index + 1U) * sizeof(value) <= bytes.size)
        std::memcpy(&value, bytes.data + index * sizeof(value), sizeof(value));
    return value;
}

double little_double(ByteSpan bytes, std::size_t index) noexcept {
    double value = 0.0;
    if (bytes.data != nullptr && (index + 1U) * sizeof(value) <= bytes.size)
        std::memcpy(&value, bytes.data + index * sizeof(value), sizeof(value));
    return value;
}

std::uint64_t little_u64(ByteSpan bytes, std::size_t index) noexcept {
    std::uint64_t value = 0U;
    if (bytes.data != nullptr && (index + 1U) * sizeof(value) <= bytes.size)
        std::memcpy(&value, bytes.data + index * sizeof(value), sizeof(value));
    return value;
}

Q artifact_q(const std::array<std::uint8_t, ARTIFACT_BYTES>& bytes,
    std::size_t offset) noexcept {
    std::array<std::uint8_t, 16U> reversed{};
    for (std::size_t index = 0U; index < reversed.size(); ++index)
        reversed[index] = bytes[offset + reversed.size() - 1U - index];
    Q value{};
    std::memcpy(&value, reversed.data(), sizeof(value));
    return value;
}

bool write_early_rejection(std::uint32_t route,
    const FileObservation<CACHE_BYTES>& cache,
    const FileObservation<ARTIFACT_BYTES>& artifact,
    const FileObservation<AUDIT_BYTES>& audit,
    std::uint64_t read_calls, std::uint64_t artifact_checks,
    std::uint64_t audit_checks, std::uint64_t parse_reads,
    std::uint64_t parse_bytes, std::uint64_t parse_predicates,
    std::uint64_t route_decisions,
    const char* output_path) noexcept {
    const std::uint64_t input_bytes = cache.bytes_read + artifact.bytes_read
        + audit.bytes_read;
    const std::uint64_t input_open_calls = cache.open_calls
        + artifact.open_calls + audit.open_calls;
    const std::uint64_t input_seek_calls = cache.seek_calls
        + artifact.seek_calls + audit.seek_calls;
    const std::uint64_t input_tell_calls = cache.tell_calls
        + artifact.tell_calls + audit.tell_calls;
    const std::uint64_t input_read_calls = cache.read_calls
        + artifact.read_calls + audit.read_calls;
    const std::uint64_t input_trailing_checks = cache.trailing_checks
        + artifact.trailing_checks + audit.trailing_checks;
    const std::uint64_t input_close_calls = cache.close_calls
        + artifact.close_calls + audit.close_calls;
    const std::uint64_t hash_calls = (cache.exact ? 1U : 0U)
        + (artifact.exact ? 1U : 0U) + (audit.exact ? 1U : 0U);
    const std::uint64_t hash_bytes = (cache.exact ? CACHE_BYTES : 0U)
        + (artifact.exact ? ARTIFACT_BYTES : 0U)
        + (audit.exact ? AUDIT_BYTES : 0U);
    const std::uint64_t work_root_bytes =
        std::strlen("nextengine.nonlocal.r63zn.candidate-work.v1") + 571U;
    const std::uint64_t trace_bytes =
        std::strlen("nextengine.nonlocal.r63zn.trace.v1") + 52U;
    const std::uint64_t result_bytes =
        std::strlen("nextengine.nonlocal.r63zn.result.v1") + 247U;
    std::array<std::uint64_t, WORK_COUNTERS> work{};
    work[RoundingModeSetCalls] = 1U;
    work[RoundingModeChecks] = 1U;
    work[InputReadAttempts] = read_calls;
    work[InputOpenCalls] = input_open_calls;
    work[InputSeekCalls] = input_seek_calls;
    work[InputTellCalls] = input_tell_calls;
    work[InputReadCalls] = input_read_calls;
    work[InputTrailingChecks] = input_trailing_checks;
    work[InputCloseCalls] = input_close_calls;
    work[InputReadBytes] = input_bytes;
    work[InputHashCalls] = hash_calls;
    work[InputHashBytes] = hash_bytes;
    work[CacheTakeCalls] = parse_reads;
    work[CacheTakeBytes] = parse_bytes;
    work[CachePredicates] = parse_predicates;
    work[ParentArtifactChecks] = artifact_checks;
    work[ParentAuditChecks] = audit_checks;
    work[StructuralRootCalls] = 1U;
    work[StructuralRootBytes] = work_root_bytes;
    work[TraceRootCalls] = 1U;
    work[TraceRootBytes] = trace_bytes;
    work[ResultRootCalls] = 1U;
    work[ResultRootBytes] = result_bytes;
    work[ReceiptFieldsWritten] = 102U;
    work[ReceiptOpenCalls] = 1U;
    work[ReceiptWriteCalls] = 1U;
    work[ReceiptCloseCalls] = 1U;
    work[ReceiptWriteBytes] = RECEIPT_BYTES;
    work[RouteDecisions] = route_decisions;
    work[ReceiptZeroFillBytes] = RECEIPT_BYTES;
    work[PackageAllocations] = 0U;
    const r63zm::Digest candidate_work_root = work_root(work);
    const r63zm::Digest zero{};
    const std::array<r63zm::Digest, 3U> inputs{{
        cache.exact ? cache.root : zero,
        artifact.exact ? artifact.root : zero,
        audit.exact ? audit.root : zero}};
    const std::array<r63zm::Digest, EVENTS> events{};
    const r63zm::Digest trace = trace_root(route, events, 0U);
    const r63zm::Digest result = result_root(
        route, inputs, trace, candidate_work_root);

    std::array<std::uint8_t, RECEIPT_BYTES> receipt{};
    constexpr std::array<std::uint8_t, 8U> receipt_magic{{
        'N','E','R','6','3','Z','N','1'}};
    std::memcpy(receipt.data(), receipt_magic.data(), receipt_magic.size());
    put_be32(receipt.data() + RECEIPT_VERSION_OFFSET, 3U);
    put_be64(receipt.data() + RECEIPT_SIZE_OFFSET, RECEIPT_BYTES);
    put_be32(receipt.data() + RECEIPT_ROUTE_OFFSET, route);
    receipt[RECEIPT_FLAGS_OFFSET] = 0U;
    receipt[RECEIPT_FLAGS_OFFSET + 1U] = 0U;
    receipt[RECEIPT_FLAGS_OFFSET + 2U] = 0U;
    put_be64(receipt.data() + RECEIPT_CACHE_SIZE_OFFSET,
        cache.observed_size);
    put_digest(receipt.data() + RECEIPT_CACHE_ROOT_OFFSET, inputs[0U]);
    put_be64(receipt.data() + RECEIPT_ARTIFACT_SIZE_OFFSET,
        artifact.observed_size);
    put_digest(receipt.data() + RECEIPT_ARTIFACT_ROOT_OFFSET, inputs[1U]);
    put_be64(receipt.data() + RECEIPT_AUDIT_SIZE_OFFSET,
        audit.observed_size);
    put_digest(receipt.data() + RECEIPT_AUDIT_ROOT_OFFSET, inputs[2U]);
    constexpr std::array<std::size_t, 12U> semantic_offsets{{
        RECEIPT_BUNDLE_ROOT_OFFSET, RECEIPT_FACTOR_ROOT_OFFSET,
        RECEIPT_PERMUTATION_ROOT_OFFSET, RECEIPT_INVERSE_ROOT_OFFSET,
        RECEIPT_RHS_ROOT_OFFSET, RECEIPT_BASELINE0_ROOT_OFFSET,
        RECEIPT_X0_ROOT_OFFSET, RECEIPT_HX0_ROOT_OFFSET,
        RECEIPT_RESIDUAL_ROOT_OFFSET,
        RECEIPT_RESIDUAL_BOUND_ROOT_OFFSET, RECEIPT_Z0_ROOT_OFFSET,
        RECEIPT_RHO_ROOT_OFFSET}};
    for (std::size_t offset : semantic_offsets)
        put_digest(receipt.data() + offset, zero);
    for (std::size_t index = 0U; index < work.size(); ++index)
        put_be64(receipt.data() + RECEIPT_WORK_OFFSET + index * 8U,
            work[index]);
    put_be64(receipt.data() + RECEIPT_EVENT_COUNT_OFFSET, 0U);
    for (std::size_t index = 0U; index < events.size(); ++index)
        put_digest(receipt.data() + RECEIPT_EVENTS_OFFSET + index * 32U,
            zero);
    put_digest(receipt.data() + RECEIPT_TRACE_ROOT_OFFSET, trace);
    put_digest(receipt.data() + RECEIPT_RESULT_ROOT_OFFSET, result);
    return write_file(output_path, receipt);
}

using SemanticRoots = std::array<r63zm::Digest, 12U>;

std::array<std::uint64_t, WORK_COUNTERS> make_prefix_work(
    const FileObservation<CACHE_BYTES>& cache,
    const FileObservation<ARTIFACT_BYTES>& artifact,
    const FileObservation<AUDIT_BYTES>& audit,
    const CacheSelection& selection, std::uint64_t artifact_checks,
    std::uint64_t audit_checks, const ControlChoice& control,
    const ControlAccounting& control_work, std::uint64_t factor_solves,
    std::uint64_t factor_terms, std::uint64_t factor_divisions,
    std::uint64_t solve_finite_checks, std::uint64_t baseline_comparisons,
    std::uint64_t residual_updates, std::uint64_t residual_dot_calls,
    std::uint64_t residual_dot_terms, std::uint64_t rho_dot_calls,
    std::uint64_t rho_dot_terms, std::uint64_t two_products,
    std::uint64_t two_sums, std::uint64_t positivity_checks,
    std::uint64_t canonical_root_calls,
    std::uint64_t canonical_root_bytes, std::size_t event_count,
    std::uint64_t route_decisions, bool parent_product_consumed) noexcept {
    constexpr std::uint64_t input_bytes =
        CACHE_BYTES + ARTIFACT_BYTES + AUDIT_BYTES;
    const std::uint64_t structural_root_bytes =
        std::strlen("nextengine.nonlocal.r63zn.input-bundle.v1") + 291U
        + std::strlen("nextengine.nonlocal.r63zn.parent-set.v1") + 131U
        + std::strlen("nextengine.nonlocal.r63zn.candidate-work.v1") + 571U;
    const std::uint64_t event_root_bytes = event_count
        * (std::strlen("nextengine.nonlocal.r63zn.event.v1") + 108U);
    const std::uint64_t trace_root_bytes =
        std::strlen("nextengine.nonlocal.r63zn.trace.v1") + 52U
        + event_count * 32U;
    const std::uint64_t result_root_bytes =
        std::strlen("nextengine.nonlocal.r63zn.result.v1") + 247U;
    std::array<std::uint64_t, WORK_COUNTERS> work{};
    work[RoundingModeSetCalls] = 1U;
    work[RoundingModeChecks] = 1U;
    work[InputReadAttempts] = 3U;
    work[InputOpenCalls] = cache.open_calls + artifact.open_calls
        + audit.open_calls;
    work[InputSeekCalls] = cache.seek_calls + artifact.seek_calls
        + audit.seek_calls;
    work[InputTellCalls] = cache.tell_calls + artifact.tell_calls
        + audit.tell_calls;
    work[InputReadCalls] = cache.read_calls + artifact.read_calls
        + audit.read_calls;
    work[InputTrailingChecks] = cache.trailing_checks
        + artifact.trailing_checks + audit.trailing_checks;
    work[InputCloseCalls] = cache.close_calls + artifact.close_calls
        + audit.close_calls;
    work[InputReadBytes] = input_bytes;
    work[InputHashCalls] = 3U;
    work[InputHashBytes] = input_bytes;
    work[CacheTakeCalls] = selection.parse_reads;
    work[CacheTakeBytes] = selection.parse_bytes;
    work[CachePredicates] = selection.parse_predicates;
    work[ParentArtifactChecks] = artifact_checks;
    work[ParentAuditChecks] = audit_checks;
    work[FactorDecodes] = FACTOR_N;
    work[PermutationDecodes] = N;
    work[QuadDecodes] = (parent_product_consumed ? 3U : 2U) * N + 1U;
    work[FactorFiniteChecks] = FACTOR_N;
    work[PermutationRangeChecks] = N;
    work[PermutationUniquenessChecks] = N;
    work[QuadFiniteChecks] =
        (parent_product_consumed ? 3U : 2U) * N + 1U;
    work[InversePositiveChecks] = 1U;
    work[FactorSolves] = factor_solves;
    work[FactorTerms] = factor_terms;
    work[FactorDivisions] = factor_divisions;
    work[SolveFiniteChecks] = solve_finite_checks;
    work[BaselineComparisons] = baseline_comparisons;
    work[ResidualUpdates] = residual_updates;
    work[ResidualDotCalls] = residual_dot_calls;
    work[ResidualDotTerms] = residual_dot_terms;
    work[RhoDotCalls] = rho_dot_calls;
    work[RhoDotTerms] = rho_dot_terms;
    work[TwoProducts] = two_products;
    work[TwoSums] = two_sums;
    work[DotExactChecks] = residual_dot_calls + rho_dot_calls;
    work[DotUnderflowChecks] = residual_dot_calls + rho_dot_calls;
    work[PositivityChecks] = positivity_checks;
    work[CanonicalRootCalls] = canonical_root_calls;
    work[CanonicalRootBytes] = canonical_root_bytes;
    work[Role2ValueRootComparisons] = parent_product_consumed ? 1U : 0U;
    work[StructuralRootCalls] = 3U;
    work[StructuralRootBytes] = structural_root_bytes;
    work[EventRootCalls] = event_count;
    work[EventRootBytes] = event_root_bytes;
    work[TraceRootCalls] = 1U;
    work[TraceRootBytes] = trace_root_bytes;
    work[ResultRootCalls] = 1U;
    work[ResultRootBytes] = result_root_bytes;
    work[ReceiptFieldsWritten] = 102U;
    work[ReceiptOpenCalls] = 1U;
    work[ReceiptWriteCalls] = 1U;
    work[ReceiptCloseCalls] = 1U;
    work[ReceiptWriteBytes] = RECEIPT_BYTES;
    work[RouteDecisions] = route_decisions;
    work[ControlSelectorChecks] = control.selector_checks;
    work[ControlSelectorBytes] = control.selector_bytes;
    work[ControlFixtureMutations] = control_work.fixture_mutations;
    work[ControlSmallSolveCalls] = control_work.small_solve_calls;
    work[ControlSmallSolveTerms] = control_work.small_solve_terms;
    work[ControlSmallSolveDivisions] = control_work.small_solve_divisions;
    work[ControlWitnessOperations] = control_work.witness_operations;
    work[ControlWitnessChecks] = control_work.witness_checks;
    work[ReceiptZeroFillBytes] = RECEIPT_BYTES;
    work[PackageAllocations] = 0U;
    return work;
}

bool write_prefix_receipt(std::uint32_t route,
    const std::array<std::uint8_t, 3U>& flags,
    const FileObservation<CACHE_BYTES>& cache,
    const FileObservation<ARTIFACT_BYTES>& artifact,
    const FileObservation<AUDIT_BYTES>& audit,
    const SemanticRoots& semantic,
    const std::array<std::uint64_t, WORK_COUNTERS>& work,
    const r63zm::Digest& candidate_work_root,
    const std::array<r63zm::Digest, EVENTS>& events,
    std::size_t event_count, const char* output_path) noexcept {
    const std::array<r63zm::Digest, 3U> inputs{{
        cache.root, artifact.root, audit.root}};
    const r63zm::Digest trace = trace_root(route, events, event_count);
    const r63zm::Digest result = result_root(
        route, inputs, trace, candidate_work_root);
    std::array<std::uint8_t, RECEIPT_BYTES> receipt{};
    constexpr std::array<std::uint8_t, 8U> magic{{
        'N','E','R','6','3','Z','N','1'}};
    std::memcpy(receipt.data(), magic.data(), magic.size());
    put_be32(receipt.data() + RECEIPT_VERSION_OFFSET, 3U);
    put_be64(receipt.data() + RECEIPT_SIZE_OFFSET, RECEIPT_BYTES);
    put_be32(receipt.data() + RECEIPT_ROUTE_OFFSET, route);
    for (std::size_t index = 0U; index < flags.size(); ++index)
        receipt[RECEIPT_FLAGS_OFFSET + index] = flags[index];
    put_be64(receipt.data() + RECEIPT_CACHE_SIZE_OFFSET,
        cache.observed_size);
    put_digest(receipt.data() + RECEIPT_CACHE_ROOT_OFFSET, cache.root);
    put_be64(receipt.data() + RECEIPT_ARTIFACT_SIZE_OFFSET,
        artifact.observed_size);
    put_digest(receipt.data() + RECEIPT_ARTIFACT_ROOT_OFFSET, artifact.root);
    put_be64(receipt.data() + RECEIPT_AUDIT_SIZE_OFFSET,
        audit.observed_size);
    put_digest(receipt.data() + RECEIPT_AUDIT_ROOT_OFFSET, audit.root);
    constexpr std::array<std::size_t, 12U> offsets{{
        RECEIPT_BUNDLE_ROOT_OFFSET, RECEIPT_FACTOR_ROOT_OFFSET,
        RECEIPT_PERMUTATION_ROOT_OFFSET, RECEIPT_INVERSE_ROOT_OFFSET,
        RECEIPT_RHS_ROOT_OFFSET, RECEIPT_BASELINE0_ROOT_OFFSET,
        RECEIPT_X0_ROOT_OFFSET, RECEIPT_HX0_ROOT_OFFSET,
        RECEIPT_RESIDUAL_ROOT_OFFSET, RECEIPT_RESIDUAL_BOUND_ROOT_OFFSET,
        RECEIPT_Z0_ROOT_OFFSET, RECEIPT_RHO_ROOT_OFFSET}};
    for (std::size_t index = 0U; index < offsets.size(); ++index)
        put_digest(receipt.data() + offsets[index], semantic[index]);
    for (std::size_t index = 0U; index < work.size(); ++index)
        put_be64(receipt.data() + RECEIPT_WORK_OFFSET + index * 8U,
            work[index]);
    put_be64(receipt.data() + RECEIPT_EVENT_COUNT_OFFSET, event_count);
    for (std::size_t index = 0U; index < events.size(); ++index)
        put_digest(receipt.data() + RECEIPT_EVENTS_OFFSET + index * 32U,
            events[index]);
    put_digest(receipt.data() + RECEIPT_TRACE_ROOT_OFFSET, trace);
    put_digest(receipt.data() + RECEIPT_RESULT_ROOT_OFFSET, result);
    return write_file(output_path, receipt);
}

} // namespace

int main(int argc, char** argv) {
    if (argc != 5 && argc != 6) return 64;
    if (std::fesetround(FE_TONEAREST) != 0
        || std::fegetround() != FE_TONEAREST)
        return 67;
    FileObservation<CACHE_BYTES> cache_file;
    FileObservation<ARTIFACT_BYTES> artifact_file;
    FileObservation<AUDIT_BYTES> audit_file;
    std::uint64_t read_calls = 1U;
    read_file(argv[1], cache_file);
    if (!cache_file.exact)
        return write_early_rejection(0U, cache_file, artifact_file,
            audit_file, read_calls, 0U, 0U, 0U, 0U, 0U, 1U, argv[4])
            ? 0 : 69;
    ++read_calls;
    read_file(argv[2], artifact_file);
    if (!artifact_file.exact)
        return write_early_rejection(1U, cache_file, artifact_file,
            audit_file, read_calls, 0U, 0U, 0U, 0U, 0U, 2U, argv[4])
            ? 0 : 69;
    std::uint64_t artifact_checks = 0U;
    const bool artifact_valid = parent_artifact_valid(artifact_file.bytes,
        artifact_file.root, artifact_checks);
    if (!artifact_valid)
        return write_early_rejection(1U, cache_file, artifact_file,
            audit_file, read_calls, artifact_checks, 0U, 0U, 0U, 0U, 3U,
            argv[4]) ? 0 : 69;
    ++read_calls;
    read_file(argv[3], audit_file);
    if (!audit_file.exact)
        return write_early_rejection(2U, cache_file, artifact_file,
            audit_file, read_calls, artifact_checks, 0U, 0U, 0U, 0U, 4U,
            argv[4]) ? 0 : 69;
    std::uint64_t audit_checks = 0U;
    const bool audit_valid = parent_audit_valid(audit_file.bytes,
        audit_file.root, audit_checks);
    if (!audit_valid)
        return write_early_rejection(2U, cache_file, artifact_file,
            audit_file, read_calls, artifact_checks, audit_checks,
            0U, 0U, 0U, 5U, argv[4]) ? 0 : 69;
    if (!digest_equal_hex(cache_file.root, CACHE_SHA256))
        return write_early_rejection(3U, cache_file, artifact_file,
            audit_file, read_calls, artifact_checks, audit_checks,
            0U, 0U, 0U, 6U, argv[4]) ? 0 : 69;
    const auto& cache = cache_file.bytes;
    const auto& artifact = artifact_file.bytes;
    const r63zm::Digest& cache_root = cache_file.root;
    const r63zm::Digest& artifact_root = artifact_file.root;
    const r63zm::Digest& audit_root = audit_file.root;
    const CacheSelection selection = parse_cache(cache);
    if (!selection.exact || selection.dimension != N
        || selection.columns != 315U || selection.baseline0.count != N
        || selection.factor.count != FACTOR_N
        || selection.permutation.count != N
        || selection.inverse.size != sizeof(Q)
        || selection.original_rhs.count != N)
        return write_early_rejection(3U, cache_file, artifact_file,
            audit_file, read_calls, artifact_checks, audit_checks,
            selection.parse_reads, selection.parse_bytes,
            selection.parse_predicates, 7U, argv[4]) ? 0 : 69;

    std::array<Q, FACTOR_N> upper{};
    bool factor_inputs_finite = true;
    for (std::size_t index = 0U; index < upper.size(); ++index)
    {
        upper[index] = static_cast<Q>(little_double(selection.factor.bytes,
            index));
        factor_inputs_finite = factor_inputs_finite
            && finiteq(upper[index]) != 0;
    }
    std::array<std::uint64_t, N> permutation{};
    std::array<bool, N> seen{};
    bool permutation_exact = true;
    for (std::size_t index = 0U; index < N; ++index) {
        permutation[index] = little_u64(selection.permutation.bytes, index);
        permutation_exact = permutation_exact && permutation[index] < N
            && !seen[permutation[index]];
        if (permutation[index] < N) seen[permutation[index]] = true;
    }
    std::array<Q, N> rhs{};
    std::array<Q, N> baseline0{};
    bool q_inputs_finite = true;
    for (std::size_t index = 0U; index < N; ++index) {
        rhs[index] = little_q(selection.original_rhs.bytes, index);
        baseline0[index] = little_q(selection.baseline0.bytes, index);
        q_inputs_finite = q_inputs_finite && finiteq(rhs[index]) != 0
            && finiteq(baseline0[index]) != 0;
    }
    const Q inverse = little_q(selection.inverse, 0U);
    q_inputs_finite = q_inputs_finite && finiteq(inverse) != 0;
    const bool inverse_positive = inverse > static_cast<Q>(0.0);
    const ControlChoice control = parse_control(argc,
        argc == 6 ? argv[5] : nullptr);
    if (control.value == Control::Invalid) return 64;
    ControlAccounting control_work;
    if (control.value == Control::SmallSolve
        && !run_small_solve_control(control_work))
        return 68;
    if (control.value == Control::StateDifference
        && !run_state_difference_control(control_work))
        return 68;

    Solve start = solve(upper, permutation, rhs, inverse);
    if (control.value == Control::X0Mismatch) {
        start.solution[0U] = nextafterq(start.solution[0U], HUGE_VALQ);
        ++control_work.fixture_mutations;
    }
    std::size_t x0_matches = 0U;
    for (std::size_t index = 0U; index < N; ++index)
        x0_matches += std::memcmp(&start.solution[index], &baseline0[index],
            sizeof(Q)) == 0 ? 1U : 0U;
    const bool common_valid = factor_inputs_finite && permutation_exact
        && q_inputs_finite && inverse_positive && start.exact
        && start.finite_checks == 613U;
    if (!common_valid) return 68;

    const r63zm::Digest factor_digest = binary64_factor_root(
        selection.factor.bytes, selection.factor.count);
    const r63zm::Digest permutation_digest = permutation_root(permutation);
    const r63zm::Digest inverse_digest = q_scalar_root(
        "nextengine.nonlocal.r63zn.inverse-scale.v1", inverse);
    const r63zm::Digest rhs_digest = q_vector_root(rhs);
    const r63zm::Digest baseline_digest = q_vector_root(baseline0);
    const r63zm::Digest x0_digest = q_vector_root(start.solution);
    const std::array<r63zm::Digest, 8U> bundle_fields{{cache_root,
        artifact_root, audit_root, factor_digest, permutation_digest,
        inverse_digest, rhs_digest, baseline_digest}};
    const r63zm::Digest bundle_digest = digest_set_root(
        "nextengine.nonlocal.r63zn.input-bundle.v1", bundle_fields);
    const std::array<r63zm::Digest, 3U> input_roots{{cache_root,
        artifact_root, audit_root}};
    const r63zm::Digest parent_set_digest = digest_set_root(
        "nextengine.nonlocal.r63zn.parent-set.v1", input_roots);
    SemanticRoots semantic{{bundle_digest, factor_digest,
        permutation_digest, inverse_digest, rhs_digest, baseline_digest,
        x0_digest, {}, {}, {}, {}, {}}};
    const std::uint64_t preproduct_canonical_bytes =
        3U * (std::strlen(
            "nextengine.nonlocal.r63zn.binary128-vector.v1") + 1667U)
        + std::strlen("nextengine.nonlocal.r63zn.binary64-factor.v1")
            + 83284U
        + std::strlen("nextengine.nonlocal.r63zn.permutation.v1") + 851U
        + std::strlen("nextengine.nonlocal.r63zn.inverse-scale.v1") + 34U;

    if (x0_matches != N) {
        constexpr std::size_t event_count = 3U;
        const auto work = make_prefix_work(cache_file, artifact_file,
            audit_file, selection, artifact_checks,
            audit_checks, control, control_work, 1U,
            start.forward_terms + start.backward_terms, start.divisions,
            start.finite_checks, N, 0U, 0U, 0U, 0U, 0U, 0U, 0U, 0U,
            6U, preproduct_canonical_bytes, event_count, 8U, false);
        const r63zm::Digest work_digest = work_root(work);
        std::array<r63zm::Digest, EVENTS> events{};
        const r63zm::Digest zero{};
        events[0U] = event_root(0U, parent_set_digest, zero);
        events[1U] = event_root(1U, bundle_digest, factor_digest);
        events[2U] = event_root(2U, x0_digest, baseline_digest);
        constexpr std::array<std::uint8_t, 3U> flags{{1U, 1U, 0U}};
        return write_prefix_receipt(4U, flags, cache_file, artifact_file,
            audit_file, semantic, work, work_digest, events, event_count,
            argv[4]) ? 0 : 69;
    }
    if (control.value == Control::X0Mismatch) return 68;

    std::array<Q, N> hx0{};
    bool hx0_finite = true;
    for (std::size_t index = 0U; index < N; ++index) {
        hx0[index] = artifact_q(artifact, ROLE2_VALUE_OFFSET + 16U * index);
        hx0_finite = hx0_finite && finiteq(hx0[index]) != 0;
    }
    if (!hx0_finite) return 68;
    const r63zm::Digest hx0_digest = q_vector_root(hx0);
    const r63zm::Digest parent_hx0_digest = parent_q_vector_root(hx0);
    if (!digest_equal_hex(parent_hx0_digest, ROLE2_VALUE_SHA256)) return 68;
    semantic[7U] = hx0_digest;
    const std::uint64_t product_canonical_bytes = preproduct_canonical_bytes
        + std::strlen("nextengine.nonlocal.r63zn.binary128-vector.v1") + 1667U
        + std::strlen("nextengine.nonlocal.r63zm.binary128-vector.v1") + 1667U;

    std::array<Q, N> residual{};
    std::array<Q, N> residual_bound{};
    bool residual_exact = true;
    bool residual_no_underflow = true;
    std::size_t residual_updates = 0U;
    std::size_t residual_products = 0U;
    std::size_t residual_sums = 0U;
    const Q one = static_cast<Q>(1.0);
    for (std::size_t index = 0U; index < N; ++index) {
        std::array<Q, 2U> left{{rhs[index], -hx0[index]}};
        const std::array<Q, 2U> right{{one, one}};
        if (control.value == Control::PrefixUnderflow && index == 0U) {
            left[0U] = ldexpq(one, -16494);
            left[1U] = static_cast<Q>(0.0);
            control_work.fixture_mutations += 2U;
        }
        if (control.value == Control::PrefixNonfinite && index == 0U) {
            left[0U] = HUGE_VALQ;
            ++control_work.fixture_mutations;
        }
        const Dot2 dot = dot2(left.data(), right.data(), left.size());
        residual[index] = dot.value;
        residual_bound[index] = dot.bound;
        residual_exact = residual_exact && dot.exact;
        residual_no_underflow = residual_no_underflow && dot.no_underflow;
        ++residual_updates;
        residual_products += dot.products;
        residual_sums += dot.sums;
        if (!dot.exact || !dot.no_underflow) break;
    }
    if (!residual_exact || !residual_no_underflow) {
        constexpr std::size_t event_count = 4U;
        const auto work = make_prefix_work(cache_file, artifact_file,
            audit_file, selection, artifact_checks,
            audit_checks, control, control_work, 1U,
            start.forward_terms + start.backward_terms, start.divisions,
            start.finite_checks, N, residual_updates, residual_updates,
            2U * residual_updates, 0U, 0U, residual_products,
            residual_sums, 0U, 8U, product_canonical_bytes, event_count, 9U,
            true);
        const r63zm::Digest work_digest = work_root(work);
        std::array<r63zm::Digest, EVENTS> events{};
        const r63zm::Digest zero{};
        events[0U] = event_root(0U, parent_set_digest, zero);
        events[1U] = event_root(1U, bundle_digest, factor_digest);
        events[2U] = event_root(2U, x0_digest, baseline_digest);
        events[3U] = event_root(3U, hx0_digest, artifact_root);
        constexpr std::array<std::uint8_t, 3U> flags{{1U, 1U, 0U}};
        return write_prefix_receipt(5U, flags, cache_file, artifact_file,
            audit_file, semantic, work, work_digest, events, event_count,
            argv[4]) ? 0 : 69;
    }
    if (control.value == Control::PrefixUnderflow
        || control.value == Control::PrefixNonfinite)
        return 68;
    if (residual_updates != N || residual_products != 2U * N
        || residual_sums != N)
        return 68;

    const Solve preconditioned = solve(upper, permutation, residual, inverse);
    if (!preconditioned.exact || preconditioned.finite_checks != 613U)
        return 68;
    const r63zm::Digest residual_digest = q_vector_root(residual);
    const r63zm::Digest residual_bound_digest = q_vector_root(residual_bound);
    const r63zm::Digest z0_digest = q_vector_root(preconditioned.solution);
    const Q* rho_right_values = preconditioned.solution.data();
    if (control.value == Control::RhoNonpositive) {
        for (std::size_t index = 0U; index < N; ++index) {
            start.solution[index] = -preconditioned.solution[index];
            ++control_work.fixture_mutations;
        }
        rho_right_values = start.solution.data();
    }
    const Dot2 rho = dot2(residual.data(), rho_right_values, N);
    const Q rho_lower = rho.value - rho.bound;
    const bool positive = rho.exact && rho.no_underflow
        && rho_lower > static_cast<Q>(0.0);
    if (!rho.exact || !rho.no_underflow || rho.products != N
        || rho.sums != N - 1U)
        return 68;
    const r63zm::Digest rho_digest = rho_root(rho);
    semantic[8U] = residual_digest;
    semantic[9U] = residual_bound_digest;
    semantic[10U] = z0_digest;
    semantic[11U] = rho_digest;
    const std::uint64_t full_canonical_bytes = product_canonical_bytes
        + 3U * (std::strlen(
            "nextengine.nonlocal.r63zn.binary128-vector.v1") + 1667U)
        + std::strlen("nextengine.nonlocal.r63zn.rho0.v1") + 118U;
    const std::uint64_t total_factor_terms = start.forward_terms
        + start.backward_terms + preconditioned.forward_terms
        + preconditioned.backward_terms;
    const std::uint64_t total_factor_divisions = start.divisions
        + preconditioned.divisions;
    const std::uint64_t total_solve_finite = start.finite_checks
        + preconditioned.finite_checks;
    const std::uint64_t total_products = residual_products + rho.products;
    const std::uint64_t total_sums = residual_sums + rho.sums;

    if (!positive) {
        constexpr std::size_t event_count = 6U;
        const auto work = make_prefix_work(cache_file, artifact_file,
            audit_file, selection, artifact_checks,
            audit_checks, control, control_work, 2U,
            total_factor_terms, total_factor_divisions, total_solve_finite,
            N, residual_updates, residual_updates, 2U * residual_updates,
            1U, N, total_products, total_sums, 1U, 12U,
            full_canonical_bytes, event_count, 10U, true);
        const r63zm::Digest work_digest = work_root(work);
        std::array<r63zm::Digest, EVENTS> events{};
        const r63zm::Digest zero{};
        events[0U] = event_root(0U, parent_set_digest, zero);
        events[1U] = event_root(1U, bundle_digest, factor_digest);
        events[2U] = event_root(2U, x0_digest, baseline_digest);
        events[3U] = event_root(3U, hx0_digest, artifact_root);
        events[4U] = event_root(4U, residual_digest,
            residual_bound_digest);
        events[5U] = event_root(5U, z0_digest, factor_digest);
        constexpr std::array<std::uint8_t, 3U> flags{{1U, 1U, 0U}};
        return write_prefix_receipt(6U, flags, cache_file, artifact_file,
            audit_file, semantic, work, work_digest, events, event_count,
            argv[4]) ? 0 : 69;
    }
    if (control.value == Control::RhoNonpositive) return 68;

    constexpr std::size_t event_count = EVENTS;
    const auto work = make_prefix_work(cache_file, artifact_file, audit_file,
        selection, artifact_checks,
        audit_checks, control, control_work, 2U, total_factor_terms,
        total_factor_divisions, total_solve_finite, N, residual_updates,
        residual_updates, 2U * residual_updates, 1U, N, total_products,
        total_sums, 1U, 12U, full_canonical_bytes, event_count, 10U, true);
    const r63zm::Digest work_digest = work_root(work);
    const r63zm::Digest zero{};
    const std::array<r63zm::Digest, EVENTS> events{{
        event_root(0U, parent_set_digest, zero),
        event_root(1U, bundle_digest, factor_digest),
        event_root(2U, x0_digest, baseline_digest),
        event_root(3U, hx0_digest, artifact_root),
        event_root(4U, residual_digest, residual_bound_digest),
        event_root(5U, z0_digest, factor_digest),
        event_root(6U, rho_digest, work_digest)}};
    constexpr std::array<std::uint8_t, 3U> flags{{1U, 1U, 1U}};
    return write_prefix_receipt(7U, flags, cache_file, artifact_file,
        audit_file, semantic, work, work_digest, events, event_count,
        argv[4]) ? 0 : 69;
}
