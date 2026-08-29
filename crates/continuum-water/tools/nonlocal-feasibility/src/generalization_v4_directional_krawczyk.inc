namespace {

constexpr const char* AL_R20_R39_SEMANTIC_SHA256 =
    "83e1f7387a1ff2675d3899af904666734fd6b71f3e3079b34617d46f63d21bac";
constexpr const char* AL_R20_R39_RHS_SHA256 =
    "464263aebecdc530c08750698ffc24e86369008463a895517ba2286649f242f8";
constexpr const char* AL_R20_R39_SOLUTION_SHA256 =
    "dbf5c26c33e2d187a8855f076029754d40f05cb90f5becf88eeb5e807cdfb26e";
constexpr const char* AL_R20_R39_RESIDUAL_SHA256 =
    "8a0e41ddfbd1425be0fd6d76e70befad6b393b655de19df171d1e56821c079c4";
constexpr const char* AL_R20_R39_SOLUTION_AUDIT_SHA256 =
    "19a90db0e65869da4edc2abbb16b095beb41fcc9425087ad8e649a3d33444713";

struct ALR20R40ResidualSamples {
    bool exact = false;
    bool no_underflow = false;
    bool contained = false;
    std::size_t dimension = 0U;
    std::size_t contained_entries = 0U;
    std::vector<Binary128> nominal;
    std::vector<Binary128> bound;
    Binary128 maximum_bound = static_cast<Binary128>(0.0);
    Binary128 maximum_exact_error = static_cast<Binary128>(0.0);
    std::string root;
};

ALR20R40ResidualSamples al_r20_r40_residual_samples(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& rhs,
    const std::vector<Binary128>& solution,
    const ALR20R39ExactVectorResidual& oracle) {
    ALR20R40ResidualSamples result;
    result.dimension = rhs.size();
    const std::size_t dimension = result.dimension;
    if (!oracle.exact || oracle.residual.size() != dimension
        || solution.size() != dimension
        || matrix.size() != dimension * dimension)
        return result;
    result.nominal.resize(dimension);
    result.bound.resize(dimension);
    bool no_underflow = true;
    bool contained = true;
    bool apparatus_exact = true;
    std::ostringstream material;
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
        const ALR20R37Conversion nominal =
            al_r20_r37_from_binary128(dot.value);
        const ALR20R37Conversion bound =
            al_r20_r37_from_binary128(dot.bound);
        const bool entry_exact = dot.exact && nominal.exact && bound.exact;
        ALR20R37Dyadic exact_difference;
        if (entry_exact)
            exact_difference = al_r20_r37_absolute(al_r20_r37_add(
                oracle.residual[row],
                {-nominal.value.numerator, nominal.value.exponent}));
        const Binary128 exact_error = entry_exact
            ? al_r20_r37_binary128_up(exact_difference)
            : static_cast<Binary128>(0.0);
        const bool entry_contained = entry_exact
            && al_r20_r37_compare_positive(
                exact_difference, bound.value) <= 0;
        result.nominal[row] = dot.value;
        result.bound[row] = dot.bound;
        result.contained_entries += entry_contained ? 1U : 0U;
        no_underflow = no_underflow && dot.no_underflow;
        contained = contained && entry_contained;
        apparatus_exact = apparatus_exact && entry_exact
            && finiteq(exact_error) != 0;
        result.maximum_bound = fmaxq(result.maximum_bound, dot.bound);
        result.maximum_exact_error = fmaxq(
            result.maximum_exact_error, exact_error);
        material << dot.root << ':' << entry_contained << ';';
    }
    result.no_underflow = no_underflow;
    result.contained = contained
        && result.contained_entries == dimension;
    result.exact = apparatus_exact && result.nominal.size() == dimension
        && al_r20_admm_finite(result.nominal)
        && al_r20_admm_finite(result.bound)
        && finiteq(result.maximum_bound) != 0
        && finiteq(result.maximum_exact_error) != 0;
    material << '|' << result.exact << ':' << result.no_underflow << ':'
        << result.contained << ':' << result.contained_entries << ':'
        << al_binary128_hex(result.maximum_bound) << ':'
        << al_binary128_hex(result.maximum_exact_error) << ':' << oracle.root;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R40ExactCorrection {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t worst_row = 0U;
    ALR20R37Dyadic norm;
    Binary128 outward_norm = static_cast<Binary128>(0.0);
    std::vector<ALR20R37Dyadic> value;
    std::string root;
};

ALR20R40ExactCorrection al_r20_r40_exact_correction(
    const std::vector<Binary128>& inverse,
    const ALR20R39ExactVectorResidual& residual) {
    ALR20R40ExactCorrection result;
    result.dimension = residual.dimension;
    const std::size_t dimension = result.dimension;
    if (!residual.exact || residual.residual.size() != dimension
        || inverse.size() != dimension * dimension)
        return result;
    std::vector<ALR20R37Dyadic> inverse_exact(inverse.size());
    bool operands_exact = true;
    for (std::size_t index = 0U; index < inverse.size(); ++index) {
        const ALR20R37Conversion converted =
            al_r20_r37_from_binary128(inverse[index]);
        operands_exact = operands_exact && converted.exact;
        inverse_exact[index] = converted.value;
    }
    if (!operands_exact) return result;
    std::ostringstream material;
    for (std::size_t row = 0U; row < dimension; ++row) {
        ALR20R37Dyadic correction;
        for (std::size_t column = 0U; column < dimension; ++column)
            correction = al_r20_r37_add(correction, al_r20_r37_multiply(
                inverse_exact[row * dimension + column],
                residual.residual[column]));
        result.value.push_back(correction);
        const ALR20R37Dyadic absolute = al_r20_r37_absolute(correction);
        if (row == 0U
            || al_r20_r37_compare_positive(absolute, result.norm) > 0) {
            result.norm = absolute;
            result.worst_row = row;
        }
        material << al_r20_r37_dyadic_text(correction) << ';';
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

struct ALR20R40DirectionalAudit {
    bool exact = false;
    bool no_underflow = false;
    bool contained = false;
    bool left_contractive = false;
    bool signs_resolved = false;
    std::size_t dimension = 0U;
    std::size_t dots = 0U;
    std::size_t contained_entries = 0U;
    std::size_t positive = 0U;
    std::size_t negative = 0U;
    std::size_t unresolved = 0U;
    std::size_t worst_row = 0U;
    Binary128 z_inf_bound = static_cast<Binary128>(0.0);
    Binary128 directional_error = static_cast<Binary128>(0.0);
    Binary128 minimum_separation = HUGE_VALQ;
    Binary128 maximum_dot_bound = static_cast<Binary128>(0.0);
    Binary128 maximum_input_bound = static_cast<Binary128>(0.0);
    Binary128 maximum_exact_error = static_cast<Binary128>(0.0);
    Binary128 maximum_error_to_bound = static_cast<Binary128>(0.0);
    std::string dot_root;
    std::string root;
};

ALR20R40DirectionalAudit al_r20_r40_directional_audit(
    const std::vector<Binary128>& inverse,
    const std::vector<Binary128>& solution,
    const ALR20R40ResidualSamples& residual,
    const ALR20R40ExactCorrection& oracle,
    const ALR20R38MatrixAudit& left_certificate) {
    ALR20R40DirectionalAudit result;
    result.dimension = solution.size();
    const std::size_t dimension = result.dimension;
    if (!residual.exact || !oracle.exact || !left_certificate.exact
        || residual.nominal.size() != dimension
        || residual.bound.size() != dimension
        || oracle.value.size() != dimension
        || inverse.size() != dimension * dimension)
        return result;
    bool no_underflow = true;
    bool contained = true;
    bool apparatus_exact = true;
    std::ostringstream material;
    for (std::size_t row = 0U; row < dimension; ++row) {
        std::vector<Binary128> left(dimension);
        for (std::size_t column = 0U; column < dimension; ++column)
            left[column] = inverse[row * dimension + column];
        const ALR20R38Dot2 dot =
            al_r20_r38_dot2(left, residual.nominal);
        Binary128 input_bound = static_cast<Binary128>(0.0);
        for (std::size_t column = 0U; column < dimension; ++column)
            input_bound = al_r20_r38_up_add(input_bound,
                al_r20_r38_up_multiply(
                    fabsq(left[column]), residual.bound[column]));
        const Binary128 combined_bound = al_r20_r38_up_add(
            dot.bound, input_bound);
        const ALR20R37Conversion nominal =
            al_r20_r37_from_binary128(dot.value);
        const ALR20R37Conversion bound =
            al_r20_r37_from_binary128(combined_bound);
        ALR20R37Dyadic exact_difference;
        const bool entry_exact = dot.exact && nominal.exact && bound.exact;
        if (entry_exact) {
            exact_difference = al_r20_r37_absolute(al_r20_r37_add(
                oracle.value[row],
                {-nominal.value.numerator, nominal.value.exponent}));
        }
        const Binary128 exact_error = entry_exact
            ? al_r20_r37_binary128_up(exact_difference)
            : static_cast<Binary128>(0.0);
        const bool entry_contained = entry_exact
            && al_r20_r37_compare_positive(
                exact_difference, bound.value) <= 0;
        ++result.dots;
        result.contained_entries += entry_contained ? 1U : 0U;
        no_underflow = no_underflow && dot.no_underflow;
        contained = contained && entry_contained;
        apparatus_exact = apparatus_exact && entry_exact
            && finiteq(exact_error) != 0;
        result.maximum_dot_bound = fmaxq(
            result.maximum_dot_bound, dot.bound);
        result.maximum_input_bound = fmaxq(
            result.maximum_input_bound, input_bound);
        result.maximum_exact_error = fmaxq(
            result.maximum_exact_error, exact_error);
        if (entry_exact
            && combined_bound > static_cast<Binary128>(0.0))
            result.maximum_error_to_bound = fmaxq(
                result.maximum_error_to_bound,
                exact_error / combined_bound);
        const Binary128 component_bound = al_r20_r38_up_add(
            fabsq(dot.value), combined_bound);
        if (row == 0U || component_bound > result.z_inf_bound) {
            result.z_inf_bound = component_bound;
            result.worst_row = row;
        }
        material << dot.root << ':' << al_binary128_hex(input_bound) << ':'
            << al_binary128_hex(combined_bound) << ':'
            << entry_contained << ';';
    }
    result.left_contractive = left_certificate.contractive;
    if (result.left_contractive) {
        const Binary128 denominator = nextafterq(
            static_cast<Binary128>(1.0) - left_certificate.rho_bound,
            -HUGE_VALQ);
        result.directional_error = al_r20_r38_up_divide(
            result.z_inf_bound, denominator);
    } else {
        // A non-contractive left defect is a valid negative result, not an
        // apparatus failure.  Leave every sign unresolved; the route reports
        // LEFT_NONCONTRACTIVE before consulting the sign classification.
        result.directional_error = static_cast<Binary128>(0.0);
    }
    if (result.left_contractive) {
        for (Binary128 component : solution) {
            result.minimum_separation = fminq(result.minimum_separation,
                fabsq(component) - result.directional_error);
            if (component + result.directional_error
                    < static_cast<Binary128>(0.0))
                ++result.negative;
            else if (component - result.directional_error
                    > static_cast<Binary128>(0.0))
                ++result.positive;
            else
                ++result.unresolved;
        }
    } else {
        result.unresolved = dimension;
        result.minimum_separation = static_cast<Binary128>(0.0);
    }
    result.no_underflow = no_underflow;
    result.contained = contained
        && result.contained_entries == dimension;
    result.signs_resolved = result.unresolved == 0U
        && result.positive + result.negative == dimension;
    result.dot_root = sha256_hex(material.str());
    result.exact = apparatus_exact && result.dots == dimension
        && finiteq(result.z_inf_bound) != 0
        && finiteq(result.directional_error) != 0
        && finiteq(result.minimum_separation) != 0
        && finiteq(result.maximum_dot_bound) != 0
        && finiteq(result.maximum_input_bound) != 0
        && finiteq(result.maximum_exact_error) != 0
        && finiteq(result.maximum_error_to_bound) != 0;
    std::ostringstream root_material;
    root_material << result.exact << ':' << result.no_underflow << ':'
        << result.contained << ':' << result.left_contractive << ':'
        << result.signs_resolved << ':' << dimension << ':' << result.dots
        << ':' << result.contained_entries << ':' << result.positive << ':'
        << result.negative << ':' << result.unresolved << ':'
        << result.worst_row << ':' << al_binary128_hex(result.z_inf_bound)
        << ':' << al_binary128_hex(result.directional_error) << ':'
        << al_binary128_hex(result.minimum_separation) << ':'
        << al_binary128_hex(result.maximum_dot_bound) << ':'
        << al_binary128_hex(result.maximum_input_bound) << ':'
        << al_binary128_hex(result.maximum_exact_error) << ':'
        << al_binary128_hex(result.maximum_error_to_bound) << ':'
        << result.dot_root << ':' << oracle.root << ':'
        << left_certificate.root;
    result.root = sha256_hex(root_material.str());
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v4_directional_krawczyk_controls() {
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
    const bool reproduction_exact = parent_exact && capture_slot_clear
        && torsion.root == AL_R20_R35_TORSION_CASE_SHA256
        && !torsion.diagnostic.steps.empty()
        && torsion.diagnostic.steps.back().root
            == AL_R20_R35_TORSION_STEP_SHA256
        && capture.calls == 2U && target_capture_count == 1U
        && capture.audit.root == AL_R20_R36_TORSION_INVERSE_SHA256
        && capture.matrix_root == AL_R20_R37_MATRIX_SHA256
        && capture.factor_root == AL_R20_R37_FACTOR_SHA256
        && al_r20_q_scalar_root(capture.inverse)
            == AL_R20_R37_INVERSE_SHA256
        && al_r20_q_scalar_root(capture.rhs) == AL_R20_R39_RHS_SHA256
        && al_r20_q_scalar_root(capture.solution)
            == AL_R20_R39_SOLUTION_SHA256
        && solver_executions == 1U;
    const ALR20R37ExactResidual right_oracle = reproduction_exact
        ? al_r20_r37_exact_residual(
            capture.matrix, capture.inverse, capture.audit.dimension)
        : ALR20R37ExactResidual{};
    const ALR20R38MatrixAudit right_certificate =
        right_oracle.exact ? al_r20_r38_matrix_audit(
            capture.matrix, capture.inverse, right_oracle)
        : ALR20R38MatrixAudit{};
    const ALR20R39ExactVectorResidual residual_oracle =
        reproduction_exact && right_certificate.root
                == AL_R20_R38_CERTIFICATE_SHA256
        ? al_r20_r39_exact_vector_residual(
            capture.matrix, capture.rhs, capture.solution)
        : ALR20R39ExactVectorResidual{};
    const ALR20R39SolutionAudit parent_solution =
        residual_oracle.exact
        ? al_r20_r39_solution_audit(capture.matrix, capture.inverse,
            capture.rhs, capture.solution, right_certificate, residual_oracle)
        : ALR20R39SolutionAudit{};
    const bool parent_solution_exact =
        residual_oracle.root == AL_R20_R39_RESIDUAL_SHA256
        && parent_solution.root == AL_R20_R39_SOLUTION_AUDIT_SHA256;
    const ALR20R37ExactResidual left_oracle = parent_solution_exact
        ? al_r20_r37_exact_residual(
            capture.inverse, capture.matrix, capture.audit.dimension)
        : ALR20R37ExactResidual{};
    const ALR20R38MatrixAudit left_certificate = left_oracle.exact
        ? al_r20_r38_matrix_audit(
            capture.inverse, capture.matrix, left_oracle)
        : ALR20R38MatrixAudit{};
    const ALR20R40ResidualSamples residual_samples =
        parent_solution_exact
        ? al_r20_r40_residual_samples(
            capture.matrix, capture.rhs, capture.solution, residual_oracle)
        : ALR20R40ResidualSamples{};
    const ALR20R40ExactCorrection correction_oracle =
        residual_samples.exact
        ? al_r20_r40_exact_correction(capture.inverse, residual_oracle)
        : ALR20R40ExactCorrection{};
    const ALR20R40DirectionalAudit directional =
        correction_oracle.exact && left_certificate.exact
        ? al_r20_r40_directional_audit(capture.inverse, capture.solution,
            residual_samples, correction_oracle, left_certificate)
        : ALR20R40DirectionalAudit{};
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool exact = parent_exact && reproduction_exact
        && parent_solution_exact && left_oracle.exact
        && left_certificate.exact && residual_samples.exact
        && correction_oracle.exact && directional.exact && lifecycle_exact;
    const std::string route = !parent_exact || !reproduction_exact
        || !parent_solution_exact
        ? "DIRECTIONAL_KRAWCZYK_PARENT_REJECTED"
        : (!left_oracle.exact || !left_certificate.exact
            || !residual_samples.exact || !correction_oracle.exact
            || !directional.exact
            ? "DIRECTIONAL_KRAWCZYK_APPARATUS_REJECTED"
            : (!left_certificate.no_underflow
                    || !residual_samples.no_underflow
                    || !directional.no_underflow
                ? "DIRECTIONAL_KRAWCZYK_UNDERFLOW_REJECTED"
                : (!left_certificate.contained
                        || !residual_samples.contained
                        || !directional.contained
                    ? "DIRECTIONAL_KRAWCZYK_CONTAINMENT_REJECTED"
                    : (!left_certificate.contractive
                        ? "DIRECTIONAL_KRAWCZYK_LEFT_NONCONTRACTIVE"
                        : (directional.signs_resolved
                            ? "DIRECTIONAL_KRAWCZYK_ERROR_CANDIDATE"
                            : "DIRECTIONAL_KRAWCZYK_SIGN_UNRESOLVED")))));
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r40-directional.v1|"
        << AL_R20_R39_SEMANTIC_SHA256 << ':' << route << ':'
        << torsion.root << ':' << parent_solution.root << ':'
        << left_oracle.root << ':' << left_certificate.root << ':'
        << residual_samples.root << ':' << correction_oracle.root << ':'
        << directional.root;
    const std::string result_sha = sha256_hex(semantic.str());
    const Binary128 improvement = directional.directional_error
            > static_cast<Binary128>(0.0)
        ? parent_solution.solution_error_bound / directional.directional_error
        : HUGE_VALQ;
    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r40_directional.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r39_sha256\":\""
        << AL_R20_R39_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"reproduction_exact\":"
        << (reproduction_exact ? "true" : "false")
        << ",\"parent_solution_exact\":"
        << (parent_solution_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"case_sha256\":\"" << torsion.root
        << "\",\"parent_solution_sha256\":\"" << parent_solution.root
        << "\",\"left_defect\":{\"exact_sha256\":\""
        << left_oracle.root
        << "\",\"dot2_sha256\":\"" << left_certificate.root
        << "\",\"exact_rho_outward\":\""
        << al_binary128_text(left_oracle.outward_norm)
        << "\",\"rho_bound\":\""
        << al_binary128_text(left_certificate.rho_bound)
        << "\",\"worst_row\":" << left_certificate.worst_row
        << ",\"contained_entries\":"
        << left_certificate.contained_entries
        << ",\"no_underflow\":"
        << (left_certificate.no_underflow ? "true" : "false") << "}"
        << ",\"residual_samples_sha256\":\"" << residual_samples.root
        << "\",\"exact_correction\":{\"root\":\""
        << correction_oracle.root
        << "\",\"norm_outward\":\""
        << al_binary128_text(correction_oracle.outward_norm)
        << "\",\"worst_row\":" << correction_oracle.worst_row << "}"
        << ",\"directional\":{\"root\":\"" << directional.root
        << "\",\"no_underflow\":"
        << (directional.no_underflow ? "true" : "false")
        << ",\"contained\":"
        << (directional.contained ? "true" : "false")
        << ",\"left_contractive\":"
        << (directional.left_contractive ? "true" : "false")
        << ",\"signs_resolved\":"
        << (directional.signs_resolved ? "true" : "false")
        << ",\"dots\":" << directional.dots
        << ",\"contained_entries\":" << directional.contained_entries
        << ",\"positive\":" << directional.positive
        << ",\"negative\":" << directional.negative
        << ",\"unresolved\":" << directional.unresolved
        << ",\"z_inf_bound\":\""
        << al_binary128_text(directional.z_inf_bound)
        << "\",\"directional_error\":\""
        << al_binary128_text(directional.directional_error)
        << "\",\"r39_to_directional_ratio\":\""
        << al_binary128_text(improvement)
        << "\",\"minimum_separation\":\""
        << al_binary128_text(directional.minimum_separation)
        << "\",\"worst_row\":" << directional.worst_row
        << ",\"maximum_dot_bound\":\""
        << al_binary128_text(directional.maximum_dot_bound)
        << "\",\"maximum_input_bound\":\""
        << al_binary128_text(directional.maximum_input_bound)
        << "\",\"maximum_exact_error\":\""
        << al_binary128_text(directional.maximum_exact_error)
        << "\",\"maximum_error_to_bound\":\""
        << al_binary128_text(directional.maximum_error_to_bound) << "\"}"
        << ",\"solver_executions\":" << solver_executions
        << ",\"captured_inverse_calls\":" << capture.calls
        << ",\"captured_inverse_columns\":"
        << capture.total_column_solves
        << ",\"left_defect_dots\":" << left_certificate.dots
        << ",\"solution_residual_dots\":" << residual_samples.dimension
        << ",\"directional_dots\":" << directional.dots
        << ",\"additional_factorizations\":0"
        << ",\"additional_inverse_columns\":0,\"additional_solves\":0"
        << ",\"correction_applies\":0,\"nnqp_decisions\":0"
        << ",\"counterflow_replays\":0,\"new_trials\":0"
        << ",\"state_updates\":0,\"parameter_changes\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
