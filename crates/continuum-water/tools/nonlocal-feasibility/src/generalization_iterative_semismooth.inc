namespace {

constexpr const char* AL_R20_NNQP_SHA256 =
    "bfa10141c0f50658548205e44d397a8999f36a0d0c0e00da0cb9d3d7ebc01500";
constexpr std::size_t AL_R20_SEMISMOOTH_ITERATION_CAP = 32U;

struct ALR20SemismoothStep {
    bool harness_exact = false;
    bool natural_face_resolved = false;
    bool direction_certified = false;
    bool accepted = false;
    std::size_t iteration = 0U;
    std::size_t natural_face_size = 0U;
    std::size_t ambiguous_rows = 0U;
    Binary128 slope = static_cast<Binary128>(0.0);
    Binary128 slope_bound = static_cast<Binary128>(0.0);
    Binary128 direction_error_bound = static_cast<Binary128>(0.0);
    ALR20NNQPSolve solve;
    std::vector<ALR20NewtonTrial> trials;
    ALR20NewtonTrial selected;
    std::vector<Binary128> next_lambda;
    std::string natural_face_root;
    std::string hessian_root;
    std::string root;
};

ALR20SemismoothStep al_r20_semismooth_step(
    const ALR20ProblemData& problem,
    const ALR20NewtonDualEvaluation& current,
    const std::vector<Binary128>& lambda,
    const std::vector<Binary128>& diagonal,
    std::size_t iteration) {
    ALR20SemismoothStep result;
    result.iteration = iteration;
    const std::size_t all_rows = lambda.size();
    std::vector<std::size_t> natural_face;
    for (std::size_t row = 0U; row < all_rows; ++row) {
        const Binary128 quotient = current.residual[row] / diagonal[row];
        const Binary128 argument = lambda[row] + quotient;
        const Binary128 bound = current.residual_bound[row] / diagonal[row]
            + al_r20_q_gamma(256U)
                * (fabsq(lambda[row]) + fabsq(quotient));
        if (argument - bound > static_cast<Binary128>(0.0))
            natural_face.push_back(row);
        else if (!(argument + bound < static_cast<Binary128>(0.0)))
            ++result.ambiguous_rows;
    }
    result.natural_face_size = natural_face.size();
    result.natural_face_resolved = result.ambiguous_rows == 0U
        && !natural_face.empty();
    std::ostringstream face_material;
    for (std::size_t row : natural_face) face_material << row << ',';
    result.natural_face_root = sha256_hex(face_material.str());
    result.harness_exact = current.exact
        && lambda.size() == problem.row_scale.size()
        && diagonal.size() == all_rows;
    if (!result.natural_face_resolved) {
        std::ostringstream material;
        material << result.harness_exact << ':'
                 << result.natural_face_resolved << ':' << iteration << ':'
                 << result.natural_face_size << ':' << result.ambiguous_rows
                 << ':' << current.root << ':' << result.natural_face_root;
        result.root = sha256_hex(material.str());
        return result;
    }

    std::vector<std::vector<Binary128Vec3>> rows;
    std::vector<std::vector<Binary128Vec3>> images;
    rows.reserve(all_rows);
    images.reserve(all_rows);
    for (std::size_t row = 0U; row < all_rows; ++row) {
        rows.push_back(al_r20_projector_row_vector(
            problem.op, row, problem.target.size()));
        images.push_back(al_r20_projector_jvp(
            current.projector, rows.back()));
    }
    const std::size_t dimension = natural_face.size();
    std::vector<Binary128> principal(dimension * dimension);
    std::vector<Binary128> rhs(dimension);
    std::vector<Binary128> rhs_bound(dimension);
    std::vector<Binary128> h_lambda(dimension,
        static_cast<Binary128>(0.0));
    std::vector<Binary128> h_lambda_bound(dimension,
        static_cast<Binary128>(0.0));
    std::vector<Binary128> raw_principal(dimension * dimension);
    for (std::size_t local_row = 0U;
         local_row < dimension; ++local_row) {
        const std::size_t row = natural_face[local_row];
        Binary128Accumulator image;
        Binary128 absolute = static_cast<Binary128>(0.0);
        for (std::size_t column = 0U; column < all_rows; ++column) {
            const Binary128 value = al_r20_projector_vector_dot(
                rows[row], images[column]);
            const Binary128 term = value * lambda[column];
            image.add(term);
            absolute += fabsq(term);
        }
        h_lambda[local_row] = image.value();
        h_lambda_bound[local_row] = al_r20_q_gamma(
            16U * all_rows * problem.target.size() + 8192U)
            * fmaxq(absolute,
                ldexpq(static_cast<Binary128>(1.0), -16382));
        rhs[local_row] = current.residual[row] + h_lambda[local_row];
        rhs_bound[local_row] = current.residual_bound[row]
            + h_lambda_bound[local_row];
        for (std::size_t local_column = 0U;
             local_column < dimension; ++local_column) {
            const std::size_t column = natural_face[local_column];
            raw_principal[local_row * dimension + local_column] =
                al_r20_projector_vector_dot(rows[row], images[column]);
        }
    }
    for (std::size_t row = 0U; row < dimension; ++row)
        for (std::size_t column = 0U; column < dimension; ++column)
            principal[row * dimension + column]
                = static_cast<Binary128>(0.5)
                * (raw_principal[row * dimension + column]
                    + raw_principal[column * dimension + row]);
    result.hessian_root = al_r20_q_scalar_root(raw_principal);
    result.solve = al_r20_nnqp_solve(
        principal, rhs, rhs_bound, natural_face);
    result.direction_error_bound = result.solve.direction_error_bound;
    std::vector<Binary128> representative(all_rows,
        static_cast<Binary128>(0.0));
    for (std::size_t local = 0U; local < dimension; ++local)
        representative[natural_face[local]] = result.solve.direction[local];
    std::vector<Binary128> direction(all_rows,
        static_cast<Binary128>(0.0));
    Binary128Accumulator slope;
    Binary128 slope_absolute = static_cast<Binary128>(0.0);
    for (std::size_t row = 0U; row < all_rows; ++row) {
        direction[row] = representative[row] - lambda[row];
        const Binary128 term = current.residual[row] * direction[row];
        slope.add(term);
        slope_absolute += fabsq(term);
        const Binary128 direction_bound = std::binary_search(
            natural_face.begin(), natural_face.end(), row)
            ? result.direction_error_bound
            : static_cast<Binary128>(0.0);
        result.slope_bound += current.residual_bound[row]
                * (fabsq(direction[row]) + direction_bound)
            + fabsq(current.residual[row]) * direction_bound;
    }
    result.slope = slope.value();
    result.slope_bound += al_r20_q_gamma(16U * all_rows + 512U)
        * slope_absolute;
    result.direction_certified = result.solve.kkt_certified
        && result.slope - result.slope_bound
            > static_cast<Binary128>(0.0);
    result.harness_exact = result.harness_exact && result.solve.exact
        && al_r20_admm_finite(direction)
        && finiteq(result.slope) != 0 && finiteq(result.slope_bound) != 0;
    bool line_exact = true;
    if (result.direction_certified) {
        for (std::size_t power = 0U;
             power < AL_R20_NEWTON_LINE_TRIALS; ++power) {
            ALR20NewtonTrial trial;
            trial.power = power;
            trial.alpha = ldexpq(static_cast<Binary128>(1.0),
                -static_cast<int>(power));
            std::vector<Binary128> trial_lambda(all_rows);
            bool cone_exact = true;
            for (std::size_t row = 0U; row < all_rows; ++row) {
                trial_lambda[row] =
                    (static_cast<Binary128>(1.0) - trial.alpha) * lambda[row]
                    + trial.alpha * representative[row];
                cone_exact = cone_exact
                    && trial_lambda[row] >= static_cast<Binary128>(0.0);
            }
            trial.dual = al_r20_newton_dual_evaluate(
                problem, trial_lambda);
            trial.exact = cone_exact && trial.dual.exact;
            trial.dual_increase_lower = trial.dual.value - trial.dual.bound
                - current.value - current.bound;
            trial.ball_activity_changed =
                trial.dual.projector.ball_active
                != current.projector.ball_active;
            for (std::size_t scalar = 0U;
                 scalar < current.projector.mask.size(); ++scalar)
                if (trial.dual.projector.mask[scalar]
                        != current.projector.mask[scalar])
                    ++trial.mask_changes;
            trial.accepted = trial.exact && al_r20_newton_armijo(
                current.value, current.bound,
                trial.dual.value, trial.dual.bound,
                trial.alpha, result.slope, result.slope_bound);
            trial.metrics = al_r20_q_metrics(problem,
                trial.dual.projector.value, trial_lambda,
                diagonal, iteration);
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
                result.next_lambda = std::move(trial_lambda);
                break;
            }
        }
    }
    result.harness_exact = result.harness_exact && line_exact;
    std::ostringstream material;
    material << result.harness_exact << ':'
             << result.natural_face_resolved << ':'
             << result.direction_certified << ':' << result.accepted << ':'
             << iteration << ':' << result.natural_face_size << ':'
             << result.ambiguous_rows << ':'
             << al_binary128_hex(result.slope) << ':'
             << al_binary128_hex(result.slope_bound) << ':'
             << al_binary128_hex(result.direction_error_bound) << ':'
             << current.root << ':' << al_r20_q_scalar_root(lambda) << ':'
             << result.natural_face_root << ':' << result.hessian_root << ':'
             << result.solve.root << ':' << al_r20_q_scalar_root(direction)
             << '|';
    for (const ALR20NewtonTrial& trial : result.trials)
        material << trial.root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20SemismoothCase {
    bool harness_exact = false;
    bool certified = false;
    bool active_set_rejected = false;
    bool direction_rejected = false;
    bool globalization_rejected = false;
    std::size_t accepted_iterations = 0U;
    std::size_t total_mask_changes = 0U;
    std::size_t total_principal_solves = 0U;
    std::size_t total_transitions = 0U;
    ALR20OracleMetrics initial_metrics;
    ALR20OracleMetrics final_metrics;
    std::vector<ALR20SemismoothStep> steps;
    std::vector<Binary128> final_lambda;
    std::string root;
};

ALR20SemismoothCase al_r20_semismooth_case(
    const ALR20ProblemData& problem) {
    ALR20SemismoothCase result;
    const std::size_t rows = problem.row_scale.size();
    const std::vector<Binary128> diagonal = al_r20_q_diagonal(problem.op);
    std::vector<Binary128> lambda(rows, static_cast<Binary128>(0.0));
    ALR20NewtonDualEvaluation current =
        al_r20_newton_dual_evaluate(problem, lambda);
    result.initial_metrics = al_r20_q_metrics(problem,
        current.projector.value, lambda, diagonal, 0U);
    result.final_metrics = result.initial_metrics;
    result.harness_exact = current.exact && result.initial_metrics.finite;
    if (result.initial_metrics.certified) {
        result.certified = true;
        result.final_lambda = lambda;
        result.root = sha256_hex(current.root + ':'
            + result.initial_metrics.root + ":quiet");
        return result;
    }
    for (std::size_t iteration = 1U;
         iteration <= AL_R20_SEMISMOOTH_ITERATION_CAP; ++iteration) {
        ALR20SemismoothStep step = al_r20_semismooth_step(
            problem, current, lambda, diagonal, iteration);
        result.harness_exact = result.harness_exact && step.harness_exact;
        result.total_principal_solves += step.solve.principal_solves;
        result.total_transitions += step.solve.transitions;
        if (!step.natural_face_resolved || !step.solve.exact) {
            result.active_set_rejected = true;
            result.steps.push_back(std::move(step));
            break;
        }
        if (!step.direction_certified) {
            result.direction_rejected = true;
            result.steps.push_back(std::move(step));
            break;
        }
        if (!step.accepted) {
            result.globalization_rejected = true;
            result.steps.push_back(std::move(step));
            break;
        }
        result.total_mask_changes += step.selected.mask_changes;
        lambda = step.next_lambda;
        current = step.selected.dual;
        result.final_metrics = step.selected.metrics;
        ++result.accepted_iterations;
        result.steps.push_back(std::move(step));
        if (result.final_metrics.certified) {
            result.certified = true;
            break;
        }
    }
    result.final_lambda = lambda;
    std::ostringstream material;
    material << result.harness_exact << ':' << result.certified << ':'
             << result.active_set_rejected << ':'
             << result.direction_rejected << ':'
             << result.globalization_rejected << ':'
             << result.accepted_iterations << ':'
             << result.total_mask_changes << ':'
             << result.total_principal_solves << ':'
             << result.total_transitions << ':'
             << result.initial_metrics.root << ':'
             << result.final_metrics.root << ':'
             << al_r20_q_scalar_root(result.final_lambda) << '|';
    for (const ALR20SemismoothStep& step : result.steps)
        material << step.root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

bool al_r20_semismooth_controls() {
    const Binary128 zero = static_cast<Binary128>(0.0);
    const Binary128 one = static_cast<Binary128>(1.0);
    const Binary128 half = static_cast<Binary128>(0.5);
    const Binary128 lambda = static_cast<Binary128>(0.25);
    const Binary128 representative = static_cast<Binary128>(0.75);
    const Binary128 update = lambda + half * (representative - lambda);
    const bool cone_control = update == static_cast<Binary128>(0.5)
        && update >= zero;
    const bool natural_active = lambda + one > zero;
    const bool natural_inactive = zero - one < zero;
    const bool slope_rejected = !al_r20_newton_armijo(
        zero, zero, one, zero, one, -one, zero);
    return cone_control && natural_active && natural_inactive
        && slope_rejected && al_r20_nnqp_controls();
}

} // namespace

SplitBoundaryReport run_al_generalization_iterative_semismooth_controls() {
#if defined(__linux__) && defined(__x86_64__) && defined(__GNUC__) \
    && defined(__SIZEOF_FLOAT128__)
    constexpr bool frozen_platform = true;
#else
    constexpr bool frozen_platform = false;
#endif
    const bool profile_exact = frozen_platform
        && sizeof(Binary128) == 16U && FLT_RADIX == 2
        && FLT128_MANT_DIG == 113;
    const bool controls_exact = profile_exact && al_r20_semismooth_controls();
    std::vector<ALR20ManifestCase> cases = al_r20_manifest_cases();
    const bool parent_exact = controls_exact
        && al_r20_manifest_semantic_root(cases) == AL_R20_MANIFEST_SHA256
        && cases.size() == AL_R20_PROBLEM_SHA256.size();
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    std::vector<ALR20ProblemData> problems;
    std::vector<ALR20SemismoothCase> diagnostics;
    bool harness_exact = parent_exact;
    bool all_certified = true;
    bool active_set_rejected = false;
    bool direction_rejected = false;
    bool globalization_rejected = false;
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        ALR20ProblemData problem = al_r20_materialize_problem(
            cases[index], &sparse_work, &static_work, &adjacency_work);
        const bool problem_exact = problem.exact
            && index < AL_R20_PROBLEM_SHA256.size()
            && problem.problem_root == AL_R20_PROBLEM_SHA256[index]
            && problem.source_positive == 0U;
        ALR20SemismoothCase diagnostic = problem_exact
            ? al_r20_semismooth_case(problem) : ALR20SemismoothCase{};
        harness_exact = harness_exact
            && problem_exact && diagnostic.harness_exact;
        all_certified = all_certified && diagnostic.certified;
        active_set_rejected = active_set_rejected
            || diagnostic.active_set_rejected;
        direction_rejected = direction_rejected
            || diagnostic.direction_rejected;
        globalization_rejected = globalization_rejected
            || diagnostic.globalization_rejected;
        diagnostics.push_back(std::move(diagnostic));
        problems.push_back(std::move(problem));
    }
    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    harness_exact = harness_exact && lifecycle_exact;
    const std::string route = !parent_exact
        ? "SEMISMOOTH_PARENT_REJECTED"
        : (active_set_rejected ? "SEMISMOOTH_ACTIVE_SET_REJECTED"
            : (direction_rejected ? "SEMISMOOTH_DIRECTION_REJECTED"
                : (globalization_rejected
                    ? "SEMISMOOTH_GLOBALIZATION_REJECTED"
                    : (all_certified ? "SEMISMOOTH_DEVELOPMENT_CERTIFIED"
                        : "SEMISMOOTH_DEVELOPMENT_UNRESOLVED"))));
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r7-iterative-semismooth.v1|"
             << AL_R20_NNQP_SHA256 << ':' << route << '|';
    for (const ALR20SemismoothCase& value : diagnostics)
        semantic << value.root << ';';
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r7_iterative_semismooth.v1\""
           << ",\"status\":\"" << (harness_exact ? "PASS" : "FAIL") << "\""
           << ",\"route\":\"" << route << "\""
           << ",\"parent_nnqp_sha256\":\"" << AL_R20_NNQP_SHA256
           << "\",\"profile_exact\":" << (profile_exact ? "true" : "false")
           << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
           << ",\"lifecycle_exact\":" << (lifecycle_exact ? "true" : "false")
           << ",\"all_certified\":" << (all_certified ? "true" : "false")
           << ",\"cases\":[";
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20SemismoothCase& value = diagnostics[index];
        report << "{\"name\":\"" << cases[index].fixture.name
               << "\",\"role\":\"" << cases[index].role
               << "\",\"certified\":"
               << (value.certified ? "true" : "false")
               << ",\"accepted_iterations\":" << value.accepted_iterations
               << ",\"step_count\":" << value.steps.size()
               << ",\"total_mask_changes\":" << value.total_mask_changes
               << ",\"total_principal_solves\":"
               << value.total_principal_solves
               << ",\"total_transitions\":" << value.total_transitions
               << ",\"final_primal\":\""
               << al_binary128_text(value.final_metrics.primal)
               << "\",\"final_dual_mapping\":\""
               << al_binary128_text(value.final_metrics.dual_mapping)
               << "\",\"final_complementarity\":\""
               << al_binary128_text(value.final_metrics.complementarity)
               << "\",\"final_stationarity\":\""
               << al_binary128_text(value.final_metrics.stationarity)
               << "\",\"final_gap_scaled\":\""
               << al_binary128_text(value.final_metrics.gap_scaled)
               << "\",\"steps\":[";
        for (std::size_t step_index = 0U;
             step_index < value.steps.size(); ++step_index) {
            if (step_index != 0U) report << ',';
            const ALR20SemismoothStep& step = value.steps[step_index];
            report << "{\"iteration\":" << step.iteration
                   << ",\"harness_exact\":"
                   << (step.harness_exact ? "true" : "false")
                   << ",\"natural_face_resolved\":"
                   << (step.natural_face_resolved ? "true" : "false")
                   << ",\"face\":" << step.natural_face_size
                   << ",\"ambiguous\":" << step.ambiguous_rows
                   << ",\"nnqp_exact\":"
                   << (step.solve.exact ? "true" : "false")
                   << ",\"nnqp_terminated\":"
                   << (step.solve.terminated ? "true" : "false")
                   << ",\"nnqp_kkt\":"
                   << (step.solve.kkt_certified ? "true" : "false")
                   << ",\"nnqp_failure\":\"" << step.solve.failure << "\""
                   << ",\"nnqp_failure_source_row\":"
                   << step.solve.failure_source_row
                   << ",\"nnqp_failure_value\":\""
                   << al_binary128_text(step.solve.failure_value)
                   << "\",\"nnqp_failure_bound\":\""
                   << al_binary128_text(step.solve.failure_bound) << "\""
                   << ",\"direction_certified\":"
                   << (step.direction_certified ? "true" : "false")
                   << ",\"support\":" << step.solve.support_size
                   << ",\"transitions\":" << step.solve.transitions
                   << ",\"solves\":" << step.solve.principal_solves
                   << ",\"line_trials\":" << step.trials.size()
                   << ",\"accepted\":"
                   << (step.accepted ? "true" : "false");
            if (step.accepted)
                report << ",\"power\":" << step.selected.power
                       << ",\"mask_changes\":"
                       << step.selected.mask_changes
                       << ",\"dual_increase_lower\":\""
                       << al_binary128_text(
                            step.selected.dual_increase_lower) << "\""
                       << ",\"primal\":\""
                       << al_binary128_text(step.selected.metrics.primal)
                       << "\",\"dual_mapping\":\""
                       << al_binary128_text(
                            step.selected.metrics.dual_mapping) << "\"";
            report << ",\"root\":\"" << step.root << "\"}";
        }
        report << "],\"root\":\"" << value.root << "\"}";
    }
    report << "]"
              ",\"accepted_iteration_cap\":32"
              ",\"cold_start_nnqp\":true,\"stale_derivative_reuse\":false"
              ",\"diagonal_regularization\":false"
              ",\"old_oracle_reruns\":0,\"timing_admitted\":false"
              ",\"generalization_authority\":false"
              ",\"runtime_authority\":false,\"production_authority\":false"
              ",\"result_sha256\":\""
           << result_sha << "\"}";
    return {harness_exact, report.str()};
}
