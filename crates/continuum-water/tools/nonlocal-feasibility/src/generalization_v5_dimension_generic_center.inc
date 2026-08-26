namespace {

constexpr const char* AL_R20_R59_SEMANTIC_SHA256 =
    "a0881eaa583ed3ed05e9a743957d008db4a45c0c9100f0dd94e458ff9051faa9";
constexpr const char* AL_R20_R60_PROBLEM_SHA256 =
    "5f9c2b59d28c5f4de6f30a0341e76edf01bb3462fe574b55f687b5911f0cd54c";
constexpr const char* AL_R20_R60_CASE_SHA256 =
    "0337d014a68fac594161b90817e534313671008cdc3fc0deb9a16d619e3d6644";

struct ALR20R60Capture {
    std::size_t calls = 0U;
    std::size_t dimension = 0U;
    std::vector<Binary128> matrix;
    std::vector<Binary128> inverse;
    std::vector<Binary128> rhs;
    std::vector<Binary128> solution;
    std::string matrix_root;
    std::string inverse_root;
    std::string rhs_root;
    std::string solution_root;
    std::string material_root;
    std::string rejection_root;
};

thread_local ALR20R60Capture* al_r20_r60_capture = nullptr;

ALR20VerifiedSolutionRefinement al_r20_r60_capture_refiner(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& inverse,
    const std::vector<Binary128>& rhs,
    const std::vector<Binary128>& solution) {
    ALR20VerifiedSolutionRefinement result;
    ALR20R60Capture* capture = al_r20_r60_capture;
    if (capture == nullptr) return result;
    ++capture->calls;
    const std::string matrix_root = al_r20_q_scalar_root(matrix);
    const std::string inverse_root = al_r20_q_scalar_root(inverse);
    const std::string rhs_root = al_r20_q_scalar_root(rhs);
    const std::string solution_root = al_r20_q_scalar_root(solution);
    const std::string material_root = sha256_hex(matrix_root + ':'
        + inverse_root + ':' + rhs_root + ':' + solution_root);
    if (capture->calls == 1U) {
        capture->dimension = rhs.size();
        capture->matrix = matrix;
        capture->inverse = inverse;
        capture->rhs = rhs;
        capture->solution = solution;
        capture->matrix_root = matrix_root;
        capture->inverse_root = inverse_root;
        capture->rhs_root = rhs_root;
        capture->solution_root = solution_root;
        capture->material_root = material_root;
    }
    result.exact = true;
    result.root = sha256_hex(std::string("r50-dimension-rejected:")
        + material_root + ':' + std::to_string(rhs.size()));
    if (capture->calls == 1U) capture->rejection_root = result.root;
    return result;
}

std::size_t al_r20_r60_certificate_dots(
    const ALR20R42PracticalCertificate& certificate) {
    return certificate.left_dots + certificate.residual_dots
        + certificate.z_dots + certificate.center_dots
        + certificate.certificate_dots + certificate.sum_dots;
}

std::size_t al_r20_r60_certificate_pairs(
    const ALR20R42PracticalCertificate& certificate) {
    const std::size_t dimension = certificate.dimension;
    return certificate.left_dots * (dimension + 1U)
        + certificate.residual_dots * (dimension + 1U)
        + certificate.z_dots * dimension
        + certificate.center_dots * (dimension + 1U)
        + certificate.certificate_dots * (dimension + 2U)
        + certificate.sum_dots * 2U;
}

std::string al_r20_r60_control_root(
    const ALR20R37ExactResidual& good_right,
    const ALR20R38MatrixAudit& good_right_dot2,
    const ALR20R37ExactResidual& good_left,
    const ALR20R38MatrixAudit& good_left_dot2,
    const ALR20R42PracticalCertificate& good_center,
    const ALR20R37ExactResidual& bad_right,
    const ALR20R38MatrixAudit& bad_right_dot2,
    const ALR20R42PracticalCertificate& bad_center) {
    return sha256_hex(good_right.root + ':' + good_right_dot2.root + ':'
        + good_left.root + ':' + good_left_dot2.root + ':'
        + good_center.root + ':' + bad_right.root + ':'
        + bad_right_dot2.root + ':' + bad_center.root);
}

} // namespace

SplitBoundaryReport run_al_generalization_v5_dimension_generic_center_controls() {
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
    ALR20ProblemData problem;
    bool problem_materialized = false;
    if (parent_exact) {
        problem = al_r20_materialize_problem(
            cases[3U], &sparse_work, &static_work, &adjacency_work);
        problem_materialized = true;
        parent_exact = problem.exact && problem.source_positive == 0U
            && problem.problem_root == AL_R20_R60_PROBLEM_SHA256;
    }

    ALR20R60Capture capture;
    ALR20R55SlopeTrace slope;
    ALR20R35CaseAudit oblique;
    std::size_t solver_executions = 0U;
    const bool hook_slot_clear = al_r20_verified_solution_refiner == nullptr
        && al_r20_r60_capture == nullptr
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr;
    if (parent_exact && hook_slot_clear) {
        al_r20_r60_capture = &capture;
        al_r20_verified_solution_refiner = &al_r20_r60_capture_refiner;
        al_r20_r55_slope_trace = &slope;
        al_r20_slope_error_refiner = &al_r20_r55_slope_refiner;
        oblique = al_r20_r35_case(problem);
        al_r20_slope_error_refiner = nullptr;
        al_r20_r55_slope_trace = nullptr;
        al_r20_verified_solution_refiner = nullptr;
        al_r20_r60_capture = nullptr;
        solver_executions = 1U;
    }
    const bool hook_slot_released =
        al_r20_verified_solution_refiner == nullptr
        && al_r20_r60_capture == nullptr
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr
        && al_r20_verified_inverse_capture == nullptr
        && al_r20_direct_solution_capture == nullptr;
    const bool trajectory_exact = parent_exact && hook_slot_clear
        && hook_slot_released && solver_executions == 1U
        && al_r20_r35_case_observation_exact(oblique)
        && oblique.root == AL_R20_R60_CASE_SHA256
        && oblique.route == "ACTIVE_SET_REJECTED"
        && !oblique.diagnostic.certified
        && oblique.diagnostic.accepted_iterations == 8U
        && oblique.diagnostic.total_principal_solves == 1092U
        && oblique.diagnostic.total_transitions == 1092U
        && capture.calls == 1U && capture.dimension != 0U
        && capture.dimension != AL_R20_R50_DIMENSION
        && capture.matrix.size() == capture.dimension * capture.dimension
        && capture.inverse.size() == capture.matrix.size()
        && capture.rhs.size() == capture.dimension
        && capture.solution.size() == capture.dimension
        && slope.calls == 0U && slope.inverse_audits == 0U
        && slope.cap_hits == 0U && slope.inverse_columns == 0U;

    const std::vector<Binary128> good_matrix{
        static_cast<Binary128>(2.0)};
    const std::vector<Binary128> good_inverse{
        static_cast<Binary128>(0.5)};
    const std::vector<Binary128> bad_inverse{
        static_cast<Binary128>(0.0)};
    const std::vector<Binary128> control_rhs{
        static_cast<Binary128>(2.0)};
    const std::vector<Binary128> control_solution{
        static_cast<Binary128>(1.0)};
    const ALR20R37ExactResidual good_right =
        al_r20_r37_exact_residual(good_matrix, good_inverse, 1U);
    const ALR20R38MatrixAudit good_right_dot2 =
        al_r20_r38_matrix_audit(good_matrix, good_inverse, good_right);
    const ALR20R37ExactResidual good_left =
        al_r20_r37_exact_residual(good_inverse, good_matrix, 1U);
    const ALR20R38MatrixAudit good_left_dot2 =
        al_r20_r38_matrix_audit(good_inverse, good_matrix, good_left);
    const ALR20R42PracticalCertificate good_center =
        al_r20_r42_practical_certificate(good_matrix, good_inverse,
            control_rhs, control_solution, 4U, false);
    const ALR20R37ExactResidual bad_right =
        al_r20_r37_exact_residual(good_matrix, bad_inverse, 1U);
    const ALR20R38MatrixAudit bad_right_dot2 =
        al_r20_r38_matrix_audit(good_matrix, bad_inverse, bad_right);
    const ALR20R42PracticalCertificate bad_center =
        al_r20_r42_practical_certificate(good_matrix, bad_inverse,
            control_rhs, control_solution, 4U, false);
    const bool witness_controls_exact = good_right.exact
        && good_right_dot2.exact && good_right_dot2.contained
        && good_right_dot2.contractive
        && good_left.exact && good_left_dot2.exact
        && good_left_dot2.contained && good_left_dot2.contractive
        && good_center.exact && good_center.no_underflow
        && good_center.contractive && good_center.passed
        && good_center.positive == 1U && good_center.unresolved == 0U
        && bad_right.exact && bad_right_dot2.exact
        && bad_right_dot2.contained && !bad_right_dot2.contractive
        && bad_right_dot2.rho_bound >= static_cast<Binary128>(1.0)
        && bad_center.exact && bad_center.no_underflow
        && !bad_center.contractive && !bad_center.passed
        && bad_center.unresolved == 1U;
    const std::string control_root = al_r20_r60_control_root(
        good_right, good_right_dot2, good_left, good_left_dot2, good_center,
        bad_right, bad_right_dot2, bad_center);

    const ALR20R37ExactResidual right_oracle = trajectory_exact
        ? al_r20_r37_exact_residual(
            capture.matrix, capture.inverse, capture.dimension)
        : ALR20R37ExactResidual{};
    const ALR20R38MatrixAudit right_dot2 = right_oracle.exact
        ? al_r20_r38_matrix_audit(
            capture.matrix, capture.inverse, right_oracle)
        : ALR20R38MatrixAudit{};
    const ALR20R37ExactResidual left_oracle = trajectory_exact
        ? al_r20_r37_exact_residual(
            capture.inverse, capture.matrix, capture.dimension)
        : ALR20R37ExactResidual{};
    const ALR20R38MatrixAudit left_dot2 = left_oracle.exact
        ? al_r20_r38_matrix_audit(
            capture.inverse, capture.matrix, left_oracle)
        : ALR20R38MatrixAudit{};
    const ALR20R42PracticalCertificate depth4 = trajectory_exact
        ? al_r20_r42_practical_certificate(capture.matrix, capture.inverse,
            capture.rhs, capture.solution, 4U, false)
        : ALR20R42PracticalCertificate{};
    const ALR20R42PracticalCertificate depth8 = trajectory_exact
        ? al_r20_r42_practical_certificate(capture.matrix, capture.inverse,
            capture.rhs, capture.solution, 8U, false)
        : ALR20R42PracticalCertificate{};
    const ALR20R42PracticalCertificate depth16 = trajectory_exact
        ? al_r20_r42_practical_certificate(capture.matrix, capture.inverse,
            capture.rhs, capture.solution, 16U, false)
        : ALR20R42PracticalCertificate{};

    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const std::size_t dimension = capture.dimension;
    const bool oracle_exact = right_oracle.exact && right_dot2.exact
        && left_oracle.exact && left_dot2.exact
        && right_oracle.dimension == dimension
        && left_oracle.dimension == dimension
        && right_dot2.dots == dimension * dimension
        && left_dot2.dots == dimension * dimension;
    const bool center_exact = depth4.exact && depth8.exact && depth16.exact
        && depth4.dimension == dimension && depth8.dimension == dimension
        && depth16.dimension == dimension;
    const bool work_exact = center_exact
        && al_r20_r60_certificate_dots(depth4)
            == dimension * dimension + 8U * dimension
        && al_r20_r60_certificate_dots(depth8)
            == dimension * dimension + 12U * dimension
        && al_r20_r60_certificate_dots(depth16)
            == dimension * dimension + 20U * dimension
        && al_r20_r60_certificate_pairs(depth4)
            == dimension * dimension * (dimension + 1U)
                + dimension * (dimension + 1U)
                + dimension * dimension
                + 4U * dimension * (dimension + 1U)
                + dimension * (dimension + 2U) + 2U * dimension
        && al_r20_r60_certificate_pairs(depth8)
            == dimension * dimension * (dimension + 1U)
                + dimension * (dimension + 1U)
                + dimension * dimension
                + 8U * dimension * (dimension + 1U)
                + dimension * (dimension + 2U) + 2U * dimension
        && al_r20_r60_certificate_pairs(depth16)
            == dimension * dimension * (dimension + 1U)
                + dimension * (dimension + 1U)
                + dimension * dimension
                + 16U * dimension * (dimension + 1U)
                + dimension * (dimension + 2U) + 2U * dimension;
    const bool containment_exact = oracle_exact
        && right_dot2.contained && left_dot2.contained
        && right_dot2.contained_entries == dimension * dimension
        && left_dot2.contained_entries == dimension * dimension;
    const bool apparatus_exact = trajectory_exact && witness_controls_exact
        && oracle_exact && center_exact && work_exact && containment_exact
        && lifecycle_exact;
    const bool underflow_free = right_dot2.no_underflow
        && left_dot2.no_underflow && depth4.no_underflow
        && depth8.no_underflow && depth16.no_underflow;
    const bool signs_resolved = depth16.passed
        && depth16.positive + depth16.negative == dimension
        && depth16.unresolved == 0U
        && depth16.minimum_separation > static_cast<Binary128>(0.0);
    const std::string route = !parent_exact || !trajectory_exact
        ? "DIMENSION_GENERIC_CENTER_PARENT_REJECTED"
        : (!witness_controls_exact
            ? "DIMENSION_GENERIC_CENTER_CONTROL_REJECTED"
            : (!oracle_exact || !center_exact || !work_exact
                || !containment_exact || !lifecycle_exact
                ? "DIMENSION_GENERIC_CENTER_APPARATUS_REJECTED"
                : (!underflow_free
                    ? "DIMENSION_GENERIC_CENTER_UNDERFLOW_BOUNDARY"
                    : (!right_dot2.contractive
                        ? "DIMENSION_GENERIC_CENTER_RIGHT_NONCONTRACTIVE"
                        : (!left_dot2.contractive
                            ? "DIMENSION_GENERIC_CENTER_LEFT_NONCONTRACTIVE"
                            : (!signs_resolved
                                ? "DIMENSION_GENERIC_CENTER_SIGN_BOUNDARY"
                                : "DIMENSION_GENERIC_CENTER_CANDIDATE"))))));
    const bool exact = apparatus_exact;
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r60-generic-center.v1|"
        << AL_R20_R59_SEMANTIC_SHA256 << ':' << route << ':'
        << oblique.root << ':' << capture.material_root << ':'
        << control_root << ':'
        << right_oracle.root << ':' << right_dot2.root << ':'
        << left_oracle.root << ':' << left_dot2.root << ':'
        << depth4.root << ':' << depth8.root << ':' << depth16.root;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r60_generic_center.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r59_sha256\":\"" << AL_R20_R59_SEMANTIC_SHA256
        << "\",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"trajectory_exact\":"
        << (trajectory_exact ? "true" : "false")
        << ",\"parent_case_root\":\"" << oblique.root
        << "\",\"parent_case_route\":\"" << oblique.route << "\""
        << ",\"accepted_iterations\":"
        << oblique.diagnostic.accepted_iterations
        << ",\"principal_solves\":"
        << oblique.diagnostic.total_principal_solves
        << ",\"transitions\":" << oblique.diagnostic.total_transitions
        << ",\"witness_controls_exact\":"
        << (witness_controls_exact ? "true" : "false")
        << ",\"control_root\":\"" << control_root << "\""
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"capture\":{\"calls\":" << capture.calls
        << ",\"dimension\":" << dimension
        << ",\"material_root\":\"" << capture.material_root
        << "\",\"matrix_root\":\"" << capture.matrix_root
        << "\",\"inverse_root\":\"" << capture.inverse_root
        << "\",\"rhs_root\":\"" << capture.rhs_root
        << "\",\"solution_root\":\"" << capture.solution_root
        << "\",\"rejection_root\":\"" << capture.rejection_root << "\"}"
        << ",\"right\":{\"exact_root\":\"" << right_oracle.root
        << "\",\"dot2_root\":\"" << right_dot2.root
        << "\",\"contained\":" << (right_dot2.contained ? "true" : "false")
        << ",\"contractive\":"
        << (right_dot2.contractive ? "true" : "false")
        << ",\"exact_rho_outward\":\""
        << al_binary128_text(right_oracle.outward_norm)
        << "\",\"rho_bound\":\""
        << al_binary128_text(right_dot2.rho_bound)
        << "\",\"contained_entries\":" << right_dot2.contained_entries
        << ",\"dots\":" << right_dot2.dots << "}"
        << ",\"left\":{\"exact_root\":\"" << left_oracle.root
        << "\",\"dot2_root\":\"" << left_dot2.root
        << "\",\"contained\":" << (left_dot2.contained ? "true" : "false")
        << ",\"contractive\":"
        << (left_dot2.contractive ? "true" : "false")
        << ",\"exact_rho_outward\":\""
        << al_binary128_text(left_oracle.outward_norm)
        << "\",\"rho_bound\":\""
        << al_binary128_text(left_dot2.rho_bound)
        << "\",\"contained_entries\":" << left_dot2.contained_entries
        << ",\"dots\":" << left_dot2.dots << "}"
        << ",\"depths\":[";
    const std::array<const ALR20R42PracticalCertificate*, 3U> depths{{
        &depth4, &depth8, &depth16}};
    const std::array<std::size_t, 3U> depth_values{{4U, 8U, 16U}};
    for (std::size_t index = 0U; index < depths.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20R42PracticalCertificate& certificate = *depths[index];
        report << "{\"depth\":" << depth_values[index]
            << ",\"exact\":" << (certificate.exact ? "true" : "false")
            << ",\"passed\":" << (certificate.passed ? "true" : "false")
            << ",\"no_underflow\":"
            << (certificate.no_underflow ? "true" : "false")
            << ",\"contractive\":"
            << (certificate.contractive ? "true" : "false")
            << ",\"positive\":" << certificate.positive
            << ",\"negative\":" << certificate.negative
            << ",\"unresolved\":" << certificate.unresolved
            << ",\"rho_bound\":\""
            << al_binary128_text(certificate.rho_bound)
            << "\",\"radius\":\"" << al_binary128_text(certificate.radius)
            << "\",\"error\":\"" << al_binary128_text(certificate.error)
            << "\",\"minimum_separation\":\""
            << al_binary128_text(certificate.minimum_separation)
            << "\",\"dots\":" << al_r20_r60_certificate_dots(certificate)
            << ",\"input_pairs\":"
            << al_r20_r60_certificate_pairs(certificate)
            << ",\"root\":\"" << certificate.root << "\"}";
    }
    report << ']'
        << ",\"oracle_exact\":" << (oracle_exact ? "true" : "false")
        << ",\"center_exact\":" << (center_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"containment_exact\":"
        << (containment_exact ? "true" : "false")
        << ",\"underflow_free\":"
        << (underflow_free ? "true" : "false")
        << ",\"signs_resolved\":"
        << (signs_resolved ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"slope_calls\":" << slope.calls
        << ",\"replacement_applies\":0,\"new_trials\":0"
        << ",\"state_updates\":0,\"parameter_changes\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
