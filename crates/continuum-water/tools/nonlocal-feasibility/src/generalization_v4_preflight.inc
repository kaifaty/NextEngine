namespace {

constexpr const char* AL_R20_V4_MANIFEST_SHA256 =
    "33ff1cbf1a8ae753fb84a607d728e0e82ef487d44875ee7615e1d4aa19fcee63";

struct ALR20V4PreflightCase {
    bool exact = false;
    bool excited = false;
    std::size_t certified_positive = 0U;
    double certified_maximum = 0.0;
    std::string problem_root;
    std::string root;
};

} // namespace

SplitBoundaryReport run_al_generalization_v4_preflight_controls() {
    std::vector<ALR20ManifestCase> cases = al_r20_v4_manifest_cases();
    const bool parent_exact = cases.size() == 5U
        && al_r20_v4_manifest_root(cases) == AL_R20_V4_MANIFEST_SHA256;
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    std::vector<ALR20ProblemData> problems;
    std::vector<ALR20V4PreflightCase> diagnostics;
    bool operator_exact = parent_exact;
    bool all_excited = parent_exact;
    bool scaling_exact = true;
    double maximum_primal_gap = 0.0;
    double maximum_dual_gap = 0.0;
    double maximum_scale_gap = 0.0;
    for (const ALR20ManifestCase& source : cases) {
        ALR20ProblemData problem = al_r20_materialize_problem(
            source, &sparse_work, &static_work, &adjacency_work);
        ALR20V4PreflightCase diagnostic;
        diagnostic.problem_root = problem.problem_root;
        diagnostic.exact = problem.exact
            && problem.structure.slot && problem.structure.entry
            && problem.structure.incidence
            && problem.source_positive == 0U;
        if (problem.exact) {
            for (std::size_t row = 0U;
                 row < problem.projected_raw.size(); ++row)
                if (problem.projected_raw[row] > problem.row_bound[row]) {
                    ++diagnostic.certified_positive;
                    diagnostic.certified_maximum = std::max(
                        diagnostic.certified_maximum,
                        problem.projected_raw[row] - problem.row_bound[row]);
                }
            const std::array<std::size_t, 3U> selected{{
                0U, problem.diagonal.size() / 2U,
                problem.diagonal.size() - 1U}};
            for (std::size_t row : selected)
                for (double factor : {AL_R20_SCALE_LOW, AL_R20_SCALE_HIGH}) {
                    double primal_gap = 0.0;
                    double dual_gap = 0.0;
                    double scale_gap = 0.0;
                    const bool scale_pass = al_r20_scale_control(
                        problem.projected_raw[row],
                        problem.workspace.support.constraint[row],
                        problem.diagonal[row], problem.row_bound[row], factor,
                        &primal_gap, &dual_gap, &scale_gap);
                    scaling_exact = scaling_exact && scale_pass;
                    maximum_primal_gap = std::max(
                        maximum_primal_gap, primal_gap);
                    maximum_dual_gap = std::max(
                        maximum_dual_gap, dual_gap);
                    maximum_scale_gap = std::max(
                        maximum_scale_gap, scale_gap);
                }
        }
        diagnostic.excited = diagnostic.exact
            && diagnostic.certified_positive > 0U;
        std::ostringstream material;
        material << diagnostic.exact << ':' << diagnostic.excited << ':'
            << diagnostic.certified_positive << ':'
            << binary64_bits(diagnostic.certified_maximum) << ':'
            << problem.target_root << ':' << problem.constraint_root << ':'
            << problem.domain_root << ':' << problem.operator_root << ':'
            << problem.problem_root;
        diagnostic.root = sha256_hex(material.str());
        operator_exact = operator_exact && diagnostic.exact;
        all_excited = all_excited && diagnostic.excited;
        diagnostics.push_back(std::move(diagnostic));
        problems.push_back(std::move(problem));
    }

    std::vector<std::string> roots;
    for (const ALR20V4PreflightCase& value : diagnostics)
        roots.push_back(value.problem_root);
    std::sort(roots.begin(), roots.end());
    const bool distinct_problems = roots.size() == cases.size()
        && std::adjacent_find(roots.begin(), roots.end()) == roots.end();
    operator_exact = operator_exact && scaling_exact && distinct_problems;
    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    operator_exact = operator_exact && lifecycle_exact;
    const std::string route = !parent_exact
        ? "V4_PREFLIGHT_PARENT_REJECTED"
        : (!operator_exact
            ? "V4_PREFLIGHT_OPERATOR_REJECTED"
            : (!all_excited
                ? "V4_PREFLIGHT_EXCITATION_REJECTED"
                : "V4_PREFLIGHT_PASS"));
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r34-v4-preflight.v1|"
        << AL_R20_V4_MANIFEST_SHA256 << ':' << route << '|';
    for (const ALR20V4PreflightCase& value : diagnostics)
        semantic << value.root << ';';
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r34_v4_preflight.v1\""
        << ",\"status\":\"" << (operator_exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_v4_manifest_sha256\":\""
        << AL_R20_V4_MANIFEST_SHA256 << "\""
        << ",\"scaling_exact\":" << (scaling_exact ? "true" : "false")
        << ",\"distinct_problems\":"
        << (distinct_problems ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"all_excited\":" << (all_excited ? "true" : "false")
        << ",\"maximum_scale_gaps\":{\"primal\":"
        << maximum_primal_gap << ",\"dual\":" << maximum_dual_gap
        << ",\"scale\":" << maximum_scale_gap << "},\"cases\":[";
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20ProblemData& problem = problems[index];
        const ALR20V4PreflightCase& value = diagnostics[index];
        report << "{\"name\":\"" << cases[index].fixture.name
            << "\",\"rows\":" << problem.row_scale.size()
            << ",\"slots\":" << problem.op.slots.size()
            << ",\"entries\":" << problem.op.entries.size()
            << ",\"source_positive\":" << problem.source_positive
            << ",\"projected_positive\":" << problem.projected_positive
            << ",\"certified_positive\":" << value.certified_positive
            << ",\"projected_maximum\":" << problem.projected_maximum
            << ",\"certified_maximum\":" << value.certified_maximum
            << ",\"target_sha256\":\"" << problem.target_root
            << "\",\"constraint_sha256\":\"" << problem.constraint_root
            << "\",\"domain_sha256\":\"" << problem.domain_root
            << "\",\"operator_sha256\":\"" << problem.operator_root
            << "\",\"problem_sha256\":\"" << problem.problem_root
            << "\",\"root\":\"" << value.root << "\"}";
    }
    report << "]"
        << ",\"solver_iterations\":0,\"candidate_iterations\":0"
        << ",\"inverse_audits\":0,\"timing_admitted\":false"
        << ",\"generalization_authority\":false"
        << ",\"runtime_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {operator_exact, report.str()};
}
