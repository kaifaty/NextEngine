#include "package_format.hpp"

#include "../r63zm/fixed_sha256.hpp"

#include <algorithm>
#include <array>
#include <bit>
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
using CandidateRoute = r63zp_format::CandidateRoute;
using CheckerRoute = r63zp_format::CheckerRoute;
using CheckerWork = std::array<std::uint64_t,
    r63zp_format::checker_work_fields>;
using CandidateWork = std::array<std::uint64_t,
    r63zp_format::candidate_work_fields>;

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
constexpr std::size_t LIMBS = 64U;

constexpr char CACHE_ID[] =
    "23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84";
constexpr char ARTIFACT_ID[] =
    "ac6946e872799baef366d8a6648e7bb5cd70c6f2acc326747fdf153c471f0b87";
constexpr char AUDIT_ID[] =
    "fd4bcf0094e9f6ab8c64080c3a22566e3a23d2544e187641075350519a281f80";
constexpr char EXACT_PRODUCT_ID[] =
    "271facfdbbb97c777d1a661cf73f5a1eaa25853d42f9e520cab9c785af272f2f";
constexpr char PARENT_VECTOR_DOMAIN[] =
    "nextengine.nonlocal.r63zm.binary128-vector.v1";
constexpr char VECTOR_DOMAIN[] =
    "nextengine.nonlocal.r63zp.quad-vector.v1";
constexpr char CANDIDATE_WORK_DOMAIN[] =
    "nextengine.nonlocal.r63zp.candidate-work.v1";
constexpr char BODY_DOMAIN[] =
    "nextengine.nonlocal.r63zp.product-body.v1";
constexpr char CANDIDATE_EVENT_DOMAIN[] =
    "nextengine.nonlocal.r63zp.candidate-event.v1";
constexpr char CANDIDATE_TRACE_DOMAIN[] =
    "nextengine.nonlocal.r63zp.candidate-trace.v1";
constexpr char CANDIDATE_RESULT_DOMAIN[] =
    "nextengine.nonlocal.r63zp.candidate-result.v1";
constexpr char EXACT_VECTOR_DOMAIN[] =
    "nextengine.nonlocal.r63zp.exact-dyadic-vector.v1";
constexpr char CHECKER_WORK_DOMAIN[] =
    "nextengine.nonlocal.r63zp.checker-work.v1";
constexpr char CHECKER_EVENT_DOMAIN[] =
    "nextengine.nonlocal.r63zp.checker-event.v1";
constexpr char CHECKER_TRACE_DOMAIN[] =
    "nextengine.nonlocal.r63zp.checker-trace.v1";
constexpr char CHECKER_RESULT_DOMAIN[] =
    "nextengine.nonlocal.r63zp.checker-result.v1";

static_assert(sizeof(Quad) == 16U);
static_assert(__FLT128_MANT_DIG__ == 113);
static_assert(__FLT128_MAX_EXP__ == 16384);

void put32(std::uint8_t* out, std::uint32_t value) noexcept {
    for (std::size_t i = 0U; i < 4U; ++i)
        out[i] = static_cast<std::uint8_t>(value >> (24U - 8U * i));
}

void put64(std::uint8_t* out, std::uint64_t value) noexcept {
    for (std::size_t i = 0U; i < 8U; ++i)
        out[i] = static_cast<std::uint8_t>(value >> (56U - 8U * i));
}

std::uint32_t take32(const std::uint8_t* bytes) noexcept {
    std::uint32_t result = 0U;
    for (std::size_t i = 0U; i < 4U; ++i)
        result = (result << 8U) | bytes[i];
    return result;
}

std::uint64_t take64(const std::uint8_t* bytes) noexcept {
    std::uint64_t result = 0U;
    for (std::size_t i = 0U; i < 8U; ++i)
        result = (result << 8U) | bytes[i];
    return result;
}

Quad take_little_quad(const std::uint8_t* bytes) noexcept {
    Quad value = ZERO;
    std::memcpy(&value, bytes, sizeof(value));
    return value;
}

Quad take_big_quad(const std::uint8_t* bytes) noexcept {
    std::array<std::uint8_t, 16U> little{};
    for (std::size_t i = 0U; i < little.size(); ++i)
        little[i] = bytes[little.size() - 1U - i];
    return take_little_quad(little.data());
}

double take_little_double(const std::uint8_t* bytes) noexcept {
    double value = 0.0;
    std::memcpy(&value, bytes, sizeof(value));
    return value;
}

std::uint64_t take_little_u64(const std::uint8_t* bytes) noexcept {
    std::uint64_t value = 0U;
    std::memcpy(&value, bytes, sizeof(value));
    return value;
}

std::array<std::uint8_t, 16U> encode_quad(Quad value) noexcept {
    std::array<std::uint8_t, 16U> little{};
    std::array<std::uint8_t, 16U> big{};
    std::memcpy(little.data(), &value, little.size());
    for (std::size_t i = 0U; i < big.size(); ++i)
        big[i] = little[big.size() - 1U - i];
    return big;
}

Digest take_digest(const std::uint8_t* bytes) noexcept {
    Digest result{};
    std::memcpy(result.data(), bytes, result.size());
    return result;
}

void put_digest(std::uint8_t* bytes, const Digest& value) noexcept {
    std::memcpy(bytes, value.data(), value.size());
}

bool same_digest(const Digest& left, const Digest& right) noexcept {
    bool same = true;
    for (std::size_t i = 0U; i < left.size(); ++i)
        same = (left[i] == right[i]) && same;
    return same;
}

std::uint8_t from_hex(char value) noexcept {
    if (value >= '0' && value <= '9')
        return static_cast<std::uint8_t>(value - '0');
    if (value >= 'a' && value <= 'f')
        return static_cast<std::uint8_t>(value - 'a' + 10);
    return 0xffU;
}

bool matches_hex(const Digest& value, const char* expected) noexcept {
    bool matches = expected[64U] == '\0';
    for (std::size_t i = 0U; i < value.size(); ++i) {
        const std::uint8_t high = from_hex(expected[2U * i]);
        const std::uint8_t low = from_hex(expected[2U * i + 1U]);
        const bool component = high < 16U && low < 16U
            && value[i] == static_cast<std::uint8_t>((high << 4U) | low);
        matches = component && matches;
    }
    return matches;
}

struct HashFields final {
    r63zm::Sha256 sha;
    std::uint64_t consumed = 0U;
    void bytes(const std::uint8_t* data, std::size_t count) noexcept {
        sha.update(std::span<const std::uint8_t>(data, count));
        consumed += count;
    }
    void one(std::uint8_t value) noexcept {
        sha.update_byte(value);
        ++consumed;
    }
    void raw64(std::uint64_t value) noexcept {
        std::array<std::uint8_t, 8U> encoded{};
        put64(encoded.data(), value);
        bytes(encoded.data(), encoded.size());
    }
    void begin(std::uint8_t tag, std::uint64_t length) noexcept {
        one(tag);
        raw64(length);
    }
    void raw_field(const std::uint8_t* value, std::size_t count) noexcept {
        begin(1U, count);
        bytes(value, count);
    }
    void u32(std::uint32_t value) noexcept {
        std::array<std::uint8_t, 4U> encoded{};
        put32(encoded.data(), value);
        begin(2U, encoded.size());
        bytes(encoded.data(), encoded.size());
    }
    void u64(std::uint64_t value) noexcept {
        begin(3U, 8U);
        raw64(value);
    }
    void quad(Quad value) noexcept {
        const auto encoded = encode_quad(value);
        begin(4U, encoded.size());
        bytes(encoded.data(), encoded.size());
    }
    void digest(const Digest& value) noexcept {
        begin(5U, value.size());
        bytes(value.data(), value.size());
    }
    template <std::size_t Count>
    void u64_array(const std::array<std::uint64_t, Count>& values,
        std::size_t used = Count) noexcept {
        begin(6U, used * 8U);
        for (std::size_t i = 0U; i < used; ++i) raw64(values[i]);
    }
    template <std::size_t Count>
    void digest_array(const std::array<Digest, Count>& values,
        std::size_t used = Count) noexcept {
        begin(7U, used * 32U);
        for (std::size_t i = 0U; i < used; ++i)
            bytes(values[i].data(), values[i].size());
    }
    template <std::size_t Count>
    void quad_array(const std::array<Quad, Count>& values) noexcept {
        begin(8U, Count * sizeof(Quad));
        for (Quad value : values) {
            const auto encoded = encode_quad(value);
            bytes(encoded.data(), encoded.size());
        }
    }
    Digest finish() noexcept { return sha.finish(); }
};

template <std::size_t Size>
struct FileImage final {
    std::array<std::uint8_t, Size> data{};
    Digest root{};
    bool exact = false;
};

template <std::size_t Size>
void load_file(const char* path, FileImage<Size>& file,
    CheckerWork& work) noexcept {
    ++work[r63zp_format::KwInputOpenAttempts];
    const int fd = open(path, O_RDONLY);
    if (fd < 0) return;
    std::size_t offset = 0U;
    bool reads_ok = true;
    while (offset < file.data.size()) {
        ++work[r63zp_format::KwInputReadCalls];
        const ssize_t amount = read(fd, file.data.data() + offset,
            file.data.size() - offset);
        if (amount <= 0) {
            reads_ok = false;
            break;
        }
        offset += static_cast<std::size_t>(amount);
        work[r63zp_format::KwInputReadBytes]
            += static_cast<std::uint64_t>(amount);
    }
    std::uint8_t extra = 0U;
    ++work[r63zp_format::KwInputReadCalls];
    ++work[r63zp_format::KwInputTrailingChecks];
    const ssize_t extra_amount = read(fd, &extra, 1U);
    if (extra_amount > 0)
        work[r63zp_format::KwInputReadBytes]
            += static_cast<std::uint64_t>(extra_amount);
    ++work[r63zp_format::KwInputCloseCalls];
    const bool closed = close(fd) == 0;
    file.exact = reads_ok && offset == file.data.size()
        && extra_amount == 0 && closed;
    if (file.exact) {
        ++work[r63zp_format::KwInputHashCalls];
        work[r63zp_format::KwInputHashBytes] += file.data.size();
        file.root = r63zm::sha256(file.data);
    }
}

bool finite_normal_or_zero(Quad value) noexcept {
    return finiteq(value) != 0
        && (value == ZERO || fabsq(value) >= ldexpq(ONE, -16382));
}

Quad upward_add(Quad left, Quad right) noexcept {
    if (left == ZERO && right == ZERO) return ZERO;
    return nextafterq(left + right, HUGE_VALQ);
}

Quad upward_multiply(Quad left, Quad right) noexcept {
    if (left == ZERO || right == ZERO) return ZERO;
    return nextafterq(left * right, HUGE_VALQ);
}

Quad upward_divide(Quad left, Quad right) noexcept {
    if (left == ZERO) return ZERO;
    return nextafterq(left / right, HUGE_VALQ);
}

Quad directed(Quad left, Quad right, int rounding, char operation) noexcept {
    if (std::fesetround(rounding) != 0) return nanq("");
    volatile Quad answer = operation == '+' ? left + right
        : operation == '-' ? left - right : left / right;
    if (std::fesetround(FE_TONEAREST) != 0) return nanq("");
    return answer;
}

struct ErrorFree final {
    Quad high = ZERO;
    Quad low = ZERO;
    bool finite = false;
    bool normal = false;
};

ErrorFree two_sum(Quad a, Quad b) noexcept {
    ErrorFree out;
    out.high = a + b;
    const Quad b_virtual = out.high - a;
    out.low = (a - (out.high - b_virtual)) + (b - b_virtual);
    out.finite = finiteq(out.high) != 0 && finiteq(out.low) != 0;
    out.normal = finite_normal_or_zero(a) && finite_normal_or_zero(b)
        && finite_normal_or_zero(out.high) && finite_normal_or_zero(b_virtual)
        && finite_normal_or_zero(out.low);
    return out;
}

ErrorFree two_product(Quad a, Quad b) noexcept {
    ErrorFree out;
    out.high = a * b;
    out.low = fmaq(a, b, -out.high);
    const bool represented = a == ZERO || b == ZERO || out.high != ZERO
        || out.low != ZERO;
    out.finite = finiteq(out.high) != 0 && finiteq(out.low) != 0;
    out.normal = represented && finite_normal_or_zero(a)
        && finite_normal_or_zero(b) && finite_normal_or_zero(out.high)
        && finite_normal_or_zero(out.low);
    return out;
}

struct CheckedDot final {
    Quad center = ZERO;
    Quad radius = ZERO;
    bool exact = false;
    bool normal = false;
};

template <class Left, class Right>
CheckedDot compensated_dot(std::size_t count, const Left& left,
    const Right& right) noexcept {
    CheckedDot out;
    if (count == 0U) return out;
    const ErrorFree first = two_product(left(0U), right(0U));
    Quad high = first.high;
    Quad correction = first.low;
    Quad magnitude = upward_multiply(fabsq(left(0U)), fabsq(right(0U)));
    bool valid = first.finite;
    bool normal = first.normal;
    for (std::size_t i = 1U; i < count; ++i) {
        const ErrorFree product = two_product(left(i), right(i));
        const ErrorFree sum = two_sum(high, product.high);
        const Quad local = sum.low + product.low;
        correction += local;
        high = sum.high;
        valid = product.finite && sum.finite && valid;
        normal = product.normal && sum.normal && normal
            && finite_normal_or_zero(local)
            && finite_normal_or_zero(correction);
        magnitude = upward_add(magnitude,
            upward_multiply(fabsq(left(i)), fabsq(right(i))));
    }
    out.center = high + correction;
    normal = finite_normal_or_zero(out.center) && normal;
    const Quad u = ldexpq(ONE, -112);
    const Quad nu = static_cast<Quad>(count) * u;
    const Quad gamma = upward_divide(nu, ONE - nu);
    const Quad numerator = upward_add(upward_multiply(u, fabsq(out.center)),
        upward_multiply(upward_multiply(gamma, gamma), magnitude));
    out.radius = upward_divide(numerator, ONE - u);
    out.exact = valid && finiteq(out.center) != 0
        && finiteq(out.radius) != 0 && out.radius >= ZERO;
    out.normal = normal;
    return out;
}

struct StableSum final {
    Quad high = ZERO;
    Quad low = ZERO;
    void consume(Quad value) noexcept {
        const Quad next = high + value;
        low += fabsq(high) >= fabsq(value)
            ? (high - next) + value : (value - next) + high;
        high = next;
    }
    Quad result() const noexcept { return high + low; }
};

struct FactorResult final {
    std::array<Quad, N> value{};
    std::uint64_t terms = 0U;
    std::uint64_t divisions = 0U;
    bool valid = false;
};

FactorResult apply_factor(const std::array<Quad, FACTOR_ENTRIES>& upper,
    const std::array<std::uint64_t, N>& permutation,
    const std::array<Quad, N>& source, Quad inverse,
    CheckerWork& work) noexcept {
    FactorResult out;
    std::array<Quad, N> permuted{};
    std::array<Quad, N> forward{};
    std::array<Quad, N> backward{};
    bool valid = inverse > ZERO && finiteq(inverse) != 0;
    for (std::size_t i = 0U; i < N; ++i) {
        const bool index_ok = permutation[i] < N;
        valid = index_ok && valid;
        if (index_ok) permuted[i] = source[permutation[i]] * inverse;
    }
    for (std::size_t row = 0U; row < N; ++row) {
        StableSum sum;
        for (std::size_t column = 0U; column < row; ++column) {
            sum.consume(upper[column * N + row] * forward[column]);
            ++out.terms;
        }
        const Quad diagonal = upper[row * N + row];
        const bool row_ok = diagonal != ZERO && finiteq(diagonal) != 0;
        valid = row_ok && valid;
        if (row_ok) {
            forward[row] = (permuted[row] - sum.result()) / diagonal;
            ++out.divisions;
            valid = finiteq(forward[row]) != 0 && valid;
        }
    }
    for (std::size_t remaining = N; remaining > 0U;) {
        const std::size_t row = --remaining;
        StableSum sum;
        for (std::size_t column = row + 1U; column < N; ++column) {
            sum.consume(upper[row * N + column] * backward[column]);
            ++out.terms;
        }
        const Quad diagonal = upper[row * N + row];
        const bool row_ok = diagonal != ZERO && finiteq(diagonal) != 0;
        valid = row_ok && valid;
        if (row_ok) {
            backward[row] = (forward[row] - sum.result()) / diagonal;
            ++out.divisions;
            valid = finiteq(backward[row]) != 0 && valid;
        }
    }
    for (std::size_t i = 0U; i < N; ++i)
        if (permutation[i] < N) out.value[permutation[i]] = backward[i];
    out.valid = valid && out.terms == 10302U && out.divisions == 204U;
    ++work[r63zp_format::KwFactorSolves];
    work[r63zp_format::KwFactorTerms] += out.terms;
    work[r63zp_format::KwFactorDivisions] += out.divisions;
    return out;
}

struct Magnitude final {
    std::array<std::uint32_t, LIMBS> word{};
    std::size_t used = 0U;
    bool negative = false;
    bool valid = true;
};

void trim(Magnitude& value) noexcept {
    while (value.used > 0U && value.word[value.used - 1U] == 0U)
        --value.used;
    if (value.used == 0U) value.negative = false;
}

bool is_zero(const Magnitude& value) noexcept { return value.used == 0U; }

int magnitude_order(const Magnitude& left, const Magnitude& right) noexcept {
    if (left.used != right.used) return left.used < right.used ? -1 : 1;
    for (std::size_t remaining = left.used; remaining > 0U;) {
        const std::size_t i = --remaining;
        if (left.word[i] != right.word[i])
            return left.word[i] < right.word[i] ? -1 : 1;
    }
    return 0;
}

Magnitude add_unsigned(const Magnitude& left, const Magnitude& right,
    CheckerWork& work) noexcept {
    Magnitude out;
    const std::size_t width = std::max(left.used, right.used);
    ++work[r63zp_format::KwBigintCapacityPredicates];
    if (width >= LIMBS) {
        out.valid = false;
        return out;
    }
    std::uint64_t carry = 0U;
    for (std::size_t i = 0U; i < width; ++i) {
        const std::uint64_t a = i < left.used ? left.word[i] : 0U;
        const std::uint64_t b = i < right.used ? right.word[i] : 0U;
        const std::uint64_t sum = a + b + carry;
        out.word[i] = static_cast<std::uint32_t>(sum);
        carry = sum >> 32U;
    }
    out.used = width;
    if (carry != 0U) out.word[out.used++] = static_cast<std::uint32_t>(carry);
    trim(out);
    return out;
}

Magnitude subtract_unsigned(const Magnitude& larger,
    const Magnitude& smaller) noexcept {
    Magnitude out;
    out.used = larger.used;
    std::uint64_t borrow = 0U;
    for (std::size_t i = 0U; i < larger.used; ++i) {
        const std::uint64_t a = larger.word[i];
        const std::uint64_t b = (i < smaller.used ? smaller.word[i] : 0U)
            + borrow;
        out.word[i] = static_cast<std::uint32_t>(a - b);
        borrow = a < b ? 1U : 0U;
    }
    trim(out);
    return out;
}

Magnitude signed_add(const Magnitude& left, const Magnitude& right,
    CheckerWork& work) noexcept {
    if (!left.valid || !right.valid) {
        Magnitude out;
        out.valid = false;
        return out;
    }
    if (is_zero(left)) return right;
    if (is_zero(right)) return left;
    if (left.negative == right.negative) {
        Magnitude out = add_unsigned(left, right, work);
        out.negative = left.negative && !is_zero(out);
        return out;
    }
    const int order = magnitude_order(left, right);
    if (order == 0) return {};
    const Magnitude& larger = order > 0 ? left : right;
    const Magnitude& smaller = order > 0 ? right : left;
    Magnitude out = subtract_unsigned(larger, smaller);
    out.negative = larger.negative && !is_zero(out);
    return out;
}

Magnitude multiply_unsigned(const Magnitude& left, const Magnitude& right,
    CheckerWork& work) noexcept {
    Magnitude out;
    if (!left.valid || !right.valid) {
        out.valid = false;
        return out;
    }
    ++work[r63zp_format::KwBigintCapacityPredicates];
    if (left.used + right.used > LIMBS) {
        out.valid = false;
        return out;
    }
    out.used = left.used + right.used;
    for (std::size_t i = 0U; i < left.used; ++i) {
        std::uint64_t carry = 0U;
        for (std::size_t j = 0U; j < right.used; ++j) {
            const std::size_t target = i + j;
            const std::uint64_t product =
                static_cast<std::uint64_t>(left.word[i]) * right.word[j]
                + out.word[target] + carry;
            out.word[target] = static_cast<std::uint32_t>(product);
            carry = product >> 32U;
        }
        std::size_t target = i + right.used;
        while (carry != 0U && target < LIMBS) {
            const std::uint64_t sum = out.word[target] + carry;
            out.word[target] = static_cast<std::uint32_t>(sum);
            carry = sum >> 32U;
            ++target;
        }
        if (carry != 0U) out.valid = false;
    }
    out.negative = left.negative != right.negative;
    trim(out);
    return out;
}

bool shift_left(Magnitude& value, std::uint64_t bits,
    CheckerWork& work) noexcept {
    if (is_zero(value) || bits == 0U) return value.valid;
    const std::size_t whole = static_cast<std::size_t>(bits / 32U);
    const unsigned part = static_cast<unsigned>(bits % 32U);
    const std::size_t required = value.used + whole + (part == 0U ? 0U : 1U);
    ++work[r63zp_format::KwBigintCapacityPredicates];
    if (required > LIMBS) {
        value.valid = false;
        return false;
    }
    Magnitude shifted;
    shifted.negative = value.negative;
    shifted.valid = value.valid;
    std::uint64_t carry = 0U;
    for (std::size_t i = 0U; i < value.used; ++i) {
        const std::uint64_t combined =
            (static_cast<std::uint64_t>(value.word[i]) << part) | carry;
        shifted.word[i + whole] = static_cast<std::uint32_t>(combined);
        carry = combined >> 32U;
    }
    shifted.used = value.used + whole;
    if (part != 0U && carry != 0U)
        shifted.word[shifted.used++] = static_cast<std::uint32_t>(carry);
    trim(shifted);
    value = shifted;
    ++work[r63zp_format::KwAlignmentShifts];
    work[r63zp_format::KwAlignmentBits] += bits;
    return value.valid;
}

std::uint64_t trailing_zeros(const Magnitude& value) noexcept {
    std::uint64_t count = 0U;
    for (std::size_t i = 0U; i < value.used; ++i) {
        if (value.word[i] == 0U) count += 32U;
        else {
            count += std::countr_zero(value.word[i]);
            break;
        }
    }
    return count;
}

void shift_right_exact(Magnitude& value, std::uint64_t bits) noexcept {
    if (is_zero(value) || bits == 0U) return;
    const std::size_t whole = static_cast<std::size_t>(bits / 32U);
    const unsigned part = static_cast<unsigned>(bits % 32U);
    for (std::size_t i = 0U; i + whole < value.used; ++i)
        value.word[i] = value.word[i + whole];
    for (std::size_t i = value.used - whole; i < value.used; ++i)
        value.word[i] = 0U;
    value.used -= whole;
    if (part != 0U) {
        std::uint32_t carry = 0U;
        for (std::size_t remaining = value.used; remaining > 0U;) {
            const std::size_t i = --remaining;
            const std::uint32_t next = static_cast<std::uint32_t>(
                value.word[i] << (32U - part));
            value.word[i] = (value.word[i] >> part) | carry;
            carry = next;
        }
    }
    trim(value);
}

struct Dyadic final {
    Magnitude numerator;
    std::int64_t exponent = 0;
};

void normalize(Dyadic& value) noexcept {
    if (is_zero(value.numerator)) {
        value.exponent = 0;
        return;
    }
    const std::uint64_t trailing = trailing_zeros(value.numerator);
    if (trailing != 0U) {
        shift_right_exact(value.numerator, trailing);
        value.exponent += static_cast<std::int64_t>(trailing);
    }
}

Dyadic decode_dyadic(Quad value, CheckerWork& work) noexcept {
    ++work[r63zp_format::KwDyadicDecodes];
    const auto bytes = encode_quad(value);
    const bool negative = (bytes[0U] & 0x80U) != 0U;
    const std::uint16_t raw_exponent = static_cast<std::uint16_t>(
        (static_cast<std::uint16_t>(bytes[0U] & 0x7fU) << 8U) | bytes[1U]);
    Dyadic out;
    ++work[r63zp_format::KwBigintCapacityPredicates];
    if (raw_exponent == 0x7fffU) {
        out.numerator.valid = false;
        return out;
    }
    for (std::size_t i = 2U; i < bytes.size(); ++i) {
        if (!shift_left(out.numerator, 8U, work)) return out;
        Magnitude byte;
        if (bytes[i] != 0U) {
            byte.word[0U] = bytes[i];
            byte.used = 1U;
        }
        out.numerator = signed_add(out.numerator, byte, work);
    }
    out.exponent = 1 - 16383 - 112;
    if (raw_exponent != 0U) {
        Magnitude hidden;
        hidden.word[0U] = 1U;
        hidden.used = 1U;
        if (!shift_left(hidden, 112U, work)) {
            out.numerator.valid = false;
            return out;
        }
        out.numerator = signed_add(out.numerator, hidden, work);
        out.exponent = static_cast<std::int64_t>(raw_exponent) - 16383 - 112;
    }
    out.numerator.negative = negative && !is_zero(out.numerator);
    normalize(out);
    return out;
}

Dyadic exact_add(Dyadic left, Dyadic right, CheckerWork& work) noexcept {
    ++work[r63zp_format::KwExactAdditions];
    if (is_zero(left.numerator)) return right;
    if (is_zero(right.numerator)) return left;
    const std::int64_t exponent = std::min(left.exponent, right.exponent);
    if (left.exponent != exponent)
        shift_left(left.numerator,
            static_cast<std::uint64_t>(left.exponent - exponent), work);
    if (right.exponent != exponent)
        shift_left(right.numerator,
            static_cast<std::uint64_t>(right.exponent - exponent), work);
    Dyadic out{signed_add(left.numerator, right.numerator, work), exponent};
    normalize(out);
    return out;
}

Dyadic exact_multiply(const Dyadic& left, const Dyadic& right,
    CheckerWork& work) noexcept {
    ++work[r63zp_format::KwExactMultiplies];
    Dyadic out{multiply_unsigned(left.numerator, right.numerator, work),
        left.exponent + right.exponent};
    normalize(out);
    return out;
}

int compare_exact(const Dyadic& left, const Dyadic& right,
    CheckerWork& work) noexcept {
    Dyadic negated = right;
    if (!is_zero(negated.numerator))
        negated.numerator.negative = !negated.numerator.negative;
    const Dyadic difference = exact_add(left, negated, work);
    if (is_zero(difference.numerator)) return 0;
    return difference.numerator.negative ? -1 : 1;
}

std::size_t magnitude_bytes(const Magnitude& value) noexcept {
    if (is_zero(value)) return 0U;
    const std::uint32_t high = value.word[value.used - 1U];
    return (value.used - 1U) * 4U
        + static_cast<std::size_t>((32U - std::countl_zero(high) + 7U) / 8U);
}

void hash_exact_magnitude(r63zm::Sha256& hash, const Magnitude& value) noexcept {
    bool leading = true;
    for (std::size_t remaining = value.used; remaining > 0U;) {
        const std::uint32_t word = value.word[--remaining];
        for (std::size_t byte = 0U; byte < 4U; ++byte) {
            const std::uint8_t next = static_cast<std::uint8_t>(
                word >> (24U - 8U * byte));
            if (leading && next == 0U) continue;
            leading = false;
            hash.update_byte(next);
        }
    }
}

Digest exact_vector_root(const std::array<Dyadic, N>& values,
    CheckerWork& work) noexcept {
    r63zm::Sha256 hash;
    std::array<std::uint8_t, 8U> encoded{};
    put64(encoded.data(), sizeof(EXACT_VECTOR_DOMAIN) - 1U);
    hash.update(encoded);
    hash.update(std::span<const std::uint8_t>(
        reinterpret_cast<const std::uint8_t*>(EXACT_VECTOR_DOMAIN),
        sizeof(EXACT_VECTOR_DOMAIN) - 1U));
    put64(encoded.data(), N);
    hash.update(encoded);
    for (const Dyadic& value : values) {
        hash.update_byte(value.numerator.negative ? 1U : 0U);
        put64(encoded.data(), static_cast<std::uint64_t>(value.exponent));
        hash.update(encoded);
        put64(encoded.data(), magnitude_bytes(value.numerator));
        hash.update(encoded);
        hash_exact_magnitude(hash, value.numerator);
        ++work[r63zp_format::KwExactVectorHashFields];
    }
    return hash.finish();
}

struct CandidateImage final {
    std::uint32_t route = 0U;
    std::uint32_t flags = 0U;
    std::array<Digest, 3U> parents{};
    std::array<Digest, 2U> roles{};
    std::array<Digest, 3U> roots{};
    std::array<Quad, 5U> scalars{};
    CandidateWork work{};
    Digest work_root{};
    std::uint64_t event_count = 0U;
    std::array<Digest, r63zp_format::event_slots> events{};
    Digest trace{};
    std::array<Quad, N> p0{};
    std::array<Quad, N> q0{};
    std::array<Quad, N> bounds{};
    Digest body{};
    Digest result{};
    bool header_valid = false;
};

CandidateImage parse_candidate(
    const FileImage<r63zp_format::candidate_bytes>& source,
    CheckerWork& work) noexcept {
    CandidateImage out;
    if (!source.exact) return out;
    bool header = true;
    auto predicate = [&](bool value) noexcept {
        ++work[r63zp_format::KwHeaderPredicatesExecuted];
        header = value && header;
    };
    predicate(std::memcmp(source.data.data(),
        r63zp_format::candidate_magic.data(), 8U) == 0);
    predicate(take32(source.data.data() + r63zp_format::c_version)
        == r63zp_format::version);
    predicate(take32(source.data.data() + r63zp_format::c_total)
        == r63zp_format::candidate_bytes);
    out.route = take32(source.data.data() + r63zp_format::c_route);
    predicate(out.route >= 1U && out.route <= 7U);
    out.flags = take32(source.data.data() + r63zp_format::c_flags);
    predicate((out.flags & ~0x3fU) == 0U);
    predicate(take64(source.data.data() + r63zp_format::c_dimension) == N);
    predicate(take64(source.data.data() + r63zp_format::c_work_count)
        == r63zp_format::candidate_work_fields);
    out.event_count = take64(source.data.data()
        + r63zp_format::c_event_count);
    predicate(out.event_count <= r63zp_format::event_slots);
    out.header_valid = header;
    if (!header) return out;

    for (std::size_t i = 0U; i < out.parents.size(); ++i)
        out.parents[i] = take_digest(source.data.data()
            + r63zp_format::c_parent_roots + i * 32U);
    for (std::size_t i = 0U; i < out.roles.size(); ++i)
        out.roles[i] = take_digest(source.data.data()
            + r63zp_format::c_role_roots + i * 32U);
    for (std::size_t i = 0U; i < out.roots.size(); ++i)
        out.roots[i] = take_digest(source.data.data()
            + r63zp_format::c_semantic_roots + i * 32U);
    for (std::size_t i = 0U; i < out.scalars.size(); ++i)
        out.scalars[i] = take_big_quad(source.data.data()
            + r63zp_format::c_scalars + i * sizeof(Quad));
    for (std::size_t i = 0U; i < out.work.size(); ++i)
        out.work[i] = take64(source.data.data() + r63zp_format::c_work
            + i * 8U);
    out.work_root = take_digest(source.data.data()
        + r63zp_format::c_work_root);
    for (std::size_t i = 0U; i < out.events.size(); ++i)
        out.events[i] = take_digest(source.data.data()
            + r63zp_format::c_events + i * 32U);
    out.trace = take_digest(source.data.data() + r63zp_format::c_trace_root);
    for (std::size_t i = 0U; i < N; ++i) {
        out.p0[i] = take_big_quad(source.data.data() + r63zp_format::c_p0
            + i * sizeof(Quad));
        out.q0[i] = take_big_quad(source.data.data() + r63zp_format::c_q0
            + i * sizeof(Quad));
        out.bounds[i] = take_big_quad(source.data.data()
            + r63zp_format::c_bounds + i * sizeof(Quad));
    }
    out.body = take_digest(source.data.data()
        + r63zp_format::c_product_body_root);
    out.result = take_digest(source.data.data()
        + r63zp_format::c_result_root);
    work[r63zp_format::KwCandidateBytesDecoded]
        = r63zp_format::candidate_bytes;
    work[r63zp_format::KwCandidateQuadDecodes] = 5U + 3U * N;
    work[r63zp_format::KwCandidateU64Decodes]
        = 3U + r63zp_format::candidate_work_fields;
    work[r63zp_format::KwCandidateDigestDecodes] = 20U;
    work[r63zp_format::KwCandidatePaddingBytesChecked] = 0U;
    return out;
}

Digest parent_vector(const std::array<Quad, N>& values,
    std::uint64_t& bytes) noexcept {
    HashFields fields;
    fields.begin(1U, sizeof(PARENT_VECTOR_DOMAIN) - 1U);
    fields.bytes(reinterpret_cast<const std::uint8_t*>(PARENT_VECTOR_DOMAIN),
        sizeof(PARENT_VECTOR_DOMAIN) - 1U);
    fields.begin(2U, 8U);
    fields.raw64(N);
    fields.begin(3U, N * sizeof(Quad));
    for (Quad value : values) {
        const auto encoded = encode_quad(value);
        fields.bytes(encoded.data(), encoded.size());
    }
    bytes = fields.consumed;
    return fields.finish();
}

Digest candidate_vector(const std::array<Quad, N>& values,
    std::uint64_t& bytes) noexcept {
    HashFields fields;
    fields.raw_field(reinterpret_cast<const std::uint8_t*>(VECTOR_DOMAIN),
        sizeof(VECTOR_DOMAIN) - 1U);
    fields.u64(N);
    fields.quad_array(values);
    bytes = fields.consumed;
    return fields.finish();
}

Digest candidate_work_root(std::uint32_t route, const CandidateWork& work,
    std::uint64_t& bytes) noexcept {
    HashFields fields;
    fields.raw_field(reinterpret_cast<const std::uint8_t*>(
        CANDIDATE_WORK_DOMAIN), sizeof(CANDIDATE_WORK_DOMAIN) - 1U);
    fields.u32(route);
    fields.u64_array(work);
    bytes = fields.consumed;
    return fields.finish();
}

Digest candidate_body(const CandidateImage& image,
    const CandidateWork& work,
    const std::array<Digest, r63zp_format::event_slots>& events,
    std::uint64_t& bytes) noexcept {
    HashFields fields;
    fields.raw_field(reinterpret_cast<const std::uint8_t*>(BODY_DOMAIN),
        sizeof(BODY_DOMAIN) - 1U);
    fields.digest_array(image.parents);
    fields.digest_array(image.roles);
    fields.digest_array(image.roots);
    for (Quad scalar : image.scalars) fields.quad(scalar);
    fields.u64(N);
    fields.u64_array(work, 40U);
    fields.digest_array(events, 4U);
    fields.quad_array(image.p0);
    fields.quad_array(image.q0);
    fields.quad_array(image.bounds);
    bytes = fields.consumed;
    return fields.finish();
}

Digest candidate_event_root(std::uint32_t ordinal, std::uint32_t stage_route,
    const CandidateImage& image, const Digest& derived_x0,
    const Digest& work_root, std::uint64_t& bytes) noexcept {
    HashFields fields;
    fields.raw_field(reinterpret_cast<const std::uint8_t*>(
        CANDIDATE_EVENT_DOMAIN), sizeof(CANDIDATE_EVENT_DOMAIN) - 1U);
    fields.u32(ordinal);
    fields.u32(stage_route);
    switch (ordinal) {
    case 1U: fields.digest_array(image.parents); break;
    case 2U:
        fields.digest(image.roles[0U]);
        fields.digest(derived_x0);
        break;
    case 3U: fields.digest_array(image.roles); break;
    case 4U:
        fields.digest(image.roots[0U]);
        fields.quad(image.scalars[0U]);
        fields.quad(image.scalars[1U]);
        break;
    case 5U: fields.digest(image.body); break;
    case 6U:
        fields.digest(image.roots[1U]);
        fields.digest(image.roots[2U]);
        fields.quad(image.scalars[2U]);
        fields.quad(image.scalars[3U]);
        fields.quad(image.scalars[4U]);
        break;
    case 7U:
        fields.digest(image.body); // replaced below by replay x1 when checked
        fields.u64(0U);
        break;
    case 8U:
        fields.digest(image.body);
        fields.digest(work_root);
        break;
    default: break;
    }
    bytes = fields.consumed;
    return fields.finish();
}

Digest candidate_event7_root(std::uint32_t route, const Digest& x1,
    std::uint64_t matches, std::uint64_t& bytes) noexcept {
    HashFields fields;
    fields.raw_field(reinterpret_cast<const std::uint8_t*>(
        CANDIDATE_EVENT_DOMAIN), sizeof(CANDIDATE_EVENT_DOMAIN) - 1U);
    fields.u32(7U);
    fields.u32(route);
    fields.digest(x1);
    fields.u64(matches);
    bytes = fields.consumed;
    return fields.finish();
}

Digest candidate_trace_root(std::uint64_t count,
    const std::array<Digest, r63zp_format::event_slots>& events,
    std::uint64_t& bytes) noexcept {
    HashFields fields;
    fields.raw_field(reinterpret_cast<const std::uint8_t*>(
        CANDIDATE_TRACE_DOMAIN), sizeof(CANDIDATE_TRACE_DOMAIN) - 1U);
    fields.u64(count);
    fields.digest_array(events);
    bytes = fields.consumed;
    return fields.finish();
}

Digest candidate_result_root(const CandidateImage& image,
    const Digest& work_root, const Digest& trace, std::uint64_t update_matches,
    std::uint64_t& bytes) noexcept {
    HashFields fields;
    fields.raw_field(reinterpret_cast<const std::uint8_t*>(
        CANDIDATE_RESULT_DOMAIN), sizeof(CANDIDATE_RESULT_DOMAIN) - 1U);
    fields.u32(image.route);
    fields.u32(image.flags);
    fields.digest_array(image.parents);
    fields.digest_array(image.roles);
    fields.digest_array(image.roots);
    for (Quad scalar : image.scalars) fields.quad(scalar);
    fields.u64(N);
    fields.digest(work_root);
    fields.u64(image.event_count);
    fields.digest(trace);
    fields.digest(image.body);
    fields.u64(update_matches);
    bytes = fields.consumed;
    return fields.finish();
}

struct ReplayProduct final {
    std::array<Quad, N> center{};
    std::array<Quad, N> radius{};
    bool valid = false;
};

ReplayProduct replay_product(const std::array<Quad, N * M>& tangent,
    Quad sigma, const std::array<Quad, N>& input,
    CheckerWork& work) noexcept {
    ReplayProduct out;
    ++work[r63zp_format::KwRoundedProductCalls];
    std::array<Quad, M> intermediate{};
    std::array<Quad, M> uncertainty{};
    bool valid = true;
    for (std::size_t column = 0U; column < M; ++column) {
        const CheckedDot dot = compensated_dot(N,
            [&](std::size_t row) { return tangent[row * M + column]; },
            [&](std::size_t row) { return input[row]; });
        ++work[r63zp_format::KwRoundedInnerDots];
        work[r63zp_format::KwRoundedInnerTerms] += N;
        intermediate[column] = dot.center;
        uncertainty[column] = dot.radius;
        valid = dot.exact && dot.normal && valid;
    }
    for (std::size_t row = 0U; row < N; ++row) {
        const CheckedDot dot = compensated_dot(M,
            [&](std::size_t column) { return tangent[row * M + column]; },
            [&](std::size_t column) { return intermediate[column]; });
        ++work[r63zp_format::KwRoundedOuterDots];
        work[r63zp_format::KwRoundedOuterTerms] += M;
        Quad propagated = dot.radius;
        for (std::size_t column = 0U; column < M; ++column)
            propagated = upward_add(propagated, upward_multiply(
                fabsq(tangent[row * M + column]), uncertainty[column]));
        work[r63zp_format::KwRoundedPropagationTerms] += M;
        const ErrorFree scaled = two_product(sigma, dot.center);
        ++work[r63zp_format::KwRoundedScaleProducts];
        out.center[row] = scaled.high;
        out.radius[row] = upward_add(fabsq(scaled.low),
            upward_multiply(fabsq(sigma), propagated));
        valid = dot.exact && dot.normal && scaled.finite && scaled.normal
            && finiteq(out.center[row]) != 0
            && finiteq(out.radius[row]) != 0 && out.radius[row] >= ZERO
            && valid;
    }
    out.valid = valid;
    return out;
}

CandidateWork expected_success_work() noexcept {
    CandidateWork work{};
    constexpr std::array<std::uint64_t,
        r63zp_format::candidate_work_fields> expected{{
        1U, 1U, 3U, 6U, 1046961U, 3U, 3U, 3U, 1046961U, 10U,
        32336U, 10404U, 102U, 102U, 42944U, 206U, 102U, 5U,
        2U, 20604U, 408U, 102U, 204U, 1U, 102U, 1U, 315U,
        32130U, 102U, 32130U, 32130U, 102U, 1U, 204U, 4U, 2U,
        3U, 2U, 6U, 14409U, 102U, 102U, 102U, 204U, 102U, 8U, 1321U,
        1U, 335U, 3U, 2936U, 6U, 6152U, 400U, 6152U, 1U, 1U,
        6152U, 1U, 0U, 204U}};
    work = expected;
    return work;
}

struct Replay final {
    CandidateImage expected{};
    CandidateWork expected_work{};
    Digest derived_x0{};
    Digest exact_product_root{};
    std::array<Dyadic, N> exact_product{};
    bool apparatus = false;
    bool semantic = false;
    bool contained = false;
    bool curvature = false;
    bool step = false;
    bool update = false;
    std::uint64_t update_matches = 0U;
};

std::uint64_t exact_root_byte_count(
    const std::array<Dyadic, N>& values) noexcept;

Replay derive_replay(
    const FileImage<r63zp_format::parent_cache_bytes>& cache,
    const FileImage<r63zp_format::parent_artifact_bytes>& parent,
    const FileImage<r63zp_format::parent_audit_bytes>& parent_audit,
    const CandidateImage& candidate, CheckerWork& work,
    std::uint64_t& root_calls, std::uint64_t& root_bytes) noexcept {
    Replay replay;
    replay.expected.parents = {{cache.root, parent.root, parent_audit.root}};
    bool parents = cache.exact && parent.exact && parent_audit.exact;
    const std::array<bool, 3U> identities{{matches_hex(cache.root, CACHE_ID),
        matches_hex(parent.root, ARTIFACT_ID),
        matches_hex(parent_audit.root, AUDIT_ID)}};
    for (bool identity : identities) {
        ++work[r63zp_format::KwParentRootComparisons];
        parents = identity && parents;
    }
    if (!parents) return replay;

    constexpr std::array<std::uint8_t, 8U> parent_magic{{
        'N','E','R','6','3','Z','M','1'}};
    constexpr std::array<std::uint8_t, 8U> audit_magic{{
        'N','E','R','6','3','Q','C','1'}};
    auto parent_predicate = [&](bool value) noexcept {
        ++work[r63zp_format::KwParentHeaderPredicates];
        parents = value && parents;
    };
    parent_predicate(std::memcmp(parent.data.data(), parent_magic.data(), 8U)
        == 0);
    parent_predicate(take64(parent.data.data() + 12U)
        == r63zp_format::parent_artifact_bytes);
    parent_predicate(parent.data[ROLE2] == 1U);
    parent_predicate(parent.data[ROLE2 + 1U] == 1U);
    parent_predicate(parent.data[ROLE2 + 2U] == 2U);
    parent_predicate(take64(parent.data.data() + ROLE2 + 304U) == N);
    parent_predicate(std::memcmp(parent_audit.data.data(), audit_magic.data(),
        8U) == 0);
    parent_predicate(take64(parent_audit.data.data() + 12U)
        == r63zp_format::parent_audit_bytes);
    parent_predicate(parent_audit.data[24U] == 1U);
    parent_predicate(parent_audit.data[25U] == 1U
        && parent_audit.data[26U] == 1U);
    if (!parents) return replay;

    std::array<Quad, N> x0{};
    std::array<Quad, N> rhs{};
    std::array<Quad, N * M> tangent{};
    std::array<Quad, FACTOR_ENTRIES> factor{};
    std::array<std::uint64_t, N> permutation{};
    bool inputs = true;
    for (std::size_t i = 0U; i < N; ++i) {
        x0[i] = take_little_quad(cache.data.data() + CACHE_X0
            + i * sizeof(Quad));
        rhs[i] = take_little_quad(cache.data.data() + CACHE_RHS
            + i * sizeof(Quad));
        permutation[i] = take_little_u64(cache.data.data()
            + CACHE_PERMUTATION + i * sizeof(std::uint64_t));
        work[r63zp_format::KwCacheQuadDecodesPreseal] += 2U;
        ++work[r63zp_format::KwCacheIndexDecodes];
        work[r63zp_format::KwFinitePredicates] += 2U;
        ++work[r63zp_format::KwRangePredicates];
        const bool row = finiteq(x0[i]) != 0 && finiteq(rhs[i]) != 0
            && permutation[i] < N;
        inputs = row && inputs;
    }
    for (std::size_t i = 0U; i < tangent.size(); ++i) {
        tangent[i] = take_little_quad(cache.data.data() + CACHE_TANGENT
            + i * sizeof(Quad));
        ++work[r63zp_format::KwCacheQuadDecodesPreseal];
        ++work[r63zp_format::KwFinitePredicates];
        inputs = finiteq(tangent[i]) != 0 && inputs;
    }
    for (std::size_t i = 0U; i < factor.size(); ++i) {
        factor[i] = static_cast<Quad>(take_little_double(cache.data.data()
            + CACHE_FACTOR + i * sizeof(double)));
        ++work[r63zp_format::KwCacheDoubleDecodes];
        ++work[r63zp_format::KwFinitePredicates];
        inputs = finiteq(factor[i]) != 0 && inputs;
    }
    const Quad sigma = take_little_quad(cache.data.data() + CACHE_SIGMA);
    const Quad inverse = take_little_quad(cache.data.data() + CACHE_INVERSE);
    work[r63zp_format::KwCacheQuadDecodesPreseal] += 2U;
    work[r63zp_format::KwFinitePredicates] += 2U;
    work[r63zp_format::KwRangePredicates] += 2U;
    inputs = finiteq(sigma) != 0 && finiteq(inverse) != 0
        && sigma > ZERO && inverse > ZERO && inputs;
    if (!inputs) return replay;

    const FactorResult initial = apply_factor(factor, permutation, rhs,
        inverse, work);
    bool x0_matches = initial.valid;
    for (std::size_t i = 0U; i < N; ++i) {
        const bool equal = std::memcmp(&initial.value[i], &x0[i],
            sizeof(Quad)) == 0;
        ++work[r63zp_format::KwX0ComponentComparisons];
        ++work[r63zp_format::KwCandidateFixedLoopIterations];
        x0_matches = equal && x0_matches;
    }
    std::uint64_t bytes = 0U;
    replay.derived_x0 = parent_vector(initial.value, bytes);
    ++root_calls;
    root_bytes += bytes;
    replay.expected.roles[0U] = take_digest(parent.data.data()
        + ROLE2_INPUT_ROOT);
    ++work[r63zp_format::KwParentRootComparisons];
    x0_matches = same_digest(replay.derived_x0,
        replay.expected.roles[0U]) && x0_matches;
    if (!x0_matches) return replay;

    std::array<Quad, N> y0{};
    bool role = true;
    for (std::size_t i = 0U; i < N; ++i) {
        y0[i] = take_big_quad(parent.data.data() + ROLE2 + PRODUCT_VALUES
            + i * sizeof(Quad));
        ++work[r63zp_format::KwParentArtifactQuadDecodes];
        ++work[r63zp_format::KwFinitePredicates];
        role = finiteq(y0[i]) != 0 && role;
    }
    replay.expected.roles[1U] = take_digest(parent.data.data()
        + ROLE2_VALUE_ROOT);
    bytes = 0U;
    const Digest y0_root = parent_vector(y0, bytes);
    ++root_calls;
    root_bytes += bytes;
    ++work[r63zp_format::KwParentRootComparisons];
    role = same_digest(y0_root, replay.expected.roles[1U]) && role;
    if (!role) return replay;

    std::array<Quad, N> residual{};
    bool residual_valid = true;
    for (std::size_t i = 0U; i < N; ++i) {
        const CheckedDot dot = compensated_dot(2U,
            [&](std::size_t term) { return term == 0U ? rhs[i] : -y0[i]; },
            [&](std::size_t) { return ONE; });
        ++work[r63zp_format::KwResidualDots];
        work[r63zp_format::KwResidualTerms] += 2U;
        residual[i] = dot.center;
        residual_valid = dot.exact && dot.normal && residual_valid;
    }
    const FactorResult direction = apply_factor(factor, permutation, residual,
        inverse, work);
    replay.expected.p0 = direction.value;
    const CheckedDot rho = compensated_dot(N,
        [&](std::size_t i) { return residual[i]; },
        [&](std::size_t i) { return replay.expected.p0[i]; });
    ++work[r63zp_format::KwRhoDots];
    work[r63zp_format::KwRhoTerms] += N;
    ++work[r63zp_format::KwPositivityPredicates];
    const bool direction_valid = residual_valid && direction.valid
        && rho.exact && rho.normal && rho.center - rho.radius > ZERO;
    if (!direction_valid) return replay;

    replay.expected.scalars[0U] = rho.center;
    replay.expected.scalars[1U] = rho.radius;
    const ReplayProduct candidate_product = replay_product(tangent, sigma,
        replay.expected.p0, work);
    replay.expected.q0 = candidate_product.center;
    replay.expected.bounds = candidate_product.radius;
    if (!candidate_product.valid) return replay;

    bytes = 0U;
    replay.expected.roots[0U] = candidate_vector(replay.expected.p0, bytes);
    ++root_calls;
    root_bytes += bytes;
    bytes = 0U;
    replay.expected.roots[1U] = candidate_vector(replay.expected.q0, bytes);
    ++root_calls;
    root_bytes += bytes;
    bytes = 0U;
    replay.expected.roots[2U] = candidate_vector(replay.expected.bounds,
        bytes);
    ++root_calls;
    root_bytes += bytes;

    const CheckedDot denominator = compensated_dot(N,
        [&](std::size_t i) { return replay.expected.p0[i]; },
        [&](std::size_t i) { return replay.expected.q0[i]; });
    Quad denominator_radius = denominator.radius;
    for (std::size_t i = 0U; i < N; ++i)
        denominator_radius = upward_add(denominator_radius,
            upward_multiply(fabsq(replay.expected.p0[i]),
                replay.expected.bounds[i]));
    replay.expected.scalars[2U] = denominator.center;
    replay.expected.scalars[3U] = denominator_radius;
    const Quad rho_lower = directed(rho.center, rho.radius, FE_DOWNWARD, '-');
    const Quad rho_upper = directed(rho.center, rho.radius, FE_UPWARD, '+');
    const Quad denominator_lower = directed(denominator.center,
        denominator_radius, FE_DOWNWARD, '-');
    const Quad denominator_upper = directed(denominator.center,
        denominator_radius, FE_UPWARD, '+');
    work[r63zp_format::KwDivisionEndpointOperations] += 4U;
    ++work[r63zp_format::KwPositivityPredicates];
    replay.curvature = denominator.exact && denominator.normal
        && denominator_lower > ZERO;
    if (!replay.curvature) return replay;
    const Quad step_lower = directed(rho_lower, denominator_upper,
        FE_DOWNWARD, '/');
    const Quad step_upper = directed(rho_upper, denominator_lower,
        FE_UPWARD, '/');
    replay.expected.scalars[4U] = rho.center / denominator.center;
    work[r63zp_format::KwDivisionEndpointOperations] += 3U;
    work[r63zp_format::KwDivisionPredicates] += 2U;
    replay.step = step_lower <= replay.expected.scalars[4U]
        && replay.expected.scalars[4U] <= step_upper;
    if (!replay.step) return replay;

    std::array<Dyadic, N> p0_exact{};
    for (std::size_t i = 0U; i < N; ++i)
        p0_exact[i] = decode_dyadic(replay.expected.p0[i], work);
    const Dyadic sigma_exact = decode_dyadic(sigma, work);
    std::array<Dyadic, M> intermediate{};
    for (std::size_t column = 0U; column < M; ++column) {
        Dyadic sum;
        for (std::size_t row = 0U; row < N; ++row) {
            const Dyadic coefficient = decode_dyadic(
                tangent[row * M + column], work);
            sum = exact_add(sum, exact_multiply(coefficient, p0_exact[row],
                work), work);
        }
        intermediate[column] = sum;
    }
    bool exact_capacity = sigma_exact.numerator.valid;
    for (std::size_t row = 0U; row < N; ++row) {
        Dyadic sum;
        for (std::size_t column = 0U; column < M; ++column) {
            const Dyadic coefficient = decode_dyadic(
                tangent[row * M + column], work);
            sum = exact_add(sum, exact_multiply(coefficient,
                intermediate[column], work), work);
        }
        replay.exact_product[row] = exact_multiply(sigma_exact, sum, work);
        exact_capacity = replay.exact_product[row].numerator.valid
            && exact_capacity;
    }
    replay.exact_product_root = exact_vector_root(replay.exact_product, work);
    ++root_calls;
    root_bytes += exact_root_byte_count(replay.exact_product);
    if (!matches_hex(replay.exact_product_root, EXACT_PRODUCT_ID)
        || !exact_capacity)
        return replay;

    replay.contained = true;
    for (std::size_t i = 0U; i < N; ++i) {
        const Quad lower = directed(replay.expected.q0[i],
            replay.expected.bounds[i], FE_DOWNWARD, '-');
        const Quad upper = directed(replay.expected.q0[i],
            replay.expected.bounds[i], FE_UPWARD, '+');
        const Dyadic exact_lower = decode_dyadic(lower, work);
        const Dyadic exact_upper = decode_dyadic(upper, work);
        const bool lower_ok = compare_exact(replay.exact_product[i],
            exact_lower, work) >= 0;
        const bool upper_ok = compare_exact(replay.exact_product[i],
            exact_upper, work) <= 0;
        work[r63zp_format::KwIntervalComparisons] += 2U;
        replay.contained = lower_ok && upper_ok && replay.contained;
    }
    if (!replay.contained) return replay;

    Dyadic exact_rho;
    Dyadic exact_denominator;
    for (std::size_t i = 0U; i < N; ++i) {
        const Dyadic residual_exact = decode_dyadic(residual[i], work);
        exact_rho = exact_add(exact_rho,
            exact_multiply(residual_exact, p0_exact[i], work), work);
        exact_denominator = exact_add(exact_denominator,
            exact_multiply(p0_exact[i], replay.exact_product[i], work), work);
        ++work[r63zp_format::KwExactDenominatorTerms];
    }
    const Quad rho_candidate_lower = directed(rho.center, rho.radius,
        FE_DOWNWARD, '-');
    const Quad rho_candidate_upper = directed(rho.center, rho.radius,
        FE_UPWARD, '+');
    const Quad denominator_candidate_lower = directed(denominator.center,
        denominator_radius, FE_DOWNWARD, '-');
    const Quad denominator_candidate_upper = directed(denominator.center,
        denominator_radius, FE_UPWARD, '+');
    const std::array<Dyadic, 4U> scalar_endpoints{{
        decode_dyadic(rho_candidate_lower, work),
        decode_dyadic(rho_candidate_upper, work),
        decode_dyadic(denominator_candidate_lower, work),
        decode_dyadic(denominator_candidate_upper, work)}};
    const bool rho_inside = compare_exact(exact_rho, scalar_endpoints[0U], work)
            >= 0
        && compare_exact(exact_rho, scalar_endpoints[1U], work) <= 0;
    const bool denominator_inside = compare_exact(exact_denominator,
            scalar_endpoints[2U], work) >= 0
        && compare_exact(exact_denominator, scalar_endpoints[3U], work) <= 0;
    work[r63zp_format::KwIntervalComparisons] += 4U;
    replay.curvature = rho_inside && denominator_inside
        && !is_zero(exact_denominator.numerator)
        && !exact_denominator.numerator.negative
        && denominator_candidate_lower > ZERO;
    if (!replay.curvature) return replay;

    std::array<Quad, N> x1{};
    for (std::size_t i = 0U; i < N; ++i) {
        x1[i] = take_little_quad(cache.data.data() + CACHE_X1
            + i * sizeof(Quad));
        ++work[r63zp_format::KwPostsealX1Decodes];
        ++work[r63zp_format::KwFinitePredicates];
        const CheckedDot update = compensated_dot(2U,
            [&](std::size_t term) {
                return term == 0U ? x0[i] : replay.expected.scalars[4U];
            },
            [&](std::size_t term) {
                return term == 0U ? ONE : replay.expected.p0[i];
            });
        ++work[r63zp_format::KwUpdateDots];
        work[r63zp_format::KwUpdateTerms] += 2U;
        ++work[r63zp_format::KwUpdateComparisons];
        ++work[r63zp_format::KwCandidateFixedLoopIterations];
        const bool match = update.exact && update.normal
            && finiteq(x1[i]) != 0
            && std::memcmp(&update.center, &x1[i], sizeof(Quad)) == 0;
        replay.update_matches += match ? 1U : 0U;
    }
    replay.update = replay.update_matches == N;
    bytes = 0U;
    const Digest x1_root = parent_vector(x1, bytes);
    ++root_calls;
    root_bytes += bytes;

    replay.expected.route = replay.update ? 7U : 6U;
    replay.expected.flags = 0x1fU | (replay.update ? 0x20U : 0U);
    replay.expected.event_count = 8U;
    replay.expected_work = expected_success_work();
    replay.expected.work = replay.expected_work;

    for (std::uint32_t ordinal = 1U; ordinal <= 4U; ++ordinal) {
        bytes = 0U;
        replay.expected.events[ordinal - 1U] = candidate_event_root(ordinal,
            7U, replay.expected, replay.derived_x0, Digest{}, bytes);
        ++root_calls;
        root_bytes += bytes;
    }
    bytes = 0U;
    replay.expected.body = candidate_body(replay.expected,
        replay.expected_work, replay.expected.events, bytes);
    ++root_calls;
    root_bytes += bytes;
    bytes = 0U;
    replay.expected.events[4U] = candidate_event_root(5U, 7U,
        replay.expected, replay.derived_x0, Digest{}, bytes);
    ++root_calls;
    root_bytes += bytes;
    bytes = 0U;
    replay.expected.events[5U] = candidate_event_root(6U, 7U,
        replay.expected, replay.derived_x0, Digest{}, bytes);
    ++root_calls;
    root_bytes += bytes;
    bytes = 0U;
    replay.expected.events[6U] = candidate_event7_root(
        replay.expected.route, x1_root, replay.update_matches, bytes);
    ++root_calls;
    root_bytes += bytes;
    std::uint64_t work_root_bytes = 0U;
    replay.expected.work_root = candidate_work_root(replay.expected.route,
        replay.expected_work, work_root_bytes);
    ++root_calls;
    root_bytes += work_root_bytes;
    bytes = 0U;
    replay.expected.events[7U] = candidate_event_root(8U,
        replay.expected.route, replay.expected, replay.derived_x0,
        replay.expected.work_root, bytes);
    ++root_calls;
    root_bytes += bytes;
    bytes = 0U;
    replay.expected.trace = candidate_trace_root(8U,
        replay.expected.events, bytes);
    ++root_calls;
    root_bytes += bytes;
    bytes = 0U;
    replay.expected.result = candidate_result_root(replay.expected,
        replay.expected.work_root, replay.expected.trace,
        replay.update_matches, bytes);
    ++root_calls;
    root_bytes += bytes;

    bool semantic = true;
    auto compare = [&](bool value) noexcept {
        ++work[r63zp_format::KwSemanticFieldComparisons];
        semantic = value && semantic;
    };
    compare(candidate.route == replay.expected.route);
    compare(candidate.flags == replay.expected.flags);
    for (std::size_t i = 0U; i < 3U; ++i)
        compare(same_digest(candidate.parents[i], replay.expected.parents[i]));
    for (std::size_t i = 0U; i < 2U; ++i)
        compare(same_digest(candidate.roles[i], replay.expected.roles[i]));
    for (std::size_t i = 0U; i < 3U; ++i)
        compare(same_digest(candidate.roots[i], replay.expected.roots[i]));
    for (std::size_t i = 0U; i < 5U; ++i)
        compare(std::memcmp(&candidate.scalars[i], &replay.expected.scalars[i],
            sizeof(Quad)) == 0);
    for (std::size_t i = 0U; i < N; ++i) {
        compare(std::memcmp(&candidate.p0[i], &replay.expected.p0[i],
            sizeof(Quad)) == 0);
        compare(std::memcmp(&candidate.q0[i], &replay.expected.q0[i],
            sizeof(Quad)) == 0);
        compare(std::memcmp(&candidate.bounds[i], &replay.expected.bounds[i],
            sizeof(Quad)) == 0);
    }
    compare(candidate.event_count == replay.expected.event_count);
    replay.semantic = semantic;
    replay.apparatus = true;
    return replay;
}

Digest checker_work_root(CheckerRoute route, const CheckerWork& work,
    std::uint64_t& bytes) noexcept {
    HashFields fields;
    fields.raw_field(reinterpret_cast<const std::uint8_t*>(CHECKER_WORK_DOMAIN),
        sizeof(CHECKER_WORK_DOMAIN) - 1U);
    fields.u32(static_cast<std::uint32_t>(route));
    fields.u64_array(work);
    bytes = fields.consumed;
    return fields.finish();
}

Digest checker_event_root(std::uint32_t ordinal, CheckerRoute route,
    const Digest& observed, const Digest& expected, bool equal,
    std::uint64_t& bytes) noexcept {
    HashFields fields;
    fields.raw_field(reinterpret_cast<const std::uint8_t*>(
        CHECKER_EVENT_DOMAIN), sizeof(CHECKER_EVENT_DOMAIN) - 1U);
    fields.u32(ordinal);
    fields.u32(static_cast<std::uint32_t>(route));
    fields.digest(observed);
    fields.digest(expected);
    fields.u32(equal ? 1U : 0U);
    bytes = fields.consumed;
    return fields.finish();
}

Digest checker_trace_root(std::uint64_t count,
    const std::array<Digest, r63zp_format::event_slots>& events,
    std::uint64_t& bytes) noexcept {
    HashFields fields;
    fields.raw_field(reinterpret_cast<const std::uint8_t*>(
        CHECKER_TRACE_DOMAIN), sizeof(CHECKER_TRACE_DOMAIN) - 1U);
    fields.u64(count);
    fields.digest_array(events);
    bytes = fields.consumed;
    return fields.finish();
}

Digest checker_result_root(CheckerRoute route, std::uint32_t flags,
    const std::array<Digest, 3U>& parents,
    const std::array<Digest, 4U>& candidate_roots,
    const Digest& work_root, std::uint64_t event_count,
    const Digest& trace, std::uint64_t& bytes) noexcept {
    HashFields fields;
    fields.raw_field(reinterpret_cast<const std::uint8_t*>(
        CHECKER_RESULT_DOMAIN), sizeof(CHECKER_RESULT_DOMAIN) - 1U);
    fields.u32(static_cast<std::uint32_t>(route));
    fields.u32(flags);
    fields.digest_array(parents);
    fields.digest_array(candidate_roots);
    fields.digest(work_root);
    fields.u64(event_count);
    fields.digest(trace);
    bytes = fields.consumed;
    return fields.finish();
}

void serialize_audit(
    std::array<std::uint8_t, r63zp_format::checker_bytes>& output,
    CheckerRoute route, std::uint32_t flags,
    const std::array<Digest, 3U>& parents,
    const std::array<Digest, 4U>& candidate_roots,
    const CheckerWork& work, const Digest& work_root,
    std::uint64_t event_count,
    const std::array<Digest, r63zp_format::event_slots>& events,
    const Digest& trace, const Digest& result) noexcept {
    std::memcpy(output.data(), r63zp_format::checker_magic.data(), 8U);
    put32(output.data() + r63zp_format::k_version, r63zp_format::version);
    put32(output.data() + r63zp_format::k_total,
        static_cast<std::uint32_t>(output.size()));
    put32(output.data() + r63zp_format::k_route,
        static_cast<std::uint32_t>(route));
    put32(output.data() + r63zp_format::k_flags, flags);
    for (std::size_t i = 0U; i < parents.size(); ++i)
        put_digest(output.data() + r63zp_format::k_parent_roots + i * 32U,
            parents[i]);
    for (std::size_t i = 0U; i < candidate_roots.size(); ++i)
        put_digest(output.data() + r63zp_format::k_candidate_roots + i * 32U,
            candidate_roots[i]);
    for (std::size_t i = 0U; i < work.size(); ++i)
        put64(output.data() + r63zp_format::k_work + i * 8U, work[i]);
    put_digest(output.data() + r63zp_format::k_work_root, work_root);
    put64(output.data() + r63zp_format::k_event_count, event_count);
    for (std::size_t i = 0U; i < events.size(); ++i)
        put_digest(output.data() + r63zp_format::k_events + i * 32U,
            events[i]);
    put_digest(output.data() + r63zp_format::k_trace_root, trace);
    put_digest(output.data() + r63zp_format::k_result_root, result);
}

bool emit_audit(const char* path,
    const std::array<std::uint8_t, r63zp_format::checker_bytes>& bytes)
    noexcept {
    const int fd = open(path, O_WRONLY | O_CREAT | O_TRUNC, 0600);
    if (fd < 0) return false;
    const ssize_t written = write(fd, bytes.data(), bytes.size());
    const bool closed = close(fd) == 0;
    return written == static_cast<ssize_t>(bytes.size()) && closed;
}

std::uint64_t exact_root_byte_count(
    const std::array<Dyadic, N>& values) noexcept {
    std::uint64_t bytes = 8U + (sizeof(EXACT_VECTOR_DOMAIN) - 1U) + 8U;
    for (const Dyadic& value : values)
        bytes += 1U + 8U + 8U + magnitude_bytes(value.numerator);
    return bytes;
}

} // namespace

int main(int argc, char** argv) {
    if (argc != 6) return 64;

    CheckerWork work{};
    ++work[r63zp_format::KwRoundingModeSetCalls];
    const bool rounding_set = std::fesetround(FE_TONEAREST) == 0;
    ++work[r63zp_format::KwRoundingModeChecks];
    const bool rounding = rounding_set && std::fegetround() == FE_TONEAREST;

    FileImage<r63zp_format::parent_cache_bytes> cache;
    FileImage<r63zp_format::parent_artifact_bytes> parent;
    FileImage<r63zp_format::parent_audit_bytes> parent_audit;
    FileImage<r63zp_format::candidate_bytes> candidate_file;
    load_file(argv[1], cache, work);
    load_file(argv[2], parent, work);
    load_file(argv[3], parent_audit, work);
    load_file(argv[4], candidate_file, work);
    CandidateImage candidate = parse_candidate(candidate_file, work);

    std::uint64_t root_calls = 0U;
    std::uint64_t root_bytes = 0U;
    Replay replay;
    if (rounding && candidate.header_valid)
        replay = derive_replay(cache, parent, parent_audit, candidate, work,
            root_calls, root_bytes);
    bool work_equal = candidate.header_valid;
    for (std::size_t i = 0U; i < candidate.work.size(); ++i) {
        const bool equal = candidate.work[i] == replay.expected_work[i];
        ++work[r63zp_format::KwCandidateWorkComparisons];
        work_equal = equal && work_equal;
    }

    bool events_equal = candidate.header_valid && replay.apparatus;
    for (std::size_t i = 0U; i < candidate.events.size(); ++i) {
        const bool equal = same_digest(candidate.events[i],
            replay.expected.events[i]);
        ++work[r63zp_format::KwEventComparisons];
        events_equal = equal && events_equal;
    }

    std::uint64_t bytes = 0U;
    const Digest observed_work_root = candidate_work_root(candidate.route,
        candidate.work, bytes);
    ++root_calls;
    root_bytes += bytes;
    const bool work_seal = same_digest(observed_work_root,
        candidate.work_root);
    ++work[r63zp_format::KwSealComparisons];

    bytes = 0U;
    const Digest observed_body = candidate_body(candidate, candidate.work,
        candidate.events, bytes);
    ++root_calls;
    root_bytes += bytes;
    const bool body_seal = same_digest(observed_body, candidate.body);
    ++work[r63zp_format::KwSealComparisons];

    bytes = 0U;
    const Digest observed_trace = candidate_trace_root(candidate.event_count,
        candidate.events, bytes);
    ++root_calls;
    root_bytes += bytes;
    const bool trace_seal = same_digest(observed_trace, candidate.trace);
    ++work[r63zp_format::KwSealComparisons];

    bytes = 0U;
    const Digest observed_result = candidate_result_root(candidate,
        candidate.work_root, candidate.trace, replay.update_matches, bytes);
    ++root_calls;
    root_bytes += bytes;
    const bool result_seal = same_digest(observed_result, candidate.result);
    ++work[r63zp_format::KwSealComparisons];
    const bool seals_equal = work_seal && body_seal && trace_seal
        && result_seal;

    CheckerRoute route = CheckerRoute::CheckerApparatusRejected;
    auto reject = [&](bool condition) noexcept {
        ++work[r63zp_format::KwRoutePredicates];
        return condition;
    };
    if (reject(!candidate.header_valid)) route = CheckerRoute::CandidateMalformed;
    else if (reject(!replay.apparatus))
        route = CheckerRoute::CheckerApparatusRejected;
    else if (reject(!replay.semantic)) route = CheckerRoute::SemanticMismatch;
    else if (reject(!work_equal)) route = CheckerRoute::WorkMismatch;
    else if (reject(!events_equal)) route = CheckerRoute::EventMismatch;
    else if (reject(!seals_equal)) route = CheckerRoute::SealMismatch;
    else {
        ++work[r63zp_format::KwRoutePredicates];
        route = static_cast<CheckerRoute>(candidate.route);
    }

    std::uint32_t flags = 0U;
    if (replay.semantic) flags |= 1U << 0U;
    if (replay.contained) flags |= 1U << 1U;
    if (replay.curvature) flags |= 1U << 2U;
    if (replay.step) flags |= 1U << 3U;
    if (replay.update) flags |= 1U << 4U;

    std::array<Digest, 4U> candidate_roots{{candidate_file.root,
        candidate.result, candidate.work_root, replay.exact_product_root}};
    const std::array<Digest, 3U> parent_roots{{cache.root, parent.root,
        parent_audit.root}};

    work[r63zp_format::KwAuditZeroFillBytes] = r63zp_format::checker_bytes;
    work[r63zp_format::KwAuditFieldsSerialized] = 94U;
    work[r63zp_format::KwAuditBytesSerialized] = r63zp_format::checker_bytes;
    work[r63zp_format::KwOutputOpenAttempts] = 1U;
    work[r63zp_format::KwOutputWriteCalls] = 1U;
    work[r63zp_format::KwOutputWriteBytes] = r63zp_format::checker_bytes;
    work[r63zp_format::KwOutputCloseCalls] = 1U;
    work[r63zp_format::KwPackageAllocations] = 0U;

    const std::uint64_t checker_work_bytes_expected =
        9U + (sizeof(CHECKER_WORK_DOMAIN) - 1U) + (9U + 4U)
        + (9U + r63zp_format::checker_work_fields * 8U);
    const std::uint64_t checker_event_bytes_expected =
        9U + (sizeof(CHECKER_EVENT_DOMAIN) - 1U) + 3U * (9U + 4U)
        + 2U * (9U + 32U);
    const std::uint64_t checker_trace_bytes_expected =
        9U + (sizeof(CHECKER_TRACE_DOMAIN) - 1U) + (9U + 8U)
        + (9U + r63zp_format::event_slots * 32U);
    const std::uint64_t checker_result_bytes_expected =
        9U + (sizeof(CHECKER_RESULT_DOMAIN) - 1U) + 2U * (9U + 4U)
        + (9U + 3U * 32U) + (9U + 4U * 32U) + (9U + 32U)
        + (9U + 8U) + (9U + 32U);
    const std::uint64_t future_root_calls = 1U
        + r63zp_format::event_slots + 1U + 1U;
    const std::uint64_t future_root_bytes = checker_work_bytes_expected
        + r63zp_format::event_slots * checker_event_bytes_expected
        + checker_trace_bytes_expected + checker_result_bytes_expected;
    work[r63zp_format::KwRootCalls] = root_calls + future_root_calls;
    work[r63zp_format::KwRootBytes] = root_bytes + future_root_bytes;

    std::uint64_t checker_work_bytes = 0U;
    const Digest checker_work = checker_work_root(route, work,
        checker_work_bytes);
    if (checker_work_bytes != checker_work_bytes_expected) return 65;

    std::array<Digest, r63zp_format::event_slots> checker_events{};
    for (std::size_t i = 0U; i < checker_events.size(); ++i) {
        const bool equal = same_digest(candidate.events[i],
            replay.expected.events[i]);
        std::uint64_t event_bytes = 0U;
        checker_events[i] = checker_event_root(static_cast<std::uint32_t>(i + 1U),
            route, candidate.events[i], replay.expected.events[i], equal,
            event_bytes);
        if (event_bytes != checker_event_bytes_expected) return 65;
    }
    std::uint64_t checker_trace_bytes = 0U;
    const Digest checker_trace = checker_trace_root(
        r63zp_format::event_slots, checker_events, checker_trace_bytes);
    if (checker_trace_bytes != checker_trace_bytes_expected) return 65;
    std::uint64_t checker_result_bytes = 0U;
    const Digest checker_result = checker_result_root(route, flags,
        parent_roots, candidate_roots, checker_work,
        r63zp_format::event_slots, checker_trace, checker_result_bytes);
    if (checker_result_bytes != checker_result_bytes_expected) return 65;

    std::array<std::uint8_t, r63zp_format::checker_bytes> output{};
    serialize_audit(output, route, flags, parent_roots, candidate_roots, work,
        checker_work, r63zp_format::event_slots, checker_events, checker_trace,
        checker_result);
    if (!emit_audit(argv[5], output)) return 65;
    return route == CheckerRoute::Admitted ? 0 : 1;
}
