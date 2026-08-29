namespace {

constexpr const char* AL_R20_R48_SEMANTIC_SHA256 =
    "7d4d7a9e3765274338707855df4953d61c44b6a3d15c9362bf12b70eccbe7802";
constexpr const char* AL_R20_R48_DEPTH16_SHA256 =
    "f0eade1dd3db1bcc8ca6a6413144e7c132694c141e737ca8375af57420130441";

struct ALR20R49HookTrace {
    std::size_t attempts = 0U;
    std::size_t first_matches = 0U;
    std::size_t first_replacements = 0U;
    std::size_t second_matches = 0U;
    std::size_t second_replacements = 0U;
    std::size_t third_matches = 0U;
    std::size_t third_replacements = 0U;
    std::size_t unknown_attempts = 0U;
    ALR20R42PracticalCertificate first_certificate;
    ALR20R42PracticalCertificate second_certificate;
    ALR20R42PracticalCertificate third_certificate;
    bool second_values_exact = false;
    bool third_values_exact = false;
    std::size_t unknown_dimension = 0U;
    std::string unknown_material_root;
    std::string unknown_matrix_root;
    std::string unknown_inverse_root;
    std::string unknown_rhs_root;
    std::string unknown_solution_root;
};

thread_local ALR20R49HookTrace* al_r20_r49_hook_trace = nullptr;

ALR20VerifiedSolutionRefinement al_r20_r49_result(
    const ALR20R42PracticalCertificate& certificate,
    bool values_exact) {
    ALR20VerifiedSolutionRefinement result;
    result.exact = certificate.exact;
    result.passed = certificate.passed && values_exact;
    result.error = certificate.error;
    result.solution = certificate.solution;
    result.root = certificate.root;
    return result;
}

ALR20VerifiedSolutionRefinement al_r20_r49_refiner(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& inverse,
    const std::vector<Binary128>& rhs,
    const std::vector<Binary128>& solution) {
    ALR20VerifiedSolutionRefinement result;
    ALR20R49HookTrace* trace = al_r20_r49_hook_trace;
    if (trace == nullptr) return result;
    ++trace->attempts;
    const std::string matrix_root = al_r20_q_scalar_root(matrix);
    const std::string inverse_root = al_r20_q_scalar_root(inverse);
    const std::string rhs_root = al_r20_q_scalar_root(rhs);
    const std::string solution_root = al_r20_q_scalar_root(solution);
    const std::string material_root = sha256_hex(matrix_root + ':'
        + inverse_root + ':' + rhs_root + ':' + solution_root);
    const bool first_target = matrix_root == AL_R20_R37_MATRIX_SHA256
        && inverse_root == AL_R20_R37_INVERSE_SHA256
        && rhs_root == AL_R20_R39_RHS_SHA256
        && solution_root == AL_R20_R39_SOLUTION_SHA256;
    if (first_target) {
        ++trace->first_matches;
        if (trace->first_replacements == 0U
            && trace->second_replacements == 0U
            && trace->third_replacements == 0U) {
            trace->first_certificate = al_r20_r42_practical_certificate(
                matrix, inverse, rhs, solution, 8U);
            result = al_r20_r49_result(trace->first_certificate, true);
            if (result.passed) ++trace->first_replacements;
            return result;
        }
    }
    const bool second_target = rhs.size() == 65U
        && material_root == AL_R20_R43_SECOND_MATERIAL_SHA256
        && matrix_root == AL_R20_R43_SECOND_MATRIX_SHA256
        && inverse_root == AL_R20_R43_SECOND_INVERSE_SHA256
        && rhs_root == AL_R20_R43_SECOND_RHS_SHA256
        && solution_root == AL_R20_R43_SECOND_SOLUTION_SHA256;
    if (second_target) {
        ++trace->second_matches;
        if (trace->first_replacements == 1U
            && trace->second_replacements == 0U
            && trace->third_replacements == 0U) {
            trace->second_certificate = al_r20_r42_practical_certificate(
                matrix, inverse, rhs, solution, 16U, false);
            trace->second_values_exact =
                trace->second_certificate.radius == strtoflt128(
                    "3.165878099507370786634664689978870019e-14", nullptr)
                && trace->second_certificate.positive == 29U
                && trace->second_certificate.negative == 36U
                && trace->second_certificate.unresolved == 0U
                && trace->second_certificate.minimum_separation
                    > static_cast<Binary128>(0.0);
            result = al_r20_r49_result(
                trace->second_certificate, trace->second_values_exact);
            if (result.passed) ++trace->second_replacements;
            return result;
        }
    }
    const bool third_target = rhs.size() == 65U
        && material_root == AL_R20_R46_THIRD_MATERIAL_SHA256
        && matrix_root == AL_R20_R46_THIRD_MATRIX_SHA256
        && inverse_root == AL_R20_R46_THIRD_INVERSE_SHA256
        && rhs_root == AL_R20_R46_THIRD_RHS_SHA256
        && solution_root == AL_R20_R46_THIRD_SOLUTION_SHA256;
    if (third_target) {
        ++trace->third_matches;
        if (trace->first_replacements == 1U
            && trace->second_replacements == 1U
            && trace->third_replacements == 0U) {
            trace->third_certificate = al_r20_r42_practical_certificate(
                matrix, inverse, rhs, solution, 16U, false);
            trace->third_values_exact =
                trace->third_certificate.radius == strtoflt128(
                    "6.492670105723917989289204994889832681e-14", nullptr)
                && trace->third_certificate.positive == 32U
                && trace->third_certificate.negative == 33U
                && trace->third_certificate.unresolved == 0U
                && trace->third_certificate.minimum_separation
                    > static_cast<Binary128>(0.0);
            result = al_r20_r49_result(
                trace->third_certificate, trace->third_values_exact);
            if (result.passed) ++trace->third_replacements;
            return result;
        }
    }
    if (!first_target && !second_target && !third_target) {
        ++trace->unknown_attempts;
        if (trace->unknown_attempts == 1U) {
            trace->unknown_dimension = rhs.size();
            trace->unknown_material_root = material_root;
            trace->unknown_matrix_root = matrix_root;
            trace->unknown_inverse_root = inverse_root;
            trace->unknown_rhs_root = rhs_root;
            trace->unknown_solution_root = solution_root;
        }
    }
    result.exact = true;
    result.root = sha256_hex(std::string("r49-no-refinement:")
        + material_root + ':' + std::to_string(trace->first_replacements)
        + ':' + std::to_string(trace->second_replacements) + ':'
        + std::to_string(trace->third_replacements));
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v4_three_refinement_trajectory_controls() {
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
    ALR20R49HookTrace trace;
    ALR20R35CaseAudit torsion;
    std::size_t solver_executions = 0U;
    const bool hook_slot_clear = al_r20_verified_solution_refiner == nullptr
        && al_r20_r49_hook_trace == nullptr;
    if (parent_exact && hook_slot_clear) {
        al_r20_r49_hook_trace = &trace;
        al_r20_verified_solution_refiner = &al_r20_r49_refiner;
        torsion = al_r20_r35_case(problem);
        al_r20_verified_solution_refiner = nullptr;
        al_r20_r49_hook_trace = nullptr;
        solver_executions = 1U;
    }
    const bool hook_slot_released = al_r20_verified_solution_refiner == nullptr
        && al_r20_r49_hook_trace == nullptr;
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool observation_exact = parent_exact && hook_slot_clear
        && hook_slot_released && solver_executions == 1U
        && al_r20_r35_case_observation_exact(torsion);
    const bool first_exact = trace.first_certificate.exact
        && trace.first_certificate.passed
        && trace.first_certificate.no_underflow
        && trace.first_certificate.root == AL_R20_R43_CERTIFICATE_SHA256;
    const bool second_exact = trace.second_certificate.exact
        && trace.second_certificate.passed
        && trace.second_certificate.no_underflow
        && trace.second_values_exact
        && trace.second_certificate.root
            == AL_R20_R46_SECOND_CERTIFICATE_SHA256;
    const bool third_exact = trace.third_certificate.exact
        && trace.third_certificate.passed
        && trace.third_certificate.no_underflow
        && trace.third_values_exact;
    const bool cardinality_exact = trace.attempts >= 3U
        && trace.first_matches == 1U && trace.first_replacements == 1U
        && trace.second_matches == 1U && trace.second_replacements == 1U
        && trace.third_matches == 1U && trace.third_replacements == 1U;
    const bool consumed = cardinality_exact
        && !torsion.diagnostic.steps.empty()
        && torsion.diagnostic.total_principal_solves > 0U
        && torsion.diagnostic.total_transitions > 0U;
    const ALR20SemismoothStep* final_step = torsion.diagnostic.steps.empty()
        ? nullptr : &torsion.diagnostic.steps.back();
    const ALR20NNQPSolve* final_solve = final_step == nullptr
        ? nullptr : &final_step->solve;
    const bool later_inverse_boundary = final_solve != nullptr
        && final_solve->failure == "VERIFIED_INVERSE_AUDIT_REJECTED";
    const bool exact = observation_exact && first_exact && second_exact
        && third_exact && cardinality_exact && consumed && lifecycle_exact;
    const std::string route = !parent_exact || !observation_exact
        ? "THREE_REFINEMENT_PARENT_REJECTED"
        : (!first_exact
            ? "THREE_REFINEMENT_FIRST_REJECTED"
            : (!second_exact
                ? "THREE_REFINEMENT_SECOND_REJECTED"
                : (!third_exact
                    ? "THREE_REFINEMENT_THIRD_REJECTED"
                    : (!cardinality_exact
                        ? "THREE_REFINEMENT_CARDINALITY_REJECTED"
                        : (!consumed
                            ? "THREE_REFINEMENT_NOT_CONSUMED"
                            : (later_inverse_boundary
                                ? "THREE_REFINEMENT_LATER_INVERSE_BOUNDARY"
                                : (!torsion.diagnostic.certified
                                    ? "THREE_REFINEMENT_LATER_BOUNDARY"
                                    : "THREE_REFINEMENT_TORSION_CANDIDATE")))))));
    const std::string final_step_root = final_step == nullptr
        ? "" : final_step->root;
    const std::string final_solve_failure = final_solve == nullptr
        ? "NONE" : final_solve->failure;
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r49-three-refinement.v1|"
        << AL_R20_R48_SEMANTIC_SHA256 << ':' << route << ':'
        << trace.first_certificate.root << ':'
        << trace.second_certificate.root << ':'
        << trace.third_certificate.root << ':' << torsion.root << ':'
        << final_step_root << ':' << trace.unknown_material_root;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r49_three_refinement.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r48_sha256\":\""
        << AL_R20_R48_SEMANTIC_SHA256 << "\""
        << ",\"parent_r48_depth16_sha256\":\""
        << AL_R20_R48_DEPTH16_SHA256 << "\""
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
        << ",\"first_matches\":" << trace.first_matches
        << ",\"first_replacements\":" << trace.first_replacements
        << ",\"second_matches\":" << trace.second_matches
        << ",\"second_replacements\":" << trace.second_replacements
        << ",\"third_matches\":" << trace.third_matches
        << ",\"third_replacements\":" << trace.third_replacements
        << ",\"unknown_attempts\":" << trace.unknown_attempts
        << ",\"unknown_dimension\":" << trace.unknown_dimension
        << ",\"unknown_material_root\":\""
        << trace.unknown_material_root
        << "\",\"unknown_matrix_root\":\"" << trace.unknown_matrix_root
        << "\",\"unknown_inverse_root\":\"" << trace.unknown_inverse_root
        << "\",\"unknown_rhs_root\":\"" << trace.unknown_rhs_root
        << "\",\"unknown_solution_root\":\""
        << trace.unknown_solution_root << "\"}"
        << ",\"first_certificate\":";
    al_r20_r46_write_certificate(report, trace.first_certificate);
    report << ",\"second_certificate\":";
    al_r20_r46_write_certificate(report, trace.second_certificate);
    report << ",\"third_certificate\":";
    al_r20_r46_write_certificate(report, trace.third_certificate);
    report
        << ",\"second_values_exact\":"
        << (trace.second_values_exact ? "true" : "false")
        << ",\"third_values_exact\":"
        << (trace.third_values_exact ? "true" : "false")
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
        << "\",\"final_step_root\":\"" << final_step_root << "\"}"
        << ",\"solver_executions\":" << solver_executions
        << ",\"additional_factorizations\":0"
        << ",\"additional_inverse_columns\":0"
        << ",\"correction_applies\":"
        << trace.first_replacements + trace.second_replacements
            + trace.third_replacements
        << ",\"counterflow_replays\":0,\"new_trials\":0"
        << ",\"state_updates\":0,\"parameter_changes\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
