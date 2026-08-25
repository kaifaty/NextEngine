namespace {

constexpr const char* AL_R20_V3_RATIO_ERROR_BUDGET_SHA256 =
    "8707531a56963b5ce0a456e33b27d0b831a76061d354188b9a1232d9f959d30c";
constexpr std::array<const char*, 2U> AL_R20_V3_RATIO_INVERSE_SHA256{{
    "d1ea327292d376f27bd1cdc321dc8f9cd2ee86000237bf279bdb02d1d26ac217",
    "e9110976df1a3e2b52800ba71764eefe4998e8fb1265a1db99d747f133618b4d",
}};

} // namespace

SplitBoundaryReport run_al_generalization_v3_affine_shadow_controls() {
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
    bool shadows_exact = parent_exact;
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
                shadows_exact = false;
            } else {
                const ALR20NNQPSolve& solve =
                    diagnostic.steps.back().solve;
                reproduction_exact = reproduction_exact
                    && solve.ratio_inverse_audit.root
                        == AL_R20_V3_RATIO_INVERSE_SHA256[local];
                shadows_exact = shadows_exact
                    && solve.affine_shadow_exact
                    && solve.affine_shadow_updates > 0U
                    && solve.affine_direction_error
                        <= solve.ratio_current_error
                    && solve.ratio_affine_refined_failure != "NOT_RUN"
                    && solve.ratio_affine_refined_failure
                        != "AUDIT_REJECTED";
                if (solve.ratio_affine_refined_exact) ++refined_exact;
            }
            diagnostics.push_back(std::move(diagnostic));
        }
    reproduction_exact = reproduction_exact
        && solver_executions == AL_R20_V3_FAILED_CASES.size();
    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    std::string outcome = "SHADOW_REJECTED";
    if (shadows_exact) {
        if (refined_exact == diagnostics.size())
            outcome = "AFFINE_SHADOW_RESOLVES_ALL";
        else if (refined_exact == 0U)
            outcome = "AFFINE_SHADOW_RESOLVES_NONE";
        else
            outcome = "AFFINE_SHADOW_RESOLVES_SUBSET";
    }
    const std::string route = !parent_exact
        ? "V3_AFFINE_SHADOW_PARENT_REJECTED"
        : (!reproduction_exact ? "V3_AFFINE_SHADOW_REPRODUCTION_REJECTED"
            : (shadows_exact ? "V3_AFFINE_SHADOW_IDENTIFIED"
                             : "V3_AFFINE_SHADOW_REJECTED"));
    const bool exact = parent_exact && reproduction_exact
        && shadows_exact && lifecycle_exact;
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r15-v3-affine-shadow.v1|"
             << AL_R20_V3_RATIO_ERROR_BUDGET_SHA256 << ':' << route << ':'
             << outcome << '|';
    for (const ALR20SemismoothCase& diagnostic : diagnostics) {
        const ALR20NNQPSolve& solve = diagnostic.steps.back().solve;
        semantic << diagnostic.root << ':'
                 << diagnostic.steps.back().root << ':'
                 << solve.ratio_inverse_audit.root << ':'
                 << solve.affine_shadow_exact << ':'
                 << solve.affine_shadow_updates << ':'
                 << al_binary128_hex(solve.ratio_current_error) << ':'
                 << al_binary128_hex(solve.affine_direction_error) << ':'
                 << solve.ratio_affine_refined_exact << ':'
                 << solve.ratio_affine_refined_failure << ':'
                 << al_binary128_hex(solve.ratio_affine_refined_alpha) << ':'
                 << al_binary128_hex(
                        solve.ratio_affine_refined_best_bound) << ':'
                 << al_binary128_hex(
                        solve.ratio_affine_refined_competitor_bound) << ';';
    }
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r15_v3_affine_shadow.v1\""
           << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
           << ",\"route\":\"" << route << "\""
           << ",\"outcome\":\"" << outcome << "\""
           << ",\"parent_ratio_error_budget_sha256\":\""
           << AL_R20_V3_RATIO_ERROR_BUDGET_SHA256
           << "\",\"profile_exact\":"
           << (profile_exact ? "true" : "false")
           << ",\"controls_exact\":"
           << (controls_exact ? "true" : "false")
           << ",\"parent_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"reproduction_exact\":"
           << (reproduction_exact ? "true" : "false")
           << ",\"shadows_exact\":"
           << (shadows_exact ? "true" : "false")
           << ",\"lifecycle_exact\":"
           << (lifecycle_exact ? "true" : "false")
           << ",\"cases\":[";
    for (std::size_t local = 0U; local < diagnostics.size(); ++local) {
        if (local != 0U) report << ',';
        const std::size_t index = AL_R20_V3_FAILED_CASES[local];
        const ALR20NNQPSolve& solve = diagnostics[local].steps.back().solve;
        report << "{\"name\":\"" << cases[index].fixture.name
               << "\",\"case_sha256\":\"" << diagnostics[local].root
               << "\",\"inverse_sha256\":\""
               << solve.ratio_inverse_audit.root
               << "\",\"shadow_updates\":"
               << solve.affine_shadow_updates
               << ",\"old_current_error\":\""
               << al_binary128_text(solve.ratio_current_error)
               << "\",\"affine_current_error\":\""
               << al_binary128_text(solve.affine_direction_error)
               << "\",\"refined_candidate_error\":\""
               << al_binary128_text(
                    solve.ratio_inverse_audit.refined_error)
               << "\",\"affine_ratio_exact\":"
               << (solve.ratio_affine_refined_exact ? "true" : "false")
               << ",\"affine_ratio_failure\":\""
               << solve.ratio_affine_refined_failure
               << "\",\"affine_alpha\":\""
               << al_binary128_text(solve.ratio_affine_refined_alpha)
               << "\",\"affine_best_bound\":\""
               << al_binary128_text(
                    solve.ratio_affine_refined_best_bound)
               << "\",\"affine_competitor_bound\":\""
               << al_binary128_text(
                    solve.ratio_affine_refined_competitor_bound) << "\"}";
    }
    report << "]"
              ",\"solver_executions\":" << solver_executions
           << ",\"successful_case_solver_executions\":0"
              ",\"shadow_state_consumers\":0"
              ",\"solver_decision_changes\":0"
              ",\"timing_admitted\":false,\"solver_authority\":false"
              ",\"runtime_authority\":false"
              ",\"production_authority\":false"
              ",\"result_sha256\":\""
           << result_sha << "\"}";
    return {exact, report.str()};
}
