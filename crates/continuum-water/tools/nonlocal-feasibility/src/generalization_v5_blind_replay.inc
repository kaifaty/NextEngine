namespace {

constexpr const char* AL_R20_R58_SEMANTIC_SHA256 =
    "966c72648900e7b551091b4a7025312cbce7b4f71b8e3b7f48a5126c90ddbfad";
constexpr std::array<const char*, 6U> AL_R20_R59_PROBLEM_SHA256{{
    "80b74858b33f3c4e5e0e1320e87ec3bdff33b75f58f5fbb39d634cea1043f158",
    "6801a64e730fa55a5e2446173cee1470d7a0779681b165a44424084b7f935e31",
    "e61adb0843c71c996ef97e7d05b278d8c4be5aaaa35cd0950dc2cfbec985a9e0",
    "5f9c2b59d28c5f4de6f30a0341e76edf01bb3462fe574b55f687b5911f0cd54c",
    "18e402054adc757be06f70640a923fb05f63c735395b158032402379c5db66b3",
    "2264121d0a268db4e26eda4b6cb8e94304ce2ac201ce22b1aed9a48e7d03eaf1",
}};

bool al_r20_r59_centered_trace_exact(
    const ALR20R50Trace& trace,
    std::size_t& worked,
    std::size_t& dots,
    std::size_t& input_pairs) {
    std::size_t local_worked = 0U;
    bool exact = trace.replacements <= AL_R20_SEMISMOOTH_ITERATION_CAP
        && trace.replacements + trace.certificate_rejections
            + trace.dimension_rejections + trace.cap_hits
            == trace.entries.size();
    for (const ALR20R50Entry& entry : trace.entries) {
        if (!entry.worked) {
            exact = exact && (entry.dimension != AL_R20_R50_DIMENSION
                || trace.cap_hits > 0U);
            continue;
        }
        ++local_worked;
        exact = exact && entry.dimension == AL_R20_R50_DIMENSION
            && entry.certificate.exact && entry.certificate.no_underflow
            && entry.dots == AL_R20_R50_DOTS_PER_CERTIFICATE
            && entry.input_pairs == AL_R20_R50_PAIRS_PER_CERTIFICATE;
        dots += entry.dots;
        input_pairs += entry.input_pairs;
    }
    exact = exact && local_worked
        == trace.replacements + trace.certificate_rejections;
    worked += local_worked;
    return exact;
}

bool al_r20_r59_slope_trace_exact(
    const ALR20R55SlopeTrace& trace,
    const ALR20R35CaseAudit& audit) {
    std::size_t attempted = 0U;
    std::size_t inverse_columns = 0U;
    bool exact = trace.inverse_audits <= 1U
        && trace.cap_hits + trace.inverse_audits == trace.calls
        && trace.matrix_roots.size() == trace.calls
        && trace.rhs_roots.size() == trace.calls
        && trace.solution_roots.size() == trace.calls
        && trace.result_roots.size() == trace.calls
        && trace.audit_roots.size() == trace.inverse_audits;
    for (const ALR20SemismoothStep& step : audit.diagnostic.steps)
        if (step.slope_refinement_attempted) {
            ++attempted;
            exact = exact && step.slope_refinement.exact;
            inverse_columns += step.slope_refinement.inverse_columns;
        }
    return exact && attempted == trace.calls
        && inverse_columns == trace.inverse_columns;
}

} // namespace

SplitBoundaryReport run_al_generalization_v5_blind_replay_controls() {
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
    std::vector<ALR20ManifestCase> cases = al_r20_v5_manifest_cases();
    bool parent_exact = controls_exact && cases.size() == 6U
        && al_r20_v5_manifest_root(cases) == AL_R20_V5_MANIFEST_SHA256;
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    std::vector<ALR20ProblemData> problems;
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        ALR20ProblemData problem = al_r20_materialize_problem(
            cases[index], &sparse_work, &static_work, &adjacency_work);
        parent_exact = parent_exact && problem.exact
            && problem.source_positive == 0U
            && index < AL_R20_R59_PROBLEM_SHA256.size()
            && problem.problem_root == AL_R20_R59_PROBLEM_SHA256[index];
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

    std::size_t total_centered_calls = 0U;
    std::size_t total_centered_worked = 0U;
    std::size_t total_centered_replacements = 0U;
    std::size_t total_centered_rejections = 0U;
    std::size_t total_dimension_rejections = 0U;
    std::size_t total_centered_cap_hits = 0U;
    std::size_t total_dots = 0U;
    std::size_t total_input_pairs = 0U;
    std::size_t total_slope_calls = 0U;
    std::size_t total_slope_audits = 0U;
    std::size_t total_slope_cap_hits = 0U;
    std::size_t total_inverse_columns = 0U;
    bool policy_exact = centered_traces.size() == cases.size()
        && slope_traces.size() == cases.size();
    std::vector<bool> centered_accounting;
    std::vector<bool> slope_accounting;
    for (std::size_t index = 0U; index < centered_traces.size(); ++index) {
        const ALR20R50Trace& centered = centered_traces[index];
        const ALR20R55SlopeTrace& slope = slope_traces[index];
        const bool centered_exact = al_r20_r59_centered_trace_exact(centered,
            total_centered_worked, total_dots, total_input_pairs);
        const bool slope_exact =
            al_r20_r59_slope_trace_exact(slope, diagnostics[index]);
        centered_accounting.push_back(centered_exact);
        slope_accounting.push_back(slope_exact);
        policy_exact = policy_exact && centered_exact && slope_exact;
        total_centered_calls += centered.entries.size();
        total_centered_replacements += centered.replacements;
        total_centered_rejections += centered.certificate_rejections;
        total_dimension_rejections += centered.dimension_rejections;
        total_centered_cap_hits += centered.cap_hits;
        total_slope_calls += slope.calls;
        total_slope_audits += slope.inverse_audits;
        total_slope_cap_hits += slope.cap_hits;
        total_inverse_columns += slope.inverse_columns;
    }
    const bool work_exact = total_dots
            == total_centered_worked * AL_R20_R50_DOTS_PER_CERTIFICATE
        && total_input_pairs
            == total_centered_worked * AL_R20_R50_PAIRS_PER_CERTIFICATE
        && total_centered_calls
            <= cases.size() * AL_R20_SEMISMOOTH_ITERATION_CAP
        && total_slope_audits <= cases.size()
        && total_inverse_columns <= 720U;
    std::size_t certified_cases = 0U;
    for (const ALR20R35CaseAudit& audit : diagnostics)
        if (audit.diagnostic.certified) ++certified_cases;
    const bool all_certified = certified_cases == cases.size();
    const bool apparatus_exact = observation_exact && policy_exact
        && work_exact && lifecycle_exact;
    const bool exact = apparatus_exact;
    const std::string route = !parent_exact || !observation_exact
        ? "BLIND_V5_PARENT_REJECTED"
        : (!lifecycle_exact
            ? "BLIND_V5_APPARATUS_REJECTED"
            : (!policy_exact || !work_exact
                ? "BLIND_V5_POLICY_ACCOUNTING_REJECTED"
                : (all_certified
                    ? "BLIND_V5_6_OF_6_CANDIDATE"
                    : "BLIND_V5_BOUNDARY_IDENTIFIED")));

    std::ostringstream case_material;
    std::ostringstream case_report;
    case_report << '[';
    for (std::size_t index = 0U; index < diagnostics.size(); ++index) {
        if (index != 0U) case_report << ',';
        const ALR20R35CaseAudit& audit = diagnostics[index];
        const ALR20R50Trace& centered = centered_traces[index];
        const ALR20R55SlopeTrace& slope = slope_traces[index];
        std::ostringstream centered_material;
        std::size_t case_dots = 0U;
        std::size_t case_pairs = 0U;
        for (const ALR20R50Entry& entry : centered.entries) {
            centered_material << entry.dimension << ':' << entry.worked << ':'
                << entry.material_root << ':' << entry.certificate.root << ':'
                << entry.replaced << ';';
            case_dots += entry.dots;
            case_pairs += entry.input_pairs;
        }
        const std::string centered_root = sha256_hex(centered_material.str());
        const std::string slope_root = al_r20_r55_trace_root(slope);
        const ALR20SemismoothStep* final_step = audit.diagnostic.steps.empty()
            ? nullptr : &audit.diagnostic.steps.back();
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
            << ",\"final_solve_failure\":\""
            << (final_step == nullptr ? "ABSENT" : final_step->solve.failure)
            << "\",\"centered_calls\":" << centered.entries.size()
            << ",\"centered_accounting\":"
            << (centered_accounting[index] ? "true" : "false")
            << ",\"centered_replacements\":" << centered.replacements
            << ",\"centered_rejections\":"
            << centered.certificate_rejections
            << ",\"dimension_rejections\":"
            << centered.dimension_rejections
            << ",\"centered_cap_hits\":" << centered.cap_hits
            << ",\"centered_trace_root\":\"" << centered_root
            << "\",\"slope_calls\":" << slope.calls
            << ",\"slope_accounting\":"
            << (slope_accounting[index] ? "true" : "false")
            << ",\"slope_audits\":" << slope.inverse_audits
            << ",\"slope_cap_hits\":" << slope.cap_hits
            << ",\"slope_trace_root\":\"" << slope_root
            << "\",\"dots\":" << case_dots
            << ",\"input_pairs\":" << case_pairs << '}';
    }
    case_report << ']';
    const std::string case_root = sha256_hex(case_material.str());
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r59-blind-v5.v1|"
        << AL_R20_R58_SEMANTIC_SHA256 << ':' << route << ':' << case_root
        << ':' << certified_cases << ':' << total_centered_calls << ':'
        << total_centered_worked << ':' << total_centered_replacements << ':'
        << total_centered_rejections << ':' << total_dimension_rejections << ':'
        << total_centered_cap_hits << ':' << total_slope_calls << ':'
        << total_slope_audits << ':' << total_slope_cap_hits << ':'
        << total_inverse_columns << ':' << total_dots << ':' << total_input_pairs;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r59_blind_v5.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r58_sha256\":\"" << AL_R20_R58_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"observation_exact\":" << (observation_exact ? "true" : "false")
        << ",\"policy_exact\":" << (policy_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":" << (lifecycle_exact ? "true" : "false")
        << ",\"certified_cases\":" << certified_cases
        << ",\"total_centered_calls\":" << total_centered_calls
        << ",\"total_centered_worked\":" << total_centered_worked
        << ",\"total_centered_replacements\":" << total_centered_replacements
        << ",\"total_centered_rejections\":" << total_centered_rejections
        << ",\"total_dimension_rejections\":" << total_dimension_rejections
        << ",\"total_centered_cap_hits\":" << total_centered_cap_hits
        << ",\"total_slope_calls\":" << total_slope_calls
        << ",\"total_slope_audits\":" << total_slope_audits
        << ",\"total_slope_cap_hits\":" << total_slope_cap_hits
        << ",\"total_inverse_columns\":" << total_inverse_columns
        << ",\"total_dots\":" << total_dots
        << ",\"total_input_pairs\":" << total_input_pairs
        << ",\"case_root\":\"" << case_root
        << "\",\"cases\":" << case_report.str()
        << ",\"solver_executions\":" << solver_executions
        << ",\"additional_factorizations\":0,\"new_mechanisms\":0"
        << ",\"source_changes\":0,\"retry_count\":0"
        << ",\"parameter_changes\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
