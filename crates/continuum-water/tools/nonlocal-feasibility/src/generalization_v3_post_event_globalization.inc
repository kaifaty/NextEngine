namespace {

constexpr const char* AL_R20_V3_CERTIFIED_EVENT_TRAJECTORY_SHA256 =
    "4203c7ce2b86014a3099eb440bb0d011f8b6e31856ef142c3acbd93ed807a8d2";
constexpr const char* AL_R20_V3_CERTIFIED_EVENT_SHEAR_SHA256 =
    "1c9a0bf3457b32e6e81c3aab256dd1ea7ac26cdbd6f5e9c51943e60cd4c9cf91";

struct ALR20R27EventLineage {
    bool exact = false;
    std::size_t iteration = 0U;
    std::size_t scalar = 0U;
    int old_mask = 0;
    int new_mask = 0;
    std::size_t mask_changes = 0U;
    std::string old_mask_root;
    std::string new_mask_root;
    std::string source_face_root;
    std::string source_support_root;
    std::string next_face_root;
    std::string next_support_root;
    std::string root;
};

} // namespace

SplitBoundaryReport
run_al_generalization_v3_post_event_globalization_controls() {
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

    ALR20SemismoothCase diagnostic;
    std::size_t solver_executions = 0U;
    if (parent_exact) {
        diagnostic = al_r20_semismooth_case(problem, true, true, true);
        ++solver_executions;
    }
    bool reproduction_exact = parent_exact
        && diagnostic.harness_exact
        && diagnostic.root == AL_R20_V3_CERTIFIED_EVENT_SHEAR_SHA256
        && !diagnostic.certified
        && !diagnostic.active_set_rejected
        && !diagnostic.direction_rejected
        && diagnostic.globalization_rejected
        && diagnostic.accepted_iterations == 19U
        && diagnostic.steps.size() == 20U
        && diagnostic.event_replacements == 3U
        && diagnostic.event_fallbacks == 0U
        && diagnostic.event_extra_evaluations == 3U;

    ALR20NewtonDualEvaluation initial;
    if (parent_exact) {
        initial = al_r20_newton_dual_evaluate(problem,
            std::vector<Binary128>(problem.row_scale.size(),
                static_cast<Binary128>(0.0)));
        reproduction_exact = reproduction_exact && initial.exact;
    }

    std::vector<ALR20R27EventLineage> event_lineage;
    bool face_lineage_exact = reproduction_exact;
    for (std::size_t index = 0U;
         index < diagnostic.steps.size(); ++index) {
        const ALR20SemismoothStep& step = diagnostic.steps[index];
        reproduction_exact = reproduction_exact
            && step.iteration == index + 1U
            && (index + 1U == diagnostic.steps.size()
                ? !step.accepted : step.accepted);
        if (!step.event_replacement_applied) continue;
        const ALR20NewtonDualEvaluation& current = index == 0U
            ? initial : diagnostic.steps[index - 1U].selected.dual;
        ALR20R27EventLineage lineage;
        lineage.iteration = step.iteration;
        lineage.scalar = step.event_scalar;
        lineage.mask_changes = step.selected.mask_changes;
        lineage.old_mask_root = al_r20_r20_mask_root(current.projector);
        lineage.new_mask_root = al_r20_r20_mask_root(step.selected.dual.projector);
        lineage.source_face_root = step.natural_face_root;
        lineage.source_support_root = step.solve.support_root;
        const bool scalar_in_range = step.event_scalar
                < current.projector.mask.size()
            && step.event_scalar < step.selected.dual.projector.mask.size();
        if (scalar_in_range) {
            lineage.old_mask = current.projector.mask[step.event_scalar];
            lineage.new_mask =
                step.selected.dual.projector.mask[step.event_scalar];
        }
        std::size_t observed_changes = 0U;
        bool predicted_scalar_changed = false;
        if (current.projector.mask.size()
                == step.selected.dual.projector.mask.size()) {
            for (std::size_t scalar = 0U;
                 scalar < current.projector.mask.size(); ++scalar)
                if (current.projector.mask[scalar]
                        != step.selected.dual.projector.mask[scalar]) {
                    ++observed_changes;
                    predicted_scalar_changed = predicted_scalar_changed
                        || scalar == step.event_scalar;
                }
        }
        const bool has_next = index + 1U < diagnostic.steps.size();
        if (has_next) {
            const ALR20SemismoothStep& next = diagnostic.steps[index + 1U];
            lineage.next_face_root = next.natural_face_root;
            lineage.next_support_root = next.solve.support_root;
        }
        lineage.exact = scalar_in_range && has_next
            && step.event_replacement_eligible
            && step.selected.exact && step.selected.accepted
            && step.selected.power == AL_R20_NEWTON_LINE_TRIALS
            && step.selected.mask_changes == 1U
            && observed_changes == 1U && predicted_scalar_changed
            && !step.selected.ball_activity_changed
            && lineage.old_mask == -1 && lineage.new_mask == 0
            && step.event_certified_alpha > step.event_root_alpha
            && step.event_forward_bound > static_cast<Binary128>(0.0)
            && diagnostic.steps[index + 1U].natural_face_resolved
            && diagnostic.steps[index + 1U].solve.exact
            && diagnostic.steps[index + 1U].solve.kkt_certified
            && !lineage.source_face_root.empty()
            && !lineage.source_support_root.empty()
            && !lineage.next_face_root.empty()
            && !lineage.next_support_root.empty();
        std::ostringstream lineage_material;
        lineage_material << lineage.exact << ':' << lineage.iteration << ':'
            << lineage.scalar << ':' << lineage.old_mask << ':'
            << lineage.new_mask << ':' << lineage.mask_changes << ':'
            << lineage.old_mask_root << ':' << lineage.new_mask_root << ':'
            << lineage.source_face_root << ':' << lineage.source_support_root
            << ':' << lineage.next_face_root << ':'
            << lineage.next_support_root;
        lineage.root = sha256_hex(lineage_material.str());
        face_lineage_exact = face_lineage_exact && lineage.exact;
        event_lineage.push_back(std::move(lineage));
    }
    face_lineage_exact = face_lineage_exact
        && event_lineage.size() == 3U;

    std::size_t final_trials = 0U;
    std::size_t final_stable_trials = 0U;
    std::size_t final_stable_rejections = 0U;
    std::size_t final_crossing_trials = 0U;
    std::size_t final_strict_negative_margins = 0U;
    std::size_t final_unsigned_margins = 0U;
    bool margin_parity = reproduction_exact;
    bool all_final_trials_crossing = reproduction_exact;
    Binary128 minimum_absolute_margin = HUGE_VALQ;
    Binary128 minimum_absolute_stable_margin = HUGE_VALQ;
    if (reproduction_exact) {
        const ALR20SemismoothStep& final = diagnostic.steps.back();
        const ALR20NewtonDualEvaluation& current =
            diagnostic.steps[diagnostic.steps.size() - 2U].selected.dual;
        reproduction_exact = final.natural_face_resolved
            && final.solve.exact && final.solve.kkt_certified
            && final.direction_certified && !final.accepted
            && final.trials.size() == AL_R20_NEWTON_LINE_TRIALS;
        for (const ALR20NewtonTrial& trial : final.trials) {
            const Binary128 margin =
                al_r20_r19_armijo_margin(current, final, trial);
            const bool signed_margin = finiteq(margin) != 0
                && margin != static_cast<Binary128>(0.0);
            const bool stable = trial.mask_changes == 0U
                && !trial.ball_activity_changed;
            ++final_trials;
            if (stable) {
                ++final_stable_trials;
                if (!trial.accepted) ++final_stable_rejections;
                minimum_absolute_stable_margin = fminq(
                    minimum_absolute_stable_margin, fabsq(margin));
            } else {
                ++final_crossing_trials;
            }
            if (margin < static_cast<Binary128>(0.0))
                ++final_strict_negative_margins;
            if (!signed_margin) ++final_unsigned_margins;
            minimum_absolute_margin = fminq(
                minimum_absolute_margin, fabsq(margin));
            margin_parity = margin_parity
                && ((margin > static_cast<Binary128>(0.0))
                    == trial.accepted);
            all_final_trials_crossing = all_final_trials_crossing && !stable;
            reproduction_exact = reproduction_exact
                && trial.exact && !trial.accepted;
        }
    }

    std::string classification = "POST_EVENT_GLOBALIZATION_UNRESOLVED";
    if (!face_lineage_exact)
        classification = "POST_EVENT_FACE_INCONSISTENCY";
    else if (final_stable_rejections > 0U
        && final_strict_negative_margins > 0U)
        classification = "POST_EVENT_SAME_FACE_REJECTION";
    else if (all_final_trials_crossing)
        classification = "POST_EVENT_LINE_ENVELOPE_EXHAUSTED";
    else if (final_unsigned_margins > 0U)
        classification = "POST_EVENT_PRECISION_BOUNDARY";

    if (parent_exact)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = !parent_exact
        || (sparse_work.workspace_releases == 1U
            && sparse_work.live_workspaces == 0U
            && !sparse_work.lifecycle_underflow);
    const bool audit_exact = parent_exact && reproduction_exact
        && margin_parity && lifecycle_exact;
    const std::string route = !parent_exact
        ? "V3_POST_EVENT_GLOBALIZATION_PARENT_REJECTED"
        : (!reproduction_exact
            ? "V3_POST_EVENT_GLOBALIZATION_REPRODUCTION_REJECTED"
            : (!margin_parity
                ? "V3_POST_EVENT_GLOBALIZATION_MARGIN_REJECTED"
                : (classification == "POST_EVENT_GLOBALIZATION_UNRESOLVED"
                    ? "V3_POST_EVENT_GLOBALIZATION_UNRESOLVED"
                    : "V3_POST_EVENT_GLOBALIZATION_IDENTIFIED")));

    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r27-post-event-globalization.v1|"
        << AL_R20_V3_CERTIFIED_EVENT_TRAJECTORY_SHA256 << ':' << route << ':'
        << classification << ':' << problem.problem_root << ':'
        << diagnostic.root << '|';
    for (const ALR20R27EventLineage& lineage : event_lineage)
        semantic << lineage.root << ';';
    for (std::size_t index = 0U;
         index < diagnostic.steps.size(); ++index) {
        const ALR20SemismoothStep& step = diagnostic.steps[index];
        const ALR20NewtonDualEvaluation& current = index == 0U
            ? initial : diagnostic.steps[index - 1U].selected.dual;
        semantic << step.root << ':' << current.root << ':'
            << step.natural_face_root << ':' << step.solve.support_root << '|';
        for (const ALR20NewtonTrial& trial : step.trials)
            semantic << trial.root << ':'
                << al_binary128_hex(
                    al_r20_r19_armijo_margin(current, step, trial)) << ';';
    }
    semantic << final_trials << ':' << final_stable_trials << ':'
        << final_stable_rejections << ':' << final_crossing_trials << ':'
        << final_strict_negative_margins << ':' << final_unsigned_margins;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r27_post_event_globalization.v1\""
        << ",\"status\":\"" << (audit_exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"classification\":\"" << classification << "\""
        << ",\"parent_certified_event_trajectory_sha256\":\""
        << AL_R20_V3_CERTIFIED_EVENT_TRAJECTORY_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"manifest_exact\":" << (manifest_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"reproduction_exact\":"
        << (reproduction_exact ? "true" : "false")
        << ",\"margin_parity\":" << (margin_parity ? "true" : "false")
        << ",\"face_lineage_exact\":"
        << (face_lineage_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"problem_sha256\":\"" << problem.problem_root << "\""
        << ",\"case_sha256\":\"" << diagnostic.root << "\""
        << ",\"accepted_iterations\":" << diagnostic.accepted_iterations
        << ",\"step_count\":" << diagnostic.steps.size()
        << ",\"event_replacements\":" << diagnostic.event_replacements
        << ",\"event_fallbacks\":" << diagnostic.event_fallbacks
        << ",\"event_lineage\":[";
    for (std::size_t index = 0U; index < event_lineage.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20R27EventLineage& lineage = event_lineage[index];
        report << "{\"iteration\":" << lineage.iteration
            << ",\"scalar\":" << lineage.scalar
            << ",\"exact\":" << (lineage.exact ? "true" : "false")
            << ",\"old_mask\":" << lineage.old_mask
            << ",\"new_mask\":" << lineage.new_mask
            << ",\"mask_changes\":" << lineage.mask_changes
            << ",\"old_mask_sha256\":\"" << lineage.old_mask_root
            << "\",\"new_mask_sha256\":\"" << lineage.new_mask_root
            << "\",\"source_face_sha256\":\""
            << lineage.source_face_root
            << "\",\"source_support_sha256\":\""
            << lineage.source_support_root
            << "\",\"next_face_sha256\":\"" << lineage.next_face_root
            << "\",\"next_support_sha256\":\""
            << lineage.next_support_root
            << "\",\"root\":\"" << lineage.root << "\"}";
    }
    report << "],\"steps\":[";
    for (std::size_t index = 0U;
         index < diagnostic.steps.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20SemismoothStep& step = diagnostic.steps[index];
        const ALR20NewtonDualEvaluation& current = index == 0U
            ? initial : diagnostic.steps[index - 1U].selected.dual;
        report << "{\"iteration\":" << step.iteration
            << ",\"accepted\":" << (step.accepted ? "true" : "false")
            << ",\"natural_face_resolved\":"
            << (step.natural_face_resolved ? "true" : "false")
            << ",\"natural_face\":" << step.natural_face_size
            << ",\"natural_face_sha256\":\"" << step.natural_face_root
            << "\",\"support\":" << step.solve.support_size
            << ",\"support_sha256\":\"" << step.solve.support_root
            << "\",\"nnqp_exact\":"
            << (step.solve.exact ? "true" : "false")
            << ",\"nnqp_kkt_certified\":"
            << (step.solve.kkt_certified ? "true" : "false")
            << ",\"principal_solves\":" << step.solve.principal_solves
            << ",\"transitions\":" << step.solve.transitions
            << ",\"nnqp_primal_violation\":\""
            << al_binary128_text(step.solve.primal_violation)
            << "\",\"nnqp_dual_violation\":\""
            << al_binary128_text(step.solve.dual_violation)
            << "\",\"nnqp_complementarity\":\""
            << al_binary128_text(step.solve.complementarity)
            << "\",\"slope\":\"" << al_binary128_text(step.slope)
            << "\",\"slope_bound\":\""
            << al_binary128_text(step.slope_bound)
            << "\",\"direction_error_bound\":\""
            << al_binary128_text(step.direction_error_bound)
            << "\",\"current_dual_sha256\":\"" << current.root
            << "\",\"current_mask_sha256\":\""
            << al_r20_r20_mask_root(current.projector)
            << "\",\"event_eligible\":"
            << (step.event_replacement_eligible ? "true" : "false")
            << ",\"event_applied\":"
            << (step.event_replacement_applied ? "true" : "false")
            << ",\"event_scalar\":" << step.event_scalar
            << ",\"event_root_alpha\":\""
            << al_binary128_text(step.event_root_alpha)
            << "\",\"event_certified_alpha\":\""
            << al_binary128_text(step.event_certified_alpha)
            << "\",\"event_forward_bound\":\""
            << al_binary128_text(step.event_forward_bound)
            << "\",\"trials\":[";
        for (std::size_t trial_index = 0U;
             trial_index < step.trials.size(); ++trial_index) {
            if (trial_index != 0U) report << ',';
            const ALR20NewtonTrial& trial = step.trials[trial_index];
            const Binary128 margin =
                al_r20_r19_armijo_margin(current, step, trial);
            report << "{\"power\":" << trial.power
                << ",\"event_trial\":"
                << (trial.power == AL_R20_NEWTON_LINE_TRIALS
                    ? "true" : "false")
                << ",\"alpha\":\"" << al_binary128_text(trial.alpha)
                << "\",\"exact\":" << (trial.exact ? "true" : "false")
                << ",\"accepted\":"
                << (trial.accepted ? "true" : "false")
                << ",\"armijo_margin\":\""
                << al_binary128_text(margin)
                << "\",\"margin_strictly_signed\":"
                << (finiteq(margin) != 0
                        && margin != static_cast<Binary128>(0.0)
                    ? "true" : "false")
                << ",\"dual_increase_lower\":\""
                << al_binary128_text(trial.dual_increase_lower)
                << "\",\"mask_changes\":" << trial.mask_changes
                << ",\"ball_activity_changed\":"
                << (trial.ball_activity_changed ? "true" : "false")
                << ",\"mask_sha256\":\""
                << al_r20_r20_mask_root(trial.dual.projector)
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
        report << "]";
        if (step.accepted) {
            report << ",\"selected\":{\"power\":" << step.selected.power
                << ",\"alpha\":\""
                << al_binary128_text(step.selected.alpha)
                << "\",\"mask_changes\":" << step.selected.mask_changes
                << ",\"ball_activity_changed\":"
                << (step.selected.ball_activity_changed ? "true" : "false")
                << ",\"dual_increase_lower\":\""
                << al_binary128_text(step.selected.dual_increase_lower)
                << "\",\"primal\":\""
                << al_binary128_text(step.selected.metrics.primal)
                << "\",\"dual_mapping\":\""
                << al_binary128_text(step.selected.metrics.dual_mapping)
                << "\",\"complementarity\":\""
                << al_binary128_text(step.selected.metrics.complementarity)
                << "\",\"stationarity\":\""
                << al_binary128_text(step.selected.metrics.stationarity)
                << "\",\"gap_scaled\":\""
                << al_binary128_text(step.selected.metrics.gap_scaled)
                << "\",\"root\":\"" << step.selected.root << "\"}";
        } else {
            report << ",\"selected\":null";
        }
        report << ",\"root\":\"" << step.root << "\"}";
    }
    report << "]"
        << ",\"final_trials\":" << final_trials
        << ",\"final_stable_trials\":" << final_stable_trials
        << ",\"final_stable_rejections\":" << final_stable_rejections
        << ",\"final_crossing_trials\":" << final_crossing_trials
        << ",\"final_strict_negative_margins\":"
        << final_strict_negative_margins
        << ",\"final_unsigned_margins\":" << final_unsigned_margins
        << ",\"all_final_trials_crossing\":"
        << (all_final_trials_crossing ? "true" : "false")
        << ",\"minimum_absolute_margin\":\""
        << al_binary128_text(minimum_absolute_margin)
        << "\",\"minimum_absolute_stable_margin\":\""
        << al_binary128_text(minimum_absolute_stable_margin)
        << "\",\"solver_executions\":" << solver_executions
        << ",\"new_line_trials\":0,\"state_updates\":0"
        << ",\"parameter_changes\":0,\"timing_admitted\":false"
        << ",\"solver_authority\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {audit_exact, report.str()};
}
