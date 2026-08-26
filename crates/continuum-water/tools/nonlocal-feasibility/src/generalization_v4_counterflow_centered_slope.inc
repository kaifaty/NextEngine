namespace {

constexpr const char* AL_R20_R51_SEMANTIC_SHA256 =
    "48df3b2650abb5ff9338c6926c60754494418cfd7037ef4c43851b8c7946adca";
constexpr const char* AL_R20_R36_CURRENT_SHA256 =
    "e622136383b33b422bb4b92ffefe9775efba0d799da424cc08cb934f07f6526e";
constexpr const char* AL_R20_R36_DIRECTION_SHA256 =
    "f09a8af6f23c7c27f5d0e4963e5ff19448de6df87f4e598c949be4a9bb49821e";
constexpr const char* AL_R20_R36_SLOPE_SHA256 =
    "57b1b93b4e0bc0c5c77ee0097bf192abb2771157fc2315dc337e33dd5184f02f";

struct ALR20R52SlopeAudit {
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
    std::string direction_root;
    std::string natural_face_root;
    std::string root;
};

ALR20R52SlopeAudit al_r20_r52_slope_audit(
    const ALR20ProblemData& problem,
    const ALR20R35CaseAudit& parent,
    Binary128 direction_error) {
    ALR20R52SlopeAudit audit;
    const ALR20SemismoothCase& diagnostic = parent.diagnostic;
    if (parent.root != AL_R20_R35_COUNTERFLOW_CASE_SHA256
        || diagnostic.steps.empty()
        || diagnostic.final_lambda.size() != problem.row_scale.size()
        || !(direction_error >= static_cast<Binary128>(0.0))
        || finiteq(direction_error) == 0)
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
    std::vector<Binary128> direction(diagnostic.final_lambda.size());
    Binary128Accumulator slope;
    Binary128 slope_absolute = static_cast<Binary128>(0.0);
    for (std::size_t row = 0U; row < direction.size(); ++row) {
        direction[row] = step.line_representative[row]
            - diagnostic.final_lambda[row];
        const Binary128 slope_term = current.residual[row] * direction[row];
        slope.add(slope_term);
        slope_absolute += fabsq(slope_term);
        const Binary128 component_error = std::binary_search(
            natural_face.begin(), natural_face.end(), row)
            ? direction_error : static_cast<Binary128>(0.0);
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
    audit.direction_root = al_r20_q_scalar_root(direction);
    audit.exact = current.exact
        && audit.natural_face_size == step.natural_face_size
        && audit.natural_face_root == step.natural_face_root
        && audit.current_root == AL_R20_R36_CURRENT_SHA256
        && audit.direction_root == AL_R20_R36_DIRECTION_SHA256
        && audit.nominal_slope == step.slope
        && al_r20_admm_finite(direction)
        && finiteq(audit.residual_bound_term) != 0
        && finiteq(audit.direction_error_term) != 0
        && finiteq(audit.rounding_term) != 0
        && finiteq(audit.ordered_bound) != 0
        && finiteq(audit.lower_slope) != 0;
    std::ostringstream material;
    material << audit.exact << ':' << audit.certified << ':'
        << audit.natural_face_size << ':'
        << al_binary128_hex(direction_error) << ':'
        << al_binary128_hex(audit.nominal_slope) << ':'
        << al_binary128_hex(audit.residual_bound_term) << ':'
        << al_binary128_hex(audit.direction_error_term) << ':'
        << al_binary128_hex(audit.rounding_term) << ':'
        << al_binary128_hex(audit.ordered_bound) << ':'
        << al_binary128_hex(audit.lower_slope) << ':'
        << audit.current_root << ':' << audit.direction_root << ':'
        << audit.natural_face_root;
    audit.root = sha256_hex(material.str());
    return audit;
}

} // namespace

SplitBoundaryReport run_al_generalization_v4_counterflow_centered_slope_controls() {
#if defined(__linux__) && defined(__x86_64__) && defined(__GNUC__) \
    && defined(__SIZEOF_FLOAT128__)
    constexpr bool frozen_platform = true;
#else
    constexpr bool frozen_platform = false;
#endif
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
    ALR20VerifiedInverseCapture capture;
    ALR20R35CaseAudit counterflow;
    std::size_t solver_executions = 0U;
    const bool hook_slot_clear = al_r20_verified_solution_refiner == nullptr
        && al_r20_verified_inverse_capture == nullptr;
    if (parent_exact && hook_slot_clear) {
        al_r20_verified_inverse_capture = &capture;
        counterflow = al_r20_r35_case(problem);
        al_r20_verified_inverse_capture = nullptr;
        solver_executions = 1U;
    }
    const bool hook_slot_released = al_r20_verified_solution_refiner == nullptr
        && al_r20_verified_inverse_capture == nullptr;
    const ALR20SemismoothStep* final_step =
        counterflow.diagnostic.steps.empty()
        ? nullptr : &counterflow.diagnostic.steps.back();
    const ALR20NNQPSolve* final_solve = final_step == nullptr
        ? nullptr : &final_step->solve;
    const ALR20VerifiedInverse* final_inverse = final_solve == nullptr
            || final_solve->inverse_audits.empty()
        ? nullptr : &final_solve->inverse_audits.back();
    const bool reproduction_exact = parent_exact && hook_slot_clear
        && hook_slot_released && solver_executions == 1U
        && al_r20_r35_case_observation_exact(counterflow)
        && counterflow.root == AL_R20_R35_COUNTERFLOW_CASE_SHA256
        && final_step != nullptr
        && final_step->root == AL_R20_R35_COUNTERFLOW_STEP_SHA256
        && counterflow.diagnostic.total_transitions == 701U;
    const bool capture_exact = reproduction_exact
        && capture.calls > 0U && capture.audit.passed
        && !capture.audit.refinement_attempted
        && final_inverse != nullptr
        && capture.audit.root == final_inverse->root
        && capture.audit.refined_error == final_step->direction_error_bound
        && capture.matrix.size() == capture.audit.dimension
            * capture.audit.dimension
        && capture.inverse.size() == capture.matrix.size()
        && capture.rhs.size() == capture.audit.dimension
        && capture.solution.size() == capture.audit.dimension;
    const ALR20R42PracticalCertificate certificate = capture_exact
        ? al_r20_r42_practical_certificate(capture.matrix, capture.inverse,
            capture.rhs, capture.solution, 16U, false)
        : ALR20R42PracticalCertificate{};
    const ALR20R36SlopeAudit inherited = reproduction_exact
        ? al_r20_r36_slope_audit(problem, counterflow)
        : ALR20R36SlopeAudit{};
    const ALR20R52SlopeAudit refined = certificate.exact
        ? al_r20_r52_slope_audit(problem, counterflow, certificate.error)
        : ALR20R52SlopeAudit{};
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool apparatus_exact = capture_exact && inherited.exact
        && inherited.root == AL_R20_R36_SLOPE_SHA256
        && certificate.exact && refined.exact;
    const bool exact = reproduction_exact && apparatus_exact && lifecycle_exact;
    const std::string route = !parent_exact || !reproduction_exact
        ? "COUNTERFLOW_CENTER_PARENT_REJECTED"
        : (!capture_exact
            ? "COUNTERFLOW_CENTER_CAPTURE_REJECTED"
            : (!apparatus_exact
                ? "COUNTERFLOW_CENTER_APPARATUS_REJECTED"
                : (!certificate.no_underflow
                    ? "COUNTERFLOW_CENTER_UNDERFLOW_REJECTED"
                    : (!certificate.contractive
                        ? "COUNTERFLOW_CENTER_LEFT_NONCONTRACTIVE"
                        : (certificate.unresolved != 0U
                                || !(certificate.minimum_separation
                                    > static_cast<Binary128>(0.0))
                            ? "COUNTERFLOW_CENTER_SIGN_UNRESOLVED"
                            : (!refined.certified
                                ? "COUNTERFLOW_CENTER_SLOPE_STILL_UNRESOLVED"
                                : "COUNTERFLOW_CENTERED_SLOPE_CANDIDATE"))))));
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r52-counterflow-center.v1|"
        << AL_R20_R51_SEMANTIC_SHA256 << ':' << route << ':'
        << capture.audit.root << ':' << certificate.root << ':'
        << inherited.root << ':' << refined.root;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r52_counterflow_center.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r51_sha256\":\""
        << AL_R20_R51_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"reproduction_exact\":"
        << (reproduction_exact ? "true" : "false")
        << ",\"capture_exact\":" << (capture_exact ? "true" : "false")
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"capture\":{\"calls\":" << capture.calls
        << ",\"total_column_solves\":" << capture.total_column_solves
        << ",\"dimension\":" << capture.audit.dimension
        << ",\"audit_root\":\"" << capture.audit.root
        << "\",\"matrix_root\":\"" << capture.matrix_root
        << "\",\"rhs_root\":\"" << al_r20_q_scalar_root(capture.rhs)
        << "\",\"solution_root\":\""
        << al_r20_q_scalar_root(capture.solution)
        << "\",\"inherited_error\":\""
        << al_binary128_text(capture.audit.refined_error) << "\"}"
        << ",\"final_solve\":{\"inverse_audits\":"
        << (final_solve == nullptr ? 0U : final_solve->inverse_audits.size())
        << ",\"support_size\":"
        << (final_solve == nullptr ? 0U : final_solve->support_size)
        << ",\"affine_shadow_updates\":"
        << (final_solve == nullptr ? 0U : final_solve->affine_shadow_updates)
        << ",\"direction_error_bound\":\""
        << al_binary128_text(final_solve == nullptr
            ? static_cast<Binary128>(0.0)
            : final_solve->direction_error_bound)
        << "\"}"
        << ",\"certificate\":{\"root\":\"" << certificate.root
        << "\",\"rho_bound\":\""
        << al_binary128_text(certificate.rho_bound)
        << "\",\"radius\":\"" << al_binary128_text(certificate.radius)
        << "\",\"uniform_error\":\""
        << al_binary128_text(certificate.error)
        << "\",\"positive\":" << certificate.positive
        << ",\"negative\":" << certificate.negative
        << ",\"unresolved\":" << certificate.unresolved
        << ",\"minimum_separation\":\""
        << al_binary128_text(certificate.minimum_separation) << "\"}"
        << ",\"inherited_slope\":{\"root\":\"" << inherited.root
        << "\",\"nominal\":\""
        << al_binary128_text(inherited.nominal_slope)
        << "\",\"bound\":\""
        << al_binary128_text(inherited.ordered_bound)
        << "\",\"direction_term\":\""
        << al_binary128_text(inherited.direction_error_term) << "\"}"
        << ",\"refined_slope\":{\"root\":\"" << refined.root
        << "\",\"certified\":"
        << (refined.certified ? "true" : "false")
        << ",\"natural_face_size\":" << refined.natural_face_size
        << ",\"nominal\":\"" << al_binary128_text(refined.nominal_slope)
        << "\",\"residual_term\":\""
        << al_binary128_text(refined.residual_bound_term)
        << "\",\"direction_term\":\""
        << al_binary128_text(refined.direction_error_term)
        << "\",\"rounding_term\":\""
        << al_binary128_text(refined.rounding_term)
        << "\",\"bound\":\"" << al_binary128_text(refined.ordered_bound)
        << "\",\"lower_slope\":\""
        << al_binary128_text(refined.lower_slope) << "\"}"
        << ",\"solver_executions\":" << solver_executions
        << ",\"new_inverse_solves\":0,\"correction_applies\":0"
        << ",\"new_directions\":0,\"new_trials\":0"
        << ",\"state_updates\":0,\"parameter_changes\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
