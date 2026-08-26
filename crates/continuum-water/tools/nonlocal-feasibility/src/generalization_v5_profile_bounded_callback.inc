namespace {

constexpr const char* AL_R20_R60_SEMANTIC_SHA256 =
    "c8f1180669c8e4c75a094467fc5816ba45a2ae384726d4b0554a897e61c1ae2a";
constexpr std::size_t AL_R20_R61_MAX_DIMENSION = 120U;
constexpr std::size_t AL_R20_R61_CENTER_DEPTH = 16U;
constexpr std::array<std::size_t, 6U> AL_R20_R61_ROWS{{
    100U, 108U, 96U, 105U, 108U, 120U}};

struct ALR20R61Entry {
    std::size_t dimension = 0U;
    bool worked = false;
    bool replaced = false;
    ALR20R42PracticalCertificate certificate;
    std::string rejection;
    std::string material_root;
    std::string matrix_root;
    std::string inverse_root;
    std::string rhs_root;
    std::string solution_root;
    std::size_t dots = 0U;
    std::size_t input_pairs = 0U;
};

struct ALR20R61Trace {
    std::vector<ALR20R61Entry> entries;
    std::size_t replacements = 0U;
    std::size_t certificate_rejections = 0U;
    std::size_t dimension_rejections = 0U;
    std::size_t cap_hits = 0U;
};

thread_local ALR20R61Trace* al_r20_r61_trace = nullptr;

ALR20VerifiedSolutionRefinement al_r20_r61_refiner(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& inverse,
    const std::vector<Binary128>& rhs,
    const std::vector<Binary128>& solution) {
    ALR20VerifiedSolutionRefinement result;
    ALR20R61Trace* trace = al_r20_r61_trace;
    if (trace == nullptr) return result;
    ALR20R61Entry entry;
    entry.dimension = rhs.size();
    entry.matrix_root = al_r20_q_scalar_root(matrix);
    entry.inverse_root = al_r20_q_scalar_root(inverse);
    entry.rhs_root = al_r20_q_scalar_root(rhs);
    entry.solution_root = al_r20_q_scalar_root(solution);
    entry.material_root = sha256_hex(entry.matrix_root + ':'
        + entry.inverse_root + ':' + entry.rhs_root + ':'
        + entry.solution_root);
    if (entry.dimension == 0U
        || entry.dimension > AL_R20_R61_MAX_DIMENSION) {
        entry.rejection = "DIMENSION";
        ++trace->dimension_rejections;
        result.exact = true;
        result.root = sha256_hex(std::string("r61-dimension-rejected:")
            + entry.material_root + ':' + std::to_string(entry.dimension));
        trace->entries.push_back(std::move(entry));
        return result;
    }
    if (trace->replacements >= AL_R20_SEMISMOOTH_ITERATION_CAP) {
        entry.rejection = "CAP";
        ++trace->cap_hits;
        result.exact = true;
        result.root = sha256_hex(std::string("r61-cap-rejected:")
            + entry.material_root + ':'
            + std::to_string(trace->replacements));
        trace->entries.push_back(std::move(entry));
        return result;
    }
    entry.worked = true;
    entry.certificate = al_r20_r42_practical_certificate(
        matrix, inverse, rhs, solution, AL_R20_R61_CENTER_DEPTH, false);
    entry.dots = al_r20_r60_certificate_dots(entry.certificate);
    entry.input_pairs = al_r20_r60_certificate_pairs(entry.certificate);
    result.exact = entry.certificate.exact;
    result.passed = entry.certificate.passed;
    result.error = entry.certificate.error;
    result.solution = entry.certificate.solution;
    result.root = entry.certificate.root;
    entry.replaced = result.passed;
    if (entry.replaced)
        ++trace->replacements;
    else {
        entry.rejection = "CERTIFICATE";
        ++trace->certificate_rejections;
    }
    trace->entries.push_back(std::move(entry));
    return result;
}

bool al_r20_r61_trace_exact(
    const ALR20R61Trace& trace,
    std::size_t problem_rows,
    std::size_t& worked,
    std::size_t& dots,
    std::size_t& input_pairs) {
    std::size_t local_worked = 0U;
    bool exact = trace.replacements <= AL_R20_SEMISMOOTH_ITERATION_CAP
        && trace.replacements + trace.certificate_rejections
            + trace.dimension_rejections + trace.cap_hits
            == trace.entries.size();
    for (const ALR20R61Entry& entry : trace.entries) {
        exact = exact && entry.dimension <= problem_rows;
        if (!entry.worked) {
            exact = exact
                && ((entry.rejection == "DIMENSION"
                        && (entry.dimension == 0U
                            || entry.dimension > AL_R20_R61_MAX_DIMENSION))
                    || (entry.rejection == "CAP"
                        && trace.replacements
                            == AL_R20_SEMISMOOTH_ITERATION_CAP));
            continue;
        }
        ++local_worked;
        const std::size_t dimension = entry.dimension;
        const std::size_t expected_dots = dimension * dimension
            + 20U * dimension;
        const std::size_t expected_pairs =
            dimension * dimension * (dimension + 1U)
            + dimension * (dimension + 1U)
            + dimension * dimension
            + AL_R20_R61_CENTER_DEPTH * dimension * (dimension + 1U)
            + dimension * (dimension + 2U) + 2U * dimension;
        exact = exact && dimension > 0U
            && dimension <= AL_R20_R61_MAX_DIMENSION
            && entry.certificate.exact && entry.certificate.no_underflow
            && entry.certificate.dimension == dimension
            && entry.dots == expected_dots
            && entry.input_pairs == expected_pairs
            && entry.replaced == entry.certificate.passed
            && ((entry.replaced && entry.rejection.empty())
                || (!entry.replaced && entry.rejection == "CERTIFICATE"));
        dots += entry.dots;
        input_pairs += entry.input_pairs;
    }
    exact = exact && local_worked
        == trace.replacements + trace.certificate_rejections;
    worked += local_worked;
    return exact;
}

std::string al_r20_r61_trace_root(const ALR20R61Trace& trace) {
    std::ostringstream material;
    material << trace.replacements << ':' << trace.certificate_rejections
        << ':' << trace.dimension_rejections << ':' << trace.cap_hits << '|';
    for (const ALR20R61Entry& entry : trace.entries)
        material << entry.dimension << ':' << entry.worked << ':'
            << entry.replaced << ':' << entry.rejection << ':'
            << entry.material_root << ':' << entry.certificate.root << ':'
            << entry.dots << ':' << entry.input_pairs << ';';
    return sha256_hex(material.str());
}

} // namespace

SplitBoundaryReport run_al_generalization_v5_profile_bounded_callback_controls() {
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
            && problem.problem_root == AL_R20_R59_PROBLEM_SHA256[index]
            && index < AL_R20_R61_ROWS.size()
            && problem.row_scale.size() == AL_R20_R61_ROWS[index];
        problems.push_back(std::move(problem));
    }

    std::vector<ALR20R35CaseAudit> diagnostics;
    std::vector<ALR20R61Trace> centered_traces;
    std::vector<ALR20R55SlopeTrace> slope_traces;
    std::size_t solver_executions = 0U;
    const bool hook_slot_clear = al_r20_verified_solution_refiner == nullptr
        && al_r20_r61_trace == nullptr
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr
        && al_r20_verified_inverse_capture == nullptr
        && al_r20_direct_solution_capture == nullptr;
    if (parent_exact && hook_slot_clear) {
        for (const ALR20ProblemData& problem : problems) {
            ALR20R61Trace centered;
            ALR20R55SlopeTrace slope;
            al_r20_r61_trace = &centered;
            al_r20_verified_solution_refiner = &al_r20_r61_refiner;
            al_r20_r55_slope_trace = &slope;
            al_r20_slope_error_refiner = &al_r20_r55_slope_refiner;
            diagnostics.push_back(al_r20_r35_case(problem));
            al_r20_slope_error_refiner = nullptr;
            al_r20_r55_slope_trace = nullptr;
            al_r20_verified_solution_refiner = nullptr;
            al_r20_r61_trace = nullptr;
            centered_traces.push_back(std::move(centered));
            slope_traces.push_back(std::move(slope));
            ++solver_executions;
        }
    }
    const bool hook_slot_released =
        al_r20_verified_solution_refiner == nullptr
        && al_r20_r61_trace == nullptr
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
        const ALR20R61Trace& centered = centered_traces[index];
        const ALR20R55SlopeTrace& slope = slope_traces[index];
        const bool centered_exact = al_r20_r61_trace_exact(centered,
            problems[index].row_scale.size(), total_centered_worked,
            total_dots, total_input_pairs);
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
    const bool work_exact = total_centered_calls
            <= cases.size() * AL_R20_SEMISMOOTH_ITERATION_CAP
        && total_centered_replacements
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
        ? "PROFILE_BOUNDED_V5_PARENT_REJECTED"
        : (!lifecycle_exact || !policy_exact || !work_exact
            ? "PROFILE_BOUNDED_V5_APPARATUS_REJECTED"
            : (all_certified
                ? "PROFILE_BOUNDED_V5_6_OF_6_CANDIDATE"
                : "PROFILE_BOUNDED_V5_LATER_BOUNDARY"));

    std::ostringstream case_material;
    std::ostringstream case_report;
    case_report << '[';
    for (std::size_t index = 0U; index < diagnostics.size(); ++index) {
        if (index != 0U) case_report << ',';
        const ALR20R35CaseAudit& audit = diagnostics[index];
        const ALR20R61Trace& centered = centered_traces[index];
        const ALR20R55SlopeTrace& slope = slope_traces[index];
        const std::string centered_root = al_r20_r61_trace_root(centered);
        const std::string slope_root = al_r20_r55_trace_root(slope);
        case_material << problems[index].problem_root << ':' << audit.root
            << ':' << centered_root << ':' << slope_root << ';';
        case_report << "{\"index\":" << index
            << ",\"name\":\"" << cases[index].fixture.name
            << "\",\"rows\":" << problems[index].row_scale.size()
            << ",\"problem_root\":\"" << problems[index].problem_root
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
            << ",\"centered_accounting\":"
            << (centered_accounting[index] ? "true" : "false")
            << ",\"centered_replacements\":" << centered.replacements
            << ",\"centered_rejections\":"
            << centered.certificate_rejections
            << ",\"dimension_rejections\":"
            << centered.dimension_rejections
            << ",\"centered_cap_hits\":" << centered.cap_hits
            << ",\"centered_trace_root\":\"" << centered_root
            << "\",\"centered_dimensions\":[";
        for (std::size_t entry_index = 0U;
             entry_index < centered.entries.size(); ++entry_index) {
            if (entry_index != 0U) case_report << ',';
            case_report << centered.entries[entry_index].dimension;
        }
        case_report << "]"
            << ",\"slope_calls\":" << slope.calls
            << ",\"slope_accounting\":"
            << (slope_accounting[index] ? "true" : "false")
            << ",\"slope_audits\":" << slope.inverse_audits
            << ",\"slope_cap_hits\":" << slope.cap_hits
            << ",\"slope_trace_root\":\"" << slope_root << "\"}";
    }
    case_report << ']';
    const std::string case_root = sha256_hex(case_material.str());
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r61-profile-bounded.v1|"
        << AL_R20_R60_SEMANTIC_SHA256 << ':' << route << ':' << case_root
        << ':' << certified_cases << ':' << total_centered_calls << ':'
        << total_centered_worked << ':' << total_centered_replacements << ':'
        << total_centered_rejections << ':' << total_dimension_rejections << ':'
        << total_centered_cap_hits << ':' << total_slope_calls << ':'
        << total_slope_audits << ':' << total_slope_cap_hits << ':'
        << total_inverse_columns << ':' << total_dots << ':' << total_input_pairs;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r61_profile_bounded.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r60_sha256\":\"" << AL_R20_R60_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"observation_exact\":"
        << (observation_exact ? "true" : "false")
        << ",\"policy_exact\":" << (policy_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"certified_cases\":" << certified_cases
        << ",\"maximum_dimension\":" << AL_R20_R61_MAX_DIMENSION
        << ",\"center_depth\":" << AL_R20_R61_CENTER_DEPTH
        << ",\"total_centered_calls\":" << total_centered_calls
        << ",\"total_centered_worked\":" << total_centered_worked
        << ",\"total_centered_replacements\":"
        << total_centered_replacements
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
        << ",\"new_mechanisms\":0,\"source_changes\":0"
        << ",\"retry_count\":0,\"parameter_changes\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
