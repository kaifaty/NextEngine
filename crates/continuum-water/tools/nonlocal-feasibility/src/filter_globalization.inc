namespace {

constexpr const char* B4E2D7R19R46_IDENTITY_SHA256 =
    "afa98b317f26e6cc6a11d26253d2d27723d4e77f7f44b89ff3eabd9100214ee4";
constexpr const char* B4E2D7R19R46_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r46-filter-globalization|v1|parent=c62bda39:2ab62170eb816eee7f71ee330e5303cdb6852a241cd5e7c54160fe0914cbabc7:55c04d4294290e212a48911ac717df10be34e39013751ed387b184df479a3773:COMPOSITE_MERIT_RECOVERY_REQUIRED|source=r43-position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;trial-0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50;owner-baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187;superset-355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01|coordinates=f=0.5*norm2(y-yhat);h=norm2(max(c,0));psi=0.5*h*h;phr-excluded-from-f;f-lower-bound0;fresh-exact-support|filter=initial-empty;admission-against-initial-union-source;envelope=htrial<=(1-gamma)*hsource-or-objective-reduction>=gamma*hsource;gamma-k=2^-k;k1through24;strongest-first;all24-required;strongest-h-branch-required;resolution-slack>1024*epsilon*max(hsource,htrial,min-normal)|model=inherited-r43-actual-and-predicted-psi-reductions-positive;rho>=0.1|update=feasibility-step-adds-source-entry-to-hypothetical-next-filter;canonical-binary64-entry-order;candidate-not-added;no-state-commit|nonlinear=source-and-r43-trial-exact-workspaces;stable-superset-coverage;canonical-correspondence;all-36000-contact-faces|controls=coordinate-separation;pure-inertia-lower-bound;h-branch;f-branch;strongest-margin-reject;gamma-order;empty-current-ownership;filter-update-root;route-precedence;parent;source;workspace;model;topology;contact;work;rollback|routes=filter-parent-rejected;filter-source-rejected;filter-coordinate-rejected;filter-model-rejected;filter-topology-rejected;filter-contact-rejected;filter-restoration-required;filter-update-rejected;filter-work-rejected;filter-feasibility-step-candidate|precedence=parent,source,coordinate,model,topology,contact,envelope,update,work,candidate|work=parent-r45-replays1;static-index1;superset-build1;filters2;workspaces2;releases2;precancelled-divided-repeats2;contact-tests36000;gamma-tests24;new-jvp0;new-vjp0;new-hvp0;new-model0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45=unchanged;correction=none;candidate-commit=none;filter-runtime-commit=none;following-outer=none;tolerance=none;penalty-change=none;support-expansion=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-filter-feasibility-admission-classification-only)IDENTITY";

constexpr std::size_t AL_R46_MARGIN_TESTS = 24U;
constexpr double AL_R46_MODEL_RATIO = 0.1;

struct ALR46FilterDecision {
    bool h_branch = false;
    bool f_branch = false;
    bool acceptable = false;
    double h_slack = 0.0;
    double f_slack = 0.0;
};

ALR46FilterDecision al_r46_filter_decision(double source_h,
    double trial_h, double objective_reduction, double gamma) {
    ALR46FilterDecision result;
    result.h_slack = (1.0 - gamma) * source_h - trial_h;
    result.f_slack = objective_reduction - gamma * source_h;
    result.h_branch = result.h_slack >= 0.0;
    result.f_branch = result.f_slack >= 0.0;
    result.acceptable = result.h_branch || result.f_branch;
    return result;
}

std::string al_r46_filter_entry_root(double h, double f) {
    std::ostringstream material;
    material << "nextengine.nonlocal.filter-entry|v1|h="
             << binary64_bits(h) << "|f=" << binary64_bits(f);
    return sha256_hex(material.str());
}

std::string al_r46_route(bool parent, bool source, bool coordinate,
    bool model, bool topology, bool contact, bool envelope, bool update,
    bool work) {
    if (!parent) return "FILTER_PARENT_REJECTED";
    if (!source) return "FILTER_SOURCE_REJECTED";
    if (!coordinate) return "FILTER_COORDINATE_REJECTED";
    if (!model) return "FILTER_MODEL_REJECTED";
    if (!topology) return "FILTER_TOPOLOGY_REJECTED";
    if (!contact) return "FILTER_CONTACT_REJECTED";
    if (!envelope) return "FILTER_RESTORATION_REQUIRED";
    if (!update) return "FILTER_UPDATE_REJECTED";
    if (!work) return "FILTER_WORK_REJECTED";
    return "FILTER_FEASIBILITY_STEP_CANDIDATE";
}

struct ALR46ReplayCapture {
    bool captured = false;
    ALR45ReplayCapture r45;
    double source_f = 0.0;
    double trial_f = 0.0;
    double objective_reduction = 0.0;
    double source_h = 0.0;
    double trial_h = 0.0;
    double source_psi = 0.0;
    double trial_psi = 0.0;
    bool filter_admitted = false;
};

} // namespace

SplitBoundaryReport run_al_filter_globalization_controls_impl(
    ALR46ReplayCapture* replay_capture) {
    const bool identity = sha256_hex(B4E2D7R19R46_IDENTITY_PROJECTION)
        == B4E2D7R19R46_IDENTITY_SHA256;
    ALR45ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_nonlinear_line_globalization_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "2ab62170eb816eee7f71ee330e5303cdb6852a241cd5e7c54160fe0914cbabc7"
        && parent.json.find(
            "55c04d4294290e212a48911ac717df10be34e39013751ed387b184df479a3773")
            != std::string::npos
        && parent.json.find("COMPOSITE_MERIT_RECOVERY_REQUIRED")
            != std::string::npos;

    ALR43ReplayCapture& r43 = capture.r44.r43;
    ALR40ReplayCapture& r40 = r43.r42.r40;
    ALR30RangeProjectionCapture& r30 =
        r40.r39.r38.r37.r36.r35.r34.r33.r32.r31.r30;
    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string trial_before =
        al_binary64_vec3_root(r43.projected_trial);
    const std::string owner_before = al_r29_mask_root(r43.owners);
    const std::string superset_before = al_r42_superset_root(r43.r42.superset);
    const bool source_exact = parent_exact && r30.captured
        && position_before
            == "e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828"
        && trial_before
            == "0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50"
        && owner_before
            == "baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187"
        && superset_before
            == "355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01";

    const ALR46FilterDecision dense_h =
        al_r46_filter_decision(1.0, 0.25, -10.0, 0.5);
    const ALR46FilterDecision dense_f =
        al_r46_filter_decision(1.0, 0.75, 0.75, 0.5);
    const ALR46FilterDecision dense_reject =
        al_r46_filter_decision(1.0, 0.75, 0.25, 0.5);
    bool gamma_order = true;
    double previous_gamma = 1.0;
    for (std::size_t k = 1U; k <= AL_R46_MARGIN_TESTS; ++k) {
        const double gamma = std::ldexp(1.0, -static_cast<int>(k));
        gamma_order = gamma_order && gamma > 0.0 && gamma < previous_gamma;
        previous_gamma = gamma;
    }
    const bool dense = identity && dense_h.acceptable && dense_h.h_branch
        && !dense_h.f_branch && dense_f.acceptable && !dense_f.h_branch
        && dense_f.f_branch && !dense_reject.acceptable && gamma_order
        && al_r46_route(true, true, true, true, true, true, false,
               true, true)
            == "FILTER_RESTORATION_REQUIRED";

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
    JointNeighborhood source_filtered = b4ep3_filter_superset(
        tagged_points(r30.position), superset);
    JointNeighborhood trial_filtered = b4ep3_filter_superset(
        tagged_points(r43.projected_trial), superset);
    ALNormalizedWorkspace source_workspace =
        build_al_normalized_sparse_workspace(r30.position, binding,
            r30.dual, r30.theta, &work, &static_work, &adjacency_work);
    ALNormalizedWorkspace trial_workspace =
        build_al_normalized_sparse_workspace(r43.projected_trial, binding,
            r30.dual, r30.theta, &work, &static_work, &adjacency_work);
    const ALR42Coverage source_coverage =
        al_r42_coverage(superset, source_filtered);
    const ALR42Coverage trial_coverage =
        al_r42_coverage(superset, trial_filtered);
    const bool superset_root_exact =
        al_r42_superset_root(superset) == superset_before;
    const bool source_pairs_exact =
        source_filtered.pairs == source_workspace.neighborhood.pairs;
    const bool trial_pairs_exact =
        trial_filtered.pairs == trial_workspace.neighborhood.pairs;
    const bool source_workspace_passed = source_workspace.passed;
    const bool trial_workspace_passed = trial_workspace.passed;
    const bool workspace_exact = source_exact && dense && index.passed
        && binding.passed && superset.passed
        && superset_root_exact
        && source_workspace.passed && trial_workspace.passed
        && source_coverage.exact && trial_coverage.exact
        && source_pairs_exact && trial_pairs_exact;

    const ALR40HingeMetric source_metric =
        al_r40_hinge_metric(source_workspace.support.constraint);
    const ALR40HingeMetric trial_metric =
        al_r40_hinge_metric(trial_workspace.support.constraint);
    const ALNormalizedInner source_inner = evaluate_al_normalized_inner(
        source_workspace.support, r30.position, r30.predicted_position);
    const ALNormalizedInner trial_inner = evaluate_al_normalized_inner(
        trial_workspace.support, r43.projected_trial,
        r30.predicted_position);
    const ALNormalizedPrecancellationDivided divided =
        evaluate_al_normalized_sparse_precancelled_divided(source_workspace,
            trial_workspace, r30.predicted_position, r30.dual, r30.theta,
            &work);
    const ALNormalizedPrecancellationDivided divided_repeat =
        evaluate_al_normalized_sparse_precancelled_divided(source_workspace,
            trial_workspace, r30.predicted_position, r30.dual, r30.theta,
            &work);
    const bool divided_exact =
        al_normalized_precancellation_exact(divided, divided_repeat);
    const double source_f = source_inner.inertia;
    const double trial_f = trial_inner.inertia;
    const double objective_reduction = divided.value.inertia_reduction;
    const bool lower_bound_exact = source_f >= 0.0 && trial_f >= 0.0
        && std::isfinite(source_f) && std::isfinite(trial_f);
    const bool norm_correspondence =
        source_metric.norm == std::sqrt(2.0 * source_metric.psi)
        && trial_metric.norm == std::sqrt(2.0 * trial_metric.psi);
    const bool coordinate_exact = workspace_exact && source_metric.finite
        && trial_metric.finite && source_metric.norm > 0.0
        && trial_metric.norm >= 0.0 && source_inner.passed
        && trial_inner.passed && lower_bound_exact
        && divided.value.finite_values && divided_exact
        && norm_correspondence;
    const bool model_exact = coordinate_exact
        && r43.predicted_psi_reduction > 0.0
        && r43.actual_psi_reduction > 0.0
        && std::isfinite(r43.model_ratio)
        && r43.model_ratio >= AL_R46_MODEL_RATIO
        && binary64_bits(r43.actual_psi_reduction)
            == binary64_bits(source_metric.psi - trial_metric.psi);

    std::size_t gamma_tests = 0U;
    std::size_t acceptable_margins = 0U;
    std::size_t h_branches = 0U;
    std::size_t f_branches = 0U;
    bool all_margins = true;
    ALR46FilterDecision strongest;
    std::ostringstream margin_material;
    for (std::size_t k = 1U; k <= AL_R46_MARGIN_TESTS; ++k) {
        const double gamma = std::ldexp(1.0, -static_cast<int>(k));
        const ALR46FilterDecision decision = al_r46_filter_decision(
            source_metric.norm, trial_metric.norm, objective_reduction,
            gamma);
        if (k == 1U) strongest = decision;
        ++gamma_tests;
        acceptable_margins += decision.acceptable ? 1U : 0U;
        h_branches += decision.h_branch ? 1U : 0U;
        f_branches += decision.f_branch ? 1U : 0U;
        all_margins = all_margins && decision.acceptable;
        margin_material << k << ':' << binary64_bits(gamma) << ':'
                        << binary64_bits(decision.h_slack) << ':'
                        << binary64_bits(decision.f_slack) << ':'
                        << decision.h_branch << ':' << decision.f_branch
                        << ':' << decision.acceptable << '|';
    }
    const double resolution_guard = 1024.0
        * std::numeric_limits<double>::epsilon()
        * std::max({source_metric.norm, trial_metric.norm,
            std::numeric_limits<double>::min()});
    const bool envelope_exact = coordinate_exact
        && gamma_tests == AL_R46_MARGIN_TESTS && all_margins
        && strongest.h_branch && strongest.h_slack > resolution_guard;
    const std::string margin_root = sha256_hex(margin_material.str());

    const ALR41ContactAudit contact_audit = al_r41_contact_audit(
        r30.position, r43.projected_trial, fixture);
    const bool topology_exact = workspace_exact
        && source_coverage.root
            == al_r42_coverage(superset, source_filtered).root
        && trial_coverage.root
            == al_r42_coverage(superset, trial_filtered).root;
    const bool contact_exact = contact_audit.finite
        && contact_audit.tests == 36000U
        && contact_audit.newly_penetrating == 0U
        && contact_audit.worsened == 0U
        && contact_audit.trial_maximum <= contact_audit.source_maximum;

    const std::string empty_filter_root = sha256_hex(
        "nextengine.nonlocal.filter|v1|entries=0");
    const std::string source_entry_root =
        al_r46_filter_entry_root(source_metric.norm, source_f);
    const std::string next_filter_root = sha256_hex(
        std::string("nextengine.nonlocal.filter|v1|entries=1|")
        + source_entry_root);
    const bool update_exact = !envelope_exact
        || (empty_filter_root != source_entry_root
            && source_entry_root != next_filter_root
            && next_filter_root != empty_filter_root);

    if (trial_workspace.passed)
        release_al_normalized_workspace(trial_workspace, &work);
    if (source_workspace.passed)
        release_al_normalized_workspace(source_workspace, &work);
    const bool work_exact = work.workspace_builds == 2U
        && work.workspace_releases == 2U && work.live_workspaces == 0U
        && work.maximum_live_workspaces == 2U
        && !work.lifecycle_underflow && work.all_pair_candidate_calls == 0U
        && static_work.static_index_builds == 1U
        && static_work.support_canonicalizations == 1U
        && static_work.workspace_builds == 2U
        && adjacency_work.workspace_builds == 2U
        && gamma_tests == AL_R46_MARGIN_TESTS
        && contact_audit.tests == 36000U;
    const bool rollback = position_before == al_binary64_vec3_root(r30.position)
        && trial_before == al_binary64_vec3_root(r43.projected_trial)
        && owner_before == al_r29_mask_root(r43.owners)
        && superset_before == al_r42_superset_root(r43.r42.superset);

    const std::array<const char*, 10U> expected_routes{{
        "FILTER_PARENT_REJECTED",
        "FILTER_SOURCE_REJECTED",
        "FILTER_COORDINATE_REJECTED",
        "FILTER_MODEL_REJECTED",
        "FILTER_TOPOLOGY_REJECTED",
        "FILTER_CONTACT_REJECTED",
        "FILTER_RESTORATION_REQUIRED",
        "FILTER_UPDATE_REJECTED",
        "FILTER_WORK_REJECTED",
        "FILTER_FEASIBILITY_STEP_CANDIDATE"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t index_value = 0U;
         index_value < expected_routes.size(); ++index_value) {
        std::array<bool, 9U> gates{};
        gates.fill(true);
        if (index_value < gates.size()) gates[index_value] = false;
        const std::string observed = al_r46_route(gates[0], gates[1],
            gates[2], gates[3], gates[4], gates[5], gates[6], gates[7],
            gates[8]);
        routes_exact = routes_exact
            && observed == expected_routes[index_value];
        route_material << expected_routes[index_value] << ':' << observed
                       << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r46_route(parent_exact, source_exact,
        coordinate_exact, model_exact, topology_exact, contact_exact,
        envelope_exact, update_exact, work_exact && rollback);
    const bool passed = identity && parent_exact && source_exact && dense
        && coordinate_exact && model_exact && topology_exact && contact_exact
        && update_exact && work_exact && rollback && routes_exact;
    std::ostringstream coordinate_material;
    coordinate_material << binary64_bits(source_f) << ':'
                        << binary64_bits(trial_f) << ':'
                        << binary64_bits(source_metric.norm) << ':'
                        << binary64_bits(trial_metric.norm) << ':'
                        << binary64_bits(source_metric.psi) << ':'
                        << binary64_bits(trial_metric.psi) << ':'
                        << binary64_bits(objective_reduction);
    const std::string coordinate_root =
        sha256_hex(coordinate_material.str());
    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R46_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << coordinate_root << ':' << source_f << ':' << trial_f << ':'
             << source_metric.norm << ':' << trial_metric.norm << ':'
             << objective_reduction << '|' << r43.predicted_psi_reduction
             << ':' << r43.actual_psi_reduction << ':' << r43.model_ratio
             << '|' << gamma_tests << ':' << acceptable_margins << ':'
             << h_branches << ':' << f_branches << ':'
             << strongest.h_slack << ':' << strongest.f_slack << ':'
             << resolution_guard << ':' << margin_root << '|'
             << empty_filter_root << ':' << source_entry_root << ':'
             << next_filter_root << '|' << work_exact << ':' << rollback
             << ':' << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r46_filter_globalization.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R46_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"coordinates\":{\"source_f\":" << source_f
           << ",\"trial_f\":" << trial_f
           << ",\"objective_reduction\":" << objective_reduction
           << ",\"source_h\":" << source_metric.norm
           << ",\"trial_h\":" << trial_metric.norm
           << ",\"source_psi\":" << source_metric.psi
           << ",\"trial_psi\":" << trial_metric.psi
           << ",\"source_active\":" << source_metric.active
           << ",\"trial_active\":" << trial_metric.active
           << ",\"gates\":{\"workspace\":"
           << (workspace_exact ? "true" : "false")
           << ",\"source_metric\":" << (source_metric.finite ? "true" : "false")
           << ",\"trial_metric\":" << (trial_metric.finite ? "true" : "false")
           << ",\"source_inner\":" << (source_inner.passed ? "true" : "false")
           << ",\"trial_inner\":" << (trial_inner.passed ? "true" : "false")
           << ",\"lower_bound\":" << (lower_bound_exact ? "true" : "false")
           << ",\"divided_finite\":"
           << (divided.value.finite_values ? "true" : "false")
           << ",\"divided_repeat\":" << (divided_exact ? "true" : "false")
           << ",\"norm_correspondence\":"
           << (norm_correspondence ? "true" : "false") << "}"
           << ",\"root\":\"" << coordinate_root << "\"}"
           << ",\"workspace_gates\":{\"dense\":"
           << (dense ? "true" : "false") << ",\"index\":"
           << (index.passed ? "true" : "false") << ",\"binding\":"
           << (binding.passed ? "true" : "false") << ",\"superset\":"
           << (superset.passed ? "true" : "false")
           << ",\"superset_root\":" << (superset_root_exact ? "true" : "false")
           << ",\"source_workspace\":"
           << (source_workspace_passed ? "true" : "false")
           << ",\"trial_workspace\":"
           << (trial_workspace_passed ? "true" : "false")
           << ",\"source_coverage\":"
           << (source_coverage.exact ? "true" : "false")
           << ",\"trial_coverage\":"
           << (trial_coverage.exact ? "true" : "false")
           << ",\"source_pairs\":" << (source_pairs_exact ? "true" : "false")
           << ",\"trial_pairs\":" << (trial_pairs_exact ? "true" : "false")
           << "}"
           << ",\"model\":{\"predicted_psi_reduction\":"
           << r43.predicted_psi_reduction
           << ",\"actual_psi_reduction\":"
           << r43.actual_psi_reduction << ",\"rho\":"
           << r43.model_ratio << "}"
           << ",\"filter\":{\"gamma_tests\":" << gamma_tests
           << ",\"acceptable\":" << acceptable_margins
           << ",\"h_branches\":" << h_branches
           << ",\"f_branches\":" << f_branches
           << ",\"strongest_gamma\":0.5,\"strongest_h_slack\":"
           << strongest.h_slack << ",\"strongest_f_slack\":"
           << strongest.f_slack << ",\"resolution_guard\":"
           << resolution_guard << ",\"margin_root\":\"" << margin_root
           << "\",\"initial_root\":\"" << empty_filter_root
           << "\",\"source_entry_root\":\"" << source_entry_root
           << "\",\"hypothetical_next_root\":\"" << next_filter_root
           << "\",\"admitted\":"
           << (envelope_exact ? "true" : "false") << "}"
           << ",\"topology\":{\"source_coverage\":\""
           << source_coverage.root << "\",\"trial_coverage\":\""
           << trial_coverage.root << "\"}"
           << ",\"contact\":{\"tests\":" << contact_audit.tests
           << ",\"new\":" << contact_audit.newly_penetrating
           << ",\"worsened\":" << contact_audit.worsened << "}"
           << ",\"work\":{\"workspaces\":" << work.workspace_builds
           << ",\"releases\":" << work.workspace_releases
           << ",\"maximum_live\":" << work.maximum_live_workspaces
           << ",\"exact\":" << (work_exact ? "true" : "false") << "}"
           << ",\"route_cases\":" << expected_routes.size()
           << ",\"route_root\":\"" << route_root
           << "\",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback ? "true" : "false")
           << ",\"route\":\"" << route
           << "\",\"correction_applied\":false,\"candidate_committed\":false"
              ",\"filter_runtime_committed\":false,\"following_outer_executed\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    if (replay_capture != nullptr) {
        replay_capture->captured = passed;
        replay_capture->r45 = std::move(capture);
        replay_capture->source_f = source_f;
        replay_capture->trial_f = trial_f;
        replay_capture->objective_reduction = objective_reduction;
        replay_capture->source_h = source_metric.norm;
        replay_capture->trial_h = trial_metric.norm;
        replay_capture->source_psi = source_metric.psi;
        replay_capture->trial_psi = trial_metric.psi;
        replay_capture->filter_admitted = envelope_exact;
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_filter_globalization_controls() {
    return run_al_filter_globalization_controls_impl(nullptr);
}
