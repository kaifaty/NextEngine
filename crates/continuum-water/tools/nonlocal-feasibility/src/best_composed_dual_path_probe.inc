namespace {

constexpr const char* AL_R65D_PARENT_SHA256 =
    "77cbed07c8cdc615110677cbae0266946b1fed777f2ba6ceba875a1ba4f583af";

struct ALR65DRecord {
    std::size_t outer = 0U;
    std::size_t working = 0U;
    std::size_t additions = 0U;
    std::size_t face = 0U;
    std::size_t pcg_iterations = 0U;
    std::size_t candidate_trials = 0U;
    std::size_t candidate_projections = 0U;
    std::size_t normal_rejections = 0U;
    std::size_t dual_rejections = 0U;
    std::size_t admissible_candidates = 0U;
    std::size_t selected_exponent = 16U;
    std::size_t applied_zeros = 0U;
    double line_alpha = 0.0;
    double maximum_raw = 0.0;
    double projected_gradient_norm = 0.0;
    double baseline_composed_dual = 0.0;
    double predicted_composed_dual = 0.0;
    double committed_composed_dual = 0.0;
    double composed_dual_change = 0.0;
    double dual_prediction_gap = 0.0;
    double dual_prediction_bound = 0.0;
    double model_reduction = 0.0;
    double model_prediction_gap = 0.0;
    double model_prediction_bound = 0.0;
    double recurrence_gap = 0.0;
    double recurrence_bound = 0.0;
    double stationarity_norm = 0.0;
    double complementarity_maximum = 0.0;
    bool line_accepted = false;
    bool dual_prediction_exact = false;
    bool model_prediction_exact = false;
    bool reprojection_exact = false;
    std::string batch_root;
    std::string root;
};

bool al_r65d_dense_controls() {
    double best_change = 0.0;
    std::size_t best = 2U;
    const std::array<double, 2U> dual_change{{2.0, 3.0}};
    const std::array<double, 2U> primal_reduction{{4.0, 1.0}};
    for (std::size_t candidate = 0U; candidate < dual_change.size();
         ++candidate) {
        const bool fixed_dual = true;
        const bool normal_safe = true;
        if (!fixed_dual || !normal_safe || dual_change[candidate] <= 0.0)
            continue;
        if (dual_change[candidate] <= best_change) continue;
        best_change = dual_change[candidate];
        best = candidate;
    }
    const bool selects_dual = best == 1U && primal_reduction[best] == 1.0;
    const bool normal_reject = !(true && false && 4.0 > 0.0);
    const bool equality_reject = !(0.0 > 0.0)
        && !(-0.0 > 0.0);
    const double prediction = 3.0;
    const double committed = 3.0;
    const double bound = gamma_factor(256U + 1024U)
        * (std::abs(prediction) + std::abs(committed));
    const bool agreement = std::abs(prediction - committed) <= bound
        && std::abs(prediction - (committed + 1.0)) > bound;
    return selects_dual && normal_reject && equality_reject && agreement;
}

std::string al_r65d_route(bool exact, bool line, bool dual, bool model,
    bool lower_work, bool dominates) {
    if (!exact || !line || !dual || !model)
        return "COMPOSED_DUAL_TRANSACTION_REJECTED";
    if (lower_work && dominates)
        return "COMPOSED_DUAL_ACCELERATION_CANDIDATE";
    return "COMPOSED_DUAL_PATH_REFERENCE_RETAINED";
}

bool al_r65d_route_controls() {
    return al_r65d_route(false, true, true, true, true, true)
            == "COMPOSED_DUAL_TRANSACTION_REJECTED"
        && al_r65d_route(true, false, true, true, true, true)
            == "COMPOSED_DUAL_TRANSACTION_REJECTED"
        && al_r65d_route(true, true, false, true, true, true)
            == "COMPOSED_DUAL_TRANSACTION_REJECTED"
        && al_r65d_route(true, true, true, true, true, true)
            == "COMPOSED_DUAL_ACCELERATION_CANDIDATE"
        && al_r65d_route(true, true, true, true, true, false)
            == "COMPOSED_DUAL_PATH_REFERENCE_RETAINED";
}

} // namespace

SplitBoundaryReport run_al_best_composed_dual_path_controls() {
    ALR65PReplayCapture parent_capture;
    const SplitBoundaryReport parent =
        run_al_active_face_pcg_probe_controls_impl(&parent_capture);
    const bool parent_exact = parent.passed && parent_capture.captured
        && parent_capture.result_sha == AL_R65D_PARENT_SHA256;
    if (!parent_exact)
        return {false,
            "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r65_best_composed_dual_path.v10\",\"status\":\"FAIL\",\"route\":\"COMPOSED_DUAL_TRANSACTION_REJECTED\",\"parent_exact\":false}"};

    ALR63ReplayCapture& capture = parent_capture.r63;
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
    const bool workspace_exact = workspace.passed && index.passed
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
    for (std::size_t particle = 0U; particle < 6000U; ++particle)
        for (int axis = 0; axis < 3; ++axis) {
            const double y = al_r41_component(
                r43.projected_trial[particle], axis);
            const double low_face = al_r41_component(
                fixture.contact_low, axis);
            const double high_face = al_r41_component(
                fixture.contact_high, axis);
            double low = (low_face - y) / SPACING;
            double high = (high_face - y) / SPACING;
            if (y < low_face) low = 0.0;
            else if (y > high_face) high = 0.0;
            al_r43_component(lower[particle], axis) = low;
            al_r43_component(upper[particle], axis) = high;
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
    for (std::size_t particle = 0U; particle < 6000U; ++particle)
        normal_endpoint[particle] = r43.projected_trial[particle]
            + SPACING * r62.publication.cached_normal[particle];
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
    bool dual_prediction_exact = true;
    bool model_prediction_exact = true;
    bool recurrence_exact = true;
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
    std::size_t candidate_fixed_dual_rejections = 0U;
    std::size_t candidate_normal_rejections = 0U;
    std::size_t candidate_composed_dual_rejections = 0U;
    std::size_t admissible_candidates = 0U;
    std::size_t accepted_lines = 0U;
    std::size_t transpose_calls = 0U;
    std::size_t action_calls = 0U;
    std::size_t projection_calls = 0U;
    std::size_t audit_calls = 1U;
    std::size_t checkpoint_reprojections = 0U;
    std::size_t composed_dual_vector_terms = 0U;
    std::vector<ALR65DRecord> records;

    for (std::size_t outer = 1U; outer <= AL_R65P_OUTERS && exact; ++outer) {
        std::vector<Vec3> at_lambda = al_r64_transpose(op, lambda, nullptr);
        ++transpose_calls;
        exact = at_lambda.size() == 6000U && al_r48_vec_finite(at_lambda);
        std::vector<Vec3> q(6000U);
        for (std::size_t particle = 0U; particle < 6000U && exact; ++particle)
            q[particle] = capture.target[particle] - correction[particle]
                - at_lambda[particle];
        ++identification_sweeps;
        for (std::size_t row = 0U; row < 6000U && exact; ++row) {
            if (owned[row] == 0U) continue;
            ++coordinate_visits;
            const std::size_t degree = op.entry_offsets[row + 1U]
                - op.entry_offsets[row];
            coordinate_residual_terms += degree;
            const double residual = al_r65_sparse_residual(
                workspace, op, row, q);
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

        const ALR65PBlock block = al_r65p_block(
            op, workspace, outer, ALR65PSelectionPolicy::ComposedDual,
            nullptr, owned, diagonal, lambda, q, raw, correction, lower,
            upper, r43.projected_trial, capture.target,
            r30.predicted_position, normal_inertia);
        face_pcg_iterations += block.face_iterations;
        candidate_trials += block.candidate_trials;
        baseline_projection_calls += block.baseline_projection_calls;
        candidate_projection_calls += block.candidate_projection_calls;
        candidate_fixed_dual_rejections += block.dual_rejections;
        candidate_normal_rejections += block.model_rejections;
        candidate_composed_dual_rejections += block.composed_rejections;
        admissible_candidates += block.admissible_candidates;
        if (block.line_exact) ++accepted_lines;
        transpose_calls += block.transpose_calls;
        action_calls += block.action_calls;
        composed_dual_vector_terms += block.composed_dual_vector_terms;
        curvature_exact = curvature_exact && block.curvature_exact;
        line_exact = line_exact && block.line_exact;
        exact = block.passed && block.composed_dual_exact;
        if (!exact) break;
        lambda = block.lambda;
        const std::vector<Vec3> maintained_q = block.q;

        at_lambda = al_r64_transpose(op, lambda, nullptr);
        ++transpose_calls;
        exact = at_lambda.size() == 6000U && al_r48_vec_finite(at_lambda);
        std::vector<Vec3> q_difference(6000U);
        for (std::size_t particle = 0U; particle < 6000U && exact; ++particle) {
            q[particle] = capture.target[particle] - correction[particle]
                - at_lambda[particle];
            q_difference[particle] = maintained_q[particle] - q[particle];
        }
        const double recurrence_gap = al_r32_vec3_norm(q_difference);
        const double recurrence_bound = gamma_factor(
            64U * (coordinate_updates + face_pcg_iterations
                + candidate_trials + block.applied_zeros) + 256U)
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
        exact = terminal_image.passed && terminal_image.value.size() == 6000U;
        std::vector<double> terminal_raw(6000U);
        long double projected_squared = 0.0L;
        for (std::size_t row = 0U; row < 6000U && exact; ++row) {
            terminal_raw[row] = workspace.support.constraint[row]
                + terminal_image.value[row];
            const double gradient = -terminal_raw[row];
            const double projected = owned[row] == 0U ? 0.0
                : (lambda[row] > 0.0 ? gradient
                    : std::min(gradient, 0.0));
            projected_squared += static_cast<long double>(projected)
                * projected;
        }
        const double projected_gradient_norm = std::sqrt(
            static_cast<double>(projected_squared));
        exact = exact && al_r30_finite(terminal_raw)
            && std::isfinite(projected_gradient_norm);
        if (!exact) break;

        std::vector<Vec3> joint_input(6000U);
        for (std::size_t particle = 0U; particle < 6000U; ++particle)
            joint_input[particle] = capture.target[particle]
                - at_lambda[particle];
        const ALR48Projection projection = al_r48_project_ball_box(
            joint_input, lower, upper, AL_R62_NEXT_RADIUS);
        ++projection_calls;
        exact = projection.passed;
        if (!exact) break;
        composite = projection.value;
        for (std::size_t particle = 0U; particle < 6000U; ++particle)
            correction[particle] = joint_input[particle] - composite[particle];
        std::vector<Vec3> committed_endpoint(6000U);
        for (std::size_t particle = 0U; particle < 6000U; ++particle)
            committed_endpoint[particle] = r43.projected_trial[particle]
                + SPACING * composite[particle];
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
                    + std::abs(committed_inertia)) : 0.0;
        const bool outer_model_prediction = !block.line_exact
            || (std::isfinite(model_prediction_gap)
                && std::isfinite(model_prediction_bound)
                && model_prediction_gap <= model_prediction_bound);
        model_prediction_exact = model_prediction_exact
            && outer_model_prediction && committed_model_reduction > 0.0;

        audit = al_r61_topology_owned_audit(workspace, composite);
        ++audit_calls;
        exact = audit.owner_exact && al_r48_vec_finite(correction);
        if (!exact) break;
        const ALR65PComposedDual committed_dual = al_r65p_composed_dual(
            composite, capture.target, joint_input, lambda,
            workspace.support.constraint, audit.raw, committed_inertia);
        const double predicted_composed_dual = block.line_exact
            ? block.candidate_composed_dual
            : block.baseline_composed_dual;
        const double dual_prediction_gap = std::abs(
            predicted_composed_dual - committed_dual.direct);
        const double dual_prediction_bound = gamma_factor(
            256U * 6000U + 1024U)
            * (std::abs(predicted_composed_dual)
                + std::abs(committed_dual.direct)
                + std::abs(committed_dual.completed_square));
        const bool outer_dual_prediction = committed_dual.passed
            && std::isfinite(dual_prediction_gap)
            && std::isfinite(dual_prediction_bound)
            && dual_prediction_gap <= dual_prediction_bound
            && (!block.line_exact || block.composed_dual_change > 0.0);
        dual_prediction_exact = dual_prediction_exact
            && outer_dual_prediction;
        exact = exact && outer_dual_prediction && outer_model_prediction;
        if (!exact) break;

        std::size_t additions = 0U;
        for (std::size_t row = 0U; row < 6000U; ++row)
            if (audit.candidate_upper[row] > 0.0 && owned[row] == 0U) {
                owned[row] = 1U;
                ++additions;
            }
        total_additions += additions;
        if (!al_r65p_checkpoint(outer)) continue;

        ALR65DRecord record;
        record.outer = outer;
        record.working = static_cast<std::size_t>(std::count(
            owned.begin(), owned.end(), static_cast<std::uint8_t>(1U)));
        record.additions = additions;
        record.face = face_count;
        record.pcg_iterations = block.face_iterations;
        record.candidate_trials = block.candidate_trials;
        record.candidate_projections = block.candidate_projection_calls;
        record.normal_rejections = block.model_rejections;
        record.dual_rejections = block.composed_rejections;
        record.admissible_candidates = block.admissible_candidates;
        record.selected_exponent = block.selected_exponent;
        record.applied_zeros = block.applied_zeros;
        record.line_alpha = block.line_alpha;
        record.maximum_raw = *std::max_element(
            audit.raw.begin(), audit.raw.end());
        record.projected_gradient_norm = projected_gradient_norm;
        record.baseline_composed_dual = block.baseline_composed_dual;
        record.predicted_composed_dual = predicted_composed_dual;
        record.committed_composed_dual = committed_dual.direct;
        record.composed_dual_change = block.composed_dual_change;
        record.dual_prediction_gap = dual_prediction_gap;
        record.dual_prediction_bound = dual_prediction_bound;
        record.model_reduction = committed_model_reduction;
        record.model_prediction_gap = model_prediction_gap;
        record.model_prediction_bound = model_prediction_bound;
        record.recurrence_gap = recurrence_gap;
        record.recurrence_bound = recurrence_bound;
        std::vector<Vec3> stationarity(6000U);
        for (std::size_t particle = 0U; particle < 6000U; ++particle)
            stationarity[particle] = composite[particle]
                - capture.target[particle] + at_lambda[particle]
                + correction[particle];
        record.stationarity_norm = al_r32_vec3_norm(stationarity);
        for (std::size_t row = 0U; row < 6000U; ++row)
            record.complementarity_maximum = std::max(
                record.complementarity_maximum,
                std::abs(lambda[row] * audit.raw[row]));
        std::vector<Vec3> reprojection_input(6000U);
        for (std::size_t particle = 0U; particle < 6000U; ++particle)
            reprojection_input[particle] = composite[particle]
                + correction[particle];
        const ALR48Projection reprojection = al_r48_project_ball_box(
            reprojection_input, lower, upper, AL_R62_NEXT_RADIUS);
        ++checkpoint_reprojections;
        record.reprojection_exact = reprojection.passed
            && exact_vec3_values(reprojection.value, composite);
        record.line_accepted = block.line_exact;
        record.dual_prediction_exact = outer_dual_prediction;
        record.model_prediction_exact = outer_model_prediction;
        record.batch_root = block.batch_root;
        std::ostringstream material;
        material << std::setprecision(17) << record.outer << ':'
            << record.working << ':' << record.additions << ':' << record.face
            << ':' << record.pcg_iterations << ':' << record.candidate_trials
            << ':' << record.candidate_projections << ':'
            << record.normal_rejections << ':' << record.dual_rejections << ':'
            << record.admissible_candidates << ':' << record.selected_exponent
            << ':' << record.applied_zeros << ':' << record.line_alpha << ':'
            << record.maximum_raw << ':' << record.projected_gradient_norm
            << ':' << record.baseline_composed_dual << ':'
            << record.predicted_composed_dual << ':'
            << record.committed_composed_dual << ':'
            << record.composed_dual_change << ':'
            << record.dual_prediction_gap << ':'
            << record.dual_prediction_bound << ':' << record.model_reduction
            << ':' << record.model_prediction_gap << ':'
            << record.model_prediction_bound << ':' << record.recurrence_gap
            << ':' << record.recurrence_bound << ':'
            << record.stationarity_norm << ':'
            << record.complementarity_maximum << ':'
            << record.line_accepted << ':' << record.dual_prediction_exact
            << ':' << record.model_prediction_exact << ':'
            << record.reprojection_exact << ':' << record.batch_root << ':'
            << al_binary64_vec3_root(composite) << ':'
            << al_r10_dual_root(lambda);
        record.root = sha256_hex(material.str());
        exact = exact && record.reprojection_exact
            && std::isfinite(record.maximum_raw)
            && std::isfinite(record.projected_gradient_norm)
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
        && candidate_fixed_dual_rejections + candidate_projection_calls
            == candidate_trials
        && candidate_normal_rejections + candidate_composed_dual_rejections
                + admissible_candidates
            == candidate_projection_calls
        && accepted_lines <= AL_R65P_OUTERS
        && transpose_calls == 2U * AL_R65P_OUTERS
            + face_pcg_iterations + candidate_trials
        && action_calls == 2U * AL_R65P_OUTERS + face_pcg_iterations
        && projection_calls == AL_R65P_OUTERS
        && audit_calls == AL_R65P_OUTERS + 1U
        && checkpoint_reprojections == AL_R65P_CHECKPOINTS.size()
        && records.size() == AL_R65P_CHECKPOINTS.size()
        && composed_dual_vector_terms
            == (baseline_projection_calls + candidate_projection_calls
                - candidate_normal_rejections) * 6000U
        && total_additions <= 6000U && lower_work;
    const bool rollback = trial_before
            == al_binary64_vec3_root(r43.projected_trial)
        && target_before == al_binary64_vec3_root(capture.target);
    const bool controls = al_r65d_dense_controls()
        && al_r65d_route_controls();
    const bool passed = exact && parent_exact && workspace_exact
        && operator_exact && geometry_exact && curvature_exact && line_exact
        && accepted_lines == AL_R65P_OUTERS && dual_prediction_exact
        && model_prediction_exact && recurrence_exact && work_exact
        && rollback && controls;
    const std::string route = al_r65d_route(passed, line_exact,
        dual_prediction_exact, model_prediction_exact, lower_work, dominates);

    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
        << parent_capture.result_sha << '|' << route << '|'
        << initial_working << ':' << total_additions << ':'
        << identification_sweeps << ':' << coordinate_visits << ':'
        << coordinate_updates << ':' << coordinate_residual_terms << ':'
        << coordinate_update_terms << ':' << face_pcg_iterations << ':'
        << candidate_trials << ':' << baseline_projection_calls << ':'
        << candidate_projection_calls << ':'
        << candidate_fixed_dual_rejections << ':'
        << candidate_normal_rejections << ':'
        << candidate_composed_dual_rejections << ':'
        << admissible_candidates << ':' << accepted_lines << ':'
        << transpose_calls << ':' << action_calls << ':' << audit_calls << ':'
        << structural_terms << ':' << composed_dual_vector_terms << '|';
    for (const ALR65DRecord& record : records) semantic << record.root << ':';
    semantic << "|EXPLORATORY_ONLY_NO_CREDIT";
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r65_best_composed_dual_path.v10\",\"status\":\""
        << (passed ? "PASS" : "FAIL") << "\",\"authority\":\"EXPLORATORY_ONLY_NO_CREDIT\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_result_sha256\":\""
        << parent_capture.result_sha << "\""
        << ",\"records\":[";
    for (std::size_t index_record = 0U; index_record < records.size();
         ++index_record) {
        if (index_record != 0U) report << ',';
        const ALR65DRecord& record = records[index_record];
        report << "{\"outer\":" << record.outer
            << ",\"working\":" << record.working
            << ",\"additions\":" << record.additions
            << ",\"face\":" << record.face
            << ",\"pcg_iterations\":" << record.pcg_iterations
            << ",\"candidate_trials\":" << record.candidate_trials
            << ",\"candidate_projections\":"
            << record.candidate_projections
            << ",\"normal_rejections\":" << record.normal_rejections
            << ",\"dual_rejections\":" << record.dual_rejections
            << ",\"admissible_candidates\":"
            << record.admissible_candidates
            << ",\"selected_exponent\":" << record.selected_exponent
            << ",\"applied_zeros\":" << record.applied_zeros
            << ",\"line_alpha\":" << record.line_alpha
            << ",\"maximum_raw\":" << record.maximum_raw
            << ",\"projected_gradient_norm\":"
            << record.projected_gradient_norm
            << ",\"baseline_composed_dual\":"
            << record.baseline_composed_dual
            << ",\"predicted_composed_dual\":"
            << record.predicted_composed_dual
            << ",\"committed_composed_dual\":"
            << record.committed_composed_dual
            << ",\"composed_dual_change\":"
            << record.composed_dual_change
            << ",\"dual_prediction_gap\":"
            << record.dual_prediction_gap
            << ",\"dual_prediction_bound\":"
            << record.dual_prediction_bound
            << ",\"model_reduction\":" << record.model_reduction
            << ",\"model_prediction_gap\":"
            << record.model_prediction_gap
            << ",\"model_prediction_bound\":"
            << record.model_prediction_bound
            << ",\"recurrence_gap\":" << record.recurrence_gap
            << ",\"recurrence_bound\":" << record.recurrence_bound
            << ",\"stationarity_norm\":" << record.stationarity_norm
            << ",\"complementarity_maximum\":"
            << record.complementarity_maximum
            << ",\"line_accepted\":"
            << (record.line_accepted ? "true" : "false")
            << ",\"dual_prediction_exact\":"
            << (record.dual_prediction_exact ? "true" : "false")
            << ",\"model_prediction_exact\":"
            << (record.model_prediction_exact ? "true" : "false")
            << ",\"reprojection_exact\":"
            << (record.reprojection_exact ? "true" : "false")
            << ",\"batch_root\":\"" << record.batch_root << "\""
            << ",\"root\":\"" << record.root << "\"}";
    }
    report << "],\"work\":{\"outer_blocks\":" << identification_sweeps
        << ",\"coordinate_visits\":" << coordinate_visits
        << ",\"coordinate_updates\":" << coordinate_updates
        << ",\"coordinate_residual_terms\":"
        << coordinate_residual_terms
        << ",\"coordinate_update_terms\":" << coordinate_update_terms
        << ",\"face_pcg_iterations\":" << face_pcg_iterations
        << ",\"candidate_trials\":" << candidate_trials
        << ",\"baseline_joint_projections\":"
        << baseline_projection_calls
        << ",\"candidate_joint_projections\":"
        << candidate_projection_calls
        << ",\"candidate_fixed_dual_rejections\":"
        << candidate_fixed_dual_rejections
        << ",\"candidate_normal_rejections\":"
        << candidate_normal_rejections
        << ",\"candidate_composed_dual_rejections\":"
        << candidate_composed_dual_rejections
        << ",\"admissible_candidates\":" << admissible_candidates
        << ",\"accepted_lines\":" << accepted_lines
        << ",\"transpose_calls\":" << transpose_calls
        << ",\"action_calls\":" << action_calls
        << ",\"all_row_audits\":" << audit_calls
        << ",\"checkpoint_reprojections\":"
        << checkpoint_reprojections
        << ",\"completed_dual_vector_terms\":"
        << composed_dual_vector_terms
        << ",\"structural_terms\":" << structural_terms
        << ",\"fista_structural_terms\":"
        << AL_R65P_FISTA_STRUCTURAL_TERMS
        << ",\"lower_than_fista\":"
        << (lower_work ? "true" : "false")
        << ",\"exact\":" << (work_exact ? "true" : "false") << "}"
        << ",\"dominates_fista\":" << (dominates ? "true" : "false")
        << ",\"curvature_exact\":"
        << (curvature_exact ? "true" : "false")
        << ",\"line_exact\":" << (line_exact ? "true" : "false")
        << ",\"dual_prediction_exact\":"
        << (dual_prediction_exact ? "true" : "false")
        << ",\"model_prediction_exact\":"
        << (model_prediction_exact ? "true" : "false")
        << ",\"recurrence_exact\":"
        << (recurrence_exact ? "true" : "false")
        << ",\"rollback_exact\":" << (rollback ? "true" : "false")
        << ",\"controls_exact\":" << (controls ? "true" : "false")
        << ",\"nonlinear_trial_executed\":false"
           ",\"runtime_state_mutated\":false,\"timing_admitted\":false"
           ",\"production_authority\":false,\"result_sha256\":\""
        << result_sha << "\"}";
    return {passed, report.str()};
}
