#include "package_format.hpp"

#include "../r63zm/fixed_sha256.hpp"

#include <algorithm>
#include <array>
#include <cfenv>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <fcntl.h>
#include <quadmath.h>
#include <unistd.h>

namespace {

__extension__ using Quad = __float128;
using Digest = r63zm::Digest;
using Work = std::array<std::uint64_t, r63zp_format::candidate_work_fields>;
using Route = r63zp_format::CandidateRoute;

constexpr std::size_t N = r63zp_format::dimension;
constexpr std::size_t M = r63zp_format::columns;
constexpr std::size_t FACTOR_ENTRIES = N * N;
constexpr std::size_t PRODUCT_BASE = 1236U;
constexpr std::size_t PRODUCT_BYTES = 1944U;
constexpr std::size_t PRODUCT_VALUES = 312U;
constexpr std::size_t ROLE2 = PRODUCT_BASE + 2U * PRODUCT_BYTES;
constexpr std::size_t ROLE2_INPUT_ROOT = 1004U + 3U * 32U;
constexpr std::size_t ROLE2_VALUE_ROOT = ROLE2 + 208U;
constexpr std::size_t CACHE_X0 = 770U;
constexpr std::size_t CACHE_X1 = 2410U;
constexpr std::size_t CACHE_TANGENT = 6448U;
constexpr std::size_t CACHE_SIGMA = 520528U;
constexpr std::size_t CACHE_FACTOR = 520624U;
constexpr std::size_t CACHE_PERMUTATION = 603864U;
constexpr std::size_t CACHE_INVERSE = 604680U;
constexpr std::size_t CACHE_RHS = 604920U;
constexpr Quad ZERO = static_cast<Quad>(0.0);
constexpr Quad ONE = static_cast<Quad>(1.0);

constexpr char CACHE_ID[] =
    "23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84";
constexpr char ARTIFACT_ID[] =
    "ac6946e872799baef366d8a6648e7bb5cd70c6f2acc326747fdf153c471f0b87";
constexpr char AUDIT_ID[] =
    "fd4bcf0094e9f6ab8c64080c3a22566e3a23d2544e187641075350519a281f80";
constexpr char PARENT_VECTOR_DOMAIN[] =
    "nextengine.nonlocal.r63zm.binary128-vector.v1";
constexpr char VECTOR_DOMAIN[] =
    "nextengine.nonlocal.r63zp.quad-vector.v1";
constexpr char WORK_DOMAIN[] =
    "nextengine.nonlocal.r63zp.candidate-work.v1";
constexpr char BODY_DOMAIN[] =
    "nextengine.nonlocal.r63zp.product-body.v1";
constexpr char EVENT_DOMAIN[] =
    "nextengine.nonlocal.r63zp.candidate-event.v1";
constexpr char TRACE_DOMAIN[] =
    "nextengine.nonlocal.r63zp.candidate-trace.v1";
constexpr char RESULT_DOMAIN[] =
    "nextengine.nonlocal.r63zp.candidate-result.v1";

static_assert(sizeof(Quad) == r63zp_format::quad_bytes);
static_assert(__FLT128_MANT_DIG__ == 113);
static_assert(__FLT128_MAX_EXP__ == 16384);
static_assert(ROLE2 + PRODUCT_VALUES == 5436U);
static_assert(CACHE_TANGENT + N * M * sizeof(Quad) == CACHE_SIGMA);
static_assert(CACHE_FACTOR + FACTOR_ENTRIES * sizeof(double) + 8U
    == CACHE_PERMUTATION);

void be32(std::uint8_t* out, std::uint32_t value) noexcept {
    for (std::size_t i = 0U; i < 4U; ++i)
        out[i] = static_cast<std::uint8_t>(value >> (24U - 8U * i));
}

void be64(std::uint8_t* out, std::uint64_t value) noexcept {
    for (std::size_t i = 0U; i < 8U; ++i)
        out[i] = static_cast<std::uint8_t>(value >> (56U - 8U * i));
}

std::uint64_t load_be64(const std::uint8_t* bytes) noexcept {
    std::uint64_t value = 0U;
    for (std::size_t i = 0U; i < 8U; ++i)
        value = (value << 8U) | bytes[i];
    return value;
}

Quad little_quad(const std::uint8_t* bytes) noexcept {
    Quad value = ZERO;
    std::memcpy(&value, bytes, sizeof(value));
    return value;
}

Quad big_quad(const std::uint8_t* bytes) noexcept {
    std::array<std::uint8_t, sizeof(Quad)> little{};
    for (std::size_t i = 0U; i < little.size(); ++i)
        little[i] = bytes[little.size() - 1U - i];
    return little_quad(little.data());
}

double little_double(const std::uint8_t* bytes) noexcept {
    double value = 0.0;
    std::memcpy(&value, bytes, sizeof(value));
    return value;
}

std::uint64_t little_u64(const std::uint8_t* bytes) noexcept {
    std::uint64_t value = 0U;
    std::memcpy(&value, bytes, sizeof(value));
    return value;
}

std::array<std::uint8_t, sizeof(Quad)> canonical(Quad value) noexcept {
    std::array<std::uint8_t, sizeof(Quad)> little{};
    std::array<std::uint8_t, sizeof(Quad)> result{};
    std::memcpy(little.data(), &value, little.size());
    for (std::size_t i = 0U; i < result.size(); ++i)
        result[i] = little[result.size() - 1U - i];
    return result;
}

std::uint8_t nibble(char value) noexcept {
    if (value >= '0' && value <= '9')
        return static_cast<std::uint8_t>(value - '0');
    if (value >= 'a' && value <= 'f')
        return static_cast<std::uint8_t>(value - 'a' + 10);
    return 0xffU;
}

bool digest_is(const Digest& digest, const char* expected) noexcept {
    bool equal = expected[64U] == '\0';
    for (std::size_t i = 0U; i < digest.size(); ++i) {
        const std::uint8_t high = nibble(expected[2U * i]);
        const std::uint8_t low = nibble(expected[2U * i + 1U]);
        const bool component = high <= 15U && low <= 15U
            && digest[i] == static_cast<std::uint8_t>((high << 4U) | low);
        equal = component && equal;
    }
    return equal;
}

bool digest_equal(const Digest& left, const Digest& right) noexcept {
    bool equal = true;
    for (std::size_t i = 0U; i < left.size(); ++i)
        equal = (left[i] == right[i]) && equal;
    return equal;
}

Digest load_digest(const std::uint8_t* bytes) noexcept {
    Digest result{};
    std::memcpy(result.data(), bytes, result.size());
    return result;
}

void store_digest(std::uint8_t* bytes, const Digest& digest) noexcept {
    std::memcpy(bytes, digest.data(), digest.size());
}

void store_quad(std::uint8_t* bytes, Quad value) noexcept {
    const auto encoded = canonical(value);
    std::memcpy(bytes, encoded.data(), encoded.size());
}

struct RootBuilder final {
    r63zm::Sha256 hash;
    std::uint64_t bytes = 0U;

    void raw(const std::uint8_t* value, std::size_t count) noexcept {
        hash.update(std::span<const std::uint8_t>(value, count));
        bytes += count;
    }
    void byte(std::uint8_t value) noexcept {
        hash.update_byte(value);
        ++bytes;
    }
    void raw_u64(std::uint64_t value) noexcept {
        std::array<std::uint8_t, 8U> encoded{};
        be64(encoded.data(), value);
        raw(encoded.data(), encoded.size());
    }
    void header(std::uint8_t tag, std::uint64_t count) noexcept {
        byte(tag);
        raw_u64(count);
    }
    void field_raw(const std::uint8_t* value, std::size_t count) noexcept {
        header(1U, count);
        raw(value, count);
    }
    void field_u32(std::uint32_t value) noexcept {
        std::array<std::uint8_t, 4U> encoded{};
        be32(encoded.data(), value);
        header(2U, encoded.size());
        raw(encoded.data(), encoded.size());
    }
    void field_u64(std::uint64_t value) noexcept {
        header(3U, 8U);
        raw_u64(value);
    }
    void field_quad(Quad value) noexcept {
        const auto encoded = canonical(value);
        header(4U, encoded.size());
        raw(encoded.data(), encoded.size());
    }
    void field_digest(const Digest& digest) noexcept {
        header(5U, digest.size());
        raw(digest.data(), digest.size());
    }
    template <std::size_t Count>
    void field_u64s(const std::array<std::uint64_t, Count>& values,
        std::size_t used = Count) noexcept {
        header(6U, used * 8U);
        for (std::size_t i = 0U; i < used; ++i) raw_u64(values[i]);
    }
    template <std::size_t Count>
    void field_digests(const std::array<Digest, Count>& values,
        std::size_t used = Count) noexcept {
        header(7U, used * 32U);
        for (std::size_t i = 0U; i < used; ++i)
            raw(values[i].data(), values[i].size());
    }
    template <std::size_t Count>
    void field_quads(const std::array<Quad, Count>& values) noexcept {
        header(8U, Count * sizeof(Quad));
        for (Quad value : values) {
            const auto encoded = canonical(value);
            raw(encoded.data(), encoded.size());
        }
    }
    Digest finish() noexcept { return hash.finish(); }
};

template <std::size_t Size>
struct FileInput final {
    std::array<std::uint8_t, Size> bytes{};
    Digest root{};
    bool exact = false;
};

template <std::size_t Size>
void read_input(const char* path, FileInput<Size>& input, Work& work) noexcept {
    ++work[r63zp_format::CwInputOpenAttempts];
    const int descriptor = open(path, O_RDONLY);
    if (descriptor < 0) return;
    std::size_t offset = 0U;
    bool io_ok = true;
    while (offset < input.bytes.size()) {
        ++work[r63zp_format::CwInputReadCalls];
        const ssize_t count = read(descriptor, input.bytes.data() + offset,
            input.bytes.size() - offset);
        if (count <= 0) {
            io_ok = false;
            break;
        }
        offset += static_cast<std::size_t>(count);
        work[r63zp_format::CwInputReadBytes]
            += static_cast<std::uint64_t>(count);
    }
    std::uint8_t trailing = 0U;
    ++work[r63zp_format::CwInputReadCalls];
    ++work[r63zp_format::CwInputTrailingChecks];
    const ssize_t trailing_count = read(descriptor, &trailing, 1U);
    if (trailing_count > 0)
        work[r63zp_format::CwInputReadBytes]
            += static_cast<std::uint64_t>(trailing_count);
    ++work[r63zp_format::CwInputCloseCalls];
    const bool closed = close(descriptor) == 0;
    input.exact = io_ok && offset == input.bytes.size()
        && trailing_count == 0 && closed;
    if (input.exact) {
        ++work[r63zp_format::CwInputHashCalls];
        work[r63zp_format::CwInputHashBytes] += input.bytes.size();
        input.root = r63zm::sha256(input.bytes);
    }
}

bool normal_or_zero(Quad value) noexcept {
    return finiteq(value) != 0
        && (value == ZERO || fabsq(value) >= ldexpq(ONE, -16382));
}

Quad up_add(Quad left, Quad right) noexcept {
    if (left == ZERO && right == ZERO) return ZERO;
    return nextafterq(left + right, HUGE_VALQ);
}

Quad up_multiply(Quad left, Quad right) noexcept {
    if (left == ZERO || right == ZERO) return ZERO;
    return nextafterq(left * right, HUGE_VALQ);
}

Quad up_divide(Quad left, Quad right) noexcept {
    if (left == ZERO) return ZERO;
    return nextafterq(left / right, HUGE_VALQ);
}

Quad directed_binary(Quad left, Quad right, int mode, char operation) noexcept {
    if (std::fesetround(mode) != 0) return nanq("");
    volatile Quad result = operation == '+' ? left + right
        : operation == '-' ? left - right : left / right;
    if (std::fesetround(FE_TONEAREST) != 0) return nanq("");
    return result;
}

struct Pair final {
    bool exact = false;
    bool normal = false;
    Quad primary = ZERO;
    Quad error = ZERO;
};

Pair exact_sum(Quad left, Quad right) noexcept {
    Pair result;
    result.primary = left + right;
    const Quad virtual_right = result.primary - left;
    result.error = (left - (result.primary - virtual_right))
        + (right - virtual_right);
    result.exact = finiteq(result.primary) != 0
        && finiteq(result.error) != 0;
    result.normal = normal_or_zero(left) && normal_or_zero(right)
        && normal_or_zero(result.primary) && normal_or_zero(virtual_right)
        && normal_or_zero(result.error);
    return result;
}

Pair exact_product(Quad left, Quad right) noexcept {
    Pair result;
    result.primary = left * right;
    result.error = fmaq(left, right, -result.primary);
    const bool represented = left == ZERO || right == ZERO
        || result.primary != ZERO || result.error != ZERO;
    result.exact = finiteq(result.primary) != 0
        && finiteq(result.error) != 0;
    result.normal = represented && normal_or_zero(left)
        && normal_or_zero(right) && normal_or_zero(result.primary)
        && normal_or_zero(result.error);
    return result;
}

struct Dot final {
    bool exact = false;
    bool normal = false;
    Quad value = ZERO;
    Quad bound = ZERO;
};

template <class Left, class Right>
Dot dot2(std::size_t count, const Left& left, const Right& right) noexcept {
    Dot output;
    if (count == 0U) return output;
    const Pair initial = exact_product(left(0U), right(0U));
    Quad primary = initial.primary;
    Quad correction = initial.error;
    Quad absolute = up_multiply(fabsq(left(0U)), fabsq(right(0U)));
    bool exact = initial.exact;
    bool normal = initial.normal;
    for (std::size_t i = 1U; i < count; ++i) {
        const Pair product = exact_product(left(i), right(i));
        const Pair sum = exact_sum(primary, product.primary);
        const Quad local = sum.error + product.error;
        correction += local;
        primary = sum.primary;
        exact = product.exact && sum.exact && exact;
        normal = product.normal && sum.normal && normal
            && normal_or_zero(local) && normal_or_zero(correction);
        absolute = up_add(absolute,
            up_multiply(fabsq(left(i)), fabsq(right(i))));
    }
    output.value = primary + correction;
    normal = normal && normal_or_zero(output.value);
    const Quad unit = ldexpq(ONE, -112);
    const Quad count_unit = static_cast<Quad>(count) * unit;
    const Quad gamma = up_divide(count_unit, ONE - count_unit);
    const Quad numerator = up_add(up_multiply(unit, fabsq(output.value)),
        up_multiply(up_multiply(gamma, gamma), absolute));
    output.bound = up_divide(numerator, ONE - unit);
    output.exact = exact && finiteq(output.value) != 0
        && finiteq(output.bound) != 0 && output.bound >= ZERO;
    output.normal = normal;
    return output;
}

struct Accumulator final {
    Quad sum = ZERO;
    Quad correction = ZERO;
    void add(Quad value) noexcept {
        const Quad next = sum + value;
        correction += fabsq(sum) >= fabsq(value)
            ? (sum - next) + value : (value - next) + sum;
        sum = next;
    }
    Quad value() const noexcept { return sum + correction; }
};

struct Solve final {
    bool exact = false;
    std::uint64_t terms = 0U;
    std::uint64_t divisions = 0U;
    std::array<Quad, N> solution{};
};

Solve solve(const std::array<Quad, FACTOR_ENTRIES>& upper,
    const std::array<std::uint64_t, N>& permutation,
    const std::array<Quad, N>& rhs, Quad inverse, Work& work) noexcept {
    Solve output;
    std::array<Quad, N> forward{};
    std::array<Quad, N> backward{};
    bool valid = inverse > ZERO && finiteq(inverse) != 0;
    for (std::size_t row = 0U; row < N; ++row) {
        const bool active = valid && permutation[row] < N;
        if (!active) {
            valid = false;
            continue;
        }
        const Quad scaled = rhs[permutation[row]] * inverse;
        Accumulator dot;
        for (std::size_t column = 0U; column < row; ++column) {
            dot.add(upper[column * N + row] * forward[column]);
            ++output.terms;
        }
        const Quad diagonal = upper[row * N + row];
        const bool row_valid = diagonal != ZERO && finiteq(diagonal) != 0;
        valid = row_valid && valid;
        if (row_valid) {
            forward[row] = (scaled - dot.value()) / diagonal;
            ++output.divisions;
            valid = finiteq(forward[row]) != 0 && valid;
        }
    }
    for (std::size_t remaining = N; remaining > 0U;) {
        const std::size_t row = --remaining;
        Accumulator dot;
        for (std::size_t column = row + 1U; column < N; ++column) {
            dot.add(upper[row * N + column] * backward[column]);
            ++output.terms;
        }
        const Quad diagonal = upper[row * N + row];
        const bool row_valid = valid && diagonal != ZERO
            && finiteq(diagonal) != 0;
        if (row_valid) {
            backward[row] = (forward[row] - dot.value()) / diagonal;
            ++output.divisions;
            valid = finiteq(backward[row]) != 0 && valid;
        } else {
            valid = false;
        }
    }
    for (std::size_t i = 0U; i < N; ++i)
        if (permutation[i] < N) output.solution[permutation[i]] = backward[i];
    output.exact = valid && output.terms == 10302U
        && output.divisions == 204U;
    ++work[r63zp_format::CwFactorSolves];
    work[r63zp_format::CwFactorTerms] += output.terms;
    work[r63zp_format::CwFactorDivisions] += output.divisions;
    return output;
}

struct Product final {
    bool exact = false;
    std::array<Quad, M> intermediate{};
    std::array<Quad, M> intermediate_bound{};
    std::array<Quad, N> value{};
    std::array<Quad, N> bound{};
};

Product product(const std::array<Quad, N * M>& tangent, Quad sigma,
    const std::array<Quad, N>& input, Work& work) noexcept {
    Product output;
    bool exact = true;
    ++work[r63zp_format::CwProductKernels];
    for (std::size_t column = 0U; column < M; ++column) {
        const Dot dot = dot2(N,
            [&](std::size_t row) { return tangent[row * M + column]; },
            [&](std::size_t row) { return input[row]; });
        output.intermediate[column] = dot.value;
        output.intermediate_bound[column] = dot.bound;
        exact = dot.exact && dot.normal && exact;
        ++work[r63zp_format::CwInnerDots];
        work[r63zp_format::CwInnerTerms] += N;
    }
    for (std::size_t row = 0U; row < N; ++row) {
        const Dot dot = dot2(M,
            [&](std::size_t column) { return tangent[row * M + column]; },
            [&](std::size_t column) { return output.intermediate[column]; });
        ++work[r63zp_format::CwOuterDots];
        work[r63zp_format::CwOuterTerms] += M;
        Quad propagated = dot.bound;
        for (std::size_t column = 0U; column < M; ++column) {
            propagated = up_add(propagated, up_multiply(
                fabsq(tangent[row * M + column]),
                output.intermediate_bound[column]));
            ++work[r63zp_format::CwPropagationTerms];
        }
        const Pair scaled = exact_product(sigma, dot.value);
        ++work[r63zp_format::CwScaleProducts];
        output.value[row] = scaled.primary;
        output.bound[row] = up_add(fabsq(scaled.error),
            up_multiply(fabsq(sigma), propagated));
        exact = dot.exact && dot.normal && scaled.exact && scaled.normal
            && finiteq(output.value[row]) != 0
            && finiteq(output.bound[row]) != 0
            && output.bound[row] >= ZERO && exact;
    }
    output.exact = exact;
    return output;
}

Digest parent_vector_root(const std::array<Quad, N>& values,
    std::uint64_t& bytes) noexcept {
    RootBuilder builder;
    builder.header(1U, sizeof(PARENT_VECTOR_DOMAIN) - 1U);
    builder.raw(reinterpret_cast<const std::uint8_t*>(PARENT_VECTOR_DOMAIN),
        sizeof(PARENT_VECTOR_DOMAIN) - 1U);
    builder.header(2U, 8U);
    builder.raw_u64(N);
    builder.header(3U, N * sizeof(Quad));
    for (Quad value : values) {
        const auto encoded = canonical(value);
        builder.raw(encoded.data(), encoded.size());
    }
    bytes = builder.bytes;
    return builder.finish();
}

Digest vector_root(const std::array<Quad, N>& values,
    std::uint64_t& bytes) noexcept {
    RootBuilder builder;
    builder.field_raw(reinterpret_cast<const std::uint8_t*>(VECTOR_DOMAIN),
        sizeof(VECTOR_DOMAIN) - 1U);
    builder.field_u64(N);
    builder.field_quads(values);
    bytes = builder.bytes;
    return builder.finish();
}

Digest work_root(Route route, const Work& work, std::uint64_t& bytes) noexcept {
    RootBuilder builder;
    builder.field_raw(reinterpret_cast<const std::uint8_t*>(WORK_DOMAIN),
        sizeof(WORK_DOMAIN) - 1U);
    builder.field_u32(static_cast<std::uint32_t>(route));
    builder.field_u64s(work);
    bytes = builder.bytes;
    return builder.finish();
}

struct Semantic final {
    std::array<Digest, 3U> parents{};
    std::array<Digest, 2U> roles{};
    std::array<Digest, 3U> roots{};
    std::array<Quad, 5U> scalars{};
    std::array<Quad, N> p0{};
    std::array<Quad, N> q0{};
    std::array<Quad, N> bounds{};
    Digest product_body{};
    Digest derived_x1{};
    std::uint64_t update_matches = 0U;
};

Digest product_body_root(const Semantic& semantic, const Work& work,
    const std::array<Digest, r63zp_format::event_slots>& events,
    std::uint64_t& bytes) noexcept {
    RootBuilder builder;
    builder.field_raw(reinterpret_cast<const std::uint8_t*>(BODY_DOMAIN),
        sizeof(BODY_DOMAIN) - 1U);
    builder.field_digests(semantic.parents);
    builder.field_digests(semantic.roles);
    builder.field_digests(semantic.roots);
    for (Quad scalar : semantic.scalars) builder.field_quad(scalar);
    builder.field_u64(N);
    builder.field_u64s(work, 40U);
    builder.field_digests(events, 4U);
    builder.field_quads(semantic.p0);
    builder.field_quads(semantic.q0);
    builder.field_quads(semantic.bounds);
    bytes = builder.bytes;
    return builder.finish();
}

Digest candidate_event(std::uint32_t ordinal, Route stage_route,
    const Semantic& semantic, const Digest& derived_x0,
    const Digest& work_digest, std::uint64_t& bytes) noexcept {
    RootBuilder builder;
    builder.field_raw(reinterpret_cast<const std::uint8_t*>(EVENT_DOMAIN),
        sizeof(EVENT_DOMAIN) - 1U);
    builder.field_u32(ordinal);
    builder.field_u32(static_cast<std::uint32_t>(stage_route));
    switch (ordinal) {
    case 1U:
        builder.field_digests(semantic.parents);
        break;
    case 2U:
        builder.field_digest(semantic.roles[0U]);
        builder.field_digest(derived_x0);
        break;
    case 3U:
        builder.field_digests(semantic.roles);
        break;
    case 4U:
        builder.field_digest(semantic.roots[0U]);
        builder.field_quad(semantic.scalars[0U]);
        builder.field_quad(semantic.scalars[1U]);
        break;
    case 5U:
        builder.field_digest(semantic.product_body);
        break;
    case 6U:
        builder.field_digest(semantic.roots[1U]);
        builder.field_digest(semantic.roots[2U]);
        builder.field_quad(semantic.scalars[2U]);
        builder.field_quad(semantic.scalars[3U]);
        builder.field_quad(semantic.scalars[4U]);
        break;
    case 7U:
        builder.field_digest(semantic.derived_x1);
        builder.field_u64(semantic.update_matches);
        break;
    case 8U:
        builder.field_digest(semantic.product_body);
        builder.field_digest(work_digest);
        break;
    default:
        break;
    }
    bytes = builder.bytes;
    return builder.finish();
}

Digest trace_root(std::uint64_t count,
    const std::array<Digest, r63zp_format::event_slots>& events,
    std::uint64_t& bytes) noexcept {
    RootBuilder builder;
    builder.field_raw(reinterpret_cast<const std::uint8_t*>(TRACE_DOMAIN),
        sizeof(TRACE_DOMAIN) - 1U);
    builder.field_u64(count);
    builder.field_digests(events);
    bytes = builder.bytes;
    return builder.finish();
}

Digest result_root(Route route, std::uint32_t flags,
    const Semantic& semantic, const Digest& work_digest,
    std::uint64_t event_count, const Digest& trace,
    std::uint64_t& bytes) noexcept {
    RootBuilder builder;
    builder.field_raw(reinterpret_cast<const std::uint8_t*>(RESULT_DOMAIN),
        sizeof(RESULT_DOMAIN) - 1U);
    builder.field_u32(static_cast<std::uint32_t>(route));
    builder.field_u32(flags);
    builder.field_digests(semantic.parents);
    builder.field_digests(semantic.roles);
    builder.field_digests(semantic.roots);
    for (Quad scalar : semantic.scalars) builder.field_quad(scalar);
    builder.field_u64(N);
    builder.field_digest(work_digest);
    builder.field_u64(event_count);
    builder.field_digest(trace);
    builder.field_digest(semantic.product_body);
    builder.field_u64(semantic.update_matches);
    bytes = builder.bytes;
    return builder.finish();
}

bool parent_headers(const FileInput<r63zp_format::parent_artifact_bytes>& artifact,
    const FileInput<r63zp_format::parent_audit_bytes>& audit,
    Work& work) noexcept {
    constexpr std::array<std::uint8_t, 8U> artifact_magic{{
        'N','E','R','6','3','Z','M','1'}};
    constexpr std::array<std::uint8_t, 8U> audit_magic{{
        'N','E','R','6','3','Q','C','1'}};
    bool valid = true;
    auto check = [&](bool predicate) noexcept {
        ++work[r63zp_format::CwParentHeaderPredicates];
        valid = predicate && valid;
    };
    check(std::memcmp(artifact.bytes.data(), artifact_magic.data(), 8U) == 0);
    check(load_be64(artifact.bytes.data() + 12U)
        == r63zp_format::parent_artifact_bytes);
    check(artifact.bytes[ROLE2] == 1U);
    check(artifact.bytes[ROLE2 + 1U] == 1U);
    check(artifact.bytes[ROLE2 + 2U] == 2U);
    check(load_be64(artifact.bytes.data() + ROLE2 + 304U) == N);
    check(std::memcmp(audit.bytes.data(), audit_magic.data(), 8U) == 0);
    check(load_be64(audit.bytes.data() + 12U)
        == r63zp_format::parent_audit_bytes);
    check(audit.bytes[24U] == 1U);
    check(audit.bytes[25U] == 1U && audit.bytes[26U] == 1U);
    return valid;
}

void serialize(std::array<std::uint8_t, r63zp_format::candidate_bytes>& out,
    Route route, std::uint32_t flags, const Semantic& semantic,
    const Work& work, const Digest& work_digest, std::uint64_t event_count,
    const std::array<Digest, r63zp_format::event_slots>& events,
    const Digest& trace, const Digest& result) noexcept {
    std::memcpy(out.data(), r63zp_format::candidate_magic.data(), 8U);
    be32(out.data() + r63zp_format::c_version, r63zp_format::version);
    be32(out.data() + r63zp_format::c_total,
        static_cast<std::uint32_t>(out.size()));
    be32(out.data() + r63zp_format::c_route,
        static_cast<std::uint32_t>(route));
    be32(out.data() + r63zp_format::c_flags, flags);
    for (std::size_t i = 0U; i < semantic.parents.size(); ++i)
        store_digest(out.data() + r63zp_format::c_parent_roots + i * 32U,
            semantic.parents[i]);
    for (std::size_t i = 0U; i < semantic.roles.size(); ++i)
        store_digest(out.data() + r63zp_format::c_role_roots + i * 32U,
            semantic.roles[i]);
    for (std::size_t i = 0U; i < semantic.roots.size(); ++i)
        store_digest(out.data() + r63zp_format::c_semantic_roots + i * 32U,
            semantic.roots[i]);
    for (std::size_t i = 0U; i < semantic.scalars.size(); ++i)
        store_quad(out.data() + r63zp_format::c_scalars + i * sizeof(Quad),
            semantic.scalars[i]);
    be64(out.data() + r63zp_format::c_dimension, N);
    be64(out.data() + r63zp_format::c_work_count,
        r63zp_format::candidate_work_fields);
    for (std::size_t i = 0U; i < work.size(); ++i)
        be64(out.data() + r63zp_format::c_work + i * 8U, work[i]);
    store_digest(out.data() + r63zp_format::c_work_root, work_digest);
    be64(out.data() + r63zp_format::c_event_count, event_count);
    for (std::size_t i = 0U; i < events.size(); ++i)
        store_digest(out.data() + r63zp_format::c_events + i * 32U, events[i]);
    store_digest(out.data() + r63zp_format::c_trace_root, trace);
    for (std::size_t i = 0U; i < N; ++i) {
        store_quad(out.data() + r63zp_format::c_p0 + i * sizeof(Quad),
            semantic.p0[i]);
        store_quad(out.data() + r63zp_format::c_q0 + i * sizeof(Quad),
            semantic.q0[i]);
        store_quad(out.data() + r63zp_format::c_bounds + i * sizeof(Quad),
            semantic.bounds[i]);
    }
    store_digest(out.data() + r63zp_format::c_product_body_root,
        semantic.product_body);
    store_digest(out.data() + r63zp_format::c_result_root, result);
}

bool write_output(const char* path,
    const std::array<std::uint8_t, r63zp_format::candidate_bytes>& bytes)
    noexcept {
    const int descriptor = open(path, O_WRONLY | O_CREAT | O_TRUNC, 0600);
    if (descriptor < 0) return false;
    const ssize_t count = write(descriptor, bytes.data(), bytes.size());
    const bool closed = close(descriptor) == 0;
    return count == static_cast<ssize_t>(bytes.size()) && closed;
}

} // namespace

int main(int argc, char** argv) {
    if (argc != 5) return 64;

    Work work{};
    ++work[r63zp_format::CwRoundingModeSetCalls];
    const bool rounding_set = std::fesetround(FE_TONEAREST) == 0;
    ++work[r63zp_format::CwRoundingModeChecks];
    const bool rounding_exact = rounding_set
        && std::fegetround() == FE_TONEAREST;

    FileInput<r63zp_format::parent_cache_bytes> cache;
    FileInput<r63zp_format::parent_artifact_bytes> parent;
    FileInput<r63zp_format::parent_audit_bytes> parent_audit;
    read_input(argv[1], cache, work);
    read_input(argv[2], parent, work);
    read_input(argv[3], parent_audit, work);

    Semantic semantic{};
    semantic.parents = {{cache.root, parent.root, parent_audit.root}};
    bool parents_exact = rounding_exact && cache.exact && parent.exact
        && parent_audit.exact;
    const std::array<bool, 3U> identities{{digest_is(cache.root, CACHE_ID),
        digest_is(parent.root, ARTIFACT_ID),
        digest_is(parent_audit.root, AUDIT_ID)}};
    for (bool identity : identities) {
        ++work[r63zp_format::CwParentRootComparisons];
        parents_exact = identity && parents_exact;
    }
    if (parents_exact) parents_exact = parent_headers(parent, parent_audit, work);

    Route route = Route::ApparatusRejected;
    std::uint32_t flags = 0U;
    std::array<Digest, r63zp_format::event_slots> events{};
    std::uint64_t event_count = 0U;
    Digest derived_x0{};
    std::uint64_t update_matches = 0U;

    std::array<Quad, N> x0{};
    std::array<Quad, N> rhs{};
    std::array<Quad, N * M> tangent{};
    std::array<Quad, FACTOR_ENTRIES> factor{};
    std::array<std::uint64_t, N> permutation{};
    Quad sigma = ZERO;
    Quad inverse = ZERO;
    bool direction_exact = true;
    bool product_exact = false;
    bool bounds_valid = false;
    bool curvature_positive = false;
    bool step_valid = false;

    if (parents_exact) {
        for (std::size_t i = 0U; i < N; ++i) {
            x0[i] = little_quad(cache.bytes.data() + CACHE_X0
                + i * sizeof(Quad));
            rhs[i] = little_quad(cache.bytes.data() + CACHE_RHS
                + i * sizeof(Quad));
            permutation[i] = little_u64(cache.bytes.data() + CACHE_PERMUTATION
                + i * sizeof(std::uint64_t));
            work[r63zp_format::CwCacheQuadDecodesPreseal] += 2U;
            ++work[r63zp_format::CwCacheIndexDecodes];
            work[r63zp_format::CwFinitePredicates] += 2U;
            ++work[r63zp_format::CwRangePredicates];
            const bool row_valid = finiteq(x0[i]) != 0
                && finiteq(rhs[i]) != 0 && permutation[i] < N;
            direction_exact = row_valid && direction_exact;
        }
        for (std::size_t i = 0U; i < tangent.size(); ++i) {
            tangent[i] = little_quad(cache.bytes.data() + CACHE_TANGENT
                + i * sizeof(Quad));
            ++work[r63zp_format::CwCacheQuadDecodesPreseal];
            ++work[r63zp_format::CwFinitePredicates];
            direction_exact = finiteq(tangent[i]) != 0 && direction_exact;
        }
        for (std::size_t i = 0U; i < factor.size(); ++i) {
            factor[i] = static_cast<Quad>(little_double(cache.bytes.data()
                + CACHE_FACTOR + i * sizeof(double)));
            ++work[r63zp_format::CwCacheDoubleDecodes];
            ++work[r63zp_format::CwFinitePredicates];
            direction_exact = finiteq(factor[i]) != 0 && direction_exact;
        }
        sigma = little_quad(cache.bytes.data() + CACHE_SIGMA);
        inverse = little_quad(cache.bytes.data() + CACHE_INVERSE);
        work[r63zp_format::CwCacheQuadDecodesPreseal] += 2U;
        work[r63zp_format::CwFinitePredicates] += 2U;
        work[r63zp_format::CwRangePredicates] += 2U;
        direction_exact = finiteq(sigma) != 0 && finiteq(inverse) != 0
            && sigma > ZERO && inverse > ZERO && direction_exact;

        const Solve start = solve(factor, permutation, rhs, inverse, work);
        bool x0_equal = start.exact;
        for (std::size_t i = 0U; i < N; ++i) {
            const bool equal = std::memcmp(&start.solution[i], &x0[i],
                sizeof(Quad)) == 0;
            ++work[r63zp_format::CwX0ComponentComparisons];
            ++work[r63zp_format::CwFixedLoopIterations];
            x0_equal = equal && x0_equal;
        }
        std::uint64_t root_bytes = 0U;
        derived_x0 = parent_vector_root(start.solution, root_bytes);
        ++work[r63zp_format::CwPresealRootCalls];
        work[r63zp_format::CwPresealRootBytes] += root_bytes;
        semantic.roles[0U] = load_digest(parent.bytes.data()
            + ROLE2_INPUT_ROOT);
        ++work[r63zp_format::CwParentRootComparisons];
        x0_equal = digest_equal(derived_x0, semantic.roles[0U]) && x0_equal;

        if (direction_exact && x0_equal) {
            std::array<Quad, N> y0{};
            for (std::size_t i = 0U; i < N; ++i) {
                y0[i] = big_quad(parent.bytes.data() + ROLE2
                    + PRODUCT_VALUES + i * sizeof(Quad));
                ++work[r63zp_format::CwParentArtifactQuadDecodes];
                ++work[r63zp_format::CwFinitePredicates];
                direction_exact = finiteq(y0[i]) != 0 && direction_exact;
            }
            semantic.roles[1U] = load_digest(parent.bytes.data()
                + ROLE2_VALUE_ROOT);
            std::uint64_t y0_root_bytes = 0U;
            const Digest y0_root = parent_vector_root(y0, y0_root_bytes);
            ++work[r63zp_format::CwPresealRootCalls];
            work[r63zp_format::CwPresealRootBytes] += y0_root_bytes;
            ++work[r63zp_format::CwParentRootComparisons];
            direction_exact = digest_equal(y0_root, semantic.roles[1U])
                && direction_exact;

            std::array<Quad, N> residual{};
            bool residual_exact = true;
            for (std::size_t i = 0U; i < N; ++i) {
                const Dot dot = dot2(2U,
                    [&](std::size_t term) {
                        return term == 0U ? rhs[i] : -y0[i];
                    },
                    [&](std::size_t) { return ONE; });
                ++work[r63zp_format::CwResidualDots];
                work[r63zp_format::CwResidualTerms] += 2U;
                residual[i] = dot.value;
                residual_exact = dot.exact && dot.normal && residual_exact;
            }
            const Solve preconditioned = solve(factor, permutation, residual,
                inverse, work);
            semantic.p0 = preconditioned.solution;
            const Dot rho = dot2(N,
                [&](std::size_t i) { return residual[i]; },
                [&](std::size_t i) { return semantic.p0[i]; });
            ++work[r63zp_format::CwRhoDots];
            work[r63zp_format::CwRhoTerms] += N;
            semantic.scalars[0U] = rho.value;
            semantic.scalars[1U] = rho.bound;
            ++work[r63zp_format::CwPositivityPredicates];
            direction_exact = direction_exact && residual_exact
                && preconditioned.exact && rho.exact && rho.normal
                && rho.value - rho.bound > ZERO;

            if (direction_exact) {
                const Product direct = product(tangent, sigma, semantic.p0,
                    work);
                semantic.q0 = direct.value;
                semantic.bounds = direct.bound;
                product_exact = direct.exact;
                bounds_valid = true;
                for (std::size_t i = 0U; i < N; ++i) {
                    ++work[r63zp_format::CwFinitePredicates];
                    ++work[r63zp_format::CwRangePredicates];
                    bounds_valid = finiteq(semantic.bounds[i]) != 0
                        && semantic.bounds[i] >= ZERO && bounds_valid;
                }

                std::uint64_t p_bytes = 0U;
                std::uint64_t q_bytes = 0U;
                std::uint64_t b_bytes = 0U;
                semantic.roots[0U] = vector_root(semantic.p0, p_bytes);
                semantic.roots[1U] = vector_root(semantic.q0, q_bytes);
                semantic.roots[2U] = vector_root(semantic.bounds, b_bytes);
                work[r63zp_format::CwPresealRootCalls] += 3U;
                work[r63zp_format::CwPresealRootBytes]
                    += p_bytes + q_bytes + b_bytes;

                const Dot denominator = dot2(N,
                    [&](std::size_t i) { return semantic.p0[i]; },
                    [&](std::size_t i) { return semantic.q0[i]; });
                ++work[r63zp_format::CwDenominatorDots];
                work[r63zp_format::CwDenominatorTerms] += N;
                Quad denominator_bound = denominator.bound;
                for (std::size_t i = 0U; i < N; ++i) {
                    denominator_bound = up_add(denominator_bound,
                        up_multiply(fabsq(semantic.p0[i]),
                            semantic.bounds[i]));
                    ++work[r63zp_format::CwDenominatorTerms];
                }
                semantic.scalars[2U] = denominator.value;
                semantic.scalars[3U] = denominator_bound;
                const Quad rho_lower = directed_binary(rho.value, rho.bound,
                    FE_DOWNWARD, '-');
                const Quad rho_upper = directed_binary(rho.value, rho.bound,
                    FE_UPWARD, '+');
                const Quad denominator_lower = directed_binary(
                    denominator.value, denominator_bound, FE_DOWNWARD, '-');
                const Quad denominator_upper = directed_binary(
                    denominator.value, denominator_bound, FE_UPWARD, '+');
                work[r63zp_format::CwIntervalEndpointOperations] += 4U;
                ++work[r63zp_format::CwPositivityPredicates];
                curvature_positive = product_exact && bounds_valid
                    && denominator.exact && denominator.normal
                    && denominator_lower > ZERO;
                if (curvature_positive) {
                    const Quad alpha_lower = directed_binary(rho_lower,
                        denominator_upper, FE_DOWNWARD, '/');
                    const Quad alpha_upper = directed_binary(rho_upper,
                        denominator_lower, FE_UPWARD, '/');
                    work[r63zp_format::CwDivisionEndpointOperations] += 2U;
                    semantic.scalars[4U] = rho.value / denominator.value;
                    ++work[r63zp_format::CwDivisionEndpointOperations];
                    work[r63zp_format::CwDivisionPredicates] += 2U;
                    step_valid = finiteq(alpha_lower) != 0
                        && finiteq(alpha_upper) != 0
                        && alpha_lower <= semantic.scalars[4U]
                        && semantic.scalars[4U] <= alpha_upper;
                }
            }
        }
    }

    auto route_reject = [&](bool condition) noexcept {
        ++work[r63zp_format::CwRoutePredicates];
        return condition;
    };
    if (route_reject(!parents_exact)) route = Route::ApparatusRejected;
    else if (route_reject(!direction_exact)) route = Route::DirectionInputRejected;
    else if (route_reject(!product_exact || !bounds_valid))
        route = Route::ProductBoundRejected;
    else if (route_reject(!curvature_positive)) route = Route::CurvatureRejected;
    else if (route_reject(!step_valid)) route = Route::StepRejected;
    else {
        ++work[r63zp_format::CwRoutePredicates];
        route = Route::Admitted;
    }

    if (direction_exact) flags |= 1U << 0U;
    if (product_exact) flags |= 1U << 1U;
    if (bounds_valid) flags |= 1U << 2U;
    if (curvature_positive) flags |= 1U << 3U;
    if (step_valid) flags |= 1U << 4U;

    if (parents_exact) event_count = 1U;
    if (direction_exact) event_count = 4U;
    if (product_exact && bounds_valid && curvature_positive && step_valid)
        event_count = 6U;

    work[r63zp_format::CwEventRootCalls] = event_count;
    work[r63zp_format::CwTraceRootCalls] = 1U;
    work[r63zp_format::CwReceiptZeroFillBytes]
        = r63zp_format::candidate_bytes;
    work[r63zp_format::CwReceiptFieldsSerialized] = 400U;
    work[r63zp_format::CwReceiptBytesSerialized]
        = r63zp_format::candidate_bytes;
    work[r63zp_format::CwOutputOpenAttempts] = 1U;
    work[r63zp_format::CwOutputWriteCalls] = 1U;
    work[r63zp_format::CwOutputWriteBytes]
        = r63zp_format::candidate_bytes;
    work[r63zp_format::CwOutputCloseCalls] = 1U;
    work[r63zp_format::CwPackageAllocations] = 0U;

    std::uint64_t observed_event_bytes = 0U;
    for (std::uint32_t ordinal = 1U; ordinal <= event_count
         && ordinal <= 4U; ++ordinal) {
        std::uint64_t bytes = 0U;
        events[ordinal - 1U] = candidate_event(ordinal, Route::Admitted,
            semantic, derived_x0, Digest{}, bytes);
        observed_event_bytes += bytes;
    }

    if (event_count >= 5U || route == Route::Admitted) {
        const std::uint64_t body_bytes_expected =
            9U + (sizeof(BODY_DOMAIN) - 1U)
            + (9U + 3U * 32U) + (9U + 2U * 32U) + (9U + 3U * 32U)
            + 5U * (9U + 16U) + (9U + 8U) + (9U + 40U * 8U)
            + (9U + 4U * 32U) + 3U * (9U + N * 16U);
        ++work[r63zp_format::CwPresealRootCalls];
        work[r63zp_format::CwPresealRootBytes] += body_bytes_expected;
        std::uint64_t body_bytes = 0U;
        semantic.product_body = product_body_root(semantic, work, events,
            body_bytes);
        if (body_bytes != body_bytes_expected) return 65;
        std::uint64_t event_bytes = 0U;
        events[4U] = candidate_event(5U, Route::Admitted, semantic,
            derived_x0, Digest{}, event_bytes);
        observed_event_bytes += event_bytes;
        if (event_count < 5U) event_count = 5U;
    }

    if (event_count >= 6U) {
        std::uint64_t event_bytes = 0U;
        events[5U] = candidate_event(6U, Route::Admitted, semantic,
            derived_x0, Digest{}, event_bytes);
        observed_event_bytes += event_bytes;
    }

    if (route == Route::Admitted) {
        std::array<Quad, N> x1{};
        for (std::size_t i = 0U; i < N; ++i) {
            x1[i] = little_quad(cache.bytes.data() + CACHE_X1
                + i * sizeof(Quad));
            ++work[r63zp_format::CwPostsealX1Decodes];
            ++work[r63zp_format::CwPostsealX1FinitePredicates];
            const Dot update = dot2(2U,
                [&](std::size_t term) {
                    return term == 0U ? x0[i] : semantic.scalars[4U];
                },
                [&](std::size_t term) {
                    return term == 0U ? ONE : semantic.p0[i];
                });
            ++work[r63zp_format::CwUpdateDots];
            work[r63zp_format::CwUpdateTerms] += 2U;
            ++work[r63zp_format::CwUpdateComparisons];
            ++work[r63zp_format::CwFixedLoopIterations];
            const bool match = update.exact && update.normal
                && finiteq(x1[i]) != 0
                && std::memcmp(&update.value, &x1[i], sizeof(Quad)) == 0;
            update_matches += match ? 1U : 0U;
        }
        semantic.update_matches = update_matches;
        std::uint64_t x1_root_bytes = 0U;
        semantic.derived_x1 = parent_vector_root(x1, x1_root_bytes);
        work[r63zp_format::CwFinalRootCalls] = 3U;
        work[r63zp_format::CwFinalRootBytes] = x1_root_bytes;
        if (update_matches != N) route = Route::UpdateRejected;
        else flags |= 1U << 5U;
        event_count = 7U;
        std::uint64_t event_bytes = 0U;
        events[6U] = candidate_event(7U, route, semantic, derived_x0,
            Digest{}, event_bytes);
        observed_event_bytes += event_bytes;
    } else {
        work[r63zp_format::CwFinalRootCalls] = 2U;
    }

    const std::uint64_t work_bytes_expected =
        9U + (sizeof(WORK_DOMAIN) - 1U) + (9U + 4U)
        + (9U + work.size() * 8U);
    const std::uint64_t result_bytes_expected =
        9U + (sizeof(RESULT_DOMAIN) - 1U) + 2U * (9U + 4U)
        + (9U + 3U * 32U) + (9U + 2U * 32U) + (9U + 3U * 32U)
        + 5U * (9U + 16U) + (9U + 8U) + (9U + 32U)
        + (9U + 8U) + 2U * (9U + 32U) + (9U + 8U);
    work[r63zp_format::CwFinalRootBytes]
        += work_bytes_expected + result_bytes_expected;

    const bool final_event_available = event_count == 7U;
    const std::uint64_t event8_expected = final_event_available
        ? 9U + (sizeof(EVENT_DOMAIN) - 1U)
            + 2U * (9U + 4U) + 2U * (9U + 32U)
        : 0U;
    work[r63zp_format::CwEventRootCalls] = event_count
        + (final_event_available ? 1U : 0U);
    work[r63zp_format::CwEventRootBytes] = observed_event_bytes
        + event8_expected;
    const std::uint64_t trace_bytes_expected =
        9U + (sizeof(TRACE_DOMAIN) - 1U) + (9U + 8U)
        + (9U + r63zp_format::event_slots * 32U);
    work[r63zp_format::CwTraceRootBytes] = trace_bytes_expected;

    std::uint64_t work_bytes = 0U;
    const Digest work_digest = work_root(route, work, work_bytes);
    if (work_bytes != work_bytes_expected) return 65;
    if (final_event_available) {
        std::uint64_t event8_bytes = 0U;
        events[7U] = candidate_event(8U, route, semantic, derived_x0,
            work_digest, event8_bytes);
        if (event8_bytes != event8_expected) return 65;
        event_count = 8U;
    }
    std::uint64_t trace_bytes = 0U;
    const Digest trace = trace_root(event_count, events, trace_bytes);
    if (trace_bytes != trace_bytes_expected) return 65;
    std::uint64_t result_bytes = 0U;
    const Digest result = result_root(route, flags, semantic, work_digest,
        event_count, trace, result_bytes);
    if (result_bytes != result_bytes_expected) return 65;

    std::array<std::uint8_t, r63zp_format::candidate_bytes> output{};
    serialize(output, route, flags, semantic, work, work_digest, event_count,
        events, trace, result);
    if (!write_output(argv[4], output)) return 65;
    return route == Route::Admitted ? 0 : 1;
}
