namespace {

constexpr const char* AL_R20_R63U_PARENT_SEMANTIC_SHA256 =
    "97d992f18912ed9a223a17080820b72d4da54635534d96dba3dc9e5d71454e14";
constexpr const char* AL_R20_R63U_CAUSAL_R63T_SHA256 =
    "8aa1e245058243cf1d33fd00c7d6688082fe7c708c532fb5f1fb9fd0454fff2c";
constexpr const char* AL_R20_R63U_WIDE_LANE_SHA256 =
    "2ec70f637f37e8927a86c071b24029e6d0bf4724968ba40609a46dbc23195376";
constexpr const char* AL_R20_R63U_EXPORTED_LANE_SHA256 =
    "5dcc65f74b8769d5a53cf5db2937f4c2013dc4b92f0333bb42a791006bc3c255";

struct ALR20R63UCertificate {
    bool exact = false;
    bool passed = false;
    std::size_t dimension = 0U;
    std::size_t solution_conversions = 0U;
    std::size_t residual_products = 0U;
    std::size_t residual_entries = 0U;
    std::size_t image_products = 0U;
    std::size_t image_entries = 0U;
    std::size_t sign_comparisons = 0U;
    std::size_t positive = 0U;
    std::size_t negative = 0U;
    std::size_t unresolved = 0U;
    ALR20R37Dyadic d;
    ALR20R37Dyadic residual_infinity_numerator;
    ALR20R37Dyadic image_infinity_numerator;
    ALR20R37Dyadic coarse_error_numerator;
    ALR20R37Dyadic error_denominator;
    ALR20R37Dyadic minimum_resolved_separation;
    std::vector<ALR20R37Dyadic> residual;
    std::vector<ALR20R37Dyadic> image;
    std::vector<int> signs;
    std::string solution_root;
    std::string residual_root;
    std::string image_root;
    std::string sign_root;
    std::string root;
};

ALR20R63UCertificate al_r20_r63u_certificate(
    const std::vector<ALR20R37Dyadic>& rhs,
    const ALR20R63QExactMatrix& matrix,
    const ALR20R63RDefect& defect,
    const std::vector<ALR20R37Dyadic>& inverse,
    const std::vector<Binary128>& solution) {
    ALR20R63UCertificate result;
    result.dimension = rhs.size();
    result.d = matrix.d;
    const std::size_t dimension = result.dimension;
    if (dimension == 0U || solution.size() != dimension
        || inverse.size() != dimension * dimension
        || !matrix.exact || matrix.rows != dimension
        || matrix.oracle_numerator.size() != dimension * dimension
        || !defect.exact || !defect.left_contractive
        || !al_r20_r63p_equal(matrix.d, defect.d)
        || defect.inverse_root != al_r20_r63q_vector_root(inverse)
        || al_r20_r63p_sign(defect.amplification_denominator) <= 0)
        return result;
    bool conversions_exact = true;
    std::vector<ALR20R37Dyadic> exact_solution(dimension);
    for (std::size_t index = 0U; index < dimension; ++index) {
        const ALR20R37Conversion conversion =
            al_r20_r37_from_binary128(solution[index]);
        conversions_exact = conversions_exact && conversion.exact;
        exact_solution[index] = conversion.value;
        ++result.solution_conversions;
    }
    result.residual.resize(dimension);
    result.image.resize(dimension);
    for (std::size_t row = 0U; row < dimension; ++row) {
        ALR20R37Dyadic value;
        for (std::size_t column = 0U; column < dimension; ++column) {
            value = al_r20_r37_add(value, al_r20_r37_multiply(
                matrix.oracle_numerator[row * dimension + column],
                exact_solution[column]));
            ++result.residual_products;
        }
        result.residual[row] = al_r20_r63p_subtract(
            al_r20_r37_multiply(rhs[row], matrix.d), value);
        const ALR20R37Dyadic absolute =
            al_r20_r37_absolute(result.residual[row]);
        if (row == 0U || al_r20_r37_compare_positive(
                absolute, result.residual_infinity_numerator) > 0)
            result.residual_infinity_numerator = absolute;
        ++result.residual_entries;
    }
    for (std::size_t row = 0U; row < dimension; ++row) {
        for (std::size_t column = 0U; column < dimension; ++column) {
            result.image[row] = al_r20_r37_add(result.image[row],
                al_r20_r37_multiply(
                    inverse[row * dimension + column],
                    result.residual[column]));
            ++result.image_products;
        }
        const ALR20R37Dyadic absolute =
            al_r20_r37_absolute(result.image[row]);
        if (row == 0U || al_r20_r37_compare_positive(
                absolute, result.image_infinity_numerator) > 0)
            result.image_infinity_numerator = absolute;
        ++result.image_entries;
    }
    result.coarse_error_numerator = al_r20_r37_multiply(
        defect.inverse_infinity_norm,
        result.residual_infinity_numerator);
    result.error_denominator = defect.amplification_denominator;
    result.signs.resize(dimension);
    bool have_resolved = false;
    for (std::size_t index = 0U; index < dimension; ++index) {
        const ALR20R37Dyadic scaled = al_r20_r37_multiply(
            al_r20_r37_absolute(exact_solution[index]),
            result.error_denominator);
        const int comparison = al_r20_r37_compare_positive(
            scaled, result.image_infinity_numerator);
        if (al_r20_r63p_sign(exact_solution[index]) > 0
            && comparison > 0) {
            result.signs[index] = 1;
            ++result.positive;
        } else if (al_r20_r63p_sign(exact_solution[index]) < 0
            && comparison > 0) {
            result.signs[index] = -1;
            ++result.negative;
        } else {
            ++result.unresolved;
        }
        if (comparison > 0) {
            const ALR20R37Dyadic separation = al_r20_r63p_subtract(
                scaled, result.image_infinity_numerator);
            if (!have_resolved || al_r20_r37_compare_positive(
                    separation, result.minimum_resolved_separation) < 0) {
                result.minimum_resolved_separation = separation;
                have_resolved = true;
            }
        }
        ++result.sign_comparisons;
    }
    result.passed = result.unresolved == 0U
        && result.positive + result.negative == dimension;
    result.solution_root = al_r20_r63q_vector_root(exact_solution);
    result.residual_root = al_r20_r63q_vector_root(result.residual);
    result.image_root = al_r20_r63q_vector_root(result.image);
    std::ostringstream sign_material;
    for (int sign : result.signs) sign_material << sign << ',';
    result.sign_root = sha256_hex(sign_material.str());
    result.exact = conversions_exact
        && result.solution_conversions == dimension
        && result.residual_products == dimension * dimension
        && result.residual_entries == dimension
        && result.image_products == dimension * dimension
        && result.image_entries == dimension
        && result.sign_comparisons == dimension
        && result.positive + result.negative + result.unresolved == dimension
        && (!result.passed || have_resolved);
    std::ostringstream material;
    material << result.exact << ':' << result.passed << ':'
        << result.dimension << ':' << result.solution_conversions << ':'
        << result.residual_products << ':' << result.residual_entries << ':'
        << result.image_products << ':' << result.image_entries << ':'
        << result.sign_comparisons << ':' << result.positive << ':'
        << result.negative << ':' << result.unresolved << ':'
        << al_r20_r37_dyadic_text(result.d) << ':'
        << al_r20_r37_dyadic_text(result.residual_infinity_numerator) << ':'
        << al_r20_r37_dyadic_text(result.image_infinity_numerator) << ':'
        << al_r20_r37_dyadic_text(result.coarse_error_numerator) << ':'
        << al_r20_r37_dyadic_text(result.error_denominator) << ':'
        << al_r20_r37_dyadic_text(result.minimum_resolved_separation) << ':'
        << result.solution_root << ':' << result.residual_root << ':'
        << result.image_root << ':' << result.sign_root;
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r20_r63u_classify(
    bool apparatus, bool certificate, bool wide, bool exported) {
    if (!apparatus) return "EXACT_RESIDUAL_IMAGE_APPARATUS_REJECTED";
    if (!certificate) return "EXACT_RESIDUAL_IMAGE_CERTIFICATE_REJECTED";
    if (!wide) return "RETAINED_WIDE_RESIDUAL_IMAGE_REJECTED";
    if (!exported) return "EXPORTED_FACTOR_RESIDUAL_IMAGE_REJECTED";
    return "TWO_LANE_EXACT_RESIDUAL_IMAGE_CANDIDATE";
}

struct ALR20R63UControls {
    bool exact = false;
    bool identity = false;
    bool contraction = false;
    bool sharper = false;
    bool unresolved = false;
    bool invalid = false;
    bool mutation = false;
    bool classifier = false;
    std::string root;
};

ALR20R63UControls al_r20_r63u_controls() {
    ALR20R63UControls result;
    const ALR20R37Dyadic zero = al_r20_r63p_control_dyadic(0);
    const ALR20R37Dyadic one = al_r20_r63p_control_dyadic(1);
    const ALR20R37Dyadic three_quarters =
        al_r20_r63p_control_dyadic(3, -2);
    const std::vector<ALR20R37Dyadic> identity{one, zero, zero, one};
    ALR20R63QExactMatrix matrix;
    matrix.exact = true;
    matrix.rows = 2U;
    matrix.columns = 2U;
    matrix.d = one;
    matrix.oracle_numerator = identity;
    const ALR20R63RDefect identity_defect = al_r20_r63r_defect(
        identity, identity, one, 2U);
    const std::vector<ALR20R37Dyadic> rhs{
        one, al_r20_r63p_control_dyadic(-1)};
    const ALR20R63UCertificate identity_certificate =
        al_r20_r63u_certificate(rhs, matrix, identity_defect, identity,
            {static_cast<Binary128>(1.0), static_cast<Binary128>(-1.0)});
    result.identity = identity_certificate.exact
        && identity_certificate.passed
        && identity_certificate.image_infinity_numerator.numerator == 0
        && identity_certificate.positive == 1U
        && identity_certificate.negative == 1U;
    const std::vector<ALR20R37Dyadic> scaled_inverse{
        three_quarters, zero, zero, three_quarters};
    const ALR20R63RDefect scaled_defect = al_r20_r63r_defect(
        scaled_inverse, identity, one, 2U);
    const ALR20R63UCertificate contraction = al_r20_r63u_certificate(
        rhs, matrix, scaled_defect, scaled_inverse,
        {static_cast<Binary128>(0.75), static_cast<Binary128>(-0.75)});
    result.contraction = contraction.exact && contraction.passed
        && contraction.image_infinity_numerator.numerator != 0;
    const ALR20R37Dyadic large = al_r20_r63p_control_dyadic(1024);
    const ALR20R37Dyadic small = al_r20_r63p_control_dyadic(1, -10);
    ALR20R63QExactMatrix diagonal_matrix = matrix;
    diagonal_matrix.oracle_numerator = {small, zero, zero, one};
    const std::vector<ALR20R37Dyadic> diagonal_inverse{
        large, zero, zero, one};
    const ALR20R63RDefect diagonal_defect = al_r20_r63r_defect(
        diagonal_inverse, diagonal_matrix.oracle_numerator, one, 2U);
    const ALR20R63UCertificate sharper = al_r20_r63u_certificate(
        {zero, one}, diagonal_matrix, diagonal_defect, diagonal_inverse,
        {static_cast<Binary128>(1.0), static_cast<Binary128>(0.5)});
    result.sharper = sharper.exact
        && al_r20_r37_compare_positive(
            sharper.image_infinity_numerator,
            sharper.coarse_error_numerator) < 0;
    const ALR20R63UCertificate unresolved = al_r20_r63u_certificate(
        {zero, al_r20_r63p_control_dyadic(-1)}, matrix,
        identity_defect, identity,
        {static_cast<Binary128>(0.0), static_cast<Binary128>(-1.0)});
    result.unresolved = unresolved.exact && !unresolved.passed
        && unresolved.unresolved == 1U;
    ALR20R63RDefect invalid_defect = identity_defect;
    invalid_defect.amplification_denominator = {};
    const ALR20R63UCertificate invalid = al_r20_r63u_certificate(
        rhs, matrix, invalid_defect, identity,
        {static_cast<Binary128>(1.0), static_cast<Binary128>(-1.0)});
    result.invalid = !invalid.exact;
    std::vector<ALR20R37Dyadic> mutated_image = contraction.image;
    mutated_image[0U] = al_r20_r37_add(mutated_image[0U], one);
    result.mutation = al_r20_r63q_vector_root(mutated_image)
        != contraction.image_root
        && identity_certificate.root != contraction.root;
    result.classifier =
        al_r20_r63u_classify(false, true, true, true)
            == "EXACT_RESIDUAL_IMAGE_APPARATUS_REJECTED"
        && al_r20_r63u_classify(true, false, true, true)
            == "EXACT_RESIDUAL_IMAGE_CERTIFICATE_REJECTED"
        && al_r20_r63u_classify(true, true, false, true)
            == "RETAINED_WIDE_RESIDUAL_IMAGE_REJECTED"
        && al_r20_r63u_classify(true, true, true, false)
            == "EXPORTED_FACTOR_RESIDUAL_IMAGE_REJECTED"
        && al_r20_r63u_classify(true, true, true, true)
            == "TWO_LANE_EXACT_RESIDUAL_IMAGE_CANDIDATE";
    result.exact = result.identity && result.contraction && result.sharper
        && result.unresolved && result.invalid && result.mutation
        && result.classifier;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + identity_certificate.root + ':' + contraction.root + ':'
        + sharper.root + ':' + unresolved.root + ':' + invalid.root + ':'
        + al_r20_r63q_vector_root(mutated_image));
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v5_exact_residual_image_controls() {
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
    const ALR20R63UControls controls = al_r20_r63u_controls();
    const ALR20R63SControls parent_controls = al_r20_r63s_controls();
    const bool controls_exact = profile_exact && controls.exact
        && parent_controls.exact
        && parent_controls.root == AL_R20_R63T_PARENT_CONTROLS_SHA256;
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
    auto parent_work = [](const ALR20R63SLane& lane) {
        return lane.completed_iterations == 8U
            && lane.rectangular_products == 9U
            && lane.preconditioner_solves == 8U
            && lane.certificates == 9U
            && lane.certificate_residual_products == 93636U;
    };
    const std::string parent_route = al_r20_r63s_classify(true, true,
        parent_wide.passed, parent_exported.passed);
    std::ostringstream parent_material;
    parent_material
        << "nextengine.nonlocal.nsr3b4e2d7r20r63s-common-operator-pcg.v1|"
        << AL_R20_R63S_PARENT_SEMANTIC_SHA256 << ':' << parent_route << ':'
        << parent_controls.root << ':' << common_audit.root << ':'
        << verifier_audit.root << ':' << exact_rhs_root << ':'
        << parent_wide.root << ':' << parent_exported.root;
    const std::string parent_semantic = sha256_hex(parent_material.str());
    const bool parent_r63s_exact = parent_r63r_exact
        && preconditioners_exact && starts_exact && rhs_conversions_exact
        && exact_rhs_root == AL_R20_R63T_EXACT_RHS_SHA256
        && parent_wide.exact && parent_exported.exact
        && parent_work(parent_wide) && parent_work(parent_exported)
        && !parent_wide.passed && !parent_exported.passed
        && parent_wide.root == AL_R20_R63U_WIDE_LANE_SHA256
        && parent_exported.root == AL_R20_R63U_EXPORTED_LANE_SHA256
        && parent_semantic == AL_R20_R63U_PARENT_SEMANTIC_SHA256;
    bool inverse_conversions_exact = true;
    std::size_t inverse_conversions = 0U;
    std::vector<ALR20R37Dyadic> exact_inverse(verifier_inverse.value.size());
    for (std::size_t index = 0U; index < verifier_inverse.value.size(); ++index) {
        const ALR20R37Conversion conversion =
            al_r20_r37_from_binary128(verifier_inverse.value[index]);
        inverse_conversions_exact = inverse_conversions_exact && conversion.exact;
        exact_inverse[index] = conversion.value;
        ++inverse_conversions;
    }
    const std::string inverse_root = al_r20_r63q_vector_root(exact_inverse);
    const ALR20R63UCertificate wide = parent_r63s_exact
        && inverse_conversions_exact
        ? al_r20_r63u_certificate(exact_rhs, common_audit.matrix,
            verifier_audit.defect, exact_inverse, parent_wide.solution)
        : ALR20R63UCertificate{};
    const ALR20R63UCertificate exported = parent_r63s_exact
        && inverse_conversions_exact
        ? al_r20_r63u_certificate(exact_rhs, common_audit.matrix,
            verifier_audit.defect, exact_inverse, parent_exported.solution)
        : ALR20R63UCertificate{};
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    auto certificate_work = [](const ALR20R63UCertificate& certificate) {
        return certificate.solution_conversions == 102U
            && certificate.residual_products == 10404U
            && certificate.residual_entries == 102U
            && certificate.image_products == 10404U
            && certificate.image_entries == 102U
            && certificate.sign_comparisons == 102U;
    };
    const bool work_exact = inverse_conversions == 10404U
        && certificate_work(wide) && certificate_work(exported);
    const bool final_signs_equal = wide.signs == exported.signs
        && wide.signs.size() == 102U;
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_r63s_exact && inverse_conversions_exact
        && inverse_root == verifier_audit.conversion_root
        && lifecycle_exact;
    const bool certificate_exact = wide.exact && exported.exact
        && work_exact && final_signs_equal;
    const std::string route = al_r20_r63u_classify(apparatus_exact,
        certificate_exact, wide.passed, exported.passed);
    const bool exact = apparatus_exact && certificate_exact;
    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63u-exact-residual-image.v1|"
        << AL_R20_R63U_CAUSAL_R63T_SHA256 << ':' << parent_semantic << ':'
        << route << ':' << controls.root << ':' << inverse_root << ':'
        << wide.root << ':' << exported.root;
    const std::string result_sha = sha256_hex(semantic.str());
    auto certificate_json = [](const ALR20R63UCertificate& certificate) {
        std::ostringstream stream;
        stream << "{\"exact\":" << (certificate.exact ? "true" : "false")
            << ",\"passed\":" << (certificate.passed ? "true" : "false")
            << ",\"residual_infinity\":\""
            << al_binary128_text(al_r20_r63p_fraction_projection(
                certificate.residual_infinity_numerator, certificate.d))
            << "\",\"image_infinity\":\""
            << al_binary128_text(al_r20_r63p_fraction_projection(
                certificate.image_infinity_numerator, certificate.d))
            << "\",\"image_error\":\""
            << al_binary128_text(al_r20_r63p_fraction_projection(
                certificate.image_infinity_numerator,
                certificate.error_denominator))
            << "\",\"coarse_error\":\""
            << al_binary128_text(al_r20_r63p_fraction_projection(
                certificate.coarse_error_numerator,
                certificate.error_denominator))
            << "\",\"positive\":" << certificate.positive
            << ",\"negative\":" << certificate.negative
            << ",\"unresolved\":" << certificate.unresolved
            << ",\"sign_root\":\"" << certificate.sign_root << "\""
            << ",\"image_root\":\"" << certificate.image_root << "\""
            << ",\"root\":\"" << certificate.root << "\"}";
        return stream.str();
    };
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63u_exact_residual_image.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"causal_r63t_sha256\":\""
        << AL_R20_R63U_CAUSAL_R63T_SHA256 << "\""
        << ",\"parent_r63s_sha256\":\"" << parent_semantic << "\""
        << ",\"controls_root\":\"" << controls.root << "\""
        << ",\"parent_r63s_exact\":"
        << (parent_r63s_exact ? "true" : "false")
        << ",\"inverse_conversions\":" << inverse_conversions
        << ",\"inverse_root\":\"" << inverse_root << "\""
        << ",\"wide\":" << certificate_json(wide)
        << ",\"exported\":" << certificate_json(exported)
        << ",\"final_signs_equal\":"
        << (final_signs_equal ? "true" : "false")
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"certificate_exact\":"
        << (certificate_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"exact_residual_products\":"
        << (wide.residual_products + exported.residual_products)
        << ",\"exact_image_products\":"
        << (wide.image_products + exported.image_products)
        << ",\"candidate_updates\":0,\"rounded_residual_uses\":0"
        << ",\"r63t_center_uses\":0,\"stored_dense_h_uses\":0"
        << ",\"stored_dense_x_uses\":0,\"legacy_sign_uses\":0"
        << ",\"factor_correction_solves\":0,\"sparse_constructions\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"gpu_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
