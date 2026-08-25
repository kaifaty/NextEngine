namespace {

constexpr const char* AL_R20_V3_FAILURE_DIAGNOSTIC_SHA256 =
    "99b0b72c964044c6b17e31439fcad4b6001db467d1e8d6aeb27fe972d4647f57";
constexpr std::array<const char*, 2U> AL_R20_V3_FAILED_STEP_SHA256{{
    "b030666ae7293d1b779bb8ec6a2c8b32c6495fb5166580e9d66f7fa782ed6a8b",
    "4cabd7c347ae4cc0a4a656f2c4de21d115b052875b92e862b75f06a168b97a50",
}};

bool al_r20_v3_ratio_failure_named(const std::string& failure) {
    return failure == "INVALID_RATIO_INPUT"
        || failure == "NONPOSITIVE_DENOMINATOR"
        || failure == "RATIO_ORDER_AMBIGUOUS"
        || failure == "ALPHA_OUT_OF_RANGE"
        || failure == "RATIO_SUBPREDICATE_UNRESOLVED";
}

} // namespace

SplitBoundaryReport run_al_generalization_v3_ratio_diagnostic_controls() {
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
    bool observer_exact = parent_exact;
    std::size_t solver_executions = 0U;
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
                observer_exact = false;
            } else {
                const ALR20NNQPSolve& solve =
                    diagnostic.steps.back().solve;
                observer_exact = observer_exact
                    && solve.failure == "BOUNDARY_RATIO_UNRESOLVED"
                    && al_r20_v3_ratio_failure_named(solve.ratio_failure);
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
    const std::string route = !parent_exact
        ? "V3_RATIO_PARENT_REJECTED"
        : (!reproduction_exact ? "V3_RATIO_REPRODUCTION_REJECTED"
            : (observer_exact ? "V3_RATIO_SUBPREDICATE_IDENTIFIED"
                              : "V3_RATIO_OBSERVER_REJECTED"));
    const bool exact = parent_exact && reproduction_exact
        && observer_exact && lifecycle_exact;
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r13-v3-ratio-diagnostic.v1|"
             << AL_R20_V3_FAILURE_DIAGNOSTIC_SHA256 << ':' << route << '|';
    for (std::size_t local = 0U; local < diagnostics.size(); ++local) {
        const ALR20NNQPSolve& solve = diagnostics[local].steps.back().solve;
        semantic << diagnostics[local].root << ':'
                 << diagnostics[local].steps.back().root << ':'
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
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r13_v3_ratio_diagnostic.v1\""
           << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
           << ",\"route\":\"" << route << "\""
           << ",\"parent_failure_diagnostic_sha256\":\""
           << AL_R20_V3_FAILURE_DIAGNOSTIC_SHA256
           << "\",\"profile_exact\":"
           << (profile_exact ? "true" : "false")
           << ",\"controls_exact\":"
           << (controls_exact ? "true" : "false")
           << ",\"parent_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"reproduction_exact\":"
           << (reproduction_exact ? "true" : "false")
           << ",\"observer_exact\":"
           << (observer_exact ? "true" : "false")
           << ",\"lifecycle_exact\":"
           << (lifecycle_exact ? "true" : "false")
           << ",\"cases\":[";
    for (std::size_t local = 0U; local < diagnostics.size(); ++local) {
        if (local != 0U) report << ',';
        const std::size_t index = AL_R20_V3_FAILED_CASES[local];
        const ALR20NNQPSolve& solve = diagnostics[local].steps.back().solve;
        report << "{\"name\":\"" << cases[index].fixture.name
               << "\",\"case_sha256\":\"" << diagnostics[local].root
               << "\",\"step_sha256\":\""
               << diagnostics[local].steps.back().root
               << "\",\"ratio_failure\":\"" << solve.ratio_failure
               << "\",\"best_source_row\":"
               << solve.ratio_best_source_row
               << ",\"competitor_source_row\":"
               << solve.ratio_competitor_source_row
               << ",\"best_current\":\""
               << al_binary128_text(solve.ratio_best_current)
               << "\",\"best_candidate\":\""
               << al_binary128_text(solve.ratio_best_candidate)
               << "\",\"best_denominator\":\""
               << al_binary128_text(solve.ratio_best_denominator)
               << "\",\"best_ratio\":\""
               << al_binary128_text(solve.ratio_best_value)
               << "\",\"best_bound\":\""
               << al_binary128_text(solve.ratio_best_bound)
               << "\",\"competitor_current\":\""
               << al_binary128_text(solve.ratio_competitor_current)
               << "\",\"competitor_candidate\":\""
               << al_binary128_text(solve.ratio_competitor_candidate)
               << "\",\"competitor_denominator\":\""
               << al_binary128_text(solve.ratio_competitor_denominator)
               << "\",\"competitor_ratio\":\""
               << al_binary128_text(solve.ratio_competitor_value)
               << "\",\"competitor_bound\":\""
               << al_binary128_text(solve.ratio_competitor_bound)
               << "\"}";
    }
    report << "]"
              ",\"solver_executions\":" << solver_executions
           << ",\"successful_case_solver_executions\":0"
              ",\"solver_decision_changes\":0"
              ",\"timing_admitted\":false,\"repair_authority\":false"
              ",\"runtime_authority\":false"
              ",\"production_authority\":false"
              ",\"result_sha256\":\""
           << result_sha << "\"}";
    return {exact, report.str()};
}
