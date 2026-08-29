namespace {

constexpr const char* AL_R20_V3_PREFLIGHT_SHA256 =
    "c2328bfb4c6b0ed90a66274a03b5abe51535918c513b3f56693d82ff4c87b577";
constexpr std::array<const char*, 4U> AL_R20_V3_PROBLEM_SHA256{{
    "3dc846e59193dd0162a96d9728c313b65b2cf83bcbbe50b3d71d9da5a373dee3",
    "9911ae713962919d0c0f31e1d595783be8c773370d3c87cd052f4fc247cd8c51",
    "08c636f4bf6fbbb55ca97ebada500d387d7071b3f00194b97b3493e11ed68b62",
    "948f5abd99a54feb83d319e0b7982518842680e91c33983581182736aef8ab22",
}};

} // namespace

SplitBoundaryReport run_al_generalization_v3_solver_controls() {
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
    std::vector<ALR20SemismoothCase> diagnostics;
    bool problems_exact = parent_exact;
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        ALR20ProblemData problem = al_r20_materialize_problem(
            cases[index], &sparse_work, &static_work, &adjacency_work);
        const bool problem_exact = problem.exact
            && index < AL_R20_V3_PROBLEM_SHA256.size()
            && problem.problem_root == AL_R20_V3_PROBLEM_SHA256[index]
            && problem.source_positive == 0U;
        problems_exact = problems_exact && problem_exact;
        problems.push_back(std::move(problem));
    }
    parent_exact = parent_exact && problems_exact;
    bool harness_exact = parent_exact;
    bool all_certified = parent_exact;
    bool solver_rejected = false;
    std::size_t solver_executions = 0U;
    for (const ALR20ProblemData& problem : problems) {
        ALR20SemismoothCase diagnostic;
        if (parent_exact) {
            diagnostic = al_r20_semismooth_case(problem, true);
            ++solver_executions;
        }
        harness_exact = harness_exact && diagnostic.harness_exact;
        all_certified = all_certified && diagnostic.certified;
        solver_rejected = solver_rejected
            || diagnostic.active_set_rejected
            || diagnostic.direction_rejected
            || diagnostic.globalization_rejected;
        diagnostics.push_back(std::move(diagnostic));
    }
    parent_exact = parent_exact
        && solver_executions == AL_R20_V3_PROBLEM_SHA256.size();
    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    harness_exact = harness_exact && lifecycle_exact;
    const std::string route = !parent_exact
        ? "V3_GENERALIZATION_PARENT_REJECTED"
        : (solver_rejected ? "V3_GENERALIZATION_SOLVER_REJECTED"
            : (all_certified ? "V3_GENERALIZATION_CERTIFIED"
                             : "V3_GENERALIZATION_UNRESOLVED"));
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r11-v3-generalization.v1|"
             << AL_R20_V3_PREFLIGHT_SHA256 << ':'
             << AL_R20_VERIFIED_INVERSE_SHA256 << ':' << route << '|';
    for (std::size_t index = 0U; index < diagnostics.size(); ++index)
        semantic << problems[index].problem_root << ':'
                 << diagnostics[index].root << ';';
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r11_v3_generalization.v1\""
           << ",\"status\":\"" << (harness_exact ? "PASS" : "FAIL") << "\""
           << ",\"route\":\"" << route << "\""
           << ",\"parent_v3_preflight_sha256\":\""
           << AL_R20_V3_PREFLIGHT_SHA256
           << "\",\"solver_implementation_sha256\":\""
           << AL_R20_VERIFIED_INVERSE_SHA256
           << "\",\"profile_exact\":"
           << (profile_exact ? "true" : "false")
           << ",\"controls_exact\":"
           << (controls_exact ? "true" : "false")
           << ",\"parent_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"lifecycle_exact\":"
           << (lifecycle_exact ? "true" : "false")
           << ",\"all_certified\":"
           << (all_certified ? "true" : "false")
           << ",\"solver_rejected\":"
           << (solver_rejected ? "true" : "false")
           << ",\"cases\":[";
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20SemismoothCase& value = diagnostics[index];
        std::size_t inverse_audits = 0U;
        std::size_t inverse_column_solves = 0U;
        for (const ALR20SemismoothStep& step : value.steps)
            for (const ALR20VerifiedInverse& audit :
                 step.solve.inverse_audits) {
                ++inverse_audits;
                inverse_column_solves += audit.column_solves;
            }
        report << "{\"name\":\"" << cases[index].fixture.name
               << "\",\"role\":\"" << cases[index].role
               << "\",\"problem_sha256\":\""
               << problems[index].problem_root
               << "\",\"certified\":"
               << (value.certified ? "true" : "false")
               << ",\"active_set_rejected\":"
               << (value.active_set_rejected ? "true" : "false")
               << ",\"direction_rejected\":"
               << (value.direction_rejected ? "true" : "false")
               << ",\"globalization_rejected\":"
               << (value.globalization_rejected ? "true" : "false")
               << ",\"accepted_iterations\":"
               << value.accepted_iterations
               << ",\"step_count\":" << value.steps.size()
               << ",\"total_mask_changes\":" << value.total_mask_changes
               << ",\"total_principal_solves\":"
               << value.total_principal_solves
               << ",\"total_transitions\":" << value.total_transitions
               << ",\"inverse_audit_count\":" << inverse_audits
               << ",\"inverse_column_solves\":"
               << inverse_column_solves
               << ",\"final_primal\":\""
               << al_binary128_text(value.final_metrics.primal)
               << "\",\"final_dual_mapping\":\""
               << al_binary128_text(value.final_metrics.dual_mapping)
               << "\",\"final_complementarity\":\""
               << al_binary128_text(value.final_metrics.complementarity)
               << "\",\"final_stationarity\":\""
               << al_binary128_text(value.final_metrics.stationarity)
               << "\",\"final_gap_scaled\":\""
               << al_binary128_text(value.final_metrics.gap_scaled)
               << "\",\"root\":\"" << value.root << "\"}";
    }
    report << "]"
              ",\"solver_executions\":" << solver_executions
           << ",\"accepted_iteration_cap\":32"
              ",\"verified_inverse_on_demand\":true"
              ",\"old_development_solver_reruns\":0"
              ",\"parameter_changes_after_observation\":0"
              ",\"timing_admitted\":false"
              ",\"solver_generalization_authority\":"
           << (route == "V3_GENERALIZATION_CERTIFIED" ? "true" : "false")
           << ",\"runtime_authority\":false"
              ",\"production_authority\":false"
              ",\"result_sha256\":\""
           << result_sha << "\"}";
    return {harness_exact, report.str()};
}
