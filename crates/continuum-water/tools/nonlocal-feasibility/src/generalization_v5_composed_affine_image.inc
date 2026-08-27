namespace {

constexpr const char* AL_R20_R63X_PARENT_SEMANTIC_SHA256 =
    "35a74726a47d377ec8f97aac77dc52b22fb2463de1b33d77e6ea1fa63b48d2c3";
constexpr const char* AL_R20_R63X_PARENT_STDOUT_SHA256 =
    "1e85677e07e22d91fc8b3ca7f9a4b7f9bdfbafcb7a404b4797854c1fc531c2fe";

struct ALR20R63XProjection {
    bool exact = false;
    Binary128 center = static_cast<Binary128>(0.0);
    Binary128 radius = static_cast<Binary128>(0.0);
};

ALR20R63XProjection al_r20_r63x_projection(
    const ALR20R37Dyadic& numerator,
    const ALR20R37Dyadic& denominator) {
    ALR20R63XProjection result;
    if (al_r20_r63p_sign(denominator) <= 0) return result;
    result.center = al_r20_r63p_fraction_projection(numerator, denominator);
    const ALR20R37Conversion center_exact =
        al_r20_r37_from_binary128(result.center);
    if (!center_exact.exact) return result;
    const ALR20R37Dyadic error = al_r20_r37_absolute(
        al_r20_r63p_subtract(
            al_r20_r37_multiply(center_exact.value, denominator),
            numerator));
    const Binary128 denominator_down =
        al_r20_r63w_positive_dyadic_down(denominator);
    result.radius = error.numerator == 0
        ? static_cast<Binary128>(0.0)
        : al_r20_r38_up_divide(
            al_r20_r37_binary128_up(error), denominator_down);
    const ALR20R37Conversion radius_exact =
        al_r20_r37_from_binary128(result.radius);
    result.exact = denominator_down > static_cast<Binary128>(0.0)
        && radius_exact.exact
        && result.radius >= static_cast<Binary128>(0.0)
        && al_r20_r37_compare_positive(
            al_r20_r37_multiply(radius_exact.value, denominator),
            error) >= 0
        && al_r20_r38_normal_or_zero(result.center)
        && al_r20_r38_normal_or_zero(result.radius);
    return result;
}

struct ALR20R63XProfile {
    bool exact = false;
    bool contained = false;
    bool contractive = false;
    std::size_t dimension = 0U;
    std::size_t rhs_products = 0U;
    std::size_t matrix_products = 0U;
    std::size_t vector_entries = 0U;
    std::size_t matrix_entries = 0U;
    std::size_t vector_containments = 0U;
    std::size_t matrix_containments = 0U;
    std::vector<Binary128> vector_center;
    std::vector<Binary128> vector_radius;
    std::vector<Binary128> matrix_center;
    std::vector<Binary128> matrix_radius;
    Binary128 rho_upper = static_cast<Binary128>(0.0);
    Binary128 maximum_vector_radius = static_cast<Binary128>(0.0);
    Binary128 maximum_matrix_radius = static_cast<Binary128>(0.0);
    std::string exact_vector_root;
    std::string exact_matrix_root;
    std::string vector_center_root;
    std::string vector_radius_root;
    std::string matrix_center_root;
    std::string matrix_radius_root;
    std::string root;
};

ALR20R63XProfile al_r20_r63x_profile(
    const std::vector<ALR20R37Dyadic>& inverse,
    const std::vector<ALR20R37Dyadic>& rhs,
    const ALR20R63QExactMatrix& matrix,
    const ALR20R63RDefect& defect) {
    ALR20R63XProfile result;
    result.dimension = rhs.size();
    const std::size_t dimension = result.dimension;
    if (dimension == 0U || inverse.size() != dimension * dimension
        || !matrix.exact || matrix.rows != dimension
        || matrix.oracle_numerator.size() != dimension * dimension
        || !defect.exact || !defect.left_contractive
        || defect.inverse_root != al_r20_r63q_vector_root(inverse)
        || !al_r20_r63p_equal(matrix.d, defect.d)) return result;
    std::vector<ALR20R37Dyadic> exact_vector(dimension);
    std::vector<ALR20R37Dyadic> exact_matrix(dimension * dimension);
    for (std::size_t row = 0U; row < dimension; ++row)
        for (std::size_t inner = 0U; inner < dimension; ++inner) {
            exact_vector[row] = al_r20_r37_add(exact_vector[row],
                al_r20_r37_multiply(
                    inverse[row * dimension + inner], rhs[inner]));
            ++result.rhs_products;
        }
    for (std::size_t row = 0U; row < dimension; ++row)
        for (std::size_t column = 0U; column < dimension; ++column)
            for (std::size_t inner = 0U; inner < dimension; ++inner) {
                exact_matrix[row * dimension + column] = al_r20_r37_add(
                    exact_matrix[row * dimension + column],
                    al_r20_r37_multiply(
                        inverse[row * dimension + inner],
                        matrix.oracle_numerator[
                            inner * dimension + column]));
                ++result.matrix_products;
            }
    result.vector_center.resize(dimension);
    result.vector_radius.resize(dimension);
    result.matrix_center.resize(dimension * dimension);
    result.matrix_radius.resize(dimension * dimension);
    bool projections_exact = true;
    const ALR20R37Dyadic one = al_r20_r63p_control_dyadic(1);
    for (std::size_t index = 0U; index < dimension; ++index) {
        const ALR20R63XProjection projection = al_r20_r63x_projection(
            exact_vector[index], one);
        projections_exact = projections_exact && projection.exact;
        result.vector_center[index] = projection.center;
        result.vector_radius[index] = projection.radius;
        result.maximum_vector_radius = fmaxq(
            result.maximum_vector_radius, projection.radius);
        ++result.vector_entries;
        result.vector_containments += projection.exact ? 1U : 0U;
    }
    for (std::size_t index = 0U;
         index < dimension * dimension; ++index) {
        const ALR20R63XProjection projection = al_r20_r63x_projection(
            exact_matrix[index], matrix.d);
        projections_exact = projections_exact && projection.exact;
        result.matrix_center[index] = projection.center;
        result.matrix_radius[index] = projection.radius;
        result.maximum_matrix_radius = fmaxq(
            result.maximum_matrix_radius, projection.radius);
        ++result.matrix_entries;
        result.matrix_containments += projection.exact ? 1U : 0U;
    }
    const Binary128 denominator_down =
        al_r20_r63w_positive_dyadic_down(matrix.d);
    result.rho_upper = al_r20_r38_up_divide(
        al_r20_r37_binary128_up(defect.left_infinity_numerator),
        denominator_down);
    const ALR20R37Conversion rho_exact =
        al_r20_r37_from_binary128(result.rho_upper);
    const bool rho_contained = rho_exact.exact
        && al_r20_r37_compare_positive(
            al_r20_r37_multiply(rho_exact.value, matrix.d),
            defect.left_infinity_numerator) >= 0;
    result.contained = projections_exact && rho_contained
        && result.vector_containments == dimension
        && result.matrix_containments == dimension * dimension;
    result.contractive = result.rho_upper >= static_cast<Binary128>(0.0)
        && result.rho_upper < static_cast<Binary128>(1.0);
    result.exact_vector_root = al_r20_r63q_vector_root(exact_vector);
    result.exact_matrix_root = al_r20_r63q_vector_root(exact_matrix);
    result.vector_center_root = al_r20_q_scalar_root(result.vector_center);
    result.vector_radius_root = al_r20_q_scalar_root(result.vector_radius);
    result.matrix_center_root = al_r20_q_scalar_root(result.matrix_center);
    result.matrix_radius_root = al_r20_q_scalar_root(result.matrix_radius);
    result.exact = result.contained && result.contractive
        && result.rhs_products == dimension * dimension
        && result.matrix_products == dimension * dimension * dimension
        && result.vector_entries == dimension
        && result.matrix_entries == dimension * dimension
        && al_r20_r38_normal_or_zero(result.maximum_vector_radius)
        && al_r20_r38_normal_or_zero(result.maximum_matrix_radius)
        && al_r20_r38_normal_or_zero(result.rho_upper);
    std::ostringstream material;
    material << result.exact << ':' << result.contained << ':'
        << result.contractive << ':' << result.dimension << ':'
        << result.rhs_products << ':' << result.matrix_products << ':'
        << result.vector_entries << ':' << result.matrix_entries << ':'
        << result.vector_containments << ':' << result.matrix_containments
        << ':' << al_binary128_hex(result.rho_upper) << ':'
        << al_binary128_hex(result.maximum_vector_radius) << ':'
        << al_binary128_hex(result.maximum_matrix_radius) << ':'
        << result.exact_vector_root << ':' << result.exact_matrix_root << ':'
        << result.vector_center_root << ':' << result.vector_radius_root << ':'
        << result.matrix_center_root << ':' << result.matrix_radius_root;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63XCertificate {
    bool exact = false;
    bool no_underflow = false;
    bool passed = false;
    std::size_t dimension = 0U;
    std::size_t dots = 0U;
    std::size_t radius_terms = 0U;
    std::size_t sign_comparisons = 0U;
    std::size_t positive = 0U;
    std::size_t negative = 0U;
    std::size_t unresolved = 0U;
    Binary128 image_infinity_upper = static_cast<Binary128>(0.0);
    Binary128 denominator_lower = static_cast<Binary128>(0.0);
    Binary128 error_upper = static_cast<Binary128>(0.0);
    Binary128 minimum_separation = HUGE_VALQ;
    std::vector<Binary128> image_center;
    std::vector<Binary128> image_radius;
    std::vector<int> signs;
    std::string solution_root;
    std::string dot_root;
    std::string sign_root;
    std::string root;
};

ALR20R63XCertificate al_r20_r63x_certificate(
    const ALR20R63XProfile& profile,
    const std::vector<Binary128>& solution) {
    ALR20R63XCertificate result;
    result.dimension = solution.size();
    const std::size_t dimension = result.dimension;
    if (!profile.exact || profile.dimension != dimension || dimension == 0U
        || profile.vector_center.size() != dimension
        || profile.vector_radius.size() != dimension
        || profile.matrix_center.size() != dimension * dimension
        || profile.matrix_radius.size() != dimension * dimension) return result;
    bool exact = profile.rho_upper >= static_cast<Binary128>(0.0)
        && profile.rho_upper < static_cast<Binary128>(1.0);
    bool no_underflow = true;
    for (std::size_t index = 0U; index < dimension; ++index)
        exact = exact
            && profile.vector_radius[index] >= static_cast<Binary128>(0.0)
            && al_r20_r38_normal_or_zero(profile.vector_center[index])
            && al_r20_r38_normal_or_zero(profile.vector_radius[index])
            && al_r20_r38_normal_or_zero(solution[index]);
    for (std::size_t index = 0U; index < dimension * dimension; ++index)
        exact = exact
            && profile.matrix_radius[index] >= static_cast<Binary128>(0.0)
            && al_r20_r38_normal_or_zero(profile.matrix_center[index])
            && al_r20_r38_normal_or_zero(profile.matrix_radius[index]);
    result.image_center.resize(dimension);
    result.image_radius.resize(dimension);
    std::ostringstream dot_material;
    for (std::size_t row = 0U; row < dimension; ++row) {
        std::vector<Binary128> left(dimension + 1U);
        std::vector<Binary128> right(dimension + 1U);
        left[0U] = profile.vector_center[row];
        right[0U] = static_cast<Binary128>(1.0);
        for (std::size_t column = 0U; column < dimension; ++column) {
            left[column + 1U] =
                profile.matrix_center[row * dimension + column];
            right[column + 1U] = -solution[column];
        }
        const ALR20R38Dot2 dot = al_r20_r38_dot2(left, right);
        ++result.dots;
        exact = exact && dot.exact;
        no_underflow = no_underflow && dot.no_underflow;
        Binary128 radius = al_r20_r38_up_add(
            dot.bound, profile.vector_radius[row]);
        for (std::size_t column = 0U; column < dimension; ++column) {
            radius = al_r20_r38_up_add(radius,
                al_r20_r38_up_multiply(
                    profile.matrix_radius[row * dimension + column],
                    fabsq(solution[column])));
            ++result.radius_terms;
        }
        result.image_center[row] = dot.value;
        result.image_radius[row] = radius;
        const Binary128 magnitude = al_r20_r38_up_add(
            fabsq(dot.value), radius);
        result.image_infinity_upper = fmaxq(
            result.image_infinity_upper, magnitude);
        exact = exact && radius >= static_cast<Binary128>(0.0)
            && al_r20_r38_normal_or_zero(dot.value)
            && al_r20_r38_normal_or_zero(radius)
            && al_r20_r38_normal_or_zero(magnitude);
        dot_material << dot.root << ':' << al_binary128_hex(radius) << ';';
    }
    result.denominator_lower = nextafterq(
        static_cast<Binary128>(1.0) - profile.rho_upper, -HUGE_VALQ);
    exact = exact
        && result.denominator_lower > static_cast<Binary128>(0.0);
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
    result.no_underflow = no_underflow;
    result.solution_root = al_r20_q_scalar_root(solution);
    result.dot_root = sha256_hex(dot_material.str());
    std::ostringstream sign_material;
    for (int sign : result.signs) sign_material << sign << ',';
    result.sign_root = sha256_hex(sign_material.str());
    result.exact = exact && no_underflow && result.dots == dimension
        && result.radius_terms == dimension * dimension
        && result.sign_comparisons == dimension
        && result.positive + result.negative + result.unresolved == dimension
        && al_r20_r38_normal_or_zero(result.image_infinity_upper)
        && al_r20_r38_normal_or_zero(result.denominator_lower)
        && al_r20_r38_normal_or_zero(result.error_upper);
    std::ostringstream material;
    material << result.exact << ':' << result.no_underflow << ':'
        << result.passed << ':' << result.dimension << ':' << result.dots
        << ':' << result.radius_terms << ':' << result.sign_comparisons << ':'
        << result.positive << ':' << result.negative << ':'
        << result.unresolved << ':'
        << al_binary128_hex(result.image_infinity_upper) << ':'
        << al_binary128_hex(result.denominator_lower) << ':'
        << al_binary128_hex(result.error_upper) << ':'
        << al_binary128_hex(result.minimum_separation) << ':'
        << result.solution_root << ':' << result.dot_root << ':'
        << result.sign_root << ':'
        << al_r20_q_scalar_root(result.image_center) << ':'
        << al_r20_q_scalar_root(result.image_radius);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63XContainment {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t image_entries = 0U;
    std::string root;
};

ALR20R63XContainment al_r20_r63x_containment(
    const ALR20R63XCertificate& finite,
    const ALR20R63UCertificate& oracle) {
    ALR20R63XContainment result;
    result.dimension = finite.dimension;
    if (!finite.exact || !oracle.exact || oracle.dimension != result.dimension
        || finite.image_center.size() != result.dimension
        || finite.image_radius.size() != result.dimension
        || oracle.image.size() != result.dimension) return result;
    std::ostringstream material;
    for (std::size_t index = 0U; index < result.dimension; ++index) {
        const bool contained = al_r20_r63w_contains_fraction(
            finite.image_center[index], finite.image_radius[index],
            oracle.image[index], oracle.d);
        result.image_entries += contained ? 1U : 0U;
        material << index << ':' << contained << ';';
    }
    result.exact = result.image_entries == result.dimension;
    material << '|' << result.exact << ':' << result.dimension << ':'
        << result.image_entries;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63XLane {
    bool exact = false;
    bool contained = false;
    bool passed = false;
    bool reject_reject_pass = false;
    std::size_t dimension = 0U;
    std::size_t image_containments = 0U;
    std::vector<ALR20R63XCertificate> certificates;
    std::vector<ALR20R63XContainment> containments;
    std::string root;
};

ALR20R63XLane al_r20_r63x_lane(
    const ALR20R63XProfile& profile,
    const std::vector<std::vector<Binary128>>& solutions,
    const std::vector<ALR20R63UCertificate>& oracles) {
    ALR20R63XLane result;
    result.dimension = profile.dimension;
    if (solutions.size() != 3U || oracles.size() != 3U) return result;
    bool exact = true;
    bool contained = true;
    for (std::size_t state = 0U; state < 3U; ++state) {
        ALR20R63XCertificate certificate = al_r20_r63x_certificate(
            profile, solutions[state]);
        ALR20R63XContainment containment = al_r20_r63x_containment(
            certificate, oracles[state]);
        exact = exact && certificate.exact;
        contained = contained && containment.exact;
        result.image_containments += containment.image_entries;
        result.certificates.push_back(std::move(certificate));
        result.containments.push_back(std::move(containment));
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
        && result.image_containments == 3U * result.dimension;
    result.exact = exact && result.contained;
    result.passed = result.exact && result.reject_reject_pass;
    std::ostringstream material;
    material << result.exact << ':' << result.contained << ':'
        << result.passed << ':' << result.reject_reject_pass << ':'
        << result.dimension << ':' << result.image_containments << '|';
    for (std::size_t state = 0U; state < 3U; ++state)
        material << result.certificates[state].root << ':'
            << result.containments[state].root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r20_r63x_classify(bool apparatus, bool containment,
    bool certificate, bool wide, bool exported) {
    if (!apparatus) return "AFFINE_IMAGE_APPARATUS_REJECTED";
    if (!containment) return "AFFINE_IMAGE_CONTAINMENT_REJECTED";
    if (!certificate) return "AFFINE_IMAGE_CERTIFICATE_REJECTED";
    if (!wide) return "RETAINED_WIDE_AFFINE_IMAGE_REJECTED";
    if (!exported) return "EXPORTED_FACTOR_AFFINE_IMAGE_REJECTED";
    return "TWO_LANE_AFFINE_IMAGE_CANDIDATE";
}

struct ALR20R63XControls {
    bool exact = false;
    bool identity = false;
    bool cancellation = false;
    bool vector_radius = false;
    bool matrix_radius = false;
    bool dropped_vector_radius = false;
    bool dropped_matrix_radius = false;
    bool orientation = false;
    bool invalid = false;
    bool independence = false;
    bool mutation = false;
    bool classifier = false;
    std::string root;
};

struct ALR20R63XFixture {
    bool exact = false;
    std::vector<ALR20R37Dyadic> inverse;
    std::vector<ALR20R37Dyadic> rhs;
    ALR20R63QExactMatrix matrix;
    ALR20R63RDefect defect;
    std::vector<std::vector<Binary128>> wide_solutions;
    std::vector<std::vector<Binary128>> exported_solutions;
    std::vector<ALR20R63UCertificate> wide_oracles;
    std::vector<ALR20R63UCertificate> exported_oracles;
    std::string semantic;
    std::string profile_root;
    std::string wide_root;
    std::string exported_root;
};

ALR20R63XFixture al_r20_r63x_last_fixture;

ALR20R63XControls al_r20_r63x_controls() {
    ALR20R63XControls result;
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
    const std::vector<ALR20R37Dyadic> exact_rhs{
        one, al_r20_r63p_control_dyadic(-1)};
    const ALR20R63XProfile profile = al_r20_r63x_profile(
        identity, exact_rhs, matrix, identity_defect);
    const std::vector<Binary128> solution{
        static_cast<Binary128>(1.0), static_cast<Binary128>(-1.0)};
    const ALR20R63XCertificate identity_certificate =
        al_r20_r63x_certificate(profile, solution);
    const ALR20R63UCertificate identity_oracle = al_r20_r63u_certificate(
        exact_rhs, matrix, identity_defect, identity, solution);
    const ALR20R63XContainment identity_containment =
        al_r20_r63x_containment(identity_certificate, identity_oracle);
    result.identity = profile.exact && identity_certificate.exact
        && identity_certificate.passed && identity_containment.exact;
    const ALR20R63XCertificate cancellation = al_r20_r63x_certificate(
        profile,
        {nextafterq(static_cast<Binary128>(1.0), HUGE_VALQ),
            static_cast<Binary128>(-1.0)});
    result.cancellation = cancellation.exact && cancellation.passed
        && cancellation.dots == 2U
        && cancellation.image_center.size() == 2U
        && cancellation.image_center[0U] < static_cast<Binary128>(0.0)
        && cancellation.image_center[1U] == static_cast<Binary128>(0.0);

    ALR20R63XProfile vector_profile;
    vector_profile.exact = true;
    vector_profile.dimension = 1U;
    vector_profile.vector_center = {static_cast<Binary128>(0.5)};
    vector_profile.vector_radius = {static_cast<Binary128>(0.5)};
    vector_profile.matrix_center = {static_cast<Binary128>(1.0)};
    vector_profile.matrix_radius = {static_cast<Binary128>(0.0)};
    vector_profile.rho_upper = static_cast<Binary128>(0.0);
    const ALR20R63XCertificate vector_certificate =
        al_r20_r63x_certificate(
            vector_profile, {static_cast<Binary128>(1.0)});
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
    const ALR20R63XContainment vector_containment =
        al_r20_r63x_containment(vector_certificate, scalar_oracle);
    result.vector_radius = vector_certificate.exact
        && vector_containment.exact;
    ALR20R63XProfile dropped_vector = vector_profile;
    dropped_vector.vector_radius[0U] = static_cast<Binary128>(0.0);
    const ALR20R63XCertificate dropped_vector_certificate =
        al_r20_r63x_certificate(
            dropped_vector, {static_cast<Binary128>(1.0)});
    const ALR20R63XContainment dropped_vector_containment =
        al_r20_r63x_containment(dropped_vector_certificate, scalar_oracle);
    result.dropped_vector_radius = dropped_vector_certificate.exact
        && !dropped_vector_containment.exact;

    ALR20R63XProfile matrix_profile = vector_profile;
    matrix_profile.vector_center[0U] = static_cast<Binary128>(1.0);
    matrix_profile.vector_radius[0U] = static_cast<Binary128>(0.0);
    matrix_profile.matrix_center[0U] = static_cast<Binary128>(0.5);
    matrix_profile.matrix_radius[0U] = static_cast<Binary128>(0.5);
    const ALR20R63XCertificate matrix_certificate =
        al_r20_r63x_certificate(
            matrix_profile, {static_cast<Binary128>(1.0)});
    const ALR20R63XContainment matrix_containment =
        al_r20_r63x_containment(matrix_certificate, scalar_oracle);
    result.matrix_radius = matrix_certificate.exact
        && matrix_containment.exact;
    ALR20R63XProfile dropped_matrix = matrix_profile;
    dropped_matrix.matrix_radius[0U] = static_cast<Binary128>(0.0);
    const ALR20R63XCertificate dropped_matrix_certificate =
        al_r20_r63x_certificate(
            dropped_matrix, {static_cast<Binary128>(1.0)});
    const ALR20R63XContainment dropped_matrix_containment =
        al_r20_r63x_containment(dropped_matrix_certificate, scalar_oracle);
    result.dropped_matrix_radius = dropped_matrix_certificate.exact
        && !dropped_matrix_containment.exact;

    const std::vector<ALR20R37Dyadic> oriented_inverse{
        one, half, zero, one};
    const std::vector<ALR20R37Dyadic> reversed_inverse{
        one, zero, half, one};
    const ALR20R63RDefect oriented_defect = al_r20_r63r_defect(
        oriented_inverse, identity, one, 2U);
    const ALR20R63RDefect reversed_defect = al_r20_r63r_defect(
        reversed_inverse, identity, one, 2U);
    const ALR20R63XProfile oriented_profile = al_r20_r63x_profile(
        oriented_inverse, exact_rhs, matrix, oriented_defect);
    const ALR20R63XProfile reversed_profile = al_r20_r63x_profile(
        reversed_inverse, exact_rhs, matrix, reversed_defect);
    const std::vector<Binary128> oriented_solution{
        static_cast<Binary128>(1.0), static_cast<Binary128>(-0.5)};
    const ALR20R63UCertificate oriented_oracle = al_r20_r63u_certificate(
        exact_rhs, matrix, oriented_defect, oriented_inverse,
        oriented_solution);
    const ALR20R63XCertificate oriented_certificate =
        al_r20_r63x_certificate(oriented_profile, oriented_solution);
    const ALR20R63XCertificate reversed_certificate =
        al_r20_r63x_certificate(reversed_profile, oriented_solution);
    const ALR20R63XContainment oriented_containment =
        al_r20_r63x_containment(oriented_certificate, oriented_oracle);
    const ALR20R63XContainment reversed_containment =
        al_r20_r63x_containment(reversed_certificate, oriented_oracle);
    result.orientation = oriented_profile.exact && reversed_profile.exact
        && oriented_containment.exact && !reversed_containment.exact;
    ALR20R63XProfile negative = vector_profile;
    negative.vector_radius[0U] = static_cast<Binary128>(-0.5);
    ALR20R63XProfile subnormal = vector_profile;
    subnormal.vector_center[0U] = nextafterq(
        static_cast<Binary128>(0.0), static_cast<Binary128>(1.0));
    ALR20R63XProfile nonfinite = vector_profile;
    nonfinite.matrix_center[0U] = HUGE_VALQ;
    ALR20R63XProfile noncontractive = vector_profile;
    noncontractive.rho_upper = static_cast<Binary128>(1.0);
    result.invalid = !al_r20_r63x_certificate(
        negative, {static_cast<Binary128>(1.0)}).exact
        && !al_r20_r63x_certificate(
            subnormal, {static_cast<Binary128>(1.0)}).exact
        && !al_r20_r63x_certificate(
            nonfinite, {static_cast<Binary128>(1.0)}).exact
        && !al_r20_r63x_certificate(
            noncontractive, {static_cast<Binary128>(1.0)}).exact;
    ALR20R63UCertificate mutated_oracle = identity_oracle;
    mutated_oracle.image[0U] = one;
    const ALR20R63XContainment mutated_containment =
        al_r20_r63x_containment(identity_certificate, mutated_oracle);
    result.independence = identity_certificate.root
            == al_r20_r63x_certificate(profile, solution).root
        && !mutated_containment.exact
        && mutated_containment.root != identity_containment.root;
    result.mutation = vector_certificate.root
        != dropped_vector_certificate.root
        && matrix_certificate.root != dropped_matrix_certificate.root;
    result.classifier =
        al_r20_r63x_classify(false, true, true, true, true)
            == "AFFINE_IMAGE_APPARATUS_REJECTED"
        && al_r20_r63x_classify(true, false, true, true, true)
            == "AFFINE_IMAGE_CONTAINMENT_REJECTED"
        && al_r20_r63x_classify(true, true, false, true, true)
            == "AFFINE_IMAGE_CERTIFICATE_REJECTED"
        && al_r20_r63x_classify(true, true, true, false, true)
            == "RETAINED_WIDE_AFFINE_IMAGE_REJECTED"
        && al_r20_r63x_classify(true, true, true, true, false)
            == "EXPORTED_FACTOR_AFFINE_IMAGE_REJECTED"
        && al_r20_r63x_classify(true, true, true, true, true)
            == "TWO_LANE_AFFINE_IMAGE_CANDIDATE";
    result.exact = result.identity && result.cancellation
        && result.vector_radius && result.matrix_radius
        && result.dropped_vector_radius && result.dropped_matrix_radius
        && result.orientation && result.invalid && result.independence
        && result.mutation && result.classifier;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + profile.root + ':' + identity_certificate.root + ':'
        + cancellation.root + ':' + vector_certificate.root + ':'
        + matrix_certificate.root + ':' + dropped_vector_certificate.root
        + ':' + dropped_matrix_certificate.root + ':'
        + oriented_containment.root + ':' + reversed_containment.root + ':'
        + mutated_containment.root);
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v5_composed_affine_image_controls() {
    al_r20_r63x_last_fixture = ALR20R63XFixture{};
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
    const ALR20R63XControls controls = al_r20_r63x_controls();
    const ALR20R63WControls parent_controls = al_r20_r63w_controls();
    const ALR20R63VControls parent_r63v_controls = al_r20_r63v_controls();
    const ALR20R63UControls parent_r63u_controls = al_r20_r63u_controls();
    const ALR20R63SControls parent_r63s_controls = al_r20_r63s_controls();
    const bool controls_exact = profile_exact && controls.exact
        && parent_controls.exact && parent_r63v_controls.exact
        && parent_r63u_controls.exact && parent_r63s_controls.exact
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
        << parent_r63v_controls.root << ':' << parent_v_wide.root << ':'
        << parent_v_exported.root;
    const std::string parent_r63v_semantic =
        sha256_hex(parent_r63v_material.str());
    const bool parent_r63v_exact = parent_r63u_exact
        && parent_v_wide.root == AL_R20_R63W_WIDE_LANE_SHA256
        && parent_v_exported.root == AL_R20_R63W_EXPORTED_LANE_SHA256
        && parent_r63v_semantic == AL_R20_R63W_PARENT_SEMANTIC_SHA256;
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
    const ALR20R63WProfile parent_w_profile = parent_r63v_exact
        ? al_r20_r63w_profile(common_audit.matrix, verifier_audit.defect)
        : ALR20R63WProfile{};
    const ALR20R63WLane parent_w_wide = parent_w_profile.exact
        ? al_r20_r63w_lane(parent_w_profile, verifier_inverse.value,
            factor_capture.rhs, wide_solutions, wide_oracles)
        : ALR20R63WLane{};
    const ALR20R63WLane parent_w_exported = parent_w_profile.exact
        ? al_r20_r63w_lane(parent_w_profile, verifier_inverse.value,
            factor_capture.rhs, exported_solutions, exported_oracles)
        : ALR20R63WLane{};
    const std::string parent_r63w_route = al_r20_r63w_classify(
        true, true, true, parent_w_wide.passed, parent_w_exported.passed);
    std::ostringstream parent_r63w_material;
    parent_r63w_material
        << "nextengine.nonlocal.nsr3b4e2d7r20r63w-finite-residual-image.v1|"
        << parent_r63v_semantic << ':' << parent_r63w_route << ':'
        << parent_controls.root << ':' << parent_w_profile.root << ':'
        << parent_w_wide.root << ':' << parent_w_exported.root;
    const std::string parent_r63w_semantic =
        sha256_hex(parent_r63w_material.str());
    const bool parent_r63w_exact = parent_r63v_exact
        && parent_w_profile.exact && parent_w_wide.exact
        && parent_w_exported.exact && !parent_w_wide.passed
        && !parent_w_exported.passed
        && parent_r63w_semantic == AL_R20_R63X_PARENT_SEMANTIC_SHA256;

    const ALR20R63XProfile affine_profile = parent_r63w_exact
        ? al_r20_r63x_profile(exact_inverse, exact_rhs,
            common_audit.matrix, verifier_audit.defect)
        : ALR20R63XProfile{};
    const ALR20R63XLane wide = affine_profile.exact
        ? al_r20_r63x_lane(affine_profile, wide_solutions, wide_oracles)
        : ALR20R63XLane{};
    const ALR20R63XLane exported = affine_profile.exact
        ? al_r20_r63x_lane(affine_profile, exported_solutions,
            exported_oracles)
        : ALR20R63XLane{};
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool containment_exact = wide.contained && exported.contained
        && wide.image_containments == 306U
        && exported.image_containments == 306U;
    const bool state2_signs_exact = wide.certificates.size() == 3U
        && exported.certificates.size() == 3U
        && wide.certificates[2U].sign_root == AL_R20_R63W_SIGN_SHA256
        && exported.certificates[2U].sign_root == AL_R20_R63W_SIGN_SHA256;
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_r63w_exact && affine_profile.exact && lifecycle_exact;
    const bool certificate_exact = wide.exact && exported.exact
        && state2_signs_exact;
    const std::string route = al_r20_r63x_classify(apparatus_exact,
        containment_exact, certificate_exact, wide.passed, exported.passed);
    const bool exact = apparatus_exact && containment_exact
        && certificate_exact;
    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63x-composed-affine-image.v1|"
        << parent_r63w_semantic << ':' << route << ':' << controls.root << ':'
        << affine_profile.root << ':' << wide.root << ':' << exported.root;
    const std::string result_sha = sha256_hex(semantic.str());
    al_r20_r63x_last_fixture.exact = exact;
    al_r20_r63x_last_fixture.inverse = exact_inverse;
    al_r20_r63x_last_fixture.rhs = exact_rhs;
    al_r20_r63x_last_fixture.matrix = common_audit.matrix;
    al_r20_r63x_last_fixture.defect = verifier_audit.defect;
    al_r20_r63x_last_fixture.wide_solutions = wide_solutions;
    al_r20_r63x_last_fixture.exported_solutions = exported_solutions;
    al_r20_r63x_last_fixture.wide_oracles = wide_oracles;
    al_r20_r63x_last_fixture.exported_oracles = exported_oracles;
    al_r20_r63x_last_fixture.semantic = result_sha;
    al_r20_r63x_last_fixture.profile_root = affine_profile.root;
    al_r20_r63x_last_fixture.wide_root = wide.root;
    al_r20_r63x_last_fixture.exported_root = exported.root;
    auto lane_json = [](const char* name, const ALR20R63XLane& lane) {
        std::ostringstream stream;
        stream << "\"" << name << "\":{\"exact\":"
            << (lane.exact ? "true" : "false")
            << ",\"contained\":" << (lane.contained ? "true" : "false")
            << ",\"passed\":" << (lane.passed ? "true" : "false")
            << ",\"states\":[";
        for (std::size_t state = 0U;
             state < lane.certificates.size(); ++state) {
            if (state != 0U) stream << ',';
            const ALR20R63XCertificate& certificate =
                lane.certificates[state];
            stream << "{\"state\":" << state << ",\"passed\":"
                << (certificate.passed ? "true" : "false")
                << ",\"error_upper\":\""
                << al_binary128_text(certificate.error_upper)
                << "\",\"positive\":" << certificate.positive
                << ",\"negative\":" << certificate.negative
                << ",\"unresolved\":" << certificate.unresolved
                << ",\"image_contained\":"
                << (lane.containments[state].exact ? "true" : "false")
                << ",\"root\":\"" << certificate.root << "\"}";
        }
        stream << "],\"image_containments\":"
            << lane.image_containments << ",\"root\":\""
            << lane.root << "\"}";
        return stream.str();
    };
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63x_composed_affine_image.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63w_sha256\":\"" << parent_r63w_semantic << "\""
        << ",\"controls_exact\":"
        << (controls.exact ? "true" : "false")
        << ",\"controls_root\":\"" << controls.root << "\""
        << ",\"parent_r63w_exact\":"
        << (parent_r63w_exact ? "true" : "false")
        << ",\"affine_profile\":{\"exact\":"
        << (affine_profile.exact ? "true" : "false")
        << ",\"rho_upper\":\""
        << al_binary128_text(affine_profile.rho_upper)
        << "\",\"maximum_vector_radius\":\""
        << al_binary128_text(affine_profile.maximum_vector_radius)
        << "\",\"maximum_matrix_radius\":\""
        << al_binary128_text(affine_profile.maximum_matrix_radius)
        << "\",\"rhs_products\":" << affine_profile.rhs_products
        << ",\"matrix_products\":" << affine_profile.matrix_products
        << ",\"root\":\"" << affine_profile.root << "\"}"
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
        << ",\"finite_affine_dots\":"
        << (wide.certificates.size() + exported.certificates.size()) * 102U
        << ",\"intermediate_residual_intervals\":0"
        << ",\"exact_oracle_classification_uses\":0"
        << ",\"candidate_updates\":0,\"sparse_constructions\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"gpu_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
