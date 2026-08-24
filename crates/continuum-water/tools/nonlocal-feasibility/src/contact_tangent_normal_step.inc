namespace {

constexpr const char* B4E2D7R19R43_IDENTITY_SHA256 =
    "a6623368209ebc9cc08090ff40584fe0e16cc68d852ef2c7c1f1322f74b944fa";
constexpr const char* B4E2D7R19R43_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r43-contact-tangent-normal-step|v1|parent=11c970e4:f7ae3a0c60efce0a78d3d1821e51b12c97f2fb8557724e5f3082bf493bb45106:fed11eefb017fce471b41fa25a20f5db9ea93e563b90fe5d02123380d52784d2|source=r42-route-stable-superset-relinearization-candidate;position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;unprojected-trial-60e793aff919fada6b49c1b98be7ee5cc4a2c74a4254fa762cdece5700e1e804;endpoint-a73caf5baa6708ca89693077bad943dcdbf1f5069d50270ccc0f6b184a102e3a;physical-step-424c84b72409f5b6665874f17afc18cf2784ae84593e30be41eaeec0d25e514b;superset-355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01;contact-root-1df2083d4096522f329cdb1ff2a70c5da448ead0d40cda2cdb02e520c88f1ef7|projection=source-strict-positive-box-penetration-owns-tangent-halfspace;lower-active-dimensionless-component>=0;upper-active-component<=0;inadmissible-to-positive-zero;other-bits-retained;physical=spacing*projected-dimensionless;euclidean-norm-nonincreasing|contact=all-36000-source-projected-trial-face-tests;new0;no-face-penetration-increase;projection-owner-root;projected-ledger-root|coverage=one-source-anchored-r42-superset;exact-projected-r<=h-mask;certificate;canonical-correspondence;physical-h-unchanged|feasibility=source-linear-response-of-projected-step;predicted=source-psi-minus-linear-psi;actual=source-psi-minus-projected-trial-psi;both-positive;rho=actual/predicted;rho>=0.1|relinearization=fresh-projected-trial-radii-weight-first-constraint;zero-origin-mapping=AtrialT*max(cTrial,0);finite-root;no-stationarity-tolerance|merit=complete-normalized-inertia-plus-phr;two-precancelled-divided-evaluations;positive-reduction-required;always-long-double-binary64-owned;binary128-only-if-long-unresolved-or-sign-disagrees|controls=scalar-lower-upper-free-projection;multi-face;route-precedence;parent;source;projection;contact;superset;coverage;correspondence;feasibility;relinearization;divided-repeat;precision;work;rollback|routes=contact-tangent-parent-rejected;contact-tangent-source-rejected;contact-tangent-projection-rejected;contact-tangent-contact-rejected;contact-tangent-superset-rejected;contact-tangent-relinearization-rejected;contact-tangent-feasibility-rejected;contact-tangent-merit-precision-required;tangential-merit-step-required;contact-tangent-normal-step-candidate|precedence=parent,source,projection,contact,superset-and-relinearization,feasibility,precision,full-merit,candidate|work=parent-r42-replays1;static-index1;superset-build1;diagnostic-workspaces2;releases2;maximum-live2;filters2;source-jvp1;trial-vjp1;contact-tests36000;divided-repeats2;long-audits1;binary128-audits<=1;new-hvp0;new-model0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42=unchanged;unprojected-trial=unchanged;correction=none;candidate-commit=none;tolerance=none;penalty-change=none;support-expansion=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-contact-tangent-normal-step-classification-only)IDENTITY";

double& al_r43_component(Vec3& value, int axis) {
    return axis == 0 ? value.x : (axis == 1 ? value.y : value.z);
}

std::string al_r43_route(bool parent, bool source, bool projection,
    bool contact, bool superset, bool relin, bool feasibility,
    bool precision, bool merit) {
    if (!parent) return "CONTACT_TANGENT_PARENT_REJECTED";
    if (!source) return "CONTACT_TANGENT_SOURCE_REJECTED";
    if (!projection) return "CONTACT_TANGENT_PROJECTION_REJECTED";
    if (!contact) return "CONTACT_TANGENT_CONTACT_REJECTED";
    if (!superset) return "CONTACT_TANGENT_SUPERSET_REJECTED";
    if (!relin) return "CONTACT_TANGENT_RELINEARIZATION_REJECTED";
    if (!feasibility) return "CONTACT_TANGENT_FEASIBILITY_REJECTED";
    if (!precision) return "CONTACT_TANGENT_MERIT_PRECISION_REQUIRED";
    if (!merit) return "TANGENTIAL_MERIT_STEP_REQUIRED";
    return "CONTACT_TANGENT_NORMAL_STEP_CANDIDATE";
}

} // namespace

SplitBoundaryReport run_al_contact_tangent_normal_step_controls() {
    const bool identity = sha256_hex(B4E2D7R19R43_IDENTITY_PROJECTION)
        == B4E2D7R19R43_IDENTITY_SHA256;
    ALR42ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_stable_superset_relinearization_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "f7ae3a0c60efce0a78d3d1821e51b12c97f2fb8557724e5f3082bf493bb45106"
        && parent.json.find("fed11eefb017fce471b41fa25a20f5db9ea93e563b90fe5d02123380d52784d2")
            != std::string::npos;
    ALR40ReplayCapture& r40 = capture.r40;
    ALR30RangeProjectionCapture& r30 =
        r40.r39.r38.r37.r36.r35.r34.r33.r32.r31.r30;
    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string unprojected_before =
        al_binary64_vec3_root(r40.trial_position);
    const std::string endpoint_before =
        al_binary64_vec3_root(r40.r39.hz.terminal_iterate);
    const SmokeFixture fixture = make_b4e2d_dam_fixture();
    const ALR41ContactAudit unprojected_contact =
        al_r41_contact_audit(r30.position, r40.trial_position, fixture);
    const bool source = parent_exact && position_before
            == "e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828"
        && unprojected_before
            == "60e793aff919fada6b49c1b98be7ee5cc4a2c74a4254fa762cdece5700e1e804"
        && endpoint_before
            == "a73caf5baa6708ca89693077bad943dcdbf1f5069d50270ccc0f6b184a102e3a"
        && al_r42_superset_root(capture.superset)
            == "355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01"
        && unprojected_contact.root
            == "1df2083d4096522f329cdb1ff2a70c5da448ead0d40cda2cdb02e520c88f1ef7";
    const auto scalar_project = [](double x, double low, double high,
                                   double step) {
        if (low - x > 0.0 && step < 0.0) return 0.0;
        if (x - high > 0.0 && step > 0.0) return 0.0;
        return step;
    };
    const bool dense = scalar_project(-1.0, 0.0, 2.0, -3.0) == 0.0
        && scalar_project(3.0, 0.0, 2.0, 4.0) == 0.0
        && scalar_project(1.0, 0.0, 2.0, -5.0) == -5.0
        && scalar_project(-1.0, 0.0, 2.0, 3.0) == 3.0;
    std::vector<Vec3> projected = r40.r39.hz.terminal_iterate;
    std::vector<std::uint8_t> owners(projected.size() * 3U, 0U);
    std::size_t active_faces = 0U;
    std::size_t clamped = 0U;
    for (std::size_t i = 0U; i < projected.size(); ++i) {
        for (int axis = 0; axis < 3; ++axis) {
            const double x = al_r41_component(r30.position[i], axis);
            const double low = al_r41_component(fixture.contact_low, axis);
            const double high = al_r41_component(fixture.contact_high, axis);
            double& step = al_r43_component(projected[i], axis);
            if (low - x > 0.0) {
                owners[3U * i + static_cast<std::size_t>(axis)] = 1U;
                ++active_faces;
                if (step < 0.0) { step = 0.0; ++clamped; }
            } else if (x - high > 0.0) {
                owners[3U * i + static_cast<std::size_t>(axis)] = 2U;
                ++active_faces;
                if (step > 0.0) { step = 0.0; ++clamped; }
            }
        }
    }
    std::vector<Vec3> physical(projected.size());
    std::vector<Vec3> trial(projected.size());
    for (std::size_t i = 0U; i < projected.size(); ++i) {
        physical[i] = SPACING * projected[i];
        trial[i] = r30.position[i] + physical[i];
    }
    std::vector<Vec3> projected_twice = projected;
    for (std::size_t i = 0U; i < projected_twice.size(); ++i)
        for (int axis = 0; axis < 3; ++axis) {
            double& step = al_r43_component(projected_twice[i], axis);
            const std::uint8_t owner = owners[3U * i + axis];
            if ((owner == 1U && step < 0.0) || (owner == 2U && step > 0.0))
                step = 0.0;
        }
    const bool projection_exact = source && dense && active_faces == 1290U
        && clamped == 1268U
        && exact_vec3_values(projected_twice, projected)
        && al_r32_vec3_norm(projected)
            <= al_r32_vec3_norm(r40.r39.hz.terminal_iterate)
        && al_r32_vec3_norm(physical) <= al_r32_vec3_norm(r40.physical_step);
    const std::string owner_root = al_r29_mask_root(owners);
    const std::string projected_root = al_binary64_vec3_root(projected);
    const std::string physical_root = al_binary64_vec3_root(physical);
    const std::string trial_root = al_binary64_vec3_root(trial);

    const ALR41ContactAudit contact_audit =
        al_r41_contact_audit(r30.position, trial, fixture);
    const bool contact_exact = contact_audit.finite
        && contact_audit.tests == 36000U
        && contact_audit.newly_penetrating == 0U
        && contact_audit.worsened == 0U
        && contact_audit.trial_maximum <= contact_audit.source_maximum;

    ALSparseWorkTrace work;
    ALSparsePrecisionWorkTrace precision_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    const JointStaticSupportIndex index = build_joint_static_support_index(
        tagged_points(fixture.boundary), &static_work);
    const JointStaticSupportBinding binding = bind_joint_static_support_index(
        &index, index.identity_sha256);
    JointNeighborhood superset = b4ep3_build_superset(
        tagged_points(r30.position), binding);
    double maximum_displacement_squared = 0.0;
    const bool certificate = b4ep3_max_displacement_squared(
        tagged_points(r30.position), tagged_points(trial),
        maximum_displacement_squared) && b4ep3_certificate(
            maximum_displacement_squared);
    JointNeighborhood current_filtered = b4ep3_filter_superset(
        tagged_points(r30.position), superset);
    JointNeighborhood trial_filtered = b4ep3_filter_superset(
        tagged_points(trial), superset);
    ALNormalizedWorkspace current = build_al_normalized_sparse_workspace(
        r30.position, binding, r30.dual, r30.theta, &work,
        &static_work, &adjacency_work);
    ALNormalizedWorkspace trial_workspace = build_al_normalized_sparse_workspace(
        trial, binding, r30.dual, r30.theta, &work,
        &static_work, &adjacency_work);
    const bool superset_exact = certificate && superset.passed
        && al_r42_superset_root(superset)
            == "355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01"
        && current_filtered.pairs == current.neighborhood.pairs
        && trial_filtered.pairs == trial_workspace.neighborhood.pairs;

    const ALLinearizedJvp response = al_r29_pair_once_jvp(current, projected);
    std::vector<double> linear_constraint(r30.constraint.size());
    for (std::size_t i = 0U; i < linear_constraint.size(); ++i)
        linear_constraint[i] = r30.constraint[i] + response.value[i];
    const ALR40HingeMetric source_metric = al_r40_hinge_metric(r30.constraint);
    const ALR40HingeMetric linear_metric = al_r40_hinge_metric(linear_constraint);
    const ALR40HingeMetric trial_metric =
        al_r40_hinge_metric(trial_workspace.support.constraint);
    const double predicted_reduction = source_metric.psi - linear_metric.psi;
    const double actual_reduction = source_metric.psi - trial_metric.psi;
    const double rho = actual_reduction / predicted_reduction;
    const bool feasibility = response.passed && predicted_reduction > 0.0
        && actual_reduction > 0.0 && std::isfinite(rho) && rho >= 0.1;
    std::vector<double> positive(trial_workspace.support.constraint.size());
    for (std::size_t i = 0U; i < positive.size(); ++i)
        positive[i] = std::max(0.0, trial_workspace.support.constraint[i]);
    const std::vector<Vec3> mapping =
        al_r29_pair_once_vjp(trial_workspace, positive);
    const double mapping_norm = al_r32_vec3_norm(mapping);
    const bool relinearization = superset_exact && trial_metric.finite
        && mapping.size() == trial.size() && std::isfinite(mapping_norm);

    const ALNormalizedPrecancellationDivided divided =
        evaluate_al_normalized_sparse_precancelled_divided(current,
            trial_workspace, r30.predicted_position, r30.dual, r30.theta, &work);
    const ALNormalizedPrecancellationDivided divided_repeat =
        evaluate_al_normalized_sparse_precancelled_divided(current,
            trial_workspace, r30.predicted_position, r30.dual, r30.theta, &work);
    const bool divided_exact =
        al_normalized_precancellation_exact(divided, divided_repeat);
    const LongDoubleTrialEnergyAudit long_audit =
        audit_al_normalized_sparse_long_double(r30.position, trial,
            r30.predicted_position, r30.dual, r30.theta, binding,
            &precision_work, ALPrecisionMembershipPolicy::Binary64Owned);
    const bool candidate_positive = divided.value.reduction > 0.0;
    const bool candidate_negative = divided.value.reduction < 0.0;
    const bool long_disagrees = long_audit.resolved
        && ((candidate_positive && !long_audit.resolved_positive)
            || (candidate_negative && !long_audit.resolved_negative)
            || (!candidate_positive && !candidate_negative));
    const bool binary_executed = !long_audit.resolved || long_disagrees;
    Binary128TrialEnergyAudit binary_audit;
    if (binary_executed)
        binary_audit = audit_al_normalized_sparse_binary128(r30.position,
            trial, r30.predicted_position, r30.dual, r30.theta, binding,
            divided.value.reduction, &precision_work,
            ALPrecisionMembershipPolicy::Binary64Owned);
    const bool sign_agrees = (candidate_positive
            && (binary_executed ? binary_audit.resolved_positive
                                : long_audit.resolved_positive))
        || (candidate_negative
            && (binary_executed ? binary_audit.resolved_negative
                                : long_audit.resolved_negative));
    const bool precision = divided_exact && long_audit.finite_values
        && (binary_executed ? binary_audit.resolved : long_audit.resolved)
        && (binary_executed ? binary_audit.finite_values : true)
        && sign_agrees;
    const bool merit = precision && candidate_positive
        && (binary_executed ? binary_audit.resolved_positive
                            : long_audit.resolved_positive);
    if (trial_workspace.passed) release_al_normalized_workspace(trial_workspace, &work);
    if (current.passed) release_al_normalized_workspace(current, &work);
    const bool work_exact = work.workspace_builds == 2U
        && work.workspace_releases == 2U && work.live_workspaces == 0U
        && work.maximum_live_workspaces == 2U
        && !work.lifecycle_underflow && work.all_pair_candidate_calls == 0U
        && static_work.static_index_builds == 1U
        && static_work.support_canonicalizations == 1U
        && static_work.workspace_builds == 2U
        && adjacency_work.workspace_builds == 2U
        && precision_work.long_double_audits == 1U
        && precision_work.binary128_audits == (binary_executed ? 1U : 0U)
        && precision_work.all_pair_candidate_calls == 0U;
    const bool rollback = position_before == al_binary64_vec3_root(r30.position)
        && unprojected_before == al_binary64_vec3_root(r40.trial_position)
        && endpoint_before == al_binary64_vec3_root(r40.r39.hz.terminal_iterate);
    const std::array<const char*, 10U> expected_routes{{
        "CONTACT_TANGENT_PARENT_REJECTED", "CONTACT_TANGENT_SOURCE_REJECTED",
        "CONTACT_TANGENT_PROJECTION_REJECTED", "CONTACT_TANGENT_CONTACT_REJECTED",
        "CONTACT_TANGENT_SUPERSET_REJECTED",
        "CONTACT_TANGENT_RELINEARIZATION_REJECTED",
        "CONTACT_TANGENT_FEASIBILITY_REJECTED",
        "CONTACT_TANGENT_MERIT_PRECISION_REQUIRED",
        "TANGENTIAL_MERIT_STEP_REQUIRED", "CONTACT_TANGENT_NORMAL_STEP_CANDIDATE"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t index_value = 0U; index_value < expected_routes.size(); ++index_value) {
        std::array<bool, 9U> gates{};
        gates.fill(true);
        if (index_value < gates.size()) gates[index_value] = false;
        const std::string observed = al_r43_route(gates[0], gates[1], gates[2],
            gates[3], gates[4], gates[5], gates[6], gates[7], gates[8]);
        routes_exact = routes_exact && observed == expected_routes[index_value];
        route_material << expected_routes[index_value] << ':' << observed << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r43_route(parent_exact, source,
        projection_exact, contact_exact, superset_exact, relinearization,
        feasibility, precision, merit);
    const bool passed = identity && parent_exact && source && dense
        && projection_exact
        && contact_exact && superset_exact && relinearization && feasibility
        && precision && work_exact && rollback && routes_exact;
    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R43_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << owner_root << ':' << projected_root << ':' << physical_root
             << ':' << trial_root << '|' << active_faces << ':' << clamped
             << ':' << contact_audit.root << '|' << predicted_reduction << ':'
             << actual_reduction << ':' << rho << ':' << trial_metric.psi << ':'
             << mapping_norm << '|' << divided.value.reduction << ':'
             << binary_executed << ':' << precision << ':' << merit << '|'
             << work_exact << ':' << rollback << ':' << route_root << ':'
             << route;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r43_contact_tangent_normal_step.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R43_IDENTITY_SHA256 << "\",\"parent_stdout_sha256\":\""
           << parent_stdout << "\",\"projection\":{\"active_faces\":"
           << active_faces << ",\"clamped_components\":" << clamped
           << ",\"owner_root\":\"" << owner_root
           << "\",\"endpoint_root\":\"" << projected_root
           << "\",\"physical_root\":\"" << physical_root
           << "\",\"trial_root\":\"" << trial_root << "\"}"
           << ",\"contact\":{\"new\":" << contact_audit.newly_penetrating
           << ",\"worsened\":" << contact_audit.worsened
           << ",\"source_maximum\":" << contact_audit.source_maximum
           << ",\"trial_maximum\":" << contact_audit.trial_maximum << "}"
           << ",\"feasibility\":{\"predicted_reduction\":"
           << predicted_reduction << ",\"actual_reduction\":"
           << actual_reduction << ",\"rho\":" << rho
           << ",\"trial_psi\":" << trial_metric.psi
           << ",\"trial_violation\":" << trial_metric.norm
           << ",\"trial_active\":" << trial_metric.active
           << ",\"trial_mapping\":" << mapping_norm << "}"
           << ",\"merit\":{\"reduction\":" << divided.value.reduction
           << ",\"long_resolved\":" << (long_audit.resolved ? "true" : "false")
           << ",\"binary128_executed\":" << (binary_executed ? "true" : "false")
           << ",\"positive\":" << (merit ? "true" : "false") << "}"
           << ",\"route_cases\":" << expected_routes.size()
           << ",\"route_root\":\"" << route_root
           << "\",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"work_exact\":" << (work_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback ? "true" : "false")
           << ",\"route\":\"" << route
           << "\",\"correction_applied\":false,\"candidate_committed\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    return {passed, report.str()};
}
