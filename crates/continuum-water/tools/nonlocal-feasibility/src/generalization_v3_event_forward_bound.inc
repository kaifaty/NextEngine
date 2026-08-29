namespace {

constexpr const char* AL_R20_V3_ZERO_BOUND_FLUTTER_SHA256 =
    "9f3c90728389047d56f777750f9941a33ced533ef97975405f534e69e6d6d67b";

struct ALR20ForwardBoundAudit {
    bool exact = false;
    bool contained = false;
    bool certified_side_found = false;
    bool certified_suffix_valid = false;
    std::size_t iteration = 0U;
    std::size_t predicted_scalar = 0U;
    std::size_t contributing_entries = 0U;
    std::size_t first_certified_power = AL_R20_EVENT_ULP_MAX_POWER + 1U;
    Binary128 scale = static_cast<Binary128>(0.0);
    Binary128 bound = static_cast<Binary128>(0.0);
    Binary128 maximum_discrepancy = static_cast<Binary128>(0.0);
    Binary128 containment_margin = static_cast<Binary128>(0.0);
    Binary128 first_certified_lower = static_cast<Binary128>(0.0);
    std::string root;
};

ALR20ForwardBoundAudit al_r20_r25_forward_bound(
    const ALR20ProblemData& problem,
    const std::vector<Binary128>& lambda,
    const ALR20SemismoothStep& step,
    const ALR20UlpLadderAudit& ladder) {
    ALR20ForwardBoundAudit result;
    result.iteration = step.iteration;
    result.predicted_scalar = ladder.predicted_scalar;
    const std::size_t particle = result.predicted_scalar / 3U;
    const int axis = static_cast<int>(result.predicted_scalar % 3U);
    const std::vector<Binary128Vec3> target = al_r20_q_promote(problem.target);
    Binary128Accumulator scale;
    scale.add(fabsq(al_r20_q_component(target[particle], axis)));
    for (std::size_t row = 0U; row < lambda.size(); ++row)
        for (std::size_t index = problem.op.entry_offsets[row];
             index < problem.op.entry_offsets[row + 1U]; ++index) {
            const ALR64Entry& entry = problem.op.entries[index];
            if (entry.particle != particle) continue;
            const Binary128 coefficient = fabsq(al_r20_q_component(
                al_binary128_vec3(entry.coefficient), axis));
            if (coefficient == static_cast<Binary128>(0.0)) continue;
            scale.add(coefficient * (fabsq(lambda[row])
                + fabsq(step.line_representative[row])));
            ++result.contributing_entries;
        }
    result.scale = scale.value();
    result.bound = al_r20_q_gamma(
        32U * result.contributing_entries + 256U)
        * fmaxq(result.scale,
            ldexpq(static_cast<Binary128>(1.0), -16382));
    bool exact = ladder.exact && result.contributing_entries > 0U
        && ladder.actual_inputs.size() == AL_R20_EVENT_ULP_MAX_POWER + 1U
        && ladder.analytic_inputs.size() == ladder.actual_inputs.size()
        && ladder.face_codes.size() == ladder.actual_inputs.size()
        && ladder.margins.size() == ladder.actual_inputs.size()
        && finiteq(result.scale) != 0 && finiteq(result.bound) != 0
        && result.bound > static_cast<Binary128>(0.0);
    result.maximum_discrepancy = static_cast<Binary128>(0.0);
    result.contained = exact;
    for (std::size_t power = 0U;
         power < ladder.actual_inputs.size(); ++power) {
        const Binary128 discrepancy = fabsq(
            ladder.actual_inputs[power] - ladder.analytic_inputs[power]);
        result.maximum_discrepancy = fmaxq(
            result.maximum_discrepancy, discrepancy);
        result.contained = result.contained && discrepancy <= result.bound;
        const Binary128 certified_lower =
            ladder.analytic_inputs[power] - result.bound;
        if (!result.certified_side_found
            && certified_lower > static_cast<Binary128>(0.0)) {
            result.certified_side_found = true;
            result.first_certified_power = power;
            result.first_certified_lower = certified_lower;
        }
    }
    result.containment_margin = result.bound - result.maximum_discrepancy;
    result.certified_suffix_valid = result.certified_side_found;
    if (result.certified_side_found)
        for (std::size_t power = result.first_certified_power;
             power < ladder.face_codes.size(); ++power)
            result.certified_suffix_valid = result.certified_suffix_valid
                && ladder.face_codes[power] == 1
                && ladder.margins[power]
                    >= static_cast<Binary128>(0.0);
    result.exact = exact;
    std::ostringstream material;
    material << result.exact << ':' << result.contained << ':'
        << result.certified_side_found << ':'
        << result.certified_suffix_valid << ':' << result.iteration << ':'
        << result.predicted_scalar << ':' << result.contributing_entries << ':'
        << result.first_certified_power << ':'
        << al_binary128_hex(result.scale) << ':'
        << al_binary128_hex(result.bound) << ':'
        << al_binary128_hex(result.maximum_discrepancy) << ':'
        << al_binary128_hex(result.containment_margin) << ':'
        << al_binary128_hex(result.first_certified_lower) << ':'
        << ladder.root;
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v3_event_forward_bound_controls() {
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
    std::vector<ALR20ForwardBoundAudit> audits;
    std::vector<Binary128> lambda(
        problems[AL_R20_V3_SHEAR_INDEX].row_scale.size(),
        static_cast<Binary128>(0.0));
    ALR20NewtonDualEvaluation current = al_r20_newton_dual_evaluate(
        problems[AL_R20_V3_SHEAR_INDEX], lambda);
    bool lineage_exact = reproduction_exact && current.exact;
    bool audit_exact = reproduction_exact && current.exact;
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
                ALR20ForwardBoundAudit audit = al_r20_r25_forward_bound(
                    problems[AL_R20_V3_SHEAR_INDEX], lambda, step, ladder);
                lineage_exact = lineage_exact && bracket.exact
                    && event.exact && shadow.exact && ladder.exact;
                audit_exact = audit_exact && audit.exact;
                brackets.push_back(std::move(bracket));
                events.push_back(std::move(event));
                shadows.push_back(std::move(shadow));
                ladders.push_back(std::move(ladder));
                audits.push_back(std::move(audit));
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
    audit_exact = audit_exact && audits.size() == 7U;

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

    const ALR20UlpLadderAudit* flutter = nullptr;
    for (const ALR20UlpLadderAudit& ladder : ladders)
        if (ladder.iteration == 32U) flutter = &ladder;
    std::vector<ALR20MaskRun> runs;
    std::size_t old_count = 0U;
    std::size_t new_count = 0U;
    std::size_t other_count = 0U;
    std::size_t mask_transitions = 0U;
    std::size_t actual_sign_transitions = 0U;
    std::size_t analytic_sign_transitions = 0U;
    std::size_t final_new_suffix_first = AL_R20_EVENT_ULP_MAX_POWER + 1U;
    Binary128 maximum_difference = static_cast<Binary128>(0.0);
    bool analytic_monotone = flutter != nullptr;
    bool analytic_nonnegative = flutter != nullptr;
    bool old_after_new = false;
    bool seen_new = false;
    if (flutter != nullptr) {
        for (std::size_t index = 0U; index < flutter->face_codes.size(); ++index) {
            const int code = flutter->face_codes[index];
            if (code == 0) {
                ++old_count;
                if (seen_new) old_after_new = true;
            } else if (code == 1) {
                ++new_count;
                seen_new = true;
            } else ++other_count;
            if (runs.empty() || runs.back().code != code)
                runs.push_back({code, index, index});
            else runs.back().last = index;
            if (index > 0U) {
                if (flutter->face_codes[index - 1U] != code) ++mask_transitions;
                if (al_r20_r24_sign(flutter->actual_inputs[index - 1U])
                        != al_r20_r24_sign(flutter->actual_inputs[index]))
                    ++actual_sign_transitions;
                if (al_r20_r24_sign(flutter->analytic_inputs[index - 1U])
                        != al_r20_r24_sign(flutter->analytic_inputs[index]))
                    ++analytic_sign_transitions;
                analytic_monotone = analytic_monotone
                    && flutter->analytic_inputs[index]
                        >= flutter->analytic_inputs[index - 1U];
            }
            analytic_nonnegative = analytic_nonnegative
                && flutter->analytic_inputs[index]
                    >= static_cast<Binary128>(0.0);
            maximum_difference = fmaxq(maximum_difference,
                fabsq(flutter->actual_inputs[index]
                    - flutter->analytic_inputs[index]));
        }
        if (flutter->face_codes.back() == 1) {
            final_new_suffix_first = AL_R20_EVENT_ULP_MAX_POWER;
            while (final_new_suffix_first > 0U
                && flutter->face_codes[final_new_suffix_first - 1U] == 1)
                --final_new_suffix_first;
        }
    }
    std::ostringstream flutter_semantic;
    flutter_semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r24-v3-zero-bound-flutter.v1|"
        << AL_R20_V3_ULP_LADDER_SHA256
        << ":V3_ZERO_BOUND_FLUTTER_IDENTIFIED:ZERO_BOUND_ROUNDING_FLUTTER:"
        << old_count << ':' << new_count << ':' << other_count << ':'
        << mask_transitions << ':' << actual_sign_transitions << ':'
        << analytic_sign_transitions << ':' << final_new_suffix_first << ':'
        << al_binary128_hex(maximum_difference) << '|';
    for (const ALR20MaskRun& run : runs)
        flutter_semantic << run.code << ':' << run.first << ':' << run.last << ';';
    if (flutter != nullptr)
        for (std::size_t index = 0U; index < flutter->face_codes.size(); ++index)
            flutter_semantic << flutter->face_codes[index] << ':'
                << al_binary128_hex(flutter->actual_inputs[index]) << ':'
                << al_binary128_hex(flutter->analytic_inputs[index]) << ';';
    const std::string flutter_sha = sha256_hex(flutter_semantic.str());
    const bool r24_exact = r23_exact && analytic_monotone
        && analytic_nonnegative && old_after_new
        && flutter_sha == AL_R20_V3_ZERO_BOUND_FLUTTER_SHA256;

    std::size_t containment_rejections = 0U;
    std::size_t no_certified_sides = 0U;
    std::size_t suffix_rejections = 0U;
    std::size_t certified_candidates = 0U;
    for (const ALR20ForwardBoundAudit& audit : audits) {
        if (!audit.contained) ++containment_rejections;
        else if (!audit.certified_side_found) ++no_certified_sides;
        else if (!audit.certified_suffix_valid) ++suffix_rejections;
        else ++certified_candidates;
    }
    std::string classification = "EVENT_FORWARD_BOUND_UNRESOLVED";
    if (containment_rejections > 0U)
        classification = "EVENT_FORWARD_BOUND_CONTAINMENT_REJECTED";
    else if (no_certified_sides > 0U)
        classification = "EVENT_FORWARD_BOUND_NO_CERTIFIED_SIDE";
    else if (suffix_rejections > 0U)
        classification = "EVENT_FORWARD_BOUND_SUFFIX_REJECTED";
    else if (certified_candidates == 7U)
        classification = "EVENT_FORWARD_BOUND_CANDIDATE";

    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const std::string route = !parent_exact
        ? "V3_EVENT_FORWARD_BOUND_PARENT_REJECTED"
        : (!r24_exact ? "V3_EVENT_FORWARD_BOUND_R24_REJECTED"
            : (!audit_exact ? "V3_EVENT_FORWARD_BOUND_AUDIT_REJECTED"
                            : "V3_EVENT_FORWARD_BOUND_IDENTIFIED"));
    const bool exact = parent_exact && r24_exact
        && audit_exact && lifecycle_exact;
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r25-v3-event-forward-bound.v1|"
        << AL_R20_V3_ZERO_BOUND_FLUTTER_SHA256 << ':' << route << ':'
        << classification << '|';
    for (const ALR20ForwardBoundAudit& audit : audits)
        semantic << audit.root << ';';
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r25_v3_event_forward_bound.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"classification\":\"" << classification << "\""
        << ",\"parent_zero_bound_flutter_sha256\":\""
        << AL_R20_V3_ZERO_BOUND_FLUTTER_SHA256 << "\""
        << ",\"reproduced_zero_bound_flutter_sha256\":\""
        << flutter_sha << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"r24_exact\":" << (r24_exact ? "true" : "false")
        << ",\"audit_exact\":" << (audit_exact ? "true" : "false")
        << ",\"lifecycle_exact\":" << (lifecycle_exact ? "true" : "false")
        << ",\"containment_rejections\":" << containment_rejections
        << ",\"no_certified_sides\":" << no_certified_sides
        << ",\"suffix_rejections\":" << suffix_rejections
        << ",\"certified_candidates\":" << certified_candidates
        << ",\"audits\":[";
    for (std::size_t index = 0U; index < audits.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20ForwardBoundAudit& audit = audits[index];
        report << "{\"iteration\":" << audit.iteration
            << ",\"predicted_scalar\":" << audit.predicted_scalar
            << ",\"contained\":" << (audit.contained ? "true" : "false")
            << ",\"certified_side_found\":"
            << (audit.certified_side_found ? "true" : "false")
            << ",\"certified_suffix_valid\":"
            << (audit.certified_suffix_valid ? "true" : "false")
            << ",\"contributing_entries\":"
            << audit.contributing_entries
            << ",\"first_certified_power\":"
            << audit.first_certified_power
            << ",\"scale\":\"" << al_binary128_text(audit.scale)
            << "\",\"bound\":\"" << al_binary128_text(audit.bound)
            << "\",\"maximum_discrepancy\":\""
            << al_binary128_text(audit.maximum_discrepancy)
            << "\",\"containment_margin\":\""
            << al_binary128_text(audit.containment_margin)
            << "\",\"first_certified_lower\":\""
            << al_binary128_text(audit.first_certified_lower)
            << "\",\"root\":\"" << audit.root << "\"}";
    }
    report << "]"
        << ",\"solver_executions\":" << solver_executions
        << ",\"inherited_ladder_points\":455,\"new_alpha_points\":0"
        << ",\"generated_solver_trials\":0,\"state_updates\":0"
        << ",\"cap_changes\":0,\"timing_admitted\":false"
        << ",\"solver_authority\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
