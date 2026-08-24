namespace {

constexpr const char* B4E2D7R19R45_IDENTITY_SHA256 =
    "37da4b9713e015a3aabe102207ca28e1fe71835dc348041b01d5333199e07bc3";
constexpr const char* B4E2D7R19R45_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r45-nonlinear-line-globalization|v1|parent=d774b7ac:2fb4299081627e36752d13b4d3ca53046b3e19e8b72596bc318bb4ab85355292:b1485fd07b22ce57c291be9842cfcab9e3993c1ddb5aec730b732c3836c7a9a1|source=r44-route-contact-feasible-common-descent-candidate;position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;projected-endpoint-30885f22768de119065907526f36c225d058c6b97693770be109faf3ddd3b960;projected-trial-0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50;owner-baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187;superset-355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01;direction-71765e02ba9965489e79d86f3399d1c6401f25fa873d04d664146e4308d45b93;direction-norm5.4585174082622525e-9|parameter=u=d/norm2d;unit-global-l2;composite=n+alpha*u;trial=x+spacing*composite;alpha>=0|domain=minimum-of-trust,skin,contact;trust=0.25-norm2n;skin=(0.5*0.006*sqrt(1-2e-12)/spacing-max-particle-norm-n)/max-particle-norm-u;contact=exact-source-inactive-face-crossing;source-active-tangent-unbounded;alpha0=nextafter(domain,0);direct-audit|ladder=largest-first;alpha-k=ldexp(alpha0,-k);k0through24;maximum25;no-outcome-fit|prediction=unit-r44-long-double-slopes;hinge-and-merit-local-reduction-ratio>=0.1|nonlinear=fresh-exact-r<=h-mask-and-workspace-per-trial;trust;certificate;coverage;all-36000-contact-faces;local-hinge-positive;source-hinge-positive;local-merit-positive;source-merit-positive|precision=selected-binary64-precancelled-local-and-source;long-double-binary64-owned-both;binary128-conditional-on-unresolved-or-disagreement;accepted-signs-resolved-positive|selection=first-largest-all-gates;selected-independent-canonical-correspondence;fresh-selected-mapping;no-stationarity-tolerance|controls=dense-domain-trust;dense-domain-skin;dense-domain-contact;dyadic-order;local-pass-source-reject;precision-route;route-precedence;parent;source;workspace;direction;domain;trial;topology;contact;feasibility;merit;precision;correspondence;work;rollback|routes=line-globalization-parent-rejected;line-globalization-source-rejected;line-globalization-workspace-rejected;line-globalization-direction-rejected;line-globalization-domain-rejected;line-globalization-topology-rejected;line-globalization-contact-rejected;finite-common-descent-not-observed;line-globalization-precision-required;composite-merit-recovery-required;line-globalization-correspondence-rejected;line-globalization-work-rejected;nonlinear-line-globalization-candidate|precedence=parent,source,workspace,direction,domain,topology,contact,local-common-descent,precision,source-merit,correspondence,work,candidate|work=parent-r44-replays1;static-index1;superset-build1;source-and-base-workspaces2;line-trials<=25;masked-trial-workspaces<=25;selected-canonical-workspaces<=1;workspace-releases-match;maximum-live4;contact-tests<=900000;divided-evaluations<=2*trials+2;long-audits<=2*trials;binary128-audits<=2*trials;fresh-selected-vjp<=1;new-hvp0;new-model0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44=unchanged;correction=none;candidate-commit=none;following-outer=none;tolerance=none;penalty-change=none;support-expansion=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-finite-composite-line-globalization-classification-only)IDENTITY";

constexpr std::size_t AL_R45_MAX_BACKTRACKS = 24U;
constexpr double AL_R45_ACCEPTANCE_RATIO = 0.1;

struct ALR45PositiveAudit {
    bool attempted = false;
    bool confirmed = false;
    bool blocked = false;
    bool binary128_executed = false;
    long double compensated_reduction = 0.0L;
};

std::string al_r45_route(bool parent, bool source, bool workspace,
    bool direction, bool domain, bool topology, bool contact,
    bool local_common_descent, bool precision, bool source_merit,
    bool correspondence, bool work) {
    if (!parent) return "LINE_GLOBALIZATION_PARENT_REJECTED";
    if (!source) return "LINE_GLOBALIZATION_SOURCE_REJECTED";
    if (!workspace) return "LINE_GLOBALIZATION_WORKSPACE_REJECTED";
    if (!direction) return "LINE_GLOBALIZATION_DIRECTION_REJECTED";
    if (!domain) return "LINE_GLOBALIZATION_DOMAIN_REJECTED";
    if (!topology) return "LINE_GLOBALIZATION_TOPOLOGY_REJECTED";
    if (!contact) return "LINE_GLOBALIZATION_CONTACT_REJECTED";
    if (!local_common_descent) return "FINITE_COMMON_DESCENT_NOT_OBSERVED";
    if (!precision) return "LINE_GLOBALIZATION_PRECISION_REQUIRED";
    if (!source_merit) return "COMPOSITE_MERIT_RECOVERY_REQUIRED";
    if (!correspondence)
        return "LINE_GLOBALIZATION_CORRESPONDENCE_REJECTED";
    if (!work) return "LINE_GLOBALIZATION_WORK_REJECTED";
    return "NONLINEAR_LINE_GLOBALIZATION_CANDIDATE";
}

double al_r45_max_particle_norm(const std::vector<Vec3>& values) {
    double maximum = 0.0;
    for (const Vec3 value : values) maximum = std::max(maximum, norm(value));
    return maximum;
}

struct ALR45ReplayCapture {
    bool captured = false;
    ALR44ReplayCapture r44;
};

} // namespace

SplitBoundaryReport run_al_nonlinear_line_globalization_controls_impl(
    ALR45ReplayCapture* replay_capture) {
    const bool identity = sha256_hex(B4E2D7R19R45_IDENTITY_PROJECTION)
        == B4E2D7R19R45_IDENTITY_SHA256;
    ALR44ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_contact_feasible_common_descent_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "2fb4299081627e36752d13b4d3ca53046b3e19e8b72596bc318bb4ab85355292"
        && parent.json.find(
            "b1485fd07b22ce57c291be9842cfcab9e3993c1ddb5aec730b732c3836c7a9a1")
            != std::string::npos
        && parent.json.find("CONTACT_FEASIBLE_COMMON_DESCENT_CANDIDATE")
            != std::string::npos;
    ALR43ReplayCapture& r43 = capture.r43;
    ALR40ReplayCapture& r40 = r43.r42.r40;
    ALR30RangeProjectionCapture& r30 =
        r40.r39.r38.r37.r36.r35.r34.r33.r32.r31.r30;

    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string endpoint_before =
        al_binary64_vec3_root(r43.projected_endpoint);
    const std::string base_trial_before =
        al_binary64_vec3_root(r43.projected_trial);
    const std::string owner_before = al_r29_mask_root(r43.owners);
    const std::string direction_before =
        al_binary64_vec3_root(capture.direction);
    const bool source = parent_exact && r30.captured
        && position_before
            == "e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828"
        && endpoint_before
            == "30885f22768de119065907526f36c225d058c6b97693770be109faf3ddd3b960"
        && base_trial_before
            == "0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50"
        && owner_before
            == "baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187"
        && direction_before
            == "71765e02ba9965489e79d86f3399d1c6401f25fa873d04d664146e4308d45b93"
        && capture.direction_norm == 5.4585174082622525e-9;

    const auto dense_contact_bound = [](double x, double low, double high,
                                        double unit) {
        if (unit < 0.0) return (x - low) / (-SPACING * unit);
        if (unit > 0.0) return (high - x) / (SPACING * unit);
        return std::numeric_limits<double>::infinity();
    };
    const double dense_alpha = std::nextafter(0.125, 0.0);
    const bool dense = identity
        && 0.25 - 0.1 == 0.15
        && dense_contact_bound(0.5, 0.0, 1.0, 0.25) == 40.0
        && std::ldexp(dense_alpha, -1) < dense_alpha
        && std::ldexp(dense_alpha, -24) > 0.0
        && al_r45_route(true, true, true, true, true, true, true,
               true, true, false, true, true)
            == "COMPOSITE_MERIT_RECOVERY_REQUIRED";

    ALSparseWorkTrace work;
    ALSparsePrecisionWorkTrace precision_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    const SmokeFixture fixture = make_b4e2d_dam_fixture();
    const JointStaticSupportIndex index = build_joint_static_support_index(
        tagged_points(fixture.boundary), &static_work);
    const JointStaticSupportBinding binding = bind_joint_static_support_index(
        &index, index.identity_sha256);
    JointNeighborhood superset = b4ep3_build_superset(
        tagged_points(r30.position), binding);
    JointNeighborhood source_filtered = b4ep3_filter_superset(
        tagged_points(r30.position), superset);
    JointNeighborhood base_filtered = b4ep3_filter_superset(
        tagged_points(r43.projected_trial), superset);
    ALNormalizedWorkspace source_workspace =
        build_al_normalized_sparse_workspace(r30.position, binding,
            r30.dual, r30.theta, &work, &static_work, &adjacency_work);
    ALNormalizedWorkspace base_workspace =
        build_al_normalized_sparse_workspace(r43.projected_trial, binding,
            r30.dual, r30.theta, &work, &static_work, &adjacency_work);
    const ALR40HingeMetric source_metric =
        al_r40_hinge_metric(source_workspace.support.constraint);
    const ALR40HingeMetric base_metric =
        al_r40_hinge_metric(base_workspace.support.constraint);
    const bool workspace_exact = source && dense && index.passed
        && binding.passed && superset.passed
        && al_r42_superset_root(superset)
            == "355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01"
        && source_workspace.passed && base_workspace.passed
        && source_filtered.pairs == source_workspace.neighborhood.pairs
        && base_filtered.pairs == base_workspace.neighborhood.pairs
        && source_workspace.support.constraint == r30.constraint
        && source_metric.finite && base_metric.finite
        && base_metric.psi == 6.8540208964482797e-16
        && base_metric.active == 464U;

    std::vector<Vec3> unit(capture.direction.size());
    for (std::size_t i = 0U; i < unit.size(); ++i)
        unit[i] = (1.0 / capture.direction_norm) * capture.direction[i];
    const double unit_norm = al_r32_vec3_norm(unit);
    const long double merit_unit_slope = capture.merit_slope
        / static_cast<long double>(capture.direction_norm);
    const long double feasibility_unit_slope = capture.feasibility_slope
        / static_cast<long double>(capture.direction_norm);
    bool unit_contact = true;
    for (std::size_t i = 0U; i < unit.size(); ++i)
        for (int axis = 0; axis < 3; ++axis) {
            const std::uint8_t owner =
                r43.owners[3U * i + static_cast<std::size_t>(axis)];
            const double value = al_r41_component(unit[i], axis);
            unit_contact = unit_contact
                && !(owner == 1U && value < 0.0)
                && !(owner == 2U && value > 0.0);
        }
    const bool direction_exact = workspace_exact
        && al_r32_vec3_finite(unit) && std::isfinite(unit_norm)
        && relative_error(unit_norm, 1.0) <= 64.0
            * std::numeric_limits<double>::epsilon()
        && merit_unit_slope < 0.0L && feasibility_unit_slope < 0.0L
        && unit_contact;

    const double normal_norm = al_r32_vec3_norm(r43.projected_endpoint);
    const double normal_particle_max =
        al_r45_max_particle_norm(r43.projected_endpoint);
    const double unit_particle_max = al_r45_max_particle_norm(unit);
    const double trust_bound = 0.25 - normal_norm;
    const double skin_radius = 0.5 * B4EP3_SKIN
        * std::sqrt(1.0 - 2.0e-12) / SPACING;
    const double skin_bound =
        (skin_radius - normal_particle_max) / unit_particle_max;
    double contact_bound = std::numeric_limits<double>::infinity();
    bool contact_domain_exact = true;
    for (std::size_t i = 0U; i < unit.size(); ++i) {
        for (int axis = 0; axis < 3; ++axis) {
            const std::size_t component =
                3U * i + static_cast<std::size_t>(axis);
            const std::uint8_t owner = r43.owners[component];
            const double value = al_r41_component(unit[i], axis);
            if (owner != 0U) continue;
            const double x = al_r41_component(r43.projected_trial[i], axis);
            const double low = al_r41_component(fixture.contact_low, axis);
            const double high = al_r41_component(fixture.contact_high, axis);
            const double bound = dense_contact_bound(x, low, high, value);
            contact_domain_exact = contact_domain_exact
                && std::isfinite(x) && x >= low && x <= high
                && !std::isnan(bound) && bound >= 0.0;
            contact_bound = std::min(contact_bound, bound);
        }
    }
    const double domain_bound =
        std::min({trust_bound, skin_bound, contact_bound});
    const double alpha0 = std::nextafter(domain_bound, 0.0);
    const bool domain_exact = direction_exact && contact_domain_exact
        && std::isfinite(trust_bound) && trust_bound > 0.0
        && std::isfinite(skin_bound) && skin_bound > 0.0
        && contact_bound > 0.0 && std::isfinite(domain_bound)
        && domain_bound > 0.0 && alpha0 > 0.0 && alpha0 < domain_bound;

    std::size_t trials = 0U;
    std::size_t masked_workspace_builds = 0U;
    std::size_t canonical_workspace_builds = 0U;
    std::size_t contact_tests = 0U;
    std::size_t divided_evaluations = 0U;
    int selected_backtrack = -1;
    std::size_t selected_active = 0U;
    int first_local_backtrack = -1;
    int maximum_source_merit_backtrack = -1;
    bool topology_exact = domain_exact;
    bool contact_exact = domain_exact;
    bool local_common_seen = false;
    bool precision_blocked = false;
    bool source_merit_seen = false;
    bool selected = false;
    bool selected_correspondence = false;
    bool selected_repeat_exact = false;
    double selected_alpha = 0.0;
    double selected_hinge_ratio = 0.0;
    double selected_merit_ratio = 0.0;
    double selected_source_hinge_reduction = 0.0;
    double selected_source_merit_reduction = 0.0;
    double selected_mapping_norm = 0.0;
    double first_local_source_merit_reduction = 0.0;
    double maximum_source_merit_reduction =
        -std::numeric_limits<double>::infinity();
    std::string selected_endpoint_root;
    std::string selected_physical_root;
    std::string selected_trial_root;
    std::string selected_mask_root;
    std::string selected_mapping_root;
    std::ostringstream trial_material;

    const auto audit_positive = [&](const std::vector<Vec3>& current_position,
                                    const std::vector<Vec3>& trial_position,
                                    double binary64_reduction) {
        ALR45PositiveAudit result;
        result.attempted = binary64_reduction > 0.0;
        if (!result.attempted) return result;
        const LongDoubleTrialEnergyAudit long_audit =
            audit_al_normalized_sparse_long_double(current_position,
                trial_position, r30.predicted_position, r30.dual, r30.theta,
                binding, &precision_work,
                ALPrecisionMembershipPolicy::Binary64Owned);
        result.compensated_reduction = long_audit.compensated_reduction;
        const bool disagreement = long_audit.resolved
            && !long_audit.resolved_positive;
        result.binary128_executed = !long_audit.resolved || disagreement;
        if (result.binary128_executed) {
            const Binary128TrialEnergyAudit binary_audit =
                audit_al_normalized_sparse_binary128(current_position,
                    trial_position, r30.predicted_position, r30.dual,
                    r30.theta, binding, binary64_reduction, &precision_work,
                    ALPrecisionMembershipPolicy::Binary64Owned);
            result.confirmed = long_audit.finite_values
                && binary_audit.finite_values && binary_audit.resolved
                && binary_audit.resolved_positive;
        } else {
            result.confirmed = long_audit.finite_values
                && long_audit.resolved_positive;
        }
        result.blocked = !result.confirmed;
        return result;
    };

    if (domain_exact) {
        for (std::size_t backtrack = 0U;
             backtrack <= AL_R45_MAX_BACKTRACKS; ++backtrack) {
            const double alpha = std::ldexp(alpha0,
                -static_cast<int>(backtrack));
            ++trials;
            std::vector<Vec3> endpoint(r43.projected_endpoint.size());
            std::vector<Vec3> physical(endpoint.size());
            std::vector<Vec3> trial(endpoint.size());
            for (std::size_t i = 0U; i < endpoint.size(); ++i) {
                endpoint[i] = r43.projected_endpoint[i] + alpha * unit[i];
                physical[i] = SPACING * endpoint[i];
                trial[i] = r30.position[i] + physical[i];
            }
            const double endpoint_norm = al_r32_vec3_norm(endpoint);
            double maximum_displacement_squared = 0.0;
            const bool trust = al_r32_vec3_finite(endpoint)
                && std::isfinite(endpoint_norm) && endpoint_norm <= 0.25;
            const bool certificate = b4ep3_max_displacement_squared(
                    tagged_points(r30.position), tagged_points(trial),
                    maximum_displacement_squared)
                && b4ep3_certificate(maximum_displacement_squared);
            JointNeighborhood filtered = b4ep3_filter_superset(
                tagged_points(trial), superset);
            const ALR42Coverage coverage = al_r42_coverage(superset, filtered);
            const bool topology = trust && certificate && filtered.passed
                && coverage.exact;
            topology_exact = topology_exact && topology;
            const ALR41ContactAudit contact_audit =
                al_r41_contact_audit(r30.position, trial, fixture);
            contact_tests += contact_audit.tests;
            const bool contact = contact_audit.finite
                && contact_audit.tests == 36000U
                && contact_audit.newly_penetrating == 0U
                && contact_audit.worsened == 0U
                && contact_audit.trial_maximum
                    <= contact_audit.source_maximum;
            contact_exact = contact_exact && contact;
            if (!topology || !contact) {
                trial_material << backtrack << ':' << binary64_bits(alpha)
                               << ":BOUNDARY_REJECT;";
                break;
            }

            const std::string mask_root = coverage.root;
            ALNormalizedWorkspace candidate =
                build_al_normalized_workspace_from_neighborhood(
                    std::move(filtered), r30.dual, r30.theta, &work);
            ++masked_workspace_builds;
            const ALR40HingeMetric candidate_metric =
                al_r40_hinge_metric(candidate.support.constraint);
            const double predicted_hinge = static_cast<double>(
                -static_cast<long double>(alpha) * feasibility_unit_slope);
            const double actual_hinge = base_metric.psi - candidate_metric.psi;
            const double source_hinge =
                source_metric.psi - candidate_metric.psi;
            const double hinge_ratio = actual_hinge / predicted_hinge;
            const ALNormalizedPrecancellationDivided local_divided =
                evaluate_al_normalized_sparse_precancelled_divided(
                    base_workspace, candidate, r30.predicted_position,
                    r30.dual, r30.theta, &work);
            const ALNormalizedPrecancellationDivided source_divided =
                evaluate_al_normalized_sparse_precancelled_divided(
                    source_workspace, candidate, r30.predicted_position,
                    r30.dual, r30.theta, &work);
            divided_evaluations += 2U;
            const double predicted_merit = static_cast<double>(
                -static_cast<long double>(alpha) * merit_unit_slope);
            const double merit_ratio =
                local_divided.value.reduction / predicted_merit;
            if (source_divided.value.finite_values
                && source_divided.value.reduction
                    > maximum_source_merit_reduction) {
                maximum_source_merit_reduction =
                    source_divided.value.reduction;
                maximum_source_merit_backtrack =
                    static_cast<int>(backtrack);
            }
            const bool local_numeric = candidate.passed
                && candidate_metric.finite && predicted_hinge > 0.0
                && actual_hinge > 0.0 && source_hinge > 0.0
                && std::isfinite(hinge_ratio)
                && hinge_ratio >= AL_R45_ACCEPTANCE_RATIO
                && local_divided.value.finite_values
                && predicted_merit > 0.0
                && local_divided.value.reduction > 0.0
                && std::isfinite(merit_ratio)
                && merit_ratio >= AL_R45_ACCEPTANCE_RATIO;
            ALR45PositiveAudit local_audit;
            ALR45PositiveAudit source_audit;
            if (local_numeric) {
                local_audit = audit_positive(r43.projected_trial, trial,
                    local_divided.value.reduction);
                precision_blocked = precision_blocked || local_audit.blocked;
                if (local_audit.confirmed) {
                    local_common_seen = true;
                    if (first_local_backtrack < 0) {
                        first_local_backtrack = static_cast<int>(backtrack);
                        first_local_source_merit_reduction =
                            source_divided.value.reduction;
                    }
                }
                if (local_audit.confirmed
                    && source_divided.value.finite_values
                    && source_divided.value.reduction > 0.0) {
                    source_audit = audit_positive(r30.position, trial,
                        source_divided.value.reduction);
                    precision_blocked =
                        precision_blocked || source_audit.blocked;
                    source_merit_seen =
                        source_merit_seen || source_audit.confirmed;
                }
            }
            trial_material << backtrack << ':' << binary64_bits(alpha) << ':'
                           << endpoint_norm << ':' << candidate_metric.psi
                           << ':' << hinge_ratio << ':' << merit_ratio << ':'
                           << source_divided.value.reduction << ':'
                           << local_audit.confirmed << ':'
                           << source_audit.confirmed << ':' << mask_root << ';';

            if (local_numeric && local_audit.confirmed
                && source_audit.confirmed) {
                const ALNormalizedPrecancellationDivided local_repeat =
                    evaluate_al_normalized_sparse_precancelled_divided(
                        base_workspace, candidate, r30.predicted_position,
                        r30.dual, r30.theta, &work);
                const ALNormalizedPrecancellationDivided source_repeat =
                    evaluate_al_normalized_sparse_precancelled_divided(
                        source_workspace, candidate, r30.predicted_position,
                        r30.dual, r30.theta, &work);
                divided_evaluations += 2U;
                selected_repeat_exact =
                    al_normalized_precancellation_exact(
                        local_divided, local_repeat)
                    && al_normalized_precancellation_exact(
                        source_divided, source_repeat);
                ALNormalizedWorkspace canonical =
                    build_al_normalized_sparse_workspace(trial, binding,
                        r30.dual, r30.theta, &work, &static_work,
                        &adjacency_work);
                ++canonical_workspace_builds;
                selected_correspondence = selected_repeat_exact
                    && al_r42_workspace_exact(canonical, candidate);
                std::vector<double> positive(
                    candidate.support.constraint.size());
                for (std::size_t i = 0U; i < positive.size(); ++i)
                    positive[i] =
                        std::max(0.0, candidate.support.constraint[i]);
                const std::vector<Vec3> mapping =
                    al_r29_pair_once_vjp(candidate, positive);
                selected_mapping_norm = al_r32_vec3_norm(mapping);
                selected_mapping_root = al_binary64_vec3_root(mapping);
                selected = selected_correspondence
                    && std::isfinite(selected_mapping_norm);
                selected_backtrack = static_cast<int>(backtrack);
                selected_alpha = alpha;
                selected_active = candidate_metric.active;
                selected_hinge_ratio = hinge_ratio;
                selected_merit_ratio = merit_ratio;
                selected_source_hinge_reduction = source_hinge;
                selected_source_merit_reduction =
                    source_divided.value.reduction;
                selected_endpoint_root = al_binary64_vec3_root(endpoint);
                selected_physical_root = al_binary64_vec3_root(physical);
                selected_trial_root = al_binary64_vec3_root(trial);
                selected_mask_root = mask_root;
                if (canonical.passed)
                    release_al_normalized_workspace(canonical, &work);
                if (candidate.passed)
                    release_al_normalized_workspace(candidate, &work);
                break;
            }
            if (candidate.passed)
                release_al_normalized_workspace(candidate, &work);
        }
    }

    if (base_workspace.passed)
        release_al_normalized_workspace(base_workspace, &work);
    if (source_workspace.passed)
        release_al_normalized_workspace(source_workspace, &work);
    const bool precision_exact = !precision_blocked;
    const bool correspondence_exact = !source_merit_seen
        || (selected && selected_correspondence && selected_repeat_exact);
    const std::size_t expected_workspace_builds = 2U
        + masked_workspace_builds + canonical_workspace_builds;
    const bool work_exact = trials <= AL_R45_MAX_BACKTRACKS + 1U
        && masked_workspace_builds <= trials
        && canonical_workspace_builds <= 1U
        && contact_tests == 36000U * trials
        && divided_evaluations <= 2U * trials + 2U
        && precision_work.long_double_audits <= 2U * trials
        && precision_work.binary128_audits <= 2U * trials
        && work.workspace_builds == expected_workspace_builds
        && work.workspace_releases == expected_workspace_builds
        && work.live_workspaces == 0U
        && work.maximum_live_workspaces <= 4U
        && !work.lifecycle_underflow
        && static_work.static_index_builds == 1U
        && static_work.support_canonicalizations == 1U
        && static_work.workspace_builds == 2U + canonical_workspace_builds
        && adjacency_work.workspace_builds == 2U + canonical_workspace_builds
        && work.all_pair_candidate_calls == 0U
        && precision_work.all_pair_candidate_calls == 0U;
    const bool rollback = position_before == al_binary64_vec3_root(r30.position)
        && endpoint_before == al_binary64_vec3_root(r43.projected_endpoint)
        && base_trial_before == al_binary64_vec3_root(r43.projected_trial)
        && owner_before == al_r29_mask_root(r43.owners)
        && direction_before == al_binary64_vec3_root(capture.direction);

    const std::array<const char*, 13U> expected_routes{{
        "LINE_GLOBALIZATION_PARENT_REJECTED",
        "LINE_GLOBALIZATION_SOURCE_REJECTED",
        "LINE_GLOBALIZATION_WORKSPACE_REJECTED",
        "LINE_GLOBALIZATION_DIRECTION_REJECTED",
        "LINE_GLOBALIZATION_DOMAIN_REJECTED",
        "LINE_GLOBALIZATION_TOPOLOGY_REJECTED",
        "LINE_GLOBALIZATION_CONTACT_REJECTED",
        "FINITE_COMMON_DESCENT_NOT_OBSERVED",
        "LINE_GLOBALIZATION_PRECISION_REQUIRED",
        "COMPOSITE_MERIT_RECOVERY_REQUIRED",
        "LINE_GLOBALIZATION_CORRESPONDENCE_REJECTED",
        "LINE_GLOBALIZATION_WORK_REJECTED",
        "NONLINEAR_LINE_GLOBALIZATION_CANDIDATE"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t index_value = 0U;
         index_value < expected_routes.size(); ++index_value) {
        std::array<bool, 12U> gates{};
        gates.fill(true);
        if (index_value < gates.size()) gates[index_value] = false;
        const std::string observed = al_r45_route(gates[0], gates[1],
            gates[2], gates[3], gates[4], gates[5], gates[6], gates[7],
            gates[8], gates[9], gates[10], gates[11]);
        routes_exact = routes_exact && observed == expected_routes[index_value];
        route_material << expected_routes[index_value] << ':' << observed
                       << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r45_route(parent_exact, source,
        workspace_exact, direction_exact, domain_exact, topology_exact,
        contact_exact, local_common_seen, precision_exact, source_merit_seen,
        correspondence_exact, work_exact && rollback);
    const bool passed = identity && parent_exact && source && dense
        && workspace_exact && direction_exact && routes_exact
        && work_exact && rollback;
    const std::string unit_root = al_binary64_vec3_root(unit);
    const std::string trial_root = sha256_hex(trial_material.str());
    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R45_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << unit_root << ':' << unit_norm << '|'
             << trust_bound << ':' << skin_bound << ':' << contact_bound << ':'
             << domain_bound << ':' << alpha0 << '|'
             << trials << ':' << trial_root << ':' << selected << ':'
             << selected_backtrack << ':' << selected_alpha << '|'
             << selected_endpoint_root << ':' << selected_trial_root << ':'
             << selected_mask_root << ':' << selected_mapping_root << '|'
             << selected_hinge_ratio << ':' << selected_merit_ratio << ':'
             << selected_source_hinge_reduction << ':'
             << selected_source_merit_reduction << ':'
             << first_local_backtrack << ':'
             << first_local_source_merit_reduction << ':'
             << maximum_source_merit_backtrack << ':'
             << maximum_source_merit_reduction << '|'
             << precision_work.long_double_audits << ':'
             << precision_work.binary128_audits << ':' << work_exact << ':'
             << rollback << ':' << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r45_nonlinear_line_globalization.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R45_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"direction\":{\"unit_norm\":" << unit_norm
           << ",\"unit_root\":\"" << unit_root
           << "\",\"merit_unit_slope\":"
           << static_cast<double>(merit_unit_slope)
           << ",\"hinge_unit_slope\":"
           << static_cast<double>(feasibility_unit_slope) << "}"
           << ",\"domain\":{\"trust\":" << trust_bound
           << ",\"skin\":" << skin_bound << ",\"contact\":"
           << contact_bound << ",\"selected_bound\":" << domain_bound
           << ",\"alpha0\":" << alpha0 << "}"
           << ",\"line\":{\"trials\":" << trials
           << ",\"maximum\":" << AL_R45_MAX_BACKTRACKS + 1U
           << ",\"trial_root\":\"" << trial_root
           << "\",\"selected\":" << (selected ? "true" : "false")
           << ",\"backtrack\":" << selected_backtrack
           << ",\"alpha\":" << selected_alpha
           << ",\"active\":" << selected_active
           << ",\"hinge_ratio\":" << selected_hinge_ratio
           << ",\"merit_ratio\":" << selected_merit_ratio
           << ",\"source_hinge_reduction\":"
           << selected_source_hinge_reduction
           << ",\"source_merit_reduction\":"
           << selected_source_merit_reduction
           << ",\"first_local_backtrack\":" << first_local_backtrack
           << ",\"first_local_source_merit_reduction\":"
           << first_local_source_merit_reduction
           << ",\"maximum_source_merit_backtrack\":"
           << maximum_source_merit_backtrack
           << ",\"maximum_source_merit_reduction\":"
           << maximum_source_merit_reduction << "}"
           << ",\"selected_roots\":{\"endpoint\":\""
           << selected_endpoint_root << "\",\"physical\":\""
           << selected_physical_root << "\",\"trial\":\""
           << selected_trial_root << "\",\"mask\":\""
           << selected_mask_root << "\",\"mapping\":\""
           << selected_mapping_root << "\",\"mapping_norm\":"
           << selected_mapping_norm << "}"
           << ",\"precision\":{\"long_double_audits\":"
           << precision_work.long_double_audits
           << ",\"binary128_audits\":"
           << precision_work.binary128_audits
           << ",\"blocked\":" << (precision_blocked ? "true" : "false")
           << "},\"work\":{\"masked_workspaces\":"
           << masked_workspace_builds << ",\"canonical_workspaces\":"
           << canonical_workspace_builds << ",\"divided_evaluations\":"
           << divided_evaluations << ",\"contact_tests\":" << contact_tests
           << ",\"maximum_live\":" << work.maximum_live_workspaces
           << ",\"exact\":" << (work_exact ? "true" : "false") << "}"
           << ",\"route_cases\":" << expected_routes.size()
           << ",\"route_root\":\"" << route_root
           << "\",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback ? "true" : "false")
           << ",\"route\":\"" << route
           << "\",\"correction_applied\":false,\"candidate_committed\":false"
              ",\"following_outer_executed\":false,\"timing_admitted\":false"
              ",\"runtime_authority\":false,\"production_authority\":false"
              ",\"result_sha256\":\""
           << result_sha << "\"}";
    if (replay_capture != nullptr) {
        replay_capture->captured = passed;
        replay_capture->r44 = std::move(capture);
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_nonlinear_line_globalization_controls() {
    return run_al_nonlinear_line_globalization_controls_impl(nullptr);
}
