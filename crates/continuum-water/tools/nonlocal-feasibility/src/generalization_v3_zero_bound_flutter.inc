namespace {

constexpr const char* AL_R20_V3_ULP_LADDER_SHA256 =
    "48d97d2110b3d3d5c6ebeff24a46c5183bb58063005a7af168b30d383aa11987";

struct ALR20MaskRun {
    int code = 0;
    std::size_t first = 0U;
    std::size_t last = 0U;
};

int al_r20_r24_sign(Binary128 value) {
    if (value < static_cast<Binary128>(0.0)) return -1;
    if (value > static_cast<Binary128>(0.0)) return 1;
    return 0;
}

} // namespace

SplitBoundaryReport run_al_generalization_v3_zero_bound_flutter_controls() {
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
                    && event.exact && shadow.exact && ladder.exact;
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
        && events.size() == 7U && shadows.size() == 7U
        && ladders.size() == 7U;
    std::ostringstream ladder_semantic;
    ladder_semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r23-v3-ulp-ladder.v1|"
        << AL_R20_V3_EVENT_SIDE_SHA256
        << ":V3_ULP_LADDER_IDENTIFIED:ULP_LADDER_MULTI_EVENT|";
    for (const ALR20UlpLadderAudit& ladder : ladders)
        ladder_semantic << ladder.root << ';';
    const std::string ladder_sha = sha256_hex(ladder_semantic.str());
    const bool r23_exact = lineage_exact
        && ladder_sha == AL_R20_V3_ULP_LADDER_SHA256;

    const ALR20UlpLadderAudit* selected = nullptr;
    for (const ALR20UlpLadderAudit& ladder : ladders)
        if (ladder.iteration == 32U) selected = &ladder;
    bool audit_exact = r23_exact && selected != nullptr
        && selected->face_codes.size() == AL_R20_EVENT_ULP_MAX_POWER + 1U
        && selected->actual_inputs.size() == selected->face_codes.size()
        && selected->analytic_inputs.size() == selected->face_codes.size();
    std::vector<ALR20MaskRun> runs;
    std::size_t old_count = 0U;
    std::size_t new_count = 0U;
    std::size_t other_count = 0U;
    std::size_t mask_transitions = 0U;
    std::size_t actual_sign_transitions = 0U;
    std::size_t analytic_sign_transitions = 0U;
    std::size_t final_new_suffix_first = AL_R20_EVENT_ULP_MAX_POWER + 1U;
    Binary128 maximum_difference = static_cast<Binary128>(0.0);
    bool analytic_monotone = audit_exact;
    bool analytic_nonnegative = audit_exact;
    bool old_after_new = false;
    bool seen_new = false;
    if (audit_exact) {
        for (std::size_t index = 0U; index < selected->face_codes.size(); ++index) {
            const int code = selected->face_codes[index];
            if (code == 0) {
                ++old_count;
                if (seen_new) old_after_new = true;
            } else if (code == 1) {
                ++new_count;
                seen_new = true;
            } else {
                ++other_count;
            }
            if (runs.empty() || runs.back().code != code)
                runs.push_back({code, index, index});
            else
                runs.back().last = index;
            if (index > 0U) {
                if (selected->face_codes[index - 1U] != code)
                    ++mask_transitions;
                if (al_r20_r24_sign(selected->actual_inputs[index - 1U])
                        != al_r20_r24_sign(selected->actual_inputs[index]))
                    ++actual_sign_transitions;
                if (al_r20_r24_sign(selected->analytic_inputs[index - 1U])
                        != al_r20_r24_sign(selected->analytic_inputs[index]))
                    ++analytic_sign_transitions;
                analytic_monotone = analytic_monotone
                    && selected->analytic_inputs[index]
                        >= selected->analytic_inputs[index - 1U];
            }
            analytic_nonnegative = analytic_nonnegative
                && selected->analytic_inputs[index]
                    >= static_cast<Binary128>(0.0);
            maximum_difference = fmaxq(maximum_difference,
                fabsq(selected->actual_inputs[index]
                    - selected->analytic_inputs[index]));
        }
        if (selected->face_codes.back() == 1) {
            final_new_suffix_first = AL_R20_EVENT_ULP_MAX_POWER;
            while (final_new_suffix_first > 0U
                && selected->face_codes[final_new_suffix_first - 1U] == 1)
                --final_new_suffix_first;
        }
    }
    std::string classification = "ZERO_BOUND_FLUTTER_UNRESOLVED";
    if (other_count > 0U || !analytic_monotone)
        classification = "GENUINE_SECOND_EVENT";
    else if (old_after_new && analytic_nonnegative
             && actual_sign_transitions > 1U)
        classification = "ZERO_BOUND_ROUNDING_FLUTTER";
    else if (!old_after_new && seen_new)
        classification = "LADDER_EVENT_STABLE";

    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const std::string route = !parent_exact
        ? "V3_ZERO_BOUND_FLUTTER_PARENT_REJECTED"
        : (!r23_exact ? "V3_ZERO_BOUND_FLUTTER_R23_REJECTED"
            : (!audit_exact ? "V3_ZERO_BOUND_FLUTTER_AUDIT_REJECTED"
                            : "V3_ZERO_BOUND_FLUTTER_IDENTIFIED"));
    const bool exact = parent_exact && r23_exact
        && audit_exact && lifecycle_exact;
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r24-v3-zero-bound-flutter.v1|"
        << AL_R20_V3_ULP_LADDER_SHA256 << ':' << route << ':'
        << classification << ':' << old_count << ':' << new_count << ':'
        << other_count << ':' << mask_transitions << ':'
        << actual_sign_transitions << ':' << analytic_sign_transitions << ':'
        << final_new_suffix_first << ':'
        << al_binary128_hex(maximum_difference) << '|';
    for (const ALR20MaskRun& run : runs)
        semantic << run.code << ':' << run.first << ':' << run.last << ';';
    if (selected != nullptr)
        for (std::size_t index = 0U; index < selected->face_codes.size(); ++index)
            semantic << selected->face_codes[index] << ':'
                << al_binary128_hex(selected->actual_inputs[index]) << ':'
                << al_binary128_hex(selected->analytic_inputs[index]) << ';';
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r24_v3_zero_bound_flutter.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"classification\":\"" << classification << "\""
        << ",\"parent_ulp_ladder_sha256\":\""
        << AL_R20_V3_ULP_LADDER_SHA256 << "\""
        << ",\"reproduced_ulp_ladder_sha256\":\"" << ladder_sha << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"r23_exact\":" << (r23_exact ? "true" : "false")
        << ",\"audit_exact\":" << (audit_exact ? "true" : "false")
        << ",\"lifecycle_exact\":" << (lifecycle_exact ? "true" : "false")
        << ",\"iteration\":32,\"predicted_scalar\":189"
        << ",\"old_count\":" << old_count
        << ",\"new_count\":" << new_count
        << ",\"other_count\":" << other_count
        << ",\"mask_transitions\":" << mask_transitions
        << ",\"actual_sign_transitions\":" << actual_sign_transitions
        << ",\"analytic_sign_transitions\":" << analytic_sign_transitions
        << ",\"analytic_monotone\":"
        << (analytic_monotone ? "true" : "false")
        << ",\"analytic_nonnegative\":"
        << (analytic_nonnegative ? "true" : "false")
        << ",\"old_after_new\":" << (old_after_new ? "true" : "false")
        << ",\"final_new_suffix_first\":" << final_new_suffix_first
        << ",\"maximum_actual_affine_difference\":\""
        << al_binary128_text(maximum_difference) << "\",\"runs\":[";
    for (std::size_t index = 0U; index < runs.size(); ++index) {
        if (index != 0U) report << ',';
        report << "{\"code\":" << runs[index].code
            << ",\"first\":" << runs[index].first
            << ",\"last\":" << runs[index].last << "}";
    }
    report << "]"
        << ",\"solver_executions\":" << solver_executions
        << ",\"inherited_ladder_points\":65,\"new_ladder_points\":0"
        << ",\"generated_solver_trials\":0,\"state_updates\":0"
        << ",\"cap_changes\":0,\"timing_admitted\":false"
        << ",\"solver_authority\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
