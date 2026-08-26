namespace {

constexpr const char* AL_R20_R52_SEMANTIC_SHA256 =
    "86d2dcd45425b2885a86928bfdd17da3df51da6d47ea4a03538b299f24bb17a6";

struct ALR20R53SlopeAudit {
    bool exact = false;
    bool certified = false;
    std::size_t natural_face_size = 0U;
    Binary128 nominal_slope = static_cast<Binary128>(0.0);
    Binary128 residual_bound_term = static_cast<Binary128>(0.0);
    Binary128 direction_error_term = static_cast<Binary128>(0.0);
    Binary128 rounding_term = static_cast<Binary128>(0.0);
    Binary128 ordered_bound = static_cast<Binary128>(0.0);
    Binary128 lower_slope = static_cast<Binary128>(0.0);
    std::string current_root;
    std::string center_root;
    std::string representative_root;
    std::string direction_root;
    std::string natural_face_root;
    std::string root;
};

ALR20R53SlopeAudit al_r20_r53_slope_audit(
    const ALR20ProblemData& problem,
    const ALR20R35CaseAudit& parent,
    const std::vector<Binary128>& center,
    Binary128 center_error) {
    ALR20R53SlopeAudit audit;
    const ALR20SemismoothCase& diagnostic = parent.diagnostic;
    if (parent.root != AL_R20_R35_COUNTERFLOW_CASE_SHA256
        || diagnostic.steps.empty()
        || diagnostic.final_lambda.size() != problem.row_scale.size()
        || center.empty()
        || !(center_error >= static_cast<Binary128>(0.0))
        || finiteq(center_error) == 0)
        return audit;
    const ALR20SemismoothStep& step = diagnostic.steps.back();
    if (step.root != AL_R20_R35_COUNTERFLOW_STEP_SHA256)
        return audit;
    const ALR20NewtonDualEvaluation current =
        al_r20_newton_dual_evaluate(problem, diagnostic.final_lambda);
    const std::vector<Binary128> diagonal = al_r20_q_diagonal(problem.op);
    std::vector<std::size_t> natural_face;
    for (std::size_t row = 0U;
         row < diagnostic.final_lambda.size(); ++row) {
        const Binary128 quotient = current.residual[row] / diagonal[row];
        const Binary128 argument = diagnostic.final_lambda[row] + quotient;
        const Binary128 bound = current.residual_bound[row] / diagonal[row]
            + al_r20_q_gamma(256U)
                * (fabsq(diagnostic.final_lambda[row]) + fabsq(quotient));
        if (argument - bound > static_cast<Binary128>(0.0))
            natural_face.push_back(row);
    }
    std::ostringstream face_material;
    for (std::size_t row : natural_face) face_material << row << ',';
    audit.natural_face_size = natural_face.size();
    audit.natural_face_root = sha256_hex(face_material.str());
    if (center.size() != natural_face.size()) return audit;

    std::vector<Binary128> representative(
        diagnostic.final_lambda.size(), static_cast<Binary128>(0.0));
    for (std::size_t local = 0U; local < natural_face.size(); ++local)
        representative[natural_face[local]] = center[local];
    std::vector<Binary128> direction(representative.size());
    Binary128Accumulator slope;
    Binary128 slope_absolute = static_cast<Binary128>(0.0);
    for (std::size_t row = 0U; row < direction.size(); ++row) {
        direction[row] = representative[row]
            - diagnostic.final_lambda[row];
        const Binary128 slope_term = current.residual[row] * direction[row];
        slope.add(slope_term);
        slope_absolute += fabsq(slope_term);
        const Binary128 component_error = std::binary_search(
            natural_face.begin(), natural_face.end(), row)
            ? center_error : static_cast<Binary128>(0.0);
        const Binary128 residual_term = current.residual_bound[row]
            * (fabsq(direction[row]) + component_error);
        const Binary128 direction_term =
            fabsq(current.residual[row]) * component_error;
        audit.residual_bound_term += residual_term;
        audit.direction_error_term += direction_term;
        audit.ordered_bound += residual_term + direction_term;
    }
    audit.nominal_slope = slope.value();
    audit.rounding_term = al_r20_q_gamma(16U * direction.size() + 512U)
        * slope_absolute;
    audit.ordered_bound += audit.rounding_term;
    audit.lower_slope = audit.nominal_slope - audit.ordered_bound;
    audit.certified = audit.lower_slope > static_cast<Binary128>(0.0);
    audit.current_root = current.root;
    audit.center_root = al_r20_q_scalar_root(center);
    audit.representative_root = al_r20_q_scalar_root(representative);
    audit.direction_root = al_r20_q_scalar_root(direction);
    audit.exact = current.exact
        && audit.natural_face_size == step.natural_face_size
        && audit.natural_face_root == step.natural_face_root
        && audit.current_root == AL_R20_R36_CURRENT_SHA256
        && al_r20_admm_finite(center)
        && al_r20_admm_finite(representative)
        && al_r20_admm_finite(direction)
        && finiteq(audit.nominal_slope) != 0
        && finiteq(audit.residual_bound_term) != 0
        && finiteq(audit.direction_error_term) != 0
        && finiteq(audit.rounding_term) != 0
        && finiteq(audit.ordered_bound) != 0
        && finiteq(audit.lower_slope) != 0;
    std::ostringstream material;
    material << audit.exact << ':' << audit.certified << ':'
        << audit.natural_face_size << ':'
        << al_binary128_hex(center_error) << ':'
        << al_binary128_hex(audit.nominal_slope) << ':'
        << al_binary128_hex(audit.residual_bound_term) << ':'
        << al_binary128_hex(audit.direction_error_term) << ':'
        << al_binary128_hex(audit.rounding_term) << ':'
        << al_binary128_hex(audit.ordered_bound) << ':'
        << al_binary128_hex(audit.lower_slope) << ':'
        << audit.current_root << ':' << audit.center_root << ':'
        << audit.representative_root << ':' << audit.direction_root << ':'
        << audit.natural_face_root;
    audit.root = sha256_hex(material.str());
    return audit;
}

std::string al_r20_r53_row_root(const std::vector<std::size_t>& rows) {
    std::ostringstream material;
    for (std::size_t row : rows) material << row << ',';
    return sha256_hex(material.str());
}

} // namespace

SplitBoundaryReport run_al_generalization_v4_counterflow_direct_center_controls() {
#if defined(__linux__) && defined(__x86_64__) && defined(__GNUC__) \
    && defined(__SIZEOF_FLOAT128__)
    constexpr bool frozen_platform = true;
#else
    constexpr bool frozen_platform = false;
#endif
    constexpr std::size_t dimension = 66U;
    constexpr std::size_t expected_dots = 5676U;
    constexpr std::size_t expected_input_pairs = 376002U;
    const bool profile_exact = frozen_platform
        && sizeof(Binary128) == 16U && FLT_RADIX == 2
        && FLT128_MANT_DIG == 113;
    const bool controls_exact = profile_exact
        && al_r20_verified_inverse_controls()
        && al_r20_semismooth_controls()
        && al_r20_r21_event_controls()
        && al_r20_r37_dyadic_controls()
        && al_r20_r38_dot2_controls();
    std::vector<ALR20ManifestCase> cases = al_r20_v4_manifest_cases();
    bool parent_exact = controls_exact && cases.size() == 5U
        && al_r20_v4_manifest_root(cases) == AL_R20_V4_MANIFEST_SHA256;
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    ALR20ProblemData problem;
    bool problem_materialized = false;
    if (parent_exact) {
        problem = al_r20_materialize_problem(
            cases[2U], &sparse_work, &static_work, &adjacency_work);
        problem_materialized = true;
        parent_exact = problem.exact && problem.source_positive == 0U
            && problem.problem_root == AL_R20_V4_PROBLEM_SHA256[2U];
    }

    ALR20DirectSolutionCapture direct;
    ALR20VerifiedInverseCapture replay_inverse;
    ALR20R35CaseAudit counterflow;
    std::size_t solver_executions = 0U;
    const bool hook_slot_clear = al_r20_verified_solution_refiner == nullptr
        && al_r20_verified_inverse_capture == nullptr
        && al_r20_direct_solution_capture == nullptr;
    if (parent_exact && hook_slot_clear) {
        al_r20_direct_solution_capture = &direct;
        al_r20_verified_inverse_capture = &replay_inverse;
        counterflow = al_r20_r35_case(problem);
        al_r20_verified_inverse_capture = nullptr;
        al_r20_direct_solution_capture = nullptr;
        solver_executions = 1U;
    }
    const bool replay_hook_released =
        al_r20_verified_solution_refiner == nullptr
        && al_r20_verified_inverse_capture == nullptr
        && al_r20_direct_solution_capture == nullptr;
    const ALR20SemismoothStep* final_step =
        counterflow.diagnostic.steps.empty()
        ? nullptr : &counterflow.diagnostic.steps.back();
    const ALR20NNQPSolve* final_solve = final_step == nullptr
        ? nullptr : &final_step->solve;
    const bool reproduction_exact = parent_exact && hook_slot_clear
        && replay_hook_released && solver_executions == 1U
        && al_r20_r35_case_observation_exact(counterflow)
        && counterflow.root == AL_R20_R35_COUNTERFLOW_CASE_SHA256
        && final_step != nullptr
        && final_step->root == AL_R20_R35_COUNTERFLOW_STEP_SHA256
        && counterflow.diagnostic.total_transitions == 701U
        && replay_inverse.calls == 0U;
    bool identity_indices = direct.indices.size() == dimension;
    for (std::size_t index = 0U; identity_indices && index < dimension; ++index)
        identity_indices = direct.indices[index] == index;
    const bool capture_exact = reproduction_exact && final_solve != nullptr
        && direct.calls > 0U && direct.factor.passed
        && direct.factor.dimension == dimension
        && direct.transition == final_solve->transitions
        && direct.source_rows.size() == dimension
        && al_r20_r53_row_root(direct.source_rows)
            == final_step->natural_face_root
        && identity_indices && final_solve->support_size == dimension
        && final_solve->passive.size() == dimension
        && std::all_of(final_solve->passive.begin(), final_solve->passive.end(),
            [](bool value) { return value; })
        && direct.rhs.size() == dimension
        && direct.solution == final_solve->direction
        && direct.cheap_error == final_solve->direction_error_bound
        && direct.factor.matrix.size() == dimension * dimension
        && direct.factor.lower.size() == direct.factor.matrix.size();

    ALR20ADMMWork inverse_work;
    ALR20VerifiedInverseCapture inverse_capture;
    ALR20VerifiedInverse inverse_audit;
    bool inverse_hook_clear = al_r20_verified_inverse_capture == nullptr;
    if (capture_exact && inverse_hook_clear) {
        al_r20_verified_inverse_capture = &inverse_capture;
        inverse_audit = al_r20_verified_inverse(direct.factor,
            direct.solve_residual, direct.solve_bound, direct.solution,
            direct.cheap_error, &inverse_work, &direct.rhs);
        al_r20_verified_inverse_capture = nullptr;
    }
    const bool inverse_hook_released =
        al_r20_verified_inverse_capture == nullptr;
    const bool inverse_exact = capture_exact && inverse_hook_clear
        && inverse_hook_released && inverse_capture.calls == 1U
        && inverse_audit.column_solves == dimension
        && inverse_capture.total_column_solves == dimension
        && inverse_capture.audit.root == inverse_audit.root
        && inverse_capture.matrix == direct.factor.matrix
        && inverse_capture.rhs == direct.rhs
        && inverse_capture.solution == direct.solution
        && inverse_capture.inverse.size() == dimension * dimension
        && inverse_work.factor_terms == 0U
        && inverse_work.matrix_outer_terms == 0U;
    const ALR20R42PracticalCertificate certificate = inverse_exact
        ? al_r20_r42_practical_certificate(direct.factor.matrix,
            inverse_capture.inverse, direct.rhs, direct.solution, 16U, false)
        : ALR20R42PracticalCertificate{};
    const std::size_t certificate_dots = certificate.left_dots
        + certificate.residual_dots + certificate.z_dots
        + certificate.center_dots + certificate.certificate_dots
        + certificate.sum_dots;
    const std::size_t certificate_input_pairs =
        certificate.left_dots * (dimension + 1U)
        + certificate.residual_dots * (dimension + 1U)
        + certificate.z_dots * dimension
        + certificate.center_dots * (dimension + 1U)
        + certificate.certificate_dots * (dimension + 2U)
        + certificate.sum_dots * 2U;
    const bool work_exact = inverse_exact
        && certificate_dots == expected_dots
        && certificate_input_pairs == expected_input_pairs;
    const bool certificate_exact = work_exact && certificate.exact
        && certificate.dimension == dimension;
    const bool face_preserved = certificate_exact
        && certificate.positive == dimension
        && certificate.negative == 0U && certificate.unresolved == 0U
        && certificate.minimum_separation > static_cast<Binary128>(0.0);
    const ALR20R36SlopeAudit inherited = reproduction_exact
        ? al_r20_r36_slope_audit(problem, counterflow)
        : ALR20R36SlopeAudit{};
    const ALR20R53SlopeAudit shadow = face_preserved
        ? al_r20_r53_slope_audit(problem, counterflow,
            certificate.solution, certificate.error)
        : ALR20R53SlopeAudit{};
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool apparatus_exact = capture_exact && inverse_exact
        && certificate_exact && inherited.exact
        && inherited.root == AL_R20_R36_SLOPE_SHA256
        && (!face_preserved || shadow.exact);
    const bool exact = reproduction_exact && apparatus_exact && lifecycle_exact;
    const std::string route = !parent_exact || !reproduction_exact
        ? "COUNTERFLOW_DIRECT_PARENT_REJECTED"
        : (!capture_exact
            ? "COUNTERFLOW_DIRECT_CAPTURE_REJECTED"
            : (!inverse_exact
                ? "COUNTERFLOW_DIRECT_INVERSE_REJECTED"
                : (!certificate_exact
                    ? "COUNTERFLOW_DIRECT_CERTIFICATE_REJECTED"
                    : (!certificate.no_underflow
                        ? "COUNTERFLOW_DIRECT_UNDERFLOW_REJECTED"
                        : (!certificate.contractive
                            ? "COUNTERFLOW_DIRECT_LEFT_NONCONTRACTIVE"
                            : (!face_preserved
                                ? "COUNTERFLOW_DIRECT_FACE_CHANGED"
                                : (!shadow.certified
                                    ? "COUNTERFLOW_DIRECT_SLOPE_STILL_UNRESOLVED"
                                    : "COUNTERFLOW_DIRECT_CENTERED_SLOPE_CANDIDATE")))))));
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r53-counterflow-direct.v1|"
        << AL_R20_R52_SEMANTIC_SHA256 << ':' << route << ':'
        << direct.factor.matrix_root << ':' << direct.factor.factor_root << ':'
        << al_r20_q_scalar_root(direct.rhs) << ':'
        << al_r20_q_scalar_root(direct.solution) << ':'
        << inverse_audit.root << ':' << certificate.root << ':'
        << inherited.root << ':' << shadow.root;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r53_counterflow_direct.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r52_sha256\":\"" << AL_R20_R52_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"reproduction_exact\":" << (reproduction_exact ? "true" : "false")
        << ",\"capture_exact\":" << (capture_exact ? "true" : "false")
        << ",\"inverse_exact\":" << (inverse_exact ? "true" : "false")
        << ",\"certificate_exact\":" << (certificate_exact ? "true" : "false")
        << ",\"apparatus_exact\":" << (apparatus_exact ? "true" : "false")
        << ",\"lifecycle_exact\":" << (lifecycle_exact ? "true" : "false")
        << ",\"direct\":{\"calls\":" << direct.calls
        << ",\"transition\":" << direct.transition
        << ",\"dimension\":" << direct.factor.dimension
        << ",\"matrix_root\":\"" << direct.factor.matrix_root
        << "\",\"factor_root\":\"" << direct.factor.factor_root
        << "\",\"rhs_root\":\"" << al_r20_q_scalar_root(direct.rhs)
        << "\",\"solution_root\":\"" << al_r20_q_scalar_root(direct.solution)
        << "\",\"cheap_error\":\"" << al_binary128_text(direct.cheap_error)
        << "\",\"source_root\":\"" << al_r20_r53_row_root(direct.source_rows)
        << "\"}"
        << ",\"inverse\":{\"legacy_passed\":"
        << (inverse_audit.passed ? "true" : "false")
        << ",\"audit_root\":\"" << inverse_audit.root
        << "\",\"column_solves\":" << inverse_audit.column_solves
        << ",\"legacy_rho\":\"" << al_binary128_text(inverse_audit.norm.rho)
        << "\",\"legacy_refined_error\":\""
        << al_binary128_text(inverse_audit.refined_error) << "\"}"
        << ",\"certificate\":{\"root\":\"" << certificate.root
        << "\",\"passed\":" << (certificate.passed ? "true" : "false")
        << ",\"no_underflow\":" << (certificate.no_underflow ? "true" : "false")
        << ",\"contractive\":" << (certificate.contractive ? "true" : "false")
        << ",\"rho_bound\":\"" << al_binary128_text(certificate.rho_bound)
        << "\",\"radius\":\"" << al_binary128_text(certificate.radius)
        << "\",\"uniform_error\":\"" << al_binary128_text(certificate.error)
        << "\",\"positive\":" << certificate.positive
        << ",\"negative\":" << certificate.negative
        << ",\"unresolved\":" << certificate.unresolved
        << ",\"minimum_separation\":\""
        << al_binary128_text(certificate.minimum_separation)
        << "\",\"center_root\":\"" << certificate.center_root << "\"}"
        << ",\"inherited_slope\":{\"root\":\"" << inherited.root
        << "\",\"nominal\":\"" << al_binary128_text(inherited.nominal_slope)
        << "\",\"bound\":\"" << al_binary128_text(inherited.ordered_bound)
        << "\"}"
        << ",\"shadow_slope\":{\"root\":\"" << shadow.root
        << "\",\"certified\":" << (shadow.certified ? "true" : "false")
        << ",\"nominal\":\"" << al_binary128_text(shadow.nominal_slope)
        << "\",\"residual_term\":\""
        << al_binary128_text(shadow.residual_bound_term)
        << "\",\"direction_term\":\""
        << al_binary128_text(shadow.direction_error_term)
        << "\",\"rounding_term\":\"" << al_binary128_text(shadow.rounding_term)
        << "\",\"bound\":\"" << al_binary128_text(shadow.ordered_bound)
        << "\",\"lower_slope\":\"" << al_binary128_text(shadow.lower_slope)
        << "\",\"center_root\":\"" << shadow.center_root
        << "\",\"representative_root\":\"" << shadow.representative_root
        << "\",\"direction_root\":\"" << shadow.direction_root << "\"}"
        << ",\"work\":{\"inverse_columns\":" << inverse_audit.column_solves
        << ",\"compensated_dots\":" << certificate_dots
        << ",\"input_pairs\":" << certificate_input_pairs << "}"
        << ",\"solver_executions\":" << solver_executions
        << ",\"additional_factorizations\":0"
        << ",\"correction_applies\":0,\"new_directions\":0"
        << ",\"new_trials\":0,\"state_updates\":0,\"parameter_changes\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
