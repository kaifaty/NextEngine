#include "../r63zm/fixed_sha256.hpp"

#include <algorithm>
#include <array>
#include <cfenv>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <quadmath.h>

namespace {

__extension__ using Quad = __float128;
using r63zm::Digest;

constexpr std::size_t CACHE_BYTES = 1033625U;
constexpr std::size_t ARTIFACT_BYTES = 12916U;
constexpr std::size_t AUDIT_BYTES = 420U;
constexpr std::size_t ROWS = 102U;
constexpr std::size_t COLUMNS = 315U;
constexpr std::size_t FACTOR_ENTRIES = ROWS * ROWS;
constexpr std::size_t PRODUCT_BASE = 1236U;
constexpr std::size_t PRODUCT_BYTES = 1944U;
constexpr std::size_t PRODUCT_ROOTS = 144U;
constexpr std::size_t PRODUCT_VALUES = 312U;
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

constexpr const char* CACHE_SHA256 =
    "23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84";
constexpr const char* ARTIFACT_SHA256 =
    "ac6946e872799baef366d8a6648e7bb5cd70c6f2acc326747fdf153c471f0b87";
constexpr const char* AUDIT_SHA256 =
    "fd4bcf0094e9f6ab8c64080c3a22566e3a23d2544e187641075350519a281f80";
constexpr char VECTOR_DOMAIN[] =
    "nextengine.nonlocal.r63zm.binary128-vector.v1";

static_assert(sizeof(Quad) == 16U);
static_assert(__FLT128_MANT_DIG__ == 113);
static_assert(__FLT128_MAX_EXP__ == 16384);
static_assert(PRODUCT_BASE + 6U * PRODUCT_BYTES + 16U == ARTIFACT_BYTES);
static_assert(CACHE_X0 + ROWS * sizeof(Quad) + 8U == CACHE_X1);
static_assert(CACHE_X1 + ROWS * sizeof(Quad) < CACHE_TANGENT);
static_assert(CACHE_TANGENT + ROWS * COLUMNS * sizeof(Quad) == CACHE_SIGMA);
static_assert(CACHE_SIGMA + sizeof(Quad) + 80U == CACHE_FACTOR);
static_assert(CACHE_FACTOR + FACTOR_ENTRIES * sizeof(double) + 8U
    == CACHE_PERMUTATION);
static_assert(CACHE_PERMUTATION + ROWS * sizeof(std::uint64_t)
    == CACHE_INVERSE);

struct Work final {
    std::uint64_t file_reads = 0U;
    std::uint64_t file_bytes = 0U;
    std::uint64_t file_hashes = 0U;
    std::uint64_t state_products = 0U;
    std::uint64_t state_inner_terms = 0U;
    std::uint64_t state_outer_terms = 0U;
    std::uint64_t factor_solves = 0U;
    std::uint64_t factor_terms = 0U;
    std::uint64_t update_cells = 0U;
    std::uint64_t zero_direction_cells = 0U;
    std::uint64_t dense_dots = 0U;
    std::uint64_t dense_terms = 0U;
    std::uint64_t dense_bound_terms = 0U;
    std::uint64_t rectangular_inner_terms = 0U;
    std::uint64_t rectangular_outer_terms = 0U;
    std::uint64_t oracle_products = 0U;
    std::uint64_t oracle_terms = 0U;
    std::uint64_t oracle_update_dots = 0U;
    std::uint64_t interval_divisions = 0U;
    std::uint64_t denominator_terms = 0U;
};

template <std::size_t Size>
bool read_exact(const char* path, std::array<std::uint8_t, Size>& bytes,
    Digest& root, Work& work) noexcept {
    ++work.file_reads;
    std::FILE* file = std::fopen(path, "rb");
    if (file == nullptr) return false;
    const std::size_t count = std::fread(bytes.data(), 1U, bytes.size(), file);
    const int trailing = std::fgetc(file);
    const int closed = std::fclose(file);
    if (count != bytes.size() || trailing != EOF || closed != 0) return false;
    work.file_bytes += bytes.size();
    ++work.file_hashes;
    root = r63zm::sha256(bytes);
    return true;
}

std::uint8_t nibble(char value) noexcept {
    if (value >= '0' && value <= '9')
        return static_cast<std::uint8_t>(value - '0');
    if (value >= 'a' && value <= 'f')
        return static_cast<std::uint8_t>(value - 'a' + 10);
    return 0xffU;
}

bool digest_is(const Digest& digest, const char* expected) noexcept {
    for (std::size_t index = 0U; index < digest.size(); ++index) {
        const std::uint8_t high = nibble(expected[2U * index]);
        const std::uint8_t low = nibble(expected[2U * index + 1U]);
        if (high > 15U || low > 15U
            || digest[index] != static_cast<std::uint8_t>((high << 4U) | low))
            return false;
    }
    return expected[64U] == '\0';
}

Quad little_quad(const std::uint8_t* bytes) noexcept {
    Quad value = ZERO;
    std::memcpy(&value, bytes, sizeof(value));
    return value;
}

Quad big_quad(const std::uint8_t* bytes) noexcept {
    std::array<std::uint8_t, sizeof(Quad)> little{};
    for (std::size_t index = 0U; index < little.size(); ++index)
        little[index] = bytes[little.size() - 1U - index];
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
    for (std::size_t index = 0U; index < result.size(); ++index)
        result[index] = little[result.size() - 1U - index];
    return result;
}

void hash_be64(r63zm::Sha256& hash, std::uint64_t value) noexcept {
    std::array<std::uint8_t, 8U> bytes{};
    for (std::size_t index = 0U; index < bytes.size(); ++index)
        bytes[index] = static_cast<std::uint8_t>(
            value >> (56U - 8U * index));
    hash.update(bytes);
}

void tlv_header(r63zm::Sha256& hash, std::uint8_t tag,
    std::uint64_t bytes) noexcept {
    hash.update_byte(tag);
    hash_be64(hash, bytes);
}

template <std::size_t Count>
Digest vector_root(const std::array<Quad, Count>& values) noexcept {
    r63zm::Sha256 hash;
    tlv_header(hash, 1U, sizeof(VECTOR_DOMAIN) - 1U);
    hash.update(std::span<const std::uint8_t>(
        reinterpret_cast<const std::uint8_t*>(VECTOR_DOMAIN),
        sizeof(VECTOR_DOMAIN) - 1U));
    tlv_header(hash, 2U, 8U);
    hash_be64(hash, Count);
    tlv_header(hash, 3U, Count * sizeof(Quad));
    for (Quad value : values) hash.update(canonical(value));
    return hash.finish();
}

bool digest_bytes_equal(const Digest& digest, const std::uint8_t* bytes) noexcept {
    return std::memcmp(digest.data(), bytes, digest.size()) == 0;
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

Quad directed_add(Quad left, Quad right, int mode) noexcept {
    if (std::fesetround(mode) != 0) return nanq("");
    volatile Quad result = left + right;
    if (std::fesetround(FE_TONEAREST) != 0) return nanq("");
    return result;
}

Quad directed_subtract(Quad left, Quad right, int mode) noexcept {
    if (std::fesetround(mode) != 0) return nanq("");
    volatile Quad result = left - right;
    if (std::fesetround(FE_TONEAREST) != 0) return nanq("");
    return result;
}

Quad directed_multiply(Quad left, Quad right, int mode) noexcept {
    if (std::fesetround(mode) != 0) return nanq("");
    volatile Quad result = left * right;
    if (std::fesetround(FE_TONEAREST) != 0) return nanq("");
    return result;
}

Quad directed_divide(Quad left, Quad right, int mode) noexcept {
    if (std::fesetround(mode) != 0) return nanq("");
    volatile Quad result = left / right;
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
    Quad absolute = ZERO;
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
    for (std::size_t index = 1U; index < count; ++index) {
        const Pair product = exact_product(left(index), right(index));
        const Pair sum = exact_sum(primary, product.primary);
        const Quad local = sum.error + product.error;
        correction += local;
        primary = sum.primary;
        exact = exact && product.exact && sum.exact;
        normal = normal && product.normal && sum.normal
            && normal_or_zero(local) && normal_or_zero(correction);
        absolute = up_add(absolute,
            up_multiply(fabsq(left(index)), fabsq(right(index))));
    }
    output.value = primary + correction;
    normal = normal && normal_or_zero(output.value);
    const Quad unit = ldexpq(ONE, -112);
    const Quad count_unit = static_cast<Quad>(count) * unit;
    const Quad gamma = up_divide(count_unit, ONE - count_unit);
    const Quad numerator = up_add(
        up_multiply(unit, fabsq(output.value)),
        up_multiply(up_multiply(gamma, gamma), absolute));
    output.bound = up_divide(numerator, ONE - unit);
    output.absolute = absolute;
    output.exact = exact && finiteq(output.value) != 0
        && finiteq(output.bound) != 0 && output.bound >= ZERO;
    output.normal = normal;
    return output;
}

struct Product final {
    bool exact = false;
    std::array<Quad, COLUMNS> intermediate{};
    std::array<Quad, COLUMNS> intermediate_bound{};
    std::array<Quad, ROWS> value{};
    std::array<Quad, ROWS> bound{};
};

Product tangent_product(const std::array<Quad, ROWS * COLUMNS>& tangent,
    Quad sigma, const std::array<Quad, ROWS>& input, Work& work,
    bool oracle) noexcept {
    Product output;
    bool exact = true;
    for (std::size_t column = 0U; column < COLUMNS; ++column) {
        const Dot dot = dot2(ROWS,
            [&](std::size_t row) { return tangent[row * COLUMNS + column]; },
            [&](std::size_t row) { return input[row]; });
        output.intermediate[column] = dot.value;
        output.intermediate_bound[column] = dot.bound;
        exact = exact && dot.exact && dot.normal;
    }
    for (std::size_t row = 0U; row < ROWS; ++row) {
        const Dot dot = dot2(COLUMNS,
            [&](std::size_t column) {
                return tangent[row * COLUMNS + column];
            },
            [&](std::size_t column) { return output.intermediate[column]; });
        Quad propagation = dot.bound;
        for (std::size_t column = 0U; column < COLUMNS; ++column)
            propagation = up_add(propagation, up_multiply(
                fabsq(tangent[row * COLUMNS + column]),
                output.intermediate_bound[column]));
        const Pair scale = exact_product(sigma, dot.value);
        output.value[row] = scale.primary;
        output.bound[row] = up_add(fabsq(scale.error),
            up_multiply(fabsq(sigma), propagation));
        exact = exact && dot.exact && dot.normal && scale.exact && scale.normal
            && finiteq(output.value[row]) != 0
            && finiteq(output.bound[row]) != 0;
    }
    output.exact = exact;
    if (oracle) {
        ++work.oracle_products;
        work.oracle_terms += 2U * ROWS * COLUMNS;
    } else {
        ++work.state_products;
        work.state_inner_terms += ROWS * COLUMNS;
        work.state_outer_terms += ROWS * COLUMNS;
    }
    return output;
}

struct Accumulator final {
    Quad sum = ZERO;
    Quad correction = ZERO;

    void add(Quad value) noexcept {
        const Quad next = sum + value;
        correction += fabsq(sum) >= fabsq(value)
            ? (sum - next) + value
            : (value - next) + sum;
        sum = next;
    }
    Quad value() const noexcept { return sum + correction; }
};

struct Solve final {
    bool exact = false;
    std::uint64_t terms = 0U;
    std::array<Quad, ROWS> scaled{};
    std::array<Quad, ROWS> forward{};
    std::array<Quad, ROWS> backward{};
    std::array<Quad, ROWS> solution{};
};

Solve solve(const std::array<Quad, FACTOR_ENTRIES>& upper,
    const std::array<std::uint64_t, ROWS>& permutation,
    const std::array<Quad, ROWS>& rhs, Quad inverse, Work& work) noexcept {
    Solve output;
    bool valid = inverse > ZERO && finiteq(inverse) != 0;
    for (std::size_t row = 0U; row < ROWS && valid; ++row) {
        valid = permutation[row] < ROWS;
        if (!valid) break;
        output.scaled[row] = rhs[permutation[row]] * inverse;
        Accumulator dot;
        for (std::size_t column = 0U; column < row; ++column) {
            dot.add(upper[column * ROWS + row] * output.forward[column]);
            ++output.terms;
        }
        const Quad diagonal = upper[row * ROWS + row];
        valid = diagonal != ZERO && finiteq(diagonal) != 0;
        if (!valid) break;
        output.forward[row] = (output.scaled[row] - dot.value()) / diagonal;
        valid = finiteq(output.forward[row]) != 0;
    }
    for (std::size_t remaining = valid ? ROWS : 0U;
         remaining > 0U && valid;) {
        const std::size_t row = --remaining;
        Accumulator dot;
        for (std::size_t column = row + 1U; column < ROWS; ++column) {
            dot.add(upper[row * ROWS + column] * output.backward[column]);
            ++output.terms;
        }
        const Quad diagonal = upper[row * ROWS + row];
        valid = diagonal != ZERO && finiteq(diagonal) != 0;
        if (!valid) break;
        output.backward[row] = (output.forward[row] - dot.value()) / diagonal;
        valid = finiteq(output.backward[row]) != 0;
    }
    for (std::size_t index = 0U; index < ROWS; ++index)
        if (permutation[index] < ROWS)
            output.solution[permutation[index]] = output.backward[index];
    output.exact = valid && output.terms == 10302U;
    ++work.factor_solves;
    work.factor_terms += output.terms;
    return output;
}

struct Interval final {
    Quad lower = ZERO;
    Quad upper = ZERO;
    bool valid = false;
};

Interval intersect(Interval left, Interval right) noexcept {
    if (!left.valid) return right;
    if (!right.valid) return left;
    Interval output{std::max(left.lower, right.lower),
        std::min(left.upper, right.upper), true};
    output.valid = output.lower <= output.upper;
    return output;
}

Interval outer_rounding_cell(Quad value) noexcept {
    if (finiteq(value) == 0) return {};
    const Quad previous = nextafterq(value, -HUGE_VALQ);
    const Quad following = nextafterq(value, HUGE_VALQ);
    const Quad lower_sum = directed_add(previous, value, FE_DOWNWARD);
    const Quad upper_sum = directed_add(value, following, FE_UPWARD);
    return {directed_divide(lower_sum, static_cast<Quad>(2.0), FE_DOWNWARD),
        directed_divide(upper_sum, static_cast<Quad>(2.0), FE_UPWARD), true};
}

Interval alpha_constraint(Quad x0, Quad p0, Quad x1, Work& work) noexcept {
    ++work.update_cells;
    const Interval cell = outer_rounding_cell(x1);
    if (!cell.valid) return {};
    if (p0 == ZERO) {
        ++work.zero_direction_cells;
        return x0 >= cell.lower && x0 <= cell.upper
            ? Interval{-HUGE_VALQ, HUGE_VALQ, true} : Interval{};
    }
    ++work.interval_divisions;
    if (p0 > ZERO) {
        return {directed_divide(directed_subtract(cell.lower, x0,
                    FE_DOWNWARD), p0, FE_DOWNWARD),
            directed_divide(directed_subtract(cell.upper, x0,
                    FE_UPWARD), p0, FE_UPWARD), true};
    }
    return {directed_divide(directed_subtract(cell.upper, x0,
                FE_UPWARD), p0, FE_DOWNWARD),
        directed_divide(directed_subtract(cell.lower, x0,
                FE_DOWNWARD), p0, FE_UPWARD), true};
}

Quad delta_upper(Quad x0, Quad p0, Quad x1,
    const Interval& alpha) noexcept {
    Quad product_lower = ZERO;
    Quad product_upper = ZERO;
    if (p0 >= ZERO) {
        product_lower = directed_multiply(alpha.lower, p0, FE_DOWNWARD);
        product_upper = directed_multiply(alpha.upper, p0, FE_UPWARD);
    } else {
        product_lower = directed_multiply(alpha.upper, p0, FE_DOWNWARD);
        product_upper = directed_multiply(alpha.lower, p0, FE_UPWARD);
    }
    const Quad update_lower = directed_add(x0, product_lower, FE_DOWNWARD);
    const Quad update_upper = directed_add(x0, product_upper, FE_UPWARD);
    const Quad delta_lower = directed_subtract(x1, update_upper, FE_DOWNWARD);
    const Quad delta_upper_value = directed_subtract(x1, update_lower,
        FE_UPWARD);
    return std::max(fabsq(delta_lower), fabsq(delta_upper_value));
}

std::array<Quad, ROWS> dense_delta_bound(
    const std::array<Quad, ROWS * COLUMNS>& tangent, Quad sigma,
    const std::array<Quad, ROWS>& delta, Work& work) noexcept {
    std::array<Quad, ROWS * ROWS> absolute_operator{};
    for (std::size_t row = 0U; row < ROWS; ++row) {
        for (std::size_t column = row; column < ROWS; ++column) {
            const Dot dot = dot2(COLUMNS,
                [&](std::size_t index) {
                    return tangent[row * COLUMNS + index];
                },
                [&](std::size_t index) {
                    return tangent[column * COLUMNS + index];
                });
            const Pair scaled = exact_product(sigma, dot.value);
            const Quad bound = up_add(fabsq(scaled.error),
                up_multiply(fabsq(sigma), dot.bound));
            const Quad absolute = up_add(fabsq(scaled.primary), bound);
            absolute_operator[row * ROWS + column] = absolute;
            absolute_operator[column * ROWS + row] = absolute;
            ++work.dense_dots;
            work.dense_terms += COLUMNS;
        }
    }
    std::array<Quad, ROWS> output{};
    for (std::size_t row = 0U; row < ROWS; ++row) {
        Quad bound = ZERO;
        for (std::size_t column = 0U; column < ROWS; ++column) {
            bound = up_add(bound, up_multiply(
                absolute_operator[row * ROWS + column], delta[column]));
            ++work.dense_bound_terms;
        }
        output[row] = bound;
    }
    return output;
}

std::array<Quad, ROWS> rectangular_delta_bound(
    const std::array<Quad, ROWS * COLUMNS>& tangent, Quad sigma,
    const std::array<Quad, ROWS>& delta, Work& work) noexcept {
    std::array<Quad, COLUMNS> intermediate{};
    for (std::size_t column = 0U; column < COLUMNS; ++column) {
        Quad bound = ZERO;
        for (std::size_t row = 0U; row < ROWS; ++row) {
            bound = up_add(bound, up_multiply(
                fabsq(tangent[row * COLUMNS + column]), delta[row]));
            ++work.rectangular_inner_terms;
        }
        intermediate[column] = bound;
    }
    std::array<Quad, ROWS> output{};
    for (std::size_t row = 0U; row < ROWS; ++row) {
        Quad bound = ZERO;
        for (std::size_t column = 0U; column < COLUMNS; ++column) {
            bound = up_add(bound, up_multiply(
                fabsq(tangent[row * COLUMNS + column]),
                intermediate[column]));
            ++work.rectangular_outer_terms;
        }
        output[row] = up_multiply(fabsq(sigma), bound);
    }
    return output;
}

Interval divide_by_positive_interval(Interval numerator, Interval denominator,
    Work& work) noexcept {
    if (!numerator.valid || !denominator.valid || denominator.lower <= ZERO)
        return {};
    std::array<Quad, 4U> lower{{
        directed_divide(numerator.lower, denominator.lower, FE_DOWNWARD),
        directed_divide(numerator.lower, denominator.upper, FE_DOWNWARD),
        directed_divide(numerator.upper, denominator.lower, FE_DOWNWARD),
        directed_divide(numerator.upper, denominator.upper, FE_DOWNWARD)}};
    std::array<Quad, 4U> upper{{
        directed_divide(numerator.lower, denominator.lower, FE_UPWARD),
        directed_divide(numerator.lower, denominator.upper, FE_UPWARD),
        directed_divide(numerator.upper, denominator.lower, FE_UPWARD),
        directed_divide(numerator.upper, denominator.upper, FE_UPWARD)}};
    work.interval_divisions += 8U;
    return {*std::min_element(lower.begin(), lower.end()),
        *std::max_element(upper.begin(), upper.end()), true};
}

struct Envelope final {
    bool finite = true;
    bool oracle_contained = true;
    bool denominator_positive = false;
    bool step_subset = false;
    std::size_t worst_component = 0U;
    Quad maximum_radius = ZERO;
    Interval denominator;
    Interval step;
};

Envelope evaluate_envelope(const std::array<Quad, ROWS>& y0,
    const std::array<Quad, ROWS>& e0,
    const std::array<Quad, ROWS>& y1,
    const std::array<Quad, ROWS>& e1,
    const std::array<Quad, ROWS>& hdelta,
    const std::array<Quad, ROWS>& p0, Quad rho0,
    const Interval& alpha, const Product& oracle, Work& work) noexcept {
    Envelope output;
    std::array<Interval, ROWS> q{};
    for (std::size_t row = 0U; row < ROWS; ++row) {
        const Quad error = up_add(up_add(e0[row], e1[row]), hdelta[row]);
        const Quad center = y1[row] - y0[row];
        const Interval numerator{
            directed_subtract(center, error, FE_DOWNWARD),
            directed_add(center, error, FE_UPWARD), true};
        q[row] = divide_by_positive_interval(numerator, alpha, work);
        const Quad oracle_lower = directed_subtract(oracle.value[row],
            oracle.bound[row], FE_DOWNWARD);
        const Quad oracle_upper = directed_add(oracle.value[row],
            oracle.bound[row], FE_UPWARD);
        output.oracle_contained = output.oracle_contained && q[row].valid
            && q[row].lower <= oracle_lower && q[row].upper >= oracle_upper;
        output.finite = output.finite && q[row].valid
            && finiteq(q[row].lower) != 0 && finiteq(q[row].upper) != 0;
        const Quad radius = up_divide(
            directed_subtract(q[row].upper, q[row].lower, FE_UPWARD),
            static_cast<Quad>(2.0));
        if (radius > output.maximum_radius) {
            output.maximum_radius = radius;
            output.worst_component = row;
        }
    }

    Interval denominator{ZERO, ZERO, true};
    Quad absolute_products = ZERO;
    for (std::size_t row = 0U; row < ROWS; ++row) {
        Quad lower = ZERO;
        Quad upper = ZERO;
        if (p0[row] >= ZERO) {
            lower = directed_multiply(p0[row], q[row].lower, FE_DOWNWARD);
            upper = directed_multiply(p0[row], q[row].upper, FE_UPWARD);
        } else {
            lower = directed_multiply(p0[row], q[row].upper, FE_DOWNWARD);
            upper = directed_multiply(p0[row], q[row].lower, FE_UPWARD);
        }
        denominator.lower = directed_add(denominator.lower, lower, FE_DOWNWARD);
        denominator.upper = directed_add(denominator.upper, upper, FE_UPWARD);
        const Quad q_absolute = std::max(fabsq(q[row].lower),
            fabsq(q[row].upper));
        absolute_products = up_add(absolute_products,
            up_multiply(fabsq(p0[row]), q_absolute));
        ++work.denominator_terms;
    }
    const Quad unit = ldexpq(ONE, -112);
    const Quad count_unit = static_cast<Quad>(ROWS) * unit;
    const Quad gamma = up_divide(count_unit, ONE - count_unit);
    const Quad maximum_value = std::max(fabsq(denominator.lower),
        fabsq(denominator.upper));
    const Quad rounding = up_divide(up_add(
        up_multiply(unit, maximum_value),
        up_multiply(up_multiply(gamma, gamma), absolute_products)),
        ONE - unit);
    denominator.lower = directed_subtract(denominator.lower, rounding,
        FE_DOWNWARD);
    denominator.upper = directed_add(denominator.upper, rounding, FE_UPWARD);
    output.denominator = denominator;
    output.denominator_positive = denominator.lower > ZERO;
    if (output.denominator_positive) {
        output.step = {directed_divide(rho0, denominator.upper, FE_DOWNWARD),
            directed_divide(rho0, denominator.lower, FE_UPWARD), true};
        work.interval_divisions += 2U;
        output.step_subset = output.step.lower >= alpha.lower
            && output.step.upper <= alpha.upper;
    }
    return output;
}

void print_quad(const char* name, Quad value) noexcept {
    std::array<char, 160U> text{};
    quadmath_snprintf(text.data(), text.size(), "%+-#46Qa", value);
    std::printf("%s=%s\n", name, text.data());
}

void print_interval(const char* name, Interval value) noexcept {
    std::array<char, 160U> lower{};
    std::array<char, 160U> upper{};
    quadmath_snprintf(lower.data(), lower.size(), "%+-#46Qa", value.lower);
    quadmath_snprintf(upper.data(), upper.size(), "%+-#46Qa", value.upper);
    std::printf("%s_lower=%s\n%s_upper=%s\n", name, lower.data(), name,
        upper.data());
}

} // namespace

int main(int argc, char** argv) {
    if (argc != 4) return 64;
    if (std::fesetround(FE_TONEAREST) != 0 || std::fegetround() != FE_TONEAREST)
        return 65;

    Work work;
    std::array<std::uint8_t, CACHE_BYTES> cache{};
    std::array<std::uint8_t, ARTIFACT_BYTES> artifact{};
    std::array<std::uint8_t, AUDIT_BYTES> audit{};
    Digest cache_root{};
    Digest artifact_root{};
    Digest audit_root{};
    if (!read_exact(argv[1], cache, cache_root, work)
        || !read_exact(argv[2], artifact, artifact_root, work)
        || !read_exact(argv[3], audit, audit_root, work)
        || !digest_is(cache_root, CACHE_SHA256)
        || !digest_is(artifact_root, ARTIFACT_SHA256)
        || !digest_is(audit_root, AUDIT_SHA256))
        return 66;

    std::array<Quad, ROWS> x0{};
    std::array<Quad, ROWS> x1{};
    std::array<Quad, ROWS> rhs{};
    std::array<Quad, ROWS * COLUMNS> tangent{};
    std::array<Quad, FACTOR_ENTRIES> factor{};
    std::array<std::uint64_t, ROWS> permutation{};
    bool inputs_valid = true;
    for (std::size_t row = 0U; row < ROWS; ++row) {
        x0[row] = little_quad(cache.data() + CACHE_X0 + row * sizeof(Quad));
        x1[row] = little_quad(cache.data() + CACHE_X1 + row * sizeof(Quad));
        rhs[row] = little_quad(cache.data() + CACHE_RHS + row * sizeof(Quad));
        permutation[row] = little_u64(cache.data() + CACHE_PERMUTATION
            + row * sizeof(std::uint64_t));
        inputs_valid = inputs_valid && finiteq(x0[row]) != 0
            && finiteq(x1[row]) != 0 && finiteq(rhs[row]) != 0
            && permutation[row] < ROWS;
    }
    for (std::size_t index = 0U; index < tangent.size(); ++index) {
        tangent[index] = little_quad(cache.data() + CACHE_TANGENT
            + index * sizeof(Quad));
        inputs_valid = inputs_valid && finiteq(tangent[index]) != 0;
    }
    for (std::size_t index = 0U; index < factor.size(); ++index) {
        factor[index] = static_cast<Quad>(little_double(cache.data()
            + CACHE_FACTOR + index * sizeof(double)));
        inputs_valid = inputs_valid && finiteq(factor[index]) != 0;
    }
    const Quad sigma = little_quad(cache.data() + CACHE_SIGMA);
    const Quad inverse = little_quad(cache.data() + CACHE_INVERSE);
    inputs_valid = inputs_valid && sigma > ZERO && inverse > ZERO
        && finiteq(sigma) != 0 && finiteq(inverse) != 0;
    if (!inputs_valid) return 67;

    const Solve start = solve(factor, permutation, rhs, inverse, work);
    std::size_t x0_matches = 0U;
    for (std::size_t row = 0U; row < ROWS; ++row)
        x0_matches += std::memcmp(&start.solution[row], &x0[row],
            sizeof(Quad)) == 0 ? 1U : 0U;
    if (!start.exact || x0_matches != ROWS) return 68;

    std::array<Quad, ROWS> artifact_y0{};
    std::array<Quad, ROWS> artifact_y1{};
    const std::size_t role2 = PRODUCT_BASE + 2U * PRODUCT_BYTES;
    const std::size_t role3 = PRODUCT_BASE + 3U * PRODUCT_BYTES;
    for (std::size_t row = 0U; row < ROWS; ++row) {
        artifact_y0[row] = big_quad(artifact.data() + role2 + PRODUCT_VALUES
            + row * sizeof(Quad));
        artifact_y1[row] = big_quad(artifact.data() + role3 + PRODUCT_VALUES
            + row * sizeof(Quad));
    }
    const Product state0 = tangent_product(tangent, sigma, x0, work, false);
    const Product state1 = tangent_product(tangent, sigma, x1, work, false);
    bool state_products_exact = state0.exact && state1.exact;
    for (std::size_t row = 0U; row < ROWS; ++row)
        state_products_exact = state_products_exact
            && std::memcmp(&state0.value[row], &artifact_y0[row], sizeof(Quad))
                == 0
            && std::memcmp(&state1.value[row], &artifact_y1[row], sizeof(Quad))
                == 0;
    const Digest state0_bound_root = vector_root(state0.bound);
    const Digest state1_bound_root = vector_root(state1.bound);
    state_products_exact = state_products_exact
        && digest_bytes_equal(state0_bound_root,
            artifact.data() + role2 + PRODUCT_ROOTS + 3U * 32U)
        && digest_bytes_equal(state1_bound_root,
            artifact.data() + role3 + PRODUCT_ROOTS + 3U * 32U);
    if (!state_products_exact) return 69;

    std::array<Quad, ROWS> residual{};
    for (std::size_t row = 0U; row < ROWS; ++row) {
        const std::array<Quad, 2U> left{{rhs[row], -artifact_y0[row]}};
        const Dot dot = dot2(2U,
            [&](std::size_t index) { return left[index]; },
            [&](std::size_t) { return ONE; });
        if (!dot.exact || !dot.normal) return 70;
        residual[row] = dot.value;
    }
    const Solve preconditioned = solve(factor, permutation, residual, inverse,
        work);
    if (!preconditioned.exact) return 71;
    const std::array<Quad, ROWS>& p0 = preconditioned.solution;
    const Dot rho = dot2(ROWS,
        [&](std::size_t row) { return residual[row]; },
        [&](std::size_t row) { return p0[row]; });
    if (!rho.exact || !rho.normal || rho.value - rho.bound <= ZERO) return 72;

    Interval alpha{};
    for (std::size_t row = 0U; row < ROWS; ++row) {
        const Interval local = alpha_constraint(x0[row], p0[row], x1[row],
            work);
        if (!local.valid) return 73;
        alpha = intersect(alpha, local);
        if (!alpha.valid) return 73;
    }
    if (alpha.lower <= ZERO || finiteq(alpha.lower) == 0
        || finiteq(alpha.upper) == 0)
        return 73;

    std::array<Quad, ROWS> delta{};
    for (std::size_t row = 0U; row < ROWS; ++row)
        delta[row] = delta_upper(x0[row], p0[row], x1[row], alpha);
    const auto dense = dense_delta_bound(tangent, sigma, delta, work);
    const auto rectangular = rectangular_delta_bound(tangent, sigma, delta,
        work);
    Quad maximum_state_product_error = ZERO;
    Quad maximum_dense_delta_error = ZERO;
    Quad maximum_rectangular_delta_error = ZERO;
    Quad state_denominator_uncertainty = ZERO;
    Quad dense_delta_denominator_uncertainty = ZERO;
    Quad rectangular_delta_denominator_uncertainty = ZERO;
    for (std::size_t row = 0U; row < ROWS; ++row) {
        const Quad state_error = up_add(state0.bound[row], state1.bound[row]);
        maximum_state_product_error = std::max(maximum_state_product_error,
            state_error);
        maximum_dense_delta_error = std::max(maximum_dense_delta_error,
            dense[row]);
        maximum_rectangular_delta_error = std::max(
            maximum_rectangular_delta_error, rectangular[row]);
        state_denominator_uncertainty = up_add(
            state_denominator_uncertainty,
            up_multiply(fabsq(p0[row]), state_error));
        dense_delta_denominator_uncertainty = up_add(
            dense_delta_denominator_uncertainty,
            up_multiply(fabsq(p0[row]), dense[row]));
        rectangular_delta_denominator_uncertainty = up_add(
            rectangular_delta_denominator_uncertainty,
            up_multiply(fabsq(p0[row]), rectangular[row]));
    }
    state_denominator_uncertainty = up_divide(
        state_denominator_uncertainty, alpha.lower);
    dense_delta_denominator_uncertainty = up_divide(
        dense_delta_denominator_uncertainty, alpha.lower);
    rectangular_delta_denominator_uncertainty = up_divide(
        rectangular_delta_denominator_uncertainty, alpha.lower);

    const Product oracle = tangent_product(tangent, sigma, p0, work, true);
    if (!oracle.exact) return 74;
    const Dot oracle_denominator = dot2(ROWS,
        [&](std::size_t row) { return p0[row]; },
        [&](std::size_t row) { return oracle.value[row]; });
    if (!oracle_denominator.exact || !oracle_denominator.normal
        || oracle_denominator.value - oracle_denominator.bound <= ZERO)
        return 74;
    const Quad oracle_alpha = rho.value / oracle_denominator.value;
    std::size_t oracle_updates = 0U;
    for (std::size_t row = 0U; row < ROWS; ++row) {
        const Dot update = dot2(2U,
            [&](std::size_t index) {
                return index == 0U ? x0[row] : oracle_alpha;
            },
            [&](std::size_t index) {
                return index == 0U ? ONE : p0[row];
            });
        ++work.oracle_update_dots;
        oracle_updates += update.exact && update.normal
            && std::memcmp(&update.value, &x1[row], sizeof(Quad)) == 0
            ? 1U : 0U;
    }

    const Envelope tight = evaluate_envelope(artifact_y0, state0.bound,
        artifact_y1, state1.bound, dense, p0, rho.value, alpha, oracle, work);
    const Envelope loose = evaluate_envelope(artifact_y0, state0.bound,
        artifact_y1, state1.bound, rectangular, p0, rho.value, alpha, oracle,
        work);
    const bool apparatus = tight.finite && loose.finite
        && tight.oracle_contained && loose.oracle_contained
        && oracle_updates == ROWS;
    const bool tight_pass = apparatus && tight.denominator_positive
        && tight.step_subset;
    const bool rectangular_pass = apparatus && loose.denominator_positive
        && loose.step_subset;
    const char* route = !apparatus ? "R63ZO_APPARATUS_REJECTED"
        : !tight_pass ? "TIGHT_PRODUCT_ENCLOSURE_REJECTED"
        : !rectangular_pass ? "DENSE_ABSOLUTE_ENVELOPE_REQUIRED"
        : "RECTANGULAR_ROUNDING_ENVELOPE_CANDIDATE";

    std::printf("route=%s\n", route);
    std::printf("x0_matches=%zu\n", x0_matches);
    std::printf("state_products_exact=%d\n", state_products_exact ? 1 : 0);
    std::printf("oracle_updates=%zu\n", oracle_updates);
    print_interval("alpha_preimage", alpha);
    print_quad("alpha_preimage_width",
        directed_subtract(alpha.upper, alpha.lower, FE_UPWARD));
    print_quad("rho0_value", rho.value);
    print_quad("rho0_bound", rho.bound);
    print_quad("oracle_alpha", oracle_alpha);
    print_quad("maximum_state_product_error",
        maximum_state_product_error);
    print_quad("maximum_dense_delta_error", maximum_dense_delta_error);
    print_quad("maximum_rectangular_delta_error",
        maximum_rectangular_delta_error);
    print_quad("state_denominator_uncertainty",
        state_denominator_uncertainty);
    print_quad("dense_delta_denominator_uncertainty",
        dense_delta_denominator_uncertainty);
    print_quad("rectangular_delta_denominator_uncertainty",
        rectangular_delta_denominator_uncertainty);
    std::printf("tight_oracle_contained=%d\n",
        tight.oracle_contained ? 1 : 0);
    std::printf("tight_denominator_positive=%d\n",
        tight.denominator_positive ? 1 : 0);
    std::printf("tight_step_subset=%d\n", tight.step_subset ? 1 : 0);
    std::printf("tight_worst_component=%zu\n", tight.worst_component);
    print_quad("tight_maximum_radius", tight.maximum_radius);
    print_interval("tight_denominator", tight.denominator);
    print_interval("tight_step", tight.step);
    std::printf("rectangular_oracle_contained=%d\n",
        loose.oracle_contained ? 1 : 0);
    std::printf("rectangular_denominator_positive=%d\n",
        loose.denominator_positive ? 1 : 0);
    std::printf("rectangular_step_subset=%d\n", loose.step_subset ? 1 : 0);
    std::printf("rectangular_worst_component=%zu\n", loose.worst_component);
    print_quad("rectangular_maximum_radius", loose.maximum_radius);
    print_interval("rectangular_denominator", loose.denominator);
    print_interval("rectangular_step", loose.step);
    std::printf("work_file_reads=%llu\n",
        static_cast<unsigned long long>(work.file_reads));
    std::printf("work_file_bytes=%llu\n",
        static_cast<unsigned long long>(work.file_bytes));
    std::printf("work_file_hashes=%llu\n",
        static_cast<unsigned long long>(work.file_hashes));
    std::printf("work_state_products=%llu\n",
        static_cast<unsigned long long>(work.state_products));
    std::printf("work_state_terms=%llu/%llu\n",
        static_cast<unsigned long long>(work.state_inner_terms),
        static_cast<unsigned long long>(work.state_outer_terms));
    std::printf("work_factor_solves=%llu\n",
        static_cast<unsigned long long>(work.factor_solves));
    std::printf("work_factor_terms=%llu\n",
        static_cast<unsigned long long>(work.factor_terms));
    std::printf("work_update_cells=%llu\n",
        static_cast<unsigned long long>(work.update_cells));
    std::printf("work_zero_direction_cells=%llu\n",
        static_cast<unsigned long long>(work.zero_direction_cells));
    std::printf("work_dense_dots=%llu\n",
        static_cast<unsigned long long>(work.dense_dots));
    std::printf("work_dense_terms=%llu\n",
        static_cast<unsigned long long>(work.dense_terms));
    std::printf("work_dense_bound_terms=%llu\n",
        static_cast<unsigned long long>(work.dense_bound_terms));
    std::printf("work_rectangular_terms=%llu/%llu\n",
        static_cast<unsigned long long>(work.rectangular_inner_terms),
        static_cast<unsigned long long>(work.rectangular_outer_terms));
    std::printf("work_oracle_products=%llu\n",
        static_cast<unsigned long long>(work.oracle_products));
    std::printf("work_oracle_terms=%llu\n",
        static_cast<unsigned long long>(work.oracle_terms));
    std::printf("work_oracle_update_dots=%llu\n",
        static_cast<unsigned long long>(work.oracle_update_dots));
    std::printf("work_interval_divisions=%llu\n",
        static_cast<unsigned long long>(work.interval_divisions));
    std::printf("work_denominator_terms=%llu\n",
        static_cast<unsigned long long>(work.denominator_terms));
    return apparatus ? 0 : 75;
}
