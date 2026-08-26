namespace {

struct ALR20R28EventRoot {
    bool exact = false;
    bool admissible = false;
    bool ball_event = false;
    bool zero_bound = false;
    std::size_t scalar = 0U;
    int bound_code = 0;
    Binary128 alpha = static_cast<Binary128>(0.0);
    Binary128 derivative = static_cast<Binary128>(0.0);
    Binary128 relation_residual = static_cast<Binary128>(0.0);
    Binary128 a = static_cast<Binary128>(0.0);
    Binary128 b = static_cast<Binary128>(0.0);
    Binary128 c = static_cast<Binary128>(0.0);
    std::string root;
};

struct ALR20R28ShadowTrial {
    bool exact = false;
    bool cone_exact = false;
    bool face_exact = false;
    bool accepted = false;
    bool old_side = false;
    std::size_t changed_scalar = 0U;
    std::size_t mask_changes = 0U;
    bool ball_activity_changed = false;
    Binary128 alpha = static_cast<Binary128>(0.0);
    Binary128 margin = static_cast<Binary128>(0.0);
    Binary128 dual_increase_lower = static_cast<Binary128>(0.0);
    ALR20NewtonDualEvaluation dual;
    ALR20OracleMetrics metrics;
    std::string root;
};

bool al_r20_r28_current_face_valid(
    const ALR20NewtonDualEvaluation& current,
    const std::vector<Binary128Vec3>& lower,
    const std::vector<Binary128Vec3>& upper,
    const std::vector<Binary128Vec3>& z0,
    const std::vector<Binary128Vec3>& dz,
    Binary128 s0, Binary128 s1, Binary128 s2,
    Binary128 alpha, std::size_t skipped_scalar,
    bool ball_boundary) {
    const Binary128 zero = static_cast<Binary128>(0.0);
    const Binary128 one = static_cast<Binary128>(1.0);
    const Binary128 norm_squared = (s2 * alpha + s1) * alpha + s0;
    Binary128 divisor = one;
    if (ball_boundary) {
        if (!(norm_squared >= zero)) return false;
    } else if (current.projector.ball_active) {
        if (!(current.projector.remaining_radius_squared > zero)
            || !(norm_squared >= current.projector.remaining_radius_squared))
            return false;
        divisor = sqrtq(norm_squared
            / current.projector.remaining_radius_squared);
        if (!(divisor >= one) || finiteq(divisor) == 0) return false;
    } else if (!(norm_squared
        <= current.projector.remaining_radius_squared)) {
        return false;
    }
    for (std::size_t scalar = 0U;
         scalar < current.projector.mask.size(); ++scalar) {
        if (scalar == skipped_scalar || current.projector.mask[scalar] == 2)
            continue;
        const std::size_t particle = scalar / 3U;
        const int axis = static_cast<int>(scalar % 3U);
        const Binary128 value = (al_r20_q_component(z0[particle], axis)
            + alpha * al_r20_q_component(dz[particle], axis)) / divisor;
        const Binary128 low = al_r20_q_component(lower[particle], axis);
        const Binary128 high = al_r20_q_component(upper[particle], axis);
        if (current.projector.mask[scalar] == -1 && !(value <= low))
            return false;
        if (current.projector.mask[scalar] == 1 && !(value >= high))
            return false;
        if (current.projector.mask[scalar] == 0
            && (!(value >= low) || !(value <= high)))
            return false;
    }
    return true;
}

ALR20R28ShadowTrial al_r20_r28_shadow_trial(
    const ALR20ProblemData& problem,
    const ALR20NewtonDualEvaluation& current,
    const std::vector<Binary128>& lambda,
    const std::vector<Binary128>& diagonal,
    const ALR20SemismoothStep& step,
    Binary128 alpha, std::size_t predicted_scalar,
    int expected_old_mask, int expected_new_mask,
    bool old_side) {
    ALR20R28ShadowTrial result;
    result.alpha = alpha;
    result.old_side = old_side;
    std::vector<Binary128> trial_lambda(lambda.size());
    result.cone_exact = alpha > static_cast<Binary128>(0.0);
    for (std::size_t row = 0U; row < lambda.size(); ++row) {
        trial_lambda[row] =
            (static_cast<Binary128>(1.0) - alpha) * lambda[row]
            + alpha * step.line_representative[row];
        result.cone_exact = result.cone_exact
            && trial_lambda[row] >= static_cast<Binary128>(0.0);
    }
    result.dual = al_r20_newton_dual_evaluate(problem, trial_lambda);
    result.ball_activity_changed = result.dual.projector.ball_active
        != current.projector.ball_active;
    bool predicted_changed = false;
    if (current.projector.mask.size() == result.dual.projector.mask.size()) {
        for (std::size_t scalar = 0U;
             scalar < current.projector.mask.size(); ++scalar)
            if (current.projector.mask[scalar]
                    != result.dual.projector.mask[scalar]) {
                ++result.mask_changes;
                result.changed_scalar = scalar;
                predicted_changed = predicted_changed
                    || scalar == predicted_scalar;
            }
    }
    const bool predicted_in_range = predicted_scalar
            < current.projector.mask.size()
        && predicted_scalar < result.dual.projector.mask.size();
    result.face_exact = predicted_in_range
        && !result.ball_activity_changed
        && current.projector.mask[predicted_scalar] == expected_old_mask
        && (old_side
            ? (result.mask_changes == 0U
                && result.dual.projector.mask[predicted_scalar]
                    == expected_old_mask)
            : (result.mask_changes == 1U && predicted_changed
                && result.dual.projector.mask[predicted_scalar]
                    == expected_new_mask));
    ALR20NewtonTrial margin_trial;
    margin_trial.alpha = alpha;
    margin_trial.dual = result.dual;
    result.margin = al_r20_r19_armijo_margin(current, step, margin_trial);
    result.accepted = result.cone_exact && result.dual.exact
        && al_r20_newton_armijo(
            current.value, current.bound,
            result.dual.value, result.dual.bound,
            alpha, step.slope, step.slope_bound);
    result.dual_increase_lower =
        result.dual.value - result.dual.bound
        - current.value - current.bound;
    result.metrics = al_r20_q_metrics(problem,
        result.dual.projector.value, trial_lambda,
        diagonal, step.iteration);
    result.exact = result.cone_exact && result.dual.exact
        && result.metrics.finite && finiteq(result.margin) != 0
        && finiteq(result.dual_increase_lower) != 0;
    std::ostringstream material;
    material << result.exact << ':' << result.cone_exact << ':'
        << result.face_exact << ':' << result.accepted << ':'
        << result.old_side << ':' << result.changed_scalar << ':'
        << result.mask_changes << ':' << result.ball_activity_changed << ':'
        << al_binary128_hex(result.alpha) << ':'
        << al_binary128_hex(result.margin) << ':'
        << al_binary128_hex(result.dual_increase_lower) << ':'
        << result.dual.root << ':' << result.metrics.root;
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v3_sub_envelope_event_controls() {
#if defined(__linux__) && defined(__x86_64__) && defined(__GNUC__) \
    && defined(__SIZEOF_FLOAT128__)
    constexpr bool frozen_platform = true;
#else
    constexpr bool frozen_platform = false;
#endif
    const Binary128 zero = static_cast<Binary128>(0.0);
    const Binary128 envelope = ldexpq(static_cast<Binary128>(1.0), -20);
    const bool profile_exact = frozen_platform
        && sizeof(Binary128) == 16U && FLT_RADIX == 2
        && FLT128_MANT_DIG == 113;
    const bool controls_exact = profile_exact
        && al_r20_verified_inverse_controls()
        && al_r20_semismooth_controls()
        && al_r20_r21_event_controls();
    std::vector<ALR20ManifestCase> cases = al_r20_v3_manifest_cases();
    const bool manifest_exact = cases.size() == AL_R20_V3_PROBLEM_SHA256.size()
        && AL_R20_V3_SHEAR_INDEX < cases.size()
        && al_r20_v3_manifest_root(cases) == AL_R20_V3_MANIFEST_SHA256;
    bool parent_exact = controls_exact && manifest_exact;
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    ALR20ProblemData problem;
    if (parent_exact) {
        problem = al_r20_materialize_problem(
            cases[AL_R20_V3_SHEAR_INDEX], &sparse_work,
            &static_work, &adjacency_work);
        parent_exact = problem.exact && problem.source_positive == 0U
            && problem.problem_root
                == AL_R20_V3_PROBLEM_SHA256[AL_R20_V3_SHEAR_INDEX];
    }

    ALR20SemismoothCase diagnostic;
    std::size_t solver_executions = 0U;
    if (parent_exact) {
        diagnostic = al_r20_semismooth_case(problem, true, true, true);
        ++solver_executions;
    }
    bool reproduction_exact = parent_exact
        && diagnostic.harness_exact
        && diagnostic.root == AL_R20_V3_CERTIFIED_EVENT_SHEAR_SHA256
        && diagnostic.globalization_rejected
        && diagnostic.accepted_iterations == 19U
        && diagnostic.steps.size() == 20U
        && diagnostic.event_replacements == 3U;
    ALR20SemismoothStep final;
    ALR20NewtonDualEvaluation current;
    std::vector<Binary128> lambda;
    std::vector<Binary128> diagonal;
    if (reproduction_exact) {
        final = diagnostic.steps.back();
        lambda = diagnostic.final_lambda;
        diagonal = al_r20_q_diagonal(problem.op);
        current = al_r20_newton_dual_evaluate(problem, lambda);
        reproduction_exact = current.exact && !final.accepted
            && final.direction_certified && final.solve.kkt_certified
            && final.trials.size() == AL_R20_NEWTON_LINE_TRIALS
            && final.trials.back().power == 20U
            && final.trials.back().alpha == envelope
            && final.trials.back().mask_changes == 1U
            && !final.trials.back().ball_activity_changed;
    }

    std::size_t observed_scalar = 0U;
    int observed_old_mask = 0;
    int observed_new_mask = 0;
    bool observed_endpoint_exact = reproduction_exact;
    if (reproduction_exact) {
        const ALR20NewtonTrial& endpoint = final.trials.back();
        std::size_t observed_changes = 0U;
        for (std::size_t scalar = 0U;
             scalar < current.projector.mask.size(); ++scalar)
            if (current.projector.mask[scalar]
                    != endpoint.dual.projector.mask[scalar]) {
                ++observed_changes;
                observed_scalar = scalar;
                observed_old_mask = current.projector.mask[scalar];
                observed_new_mask = endpoint.dual.projector.mask[scalar];
            }
        observed_endpoint_exact = observed_endpoint_exact
            && observed_changes == 1U;
    }

    const std::vector<Binary128Vec3> target =
        reproduction_exact ? al_r20_q_promote(problem.target)
                           : std::vector<Binary128Vec3>{};
    const std::vector<Binary128Vec3> lower = reproduction_exact
        ? al_r20_projector_promote_bounds(problem.lower)
        : std::vector<Binary128Vec3>{};
    const std::vector<Binary128Vec3> upper = reproduction_exact
        ? al_r20_projector_promote_bounds(problem.upper)
        : std::vector<Binary128Vec3>{};
    const std::vector<Binary128Vec3> at = reproduction_exact
        ? al_r20_q_transpose(problem.op, lambda, target.size())
        : std::vector<Binary128Vec3>{};
    std::vector<Binary128> line_direction(lambda.size());
    for (std::size_t row = 0U; row < lambda.size(); ++row)
        line_direction[row] = final.line_representative[row] - lambda[row];
    const std::vector<Binary128Vec3> at_direction = reproduction_exact
        ? al_r20_q_transpose(problem.op, line_direction, target.size())
        : std::vector<Binary128Vec3>{};
    std::vector<Binary128Vec3> z0(target.size());
    std::vector<Binary128Vec3> dz(target.size());
    for (std::size_t particle = 0U; particle < target.size(); ++particle) {
        z0[particle] = al_r20_q_add(target[particle],
            al_r20_q_scale(static_cast<Binary128>(-1.0), at[particle]));
        dz[particle] = al_r20_q_scale(
            static_cast<Binary128>(-1.0), at_direction[particle]);
    }
    bool enumeration_exact = reproduction_exact
        && al_r20_admm_finite(line_direction);
    Binary128Accumulator s0_accumulator;
    Binary128Accumulator s1_accumulator;
    Binary128Accumulator s2_accumulator;
    if (reproduction_exact)
        for (std::size_t scalar = 0U;
             scalar < current.projector.mask.size(); ++scalar) {
            if (current.projector.mask[scalar] != 0) continue;
            const std::size_t particle = scalar / 3U;
            const int axis = static_cast<int>(scalar % 3U);
            const Binary128 initial = al_r20_q_component(z0[particle], axis);
            const Binary128 direction = al_r20_q_component(dz[particle], axis);
            s0_accumulator.add(initial * initial);
            s1_accumulator.add(static_cast<Binary128>(2.0)
                * initial * direction);
            s2_accumulator.add(direction * direction);
        }
    const Binary128 s0 = s0_accumulator.value();
    const Binary128 s1 = s1_accumulator.value();
    const Binary128 s2 = s2_accumulator.value();

    std::size_t enumerated_polynomials = 0U;
    std::size_t algebraic_roots = 0U;
    std::size_t bounded_roots = 0U;
    std::vector<ALR20R28EventRoot> roots;
    if (reproduction_exact) {
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
                if (!current.projector.ball_active
                    || polynomial.zero_bound) {
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
                enumeration_exact = enumeration_exact && polynomial_exact;
                ++enumerated_polynomials;
                const std::vector<Binary128> polynomial_roots =
                    al_r20_r21_real_roots(
                        polynomial.a, polynomial.b, polynomial.c);
                algebraic_roots += polynomial_roots.size();
                for (Binary128 alpha : polynomial_roots) {
                    if (!(alpha > zero && alpha <= envelope)
                        || finiteq(alpha) == 0)
                        continue;
                    ++bounded_roots;
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
                        || current.projector.mask[scalar]
                            == bound_entry.first;
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

        ++enumerated_polynomials;
        const Binary128 ball_a = s2;
        const Binary128 ball_b = s1;
        const Binary128 ball_c = s0
            - current.projector.remaining_radius_squared;
        const bool ball_polynomial_exact = finiteq(ball_a) != 0
            && finiteq(ball_b) != 0 && finiteq(ball_c) != 0;
        enumeration_exact = enumeration_exact && ball_polynomial_exact;
        const std::vector<Binary128> ball_roots =
            al_r20_r21_real_roots(ball_a, ball_b, ball_c);
        algebraic_roots += ball_roots.size();
        for (Binary128 alpha : ball_roots) {
            if (!(alpha > zero && alpha <= envelope)
                || finiteq(alpha) == 0)
                continue;
            ++bounded_roots;
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
                current, lower, upper, z0, dz,
                s0, s1, s2, alpha,
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
    }
    std::sort(roots.begin(), roots.end(),
        [](const ALR20R28EventRoot& lhs, const ALR20R28EventRoot& rhs) {
            if (lhs.alpha != rhs.alpha) return lhs.alpha < rhs.alpha;
            if (lhs.ball_event != rhs.ball_event)
                return lhs.ball_event < rhs.ball_event;
            if (lhs.scalar != rhs.scalar) return lhs.scalar < rhs.scalar;
            return lhs.bound_code < rhs.bound_code;
        });
    std::vector<std::size_t> admissible_indices;
    for (std::size_t index = 0U; index < roots.size(); ++index)
        if (roots[index].admissible) admissible_indices.push_back(index);
    std::size_t nearest_multiplicity = 0U;
    Binary128 nearest_alpha = zero;
    Binary128 second_alpha = HUGE_VALQ;
    if (!admissible_indices.empty()) {
        nearest_alpha = roots[admissible_indices.front()].alpha;
        for (std::size_t index : admissible_indices) {
            if (roots[index].alpha == nearest_alpha)
                ++nearest_multiplicity;
            else {
                second_alpha = roots[index].alpha;
                break;
            }
        }
    }

    bool nearest_unique_linear = admissible_indices.size() > 0U
        && nearest_multiplicity == 1U;
    const ALR20R28EventRoot* nearest = nearest_unique_linear
        ? &roots[admissible_indices.front()] : nullptr;
    nearest_unique_linear = nearest_unique_linear && nearest != nullptr
        && !nearest->ball_event && nearest->zero_bound
        && nearest->derivative != zero
        && nearest->scalar == observed_scalar;

    std::size_t contributing_entries = 0U;
    Binary128 forward_scale = zero;
    Binary128 forward_bound = zero;
    Binary128 alpha_old = zero;
    Binary128 alpha_new = zero;
    bool bracket_exact = false;
    ALR20R28ShadowTrial old_trial;
    ALR20R28ShadowTrial new_trial;
    std::size_t shadow_evaluations = 0U;
    if (nearest_unique_linear) {
        const std::size_t particle = nearest->scalar / 3U;
        const int axis = static_cast<int>(nearest->scalar % 3U);
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
                    + fabsq(final.line_representative[row])));
                ++contributing_entries;
            }
        forward_scale = scale.value();
        forward_bound = al_r20_q_gamma(
            32U * contributing_entries + 256U)
            * fmaxq(forward_scale,
                ldexpq(static_cast<Binary128>(1.0), -16382));
        const Binary128 half_width =
            forward_bound / fabsq(nearest->derivative);
        const Binary128 old_edge = nearest->alpha - half_width;
        const Binary128 new_edge = nearest->alpha + half_width;
        if (old_edge > zero && finiteq(old_edge) != 0
            && finiteq(new_edge) != 0) {
            alpha_old = nextafterq(old_edge, zero);
            alpha_new = nextafterq(new_edge, HUGE_VALQ);
        }
        bracket_exact = contributing_entries > 0U
            && forward_bound > zero
            && alpha_old > zero && alpha_old < nearest->alpha
            && alpha_new > nearest->alpha && alpha_new <= envelope
            && (finiteq(second_alpha) == 0 || alpha_new < second_alpha);
        if (bracket_exact) {
            old_trial = al_r20_r28_shadow_trial(
                problem, current, lambda, diagonal, final,
                alpha_old, nearest->scalar,
                observed_old_mask, observed_new_mask, true);
            new_trial = al_r20_r28_shadow_trial(
                problem, current, lambda, diagonal, final,
                alpha_new, nearest->scalar,
                observed_old_mask, observed_new_mask, false);
            shadow_evaluations = 2U;
            bracket_exact = old_trial.exact && new_trial.exact
                && old_trial.face_exact && new_trial.face_exact;
        }
    }

    const bool precision_boundary = bracket_exact
        && (old_trial.margin == zero || new_trial.margin == zero
            || finiteq(old_trial.margin) == 0
            || finiteq(new_trial.margin) == 0);
    std::string classification = "SUB_ENVELOPE_EVENT_UNRESOLVED";
    if (admissible_indices.empty())
        classification = "SUB_ENVELOPE_EVENT_UNBRACKETED";
    else if (!nearest_unique_linear)
        classification = "SUB_ENVELOPE_EVENT_COUPLED_OR_AMBIGUOUS";
    else if (!bracket_exact)
        classification = "SUB_ENVELOPE_SIDE_UNCERTIFIED";
    else if (precision_boundary)
        classification = "SUB_ENVELOPE_PRECISION_BOUNDARY";
    else if (old_trial.accepted && new_trial.accepted)
        classification = "SUB_ENVELOPE_BIDIRECTIONAL_ACCEPTANCE";
    else if (old_trial.accepted)
        classification = "SUB_ENVELOPE_OLD_SIDE_ONLY";
    else if (new_trial.accepted)
        classification = "SUB_ENVELOPE_NEW_SIDE_ONLY";
    else
        classification = "SUB_ENVELOPE_BOTH_SIDES_REJECTED";

    if (parent_exact)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = !parent_exact
        || (sparse_work.workspace_releases == 1U
            && sparse_work.live_workspaces == 0U
            && !sparse_work.lifecycle_underflow);
    const bool audit_exact = parent_exact && reproduction_exact
        && observed_endpoint_exact && enumeration_exact && lifecycle_exact;
    const std::string route = !parent_exact
        ? "V3_SUB_ENVELOPE_EVENT_PARENT_REJECTED"
        : (!reproduction_exact || !observed_endpoint_exact
            ? "V3_SUB_ENVELOPE_EVENT_REPRODUCTION_REJECTED"
            : (!enumeration_exact
                ? "V3_SUB_ENVELOPE_EVENT_ENUMERATION_REJECTED"
                : (classification == "SUB_ENVELOPE_EVENT_UNRESOLVED"
                    ? "V3_SUB_ENVELOPE_EVENT_UNRESOLVED"
                    : "V3_SUB_ENVELOPE_EVENT_IDENTIFIED")));

    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r28-sub-envelope-event.v1|"
        << AL_R20_V3_CERTIFIED_EVENT_TRAJECTORY_SHA256 << ':'
        << AL_R20_V3_CERTIFIED_EVENT_SHEAR_SHA256 << ':' << route << ':'
        << classification << ':' << observed_scalar << ':'
        << observed_old_mask << ':' << observed_new_mask << ':'
        << enumerated_polynomials << ':' << algebraic_roots << ':'
        << bounded_roots << ':' << admissible_indices.size() << ':'
        << nearest_multiplicity << ':' << al_binary128_hex(nearest_alpha)
        << ':' << al_binary128_hex(second_alpha) << '|';
    for (const ALR20R28EventRoot& event : roots)
        semantic << event.root << ';';
    semantic << '|' << contributing_entries << ':'
        << al_binary128_hex(forward_scale) << ':'
        << al_binary128_hex(forward_bound) << ':'
        << al_binary128_hex(alpha_old) << ':'
        << al_binary128_hex(alpha_new) << ':' << bracket_exact << ':'
        << old_trial.root << ':' << new_trial.root;
    const std::string result_sha = sha256_hex(semantic.str());

    auto append_trial = [](std::ostringstream& report,
        const ALR20R28ShadowTrial& trial) {
        report << "{\"exact\":" << (trial.exact ? "true" : "false")
            << ",\"face_exact\":"
            << (trial.face_exact ? "true" : "false")
            << ",\"accepted\":" << (trial.accepted ? "true" : "false")
            << ",\"old_side\":" << (trial.old_side ? "true" : "false")
            << ",\"alpha\":\"" << al_binary128_text(trial.alpha)
            << "\",\"armijo_margin\":\""
            << al_binary128_text(trial.margin)
            << "\",\"dual_increase_lower\":\""
            << al_binary128_text(trial.dual_increase_lower)
            << "\",\"mask_changes\":" << trial.mask_changes
            << ",\"changed_scalar\":" << trial.changed_scalar
            << ",\"ball_activity_changed\":"
            << (trial.ball_activity_changed ? "true" : "false")
            << ",\"primal\":\""
            << al_binary128_text(trial.metrics.primal)
            << "\",\"dual_mapping\":\""
            << al_binary128_text(trial.metrics.dual_mapping)
            << "\",\"complementarity\":\""
            << al_binary128_text(trial.metrics.complementarity)
            << "\",\"stationarity\":\""
            << al_binary128_text(trial.metrics.stationarity)
            << "\",\"gap_scaled\":\""
            << al_binary128_text(trial.metrics.gap_scaled)
            << "\",\"mask_sha256\":\""
            << al_r20_r20_mask_root(trial.dual.projector)
            << "\",\"root\":\"" << trial.root << "\"}";
    };

    std::ostringstream report;
    report
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r28_sub_envelope_event.v1\""
        << ",\"status\":\"" << (audit_exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"classification\":\"" << classification << "\""
        << ",\"parent_post_event_globalization_sha256\":\""
        << "6ef46e9ec490bfb40e4b7eecbf50898b3efb4d494fa1028fd501c1f14be6e003\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"manifest_exact\":" << (manifest_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"reproduction_exact\":"
        << (reproduction_exact ? "true" : "false")
        << ",\"observed_endpoint_exact\":"
        << (observed_endpoint_exact ? "true" : "false")
        << ",\"enumeration_exact\":"
        << (enumeration_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"case_sha256\":\"" << diagnostic.root << "\""
        << ",\"envelope_alpha\":\"" << al_binary128_text(envelope)
        << "\",\"observed_scalar\":" << observed_scalar
        << ",\"observed_old_mask\":" << observed_old_mask
        << ",\"observed_new_mask\":" << observed_new_mask
        << ",\"current_mask_sha256\":\""
        << al_r20_r20_mask_root(current.projector)
        << "\",\"enumerated_polynomials\":" << enumerated_polynomials
        << ",\"algebraic_roots\":" << algebraic_roots
        << ",\"bounded_roots\":" << bounded_roots
        << ",\"admissible_roots\":" << admissible_indices.size()
        << ",\"nearest_multiplicity\":" << nearest_multiplicity
        << ",\"nearest_alpha\":\"" << al_binary128_text(nearest_alpha)
        << "\",\"second_alpha\":\"" << al_binary128_text(second_alpha)
        << "\",\"roots\":[";
    for (std::size_t index = 0U; index < roots.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20R28EventRoot& event = roots[index];
        report << "{\"exact\":" << (event.exact ? "true" : "false")
            << ",\"admissible\":"
            << (event.admissible ? "true" : "false")
            << ",\"kind\":\""
            << (event.ball_event ? "BALL" : "COMPONENT") << "\""
            << ",\"scalar\":" << event.scalar
            << ",\"bound_code\":" << event.bound_code
            << ",\"zero_bound\":"
            << (event.zero_bound ? "true" : "false")
            << ",\"alpha\":\"" << al_binary128_text(event.alpha)
            << "\",\"derivative\":\""
            << al_binary128_text(event.derivative)
            << "\",\"relation_residual\":\""
            << al_binary128_text(event.relation_residual)
            << "\",\"a\":\"" << al_binary128_text(event.a)
            << "\",\"b\":\"" << al_binary128_text(event.b)
            << "\",\"c\":\"" << al_binary128_text(event.c)
            << "\",\"root\":\"" << event.root << "\"}";
    }
    report << "]"
        << ",\"nearest_unique_zero_bound\":"
        << (nearest_unique_linear ? "true" : "false")
        << ",\"contributing_entries\":" << contributing_entries
        << ",\"forward_scale\":\"" << al_binary128_text(forward_scale)
        << "\",\"forward_bound\":\"" << al_binary128_text(forward_bound)
        << "\",\"alpha_old\":\"" << al_binary128_text(alpha_old)
        << "\",\"alpha_new\":\"" << al_binary128_text(alpha_new)
        << "\",\"bracket_exact\":"
        << (bracket_exact ? "true" : "false")
        << ",\"old_trial\":";
    append_trial(report, old_trial);
    report << ",\"new_trial\":";
    append_trial(report, new_trial);
    report << ",\"solver_executions\":" << solver_executions
        << ",\"shadow_evaluations\":" << shadow_evaluations
        << ",\"new_dyadic_trials\":0,\"new_ulp_trials\":0"
        << ",\"state_updates\":0,\"parameter_changes\":0"
        << ",\"timing_admitted\":false,\"solver_authority\":false"
        << ",\"runtime_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {audit_exact, report.str()};
}
