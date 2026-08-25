namespace {

constexpr const char* AL_R20_NEWTON_DIRECTION_SHA256 =
    "5ae59256f4b4836d9ceccf5e2ad6b1a071e6fce7097463b4b8b4ff1e921745b7";

struct ALR20NNQPSolve {
    bool exact = false;
    bool terminated = false;
    bool kkt_certified = false;
    std::size_t transitions = 0U;
    std::size_t principal_solves = 0U;
    std::size_t entered = 0U;
    std::size_t removed = 0U;
    std::size_t support_size = 0U;
    Binary128 direction_error_bound = static_cast<Binary128>(0.0);
    Binary128 primal_violation = static_cast<Binary128>(0.0);
    Binary128 dual_violation = static_cast<Binary128>(0.0);
    Binary128 complementarity = static_cast<Binary128>(0.0);
    Binary128 slope = static_cast<Binary128>(0.0);
    Binary128 slope_bound = static_cast<Binary128>(0.0);
    Binary128 minimum_positive = static_cast<Binary128>(0.0);
    Binary128 failure_value = static_cast<Binary128>(0.0);
    Binary128 failure_bound = static_cast<Binary128>(0.0);
    std::size_t failure_source_row = 0U;
    std::vector<Binary128> direction;
    std::vector<Binary128> reduced_gradient;
    std::vector<bool> passive;
    ALR20ADMMWork work;
    std::string failure = "NONE";
    std::string support_root;
    std::string root;
};

struct ALR20NNQPRatio {
    bool exact = false;
    Binary128 alpha = static_cast<Binary128>(0.0);
    std::vector<std::size_t> hits;
};

ALR20NNQPRatio al_r20_nnqp_ratio(
    const std::vector<Binary128>& current,
    const std::vector<Binary128>& candidate,
    Binary128 current_error, Binary128 candidate_error,
    const std::vector<std::size_t>& negative,
    const std::vector<std::size_t>& source_rows) {
    ALR20NNQPRatio result;
    if (current.size() != candidate.size()
        || current.size() != source_rows.size() || negative.empty())
        return result;
    std::vector<Binary128> ratio(current.size());
    std::vector<Binary128> bound(current.size());
    std::size_t best = current.size();
    for (std::size_t index : negative) {
        const Binary128 denominator = current[index] - candidate[index];
        if (!(denominator > static_cast<Binary128>(0.0))) return result;
        ratio[index] = current[index] / denominator;
        bound[index] = al_r20_q_gamma(512U)
            * (static_cast<Binary128>(1.0) + fabsq(ratio[index]))
            + (current_error + candidate_error)
                * (static_cast<Binary128>(1.0) + fabsq(ratio[index]))
                / denominator;
        if (best == current.size() || ratio[index] < ratio[best]
            || (ratio[index] == ratio[best]
                && source_rows[index] < source_rows[best]))
            best = index;
    }
    if (best == current.size()) return result;
    for (std::size_t index : negative) {
        if (ratio[index] == ratio[best]) result.hits.push_back(index);
        else if (ratio[index] - bound[index]
                <= ratio[best] + bound[best]
            && ratio[index] + bound[index]
                >= ratio[best] - bound[best])
            return result;
    }
    std::sort(result.hits.begin(), result.hits.end(),
        [&](std::size_t lhs, std::size_t rhs) {
            return source_rows[lhs] < source_rows[rhs];
        });
    result.alpha = ratio[best];
    result.exact = result.alpha >= static_cast<Binary128>(0.0)
        && result.alpha <= static_cast<Binary128>(1.0);
    return result;
}

void al_r20_nnqp_reduced_gradient(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& rhs,
    const std::vector<Binary128>& rhs_bound,
    const std::vector<Binary128>& direction,
    Binary128 direction_error,
    std::vector<Binary128>* reduced,
    std::vector<Binary128>* bound) {
    const std::size_t dimension = rhs.size();
    reduced->assign(dimension, static_cast<Binary128>(0.0));
    bound->assign(dimension, static_cast<Binary128>(0.0));
    for (std::size_t row = 0U; row < dimension; ++row) {
        Binary128Accumulator image;
        Binary128 absolute = fabsq(rhs[row]);
        for (std::size_t column = 0U; column < dimension; ++column) {
            const Binary128 term = matrix[row * dimension + column]
                * direction[column];
            image.add(term);
            absolute += fabsq(term)
                + fabsq(matrix[row * dimension + column])
                    * direction_error;
        }
        (*reduced)[row] = rhs[row] - image.value();
        (*bound)[row] = rhs_bound[row]
            + al_r20_q_gamma(8U * dimension + 512U)
                * fmaxq(absolute,
                    ldexpq(static_cast<Binary128>(1.0), -16382));
    }
}

ALR20NNQPSolve al_r20_nnqp_solve(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& rhs,
    const std::vector<Binary128>& rhs_bound,
    const std::vector<std::size_t>& source_rows) {
    ALR20NNQPSolve result;
    const std::size_t dimension = rhs.size();
    if (dimension == 0U || matrix.size() != dimension * dimension
        || rhs_bound.size() != dimension
        || source_rows.size() != dimension
        || !al_r20_admm_finite(matrix)
        || !al_r20_admm_finite(rhs)
        || !al_r20_admm_finite(rhs_bound)) {
        result.failure = "INVALID_INPUT";
        return result;
    }
    result.direction.assign(dimension, static_cast<Binary128>(0.0));
    result.reduced_gradient = rhs;
    result.passive.assign(dimension, false);
    Binary128 direction_error = static_cast<Binary128>(0.0);
    const std::size_t transition_cap = 8U * dimension * dimension + 8U;
    bool exact = true;
    bool terminated = false;
    while (exact && result.transitions < transition_cap) {
        std::vector<Binary128> reduced_bound;
        al_r20_nnqp_reduced_gradient(matrix, rhs, rhs_bound,
            result.direction, direction_error,
            &result.reduced_gradient, &reduced_bound);
        std::size_t entering = dimension;
        for (std::size_t index = 0U; index < dimension; ++index) {
            if (result.passive[index]
                || !(result.reduced_gradient[index]
                    - reduced_bound[index]
                        > static_cast<Binary128>(0.0))) continue;
            if (entering == dimension
                || result.reduced_gradient[index]
                    > result.reduced_gradient[entering]
                || (result.reduced_gradient[index]
                        == result.reduced_gradient[entering]
                    && source_rows[index] < source_rows[entering]))
                entering = index;
        }
        if (entering == dimension) {
            for (std::size_t index = 0U; index < dimension; ++index)
                if (!result.passive[index]
                    && result.reduced_gradient[index]
                        + reduced_bound[index]
                            > static_cast<Binary128>(0.0))
                    exact = false;
            if (!exact) result.failure = "INACTIVE_SIGN_UNRESOLVED";
            terminated = exact;
            break;
        }
        result.passive[entering] = true;
        ++result.entered;
        ++result.transitions;

        bool passive_solution = false;
        while (exact && !passive_solution
            && result.transitions < transition_cap) {
            std::vector<std::size_t> indices;
            for (std::size_t index = 0U; index < dimension; ++index)
                if (result.passive[index]) indices.push_back(index);
            const std::size_t passive_dimension = indices.size();
            std::vector<Binary128> principal(
                passive_dimension * passive_dimension);
            std::vector<Binary128> principal_rhs(passive_dimension);
            for (std::size_t row = 0U; row < passive_dimension; ++row) {
                principal_rhs[row] = rhs[indices[row]];
                for (std::size_t column = 0U;
                     column < passive_dimension; ++column)
                    principal[row * passive_dimension + column] = matrix[
                        indices[row] * dimension + indices[column]];
            }
            ALR20ADMMFactor factor = al_r20_admm_factor_matrix(
                std::move(principal), passive_dimension, &result.work);
            std::vector<Binary128> passive_value;
            Binary128 solve_residual = static_cast<Binary128>(0.0);
            Binary128 solve_bound = static_cast<Binary128>(0.0);
            const bool solved = factor.passed && al_r20_admm_solve(
                factor, principal_rhs, &passive_value,
                &solve_residual, &solve_bound, &result.work);
            ++result.principal_solves;
            if (!solved) {
                exact = false;
                result.failure = "PRINCIPAL_SOLVE_REJECTED";
                break;
            }
            const Binary128 passive_error =
                al_r20_newton_direction_error_bound(factor,
                    solve_residual, solve_bound, passive_value);
            std::vector<Binary128> candidate(dimension,
                static_cast<Binary128>(0.0));
            std::vector<std::size_t> negative;
            for (std::size_t local = 0U;
                 local < passive_dimension; ++local) {
                candidate[indices[local]] = passive_value[local];
                if (passive_value[local] + passive_error
                        < static_cast<Binary128>(0.0))
                    negative.push_back(indices[local]);
                else if (passive_value[local] - passive_error
                        <= static_cast<Binary128>(0.0)) {
                    exact = false;
                    result.failure = "PASSIVE_SIGN_UNRESOLVED";
                    result.failure_value = passive_value[local];
                    result.failure_bound = passive_error;
                    result.failure_source_row = source_rows[indices[local]];
                }
            }
            if (!exact) break;
            if (negative.empty()) {
                result.direction = std::move(candidate);
                direction_error = passive_error;
                passive_solution = true;
                continue;
            }

            const ALR20NNQPRatio ratio = al_r20_nnqp_ratio(
                result.direction, candidate, direction_error,
                passive_error, negative, source_rows);
            if (!ratio.exact) {
                exact = false;
                result.failure = "BOUNDARY_RATIO_UNRESOLVED";
                break;
            }
            Binary128 step_scale = static_cast<Binary128>(1.0);
            for (std::size_t index = 0U; index < dimension; ++index)
                step_scale = fmaxq(step_scale,
                    fabsq(candidate[index] - result.direction[index]));
            for (std::size_t index = 0U; index < dimension; ++index)
                result.direction[index] += ratio.alpha
                    * (candidate[index] - result.direction[index]);
            direction_error = direction_error
                + ratio.alpha * (passive_error + direction_error)
                + al_r20_q_gamma(32U * dimension + 512U) * step_scale;
            for (std::size_t hitting : ratio.hits) {
                result.direction[hitting] = static_cast<Binary128>(0.0);
                result.passive[hitting] = false;
                ++result.removed;
                ++result.transitions;
            }
            for (std::size_t index = 0U; index < dimension; ++index)
                if (result.passive[index]
                    && result.direction[index] - direction_error
                        <= static_cast<Binary128>(0.0)) {
                    exact = false;
                    result.failure = "BOUNDARY_UPDATE_UNRESOLVED";
                    break;
                }
        }
    }
    if (exact && !terminated
        && result.transitions >= transition_cap)
        result.failure = "TRANSITION_CAP";
    result.terminated = terminated;
    result.direction_error_bound = direction_error;
    std::vector<Binary128> reduced_bound;
    al_r20_nnqp_reduced_gradient(matrix, rhs, rhs_bound,
        result.direction, direction_error,
        &result.reduced_gradient, &reduced_bound);
    result.minimum_positive = HUGE_VALQ;
    Binary128 energy_scale = static_cast<Binary128>(1.0);
    for (std::size_t index = 0U; index < dimension; ++index) {
        if (result.passive[index]) {
            ++result.support_size;
            result.minimum_positive = fminq(
                result.minimum_positive, result.direction[index]);
            result.primal_violation = fmaxq(result.primal_violation,
                fmaxq(static_cast<Binary128>(0.0),
                    direction_error - result.direction[index]));
            result.dual_violation = fmaxq(result.dual_violation,
                fmaxq(static_cast<Binary128>(0.0),
                    fabsq(result.reduced_gradient[index])
                        - reduced_bound[index]));
        } else {
            result.primal_violation = fmaxq(result.primal_violation,
                fabsq(result.direction[index]));
            result.dual_violation = fmaxq(result.dual_violation,
                fmaxq(static_cast<Binary128>(0.0),
                    result.reduced_gradient[index] + reduced_bound[index]));
        }
        energy_scale = fmaxq(energy_scale,
            fabsq(rhs[index] * result.direction[index]));
        result.complementarity = fmaxq(result.complementarity,
            fabsq(result.direction[index]
                * result.reduced_gradient[index]));
    }
    if (result.support_size == 0U)
        result.minimum_positive = static_cast<Binary128>(0.0);
    result.complementarity /= energy_scale;
    Binary128Accumulator slope;
    Binary128 slope_absolute = static_cast<Binary128>(0.0);
    for (std::size_t index = 0U; index < dimension; ++index) {
        const Binary128 term = rhs[index] * result.direction[index];
        slope.add(term);
        slope_absolute += fabsq(term);
        result.slope_bound += rhs_bound[index]
            * (fabsq(result.direction[index]) + direction_error)
            + fabsq(rhs[index]) * direction_error;
    }
    result.slope = slope.value();
    result.slope_bound += al_r20_q_gamma(16U * dimension + 512U)
        * slope_absolute;
    result.kkt_certified = exact && terminated
        && result.primal_violation == static_cast<Binary128>(0.0)
        && result.dual_violation == static_cast<Binary128>(0.0)
        && result.complementarity
            <= al_r20_q_gamma(64U * dimension + 4096U)
        && result.slope - result.slope_bound
            > static_cast<Binary128>(0.0);
    result.exact = exact && terminated;
    std::ostringstream support_material;
    for (std::size_t index = 0U; index < dimension; ++index)
        if (result.passive[index]) support_material << source_rows[index] << ',';
    result.support_root = sha256_hex(support_material.str());
    std::ostringstream material;
    material << result.exact << ':' << result.terminated << ':'
             << result.kkt_certified << ':' << result.transitions << ':'
             << result.principal_solves << ':' << result.entered << ':'
             << result.removed << ':' << result.support_size << ':'
             << al_binary128_hex(result.direction_error_bound) << ':'
             << al_binary128_hex(result.primal_violation) << ':'
             << al_binary128_hex(result.dual_violation) << ':'
             << al_binary128_hex(result.complementarity) << ':'
             << al_binary128_hex(result.slope) << ':'
             << al_binary128_hex(result.slope_bound) << ':'
             << al_binary128_hex(result.minimum_positive) << ':'
             << al_r20_q_scalar_root(result.direction) << ':'
             << al_r20_q_scalar_root(result.reduced_gradient) << ':'
             << result.support_root;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20NNQPCase {
    bool exact = false;
    bool quiet = false;
    bool accepted = false;
    std::size_t positive_rows = 0U;
    ALR20NewtonDualEvaluation base;
    ALR20OracleMetrics base_metrics;
    ALR20NNQPSolve solve;
    std::vector<ALR20NewtonTrial> trials;
    ALR20NewtonTrial selected;
    std::string active_rows_root;
    std::string face_matrix_root;
    std::string root;
};

ALR20NNQPCase al_r20_nnqp_case(
    const ALR20ProblemData& problem, std::size_t case_index) {
    ALR20NNQPCase result;
    const std::size_t all_rows = problem.row_scale.size();
    const std::vector<Binary128> zero_lambda(all_rows,
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
    result.quiet = active_rows.empty();
    std::ostringstream active_material;
    for (std::size_t row : active_rows) active_material << row << ',';
    result.active_rows_root = sha256_hex(active_material.str());
    if (result.quiet) {
        result.face_matrix_root =
            al_r20_q_scalar_root(std::vector<Binary128>{});
        result.exact = result.base.exact && result.base_metrics.certified
            && case_index < AL_R20_FACE_MATRIX_SHA256.size()
            && result.face_matrix_root
                == AL_R20_FACE_MATRIX_SHA256[case_index];
        result.root = sha256_hex(result.base.root + ':'
            + result.base_metrics.root + ':' + result.active_rows_root
            + ':' + result.face_matrix_root);
        return result;
    }

    const std::size_t dimension = active_rows.size();
    std::vector<std::vector<Binary128Vec3>> row_vectors;
    std::vector<std::vector<Binary128Vec3>> images;
    for (std::size_t row : active_rows) {
        row_vectors.push_back(al_r20_projector_row_vector(
            problem.op, row, problem.target.size()));
        images.push_back(al_r20_projector_jvp(
            result.base.projector, row_vectors.back()));
    }
    std::vector<Binary128> raw_matrix(dimension * dimension);
    std::vector<Binary128> matrix(dimension * dimension);
    std::vector<Binary128> rhs(dimension);
    std::vector<Binary128> rhs_bound(dimension);
    for (std::size_t row = 0U; row < dimension; ++row) {
        rhs[row] = result.base.residual[active_rows[row]];
        rhs_bound[row] = result.base.residual_bound[active_rows[row]];
        for (std::size_t column = 0U; column < dimension; ++column)
            raw_matrix[row * dimension + column] =
                al_r20_projector_vector_dot(
                    row_vectors[row], images[column]);
    }
    for (std::size_t row = 0U; row < dimension; ++row)
        for (std::size_t column = 0U; column < dimension; ++column)
            matrix[row * dimension + column] = static_cast<Binary128>(0.5)
                * (raw_matrix[row * dimension + column]
                    + raw_matrix[column * dimension + row]);
    result.face_matrix_root = al_r20_q_scalar_root(raw_matrix);
    const bool face_exact = case_index < AL_R20_FACE_MATRIX_SHA256.size()
        && result.face_matrix_root == AL_R20_FACE_MATRIX_SHA256[case_index];
    result.solve = al_r20_nnqp_solve(
        matrix, rhs, rhs_bound, active_rows);

    bool line_exact = true;
    if (result.solve.kkt_certified) {
        for (std::size_t power = 0U;
             power < AL_R20_NEWTON_LINE_TRIALS; ++power) {
            ALR20NewtonTrial trial;
            trial.power = power;
            trial.alpha = ldexpq(static_cast<Binary128>(1.0),
                -static_cast<int>(power));
            std::vector<Binary128> lambda(all_rows,
                static_cast<Binary128>(0.0));
            for (std::size_t index = 0U; index < dimension; ++index)
                lambda[active_rows[index]] =
                    trial.alpha * result.solve.direction[index];
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
                trial.alpha, result.solve.slope,
                result.solve.slope_bound);
            trial.metrics = al_r20_q_metrics(problem,
                trial.dual.projector.value, lambda, diagonal, 1U);
            trial.exact = trial.exact && trial.metrics.finite;
            std::ostringstream material;
            material << trial.exact << ':' << trial.accepted << ':'
                << power << ':' << al_binary128_hex(trial.alpha) << ':'
                << al_binary128_hex(trial.dual_increase_lower) << ':'
                << trial.mask_changes << ':' << trial.ball_activity_changed
                << ':' << trial.dual.root << ':' << trial.metrics.root;
            trial.root = sha256_hex(material.str());
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
        && face_exact && result.solve.exact && line_exact;
    std::ostringstream material;
    material << result.exact << ':' << result.accepted << ':'
             << result.positive_rows << ':' << result.base.root << ':'
             << result.base_metrics.root << ':' << result.solve.root << ':'
             << result.active_rows_root << ':' << result.face_matrix_root << '|';
    for (const ALR20NewtonTrial& trial : result.trials)
        material << trial.root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

bool al_r20_nnqp_controls() {
    const Binary128 zero = static_cast<Binary128>(0.0);
    const Binary128 one = static_cast<Binary128>(1.0);
    const Binary128 tenth = one / static_cast<Binary128>(10.0);
    const std::vector<Binary128> no_bound(2U, zero);
    const ALR20NNQPSolve interior = al_r20_nnqp_solve(
        {static_cast<Binary128>(2.0), one,
         one, static_cast<Binary128>(2.0)},
        {one, one}, no_bound, {0U, 1U});
    const ALR20NNQPSolve boundary = al_r20_nnqp_solve(
        {one, static_cast<Binary128>(0.9),
         static_cast<Binary128>(0.9), one},
        {one, tenth}, no_bound, {0U, 1U});
    const Binary128 tolerance = ldexpq(one, -90);
    const ALR20NNQPRatio ratio = al_r20_nnqp_ratio(
        {one, one}, {-one, -one}, zero, zero,
        {0U, 1U}, {1U, 0U});
    return interior.exact && interior.kkt_certified
        && interior.support_size == 2U
        && fabsq(interior.direction[0] - one / static_cast<Binary128>(3.0))
            <= tolerance
        && fabsq(interior.direction[1] - one / static_cast<Binary128>(3.0))
            <= tolerance
        && boundary.exact && boundary.kkt_certified
        && boundary.support_size == 1U
        && fabsq(boundary.direction[0] - one) <= tolerance
        && boundary.direction[1] == zero
        && ratio.exact && ratio.alpha == static_cast<Binary128>(0.5)
        && ratio.hits == std::vector<std::size_t>({1U, 0U})
        && al_r20_newton_direction_controls();
}

} // namespace

SplitBoundaryReport run_al_generalization_nnqp_representative_controls() {
#if defined(__linux__) && defined(__x86_64__) && defined(__GNUC__) \
    && defined(__SIZEOF_FLOAT128__)
    constexpr bool frozen_platform = true;
#else
    constexpr bool frozen_platform = false;
#endif
    const bool profile_exact = frozen_platform
        && sizeof(Binary128) == 16U && FLT_RADIX == 2
        && FLT128_MANT_DIG == 113;
    const bool controls_exact = profile_exact && al_r20_nnqp_controls();
    std::vector<ALR20ManifestCase> cases = al_r20_manifest_cases();
    const bool parent_exact = controls_exact
        && al_r20_manifest_semantic_root(cases) == AL_R20_MANIFEST_SHA256
        && cases.size() == AL_R20_PROBLEM_SHA256.size();
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    std::vector<ALR20ProblemData> problems;
    std::vector<ALR20NNQPCase> diagnostics;
    bool arithmetic_exact = parent_exact;
    bool directions_exact = true;
    bool globalization_exact = true;
    bool all_unit = true;
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        ALR20ProblemData problem = al_r20_materialize_problem(
            cases[index], &sparse_work, &static_work, &adjacency_work);
        const bool problem_exact = problem.exact
            && index < AL_R20_PROBLEM_SHA256.size()
            && problem.problem_root == AL_R20_PROBLEM_SHA256[index]
            && problem.source_positive == 0U;
        ALR20NNQPCase diagnostic = problem_exact
            ? al_r20_nnqp_case(problem, index) : ALR20NNQPCase{};
        arithmetic_exact = arithmetic_exact
            && problem_exact && diagnostic.exact;
        if (!diagnostic.quiet) {
            directions_exact = directions_exact
                && diagnostic.solve.kkt_certified;
            globalization_exact = globalization_exact && diagnostic.accepted;
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
        ? "NNQP_PARENT_REJECTED"
        : (!arithmetic_exact ? "NNQP_ACTIVE_SET_REJECTED"
            : (!directions_exact ? "NNQP_DIRECTION_REJECTED"
                : (!globalization_exact ? "NNQP_GLOBALIZATION_REJECTED"
                    : (all_unit ? "NNQP_UNIT_DEVELOPMENT_CANDIDATE"
                                : "NNQP_RELINEARIZATION_REQUIRED"))));
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r6-nnqp-representative.v1|"
             << AL_R20_NEWTON_DIRECTION_SHA256 << ':' << route << '|';
    for (const ALR20NNQPCase& value : diagnostics)
        semantic << value.root << ';';
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r6_nnqp_representative.v1\""
           << ",\"status\":\"" << (arithmetic_exact ? "PASS" : "FAIL") << "\""
           << ",\"route\":\"" << route << "\""
           << ",\"parent_newton_sha256\":\""
           << AL_R20_NEWTON_DIRECTION_SHA256
           << "\",\"profile_exact\":" << (profile_exact ? "true" : "false")
           << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
           << ",\"lifecycle_exact\":" << (lifecycle_exact ? "true" : "false")
           << ",\"cases\":[";
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20NNQPCase& value = diagnostics[index];
        report << "{\"name\":\"" << cases[index].fixture.name
               << "\",\"role\":\"" << cases[index].role
               << "\",\"exact\":" << (value.exact ? "true" : "false")
               << ",\"quiet\":" << (value.quiet ? "true" : "false")
               << ",\"positive_rows\":" << value.positive_rows
               << ",\"terminated\":"
               << (value.solve.terminated ? "true" : "false")
               << ",\"direction_certified\":"
               << (value.solve.kkt_certified ? "true" : "false")
               << ",\"support_size\":" << value.solve.support_size
               << ",\"transitions\":" << value.solve.transitions
               << ",\"principal_solves\":" << value.solve.principal_solves
               << ",\"entered\":" << value.solve.entered
               << ",\"removed\":" << value.solve.removed
               << ",\"direction_error_bound\":\""
               << al_binary128_text(value.solve.direction_error_bound)
               << "\",\"qp_primal\":\""
               << al_binary128_text(value.solve.primal_violation)
               << "\",\"qp_dual\":\""
               << al_binary128_text(value.solve.dual_violation)
               << "\",\"qp_complementarity\":\""
               << al_binary128_text(value.solve.complementarity)
               << "\",\"slope\":\"" << al_binary128_text(value.solve.slope)
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
        report << ",\"factor_terms\":" << value.solve.work.factor_terms
               << ",\"solve_terms\":" << value.solve.work.solve_terms
               << ",\"support_root\":\"" << value.solve.support_root
               << "\",\"root\":\"" << value.root << "\"}";
    }
    report << "]"
              ",\"nonlinear_candidates_per_excited_case_maximum\":1"
              ",\"nonlinear_iterations\":0,\"block_pivots\":0"
              ",\"diagonal_regularization\":false"
              ",\"old_oracle_reruns\":0,\"timing_admitted\":false"
              ",\"generalization_authority\":false"
              ",\"runtime_authority\":false,\"production_authority\":false"
              ",\"result_sha256\":\""
           << result_sha << "\"}";
    return {arithmetic_exact, report.str()};
}
