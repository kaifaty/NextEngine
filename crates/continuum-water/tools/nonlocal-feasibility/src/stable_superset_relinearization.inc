namespace {

constexpr const char* B4E2D7R19R42_IDENTITY_SHA256 =
    "14ad489cbf6985aea25f7c697e2cb6cb27c979883a2a27898331c96d6dcd78df";
constexpr const char* B4E2D7R19R42_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r42-stable-superset-relinearization|v1|parent=c0fa38fb:0237a380e86732e1ef78c50ca390ef2f93b1a82ac47338488403d9c5bb00d81c:3462fdcbc39d821c51fd4fdfbf9d9141ca7fd192fa53e9fb2885162c110f2ad8|source=r41-route-nonzero-support-crossing-requires-relinearization;position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;trial-60e793aff919fada6b49c1b98be7ee5cc4a2c74a4254fa762cdece5700e1e804;physical-step-424c84b72409f5b6665874f17afc18cf2784ae84593e30be41eaeec0d25e514b;current-topology-070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5;trial-topology-25d6148fc3f460ba6c324a539499deebabb86e88267ab6bdf3fe700dc44c7402;current-pairs-58ce6532f180f9f2803a069f8cfeb25278fbc3930796d7cf29700db08ce4fac3;trial-pairs-2e74673c3ae59c099c91902d7eac31b97b1c5a9496497128174d9a899f2bae06|ownership=one-source-anchored-canonical-verlet-superset;skin=0.04h=0.006;list-radius=h+skin=0.156;physical-h-unchanged;exact-r<=h-state-mask;lexicographic-pairs|certificate=actual-max-fluid-displacement-squared;4*dmax2<=skin2*(1-2e-12);analytic-global-l2-upper-bound;static-support-fixed|coverage=ordered-current-and-trial-subset;state-mask-roots;no-missing-pair;candidate-degree-cap|correspondence=independent-canonical-current-and-trial;filtered-pairs;flat-csr;radii;weight-first;weight-second;density;constraint;bit-exact|trial-operator=Atrial=spacing*Jc-at-trial;fresh-trial-radii-and-weight-first;pair-once-jvp;directed-jvp;superset-centered-finite-difference-epsilon2^-16;vjp-adjoint;constant-translation-interior|bounds=pair-directed-componentwise-gamma(16*max-degree+64);finite-difference-relative-l2<=1e-6;adjoint-relative<=1e-12;translation-interior-exact-zero|residual=trial-psi;violation;active;zero-origin-mapping=norm(AtrialT*max(cTrial,0));no-stationarity-tolerance|attribution=deterministic-current-trial-jvp-difference;nonzero-not-gate|controls=certificate-pass-fail;missing-pair;correspondence-mutation;operator-mutation;route-precedence;parent;source;dense;binding;superset;certificate;coverage;correspondence;operator;work;rollback|routes=stable-superset-parent-rejected;stable-superset-source-rejected;stable-superset-dense-rejected;stable-superset-binding-rejected;stable-superset-build-rejected;stable-superset-certificate-rejected;stable-superset-coverage-rejected;stable-superset-correspondence-rejected;trial-relinearization-rejected;stable-superset-work-rejected;stable-superset-relinearization-candidate|precedence=parent,source,dense,binding,superset,certificate,coverage,correspondence,trial-operator,work,candidate|work=parent-r41-replays1;static-index1;superset-builds1;canonical-workspaces2;masked-workspaces2;workspace-releases4;maximum-live2;filters2;jvp-pair2;jvp-directed1;vjp1;finite-difference-superset-evaluations2;translation-jvp1;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-gates=unchanged;support-expansion=none;contact-projection=none;correction=none;candidate-commit=none;tolerance=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-stable-superset-trial-relinearization-classification-only)IDENTITY";

struct ALR42Coverage {
    bool exact = false;
    std::size_t count = 0U;
    std::string root;
};

std::string al_r42_route(bool p, bool s, bool d, bool b, bool u,
    bool cert, bool cover, bool corr, bool op, bool work) {
    if (!p) return "STABLE_SUPERSET_PARENT_REJECTED";
    if (!s) return "STABLE_SUPERSET_SOURCE_REJECTED";
    if (!d) return "STABLE_SUPERSET_DENSE_REJECTED";
    if (!b) return "STABLE_SUPERSET_BINDING_REJECTED";
    if (!u) return "STABLE_SUPERSET_BUILD_REJECTED";
    if (!cert) return "STABLE_SUPERSET_CERTIFICATE_REJECTED";
    if (!cover) return "STABLE_SUPERSET_COVERAGE_REJECTED";
    if (!corr) return "STABLE_SUPERSET_CORRESPONDENCE_REJECTED";
    if (!op) return "TRIAL_RELINEARIZATION_REJECTED";
    if (!work) return "STABLE_SUPERSET_WORK_REJECTED";
    return "STABLE_SUPERSET_RELINEARIZATION_CANDIDATE";
}

ALR42Coverage al_r42_coverage(const JointNeighborhood& superset,
    const JointNeighborhood& active) {
    ALR42Coverage result;
    if (!superset.passed || !active.passed) return result;
    std::vector<std::uint8_t> mask(superset.pairs.size(), 0U);
    std::size_t cursor = 0U;
    for (std::size_t index = 0U; index < superset.pairs.size(); ++index) {
        if (cursor < active.pairs.size()
            && superset.pairs[index] == active.pairs[cursor]) {
            mask[index] = 1U;
            ++cursor;
        }
    }
    result.exact = cursor == active.pairs.size();
    result.count = cursor;
    result.root = al_r29_mask_root(mask);
    return result;
}

std::string al_r42_superset_root(const JointNeighborhood& value) {
    std::ostringstream projection;
    projection << binary64_bits(HORIZON) << ':' << binary64_bits(B4EP3_SKIN)
               << ':' << binary64_bits(B4EP3_LIST_RADIUS) << ':'
               << value.fluid.size() << ':' << value.support.size() << ':'
               << value.pairs.size() << ':' << value.maximum_degree << '|';
    for (const JointPair pair : value.pairs)
        projection << pair.fluid << ':' << pair.participant << '|';
    return sha256_hex(projection.str());
}

bool al_r42_workspace_exact(const ALNormalizedWorkspace& lhs,
    const ALNormalizedWorkspace& rhs) {
    return lhs.passed && rhs.passed
        && lhs.neighborhood.pairs == rhs.neighborhood.pairs
        && lhs.neighborhood.flat_offsets == rhs.neighborhood.flat_offsets
        && lhs.neighborhood.flat_directed_pair_indices
            == rhs.neighborhood.flat_directed_pair_indices
        && lhs.radius == rhs.radius && lhs.weight_first == rhs.weight_first
        && lhs.weight_second_value == rhs.weight_second_value
        && lhs.support.density == rhs.support.density
        && lhs.support.constraint == rhs.support.constraint;
}

std::vector<double> al_r42_constraints(const JointNeighborhood& superset,
    const std::vector<Vec3>& position) {
    const std::size_t count = superset.fluid.size();
    if (!superset.passed || position.size() != count) return {};
    std::vector<double> density(count, MASS * weight(0.0));
    for (const JointPair pair : superset.pairs) {
        const Vec3 other = pair.participant < count
            ? position[pair.participant]
            : superset.support[pair.participant - count].position;
        const double contribution = MASS * weight(
            norm(position[pair.fluid] - other));
        density[pair.fluid] += contribution;
        if (pair.participant < count) density[pair.participant] += contribution;
    }
    std::vector<double> result(count);
    for (std::size_t index = 0U; index < count; ++index)
        result[index] = density[index] / REST_DENSITY - 1.0;
    return result;
}

double al_r42_jvp_bound_ratio(const ALLinearizedJvp& pair,
    const ALLinearizedJvp& directed, std::size_t degree,
    std::size_t& failures) {
    failures = pair.value.size() == directed.value.size() ? 0U : 1U;
    double maximum = 0.0;
    const double operations = 16.0 * static_cast<double>(degree) + 64.0;
    const double unit = std::numeric_limits<double>::epsilon();
    const double gamma = operations * unit / (1.0 - operations * unit);
    for (std::size_t i = 0U; i < pair.value.size() && failures == 0U; ++i) {
        const double error = std::abs(pair.value[i] - directed.value[i]);
        const double scale = std::max(
            {pair.absolute_sum[i], directed.absolute_sum[i], 1.0e-300});
        const double bound = gamma * scale;
        if (error > bound) ++failures;
        maximum = std::max(maximum, bound > 0.0 ? error / bound
            : (error == 0.0 ? 0.0
                            : std::numeric_limits<double>::infinity()));
    }
    return maximum;
}

struct ALR42ReplayCapture {
    bool captured = false;
    ALR40ReplayCapture r40;
    JointNeighborhood superset;
};

} // namespace

SplitBoundaryReport run_al_stable_superset_relinearization_controls_impl(
    ALR42ReplayCapture* replay_capture) {
    const bool identity = sha256_hex(B4E2D7R19R42_IDENTITY_PROJECTION)
        == B4E2D7R19R42_IDENTITY_SHA256;
    ALR40ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_support_crossing_contact_audit_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "0237a380e86732e1ef78c50ca390ef2f93b1a82ac47338488403d9c5bb00d81c"
        && parent.json.find("3462fdcbc39d821c51fd4fdfbf9d9141ca7fd192fa53e9fb2885162c110f2ad8")
            != std::string::npos
        && parent.json.find("NONZERO_SUPPORT_CROSSING_REQUIRES_RELINEARIZATION")
            != std::string::npos;
    ALR30RangeProjectionCapture& r30 =
        capture.r39.r38.r37.r36.r35.r34.r33.r32.r31.r30;
    const std::string position_root = al_binary64_vec3_root(r30.position);
    const std::string trial_root = al_binary64_vec3_root(capture.trial_position);
    const std::string step_root = al_binary64_vec3_root(capture.physical_step);
    const bool source = parent_exact && r30.captured
        && position_root
            == "e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828"
        && trial_root
            == "60e793aff919fada6b49c1b98be7ee5cc4a2c74a4254fa762cdece5700e1e804"
        && step_root
            == "424c84b72409f5b6665874f17afc18cf2784ae84593e30be41eaeec0d25e514b";
    const bool dense = identity && b4ep3_certificate(0.0)
        && b4ep3_certificate(0.24 * B4EP3_SKIN * B4EP3_SKIN)
        && !b4ep3_certificate(0.26 * B4EP3_SKIN * B4EP3_SKIN)
        && B4EP3_SKIN == 0.006 && B4EP3_LIST_RADIUS == 0.156;

    ALSparseWorkTrace work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    const SmokeFixture fixture = make_b4e2d_dam_fixture();
    const JointStaticSupportIndex index = build_joint_static_support_index(
        tagged_points(fixture.boundary), &static_work);
    const JointStaticSupportBinding binding = bind_joint_static_support_index(
        &index, index.identity_sha256);
    const bool binding_exact = index.passed && binding.passed;
    JointNeighborhood superset;
    if (source && dense && binding_exact)
        superset = b4ep3_build_superset(tagged_points(r30.position), binding);
    const bool superset_exact = superset.passed
        && superset.maximum_degree <= B4C0_MAX_NEIGHBORS
        && std::is_sorted(superset.pairs.begin(), superset.pairs.end(),
            al_joint_pair_less);
    const std::string superset_root = al_r42_superset_root(superset);

    double maximum_displacement_squared = 0.0;
    const double physical_norm = al_r32_vec3_norm(capture.physical_step);
    const bool certificate = superset_exact
        && b4ep3_max_displacement_squared(tagged_points(r30.position),
            tagged_points(capture.trial_position),
            maximum_displacement_squared)
        && b4ep3_certificate(maximum_displacement_squared)
        && maximum_displacement_squared <= physical_norm * physical_norm;
    JointNeighborhood filtered_current;
    JointNeighborhood filtered_trial;
    if (certificate) {
        filtered_current = b4ep3_filter_superset(
            tagged_points(r30.position), superset);
        filtered_trial = b4ep3_filter_superset(
            tagged_points(capture.trial_position), superset);
    }
    const ALR42Coverage current_coverage =
        al_r42_coverage(superset, filtered_current);
    const ALR42Coverage trial_coverage =
        al_r42_coverage(superset, filtered_trial);
    JointNeighborhood missing = superset;
    if (missing.passed && !filtered_trial.pairs.empty()) {
        const auto found = std::lower_bound(missing.pairs.begin(),
            missing.pairs.end(), filtered_trial.pairs.front(),
            al_joint_pair_less);
        if (found != missing.pairs.end() && *found == filtered_trial.pairs.front())
            missing.pairs.erase(found);
    }
    const bool missing_rejected =
        !al_r42_coverage(missing, filtered_trial).exact;
    const bool coverage = current_coverage.exact && trial_coverage.exact
        && missing_rejected && current_coverage.count == 340340U
        && trial_coverage.count == 340262U;

    const std::vector<Vec3> direction = al_r29_fluid_probe(r30.position.size());
    ALLinearizedJvp current_jvp;
    bool current_correspondence = false;
    if (coverage) {
        ALNormalizedWorkspace canonical = build_al_normalized_sparse_workspace(
            r30.position, binding, r30.dual, r30.theta, &work,
            &static_work, &adjacency_work);
        ALNormalizedWorkspace masked =
            build_al_normalized_workspace_from_neighborhood(
                std::move(filtered_current), r30.dual, r30.theta, &work);
        current_correspondence = al_r42_workspace_exact(canonical, masked)
            && al_r29_topology_root(canonical)
                == "070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5";
        current_jvp = al_r29_pair_once_jvp(masked, direction);
        release_al_normalized_workspace(masked, &work);
        release_al_normalized_workspace(canonical, &work);
    }
    ALNormalizedWorkspace trial_workspace;
    bool trial_correspondence = false;
    bool mutation_rejected = false;
    if (coverage && current_correspondence) {
        ALNormalizedWorkspace canonical = build_al_normalized_sparse_workspace(
            capture.trial_position, binding, r30.dual, r30.theta, &work,
            &static_work, &adjacency_work);
        trial_workspace = build_al_normalized_workspace_from_neighborhood(
            std::move(filtered_trial), r30.dual, r30.theta, &work);
        trial_correspondence = al_r42_workspace_exact(canonical, trial_workspace)
            && al_r29_topology_root(canonical)
                == "25d6148fc3f460ba6c324a539499deebabb86e88267ab6bdf3fe700dc44c7402";
        ALNormalizedWorkspace mutated = trial_workspace;
        if (!mutated.radius.empty()) mutated.radius.front() += 1.0;
        mutation_rejected = !al_r42_workspace_exact(canonical, mutated);
        release_al_normalized_workspace(canonical, &work);
    }
    const bool correspondence = current_correspondence
        && trial_correspondence && mutation_rejected;

    const ALLinearizedJvp trial_jvp =
        al_r29_pair_once_jvp(trial_workspace, direction);
    const ALLinearizedJvp directed =
        al_r29_directed_jvp(trial_workspace, direction);
    std::size_t jvp_failures = 0U;
    const double jvp_bound_ratio = al_r42_jvp_bound_ratio(trial_jvp,
        directed, trial_workspace.neighborhood.maximum_degree, jvp_failures);
    constexpr double epsilon = 0x1p-16;
    std::vector<Vec3> plus = capture.trial_position;
    std::vector<Vec3> minus = capture.trial_position;
    for (std::size_t i = 0U; i < plus.size(); ++i) {
        plus[i] += SPACING * epsilon * direction[i];
        minus[i] += -SPACING * epsilon * direction[i];
    }
    const std::vector<double> plus_c = al_r42_constraints(superset, plus);
    const std::vector<double> minus_c = al_r42_constraints(superset, minus);
    std::vector<double> difference(plus.size());
    for (std::size_t i = 0U; i < difference.size()
         && plus_c.size() == difference.size()
         && minus_c.size() == difference.size(); ++i)
        difference[i] = (plus_c[i] - minus_c[i]) / (2.0 * epsilon);
    const double fd_error = al_r29_relative_error(difference, trial_jvp.value);

    std::vector<double> positive(trial_workspace.support.constraint.size());
    for (std::size_t i = 0U; i < positive.size(); ++i)
        positive[i] = std::max(0.0, trial_workspace.support.constraint[i]);
    const std::vector<double> row_probe = al_r29_row_probe(positive.size());
    std::vector<Vec3> adjoint(trial_workspace.neighborhood.fluid.size());
    std::vector<Vec3> mapping(trial_workspace.neighborhood.fluid.size());
    for (std::size_t p = 0U; p < trial_workspace.neighborhood.pairs.size(); ++p) {
        const JointPair pair = trial_workspace.neighborhood.pairs[p];
        const Vec3 jacobian = al_r29_pair_jacobian(trial_workspace, p);
        const auto fold = [&](const std::vector<double>& weights,
                              std::vector<Vec3>& target) {
            const double scale = pair.participant < positive.size()
                ? weights[pair.fluid] + weights[pair.participant]
                : weights[pair.fluid];
            const Vec3 term = SPACING * scale * jacobian;
            target[pair.fluid] += term;
            if (pair.participant < positive.size())
                target[pair.participant] += -term;
        };
        fold(row_probe, adjoint);
        fold(positive, mapping);
    }
    long double lhs = 0.0L;
    long double rhs = 0.0L;
    for (std::size_t i = 0U; i < positive.size(); ++i) {
        lhs += static_cast<long double>(trial_jvp.value[i]) * row_probe[i];
        rhs += static_cast<long double>(direction[i].x) * adjoint[i].x
            + static_cast<long double>(direction[i].y) * adjoint[i].y
            + static_cast<long double>(direction[i].z) * adjoint[i].z;
    }
    const double adjoint_error = static_cast<double>(std::abs(lhs - rhs)
        / std::max({std::abs(lhs), std::abs(rhs), 1.0e-300L}));
    std::vector<std::uint8_t> boundary(positive.size(), 0U);
    for (const JointPair pair : trial_workspace.neighborhood.pairs)
        if (pair.participant >= positive.size()) boundary[pair.fluid] = 1U;
    const ALLinearizedJvp translation = al_r29_pair_once_jvp(trial_workspace,
        std::vector<Vec3>(positive.size(), {1.0, -0.5, 0.25}));
    bool translation_exact = translation.passed;
    bool boundary_nonzero = false;
    for (std::size_t i = 0U; i < positive.size(); ++i) {
        if (!boundary[i] && translation.value[i] != 0.0)
            translation_exact = false;
        if (boundary[i] && translation.value[i] != 0.0)
            boundary_nonzero = true;
    }
    translation_exact = translation_exact && boundary_nonzero;
    const bool operator_exact = correspondence && trial_jvp.passed
        && directed.passed && jvp_failures == 0U && std::isfinite(jvp_bound_ratio)
        && std::isfinite(fd_error) && fd_error <= 1.0e-6
        && std::isfinite(adjoint_error) && adjoint_error <= 1.0e-12
        && translation_exact;
    const ALR40HingeMetric metric =
        al_r40_hinge_metric(trial_workspace.support.constraint);
    const double mapping_norm = al_r32_vec3_norm(mapping);
    std::vector<double> jvp_difference(trial_jvp.value.size());
    for (std::size_t i = 0U; i < jvp_difference.size(); ++i)
        jvp_difference[i] = trial_jvp.value[i] - current_jvp.value[i];
    const double jvp_difference_norm = al_r29_scalar_norm(jvp_difference);
    const std::string current_jvp_root = al_r10_dual_root(current_jvp.value);
    const std::string trial_jvp_root = al_r10_dual_root(trial_jvp.value);
    const std::string mapping_root = al_binary64_vec3_root(mapping);
    if (trial_workspace.passed)
        release_al_normalized_workspace(trial_workspace, &work);

    const bool work_exact = work.workspace_builds == 4U
        && work.workspace_releases == 4U && work.live_workspaces == 0U
        && work.maximum_live_workspaces == 2U && !work.lifecycle_underflow
        && static_work.static_index_builds == 1U
        && static_work.support_canonicalizations == 1U
        && static_work.workspace_builds == 2U
        && adjacency_work.workspace_builds == 2U;
    const bool rollback = position_root == al_binary64_vec3_root(r30.position)
        && trial_root == al_binary64_vec3_root(capture.trial_position)
        && step_root == al_binary64_vec3_root(capture.physical_step);

    const std::array<const char*, 11U> expected{{
        "STABLE_SUPERSET_PARENT_REJECTED", "STABLE_SUPERSET_SOURCE_REJECTED",
        "STABLE_SUPERSET_DENSE_REJECTED", "STABLE_SUPERSET_BINDING_REJECTED",
        "STABLE_SUPERSET_BUILD_REJECTED", "STABLE_SUPERSET_CERTIFICATE_REJECTED",
        "STABLE_SUPERSET_COVERAGE_REJECTED",
        "STABLE_SUPERSET_CORRESPONDENCE_REJECTED",
        "TRIAL_RELINEARIZATION_REJECTED", "STABLE_SUPERSET_WORK_REJECTED",
        "STABLE_SUPERSET_RELINEARIZATION_CANDIDATE"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t index_value = 0U; index_value < expected.size(); ++index_value) {
        std::array<bool, 10U> gates{};
        gates.fill(true);
        if (index_value < gates.size()) gates[index_value] = false;
        const std::string observed = al_r42_route(gates[0], gates[1], gates[2],
            gates[3], gates[4], gates[5], gates[6], gates[7], gates[8], gates[9]);
        routes_exact = routes_exact && observed == expected[index_value];
        route_material << expected[index_value] << ':' << observed << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r42_route(parent_exact, source, dense,
        binding_exact, superset_exact, certificate, coverage, correspondence,
        operator_exact, work_exact && rollback);
    const bool passed = parent_exact && source && dense && binding_exact
        && superset_exact && certificate && coverage && correspondence
        && operator_exact && work_exact && rollback && routes_exact;
    std::string failure;
    if (!identity) failure = "IDENTITY";
    else if (!parent_exact) failure = "PARENT";
    else if (!source) failure = "SOURCE";
    else if (!dense) failure = "DENSE";
    else if (!binding_exact) failure = "BINDING";
    else if (!superset_exact) failure = "SUPERSET";
    else if (!certificate) failure = "CERTIFICATE";
    else if (!coverage) failure = "COVERAGE";
    else if (!correspondence) failure = "CORRESPONDENCE";
    else if (!operator_exact) failure = "OPERATOR";
    else if (!work_exact) failure = "WORK";
    else if (!rollback) failure = "ROLLBACK";
    else if (!routes_exact) failure = "ROUTES";
    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << failure << '|' << B4E2D7R19R42_IDENTITY_SHA256 << '|'
             << parent_stdout << '|' << superset_root << ':'
             << current_coverage.root << ':' << trial_coverage.root << '|'
             << maximum_displacement_squared << ':' << correspondence << '|'
             << current_jvp_root << ':' << trial_jvp_root << ':' << mapping_root
             << ':' << fd_error << ':' << adjoint_error << '|'
             << metric.psi << ':' << metric.norm << ':' << metric.active << ':'
             << mapping_norm << ':' << jvp_difference_norm << '|'
             << work_exact << ':' << rollback << ':' << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r42_stable_superset_relinearization.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R42_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"superset\":{\"pairs\":" << superset.pairs.size()
           << ",\"maximum_degree\":" << superset.maximum_degree
           << ",\"root\":\"" << superset_root << "\"}"
           << ",\"certificate\":{\"maximum_displacement_squared\":"
           << maximum_displacement_squared << ",\"physical_global_l2\":"
           << physical_norm << ",\"skin\":" << B4EP3_SKIN
           << ",\"passed\":" << (certificate ? "true" : "false") << "}"
           << ",\"coverage\":{\"current_active\":" << current_coverage.count
           << ",\"trial_active\":" << trial_coverage.count
           << ",\"current_mask_root\":\"" << current_coverage.root
           << "\",\"trial_mask_root\":\"" << trial_coverage.root
           << "\",\"missing_control_rejected\":"
           << (missing_rejected ? "true" : "false") << "}"
           << ",\"correspondence\":{\"current\":"
           << (current_correspondence ? "true" : "false")
           << ",\"trial\":" << (trial_correspondence ? "true" : "false")
           << ",\"mutation_rejected\":"
           << (mutation_rejected ? "true" : "false") << "}"
           << ",\"operator\":{\"jvp_bound_failures\":" << jvp_failures
           << ",\"maximum_jvp_bound_ratio\":" << jvp_bound_ratio
           << ",\"finite_difference_error\":" << fd_error
           << ",\"adjoint_error\":" << adjoint_error
           << ",\"translation_exact\":"
           << (translation_exact ? "true" : "false")
           << ",\"current_jvp_root\":\"" << current_jvp_root
           << "\",\"trial_jvp_root\":\"" << trial_jvp_root
           << "\",\"jvp_difference_l2\":" << jvp_difference_norm << "}"
           << ",\"trial_residual\":{\"psi\":" << metric.psi
           << ",\"violation\":" << metric.norm << ",\"active\":"
           << metric.active << ",\"zero_origin_mapping\":" << mapping_norm
           << ",\"mapping_root\":\"" << mapping_root << "\"}"
           << ",\"work\":{\"parent_r41_replays\":1,\"static_indices\":1,"
              "\"superset_builds\":1,\"filters\":2,"
              "\"canonical_workspaces\":2,\"masked_workspaces\":2,"
              "\"workspace_releases\":4,\"maximum_live\":2,"
              "\"jvp_pair\":2,\"jvp_directed\":1,\"vjp\":1,"
              "\"finite_difference_evaluations\":2,\"translation_jvp\":1,"
              "\"new_hvp\":0,\"new_model\":0,\"new_trial\":0,"
              "\"new_precision\":0,\"new_outer\":0,\"exact\":"
           << (work_exact ? "true" : "false") << "}"
           << ",\"route_cases\":" << expected.size()
           << ",\"route_root\":\"" << route_root << "\",\"routes_exact\":"
           << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback ? "true" : "false")
           << ",\"first_failure\":\"" << failure << "\",\"route\":\""
           << route << "\",\"r40_r41_gates_changed\":false"
              ",\"physical_support_expanded\":false"
              ",\"contact_projection_applied\":false"
              ",\"correction_applied\":false,\"candidate_committed\":false"
              ",\"tolerance_selected\":false,\"state_mutation\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    if (replay_capture != nullptr) {
        replay_capture->captured = passed;
        replay_capture->r40 = std::move(capture);
        replay_capture->superset = std::move(superset);
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_stable_superset_relinearization_controls() {
    return run_al_stable_superset_relinearization_controls_impl(nullptr);
}
