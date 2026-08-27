namespace {

constexpr const char* AL_R20_R63W_PARENT_SEMANTIC_SHA256 =
    "905ce406943303ac51b5f33a1580ddc438ad8e6adb6d38ed2b8eb179954acad8";
constexpr const char* AL_R20_R63W_WIDE_LANE_SHA256 =
    "22d3b114dcec5845183fa72982203242f8c608fdeb0b6c1f6cbbf37ccc75d8a0";
constexpr const char* AL_R20_R63W_EXPORTED_LANE_SHA256 =
    "1f6a7041734089ed0a07bfc5fb6ecce51ea742376fa31b2f8efe1353520407d6";
constexpr const char* AL_R20_R63W_SIGN_SHA256 =
    "89b2908b21369cf77a376287ed8142026827815c45a59aa70d2e831aac406094";

Binary128 al_r20_r63w_down_add(Binary128 left, Binary128 right) {
    if (left == static_cast<Binary128>(0.0)
        && right == static_cast<Binary128>(0.0))
        return static_cast<Binary128>(0.0);
    return nextafterq(left + right, -HUGE_VALQ);
}

Binary128 al_r20_r63w_positive_dyadic_down(
    const ALR20R37Dyadic& value) {
    if (al_r20_r63p_sign(value) <= 0)
        return static_cast<Binary128>(0.0);
    return nextafterq(al_r20_r37_binary128_up(value),
        static_cast<Binary128>(0.0));
}

int al_r20_r63w_compare_signed(
    const ALR20R37Dyadic& left, const ALR20R37Dyadic& right) {
    const int left_sign = al_r20_r63p_sign(left);
    const int right_sign = al_r20_r63p_sign(right);
    if (left_sign < right_sign) return -1;
    if (left_sign > right_sign) return 1;
    if (left_sign == 0) return 0;
    const int magnitude = al_r20_r37_compare_positive(
        al_r20_r37_absolute(left), al_r20_r37_absolute(right));
    return left_sign > 0 ? magnitude : -magnitude;
}

struct ALR20R63WProfile {
    bool exact = false;
    bool contained = false;
    bool contractive = false;
    std::size_t dimension = 0U;
    std::size_t matrix_entries = 0U;
    std::size_t matrix_containments = 0U;
    std::vector<Binary128> matrix_center;
    std::vector<Binary128> matrix_radius;
    Binary128 rho_upper = static_cast<Binary128>(0.0);
    Binary128 maximum_matrix_radius = static_cast<Binary128>(0.0);
    std::string center_root;
    std::string radius_root;
    std::string root;
};

ALR20R63WProfile al_r20_r63w_profile(
    const ALR20R63QExactMatrix& matrix,
    const ALR20R63RDefect& defect) {
    ALR20R63WProfile result;
    result.dimension = matrix.rows;
    const std::size_t dimension = result.dimension;
    if (!matrix.exact || !defect.exact || !defect.left_contractive
        || dimension == 0U
        || matrix.oracle_numerator.size() != dimension * dimension
        || !al_r20_r63p_equal(matrix.d, defect.d)
        || al_r20_r63p_sign(matrix.d) <= 0) return result;
    const Binary128 denominator_down =
        al_r20_r63w_positive_dyadic_down(matrix.d);
    if (!(denominator_down > static_cast<Binary128>(0.0))) return result;
    result.matrix_center.resize(dimension * dimension);
    result.matrix_radius.resize(dimension * dimension);
    bool entries_exact = true;
    for (std::size_t index = 0U;
         index < matrix.oracle_numerator.size(); ++index) {
        const ALR20R37Dyadic& numerator = matrix.oracle_numerator[index];
        const Binary128 center = al_r20_r63p_fraction_projection(
            numerator, matrix.d);
        const ALR20R37Conversion center_exact =
            al_r20_r37_from_binary128(center);
        entries_exact = entries_exact && center_exact.exact;
        if (!center_exact.exact) continue;
        const ALR20R37Dyadic error = al_r20_r37_absolute(
            al_r20_r63p_subtract(
                al_r20_r37_multiply(center_exact.value, matrix.d),
                numerator));
        const Binary128 radius = error.numerator == 0
            ? static_cast<Binary128>(0.0)
            : al_r20_r38_up_divide(
                al_r20_r37_binary128_up(error), denominator_down);
        const ALR20R37Conversion radius_exact =
            al_r20_r37_from_binary128(radius);
        const bool contained = radius_exact.exact
            && radius >= static_cast<Binary128>(0.0)
            && al_r20_r37_compare_positive(
                al_r20_r37_multiply(radius_exact.value, matrix.d),
                error) >= 0;
        entries_exact = entries_exact && contained
            && al_r20_r38_normal_or_zero(center)
            && al_r20_r38_normal_or_zero(radius);
        result.matrix_center[index] = center;
        result.matrix_radius[index] = radius;
        result.maximum_matrix_radius = fmaxq(
            result.maximum_matrix_radius, radius);
        ++result.matrix_entries;
        result.matrix_containments += contained ? 1U : 0U;
    }
    const Binary128 defect_numerator_up = al_r20_r37_binary128_up(
        defect.left_infinity_numerator);
    result.rho_upper = al_r20_r38_up_divide(
        defect_numerator_up, denominator_down);
    const ALR20R37Conversion rho_exact =
        al_r20_r37_from_binary128(result.rho_upper);
    const bool rho_contained = rho_exact.exact
        && al_r20_r37_compare_positive(
            al_r20_r37_multiply(rho_exact.value, matrix.d),
            defect.left_infinity_numerator) >= 0;
    result.contained = entries_exact && rho_contained
        && result.matrix_containments == dimension * dimension;
    result.contractive = result.rho_upper >= static_cast<Binary128>(0.0)
        && result.rho_upper < static_cast<Binary128>(1.0);
    result.center_root = al_r20_q_scalar_root(result.matrix_center);
    result.radius_root = al_r20_q_scalar_root(result.matrix_radius);
    result.exact = result.contained && result.contractive
        && result.matrix_entries == dimension * dimension
        && al_r20_r38_normal_or_zero(result.rho_upper)
        && al_r20_r38_normal_or_zero(result.maximum_matrix_radius);
    std::ostringstream material;
    material << result.exact << ':' << result.contained << ':'
        << result.contractive << ':' << result.dimension << ':'
        << result.matrix_entries << ':' << result.matrix_containments << ':'
        << al_binary128_hex(result.rho_upper) << ':'
        << al_binary128_hex(result.maximum_matrix_radius) << ':'
        << result.center_root << ':' << result.radius_root;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63WCertificate {
    bool exact = false;
    bool no_underflow = false;
    bool passed = false;
    std::size_t dimension = 0U;
    std::size_t residual_dots = 0U;
    std::size_t residual_radius_terms = 0U;
    std::size_t image_dots = 0U;
    std::size_t image_radius_terms = 0U;
    std::size_t sign_comparisons = 0U;
    std::size_t positive = 0U;
    std::size_t negative = 0U;
    std::size_t unresolved = 0U;
    Binary128 image_infinity_upper = static_cast<Binary128>(0.0);
    Binary128 denominator_lower = static_cast<Binary128>(0.0);
    Binary128 error_upper = static_cast<Binary128>(0.0);
    Binary128 minimum_separation = HUGE_VALQ;
    std::vector<Binary128> residual_center;
    std::vector<Binary128> residual_radius;
    std::vector<Binary128> image_center;
    std::vector<Binary128> image_radius;
    std::vector<int> signs;
    std::string solution_root;
    std::string residual_dot_root;
    std::string image_dot_root;
    std::string sign_root;
    std::string root;
};

ALR20R63WCertificate al_r20_r63w_certificate(
    const ALR20R63WProfile& profile,
    const std::vector<Binary128>& inverse,
    const std::vector<Binary128>& rhs,
    const std::vector<Binary128>& solution) {
    ALR20R63WCertificate result;
    result.dimension = rhs.size();
    const std::size_t dimension = result.dimension;
    if (!profile.exact || profile.dimension != dimension || dimension == 0U
        || inverse.size() != dimension * dimension
        || profile.matrix_center.size() != dimension * dimension
        || profile.matrix_radius.size() != dimension * dimension
        || solution.size() != dimension) return result;
    bool exact = true;
    bool no_underflow = true;
    if (!(profile.rho_upper >= static_cast<Binary128>(0.0))
        || !(profile.rho_upper < static_cast<Binary128>(1.0))) exact = false;
    for (std::size_t index = 0U; index < dimension * dimension; ++index) {
        exact = exact
            && profile.matrix_radius[index]
                >= static_cast<Binary128>(0.0)
            && al_r20_r38_normal_or_zero(profile.matrix_center[index])
            && al_r20_r38_normal_or_zero(profile.matrix_radius[index])
            && al_r20_r38_normal_or_zero(inverse[index]);
    }
    for (std::size_t index = 0U; index < dimension; ++index)
        exact = exact && al_r20_r38_normal_or_zero(rhs[index])
            && al_r20_r38_normal_or_zero(solution[index]);
    result.residual_center.resize(dimension);
    result.residual_radius.resize(dimension);
    std::ostringstream residual_material;
    for (std::size_t row = 0U; row < dimension; ++row) {
        std::vector<Binary128> left(dimension + 1U);
        std::vector<Binary128> right(dimension + 1U);
        left[0U] = rhs[row];
        right[0U] = static_cast<Binary128>(1.0);
        for (std::size_t column = 0U; column < dimension; ++column) {
            left[column + 1U] =
                profile.matrix_center[row * dimension + column];
            right[column + 1U] = -solution[column];
        }
        const ALR20R38Dot2 dot = al_r20_r38_dot2(left, right);
        ++result.residual_dots;
        exact = exact && dot.exact;
        no_underflow = no_underflow && dot.no_underflow;
        result.residual_center[row] = dot.value;
        Binary128 radius = dot.bound;
        for (std::size_t column = 0U; column < dimension; ++column) {
            radius = al_r20_r38_up_add(radius,
                al_r20_r38_up_multiply(
                    profile.matrix_radius[row * dimension + column],
                    fabsq(solution[column])));
            ++result.residual_radius_terms;
        }
        result.residual_radius[row] = radius;
        exact = exact && radius >= static_cast<Binary128>(0.0)
            && al_r20_r38_normal_or_zero(dot.value)
            && al_r20_r38_normal_or_zero(radius);
        residual_material << dot.root << ':'
            << al_binary128_hex(radius) << ';';
    }
    result.image_center.resize(dimension);
    result.image_radius.resize(dimension);
    std::ostringstream image_material;
    for (std::size_t row = 0U; row < dimension; ++row) {
        std::vector<Binary128> left(dimension);
        for (std::size_t column = 0U; column < dimension; ++column)
            left[column] = inverse[row * dimension + column];
        const ALR20R38Dot2 dot = al_r20_r38_dot2(
            left, result.residual_center);
        ++result.image_dots;
        exact = exact && dot.exact;
        no_underflow = no_underflow && dot.no_underflow;
        result.image_center[row] = dot.value;
        Binary128 radius = dot.bound;
        for (std::size_t column = 0U; column < dimension; ++column) {
            radius = al_r20_r38_up_add(radius,
                al_r20_r38_up_multiply(
                    fabsq(inverse[row * dimension + column]),
                    result.residual_radius[column]));
            ++result.image_radius_terms;
        }
        result.image_radius[row] = radius;
        const Binary128 magnitude = al_r20_r38_up_add(
            fabsq(dot.value), radius);
        result.image_infinity_upper = fmaxq(
            result.image_infinity_upper, magnitude);
        exact = exact && radius >= static_cast<Binary128>(0.0)
            && al_r20_r38_normal_or_zero(dot.value)
            && al_r20_r38_normal_or_zero(radius)
            && al_r20_r38_normal_or_zero(magnitude);
        image_material << dot.root << ':'
            << al_binary128_hex(radius) << ';';
    }
    result.denominator_lower = nextafterq(
        static_cast<Binary128>(1.0) - profile.rho_upper, -HUGE_VALQ);
    if (!(result.denominator_lower > static_cast<Binary128>(0.0)))
        exact = false;
    if (exact)
        result.error_upper = al_r20_r38_up_divide(
            result.image_infinity_upper, result.denominator_lower);
    result.signs.resize(dimension);
    for (std::size_t index = 0U; index < dimension; ++index) {
        const Binary128 lower = al_r20_r63w_down_add(
            solution[index], -result.error_upper);
        const Binary128 upper = al_r20_r38_up_add(
            solution[index], result.error_upper);
        if (lower > static_cast<Binary128>(0.0)) {
            result.signs[index] = 1;
            ++result.positive;
            result.minimum_separation = fminq(
                result.minimum_separation, lower);
        } else if (upper < static_cast<Binary128>(0.0)) {
            result.signs[index] = -1;
            ++result.negative;
            result.minimum_separation = fminq(
                result.minimum_separation, -upper);
        } else {
            ++result.unresolved;
        }
        ++result.sign_comparisons;
    }
    result.passed = exact && result.unresolved == 0U
        && result.positive + result.negative == dimension;
    result.solution_root = al_r20_q_scalar_root(solution);
    result.residual_dot_root = sha256_hex(residual_material.str());
    result.image_dot_root = sha256_hex(image_material.str());
    std::ostringstream sign_material;
    for (int sign : result.signs) sign_material << sign << ',';
    result.sign_root = sha256_hex(sign_material.str());
    result.no_underflow = no_underflow;
    result.exact = exact && no_underflow
        && result.residual_dots == dimension
        && result.residual_radius_terms == dimension * dimension
        && result.image_dots == dimension
        && result.image_radius_terms == dimension * dimension
        && result.sign_comparisons == dimension
        && result.positive + result.negative + result.unresolved == dimension
        && al_r20_r38_normal_or_zero(result.image_infinity_upper)
        && al_r20_r38_normal_or_zero(result.denominator_lower)
        && al_r20_r38_normal_or_zero(result.error_upper);
    std::ostringstream material;
    material << result.exact << ':' << result.no_underflow << ':'
        << result.passed << ':' << result.dimension << ':'
        << result.residual_dots << ':' << result.residual_radius_terms << ':'
        << result.image_dots << ':' << result.image_radius_terms << ':'
        << result.sign_comparisons << ':' << result.positive << ':'
        << result.negative << ':' << result.unresolved << ':'
        << al_binary128_hex(result.image_infinity_upper) << ':'
        << al_binary128_hex(result.denominator_lower) << ':'
        << al_binary128_hex(result.error_upper) << ':'
        << al_binary128_hex(result.minimum_separation) << ':'
        << result.solution_root << ':' << result.residual_dot_root << ':'
        << result.image_dot_root << ':' << result.sign_root << ':'
        << al_r20_q_scalar_root(result.residual_center) << ':'
        << al_r20_q_scalar_root(result.residual_radius) << ':'
        << al_r20_q_scalar_root(result.image_center) << ':'
        << al_r20_q_scalar_root(result.image_radius);
    result.root = sha256_hex(material.str());
    return result;
}

bool al_r20_r63w_contains_fraction(
    Binary128 center, Binary128 radius,
    const ALR20R37Dyadic& numerator,
    const ALR20R37Dyadic& denominator) {
    if (!(radius >= static_cast<Binary128>(0.0))
        || al_r20_r63p_sign(denominator) <= 0) return false;
    const Binary128 lower = al_r20_r63w_down_add(center, -radius);
    const Binary128 upper = al_r20_r38_up_add(center, radius);
    const ALR20R37Conversion lower_exact =
        al_r20_r37_from_binary128(lower);
    const ALR20R37Conversion upper_exact =
        al_r20_r37_from_binary128(upper);
    if (!lower_exact.exact || !upper_exact.exact) return false;
    const ALR20R37Dyadic scaled_lower = al_r20_r37_multiply(
        lower_exact.value, denominator);
    const ALR20R37Dyadic scaled_upper = al_r20_r37_multiply(
        upper_exact.value, denominator);
    return al_r20_r63w_compare_signed(numerator, scaled_lower) >= 0
        && al_r20_r63w_compare_signed(scaled_upper, numerator) >= 0;
}

struct ALR20R63WContainment {
    bool exact = false;
    bool residual_contained = false;
    bool image_contained = false;
    std::size_t dimension = 0U;
    std::size_t residual_entries = 0U;
    std::size_t image_entries = 0U;
    std::string root;
};

ALR20R63WContainment al_r20_r63w_containment(
    const ALR20R63WCertificate& finite,
    const ALR20R63UCertificate& oracle) {
    ALR20R63WContainment result;
    result.dimension = finite.dimension;
    const std::size_t dimension = result.dimension;
    if (!finite.exact || !oracle.exact || oracle.dimension != dimension
        || finite.residual_center.size() != dimension
        || finite.residual_radius.size() != dimension
        || finite.image_center.size() != dimension
        || finite.image_radius.size() != dimension
        || oracle.residual.size() != dimension
        || oracle.image.size() != dimension) return result;
    bool residual_contained = true;
    bool image_contained = true;
    std::ostringstream material;
    for (std::size_t index = 0U; index < dimension; ++index) {
        const bool residual = al_r20_r63w_contains_fraction(
            finite.residual_center[index], finite.residual_radius[index],
            oracle.residual[index], oracle.d);
        const bool image = al_r20_r63w_contains_fraction(
            finite.image_center[index], finite.image_radius[index],
            oracle.image[index], oracle.d);
        residual_contained = residual_contained && residual;
        image_contained = image_contained && image;
        result.residual_entries += residual ? 1U : 0U;
        result.image_entries += image ? 1U : 0U;
        material << index << ':' << residual << ':' << image << ';';
    }
    result.residual_contained = residual_contained
        && result.residual_entries == dimension;
    result.image_contained = image_contained
        && result.image_entries == dimension;
    result.exact = result.residual_contained && result.image_contained;
    material << '|' << result.exact << ':' << result.dimension << ':'
        << result.residual_entries << ':' << result.image_entries;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63WLane {
    bool exact = false;
    bool contained = false;
    bool passed = false;
    bool reject_reject_pass = false;
    std::size_t dimension = 0U;
    std::size_t states = 0U;
    std::size_t residual_containments = 0U;
    std::size_t image_containments = 0U;
    std::vector<ALR20R63WCertificate> certificates;
    std::vector<ALR20R63WContainment> containments;
    std::string root;
};

ALR20R63WLane al_r20_r63w_lane(
    const ALR20R63WProfile& profile,
    const std::vector<Binary128>& inverse,
    const std::vector<Binary128>& rhs,
    const std::vector<std::vector<Binary128>>& solutions,
    const std::vector<ALR20R63UCertificate>& oracles) {
    ALR20R63WLane result;
    result.dimension = rhs.size();
    if (solutions.size() != 3U || oracles.size() != 3U) return result;
    bool exact = true;
    bool contained = true;
    for (std::size_t state = 0U; state < 3U; ++state) {
        ALR20R63WCertificate certificate = al_r20_r63w_certificate(
            profile, inverse, rhs, solutions[state]);
        ALR20R63WContainment containment = al_r20_r63w_containment(
            certificate, oracles[state]);
        exact = exact && certificate.exact;
        contained = contained && containment.exact;
        result.residual_containments += containment.residual_entries;
        result.image_containments += containment.image_entries;
        result.certificates.push_back(std::move(certificate));
        result.containments.push_back(std::move(containment));
        ++result.states;
    }
    result.reject_reject_pass = !result.certificates[0U].passed
        && result.certificates[0U].unresolved > 0U
        && !result.certificates[1U].passed
        && result.certificates[1U].unresolved > 0U
        && result.certificates[2U].passed
        && result.certificates[2U].positive == 24U
        && result.certificates[2U].negative == 78U
        && result.certificates[2U].unresolved == 0U;
    result.contained = contained
        && result.residual_containments == 3U * result.dimension
        && result.image_containments == 3U * result.dimension;
    result.exact = exact && result.contained && result.states == 3U;
    result.passed = result.exact && result.reject_reject_pass;
    std::ostringstream material;
    material << result.exact << ':' << result.contained << ':'
        << result.passed << ':' << result.reject_reject_pass << ':'
        << result.dimension << ':' << result.states << ':'
        << result.residual_containments << ':'
        << result.image_containments << '|';
    for (std::size_t state = 0U; state < result.states; ++state)
        material << result.certificates[state].root << ':'
            << result.containments[state].root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r20_r63w_classify(bool apparatus, bool containment,
    bool certificate, bool wide, bool exported) {
    if (!apparatus) return "FINITE_IMAGE_APPARATUS_REJECTED";
    if (!containment) return "FINITE_IMAGE_CONTAINMENT_REJECTED";
    if (!certificate) return "FINITE_IMAGE_CERTIFICATE_REJECTED";
    if (!wide) return "RETAINED_WIDE_FINITE_IMAGE_REJECTED";
    if (!exported) return "EXPORTED_FACTOR_FINITE_IMAGE_REJECTED";
    return "TWO_LANE_FINITE_IMAGE_CANDIDATE";
}

struct ALR20R63WControls {
    bool exact = false;
    bool identity = false;
    bool propagation = false;
    bool cancellation = false;
    bool dropped_matrix_radius = false;
    bool dropped_image_radius = false;
    bool invalid = false;
    bool orientation = false;
    bool independence = false;
    bool mutation = false;
    bool classifier = false;
    bool identity_profile = false;
    bool identity_finite = false;
    bool identity_oracle = false;
    bool identity_containment = false;
    bool propagation_finite = false;
    bool propagation_containment = false;
    bool orientation_oracle = false;
    bool orientation_finite = false;
    bool orientation_containment = false;
    bool reversed_containment = false;
    bool mutated_containment = false;
    std::string root;
};

ALR20R63WControls al_r20_r63w_controls() {
    ALR20R63WControls result;
    const ALR20R37Dyadic zero = al_r20_r63p_control_dyadic(0);
    const ALR20R37Dyadic one = al_r20_r63p_control_dyadic(1);
    const ALR20R37Dyadic half = al_r20_r63p_control_dyadic(1, -1);
    const std::vector<ALR20R37Dyadic> identity{one, zero, zero, one};
    ALR20R63QExactMatrix matrix;
    matrix.exact = true;
    matrix.rows = 2U;
    matrix.columns = 2U;
    matrix.d = one;
    matrix.oracle_numerator = identity;
    const ALR20R63RDefect identity_defect = al_r20_r63r_defect(
        identity, identity, one, 2U);
    const ALR20R63WProfile profile = al_r20_r63w_profile(
        matrix, identity_defect);
    const std::vector<Binary128> inverse{
        static_cast<Binary128>(1.0), static_cast<Binary128>(0.0),
        static_cast<Binary128>(0.0), static_cast<Binary128>(1.0)};
    const std::vector<Binary128> rhs{
        static_cast<Binary128>(1.0), static_cast<Binary128>(-1.0)};
    const std::vector<Binary128> solution = rhs;
    const ALR20R63WCertificate identity_certificate =
        al_r20_r63w_certificate(profile, inverse, rhs, solution);
    const ALR20R63UCertificate identity_oracle = al_r20_r63u_certificate(
        {one, al_r20_r63p_control_dyadic(-1)}, matrix, identity_defect,
        identity, solution);
    const ALR20R63WContainment identity_containment =
        al_r20_r63w_containment(identity_certificate, identity_oracle);
    result.identity_profile = profile.exact;
    result.identity_finite = identity_certificate.exact;
    result.identity_oracle = identity_oracle.exact;
    result.identity_containment = identity_containment.exact;
    result.identity = profile.exact && identity_certificate.exact
        && identity_certificate.passed && identity_containment.exact;

    ALR20R63WProfile interval_profile;
    interval_profile.exact = true;
    interval_profile.dimension = 1U;
    interval_profile.matrix_center = {static_cast<Binary128>(0.5)};
    interval_profile.matrix_radius = {static_cast<Binary128>(0.5)};
    interval_profile.rho_upper = static_cast<Binary128>(0.0);
    const ALR20R63WCertificate interval = al_r20_r63w_certificate(
        interval_profile, {static_cast<Binary128>(1.0)},
        {static_cast<Binary128>(1.0)},
        {static_cast<Binary128>(1.0)});
    ALR20R63QExactMatrix scalar_matrix;
    scalar_matrix.exact = true;
    scalar_matrix.rows = 1U;
    scalar_matrix.columns = 1U;
    scalar_matrix.d = one;
    scalar_matrix.oracle_numerator = {one};
    const ALR20R63RDefect scalar_defect = al_r20_r63r_defect(
        {one}, {one}, one, 1U);
    const ALR20R63UCertificate scalar_oracle = al_r20_r63u_certificate(
        {one}, scalar_matrix, scalar_defect, {one},
        {static_cast<Binary128>(1.0)});
    const ALR20R63WContainment interval_containment =
        al_r20_r63w_containment(interval, scalar_oracle);
    result.propagation_finite = interval.exact;
    result.propagation_containment = interval_containment.exact;
    result.propagation = interval.exact && interval_containment.exact
        && interval.residual_radius[0U] >= static_cast<Binary128>(0.5)
        && interval.image_radius[0U] >= static_cast<Binary128>(0.5);
    const ALR20R63WCertificate cancellation = al_r20_r63w_certificate(
        profile, inverse, rhs,
        {nextafterq(static_cast<Binary128>(1.0), HUGE_VALQ),
            static_cast<Binary128>(-1.0)});
    result.cancellation = cancellation.exact
        && cancellation.residual_dots == 2U
        && cancellation.image_dots == 2U;
    ALR20R63WProfile dropped_profile = interval_profile;
    dropped_profile.matrix_radius[0U] = static_cast<Binary128>(0.0);
    const ALR20R63WCertificate dropped = al_r20_r63w_certificate(
        dropped_profile, {static_cast<Binary128>(1.0)},
        {static_cast<Binary128>(1.0)},
        {static_cast<Binary128>(1.0)});
    const ALR20R63WContainment dropped_containment =
        al_r20_r63w_containment(dropped, scalar_oracle);
    result.dropped_matrix_radius = dropped.exact
        && !dropped_containment.exact;
    ALR20R63WCertificate dropped_image = interval;
    dropped_image.image_radius[0U] = static_cast<Binary128>(0.0);
    const ALR20R63WContainment dropped_image_containment =
        al_r20_r63w_containment(dropped_image, scalar_oracle);
    result.dropped_image_radius = !dropped_image_containment.exact;
    ALR20R63WProfile negative = interval_profile;
    negative.matrix_radius[0U] = static_cast<Binary128>(-0.5);
    ALR20R63WProfile noncontractive = interval_profile;
    noncontractive.rho_upper = static_cast<Binary128>(1.0);
    const ALR20R63WCertificate negative_certificate =
        al_r20_r63w_certificate(negative,
            {static_cast<Binary128>(1.0)},
            {static_cast<Binary128>(1.0)},
            {static_cast<Binary128>(1.0)});
    const ALR20R63WCertificate noncontractive_certificate =
        al_r20_r63w_certificate(noncontractive,
            {static_cast<Binary128>(1.0)},
            {static_cast<Binary128>(1.0)},
            {static_cast<Binary128>(1.0)});
    result.invalid = !negative_certificate.exact
        && !noncontractive_certificate.exact;

    const std::vector<ALR20R37Dyadic> oriented_inverse{
        one, half, zero, one};
    const ALR20R63RDefect oriented_defect = al_r20_r63r_defect(
        oriented_inverse, identity, one, 2U);
    const std::vector<Binary128> oriented_q{
        static_cast<Binary128>(1.0), static_cast<Binary128>(0.5),
        static_cast<Binary128>(0.0), static_cast<Binary128>(1.0)};
    const std::vector<Binary128> reversed_q{
        static_cast<Binary128>(1.0), static_cast<Binary128>(0.0),
        static_cast<Binary128>(0.5), static_cast<Binary128>(1.0)};
    const std::vector<Binary128> oriented_solution{
        static_cast<Binary128>(1.0), static_cast<Binary128>(-0.5)};
    const ALR20R63UCertificate oriented_oracle = al_r20_r63u_certificate(
        {one, al_r20_r63p_control_dyadic(-1)}, matrix, oriented_defect,
        oriented_inverse, oriented_solution);
    const ALR20R63WCertificate oriented_finite = al_r20_r63w_certificate(
        profile, oriented_q, rhs, oriented_solution);
    const ALR20R63WCertificate reversed_finite = al_r20_r63w_certificate(
        profile, reversed_q, rhs, oriented_solution);
    const ALR20R63WContainment oriented_containment =
        al_r20_r63w_containment(oriented_finite, oriented_oracle);
    const ALR20R63WContainment reversed_containment =
        al_r20_r63w_containment(reversed_finite, oriented_oracle);
    result.orientation_oracle = oriented_oracle.exact;
    result.orientation_finite = oriented_finite.exact
        && reversed_finite.exact;
    result.orientation_containment = oriented_containment.exact;
    result.reversed_containment = reversed_containment.exact;
    result.orientation = oriented_containment.exact
        && !reversed_containment.exact;
    ALR20R63UCertificate mutated_oracle = identity_oracle;
    mutated_oracle.residual[0U] = one;
    const std::string finite_root = identity_certificate.root;
    const ALR20R63WContainment mutated_oracle_containment =
        al_r20_r63w_containment(identity_certificate, mutated_oracle);
    result.mutated_containment = mutated_oracle_containment.exact;
    result.independence = identity_certificate.root == finite_root
        && mutated_oracle_containment.root != identity_containment.root;
    result.mutation = interval.root != dropped.root;
    result.classifier =
        al_r20_r63w_classify(false, true, true, true, true)
            == "FINITE_IMAGE_APPARATUS_REJECTED"
        && al_r20_r63w_classify(true, false, true, true, true)
            == "FINITE_IMAGE_CONTAINMENT_REJECTED"
        && al_r20_r63w_classify(true, true, false, true, true)
            == "FINITE_IMAGE_CERTIFICATE_REJECTED"
        && al_r20_r63w_classify(true, true, true, false, true)
            == "RETAINED_WIDE_FINITE_IMAGE_REJECTED"
        && al_r20_r63w_classify(true, true, true, true, false)
            == "EXPORTED_FACTOR_FINITE_IMAGE_REJECTED"
        && al_r20_r63w_classify(true, true, true, true, true)
            == "TWO_LANE_FINITE_IMAGE_CANDIDATE";
    result.exact = result.identity && result.propagation
        && result.cancellation && result.dropped_matrix_radius
        && result.dropped_image_radius && result.invalid
        && result.orientation && result.independence && result.mutation
        && result.classifier;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + profile.root + ':' + identity_certificate.root + ':'
        + interval.root + ':' + dropped.root + ':' + cancellation.root + ':'
        + oriented_containment.root + ':' + reversed_containment.root + ':'
        + mutated_oracle_containment.root);
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v5_finite_residual_image_controls() {
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
    const ALR20R63WControls controls = al_r20_r63w_controls();
    const ALR20R63VControls parent_controls = al_r20_r63v_controls();
    const ALR20R63UControls parent_r63u_controls = al_r20_r63u_controls();
    const ALR20R63SControls parent_r63s_controls = al_r20_r63s_controls();
    const bool controls_exact = profile_exact && controls.exact
        && parent_controls.exact && parent_r63u_controls.exact
        && parent_r63s_controls.exact
        && parent_r63s_controls.root == AL_R20_R63T_PARENT_CONTROLS_SHA256;
    std::vector<ALR20ManifestCase> cases = al_r20_v5_manifest_cases();
    bool parent_exact = controls_exact && cases.size() == 6U
        && al_r20_v5_manifest_root(cases) == AL_R20_V5_MANIFEST_SHA256;
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    ALR20ProblemData problem;
    bool problem_materialized = false;
    if (parent_exact) {
        problem = al_r20_materialize_problem(
            cases[3U], &sparse_work, &static_work, &adjacency_work);
        problem_materialized = true;
        parent_exact = problem.exact && problem.source_positive == 0U
            && problem.target.size() == 105U
            && problem.problem_root == AL_R20_R60_PROBLEM_SHA256;
    }
    ALR20R60Capture solution_capture;
    ALR20VerifiedInverseCapture factor_capture;
    factor_capture.selected_matrix_root = AL_R20_R63_MATRIX_SHA256;
    ALR20R55SlopeTrace slope;
    ALR20R35CaseAudit oblique;
    std::size_t solver_executions = 0U;
    const bool hook_slot_clear = al_r20_verified_solution_refiner == nullptr
        && al_r20_r60_capture == nullptr
        && al_r20_verified_inverse_capture == nullptr
        && al_r20_verified_inverse_projector_context == nullptr
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr;
    if (parent_exact && hook_slot_clear) {
        al_r20_r60_capture = &solution_capture;
        al_r20_verified_solution_refiner = &al_r20_r60_capture_refiner;
        al_r20_verified_inverse_capture = &factor_capture;
        al_r20_r55_slope_trace = &slope;
        al_r20_slope_error_refiner = &al_r20_r55_slope_refiner;
        oblique = al_r20_r35_case(problem);
        al_r20_slope_error_refiner = nullptr;
        al_r20_r55_slope_trace = nullptr;
        al_r20_verified_inverse_capture = nullptr;
        al_r20_verified_solution_refiner = nullptr;
        al_r20_r60_capture = nullptr;
        solver_executions = 1U;
    }
    const bool hook_slot_released =
        al_r20_verified_solution_refiner == nullptr
        && al_r20_r60_capture == nullptr
        && al_r20_verified_inverse_capture == nullptr
        && al_r20_verified_inverse_projector_context == nullptr
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr
        && al_r20_direct_solution_capture == nullptr;
    const bool capture_exact = parent_exact && hook_slot_clear
        && hook_slot_released && solver_executions == 1U
        && al_r20_r35_case_observation_exact(oblique)
        && oblique.root == AL_R20_R60_CASE_SHA256
        && solution_capture.calls == 1U
        && factor_capture.calls == 125U
        && factor_capture.selected_calls == 1U
        && factor_capture.selected_column_solves == 102U
        && factor_capture.matrix_root == AL_R20_R63_MATRIX_SHA256
        && al_r20_q_scalar_root(factor_capture.rhs) == AL_R20_R63I_RHS_SHA256
        && factor_capture.source_rows.size() == 102U
        && factor_capture.projector.exact
        && slope.calls == 0U && slope.inverse_audits == 0U;
    const ALR20R63EProjectorGrams grams = capture_exact
        ? al_r20_r63e_projector_grams(problem.op,
            factor_capture.source_rows, problem.target.size(),
            factor_capture.projector, factor_capture.matrix)
        : ALR20R63EProjectorGrams{};
    const ALR20R63ERankProfile full_profile = grams.exact
        ? al_r20_r63e_rank_profile(grams.full, grams.rows)
        : ALR20R63ERankProfile{};
    const ALR20R63FTangentMatrix tangent = grams.exact
        ? al_r20_r63f_tangent_matrix(grams, factor_capture.projector,
            factor_capture.source_rows)
        : ALR20R63FTangentMatrix{};
    const Binary128 inverse_scale = static_cast<Binary128>(1.0)
        + factor_capture.projector.eta;
    const Binary128 sigma = static_cast<Binary128>(1.0) / inverse_scale;
    const ALR20R63QAudit common_audit = tangent.exact
        ? al_r20_r63q_audit(grams, tangent, factor_capture.projector,
            factor_capture.matrix, sigma)
        : ALR20R63QAudit{};
    const std::vector<std::size_t> permutation = full_profile.exact
        ? full_profile.binary128.permutation
        : std::vector<std::size_t>{};
    const bool permutation_exact = al_r20_r63g_permutation_exact(
        permutation, tangent.rows)
        && al_r20_r63g_index_root(permutation)
            == AL_R20_R63R_PERMUTATION_SHA256;
    std::vector<Binary128> permuted_original;
    std::vector<double> permuted_d;
    if (common_audit.exact && permutation_exact) {
        permuted_original.resize(tangent.columns * tangent.rows);
        permuted_d.resize(permuted_original.size());
        for (std::size_t scalar = 0U; scalar < tangent.columns; ++scalar)
            for (std::size_t column = 0U; column < tangent.rows; ++column) {
                const Binary128 value = tangent.values[
                    permutation[column] * tangent.columns + scalar];
                permuted_original[scalar * tangent.rows + column] = value;
                permuted_d[scalar * tangent.rows + column] =
                    static_cast<double>(value);
            }
    }
    const ALR20R63GQFactor original_factor = !permuted_original.empty()
        ? al_r20_r63g_q_factor(
            permuted_original, tangent.columns, tangent.rows)
        : ALR20R63GQFactor{};
    const ALR20R63RInverse verifier_inverse = original_factor.exact
        ? al_r20_r63r_inverse(
            original_factor.r, permutation, inverse_scale)
        : ALR20R63RInverse{};
    const ALR20R63RExactAudit verifier_audit = verifier_inverse.exact
        ? al_r20_r63r_exact_audit(verifier_inverse, common_audit.matrix)
        : ALR20R63RExactAudit{};
    const bool parent_r63r_exact = capture_exact && common_audit.exact
        && common_audit.root == AL_R20_R63R_AUDIT_SHA256
        && common_audit.matrix.root == AL_R20_R63R_MATRIX_SHA256
        && original_factor.root == AL_R20_R63R_FACTOR_SHA256
        && verifier_inverse.root == AL_R20_R63S_INVERSE_SHA256
        && verifier_audit.defect.root == AL_R20_R63S_DEFECT_SHA256
        && verifier_audit.root == AL_R20_R63S_INVERSE_AUDIT_SHA256;
    const ALR20R63GDFactor stored_d_factor = !permuted_d.empty()
        ? al_r20_r63g_d_factor(
            permuted_d, tangent.columns, tangent.rows)
        : ALR20R63GDFactor{};
    const std::vector<Binary128> stored_permuted =
        al_r20_r63g_q_from_double(stored_d_factor.input);
    const ALR20R63GQFactor wide_factor = !stored_permuted.empty()
        ? al_r20_r63g_q_factor(
            stored_permuted, tangent.columns, tangent.rows)
        : ALR20R63GQFactor{};
    std::vector<double> exported_d;
    if (wide_factor.exact)
        for (Binary128 value : wide_factor.r)
            exported_d.push_back(static_cast<double>(value));
    const std::vector<Binary128> exported_r =
        al_r20_r63g_q_from_double(exported_d);
    const bool preconditioners_exact = stored_d_factor.exact
        && wide_factor.exact
        && wide_factor.root == AL_R20_R63I_WIDE_FACTOR_SHA256
        && al_r20_double_root(exported_d) == AL_R20_R63I_EXPORT_R_SHA256;
    const ALR20R63IQSolve wide_start = preconditioners_exact
        ? al_r20_r63i_q_solve(wide_factor.r, permutation,
            factor_capture.rhs, inverse_scale)
        : ALR20R63IQSolve{};
    const ALR20R63IQSolve export_start = preconditioners_exact
        ? al_r20_r63i_q_solve(exported_r, permutation,
            factor_capture.rhs, inverse_scale)
        : ALR20R63IQSolve{};
    const bool starts_exact = wide_start.exact && export_start.exact
        && wide_start.root == AL_R20_R63J_WIDE_SOLVE_SHA256
        && export_start.root == AL_R20_R63J_EXPORT_WIDE_SOLVE_SHA256;
    bool rhs_conversions_exact = true;
    std::vector<ALR20R37Dyadic> exact_rhs(factor_capture.rhs.size());
    for (std::size_t index = 0U; index < factor_capture.rhs.size(); ++index) {
        const ALR20R37Conversion conversion =
            al_r20_r37_from_binary128(factor_capture.rhs[index]);
        rhs_conversions_exact = rhs_conversions_exact && conversion.exact;
        exact_rhs[index] = conversion.value;
    }
    const std::string exact_rhs_root = al_r20_r63q_vector_root(exact_rhs);
    const ALR20R63SLane parent_wide = parent_r63r_exact && starts_exact
        && rhs_conversions_exact
        ? al_r20_r63s_lane(wide_factor.r, permutation, inverse_scale,
            tangent.values, tangent.columns, sigma, factor_capture.rhs,
            exact_rhs, common_audit.matrix, verifier_audit.defect,
            wide_start.solution)
        : ALR20R63SLane{};
    const ALR20R63SLane parent_exported = parent_r63r_exact && starts_exact
        && rhs_conversions_exact
        ? al_r20_r63s_lane(exported_r, permutation, inverse_scale,
            tangent.values, tangent.columns, sigma, factor_capture.rhs,
            exact_rhs, common_audit.matrix, verifier_audit.defect,
            export_start.solution)
        : ALR20R63SLane{};
    const std::string parent_r63s_route = al_r20_r63s_classify(true, true,
        parent_wide.passed, parent_exported.passed);
    std::ostringstream parent_r63s_material;
    parent_r63s_material
        << "nextengine.nonlocal.nsr3b4e2d7r20r63s-common-operator-pcg.v1|"
        << AL_R20_R63S_PARENT_SEMANTIC_SHA256 << ':' << parent_r63s_route
        << ':' << parent_r63s_controls.root << ':' << common_audit.root << ':'
        << verifier_audit.root << ':' << exact_rhs_root << ':'
        << parent_wide.root << ':' << parent_exported.root;
    const std::string parent_r63s_semantic =
        sha256_hex(parent_r63s_material.str());
    const bool parent_r63s_exact = parent_r63r_exact
        && preconditioners_exact && starts_exact && rhs_conversions_exact
        && exact_rhs_root == AL_R20_R63T_EXACT_RHS_SHA256
        && parent_wide.exact && parent_exported.exact
        && al_r20_r63v_parent_work_exact(parent_wide)
        && al_r20_r63v_parent_work_exact(parent_exported)
        && parent_wide.root == AL_R20_R63U_WIDE_LANE_SHA256
        && parent_exported.root == AL_R20_R63U_EXPORTED_LANE_SHA256
        && parent_r63s_semantic == AL_R20_R63U_PARENT_SEMANTIC_SHA256;
    bool inverse_conversions_exact = true;
    std::vector<ALR20R37Dyadic> exact_inverse(verifier_inverse.value.size());
    for (std::size_t index = 0U;
         index < verifier_inverse.value.size(); ++index) {
        const ALR20R37Conversion conversion =
            al_r20_r37_from_binary128(verifier_inverse.value[index]);
        inverse_conversions_exact = inverse_conversions_exact
            && conversion.exact;
        exact_inverse[index] = conversion.value;
    }
    const std::string inverse_root = al_r20_r63q_vector_root(exact_inverse);
    const ALR20R63UCertificate parent_wide_certificate = parent_r63s_exact
        && inverse_conversions_exact
        ? al_r20_r63u_certificate(exact_rhs, common_audit.matrix,
            verifier_audit.defect, exact_inverse, parent_wide.solution)
        : ALR20R63UCertificate{};
    const ALR20R63UCertificate parent_exported_certificate = parent_r63s_exact
        && inverse_conversions_exact
        ? al_r20_r63u_certificate(exact_rhs, common_audit.matrix,
            verifier_audit.defect, exact_inverse, parent_exported.solution)
        : ALR20R63UCertificate{};
    const std::string parent_r63u_route = al_r20_r63u_classify(
        true, true, parent_wide_certificate.passed,
        parent_exported_certificate.passed);
    std::ostringstream parent_r63u_material;
    parent_r63u_material
        << "nextengine.nonlocal.nsr3b4e2d7r20r63u-exact-residual-image.v1|"
        << AL_R20_R63U_CAUSAL_R63T_SHA256 << ':' << parent_r63s_semantic
        << ':' << parent_r63u_route << ':' << parent_r63u_controls.root << ':'
        << inverse_root << ':' << parent_wide_certificate.root << ':'
        << parent_exported_certificate.root;
    const std::string parent_r63u_semantic =
        sha256_hex(parent_r63u_material.str());
    const bool parent_r63u_exact = parent_r63s_exact
        && inverse_conversions_exact
        && inverse_root == verifier_audit.conversion_root
        && parent_wide_certificate.root
            == AL_R20_R63V_WIDE_CERTIFICATE_SHA256
        && parent_exported_certificate.root
            == AL_R20_R63V_EXPORTED_CERTIFICATE_SHA256
        && parent_r63u_semantic == AL_R20_R63V_PARENT_SEMANTIC_SHA256;
    const ALR20R63VLane parent_v_wide = parent_r63u_exact
        ? al_r20_r63v_lane(parent_wide, wide_start.solution, exact_rhs,
            common_audit.matrix, verifier_audit.defect, exact_inverse)
        : ALR20R63VLane{};
    const ALR20R63VLane parent_v_exported = parent_r63u_exact
        ? al_r20_r63v_lane(parent_exported, export_start.solution, exact_rhs,
            common_audit.matrix, verifier_audit.defect, exact_inverse)
        : ALR20R63VLane{};
    const std::string parent_r63v_route = al_r20_r63v_classify(
        true, true, parent_v_wide.passed, parent_v_exported.passed);
    std::ostringstream parent_r63v_material;
    parent_r63v_material
        << "nextengine.nonlocal.nsr3b4e2d7r20r63v-earliest-residual-image.v1|"
        << parent_r63u_semantic << ':' << parent_r63v_route << ':'
        << parent_controls.root << ':' << parent_v_wide.root << ':'
        << parent_v_exported.root;
    const std::string parent_r63v_semantic =
        sha256_hex(parent_r63v_material.str());
    const bool parent_r63v_exact = parent_r63u_exact
        && parent_v_wide.exact && parent_v_exported.exact
        && parent_v_wide.root == AL_R20_R63W_WIDE_LANE_SHA256
        && parent_v_exported.root == AL_R20_R63W_EXPORTED_LANE_SHA256
        && parent_r63v_semantic == AL_R20_R63W_PARENT_SEMANTIC_SHA256;

    const ALR20R63WProfile finite_profile = parent_r63v_exact
        ? al_r20_r63w_profile(common_audit.matrix, verifier_audit.defect)
        : ALR20R63WProfile{};
    std::vector<std::vector<Binary128>> wide_solutions;
    std::vector<std::vector<Binary128>> exported_solutions;
    if (parent_r63v_exact && parent_wide.steps.size() >= 2U
        && parent_exported.steps.size() >= 2U) {
        wide_solutions = {wide_start.solution,
            parent_wide.steps[0U].solution, parent_wide.steps[1U].solution};
        exported_solutions = {export_start.solution,
            parent_exported.steps[0U].solution,
            parent_exported.steps[1U].solution};
    }
    std::vector<ALR20R63UCertificate> wide_oracles;
    std::vector<ALR20R63UCertificate> exported_oracles;
    if (parent_v_wide.state_certificates.size() >= 3U
        && parent_v_exported.state_certificates.size() >= 3U) {
        wide_oracles.assign(parent_v_wide.state_certificates.begin(),
            parent_v_wide.state_certificates.begin() + 3);
        exported_oracles.assign(
            parent_v_exported.state_certificates.begin(),
            parent_v_exported.state_certificates.begin() + 3);
    }
    const ALR20R63WLane wide = finite_profile.exact
        ? al_r20_r63w_lane(finite_profile, verifier_inverse.value,
            factor_capture.rhs, wide_solutions, wide_oracles)
        : ALR20R63WLane{};
    const ALR20R63WLane exported = finite_profile.exact
        ? al_r20_r63w_lane(finite_profile, verifier_inverse.value,
            factor_capture.rhs, exported_solutions, exported_oracles)
        : ALR20R63WLane{};
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool containment_exact = wide.contained && exported.contained
        && wide.residual_containments == 306U
        && wide.image_containments == 306U
        && exported.residual_containments == 306U
        && exported.image_containments == 306U;
    const bool state2_signs_exact = wide.certificates.size() == 3U
        && exported.certificates.size() == 3U
        && wide.certificates[2U].sign_root == AL_R20_R63W_SIGN_SHA256
        && exported.certificates[2U].sign_root == AL_R20_R63W_SIGN_SHA256;
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_r63v_exact && finite_profile.exact && lifecycle_exact;
    const bool certificate_exact = wide.exact && exported.exact;
    const std::string route = al_r20_r63w_classify(apparatus_exact,
        containment_exact, certificate_exact, wide.passed, exported.passed);
    const bool exact = apparatus_exact && containment_exact
        && certificate_exact;
    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63w-finite-residual-image.v1|"
        << parent_r63v_semantic << ':' << route << ':' << controls.root << ':'
        << finite_profile.root << ':' << wide.root << ':' << exported.root;
    const std::string result_sha = sha256_hex(semantic.str());
    auto lane_json = [](const char* name, const ALR20R63WLane& lane) {
        std::ostringstream stream;
        stream << "\"" << name << "\":{\"exact\":"
            << (lane.exact ? "true" : "false")
            << ",\"contained\":" << (lane.contained ? "true" : "false")
            << ",\"passed\":" << (lane.passed ? "true" : "false")
            << ",\"states\":[";
        for (std::size_t state = 0U;
             state < lane.certificates.size(); ++state) {
            if (state != 0U) stream << ',';
            const ALR20R63WCertificate& certificate =
                lane.certificates[state];
            const ALR20R63WContainment& containment =
                lane.containments[state];
            stream << "{\"state\":" << state << ",\"passed\":"
                << (certificate.passed ? "true" : "false")
                << ",\"error_upper\":\""
                << al_binary128_text(certificate.error_upper)
                << "\",\"positive\":" << certificate.positive
                << ",\"negative\":" << certificate.negative
                << ",\"unresolved\":" << certificate.unresolved
                << ",\"residual_contained\":"
                << (containment.residual_contained ? "true" : "false")
                << ",\"image_contained\":"
                << (containment.image_contained ? "true" : "false")
                << ",\"root\":\"" << certificate.root << "\"}";
        }
        stream << "],\"residual_containments\":"
            << lane.residual_containments
            << ",\"image_containments\":" << lane.image_containments
            << ",\"root\":\"" << lane.root << "\"}";
        return stream.str();
    };
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63w_finite_residual_image.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63v_sha256\":\"" << parent_r63v_semantic << "\""
        << ",\"controls_root\":\"" << controls.root << "\""
        << ",\"controls\":{\"exact\":"
        << (controls.exact ? "true" : "false")
        << ",\"identity\":" << (controls.identity ? "true" : "false")
        << ",\"propagation\":"
        << (controls.propagation ? "true" : "false")
        << ",\"cancellation\":"
        << (controls.cancellation ? "true" : "false")
        << ",\"dropped_matrix_radius\":"
        << (controls.dropped_matrix_radius ? "true" : "false")
        << ",\"dropped_image_radius\":"
        << (controls.dropped_image_radius ? "true" : "false")
        << ",\"invalid\":" << (controls.invalid ? "true" : "false")
        << ",\"orientation\":"
        << (controls.orientation ? "true" : "false")
        << ",\"independence\":"
        << (controls.independence ? "true" : "false")
        << ",\"mutation\":" << (controls.mutation ? "true" : "false")
        << ",\"classifier\":"
        << (controls.classifier ? "true" : "false")
        << ",\"identity_profile\":"
        << (controls.identity_profile ? "true" : "false")
        << ",\"identity_finite\":"
        << (controls.identity_finite ? "true" : "false")
        << ",\"identity_oracle\":"
        << (controls.identity_oracle ? "true" : "false")
        << ",\"identity_containment\":"
        << (controls.identity_containment ? "true" : "false")
        << ",\"propagation_finite\":"
        << (controls.propagation_finite ? "true" : "false")
        << ",\"propagation_containment\":"
        << (controls.propagation_containment ? "true" : "false")
        << ",\"orientation_oracle\":"
        << (controls.orientation_oracle ? "true" : "false")
        << ",\"orientation_finite\":"
        << (controls.orientation_finite ? "true" : "false")
        << ",\"orientation_containment\":"
        << (controls.orientation_containment ? "true" : "false")
        << ",\"reversed_containment\":"
        << (controls.reversed_containment ? "true" : "false")
        << ",\"mutated_containment\":"
        << (controls.mutated_containment ? "true" : "false") << '}'
        << ",\"parent_r63v_exact\":"
        << (parent_r63v_exact ? "true" : "false")
        << ",\"finite_profile\":{\"exact\":"
        << (finite_profile.exact ? "true" : "false")
        << ",\"rho_upper\":\""
        << al_binary128_text(finite_profile.rho_upper)
        << "\",\"maximum_matrix_radius\":\""
        << al_binary128_text(finite_profile.maximum_matrix_radius)
        << "\",\"center_root\":\"" << finite_profile.center_root << "\""
        << ",\"radius_root\":\"" << finite_profile.radius_root << "\""
        << ",\"root\":\"" << finite_profile.root << "\"}"
        << ",\"lanes\":{" << lane_json("wide", wide) << ','
        << lane_json("exported", exported) << '}'
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"containment_exact\":"
        << (containment_exact ? "true" : "false")
        << ",\"certificate_exact\":"
        << (certificate_exact ? "true" : "false")
        << ",\"state2_signs_exact\":"
        << (state2_signs_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"finite_residual_dots\":"
        << (wide.certificates.size() + exported.certificates.size()) * 102U
        << ",\"finite_image_dots\":"
        << (wide.certificates.size() + exported.certificates.size()) * 102U
        << ",\"exact_oracle_classification_uses\":0"
        << ",\"candidate_updates\":0,\"sparse_constructions\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"gpu_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
