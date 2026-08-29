namespace {

constexpr const char* AL_R20_R63P_PARENT_SEMANTIC_SHA256 =
    "4b2434fa485f6d3f5362169587da44ffc744d0b448a9384ebe769f3f3dc2f270";
constexpr const char* AL_R20_R63P_FULL_MATRIX_SHA256 =
    "aa401d0191ad53b7caa7837c827913fa711e1fdc433bc200c020719d297fb09d";
constexpr const char* AL_R20_R63P_WITNESS_SHA256 =
    "507258b6c55abcb9261c7c7e45f0834ad2189cab139feac941d290d994dce864";
constexpr const char* AL_R20_R63P_DEFECT_SHA256 =
    "e531f06788e134c1fa65e39dd8e4f0be8e6350b23b781c990d6e7d72ac666f94";

ALR20R37Dyadic al_r20_r63p_negate(ALR20R37Dyadic value) {
    value.numerator = -value.numerator;
    return value;
}

ALR20R37Dyadic al_r20_r63p_subtract(
    const ALR20R37Dyadic& left, const ALR20R37Dyadic& right) {
    return al_r20_r37_add(left, al_r20_r63p_negate(right));
}

bool al_r20_r63p_equal(
    const ALR20R37Dyadic& left, const ALR20R37Dyadic& right) {
    return left.exponent == right.exponent
        && left.numerator == right.numerator;
}

int al_r20_r63p_sign(const ALR20R37Dyadic& value) {
    if (value.numerator == 0) return 0;
    return value.numerator.negative ? -1 : 1;
}

bool al_r20_r63p_less_absolute(
    const ALR20R37Dyadic& left, const ALR20R37Dyadic& right) {
    return al_r20_r37_compare_positive(
        al_r20_r37_absolute(left), al_r20_r37_absolute(right)) < 0;
}

Binary128 al_r20_r63p_dyadic_projection(const ALR20R37Dyadic& value) {
    return al_r20_r37_binary128_up(value);
}

Binary128 al_r20_r63p_fraction_projection(
    const ALR20R37Dyadic& numerator,
    const ALR20R37Dyadic& denominator) {
    if (denominator.numerator == 0) return HUGE_VALQ;
    return al_r20_r63p_dyadic_projection(numerator)
        / al_r20_r63p_dyadic_projection(denominator);
}

std::string al_r20_r63p_dyadic_root(const ALR20R37Dyadic& value) {
    return sha256_hex(al_r20_r37_dyadic_text(value));
}

struct ALR20R63PArbitration {
    bool exact = false;
    bool nonidempotent = false;
    bool rank_one_identity = false;
    bool dense_selects_h = false;
    bool tangent_selects_k = false;
    bool common_oracle_selects_tangent = false;
    bool gap_sign = false;
    bool gap_dominance = false;
    bool structural = false;
    ALR20R37Dyadic s_minus_d;
    ALR20R37Dyadic d_squared;
    ALR20R37Dyadic h_star_numerator;
    ALR20R37Dyadic k_star_numerator;
    ALR20R37Dyadic delta_numerator;
    ALR20R37Dyadic observed_gap;
    ALR20R37Dyadic dense_h_error_numerator;
    ALR20R37Dyadic dense_k_error_numerator;
    ALR20R37Dyadic tangent_h_error_numerator;
    ALR20R37Dyadic tangent_k_error_numerator;
    ALR20R37Dyadic gap_residual_numerator;
    std::string root;
};

ALR20R63PArbitration al_r20_r63p_arbitrate(
    const ALR20R37Dyadic& s,
    const ALR20R37Dyadic& d,
    const ALR20R37Dyadic& z2,
    const ALR20R37Dyadic& g,
    const ALR20R37Dyadic& sigma,
    const ALR20R37Dyadic& h_observed,
    const ALR20R37Dyadic& k_observed) {
    ALR20R63PArbitration result;
    const ALR20R37Dyadic two{ALR20R37Integer(2), 0};
    result.s_minus_d = al_r20_r63p_subtract(s, d);
    result.d_squared = al_r20_r37_multiply(d, d);
    const ALR20R37Dyadic g_squared = al_r20_r37_multiply(g, g);
    const ALR20R37Dyadic z2_d = al_r20_r37_multiply(z2, d);
    const ALR20R37Dyadic h_inner = al_r20_r63p_subtract(z2_d, g_squared);
    result.h_star_numerator = al_r20_r37_multiply(sigma, h_inner);
    result.delta_numerator = al_r20_r37_multiply(sigma,
        al_r20_r37_multiply(result.s_minus_d, g_squared));

    const ALR20R37Dyadic z2_d2 = al_r20_r37_multiply(z2,
        result.d_squared);
    const ALR20R37Dyadic two_g2_d = al_r20_r37_multiply(two,
        al_r20_r37_multiply(g_squared, d));
    const ALR20R37Dyadic s_g2 = al_r20_r37_multiply(s, g_squared);
    const ALR20R37Dyadic k_inner = al_r20_r37_add(
        al_r20_r63p_subtract(z2_d2, two_g2_d), s_g2);
    result.k_star_numerator = al_r20_r37_multiply(sigma, k_inner);

    const ALR20R37Dyadic h_common = al_r20_r37_multiply(
        result.h_star_numerator, d);
    result.rank_one_identity = al_r20_r63p_equal(
        al_r20_r63p_subtract(result.k_star_numerator, h_common),
        result.delta_numerator);
    result.observed_gap = al_r20_r63p_subtract(k_observed, h_observed);

    result.dense_h_error_numerator = al_r20_r37_absolute(
        al_r20_r63p_subtract(
            al_r20_r37_multiply(h_observed, d),
            result.h_star_numerator));
    result.dense_k_error_numerator = al_r20_r37_absolute(
        al_r20_r63p_subtract(
            al_r20_r37_multiply(h_observed, result.d_squared),
            result.k_star_numerator));
    result.tangent_h_error_numerator = al_r20_r37_absolute(
        al_r20_r63p_subtract(
            al_r20_r37_multiply(k_observed, d),
            result.h_star_numerator));
    result.tangent_k_error_numerator = al_r20_r37_absolute(
        al_r20_r63p_subtract(
            al_r20_r37_multiply(k_observed, result.d_squared),
            result.k_star_numerator));

    const ALR20R37Dyadic dense_h_error_common = al_r20_r37_multiply(
        result.dense_h_error_numerator, d);
    const ALR20R37Dyadic tangent_h_error_common = al_r20_r37_multiply(
        result.tangent_h_error_numerator, d);
    result.dense_selects_h = al_r20_r63p_less_absolute(
        dense_h_error_common, result.dense_k_error_numerator);
    result.tangent_selects_k = al_r20_r63p_less_absolute(
        result.tangent_k_error_numerator, tangent_h_error_common);
    result.common_oracle_selects_tangent = al_r20_r63p_less_absolute(
        result.tangent_h_error_numerator,
        result.dense_h_error_numerator);

    const ALR20R37Dyadic observed_gap_common = al_r20_r37_multiply(
        result.observed_gap, result.d_squared);
    result.gap_residual_numerator = al_r20_r37_absolute(
        al_r20_r63p_subtract(
            observed_gap_common, result.delta_numerator));
    result.gap_sign = al_r20_r63p_sign(result.observed_gap)
            == al_r20_r63p_sign(result.delta_numerator)
        && al_r20_r63p_sign(result.delta_numerator) != 0;
    result.gap_dominance = al_r20_r63p_less_absolute(
            result.gap_residual_numerator, observed_gap_common)
        && al_r20_r63p_less_absolute(
            result.gap_residual_numerator, result.delta_numerator);
    result.nonidempotent = al_r20_r63p_sign(result.s_minus_d) != 0;
    result.structural = result.rank_one_identity
        && result.dense_selects_h && result.tangent_selects_k
        && result.gap_sign && result.gap_dominance;
    result.exact = al_r20_r63p_sign(d) > 0
        && al_r20_r63p_sign(sigma) > 0
        && al_r20_r63p_sign(z2) >= 0
        && al_r20_r63p_sign(result.d_squared) > 0
        && result.rank_one_identity;
    std::ostringstream material;
    material << result.exact << ':' << result.nonidempotent << ':'
        << result.rank_one_identity << ':' << result.dense_selects_h << ':'
        << result.tangent_selects_k << ':'
        << result.common_oracle_selects_tangent << ':' << result.gap_sign << ':'
        << result.gap_dominance << ':' << result.structural << ':'
        << al_r20_r37_dyadic_text(result.s_minus_d) << ':'
        << al_r20_r37_dyadic_text(result.d_squared) << ':'
        << al_r20_r37_dyadic_text(result.h_star_numerator) << ':'
        << al_r20_r37_dyadic_text(result.k_star_numerator) << ':'
        << al_r20_r37_dyadic_text(result.delta_numerator) << ':'
        << al_r20_r37_dyadic_text(result.observed_gap) << ':'
        << al_r20_r37_dyadic_text(result.dense_h_error_numerator) << ':'
        << al_r20_r37_dyadic_text(result.dense_k_error_numerator) << ':'
        << al_r20_r37_dyadic_text(result.tangent_h_error_numerator) << ':'
        << al_r20_r37_dyadic_text(result.tangent_k_error_numerator) << ':'
        << al_r20_r37_dyadic_text(result.gap_residual_numerator);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63PObservation {
    bool exact = false;
    std::size_t free_components = 0U;
    std::size_t norm_terms = 0U;
    std::size_t source_combination_terms = 0U;
    std::size_t source_norm_terms = 0U;
    std::size_t radial_dot_terms = 0U;
    std::size_t dense_quadratic_terms = 0U;
    std::size_t dense_multiplications = 0U;
    std::size_t tangent_combination_terms = 0U;
    std::size_t tangent_norm_terms = 0U;
    ALR20R37Dyadic s;
    ALR20R37Dyadic d;
    ALR20R37Dyadic z2;
    ALR20R37Dyadic g;
    ALR20R37Dyadic sigma;
    ALR20R37Dyadic h_observed;
    ALR20R37Dyadic k_observed;
    ALR20R63PArbitration arbitration;
    std::string source_root;
    std::string tangent_image_root;
    std::string root;
};

ALR20R63PObservation al_r20_r63p_observe(
    const ALR20R63EProjectorGrams& grams,
    const ALR20R63FTangentMatrix& tangent,
    const ALR20ProjectorState& projector,
    const ALR20R63EWitness& witness,
    const std::vector<Binary128>& dense,
    Binary128 sigma_value) {
    ALR20R63PObservation result;
    const std::size_t rows = grams.rows;
    const std::size_t scalars = grams.scalars;
    if (!grams.exact || !tangent.exact || !projector.exact || !witness.exact
        || rows == 0U || tangent.rows != rows || tangent.columns != scalars
        || witness.alpha.size() != rows || dense.size() != rows * rows
        || projector.mask.size() != scalars
        || projector.value.size() * 3U != scalars) return result;
    bool conversions_exact = true;
    auto convert = [&conversions_exact](Binary128 value) {
        const ALR20R37Conversion conversion =
            al_r20_r37_from_binary128(value);
        conversions_exact = conversions_exact && conversion.exact;
        return conversion.value;
    };
    result.d = convert(projector.free_norm_squared);
    result.sigma = convert(sigma_value);
    std::vector<ALR20R37Dyadic> alpha(rows);
    for (std::size_t row = 0U; row < rows; ++row)
        alpha[row] = convert(witness.alpha[row]);

    std::vector<ALR20R37Dyadic> y(scalars);
    std::vector<ALR20R37Dyadic> z(scalars);
    for (std::size_t scalar = 0U; scalar < scalars; ++scalar) {
        if (projector.mask[scalar] == 0) {
            y[scalar] = convert(al_r20_q_component(
                projector.value[scalar / 3U],
                static_cast<int>(scalar % 3U)));
            result.s = al_r20_r37_add(result.s,
                al_r20_r37_multiply(y[scalar], y[scalar]));
            ++result.free_components;
            ++result.norm_terms;
        }
        for (std::size_t row = 0U; row < rows; ++row) {
            const ALR20R37Dyadic source = convert(al_r20_q_component(
                grams.source_rows[row][scalar / 3U],
                static_cast<int>(scalar % 3U)));
            z[scalar] = al_r20_r37_add(z[scalar],
                al_r20_r37_multiply(alpha[row], source));
            ++result.source_combination_terms;
        }
    }
    std::ostringstream source_material;
    for (const ALR20R37Dyadic& value : z)
        source_material << al_r20_r37_dyadic_text(value) << ';';
    result.source_root = sha256_hex(source_material.str());
    for (std::size_t scalar = 0U; scalar < scalars; ++scalar)
        if (projector.mask[scalar] == 0) {
            result.z2 = al_r20_r37_add(result.z2,
                al_r20_r37_multiply(z[scalar], z[scalar]));
            result.g = al_r20_r37_add(result.g,
                al_r20_r37_multiply(y[scalar], z[scalar]));
            ++result.source_norm_terms;
            ++result.radial_dot_terms;
        }

    for (std::size_t row = 0U; row < rows; ++row)
        for (std::size_t column = 0U; column < rows; ++column) {
            const ALR20R37Dyadic matrix_value = convert(
                dense[row * rows + column]);
            const ALR20R37Dyadic term = al_r20_r37_multiply(alpha[row],
                al_r20_r37_multiply(matrix_value, alpha[column]));
            result.h_observed = al_r20_r37_add(result.h_observed, term);
            ++result.dense_quadratic_terms;
            result.dense_multiplications += 2U;
        }

    std::vector<ALR20R37Dyadic> tangent_image(scalars);
    for (std::size_t scalar = 0U; scalar < scalars; ++scalar) {
        for (std::size_t row = 0U; row < rows; ++row) {
            const ALR20R37Dyadic coefficient = convert(
                tangent.values[row * scalars + scalar]);
            tangent_image[scalar] = al_r20_r37_add(tangent_image[scalar],
                al_r20_r37_multiply(alpha[row], coefficient));
            ++result.tangent_combination_terms;
        }
        result.k_observed = al_r20_r37_add(result.k_observed,
            al_r20_r37_multiply(
                tangent_image[scalar], tangent_image[scalar]));
        ++result.tangent_norm_terms;
    }
    result.k_observed = al_r20_r37_multiply(
        result.sigma, result.k_observed);
    std::ostringstream tangent_material;
    for (const ALR20R37Dyadic& value : tangent_image)
        tangent_material << al_r20_r37_dyadic_text(value) << ';';
    result.tangent_image_root = sha256_hex(tangent_material.str());
    result.arbitration = al_r20_r63p_arbitrate(result.s, result.d,
        result.z2, result.g, result.sigma, result.h_observed,
        result.k_observed);
    result.exact = conversions_exact && result.arbitration.exact
        && result.free_components == 174U && result.norm_terms == 174U
        && result.source_combination_terms == rows * scalars
        && result.source_norm_terms == 174U
        && result.radial_dot_terms == 174U
        && result.dense_quadratic_terms == rows * rows
        && result.dense_multiplications == 2U * rows * rows
        && result.tangent_combination_terms == rows * scalars
        && result.tangent_norm_terms == scalars;
    std::ostringstream material;
    material << result.exact << ':' << result.free_components << ':'
        << result.norm_terms << ':' << result.source_combination_terms << ':'
        << result.source_norm_terms << ':' << result.radial_dot_terms << ':'
        << result.dense_quadratic_terms << ':'
        << result.dense_multiplications << ':'
        << result.tangent_combination_terms << ':'
        << result.tangent_norm_terms << ':'
        << al_r20_r37_dyadic_text(result.s) << ':'
        << al_r20_r37_dyadic_text(result.d) << ':'
        << al_r20_r37_dyadic_text(result.z2) << ':'
        << al_r20_r37_dyadic_text(result.g) << ':'
        << al_r20_r37_dyadic_text(result.sigma) << ':'
        << al_r20_r37_dyadic_text(result.h_observed) << ':'
        << al_r20_r37_dyadic_text(result.k_observed) << ':'
        << result.arbitration.root << ':' << result.source_root << ':'
        << result.tangent_image_root;
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r20_r63p_classify(
    bool apparatus, bool nonidempotent, bool structural) {
    if (!apparatus) return "PROJECTOR_IDEMPOTENCE_APPARATUS_REJECTED";
    if (!nonidempotent) return "STORED_PROJECTOR_IDEMPOTENT_BOUNDARY";
    if (!structural) return "PROJECTOR_RANK_ONE_EXPLANATION_REJECTED";
    return "SOURCE_JVP_SOURCE_OPERATOR_CANDIDATE";
}

struct ALR20R63PControls {
    bool exact = false;
    bool idempotent = false;
    bool negative_defect = false;
    bool positive_defect = false;
    bool normalized = false;
    bool orientation = false;
    bool mutation = false;
    bool classifier = false;
    std::string root;
};

ALR20R37Dyadic al_r20_r63p_control_dyadic(
    std::int64_t numerator, int exponent = 0) {
    return al_r20_r37_normalize({ALR20R37Integer(numerator), exponent});
}

ALR20R63PControls al_r20_r63p_controls() {
    ALR20R63PControls result;
    const ALR20R37Dyadic one = al_r20_r63p_control_dyadic(1);
    const ALR20R37Dyadic two = al_r20_r63p_control_dyadic(2);
    const ALR20R37Dyadic three = al_r20_r63p_control_dyadic(3);
    const ALR20R37Dyadic four = al_r20_r63p_control_dyadic(4);
    const ALR20R37Dyadic seven_halves = al_r20_r63p_control_dyadic(7, -1);
    const ALR20R37Dyadic thirteen_quarters =
        al_r20_r63p_control_dyadic(13, -2);
    const ALR20R63PArbitration idempotent = al_r20_r63p_arbitrate(
        one, one, four, one, one, three, three);
    result.idempotent = idempotent.exact && !idempotent.nonidempotent
        && al_r20_r63p_classify(true, idempotent.nonidempotent,
            idempotent.structural)
            == "STORED_PROJECTOR_IDEMPOTENT_BOUNDARY";
    const ALR20R63PArbitration negative = al_r20_r63p_arbitrate(
        one, two, four, one, one, seven_halves, thirteen_quarters);
    result.negative_defect = negative.exact && negative.nonidempotent
        && negative.structural
        && al_r20_r63p_sign(negative.s_minus_d) < 0;
    const ALR20R63PArbitration positive = al_r20_r63p_arbitrate(
        two, one, four, one, one, three, four);
    result.positive_defect = positive.exact && positive.nonidempotent
        && positive.structural
        && al_r20_r63p_sign(positive.s_minus_d) > 0;
    const ALR20R63PArbitration normalized = al_r20_r63p_arbitrate(
        two, two, four, one, one, seven_halves, seven_halves);
    result.normalized = normalized.exact && !normalized.nonidempotent
        && normalized.delta_numerator.numerator == 0;
    const ALR20R63PArbitration swapped = al_r20_r63p_arbitrate(
        one, two, four, one, one, thirteen_quarters, seven_halves);
    result.orientation = swapped.exact && !swapped.structural
        && swapped.root != negative.root;
    const ALR20R63PArbitration mutated = al_r20_r63p_arbitrate(
        one, two, four, one, one, seven_halves,
        al_r20_r63p_control_dyadic(25, -3));
    result.mutation = mutated.exact && mutated.root != negative.root;
    result.classifier =
        al_r20_r63p_classify(false, false, false)
            == "PROJECTOR_IDEMPOTENCE_APPARATUS_REJECTED"
        && al_r20_r63p_classify(true, false, true)
            == "STORED_PROJECTOR_IDEMPOTENT_BOUNDARY"
        && al_r20_r63p_classify(true, true, false)
            == "PROJECTOR_RANK_ONE_EXPLANATION_REJECTED"
        && al_r20_r63p_classify(true, true, true)
            == "SOURCE_JVP_SOURCE_OPERATOR_CANDIDATE";
    result.exact = result.idempotent && result.negative_defect
        && result.positive_defect && result.normalized && result.orientation
        && result.mutation && result.classifier;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + idempotent.root + ':' + negative.root + ':' + positive.root + ':'
        + normalized.root + ':' + swapped.root + ':' + mutated.root);
    return result;
}

} // namespace

SplitBoundaryReport
run_al_generalization_v5_projector_idempotence_controls() {
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
    const ALR20R63PControls controls = al_r20_r63p_controls();
    const bool controls_exact = profile_exact && controls.exact;
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
    const ALR20R63EWitness witness = full_profile.exact
        ? al_r20_r63e_witness(
            grams, factor_capture.projector, full_profile.binary128)
        : ALR20R63EWitness{};
    const ALR20R63FTangentMatrix tangent = grams.exact
        ? al_r20_r63f_tangent_matrix(grams, factor_capture.projector,
            factor_capture.source_rows)
        : ALR20R63FTangentMatrix{};
    const Binary128 inverse_scale = static_cast<Binary128>(1.0)
        + factor_capture.projector.eta;
    const Binary128 sigma = static_cast<Binary128>(1.0) / inverse_scale;
    const ALR20R63ODefect defect = tangent.exact
        ? al_r20_r63o_defect(factor_capture.matrix, tangent.values,
            tangent.rows, tangent.columns, sigma)
        : ALR20R63ODefect{};
    const bool parent_r63o_exact = capture_exact && grams.exact
        && grams.full_root == AL_R20_R63P_FULL_MATRIX_SHA256
        && full_profile.exact && witness.exact
        && witness.root == AL_R20_R63P_WITNESS_SHA256
        && tangent.exact && tangent.value_root
            == AL_R20_R63N_TANGENT_VALUE_SHA256
        && defect.exact && defect.root == AL_R20_R63P_DEFECT_SHA256;
    const ALR20R63PObservation observation = parent_r63o_exact
        ? al_r20_r63p_observe(grams, tangent, factor_capture.projector,
            witness, factor_capture.matrix, sigma)
        : ALR20R63PObservation{};
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool work_exact = observation.free_components == 174U
        && observation.norm_terms == 174U
        && observation.source_combination_terms == 32130U
        && observation.source_norm_terms == 174U
        && observation.radial_dot_terms == 174U
        && observation.dense_quadratic_terms == 10404U
        && observation.dense_multiplications == 20808U
        && observation.tangent_combination_terms == 32130U
        && observation.tangent_norm_terms == 315U;
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_r63o_exact && observation.exact && work_exact
        && lifecycle_exact;
    const std::string route = al_r20_r63p_classify(apparatus_exact,
        observation.arbitration.nonidempotent,
        observation.arbitration.structural);
    const bool exact = apparatus_exact;
    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63p-projector-idempotence.v1|"
        << AL_R20_R63P_PARENT_SEMANTIC_SHA256 << ':' << route << ':'
        << controls.root << ':' << defect.root << ':' << observation.root;
    const std::string result_sha = sha256_hex(semantic.str());
    const ALR20R63PArbitration& arbitration = observation.arbitration;
    const Binary128 h_star = al_r20_r63p_fraction_projection(
        arbitration.h_star_numerator, observation.d);
    const Binary128 k_star = al_r20_r63p_fraction_projection(
        arbitration.k_star_numerator, arbitration.d_squared);
    const Binary128 delta = al_r20_r63p_fraction_projection(
        arbitration.delta_numerator, arbitration.d_squared);
    const Binary128 gap_residual = al_r20_r63p_fraction_projection(
        arbitration.gap_residual_numerator, arbitration.d_squared);
    const Binary128 dense_h_error = al_r20_r63p_fraction_projection(
        arbitration.dense_h_error_numerator, observation.d);
    const Binary128 dense_k_error = al_r20_r63p_fraction_projection(
        arbitration.dense_k_error_numerator, arbitration.d_squared);
    const Binary128 tangent_h_error = al_r20_r63p_fraction_projection(
        arbitration.tangent_h_error_numerator, observation.d);
    const Binary128 tangent_k_error = al_r20_r63p_fraction_projection(
        arbitration.tangent_k_error_numerator, arbitration.d_squared);
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63p_projector_idempotence.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63o_sha256\":\""
        << AL_R20_R63P_PARENT_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"controls_root\":\"" << controls.root << "\""
        << ",\"parent_r63o_exact\":"
        << (parent_r63o_exact ? "true" : "false")
        << ",\"projector\":{\"free_components\":"
        << observation.free_components << ",\"s\":\""
        << al_binary128_text(al_r20_r63p_dyadic_projection(observation.s))
        << "\",\"stored_d\":\""
        << al_binary128_text(al_r20_r63p_dyadic_projection(observation.d))
        << "\",\"s_minus_d\":\""
        << al_binary128_text(
            al_r20_r63p_dyadic_projection(arbitration.s_minus_d))
        << "\",\"s_root\":\"" << al_r20_r63p_dyadic_root(observation.s)
        << "\",\"d_root\":\"" << al_r20_r63p_dyadic_root(observation.d)
        << "\",\"difference_root\":\""
        << al_r20_r63p_dyadic_root(arbitration.s_minus_d) << "\"}"
        << ",\"weak_operator\":{\"h_star\":\""
        << al_binary128_text(h_star) << "\",\"k_star\":\""
        << al_binary128_text(k_star) << "\",\"delta\":\""
        << al_binary128_text(delta) << "\",\"h_observed\":\""
        << al_binary128_text(
            al_r20_r63p_dyadic_projection(observation.h_observed))
        << "\",\"k_observed\":\""
        << al_binary128_text(
            al_r20_r63p_dyadic_projection(observation.k_observed))
        << "\",\"observed_gap\":\""
        << al_binary128_text(
            al_r20_r63p_dyadic_projection(arbitration.observed_gap))
        << "\",\"gap_residual\":\""
        << al_binary128_text(gap_residual)
        << "\",\"dense_to_h_star_error\":\""
        << al_binary128_text(dense_h_error)
        << "\",\"dense_to_k_star_error\":\""
        << al_binary128_text(dense_k_error)
        << "\",\"tangent_to_h_star_error\":\""
        << al_binary128_text(tangent_h_error)
        << "\",\"tangent_to_k_star_error\":\""
        << al_binary128_text(tangent_k_error)
        << "\",\"rank_one_identity\":"
        << (arbitration.rank_one_identity ? "true" : "false")
        << ",\"dense_selects_h\":"
        << (arbitration.dense_selects_h ? "true" : "false")
        << ",\"tangent_selects_k\":"
        << (arbitration.tangent_selects_k ? "true" : "false")
        << ",\"common_oracle_selects_tangent\":"
        << (arbitration.common_oracle_selects_tangent ? "true" : "false")
        << ",\"gap_sign\":" << (arbitration.gap_sign ? "true" : "false")
        << ",\"gap_dominance\":"
        << (arbitration.gap_dominance ? "true" : "false")
        << ",\"root\":\"" << arbitration.root << "\"}"
        << ",\"observation_root\":\"" << observation.root << "\""
        << ",\"source_root\":\"" << observation.source_root << "\""
        << ",\"tangent_image_root\":\""
        << observation.tangent_image_root << "\""
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"exact_source_combination_terms\":"
        << observation.source_combination_terms
        << ",\"exact_dense_quadratic_terms\":"
        << observation.dense_quadratic_terms
        << ",\"exact_tangent_combination_terms\":"
        << observation.tangent_combination_terms
        << ",\"rhs_solves\":0,\"factor_solves\":0,\"pcg_updates\":0"
        << ",\"rank_one_corrections\":0,\"sparse_constructions\":0"
        << ",\"rank_actions\":0,\"row_drops\":0,\"regularizations\":0"
        << ",\"replacement_applies\":0,\"following_transitions\":0"
        << ",\"state_updates\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"gpu_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
