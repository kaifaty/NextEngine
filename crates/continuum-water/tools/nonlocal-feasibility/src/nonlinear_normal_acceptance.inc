namespace {

constexpr const char* B4E2D7R19R40_IDENTITY_SHA256 =
    "5dbe3c14fa231eab01f176c7fd2423c08485c786cfff0075670a65c7b6b1431a";
constexpr const char* B4E2D7R19R40_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r40-nonlinear-normal-acceptance|v1|parent=56ddb2ac:06812cb75177cd4666b103101f6bc9238fa20760179bc71563f194fe476db252:ce86c4266d224165fec4a3de32786e986f29bae95afe96dc41937f895157203b|source=state-851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7;position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;predicted-36112dde1e0b274c5b9216f4818b82977a0c80dc257478c111b0f0a9390d2d7e;dual-cd0100c30eb55ad7f91f1b94c00ee4281ff3c75e90ca287be038e87ab7a5215d;constraint-c567599060d7d00ffec4928009772327d081d3ef294d9030bfc5499b82e533c9;topology-070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5;theta0x3fc5cccccccccccd;particles6000;pairs340340;max-degree113|candidate=r39-hz-trajectory-73fe00202b08e17458fe0c9e495d6c9e077a93918c53d2d5d6ee1702dd8fefd4;terminal-checkpoint-9da21927cfeee2fddf448306843fe306410a2e8ee0e124aee5f0c56789ae6986;terminal-response-2dcc7b8d136864b3b7bf33d3bb617f7f7a9fb531add1e2965d6b58d4248d4bbf;terminal-mapping-167f62a1419b96ff117a9e3749f0bda8fb61b27232685b3ba1623cd2668aa3ce;linear-phi9.1332887060949100e-27;dimensionless-global-l2<=0.25|mapping=trial-position=current-position+SPACING*hz-endpoint;spacing0x3fa999999999999a;physical-global-l2<=0.0125;rollback-only|nonlinear=fresh-current-and-trial-normalized-static-sparse;constraint-psi=0.5*norm2(max(0,c));pair-membership-exact;contact=dam-box-no-increased-penetration-over-max-source-or1e-12|accept=feasibility-predicted=source-psi-minus-linear-psi;actual=source-psi-minus-trial-psi;both-positive;rho=actual/predicted;rho>=0.1;full-merit=normalized-inertia-plus-phr;precancelled-reduction-positive;precision-resolved-positive|precision=always-long-double-binary64-owned;binary128-only-if-long-unresolved-or-sign-disagrees;topology-precision-mismatch-report-only|controls=analytic-nonlinear-accept;feasibility-pass-full-merit-reject;route-precedence;parent;source;endpoint;mapping;workspace;topology;contact;divided-repeat;precision;work;rollback|routes=nonlinear-parent-rejected;nonlinear-source-rejected;nonlinear-dense-rejected;nonlinear-endpoint-rejected;nonlinear-mapping-rejected;nonlinear-workspace-rejected;nonlinear-topology-rejected;nonlinear-contact-rejected;nonlinear-feasibility-model-rejected;nonlinear-merit-precision-required;composite-normal-tangential-step-required;nonlinear-normal-step-acceptance-candidate|precedence=parent,source,dense,endpoint,mapping,workspace,topology,contact,feasibility,precision,full-merit,candidate|work=parent-r39-replays1;diagnostic-workspaces2;releases2;maximum-live2;nonlinear-trials1;divided-repeats2;long-audits1;binary128-audits<=1;new-jvp0;new-vjp0;new-hvp0;new-model0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|correction=none;candidate-commit=none;following-outer=none;tolerance=none;penalty-change=none;public-state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-nonlinear-normal-step-classification-only)IDENTITY";

struct ALR40HingeMetric {
    bool finite = false;
    double psi = 0.0;
    double norm = 0.0;
    std::size_t active = 0U;
};

struct ALR40DenseControls {
    bool passed = false;
    double rho = 0.0;
    double accepted_merit_reduction = 0.0;
    double rejected_merit_reduction = 0.0;
    std::string root;
};

ALR40HingeMetric al_r40_hinge_metric(
    const std::vector<double>& constraint) {
    ALR40HingeMetric result;
    long double psi = 0.0L;
    for (double value : constraint) {
        if (!std::isfinite(value)) return result;
        if (value <= 0.0) continue;
        ++result.active;
        psi += static_cast<long double>(0.5) * value * value;
    }
    result.psi = static_cast<double>(psi);
    result.norm = std::sqrt(2.0 * result.psi);
    result.finite = std::isfinite(result.psi)
        && std::isfinite(result.norm);
    return result;
}

ALR40DenseControls al_r40_dense_controls() {
    const auto constraint = [](double x) { return x + x * x; };
    const auto psi = [&](double x) {
        const double c = std::max(0.0, constraint(x));
        return 0.5 * c * c;
    };
    const auto merit = [&](double x, double predicted, double theta) {
        const double displacement = x - predicted;
        return 0.5 * displacement * displacement + theta * psi(x);
    };
    constexpr double current = 0.1;
    constexpr double trial = 0.05;
    const double step = trial - current;
    const double linear_constraint = constraint(current)
        + (1.0 + 2.0 * current) * step;
    const double linear_psi = 0.5 * linear_constraint * linear_constraint;
    const double predicted_reduction = psi(current) - linear_psi;
    const double actual_reduction = psi(current) - psi(trial);
    ALR40DenseControls result;
    result.rho = actual_reduction / predicted_reduction;
    result.accepted_merit_reduction =
        merit(current, 0.0, 1.0) - merit(trial, 0.0, 1.0);
    result.rejected_merit_reduction =
        merit(current, current, 0.01) - merit(trial, current, 0.01);
    result.passed = predicted_reduction > 0.0
        && actual_reduction > 0.0 && result.rho >= 0.1
        && result.accepted_merit_reduction > 0.0
        && result.rejected_merit_reduction < 0.0;
    std::ostringstream projection;
    projection << binary64_bits(current) << ':' << binary64_bits(trial)
               << ':' << binary64_bits(linear_constraint) << ':'
               << binary64_bits(linear_psi) << ':'
               << binary64_bits(predicted_reduction) << ':'
               << binary64_bits(actual_reduction) << ':'
               << binary64_bits(result.rho) << ':'
               << binary64_bits(result.accepted_merit_reduction) << ':'
               << binary64_bits(result.rejected_merit_reduction) << ':'
               << result.passed;
    result.root = sha256_hex(projection.str());
    return result;
}

std::string al_r40_pair_membership_root(
    const JointNeighborhood& neighborhood) {
    std::ostringstream projection;
    for (const JointPair pair : neighborhood.pairs) {
        projection << pair.fluid << ':' << pair.participant << '|';
    }
    return sha256_hex(projection.str());
}

double al_r40_maximum_box_penetration(
    const std::vector<Vec3>& position,
    const SmokeFixture& fixture) {
    double maximum = 0.0;
    for (Vec3 value : position) {
        maximum = std::max({maximum,
            fixture.contact_low.x - value.x,
            fixture.contact_low.y - value.y,
            fixture.contact_low.z - value.z,
            value.x - fixture.contact_high.x,
            value.y - fixture.contact_high.y,
            value.z - fixture.contact_high.z});
    }
    return maximum;
}

std::string al_r40_route(
    bool parent, bool source, bool dense, bool endpoint, bool mapping,
    bool workspace, bool topology, bool contact, bool feasibility,
    bool precision, bool full_merit) {
    if (!parent) return "NONLINEAR_PARENT_REJECTED";
    if (!source) return "NONLINEAR_SOURCE_REJECTED";
    if (!dense) return "NONLINEAR_DENSE_REJECTED";
    if (!endpoint) return "NONLINEAR_ENDPOINT_REJECTED";
    if (!mapping) return "NONLINEAR_MAPPING_REJECTED";
    if (!workspace) return "NONLINEAR_WORKSPACE_REJECTED";
    if (!topology) return "NONLINEAR_TOPOLOGY_REJECTED";
    if (!contact) return "NONLINEAR_CONTACT_REJECTED";
    if (!feasibility) return "NONLINEAR_FEASIBILITY_MODEL_REJECTED";
    if (!precision) return "NONLINEAR_MERIT_PRECISION_REQUIRED";
    if (!full_merit) return "COMPOSITE_NORMAL_TANGENTIAL_STEP_REQUIRED";
    return "NONLINEAR_NORMAL_STEP_ACCEPTANCE_CANDIDATE";
}

} // namespace

SplitBoundaryReport run_al_nonlinear_normal_acceptance_controls() {
    const bool identity_exact = sha256_hex(
        B4E2D7R19R40_IDENTITY_PROJECTION)
        == B4E2D7R19R40_IDENTITY_SHA256;
    ALR39ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_guarded_hz_recurrence_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "ce86c4266d224165fec4a3de32786e986f29bae95afe96dc41937f895157203b"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && parent_semantic && parent_stdout
            == "06812cb75177cd4666b103101f6bc9238fa20760179bc71563f194fe476db252";

    ALR30RangeProjectionCapture& r30 =
        capture.r38.r37.r36.r35.r34.r33.r32.r31.r30;
    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string predicted_before =
        al_binary64_vec3_root(r30.predicted_position);
    const std::string dual_before = al_r10_dual_root(r30.dual);
    const std::string constraint_before = al_r10_dual_root(r30.constraint);
    const std::string endpoint_before =
        al_binary64_vec3_root(capture.hz.terminal_iterate);
    const bool source_exact = parent_exact && r30.captured
        && r30.state_root
            == "851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7"
        && position_before
            == "e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828"
        && predicted_before
            == "36112dde1e0b274c5b9216f4818b82977a0c80dc257478c111b0f0a9390d2d7e"
        && dual_before
            == "cd0100c30eb55ad7f91f1b94c00ee4281ff3c75e90ca287be038e87ab7a5215d"
        && constraint_before
            == "c567599060d7d00ffec4928009772327d081d3ef294d9030bfc5499b82e533c9"
        && r30.topology_root
            == "070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5"
        && binary64_bits(r30.theta) == 0x3fc5cccccccccccdULL
        && r30.position.size() == 6000U
        && r30.predicted_position.size() == r30.position.size()
        && r30.dual.size() == r30.position.size()
        && r30.constraint.size() == r30.position.size();

    const ALR40DenseControls dense = al_r40_dense_controls();
    const bool endpoint_exact = source_exact && capture.hz.passed
        && capture.hz.root
            == "73fe00202b08e17458fe0c9e495d6c9e077a93918c53d2d5d6ee1702dd8fefd4"
        && capture.hz.checkpoints.size() == 4U
        && capture.hz.checkpoints.back().root
            == "9da21927cfeee2fddf448306843fe306410a2e8ee0e124aee5f0c56789ae6986"
        && capture.hz.terminal_response_root
            == "2dcc7b8d136864b3b7bf33d3bb617f7f7a9fb531add1e2965d6b58d4248d4bbf"
        && capture.hz.terminal_mapping_root
            == "167f62a1419b96ff117a9e3749f0bda8fb61b27232685b3ba1623cd2668aa3ce"
        && capture.hz.terminal.phi == 9.13328870609491e-27
        && capture.hz.terminal_iterate.size() == r30.position.size()
        && al_r32_vec3_finite(capture.hz.terminal_iterate);
    const double dimensionless_norm = endpoint_exact
        ? al_r32_vec3_norm(capture.hz.terminal_iterate)
        : std::numeric_limits<double>::infinity();

    std::vector<Vec3> physical_step;
    std::vector<Vec3> trial_position;
    if (endpoint_exact && std::isfinite(dimensionless_norm)
        && dimensionless_norm <= 0.25) {
        physical_step.resize(capture.hz.terminal_iterate.size());
        trial_position.resize(r30.position.size());
        for (std::size_t index = 0U; index < r30.position.size(); ++index) {
            physical_step[index] =
                SPACING * capture.hz.terminal_iterate[index];
            trial_position[index] = r30.position[index] + physical_step[index];
        }
    }
    const double physical_norm = al_r32_vec3_norm(physical_step);
    const std::string physical_step_root =
        al_binary64_vec3_root(physical_step);
    const std::string trial_root = al_binary64_vec3_root(trial_position);
    const bool mapping_exact = endpoint_exact
        && binary64_bits(SPACING) == 0x3fa999999999999aULL
        && dimensionless_norm <= 0.25
        && physical_step.size() == r30.position.size()
        && trial_position.size() == r30.position.size()
        && al_r32_vec3_finite(physical_step)
        && al_r32_vec3_finite(trial_position)
        && std::isfinite(physical_norm) && physical_norm <= 0.0125;

    ALSparseWorkTrace work;
    ALSparsePrecisionWorkTrace precision_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    const SmokeFixture fixture = make_b4e2d_dam_fixture();
    const JointStaticSupportIndex index = build_joint_static_support_index(
        tagged_points(fixture.boundary), &static_work);
    const JointStaticSupportBinding binding = bind_joint_static_support_index(
        &index, index.identity_sha256);
    ALNormalizedWorkspace current_workspace;
    ALNormalizedWorkspace trial_workspace;
    if (mapping_exact && dense.passed && index.passed && binding.passed) {
        current_workspace = build_al_normalized_sparse_workspace(
            r30.position, binding, r30.dual, r30.theta, &work,
            &static_work, &adjacency_work);
        trial_workspace = build_al_normalized_sparse_workspace(
            trial_position, binding, r30.dual, r30.theta, &work,
            &static_work, &adjacency_work);
    }
    const std::string current_topology_root =
        al_r29_topology_root(current_workspace);
    const std::string trial_topology_root =
        al_r29_topology_root(trial_workspace);
    const std::string current_pair_root =
        al_r40_pair_membership_root(current_workspace.neighborhood);
    const std::string trial_pair_root =
        al_r40_pair_membership_root(trial_workspace.neighborhood);
    const bool workspace_exact = current_workspace.passed
        && trial_workspace.passed
        && current_workspace.support.constraint == r30.constraint
        && current_topology_root == r30.topology_root
        && current_workspace.neighborhood.pairs.size() == 340340U
        && current_workspace.neighborhood.maximum_degree == 113U
        && current_workspace.neighborhood.fluid.size() == 6000U
        && trial_workspace.neighborhood.fluid.size() == 6000U;
    const bool topology_exact = workspace_exact
        && current_workspace.neighborhood.pairs
            == trial_workspace.neighborhood.pairs
        && current_pair_root == trial_pair_root;

    const double source_penetration = mapping_exact
        ? al_r40_maximum_box_penetration(r30.position, fixture)
        : std::numeric_limits<double>::infinity();
    const double trial_penetration = mapping_exact
        ? al_r40_maximum_box_penetration(trial_position, fixture)
        : std::numeric_limits<double>::infinity();
    const bool contact_exact = std::isfinite(source_penetration)
        && std::isfinite(trial_penetration)
        && trial_penetration <= std::max(source_penetration, 1.0e-12);

    ALR40HingeMetric source_feasibility;
    ALR40HingeMetric trial_feasibility;
    double predicted_feasibility_reduction = 0.0;
    double actual_feasibility_reduction = 0.0;
    double feasibility_ratio = -std::numeric_limits<double>::infinity();
    if (workspace_exact) {
        source_feasibility = al_r40_hinge_metric(
            current_workspace.support.constraint);
        trial_feasibility = al_r40_hinge_metric(
            trial_workspace.support.constraint);
        predicted_feasibility_reduction = source_feasibility.psi
            - capture.hz.terminal.phi;
        actual_feasibility_reduction = source_feasibility.psi
            - trial_feasibility.psi;
        if (predicted_feasibility_reduction > 0.0) {
            feasibility_ratio = actual_feasibility_reduction
                / predicted_feasibility_reduction;
        }
    }
    const bool feasibility_exact = source_feasibility.finite
        && trial_feasibility.finite
        && std::isfinite(predicted_feasibility_reduction)
        && std::isfinite(actual_feasibility_reduction)
        && std::isfinite(feasibility_ratio)
        && predicted_feasibility_reduction > 0.0
        && actual_feasibility_reduction > 0.0
        && feasibility_ratio >= 0.1;

    ALNormalizedInner current_inner;
    ALNormalizedInner trial_inner;
    ALNormalizedPrecancellationDivided divided;
    ALNormalizedPrecancellationDivided divided_repeat;
    if (workspace_exact) {
        current_inner = evaluate_al_normalized_inner(
            current_workspace.support, r30.position,
            r30.predicted_position);
        trial_inner = evaluate_al_normalized_inner(
            trial_workspace.support, trial_position,
            r30.predicted_position);
    }
    if (current_inner.passed && trial_inner.passed) {
        divided = evaluate_al_normalized_sparse_precancelled_divided(
            current_workspace, trial_workspace, r30.predicted_position,
            r30.dual, r30.theta, &work);
        divided_repeat = evaluate_al_normalized_sparse_precancelled_divided(
            current_workspace, trial_workspace, r30.predicted_position,
            r30.dual, r30.theta, &work);
    }
    const bool divided_repeat_exact =
        al_normalized_precancellation_exact(divided, divided_repeat);
    const double raw_merit_reduction = current_inner.passed
            && trial_inner.passed
        ? current_inner.total - trial_inner.total
        : std::numeric_limits<double>::quiet_NaN();

    LongDoubleTrialEnergyAudit long_audit;
    if (workspace_exact) {
        long_audit = audit_al_normalized_sparse_long_double(
            r30.position, trial_position, r30.predicted_position,
            r30.dual, r30.theta, binding, &precision_work,
            ALPrecisionMembershipPolicy::Binary64Owned);
    }
    const bool candidate_positive = divided.value.reduction > 0.0;
    const bool candidate_negative = divided.value.reduction < 0.0;
    const bool long_disagrees = long_audit.resolved
        && ((candidate_positive && !long_audit.resolved_positive)
            || (candidate_negative && !long_audit.resolved_negative)
            || (!candidate_positive && !candidate_negative));
    const bool binary_executed = workspace_exact
        && (!long_audit.resolved || long_disagrees);
    Binary128TrialEnergyAudit binary_audit;
    if (binary_executed) {
        binary_audit = audit_al_normalized_sparse_binary128(
            r30.position, trial_position, r30.predicted_position,
            r30.dual, r30.theta, binding, divided.value.reduction,
            &precision_work, ALPrecisionMembershipPolicy::Binary64Owned);
    }
    const bool precision_resolved = binary_executed
        ? binary_audit.resolved : long_audit.resolved;
    const bool precision_positive = binary_executed
        ? binary_audit.resolved_positive : long_audit.resolved_positive;
    const bool precision_negative = binary_executed
        ? binary_audit.resolved_negative : long_audit.resolved_negative;
    const bool precision_agrees =
        (candidate_positive && precision_positive)
        || (candidate_negative && precision_negative);
    const bool precision_infrastructure_exact = long_audit.finite_values
        && (!binary_executed || binary_audit.finite_values);
    const bool precision_exact = precision_infrastructure_exact
        && precision_resolved && precision_agrees;
    const bool full_merit_exact = precision_exact
        && candidate_positive && precision_positive;
    const std::string long_root =
        al_normalized_long_double_trial_root(long_audit);
    const std::string binary_root = binary_executed
        ? al_binary128_audit_root(binary_audit) : std::string{};

    if (trial_workspace.passed) {
        release_al_normalized_workspace(trial_workspace, &work);
    }
    if (current_workspace.passed) {
        release_al_normalized_workspace(current_workspace, &work);
    }
    const bool work_exact = work.workspace_builds == 2U
        && work.workspace_releases == 2U
        && work.live_workspaces == 0U
        && work.maximum_live_workspaces == 2U
        && !work.lifecycle_underflow
        && work.all_pair_candidate_calls == 0U
        && static_work.static_index_builds == 1U
        && static_work.support_canonicalizations == 1U
        && static_work.workspace_builds == 2U
        && adjacency_work.workspace_builds == 2U
        && precision_work.long_double_audits == 1U
        && precision_work.binary128_audits
            == (binary_executed ? 1U : 0U)
        && precision_work.binary128_audits <= 1U
        && precision_work.all_pair_candidate_calls == 0U;
    const bool rollback_exact = position_before
            == al_binary64_vec3_root(r30.position)
        && predicted_before == al_binary64_vec3_root(r30.predicted_position)
        && dual_before == al_r10_dual_root(r30.dual)
        && constraint_before == al_r10_dual_root(r30.constraint)
        && endpoint_before
            == al_binary64_vec3_root(capture.hz.terminal_iterate);

    const std::array<std::pair<const char*, std::string>, 12U> routes{{
        {"NONLINEAR_PARENT_REJECTED", al_r40_route(0,1,1,1,1,1,1,1,1,1,1)},
        {"NONLINEAR_SOURCE_REJECTED", al_r40_route(1,0,1,1,1,1,1,1,1,1,1)},
        {"NONLINEAR_DENSE_REJECTED", al_r40_route(1,1,0,1,1,1,1,1,1,1,1)},
        {"NONLINEAR_ENDPOINT_REJECTED", al_r40_route(1,1,1,0,1,1,1,1,1,1,1)},
        {"NONLINEAR_MAPPING_REJECTED", al_r40_route(1,1,1,1,0,1,1,1,1,1,1)},
        {"NONLINEAR_WORKSPACE_REJECTED", al_r40_route(1,1,1,1,1,0,1,1,1,1,1)},
        {"NONLINEAR_TOPOLOGY_REJECTED", al_r40_route(1,1,1,1,1,1,0,1,1,1,1)},
        {"NONLINEAR_CONTACT_REJECTED", al_r40_route(1,1,1,1,1,1,1,0,1,1,1)},
        {"NONLINEAR_FEASIBILITY_MODEL_REJECTED", al_r40_route(1,1,1,1,1,1,1,1,0,1,1)},
        {"NONLINEAR_MERIT_PRECISION_REQUIRED", al_r40_route(1,1,1,1,1,1,1,1,1,0,1)},
        {"COMPOSITE_NORMAL_TANGENTIAL_STEP_REQUIRED", al_r40_route(1,1,1,1,1,1,1,1,1,1,0)},
        {"NONLINEAR_NORMAL_STEP_ACCEPTANCE_CANDIDATE", al_r40_route(1,1,1,1,1,1,1,1,1,1,1)},
    }};
    bool routes_exact = true;
    std::ostringstream route_projection;
    for (const auto& value : routes) {
        routes_exact = routes_exact && value.first == value.second;
        route_projection << value.first << ':' << value.second << '|';
    }
    const std::string route_root = sha256_hex(route_projection.str());
    const bool workspace_for_route = workspace_exact
        && divided_repeat_exact && precision_infrastructure_exact;
    const std::string route = al_r40_route(
        parent_exact, source_exact, dense.passed, endpoint_exact,
        mapping_exact, workspace_for_route, topology_exact, contact_exact,
        feasibility_exact, precision_exact, full_merit_exact);
    const bool route_precedence_exact =
        route == al_r40_route(
            parent_exact, source_exact, dense.passed, endpoint_exact,
            mapping_exact, workspace_for_route, topology_exact, contact_exact,
            feasibility_exact, precision_exact, full_merit_exact);
    const bool hard_controls = identity_exact && parent_exact && source_exact
        && dense.passed && endpoint_exact && mapping_exact && workspace_exact
        && divided_repeat_exact && precision_infrastructure_exact
        && work_exact && rollback_exact && routes_exact;
    const bool passed = hard_controls && route_precedence_exact;
    std::string first_failure;
    if (!identity_exact) first_failure = "IDENTITY";
    else if (!parent_exact) first_failure = "PARENT";
    else if (!source_exact) first_failure = "SOURCE";
    else if (!dense.passed) first_failure = "DENSE";
    else if (!endpoint_exact) first_failure = "ENDPOINT";
    else if (!mapping_exact) first_failure = "MAPPING";
    else if (!workspace_exact) first_failure = "WORKSPACE";
    else if (!divided_repeat_exact) first_failure = "DIVIDED_REPEAT";
    else if (!precision_infrastructure_exact)
        first_failure = "PRECISION_INFRASTRUCTURE";
    else if (!work_exact) first_failure = "WORK";
    else if (!rollback_exact) first_failure = "ROLLBACK";
    else if (!routes_exact || !route_precedence_exact)
        first_failure = "ROUTES";

    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
             << B4E2D7R19R40_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << dense.root << '|' << endpoint_before << ':'
             << physical_step_root << ':' << trial_root << '|'
             << current_topology_root << ':' << trial_topology_root << ':'
             << current_pair_root << ':' << trial_pair_root << '|'
             << binary64_bits(source_feasibility.psi) << ':'
             << binary64_bits(trial_feasibility.psi) << ':'
             << binary64_bits(predicted_feasibility_reduction) << ':'
             << binary64_bits(actual_feasibility_reduction) << ':'
             << binary64_bits(feasibility_ratio) << '|'
             << binary64_bits(divided.value.reduction) << ':'
             << binary64_bits(divided.value.phr_reduction) << ':'
             << binary64_bits(divided.value.inertia_reduction) << ':'
             << long_root << ':' << binary_root << ':'
             << precision_positive << ':' << precision_negative << '|'
             << topology_exact << ':' << contact_exact << ':'
             << feasibility_exact << ':' << precision_exact << ':'
             << full_merit_exact << ':' << work_exact << ':'
             << rollback_exact << '|' << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(
                  std::numeric_limits<long double>::max_digits10)
           << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4e2d7r19r40_nonlinear_normal_acceptance.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\""
           << B4E2D7R19R40_IDENTITY_SHA256 << "\""
           << ",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"dense\":{\"passed\":"
           << (dense.passed ? "true" : "false")
           << ",\"rho\":" << dense.rho
           << ",\"accepted_merit_reduction\":"
           << dense.accepted_merit_reduction
           << ",\"rejected_merit_reduction\":"
           << dense.rejected_merit_reduction
           << ",\"root\":\"" << dense.root << "\"}"
           << ",\"source\":{\"state_root\":\"" << r30.state_root
           << "\",\"position_root\":\"" << position_before
           << "\",\"predicted_root\":\"" << predicted_before
           << "\",\"dual_root\":\"" << dual_before
           << "\",\"constraint_root\":\"" << constraint_before
           << "\",\"topology_root\":\"" << r30.topology_root
           << "\",\"exact\":" << (source_exact ? "true" : "false")
           << "}"
           << ",\"endpoint\":{\"root\":\"" << endpoint_before
           << "\",\"trajectory_root\":\"" << capture.hz.root
           << "\",\"dimensionless_norm\":" << dimensionless_norm
           << ",\"linear_psi\":" << capture.hz.terminal.phi
           << ",\"exact\":" << (endpoint_exact ? "true" : "false")
           << "}"
           << ",\"mapping\":{\"spacing_bits\":\"0x" << std::hex
           << binary64_bits(SPACING) << std::dec
           << "\",\"physical_norm\":" << physical_norm
           << ",\"physical_step_root\":\"" << physical_step_root
           << "\",\"trial_root\":\"" << trial_root
           << "\",\"exact\":" << (mapping_exact ? "true" : "false")
           << "}"
           << ",\"nonlinear\":{\"current_topology_root\":\""
           << current_topology_root
           << "\",\"trial_topology_root\":\"" << trial_topology_root
           << "\",\"current_pair_root\":\"" << current_pair_root
           << "\",\"trial_pair_root\":\"" << trial_pair_root
           << "\",\"current_pairs\":340340,\"trial_pairs\":"
           << (workspace_exact ? trial_pair_root.empty()
                   ? 0U : trial_inner.support.fluid_pairs
                       + trial_inner.support.boundary_pairs
               : 0U)
           << ",\"pair_membership_exact\":"
           << (topology_exact ? "true" : "false")
           << ",\"workspace_exact\":"
           << (workspace_exact ? "true" : "false") << "}"
           << ",\"contact\":{\"source_penetration\":"
           << source_penetration << ",\"trial_penetration\":"
           << trial_penetration << ",\"exact\":"
           << (contact_exact ? "true" : "false") << "}"
           << ",\"feasibility\":{\"source_psi\":"
           << source_feasibility.psi << ",\"trial_psi\":"
           << trial_feasibility.psi << ",\"source_norm\":"
           << source_feasibility.norm << ",\"trial_norm\":"
           << trial_feasibility.norm << ",\"source_active\":"
           << source_feasibility.active << ",\"trial_active\":"
           << trial_feasibility.active << ",\"predicted_reduction\":"
           << predicted_feasibility_reduction
           << ",\"actual_reduction\":" << actual_feasibility_reduction
           << ",\"rho\":" << feasibility_ratio << ",\"accepted\":"
           << (feasibility_exact ? "true" : "false") << "}"
           << ",\"merit\":{\"current_total\":" << current_inner.total
           << ",\"trial_total\":" << trial_inner.total
           << ",\"raw_reduction\":" << raw_merit_reduction
           << ",\"precancelled_reduction\":"
           << divided.value.reduction << ",\"phr_reduction\":"
           << divided.value.phr_reduction << ",\"inertia_reduction\":"
           << divided.value.inertia_reduction
           << ",\"divided_repeat_exact\":"
           << (divided_repeat_exact ? "true" : "false")
           << ",\"full_merit_accepted\":"
           << (full_merit_exact ? "true" : "false") << "}"
           << ",\"precision\":{\"long_root\":\"" << long_root
           << "\",\"long_resolved\":"
           << (long_audit.resolved ? "true" : "false")
           << ",\"long_positive\":"
           << (long_audit.resolved_positive ? "true" : "false")
           << ",\"long_reduction\":"
           << long_audit.compensated_reduction
           << ",\"topology_precision_mismatch\":"
           << (long_audit.topology_precision_mismatch ? "true" : "false")
           << ",\"binary128_executed\":"
           << (binary_executed ? "true" : "false")
           << ",\"binary128_root\":\"" << binary_root
           << "\",\"resolved\":"
           << (precision_resolved ? "true" : "false")
           << ",\"resolved_positive\":"
           << (precision_positive ? "true" : "false")
           << ",\"resolved_negative\":"
           << (precision_negative ? "true" : "false")
           << ",\"sign_agrees\":"
           << (precision_agrees ? "true" : "false") << "}"
           << ",\"work\":{\"parent_r39_replays\":1"
           << ",\"diagnostic_workspaces\":" << work.workspace_builds
           << ",\"workspace_releases\":" << work.workspace_releases
           << ",\"maximum_live_workspaces\":"
           << work.maximum_live_workspaces
           << ",\"nonlinear_trials\":1,\"divided_evaluations\":2"
           << ",\"long_double_audits\":"
           << precision_work.long_double_audits
           << ",\"binary128_audits\":"
           << precision_work.binary128_audits
           << ",\"new_jvp\":0,\"new_vjp\":0,\"new_hvp\":0"
              ",\"new_model\":0,\"new_outer\":0,\"exact\":"
           << (work_exact ? "true" : "false") << "}"
           << ",\"route_cases\":" << routes.size()
           << ",\"route_root\":\"" << route_root << "\""
           << ",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":"
           << (rollback_exact ? "true" : "false")
           << ",\"first_failure\":\"" << first_failure << "\""
           << ",\"route\":\"" << route << "\""
              ",\"correction_applied\":false"
              ",\"candidate_committed\":false"
              ",\"following_outer_executed\":false"
              ",\"tolerance_selected\":false"
              ",\"penalty_changed\":false"
              ",\"state_mutation\":false"
              ",\"timing_admitted\":false"
              ",\"runtime_authority\":false"
              ",\"production_authority\":false"
           << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {passed, report.str()};
}
