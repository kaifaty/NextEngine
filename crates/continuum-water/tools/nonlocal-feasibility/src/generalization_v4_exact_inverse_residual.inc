namespace {

__extension__ typedef unsigned __int128 ALR20R37WideInteger;

struct ALR20R37Integer {
    bool negative = false;
    std::vector<std::uint32_t> limbs;

    ALR20R37Integer() = default;
    ALR20R37Integer(std::int64_t value) {
        negative = value < 0;
        std::uint64_t magnitude = negative
            ? static_cast<std::uint64_t>(-(value + 1)) + 1U
            : static_cast<std::uint64_t>(value);
        while (magnitude != 0U) {
            limbs.push_back(static_cast<std::uint32_t>(magnitude));
            magnitude >>= 32U;
        }
    }

    bool zero() const { return limbs.empty(); }

    void normalize() {
        while (!limbs.empty() && limbs.back() == 0U) limbs.pop_back();
        if (limbs.empty()) negative = false;
    }

    std::uint32_t low_limb() const {
        return limbs.empty() ? 0U : limbs.front();
    }

    unsigned int bit_count() const {
        return limbs.empty() ? 0U
            : static_cast<unsigned int>(32U * (limbs.size() - 1U)
                + (32U - static_cast<unsigned int>(
                    __builtin_clz(limbs.back()))));
    }

    unsigned int trailing_zero_bits() const {
        unsigned int count = 0U;
        for (std::uint32_t limb : limbs) {
            if (limb != 0U)
                return count + static_cast<unsigned int>(__builtin_ctz(limb));
            count += 32U;
        }
        return 0U;
    }

    std::uint32_t divide_small(std::uint32_t divisor) {
        std::uint64_t remainder = 0U;
        for (std::size_t index = limbs.size(); index-- > 0U;) {
            const std::uint64_t current =
                (remainder << 32U) | limbs[index];
            limbs[index] = static_cast<std::uint32_t>(current / divisor);
            remainder = current % divisor;
        }
        normalize();
        return static_cast<std::uint32_t>(remainder);
    }

    std::string decimal() const {
        if (zero()) return "0";
        ALR20R37Integer remaining = *this;
        remaining.negative = false;
        std::vector<std::uint32_t> blocks;
        while (!remaining.zero())
            blocks.push_back(remaining.divide_small(1000000000U));
        std::ostringstream text;
        if (negative) text << '-';
        text << blocks.back();
        for (std::size_t index = blocks.size() - 1U; index-- > 0U;)
            text << std::setw(9) << std::setfill('0') << blocks[index];
        return text.str();
    }
};

int al_r20_r37_integer_magnitude_compare(
    const ALR20R37Integer& left, const ALR20R37Integer& right) {
    if (left.limbs.size() != right.limbs.size())
        return left.limbs.size() < right.limbs.size() ? -1 : 1;
    for (std::size_t index = left.limbs.size(); index-- > 0U;)
        if (left.limbs[index] != right.limbs[index])
            return left.limbs[index] < right.limbs[index] ? -1 : 1;
    return 0;
}

bool operator==(const ALR20R37Integer& left,
    const ALR20R37Integer& right) {
    return left.negative == right.negative && left.limbs == right.limbs;
}

bool operator!=(const ALR20R37Integer& left,
    const ALR20R37Integer& right) {
    return !(left == right);
}

bool operator==(const ALR20R37Integer& left, std::int64_t right) {
    return left == ALR20R37Integer(right);
}

bool operator!=(const ALR20R37Integer& left, std::int64_t right) {
    return !(left == right);
}

bool operator<(const ALR20R37Integer& left, std::int64_t right) {
    const ALR20R37Integer converted(right);
    if (left.negative != converted.negative) return left.negative;
    const int magnitude = al_r20_r37_integer_magnitude_compare(left, converted);
    return left.negative ? magnitude > 0 : magnitude < 0;
}

ALR20R37Integer operator-(ALR20R37Integer value) {
    if (!value.zero()) value.negative = !value.negative;
    return value;
}

ALR20R37Integer al_r20_r37_integer_magnitude_add(
    const ALR20R37Integer& left, const ALR20R37Integer& right) {
    ALR20R37Integer result;
    const std::size_t count = std::max(left.limbs.size(), right.limbs.size());
    result.limbs.resize(count);
    std::uint64_t carry = 0U;
    for (std::size_t index = 0U; index < count; ++index) {
        const std::uint64_t sum = carry
            + (index < left.limbs.size() ? left.limbs[index] : 0U)
            + (index < right.limbs.size() ? right.limbs[index] : 0U);
        result.limbs[index] = static_cast<std::uint32_t>(sum);
        carry = sum >> 32U;
    }
    if (carry != 0U) result.limbs.push_back(static_cast<std::uint32_t>(carry));
    return result;
}

ALR20R37Integer al_r20_r37_integer_magnitude_subtract(
    const ALR20R37Integer& larger, const ALR20R37Integer& smaller) {
    ALR20R37Integer result;
    result.limbs.resize(larger.limbs.size());
    std::uint64_t borrow = 0U;
    for (std::size_t index = 0U; index < larger.limbs.size(); ++index) {
        const std::uint64_t left = larger.limbs[index];
        const std::uint64_t right = borrow
            + (index < smaller.limbs.size() ? smaller.limbs[index] : 0U);
        result.limbs[index] = static_cast<std::uint32_t>(left - right);
        borrow = left < right ? 1U : 0U;
    }
    result.normalize();
    return result;
}

ALR20R37Integer operator+(
    const ALR20R37Integer& left, const ALR20R37Integer& right) {
    if (left.negative == right.negative) {
        ALR20R37Integer result =
            al_r20_r37_integer_magnitude_add(left, right);
        result.negative = left.negative && !result.zero();
        return result;
    }
    const int magnitude = al_r20_r37_integer_magnitude_compare(left, right);
    if (magnitude == 0) return {};
    const ALR20R37Integer& larger = magnitude > 0 ? left : right;
    const ALR20R37Integer& smaller = magnitude > 0 ? right : left;
    ALR20R37Integer result =
        al_r20_r37_integer_magnitude_subtract(larger, smaller);
    result.negative = larger.negative && !result.zero();
    return result;
}

ALR20R37Integer& operator+=(ALR20R37Integer& left,
    const ALR20R37Integer& right) {
    left = left + right;
    return left;
}

ALR20R37Integer operator*(
    const ALR20R37Integer& left, const ALR20R37Integer& right) {
    if (left.zero() || right.zero()) return {};
    ALR20R37Integer result;
    result.negative = left.negative != right.negative;
    result.limbs.assign(left.limbs.size() + right.limbs.size(), 0U);
    for (std::size_t lhs = 0U; lhs < left.limbs.size(); ++lhs) {
        ALR20R37WideInteger carry = 0U;
        for (std::size_t rhs = 0U; rhs < right.limbs.size(); ++rhs) {
            const std::size_t index = lhs + rhs;
            const ALR20R37WideInteger current =
                static_cast<ALR20R37WideInteger>(left.limbs[lhs])
                    * right.limbs[rhs]
                + result.limbs[index] + carry;
            result.limbs[index] = static_cast<std::uint32_t>(current);
            carry = current >> 32U;
        }
        std::size_t index = lhs + right.limbs.size();
        while (carry != 0U) {
            const ALR20R37WideInteger current =
                static_cast<ALR20R37WideInteger>(result.limbs[index]) + carry;
            result.limbs[index] = static_cast<std::uint32_t>(current);
            carry = current >> 32U;
            ++index;
            if (index == result.limbs.size() && carry != 0U)
                result.limbs.push_back(0U);
        }
    }
    result.normalize();
    return result;
}

ALR20R37Integer operator<<(ALR20R37Integer value, unsigned int shift) {
    if (value.zero() || shift == 0U) return value;
    const unsigned int whole = shift / 32U;
    const unsigned int bits = shift % 32U;
    std::vector<std::uint32_t> shifted(whole, 0U);
    std::uint64_t carry = 0U;
    for (std::uint32_t limb : value.limbs) {
        const std::uint64_t current =
            (static_cast<std::uint64_t>(limb) << bits) | carry;
        shifted.push_back(static_cast<std::uint32_t>(current));
        carry = current >> 32U;
    }
    if (carry != 0U) shifted.push_back(static_cast<std::uint32_t>(carry));
    value.limbs = std::move(shifted);
    return value;
}

ALR20R37Integer operator>>(ALR20R37Integer value, unsigned int shift) {
    if (value.zero() || shift == 0U) return value;
    const unsigned int whole = shift / 32U;
    const unsigned int bits = shift % 32U;
    if (whole >= value.limbs.size()) return {};
    value.limbs.erase(value.limbs.begin(),
        value.limbs.begin() + static_cast<std::ptrdiff_t>(whole));
    if (bits != 0U) {
        std::uint32_t carry = 0U;
        for (std::size_t index = value.limbs.size(); index-- > 0U;) {
            const std::uint32_t next = static_cast<std::uint32_t>(
                value.limbs[index] << (32U - bits));
            value.limbs[index] = (value.limbs[index] >> bits) | carry;
            carry = next;
        }
    }
    value.normalize();
    return value;
}

ALR20R37Integer& operator>>=(ALR20R37Integer& value, unsigned int shift) {
    value = value >> shift;
    return value;
}

ALR20R37Integer& operator++(ALR20R37Integer& value) {
    value += ALR20R37Integer(1);
    return value;
}

constexpr const char* AL_R20_R36_SEMANTIC_SHA256 =
    "448a9b8b64a015d0dcbceac82555216ad6421e22a5b4a8b62a95fcfe42eab90f";
constexpr const char* AL_R20_R36_TORSION_INVERSE_SHA256 =
    "5fe4d71e68163c4554ec76482fc5e0df838786373b80807b132952d240c85e7c";
constexpr const char* AL_R20_R36_TORSION_NORM_SHA256 =
    "c2d53190ec86534a346a556ddf82fdb3642c55bdde1a6363afa3dc8465296dfd";

struct ALR20R37Dyadic {
    ALR20R37Integer numerator = 0;
    int exponent = 0;
};

struct ALR20R37Conversion {
    bool exact = false;
    ALR20R37Dyadic value;
};

ALR20R37Dyadic al_r20_r37_normalize(ALR20R37Dyadic value) {
    if (value.numerator == 0) {
        value.exponent = 0;
        return value;
    }
    const bool negative = value.numerator < 0;
    ALR20R37Integer magnitude = negative
        ? -value.numerator : value.numerator;
    const unsigned int zero_bits = magnitude.trailing_zero_bits();
    magnitude >>= zero_bits;
    value.numerator = negative ? -magnitude : magnitude;
    value.exponent += static_cast<int>(zero_bits);
    return value;
}

ALR20R37Conversion al_r20_r37_from_binary128(Binary128 input) {
    ALR20R37Conversion result;
    if (finiteq(input) == 0) return result;
    if (input == static_cast<Binary128>(0.0)) {
        result.exact = true;
        return result;
    }
    const bool negative = signbitq(input) != 0;
    int exponent = 0;
    const Binary128 fraction = frexpq(fabsq(input), &exponent);
    Binary128 integer = ldexpq(fraction, FLT128_MANT_DIG);
    const Binary128 radix = ldexpq(static_cast<Binary128>(1.0), 32);
    ALR20R37Integer significand = 0;
    for (unsigned int limb_index = 0U; limb_index < 4U; ++limb_index) {
        const Binary128 quotient = floorq(integer / radix);
        const Binary128 remainder = integer - quotient * radix;
        if (remainder < static_cast<Binary128>(0.0)
            || remainder >= radix || floorq(remainder) != remainder)
            return result;
        const std::uint32_t limb = static_cast<std::uint32_t>(remainder);
        significand += ALR20R37Integer(limb) << (32U * limb_index);
        integer = quotient;
    }
    if (integer != static_cast<Binary128>(0.0)) return result;
    result.value.numerator = negative ? -significand : significand;
    result.value.exponent = exponent - FLT128_MANT_DIG;
    result.value = al_r20_r37_normalize(std::move(result.value));
    result.exact = result.value.numerator != 0;
    return result;
}

Binary128 al_r20_r37_integer_binary128(const ALR20R37Integer& input) {
    ALR20R37Integer remaining = input;
    if (remaining < 0) remaining = -remaining;
    std::vector<std::uint32_t> limbs;
    while (remaining != 0) {
        limbs.push_back(remaining.low_limb());
        remaining >>= 32U;
    }
    Binary128 value = static_cast<Binary128>(0.0);
    for (std::size_t index = limbs.size(); index-- > 0U;)
        value = ldexpq(value, 32) + static_cast<Binary128>(limbs[index]);
    return value;
}

Binary128 al_r20_r37_binary128_up(const ALR20R37Dyadic& input) {
    if (input.numerator == 0) return static_cast<Binary128>(0.0);
    if (input.numerator < 0) return -al_r20_r37_binary128_up(
        ALR20R37Dyadic{-input.numerator, input.exponent});
    const unsigned int bits = input.numerator.bit_count();
    const unsigned int shift = bits > FLT128_MANT_DIG
        ? bits - FLT128_MANT_DIG : 0U;
    ALR20R37Integer top = input.numerator >> shift;
    const ALR20R37Integer restored = top << shift;
    if (restored != input.numerator) ++top;
    const Binary128 significand = al_r20_r37_integer_binary128(top);
    return ldexpq(significand,
        input.exponent + static_cast<int>(shift));
}

ALR20R37Dyadic al_r20_r37_add(
    const ALR20R37Dyadic& left, const ALR20R37Dyadic& right) {
    if (left.numerator == 0) return right;
    if (right.numerator == 0) return left;
    const int exponent = std::min(left.exponent, right.exponent);
    ALR20R37Dyadic result;
    result.numerator =
        (left.numerator << static_cast<unsigned int>(left.exponent - exponent))
        + (right.numerator
            << static_cast<unsigned int>(right.exponent - exponent));
    result.exponent = exponent;
    return al_r20_r37_normalize(std::move(result));
}

ALR20R37Dyadic al_r20_r37_multiply(
    const ALR20R37Dyadic& left, const ALR20R37Dyadic& right) {
    return al_r20_r37_normalize({
        left.numerator * right.numerator,
        left.exponent + right.exponent});
}

ALR20R37Dyadic al_r20_r37_absolute(ALR20R37Dyadic value) {
    if (value.numerator < 0) value.numerator = -value.numerator;
    return value;
}

int al_r20_r37_compare_positive(
    const ALR20R37Dyadic& left, const ALR20R37Dyadic& right) {
    const int exponent = std::min(left.exponent, right.exponent);
    const ALR20R37Integer left_integer = left.numerator
        << static_cast<unsigned int>(left.exponent - exponent);
    const ALR20R37Integer right_integer = right.numerator
        << static_cast<unsigned int>(right.exponent - exponent);
    return al_r20_r37_integer_magnitude_compare(
        left_integer, right_integer);
}

std::string al_r20_r37_dyadic_text(const ALR20R37Dyadic& value) {
    return value.numerator.decimal() + "p"
        + std::to_string(value.exponent);
}

struct ALR20R37ExactResidual {
    bool exact = false;
    bool outward_contains = false;
    bool contractive = false;
    std::size_t dimension = 0U;
    std::size_t worst_row = 0U;
    ALR20R37Dyadic norm;
    Binary128 outward_norm = static_cast<Binary128>(0.0);
    std::vector<ALR20R37Dyadic> residual;
    std::string residual_root;
    std::string root;
};

ALR20R37ExactResidual al_r20_r37_exact_residual(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& inverse,
    std::size_t dimension) {
    ALR20R37ExactResidual result;
    result.dimension = dimension;
    if (dimension == 0U || matrix.size() != dimension * dimension
        || inverse.size() != dimension * dimension)
        return result;
    std::vector<ALR20R37Dyadic> matrix_exact(matrix.size());
    std::vector<ALR20R37Dyadic> inverse_exact(inverse.size());
    bool operands_exact = true;
    for (std::size_t index = 0U; index < matrix.size(); ++index) {
        const ALR20R37Conversion converted =
            al_r20_r37_from_binary128(matrix[index]);
        operands_exact = operands_exact && converted.exact;
        matrix_exact[index] = converted.value;
    }
    for (std::size_t index = 0U; index < inverse.size(); ++index) {
        const ALR20R37Conversion converted =
            al_r20_r37_from_binary128(inverse[index]);
        operands_exact = operands_exact && converted.exact;
        inverse_exact[index] = converted.value;
    }
    if (!operands_exact) return result;
    std::ostringstream residual_material;
    for (std::size_t row = 0U; row < dimension; ++row) {
        ALR20R37Dyadic row_sum;
        for (std::size_t column = 0U; column < dimension; ++column) {
            ALR20R37Dyadic image;
            for (std::size_t inner = 0U; inner < dimension; ++inner)
                image = al_r20_r37_add(image, al_r20_r37_multiply(
                    matrix_exact[row * dimension + inner],
                    inverse_exact[inner * dimension + column]));
            const ALR20R37Dyadic identity{
                row == column ? ALR20R37Integer(1) : ALR20R37Integer(0), 0};
            const ALR20R37Dyadic residual = al_r20_r37_add(
                identity, {-image.numerator, image.exponent});
            result.residual.push_back(residual);
            row_sum = al_r20_r37_add(
                row_sum, al_r20_r37_absolute(residual));
            residual_material << al_r20_r37_dyadic_text(residual) << ';';
        }
        residual_material << '|';
        if (row == 0U
            || al_r20_r37_compare_positive(row_sum, result.norm) > 0) {
            result.norm = row_sum;
            result.worst_row = row;
        }
    }
    result.residual_root = sha256_hex(residual_material.str());
    result.outward_norm = al_r20_r37_binary128_up(result.norm);
    const ALR20R37Conversion outward =
        al_r20_r37_from_binary128(result.outward_norm);
    result.outward_contains = outward.exact
        && al_r20_r37_compare_positive(outward.value, result.norm) >= 0;
    const ALR20R37Dyadic one{ALR20R37Integer(1), 0};
    result.contractive = al_r20_r37_compare_positive(result.norm, one) < 0;
    result.exact = result.outward_contains
        && finiteq(result.outward_norm) != 0;
    std::ostringstream material;
    material << result.exact << ':' << result.outward_contains << ':'
        << result.contractive << ':' << dimension << ':' << result.worst_row
        << ':' << al_r20_r37_dyadic_text(result.norm) << ':'
        << al_binary128_hex(result.outward_norm) << ':'
        << result.residual_root;
    result.root = sha256_hex(material.str());
    return result;
}

bool al_r20_r37_dyadic_controls() {
    const Binary128 one = static_cast<Binary128>(1.0);
    const Binary128 zero = static_cast<Binary128>(0.0);
    const std::array<Binary128, 7U> values{{
        zero, one, -one, strtoflt128("0.1", nullptr),
        ldexpq(one, -60), ldexpq(one, 100), nextafterq(zero, one)}};
    bool roundtrip_exact = true;
    for (Binary128 value : values) {
        const ALR20R37Conversion exact = al_r20_r37_from_binary128(value);
        const Binary128 roundtrip = exact.exact
            ? al_r20_r37_binary128_up(exact.value) : HUGE_VALQ;
        roundtrip_exact = roundtrip_exact && exact.exact
            && al_binary128_hex(roundtrip) == al_binary128_hex(value);
    }
    const ALR20R37ExactResidual identity = al_r20_r37_exact_residual(
        {one, zero, zero, one}, {one, zero, zero, one}, 2U);
    const ALR20R37ExactResidual diagonal = al_r20_r37_exact_residual(
        {ldexpq(one, 1), zero, zero, ldexpq(one, -1)},
        {ldexpq(one, -1), zero, zero, ldexpq(one, 1)}, 2U);
    const Binary128 epsilon = ldexpq(one, -60);
    const Binary128 inverse_scale = ldexpq(one, 60);
    const std::vector<Binary128> cancellation_matrix{
        one, one, one, one + epsilon};
    const std::vector<Binary128> cancellation_inverse{
        inverse_scale + one, -inverse_scale,
        -inverse_scale, inverse_scale};
    const ALR20R37ExactResidual cancellation = al_r20_r37_exact_residual(
        cancellation_matrix, cancellation_inverse, 2U);
    const ALR20R37ExactResidual noncontractive = al_r20_r37_exact_residual(
        {one, zero, zero, one}, {zero, zero, zero, zero}, 2U);
    const std::vector<Binary128> permuted_matrix{
        one + epsilon, one, one, one};
    const std::vector<Binary128> permuted_inverse{
        inverse_scale, -inverse_scale,
        -inverse_scale, inverse_scale + one};
    const ALR20R37ExactResidual permuted_a = al_r20_r37_exact_residual(
        permuted_matrix, permuted_inverse, 2U);
    const ALR20R37ExactResidual permuted_b = al_r20_r37_exact_residual(
        permuted_matrix, permuted_inverse, 2U);
    return roundtrip_exact
        && identity.exact && identity.contractive
        && identity.norm.numerator == 0
        && diagonal.exact && diagonal.contractive
        && diagonal.norm.numerator == 0
        && cancellation.exact && cancellation.contractive
        && cancellation.norm.numerator == 0
        && noncontractive.exact && !noncontractive.contractive
        && al_r20_r37_dyadic_text(noncontractive.norm) == "1p0"
        && permuted_a.exact && permuted_a.contractive
        && permuted_a.norm.numerator == 0
        && permuted_a.root == permuted_b.root;
}

} // namespace

SplitBoundaryReport
run_al_generalization_v4_exact_inverse_residual_controls() {
#if defined(__linux__) && defined(__x86_64__) && defined(__GNUC__) \
    && defined(__SIZEOF_FLOAT128__)
    constexpr bool frozen_platform = true;
#else
    constexpr bool frozen_platform = false;
#endif
    const bool profile_exact = frozen_platform
        && sizeof(Binary128) == 16U && FLT_RADIX == 2
        && FLT128_MANT_DIG == 113;
    const bool parent_controls_exact = profile_exact
        && al_r20_verified_inverse_controls()
        && al_r20_semismooth_controls()
        && al_r20_r21_event_controls();
    const bool dyadic_controls_exact =
        profile_exact && al_r20_r37_dyadic_controls();
    const bool controls_exact =
        parent_controls_exact && dyadic_controls_exact;
    std::vector<ALR20ManifestCase> cases = al_r20_v4_manifest_cases();
    bool parent_exact = parent_controls_exact && cases.size() == 5U
        && al_r20_v4_manifest_root(cases) == AL_R20_V4_MANIFEST_SHA256;
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    ALR20ProblemData problem;
    bool problem_materialized = false;
    if (parent_exact) {
        problem = al_r20_materialize_problem(
            cases[0U], &sparse_work, &static_work, &adjacency_work);
        problem_materialized = true;
        parent_exact = problem.exact && problem.source_positive == 0U
            && problem.problem_root == AL_R20_V4_PROBLEM_SHA256[0U];
    }
    ALR20VerifiedInverseCapture capture;
    capture.selected_root = AL_R20_R36_TORSION_INVERSE_SHA256;
    ALR20R35CaseAudit torsion;
    std::size_t solver_executions = 0U;
    const bool capture_slot_clear =
        al_r20_verified_inverse_capture == nullptr;
    if (parent_exact && capture_slot_clear) {
        al_r20_verified_inverse_capture = &capture;
        torsion = al_r20_r35_case(problem);
        al_r20_verified_inverse_capture = nullptr;
        solver_executions = 1U;
    }
    const std::size_t target_capture_count = static_cast<std::size_t>(
        std::count(capture.audit_roots.begin(), capture.audit_roots.end(),
            AL_R20_R36_TORSION_INVERSE_SHA256));
    const bool reproduction_exact = parent_exact && capture_slot_clear
        && torsion.root == AL_R20_R35_TORSION_CASE_SHA256
        && !torsion.diagnostic.steps.empty()
        && torsion.diagnostic.steps.back().root
            == AL_R20_R35_TORSION_STEP_SHA256
        && capture.calls == 2U
        && capture.audit_roots.size() == capture.calls
        && target_capture_count == 1U && capture.factor_passed
        && capture.audit.root == AL_R20_R36_TORSION_INVERSE_SHA256
        && capture.audit.norm.root == AL_R20_R36_TORSION_NORM_SHA256
        && capture.audit.dimension == 65U
        && capture.audit.column_solves == 65U
        && capture.matrix.size() == 65U * 65U
        && capture.lower.size() == 65U * 65U
        && capture.inverse.size() == 65U * 65U
        && capture.column_bound.size() == 65U
        && solver_executions == 1U;
    const ALR20InverseNormAudit inherited_norm = reproduction_exact
        ? al_r20_inverse_norm_audit(capture.matrix, capture.inverse,
            capture.column_bound, capture.audit.dimension)
        : ALR20InverseNormAudit{};
    const bool inherited_norm_exact = reproduction_exact
        && inherited_norm.root == AL_R20_R36_TORSION_NORM_SHA256
        && al_binary128_hex(inherited_norm.rho)
            == al_binary128_hex(capture.audit.norm.rho);
    const ALR20R37ExactResidual exact_residual = inherited_norm_exact
        ? al_r20_r37_exact_residual(
            capture.matrix, capture.inverse, capture.audit.dimension)
        : ALR20R37ExactResidual{};
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool exact = controls_exact && parent_exact && reproduction_exact
        && inherited_norm_exact && exact_residual.exact && lifecycle_exact;
    const std::string route = !parent_exact
        ? "EXACT_INVERSE_PARENT_REJECTED"
        : (!dyadic_controls_exact
            ? "EXACT_DYADIC_APPARATUS_REJECTED"
            : (!reproduction_exact || !inherited_norm_exact
                ? "EXACT_INVERSE_CAPTURE_REJECTED"
                : (exact_residual.contractive
                    ? "ARITHMETIC_ENCLOSURE_DOMINATES"
                    : "INVERSE_CANDIDATE_NONCONTRACTIVE")));
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r37-exact-residual.v1|"
        << AL_R20_R36_SEMANTIC_SHA256 << ':' << route << ':'
        << torsion.root << ':' << capture.audit.root << ':'
        << capture.matrix_root << ':' << capture.factor_root << ':'
        << al_r20_q_scalar_root(capture.inverse) << ':'
        << inherited_norm.root << ':' << exact_residual.root;
    const std::string result_sha = sha256_hex(semantic.str());
    const Binary128 enclosure_ratio = exact_residual.outward_norm
            > static_cast<Binary128>(0.0)
        ? inherited_norm.rho / exact_residual.outward_norm
        : HUGE_VALQ;
    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r37_exact_residual.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r36_sha256\":\""
        << AL_R20_R36_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_controls_exact\":"
        << (parent_controls_exact ? "true" : "false")
        << ",\"dyadic_controls_exact\":"
        << (dyadic_controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"reproduction_exact\":"
        << (reproduction_exact ? "true" : "false")
        << ",\"inherited_norm_exact\":"
        << (inherited_norm_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"case_sha256\":\"" << torsion.root
        << "\",\"step_sha256\":\""
        << (torsion.diagnostic.steps.empty()
            ? "" : torsion.diagnostic.steps.back().root)
        << "\",\"inverse_audit_sha256\":\"" << capture.audit.root
        << "\",\"inherited_norm_sha256\":\"" << inherited_norm.root
        << "\",\"matrix_sha256\":\"" << capture.matrix_root
        << "\",\"factor_sha256\":\"" << capture.factor_root
        << "\",\"inverse_sha256\":\""
        << al_r20_q_scalar_root(capture.inverse)
        << "\",\"dimension\":" << capture.audit.dimension
        << ",\"capture_calls\":" << capture.calls
        << ",\"capture_roots\":[";
    for (std::size_t index = 0U; index < capture.audit_roots.size(); ++index) {
        if (index != 0U) report << ',';
        report << "\"" << capture.audit_roots[index] << "\"";
    }
    report << "]"
        << ",\"captured_column_solves\":" << capture.total_column_solves
        << ",\"existing_column_solves\":" << capture.audit.column_solves
        << ",\"inherited_rho\":\""
        << al_binary128_text(inherited_norm.rho)
        << "\",\"exact_residual\":{\"contractive\":"
        << (exact_residual.contractive ? "true" : "false")
        << ",\"worst_row\":" << exact_residual.worst_row
        << ",\"norm_dyadic\":\""
        << al_r20_r37_dyadic_text(exact_residual.norm)
        << "\",\"norm_outward\":\""
        << al_binary128_text(exact_residual.outward_norm)
        << "\",\"outward_contains\":"
        << (exact_residual.outward_contains ? "true" : "false")
        << ",\"enclosure_ratio\":\""
        << al_binary128_text(enclosure_ratio)
        << "\",\"residual_sha256\":\""
        << exact_residual.residual_root
        << "\",\"root\":\"" << exact_residual.root << "\"}"
        << ",\"solver_executions\":" << solver_executions
        << ",\"additional_factorizations\":0"
        << ",\"additional_inverse_columns\":0"
        << ",\"counterflow_replays\":0,\"new_trials\":0"
        << ",\"state_updates\":0,\"parameter_changes\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
