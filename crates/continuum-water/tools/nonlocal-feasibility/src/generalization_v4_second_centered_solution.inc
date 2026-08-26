namespace {

constexpr const char* AL_R20_R44_SEMANTIC_SHA256 =
    "cdb334f4981ad13c0bcd5a337bd39054e21a6e76b832543141785c02c53ebc08";
constexpr const char* AL_R20_R44_RIGHT_EXACT_SHA256 =
    "9ac1d0e0d2af4679fa1cc23114c89e64f15f88db91861a320bfd4decd17dd214";
constexpr const char* AL_R20_R44_RIGHT_DOT2_SHA256 =
    "283e803a0bd01ce43e92530bbf898625df0f8cd866e38b23a3b47e522de7c691";
constexpr const char* AL_R20_R44_LEFT_EXACT_SHA256 =
    "c743e1ff5c39ad57bdd0a4a6fd1051af03e5a996d633b0ab8d4e18ba290f2d39";
constexpr const char* AL_R20_R44_LEFT_DOT2_SHA256 =
    "f2a7457cd621b0191c243686ca7c21636448b78f9711d90f803a09e0724d56b9";

} // namespace

SplitBoundaryReport run_al_generalization_v4_second_centered_solution_controls() {
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
    const bool capture_exact = parent_exact && hook_slot_clear
        && hook_slot_released && solver_executions == 1U
        && capture.attempts == 2U && capture.target_matches == 1U
        && capture.replacements == 1U && capture.second_matches == 1U
        && capture.first_certificate.root == AL_R20_R43_CERTIFICATE_SHA256
        && capture.second_material_root == AL_R20_R43_SECOND_MATERIAL_SHA256
        && torsion.root == AL_R20_R43_CASE_SHA256
        && !torsion.diagnostic.steps.empty()
        && torsion.diagnostic.steps.back().root == AL_R20_R43_STEP_SHA256;
    const ALR20R37ExactResidual right_oracle = capture_exact
        ? al_r20_r37_exact_residual(capture.matrix, capture.inverse, 65U)
        : ALR20R37ExactResidual{};
    const ALR20R38MatrixAudit right_certificate = right_oracle.exact
        ? al_r20_r38_matrix_audit(
            capture.matrix, capture.inverse, right_oracle)
        : ALR20R38MatrixAudit{};
    const ALR20R37ExactResidual left_oracle = capture_exact
        ? al_r20_r37_exact_residual(capture.inverse, capture.matrix, 65U)
        : ALR20R37ExactResidual{};
    const ALR20R38MatrixAudit left_certificate = left_oracle.exact
        ? al_r20_r38_matrix_audit(
            capture.inverse, capture.matrix, left_oracle)
        : ALR20R38MatrixAudit{};
    const bool r44_exact = right_oracle.root
            == AL_R20_R44_RIGHT_EXACT_SHA256
        && right_certificate.root == AL_R20_R44_RIGHT_DOT2_SHA256
        && left_oracle.root == AL_R20_R44_LEFT_EXACT_SHA256
        && left_certificate.root == AL_R20_R44_LEFT_DOT2_SHA256;
    const ALR20R39ExactVectorResidual residual_oracle = r44_exact
        ? al_r20_r39_exact_vector_residual(
            capture.matrix, capture.rhs, capture.solution)
        : ALR20R39ExactVectorResidual{};
    const ALR20R40ResidualSamples residual_samples = residual_oracle.exact
        ? al_r20_r40_residual_samples(capture.matrix, capture.rhs,
            capture.solution, residual_oracle)
        : ALR20R40ResidualSamples{};
    const ALR20R40ExactCorrection correction_oracle = residual_samples.exact
        ? al_r20_r40_exact_correction(capture.inverse, residual_oracle)
        : ALR20R40ExactCorrection{};
    const ALR20R41IntervalMatrix left = r44_exact
        ? al_r20_r41_left_matrix(
            capture.inverse, capture.matrix, left_oracle)
        : ALR20R41IntervalMatrix{};
    const ALR20R41IntervalVector z = correction_oracle.exact
        ? al_r20_r41_z_vector(
            capture.inverse, residual_samples, correction_oracle)
        : ALR20R41IntervalVector{};

    Binary128 inherited_error = static_cast<Binary128>(0.0);
    const ALR20NNQPSolve& final_solve =
        torsion.diagnostic.steps.back().solve;
    if (!final_solve.inverse_audits.empty())
        inherited_error = final_solve.inverse_audits.back().cheap_error;
    std::vector<Binary128> center(65U, static_cast<Binary128>(0.0));
    std::vector<ALR20R41Checkpoint> checkpoints;
    std::size_t generation_dots = 0U;
    bool generation_exact = r44_exact && residual_samples.exact
        && correction_oracle.exact && left.exact && z.exact
        && finiteq(inherited_error) != 0;
    bool generation_no_underflow = true;
    std::ostringstream generation_material;
    for (std::size_t depth = 1U; generation_exact && depth <= 32U; ++depth) {
        std::vector<Binary128> next(center.size());
        for (std::size_t row = 0U; row < center.size(); ++row) {
            std::vector<Binary128> lhs(center.size() + 1U);
            std::vector<Binary128> rhs(center.size() + 1U);
            lhs[0U] = z.value[row];
            rhs[0U] = static_cast<Binary128>(1.0);
            for (std::size_t column = 0U; column < center.size(); ++column) {
                lhs[column + 1U] = left.value[row * center.size() + column];
                rhs[column + 1U] = center[column];
            }
            const ALR20R38Dot2 dot = al_r20_r38_dot2(lhs, rhs);
            next[row] = dot.value;
            ++generation_dots;
            generation_exact = generation_exact && dot.exact
                && finiteq(dot.value) != 0 && finiteq(dot.bound) != 0;
            generation_no_underflow = generation_no_underflow
                && dot.no_underflow;
            generation_material << depth << ':' << row << ':'
                << dot.root << ';';
        }
        center = std::move(next);
        if (al_r20_r41_checkpoint_depth(depth))
            checkpoints.push_back(al_r20_r41_checkpoint(depth, center,
                capture.solution, inherited_error, left, left_oracle,
                z, correction_oracle));
    }
    const std::string generation_root =
        sha256_hex(generation_material.str());
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    bool checkpoints_exact = checkpoints.size() == AL_R20_R41_DEPTHS.size();
    bool checkpoints_no_underflow = true;
    bool checkpoints_contained = true;
    std::size_t first_resolved_depth = 0U;
    std::size_t total_residual_dots = 0U;
    std::size_t total_center_dots = 0U;
    std::ostringstream checkpoint_material;
    for (const ALR20R41Checkpoint& checkpoint : checkpoints) {
        checkpoints_exact = checkpoints_exact && checkpoint.exact;
        checkpoints_no_underflow = checkpoints_no_underflow
            && checkpoint.no_underflow;
        checkpoints_contained = checkpoints_contained
            && checkpoint.contained;
        total_residual_dots += checkpoint.residual_dots;
        total_center_dots += checkpoint.center_dots;
        if (first_resolved_depth == 0U && checkpoint.signs_resolved)
            first_resolved_depth = checkpoint.depth;
        checkpoint_material << checkpoint.root << ';';
    }
    const std::string checkpoint_root =
        sha256_hex(checkpoint_material.str());
    const ALR20R41Checkpoint final_checkpoint = checkpoints.empty()
        ? ALR20R41Checkpoint{} : checkpoints.back();
    const bool apparatus_exact = capture_exact && r44_exact
        && residual_oracle.exact && residual_samples.exact
        && correction_oracle.exact && left.exact && z.exact
        && generation_exact && checkpoints_exact;
    const bool exact = apparatus_exact && lifecycle_exact;
    const std::string route = !parent_exact || !capture_exact || !r44_exact
        ? "SECOND_CENTER_PARENT_REJECTED"
        : (capture.second_matches != 1U
            ? "SECOND_CENTER_CAPTURE_REJECTED"
            : (!apparatus_exact
                ? "SECOND_CENTER_APPARATUS_REJECTED"
                : (!right_certificate.no_underflow
                        || !left_certificate.no_underflow
                        || !residual_samples.no_underflow
                        || !left.no_underflow || !z.no_underflow
                        || !generation_no_underflow
                        || !checkpoints_no_underflow
                    ? "SECOND_CENTER_UNDERFLOW_REJECTED"
                    : (!right_certificate.contained
                            || !left_certificate.contained
                            || !residual_samples.contained
                            || !left.contained || !z.contained
                            || !checkpoints_contained
                        ? "SECOND_CENTER_CONTAINMENT_REJECTED"
                        : (!left.contractive
                            ? "SECOND_CENTER_LEFT_NONCONTRACTIVE"
                            : (!final_checkpoint.signs_resolved
                                ? "SECOND_CENTER_SIGN_UNRESOLVED"
                                : "SECOND_CENTER_SOLUTION_CANDIDATE"))))));
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r45-second-center.v1|"
        << AL_R20_R44_SEMANTIC_SHA256 << ':' << route << ':'
        << residual_oracle.root << ':' << residual_samples.root << ':'
        << correction_oracle.root << ':' << left.root << ':' << z.root << ':'
        << generation_root << ':' << checkpoint_root;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream checkpoint_report;
    checkpoint_report << '[';
    for (std::size_t index = 0U; index < checkpoints.size(); ++index) {
        if (index != 0U) checkpoint_report << ',';
        const ALR20R41Checkpoint& checkpoint = checkpoints[index];
        checkpoint_report << "{\"depth\":" << checkpoint.depth
            << ",\"root\":\"" << checkpoint.root
            << "\",\"contained\":"
            << (checkpoint.contained ? "true" : "false")
            << ",\"no_underflow\":"
            << (checkpoint.no_underflow ? "true" : "false")
            << ",\"g_inf_bound\":\""
            << al_binary128_text(checkpoint.g_inf_bound)
            << "\",\"radius\":\"" << al_binary128_text(checkpoint.radius)
            << "\",\"positive\":" << checkpoint.positive
            << ",\"negative\":" << checkpoint.negative
            << ",\"unresolved\":" << checkpoint.unresolved
            << ",\"minimum_separation\":\""
            << al_binary128_text(checkpoint.minimum_separation)
            << "\",\"worst_row\":" << checkpoint.worst_row
            << ",\"maximum_residual_bound\":\""
            << al_binary128_text(checkpoint.maximum_residual_bound)
            << "\",\"maximum_residual_exact_error\":\""
            << al_binary128_text(checkpoint.maximum_residual_exact_error)
            << "\"}";
    }
    checkpoint_report << ']';
    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r45_second_center.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r44_sha256\":\""
        << AL_R20_R44_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"capture_exact\":" << (capture_exact ? "true" : "false")
        << ",\"r44_exact\":" << (r44_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"residual_exact_root\":\"" << residual_oracle.root
        << "\",\"residual_samples_root\":\"" << residual_samples.root
        << "\",\"correction_root\":\"" << correction_oracle.root
        << "\",\"left_root\":\"" << left.root
        << "\",\"z_root\":\"" << z.root
        << "\",\"generation_root\":\"" << generation_root
        << "\",\"checkpoint_root\":\"" << checkpoint_root
        << "\",\"first_resolved_depth\":" << first_resolved_depth
        << ",\"checkpoints\":" << checkpoint_report.str()
        << ",\"solver_executions\":" << solver_executions
        << ",\"first_correction_applies\":" << capture.replacements
        << ",\"second_correction_applies\":0"
        << ",\"solution_residual_dots\":" << residual_samples.dimension
        << ",\"z_dots\":" << z.dots
        << ",\"center_generation_dots\":" << generation_dots
        << ",\"checkpoint_residual_dots\":" << total_residual_dots
        << ",\"checkpoint_center_dots\":" << total_center_dots
        << ",\"additional_factorizations\":0"
        << ",\"additional_inverse_columns\":0,\"additional_solves\":0"
        << ",\"nnqp_decisions\":0,\"counterflow_replays\":0"
        << ",\"new_trials\":0,\"state_updates\":0"
        << ",\"parameter_changes\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
