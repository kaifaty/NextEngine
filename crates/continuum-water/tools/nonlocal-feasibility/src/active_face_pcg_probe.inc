namespace {

constexpr std::size_t AL_R65P_OUTERS = 16U;
constexpr std::size_t AL_R65P_INNERS = 16U;
constexpr std::array<std::size_t, 5U> AL_R65P_CHECKPOINTS{{
    1U, 2U, 4U, 8U, 16U}};
constexpr std::size_t AL_R65P_FISTA_STRUCTURAL_TERMS = 596971680U;
constexpr double AL_R65P_FISTA_MAXIMUM_RAW =
    1.1823169686944491e-9;
constexpr double AL_R65P_FISTA_PROJECTED_GRADIENT =
    9.5382120867736572e-9;
constexpr const char* AL_R65P_V6_RESULT_SHA256 =
    "bd568e0f367d34ef75f5ebeeca085f6cd6c36bb9fd56cb6966965a629ae8f0e2";

bool al_r65p_checkpoint(std::size_t outer) {
    return std::find(AL_R65P_CHECKPOINTS.begin(),
        AL_R65P_CHECKPOINTS.end(), outer) != AL_R65P_CHECKPOINTS.end();
}

struct ALR65PBlock {
    bool passed = false;
    bool curvature_exact = true;
    bool line_exact = true;
    std::vector<double> lambda;
    std::vector<Vec3> q;
    std::vector<double> raw;
    std::size_t iterations = 0U;
    std::size_t face_iterations = 0U;
    std::size_t candidate_trials = 0U;
    std::size_t baseline_projection_calls = 0U;
    std::size_t candidate_projection_calls = 0U;
    std::size_t dual_rejections = 0U;
    std::size_t model_rejections = 0U;
    std::size_t composed_rejections = 0U;
    std::size_t admissible_candidates = 0U;
    std::size_t selected_exponent = 16U;
    std::size_t transpose_calls = 0U;
    std::size_t action_calls = 0U;
    std::size_t curvature_stops = 0U;
    std::size_t predicted_zeros = 0U;
    std::size_t applied_zeros = 0U;
    double initial_residual_norm = 0.0;
    double final_residual_norm = 0.0;
    double descent = 0.0;
    double line_alpha = 0.0;
    double line_slope = 0.0;
    double line_curvature = 0.0;
    double baseline_inertia = 0.0;
    double baseline_model_reduction = 0.0;
    double candidate_inertia = 0.0;
    double candidate_model_reduction = 0.0;
    std::string batch_root;
};

ALR65PBlock al_r65p_block(const ALR64Operator& op,
    const std::vector<std::uint8_t>& owned,
    const std::vector<double>& diagonal,
    const std::vector<double>& initial_lambda,
    const std::vector<Vec3>& initial_q,
    const std::vector<double>& initial_raw,
    const std::vector<Vec3>& correction,
    const std::vector<Vec3>& lower,
    const std::vector<Vec3>& upper,
    const std::vector<Vec3>& trial_origin,
    const std::vector<Vec3>& predicted_position,
    double normal_inertia) {
    ALR65PBlock result;
    const std::size_t count = initial_raw.size();
    if (count == 0U || owned.size() != count || diagonal.size() != count
        || initial_lambda.size() != count
        || initial_q.size() != count || correction.size() != count
        || lower.size() != count || upper.size() != count
        || trial_origin.size() != count || predicted_position.size() != count
        || !std::isfinite(normal_inertia))
        return result;
    result.lambda = initial_lambda;
    result.q = initial_q;
    result.raw = initial_raw;
    std::vector<std::uint8_t> face(count, 0U);
    std::vector<double> residual(count, 0.0);
    std::vector<double> preconditioned(count, 0.0);
    std::vector<double> direction(count, 0.0);
    std::vector<double> step(count, 0.0);
    bool exact = al_r30_finite(result.lambda)
        && al_r48_vec_finite(result.q) && al_r30_finite(result.raw);
    for (std::size_t row = 0U; row < count; ++row) {
        face[row] = owned[row] != 0U && result.lambda[row] > 0.0 ? 1U : 0U;
        exact = exact && std::isfinite(diagonal[row]) && diagonal[row] > 0.0;
        residual[row] = face[row] != 0U ? result.raw[row] : 0.0;
        preconditioned[row] = face[row] != 0U
            ? residual[row] / diagonal[row] : 0.0;
        direction[row] = preconditioned[row];
    }
    long double residual_squared = al_r30_dot(residual, residual);
    long double rz = al_r30_dot(residual, preconditioned);
    result.initial_residual_norm = std::sqrt(
        static_cast<double>(residual_squared));
    exact = exact && std::isfinite(result.initial_residual_norm)
        && std::isfinite(static_cast<double>(rz)) && rz >= 0.0L;

    for (std::size_t iteration = 0U;
         iteration + 1U < AL_R65P_INNERS && exact && rz != 0.0L;
         ++iteration) {
        const std::vector<Vec3> at_direction =
            al_r64_transpose(op, direction, nullptr);
        ++result.transpose_calls;
        if (at_direction.size() != count
            || !al_r48_vec_finite(at_direction)) {
            exact = false;
            break;
        }
        const ALLinearizedJvp image = al_r64_action(op, at_direction);
        ++result.action_calls;
        if (!image.passed || image.value.size() != count) {
            exact = false;
            break;
        }
        long double curvature = 0.0L;
        for (std::size_t row = 0U; row < count; ++row)
            if (face[row] != 0U)
                curvature += static_cast<long double>(direction[row])
                    * image.value[row];
        if (!std::isfinite(static_cast<double>(curvature))) {
            exact = false;
            break;
        }
        if (curvature <= 0.0L) {
            result.curvature_exact = false;
            ++result.curvature_stops;
            break;
        }
        const long double alpha = rz / curvature;
        exact = std::isfinite(static_cast<double>(alpha)) && alpha > 0.0L;
        if (!exact) break;
        for (std::size_t row = 0U; row < count; ++row) {
            if (face[row] == 0U) continue;
            step[row] += static_cast<double>(alpha) * direction[row];
            residual[row] -= static_cast<double>(alpha) * image.value[row];
            preconditioned[row] = residual[row] / diagonal[row];
        }
        ++result.iterations;
        residual_squared = al_r30_dot(residual, residual);
        const long double next_rz = al_r30_dot(residual, preconditioned);
        exact = std::isfinite(static_cast<double>(residual_squared))
            && residual_squared >= 0.0L
            && std::isfinite(static_cast<double>(next_rz))
            && next_rz >= 0.0L;
        if (!exact || next_rz == 0.0L) {
            rz = next_rz;
            break;
        }
        const long double beta = next_rz / rz;
        exact = std::isfinite(static_cast<double>(beta)) && beta >= 0.0L;
        if (!exact) break;
        for (std::size_t row = 0U; row < count; ++row)
            direction[row] = face[row] != 0U
                ? preconditioned[row]
                    + static_cast<double>(beta) * direction[row]
                : 0.0;
        rz = next_rz;
    }

    result.final_residual_norm = std::sqrt(
        static_cast<double>(residual_squared));
    std::vector<std::uint8_t> predicted_mask(count, 0U);
    result.face_iterations = result.iterations;
    for (std::size_t row = 0U; row < count; ++row) {
        if (face[row] == 0U) continue;
        const double candidate = std::max(0.0, result.lambda[row] + step[row]);
        if (candidate == 0.0 && result.lambda[row] > 0.0) {
            predicted_mask[row] = 1U;
            ++result.predicted_zeros;
        }
    }
    std::vector<double> selected_lambda = result.lambda;
    std::vector<Vec3> selected_at(count);
    std::vector<std::uint8_t> applied_mask(count, 0U);
    std::vector<Vec3> baseline_joint_input(count);
    for (std::size_t particle = 0U; particle < count; ++particle)
        baseline_joint_input[particle] =
            correction[particle] + initial_q[particle];
    const ALR48Projection baseline_projection = al_r48_project_ball_box(
        baseline_joint_input, lower, upper, AL_R62_NEXT_RADIUS);
    ++result.baseline_projection_calls;
    exact = exact && baseline_projection.passed
        && baseline_projection.value.size() == count;
    std::vector<Vec3> baseline_endpoint(count);
    for (std::size_t particle = 0U; particle < count && exact; ++particle)
        baseline_endpoint[particle] = trial_origin[particle]
            + SPACING * baseline_projection.value[particle];
    result.baseline_inertia = exact
        ? al_r63_inertia(baseline_endpoint, predicted_position)
        : std::numeric_limits<double>::infinity();
    exact = exact && std::isfinite(result.baseline_inertia);
    result.line_exact = false;
    double best_baseline_reduction = 0.0;
    for (std::size_t exponent = 0U; exponent < 16U && exact; ++exponent) {
        const double alpha = std::ldexp(1.0, -static_cast<int>(exponent));
        std::vector<double> trial_lambda = result.lambda;
        std::vector<double> trial_direction(count, 0.0);
        for (std::size_t row = 0U; row < count; ++row) {
            if (face[row] != 0U)
                trial_lambda[row] = std::max(
                    0.0, result.lambda[row] + alpha * step[row]);
            trial_direction[row] = trial_lambda[row] - result.lambda[row];
        }
        const std::vector<Vec3> at_trial =
            al_r64_transpose(op, trial_direction, nullptr);
        ++result.transpose_calls;
        ++result.candidate_trials;
        exact = at_trial.size() == count && al_r48_vec_finite(at_trial)
            && al_r30_finite(trial_lambda);
        long double slope = 0.0L;
        for (std::size_t row = 0U; row < count && exact; ++row)
            slope -= static_cast<long double>(result.raw[row])
                * trial_direction[row];
        const long double curvature = al_r35_dot_vec3(at_trial, at_trial);
        const long double change = slope + 0.5L * curvature;
        exact = exact && std::isfinite(static_cast<double>(slope))
            && std::isfinite(static_cast<double>(curvature))
            && curvature >= 0.0L
            && std::isfinite(static_cast<double>(change));
        if (!exact) continue;
        if (change >= 0.0L) {
            ++result.dual_rejections;
            continue;
        }
        std::vector<Vec3> candidate_joint_input(count);
        for (std::size_t particle = 0U; particle < count; ++particle)
            candidate_joint_input[particle] = correction[particle]
                + initial_q[particle] - at_trial[particle];
        const ALR48Projection candidate_projection = al_r48_project_ball_box(
            candidate_joint_input, lower, upper, AL_R62_NEXT_RADIUS);
        ++result.candidate_projection_calls;
        exact = candidate_projection.passed
            && candidate_projection.value.size() == count;
        if (!exact) break;
        std::vector<Vec3> candidate_endpoint(count);
        for (std::size_t particle = 0U; particle < count; ++particle)
            candidate_endpoint[particle] = trial_origin[particle]
                + SPACING * candidate_projection.value[particle];
        const double candidate_inertia = al_r63_inertia(
            candidate_endpoint, predicted_position);
        const double candidate_model_reduction =
            normal_inertia - candidate_inertia;
        const double baseline_model_reduction =
            result.baseline_inertia - candidate_inertia;
        exact = std::isfinite(candidate_inertia)
            && std::isfinite(candidate_model_reduction)
            && std::isfinite(baseline_model_reduction);
        if (!exact) break;
        if (candidate_model_reduction <= 0.0) {
            ++result.model_rejections;
            continue;
        }
        if (baseline_model_reduction <= 0.0) {
            ++result.composed_rejections;
            continue;
        }
        ++result.admissible_candidates;
        if (baseline_model_reduction <= best_baseline_reduction) continue;
        result.line_exact = true;
        best_baseline_reduction = baseline_model_reduction;
        result.line_alpha = alpha;
        result.line_slope = static_cast<double>(slope);
        result.line_curvature = static_cast<double>(curvature);
        result.descent = static_cast<double>(change);
        result.candidate_inertia = candidate_inertia;
        result.candidate_model_reduction = candidate_model_reduction;
        result.baseline_model_reduction = baseline_model_reduction;
        result.selected_exponent = exponent;
        selected_lambda = trial_lambda;
        selected_at = at_trial;
    }
    if (result.line_exact) {
        result.lambda = std::move(selected_lambda);
        for (std::size_t row = 0U; row < count; ++row)
            if (result.lambda[row] == 0.0 && initial_lambda[row] > 0.0) {
                applied_mask[row] = 1U;
                ++result.applied_zeros;
            }
        for (std::size_t particle = 0U; particle < count; ++particle)
            result.q[particle] = result.q[particle] - selected_at[particle];
    }
    std::ostringstream batch_material;
    batch_material << result.face_iterations << ':'
        << result.candidate_trials << ':' << result.selected_exponent << ':'
        << result.baseline_projection_calls << ':'
        << result.candidate_projection_calls << ':'
        << result.dual_rejections << ':' << result.model_rejections << ':'
        << result.composed_rejections << ':'
        << result.admissible_candidates << ':'
        << result.predicted_zeros << ':'
        << result.applied_zeros << ':' << al_r29_mask_root(predicted_mask)
        << ':' << al_r29_mask_root(applied_mask) << ':'
        << binary64_bits(result.line_slope) << ':'
        << binary64_bits(result.line_curvature) << ':'
        << binary64_bits(result.line_alpha) << ':'
        << binary64_bits(result.baseline_inertia) << ':'
        << binary64_bits(result.baseline_model_reduction) << ':'
        << binary64_bits(result.candidate_inertia) << ':'
        << binary64_bits(result.candidate_model_reduction);
    result.batch_root = sha256_hex(batch_material.str());
    result.passed = exact && al_r30_finite(result.lambda)
        && al_r48_vec_finite(result.q) && al_r30_finite(result.raw)
        && std::isfinite(result.final_residual_norm)
        && std::isfinite(result.descent);
    return result;
}

struct ALR65PRecord {
    std::size_t outer = 0U;
    std::size_t working = 0U;
    std::size_t additions = 0U;
    std::size_t raw_positive = 0U;
    std::size_t candidate_positive = 0U;
    std::size_t nonzero_lambda = 0U;
    std::size_t face = 0U;
    std::size_t pcg_iterations = 0U;
    std::size_t candidate_trials = 0U;
    std::size_t baseline_projection_calls = 0U;
    std::size_t candidate_projection_calls = 0U;
    std::size_t dual_rejections = 0U;
    std::size_t model_rejections = 0U;
    std::size_t composed_rejections = 0U;
    std::size_t admissible_candidates = 0U;
    std::size_t selected_exponent = 16U;
    std::size_t predicted_zeros = 0U;
    std::size_t applied_zeros = 0U;
    double maximum_raw = 0.0;
    double projected_gradient_norm = 0.0;
    double pcg_initial_residual_norm = 0.0;
    double pcg_final_residual_norm = 0.0;
    double pcg_descent = 0.0;
    double line_alpha = 0.0;
    double line_slope = 0.0;
    double line_curvature = 0.0;
    double baseline_inertia = 0.0;
    double baseline_model_reduction = 0.0;
    double candidate_model_reduction = 0.0;
    double committed_model_reduction = 0.0;
    double model_prediction_gap = 0.0;
    double model_prediction_bound = 0.0;
    double recurrence_gap = 0.0;
    double recurrence_bound = 0.0;
    double dual_objective = 0.0;
    double stationarity_norm = 0.0;
    double complementarity_maximum = 0.0;
    double inertia = 0.0;
    double model_reduction = 0.0;
    bool reprojection_exact = false;
    std::string batch_root;
    std::string root;
};

struct ALR65PProportioningDiagnostic {
    std::size_t outer = 0U;
    double free_squared = 0.0;
    double chopped_squared = 0.0;
    double projected_squared = 0.0;
    double decomposed_squared = 0.0;
    double free_norm = 0.0;
    double chopped_norm = 0.0;
    double projected_norm = 0.0;
    double chopped_free_ratio = 0.0;
    double decomposition_gap = 0.0;
    double decomposition_bound = 0.0;
    bool decomposition_exact = false;
    bool chopped_dominant = false;
    bool stationary = false;
    bool line_accepted = false;
    std::string root;
};

struct ALR65PProportioningControl {
    double free_squared = 0.0;
    double chopped_squared = 0.0;
    double projected_squared = 0.0;
    double wrong_sign_squared = 0.0;
    bool successful_exact = false;
    bool negative_rejected = false;
};

ALR65PProportioningControl al_r65p_proportioning_control() {
    constexpr std::array<double, 3U> lambda{{1.0, 0.0, 0.0}};
    constexpr std::array<double, 3U> raw{{2.0, 3.0, -4.0}};
    long double free_squared = 0.0L;
    long double chopped_squared = 0.0L;
    long double projected_squared = 0.0L;
    long double wrong_sign_squared = 0.0L;
    for (std::size_t row = 0U; row < lambda.size(); ++row) {
        const double gradient = -raw[row];
        const double free = lambda[row] > 0.0 ? gradient : 0.0;
        const double chopped = lambda[row] == 0.0
            ? std::min(gradient, 0.0) : 0.0;
        const double projected = lambda[row] > 0.0
            ? gradient : std::min(gradient, 0.0);
        const double wrong_sign = lambda[row] > 0.0
            ? gradient : std::max(gradient, 0.0);
        free_squared += static_cast<long double>(free) * free;
        chopped_squared += static_cast<long double>(chopped) * chopped;
        projected_squared += static_cast<long double>(projected) * projected;
        wrong_sign_squared += static_cast<long double>(wrong_sign) * wrong_sign;
    }
    ALR65PProportioningControl result;
    result.free_squared = static_cast<double>(free_squared);
    result.chopped_squared = static_cast<double>(chopped_squared);
    result.projected_squared = static_cast<double>(projected_squared);
    result.wrong_sign_squared = static_cast<double>(wrong_sign_squared);
    result.successful_exact = result.free_squared == 4.0
        && result.chopped_squared == 9.0
        && result.projected_squared == 13.0;
    result.negative_rejected = result.wrong_sign_squared == 20.0
        && result.wrong_sign_squared != result.projected_squared;
    return result;
}

std::string al_r65p_diagnostic_route(bool identity, bool exact,
    std::size_t blocked, std::size_t chopped_dominant,
    std::size_t free_dominant, std::size_t stationary) {
    if (!identity || !exact || blocked == 0U || stationary != 0U)
        return "DIAGNOSTIC_REFERENCE_RETAINED";
    if (chopped_dominant == blocked)
        return "PROPORTIONING_PHASE_CANDIDATE";
    if (free_dominant == blocked)
        return "FILTER_MERIT_CONFLICT_CANDIDATE";
    return "PHASE_DEPENDENT_CONTROLLER_REQUIRED";
}

std::string al_r65p_route(bool exact, bool curvature, bool line,
    bool normal_model_rejected, bool composed_model_rejected,
    bool model_agreement, bool recurrence, bool model, bool lower_work,
    bool dominates) {
    if (!exact) return "BEST_MODEL_DYADIC_REFERENCE_RETAINED";
    if (!curvature)
        return "BEST_MODEL_DYADIC_CURVATURE_REJECTED";
    if (!line && composed_model_rejected)
        return "BEST_MODEL_DYADIC_NO_COMPOSED_DESCENT";
    if (!line && normal_model_rejected)
        return "BEST_MODEL_DYADIC_NORMAL_MODEL_REJECTED";
    if (!line) return "BEST_MODEL_DYADIC_NO_COMPOSED_DESCENT";
    if (!model_agreement)
        return "BEST_MODEL_DYADIC_MODEL_AGREEMENT_REJECTED";
    if (!recurrence)
        return "BEST_MODEL_DYADIC_RECURRENCE_REJECTED";
    if (!model)
        return "BEST_MODEL_DYADIC_CHECKPOINT_MODEL_REJECTED";
    if (lower_work && dominates)
        return "BEST_MODEL_DYADIC_ACCELERATION_CANDIDATE";
    return "BEST_MODEL_DYADIC_REFERENCE_RETAINED";
}

} // namespace

SplitBoundaryReport run_al_active_face_pcg_probe_controls() {
    ALR63ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_cached_normal_projection_trqp_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "38298214e352a87cfffb5c5432be90ef822c3f8ab00da5a2f7a3b1cb64565b62";
    ALR62ReplayCapture& r62 = capture.r62;
    ALR61ReplayCapture& r61 = r62.r61;
    ALR58ReplayCapture& r58_capture = r61.r60.r59.r58;
    ALR57ReplayCapture& r57_capture = r58_capture.r57;
    ALR56ReplayCapture& r56_capture = r57_capture.r56;
    ALR55ReplayCapture& r55_capture = r56_capture.r55;
    ALR54ReplayCapture& r54_capture = r55_capture.r54;
    ALR53ReplayCapture& r53_capture = r54_capture.r53;
    ALR51ReplayCapture& r51_capture = r53_capture.r52.r51;
    ALR43ReplayCapture& r43 =
        r51_capture.r50.r49.r48.r47.r46.r45.r44.r43;
    ALR40ReplayCapture& r40 = r43.r42.r40;
    ALR30RangeProjectionCapture& r30 =
        r40.r39.r38.r37.r36.r35.r34.r33.r32.r31.r30;
    const std::string trial_before = al_binary64_vec3_root(r43.projected_trial);
    const std::string target_before = al_binary64_vec3_root(capture.target);

    ALSparseWorkTrace workspace_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    const SmokeFixture fixture = make_b4e2d_dam_fixture();
    const JointStaticSupportIndex index = build_joint_static_support_index(
        tagged_points(fixture.boundary), &static_work);
    const JointStaticSupportBinding binding = bind_joint_static_support_index(
        &index, index.identity_sha256);
    ALNormalizedWorkspace workspace = build_al_normalized_sparse_workspace(
        r43.projected_trial, binding, r30.dual, r30.theta,
        &workspace_work, &static_work, &adjacency_work);
    const bool workspace_exact = parent_exact && workspace.passed && index.passed
        && binding.passed && al_r42_superset_root(workspace.neighborhood)
            == r62.payload.workspace_topology_root;
    ALR64Operator op;
    if (workspace.passed) op = al_r64_build(workspace);
    const ALR64StructureAudit structure = al_r64_structure(workspace, op);
    const bool operator_exact = workspace_exact && structure.slot
        && structure.entry && structure.incidence
        && op.slots.size() == 605144U && op.entries.size() == 535588U;

    std::vector<Vec3> lower(6000U), upper(6000U);
    bool geometry_exact = operator_exact && capture.target.size() == 6000U;
    for (std::size_t i = 0U; i < 6000U; ++i)
        for (int axis = 0; axis < 3; ++axis) {
            const double y = al_r41_component(r43.projected_trial[i], axis);
            const double low_face = al_r41_component(fixture.contact_low, axis);
            const double high_face = al_r41_component(fixture.contact_high, axis);
            double low = (low_face - y) / SPACING;
            double high = (high_face - y) / SPACING;
            if (y < low_face) low = 0.0;
            else if (y > high_face) high = 0.0;
            al_r43_component(lower[i], axis) = low;
            al_r43_component(upper[i], axis) = high;
            geometry_exact = geometry_exact && std::isfinite(low)
                && std::isfinite(high) && low <= 0.0 && high >= 0.0;
        }

    std::vector<double> diagonal(6000U);
    for (std::size_t row = 0U; row < 6000U; ++row) {
        diagonal[row] = al_r65_sparse_diagonal(op, row);
        geometry_exact = geometry_exact && std::isfinite(diagonal[row])
            && diagonal[row] > 0.0;
    }
    std::vector<Vec3> normal_endpoint(6000U);
    for (std::size_t i = 0U; i < 6000U; ++i)
        normal_endpoint[i] = r43.projected_trial[i]
            + SPACING * r62.publication.cached_normal[i];
    const double normal_inertia = al_r63_inertia(
        normal_endpoint, r30.predicted_position);

    std::vector<Vec3> composite = capture.target;
    std::vector<Vec3> correction(6000U);
    std::vector<double> lambda(6000U, 0.0);
    std::vector<std::uint8_t> owned(6000U, 0U);
    ALR61Audit audit = geometry_exact
        ? al_r61_topology_owned_audit(workspace, composite) : ALR61Audit{};
    std::size_t initial_working = 0U;
    if (audit.owner_exact)
        for (std::size_t row = 0U; row < 6000U; ++row)
            if (audit.candidate_upper[row] > 0.0) {
                owned[row] = 1U;
                ++initial_working;
            }

    bool exact = geometry_exact && audit.owner_exact && initial_working > 0U;
    bool curvature_exact = true;
    bool line_exact = true;
    bool normal_model_rejected = false;
    bool composed_model_rejected = false;
    bool model_agreement_exact = true;
    bool recurrence_exact = true;
    bool model_exact = true;
    std::size_t total_additions = initial_working;
    std::size_t identification_sweeps = 0U;
    std::size_t coordinate_visits = 0U;
    std::size_t coordinate_updates = 0U;
    std::size_t coordinate_residual_terms = 0U;
    std::size_t coordinate_update_terms = 0U;
    std::size_t face_pcg_iterations = 0U;
    std::size_t candidate_trials = 0U;
    std::size_t baseline_projection_calls = 0U;
    std::size_t candidate_projection_calls = 0U;
    std::size_t candidate_dual_rejections = 0U;
    std::size_t candidate_model_rejections = 0U;
    std::size_t candidate_composed_rejections = 0U;
    std::size_t admissible_candidates = 0U;
    std::size_t accepted_lines = 0U;
    std::size_t curvature_stops = 0U;
    std::size_t batch_predicted_zeros = 0U;
    std::size_t batch_applied_zeros = 0U;
    std::size_t transpose_calls = 0U;
    std::size_t action_calls = 0U;
    std::size_t projection_calls = 0U;
    std::size_t audit_calls = 1U;
    std::vector<ALR65PRecord> records;
    std::vector<ALR65PProportioningDiagnostic> diagnostics;
    bool diagnostic_decomposition_exact = true;
    std::size_t blocked_diagnostics = 0U;
    std::size_t blocked_chopped_dominant = 0U;
    std::size_t blocked_free_dominant = 0U;
    std::size_t blocked_stationary = 0U;

    for (std::size_t outer = 1U; outer <= AL_R65P_OUTERS && exact; ++outer) {
        std::vector<Vec3> at_lambda = al_r64_transpose(op, lambda, nullptr);
        ++transpose_calls;
        exact = at_lambda.size() == 6000U && al_r48_vec_finite(at_lambda);
        std::vector<Vec3> q(6000U);
        for (std::size_t i = 0U; i < 6000U && exact; ++i)
            q[i] = capture.target[i] - correction[i] - at_lambda[i];
        ++identification_sweeps;
        for (std::size_t row = 0U; row < 6000U && exact; ++row) {
            if (owned[row] == 0U) continue;
            ++coordinate_visits;
            const std::size_t degree = op.entry_offsets[row + 1U]
                - op.entry_offsets[row];
            coordinate_residual_terms += degree;
            const double residual =
                al_r65_sparse_residual(workspace, op, row, q);
            const double next = std::max(
                0.0, lambda[row] + residual / diagonal[row]);
            const double delta = next - lambda[row];
            exact = std::isfinite(residual) && std::isfinite(next)
                && std::isfinite(delta);
            if (!exact) break;
            lambda[row] = next;
            if (delta == 0.0) continue;
            ++coordinate_updates;
            coordinate_update_terms += degree;
            for (std::size_t entry = op.entry_offsets[row];
                 entry < op.entry_offsets[row + 1U]; ++entry) {
                const ALR64Entry& item = op.entries[entry];
                q[item.particle] = q[item.particle]
                    - delta * item.coefficient;
            }
        }
        exact = exact && al_r48_vec_finite(q) && al_r30_finite(lambda);
        if (!exact) break;

        const ALLinearizedJvp pre_image = al_r64_action(op, q);
        ++action_calls;
        exact = pre_image.passed && pre_image.value.size() == 6000U;
        std::vector<double> raw(6000U);
        std::size_t face_count = 0U;
        for (std::size_t row = 0U; row < 6000U && exact; ++row) {
            raw[row] = workspace.support.constraint[row]
                + pre_image.value[row];
            if (owned[row] != 0U && lambda[row] > 0.0) ++face_count;
        }
        exact = exact && al_r30_finite(raw) && face_count > 0U;
        if (!exact) break;

        ALR65PProportioningDiagnostic diagnostic;
        diagnostic.outer = outer;
        long double diagnostic_free_squared = 0.0L;
        long double diagnostic_chopped_squared = 0.0L;
        long double diagnostic_projected_squared = 0.0L;
        for (std::size_t row = 0U; row < 6000U; ++row) {
            if (owned[row] == 0U) continue;
            const double gradient = -raw[row];
            const double free = lambda[row] > 0.0 ? gradient : 0.0;
            const double chopped = lambda[row] == 0.0
                ? std::min(gradient, 0.0) : 0.0;
            const double projected = lambda[row] > 0.0
                ? gradient : std::min(gradient, 0.0);
            diagnostic_free_squared += static_cast<long double>(free) * free;
            diagnostic_chopped_squared +=
                static_cast<long double>(chopped) * chopped;
            diagnostic_projected_squared +=
                static_cast<long double>(projected) * projected;
        }
        diagnostic.free_squared = static_cast<double>(
            diagnostic_free_squared);
        diagnostic.chopped_squared = static_cast<double>(
            diagnostic_chopped_squared);
        diagnostic.projected_squared = static_cast<double>(
            diagnostic_projected_squared);
        diagnostic.decomposed_squared = static_cast<double>(
            diagnostic_free_squared + diagnostic_chopped_squared);
        diagnostic.free_norm = std::sqrt(diagnostic.free_squared);
        diagnostic.chopped_norm = std::sqrt(diagnostic.chopped_squared);
        diagnostic.projected_norm = std::sqrt(
            diagnostic.projected_squared);
        diagnostic.chopped_free_ratio = diagnostic.free_norm > 0.0
            ? diagnostic.chopped_norm / diagnostic.free_norm
            : (diagnostic.chopped_norm > 0.0
                ? std::numeric_limits<double>::max() : 0.0);
        diagnostic.decomposition_gap = std::abs(
            diagnostic.projected_squared - diagnostic.decomposed_squared);
        diagnostic.decomposition_bound = gamma_factor(64U * 6000U + 256U)
            * (std::abs(diagnostic.projected_squared)
                + std::abs(diagnostic.free_squared)
                + std::abs(diagnostic.chopped_squared));
        diagnostic.decomposition_exact =
            std::isfinite(diagnostic.free_squared)
            && std::isfinite(diagnostic.chopped_squared)
            && std::isfinite(diagnostic.projected_squared)
            && std::isfinite(diagnostic.decomposed_squared)
            && std::isfinite(diagnostic.free_norm)
            && std::isfinite(diagnostic.chopped_norm)
            && std::isfinite(diagnostic.projected_norm)
            && std::isfinite(diagnostic.chopped_free_ratio)
            && std::isfinite(diagnostic.decomposition_gap)
            && std::isfinite(diagnostic.decomposition_bound)
            && diagnostic.decomposition_gap
                <= diagnostic.decomposition_bound;
        diagnostic.stationary = diagnostic.free_squared == 0.0
            && diagnostic.chopped_squared == 0.0;
        diagnostic.chopped_dominant = !diagnostic.stationary
            && diagnostic.chopped_norm > diagnostic.free_norm;
        diagnostic_decomposition_exact = diagnostic_decomposition_exact
            && diagnostic.decomposition_exact;

        const ALR65PBlock block = al_r65p_block(
            op, owned, diagonal, lambda, q, raw, correction, lower, upper,
            r43.projected_trial, r30.predicted_position, normal_inertia);
        face_pcg_iterations += block.face_iterations;
        candidate_trials += block.candidate_trials;
        baseline_projection_calls += block.baseline_projection_calls;
        candidate_projection_calls += block.candidate_projection_calls;
        candidate_dual_rejections += block.dual_rejections;
        candidate_model_rejections += block.model_rejections;
        candidate_composed_rejections += block.composed_rejections;
        admissible_candidates += block.admissible_candidates;
        if (block.line_exact) ++accepted_lines;
        transpose_calls += block.transpose_calls;
        action_calls += block.action_calls;
        curvature_stops += block.curvature_stops;
        batch_predicted_zeros += block.predicted_zeros;
        batch_applied_zeros += block.applied_zeros;
        curvature_exact = curvature_exact && block.curvature_exact;
        line_exact = line_exact && block.line_exact;
        normal_model_rejected = normal_model_rejected
            || (!block.line_exact && block.model_rejections > 0U);
        composed_model_rejected = composed_model_rejected
            || (!block.line_exact && block.composed_rejections > 0U);
        diagnostic.line_accepted = block.line_exact;
        if (!diagnostic.line_accepted) {
            ++blocked_diagnostics;
            if (diagnostic.stationary) ++blocked_stationary;
            else if (diagnostic.chopped_dominant)
                ++blocked_chopped_dominant;
            else ++blocked_free_dominant;
        }
        std::ostringstream diagnostic_material;
        diagnostic_material << std::setprecision(17) << diagnostic.outer << ':'
            << diagnostic.free_squared << ':'
            << diagnostic.chopped_squared << ':'
            << diagnostic.projected_squared << ':'
            << diagnostic.decomposed_squared << ':'
            << diagnostic.free_norm << ':' << diagnostic.chopped_norm << ':'
            << diagnostic.projected_norm << ':'
            << diagnostic.chopped_free_ratio << ':'
            << diagnostic.decomposition_gap << ':'
            << diagnostic.decomposition_bound << ':'
            << diagnostic.decomposition_exact << ':'
            << diagnostic.chopped_dominant << ':'
            << diagnostic.stationary << ':' << diagnostic.line_accepted;
        diagnostic.root = sha256_hex(diagnostic_material.str());
        diagnostics.push_back(diagnostic);
        exact = block.passed;
        if (!exact) break;
        lambda = block.lambda;
        const std::vector<Vec3> maintained_q = block.q;
        at_lambda = al_r64_transpose(op, lambda, nullptr);
        ++transpose_calls;
        exact = at_lambda.size() == 6000U && al_r48_vec_finite(at_lambda);
        std::vector<Vec3> q_difference(6000U);
        for (std::size_t i = 0U; i < 6000U && exact; ++i)
        {
            q[i] = capture.target[i] - correction[i] - at_lambda[i];
            q_difference[i] = maintained_q[i] - q[i];
        }
        const double recurrence_gap = al_r32_vec3_norm(q_difference);
        const double recurrence_bound = gamma_factor(
            64U * (coordinate_updates + face_pcg_iterations
                + candidate_trials + batch_applied_zeros)
                + 256U)
            * (al_r32_vec3_norm(maintained_q) + al_r32_vec3_norm(q));
        const bool outer_recurrence = std::isfinite(recurrence_gap)
            && std::isfinite(recurrence_bound)
            && recurrence_gap <= recurrence_bound;
        recurrence_exact = recurrence_exact && outer_recurrence;
        exact = exact && std::isfinite(recurrence_gap)
            && std::isfinite(recurrence_bound);
        if (!exact) break;
        const ALLinearizedJvp terminal_image = al_r64_action(op, q);
        ++action_calls;
        exact = exact && terminal_image.passed
            && terminal_image.value.size() == 6000U;
        std::vector<double> terminal_raw(6000U);
        long double projected_squared = 0.0L;
        long double lambda_raw = 0.0L;
        for (std::size_t row = 0U; row < 6000U && exact; ++row) {
            terminal_raw[row] = workspace.support.constraint[row]
                + terminal_image.value[row];
            const double gradient = -terminal_raw[row];
            const double projected = owned[row] == 0U ? 0.0
                : (lambda[row] > 0.0 ? gradient
                    : std::min(gradient, 0.0));
            projected_squared += static_cast<long double>(projected)
                * projected;
            lambda_raw += static_cast<long double>(lambda[row])
                * terminal_raw[row];
        }
        const double projected_gradient_norm = std::sqrt(
            static_cast<double>(projected_squared));
        const double dual_objective = -0.5 * al_r65f_vec_squared(at_lambda)
            - static_cast<double>(lambda_raw);
        exact = exact && al_r30_finite(terminal_raw)
            && std::isfinite(projected_gradient_norm)
            && std::isfinite(dual_objective);
        if (!exact) break;

        std::vector<Vec3> joint_input(6000U);
        for (std::size_t i = 0U; i < 6000U; ++i)
            joint_input[i] = capture.target[i] - at_lambda[i];
        const ALR48Projection projection = al_r48_project_ball_box(
            joint_input, lower, upper, AL_R62_NEXT_RADIUS);
        ++projection_calls;
        exact = projection.passed;
        if (!exact) break;
        composite = projection.value;
        for (std::size_t i = 0U; i < 6000U; ++i)
            correction[i] = joint_input[i] - composite[i];
        std::vector<Vec3> committed_endpoint(6000U);
        for (std::size_t i = 0U; i < 6000U; ++i)
            committed_endpoint[i] = r43.projected_trial[i]
                + SPACING * composite[i];
        const double committed_inertia = al_r63_inertia(
            committed_endpoint, r30.predicted_position);
        const double committed_model_reduction =
            normal_inertia - committed_inertia;
        const double model_prediction_gap = block.line_exact
            ? std::abs(block.candidate_model_reduction
                - committed_model_reduction) : 0.0;
        const double model_prediction_bound = block.line_exact
            ? gamma_factor(128U * 6000U + 512U)
                * (std::abs(normal_inertia)
                    + std::abs(block.candidate_inertia)
                    + std::abs(committed_inertia))
            : 0.0;
        const bool outer_model_agreement = !block.line_exact
            || (std::isfinite(model_prediction_gap)
                && std::isfinite(model_prediction_bound)
                && model_prediction_gap <= model_prediction_bound);
        const bool outer_model_exact = std::isfinite(committed_inertia)
            && std::isfinite(committed_model_reduction)
            && committed_model_reduction > 0.0;
        model_agreement_exact = model_agreement_exact
            && outer_model_agreement;
        model_exact = model_exact && outer_model_exact;
        audit = al_r61_topology_owned_audit(workspace, composite);
        ++audit_calls;
        exact = audit.owner_exact && al_r48_vec_finite(correction);
        std::size_t additions = 0U;
        for (std::size_t row = 0U; row < 6000U; ++row)
            if (audit.candidate_upper[row] > 0.0 && owned[row] == 0U) {
                owned[row] = 1U;
                ++additions;
            }
        total_additions += additions;
        if (!al_r65p_checkpoint(outer)) continue;

        ALR65PRecord record;
        record.outer = outer;
        record.working = static_cast<std::size_t>(std::count(
            owned.begin(), owned.end(), static_cast<std::uint8_t>(1U)));
        record.additions = additions;
        record.raw_positive = audit.raw_positive;
        record.candidate_positive = audit.candidate_positive;
        record.nonzero_lambda = static_cast<std::size_t>(std::count_if(
            lambda.begin(), lambda.end(),
            [](double value) { return value > 0.0; }));
        record.face = face_count;
        record.pcg_iterations = block.face_iterations;
        record.candidate_trials = block.candidate_trials;
        record.baseline_projection_calls =
            block.baseline_projection_calls;
        record.candidate_projection_calls =
            block.candidate_projection_calls;
        record.dual_rejections = block.dual_rejections;
        record.model_rejections = block.model_rejections;
        record.composed_rejections = block.composed_rejections;
        record.admissible_candidates = block.admissible_candidates;
        record.selected_exponent = block.selected_exponent;
        record.predicted_zeros = block.predicted_zeros;
        record.applied_zeros = block.applied_zeros;
        record.maximum_raw = *std::max_element(
            audit.raw.begin(), audit.raw.end());
        record.projected_gradient_norm = projected_gradient_norm;
        record.pcg_initial_residual_norm = block.initial_residual_norm;
        record.pcg_final_residual_norm = block.final_residual_norm;
        record.pcg_descent = block.descent;
        record.line_alpha = block.line_alpha;
        record.line_slope = block.line_slope;
        record.line_curvature = block.line_curvature;
        record.baseline_inertia = block.baseline_inertia;
        record.baseline_model_reduction =
            block.baseline_model_reduction;
        record.candidate_model_reduction =
            block.candidate_model_reduction;
        record.committed_model_reduction = committed_model_reduction;
        record.model_prediction_gap = model_prediction_gap;
        record.model_prediction_bound = model_prediction_bound;
        record.recurrence_gap = recurrence_gap;
        record.recurrence_bound = recurrence_bound;
        record.dual_objective = dual_objective;
        record.batch_root = block.batch_root;
        std::vector<Vec3> stationarity(6000U);
        for (std::size_t i = 0U; i < 6000U; ++i)
            stationarity[i] = composite[i] - capture.target[i]
                + at_lambda[i] + correction[i];
        record.stationarity_norm = al_r32_vec3_norm(stationarity);
        for (std::size_t row = 0U; row < 6000U; ++row)
            record.complementarity_maximum = std::max(
                record.complementarity_maximum,
                std::abs(lambda[row] * audit.raw[row]));
        std::vector<Vec3> reprojection_input(6000U);
        for (std::size_t i = 0U; i < 6000U; ++i)
            reprojection_input[i] = composite[i] + correction[i];
        const ALR48Projection reprojection = al_r48_project_ball_box(
            reprojection_input, lower, upper, AL_R62_NEXT_RADIUS);
        record.reprojection_exact = reprojection.passed
            && exact_vec3_values(reprojection.value, composite);
        record.inertia = committed_inertia;
        record.model_reduction = committed_model_reduction;
        std::ostringstream material;
        material << std::setprecision(17) << record.outer << ':'
            << record.working << ':' << record.additions << ':'
            << record.raw_positive << ':' << record.candidate_positive << ':'
            << record.nonzero_lambda << ':' << record.face << ':'
            << record.pcg_iterations << ':' << record.candidate_trials << ':'
            << record.baseline_projection_calls << ':'
            << record.candidate_projection_calls << ':'
            << record.dual_rejections << ':' << record.model_rejections << ':'
            << record.composed_rejections << ':'
            << record.admissible_candidates << ':'
            << record.selected_exponent << ':' << record.predicted_zeros << ':'
            << record.applied_zeros << ':'
            << record.maximum_raw << ':' << record.projected_gradient_norm
            << ':' << record.pcg_initial_residual_norm << ':'
            << record.pcg_final_residual_norm << ':' << record.pcg_descent
            << ':' << record.line_alpha << ':' << record.line_slope << ':'
            << record.line_curvature << ':' << record.recurrence_gap << ':'
            << record.baseline_inertia << ':'
            << record.baseline_model_reduction << ':'
            << record.candidate_model_reduction << ':'
            << record.committed_model_reduction << ':'
            << record.model_prediction_gap << ':'
            << record.model_prediction_bound << ':'
            << record.recurrence_bound << ':' << record.dual_objective << ':'
            << record.stationarity_norm << ':'
            << record.complementarity_maximum << ':' << record.inertia << ':'
            << record.model_reduction << ':' << record.reprojection_exact
            << ':' << record.batch_root << ':'
            << al_binary64_vec3_root(composite) << ':'
            << al_r10_dual_root(lambda);
        record.root = sha256_hex(material.str());
        exact = exact && record.reprojection_exact
            && std::isfinite(record.maximum_raw)
            && std::isfinite(record.stationarity_norm)
            && std::isfinite(record.complementarity_maximum);
        records.push_back(record);
    }

    if (workspace.passed)
        release_al_normalized_workspace(workspace, &workspace_work);
    const std::size_t structural_terms =
        transpose_calls * op.entries.size()
        + (action_calls + audit_calls) * op.slots.size()
        + coordinate_residual_terms + coordinate_update_terms;
    const bool lower_work = structural_terms < AL_R65P_FISTA_STRUCTURAL_TERMS;
    const bool dominates = records.size() == AL_R65P_CHECKPOINTS.size()
        && records.back().maximum_raw < AL_R65P_FISTA_MAXIMUM_RAW
        && records.back().projected_gradient_norm
            < AL_R65P_FISTA_PROJECTED_GRADIENT;
    const bool work_exact = workspace_work.workspace_builds == 1U
        && workspace_work.workspace_releases == 1U
        && workspace_work.live_workspaces == 0U
        && identification_sweeps == AL_R65P_OUTERS
        && face_pcg_iterations
            <= AL_R65P_OUTERS * (AL_R65P_INNERS - 1U)
        && candidate_trials == 16U * AL_R65P_OUTERS
        && baseline_projection_calls == AL_R65P_OUTERS
        && candidate_projection_calls <= candidate_trials
        && candidate_dual_rejections + candidate_projection_calls
            == candidate_trials
        && candidate_model_rejections + candidate_composed_rejections
            + admissible_candidates
            == candidate_projection_calls
        && accepted_lines <= AL_R65P_OUTERS
        && accepted_lines <= admissible_candidates
        && transpose_calls == 2U * AL_R65P_OUTERS
            + face_pcg_iterations + candidate_trials
        && action_calls == 2U * AL_R65P_OUTERS + face_pcg_iterations
        && projection_calls == AL_R65P_OUTERS
        && audit_calls == AL_R65P_OUTERS + 1U
        && records.size() == AL_R65P_CHECKPOINTS.size()
        && total_additions == 4680U && lower_work;
    const bool rollback = trial_before
            == al_binary64_vec3_root(r43.projected_trial)
        && target_before == al_binary64_vec3_root(capture.target);
    const bool passed = exact && parent_exact && workspace_exact
        && operator_exact && geometry_exact && work_exact && rollback;
    const std::string route = al_r65p_route(passed, curvature_exact,
        line_exact, normal_model_rejected, composed_model_rejected,
        model_agreement_exact, recurrence_exact, model_exact, lower_work,
        dominates);

    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
        << parent_stdout << '|' << route << '|' << initial_working << ':'
        << total_additions << ':' << identification_sweeps << ':'
        << coordinate_visits << ':' << coordinate_updates << ':'
        << coordinate_residual_terms << ':' << coordinate_update_terms << ':'
        << face_pcg_iterations << ':' << candidate_trials << ':'
        << baseline_projection_calls << ':' << candidate_projection_calls
        << ':' << candidate_dual_rejections << ':'
        << candidate_model_rejections << ':'
        << candidate_composed_rejections << ':' << admissible_candidates
        << ':' << accepted_lines << ':'
        << curvature_stops << ':'
        << batch_predicted_zeros << ':' << batch_applied_zeros << ':'
        << transpose_calls << ':'
        << action_calls << ':' << structural_terms << '|';
    for (const ALR65PRecord& record : records) semantic << record.root << ':';
    semantic << "|EXPLORATORY_ONLY_NO_CREDIT";
    const std::string result_sha = sha256_hex(semantic.str());

    const ALR65PProportioningControl diagnostic_control =
        al_r65p_proportioning_control();
    const bool solver_identity_exact = passed
        && result_sha == AL_R65P_V6_RESULT_SHA256;
    const bool diagnostic_work_exact = diagnostic_decomposition_exact
        && diagnostics.size() == AL_R65P_OUTERS
        && blocked_diagnostics + accepted_lines == AL_R65P_OUTERS
        && blocked_chopped_dominant + blocked_free_dominant
                + blocked_stationary
            == blocked_diagnostics;
    const bool diagnostic_exact = solver_identity_exact
        && diagnostic_control.successful_exact
        && diagnostic_control.negative_rejected
        && diagnostic_work_exact;
    const std::string diagnostic_route = al_r65p_diagnostic_route(
        solver_identity_exact, diagnostic_exact, blocked_diagnostics,
        blocked_chopped_dominant, blocked_free_dominant,
        blocked_stationary);
    std::ostringstream diagnostic_semantic;
    diagnostic_semantic << std::setprecision(17)
        << (diagnostic_exact ? "PASS|" : "FAIL|")
        << result_sha << '|' << diagnostic_route << '|'
        << diagnostic_control.free_squared << ':'
        << diagnostic_control.chopped_squared << ':'
        << diagnostic_control.projected_squared << ':'
        << diagnostic_control.wrong_sign_squared << ':'
        << diagnostic_control.successful_exact << ':'
        << diagnostic_control.negative_rejected << '|'
        << blocked_diagnostics << ':' << blocked_chopped_dominant << ':'
        << blocked_free_dominant << ':' << blocked_stationary << '|';
    for (const ALR65PProportioningDiagnostic& diagnostic : diagnostics)
        diagnostic_semantic << diagnostic.root << ':';
    diagnostic_semantic << "|MEASUREMENT_ONLY_NO_SOLVER_CREDIT";
    const std::string diagnostic_result_sha = sha256_hex(
        diagnostic_semantic.str());
    const bool report_passed = passed && diagnostic_exact;

    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r65_proportioning_filter_diagnostic.v7\",\"status\":\""
        << (report_passed ? "PASS" : "FAIL")
        << "\",\"authority\":\"EXPLORATORY_ONLY_NO_CREDIT\""
        << ",\"route\":\"" << route << "\""
        << ",\"diagnostic_route\":\"" << diagnostic_route << "\""
        << ",\"solver_identity_exact\":"
        << (solver_identity_exact ? "true" : "false")
        << ",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
        << ",\"initial_working\":" << initial_working
        << ",\"records\":[";
    for (std::size_t i = 0U; i < records.size(); ++i) {
        if (i != 0U) report << ',';
        const ALR65PRecord& record = records[i];
        report << "{\"outer\":" << record.outer
            << ",\"working\":" << record.working
            << ",\"additions\":" << record.additions
            << ",\"raw_positive\":" << record.raw_positive
            << ",\"candidate_positive\":" << record.candidate_positive
            << ",\"nonzero_lambda\":" << record.nonzero_lambda
            << ",\"face\":" << record.face
            << ",\"pcg_iterations\":" << record.pcg_iterations
            << ",\"candidate_trials\":" << record.candidate_trials
            << ",\"baseline_projection_calls\":"
            << record.baseline_projection_calls
            << ",\"candidate_projection_calls\":"
            << record.candidate_projection_calls
            << ",\"dual_rejections\":" << record.dual_rejections
            << ",\"model_rejections\":" << record.model_rejections
            << ",\"composed_rejections\":"
            << record.composed_rejections
            << ",\"admissible_candidates\":"
            << record.admissible_candidates
            << ",\"selected_exponent\":" << record.selected_exponent
            << ",\"predicted_zeros\":" << record.predicted_zeros
            << ",\"applied_zeros\":" << record.applied_zeros
            << ",\"maximum_raw\":" << record.maximum_raw
            << ",\"projected_gradient_norm\":"
            << record.projected_gradient_norm
            << ",\"pcg_initial_residual_norm\":"
            << record.pcg_initial_residual_norm
            << ",\"pcg_final_residual_norm\":"
            << record.pcg_final_residual_norm
            << ",\"pcg_descent\":" << record.pcg_descent
            << ",\"line_alpha\":" << record.line_alpha
            << ",\"line_slope\":" << record.line_slope
            << ",\"line_curvature\":" << record.line_curvature
            << ",\"baseline_inertia\":" << record.baseline_inertia
            << ",\"baseline_model_reduction\":"
            << record.baseline_model_reduction
            << ",\"candidate_model_reduction\":"
            << record.candidate_model_reduction
            << ",\"committed_model_reduction\":"
            << record.committed_model_reduction
            << ",\"model_prediction_gap\":"
            << record.model_prediction_gap
            << ",\"model_prediction_bound\":"
            << record.model_prediction_bound
            << ",\"recurrence_gap\":" << record.recurrence_gap
            << ",\"recurrence_bound\":" << record.recurrence_bound
            << ",\"dual_objective\":" << record.dual_objective
            << ",\"stationarity_norm\":" << record.stationarity_norm
            << ",\"complementarity_maximum\":"
            << record.complementarity_maximum
            << ",\"inertia\":" << record.inertia
            << ",\"model_reduction\":" << record.model_reduction
            << ",\"reprojection_exact\":"
            << (record.reprojection_exact ? "true" : "false")
            << ",\"batch_root\":\"" << record.batch_root << "\""
            << ",\"root\":\"" << record.root << "\"}";
    }
    report << "],\"diagnostic\":{\"research_id\":\"NSR3-B4E2D7R19R65-PROP-FILTER-R1\""
        << ",\"measurement_only\":true"
        << ",\"decomposition_exact\":"
        << (diagnostic_decomposition_exact ? "true" : "false")
        << ",\"work_exact\":"
        << (diagnostic_work_exact ? "true" : "false")
        << ",\"blocked_outers\":" << blocked_diagnostics
        << ",\"blocked_chopped_dominant\":"
        << blocked_chopped_dominant
        << ",\"blocked_free_dominant\":" << blocked_free_dominant
        << ",\"blocked_stationary\":" << blocked_stationary
        << ",\"control\":{\"free_squared\":"
        << diagnostic_control.free_squared
        << ",\"chopped_squared\":"
        << diagnostic_control.chopped_squared
        << ",\"projected_squared\":"
        << diagnostic_control.projected_squared
        << ",\"wrong_sign_squared\":"
        << diagnostic_control.wrong_sign_squared
        << ",\"successful_exact\":"
        << (diagnostic_control.successful_exact ? "true" : "false")
        << ",\"negative_rejected\":"
        << (diagnostic_control.negative_rejected ? "true" : "false")
        << "},\"states\":[";
    for (std::size_t i = 0U; i < diagnostics.size(); ++i) {
        if (i != 0U) report << ',';
        const ALR65PProportioningDiagnostic& diagnostic = diagnostics[i];
        const char* classification = diagnostic.stationary
            ? "STATIONARY"
            : (diagnostic.chopped_dominant
                ? "CHOPPED_DOMINANT" : "FREE_DOMINANT");
        report << "{\"outer\":" << diagnostic.outer
            << ",\"free_squared\":" << diagnostic.free_squared
            << ",\"chopped_squared\":" << diagnostic.chopped_squared
            << ",\"projected_squared\":"
            << diagnostic.projected_squared
            << ",\"decomposed_squared\":"
            << diagnostic.decomposed_squared
            << ",\"free_norm\":" << diagnostic.free_norm
            << ",\"chopped_norm\":" << diagnostic.chopped_norm
            << ",\"projected_norm\":" << diagnostic.projected_norm
            << ",\"chopped_free_ratio\":"
            << diagnostic.chopped_free_ratio
            << ",\"decomposition_gap\":"
            << diagnostic.decomposition_gap
            << ",\"decomposition_bound\":"
            << diagnostic.decomposition_bound
            << ",\"decomposition_exact\":"
            << (diagnostic.decomposition_exact ? "true" : "false")
            << ",\"classification\":\"" << classification << "\""
            << ",\"line_accepted\":"
            << (diagnostic.line_accepted ? "true" : "false")
            << ",\"root\":\"" << diagnostic.root << "\"}";
    }
    report << "]},\"work\":{\"outer_blocks\":" << identification_sweeps
        << ",\"identification_sweeps\":" << identification_sweeps
        << ",\"coordinate_visits\":" << coordinate_visits
        << ",\"coordinate_updates\":" << coordinate_updates
        << ",\"coordinate_residual_terms\":" << coordinate_residual_terms
        << ",\"coordinate_update_terms\":" << coordinate_update_terms
        << ",\"face_pcg_iterations\":" << face_pcg_iterations
        << ",\"projected_candidate_trials\":" << candidate_trials
        << ",\"baseline_joint_projections\":"
        << baseline_projection_calls
        << ",\"candidate_joint_projections\":"
        << candidate_projection_calls
        << ",\"candidate_dual_rejections\":"
        << candidate_dual_rejections
        << ",\"candidate_model_rejections\":"
        << candidate_model_rejections
        << ",\"candidate_composed_rejections\":"
        << candidate_composed_rejections
        << ",\"admissible_candidates\":" << admissible_candidates
        << ",\"accepted_lines\":" << accepted_lines
        << ",\"curvature_stops\":" << curvature_stops
        << ",\"batch_predicted_zeros\":" << batch_predicted_zeros
        << ",\"batch_applied_zeros\":" << batch_applied_zeros
        << ",\"transpose_calls\":" << transpose_calls
        << ",\"action_calls\":" << action_calls
        << ",\"joint_projections\":" << projection_calls
        << ",\"all_row_audits\":" << audit_calls
        << ",\"structural_terms\":" << structural_terms
        << ",\"fista_structural_terms\":"
        << AL_R65P_FISTA_STRUCTURAL_TERMS
        << ",\"dense_gram_storage\":0,\"lower_than_fista\":"
        << (lower_work ? "true" : "false")
        << ",\"exact\":" << (work_exact ? "true" : "false") << "}"
        << ",\"dominates_fista\":" << (dominates ? "true" : "false")
        << ",\"model_agreement_exact\":"
        << (model_agreement_exact ? "true" : "false")
        << ",\"rollback_exact\":" << (rollback ? "true" : "false")
        << ",\"nonlinear_trial_executed\":false"
           ",\"runtime_state_mutated\":false,\"timing_admitted\":false"
           ",\"production_authority\":false"
        << ",\"solver_result_sha256\":\"" << result_sha << "\""
        << ",\"diagnostic_result_sha256\":\""
        << diagnostic_result_sha << "\""
        << ",\"result_sha256\":\"" << diagnostic_result_sha << "\"}";
    return {report_passed, report.str()};
}
