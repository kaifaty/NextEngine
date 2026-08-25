namespace {

constexpr std::size_t AL_R65F_OUTERS = 16U;
constexpr std::size_t AL_R65F_INNERS = 16U;
constexpr std::array<std::size_t, 5U> AL_R65F_CHECKPOINTS{{1U, 2U, 4U, 8U, 16U}};

bool al_r65f_checkpoint(std::size_t outer) {
    return std::find(AL_R65F_CHECKPOINTS.begin(),
        AL_R65F_CHECKPOINTS.end(), outer) != AL_R65F_CHECKPOINTS.end();
}

double al_r65f_vec_squared(const std::vector<Vec3>& values) {
    long double result = 0.0L;
    for (Vec3 value : values)
        result += static_cast<long double>(value.x) * value.x
            + static_cast<long double>(value.y) * value.y
            + static_cast<long double>(value.z) * value.z;
    return static_cast<double>(result);
}

double al_r65f_dual_objective(const std::vector<Vec3>& at_lambda,
    const std::vector<double>& b, const std::vector<double>& lambda,
    const std::vector<std::uint8_t>& owned) {
    long double result = 0.5L * al_r65f_vec_squared(at_lambda);
    for (std::size_t row = 0U; row < lambda.size(); ++row)
        if (owned[row] != 0U)
            result -= static_cast<long double>(b[row]) * lambda[row];
    return static_cast<double>(result);
}

struct ALR65FEvaluation {
    bool passed = false;
    std::vector<Vec3> at_lambda;
    std::vector<double> gradient;
    double objective = std::numeric_limits<double>::infinity();
};

ALR65FEvaluation al_r65f_evaluate(const ALR64Operator& op,
    const std::vector<double>& b, const std::vector<double>& lambda,
    const std::vector<std::uint8_t>& owned) {
    ALR65FEvaluation result;
    result.at_lambda = al_r64_transpose(op, lambda, nullptr);
    if (result.at_lambda.size() != lambda.size()) return result;
    const ALLinearizedJvp image = al_r64_action(op, result.at_lambda);
    if (!image.passed || image.value.size() != lambda.size()) return result;
    result.gradient.resize(lambda.size());
    for (std::size_t row = 0U; row < lambda.size(); ++row)
        result.gradient[row] = owned[row] != 0U
            ? image.value[row] - b[row] : 0.0;
    result.objective = al_r65f_dual_objective(
        result.at_lambda, b, lambda, owned);
    result.passed = al_r48_vec_finite(result.at_lambda)
        && al_r30_finite(result.gradient) && std::isfinite(result.objective);
    return result;
}

struct ALR65FBlock {
    bool passed = false;
    std::vector<double> lambda;
    std::vector<Vec3> at_lambda;
    double objective = std::numeric_limits<double>::infinity();
    double projected_gradient_norm = std::numeric_limits<double>::infinity();
    double accepted_lipschitz = 0.0;
    std::size_t iterations = 0U;
    std::size_t backtracks = 0U;
    std::size_t restarts = 0U;
    std::size_t transpose_calls = 0U;
    std::size_t action_calls = 0U;
};

ALR65FBlock al_r65f_block(const ALR64Operator& op,
    const std::vector<double>& b, const std::vector<std::uint8_t>& owned,
    const std::vector<double>& initial, double initial_lipschitz) {
    ALR65FBlock result;
    const std::size_t count = initial.size();
    if (count == 0U || b.size() != count || owned.size() != count
        || !std::isfinite(initial_lipschitz) || initial_lipschitz <= 0.0)
        return result;
    std::vector<double> x = initial;
    std::vector<double> y = initial;
    double momentum = 1.0;
    double lipschitz = initial_lipschitz;
    bool exact = al_r30_finite(x);
    for (std::size_t iteration = 0U;
         iteration < AL_R65F_INNERS && exact; ++iteration) {
        const ALR65FEvaluation evaluation =
            al_r65f_evaluate(op, b, y, owned);
        ++result.transpose_calls;
        ++result.action_calls;
        exact = evaluation.passed;
        if (!exact) break;
        double trial_lipschitz = iteration == 0U
            ? lipschitz : std::max(0.5 * lipschitz,
                std::numeric_limits<double>::min());
        std::vector<double> candidate(count);
        std::vector<Vec3> candidate_at;
        double candidate_objective = std::numeric_limits<double>::infinity();
        bool accepted = false;
        for (std::size_t backtrack = 0U;
             backtrack < 64U && !accepted; ++backtrack) {
            long double linear = 0.0L;
            long double squared = 0.0L;
            for (std::size_t row = 0U; row < count; ++row) {
                candidate[row] = owned[row] != 0U
                    ? std::max(0.0,
                        y[row] - evaluation.gradient[row] / trial_lipschitz)
                    : 0.0;
                const double difference = candidate[row] - y[row];
                linear += static_cast<long double>(evaluation.gradient[row])
                    * difference;
                squared += static_cast<long double>(difference) * difference;
            }
            candidate_at = al_r64_transpose(op, candidate, nullptr);
            ++result.transpose_calls;
            if (candidate_at.size() == count)
                candidate_objective = al_r65f_dual_objective(
                    candidate_at, b, candidate, owned);
            const long double model = evaluation.objective + linear
                + 0.5L * trial_lipschitz * squared;
            const double bound = gamma_factor(64U * count + 256U)
                * (std::abs(candidate_objective)
                    + std::abs(static_cast<double>(model)));
            accepted = candidate_at.size() == count
                && al_r48_vec_finite(candidate_at)
                && std::isfinite(candidate_objective)
                && std::isfinite(static_cast<double>(model))
                && candidate_objective <= static_cast<double>(model) + bound;
            ++result.backtracks;
            if (!accepted) trial_lipschitz *= 2.0;
            exact = exact && std::isfinite(trial_lipschitz)
                && trial_lipschitz > 0.0;
            if (!exact) break;
        }
        if (!exact || !accepted) {
            exact = false;
            break;
        }
        lipschitz = trial_lipschitz;
        const double next_momentum = 0.5
            * (1.0 + std::sqrt(1.0 + 4.0 * momentum * momentum));
        const double beta = (momentum - 1.0) / next_momentum;
        std::vector<double> next_y(count);
        long double restart_dot = 0.0L;
        for (std::size_t row = 0U; row < count; ++row) {
            next_y[row] = owned[row] != 0U
                ? candidate[row] + beta * (candidate[row] - x[row]) : 0.0;
            restart_dot += static_cast<long double>(y[row] - candidate[row])
                * (candidate[row] - x[row]);
        }
        if (restart_dot > 0.0L) {
            next_y = candidate;
            momentum = 1.0;
            ++result.restarts;
        } else {
            momentum = next_momentum;
        }
        x = std::move(candidate);
        y = std::move(next_y);
        result.at_lambda = std::move(candidate_at);
        result.objective = candidate_objective;
        result.iterations = iteration + 1U;
        exact = exact && al_r30_finite(x) && al_r30_finite(y)
            && std::isfinite(momentum) && momentum >= 1.0;
    }
    if (!exact || result.iterations != AL_R65F_INNERS) return result;
    const ALR65FEvaluation terminal = al_r65f_evaluate(op, b, x, owned);
    ++result.transpose_calls;
    ++result.action_calls;
    if (!terminal.passed) return result;
    long double projected_squared = 0.0L;
    for (std::size_t row = 0U; row < count; ++row) {
        const double projected = owned[row] != 0U
            ? std::max(0.0, x[row] - terminal.gradient[row] / lipschitz) : 0.0;
        const double mapping = lipschitz * (x[row] - projected);
        projected_squared += static_cast<long double>(mapping) * mapping;
    }
    result.lambda = std::move(x);
    result.at_lambda = terminal.at_lambda;
    result.objective = terminal.objective;
    result.projected_gradient_norm =
        std::sqrt(static_cast<double>(projected_squared));
    result.accepted_lipschitz = lipschitz;
    result.passed = al_r30_finite(result.lambda)
        && al_r48_vec_finite(result.at_lambda)
        && std::isfinite(result.projected_gradient_norm)
        && std::isfinite(result.objective);
    return result;
}

struct ALR65FRecord {
    std::size_t outer = 0U;
    std::size_t working = 0U;
    std::size_t additions = 0U;
    std::size_t raw_positive = 0U;
    std::size_t candidate_positive = 0U;
    std::size_t nonzero_lambda = 0U;
    double maximum_raw = 0.0;
    double objective = 0.0;
    double projected_gradient_norm = 0.0;
    double stationarity_norm = 0.0;
    double complementarity_maximum = 0.0;
    double lipschitz = 0.0;
    double inertia = 0.0;
    double model_reduction = 0.0;
    bool reprojection_exact = false;
    std::string root;
};

} // namespace

SplitBoundaryReport run_al_active_face_fista_probe_controls() {
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
        && op.entries.size() == 535588U;

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
    double lipschitz = 0.0;
    for (std::size_t row = 0U; row < 6000U; ++row)
        if (owned[row] != 0U)
            lipschitz = std::max(lipschitz, al_r65_sparse_diagonal(op, row));
    bool exact = geometry_exact && audit.owner_exact && initial_working > 0U
        && std::isfinite(lipschitz) && lipschitz > 0.0;
    std::size_t total_additions = initial_working;
    std::size_t total_iterations = 0U;
    std::size_t total_backtracks = 0U;
    std::size_t total_restarts = 0U;
    std::size_t transpose_calls = 0U;
    std::size_t action_calls = 0U;
    std::size_t projection_calls = 0U;
    std::size_t audit_calls = 1U;
    std::vector<ALR65FRecord> records;

    for (std::size_t outer = 1U; outer <= AL_R65F_OUTERS && exact; ++outer) {
        std::vector<Vec3> z(6000U);
        for (std::size_t i = 0U; i < 6000U; ++i)
            z[i] = capture.target[i] - correction[i];
        const ALLinearizedJvp az = al_r64_action(op, z);
        ++action_calls;
        std::vector<double> b(6000U);
        exact = az.passed;
        for (std::size_t row = 0U; row < 6000U && exact; ++row)
            b[row] = workspace.support.constraint[row] + az.value[row];
        if (!exact) break;
        const ALR65FBlock block =
            al_r65f_block(op, b, owned, lambda, lipschitz);
        total_iterations += block.iterations;
        total_backtracks += block.backtracks;
        total_restarts += block.restarts;
        transpose_calls += block.transpose_calls;
        action_calls += block.action_calls;
        exact = block.passed;
        if (!exact) break;
        lambda = block.lambda;
        lipschitz = block.accepted_lipschitz;
        std::vector<Vec3> projection_input(6000U);
        for (std::size_t i = 0U; i < 6000U; ++i)
            projection_input[i] = capture.target[i] - block.at_lambda[i];
        const ALR48Projection projection = al_r48_project_ball_box(
            projection_input, lower, upper, AL_R62_NEXT_RADIUS);
        ++projection_calls;
        exact = projection.passed;
        if (!exact) break;
        composite = projection.value;
        for (std::size_t i = 0U; i < 6000U; ++i)
            correction[i] = projection_input[i] - composite[i];
        audit = al_r61_topology_owned_audit(workspace, composite);
        ++audit_calls;
        exact = audit.owner_exact && al_r48_vec_finite(correction);
        std::size_t additions = 0U;
        for (std::size_t row = 0U; row < 6000U; ++row)
            if (audit.candidate_upper[row] > 0.0 && owned[row] == 0U) {
                owned[row] = 1U;
                ++additions;
                lipschitz = std::max(
                    lipschitz, al_r65_sparse_diagonal(op, row));
            }
        total_additions += additions;
        if (!al_r65f_checkpoint(outer)) continue;

        ALR65FRecord record;
        record.outer = outer;
        record.working = static_cast<std::size_t>(std::count(
            owned.begin(), owned.end(), static_cast<std::uint8_t>(1U)));
        record.additions = additions;
        record.raw_positive = audit.raw_positive;
        record.candidate_positive = audit.candidate_positive;
        record.nonzero_lambda = static_cast<std::size_t>(std::count_if(
            lambda.begin(), lambda.end(),
            [](double value) { return value > 0.0; }));
        record.maximum_raw = *std::max_element(
            audit.raw.begin(), audit.raw.end());
        record.objective = block.objective;
        record.projected_gradient_norm = block.projected_gradient_norm;
        record.lipschitz = lipschitz;
        std::vector<Vec3> stationarity(6000U);
        for (std::size_t i = 0U; i < 6000U; ++i)
            stationarity[i] = composite[i] - capture.target[i]
                + block.at_lambda[i] + correction[i];
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
        std::vector<Vec3> endpoint(6000U);
        for (std::size_t i = 0U; i < 6000U; ++i)
            endpoint[i] = r43.projected_trial[i] + SPACING * composite[i];
        record.inertia = al_r63_inertia(endpoint, r30.predicted_position);
        record.model_reduction = normal_inertia - record.inertia;
        std::ostringstream material;
        material << std::setprecision(17) << record.outer << ':'
            << record.working << ':' << record.additions << ':'
            << record.raw_positive << ':' << record.candidate_positive << ':'
            << record.nonzero_lambda << ':' << record.maximum_raw << ':'
            << record.objective << ':' << record.projected_gradient_norm << ':'
            << record.stationarity_norm << ':'
            << record.complementarity_maximum << ':' << record.lipschitz << ':'
            << record.inertia << ':' << record.model_reduction << ':'
            << record.reprojection_exact << ':'
            << al_binary64_vec3_root(composite) << ':'
            << al_r10_dual_root(lambda);
        record.root = sha256_hex(material.str());
        exact = exact && record.reprojection_exact
            && std::isfinite(record.maximum_raw)
            && std::isfinite(record.projected_gradient_norm)
            && std::isfinite(record.stationarity_norm)
            && std::isfinite(record.model_reduction);
        records.push_back(record);
    }

    if (workspace.passed)
        release_al_normalized_workspace(workspace, &workspace_work);
    const bool work_exact = workspace_work.workspace_builds == 1U
        && workspace_work.workspace_releases == 1U
        && total_iterations == AL_R65F_OUTERS * AL_R65F_INNERS
        && projection_calls == AL_R65F_OUTERS
        && audit_calls == AL_R65F_OUTERS + 1U
        && records.size() == AL_R65F_CHECKPOINTS.size()
        && transpose_calls > total_iterations
        && action_calls == AL_R65F_OUTERS * (AL_R65F_INNERS + 2U);
    const bool rollback = trial_before
            == al_binary64_vec3_root(r43.projected_trial)
        && target_before == al_binary64_vec3_root(capture.target);
    const bool passed = exact && parent_exact && workspace_exact
        && operator_exact && geometry_exact && work_exact && rollback;

    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
        << parent_stdout << '|' << initial_working << ':' << total_additions
        << ':' << total_iterations << ':' << total_backtracks << ':'
        << total_restarts << ':' << transpose_calls << ':' << action_calls << '|';
    for (const ALR65FRecord& record : records) semantic << record.root << ':';
    semantic << "|EXPLORATORY_ONLY_NO_CREDIT";
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r65_active_face_fista_probe.v1\",\"status\":\""
        << (passed ? "PASS" : "FAIL")
        << "\",\"authority\":\"EXPLORATORY_ONLY_NO_CREDIT\""
        << ",\"parent_stdout_sha256\":\"" << parent_stdout
        << "\",\"initial_working\":" << initial_working
        << ",\"records\":[";
    for (std::size_t i = 0U; i < records.size(); ++i) {
        if (i != 0U) report << ',';
        const ALR65FRecord& record = records[i];
        report << "{\"outer\":" << record.outer
            << ",\"working\":" << record.working
            << ",\"additions\":" << record.additions
            << ",\"raw_positive\":" << record.raw_positive
            << ",\"candidate_positive\":" << record.candidate_positive
            << ",\"nonzero_lambda\":" << record.nonzero_lambda
            << ",\"maximum_raw\":" << record.maximum_raw
            << ",\"dual_objective\":" << record.objective
            << ",\"projected_gradient_norm\":"
            << record.projected_gradient_norm
            << ",\"stationarity_norm\":" << record.stationarity_norm
            << ",\"complementarity_maximum\":"
            << record.complementarity_maximum
            << ",\"lipschitz\":" << record.lipschitz
            << ",\"inertia\":" << record.inertia
            << ",\"model_reduction\":" << record.model_reduction
            << ",\"reprojection_exact\":"
            << (record.reprojection_exact ? "true" : "false")
            << ",\"root\":\"" << record.root << "\"}";
    }
    report << "],\"work\":{\"outer_blocks\":" << AL_R65F_OUTERS
        << ",\"fista_iterations\":" << total_iterations
        << ",\"backtracks\":" << total_backtracks
        << ",\"restarts\":" << total_restarts
        << ",\"transpose_calls\":" << transpose_calls
        << ",\"action_calls\":" << action_calls
        << ",\"joint_projections\":" << projection_calls
        << ",\"all_row_audits\":" << audit_calls
        << ",\"total_additions\":" << total_additions
        << ",\"dense_gram_storage\":0,\"exact\":"
        << (work_exact ? "true" : "false") << "}"
        << ",\"rollback_exact\":" << (rollback ? "true" : "false")
        << ",\"nonlinear_trial_executed\":false"
           ",\"runtime_state_mutated\":false,\"timing_admitted\":false"
           ",\"production_authority\":false,\"result_sha256\":\""
        << result_sha << "\"}";
    return {passed, report.str()};
}
