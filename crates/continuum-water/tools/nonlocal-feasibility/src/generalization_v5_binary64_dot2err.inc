namespace {

constexpr const char* AL_R20_R63_PARENT_SEMANTIC_SHA256 =
    "c8f1180669c8e4c75a094467fc5816ba45a2ae384726d4b0554a897e61c1ae2a";
constexpr const char* AL_R20_R63_MATERIAL_SHA256 =
    "2d7693464876f24a11b205b8218e2a8c3042ba3256275c121d9f6ca2aa87857c";
constexpr const char* AL_R20_R63_MATRIX_SHA256 =
    "aa401d0191ad53b7caa7837c827913fa711e1fdc433bc200c020719d297fb09d";
constexpr const char* AL_R20_R63_INVERSE_SHA256 =
    "467e815a4813e7774523699147db38fbcda06dea9b93bfb1696b410e0df0cfd7";
constexpr const char* AL_R20_R63_RHS_SHA256 =
    "64be49510b51f8e9898ed2d021fb2d41d98690cecebe5d95e0a108406cf192b1";
constexpr const char* AL_R20_R63_SOLUTION_SHA256 =
    "2c3a68b39bc82c4de5c09d9f4d4a091cbfd41024b0cdc3a8e509da7effcfe6e3";
constexpr std::size_t AL_R20_R63_DIMENSION = 102U;
constexpr double AL_R20_R63_UNIT = 0x1p-53;

struct ALR20R63Pair {
    bool finite = false;
    double high = 0.0;
    double low = 0.0;
};

ALR20R63Pair al_r20_r63_two_sum(double left, double right) {
    ALR20R63Pair result;
    result.high = left + right;
    const double virtual_right = result.high - left;
    result.low = (left - (result.high - virtual_right))
        + (right - virtual_right);
    result.finite = std::isfinite(result.high) && std::isfinite(result.low);
    return result;
}

ALR20R63Pair al_r20_r63_two_product(double left, double right) {
    ALR20R63Pair result;
    result.high = left * right;
    result.low = std::fma(left, right, -result.high);
    result.finite = std::isfinite(result.high) && std::isfinite(result.low);
    return result;
}

struct ALR20R63Dot2Err {
    bool exact = false;
    std::size_t count = 0U;
    double result = 0.0;
    double error = 0.0;
    double accumulated_error = 0.0;
    std::string root;
};

template <typename Left, typename Right>
ALR20R63Dot2Err al_r20_r63_dot2err(
    std::size_t count, Left left, Right right) {
    ALR20R63Dot2Err result;
    result.count = count;
    const double count_unit = static_cast<double>(count) * AL_R20_R63_UNIT;
    if (count == 0U || !(2.0 * count_unit < 1.0)) return result;
    const ALR20R63Pair first = al_r20_r63_two_product(left(0U), right(0U));
    if (!first.finite) return result;
    double primary = first.high;
    double correction = first.low;
    double accumulated = std::abs(first.low);
    bool finite = std::isfinite(accumulated);
    std::ostringstream material;
    material << count << ':' << binary64_bits(first.high) << ':'
             << binary64_bits(first.low) << ';';
    for (std::size_t index = 1U; index < count && finite; ++index) {
        const ALR20R63Pair product =
            al_r20_r63_two_product(left(index), right(index));
        const ALR20R63Pair sum =
            al_r20_r63_two_sum(primary, product.high);
        const double local = sum.low + product.low;
        correction = correction + local;
        accumulated = accumulated + std::abs(local);
        primary = sum.high;
        finite = product.finite && sum.finite && std::isfinite(local)
            && std::isfinite(correction) && std::isfinite(accumulated);
        material << binary64_bits(product.high) << ':'
                 << binary64_bits(product.low) << ':'
                 << binary64_bits(sum.high) << ':'
                 << binary64_bits(sum.low) << ':'
                 << binary64_bits(local) << ';';
    }
    result.result = primary + correction;
    const double delta = count_unit / (1.0 - 2.0 * count_unit);
    const double eta = std::numeric_limits<double>::denorm_min();
    const double underflow_term = (3.0 * eta) / AL_R20_R63_UNIT;
    const double alpha = AL_R20_R63_UNIT * std::abs(result.result)
        + (delta * accumulated + underflow_term);
    result.error = alpha / (1.0 - 2.0 * AL_R20_R63_UNIT);
    result.accumulated_error = accumulated;
    result.exact = finite && std::isfinite(result.result)
        && std::isfinite(result.error) && result.error >= 0.0
        && std::isfinite(delta) && std::isfinite(underflow_term);
    material << '|' << result.exact << ':'
             << binary64_bits(result.result) << ':'
             << binary64_bits(result.error) << ':'
             << binary64_bits(result.accumulated_error);
    result.root = sha256_hex(material.str());
    return result;
}

ALR20R37Conversion al_r20_r63_exact(double value) {
    return al_r20_r37_from_binary128(static_cast<Binary128>(value));
}

ALR20R37Dyadic al_r20_r63_negate(const ALR20R37Dyadic& value) {
    return {-value.numerator, value.exponent};
}

bool al_r20_r63_nonnegative(const ALR20R37Dyadic& value) {
    return !(value.numerator < 0);
}

bool al_r20_r63_contains(
    const ALR20R37Dyadic& exact, double lower, double upper) {
    const ALR20R37Conversion lower_exact = al_r20_r63_exact(lower);
    const ALR20R37Conversion upper_exact = al_r20_r63_exact(upper);
    if (!lower_exact.exact || !upper_exact.exact || lower > upper)
        return false;
    const ALR20R37Dyadic lower_gap = al_r20_r37_add(
        exact, al_r20_r63_negate(lower_exact.value));
    const ALR20R37Dyadic upper_gap = al_r20_r37_add(
        upper_exact.value, al_r20_r63_negate(exact));
    return al_r20_r63_nonnegative(lower_gap)
        && al_r20_r63_nonnegative(upper_gap);
}

ALR20R37Dyadic al_r20_r63_exact_dot(
    const std::vector<double>& left, const std::vector<double>& right) {
    ALR20R37Dyadic result;
    if (left.size() != right.size()) return result;
    for (std::size_t index = 0U; index < left.size(); ++index) {
        const ALR20R37Conversion lhs = al_r20_r63_exact(left[index]);
        const ALR20R37Conversion rhs = al_r20_r63_exact(right[index]);
        if (!lhs.exact || !rhs.exact) return {};
        result = al_r20_r37_add(result,
            al_r20_r37_multiply(lhs.value, rhs.value));
    }
    return result;
}

bool al_r20_r63_pair_reconstructs(
    double left, double right, bool product) {
    const ALR20R63Pair pair = product
        ? al_r20_r63_two_product(left, right)
        : al_r20_r63_two_sum(left, right);
    const ALR20R37Conversion lhs = al_r20_r63_exact(left);
    const ALR20R37Conversion rhs = al_r20_r63_exact(right);
    const ALR20R37Conversion high = al_r20_r63_exact(pair.high);
    const ALR20R37Conversion low = al_r20_r63_exact(pair.low);
    if (!pair.finite || !lhs.exact || !rhs.exact
        || !high.exact || !low.exact)
        return false;
    const ALR20R37Dyadic expected = product
        ? al_r20_r37_multiply(lhs.value, rhs.value)
        : al_r20_r37_add(lhs.value, rhs.value);
    const ALR20R37Dyadic reconstructed =
        al_r20_r37_add(high.value, low.value);
    return expected.numerator == reconstructed.numerator
        && expected.exponent == reconstructed.exponent;
}

struct ALR20R63DefectInterval {
    bool exact = false;
    double lower = 0.0;
    double upper = 0.0;
    double absolute_bound = 0.0;
};

ALR20R63DefectInterval al_r20_r63_defect_interval(
    const ALR20R63Dot2Err& dot, double identity) {
    ALR20R63DefectInterval result;
    if (!dot.exact) return result;
    const double dot_lower = std::nextafter(
        dot.result - dot.error, -std::numeric_limits<double>::infinity());
    const double dot_upper = std::nextafter(
        dot.result + dot.error, std::numeric_limits<double>::infinity());
    result.lower = std::nextafter(
        identity - dot_upper, -std::numeric_limits<double>::infinity());
    result.upper = std::nextafter(
        identity - dot_lower, std::numeric_limits<double>::infinity());
    result.absolute_bound = std::nextafter(
        std::max(std::abs(result.lower), std::abs(result.upper)),
        std::numeric_limits<double>::infinity());
    result.exact = std::isfinite(result.lower)
        && std::isfinite(result.upper)
        && std::isfinite(result.absolute_bound)
        && result.lower <= result.upper && result.absolute_bound >= 0.0;
    return result;
}

double al_r20_r63_up_add(double left, double right) {
    const double sum = left + right;
    return std::isfinite(sum)
        ? std::nextafter(sum, std::numeric_limits<double>::infinity())
        : sum;
}

struct ALR20R63MatrixAudit {
    bool candidate_exact = false;
    bool oracle_metrics_exact = false;
    bool contained = false;
    bool candidate_contractive = false;
    bool exact_contractive = false;
    std::size_t dimension = 0U;
    std::size_t candidate_dots = 0U;
    std::size_t oracle_dots = 0U;
    std::size_t contained_entries = 0U;
    std::size_t worst_row = 0U;
    double rho_bound = 0.0;
    double maximum_dot_bound = 0.0;
    Binary128 maximum_exact_error = static_cast<Binary128>(0.0);
    Binary128 maximum_error_to_bound = static_cast<Binary128>(0.0);
    std::string candidate_root;
    std::string oracle_root;
    std::string root;
};

ALR20R63MatrixAudit al_r20_r63_matrix_audit(
    const std::vector<double>& matrix,
    const std::vector<double>& inverse,
    const ALR20R37ExactResidual& oracle) {
    ALR20R63MatrixAudit result;
    result.dimension = oracle.dimension;
    result.oracle_root = oracle.root;
    const std::size_t dimension = result.dimension;
    if (!oracle.exact || dimension == 0U
        || matrix.size() != dimension * dimension
        || inverse.size() != matrix.size()
        || oracle.residual.size() != matrix.size())
        return result;
    bool candidate_exact = true;
    bool oracle_metrics_exact = true;
    bool contained = true;
    std::ostringstream candidate_material;
    for (std::size_t row = 0U; row < dimension; ++row) {
        double row_bound = 0.0;
        for (std::size_t column = 0U; column < dimension; ++column) {
            const ALR20R63Dot2Err dot = al_r20_r63_dot2err(
                dimension,
                [&](std::size_t inner) {
                    return matrix[row * dimension + inner];
                },
                [&](std::size_t inner) {
                    return inverse[inner * dimension + column];
                });
            ++result.candidate_dots;
            const ALR20R63DefectInterval interval =
                al_r20_r63_defect_interval(
                    dot, row == column ? 1.0 : 0.0);
            const std::size_t flat = row * dimension + column;
            const bool entry_contained = interval.exact
                && al_r20_r63_contains(
                    oracle.residual[flat], interval.lower, interval.upper);
            if (entry_contained) ++result.contained_entries;
            contained = contained && entry_contained;
            candidate_exact = candidate_exact && dot.exact && interval.exact;
            row_bound = al_r20_r63_up_add(
                row_bound, interval.absolute_bound);
            result.maximum_dot_bound =
                std::max(result.maximum_dot_bound, dot.error);

            const double center = (row == column ? 1.0 : 0.0) - dot.result;
            const ALR20R37Conversion center_exact = al_r20_r63_exact(center);
            const ALR20R37Conversion bound_exact = al_r20_r63_exact(dot.error);
            if (!center_exact.exact || !bound_exact.exact) {
                oracle_metrics_exact = false;
            } else {
                const ALR20R37Dyadic error = al_r20_r37_absolute(
                    al_r20_r37_add(oracle.residual[flat],
                        al_r20_r63_negate(center_exact.value)));
                const Binary128 error_outward =
                    al_r20_r37_binary128_up(error);
                result.maximum_exact_error = std::max(
                    result.maximum_exact_error, error_outward);
                Binary128 ratio = static_cast<Binary128>(0.0);
                if (error.numerator != 0) {
                    ratio = dot.error > 0.0
                        ? error_outward / static_cast<Binary128>(dot.error)
                        : HUGE_VALQ;
                }
                result.maximum_error_to_bound = std::max(
                    result.maximum_error_to_bound, ratio);
            }
            candidate_material << dot.root << ':'
                << binary64_bits(interval.lower) << ':'
                << binary64_bits(interval.upper) << ';';
        }
        if (row == 0U || row_bound > result.rho_bound) {
            result.rho_bound = row_bound;
            result.worst_row = row;
        }
        candidate_material << '|';
    }
    result.oracle_dots = dimension * dimension;
    result.candidate_exact = candidate_exact
        && result.candidate_dots == dimension * dimension
        && std::isfinite(result.rho_bound);
    result.oracle_metrics_exact = oracle_metrics_exact;
    result.contained = contained
        && result.contained_entries == dimension * dimension;
    result.candidate_contractive = result.candidate_exact
        && result.rho_bound < 1.0;
    result.exact_contractive = oracle.contractive;
    result.candidate_root = sha256_hex(candidate_material.str());
    std::ostringstream material;
    material << result.candidate_exact << ':' << result.oracle_metrics_exact
        << ':' << result.contained << ':'
        << result.candidate_contractive << ':' << result.exact_contractive
        << ':' << result.dimension << ':' << result.candidate_dots << ':'
        << result.oracle_dots << ':' << result.contained_entries << ':'
        << result.worst_row << ':' << binary64_bits(result.rho_bound) << ':'
        << binary64_bits(result.maximum_dot_bound) << ':'
        << al_binary128_hex(result.maximum_exact_error) << ':'
        << al_binary128_hex(result.maximum_error_to_bound) << ':'
        << result.candidate_root << ':' << result.oracle_root;
    result.root = sha256_hex(material.str());
    return result;
}

std::vector<Binary128> al_r20_r63_promote(
    const std::vector<double>& values) {
    std::vector<Binary128> promoted;
    promoted.reserve(values.size());
    for (double value : values)
        promoted.push_back(static_cast<Binary128>(value));
    return promoted;
}

struct ALR20R63Controls {
    bool exact = false;
    bool subnormal_preserved = false;
    bool two_sum_exact = false;
    bool two_product_exact = false;
    bool cancellation_contained = false;
    bool underflow_contained = false;
    bool good_contractive = false;
    bool bad_noncontractive = false;
    bool shrunk_rejected = false;
    std::string root;
};

ALR20R63Controls al_r20_r63_controls() {
    ALR20R63Controls result;
    const double denorm = std::numeric_limits<double>::denorm_min();
    result.subnormal_preserved = denorm > 0.0
        && denorm * 1.0 == denorm
        && denorm + denorm == std::ldexp(denorm, 1)
        && std::fma(denorm, 1.0, 0.0) == denorm;
    result.two_sum_exact = al_r20_r63_pair_reconstructs(
        0x1p53, 1.0, false);
    result.two_product_exact = al_r20_r63_pair_reconstructs(
        1.0 + 0x1p-27, 1.0 - 0x1p-27, true);

    const std::vector<double> cancellation_left{1.0, 0x1p53, -0x1p53};
    const std::vector<double> cancellation_right{1.0, 1.0, 1.0};
    const ALR20R63Dot2Err cancellation = al_r20_r63_dot2err(
        cancellation_left.size(),
        [&](std::size_t index) { return cancellation_left[index]; },
        [&](std::size_t index) { return cancellation_right[index]; });
    const ALR20R37Dyadic cancellation_exact =
        al_r20_r63_exact_dot(cancellation_left, cancellation_right);
    result.cancellation_contained = cancellation.exact
        && al_r20_r63_contains(cancellation_exact,
            std::nextafter(cancellation.result - cancellation.error,
                -std::numeric_limits<double>::infinity()),
            std::nextafter(cancellation.result + cancellation.error,
                std::numeric_limits<double>::infinity()));

    const std::vector<double> underflow_left{denorm};
    const std::vector<double> underflow_right{0.5};
    const ALR20R63Dot2Err underflow = al_r20_r63_dot2err(
        1U,
        [&](std::size_t index) { return underflow_left[index]; },
        [&](std::size_t index) { return underflow_right[index]; });
    const ALR20R37Dyadic underflow_exact =
        al_r20_r63_exact_dot(underflow_left, underflow_right);
    result.underflow_contained = underflow.exact
        && underflow_exact.numerator != 0
        && al_r20_r63_contains(underflow_exact,
            std::nextafter(underflow.result - underflow.error,
                -std::numeric_limits<double>::infinity()),
            std::nextafter(underflow.result + underflow.error,
                std::numeric_limits<double>::infinity()));

    const std::vector<double> one{1.0};
    const std::vector<double> minus_one{-1.0};
    const ALR20R37ExactResidual good_oracle = al_r20_r37_exact_residual(
        al_r20_r63_promote(one), al_r20_r63_promote(one), 1U);
    const ALR20R37ExactResidual bad_oracle = al_r20_r37_exact_residual(
        al_r20_r63_promote(one), al_r20_r63_promote(minus_one), 1U);
    const ALR20R63MatrixAudit good =
        al_r20_r63_matrix_audit(one, one, good_oracle);
    const ALR20R63MatrixAudit bad =
        al_r20_r63_matrix_audit(one, minus_one, bad_oracle);
    result.good_contractive = good.candidate_exact && good.contained
        && good.candidate_contractive && good.exact_contractive;
    result.bad_noncontractive = bad.candidate_exact && bad.contained
        && !bad.candidate_contractive && !bad.exact_contractive;

    const std::vector<double> tenth{0.1};
    const ALR20R63Dot2Err inexact = al_r20_r63_dot2err(
        1U,
        [&](std::size_t index) { return tenth[index]; },
        [&](std::size_t index) { return tenth[index]; });
    const ALR20R37Dyadic inexact_exact = al_r20_r63_exact_dot(tenth, tenth);
    result.shrunk_rejected = inexact.exact
        && !al_r20_r63_contains(
            inexact_exact, inexact.result, inexact.result);

    result.exact = result.subnormal_preserved && result.two_sum_exact
        && result.two_product_exact && result.cancellation_contained
        && result.underflow_contained && result.good_contractive
        && result.bad_noncontractive && result.shrunk_rejected;
    std::ostringstream material;
    material << result.exact << ':' << result.subnormal_preserved << ':'
        << result.two_sum_exact << ':' << result.two_product_exact << ':'
        << result.cancellation_contained << ':'
        << result.underflow_contained << ':' << result.good_contractive << ':'
        << result.bad_noncontractive << ':' << result.shrunk_rejected << ':'
        << cancellation.root << ':' << underflow.root << ':'
        << good.root << ':' << bad.root << ':' << inexact.root;
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v5_binary64_dot2err_controls() {
#if defined(__linux__) && defined(__x86_64__) && defined(__GNUC__) \
    && !defined(__FAST_MATH__)
    constexpr bool frozen_platform = true;
#else
    constexpr bool frozen_platform = false;
#endif
    const bool profile_exact = frozen_platform
        && sizeof(double) == 8U && FLT_RADIX == 2 && DBL_MANT_DIG == 53
        && FLT_EVAL_METHOD == 0 && std::numeric_limits<double>::is_iec559
        && std::numeric_limits<double>::has_denorm
            == std::denorm_present
        && std::fegetround() == FE_TONEAREST
        && 2.0 * static_cast<double>(AL_R20_R63_DIMENSION)
            * AL_R20_R63_UNIT < 1.0;
    const ALR20R63Controls controls = al_r20_r63_controls();
    const bool controls_exact = profile_exact && controls.exact
        && al_r20_r37_dyadic_controls();

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
            && problem.problem_root == AL_R20_R60_PROBLEM_SHA256;
    }

    ALR20R60Capture capture;
    ALR20R55SlopeTrace slope;
    ALR20R35CaseAudit oblique;
    std::size_t solver_executions = 0U;
    const bool hook_slot_clear = al_r20_verified_solution_refiner == nullptr
        && al_r20_r60_capture == nullptr
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr;
    if (parent_exact && hook_slot_clear) {
        al_r20_r60_capture = &capture;
        al_r20_verified_solution_refiner = &al_r20_r60_capture_refiner;
        al_r20_r55_slope_trace = &slope;
        al_r20_slope_error_refiner = &al_r20_r55_slope_refiner;
        oblique = al_r20_r35_case(problem);
        al_r20_slope_error_refiner = nullptr;
        al_r20_r55_slope_trace = nullptr;
        al_r20_verified_solution_refiner = nullptr;
        al_r20_r60_capture = nullptr;
        solver_executions = 1U;
    }
    const bool hook_slot_released =
        al_r20_verified_solution_refiner == nullptr
        && al_r20_r60_capture == nullptr
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr
        && al_r20_verified_inverse_capture == nullptr
        && al_r20_direct_solution_capture == nullptr;
    const bool capture_exact = parent_exact && hook_slot_clear
        && hook_slot_released && solver_executions == 1U
        && al_r20_r35_case_observation_exact(oblique)
        && oblique.root == AL_R20_R60_CASE_SHA256
        && oblique.route == "ACTIVE_SET_REJECTED"
        && oblique.diagnostic.accepted_iterations == 8U
        && oblique.diagnostic.total_principal_solves == 1092U
        && oblique.diagnostic.total_transitions == 1092U
        && capture.calls == 1U && capture.dimension == AL_R20_R63_DIMENSION
        && capture.material_root == AL_R20_R63_MATERIAL_SHA256
        && capture.matrix_root == AL_R20_R63_MATRIX_SHA256
        && capture.inverse_root == AL_R20_R63_INVERSE_SHA256
        && capture.rhs_root == AL_R20_R63_RHS_SHA256
        && capture.solution_root == AL_R20_R63_SOLUTION_SHA256
        && capture.matrix.size() == capture.dimension * capture.dimension
        && capture.inverse.size() == capture.matrix.size()
        && capture.rhs.size() == capture.dimension
        && capture.solution.size() == capture.dimension
        && slope.calls == 0U && slope.inverse_audits == 0U;

    std::vector<double> matrix;
    std::vector<double> inverse;
    bool projection_exact = capture_exact;
    if (capture_exact) {
        matrix.reserve(capture.matrix.size());
        inverse.reserve(capture.inverse.size());
        for (Binary128 value : capture.matrix) {
            const double projected = static_cast<double>(value);
            projection_exact = projection_exact && std::isfinite(projected);
            matrix.push_back(projected);
        }
        for (Binary128 value : capture.inverse) {
            const double projected = static_cast<double>(value);
            projection_exact = projection_exact && std::isfinite(projected);
            inverse.push_back(projected);
        }
    }
    const std::string matrix_projection_root = al_r20_double_root(matrix);
    const std::string inverse_projection_root = al_r20_double_root(inverse);

    const std::vector<Binary128> matrix_oracle_input =
        projection_exact ? al_r20_r63_promote(matrix)
                         : std::vector<Binary128>{};
    const std::vector<Binary128> inverse_oracle_input =
        projection_exact ? al_r20_r63_promote(inverse)
                         : std::vector<Binary128>{};
    const ALR20R37ExactResidual right_oracle = projection_exact
        ? al_r20_r37_exact_residual(
            matrix_oracle_input, inverse_oracle_input, capture.dimension)
        : ALR20R37ExactResidual{};
    const ALR20R37ExactResidual left_oracle = projection_exact
        ? al_r20_r37_exact_residual(
            inverse_oracle_input, matrix_oracle_input, capture.dimension)
        : ALR20R37ExactResidual{};
    const ALR20R63MatrixAudit right = right_oracle.exact
        ? al_r20_r63_matrix_audit(matrix, inverse, right_oracle)
        : ALR20R63MatrixAudit{};
    const ALR20R63MatrixAudit left = left_oracle.exact
        ? al_r20_r63_matrix_audit(inverse, matrix, left_oracle)
        : ALR20R63MatrixAudit{};

    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool work_exact = right.candidate_dots == 10404U
        && left.candidate_dots == 10404U
        && right.oracle_dots == 10404U && left.oracle_dots == 10404U;
    const bool oracle_exact = right_oracle.exact && left_oracle.exact
        && right_oracle.dimension == AL_R20_R63_DIMENSION
        && left_oracle.dimension == AL_R20_R63_DIMENSION;
    const bool candidate_exact = right.candidate_exact && left.candidate_exact;
    const bool correspondence_exact = right.oracle_metrics_exact
        && left.oracle_metrics_exact && right.contained && left.contained
        && right.contained_entries == 10404U
        && left.contained_entries == 10404U;
    const bool candidate_decision = candidate_exact
        && right.candidate_contractive && left.candidate_contractive;
    const bool exact = profile_exact && controls_exact && parent_exact
        && capture_exact && projection_exact && oracle_exact
        && candidate_exact && correspondence_exact && candidate_decision
        && work_exact && lifecycle_exact;
    std::string route = "BINARY64_DOT2ERR_PROJECTED_INVERSE_CANDIDATE";
    if (!profile_exact)
        route = "BINARY64_PROFILE_REJECTED";
    else if (!controls_exact)
        route = "BINARY64_DOT2ERR_CONTROL_REJECTED";
    else if (!parent_exact || !capture_exact || !lifecycle_exact)
        route = "BINARY64_DOT2ERR_PARENT_REJECTED";
    else if (!projection_exact || !oracle_exact || !work_exact)
        route = "BINARY64_DOT2ERR_APPARATUS_REJECTED";
    else if (!candidate_exact)
        route = "BINARY64_DOT2ERR_CANDIDATE_REJECTED";
    else if (!correspondence_exact)
        route = "BINARY64_DOT2ERR_CONTAINMENT_REJECTED";
    else if (!right.candidate_contractive)
        route = "BINARY64_RIGHT_NONCONTRACTIVE";
    else if (!left.candidate_contractive)
        route = "BINARY64_LEFT_NONCONTRACTIVE";
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r63-binary64-dot2err.v1|"
        << AL_R20_R63_PARENT_SEMANTIC_SHA256 << ':' << route << ':'
        << oblique.root << ':' << capture.material_root << ':'
        << matrix_projection_root << ':' << inverse_projection_root << ':'
        << controls.root << ':' << right.root << ':' << left.root;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63_binary64_dot2err.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r60_sha256\":\""
        << AL_R20_R63_PARENT_SEMANTIC_SHA256 << "\""
        << ",\"profile\":{\"exact\":"
        << (profile_exact ? "true" : "false")
        << ",\"iec559\":"
        << (std::numeric_limits<double>::is_iec559 ? "true" : "false")
        << ",\"radix\":" << FLT_RADIX
        << ",\"mantissa_bits\":" << DBL_MANT_DIG
        << ",\"eval_method\":" << FLT_EVAL_METHOD
        << ",\"rounding_mode\":" << std::fegetround()
        << ",\"denorm_present\":"
        << (std::numeric_limits<double>::has_denorm
                == std::denorm_present ? "true" : "false") << "}"
        << ",\"controls\":{\"exact\":"
        << (controls_exact ? "true" : "false")
        << ",\"subnormal_preserved\":"
        << (controls.subnormal_preserved ? "true" : "false")
        << ",\"two_sum_exact\":"
        << (controls.two_sum_exact ? "true" : "false")
        << ",\"two_product_exact\":"
        << (controls.two_product_exact ? "true" : "false")
        << ",\"cancellation_contained\":"
        << (controls.cancellation_contained ? "true" : "false")
        << ",\"underflow_contained\":"
        << (controls.underflow_contained ? "true" : "false")
        << ",\"good_contractive\":"
        << (controls.good_contractive ? "true" : "false")
        << ",\"bad_noncontractive\":"
        << (controls.bad_noncontractive ? "true" : "false")
        << ",\"shrunk_rejected\":"
        << (controls.shrunk_rejected ? "true" : "false")
        << ",\"root\":\"" << controls.root << "\"}"
        << ",\"capture\":{\"exact\":"
        << (capture_exact ? "true" : "false")
        << ",\"case_root\":\"" << oblique.root
        << "\",\"material_root\":\"" << capture.material_root
        << "\",\"dimension\":" << capture.dimension
        << ",\"accepted_iterations\":"
        << oblique.diagnostic.accepted_iterations
        << ",\"principal_solves\":"
        << oblique.diagnostic.total_principal_solves
        << ",\"transitions\":"
        << oblique.diagnostic.total_transitions << "}"
        << ",\"projection\":{\"exact\":"
        << (projection_exact ? "true" : "false")
        << ",\"matrix_root\":\"" << matrix_projection_root
        << "\",\"inverse_root\":\"" << inverse_projection_root << "\"}"
        << ",\"right\":{\"candidate_exact\":"
        << (right.candidate_exact ? "true" : "false")
        << ",\"oracle_metrics_exact\":"
        << (right.oracle_metrics_exact ? "true" : "false")
        << ",\"contained\":" << (right.contained ? "true" : "false")
        << ",\"candidate_contractive\":"
        << (right.candidate_contractive ? "true" : "false")
        << ",\"exact_contractive\":"
        << (right.exact_contractive ? "true" : "false")
        << ",\"candidate_rho\":" << right.rho_bound
        << ",\"exact_rho_outward\":\""
        << al_binary128_text(right_oracle.outward_norm)
        << "\",\"contained_entries\":" << right.contained_entries
        << ",\"candidate_dots\":" << right.candidate_dots
        << ",\"oracle_dots\":" << right.oracle_dots
        << ",\"maximum_dot_bound\":" << right.maximum_dot_bound
        << ",\"maximum_exact_error\":\""
        << al_binary128_text(right.maximum_exact_error)
        << "\",\"maximum_error_to_bound\":\""
        << al_binary128_text(right.maximum_error_to_bound)
        << "\",\"candidate_root\":\"" << right.candidate_root
        << "\",\"oracle_root\":\"" << right.oracle_root
        << "\",\"root\":\"" << right.root << "\"}"
        << ",\"left\":{\"candidate_exact\":"
        << (left.candidate_exact ? "true" : "false")
        << ",\"oracle_metrics_exact\":"
        << (left.oracle_metrics_exact ? "true" : "false")
        << ",\"contained\":" << (left.contained ? "true" : "false")
        << ",\"candidate_contractive\":"
        << (left.candidate_contractive ? "true" : "false")
        << ",\"exact_contractive\":"
        << (left.exact_contractive ? "true" : "false")
        << ",\"candidate_rho\":" << left.rho_bound
        << ",\"exact_rho_outward\":\""
        << al_binary128_text(left_oracle.outward_norm)
        << "\",\"contained_entries\":" << left.contained_entries
        << ",\"candidate_dots\":" << left.candidate_dots
        << ",\"oracle_dots\":" << left.oracle_dots
        << ",\"maximum_dot_bound\":" << left.maximum_dot_bound
        << ",\"maximum_exact_error\":\""
        << al_binary128_text(left.maximum_exact_error)
        << "\",\"maximum_error_to_bound\":\""
        << al_binary128_text(left.maximum_error_to_bound)
        << "\",\"candidate_root\":\"" << left.candidate_root
        << "\",\"oracle_root\":\"" << left.oracle_root
        << "\",\"root\":\"" << left.root << "\"}"
        << ",\"candidate_decision\":"
        << (candidate_decision ? "true" : "false")
        << ",\"correspondence_exact\":"
        << (correspondence_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"center_iterations\":0,\"replacement_applies\":0"
        << ",\"new_trials\":0,\"state_updates\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"gpu_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
