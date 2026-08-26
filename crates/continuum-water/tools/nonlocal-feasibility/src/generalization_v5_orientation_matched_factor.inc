namespace {

constexpr const char* AL_R20_R63C_PARENT_SEMANTIC_SHA256 =
    "28bfb856cf7b537b0c00acf39626f703c1007c7e59e20707aca58b4e2ae160fc";
constexpr const char* AL_R20_R63C_FACTOR_SHA256 =
    "782588f764834bf498b2a25ad4ab20e84083b51a3d9016786e0c411227c70358";

struct ALR20R63CInverse {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t systems = 0U;
    std::size_t fmas = 0U;
    std::size_t divisions = 0U;
    double minimum_diagonal = 0.0;
    double maximum_diagonal = 0.0;
    std::vector<double> values;
    std::string root;
};

ALR20R63CInverse al_r20_r63c_column_right_inverse(
    const std::vector<double>& lower, std::size_t dimension) {
    ALR20R63CInverse result;
    result.dimension = dimension;
    if (dimension == 0U || lower.size() != dimension * dimension)
        return result;
    result.values.assign(dimension * dimension, 0.0);
    bool exact = true;
    for (std::size_t column = 0U; column < dimension && exact; ++column) {
        for (std::size_t row = column; row < dimension; ++row) {
            double value = row == column ? 1.0 : 0.0;
            for (std::size_t inner = column; inner < row; ++inner) {
                value = std::fma(
                    -lower[row * dimension + inner],
                    result.values[inner * dimension + column], value);
                ++result.fmas;
            }
            const double diagonal = lower[row * dimension + row];
            exact = std::isfinite(value) && std::isfinite(diagonal)
                && diagonal > 0.0;
            if (!exact) break;
            value /= diagonal;
            exact = std::isfinite(value);
            if (!exact) break;
            result.values[row * dimension + column] = value;
            ++result.divisions;
            if (row == column) {
                if (column == 0U || diagonal < result.minimum_diagonal)
                    result.minimum_diagonal = diagonal;
                result.maximum_diagonal = std::max(
                    result.maximum_diagonal, diagonal);
            }
        }
        ++result.systems;
    }
    result.exact = exact && result.systems == dimension
        && result.fmas
            == dimension * (dimension - 1U) * (dimension + 1U) / 6U
        && result.divisions == dimension * (dimension + 1U) / 2U;
    std::ostringstream material;
    material << result.exact << ':' << dimension << ':' << result.systems
        << ':' << result.fmas << ':' << result.divisions << ':'
        << binary64_bits(result.minimum_diagonal) << ':'
        << binary64_bits(result.maximum_diagonal) << ':'
        << al_r20_double_root(result.values);
    result.root = sha256_hex(material.str());
    return result;
}

ALR20R63CInverse al_r20_r63c_row_left_inverse(
    const std::vector<double>& lower, std::size_t dimension) {
    ALR20R63CInverse result;
    result.dimension = dimension;
    if (dimension == 0U || lower.size() != dimension * dimension)
        return result;
    result.values.assign(dimension * dimension, 0.0);
    std::vector<double> solution(dimension, 0.0);
    bool exact = true;
    for (std::size_t system = 0U; system < dimension && exact; ++system) {
        std::fill(solution.begin(), solution.end(), 0.0);
        for (std::size_t reverse = 0U; reverse <= system; ++reverse) {
            const std::size_t row = system - reverse;
            double value = row == system ? 1.0 : 0.0;
            for (std::size_t inner = row + 1U;
                 inner <= system; ++inner) {
                value = std::fma(
                    -lower[inner * dimension + row],
                    solution[inner], value);
                ++result.fmas;
            }
            const double diagonal = lower[row * dimension + row];
            exact = std::isfinite(value) && std::isfinite(diagonal)
                && diagonal > 0.0;
            if (!exact) break;
            value /= diagonal;
            exact = std::isfinite(value);
            if (!exact) break;
            solution[row] = value;
            ++result.divisions;
        }
        if (!exact) break;
        for (std::size_t column = 0U; column < dimension; ++column)
            result.values[system * dimension + column] = solution[column];
        ++result.systems;
    }
    for (std::size_t index = 0U; index < dimension; ++index) {
        const double diagonal = lower[index * dimension + index];
        if (index == 0U || diagonal < result.minimum_diagonal)
            result.minimum_diagonal = diagonal;
        result.maximum_diagonal = std::max(
            result.maximum_diagonal, diagonal);
    }
    result.exact = exact && result.systems == dimension
        && result.fmas
            == dimension * (dimension - 1U) * (dimension + 1U) / 6U
        && result.divisions == dimension * (dimension + 1U) / 2U;
    std::ostringstream material;
    material << result.exact << ':' << dimension << ':' << result.systems
        << ':' << result.fmas << ':' << result.divisions << ':'
        << binary64_bits(result.minimum_diagonal) << ':'
        << binary64_bits(result.maximum_diagonal) << ':'
        << al_r20_double_root(result.values);
    result.root = sha256_hex(material.str());
    return result;
}

std::vector<double> al_r20_r63c_transpose(
    const std::vector<double>& matrix, std::size_t dimension) {
    std::vector<double> result;
    if (matrix.size() != dimension * dimension) return result;
    result.resize(matrix.size());
    for (std::size_t row = 0U; row < dimension; ++row)
        for (std::size_t column = 0U; column < dimension; ++column)
            result[column * dimension + row] =
                matrix[row * dimension + column];
    return result;
}

struct ALR20R63CControls {
    bool exact = false;
    bool good_factor = false;
    bool zero_diagonal_rejected = false;
    bool shrunk_rejected = false;
    bool mutated_root_rejected = false;
    std::string root;
};

ALR20R63CControls al_r20_r63c_controls() {
    ALR20R63CControls result;
    const std::vector<double> lower{2.0, 0.0, 0.5, 1.0};
    const ALR20R63CInverse right =
        al_r20_r63c_column_right_inverse(lower, 2U);
    const ALR20R63CInverse left =
        al_r20_r63c_row_left_inverse(lower, 2U);
    const std::vector<double> right_t =
        al_r20_r63c_transpose(right.values, 2U);
    const std::vector<double> lower_t =
        al_r20_r63c_transpose(lower, 2U);
    const ALR20R37ExactResidual lower_left_oracle =
        al_r20_r37_exact_residual(al_r20_r63_promote(left.values),
            al_r20_r63_promote(lower), 2U);
    const ALR20R37ExactResidual upper_left_oracle =
        al_r20_r37_exact_residual(al_r20_r63_promote(right_t),
            al_r20_r63_promote(lower_t), 2U);
    const ALR20R63MatrixAudit lower_left = al_r20_r63_matrix_audit(
        left.values, lower, lower_left_oracle);
    const ALR20R63MatrixAudit upper_left = al_r20_r63_matrix_audit(
        right_t, lower_t, upper_left_oracle);
    result.good_factor = right.exact && left.exact
        && lower_left.candidate_exact && lower_left.contained
        && lower_left.candidate_contractive
        && upper_left.candidate_exact && upper_left.contained
        && upper_left.candidate_contractive;

    const Binary128 tiny = ldexpq(static_cast<Binary128>(1.0), -1075);
    const std::vector<double> zero_diagonal{
        1.0, 0.0, 0.0, static_cast<double>(tiny)};
    const ALR20R63CInverse rejected =
        al_r20_r63c_column_right_inverse(zero_diagonal, 2U);
    result.zero_diagonal_rejected = tiny > static_cast<Binary128>(0.0)
        && zero_diagonal[3U] == 0.0 && !rejected.exact;
    result.shrunk_rejected = al_r20_r63_controls().shrunk_rejected;

    const std::string bound_root = al_r20_double_root(lower);
    std::vector<double> mutated = lower;
    mutated[2U] = std::nextafter(
        mutated[2U], std::numeric_limits<double>::infinity());
    result.mutated_root_rejected =
        al_r20_double_root(mutated) != bound_root;
    result.exact = result.good_factor && result.zero_diagonal_rejected
        && result.shrunk_rejected && result.mutated_root_rejected;
    std::ostringstream material;
    material << result.exact << ':' << result.good_factor << ':'
        << result.zero_diagonal_rejected << ':'
        << result.shrunk_rejected << ':'
        << result.mutated_root_rejected << ':' << right.root << ':'
        << left.root << ':' << lower_left.root << ':' << upper_left.root
        << ':' << rejected.root << ':' << bound_root << ':'
        << al_r20_double_root(mutated);
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

SplitBoundaryReport
run_al_generalization_v5_orientation_matched_factor_controls() {
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
    const ALR20R63CControls controls = al_r20_r63c_controls();
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
        && factor_capture.factor_root == AL_R20_R63C_FACTOR_SHA256
        && factor_capture.matrix_root == solution_capture.matrix_root
        && slope.calls == 0U && slope.inverse_audits == 0U;

    std::vector<double> lower;
    bool projection_exact = capture_exact;
    if (capture_exact) {
        lower.reserve(factor_capture.lower.size());
        for (Binary128 value : factor_capture.lower) {
            const double projected = static_cast<double>(value);
            projection_exact = projection_exact && std::isfinite(projected);
            lower.push_back(projected);
        }
    }
    const std::string lower_root = al_r20_double_root(lower);
    const ALR20R63CInverse column_right = projection_exact
        ? al_r20_r63c_column_right_inverse(lower, AL_R20_R63_DIMENSION)
        : ALR20R63CInverse{};
    const ALR20R63CInverse row_left = projection_exact
        ? al_r20_r63c_row_left_inverse(lower, AL_R20_R63_DIMENSION)
        : ALR20R63CInverse{};
    const std::vector<double> lower_t =
        al_r20_r63c_transpose(lower, AL_R20_R63_DIMENSION);
    const std::vector<double> column_right_t =
        al_r20_r63c_transpose(
            column_right.values, AL_R20_R63_DIMENSION);

    const ALR20R37ExactResidual lower_left_oracle = row_left.exact
        ? al_r20_r37_exact_residual(al_r20_r63_promote(row_left.values),
            al_r20_r63_promote(lower), AL_R20_R63_DIMENSION)
        : ALR20R37ExactResidual{};
    const ALR20R37ExactResidual lower_right_oracle = row_left.exact
        ? al_r20_r37_exact_residual(al_r20_r63_promote(lower),
            al_r20_r63_promote(row_left.values), AL_R20_R63_DIMENSION)
        : ALR20R37ExactResidual{};
    const ALR20R37ExactResidual upper_left_oracle = column_right.exact
        ? al_r20_r37_exact_residual(
            al_r20_r63_promote(column_right_t),
            al_r20_r63_promote(lower_t), AL_R20_R63_DIMENSION)
        : ALR20R37ExactResidual{};
    const ALR20R37ExactResidual column_left_oracle = column_right.exact
        ? al_r20_r37_exact_residual(
            al_r20_r63_promote(column_right.values),
            al_r20_r63_promote(lower), AL_R20_R63_DIMENSION)
        : ALR20R37ExactResidual{};

    const ALR20R63MatrixAudit lower_left = lower_left_oracle.exact
        ? al_r20_r63_matrix_audit(
            row_left.values, lower, lower_left_oracle)
        : ALR20R63MatrixAudit{};
    const ALR20R63MatrixAudit lower_right = lower_right_oracle.exact
        ? al_r20_r63_matrix_audit(
            lower, row_left.values, lower_right_oracle)
        : ALR20R63MatrixAudit{};
    const ALR20R63MatrixAudit upper_left = upper_left_oracle.exact
        ? al_r20_r63_matrix_audit(
            column_right_t, lower_t, upper_left_oracle)
        : ALR20R63MatrixAudit{};
    const ALR20R63MatrixAudit column_left = column_left_oracle.exact
        ? al_r20_r63_matrix_audit(
            column_right.values, lower, column_left_oracle)
        : ALR20R63MatrixAudit{};

    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool work_exact = column_right.systems == 102U
        && row_left.systems == 102U
        && column_right.fmas == 176851U && row_left.fmas == 176851U
        && column_right.divisions == 5253U && row_left.divisions == 5253U
        && lower_left.candidate_dots == 10404U
        && lower_right.candidate_dots == 10404U
        && upper_left.candidate_dots == 10404U
        && column_left.candidate_dots == 10404U
        && lower_left.oracle_dots == 10404U
        && lower_right.oracle_dots == 10404U
        && upper_left.oracle_dots == 10404U
        && column_left.oracle_dots == 10404U;
    const bool correspondence_exact = lower_left.contained
        && lower_right.contained && upper_left.contained
        && column_left.contained
        && lower_left.contained_entries == 10404U
        && lower_right.contained_entries == 10404U
        && upper_left.contained_entries == 10404U
        && column_left.contained_entries == 10404U;
    const bool candidate_decision = lower_left.candidate_exact
        && upper_left.candidate_exact
        && lower_left.candidate_contractive
        && upper_left.candidate_contractive;
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_exact && capture_exact && projection_exact
        && column_right.exact && row_left.exact
        && lower_left_oracle.exact && lower_right_oracle.exact
        && upper_left_oracle.exact && column_left_oracle.exact
        && lower_left.candidate_exact && lower_right.candidate_exact
        && upper_left.candidate_exact && column_left.candidate_exact
        && work_exact && lifecycle_exact;
    const bool exact = apparatus_exact && correspondence_exact
        && candidate_decision;

    std::string route = "BINARY64_ORIENTATION_MATCHED_FACTORS_CANDIDATE";
    if (!profile_exact || !controls_exact)
        route = "ORIENTATION_FACTOR_CONTROL_REJECTED";
    else if (!parent_exact || !capture_exact || !lifecycle_exact)
        route = "ORIENTATION_FACTOR_CAPTURE_REJECTED";
    else if (!projection_exact)
        route = "ORIENTATION_FACTOR_PROJECTION_REJECTED";
    else if (!column_right.exact)
        route = "ORIENTATION_FACTOR_COLUMN_CONSTRUCTION_REJECTED";
    else if (!row_left.exact)
        route = "ORIENTATION_FACTOR_ROW_CONSTRUCTION_REJECTED";
    else if (!work_exact)
        route = "ORIENTATION_FACTOR_WORK_REJECTED";
    else if (!correspondence_exact)
        route = "ORIENTATION_FACTOR_CONTAINMENT_REJECTED";
    else if (!lower_left.candidate_contractive)
        route = "BINARY64_LOWER_LEFT_NONCONTRACTIVE";
    else if (!upper_left.candidate_contractive)
        route = "BINARY64_UPPER_TRANSPOSE_LEFT_NONCONTRACTIVE";

    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63c-orientation-factor.v1|"
        << AL_R20_R63C_PARENT_SEMANTIC_SHA256 << ':' << route << ':'
        << oblique.root << ':' << factor_capture.factor_root << ':'
        << controls.root << ':' << lower_root << ':'
        << column_right.root << ':' << row_left.root << ':'
        << lower_left.root << ':' << lower_right.root << ':'
        << upper_left.root << ':' << column_left.root;
    const std::string result_sha = sha256_hex(semantic.str());

    auto audit_json = [](std::ostringstream& report, const char* name,
                         const ALR20R63MatrixAudit& audit,
                         const ALR20R37ExactResidual& oracle) {
        report << ",\"" << name << "\":{\"contained\":"
            << (audit.contained ? "true" : "false")
            << ",\"contractive\":"
            << (audit.candidate_contractive ? "true" : "false")
            << ",\"rho\":" << audit.rho_bound
            << ",\"exact_rho_outward\":\""
            << al_binary128_text(oracle.outward_norm)
            << "\",\"contained_entries\":" << audit.contained_entries
            << ",\"dots\":" << audit.candidate_dots
            << ",\"candidate_root\":\"" << audit.candidate_root
            << "\",\"oracle_root\":\"" << audit.oracle_root << "\"}";
    };

    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63c_orientation_factor.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63b_sha256\":\""
        << AL_R20_R63C_PARENT_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls\":{\"exact\":"
        << (controls_exact ? "true" : "false")
        << ",\"good_factor\":"
        << (controls.good_factor ? "true" : "false")
        << ",\"zero_diagonal_rejected\":"
        << (controls.zero_diagonal_rejected ? "true" : "false")
        << ",\"shrunk_rejected\":"
        << (controls.shrunk_rejected ? "true" : "false")
        << ",\"mutated_root_rejected\":"
        << (controls.mutated_root_rejected ? "true" : "false")
        << ",\"root\":\"" << controls.root << "\"}"
        << ",\"capture\":{\"exact\":"
        << (capture_exact ? "true" : "false")
        << ",\"observed_inverse_calls\":" << factor_capture.calls
        << ",\"selected_inverse_calls\":" << factor_capture.selected_calls
        << ",\"selected_column_solves\":"
        << factor_capture.selected_column_solves
        << ",\"dimension\":" << solution_capture.dimension
        << ",\"factor_root\":\"" << factor_capture.factor_root
        << "\",\"projected_root\":\"" << lower_root << "\"}"
        << ",\"column_right\":{\"exact\":"
        << (column_right.exact ? "true" : "false")
        << ",\"systems\":" << column_right.systems
        << ",\"fmas\":" << column_right.fmas
        << ",\"divisions\":" << column_right.divisions
        << ",\"root\":\"" << column_right.root << "\"}"
        << ",\"row_left\":{\"exact\":"
        << (row_left.exact ? "true" : "false")
        << ",\"systems\":" << row_left.systems
        << ",\"fmas\":" << row_left.fmas
        << ",\"divisions\":" << row_left.divisions
        << ",\"root\":\"" << row_left.root << "\"}";
    audit_json(report, "required_lower_left", lower_left, lower_left_oracle);
    audit_json(report, "report_lower_right", lower_right, lower_right_oracle);
    audit_json(report, "required_upper_transpose_left",
        upper_left, upper_left_oracle);
    audit_json(report, "report_column_left", column_left, column_left_oracle);
    report << ",\"candidate_decision\":"
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
