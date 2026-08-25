namespace {

constexpr const char* AL_R20_PROJECTOR_SHA256 =
    "18d3d90b0f3994bb4fac9d4cddeef961a93830c88c82818766de0f47c6209a4d";
constexpr std::array<const char*, 8U> AL_R20_FACE_MATRIX_SHA256{{
    "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9",
    "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9",
    "54857cb1199df632fcc603a91310ece3d199083ec8eee5cf1bfdb952863b3bd3",
    "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9",
    "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9",
    "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9",
    "34d926b3ae3e1723e33a6b4648fc7cd94b888225fb9e2cfbd238b61299de147d",
    "b2e62a0cd858fe111f909f2f8c5bb0ce6cb8ed3f9c7049bf8cf7e72d413f6af8",
}};
constexpr std::size_t AL_R20_NEWTON_LINE_TRIALS = 21U;

struct ALR20NewtonDualEvaluation {
    bool exact = false;
    Binary128 value = static_cast<Binary128>(0.0);
    Binary128 bound = static_cast<Binary128>(0.0);
    ALR20ProjectorState projector;
    std::vector<Binary128> residual;
    std::vector<Binary128> residual_bound;
    std::string root;
};

ALR20NewtonDualEvaluation al_r20_newton_dual_evaluate(
    const ALR20ProblemData& problem,
    const std::vector<Binary128>& lambda) {
    ALR20NewtonDualEvaluation result;
    if (lambda.size() != problem.row_scale.size()
        || !al_r20_admm_finite(lambda)) return result;
    const std::vector<Binary128Vec3> target = al_r20_q_promote(problem.target);
    const std::vector<Binary128Vec3> at = al_r20_q_transpose(
        problem.op, lambda, target.size());
    std::vector<Binary128Vec3> input(target.size());
    for (std::size_t particle = 0U; particle < target.size(); ++particle)
        input[particle] = target[particle] - at[particle];
    result.projector = al_r20_projector_state(input,
        al_r20_projector_promote_bounds(problem.lower),
        al_r20_projector_promote_bounds(problem.upper),
        static_cast<Binary128>(AL_R20_TRUST_RADIUS));
    result.residual = al_r20_q_action(problem.op, result.projector.value,
        problem.workspace.support.constraint, &result.residual_bound);
    Binary128Accumulator dual;
    Binary128 absolute = static_cast<Binary128>(0.0);
    for (std::size_t particle = 0U; particle < target.size(); ++particle) {
        const Binary128Vec3 delta =
            result.projector.value[particle] - target[particle];
        const Binary128 term = static_cast<Binary128>(0.5)
            * al_r20_q_dot(delta, delta);
        dual.add(term);
        absolute += fabsq(term);
    }
    for (std::size_t row = 0U; row < lambda.size(); ++row) {
        const Binary128 term = lambda[row] * result.residual[row];
        dual.add(term);
        absolute += fabsq(term) + fabsq(lambda[row])
            * result.residual_bound[row];
    }
    result.value = dual.value();
    const Binary128 state_scale = static_cast<Binary128>(1.0)
        + al_r20_projector_vector_norm(input)
        + al_r20_projector_vector_norm(result.projector.value);
    result.bound = al_r20_q_gamma(
        96U * target.size() + 48U * lambda.size() + 8192U)
        * fmaxq(absolute, ldexpq(static_cast<Binary128>(1.0), -16382))
        + result.projector.kkt_bound * state_scale;
    result.exact = result.projector.exact
        && al_r20_admm_finite(result.residual)
        && al_r20_admm_finite(result.residual_bound)
        && finiteq(result.value) != 0 && finiteq(result.bound) != 0;
    std::ostringstream material;
    material << result.exact << ':' << al_binary128_hex(result.value) << ':'
             << al_binary128_hex(result.bound) << ':'
             << result.projector.root << ':'
             << al_r20_q_scalar_root(result.residual) << ':'
             << al_r20_q_scalar_root(result.residual_bound) << ':'
             << al_r20_q_scalar_root(lambda);
    result.root = sha256_hex(material.str());
    return result;
}

Binary128 al_r20_newton_direction_error_bound(
    const ALR20ADMMFactor& factor,
    Binary128 residual, Binary128 residual_bound,
    const std::vector<Binary128>& direction) {
    const std::size_t dimension = factor.dimension;
    if (!factor.passed || dimension == 0U)
        return HUGE_VALQ;
    std::vector<Binary128> inverse_row_bound(dimension,
        static_cast<Binary128>(0.0));
    Binary128 maximum_inverse_row = static_cast<Binary128>(0.0);
    for (std::size_t row = 0U; row < dimension; ++row) {
        Binary128Accumulator numerator;
        numerator.add(static_cast<Binary128>(1.0));
        for (std::size_t column = 0U; column < row; ++column)
            numerator.add(fabsq(factor.lower[row * dimension + column])
                * inverse_row_bound[column]);
        inverse_row_bound[row] = numerator.value()
            / factor.lower[row * dimension + row];
        maximum_inverse_row = fmaxq(
            maximum_inverse_row, inverse_row_bound[row]);
    }
    const Binary128 inflation = static_cast<Binary128>(1.0)
        + al_r20_q_gamma(32U * dimension * dimension + 4096U);
    maximum_inverse_row *= inflation;
    const Binary128 eigenvalue_lower = static_cast<Binary128>(1.0)
        / (static_cast<Binary128>(dimension)
            * maximum_inverse_row * maximum_inverse_row);
    const Binary128 residual_norm_bound = sqrtq(
        static_cast<Binary128>(dimension))
        * (residual + residual_bound);
    Binary128 direction_scale = static_cast<Binary128>(1.0);
    for (Binary128 item : direction)
        direction_scale = fmaxq(direction_scale, fabsq(item));
    return residual_norm_bound / eigenvalue_lower
        + al_r20_q_gamma(64U * dimension * dimension + 8192U)
            * direction_scale;
}

bool al_r20_newton_armijo(
    Binary128 old_value, Binary128 old_bound,
    Binary128 new_value, Binary128 new_bound,
    Binary128 alpha, Binary128 slope, Binary128 slope_bound) {
    const Binary128 slope_lower = slope - slope_bound;
    if (!(slope_lower > static_cast<Binary128>(0.0))) return false;
    const Binary128 coefficient = ldexpq(static_cast<Binary128>(1.0), -10);
    const Binary128 rhs = old_value + old_bound
        + coefficient * alpha * slope_lower;
    const Binary128 rhs_bound = al_r20_q_gamma(128U)
        * (fabsq(old_value) + old_bound
            + coefficient * alpha * fabsq(slope_lower));
    return new_value - new_bound >= rhs + rhs_bound;
}

struct ALR20NewtonTrial {
    bool exact = false;
    bool accepted = false;
    std::size_t power = 0U;
    Binary128 alpha = static_cast<Binary128>(0.0);
    Binary128 dual_increase_lower = static_cast<Binary128>(0.0);
    std::size_t mask_changes = 0U;
    bool ball_activity_changed = false;
    ALR20NewtonDualEvaluation dual;
    ALR20OracleMetrics metrics;
    std::string root;
};

struct ALR20NewtonDirectionCase {
    bool exact = false;
    bool quiet = false;
    bool solved = false;
    bool direction_feasible = false;
    bool slope_positive = false;
    bool accepted = false;
    std::size_t positive_rows = 0U;
    std::size_t negative_components = 0U;
    std::size_t uncertain_components = 0U;
    Binary128 solve_residual = static_cast<Binary128>(0.0);
    Binary128 solve_bound = static_cast<Binary128>(0.0);
    Binary128 direction_error_bound = static_cast<Binary128>(0.0);
    Binary128 minimum_direction = static_cast<Binary128>(0.0);
    Binary128 slope = static_cast<Binary128>(0.0);
    Binary128 slope_bound = static_cast<Binary128>(0.0);
    ALR20NewtonDualEvaluation base;
    ALR20OracleMetrics base_metrics;
    ALR20ADMMFactor factor;
    ALR20ADMMWork work;
    std::vector<Binary128> direction;
    std::vector<ALR20NewtonTrial> trials;
    ALR20NewtonTrial selected;
    std::string active_rows_root;
    std::string face_matrix_root;
    std::string root;
};

ALR20NewtonDirectionCase al_r20_newton_direction_case(
    const ALR20ProblemData& problem, std::size_t case_index) {
    ALR20NewtonDirectionCase result;
    const std::size_t rows_count = problem.row_scale.size();
    const std::vector<Binary128> zero_lambda(rows_count,
        static_cast<Binary128>(0.0));
    result.base = al_r20_newton_dual_evaluate(problem, zero_lambda);
    const std::vector<Binary128> diagonal = al_r20_q_diagonal(problem.op);
    result.base_metrics = al_r20_q_metrics(problem,
        result.base.projector.value, zero_lambda, diagonal, 0U);
    std::vector<std::size_t> active_rows;
    for (std::size_t row = 0U; row < result.base.residual.size(); ++row)
        if (result.base.residual[row] > result.base.residual_bound[row])
            active_rows.push_back(row);
    result.positive_rows = active_rows.size();
    std::ostringstream active_material;
    for (std::size_t row : active_rows) active_material << row << ',';
    result.active_rows_root = sha256_hex(active_material.str());
    result.quiet = active_rows.empty();
    if (result.quiet) {
        result.exact = result.base.exact && result.base_metrics.certified
            && case_index < AL_R20_FACE_MATRIX_SHA256.size()
            && al_r20_q_scalar_root(std::vector<Binary128>{})
                == AL_R20_FACE_MATRIX_SHA256[case_index];
        result.direction_feasible = true;
        result.slope_positive = true;
        result.face_matrix_root =
            al_r20_q_scalar_root(std::vector<Binary128>{});
        result.root = sha256_hex(result.base.root + ':'
            + result.base_metrics.root + ':' + result.active_rows_root
            + ':' + result.face_matrix_root);
        return result;
    }

    std::vector<std::vector<Binary128Vec3>> row_vectors;
    std::vector<std::vector<Binary128Vec3>> images;
    row_vectors.reserve(active_rows.size());
    images.reserve(active_rows.size());
    for (std::size_t row : active_rows) {
        row_vectors.push_back(al_r20_projector_row_vector(
            problem.op, row, problem.target.size()));
        images.push_back(al_r20_projector_jvp(
            result.base.projector, row_vectors.back()));
    }
    const std::size_t dimension = active_rows.size();
    std::vector<Binary128> raw_matrix(dimension * dimension,
        static_cast<Binary128>(0.0));
    std::vector<Binary128> matrix(dimension * dimension,
        static_cast<Binary128>(0.0));
    Binary128 symmetry_residual = static_cast<Binary128>(0.0);
    Binary128 matrix_scale = static_cast<Binary128>(0.0);
    for (std::size_t row = 0U; row < dimension; ++row)
        for (std::size_t column = 0U; column < dimension; ++column) {
            raw_matrix[row * dimension + column] =
                al_r20_projector_vector_dot(
                    row_vectors[row], images[column]);
            matrix_scale = fmaxq(matrix_scale,
                fabsq(raw_matrix[row * dimension + column]));
        }
    for (std::size_t row = 0U; row < dimension; ++row)
        for (std::size_t column = 0U; column < dimension; ++column) {
            symmetry_residual = fmaxq(symmetry_residual,
                fabsq(raw_matrix[row * dimension + column]
                    - raw_matrix[column * dimension + row]));
            matrix[row * dimension + column] = static_cast<Binary128>(0.5)
                * (raw_matrix[row * dimension + column]
                    + raw_matrix[column * dimension + row]);
        }
    result.face_matrix_root = al_r20_q_scalar_root(raw_matrix);
    const Binary128 symmetry_bound = al_r20_q_gamma(
        128U * problem.target.size() + 8192U)
        * fmaxq(matrix_scale,
            ldexpq(static_cast<Binary128>(1.0), -16382));
    const bool face_exact = case_index < AL_R20_FACE_MATRIX_SHA256.size()
        && result.face_matrix_root == AL_R20_FACE_MATRIX_SHA256[case_index]
        && symmetry_residual <= symmetry_bound;
    result.factor = al_r20_admm_factor_matrix(
        std::move(matrix), dimension, &result.work);
    std::vector<Binary128> rhs(dimension);
    for (std::size_t index = 0U; index < dimension; ++index)
        rhs[index] = result.base.residual[active_rows[index]];
    result.solved = result.factor.passed && al_r20_admm_solve(
        result.factor, rhs, &result.direction,
        &result.solve_residual, &result.solve_bound, &result.work);
    result.direction_error_bound = result.solved
        ? al_r20_newton_direction_error_bound(result.factor,
            result.solve_residual, result.solve_bound, result.direction)
        : HUGE_VALQ;
    result.minimum_direction = result.direction.empty()
        ? static_cast<Binary128>(0.0) : result.direction.front();
    for (Binary128 item : result.direction) {
        result.minimum_direction = fminq(result.minimum_direction, item);
        if (item + result.direction_error_bound
                < static_cast<Binary128>(0.0))
            ++result.negative_components;
        else if (item - result.direction_error_bound
                < static_cast<Binary128>(0.0))
            ++result.uncertain_components;
    }
    result.direction_feasible = result.solved
        && result.negative_components == 0U
        && result.uncertain_components == 0U;
    Binary128Accumulator slope;
    Binary128 slope_absolute = static_cast<Binary128>(0.0);
    for (std::size_t index = 0U; index < dimension; ++index) {
        const Binary128 term = rhs[index] * result.direction[index];
        slope.add(term);
        slope_absolute += fabsq(term);
        result.slope_bound +=
            result.base.residual_bound[active_rows[index]]
                * (fabsq(result.direction[index])
                    + result.direction_error_bound)
            + fabsq(rhs[index]) * result.direction_error_bound;
    }
    result.slope = slope.value();
    result.slope_bound += al_r20_q_gamma(16U * dimension + 512U)
        * slope_absolute;
    result.slope_positive = result.solved
        && result.slope - result.slope_bound
            > static_cast<Binary128>(0.0);

    bool line_exact = true;
    if (result.direction_feasible && result.slope_positive) {
        for (std::size_t power = 0U;
             power < AL_R20_NEWTON_LINE_TRIALS; ++power) {
            ALR20NewtonTrial trial;
            trial.power = power;
            trial.alpha = ldexpq(static_cast<Binary128>(1.0),
                -static_cast<int>(power));
            std::vector<Binary128> lambda(rows_count,
                static_cast<Binary128>(0.0));
            for (std::size_t index = 0U; index < dimension; ++index)
                lambda[active_rows[index]] =
                    trial.alpha * result.direction[index];
            trial.dual = al_r20_newton_dual_evaluate(problem, lambda);
            trial.exact = trial.dual.exact;
            trial.dual_increase_lower = trial.dual.value - trial.dual.bound
                - result.base.value - result.base.bound;
            trial.ball_activity_changed =
                trial.dual.projector.ball_active
                != result.base.projector.ball_active;
            for (std::size_t scalar = 0U;
                 scalar < result.base.projector.mask.size(); ++scalar)
                if (trial.dual.projector.mask[scalar]
                        != result.base.projector.mask[scalar])
                    ++trial.mask_changes;
            trial.accepted = trial.exact && al_r20_newton_armijo(
                result.base.value, result.base.bound,
                trial.dual.value, trial.dual.bound,
                trial.alpha, result.slope, result.slope_bound);
            trial.metrics = al_r20_q_metrics(problem,
                trial.dual.projector.value, lambda, diagonal, 1U);
            trial.exact = trial.exact && trial.metrics.finite;
            std::ostringstream trial_material;
            trial_material << trial.exact << ':' << trial.accepted << ':'
                << power << ':' << al_binary128_hex(trial.alpha) << ':'
                << al_binary128_hex(trial.dual_increase_lower) << ':'
                << trial.mask_changes << ':' << trial.ball_activity_changed
                << ':' << trial.dual.root << ':' << trial.metrics.root;
            trial.root = sha256_hex(trial_material.str());
            line_exact = line_exact && trial.exact;
            result.trials.push_back(trial);
            if (trial.accepted) {
                result.accepted = true;
                result.selected = trial;
                break;
            }
        }
    }
    result.exact = result.base.exact && result.base_metrics.finite
        && face_exact && result.solved && line_exact;
    std::ostringstream material;
    material << result.exact << ':' << result.solved << ':'
             << result.direction_feasible << ':' << result.slope_positive
             << ':' << result.accepted << ':' << result.positive_rows << ':'
             << result.negative_components << ':'
             << result.uncertain_components << ':'
             << al_binary128_hex(result.solve_residual) << ':'
             << al_binary128_hex(result.solve_bound) << ':'
             << al_binary128_hex(result.direction_error_bound) << ':'
             << al_binary128_hex(result.minimum_direction) << ':'
             << al_binary128_hex(result.slope) << ':'
             << al_binary128_hex(result.slope_bound) << ':'
             << result.base.root << ':' << result.base_metrics.root << ':'
             << result.factor.factor_root << ':'
             << al_r20_q_scalar_root(result.direction) << ':'
             << result.active_rows_root << ':' << result.face_matrix_root << '|';
    for (const ALR20NewtonTrial& trial : result.trials)
        material << trial.root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

bool al_r20_newton_direction_controls() {
    const Binary128 zero = static_cast<Binary128>(0.0);
    const Binary128 one = static_cast<Binary128>(1.0);
    ALR20ADMMWork work;
    const ALR20ADMMFactor factor = al_r20_admm_factor_matrix(
        {static_cast<Binary128>(4.0), one,
         one, static_cast<Binary128>(3.0)}, 2U, &work);
    std::vector<Binary128> direction;
    Binary128 residual = zero;
    Binary128 bound = zero;
    const bool solved = al_r20_admm_solve(factor, {one, one},
        &direction, &residual, &bound, &work);
    const Binary128 error = solved
        ? al_r20_newton_direction_error_bound(
            factor, residual, bound, direction) : HUGE_VALQ;
    const bool direction_control = solved && direction.size() == 2U
        && direction[0] - error > zero && direction[1] - error > zero
        && residual <= bound;
    const bool accept_control = al_r20_newton_armijo(
        one, zero, static_cast<Binary128>(2.0), zero,
        one, one, zero);
    const bool reject_control = !al_r20_newton_armijo(
        one, zero, one, zero, one, one, zero);
    const ALR20ADMMFactor rejected = al_r20_admm_factor_matrix(
        {-one, zero, zero, one}, 2U, nullptr);
    return direction_control && accept_control && reject_control
        && !rejected.passed;
}

} // namespace

SplitBoundaryReport run_al_generalization_newton_direction_controls() {
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
        && al_r20_newton_direction_controls();
    std::vector<ALR20ManifestCase> cases = al_r20_manifest_cases();
    const bool parent_exact = controls_exact
        && al_r20_manifest_semantic_root(cases) == AL_R20_MANIFEST_SHA256
        && cases.size() == AL_R20_PROBLEM_SHA256.size();
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    std::vector<ALR20ProblemData> problems;
    std::vector<ALR20NewtonDirectionCase> diagnostics;
    bool arithmetic_exact = parent_exact;
    bool projected_selection = false;
    bool all_unit = true;
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        ALR20ProblemData problem = al_r20_materialize_problem(
            cases[index], &sparse_work, &static_work, &adjacency_work);
        const bool problem_exact = problem.exact
            && index < AL_R20_PROBLEM_SHA256.size()
            && problem.problem_root == AL_R20_PROBLEM_SHA256[index]
            && problem.source_positive == 0U;
        ALR20NewtonDirectionCase diagnostic = problem_exact
            ? al_r20_newton_direction_case(problem, index)
            : ALR20NewtonDirectionCase{};
        arithmetic_exact = arithmetic_exact
            && problem_exact && diagnostic.exact;
        if (!diagnostic.quiet) {
            projected_selection = projected_selection
                || !diagnostic.direction_feasible
                || !diagnostic.slope_positive || !diagnostic.accepted;
            all_unit = all_unit && diagnostic.accepted
                && diagnostic.selected.power == 0U
                && diagnostic.selected.mask_changes == 0U
                && !diagnostic.selected.ball_activity_changed
                && diagnostic.selected.metrics.certified;
        }
        diagnostics.push_back(std::move(diagnostic));
        problems.push_back(std::move(problem));
    }
    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    arithmetic_exact = arithmetic_exact && lifecycle_exact;
    const std::string route = !parent_exact
        ? "NEWTON_DIRECTION_PARENT_REJECTED"
        : (!arithmetic_exact ? "NEWTON_DIRECTION_REJECTED"
            : (projected_selection
                ? "PROJECTED_REPRESENTATIVE_SELECTION_REQUIRED"
                : (all_unit ? "UNIT_NEWTON_DEVELOPMENT_CANDIDATE"
                            : "GLOBALIZED_NEWTON_RELINEARIZATION_REQUIRED")));
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r5-newton-direction.v1|"
             << AL_R20_PROJECTOR_SHA256 << ':' << route << '|';
    for (const ALR20NewtonDirectionCase& value : diagnostics)
        semantic << value.root << ';';
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r5_newton_direction.v1\""
           << ",\"status\":\"" << (arithmetic_exact ? "PASS" : "FAIL") << "\""
           << ",\"route\":\"" << route << "\""
           << ",\"parent_projector_sha256\":\""
           << AL_R20_PROJECTOR_SHA256
           << "\",\"profile_exact\":" << (profile_exact ? "true" : "false")
           << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
           << ",\"lifecycle_exact\":" << (lifecycle_exact ? "true" : "false")
           << ",\"cases\":[";
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20NewtonDirectionCase& value = diagnostics[index];
        report << "{\"name\":\"" << cases[index].fixture.name
               << "\",\"role\":\"" << cases[index].role
               << "\",\"exact\":" << (value.exact ? "true" : "false")
               << ",\"quiet\":" << (value.quiet ? "true" : "false")
               << ",\"positive_rows\":" << value.positive_rows
               << ",\"solved\":" << (value.solved ? "true" : "false")
               << ",\"direction_feasible\":"
               << (value.direction_feasible ? "true" : "false")
               << ",\"negative_components\":" << value.negative_components
               << ",\"uncertain_components\":" << value.uncertain_components
               << ",\"minimum_direction\":\""
               << al_binary128_text(value.minimum_direction)
               << "\",\"direction_error_bound\":\""
               << al_binary128_text(value.direction_error_bound)
               << "\",\"solve_residual\":\""
               << al_binary128_text(value.solve_residual)
               << "\",\"solve_bound\":\""
               << al_binary128_text(value.solve_bound)
               << "\",\"slope\":\"" << al_binary128_text(value.slope)
               << "\",\"slope_bound\":\""
               << al_binary128_text(value.slope_bound)
               << "\",\"line_trials\":" << value.trials.size()
               << ",\"accepted\":" << (value.accepted ? "true" : "false");
        if (value.accepted)
            report << ",\"accepted_power\":" << value.selected.power
                   << ",\"accepted_alpha\":\""
                   << al_binary128_text(value.selected.alpha)
                   << "\",\"dual_increase_lower\":\""
                   << al_binary128_text(value.selected.dual_increase_lower)
                   << "\",\"mask_changes\":" << value.selected.mask_changes
                   << ",\"ball_activity_changed\":"
                   << (value.selected.ball_activity_changed ? "true" : "false")
                   << ",\"candidate_certified\":"
                   << (value.selected.metrics.certified ? "true" : "false")
                   << ",\"candidate_primal\":\""
                   << al_binary128_text(value.selected.metrics.primal)
                   << "\",\"candidate_dual_mapping\":\""
                   << al_binary128_text(value.selected.metrics.dual_mapping)
                   << "\",\"candidate_complementarity\":\""
                   << al_binary128_text(value.selected.metrics.complementarity)
                   << "\",\"candidate_stationarity\":\""
                   << al_binary128_text(value.selected.metrics.stationarity)
                   << "\",\"candidate_gap_scaled\":\""
                   << al_binary128_text(value.selected.metrics.gap_scaled) << "\"";
        report << ",\"factor_terms\":" << value.work.factor_terms
               << ",\"solve_terms\":" << value.work.solve_terms
               << ",\"active_rows_root\":\"" << value.active_rows_root
               << "\",\"face_matrix_root\":\"" << value.face_matrix_root
               << "\",\"root\":\"" << value.root << "\"}";
    }
    report << "]"
              ",\"newton_solves_per_excited_case\":1"
              ",\"accepted_candidates_per_excited_case_maximum\":1"
              ",\"newton_iterations\":0,\"active_set_pruning\":0"
              ",\"diagonal_regularization\":false"
              ",\"old_oracle_reruns\":0,\"timing_admitted\":false"
              ",\"generalization_authority\":false"
              ",\"runtime_authority\":false,\"production_authority\":false"
              ",\"result_sha256\":\""
           << result_sha << "\"}";
    return {arithmetic_exact, report.str()};
}
