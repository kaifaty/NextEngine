namespace {

constexpr const char* AL_R20_V3_EVENT_FORWARD_BOUND_SHA256 =
    "d3544d4255d066245d6055b71baa56e67d70e44fce001ba1da83844a4e62d449";

} // namespace

SplitBoundaryReport run_al_generalization_v3_certified_event_trajectory_controls() {
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
        && al_r20_r21_event_controls();
    std::vector<ALR20R17Case> cases = al_r20_r17_cases();
    const bool manifests_exact = cases.size()
            == AL_R20_PROBLEM_SHA256.size()
                + AL_R20_V3_PROBLEM_SHA256.size()
        && al_r20_manifest_semantic_root(al_r20_manifest_cases())
            == AL_R20_MANIFEST_SHA256
        && al_r20_v3_manifest_root(al_r20_v3_manifest_cases())
            == AL_R20_V3_MANIFEST_SHA256;
    bool parent_exact = controls_exact && manifests_exact;
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    std::vector<ALR20ProblemData> problems;
    for (ALR20R17Case& value : cases) {
        ALR20ProblemData problem = al_r20_materialize_problem(
            value.source, &sparse_work, &static_work, &adjacency_work);
        parent_exact = parent_exact && problem.exact
            && value.expected_problem_root != nullptr
            && problem.problem_root == value.expected_problem_root
            && problem.source_positive == 0U;
        problems.push_back(std::move(problem));
    }
    std::vector<ALR20SemismoothCase> diagnostics;
    bool harness_exact = parent_exact;
    bool all_certified = parent_exact;
    bool solver_rejected = false;
    std::size_t solver_executions = 0U;
    std::size_t total_replacements = 0U;
    std::size_t total_fallbacks = 0U;
    std::size_t total_extra_evaluations = 0U;
    std::size_t total_line_evaluations = 0U;
    std::size_t total_dual_refinements = 0U;
    std::size_t total_ratio_inverse_columns = 0U;
    if (parent_exact)
        for (const ALR20ProblemData& problem : problems) {
            ALR20SemismoothCase diagnostic =
                al_r20_semismooth_case(problem, true, true, true);
            ++solver_executions;
            harness_exact = harness_exact && diagnostic.harness_exact;
            all_certified = all_certified && diagnostic.certified;
            solver_rejected = solver_rejected
                || diagnostic.active_set_rejected
                || diagnostic.direction_rejected
                || diagnostic.globalization_rejected;
            total_replacements += diagnostic.event_replacements;
            total_fallbacks += diagnostic.event_fallbacks;
            total_extra_evaluations += diagnostic.event_extra_evaluations;
            for (const ALR20SemismoothStep& step : diagnostic.steps) {
                total_line_evaluations += step.trials.size();
                total_dual_refinements += step.solve.dual_ratio_refinements;
                total_ratio_inverse_columns += step.solve.ratio_inverse_columns;
            }
            diagnostics.push_back(std::move(diagnostic));
        }
    harness_exact = harness_exact && solver_executions == cases.size();
    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    harness_exact = harness_exact && lifecycle_exact;
    const std::string route = !parent_exact
        ? "V3_CERTIFIED_EVENT_PARENT_REJECTED"
        : (solver_rejected ? "V3_CERTIFIED_EVENT_SOLVER_REJECTED"
            : (all_certified ? "CERTIFIED_EVENT_TRAJECTORY_CANDIDATE"
                             : "V3_CERTIFIED_EVENT_UNRESOLVED"));
    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r26-v3-certified-event-trajectory.v1|"
        << AL_R20_V3_EVENT_FORWARD_BOUND_SHA256 << ':' << route << '|';
    for (std::size_t index = 0U; index < diagnostics.size(); ++index)
        semantic << cases[index].family << ':'
            << problems[index].problem_root << ':'
            << diagnostics[index].root << ';';
    semantic << total_replacements << ':' << total_fallbacks << ':'
        << total_extra_evaluations << ':' << total_line_evaluations << ':'
        << total_dual_refinements << ':' << total_ratio_inverse_columns;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r26_v3_certified_event_trajectory.v1\""
        << ",\"status\":\"" << (harness_exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_event_forward_bound_sha256\":\""
        << AL_R20_V3_EVENT_FORWARD_BOUND_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"manifests_exact\":" << (manifests_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"lifecycle_exact\":" << (lifecycle_exact ? "true" : "false")
        << ",\"all_certified\":" << (all_certified ? "true" : "false")
        << ",\"solver_rejected\":" << (solver_rejected ? "true" : "false")
        << ",\"cases\":[";
    for (std::size_t index = 0U; index < diagnostics.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20SemismoothCase& value = diagnostics[index];
        std::size_t line_evaluations = 0U;
        std::size_t dual_refinements = 0U;
        std::size_t ratio_inverse_columns = 0U;
        for (const ALR20SemismoothStep& step : value.steps) {
            line_evaluations += step.trials.size();
            dual_refinements += step.solve.dual_ratio_refinements;
            ratio_inverse_columns += step.solve.ratio_inverse_columns;
        }
        const std::string case_route = value.certified
            ? "CERTIFIED"
            : (value.active_set_rejected ? "ACTIVE_SET_REJECTED"
                : (value.direction_rejected ? "DIRECTION_REJECTED"
                    : (value.globalization_rejected
                        ? "GLOBALIZATION_REJECTED" : "CAP_UNRESOLVED")));
        report << "{\"family\":\"" << cases[index].family
            << "\",\"name\":\"" << cases[index].source.fixture.name
            << "\",\"role\":\"" << cases[index].source.role
            << "\",\"problem_sha256\":\"" << problems[index].problem_root
            << "\",\"route\":\"" << case_route
            << "\",\"certified\":" << (value.certified ? "true" : "false")
            << ",\"accepted_iterations\":" << value.accepted_iterations
            << ",\"step_count\":" << value.steps.size()
            << ",\"event_replacements\":" << value.event_replacements
            << ",\"event_fallbacks\":" << value.event_fallbacks
            << ",\"event_extra_evaluations\":"
            << value.event_extra_evaluations
            << ",\"line_evaluations\":" << line_evaluations
            << ",\"total_mask_changes\":" << value.total_mask_changes
            << ",\"total_principal_solves\":"
            << value.total_principal_solves
            << ",\"total_transitions\":" << value.total_transitions
            << ",\"dual_ratio_refinements\":" << dual_refinements
            << ",\"ratio_inverse_columns\":" << ratio_inverse_columns
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
        << ",\"solver_executions\":" << solver_executions
        << ",\"event_replacements\":" << total_replacements
        << ",\"event_fallbacks\":" << total_fallbacks
        << ",\"event_extra_evaluations\":" << total_extra_evaluations
        << ",\"line_evaluations\":" << total_line_evaluations
        << ",\"dual_ratio_refinements\":" << total_dual_refinements
        << ",\"ratio_inverse_columns\":" << total_ratio_inverse_columns
        << ",\"accepted_iteration_cap\":32"
        << ",\"default_mode_unchanged_by_construction\":true"
        << ",\"default_r8_external_recheck_required\":true"
        << ",\"r17_external_recheck_required\":true"
        << ",\"r25_external_recheck_required\":true"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {harness_exact, report.str()};
}
