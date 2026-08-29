namespace {

constexpr const char* AL_R20_R38_SEMANTIC_SHA256 =
    "6898dcbe1b610a9a4cf38276d5af4e5f79ee63fe095315ddafb86b6deaa706ae";
constexpr const char* AL_R20_R38_DOT_SHA256 =
    "d388080b0ae9a8c8026e43856f4fb1c6abe8d492e4a3f393d22ef9271ad030f8";
constexpr const char* AL_R20_R38_CERTIFICATE_SHA256 =
    "cd14942dd78c3397568c663137daeecbc1cf89b0fe68666cd2ac1246c2323181";

struct ALR20R39ExactVectorResidual {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t worst_row = 0U;
    ALR20R37Dyadic norm;
    Binary128 outward_norm = static_cast<Binary128>(0.0);
    std::vector<ALR20R37Dyadic> residual;
    std::string root;
};

ALR20R39ExactVectorResidual al_r20_r39_exact_vector_residual(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& rhs,
    const std::vector<Binary128>& solution) {
    ALR20R39ExactVectorResidual result;
    result.dimension = rhs.size();
    const std::size_t dimension = result.dimension;
    if (dimension == 0U || solution.size() != dimension
        || matrix.size() != dimension * dimension)
        return result;
    std::vector<ALR20R37Dyadic> matrix_exact(matrix.size());
    std::vector<ALR20R37Dyadic> rhs_exact(dimension);
    std::vector<ALR20R37Dyadic> solution_exact(dimension);
    bool operands_exact = true;
    for (std::size_t index = 0U; index < matrix.size(); ++index) {
        const ALR20R37Conversion converted =
            al_r20_r37_from_binary128(matrix[index]);
        operands_exact = operands_exact && converted.exact;
        matrix_exact[index] = converted.value;
    }
    for (std::size_t index = 0U; index < dimension; ++index) {
        const ALR20R37Conversion rhs_value =
            al_r20_r37_from_binary128(rhs[index]);
        const ALR20R37Conversion solution_value =
            al_r20_r37_from_binary128(solution[index]);
        operands_exact = operands_exact
            && rhs_value.exact && solution_value.exact;
        rhs_exact[index] = rhs_value.value;
        solution_exact[index] = solution_value.value;
    }
    if (!operands_exact) return result;
    std::ostringstream material;
    for (std::size_t row = 0U; row < dimension; ++row) {
        ALR20R37Dyadic image;
        for (std::size_t column = 0U; column < dimension; ++column)
            image = al_r20_r37_add(image, al_r20_r37_multiply(
                matrix_exact[row * dimension + column],
                solution_exact[column]));
        const ALR20R37Dyadic residual = al_r20_r37_add(
            rhs_exact[row], {-image.numerator, image.exponent});
        result.residual.push_back(residual);
        const ALR20R37Dyadic absolute = al_r20_r37_absolute(residual);
        if (row == 0U
            || al_r20_r37_compare_positive(absolute, result.norm) > 0) {
            result.norm = absolute;
            result.worst_row = row;
        }
        material << al_r20_r37_dyadic_text(residual) << ';';
    }
    result.outward_norm = al_r20_r37_binary128_up(result.norm);
    const ALR20R37Conversion outward =
        al_r20_r37_from_binary128(result.outward_norm);
    result.exact = outward.exact
        && al_r20_r37_compare_positive(outward.value, result.norm) >= 0;
    material << '|' << result.exact << ':' << result.worst_row << ':'
        << al_r20_r37_dyadic_text(result.norm) << ':'
        << al_binary128_hex(result.outward_norm);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R39SolutionAudit {
    bool exact = false;
    bool no_underflow = false;
    bool contained = false;
    bool signs_resolved = false;
    std::size_t dimension = 0U;
    std::size_t dots = 0U;
    std::size_t contained_entries = 0U;
    std::size_t positive = 0U;
    std::size_t negative = 0U;
    std::size_t unresolved = 0U;
    std::size_t worst_residual_row = 0U;
    std::size_t minimum_component_row = 0U;
    Binary128 inverse_norm = static_cast<Binary128>(0.0);
    Binary128 inverse_norm_bound = static_cast<Binary128>(0.0);
    Binary128 residual_inf_bound = static_cast<Binary128>(0.0);
    Binary128 solution_error_bound = static_cast<Binary128>(0.0);
    Binary128 minimum_absolute_component = HUGE_VALQ;
    Binary128 minimum_separation = HUGE_VALQ;
    Binary128 maximum_dot_bound = static_cast<Binary128>(0.0);
    Binary128 maximum_exact_error = static_cast<Binary128>(0.0);
    Binary128 maximum_error_to_bound = static_cast<Binary128>(0.0);
    std::string dot_root;
    std::string root;
};

ALR20R39SolutionAudit al_r20_r39_solution_audit(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& inverse,
    const std::vector<Binary128>& rhs,
    const std::vector<Binary128>& solution,
    const ALR20R38MatrixAudit& inverse_certificate,
    const ALR20R39ExactVectorResidual& oracle) {
    ALR20R39SolutionAudit result;
    result.dimension = rhs.size();
    const std::size_t dimension = result.dimension;
    if (!inverse_certificate.exact || !inverse_certificate.no_underflow
        || !inverse_certificate.contained || !inverse_certificate.contractive
        || !oracle.exact || oracle.residual.size() != dimension
        || solution.size() != dimension || inverse.size() != dimension * dimension
        || matrix.size() != dimension * dimension)
        return result;
    for (std::size_t row = 0U; row < dimension; ++row) {
        Binary128 row_norm = static_cast<Binary128>(0.0);
        for (std::size_t column = 0U; column < dimension; ++column)
            row_norm = al_r20_r38_up_add(
                row_norm, fabsq(inverse[row * dimension + column]));
        result.inverse_norm = fmaxq(result.inverse_norm, row_norm);
    }
    const Binary128 denominator = nextafterq(
        static_cast<Binary128>(1.0) - inverse_certificate.rho_bound,
        -HUGE_VALQ);
    result.inverse_norm_bound = al_r20_r38_up_divide(
        result.inverse_norm, denominator);
    bool no_underflow = true;
    bool contained = true;
    std::ostringstream dot_material;
    for (std::size_t row = 0U; row < dimension; ++row) {
        std::vector<Binary128> left(dimension + 1U);
        std::vector<Binary128> right(dimension + 1U);
        left[0U] = rhs[row];
        right[0U] = static_cast<Binary128>(1.0);
        for (std::size_t column = 0U; column < dimension; ++column) {
            left[column + 1U] = matrix[row * dimension + column];
            right[column + 1U] = -solution[column];
        }
        const ALR20R38Dot2 dot = al_r20_r38_dot2(left, right);
        Binary128 exact_error = static_cast<Binary128>(0.0);
        const bool entry_contained = al_r20_r38_dot_contains(
            dot, oracle.residual[row], &exact_error);
        ++result.dots;
        result.contained_entries += entry_contained ? 1U : 0U;
        no_underflow = no_underflow && dot.no_underflow;
        contained = contained && entry_contained;
        result.maximum_dot_bound = fmaxq(
            result.maximum_dot_bound, dot.bound);
        result.maximum_exact_error = fmaxq(
            result.maximum_exact_error, exact_error);
        if (dot.bound > static_cast<Binary128>(0.0))
            result.maximum_error_to_bound = fmaxq(
                result.maximum_error_to_bound, exact_error / dot.bound);
        const Binary128 residual_bound = al_r20_r38_up_add(
            fabsq(dot.value), dot.bound);
        if (row == 0U || residual_bound > result.residual_inf_bound) {
            result.residual_inf_bound = residual_bound;
            result.worst_residual_row = row;
        }
        dot_material << dot.root << ':' << entry_contained << ';';
    }
    result.solution_error_bound = al_r20_r38_up_multiply(
        result.inverse_norm_bound, result.residual_inf_bound);
    for (std::size_t row = 0U; row < dimension; ++row) {
        const Binary128 absolute = fabsq(solution[row]);
        if (row == 0U || absolute < result.minimum_absolute_component) {
            result.minimum_absolute_component = absolute;
            result.minimum_component_row = row;
        }
        result.minimum_separation = fminq(result.minimum_separation,
            absolute - result.solution_error_bound);
        if (solution[row] + result.solution_error_bound
                < static_cast<Binary128>(0.0))
            ++result.negative;
        else if (solution[row] - result.solution_error_bound
                > static_cast<Binary128>(0.0))
            ++result.positive;
        else
            ++result.unresolved;
    }
    result.no_underflow = no_underflow;
    result.contained = contained
        && result.contained_entries == dimension;
    result.signs_resolved = result.unresolved == 0U
        && result.positive + result.negative == dimension;
    result.dot_root = sha256_hex(dot_material.str());
    result.exact = result.dots == dimension
        && finiteq(result.inverse_norm) != 0
        && finiteq(result.inverse_norm_bound) != 0
        && finiteq(result.residual_inf_bound) != 0
        && finiteq(result.solution_error_bound) != 0
        && finiteq(result.minimum_absolute_component) != 0
        && finiteq(result.minimum_separation) != 0
        && finiteq(result.maximum_dot_bound) != 0
        && finiteq(result.maximum_exact_error) != 0
        && finiteq(result.maximum_error_to_bound) != 0;
    std::ostringstream material;
    material << result.exact << ':' << result.no_underflow << ':'
        << result.contained << ':' << result.signs_resolved << ':'
        << dimension << ':' << result.dots << ':' << result.contained_entries
        << ':' << result.positive << ':' << result.negative << ':'
        << result.unresolved << ':' << result.worst_residual_row << ':'
        << result.minimum_component_row << ':'
        << al_binary128_hex(result.inverse_norm) << ':'
        << al_binary128_hex(result.inverse_norm_bound) << ':'
        << al_binary128_hex(result.residual_inf_bound) << ':'
        << al_binary128_hex(result.solution_error_bound) << ':'
        << al_binary128_hex(result.minimum_absolute_component) << ':'
        << al_binary128_hex(result.minimum_separation) << ':'
        << al_binary128_hex(result.maximum_dot_bound) << ':'
        << al_binary128_hex(result.maximum_exact_error) << ':'
        << al_binary128_hex(result.maximum_error_to_bound) << ':'
        << result.dot_root << ':' << oracle.root << ':'
        << inverse_certificate.root;
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v4_dot2_solution_error_controls() {
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
    ALR20VerifiedInverseCapture capture;
    capture.selected_root = AL_R20_R36_TORSION_INVERSE_SHA256;
    ALR20R35CaseAudit torsion;
    std::size_t solver_executions = 0U;
    const bool capture_slot_clear =
        al_r20_verified_inverse_capture == nullptr;
    if (parent_exact && capture_slot_clear) {
        al_r20_verified_inverse_capture = &capture;
        torsion = al_r20_r35_case(problem);
        al_r20_verified_inverse_capture = nullptr;
        solver_executions = 1U;
    }
    const std::size_t target_capture_count = static_cast<std::size_t>(
        std::count(capture.audit_roots.begin(), capture.audit_roots.end(),
            AL_R20_R36_TORSION_INVERSE_SHA256));
    ALR20ADMMFactor captured_factor;
    captured_factor.passed = capture.factor_passed;
    captured_factor.dimension = capture.audit.dimension;
    captured_factor.matrix = capture.matrix;
    captured_factor.lower = capture.lower;
    captured_factor.matrix_root = capture.matrix_root;
    captured_factor.factor_root = capture.factor_root;
    const Binary128 legacy_error = capture.factor_passed
        ? al_r20_newton_direction_error_bound(captured_factor,
            capture.solve_residual, capture.solve_bound, capture.solution)
        : HUGE_VALQ;
    const bool reproduction_exact = parent_exact && capture_slot_clear
        && torsion.root == AL_R20_R35_TORSION_CASE_SHA256
        && !torsion.diagnostic.steps.empty()
        && torsion.diagnostic.steps.back().root
            == AL_R20_R35_TORSION_STEP_SHA256
        && capture.calls == 2U && capture.audit_roots.size() == 2U
        && target_capture_count == 1U
        && capture.audit.root == AL_R20_R36_TORSION_INVERSE_SHA256
        && capture.matrix_root == AL_R20_R37_MATRIX_SHA256
        && capture.factor_root == AL_R20_R37_FACTOR_SHA256
        && al_r20_q_scalar_root(capture.inverse)
            == AL_R20_R37_INVERSE_SHA256
        && capture.rhs.size() == 65U && capture.solution.size() == 65U
        && capture.solve_residual <= capture.solve_bound
        && al_binary128_hex(capture.cheap_error)
            == al_binary128_hex(capture.audit.cheap_error)
        && al_binary128_hex(legacy_error)
            == al_binary128_hex(capture.cheap_error)
        && solver_executions == 1U;
    const ALR20R37ExactResidual inverse_oracle = reproduction_exact
        ? al_r20_r37_exact_residual(
            capture.matrix, capture.inverse, capture.audit.dimension)
        : ALR20R37ExactResidual{};
    const ALR20R38MatrixAudit inverse_certificate =
        reproduction_exact && inverse_oracle.exact
        ? al_r20_r38_matrix_audit(
            capture.matrix, capture.inverse, inverse_oracle)
        : ALR20R38MatrixAudit{};
    const bool inverse_exact = inverse_oracle.root == AL_R20_R37_AUDIT_SHA256
        && inverse_certificate.dot_root == AL_R20_R38_DOT_SHA256
        && inverse_certificate.root == AL_R20_R38_CERTIFICATE_SHA256;
    const ALR20R39ExactVectorResidual solution_oracle =
        reproduction_exact && inverse_exact
        ? al_r20_r39_exact_vector_residual(
            capture.matrix, capture.rhs, capture.solution)
        : ALR20R39ExactVectorResidual{};
    const ALR20R39SolutionAudit solution =
        solution_oracle.exact && inverse_exact
        ? al_r20_r39_solution_audit(capture.matrix, capture.inverse,
            capture.rhs, capture.solution,
            inverse_certificate, solution_oracle)
        : ALR20R39SolutionAudit{};
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool exact = parent_exact && reproduction_exact && inverse_exact
        && solution_oracle.exact && solution.exact && lifecycle_exact;
    const std::string route = !parent_exact
        ? "DOT2_SOLUTION_PARENT_REJECTED"
        : (!reproduction_exact
            ? "DOT2_SOLUTION_CAPTURE_REJECTED"
            : (!inverse_exact
                ? "DOT2_SOLUTION_PARENT_REJECTED"
                : (!solution.exact
                    ? "DOT2_SOLUTION_APPARATUS_REJECTED"
                    : (!solution.no_underflow
                        ? "DOT2_SOLUTION_UNDERFLOW_REJECTED"
                        : (!solution.contained
                            ? "DOT2_SOLUTION_CONTAINMENT_REJECTED"
                            : (solution.signs_resolved
                                ? "DOT2_SOLUTION_ERROR_CANDIDATE"
                                : "DOT2_SOLUTION_SIGN_UNRESOLVED"))))));
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r39-solution-error.v1|"
        << AL_R20_R38_SEMANTIC_SHA256 << ':' << route << ':'
        << torsion.root << ':' << capture.audit.root << ':'
        << al_r20_q_scalar_root(capture.rhs) << ':'
        << al_r20_q_scalar_root(capture.solution) << ':'
        << inverse_certificate.root << ':' << solution_oracle.root << ':'
        << solution.root;
    const std::string result_sha = sha256_hex(semantic.str());
    const Binary128 improvement = solution.solution_error_bound
            > static_cast<Binary128>(0.0)
        ? capture.cheap_error / solution.solution_error_bound : HUGE_VALQ;
    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r39_solution_error.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r38_sha256\":\""
        << AL_R20_R38_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"reproduction_exact\":"
        << (reproduction_exact ? "true" : "false")
        << ",\"inverse_exact\":" << (inverse_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"case_sha256\":\"" << torsion.root
        << "\",\"rhs_sha256\":\"" << al_r20_q_scalar_root(capture.rhs)
        << "\",\"represented_solution_sha256\":\""
        << al_r20_q_scalar_root(capture.solution)
        << "\",\"stored_solve_residual\":\""
        << al_binary128_text(capture.solve_residual)
        << "\",\"stored_solve_bound\":\""
        << al_binary128_text(capture.solve_bound)
        << "\",\"old_cheap_error\":\""
        << al_binary128_text(capture.cheap_error)
        << "\",\"inverse_certificate_sha256\":\""
        << inverse_certificate.root
        << "\",\"solution_exact_residual_sha256\":\""
        << solution_oracle.root
        << "\",\"solution\":{\"no_underflow\":"
        << (solution.no_underflow ? "true" : "false")
        << ",\"contained\":" << (solution.contained ? "true" : "false")
        << ",\"signs_resolved\":"
        << (solution.signs_resolved ? "true" : "false")
        << ",\"dots\":" << solution.dots
        << ",\"contained_entries\":" << solution.contained_entries
        << ",\"positive\":" << solution.positive
        << ",\"negative\":" << solution.negative
        << ",\"unresolved\":" << solution.unresolved
        << ",\"inverse_norm\":\""
        << al_binary128_text(solution.inverse_norm)
        << "\",\"inverse_norm_bound\":\""
        << al_binary128_text(solution.inverse_norm_bound)
        << "\",\"exact_residual_inf_outward\":\""
        << al_binary128_text(solution_oracle.outward_norm)
        << "\",\"residual_inf_bound\":\""
        << al_binary128_text(solution.residual_inf_bound)
        << "\",\"solution_error_bound\":\""
        << al_binary128_text(solution.solution_error_bound)
        << "\",\"old_to_new_error_ratio\":\""
        << al_binary128_text(improvement)
        << "\",\"minimum_absolute_component\":\""
        << al_binary128_text(solution.minimum_absolute_component)
        << "\",\"minimum_component_row\":"
        << solution.minimum_component_row
        << ",\"minimum_separation\":\""
        << al_binary128_text(solution.minimum_separation)
        << "\",\"worst_residual_row\":"
        << solution.worst_residual_row
        << ",\"maximum_dot_bound\":\""
        << al_binary128_text(solution.maximum_dot_bound)
        << "\",\"maximum_exact_error\":\""
        << al_binary128_text(solution.maximum_exact_error)
        << "\",\"maximum_error_to_bound\":\""
        << al_binary128_text(solution.maximum_error_to_bound)
        << "\",\"dot_sha256\":\"" << solution.dot_root
        << "\",\"root\":\"" << solution.root << "\"}"
        << ",\"solver_executions\":" << solver_executions
        << ",\"captured_inverse_calls\":" << capture.calls
        << ",\"captured_inverse_columns\":"
        << capture.total_column_solves
        << ",\"solution_residual_dots\":" << solution.dots
        << ",\"additional_factorizations\":0"
        << ",\"additional_inverse_columns\":0,\"additional_solves\":0"
        << ",\"nnqp_decisions\":0,\"counterflow_replays\":0"
        << ",\"new_trials\":0,\"state_updates\":0"
        << ",\"parameter_changes\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
