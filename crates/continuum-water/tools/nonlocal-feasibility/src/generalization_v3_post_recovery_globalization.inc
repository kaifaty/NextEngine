namespace {

constexpr const char* AL_R20_R29_SEMANTIC_SHA256 =
    "3c4f5d5149d1510880f2d65b4c122e51fa441a6102d43f492a5c4e25930bba7b";
constexpr const char* AL_R20_R29_SHEAR_SHA256 =
    "3f4798c8882eee7226a67381414fc7bba9fc47a0ffa06a7f5a3b90e1bf83e473";

} // namespace

SplitBoundaryReport
run_al_generalization_v3_post_recovery_globalization_controls() {
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
    ALR20R29CaseAudit audit;
    std::size_t solver_executions = 0U;
    if (parent_exact) {
        audit = al_r20_r29_shear_case(problem);
        ++solver_executions;
    }
    const ALR20SemismoothCase& diagnostic = audit.diagnostic;
    bool reproduction_exact = parent_exact
        && audit.root == AL_R20_R29_SHEAR_SHA256
        && audit.recovery_applied && audit.recovery_correspondence
        && diagnostic.harness_exact && !diagnostic.certified
        && !diagnostic.active_set_rejected
        && !diagnostic.direction_rejected
        && diagnostic.globalization_rejected
        && diagnostic.accepted_iterations == 21U
        && diagnostic.steps.size() == 22U;
    ALR20NewtonDualEvaluation current;
    ALR20SemismoothStep final;
    if (reproduction_exact) {
        current = diagnostic.steps[20U].selected.dual;
        final = diagnostic.steps[21U];
        const ALR20NewtonDualEvaluation recomputed =
            al_r20_newton_dual_evaluate(problem, diagnostic.final_lambda);
        reproduction_exact = recomputed.exact
            && recomputed.root == current.root
            && final.iteration == 22U
            && final.natural_face_resolved
            && final.solve.exact && final.solve.kkt_certified
            && final.direction_certified && !final.accepted
            && final.trials.size() == AL_R20_NEWTON_LINE_TRIALS;
    }

    const ALR20OracleMetrics& metrics = diagnostic.final_metrics;
    const Binary128 primal_ratio = metrics.primal / tolerance;
    const Binary128 dual_ratio = metrics.dual_mapping / tolerance;
    const Binary128 complementarity_ratio =
        metrics.complementarity / tolerance;
    const Binary128 stationarity_ratio = metrics.stationarity / tolerance;
    const Binary128 gap_ratio = metrics.gap_scaled / tolerance;
    const bool primal_pass = metrics.primal <= tolerance;
    const bool dual_pass = metrics.dual_mapping <= tolerance;
    const bool complementarity_pass = metrics.complementarity <= tolerance;
    const bool stationarity_pass = metrics.stationarity <= tolerance;
    const bool gap_sign_pass = metrics.gap >= -metrics.gap_bound;
    const bool gap_scaled_pass = metrics.gap_scaled <= tolerance;

    std::size_t stable_trials = 0U;
    std::size_t stable_negative_trials = 0U;
    std::size_t crossing_trials = 0U;
    std::size_t unsigned_margins = 0U;
    std::size_t maximum_mask_changes = 0U;
    bool all_crossing = reproduction_exact;
    bool margin_parity = reproduction_exact;
    std::vector<std::string> regime_roots;
    std::vector<std::vector<std::size_t>> changed_scalars;
    std::vector<Binary128> margins;
    for (const ALR20NewtonTrial& trial : final.trials) {
        const bool stable = trial.mask_changes == 0U
            && !trial.ball_activity_changed;
        const Binary128 margin =
            al_r20_r19_armijo_margin(current, final, trial);
        std::vector<std::size_t> changed;
        if (current.projector.mask.size() == trial.dual.projector.mask.size())
            for (std::size_t scalar = 0U;
                 scalar < current.projector.mask.size(); ++scalar)
                if (current.projector.mask[scalar]
                        != trial.dual.projector.mask[scalar])
                    changed.push_back(scalar);
        reproduction_exact = reproduction_exact && trial.exact
            && !trial.accepted && changed.size() == trial.mask_changes;
        if (stable) {
            ++stable_trials;
            if (margin < zero) ++stable_negative_trials;
        } else {
            ++crossing_trials;
        }
        if (finiteq(margin) == 0 || margin == zero) ++unsigned_margins;
        maximum_mask_changes = std::max(
            maximum_mask_changes, trial.mask_changes);
        all_crossing = all_crossing && !stable;
        margin_parity = margin_parity
            && ((margin > zero) == trial.accepted);
        regime_roots.push_back(al_r20_r20_mask_root(trial.dual.projector));
        changed_scalars.push_back(std::move(changed));
        margins.push_back(margin);
    }
    std::vector<std::string> unique_regimes = regime_roots;
    std::sort(unique_regimes.begin(), unique_regimes.end());
    unique_regimes.erase(
        std::unique(unique_regimes.begin(), unique_regimes.end()),
        unique_regimes.end());
    const bool mixed_regimes = unique_regimes.size() > 1U;

    std::string classification = "POST_RECOVERY_GLOBALIZATION_UNRESOLVED";
    if (stable_negative_trials > 0U)
        classification = "POST_RECOVERY_SAME_FACE_REJECTION";
    else if (all_crossing)
        classification = "POST_RECOVERY_LINE_ENVELOPE_EXHAUSTED";
    else if (mixed_regimes)
        classification = "POST_RECOVERY_MIXED_EVENT_REGIMES";
    else if (unsigned_margins > 0U)
        classification = "POST_RECOVERY_PRECISION_BOUNDARY";

    if (parent_exact)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = !parent_exact
        || (sparse_work.workspace_releases == 1U
            && sparse_work.live_workspaces == 0U
            && !sparse_work.lifecycle_underflow);
    const bool audit_exact = parent_exact && reproduction_exact
        && margin_parity && lifecycle_exact;
    const std::string route = !parent_exact
        ? "V3_POST_RECOVERY_PARENT_REJECTED"
        : (!reproduction_exact
            ? "V3_POST_RECOVERY_REPRODUCTION_REJECTED"
            : (!margin_parity
                ? "V3_POST_RECOVERY_MARGIN_REJECTED"
                : (classification == "POST_RECOVERY_GLOBALIZATION_UNRESOLVED"
                    ? "V3_POST_RECOVERY_UNRESOLVED"
                    : "V3_POST_RECOVERY_IDENTIFIED")));

    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r30-post-recovery-globalization.v1|"
        << AL_R20_R29_SEMANTIC_SHA256 << ':' << route << ':'
        << classification << ':' << diagnostic.root << ':'
        << diagnostic.final_metrics.root << ':'
        << al_binary128_hex(tolerance) << ':'
        << al_binary128_hex(primal_ratio) << ':'
        << al_binary128_hex(dual_ratio) << ':'
        << al_binary128_hex(complementarity_ratio) << ':'
        << al_binary128_hex(stationarity_ratio) << ':'
        << al_binary128_hex(gap_ratio) << ':' << final.root << '|';
    for (std::size_t index = 0U; index < final.trials.size(); ++index) {
        semantic << final.trials[index].root << ':'
            << al_binary128_hex(margins[index]) << ':'
            << regime_roots[index] << ':';
        for (std::size_t scalar : changed_scalars[index])
            semantic << scalar << ',';
        semantic << ';';
    }
    semantic << stable_trials << ':' << stable_negative_trials << ':'
        << crossing_trials << ':' << unsigned_margins << ':'
        << unique_regimes.size();
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r30_post_recovery_globalization.v1\""
        << ",\"status\":\"" << (audit_exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"classification\":\"" << classification << "\""
        << ",\"parent_exhausted_recovery_sha256\":\""
        << AL_R20_R29_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"manifest_exact\":" << (manifest_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"reproduction_exact\":"
        << (reproduction_exact ? "true" : "false")
        << ",\"margin_parity\":" << (margin_parity ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"case_sha256\":\"" << diagnostic.root << "\""
        << ",\"certificate\":{\"tolerance\":\""
        << al_binary128_text(tolerance)
        << "\",\"metrics_root\":\"" << metrics.root
        << "\",\"finite\":" << (metrics.finite ? "true" : "false")
        << ",\"certified\":"
        << (metrics.certified ? "true" : "false")
        << ",\"primal\":\"" << al_binary128_text(metrics.primal)
        << "\",\"primal_ratio\":\"" << al_binary128_text(primal_ratio)
        << "\",\"primal_pass\":" << (primal_pass ? "true" : "false")
        << ",\"dual_mapping\":\""
        << al_binary128_text(metrics.dual_mapping)
        << "\",\"dual_ratio\":\"" << al_binary128_text(dual_ratio)
        << "\",\"dual_pass\":" << (dual_pass ? "true" : "false")
        << ",\"complementarity\":\""
        << al_binary128_text(metrics.complementarity)
        << "\",\"complementarity_ratio\":\""
        << al_binary128_text(complementarity_ratio)
        << "\",\"complementarity_pass\":"
        << (complementarity_pass ? "true" : "false")
        << ",\"stationarity\":\""
        << al_binary128_text(metrics.stationarity)
        << "\",\"stationarity_ratio\":\""
        << al_binary128_text(stationarity_ratio)
        << "\",\"stationarity_pass\":"
        << (stationarity_pass ? "true" : "false")
        << ",\"gap\":\"" << al_binary128_text(metrics.gap)
        << "\",\"gap_bound\":\"" << al_binary128_text(metrics.gap_bound)
        << "\",\"gap_sign_pass\":"
        << (gap_sign_pass ? "true" : "false")
        << ",\"gap_scaled\":\"" << al_binary128_text(metrics.gap_scaled)
        << "\",\"gap_ratio\":\"" << al_binary128_text(gap_ratio)
        << "\",\"gap_scaled_pass\":"
        << (gap_scaled_pass ? "true" : "false") << "}"
        << ",\"rejected_step\":{\"iteration\":" << final.iteration
        << ",\"natural_face\":" << final.natural_face_size
        << ",\"natural_face_sha256\":\"" << final.natural_face_root
        << "\",\"support\":" << final.solve.support_size
        << ",\"support_sha256\":\"" << final.solve.support_root
        << "\",\"nnqp_kkt_certified\":"
        << (final.solve.kkt_certified ? "true" : "false")
        << ",\"slope\":\"" << al_binary128_text(final.slope)
        << "\",\"slope_bound\":\""
        << al_binary128_text(final.slope_bound)
        << "\",\"current_mask_sha256\":\""
        << al_r20_r20_mask_root(current.projector)
        << "\",\"trials\":[";
    for (std::size_t index = 0U; index < final.trials.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20NewtonTrial& trial = final.trials[index];
        report << "{\"power\":" << trial.power
            << ",\"alpha\":\"" << al_binary128_text(trial.alpha)
            << "\",\"exact\":" << (trial.exact ? "true" : "false")
            << ",\"accepted\":" << (trial.accepted ? "true" : "false")
            << ",\"armijo_margin\":\""
            << al_binary128_text(margins[index])
            << "\",\"margin_strictly_signed\":"
            << (finiteq(margins[index]) != 0 && margins[index] != zero
                ? "true" : "false")
            << ",\"mask_changes\":" << trial.mask_changes
            << ",\"changed_scalars\":[";
        for (std::size_t local = 0U;
             local < changed_scalars[index].size(); ++local) {
            if (local != 0U) report << ',';
            report << changed_scalars[index][local];
        }
        report << "]"
            << ",\"ball_activity_changed\":"
            << (trial.ball_activity_changed ? "true" : "false")
            << ",\"mask_sha256\":\"" << regime_roots[index]
            << "\",\"dual_increase_lower\":\""
            << al_binary128_text(trial.dual_increase_lower)
            << "\",\"primal\":\""
            << al_binary128_text(trial.metrics.primal)
            << "\",\"dual_mapping\":\""
            << al_binary128_text(trial.metrics.dual_mapping)
            << "\",\"complementarity\":\""
            << al_binary128_text(trial.metrics.complementarity)
            << "\",\"stationarity\":\""
            << al_binary128_text(trial.metrics.stationarity)
            << "\",\"gap_scaled\":\""
            << al_binary128_text(trial.metrics.gap_scaled)
            << "\",\"root\":\"" << trial.root << "\"}";
    }
    report << "],\"root\":\"" << final.root << "\"}"
        << ",\"stable_trials\":" << stable_trials
        << ",\"stable_negative_trials\":" << stable_negative_trials
        << ",\"crossing_trials\":" << crossing_trials
        << ",\"unsigned_margins\":" << unsigned_margins
        << ",\"maximum_mask_changes\":" << maximum_mask_changes
        << ",\"unique_mask_regimes\":" << unique_regimes.size()
        << ",\"all_trials_crossing\":"
        << (all_crossing ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"new_line_trials\":0,\"new_event_trials\":0"
        << ",\"state_updates\":0,\"tolerance_changes\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {audit_exact, report.str()};
}
