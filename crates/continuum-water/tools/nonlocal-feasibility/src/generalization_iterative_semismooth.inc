namespace {

constexpr const char* AL_R20_NNQP_SHA256 =
    "bfa10141c0f50658548205e44d397a8999f36a0d0c0e00da0cb9d3d7ebc01500";
constexpr std::size_t AL_R20_SEMISMOOTH_ITERATION_CAP = 32U;

struct ALR20SlopeErrorRefinement {
    bool exact = false;
    bool passed = false;
    std::size_t inverse_columns = 0U;
    Binary128 error = static_cast<Binary128>(0.0);
    std::string audit_root;
    std::string root;
};

using ALR20SlopeErrorRefiner = ALR20SlopeErrorRefinement (*)(
    const ALR20DirectSolutionCapture&);

thread_local ALR20SlopeErrorRefiner al_r20_slope_error_refiner = nullptr;

struct ALR20SemismoothStep {
    bool harness_exact = false;
    bool natural_face_resolved = false;
    bool direction_certified = false;
    bool accepted = false;
    bool event_replacement_eligible = false;
    bool event_replacement_applied = false;
    bool slope_refinement_attempted = false;
    bool slope_refinement_consumed = false;
    std::size_t iteration = 0U;
    std::size_t natural_face_size = 0U;
    std::size_t ambiguous_rows = 0U;
    Binary128 slope = static_cast<Binary128>(0.0);
    Binary128 slope_bound = static_cast<Binary128>(0.0);
    Binary128 direction_error_bound = static_cast<Binary128>(0.0);
    Binary128 initial_direction_error_bound = static_cast<Binary128>(0.0);
    Binary128 initial_slope_bound = static_cast<Binary128>(0.0);
    Binary128 event_root_alpha = static_cast<Binary128>(0.0);
    Binary128 event_certified_alpha = static_cast<Binary128>(0.0);
    Binary128 event_forward_bound = static_cast<Binary128>(0.0);
    std::size_t event_scalar = 0U;
    std::size_t event_contributing_entries = 0U;
    ALR20SlopeErrorRefinement slope_refinement;
    ALR20NNQPSolve solve;
    std::vector<ALR20NewtonTrial> trials;
    ALR20NewtonTrial selected;
    std::vector<Binary128> line_representative;
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
    std::size_t iteration,
    bool verified_inverse = false,
    bool dual_ratio_refinement = false,
    bool certified_event_replacement = false) {
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
    ALR20DirectSolutionCapture slope_direct;
    const bool slope_capture_armed = al_r20_slope_error_refiner != nullptr
        && al_r20_direct_solution_capture == nullptr;
    if (slope_capture_armed)
        al_r20_direct_solution_capture = &slope_direct;
    result.solve = al_r20_nnqp_solve(
        principal, rhs, rhs_bound, natural_face,
        verified_inverse, dual_ratio_refinement);
    if (slope_capture_armed
        && al_r20_direct_solution_capture == &slope_direct)
        al_r20_direct_solution_capture = nullptr;
    const bool slope_capture_released =
        al_r20_direct_solution_capture == nullptr;
    result.direction_error_bound = result.solve.direction_error_bound;
    result.initial_direction_error_bound = result.direction_error_bound;
    std::vector<Binary128> representative(all_rows,
        static_cast<Binary128>(0.0));
    for (std::size_t local = 0U; local < dimension; ++local)
        representative[natural_face[local]] = result.solve.direction[local];
    result.line_representative = representative;
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
    result.initial_slope_bound = result.slope_bound;
    result.direction_certified = result.solve.kkt_certified
        && result.slope - result.slope_bound
            > static_cast<Binary128>(0.0);
    const bool slope_refinement_eligible = !result.direction_certified
        && al_r20_slope_error_refiner != nullptr
        && result.solve.exact && result.solve.terminated
        && result.solve.kkt_certified && result.solve.failure == "NONE";
    if (slope_refinement_eligible) {
        result.slope_refinement_attempted = true;
        bool direct_correspondence = slope_capture_armed
            && slope_capture_released && slope_direct.calls > 0U
            && result.solve.passive.size() == dimension
            && result.solve.direction.size() == dimension
            && slope_direct.factor.passed
            && slope_direct.indices.size() == slope_direct.solution.size()
            && slope_direct.source_rows.size() == slope_direct.solution.size()
            && slope_direct.rhs.size() == slope_direct.solution.size()
            && slope_direct.factor.dimension == slope_direct.solution.size()
            && slope_direct.cheap_error == result.solve.direction_error_bound;
        std::vector<bool> captured_support(dimension, false);
        for (std::size_t local = 0U;
             direct_correspondence && local < slope_direct.indices.size();
             ++local) {
            const std::size_t index = slope_direct.indices[local];
            direct_correspondence = index < dimension
                && result.solve.passive[index]
                && result.solve.direction[index] == slope_direct.solution[local]
                && slope_direct.source_rows[local] == natural_face[index];
            if (direct_correspondence) captured_support[index] = true;
        }
        for (std::size_t index = 0U;
             direct_correspondence && index < dimension; ++index)
            direct_correspondence = captured_support[index]
                || (!result.solve.passive[index]
                    && result.solve.direction[index]
                        == static_cast<Binary128>(0.0));
        if (direct_correspondence)
            result.slope_refinement =
                al_r20_slope_error_refiner(slope_direct);
        else
            result.slope_refinement.root =
                sha256_hex("slope-direct-correspondence-rejected");
        const bool refinement_valid = direct_correspondence
            && result.slope_refinement.exact
            && result.slope_refinement.passed
            && finiteq(result.slope_refinement.error) != 0
            && result.slope_refinement.error
                >= static_cast<Binary128>(0.0)
            && result.slope_refinement.error
                <= result.initial_direction_error_bound;
        if (refinement_valid) {
            result.slope_refinement_consumed = true;
            result.direction_error_bound = result.slope_refinement.error;
            result.slope_bound = static_cast<Binary128>(0.0);
            for (std::size_t row = 0U; row < all_rows; ++row) {
                const Binary128 direction_bound = std::binary_search(
                    natural_face.begin(), natural_face.end(), row)
                    ? result.direction_error_bound
                    : static_cast<Binary128>(0.0);
                result.slope_bound += current.residual_bound[row]
                        * (fabsq(direction[row]) + direction_bound)
                    + fabsq(current.residual[row]) * direction_bound;
            }
            result.slope_bound += al_r20_q_gamma(
                16U * all_rows + 512U) * slope_absolute;
            result.direction_certified = result.solve.kkt_certified
                && result.slope - result.slope_bound
                    > static_cast<Binary128>(0.0);
        }
    }
    result.harness_exact = result.harness_exact && result.solve.exact
        && (al_r20_slope_error_refiner == nullptr
            || (slope_capture_armed && slope_capture_released))
        && (!result.slope_refinement_attempted
            || result.slope_refinement.exact)
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
    if (certified_event_replacement && result.accepted
        && result.selected.power > 0U
        && result.selected.mask_changes == 0U
        && !result.selected.ball_activity_changed) {
        const ALR20NewtonTrial* rejected_double = nullptr;
        bool all_prior_crossing = true;
        for (const ALR20NewtonTrial& trial : result.trials) {
            if (trial.power + 1U == result.selected.power)
                rejected_double = &trial;
            if (trial.power < result.selected.power)
                all_prior_crossing = all_prior_crossing
                    && (trial.mask_changes > 0U
                        || trial.ball_activity_changed);
        }
        const std::vector<Binary128Vec3> lower =
            al_r20_projector_promote_bounds(problem.lower);
        std::vector<Binary128> multiplier_direction(all_rows);
        for (std::size_t row = 0U; row < all_rows; ++row)
            multiplier_direction[row] = representative[row] - lambda[row];
        const std::vector<Binary128Vec3> input_direction_image =
            al_r20_q_transpose(
                problem.op, multiplier_direction, problem.target.size());
        struct ZeroRelease {
            std::size_t scalar = 0U;
            Binary128 root = static_cast<Binary128>(0.0);
            Binary128 derivative = static_cast<Binary128>(0.0);
        };
        std::vector<ZeroRelease> releases;
        if (all_prior_crossing && rejected_double != nullptr)
            for (std::size_t scalar = 0U;
                 scalar < current.projector.mask.size(); ++scalar) {
                if (current.projector.mask[scalar] != -1) continue;
                const std::size_t particle = scalar / 3U;
                const int axis = static_cast<int>(scalar % 3U);
                if (al_r20_q_component(lower[particle], axis)
                        != static_cast<Binary128>(0.0))
                    continue;
                const Binary128 derivative = -al_r20_q_component(
                    input_direction_image[particle], axis);
                if (!(derivative > static_cast<Binary128>(0.0))) continue;
                const Binary128 initial = al_r20_q_component(
                    current.input[particle], axis);
                const Binary128 root = -initial / derivative;
                if (finiteq(root) != 0
                    && root >= result.selected.alpha
                    && root <= rejected_double->alpha)
                    releases.push_back({scalar, root, derivative});
            }
        result.event_replacement_eligible = releases.size() == 1U;
        if (result.event_replacement_eligible) {
            const ZeroRelease& release = releases.front();
            result.event_scalar = release.scalar;
            result.event_root_alpha = release.root;
            const std::size_t particle = release.scalar / 3U;
            const int axis = static_cast<int>(release.scalar % 3U);
            const std::vector<Binary128Vec3> target =
                al_r20_q_promote(problem.target);
            Binary128Accumulator event_scale;
            event_scale.add(fabsq(
                al_r20_q_component(target[particle], axis)));
            for (std::size_t row = 0U; row < all_rows; ++row)
                for (std::size_t index = problem.op.entry_offsets[row];
                     index < problem.op.entry_offsets[row + 1U]; ++index) {
                    const ALR64Entry& entry = problem.op.entries[index];
                    if (entry.particle != particle) continue;
                    const Binary128 coefficient = fabsq(al_r20_q_component(
                        al_binary128_vec3(entry.coefficient), axis));
                    if (coefficient == static_cast<Binary128>(0.0)) continue;
                    event_scale.add(coefficient * (fabsq(lambda[row])
                        + fabsq(representative[row])));
                    ++result.event_contributing_entries;
                }
            result.event_forward_bound = al_r20_q_gamma(
                32U * result.event_contributing_entries + 256U)
                * fmaxq(event_scale.value(),
                    ldexpq(static_cast<Binary128>(1.0), -16382));
            const Binary128 safe_edge = release.root
                + result.event_forward_bound / release.derivative;
            result.event_certified_alpha = nextafterq(safe_edge, HUGE_VALQ);
            ALR20NewtonTrial event_trial;
            event_trial.power = AL_R20_NEWTON_LINE_TRIALS;
            event_trial.alpha = result.event_certified_alpha;
            std::vector<Binary128> event_lambda(all_rows);
            bool cone_exact = event_trial.alpha > release.root
                && event_trial.alpha <= rejected_double->alpha;
            for (std::size_t row = 0U; row < all_rows; ++row) {
                event_lambda[row] =
                    (static_cast<Binary128>(1.0) - event_trial.alpha)
                        * lambda[row]
                    + event_trial.alpha * representative[row];
                cone_exact = cone_exact
                    && event_lambda[row] >= static_cast<Binary128>(0.0);
            }
            event_trial.dual = al_r20_newton_dual_evaluate(
                problem, event_lambda);
            event_trial.exact = cone_exact && event_trial.dual.exact;
            event_trial.dual_increase_lower =
                event_trial.dual.value - event_trial.dual.bound
                - current.value - current.bound;
            event_trial.ball_activity_changed =
                event_trial.dual.projector.ball_active
                != current.projector.ball_active;
            bool predicted_changed = false;
            for (std::size_t scalar = 0U;
                 scalar < current.projector.mask.size(); ++scalar)
                if (event_trial.dual.projector.mask[scalar]
                        != current.projector.mask[scalar]) {
                    ++event_trial.mask_changes;
                    predicted_changed = predicted_changed
                        || scalar == release.scalar;
                }
            event_trial.accepted = event_trial.exact
                && !event_trial.ball_activity_changed
                && event_trial.mask_changes == 1U && predicted_changed
                && al_r20_newton_armijo(
                    current.value, current.bound,
                    event_trial.dual.value, event_trial.dual.bound,
                    event_trial.alpha, result.slope, result.slope_bound);
            event_trial.metrics = al_r20_q_metrics(problem,
                event_trial.dual.projector.value, event_lambda,
                diagonal, iteration);
            event_trial.exact = event_trial.exact
                && event_trial.metrics.finite;
            std::ostringstream event_trial_material;
            event_trial_material << "certified-event:"
                << event_trial.exact << ':' << event_trial.accepted << ':'
                << release.scalar << ':'
                << al_binary128_hex(release.root) << ':'
                << al_binary128_hex(result.event_forward_bound) << ':'
                << al_binary128_hex(event_trial.alpha) << ':'
                << al_binary128_hex(event_trial.dual_increase_lower) << ':'
                << event_trial.mask_changes << ':'
                << event_trial.ball_activity_changed << ':'
                << event_trial.dual.root << ':' << event_trial.metrics.root;
            event_trial.root = sha256_hex(event_trial_material.str());
            result.trials.push_back(event_trial);
            line_exact = line_exact && event_trial.exact;
            if (event_trial.accepted) {
                result.event_replacement_applied = true;
                result.selected = event_trial;
                result.next_lambda = std::move(event_lambda);
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
    if (certified_event_replacement)
        material << "|certified-event:" << result.event_replacement_eligible
                 << ':' << result.event_replacement_applied << ':'
                 << result.event_scalar << ':'
                 << result.event_contributing_entries << ':'
                 << al_binary128_hex(result.event_root_alpha) << ':'
                 << al_binary128_hex(result.event_certified_alpha) << ':'
                 << al_binary128_hex(result.event_forward_bound);
    if (result.slope_refinement_attempted)
        material << "|slope-refinement:"
                 << result.slope_refinement_consumed << ':'
                 << result.slope_refinement.inverse_columns << ':'
                 << al_binary128_hex(result.initial_direction_error_bound)
                 << ':' << al_binary128_hex(result.initial_slope_bound)
                 << ':' << al_binary128_hex(result.direction_error_bound)
                 << ':' << result.slope_refinement.audit_root
                 << ':' << result.slope_refinement.root;
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
    std::size_t event_replacements = 0U;
    std::size_t event_fallbacks = 0U;
    std::size_t event_extra_evaluations = 0U;
    ALR20OracleMetrics initial_metrics;
    ALR20OracleMetrics final_metrics;
    std::vector<ALR20SemismoothStep> steps;
    std::vector<Binary128> final_lambda;
    std::string root;
};

ALR20SemismoothCase al_r20_semismooth_case(
    const ALR20ProblemData& problem,
    bool verified_inverse = false,
    bool dual_ratio_refinement = false,
    bool certified_event_replacement = false) {
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
            problem, current, lambda, diagonal, iteration,
            verified_inverse, dual_ratio_refinement,
            certified_event_replacement);
        result.harness_exact = result.harness_exact && step.harness_exact;
        result.total_principal_solves += step.solve.principal_solves;
        result.total_transitions += step.solve.transitions;
        result.event_replacements +=
            step.event_replacement_applied ? 1U : 0U;
        result.event_fallbacks += step.event_replacement_eligible
                && !step.event_replacement_applied ? 1U : 0U;
        result.event_extra_evaluations +=
            step.event_replacement_eligible ? 1U : 0U;
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
    if (certified_event_replacement)
        material << "|certified-event:" << result.event_replacements << ':'
                 << result.event_fallbacks << ':'
                 << result.event_extra_evaluations;
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
