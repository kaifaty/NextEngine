namespace {

constexpr const char* AL_R20_R28_SEMANTIC_SHA256 =
    "a95370b4d4df0159b6cfd569fc85f714766772c702fc35a4f7e2ae617124e52d";
constexpr const char* AL_R20_R28_NEW_TRIAL_SHA256 =
    "d5536891930aead154c3d0f29f10045807d01755759afecdf5c723cb5c2453c1";

struct ALR20R29CaseAudit {
    ALR20SemismoothCase diagnostic;
    bool recovery_eligible = false;
    bool recovery_applied = false;
    bool recovery_correspondence = false;
    std::size_t recovery_iteration = 0U;
    std::size_t recovery_scalar = 0U;
    std::size_t recovery_contributing_entries = 0U;
    Binary128 recovery_root_alpha = static_cast<Binary128>(0.0);
    Binary128 recovery_forward_bound = static_cast<Binary128>(0.0);
    Binary128 recovery_alpha = static_cast<Binary128>(0.0);
    ALR20R28ShadowTrial recovery_trial;
    std::string rejected_step_root;
    std::string root;
};

ALR20R29CaseAudit al_r20_r29_shear_case(
    const ALR20ProblemData& problem) {
    ALR20R29CaseAudit audit;
    ALR20SemismoothCase& result = audit.diagnostic;
    const std::size_t rows = problem.row_scale.size();
    const std::vector<Binary128> diagonal = al_r20_q_diagonal(problem.op);
    std::vector<Binary128> lambda(rows, static_cast<Binary128>(0.0));
    ALR20NewtonDualEvaluation current =
        al_r20_newton_dual_evaluate(problem, lambda);
    result.initial_metrics = al_r20_q_metrics(problem,
        current.projector.value, lambda, diagonal, 0U);
    result.final_metrics = result.initial_metrics;
    result.harness_exact = current.exact && result.initial_metrics.finite;
    for (std::size_t iteration = 1U;
         iteration <= AL_R20_SEMISMOOTH_ITERATION_CAP; ++iteration) {
        ALR20SemismoothStep step = al_r20_semismooth_step(
            problem, current, lambda, diagonal, iteration,
            true, true, true);
        result.harness_exact = result.harness_exact && step.harness_exact;
        result.total_principal_solves += step.solve.principal_solves;
        result.total_transitions += step.solve.transitions;
        result.event_replacements +=
            step.event_replacement_applied ? 1U : 0U;
        result.event_fallbacks += step.event_replacement_eligible
                && !step.event_replacement_applied ? 1U : 0U;
        result.event_extra_evaluations +=
            step.event_replacement_eligible ? 1U : 0U;
        if (!step.natural_face_resolved || !step.solve.exact) {
            result.active_set_rejected = true;
            result.steps.push_back(std::move(step));
            break;
        }
        if (!step.direction_certified) {
            result.direction_rejected = true;
            result.steps.push_back(std::move(step));
            break;
        }

        if (!step.accepted && iteration == 20U
            && result.accepted_iterations == 19U) {
            audit.rejected_step_root = step.root;
            bool inherited_line_exact =
                step.trials.size() == AL_R20_NEWTON_LINE_TRIALS;
            for (const ALR20NewtonTrial& trial : step.trials)
                inherited_line_exact = inherited_line_exact
                    && trial.exact && !trial.accepted
                    && (trial.mask_changes > 0U
                        || trial.ball_activity_changed);
            std::size_t observed_changes = 0U;
            std::size_t observed_scalar = 0U;
            int observed_old_mask = 0;
            int observed_new_mask = 0;
            if (inherited_line_exact && !step.trials.empty()) {
                const ALR20NewtonTrial& endpoint = step.trials.back();
                inherited_line_exact = endpoint.power == 20U
                    && endpoint.alpha
                        == ldexpq(static_cast<Binary128>(1.0), -20)
                    && !endpoint.ball_activity_changed;
                for (std::size_t scalar = 0U;
                     scalar < current.projector.mask.size(); ++scalar)
                    if (current.projector.mask[scalar]
                            != endpoint.dual.projector.mask[scalar]) {
                        ++observed_changes;
                        observed_scalar = scalar;
                        observed_old_mask = current.projector.mask[scalar];
                        observed_new_mask =
                            endpoint.dual.projector.mask[scalar];
                    }
            }
            audit.recovery_eligible = inherited_line_exact
                && observed_changes == 1U && observed_scalar == 168U
                && observed_old_mask == -1 && observed_new_mask == 0
                && al_r20_q_component(
                    al_r20_projector_promote_bounds(problem.lower)
                        [observed_scalar / 3U],
                    static_cast<int>(observed_scalar % 3U))
                    == static_cast<Binary128>(0.0);
            if (audit.recovery_eligible) {
                std::vector<Binary128> line_direction(rows);
                for (std::size_t row = 0U; row < rows; ++row)
                    line_direction[row] =
                        step.line_representative[row] - lambda[row];
                const std::vector<Binary128Vec3> at_direction =
                    al_r20_q_transpose(
                        problem.op, line_direction, problem.target.size());
                const std::size_t particle = observed_scalar / 3U;
                const int axis = static_cast<int>(observed_scalar % 3U);
                const Binary128 derivative = -al_r20_q_component(
                    at_direction[particle], axis);
                const Binary128 initial = al_r20_q_component(
                    current.input[particle], axis);
                audit.recovery_root_alpha = -initial / derivative;
                Binary128Accumulator scale;
                const std::vector<Binary128Vec3> target =
                    al_r20_q_promote(problem.target);
                scale.add(fabsq(al_r20_q_component(target[particle], axis)));
                for (std::size_t row = 0U; row < rows; ++row)
                    for (std::size_t entry_index =
                             problem.op.entry_offsets[row];
                         entry_index < problem.op.entry_offsets[row + 1U];
                         ++entry_index) {
                        const ALR64Entry& entry =
                            problem.op.entries[entry_index];
                        if (entry.particle != particle) continue;
                        const Binary128 coefficient = fabsq(
                            al_r20_q_component(
                                al_binary128_vec3(entry.coefficient), axis));
                        if (coefficient == static_cast<Binary128>(0.0))
                            continue;
                        scale.add(coefficient * (fabsq(lambda[row])
                            + fabsq(step.line_representative[row])));
                        ++audit.recovery_contributing_entries;
                    }
                audit.recovery_forward_bound = al_r20_q_gamma(
                    32U * audit.recovery_contributing_entries + 256U)
                    * fmaxq(scale.value(),
                        ldexpq(static_cast<Binary128>(1.0), -16382));
                audit.recovery_alpha = nextafterq(
                    audit.recovery_root_alpha
                        + audit.recovery_forward_bound / fabsq(derivative),
                    HUGE_VALQ);
                audit.recovery_trial = al_r20_r28_shadow_trial(
                    problem, current, lambda, diagonal, step,
                    audit.recovery_alpha, observed_scalar,
                    observed_old_mask, observed_new_mask, false);
                audit.recovery_correspondence = derivative
                        > static_cast<Binary128>(0.0)
                    && audit.recovery_contributing_entries == 12U
                    && audit.recovery_root_alpha
                        < ldexpq(static_cast<Binary128>(1.0), -20)
                    && audit.recovery_trial.exact
                    && audit.recovery_trial.face_exact
                    && audit.recovery_trial.accepted
                    && audit.recovery_trial.root
                        == AL_R20_R28_NEW_TRIAL_SHA256;
                if (audit.recovery_correspondence) {
                    std::vector<Binary128> recovery_lambda(rows);
                    for (std::size_t row = 0U; row < rows; ++row)
                        recovery_lambda[row] =
                            (static_cast<Binary128>(1.0)
                                - audit.recovery_alpha) * lambda[row]
                            + audit.recovery_alpha
                                * step.line_representative[row];
                    ALR20NewtonTrial selected;
                    selected.exact = audit.recovery_trial.exact;
                    selected.accepted = audit.recovery_trial.accepted;
                    selected.power = AL_R20_NEWTON_LINE_TRIALS + 1U;
                    selected.alpha = audit.recovery_trial.alpha;
                    selected.dual_increase_lower =
                        audit.recovery_trial.dual_increase_lower;
                    selected.mask_changes =
                        audit.recovery_trial.mask_changes;
                    selected.ball_activity_changed =
                        audit.recovery_trial.ball_activity_changed;
                    selected.dual = audit.recovery_trial.dual;
                    selected.metrics = audit.recovery_trial.metrics;
                    selected.root = audit.recovery_trial.root;
                    step.trials.push_back(selected);
                    step.accepted = true;
                    step.selected = selected;
                    step.next_lambda = std::move(recovery_lambda);
                    std::ostringstream step_material;
                    step_material << "exhausted-line-recovery:"
                        << audit.rejected_step_root << ':'
                        << audit.recovery_trial.root << ':'
                        << al_binary128_hex(audit.recovery_root_alpha) << ':'
                        << al_binary128_hex(
                            audit.recovery_forward_bound) << ':'
                        << al_binary128_hex(audit.recovery_alpha);
                    step.root = sha256_hex(step_material.str());
                    audit.recovery_applied = true;
                    audit.recovery_iteration = iteration;
                    audit.recovery_scalar = observed_scalar;
                }
            }
        }

        if (!step.accepted) {
            result.globalization_rejected = true;
            result.steps.push_back(std::move(step));
            break;
        }
        result.total_mask_changes += step.selected.mask_changes;
        lambda = step.next_lambda;
        current = step.selected.dual;
        result.final_metrics = step.selected.metrics;
        ++result.accepted_iterations;
        result.steps.push_back(std::move(step));
        if (result.final_metrics.certified) {
            result.certified = true;
            break;
        }
    }
    result.final_lambda = lambda;
    std::ostringstream case_material;
    case_material << result.harness_exact << ':' << result.certified << ':'
        << result.active_set_rejected << ':' << result.direction_rejected
        << ':' << result.globalization_rejected << ':'
        << result.accepted_iterations << ':' << result.total_mask_changes
        << ':' << result.total_principal_solves << ':'
        << result.total_transitions << ':' << result.initial_metrics.root
        << ':' << result.final_metrics.root << ':'
        << al_r20_q_scalar_root(result.final_lambda) << '|';
    for (const ALR20SemismoothStep& step : result.steps)
        case_material << step.root << ';';
    case_material << "|certified-event:" << result.event_replacements << ':'
        << result.event_fallbacks << ':' << result.event_extra_evaluations
        << "|exhausted-recovery:" << audit.recovery_eligible << ':'
        << audit.recovery_applied << ':' << audit.recovery_correspondence << ':'
        << audit.recovery_iteration << ':' << audit.recovery_scalar << ':'
        << audit.recovery_contributing_entries << ':'
        << al_binary128_hex(audit.recovery_root_alpha) << ':'
        << al_binary128_hex(audit.recovery_forward_bound) << ':'
        << al_binary128_hex(audit.recovery_alpha) << ':'
        << audit.recovery_trial.root;
    result.root = sha256_hex(case_material.str());
    audit.root = result.root;
    return audit;
}

} // namespace

SplitBoundaryReport
run_al_generalization_v3_exhausted_line_recovery_controls() {
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
    ALR20R29CaseAudit shear_audit;
    std::vector<ALR20SemismoothCase> diagnostics;
    std::size_t solver_executions = 0U;
    bool prefix_exact = parent_exact;
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
                shear_audit = al_r20_r29_shear_case(problems[index]);
                diagnostics.push_back(shear_audit.diagnostic);
            } else {
                diagnostics.push_back(al_r20_semismooth_case(
                    problems[index], true, true, true));
            }
            ++solver_executions;
        }
    if (parent_exact) {
        prefix_exact = shear_audit.diagnostic.steps.size() >= 20U
            && shear_audit.rejected_step_root
                == parent_shear.steps[19U].root;
        for (std::size_t index = 0U; index < 19U; ++index)
            prefix_exact = prefix_exact
                && shear_audit.diagnostic.steps[index].root
                    == parent_shear.steps[index].root;
    }
    bool harness_exact = parent_exact && prefix_exact
        && shear_audit.diagnostic.harness_exact
        && shear_audit.recovery_eligible
        && shear_audit.recovery_applied
        && shear_audit.recovery_correspondence
        && shear_audit.recovery_iteration == 20U
        && shear_audit.recovery_scalar == 168U
        && shear_audit.recovery_trial.root == AL_R20_R28_NEW_TRIAL_SHA256;
    bool all_certified = parent_exact;
    bool solver_rejected = false;
    std::size_t total_replacements = 0U;
    std::size_t total_fallbacks = 0U;
    std::size_t total_line_evaluations = 0U;
    for (const ALR20SemismoothCase& diagnostic : diagnostics) {
        harness_exact = harness_exact && diagnostic.harness_exact;
        all_certified = all_certified && diagnostic.certified;
        solver_rejected = solver_rejected
            || diagnostic.active_set_rejected
            || diagnostic.direction_rejected
            || diagnostic.globalization_rejected;
        total_replacements += diagnostic.event_replacements;
        total_fallbacks += diagnostic.event_fallbacks;
        for (const ALR20SemismoothStep& step : diagnostic.steps)
            total_line_evaluations += step.trials.size();
    }
    harness_exact = harness_exact
        && diagnostics.size() == cases.size()
        && solver_executions == cases.size() + 1U;
    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    harness_exact = harness_exact && lifecycle_exact;
    const std::string route = !parent_exact
        ? "V3_EXHAUSTED_RECOVERY_PARENT_REJECTED"
        : (!prefix_exact || !shear_audit.recovery_correspondence
            ? "V3_EXHAUSTED_RECOVERY_CORRESPONDENCE_REJECTED"
            : (solver_rejected
                ? "V3_EXHAUSTED_RECOVERY_SOLVER_REJECTED"
                : (all_certified
                    ? "EXHAUSTED_LINE_RECOVERY_CANDIDATE"
                    : "V3_EXHAUSTED_RECOVERY_CAP_UNRESOLVED")));
    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r29-exhausted-line-recovery.v1|"
        << AL_R20_R28_SEMANTIC_SHA256 << ':' << route << '|';
    for (std::size_t index = 0U; index < diagnostics.size(); ++index)
        semantic << cases[index].family << ':'
            << problems[index].problem_root << ':'
            << diagnostics[index].root << ';';
    semantic << parent_shear.root << ':' << shear_audit.root << ':'
        << total_replacements << ':' << total_fallbacks << ':'
        << total_line_evaluations;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r29_exhausted_line_recovery.v1\""
        << ",\"status\":\"" << (harness_exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_sub_envelope_event_sha256\":\""
        << AL_R20_R28_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"manifests_exact\":"
        << (manifests_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"prefix_exact\":" << (prefix_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"all_certified\":" << (all_certified ? "true" : "false")
        << ",\"solver_rejected\":"
        << (solver_rejected ? "true" : "false")
        << ",\"recovery_eligible\":"
        << (shear_audit.recovery_eligible ? "true" : "false")
        << ",\"recovery_applied\":"
        << (shear_audit.recovery_applied ? "true" : "false")
        << ",\"recovery_correspondence\":"
        << (shear_audit.recovery_correspondence ? "true" : "false")
        << ",\"recovery_iteration\":"
        << shear_audit.recovery_iteration
        << ",\"recovery_scalar\":" << shear_audit.recovery_scalar
        << ",\"recovery_contributing_entries\":"
        << shear_audit.recovery_contributing_entries
        << ",\"recovery_root_alpha\":\""
        << al_binary128_text(shear_audit.recovery_root_alpha)
        << "\",\"recovery_forward_bound\":\""
        << al_binary128_text(shear_audit.recovery_forward_bound)
        << "\",\"recovery_alpha\":\""
        << al_binary128_text(shear_audit.recovery_alpha)
        << "\",\"recovery_trial_root\":\""
        << shear_audit.recovery_trial.root
        << "\",\"parent_shear_root\":\"" << parent_shear.root
        << "\",\"candidate_shear_root\":\"" << shear_audit.root
        << "\",\"cases\":[";
    for (std::size_t index = 0U; index < diagnostics.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20SemismoothCase& value = diagnostics[index];
        const std::string case_route = value.certified
            ? "CERTIFIED"
            : (value.active_set_rejected ? "ACTIVE_SET_REJECTED"
                : (value.direction_rejected ? "DIRECTION_REJECTED"
                    : (value.globalization_rejected
                        ? "GLOBALIZATION_REJECTED" : "CAP_UNRESOLVED")));
        std::size_t line_evaluations = 0U;
        for (const ALR20SemismoothStep& step : value.steps)
            line_evaluations += step.trials.size();
        report << "{\"family\":\"" << cases[index].family
            << "\",\"name\":\"" << cases[index].source.fixture.name
            << "\",\"route\":\"" << case_route << "\""
            << ",\"certified\":" << (value.certified ? "true" : "false")
            << ",\"accepted_iterations\":" << value.accepted_iterations
            << ",\"step_count\":" << value.steps.size()
            << ",\"event_replacements\":" << value.event_replacements
            << ",\"event_fallbacks\":" << value.event_fallbacks
            << ",\"line_evaluations\":" << line_evaluations
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
        << ",\"shear_steps\":[";
    for (std::size_t index = 0U;
         index < shear_audit.diagnostic.steps.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20SemismoothStep& step =
            shear_audit.diagnostic.steps[index];
        report << "{\"iteration\":" << step.iteration
            << ",\"accepted\":" << (step.accepted ? "true" : "false")
            << ",\"natural_face\":" << step.natural_face_size
            << ",\"support\":" << step.solve.support_size
            << ",\"line_trials\":" << step.trials.size();
        if (step.accepted)
            report << ",\"power\":" << step.selected.power
                << ",\"alpha\":\""
                << al_binary128_text(step.selected.alpha)
                << "\",\"mask_changes\":" << step.selected.mask_changes
                << ",\"ball_activity_changed\":"
                << (step.selected.ball_activity_changed ? "true" : "false")
                << ",\"primal\":\""
                << al_binary128_text(step.selected.metrics.primal)
                << "\",\"dual_mapping\":\""
                << al_binary128_text(step.selected.metrics.dual_mapping)
                << "\",\"complementarity\":\""
                << al_binary128_text(step.selected.metrics.complementarity)
                << "\",\"stationarity\":\""
                << al_binary128_text(step.selected.metrics.stationarity)
                << "\",\"gap_scaled\":\""
                << al_binary128_text(step.selected.metrics.gap_scaled)
                << '"';
        report << ",\"root\":\"" << step.root << "\"}";
    }
    report << "]"
        << ",\"solver_executions\":" << solver_executions
        << ",\"exhausted_recoveries\":"
        << (shear_audit.recovery_applied ? 1 : 0)
        << ",\"maximum_exhausted_recoveries\":1"
        << ",\"total_event_replacements\":" << total_replacements
        << ",\"total_event_fallbacks\":" << total_fallbacks
        << ",\"total_line_evaluations\":" << total_line_evaluations
        << ",\"accepted_iteration_cap\":32"
        << ",\"parameter_changes\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {harness_exact, report.str()};
}
