namespace {

constexpr const char* AL_R20_R53_SEMANTIC_SHA256 =
    "5418141a96d4a2c2d89d875450aeaf2164a66811b01a510a30e64625b91740a4";
constexpr const char* AL_R20_R53_MATRIX_SHA256 =
    "3b0b5b6b7fc6a4c8afc26cbf488c89bec735f4a7d97cf7f0e943bb16585ba98a";
constexpr const char* AL_R20_R53_FACTOR_SHA256 =
    "23f1bbb2224eb1cbe5ea1227a4510b0a30c5aa6dd1337170b525d9719d6a10b7";
constexpr const char* AL_R20_R53_RHS_SHA256 =
    "698106e6086678e6bf93bd7b72b44ffa8a77b11b7de514ef76881096175a6d3a";
constexpr const char* AL_R20_R53_SOLUTION_SHA256 =
    "820c3d59b71a9a331c512a07958173b3be92afdd0a741f9754f276bd60f23ab7";
constexpr const char* AL_R20_R53_LEGACY_AUDIT_SHA256 =
    "e49c0000e85ae90b34aadbb7df95a7b9480405c7d628c8caffbc759f8c0a3fc6";

} // namespace

SplitBoundaryReport run_al_generalization_v4_counterflow_legacy_slope_controls() {
#if defined(__linux__) && defined(__x86_64__) && defined(__GNUC__) \
    && defined(__SIZEOF_FLOAT128__)
    constexpr bool frozen_platform = true;
#else
    constexpr bool frozen_platform = false;
#endif
    constexpr std::size_t dimension = 66U;
    const bool profile_exact = frozen_platform
        && sizeof(Binary128) == 16U && FLT_RADIX == 2
        && FLT128_MANT_DIG == 113;
    const bool controls_exact = profile_exact
        && al_r20_verified_inverse_controls()
        && al_r20_semismooth_controls()
        && al_r20_r21_event_controls();
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
        && direct.calls == 622U && direct.factor.passed
        && direct.factor.dimension == dimension
        && direct.transition == final_solve->transitions
        && direct.transition == 86U && identity_indices
        && direct.source_rows.size() == dimension
        && al_r20_r53_row_root(direct.source_rows)
            == final_step->natural_face_root
        && final_solve->support_size == dimension
        && direct.solution == final_solve->direction
        && direct.cheap_error == final_solve->direction_error_bound
        && direct.factor.matrix_root == AL_R20_R53_MATRIX_SHA256
        && direct.factor.factor_root == AL_R20_R53_FACTOR_SHA256
        && al_r20_q_scalar_root(direct.rhs) == AL_R20_R53_RHS_SHA256
        && al_r20_q_scalar_root(direct.solution)
            == AL_R20_R53_SOLUTION_SHA256;

    ALR20ADMMWork inverse_work;
    ALR20VerifiedInverseCapture inverse_capture;
    ALR20VerifiedInverse inverse_audit;
    const bool inverse_hook_clear =
        al_r20_verified_inverse_capture == nullptr;
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
        && inverse_audit.passed && !inverse_audit.refinement_attempted
        && inverse_audit.dimension == dimension
        && inverse_audit.column_solves == dimension
        && inverse_capture.total_column_solves == dimension
        && inverse_audit.root == AL_R20_R53_LEGACY_AUDIT_SHA256
        && inverse_capture.audit.root == inverse_audit.root
        && inverse_audit.refined_error == strtoflt128(
            "1.093151728063539466044957165073324068e-24", nullptr)
        && inverse_work.factor_terms == 0U
        && inverse_work.matrix_outer_terms == 0U;
    const ALR20R36SlopeAudit inherited = reproduction_exact
        ? al_r20_r36_slope_audit(problem, counterflow)
        : ALR20R36SlopeAudit{};
    const ALR20R52SlopeAudit refined = inverse_exact
        ? al_r20_r52_slope_audit(
            problem, counterflow, inverse_audit.refined_error)
        : ALR20R52SlopeAudit{};
    const bool slope_exact = inherited.exact && refined.exact
        && inherited.root == AL_R20_R36_SLOPE_SHA256
        && refined.current_root == inherited.current_root
        && refined.direction_root == inherited.direction_root
        && refined.natural_face_root == final_step->natural_face_root
        && refined.nominal_slope == inherited.nominal_slope;
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool apparatus_exact = capture_exact && inverse_exact && slope_exact;
    const bool exact = reproduction_exact && apparatus_exact && lifecycle_exact;
    const std::string route = !parent_exact || !reproduction_exact
        ? "COUNTERFLOW_LEGACY_PARENT_REJECTED"
        : (!capture_exact
            ? "COUNTERFLOW_LEGACY_CAPTURE_REJECTED"
            : (!inverse_exact
                ? "COUNTERFLOW_LEGACY_INVERSE_REJECTED"
                : (!slope_exact
                    ? "COUNTERFLOW_LEGACY_SLOPE_APPARATUS_REJECTED"
                    : (!refined.certified
                        ? "COUNTERFLOW_LEGACY_SLOPE_STILL_UNRESOLVED"
                        : "COUNTERFLOW_LEGACY_SLOPE_CANDIDATE"))));
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r54-counterflow-legacy.v1|"
        << AL_R20_R53_SEMANTIC_SHA256 << ':' << route << ':'
        << inverse_audit.root << ':' << inherited.root << ':' << refined.root;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r54_counterflow_legacy.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r53_sha256\":\"" << AL_R20_R53_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"reproduction_exact\":" << (reproduction_exact ? "true" : "false")
        << ",\"capture_exact\":" << (capture_exact ? "true" : "false")
        << ",\"inverse_exact\":" << (inverse_exact ? "true" : "false")
        << ",\"slope_exact\":" << (slope_exact ? "true" : "false")
        << ",\"apparatus_exact\":" << (apparatus_exact ? "true" : "false")
        << ",\"lifecycle_exact\":" << (lifecycle_exact ? "true" : "false")
        << ",\"direct\":{\"calls\":" << direct.calls
        << ",\"transition\":" << direct.transition
        << ",\"dimension\":" << direct.factor.dimension
        << ",\"matrix_root\":\"" << direct.factor.matrix_root
        << "\",\"rhs_root\":\"" << al_r20_q_scalar_root(direct.rhs)
        << "\",\"solution_root\":\"" << al_r20_q_scalar_root(direct.solution)
        << "\",\"cheap_error\":\"" << al_binary128_text(direct.cheap_error)
        << "\"}"
        << ",\"inverse\":{\"root\":\"" << inverse_audit.root
        << "\",\"passed\":" << (inverse_audit.passed ? "true" : "false")
        << ",\"column_solves\":" << inverse_audit.column_solves
        << ",\"rho\":\"" << al_binary128_text(inverse_audit.norm.rho)
        << "\",\"refined_error\":\""
        << al_binary128_text(inverse_audit.refined_error) << "\"}"
        << ",\"inherited_slope\":{\"root\":\"" << inherited.root
        << "\",\"nominal\":\"" << al_binary128_text(inherited.nominal_slope)
        << "\",\"bound\":\"" << al_binary128_text(inherited.ordered_bound)
        << "\",\"direction_term\":\""
        << al_binary128_text(inherited.direction_error_term) << "\"}"
        << ",\"refined_slope\":{\"root\":\"" << refined.root
        << "\",\"certified\":" << (refined.certified ? "true" : "false")
        << ",\"nominal\":\"" << al_binary128_text(refined.nominal_slope)
        << "\",\"residual_term\":\""
        << al_binary128_text(refined.residual_bound_term)
        << "\",\"direction_term\":\""
        << al_binary128_text(refined.direction_error_term)
        << "\",\"rounding_term\":\"" << al_binary128_text(refined.rounding_term)
        << "\",\"bound\":\"" << al_binary128_text(refined.ordered_bound)
        << "\",\"lower_slope\":\"" << al_binary128_text(refined.lower_slope)
        << "\",\"direction_root\":\"" << refined.direction_root << "\"}"
        << ",\"solver_executions\":" << solver_executions
        << ",\"additional_inverse_columns\":" << inverse_audit.column_solves
        << ",\"additional_factorizations\":0,\"compensated_dots\":0"
        << ",\"correction_applies\":0,\"new_directions\":0"
        << ",\"new_trials\":0,\"state_updates\":0,\"parameter_changes\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
