namespace {

constexpr const char* AL_R20_R63B_PARENT_SEMANTIC_SHA256 =
    "c08b7abead8ce5623f78761be7af3f9546ed389256e691aefac95c355317639b";

struct ALR20R63BTriangularInverse {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t columns = 0U;
    std::size_t multiply_subtracts = 0U;
    std::size_t divisions = 0U;
    Binary128 minimum_diagonal = static_cast<Binary128>(0.0);
    Binary128 maximum_diagonal = static_cast<Binary128>(0.0);
    std::vector<Binary128> inverse;
    std::string root;
};

ALR20R63BTriangularInverse al_r20_r63b_triangular_inverse(
    const std::vector<Binary128>& lower, std::size_t dimension) {
    ALR20R63BTriangularInverse result;
    result.dimension = dimension;
    if (dimension == 0U || lower.size() != dimension * dimension)
        return result;
    result.inverse.assign(dimension * dimension,
        static_cast<Binary128>(0.0));
    bool exact = true;
    for (std::size_t column = 0U; column < dimension && exact; ++column) {
        for (std::size_t row = column; row < dimension; ++row) {
            Binary128Accumulator image;
            for (std::size_t inner = column; inner < row; ++inner) {
                image.add(lower[row * dimension + inner]
                    * result.inverse[inner * dimension + column]);
                ++result.multiply_subtracts;
            }
            const Binary128 diagonal = lower[row * dimension + row];
            const Binary128 rhs = row == column
                ? static_cast<Binary128>(1.0)
                : static_cast<Binary128>(0.0);
            exact = finiteq(diagonal) != 0
                && diagonal > static_cast<Binary128>(0.0);
            if (!exact) break;
            const Binary128 value = (rhs - image.value()) / diagonal;
            exact = finiteq(value) != 0;
            if (!exact) break;
            result.inverse[row * dimension + column] = value;
            ++result.divisions;
            if (row == column) {
                if (column == 0U || diagonal < result.minimum_diagonal)
                    result.minimum_diagonal = diagonal;
                result.maximum_diagonal = fmaxq(
                    result.maximum_diagonal, diagonal);
            }
        }
        ++result.columns;
    }
    result.exact = exact && result.columns == dimension
        && result.multiply_subtracts
            == dimension * (dimension - 1U) * (dimension + 1U) / 6U
        && result.divisions == dimension * (dimension + 1U) / 2U
        && al_r20_admm_finite(result.inverse);
    std::ostringstream material;
    material << result.exact << ':' << dimension << ':' << result.columns
        << ':' << result.multiply_subtracts << ':' << result.divisions << ':'
        << al_binary128_hex(result.minimum_diagonal) << ':'
        << al_binary128_hex(result.maximum_diagonal) << ':'
        << al_r20_q_scalar_root(result.inverse);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63BReconstruction {
    bool exact = false;
    bool contained = false;
    std::size_t dimension = 0U;
    ALR20R63ANorm norm;
    ALR20R37Dyadic maximum_entry;
    Binary128 maximum_entry_outward = static_cast<Binary128>(0.0);
    std::string root;
};

ALR20R63BReconstruction al_r20_r63b_reconstruction(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& lower,
    std::size_t dimension,
    Binary128 inherited_bound) {
    ALR20R63BReconstruction result;
    result.dimension = dimension;
    const ALR20R63AExactMatrix matrix_exact =
        al_r20_r63a_exact_matrix(matrix, dimension);
    const ALR20R63AExactMatrix lower_exact =
        al_r20_r63a_exact_matrix(lower, dimension);
    if (!matrix_exact.exact || !lower_exact.exact) return result;
    std::vector<ALR20R37Dyadic> residual;
    residual.reserve(dimension * dimension);
    for (std::size_t row = 0U; row < dimension; ++row)
        for (std::size_t column = 0U; column < dimension; ++column) {
            ALR20R37Dyadic image;
            const std::size_t end = std::min(row, column);
            for (std::size_t inner = 0U; inner <= end; ++inner)
                image = al_r20_r37_add(image, al_r20_r37_multiply(
                    lower_exact.values[row * dimension + inner],
                    lower_exact.values[column * dimension + inner]));
            const ALR20R37Dyadic gap = al_r20_r37_add(
                matrix_exact.values[row * dimension + column],
                al_r20_r63_negate(image));
            residual.push_back(gap);
            const ALR20R37Dyadic absolute = al_r20_r37_absolute(gap);
            if (residual.size() == 1U
                || al_r20_r37_compare_positive(
                    absolute, result.maximum_entry) > 0)
                result.maximum_entry = absolute;
        }
    result.norm = al_r20_r63a_norm(residual, dimension);
    result.maximum_entry_outward =
        al_r20_r37_binary128_up(result.maximum_entry);
    const ALR20R37Conversion inherited =
        al_r20_r37_from_binary128(inherited_bound);
    result.contained = inherited.exact
        && al_r20_r37_compare_positive(
            result.maximum_entry, inherited.value) <= 0;
    result.exact = result.norm.exact && result.contained
        && finiteq(result.maximum_entry_outward) != 0;
    std::ostringstream material;
    material << result.exact << ':' << result.contained << ':' << dimension
        << ':' << result.norm.root << ':'
        << al_r20_r37_dyadic_text(result.maximum_entry) << ':'
        << al_binary128_hex(result.maximum_entry_outward) << ':'
        << al_binary128_hex(inherited_bound);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63BConditionBound {
    bool exact = false;
    Binary128 lower = static_cast<Binary128>(0.0);
    Binary128 unit_lower = static_cast<Binary128>(0.0);
    std::string root;
};

ALR20R63BConditionBound al_r20_r63b_condition_lower(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& inverse,
    const ALR20R37ExactResidual& left) {
    ALR20R63BConditionBound result;
    const std::size_t dimension = left.dimension;
    const ALR20R63AExactMatrix matrix_exact =
        al_r20_r63a_exact_matrix(matrix, dimension);
    const ALR20R63AExactMatrix inverse_exact =
        al_r20_r63a_exact_matrix(inverse, dimension);
    const ALR20R63AFacts matrix_facts =
        al_r20_r63a_facts(matrix_exact.values, dimension);
    const ALR20R63AFacts inverse_facts =
        al_r20_r63a_facts(inverse_exact.values, dimension);
    if (!matrix_exact.exact || !inverse_exact.exact
        || !matrix_facts.exact || !inverse_facts.exact
        || !left.exact || !left.contractive)
        return result;
    const ALR20R37Dyadic numerator = al_r20_r37_multiply(
        matrix_facts.norm.value, inverse_facts.norm.value);
    const ALR20R37Dyadic denominator = al_r20_r37_add(
        {ALR20R37Integer(1), 0}, left.norm);
    const Binary128 numerator_down =
        al_r20_r63a_binary128_down(numerator);
    const Binary128 denominator_up =
        al_r20_r37_binary128_up(denominator);
    if (!(numerator_down > static_cast<Binary128>(0.0))
        || !(denominator_up > static_cast<Binary128>(0.0)))
        return result;
    result.lower = nextafterq(
        numerator_down / denominator_up,
        static_cast<Binary128>(0.0));
    result.unit_lower = ldexpq(result.lower, -53);
    result.exact = finiteq(result.lower) != 0
        && finiteq(result.unit_lower) != 0
        && result.lower > static_cast<Binary128>(0.0)
        && result.unit_lower > static_cast<Binary128>(1.0);
    std::ostringstream material;
    material << result.exact << ':' << al_binary128_hex(result.lower) << ':'
        << al_binary128_hex(result.unit_lower) << ':'
        << al_r20_r37_dyadic_text(numerator) << ':'
        << al_r20_r37_dyadic_text(denominator);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63BControls {
    bool exact = false;
    bool good_factor = false;
    bool tiny_boundary = false;
    bool shrunk_rejected = false;
    bool reconstruction_rejected = false;
    std::string root;
};

ALR20R63BControls al_r20_r63b_controls() {
    ALR20R63BControls result;
    const Binary128 one = static_cast<Binary128>(1.0);
    const Binary128 zero = static_cast<Binary128>(0.0);
    const std::vector<Binary128> lower{
        static_cast<Binary128>(2.0), zero,
        static_cast<Binary128>(0.5), one};
    const ALR20R63BTriangularInverse inverse =
        al_r20_r63b_triangular_inverse(lower, 2U);
    const ALR20R37ExactResidual right = inverse.exact
        ? al_r20_r37_exact_residual(lower, inverse.inverse, 2U)
        : ALR20R37ExactResidual{};
    const ALR20R37ExactResidual left = inverse.exact
        ? al_r20_r37_exact_residual(inverse.inverse, lower, 2U)
        : ALR20R37ExactResidual{};
    std::vector<double> lower_d;
    std::vector<double> inverse_d;
    for (Binary128 value : lower)
        lower_d.push_back(static_cast<double>(value));
    for (Binary128 value : inverse.inverse)
        inverse_d.push_back(static_cast<double>(value));
    const ALR20R37ExactResidual right_d = al_r20_r37_exact_residual(
        al_r20_r63_promote(lower_d), al_r20_r63_promote(inverse_d), 2U);
    const ALR20R37ExactResidual left_d = al_r20_r37_exact_residual(
        al_r20_r63_promote(inverse_d), al_r20_r63_promote(lower_d), 2U);
    const ALR20R63MatrixAudit right_candidate =
        al_r20_r63_matrix_audit(lower_d, inverse_d, right_d);
    const ALR20R63MatrixAudit left_candidate =
        al_r20_r63_matrix_audit(inverse_d, lower_d, left_d);
    result.good_factor = inverse.exact && right.exact && left.exact
        && right.contractive && left.contractive
        && right_candidate.candidate_exact && right_candidate.contained
        && right_candidate.candidate_contractive
        && left_candidate.candidate_exact && left_candidate.contained
        && left_candidate.candidate_contractive;

    const Binary128 tiny = ldexpq(one, -1075);
    const double tiny_projected = static_cast<double>(tiny);
    result.tiny_boundary = tiny > static_cast<Binary128>(0.0)
        && tiny_projected == 0.0;
    result.shrunk_rejected = al_r20_r63_controls().shrunk_rejected;

    const std::vector<Binary128> matrix{
        static_cast<Binary128>(4.0), one,
        one, static_cast<Binary128>(1.25)};
    const ALR20R63BReconstruction good_reconstruction =
        al_r20_r63b_reconstruction(
            matrix, lower, 2U, ldexpq(one, -100));
    std::vector<Binary128> corrupt = lower;
    corrupt[2U] += ldexpq(one, -20);
    const ALR20R63BReconstruction bad_reconstruction =
        al_r20_r63b_reconstruction(
            matrix, corrupt, 2U, ldexpq(one, -100));
    result.reconstruction_rejected = good_reconstruction.exact
        && !bad_reconstruction.exact && !bad_reconstruction.contained;
    result.exact = result.good_factor && result.tiny_boundary
        && result.shrunk_rejected && result.reconstruction_rejected;
    std::ostringstream material;
    material << result.exact << ':' << result.good_factor << ':'
        << result.tiny_boundary << ':' << result.shrunk_rejected << ':'
        << result.reconstruction_rejected << ':' << inverse.root << ':'
        << right_candidate.root << ':' << left_candidate.root << ':'
        << good_reconstruction.root << ':' << bad_reconstruction.root;
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v5_factor_space_controls() {
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
        && std::fegetround() == FE_TONEAREST;
    const ALR20R63BControls controls = al_r20_r63b_controls();
    const bool controls_exact = profile_exact && controls.exact
        && al_r20_r37_dyadic_controls();

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
    ALR20R60Capture solution_capture;
    ALR20VerifiedInverseCapture factor_capture;
    factor_capture.selected_matrix_root = AL_R20_R63_MATRIX_SHA256;
    ALR20R55SlopeTrace slope;
    ALR20R35CaseAudit oblique;
    std::size_t solver_executions = 0U;
    const bool hook_slot_clear = al_r20_verified_solution_refiner == nullptr
        && al_r20_r60_capture == nullptr
        && al_r20_verified_inverse_capture == nullptr
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
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr
        && al_r20_direct_solution_capture == nullptr;
    const bool capture_exact = parent_exact && hook_slot_clear
        && hook_slot_released && solver_executions == 1U
        && al_r20_r35_case_observation_exact(oblique)
        && oblique.root == AL_R20_R60_CASE_SHA256
        && solution_capture.calls == 1U
        && solution_capture.dimension == AL_R20_R63_DIMENSION
        && solution_capture.material_root == AL_R20_R63_MATERIAL_SHA256
        && factor_capture.selected_calls == 1U
        && factor_capture.selected_column_solves == AL_R20_R63_DIMENSION
        && factor_capture.factor_passed
        && factor_capture.matrix_root == solution_capture.matrix_root
        && al_r20_q_scalar_root(factor_capture.inverse)
            == solution_capture.inverse_root
        && factor_capture.rhs == solution_capture.rhs
        && factor_capture.solution == solution_capture.solution
        && factor_capture.lower.size()
            == AL_R20_R63_DIMENSION * AL_R20_R63_DIMENSION
        && slope.calls == 0U && slope.inverse_audits == 0U;

    const ALR20R63BReconstruction reconstruction = capture_exact
        ? al_r20_r63b_reconstruction(factor_capture.matrix,
            factor_capture.lower, AL_R20_R63_DIMENSION,
            factor_capture.factor_maximum_reconstruction_bound)
        : ALR20R63BReconstruction{};
    const ALR20R63BTriangularInverse triangular = capture_exact
        ? al_r20_r63b_triangular_inverse(
            factor_capture.lower, AL_R20_R63_DIMENSION)
        : ALR20R63BTriangularInverse{};
    const ALR20R37ExactResidual exact_right = triangular.exact
        ? al_r20_r37_exact_residual(factor_capture.lower,
            triangular.inverse, AL_R20_R63_DIMENSION)
        : ALR20R37ExactResidual{};
    const ALR20R37ExactResidual exact_left = triangular.exact
        ? al_r20_r37_exact_residual(triangular.inverse,
            factor_capture.lower, AL_R20_R63_DIMENSION)
        : ALR20R37ExactResidual{};

    std::vector<double> lower_d;
    std::vector<double> inverse_d;
    bool projection_exact = exact_right.exact && exact_left.exact;
    if (projection_exact) {
        lower_d.reserve(factor_capture.lower.size());
        inverse_d.reserve(triangular.inverse.size());
        for (Binary128 value : factor_capture.lower) {
            const double projected = static_cast<double>(value);
            projection_exact = projection_exact && std::isfinite(projected);
            lower_d.push_back(projected);
        }
        for (Binary128 value : triangular.inverse) {
            const double projected = static_cast<double>(value);
            projection_exact = projection_exact && std::isfinite(projected);
            inverse_d.push_back(projected);
        }
    }
    const std::string lower_projection_root = al_r20_double_root(lower_d);
    const std::string inverse_projection_root = al_r20_double_root(inverse_d);
    const ALR20R37ExactResidual projected_right = projection_exact
        ? al_r20_r37_exact_residual(al_r20_r63_promote(lower_d),
            al_r20_r63_promote(inverse_d), AL_R20_R63_DIMENSION)
        : ALR20R37ExactResidual{};
    const ALR20R37ExactResidual projected_left = projection_exact
        ? al_r20_r37_exact_residual(al_r20_r63_promote(inverse_d),
            al_r20_r63_promote(lower_d), AL_R20_R63_DIMENSION)
        : ALR20R37ExactResidual{};
    const ALR20R63MatrixAudit candidate_right = projected_right.exact
        ? al_r20_r63_matrix_audit(lower_d, inverse_d, projected_right)
        : ALR20R63MatrixAudit{};
    const ALR20R63MatrixAudit candidate_left = projected_left.exact
        ? al_r20_r63_matrix_audit(inverse_d, lower_d, projected_left)
        : ALR20R63MatrixAudit{};
    const ALR20R37ExactResidual matrix_left = capture_exact
        ? al_r20_r37_exact_residual(factor_capture.inverse,
            factor_capture.matrix, AL_R20_R63_DIMENSION)
        : ALR20R37ExactResidual{};
    const ALR20R63BConditionBound condition = matrix_left.exact
        ? al_r20_r63b_condition_lower(factor_capture.matrix,
            factor_capture.inverse, matrix_left)
        : ALR20R63BConditionBound{};

    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool work_exact = triangular.columns == 102U
        && triangular.multiply_subtracts == 176851U
        && triangular.divisions == 5253U
        && candidate_right.candidate_dots == 10404U
        && candidate_left.candidate_dots == 10404U
        && candidate_right.oracle_dots == 10404U
        && candidate_left.oracle_dots == 10404U;
    const bool exact_factor_contractive = exact_right.exact && exact_left.exact
        && exact_right.contractive && exact_left.contractive;
    const bool correspondence_exact = candidate_right.contained
        && candidate_left.contained
        && candidate_right.contained_entries == 10404U
        && candidate_left.contained_entries == 10404U;
    const bool candidate_decision = candidate_right.candidate_exact
        && candidate_left.candidate_exact
        && candidate_right.candidate_contractive
        && candidate_left.candidate_contractive;
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_exact && capture_exact && reconstruction.exact
        && triangular.exact && exact_factor_contractive && projection_exact
        && projected_right.exact && projected_left.exact
        && condition.exact && work_exact && lifecycle_exact;
    const bool exact = apparatus_exact && correspondence_exact
        && candidate_decision;
    std::string route = "BINARY64_CHOLESKY_FACTOR_SPACE_CANDIDATE";
    if (!profile_exact || !controls_exact)
        route = "FACTOR_SPACE_CONTROL_REJECTED";
    else if (!parent_exact || !capture_exact || !lifecycle_exact)
        route = "FACTOR_SPACE_CAPTURE_REJECTED";
    else if (!reconstruction.exact)
        route = "FACTOR_SPACE_RECONSTRUCTION_REJECTED";
    else if (!triangular.exact)
        route = "FACTOR_SPACE_TRIANGULAR_INVERSE_REJECTED";
    else if (!exact_factor_contractive)
        route = "FACTOR_SPACE_EXACT_NONCONTRACTIVE";
    else if (!projection_exact || !projected_right.exact
        || !projected_left.exact || !condition.exact || !work_exact)
        route = "FACTOR_SPACE_APPARATUS_REJECTED";
    else if (!correspondence_exact)
        route = "FACTOR_SPACE_CONTAINMENT_REJECTED";
    else if (!candidate_right.candidate_contractive)
        route = "BINARY64_FACTOR_RIGHT_NONCONTRACTIVE";
    else if (!candidate_left.candidate_contractive)
        route = "BINARY64_FACTOR_LEFT_NONCONTRACTIVE";

    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r63b-factor-space.v1|"
        << AL_R20_R63B_PARENT_SEMANTIC_SHA256 << ':' << route << ':'
        << oblique.root << ':' << solution_capture.material_root << ':'
        << factor_capture.factor_root << ':' << controls.root << ':'
        << reconstruction.root << ':' << triangular.root << ':'
        << exact_right.root << ':' << exact_left.root << ':'
        << lower_projection_root << ':' << inverse_projection_root << ':'
        << candidate_right.root << ':' << candidate_left.root << ':'
        << condition.root;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63b_factor_space.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63a_sha256\":\""
        << AL_R20_R63B_PARENT_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls\":{\"exact\":"
        << (controls_exact ? "true" : "false")
        << ",\"good_factor\":"
        << (controls.good_factor ? "true" : "false")
        << ",\"tiny_boundary\":"
        << (controls.tiny_boundary ? "true" : "false")
        << ",\"shrunk_rejected\":"
        << (controls.shrunk_rejected ? "true" : "false")
        << ",\"reconstruction_rejected\":"
        << (controls.reconstruction_rejected ? "true" : "false")
        << ",\"root\":\"" << controls.root << "\"}"
        << ",\"capture\":{\"exact\":"
        << (capture_exact ? "true" : "false")
        << ",\"solution_calls\":" << solution_capture.calls
        << ",\"observed_inverse_calls\":" << factor_capture.calls
        << ",\"selected_inverse_calls\":" << factor_capture.selected_calls
        << ",\"selected_column_solves\":"
        << factor_capture.selected_column_solves
        << ",\"dimension\":" << solution_capture.dimension
        << ",\"matrix_root\":\"" << factor_capture.matrix_root
        << "\",\"inverse_root\":\""
        << al_r20_q_scalar_root(factor_capture.inverse)
        << "\",\"lower_root\":\"" << factor_capture.factor_root << "\"}"
        << ",\"reconstruction\":{\"exact\":"
        << (reconstruction.exact ? "true" : "false")
        << ",\"contained\":"
        << (reconstruction.contained ? "true" : "false")
        << ",\"norm_outward\":\""
        << al_binary128_text(reconstruction.norm.outward)
        << "\",\"maximum_entry_outward\":\""
        << al_binary128_text(reconstruction.maximum_entry_outward)
        << "\",\"inherited_bound\":\""
        << al_binary128_text(
            factor_capture.factor_maximum_reconstruction_bound)
        << "\",\"root\":\"" << reconstruction.root << "\"}"
        << ",\"triangular\":{\"exact\":"
        << (triangular.exact ? "true" : "false")
        << ",\"columns\":" << triangular.columns
        << ",\"multiply_subtracts\":" << triangular.multiply_subtracts
        << ",\"divisions\":" << triangular.divisions
        << ",\"minimum_diagonal\":\""
        << al_binary128_text(triangular.minimum_diagonal)
        << "\",\"maximum_diagonal\":\""
        << al_binary128_text(triangular.maximum_diagonal)
        << "\",\"inverse_root\":\""
        << al_r20_q_scalar_root(triangular.inverse)
        << "\",\"root\":\"" << triangular.root << "\"}"
        << ",\"exact_factor\":{\"right_rho\":\""
        << al_binary128_text(exact_right.outward_norm)
        << "\",\"left_rho\":\""
        << al_binary128_text(exact_left.outward_norm)
        << "\",\"right_root\":\"" << exact_right.root
        << "\",\"left_root\":\"" << exact_left.root << "\"}"
        << ",\"projection\":{\"exact\":"
        << (projection_exact ? "true" : "false")
        << ",\"lower_root\":\"" << lower_projection_root
        << "\",\"inverse_root\":\"" << inverse_projection_root << "\"}"
        << ",\"candidate_right\":{\"contained\":"
        << (candidate_right.contained ? "true" : "false")
        << ",\"contractive\":"
        << (candidate_right.candidate_contractive ? "true" : "false")
        << ",\"rho\":" << candidate_right.rho_bound
        << ",\"exact_rho_outward\":\""
        << al_binary128_text(projected_right.outward_norm)
        << "\",\"contained_entries\":"
        << candidate_right.contained_entries
        << ",\"dots\":" << candidate_right.candidate_dots
        << ",\"candidate_root\":\"" << candidate_right.candidate_root
        << "\",\"oracle_root\":\"" << candidate_right.oracle_root << "\"}"
        << ",\"candidate_left\":{\"contained\":"
        << (candidate_left.contained ? "true" : "false")
        << ",\"contractive\":"
        << (candidate_left.candidate_contractive ? "true" : "false")
        << ",\"rho\":" << candidate_left.rho_bound
        << ",\"exact_rho_outward\":\""
        << al_binary128_text(projected_left.outward_norm)
        << "\",\"contained_entries\":"
        << candidate_left.contained_entries
        << ",\"dots\":" << candidate_left.candidate_dots
        << ",\"candidate_root\":\"" << candidate_left.candidate_root
        << "\",\"oracle_root\":\"" << candidate_left.oracle_root << "\"}"
        << ",\"condition_lower_bound\":{\"exact\":"
        << (condition.exact ? "true" : "false")
        << ",\"value\":\"" << al_binary128_text(condition.lower)
        << "\",\"unit_times_value\":\""
        << al_binary128_text(condition.unit_lower)
        << "\",\"root\":\"" << condition.root << "\"}"
        << ",\"candidate_decision\":"
        << (candidate_decision ? "true" : "false")
        << ",\"correspondence_exact\":"
        << (correspondence_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"nnqp_rhs_solves\":0,\"center_iterations\":0"
        << ",\"replacement_applies\":0,\"state_updates\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"gpu_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
