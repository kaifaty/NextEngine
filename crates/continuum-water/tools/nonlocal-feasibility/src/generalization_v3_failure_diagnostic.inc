namespace {

constexpr const char* AL_R20_V3_GENERALIZATION_SHA256 =
    "d24dd8da48a1e1d9c15b91bc67241fff22e504fad4c01ef4fe82f56e13399b35";
constexpr std::array<std::size_t, 2U> AL_R20_V3_FAILED_CASES{{1U, 2U}};
constexpr std::array<const char*, 2U> AL_R20_V3_FAILED_CASE_SHA256{{
    "cd3ee19c49b2bd40a9ec1b39ee0010da7658cf80929379dda879b598501a0ec4",
    "7929dffcdb0de8f59f2865a8ce4d18b6af27eb21134f1738739a92ddafa29c5d",
}};

std::string al_r20_v3_failure_classification(
    const ALR20SemismoothCase& diagnostic) {
    if (diagnostic.steps.empty()) return "NO_FINAL_STEP";
    const ALR20SemismoothStep& step = diagnostic.steps.back();
    if (!step.natural_face_resolved)
        return step.ambiguous_rows > 0U
            ? "NATURAL_FACE_AMBIGUITY" : "NATURAL_FACE_EMPTY";
    if (!step.solve.exact && step.solve.failure != "NONE")
        return step.solve.failure;
    if (!step.direction_certified) return "DIRECTION_UNCERTIFIED";
    if (!step.accepted) return "GLOBALIZATION_UNACCEPTED";
    return "NO_REJECTION";
}

bool al_r20_v3_failure_identified(
    const ALR20SemismoothCase& diagnostic,
    const std::string& classification) {
    if (!diagnostic.active_set_rejected || diagnostic.steps.empty())
        return false;
    const ALR20SemismoothStep& step = diagnostic.steps.back();
    if (!step.natural_face_resolved)
        return classification == "NATURAL_FACE_AMBIGUITY"
            || classification == "NATURAL_FACE_EMPTY";
    return !step.solve.exact && step.solve.failure != "NONE"
        && classification == step.solve.failure;
}

} // namespace

SplitBoundaryReport run_al_generalization_v3_failure_diagnostic_controls() {
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
    std::vector<ALR20SemismoothCase> diagnostics;
    std::vector<std::string> classifications;
    bool reproduction_exact = parent_exact;
    bool mechanisms_identified = parent_exact;
    std::size_t solver_executions = 0U;
    if (parent_exact)
        for (std::size_t local = 0U;
             local < AL_R20_V3_FAILED_CASES.size(); ++local) {
            const std::size_t index = AL_R20_V3_FAILED_CASES[local];
            ALR20SemismoothCase diagnostic =
                al_r20_semismooth_case(problems[index], true);
            ++solver_executions;
            const std::string classification =
                al_r20_v3_failure_classification(diagnostic);
            reproduction_exact = reproduction_exact
                && diagnostic.root == AL_R20_V3_FAILED_CASE_SHA256[local];
            mechanisms_identified = mechanisms_identified
                && al_r20_v3_failure_identified(
                    diagnostic, classification);
            diagnostics.push_back(std::move(diagnostic));
            classifications.push_back(classification);
        }
    reproduction_exact = reproduction_exact
        && solver_executions == AL_R20_V3_FAILED_CASES.size();
    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const std::string route = !parent_exact
        ? "V3_FAILURE_PARENT_REJECTED"
        : (!reproduction_exact ? "V3_FAILURE_REPRODUCTION_REJECTED"
            : (mechanisms_identified
                ? "V3_FAILURE_MECHANISM_IDENTIFIED"
                : "V3_FAILURE_MECHANISM_UNRESOLVED"));
    const bool exact = parent_exact && reproduction_exact
        && mechanisms_identified && lifecycle_exact;
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r12-v3-failure-diagnostic.v1|"
             << AL_R20_V3_GENERALIZATION_SHA256 << ':' << route << '|';
    for (std::size_t local = 0U; local < diagnostics.size(); ++local) {
        const std::size_t index = AL_R20_V3_FAILED_CASES[local];
        semantic << problems[index].problem_root << ':'
                 << AL_R20_V3_FAILED_CASE_SHA256[local] << ':'
                 << diagnostics[local].root << ':'
                 << classifications[local] << ':'
                 << diagnostics[local].steps.back().root << ';';
    }
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r12_v3_failure_diagnostic.v1\""
           << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
           << ",\"route\":\"" << route << "\""
           << ",\"parent_v3_generalization_sha256\":\""
           << AL_R20_V3_GENERALIZATION_SHA256
           << "\",\"profile_exact\":"
           << (profile_exact ? "true" : "false")
           << ",\"controls_exact\":"
           << (controls_exact ? "true" : "false")
           << ",\"parent_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"reproduction_exact\":"
           << (reproduction_exact ? "true" : "false")
           << ",\"mechanisms_identified\":"
           << (mechanisms_identified ? "true" : "false")
           << ",\"lifecycle_exact\":"
           << (lifecycle_exact ? "true" : "false")
           << ",\"cases\":[";
    for (std::size_t local = 0U; local < diagnostics.size(); ++local) {
        if (local != 0U) report << ',';
        const std::size_t index = AL_R20_V3_FAILED_CASES[local];
        const ALR20SemismoothCase& value = diagnostics[local];
        const ALR20SemismoothStep& step = value.steps.back();
        report << "{\"name\":\"" << cases[index].fixture.name
               << "\",\"problem_sha256\":\""
               << problems[index].problem_root
               << "\",\"expected_case_sha256\":\""
               << AL_R20_V3_FAILED_CASE_SHA256[local]
               << "\",\"observed_case_sha256\":\"" << value.root
               << "\",\"classification\":\""
               << classifications[local]
               << "\",\"accepted_iterations\":"
               << value.accepted_iterations
               << ",\"final_step\":{\"iteration\":" << step.iteration
               << ",\"harness_exact\":"
               << (step.harness_exact ? "true" : "false")
               << ",\"natural_face_resolved\":"
               << (step.natural_face_resolved ? "true" : "false")
               << ",\"natural_face_size\":" << step.natural_face_size
               << ",\"ambiguous_rows\":" << step.ambiguous_rows
               << ",\"natural_face_sha256\":\""
               << step.natural_face_root
               << "\",\"hessian_sha256\":\"" << step.hessian_root
               << "\",\"nnqp_exact\":"
               << (step.solve.exact ? "true" : "false")
               << ",\"nnqp_terminated\":"
               << (step.solve.terminated ? "true" : "false")
               << ",\"nnqp_kkt\":"
               << (step.solve.kkt_certified ? "true" : "false")
               << ",\"failure\":\"" << step.solve.failure
               << "\",\"failure_source_row\":"
               << step.solve.failure_source_row
               << ",\"failure_value\":\""
               << al_binary128_text(step.solve.failure_value)
               << "\",\"failure_bound\":\""
               << al_binary128_text(step.solve.failure_bound)
               << "\",\"support\":" << step.solve.support_size
               << ",\"entered\":" << step.solve.entered
               << ",\"removed\":" << step.solve.removed
               << ",\"transitions\":" << step.solve.transitions
               << ",\"principal_solves\":"
               << step.solve.principal_solves
               << ",\"direction_error_bound\":\""
               << al_binary128_text(step.solve.direction_error_bound)
               << "\",\"primal_violation\":\""
               << al_binary128_text(step.solve.primal_violation)
               << "\",\"dual_violation\":\""
               << al_binary128_text(step.solve.dual_violation)
               << "\",\"complementarity\":\""
               << al_binary128_text(step.solve.complementarity)
               << "\",\"slope\":\""
               << al_binary128_text(step.solve.slope)
               << "\",\"slope_bound\":\""
               << al_binary128_text(step.solve.slope_bound)
               << "\",\"inverse_audits\":[";
        for (std::size_t audit = 0U;
             audit < step.solve.inverse_audits.size(); ++audit) {
            if (audit != 0U) report << ',';
            report << "\"" << step.solve.inverse_audits[audit].root
                   << "\"";
        }
        report << "],\"root\":\"" << step.root << "\"}}";
    }
    report << "]"
              ",\"solver_executions\":" << solver_executions
           << ",\"successful_case_solver_executions\":0"
              ",\"algorithm_changes\":0,\"timing_admitted\":false"
              ",\"repair_authority\":false"
              ",\"runtime_authority\":false"
              ",\"production_authority\":false"
              ",\"result_sha256\":\""
           << result_sha << "\"}";
    return {exact, report.str()};
}
