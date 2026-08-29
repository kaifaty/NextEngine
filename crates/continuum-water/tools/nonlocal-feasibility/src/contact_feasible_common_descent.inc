namespace {

constexpr const char* B4E2D7R19R44_IDENTITY_SHA256 =
    "7e10698ee75568845fe432f83d9a31e01f516a2f131840e83ee15ae8b48a0aba";
constexpr const char* B4E2D7R19R44_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r44-contact-feasible-common-descent|v1|parent=a47d7ea7:565c70d251612bac17c0b4d20b5ae4f284f9d64c2edd34871db640b1d58bc993:e1f019dae6e71e40633e56c79ee60e8e81dc3b146fdf55d75542298737f9eea6|source=r43-route-tangential-merit-step-required;projected-endpoint-30885f22768de119065907526f36c225d058c6b97693770be109faf3ddd3b960;projected-trial-0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50;owner-baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187;superset-355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01;trial-psi6.8540208964482797e-16;trial-active464;trial-mapping9.7690943252330519e-9|gradient=complete-normalized-inner-at-projected-trial;dimensionless=spacing*physical-gradient;raw-negative-gradient;source-contact-cone-projection;inward-to-positive-zero;other-bits-retained|operator=fresh-projected-trial-A=spacing*Jc;image=A*dcontact;positive-trial-rows|slopes=long-double-dot;merit=gT*dcontact;feasibility=max(cTrial,0)T*A*dcontact;strict-sign-no-fitted-tolerance;normalized-slopes;direction-and-image-roots|selection=both-negative-common-descent;merit-negative-feasibility-nonnegative-nullspace-required;merit-nonnegative-decomposition-required|controls=analytic-orthant-gradient;route-precedence;parent;source;workspace;gradient;projection;operator;slopes;work;rollback|routes=common-descent-parent-rejected;common-descent-source-rejected;common-descent-workspace-rejected;common-descent-gradient-rejected;common-descent-contact-rejected;common-descent-operator-rejected;common-descent-work-rejected;contact-feasible-common-descent-candidate;density-nullspace-projection-required;composite-decomposition-required|precedence=parent,source,workspace,gradient,contact,operator,work,merit-sign,feasibility-sign|work=parent-r43-replays1;static-index1;superset-build1;trial-workspace1;release1;trial-jvp1;long-double-dot2;new-vjp0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|line-search=none;moved-state=none;nullspace-solve=none;correction=none;candidate-commit=none;tolerance=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-contact-feasible-common-descent-classification-only)IDENTITY";

std::string al_r44_route(bool parent, bool source, bool workspace,
    bool gradient, bool contact, bool op, bool work,
    bool merit_negative, bool feasibility_negative) {
    if (!parent) return "COMMON_DESCENT_PARENT_REJECTED";
    if (!source) return "COMMON_DESCENT_SOURCE_REJECTED";
    if (!workspace) return "COMMON_DESCENT_WORKSPACE_REJECTED";
    if (!gradient) return "COMMON_DESCENT_GRADIENT_REJECTED";
    if (!contact) return "COMMON_DESCENT_CONTACT_REJECTED";
    if (!op) return "COMMON_DESCENT_OPERATOR_REJECTED";
    if (!work) return "COMMON_DESCENT_WORK_REJECTED";
    if (!merit_negative) return "COMPOSITE_DECOMPOSITION_REQUIRED";
    if (!feasibility_negative) return "DENSITY_NULLSPACE_PROJECTION_REQUIRED";
    return "CONTACT_FEASIBLE_COMMON_DESCENT_CANDIDATE";
}

struct ALR44ReplayCapture {
    bool captured = false;
    ALR43ReplayCapture r43;
    std::vector<Vec3> gradient;
    std::vector<Vec3> direction;
    std::vector<double> image;
    long double merit_slope = 0.0L;
    long double feasibility_slope = 0.0L;
    double direction_norm = 0.0;
};

} // namespace

SplitBoundaryReport run_al_contact_feasible_common_descent_controls_impl(
    ALR44ReplayCapture* replay_capture) {
    const bool identity = sha256_hex(B4E2D7R19R44_IDENTITY_PROJECTION)
        == B4E2D7R19R44_IDENTITY_SHA256;
    ALR43ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_contact_tangent_normal_step_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "565c70d251612bac17c0b4d20b5ae4f284f9d64c2edd34871db640b1d58bc993"
        && parent.json.find("e1f019dae6e71e40633e56c79ee60e8e81dc3b146fdf55d75542298737f9eea6")
            != std::string::npos
        && parent.json.find("TANGENTIAL_MERIT_STEP_REQUIRED")
            != std::string::npos;
    ALR40ReplayCapture& r40 = capture.r42.r40;
    ALR30RangeProjectionCapture& r30 =
        r40.r39.r38.r37.r36.r35.r34.r33.r32.r31.r30;
    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string projected_before =
        al_binary64_vec3_root(capture.projected_endpoint);
    const std::string trial_before =
        al_binary64_vec3_root(capture.projected_trial);
    const std::string owner_root = al_r29_mask_root(capture.owners);
    const bool source = parent_exact
        && projected_before
            == "30885f22768de119065907526f36c225d058c6b97693770be109faf3ddd3b960"
        && trial_before
            == "0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50"
        && owner_root
            == "baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187";
    const auto project_scalar = [](std::uint8_t owner, double value) {
        if ((owner == 1U && value < 0.0)
            || (owner == 2U && value > 0.0)) return 0.0;
        return value;
    };
    const bool dense = project_scalar(1U, -2.0) == 0.0
        && project_scalar(1U, 2.0) == 2.0
        && project_scalar(2U, 2.0) == 0.0
        && project_scalar(0U, -3.0) == -3.0;

    ALSparseWorkTrace work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    const SmokeFixture fixture = make_b4e2d_dam_fixture();
    const JointStaticSupportIndex index = build_joint_static_support_index(
        tagged_points(fixture.boundary), &static_work);
    const JointStaticSupportBinding binding = bind_joint_static_support_index(
        &index, index.identity_sha256);
    JointNeighborhood superset = b4ep3_build_superset(
        tagged_points(r30.position), binding);
    JointNeighborhood filtered = b4ep3_filter_superset(
        tagged_points(capture.projected_trial), superset);
    ALNormalizedWorkspace workspace = build_al_normalized_sparse_workspace(
        capture.projected_trial, binding, r30.dual, r30.theta, &work,
        &static_work, &adjacency_work);
    const ALR40HingeMetric metric =
        al_r40_hinge_metric(workspace.support.constraint);
    const bool workspace_exact = source && dense && workspace.passed
        && superset.passed && al_r42_superset_root(superset)
            == "355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01"
        && filtered.pairs == workspace.neighborhood.pairs
        && metric.finite && metric.psi == 6.8540208964482797e-16
        && metric.active == 464U;
    const ALNormalizedInner inner = evaluate_al_normalized_inner(
        workspace.support, capture.projected_trial, r30.predicted_position);
    std::vector<Vec3> gradient(inner.gradient.size());
    std::vector<Vec3> raw(inner.gradient.size());
    std::vector<Vec3> direction(inner.gradient.size());
    std::size_t clamped = 0U;
    for (std::size_t i = 0U; i < inner.gradient.size(); ++i) {
        gradient[i] = SPACING * inner.gradient[i];
        raw[i] = -1.0 * gradient[i];
        direction[i] = raw[i];
        for (int axis = 0; axis < 3; ++axis) {
            double& value = al_r43_component(direction[i], axis);
            const double projected = project_scalar(
                capture.owners[3U * i + static_cast<std::size_t>(axis)], value);
            if (projected != value) ++clamped;
            value = projected;
        }
    }
    std::vector<Vec3> twice = direction;
    for (std::size_t i = 0U; i < twice.size(); ++i)
        for (int axis = 0; axis < 3; ++axis) {
            double& value = al_r43_component(twice[i], axis);
            value = project_scalar(
                capture.owners[3U * i + static_cast<std::size_t>(axis)], value);
        }
    const double gradient_norm = al_r32_vec3_norm(gradient);
    const double raw_norm = al_r32_vec3_norm(raw);
    const double direction_norm = al_r32_vec3_norm(direction);
    const bool gradient_exact = inner.passed && al_r32_vec3_finite(gradient)
        && gradient_norm > 0.0 && raw_norm == gradient_norm;
    const bool contact_exact = exact_vec3_values(twice, direction)
        && direction_norm > 0.0 && direction_norm <= raw_norm;
    const ALLinearizedJvp image = al_r29_pair_once_jvp(workspace, direction);
    std::vector<double> positive(workspace.support.constraint.size());
    for (std::size_t i = 0U; i < positive.size(); ++i)
        positive[i] = std::max(0.0, workspace.support.constraint[i]);
    long double merit_slope = 0.0L;
    long double feasibility_slope = 0.0L;
    for (std::size_t i = 0U; i < direction.size(); ++i) {
        merit_slope += static_cast<long double>(gradient[i].x) * direction[i].x
            + static_cast<long double>(gradient[i].y) * direction[i].y
            + static_cast<long double>(gradient[i].z) * direction[i].z;
        feasibility_slope += static_cast<long double>(positive[i])
            * static_cast<long double>(image.value[i]);
    }
    const double image_norm = al_r29_scalar_norm(image.value);
    const double positive_norm = al_r29_scalar_norm(positive);
    const double merit_normalized = static_cast<double>(merit_slope)
        / (gradient_norm * direction_norm);
    const double feasibility_normalized = static_cast<double>(feasibility_slope)
        / std::max(positive_norm * image_norm, 1.0e-300);
    const bool operator_exact = image.passed && std::isfinite(image_norm)
        && std::isfinite(merit_normalized)
        && std::isfinite(feasibility_normalized);
    const bool merit_negative = merit_slope < 0.0L;
    const bool feasibility_negative = feasibility_slope < 0.0L;
    const std::string gradient_root = al_binary64_vec3_root(gradient);
    const std::string raw_root = al_binary64_vec3_root(raw);
    const std::string direction_root = al_binary64_vec3_root(direction);
    const std::string image_root = al_r10_dual_root(image.value);
    release_al_normalized_workspace(workspace, &work);
    const bool work_exact = work.workspace_builds == 1U
        && work.workspace_releases == 1U && work.live_workspaces == 0U
        && work.maximum_live_workspaces == 1U && !work.lifecycle_underflow
        && static_work.static_index_builds == 1U
        && static_work.support_canonicalizations == 1U
        && static_work.workspace_builds == 1U
        && adjacency_work.workspace_builds == 1U;
    const bool rollback = position_before == al_binary64_vec3_root(r30.position)
        && projected_before == al_binary64_vec3_root(capture.projected_endpoint)
        && trial_before == al_binary64_vec3_root(capture.projected_trial);
    const std::array<const char*, 10U> expected{{
        "COMMON_DESCENT_PARENT_REJECTED", "COMMON_DESCENT_SOURCE_REJECTED",
        "COMMON_DESCENT_WORKSPACE_REJECTED", "COMMON_DESCENT_GRADIENT_REJECTED",
        "COMMON_DESCENT_CONTACT_REJECTED", "COMMON_DESCENT_OPERATOR_REJECTED",
        "COMMON_DESCENT_WORK_REJECTED", "COMPOSITE_DECOMPOSITION_REQUIRED",
        "DENSITY_NULLSPACE_PROJECTION_REQUIRED",
        "CONTACT_FEASIBLE_COMMON_DESCENT_CANDIDATE"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t i = 0U; i < expected.size(); ++i) {
        std::array<bool, 9U> gates{};
        gates.fill(true);
        if (i < 7U) gates[i] = false;
        else if (i == 7U) gates[7] = false;
        else if (i == 8U) gates[8] = false;
        const std::string observed = al_r44_route(gates[0], gates[1], gates[2],
            gates[3], gates[4], gates[5], gates[6], gates[7], gates[8]);
        routes_exact = routes_exact && observed == expected[i];
        route_material << expected[i] << ':' << observed << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r44_route(parent_exact, source,
        workspace_exact, gradient_exact, contact_exact, operator_exact,
        work_exact && rollback, merit_negative, feasibility_negative);
    const bool passed = identity && parent_exact && source && dense
        && workspace_exact && gradient_exact && contact_exact && operator_exact
        && work_exact && rollback && routes_exact;
    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R44_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << gradient_root << ':' << raw_root << ':' << direction_root << ':'
             << image_root << '|' << gradient_norm << ':' << direction_norm
             << ':' << image_norm << ':' << clamped << '|'
             << static_cast<double>(merit_slope) << ':'
             << static_cast<double>(feasibility_slope) << ':'
             << merit_normalized << ':' << feasibility_normalized << '|'
             << work_exact << ':' << rollback << ':' << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r44_contact_feasible_common_descent.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R44_IDENTITY_SHA256 << "\",\"parent_stdout_sha256\":\""
           << parent_stdout << "\",\"direction\":{\"gradient_norm\":"
           << gradient_norm << ",\"raw_norm\":" << raw_norm
           << ",\"contact_norm\":" << direction_norm
           << ",\"clamped_components\":" << clamped
           << ",\"gradient_root\":\"" << gradient_root
           << "\",\"raw_root\":\"" << raw_root
           << "\",\"contact_root\":\"" << direction_root << "\"}"
           << ",\"operator\":{\"image_norm\":" << image_norm
           << ",\"image_root\":\"" << image_root << "\"}"
           << ",\"slopes\":{\"merit\":" << static_cast<double>(merit_slope)
           << ",\"feasibility\":" << static_cast<double>(feasibility_slope)
           << ",\"merit_normalized\":" << merit_normalized
           << ",\"feasibility_normalized\":" << feasibility_normalized << "}"
           << ",\"route_cases\":" << expected.size()
           << ",\"route_root\":\"" << route_root
           << "\",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"work_exact\":" << (work_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback ? "true" : "false")
           << ",\"route\":\"" << route
           << "\",\"line_search_executed\":false,\"moved_state\":false"
              ",\"nullspace_solve_executed\":false,\"candidate_committed\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    if (replay_capture != nullptr) {
        replay_capture->captured = passed;
        replay_capture->r43 = std::move(capture);
        replay_capture->gradient = std::move(gradient);
        replay_capture->direction = std::move(direction);
        replay_capture->image = image.value;
        replay_capture->merit_slope = merit_slope;
        replay_capture->feasibility_slope = feasibility_slope;
        replay_capture->direction_norm = direction_norm;
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_contact_feasible_common_descent_controls() {
    return run_al_contact_feasible_common_descent_controls_impl(nullptr);
}
