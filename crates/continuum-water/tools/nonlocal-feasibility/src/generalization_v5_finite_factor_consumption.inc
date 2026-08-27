namespace {

constexpr const char* AL_R20_R63ZA_PARENT_SEMANTIC_SHA256 =
    "e27ee8616c1a34a1a224f65f32d5cebe1cd6dac973ccc2ae89c3fbad8343e4be";
constexpr const char* AL_R20_R63ZA_PARENT_ARTIFACT_SHA256 =
    "38cd1871eaf35c2153b469f4e98bba15754baa3853ceb76fdecc40d2d0146de1";
constexpr const char* AL_R20_R63ZA_EXPORTED_FACTOR_SHA256 =
    "a7a85789364f5af91aaf71e1531759713c95ec085e48b7ca614daa524c3a4f85";
constexpr const char* AL_R20_R63ZA_PERMUTATION_SHA256 =
    "335235cbaf9a93c805a2bfdbd17c593eb3ae44c9a20ea8a752782fa10f15826f";
constexpr const char* AL_R20_R63ZA_EXPORTED_LANE_SHA256 =
    "5dcc65f74b8769d5a53cf5db2937f4c2013dc4b92f0333bb42a791006bc3c255";
constexpr std::size_t AL_R20_R63ZA_DIMENSION = 102U;

struct ALR20R63ZAScalar {
    bool exact = false;
    double high = 0.0;
    double low = 0.0;
    double radius = 0.0;
    std::string root;
};

std::string al_r20_r63za_scalar_root(const ALR20R63ZAScalar& value) {
    std::ostringstream material;
    material << value.exact << ':' << binary64_bits(value.high) << ':'
        << binary64_bits(value.low) << ':' << binary64_bits(value.radius);
    return sha256_hex(material.str());
}

bool al_r20_r63za_canonical(const ALR20R63ZAScalar& value) {
    if (!value.exact || !al_r20_r63y_normal_or_zero(value.high)
        || !al_r20_r63y_normal_or_zero(value.low)
        || !al_r20_r63y_normal_or_zero(value.radius)
        || value.radius < 0.0
        || value.root != al_r20_r63za_scalar_root(value)) return false;
    if (value.high == 0.0) return value.low == 0.0;
    const double magnitude = std::abs(value.high);
    const double ulp = std::nextafter(
        magnitude, std::numeric_limits<double>::infinity()) - magnitude;
    return std::isfinite(ulp) && ulp > 0.0
        && std::abs(value.low) <= 0.5 * ulp;
}

ALR20R63ZAScalar al_r20_r63za_seal(
    double high, double low, double radius) {
    ALR20R63ZAScalar result;
    const ALR20R63Pair normalized = al_r20_r63_two_sum(high, low);
    result.high = normalized.high;
    result.low = normalized.low;
    result.radius = radius;
    result.exact = normalized.finite
        && al_r20_r63y_normal_or_zero(result.high)
        && al_r20_r63y_normal_or_zero(result.low)
        && al_r20_r63y_normal_or_zero(radius) && radius >= 0.0;
    result.root = al_r20_r63za_scalar_root(result);
    result.exact = result.exact && al_r20_r63za_canonical(result);
    if (!result.exact) result.root = al_r20_r63za_scalar_root(result);
    return result;
}

ALR20R63ZAScalar al_r20_r63za_exact_double(double value) {
    return al_r20_r63za_seal(value, 0.0, 0.0);
}

ALR20R63ZAScalar al_r20_r63za_project(Binary128 value) {
    const ALR20R37Conversion conversion = al_r20_r37_from_binary128(value);
    const ALR20R37Dyadic one = al_r20_r63p_control_dyadic(1);
    const ALR20R63YExpansion expansion = conversion.exact
        ? al_r20_r63y_project(conversion.value, one, 2U)
        : ALR20R63YExpansion{};
    if (!expansion.exact || expansion.components.size() != 2U)
        return ALR20R63ZAScalar{};
    return al_r20_r63za_seal(expansion.components[0U],
        expansion.components[1U], expansion.radius);
}

double al_r20_r63za_abs_center(const ALR20R63ZAScalar& value) {
    return al_r20_r63y_up_add(
        std::abs(value.high), std::abs(value.low));
}

double al_r20_r63za_center_error(
    const ALR20R63Dot2Err& dot, double high, double low) {
    const ALR20R63Dot2Err difference = al_r20_r63_dot2err(
        3U,
        [&](std::size_t index) {
            return index == 0U ? dot.result : (index == 1U ? -high : -low);
        },
        [](std::size_t) { return 1.0; });
    if (!dot.exact || !difference.exact)
        return std::numeric_limits<double>::infinity();
    return al_r20_r63y_up_add(dot.error,
        al_r20_r63y_up_add(std::abs(difference.result), difference.error));
}

ALR20R63ZAScalar al_r20_r63za_add(
    const ALR20R63ZAScalar& left, const ALR20R63ZAScalar& right,
    double right_sign = 1.0) {
    if (!al_r20_r63za_canonical(left) || !al_r20_r63za_canonical(right)
        || (right_sign != 1.0 && right_sign != -1.0))
        return ALR20R63ZAScalar{};
    const ALR20R63Pair leading = al_r20_r63_two_sum(
        left.high, right_sign * right.high);
    const ALR20R63Pair trailing = al_r20_r63_two_sum(
        left.low, right_sign * right.low);
    const ALR20R63Pair middle = al_r20_r63_two_sum(
        leading.low, trailing.high);
    const ALR20R63Pair folded = al_r20_r63_two_sum(
        leading.high, middle.high);
    const double remaining = (middle.low + trailing.low) + folded.low;
    const ALR20R63Pair normalized = al_r20_r63_two_sum(
        folded.high, remaining);
    const ALR20R63Dot2Err exact_sum = al_r20_r63_dot2err(
        4U,
        [&](std::size_t index) {
            if (index == 0U) return left.high;
            if (index == 1U) return left.low;
            if (index == 2U) return right_sign * right.high;
            return right_sign * right.low;
        },
        [](std::size_t) { return 1.0; });
    double radius = al_r20_r63y_up_add(left.radius, right.radius);
    radius = al_r20_r63y_up_add(radius,
        al_r20_r63za_center_error(
            exact_sum, normalized.high, normalized.low));
    return al_r20_r63za_seal(normalized.high, normalized.low, radius);
}

ALR20R63ZAScalar al_r20_r63za_multiply(
    const ALR20R63ZAScalar& left, const ALR20R63ZAScalar& right) {
    if (!al_r20_r63za_canonical(left) || !al_r20_r63za_canonical(right))
        return ALR20R63ZAScalar{};
    const ALR20R63Pair p00 = al_r20_r63_two_product(
        left.high, right.high);
    const double cross = left.high * right.low + left.low * right.high;
    const double tail = left.low * right.low;
    const ALR20R63Pair folded = al_r20_r63_two_sum(
        p00.low, cross + tail);
    const ALR20R63Pair normalized = al_r20_r63_two_sum(
        p00.high, folded.high + folded.low);
    const ALR20R63Dot2Err exact_product = al_r20_r63_dot2err(
        4U,
        [&](std::size_t index) {
            return index < 2U ? left.high : left.low;
        },
        [&](std::size_t index) {
            return index % 2U == 0U ? right.high : right.low;
        });
    double radius = al_r20_r63za_center_error(
        exact_product, normalized.high, normalized.low);
    radius = al_r20_r63y_up_add(radius,
        al_r20_r63y_up_multiply(
            al_r20_r63za_abs_center(left), right.radius));
    radius = al_r20_r63y_up_add(radius,
        al_r20_r63y_up_multiply(
            al_r20_r63za_abs_center(right), left.radius));
    radius = al_r20_r63y_up_add(radius,
        al_r20_r63y_up_multiply(left.radius, right.radius));
    return al_r20_r63za_seal(normalized.high, normalized.low, radius);
}

ALR20R63ZAScalar al_r20_r63za_divide(
    const ALR20R63ZAScalar& numerator,
    const ALR20R63ZAScalar& denominator) {
    if (!al_r20_r63za_canonical(numerator)
        || !al_r20_r63za_canonical(denominator))
        return ALR20R63ZAScalar{};
    const double denominator_abs = al_r20_r63za_abs_center(denominator);
    if (!(denominator_abs > denominator.radius)
        || denominator.high == 0.0) return ALR20R63ZAScalar{};
    const ALR20R63ZAScalar zero_radius_n = al_r20_r63za_seal(
        numerator.high, numerator.low, 0.0);
    const ALR20R63ZAScalar zero_radius_d = al_r20_r63za_seal(
        denominator.high, denominator.low, 0.0);
    const double q0 = numerator.high / denominator.high;
    ALR20R63ZAScalar quotient = al_r20_r63za_exact_double(q0);
    ALR20R63ZAScalar residual = al_r20_r63za_add(zero_radius_n,
        al_r20_r63za_multiply(zero_radius_d, quotient), -1.0);
    const double q1 = residual.high / denominator.high;
    quotient = al_r20_r63za_add(
        quotient, al_r20_r63za_exact_double(q1));
    residual = al_r20_r63za_add(zero_radius_n,
        al_r20_r63za_multiply(zero_radius_d,
            al_r20_r63za_seal(quotient.high, quotient.low, 0.0)), -1.0);
    const double q2 = residual.high / denominator.high;
    quotient = al_r20_r63za_add(
        al_r20_r63za_seal(quotient.high, quotient.low, 0.0),
        al_r20_r63za_exact_double(q2));
    const ALR20R63ZAScalar center = al_r20_r63za_seal(
        quotient.high, quotient.low, 0.0);
    const ALR20R63ZAScalar final_residual = al_r20_r63za_add(
        zero_radius_n,
        al_r20_r63za_multiply(zero_radius_d, center), -1.0);
    double residual_bound = al_r20_r63y_up_add(
        al_r20_r63za_abs_center(final_residual), final_residual.radius);
    residual_bound = al_r20_r63y_up_add(residual_bound, numerator.radius);
    residual_bound = al_r20_r63y_up_add(residual_bound,
        al_r20_r63y_up_multiply(
            al_r20_r63za_abs_center(center), denominator.radius));
    const double denominator_lower = std::nextafter(
        denominator_abs - denominator.radius, 0.0);
    const double radius = denominator_lower > 0.0
        ? al_r20_r63y_up_divide(residual_bound, denominator_lower)
        : std::numeric_limits<double>::infinity();
    return al_r20_r63za_seal(center.high, center.low, radius);
}

struct ALR20R63ZASolve {
    bool exact = false;
    bool contained = false;
    std::string role;
    std::size_t dimension = 0U;
    std::size_t forward_terms = 0U;
    std::size_t backward_terms = 0U;
    std::size_t divisions = 0U;
    std::size_t audits = 0U;
    std::size_t containments = 0U;
    std::vector<ALR20R63ZAScalar> scaled_rhs;
    std::vector<ALR20R63ZAScalar> intermediate;
    std::vector<ALR20R63ZAScalar> permuted_solution;
    std::vector<ALR20R63ZAScalar> solution;
    double maximum_radius = 0.0;
    Binary128 maximum_reference_difference = static_cast<Binary128>(0.0);
    std::string finite_root;
    std::string root;
};

bool al_r20_r63za_contains(
    const ALR20R63ZAScalar& finite, Binary128 reference) {
    if (!al_r20_r63za_canonical(finite)) return false;
    const Binary128 center = static_cast<Binary128>(finite.high)
        + static_cast<Binary128>(finite.low);
    const ALR20R37Conversion exact_reference =
        al_r20_r37_from_binary128(reference);
    const ALR20R37Dyadic one = al_r20_r63p_control_dyadic(1);
    return exact_reference.exact && al_r20_r63w_contains_fraction(
        center, static_cast<Binary128>(finite.radius),
        exact_reference.value, one);
}

ALR20R63ZASolve al_r20_r63za_solve(
    const std::string& role, const std::vector<double>& upper,
    const std::vector<std::size_t>& permutation,
    Binary128 inverse_scale, const std::vector<Binary128>& source,
    const ALR20R63IQSolve& reference) {
    ALR20R63ZASolve result;
    result.role = role;
    result.dimension = source.size();
    const std::size_t dimension = result.dimension;
    if (role.empty() || dimension == 0U
        || upper.size() != dimension * dimension
        || !al_r20_r63g_permutation_exact(permutation, dimension)
        || !reference.exact || reference.dimension != dimension)
        return result;
    const ALR20R63ZAScalar scale = al_r20_r63za_project(inverse_scale);
    result.scaled_rhs.resize(dimension);
    result.intermediate.resize(dimension);
    result.permuted_solution.resize(dimension);
    result.solution.resize(dimension);
    bool exact = al_r20_r63za_canonical(scale);
    for (std::size_t row = 0U; row < dimension && exact; ++row) {
        const ALR20R63ZAScalar input = al_r20_r63za_project(
            source[permutation[row]]);
        result.scaled_rhs[row] = al_r20_r63za_multiply(input, scale);
        ALR20R63ZAScalar numerator = result.scaled_rhs[row];
        for (std::size_t column = 0U; column < row; ++column) {
            const ALR20R63ZAScalar term = al_r20_r63za_multiply(
                al_r20_r63za_exact_double(
                    upper[column * dimension + row]),
                result.intermediate[column]);
            numerator = al_r20_r63za_add(numerator, term, -1.0);
            ++result.forward_terms;
        }
        result.intermediate[row] = al_r20_r63za_divide(numerator,
            al_r20_r63za_exact_double(upper[row * dimension + row]));
        exact = al_r20_r63za_canonical(result.scaled_rhs[row])
            && al_r20_r63za_canonical(result.intermediate[row]);
        ++result.divisions;
    }
    for (std::size_t reverse = dimension; reverse-- > 0U && exact;) {
        ALR20R63ZAScalar numerator = result.intermediate[reverse];
        for (std::size_t column = reverse + 1U;
             column < dimension; ++column) {
            const ALR20R63ZAScalar term = al_r20_r63za_multiply(
                al_r20_r63za_exact_double(
                    upper[reverse * dimension + column]),
                result.permuted_solution[column]);
            numerator = al_r20_r63za_add(numerator, term, -1.0);
            ++result.backward_terms;
        }
        result.permuted_solution[reverse] = al_r20_r63za_divide(numerator,
            al_r20_r63za_exact_double(
                upper[reverse * dimension + reverse]));
        exact = al_r20_r63za_canonical(result.permuted_solution[reverse]);
        ++result.divisions;
    }
    for (std::size_t index = 0U; index < dimension; ++index)
        result.solution[permutation[index]] = result.permuted_solution[index];
    if (exact && reference.intermediate.size() == dimension
        && reference.solution.size() == dimension) {
        for (std::size_t index = 0U; index < dimension; ++index) {
            const bool intermediate_contained = al_r20_r63za_contains(
                result.intermediate[index], reference.intermediate[index]);
            const bool solution_contained = al_r20_r63za_contains(
                result.solution[index], reference.solution[index]);
            result.audits += 2U;
            result.containments += intermediate_contained ? 1U : 0U;
            result.containments += solution_contained ? 1U : 0U;
            const Binary128 intermediate_center =
                static_cast<Binary128>(result.intermediate[index].high)
                + static_cast<Binary128>(result.intermediate[index].low);
            const Binary128 solution_center =
                static_cast<Binary128>(result.solution[index].high)
                + static_cast<Binary128>(result.solution[index].low);
            result.maximum_reference_difference = fmaxq(
                result.maximum_reference_difference,
                fmaxq(fabsq(reference.intermediate[index]
                            - intermediate_center),
                    fabsq(reference.solution[index] - solution_center)));
            result.maximum_radius = std::max(result.maximum_radius,
                std::max(result.intermediate[index].radius,
                    result.solution[index].radius));
        }
    }
    result.contained = result.audits == 2U * dimension
        && result.containments == 2U * dimension;
    result.exact = exact
        && result.forward_terms == dimension * (dimension - 1U) / 2U
        && result.backward_terms == dimension * (dimension - 1U) / 2U
        && result.divisions == 2U * dimension;
    std::ostringstream finite_material;
    finite_material << result.exact << ':' << role << ':'
        << result.dimension << ':' << result.forward_terms << ':'
        << result.backward_terms << ':' << result.divisions << '|';
    for (std::size_t index = 0U; index < dimension; ++index)
        finite_material << result.intermediate[index].root << ':'
            << result.solution[index].root << ';';
    result.finite_root = sha256_hex(finite_material.str());
    std::ostringstream material;
    material << result.exact << ':' << result.contained << ':'
        << result.audits << ':' << result.containments << ':'
        << binary64_bits(result.maximum_radius) << ':' << result.finite_root
        << ':' << al_binary128_hex(result.maximum_reference_difference)
        << ':' << reference.root;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63ZAIdentity {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t width = 0U;
    std::string parent_semantic;
    std::string operator_root;
    std::string factor_root;
    std::string permutation_root;
    std::string lane_root;
    std::string order;
    std::string root;
};

std::string al_r20_r63za_identity_root(
    const ALR20R63ZAIdentity& identity) {
    std::ostringstream material;
    material << identity.dimension << ':' << identity.width << ':'
        << identity.parent_semantic << ':' << identity.operator_root << ':'
        << identity.factor_root << ':' << identity.permutation_root << ':'
        << identity.lane_root << ':' << identity.order;
    return sha256_hex(material.str());
}

bool al_r20_r63za_identity_valid(const ALR20R63ZAIdentity& identity) {
    return identity.exact && identity.dimension == AL_R20_R63ZA_DIMENSION
        && identity.width == 2U
        && identity.parent_semantic == AL_R20_R63ZA_PARENT_SEMANTIC_SHA256
        && identity.operator_root == AL_R20_R63ZA_PARENT_ARTIFACT_SHA256
        && identity.factor_root == AL_R20_R63ZA_EXPORTED_FACTOR_SHA256
        && identity.permutation_root == AL_R20_R63ZA_PERMUTATION_SHA256
        && identity.lane_root == AL_R20_R63ZA_EXPORTED_LANE_SHA256
        && identity.order
            == "subject,forward/backward,row,column,component"
        && identity.root == al_r20_r63za_identity_root(identity);
}

ALR20R63ZAIdentity al_r20_r63za_identity() {
    ALR20R63ZAIdentity result;
    result.dimension = AL_R20_R63ZA_DIMENSION;
    result.width = 2U;
    result.parent_semantic = AL_R20_R63ZA_PARENT_SEMANTIC_SHA256;
    result.operator_root = AL_R20_R63ZA_PARENT_ARTIFACT_SHA256;
    result.factor_root = AL_R20_R63ZA_EXPORTED_FACTOR_SHA256;
    result.permutation_root = AL_R20_R63ZA_PERMUTATION_SHA256;
    result.lane_root = AL_R20_R63ZA_EXPORTED_LANE_SHA256;
    result.order = "subject,forward/backward,row,column,component";
    result.exact = true;
    result.root = al_r20_r63za_identity_root(result);
    result.exact = al_r20_r63za_identity_valid(result);
    return result;
}

std::string al_r20_r63za_classify(bool apparatus, bool identity,
    bool arithmetic, bool start, bool initial, bool iteration_one) {
    if (!apparatus) return "FACTOR_CONSUMPTION_APPARATUS_REJECTED";
    if (!identity) return "FACTOR_CONSUMPTION_IDENTITY_REJECTED";
    if (!arithmetic) return "TWOFOLD_TRIANGULAR_ARITHMETIC_REJECTED";
    if (!start) return "START_FACTOR_CONSUMPTION_REJECTED";
    if (!initial) return "INITIAL_RESIDUAL_FACTOR_CONSUMPTION_REJECTED";
    if (!iteration_one)
        return "ITERATION1_RESIDUAL_FACTOR_CONSUMPTION_REJECTED";
    return "TWOFOLD_EXPORTED_FACTOR_CONSUMPTION_CANDIDATE";
}

struct ALR20R63ZAControls {
    bool exact = false;
    bool scalar = false;
    bool solve = false;
    bool cancellation = false;
    bool division = false;
    bool dropped_radius = false;
    bool invalid = false;
    bool overflow = false;
    bool orientation = false;
    bool stale = false;
    bool mutation = false;
    bool oracle_independence = false;
    bool classifier = false;
    std::string root;
};

ALR20R63ZAControls al_r20_r63za_controls() {
    ALR20R63ZAControls result;
    const Binary128 one_q = static_cast<Binary128>(1.0);
    const Binary128 three_q = static_cast<Binary128>(3.0);
    const ALR20R63ZAScalar one = al_r20_r63za_project(one_q);
    const ALR20R63ZAScalar third = al_r20_r63za_divide(
        one, al_r20_r63za_project(three_q));
    result.scalar = al_r20_r63za_canonical(one)
        && al_r20_r63za_canonical(third);
    result.division = result.scalar
        && al_r20_r63za_contains(third, one_q / three_q)
        && third.low != 0.0 && third.radius > 0.0;

    const Binary128 next = nextafterq(one_q, HUGE_VALQ);
    const ALR20R63ZAScalar cancellation = al_r20_r63za_add(
        al_r20_r63za_project(next), one, -1.0);
    result.cancellation = al_r20_r63za_canonical(cancellation)
        && cancellation.high != 0.0
        && al_r20_r63za_contains(cancellation, next - one_q);

    const std::vector<double> upper{2.0, 0.5, 0.0, 1.0};
    const std::vector<std::size_t> permutation{1U, 0U};
    const std::vector<Binary128> source{
        static_cast<Binary128>(3.0), static_cast<Binary128>(2.0)};
    const std::vector<Binary128> upper_q = al_r20_r63g_q_from_double(upper);
    const ALR20R63IQSolve reference = al_r20_r63i_q_solve(
        upper_q, permutation, source, one_q);
    const ALR20R63ZASolve solve = al_r20_r63za_solve(
        "control", upper, permutation, one_q, source, reference);
    result.solve = solve.exact && solve.containments == 4U
        && solve.audits == 4U
        && solve.forward_terms == 1U && solve.backward_terms == 1U
        && solve.divisions == 4U;

    ALR20R63ZAScalar dropped = third;
    dropped.radius = 0.0;
    dropped.root = al_r20_r63za_scalar_root(dropped);
    result.dropped_radius = al_r20_r63za_canonical(dropped)
        && !al_r20_r63za_contains(dropped, one_q / three_q);
    const ALR20R63ZAScalar zero = al_r20_r63za_exact_double(0.0);
    ALR20R63ZAScalar negative = one;
    negative.radius = -1.0;
    negative.root = al_r20_r63za_scalar_root(negative);
    const ALR20R63ZAScalar subnormal = al_r20_r63za_exact_double(
        std::numeric_limits<double>::denorm_min());
    const ALR20R63ZAScalar infinite = al_r20_r63za_exact_double(
        std::numeric_limits<double>::infinity());
    result.invalid = !al_r20_r63za_divide(one, zero).exact
        && !al_r20_r63za_canonical(negative)
        && !subnormal.exact && !infinite.exact;
    result.overflow = !al_r20_r63za_multiply(
        al_r20_r63za_exact_double(std::numeric_limits<double>::max()),
        al_r20_r63za_exact_double(std::numeric_limits<double>::max())).exact;

    const std::vector<double> changed_upper{2.0, 0.25, 0.0, 1.0};
    const std::vector<Binary128> changed_upper_q =
        al_r20_r63g_q_from_double(changed_upper);
    const ALR20R63IQSolve changed_reference = al_r20_r63i_q_solve(
        changed_upper_q, permutation, source, one_q);
    const ALR20R63ZASolve changed = al_r20_r63za_solve(
        "control", changed_upper, permutation, one_q,
        source, changed_reference);
    result.orientation = changed.exact && changed.root != solve.root;

    ALR20R63ZAIdentity identity = al_r20_r63za_identity();
    ALR20R63ZAIdentity stale = identity;
    stale.order = "stale-order";
    result.stale = al_r20_r63za_identity_valid(identity)
        && !al_r20_r63za_identity_valid(stale);
    ALR20R63ZAScalar mutated = third;
    mutated.low = std::nextafter(
        mutated.low, std::numeric_limits<double>::infinity());
    result.mutation = mutated.root == third.root
        && !al_r20_r63za_canonical(mutated);

    const std::string finite_root = solve.root;
    ALR20R63IQSolve oracle_mutation = reference;
    oracle_mutation.solution[0U] += static_cast<Binary128>(1.0);
    const ALR20R63ZASolve rejected = al_r20_r63za_solve(
        "control", upper, permutation, one_q, source, oracle_mutation);
    result.oracle_independence = rejected.root != finite_root
        && rejected.finite_root == solve.finite_root
        && solve.root == finite_root;
    result.classifier =
        al_r20_r63za_classify(false, false, false, false, false, false)
            == "FACTOR_CONSUMPTION_APPARATUS_REJECTED"
        && al_r20_r63za_classify(true, false, false, false, false, false)
            == "FACTOR_CONSUMPTION_IDENTITY_REJECTED"
        && al_r20_r63za_classify(true, true, false, false, false, false)
            == "TWOFOLD_TRIANGULAR_ARITHMETIC_REJECTED"
        && al_r20_r63za_classify(true, true, true, false, true, true)
            == "START_FACTOR_CONSUMPTION_REJECTED"
        && al_r20_r63za_classify(true, true, true, true, false, true)
            == "INITIAL_RESIDUAL_FACTOR_CONSUMPTION_REJECTED"
        && al_r20_r63za_classify(true, true, true, true, true, false)
            == "ITERATION1_RESIDUAL_FACTOR_CONSUMPTION_REJECTED"
        && al_r20_r63za_classify(true, true, true, true, true, true)
            == "TWOFOLD_EXPORTED_FACTOR_CONSUMPTION_CANDIDATE";
    result.exact = result.scalar && result.solve && result.cancellation
        && result.division && result.dropped_radius && result.invalid
        && result.overflow && result.orientation && result.stale
        && result.mutation && result.oracle_independence
        && result.classifier;
    std::ostringstream material;
    material << result.exact << ':' << result.scalar << ':' << result.solve
        << ':' << result.cancellation << ':' << result.division << ':'
        << result.dropped_radius << ':' << result.invalid << ':'
        << result.overflow << ':' << result.orientation << ':'
        << result.stale << ':' << result.mutation << ':'
        << result.oracle_independence << ':' << result.classifier << ':'
        << third.root << ':' << cancellation.root << ':' << solve.root << ':'
        << changed.root << ':' << identity.root;
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

SplitBoundaryReport
run_al_generalization_v5_finite_factor_consumption_controls() {
#if defined(__linux__) && defined(__x86_64__) && defined(__GNUC__) \
    && defined(__SIZEOF_FLOAT128__) && !defined(__FAST_MATH__)
    constexpr bool frozen_platform = true;
#else
    constexpr bool frozen_platform = false;
#endif
    const bool profile_exact = frozen_platform
        && sizeof(Binary128) == 16U && FLT128_MANT_DIG == 113
        && sizeof(double) == 8U && FLT_RADIX == 2 && DBL_MANT_DIG == 53
        && FLT_EVAL_METHOD == 0 && std::numeric_limits<double>::is_iec559
        && std::numeric_limits<double>::has_denorm == std::denorm_present
        && std::fegetround() == FE_TONEAREST;
    const ALR20R63ZAControls controls = al_r20_r63za_controls();
    const SplitBoundaryReport parent =
        run_al_generalization_v5_active_block_operator_controls();
    const ALR20R63ZLastResult parent_result = al_r20_r63z_last_result;
    const ALR20R63SConsumptionFixture fixture =
        al_r20_r63s_consumption_fixture;
    const bool parent_exact = parent.passed && parent_result.exact
        && parent_result.semantic == AL_R20_R63ZA_PARENT_SEMANTIC_SHA256
        && parent_result.artifact_root == AL_R20_R63ZA_PARENT_ARTIFACT_SHA256;
    const ALR20R63ZAIdentity identity = al_r20_r63za_identity();
    const bool identity_exact = parent_exact && fixture.exact
        && al_r20_r63za_identity_valid(identity)
        && fixture.dimension == AL_R20_R63ZA_DIMENSION
        && al_r20_double_root(fixture.exported_upper)
            == AL_R20_R63ZA_EXPORTED_FACTOR_SHA256
        && al_r20_r63g_index_root(fixture.permutation)
            == AL_R20_R63ZA_PERMUTATION_SHA256
        && fixture.exported_lane_root == AL_R20_R63ZA_EXPORTED_LANE_SHA256
        && fixture.sources.size() == 3U && fixture.references.size() == 3U;
    static const std::array<const char*, 3U> roles{
        "start", "initial_residual", "iteration1_residual"};
    std::vector<ALR20R63ZASolve> solves;
    if (identity_exact)
        for (std::size_t subject = 0U; subject < 3U; ++subject)
            solves.push_back(al_r20_r63za_solve(roles[subject],
                fixture.exported_upper, fixture.permutation,
                fixture.inverse_scale, fixture.sources[subject],
                fixture.references[subject]));
    const bool arithmetic_exact = solves.size() == 3U
        && std::all_of(solves.begin(), solves.end(),
            [](const ALR20R63ZASolve& value) { return value.exact; });
    const bool start_exact = solves.size() == 3U && solves[0U].contained;
    const bool initial_exact = solves.size() == 3U && solves[1U].contained;
    const bool iteration_one_exact = solves.size() == 3U
        && solves[2U].contained;
    std::size_t forward_terms = 0U;
    std::size_t backward_terms = 0U;
    std::size_t divisions = 0U;
    std::size_t audits = 0U;
    std::size_t containments = 0U;
    for (const ALR20R63ZASolve& solve : solves) {
        forward_terms += solve.forward_terms;
        backward_terms += solve.backward_terms;
        divisions += solve.divisions;
        audits += solve.audits;
        containments += solve.containments;
    }
    const bool work_exact = solves.size() == 3U
        && forward_terms == 15453U && backward_terms == 15453U
        && divisions == 612U && audits == 612U;
    const bool apparatus_exact = profile_exact && controls.exact
        && parent_exact && fixture.exact && work_exact;
    const std::string route = al_r20_r63za_classify(apparatus_exact,
        identity_exact, arithmetic_exact, start_exact, initial_exact,
        iteration_one_exact);
    const bool exact = apparatus_exact && identity_exact && arithmetic_exact
        && start_exact && initial_exact && iteration_one_exact;
    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63za-finite-factor-consumption.v1|"
        << parent_result.semantic << ':' << route << ':' << controls.root
        << ':' << identity.root << ':' << fixture.root << '|';
    for (const ALR20R63ZASolve& solve : solves)
        semantic << solve.root << ';';
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63za_finite_factor_consumption.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63z_sha256\":\"" << parent_result.semantic << "\""
        << ",\"controls_exact\":" << (controls.exact ? "true" : "false")
        << ",\"controls_root\":\"" << controls.root << "\""
        << ",\"controls\":{\"scalar\":"
        << (controls.scalar ? "true" : "false")
        << ",\"solve\":" << (controls.solve ? "true" : "false")
        << ",\"cancellation\":"
        << (controls.cancellation ? "true" : "false")
        << ",\"division\":" << (controls.division ? "true" : "false")
        << ",\"dropped_radius\":"
        << (controls.dropped_radius ? "true" : "false")
        << ",\"invalid\":" << (controls.invalid ? "true" : "false")
        << ",\"overflow\":" << (controls.overflow ? "true" : "false")
        << ",\"orientation\":"
        << (controls.orientation ? "true" : "false")
        << ",\"stale\":" << (controls.stale ? "true" : "false")
        << ",\"mutation\":" << (controls.mutation ? "true" : "false")
        << ",\"oracle_independence\":"
        << (controls.oracle_independence ? "true" : "false")
        << ",\"classifier\":"
        << (controls.classifier ? "true" : "false") << '}'
        << ",\"identity_exact\":" << (identity_exact ? "true" : "false")
        << ",\"identity_root\":\"" << identity.root << "\""
        << ",\"fixture_root\":\"" << fixture.root << "\""
        << ",\"solves\":[";
    for (std::size_t index = 0U; index < solves.size(); ++index) {
        if (index != 0U) report << ',';
        report << "{\"role\":\"" << solves[index].role << "\""
            << ",\"exact\":" << (solves[index].exact ? "true" : "false")
            << ",\"contained\":"
            << (solves[index].contained ? "true" : "false")
            << ",\"containments\":" << solves[index].containments
            << ",\"maximum_radius\":" << solves[index].maximum_radius
            << ",\"maximum_reference_difference\":\""
            << al_binary128_text(
                solves[index].maximum_reference_difference) << "\""
            << ",\"root\":\"" << solves[index].root << "\"}";
    }
    report << "]"
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"arithmetic_exact\":"
        << (arithmetic_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"forward_terms\":" << forward_terms
        << ",\"backward_terms\":" << backward_terms
        << ",\"divisions\":" << divisions
        << ",\"row_equation_audits\":" << audits
        << ",\"reference_containments\":" << containments
        << ",\"operator_products\":0,\"pcg_scalar_dots\":0"
        << ",\"candidate_updates\":0,\"factor_builds\":0"
        << ",\"exact_oracle_classification_uses\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"gpu_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
