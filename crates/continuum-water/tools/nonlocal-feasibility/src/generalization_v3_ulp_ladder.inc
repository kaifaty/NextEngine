namespace {

constexpr const char* AL_R20_V3_EVENT_SIDE_SHA256 =
    "3c39b724b35e097d2208d5f318d7680755360fff6d29923ce3d8c758a8229ad6";
constexpr std::size_t AL_R20_EVENT_ULP_MAX_POWER = 64U;

struct ALR20UlpLadderAudit {
    bool exact = false;
    bool crossed = false;
    bool correct_face = false;
    bool first_armijo = false;
    bool later_event = false;
    std::size_t iteration = 0U;
    std::size_t predicted_scalar = 0U;
    std::size_t first_power = AL_R20_EVENT_ULP_MAX_POWER + 1U;
    std::size_t first_mask_changes = 0U;
    Binary128 root_alpha = static_cast<Binary128>(0.0);
    Binary128 ulp = static_cast<Binary128>(0.0);
    Binary128 first_alpha = static_cast<Binary128>(0.0);
    Binary128 first_displacement = static_cast<Binary128>(0.0);
    Binary128 first_margin = static_cast<Binary128>(0.0);
    Binary128 first_dual_increase_lower = static_cast<Binary128>(0.0);
    ALR20OracleMetrics first_metrics;
    std::vector<int> face_codes;
    std::vector<Binary128> actual_inputs;
    std::vector<Binary128> analytic_inputs;
    std::string first_mask_root;
    std::string root;
};

ALR20UlpLadderAudit al_r20_r23_ulp_ladder(
    const ALR20ProblemData& problem,
    const ALR20NewtonDualEvaluation& current,
    const std::vector<Binary128>& lambda,
    const std::vector<Binary128>& diagonal,
    const ALR20SemismoothStep& step,
    const ALR20BreakpointBracket& bracket,
    const ALR20EventBracketAudit& event) {
    ALR20UlpLadderAudit result;
    result.iteration = step.iteration;
    if (!event.exact || event.events.size() != 1U
        || step.line_representative.size() != lambda.size()) return result;
    result.predicted_scalar = event.predicted_scalar;
    result.root_alpha = event.predicted_alpha;
    result.ulp = nextafterq(result.root_alpha, HUGE_VALQ)
        - result.root_alpha;
    bool exact = result.ulp > static_cast<Binary128>(0.0);
    std::string crossing_mask;
    std::ostringstream sample_material;
    for (std::size_t power = 0U;
         power <= AL_R20_EVENT_ULP_MAX_POWER; ++power) {
        const Binary128 displacement = ldexpq(result.ulp,
            static_cast<int>(power));
        const Binary128 alpha = result.root_alpha + displacement;
        std::vector<Binary128> trial_lambda(lambda.size());
        bool cone_exact = alpha > static_cast<Binary128>(0.0)
            && alpha <= bracket.samples[
                bracket.first_mask_transition_cell + 1U].alpha;
        for (std::size_t row = 0U; row < lambda.size(); ++row) {
            trial_lambda[row] =
                (static_cast<Binary128>(1.0) - alpha) * lambda[row]
                + alpha * step.line_representative[row];
            cone_exact = cone_exact
                && trial_lambda[row] >= static_cast<Binary128>(0.0);
        }
        const ALR20NewtonDualEvaluation dual =
            al_r20_newton_dual_evaluate(problem, trial_lambda);
        ALR20NewtonTrial trial;
        trial.alpha = alpha;
        trial.dual = dual;
        const Binary128 margin =
            al_r20_r19_armijo_margin(current, step, trial);
        std::size_t mask_changes = 0U;
        bool predicted_changed = false;
        for (std::size_t scalar = 0U;
             scalar < current.projector.mask.size(); ++scalar) {
            if (dual.projector.mask[scalar]
                    == current.projector.mask[scalar]) continue;
            ++mask_changes;
            predicted_changed = predicted_changed
                || scalar == result.predicted_scalar;
        }
        const bool ball_changed = dual.projector.ball_active
            != current.projector.ball_active;
        int face_code = 2;
        if (!ball_changed && mask_changes == 0U) face_code = 0;
        else if (!ball_changed && mask_changes == 1U && predicted_changed)
            face_code = 1;
        result.face_codes.push_back(face_code);
        const std::size_t predicted_particle = result.predicted_scalar / 3U;
        const int predicted_axis = static_cast<int>(
            result.predicted_scalar % 3U);
        result.actual_inputs.push_back(al_r20_q_component(
            dual.input[predicted_particle], predicted_axis));
        const ALR20EventPolynomial& selected_polynomial =
            event.events.front().polynomial;
        result.analytic_inputs.push_back(
            selected_polynomial.zero_bound
                ? selected_polynomial.b * alpha + selected_polynomial.c
                : al_r20_r21_polynomial_value(selected_polynomial, alpha));
        const std::string mask_root = al_r20_r20_mask_root(dual.projector);
        const bool sample_exact = cone_exact && dual.exact
            && finiteq(alpha) != 0 && finiteq(margin) != 0;
        exact = exact && sample_exact;
        if (!result.crossed && (mask_changes > 0U || ball_changed)) {
            result.crossed = true;
            result.first_power = power;
            result.first_mask_changes = mask_changes;
            result.correct_face = !ball_changed && mask_changes == 1U
                && predicted_changed;
            result.first_armijo = margin
                >= static_cast<Binary128>(0.0);
            result.first_alpha = alpha;
            result.first_displacement = displacement;
            result.first_margin = margin;
            result.first_dual_increase_lower =
                dual.value - dual.bound - current.value - current.bound;
            result.first_metrics = al_r20_q_metrics(problem,
                dual.projector.value, trial_lambda, diagonal, step.iteration);
            result.first_mask_root = mask_root;
            exact = exact && result.first_metrics.finite;
            crossing_mask = mask_root;
        } else if (result.crossed
            && (ball_changed || mask_root != crossing_mask)) {
            result.later_event = true;
        }
        sample_material << power << ':' << sample_exact << ':'
            << al_binary128_hex(alpha) << ':'
            << al_binary128_hex(margin) << ':' << mask_changes << ':'
            << ball_changed << ':' << mask_root << ':' << dual.root << ';';
    }
    result.exact = exact;
    std::ostringstream material;
    material << result.exact << ':' << result.crossed << ':'
        << result.correct_face << ':' << result.first_armijo << ':'
        << result.later_event << ':' << result.iteration << ':'
        << result.predicted_scalar << ':' << result.first_power << ':'
        << result.first_mask_changes << ':'
        << al_binary128_hex(result.root_alpha) << ':'
        << al_binary128_hex(result.ulp) << ':'
        << al_binary128_hex(result.first_alpha) << ':'
        << al_binary128_hex(result.first_displacement) << ':'
        << al_binary128_hex(result.first_margin) << ':'
        << al_binary128_hex(result.first_dual_increase_lower) << ':'
        << result.first_mask_root << ':' << result.first_metrics.root << '|'
        << sample_material.str();
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v3_ulp_ladder_controls() {
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
    const bool reproduction_exact = parent_exact
        && diagnostic.root == AL_R20_V3_SHEAR_CANDIDATE_SHA256
        && diagnostic.steps.size() == 32U;
    const std::vector<Binary128> diagonal =
        al_r20_q_diagonal(problems[AL_R20_V3_SHEAR_INDEX].op);
    std::vector<ALR20BreakpointBracket> brackets;
    std::vector<ALR20EventBracketAudit> events;
    std::vector<ALR20EventSideShadow> shadows;
    std::vector<ALR20UlpLadderAudit> ladders;
    std::vector<Binary128> lambda(
        problems[AL_R20_V3_SHEAR_INDEX].row_scale.size(),
        static_cast<Binary128>(0.0));
    ALR20NewtonDualEvaluation current = al_r20_newton_dual_evaluate(
        problems[AL_R20_V3_SHEAR_INDEX], lambda);
    bool lineage_exact = reproduction_exact && current.exact;
    bool ladder_exact = reproduction_exact && current.exact;
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
                ALR20UlpLadderAudit ladder = al_r20_r23_ulp_ladder(
                    problems[AL_R20_V3_SHEAR_INDEX], current,
                    lambda, diagonal, step, bracket, event);
                lineage_exact = lineage_exact && bracket.exact
                    && event.exact && shadow.exact;
                ladder_exact = ladder_exact && ladder.exact;
                brackets.push_back(std::move(bracket));
                events.push_back(std::move(event));
                shadows.push_back(std::move(shadow));
                ladders.push_back(std::move(ladder));
            }
        }
        if (step.accepted) {
            lambda = step.next_lambda;
            current = step.selected.dual;
        }
    }
    lineage_exact = lineage_exact && brackets.size() == 7U
        && events.size() == 7U && shadows.size() == 7U;
    ladder_exact = ladder_exact && ladders.size() == 7U;

    std::ostringstream shadow_semantic;
    shadow_semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r22-v3-event-side.v1|"
        << AL_R20_V3_FIXED_FACE_EVENT_SHA256
        << ":V3_EVENT_SIDE_IDENTIFIED:NEXT_REPRESENTABLE_FACE_REJECTED|";
    for (const ALR20EventSideShadow& shadow : shadows)
        shadow_semantic << shadow.root << ';';
    const std::string shadow_sha = sha256_hex(shadow_semantic.str());
    const bool r22_exact = lineage_exact
        && shadow_sha == AL_R20_V3_EVENT_SIDE_SHA256;

    std::size_t correct_crossings = 0U;
    std::size_t no_crossings = 0U;
    std::size_t wrong_face_crossings = 0U;
    std::size_t armijo_rejections = 0U;
    std::size_t later_events = 0U;
    for (const ALR20UlpLadderAudit& ladder : ladders) {
        if (!ladder.crossed) ++no_crossings;
        else if (!ladder.correct_face) ++wrong_face_crossings;
        else {
            ++correct_crossings;
            if (!ladder.first_armijo) ++armijo_rejections;
        }
        if (ladder.later_event) ++later_events;
    }
    std::string classification = "ULP_LADDER_UNRESOLVED";
    if (wrong_face_crossings > 0U)
        classification = "ULP_LADDER_WRONG_FACE";
    else if (no_crossings > 0U)
        classification = "ULP_LADDER_NO_CROSSING";
    else if (armijo_rejections > 0U)
        classification = "ULP_LADDER_ARMIJO_REJECTED";
    else if (later_events > 0U)
        classification = "ULP_LADDER_MULTI_EVENT";
    else if (correct_crossings == 7U)
        classification = "ULP_LADDER_EVENT_SIDE_CANDIDATE";

    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const std::string route = !parent_exact
        ? "V3_ULP_LADDER_PARENT_REJECTED"
        : (!r22_exact ? "V3_ULP_LADDER_R22_REJECTED"
            : (!ladder_exact ? "V3_ULP_LADDER_AUDIT_REJECTED"
                             : "V3_ULP_LADDER_IDENTIFIED"));
    const bool exact = parent_exact && r22_exact
        && ladder_exact && lifecycle_exact;
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r23-v3-ulp-ladder.v1|"
        << AL_R20_V3_EVENT_SIDE_SHA256 << ':' << route << ':'
        << classification << '|';
    for (const ALR20UlpLadderAudit& ladder : ladders)
        semantic << ladder.root << ';';
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r23_v3_ulp_ladder.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"classification\":\"" << classification << "\""
        << ",\"parent_event_side_sha256\":\""
        << AL_R20_V3_EVENT_SIDE_SHA256 << "\""
        << ",\"reproduced_event_side_sha256\":\"" << shadow_sha << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"r22_exact\":" << (r22_exact ? "true" : "false")
        << ",\"ladder_exact\":" << (ladder_exact ? "true" : "false")
        << ",\"lifecycle_exact\":" << (lifecycle_exact ? "true" : "false")
        << ",\"correct_crossings\":" << correct_crossings
        << ",\"no_crossings\":" << no_crossings
        << ",\"wrong_face_crossings\":" << wrong_face_crossings
        << ",\"armijo_rejections\":" << armijo_rejections
        << ",\"later_events\":" << later_events
        << ",\"ladders\":[";
    for (std::size_t index = 0U; index < ladders.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20UlpLadderAudit& ladder = ladders[index];
        report << "{\"iteration\":" << ladder.iteration
            << ",\"predicted_scalar\":" << ladder.predicted_scalar
            << ",\"crossed\":" << (ladder.crossed ? "true" : "false")
            << ",\"correct_face\":"
            << (ladder.correct_face ? "true" : "false")
            << ",\"first_armijo\":"
            << (ladder.first_armijo ? "true" : "false")
            << ",\"later_event\":"
            << (ladder.later_event ? "true" : "false")
            << ",\"first_power\":" << ladder.first_power
            << ",\"first_mask_changes\":" << ladder.first_mask_changes
            << ",\"root_alpha\":\""
            << al_binary128_text(ladder.root_alpha)
            << "\",\"ulp\":\"" << al_binary128_text(ladder.ulp)
            << "\",\"first_alpha\":\""
            << al_binary128_text(ladder.first_alpha)
            << "\",\"first_displacement\":\""
            << al_binary128_text(ladder.first_displacement)
            << "\",\"first_margin\":\""
            << al_binary128_text(ladder.first_margin)
            << "\",\"first_dual_increase_lower\":\""
            << al_binary128_text(ladder.first_dual_increase_lower)
            << "\",\"primal\":\""
            << al_binary128_text(ladder.first_metrics.primal)
            << "\",\"dual_mapping\":\""
            << al_binary128_text(ladder.first_metrics.dual_mapping)
            << "\",\"root\":\"" << ladder.root << "\"}";
    }
    report << "]"
        << ",\"solver_executions\":" << solver_executions
        << ",\"ladder_evaluations\":"
        << ladders.size() * (AL_R20_EVENT_ULP_MAX_POWER + 1U)
        << ",\"generated_solver_trials\":0,\"state_updates\":0"
        << ",\"cap_changes\":0,\"timing_admitted\":false"
        << ",\"solver_authority\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
