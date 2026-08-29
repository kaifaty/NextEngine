namespace {

constexpr const char* AL_R20_V3_DUAL_REFINEMENT_SHA256 =
    "d224bc8c0f451e1a07aa4cbdf6cc194c85f6341ca3be5a1d74e2a7d97cfe6ab8";
constexpr const char* AL_R20_V3_SHEAR_CANDIDATE_SHA256 =
    "fdd57c9e82da7ca1e22419619e24233670e65a7e9f98ea09dc0166c11491fbb9";
constexpr std::size_t AL_R20_V3_SHEAR_INDEX = 2U;
constexpr std::size_t AL_R20_V3_LATE_FIRST = 25U;

} // namespace

SplitBoundaryReport run_al_generalization_v3_shear_trajectory_controls() {
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
        && !diagnostic.certified
        && !diagnostic.active_set_rejected
        && !diagnostic.direction_rejected
        && !diagnostic.globalization_rejected
        && diagnostic.accepted_iterations == 32U
        && diagnostic.steps.size() == 32U;
    bool useful_contraction = reproduction_exact;
    bool plateau = reproduction_exact;
    std::size_t late_steps = 0U;
    std::size_t late_positive_powers = 0U;
    std::size_t late_mask_changes = 0U;
    std::size_t late_face_changes = 0U;
    std::size_t late_ball_changes = 0U;
    Binary128 late_min_primal_ratio = HUGE_VALQ;
    Binary128 late_max_primal_ratio = static_cast<Binary128>(0.0);
    Binary128 late_min_dual_ratio = HUGE_VALQ;
    Binary128 late_max_dual_ratio = static_cast<Binary128>(0.0);
    ALR20OracleMetrics previous = diagnostic.initial_metrics;
    std::string previous_face;
    for (const ALR20SemismoothStep& step : diagnostic.steps) {
        reproduction_exact = reproduction_exact && step.accepted;
        const Binary128 primal_ratio = previous.primal
                > static_cast<Binary128>(0.0)
            ? step.selected.metrics.primal / previous.primal
            : static_cast<Binary128>(0.0);
        const Binary128 dual_ratio = previous.dual_mapping
                > static_cast<Binary128>(0.0)
            ? step.selected.metrics.dual_mapping / previous.dual_mapping
            : static_cast<Binary128>(0.0);
        if (step.iteration >= AL_R20_V3_LATE_FIRST) {
            ++late_steps;
            if (step.selected.power > 0U) ++late_positive_powers;
            late_mask_changes += step.selected.mask_changes;
            late_ball_changes += step.selected.ball_activity_changed ? 1U : 0U;
            if (!previous_face.empty()
                && step.natural_face_root != previous_face)
                ++late_face_changes;
            const bool ratios_finite = finiteq(primal_ratio) != 0
                && finiteq(dual_ratio) != 0;
            reproduction_exact = reproduction_exact && ratios_finite;
            useful_contraction = useful_contraction && ratios_finite
                && primal_ratio <= static_cast<Binary128>(0.9)
                && dual_ratio <= static_cast<Binary128>(0.9);
            plateau = plateau && ratios_finite
                && primal_ratio >= static_cast<Binary128>(0.99)
                && dual_ratio >= static_cast<Binary128>(0.99);
            late_min_primal_ratio = fminq(
                late_min_primal_ratio, primal_ratio);
            late_max_primal_ratio = fmaxq(
                late_max_primal_ratio, primal_ratio);
            late_min_dual_ratio = fminq(late_min_dual_ratio, dual_ratio);
            late_max_dual_ratio = fmaxq(late_max_dual_ratio, dual_ratio);
        }
        previous = step.selected.metrics;
        previous_face = step.natural_face_root;
    }
    const bool active_set_chatter = late_mask_changes > 0U
        || late_face_changes > 0U;
    const bool globalization_throttled = late_positive_powers >= 6U;
    reproduction_exact = reproduction_exact && late_steps == 8U;
    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    std::string classification = "MIXED_UNRESOLVED";
    if (active_set_chatter)
        classification = "ACTIVE_SET_CHATTER";
    else if (globalization_throttled)
        classification = "GLOBALIZATION_THROTTLED";
    else if (plateau)
        classification = "NUMERICAL_MODEL_PLATEAU";
    else if (useful_contraction)
        classification = "CONTRACTING_CAP_LIMITED";
    const std::string route = !parent_exact
        ? "V3_SHEAR_TRAJECTORY_PARENT_REJECTED"
        : (!reproduction_exact ? "V3_SHEAR_TRAJECTORY_REPRODUCTION_REJECTED"
                              : "V3_SHEAR_TRAJECTORY_IDENTIFIED");
    const bool exact = parent_exact && reproduction_exact && lifecycle_exact;
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r18-v3-shear-trajectory.v1|"
             << AL_R20_V3_DUAL_REFINEMENT_SHA256 << ':' << route << ':'
             << classification << '|';
    previous = diagnostic.initial_metrics;
    for (const ALR20SemismoothStep& step : diagnostic.steps) {
        const Binary128 primal_ratio = previous.primal
                > static_cast<Binary128>(0.0)
            ? step.selected.metrics.primal / previous.primal
            : static_cast<Binary128>(0.0);
        const Binary128 dual_ratio = previous.dual_mapping
                > static_cast<Binary128>(0.0)
            ? step.selected.metrics.dual_mapping / previous.dual_mapping
            : static_cast<Binary128>(0.0);
        semantic << step.root << ':'
                 << al_binary128_hex(primal_ratio) << ':'
                 << al_binary128_hex(dual_ratio) << ';';
        previous = step.selected.metrics;
    }
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r18_v3_shear_trajectory.v1\""
           << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
           << ",\"route\":\"" << route << "\""
           << ",\"classification\":\"" << classification << "\""
           << ",\"parent_dual_refinement_sha256\":\""
           << AL_R20_V3_DUAL_REFINEMENT_SHA256
           << "\",\"profile_exact\":"
           << (profile_exact ? "true" : "false")
           << ",\"controls_exact\":"
           << (controls_exact ? "true" : "false")
           << ",\"parent_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"reproduction_exact\":"
           << (reproduction_exact ? "true" : "false")
           << ",\"lifecycle_exact\":"
           << (lifecycle_exact ? "true" : "false")
           << ",\"case_sha256\":\"" << diagnostic.root
           << "\",\"late_window\":{\"first\":25,\"last\":32"
           << ",\"useful_contraction\":"
           << (useful_contraction ? "true" : "false")
           << ",\"plateau\":" << (plateau ? "true" : "false")
           << ",\"active_set_chatter\":"
           << (active_set_chatter ? "true" : "false")
           << ",\"globalization_throttled\":"
           << (globalization_throttled ? "true" : "false")
           << ",\"positive_powers\":" << late_positive_powers
           << ",\"mask_changes\":" << late_mask_changes
           << ",\"face_changes\":" << late_face_changes
           << ",\"ball_changes\":" << late_ball_changes
           << ",\"minimum_primal_ratio\":\""
           << al_binary128_text(late_min_primal_ratio)
           << "\",\"maximum_primal_ratio\":\""
           << al_binary128_text(late_max_primal_ratio)
           << "\",\"minimum_dual_ratio\":\""
           << al_binary128_text(late_min_dual_ratio)
           << "\",\"maximum_dual_ratio\":\""
           << al_binary128_text(late_max_dual_ratio)
           << "\"},\"steps\":[";
    previous = diagnostic.initial_metrics;
    for (std::size_t index = 0U; index < diagnostic.steps.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20SemismoothStep& step = diagnostic.steps[index];
        const Binary128 primal_ratio = previous.primal
                > static_cast<Binary128>(0.0)
            ? step.selected.metrics.primal / previous.primal
            : static_cast<Binary128>(0.0);
        const Binary128 dual_ratio = previous.dual_mapping
                > static_cast<Binary128>(0.0)
            ? step.selected.metrics.dual_mapping / previous.dual_mapping
            : static_cast<Binary128>(0.0);
        report << "{\"iteration\":" << step.iteration
               << ",\"natural_face\":" << step.natural_face_size
               << ",\"support\":" << step.solve.support_size
               << ",\"principal_solves\":"
               << step.solve.principal_solves
               << ",\"transitions\":" << step.solve.transitions
               << ",\"line_trials\":" << step.trials.size()
               << ",\"power\":" << step.selected.power
               << ",\"alpha\":\""
               << al_binary128_text(step.selected.alpha)
               << "\",\"dual_increase_lower\":\""
               << al_binary128_text(step.selected.dual_increase_lower)
               << "\",\"mask_changes\":"
               << step.selected.mask_changes
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
               << "\",\"primal_ratio\":\""
               << al_binary128_text(primal_ratio)
               << "\",\"dual_mapping_ratio\":\""
               << al_binary128_text(dual_ratio)
               << "\",\"dual_ratio_refinements\":"
               << step.solve.dual_ratio_refinements
               << ",\"ratio_inverse_columns\":"
               << step.solve.ratio_inverse_columns
               << ",\"face_sha256\":\"" << step.natural_face_root
               << "\",\"root\":\"" << step.root << "\"}";
        previous = step.selected.metrics;
    }
    report << "]"
              ",\"solver_executions\":" << solver_executions
           << ",\"accepted_iteration_cap\":32"
              ",\"extended_cap_iterations\":0"
              ",\"parameter_changes\":0,\"timing_admitted\":false"
              ",\"solver_authority\":false"
              ",\"runtime_authority\":false"
              ",\"production_authority\":false"
              ",\"result_sha256\":\""
           << result_sha << "\"}";
    return {exact, report.str()};
}
