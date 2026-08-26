namespace {

constexpr const char* AL_R20_R31_SEMANTIC_SHA256 =
    "3425772c1e7480d690a31b52e2456857b17feb15f9967892e3830b01c907de5b";
constexpr const char* AL_R20_R31_POWER0_AUDIT_SHA256 =
    "e36e8d5eb24e6aa788f1e4588e1eca3e725f83b1e029dba8f3f47261fd8926d7";
constexpr const char* AL_R20_R31_POWER0_METRICS_SHA256 =
    "05b71f6fed26a95ac1a3e215fcb4b5d8b1adc1b09842d4340d5a3ced67529b53";

struct ALR20R32TerminalAudit {
    bool eligible = false;
    bool selected = false;
    bool state_correspondence = false;
    bool fresh_certificate = false;
    bool exact = false;
    std::size_t certified_trials = 0U;
    std::size_t selected_power = AL_R20_NEWTON_LINE_TRIALS;
    ALR20NewtonTrial trial;
    ALR20R31TrialAudit r31;
    ALR20NewtonDualEvaluation fresh_dual;
    ALR20OracleMetrics fresh_metrics;
    std::vector<Binary128> terminal_lambda;
    std::string terminal_lambda_root;
    std::string root;
};

ALR20R32TerminalAudit al_r20_r32_terminal_audit(
    const ALR20ProblemData& problem,
    const ALR20R29CaseAudit& parent) {
    ALR20R32TerminalAudit audit;
    const ALR20SemismoothCase& diagnostic = parent.diagnostic;
    audit.eligible = parent.root == AL_R20_R29_SHEAR_SHA256
        && parent.recovery_applied && parent.recovery_correspondence
        && diagnostic.harness_exact && !diagnostic.certified
        && diagnostic.globalization_rejected
        && diagnostic.accepted_iterations == 21U
        && diagnostic.steps.size() == 22U
        && diagnostic.final_lambda.size() == problem.row_scale.size();
    if (!audit.eligible) return audit;
    const ALR20NewtonDualEvaluation& current =
        diagnostic.steps[20U].selected.dual;
    const ALR20SemismoothStep& final = diagnostic.steps[21U];
    audit.eligible = final.iteration == 22U
        && final.direction_certified && final.solve.kkt_certified
        && !final.accepted
        && final.trials.size() == AL_R20_NEWTON_LINE_TRIALS;
    if (!audit.eligible) return audit;

    const ALR20NewtonTrial* selected = nullptr;
    for (const ALR20NewtonTrial& trial : final.trials)
        if (trial.metrics.certified) {
            ++audit.certified_trials;
            if (selected == nullptr) selected = &trial;
        }
    if (selected == nullptr) return audit;
    audit.trial = *selected;
    audit.selected_power = selected->power;
    audit.r31 = al_r20_r31_trial_audit(current, final, *selected);

    audit.terminal_lambda.resize(problem.row_scale.size());
    for (std::size_t row = 0U; row < audit.terminal_lambda.size(); ++row)
        audit.terminal_lambda[row] =
            (static_cast<Binary128>(1.0) - selected->alpha)
                * diagnostic.final_lambda[row]
            + selected->alpha * final.line_representative[row];
    audit.terminal_lambda_root =
        al_r20_q_scalar_root(audit.terminal_lambda);
    audit.fresh_dual = al_r20_newton_dual_evaluate(
        problem, audit.terminal_lambda);
    audit.fresh_metrics = al_r20_q_metrics(problem,
        audit.fresh_dual.projector.value, audit.terminal_lambda,
        al_r20_q_diagonal(problem.op), final.iteration);
    audit.state_correspondence = audit.fresh_dual.exact
        && audit.fresh_dual.root == selected->dual.root
        && audit.fresh_metrics.root == selected->metrics.root
        && audit.terminal_lambda_root == selected->metrics.lambda_root;
    audit.fresh_certificate = audit.fresh_metrics.finite
        && audit.fresh_metrics.certified;
    audit.selected = audit.certified_trials == 1U
        && audit.selected_power == 0U
        && selected->alpha == static_cast<Binary128>(1.0)
        && selected->root.size() == 64U
        && audit.r31.root == AL_R20_R31_POWER0_AUDIT_SHA256
        && selected->metrics.root == AL_R20_R31_POWER0_METRICS_SHA256
        && audit.r31.exact && audit.r31.certificate_consistent
        && audit.r31.margin < static_cast<Binary128>(0.0)
        && selected->mask_changes == 0U
        && !selected->ball_activity_changed
        && audit.state_correspondence && audit.fresh_certificate;
    audit.exact = audit.eligible && audit.selected;
    std::ostringstream material;
    material << audit.eligible << ':' << audit.selected << ':'
        << audit.state_correspondence << ':' << audit.fresh_certificate << ':'
        << audit.certified_trials << ':' << audit.selected_power << ':'
        << audit.trial.root << ':' << audit.r31.root << ':'
        << audit.fresh_dual.root << ':' << audit.fresh_metrics.root << ':'
        << audit.terminal_lambda_root;
    audit.root = sha256_hex(material.str());
    return audit;
}

} // namespace

SplitBoundaryReport run_al_generalization_v3_terminal_trajectory_controls() {
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

    const std::size_t shear_case_index =
        AL_R20_PROBLEM_SHA256.size() + AL_R20_V3_SHEAR_INDEX;
    ALR20SemismoothCase parent_shear;
    ALR20R29CaseAudit shear_parent;
    std::vector<ALR20SemismoothCase> diagnostics;
    std::size_t solver_executions = 0U;
    if (parent_exact) {
        parent_shear = al_r20_semismooth_case(
            problems[shear_case_index], true, true, true);
        ++solver_executions;
        parent_exact = parent_shear.root
                == AL_R20_V3_CERTIFIED_EVENT_SHEAR_SHA256
            && parent_shear.steps.size() == 20U
            && parent_shear.accepted_iterations == 19U
            && parent_shear.globalization_rejected;
    }
    if (parent_exact)
        for (std::size_t index = 0U; index < problems.size(); ++index) {
            if (index == shear_case_index) {
                shear_parent = al_r20_r29_shear_case(problems[index]);
                diagnostics.push_back(shear_parent.diagnostic);
            } else {
                diagnostics.push_back(al_r20_semismooth_case(
                    problems[index], true, true, true));
            }
            ++solver_executions;
        }

    bool prefix_exact = parent_exact
        && shear_parent.diagnostic.steps.size() >= 20U
        && shear_parent.rejected_step_root == parent_shear.steps[19U].root;
    if (prefix_exact)
        for (std::size_t index = 0U; index < 19U; ++index)
            prefix_exact = prefix_exact
                && shear_parent.diagnostic.steps[index].root
                    == parent_shear.steps[index].root;
    bool r29_harness_exact = parent_exact && prefix_exact
        && shear_parent.diagnostic.harness_exact
        && shear_parent.recovery_eligible
        && shear_parent.recovery_applied
        && shear_parent.recovery_correspondence
        && shear_parent.recovery_iteration == 20U
        && shear_parent.recovery_scalar == 168U
        && shear_parent.recovery_trial.root == AL_R20_R28_NEW_TRIAL_SHA256;
    bool r29_all_certified = parent_exact;
    bool r29_solver_rejected = false;
    std::size_t total_replacements = 0U;
    std::size_t total_fallbacks = 0U;
    std::size_t total_line_evaluations = 0U;
    for (const ALR20SemismoothCase& diagnostic : diagnostics) {
        r29_harness_exact = r29_harness_exact && diagnostic.harness_exact;
        r29_all_certified = r29_all_certified && diagnostic.certified;
        r29_solver_rejected = r29_solver_rejected
            || diagnostic.active_set_rejected
            || diagnostic.direction_rejected
            || diagnostic.globalization_rejected;
        total_replacements += diagnostic.event_replacements;
        total_fallbacks += diagnostic.event_fallbacks;
        for (const ALR20SemismoothStep& step : diagnostic.steps)
            total_line_evaluations += step.trials.size();
    }
    r29_harness_exact = r29_harness_exact
        && diagnostics.size() == cases.size()
        && solver_executions == cases.size() + 1U;
    const std::string r29_route = !parent_exact
        ? "V3_EXHAUSTED_RECOVERY_PARENT_REJECTED"
        : (!prefix_exact || !shear_parent.recovery_correspondence
            ? "V3_EXHAUSTED_RECOVERY_CORRESPONDENCE_REJECTED"
            : (r29_solver_rejected
                ? "V3_EXHAUSTED_RECOVERY_SOLVER_REJECTED"
                : (r29_all_certified
                    ? "EXHAUSTED_LINE_RECOVERY_CANDIDATE"
                    : "V3_EXHAUSTED_RECOVERY_CAP_UNRESOLVED")));
    std::ostringstream r29_semantic;
    r29_semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r29-exhausted-line-recovery.v1|"
        << AL_R20_R28_SEMANTIC_SHA256 << ':' << r29_route << '|';
    for (std::size_t index = 0U; index < diagnostics.size(); ++index)
        r29_semantic << cases[index].family << ':'
            << problems[index].problem_root << ':'
            << diagnostics[index].root << ';';
    r29_semantic << parent_shear.root << ':' << shear_parent.root << ':'
        << total_replacements << ':' << total_fallbacks << ':'
        << total_line_evaluations;
    const std::string r29_result_sha = sha256_hex(r29_semantic.str());
    const bool r29_parent_exact = r29_harness_exact
        && r29_result_sha == AL_R20_R29_SEMANTIC_SHA256;

    ALR20R32TerminalAudit terminal;
    if (r29_parent_exact)
        terminal = al_r20_r32_terminal_audit(
            problems[shear_case_index], shear_parent);
    bool historical_roots_exact = r29_parent_exact;
    std::size_t candidate_certified_cases = 0U;
    for (std::size_t index = 0U; index < diagnostics.size(); ++index) {
        if (index == shear_case_index) continue;
        historical_roots_exact = historical_roots_exact
            && diagnostics[index].certified;
        if (diagnostics[index].certified) ++candidate_certified_cases;
    }
    if (terminal.selected && terminal.fresh_certificate)
        ++candidate_certified_cases;
    const bool all_certified = candidate_certified_cases == cases.size();

    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool exact = r29_parent_exact && historical_roots_exact
        && terminal.exact && lifecycle_exact;
    const std::string route = !r29_parent_exact
        ? "TERMINAL_TRAJECTORY_PARENT_REJECTED"
        : (!terminal.eligible
            ? "TERMINAL_TRAJECTORY_BOUNDARY_REJECTED"
            : (terminal.certified_trials == 0U
                ? "TERMINAL_CERTIFICATE_NOT_FOUND"
                : (!terminal.state_correspondence
                    || !terminal.fresh_certificate
                    ? "TERMINAL_STATE_CORRESPONDENCE_REJECTED"
                    : (!historical_roots_exact
                        ? "TERMINAL_TRAJECTORY_REGRESSION"
                        : (all_certified
                            ? "TERMINAL_CERTIFICATE_TRAJECTORY_CANDIDATE"
                            : "TERMINAL_TRAJECTORY_UNRESOLVED")))));

    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r32-terminal-trajectory.v1|"
        << AL_R20_R31_SEMANTIC_SHA256 << ':' << AL_R20_R29_SEMANTIC_SHA256
        << ':' << route << ':' << r29_result_sha << ':' << terminal.root
        << ':' << candidate_certified_cases << '|';
    for (std::size_t index = 0U; index < diagnostics.size(); ++index)
        semantic << cases[index].family << ':'
            << (index == shear_case_index
                    ? terminal.root : diagnostics[index].root)
            << ';';
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r32_terminal_trajectory.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_terminal_certificate_sha256\":\""
        << AL_R20_R31_SEMANTIC_SHA256 << "\""
        << ",\"parent_exhausted_recovery_sha256\":\""
        << AL_R20_R29_SEMANTIC_SHA256 << "\""
        << ",\"recomputed_parent_sha256\":\"" << r29_result_sha << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"manifests_exact\":" << (manifests_exact ? "true" : "false")
        << ",\"r29_parent_exact\":"
        << (r29_parent_exact ? "true" : "false")
        << ",\"prefix_exact\":" << (prefix_exact ? "true" : "false")
        << ",\"historical_roots_exact\":"
        << (historical_roots_exact ? "true" : "false")
        << ",\"terminal_eligible\":"
        << (terminal.eligible ? "true" : "false")
        << ",\"terminal_selected\":"
        << (terminal.selected ? "true" : "false")
        << ",\"terminal_state_correspondence\":"
        << (terminal.state_correspondence ? "true" : "false")
        << ",\"fresh_terminal_certificate\":"
        << (terminal.fresh_certificate ? "true" : "false")
        << ",\"all_certified\":" << (all_certified ? "true" : "false")
        << ",\"candidate_certified_cases\":" << candidate_certified_cases
        << ",\"terminal_certified_trials\":" << terminal.certified_trials
        << ",\"terminal_power\":" << terminal.selected_power
        << ",\"terminal_alpha\":\""
        << al_binary128_text(terminal.trial.alpha)
        << "\",\"terminal_armijo_accepted\":"
        << (terminal.trial.accepted ? "true" : "false")
        << ",\"terminal_armijo_margin\":\""
        << al_binary128_text(terminal.r31.margin)
        << "\",\"terminal_mask_changes\":"
        << terminal.trial.mask_changes
        << ",\"terminal_ball_activity_changed\":"
        << (terminal.trial.ball_activity_changed ? "true" : "false")
        << ",\"terminal_trial_sha256\":\"" << terminal.trial.root << "\""
        << ",\"terminal_r31_audit_sha256\":\"" << terminal.r31.root << "\""
        << ",\"terminal_dual_sha256\":\""
        << terminal.fresh_dual.root << "\""
        << ",\"terminal_metrics_sha256\":\""
        << terminal.fresh_metrics.root << "\""
        << ",\"terminal_lambda_sha256\":\""
        << terminal.terminal_lambda_root << "\""
        << ",\"terminal_primal\":\""
        << al_binary128_text(terminal.fresh_metrics.primal)
        << "\",\"terminal_dual_mapping\":\""
        << al_binary128_text(terminal.fresh_metrics.dual_mapping)
        << "\",\"terminal_complementarity\":\""
        << al_binary128_text(terminal.fresh_metrics.complementarity)
        << "\",\"terminal_stationarity\":\""
        << al_binary128_text(terminal.fresh_metrics.stationarity)
        << "\",\"terminal_gap_scaled\":\""
        << al_binary128_text(terminal.fresh_metrics.gap_scaled)
        << "\",\"terminal_root\":\"" << terminal.root << "\""
        << ",\"cases\":[";
    for (std::size_t index = 0U; index < diagnostics.size(); ++index) {
        if (index != 0U) report << ',';
        const bool shear = index == shear_case_index;
        const bool certified = shear
            ? terminal.selected && terminal.fresh_certificate
            : diagnostics[index].certified;
        report << "{\"family\":\"" << cases[index].family
            << "\",\"name\":\"" << cases[index].source.fixture.name
            << "\",\"certified\":" << (certified ? "true" : "false")
            << ",\"terminal_selection\":" << (shear ? "true" : "false")
            << ",\"accepted_iterations\":"
            << diagnostics[index].accepted_iterations
            << ",\"parent_root\":\"" << diagnostics[index].root << "\""
            << ",\"candidate_root\":\""
            << (shear ? terminal.root : diagnostics[index].root) << "\"}";
    }
    report << "]"
        << ",\"solver_executions\":" << solver_executions
        << ",\"new_trials\":0,\"terminal_selections\":"
        << (terminal.selected ? 1 : 0)
        << ",\"state_updates\":" << (terminal.selected ? 1 : 0)
        << ",\"ordinary_armijo_changes\":0,\"tolerance_changes\":0"
        << ",\"accepted_iteration_cap\":32,\"following_iterations\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"lifecycle_exact\":" << (lifecycle_exact ? "true" : "false")
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
