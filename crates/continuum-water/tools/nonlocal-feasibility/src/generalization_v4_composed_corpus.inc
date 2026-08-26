namespace {

constexpr const char* AL_R20_R55_SEMANTIC_SHA256 =
    "9684d2adddeab991c9c82977b68d8f634dee605f62cd9ca11324ee3e37f0fc6e";
constexpr const char* AL_R20_R55_COUNTERFLOW_CASE_SHA256 =
    "5e1807211eff265bc4bcb3db1902eaa7ed1a97f6d46d942b06d205ac1e3b0195";
constexpr std::array<const char*, 5U> AL_R20_R56_CASE_SHA256{{
    "1afe1d2a65f3f5926a8d078cd260f4f0db8b58f3bdf1ff1d82f12529e6afe806",
    "204b49d528ed4a1da64d7088cff301af3101d24972b8737993935670f18ee0e3",
    AL_R20_R55_COUNTERFLOW_CASE_SHA256,
    "11d800fdc76526fee7c29ba19be104516cfa09048c101988337efb08a3d80f05",
    "98b9eafd98368817aebed86ec67beeb444dd24dc797467bfaffdb93125909fe5",
}};

bool al_r20_r56_slope_trace_exact(
    const ALR20R55SlopeTrace& trace,
    bool expected) {
    if (!expected)
        return trace.calls == 0U && trace.inverse_audits == 0U
            && trace.cap_hits == 0U && trace.inverse_columns == 0U
            && trace.matrix_roots.empty() && trace.rhs_roots.empty()
            && trace.solution_roots.empty() && trace.audit_roots.empty()
            && trace.result_roots.empty();
    return trace.calls == 1U && trace.inverse_audits == 1U
        && trace.cap_hits == 0U && trace.inverse_columns == 66U
        && trace.audit_roots.size() == 1U
        && trace.audit_roots.front() == AL_R20_R53_LEGACY_AUDIT_SHA256
        && trace.matrix_roots.size() == 1U
        && trace.matrix_roots.front() == AL_R20_R53_MATRIX_SHA256
        && trace.rhs_roots.size() == 1U
        && trace.rhs_roots.front() == AL_R20_R53_RHS_SHA256
        && trace.solution_roots.size() == 1U
        && trace.solution_roots.front() == AL_R20_R53_SOLUTION_SHA256
        && trace.result_roots.size() == 1U;
}

} // namespace

SplitBoundaryReport run_al_generalization_v4_composed_corpus_controls() {
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
        && al_r20_r21_event_controls()
        && al_r20_r37_dyadic_controls()
        && al_r20_r38_dot2_controls();
    std::vector<ALR20ManifestCase> cases = al_r20_v4_manifest_cases();
    bool parent_exact = controls_exact && cases.size() == 5U
        && al_r20_v4_manifest_root(cases) == AL_R20_V4_MANIFEST_SHA256;
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    std::vector<ALR20ProblemData> problems;
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        ALR20ProblemData problem = al_r20_materialize_problem(
            cases[index], &sparse_work, &static_work, &adjacency_work);
        parent_exact = parent_exact && problem.exact
            && problem.source_positive == 0U
            && index < AL_R20_V4_PROBLEM_SHA256.size()
            && problem.problem_root == AL_R20_V4_PROBLEM_SHA256[index];
        problems.push_back(std::move(problem));
    }

    std::vector<ALR20R35CaseAudit> diagnostics;
    std::vector<ALR20R50Trace> centered_traces;
    std::vector<ALR20R55SlopeTrace> slope_traces;
    std::size_t solver_executions = 0U;
    const bool hook_slot_clear = al_r20_verified_solution_refiner == nullptr
        && al_r20_r50_trace == nullptr
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr
        && al_r20_verified_inverse_capture == nullptr
        && al_r20_direct_solution_capture == nullptr;
    if (parent_exact && hook_slot_clear) {
        for (const ALR20ProblemData& problem : problems) {
            ALR20R50Trace centered;
            ALR20R55SlopeTrace slope;
            al_r20_r50_trace = &centered;
            al_r20_verified_solution_refiner = &al_r20_r50_refiner;
            al_r20_r55_slope_trace = &slope;
            al_r20_slope_error_refiner = &al_r20_r55_slope_refiner;
            diagnostics.push_back(al_r20_r35_case(problem));
            al_r20_slope_error_refiner = nullptr;
            al_r20_r55_slope_trace = nullptr;
            al_r20_verified_solution_refiner = nullptr;
            al_r20_r50_trace = nullptr;
            centered_traces.push_back(std::move(centered));
            slope_traces.push_back(std::move(slope));
            ++solver_executions;
        }
    }
    const bool hook_slot_released = al_r20_verified_solution_refiner == nullptr
        && al_r20_r50_trace == nullptr
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr
        && al_r20_verified_inverse_capture == nullptr
        && al_r20_direct_solution_capture == nullptr;
    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    bool observation_exact = parent_exact && hook_slot_clear
        && hook_slot_released && diagnostics.size() == cases.size()
        && centered_traces.size() == cases.size()
        && slope_traces.size() == cases.size()
        && solver_executions == cases.size();
    for (const ALR20R35CaseAudit& audit : diagnostics)
        observation_exact = observation_exact
            && al_r20_r35_case_observation_exact(audit);

    std::size_t total_dots = 0U;
    std::size_t total_input_pairs = 0U;
    std::size_t total_centered_calls = 0U;
    std::size_t total_centered_replacements = 0U;
    std::size_t total_slope_calls = 0U;
    std::size_t total_slope_audits = 0U;
    std::size_t total_inverse_columns = 0U;
    bool policy_exact = centered_traces.size() == cases.size()
        && slope_traces.size() == cases.size();
    bool trigger_exact = policy_exact;
    for (std::size_t index = 0U; index < centered_traces.size(); ++index) {
        const ALR20R50Trace& centered = centered_traces[index];
        const ALR20R55SlopeTrace& slope = slope_traces[index];
        policy_exact = policy_exact
            && al_r20_r51_trace_exact(
                centered, total_dots, total_input_pairs)
            && al_r20_r56_slope_trace_exact(slope, index == 2U);
        const std::size_t expected_centered = index == 0U ? 5U : 0U;
        trigger_exact = trigger_exact
            && centered.entries.size() == expected_centered
            && centered.replacements == expected_centered
            && centered.certificate_rejections == 0U
            && centered.dimension_rejections == 0U
            && centered.cap_hits == 0U
            && slope.calls == (index == 2U ? 1U : 0U)
            && slope.inverse_audits == (index == 2U ? 1U : 0U);
        total_centered_calls += centered.entries.size();
        total_centered_replacements += centered.replacements;
        total_slope_calls += slope.calls;
        total_slope_audits += slope.inverse_audits;
        total_inverse_columns += slope.inverse_columns;
    }
    const bool work_exact = total_centered_calls == 5U
        && total_centered_replacements == 5U
        && total_dots == 27625U && total_input_pairs == 1802450U
        && total_slope_calls == 1U && total_slope_audits == 1U
        && total_inverse_columns == 66U;
    std::size_t certified_cases = 0U;
    bool roots_exact = diagnostics.size() == AL_R20_R56_CASE_SHA256.size();
    for (std::size_t index = 0U; index < diagnostics.size(); ++index) {
        const ALR20R35CaseAudit& audit = diagnostics[index];
        if (audit.diagnostic.certified) ++certified_cases;
        roots_exact = roots_exact && audit.root == AL_R20_R56_CASE_SHA256[index]
            && audit.route == "ORDINARY_CERTIFIED";
    }
    const bool all_certified = certified_cases == cases.size();
    const bool apparatus_exact = observation_exact && policy_exact
        && trigger_exact && work_exact && roots_exact && lifecycle_exact;
    const bool exact = apparatus_exact;
    const std::string route = !parent_exact || !observation_exact
        ? "COMPOSED_V4_PARENT_REJECTED"
        : (!policy_exact
            ? "COMPOSED_V4_POLICY_REJECTED"
            : (!work_exact
                ? "COMPOSED_V4_WORK_REJECTED"
                : (!trigger_exact || !roots_exact
                    ? "COMPOSED_V4_CROSS_TRIGGERED"
                    : (!all_certified
                        ? "COMPOSED_V4_BOUNDARY"
                        : "COMPOSED_V4_5_OF_5_CANDIDATE"))));

    std::ostringstream case_material;
    std::ostringstream case_report;
    case_report << '[';
    for (std::size_t index = 0U; index < diagnostics.size(); ++index) {
        if (index != 0U) case_report << ',';
        const ALR20R35CaseAudit& audit = diagnostics[index];
        const ALR20R50Trace& centered = centered_traces[index];
        const ALR20R55SlopeTrace& slope = slope_traces[index];
        const std::string slope_root = al_r20_r55_trace_root(slope);
        std::ostringstream centered_material;
        std::size_t case_dots = 0U;
        std::size_t case_pairs = 0U;
        for (const ALR20R50Entry& entry : centered.entries) {
            centered_material << entry.material_root << ':'
                << entry.certificate.root << ':' << entry.replaced << ';';
            case_dots += entry.dots;
            case_pairs += entry.input_pairs;
        }
        const std::string centered_root = sha256_hex(centered_material.str());
        case_material << problems[index].problem_root << ':' << audit.root
            << ':' << centered_root << ':' << slope_root << ';';
        case_report << "{\"index\":" << index
            << ",\"name\":\"" << cases[index].fixture.name
            << "\",\"problem_root\":\"" << problems[index].problem_root
            << "\",\"case_root\":\"" << audit.root
            << "\",\"route\":\"" << audit.route
            << "\",\"certified\":"
            << (audit.diagnostic.certified ? "true" : "false")
            << ",\"accepted_iterations\":"
            << audit.diagnostic.accepted_iterations
            << ",\"principal_solves\":"
            << audit.diagnostic.total_principal_solves
            << ",\"transitions\":" << audit.diagnostic.total_transitions
            << ",\"centered_calls\":" << centered.entries.size()
            << ",\"centered_replacements\":" << centered.replacements
            << ",\"centered_trace_root\":\"" << centered_root
            << "\",\"slope_calls\":" << slope.calls
            << ",\"slope_audits\":" << slope.inverse_audits
            << ",\"slope_trace_root\":\"" << slope_root
            << "\",\"dots\":" << case_dots
            << ",\"input_pairs\":" << case_pairs << '}';
    }
    case_report << ']';
    const std::string case_root = sha256_hex(case_material.str());
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r56-composed-v4.v1|"
        << AL_R20_R55_SEMANTIC_SHA256 << ':' << route << ':' << case_root
        << ':' << total_centered_calls << ':' << total_centered_replacements
        << ':' << total_slope_calls << ':' << total_slope_audits << ':'
        << total_dots << ':' << total_input_pairs << ':'
        << total_inverse_columns;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r56_composed_v4.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r55_sha256\":\"" << AL_R20_R55_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"observation_exact\":" << (observation_exact ? "true" : "false")
        << ",\"policy_exact\":" << (policy_exact ? "true" : "false")
        << ",\"trigger_exact\":" << (trigger_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"roots_exact\":" << (roots_exact ? "true" : "false")
        << ",\"lifecycle_exact\":" << (lifecycle_exact ? "true" : "false")
        << ",\"certified_cases\":" << certified_cases
        << ",\"total_centered_calls\":" << total_centered_calls
        << ",\"total_centered_replacements\":" << total_centered_replacements
        << ",\"total_slope_calls\":" << total_slope_calls
        << ",\"total_slope_audits\":" << total_slope_audits
        << ",\"total_inverse_columns\":" << total_inverse_columns
        << ",\"total_dots\":" << total_dots
        << ",\"total_input_pairs\":" << total_input_pairs
        << ",\"case_root\":\"" << case_root
        << "\",\"cases\":" << case_report.str()
        << ",\"solver_executions\":" << solver_executions
        << ",\"additional_factorizations\":0,\"new_trial_formulae\":0"
        << ",\"parameter_changes\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
