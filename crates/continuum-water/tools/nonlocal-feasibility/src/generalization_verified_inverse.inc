namespace {

constexpr const char* AL_R20_SEMISMOOTH_SHA256 =
    "8ca318ee6328d44e3a4a4f6959849d12b6e6a5da79fe271f8a00192b1f2f1c58";

bool al_r20_verified_inverse_controls() {
    const Binary128 zero = static_cast<Binary128>(0.0);
    const Binary128 one = static_cast<Binary128>(1.0);
    const Binary128 correlation = static_cast<Binary128>(0.999);
    ALR20ADMMWork work;
    const std::vector<Binary128> matrix{
        one, correlation, correlation, one};
    const ALR20ADMMFactor factor = al_r20_admm_factor_matrix(
        matrix, 2U, &work);
    std::vector<Binary128> solution;
    Binary128 residual = zero;
    Binary128 bound = zero;
    const bool solved = al_r20_admm_solve(factor,
        {one, static_cast<Binary128>(0.1)},
        &solution, &residual, &bound, &work);
    const Binary128 cheap = solved
        ? al_r20_newton_direction_error_bound(
            factor, residual, bound, solution) : HUGE_VALQ;
    const ALR20VerifiedInverse verified = solved
        ? al_r20_verified_inverse(factor, residual, bound,
            solution, cheap, &work) : ALR20VerifiedInverse{};
    const ALR20InverseNormAudit rejected = al_r20_inverse_norm_audit(
        {one, zero, zero, one},
        {zero, zero, zero, zero}, {zero, zero}, 2U);
    return factor.passed && solved && verified.passed
        && verified.column_solves == 2U
        && verified.norm.rho < one
        && verified.refined_error < verified.cheap_error
        && !rejected.passed && rejected.rho >= one;
}

} // namespace

SplitBoundaryReport run_al_generalization_verified_inverse_controls() {
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
    std::vector<ALR20ManifestCase> cases = al_r20_manifest_cases();
    const bool parent_exact = controls_exact
        && al_r20_manifest_semantic_root(cases) == AL_R20_MANIFEST_SHA256
        && cases.size() == AL_R20_PROBLEM_SHA256.size();
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    std::vector<ALR20ProblemData> problems;
    std::vector<ALR20SemismoothCase> diagnostics;
    bool harness_exact = parent_exact;
    bool all_certified = true;
    bool inverse_rejected = false;
    bool sign_unresolved = false;
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        ALR20ProblemData problem = al_r20_materialize_problem(
            cases[index], &sparse_work, &static_work, &adjacency_work);
        const bool problem_exact = problem.exact
            && index < AL_R20_PROBLEM_SHA256.size()
            && problem.problem_root == AL_R20_PROBLEM_SHA256[index]
            && problem.source_positive == 0U;
        ALR20SemismoothCase diagnostic = problem_exact
            ? al_r20_semismooth_case(problem, true)
            : ALR20SemismoothCase{};
        harness_exact = harness_exact
            && problem_exact && diagnostic.harness_exact;
        all_certified = all_certified && diagnostic.certified;
        for (const ALR20SemismoothStep& step : diagnostic.steps) {
            inverse_rejected = inverse_rejected
                || step.solve.failure == "VERIFIED_INVERSE_AUDIT_REJECTED";
            sign_unresolved = sign_unresolved
                || (step.solve.failure == "PASSIVE_SIGN_UNRESOLVED"
                    && !step.solve.inverse_audits.empty());
        }
        diagnostics.push_back(std::move(diagnostic));
        problems.push_back(std::move(problem));
    }
    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    harness_exact = harness_exact && lifecycle_exact;
    const std::string route = !parent_exact
        ? "VERIFIED_INVERSE_PARENT_REJECTED"
        : (inverse_rejected ? "VERIFIED_INVERSE_AUDIT_REJECTED"
            : (sign_unresolved ? "VERIFIED_INVERSE_SIGN_UNRESOLVED"
                : (all_certified
                    ? "VERIFIED_INVERSE_DEVELOPMENT_CERTIFIED"
                    : "VERIFIED_INVERSE_DEVELOPMENT_UNRESOLVED")));
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r8-verified-inverse.v1|"
             << AL_R20_SEMISMOOTH_SHA256 << ':' << route << '|';
    for (const ALR20SemismoothCase& value : diagnostics)
        semantic << value.root << ';';
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r8_verified_inverse.v1\""
           << ",\"status\":\"" << (harness_exact ? "PASS" : "FAIL") << "\""
           << ",\"route\":\"" << route << "\""
           << ",\"parent_semismooth_sha256\":\""
           << AL_R20_SEMISMOOTH_SHA256
           << "\",\"profile_exact\":" << (profile_exact ? "true" : "false")
           << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
           << ",\"lifecycle_exact\":" << (lifecycle_exact ? "true" : "false")
           << ",\"all_certified\":" << (all_certified ? "true" : "false")
           << ",\"cases\":[";
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20SemismoothCase& value = diagnostics[index];
        std::size_t inverse_audits = 0U;
        std::size_t inverse_column_solves = 0U;
        report << "{\"name\":\"" << cases[index].fixture.name
               << "\",\"role\":\"" << cases[index].role
               << "\",\"certified\":"
               << (value.certified ? "true" : "false")
               << ",\"accepted_iterations\":" << value.accepted_iterations
               << ",\"inverse_audits\":[";
        bool first_audit = true;
        for (const ALR20SemismoothStep& step : value.steps)
            for (const ALR20VerifiedInverse& audit :
                 step.solve.inverse_audits) {
                if (!first_audit) report << ',';
                first_audit = false;
                ++inverse_audits;
                inverse_column_solves += audit.column_solves;
                report << "{\"iteration\":" << step.iteration
                       << ",\"dimension\":" << audit.dimension
                       << ",\"column_solves\":" << audit.column_solves
                       << ",\"rho\":\""
                       << al_binary128_text(audit.norm.rho)
                       << "\",\"inverse_norm_bound\":\""
                       << al_binary128_text(audit.norm.inverse_norm_bound)
                       << "\",\"cheap_error\":\""
                       << al_binary128_text(audit.cheap_error)
                       << "\",\"refined_error\":\""
                       << al_binary128_text(audit.refined_error)
                       << "\",\"root\":\"" << audit.root << "\"}";
            }
        report << "],\"inverse_audit_count\":" << inverse_audits
               << ",\"inverse_column_solves\":" << inverse_column_solves
               << ",\"final_primal\":\""
               << al_binary128_text(value.final_metrics.primal)
               << "\",\"final_dual_mapping\":\""
               << al_binary128_text(value.final_metrics.dual_mapping)
               << "\",\"final_complementarity\":\""
               << al_binary128_text(value.final_metrics.complementarity)
               << "\",\"final_stationarity\":\""
               << al_binary128_text(value.final_metrics.stationarity)
               << "\",\"final_gap_scaled\":\""
               << al_binary128_text(value.final_metrics.gap_scaled)
               << "\",\"root\":\"" << value.root << "\"}";
    }
    report << "]"
              ",\"on_demand_only\":true,\"value_refinement\":false"
              ",\"pivot_changes\":0,\"diagonal_regularization\":false"
              ",\"accepted_iteration_cap\":32"
              ",\"old_semismooth_reruns\":0,\"timing_admitted\":false"
              ",\"generalization_authority\":false"
              ",\"runtime_authority\":false,\"production_authority\":false"
              ",\"result_sha256\":\""
           << result_sha << "\"}";
    return {harness_exact, report.str()};
}
