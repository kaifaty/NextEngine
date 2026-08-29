namespace {

constexpr const char* AL_R20_R35_SEMANTIC_SHA256 =
    "59c32c19863321eeee98838ef907ee02630c37ca032bb2846d655ae6165037b3";
constexpr const char* AL_R20_R35_TORSION_CASE_SHA256 =
    "a543bc0626ea63cd5b52bc06a6e6a08e405223985ccbfd0ad15d4c1bd3ac1d6d";
constexpr const char* AL_R20_R35_TORSION_STEP_SHA256 =
    "d274e20e6626833fe3fe4d97ad1dd58959c97923a0ece4ab4d35a89d7ba54cd9";
constexpr const char* AL_R20_R35_COUNTERFLOW_CASE_SHA256 =
    "7e2735e1658df0537cc3082d764f601df5e43f8a66a1e3d75942eb2b30dac5d3";
constexpr const char* AL_R20_R35_COUNTERFLOW_STEP_SHA256 =
    "c7a33746663809683c1ac1e8faec2725c1690d17631cb58cc6e16f577ff31590";

std::string al_r20_r36_inverse_classification(
    const ALR20VerifiedInverse& audit) {
    if (audit.passed) return "INVERSE_AUDIT_PASSED";
    if (audit.dimension == 0U
        || audit.column_solves != audit.dimension)
        return "INVERSE_FACTOR_OR_COLUMN_SOLVE_REJECTED";
    if (finiteq(audit.maximum_column_residual) == 0
        || finiteq(audit.maximum_column_bound) == 0
        || finiteq(audit.norm.rho) == 0)
        return "INVERSE_AUDIT_NONFINITE";
    if (!audit.norm.passed
        && audit.norm.rho >= static_cast<Binary128>(1.0))
        return "INVERSE_NORM_RHO_NOT_CONTRACTIVE";
    if (audit.norm.passed
        && finiteq(audit.refined_error) == 0)
        return "INVERSE_REFINED_ERROR_NONFINITE";
    return "INVERSE_AUDIT_OTHER_REJECTION";
}

struct ALR20R36SlopeAudit {
    bool exact = false;
    bool natural_face_parity = false;
    bool slope_parity = false;
    bool bound_parity = false;
    bool bound_dominated = false;
    Binary128 nominal_slope = static_cast<Binary128>(0.0);
    Binary128 residual_bound_term = static_cast<Binary128>(0.0);
    Binary128 direction_error_term = static_cast<Binary128>(0.0);
    Binary128 rounding_term = static_cast<Binary128>(0.0);
    Binary128 ordered_bound = static_cast<Binary128>(0.0);
    Binary128 residual_ratio = static_cast<Binary128>(0.0);
    Binary128 direction_ratio = static_cast<Binary128>(0.0);
    Binary128 rounding_ratio = static_cast<Binary128>(0.0);
    std::size_t natural_face_size = 0U;
    std::string classification;
    std::string current_root;
    std::string direction_root;
    std::string root;
};

ALR20R36SlopeAudit al_r20_r36_slope_audit(
    const ALR20ProblemData& problem,
    const ALR20R35CaseAudit& parent) {
    ALR20R36SlopeAudit audit;
    const ALR20SemismoothCase& diagnostic = parent.diagnostic;
    if (parent.root != AL_R20_R35_COUNTERFLOW_CASE_SHA256
        || diagnostic.steps.empty()
        || diagnostic.final_lambda.size() != problem.row_scale.size())
        return audit;
    const ALR20SemismoothStep& step = diagnostic.steps.back();
    if (step.root != AL_R20_R35_COUNTERFLOW_STEP_SHA256)
        return audit;
    const ALR20NewtonDualEvaluation current =
        al_r20_newton_dual_evaluate(problem, diagnostic.final_lambda);
    const std::vector<Binary128> diagonal = al_r20_q_diagonal(problem.op);
    std::vector<std::size_t> natural_face;
    for (std::size_t row = 0U;
         row < diagnostic.final_lambda.size(); ++row) {
        const Binary128 quotient = current.residual[row] / diagonal[row];
        const Binary128 argument = diagnostic.final_lambda[row] + quotient;
        const Binary128 bound = current.residual_bound[row] / diagonal[row]
            + al_r20_q_gamma(256U)
                * (fabsq(diagnostic.final_lambda[row]) + fabsq(quotient));
        if (argument - bound > static_cast<Binary128>(0.0))
            natural_face.push_back(row);
    }
    std::ostringstream face_material;
    for (std::size_t row : natural_face) face_material << row << ',';
    audit.natural_face_size = natural_face.size();
    audit.natural_face_parity = natural_face.size()
            == step.natural_face_size
        && sha256_hex(face_material.str()) == step.natural_face_root;
    std::vector<Binary128> direction(diagnostic.final_lambda.size());
    Binary128Accumulator slope;
    Binary128 slope_absolute = static_cast<Binary128>(0.0);
    Binary128 residual_bound_term = static_cast<Binary128>(0.0);
    Binary128 direction_error_term = static_cast<Binary128>(0.0);
    Binary128 ordered_bound = static_cast<Binary128>(0.0);
    for (std::size_t row = 0U; row < direction.size(); ++row) {
        direction[row] = step.line_representative[row]
            - diagnostic.final_lambda[row];
        const Binary128 slope_term = current.residual[row] * direction[row];
        slope.add(slope_term);
        slope_absolute += fabsq(slope_term);
        const Binary128 direction_bound = std::binary_search(
            natural_face.begin(), natural_face.end(), row)
            ? step.direction_error_bound
            : static_cast<Binary128>(0.0);
        const Binary128 residual_term = current.residual_bound[row]
            * (fabsq(direction[row]) + direction_bound);
        const Binary128 direction_term =
            fabsq(current.residual[row]) * direction_bound;
        residual_bound_term += residual_term;
        direction_error_term += direction_term;
        ordered_bound += residual_term + direction_term;
    }
    audit.nominal_slope = slope.value();
    audit.residual_bound_term = residual_bound_term;
    audit.direction_error_term = direction_error_term;
    audit.rounding_term = al_r20_q_gamma(16U * direction.size() + 512U)
        * slope_absolute;
    ordered_bound += audit.rounding_term;
    audit.ordered_bound = ordered_bound;
    audit.slope_parity = al_binary128_hex(audit.nominal_slope)
        == al_binary128_hex(step.slope);
    audit.bound_parity = al_binary128_hex(audit.ordered_bound)
        == al_binary128_hex(step.slope_bound);
    audit.bound_dominated = step.slope_bound >= fabsq(step.slope);
    const Binary128 denominator = fmaxq(fabsq(step.slope),
        ldexpq(static_cast<Binary128>(1.0), -16382));
    audit.residual_ratio = audit.residual_bound_term / denominator;
    audit.direction_ratio = audit.direction_error_term / denominator;
    audit.rounding_ratio = audit.rounding_term / denominator;
    audit.current_root = current.root;
    audit.direction_root = al_r20_q_scalar_root(direction);
    if (finiteq(step.slope) == 0 || !(step.slope > 0))
        audit.classification = "NONPOSITIVE_OR_NONFINITE_MODEL";
    else if (!audit.slope_parity || !audit.bound_parity)
        audit.classification = "SLOPE_DECOMPOSITION_REJECTED";
    else if (!audit.bound_dominated)
        audit.classification = "SLOPE_CERTIFIED";
    else if (audit.direction_error_term >= audit.residual_bound_term
        && audit.direction_error_term >= audit.rounding_term)
        audit.classification = "DIRECTION_ERROR_BOUND_DOMINATES";
    else if (audit.residual_bound_term >= audit.rounding_term)
        audit.classification = "RESIDUAL_BOUND_DOMINATES";
    else
        audit.classification = "ROUNDING_BOUND_DOMINATES";
    audit.exact = current.exact && audit.natural_face_parity
        && al_r20_admm_finite(direction)
        && audit.slope_parity && audit.bound_parity
        && finiteq(audit.residual_bound_term) != 0
        && finiteq(audit.direction_error_term) != 0
        && finiteq(audit.rounding_term) != 0;
    std::ostringstream material;
    material << audit.exact << ':' << audit.natural_face_parity << ':'
        << audit.natural_face_size << ':' << audit.slope_parity << ':'
        << audit.bound_parity << ':' << audit.bound_dominated << ':'
        << audit.classification << ':'
        << al_binary128_hex(audit.nominal_slope) << ':'
        << al_binary128_hex(audit.residual_bound_term) << ':'
        << al_binary128_hex(audit.direction_error_term) << ':'
        << al_binary128_hex(audit.rounding_term) << ':'
        << al_binary128_hex(audit.ordered_bound) << ':'
        << al_binary128_hex(audit.residual_ratio) << ':'
        << al_binary128_hex(audit.direction_ratio) << ':'
        << al_binary128_hex(audit.rounding_ratio) << ':'
        << audit.current_root << ':' << audit.direction_root;
    audit.root = sha256_hex(material.str());
    return audit;
}

} // namespace

SplitBoundaryReport run_al_generalization_v4_failure_budget_controls() {
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
    std::vector<ALR20ManifestCase> cases = al_r20_v4_manifest_cases();
    const std::array<std::size_t, 2U> selected{{0U, 2U}};
    bool parent_exact = controls_exact && cases.size() == 5U
        && al_r20_v4_manifest_root(cases) == AL_R20_V4_MANIFEST_SHA256;
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    std::vector<ALR20ProblemData> problems;
    if (parent_exact) {
        for (std::size_t index : selected) {
            ALR20ProblemData problem = al_r20_materialize_problem(
                cases[index], &sparse_work, &static_work, &adjacency_work);
            parent_exact = parent_exact && problem.exact
                && problem.source_positive == 0U
                && problem.problem_root == AL_R20_V4_PROBLEM_SHA256[index];
            problems.push_back(std::move(problem));
        }
    }
    ALR20R35CaseAudit torsion;
    ALR20R35CaseAudit counterflow;
    std::size_t solver_executions = 0U;
    if (parent_exact && problems.size() == selected.size()) {
        torsion = al_r20_r35_case(problems[0U]);
        counterflow = al_r20_r35_case(problems[1U]);
        solver_executions = 2U;
    }
    bool reproduction_exact = parent_exact
        && torsion.root == AL_R20_R35_TORSION_CASE_SHA256
        && counterflow.root == AL_R20_R35_COUNTERFLOW_CASE_SHA256
        && !torsion.diagnostic.steps.empty()
        && !counterflow.diagnostic.steps.empty()
        && torsion.diagnostic.steps.back().root
            == AL_R20_R35_TORSION_STEP_SHA256
        && counterflow.diagnostic.steps.back().root
            == AL_R20_R35_COUNTERFLOW_STEP_SHA256
        && solver_executions == 2U;
    const ALR20SemismoothStep torsion_step =
        torsion.diagnostic.steps.empty()
            ? ALR20SemismoothStep{}
            : torsion.diagnostic.steps.back();
    const ALR20SemismoothStep counterflow_step =
        counterflow.diagnostic.steps.empty()
            ? ALR20SemismoothStep{}
            : counterflow.diagnostic.steps.back();
    std::size_t rejected_inverse_audits = 0U;
    std::string torsion_classification = "TORSION_INVERSE_UNRESOLVED";
    for (const ALR20VerifiedInverse& audit : torsion_step.solve.inverse_audits)
        if (!audit.passed) {
            ++rejected_inverse_audits;
            torsion_classification =
                al_r20_r36_inverse_classification(audit);
        }
    const bool torsion_audit_exact = reproduction_exact
        && torsion_step.solve.failure == "VERIFIED_INVERSE_AUDIT_REJECTED"
        && !torsion_step.solve.inverse_audits.empty()
        && rejected_inverse_audits == 1U;
    const ALR20R36SlopeAudit counterflow_slope = reproduction_exact
        ? al_r20_r36_slope_audit(problems[1U], counterflow)
        : ALR20R36SlopeAudit{};

    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 2U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool exact = parent_exact && reproduction_exact
        && torsion_audit_exact && counterflow_slope.exact
        && lifecycle_exact;
    const std::string route = !parent_exact
        ? "V4_FAILURE_BUDGET_PARENT_REJECTED"
        : (!reproduction_exact
            ? "V4_FAILURE_BUDGET_REPRODUCTION_REJECTED"
            : (!torsion_audit_exact || !counterflow_slope.exact
                ? "V4_FAILURE_BUDGET_AUDIT_REJECTED"
                : "V4_FAILURE_BUDGET_IDENTIFIED"));
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r36-failure-budget.v1|"
        << AL_R20_R35_SEMANTIC_SHA256 << ':' << route << ':'
        << torsion_classification << ':'
        << counterflow_slope.classification << '|';
    for (const ALR20VerifiedInverse& audit : torsion_step.solve.inverse_audits)
        semantic << audit.root << ';';
    semantic << '|' << counterflow_slope.root;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r36_failure_budget.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_v4_solver_sha256\":\""
        << AL_R20_R35_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"reproduction_exact\":"
        << (reproduction_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"torsion\":{\"classification\":\""
        << torsion_classification << "\""
        << ",\"case_sha256\":\"" << torsion.root
        << "\",\"step_sha256\":\"" << torsion_step.root
        << "\",\"inverse_audits\":[";
    for (std::size_t index = 0U;
         index < torsion_step.solve.inverse_audits.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20VerifiedInverse& audit =
            torsion_step.solve.inverse_audits[index];
        report << "{\"passed\":" << (audit.passed ? "true" : "false")
            << ",\"classification\":\""
            << al_r20_r36_inverse_classification(audit)
            << "\",\"dimension\":" << audit.dimension
            << ",\"column_solves\":" << audit.column_solves
            << ",\"cheap_error\":\""
            << al_binary128_text(audit.cheap_error)
            << "\",\"refined_error\":\""
            << al_binary128_text(audit.refined_error)
            << "\",\"maximum_column_residual\":\""
            << al_binary128_text(audit.maximum_column_residual)
            << "\",\"maximum_column_bound\":\""
            << al_binary128_text(audit.maximum_column_bound)
            << "\",\"norm_passed\":"
            << (audit.norm.passed ? "true" : "false")
            << ",\"rho\":\"" << al_binary128_text(audit.norm.rho)
            << "\",\"inverse_norm_bound\":\""
            << al_binary128_text(audit.norm.inverse_norm_bound)
            << "\",\"norm_sha256\":\"" << audit.norm.root
            << "\",\"root\":\"" << audit.root << "\"}";
    }
    report << "]}"
        << ",\"counterflow\":{\"classification\":\""
        << counterflow_slope.classification << "\""
        << ",\"case_sha256\":\"" << counterflow.root
        << "\",\"step_sha256\":\""
        << counterflow_step.root
        << "\",\"natural_face_parity\":"
        << (counterflow_slope.natural_face_parity ? "true" : "false")
        << ",\"natural_face_size\":"
        << counterflow_slope.natural_face_size
        << ",\"slope_parity\":"
        << (counterflow_slope.slope_parity ? "true" : "false")
        << ",\"bound_parity\":"
        << (counterflow_slope.bound_parity ? "true" : "false")
        << ",\"bound_dominated\":"
        << (counterflow_slope.bound_dominated ? "true" : "false")
        << ",\"nominal_slope\":\""
        << al_binary128_text(counterflow_slope.nominal_slope)
        << "\",\"stored_slope_bound\":\""
        << al_binary128_text(counterflow_step.slope_bound)
        << "\",\"ordered_slope_bound\":\""
        << al_binary128_text(counterflow_slope.ordered_bound)
        << "\",\"residual_bound_term\":\""
        << al_binary128_text(counterflow_slope.residual_bound_term)
        << "\",\"direction_error_term\":\""
        << al_binary128_text(counterflow_slope.direction_error_term)
        << "\",\"rounding_term\":\""
        << al_binary128_text(counterflow_slope.rounding_term)
        << "\",\"residual_ratio\":\""
        << al_binary128_text(counterflow_slope.residual_ratio)
        << "\",\"direction_ratio\":\""
        << al_binary128_text(counterflow_slope.direction_ratio)
        << "\",\"rounding_ratio\":\""
        << al_binary128_text(counterflow_slope.rounding_ratio)
        << "\",\"direction_error_bound\":\""
        << al_binary128_text(counterflow_step.direction_error_bound)
        << "\",\"nnqp_slope\":\""
        << al_binary128_text(counterflow_step.solve.slope)
        << "\",\"nnqp_slope_bound\":\""
        << al_binary128_text(counterflow_step.solve.slope_bound)
        << "\",\"current_sha256\":\""
        << counterflow_slope.current_root
        << "\",\"direction_sha256\":\""
        << counterflow_slope.direction_root
        << "\",\"root\":\"" << counterflow_slope.root << "\"}"
        << ",\"solver_executions\":" << solver_executions
        << ",\"new_inverse_solves\":0,\"new_trials\":0,\"state_updates\":0"
        << ",\"parameter_changes\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
