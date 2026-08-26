namespace {

constexpr const char* AL_R20_R50_SEMANTIC_SHA256 =
    "190ac441077ac0499e5e9e4a9c419ef0d604bf6aa06017b49d3051ff7e12d86e";
constexpr const char* AL_R20_R50_TORSION_CASE_SHA256 =
    "1afe1d2a65f3f5926a8d078cd260f4f0db8b58f3bdf1ff1d82f12529e6afe806";

bool al_r20_r51_trace_exact(
    const ALR20R50Trace& trace,
    std::size_t& dots,
    std::size_t& input_pairs) {
    bool exact = trace.dimension_rejections == 0U && trace.cap_hits == 0U
        && trace.replacements <= AL_R20_SEMISMOOTH_ITERATION_CAP;
    for (const ALR20R50Entry& entry : trace.entries) {
        if (!entry.worked) {
            exact = false;
            continue;
        }
        exact = exact && entry.certificate.exact
            && entry.certificate.no_underflow
            && entry.dots == AL_R20_R50_DOTS_PER_CERTIFICATE
            && entry.input_pairs == AL_R20_R50_PAIRS_PER_CERTIFICATE;
        dots += entry.dots;
        input_pairs += entry.input_pairs;
    }
    return exact;
}

} // namespace

SplitBoundaryReport run_al_generalization_v4_generic_corpus_controls() {
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
    std::vector<ALR20R50Trace> traces;
    std::size_t solver_executions = 0U;
    const bool hook_slot_clear = al_r20_verified_solution_refiner == nullptr
        && al_r20_r50_trace == nullptr;
    if (parent_exact && hook_slot_clear) {
        for (const ALR20ProblemData& problem : problems) {
            ALR20R50Trace trace;
            al_r20_r50_trace = &trace;
            al_r20_verified_solution_refiner = &al_r20_r50_refiner;
            diagnostics.push_back(al_r20_r35_case(problem));
            al_r20_verified_solution_refiner = nullptr;
            al_r20_r50_trace = nullptr;
            traces.push_back(std::move(trace));
            ++solver_executions;
        }
    }
    const bool hook_slot_released = al_r20_verified_solution_refiner == nullptr
        && al_r20_r50_trace == nullptr;
    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    bool observation_exact = parent_exact && hook_slot_clear
        && hook_slot_released && diagnostics.size() == cases.size()
        && traces.size() == cases.size()
        && solver_executions == cases.size();
    for (const ALR20R35CaseAudit& audit : diagnostics)
        observation_exact = observation_exact
            && al_r20_r35_case_observation_exact(audit);
    std::size_t total_dots = 0U;
    std::size_t total_input_pairs = 0U;
    bool policy_exact = traces.size() == cases.size();
    std::size_t total_calls = 0U;
    std::size_t total_replacements = 0U;
    std::size_t total_certificate_rejections = 0U;
    for (const ALR20R50Trace& trace : traces) {
        policy_exact = policy_exact
            && al_r20_r51_trace_exact(
                trace, total_dots, total_input_pairs);
        total_calls += trace.entries.size();
        total_replacements += trace.replacements;
        total_certificate_rejections += trace.certificate_rejections;
    }
    const bool work_exact = total_dots
            == total_calls * AL_R20_R50_DOTS_PER_CERTIFICATE
        && total_input_pairs
            == total_calls * AL_R20_R50_PAIRS_PER_CERTIFICATE
        && total_dots <= cases.size() * AL_R20_SEMISMOOTH_ITERATION_CAP
            * AL_R20_R50_DOTS_PER_CERTIFICATE
        && total_input_pairs
            <= cases.size() * AL_R20_SEMISMOOTH_ITERATION_CAP
                * AL_R20_R50_PAIRS_PER_CERTIFICATE;
    const bool r50_exact = diagnostics.size() == 5U && traces.size() == 5U
        && diagnostics[0U].root == AL_R20_R50_TORSION_CASE_SHA256
        && diagnostics[0U].diagnostic.certified
        && diagnostics[0U].diagnostic.accepted_iterations == 11U
        && diagnostics[0U].diagnostic.total_principal_solves == 887U
        && diagnostics[0U].diagnostic.total_transitions == 887U
        && traces[0U].entries.size() == 5U
        && traces[0U].replacements == 5U;
    std::size_t certified_cases = 0U;
    std::size_t direction_boundaries = 0U;
    std::size_t other_boundaries = 0U;
    for (std::size_t index = 0U; index < diagnostics.size(); ++index) {
        const ALR20R35CaseAudit& audit = diagnostics[index];
        if (audit.diagnostic.certified) {
            ++certified_cases;
        } else if (index == 2U && audit.route == "DIRECTION_REJECTED"
            && audit.diagnostic.direction_rejected) {
            ++direction_boundaries;
        } else {
            ++other_boundaries;
        }
    }
    const bool counterflow_only = certified_cases == 4U
        && direction_boundaries == 1U && other_boundaries == 0U;
    const bool all_certified = certified_cases == cases.size();
    const bool exact = observation_exact && r50_exact && policy_exact
        && work_exact && lifecycle_exact;
    const std::string route = !parent_exact || !observation_exact || !r50_exact
        ? "GENERIC_CORPUS_PARENT_REJECTED"
        : (!policy_exact
            ? "GENERIC_CORPUS_POLICY_REJECTED"
            : (!work_exact
                ? "GENERIC_CORPUS_WORK_REJECTED"
                : (all_certified
                    ? "GENERIC_CORPUS_5_OF_5_CANDIDATE"
                    : (counterflow_only
                        ? "GENERIC_CORPUS_COUNTERFLOW_ONLY_BOUNDARY"
                        : "GENERIC_CORPUS_MULTI_BOUNDARY"))));
    std::ostringstream case_material;
    std::ostringstream case_report;
    case_report << '[';
    for (std::size_t index = 0U; index < diagnostics.size(); ++index) {
        if (index != 0U) case_report << ',';
        const ALR20R35CaseAudit& audit = diagnostics[index];
        const ALR20R50Trace& trace = traces[index];
        const ALR20SemismoothStep* final_step =
            audit.diagnostic.steps.empty()
            ? nullptr : &audit.diagnostic.steps.back();
        const ALR20NNQPSolve* final_solve = final_step == nullptr
            ? nullptr : &final_step->solve;
        std::ostringstream trace_material;
        std::size_t case_dots = 0U;
        std::size_t case_pairs = 0U;
        for (const ALR20R50Entry& entry : trace.entries) {
            trace_material << entry.material_root << ':'
                << entry.certificate.root << ':' << entry.replaced << ';';
            case_dots += entry.dots;
            case_pairs += entry.input_pairs;
        }
        const std::string trace_root = sha256_hex(trace_material.str());
        case_material << problems[index].problem_root << ':' << audit.root
            << ':' << trace_root << ';';
        case_report << "{\"index\":" << index
            << ",\"name\":\"" << cases[index].fixture.name
            << "\",\"problem_root\":\"" << problems[index].problem_root
            << "\",\"case_root\":\"" << audit.root
            << "\",\"route\":\"" << audit.route
            << "\",\"certified\":"
            << (audit.diagnostic.certified ? "true" : "false")
            << ",\"direction_rejected\":"
            << (audit.diagnostic.direction_rejected ? "true" : "false")
            << ",\"active_set_rejected\":"
            << (audit.diagnostic.active_set_rejected ? "true" : "false")
            << ",\"accepted_iterations\":"
            << audit.diagnostic.accepted_iterations
            << ",\"principal_solves\":"
            << audit.diagnostic.total_principal_solves
            << ",\"transitions\":"
            << audit.diagnostic.total_transitions
            << ",\"final_solve_failure\":\""
            << (final_solve == nullptr ? "NONE" : final_solve->failure)
            << "\",\"generic_calls\":" << trace.entries.size()
            << ",\"generic_replacements\":" << trace.replacements
            << ",\"generic_rejections\":"
            << trace.certificate_rejections
            << ",\"trace_root\":\"" << trace_root
            << "\",\"dots\":" << case_dots
            << ",\"input_pairs\":" << case_pairs << '}';
    }
    case_report << ']';
    const std::string case_root = sha256_hex(case_material.str());
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r51-generic-corpus.v1|"
        << AL_R20_R50_SEMANTIC_SHA256 << ':' << route << ':' << case_root
        << ':' << total_calls << ':' << total_replacements << ':'
        << total_dots << ':' << total_input_pairs;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r51_generic_corpus.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r50_sha256\":\""
        << AL_R20_R50_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"observation_exact\":"
        << (observation_exact ? "true" : "false")
        << ",\"r50_exact\":" << (r50_exact ? "true" : "false")
        << ",\"policy_exact\":" << (policy_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"certified_cases\":" << certified_cases
        << ",\"direction_boundaries\":" << direction_boundaries
        << ",\"other_boundaries\":" << other_boundaries
        << ",\"total_generic_calls\":" << total_calls
        << ",\"total_replacements\":" << total_replacements
        << ",\"total_certificate_rejections\":"
        << total_certificate_rejections
        << ",\"total_dots\":" << total_dots
        << ",\"total_input_pairs\":" << total_input_pairs
        << ",\"case_root\":\"" << case_root
        << "\",\"cases\":" << case_report.str()
        << ",\"solver_executions\":" << solver_executions
        << ",\"additional_factorizations\":0"
        << ",\"additional_inverse_columns\":0"
        << ",\"counterflow_policy_changes\":0,\"new_trials\":0"
        << ",\"state_updates\":0,\"parameter_changes\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
