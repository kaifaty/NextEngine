namespace {

constexpr const char* AL_R20_R63L_PARENT_SEMANTIC_SHA256 =
    "61c739a2f7fe417de69cc596d925d6a7ec417360b25118e88875eb8c7b474ade";
constexpr const char* AL_R20_R63L_WIDE_PARENT_SHA256 =
    "c7301c0cf5b78d0a18c12a6b0e55f2ab268b1bc9619ee7a431305d21b5d156e7";
constexpr const char* AL_R20_R63L_EXPORT_PARENT_SHA256 =
    "16bcae3027caa2fd10a8c6244d904e3c0bb793b622ecc9227749c99f4b512eaa";
constexpr const char* AL_R20_R63L_STRICT_PARENT_SHA256 =
    "49954a7ea4289bd3e01f42863839158a7c823f54ccb565a2c709b7f2997933a7";

struct ALR20R63LStep {
    bool exact = false;
    std::size_t iteration = 0U;
    ALR20R63ISignCertificate certificate;
    std::string root;
};

struct ALR20R63LContinuation {
    bool exact = false;
    bool finite = false;
    bool passed = false;
    bool monotone_error = false;
    std::size_t dimension = 0U;
    std::size_t completed_iterations = 0U;
    std::size_t failure_iteration = 0U;
    std::size_t residual_dots = 0U;
    std::size_t update_dots = 0U;
    std::size_t forward_terms = 0U;
    std::size_t backward_terms = 0U;
    std::size_t divisions = 0U;
    std::size_t certificate_residual_dots = 0U;
    std::size_t certificate_image_dots = 0U;
    std::size_t first_passing_iteration = 0U;
    Binary128 minimum_error = HUGE_VALQ;
    std::vector<Binary128> solution;
    std::vector<ALR20R63LStep> steps;
    std::string failure_stage;
    std::string root;
};

ALR20R63LContinuation al_r20_r63l_continue(
    const std::vector<Binary128>& upper,
    const std::vector<std::size_t>& permutation,
    Binary128 inverse_scale,
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& inverse,
    const std::vector<Binary128>& rhs,
    const ALR20R63KLane& parent,
    const std::vector<int>& reference_signs,
    const std::vector<Binary128>& reference_solution) {
    ALR20R63LContinuation result;
    result.dimension = rhs.size();
    const std::size_t dimension = result.dimension;
    if (dimension == 0U || upper.size() != dimension * dimension
        || !al_r20_r63g_permutation_exact(permutation, dimension)
        || matrix.size() != dimension * dimension
        || inverse.size() != dimension * dimension
        || !parent.exact || !parent.finite || parent.passed
        || parent.completed_iterations != 16U
        || parent.solution.size() != dimension
        || parent.checkpoints.size() != AL_R20_R63K_DEPTHS.size()
        || reference_signs.size() != dimension
        || reference_solution.size() != dimension) return result;
    result.solution = parent.solution;
    bool exact = true;
    bool finite = true;
    bool monotone = true;
    Binary128 previous_error = parent.checkpoints.back().certificate.error;
    for (std::size_t iteration = 17U; iteration <= 32U; ++iteration) {
        const ALR20R63KResidual residual = al_r20_r63k_residual(
            matrix, rhs, result.solution);
        result.residual_dots += residual.dots;
        if (!residual.exact) {
            finite = false;
            result.failure_iteration = iteration;
            result.failure_stage = "residual";
            break;
        }
        const ALR20R63IQSolve correction = al_r20_r63i_q_solve(
            upper, permutation, residual.value, inverse_scale);
        result.forward_terms += correction.forward_terms;
        result.backward_terms += correction.backward_terms;
        result.divisions += correction.divisions;
        if (!correction.exact) {
            finite = false;
            result.failure_iteration = iteration;
            result.failure_stage = "factor_solve";
            break;
        }
        std::vector<Binary128> updated(dimension);
        std::ostringstream update_material;
        bool update_exact = true;
        for (std::size_t row = 0U; row < dimension; ++row) {
            const std::vector<Binary128> left{
                result.solution[row], correction.solution[row]};
            const std::vector<Binary128> right{
                static_cast<Binary128>(1.0),
                static_cast<Binary128>(1.0)};
            const ALR20R38Dot2 dot = al_r20_r38_dot2(left, right);
            updated[row] = dot.value;
            update_exact = update_exact && dot.exact
                && dot.no_underflow && finiteq(dot.value) != 0
                && finiteq(dot.bound) != 0;
            update_material << dot.root << ';';
            ++result.update_dots;
        }
        if (!update_exact) {
            finite = false;
            result.failure_iteration = iteration;
            result.failure_stage = "update";
            break;
        }
        result.solution = std::move(updated);
        ++result.completed_iterations;
        ALR20R63LStep step;
        step.iteration = iteration;
        step.certificate = al_r20_r63i_sign_certificate(
            matrix, inverse, rhs, result.solution, reference_signs,
            parent.checkpoints.back().certificate.rho_bound,
            reference_solution);
        step.exact = step.certificate.exact;
        result.certificate_residual_dots += step.certificate.residual_dots;
        result.certificate_image_dots += step.certificate.image_dots;
        monotone = monotone && step.certificate.error < previous_error;
        previous_error = step.certificate.error;
        result.minimum_error = fminq(
            result.minimum_error, step.certificate.error);
        if (step.certificate.passed
            && result.first_passing_iteration == 0U)
            result.first_passing_iteration = iteration;
        step.root = sha256_hex(std::to_string(step.exact) + ':'
            + std::to_string(step.iteration) + ':' + residual.root + ':'
            + correction.root + ':' + sha256_hex(update_material.str()) + ':'
            + al_r20_q_scalar_root(result.solution) + ':'
            + step.certificate.root);
        exact = exact && step.exact;
        result.steps.push_back(std::move(step));
    }
    result.finite = finite;
    result.monotone_error = finite && monotone;
    const bool complete = result.completed_iterations == 16U;
    result.passed = finite && complete
        && result.first_passing_iteration != 0U;
    result.exact = exact && ((finite && complete && result.steps.size() == 16U)
        || (!finite && result.failure_iteration >= 17U
            && result.failure_iteration <= 32U
            && !result.failure_stage.empty()));
    std::ostringstream material;
    material << result.exact << ':' << result.finite << ':'
        << result.passed << ':' << result.monotone_error << ':'
        << result.dimension << ':' << result.completed_iterations << ':'
        << result.failure_iteration << ':' << result.failure_stage << ':'
        << result.residual_dots << ':' << result.update_dots << ':'
        << result.forward_terms << ':' << result.backward_terms << ':'
        << result.divisions << ':' << result.certificate_residual_dots << ':'
        << result.certificate_image_dots << ':'
        << result.first_passing_iteration << ':'
        << al_binary128_hex(result.minimum_error) << ':'
        << al_r20_q_scalar_root(result.solution) << '|';
    for (const ALR20R63LStep& step : result.steps)
        material << step.root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r20_r63l_classify(
    bool apparatus, bool wide, bool exported) {
    if (!apparatus) return "WIDE_REFINEMENT_FRONTIER_APPARATUS_REJECTED";
    if (!wide) return "RETAINED_WIDE_REFINEMENT_FRONTIER_EXHAUSTED";
    if (!exported)
        return "EXPORTED_FACTOR_WIDE_REFINEMENT_FRONTIER_EXHAUSTED";
    return "EXPORTED_FACTOR_WIDE_STANDARD_REFINEMENT_CANDIDATE";
}

struct ALR20R63LControls {
    bool exact = false;
    bool continuation = false;
    bool passed_parent = false;
    bool failure = false;
    bool mutation = false;
    bool classifier = false;
    std::string root;
};

ALR20R63LControls al_r20_r63l_controls() {
    ALR20R63LControls result;
    const std::vector<Binary128> matrix{static_cast<Binary128>(1.0)};
    const std::vector<Binary128> inverse{static_cast<Binary128>(0.75)};
    const std::vector<Binary128> rhs{static_cast<Binary128>(1.0)};
    const std::vector<Binary128> reference{static_cast<Binary128>(1.0)};
    const std::vector<int> signs{1};
    const std::vector<std::size_t> permutation{0U};
    const std::vector<Binary128> upper{sqrtq(
        static_cast<Binary128>(4.0) / static_cast<Binary128>(3.0))};
    ALR20R63KLane parent;
    parent.exact = true;
    parent.finite = true;
    parent.passed = false;
    parent.completed_iterations = 16U;
    parent.solution = {static_cast<Binary128>(0.0)};
    for (std::size_t depth : AL_R20_R63K_DEPTHS) {
        ALR20R63KCheckpoint checkpoint;
        checkpoint.iteration = depth;
        checkpoint.exact = true;
        checkpoint.certificate = al_r20_r63i_sign_certificate(
            matrix, inverse, rhs, parent.solution, signs,
            static_cast<Binary128>(0.25), reference);
        parent.checkpoints.push_back(std::move(checkpoint));
    }
    const ALR20R63LContinuation lane = al_r20_r63l_continue(
        upper, permutation, static_cast<Binary128>(1.0), matrix, inverse,
        rhs, parent, signs, reference);
    result.continuation = lane.exact && lane.finite && lane.passed
        && lane.first_passing_iteration == 17U && lane.steps.size() == 16U;
    ALR20R63KLane passed_parent = parent;
    passed_parent.passed = true;
    result.passed_parent = !al_r20_r63l_continue(
        upper, permutation, static_cast<Binary128>(1.0), matrix, inverse,
        rhs, passed_parent, signs, reference).exact;
    const ALR20R63LContinuation failed = al_r20_r63l_continue(
        {static_cast<Binary128>(0.0)}, permutation,
        static_cast<Binary128>(1.0), matrix, inverse, rhs, parent, signs,
        reference);
    result.failure = failed.exact && !failed.finite && !failed.passed
        && failed.failure_iteration == 17U
        && failed.failure_stage == "factor_solve";
    std::vector<Binary128> mutated = lane.solution;
    mutated[0U] = nextafterq(mutated[0U], HUGE_VALQ);
    result.mutation = al_r20_q_scalar_root(mutated)
        != al_r20_q_scalar_root(lane.solution);
    result.classifier =
        al_r20_r63l_classify(false, true, true)
            == "WIDE_REFINEMENT_FRONTIER_APPARATUS_REJECTED"
        && al_r20_r63l_classify(true, false, true)
            == "RETAINED_WIDE_REFINEMENT_FRONTIER_EXHAUSTED"
        && al_r20_r63l_classify(true, true, false)
            == "EXPORTED_FACTOR_WIDE_REFINEMENT_FRONTIER_EXHAUSTED"
        && al_r20_r63l_classify(true, true, true)
            == "EXPORTED_FACTOR_WIDE_STANDARD_REFINEMENT_CANDIDATE";
    result.exact = result.continuation && result.passed_parent
        && result.failure && result.mutation && result.classifier;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + lane.root + ':' + failed.root + ':'
        + al_r20_q_scalar_root(mutated));
    return result;
}

} // namespace

SplitBoundaryReport
run_al_generalization_v5_wide_refinement_frontier_controls() {
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
    const ALR20R63LControls controls = al_r20_r63l_controls();
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
        && al_r20_q_scalar_root(factor_capture.inverse)
            == AL_R20_R63I_INVERSE_SHA256
        && al_r20_q_scalar_root(factor_capture.rhs)
            == AL_R20_R63I_RHS_SHA256
        && al_r20_q_scalar_root(factor_capture.solution)
            == AL_R20_R63I_SOLUTION_SHA256
        && factor_capture.rhs.size() == 102U
        && factor_capture.solution.size() == 102U
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
    const std::vector<std::size_t> permutation = full_profile.exact
        ? full_profile.binary128.permutation
        : std::vector<std::size_t>{};
    const bool permutation_exact = al_r20_r63g_permutation_exact(
        permutation, tangent.rows);
    std::vector<Binary128> transposed;
    std::vector<double> permuted_d;
    if (tangent.exact && permutation_exact) {
        transposed.resize(tangent.columns * tangent.rows);
        permuted_d.resize(transposed.size());
        for (std::size_t scalar = 0U; scalar < tangent.columns; ++scalar)
            for (std::size_t column = 0U; column < tangent.rows; ++column)
                transposed[scalar * tangent.rows + column] =
                    tangent.values[column * tangent.columns + scalar];
        for (std::size_t scalar = 0U; scalar < tangent.columns; ++scalar)
            for (std::size_t column = 0U; column < tangent.rows; ++column)
                permuted_d[scalar * tangent.rows + column] =
                    static_cast<double>(transposed[
                        scalar * tangent.rows + permutation[column]]);
    }
    const ALR20R63GDFactor parent_d_factor = !permuted_d.empty()
        ? al_r20_r63g_d_factor(
            permuted_d, tangent.columns, tangent.rows)
        : ALR20R63GDFactor{};
    const std::vector<Binary128> stored_permuted =
        al_r20_r63g_q_from_double(parent_d_factor.input);
    const ALR20R63GQFactor wide_factor = !stored_permuted.empty()
        ? al_r20_r63g_q_factor(
            stored_permuted, tangent.columns, tangent.rows)
        : ALR20R63GQFactor{};
    std::vector<double> exported_r_d;
    if (wide_factor.exact) {
        exported_r_d.reserve(wide_factor.r.size());
        for (Binary128 value : wide_factor.r)
            exported_r_d.push_back(static_cast<double>(value));
    }
    const std::vector<Binary128> exported_r =
        al_r20_r63g_q_from_double(exported_r_d);
    const bool factor_exact = capture_exact && grams.exact
        && full_profile.exact && tangent.exact && permutation_exact
        && parent_d_factor.exact && wide_factor.exact
        && wide_factor.root == AL_R20_R63I_WIDE_FACTOR_SHA256
        && al_r20_double_root(exported_r_d)
            == AL_R20_R63I_EXPORT_R_SHA256;

    const ALR20R42PracticalCertificate reference_certificate = capture_exact
        ? al_r20_r42_practical_certificate(factor_capture.matrix,
            factor_capture.inverse, factor_capture.rhs,
            factor_capture.solution, 4U, false)
        : ALR20R42PracticalCertificate{};
    std::vector<int> reference_signs(102U, 0);
    bool reference_signs_exact = reference_certificate.passed
        && reference_certificate.solution.size() == 102U;
    for (std::size_t index = 0U;
         index < reference_signs.size() && reference_signs_exact; ++index) {
        if (reference_certificate.solution[index]
                - reference_certificate.error
            > static_cast<Binary128>(0.0))
            reference_signs[index] = 1;
        else if (reference_certificate.solution[index]
                + reference_certificate.error
            < static_cast<Binary128>(0.0))
            reference_signs[index] = -1;
        else
            reference_signs_exact = false;
    }
    const bool reference_exact = reference_signs_exact
        && reference_certificate.exact && reference_certificate.passed
        && reference_certificate.positive == 24U
        && reference_certificate.negative == 78U
        && reference_certificate.unresolved == 0U
        && reference_certificate.root == AL_R20_R63J_REFERENCE_SHA256;
    const Binary128 inverse_scale = static_cast<Binary128>(1.0)
        + factor_capture.projector.eta;
    const ALR20R63IQSolve wide_solve = factor_exact
        ? al_r20_r63i_q_solve(wide_factor.r, permutation,
            factor_capture.rhs, inverse_scale)
        : ALR20R63IQSolve{};
    const ALR20R63IQSolve export_wide_solve = factor_exact
        ? al_r20_r63i_q_solve(exported_r, permutation,
            factor_capture.rhs, inverse_scale)
        : ALR20R63IQSolve{};
    const ALR20R63IDSolve strict_solve = factor_exact
        ? al_r20_r63i_d_solve(exported_r_d, permutation,
            factor_capture.rhs, inverse_scale)
        : ALR20R63IDSolve{};
    const std::vector<Binary128> strict_solution =
        al_r20_r63g_q_from_double(strict_solve.solution);
    const ALR20R63ISignCertificate wide_baseline =
        wide_solve.exact && reference_exact
        ? al_r20_r63i_sign_certificate(factor_capture.matrix,
            factor_capture.inverse, factor_capture.rhs,
            wide_solve.solution, reference_signs,
            reference_certificate.rho_bound,
            reference_certificate.solution)
        : ALR20R63ISignCertificate{};
    const ALR20R63ISignCertificate export_wide_baseline =
        export_wide_solve.exact && reference_exact
        ? al_r20_r63i_sign_certificate(factor_capture.matrix,
            factor_capture.inverse, factor_capture.rhs,
            export_wide_solve.solution, reference_signs,
            reference_certificate.rho_bound,
            reference_certificate.solution)
        : ALR20R63ISignCertificate{};
    const ALR20R63ISignCertificate strict_baseline =
        strict_solve.exact && reference_exact
        ? al_r20_r63i_sign_certificate(factor_capture.matrix,
            factor_capture.inverse, factor_capture.rhs,
            strict_solution, reference_signs,
            reference_certificate.rho_bound,
            reference_certificate.solution)
        : ALR20R63ISignCertificate{};
    const bool r63i_exact = factor_exact && reference_exact
        && wide_solve.root == AL_R20_R63J_WIDE_SOLVE_SHA256
        && export_wide_solve.root == AL_R20_R63J_EXPORT_WIDE_SOLVE_SHA256
        && strict_solve.root == AL_R20_R63J_STRICT_SOLVE_SHA256
        && wide_baseline.root == AL_R20_R63J_WIDE_CERTIFICATE_SHA256
        && export_wide_baseline.root
            == AL_R20_R63J_EXPORT_WIDE_CERTIFICATE_SHA256
        && strict_baseline.root == AL_R20_R63J_STRICT_CERTIFICATE_SHA256;

    const ALR20R63KLane parent_wide = r63i_exact
        ? al_r20_r63k_q_lane(wide_factor.r, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, wide_solve.solution, wide_baseline,
            reference_signs, reference_certificate.solution)
        : ALR20R63KLane{};
    const ALR20R63KLane parent_export = r63i_exact
        ? al_r20_r63k_q_lane(exported_r, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, export_wide_solve.solution,
            export_wide_baseline, reference_signs,
            reference_certificate.solution)
        : ALR20R63KLane{};
    const ALR20R63KLane parent_strict = r63i_exact
        ? al_r20_r63k_d_lane(exported_r_d, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, strict_solve.solution, strict_baseline,
            reference_signs, reference_certificate.solution)
        : ALR20R63KLane{};
    const bool parent_r63k_exact = r63i_exact
        && parent_wide.exact && parent_wide.finite && !parent_wide.passed
        && parent_export.exact && parent_export.finite && !parent_export.passed
        && parent_strict.exact && parent_strict.finite && !parent_strict.passed
        && parent_wide.root == AL_R20_R63L_WIDE_PARENT_SHA256
        && parent_export.root == AL_R20_R63L_EXPORT_PARENT_SHA256
        && parent_strict.root == AL_R20_R63L_STRICT_PARENT_SHA256;

    const ALR20R63LContinuation wide_frontier = parent_r63k_exact
        ? al_r20_r63l_continue(wide_factor.r, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, parent_wide, reference_signs,
            reference_certificate.solution)
        : ALR20R63LContinuation{};
    const ALR20R63LContinuation export_frontier = parent_r63k_exact
        ? al_r20_r63l_continue(exported_r, permutation, inverse_scale,
            factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, parent_export, reference_signs,
            reference_certificate.solution)
        : ALR20R63LContinuation{};

    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    auto frontier_work_exact = [](const ALR20R63LContinuation& lane) {
        if (!lane.finite) return lane.exact
            && lane.failure_iteration >= 17U && lane.failure_iteration <= 32U;
        return lane.completed_iterations == 16U
            && lane.residual_dots == 1632U && lane.update_dots == 1632U
            && lane.forward_terms == 82416U
            && lane.backward_terms == 82416U && lane.divisions == 3264U
            && lane.certificate_residual_dots == 1632U
            && lane.certificate_image_dots == 1632U;
    };
    const bool work_exact = frontier_work_exact(wide_frontier)
        && frontier_work_exact(export_frontier);
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_r63k_exact && wide_frontier.exact && export_frontier.exact
        && work_exact && lifecycle_exact;
    const std::string route = al_r20_r63l_classify(apparatus_exact,
        wide_frontier.passed, export_frontier.passed);
    const bool exact = apparatus_exact;
    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63l-wide-refinement-frontier.v1|"
        << AL_R20_R63L_PARENT_SEMANTIC_SHA256 << ':' << route << ':'
        << controls.root << ':' << parent_wide.root << ':'
        << parent_export.root << ':' << parent_strict.root << ':'
        << wide_frontier.root << ':' << export_frontier.root;
    const std::string result_sha = sha256_hex(semantic.str());

    auto frontier_json = [](const char* name,
        const ALR20R63LContinuation& lane) {
        std::ostringstream stream;
        stream << "\"" << name << "\":{\"exact\":"
            << (lane.exact ? "true" : "false")
            << ",\"finite\":" << (lane.finite ? "true" : "false")
            << ",\"passed\":" << (lane.passed ? "true" : "false")
            << ",\"monotone_error\":"
            << (lane.monotone_error ? "true" : "false")
            << ",\"first_passing_iteration\":"
            << lane.first_passing_iteration
            << ",\"minimum_error\":\""
            << al_binary128_text(lane.minimum_error) << "\",\"steps\":[";
        for (std::size_t index = 0U; index < lane.steps.size(); ++index) {
            if (index != 0U) stream << ',';
            const ALR20R63LStep& step = lane.steps[index];
            stream << "{\"iteration\":" << step.iteration
                << ",\"passed\":"
                << (step.certificate.passed ? "true" : "false")
                << ",\"error\":\""
                << al_binary128_text(step.certificate.error)
                << "\",\"positive\":" << step.certificate.positive
                << ",\"negative\":" << step.certificate.negative
                << ",\"unresolved\":" << step.certificate.unresolved
                << ",\"minimum_separation\":\""
                << al_binary128_text(step.certificate.minimum_separation)
                << "\",\"maximum_reference_difference\":\""
                << al_binary128_text(
                    step.certificate.maximum_reference_difference)
                << "\",\"root\":\"" << step.root << "\"}";
        }
        stream << "],\"root\":\"" << lane.root << "\"}";
        return stream.str();
    };
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63l_wide_refinement_frontier.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63k_sha256\":\""
        << AL_R20_R63L_PARENT_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":"
        << (controls_exact ? "true" : "false")
        << ",\"controls_root\":\"" << controls.root << "\""
        << ",\"parent_r63k_exact\":"
        << (parent_r63k_exact ? "true" : "false")
        << ",\"frontiers\":{" << frontier_json("wide", wide_frontier)
        << ',' << frontier_json("export_wide", export_frontier) << '}'
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"added_factor_refinement_iterations\":"
        << (wide_frontier.completed_iterations
            + export_frontier.completed_iterations)
        << ",\"added_certificates\":"
        << (wide_frontier.steps.size() + export_frontier.steps.size())
        << ",\"strict_binary64_continuations\":0"
        << ",\"dense_inverse_corrections\":0,\"gmres_solves\":0"
        << ",\"pcg_solves\":0,\"minres_solves\":0"
        << ",\"new_inverse_constructions\":0,\"new_factorizations\":0"
        << ",\"rank_actions\":0,\"row_drops\":0,\"regularizations\":0"
        << ",\"replacement_applies\":0,\"following_transitions\":0"
        << ",\"state_updates\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"gpu_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
