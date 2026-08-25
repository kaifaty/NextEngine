namespace {

constexpr const char* B4E2D7R19R58_IDENTITY_SHA256 =
    "a0327b38e7d2f7552bfc223efdb99a796e968ec8cfa8b70560ac32493206542f";
constexpr const char* B4E2D7R19R58_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r58-directed-enclosure-fixed-point|v1|parent=5aaae13e:a1930bc92665cb08f5f46ff53e9b53da48d09438d3bc52c25ec848a2db0a6b5c:666e9f62ab3e0ac027bd855893f90c08dc2d61433f913170e736c6d9e50a39b6:CERTIFICATE_REFINEMENT_ENCLOSURE_FIXED_POINT_REQUIRED|source=r57-state-952d2541ed707bc36bec78e10f32ae6c60bd062c081f3811a611fe68ead27d63;cycle64-record-856dc573d7b1320659a554d0b33522a74b03eb34dc1747518e7692a684158231;witness-e4c5aa2d5cdddaaeb55c620d37f43856d5ed797f199fad2253175bef2ddd341d;correction-18bc74476b5887a4b832e226c8f09984a91e91d84d8636d4ceffd4a9c824d53b;boxdual-e58553b7864502442b95d8b20e20e16a255afe0b59f64d43bf1f3e6fc7ca7d5f;upper=exact-transitive-capture;active245;raw0;maximum-upper6.3619020626182659e-24;full128-positive0;negative6000;unresolved0;maximum-raw-5.1986625146610136e-22|map=T-r57;evaluate-T-of-r57-selected-once|problem=d0;min0.5*norm2delta;r57-directed-upper-plus-A-delta<=0-on-master494;box=contact-lower-minus-r57-witness<=delta<=contact-upper-minus-r57-witness;normal-ball-audit-only|solver=reuse-r57-zero-initialized-grouped-box-dykstra;density-lambda0;box-correction0;delta0;stable-master-order;fresh-residual-after-box|depth=64;checkpoints8,16,32,64;no-tolerance-stop;first-certified-else64;no-third-outer|oracle=unchanged-directed-audit-each-checkpoint;selected-fresh-pair-once-binary64;selected-compensated-binary128-directed-traversal;frozen-coefficient-binary128-recompute;current-gamma-unchanged|contraction=selected-active<245-and-selected-maximum-upper<6.3619020626182659e-24;stall=remaining-positive-and-not-contraction|controls=r57-parent-bytes;source;workspace;topology;master;geometry;core-reuse;dykstra;box;ball;primal;binary128;selection;work;rollback;route-precedence|routes=fixed-point-parent-rejected;fixed-point-source-rejected;fixed-point-workspace-rejected;fixed-point-topology-rejected;fixed-point-master-rejected;fixed-point-geometry-rejected;fixed-point-dykstra-rejected;fixed-point-box-rejected;fixed-point-ball-rejected;fixed-point-primal-audit-rejected;fixed-point-binary128-rejected;fixed-point-selection-rejected;fixed-point-work-rejected;fixed-point-master-expansion-required;fixed-point-ball-integration-required;restoration-next-trqp-compatible-candidate;fixed-point-raw-regression;fixed-point-sign-unresolved;fixed-point-stalled;fixed-point-contraction-candidate;fixed-point-reference-retained|precedence=parent,source,workspace,topology,master,geometry,dykstra,box,ball,primal,binary128,selection,work,outside-master,ball-integration,certified,raw-positive,unresolved,stall,contraction,reference|work=parent-r57-replays1;moved-workspace1;workspace-release1;density-sweeps64;box-blocks64;fresh-residual-jvp64;checkpoint-directed-jvp4;selected-fresh-pair-jvp1;binary128-row-traversal1;new-pairpasses69;quad-row-passes1;new-row-vjp0;new-gram-jvp0;new-basis0;new-projection0;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53-r54-r55-r56-r57=unchanged;certificate=current-directed-unchanged;binary128=offline-only;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;gamma-change=none;capacity-change=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-directed-enclosure-fixed-point-classification-only)IDENTITY";

std::string al_r58_route(bool parent, bool source, bool workspace,
    bool topology, bool master, bool geometry, bool dykstra, bool box,
    bool ball, bool primal, bool binary128, bool selection, bool work,
    bool outside_master, bool ball_integration, bool certified,
    bool raw_positive, bool unresolved, bool stalled, bool contraction) {
    if (!parent) return "FIXED_POINT_PARENT_REJECTED";
    if (!source) return "FIXED_POINT_SOURCE_REJECTED";
    if (!workspace) return "FIXED_POINT_WORKSPACE_REJECTED";
    if (!topology) return "FIXED_POINT_TOPOLOGY_REJECTED";
    if (!master) return "FIXED_POINT_MASTER_REJECTED";
    if (!geometry) return "FIXED_POINT_GEOMETRY_REJECTED";
    if (!dykstra) return "FIXED_POINT_DYKSTRA_REJECTED";
    if (!box) return "FIXED_POINT_BOX_REJECTED";
    if (!ball) return "FIXED_POINT_BALL_REJECTED";
    if (!primal) return "FIXED_POINT_PRIMAL_AUDIT_REJECTED";
    if (!binary128) return "FIXED_POINT_BINARY128_REJECTED";
    if (!selection) return "FIXED_POINT_SELECTION_REJECTED";
    if (!work) return "FIXED_POINT_WORK_REJECTED";
    if (outside_master) return "FIXED_POINT_MASTER_EXPANSION_REQUIRED";
    if (ball_integration) return "FIXED_POINT_BALL_INTEGRATION_REQUIRED";
    if (certified) return "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE";
    if (raw_positive) return "FIXED_POINT_RAW_REGRESSION";
    if (unresolved) return "FIXED_POINT_SIGN_UNRESOLVED";
    if (stalled) return "FIXED_POINT_STALLED";
    if (contraction) return "FIXED_POINT_CONTRACTION_CANDIDATE";
    return "FIXED_POINT_REFERENCE_RETAINED";
}

} // namespace

SplitBoundaryReport run_al_directed_enclosure_fixed_point_controls() {
    const bool identity = sha256_hex(B4E2D7R19R58_IDENTITY_PROJECTION)
        == B4E2D7R19R58_IDENTITY_SHA256;
    ALR57ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_certificate_aware_refinement_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "a1930bc92665cb08f5f46ff53e9b53da48d09438d3bc52c25ec848a2db0a6b5c"
        && parent.json.find(
            "666e9f62ab3e0ac027bd855893f90c08dc2d61433f913170e736c6d9e50a39b6")
            != std::string::npos
        && parent.json.find(
            "CERTIFICATE_REFINEMENT_ENCLOSURE_FIXED_POINT_REQUIRED")
            != std::string::npos;

    ALR57Result& r57 = capture.result;
    ALR56ReplayCapture& r56_capture = capture.r56;
    ALR55ReplayCapture& r55_capture = r56_capture.r55;
    ALR54ReplayCapture& r54_capture = r55_capture.r54;
    ALR53ReplayCapture& r53_capture = r54_capture.r53;
    ALR51ReplayCapture& r51_capture = r53_capture.r52.r51;
    ALR51MasterResult& r51 = r51_capture.master;
    ALR50Generator& r50 = r51_capture.r50.generator;
    ALR43ReplayCapture& r43 =
        r51_capture.r50.r49.r48.r47.r46.r45.r44.r43;
    ALR40ReplayCapture& r40 = r43.r42.r40;
    ALR30RangeProjectionCapture& r30 =
        r40.r39.r38.r37.r36.r35.r34.r33.r32.r31.r30;

    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string trial_before = al_binary64_vec3_root(r43.projected_trial);
    const std::string owner_before = al_r29_mask_root(r43.owners);
    const std::string superset_before = al_r42_superset_root(r43.r42.superset);
    const std::string anchor_before = al_binary64_vec3_root(r50.witness);
    const std::string witness_before = al_binary64_vec3_root(r57.witness);
    const std::string upper_before = al_r10_dual_root(r57.terminal.upper);

    const bool source_exact = parent_exact && r30.captured
        && r57.state_root
            == "952d2541ed707bc36bec78e10f32ae6c60bd062c081f3811a611fe68ead27d63"
        && r57.records.size() == 4U && r57.selected_index == 3U
        && r57.records.back().root
            == "856dc573d7b1320659a554d0b33522a74b03eb34dc1747518e7692a684158231"
        && witness_before
            == "e4c5aa2d5cdddaaeb55c620d37f43856d5ed797f199fad2253175bef2ddd341d"
        && r57.records.back().correction_root
            == "18bc74476b5887a4b832e226c8f09984a91e91d84d8636d4ceffd4a9c824d53b"
        && r57.records.back().box_dual_root
            == "e58553b7864502442b95d8b20e20e16a255afe0b59f64d43bf1f3e6fc7ca7d5f"
        && r57.terminal.passed && r57.terminal.active.size() == 245U
        && r57.terminal.primal.raw_active == 0U
        && binary64_bits(r57.terminal.primal.maximum_upper)
            == binary64_bits(6.3619020626182659e-24)
        && capture.decomposition.full128_resolved_positive == 0U
        && capture.decomposition.full128_resolved_negative == 6000U
        && capture.decomposition.full128_unresolved == 0U
        && r51.union_root
            == "38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28";

    std::string dense_root;
    const bool dense = identity && al_r55_dense_controls(&dense_root);
    ALSparseWorkTrace workspace_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    const SmokeFixture fixture = make_b4e2d_dam_fixture();
    const JointStaticSupportIndex index = build_joint_static_support_index(
        tagged_points(fixture.boundary), &static_work);
    const JointStaticSupportBinding binding = bind_joint_static_support_index(
        &index, index.identity_sha256);
    JointNeighborhood superset = b4ep3_build_superset(
        tagged_points(r30.position), binding);
    JointNeighborhood moved_filtered = b4ep3_filter_superset(
        tagged_points(r43.projected_trial), superset);
    ALNormalizedWorkspace moved = build_al_normalized_sparse_workspace(
        r43.projected_trial, binding, r30.dual, r30.theta,
        &workspace_work, &static_work, &adjacency_work);
    const bool workspace_exact = source_exact && dense && index.passed
        && binding.passed && superset.passed && moved.passed;
    const bool topology_exact = workspace_exact
        && al_r42_superset_root(superset) == superset_before
        && moved_filtered.pairs == moved.neighborhood.pairs;
    const bool master_exact = topology_exact && r51.rows.size() == 494U
        && r51.cache.size() == 494U
        && al_r51_cache_root(r51.cache) == r51.extended_cache_root;

    std::vector<Vec3> absolute_lower(r43.projected_trial.size());
    std::vector<Vec3> absolute_upper(r43.projected_trial.size());
    bool geometry_exact = master_exact
        && r57.witness.size() == r43.projected_trial.size();
    for (std::size_t i = 0U; i < r43.projected_trial.size(); ++i)
        for (int axis = 0; axis < 3; ++axis) {
            const double y = al_r41_component(r43.projected_trial[i], axis);
            const double low_face = al_r41_component(fixture.contact_low, axis);
            const double high_face =
                al_r41_component(fixture.contact_high, axis);
            double low = (low_face - y) / SPACING;
            double high = (high_face - y) / SPACING;
            if (y < low_face) low = 0.0;
            else if (y > high_face) high = 0.0;
            al_r43_component(absolute_lower[i], axis) = low;
            al_r43_component(absolute_upper[i], axis) = high;
            const double witness = i < r57.witness.size()
                ? al_r41_component(r57.witness[i], axis)
                : 0.0;
            geometry_exact = geometry_exact && std::isfinite(low)
                && std::isfinite(high) && low <= witness && witness <= high;
        }

    ALR57Result result;
    if (geometry_exact)
        result = al_r57_generate(moved, r57.witness, r57.terminal, r51,
            absolute_lower, absolute_upper);
    ALR53Result decomposition;
    if (result.selection_exact)
        decomposition = al_r53_decompose(moved, result.witness, result.terminal);
    if (moved.passed) release_al_normalized_workspace(moved, &workspace_work);

    const bool binary128_exact = decomposition.pair_exact
        && decomposition.directed_exact && decomposition.binary128_exact
        && decomposition.comparison_exact && decomposition.enclosure_exact;
    const bool work_exact = workspace_work.workspace_builds == 1U
        && workspace_work.workspace_releases == 1U
        && workspace_work.live_workspaces == 0U
        && workspace_work.maximum_live_workspaces == 1U
        && !workspace_work.lifecycle_underflow
        && workspace_work.all_pair_candidate_calls == 0U
        && static_work.static_index_builds == 1U
        && static_work.support_canonicalizations == 1U
        && static_work.workspace_builds == 1U
        && adjacency_work.workspace_builds == 1U
        && result.density_sweeps == 64U && result.box_blocks == 64U
        && result.residual_jvp == 64U && result.audit_jvp == 4U
        && result.pair_passes == 68U && decomposition.pair_passes == 1U
        && decomposition.quad_row_passes == 1U;
    const bool rollback =
        position_before == al_binary64_vec3_root(r30.position)
        && trial_before == al_binary64_vec3_root(r43.projected_trial)
        && owner_before == al_r29_mask_root(r43.owners)
        && superset_before == al_r42_superset_root(r43.r42.superset)
        && anchor_before == al_binary64_vec3_root(r50.witness)
        && witness_before == al_binary64_vec3_root(r57.witness)
        && upper_before == al_r10_dual_root(r57.terminal.upper);

    const bool raw_positive = binary128_exact
        && decomposition.full128_resolved_positive > 0U;
    const bool unresolved = binary128_exact && !raw_positive
        && decomposition.full128_unresolved > 0U;
    const ALR57Record* selected = result.selection_exact
        ? &result.records[result.selected_index]
        : nullptr;
    const bool fixed_contraction = selected != nullptr && !result.certified
        && !raw_positive && !unresolved && !selected->audit.active.empty()
        && selected->audit.active.size() < 245U
        && selected->audit.primal.maximum_upper
            < 6.3619020626182659e-24;
    const bool fixed_stalled = selected != nullptr && !result.certified
        && !raw_positive && !unresolved && !selected->audit.active.empty()
        && !fixed_contraction;

    const std::array<const char*, 21U> expected_routes{{
        "FIXED_POINT_PARENT_REJECTED", "FIXED_POINT_SOURCE_REJECTED",
        "FIXED_POINT_WORKSPACE_REJECTED", "FIXED_POINT_TOPOLOGY_REJECTED",
        "FIXED_POINT_MASTER_REJECTED", "FIXED_POINT_GEOMETRY_REJECTED",
        "FIXED_POINT_DYKSTRA_REJECTED", "FIXED_POINT_BOX_REJECTED",
        "FIXED_POINT_BALL_REJECTED", "FIXED_POINT_PRIMAL_AUDIT_REJECTED",
        "FIXED_POINT_BINARY128_REJECTED", "FIXED_POINT_SELECTION_REJECTED",
        "FIXED_POINT_WORK_REJECTED", "FIXED_POINT_MASTER_EXPANSION_REQUIRED",
        "FIXED_POINT_BALL_INTEGRATION_REQUIRED",
        "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE",
        "FIXED_POINT_RAW_REGRESSION", "FIXED_POINT_SIGN_UNRESOLVED",
        "FIXED_POINT_STALLED", "FIXED_POINT_CONTRACTION_CANDIDATE",
        "FIXED_POINT_REFERENCE_RETAINED"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t route_index = 0U;
         route_index < expected_routes.size(); ++route_index) {
        std::array<bool, 13U> gates{};
        gates.fill(true);
        if (route_index < gates.size()) gates[route_index] = false;
        const std::string observed = al_r58_route(
            gates[0], gates[1], gates[2], gates[3], gates[4], gates[5],
            gates[6], gates[7], gates[8], gates[9], gates[10], gates[11],
            gates[12], route_index == 13U, route_index == 14U,
            route_index == 15U, route_index == 16U, route_index == 17U,
            route_index == 18U, route_index == 19U);
        routes_exact = routes_exact && observed == expected_routes[route_index];
        route_material << expected_routes[route_index] << ':' << observed << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r58_route(parent_exact, source_exact,
        workspace_exact, topology_exact, master_exact, geometry_exact,
        result.dykstra_exact, result.box_exact, result.ball_exact,
        result.primal_exact, binary128_exact, result.selection_exact,
        work_exact && rollback, result.any_outside_master,
        result.any_ball_outside, result.certified, raw_positive, unresolved,
        fixed_stalled, fixed_contraction);
    const bool passed = identity && parent_exact && source_exact && dense
        && workspace_exact && topology_exact && master_exact && geometry_exact
        && result.master_exact && result.geometry_exact && result.dykstra_exact
        && result.box_exact && result.ball_exact && result.primal_exact
        && binary128_exact && result.selection_exact && work_exact && rollback
        && routes_exact;

    const std::string selected_root = selected != nullptr ? selected->root : "";
    const std::string selected_witness =
        selected != nullptr ? selected->witness_root : "";
    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R58_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << upper_before << ':' << result.state_root << ':'
             << result.records.size() << ':' << result.contraction << ':'
             << result.certified << ':' << result.selected_index << ':'
             << result.any_outside_master << ':' << result.any_ball_outside
             << '|' << selected_root << ':' << selected_witness << '|'
             << decomposition.pair_positive << ':'
             << decomposition.full128_resolved_positive << ':'
             << decomposition.full128_resolved_negative << ':'
             << decomposition.full128_unresolved << ':'
             << decomposition.directed_bound_only_positive << '|'
             << decomposition.comparison_root << ':'
             << result.density_sweeps << ':' << result.box_blocks << ':'
             << result.residual_jvp << ':' << result.audit_jvp << ':'
             << decomposition.pair_passes << ':'
             << decomposition.quad_row_passes << ':' << fixed_stalled << ':'
             << fixed_contraction << ':' << work_exact << ':' << rollback << ':'
             << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r58_directed_enclosure_fixed_point.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R58_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source\":{\"witness_root\":\"" << witness_before
           << "\",\"upper_root\":\"" << upper_before
           << "\",\"active\":" << r57.terminal.active.size()
           << ",\"maximum_upper\":"
           << r57.terminal.primal.maximum_upper << "},\"records\":[";
    for (std::size_t i = 0U; i < result.records.size(); ++i) {
        if (i != 0U) report << ',';
        const ALR57Record& record = result.records[i];
        report << "{\"cycle\":" << record.cycle
               << ",\"model_maximum\":" << record.after_box_master_maximum
               << ",\"psi\":" << record.audit.primal.psi
               << ",\"h\":" << record.audit.primal.h
               << ",\"maximum_upper\":"
               << record.audit.primal.maximum_upper
               << ",\"raw_active\":" << record.audit.primal.raw_active
               << ",\"directed_positive\":" << record.audit.active.size()
               << ",\"raw_outside_master\":"
               << record.raw_outside_master
               << ",\"directed_outside_master\":"
               << record.directed_outside_master
               << ",\"box_exact\":" << (record.box_exact ? "true" : "false")
               << ",\"ball_exact\":" << (record.ball_exact ? "true" : "false")
               << ",\"certified\":" << (record.certified ? "true" : "false")
               << ",\"witness_root\":\"" << record.witness_root
               << "\",\"correction_root\":\"" << record.correction_root
               << "\",\"box_dual_root\":\"" << record.box_dual_root
               << "\",\"root\":\"" << record.root << "\"}";
    }
    report << "],\"selection\":{\"certified\":"
           << (result.certified ? "true" : "false")
           << ",\"selected_index\":" << result.selected_index
           << ",\"fixed_point_stalled\":"
           << (fixed_stalled ? "true" : "false")
           << ",\"fixed_point_contraction\":"
           << (fixed_contraction ? "true" : "false") << "}"
           << ",\"sign\":{\"pair_positive\":"
           << decomposition.pair_positive
           << ",\"full128_resolved_positive\":"
           << decomposition.full128_resolved_positive
           << ",\"full128_resolved_negative\":"
           << decomposition.full128_resolved_negative
           << ",\"full128_unresolved\":"
           << decomposition.full128_unresolved
           << ",\"bound_only_positive\":"
           << decomposition.directed_bound_only_positive
           << ",\"maximum_raw\":"
           << static_cast<double>(decomposition.full128_maximum.value)
           << ",\"comparison_root\":\""
           << decomposition.comparison_root << "\"}"
           << ",\"work\":{\"density_sweeps\":" << result.density_sweeps
           << ",\"box_blocks\":" << result.box_blocks
           << ",\"residual_jvp\":" << result.residual_jvp
           << ",\"audit_jvp\":" << result.audit_jvp
           << ",\"terminal_pair_jvp\":" << decomposition.pair_passes
           << ",\"quad_row_passes\":" << decomposition.quad_row_passes
           << ",\"new_pair_passes\":"
           << result.pair_passes + decomposition.pair_passes
           << ",\"exact\":" << (work_exact ? "true" : "false") << "}"
           << ",\"state_root\":\"" << result.state_root
           << "\",\"dense_root\":\"" << dense_root
           << "\",\"route_cases\":" << expected_routes.size()
           << ",\"route_root\":\"" << route_root
           << "\",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback ? "true" : "false")
           << ",\"route\":\"" << route
           << "\",\"current_directed_certificate_unchanged\":true"
              ",\"binary128_offline_only\":true"
              ",\"third_fixed_point_outer_executed\":false"
              ",\"normal_witness_applied\":false"
              ",\"r43_restoration_committed\":false"
              ",\"restoration_exit_authorized\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    return {passed, report.str()};
}
