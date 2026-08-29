#include "package_format.hpp"

#include "../r63zm/fixed_sha256.hpp"

#include <algorithm>
#include <array>
#include <bit>
#include <cfenv>
#include <cmath>
#include <cstddef>
#include <cstdio>
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
    std::uint64_t open_attempts = 0U;
    std::uint64_t read_calls = 0U;
    std::uint64_t read_bytes = 0U;
    std::uint64_t trailing_checks = 0U;
    std::uint64_t close_calls = 0U;
    std::uint64_t hash_calls = 0U;
    std::uint64_t hash_bytes = 0U;
    bool exact = false;
};

template <std::size_t Size>
void load_file(const char* path, FileImage<Size>& file,
    CheckerWork& work) noexcept {
    ++file.open_attempts;
    ++work[r63zp_format::KwInputOpenAttempts];
    const int fd = open(path, O_RDONLY);
    if (fd < 0) return;
    std::size_t offset = 0U;
    bool reads_ok = true;
    while (offset < file.data.size()) {
        ++file.read_calls;
        ++work[r63zp_format::KwInputReadCalls];
        const ssize_t amount = read(fd, file.data.data() + offset,
            file.data.size() - offset);
        if (amount <= 0) {
            reads_ok = false;
            break;
        }
        offset += static_cast<std::size_t>(amount);
        file.read_bytes += static_cast<std::uint64_t>(amount);
        work[r63zp_format::KwInputReadBytes]
            += static_cast<std::uint64_t>(amount);
    }
    std::uint8_t extra = 0U;
    ++file.read_calls;
    ++file.trailing_checks;
    ++work[r63zp_format::KwInputReadCalls];
    ++work[r63zp_format::KwInputTrailingChecks];
    const ssize_t extra_amount = read(fd, &extra, 1U);
    if (extra_amount > 0) {
        file.read_bytes += static_cast<std::uint64_t>(extra_amount);
        work[r63zp_format::KwInputReadBytes]
            += static_cast<std::uint64_t>(extra_amount);
    }
    ++file.close_calls;
    ++work[r63zp_format::KwInputCloseCalls];
    const bool closed = close(fd) == 0;
    file.exact = reads_ok && offset == file.data.size()
        && extra_amount == 0 && closed;
    if (file.exact) {
        ++file.hash_calls;
        file.hash_bytes += file.data.size();
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
    std::uint32_t selector = 0U;
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
    const bool route_lower = out.route >= 1U;
    const bool route_upper = out.route <= 7U;
    predicate(route_lower && route_upper);
    out.flags = take32(source.data.data() + r63zp_format::c_flags);
    out.selector = (out.flags >> 8U) & 0xffU;
    predicate((out.flags & ~0xff3fU) == 0U);
    predicate(out.selector <= 6U);
    predicate(take64(source.data.data() + r63zp_format::c_dimension) == N);
    predicate(take64(source.data.data() + r63zp_format::c_work_count)
        == r63zp_format::candidate_work_fields);
    out.event_count = take64(source.data.data()
        + r63zp_format::c_event_count);
    predicate(out.event_count <= r63zp_format::event_slots);
    work[r63zp_format::KwCandidateBytesDecoded] += 48U;
    work[r63zp_format::KwCandidateU64Decodes] += 3U;
    out.header_valid = header;
    return out;
}

template <std::size_t Size>
Digest decode_digest(const FileImage<Size>& source, std::size_t offset,
    CheckerWork& work) noexcept {
    ++work[r63zp_format::KwCandidateDigestDecodes];
    work[r63zp_format::KwCandidateBytesDecoded] += 32U;
    return take_digest(source.data.data() + offset);
}

template <std::size_t Size>
Quad decode_quad(const FileImage<Size>& source, std::size_t offset,
    CheckerWork& work) noexcept {
    ++work[r63zp_format::KwCandidateQuadDecodes];
    work[r63zp_format::KwCandidateBytesDecoded] += sizeof(Quad);
    return take_big_quad(source.data.data() + offset);
}

template <std::size_t Size>
std::uint64_t decode_u64(const FileImage<Size>& source, std::size_t offset,
    CheckerWork& work) noexcept {
    ++work[r63zp_format::KwCandidateU64Decodes];
    work[r63zp_format::KwCandidateBytesDecoded] += 8U;
    return take64(source.data.data() + offset);
}

template <std::size_t Size>
bool zero_region(const FileImage<Size>& source, std::size_t offset,
    std::size_t count, CheckerWork& work) noexcept {
    bool zero = true;
    for (std::size_t i = 0U; i < count; ++i) {
        const bool byte_zero = source.data[offset + i] == 0U;
        ++work[r63zp_format::KwCandidatePaddingBytesChecked];
        zero = byte_zero && zero;
    }
    return zero;
}

void decode_candidate_work(
    const FileImage<r63zp_format::candidate_bytes>& source,
    CandidateImage& image, std::size_t begin, std::size_t end,
    CheckerWork& work) noexcept {
    for (std::size_t i = begin; i < end; ++i) {
        image.work[i] = decode_u64(source,
            r63zp_format::c_work + i * 8U, work);
    }
}

void decode_candidate_events(
    const FileImage<r63zp_format::candidate_bytes>& source,
    CandidateImage& image, std::size_t begin, std::size_t end,
    CheckerWork& work) noexcept {
    for (std::size_t i = begin; i < end; ++i) {
        image.events[i] = decode_digest(source,
            r63zp_format::c_events + i * 32U, work);
    }
}

bool decode_candidate_semantics(
    const FileImage<r63zp_format::candidate_bytes>& source,
    CandidateImage& image, std::uint32_t expected_route,
    CheckerWork& work) noexcept {
    bool padding_zero = true;
    for (std::size_t i = 0U; i < image.parents.size(); ++i) {
        image.parents[i] = decode_digest(source,
            r63zp_format::c_parent_roots + i * 32U, work);
    }

    if (expected_route >= 2U) {
        image.roles[0U] = decode_digest(source,
            r63zp_format::c_role_roots, work);
    } else {
        padding_zero = zero_region(source, r63zp_format::c_role_roots,
            2U * 32U, work) && padding_zero;
    }
    if (expected_route >= 3U) {
        image.roles[1U] = decode_digest(source,
            r63zp_format::c_role_roots + 32U, work);
        for (std::size_t i = 0U; i < image.roots.size(); ++i) {
            image.roots[i] = decode_digest(source,
                r63zp_format::c_semantic_roots + i * 32U, work);
        }
        const std::size_t scalar_count = expected_route == 3U ? 2U
            : expected_route == 4U ? 4U : 5U;
        for (std::size_t i = 0U; i < scalar_count; ++i) {
            image.scalars[i] = decode_quad(source,
                r63zp_format::c_scalars + i * sizeof(Quad), work);
        }
        if (scalar_count < image.scalars.size()) {
            padding_zero = zero_region(source,
                r63zp_format::c_scalars + scalar_count * sizeof(Quad),
                (image.scalars.size() - scalar_count) * sizeof(Quad), work)
                && padding_zero;
        }
        for (std::size_t i = 0U; i < N; ++i) {
            image.p0[i] = decode_quad(source,
                r63zp_format::c_p0 + i * sizeof(Quad), work);
            image.q0[i] = decode_quad(source,
                r63zp_format::c_q0 + i * sizeof(Quad), work);
            image.bounds[i] = decode_quad(source,
                r63zp_format::c_bounds + i * sizeof(Quad), work);
        }
        image.body = decode_digest(source,
            r63zp_format::c_product_body_root, work);
    } else {
        if (expected_route == 2U) {
            padding_zero = zero_region(source,
                r63zp_format::c_role_roots + 32U, 32U, work)
                && padding_zero;
        }
        padding_zero = zero_region(source, r63zp_format::c_semantic_roots,
            3U * 32U, work) && padding_zero;
        padding_zero = zero_region(source, r63zp_format::c_scalars,
            5U * sizeof(Quad), work) && padding_zero;
        padding_zero = zero_region(source, r63zp_format::c_p0,
            3U * N * sizeof(Quad), work) && padding_zero;
        padding_zero = zero_region(source,
            r63zp_format::c_product_body_root, 32U, work) && padding_zero;
    }
    return padding_zero;
}

void decode_candidate_terminal(
    const FileImage<r63zp_format::candidate_bytes>& source,
    CandidateImage& image, CheckerWork& work) noexcept {
    image.work_root = decode_digest(source, r63zp_format::c_work_root, work);
    image.trace = decode_digest(source, r63zp_format::c_trace_root, work);
    image.result = decode_digest(source, r63zp_format::c_result_root, work);
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
    fields.u32(image.selector);
    fields.u64_array(work, 42U);
    fields.digest_array(events, 4U);
    fields.quad_array(image.p0);
    fields.quad_array(image.q0);
    fields.quad_array(image.bounds);
    bytes = fields.consumed;
    return fields.finish();
}

Digest candidate_event_root(std::uint32_t ordinal, std::uint32_t stage_route,
    const CandidateImage& image, const Digest& derived_x0,
    const Digest& observed_x0, const Digest& work_root,
    std::uint64_t& bytes) noexcept {
    HashFields fields;
    fields.raw_field(reinterpret_cast<const std::uint8_t*>(
        CANDIDATE_EVENT_DOMAIN), sizeof(CANDIDATE_EVENT_DOMAIN) - 1U);
    fields.u32(ordinal);
    fields.u32(stage_route);
    switch (ordinal) {
    case 1U:
        fields.u32(image.selector);
        fields.digest_array(image.parents);
        break;
    case 2U:
        fields.digest(image.roles[0U]);
        fields.digest(derived_x0);
        fields.digest(observed_x0);
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

struct Replay final {
    CandidateImage expected{};
    CandidateWork expected_work{};
    Digest derived_x0{};
    Digest observed_x0{};
    Digest exact_product_root{};
    std::array<Dyadic, N> exact_product{};
    CheckerRoute outcome = CheckerRoute::CheckerApparatusRejected;
    bool semantic_verified = false;
    bool contained = false;
    bool curvature = false;
    bool step = false;
    bool update = false;
    std::uint64_t update_matches = 0U;
    std::uint64_t producer_event_bytes = 0U;
    std::size_t compared_work = 0U;
    std::size_t compared_events = 0U;
};

std::uint64_t exact_root_byte_count(
    const std::array<Dyadic, N>& values) noexcept;

template <std::size_t Size>
void add_candidate_input_work(const FileImage<Size>& file,
    CandidateWork& work) noexcept {
    work[r63zp_format::CwInputOpenAttempts] += file.open_attempts;
    work[r63zp_format::CwInputReadCalls] += file.read_calls;
    work[r63zp_format::CwInputReadBytes] += file.read_bytes;
    work[r63zp_format::CwInputTrailingChecks] += file.trailing_checks;
    work[r63zp_format::CwInputCloseCalls] += file.close_calls;
    work[r63zp_format::CwInputHashCalls] += file.hash_calls;
    work[r63zp_format::CwInputHashBytes] += file.hash_bytes;
}

constexpr std::uint64_t body_root_bytes() noexcept {
    return 9U + (sizeof(BODY_DOMAIN) - 1U)
        + (9U + 3U * 32U) + (9U + 2U * 32U) + (9U + 3U * 32U)
        + 5U * (9U + 16U) + (9U + 8U) + (9U + 4U)
        + (9U + 42U * 8U) + (9U + 4U * 32U)
        + 3U * (9U + N * 16U);
}

constexpr std::uint64_t candidate_work_root_bytes() noexcept {
    return 9U + (sizeof(CANDIDATE_WORK_DOMAIN) - 1U) + (9U + 4U)
        + (9U + r63zp_format::candidate_work_fields * 8U);
}

constexpr std::uint64_t candidate_trace_root_bytes() noexcept {
    return 9U + (sizeof(CANDIDATE_TRACE_DOMAIN) - 1U) + (9U + 8U)
        + (9U + r63zp_format::event_slots * 32U);
}

constexpr std::uint64_t candidate_result_root_bytes() noexcept {
    return 9U + (sizeof(CANDIDATE_RESULT_DOMAIN) - 1U)
        + 2U * (9U + 4U) + (9U + 3U * 32U)
        + (9U + 2U * 32U) + (9U + 3U * 32U)
        + 5U * (9U + 16U) + (9U + 8U) + (9U + 32U)
        + (9U + 8U) + 2U * (9U + 32U) + (9U + 8U);
}

constexpr std::uint64_t event8_root_bytes() noexcept {
    return 9U + (sizeof(CANDIDATE_EVENT_DOMAIN) - 1U)
        + 2U * (9U + 4U) + 2U * (9U + 32U);
}

bool append_expected_event(Replay& replay, std::uint32_t ordinal,
    std::uint32_t stage_route, const Digest& work_root,
    std::uint64_t& root_calls, std::uint64_t& root_bytes) noexcept {
    std::uint64_t bytes = 0U;
    replay.expected.events[ordinal - 1U] = candidate_event_root(ordinal,
        stage_route, replay.expected, replay.derived_x0, replay.observed_x0,
        work_root, bytes);
    ++root_calls;
    root_bytes += bytes;
    replay.producer_event_bytes += bytes;
    return bytes != 0U;
}

bool complete_expected_candidate(Replay& replay,
    std::uint64_t x1_root_bytes, std::uint64_t& root_calls,
    std::uint64_t& root_bytes) noexcept {
    CandidateWork& work = replay.expected_work;
    const bool final_event = replay.expected.event_count == 8U;
    work[r63zp_format::CwEventRootCalls] = replay.expected.event_count;
    work[r63zp_format::CwEventRootBytes] = replay.producer_event_bytes
        + (final_event ? event8_root_bytes() : 0U);
    work[r63zp_format::CwTraceRootCalls] = 1U;
    work[r63zp_format::CwTraceRootBytes] = candidate_trace_root_bytes();
    work[r63zp_format::CwFinalRootCalls] = final_event ? 3U : 2U;
    work[r63zp_format::CwFinalRootBytes] = x1_root_bytes
        + candidate_work_root_bytes() + candidate_result_root_bytes();
    work[r63zp_format::CwReceiptZeroFillBytes]
        = r63zp_format::candidate_bytes;
    work[r63zp_format::CwReceiptFieldsSerialized] = 403U;
    work[r63zp_format::CwReceiptBytesSerialized]
        = r63zp_format::candidate_bytes;
    work[r63zp_format::CwOutputOpenAttempts] = 1U;
    work[r63zp_format::CwOutputWriteCalls] = 1U;
    work[r63zp_format::CwOutputWriteBytes]
        = r63zp_format::candidate_bytes;
    work[r63zp_format::CwOutputCloseCalls] = 1U;
    work[r63zp_format::CwPackageAllocations] = 0U;
    replay.expected.work = work;

    std::uint64_t bytes = 0U;
    replay.expected.work_root = candidate_work_root(replay.expected.route,
        work, bytes);
    ++root_calls;
    root_bytes += bytes;
    if (bytes != candidate_work_root_bytes()) return false;
    if (final_event) {
        bytes = 0U;
        replay.expected.events[7U] = candidate_event_root(8U,
            replay.expected.route, replay.expected, replay.derived_x0,
            replay.observed_x0, replay.expected.work_root, bytes);
        ++root_calls;
        root_bytes += bytes;
        if (bytes != event8_root_bytes()) return false;
    }
    bytes = 0U;
    replay.expected.trace = candidate_trace_root(
        replay.expected.event_count, replay.expected.events, bytes);
    ++root_calls;
    root_bytes += bytes;
    if (bytes != candidate_trace_root_bytes()) return false;
    bytes = 0U;
    replay.expected.result = candidate_result_root(replay.expected,
        replay.expected.work_root, replay.expected.trace,
        replay.update_matches, bytes);
    ++root_calls;
    root_bytes += bytes;
    return bytes == candidate_result_root_bytes();
}

bool compare_semantics(const CandidateImage& candidate,
    const CandidateImage& expected, bool padding_zero,
    CheckerWork& work) noexcept {
    bool equal = true;
    auto compare = [&](bool value) noexcept {
        ++work[r63zp_format::KwSemanticFieldComparisons];
        equal = value && equal;
    };
    compare(padding_zero);
    compare(candidate.route == expected.route);
    compare(candidate.flags == expected.flags);
    compare(candidate.event_count == expected.event_count);
    for (std::size_t i = 0U; i < expected.parents.size(); ++i)
        compare(same_digest(candidate.parents[i], expected.parents[i]));
    for (std::size_t i = 0U; i < expected.roles.size(); ++i)
        compare(same_digest(candidate.roles[i], expected.roles[i]));
    for (std::size_t i = 0U; i < expected.roots.size(); ++i)
        compare(same_digest(candidate.roots[i], expected.roots[i]));
    for (std::size_t i = 0U; i < expected.scalars.size(); ++i) {
        compare(std::memcmp(&candidate.scalars[i], &expected.scalars[i],
            sizeof(Quad)) == 0);
    }
    if (expected.route >= 3U) {
        for (std::size_t i = 0U; i < N; ++i) {
            compare(std::memcmp(&candidate.p0[i], &expected.p0[i],
                sizeof(Quad)) == 0);
            compare(std::memcmp(&candidate.q0[i], &expected.q0[i],
                sizeof(Quad)) == 0);
            compare(std::memcmp(&candidate.bounds[i], &expected.bounds[i],
                sizeof(Quad)) == 0);
        }
    }
    return equal;
}

bool compare_work_range(const CandidateImage& candidate,
    const CandidateWork& expected, std::size_t begin, std::size_t end,
    CheckerWork& work) noexcept {
    bool equal = true;
    for (std::size_t i = begin; i < end; ++i) {
        const bool field_equal = candidate.work[i] == expected[i];
        ++work[r63zp_format::KwCandidateWorkComparisons];
        equal = field_equal && equal;
    }
    return equal;
}

bool compare_event_range(const CandidateImage& candidate,
    const CandidateImage& expected, std::size_t begin, std::size_t end,
    CheckerWork& work) noexcept {
    bool equal = true;
    for (std::size_t i = begin; i < end; ++i) {
        const bool event_equal = same_digest(candidate.events[i],
            expected.events[i]);
        ++work[r63zp_format::KwEventComparisons];
        equal = event_equal && equal;
    }
    return equal;
}

CheckerRoute verify_terminal_candidate(
    const FileImage<r63zp_format::candidate_bytes>& source,
    CandidateImage& candidate, Replay& replay, CheckerWork& work,
    std::uint64_t& root_calls, std::uint64_t& root_bytes) noexcept {
    decode_candidate_work(source, candidate, replay.compared_work,
        candidate.work.size(), work);
    const bool work_equal = compare_work_range(candidate,
        replay.expected_work, replay.compared_work, candidate.work.size(),
        work);
    replay.compared_work = candidate.work.size();
    if (!work_equal) return CheckerRoute::WorkMismatch;

    const std::size_t available = static_cast<std::size_t>(
        replay.expected.event_count);
    if (replay.compared_events < available) {
        decode_candidate_events(source, candidate, replay.compared_events,
            available, work);
    }
    bool events_equal = compare_event_range(candidate, replay.expected,
        replay.compared_events, available, work);
    replay.compared_events = available;
    for (std::size_t i = available; i < candidate.events.size(); ++i) {
        const bool slot_zero = zero_region(source,
            r63zp_format::c_events + i * 32U, 32U, work);
        ++work[r63zp_format::KwEventComparisons];
        events_equal = slot_zero && events_equal;
    }
    replay.compared_events = candidate.events.size();
    if (!events_equal) return CheckerRoute::EventMismatch;

    decode_candidate_terminal(source, candidate, work);
    std::uint64_t bytes = 0U;
    const Digest observed_work = candidate_work_root(candidate.route,
        candidate.work, bytes);
    ++root_calls;
    root_bytes += bytes;
    const bool work_self = same_digest(observed_work, candidate.work_root);
    const bool work_expected = same_digest(candidate.work_root,
        replay.expected.work_root);
    work[r63zp_format::KwSealComparisons] += 2U;

    bytes = 0U;
    const Digest observed_trace = candidate_trace_root(candidate.event_count,
        candidate.events, bytes);
    ++root_calls;
    root_bytes += bytes;
    const bool trace_self = same_digest(observed_trace, candidate.trace);
    const bool trace_expected = same_digest(candidate.trace,
        replay.expected.trace);
    work[r63zp_format::KwSealComparisons] += 2U;

    bytes = 0U;
    const Digest observed_result = candidate_result_root(candidate,
        candidate.work_root, candidate.trace, replay.update_matches, bytes);
    ++root_calls;
    root_bytes += bytes;
    const bool result_self = same_digest(observed_result, candidate.result);
    const bool result_expected = same_digest(candidate.result,
        replay.expected.result);
    work[r63zp_format::KwSealComparisons] += 2U;
    return work_self && work_expected && trace_self && trace_expected
            && result_self && result_expected
        ? static_cast<CheckerRoute>(candidate.route)
        : CheckerRoute::SealMismatch;
}

Replay derive_replay(
    const FileImage<r63zp_format::parent_cache_bytes>& cache,
    const FileImage<r63zp_format::parent_artifact_bytes>& parent,
    const FileImage<r63zp_format::parent_audit_bytes>& parent_audit,
    const FileImage<r63zp_format::candidate_bytes>& candidate_source,
    CandidateImage& candidate, CheckerWork& work,
    std::uint64_t& root_calls, std::uint64_t& root_bytes) noexcept {
    Replay replay;
    CandidateWork& producer = replay.expected_work;
    producer[r63zp_format::CwRoundingModeSetCalls] = 1U;
    producer[r63zp_format::CwRoundingModeChecks] = 1U;
    producer[r63zp_format::CwControlSelectorPredicates]
        = candidate.selector == 0U ? 1U : 6U;
    add_candidate_input_work(cache, producer);
    add_candidate_input_work(parent, producer);
    add_candidate_input_work(parent_audit, producer);
    replay.expected.selector = candidate.selector;
    replay.expected.parents = {{cache.root, parent.root, parent_audit.root}};
    bool parents = cache.exact && parent.exact && parent_audit.exact;
    const std::array<bool, 3U> identities{{matches_hex(cache.root, CACHE_ID),
        matches_hex(parent.root, ARTIFACT_ID),
        matches_hex(parent_audit.root, AUDIT_ID)}};
    for (bool identity : identities) {
        ++work[r63zp_format::KwParentRootComparisons];
        ++producer[r63zp_format::CwParentRootComparisons];
        parents = identity && parents;
    }

    constexpr std::array<std::uint8_t, 8U> parent_magic{{
        'N','E','R','6','3','Z','M','1'}};
    constexpr std::array<std::uint8_t, 8U> audit_magic{{
        'N','E','R','6','3','Q','C','1'}};
    if (parents) {
        auto parent_predicate = [&](bool value) noexcept {
            ++work[r63zp_format::KwParentHeaderPredicates];
            ++producer[r63zp_format::CwParentHeaderPredicates];
            parents = value && parents;
        };
        parent_predicate(std::memcmp(parent.data.data(), parent_magic.data(),
            8U) == 0);
        parent_predicate(take64(parent.data.data() + 12U)
            == r63zp_format::parent_artifact_bytes);
        parent_predicate(parent.data[ROLE2] == 1U);
        parent_predicate(parent.data[ROLE2 + 1U] == 1U);
        parent_predicate(parent.data[ROLE2 + 2U] == 2U);
        parent_predicate(take64(parent.data.data() + ROLE2 + 304U) == N);
        parent_predicate(std::memcmp(parent_audit.data.data(),
            audit_magic.data(), 8U) == 0);
        parent_predicate(take64(parent_audit.data.data() + 12U)
            == r63zp_format::parent_audit_bytes);
        parent_predicate(parent_audit.data[24U] == 1U);
        parent_predicate(parent_audit.data[25U] == 1U);
        parent_predicate(parent_audit.data[26U] == 1U);
    }

    if (!parents) {
        ++producer[r63zp_format::CwRoutePredicates];
        replay.expected.route = 1U;
        replay.expected.flags = candidate.selector << 8U;
        replay.expected.event_count = 1U;
        append_expected_event(replay, 1U, 1U, Digest{}, root_calls,
            root_bytes);
        const bool padding = decode_candidate_semantics(candidate_source,
            candidate, replay.expected.route, work);
        decode_candidate_events(candidate_source, candidate, 0U, 1U, work);
        if (!compare_semantics(candidate, replay.expected, padding, work)) {
            replay.outcome = CheckerRoute::SemanticMismatch;
            return replay;
        }
        replay.semantic_verified = true;
        replay.compared_events = 1U;
        if (!compare_event_range(candidate, replay.expected, 0U, 1U, work)) {
            replay.outcome = CheckerRoute::EventMismatch;
            return replay;
        }
        if (!complete_expected_candidate(replay, 0U, root_calls,
            root_bytes)) return replay;
        replay.outcome = verify_terminal_candidate(candidate_source,
            candidate, replay, work, root_calls, root_bytes);
        return replay;
    }

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
        producer[r63zp_format::CwCacheQuadDecodesPreseal] += 2U;
        ++work[r63zp_format::KwCacheIndexDecodes];
        ++producer[r63zp_format::CwCacheIndexDecodes];
        work[r63zp_format::KwFinitePredicates] += 2U;
        producer[r63zp_format::CwFinitePredicates] += 2U;
        ++work[r63zp_format::KwRangePredicates];
        ++producer[r63zp_format::CwRangePredicates];
        const bool x0_finite = finiteq(x0[i]) != 0;
        const bool rhs_finite = finiteq(rhs[i]) != 0;
        const bool permutation_valid = permutation[i] < N;
        const bool row = x0_finite && rhs_finite && permutation_valid;
        inputs = row && inputs;
    }
    for (std::size_t i = 0U; i < tangent.size(); ++i) {
        tangent[i] = take_little_quad(cache.data.data() + CACHE_TANGENT
            + i * sizeof(Quad));
        ++work[r63zp_format::KwCacheQuadDecodesPreseal];
        ++producer[r63zp_format::CwCacheQuadDecodesPreseal];
        ++work[r63zp_format::KwFinitePredicates];
        ++producer[r63zp_format::CwFinitePredicates];
        inputs = finiteq(tangent[i]) != 0 && inputs;
    }
    for (std::size_t i = 0U; i < factor.size(); ++i) {
        factor[i] = static_cast<Quad>(take_little_double(cache.data.data()
            + CACHE_FACTOR + i * sizeof(double)));
        ++work[r63zp_format::KwCacheDoubleDecodes];
        ++producer[r63zp_format::CwCacheDoubleDecodes];
        ++work[r63zp_format::KwFinitePredicates];
        ++producer[r63zp_format::CwFinitePredicates];
        inputs = finiteq(factor[i]) != 0 && inputs;
    }
    const Quad sigma = take_little_quad(cache.data.data() + CACHE_SIGMA);
    const Quad inverse = take_little_quad(cache.data.data() + CACHE_INVERSE);
    work[r63zp_format::KwCacheQuadDecodesPreseal] += 2U;
    producer[r63zp_format::CwCacheQuadDecodesPreseal] += 2U;
    work[r63zp_format::KwFinitePredicates] += 2U;
    producer[r63zp_format::CwFinitePredicates] += 2U;
    work[r63zp_format::KwRangePredicates] += 2U;
    producer[r63zp_format::CwRangePredicates] += 2U;
    const bool sigma_finite = finiteq(sigma) != 0;
    const bool inverse_finite = finiteq(inverse) != 0;
    const bool sigma_positive = sigma > ZERO;
    const bool inverse_positive = inverse > ZERO;
    inputs = sigma_finite && inverse_finite && sigma_positive
        && inverse_positive && inputs;

    if (candidate.selector == 1U) {
        x0[0U] = nextafterq(x0[0U], HUGE_VALQ);
        ++work[r63zp_format::KwControlInjections];
        ++producer[r63zp_format::CwPresealControlInjections];
    }

    const FactorResult initial = apply_factor(factor, permutation, rhs,
        inverse, work);
    ++producer[r63zp_format::CwFactorSolves];
    producer[r63zp_format::CwFactorTerms] += initial.terms;
    producer[r63zp_format::CwFactorDivisions] += initial.divisions;
    bool x0_matches = initial.valid;
    for (std::size_t i = 0U; i < N; ++i) {
        const bool equal = std::memcmp(&initial.value[i], &x0[i],
            sizeof(Quad)) == 0;
        ++work[r63zp_format::KwX0ComponentComparisons];
        ++work[r63zp_format::KwCandidateFixedLoopIterations];
        ++producer[r63zp_format::CwX0ComponentComparisons];
        ++producer[r63zp_format::CwFixedLoopIterations];
        x0_matches = equal && x0_matches;
    }
    std::uint64_t bytes = 0U;
    replay.derived_x0 = parent_vector(initial.value, bytes);
    ++root_calls;
    root_bytes += bytes;
    ++producer[r63zp_format::CwPresealRootCalls];
    producer[r63zp_format::CwPresealRootBytes] += bytes;
    bytes = 0U;
    replay.observed_x0 = parent_vector(x0, bytes);
    ++root_calls;
    root_bytes += bytes;
    ++producer[r63zp_format::CwPresealRootCalls];
    producer[r63zp_format::CwPresealRootBytes] += bytes;
    replay.expected.roles[0U] = take_digest(parent.data.data()
        + ROLE2_INPUT_ROOT);
    ++work[r63zp_format::KwParentRootComparisons];
    ++producer[r63zp_format::CwParentRootComparisons];
    x0_matches = same_digest(replay.derived_x0,
        replay.expected.roles[0U]) && x0_matches;
    if (!inputs || !x0_matches) {
        producer[r63zp_format::CwRoutePredicates] = 2U;
        replay.expected.route = 2U;
        replay.expected.flags = candidate.selector << 8U;
        replay.expected.event_count = 2U;
        append_expected_event(replay, 1U, 7U, Digest{}, root_calls,
            root_bytes);
        append_expected_event(replay, 2U, 2U, Digest{}, root_calls,
            root_bytes);
        const bool padding = decode_candidate_semantics(candidate_source,
            candidate, replay.expected.route, work);
        decode_candidate_events(candidate_source, candidate, 0U, 2U, work);
        if (!compare_semantics(candidate, replay.expected, padding, work)) {
            replay.outcome = CheckerRoute::SemanticMismatch;
            return replay;
        }
        replay.semantic_verified = true;
        replay.compared_events = 2U;
        if (!compare_event_range(candidate, replay.expected, 0U, 2U, work)) {
            replay.outcome = CheckerRoute::EventMismatch;
            return replay;
        }
        if (!complete_expected_candidate(replay, 0U, root_calls,
            root_bytes)) return replay;
        replay.outcome = verify_terminal_candidate(candidate_source,
            candidate, replay, work, root_calls, root_bytes);
        return replay;
    }

    std::array<Quad, N> y0{};
    bool role = true;
    for (std::size_t i = 0U; i < N; ++i) {
        y0[i] = take_big_quad(parent.data.data() + ROLE2 + PRODUCT_VALUES
            + i * sizeof(Quad));
        ++work[r63zp_format::KwParentArtifactQuadDecodes];
        ++producer[r63zp_format::CwParentArtifactQuadDecodes];
        ++work[r63zp_format::KwFinitePredicates];
        ++producer[r63zp_format::CwFinitePredicates];
        role = finiteq(y0[i]) != 0 && role;
    }
    replay.expected.roles[1U] = take_digest(parent.data.data()
        + ROLE2_VALUE_ROOT);
    bytes = 0U;
    const Digest y0_root = parent_vector(y0, bytes);
    ++root_calls;
    root_bytes += bytes;
    ++producer[r63zp_format::CwPresealRootCalls];
    producer[r63zp_format::CwPresealRootBytes] += bytes;
    ++work[r63zp_format::KwParentRootComparisons];
    ++producer[r63zp_format::CwParentRootComparisons];
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
        ++producer[r63zp_format::CwResidualDots];
        producer[r63zp_format::CwResidualTerms] += 2U;
        residual[i] = dot.center;
        residual_valid = dot.exact && dot.normal && residual_valid;
    }
    const FactorResult direction = apply_factor(factor, permutation, residual,
        inverse, work);
    ++producer[r63zp_format::CwFactorSolves];
    producer[r63zp_format::CwFactorTerms] += direction.terms;
    producer[r63zp_format::CwFactorDivisions] += direction.divisions;
    replay.expected.p0 = direction.value;
    const CheckedDot rho = compensated_dot(N,
        [&](std::size_t i) { return residual[i]; },
        [&](std::size_t i) { return replay.expected.p0[i]; });
    ++work[r63zp_format::KwRhoDots];
    work[r63zp_format::KwRhoTerms] += N;
    ++work[r63zp_format::KwPositivityPredicates];
    ++producer[r63zp_format::CwRhoDots];
    producer[r63zp_format::CwRhoTerms] += N;
    ++producer[r63zp_format::CwPositivityPredicates];
    const bool rho_positive = rho.center - rho.radius > ZERO;
    const bool direction_valid = residual_valid && direction.valid
        && rho.exact && rho.normal && rho_positive;
    if (!role || !direction_valid) {
        producer[r63zp_format::CwRoutePredicates] = 2U;
        replay.expected.route = 2U;
        replay.expected.flags = candidate.selector << 8U;
        replay.expected.event_count = 2U;
        append_expected_event(replay, 1U, 7U, Digest{}, root_calls,
            root_bytes);
        append_expected_event(replay, 2U, 2U, Digest{}, root_calls,
            root_bytes);
        const bool padding = decode_candidate_semantics(candidate_source,
            candidate, replay.expected.route, work);
        decode_candidate_events(candidate_source, candidate, 0U, 2U, work);
        if (!compare_semantics(candidate, replay.expected, padding, work)) {
            replay.outcome = CheckerRoute::SemanticMismatch;
            return replay;
        }
        replay.semantic_verified = true;
        replay.compared_events = 2U;
        if (!compare_event_range(candidate, replay.expected, 0U, 2U, work)) {
            replay.outcome = CheckerRoute::EventMismatch;
            return replay;
        }
        if (!complete_expected_candidate(replay, 0U, root_calls,
            root_bytes)) return replay;
        replay.outcome = verify_terminal_candidate(candidate_source,
            candidate, replay, work, root_calls, root_bytes);
        return replay;
    }

    replay.expected.scalars[0U] = rho.center;
    replay.expected.scalars[1U] = rho.radius;
    const ReplayProduct candidate_product = replay_product(tangent, sigma,
        replay.expected.p0, work);
    ++producer[r63zp_format::CwProductKernels];
    producer[r63zp_format::CwInnerDots] += M;
    producer[r63zp_format::CwInnerTerms] += M * N;
    producer[r63zp_format::CwOuterDots] += N;
    producer[r63zp_format::CwOuterTerms] += N * M;
    producer[r63zp_format::CwPropagationTerms] += N * M;
    producer[r63zp_format::CwScaleProducts] += N;
    replay.expected.q0 = candidate_product.center;
    replay.expected.bounds = candidate_product.radius;
    if (candidate.selector == 2U) {
        replay.expected.bounds[0U] = ZERO;
        ++work[r63zp_format::KwControlInjections];
        ++producer[r63zp_format::CwPresealControlInjections];
    }
    bool bounds_valid = true;
    for (std::size_t i = 0U; i < N; ++i) {
        ++work[r63zp_format::KwFinitePredicates];
        ++work[r63zp_format::KwRangePredicates];
        ++producer[r63zp_format::CwFinitePredicates];
        ++producer[r63zp_format::CwRangePredicates];
        const bool bound_finite = finiteq(replay.expected.bounds[i]) != 0;
        const bool bound_nonnegative = replay.expected.bounds[i] >= ZERO;
        bounds_valid = bound_finite && bound_nonnegative && bounds_valid;
    }

    bytes = 0U;
    replay.expected.roots[0U] = candidate_vector(replay.expected.p0, bytes);
    ++root_calls;
    root_bytes += bytes;
    ++producer[r63zp_format::CwPresealRootCalls];
    producer[r63zp_format::CwPresealRootBytes] += bytes;
    bytes = 0U;
    replay.expected.roots[1U] = candidate_vector(replay.expected.q0, bytes);
    ++root_calls;
    root_bytes += bytes;
    ++producer[r63zp_format::CwPresealRootCalls];
    producer[r63zp_format::CwPresealRootBytes] += bytes;
    bytes = 0U;
    replay.expected.roots[2U] = candidate_vector(replay.expected.bounds,
        bytes);
    ++root_calls;
    root_bytes += bytes;
    ++producer[r63zp_format::CwPresealRootCalls];
    producer[r63zp_format::CwPresealRootBytes] += bytes;

    CheckedDot denominator;
    Quad denominator_radius = ZERO;
    Quad rho_lower = ZERO;
    Quad rho_upper = ZERO;
    Quad denominator_lower = ZERO;
    Quad denominator_upper = ZERO;
    bool rounded_curvature = false;
    bool rounded_step = false;
    if (candidate_product.valid && bounds_valid && candidate.selector != 2U) {
        denominator = compensated_dot(N,
            [&](std::size_t i) { return replay.expected.p0[i]; },
            [&](std::size_t i) { return replay.expected.q0[i]; });
        ++producer[r63zp_format::CwDenominatorDots];
        producer[r63zp_format::CwDenominatorTerms] += N;
        denominator_radius = denominator.radius;
        for (std::size_t i = 0U; i < N; ++i) {
            denominator_radius = upward_add(denominator_radius,
                upward_multiply(fabsq(replay.expected.p0[i]),
                    replay.expected.bounds[i]));
            ++producer[r63zp_format::CwDenominatorTerms];
        }
        Quad denominator_center = denominator.center;
        if (candidate.selector == 3U) {
            denominator_radius = denominator_center;
            ++work[r63zp_format::KwControlInjections];
            ++producer[r63zp_format::CwPresealControlInjections];
        } else if (candidate.selector == 4U) {
            denominator_center = -fabsq(denominator_center);
            ++work[r63zp_format::KwControlInjections];
            ++producer[r63zp_format::CwPresealControlInjections];
        }
        replay.expected.scalars[2U] = denominator_center;
        replay.expected.scalars[3U] = denominator_radius;
        rho_lower = directed(rho.center, rho.radius, FE_DOWNWARD, '-');
        rho_upper = directed(rho.center, rho.radius, FE_UPWARD, '+');
        denominator_lower = directed(denominator_center,
            denominator_radius, FE_DOWNWARD, '-');
        denominator_upper = directed(denominator_center,
            denominator_radius, FE_UPWARD, '+');
        work[r63zp_format::KwDivisionEndpointOperations] += 4U;
        producer[r63zp_format::CwIntervalEndpointOperations] += 4U;
        ++work[r63zp_format::KwPositivityPredicates];
        ++producer[r63zp_format::CwPositivityPredicates];
        const bool denominator_positive = denominator_lower > ZERO;
        rounded_curvature = denominator.exact && denominator.normal
            && denominator_positive;
        if (rounded_curvature) {
            const Quad step_lower = directed(rho_lower, denominator_upper,
                FE_DOWNWARD, '/');
            const Quad step_upper = directed(rho_upper, denominator_lower,
                FE_UPWARD, '/');
            replay.expected.scalars[4U] = rho.center / denominator_center;
            work[r63zp_format::KwDivisionEndpointOperations] += 3U;
            producer[r63zp_format::CwDivisionEndpointOperations] += 3U;
            if (candidate.selector == 5U) {
                replay.expected.scalars[4U] = nextafterq(step_upper,
                    HUGE_VALQ);
                ++work[r63zp_format::KwControlInjections];
                ++producer[r63zp_format::CwPresealControlInjections];
            }
            const bool lower_finite = finiteq(step_lower) != 0;
            const bool upper_finite = finiteq(step_upper) != 0;
            const bool lower_contains = step_lower
                <= replay.expected.scalars[4U];
            const bool upper_contains = replay.expected.scalars[4U]
                <= step_upper;
            work[r63zp_format::KwFinitePredicates] += 2U;
            producer[r63zp_format::CwFinitePredicates] += 2U;
            work[r63zp_format::KwDivisionPredicates] += 2U;
            producer[r63zp_format::CwDivisionPredicates] += 2U;
            rounded_step = lower_finite && upper_finite
                && lower_contains && upper_contains;
        }
    }

    if (candidate.selector == 2U || !candidate_product.valid || !bounds_valid) {
        producer[r63zp_format::CwRoutePredicates] = 3U;
        replay.expected.route = 3U;
    } else if (!rounded_curvature) {
        producer[r63zp_format::CwRoutePredicates] = 4U;
        replay.expected.route = 4U;
    } else if (!rounded_step) {
        producer[r63zp_format::CwRoutePredicates] = 5U;
        replay.expected.route = 5U;
    } else {
        producer[r63zp_format::CwRoutePredicates] = 5U;
        replay.expected.route = candidate.selector == 6U ? 6U : 7U;
    }
    replay.expected.event_count = replay.expected.route <= 3U ? 5U
        : replay.expected.route <= 5U ? 6U : 8U;
    replay.expected.flags = candidate.selector << 8U;
    replay.expected.flags |= 1U << 0U;
    if (candidate_product.valid) replay.expected.flags |= 1U << 1U;
    if (bounds_valid) replay.expected.flags |= 1U << 2U;
    if (rounded_curvature) replay.expected.flags |= 1U << 3U;
    if (rounded_step) replay.expected.flags |= 1U << 4U;
    if (replay.expected.route == 7U) replay.expected.flags |= 1U << 5U;

    for (std::uint32_t ordinal = 1U; ordinal <= 4U; ++ordinal) {
        append_expected_event(replay, ordinal, 7U, Digest{}, root_calls,
            root_bytes);
    }
    producer[r63zp_format::CwPresealRootCalls] += 1U;
    producer[r63zp_format::CwPresealRootBytes] += body_root_bytes();
    replay.expected.work = producer;
    bytes = 0U;
    replay.expected.body = candidate_body(replay.expected, producer,
        replay.expected.events, bytes);
    ++root_calls;
    root_bytes += bytes;
    if (bytes != body_root_bytes()) return replay;
    const std::uint32_t body_stage = replay.expected.route <= 5U
        ? replay.expected.route : 7U;
    append_expected_event(replay, 5U, body_stage, Digest{}, root_calls,
        root_bytes);

    const bool padding = decode_candidate_semantics(candidate_source,
        candidate, replay.expected.route, work);
    decode_candidate_work(candidate_source, candidate, 0U, 42U, work);
    decode_candidate_events(candidate_source, candidate, 0U, 5U, work);
    if (!compare_semantics(candidate, replay.expected, padding, work)) {
        replay.outcome = CheckerRoute::SemanticMismatch;
        return replay;
    }
    replay.semantic_verified = true;
    replay.compared_work = 42U;
    if (!compare_work_range(candidate, producer, 0U, 42U, work)) {
        replay.outcome = CheckerRoute::WorkMismatch;
        return replay;
    }
    replay.compared_events = 5U;
    if (!compare_event_range(candidate, replay.expected, 0U, 5U, work)) {
        replay.outcome = CheckerRoute::EventMismatch;
        return replay;
    }
    bytes = 0U;
    const Digest observed_body = candidate_body(candidate, candidate.work,
        candidate.events, bytes);
    ++root_calls;
    root_bytes += bytes;
    const bool body_expected = same_digest(candidate.body,
        replay.expected.body);
    const bool body_self = same_digest(observed_body, candidate.body);
    work[r63zp_format::KwSealComparisons] += 2U;
    if (!body_expected || !body_self) {
        replay.outcome = CheckerRoute::SealMismatch;
        return replay;
    }

    // Stage 2 starts only after the independent rounded body and event 5 pass.
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
        || !exact_capacity) return replay;

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

    if (!replay.contained) {
        if (replay.expected.route != 3U) {
            replay.semantic_verified = false;
            replay.outcome = CheckerRoute::SemanticMismatch;
            return replay;
        }
        if (!complete_expected_candidate(replay, 0U, root_calls,
            root_bytes)) return replay;
        replay.outcome = verify_terminal_candidate(candidate_source,
            candidate, replay, work, root_calls, root_bytes);
        return replay;
    }

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
    const Quad denominator_candidate_lower = directed(
        replay.expected.scalars[2U], replay.expected.scalars[3U],
        FE_DOWNWARD, '-');
    const Quad denominator_candidate_upper = directed(
        replay.expected.scalars[2U], replay.expected.scalars[3U],
        FE_UPWARD, '+');
    const std::array<Dyadic, 4U> scalar_endpoints{{
        decode_dyadic(rho_candidate_lower, work),
        decode_dyadic(rho_candidate_upper, work),
        decode_dyadic(denominator_candidate_lower, work),
        decode_dyadic(denominator_candidate_upper, work)}};
    const bool rho_lower_ok = compare_exact(exact_rho,
        scalar_endpoints[0U], work) >= 0;
    const bool rho_upper_ok = compare_exact(exact_rho,
        scalar_endpoints[1U], work) <= 0;
    const bool denominator_lower_ok = compare_exact(exact_denominator,
        scalar_endpoints[2U], work) >= 0;
    const bool denominator_upper_ok = compare_exact(exact_denominator,
        scalar_endpoints[3U], work) <= 0;
    work[r63zp_format::KwIntervalComparisons] += 4U;
    work[r63zp_format::KwPositivityPredicates] += 3U;
    const bool rho_inside = rho_lower_ok && rho_upper_ok;
    const bool denominator_inside = denominator_lower_ok
        && denominator_upper_ok;
    const bool exact_denominator_nonzero =
        !is_zero(exact_denominator.numerator);
    const bool exact_denominator_nonnegative =
        !exact_denominator.numerator.negative;
    const bool candidate_denominator_positive =
        denominator_candidate_lower > ZERO;
    replay.curvature = rho_inside && denominator_inside
        && exact_denominator_nonzero && exact_denominator_nonnegative
        && candidate_denominator_positive;
    replay.step = replay.curvature && rounded_step;

    if (replay.expected.route >= 4U) {
        const std::uint32_t scalar_stage = replay.expected.route <= 5U
            ? replay.expected.route : 7U;
        append_expected_event(replay, 6U, scalar_stage, Digest{}, root_calls,
            root_bytes);
        decode_candidate_events(candidate_source, candidate, 5U, 6U, work);
        if (!compare_event_range(candidate, replay.expected, 5U, 6U, work)) {
            replay.outcome = CheckerRoute::EventMismatch;
            return replay;
        }
        replay.compared_events = 6U;
    }

    if (!replay.curvature) {
        if (replay.expected.route != 4U) {
            replay.semantic_verified = false;
            replay.outcome = CheckerRoute::SemanticMismatch;
            return replay;
        }
        if (!complete_expected_candidate(replay, 0U, root_calls,
            root_bytes)) return replay;
        replay.outcome = verify_terminal_candidate(candidate_source,
            candidate, replay, work, root_calls, root_bytes);
        return replay;
    }
    if (!replay.step) {
        if (replay.expected.route != 5U) {
            replay.semantic_verified = false;
            replay.outcome = CheckerRoute::SemanticMismatch;
            return replay;
        }
        if (!complete_expected_candidate(replay, 0U, root_calls,
            root_bytes)) return replay;
        replay.outcome = verify_terminal_candidate(candidate_source,
            candidate, replay, work, root_calls, root_bytes);
        return replay;
    }

    // Stage 3 is the first point at which the frozen future x1 is decoded.
    std::array<Quad, N> x1{};
    for (std::size_t i = 0U; i < N; ++i) {
        x1[i] = take_little_quad(cache.data.data() + CACHE_X1
            + i * sizeof(Quad));
        ++work[r63zp_format::KwPostsealX1Decodes];
        ++producer[r63zp_format::CwPostsealX1Decodes];
    }
    if (candidate.selector == 6U) {
        x1[0U] = nextafterq(x1[0U], HUGE_VALQ);
        ++work[r63zp_format::KwControlInjections];
        ++producer[r63zp_format::CwPostsealControlInjections];
    }
    for (std::size_t i = 0U; i < N; ++i) {
        ++work[r63zp_format::KwFinitePredicates];
        ++producer[r63zp_format::CwPostsealX1FinitePredicates];
        const bool x1_finite = finiteq(x1[i]) != 0;
        const CheckedDot update_value = compensated_dot(2U,
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
        ++producer[r63zp_format::CwUpdateDots];
        producer[r63zp_format::CwUpdateTerms] += 2U;
        ++producer[r63zp_format::CwUpdateComparisons];
        ++producer[r63zp_format::CwFixedLoopIterations];
        const bool value_equal = std::memcmp(&update_value.center, &x1[i],
            sizeof(Quad)) == 0;
        const bool match = update_value.exact && update_value.normal
            && x1_finite && value_equal;
        replay.update_matches += match ? 1U : 0U;
    }
    replay.update = replay.update_matches == N;
    ++producer[r63zp_format::CwRoutePredicates];
    const std::uint32_t derived_route = replay.update ? 7U : 6U;
    if (derived_route != replay.expected.route) {
        replay.semantic_verified = false;
        replay.outcome = CheckerRoute::SemanticMismatch;
        return replay;
    }
    std::uint64_t x1_bytes = 0U;
    const Digest x1_root = parent_vector(x1, x1_bytes);
    ++root_calls;
    root_bytes += x1_bytes;
    replay.expected.events[6U] = candidate_event7_root(
        replay.expected.route, x1_root, replay.update_matches, bytes);
    ++root_calls;
    root_bytes += bytes;
    replay.producer_event_bytes += bytes;
    if (!complete_expected_candidate(replay, x1_bytes, root_calls,
        root_bytes)) return replay;
    replay.outcome = verify_terminal_candidate(candidate_source, candidate,
        replay, work, root_calls, root_bytes);
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

int run_exact_controls() noexcept {
    CheckerWork work{};
    const Dyadic zero = decode_dyadic(ZERO, work);
    const Dyadic negative = decode_dyadic(-ONE, work);
    const Dyadic one = decode_dyadic(ONE, work);
    const Dyadic nonfinite = decode_dyadic(HUGE_VALQ, work);
    const Dyadic negative_product = exact_multiply(negative, one, work);
    const Dyadic tiny = decode_dyadic(ldexpq(ONE, -100), work);
    const Dyadic aligned = exact_add(one, tiny, work);
    const bool zero_valid = zero.numerator.valid
        && is_zero(zero.numerator) && zero.exponent == 0;
    const bool negative_valid = negative_product.numerator.valid
        && negative_product.numerator.negative
        && !is_zero(negative_product.numerator);
    const bool nonfinite_rejected = !nonfinite.numerator.valid;
    const bool alignment_valid = aligned.numerator.valid
        && !is_zero(aligned.numerator)
        && work[r63zp_format::KwAlignmentShifts] > 0U;
    Magnitude capacity;
    capacity.used = LIMBS;
    capacity.word[LIMBS - 1U] = 1U;
    const bool capacity_rejected = !shift_left(capacity, 1U, work)
        && !capacity.valid;
    const bool success = zero_valid && negative_valid
        && nonfinite_rejected && alignment_valid && capacity_rejected;
    std::printf(
        "{\"zero_valid\":%s,\"negative_valid\":%s,"
        "\"nonfinite_rejected\":%s,\"alignment_valid\":%s,"
        "\"capacity_rejected\":%s,\"dyadic_decodes\":%llu,"
        "\"exact_multiplies\":%llu,\"exact_additions\":%llu,"
        "\"alignment_shifts\":%llu,\"alignment_bits\":%llu,"
        "\"capacity_predicates\":%llu,\"package_allocations\":0}\n",
        zero_valid ? "true" : "false",
        negative_valid ? "true" : "false",
        nonfinite_rejected ? "true" : "false",
        alignment_valid ? "true" : "false",
        capacity_rejected ? "true" : "false",
        static_cast<unsigned long long>(
            work[r63zp_format::KwDyadicDecodes]),
        static_cast<unsigned long long>(
            work[r63zp_format::KwExactMultiplies]),
        static_cast<unsigned long long>(
            work[r63zp_format::KwExactAdditions]),
        static_cast<unsigned long long>(
            work[r63zp_format::KwAlignmentShifts]),
        static_cast<unsigned long long>(
            work[r63zp_format::KwAlignmentBits]),
        static_cast<unsigned long long>(
            work[r63zp_format::KwBigintCapacityPredicates]));
    return success ? 0 : 1;
}

} // namespace

int main(int argc, char** argv) {
    if (argc == 2 && std::strcmp(argv[1], "--exact-controls") == 0)
        return run_exact_controls();
    if (argc != 6) return 64;

    CheckerWork work{};
    ++work[r63zp_format::KwRoundingModeSetCalls];
    const bool rounding_set = std::fesetround(FE_TONEAREST) == 0;
    ++work[r63zp_format::KwRoundingModeChecks];
    const int observed_rounding = std::fegetround();
    const bool rounding_observed = observed_rounding == FE_TONEAREST;
    const bool rounding = rounding_set && rounding_observed;

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
    CheckerRoute route = CheckerRoute::CheckerApparatusRejected;
    ++work[r63zp_format::KwRoutePredicates];
    if (!candidate.header_valid) {
        route = CheckerRoute::CandidateMalformed;
    } else {
        ++work[r63zp_format::KwRoutePredicates];
        if (!rounding) {
            route = CheckerRoute::CheckerApparatusRejected;
        } else {
            replay = derive_replay(cache, parent, parent_audit,
                candidate_file, candidate, work, root_calls, root_bytes);
            ++work[r63zp_format::KwRoutePredicates];
            route = replay.outcome;
        }
    }

    std::uint32_t flags = candidate.header_valid
        ? candidate.selector << 8U : 0U;
    if (replay.semantic_verified) flags |= 1U << 0U;
    if (replay.contained) flags |= 1U << 1U;
    if (replay.curvature) flags |= 1U << 2U;
    if (replay.step) flags |= 1U << 3U;
    if (replay.update) flags |= 1U << 4U;

    std::array<Digest, 4U> candidate_roots{{candidate_file.root,
        candidate.result, candidate.work_root, replay.exact_product_root}};
    const std::array<Digest, 3U> parent_roots{{cache.root, parent.root,
        parent_audit.root}};

    work[r63zp_format::KwAuditZeroFillBytes] = r63zp_format::checker_bytes;
    work[r63zp_format::KwAuditFieldsSerialized] = 95U;
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
        const bool equal = i < replay.compared_events
            && same_digest(candidate.events[i], replay.expected.events[i]);
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
    return static_cast<std::uint32_t>(route) <= 7U ? 0 : 1;
}
