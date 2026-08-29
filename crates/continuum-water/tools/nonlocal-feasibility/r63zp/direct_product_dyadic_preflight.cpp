#define main r63zo_embedded_main
#include "../r63zo/rounded_update_preflight.cpp"
#undef main

#include <bit>
#include <string>
#include <vector>

namespace {

constexpr char EXACT_VECTOR_DOMAIN[] =
    "nextengine.nonlocal.r63zp.exact-dyadic-vector.v1";

struct ExactWork final {
    std::uint64_t decodes = 0U;
    std::uint64_t multiplies = 0U;
    std::uint64_t additions = 0U;
    std::uint64_t alignment_shifts = 0U;
    std::uint64_t alignment_bits = 0U;
    std::uint64_t interval_comparisons = 0U;
};

struct BigInt final {
    bool negative = false;
    std::vector<std::uint32_t> words;

    bool zero() const noexcept { return words.empty(); }
};

void normalize_big(BigInt& value) {
    while (!value.words.empty() && value.words.back() == 0U)
        value.words.pop_back();
    if (value.words.empty()) value.negative = false;
}

int compare_magnitude(const BigInt& left, const BigInt& right) {
    if (left.words.size() != right.words.size())
        return left.words.size() < right.words.size() ? -1 : 1;
    for (std::size_t remaining = left.words.size(); remaining > 0U;) {
        const std::size_t index = --remaining;
        if (left.words[index] != right.words[index])
            return left.words[index] < right.words[index] ? -1 : 1;
    }
    return 0;
}

BigInt add_magnitude(const BigInt& left, const BigInt& right) {
    BigInt result;
    result.words.resize(std::max(left.words.size(), right.words.size()) + 1U);
    std::uint64_t carry = 0U;
    for (std::size_t index = 0U; index + 1U < result.words.size(); ++index) {
        const std::uint64_t left_word = index < left.words.size()
            ? left.words[index] : 0U;
        const std::uint64_t right_word = index < right.words.size()
            ? right.words[index] : 0U;
        const std::uint64_t sum = left_word + right_word + carry;
        result.words[index] = static_cast<std::uint32_t>(sum);
        carry = sum >> 32U;
    }
    result.words.back() = static_cast<std::uint32_t>(carry);
    normalize_big(result);
    return result;
}

BigInt subtract_magnitude(const BigInt& left, const BigInt& right) {
    BigInt result;
    result.words.resize(left.words.size());
    std::uint64_t borrow = 0U;
    for (std::size_t index = 0U; index < left.words.size(); ++index) {
        const std::uint64_t left_word = left.words[index];
        const std::uint64_t right_word = index < right.words.size()
            ? right.words[index] : 0U;
        const std::uint64_t subtrahend = right_word + borrow;
        result.words[index] = static_cast<std::uint32_t>(
            left_word - subtrahend);
        borrow = left_word < subtrahend ? 1U : 0U;
    }
    normalize_big(result);
    return result;
}

BigInt add_big(const BigInt& left, const BigInt& right) {
    if (left.zero()) return right;
    if (right.zero()) return left;
    if (left.negative == right.negative) {
        BigInt result = add_magnitude(left, right);
        result.negative = left.negative;
        return result;
    }
    const int order = compare_magnitude(left, right);
    if (order == 0) return {};
    const BigInt& larger = order > 0 ? left : right;
    const BigInt& smaller = order > 0 ? right : left;
    BigInt result = subtract_magnitude(larger, smaller);
    result.negative = larger.negative;
    return result;
}

BigInt multiply_big(const BigInt& left, const BigInt& right) {
    if (left.zero() || right.zero()) return {};
    BigInt result;
    result.negative = left.negative != right.negative;
    result.words.assign(left.words.size() + right.words.size(), 0U);
    for (std::size_t left_index = 0U; left_index < left.words.size();
         ++left_index) {
        std::uint64_t carry = 0U;
        for (std::size_t right_index = 0U; right_index < right.words.size();
             ++right_index) {
            const std::size_t output = left_index + right_index;
            const std::uint64_t product =
                static_cast<std::uint64_t>(left.words[left_index])
                    * right.words[right_index]
                + result.words[output] + carry;
            result.words[output] = static_cast<std::uint32_t>(product);
            carry = product >> 32U;
        }
        std::size_t output = left_index + right.words.size();
        while (carry != 0U) {
            const std::uint64_t sum = result.words[output] + carry;
            result.words[output] = static_cast<std::uint32_t>(sum);
            carry = sum >> 32U;
            ++output;
        }
    }
    normalize_big(result);
    return result;
}

void shift_left(BigInt& value, std::uint64_t bits) {
    if (value.zero() || bits == 0U) return;
    const std::size_t word_shift = static_cast<std::size_t>(bits / 32U);
    const unsigned bit_shift = static_cast<unsigned>(bits % 32U);
    std::vector<std::uint32_t> shifted(
        value.words.size() + word_shift + (bit_shift == 0U ? 0U : 1U), 0U);
    std::uint64_t carry = 0U;
    for (std::size_t index = 0U; index < value.words.size(); ++index) {
        const std::uint64_t word =
            (static_cast<std::uint64_t>(value.words[index]) << bit_shift)
            | carry;
        shifted[index + word_shift] = static_cast<std::uint32_t>(word);
        carry = word >> 32U;
    }
    if (bit_shift != 0U) shifted[value.words.size() + word_shift]
        = static_cast<std::uint32_t>(carry);
    value.words = std::move(shifted);
    normalize_big(value);
}

std::uint64_t trailing_zero_bits(const BigInt& value) {
    std::uint64_t result = 0U;
    for (std::uint32_t word : value.words) {
        if (word == 0U) {
            result += 32U;
            continue;
        }
        result += std::countr_zero(word);
        break;
    }
    return result;
}

void shift_right_exact(BigInt& value, std::uint64_t bits) {
    if (value.zero() || bits == 0U) return;
    const std::size_t word_shift = static_cast<std::size_t>(bits / 32U);
    const unsigned bit_shift = static_cast<unsigned>(bits % 32U);
    value.words.erase(value.words.begin(), value.words.begin() + word_shift);
    if (bit_shift != 0U) {
        std::uint32_t carry = 0U;
        for (std::size_t remaining = value.words.size(); remaining > 0U;) {
            const std::size_t index = --remaining;
            const std::uint32_t next_carry = static_cast<std::uint32_t>(
                value.words[index] << (32U - bit_shift));
            value.words[index] = (value.words[index] >> bit_shift) | carry;
            carry = next_carry;
        }
    }
    normalize_big(value);
}

BigInt big_from_u32(std::uint32_t value) {
    BigInt result;
    if (value != 0U) result.words.push_back(value);
    return result;
}

std::uint64_t magnitude_bit_count(const BigInt& value) {
    if (value.zero()) return 0U;
    return static_cast<std::uint64_t>((value.words.size() - 1U) * 32U)
        + 32U - std::countl_zero(value.words.back());
}

std::vector<std::uint8_t> magnitude_bytes(const BigInt& value) {
    std::vector<std::uint8_t> bytes;
    bytes.reserve(value.words.size() * 4U);
    bool leading = true;
    for (std::size_t remaining = value.words.size(); remaining > 0U;) {
        const std::uint32_t word = value.words[--remaining];
        for (std::size_t byte = 0U; byte < 4U; ++byte) {
            const std::uint8_t next = static_cast<std::uint8_t>(
                word >> (24U - 8U * byte));
            if (leading && next == 0U) continue;
            leading = false;
            bytes.push_back(next);
        }
    }
    return bytes;
}

struct Dyadic final {
    BigInt numerator;
    std::int64_t exponent = 0;
};

void normalize(Dyadic& value) {
    if (value.numerator.zero()) {
        value.exponent = 0;
        return;
    }
    const std::uint64_t trailing = trailing_zero_bits(value.numerator);
    if (trailing != 0U) {
        shift_right_exact(value.numerator, trailing);
        value.exponent += static_cast<std::int64_t>(trailing);
    }
}

Dyadic dyadic_from_quad(Quad value, ExactWork* work) {
    if (work != nullptr) ++work->decodes;
    const auto bytes = canonical(value);
    const bool negative = (bytes[0U] & 0x80U) != 0U;
    const std::uint16_t raw_exponent = static_cast<std::uint16_t>(
        (static_cast<std::uint16_t>(bytes[0U] & 0x7fU) << 8U)
        | bytes[1U]);
    if (raw_exponent == 0x7fffU) return {{}, INT64_MIN};
    BigInt significand;
    for (std::size_t index = 2U; index < bytes.size(); ++index) {
        shift_left(significand, 8U);
        significand = add_big(significand, big_from_u32(bytes[index]));
    }
    std::int64_t exponent = 1 - 16383 - 112;
    if (raw_exponent != 0U) {
        BigInt hidden = big_from_u32(1U);
        shift_left(hidden, 112U);
        significand = add_big(significand, hidden);
        exponent = static_cast<std::int64_t>(raw_exponent) - 16383 - 112;
    }
    if (negative && !significand.zero()) significand.negative = true;
    Dyadic result{significand, exponent};
    normalize(result);
    return result;
}

Dyadic add_dyadic(Dyadic left, Dyadic right, ExactWork* work) {
    if (work != nullptr) ++work->additions;
    if (left.numerator.zero()) return right;
    if (right.numerator.zero()) return left;
    const std::int64_t exponent = std::min(left.exponent, right.exponent);
    if (left.exponent != exponent) {
        const std::uint64_t shift = static_cast<std::uint64_t>(
            left.exponent - exponent);
        shift_left(left.numerator, shift);
        if (work != nullptr) {
            ++work->alignment_shifts;
            work->alignment_bits += shift;
        }
    }
    if (right.exponent != exponent) {
        const std::uint64_t shift = static_cast<std::uint64_t>(
            right.exponent - exponent);
        shift_left(right.numerator, shift);
        if (work != nullptr) {
            ++work->alignment_shifts;
            work->alignment_bits += shift;
        }
    }
    Dyadic result{add_big(left.numerator, right.numerator), exponent};
    normalize(result);
    return result;
}

Dyadic multiply_dyadic(const Dyadic& left, const Dyadic& right,
    ExactWork* work) {
    if (work != nullptr) ++work->multiplies;
    Dyadic result{multiply_big(left.numerator, right.numerator),
        left.exponent + right.exponent};
    normalize(result);
    return result;
}

Dyadic negate_dyadic(Dyadic value) {
    if (!value.numerator.zero()) value.numerator.negative
        = !value.numerator.negative;
    return value;
}

Dyadic subtract_dyadic(const Dyadic& left, const Dyadic& right) {
    return add_dyadic(left, negate_dyadic(right), nullptr);
}

int compare_dyadic(const Dyadic& left, const Dyadic& right) {
    const Dyadic difference = subtract_dyadic(left, right);
    if (difference.numerator.zero()) return 0;
    return difference.numerator.negative ? -1 : 1;
}

std::uint64_t magnitude_bits(const Dyadic& value) {
    return magnitude_bit_count(value.numerator);
}

void hash_u64(r63zm::Sha256& hash, std::uint64_t value) {
    std::array<std::uint8_t, 8U> bytes{};
    for (std::size_t index = 0U; index < bytes.size(); ++index)
        bytes[index] = static_cast<std::uint8_t>(
            value >> (56U - 8U * index));
    hash.update(bytes);
}

void hash_dyadic(r63zm::Sha256& hash, const Dyadic& value) {
    const bool negative = value.numerator.negative;
    const std::vector<std::uint8_t> bytes = magnitude_bytes(value.numerator);
    hash.update_byte(negative ? 1U : 0U);
    hash_u64(hash, static_cast<std::uint64_t>(value.exponent));
    hash_u64(hash, bytes.size());
    hash.update(bytes);
}

r63zm::Digest exact_vector_root(const std::vector<Dyadic>& values) {
    r63zm::Sha256 hash;
    hash_u64(hash, sizeof(EXACT_VECTOR_DOMAIN) - 1U);
    hash.update(std::span<const std::uint8_t>(
        reinterpret_cast<const std::uint8_t*>(EXACT_VECTOR_DOMAIN),
        sizeof(EXACT_VECTOR_DOMAIN) - 1U));
    hash_u64(hash, values.size());
    for (const Dyadic& value : values) hash_dyadic(hash, value);
    return hash.finish();
}

std::string digest_hex(const r63zm::Digest& digest) {
    constexpr char digits[] = "0123456789abcdef";
    std::string text(64U, '0');
    for (std::size_t index = 0U; index < digest.size(); ++index) {
        text[2U * index] = digits[digest[index] >> 4U];
        text[2U * index + 1U] = digits[digest[index] & 0x0fU];
    }
    return text;
}

bool contained(const Dyadic& exact, Quad primary, Quad bound,
    Dyadic& slack, ExactWork& work) {
    const Quad lower = directed_subtract(primary, bound, FE_DOWNWARD);
    const Quad upper = directed_add(primary, bound, FE_UPWARD);
    const Dyadic exact_lower = dyadic_from_quad(lower, &work);
    const Dyadic exact_upper = dyadic_from_quad(upper, &work);
    work.interval_comparisons += 2U;
    const bool inside = compare_dyadic(exact, exact_lower) >= 0
        && compare_dyadic(exact, exact_upper) <= 0;
    if (!inside) return false;
    const Dyadic lower_slack = subtract_dyadic(exact, exact_lower);
    const Dyadic upper_slack = subtract_dyadic(exact_upper, exact);
    slack = compare_dyadic(lower_slack, upper_slack) <= 0
        ? lower_slack : upper_slack;
    return true;
}

} // namespace

int main(int argc, char** argv) {
    const bool bound_zero_control = argc == 5
        && std::strcmp(argv[4], "bound-zero") == 0;
    if ((argc != 4 && !bound_zero_control)
        || std::fesetround(FE_TONEAREST) != 0
        || std::fegetround() != FE_TONEAREST)
        return 64;

    Work candidate_work;
    ExactWork exact_work;
    std::array<std::uint8_t, CACHE_BYTES> cache{};
    std::array<std::uint8_t, ARTIFACT_BYTES> artifact{};
    std::array<std::uint8_t, AUDIT_BYTES> audit{};
    Digest cache_root{};
    Digest artifact_root{};
    Digest audit_root{};
    if (!read_exact(argv[1], cache, cache_root, candidate_work)
        || !read_exact(argv[2], artifact, artifact_root, candidate_work)
        || !read_exact(argv[3], audit, audit_root, candidate_work)
        || !digest_is(cache_root, CACHE_SHA256)
        || !digest_is(artifact_root, ARTIFACT_SHA256)
        || !digest_is(audit_root, AUDIT_SHA256))
        return 65;

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
    if (!inputs_valid) return 66;

    const Solve start = solve(factor, permutation, rhs, inverse,
        candidate_work);
    std::size_t x0_matches = 0U;
    for (std::size_t row = 0U; row < ROWS; ++row)
        x0_matches += std::memcmp(&start.solution[row], &x0[row], sizeof(Quad))
            == 0 ? 1U : 0U;
    if (!start.exact || x0_matches != ROWS) return 67;

    const std::size_t role2 = PRODUCT_BASE + 2U * PRODUCT_BYTES;
    const Digest x0_root = vector_root(x0);
    if (artifact[role2] != 1U || artifact[role2 + 1U] != 1U
        || artifact[role2 + 2U] != 2U
        || !digest_bytes_equal(x0_root,
            artifact.data() + 1004U + 3U * 32U))
        return 68;
    std::array<Quad, ROWS> y0{};
    for (std::size_t row = 0U; row < ROWS; ++row)
        y0[row] = big_quad(artifact.data() + role2 + PRODUCT_VALUES
            + row * sizeof(Quad));
    const Digest y0_root = vector_root(y0);
    if (!digest_bytes_equal(y0_root, artifact.data() + role2 + 208U))
        return 68;

    std::array<Quad, ROWS> residual{};
    for (std::size_t row = 0U; row < ROWS; ++row) {
        const Dot dot = dot2(2U,
            [&](std::size_t index) {
                return index == 0U ? rhs[row] : -y0[row];
            },
            [&](std::size_t) { return ONE; });
        if (!dot.exact || !dot.normal) return 69;
        residual[row] = dot.value;
    }
    const Solve preconditioned = solve(factor, permutation, residual, inverse,
        candidate_work);
    if (!preconditioned.exact) return 70;
    const std::array<Quad, ROWS>& p0 = preconditioned.solution;
    const Dot rho = dot2(ROWS,
        [&](std::size_t row) { return residual[row]; },
        [&](std::size_t row) { return p0[row]; });
    if (!rho.exact || !rho.normal || rho.value - rho.bound <= ZERO) return 71;

    Product product = tangent_product(tangent, sigma, p0,
        candidate_work, true);
    if (!product.exact) return 72;
    if (bound_zero_control) product.bound.fill(ZERO);

    std::vector<Dyadic> tangent_exact;
    tangent_exact.reserve(tangent.size());
    for (Quad value : tangent)
        tangent_exact.push_back(dyadic_from_quad(value, &exact_work));
    std::vector<Dyadic> p0_exact;
    p0_exact.reserve(ROWS);
    for (Quad value : p0)
        p0_exact.push_back(dyadic_from_quad(value, &exact_work));
    const Dyadic sigma_exact = dyadic_from_quad(sigma, &exact_work);

    std::vector<Dyadic> intermediate(COLUMNS);
    for (std::size_t column = 0U; column < COLUMNS; ++column) {
        Dyadic sum;
        for (std::size_t row = 0U; row < ROWS; ++row)
            sum = add_dyadic(sum, multiply_dyadic(
                tangent_exact[row * COLUMNS + column], p0_exact[row],
                &exact_work), &exact_work);
        intermediate[column] = std::move(sum);
    }

    std::vector<Dyadic> exact_product(ROWS);
    bool all_contained = true;
    std::size_t first_escape = ROWS;
    std::size_t contained_components = 0U;
    Dyadic minimum_slack;
    bool have_slack = false;
    for (std::size_t row = 0U; row < ROWS; ++row) {
        Dyadic sum;
        for (std::size_t column = 0U; column < COLUMNS; ++column)
            sum = add_dyadic(sum, multiply_dyadic(
                tangent_exact[row * COLUMNS + column], intermediate[column],
                &exact_work), &exact_work);
        exact_product[row] = multiply_dyadic(sigma_exact, sum, &exact_work);
        Dyadic slack;
        const bool component_contained = contained(exact_product[row],
            product.value[row], product.bound[row], slack, exact_work);
        if (!component_contained && first_escape == ROWS) first_escape = row;
        if (component_contained) ++contained_components;
        all_contained = all_contained && component_contained;
        if (component_contained && (!have_slack
                || compare_dyadic(slack, minimum_slack) < 0)) {
            minimum_slack = std::move(slack);
            have_slack = true;
        }
    }

    Dyadic exact_rho;
    Dyadic exact_denominator;
    for (std::size_t row = 0U; row < ROWS; ++row) {
        exact_rho = add_dyadic(exact_rho, multiply_dyadic(
            dyadic_from_quad(residual[row], &exact_work), p0_exact[row],
            &exact_work), &exact_work);
        exact_denominator = add_dyadic(exact_denominator, multiply_dyadic(
            p0_exact[row], exact_product[row], &exact_work), &exact_work);
    }

    const Dot denominator = dot2(ROWS,
        [&](std::size_t row) { return p0[row]; },
        [&](std::size_t row) { return product.value[row]; });
    Quad denominator_bound = denominator.bound;
    for (std::size_t row = 0U; row < ROWS; ++row)
        denominator_bound = up_add(denominator_bound,
            up_multiply(fabsq(p0[row]), product.bound[row]));
    Dyadic rho_slack;
    Dyadic denominator_slack;
    const bool rho_contained = contained(exact_rho, rho.value, rho.bound,
        rho_slack, exact_work);
    const bool denominator_contained = contained(exact_denominator,
        denominator.value, denominator_bound, denominator_slack, exact_work);
    const Quad rho_lower = directed_subtract(rho.value, rho.bound,
        FE_DOWNWARD);
    const Quad rho_upper = directed_add(rho.value, rho.bound, FE_UPWARD);
    const Quad denominator_lower = directed_subtract(denominator.value,
        denominator_bound, FE_DOWNWARD);
    const Quad denominator_upper = directed_add(denominator.value,
        denominator_bound, FE_UPWARD);
    const bool curvature_positive = !exact_denominator.numerator.zero()
        && !exact_denominator.numerator.negative
        && denominator_lower > ZERO;
    const Quad alpha = rho.value / denominator.value;
    const Interval step{
        directed_divide(rho_lower, denominator_upper, FE_DOWNWARD),
        directed_divide(rho_upper, denominator_lower, FE_UPWARD), true};
    const bool step_contains = curvature_positive
        && step.lower <= alpha && step.upper >= alpha;
    std::size_t update_matches = 0U;
    for (std::size_t row = 0U; row < ROWS; ++row) {
        const Dot update = dot2(2U,
            [&](std::size_t index) {
                return index == 0U ? x0[row] : alpha;
            },
            [&](std::size_t index) {
                return index == 0U ? ONE : p0[row];
            });
        update_matches += update.exact && update.normal
            && std::memcmp(&update.value, &x1[row], sizeof(Quad)) == 0
            ? 1U : 0U;
    }

    const bool apparatus = all_contained && rho_contained
        && denominator_contained;
    const char* route = !all_contained ? "DIRECT_PRODUCT_BOUND_REJECTED"
        : !rho_contained || !denominator_contained
            ? "R63ZP_APPARATUS_REJECTED"
        : !curvature_positive ? "DIRECT_CURVATURE_REJECTED"
        : !step_contains ? "DIRECT_STEP_REJECTED"
        : update_matches != ROWS ? "FIXED_UPDATE_CONSEQUENCE_REJECTED"
        : "DIRECT_DIRECTION_PRODUCT_ADMISSION_CANDIDATE";

    std::printf("route=%s\n", route);
    std::printf("x0_matches=%zu\n", x0_matches);
    std::printf("product_components_contained=%zu\n",
        contained_components);
    std::printf("first_escape=%zu\n", first_escape);
    std::printf("exact_product_root=%s\n",
        digest_hex(exact_vector_root(exact_product)).c_str());
    std::printf("minimum_slack_valid=%d\n", have_slack ? 1 : 0);
    std::printf("minimum_slack_bits=%llu\n",
        static_cast<unsigned long long>(magnitude_bits(minimum_slack)));
    std::printf("minimum_slack_exponent=%lld\n",
        static_cast<long long>(minimum_slack.exponent));
    std::printf("minimum_slack_log2_floor=%lld\n",
        static_cast<long long>(have_slack
            ? minimum_slack.exponent
                + static_cast<std::int64_t>(magnitude_bits(minimum_slack)) - 1
            : 0));
    std::printf("rho_exact_contained=%d\n", rho_contained ? 1 : 0);
    std::printf("denominator_exact_contained=%d\n",
        denominator_contained ? 1 : 0);
    std::printf("curvature_positive=%d\n", curvature_positive ? 1 : 0);
    std::printf("step_contains_candidate=%d\n", step_contains ? 1 : 0);
    std::printf("update_matches=%zu\n", update_matches);
    print_quad("rho_primary", rho.value);
    print_quad("rho_bound", rho.bound);
    print_quad("denominator_primary", denominator.value);
    print_quad("denominator_bound", denominator_bound);
    print_quad("alpha", alpha);
    print_interval("step", step);
    std::printf("candidate_file_reads=%llu\n",
        static_cast<unsigned long long>(candidate_work.file_reads));
    std::printf("candidate_file_bytes=%llu\n",
        static_cast<unsigned long long>(candidate_work.file_bytes));
    std::printf("candidate_file_hashes=%llu\n",
        static_cast<unsigned long long>(candidate_work.file_hashes));
    std::printf("candidate_factor_solves=%llu\n",
        static_cast<unsigned long long>(candidate_work.factor_solves));
    std::printf("candidate_factor_terms=%llu\n",
        static_cast<unsigned long long>(candidate_work.factor_terms));
    std::printf("candidate_product_terms=%llu\n",
        static_cast<unsigned long long>(candidate_work.oracle_terms));
    std::printf("exact_decodes=%llu\n",
        static_cast<unsigned long long>(exact_work.decodes));
    std::printf("exact_multiplies=%llu\n",
        static_cast<unsigned long long>(exact_work.multiplies));
    std::printf("exact_additions=%llu\n",
        static_cast<unsigned long long>(exact_work.additions));
    std::printf("exact_alignment_shifts=%llu\n",
        static_cast<unsigned long long>(exact_work.alignment_shifts));
    std::printf("exact_alignment_bits=%llu\n",
        static_cast<unsigned long long>(exact_work.alignment_bits));
    std::printf("exact_interval_comparisons=%llu\n",
        static_cast<unsigned long long>(exact_work.interval_comparisons));
    return apparatus ? 0 : 75;
}
