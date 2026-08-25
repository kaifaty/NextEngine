namespace {

constexpr const char* B4E2D7R19R61_IDENTITY_SHA256 =
    "ab51a986d1931551526becef8d38abd23a352b4d4ade994badb49c8b9533f504";
constexpr const char* B4E2D7R19R61_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r61-certificate-integration|v1|parent=0c77720d:6c6c62b6efb196d97dc4210ad7bcb56732f0c2d107a2df6aa10e9de84005c7f7:d93dbdc95eb66cf45f074541ae44b621755f45f011d2f1589af834a45238aec9:ROW_LOCAL_CERTIFICATE_VALIDATION_CANDIDATE|source=r58-state-a4343378e4055fdc06ccb134dd44f3322473c93138960ff31965f27e8fd22af2;witness-040cc9f0dc57f543c8c3f9e1324b2e68dcc827e120782f9813a7aa0652d884bd;legacy-upper-9e8ad22488a6a49f66f7d2734c9573692a807cbfdc6c3d09874b24a590151ff8;master-38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28;r59-histogram-3d4a798f7d9ea0fc71816091e786778f2368d8c0ff944b45200b004fa09dabdd;r59-comparison-5253b1c5a41fbb798cbe25df4a9c16afe663e4167b332aa9e0114c99b6de82de;r60-full-30ed1a37eb96f0967f10a36c9f014459ab4ffe4c5b26609f74a653a1798f3a50;r60-comparison-f0f18b90c88931ec6dc417ccf257109bfb094e541b17b0f69c06154dfab52ff8|owner=one-ALNormalizedWorkspace;one-witness;topology-and-certificate-same-lifecycle;no-caller-degree-cache|topology=offsets-size=rows+1;offset0=0;monotone;offset-last=directed-slot-count;all-directed-pair-indices-valid;derived-maximum=neighborhood-maximum;degree=offset-difference-including-radius-skips|operator=one-fresh-al-r29-directed-jvp;binary64;stable-row-slot-order|shared=raw=c+image.value;scale=abs(c)+image.absolute-sum;one-row-scan|shadow=gamma(16*maximum-degree+66)*scale;upper=raw+bound;must-equal-r58-upper-vector-active-root-count234-maximum1.3678206846699582e-24-row2522|candidate=gamma(16*row-degree+66)*scale;upper=raw+bound;must-equal-r59-local-active-root-count0-maximum-1.9354174334860891e-23-row4930;histogram-and-comparison|relations=candidate-upper<=shadow-upper-all;candidate-active-subset-shadow;shared-raw-image-root;shadow-certified-implies-candidate-certified|controls=r60-parent-bytes;source;workspace;topology;master;witness;owner;degree;shadow;candidate;relations;dense-controls;work;rollback;route-precedence|dense=retain-r60-degree-controls5;dual-path-valid-lower-and-max-degree;forced-offset-last-reject;cases6|routes=integration-parent-rejected;integration-source-rejected;integration-workspace-rejected;integration-topology-rejected;integration-master-rejected;integration-witness-rejected;integration-owner-rejected;integration-degree-controls-rejected;integration-shadow-rejected;integration-candidate-rejected;integration-relation-rejected;integration-controls-rejected;integration-work-rejected;topology-owned-row-local-audit-candidate;integration-reference-retained|precedence=parent,source,workspace,topology,master,witness,owner,degree,shadow,candidate,relation,controls,work,candidate,reference|work=parent-r60-replays1;moved-workspace1;workspace-release1;fresh-directed-jvp1;owner-row-scans1;dense-controls6;new-pairpasses1;new-quad-row-passes0;new-density-sweeps0;new-box-blocks0;new-vjp0;new-basis0;new-gram0;new-projection0;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53-r54-r55-r56-r57-r58-r59-r60=unchanged;legacy-replacement=none;witness-apply=none;restoration-promotion=none;restoration-exit=none;binary128=none;gamma-definition=unchanged;operator-arithmetic=unchanged;r43-restoration-commit=none;filter-runtime-commit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;capacity-change=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-topology-owned-certificate-integration-candidate-only)IDENTITY";

bool al_r61_topology_valid(const std::vector<std::uint32_t>& offsets,
    const std::vector<std::uint32_t>& directed_indices,
    std::size_t row_count, std::size_t pair_count,
    std::size_t declared_maximum) {
    if (row_count == 0U || offsets.size() != row_count + 1U
        || offsets.front() != 0U
        || static_cast<std::size_t>(offsets.back())
            != directed_indices.size()) return false;
    std::size_t maximum = 0U;
    for (std::size_t row = 0U; row < row_count; ++row) {
        if (offsets[row] > offsets[row + 1U]) return false;
        maximum = std::max(maximum, static_cast<std::size_t>(
            offsets[row + 1U] - offsets[row]));
    }
    if (maximum != declared_maximum) return false;
    return std::all_of(directed_indices.begin(), directed_indices.end(),
        [pair_count](std::uint32_t value) {
            return static_cast<std::size_t>(value) < pair_count;
        });
}

struct ALR61Audit {
    bool topology_exact = false;
    bool owner_exact = false;
    bool relation_exact = false;
    bool shadow_certified = false;
    bool candidate_certified = false;
    std::size_t pair_passes = 0U;
    std::size_t row_scans = 0U;
    std::size_t raw_positive = 0U;
    std::size_t shadow_positive = 0U;
    std::size_t candidate_positive = 0U;
    std::size_t relation_failures = 0U;
    std::size_t active_subset_failures = 0U;
    std::size_t minimum_degree = std::numeric_limits<std::size_t>::max();
    std::size_t maximum_degree = 0U;
    std::size_t distinct_degrees = 0U;
    std::size_t rows_below_maximum = 0U;
    double shadow_maximum = -std::numeric_limits<double>::infinity();
    std::size_t shadow_maximum_row = 0U;
    double candidate_maximum = -std::numeric_limits<double>::infinity();
    std::size_t candidate_maximum_row = 0U;
    ALLinearizedJvp image;
    std::vector<double> raw;
    std::vector<double> shadow_upper;
    std::vector<double> candidate_upper;
    std::string image_root;
    std::string raw_root;
    std::string shadow_root;
    std::string candidate_root;
    std::string shadow_active_root;
    std::string candidate_active_root;
    std::string histogram_root;
    std::string comparison_root;
};

ALR61Audit al_r61_topology_owned_audit(
    const ALNormalizedWorkspace& workspace,
    const std::vector<Vec3>& witness) {
    ALR61Audit result;
    const std::size_t count = workspace.neighborhood.fluid.size();
    result.topology_exact = workspace.passed && witness.size() == count
        && workspace.support.constraint.size() == count
        && al_r61_topology_valid(workspace.neighborhood.flat_offsets,
            workspace.neighborhood.flat_directed_pair_indices, count,
            workspace.neighborhood.pairs.size(),
            workspace.neighborhood.maximum_degree);
    if (!result.topology_exact) return result;

    result.image = al_r29_directed_jvp(workspace, witness);
    ++result.pair_passes;
    if (!result.image.passed || result.image.value.size() != count
        || result.image.absolute_sum.size() != count) return result;
    result.raw.resize(count);
    result.shadow_upper.resize(count);
    result.candidate_upper.resize(count);
    std::vector<std::uint8_t> shadow_mask(count, 0U);
    std::vector<std::uint8_t> candidate_mask(count, 0U);
    std::vector<std::size_t> histogram(
        workspace.neighborhood.maximum_degree + 1U, 0U);
    std::ostringstream histogram_material;
    std::ostringstream comparison_material;
    bool finite = true;
    const double shadow_gamma = gamma_factor(
        16U * workspace.neighborhood.maximum_degree + 66U);
    for (std::size_t row = 0U; row < count; ++row) {
        const std::size_t degree = static_cast<std::size_t>(
            workspace.neighborhood.flat_offsets[row + 1U]
            - workspace.neighborhood.flat_offsets[row]);
        ++histogram[degree];
        result.minimum_degree = std::min(result.minimum_degree, degree);
        result.maximum_degree = std::max(result.maximum_degree, degree);
        const double scale = std::abs(workspace.support.constraint[row])
            + result.image.absolute_sum[row];
        result.raw[row] = workspace.support.constraint[row]
            + result.image.value[row];
        const double shadow_bound = shadow_gamma * scale;
        result.shadow_upper[row] = result.raw[row] + shadow_bound;
        const double candidate_bound = gamma_factor(16U * degree + 66U)
            * scale;
        result.candidate_upper[row] = result.raw[row] + candidate_bound;
        if (result.raw[row] > 0.0) ++result.raw_positive;
        if (result.shadow_upper[row] > 0.0) {
            ++result.shadow_positive;
            shadow_mask[row] = 1U;
        }
        if (result.candidate_upper[row] > 0.0) {
            ++result.candidate_positive;
            candidate_mask[row] = 1U;
        }
        if (result.candidate_upper[row] > result.shadow_upper[row])
            ++result.relation_failures;
        if (candidate_mask[row] != 0U && shadow_mask[row] == 0U)
            ++result.active_subset_failures;
        if (result.shadow_upper[row] > result.shadow_maximum) {
            result.shadow_maximum = result.shadow_upper[row];
            result.shadow_maximum_row = row;
        }
        if (result.candidate_upper[row] > result.candidate_maximum) {
            result.candidate_maximum = result.candidate_upper[row];
            result.candidate_maximum_row = row;
        }
        comparison_material << row << ':' << degree << ':'
            << binary64_bits(result.raw[row]) << ':'
            << binary64_bits(shadow_bound) << ':'
            << binary64_bits(result.shadow_upper[row]) << ':'
            << binary64_bits(candidate_bound) << ':'
            << binary64_bits(result.candidate_upper[row]) << ';';
        finite = finite && std::isfinite(scale)
            && std::isfinite(result.raw[row])
            && std::isfinite(result.shadow_upper[row])
            && std::isfinite(result.candidate_upper[row]);
    }
    ++result.row_scans;
    for (std::size_t degree = 0U; degree < histogram.size(); ++degree)
        if (histogram[degree] > 0U) {
            ++result.distinct_degrees;
            histogram_material << degree << ':' << histogram[degree] << ';';
        }
    result.rows_below_maximum = count
        - histogram[workspace.neighborhood.maximum_degree];
    result.image_root = al_r10_dual_root(result.image.value);
    result.raw_root = al_r10_dual_root(result.raw);
    result.shadow_root = al_r10_dual_root(result.shadow_upper);
    result.candidate_root = al_r10_dual_root(result.candidate_upper);
    result.shadow_active_root = al_r29_mask_root(shadow_mask);
    result.candidate_active_root = al_r29_mask_root(candidate_mask);
    result.histogram_root = sha256_hex(histogram_material.str());
    result.comparison_root = sha256_hex(comparison_material.str());
    result.shadow_certified = result.shadow_positive == 0U;
    result.candidate_certified = result.candidate_positive == 0U;
    result.relation_exact = result.relation_failures == 0U
        && result.active_subset_failures == 0U
        && (!result.shadow_certified || result.candidate_certified);
    result.owner_exact = finite && result.row_scans == 1U
        && result.pair_passes == 1U
        && result.maximum_degree == workspace.neighborhood.maximum_degree
        && result.relation_exact;
    return result;
}

struct ALR61DenseControls {
    bool degree_controls = false;
    bool dual_path_valid = false;
    bool offset_last_rejected = false;
    std::size_t cases = 0U;
    std::string root;
};

ALR61DenseControls al_r61_dense_controls(
    const JointNeighborhood& neighborhood) {
    ALR61DenseControls result;
    const ALR60DegreeControls degrees = al_r60_degree_controls(neighborhood);
    result.degree_controls = degrees.nominal && degrees.dense_valid
        && degrees.undercount_rejected && degrees.overcount_rejected
        && degrees.decreasing_rejected && degrees.maximum_mismatch_rejected
        && degrees.dense_cases == 5U;
    result.cases = degrees.dense_cases;

    const std::vector<std::uint32_t> offsets{0U, 1U, 3U};
    const std::vector<std::uint32_t> indices{0U, 1U, 2U};
    const bool topology = al_r61_topology_valid(
        offsets, indices, 2U, 3U, 2U);
    const double scale = 1.0;
    const double shadow = gamma_factor(16U * 2U + 66U) * scale;
    const double lower = gamma_factor(16U * 1U + 66U) * scale;
    const double maximum = gamma_factor(16U * 2U + 66U) * scale;
    result.dual_path_valid = topology && lower < shadow
        && binary64_bits(maximum) == binary64_bits(shadow);
    std::vector<std::uint32_t> bad_offsets = offsets;
    bad_offsets.back() = 2U;
    result.offset_last_rejected = !al_r61_topology_valid(
        bad_offsets, indices, 2U, 3U, 2U);
    ++result.cases;

    std::ostringstream material;
    material << result.degree_controls << ':' << result.dual_path_valid << ':'
             << result.offset_last_rejected << ':' << result.cases << ':'
             << degrees.root;
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r61_route(bool parent, bool source, bool workspace,
    bool topology, bool master, bool witness, bool owner, bool degree,
    bool shadow, bool candidate, bool relation, bool controls, bool work,
    bool selected) {
    if (!parent) return "INTEGRATION_PARENT_REJECTED";
    if (!source) return "INTEGRATION_SOURCE_REJECTED";
    if (!workspace) return "INTEGRATION_WORKSPACE_REJECTED";
    if (!topology) return "INTEGRATION_TOPOLOGY_REJECTED";
    if (!master) return "INTEGRATION_MASTER_REJECTED";
    if (!witness) return "INTEGRATION_WITNESS_REJECTED";
    if (!owner) return "INTEGRATION_OWNER_REJECTED";
    if (!degree) return "INTEGRATION_DEGREE_CONTROLS_REJECTED";
    if (!shadow) return "INTEGRATION_SHADOW_REJECTED";
    if (!candidate) return "INTEGRATION_CANDIDATE_REJECTED";
    if (!relation) return "INTEGRATION_RELATION_REJECTED";
    if (!controls) return "INTEGRATION_CONTROLS_REJECTED";
    if (!work) return "INTEGRATION_WORK_REJECTED";
    if (selected) return "TOPOLOGY_OWNED_ROW_LOCAL_AUDIT_CANDIDATE";
    return "INTEGRATION_REFERENCE_RETAINED";
}

} // namespace

SplitBoundaryReport run_al_certificate_integration_controls() {
    const bool identity = sha256_hex(B4E2D7R19R61_IDENTITY_PROJECTION)
        == B4E2D7R19R61_IDENTITY_SHA256;
    ALR60ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_certificate_validation_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "6c6c62b6efb196d97dc4210ad7bcb56732f0c2d107a2df6aa10e9de84005c7f7"
        && parent.json.find(
            "d93dbdc95eb66cf45f074541ae44b621755f45f011d2f1589af834a45238aec9")
            != std::string::npos
        && parent.json.find("ROW_LOCAL_CERTIFICATE_VALIDATION_CANDIDATE")
            != std::string::npos;

    ALR59ReplayCapture& r59_capture = capture.r59;
    ALR58ReplayCapture& r58_capture = r59_capture.r58;
    ALR57Result& r58 = r58_capture.result;
    ALR57ReplayCapture& r57_capture = r58_capture.r57;
    ALR56ReplayCapture& r56_capture = r57_capture.r56;
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
    const std::string witness_before = al_binary64_vec3_root(r58.witness);
    const std::string upper_before = al_r10_dual_root(r58.terminal.upper);
    const bool source_exact = parent_exact && r30.captured
        && r58.state_root
            == "a4343378e4055fdc06ccb134dd44f3322473c93138960ff31965f27e8fd22af2"
        && witness_before
            == "040cc9f0dc57f543c8c3f9e1324b2e68dcc827e120782f9813a7aa0652d884bd"
        && upper_before
            == "9e8ad22488a6a49f66f7d2734c9573692a807cbfdc6c3d09874b24a590151ff8"
        && r51.union_root
            == "38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28"
        && r59_capture.scan.histogram_root
            == "3d4a798f7d9ea0fc71816091e786778f2368d8c0ff944b45200b004fa09dabdd"
        && r59_capture.scan.comparison_root
            == "5253b1c5a41fbb798cbe25df4a9c16afe663e4167b332aa9e0114c99b6de82de"
        && capture.validation.full_root
            == "30ed1a37eb96f0967f10a36c9f014459ab4ffe4c5b26609f74a653a1798f3a50"
        && capture.validation.comparison_root
            == "f0f18b90c88931ec6dc417ccf257109bfb094e541b17b0f69c06154dfab52ff8";

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
    const bool workspace_exact = source_exact && index.passed && binding.passed
        && superset.passed && moved.passed;
    const bool topology_exact = workspace_exact
        && al_r42_superset_root(superset) == superset_before
        && moved_filtered.pairs == moved.neighborhood.pairs;
    const bool master_exact = topology_exact && r51.rows.size() == 494U
        && r51.cache.size() == 494U
        && al_r51_cache_root(r51.cache) == r51.extended_cache_root;
    const bool witness_exact = master_exact
        && r58.witness.size() == moved.neighborhood.fluid.size()
        && al_r48_vec_finite(r58.witness);

    ALR61Audit audit;
    ALR61DenseControls controls;
    if (witness_exact) {
        audit = al_r61_topology_owned_audit(moved, r58.witness);
        controls = al_r61_dense_controls(moved.neighborhood);
    }
    if (moved.passed) release_al_normalized_workspace(moved, &workspace_work);

    const bool degree_exact = audit.minimum_degree == 44U
        && audit.maximum_degree == 113U && audit.distinct_degrees == 53U
        && audit.rows_below_maximum == 5992U
        && audit.histogram_root == r59_capture.scan.histogram_root;
    const bool shadow_exact = audit.shadow_root == upper_before
        && audit.shadow_active_root
            == "cda544504d01d5a92254b6c9cd657151a8e3cb5f4f7a3cc09ad9fcb38fb6f8fc"
        && audit.shadow_positive == 234U && audit.raw_positive == 0U
        && binary64_bits(audit.shadow_maximum)
            == binary64_bits(1.3678206846699582e-24)
        && audit.shadow_maximum_row == 2522U;
    const bool candidate_exact = audit.candidate_active_root
            == "82086595b4ed35abd2bf7f7cf6107359eaa9fe92be80df7aad1174dbf043bdb3"
        && audit.candidate_positive == 0U
        && binary64_bits(audit.candidate_maximum)
            == binary64_bits(-1.9354174334860891e-23)
        && audit.candidate_maximum_row == 4930U
        && audit.comparison_root == r59_capture.scan.comparison_root;
    const bool controls_exact = controls.degree_controls
        && controls.dual_path_valid && controls.offset_last_rejected
        && controls.cases == 6U;
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
        && audit.pair_passes == 1U && audit.row_scans == 1U
        && controls.cases == 6U;
    const bool rollback =
        position_before == al_binary64_vec3_root(r30.position)
        && trial_before == al_binary64_vec3_root(r43.projected_trial)
        && owner_before == al_r29_mask_root(r43.owners)
        && superset_before == al_r42_superset_root(r43.r42.superset)
        && anchor_before == al_binary64_vec3_root(r50.witness)
        && witness_before == al_binary64_vec3_root(r58.witness)
        && upper_before == al_r10_dual_root(r58.terminal.upper);
    const bool selected = audit.owner_exact && degree_exact && shadow_exact
        && candidate_exact && audit.relation_exact && controls_exact;

    const std::array<const char*, 15U> expected_routes{{
        "INTEGRATION_PARENT_REJECTED", "INTEGRATION_SOURCE_REJECTED",
        "INTEGRATION_WORKSPACE_REJECTED", "INTEGRATION_TOPOLOGY_REJECTED",
        "INTEGRATION_MASTER_REJECTED", "INTEGRATION_WITNESS_REJECTED",
        "INTEGRATION_OWNER_REJECTED", "INTEGRATION_DEGREE_CONTROLS_REJECTED",
        "INTEGRATION_SHADOW_REJECTED", "INTEGRATION_CANDIDATE_REJECTED",
        "INTEGRATION_RELATION_REJECTED", "INTEGRATION_CONTROLS_REJECTED",
        "INTEGRATION_WORK_REJECTED",
        "TOPOLOGY_OWNED_ROW_LOCAL_AUDIT_CANDIDATE",
        "INTEGRATION_REFERENCE_RETAINED"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t route_index = 0U;
         route_index < expected_routes.size(); ++route_index) {
        std::array<bool, 13U> gates{};
        gates.fill(true);
        if (route_index < gates.size()) gates[route_index] = false;
        const std::string observed = al_r61_route(
            gates[0], gates[1], gates[2], gates[3], gates[4], gates[5],
            gates[6], gates[7], gates[8], gates[9], gates[10], gates[11],
            gates[12], route_index == 13U);
        routes_exact = routes_exact && observed == expected_routes[route_index];
        route_material << expected_routes[route_index] << ':' << observed << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r61_route(parent_exact, source_exact,
        workspace_exact, topology_exact, master_exact, witness_exact,
        audit.owner_exact, degree_exact, shadow_exact, candidate_exact,
        audit.relation_exact, controls_exact, work_exact && rollback, selected);
    const bool passed = identity && parent_exact && source_exact
        && workspace_exact && topology_exact && master_exact && witness_exact
        && audit.owner_exact && degree_exact && shadow_exact && candidate_exact
        && audit.relation_exact && controls_exact && work_exact && rollback
        && routes_exact;

    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R61_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << witness_before << ':' << audit.image_root << ':'
             << audit.raw_root << ':' << audit.shadow_root << ':'
             << audit.candidate_root << '|'
             << audit.raw_positive << ':' << audit.shadow_positive << ':'
             << audit.candidate_positive << ':'
             << audit.relation_failures << ':'
             << audit.active_subset_failures << '|'
             << binary64_bits(audit.shadow_maximum) << ':'
             << audit.shadow_maximum_row << ':'
             << binary64_bits(audit.candidate_maximum) << ':'
             << audit.candidate_maximum_row << '|'
             << audit.shadow_active_root << ':'
             << audit.candidate_active_root << ':' << audit.histogram_root
             << ':' << audit.comparison_root << ':' << controls.root << '|'
             << audit.pair_passes << ':' << audit.row_scans << ':'
             << controls.cases << ':' << selected << ':' << work_exact << ':'
             << rollback << ':' << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r61_certificate_integration.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R61_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source\":{\"state_root\":\"" << r58.state_root
           << "\",\"witness_root\":\"" << witness_before << "\"}"
           << ",\"owner\":{\"topology_exact\":"
           << (audit.topology_exact ? "true" : "false")
           << ",\"image_root\":\"" << audit.image_root
           << "\",\"raw_root\":\"" << audit.raw_root << "\"}"
           << ",\"degree\":{\"minimum\":" << audit.minimum_degree
           << ",\"maximum\":" << audit.maximum_degree
           << ",\"distinct\":" << audit.distinct_degrees
           << ",\"rows_below_maximum\":" << audit.rows_below_maximum
           << ",\"histogram_root\":\"" << audit.histogram_root << "\"}"
           << ",\"shadow\":{\"positive\":" << audit.shadow_positive
           << ",\"maximum_upper\":" << audit.shadow_maximum
           << ",\"maximum_row\":" << audit.shadow_maximum_row
           << ",\"upper_root\":\"" << audit.shadow_root
           << "\",\"active_root\":\"" << audit.shadow_active_root << "\"}"
           << ",\"candidate\":{\"positive\":"
           << audit.candidate_positive << ",\"maximum_upper\":"
           << audit.candidate_maximum << ",\"maximum_row\":"
           << audit.candidate_maximum_row << ",\"upper_root\":\""
           << audit.candidate_root << "\",\"active_root\":\""
           << audit.candidate_active_root << "\"}"
           << ",\"relations\":{\"upper_failures\":"
           << audit.relation_failures << ",\"active_subset_failures\":"
           << audit.active_subset_failures << ",\"exact\":"
           << (audit.relation_exact ? "true" : "false")
           << ",\"comparison_root\":\"" << audit.comparison_root << "\"}"
           << ",\"controls\":{\"degree\":"
           << (controls.degree_controls ? "true" : "false")
           << ",\"dual_path\":"
           << (controls.dual_path_valid ? "true" : "false")
           << ",\"offset_last_rejected\":"
           << (controls.offset_last_rejected ? "true" : "false")
           << ",\"cases\":" << controls.cases << ",\"root\":\""
           << controls.root << "\"}"
           << ",\"work\":{\"directed_jvp\":" << audit.pair_passes
           << ",\"owner_row_scans\":" << audit.row_scans
           << ",\"dense_controls\":" << controls.cases
           << ",\"workspace_builds\":" << workspace_work.workspace_builds
           << ",\"workspace_releases\":"
           << workspace_work.workspace_releases
           << ",\"exact\":" << (work_exact ? "true" : "false") << "}"
           << ",\"route_cases\":" << expected_routes.size()
           << ",\"route_root\":\"" << route_root
           << "\",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback ? "true" : "false")
           << ",\"route\":\"" << route
           << "\",\"legacy_audit_replaced\":false"
              ",\"witness_applied\":false"
              ",\"restoration_promoted\":false"
              ",\"restoration_exit_authorized\":false"
              ",\"binary128_runtime\":false"
              ",\"gamma_definition_unchanged\":true"
              ",\"operator_arithmetic_unchanged\":true"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    return {passed, report.str()};
}
