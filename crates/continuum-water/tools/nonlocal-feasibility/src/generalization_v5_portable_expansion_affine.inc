namespace {

constexpr const char* AL_R20_R63Y_PARENT_SEMANTIC_SHA256 =
    "904614d78c1948f4af836167936bdc528a0c69a0ded7e85cfe9ae264be395a65";
constexpr const char* AL_R20_R63Y_PARENT_STDOUT_SHA256 =
    "9e218dbd248fc079fd058a44753d5d09b4dbaa8ed5ac9695eca2be68ccf25bf7";
constexpr const char* AL_R20_R63Y_PARENT_PROFILE_SHA256 =
    "0505cdb98db31a7f087cbc23290b5e0adc964dce05f4e73a4429083d860e97d0";
constexpr const char* AL_R20_R63Y_PARENT_WIDE_SHA256 =
    "a52826e657f81c401306fe19451d9dccd99fd4a8d66b1c300bfb9e88bed61cb9";
constexpr const char* AL_R20_R63Y_PARENT_EXPORTED_SHA256 =
    "907c3780ed8e41ae64a74fc60df856a1368348ee96dccb6fe12bb97f73c3a1d1";
constexpr const char* AL_R20_R63Y_SIGN_SHA256 =
    "89b2908b21369cf77a376287ed8142026827815c45a59aa70d2e831aac406094";

bool al_r20_r63y_normal_or_zero(double value) {
    return std::isfinite(value) && std::fpclassify(value) != FP_SUBNORMAL;
}

double al_r20_r63y_up_add(double left, double right) {
    if (left == 0.0 && right == 0.0) return 0.0;
    const double value = left + right;
    return std::isfinite(value)
        ? std::nextafter(value, std::numeric_limits<double>::infinity())
        : value;
}

double al_r20_r63y_up_multiply(double left, double right) {
    if (left == 0.0 || right == 0.0) return 0.0;
    const double value = left * right;
    return std::isfinite(value)
        ? std::nextafter(value, std::numeric_limits<double>::infinity())
        : value;
}

double al_r20_r63y_up_divide(double numerator, double denominator) {
    if (numerator == 0.0 && denominator > 0.0) return 0.0;
    const double value = numerator / denominator;
    return std::isfinite(value)
        ? std::nextafter(value, std::numeric_limits<double>::infinity())
        : value;
}

double al_r20_r63y_outward_double(Binary128 upper) {
    if (upper == static_cast<Binary128>(0.0)) return 0.0;
    double result = static_cast<double>(upper);
    if (std::isfinite(result)
        && static_cast<Binary128>(result) < upper)
        result = std::nextafter(
            result, std::numeric_limits<double>::infinity());
    return result;
}

struct ALR20R63YExpansion {
    bool exact = false;
    std::size_t width = 0U;
    std::vector<double> components;
    double radius = 0.0;
    bool nonzero_low = false;
    std::string root;
};

ALR20R63YExpansion al_r20_r63y_project(
    const ALR20R37Dyadic& numerator,
    const ALR20R37Dyadic& denominator,
    std::size_t width) {
    ALR20R63YExpansion result;
    result.width = width;
    if ((width != 1U && width != 2U)
        || al_r20_r63p_sign(denominator) <= 0) return result;
    ALR20R37Dyadic remainder = numerator;
    bool exact = true;
    for (std::size_t part = 0U; part < width; ++part) {
        const Binary128 projected = al_r20_r63p_fraction_projection(
            remainder, denominator);
        const double component = static_cast<double>(projected);
        const ALR20R37Conversion component_exact =
            al_r20_r63_exact(component);
        exact = exact && component_exact.exact
            && al_r20_r63y_normal_or_zero(component);
        result.components.push_back(component);
        if (component_exact.exact)
            remainder = al_r20_r63p_subtract(remainder,
                al_r20_r37_multiply(component_exact.value, denominator));
    }
    const ALR20R37Dyadic absolute = al_r20_r37_absolute(remainder);
    const Binary128 denominator_down =
        al_r20_r63w_positive_dyadic_down(denominator);
    const Binary128 radius_upper = absolute.numerator == 0
        ? static_cast<Binary128>(0.0)
        : al_r20_r38_up_divide(
            al_r20_r37_binary128_up(absolute), denominator_down);
    result.radius = al_r20_r63y_outward_double(radius_upper);
    const ALR20R37Conversion radius_exact =
        al_r20_r63_exact(result.radius);
    const bool contained = denominator_down > static_cast<Binary128>(0.0)
        && radius_exact.exact && result.radius >= 0.0
        && al_r20_r37_compare_positive(
            al_r20_r37_multiply(radius_exact.value, denominator),
            absolute) >= 0;
    result.nonzero_low = width == 2U && result.components[1U] != 0.0;
    result.exact = exact && contained
        && result.components.size() == width
        && al_r20_r63y_normal_or_zero(result.radius);
    std::ostringstream material;
    material << result.exact << ':' << result.width << ':';
    for (double component : result.components)
        material << binary64_bits(component) << ',';
    material << ':' << binary64_bits(result.radius) << ':'
        << result.nonzero_low;
    result.root = sha256_hex(material.str());
    return result;
}

bool al_r20_r63y_expansion_contains(
    const ALR20R63YExpansion& expansion,
    const ALR20R37Dyadic& numerator,
    const ALR20R37Dyadic& denominator) {
    if (expansion.components.size() != expansion.width
        || al_r20_r63p_sign(denominator) <= 0) return false;
    ALR20R37Dyadic represented;
    for (double component : expansion.components) {
        const ALR20R37Conversion conversion = al_r20_r63_exact(component);
        if (!conversion.exact) return false;
        represented = al_r20_r37_add(represented, conversion.value);
    }
    const ALR20R37Conversion radius = al_r20_r63_exact(expansion.radius);
    if (!radius.exact || expansion.radius < 0.0) return false;
    const ALR20R37Dyadic remainder = al_r20_r37_absolute(
        al_r20_r63p_subtract(numerator,
            al_r20_r37_multiply(represented, denominator)));
    return al_r20_r37_compare_positive(
        al_r20_r37_multiply(radius.value, denominator), remainder) >= 0;
}

struct ALR20R63YExactAffine {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t rhs_products = 0U;
    std::size_t matrix_products = 0U;
    std::vector<ALR20R37Dyadic> vector;
    std::vector<ALR20R37Dyadic> matrix_numerator;
    ALR20R37Dyadic matrix_denominator;
    std::string vector_root;
    std::string matrix_root;
    std::string root;
};

ALR20R63YExactAffine al_r20_r63y_exact_affine(
    const std::vector<ALR20R37Dyadic>& inverse,
    const std::vector<ALR20R37Dyadic>& rhs,
    const ALR20R63QExactMatrix& matrix) {
    ALR20R63YExactAffine result;
    result.dimension = rhs.size();
    result.matrix_denominator = matrix.d;
    const std::size_t dimension = result.dimension;
    if (dimension == 0U || inverse.size() != dimension * dimension
        || !matrix.exact || matrix.rows != dimension
        || matrix.oracle_numerator.size() != dimension * dimension
        || al_r20_r63p_sign(matrix.d) <= 0) return result;
    result.vector.resize(dimension);
    result.matrix_numerator.resize(dimension * dimension);
    for (std::size_t row = 0U; row < dimension; ++row)
        for (std::size_t inner = 0U; inner < dimension; ++inner) {
            result.vector[row] = al_r20_r37_add(result.vector[row],
                al_r20_r37_multiply(
                    inverse[row * dimension + inner], rhs[inner]));
            ++result.rhs_products;
        }
    for (std::size_t row = 0U; row < dimension; ++row)
        for (std::size_t column = 0U; column < dimension; ++column)
            for (std::size_t inner = 0U; inner < dimension; ++inner) {
                result.matrix_numerator[row * dimension + column] =
                    al_r20_r37_add(
                        result.matrix_numerator[
                            row * dimension + column],
                        al_r20_r37_multiply(
                            inverse[row * dimension + inner],
                            matrix.oracle_numerator[
                                inner * dimension + column]));
                ++result.matrix_products;
            }
    result.vector_root = al_r20_r63q_vector_root(result.vector);
    result.matrix_root = al_r20_r63q_vector_root(result.matrix_numerator);
    result.exact = result.rhs_products == dimension * dimension
        && result.matrix_products == dimension * dimension * dimension;
    std::ostringstream material;
    material << result.exact << ':' << result.dimension << ':'
        << result.rhs_products << ':' << result.matrix_products << ':'
        << result.vector_root << ':' << result.matrix_root << ':'
        << al_r20_r37_dyadic_text(result.matrix_denominator);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63YProfile {
    bool exact = false;
    bool contained = false;
    bool contractive = false;
    std::size_t width = 0U;
    std::size_t dimension = 0U;
    std::size_t vector_containments = 0U;
    std::size_t matrix_containments = 0U;
    std::size_t nonzero_vector_lows = 0U;
    std::size_t nonzero_matrix_lows = 0U;
    std::vector<double> vector_components;
    std::vector<double> vector_radius;
    std::vector<double> matrix_components;
    std::vector<double> matrix_radius;
    double rho_upper = 0.0;
    double maximum_vector_radius = 0.0;
    double maximum_matrix_radius = 0.0;
    std::string exact_root;
    std::string component_root;
    std::string radius_root;
    std::string root;
};

ALR20R63YProfile al_r20_r63y_profile(
    const ALR20R63YExactAffine& exact_affine,
    const ALR20R63RDefect& defect,
    std::size_t width) {
    ALR20R63YProfile result;
    result.width = width;
    result.dimension = exact_affine.dimension;
    result.exact_root = exact_affine.root;
    const std::size_t dimension = result.dimension;
    if (!exact_affine.exact || !defect.exact || !defect.left_contractive
        || (width != 1U && width != 2U)
        || !al_r20_r63p_equal(
            exact_affine.matrix_denominator, defect.d)) return result;
    const ALR20R37Dyadic one = al_r20_r63p_control_dyadic(1);
    bool projections_exact = true;
    result.vector_radius.resize(dimension);
    result.matrix_radius.resize(dimension * dimension);
    result.vector_components.reserve(dimension * width);
    result.matrix_components.reserve(dimension * dimension * width);
    for (std::size_t index = 0U; index < dimension; ++index) {
        const ALR20R63YExpansion expansion = al_r20_r63y_project(
            exact_affine.vector[index], one, width);
        projections_exact = projections_exact && expansion.exact;
        result.vector_components.insert(result.vector_components.end(),
            expansion.components.begin(), expansion.components.end());
        result.vector_radius[index] = expansion.radius;
        result.maximum_vector_radius = std::max(
            result.maximum_vector_radius, expansion.radius);
        result.vector_containments += expansion.exact ? 1U : 0U;
        result.nonzero_vector_lows += expansion.nonzero_low ? 1U : 0U;
    }
    for (std::size_t index = 0U; index < dimension * dimension; ++index) {
        const ALR20R63YExpansion expansion = al_r20_r63y_project(
            exact_affine.matrix_numerator[index],
            exact_affine.matrix_denominator, width);
        projections_exact = projections_exact && expansion.exact;
        result.matrix_components.insert(result.matrix_components.end(),
            expansion.components.begin(), expansion.components.end());
        result.matrix_radius[index] = expansion.radius;
        result.maximum_matrix_radius = std::max(
            result.maximum_matrix_radius, expansion.radius);
        result.matrix_containments += expansion.exact ? 1U : 0U;
        result.nonzero_matrix_lows += expansion.nonzero_low ? 1U : 0U;
    }
    const Binary128 denominator_down =
        al_r20_r63w_positive_dyadic_down(defect.d);
    const Binary128 rho128 = al_r20_r38_up_divide(
        al_r20_r37_binary128_up(defect.left_infinity_numerator),
        denominator_down);
    result.rho_upper = al_r20_r63y_outward_double(rho128);
    const ALR20R37Conversion rho_exact = al_r20_r63_exact(result.rho_upper);
    const bool rho_contained = rho_exact.exact
        && al_r20_r37_compare_positive(
            al_r20_r37_multiply(rho_exact.value, defect.d),
            defect.left_infinity_numerator) >= 0;
    result.contained = projections_exact && rho_contained
        && result.vector_containments == dimension
        && result.matrix_containments == dimension * dimension;
    result.contractive = result.rho_upper >= 0.0
        && result.rho_upper < 1.0;
    result.component_root = sha256_hex(
        al_r20_double_root(result.vector_components) + ":"
        + al_r20_double_root(result.matrix_components));
    result.radius_root = sha256_hex(
        al_r20_double_root(result.vector_radius) + ":"
        + al_r20_double_root(result.matrix_radius));
    result.exact = result.contained && result.contractive
        && result.vector_components.size() == dimension * width
        && result.matrix_components.size()
            == dimension * dimension * width
        && al_r20_r63y_normal_or_zero(result.rho_upper)
        && al_r20_r63y_normal_or_zero(result.maximum_vector_radius)
        && al_r20_r63y_normal_or_zero(result.maximum_matrix_radius);
    std::ostringstream material;
    material << result.exact << ':' << result.contained << ':'
        << result.contractive << ':' << result.width << ':'
        << result.dimension << ':' << result.vector_containments << ':'
        << result.matrix_containments << ':'
        << result.nonzero_vector_lows << ':'
        << result.nonzero_matrix_lows << ':'
        << binary64_bits(result.rho_upper) << ':'
        << binary64_bits(result.maximum_vector_radius) << ':'
        << binary64_bits(result.maximum_matrix_radius) << ':'
        << result.exact_root << ':' << result.component_root << ':'
        << result.radius_root;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63YSolution {
    bool exact = false;
    std::size_t width = 0U;
    std::size_t dimension = 0U;
    std::size_t containments = 0U;
    std::size_t nonzero_lows = 0U;
    std::vector<double> components;
    std::vector<double> radius;
    std::string root;
};

ALR20R63YSolution al_r20_r63y_solution(
    const std::vector<Binary128>& solution,
    std::size_t width) {
    ALR20R63YSolution result;
    result.width = width;
    result.dimension = solution.size();
    const ALR20R37Dyadic one = al_r20_r63p_control_dyadic(1);
    result.radius.resize(result.dimension);
    bool exact = result.dimension > 0U;
    for (std::size_t index = 0U; index < result.dimension; ++index) {
        const ALR20R37Conversion conversion =
            al_r20_r37_from_binary128(solution[index]);
        const ALR20R63YExpansion expansion = conversion.exact
            ? al_r20_r63y_project(conversion.value, one, width)
            : ALR20R63YExpansion{};
        exact = exact && conversion.exact && expansion.exact;
        result.components.insert(result.components.end(),
            expansion.components.begin(), expansion.components.end());
        if (index < result.radius.size()) result.radius[index] = expansion.radius;
        result.containments += expansion.exact ? 1U : 0U;
        result.nonzero_lows += expansion.nonzero_low ? 1U : 0U;
    }
    result.exact = exact && result.containments == result.dimension
        && result.components.size() == result.dimension * width;
    std::ostringstream material;
    material << result.exact << ':' << result.width << ':'
        << result.dimension << ':' << result.containments << ':'
        << result.nonzero_lows << ':'
        << al_r20_double_root(result.components) << ':'
        << al_r20_double_root(result.radius);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63YCertificate {
    bool exact = false;
    bool passed = false;
    std::size_t width = 0U;
    std::size_t dimension = 0U;
    std::size_t dots = 0U;
    std::size_t dot_products = 0U;
    std::size_t radius_terms = 0U;
    std::size_t solution_dots = 0U;
    std::size_t sign_comparisons = 0U;
    std::size_t positive = 0U;
    std::size_t negative = 0U;
    std::size_t unresolved = 0U;
    double image_infinity_upper = 0.0;
    double denominator_lower = 0.0;
    double error_upper = 0.0;
    double minimum_separation = std::numeric_limits<double>::infinity();
    std::vector<double> image_center;
    std::vector<double> image_radius;
    std::vector<int> signs;
    std::string profile_root;
    std::string solution_root;
    std::string dot_root;
    std::string sign_root;
    std::string root;
};

ALR20R63YCertificate al_r20_r63y_certificate(
    const ALR20R63YProfile& profile,
    const ALR20R63YSolution& solution) {
    ALR20R63YCertificate result;
    result.width = profile.width;
    result.dimension = profile.dimension;
    result.profile_root = profile.root;
    result.solution_root = solution.root;
    const std::size_t width = result.width;
    const std::size_t dimension = result.dimension;
    if (!profile.exact || !solution.exact
        || solution.width != width || solution.dimension != dimension
        || dimension == 0U
        || profile.vector_components.size() != dimension * width
        || profile.vector_radius.size() != dimension
        || profile.matrix_components.size()
            != dimension * dimension * width
        || profile.matrix_radius.size() != dimension * dimension
        || solution.components.size() != dimension * width
        || solution.radius.size() != dimension) return result;
    const std::size_t dot_count = width + dimension * width * width;
    bool exact = profile.rho_upper >= 0.0 && profile.rho_upper < 1.0
        && profile.vector_components.size() == dimension * width
        && profile.vector_radius.size() == dimension
        && profile.matrix_components.size()
            == dimension * dimension * width
        && profile.matrix_radius.size() == dimension * dimension
        && solution.components.size() == dimension * width
        && solution.radius.size() == dimension;
    for (double value : profile.vector_components)
        exact = exact && al_r20_r63y_normal_or_zero(value);
    for (double value : profile.vector_radius)
        exact = exact && value >= 0.0 && al_r20_r63y_normal_or_zero(value);
    for (double value : profile.matrix_components)
        exact = exact && al_r20_r63y_normal_or_zero(value);
    for (double value : profile.matrix_radius)
        exact = exact && value >= 0.0 && al_r20_r63y_normal_or_zero(value);
    for (double value : solution.components)
        exact = exact && al_r20_r63y_normal_or_zero(value);
    for (double value : solution.radius)
        exact = exact && value >= 0.0 && al_r20_r63y_normal_or_zero(value);
    result.image_center.resize(dimension);
    result.image_radius.resize(dimension);
    std::ostringstream dot_material;
    for (std::size_t row = 0U; row < dimension; ++row) {
        const ALR20R63Dot2Err dot = al_r20_r63_dot2err(
            dot_count,
            [&](std::size_t index) {
                if (index < width)
                    return profile.vector_components[row * width + index];
                std::size_t flat = index - width;
                const std::size_t xpart = flat % width;
                flat /= width;
                const std::size_t mpart = flat % width;
                const std::size_t column = flat / width;
                (void)xpart;
                return profile.matrix_components[
                    (row * dimension + column) * width + mpart];
            },
            [&](std::size_t index) {
                if (index < width) return 1.0;
                std::size_t flat = index - width;
                const std::size_t xpart = flat % width;
                flat /= width;
                flat /= width;
                const std::size_t column = flat;
                return -solution.components[column * width + xpart];
            });
        ++result.dots;
        result.dot_products += dot_count;
        exact = exact && dot.exact;
        double radius = al_r20_r63y_up_add(
            dot.error, profile.vector_radius[row]);
        for (std::size_t column = 0U; column < dimension; ++column) {
            double matrix_abs = 0.0;
            double solution_abs = 0.0;
            for (std::size_t part = 0U; part < width; ++part) {
                matrix_abs = al_r20_r63y_up_add(matrix_abs,
                    std::abs(profile.matrix_components[
                        (row * dimension + column) * width + part]));
                solution_abs = al_r20_r63y_up_add(solution_abs,
                    std::abs(solution.components[column * width + part]));
            }
            const double matrix_r =
                profile.matrix_radius[row * dimension + column];
            const double solution_r = solution.radius[column];
            radius = al_r20_r63y_up_add(radius,
                al_r20_r63y_up_multiply(matrix_abs, solution_r));
            radius = al_r20_r63y_up_add(radius,
                al_r20_r63y_up_multiply(matrix_r, solution_abs));
            radius = al_r20_r63y_up_add(radius,
                al_r20_r63y_up_multiply(matrix_r, solution_r));
            result.radius_terms += 3U;
        }
        result.image_center[row] = dot.result;
        result.image_radius[row] = radius;
        const double magnitude = al_r20_r63y_up_add(
            std::abs(dot.result), radius);
        result.image_infinity_upper = std::max(
            result.image_infinity_upper, magnitude);
        exact = exact && radius >= 0.0
            && al_r20_r63y_normal_or_zero(dot.result)
            && al_r20_r63y_normal_or_zero(radius)
            && al_r20_r63y_normal_or_zero(magnitude);
        dot_material << dot.root << ':' << binary64_bits(radius) << ';';
    }
    result.denominator_lower = std::nextafter(
        1.0 - profile.rho_upper,
        -std::numeric_limits<double>::infinity());
    exact = exact && result.denominator_lower > 0.0;
    if (exact)
        result.error_upper = al_r20_r63y_up_divide(
            result.image_infinity_upper, result.denominator_lower);
    result.signs.resize(dimension);
    for (std::size_t index = 0U; index < dimension; ++index) {
        const ALR20R63Dot2Err center = al_r20_r63_dot2err(
            width,
            [&](std::size_t part) {
                return solution.components[index * width + part];
            },
            [](std::size_t) { return 1.0; });
        ++result.solution_dots;
        double total = al_r20_r63y_up_add(
            center.error, solution.radius[index]);
        total = al_r20_r63y_up_add(total, result.error_upper);
        const double lower = std::nextafter(
            center.result - total,
            -std::numeric_limits<double>::infinity());
        const double upper = std::nextafter(
            center.result + total,
            std::numeric_limits<double>::infinity());
        exact = exact && center.exact && std::isfinite(lower)
            && std::isfinite(upper) && lower <= upper;
        if (lower > 0.0) {
            result.signs[index] = 1;
            ++result.positive;
            result.minimum_separation = std::min(
                result.minimum_separation, lower);
        } else if (upper < 0.0) {
            result.signs[index] = -1;
            ++result.negative;
            result.minimum_separation = std::min(
                result.minimum_separation, -upper);
        } else {
            ++result.unresolved;
        }
        ++result.sign_comparisons;
    }
    result.passed = exact && result.unresolved == 0U
        && result.positive + result.negative == dimension;
    result.dot_root = sha256_hex(dot_material.str());
    std::ostringstream sign_material;
    for (int sign : result.signs) sign_material << sign << ',';
    result.sign_root = sha256_hex(sign_material.str());
    result.exact = exact && result.dots == dimension
        && result.dot_products == dimension * dot_count
        && result.radius_terms == 3U * dimension * dimension
        && result.solution_dots == dimension
        && result.sign_comparisons == dimension
        && result.positive + result.negative + result.unresolved == dimension
        && al_r20_r63y_normal_or_zero(result.image_infinity_upper)
        && al_r20_r63y_normal_or_zero(result.denominator_lower)
        && al_r20_r63y_normal_or_zero(result.error_upper);
    std::ostringstream material;
    material << result.exact << ':' << result.passed << ':'
        << result.width << ':' << result.dimension << ':'
        << result.dots << ':' << result.dot_products << ':'
        << result.radius_terms << ':' << result.solution_dots << ':'
        << result.sign_comparisons << ':' << result.positive << ':'
        << result.negative << ':' << result.unresolved << ':'
        << binary64_bits(result.image_infinity_upper) << ':'
        << binary64_bits(result.denominator_lower) << ':'
        << binary64_bits(result.error_upper) << ':'
        << binary64_bits(result.minimum_separation) << ':'
        << result.profile_root << ':' << result.solution_root << ':'
        << result.dot_root << ':' << result.sign_root << ':'
        << al_r20_double_root(result.image_center) << ':'
        << al_r20_double_root(result.image_radius);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63YContainment {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t entries = 0U;
    std::string root;
};

ALR20R63YContainment al_r20_r63y_containment(
    const ALR20R63YCertificate& finite,
    const ALR20R63UCertificate& oracle) {
    ALR20R63YContainment result;
    result.dimension = finite.dimension;
    if (!finite.exact || !oracle.exact
        || oracle.dimension != result.dimension
        || finite.image_center.size() != result.dimension
        || finite.image_radius.size() != result.dimension
        || oracle.image.size() != result.dimension) return result;
    std::ostringstream material;
    for (std::size_t index = 0U; index < result.dimension; ++index) {
        const bool contained = al_r20_r63w_contains_fraction(
            static_cast<Binary128>(finite.image_center[index]),
            static_cast<Binary128>(finite.image_radius[index]),
            oracle.image[index], oracle.d);
        result.entries += contained ? 1U : 0U;
        material << index << ':' << contained << ';';
    }
    result.exact = result.entries == result.dimension;
    material << '|' << result.exact << ':' << result.dimension << ':'
        << result.entries;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63YLane {
    bool finite_exact = false;
    bool contained = false;
    bool passed = false;
    bool reject_reject_pass = false;
    std::size_t width = 0U;
    std::size_t dimension = 0U;
    std::size_t image_containments = 0U;
    std::vector<ALR20R63YSolution> solutions;
    std::vector<ALR20R63YCertificate> certificates;
    std::vector<ALR20R63YContainment> containments;
    std::string root;
};

ALR20R63YLane al_r20_r63y_lane(
    const ALR20R63YProfile& profile,
    const std::vector<std::vector<Binary128>>& source_solutions,
    const std::vector<ALR20R63UCertificate>& oracles) {
    ALR20R63YLane result;
    result.width = profile.width;
    result.dimension = profile.dimension;
    if (source_solutions.size() != 3U || oracles.size() != 3U) return result;
    bool finite_exact = true;
    bool contained = true;
    for (std::size_t state = 0U; state < 3U; ++state) {
        ALR20R63YSolution solution = al_r20_r63y_solution(
            source_solutions[state], profile.width);
        ALR20R63YCertificate certificate = al_r20_r63y_certificate(
            profile, solution);
        ALR20R63YContainment containment = al_r20_r63y_containment(
            certificate, oracles[state]);
        finite_exact = finite_exact && solution.exact && certificate.exact;
        contained = contained && containment.exact;
        result.image_containments += containment.entries;
        result.solutions.push_back(std::move(solution));
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
    result.finite_exact = finite_exact;
    result.contained = contained
        && result.image_containments == 3U * result.dimension;
    result.passed = result.finite_exact && result.reject_reject_pass;
    std::ostringstream material;
    material << result.finite_exact << ':' << result.contained << ':'
        << result.passed << ':' << result.reject_reject_pass << ':'
        << result.width << ':' << result.dimension << ':'
        << result.image_containments << '|';
    for (std::size_t state = 0U; state < 3U; ++state)
        material << result.solutions[state].root << ':'
            << result.certificates[state].root << ':'
            << result.containments[state].root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r20_r63y_classify(bool apparatus, bool containment,
    bool onefold, bool retained_twofold, bool exported_twofold) {
    if (!apparatus) return "PORTABLE_EXPANSION_APPARATUS_REJECTED";
    if (!containment) return "PORTABLE_EXPANSION_CONTAINMENT_REJECTED";
    if (onefold) return "ONEFOLD_AFFINE_IMAGE_CANDIDATE";
    if (!retained_twofold)
        return "RETAINED_TWOFOLD_AFFINE_IMAGE_REJECTED";
    if (!exported_twofold)
        return "EXPORTED_TWOFOLD_AFFINE_IMAGE_REJECTED";
    return "TWOFOLD_AFFINE_IMAGE_CANDIDATE";
}

struct ALR20R63YControls {
    bool exact = false;
    bool onefold_reconstruction = false;
    bool twofold_reconstruction = false;
    bool nonzero_low = false;
    bool cancellation = false;
    bool coefficient_radius = false;
    bool candidate_radius = false;
    bool dropped_coefficient_radius = false;
    bool dropped_candidate_radius = false;
    bool two_sum = false;
    bool two_product_fma = false;
    bool invalid_rejection = false;
    bool noncontractive_rejection = false;
    bool orientation = false;
    bool oracle_independence = false;
    bool mutation = false;
    bool classifier = false;
    std::string root;
};

struct ALR20R63YLastResult {
    bool exact = false;
    std::size_t selected_width = 0U;
    std::string route;
    std::string semantic;
    std::string controls_root;
    std::string onefold_profile_root;
    std::string twofold_profile_root;
};

ALR20R63YLastResult al_r20_r63y_last_result;

ALR20R63YControls al_r20_r63y_controls() {
    ALR20R63YControls result;
    const ALR20R37Dyadic zero = al_r20_r63p_control_dyadic(0);
    const ALR20R37Dyadic one = al_r20_r63p_control_dyadic(1);
    const ALR20R37Dyadic minus_one = al_r20_r63p_control_dyadic(-1);
    const ALR20R37Dyadic three = al_r20_r63p_control_dyadic(3);
    const ALR20R63YExpansion onefold =
        al_r20_r63y_project(one, three, 1U);
    const ALR20R63YExpansion twofold =
        al_r20_r63y_project(one, three, 2U);
    result.onefold_reconstruction = onefold.exact
        && al_r20_r63y_expansion_contains(onefold, one, three);
    result.twofold_reconstruction = twofold.exact
        && al_r20_r63y_expansion_contains(twofold, one, three);
    result.nonzero_low = twofold.nonzero_low
        && twofold.components.size() == 2U
        && twofold.components[1U] != 0.0;
    result.coefficient_radius = onefold.radius > 0.0
        && twofold.radius >= 0.0 && twofold.radius < onefold.radius;
    result.candidate_radius = result.coefficient_radius;
    ALR20R63YExpansion dropped_onefold = onefold;
    dropped_onefold.radius = 0.0;
    result.dropped_coefficient_radius =
        !al_r20_r63y_expansion_contains(dropped_onefold, one, three);
    ALR20R63YExpansion dropped_twofold = twofold;
    dropped_twofold.radius = 0.0;
    result.dropped_candidate_radius =
        !al_r20_r63y_expansion_contains(dropped_twofold, one, three);

    const std::vector<ALR20R37Dyadic> identity{one, zero, zero, one};
    ALR20R63QExactMatrix identity_matrix;
    identity_matrix.exact = true;
    identity_matrix.rows = 2U;
    identity_matrix.columns = 2U;
    identity_matrix.d = one;
    identity_matrix.oracle_numerator = identity;
    const ALR20R63RDefect identity_defect = al_r20_r63r_defect(
        identity, identity, one, 2U);
    const std::vector<ALR20R37Dyadic> rhs{one, minus_one};
    const ALR20R63YExactAffine exact_affine =
        al_r20_r63y_exact_affine(identity, rhs, identity_matrix);
    const ALR20R63YProfile profile = al_r20_r63y_profile(
        exact_affine, identity_defect, 1U);
    const std::vector<Binary128> source_solution{
        static_cast<Binary128>(1.0), static_cast<Binary128>(-1.0)};
    const ALR20R63YSolution solution =
        al_r20_r63y_solution(source_solution, 1U);
    const ALR20R63YCertificate cancellation =
        al_r20_r63y_certificate(profile, solution);
    result.cancellation = cancellation.exact && cancellation.passed
        && cancellation.image_center.size() == 2U
        && cancellation.image_center[0U] == 0.0
        && cancellation.image_center[1U] == 0.0
        && cancellation.image_radius[0U] > 0.0
        && cancellation.image_radius[1U] > 0.0;

    result.two_sum = al_r20_r63_pair_reconstructs(
        0x1.fffffffffffffp+0, 0x1p-53, false);
    result.two_product_fma = al_r20_r63_pair_reconstructs(
        0x1.0000000000001p+0, 0x1.fffffffffffffp-1, true);
    const ALR20R63YExpansion subnormal = al_r20_r63y_project(
        al_r20_r63p_control_dyadic(1, -1074), one, 1U);
    const ALR20R63YExpansion overflow = al_r20_r63y_project(
        al_r20_r63p_control_dyadic(1, 1024), one, 1U);
    ALR20R63YProfile nonfinite_profile = profile;
    nonfinite_profile.vector_components[0U] =
        std::numeric_limits<double>::infinity();
    const ALR20R63YCertificate nonfinite =
        al_r20_r63y_certificate(nonfinite_profile, solution);
    result.invalid_rejection = !subnormal.exact && !overflow.exact
        && !nonfinite.exact;
    ALR20R63YProfile noncontractive_profile = profile;
    noncontractive_profile.rho_upper = 1.0;
    const ALR20R63YCertificate noncontractive =
        al_r20_r63y_certificate(noncontractive_profile, solution);
    result.noncontractive_rejection = !noncontractive.exact;

    const ALR20R37Dyadic eight = al_r20_r63p_control_dyadic(8);
    ALR20R63QExactMatrix oriented_matrix = identity_matrix;
    oriented_matrix.d = eight;
    oriented_matrix.oracle_numerator = {
        eight, al_r20_r63p_control_dyadic(2),
        one, eight};
    ALR20R63QExactMatrix transposed_matrix = oriented_matrix;
    transposed_matrix.oracle_numerator = {
        eight, one,
        al_r20_r63p_control_dyadic(2), eight};
    const ALR20R63RDefect oriented_defect = al_r20_r63r_defect(
        identity, oriented_matrix.oracle_numerator, eight, 2U);
    const ALR20R63RDefect transposed_defect = al_r20_r63r_defect(
        identity, transposed_matrix.oracle_numerator, eight, 2U);
    const ALR20R63YExactAffine oriented = al_r20_r63y_exact_affine(
        identity, rhs, oriented_matrix);
    const ALR20R63YExactAffine transposed = al_r20_r63y_exact_affine(
        identity, rhs, transposed_matrix);
    const ALR20R63YProfile oriented_profile = al_r20_r63y_profile(
        oriented, oriented_defect, 2U);
    const ALR20R63YProfile transposed_profile = al_r20_r63y_profile(
        transposed, transposed_defect, 2U);
    const ALR20R63YSolution oriented_solution =
        al_r20_r63y_solution(source_solution, 2U);
    const ALR20R63YCertificate oriented_certificate =
        al_r20_r63y_certificate(oriented_profile, oriented_solution);
    const ALR20R63YCertificate transposed_certificate =
        al_r20_r63y_certificate(transposed_profile, oriented_solution);
    result.orientation = oriented.exact && transposed.exact
        && oriented_profile.exact && transposed_profile.exact
        && oriented_certificate.exact && transposed_certificate.exact
        && oriented.matrix_root != transposed.matrix_root
        && oriented_profile.root != transposed_profile.root
        && oriented_certificate.root != transposed_certificate.root;

    const ALR20R63UCertificate oracle = al_r20_r63u_certificate(
        rhs, identity_matrix, identity_defect, identity, source_solution);
    const ALR20R63YContainment contained =
        al_r20_r63y_containment(cancellation, oracle);
    ALR20R63UCertificate mutated_oracle = oracle;
    mutated_oracle.image[0U] = al_r20_r37_add(
        mutated_oracle.image[0U], al_r20_r63p_control_dyadic(16));
    const ALR20R63YContainment rejected_oracle =
        al_r20_r63y_containment(cancellation, mutated_oracle);
    const ALR20R63YCertificate repeated =
        al_r20_r63y_certificate(profile, solution);
    result.oracle_independence = contained.exact
        && !rejected_oracle.exact && repeated.root == cancellation.root;
    ALR20R63YSolution mutated_solution = solution;
    mutated_solution.components[0U] = std::nextafter(
        mutated_solution.components[0U],
        std::numeric_limits<double>::infinity());
    const ALR20R63YCertificate mutated =
        al_r20_r63y_certificate(profile, mutated_solution);
    result.mutation = mutated.exact && mutated.root != cancellation.root;
    result.classifier =
        al_r20_r63y_classify(false, false, true, true, true)
            == "PORTABLE_EXPANSION_APPARATUS_REJECTED"
        && al_r20_r63y_classify(true, false, true, true, true)
            == "PORTABLE_EXPANSION_CONTAINMENT_REJECTED"
        && al_r20_r63y_classify(true, true, true, false, false)
            == "ONEFOLD_AFFINE_IMAGE_CANDIDATE"
        && al_r20_r63y_classify(true, true, false, false, true)
            == "RETAINED_TWOFOLD_AFFINE_IMAGE_REJECTED"
        && al_r20_r63y_classify(true, true, false, true, false)
            == "EXPORTED_TWOFOLD_AFFINE_IMAGE_REJECTED"
        && al_r20_r63y_classify(true, true, false, true, true)
            == "TWOFOLD_AFFINE_IMAGE_CANDIDATE";
    result.exact = result.onefold_reconstruction
        && result.twofold_reconstruction && result.nonzero_low
        && result.cancellation && result.coefficient_radius
        && result.candidate_radius && result.dropped_coefficient_radius
        && result.dropped_candidate_radius && result.two_sum
        && result.two_product_fma && result.invalid_rejection
        && result.noncontractive_rejection && result.orientation
        && result.oracle_independence && result.mutation
        && result.classifier;
    std::ostringstream material;
    material << result.exact << ':' << result.onefold_reconstruction << ':'
        << result.twofold_reconstruction << ':' << result.nonzero_low << ':'
        << result.cancellation << ':' << result.coefficient_radius << ':'
        << result.candidate_radius << ':'
        << result.dropped_coefficient_radius << ':'
        << result.dropped_candidate_radius << ':' << result.two_sum << ':'
        << result.two_product_fma << ':' << result.invalid_rejection << ':'
        << result.noncontractive_rejection << ':' << result.orientation << ':'
        << result.oracle_independence << ':' << result.mutation << ':'
        << result.classifier << ':' << onefold.root << ':' << twofold.root
        << ':' << cancellation.root << ':' << subnormal.root << ':'
        << overflow.root << ':' << nonfinite.root << ':'
        << noncontractive.root << ':' << oriented_profile.root << ':'
        << transposed_profile.root << ':' << oriented_certificate.root << ':'
        << transposed_certificate.root << ':' << contained.root << ':'
        << rejected_oracle.root << ':' << mutated.root;
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v5_portable_expansion_affine_controls() {
    al_r20_r63y_last_result = ALR20R63YLastResult{};
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
    const ALR20R63YControls controls = al_r20_r63y_controls();
    const SplitBoundaryReport parent =
        run_al_generalization_v5_composed_affine_image_controls();
    const ALR20R63XFixture fixture = al_r20_r63x_last_fixture;
    const bool parent_exact = parent.passed && fixture.exact
        && fixture.semantic == AL_R20_R63Y_PARENT_SEMANTIC_SHA256
        && fixture.profile_root == AL_R20_R63Y_PARENT_PROFILE_SHA256
        && fixture.wide_root == AL_R20_R63Y_PARENT_WIDE_SHA256
        && fixture.exported_root == AL_R20_R63Y_PARENT_EXPORTED_SHA256;
    const ALR20R63YExactAffine exact_affine = parent_exact
        ? al_r20_r63y_exact_affine(
            fixture.inverse, fixture.rhs, fixture.matrix)
        : ALR20R63YExactAffine{};
    const ALR20R63YProfile onefold_profile = exact_affine.exact
        ? al_r20_r63y_profile(exact_affine, fixture.defect, 1U)
        : ALR20R63YProfile{};
    const ALR20R63YProfile twofold_profile = exact_affine.exact
        ? al_r20_r63y_profile(exact_affine, fixture.defect, 2U)
        : ALR20R63YProfile{};
    const ALR20R63YLane onefold_wide = onefold_profile.exact
        ? al_r20_r63y_lane(onefold_profile, fixture.wide_solutions,
            fixture.wide_oracles)
        : ALR20R63YLane{};
    const ALR20R63YLane onefold_exported = onefold_profile.exact
        ? al_r20_r63y_lane(onefold_profile, fixture.exported_solutions,
            fixture.exported_oracles)
        : ALR20R63YLane{};
    const ALR20R63YLane twofold_wide = twofold_profile.exact
        ? al_r20_r63y_lane(twofold_profile, fixture.wide_solutions,
            fixture.wide_oracles)
        : ALR20R63YLane{};
    const ALR20R63YLane twofold_exported = twofold_profile.exact
        ? al_r20_r63y_lane(twofold_profile, fixture.exported_solutions,
            fixture.exported_oracles)
        : ALR20R63YLane{};
    const bool finite_exact = onefold_wide.finite_exact
        && onefold_exported.finite_exact && twofold_wide.finite_exact
        && twofold_exported.finite_exact;
    const std::size_t image_containments =
        onefold_wide.image_containments
        + onefold_exported.image_containments
        + twofold_wide.image_containments
        + twofold_exported.image_containments;
    const bool containment_exact = onefold_wide.contained
        && onefold_exported.contained && twofold_wide.contained
        && twofold_exported.contained
        && image_containments == 1224U;
    auto frozen_state2 = [](const ALR20R63YLane& lane) {
        return lane.certificates.size() == 3U
            && (!lane.certificates[2U].passed
                || lane.certificates[2U].sign_root == AL_R20_R63Y_SIGN_SHA256);
    };
    const bool state2_signs_exact = frozen_state2(onefold_wide)
        && frozen_state2(onefold_exported) && frozen_state2(twofold_wide)
        && frozen_state2(twofold_exported);
    const bool apparatus_exact = profile_exact && controls.exact
        && parent_exact && exact_affine.exact && onefold_profile.exact
        && twofold_profile.exact && finite_exact && state2_signs_exact;
    const bool onefold_passed = onefold_wide.passed
        && onefold_exported.passed;
    const std::string route = al_r20_r63y_classify(
        apparatus_exact, containment_exact, onefold_passed,
        twofold_wide.passed, twofold_exported.passed);
    const std::size_t selected_width = onefold_passed ? 1U
        : (twofold_wide.passed && twofold_exported.passed ? 2U : 0U);
    const bool exact = apparatus_exact && containment_exact;
    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63y-portable-expansion-affine.v1|"
        << fixture.semantic << ':' << route << ':' << controls.root << ':'
        << exact_affine.root << ':' << onefold_profile.root << ':'
        << twofold_profile.root << ':' << onefold_wide.root << ':'
        << onefold_exported.root << ':' << twofold_wide.root << ':'
        << twofold_exported.root;
    const std::string result_sha = sha256_hex(semantic.str());
    al_r20_r63y_last_result.exact = exact;
    al_r20_r63y_last_result.selected_width = selected_width;
    al_r20_r63y_last_result.route = route;
    al_r20_r63y_last_result.semantic = result_sha;
    al_r20_r63y_last_result.controls_root = controls.root;
    al_r20_r63y_last_result.onefold_profile_root = onefold_profile.root;
    al_r20_r63y_last_result.twofold_profile_root = twofold_profile.root;
    auto lane_json = [](const char* name, const ALR20R63YLane& lane) {
        std::ostringstream stream;
        stream << "\"" << name << "\":{\"finite_exact\":"
            << (lane.finite_exact ? "true" : "false")
            << ",\"contained\":" << (lane.contained ? "true" : "false")
            << ",\"passed\":" << (lane.passed ? "true" : "false")
            << ",\"states\":[";
        for (std::size_t state = 0U;
             state < lane.certificates.size(); ++state) {
            if (state != 0U) stream << ',';
            const ALR20R63YCertificate& certificate =
                lane.certificates[state];
            stream << "{\"state\":" << state << ",\"passed\":"
                << (certificate.passed ? "true" : "false")
                << ",\"error_upper\":" << certificate.error_upper
                << ",\"positive\":" << certificate.positive
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
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63y_portable_expansion_affine.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63x_sha256\":\"" << fixture.semantic << "\""
        << ",\"parent_stdout_sha256\":\""
        << AL_R20_R63Y_PARENT_STDOUT_SHA256 << "\""
        << ",\"controls_exact\":"
        << (controls.exact ? "true" : "false")
        << ",\"controls_root\":\"" << controls.root << "\""
        << ",\"exact_affine\":{\"exact\":"
        << (exact_affine.exact ? "true" : "false")
        << ",\"rhs_products\":" << exact_affine.rhs_products
        << ",\"matrix_products\":" << exact_affine.matrix_products
        << ",\"root\":\"" << exact_affine.root << "\"}"
        << ",\"profiles\":[{\"width\":1,\"rho_upper\":"
        << onefold_profile.rho_upper << ",\"maximum_vector_radius\":"
        << onefold_profile.maximum_vector_radius
        << ",\"maximum_matrix_radius\":"
        << onefold_profile.maximum_matrix_radius
        << ",\"nonzero_lows\":"
        << onefold_profile.nonzero_vector_lows
            + onefold_profile.nonzero_matrix_lows
        << ",\"root\":\"" << onefold_profile.root
        << "\"},{\"width\":2,\"rho_upper\":"
        << twofold_profile.rho_upper << ",\"maximum_vector_radius\":"
        << twofold_profile.maximum_vector_radius
        << ",\"maximum_matrix_radius\":"
        << twofold_profile.maximum_matrix_radius
        << ",\"nonzero_lows\":"
        << twofold_profile.nonzero_vector_lows
            + twofold_profile.nonzero_matrix_lows
        << ",\"root\":\"" << twofold_profile.root << "\"}]"
        << ",\"lanes\":{" << lane_json("onefold_wide", onefold_wide)
        << ',' << lane_json("onefold_exported", onefold_exported) << ','
        << lane_json("twofold_wide", twofold_wide) << ','
        << lane_json("twofold_exported", twofold_exported) << '}'
        << ",\"selected_width\":" << selected_width
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"containment_exact\":"
        << (containment_exact ? "true" : "false")
        << ",\"finite_exact\":" << (finite_exact ? "true" : "false")
        << ",\"state2_signs_exact\":"
        << (state2_signs_exact ? "true" : "false")
        << ",\"image_containments\":" << image_containments
        << ",\"finite_certificate_binary128_fields\":0"
        << ",\"exact_oracle_classification_uses\":0"
        << ",\"adaptive_width\":false,\"candidate_updates\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"gpu_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
