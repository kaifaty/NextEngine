namespace {

constexpr const char* AL_R20_R43_SEMANTIC_SHA256 =
    "7a1e4aaaeb6745361b5bd3f4dcab8a85a5c0c582ceaecf74ef399e2d680f8fa8";
constexpr const char* AL_R20_R43_CASE_SHA256 =
    "787558e051df601ac5cd56e9fe6cb864fc109f067e8a974f89448988b8f68910";
constexpr const char* AL_R20_R43_STEP_SHA256 =
    "4894d0b167dd0f1950f154474b16e4e1022fef767bfa489c8001233f5d6ee494";
constexpr const char* AL_R20_R43_CERTIFICATE_SHA256 =
    "0d3e191465921512714a5dc9b7507a7216126bb301f2fc0763cb79a7893f6849";
constexpr const char* AL_R20_R43_SECOND_MATERIAL_SHA256 =
    "7e90da896485a868cad7eb2fab1c8f48e7a33611bc3763fe6328d56fe4d5f683";
constexpr const char* AL_R20_R43_SECOND_MATRIX_SHA256 =
    "f8114cbbe9caec3d2dccb8d1f7988030450e70b218a8efcadabd69ad7b33be97";
constexpr const char* AL_R20_R43_SECOND_INVERSE_SHA256 =
    "a4fa12c94d1f08efc37ed8371b5ccd0befe105909ebb6ad8735842c83eb479cf";
constexpr const char* AL_R20_R43_SECOND_RHS_SHA256 =
    "53cb4684dd776446547f341f997a0d5a27151859c78d0ff2b2e17adb9470dd32";
constexpr const char* AL_R20_R43_SECOND_SOLUTION_SHA256 =
    "1e51c3ad6f5498ea9e7d8c85ec8d92d7c26a73c5182d57bca5eda92b35bc523a";

struct ALR20R44Capture {
    std::size_t attempts = 0U;
    std::size_t target_matches = 0U;
    std::size_t replacements = 0U;
    std::size_t second_matches = 0U;
    ALR20R42PracticalCertificate first_certificate;
    std::vector<Binary128> matrix;
    std::vector<Binary128> inverse;
    std::vector<Binary128> rhs;
    std::vector<Binary128> solution;
    std::string second_material_root;
};

thread_local ALR20R44Capture* al_r20_r44_capture = nullptr;

ALR20VerifiedSolutionRefinement al_r20_r44_refiner(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& inverse,
    const std::vector<Binary128>& rhs,
    const std::vector<Binary128>& solution) {
    ALR20VerifiedSolutionRefinement result;
    ALR20R44Capture* capture = al_r20_r44_capture;
    if (capture == nullptr) return result;
    ++capture->attempts;
    const std::string matrix_root = al_r20_q_scalar_root(matrix);
    const std::string inverse_root = al_r20_q_scalar_root(inverse);
    const std::string rhs_root = al_r20_q_scalar_root(rhs);
    const std::string solution_root = al_r20_q_scalar_root(solution);
    const std::string material_root = sha256_hex(matrix_root + ':'
        + inverse_root + ':' + rhs_root + ':' + solution_root);
    const bool first_target = matrix_root == AL_R20_R37_MATRIX_SHA256
        && inverse_root == AL_R20_R37_INVERSE_SHA256
        && rhs_root == AL_R20_R39_RHS_SHA256
        && solution_root == AL_R20_R39_SOLUTION_SHA256;
    if (first_target && capture->replacements == 0U) {
        ++capture->target_matches;
        capture->first_certificate = al_r20_r42_practical_certificate(
            matrix, inverse, rhs, solution, 8U);
        result.exact = capture->first_certificate.exact;
        result.passed = capture->first_certificate.passed;
        result.error = capture->first_certificate.error;
        result.solution = capture->first_certificate.solution;
        result.root = capture->first_certificate.root;
        if (result.passed) ++capture->replacements;
        return result;
    }
    const bool second_target = rhs.size() == 65U
        && material_root == AL_R20_R43_SECOND_MATERIAL_SHA256
        && matrix_root == AL_R20_R43_SECOND_MATRIX_SHA256
        && inverse_root == AL_R20_R43_SECOND_INVERSE_SHA256
        && rhs_root == AL_R20_R43_SECOND_RHS_SHA256
        && solution_root == AL_R20_R43_SECOND_SOLUTION_SHA256;
    if (second_target) {
        ++capture->second_matches;
        capture->matrix = matrix;
        capture->inverse = inverse;
        capture->rhs = rhs;
        capture->solution = solution;
        capture->second_material_root = material_root;
    }
    result.exact = true;
    result.root = sha256_hex(std::string("r43-non-target:")
        + material_root + ':' + std::to_string(capture->replacements));
    return result;
}

} // namespace

SplitBoundaryReport
run_al_generalization_v4_second_inverse_contraction_controls() {
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
    ALR20ProblemData problem;
    bool problem_materialized = false;
    if (parent_exact) {
        problem = al_r20_materialize_problem(
            cases[0U], &sparse_work, &static_work, &adjacency_work);
        problem_materialized = true;
        parent_exact = problem.exact && problem.source_positive == 0U
            && problem.problem_root == AL_R20_V4_PROBLEM_SHA256[0U];
    }
    ALR20R44Capture capture;
    ALR20R35CaseAudit torsion;
    std::size_t solver_executions = 0U;
    const bool hook_slot_clear = al_r20_verified_solution_refiner == nullptr
        && al_r20_r44_capture == nullptr;
    if (parent_exact && hook_slot_clear) {
        al_r20_r44_capture = &capture;
        al_r20_verified_solution_refiner = &al_r20_r44_refiner;
        torsion = al_r20_r35_case(problem);
        al_r20_verified_solution_refiner = nullptr;
        al_r20_r44_capture = nullptr;
        solver_executions = 1U;
    }
    const bool hook_slot_released = al_r20_verified_solution_refiner == nullptr
        && al_r20_r44_capture == nullptr;
    const bool trajectory_exact = parent_exact && hook_slot_clear
        && hook_slot_released && solver_executions == 1U
        && capture.attempts == 2U && capture.target_matches == 1U
        && capture.replacements == 1U && capture.second_matches == 1U
        && capture.first_certificate.root == AL_R20_R43_CERTIFICATE_SHA256
        && torsion.root == AL_R20_R43_CASE_SHA256
        && !torsion.diagnostic.steps.empty()
        && torsion.diagnostic.steps.back().root == AL_R20_R43_STEP_SHA256
        && torsion.diagnostic.steps.back().solve.failure
            == "VERIFIED_INVERSE_AUDIT_REJECTED"
        && capture.second_material_root
            == AL_R20_R43_SECOND_MATERIAL_SHA256;
    const ALR20R37ExactResidual right_oracle = trajectory_exact
        ? al_r20_r37_exact_residual(capture.matrix, capture.inverse, 65U)
        : ALR20R37ExactResidual{};
    const ALR20R38MatrixAudit right_certificate = right_oracle.exact
        ? al_r20_r38_matrix_audit(
            capture.matrix, capture.inverse, right_oracle)
        : ALR20R38MatrixAudit{};
    const ALR20R37ExactResidual left_oracle = trajectory_exact
        ? al_r20_r37_exact_residual(capture.inverse, capture.matrix, 65U)
        : ALR20R37ExactResidual{};
    const ALR20R38MatrixAudit left_certificate = left_oracle.exact
        ? al_r20_r38_matrix_audit(
            capture.inverse, capture.matrix, left_oracle)
        : ALR20R38MatrixAudit{};
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool apparatus_exact = right_oracle.exact
        && right_certificate.exact && left_oracle.exact
        && left_certificate.exact;
    const bool exact = trajectory_exact && apparatus_exact && lifecycle_exact;
    const std::string route = !parent_exact || !trajectory_exact
        ? "SECOND_INVERSE_PARENT_REJECTED"
        : (capture.second_matches != 1U
            ? "SECOND_INVERSE_CAPTURE_REJECTED"
            : (!apparatus_exact
                ? "SECOND_INVERSE_APPARATUS_REJECTED"
                : (!right_certificate.no_underflow
                        || !left_certificate.no_underflow
                    ? "SECOND_INVERSE_UNDERFLOW_REJECTED"
                    : (!right_certificate.contained
                            || !left_certificate.contained
                        ? "SECOND_INVERSE_CONTAINMENT_REJECTED"
                        : (!right_certificate.contractive
                            ? "SECOND_INVERSE_RIGHT_NONCONTRACTIVE"
                            : (!left_certificate.contractive
                                ? "SECOND_INVERSE_LEFT_NONCONTRACTIVE"
                                : "SECOND_INVERSE_CONTRACTIVE_CANDIDATE"))))));
    const ALR20NNQPSolve& final_solve =
        torsion.diagnostic.steps.back().solve;
    const ALR20VerifiedInverse* inherited = final_solve.inverse_audits.empty()
        ? nullptr : &final_solve.inverse_audits.back();
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r44-second-inverse.v1|"
        << AL_R20_R43_SEMANTIC_SHA256 << ':' << route << ':'
        << right_oracle.root << ':' << right_certificate.root << ':'
        << left_oracle.root << ':' << left_certificate.root;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r44_second_inverse.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r43_sha256\":\""
        << AL_R20_R43_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"trajectory_exact\":"
        << (trajectory_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"capture\":{\"attempts\":" << capture.attempts
        << ",\"first_replacements\":" << capture.replacements
        << ",\"second_matches\":" << capture.second_matches
        << ",\"second_material_root\":\""
        << capture.second_material_root << "\"}"
        << ",\"inherited\":{\"passed\":"
        << (inherited != nullptr && inherited->passed ? "true" : "false")
        << ",\"root\":\"" << (inherited == nullptr ? "" : inherited->root)
        << "\",\"rho\":\""
        << al_binary128_text(inherited == nullptr
            ? static_cast<Binary128>(0.0) : inherited->norm.rho)
        << "\",\"maximum_column_residual\":\""
        << al_binary128_text(inherited == nullptr
            ? static_cast<Binary128>(0.0)
            : inherited->maximum_column_residual)
        << "\",\"maximum_column_bound\":\""
        << al_binary128_text(inherited == nullptr
            ? static_cast<Binary128>(0.0)
            : inherited->maximum_column_bound) << "\"}"
        << ",\"right\":{\"exact_root\":\"" << right_oracle.root
        << "\",\"dot2_root\":\"" << right_certificate.root
        << "\",\"exact_rho_outward\":\""
        << al_binary128_text(right_oracle.outward_norm)
        << "\",\"rho_bound\":\""
        << al_binary128_text(right_certificate.rho_bound)
        << "\",\"worst_row\":" << right_certificate.worst_row
        << ",\"contained_entries\":"
        << right_certificate.contained_entries
        << ",\"maximum_dot_bound\":\""
        << al_binary128_text(right_certificate.maximum_dot_bound)
        << "\",\"maximum_exact_error\":\""
        << al_binary128_text(right_certificate.maximum_exact_error) << "\"}"
        << ",\"left\":{\"exact_root\":\"" << left_oracle.root
        << "\",\"dot2_root\":\"" << left_certificate.root
        << "\",\"exact_rho_outward\":\""
        << al_binary128_text(left_oracle.outward_norm)
        << "\",\"rho_bound\":\""
        << al_binary128_text(left_certificate.rho_bound)
        << "\",\"worst_row\":" << left_certificate.worst_row
        << ",\"contained_entries\":"
        << left_certificate.contained_entries
        << ",\"maximum_dot_bound\":\""
        << al_binary128_text(left_certificate.maximum_dot_bound)
        << "\",\"maximum_exact_error\":\""
        << al_binary128_text(left_certificate.maximum_exact_error) << "\"}"
        << ",\"solver_executions\":" << solver_executions
        << ",\"first_correction_applies\":" << capture.replacements
        << ",\"second_correction_applies\":0"
        << ",\"exact_right_entries\":4225,\"exact_left_entries\":4225"
        << ",\"dot2_right_entries\":" << right_certificate.dots
        << ",\"dot2_left_entries\":" << left_certificate.dots
        << ",\"additional_factorizations\":0"
        << ",\"additional_inverse_columns\":0,\"additional_solves\":0"
        << ",\"nnqp_decisions\":0,\"counterflow_replays\":0"
        << ",\"new_trials\":0,\"state_updates\":0"
        << ",\"parameter_changes\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
