namespace {

constexpr const char* AL_R20_V3_LINE_FRONTIER_SHA256 =
    "46f8d84a950e3987609d32858f1c972d60e1cf1d937f1c3fd5b20d1f058ca06f";
constexpr std::size_t AL_R20_BREAKPOINT_SUBDIVISIONS = 64U;

std::string al_r20_r20_mask_root(const ALR20ProjectorState& state) {
    std::ostringstream material;
    material << state.ball_active << '|';
    for (int item : state.mask) material << item << ',';
    return sha256_hex(material.str());
}

struct ALR20BreakpointSample {
    bool exact = false;
    bool stable = false;
    bool armijo_positive = false;
    std::size_t index = 0U;
    Binary128 alpha = static_cast<Binary128>(0.0);
    Binary128 margin = static_cast<Binary128>(0.0);
    std::vector<std::size_t> changed_scalars;
    ALR20NewtonDualEvaluation dual;
    std::string mask_root;
    std::string root;
};

struct ALR20BreakpointBracket {
    bool exact = false;
    bool endpoint_parity = false;
    std::size_t iteration = 0U;
    std::size_t accepted_power = 0U;
    std::size_t mask_transitions = 0U;
    std::size_t armijo_transitions = 0U;
    std::size_t mask_returns = 0U;
    std::size_t stable_negative_samples = 0U;
    std::size_t distinct_changed_scalars = 0U;
    std::size_t first_mask_transition_cell = AL_R20_BREAKPOINT_SUBDIVISIONS;
    std::size_t first_armijo_transition_cell = AL_R20_BREAKPOINT_SUBDIVISIONS;
    bool ball_changed = false;
    std::vector<ALR20BreakpointSample> samples;
    std::string root;
};

ALR20BreakpointBracket al_r20_r20_breakpoint_bracket(
    const ALR20ProblemData& problem,
    const ALR20NewtonDualEvaluation& current,
    const std::vector<Binary128>& lambda,
    const ALR20SemismoothStep& step) {
    ALR20BreakpointBracket result;
    result.iteration = step.iteration;
    result.accepted_power = step.selected.power;
    const bool shape_exact = step.accepted && step.selected.power > 0U
        && step.line_representative.size() == lambda.size()
        && step.next_lambda.size() == lambda.size();
    const ALR20NewtonTrial* accepted = nullptr;
    const ALR20NewtonTrial* rejected_double = nullptr;
    for (const ALR20NewtonTrial& trial : step.trials) {
        if (trial.power == step.selected.power) accepted = &trial;
        if (trial.power + 1U == step.selected.power)
            rejected_double = &trial;
    }
    bool exact = shape_exact && accepted != nullptr
        && rejected_double != nullptr && accepted->accepted
        && !rejected_double->accepted;
    const Binary128 denominator = static_cast<Binary128>(
        AL_R20_BREAKPOINT_SUBDIVISIONS);
    std::vector<bool> changed_seen(current.projector.mask.size(), false);
    std::vector<std::string> observed_masks;
    for (std::size_t sample_index = 0U;
         sample_index <= AL_R20_BREAKPOINT_SUBDIVISIONS; ++sample_index) {
        ALR20BreakpointSample sample;
        sample.index = sample_index;
        const Binary128 fraction = static_cast<Binary128>(sample_index)
            / denominator;
        sample.alpha = step.selected.alpha
            * (static_cast<Binary128>(1.0) + fraction);
        std::vector<Binary128> trial_lambda(lambda.size());
        bool cone_exact = true;
        for (std::size_t row = 0U; row < lambda.size(); ++row) {
            trial_lambda[row] =
                (static_cast<Binary128>(1.0) - sample.alpha) * lambda[row]
                + sample.alpha * step.line_representative[row];
            cone_exact = cone_exact
                && trial_lambda[row] >= static_cast<Binary128>(0.0);
        }
        sample.dual = al_r20_newton_dual_evaluate(problem, trial_lambda);
        ALR20NewtonTrial margin_trial;
        margin_trial.alpha = sample.alpha;
        margin_trial.dual = sample.dual;
        sample.margin = al_r20_r19_armijo_margin(
            current, step, margin_trial);
        sample.armijo_positive = sample.margin
            >= static_cast<Binary128>(0.0);
        sample.stable = sample.dual.projector.ball_active
                == current.projector.ball_active
            && sample.dual.projector.mask == current.projector.mask;
        sample.mask_root = al_r20_r20_mask_root(sample.dual.projector);
        sample.exact = cone_exact && sample.dual.exact
            && finiteq(sample.alpha) != 0 && finiteq(sample.margin) != 0;
        for (std::size_t scalar = 0U;
             scalar < current.projector.mask.size(); ++scalar) {
            if (sample.dual.projector.mask[scalar]
                    != current.projector.mask[scalar]) {
                sample.changed_scalars.push_back(scalar);
                changed_seen[scalar] = true;
            }
        }
        if (sample.dual.projector.ball_active
                != current.projector.ball_active)
            result.ball_changed = true;
        if (sample.stable && !sample.armijo_positive)
            ++result.stable_negative_samples;
        if (!observed_masks.empty()
            && observed_masks.back() != sample.mask_root) {
            ++result.mask_transitions;
            if (result.first_mask_transition_cell
                    == AL_R20_BREAKPOINT_SUBDIVISIONS)
                result.first_mask_transition_cell = sample_index - 1U;
            if (std::find(observed_masks.begin(), observed_masks.end(),
                    sample.mask_root) != observed_masks.end())
                ++result.mask_returns;
        }
        if (!result.samples.empty()
            && result.samples.back().armijo_positive
                != sample.armijo_positive) {
            ++result.armijo_transitions;
            if (result.first_armijo_transition_cell
                    == AL_R20_BREAKPOINT_SUBDIVISIONS)
                result.first_armijo_transition_cell = sample_index - 1U;
        }
        std::ostringstream sample_material;
        sample_material << sample.exact << ':' << sample.index << ':'
            << al_binary128_hex(sample.alpha) << ':'
            << al_binary128_hex(sample.margin) << ':' << sample.stable << ':'
            << sample.armijo_positive << ':' << sample.mask_root << ':'
            << sample.dual.root << '|';
        for (std::size_t scalar : sample.changed_scalars)
            sample_material << scalar << ',';
        sample.root = sha256_hex(sample_material.str());
        exact = exact && sample.exact;
        observed_masks.push_back(sample.mask_root);
        result.samples.push_back(std::move(sample));
    }
    result.distinct_changed_scalars = static_cast<std::size_t>(
        std::count(changed_seen.begin(), changed_seen.end(), true));
    result.endpoint_parity = exact
        && result.samples.front().dual.root == accepted->dual.root
        && result.samples.back().dual.root == rejected_double->dual.root
        && result.samples.front().armijo_positive
        && !result.samples.back().armijo_positive
        && result.samples.front().stable
        && !result.samples.back().stable;
    result.exact = exact && result.endpoint_parity;
    std::ostringstream material;
    material << result.exact << ':' << result.endpoint_parity << ':'
        << result.iteration << ':' << result.accepted_power << ':'
        << result.mask_transitions << ':' << result.armijo_transitions << ':'
        << result.mask_returns << ':' << result.stable_negative_samples << ':'
        << result.distinct_changed_scalars << ':' << result.ball_changed << ':'
        << result.first_mask_transition_cell << ':'
        << result.first_armijo_transition_cell << '|';
    for (const ALR20BreakpointSample& sample : result.samples)
        material << sample.root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v3_breakpoint_geometry_controls() {
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

    std::size_t frontier_coincidences = 0U;
    std::size_t stable_rejections = 0U;
    std::size_t crossing_acceptances = 0U;
    bool margin_parity = reproduction_exact;
    for (std::size_t step_index = AL_R20_V3_LATE_FIRST - 1U;
         step_index < diagnostic.steps.size(); ++step_index) {
        const ALR20SemismoothStep& step = diagnostic.steps[step_index];
        const ALR20NewtonDualEvaluation& current =
            diagnostic.steps[step_index - 1U].selected.dual;
        std::size_t first_stable = AL_R20_NEWTON_LINE_TRIALS;
        std::size_t first_accepted = AL_R20_NEWTON_LINE_TRIALS;
        for (const ALR20NewtonTrial& trial : step.trials) {
            const Binary128 margin =
                al_r20_r19_armijo_margin(current, step, trial);
            margin_parity = margin_parity && trial.exact
                && (margin >= static_cast<Binary128>(0.0)) == trial.accepted;
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
    }
    std::ostringstream parent_semantic;
    parent_semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r19-v3-line-frontier.v1|"
        << AL_R20_V3_SHEAR_TRAJECTORY_SHA256
        << ":V3_LINE_FRONTIER_IDENTIFIED:MASK_CROSSING_FRONTIER|";
    for (std::size_t step_index = AL_R20_V3_LATE_FIRST - 1U;
         step_index < diagnostic.steps.size(); ++step_index) {
        const ALR20SemismoothStep& step = diagnostic.steps[step_index];
        const ALR20NewtonDualEvaluation& current =
            diagnostic.steps[step_index - 1U].selected.dual;
        parent_semantic << step.root << '|';
        for (const ALR20NewtonTrial& trial : step.trials)
            parent_semantic << trial.root << ':'
                << al_binary128_hex(
                    al_r20_r19_armijo_margin(current, step, trial)) << ';';
    }
    const std::string parent_line_sha = sha256_hex(parent_semantic.str());
    const bool r19_exact = reproduction_exact && margin_parity
        && frontier_coincidences == 7U && stable_rejections == 0U
        && crossing_acceptances == 1U
        && parent_line_sha == AL_R20_V3_LINE_FRONTIER_SHA256;

    std::vector<ALR20BreakpointBracket> brackets;
    std::vector<Binary128> lambda(
        problems[AL_R20_V3_SHEAR_INDEX].row_scale.size(),
        static_cast<Binary128>(0.0));
    ALR20NewtonDualEvaluation current = al_r20_newton_dual_evaluate(
        problems[AL_R20_V3_SHEAR_INDEX], lambda);
    bool audit_exact = r19_exact && current.exact;
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
                audit_exact = audit_exact && bracket.exact;
                brackets.push_back(std::move(bracket));
            }
        }
        if (step.accepted) {
            lambda = step.next_lambda;
            current = step.selected.dual;
        }
    }
    audit_exact = audit_exact && brackets.size() == 7U;

    std::size_t stable_negative_samples = 0U;
    std::size_t complex_brackets = 0U;
    std::size_t aligned_brackets = 0U;
    std::size_t simple_offset_brackets = 0U;
    for (const ALR20BreakpointBracket& bracket : brackets) {
        stable_negative_samples += bracket.stable_negative_samples;
        const bool simple = bracket.mask_transitions == 1U
            && bracket.armijo_transitions == 1U
            && bracket.mask_returns == 0U
            && bracket.distinct_changed_scalars == 1U
            && !bracket.ball_changed;
        if (!simple) ++complex_brackets;
        else if (bracket.first_mask_transition_cell
                == bracket.first_armijo_transition_cell)
            ++aligned_brackets;
        else
            ++simple_offset_brackets;
    }
    std::string classification = "BREAKPOINT_GEOMETRY_UNRESOLVED";
    if (stable_negative_samples > 0U)
        classification = "SAME_FACE_ARMIJO_LAYER";
    else if (complex_brackets > 0U)
        classification = "MULTI_EVENT_BREAKPOINT_PATH";
    else if (aligned_brackets == brackets.size() && !brackets.empty())
        classification = "SINGLE_BREAKPOINT_ALIGNED";
    else if (simple_offset_brackets > 0U
             && simple_offset_brackets + aligned_brackets == brackets.size())
        classification = "SIMPLE_BREAKPOINT_OFFSET";

    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const std::string route = !parent_exact
        ? "V3_BREAKPOINT_GEOMETRY_PARENT_REJECTED"
        : (!r19_exact ? "V3_BREAKPOINT_GEOMETRY_R19_REJECTED"
            : (!audit_exact ? "V3_BREAKPOINT_GEOMETRY_AUDIT_REJECTED"
                            : "V3_BREAKPOINT_GEOMETRY_IDENTIFIED"));
    const bool exact = parent_exact && r19_exact
        && audit_exact && lifecycle_exact;
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r20-v3-breakpoint-geometry.v1|"
        << AL_R20_V3_LINE_FRONTIER_SHA256 << ':' << route << ':'
        << classification << '|';
    for (const ALR20BreakpointBracket& bracket : brackets)
        semantic << bracket.root << ';';
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r20_v3_breakpoint_geometry.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"classification\":\"" << classification << "\""
        << ",\"parent_line_frontier_sha256\":\""
        << AL_R20_V3_LINE_FRONTIER_SHA256 << "\""
        << ",\"reproduced_line_frontier_sha256\":\""
        << parent_line_sha << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"r19_exact\":" << (r19_exact ? "true" : "false")
        << ",\"audit_exact\":" << (audit_exact ? "true" : "false")
        << ",\"lifecycle_exact\":" << (lifecycle_exact ? "true" : "false")
        << ",\"frontier_brackets\":" << brackets.size()
        << ",\"subdivisions_per_bracket\":"
        << AL_R20_BREAKPOINT_SUBDIVISIONS
        << ",\"diagnostic_samples\":"
        << brackets.size() * (AL_R20_BREAKPOINT_SUBDIVISIONS + 1U)
        << ",\"stable_negative_samples\":" << stable_negative_samples
        << ",\"complex_brackets\":" << complex_brackets
        << ",\"aligned_brackets\":" << aligned_brackets
        << ",\"simple_offset_brackets\":" << simple_offset_brackets
        << ",\"brackets\":[";
    for (std::size_t bracket_index = 0U;
         bracket_index < brackets.size(); ++bracket_index) {
        if (bracket_index != 0U) report << ',';
        const ALR20BreakpointBracket& bracket = brackets[bracket_index];
        report << "{\"iteration\":" << bracket.iteration
            << ",\"accepted_power\":" << bracket.accepted_power
            << ",\"endpoint_parity\":"
            << (bracket.endpoint_parity ? "true" : "false")
            << ",\"mask_transitions\":" << bracket.mask_transitions
            << ",\"armijo_transitions\":" << bracket.armijo_transitions
            << ",\"mask_returns\":" << bracket.mask_returns
            << ",\"stable_negative_samples\":"
            << bracket.stable_negative_samples
            << ",\"distinct_changed_scalars\":"
            << bracket.distinct_changed_scalars
            << ",\"ball_changed\":"
            << (bracket.ball_changed ? "true" : "false")
            << ",\"first_mask_transition_cell\":"
            << bracket.first_mask_transition_cell
            << ",\"first_armijo_transition_cell\":"
            << bracket.first_armijo_transition_cell
            << ",\"samples\":[";
        for (std::size_t sample_index = 0U;
             sample_index < bracket.samples.size(); ++sample_index) {
            if (sample_index != 0U) report << ',';
            const ALR20BreakpointSample& sample = bracket.samples[sample_index];
            report << "{\"index\":" << sample.index
                << ",\"alpha\":\"" << al_binary128_text(sample.alpha)
                << "\",\"stable\":" << (sample.stable ? "true" : "false")
                << ",\"armijo_positive\":"
                << (sample.armijo_positive ? "true" : "false")
                << ",\"margin\":\"" << al_binary128_text(sample.margin)
                << "\",\"changed_scalars\":[";
            for (std::size_t scalar_index = 0U;
                 scalar_index < sample.changed_scalars.size(); ++scalar_index) {
                if (scalar_index != 0U) report << ',';
                report << sample.changed_scalars[scalar_index];
            }
            report << "],\"mask_sha256\":\"" << sample.mask_root
                << "\",\"dual_sha256\":\"" << sample.dual.root
                << "\",\"root\":\"" << sample.root << "\"}";
        }
        report << "],\"root\":\"" << bracket.root << "\"}";
    }
    report << "]"
        << ",\"solver_executions\":" << solver_executions
        << ",\"new_line_trials\":0,\"state_updates\":0"
        << ",\"cap_changes\":0,\"timing_admitted\":false"
        << ",\"solver_authority\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
