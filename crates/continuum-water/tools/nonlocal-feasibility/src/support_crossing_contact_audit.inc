namespace {

constexpr const char* B4E2D7R19R41_IDENTITY_SHA256 =
    "7abcc0e2a406580265a84dec051fcb5745ccf1cbb76c9cfd5ac57136fc32a3a2";
constexpr const char* B4E2D7R19R41_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r41-support-crossing-contact-audit|v1|parent=52eb514d:6db946e6c6e327520b60f98d34e60f346f0b02bfe84e88767fc1e36d7db9489d:d8ea97108543603e511c68632a8993eb010f0a7f8342f034a55c8d53555e98ff|source=r40-route-nonlinear-topology-rejected;position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;trial-60e793aff919fada6b49c1b98be7ee5cc4a2c74a4254fa762cdece5700e1e804;endpoint-a73caf5baa6708ca89693077bad943dcdbf1f5069d50270ccc0f6b184a102e3a;physical-step-424c84b72409f5b6665874f17afc18cf2784ae84593e30be41eaeec0d25e514b;current-topology-070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5;trial-topology-25d6148fc3f460ba6c324a539499deebabb86e88267ab6bdf3fe700dc44c7402;current-pairs-58ce6532f180f9f2803a069f8cfeb25278fbc3930796d7cf29700db08ce4fac3;trial-pairs-2e74673c3ae59c099c91902d7eac31b97b1c5a9496497128174d9a899f2bae06|crossings=exact-sorted-pair-symmetric-difference;lost;entered;fluid-fluid;fluid-support;source-and-trial-radius-bits;signed-horizon-margin;binary64-ulp-distance;weight;gradient;second;density-delta;record-root|shell=material-zero-only-if-both-state-weight-gradient-second-exact-zero-and-crossing-density-delta-exact-zero;no-fitted-tolerance|contact=all-particle-axis-lower-upper;source-and-trial-penetration;new;resolved;worsened;normal-step-component;worst-owner;record-root|selection=nonzero-support-crossing-first;new-contact-second;existing-contact-inward-third;material-topology-equivalence-last|controls=exact-horizon-zero;interior-to-outside-nonzero;existing-contact-worsened;new-contact;route-precedence;parent;source;mapping;workspace;crossing-partition;kernel-fold;density-ledger;contact-ledger;work;rollback|routes=crossing-parent-rejected;crossing-source-rejected;crossing-dense-rejected;crossing-workspace-rejected;crossing-partition-rejected;crossing-kernel-rejected;crossing-contact-ledger-rejected;crossing-work-rejected;nonzero-support-crossing-requires-relinearization;new-contact-crossing-requires-contact-solve;contact-tangent-normal-step-required;material-topology-equivalence-candidate|precedence=parent,source,dense,workspace,partition,kernel-ledger,contact-ledger,work,nonzero-crossing,new-contact,existing-contact-inward,material-equivalence|work=parent-r40-replays1;diagnostic-workspaces2;releases2;maximum-live2;crossing-merges1;contact-particle-axis-tests36000;new-jvp0;new-vjp0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-gates=unchanged;correction=none;candidate-commit=none;contact-projection=none;support-expansion=none;tolerance=none;public-state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-support-crossing-contact-classification-only)IDENTITY";

struct ALR41DenseControls {
    bool passed = false;
    std::string root;
};

struct ALR41CrossingAudit {
    bool finite = false;
    bool partition_exact = false;
    bool member_radius_exact = false;
    bool material_zero = false;
    std::size_t intersection = 0U;
    std::size_t lost = 0U;
    std::size_t entered = 0U;
    std::size_t lost_fluid = 0U;
    std::size_t lost_support = 0U;
    std::size_t entered_fluid = 0U;
    std::size_t entered_support = 0U;
    std::uint64_t maximum_ulp_distance = 0U;
    double maximum_abs_margin = 0.0;
    double maximum_abs_weight = 0.0;
    double maximum_abs_gradient = 0.0;
    double maximum_abs_second = 0.0;
    double maximum_abs_density_delta = 0.0;
    std::string density_root;
    std::string record_root;
    std::string root;
};

struct ALR41ContactAudit {
    bool finite = false;
    std::size_t tests = 0U;
    std::size_t source_penetrating = 0U;
    std::size_t trial_penetrating = 0U;
    std::size_t newly_penetrating = 0U;
    std::size_t resolved = 0U;
    std::size_t worsened = 0U;
    std::size_t existing_inward = 0U;
    double source_maximum = 0.0;
    double trial_maximum = 0.0;
    double maximum_increase = 0.0;
    std::size_t worst_particle = 0U;
    int worst_axis = 0;
    int worst_side = 0;
    std::string root;
};

double al_r41_component(Vec3 value, int axis) {
    return axis == 0 ? value.x : (axis == 1 ? value.y : value.z);
}

std::uint64_t al_r41_positive_ulp_distance(double lhs, double rhs) {
    if (!std::isfinite(lhs) || !std::isfinite(rhs)
        || lhs < 0.0 || rhs < 0.0) {
        return std::numeric_limits<std::uint64_t>::max();
    }
    const std::uint64_t a = binary64_bits(lhs);
    const std::uint64_t b = binary64_bits(rhs);
    return a >= b ? a - b : b - a;
}

ALR41DenseControls al_r41_dense_controls() {
    const double above = std::nextafter(
        HORIZON, std::numeric_limits<double>::infinity());
    const double interior = 0.99 * HORIZON;
    const bool horizon_zero = weight(HORIZON) == 0.0
        && weight_gradient(HORIZON) == 0.0
        && weight_second(HORIZON) == 0.0
        && weight(above) == 0.0
        && weight_gradient(above) == 0.0
        && weight_second(above) == 0.0;
    const bool interior_nonzero = weight(interior) != 0.0
        || weight_gradient(interior) != 0.0
        || weight_second(interior) != 0.0;
    constexpr double low = 0.025;
    const double existing = low - 1.0e-8;
    const double worsened = existing - 1.0e-9;
    const double outside = low + 1.0e-9;
    const double newly = outside - 2.0e-9;
    const bool existing_control = low - existing > 0.0
        && low - worsened > low - existing;
    const bool new_control = low - outside <= 0.0
        && low - newly > 0.0;
    ALR41DenseControls result;
    result.passed = horizon_zero && interior_nonzero
        && existing_control && new_control;
    std::ostringstream projection;
    projection << binary64_bits(HORIZON) << ':' << binary64_bits(above)
               << ':' << binary64_bits(interior) << ':'
               << binary64_bits(weight(HORIZON)) << ':'
               << binary64_bits(weight_gradient(HORIZON)) << ':'
               << binary64_bits(weight_second(HORIZON)) << ':'
               << binary64_bits(weight(interior)) << ':'
               << binary64_bits(weight_gradient(interior)) << ':'
               << binary64_bits(weight_second(interior)) << ':'
               << existing_control << ':' << new_control << ':'
               << result.passed;
    result.root = sha256_hex(projection.str());
    return result;
}

double al_r41_pair_radius(
    const JointNeighborhood& neighborhood,
    const JointPair& pair) {
    if (pair.fluid >= neighborhood.fluid.size()
        || pair.participant >= neighborhood.fluid.size()
            + neighborhood.support.size()) {
        return std::numeric_limits<double>::quiet_NaN();
    }
    return norm(neighborhood.fluid[pair.fluid].position
        - joint_position(neighborhood, pair.participant));
}

ALR41CrossingAudit al_r41_crossing_audit(
    const ALNormalizedWorkspace& current,
    const ALNormalizedWorkspace& trial) {
    ALR41CrossingAudit result;
    const std::size_t count = current.neighborhood.fluid.size();
    if (!current.passed || !trial.passed
        || trial.neighborhood.fluid.size() != count
        || current.neighborhood.support.size()
            != trial.neighborhood.support.size()) {
        return result;
    }
    std::vector<double> density_delta(count, 0.0);
    std::ostringstream records;
    std::size_t current_index = 0U;
    std::size_t trial_index = 0U;
    bool finite_values = true;
    bool member_radius_exact = true;
    bool every_kernel_zero = true;
    while (current_index < current.neighborhood.pairs.size()
        || trial_index < trial.neighborhood.pairs.size()) {
        const bool have_current =
            current_index < current.neighborhood.pairs.size();
        const bool have_trial =
            trial_index < trial.neighborhood.pairs.size();
        const bool current_before = have_current
            && (!have_trial || al_joint_pair_less(
                current.neighborhood.pairs[current_index],
                trial.neighborhood.pairs[trial_index]));
        const bool trial_before = have_trial
            && (!have_current || al_joint_pair_less(
                trial.neighborhood.pairs[trial_index],
                current.neighborhood.pairs[current_index]));
        if (!current_before && !trial_before) {
            ++result.intersection;
            ++current_index;
            ++trial_index;
            continue;
        }
        const bool current_member = current_before;
        const JointPair pair = current_member
            ? current.neighborhood.pairs[current_index]
            : trial.neighborhood.pairs[trial_index];
        const double current_radius =
            al_r41_pair_radius(current.neighborhood, pair);
        const double trial_radius =
            al_r41_pair_radius(trial.neighborhood, pair);
        if (current_member) {
            member_radius_exact = member_radius_exact
                && binary64_bits(current_radius)
                    == binary64_bits(current.radius[current_index]);
            ++result.lost;
            if (pair.participant < count) ++result.lost_fluid;
            else ++result.lost_support;
            ++current_index;
        } else {
            member_radius_exact = member_radius_exact
                && binary64_bits(trial_radius)
                    == binary64_bits(trial.radius[trial_index]);
            ++result.entered;
            if (pair.participant < count) ++result.entered_fluid;
            else ++result.entered_support;
            ++trial_index;
        }
        const double current_weight = weight(current_radius);
        const double trial_weight = weight(trial_radius);
        const double current_gradient = weight_gradient(current_radius);
        const double trial_gradient = weight_gradient(trial_radius);
        const double current_second = weight_second(current_radius);
        const double trial_second = weight_second(trial_radius);
        const double delta = MASS * (trial_weight - current_weight);
        density_delta[pair.fluid] += delta;
        if (pair.participant < count) {
            density_delta[pair.participant] += delta;
        }
        const double current_margin = HORIZON - current_radius;
        const double trial_margin = HORIZON - trial_radius;
        result.maximum_ulp_distance = std::max({
            result.maximum_ulp_distance,
            al_r41_positive_ulp_distance(current_radius, HORIZON),
            al_r41_positive_ulp_distance(trial_radius, HORIZON)});
        result.maximum_abs_margin = std::max({result.maximum_abs_margin,
            std::abs(current_margin), std::abs(trial_margin)});
        result.maximum_abs_weight = std::max({result.maximum_abs_weight,
            std::abs(current_weight), std::abs(trial_weight)});
        result.maximum_abs_gradient = std::max({
            result.maximum_abs_gradient, std::abs(current_gradient),
            std::abs(trial_gradient)});
        result.maximum_abs_second = std::max({result.maximum_abs_second,
            std::abs(current_second), std::abs(trial_second)});
        every_kernel_zero = every_kernel_zero
            && current_weight == 0.0 && trial_weight == 0.0
            && current_gradient == 0.0 && trial_gradient == 0.0
            && current_second == 0.0 && trial_second == 0.0;
        finite_values = finite_values
            && std::isfinite(current_radius) && std::isfinite(trial_radius)
            && std::isfinite(current_weight) && std::isfinite(trial_weight)
            && std::isfinite(current_gradient)
            && std::isfinite(trial_gradient)
            && std::isfinite(current_second) && std::isfinite(trial_second)
            && std::isfinite(delta);
        records << pair.fluid << ':' << pair.participant << ':'
                << current_member << ':' << binary64_bits(current_radius)
                << ':' << binary64_bits(trial_radius) << ':'
                << binary64_bits(current_margin) << ':'
                << binary64_bits(trial_margin) << ':'
                << al_r41_positive_ulp_distance(current_radius, HORIZON)
                << ':' << al_r41_positive_ulp_distance(
                    trial_radius, HORIZON)
                << ':' << binary64_bits(current_weight) << ':'
                << binary64_bits(trial_weight) << ':'
                << binary64_bits(current_gradient) << ':'
                << binary64_bits(trial_gradient) << ':'
                << binary64_bits(current_second) << ':'
                << binary64_bits(trial_second) << ':'
                << binary64_bits(delta) << '|';
    }
    bool density_zero = true;
    for (double value : density_delta) {
        finite_values = finite_values && std::isfinite(value);
        density_zero = density_zero && value == 0.0;
        result.maximum_abs_density_delta = std::max(
            result.maximum_abs_density_delta, std::abs(value));
    }
    result.partition_exact = result.intersection + result.lost
            == current.neighborhood.pairs.size()
        && result.intersection + result.entered
            == trial.neighborhood.pairs.size()
        && result.lost == result.lost_fluid + result.lost_support
        && result.entered == result.entered_fluid + result.entered_support;
    result.member_radius_exact = member_radius_exact;
    result.finite = finite_values;
    result.material_zero = result.finite && result.partition_exact
        && result.member_radius_exact && every_kernel_zero && density_zero;
    result.density_root = al_r10_dual_root(density_delta);
    result.record_root = sha256_hex(records.str());
    std::ostringstream projection;
    projection << result.finite << ':' << result.partition_exact << ':'
               << result.member_radius_exact << ':' << result.material_zero
               << '|' << result.intersection << ':' << result.lost << ':'
               << result.entered << ':' << result.lost_fluid << ':'
               << result.lost_support << ':' << result.entered_fluid << ':'
               << result.entered_support << '|'
               << result.maximum_ulp_distance << ':'
               << binary64_bits(result.maximum_abs_margin) << ':'
               << binary64_bits(result.maximum_abs_weight) << ':'
               << binary64_bits(result.maximum_abs_gradient) << ':'
               << binary64_bits(result.maximum_abs_second) << ':'
               << binary64_bits(result.maximum_abs_density_delta) << '|'
               << result.density_root << ':' << result.record_root;
    result.root = sha256_hex(projection.str());
    return result;
}

ALR41ContactAudit al_r41_contact_audit(
    const std::vector<Vec3>& current,
    const std::vector<Vec3>& trial,
    const SmokeFixture& fixture) {
    ALR41ContactAudit result;
    if (current.size() != trial.size()) return result;
    std::ostringstream projection;
    bool finite_values = true;
    for (std::size_t particle = 0U; particle < current.size(); ++particle) {
        for (int axis = 0; axis < 3; ++axis) {
            const double current_value = al_r41_component(current[particle], axis);
            const double trial_value = al_r41_component(trial[particle], axis);
            const double step = trial_value - current_value;
            const double low = al_r41_component(fixture.contact_low, axis);
            const double high = al_r41_component(fixture.contact_high, axis);
            for (int side = 0; side < 2; ++side) {
                const double source_penetration = side == 0
                    ? std::max(0.0, low - current_value)
                    : std::max(0.0, current_value - high);
                const double trial_penetration = side == 0
                    ? std::max(0.0, low - trial_value)
                    : std::max(0.0, trial_value - high);
                const double inward = side == 0 ? -step : step;
                ++result.tests;
                result.source_penetrating += source_penetration > 0.0 ? 1U : 0U;
                result.trial_penetrating += trial_penetration > 0.0 ? 1U : 0U;
                result.newly_penetrating += source_penetration == 0.0
                        && trial_penetration > 0.0 ? 1U : 0U;
                result.resolved += source_penetration > 0.0
                        && trial_penetration == 0.0 ? 1U : 0U;
                const bool worsened = source_penetration > 0.0
                    && trial_penetration > source_penetration;
                result.worsened += worsened ? 1U : 0U;
                result.existing_inward += source_penetration > 0.0
                        && inward > 0.0 ? 1U : 0U;
                result.source_maximum = std::max(
                    result.source_maximum, source_penetration);
                if (trial_penetration > result.trial_maximum) {
                    result.trial_maximum = trial_penetration;
                    result.worst_particle = particle;
                    result.worst_axis = axis;
                    result.worst_side = side;
                }
                result.maximum_increase = std::max(result.maximum_increase,
                    trial_penetration - source_penetration);
                finite_values = finite_values
                    && std::isfinite(source_penetration)
                    && std::isfinite(trial_penetration)
                    && std::isfinite(inward);
                projection << particle << ':' << axis << ':' << side << ':'
                           << binary64_bits(source_penetration) << ':'
                           << binary64_bits(trial_penetration) << ':'
                           << binary64_bits(inward) << ':' << worsened << '|';
            }
        }
    }
    result.finite = finite_values && result.tests == current.size() * 6U;
    projection << result.tests << ':' << result.source_penetrating << ':'
               << result.trial_penetrating << ':' << result.newly_penetrating
               << ':' << result.resolved << ':' << result.worsened << ':'
               << result.existing_inward << ':'
               << binary64_bits(result.source_maximum) << ':'
               << binary64_bits(result.trial_maximum) << ':'
               << binary64_bits(result.maximum_increase) << ':'
               << result.worst_particle << ':' << result.worst_axis << ':'
               << result.worst_side << ':' << result.finite;
    result.root = sha256_hex(projection.str());
    return result;
}

std::string al_r41_route(
    bool parent, bool source, bool dense, bool workspace, bool partition,
    bool kernel, bool contact, bool work, bool nonzero_crossing,
    bool new_contact, bool existing_inward) {
    if (!parent) return "CROSSING_PARENT_REJECTED";
    if (!source) return "CROSSING_SOURCE_REJECTED";
    if (!dense) return "CROSSING_DENSE_REJECTED";
    if (!workspace) return "CROSSING_WORKSPACE_REJECTED";
    if (!partition) return "CROSSING_PARTITION_REJECTED";
    if (!kernel) return "CROSSING_KERNEL_REJECTED";
    if (!contact) return "CROSSING_CONTACT_LEDGER_REJECTED";
    if (!work) return "CROSSING_WORK_REJECTED";
    if (nonzero_crossing)
        return "NONZERO_SUPPORT_CROSSING_REQUIRES_RELINEARIZATION";
    if (new_contact)
        return "NEW_CONTACT_CROSSING_REQUIRES_CONTACT_SOLVE";
    if (existing_inward)
        return "CONTACT_TANGENT_NORMAL_STEP_REQUIRED";
    return "MATERIAL_TOPOLOGY_EQUIVALENCE_CANDIDATE";
}

} // namespace

SplitBoundaryReport run_al_support_crossing_contact_audit_controls_impl(
    ALR40ReplayCapture* replay_capture) {
    const bool identity_exact = sha256_hex(
        B4E2D7R19R41_IDENTITY_PROJECTION)
        == B4E2D7R19R41_IDENTITY_SHA256;
    ALR40ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_nonlinear_normal_acceptance_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "d8ea97108543603e511c68632a8993eb010f0a7f8342f034a55c8d53555e98ff"
        "\"") != std::string::npos;
    const bool parent_route = parent.json.find(
        "\"route\":\"NONLINEAR_TOPOLOGY_REJECTED\"")
        != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && parent_semantic && parent_route && parent_stdout
            == "6db946e6c6e327520b60f98d34e60f346f0b02bfe84e88767fc1e36d7db9489d";
    ALR30RangeProjectionCapture& r30 =
        capture.r39.r38.r37.r36.r35.r34.r33.r32.r31.r30;
    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string trial_before =
        al_binary64_vec3_root(capture.trial_position);
    const std::string endpoint_before =
        al_binary64_vec3_root(capture.r39.hz.terminal_iterate);
    const std::string step_before =
        al_binary64_vec3_root(capture.physical_step);
    const bool source_exact = parent_exact && r30.captured
        && position_before
            == "e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828"
        && trial_before
            == "60e793aff919fada6b49c1b98be7ee5cc4a2c74a4254fa762cdece5700e1e804"
        && endpoint_before
            == "a73caf5baa6708ca89693077bad943dcdbf1f5069d50270ccc0f6b184a102e3a"
        && step_before
            == "424c84b72409f5b6665874f17afc18cf2784ae84593e30be41eaeec0d25e514b"
        && r30.position.size() == 6000U
        && capture.trial_position.size() == r30.position.size()
        && capture.physical_step.size() == r30.position.size();
    const ALR41DenseControls dense = al_r41_dense_controls();

    ALSparseWorkTrace work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    const SmokeFixture fixture = make_b4e2d_dam_fixture();
    const JointStaticSupportIndex index = build_joint_static_support_index(
        tagged_points(fixture.boundary), &static_work);
    const JointStaticSupportBinding binding = bind_joint_static_support_index(
        &index, index.identity_sha256);
    ALNormalizedWorkspace current;
    ALNormalizedWorkspace trial;
    if (source_exact && dense.passed && index.passed && binding.passed) {
        current = build_al_normalized_sparse_workspace(
            r30.position, binding, r30.dual, r30.theta, &work,
            &static_work, &adjacency_work);
        trial = build_al_normalized_sparse_workspace(
            capture.trial_position, binding, r30.dual, r30.theta, &work,
            &static_work, &adjacency_work);
    }
    const std::string current_topology = al_r29_topology_root(current);
    const std::string trial_topology = al_r29_topology_root(trial);
    const std::string current_pairs =
        al_r40_pair_membership_root(current.neighborhood);
    const std::string trial_pairs =
        al_r40_pair_membership_root(trial.neighborhood);
    const bool workspace_exact = current.passed && trial.passed
        && current_topology
            == "070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5"
        && trial_topology
            == "25d6148fc3f460ba6c324a539499deebabb86e88267ab6bdf3fe700dc44c7402"
        && current_pairs
            == "58ce6532f180f9f2803a069f8cfeb25278fbc3930796d7cf29700db08ce4fac3"
        && trial_pairs
            == "2e74673c3ae59c099c91902d7eac31b97b1c5a9496497128174d9a899f2bae06";
    const ALR41CrossingAudit crossings =
        al_r41_crossing_audit(current, trial);
    const ALR41ContactAudit contact = al_r41_contact_audit(
        r30.position, capture.trial_position, fixture);
    const bool contact_exact = contact.finite && contact.tests == 36000U
        && binary64_bits(contact.source_maximum)
            == binary64_bits(al_r40_maximum_box_penetration(
                r30.position, fixture))
        && binary64_bits(contact.trial_maximum)
            == binary64_bits(al_r40_maximum_box_penetration(
                capture.trial_position, fixture));
    if (trial.passed) release_al_normalized_workspace(trial, &work);
    if (current.passed) release_al_normalized_workspace(current, &work);
    const bool work_exact = work.workspace_builds == 2U
        && work.workspace_releases == 2U
        && work.live_workspaces == 0U
        && work.maximum_live_workspaces == 2U
        && !work.lifecycle_underflow
        && work.all_pair_candidate_calls == 0U
        && static_work.static_index_builds == 1U
        && static_work.support_canonicalizations == 1U
        && static_work.workspace_builds == 2U
        && adjacency_work.workspace_builds == 2U;
    const bool rollback_exact = position_before
            == al_binary64_vec3_root(r30.position)
        && trial_before == al_binary64_vec3_root(capture.trial_position)
        && endpoint_before
            == al_binary64_vec3_root(capture.r39.hz.terminal_iterate)
        && step_before == al_binary64_vec3_root(capture.physical_step);

    const std::array<std::pair<const char*, std::string>, 12U> routes{{
        {"CROSSING_PARENT_REJECTED", al_r41_route(0,1,1,1,1,1,1,1,0,0,0)},
        {"CROSSING_SOURCE_REJECTED", al_r41_route(1,0,1,1,1,1,1,1,0,0,0)},
        {"CROSSING_DENSE_REJECTED", al_r41_route(1,1,0,1,1,1,1,1,0,0,0)},
        {"CROSSING_WORKSPACE_REJECTED", al_r41_route(1,1,1,0,1,1,1,1,0,0,0)},
        {"CROSSING_PARTITION_REJECTED", al_r41_route(1,1,1,1,0,1,1,1,0,0,0)},
        {"CROSSING_KERNEL_REJECTED", al_r41_route(1,1,1,1,1,0,1,1,0,0,0)},
        {"CROSSING_CONTACT_LEDGER_REJECTED", al_r41_route(1,1,1,1,1,1,0,1,0,0,0)},
        {"CROSSING_WORK_REJECTED", al_r41_route(1,1,1,1,1,1,1,0,0,0,0)},
        {"NONZERO_SUPPORT_CROSSING_REQUIRES_RELINEARIZATION", al_r41_route(1,1,1,1,1,1,1,1,1,0,0)},
        {"NEW_CONTACT_CROSSING_REQUIRES_CONTACT_SOLVE", al_r41_route(1,1,1,1,1,1,1,1,0,1,0)},
        {"CONTACT_TANGENT_NORMAL_STEP_REQUIRED", al_r41_route(1,1,1,1,1,1,1,1,0,0,1)},
        {"MATERIAL_TOPOLOGY_EQUIVALENCE_CANDIDATE", al_r41_route(1,1,1,1,1,1,1,1,0,0,0)},
    }};
    bool routes_exact = true;
    std::ostringstream route_projection;
    for (const auto& value : routes) {
        routes_exact = routes_exact && value.first == value.second;
        route_projection << value.first << ':' << value.second << '|';
    }
    const std::string route_root = sha256_hex(route_projection.str());
    const bool nonzero_crossing = crossings.finite
        && !crossings.material_zero;
    const bool new_contact = contact.newly_penetrating != 0U;
    const bool existing_inward = contact.worsened != 0U
        && contact.existing_inward != 0U;
    const std::string route = al_r41_route(
        parent_exact, source_exact, dense.passed, workspace_exact,
        crossings.partition_exact, crossings.finite
            && crossings.member_radius_exact,
        contact_exact, work_exact && rollback_exact, nonzero_crossing,
        new_contact, existing_inward);
    const bool hard_controls = identity_exact && parent_exact && source_exact
        && dense.passed && workspace_exact && crossings.partition_exact
        && crossings.finite && crossings.member_radius_exact && contact_exact
        && work_exact && rollback_exact && routes_exact;
    const bool passed = hard_controls;
    std::string first_failure;
    if (!identity_exact) first_failure = "IDENTITY";
    else if (!parent_exact) first_failure = "PARENT";
    else if (!source_exact) first_failure = "SOURCE";
    else if (!dense.passed) first_failure = "DENSE";
    else if (!workspace_exact) first_failure = "WORKSPACE";
    else if (!crossings.partition_exact) first_failure = "PARTITION";
    else if (!crossings.finite || !crossings.member_radius_exact)
        first_failure = "KERNEL";
    else if (!contact_exact) first_failure = "CONTACT";
    else if (!work_exact) first_failure = "WORK";
    else if (!rollback_exact) first_failure = "ROLLBACK";
    else if (!routes_exact) first_failure = "ROUTES";
    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
             << B4E2D7R19R41_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << dense.root << '|' << current_topology << ':'
             << trial_topology << ':' << current_pairs << ':' << trial_pairs
             << '|' << crossings.root << ':' << contact.root << '|'
             << work_exact << ':' << rollback_exact << ':' << route_root
             << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4e2d7r19r41_support_crossing_contact_audit.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\""
           << B4E2D7R19R41_IDENTITY_SHA256 << "\""
           << ",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"dense\":{\"passed\":"
           << (dense.passed ? "true" : "false")
           << ",\"root\":\"" << dense.root << "\"}"
           << ",\"source\":{\"position_root\":\"" << position_before
           << "\",\"trial_root\":\"" << trial_before
           << "\",\"endpoint_root\":\"" << endpoint_before
           << "\",\"step_root\":\"" << step_before
           << "\",\"exact\":" << (source_exact ? "true" : "false")
           << "}"
           << ",\"crossings\":{\"intersection\":"
           << crossings.intersection << ",\"lost\":" << crossings.lost
           << ",\"entered\":" << crossings.entered
           << ",\"lost_fluid\":" << crossings.lost_fluid
           << ",\"lost_support\":" << crossings.lost_support
           << ",\"entered_fluid\":" << crossings.entered_fluid
           << ",\"entered_support\":" << crossings.entered_support
           << ",\"maximum_ulp_distance\":"
           << crossings.maximum_ulp_distance
           << ",\"maximum_abs_margin\":"
           << crossings.maximum_abs_margin
           << ",\"maximum_abs_weight\":"
           << crossings.maximum_abs_weight
           << ",\"maximum_abs_gradient\":"
           << crossings.maximum_abs_gradient
           << ",\"maximum_abs_second\":"
           << crossings.maximum_abs_second
           << ",\"maximum_abs_density_delta\":"
           << crossings.maximum_abs_density_delta
           << ",\"density_root\":\"" << crossings.density_root
           << "\",\"record_root\":\"" << crossings.record_root
           << "\",\"material_zero\":"
           << (crossings.material_zero ? "true" : "false")
           << ",\"partition_exact\":"
           << (crossings.partition_exact ? "true" : "false")
           << ",\"root\":\"" << crossings.root << "\"}"
           << ",\"contact\":{\"tests\":" << contact.tests
           << ",\"source_penetrating\":" << contact.source_penetrating
           << ",\"trial_penetrating\":" << contact.trial_penetrating
           << ",\"newly_penetrating\":" << contact.newly_penetrating
           << ",\"resolved\":" << contact.resolved
           << ",\"worsened\":" << contact.worsened
           << ",\"existing_inward\":" << contact.existing_inward
           << ",\"source_maximum\":" << contact.source_maximum
           << ",\"trial_maximum\":" << contact.trial_maximum
           << ",\"maximum_increase\":" << contact.maximum_increase
           << ",\"worst_particle\":" << contact.worst_particle
           << ",\"worst_axis\":" << contact.worst_axis
           << ",\"worst_side\":" << contact.worst_side
           << ",\"root\":\"" << contact.root << "\"}"
           << ",\"work\":{\"parent_r40_replays\":1"
              ",\"diagnostic_workspaces\":2,\"workspace_releases\":2"
              ",\"maximum_live_workspaces\":2,\"crossing_merges\":1"
              ",\"contact_face_tests\":" << contact.tests
           << ",\"new_jvp\":0,\"new_vjp\":0,\"new_hvp\":0"
              ",\"new_model\":0,\"new_trials\":0"
              ",\"new_precision\":0,\"new_outer\":0,\"exact\":"
           << (work_exact ? "true" : "false") << "}"
           << ",\"route_cases\":" << routes.size()
           << ",\"route_root\":\"" << route_root << "\""
           << ",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":"
           << (rollback_exact ? "true" : "false")
           << ",\"first_failure\":\"" << first_failure << "\""
           << ",\"route\":\"" << route << "\""
              ",\"r40_gates_changed\":false"
              ",\"support_expanded\":false"
              ",\"contact_projection_applied\":false"
              ",\"correction_applied\":false"
              ",\"candidate_committed\":false"
              ",\"tolerance_selected\":false"
              ",\"state_mutation\":false"
              ",\"timing_admitted\":false"
              ",\"runtime_authority\":false"
              ",\"production_authority\":false"
           << ",\"result_sha256\":\"" << result_sha << "\"}";
    if (replay_capture != nullptr) {
        *replay_capture = std::move(capture);
        replay_capture->captured = passed;
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_support_crossing_contact_audit_controls() {
    return run_al_support_crossing_contact_audit_controls_impl(nullptr);
}
