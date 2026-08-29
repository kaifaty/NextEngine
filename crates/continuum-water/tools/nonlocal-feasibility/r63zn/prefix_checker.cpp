#include <quadmath.h>

#include "../r63zm/fixed_sha256.hpp"

#include <array>
#include <cfenv>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <fcntl.h>
#include <sys/stat.h>
#include <unistd.h>

namespace {

__extension__ using Quad = __float128;
using Digest = r63zm::Digest;

constexpr std::size_t CACHE_SIZE = 1033625U;
constexpr std::size_t PARENT_SIZE = 12916U;
constexpr std::size_t PARENT_AUDIT_SIZE = 420U;
constexpr std::size_t RECEIPT_SIZE = 1368U;
constexpr std::size_t DIMENSION = 102U;
constexpr std::size_t FACTOR_COMPONENTS = DIMENSION * DIMENSION;
constexpr std::size_t ROLE_TWO_VALUES = 5436U;
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
constexpr std::size_t CANDIDATE_WORK_FIELDS = CandidateWorkSlotCount;
constexpr std::size_t EVENT_SLOTS = 7U;

constexpr std::size_t VERSION_AT = 8U;
constexpr std::size_t TOTAL_AT = 12U;
constexpr std::size_t ROUTE_AT = 20U;
constexpr std::size_t FLAGS_AT = 24U;
constexpr std::size_t CACHE_SIZE_AT = 32U;
constexpr std::size_t CACHE_ROOT_AT = 40U;
constexpr std::size_t PARENT_SIZE_AT = 72U;
constexpr std::size_t PARENT_ROOT_AT = 80U;
constexpr std::size_t PARENT_AUDIT_SIZE_AT = 112U;
constexpr std::size_t PARENT_AUDIT_ROOT_AT = 120U;
constexpr std::size_t BUNDLE_ROOT_AT = 152U;
constexpr std::size_t FACTOR_ROOT_AT = 184U;
constexpr std::size_t PERMUTATION_ROOT_AT = 216U;
constexpr std::size_t INVERSE_ROOT_AT = 248U;
constexpr std::size_t RHS_ROOT_AT = 280U;
constexpr std::size_t BASELINE_ROOT_AT = 312U;
constexpr std::size_t X0_ROOT_AT = 344U;
constexpr std::size_t HX0_ROOT_AT = 376U;
constexpr std::size_t RESIDUAL_ROOT_AT = 408U;
constexpr std::size_t RESIDUAL_BOUND_ROOT_AT = 440U;
constexpr std::size_t Z0_ROOT_AT = 472U;
constexpr std::size_t RHO_ROOT_AT = 504U;
constexpr std::size_t CANDIDATE_WORK_AT = 536U;
constexpr std::size_t EVENT_COUNT_AT = 1072U;
constexpr std::size_t EVENTS_AT = 1080U;
constexpr std::size_t TRACE_AT = 1304U;
constexpr std::size_t RESULT_AT = 1336U;

static_assert(8U == VERSION_AT);
static_assert(VERSION_AT + 4U == TOTAL_AT);
static_assert(TOTAL_AT + 8U == ROUTE_AT);
static_assert(ROUTE_AT + 4U == FLAGS_AT);
static_assert(FLAGS_AT + 8U == CACHE_SIZE_AT);
static_assert(CACHE_SIZE_AT + 8U == CACHE_ROOT_AT);
static_assert(CACHE_ROOT_AT + 32U == PARENT_SIZE_AT);
static_assert(PARENT_SIZE_AT + 8U == PARENT_ROOT_AT);
static_assert(PARENT_ROOT_AT + 32U == PARENT_AUDIT_SIZE_AT);
static_assert(PARENT_AUDIT_SIZE_AT + 8U == PARENT_AUDIT_ROOT_AT);
static_assert(PARENT_AUDIT_ROOT_AT + 32U == BUNDLE_ROOT_AT);
static_assert(BUNDLE_ROOT_AT + 32U == FACTOR_ROOT_AT);
static_assert(FACTOR_ROOT_AT + 32U == PERMUTATION_ROOT_AT);
static_assert(PERMUTATION_ROOT_AT + 32U == INVERSE_ROOT_AT);
static_assert(INVERSE_ROOT_AT + 32U == RHS_ROOT_AT);
static_assert(RHS_ROOT_AT + 32U == BASELINE_ROOT_AT);
static_assert(BASELINE_ROOT_AT + 32U == X0_ROOT_AT);
static_assert(X0_ROOT_AT + 32U == HX0_ROOT_AT);
static_assert(HX0_ROOT_AT + 32U == RESIDUAL_ROOT_AT);
static_assert(RESIDUAL_ROOT_AT + 32U == RESIDUAL_BOUND_ROOT_AT);
static_assert(RESIDUAL_BOUND_ROOT_AT + 32U == Z0_ROOT_AT);
static_assert(Z0_ROOT_AT + 32U == RHO_ROOT_AT);
static_assert(RHO_ROOT_AT + 32U == CANDIDATE_WORK_AT);
static_assert(CANDIDATE_WORK_AT + CANDIDATE_WORK_FIELDS * 8U
    == EVENT_COUNT_AT);
static_assert(EVENT_COUNT_AT + 8U == EVENTS_AT);
static_assert(EVENTS_AT + EVENT_SLOTS * 32U == TRACE_AT);
static_assert(TRACE_AT + 32U == RESULT_AT);
static_assert(RESULT_AT + 32U == RECEIPT_SIZE);

constexpr const char* CACHE_ID =
    "23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84";
constexpr const char* PARENT_ID =
    "ac6946e872799baef366d8a6648e7bb5cd70c6f2acc326747fdf153c471f0b87";
constexpr const char* PARENT_AUDIT_ID =
    "fd4bcf0094e9f6ab8c64080c3a22566e3a23d2544e187641075350519a281f80";
constexpr const char* PARENT_RESULT_ID =
    "7eaedbd3775bac423c4e5dfadc25c18f0ffe2ffe181e5080675ff0a457a37da3";
constexpr const char* PRODUCT_SET_ID =
    "6ac66c134a3b8eea010de7e66bab209aa995071089eb8b96bd14804e2722a06b";
constexpr const char* ROLE_TWO_VALUE_ID =
    "8b6373db6132ee119eff020cb53c01c7287d3d49e70a2d6ad7c387b7dd37dcce";
constexpr const char* PARENT_CHECKER_ID =
    "b4a7969c6676b30e87fff470f95e9c098f8951b2a477314213c359c6164c80b4";

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

ControlChoice decode_control(int argc, const char* text) noexcept {
    ControlChoice choice;
    if (argc == 6) return choice;
    const std::size_t text_size = std::strlen(text);
    choice.selector_bytes = text_size + 1U;
    auto same = [&](const char* expected, std::size_t expected_size) noexcept {
        ++choice.selector_checks;
        if (text_size != expected_size) return false;
        choice.selector_bytes += text_size;
        return std::memcmp(text, expected, text_size) == 0;
    };
    if (same("r63zn-x0-mismatch-v1",
            sizeof("r63zn-x0-mismatch-v1") - 1U)) {
        choice.value = Control::X0Mismatch;
        return choice;
    }
    if (same("r63zn-prefix-underflow-v1",
            sizeof("r63zn-prefix-underflow-v1") - 1U)) {
        choice.value = Control::PrefixUnderflow;
        return choice;
    }
    if (same("r63zn-prefix-nonfinite-v1",
            sizeof("r63zn-prefix-nonfinite-v1") - 1U)) {
        choice.value = Control::PrefixNonfinite;
        return choice;
    }
    if (same("r63zn-rho-nonpositive-v1",
            sizeof("r63zn-rho-nonpositive-v1") - 1U)) {
        choice.value = Control::RhoNonpositive;
        return choice;
    }
    if (same("r63zn-small-solve-v1",
            sizeof("r63zn-small-solve-v1") - 1U)) {
        choice.value = Control::SmallSolve;
        return choice;
    }
    if (same("r63zn-state-difference-v1",
            sizeof("r63zn-state-difference-v1") - 1U)) {
        choice.value = Control::StateDifference;
        return choice;
    }
    choice.value = Control::Invalid;
    return choice;
}

constexpr std::size_t CHECKER_WORK_FIELDS = 51U;
constexpr std::size_t CHECKER_AUDIT_SIZE = 700U;
constexpr std::size_t CHECKER_VERSION_AT = 8U;
constexpr std::size_t CHECKER_TOTAL_AT = 12U;
constexpr std::size_t CHECKER_ROUTE_AT = 20U;
constexpr std::size_t CHECKER_FLAGS_AT = 24U;
constexpr std::size_t CHECKER_SIZES_AT = 28U;
constexpr std::size_t CHECKER_ROOTS_AT = 60U;
constexpr std::size_t CHECKER_WORK_COUNT_AT = 188U;
constexpr std::size_t CHECKER_WORK_AT = 196U;
constexpr std::size_t CHECKER_OBSERVED_RESULT_AT = 604U;
constexpr std::size_t CHECKER_REPLAY_RESULT_AT = 636U;
constexpr std::size_t CHECKER_RESULT_AT = 668U;
static_assert(8U == CHECKER_VERSION_AT);
static_assert(CHECKER_VERSION_AT + 4U == CHECKER_TOTAL_AT);
static_assert(CHECKER_TOTAL_AT + 8U == CHECKER_ROUTE_AT);
static_assert(CHECKER_ROUTE_AT + 4U == CHECKER_FLAGS_AT);
static_assert(CHECKER_FLAGS_AT + 4U == CHECKER_SIZES_AT);
static_assert(CHECKER_SIZES_AT + 4U * 8U == CHECKER_ROOTS_AT);
static_assert(CHECKER_ROOTS_AT + 4U * 32U == CHECKER_WORK_COUNT_AT);
static_assert(CHECKER_WORK_COUNT_AT + 8U == CHECKER_WORK_AT);
static_assert(CHECKER_WORK_AT + CHECKER_WORK_FIELDS * 8U
    == CHECKER_OBSERVED_RESULT_AT);
static_assert(CHECKER_OBSERVED_RESULT_AT + 32U == CHECKER_REPLAY_RESULT_AT);
static_assert(CHECKER_REPLAY_RESULT_AT + 32U == CHECKER_RESULT_AT);
static_assert(CHECKER_RESULT_AT + 32U == CHECKER_AUDIT_SIZE);

struct CheckerWork final {
    std::uint64_t rounding_mode_set_calls = 0U;
    std::uint64_t rounding_mode_checks = 0U;
    std::uint64_t logical_file_reads = 0U;
    std::uint64_t file_open_calls = 0U;
    std::uint64_t file_stat_calls = 0U;
    std::uint64_t file_close_calls = 0U;
    std::uint64_t file_read_calls = 0U;
    std::uint64_t file_read_stops = 0U;
    std::uint64_t file_bytes = 0U;
    std::uint64_t hash_calls = 0U;
    std::uint64_t hash_bytes = 0U;
    std::uint64_t parent_checks = 0U;
    std::uint64_t cursor_takes = 0U;
    std::uint64_t cursor_bytes = 0U;
    std::uint64_t cursor_predicates = 0U;
    std::uint64_t scalar_loads = 0U;
    std::uint64_t factor_solves = 0U;
    std::uint64_t factor_terms = 0U;
    std::uint64_t factor_divisions = 0U;
    std::uint64_t baseline_comparisons = 0U;
    std::uint64_t dot_calls = 0U;
    std::uint64_t dot_terms = 0U;
    std::uint64_t two_products = 0U;
    std::uint64_t two_sums = 0U;
    std::uint64_t arithmetic_checks = 0U;
    std::uint64_t role2_value_root_comparisons = 0U;
    std::uint64_t receipt_header_checks = 0U;
    std::uint64_t receipt_padding_bytes_scanned = 0U;
    std::uint64_t semantic_bytes_compared = 0U;
    std::uint64_t work_fields_compared = 0U;
    std::uint64_t event_fields_compared = 0U;
    std::uint64_t seal_fields_compared = 0U;
    std::uint64_t cache_identity_checks = 0U;
    std::uint64_t candidate_route_decisions = 0U;
    std::uint64_t checker_route_decisions = 0U;
    std::uint64_t control_selector_checks = 0U;
    std::uint64_t control_selector_bytes = 0U;
    std::uint64_t control_fixture_mutations = 0U;
    std::uint64_t control_small_solve_calls = 0U;
    std::uint64_t control_small_solve_terms = 0U;
    std::uint64_t control_small_solve_divisions = 0U;
    std::uint64_t control_witness_operations = 0U;
    std::uint64_t control_witness_checks = 0U;
    std::uint64_t audit_root_bytes = 0U;
    std::uint64_t audit_open_calls = 0U;
    std::uint64_t audit_write_calls = 0U;
    std::uint64_t audit_close_calls = 0U;
    std::uint64_t audit_write_bytes = 0U;
    std::uint64_t audit_fields_written = 0U;
    std::uint64_t package_allocations = 0U;
    std::uint64_t audit_zero_fill_bytes = 0U;

    std::array<std::uint64_t, CHECKER_WORK_FIELDS> fields() const noexcept {
        return {{rounding_mode_set_calls, rounding_mode_checks,
            logical_file_reads, file_open_calls, file_stat_calls,
            file_close_calls, file_read_calls, file_read_stops, file_bytes,
            hash_calls, hash_bytes,
            parent_checks, cursor_takes, cursor_bytes, cursor_predicates,
            scalar_loads, factor_solves, factor_terms, factor_divisions,
            baseline_comparisons, dot_calls, dot_terms, two_products,
            two_sums, arithmetic_checks, role2_value_root_comparisons,
            receipt_header_checks, receipt_padding_bytes_scanned,
            semantic_bytes_compared, work_fields_compared,
            event_fields_compared, seal_fields_compared,
            cache_identity_checks, candidate_route_decisions,
            checker_route_decisions,
            control_selector_checks, control_selector_bytes,
            control_fixture_mutations,
            control_small_solve_calls, control_small_solve_terms,
            control_small_solve_divisions, control_witness_operations,
            control_witness_checks,
            audit_root_bytes, audit_open_calls, audit_write_calls,
            audit_close_calls, audit_write_bytes, audit_fields_written,
            package_allocations, audit_zero_fill_bytes}};
    }
};

template <std::size_t Size>
struct FileImage final {
    std::array<std::uint8_t, Size> bytes{};
    std::uint64_t observed_size = 0U;
    std::uint64_t bytes_read = 0U;
    Digest root{};
    bool opened = false;
    bool exact = false;
};

template <std::size_t Size>
void load_exact(const char* path, FileImage<Size>& image,
    CheckerWork& work) noexcept {
    ++work.logical_file_reads;
    const int descriptor = open(path, O_RDONLY);
    ++work.file_open_calls;
    if (descriptor < 0) return;
    image.opened = true;
    struct stat metadata {};
    ++work.file_stat_calls;
    if (fstat(descriptor, &metadata) != 0 || metadata.st_size < 0) {
        ++work.file_close_calls;
        static_cast<void>(close(descriptor));
        return;
    }
    image.observed_size = static_cast<std::uint64_t>(metadata.st_size);
    if (image.observed_size == Size) {
        std::size_t offset = 0U;
        while (offset < Size) {
            ++work.file_read_calls;
            const ssize_t count = read(descriptor, image.bytes.data() + offset,
                Size - offset);
            if (count <= 0) {
                ++work.file_read_stops;
                break;
            }
            offset += static_cast<std::size_t>(count);
        }
        work.file_bytes += offset;
        image.bytes_read = offset;
        image.exact = offset == Size;
    }
    ++work.file_close_calls;
    image.exact = image.exact && close(descriptor) == 0;
    if (image.exact) {
        image.root = r63zm::sha256(image.bytes);
        ++work.hash_calls;
        work.hash_bytes += Size;
    }
}

std::uint8_t nibble(char value) noexcept {
    if (value >= '0' && value <= '9')
        return static_cast<std::uint8_t>(value - '0');
    if (value >= 'a' && value <= 'f')
        return static_cast<std::uint8_t>(value - 'a' + 10);
    return 0xffU;
}

bool digest_is(const Digest& actual, const char* expected) noexcept {
    for (std::size_t index = 0U; index < actual.size(); ++index) {
        const std::uint8_t high = nibble(expected[index * 2U]);
        const std::uint8_t low = nibble(expected[index * 2U + 1U]);
        if (high > 15U || low > 15U
            || actual[index] != static_cast<std::uint8_t>(high * 16U + low))
            return false;
    }
    return expected[64U] == '\0';
}

bool bytes_are(const std::uint8_t* actual, const char* expected,
    std::size_t count) noexcept {
    for (std::size_t index = 0U; index < count; ++index) {
        const std::uint8_t high = nibble(expected[index * 2U]);
        const std::uint8_t low = nibble(expected[index * 2U + 1U]);
        if (high > 15U || low > 15U
            || actual[index] != static_cast<std::uint8_t>(high * 16U + low))
            return false;
    }
    return expected[count * 2U] == '\0';
}

std::uint32_t big_u32(const std::uint8_t* bytes) noexcept {
    std::uint32_t result = 0U;
    for (std::size_t index = 0U; index < 4U; ++index)
        result = (result << 8U) | bytes[index];
    return result;
}

std::uint64_t big_u64(const std::uint8_t* bytes) noexcept {
    std::uint64_t result = 0U;
    for (std::size_t index = 0U; index < 8U; ++index)
        result = (result << 8U) | bytes[index];
    return result;
}

bool check_parent_artifact(const FileImage<PARENT_SIZE>& parent,
    CheckerWork& work) noexcept {
    constexpr std::array<std::uint8_t, 8U> magic{{
        'N','E','R','6','3','Z','M','1'}};
    constexpr std::size_t record = 1236U + 2U * 1944U;
    bool valid = true;
    auto check = [&](bool condition) noexcept {
        ++work.parent_checks;
        valid = valid && condition;
    };
    check(parent.exact && digest_is(parent.root, PARENT_ID));
    check(std::memcmp(parent.bytes.data(), magic.data(), magic.size()) == 0);
    check(big_u32(parent.bytes.data() + 8U) == 3U);
    check(big_u64(parent.bytes.data() + 12U) == PARENT_SIZE);
    check(big_u32(parent.bytes.data() + 60U) == 0U);
    check(bytes_are(parent.bytes.data() + 128U, PRODUCT_SET_ID, 32U));
    check(bytes_are(parent.bytes.data() + 484U, PARENT_RESULT_ID, 32U));
    check(big_u32(parent.bytes.data() + 1228U) == 6U);
    check(parent.bytes[record] == 1U && parent.bytes[record + 1U] == 1U);
    check(parent.bytes[record + 2U] == 2U);
    check(big_u64(parent.bytes.data() + record + 304U) == DIMENSION);
    check(bytes_are(parent.bytes.data() + record + 208U,
        ROLE_TWO_VALUE_ID, 32U));
    return valid;
}

bool check_parent_audit(const FileImage<PARENT_AUDIT_SIZE>& audit,
    CheckerWork& work) noexcept {
    constexpr std::array<std::uint8_t, 8U> magic{{
        'N','E','R','6','3','Q','C','1'}};
    bool valid = true;
    auto check = [&](bool condition) noexcept {
        ++work.parent_checks;
        valid = valid && condition;
    };
    check(audit.exact && digest_is(audit.root, PARENT_AUDIT_ID));
    check(std::memcmp(audit.bytes.data(), magic.data(), magic.size()) == 0);
    check(big_u32(audit.bytes.data() + 8U) == 3U);
    check(big_u64(audit.bytes.data() + 12U) == PARENT_AUDIT_SIZE);
    check(big_u32(audit.bytes.data() + 20U) == 0U);
    check(audit.bytes[24U] == 1U && audit.bytes[25U] == 1U
        && audit.bytes[26U] == 1U);
    check(audit.bytes[27U] == 0U);
    check(bytes_are(audit.bytes.data() + 388U, PARENT_CHECKER_ID, 32U));
    return valid;
}

struct Slice final {
    const std::uint8_t* data = nullptr;
    std::size_t size = 0U;
};

struct Sequence final {
    Slice bytes;
    std::uint64_t count = 0U;
};

class WireCursor final {
public:
    WireCursor(const std::array<std::uint8_t, CACHE_SIZE>& bytes,
        CheckerWork& work) noexcept : bytes_(bytes), work_(work) {}

    Slice take(std::size_t count) noexcept {
        ++work_.cursor_takes;
        if (!ok_ || count > bytes_.size() - position_) {
            ok_ = false;
            return {};
        }
        const Slice result{bytes_.data() + position_, count};
        position_ += count;
        work_.cursor_bytes += count;
        return result;
    }

    std::uint8_t u8() noexcept {
        const Slice value = take(1U);
        return value.data == nullptr ? 0U : value.data[0U];
    }

    std::uint32_t u32() noexcept {
        const Slice value = take(4U);
        if (value.data == nullptr) return 0U;
        return static_cast<std::uint32_t>(value.data[0U])
            | (static_cast<std::uint32_t>(value.data[1U]) << 8U)
            | (static_cast<std::uint32_t>(value.data[2U]) << 16U)
            | (static_cast<std::uint32_t>(value.data[3U]) << 24U);
    }

    std::uint64_t u64() noexcept {
        const Slice value = take(8U);
        if (value.data == nullptr) return 0U;
        std::uint64_t result = 0U;
        for (std::size_t index = 0U; index < 8U; ++index)
            result |= static_cast<std::uint64_t>(value.data[index])
                << (8U * index);
        return result;
    }

    void require(bool condition) noexcept {
        ++work_.cursor_predicates;
        ok_ = ok_ && condition;
    }

    void boolean() noexcept {
        const std::uint8_t value = u8();
        require(value <= 1U);
    }

    void text() noexcept {
        const std::uint64_t count = u64();
        require(count <= CACHE_SIZE);
        if (ok_) static_cast<void>(take(static_cast<std::size_t>(count)));
    }

    Sequence vector(std::size_t width) noexcept {
        const std::uint64_t count = u64();
        require(width != 0U && count <= CACHE_SIZE / width);
        if (!ok_ || count > static_cast<std::uint64_t>(
                (bytes_.size() - position_) / width)) {
            ok_ = false;
            return {};
        }
        return {take(static_cast<std::size_t>(count) * width), count};
    }

    bool complete() noexcept {
        ++work_.cursor_predicates;
        return ok_ && position_ == bytes_.size();
    }

private:
    const std::array<std::uint8_t, CACHE_SIZE>& bytes_;
    CheckerWork& work_;
    std::size_t position_ = 0U;
    bool ok_ = true;
};

void discard_certificate(WireCursor& cursor) noexcept {
    cursor.boolean();
    cursor.boolean();
    static_cast<void>(cursor.take(24U));
    static_cast<void>(cursor.take(8U));
    cursor.text();
    cursor.text();
    cursor.text();
}

void discard_certificate_set(WireCursor& cursor) noexcept {
    const std::uint64_t count = cursor.u64();
    cursor.require(count <= 64U);
    for (std::uint64_t index = 0U; index < count; ++index)
        discard_certificate(cursor);
}

void discard_profile(WireCursor& cursor) noexcept {
    cursor.boolean();
    cursor.boolean();
    cursor.boolean();
    static_cast<void>(cursor.take(48U));
    for (std::size_t index = 0U; index < 4U; ++index)
        static_cast<void>(cursor.vector(8U));
    static_cast<void>(cursor.take(24U));
    for (std::size_t index = 0U; index < 4U; ++index) cursor.text();
}

struct Inputs final {
    bool parsed = false;
    std::uint64_t rows = 0U;
    std::uint64_t columns = 0U;
    Sequence baseline;
    Sequence factor;
    Sequence permutation;
    Slice inverse;
    Sequence rhs;
};

Inputs decode_cache(const std::array<std::uint8_t, CACHE_SIZE>& bytes,
    CheckerWork& work) noexcept {
    constexpr std::array<std::uint8_t, 8U> magic{{
        'N','E','F','P','R','B','1',0U}};
    WireCursor cursor(bytes, work);
    Inputs output;
    const Slice header = cursor.take(magic.size());
    cursor.require(header.data != nullptr
        && std::memcmp(header.data, magic.data(), magic.size()) == 0);
    cursor.require(cursor.u32() == 1U);
    cursor.require(cursor.u32() == 0x01020304U);
    cursor.require(cursor.u32() == 8U);
    cursor.require(cursor.u32() == 16U);
    cursor.boolean();
    cursor.text();
    output.rows = cursor.u64();
    output.columns = cursor.u64();
    for (std::size_t index = 0U; index < 9U; ++index) cursor.text();
    cursor.require(cursor.u64() == 3U);
    output.baseline = cursor.vector(16U);
    static_cast<void>(cursor.vector(16U));
    static_cast<void>(cursor.vector(16U));
    discard_certificate_set(cursor);
    static_cast<void>(cursor.vector(16U));
    static_cast<void>(cursor.take(16U));
    cursor.text();
    output.factor = cursor.vector(8U);
    const std::uint64_t permutation_count = cursor.u64();
    cursor.require(permutation_count <= 4096U);
    output.permutation = {
        cursor.take(static_cast<std::size_t>(permutation_count) * 8U),
        permutation_count};
    output.inverse = cursor.take(16U);
    cursor.text();
    cursor.text();
    cursor.text();
    output.rhs = cursor.vector(16U);
    static_cast<void>(cursor.vector(16U));
    static_cast<void>(cursor.take(16U));
    cursor.text();
    cursor.text();
    static_cast<void>(cursor.vector(8U));
    cursor.text();
    cursor.text();
    cursor.text();
    discard_profile(cursor);
    cursor.text();
    cursor.text();
    cursor.require(cursor.u64() == 3U);
    static_cast<void>(cursor.vector(16U));
    static_cast<void>(cursor.vector(16U));
    static_cast<void>(cursor.vector(16U));
    discard_certificate_set(cursor);
    cursor.text();
    cursor.text();
    cursor.text();
    output.parsed = cursor.complete();
    return output;
}

Quad load_quad(Slice bytes, std::size_t index, CheckerWork& work) noexcept {
    ++work.scalar_loads;
    Quad value{};
    if (bytes.data != nullptr && (index + 1U) * sizeof(value) <= bytes.size)
        std::memcpy(&value, bytes.data + index * sizeof(value), sizeof(value));
    return value;
}

double load_double(Slice bytes, std::size_t index,
    CheckerWork& work) noexcept {
    ++work.scalar_loads;
    double value = 0.0;
    if (bytes.data != nullptr && (index + 1U) * sizeof(value) <= bytes.size)
        std::memcpy(&value, bytes.data + index * sizeof(value), sizeof(value));
    return value;
}

std::uint64_t load_index(Slice bytes, std::size_t index,
    CheckerWork& work) noexcept {
    ++work.scalar_loads;
    std::uint64_t value = 0U;
    if (bytes.data != nullptr && (index + 1U) * sizeof(value) <= bytes.size)
        std::memcpy(&value, bytes.data + index * sizeof(value), sizeof(value));
    return value;
}

Quad load_parent_quad(const std::array<std::uint8_t, PARENT_SIZE>& bytes,
    std::size_t offset, CheckerWork& work) noexcept {
    ++work.scalar_loads;
    std::array<std::uint8_t, 16U> native{};
    for (std::size_t index = 0U; index < native.size(); ++index)
        native[index] = bytes[offset + native.size() - 1U - index];
    Quad value{};
    std::memcpy(&value, native.data(), sizeof(value));
    return value;
}

struct LinearSolve final {
    bool valid = false;
    std::array<Quad, DIMENSION> value{};
};

void compensated_push(Quad term, Quad& sum, Quad& correction) noexcept {
    const Quad next = sum + term;
    correction += fabsq(sum) >= fabsq(term)
        ? (sum - next) + term : (term - next) + sum;
    sum = next;
}

LinearSolve replay_factor(const std::array<Quad, FACTOR_COMPONENTS>& upper,
    const std::array<std::uint64_t, DIMENSION>& permutation,
    const std::array<Quad, DIMENSION>& rhs, Quad inverse,
    CheckerWork& work) noexcept {
    ++work.factor_solves;
    std::array<Quad, DIMENSION> transformed{};
    std::array<Quad, DIMENSION> triangular{};
    std::array<Quad, DIMENSION> ordered{};
    bool valid = finiteq(inverse) != 0 && inverse > static_cast<Quad>(0.0);
    ++work.arithmetic_checks;
    for (std::size_t row = 0U; row < DIMENSION && valid; ++row) {
        valid = permutation[row] < DIMENSION;
        ++work.arithmetic_checks;
        if (!valid) break;
        transformed[row] = rhs[permutation[row]] * inverse;
        ++work.arithmetic_checks;
        Quad sum = static_cast<Quad>(0.0);
        Quad correction = static_cast<Quad>(0.0);
        for (std::size_t column = 0U; column < row; ++column) {
            compensated_push(upper[column * DIMENSION + row]
                * triangular[column], sum, correction);
            ++work.factor_terms;
        }
        const Quad diagonal = upper[row * DIMENSION + row];
        valid = finiteq(transformed[row]) != 0 && finiteq(diagonal) != 0
            && diagonal != static_cast<Quad>(0.0);
        ++work.arithmetic_checks;
        if (!valid) break;
        triangular[row] = (transformed[row] - (sum + correction)) / diagonal;
        valid = finiteq(triangular[row]) != 0;
        ++work.arithmetic_checks;
        ++work.factor_divisions;
    }
    for (std::size_t remaining = DIMENSION; remaining > 0U && valid;) {
        const std::size_t row = --remaining;
        Quad sum = static_cast<Quad>(0.0);
        Quad correction = static_cast<Quad>(0.0);
        for (std::size_t column = row + 1U; column < DIMENSION; ++column) {
            compensated_push(upper[row * DIMENSION + column]
                * ordered[column], sum, correction);
            ++work.factor_terms;
        }
        const Quad diagonal = upper[row * DIMENSION + row];
        valid = finiteq(diagonal) != 0 && diagonal != static_cast<Quad>(0.0);
        ++work.arithmetic_checks;
        if (!valid) break;
        ordered[row] = (triangular[row] - (sum + correction)) / diagonal;
        valid = finiteq(ordered[row]) != 0;
        ++work.arithmetic_checks;
        ++work.factor_divisions;
    }
    LinearSolve result;
    if (valid) {
        for (std::size_t index = 0U; index < DIMENSION; ++index)
            result.value[permutation[index]] = ordered[index];
    }
    result.valid = valid && work.factor_terms % 10302U == 0U
        && work.factor_divisions % 204U == 0U;
    return result;
}

struct CandidateControlAccounting final {
    std::uint64_t fixture_mutations = 0U;
    std::uint64_t small_solve_calls = 0U;
    std::uint64_t small_solve_terms = 0U;
    std::uint64_t small_solve_divisions = 0U;
    std::uint64_t witness_operations = 0U;
    std::uint64_t witness_checks = 0U;
};

bool independently_check_small_solve(CandidateControlAccounting& candidate,
    CheckerWork& work) noexcept {
    candidate.small_solve_calls = 1U;
    candidate.small_solve_terms = 6U;
    candidate.small_solve_divisions = 6U;
    candidate.witness_checks = 3U;
    ++work.control_small_solve_calls;
    work.control_small_solve_terms += 6U;
    work.control_small_solve_divisions += 6U;
    const Quad forward0 = static_cast<Quad>(-6.0)
        / static_cast<Quad>(2.0);
    const Quad forward1 = (static_cast<Quad>(-3.0)
        - static_cast<Quad>(1.0) * forward0) / static_cast<Quad>(3.0);
    const Quad forward2 = (static_cast<Quad>(51.0)
        - static_cast<Quad>(-1.0) * forward0
        - static_cast<Quad>(2.0) * forward1) / static_cast<Quad>(4.0);
    const Quad solution2 = forward2 / static_cast<Quad>(4.0);
    const Quad solution1 = (forward1 - static_cast<Quad>(2.0) * solution2)
        / static_cast<Quad>(3.0);
    const Quad solution0 = (forward0 - static_cast<Quad>(1.0) * solution1
        - static_cast<Quad>(-1.0) * solution2) / static_cast<Quad>(2.0);
    const std::array<Quad, 3U> actual{{solution0, solution1, solution2}};
    const std::array<Quad, 3U> expected{{static_cast<Quad>(1.0),
        static_cast<Quad>(-2.0), static_cast<Quad>(3.0)}};
    bool exact = true;
    for (std::size_t index = 0U; index < actual.size(); ++index) {
        exact = exact && std::memcmp(&actual[index], &expected[index],
            sizeof(Quad)) == 0;
        ++work.control_witness_checks;
    }
    return exact;
}

bool independently_check_state_difference(
    CandidateControlAccounting& candidate, CheckerWork& work) noexcept {
    candidate.witness_operations = 4U;
    candidate.witness_checks = 4U;
    const Quad one = static_cast<Quad>(1.0);
    const Quad direction = ldexpq(one, -113);
    const Quad state = fmaq(one, direction, one);
    ++work.control_witness_operations;
    const Quad difference = state - one;
    ++work.control_witness_operations;
    const Quad inferred = difference / one;
    ++work.control_witness_operations;
    bool valid = std::memcmp(&state, &one, sizeof(Quad)) == 0;
    ++work.control_witness_checks;
    valid = valid && direction != static_cast<Quad>(0.0);
    ++work.control_witness_checks;
    valid = valid && difference == static_cast<Quad>(0.0);
    ++work.control_witness_checks;
    valid = valid && inferred == static_cast<Quad>(0.0);
    ++work.control_witness_checks;
    return valid;
}

bool normal_or_zero(Quad value) noexcept {
    return finiteq(value) != 0 && (value == static_cast<Quad>(0.0)
        || fabsq(value) >= ldexpq(static_cast<Quad>(1.0), -16382));
}

Quad round_up_add(Quad left, Quad right) noexcept {
    if (left == static_cast<Quad>(0.0) && right == static_cast<Quad>(0.0))
        return static_cast<Quad>(0.0);
    return nextafterq(left + right, HUGE_VALQ);
}

Quad round_up_multiply(Quad left, Quad right) noexcept {
    if (left == static_cast<Quad>(0.0) || right == static_cast<Quad>(0.0))
        return static_cast<Quad>(0.0);
    return nextafterq(left * right, HUGE_VALQ);
}

Quad round_up_divide(Quad numerator, Quad denominator) noexcept {
    if (numerator == static_cast<Quad>(0.0)) return static_cast<Quad>(0.0);
    return nextafterq(numerator / denominator, HUGE_VALQ);
}

struct ErrorFreePair final {
    Quad high = static_cast<Quad>(0.0);
    Quad low = static_cast<Quad>(0.0);
    bool finite = false;
    bool normal = false;
};

ErrorFreePair exact_product(Quad left, Quad right,
    CheckerWork& work) noexcept {
    ++work.two_products;
    ErrorFreePair pair;
    pair.high = left * right;
    pair.low = fmaq(left, right, -pair.high);
    pair.finite = finiteq(pair.high) != 0 && finiteq(pair.low) != 0;
    pair.normal = normal_or_zero(left) && normal_or_zero(right)
        && normal_or_zero(pair.high) && normal_or_zero(pair.low)
        && (left == static_cast<Quad>(0.0)
            || right == static_cast<Quad>(0.0)
            || pair.high != static_cast<Quad>(0.0)
            || pair.low != static_cast<Quad>(0.0));
    return pair;
}

ErrorFreePair exact_sum(Quad left, Quad right,
    CheckerWork& work) noexcept {
    ++work.two_sums;
    ErrorFreePair pair;
    pair.high = left + right;
    const Quad virtual_right = pair.high - left;
    pair.low = (left - (pair.high - virtual_right))
        + (right - virtual_right);
    pair.finite = finiteq(pair.high) != 0 && finiteq(pair.low) != 0;
    pair.normal = normal_or_zero(left) && normal_or_zero(right)
        && normal_or_zero(pair.high) && normal_or_zero(virtual_right)
        && normal_or_zero(pair.low);
    return pair;
}

struct EnclosedDot final {
    Quad value = static_cast<Quad>(0.0);
    Quad bound = static_cast<Quad>(0.0);
    Quad magnitude = static_cast<Quad>(0.0);
    bool exact = false;
    bool no_underflow = false;
};

EnclosedDot replay_dot(const Quad* left, const Quad* right,
    std::size_t count, CheckerWork& work) noexcept {
    ++work.dot_calls;
    work.dot_terms += count;
    EnclosedDot result;
    if (count == 0U) return result;
    const ErrorFreePair first = exact_product(left[0U], right[0U], work);
    Quad leading = first.high;
    Quad trailing = first.low;
    bool exact = first.finite;
    bool no_underflow = first.normal;
    result.magnitude = round_up_multiply(fabsq(left[0U]), fabsq(right[0U]));
    for (std::size_t index = 1U; index < count; ++index) {
        const ErrorFreePair product = exact_product(left[index], right[index],
            work);
        const ErrorFreePair sum = exact_sum(leading, product.high, work);
        const Quad local = sum.low + product.low;
        trailing += local;
        leading = sum.high;
        exact = exact && product.finite && sum.finite;
        no_underflow = no_underflow && product.normal && sum.normal
            && normal_or_zero(local) && normal_or_zero(trailing);
        result.magnitude = round_up_add(result.magnitude,
            round_up_multiply(fabsq(left[index]), fabsq(right[index])));
    }
    result.value = leading + trailing;
    no_underflow = no_underflow && normal_or_zero(result.value);
    const Quad one = static_cast<Quad>(1.0);
    const Quad unit = ldexpq(one, -112);
    const Quad scaled = static_cast<Quad>(count) * unit;
    const Quad gamma = round_up_divide(scaled, one - scaled);
    const Quad numerator = round_up_add(
        round_up_multiply(unit, fabsq(result.value)),
        round_up_multiply(round_up_multiply(gamma, gamma), result.magnitude));
    result.bound = round_up_divide(numerator, one - unit);
    result.exact = exact && finiteq(result.value) != 0
        && finiteq(result.bound) != 0
        && result.bound >= static_cast<Quad>(0.0);
    result.no_underflow = no_underflow;
    return result;
}

void append_big_u64(r63zm::Sha256& hash, std::uint64_t value) noexcept {
    std::array<std::uint8_t, 8U> encoded{};
    for (std::size_t index = 0U; index < encoded.size(); ++index)
        encoded[index] = static_cast<std::uint8_t>(value
            >> (56U - index * 8U));
    hash.update(encoded);
}

void field_header(r63zm::Sha256& hash, std::uint8_t tag,
    std::uint64_t bytes) noexcept {
    hash.update_byte(tag);
    append_big_u64(hash, bytes);
}

void field_bytes(r63zm::Sha256& hash, std::uint8_t tag,
    const std::uint8_t* bytes, std::size_t count) noexcept {
    field_header(hash, tag, count);
    hash.update(std::span<const std::uint8_t>(bytes, count));
}

void field_text(r63zm::Sha256& hash, std::uint8_t tag,
    const char* text) noexcept {
    field_bytes(hash, tag, reinterpret_cast<const std::uint8_t*>(text),
        std::strlen(text));
}

void field_u64(r63zm::Sha256& hash, std::uint8_t tag,
    std::uint64_t value) noexcept {
    field_header(hash, tag, 8U);
    append_big_u64(hash, value);
}

void field_digest(r63zm::Sha256& hash, std::uint8_t tag,
    const Digest& value) noexcept {
    field_bytes(hash, tag, value.data(), value.size());
}

std::array<std::uint8_t, 16U> canonical_quad(Quad value) noexcept {
    std::array<std::uint8_t, 16U> native{};
    std::array<std::uint8_t, 16U> output{};
    std::memcpy(native.data(), &value, native.size());
    for (std::size_t index = 0U; index < output.size(); ++index)
        output[index] = native[output.size() - 1U - index];
    return output;
}

void record_hash(CheckerWork& work, std::uint64_t bytes) noexcept {
    ++work.hash_calls;
    work.hash_bytes += bytes;
}

template <std::size_t Count>
Digest root_quad_vector(const std::array<Quad, Count>& values,
    CheckerWork& work) noexcept {
    constexpr const char* domain =
        "nextengine.nonlocal.r63zn.binary128-vector.v1";
    r63zm::Sha256 hash;
    field_text(hash, 1U, domain);
    field_u64(hash, 2U, Count);
    field_header(hash, 3U, Count * 16U);
    for (Quad value : values) hash.update(canonical_quad(value));
    record_hash(work, std::strlen(domain) + 1667U);
    return hash.finish();
}

template <std::size_t Count>
Digest root_parent_quad_vector(const std::array<Quad, Count>& values,
    CheckerWork& work) noexcept {
    constexpr const char* domain =
        "nextengine.nonlocal.r63zm.binary128-vector.v1";
    r63zm::Sha256 hash;
    field_text(hash, 1U, domain);
    field_u64(hash, 2U, Count);
    field_header(hash, 3U, Count * 16U);
    for (Quad value : values) hash.update(canonical_quad(value));
    record_hash(work, std::strlen(domain) + 1667U);
    return hash.finish();
}

Digest root_factor(Slice source, std::uint64_t count,
    CheckerWork& work) noexcept {
    constexpr const char* domain =
        "nextengine.nonlocal.r63zn.binary64-factor.v1";
    r63zm::Sha256 hash;
    field_text(hash, 1U, domain);
    field_u64(hash, 2U, DIMENSION);
    field_u64(hash, 3U, count);
    field_header(hash, 4U, count * 8U);
    for (std::size_t index = 0U; index < count; ++index)
        for (std::size_t byte = 0U; byte < 8U; ++byte)
            hash.update_byte(source.data[index * 8U + 7U - byte]);
    record_hash(work, std::strlen(domain) + 83284U);
    return hash.finish();
}

Digest root_permutation(
    const std::array<std::uint64_t, DIMENSION>& values,
    CheckerWork& work) noexcept {
    constexpr const char* domain = "nextengine.nonlocal.r63zn.permutation.v1";
    r63zm::Sha256 hash;
    field_text(hash, 1U, domain);
    field_u64(hash, 2U, values.size());
    field_header(hash, 3U, values.size() * 8U);
    for (std::uint64_t value : values) append_big_u64(hash, value);
    record_hash(work, std::strlen(domain) + 851U);
    return hash.finish();
}

Digest root_scalar(const char* domain, Quad value,
    CheckerWork& work) noexcept {
    r63zm::Sha256 hash;
    field_text(hash, 1U, domain);
    const auto bytes = canonical_quad(value);
    field_bytes(hash, 2U, bytes.data(), bytes.size());
    record_hash(work, std::strlen(domain) + 34U);
    return hash.finish();
}

Digest root_rho(const EnclosedDot& rho, CheckerWork& work) noexcept {
    constexpr const char* domain = "nextengine.nonlocal.r63zn.rho0.v1";
    r63zm::Sha256 hash;
    field_text(hash, 1U, domain);
    field_u64(hash, 2U, rho.exact ? 1U : 0U);
    field_u64(hash, 3U, rho.no_underflow ? 1U : 0U);
    const auto value = canonical_quad(rho.value);
    const auto bound = canonical_quad(rho.bound);
    const auto magnitude = canonical_quad(rho.magnitude);
    field_bytes(hash, 4U, value.data(), value.size());
    field_bytes(hash, 5U, bound.data(), bound.size());
    field_bytes(hash, 6U, magnitude.data(), magnitude.size());
    record_hash(work, std::strlen(domain) + 118U);
    return hash.finish();
}

template <std::size_t Count>
Digest root_digest_set(const char* domain,
    const std::array<Digest, Count>& values, CheckerWork& work) noexcept {
    r63zm::Sha256 hash;
    field_text(hash, 1U, domain);
    field_u64(hash, 2U, values.size());
    field_header(hash, 3U, values.size() * 32U);
    for (const Digest& value : values) hash.update(value);
    record_hash(work, std::strlen(domain) + 35U + values.size() * 32U);
    return hash.finish();
}

Digest root_candidate_work(
    const std::array<std::uint64_t, CANDIDATE_WORK_FIELDS>& values,
    CheckerWork& work) noexcept {
    constexpr const char* domain =
        "nextengine.nonlocal.r63zn.candidate-work.v1";
    r63zm::Sha256 hash;
    field_text(hash, 1U, domain);
    field_u64(hash, 2U, values.size());
    field_header(hash, 3U, values.size() * 8U);
    for (std::uint64_t value : values) append_big_u64(hash, value);
    record_hash(work, std::strlen(domain) + 571U);
    return hash.finish();
}

Digest root_event(std::uint64_t ordinal, const Digest& left,
    const Digest& right, CheckerWork& work) noexcept {
    constexpr const char* domain = "nextengine.nonlocal.r63zn.event.v1";
    r63zm::Sha256 hash;
    field_text(hash, 1U, domain);
    field_u64(hash, 2U, ordinal);
    field_digest(hash, 3U, left);
    field_digest(hash, 4U, right);
    record_hash(work, std::strlen(domain) + 108U);
    return hash.finish();
}

Digest root_trace(std::uint64_t route,
    const std::array<Digest, EVENT_SLOTS>& events,
    std::size_t event_count, CheckerWork& work) noexcept {
    constexpr const char* domain = "nextengine.nonlocal.r63zn.trace.v1";
    r63zm::Sha256 hash;
    field_text(hash, 1U, domain);
    field_u64(hash, 2U, route);
    field_u64(hash, 3U, event_count);
    field_header(hash, 4U, event_count * 32U);
    for (std::size_t index = 0U; index < event_count; ++index)
        hash.update(events[index]);
    record_hash(work, std::strlen(domain) + 52U + event_count * 32U);
    return hash.finish();
}

Digest root_candidate_result(std::uint64_t route,
    const std::array<Digest, 3U>& inputs, const Digest& trace,
    const Digest& candidate_work, CheckerWork& work) noexcept {
    constexpr const char* domain = "nextengine.nonlocal.r63zn.result.v1";
    r63zm::Sha256 hash;
    field_text(hash, 1U, domain);
    field_u64(hash, 2U, 3U);
    field_u64(hash, 3U, route);
    field_header(hash, 4U, inputs.size() * 32U);
    for (const Digest& input : inputs) hash.update(input);
    field_digest(hash, 5U, trace);
    field_digest(hash, 6U, candidate_work);
    field_u64(hash, 7U, RECEIPT_SIZE);
    record_hash(work, std::strlen(domain) + 247U);
    return hash.finish();
}

void store_u32(std::uint8_t* target, std::uint32_t value) noexcept {
    for (std::size_t index = 0U; index < 4U; ++index)
        target[index] = static_cast<std::uint8_t>(
            value >> (24U - index * 8U));
}

void store_u64(std::uint8_t* target, std::uint64_t value) noexcept {
    for (std::size_t index = 0U; index < 8U; ++index)
        target[index] = static_cast<std::uint8_t>(
            value >> (56U - index * 8U));
}

void store_digest(std::uint8_t* target, const Digest& value) noexcept {
    std::memcpy(target, value.data(), value.size());
}

struct Replay final {
    bool valid = false;
    std::uint32_t candidate_route = 0U;
    std::array<std::uint8_t, RECEIPT_SIZE> expected{};
    Digest result{};
};

Replay reconstruct_early_rejection(std::uint32_t route,
    const FileImage<CACHE_SIZE>& cache,
    const FileImage<PARENT_SIZE>& parent,
    const FileImage<PARENT_AUDIT_SIZE>& parent_audit,
    std::uint64_t read_calls, std::uint64_t artifact_checks,
    std::uint64_t audit_checks, std::uint64_t parse_calls,
    std::uint64_t parse_bytes, std::uint64_t parse_predicates,
    std::uint64_t route_decisions, CheckerWork& work) noexcept {
    Replay replay;
    replay.candidate_route = route;
    const bool cache_consumed = read_calls >= 1U;
    const bool parent_consumed = read_calls >= 2U;
    const bool audit_consumed = read_calls >= 3U;
    const std::uint64_t input_bytes =
        (cache_consumed ? cache.bytes_read : 0U)
        + (parent_consumed ? parent.bytes_read : 0U)
        + (audit_consumed ? parent_audit.bytes_read : 0U);
    const std::uint64_t input_hashes =
        (cache_consumed && cache.exact ? 1U : 0U)
        + (parent_consumed && parent.exact ? 1U : 0U)
        + (audit_consumed && parent_audit.exact ? 1U : 0U);
    const std::uint64_t input_open_calls = read_calls;
    const std::uint64_t input_seek_calls =
        (cache_consumed && cache.opened ? 2U : 0U)
        + (parent_consumed && parent.opened ? 2U : 0U)
        + (audit_consumed && parent_audit.opened ? 2U : 0U);
    const std::uint64_t input_tell_calls =
        (cache_consumed && cache.opened ? 1U : 0U)
        + (parent_consumed && parent.opened ? 1U : 0U)
        + (audit_consumed && parent_audit.opened ? 1U : 0U);
    const std::uint64_t input_read_calls =
        (cache_consumed && cache.observed_size == CACHE_SIZE ? 1U : 0U)
        + (parent_consumed && parent.observed_size == PARENT_SIZE ? 1U : 0U)
        + (audit_consumed
            && parent_audit.observed_size == PARENT_AUDIT_SIZE ? 1U : 0U);
    const std::uint64_t input_close_calls = input_tell_calls;
    const std::uint64_t work_bytes =
        std::strlen("nextengine.nonlocal.r63zn.candidate-work.v1") + 571U;
    const std::uint64_t trace_bytes =
        std::strlen("nextengine.nonlocal.r63zn.trace.v1") + 52U;
    const std::uint64_t result_bytes =
        std::strlen("nextengine.nonlocal.r63zn.result.v1") + 247U;
    std::array<std::uint64_t, CANDIDATE_WORK_FIELDS> candidate_work{};
    candidate_work[RoundingModeSetCalls] = 1U;
    candidate_work[RoundingModeChecks] = 1U;
    candidate_work[InputReadAttempts] = read_calls;
    candidate_work[InputOpenCalls] = input_open_calls;
    candidate_work[InputSeekCalls] = input_seek_calls;
    candidate_work[InputTellCalls] = input_tell_calls;
    candidate_work[InputReadCalls] = input_read_calls;
    candidate_work[InputTrailingChecks] = input_read_calls;
    candidate_work[InputCloseCalls] = input_close_calls;
    candidate_work[InputReadBytes] = input_bytes;
    candidate_work[InputHashCalls] = input_hashes;
    candidate_work[InputHashBytes] =
        (cache_consumed && cache.exact ? CACHE_SIZE : 0U)
        + (parent_consumed && parent.exact ? PARENT_SIZE : 0U)
        + (audit_consumed && parent_audit.exact ? PARENT_AUDIT_SIZE : 0U);
    candidate_work[CacheTakeCalls] = parse_calls;
    candidate_work[CacheTakeBytes] = parse_bytes;
    candidate_work[CachePredicates] = parse_predicates;
    candidate_work[ParentArtifactChecks] = artifact_checks;
    candidate_work[ParentAuditChecks] = audit_checks;
    candidate_work[StructuralRootCalls] = 1U;
    candidate_work[StructuralRootBytes] = work_bytes;
    candidate_work[TraceRootCalls] = 1U;
    candidate_work[TraceRootBytes] = trace_bytes;
    candidate_work[ResultRootCalls] = 1U;
    candidate_work[ResultRootBytes] = result_bytes;
    candidate_work[ReceiptFieldsWritten] = 102U;
    candidate_work[ReceiptOpenCalls] = 1U;
    candidate_work[ReceiptWriteCalls] = 1U;
    candidate_work[ReceiptCloseCalls] = 1U;
    candidate_work[ReceiptWriteBytes] = RECEIPT_SIZE;
    candidate_work[RouteDecisions] = route_decisions;
    candidate_work[ReceiptZeroFillBytes] = RECEIPT_SIZE;
    const Digest candidate_work_digest = root_candidate_work(candidate_work,
        work);
    const Digest zero{};
    const std::array<Digest, 3U> inputs{{
        cache_consumed && cache.exact ? cache.root : zero,
        parent_consumed && parent.exact ? parent.root : zero,
        audit_consumed && parent_audit.exact ? parent_audit.root : zero}};
    const std::array<Digest, EVENT_SLOTS> events{};
    const Digest trace = root_trace(route, events, 0U, work);
    replay.result = root_candidate_result(route, inputs, trace,
        candidate_work_digest, work);

    constexpr std::array<std::uint8_t, 8U> magic{{
        'N','E','R','6','3','Z','N','1'}};
    std::memcpy(replay.expected.data(), magic.data(), magic.size());
    store_u32(replay.expected.data() + VERSION_AT, 3U);
    store_u64(replay.expected.data() + TOTAL_AT, RECEIPT_SIZE);
    store_u32(replay.expected.data() + ROUTE_AT, route);
    replay.expected[FLAGS_AT] = 0U;
    replay.expected[FLAGS_AT + 1U] = 0U;
    replay.expected[FLAGS_AT + 2U] = 0U;
    store_u64(replay.expected.data() + CACHE_SIZE_AT,
        cache_consumed ? cache.observed_size : 0U);
    store_digest(replay.expected.data() + CACHE_ROOT_AT, inputs[0U]);
    store_u64(replay.expected.data() + PARENT_SIZE_AT,
        parent_consumed ? parent.observed_size : 0U);
    store_digest(replay.expected.data() + PARENT_ROOT_AT, inputs[1U]);
    store_u64(replay.expected.data() + PARENT_AUDIT_SIZE_AT,
        audit_consumed ? parent_audit.observed_size : 0U);
    store_digest(replay.expected.data() + PARENT_AUDIT_ROOT_AT, inputs[2U]);
    const std::array<std::size_t, 12U> semantic_offsets{{BUNDLE_ROOT_AT,
        FACTOR_ROOT_AT, PERMUTATION_ROOT_AT, INVERSE_ROOT_AT, RHS_ROOT_AT,
        BASELINE_ROOT_AT, X0_ROOT_AT, HX0_ROOT_AT, RESIDUAL_ROOT_AT,
        RESIDUAL_BOUND_ROOT_AT, Z0_ROOT_AT, RHO_ROOT_AT}};
    for (std::size_t offset : semantic_offsets)
        store_digest(replay.expected.data() + offset, zero);
    for (std::size_t index = 0U; index < candidate_work.size(); ++index)
        store_u64(replay.expected.data() + CANDIDATE_WORK_AT + index * 8U,
            candidate_work[index]);
    store_u64(replay.expected.data() + EVENT_COUNT_AT, 0U);
    for (std::size_t index = 0U; index < events.size(); ++index)
        store_digest(replay.expected.data() + EVENTS_AT + index * 32U, zero);
    store_digest(replay.expected.data() + TRACE_AT, trace);
    store_digest(replay.expected.data() + RESULT_AT, replay.result);
    replay.valid = true;
    return replay;
}

using CandidateSemantic = std::array<Digest, 12U>;

std::array<std::uint64_t, CANDIDATE_WORK_FIELDS> expected_candidate_work(
    const ControlChoice& control,
    const CandidateControlAccounting& control_work,
    std::uint64_t factor_solves, std::uint64_t factor_terms,
    std::uint64_t factor_divisions, std::uint64_t finite_checks,
    std::uint64_t baseline_comparisons, std::uint64_t residual_updates,
    std::uint64_t residual_dot_calls, std::uint64_t residual_dot_terms,
    std::uint64_t rho_dot_calls, std::uint64_t rho_dot_terms,
    std::uint64_t two_products, std::uint64_t two_sums,
    std::uint64_t positivity_checks, std::uint64_t canonical_calls,
    std::uint64_t canonical_bytes, std::size_t event_count,
    std::uint64_t route_decisions, bool parent_product_consumed) noexcept {
    constexpr std::uint64_t input_bytes =
        CACHE_SIZE + PARENT_SIZE + PARENT_AUDIT_SIZE;
    std::array<std::uint64_t, CANDIDATE_WORK_FIELDS> candidate{};
    candidate[RoundingModeSetCalls] = 1U;
    candidate[RoundingModeChecks] = 1U;
    candidate[InputReadAttempts] = 3U;
    candidate[InputOpenCalls] = 3U;
    candidate[InputSeekCalls] = 6U;
    candidate[InputTellCalls] = 3U;
    candidate[InputReadCalls] = 3U;
    candidate[InputTrailingChecks] = 3U;
    candidate[InputCloseCalls] = 3U;
    candidate[InputReadBytes] = input_bytes;
    candidate[InputHashCalls] = 3U;
    candidate[InputHashBytes] = input_bytes;
    candidate[CacheTakeCalls] = 168U;
    candidate[CacheTakeBytes] = CACHE_SIZE;
    candidate[CachePredicates] = 88U;
    candidate[ParentArtifactChecks] = 12U;
    candidate[ParentAuditChecks] = 8U;
    candidate[FactorDecodes] = FACTOR_COMPONENTS;
    candidate[PermutationDecodes] = DIMENSION;
    candidate[QuadDecodes] =
        (parent_product_consumed ? 3U : 2U) * DIMENSION + 1U;
    candidate[FactorFiniteChecks] = FACTOR_COMPONENTS;
    candidate[PermutationRangeChecks] = DIMENSION;
    candidate[PermutationUniquenessChecks] = DIMENSION;
    candidate[QuadFiniteChecks] =
        (parent_product_consumed ? 3U : 2U) * DIMENSION + 1U;
    candidate[InversePositiveChecks] = 1U;
    candidate[FactorSolves] = factor_solves;
    candidate[FactorTerms] = factor_terms;
    candidate[FactorDivisions] = factor_divisions;
    candidate[SolveFiniteChecks] = finite_checks;
    candidate[BaselineComparisons] = baseline_comparisons;
    candidate[ResidualUpdates] = residual_updates;
    candidate[ResidualDotCalls] = residual_dot_calls;
    candidate[ResidualDotTerms] = residual_dot_terms;
    candidate[RhoDotCalls] = rho_dot_calls;
    candidate[RhoDotTerms] = rho_dot_terms;
    candidate[TwoProducts] = two_products;
    candidate[TwoSums] = two_sums;
    candidate[DotExactChecks] = residual_dot_calls + rho_dot_calls;
    candidate[DotUnderflowChecks] = residual_dot_calls + rho_dot_calls;
    candidate[PositivityChecks] = positivity_checks;
    candidate[CanonicalRootCalls] = canonical_calls;
    candidate[CanonicalRootBytes] = canonical_bytes;
    candidate[Role2ValueRootComparisons] = parent_product_consumed ? 1U : 0U;
    candidate[StructuralRootCalls] = 3U;
    candidate[StructuralRootBytes] =
        std::strlen("nextengine.nonlocal.r63zn.input-bundle.v1") + 291U
        + std::strlen("nextengine.nonlocal.r63zn.parent-set.v1") + 131U
        + std::strlen("nextengine.nonlocal.r63zn.candidate-work.v1") + 571U;
    candidate[EventRootCalls] = event_count;
    candidate[EventRootBytes] = event_count
        * (std::strlen("nextengine.nonlocal.r63zn.event.v1") + 108U);
    candidate[TraceRootCalls] = 1U;
    candidate[TraceRootBytes] =
        std::strlen("nextengine.nonlocal.r63zn.trace.v1") + 52U
        + event_count * 32U;
    candidate[ResultRootCalls] = 1U;
    candidate[ResultRootBytes] =
        std::strlen("nextengine.nonlocal.r63zn.result.v1") + 247U;
    candidate[ReceiptFieldsWritten] = 102U;
    candidate[ReceiptOpenCalls] = 1U;
    candidate[ReceiptWriteCalls] = 1U;
    candidate[ReceiptCloseCalls] = 1U;
    candidate[ReceiptWriteBytes] = RECEIPT_SIZE;
    candidate[RouteDecisions] = route_decisions;
    candidate[ControlSelectorChecks] = control.selector_checks;
    candidate[ControlSelectorBytes] = control.selector_bytes;
    candidate[ControlFixtureMutations] = control_work.fixture_mutations;
    candidate[ControlSmallSolveCalls] = control_work.small_solve_calls;
    candidate[ControlSmallSolveTerms] = control_work.small_solve_terms;
    candidate[ControlSmallSolveDivisions] = control_work.small_solve_divisions;
    candidate[ControlWitnessOperations] = control_work.witness_operations;
    candidate[ControlWitnessChecks] = control_work.witness_checks;
    candidate[ReceiptZeroFillBytes] = RECEIPT_SIZE;
    candidate[PackageAllocations] = 0U;
    return candidate;
}

Replay encode_replay(std::uint32_t route,
    const std::array<std::uint8_t, 3U>& flags,
    const FileImage<CACHE_SIZE>& cache,
    const FileImage<PARENT_SIZE>& parent,
    const FileImage<PARENT_AUDIT_SIZE>& parent_audit,
    const CandidateSemantic& semantic,
    const std::array<std::uint64_t, CANDIDATE_WORK_FIELDS>& candidate_work,
    const Digest& candidate_work_digest,
    const std::array<Digest, EVENT_SLOTS>& events,
    std::size_t event_count, CheckerWork& work) noexcept {
    Replay replay;
    replay.candidate_route = route;
    const std::array<Digest, 3U> inputs{{cache.root, parent.root,
        parent_audit.root}};
    const Digest trace = root_trace(route, events, event_count, work);
    replay.result = root_candidate_result(route, inputs, trace,
        candidate_work_digest, work);
    constexpr std::array<std::uint8_t, 8U> magic{{
        'N','E','R','6','3','Z','N','1'}};
    std::memcpy(replay.expected.data(), magic.data(), magic.size());
    store_u32(replay.expected.data() + VERSION_AT, 3U);
    store_u64(replay.expected.data() + TOTAL_AT, RECEIPT_SIZE);
    store_u32(replay.expected.data() + ROUTE_AT, route);
    for (std::size_t index = 0U; index < flags.size(); ++index)
        replay.expected[FLAGS_AT + index] = flags[index];
    store_u64(replay.expected.data() + CACHE_SIZE_AT, cache.observed_size);
    store_digest(replay.expected.data() + CACHE_ROOT_AT, cache.root);
    store_u64(replay.expected.data() + PARENT_SIZE_AT, parent.observed_size);
    store_digest(replay.expected.data() + PARENT_ROOT_AT, parent.root);
    store_u64(replay.expected.data() + PARENT_AUDIT_SIZE_AT,
        parent_audit.observed_size);
    store_digest(replay.expected.data() + PARENT_AUDIT_ROOT_AT,
        parent_audit.root);
    constexpr std::array<std::size_t, 12U> offsets{{BUNDLE_ROOT_AT,
        FACTOR_ROOT_AT, PERMUTATION_ROOT_AT, INVERSE_ROOT_AT, RHS_ROOT_AT,
        BASELINE_ROOT_AT, X0_ROOT_AT, HX0_ROOT_AT, RESIDUAL_ROOT_AT,
        RESIDUAL_BOUND_ROOT_AT, Z0_ROOT_AT, RHO_ROOT_AT}};
    for (std::size_t index = 0U; index < offsets.size(); ++index)
        store_digest(replay.expected.data() + offsets[index], semantic[index]);
    for (std::size_t index = 0U; index < candidate_work.size(); ++index)
        store_u64(replay.expected.data() + CANDIDATE_WORK_AT + index * 8U,
            candidate_work[index]);
    store_u64(replay.expected.data() + EVENT_COUNT_AT, event_count);
    for (std::size_t index = 0U; index < events.size(); ++index)
        store_digest(replay.expected.data() + EVENTS_AT + index * 32U,
            events[index]);
    store_digest(replay.expected.data() + TRACE_AT, trace);
    store_digest(replay.expected.data() + RESULT_AT, replay.result);
    replay.valid = true;
    return replay;
}

Replay reconstruct(const FileImage<CACHE_SIZE>& cache,
    const FileImage<PARENT_SIZE>& parent,
    const FileImage<PARENT_AUDIT_SIZE>& parent_audit,
    const Inputs& input, const ControlChoice& control,
    CheckerWork& work) noexcept {
    Replay replay;
    if (!input.parsed || input.rows != DIMENSION || input.columns != 315U
        || input.baseline.count != DIMENSION
        || input.factor.count != FACTOR_COMPONENTS
        || input.permutation.count != DIMENSION
        || input.inverse.size != sizeof(Quad) || input.rhs.count != DIMENSION)
        return replay;

    std::array<Quad, FACTOR_COMPONENTS> upper{};
    bool input_finite = true;
    for (std::size_t index = 0U; index < upper.size(); ++index) {
        upper[index] = static_cast<Quad>(load_double(input.factor.bytes,
            index, work));
        input_finite = input_finite && finiteq(upper[index]) != 0;
        ++work.arithmetic_checks;
    }
    std::array<std::uint64_t, DIMENSION> permutation{};
    std::array<bool, DIMENSION> seen{};
    bool permutation_valid = true;
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        permutation[index] = load_index(input.permutation.bytes, index, work);
        const bool in_range = permutation[index] < DIMENSION;
        ++work.arithmetic_checks;
        const bool unique = in_range && !seen[permutation[index]];
        ++work.arithmetic_checks;
        permutation_valid = permutation_valid && in_range && unique;
        if (in_range) seen[permutation[index]] = true;
    }
    std::array<Quad, DIMENSION> rhs{};
    std::array<Quad, DIMENSION> baseline{};
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        rhs[index] = load_quad(input.rhs.bytes, index, work);
        baseline[index] = load_quad(input.baseline.bytes, index, work);
        input_finite = input_finite && finiteq(rhs[index]) != 0
            && finiteq(baseline[index]) != 0;
        work.arithmetic_checks += 2U;
    }
    const Quad inverse = load_quad(input.inverse, 0U, work);
    input_finite = input_finite && finiteq(inverse) != 0
        && inverse > static_cast<Quad>(0.0);
    work.arithmetic_checks += 2U;
    CandidateControlAccounting control_work;
    if (control.value == Control::SmallSolve
        && !independently_check_small_solve(control_work, work))
        return replay;
    if (control.value == Control::StateDifference
        && !independently_check_state_difference(control_work, work))
        return replay;
    LinearSolve x0 = replay_factor(upper, permutation, rhs, inverse, work);
    if (control.value == Control::X0Mismatch) {
        x0.value[0U] = nextafterq(x0.value[0U], HUGE_VALQ);
        ++control_work.fixture_mutations;
        ++work.control_fixture_mutations;
    }
    std::size_t matching = 0U;
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        ++work.baseline_comparisons;
        matching += std::memcmp(&x0.value[index], &baseline[index],
            sizeof(Quad)) == 0 ? 1U : 0U;
    }
    if (!input_finite || !permutation_valid || !x0.valid
        || work.factor_terms != 10302U || work.factor_divisions != 204U)
        return replay;

    const Digest factor = root_factor(input.factor.bytes, input.factor.count,
        work);
    const Digest permutation_digest = root_permutation(permutation, work);
    const Digest inverse_digest = root_scalar(
        "nextengine.nonlocal.r63zn.inverse-scale.v1", inverse, work);
    const Digest rhs_digest = root_quad_vector(rhs, work);
    const Digest baseline_digest = root_quad_vector(baseline, work);
    const Digest x0_digest = root_quad_vector(x0.value, work);
    const std::array<Digest, 8U> bundle_fields{{cache.root, parent.root,
        parent_audit.root, factor, permutation_digest, inverse_digest,
        rhs_digest, baseline_digest}};
    const Digest bundle = root_digest_set(
        "nextengine.nonlocal.r63zn.input-bundle.v1", bundle_fields, work);
    const std::array<Digest, 3U> inputs{{cache.root, parent.root,
        parent_audit.root}};
    const Digest parent_set = root_digest_set(
        "nextengine.nonlocal.r63zn.parent-set.v1", inputs, work);
    CandidateSemantic semantic{{bundle, factor, permutation_digest,
        inverse_digest, rhs_digest, baseline_digest, x0_digest, {},
        {}, {}, {}, {}}};
    const std::uint64_t preproduct_canonical_bytes = 3U
        * (std::strlen("nextengine.nonlocal.r63zn.binary128-vector.v1")
            + 1667U)
        + std::strlen("nextengine.nonlocal.r63zn.binary64-factor.v1")
            + 83284U
        + std::strlen("nextengine.nonlocal.r63zn.permutation.v1") + 851U
        + std::strlen("nextengine.nonlocal.r63zn.inverse-scale.v1") + 34U;

    ++work.candidate_route_decisions;
    if (matching != DIMENSION) {
        constexpr std::size_t event_count = 3U;
        const auto candidate = expected_candidate_work(control,
            control_work, 1U, 10302U, 204U, 613U, DIMENSION,
            0U, 0U, 0U, 0U, 0U, 0U, 0U, 0U, 6U,
            preproduct_canonical_bytes, event_count, 8U, false);
        const Digest candidate_work = root_candidate_work(candidate, work);
        std::array<Digest, EVENT_SLOTS> events{};
        const Digest zero{};
        events[0U] = root_event(0U, parent_set, zero, work);
        events[1U] = root_event(1U, bundle, factor, work);
        events[2U] = root_event(2U, x0_digest, baseline_digest, work);
        constexpr std::array<std::uint8_t, 3U> flags{{1U, 1U, 0U}};
        return encode_replay(4U, flags, cache, parent, parent_audit,
            semantic, candidate, candidate_work, events, event_count, work);
    }
    if (control.value == Control::X0Mismatch) return replay;

    std::array<Quad, DIMENSION> hx0{};
    bool hx0_finite = true;
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        hx0[index] = load_parent_quad(parent.bytes,
            ROLE_TWO_VALUES + index * 16U, work);
        hx0_finite = hx0_finite && finiteq(hx0[index]) != 0;
        ++work.arithmetic_checks;
    }
    if (!hx0_finite) return replay;
    const Digest hx0_digest = root_quad_vector(hx0, work);
    const Digest parent_hx0_digest = root_parent_quad_vector(hx0, work);
    ++work.role2_value_root_comparisons;
    if (!digest_is(parent_hx0_digest, ROLE_TWO_VALUE_ID)) return replay;
    semantic[7U] = hx0_digest;
    const std::uint64_t product_canonical_bytes = preproduct_canonical_bytes
        + std::strlen("nextengine.nonlocal.r63zn.binary128-vector.v1") + 1667U
        + std::strlen("nextengine.nonlocal.r63zm.binary128-vector.v1") + 1667U;

    std::array<Quad, DIMENSION> residual{};
    std::array<Quad, DIMENSION> residual_bound{};
    bool residual_exact = true;
    bool residual_normal = true;
    std::uint64_t residual_updates = 0U;
    const Quad one = static_cast<Quad>(1.0);
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        std::array<Quad, 2U> left{{rhs[index], -hx0[index]}};
        const std::array<Quad, 2U> right{{one, one}};
        if (control.value == Control::PrefixUnderflow && index == 0U) {
            left[0U] = ldexpq(one, -16494);
            left[1U] = static_cast<Quad>(0.0);
            control_work.fixture_mutations += 2U;
            work.control_fixture_mutations += 2U;
        }
        if (control.value == Control::PrefixNonfinite && index == 0U) {
            left[0U] = HUGE_VALQ;
            ++control_work.fixture_mutations;
            ++work.control_fixture_mutations;
        }
        const EnclosedDot component = replay_dot(left.data(), right.data(),
            left.size(), work);
        residual[index] = component.value;
        residual_bound[index] = component.bound;
        residual_exact = residual_exact && component.exact;
        residual_normal = residual_normal && component.no_underflow;
        ++residual_updates;
        work.arithmetic_checks += 2U;
        if (!component.exact || !component.no_underflow) break;
    }
    ++work.candidate_route_decisions;
    if (!residual_exact || !residual_normal) {
        constexpr std::size_t event_count = 4U;
        const auto candidate = expected_candidate_work(control,
            control_work, 1U, 10302U, 204U, 613U, DIMENSION,
            residual_updates, residual_updates, 2U * residual_updates,
            0U, 0U, work.two_products, work.two_sums, 0U, 8U,
            product_canonical_bytes, event_count, 9U, true);
        const Digest candidate_work = root_candidate_work(candidate, work);
        std::array<Digest, EVENT_SLOTS> events{};
        const Digest zero{};
        events[0U] = root_event(0U, parent_set, zero, work);
        events[1U] = root_event(1U, bundle, factor, work);
        events[2U] = root_event(2U, x0_digest, baseline_digest, work);
        events[3U] = root_event(3U, hx0_digest, parent.root, work);
        constexpr std::array<std::uint8_t, 3U> flags{{1U, 1U, 0U}};
        return encode_replay(5U, flags, cache, parent, parent_audit,
            semantic, candidate, candidate_work, events, event_count, work);
    }
    if (control.value == Control::PrefixUnderflow
        || control.value == Control::PrefixNonfinite
        || residual_updates != DIMENSION)
        return replay;

    const LinearSolve z0 = replay_factor(upper, permutation, residual,
        inverse, work);
    if (!z0.valid || work.factor_terms != 20604U
        || work.factor_divisions != 408U)
        return replay;
    const Digest residual_digest = root_quad_vector(residual, work);
    const Digest residual_bound_digest = root_quad_vector(residual_bound,
        work);
    const Digest z0_digest = root_quad_vector(z0.value, work);
    const Quad* rho_values = z0.value.data();
    if (control.value == Control::RhoNonpositive) {
        for (std::size_t index = 0U; index < DIMENSION; ++index) {
            x0.value[index] = -z0.value[index];
            ++control_work.fixture_mutations;
            ++work.control_fixture_mutations;
        }
        rho_values = x0.value.data();
    }
    const EnclosedDot rho = replay_dot(residual.data(), rho_values,
        DIMENSION, work);
    const bool positive = rho.exact && rho.no_underflow
        && rho.value - rho.bound > static_cast<Quad>(0.0);
    work.arithmetic_checks += 3U;
    if (!rho.exact || !rho.no_underflow || work.dot_calls != 103U
        || work.dot_terms != 306U || work.two_products != 306U
        || work.two_sums != 203U)
        return replay;
    const Digest rho_digest = root_rho(rho, work);
    semantic[8U] = residual_digest;
    semantic[9U] = residual_bound_digest;
    semantic[10U] = z0_digest;
    semantic[11U] = rho_digest;
    const std::uint64_t full_canonical_bytes = product_canonical_bytes
        + 3U * (std::strlen(
            "nextengine.nonlocal.r63zn.binary128-vector.v1") + 1667U)
        + std::strlen("nextengine.nonlocal.r63zn.rho0.v1") + 118U;

    ++work.candidate_route_decisions;
    const std::size_t event_count = positive ? EVENT_SLOTS : 6U;
    const auto candidate = expected_candidate_work(control,
        control_work, 2U, 20604U, 408U, 1226U, DIMENSION,
        DIMENSION, DIMENSION, 2U * DIMENSION, 1U, DIMENSION,
        work.two_products, work.two_sums, 1U, 12U, full_canonical_bytes,
        event_count, 10U, true);
    const Digest candidate_work = root_candidate_work(candidate, work);
    std::array<Digest, EVENT_SLOTS> events{};
    const Digest zero{};
    events[0U] = root_event(0U, parent_set, zero, work);
    events[1U] = root_event(1U, bundle, factor, work);
    events[2U] = root_event(2U, x0_digest, baseline_digest, work);
    events[3U] = root_event(3U, hx0_digest, parent.root, work);
    events[4U] = root_event(4U, residual_digest, residual_bound_digest, work);
    events[5U] = root_event(5U, z0_digest, factor, work);
    if (!positive) {
        constexpr std::array<std::uint8_t, 3U> flags{{1U, 1U, 0U}};
        return encode_replay(6U, flags, cache, parent, parent_audit,
            semantic, candidate, candidate_work, events, event_count, work);
    }
    if (control.value == Control::RhoNonpositive) return replay;
    events[6U] = root_event(6U, rho_digest, candidate_work, work);
    constexpr std::array<std::uint8_t, 3U> flags{{1U, 1U, 1U}};
    return encode_replay(7U, flags, cache, parent, parent_audit,
        semantic, candidate, candidate_work, events, event_count, work);
}

bool equal_region(const std::array<std::uint8_t, RECEIPT_SIZE>& actual,
    const std::array<std::uint8_t, RECEIPT_SIZE>& expected,
    std::size_t offset, std::size_t size, std::uint64_t& counter) noexcept {
    bool equal = true;
    for (std::size_t index = 0U; index < size; ++index) {
        equal = equal && actual[offset + index] == expected[offset + index];
        ++counter;
    }
    return equal;
}

enum class Route : std::uint32_t {
    Accepted = 0U,
    CacheReadRejectedVerified = 1U,
    ParentArtifactRejectedVerified = 2U,
    ParentAuditRejectedVerified = 3U,
    CacheSemanticRejectedVerified = 4U,
    X0CorrespondenceRejectedVerified = 5U,
    PrefixArithmeticRejectedVerified = 6U,
    RhoNonpositiveRejectedVerified = 7U,
    CheckerRead = 8U,
    MalformedReceipt = 9U,
    Arithmetic = 10U,
    Semantic = 11U,
    Work = 12U,
    Events = 13U,
    Seal = 14U,
};

Digest root_checker(Route route, bool semantic, bool work_exact,
    bool events, bool seals, const std::array<std::uint64_t, 4U>& sizes,
    const std::array<Digest, 4U>& roots, const Digest& observed_result,
    const Digest& replay_result, const CheckerWork& checker) noexcept {
    constexpr const char* domain =
        "nextengine.nonlocal.r63zn.independent-checker.v1";
    r63zm::Sha256 hash;
    field_text(hash, 1U, domain);
    field_u64(hash, 2U, static_cast<std::uint32_t>(route));
    field_u64(hash, 3U, semantic ? 1U : 0U);
    field_u64(hash, 4U, work_exact ? 1U : 0U);
    field_u64(hash, 5U, events ? 1U : 0U);
    field_u64(hash, 6U, seals ? 1U : 0U);
    const auto fields = checker.fields();
    field_header(hash, 7U, fields.size() * 8U);
    for (std::uint64_t value : fields) append_big_u64(hash, value);
    field_header(hash, 8U, sizes.size() * 8U);
    for (std::uint64_t value : sizes) append_big_u64(hash, value);
    field_header(hash, 9U, roots.size() * 32U);
    for (const Digest& root : roots) hash.update(root);
    field_digest(hash, 10U, observed_result);
    field_digest(hash, 11U, replay_result);
    field_u64(hash, 12U, CHECKER_AUDIT_SIZE);
    return hash.finish();
}

bool write_exact(const char* path,
    const std::array<std::uint8_t, CHECKER_AUDIT_SIZE>& bytes) noexcept {
    const int descriptor = open(path, O_WRONLY | O_CREAT | O_TRUNC, 0600);
    if (descriptor < 0) return false;
    const ssize_t count = write(descriptor, bytes.data(), bytes.size());
    const int closed = close(descriptor);
    return count == static_cast<ssize_t>(bytes.size()) && closed == 0;
}

} // namespace

int main(int argc, char** argv) {
    if (argc != 6 && argc != 7) return 64;
    CheckerWork checker;
    ++checker.rounding_mode_set_calls;
    if (std::fesetround(FE_TONEAREST) != 0) return 66;
    ++checker.rounding_mode_checks;
    if (std::fegetround() != FE_TONEAREST) return 66;
    FileImage<CACHE_SIZE> cache;
    FileImage<PARENT_SIZE> parent;
    FileImage<PARENT_AUDIT_SIZE> parent_audit;
    FileImage<RECEIPT_SIZE> receipt;
    load_exact(argv[1], cache, checker);
    load_exact(argv[2], parent, checker);
    load_exact(argv[3], parent_audit, checker);
    load_exact(argv[4], receipt, checker);

    ++checker.cache_identity_checks;
    const bool cache_identity = cache.exact && digest_is(cache.root, CACHE_ID);
    const bool parent_identity = parent.exact
        && check_parent_artifact(parent, checker);
    const bool parent_audit_identity = parent_audit.exact
        && check_parent_audit(parent_audit, checker);
    std::uint32_t expected_candidate_route = 7U;
    std::uint64_t candidate_read_calls = 3U;
    std::uint64_t expected_artifact_checks = 12U;
    std::uint64_t expected_audit_checks = 8U;
    std::uint64_t expected_route_decisions = 10U;
    auto candidate_reject = [&](bool condition) noexcept {
        ++checker.candidate_route_decisions;
        return condition;
    };
    if (candidate_reject(!cache.exact)) {
        expected_candidate_route = 0U;
        candidate_read_calls = 1U;
        expected_artifact_checks = 0U;
        expected_audit_checks = 0U;
        expected_route_decisions = 1U;
    } else if (candidate_reject(!parent.exact)) {
        expected_candidate_route = 1U;
        candidate_read_calls = 2U;
        expected_artifact_checks = 0U;
        expected_audit_checks = 0U;
        expected_route_decisions = 2U;
    } else if (candidate_reject(!parent_identity)) {
        expected_candidate_route = 1U;
        candidate_read_calls = 2U;
        expected_audit_checks = 0U;
        expected_route_decisions = 3U;
    } else if (candidate_reject(!parent_audit.exact)) {
        expected_candidate_route = 2U;
        expected_route_decisions = 4U;
        expected_audit_checks = 0U;
    } else if (candidate_reject(!parent_audit_identity)) {
        expected_candidate_route = 2U;
        expected_route_decisions = 5U;
    } else if (candidate_reject(!cache_identity)) {
        expected_candidate_route = 3U;
        expected_route_decisions = 6U;
    }

    constexpr std::array<std::uint8_t, 8U> receipt_magic{{
        'N','E','R','6','3','Z','N','1'}};
    bool malformed = !receipt.exact;
    if (receipt.exact) {
        ++checker.receipt_header_checks;
        malformed = std::memcmp(receipt.bytes.data(), receipt_magic.data(),
            receipt_magic.size()) != 0;
        ++checker.receipt_header_checks;
        malformed = malformed || big_u32(receipt.bytes.data() + VERSION_AT)
            != 3U;
        ++checker.receipt_header_checks;
        malformed = malformed || big_u64(receipt.bytes.data() + TOTAL_AT)
            != RECEIPT_SIZE;
        ++checker.receipt_header_checks;
        malformed = malformed || big_u32(receipt.bytes.data() + ROUTE_AT) > 7U;
        ++checker.receipt_header_checks;
        malformed = malformed || receipt.bytes[FLAGS_AT] > 1U
            || receipt.bytes[FLAGS_AT + 1U] > 1U
            || receipt.bytes[FLAGS_AT + 2U] > 1U;
        ++checker.receipt_header_checks;
        std::uint8_t padding = 0U;
        for (std::size_t index = FLAGS_AT + 3U; index < CACHE_SIZE_AT; ++index) {
            padding |= receipt.bytes[index];
            ++checker.receipt_padding_bytes_scanned;
        }
        malformed = malformed || padding != 0U;
    }

    Inputs decoded;
    Replay replay;
    ControlChoice control;
    if (expected_candidate_route <= 3U) {
        replay = reconstruct_early_rejection(expected_candidate_route,
            cache, parent, parent_audit, candidate_read_calls,
            expected_artifact_checks, expected_audit_checks,
            0U, 0U, 0U, expected_route_decisions, checker);
    } else {
        decoded = decode_cache(cache.bytes, checker);
        ++checker.candidate_route_decisions;
        if (!decoded.parsed || decoded.rows != DIMENSION
            || decoded.columns != 315U
            || decoded.baseline.count != DIMENSION
            || decoded.factor.count != FACTOR_COMPONENTS
            || decoded.permutation.count != DIMENSION
            || decoded.inverse.size != sizeof(Quad)
            || decoded.rhs.count != DIMENSION) {
            expected_candidate_route = 3U;
            expected_route_decisions = 7U;
            replay = reconstruct_early_rejection(expected_candidate_route,
                cache, parent, parent_audit, candidate_read_calls,
                expected_artifact_checks, expected_audit_checks,
                168U, CACHE_SIZE, 88U, expected_route_decisions, checker);
        } else {
            control = decode_control(argc, argc == 7 ? argv[6] : nullptr);
            if (control.value == Control::Invalid) return 64;
            checker.control_selector_checks += control.selector_checks;
            checker.control_selector_bytes += control.selector_bytes;
            replay = reconstruct(cache, parent, parent_audit, decoded,
                control, checker);
            if (replay.valid)
                expected_candidate_route = replay.candidate_route;
        }
    }

    bool semantic_exact = false;
    bool candidate_work_exact = false;
    bool events_exact = false;
    bool seals_exact = false;
    if (receipt.exact && replay.valid) {
        semantic_exact = equal_region(receipt.bytes, replay.expected, 0U,
            CANDIDATE_WORK_AT, checker.semantic_bytes_compared);
        candidate_work_exact = equal_region(receipt.bytes, replay.expected,
            CANDIDATE_WORK_AT, CANDIDATE_WORK_FIELDS * 8U,
            checker.work_fields_compared);
        checker.work_fields_compared /= 8U;
        events_exact = equal_region(receipt.bytes, replay.expected,
            EVENT_COUNT_AT, TRACE_AT - EVENT_COUNT_AT,
            checker.event_fields_compared);
        checker.event_fields_compared = 1U + EVENT_SLOTS;
        seals_exact = equal_region(receipt.bytes, replay.expected, TRACE_AT,
            RECEIPT_SIZE - TRACE_AT, checker.seal_fields_compared);
        checker.seal_fields_compared = 2U;
    }

    Route route = Route::Accepted;
    auto checker_reject = [&](bool condition) noexcept {
        ++checker.checker_route_decisions;
        return condition;
    };
    if (checker_reject(!receipt.exact)) route = Route::CheckerRead;
    else if (checker_reject(malformed)) route = Route::MalformedReceipt;
    else if (checker_reject(!replay.valid)) route = Route::Arithmetic;
    else if (checker_reject(!semantic_exact)) route = Route::Semantic;
    else if (checker_reject(!candidate_work_exact)) route = Route::Work;
    else if (checker_reject(!events_exact)) route = Route::Events;
    else if (checker_reject(!seals_exact)) route = Route::Seal;
    else {
        ++checker.checker_route_decisions;
        switch (expected_candidate_route) {
        case 0U: route = Route::CacheReadRejectedVerified; break;
        case 1U: route = Route::ParentArtifactRejectedVerified; break;
        case 2U: route = Route::ParentAuditRejectedVerified; break;
        case 3U: route = Route::CacheSemanticRejectedVerified; break;
        case 4U: route = Route::X0CorrespondenceRejectedVerified; break;
        case 5U: route = Route::PrefixArithmeticRejectedVerified; break;
        case 6U: route = Route::RhoNonpositiveRejectedVerified; break;
        case 7U: route = Route::Accepted; break;
        default: route = Route::Arithmetic; break;
        }
    }

    Digest observed_result{};
    if (receipt.exact)
        std::memcpy(observed_result.data(), receipt.bytes.data() + RESULT_AT,
            observed_result.size());
    const std::array<std::uint64_t, 4U> sizes{{cache.observed_size,
        parent.observed_size, parent_audit.observed_size,
        receipt.observed_size}};
    const std::array<Digest, 4U> roots{{cache.root, parent.root,
        parent_audit.root, receipt.root}};

    constexpr const char* checker_domain =
        "nextengine.nonlocal.r63zn.independent-checker.v1";
    checker.audit_root_bytes = std::strlen(checker_domain) + 9U
        + 5U * 17U + 9U + CHECKER_WORK_FIELDS * 8U
        + 9U + 4U * 8U + 9U + 4U * 32U + 2U * 41U + 17U;
    ++checker.hash_calls;
    checker.hash_bytes += checker.audit_root_bytes;
    checker.audit_open_calls = 1U;
    checker.audit_write_calls = 1U;
    checker.audit_close_calls = 1U;
    checker.audit_write_bytes = CHECKER_AUDIT_SIZE;
    checker.audit_fields_written = 20U + CHECKER_WORK_FIELDS;
    checker.audit_zero_fill_bytes = CHECKER_AUDIT_SIZE;
    const Digest checker_result = root_checker(route, semantic_exact,
        candidate_work_exact, events_exact, seals_exact, sizes, roots,
        observed_result, replay.result, checker);

    std::array<std::uint8_t, CHECKER_AUDIT_SIZE> output{};
    std::uint64_t serialized_fields = 0U;
    constexpr std::array<std::uint8_t, 8U> audit_magic{{
        'N','E','R','6','3','Z','Q','1'}};
    std::memcpy(output.data(), audit_magic.data(), audit_magic.size());
    ++serialized_fields;
    store_u32(output.data() + CHECKER_VERSION_AT, 2U);
    ++serialized_fields;
    store_u64(output.data() + CHECKER_TOTAL_AT, CHECKER_AUDIT_SIZE);
    ++serialized_fields;
    store_u32(output.data() + CHECKER_ROUTE_AT,
        static_cast<std::uint32_t>(route));
    ++serialized_fields;
    output[CHECKER_FLAGS_AT] = semantic_exact ? 1U : 0U;
    output[CHECKER_FLAGS_AT + 1U] = candidate_work_exact ? 1U : 0U;
    output[CHECKER_FLAGS_AT + 2U] = events_exact ? 1U : 0U;
    output[CHECKER_FLAGS_AT + 3U] = seals_exact ? 1U : 0U;
    serialized_fields += 4U;
    for (std::size_t index = 0U; index < sizes.size(); ++index) {
        store_u64(output.data() + CHECKER_SIZES_AT + index * 8U,
            sizes[index]);
        ++serialized_fields;
    }
    for (std::size_t index = 0U; index < roots.size(); ++index) {
        store_digest(output.data() + CHECKER_ROOTS_AT + index * 32U,
            roots[index]);
        ++serialized_fields;
    }
    store_u64(output.data() + CHECKER_WORK_COUNT_AT, CHECKER_WORK_FIELDS);
    ++serialized_fields;
    const auto fields = checker.fields();
    for (std::size_t index = 0U; index < fields.size(); ++index) {
        store_u64(output.data() + CHECKER_WORK_AT + index * 8U,
            fields[index]);
        ++serialized_fields;
    }
    store_digest(output.data() + CHECKER_OBSERVED_RESULT_AT, observed_result);
    ++serialized_fields;
    store_digest(output.data() + CHECKER_REPLAY_RESULT_AT, replay.result);
    ++serialized_fields;
    store_digest(output.data() + CHECKER_RESULT_AT, checker_result);
    ++serialized_fields;
    if (serialized_fields != checker.audit_fields_written) return 65;
    if (!write_exact(argv[5], output)) return 65;
    return static_cast<std::uint32_t>(route)
        <= static_cast<std::uint32_t>(Route::RhoNonpositiveRejectedVerified)
        ? 0 : 1;
}
