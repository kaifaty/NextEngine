namespace {

constexpr const char* AL_R20_R63T_PARENT_SEMANTIC_SHA256 =
    "97d992f18912ed9a223a17080820b72d4da54635534d96dba3dc9e5d71454e14";
constexpr const char* AL_R20_R63T_PARENT_CONTROLS_SHA256 =
    "48915ddf2007b39fe2450faaab14bafd366f25d687d22086165f473d21c71350";
constexpr const char* AL_R20_R63T_EXACT_RHS_SHA256 =
    "9db272a7000883d5f61884f05a5c86c1465d272b585ce6e9f359f91b99c5898e";
constexpr const char* AL_R20_R63T_WIDE_LANE_SHA256 =
    "2ec70f637f37e8927a86c071b24029e6d0bf4724968ba40609a46dbc23195376";
constexpr const char* AL_R20_R63T_EXPORTED_LANE_SHA256 =
    "5dcc65f74b8769d5a53cf5db2937f4c2013dc4b92f0333bb42a791006bc3c255";
constexpr const char* AL_R20_R63T_PARENT_SIGN_SHA256 =
    "4dc6f1bc8ba00c9bcfa22589b4b586324aa71cc9c811af04cd49d0b70f04150c";

struct ALR20R63TResidual {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t solution_conversions = 0U;
    std::size_t products = 0U;
    std::size_t entries = 0U;
    std::size_t projections = 0U;
    std::size_t projection_conversions = 0U;
    std::size_t projection_error_terms = 0U;
    ALR20R37Dyadic d;
    ALR20R37Dyadic residual_infinity_numerator;
    ALR20R37Dyadic projection_error_infinity_numerator;
    std::vector<ALR20R37Dyadic> numerators;
    std::vector<Binary128> projected;
    std::vector<ALR20R37Dyadic> projection_errors;
    std::string solution_root;
    std::string numerator_root;
    std::string projected_root;
    std::string projection_error_root;
    std::string root;
};

ALR20R63TResidual al_r20_r63t_residual(
    const std::vector<ALR20R37Dyadic>& rhs,
    const ALR20R63QExactMatrix& matrix,
    const std::vector<Binary128>& solution) {
    ALR20R63TResidual result;
    result.dimension = rhs.size();
    result.d = matrix.d;
    const std::size_t dimension = result.dimension;
    if (dimension == 0U || solution.size() != dimension
        || !matrix.exact || matrix.rows != dimension
        || matrix.oracle_numerator.size() != dimension * dimension
        || al_r20_r63p_sign(matrix.d) <= 0) return result;
    bool conversions_exact = true;
    std::vector<ALR20R37Dyadic> exact_solution(dimension);
    for (std::size_t index = 0U; index < dimension; ++index) {
        const ALR20R37Conversion conversion =
            al_r20_r37_from_binary128(solution[index]);
        conversions_exact = conversions_exact && conversion.exact;
        exact_solution[index] = conversion.value;
        ++result.solution_conversions;
    }
    result.numerators.resize(dimension);
    result.projected.resize(dimension);
    result.projection_errors.resize(dimension);
    bool projections_exact = true;
    for (std::size_t row = 0U; row < dimension; ++row) {
        ALR20R37Dyadic image;
        for (std::size_t column = 0U; column < dimension; ++column) {
            image = al_r20_r37_add(image, al_r20_r37_multiply(
                matrix.oracle_numerator[row * dimension + column],
                exact_solution[column]));
            ++result.products;
        }
        result.numerators[row] = al_r20_r63p_subtract(
            al_r20_r37_multiply(rhs[row], matrix.d), image);
        const ALR20R37Dyadic residual_absolute = al_r20_r37_absolute(
            result.numerators[row]);
        if (row == 0U || al_r20_r37_compare_positive(
                residual_absolute,
                result.residual_infinity_numerator) > 0)
            result.residual_infinity_numerator = residual_absolute;
        result.projected[row] = al_r20_r63p_fraction_projection(
            result.numerators[row], matrix.d);
        projections_exact = projections_exact
            && finiteq(result.projected[row]) != 0;
        ++result.projections;
        const ALR20R37Conversion projection_conversion =
            al_r20_r37_from_binary128(result.projected[row]);
        projections_exact = projections_exact && projection_conversion.exact;
        ++result.projection_conversions;
        result.projection_errors[row] = al_r20_r37_absolute(
            al_r20_r63p_subtract(
                al_r20_r37_multiply(projection_conversion.value, matrix.d),
                result.numerators[row]));
        if (row == 0U || al_r20_r37_compare_positive(
                result.projection_errors[row],
                result.projection_error_infinity_numerator) > 0)
            result.projection_error_infinity_numerator =
                result.projection_errors[row];
        ++result.projection_error_terms;
        ++result.entries;
    }
    result.solution_root = al_r20_r63q_vector_root(exact_solution);
    result.numerator_root = al_r20_r63q_vector_root(result.numerators);
    result.projected_root = al_r20_q_scalar_root(result.projected);
    result.projection_error_root =
        al_r20_r63q_vector_root(result.projection_errors);
    result.exact = conversions_exact && projections_exact
        && result.solution_conversions == dimension
        && result.products == dimension * dimension
        && result.entries == dimension && result.projections == dimension
        && result.projection_conversions == dimension
        && result.projection_error_terms == dimension;
    std::ostringstream material;
    material << result.exact << ':' << result.dimension << ':'
        << result.solution_conversions << ':' << result.products << ':'
        << result.entries << ':' << result.projections << ':'
        << result.projection_conversions << ':'
        << result.projection_error_terms << ':'
        << al_r20_r37_dyadic_text(result.d) << ':'
        << al_r20_r37_dyadic_text(result.residual_infinity_numerator) << ':'
        << al_r20_r37_dyadic_text(
            result.projection_error_infinity_numerator) << ':'
        << result.solution_root << ':' << result.numerator_root << ':'
        << result.projected_root << ':' << result.projection_error_root;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63TStep {
    bool exact = false;
    std::size_t iteration = 0U;
    ALR20R63TResidual residual;
    ALR20R63MProduct correction;
    std::size_t update_dots = 0U;
    Binary128 correction_infinity = static_cast<Binary128>(0.0);
    Binary128 correction_bound_infinity = static_cast<Binary128>(0.0);
    Binary128 update_infinity = static_cast<Binary128>(0.0);
    std::string update_root;
    ALR20R63SCertificate certificate;
    std::string root;
};

struct ALR20R63TLane {
    bool exact = false;
    bool finite = false;
    bool passed = false;
    bool has_pass = false;
    bool stable_after_pass = true;
    std::size_t dimension = 0U;
    std::size_t completed_iterations = 0U;
    std::size_t failure_iteration = 0U;
    std::size_t first_passing_iteration = 0U;
    std::size_t residual_records = 0U;
    std::size_t residual_solution_conversions = 0U;
    std::size_t residual_products = 0U;
    std::size_t residual_entries = 0U;
    std::size_t residual_projections = 0U;
    std::size_t residual_projection_conversions = 0U;
    std::size_t residual_projection_error_terms = 0U;
    std::size_t correction_products = 0U;
    std::size_t correction_dots = 0U;
    std::size_t correction_terms = 0U;
    std::size_t update_dots = 0U;
    std::size_t certificates = 0U;
    std::size_t certificate_solution_conversions = 0U;
    std::size_t certificate_residual_products = 0U;
    std::size_t certificate_residual_entries = 0U;
    std::size_t certificate_sign_comparisons = 0U;
    Binary128 minimum_error = HUGE_VALQ;
    std::vector<Binary128> solution;
    std::vector<ALR20R63TStep> steps;
    std::string failure_stage;
    std::string root;
};

ALR20R63TLane al_r20_r63t_lane(
    const std::vector<ALR20R37Dyadic>& exact_rhs,
    const ALR20R63QExactMatrix& matrix,
    const ALR20R63RDefect& inverse_defect,
    const std::vector<Binary128>& inverse,
    const std::vector<Binary128>& initial) {
    ALR20R63TLane result;
    result.dimension = exact_rhs.size();
    const std::size_t dimension = result.dimension;
    if (dimension == 0U || initial.size() != dimension
        || inverse.size() != dimension * dimension
        || !matrix.exact || matrix.rows != dimension
        || !inverse_defect.exact || !inverse_defect.left_contractive
        || !al_r20_r63p_equal(matrix.d, inverse_defect.d)) return result;
    result.solution = initial;
    bool exact = true;
    bool finite = true;
    for (std::size_t iteration = 1U; iteration <= 16U; ++iteration) {
        ALR20R63TStep step;
        step.iteration = iteration;
        step.residual = al_r20_r63t_residual(
            exact_rhs, matrix, result.solution);
        ++result.residual_records;
        result.residual_solution_conversions +=
            step.residual.solution_conversions;
        result.residual_products += step.residual.products;
        result.residual_entries += step.residual.entries;
        result.residual_projections += step.residual.projections;
        result.residual_projection_conversions +=
            step.residual.projection_conversions;
        result.residual_projection_error_terms +=
            step.residual.projection_error_terms;
        if (!step.residual.exact) {
            finite = false;
            exact = false;
            result.failure_iteration = iteration;
            result.failure_stage = "residual";
            break;
        }
        step.correction = al_r20_r63m_product(
            inverse, step.residual.projected);
        ++result.correction_products;
        result.correction_dots += step.correction.dots;
        result.correction_terms += step.correction.dots * dimension;
        if (!step.correction.exact) {
            finite = false;
            exact = false;
            result.failure_iteration = iteration;
            result.failure_stage = "correction";
            break;
        }
        std::vector<Binary128> updated(dimension);
        bool update_exact = true;
        std::ostringstream update_material;
        for (std::size_t row = 0U; row < dimension; ++row) {
            const ALR20R38Dot2 dot = al_r20_r38_dot2(
                {result.solution[row], step.correction.value[row]},
                {static_cast<Binary128>(1.0),
                    static_cast<Binary128>(1.0)});
            updated[row] = dot.value;
            update_exact = update_exact && dot.exact && dot.no_underflow
                && finiteq(dot.value) != 0;
            step.correction_infinity = fmaxq(step.correction_infinity,
                fabsq(step.correction.value[row]));
            step.correction_bound_infinity = fmaxq(
                step.correction_bound_infinity,
                step.correction.bound[row]);
            step.update_infinity = fmaxq(
                step.update_infinity, fabsq(dot.value));
            update_material << dot.root << ';';
            ++step.update_dots;
            ++result.update_dots;
        }
        step.update_root = sha256_hex(update_material.str());
        if (!update_exact) {
            finite = false;
            exact = false;
            result.failure_iteration = iteration;
            result.failure_stage = "update";
            break;
        }
        result.solution = std::move(updated);
        step.certificate = al_r20_r63s_certificate(
            exact_rhs, matrix, inverse_defect, result.solution);
        ++result.certificates;
        result.certificate_solution_conversions +=
            step.certificate.solution_conversions;
        result.certificate_residual_products +=
            step.certificate.residual_products;
        result.certificate_residual_entries +=
            step.certificate.residual_entries;
        result.certificate_sign_comparisons +=
            step.certificate.sign_comparisons;
        if (!step.certificate.exact) {
            finite = false;
            exact = false;
            result.failure_iteration = iteration;
            result.failure_stage = "certificate";
            break;
        }
        const Binary128 error = al_r20_r63p_fraction_projection(
            step.certificate.error_numerator,
            step.certificate.error_denominator);
        result.minimum_error = fminq(result.minimum_error, error);
        if (step.certificate.passed && !result.has_pass) {
            result.has_pass = true;
            result.first_passing_iteration = iteration;
        } else if (result.has_pass && !step.certificate.passed) {
            result.stable_after_pass = false;
        }
        step.exact = true;
        step.root = sha256_hex(std::to_string(step.exact) + ':'
            + std::to_string(step.iteration) + ':' + step.residual.root + ':'
            + step.correction.root + ':' + std::to_string(step.update_dots)
            + ':' + al_binary128_hex(step.correction_infinity) + ':'
            + al_binary128_hex(step.correction_bound_infinity) + ':'
            + al_binary128_hex(step.update_infinity) + ':'
            + step.update_root + ':' + step.certificate.root);
        result.steps.push_back(std::move(step));
        ++result.completed_iterations;
    }
    result.finite = finite;
    const bool complete = result.completed_iterations == 16U
        && result.steps.size() == 16U;
    result.passed = finite && complete && result.has_pass
        && result.stable_after_pass;
    result.exact = exact && ((finite && complete)
        || (!finite && !result.failure_stage.empty()));
    std::ostringstream material;
    material << result.exact << ':' << result.finite << ':'
        << result.passed << ':' << result.has_pass << ':'
        << result.stable_after_pass << ':' << result.dimension << ':'
        << result.completed_iterations << ':' << result.failure_iteration
        << ':' << result.first_passing_iteration << ':'
        << result.failure_stage << ':' << result.residual_records << ':'
        << result.residual_solution_conversions << ':'
        << result.residual_products << ':' << result.residual_entries << ':'
        << result.residual_projections << ':'
        << result.residual_projection_conversions << ':'
        << result.residual_projection_error_terms << ':'
        << result.correction_products << ':' << result.correction_dots << ':'
        << result.correction_terms << ':' << result.update_dots << ':'
        << result.certificates << ':'
        << result.certificate_solution_conversions << ':'
        << result.certificate_residual_products << ':'
        << result.certificate_residual_entries << ':'
        << result.certificate_sign_comparisons << ':'
        << al_binary128_hex(result.minimum_error) << ':'
        << al_r20_q_scalar_root(result.solution) << '|';
    for (const ALR20R63TStep& step : result.steps)
        material << step.root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r20_r63t_classify(
    bool apparatus, bool generation, bool wide, bool exported) {
    if (!apparatus) return "COMMON_RESIDUAL_RECOVERABILITY_APPARATUS_REJECTED";
    if (!generation) return "COMMON_RESIDUAL_CORRECTION_GENERATION_REJECTED";
    if (!wide) return "RETAINED_WIDE_COMMON_RESIDUAL_REFINEMENT_REJECTED";
    if (!exported)
        return "EXPORTED_FACTOR_COMMON_RESIDUAL_REFINEMENT_REJECTED";
    return "COMMON_RESIDUAL_RECOVERABILITY_CANDIDATE";
}

struct ALR20R63TControls {
    bool exact = false;
    bool identity = false;
    bool contraction = false;
    bool orientation = false;
    bool invalid = false;
    bool mutation = false;
    bool classifier = false;
    std::string root;
};

ALR20R63TControls al_r20_r63t_controls() {
    ALR20R63TControls result;
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
    const std::vector<ALR20R37Dyadic> rhs{
        one, al_r20_r63p_control_dyadic(-1)};
    const ALR20R63RDefect identity_defect = al_r20_r63r_defect(
        identity, identity, one, 2U);
    const std::vector<Binary128> identity_q{
        static_cast<Binary128>(1.0), static_cast<Binary128>(0.0),
        static_cast<Binary128>(0.0), static_cast<Binary128>(1.0)};
    const ALR20R63TLane identity_lane = al_r20_r63t_lane(
        rhs, matrix, identity_defect, identity_q,
        {static_cast<Binary128>(0.5), static_cast<Binary128>(-0.5)});
    result.identity = identity_lane.exact && identity_lane.passed
        && identity_lane.first_passing_iteration == 1U
        && identity_lane.steps.size() == 16U
        && identity_lane.steps.front().certificate.positive == 1U
        && identity_lane.steps.front().certificate.negative == 1U;
    const std::vector<ALR20R37Dyadic> scaled_inverse{
        three_quarters, zero, zero, three_quarters};
    const ALR20R63RDefect scaled_defect = al_r20_r63r_defect(
        scaled_inverse, identity, one, 2U);
    const std::vector<Binary128> scaled_q{
        static_cast<Binary128>(0.75), static_cast<Binary128>(0.0),
        static_cast<Binary128>(0.0), static_cast<Binary128>(0.75)};
    const ALR20R63TLane contraction_lane = al_r20_r63t_lane(
        rhs, matrix, scaled_defect, scaled_q,
        {static_cast<Binary128>(0.0), static_cast<Binary128>(0.0)});
    result.contraction = contraction_lane.exact && contraction_lane.passed
        && contraction_lane.first_passing_iteration == 1U
        && contraction_lane.steps.front().correction_infinity
            == static_cast<Binary128>(0.75)
        && contraction_lane.steps[1U].correction_infinity
            < contraction_lane.steps.front().correction_infinity;
    ALR20R63TResidual oriented = al_r20_r63t_residual(
        rhs, matrix,
        {static_cast<Binary128>(0.5), static_cast<Binary128>(-0.5)});
    std::vector<Binary128> negated = oriented.projected;
    for (Binary128& value : negated) value = -value;
    const ALR20R63MProduct correct = al_r20_r63m_product(
        identity_q, oriented.projected);
    const ALR20R63MProduct wrong = al_r20_r63m_product(identity_q, negated);
    result.orientation = oriented.exact && correct.exact && wrong.exact
        && correct.root != wrong.root && correct.value[0U] > 0
        && wrong.value[0U] < 0;
    ALR20R63QExactMatrix invalid_matrix = matrix;
    invalid_matrix.d = {};
    const ALR20R63TResidual invalid_residual = al_r20_r63t_residual(
        rhs, invalid_matrix,
        {static_cast<Binary128>(0.0), static_cast<Binary128>(0.0)});
    const std::vector<ALR20R37Dyadic> bad_inverse{
        al_r20_r63p_control_dyadic(2), zero, zero,
        al_r20_r63p_control_dyadic(2)};
    const ALR20R63RDefect bad_defect = al_r20_r63r_defect(
        bad_inverse, identity, one, 2U);
    const ALR20R63TLane invalid_lane = al_r20_r63t_lane(
        rhs, matrix, bad_defect, identity_q,
        {static_cast<Binary128>(0.0), static_cast<Binary128>(0.0)});
    result.invalid = !invalid_residual.exact && !invalid_lane.exact;
    ALR20R63TResidual mutated = oriented;
    mutated.projected[0U] = -mutated.projected[0U];
    const std::string mutated_projection_root =
        al_r20_q_scalar_root(mutated.projected);
    const std::string first_one = sha256_hex(
        std::string("1:1:") + identity_lane.root);
    const std::string first_two = sha256_hex(
        std::string("1:2:") + identity_lane.root);
    result.mutation = mutated_projection_root != oriented.projected_root
        && identity_lane.root != contraction_lane.root
        && first_one != first_two;
    result.classifier =
        al_r20_r63t_classify(false, true, true, true)
            == "COMMON_RESIDUAL_RECOVERABILITY_APPARATUS_REJECTED"
        && al_r20_r63t_classify(true, false, true, true)
            == "COMMON_RESIDUAL_CORRECTION_GENERATION_REJECTED"
        && al_r20_r63t_classify(true, true, false, true)
            == "RETAINED_WIDE_COMMON_RESIDUAL_REFINEMENT_REJECTED"
        && al_r20_r63t_classify(true, true, true, false)
            == "EXPORTED_FACTOR_COMMON_RESIDUAL_REFINEMENT_REJECTED"
        && al_r20_r63t_classify(true, true, true, true)
            == "COMMON_RESIDUAL_RECOVERABILITY_CANDIDATE";
    result.exact = result.identity && result.contraction
        && result.orientation && result.invalid && result.mutation
        && result.classifier;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + identity_lane.root + ':' + contraction_lane.root + ':'
        + oriented.root + ':' + invalid_residual.root + ':'
        + invalid_lane.root + ':' + mutated_projection_root + ':'
        + first_one + ':' + first_two);
    return result;
}

} // namespace

SplitBoundaryReport
run_al_generalization_v5_common_residual_recoverability_controls() {
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
    const ALR20R63TControls controls = al_r20_r63t_controls();
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
        && common_audit.matrix.oracle_root == AL_R20_R63R_ORACLE_SHA256
        && original_factor.root == AL_R20_R63R_FACTOR_SHA256
        && verifier_inverse.root == AL_R20_R63S_INVERSE_SHA256
        && verifier_audit.defect.root == AL_R20_R63S_DEFECT_SHA256
        && verifier_audit.root == AL_R20_R63S_INVERSE_AUDIT_SHA256
        && verifier_audit.defect.left_contractive
        && verifier_audit.defect.right_contractive;
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
    std::size_t rhs_conversions = 0U;
    std::vector<ALR20R37Dyadic> exact_rhs(factor_capture.rhs.size());
    for (std::size_t index = 0U; index < factor_capture.rhs.size(); ++index) {
        const ALR20R37Conversion conversion =
            al_r20_r37_from_binary128(factor_capture.rhs[index]);
        rhs_conversions_exact = rhs_conversions_exact && conversion.exact;
        exact_rhs[index] = conversion.value;
        ++rhs_conversions;
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
    auto parent_lane_work_exact = [](const ALR20R63SLane& lane) {
        return lane.completed_iterations == 8U
            && lane.rectangular_products == 9U
            && lane.direct_inner_dots == 2835U
            && lane.direct_outer_dots == 918U
            && lane.direct_inner_terms == 289170U
            && lane.direct_outer_terms == 289170U
            && lane.direct_scale_products == 918U
            && lane.initial_residual_dots == 102U
            && lane.preconditioner_solves == 8U
            && lane.forward_terms == 41208U
            && lane.backward_terms == 41208U && lane.divisions == 1632U
            && lane.rho_dots == 8U && lane.denominator_dots == 8U
            && lane.solution_update_dots == 816U
            && lane.residual_update_dots == 816U
            && lane.direction_update_dots == 714U
            && lane.certificates == 9U
            && lane.certificate_solution_conversions == 918U
            && lane.certificate_residual_products == 93636U
            && lane.certificate_residual_entries == 918U
            && lane.certificate_sign_comparisons == 918U;
    };
    const bool parent_work_exact = parent_lane_work_exact(parent_wide)
        && parent_lane_work_exact(parent_exported);
    const bool parent_finals_exact = parent_wide.steps.size() == 8U
        && parent_exported.steps.size() == 8U
        && parent_wide.steps.back().certificate.sign_root
            == AL_R20_R63T_PARENT_SIGN_SHA256
        && parent_exported.steps.back().certificate.sign_root
            == AL_R20_R63T_PARENT_SIGN_SHA256
        && parent_wide.steps.back().certificate.positive == 12U
        && parent_wide.steps.back().certificate.negative == 24U
        && parent_wide.steps.back().certificate.unresolved == 66U
        && parent_exported.steps.back().certificate.positive == 12U
        && parent_exported.steps.back().certificate.negative == 24U
        && parent_exported.steps.back().certificate.unresolved == 66U;
    const bool parent_r63s_apparatus = parent_r63r_exact
        && preconditioners_exact && starts_exact && rhs_conversions_exact
        && rhs_conversions == 102U
        && exact_rhs_root == AL_R20_R63T_EXACT_RHS_SHA256;
    const bool parent_r63s_certificate = parent_wide.exact
        && parent_exported.exact && parent_work_exact && parent_finals_exact;
    const std::string parent_route = al_r20_r63s_classify(
        parent_r63s_apparatus, parent_r63s_certificate,
        parent_wide.passed, parent_exported.passed);
    std::ostringstream parent_semantic_material;
    parent_semantic_material
        << "nextengine.nonlocal.nsr3b4e2d7r20r63s-common-operator-pcg.v1|"
        << AL_R20_R63S_PARENT_SEMANTIC_SHA256 << ':' << parent_route << ':'
        << parent_controls.root << ':' << common_audit.root << ':'
        << verifier_audit.root << ':' << exact_rhs_root << ':'
        << parent_wide.root << ':' << parent_exported.root;
    const std::string parent_semantic =
        sha256_hex(parent_semantic_material.str());
    const bool parent_r63s_exact = parent_r63s_apparatus
        && parent_r63s_certificate
        && parent_route == "RETAINED_WIDE_COMMON_OPERATOR_PCG_REJECTED"
        && parent_wide.root == AL_R20_R63T_WIDE_LANE_SHA256
        && parent_exported.root == AL_R20_R63T_EXPORTED_LANE_SHA256
        && parent_semantic == AL_R20_R63T_PARENT_SEMANTIC_SHA256;
    const ALR20R63TLane wide = parent_r63s_exact
        ? al_r20_r63t_lane(exact_rhs, common_audit.matrix,
            verifier_audit.defect, verifier_inverse.value,
            parent_wide.solution)
        : ALR20R63TLane{};
    const ALR20R63TLane exported = parent_r63s_exact
        ? al_r20_r63t_lane(exact_rhs, common_audit.matrix,
            verifier_audit.defect, verifier_inverse.value,
            parent_exported.solution)
        : ALR20R63TLane{};
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    auto lane_work_exact = [](const ALR20R63TLane& lane) {
        return lane.completed_iterations == 16U
            && lane.residual_records == 16U
            && lane.residual_solution_conversions == 1632U
            && lane.residual_products == 166464U
            && lane.residual_entries == 1632U
            && lane.residual_projections == 1632U
            && lane.residual_projection_conversions == 1632U
            && lane.residual_projection_error_terms == 1632U
            && lane.correction_products == 16U
            && lane.correction_dots == 1632U
            && lane.correction_terms == 166464U
            && lane.update_dots == 1632U
            && lane.certificates == 16U
            && lane.certificate_solution_conversions == 1632U
            && lane.certificate_residual_products == 166464U
            && lane.certificate_residual_entries == 1632U
            && lane.certificate_sign_comparisons == 1632U;
    };
    const bool work_exact = lane_work_exact(wide)
        && lane_work_exact(exported);
    const bool final_signs_equal = wide.steps.size() == 16U
        && exported.steps.size() == 16U
        && wide.steps.back().certificate.signs
            == exported.steps.back().certificate.signs;
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_r63s_exact && lifecycle_exact;
    const bool generation_exact = wide.exact && exported.exact
        && work_exact && final_signs_equal;
    const std::string route = al_r20_r63t_classify(apparatus_exact,
        generation_exact, wide.passed, exported.passed);
    const bool exact = apparatus_exact && generation_exact;
    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63t-common-residual-recoverability.v1|"
        << AL_R20_R63T_PARENT_SEMANTIC_SHA256 << ':' << route << ':'
        << controls.root << ':' << parent_semantic << ':'
        << wide.root << ':' << exported.root;
    const std::string result_sha = sha256_hex(semantic.str());
    auto certificate_json = [](const ALR20R63SCertificate& certificate) {
        std::ostringstream stream;
        stream << "{\"passed\":" << (certificate.passed ? "true" : "false")
            << ",\"residual_infinity\":\""
            << al_binary128_text(al_r20_r63p_fraction_projection(
                certificate.residual_infinity_numerator,
                certificate.residual_denominator))
            << "\",\"error\":\""
            << al_binary128_text(al_r20_r63p_fraction_projection(
                certificate.error_numerator,
                certificate.error_denominator))
            << "\",\"positive\":" << certificate.positive
            << ",\"negative\":" << certificate.negative
            << ",\"unresolved\":" << certificate.unresolved
            << ",\"sign_root\":\"" << certificate.sign_root << "\""
            << ",\"root\":\"" << certificate.root << "\"}";
        return stream.str();
    };
    auto lane_json = [&certificate_json](
                         const char* name, const ALR20R63TLane& lane) {
        std::ostringstream stream;
        stream << "\"" << name << "\":{\"exact\":"
            << (lane.exact ? "true" : "false")
            << ",\"finite\":" << (lane.finite ? "true" : "false")
            << ",\"passed\":" << (lane.passed ? "true" : "false")
            << ",\"has_pass\":" << (lane.has_pass ? "true" : "false")
            << ",\"stable_after_pass\":"
            << (lane.stable_after_pass ? "true" : "false")
            << ",\"completed_iterations\":" << lane.completed_iterations
            << ",\"first_passing_iteration\":"
            << lane.first_passing_iteration
            << ",\"failure_iteration\":" << lane.failure_iteration
            << ",\"failure_stage\":\"" << lane.failure_stage << "\""
            << ",\"minimum_error\":\""
            << al_binary128_text(lane.minimum_error)
            << "\",\"steps\":[";
        for (std::size_t index = 0U; index < lane.steps.size(); ++index) {
            if (index != 0U) stream << ',';
            const ALR20R63TStep& step = lane.steps[index];
            stream << "{\"iteration\":" << step.iteration
                << ",\"correction_residual_infinity\":\""
                << al_binary128_text(al_r20_r63p_fraction_projection(
                    step.residual.residual_infinity_numerator,
                    step.residual.d))
                << "\",\"residual_projection_error_infinity\":\""
                << al_binary128_text(al_r20_r63p_fraction_projection(
                    step.residual.projection_error_infinity_numerator,
                    step.residual.d))
                << "\",\"correction_infinity\":\""
                << al_binary128_text(step.correction_infinity)
                << "\",\"correction_bound_infinity\":\""
                << al_binary128_text(step.correction_bound_infinity)
                << "\",\"certificate\":"
                << certificate_json(step.certificate)
                << ",\"root\":\"" << step.root << "\"}";
        }
        stream << "],\"root\":\"" << lane.root << "\"}";
        return stream.str();
    };
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63t_common_residual_recoverability.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63s_sha256\":\"" << parent_semantic << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"controls_root\":\"" << controls.root << "\""
        << ",\"parent_r63s_exact\":"
        << (parent_r63s_exact ? "true" : "false")
        << ",\"lanes\":{" << lane_json("wide", wide) << ','
        << lane_json("exported", exported) << '}'
        << ",\"final_signs_equal\":"
        << (final_signs_equal ? "true" : "false")
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"generation_exact\":"
        << (generation_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"correction_residual_records\":"
        << (wide.residual_records + exported.residual_records)
        << ",\"dense_verifier_correction_uses\":"
        << (wide.correction_products + exported.correction_products)
        << ",\"correction_dot2_terms\":"
        << (wide.correction_terms + exported.correction_terms)
        << ",\"new_exact_common_products\":"
        << (wide.residual_products + exported.residual_products
            + wide.certificate_residual_products
            + exported.certificate_residual_products)
        << ",\"stored_dense_h_uses\":0,\"stored_dense_x_uses\":0"
        << ",\"legacy_sign_uses\":0,\"factor_correction_solves\":0"
        << ",\"sparse_constructions\":0,\"rank_actions\":0"
        << ",\"row_drops\":0,\"regularizations\":0"
        << ",\"replacement_applies\":0,\"following_transitions\":0"
        << ",\"state_updates\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"gpu_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
