namespace {

constexpr const char* AL_R20_R34_SEMANTIC_SHA256 =
    "811ac23180956ddf02efae5d52c949bc77619ce33907817d92974e41be4802ab";
constexpr std::array<const char*, 5U> AL_R20_V4_PROBLEM_SHA256{{
    "5584d5175a71b255bf8aa5bb5dc0cbad2f6390295d4d43634564dd9bc106d3ad",
    "ba002346e86a809057cce9f25023e30bc82dc9b574807880652586109a7fb21f",
    "62b1dd7d76c9c71c38273de7c6d37f8e74cad8806a82c3ea02affdf0b98a7a03",
    "fa59af8d97c81de82bf334f45b4a5a3985ef312bcf48200161f0a8487c3a7364",
    "192adf045d735685cd8df96bb5fe0873da069ce684beb9d8529379052f4650e7",
}};

struct ALR20R35RecoveryAudit {
    bool eligible = false;
    bool enumeration_exact = false;
    bool unique_zero_release = false;
    bool forward_exact = false;
    bool applied = false;
    std::size_t enumerated_polynomials = 0U;
    std::size_t algebraic_roots = 0U;
    std::size_t bounded_roots = 0U;
    std::size_t admissible_roots = 0U;
    std::size_t nearest_multiplicity = 0U;
    std::size_t contributing_entries = 0U;
    Binary128 second_alpha = HUGE_VALQ;
    Binary128 forward_scale = static_cast<Binary128>(0.0);
    Binary128 forward_bound = static_cast<Binary128>(0.0);
    Binary128 alpha_new = static_cast<Binary128>(0.0);
    ALR20R28EventRoot event;
    ALR20R28ShadowTrial trial;
    std::string classification = "NOT_ELIGIBLE";
    std::string root;
};

ALR20R35RecoveryAudit al_r20_r35_recovery_candidate(
    const ALR20ProblemData& problem,
    const ALR20NewtonDualEvaluation& current,
    const std::vector<Binary128>& lambda,
    const std::vector<Binary128>& diagonal,
    const ALR20SemismoothStep& step) {
    ALR20R35RecoveryAudit audit;
    const Binary128 zero = static_cast<Binary128>(0.0);
    const Binary128 envelope = ldexpq(static_cast<Binary128>(1.0), -20);
    bool inherited_line_exact = step.direction_certified && !step.accepted
        && step.trials.size() == AL_R20_NEWTON_LINE_TRIALS
        && !step.trials.empty()
        && step.trials.back().power == 20U
        && step.trials.back().alpha == envelope;
    for (const ALR20NewtonTrial& trial : step.trials)
        inherited_line_exact = inherited_line_exact
            && trial.exact && !trial.accepted
            && trial.mask_changes > 0U
            && !trial.ball_activity_changed;
    audit.eligible = inherited_line_exact;
    if (!audit.eligible) {
        audit.root = sha256_hex("r35-recovery:not-eligible:" + step.root);
        return audit;
    }

    const std::vector<Binary128Vec3> target =
        al_r20_q_promote(problem.target);
    const std::vector<Binary128Vec3> lower =
        al_r20_projector_promote_bounds(problem.lower);
    const std::vector<Binary128Vec3> upper =
        al_r20_projector_promote_bounds(problem.upper);
    const std::vector<Binary128Vec3> at =
        al_r20_q_transpose(problem.op, lambda, target.size());
    std::vector<Binary128> line_direction(lambda.size());
    for (std::size_t row = 0U; row < lambda.size(); ++row)
        line_direction[row] = step.line_representative[row] - lambda[row];
    const std::vector<Binary128Vec3> at_direction = al_r20_q_transpose(
        problem.op, line_direction, target.size());
    std::vector<Binary128Vec3> z0(target.size());
    std::vector<Binary128Vec3> dz(target.size());
    for (std::size_t particle = 0U; particle < target.size(); ++particle) {
        z0[particle] = al_r20_q_add(target[particle],
            al_r20_q_scale(static_cast<Binary128>(-1.0), at[particle]));
        dz[particle] = al_r20_q_scale(
            static_cast<Binary128>(-1.0), at_direction[particle]);
    }
    audit.enumeration_exact = al_r20_admm_finite(line_direction);
    Binary128Accumulator s0_accumulator;
    Binary128Accumulator s1_accumulator;
    Binary128Accumulator s2_accumulator;
    for (std::size_t scalar = 0U;
         scalar < current.projector.mask.size(); ++scalar) {
        if (current.projector.mask[scalar] != 0) continue;
        const std::size_t particle = scalar / 3U;
        const int axis = static_cast<int>(scalar % 3U);
        const Binary128 initial = al_r20_q_component(z0[particle], axis);
        const Binary128 direction = al_r20_q_component(dz[particle], axis);
        s0_accumulator.add(initial * initial);
        s1_accumulator.add(
            static_cast<Binary128>(2.0) * initial * direction);
        s2_accumulator.add(direction * direction);
    }
    const Binary128 s0 = s0_accumulator.value();
    const Binary128 s1 = s1_accumulator.value();
    const Binary128 s2 = s2_accumulator.value();
    std::vector<ALR20R28EventRoot> roots;
    for (std::size_t scalar = 0U;
         scalar < current.projector.mask.size(); ++scalar) {
        if (current.projector.mask[scalar] == 2) continue;
        const std::size_t particle = scalar / 3U;
        const int axis = static_cast<int>(scalar % 3U);
        const Binary128 initial = al_r20_q_component(z0[particle], axis);
        const Binary128 direction = al_r20_q_component(dz[particle], axis);
        const std::array<std::pair<int, Binary128>, 2U> bounds{{
            {-1, al_r20_q_component(lower[particle], axis)},
            {1, al_r20_q_component(upper[particle], axis)},
        }};
        for (const auto& bound_entry : bounds) {
            ALR20EventPolynomial polynomial;
            polynomial.scalar = scalar;
            polynomial.bound_code = bound_entry.first;
            polynomial.active_ball = current.projector.ball_active;
            polynomial.zero_bound = bound_entry.second == zero;
            if (!current.projector.ball_active || polynomial.zero_bound) {
                polynomial.b = direction;
                polynomial.c = initial - bound_entry.second;
            } else {
                const Binary128 bound_squared =
                    bound_entry.second * bound_entry.second;
                polynomial.a = current.projector.remaining_radius_squared
                        * direction * direction
                    - bound_squared * s2;
                polynomial.b = static_cast<Binary128>(2.0)
                        * current.projector.remaining_radius_squared
                        * initial * direction
                    - bound_squared * s1;
                polynomial.c = current.projector.remaining_radius_squared
                        * initial * initial
                    - bound_squared * s0;
            }
            const bool polynomial_exact = finiteq(polynomial.a) != 0
                && finiteq(polynomial.b) != 0
                && finiteq(polynomial.c) != 0;
            audit.enumeration_exact =
                audit.enumeration_exact && polynomial_exact;
            ++audit.enumerated_polynomials;
            const std::vector<Binary128> polynomial_roots =
                al_r20_r21_real_roots(
                    polynomial.a, polynomial.b, polynomial.c);
            audit.algebraic_roots += polynomial_roots.size();
            for (Binary128 alpha : polynomial_roots) {
                if (!(alpha > zero && alpha <= envelope)
                    || finiteq(alpha) == 0)
                    continue;
                ++audit.bounded_roots;
                ALR20R28EventRoot event;
                event.exact = polynomial_exact;
                event.scalar = scalar;
                event.bound_code = bound_entry.first;
                event.zero_bound = polynomial.zero_bound;
                event.alpha = alpha;
                event.derivative = direction;
                event.a = polynomial.a;
                event.b = polynomial.b;
                event.c = polynomial.c;
                const Binary128 root_value = initial + alpha * direction;
                event.relation_residual = root_value - bound_entry.second;
                if (current.projector.ball_active && !event.zero_bound) {
                    const Binary128 norm_squared =
                        (s2 * alpha + s1) * alpha + s0;
                    const Binary128 divisor = sqrtq(norm_squared
                        / current.projector.remaining_radius_squared);
                    event.relation_residual = root_value
                        - divisor * bound_entry.second;
                }
                const bool face_can_transition =
                    current.projector.mask[scalar] == 0
                    || current.projector.mask[scalar] == bound_entry.first;
                const bool sign_matches = event.zero_bound
                    || root_value * bound_entry.second > zero;
                event.admissible = face_can_transition && sign_matches
                    && al_r20_r28_current_face_valid(
                        current, lower, upper, z0, dz,
                        s0, s1, s2, alpha, scalar, false);
                std::ostringstream material;
                material << event.exact << ':' << event.admissible << ':'
                    << event.ball_event << ':' << event.zero_bound << ':'
                    << event.scalar << ':' << event.bound_code << ':'
                    << al_binary128_hex(event.alpha) << ':'
                    << al_binary128_hex(event.derivative) << ':'
                    << al_binary128_hex(event.relation_residual) << ':'
                    << al_binary128_hex(event.a) << ':'
                    << al_binary128_hex(event.b) << ':'
                    << al_binary128_hex(event.c);
                event.root = sha256_hex(material.str());
                roots.push_back(std::move(event));
            }
        }
    }

    ++audit.enumerated_polynomials;
    const Binary128 ball_a = s2;
    const Binary128 ball_b = s1;
    const Binary128 ball_c =
        s0 - current.projector.remaining_radius_squared;
    const bool ball_polynomial_exact = finiteq(ball_a) != 0
        && finiteq(ball_b) != 0 && finiteq(ball_c) != 0;
    audit.enumeration_exact =
        audit.enumeration_exact && ball_polynomial_exact;
    const std::vector<Binary128> ball_roots =
        al_r20_r21_real_roots(ball_a, ball_b, ball_c);
    audit.algebraic_roots += ball_roots.size();
    for (Binary128 alpha : ball_roots) {
        if (!(alpha > zero && alpha <= envelope) || finiteq(alpha) == 0)
            continue;
        ++audit.bounded_roots;
        ALR20R28EventRoot event;
        event.exact = ball_polynomial_exact;
        event.ball_event = true;
        event.alpha = alpha;
        event.a = ball_a;
        event.b = ball_b;
        event.c = ball_c;
        event.derivative = static_cast<Binary128>(2.0) * ball_a * alpha
            + ball_b;
        event.relation_residual =
            (ball_a * alpha + ball_b) * alpha + ball_c;
        event.admissible = al_r20_r28_current_face_valid(
            current, lower, upper, z0, dz, s0, s1, s2, alpha,
            current.projector.mask.size(), true);
        std::ostringstream material;
        material << event.exact << ':' << event.admissible << ':'
            << event.ball_event << ':' << event.zero_bound << ':'
            << event.scalar << ':' << event.bound_code << ':'
            << al_binary128_hex(event.alpha) << ':'
            << al_binary128_hex(event.derivative) << ':'
            << al_binary128_hex(event.relation_residual) << ':'
            << al_binary128_hex(event.a) << ':'
            << al_binary128_hex(event.b) << ':'
            << al_binary128_hex(event.c);
        event.root = sha256_hex(material.str());
        roots.push_back(std::move(event));
    }
    std::sort(roots.begin(), roots.end(),
        [](const ALR20R28EventRoot& lhs, const ALR20R28EventRoot& rhs) {
            if (lhs.alpha != rhs.alpha) return lhs.alpha < rhs.alpha;
            if (lhs.ball_event != rhs.ball_event)
                return lhs.ball_event < rhs.ball_event;
            if (lhs.scalar != rhs.scalar) return lhs.scalar < rhs.scalar;
            return lhs.bound_code < rhs.bound_code;
        });
    std::vector<std::size_t> admissible;
    for (std::size_t index = 0U; index < roots.size(); ++index)
        if (roots[index].admissible) admissible.push_back(index);
    audit.admissible_roots = admissible.size();
    if (!admissible.empty()) {
        const Binary128 nearest_alpha = roots[admissible.front()].alpha;
        for (std::size_t index : admissible) {
            if (roots[index].alpha == nearest_alpha)
                ++audit.nearest_multiplicity;
            else {
                audit.second_alpha = roots[index].alpha;
                break;
            }
        }
        audit.event = roots[admissible.front()];
    }
    const bool release_direction = audit.event.bound_code == -1
        ? audit.event.derivative > zero
        : audit.event.derivative < zero;
    audit.unique_zero_release = audit.enumeration_exact
        && !admissible.empty() && audit.nearest_multiplicity == 1U
        && !audit.event.ball_event && audit.event.zero_bound
        && audit.event.derivative != zero
        && (audit.event.bound_code == -1 || audit.event.bound_code == 1)
        && audit.event.scalar < current.projector.mask.size()
        && current.projector.mask[audit.event.scalar]
            == audit.event.bound_code
        && release_direction;
    if (audit.unique_zero_release) {
        const std::size_t particle = audit.event.scalar / 3U;
        const int axis = static_cast<int>(audit.event.scalar % 3U);
        Binary128Accumulator scale;
        scale.add(fabsq(al_r20_q_component(target[particle], axis)));
        for (std::size_t row = 0U; row < lambda.size(); ++row)
            for (std::size_t entry_index = problem.op.entry_offsets[row];
                 entry_index < problem.op.entry_offsets[row + 1U];
                 ++entry_index) {
                const ALR64Entry& entry = problem.op.entries[entry_index];
                if (entry.particle != particle) continue;
                const Binary128 coefficient = fabsq(al_r20_q_component(
                    al_binary128_vec3(entry.coefficient), axis));
                if (coefficient == zero) continue;
                scale.add(coefficient * (fabsq(lambda[row])
                    + fabsq(step.line_representative[row])));
                ++audit.contributing_entries;
            }
        audit.forward_scale = scale.value();
        audit.forward_bound = al_r20_q_gamma(
            32U * audit.contributing_entries + 256U)
            * fmaxq(audit.forward_scale,
                ldexpq(static_cast<Binary128>(1.0), -16382));
        const Binary128 new_edge = audit.event.alpha
            + audit.forward_bound / fabsq(audit.event.derivative);
        if (finiteq(new_edge) != 0)
            audit.alpha_new = nextafterq(new_edge, HUGE_VALQ);
        audit.forward_exact = audit.contributing_entries > 0U
            && audit.forward_bound > zero
            && audit.alpha_new > audit.event.alpha
            && audit.alpha_new <= envelope
            && (finiteq(audit.second_alpha) == 0
                || audit.alpha_new < audit.second_alpha);
        if (audit.forward_exact)
            audit.trial = al_r20_r28_shadow_trial(
                problem, current, lambda, diagonal, step,
                audit.alpha_new, audit.event.scalar,
                audit.event.bound_code, 0, false);
        audit.applied = audit.forward_exact
            && audit.trial.exact && audit.trial.face_exact
            && audit.trial.accepted && audit.trial.mask_changes == 1U
            && audit.trial.changed_scalar == audit.event.scalar
            && !audit.trial.ball_activity_changed;
    }
    audit.classification = !audit.enumeration_exact
        ? "ENUMERATION_REJECTED"
        : (admissible.empty()
            ? "EVENT_UNBRACKETED"
            : (!audit.unique_zero_release
                ? "EVENT_UNSUPPORTED_OR_AMBIGUOUS"
                : (!audit.forward_exact
                    ? "FORWARD_SIDE_UNCERTIFIED"
                    : (audit.applied
                        ? "ZERO_BOUND_RECOVERY_CANDIDATE"
                        : "RECOVERY_ARMIJO_OR_FACE_REJECTED"))));
    std::ostringstream semantic;
    semantic << audit.eligible << ':' << audit.enumeration_exact << ':'
        << audit.unique_zero_release << ':' << audit.forward_exact << ':'
        << audit.applied << ':' << audit.classification << ':'
        << audit.enumerated_polynomials << ':' << audit.algebraic_roots << ':'
        << audit.bounded_roots << ':' << audit.admissible_roots << ':'
        << audit.nearest_multiplicity << ':'
        << al_binary128_hex(audit.second_alpha) << ':'
        << audit.contributing_entries << ':'
        << al_binary128_hex(audit.forward_scale) << ':'
        << al_binary128_hex(audit.forward_bound) << ':'
        << al_binary128_hex(audit.alpha_new) << ':'
        << audit.event.root << ':' << audit.trial.root;
    audit.root = sha256_hex(semantic.str());
    return audit;
}

struct ALR20R35CaseAudit {
    ALR20SemismoothCase diagnostic;
    bool terminal_candidate_found = false;
    bool terminal_selected = false;
    bool terminal_correspondence = false;
    bool second_exhaustion = false;
    std::size_t terminal_iteration = 0U;
    std::size_t terminal_power = AL_R20_NEWTON_LINE_TRIALS;
    Binary128 terminal_margin = static_cast<Binary128>(0.0);
    ALR20NewtonTrial terminal_trial;
    ALR20NewtonDualEvaluation terminal_dual;
    ALR20OracleMetrics terminal_metrics;
    std::string terminal_lambda_root;
    std::size_t recovery_iteration = 0U;
    ALR20R35RecoveryAudit recovery;
    std::string route;
    std::string root;
};

ALR20R35CaseAudit al_r20_r35_case(const ALR20ProblemData& problem) {
    ALR20R35CaseAudit audit;
    ALR20SemismoothCase& result = audit.diagnostic;
    const std::size_t rows = problem.row_scale.size();
    const std::vector<Binary128> diagonal = al_r20_q_diagonal(problem.op);
    std::vector<Binary128> lambda(rows, static_cast<Binary128>(0.0));
    ALR20NewtonDualEvaluation current =
        al_r20_newton_dual_evaluate(problem, lambda);
    result.initial_metrics = al_r20_q_metrics(
        problem, current.projector.value, lambda, diagonal, 0U);
    result.final_metrics = result.initial_metrics;
    result.harness_exact = current.exact && result.initial_metrics.finite;
    if (result.initial_metrics.certified) result.certified = true;

    for (std::size_t iteration = 1U;
         !result.certified && iteration <= AL_R20_SEMISMOOTH_ITERATION_CAP;
         ++iteration) {
        ALR20SemismoothStep step = al_r20_semismooth_step(
            problem, current, lambda, diagonal, iteration,
            true, true, true);
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
            const ALR20NewtonTrial* terminal = nullptr;
            for (const ALR20NewtonTrial& trial : step.trials)
                if (trial.metrics.certified) {
                    terminal = &trial;
                    break;
                }
            if (terminal != nullptr) {
                audit.terminal_candidate_found = true;
                std::vector<Binary128> terminal_lambda(rows);
                for (std::size_t row = 0U; row < rows; ++row)
                    terminal_lambda[row] =
                        (static_cast<Binary128>(1.0) - terminal->alpha)
                            * lambda[row]
                        + terminal->alpha * step.line_representative[row];
                audit.terminal_dual = al_r20_newton_dual_evaluate(
                    problem, terminal_lambda);
                audit.terminal_metrics = al_r20_q_metrics(problem,
                    audit.terminal_dual.projector.value, terminal_lambda,
                    diagonal, iteration);
                audit.terminal_lambda_root =
                    al_r20_q_scalar_root(terminal_lambda);
                audit.terminal_correspondence = terminal->exact
                    && !terminal->accepted
                    && audit.terminal_dual.exact
                    && audit.terminal_dual.root == terminal->dual.root
                    && audit.terminal_metrics.root == terminal->metrics.root
                    && audit.terminal_lambda_root
                        == terminal->metrics.lambda_root
                    && audit.terminal_metrics.finite
                    && audit.terminal_metrics.certified;
                if (audit.terminal_correspondence) {
                    audit.terminal_selected = true;
                    audit.terminal_iteration = iteration;
                    audit.terminal_power = terminal->power;
                    audit.terminal_margin =
                        al_r20_r19_armijo_margin(current, step, *terminal);
                    audit.terminal_trial = *terminal;
                    result.certified = true;
                    result.final_metrics = audit.terminal_metrics;
                    lambda = std::move(terminal_lambda);
                    result.steps.push_back(std::move(step));
                    break;
                }
                result.globalization_rejected = true;
                result.steps.push_back(std::move(step));
                break;
            }

            if (audit.recovery.applied) {
                audit.second_exhaustion = true;
                result.globalization_rejected = true;
                result.steps.push_back(std::move(step));
                break;
            }
            audit.recovery = al_r20_r35_recovery_candidate(
                problem, current, lambda, diagonal, step);
            if (!audit.recovery.applied) {
                result.globalization_rejected = true;
                result.steps.push_back(std::move(step));
                break;
            }
            std::vector<Binary128> recovery_lambda(rows);
            for (std::size_t row = 0U; row < rows; ++row)
                recovery_lambda[row] =
                    (static_cast<Binary128>(1.0) - audit.recovery.alpha_new)
                        * lambda[row]
                    + audit.recovery.alpha_new
                        * step.line_representative[row];
            ALR20NewtonTrial selected;
            selected.exact = audit.recovery.trial.exact;
            selected.accepted = audit.recovery.trial.accepted;
            selected.power = AL_R20_NEWTON_LINE_TRIALS + 1U;
            selected.alpha = audit.recovery.trial.alpha;
            selected.dual_increase_lower =
                audit.recovery.trial.dual_increase_lower;
            selected.mask_changes = audit.recovery.trial.mask_changes;
            selected.ball_activity_changed =
                audit.recovery.trial.ball_activity_changed;
            selected.dual = audit.recovery.trial.dual;
            selected.metrics = audit.recovery.trial.metrics;
            selected.root = audit.recovery.trial.root;
            step.trials.push_back(selected);
            step.accepted = true;
            step.selected = selected;
            step.next_lambda = std::move(recovery_lambda);
            step.root = sha256_hex("generalized-exhausted-recovery:"
                + step.root + ':' + audit.recovery.root);
            audit.recovery_iteration = iteration;
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
        if (result.final_metrics.certified) result.certified = true;
    }
    result.final_lambda = lambda;
    audit.route = result.certified
        ? (audit.terminal_selected
            ? "TERMINAL_CERTIFIED"
            : (audit.recovery.applied
                ? "RECOVERY_CERTIFIED" : "ORDINARY_CERTIFIED"))
        : (result.active_set_rejected
            ? "ACTIVE_SET_REJECTED"
            : (result.direction_rejected
                ? "DIRECTION_REJECTED"
                : (audit.second_exhaustion
                    ? "SECOND_EXHAUSTION_REJECTED"
                    : (result.globalization_rejected
                        ? (audit.terminal_candidate_found
                            ? "TERMINAL_STATE_REJECTED"
                            : audit.recovery.classification)
                        : "CAP_UNRESOLVED"))));
    std::ostringstream material;
    material << result.harness_exact << ':' << result.certified << ':'
        << result.active_set_rejected << ':' << result.direction_rejected
        << ':' << result.globalization_rejected << ':'
        << result.accepted_iterations << ':' << result.total_mask_changes
        << ':' << result.total_principal_solves << ':'
        << result.total_transitions << ':' << result.initial_metrics.root
        << ':' << result.final_metrics.root << ':'
        << al_r20_q_scalar_root(result.final_lambda) << '|';
    for (const ALR20SemismoothStep& step : result.steps)
        material << step.root << ';';
    material << "|events:" << result.event_replacements << ':'
        << result.event_fallbacks << ':' << result.event_extra_evaluations
        << "|terminal:" << audit.terminal_candidate_found << ':'
        << audit.terminal_selected << ':'
        << audit.terminal_correspondence << ':' << audit.terminal_iteration
        << ':' << audit.terminal_power << ':'
        << al_binary128_hex(audit.terminal_margin) << ':'
        << audit.terminal_trial.root << ':' << audit.terminal_dual.root << ':'
        << audit.terminal_metrics.root << ':' << audit.terminal_lambda_root
        << "|recovery:" << audit.recovery_iteration << ':'
        << audit.recovery.root << ':' << audit.second_exhaustion << ':'
        << audit.route;
    result.root = sha256_hex(material.str());
    audit.root = result.root;
    return audit;
}

bool al_r20_r35_case_observation_exact(const ALR20R35CaseAudit& audit) {
    const ALR20SemismoothCase& diagnostic = audit.diagnostic;
    if (diagnostic.steps.empty())
        return diagnostic.certified && diagnostic.harness_exact;
    bool exact = true;
    for (std::size_t index = 0U;
         index + 1U < diagnostic.steps.size(); ++index)
        exact = exact && diagnostic.steps[index].harness_exact;
    const ALR20SemismoothStep& final = diagnostic.steps.back();
    if (diagnostic.active_set_rejected) {
        const bool identified = !final.natural_face_resolved
            ? final.harness_exact
                && (final.ambiguous_rows > 0U
                    || final.natural_face_size == 0U)
            : !final.solve.exact && final.solve.failure != "NONE"
                && final.solve.root.size() == 64U;
        return exact && identified;
    }
    if (diagnostic.direction_rejected)
        return exact && final.harness_exact && final.solve.exact
            && !final.direction_certified
            && finiteq(final.slope) != 0
            && finiteq(final.slope_bound) != 0;
    if (diagnostic.globalization_rejected)
        return exact && final.harness_exact && final.direction_certified
            && !final.accepted && final.trials.size() > 0U;
    return exact && diagnostic.harness_exact;
}

} // namespace

SplitBoundaryReport run_al_generalization_v4_solver_controls() {
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
        && al_r20_r21_event_controls();
    std::vector<ALR20ManifestCase> cases = al_r20_v4_manifest_cases();
    bool parent_exact = controls_exact && cases.size() == 5U
        && al_r20_v4_manifest_root(cases) == AL_R20_V4_MANIFEST_SHA256;
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    std::vector<ALR20ProblemData> problems;
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        ALR20ProblemData problem = al_r20_materialize_problem(
            cases[index], &sparse_work, &static_work, &adjacency_work);
        parent_exact = parent_exact && problem.exact
            && problem.source_positive == 0U
            && index < AL_R20_V4_PROBLEM_SHA256.size()
            && problem.problem_root == AL_R20_V4_PROBLEM_SHA256[index];
        problems.push_back(std::move(problem));
    }
    std::vector<ALR20R35CaseAudit> diagnostics;
    std::size_t solver_executions = 0U;
    if (parent_exact)
        for (const ALR20ProblemData& problem : problems) {
            diagnostics.push_back(al_r20_r35_case(problem));
            ++solver_executions;
        }
    bool harness_exact = parent_exact
        && diagnostics.size() == cases.size()
        && solver_executions == cases.size();
    bool all_certified = parent_exact;
    std::size_t terminal_selections = 0U;
    std::size_t recoveries = 0U;
    std::size_t second_exhaustions = 0U;
    std::size_t total_line_evaluations = 0U;
    std::size_t total_event_replacements = 0U;
    for (const ALR20R35CaseAudit& audit : diagnostics) {
        harness_exact = harness_exact
            && al_r20_r35_case_observation_exact(audit);
        all_certified = all_certified && audit.diagnostic.certified;
        terminal_selections += audit.terminal_selected ? 1U : 0U;
        recoveries += audit.recovery.applied ? 1U : 0U;
        second_exhaustions += audit.second_exhaustion ? 1U : 0U;
        total_event_replacements += audit.diagnostic.event_replacements;
        for (const ALR20SemismoothStep& step : audit.diagnostic.steps)
            total_line_evaluations += step.trials.size();
    }
    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    harness_exact = harness_exact && lifecycle_exact;
    const std::string route = !parent_exact
        ? "V4_SOLVER_PARENT_REJECTED"
        : (!harness_exact
            ? "V4_SOLVER_HARNESS_REJECTED"
            : (all_certified
                ? "V4_GENERALIZED_POLICY_CERTIFIED"
                : "V4_GENERALIZED_POLICY_REJECTED"));
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r35-v4-solver.v1|"
        << AL_R20_R34_SEMANTIC_SHA256 << ':' << AL_R20_R32_SEMANTIC_SHA256
        << ':' << route << '|';
    for (std::size_t index = 0U; index < diagnostics.size(); ++index)
        semantic << problems[index].problem_root << ':'
            << diagnostics[index].root << ';';
    semantic << terminal_selections << ':' << recoveries << ':'
        << second_exhaustions << ':' << total_event_replacements << ':'
        << total_line_evaluations;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r35_v4_solver.v1\""
        << ",\"status\":\"" << (harness_exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_v4_preflight_sha256\":\""
        << AL_R20_R34_SEMANTIC_SHA256 << "\""
        << ",\"parent_terminal_trajectory_sha256\":\""
        << AL_R20_R32_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"all_certified\":" << (all_certified ? "true" : "false")
        << ",\"cases\":[";
    for (std::size_t index = 0U; index < diagnostics.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20R35CaseAudit& audit = diagnostics[index];
        const ALR20SemismoothCase& value = audit.diagnostic;
        const ALR20SemismoothStep* final_step = value.steps.empty()
            ? nullptr : &value.steps.back();
        std::size_t line_evaluations = 0U;
        for (const ALR20SemismoothStep& step : value.steps)
            line_evaluations += step.trials.size();
        report << "{\"name\":\"" << cases[index].fixture.name
            << "\",\"route\":\"" << audit.route
            << "\",\"observation_exact\":"
            << (al_r20_r35_case_observation_exact(audit)
                    ? "true" : "false")
            << ",\"certified\":" << (value.certified ? "true" : "false")
            << ",\"accepted_iterations\":" << value.accepted_iterations
            << ",\"step_count\":" << value.steps.size()
            << ",\"line_evaluations\":" << line_evaluations
            << ",\"event_replacements\":" << value.event_replacements
            << ",\"terminal_selected\":"
            << (audit.terminal_selected ? "true" : "false")
            << ",\"terminal_candidate_found\":"
            << (audit.terminal_candidate_found ? "true" : "false")
            << ",\"terminal_correspondence\":"
            << (audit.terminal_correspondence ? "true" : "false")
            << ",\"terminal_iteration\":" << audit.terminal_iteration
            << ",\"terminal_power\":" << audit.terminal_power
            << ",\"terminal_margin\":\""
            << al_binary128_text(audit.terminal_margin)
            << "\",\"recovery_applied\":"
            << (audit.recovery.applied ? "true" : "false")
            << ",\"recovery_iteration\":" << audit.recovery_iteration
            << ",\"recovery_classification\":\""
            << audit.recovery.classification
            << "\",\"recovery_scalar\":" << audit.recovery.event.scalar
            << ",\"recovery_bound_code\":"
            << audit.recovery.event.bound_code
            << ",\"recovery_root_alpha\":\""
            << al_binary128_text(audit.recovery.event.alpha)
            << "\",\"recovery_alpha\":\""
            << al_binary128_text(audit.recovery.alpha_new)
            << "\",\"recovery_contributing_entries\":"
            << audit.recovery.contributing_entries
            << ",\"second_exhaustion\":"
            << (audit.second_exhaustion ? "true" : "false")
            << ",\"final_step\":{\"iteration\":"
            << (final_step == nullptr ? 0U : final_step->iteration)
            << ",\"harness_exact\":"
            << (final_step != nullptr && final_step->harness_exact
                    ? "true" : "false")
            << ",\"natural_face_resolved\":"
            << (final_step != nullptr && final_step->natural_face_resolved
                    ? "true" : "false")
            << ",\"natural_face_size\":"
            << (final_step == nullptr ? 0U : final_step->natural_face_size)
            << ",\"ambiguous_rows\":"
            << (final_step == nullptr ? 0U : final_step->ambiguous_rows)
            << ",\"nnqp_exact\":"
            << (final_step != nullptr && final_step->solve.exact
                    ? "true" : "false")
            << ",\"nnqp_terminated\":"
            << (final_step != nullptr && final_step->solve.terminated
                    ? "true" : "false")
            << ",\"nnqp_kkt\":"
            << (final_step != nullptr && final_step->solve.kkt_certified
                    ? "true" : "false")
            << ",\"nnqp_failure\":\""
            << (final_step == nullptr ? "NO_STEP" : final_step->solve.failure)
            << "\",\"failure_source_row\":"
            << (final_step == nullptr
                    ? 0U : final_step->solve.failure_source_row)
            << ",\"failure_value\":\""
            << al_binary128_text(final_step == nullptr
                    ? static_cast<Binary128>(0.0)
                    : final_step->solve.failure_value)
            << "\",\"failure_bound\":\""
            << al_binary128_text(final_step == nullptr
                    ? static_cast<Binary128>(0.0)
                    : final_step->solve.failure_bound)
            << "\",\"support\":"
            << (final_step == nullptr ? 0U : final_step->solve.support_size)
            << ",\"transitions\":"
            << (final_step == nullptr ? 0U : final_step->solve.transitions)
            << ",\"principal_solves\":"
            << (final_step == nullptr
                    ? 0U : final_step->solve.principal_solves)
            << ",\"direction_certified\":"
            << (final_step != nullptr && final_step->direction_certified
                    ? "true" : "false")
            << ",\"slope\":\""
            << al_binary128_text(final_step == nullptr
                    ? static_cast<Binary128>(0.0) : final_step->slope)
            << "\",\"slope_bound\":\""
            << al_binary128_text(final_step == nullptr
                    ? static_cast<Binary128>(0.0) : final_step->slope_bound)
            << "\",\"direction_error_bound\":\""
            << al_binary128_text(final_step == nullptr
                    ? static_cast<Binary128>(0.0)
                    : final_step->direction_error_bound)
            << "\",\"root\":\""
            << (final_step == nullptr ? "" : final_step->root) << "\"}"
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
            << "\",\"problem_sha256\":\"" << problems[index].problem_root
            << "\",\"root\":\"" << audit.root << "\"}";
    }
    report << "]"
        << ",\"solver_executions\":" << solver_executions
        << ",\"terminal_selections\":" << terminal_selections
        << ",\"exhausted_recoveries\":" << recoveries
        << ",\"maximum_recoveries_per_case\":1"
        << ",\"second_exhaustions\":" << second_exhaustions
        << ",\"total_event_replacements\":" << total_event_replacements
        << ",\"total_line_evaluations\":" << total_line_evaluations
        << ",\"accepted_iteration_cap\":32,\"tolerance_changes\":0"
        << ",\"ordinary_armijo_changes\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {harness_exact, report.str()};
}
