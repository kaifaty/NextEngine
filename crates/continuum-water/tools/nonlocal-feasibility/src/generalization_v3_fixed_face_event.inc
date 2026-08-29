namespace {

constexpr const char* AL_R20_V3_BREAKPOINT_GEOMETRY_SHA256 =
    "4ca89c1ffa825d8a09422f460c40886b7e5e60ac9992602726eb7d0a5dcc1de5";

struct ALR20EventPolynomial {
    bool exact = false;
    bool active_ball = false;
    bool zero_bound = false;
    int bound_code = 0;
    std::size_t scalar = 0U;
    Binary128 a = static_cast<Binary128>(0.0);
    Binary128 b = static_cast<Binary128>(0.0);
    Binary128 c = static_cast<Binary128>(0.0);
    Binary128 low_value = static_cast<Binary128>(0.0);
    Binary128 high_value = static_cast<Binary128>(0.0);
    std::vector<Binary128> roots;
};

std::vector<Binary128> al_r20_r21_real_roots(
    Binary128 a, Binary128 b, Binary128 c) {
    const Binary128 zero = static_cast<Binary128>(0.0);
    const Binary128 two = static_cast<Binary128>(2.0);
    const Binary128 four = static_cast<Binary128>(4.0);
    std::vector<Binary128> roots;
    if (a == zero) {
        if (b != zero) roots.push_back(-c / b);
        return roots;
    }
    const Binary128 discriminant = b * b - four * a * c;
    if (discriminant < zero) return roots;
    const Binary128 square_root = sqrtq(discriminant);
    roots.push_back((-b - square_root) / (two * a));
    const Binary128 second = (-b + square_root) / (two * a);
    if (second != roots.front()) roots.push_back(second);
    std::sort(roots.begin(), roots.end());
    return roots;
}

Binary128 al_r20_r21_polynomial_value(
    const ALR20EventPolynomial& polynomial, Binary128 alpha) {
    return (polynomial.a * alpha + polynomial.b) * alpha
        + polynomial.c;
}

bool al_r20_r21_event_controls() {
    const Binary128 zero = static_cast<Binary128>(0.0);
    const std::vector<Binary128> quadratic = al_r20_r21_real_roots(
        static_cast<Binary128>(1.0), static_cast<Binary128>(-3.0),
        static_cast<Binary128>(2.0));
    const std::vector<Binary128> degenerate = al_r20_r21_real_roots(
        zero, static_cast<Binary128>(2.0), static_cast<Binary128>(-1.0));
    const std::vector<Binary128> repeated = al_r20_r21_real_roots(
        static_cast<Binary128>(1.0), static_cast<Binary128>(-2.0),
        static_cast<Binary128>(1.0));
    const std::vector<Binary128> absent = al_r20_r21_real_roots(
        static_cast<Binary128>(1.0), zero, static_cast<Binary128>(1.0));
    const Binary128 zero_bound_root =
        -static_cast<Binary128>(1.0) / static_cast<Binary128>(-2.0);
    return quadratic.size() == 2U
        && quadratic[0] == static_cast<Binary128>(1.0)
        && quadratic[1] == static_cast<Binary128>(2.0)
        && degenerate.size() == 1U
        && degenerate[0] == static_cast<Binary128>(0.5)
        && repeated.size() == 1U
        && repeated[0] == static_cast<Binary128>(1.0)
        && absent.empty()
        && zero_bound_root == static_cast<Binary128>(0.5);
}

struct ALR20AdmissibleEvent {
    std::size_t scalar = 0U;
    int bound_code = 0;
    Binary128 alpha = static_cast<Binary128>(0.0);
    Binary128 relation_residual = static_cast<Binary128>(0.0);
    ALR20EventPolynomial polynomial;
};

struct ALR20EventBracketAudit {
    bool exact = false;
    bool unique = false;
    bool scalar_matches = false;
    bool endpoint_single_scalar = false;
    std::size_t iteration = 0U;
    std::size_t observed_scalar = 0U;
    std::size_t predicted_scalar = 0U;
    std::size_t enumerated_polynomials = 0U;
    std::size_t algebraic_roots = 0U;
    std::size_t outside_cell_roots = 0U;
    std::size_t face_rejected_roots = 0U;
    std::size_t sign_rejected_roots = 0U;
    std::size_t admissible_roots = 0U;
    Binary128 predicted_alpha = static_cast<Binary128>(0.0);
    Binary128 cell_low = static_cast<Binary128>(0.0);
    Binary128 cell_high = static_cast<Binary128>(0.0);
    std::size_t free_components = 0U;
    std::size_t clamped_components = 0U;
    bool ball_active = false;
    std::vector<ALR20AdmissibleEvent> events;
    std::string root;
};

ALR20EventBracketAudit al_r20_r21_event_audit(
    const ALR20ProblemData& problem,
    const ALR20NewtonDualEvaluation& current,
    const std::vector<Binary128>& lambda,
    const ALR20SemismoothStep& step,
    const ALR20BreakpointBracket& bracket) {
    ALR20EventBracketAudit result;
    result.iteration = step.iteration;
    result.ball_active = current.projector.ball_active;
    result.free_components = current.projector.free_components;
    result.clamped_components = current.projector.lower_components
        + current.projector.upper_components
        + current.projector.fixed_components;
    bool exact = bracket.exact
        && bracket.first_mask_transition_cell
            < AL_R20_BREAKPOINT_SUBDIVISIONS
        && step.line_representative.size() == lambda.size();
    if (!exact) return result;
    const ALR20BreakpointSample& low_sample =
        bracket.samples[bracket.first_mask_transition_cell];
    const ALR20BreakpointSample& high_sample =
        bracket.samples[bracket.first_mask_transition_cell + 1U];
    result.cell_low = low_sample.alpha;
    result.cell_high = high_sample.alpha;
    std::vector<std::size_t> changed;
    for (std::size_t scalar = 0U;
         scalar < current.projector.mask.size(); ++scalar) {
        if (low_sample.dual.projector.mask[scalar]
                != high_sample.dual.projector.mask[scalar])
            changed.push_back(scalar);
    }
    result.endpoint_single_scalar = changed.size() == 1U
        && low_sample.dual.projector.ball_active
            == high_sample.dual.projector.ball_active;
    if (result.endpoint_single_scalar) result.observed_scalar = changed.front();

    const std::vector<Binary128Vec3> target =
        al_r20_q_promote(problem.target);
    const std::vector<Binary128Vec3> at =
        al_r20_q_transpose(problem.op, lambda, target.size());
    std::vector<Binary128> line_direction(lambda.size());
    for (std::size_t row = 0U; row < lambda.size(); ++row)
        line_direction[row] = step.line_representative[row] - lambda[row];
    const std::vector<Binary128Vec3> at_direction =
        al_r20_q_transpose(problem.op, line_direction, target.size());
    std::vector<Binary128Vec3> z0(target.size());
    std::vector<Binary128Vec3> dz(target.size());
    for (std::size_t particle = 0U; particle < target.size(); ++particle) {
        z0[particle] = target[particle] - at[particle];
        dz[particle] = al_r20_q_scale(
            static_cast<Binary128>(-1.0), at_direction[particle]);
    }
    exact = exact && al_r20_admm_finite(line_direction);

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
        s1_accumulator.add(static_cast<Binary128>(2.0)
            * initial * direction);
        s2_accumulator.add(direction * direction);
    }
    const Binary128 s0 = s0_accumulator.value();
    const Binary128 s1 = s1_accumulator.value();
    const Binary128 s2 = s2_accumulator.value();
    const Binary128 remaining_radius_squared =
        current.projector.remaining_radius_squared;
    const std::vector<Binary128Vec3> lower =
        al_r20_projector_promote_bounds(problem.lower);
    const std::vector<Binary128Vec3> upper =
        al_r20_projector_promote_bounds(problem.upper);
    std::vector<ALR20AdmissibleEvent> admissible;
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
            const Binary128 bound = bound_entry.second;
            polynomial.active_ball = current.projector.ball_active;
            polynomial.zero_bound = bound
                == static_cast<Binary128>(0.0);
            if (!current.projector.ball_active || polynomial.zero_bound) {
                polynomial.a = static_cast<Binary128>(0.0);
                polynomial.b = direction;
                polynomial.c = initial - bound;
            } else {
                const Binary128 bound_squared = bound * bound;
                polynomial.a = remaining_radius_squared
                        * direction * direction
                    - bound_squared * s2;
                polynomial.b = static_cast<Binary128>(2.0)
                        * remaining_radius_squared * initial * direction
                    - bound_squared * s1;
                polynomial.c = remaining_radius_squared * initial * initial
                    - bound_squared * s0;
            }
            polynomial.low_value = al_r20_r21_polynomial_value(
                polynomial, result.cell_low);
            polynomial.high_value = al_r20_r21_polynomial_value(
                polynomial, result.cell_high);
            polynomial.roots = al_r20_r21_real_roots(
                polynomial.a, polynomial.b, polynomial.c);
            polynomial.exact = finiteq(polynomial.a) != 0
                && finiteq(polynomial.b) != 0
                && finiteq(polynomial.c) != 0
                && finiteq(polynomial.low_value) != 0
                && finiteq(polynomial.high_value) != 0;
            exact = exact && polynomial.exact;
            ++result.enumerated_polynomials;
            result.algebraic_roots += polynomial.roots.size();
            for (Binary128 root : polynomial.roots) {
                if (!(root >= result.cell_low && root <= result.cell_high)
                    || finiteq(root) == 0) {
                    ++result.outside_cell_roots;
                    continue;
                }
                const int low_mask =
                    low_sample.dual.projector.mask[scalar];
                const int high_mask =
                    high_sample.dual.projector.mask[scalar];
                const bool face_matches = low_mask != high_mask
                    && (low_mask == polynomial.bound_code
                        || high_mask == polynomial.bound_code);
                if (!face_matches) {
                    ++result.face_rejected_roots;
                    continue;
                }
                const Binary128 root_value = initial + root * direction;
                const bool sign_matches = polynomial.zero_bound
                    ? true : root_value * bound
                        > static_cast<Binary128>(0.0);
                const bool endpoint_brackets =
                    polynomial.low_value == static_cast<Binary128>(0.0)
                    || polynomial.high_value == static_cast<Binary128>(0.0)
                    || (polynomial.low_value
                            < static_cast<Binary128>(0.0))
                        != (polynomial.high_value
                            < static_cast<Binary128>(0.0));
                if (!sign_matches || !endpoint_brackets) {
                    ++result.sign_rejected_roots;
                    continue;
                }
                Binary128 relation_residual = root_value - bound;
                if (current.projector.ball_active
                    && !polynomial.zero_bound) {
                    const Binary128 norm_squared =
                        (s2 * root + s1) * root + s0;
                    const Binary128 divisor = sqrtq(
                        norm_squared / remaining_radius_squared);
                    relation_residual = root_value - divisor * bound;
                }
                ALR20AdmissibleEvent event;
                event.scalar = scalar;
                event.bound_code = polynomial.bound_code;
                event.alpha = root;
                event.relation_residual = relation_residual;
                event.polynomial = polynomial;
                admissible.push_back(std::move(event));
            }
        }
    }
    result.events = std::move(admissible);
    result.admissible_roots = result.events.size();
    result.unique = result.events.size() == 1U;
    if (result.unique) {
        result.predicted_scalar = result.events.front().scalar;
        result.predicted_alpha = result.events.front().alpha;
    }
    result.scalar_matches = result.unique && result.endpoint_single_scalar
        && result.predicted_scalar == result.observed_scalar;
    result.exact = exact && result.endpoint_single_scalar
        && result.scalar_matches;
    std::ostringstream material;
    material << result.exact << ':' << result.unique << ':'
        << result.scalar_matches << ':' << result.endpoint_single_scalar << ':'
        << result.iteration << ':' << result.observed_scalar << ':'
        << result.predicted_scalar << ':' << result.enumerated_polynomials << ':'
        << result.algebraic_roots << ':' << result.outside_cell_roots << ':'
        << result.face_rejected_roots << ':' << result.sign_rejected_roots << ':'
        << result.admissible_roots << ':'
        << al_binary128_hex(result.predicted_alpha) << ':'
        << al_binary128_hex(result.cell_low) << ':'
        << al_binary128_hex(result.cell_high) << ':'
        << result.free_components << ':' << result.clamped_components << ':'
        << result.ball_active << '|';
    for (const ALR20AdmissibleEvent& event : result.events)
        material << event.scalar << ':' << event.bound_code << ':'
            << al_binary128_hex(event.alpha) << ':'
            << al_binary128_hex(event.relation_residual) << ':'
            << al_binary128_hex(event.polynomial.a) << ':'
            << al_binary128_hex(event.polynomial.b) << ':'
            << al_binary128_hex(event.polynomial.c) << ';';
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v3_fixed_face_event_controls() {
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
    std::vector<ALR20ManifestCase> cases = al_r20_v3_manifest_cases();
    bool parent_exact = controls_exact
        && cases.size() == AL_R20_V3_PROBLEM_SHA256.size()
        && al_r20_v3_manifest_root(cases) == AL_R20_V3_MANIFEST_SHA256;
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    std::vector<ALR20ProblemData> problems;
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        ALR20ProblemData problem = al_r20_materialize_problem(
            cases[index], &sparse_work, &static_work, &adjacency_work);
        parent_exact = parent_exact && problem.exact
            && index < AL_R20_V3_PROBLEM_SHA256.size()
            && problem.problem_root == AL_R20_V3_PROBLEM_SHA256[index]
            && problem.source_positive == 0U;
        problems.push_back(std::move(problem));
    }
    ALR20SemismoothCase diagnostic;
    std::size_t solver_executions = 0U;
    if (parent_exact) {
        diagnostic = al_r20_semismooth_case(
            problems[AL_R20_V3_SHEAR_INDEX], true, true);
        ++solver_executions;
    }
    bool reproduction_exact = parent_exact
        && diagnostic.root == AL_R20_V3_SHEAR_CANDIDATE_SHA256
        && diagnostic.steps.size() == 32U;

    std::vector<ALR20BreakpointBracket> brackets;
    std::vector<ALR20EventBracketAudit> audits;
    std::vector<Binary128> lambda(
        problems[AL_R20_V3_SHEAR_INDEX].row_scale.size(),
        static_cast<Binary128>(0.0));
    ALR20NewtonDualEvaluation current = al_r20_newton_dual_evaluate(
        problems[AL_R20_V3_SHEAR_INDEX], lambda);
    bool breakpoint_exact = reproduction_exact && current.exact;
    bool audit_exact = reproduction_exact && current.exact;
    for (const ALR20SemismoothStep& step : diagnostic.steps) {
        if (step.iteration >= AL_R20_V3_LATE_FIRST && step.accepted
            && step.selected.power > 0U && step.selected.mask_changes == 0U) {
            bool all_prior_crossing = true;
            for (const ALR20NewtonTrial& trial : step.trials)
                if (trial.power < step.selected.power)
                    all_prior_crossing = all_prior_crossing
                        && (trial.mask_changes > 0U
                            || trial.ball_activity_changed);
            if (all_prior_crossing) {
                ALR20BreakpointBracket bracket =
                    al_r20_r20_breakpoint_bracket(
                        problems[AL_R20_V3_SHEAR_INDEX], current,
                        lambda, step);
                ALR20EventBracketAudit audit = al_r20_r21_event_audit(
                    problems[AL_R20_V3_SHEAR_INDEX], current,
                    lambda, step, bracket);
                breakpoint_exact = breakpoint_exact && bracket.exact;
                audit_exact = audit_exact && audit.exact;
                brackets.push_back(std::move(bracket));
                audits.push_back(std::move(audit));
            }
        }
        if (step.accepted) {
            lambda = step.next_lambda;
            current = step.selected.dual;
        }
    }
    breakpoint_exact = breakpoint_exact
        && brackets.size() == 7U;
    audit_exact = audit_exact
        && brackets.size() == 7U && audits.size() == 7U;
    std::ostringstream parent_semantic;
    parent_semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r20-v3-breakpoint-geometry.v1|"
        << AL_R20_V3_LINE_FRONTIER_SHA256
        << ":V3_BREAKPOINT_GEOMETRY_IDENTIFIED:SIMPLE_BREAKPOINT_OFFSET|";
    for (const ALR20BreakpointBracket& bracket : brackets)
        parent_semantic << bracket.root << ';';
    const std::string parent_geometry_sha = sha256_hex(parent_semantic.str());
    const bool r20_exact = breakpoint_exact
        && parent_geometry_sha == AL_R20_V3_BREAKPOINT_GEOMETRY_SHA256;

    std::size_t unique_events = 0U;
    std::size_t ambiguous_events = 0U;
    std::size_t unbracketed_events = 0U;
    std::size_t wrong_scalar_events = 0U;
    for (const ALR20EventBracketAudit& audit : audits) {
        if (audit.unique && audit.scalar_matches) ++unique_events;
        else if (audit.admissible_roots > 1U) ++ambiguous_events;
        else if (audit.admissible_roots == 0U) ++unbracketed_events;
        else ++wrong_scalar_events;
    }
    std::string classification = "EVENT_PREDICTOR_UNRESOLVED";
    if (ambiguous_events > 0U)
        classification = "EVENT_PREDICTOR_AMBIGUOUS";
    else if (wrong_scalar_events > 0U)
        classification = "EVENT_PREDICTOR_WRONG_SCALAR";
    else if (unbracketed_events > 0U)
        classification = "EVENT_PREDICTOR_UNBRACKETED";
    else if (unique_events == 7U)
        classification = "FIXED_FACE_EVENT_PREDICTOR_CANDIDATE";

    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const std::string route = !parent_exact
        ? "V3_FIXED_FACE_EVENT_PARENT_REJECTED"
        : (!r20_exact ? "V3_FIXED_FACE_EVENT_R20_REJECTED"
            : (!audit_exact ? "V3_FIXED_FACE_EVENT_AUDIT_REJECTED"
                            : "V3_FIXED_FACE_EVENT_IDENTIFIED"));
    const bool exact = parent_exact && r20_exact
        && audit_exact && lifecycle_exact;
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r21-v3-fixed-face-event.v1|"
        << AL_R20_V3_BREAKPOINT_GEOMETRY_SHA256 << ':' << route << ':'
        << classification << '|';
    for (const ALR20EventBracketAudit& audit : audits)
        semantic << audit.root << ';';
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r21_v3_fixed_face_event.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"classification\":\"" << classification << "\""
        << ",\"parent_breakpoint_geometry_sha256\":\""
        << AL_R20_V3_BREAKPOINT_GEOMETRY_SHA256 << "\""
        << ",\"reproduced_breakpoint_geometry_sha256\":\""
        << parent_geometry_sha << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
        << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
        << ",\"r20_exact\":" << (r20_exact ? "true" : "false")
        << ",\"audit_exact\":" << (audit_exact ? "true" : "false")
        << ",\"lifecycle_exact\":" << (lifecycle_exact ? "true" : "false")
        << ",\"unique_events\":" << unique_events
        << ",\"ambiguous_events\":" << ambiguous_events
        << ",\"unbracketed_events\":" << unbracketed_events
        << ",\"wrong_scalar_events\":" << wrong_scalar_events
        << ",\"audits\":[";
    for (std::size_t index = 0U; index < audits.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20EventBracketAudit& audit = audits[index];
        report << "{\"iteration\":" << audit.iteration
            << ",\"unique\":" << (audit.unique ? "true" : "false")
            << ",\"scalar_matches\":"
            << (audit.scalar_matches ? "true" : "false")
            << ",\"endpoint_single_scalar\":"
            << (audit.endpoint_single_scalar ? "true" : "false")
            << ",\"observed_scalar\":" << audit.observed_scalar
            << ",\"predicted_scalar\":" << audit.predicted_scalar
            << ",\"predicted_alpha\":\""
            << al_binary128_text(audit.predicted_alpha)
            << "\",\"cell_low\":\"" << al_binary128_text(audit.cell_low)
            << "\",\"cell_high\":\"" << al_binary128_text(audit.cell_high)
            << "\",\"free_components\":" << audit.free_components
            << ",\"clamped_components\":" << audit.clamped_components
            << ",\"ball_active\":"
            << (audit.ball_active ? "true" : "false")
            << ",\"enumerated_polynomials\":"
            << audit.enumerated_polynomials
            << ",\"algebraic_roots\":" << audit.algebraic_roots
            << ",\"outside_cell_roots\":" << audit.outside_cell_roots
            << ",\"face_rejected_roots\":" << audit.face_rejected_roots
            << ",\"sign_rejected_roots\":" << audit.sign_rejected_roots
            << ",\"admissible_roots\":" << audit.admissible_roots
            << ",\"events\":[";
        for (std::size_t event_index = 0U;
             event_index < audit.events.size(); ++event_index) {
            if (event_index != 0U) report << ',';
            const ALR20AdmissibleEvent& event = audit.events[event_index];
            report << "{\"scalar\":" << event.scalar
                << ",\"bound_code\":" << event.bound_code
                << ",\"alpha\":\"" << al_binary128_text(event.alpha)
                << "\",\"relation_residual\":\""
                << al_binary128_text(event.relation_residual)
                << "\",\"a\":\""
                << al_binary128_text(event.polynomial.a)
                << "\",\"b\":\""
                << al_binary128_text(event.polynomial.b)
                << "\",\"c\":\""
                << al_binary128_text(event.polynomial.c)
                << "\",\"low_value\":\""
                << al_binary128_text(event.polynomial.low_value)
                << "\",\"high_value\":\""
                << al_binary128_text(event.polynomial.high_value)
                << "\",\"zero_bound\":"
                << (event.polynomial.zero_bound ? "true" : "false")
                << "}";
        }
        report << "],\"root\":\"" << audit.root << "\"}";
    }
    report << "]"
        << ",\"solver_executions\":" << solver_executions
        << ",\"generated_solver_trials\":0,\"state_updates\":0"
        << ",\"cap_changes\":0,\"timing_admitted\":false"
        << ",\"solver_authority\":false,\"runtime_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
