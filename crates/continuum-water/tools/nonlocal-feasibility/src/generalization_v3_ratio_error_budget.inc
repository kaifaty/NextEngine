namespace {

constexpr const char* AL_R20_V3_RATIO_DIAGNOSTIC_SHA256 =
    "0ec0fa4a48138ca63abdde07e0c3997051982aa5833721f8b8a8e692a24eb6eb";

std::string al_r20_v3_reproduced_ratio_diagnostic_root(
    const std::vector<ALR20SemismoothCase>& diagnostics) {
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r13-v3-ratio-diagnostic.v1|"
             << AL_R20_V3_FAILURE_DIAGNOSTIC_SHA256
             << ":V3_RATIO_SUBPREDICATE_IDENTIFIED|";
    for (const ALR20SemismoothCase& diagnostic : diagnostics) {
        if (diagnostic.steps.empty()) return {};
        const ALR20NNQPSolve& solve = diagnostic.steps.back().solve;
        semantic << diagnostic.root << ':'
                 << diagnostic.steps.back().root << ':'
                 << solve.ratio_failure << ':'
                 << solve.ratio_best_source_row << ':'
                 << solve.ratio_competitor_source_row << ':'
                 << al_binary128_hex(solve.ratio_best_current) << ':'
                 << al_binary128_hex(solve.ratio_best_candidate) << ':'
                 << al_binary128_hex(solve.ratio_best_denominator) << ':'
                 << al_binary128_hex(solve.ratio_best_value) << ':'
                 << al_binary128_hex(solve.ratio_best_bound) << ':'
                 << al_binary128_hex(solve.ratio_competitor_current) << ':'
                 << al_binary128_hex(solve.ratio_competitor_candidate) << ':'
                 << al_binary128_hex(solve.ratio_competitor_denominator) << ':'
                 << al_binary128_hex(solve.ratio_competitor_value) << ':'
                 << al_binary128_hex(solve.ratio_competitor_bound) << ';';
    }
    return sha256_hex(semantic.str());
}

} // namespace

SplitBoundaryReport run_al_generalization_v3_ratio_error_budget_controls() {
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
        && al_r20_semismooth_controls();
    std::vector<ALR20ManifestCase> cases = al_r20_v3_manifest_cases();
    bool parent_exact = controls_exact
        && cases.size() == AL_R20_V3_PROBLEM_SHA256.size()
        && al_r20_v3_manifest_root(cases) == AL_R20_V3_MANIFEST_SHA256;
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    std::vector<ALR20ProblemData> problems;
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        ALR20ProblemData problem = al_r20_materialize_problem(
            cases[index], &sparse_work, &static_work, &adjacency_work);
        parent_exact = parent_exact && problem.exact
            && index < AL_R20_V3_PROBLEM_SHA256.size()
            && problem.problem_root == AL_R20_V3_PROBLEM_SHA256[index]
            && problem.source_positive == 0U;
        problems.push_back(std::move(problem));
    }
    std::vector<ALR20SemismoothCase> diagnostics;
    bool reproduction_exact = parent_exact;
    bool audits_exact = parent_exact;
    std::size_t solver_executions = 0U;
    std::size_t refined_exact = 0U;
    if (parent_exact)
        for (std::size_t local = 0U;
             local < AL_R20_V3_FAILED_CASES.size(); ++local) {
            const std::size_t index = AL_R20_V3_FAILED_CASES[local];
            ALR20SemismoothCase diagnostic =
                al_r20_semismooth_case(problems[index], true);
            ++solver_executions;
            reproduction_exact = reproduction_exact
                && diagnostic.root == AL_R20_V3_FAILED_CASE_SHA256[local]
                && !diagnostic.steps.empty()
                && diagnostic.steps.back().root
                    == AL_R20_V3_FAILED_STEP_SHA256[local];
            if (diagnostic.steps.empty()) {
                audits_exact = false;
            } else {
                const ALR20NNQPSolve& solve =
                    diagnostic.steps.back().solve;
                audits_exact = audits_exact
                    && solve.ratio_failure == "RATIO_ORDER_AMBIGUOUS"
                    && solve.ratio_inverse_audit.passed
                    && solve.ratio_inverse_audit.column_solves
                        == solve.ratio_inverse_audit.dimension;
                if (solve.ratio_refined_exact) ++refined_exact;
            }
            diagnostics.push_back(std::move(diagnostic));
        }
    reproduction_exact = reproduction_exact
        && solver_executions == AL_R20_V3_FAILED_CASES.size()
        && al_r20_v3_reproduced_ratio_diagnostic_root(diagnostics)
            == AL_R20_V3_RATIO_DIAGNOSTIC_SHA256;
    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    std::string outcome = "AUDIT_REJECTED";
    if (audits_exact) {
        if (refined_exact == diagnostics.size())
            outcome = "CANDIDATE_REFINEMENT_RESOLVES_ALL";
        else if (refined_exact == 0U)
            outcome = "CANDIDATE_REFINEMENT_RESOLVES_NONE";
        else
            outcome = "CANDIDATE_REFINEMENT_RESOLVES_SUBSET";
    }
    const std::string route = !parent_exact
        ? "V3_RATIO_ERROR_BUDGET_PARENT_REJECTED"
        : (!reproduction_exact
            ? "V3_RATIO_ERROR_BUDGET_REPRODUCTION_REJECTED"
            : (!audits_exact ? "V3_RATIO_ERROR_BUDGET_AUDIT_REJECTED"
                             : "V3_RATIO_ERROR_BUDGET_IDENTIFIED"));
    const bool exact = parent_exact && reproduction_exact
        && audits_exact && lifecycle_exact;
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r14-v3-ratio-error-budget.v1|"
             << AL_R20_V3_RATIO_DIAGNOSTIC_SHA256 << ':' << route << ':'
             << outcome << '|';
    for (const ALR20SemismoothCase& diagnostic : diagnostics) {
        const ALR20NNQPSolve& solve = diagnostic.steps.back().solve;
        semantic << diagnostic.root << ':'
                 << solve.ratio_inverse_audit.root << ':'
                 << al_binary128_hex(solve.ratio_current_error) << ':'
                 << al_binary128_hex(solve.ratio_candidate_error) << ':'
                 << al_binary128_hex(solve.ratio_best_round_bound) << ':'
                 << al_binary128_hex(solve.ratio_best_input_bound) << ':'
                 << al_binary128_hex(
                        solve.ratio_competitor_round_bound) << ':'
                 << al_binary128_hex(
                        solve.ratio_competitor_input_bound) << ':'
                 << solve.ratio_refined_exact << ':'
                 << solve.ratio_refined_failure << ':'
                 << al_binary128_hex(solve.ratio_refined_alpha) << ':'
                 << al_binary128_hex(solve.ratio_refined_best_bound) << ':'
                 << al_binary128_hex(
                        solve.ratio_refined_competitor_bound) << ';';
    }
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r14_v3_ratio_error_budget.v1\""
           << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
           << ",\"route\":\"" << route << "\""
           << ",\"outcome\":\"" << outcome << "\""
           << ",\"parent_ratio_diagnostic_sha256\":\""
           << AL_R20_V3_RATIO_DIAGNOSTIC_SHA256
           << "\",\"profile_exact\":"
           << (profile_exact ? "true" : "false")
           << ",\"controls_exact\":"
           << (controls_exact ? "true" : "false")
           << ",\"parent_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"reproduction_exact\":"
           << (reproduction_exact ? "true" : "false")
           << ",\"audits_exact\":"
           << (audits_exact ? "true" : "false")
           << ",\"lifecycle_exact\":"
           << (lifecycle_exact ? "true" : "false")
           << ",\"cases\":[";
    for (std::size_t local = 0U; local < diagnostics.size(); ++local) {
        if (local != 0U) report << ',';
        const std::size_t index = AL_R20_V3_FAILED_CASES[local];
        const ALR20NNQPSolve& solve = diagnostics[local].steps.back().solve;
        const ALR20VerifiedInverse& audit = solve.ratio_inverse_audit;
        report << "{\"name\":\"" << cases[index].fixture.name
               << "\",\"case_sha256\":\"" << diagnostics[local].root
               << "\",\"current_error\":\""
               << al_binary128_text(solve.ratio_current_error)
               << "\",\"candidate_error\":\""
               << al_binary128_text(solve.ratio_candidate_error)
               << "\",\"best_round_bound\":\""
               << al_binary128_text(solve.ratio_best_round_bound)
               << "\",\"best_input_bound\":\""
               << al_binary128_text(solve.ratio_best_input_bound)
               << "\",\"competitor_round_bound\":\""
               << al_binary128_text(
                    solve.ratio_competitor_round_bound)
               << "\",\"competitor_input_bound\":\""
               << al_binary128_text(
                    solve.ratio_competitor_input_bound)
               << "\",\"inverse\":{\"dimension\":" << audit.dimension
               << ",\"column_solves\":" << audit.column_solves
               << ",\"rho\":\"" << al_binary128_text(audit.norm.rho)
               << "\",\"inverse_norm_bound\":\""
               << al_binary128_text(audit.norm.inverse_norm_bound)
               << "\",\"old_error\":\""
               << al_binary128_text(audit.cheap_error)
               << "\",\"refined_error\":\""
               << al_binary128_text(audit.refined_error)
               << "\",\"root\":\"" << audit.root
               << "\"},\"refined_ratio_exact\":"
               << (solve.ratio_refined_exact ? "true" : "false")
               << ",\"refined_ratio_failure\":\""
               << solve.ratio_refined_failure
               << "\",\"refined_alpha\":\""
               << al_binary128_text(solve.ratio_refined_alpha)
               << "\",\"refined_best_bound\":\""
               << al_binary128_text(solve.ratio_refined_best_bound)
               << "\",\"refined_competitor_bound\":\""
               << al_binary128_text(
                    solve.ratio_refined_competitor_bound) << "\"}";
    }
    report << "]"
              ",\"solver_executions\":" << solver_executions
           << ",\"successful_case_solver_executions\":0"
              ",\"counterfactual_state_updates\":0"
              ",\"solver_decision_changes\":0"
              ",\"timing_admitted\":false,\"solver_authority\":false"
              ",\"runtime_authority\":false"
              ",\"production_authority\":false"
              ",\"result_sha256\":\""
           << result_sha << "\"}";
    return {exact, report.str()};
}
