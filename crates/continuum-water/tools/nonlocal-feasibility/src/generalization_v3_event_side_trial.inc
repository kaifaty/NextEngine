namespace {

constexpr const char* AL_R20_V3_FIXED_FACE_EVENT_SHA256 =
    "3ff49117f8ad70f55c345e4a9f9459bd11bf994125267baf04891c89ec857bca";

struct ALR20EventSideShadow {
    bool exact = false;
    bool root_face_valid = false;
    bool after_face_valid = false;
    bool after_armijo = false;
    std::size_t iteration = 0U;
    std::size_t predicted_scalar = 0U;
    std::size_t root_mask_changes = 0U;
    std::size_t after_mask_changes = 0U;
    Binary128 root_alpha = static_cast<Binary128>(0.0);
    Binary128 after_alpha = static_cast<Binary128>(0.0);
    Binary128 ulp = static_cast<Binary128>(0.0);
    Binary128 root_margin = static_cast<Binary128>(0.0);
    Binary128 after_margin = static_cast<Binary128>(0.0);
    Binary128 after_dual_increase_lower = static_cast<Binary128>(0.0);
    ALR20NewtonDualEvaluation root_dual;
    ALR20NewtonDualEvaluation after_dual;
    ALR20OracleMetrics root_metrics;
    ALR20OracleMetrics after_metrics;
    std::string root;
};

ALR20EventSideShadow al_r20_r22_event_side_shadow(
    const ALR20ProblemData& problem,
    const ALR20NewtonDualEvaluation& current,
    const std::vector<Binary128>& lambda,
    const std::vector<Binary128>& diagonal,
    const ALR20SemismoothStep& step,
    const ALR20EventBracketAudit& audit) {
    ALR20EventSideShadow result;
    result.iteration = step.iteration;
    if (!audit.exact || audit.events.size() != 1U
        || step.line_representative.size() != lambda.size()) return result;
    result.predicted_scalar = audit.predicted_scalar;
    result.root_alpha = audit.predicted_alpha;
    result.after_alpha = nextafterq(result.root_alpha, HUGE_VALQ);
    result.ulp = result.after_alpha - result.root_alpha;
    const auto evaluate = [&](Binary128 alpha,
                              ALR20NewtonDualEvaluation* dual,
                              ALR20OracleMetrics* metrics,
                              Binary128* margin,
                              std::size_t* mask_changes) {
        std::vector<Binary128> trial_lambda(lambda.size());
        bool cone_exact = true;
        for (std::size_t row = 0U; row < lambda.size(); ++row) {
            trial_lambda[row] =
                (static_cast<Binary128>(1.0) - alpha) * lambda[row]
                + alpha * step.line_representative[row];
            cone_exact = cone_exact
                && trial_lambda[row] >= static_cast<Binary128>(0.0);
        }
        *dual = al_r20_newton_dual_evaluate(problem, trial_lambda);
        ALR20NewtonTrial trial;
        trial.alpha = alpha;
        trial.dual = *dual;
        *margin = al_r20_r19_armijo_margin(current, step, trial);
        *metrics = al_r20_q_metrics(problem, dual->projector.value,
            trial_lambda, diagonal, step.iteration);
        for (std::size_t scalar = 0U;
             scalar < current.projector.mask.size(); ++scalar)
            if (dual->projector.mask[scalar]
                    != current.projector.mask[scalar])
                ++*mask_changes;
        return cone_exact && dual->exact && metrics->finite
            && finiteq(*margin) != 0;
    };
    const bool root_exact = evaluate(result.root_alpha,
        &result.root_dual, &result.root_metrics,
        &result.root_margin, &result.root_mask_changes);
    const bool after_exact = evaluate(result.after_alpha,
        &result.after_dual, &result.after_metrics,
        &result.after_margin, &result.after_mask_changes);
    const auto only_predicted_changed = [&](const ALR20ProjectorState& state,
                                             std::size_t expected_changes) {
        std::size_t changes = 0U;
        bool predicted_changed = false;
        for (std::size_t scalar = 0U;
             scalar < current.projector.mask.size(); ++scalar) {
            if (state.mask[scalar] == current.projector.mask[scalar]) continue;
            ++changes;
            predicted_changed = predicted_changed
                || scalar == result.predicted_scalar;
        }
        return state.ball_active == current.projector.ball_active
            && changes == expected_changes
            && (expected_changes == 0U || predicted_changed);
    };
    result.root_face_valid = only_predicted_changed(
            result.root_dual.projector, 0U)
        || only_predicted_changed(result.root_dual.projector, 1U);
    result.after_face_valid = only_predicted_changed(
        result.after_dual.projector, 1U);
    result.after_armijo = result.after_margin
        >= static_cast<Binary128>(0.0);
    result.after_dual_increase_lower =
        result.after_dual.value - result.after_dual.bound
        - current.value - current.bound;
    result.exact = root_exact && after_exact
        && result.root_alpha > static_cast<Binary128>(0.0)
        && result.after_alpha > result.root_alpha
        && result.ulp > static_cast<Binary128>(0.0)
        && result.root_face_valid;
    std::ostringstream material;
    material << result.exact << ':' << result.root_face_valid << ':'
        << result.after_face_valid << ':' << result.after_armijo << ':'
        << result.iteration << ':' << result.predicted_scalar << ':'
        << result.root_mask_changes << ':' << result.after_mask_changes << ':'
        << al_binary128_hex(result.root_alpha) << ':'
        << al_binary128_hex(result.after_alpha) << ':'
        << al_binary128_hex(result.ulp) << ':'
        << al_binary128_hex(result.root_margin) << ':'
        << al_binary128_hex(result.after_margin) << ':'
        << al_binary128_hex(result.after_dual_increase_lower) << ':'
        << result.root_dual.root << ':' << result.after_dual.root << ':'
        << result.root_metrics.root << ':' << result.after_metrics.root;
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v3_event_side_trial_controls() {
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

    const std::vector<Binary128> diagonal =
        al_r20_q_diagonal(problems[AL_R20_V3_SHEAR_INDEX].op);
    std::vector<ALR20BreakpointBracket> brackets;
    std::vector<ALR20EventBracketAudit> event_audits;
    std::vector<ALR20EventSideShadow> shadows;
    std::vector<Binary128> lambda(
        problems[AL_R20_V3_SHEAR_INDEX].row_scale.size(),
        static_cast<Binary128>(0.0));
    ALR20NewtonDualEvaluation current = al_r20_newton_dual_evaluate(
        problems[AL_R20_V3_SHEAR_INDEX], lambda);
    bool breakpoint_exact = reproduction_exact && current.exact;
    bool event_exact = reproduction_exact && current.exact;
    bool shadow_exact = reproduction_exact && current.exact;
    for (const ALR20SemismoothStep& step : diagnostic.steps) {
        if (step.iteration >= AL_R20_V3_LATE_FIRST && step.accepted
            && step.selected.power > 0U && step.selected.mask_changes == 0U) {
            bool all_prior_crossing = true;
            for (const ALR20NewtonTrial& trial : step.trials)
                if (trial.power < step.selected.power)
                    all_prior_crossing = all_prior_crossing
                        && (trial.mask_changes > 0U
                            || trial.ball_activity_changed);
            if (all_prior_crossing) {
                ALR20BreakpointBracket bracket =
                    al_r20_r20_breakpoint_bracket(
                        problems[AL_R20_V3_SHEAR_INDEX], current,
                        lambda, step);
                ALR20EventBracketAudit event = al_r20_r21_event_audit(
                    problems[AL_R20_V3_SHEAR_INDEX], current,
                    lambda, step, bracket);
                ALR20EventSideShadow shadow = al_r20_r22_event_side_shadow(
                    problems[AL_R20_V3_SHEAR_INDEX], current,
                    lambda, diagonal, step, event);
                breakpoint_exact = breakpoint_exact && bracket.exact;
                event_exact = event_exact && event.exact;
                shadow_exact = shadow_exact && shadow.exact;
                brackets.push_back(std::move(bracket));
                event_audits.push_back(std::move(event));
                shadows.push_back(std::move(shadow));
            }
        }
        if (step.accepted) {
            lambda = step.next_lambda;
            current = step.selected.dual;
        }
    }
    breakpoint_exact = breakpoint_exact && brackets.size() == 7U;
    event_exact = event_exact && event_audits.size() == 7U;
    shadow_exact = shadow_exact && shadows.size() == 7U;

    std::ostringstream geometry_semantic;
    geometry_semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r20-v3-breakpoint-geometry.v1|"
        << AL_R20_V3_LINE_FRONTIER_SHA256
        << ":V3_BREAKPOINT_GEOMETRY_IDENTIFIED:SIMPLE_BREAKPOINT_OFFSET|";
    for (const ALR20BreakpointBracket& bracket : brackets)
        geometry_semantic << bracket.root << ';';
    const std::string geometry_sha = sha256_hex(geometry_semantic.str());
    const bool r20_exact = breakpoint_exact
        && geometry_sha == AL_R20_V3_BREAKPOINT_GEOMETRY_SHA256;
    std::ostringstream event_semantic;
    event_semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r21-v3-fixed-face-event.v1|"
        << AL_R20_V3_BREAKPOINT_GEOMETRY_SHA256
        << ":V3_FIXED_FACE_EVENT_IDENTIFIED:FIXED_FACE_EVENT_PREDICTOR_CANDIDATE|";
    for (const ALR20EventBracketAudit& event : event_audits)
        event_semantic << event.root << ';';
    const std::string event_sha = sha256_hex(event_semantic.str());
    const bool r21_exact = r20_exact && event_exact
        && event_sha == AL_R20_V3_FIXED_FACE_EVENT_SHA256;

    std::size_t face_rejections = 0U;
    std::size_t armijo_acceptances = 0U;
    std::size_t armijo_rejections = 0U;
    bool matches_offset_subset = true;
    for (std::size_t index = 0U; index < shadows.size(); ++index) {
        const ALR20EventSideShadow& shadow = shadows[index];
        if (!shadow.after_face_valid) ++face_rejections;
        if (shadow.after_face_valid && shadow.after_armijo)
            ++armijo_acceptances;
        else if (shadow.after_face_valid)
            ++armijo_rejections;
        const bool offset_bracket =
            brackets[index].first_mask_transition_cell
                != brackets[index].first_armijo_transition_cell;
        matches_offset_subset = matches_offset_subset
            && shadow.after_armijo == offset_bracket;
    }
    std::string classification = "EVENT_SIDE_UNRESOLVED";
    if (face_rejections > 0U)
        classification = "NEXT_REPRESENTABLE_FACE_REJECTED";
    else if (armijo_rejections > 0U && armijo_acceptances != 5U)
        classification = "EVENT_SIDE_ARMIJO_REJECTED";
    else if (armijo_acceptances == 5U && matches_offset_subset)
        classification = "EVENT_SIDE_OFFSET_SUBSET";
    else if (armijo_acceptances == 7U)
        classification = "EVENT_SIDE_TRIAL_CANDIDATE";

    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const std::string route = !parent_exact
        ? "V3_EVENT_SIDE_PARENT_REJECTED"
        : (!r21_exact ? "V3_EVENT_SIDE_R21_REJECTED"
            : (!shadow_exact ? "V3_EVENT_SIDE_AUDIT_REJECTED"
                             : "V3_EVENT_SIDE_IDENTIFIED"));
    const bool exact = parent_exact && r21_exact
        && shadow_exact && lifecycle_exact;
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r22-v3-event-side.v1|"
        << AL_R20_V3_FIXED_FACE_EVENT_SHA256 << ':' << route << ':'
        << classification << '|';
    for (const ALR20EventSideShadow& shadow : shadows)
        semantic << shadow.root << ';';
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r22_v3_event_side.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"classification\":\"" << classification << "\""
        << ",\"parent_fixed_face_event_sha256\":\""
        << AL_R20_V3_FIXED_FACE_EVENT_SHA256 << "\""
        << ",\"reproduced_fixed_face_event_sha256\":\""
        << event_sha << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"r20_exact\":" << (r20_exact ? "true" : "false")
        << ",\"r21_exact\":" << (r21_exact ? "true" : "false")
        << ",\"shadow_exact\":" << (shadow_exact ? "true" : "false")
        << ",\"lifecycle_exact\":" << (lifecycle_exact ? "true" : "false")
        << ",\"face_rejections\":" << face_rejections
        << ",\"armijo_acceptances\":" << armijo_acceptances
        << ",\"armijo_rejections\":" << armijo_rejections
        << ",\"matches_offset_subset\":"
        << (matches_offset_subset ? "true" : "false")
        << ",\"shadows\":[";
    for (std::size_t index = 0U; index < shadows.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20EventSideShadow& shadow = shadows[index];
        report << "{\"iteration\":" << shadow.iteration
            << ",\"predicted_scalar\":" << shadow.predicted_scalar
            << ",\"root_face_valid\":"
            << (shadow.root_face_valid ? "true" : "false")
            << ",\"after_face_valid\":"
            << (shadow.after_face_valid ? "true" : "false")
            << ",\"after_armijo\":"
            << (shadow.after_armijo ? "true" : "false")
            << ",\"root_mask_changes\":" << shadow.root_mask_changes
            << ",\"after_mask_changes\":" << shadow.after_mask_changes
            << ",\"root_alpha\":\""
            << al_binary128_text(shadow.root_alpha)
            << "\",\"after_alpha\":\""
            << al_binary128_text(shadow.after_alpha)
            << "\",\"ulp\":\"" << al_binary128_text(shadow.ulp)
            << "\",\"root_margin\":\""
            << al_binary128_text(shadow.root_margin)
            << "\",\"after_margin\":\""
            << al_binary128_text(shadow.after_margin)
            << "\",\"after_dual_increase_lower\":\""
            << al_binary128_text(shadow.after_dual_increase_lower)
            << "\",\"primal\":\""
            << al_binary128_text(shadow.after_metrics.primal)
            << "\",\"dual_mapping\":\""
            << al_binary128_text(shadow.after_metrics.dual_mapping)
            << "\",\"complementarity\":\""
            << al_binary128_text(shadow.after_metrics.complementarity)
            << "\",\"stationarity\":\""
            << al_binary128_text(shadow.after_metrics.stationarity)
            << "\",\"root\":\"" << shadow.root << "\"}";
    }
    report << "]"
        << ",\"solver_executions\":" << solver_executions
        << ",\"shadow_evaluations\":" << shadows.size() * 2U
        << ",\"generated_solver_trials\":0,\"state_updates\":0"
        << ",\"cap_changes\":0,\"timing_admitted\":false"
        << ",\"solver_authority\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
