namespace {

constexpr const char* AL_R20_R63V_PARENT_SEMANTIC_SHA256 =
    "832aa624e33ad947ca288c1507efb6fcf752cb38b6a58f87c3a23966b357d1de";
constexpr const char* AL_R20_R63V_WIDE_CERTIFICATE_SHA256 =
    "38bd22c1b779801d807b525acb8a50a4515b7afb16dc84cf0301619358987adf";
constexpr const char* AL_R20_R63V_EXPORTED_CERTIFICATE_SHA256 =
    "f1937129df5e6daf0c6b5c2ecd810c8dbf1d52688e7c701ce108f573e18fe29f";
constexpr const char* AL_R20_R63V_FINAL_SIGN_SHA256 =
    "89b2908b21369cf77a376287ed8142026827815c45a59aa70d2e831aac406094";

struct ALR20R63VProgress {
    bool exact = false;
    bool passed = false;
    bool has_pass = false;
    bool stable_after_pass = true;
    std::size_t expected_states = 0U;
    std::size_t observed_states = 0U;
    std::size_t first_passing_state = 0U;
    std::string root;
};

ALR20R63VProgress al_r20_r63v_progress(
    const std::vector<ALR20R63UCertificate>& certificates,
    std::size_t expected_states) {
    ALR20R63VProgress result;
    result.expected_states = expected_states;
    result.observed_states = certificates.size();
    bool certificates_exact = true;
    std::ostringstream material;
    material << expected_states << ':' << certificates.size() << '|';
    for (std::size_t state = 0U; state < certificates.size(); ++state) {
        const ALR20R63UCertificate& certificate = certificates[state];
        certificates_exact = certificates_exact && certificate.exact;
        if (certificate.passed && !result.has_pass) {
            result.has_pass = true;
            result.first_passing_state = state;
        } else if (result.has_pass && !certificate.passed) {
            result.stable_after_pass = false;
        }
        material << state << ':' << certificate.exact << ':'
            << certificate.passed << ':' << certificate.root << ';';
    }
    result.exact = expected_states > 0U
        && certificates.size() == expected_states && certificates_exact;
    result.passed = result.exact && result.has_pass
        && result.stable_after_pass;
    material << '|' << result.exact << ':' << result.passed << ':'
        << result.has_pass << ':' << result.stable_after_pass << ':'
        << result.first_passing_state;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63VLane {
    bool exact = false;
    bool passed = false;
    bool parent_exact = false;
    bool solution_correspondence = false;
    std::size_t dimension = 0U;
    std::size_t certificates = 0U;
    std::size_t solution_conversions = 0U;
    std::size_t residual_products = 0U;
    std::size_t residual_entries = 0U;
    std::size_t image_products = 0U;
    std::size_t image_entries = 0U;
    std::size_t sign_comparisons = 0U;
    ALR20R63VProgress progress;
    std::vector<ALR20R63UCertificate> state_certificates;
    std::string parent_lane_root;
    std::string parent_final_solution_root;
    std::string root;
};

ALR20R63VLane al_r20_r63v_lane(
    const ALR20R63SLane& parent,
    const std::vector<Binary128>& initial_solution,
    const std::vector<ALR20R37Dyadic>& rhs,
    const ALR20R63QExactMatrix& matrix,
    const ALR20R63RDefect& defect,
    const std::vector<ALR20R37Dyadic>& inverse) {
    ALR20R63VLane result;
    result.dimension = rhs.size();
    result.parent_lane_root = parent.root;
    result.parent_final_solution_root = al_r20_q_scalar_root(parent.solution);
    const std::size_t dimension = result.dimension;
    result.parent_exact = parent.exact && parent.finite
        && parent.completed_iterations == 8U && parent.steps.size() == 8U
        && parent.initial_certificate.exact
        && initial_solution.size() == dimension
        && parent.solution.size() == dimension && dimension > 0U;
    if (!result.parent_exact) return result;
    result.state_certificates.reserve(9U);
    bool correspondence = true;
    auto record = [&result](ALR20R63UCertificate certificate) {
        ++result.certificates;
        result.solution_conversions += certificate.solution_conversions;
        result.residual_products += certificate.residual_products;
        result.residual_entries += certificate.residual_entries;
        result.image_products += certificate.image_products;
        result.image_entries += certificate.image_entries;
        result.sign_comparisons += certificate.sign_comparisons;
        result.state_certificates.push_back(std::move(certificate));
    };
    ALR20R63UCertificate initial = al_r20_r63u_certificate(
        rhs, matrix, defect, inverse, initial_solution);
    correspondence = correspondence && initial.solution_root
        == parent.initial_certificate.solution_root;
    record(std::move(initial));
    for (std::size_t index = 0U; index < parent.steps.size(); ++index) {
        const ALR20R63SStep& step = parent.steps[index];
        ALR20R63UCertificate certificate = al_r20_r63u_certificate(
            rhs, matrix, defect, inverse, step.solution);
        correspondence = correspondence && step.iteration == index + 1U
            && step.solution.size() == dimension
            && certificate.solution_root == step.certificate.solution_root;
        record(std::move(certificate));
    }
    result.solution_correspondence = correspondence;
    result.progress = al_r20_r63v_progress(
        result.state_certificates, 9U);
    const bool work_exact = result.certificates == 9U
        && result.solution_conversions == 918U
        && result.residual_products == 93636U
        && result.residual_entries == 918U
        && result.image_products == 93636U
        && result.image_entries == 918U
        && result.sign_comparisons == 918U;
    result.exact = result.parent_exact && result.solution_correspondence
        && result.progress.exact && work_exact;
    result.passed = result.exact && result.progress.passed;
    std::ostringstream material;
    material << result.exact << ':' << result.passed << ':'
        << result.parent_exact << ':' << result.solution_correspondence << ':'
        << result.dimension << ':' << result.certificates << ':'
        << result.solution_conversions << ':' << result.residual_products
        << ':' << result.residual_entries << ':' << result.image_products
        << ':' << result.image_entries << ':' << result.sign_comparisons
        << ':' << result.parent_lane_root << ':'
        << result.parent_final_solution_root << ':' << result.progress.root
        << '|';
    for (const ALR20R63UCertificate& certificate : result.state_certificates)
        material << certificate.root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

bool al_r20_r63v_parent_work_exact(const ALR20R63SLane& lane) {
    return lane.completed_iterations == 8U && lane.steps.size() == 8U
        && lane.rectangular_products == 9U
        && lane.direct_inner_dots == 2835U
        && lane.direct_outer_dots == 918U
        && lane.direct_inner_terms == 289170U
        && lane.direct_outer_terms == 289170U
        && lane.direct_scale_products == 918U
        && lane.initial_residual_dots == 102U
        && lane.preconditioner_solves == 8U
        && lane.forward_terms == 41208U
        && lane.backward_terms == 41208U && lane.divisions == 1632U
        && lane.rho_dots == 8U && lane.denominator_dots == 8U
        && lane.solution_update_dots == 816U
        && lane.residual_update_dots == 816U
        && lane.direction_update_dots == 714U
        && lane.certificates == 9U
        && lane.certificate_solution_conversions == 918U
        && lane.certificate_residual_products == 93636U
        && lane.certificate_residual_entries == 918U
        && lane.certificate_sign_comparisons == 918U;
}

std::string al_r20_r63v_classify(
    bool apparatus, bool certificate, bool wide, bool exported) {
    if (!apparatus) return "EARLIEST_IMAGE_APPARATUS_REJECTED";
    if (!certificate) return "EARLIEST_IMAGE_CERTIFICATE_REJECTED";
    if (!wide) return "RETAINED_WIDE_EARLIEST_IMAGE_REJECTED";
    if (!exported) return "EXPORTED_FACTOR_EARLIEST_IMAGE_REJECTED";
    return "TWO_LANE_EARLIEST_IMAGE_CANDIDATE";
}

struct ALR20R63VControls {
    bool exact = false;
    bool state_zero = false;
    bool state_one = false;
    bool regression = false;
    bool missing = false;
    bool independence = false;
    bool mutation = false;
    bool classifier = false;
    std::string root;
};

ALR20R63VControls al_r20_r63v_controls() {
    ALR20R63VControls result;
    auto certificate = [](bool passed, const char* label) {
        ALR20R63UCertificate value;
        value.exact = true;
        value.passed = passed;
        value.root = sha256_hex(label);
        return value;
    };
    const ALR20R63VProgress state_zero = al_r20_r63v_progress(
        {certificate(true, "z0"), certificate(true, "z1")}, 2U);
    result.state_zero = state_zero.exact && state_zero.passed
        && state_zero.has_pass && state_zero.stable_after_pass
        && state_zero.first_passing_state == 0U;
    const ALR20R63VProgress state_one = al_r20_r63v_progress(
        {certificate(false, "o0"), certificate(true, "o1"),
            certificate(true, "o2")}, 3U);
    result.state_one = state_one.exact && state_one.passed
        && state_one.first_passing_state == 1U;
    const ALR20R63VProgress regression = al_r20_r63v_progress(
        {certificate(true, "r0"), certificate(false, "r1")}, 2U);
    result.regression = regression.exact && !regression.passed
        && regression.has_pass && !regression.stable_after_pass;
    const ALR20R63VProgress missing = al_r20_r63v_progress(
        {certificate(true, "m0"), certificate(true, "m1")}, 3U);
    result.missing = !missing.exact && !missing.passed;
    std::vector<ALR20R63UCertificate> mutated_certificates{
        certificate(false, "i0"), certificate(true, "i1")};
    const ALR20R63VProgress original = al_r20_r63v_progress(
        mutated_certificates, 2U);
    const std::string parent_root = sha256_hex("parent-recurrence");
    const std::string solution_root = sha256_hex("parent-solution");
    mutated_certificates[1U].root = sha256_hex("mutated-certificate");
    const ALR20R63VProgress mutated = al_r20_r63v_progress(
        mutated_certificates, 2U);
    result.independence = parent_root == sha256_hex("parent-recurrence")
        && solution_root == sha256_hex("parent-solution")
        && original.first_passing_state == mutated.first_passing_state
        && original.passed == mutated.passed;
    result.mutation = original.root != mutated.root;
    result.classifier =
        al_r20_r63v_classify(false, true, true, true)
            == "EARLIEST_IMAGE_APPARATUS_REJECTED"
        && al_r20_r63v_classify(true, false, true, true)
            == "EARLIEST_IMAGE_CERTIFICATE_REJECTED"
        && al_r20_r63v_classify(true, true, false, true)
            == "RETAINED_WIDE_EARLIEST_IMAGE_REJECTED"
        && al_r20_r63v_classify(true, true, true, false)
            == "EXPORTED_FACTOR_EARLIEST_IMAGE_REJECTED"
        && al_r20_r63v_classify(true, true, true, true)
            == "TWO_LANE_EARLIEST_IMAGE_CANDIDATE";
    result.exact = result.state_zero && result.state_one
        && result.regression && result.missing && result.independence
        && result.mutation && result.classifier;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + state_zero.root + ':' + state_one.root + ':' + regression.root
        + ':' + missing.root + ':' + original.root + ':' + mutated.root);
    return result;
}

} // namespace

SplitBoundaryReport
run_al_generalization_v5_earliest_residual_image_controls() {
#if defined(__linux__) && defined(__x86_64__) && defined(__GNUC__) \
    && defined(__SIZEOF_FLOAT128__) && !defined(__FAST_MATH__)
    constexpr bool frozen_platform = true;
#else
    constexpr bool frozen_platform = false;
#endif
    const bool profile_exact = frozen_platform
        && sizeof(Binary128) == 16U && FLT128_MANT_DIG == 113
        && sizeof(double) == 8U && FLT_RADIX == 2 && DBL_MANT_DIG == 53
        && FLT_EVAL_METHOD == 0 && std::numeric_limits<double>::is_iec559
        && std::numeric_limits<double>::has_denorm == std::denorm_present
        && std::fegetround() == FE_TONEAREST;
    const ALR20R63VControls controls = al_r20_r63v_controls();
    const ALR20R63UControls parent_controls = al_r20_r63u_controls();
    const ALR20R63SControls parent_r63s_controls = al_r20_r63s_controls();
    const bool controls_exact = profile_exact && controls.exact
        && parent_controls.exact && parent_r63s_controls.exact
        && parent_r63s_controls.root == AL_R20_R63T_PARENT_CONTROLS_SHA256;
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
            && problem.target.size() == 105U
            && problem.problem_root == AL_R20_R60_PROBLEM_SHA256;
    }
    ALR20R60Capture solution_capture;
    ALR20VerifiedInverseCapture factor_capture;
    factor_capture.selected_matrix_root = AL_R20_R63_MATRIX_SHA256;
    ALR20R55SlopeTrace slope;
    ALR20R35CaseAudit oblique;
    std::size_t solver_executions = 0U;
    const bool hook_slot_clear = al_r20_verified_solution_refiner == nullptr
        && al_r20_r60_capture == nullptr
        && al_r20_verified_inverse_capture == nullptr
        && al_r20_verified_inverse_projector_context == nullptr
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr;
    if (parent_exact && hook_slot_clear) {
        al_r20_r60_capture = &solution_capture;
        al_r20_verified_solution_refiner = &al_r20_r60_capture_refiner;
        al_r20_verified_inverse_capture = &factor_capture;
        al_r20_r55_slope_trace = &slope;
        al_r20_slope_error_refiner = &al_r20_r55_slope_refiner;
        oblique = al_r20_r35_case(problem);
        al_r20_slope_error_refiner = nullptr;
        al_r20_r55_slope_trace = nullptr;
        al_r20_verified_inverse_capture = nullptr;
        al_r20_verified_solution_refiner = nullptr;
        al_r20_r60_capture = nullptr;
        solver_executions = 1U;
    }
    const bool hook_slot_released =
        al_r20_verified_solution_refiner == nullptr
        && al_r20_r60_capture == nullptr
        && al_r20_verified_inverse_capture == nullptr
        && al_r20_verified_inverse_projector_context == nullptr
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr
        && al_r20_direct_solution_capture == nullptr;
    const bool capture_exact = parent_exact && hook_slot_clear
        && hook_slot_released && solver_executions == 1U
        && al_r20_r35_case_observation_exact(oblique)
        && oblique.root == AL_R20_R60_CASE_SHA256
        && solution_capture.calls == 1U
        && factor_capture.calls == 125U
        && factor_capture.selected_calls == 1U
        && factor_capture.selected_column_solves == 102U
        && factor_capture.matrix_root == AL_R20_R63_MATRIX_SHA256
        && al_r20_q_scalar_root(factor_capture.rhs) == AL_R20_R63I_RHS_SHA256
        && factor_capture.source_rows.size() == 102U
        && factor_capture.projector.exact
        && slope.calls == 0U && slope.inverse_audits == 0U;
    const ALR20R63EProjectorGrams grams = capture_exact
        ? al_r20_r63e_projector_grams(problem.op,
            factor_capture.source_rows, problem.target.size(),
            factor_capture.projector, factor_capture.matrix)
        : ALR20R63EProjectorGrams{};
    const ALR20R63ERankProfile full_profile = grams.exact
        ? al_r20_r63e_rank_profile(grams.full, grams.rows)
        : ALR20R63ERankProfile{};
    const ALR20R63FTangentMatrix tangent = grams.exact
        ? al_r20_r63f_tangent_matrix(grams, factor_capture.projector,
            factor_capture.source_rows)
        : ALR20R63FTangentMatrix{};
    const Binary128 inverse_scale = static_cast<Binary128>(1.0)
        + factor_capture.projector.eta;
    const Binary128 sigma = static_cast<Binary128>(1.0) / inverse_scale;
    const ALR20R63QAudit common_audit = tangent.exact
        ? al_r20_r63q_audit(grams, tangent, factor_capture.projector,
            factor_capture.matrix, sigma)
        : ALR20R63QAudit{};
    const std::vector<std::size_t> permutation = full_profile.exact
        ? full_profile.binary128.permutation
        : std::vector<std::size_t>{};
    const bool permutation_exact = al_r20_r63g_permutation_exact(
        permutation, tangent.rows)
        && al_r20_r63g_index_root(permutation)
            == AL_R20_R63R_PERMUTATION_SHA256;
    std::vector<Binary128> permuted_original;
    std::vector<double> permuted_d;
    if (common_audit.exact && permutation_exact) {
        permuted_original.resize(tangent.columns * tangent.rows);
        permuted_d.resize(permuted_original.size());
        for (std::size_t scalar = 0U; scalar < tangent.columns; ++scalar)
            for (std::size_t column = 0U; column < tangent.rows; ++column) {
                const Binary128 value = tangent.values[
                    permutation[column] * tangent.columns + scalar];
                permuted_original[scalar * tangent.rows + column] = value;
                permuted_d[scalar * tangent.rows + column] =
                    static_cast<double>(value);
            }
    }
    const ALR20R63GQFactor original_factor = !permuted_original.empty()
        ? al_r20_r63g_q_factor(
            permuted_original, tangent.columns, tangent.rows)
        : ALR20R63GQFactor{};
    const ALR20R63RInverse verifier_inverse = original_factor.exact
        ? al_r20_r63r_inverse(
            original_factor.r, permutation, inverse_scale)
        : ALR20R63RInverse{};
    const ALR20R63RExactAudit verifier_audit = verifier_inverse.exact
        ? al_r20_r63r_exact_audit(verifier_inverse, common_audit.matrix)
        : ALR20R63RExactAudit{};
    const bool parent_r63r_exact = capture_exact && common_audit.exact
        && common_audit.root == AL_R20_R63R_AUDIT_SHA256
        && common_audit.matrix.root == AL_R20_R63R_MATRIX_SHA256
        && original_factor.root == AL_R20_R63R_FACTOR_SHA256
        && verifier_inverse.root == AL_R20_R63S_INVERSE_SHA256
        && verifier_audit.defect.root == AL_R20_R63S_DEFECT_SHA256
        && verifier_audit.root == AL_R20_R63S_INVERSE_AUDIT_SHA256;
    const ALR20R63GDFactor stored_d_factor = !permuted_d.empty()
        ? al_r20_r63g_d_factor(
            permuted_d, tangent.columns, tangent.rows)
        : ALR20R63GDFactor{};
    const std::vector<Binary128> stored_permuted =
        al_r20_r63g_q_from_double(stored_d_factor.input);
    const ALR20R63GQFactor wide_factor = !stored_permuted.empty()
        ? al_r20_r63g_q_factor(
            stored_permuted, tangent.columns, tangent.rows)
        : ALR20R63GQFactor{};
    std::vector<double> exported_d;
    if (wide_factor.exact)
        for (Binary128 value : wide_factor.r)
            exported_d.push_back(static_cast<double>(value));
    const std::vector<Binary128> exported_r =
        al_r20_r63g_q_from_double(exported_d);
    const bool preconditioners_exact = stored_d_factor.exact
        && wide_factor.exact
        && wide_factor.root == AL_R20_R63I_WIDE_FACTOR_SHA256
        && al_r20_double_root(exported_d) == AL_R20_R63I_EXPORT_R_SHA256;
    const ALR20R63IQSolve wide_start = preconditioners_exact
        ? al_r20_r63i_q_solve(wide_factor.r, permutation,
            factor_capture.rhs, inverse_scale)
        : ALR20R63IQSolve{};
    const ALR20R63IQSolve export_start = preconditioners_exact
        ? al_r20_r63i_q_solve(exported_r, permutation,
            factor_capture.rhs, inverse_scale)
        : ALR20R63IQSolve{};
    const bool starts_exact = wide_start.exact && export_start.exact
        && wide_start.root == AL_R20_R63J_WIDE_SOLVE_SHA256
        && export_start.root == AL_R20_R63J_EXPORT_WIDE_SOLVE_SHA256;
    bool rhs_conversions_exact = true;
    std::vector<ALR20R37Dyadic> exact_rhs(factor_capture.rhs.size());
    for (std::size_t index = 0U; index < factor_capture.rhs.size(); ++index) {
        const ALR20R37Conversion conversion =
            al_r20_r37_from_binary128(factor_capture.rhs[index]);
        rhs_conversions_exact = rhs_conversions_exact && conversion.exact;
        exact_rhs[index] = conversion.value;
    }
    const std::string exact_rhs_root = al_r20_r63q_vector_root(exact_rhs);
    const ALR20R63SLane parent_wide = parent_r63r_exact && starts_exact
        && rhs_conversions_exact
        ? al_r20_r63s_lane(wide_factor.r, permutation, inverse_scale,
            tangent.values, tangent.columns, sigma, factor_capture.rhs,
            exact_rhs, common_audit.matrix, verifier_audit.defect,
            wide_start.solution)
        : ALR20R63SLane{};
    const ALR20R63SLane parent_exported = parent_r63r_exact && starts_exact
        && rhs_conversions_exact
        ? al_r20_r63s_lane(exported_r, permutation, inverse_scale,
            tangent.values, tangent.columns, sigma, factor_capture.rhs,
            exact_rhs, common_audit.matrix, verifier_audit.defect,
            export_start.solution)
        : ALR20R63SLane{};
    const std::string parent_r63s_route = al_r20_r63s_classify(true, true,
        parent_wide.passed, parent_exported.passed);
    std::ostringstream parent_r63s_material;
    parent_r63s_material
        << "nextengine.nonlocal.nsr3b4e2d7r20r63s-common-operator-pcg.v1|"
        << AL_R20_R63S_PARENT_SEMANTIC_SHA256 << ':' << parent_r63s_route
        << ':' << parent_r63s_controls.root << ':' << common_audit.root << ':'
        << verifier_audit.root << ':' << exact_rhs_root << ':'
        << parent_wide.root << ':' << parent_exported.root;
    const std::string parent_r63s_semantic =
        sha256_hex(parent_r63s_material.str());
    const bool parent_r63s_exact = parent_r63r_exact
        && preconditioners_exact && starts_exact && rhs_conversions_exact
        && exact_rhs_root == AL_R20_R63T_EXACT_RHS_SHA256
        && parent_wide.exact && parent_exported.exact
        && al_r20_r63v_parent_work_exact(parent_wide)
        && al_r20_r63v_parent_work_exact(parent_exported)
        && parent_wide.root == AL_R20_R63U_WIDE_LANE_SHA256
        && parent_exported.root == AL_R20_R63U_EXPORTED_LANE_SHA256
        && parent_r63s_semantic == AL_R20_R63U_PARENT_SEMANTIC_SHA256;
    bool inverse_conversions_exact = true;
    std::size_t inverse_conversions = 0U;
    std::vector<ALR20R37Dyadic> exact_inverse(verifier_inverse.value.size());
    for (std::size_t index = 0U;
         index < verifier_inverse.value.size(); ++index) {
        const ALR20R37Conversion conversion =
            al_r20_r37_from_binary128(verifier_inverse.value[index]);
        inverse_conversions_exact = inverse_conversions_exact
            && conversion.exact;
        exact_inverse[index] = conversion.value;
        ++inverse_conversions;
    }
    const std::string inverse_root = al_r20_r63q_vector_root(exact_inverse);
    const ALR20R63UCertificate parent_wide_certificate = parent_r63s_exact
        && inverse_conversions_exact
        ? al_r20_r63u_certificate(exact_rhs, common_audit.matrix,
            verifier_audit.defect, exact_inverse, parent_wide.solution)
        : ALR20R63UCertificate{};
    const ALR20R63UCertificate parent_exported_certificate = parent_r63s_exact
        && inverse_conversions_exact
        ? al_r20_r63u_certificate(exact_rhs, common_audit.matrix,
            verifier_audit.defect, exact_inverse, parent_exported.solution)
        : ALR20R63UCertificate{};
    const std::string parent_r63u_route = al_r20_r63u_classify(
        true, true, parent_wide_certificate.passed,
        parent_exported_certificate.passed);
    std::ostringstream parent_r63u_material;
    parent_r63u_material
        << "nextengine.nonlocal.nsr3b4e2d7r20r63u-exact-residual-image.v1|"
        << AL_R20_R63U_CAUSAL_R63T_SHA256 << ':' << parent_r63s_semantic
        << ':' << parent_r63u_route << ':' << parent_controls.root << ':'
        << inverse_root << ':' << parent_wide_certificate.root << ':'
        << parent_exported_certificate.root;
    const std::string parent_r63u_semantic =
        sha256_hex(parent_r63u_material.str());
    const bool parent_r63u_exact = parent_r63s_exact
        && inverse_conversions_exact && inverse_conversions == 10404U
        && inverse_root == verifier_audit.conversion_root
        && parent_wide_certificate.exact
        && parent_exported_certificate.exact
        && parent_wide_certificate.passed
        && parent_exported_certificate.passed
        && parent_wide_certificate.root
            == AL_R20_R63V_WIDE_CERTIFICATE_SHA256
        && parent_exported_certificate.root
            == AL_R20_R63V_EXPORTED_CERTIFICATE_SHA256
        && parent_wide_certificate.sign_root == AL_R20_R63V_FINAL_SIGN_SHA256
        && parent_exported_certificate.sign_root
            == AL_R20_R63V_FINAL_SIGN_SHA256
        && parent_r63u_semantic == AL_R20_R63V_PARENT_SEMANTIC_SHA256;
    const ALR20R63VLane wide = parent_r63u_exact
        ? al_r20_r63v_lane(parent_wide, wide_start.solution, exact_rhs,
            common_audit.matrix, verifier_audit.defect, exact_inverse)
        : ALR20R63VLane{};
    const ALR20R63VLane exported = parent_r63u_exact
        ? al_r20_r63v_lane(parent_exported, export_start.solution, exact_rhs,
            common_audit.matrix, verifier_audit.defect, exact_inverse)
        : ALR20R63VLane{};
    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool final_certificate_exact = !wide.state_certificates.empty()
        && !exported.state_certificates.empty()
        && wide.state_certificates.back().root
            == AL_R20_R63V_WIDE_CERTIFICATE_SHA256
        && exported.state_certificates.back().root
            == AL_R20_R63V_EXPORTED_CERTIFICATE_SHA256
        && wide.state_certificates.back().sign_root
            == AL_R20_R63V_FINAL_SIGN_SHA256
        && exported.state_certificates.back().sign_root
            == AL_R20_R63V_FINAL_SIGN_SHA256;
    const bool parent_lane_roots_exact =
        wide.parent_lane_root == AL_R20_R63U_WIDE_LANE_SHA256
        && exported.parent_lane_root == AL_R20_R63U_EXPORTED_LANE_SHA256;
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_r63u_exact && lifecycle_exact;
    const bool certificate_exact = wide.exact && exported.exact
        && final_certificate_exact && parent_lane_roots_exact;
    const std::string route = al_r20_r63v_classify(apparatus_exact,
        certificate_exact, wide.passed, exported.passed);
    const bool exact = apparatus_exact && certificate_exact;
    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63v-earliest-residual-image.v1|"
        << parent_r63u_semantic << ':' << route << ':' << controls.root << ':'
        << wide.root << ':' << exported.root;
    const std::string result_sha = sha256_hex(semantic.str());
    auto lane_json = [](const char* name, const ALR20R63VLane& lane) {
        std::ostringstream stream;
        stream << "\"" << name << "\":{\"exact\":"
            << (lane.exact ? "true" : "false")
            << ",\"passed\":" << (lane.passed ? "true" : "false")
            << ",\"has_pass\":"
            << (lane.progress.has_pass ? "true" : "false")
            << ",\"stable_after_pass\":"
            << (lane.progress.stable_after_pass ? "true" : "false")
            << ",\"first_passing_state\":"
            << lane.progress.first_passing_state
            << ",\"states\":[";
        for (std::size_t state = 0U;
             state < lane.state_certificates.size(); ++state) {
            if (state != 0U) stream << ',';
            const ALR20R63UCertificate& certificate =
                lane.state_certificates[state];
            stream << "{\"state\":" << state
                << ",\"passed\":"
                << (certificate.passed ? "true" : "false")
                << ",\"image_error\":\""
                << al_binary128_text(al_r20_r63p_fraction_projection(
                    certificate.image_infinity_numerator,
                    certificate.error_denominator))
                << "\",\"positive\":" << certificate.positive
                << ",\"negative\":" << certificate.negative
                << ",\"unresolved\":" << certificate.unresolved
                << ",\"root\":\"" << certificate.root << "\"}";
        }
        stream << "],\"certificates\":" << lane.certificates
            << ",\"solution_correspondence\":"
            << (lane.solution_correspondence ? "true" : "false")
            << ",\"parent_lane_root\":\"" << lane.parent_lane_root << "\""
            << ",\"root\":\"" << lane.root << "\"}";
        return stream.str();
    };
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63v_earliest_residual_image.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63u_sha256\":\"" << parent_r63u_semantic << "\""
        << ",\"controls_root\":\"" << controls.root << "\""
        << ",\"parent_r63u_exact\":"
        << (parent_r63u_exact ? "true" : "false")
        << ",\"lanes\":{" << lane_json("wide", wide) << ','
        << lane_json("exported", exported) << '}'
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"certificate_exact\":"
        << (certificate_exact ? "true" : "false")
        << ",\"final_certificate_exact\":"
        << (final_certificate_exact ? "true" : "false")
        << ",\"parent_lane_roots_exact\":"
        << (parent_lane_roots_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"candidate_factor_solves\":"
        << (parent_wide.preconditioner_solves
            + parent_exported.preconditioner_solves)
        << ",\"candidate_rectangular_products\":"
        << (parent_wide.rectangular_products
            + parent_exported.rectangular_products)
        << ",\"exact_residual_products\":"
        << (wide.residual_products + exported.residual_products)
        << ",\"exact_image_products\":"
        << (wide.image_products + exported.image_products)
        << ",\"certificate_driven_updates\":0,\"early_stops\":0"
        << ",\"stored_dense_h_uses\":0,\"stored_dense_x_uses\":0"
        << ",\"legacy_sign_uses\":0,\"sparse_constructions\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"gpu_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
