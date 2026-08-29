namespace {

constexpr const char* B4E2D7R19R56_IDENTITY_SHA256 =
    "d7fd1d53edb0f1467c99fd6b76e42c96c1775cda9ab5ea95821c89301acfef6e";
constexpr const char* B4E2D7R19R56_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r56-joint-witness-certificate|v1|parent=ec8cd141:fce9bec7ef370caed4904cedf61851f0fb87bf539960134fabefbb9dc94d6db8:fb549fd34090aae8a573715991e7b30059b7c779c1e4123a07b652509fb3c83e:CONTACT_CONSTRAINED_DYKSTRA_CANDIDATE|source=r55-state-0edef9f324518fd77716b8fbc420a1c9b1ccd4187fef28af222a0d2ea840c1b8;cycle64-record-dd2db6e5ffe417ac6fc2b5a6e1eae884607cb8400aacd4d9f3e2959b5945bc6d;witness-19da2a335377f195bc0ce110977d19f6fa30223054d02d485a0b790123270406;correction-8ae05f144d760cfcfd3232dc0113365dff7e3a680769442cbdef1e0c6977b387;boxdual-512c77152fb3f4e8d55603cb765f540dd36cc30e18bb2465e910112a11aacb7d;master-38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28;raw308;directed476;outside0;h7.3368315976e-20;maximumupper4.1425104588e-20|oracle=exact-r55-cycle64-witness;captured-directed-binary64;fresh-pair-once-binary64;directed-binary64-terms-compensated-binary128-fold;frozen-coefficient-directed-binary128-recompute-compensated;stable-topology|rows=raw-positive-count;maximum-raw-and-row-per-representation;pair-direct-maximum-difference;pair-full128-sign-disagreements;full128-resolved-positive-negative-unresolved;full128-lower-and-upper-maxima;current-gamma-bound;bound-only-positive-count;roots|selection=pair-directed-difference-operator-alignment-first;else-full128-resolved-positive-high-precision-raw-residual;else-full128-unresolved-sign-unresolved;else-binary64-raw-positive-cancellation;else-current-directed-positive-enclosure-tightening;else-restoration-next-trqp-compatible|controls=r55-parent-bytes;source;workspace;topology;master;cycle64-capture;witness;pair-once;directed-capture;term128-fold;full128-recompute;row-comparison;enclosure;work;rollback;route-precedence|routes=joint-certificate-parent-rejected;joint-certificate-source-rejected;joint-certificate-workspace-rejected;joint-certificate-topology-rejected;joint-certificate-master-rejected;joint-certificate-witness-rejected;joint-certificate-pair-rejected;joint-certificate-directed-rejected;joint-certificate-binary128-rejected;joint-certificate-row-comparison-rejected;joint-certificate-enclosure-rejected;joint-certificate-work-rejected;joint-witness-operator-alignment-required;joint-witness-high-precision-raw-residual-confirmed;joint-witness-high-precision-sign-unresolved;joint-witness-binary64-cancellation-confirmed;joint-witness-directed-enclosure-tightening-required;restoration-next-trqp-compatible-candidate|precedence=parent,source,workspace,topology,master,witness,pair,directed,binary128,row-comparison,enclosure,work,alignment,raw-residual,unresolved,cancellation,enclosure-tightening,compatibility|work=parent-r55-replays1;moved-workspace1;workspace-release1;fresh-pair-once-jvp1;binary128-directed-row-traversals1;captured-directed-reused;pairpasses1;quad-row-passes1;new-density-sweeps0;new-box-blocks0;new-residual-refresh0;new-projection0;new-row-vjp0;new-gram-jvp0;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53-r54-r55=unchanged;certificate=current-directed-unchanged;binary128=offline-only;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;gamma-change=none;capacity-change=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-joint-witness-certificate-classification-only)IDENTITY";

std::string al_r56_route(bool parent, bool source, bool workspace,
    bool topology, bool master, bool witness, bool pair, bool directed,
    bool binary128, bool comparison, bool enclosure, bool work,
    bool alignment, bool raw_residual, bool unresolved, bool cancellation,
    bool tightening) {
    if (!parent) return "JOINT_CERTIFICATE_PARENT_REJECTED";
    if (!source) return "JOINT_CERTIFICATE_SOURCE_REJECTED";
    if (!workspace) return "JOINT_CERTIFICATE_WORKSPACE_REJECTED";
    if (!topology) return "JOINT_CERTIFICATE_TOPOLOGY_REJECTED";
    if (!master) return "JOINT_CERTIFICATE_MASTER_REJECTED";
    if (!witness) return "JOINT_CERTIFICATE_WITNESS_REJECTED";
    if (!pair) return "JOINT_CERTIFICATE_PAIR_REJECTED";
    if (!directed) return "JOINT_CERTIFICATE_DIRECTED_REJECTED";
    if (!binary128) return "JOINT_CERTIFICATE_BINARY128_REJECTED";
    if (!comparison) return "JOINT_CERTIFICATE_ROW_COMPARISON_REJECTED";
    if (!enclosure) return "JOINT_CERTIFICATE_ENCLOSURE_REJECTED";
    if (!work) return "JOINT_CERTIFICATE_WORK_REJECTED";
    if (alignment) return "JOINT_WITNESS_OPERATOR_ALIGNMENT_REQUIRED";
    if (raw_residual)
        return "JOINT_WITNESS_HIGH_PRECISION_RAW_RESIDUAL_CONFIRMED";
    if (unresolved) return "JOINT_WITNESS_HIGH_PRECISION_SIGN_UNRESOLVED";
    if (cancellation)
        return "JOINT_WITNESS_BINARY64_CANCELLATION_CONFIRMED";
    if (tightening)
        return "JOINT_WITNESS_DIRECTED_ENCLOSURE_TIGHTENING_REQUIRED";
    return "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE";
}

struct ALR56ReplayCapture {
    bool captured = false;
    ALR55ReplayCapture r55;
    ALR53Result decomposition;
};

} // namespace

SplitBoundaryReport run_al_joint_witness_certificate_controls_impl(
    ALR56ReplayCapture* replay_capture) {
    const bool identity = sha256_hex(B4E2D7R19R56_IDENTITY_PROJECTION)
        == B4E2D7R19R56_IDENTITY_SHA256;
    ALR55ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_contact_constrained_dykstra_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "fce9bec7ef370caed4904cedf61851f0fb87bf539960134fabefbb9dc94d6db8"
        && parent.json.find(
            "fb549fd34090aae8a573715991e7b30059b7c779c1e4123a07b652509fb3c83e")
            != std::string::npos
        && parent.json.find("CONTACT_CONSTRAINED_DYKSTRA_CANDIDATE")
            != std::string::npos;

    ALR55Result& r55 = capture.result;
    ALR54ReplayCapture& r54_capture = capture.r54;
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
    const std::string witness_before =
        al_binary64_vec3_root(r55.diagnostic_witness);
    const std::string correction_before =
        al_binary64_vec3_root(r55.diagnostic_correction);
    const std::string box_dual_before =
        al_binary64_vec3_root(r55.diagnostic_box_dual);

    const bool source_exact = parent_exact && r30.captured
        && r55.state_root
            == "0edef9f324518fd77716b8fbc420a1c9b1ccd4187fef28af222a0d2ea840c1b8"
        && r55.records.size() == 4U && r55.records.back().cycle == 64U
        && r55.records.back().root
            == "dd2db6e5ffe417ac6fc2b5a6e1eae884607cb8400aacd4d9f3e2959b5945bc6d"
        && witness_before
            == "19da2a335377f195bc0ce110977d19f6fa30223054d02d485a0b790123270406"
        && correction_before
            == "8ae05f144d760cfcfd3232dc0113365dff7e3a680769442cbdef1e0c6977b387"
        && box_dual_before
            == "512c77152fb3f4e8d55603cb765f540dd36cc30e18bb2465e910112a11aacb7d"
        && r55.records.back().witness_root == witness_before
        && r55.records.back().correction_root == correction_before
        && r55.records.back().box_dual_root == box_dual_before
        && r55.diagnostic_audit.passed
        && r55.diagnostic_audit.primal.raw_active == 308U
        && r55.diagnostic_audit.active.size() == 476U
        && r55.records.back().raw_outside_master == 0U
        && r55.records.back().directed_outside_master == 0U
        && r51.union_root
            == "38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28";

    std::string dense_root;
    const bool dense = identity && al_r53_dense_controls(&dense_root);
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
    const bool witness_exact = master_exact
        && r55.diagnostic_witness.size() == moved.neighborhood.fluid.size()
        && r55.diagnostic_correction.size() == moved.neighborhood.fluid.size()
        && r55.diagnostic_box_dual.size() == moved.neighborhood.fluid.size()
        && al_r48_vec_finite(r55.diagnostic_witness)
        && al_r48_vec_finite(r55.diagnostic_correction)
        && al_r48_vec_finite(r55.diagnostic_box_dual);

    ALR53Result decomposition;
    if (witness_exact) decomposition = al_r53_decompose(
        moved, r55.diagnostic_witness, r55.diagnostic_audit);
    if (moved.passed) release_al_normalized_workspace(moved, &workspace_work);

    const bool alignment =
        decomposition.pair_direct_difference.value
            != static_cast<Binary128>(0.0)
        || decomposition.pair_root != decomposition.directed_root;
    const bool raw_residual = !alignment
        && decomposition.full128_resolved_positive > 0U;
    const bool unresolved = !alignment && !raw_residual
        && decomposition.full128_unresolved > 0U;
    const bool cancellation = !alignment && !raw_residual && !unresolved
        && decomposition.pair_positive > 0U;
    const bool tightening = !alignment && !raw_residual && !unresolved
        && !cancellation && !r55.diagnostic_audit.active.empty();

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
        && decomposition.pair_passes == 1U
        && decomposition.quad_row_passes == 1U;
    const bool rollback =
        position_before == al_binary64_vec3_root(r30.position)
        && trial_before == al_binary64_vec3_root(r43.projected_trial)
        && owner_before == al_r29_mask_root(r43.owners)
        && superset_before == al_r42_superset_root(r43.r42.superset)
        && anchor_before == al_binary64_vec3_root(r50.witness)
        && witness_before == al_binary64_vec3_root(r55.diagnostic_witness)
        && correction_before == al_binary64_vec3_root(r55.diagnostic_correction)
        && box_dual_before == al_binary64_vec3_root(r55.diagnostic_box_dual);

    const std::array<const char*, 18U> expected_routes{{
        "JOINT_CERTIFICATE_PARENT_REJECTED",
        "JOINT_CERTIFICATE_SOURCE_REJECTED",
        "JOINT_CERTIFICATE_WORKSPACE_REJECTED",
        "JOINT_CERTIFICATE_TOPOLOGY_REJECTED",
        "JOINT_CERTIFICATE_MASTER_REJECTED",
        "JOINT_CERTIFICATE_WITNESS_REJECTED",
        "JOINT_CERTIFICATE_PAIR_REJECTED",
        "JOINT_CERTIFICATE_DIRECTED_REJECTED",
        "JOINT_CERTIFICATE_BINARY128_REJECTED",
        "JOINT_CERTIFICATE_ROW_COMPARISON_REJECTED",
        "JOINT_CERTIFICATE_ENCLOSURE_REJECTED",
        "JOINT_CERTIFICATE_WORK_REJECTED",
        "JOINT_WITNESS_OPERATOR_ALIGNMENT_REQUIRED",
        "JOINT_WITNESS_HIGH_PRECISION_RAW_RESIDUAL_CONFIRMED",
        "JOINT_WITNESS_HIGH_PRECISION_SIGN_UNRESOLVED",
        "JOINT_WITNESS_BINARY64_CANCELLATION_CONFIRMED",
        "JOINT_WITNESS_DIRECTED_ENCLOSURE_TIGHTENING_REQUIRED",
        "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t route_index = 0U;
         route_index < expected_routes.size(); ++route_index) {
        std::array<bool, 12U> gates{};
        gates.fill(true);
        if (route_index < gates.size()) gates[route_index] = false;
        const std::string observed = al_r56_route(
            gates[0], gates[1], gates[2], gates[3], gates[4], gates[5],
            gates[6], gates[7], gates[8], gates[9], gates[10], gates[11],
            route_index == 12U, route_index == 13U, route_index == 14U,
            route_index == 15U, route_index == 16U);
        routes_exact = routes_exact && observed == expected_routes[route_index];
        route_material << expected_routes[route_index] << ':' << observed << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r56_route(parent_exact, source_exact,
        workspace_exact, topology_exact, master_exact, witness_exact,
        decomposition.pair_exact, decomposition.directed_exact,
        decomposition.binary128_exact, decomposition.comparison_exact,
        decomposition.enclosure_exact, work_exact && rollback, alignment,
        raw_residual, unresolved, cancellation, tightening);
    const bool passed = identity && parent_exact && source_exact && dense
        && workspace_exact && topology_exact && master_exact && witness_exact
        && decomposition.pair_exact && decomposition.directed_exact
        && decomposition.binary128_exact && decomposition.comparison_exact
        && decomposition.enclosure_exact && work_exact && rollback
        && routes_exact;

    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R56_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << r55.state_root << ':' << r55.records.back().root << ':'
             << witness_before << ':' << correction_before << ':'
             << box_dual_before << '|'
             << decomposition.pair_positive << ':'
             << decomposition.directed_positive << ':'
             << decomposition.term128_positive << ':'
             << decomposition.full128_resolved_positive << ':'
             << decomposition.full128_resolved_negative << ':'
             << decomposition.full128_unresolved << ':'
             << decomposition.pair_full_sign_disagreements << ':'
             << decomposition.directed_bound_only_positive << '|'
             << decomposition.comparison_root << ':'
             << decomposition.pair_root << ':' << decomposition.directed_root
             << ':' << decomposition.term128_root << ':'
             << decomposition.full128_root << '|'
             << decomposition.pair_passes << ':'
             << decomposition.quad_row_passes << ':' << alignment << ':'
             << raw_residual << ':' << unresolved << ':' << cancellation << ':'
             << tightening << ':' << work_exact << ':' << rollback << ':'
             << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r56_joint_witness_certificate.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R56_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source\":{\"state_root\":\"" << r55.state_root
           << "\",\"record_root\":\"" << r55.records.back().root
           << "\",\"witness_root\":\"" << witness_before
           << "\",\"correction_root\":\"" << correction_before
           << "\",\"box_dual_root\":\"" << box_dual_before << "\"}"
           << ",\"rows\":{\"pair_positive\":"
           << decomposition.pair_positive
           << ",\"directed_positive\":" << decomposition.directed_positive
           << ",\"term128_positive\":" << decomposition.term128_positive
           << ",\"full128_resolved_positive\":"
           << decomposition.full128_resolved_positive
           << ",\"full128_resolved_negative\":"
           << decomposition.full128_resolved_negative
           << ",\"full128_unresolved\":"
           << decomposition.full128_unresolved
           << ",\"pair_full_sign_disagreements\":"
           << decomposition.pair_full_sign_disagreements
           << ",\"directed_bound_only_positive\":"
           << decomposition.directed_bound_only_positive << "}"
           << ",\"maxima\":{\"pair_raw\":"
           << static_cast<double>(decomposition.pair_maximum.value)
           << ",\"pair_row\":" << decomposition.pair_maximum.row
           << ",\"directed_raw\":"
           << static_cast<double>(decomposition.directed_maximum.value)
           << ",\"directed_row\":" << decomposition.directed_maximum.row
           << ",\"term128_raw\":"
           << static_cast<double>(decomposition.term128_maximum.value)
           << ",\"term128_row\":" << decomposition.term128_maximum.row
           << ",\"full128_raw\":"
           << static_cast<double>(decomposition.full128_maximum.value)
           << ",\"full128_row\":" << decomposition.full128_maximum.row
           << ",\"full128_lower\":"
           << static_cast<double>(decomposition.full128_lower_maximum.value)
           << ",\"full128_lower_row\":"
           << decomposition.full128_lower_maximum.row
           << ",\"full128_upper\":"
           << static_cast<double>(decomposition.full128_upper_maximum.value)
           << ",\"full128_upper_row\":"
           << decomposition.full128_upper_maximum.row
           << ",\"full128_lower_hex\":\""
           << al_binary128_hex(decomposition.full128_lower_maximum.value)
           << "\",\"full128_upper_hex\":\""
           << al_binary128_hex(decomposition.full128_upper_maximum.value)
           << "\",\"gamma_bound\":"
           << static_cast<double>(decomposition.bound_maximum.value)
           << ",\"gamma_row\":" << decomposition.bound_maximum.row
           << ",\"directed_upper\":"
           << static_cast<double>(decomposition.upper_maximum.value)
           << ",\"upper_row\":" << decomposition.upper_maximum.row << "}"
           << ",\"differences\":{\"pair_direct\":"
           << static_cast<double>(decomposition.pair_direct_difference.value)
           << ",\"pair_direct_row\":"
           << decomposition.pair_direct_difference.row
           << ",\"direct_term128\":"
           << static_cast<double>(decomposition.direct_term128_difference.value)
           << ",\"direct_term128_row\":"
           << decomposition.direct_term128_difference.row
           << ",\"term128_full128\":"
           << static_cast<double>(decomposition.term128_full_difference.value)
           << ",\"term128_full128_row\":"
           << decomposition.term128_full_difference.row << "}"
           << ",\"roots\":{\"pair\":\"" << decomposition.pair_root
           << "\",\"directed\":\"" << decomposition.directed_root
           << "\",\"term128\":\"" << decomposition.term128_root
           << "\",\"full128\":\"" << decomposition.full128_root
           << "\",\"comparison\":\"" << decomposition.comparison_root
           << "\"}"
           << ",\"work\":{\"workspace_builds\":"
           << workspace_work.workspace_builds << ",\"workspace_releases\":"
           << workspace_work.workspace_releases << ",\"pair_passes\":"
           << decomposition.pair_passes << ",\"quad_row_passes\":"
           << decomposition.quad_row_passes
           << ",\"exact\":" << (work_exact ? "true" : "false") << "}"
           << ",\"dense_root\":\"" << dense_root << "\""
           << ",\"route_cases\":" << expected_routes.size()
           << ",\"route_root\":\"" << route_root
           << "\",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback ? "true" : "false")
           << ",\"route\":\"" << route
           << "\",\"current_directed_certificate_unchanged\":true"
              ",\"binary128_offline_only\":true"
              ",\"normal_witness_applied\":false"
              ",\"r43_restoration_committed\":false"
              ",\"restoration_exit_authorized\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    if (replay_capture != nullptr) {
        replay_capture->captured = passed;
        replay_capture->r55 = std::move(capture);
        replay_capture->decomposition = std::move(decomposition);
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_joint_witness_certificate_controls() {
    return run_al_joint_witness_certificate_controls_impl(nullptr);
}
