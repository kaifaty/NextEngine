#include <quadmath.h>

#include "../r63zm/fixed_sha256.hpp"

#include <array>
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
constexpr std::size_t RECEIPT_SIZE = 1216U;
constexpr std::size_t DIMENSION = 102U;
constexpr std::size_t FACTOR_COMPONENTS = DIMENSION * DIMENSION;
constexpr std::size_t ROLE_TWO_VALUES = 5436U;
enum CandidateWorkSlot : std::size_t {
    InputReadCalls,
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
    StructuralRootCalls,
    StructuralRootBytes,
    EventRootCalls,
    EventRootBytes,
    TraceRootCalls,
    TraceRootBytes,
    ResultRootCalls,
    ResultRootBytes,
    ReceiptFieldsWritten,
    ReceiptWriteCalls,
    ReceiptWriteBytes,
    RouteDecisions,
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
constexpr std::size_t EVENT_COUNT_AT = 920U;
constexpr std::size_t EVENTS_AT = 928U;
constexpr std::size_t TRACE_AT = 1152U;
constexpr std::size_t RESULT_AT = 1184U;

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

constexpr std::size_t CHECKER_WORK_FIELDS = 32U;
constexpr std::size_t CHECKER_AUDIT_SIZE = 548U;
constexpr std::size_t CHECKER_SIZES_AT = 28U;
constexpr std::size_t CHECKER_ROOTS_AT = 60U;
constexpr std::size_t CHECKER_WORK_COUNT_AT = 188U;
constexpr std::size_t CHECKER_WORK_AT = 196U;
constexpr std::size_t CHECKER_OBSERVED_RESULT_AT = 452U;
constexpr std::size_t CHECKER_REPLAY_RESULT_AT = 484U;
constexpr std::size_t CHECKER_RESULT_AT = 516U;
static_assert(CHECKER_SIZES_AT + 4U * 8U == CHECKER_ROOTS_AT);
static_assert(CHECKER_ROOTS_AT + 4U * 32U == CHECKER_WORK_COUNT_AT);
static_assert(CHECKER_WORK_COUNT_AT + 8U == CHECKER_WORK_AT);
static_assert(CHECKER_WORK_AT + CHECKER_WORK_FIELDS * 8U
    == CHECKER_OBSERVED_RESULT_AT);
static_assert(CHECKER_OBSERVED_RESULT_AT + 32U == CHECKER_REPLAY_RESULT_AT);
static_assert(CHECKER_REPLAY_RESULT_AT + 32U == CHECKER_RESULT_AT);
static_assert(CHECKER_RESULT_AT + 32U == CHECKER_AUDIT_SIZE);

struct CheckerWork final {
    std::uint64_t logical_file_reads = 0U;
    std::uint64_t file_open_calls = 0U;
    std::uint64_t file_stat_calls = 0U;
    std::uint64_t file_close_calls = 0U;
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
    std::uint64_t receipt_header_checks = 0U;
    std::uint64_t semantic_bytes_compared = 0U;
    std::uint64_t work_fields_compared = 0U;
    std::uint64_t event_fields_compared = 0U;
    std::uint64_t seal_fields_compared = 0U;
    std::uint64_t route_checks = 0U;
    std::uint64_t audit_root_bytes = 0U;
    std::uint64_t audit_write_calls = 0U;
    std::uint64_t audit_write_bytes = 0U;
    std::uint64_t package_allocations = 0U;
    std::uint64_t audit_zero_fill_bytes = 0U;

    std::array<std::uint64_t, CHECKER_WORK_FIELDS> fields() const noexcept {
        return {{logical_file_reads, file_open_calls, file_stat_calls,
            file_close_calls, file_bytes, hash_calls, hash_bytes,
            parent_checks, cursor_takes, cursor_bytes, cursor_predicates,
            scalar_loads, factor_solves, factor_terms, factor_divisions,
            baseline_comparisons, dot_calls, dot_terms, two_products,
            two_sums, arithmetic_checks, receipt_header_checks,
            semantic_bytes_compared, work_fields_compared,
            event_fields_compared, seal_fields_compared, route_checks,
            audit_root_bytes, audit_write_calls, audit_write_bytes,
            package_allocations, audit_zero_fill_bytes}};
    }
};

template <std::size_t Size>
struct FileImage final {
    std::array<std::uint8_t, Size> bytes{};
    std::uint64_t observed_size = 0U;
    Digest root{};
    bool exact = false;
};

template <std::size_t Size>
void load_exact(const char* path, FileImage<Size>& image,
    CheckerWork& work) noexcept {
    ++work.logical_file_reads;
    const int descriptor = open(path, O_RDONLY);
    ++work.file_open_calls;
    if (descriptor < 0) return;
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
            const ssize_t count = read(descriptor, image.bytes.data() + offset,
                Size - offset);
            if (count <= 0) break;
            offset += static_cast<std::size_t>(count);
        }
        work.file_bytes += offset;
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

    bool complete() const noexcept {
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
    record_hash(work, std::strlen(domain) + 419U);
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
    CheckerWork& work) noexcept {
    constexpr const char* domain = "nextengine.nonlocal.r63zn.trace.v1";
    r63zm::Sha256 hash;
    field_text(hash, 1U, domain);
    field_u64(hash, 2U, route);
    field_u64(hash, 3U, events.size());
    field_header(hash, 4U, events.size() * 32U);
    for (const Digest& event : events) hash.update(event);
    record_hash(work, std::strlen(domain) + 276U);
    return hash.finish();
}

Digest root_candidate_result(std::uint64_t route,
    const std::array<Digest, 3U>& inputs, const Digest& trace,
    const Digest& candidate_work, CheckerWork& work) noexcept {
    constexpr const char* domain = "nextengine.nonlocal.r63zn.result.v1";
    r63zm::Sha256 hash;
    field_text(hash, 1U, domain);
    field_u64(hash, 2U, 2U);
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
    std::array<std::uint8_t, RECEIPT_SIZE> expected{};
    Digest result{};
};

Replay reconstruct(const FileImage<CACHE_SIZE>& cache,
    const FileImage<PARENT_SIZE>& parent,
    const FileImage<PARENT_AUDIT_SIZE>& parent_audit,
    const Inputs& input, CheckerWork& work) noexcept {
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
        if (permutation[index] < DIMENSION) seen[permutation[index]] = true;
    }
    std::array<Quad, DIMENSION> rhs{};
    std::array<Quad, DIMENSION> baseline{};
    std::array<Quad, DIMENSION> hx0{};
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        rhs[index] = load_quad(input.rhs.bytes, index, work);
        baseline[index] = load_quad(input.baseline.bytes, index, work);
        hx0[index] = load_parent_quad(parent.bytes,
            ROLE_TWO_VALUES + index * 16U, work);
        input_finite = input_finite && finiteq(rhs[index]) != 0
            && finiteq(baseline[index]) != 0 && finiteq(hx0[index]) != 0;
        work.arithmetic_checks += 3U;
    }
    const Quad inverse = load_quad(input.inverse, 0U, work);
    input_finite = input_finite && finiteq(inverse) != 0;
    work.arithmetic_checks += 2U;
    input_finite = input_finite && inverse > static_cast<Quad>(0.0);
    const LinearSolve x0 = replay_factor(upper, permutation, rhs, inverse,
        work);
    std::size_t matching = 0U;
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        ++work.baseline_comparisons;
        matching += std::memcmp(&x0.value[index], &baseline[index],
            sizeof(Quad)) == 0 ? 1U : 0U;
    }

    std::array<Quad, DIMENSION> residual{};
    std::array<Quad, DIMENSION> residual_bound{};
    bool residual_exact = true;
    bool residual_normal = true;
    const Quad one = static_cast<Quad>(1.0);
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        const std::array<Quad, 2U> left{{rhs[index], -hx0[index]}};
        const std::array<Quad, 2U> right{{one, one}};
        const EnclosedDot component = replay_dot(left.data(), right.data(),
            left.size(), work);
        residual[index] = component.value;
        residual_bound[index] = component.bound;
        residual_exact = residual_exact && component.exact;
        residual_normal = residual_normal && component.no_underflow;
        work.arithmetic_checks += 2U;
    }
    const LinearSolve z0 = replay_factor(upper, permutation, residual, inverse,
        work);
    const EnclosedDot rho = replay_dot(residual.data(), z0.value.data(),
        DIMENSION, work);
    const bool positive = rho.exact && rho.no_underflow
        && rho.value - rho.bound > static_cast<Quad>(0.0);
    work.arithmetic_checks += 3U;
    if (!input_finite || !permutation_valid || !x0.valid
        || matching != DIMENSION
        || !residual_exact || !residual_normal || !z0.valid || !positive
        || work.factor_terms != 20604U || work.factor_divisions != 408U
        || work.dot_calls != 103U || work.dot_terms != 306U
        || work.two_products != 306U || work.two_sums != 203U)
        return replay;

    const Digest factor = root_factor(input.factor.bytes, input.factor.count,
        work);
    const Digest permutation_digest = root_permutation(permutation, work);
    const Digest inverse_digest = root_scalar(
        "nextengine.nonlocal.r63zn.inverse-scale.v1", inverse, work);
    const Digest rhs_digest = root_quad_vector(rhs, work);
    const Digest baseline_digest = root_quad_vector(baseline, work);
    const Digest x0_digest = root_quad_vector(x0.value, work);
    const Digest hx0_digest = root_quad_vector(hx0, work);
    const Digest residual_digest = root_quad_vector(residual, work);
    const Digest residual_bound_digest = root_quad_vector(residual_bound,
        work);
    const Digest z0_digest = root_quad_vector(z0.value, work);
    const Digest rho_digest = root_rho(rho, work);
    const std::array<Digest, 9U> bundle_fields{{cache.root, parent.root,
        parent_audit.root, factor, permutation_digest, inverse_digest,
        rhs_digest, baseline_digest, hx0_digest}};
    const Digest bundle = root_digest_set(
        "nextengine.nonlocal.r63zn.input-bundle.v1", bundle_fields, work);
    const std::array<Digest, 3U> inputs{{
        cache.root, parent.root, parent_audit.root}};
    const Digest parent_set = root_digest_set(
        "nextengine.nonlocal.r63zn.parent-set.v1", inputs, work);

    constexpr std::uint64_t input_bytes =
        CACHE_SIZE + PARENT_SIZE + PARENT_AUDIT_SIZE;
    const std::uint64_t canonical_bytes =
        7U * (std::strlen(
            "nextengine.nonlocal.r63zn.binary128-vector.v1") + 1667U)
        + std::strlen("nextengine.nonlocal.r63zn.binary64-factor.v1")
            + 83284U
        + std::strlen("nextengine.nonlocal.r63zn.permutation.v1") + 851U
        + std::strlen("nextengine.nonlocal.r63zn.inverse-scale.v1") + 34U
        + std::strlen("nextengine.nonlocal.r63zn.rho0.v1") + 118U;
    const std::uint64_t structural_bytes =
        std::strlen("nextengine.nonlocal.r63zn.input-bundle.v1") + 323U
        + std::strlen("nextengine.nonlocal.r63zn.parent-set.v1") + 131U
        + std::strlen("nextengine.nonlocal.r63zn.candidate-work.v1") + 419U;
    const std::uint64_t event_bytes = EVENT_SLOTS
        * (std::strlen("nextengine.nonlocal.r63zn.event.v1") + 108U);
    const std::uint64_t trace_bytes =
        std::strlen("nextengine.nonlocal.r63zn.trace.v1") + 276U;
    const std::uint64_t result_bytes =
        std::strlen("nextengine.nonlocal.r63zn.result.v1") + 247U;
    const std::array<std::uint64_t, CANDIDATE_WORK_FIELDS> candidate_work{{
        3U, input_bytes, 3U, input_bytes, 168U, CACHE_SIZE, 87U,
        12U, 8U, FACTOR_COMPONENTS, DIMENSION, 3U * DIMENSION + 1U,
        FACTOR_COMPONENTS, DIMENSION, DIMENSION, 3U * DIMENSION + 1U, 1U,
        2U, 20604U, 408U, 1226U, DIMENSION,
        DIMENSION, DIMENSION, 2U * DIMENSION, 1U, DIMENSION,
        3U * DIMENSION, 2U * DIMENSION - 1U,
        DIMENSION + 1U, DIMENSION + 1U, 1U,
        11U, canonical_bytes, 3U, structural_bytes,
        EVENT_SLOTS, event_bytes, 1U, trace_bytes,
        1U, result_bytes, 83U, 1U, RECEIPT_SIZE, 1U,
        RECEIPT_SIZE, 0U}};
    const Digest candidate_work_digest = root_candidate_work(candidate_work,
        work);
    const std::array<Digest, EVENT_SLOTS> events{{
        root_event(0U, parent_set, bundle, work),
        root_event(1U, bundle, factor, work),
        root_event(2U, x0_digest, baseline_digest, work),
        root_event(3U, hx0_digest, parent.root, work),
        root_event(4U, residual_digest, residual_bound_digest, work),
        root_event(5U, z0_digest, factor, work),
        root_event(6U, rho_digest, candidate_work_digest, work)}};
    constexpr std::uint64_t route = 7U;
    const Digest trace = root_trace(route, events, work);
    replay.result = root_candidate_result(route, inputs, trace,
        candidate_work_digest, work);

    constexpr std::array<std::uint8_t, 8U> magic{{
        'N','E','R','6','3','Z','N','1'}};
    std::memcpy(replay.expected.data(), magic.data(), magic.size());
    store_u32(replay.expected.data() + VERSION_AT, 2U);
    store_u64(replay.expected.data() + TOTAL_AT, RECEIPT_SIZE);
    store_u32(replay.expected.data() + ROUTE_AT, route);
    replay.expected[FLAGS_AT] = 1U;
    replay.expected[FLAGS_AT + 1U] = 1U;
    replay.expected[FLAGS_AT + 2U] = 1U;
    store_u64(replay.expected.data() + CACHE_SIZE_AT, CACHE_SIZE);
    store_digest(replay.expected.data() + CACHE_ROOT_AT, cache.root);
    store_u64(replay.expected.data() + PARENT_SIZE_AT, PARENT_SIZE);
    store_digest(replay.expected.data() + PARENT_ROOT_AT, parent.root);
    store_u64(replay.expected.data() + PARENT_AUDIT_SIZE_AT,
        PARENT_AUDIT_SIZE);
    store_digest(replay.expected.data() + PARENT_AUDIT_ROOT_AT,
        parent_audit.root);
    store_digest(replay.expected.data() + BUNDLE_ROOT_AT, bundle);
    store_digest(replay.expected.data() + FACTOR_ROOT_AT, factor);
    store_digest(replay.expected.data() + PERMUTATION_ROOT_AT,
        permutation_digest);
    store_digest(replay.expected.data() + INVERSE_ROOT_AT, inverse_digest);
    store_digest(replay.expected.data() + RHS_ROOT_AT, rhs_digest);
    store_digest(replay.expected.data() + BASELINE_ROOT_AT, baseline_digest);
    store_digest(replay.expected.data() + X0_ROOT_AT, x0_digest);
    store_digest(replay.expected.data() + HX0_ROOT_AT, hx0_digest);
    store_digest(replay.expected.data() + RESIDUAL_ROOT_AT, residual_digest);
    store_digest(replay.expected.data() + RESIDUAL_BOUND_ROOT_AT,
        residual_bound_digest);
    store_digest(replay.expected.data() + Z0_ROOT_AT, z0_digest);
    store_digest(replay.expected.data() + RHO_ROOT_AT, rho_digest);
    for (std::size_t index = 0U; index < candidate_work.size(); ++index)
        store_u64(replay.expected.data() + CANDIDATE_WORK_AT + index * 8U,
            candidate_work[index]);
    store_u64(replay.expected.data() + EVENT_COUNT_AT, EVENT_SLOTS);
    for (std::size_t index = 0U; index < events.size(); ++index)
        store_digest(replay.expected.data() + EVENTS_AT + index * 32U,
            events[index]);
    store_digest(replay.expected.data() + TRACE_AT, trace);
    store_digest(replay.expected.data() + RESULT_AT, replay.result);
    replay.valid = true;
    return replay;
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
    Read = 1U,
    Input = 2U,
    MalformedReceipt = 3U,
    Arithmetic = 4U,
    Semantic = 5U,
    Work = 6U,
    Events = 7U,
    Seal = 8U,
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
    std::size_t offset = 0U;
    while (offset < bytes.size()) {
        const ssize_t count = write(descriptor, bytes.data() + offset,
            bytes.size() - offset);
        if (count <= 0) {
            static_cast<void>(close(descriptor));
            return false;
        }
        offset += static_cast<std::size_t>(count);
    }
    return close(descriptor) == 0;
}

} // namespace

int main(int argc, char** argv) {
    if (argc != 6) return 64;
    CheckerWork checker;
    FileImage<CACHE_SIZE> cache;
    FileImage<PARENT_SIZE> parent;
    FileImage<PARENT_AUDIT_SIZE> parent_audit;
    FileImage<RECEIPT_SIZE> receipt;
    load_exact(argv[1], cache, checker);
    load_exact(argv[2], parent, checker);
    load_exact(argv[3], parent_audit, checker);
    load_exact(argv[4], receipt, checker);

    const bool reads_exact = cache.exact && parent.exact
        && parent_audit.exact && receipt.exact;
    const bool cache_identity = cache.exact && digest_is(cache.root, CACHE_ID);
    const bool parent_identity = parent.exact
        && check_parent_artifact(parent, checker);
    const bool parent_audit_identity = parent_audit.exact
        && check_parent_audit(parent_audit, checker);
    const bool inputs_exact = cache_identity && parent_identity
        && parent_audit_identity;

    constexpr std::array<std::uint8_t, 8U> receipt_magic{{
        'N','E','R','6','3','Z','N','1'}};
    bool malformed = !receipt.exact;
    if (receipt.exact) {
        ++checker.receipt_header_checks;
        malformed = std::memcmp(receipt.bytes.data(), receipt_magic.data(),
            receipt_magic.size()) != 0;
        ++checker.receipt_header_checks;
        malformed = malformed || big_u32(receipt.bytes.data() + VERSION_AT)
            != 2U;
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
        for (std::size_t index = FLAGS_AT + 3U; index < CACHE_SIZE_AT; ++index)
            padding |= receipt.bytes[index];
        malformed = malformed || padding != 0U;
    }

    Inputs decoded;
    Replay replay;
    if (inputs_exact) {
        decoded = decode_cache(cache.bytes, checker);
        replay = reconstruct(cache, parent, parent_audit, decoded, checker);
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

    ++checker.route_checks;
    Route route = Route::Accepted;
    if (!reads_exact) route = Route::Read;
    else if (!inputs_exact) route = Route::Input;
    else if (malformed) route = Route::MalformedReceipt;
    else if (!replay.valid) route = Route::Arithmetic;
    else if (!semantic_exact) route = Route::Semantic;
    else if (!candidate_work_exact) route = Route::Work;
    else if (!events_exact) route = Route::Events;
    else if (!seals_exact) route = Route::Seal;

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
    checker.audit_write_calls = 1U;
    checker.audit_write_bytes = CHECKER_AUDIT_SIZE;
    checker.audit_zero_fill_bytes = CHECKER_AUDIT_SIZE;
    const Digest checker_result = root_checker(route, semantic_exact,
        candidate_work_exact, events_exact, seals_exact, sizes, roots,
        observed_result, replay.result, checker);

    std::array<std::uint8_t, CHECKER_AUDIT_SIZE> output{};
    constexpr std::array<std::uint8_t, 8U> audit_magic{{
        'N','E','R','6','3','Z','Q','1'}};
    std::memcpy(output.data(), audit_magic.data(), audit_magic.size());
    store_u32(output.data() + 8U, 1U);
    store_u64(output.data() + 12U, CHECKER_AUDIT_SIZE);
    store_u32(output.data() + 20U, static_cast<std::uint32_t>(route));
    output[24U] = semantic_exact ? 1U : 0U;
    output[25U] = candidate_work_exact ? 1U : 0U;
    output[26U] = events_exact ? 1U : 0U;
    output[27U] = seals_exact ? 1U : 0U;
    for (std::size_t index = 0U; index < sizes.size(); ++index)
        store_u64(output.data() + CHECKER_SIZES_AT + index * 8U,
            sizes[index]);
    for (std::size_t index = 0U; index < roots.size(); ++index)
        store_digest(output.data() + CHECKER_ROOTS_AT + index * 32U,
            roots[index]);
    store_u64(output.data() + CHECKER_WORK_COUNT_AT, CHECKER_WORK_FIELDS);
    const auto fields = checker.fields();
    for (std::size_t index = 0U; index < fields.size(); ++index)
        store_u64(output.data() + CHECKER_WORK_AT + index * 8U,
            fields[index]);
    store_digest(output.data() + CHECKER_OBSERVED_RESULT_AT, observed_result);
    store_digest(output.data() + CHECKER_REPLAY_RESULT_AT, replay.result);
    store_digest(output.data() + CHECKER_RESULT_AT, checker_result);
    if (!write_exact(argv[5], output)) return 65;
    return route == Route::Accepted ? 0 : 1;
}
