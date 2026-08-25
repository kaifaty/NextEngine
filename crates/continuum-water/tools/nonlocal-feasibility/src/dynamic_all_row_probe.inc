namespace {

constexpr std::array<std::size_t, 9U> AL_R65_PROBE_CHECKPOINTS{{
    8U, 16U, 32U, 64U, 128U, 256U, 512U, 1024U, 2048U}};
constexpr double AL_R65_PROBE_RELAXATION = 1.5;

bool al_r65_probe_checkpoint(std::size_t cycle) {
    return std::find(AL_R65_PROBE_CHECKPOINTS.begin(),
        AL_R65_PROBE_CHECKPOINTS.end(), cycle)
        != AL_R65_PROBE_CHECKPOINTS.end();
}

struct ALR65ProbeRecord {
    std::size_t cycle = 0U;
    std::size_t working = 0U;
    std::size_t additions = 0U;
    std::size_t raw_positive = 0U;
    std::size_t candidate_positive = 0U;
    std::size_t positive_outside = 0U;
    std::size_t nonzero_lambda = 0U;
    double maximum_raw = 0.0;
    double stationarity_norm = 0.0;
    double complementarity_maximum = 0.0;
    double composite_norm = 0.0;
    double correction_norm = 0.0;
    double inertia = 0.0;
    double model_reduction = 0.0;
    bool reprojection_exact = false;
    std::string composite_root;
    std::string lambda_root;
    std::string root;
};

double al_r65_sparse_residual(const ALNormalizedWorkspace& workspace,
    const ALR64Operator& op, std::size_t row,
    const std::vector<Vec3>& composite) {
    double result = workspace.support.constraint[row];
    for (std::size_t index = op.entry_offsets[row];
         index < op.entry_offsets[row + 1U]; ++index) {
        const ALR64Entry& entry = op.entries[index];
        result += dot(entry.coefficient, composite[entry.particle]);
    }
    return result;
}

double al_r65_sparse_diagonal(
    const ALR64Operator& op, std::size_t row) {
    double result = 0.0;
    for (std::size_t index = op.entry_offsets[row];
         index < op.entry_offsets[row + 1U]; ++index)
        result += norm_squared(op.entries[index].coefficient);
    return result;
}

} // namespace

SplitBoundaryReport run_al_dynamic_all_row_probe_controls() {
    ALR63ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_cached_normal_projection_trqp_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "38298214e352a87cfffb5c5432be90ef822c3f8ab00da5a2f7a3b1cb64565b62"
        && capture.result.state_root
            == "216c12aeeb47686e823a9911f0aa9dd14ddee53db1701874488dc325a292cea5";

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

    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string trial_before = al_binary64_vec3_root(r43.projected_trial);
    const std::string target_before = al_binary64_vec3_root(capture.target);
    const std::string normal_before =
        al_binary64_vec3_root(r62.publication.cached_normal);

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
    const bool workspace_exact = parent_exact && index.passed && binding.passed
        && workspace.passed && workspace.neighborhood.fluid.size() == 6000U
        && al_r42_superset_root(workspace.neighborhood)
            == r62.payload.workspace_topology_root;
    ALR64Operator op;
    if (workspace.passed) op = al_r64_build(workspace);
    const ALR64StructureAudit structure = al_r64_structure(workspace, op);
    const bool operator_exact = workspace_exact && structure.slot
        && structure.entry && structure.incidence
        && op.slots.size() == 605144U && op.entries.size() == 535588U;

    std::vector<Vec3> lower(6000U);
    std::vector<Vec3> upper(6000U);
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
    std::vector<Vec3> set_correction(6000U);
    std::vector<double> lambda(6000U, 0.0);
    std::vector<std::uint8_t> owned(6000U, 0U);
    ALR61Audit audit;
    if (geometry_exact)
        audit = al_r61_topology_owned_audit(workspace, composite);
    std::size_t initial_working = 0U;
    if (audit.owner_exact)
        for (std::size_t row = 0U; row < 6000U; ++row)
            if (audit.candidate_upper[row] > 0.0) {
                owned[row] = 1U;
                ++initial_working;
            }

    bool exact = geometry_exact && audit.owner_exact && initial_working > 0U;
    std::size_t coordinate_visits = 0U;
    std::size_t coordinate_updates = 0U;
    std::size_t row_entry_terms = 0U;
    std::size_t projection_calls = 0U;
    std::size_t projection_scans = 0U;
    std::size_t reprojection_calls = 0U;
    std::size_t audit_calls = 1U;
    std::size_t total_additions = initial_working;
    std::vector<ALR65ProbeRecord> records;

    for (std::size_t cycle = 1U; cycle <= 2048U && exact; ++cycle) {
        for (std::size_t row = 0U; row < 6000U; ++row) {
            if (owned[row] == 0U) continue;
            ++coordinate_visits;
            const std::size_t degree = op.entry_offsets[row + 1U]
                - op.entry_offsets[row];
            row_entry_terms += degree;
            const double residual =
                al_r65_sparse_residual(workspace, op, row, composite);
            const double diagonal = al_r65_sparse_diagonal(op, row);
            const double next = diagonal > 0.0
                ? std::max(0.0, lambda[row]
                    + AL_R65_PROBE_RELAXATION * residual / diagonal)
                : std::numeric_limits<double>::infinity();
            const double delta = next - lambda[row];
            exact = exact && std::isfinite(residual)
                && std::isfinite(diagonal) && diagonal > 0.0
                && std::isfinite(next) && std::isfinite(delta);
            if (!exact) break;
            lambda[row] = next;
            if (delta == 0.0) continue;
            ++coordinate_updates;
            for (std::size_t entry_index = op.entry_offsets[row];
                 entry_index < op.entry_offsets[row + 1U]; ++entry_index) {
                const ALR64Entry& entry = op.entries[entry_index];
                composite[entry.particle] = composite[entry.particle]
                    - delta * entry.coefficient;
            }
        }
        exact = exact && al_r48_vec_finite(composite)
            && al_r30_finite(lambda);
        if (!exact) break;

        std::vector<Vec3> projected_input(6000U);
        for (std::size_t i = 0U; i < 6000U; ++i)
            projected_input[i] = composite[i] + set_correction[i];
        const ALR48Projection projection = al_r48_project_ball_box(
            projected_input, lower, upper, AL_R62_NEXT_RADIUS);
        ++projection_calls;
        projection_scans += projection.scans;
        exact = exact && projection.passed;
        if (!exact) break;
        composite = projection.value;
        for (std::size_t i = 0U; i < 6000U; ++i)
            set_correction[i] = projected_input[i] - composite[i];
        exact = exact && al_r48_vec_finite(composite)
            && al_r48_vec_finite(set_correction)
            && al_r49_feasible(composite, lower, upper)
            && al_r32_vec3_norm(composite) <= AL_R62_NEXT_RADIUS;
        if (!exact) break;

        audit = al_r61_topology_owned_audit(workspace, composite);
        ++audit_calls;
        exact = exact && audit.owner_exact;
        if (!exact) break;
        std::size_t additions = 0U;
        std::size_t outside = 0U;
        for (std::size_t row = 0U; row < 6000U; ++row)
            if (audit.candidate_upper[row] > 0.0 && owned[row] == 0U) {
                ++outside;
                owned[row] = 1U;
                ++additions;
            }
        total_additions += additions;

        if (!al_r65_probe_checkpoint(cycle)) continue;
        ALR65ProbeRecord record;
        record.cycle = cycle;
        record.additions = additions;
        record.working = static_cast<std::size_t>(std::count(
            owned.begin(), owned.end(), static_cast<std::uint8_t>(1U)));
        record.raw_positive = audit.raw_positive;
        record.candidate_positive = audit.candidate_positive;
        record.positive_outside = outside;
        record.maximum_raw = *std::max_element(
            audit.raw.begin(), audit.raw.end());
        record.nonzero_lambda = static_cast<std::size_t>(std::count_if(
            lambda.begin(), lambda.end(),
            [](double value) { return value > 0.0; }));
        record.composite_norm = al_r32_vec3_norm(composite);
        record.correction_norm = al_r32_vec3_norm(set_correction);
        const std::vector<Vec3> at_lambda =
            al_r64_transpose(op, lambda, nullptr);
        std::vector<Vec3> stationarity(6000U);
        for (std::size_t i = 0U; i < 6000U; ++i)
            stationarity[i] = composite[i] - capture.target[i]
                + at_lambda[i] + set_correction[i];
        record.stationarity_norm = al_r32_vec3_norm(stationarity);
        for (std::size_t row = 0U; row < 6000U; ++row)
            record.complementarity_maximum = std::max(
                record.complementarity_maximum,
                std::abs(lambda[row] * audit.raw[row]));
        std::vector<Vec3> reprojection_input(6000U);
        for (std::size_t i = 0U; i < 6000U; ++i)
            reprojection_input[i] = composite[i] + set_correction[i];
        const ALR48Projection reprojection = al_r48_project_ball_box(
            reprojection_input, lower, upper, AL_R62_NEXT_RADIUS);
        ++reprojection_calls;
        record.reprojection_exact = reprojection.passed
            && exact_vec3_values(reprojection.value, composite);
        std::vector<Vec3> endpoint(6000U);
        for (std::size_t i = 0U; i < 6000U; ++i)
            endpoint[i] = r43.projected_trial[i] + SPACING * composite[i];
        record.inertia = al_r63_inertia(endpoint, r30.predicted_position);
        record.model_reduction = normal_inertia - record.inertia;
        record.composite_root = al_binary64_vec3_root(composite);
        record.lambda_root = al_r10_dual_root(lambda);
        std::ostringstream material;
        material << std::setprecision(17) << record.cycle << ':'
            << record.working << ':' << record.additions << ':'
            << record.raw_positive << ':' << record.candidate_positive << ':'
            << record.positive_outside << ':' << record.nonzero_lambda << ':'
            << record.maximum_raw << ':' << record.stationarity_norm << ':'
            << record.complementarity_maximum << ':' << record.composite_norm
            << ':' << record.correction_norm << ':' << record.inertia << ':'
            << record.model_reduction << ':' << record.reprojection_exact << ':'
            << record.composite_root << ':' << record.lambda_root;
        record.root = sha256_hex(material.str());
        exact = exact && std::isfinite(record.maximum_raw)
            && std::isfinite(record.stationarity_norm)
            && std::isfinite(record.complementarity_maximum)
            && std::isfinite(record.inertia)
            && std::isfinite(record.model_reduction)
            && record.reprojection_exact;
        records.push_back(std::move(record));
    }

    if (workspace.passed)
        release_al_normalized_workspace(workspace, &workspace_work);
    const bool work_exact = workspace_work.workspace_builds == 1U
        && workspace_work.workspace_releases == 1U
        && workspace_work.live_workspaces == 0U
        && static_work.static_index_builds == 1U
        && adjacency_work.workspace_builds == 1U
        && projection_calls == 2048U && audit_calls == 2049U
        && reprojection_calls == AL_R65_PROBE_CHECKPOINTS.size()
        && records.size() == AL_R65_PROBE_CHECKPOINTS.size();
    const bool rollback = position_before == al_binary64_vec3_root(r30.position)
        && trial_before == al_binary64_vec3_root(r43.projected_trial)
        && target_before == al_binary64_vec3_root(capture.target)
        && normal_before
            == al_binary64_vec3_root(r62.publication.cached_normal);
    const bool passed = exact && parent_exact && workspace_exact
        && operator_exact && geometry_exact && work_exact && rollback;

    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
        << parent_stdout << '|' << initial_working << ':' << total_additions
        << ':' << coordinate_visits << ':' << coordinate_updates << ':'
        << row_entry_terms << ':' << projection_scans << '|';
    for (const ALR65ProbeRecord& record : records)
        semantic << record.root << ':';
    semantic << '|' << work_exact << ':' << rollback
        << ":EXPLORATORY_ONLY_NO_CREDIT";
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r65_dynamic_all_row_probe.v1\",\"status\":\""
        << (passed ? "PASS" : "FAIL")
        << "\",\"authority\":\"EXPLORATORY_ONLY_NO_CREDIT\""
        << ",\"ordering\":\"ROW_ASCENDING\",\"relaxation\":"
        << AL_R65_PROBE_RELAXATION
        << ",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
        << ",\"initial_working\":" << initial_working
        << ",\"records\":[";
    for (std::size_t i = 0U; i < records.size(); ++i) {
        if (i != 0U) report << ',';
        const ALR65ProbeRecord& record = records[i];
        report << "{\"cycle\":" << record.cycle
            << ",\"working\":" << record.working
            << ",\"additions\":" << record.additions
            << ",\"raw_positive\":" << record.raw_positive
            << ",\"candidate_positive\":" << record.candidate_positive
            << ",\"positive_outside\":" << record.positive_outside
            << ",\"nonzero_lambda\":" << record.nonzero_lambda
            << ",\"maximum_raw\":" << record.maximum_raw
            << ",\"stationarity_norm\":" << record.stationarity_norm
            << ",\"complementarity_maximum\":"
            << record.complementarity_maximum
            << ",\"composite_norm\":" << record.composite_norm
            << ",\"correction_norm\":" << record.correction_norm
            << ",\"inertia\":" << record.inertia
            << ",\"model_reduction\":" << record.model_reduction
            << ",\"reprojection_exact\":"
            << (record.reprojection_exact ? "true" : "false")
            << ",\"composite_root\":\"" << record.composite_root
            << "\",\"lambda_root\":\"" << record.lambda_root
            << "\",\"root\":\"" << record.root << "\"}";
    }
    report << "],\"work\":{\"cycles\":2048,\"coordinate_visits\":"
        << coordinate_visits << ",\"coordinate_updates\":"
        << coordinate_updates << ",\"row_entry_terms\":" << row_entry_terms
        << ",\"projection_calls\":" << projection_calls
        << ",\"reprojection_audits\":" << reprojection_calls
        << ",\"projection_scans\":" << projection_scans
        << ",\"all_row_audits\":" << audit_calls
        << ",\"total_additions\":" << total_additions
        << ",\"dense_gram_storage\":0,\"gram_updates\":0,\"exact\":"
        << (work_exact ? "true" : "false") << "}"
        << ",\"rollback_exact\":" << (rollback ? "true" : "false")
        << ",\"nonlinear_trial_executed\":false"
           ",\"runtime_state_mutated\":false,\"timing_admitted\":false"
           ",\"production_authority\":false,\"result_sha256\":\""
        << result_sha << "\"}";
    return {passed, report.str()};
}
