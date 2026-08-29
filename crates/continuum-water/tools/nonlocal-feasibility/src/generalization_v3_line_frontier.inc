namespace {

constexpr const char* AL_R20_V3_SHEAR_TRAJECTORY_SHA256 =
    "6f019b7c314b09406f68438ec4beadfd839b7664952d346a9c898563036d40aa";

Binary128 al_r20_r19_armijo_margin(
    const ALR20NewtonDualEvaluation& current,
    const ALR20SemismoothStep& step,
    const ALR20NewtonTrial& trial) {
    const Binary128 slope_lower = step.slope - step.slope_bound;
    const Binary128 coefficient =
        ldexpq(static_cast<Binary128>(1.0), -10);
    const Binary128 rhs = current.value + current.bound
        + coefficient * trial.alpha * slope_lower;
    const Binary128 rhs_bound = al_r20_q_gamma(128U)
        * (fabsq(current.value) + current.bound
            + coefficient * trial.alpha * fabsq(slope_lower));
    return trial.dual.value - trial.dual.bound - rhs - rhs_bound;
}

} // namespace

SplitBoundaryReport run_al_generalization_v3_line_frontier_controls() {
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
    ALR20SemismoothCase diagnostic;
    std::size_t solver_executions = 0U;
    if (parent_exact) {
        diagnostic = al_r20_semismooth_case(
            problems[AL_R20_V3_SHEAR_INDEX], true, true);
        ++solver_executions;
    }
    bool reproduction_exact = parent_exact
        && diagnostic.root == AL_R20_V3_SHEAR_CANDIDATE_SHA256
        && diagnostic.steps.size() == 32U;
    bool margin_parity = reproduction_exact;
    std::size_t late_steps = 0U;
    std::size_t frontier_coincidences = 0U;
    std::size_t stable_rejections = 0U;
    std::size_t crossing_acceptances = 0U;
    std::vector<std::size_t> first_stable_powers;
    std::vector<std::size_t> first_accepted_powers;
    for (std::size_t step_index = 0U;
         step_index < diagnostic.steps.size(); ++step_index) {
        const ALR20SemismoothStep& step = diagnostic.steps[step_index];
        if (step.iteration < AL_R20_V3_LATE_FIRST) continue;
        ++late_steps;
        const ALR20NewtonDualEvaluation& current =
            diagnostic.steps[step_index - 1U].selected.dual;
        std::size_t first_stable = AL_R20_NEWTON_LINE_TRIALS;
        std::size_t first_accepted = AL_R20_NEWTON_LINE_TRIALS;
        for (const ALR20NewtonTrial& trial : step.trials) {
            const Binary128 margin =
                al_r20_r19_armijo_margin(current, step, trial);
            const bool accepted_from_margin =
                margin >= static_cast<Binary128>(0.0);
            margin_parity = margin_parity && trial.exact
                && accepted_from_margin == trial.accepted;
            const bool stable = trial.mask_changes == 0U
                && !trial.ball_activity_changed;
            if (stable && first_stable == AL_R20_NEWTON_LINE_TRIALS)
                first_stable = trial.power;
            if (stable && !trial.accepted) ++stable_rejections;
            if (trial.accepted) {
                if (first_accepted == AL_R20_NEWTON_LINE_TRIALS)
                    first_accepted = trial.power;
                if (!stable) ++crossing_acceptances;
            }
        }
        if (first_stable == first_accepted
            && first_accepted != AL_R20_NEWTON_LINE_TRIALS)
            ++frontier_coincidences;
        first_stable_powers.push_back(first_stable);
        first_accepted_powers.push_back(first_accepted);
    }
    reproduction_exact = reproduction_exact && late_steps == 8U;
    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    std::string classification = "LINE_FRONTIER_UNRESOLVED";
    if (stable_rejections > 0U)
        classification = "SAME_FACE_MODEL_REJECTION";
    else if (frontier_coincidences >= 6U)
        classification = "MASK_CROSSING_FRONTIER";
    else if (crossing_acceptances > 0U)
        classification = "CROSSING_ACCEPTANCE_TRANSITION";
    const std::string route = !parent_exact
        ? "V3_LINE_FRONTIER_PARENT_REJECTED"
        : (!reproduction_exact ? "V3_LINE_FRONTIER_REPRODUCTION_REJECTED"
            : (!margin_parity ? "V3_LINE_FRONTIER_MARGIN_REJECTED"
                              : "V3_LINE_FRONTIER_IDENTIFIED"));
    const bool exact = parent_exact && reproduction_exact
        && margin_parity && lifecycle_exact;
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r19-v3-line-frontier.v1|"
             << AL_R20_V3_SHEAR_TRAJECTORY_SHA256 << ':' << route << ':'
             << classification << '|';
    for (std::size_t step_index = AL_R20_V3_LATE_FIRST - 1U;
         step_index < diagnostic.steps.size(); ++step_index) {
        const ALR20SemismoothStep& step = diagnostic.steps[step_index];
        const ALR20NewtonDualEvaluation& current =
            diagnostic.steps[step_index - 1U].selected.dual;
        semantic << step.root << '|';
        for (const ALR20NewtonTrial& trial : step.trials)
            semantic << trial.root << ':'
                     << al_binary128_hex(
                            al_r20_r19_armijo_margin(current, step, trial))
                     << ';';
    }
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r19_v3_line_frontier.v1\""
           << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
           << ",\"route\":\"" << route << "\""
           << ",\"classification\":\"" << classification << "\""
           << ",\"parent_shear_trajectory_sha256\":\""
           << AL_R20_V3_SHEAR_TRAJECTORY_SHA256
           << "\",\"profile_exact\":"
           << (profile_exact ? "true" : "false")
           << ",\"controls_exact\":"
           << (controls_exact ? "true" : "false")
           << ",\"parent_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"reproduction_exact\":"
           << (reproduction_exact ? "true" : "false")
           << ",\"margin_parity\":"
           << (margin_parity ? "true" : "false")
           << ",\"lifecycle_exact\":"
           << (lifecycle_exact ? "true" : "false")
           << ",\"frontier_coincidences\":"
           << frontier_coincidences
           << ",\"stable_rejections\":" << stable_rejections
           << ",\"crossing_acceptances\":" << crossing_acceptances
           << ",\"steps\":[";
    for (std::size_t local = 0U; local < first_stable_powers.size(); ++local) {
        if (local != 0U) report << ',';
        const std::size_t step_index =
            AL_R20_V3_LATE_FIRST - 1U + local;
        const ALR20SemismoothStep& step = diagnostic.steps[step_index];
        const ALR20NewtonDualEvaluation& current =
            diagnostic.steps[step_index - 1U].selected.dual;
        report << "{\"iteration\":" << step.iteration
               << ",\"first_stable_power\":"
               << first_stable_powers[local]
               << ",\"first_accepted_power\":"
               << first_accepted_powers[local]
               << ",\"slope\":\"" << al_binary128_text(step.slope)
               << "\",\"slope_bound\":\""
               << al_binary128_text(step.slope_bound)
               << "\",\"trials\":[";
        for (std::size_t trial_index = 0U;
             trial_index < step.trials.size(); ++trial_index) {
            if (trial_index != 0U) report << ',';
            const ALR20NewtonTrial& trial = step.trials[trial_index];
            report << "{\"power\":" << trial.power
                   << ",\"alpha\":\""
                   << al_binary128_text(trial.alpha)
                   << "\",\"accepted\":"
                   << (trial.accepted ? "true" : "false")
                   << ",\"mask_changes\":" << trial.mask_changes
                   << ",\"ball_activity_changed\":"
                   << (trial.ball_activity_changed ? "true" : "false")
                   << ",\"armijo_margin\":\""
                   << al_binary128_text(
                        al_r20_r19_armijo_margin(current, step, trial))
                   << "\",\"dual_increase_lower\":\""
                   << al_binary128_text(trial.dual_increase_lower)
                   << "\",\"primal\":\""
                   << al_binary128_text(trial.metrics.primal)
                   << "\",\"dual_mapping\":\""
                   << al_binary128_text(trial.metrics.dual_mapping)
                   << "\",\"root\":\"" << trial.root << "\"}";
        }
        report << "],\"root\":\"" << step.root << "\"}";
    }
    report << "]"
              ",\"solver_executions\":" << solver_executions
           << ",\"new_line_trials\":0,\"state_updates\":0"
              ",\"timing_admitted\":false,\"solver_authority\":false"
              ",\"runtime_authority\":false"
              ",\"production_authority\":false"
              ",\"result_sha256\":\""
           << result_sha << "\"}";
    return {exact, report.str()};
}
