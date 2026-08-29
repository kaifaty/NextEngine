namespace {

constexpr const char* AL_R20_R42_SEMANTIC_SHA256 =
    "e435daf5e3ba43a6a51d4a014f9ed2b6d10dee4096b6f4ac5e9a942b53cd9a41";

struct ALR20R43HookTrace {
    std::size_t attempts = 0U;
    std::size_t target_matches = 0U;
    std::size_t replacements = 0U;
    std::size_t non_target_attempts = 0U;
    ALR20R42PracticalCertificate certificate;
    std::string target_material_root;
    std::size_t non_target_dimension = 0U;
    std::string non_target_material_root;
    std::string non_target_matrix_root;
    std::string non_target_inverse_root;
    std::string non_target_rhs_root;
    std::string non_target_solution_root;
};

thread_local ALR20R43HookTrace* al_r20_r43_hook_trace = nullptr;

ALR20VerifiedSolutionRefinement al_r20_r43_refiner(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& inverse,
    const std::vector<Binary128>& rhs,
    const std::vector<Binary128>& solution) {
    ALR20VerifiedSolutionRefinement result;
    ALR20R43HookTrace* trace = al_r20_r43_hook_trace;
    if (trace == nullptr) return result;
    ++trace->attempts;
    const std::string matrix_root = al_r20_q_scalar_root(matrix);
    const std::string inverse_root = al_r20_q_scalar_root(inverse);
    const std::string rhs_root = al_r20_q_scalar_root(rhs);
    const std::string solution_root = al_r20_q_scalar_root(solution);
    const std::string material_root = sha256_hex(matrix_root + ':'
        + inverse_root + ':' + rhs_root + ':' + solution_root);
    const bool target = matrix_root == AL_R20_R37_MATRIX_SHA256
        && inverse_root == AL_R20_R37_INVERSE_SHA256
        && rhs_root == AL_R20_R39_RHS_SHA256
        && solution_root == AL_R20_R39_SOLUTION_SHA256;
    if (!target || trace->replacements != 0U) {
        if (!target) {
            ++trace->non_target_attempts;
            trace->non_target_dimension = rhs.size();
            trace->non_target_material_root = material_root;
            trace->non_target_matrix_root = matrix_root;
            trace->non_target_inverse_root = inverse_root;
            trace->non_target_rhs_root = rhs_root;
            trace->non_target_solution_root = solution_root;
        }
        result.exact = true;
        result.root = sha256_hex(std::string("r43-non-target:")
            + material_root + ':'
            + std::to_string(trace->replacements));
        return result;
    }
    ++trace->target_matches;
    trace->target_material_root = material_root;
    trace->certificate = al_r20_r42_practical_certificate(
        matrix, inverse, rhs, solution, 8U);
    result.exact = trace->certificate.exact;
    result.passed = trace->certificate.passed;
    result.error = trace->certificate.error;
    result.solution = trace->certificate.solution;
    result.root = trace->certificate.root;
    if (result.passed) ++trace->replacements;
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v4_depth8_ratio_trajectory_controls() {
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
            cases[0U], &sparse_work, &static_work, &adjacency_work);
        problem_materialized = true;
        parent_exact = problem.exact && problem.source_positive == 0U
            && problem.problem_root == AL_R20_V4_PROBLEM_SHA256[0U];
    }
    ALR20R43HookTrace trace;
    ALR20R35CaseAudit torsion;
    std::size_t solver_executions = 0U;
    const bool hook_slot_clear = al_r20_verified_solution_refiner == nullptr
        && al_r20_r43_hook_trace == nullptr;
    if (parent_exact && hook_slot_clear) {
        al_r20_r43_hook_trace = &trace;
        al_r20_verified_solution_refiner = &al_r20_r43_refiner;
        torsion = al_r20_r35_case(problem);
        al_r20_verified_solution_refiner = nullptr;
        al_r20_r43_hook_trace = nullptr;
        solver_executions = 1U;
    }
    const bool hook_slot_released = al_r20_verified_solution_refiner == nullptr
        && al_r20_r43_hook_trace == nullptr;
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool observation_exact = parent_exact && hook_slot_clear
        && hook_slot_released && solver_executions == 1U
        && al_r20_r35_case_observation_exact(torsion);
    const bool hook_exact = trace.certificate.exact
        && trace.certificate.passed && trace.certificate.no_underflow;
    const bool cardinality_exact = trace.target_matches == 1U
        && trace.replacements == 1U;
    const bool consumed = cardinality_exact
        && !torsion.diagnostic.steps.empty()
        && torsion.diagnostic.total_transitions > 0U;
    const ALR20SemismoothStep* final_step = torsion.diagnostic.steps.empty()
        ? nullptr : &torsion.diagnostic.steps.back();
    const ALR20NNQPSolve* final_solve = final_step == nullptr
        ? nullptr : &final_step->solve;
    const bool ratio_still_unresolved = final_solve != nullptr
        && final_solve->failure == "BOUNDARY_RATIO_UNRESOLVED"
        && final_solve->ratio_failure == "RATIO_ORDER_AMBIGUOUS"
        && final_solve->ratio_best_source_row == 61U
        && final_solve->ratio_competitor_source_row == 13U;
    const bool exact = observation_exact && hook_exact
        && cardinality_exact && consumed && lifecycle_exact;
    const std::string route = !parent_exact || !observation_exact
        ? "DEPTH8_RATIO_PARENT_REJECTED"
        : (!hook_exact
            ? "DEPTH8_RATIO_HOOK_REJECTED"
            : (!cardinality_exact
                ? "DEPTH8_RATIO_TARGET_CARDINALITY_REJECTED"
                : (!consumed
                    ? "DEPTH8_RATIO_NOT_CONSUMED"
                    : (ratio_still_unresolved
                        ? "DEPTH8_RATIO_STILL_UNRESOLVED"
                        : (torsion.diagnostic.certified
                            ? "DEPTH8_RATIO_TORSION_CANDIDATE"
                            : "DEPTH8_RATIO_LATER_BOUNDARY")))));
    const std::string final_step_root = final_step == nullptr
        ? "" : final_step->root;
    const std::string final_solve_failure = final_solve == nullptr
        ? "NONE" : final_solve->failure;
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r43-depth8.v1|"
        << AL_R20_R42_SEMANTIC_SHA256 << ':' << route << ':'
        << trace.certificate.root << ':' << torsion.root << ':'
        << final_step_root;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r43_depth8.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r42_sha256\":\""
        << AL_R20_R42_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"observation_exact\":"
        << (observation_exact ? "true" : "false")
        << ",\"hook_slot_clear\":"
        << (hook_slot_clear ? "true" : "false")
        << ",\"hook_slot_released\":"
        << (hook_slot_released ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"hook\":{\"attempts\":" << trace.attempts
        << ",\"target_matches\":" << trace.target_matches
        << ",\"replacements\":" << trace.replacements
        << ",\"non_target_attempts\":" << trace.non_target_attempts
        << ",\"target_material_root\":\""
        << trace.target_material_root
        << "\",\"non_target_dimension\":" << trace.non_target_dimension
        << ",\"non_target_material_root\":\""
        << trace.non_target_material_root
        << "\",\"non_target_matrix_root\":\""
        << trace.non_target_matrix_root
        << "\",\"non_target_inverse_root\":\""
        << trace.non_target_inverse_root
        << "\",\"non_target_rhs_root\":\"" << trace.non_target_rhs_root
        << "\",\"non_target_solution_root\":\""
        << trace.non_target_solution_root << "\"}"
        << ",\"certificate\":{\"root\":\"" << trace.certificate.root
        << "\",\"rho_bound\":\""
        << al_binary128_text(trace.certificate.rho_bound)
        << "\",\"g_inf_bound\":\""
        << al_binary128_text(trace.certificate.g_inf_bound)
        << "\",\"radius\":\""
        << al_binary128_text(trace.certificate.radius)
        << "\",\"uniform_error\":\""
        << al_binary128_text(trace.certificate.error)
        << "\",\"positive\":" << trace.certificate.positive
        << ",\"negative\":" << trace.certificate.negative
        << ",\"unresolved\":" << trace.certificate.unresolved
        << ",\"minimum_separation\":\""
        << al_binary128_text(trace.certificate.minimum_separation)
        << "\",\"generation_dots\":" << trace.certificate.center_dots
        << "}"
        << ",\"trajectory\":{\"case_root\":\"" << torsion.root
        << "\",\"case_route\":\"" << torsion.route
        << "\",\"certified\":"
        << (torsion.diagnostic.certified ? "true" : "false")
        << ",\"active_set_rejected\":"
        << (torsion.diagnostic.active_set_rejected ? "true" : "false")
        << ",\"direction_rejected\":"
        << (torsion.diagnostic.direction_rejected ? "true" : "false")
        << ",\"globalization_rejected\":"
        << (torsion.diagnostic.globalization_rejected ? "true" : "false")
        << ",\"accepted_iterations\":"
        << torsion.diagnostic.accepted_iterations
        << ",\"principal_solves\":"
        << torsion.diagnostic.total_principal_solves
        << ",\"transitions\":" << torsion.diagnostic.total_transitions
        << ",\"steps\":" << torsion.diagnostic.steps.size()
        << ",\"final_solve_failure\":\"" << final_solve_failure
        << "\",\"final_step_root\":\"" << final_step_root
        << "\",\"ratio_failure\":\""
        << (final_solve == nullptr ? "NONE" : final_solve->ratio_failure)
        << "\",\"ratio_best_source_row\":"
        << (final_solve == nullptr ? 0U
            : final_solve->ratio_best_source_row)
        << ",\"ratio_competitor_source_row\":"
        << (final_solve == nullptr ? 0U
            : final_solve->ratio_competitor_source_row)
        << ",\"ratio_best_value\":\""
        << al_binary128_text(final_solve == nullptr
            ? static_cast<Binary128>(0.0)
            : final_solve->ratio_best_value)
        << "\",\"ratio_competitor_value\":\""
        << al_binary128_text(final_solve == nullptr
            ? static_cast<Binary128>(0.0)
            : final_solve->ratio_competitor_value)
        << "\",\"ratio_best_bound\":\""
        << al_binary128_text(final_solve == nullptr
            ? static_cast<Binary128>(0.0)
            : final_solve->ratio_best_bound)
        << "\",\"ratio_competitor_bound\":\""
        << al_binary128_text(final_solve == nullptr
            ? static_cast<Binary128>(0.0)
            : final_solve->ratio_competitor_bound)
        << "\",\"ratio_current_error\":\""
        << al_binary128_text(final_solve == nullptr
            ? static_cast<Binary128>(0.0)
            : final_solve->ratio_current_error)
        << "\",\"ratio_candidate_error\":\""
        << al_binary128_text(final_solve == nullptr
            ? static_cast<Binary128>(0.0)
            : final_solve->ratio_candidate_error)
        << "\"}"
        << ",\"solver_executions\":" << solver_executions
        << ",\"additional_factorizations\":0"
        << ",\"additional_inverse_columns\":0"
        << ",\"correction_applies\":" << trace.replacements
        << ",\"counterflow_replays\":0,\"new_trials\":0"
        << ",\"state_updates\":0,\"parameter_changes\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
