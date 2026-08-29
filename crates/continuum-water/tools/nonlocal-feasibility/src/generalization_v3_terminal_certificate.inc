namespace {

constexpr const char* AL_R20_R30_SEMANTIC_SHA256 =
    "e1f807e9ac25ad9317f07bc9e4a4364af76fa653e664ba8da10eaa2e10581ff5";

struct ALR20R31TrialAudit {
    bool exact = false;
    bool certificate_consistent = false;
    bool finite_pass = false;
    bool primal_pass = false;
    bool dual_pass = false;
    bool complementarity_pass = false;
    bool stationarity_pass = false;
    bool gap_sign_pass = false;
    bool gap_scaled_pass = false;
    bool armijo_recomposition_exact = false;
    bool bound_dominated = false;
    std::size_t power = 0U;
    Binary128 alpha = static_cast<Binary128>(0.0);
    Binary128 nominal_dual_change = static_cast<Binary128>(0.0);
    Binary128 nominal_armijo_margin = static_cast<Binary128>(0.0);
    Binary128 old_bound = static_cast<Binary128>(0.0);
    Binary128 new_bound = static_cast<Binary128>(0.0);
    Binary128 slope_term = static_cast<Binary128>(0.0);
    Binary128 rhs_round_bound = static_cast<Binary128>(0.0);
    Binary128 bound_burden = static_cast<Binary128>(0.0);
    Binary128 dual_increase_lower = static_cast<Binary128>(0.0);
    Binary128 margin = static_cast<Binary128>(0.0);
    std::string root;
};

ALR20R31TrialAudit al_r20_r31_trial_audit(
    const ALR20NewtonDualEvaluation& current,
    const ALR20SemismoothStep& step,
    const ALR20NewtonTrial& trial) {
    const Binary128 tolerance = ldexpq(static_cast<Binary128>(1.0), -70);
    const Binary128 coefficient = ldexpq(static_cast<Binary128>(1.0), -10);
    const Binary128 slope_lower = step.slope - step.slope_bound;
    ALR20R31TrialAudit audit;
    audit.power = trial.power;
    audit.alpha = trial.alpha;
    audit.finite_pass = trial.metrics.finite;
    audit.primal_pass = trial.metrics.primal <= tolerance;
    audit.dual_pass = trial.metrics.dual_mapping <= tolerance;
    audit.complementarity_pass =
        trial.metrics.complementarity <= tolerance;
    audit.stationarity_pass = trial.metrics.stationarity <= tolerance;
    audit.gap_sign_pass = trial.metrics.gap >= -trial.metrics.gap_bound;
    audit.gap_scaled_pass = trial.metrics.gap_scaled <= tolerance;
    const bool composed_certificate = audit.finite_pass
        && audit.primal_pass && audit.dual_pass
        && audit.complementarity_pass && audit.stationarity_pass
        && audit.gap_sign_pass && audit.gap_scaled_pass;
    audit.certificate_consistent =
        composed_certificate == trial.metrics.certified;
    audit.nominal_dual_change = trial.dual.value - current.value;
    audit.nominal_armijo_margin = audit.nominal_dual_change
        - coefficient * trial.alpha * step.slope;
    audit.old_bound = current.bound;
    audit.new_bound = trial.dual.bound;
    audit.slope_term = coefficient * trial.alpha * slope_lower;
    audit.rhs_round_bound = al_r20_q_gamma(128U)
        * (fabsq(current.value) + current.bound
            + coefficient * trial.alpha * fabsq(slope_lower));
    audit.bound_burden = audit.old_bound + audit.new_bound
        + audit.slope_term + audit.rhs_round_bound;
    audit.dual_increase_lower = trial.dual_increase_lower;
    audit.margin = al_r20_r19_armijo_margin(current, step, trial);
    const Binary128 recomposed_margin = trial.dual.value
        - trial.dual.bound
        - (current.value + current.bound + audit.slope_term)
        - audit.rhs_round_bound;
    audit.armijo_recomposition_exact =
        al_binary128_hex(recomposed_margin) == al_binary128_hex(audit.margin);
    audit.bound_dominated =
        audit.bound_burden >= fabsq(audit.nominal_dual_change);
    audit.exact = trial.exact && !trial.accepted
        && trial.mask_changes == 0U
        && !trial.ball_activity_changed
        && audit.armijo_recomposition_exact
        && finiteq(audit.nominal_dual_change) != 0
        && finiteq(audit.nominal_armijo_margin) != 0
        && finiteq(audit.bound_burden) != 0
        && finiteq(audit.margin) != 0;
    std::ostringstream material;
    material << audit.exact << ':' << audit.certificate_consistent << ':'
        << audit.finite_pass << ':' << audit.primal_pass << ':'
        << audit.dual_pass << ':' << audit.complementarity_pass << ':'
        << audit.stationarity_pass << ':' << audit.gap_sign_pass << ':'
        << audit.gap_scaled_pass << ':'
        << audit.armijo_recomposition_exact << ':'
        << audit.bound_dominated << ':'
        << audit.power << ':' << al_binary128_hex(audit.alpha) << ':'
        << al_binary128_hex(audit.nominal_dual_change) << ':'
        << al_binary128_hex(audit.nominal_armijo_margin) << ':'
        << al_binary128_hex(audit.old_bound) << ':'
        << al_binary128_hex(audit.new_bound) << ':'
        << al_binary128_hex(audit.slope_term) << ':'
        << al_binary128_hex(audit.rhs_round_bound) << ':'
        << al_binary128_hex(audit.bound_burden) << ':'
        << al_binary128_hex(audit.dual_increase_lower) << ':'
        << al_binary128_hex(audit.margin) << ':'
        << trial.dual.root << ':' << trial.metrics.root;
    audit.root = sha256_hex(material.str());
    return audit;
}

} // namespace

SplitBoundaryReport run_al_generalization_v3_terminal_certificate_controls() {
#if defined(__linux__) && defined(__x86_64__) && defined(__GNUC__) \
    && defined(__SIZEOF_FLOAT128__)
    constexpr bool frozen_platform = true;
#else
    constexpr bool frozen_platform = false;
#endif
    const Binary128 zero = static_cast<Binary128>(0.0);
    const Binary128 tolerance = ldexpq(static_cast<Binary128>(1.0), -70);
    const bool profile_exact = frozen_platform
        && sizeof(Binary128) == 16U && FLT_RADIX == 2
        && FLT128_MANT_DIG == 113;
    const bool controls_exact = profile_exact
        && al_r20_verified_inverse_controls()
        && al_r20_semismooth_controls()
        && al_r20_r21_event_controls();
    std::vector<ALR20ManifestCase> cases = al_r20_v3_manifest_cases();
    const bool manifest_exact = cases.size() == AL_R20_V3_PROBLEM_SHA256.size()
        && AL_R20_V3_SHEAR_INDEX < cases.size()
        && al_r20_v3_manifest_root(cases) == AL_R20_V3_MANIFEST_SHA256;
    bool parent_exact = controls_exact && manifest_exact;
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    ALR20ProblemData problem;
    if (parent_exact) {
        problem = al_r20_materialize_problem(
            cases[AL_R20_V3_SHEAR_INDEX], &sparse_work,
            &static_work, &adjacency_work);
        parent_exact = problem.exact && problem.source_positive == 0U
            && problem.problem_root
                == AL_R20_V3_PROBLEM_SHA256[AL_R20_V3_SHEAR_INDEX];
    }
    ALR20R29CaseAudit parent;
    std::size_t solver_executions = 0U;
    if (parent_exact) {
        parent = al_r20_r29_shear_case(problem);
        ++solver_executions;
    }
    const ALR20SemismoothCase& diagnostic = parent.diagnostic;
    bool reproduction_exact = parent_exact
        && parent.root == AL_R20_R29_SHEAR_SHA256
        && parent.recovery_applied && parent.recovery_correspondence
        && diagnostic.globalization_rejected
        && diagnostic.accepted_iterations == 21U
        && diagnostic.steps.size() == 22U;
    ALR20NewtonDualEvaluation current;
    ALR20SemismoothStep final;
    if (reproduction_exact) {
        current = diagnostic.steps[20U].selected.dual;
        final = diagnostic.steps[21U];
        reproduction_exact = final.iteration == 22U
            && final.direction_certified && final.solve.kkt_certified
            && !final.accepted
            && final.trials.size() == AL_R20_NEWTON_LINE_TRIALS;
    }

    const Binary128 slope_lower = final.slope - final.slope_bound;
    std::vector<ALR20R31TrialAudit> audits;
    std::size_t certified_trials = 0U;
    std::size_t first_certified_power = AL_R20_NEWTON_LINE_TRIALS;
    std::size_t bound_dominated_trials = 0U;
    std::size_t certificate_contradictions = 0U;
    bool audit_values_exact = reproduction_exact && slope_lower > zero;
    for (const ALR20NewtonTrial& trial : final.trials) {
        ALR20R31TrialAudit audit =
            al_r20_r31_trial_audit(current, final, trial);
        if (trial.metrics.certified) {
            ++certified_trials;
            if (first_certified_power == AL_R20_NEWTON_LINE_TRIALS)
                first_certified_power = trial.power;
        }
        if (audit.bound_dominated) ++bound_dominated_trials;
        if (!audit.certificate_consistent) ++certificate_contradictions;
        audit_values_exact = audit_values_exact && audit.exact;
        audits.push_back(std::move(audit));
    }

    std::string classification = "TERMINAL_CERTIFICATE_UNRESOLVED";
    if (certificate_contradictions > 0U)
        classification = "TERMINAL_CERTIFICATE_CONTRADICTION";
    else if (certified_trials > 0U)
        classification = "TERMINAL_CERTIFICATE_PRECEDES_ARMIJO";
    else
        classification = "NO_TERMINAL_CERTIFIED_TRIAL";

    if (parent_exact)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = !parent_exact
        || (sparse_work.workspace_releases == 1U
            && sparse_work.live_workspaces == 0U
            && !sparse_work.lifecycle_underflow);
    const bool exact = parent_exact && reproduction_exact
        && audit_values_exact && lifecycle_exact;
    const std::string route = !parent_exact
        ? "V3_TERMINAL_CERTIFICATE_PARENT_REJECTED"
        : (!reproduction_exact
            ? "V3_TERMINAL_CERTIFICATE_REPRODUCTION_REJECTED"
            : (!audit_values_exact
                ? "V3_TERMINAL_CERTIFICATE_AUDIT_REJECTED"
                : "V3_TERMINAL_CERTIFICATE_IDENTIFIED"));

    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r31-terminal-certificate.v1|"
        << AL_R20_R30_SEMANTIC_SHA256 << ':' << route << ':'
        << classification << ':' << diagnostic.root << ':' << final.root
        << ':' << certified_trials << ':' << first_certified_power << ':'
        << bound_dominated_trials << ':' << certificate_contradictions << '|';
    for (const ALR20R31TrialAudit& audit : audits)
        semantic << audit.root << ';';
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r31_terminal_certificate.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"classification\":\"" << classification << "\""
        << ",\"parent_post_recovery_globalization_sha256\":\""
        << AL_R20_R30_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"manifest_exact\":" << (manifest_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"reproduction_exact\":"
        << (reproduction_exact ? "true" : "false")
        << ",\"audit_values_exact\":"
        << (audit_values_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"case_sha256\":\"" << diagnostic.root << "\""
        << ",\"step_sha256\":\"" << final.root << "\""
        << ",\"tolerance\":\"" << al_binary128_text(tolerance)
        << "\",\"certified_trials\":" << certified_trials
        << ",\"first_certified_power\":" << first_certified_power
        << ",\"bound_dominated_trials\":" << bound_dominated_trials
        << ",\"certificate_contradictions\":"
        << certificate_contradictions
        << ",\"trials\":[";
    for (std::size_t index = 0U; index < audits.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20R31TrialAudit& audit = audits[index];
        const ALR20NewtonTrial& trial = final.trials[index];
        report << "{\"power\":" << audit.power
            << ",\"alpha\":\"" << al_binary128_text(audit.alpha)
            << "\",\"metrics_certified\":"
            << (trial.metrics.certified ? "true" : "false")
            << ",\"certificate_consistent\":"
            << (audit.certificate_consistent ? "true" : "false")
            << ",\"finite_pass\":"
            << (audit.finite_pass ? "true" : "false")
            << ",\"primal_pass\":"
            << (audit.primal_pass ? "true" : "false")
            << ",\"dual_pass\":" << (audit.dual_pass ? "true" : "false")
            << ",\"complementarity_pass\":"
            << (audit.complementarity_pass ? "true" : "false")
            << ",\"stationarity_pass\":"
            << (audit.stationarity_pass ? "true" : "false")
            << ",\"gap_sign_pass\":"
            << (audit.gap_sign_pass ? "true" : "false")
            << ",\"gap_scaled_pass\":"
            << (audit.gap_scaled_pass ? "true" : "false")
            << ",\"armijo_recomposition_exact\":"
            << (audit.armijo_recomposition_exact ? "true" : "false")
            << ",\"nominal_dual_change\":\""
            << al_binary128_text(audit.nominal_dual_change)
            << "\",\"nominal_armijo_margin\":\""
            << al_binary128_text(audit.nominal_armijo_margin)
            << "\",\"old_bound\":\""
            << al_binary128_text(audit.old_bound)
            << "\",\"new_bound\":\""
            << al_binary128_text(audit.new_bound)
            << "\",\"slope_term\":\""
            << al_binary128_text(audit.slope_term)
            << "\",\"rhs_round_bound\":\""
            << al_binary128_text(audit.rhs_round_bound)
            << "\",\"bound_burden\":\""
            << al_binary128_text(audit.bound_burden)
            << "\",\"bound_dominated\":"
            << (audit.bound_dominated ? "true" : "false")
            << ",\"dual_increase_lower\":\""
            << al_binary128_text(audit.dual_increase_lower)
            << "\",\"armijo_margin\":\""
            << al_binary128_text(audit.margin)
            << "\",\"primal\":\""
            << al_binary128_text(trial.metrics.primal)
            << "\",\"dual_mapping\":\""
            << al_binary128_text(trial.metrics.dual_mapping)
            << "\",\"complementarity\":\""
            << al_binary128_text(trial.metrics.complementarity)
            << "\",\"stationarity\":\""
            << al_binary128_text(trial.metrics.stationarity)
            << "\",\"gap\":\"" << al_binary128_text(trial.metrics.gap)
            << "\",\"gap_bound\":\""
            << al_binary128_text(trial.metrics.gap_bound)
            << "\",\"gap_scaled\":\""
            << al_binary128_text(trial.metrics.gap_scaled)
            << "\",\"metrics_sha256\":\"" << trial.metrics.root
            << "\",\"root\":\"" << audit.root << "\"}";
    }
    report << "]"
        << ",\"solver_executions\":" << solver_executions
        << ",\"new_trials\":0,\"state_updates\":0"
        << ",\"armijo_changes\":0,\"tolerance_changes\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
