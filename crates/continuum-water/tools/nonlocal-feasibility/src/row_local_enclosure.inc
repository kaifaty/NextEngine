namespace {

constexpr const char* B4E2D7R19R59_IDENTITY_SHA256 =
    "bab527b0018f4265c8fedce046a07615dbd4fe95687eb6883487952bea14fb9e";
constexpr const char* B4E2D7R19R59_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r59-row-local-enclosure|v2|parent=ccd2461e:8078c06230d6436253df966eb617bee202237143a1cf5def0e1dac49028ec257:f7070521395450cc3b54bfe85c9c4c2648abfa87ff37cfaa419da317628d74c3:FIXED_POINT_CONTRACTION_CANDIDATE|source=r58-state-a4343378e4055fdc06ccb134dd44f3322473c93138960ff31965f27e8fd22af2;cycle64-record-86ad77c25f07d0337b0c2208ac208271c4d163d657c15711d5e78cfcd6c38e95;witness-040cc9f0dc57f543c8c3f9e1324b2e68dcc827e120782f9813a7aa0652d884bd;correction-7f5a7c242443b35c54616e4604531bca77c0dd3e2e7e21e850b105559a30fb1b;boxdual-da085eec472077b9ce00e8ce4881cfb3e9de52910cd5c8a60373fbf4fe3bde1b;comparison-a16fc80e265f227eb101a175b7427dce7f9bc3f2db62ae6ae6dffd6ca13247c2;active234;raw0;maximum-upper1.3678206846699582e-24;full128-positive0;negative6000;unresolved0;maximum-raw-5.1987499104498105e-22|scan=exact-r58-selected-cycle64;rows6000;degree=flat-offset-difference-including-radius-skips|current=gamma(16*maximum-degree+66)*(abs-c+directed-absolute-sum);raw=c+directed-value;upper=raw+bound|local=gamma(16*row-degree+66)*(abs-c+directed-absolute-sum);local-upper=raw+local-bound;same-gamma-function;same-inputs;same-binary64-order|gates=current-recomputed-bits-equal-captured-all-rows-and-maximum;degree<=maximum;local-upper<=current-upper-all-rows;finite;active=current-upper>0;local-active=local-upper>0|observations=degree-min-max-distinct;rows-below-max;current-positive;local-positive;current-positive-at-max-degree;local-positive-at-max-degree;strict-improved-rows;closed-current-positive;current-and-local-maxima-row-degree-bits;degree-histogram-root;row-comparison-root|selection=local-positive0-row-local-certificate;else-strict-and-local-positive-at-max-degree>0-eft-accumulation-research;else-strict-partial-staged-enclosure;else-no-improvement|controls=r58-parent-bytes;source;workspace;topology;master;witness;audit;degree;global-reproduction;local-monotonicity;comparison;work;rollback;route-precedence|routes=row-local-parent-rejected;row-local-source-rejected;row-local-workspace-rejected;row-local-topology-rejected;row-local-master-rejected;row-local-witness-rejected;row-local-audit-rejected;row-local-degree-rejected;row-local-global-reproduction-rejected;row-local-monotonicity-rejected;row-local-comparison-rejected;row-local-work-rejected;row-local-enclosure-certificate-candidate;eft-accumulation-research-required;staged-enclosure-research-required;row-local-enclosure-no-improvement|precedence=parent,source,workspace,topology,master,witness,audit,degree,global,local,comparison,work,certified,max-degree,partial,no-improvement|work=parent-r58-replays1;moved-workspace1;workspace-release1;row-scans1;new-pairpasses0;new-quad-row-passes0;new-density-sweeps0;new-box-blocks0;new-jvp0;new-vjp0;new-basis0;new-gram0;new-projection0;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53-r54-r55-r56-r57-r58=unchanged;certificate-candidate=private-only;binary128=retained-offline-no-new-pass;gamma-definition=unchanged;operator-arithmetic=unchanged;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;capacity-change=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-row-local-enclosure-classification-only)IDENTITY";

struct ALR59Scan {
    bool audit_exact = false;
    bool degree_exact = false;
    bool global_exact = false;
    bool local_monotone = false;
    bool comparison_exact = false;
    std::size_t row_scans = 0U;
    std::size_t minimum_degree = std::numeric_limits<std::size_t>::max();
    std::size_t maximum_degree = 0U;
    std::size_t distinct_degrees = 0U;
    std::size_t rows_below_maximum = 0U;
    std::size_t current_positive = 0U;
    std::size_t local_positive = 0U;
    std::size_t current_positive_at_maximum_degree = 0U;
    std::size_t local_positive_at_maximum_degree = 0U;
    std::size_t strict_improved_rows = 0U;
    std::size_t closed_current_positive = 0U;
    double current_maximum = -std::numeric_limits<double>::infinity();
    std::size_t current_maximum_row = 0U;
    std::size_t current_maximum_degree = 0U;
    double local_maximum = -std::numeric_limits<double>::infinity();
    std::size_t local_maximum_row = 0U;
    std::size_t local_maximum_degree = 0U;
    std::string current_active_root;
    std::string local_active_root;
    std::string histogram_root;
    std::string comparison_root;
};

ALR59Scan al_r59_scan(const ALNormalizedWorkspace& workspace,
    const ALR50Audit& audit) {
    ALR59Scan result;
    const std::size_t count = workspace.support.constraint.size();
    result.audit_exact = workspace.passed && audit.passed
        && count == 6000U && audit.image.passed
        && audit.image.value.size() == count
        && audit.image.absolute_sum.size() == count
        && audit.upper.size() == count
        && workspace.neighborhood.flat_offsets.size() == count + 1U;
    if (!result.audit_exact) return result;

    const std::size_t global_degree = workspace.neighborhood.maximum_degree;
    std::vector<std::size_t> histogram(global_degree + 1U, 0U);
    std::vector<std::uint8_t> current_mask(count, 0U);
    std::vector<std::uint8_t> local_mask(count, 0U);
    std::ostringstream rows;
    bool degree_exact = true;
    bool global_exact = true;
    bool local_monotone = true;
    bool finite = true;
    const double global_gamma = gamma_factor(16U * global_degree + 66U);
    for (std::size_t row = 0U; row < count; ++row) {
        const std::size_t begin = workspace.neighborhood.flat_offsets[row];
        const std::size_t end = workspace.neighborhood.flat_offsets[row + 1U];
        degree_exact = degree_exact && begin <= end;
        const std::size_t degree = end >= begin ? end - begin : 0U;
        degree_exact = degree_exact && degree <= global_degree;
        if (degree <= global_degree) ++histogram[degree];
        result.minimum_degree = std::min(result.minimum_degree, degree);
        result.maximum_degree = std::max(result.maximum_degree, degree);

        const double scale = std::abs(workspace.support.constraint[row])
            + audit.image.absolute_sum[row];
        const double raw = workspace.support.constraint[row]
            + audit.image.value[row];
        const double current_bound = global_gamma * scale;
        const double current_upper = raw + current_bound;
        const double local_gamma = gamma_factor(16U * degree + 66U);
        const double local_bound = local_gamma * scale;
        const double local_upper = raw + local_bound;

        global_exact = global_exact
            && binary64_bits(current_upper)
                == binary64_bits(audit.upper[row]);
        local_monotone = local_monotone && local_upper <= current_upper;
        finite = finite && std::isfinite(scale) && std::isfinite(raw)
            && std::isfinite(current_bound) && std::isfinite(current_upper)
            && std::isfinite(local_bound) && std::isfinite(local_upper);
        if (local_upper < current_upper) ++result.strict_improved_rows;
        if (current_upper > 0.0) {
            ++result.current_positive;
            current_mask[row] = 1U;
            if (degree == global_degree)
                ++result.current_positive_at_maximum_degree;
            if (local_upper <= 0.0) ++result.closed_current_positive;
        }
        if (local_upper > 0.0) {
            ++result.local_positive;
            local_mask[row] = 1U;
            if (degree == global_degree)
                ++result.local_positive_at_maximum_degree;
        }
        if (current_upper > result.current_maximum) {
            result.current_maximum = current_upper;
            result.current_maximum_row = row;
            result.current_maximum_degree = degree;
        }
        if (local_upper > result.local_maximum) {
            result.local_maximum = local_upper;
            result.local_maximum_row = row;
            result.local_maximum_degree = degree;
        }
        rows << row << ':' << degree << ':' << binary64_bits(raw) << ':'
             << binary64_bits(current_bound) << ':'
             << binary64_bits(current_upper) << ':'
             << binary64_bits(local_bound) << ':'
             << binary64_bits(local_upper) << ';';
    }
    ++result.row_scans;

    std::ostringstream histogram_material;
    for (std::size_t degree = 0U; degree < histogram.size(); ++degree) {
        if (histogram[degree] > 0U) {
            ++result.distinct_degrees;
            histogram_material << degree << ':' << histogram[degree] << ';';
        }
    }
    result.rows_below_maximum = count - histogram[global_degree];
    result.current_active_root = al_r29_mask_root(current_mask);
    result.local_active_root = al_r29_mask_root(local_mask);
    result.histogram_root = sha256_hex(histogram_material.str());
    result.comparison_root = sha256_hex(rows.str());
    result.degree_exact = degree_exact && result.maximum_degree == global_degree
        && std::accumulate(histogram.begin(), histogram.end(), 0U) == count;
    result.global_exact = global_exact && finite
        && result.current_positive == audit.active.size()
        && result.current_active_root == audit.active_root
        && binary64_bits(result.current_maximum)
            == binary64_bits(audit.primal.maximum_upper);
    result.local_monotone = local_monotone && finite;
    result.comparison_exact = result.degree_exact && result.global_exact
        && result.local_monotone
        && result.local_positive <= result.current_positive
        && result.local_positive + result.closed_current_positive
            == result.current_positive
        && result.strict_improved_rows <= count
        && result.minimum_degree <= result.maximum_degree;
    return result;
}

std::string al_r59_route(bool parent, bool source, bool workspace,
    bool topology, bool master, bool witness, bool audit, bool degree,
    bool global, bool local, bool comparison, bool work, bool certified,
    bool eft, bool partial) {
    if (!parent) return "ROW_LOCAL_PARENT_REJECTED";
    if (!source) return "ROW_LOCAL_SOURCE_REJECTED";
    if (!workspace) return "ROW_LOCAL_WORKSPACE_REJECTED";
    if (!topology) return "ROW_LOCAL_TOPOLOGY_REJECTED";
    if (!master) return "ROW_LOCAL_MASTER_REJECTED";
    if (!witness) return "ROW_LOCAL_WITNESS_REJECTED";
    if (!audit) return "ROW_LOCAL_AUDIT_REJECTED";
    if (!degree) return "ROW_LOCAL_DEGREE_REJECTED";
    if (!global) return "ROW_LOCAL_GLOBAL_REPRODUCTION_REJECTED";
    if (!local) return "ROW_LOCAL_MONOTONICITY_REJECTED";
    if (!comparison) return "ROW_LOCAL_COMPARISON_REJECTED";
    if (!work) return "ROW_LOCAL_WORK_REJECTED";
    if (certified) return "ROW_LOCAL_ENCLOSURE_CERTIFICATE_CANDIDATE";
    if (eft) return "EFT_ACCUMULATION_RESEARCH_REQUIRED";
    if (partial) return "STAGED_ENCLOSURE_RESEARCH_REQUIRED";
    return "ROW_LOCAL_ENCLOSURE_NO_IMPROVEMENT";
}

} // namespace

SplitBoundaryReport run_al_row_local_enclosure_controls() {
    const bool identity = sha256_hex(B4E2D7R19R59_IDENTITY_PROJECTION)
        == B4E2D7R19R59_IDENTITY_SHA256;
    ALR58ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_directed_enclosure_fixed_point_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "8078c06230d6436253df966eb617bee202237143a1cf5def0e1dac49028ec257"
        && parent.json.find(
            "f7070521395450cc3b54bfe85c9c4c2648abfa87ff37cfaa419da317628d74c3")
            != std::string::npos
        && parent.json.find("FIXED_POINT_CONTRACTION_CANDIDATE")
            != std::string::npos;

    ALR57Result& r58 = capture.result;
    ALR57ReplayCapture& r57_capture = capture.r57;
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
    const std::string master_root_before = r51.union_root;

    const std::array<bool, 15U> source_checks{{
        parent_exact, r30.captured,
        r58.state_root
            == "a4343378e4055fdc06ccb134dd44f3322473c93138960ff31965f27e8fd22af2",
        r58.records.size() == 4U && r58.selected_index == 3U,
        r58.records.size() == 4U && r58.records.back().root
            == "86ad77c25f07d0337b0c2208ac208271c4d163d657c15711d5e78cfcd6c38e95",
        witness_before
            == "040cc9f0dc57f543c8c3f9e1324b2e68dcc827e120782f9813a7aa0652d884bd",
        r58.records.size() == 4U && r58.records.back().correction_root
            == "7f5a7c242443b35c54616e4604531bca77c0dd3e2e7e21e850b105559a30fb1b",
        r58.records.size() == 4U && r58.records.back().box_dual_root
            == "da085eec472077b9ce00e8ce4881cfb3e9de52910cd5c8a60373fbf4fe3bde1b",
        r58.terminal.passed && r58.terminal.active.size() == 234U,
        r58.terminal.primal.raw_active == 0U
            && binary64_bits(r58.terminal.primal.maximum_upper)
                == binary64_bits(1.3678206846699582e-24),
        capture.decomposition.comparison_root
            == "a16fc80e265f227eb101a175b7427dce7f9bc3f2db62ae6ae6dffd6ca13247c2",
        capture.decomposition.full128_resolved_positive == 0U,
        capture.decomposition.full128_resolved_negative == 6000U,
        capture.decomposition.full128_unresolved == 0U,
        master_root_before
            == "38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28"}};
    const bool source_exact = std::all_of(
        source_checks.begin(), source_checks.end(), [](bool value) {
            return value;
        });
    std::ostringstream source_check_material;
    for (bool value : source_checks) source_check_material << value;
    const std::string source_check_mask = source_check_material.str();

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
        && al_r48_vec_finite(r58.witness)
        && witness_before == r58.records.back().witness_root;

    ALR59Scan scan;
    if (witness_exact) scan = al_r59_scan(moved, r58.terminal);
    if (moved.passed) release_al_normalized_workspace(moved, &workspace_work);

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
        && scan.row_scans == 1U;
    const bool rollback =
        position_before == al_binary64_vec3_root(r30.position)
        && trial_before == al_binary64_vec3_root(r43.projected_trial)
        && owner_before == al_r29_mask_root(r43.owners)
        && superset_before == al_r42_superset_root(r43.r42.superset)
        && anchor_before == al_binary64_vec3_root(r50.witness)
        && witness_before == al_binary64_vec3_root(r58.witness)
        && upper_before == al_r10_dual_root(r58.terminal.upper);

    const bool certified = scan.comparison_exact
        && scan.local_positive == 0U;
    const bool eft = scan.comparison_exact && !certified
        && scan.strict_improved_rows > 0U
        && scan.local_positive_at_maximum_degree > 0U;
    const bool partial = scan.comparison_exact && !certified && !eft
        && scan.strict_improved_rows > 0U;

    const std::array<const char*, 16U> expected_routes{{
        "ROW_LOCAL_PARENT_REJECTED", "ROW_LOCAL_SOURCE_REJECTED",
        "ROW_LOCAL_WORKSPACE_REJECTED", "ROW_LOCAL_TOPOLOGY_REJECTED",
        "ROW_LOCAL_MASTER_REJECTED", "ROW_LOCAL_WITNESS_REJECTED",
        "ROW_LOCAL_AUDIT_REJECTED", "ROW_LOCAL_DEGREE_REJECTED",
        "ROW_LOCAL_GLOBAL_REPRODUCTION_REJECTED",
        "ROW_LOCAL_MONOTONICITY_REJECTED",
        "ROW_LOCAL_COMPARISON_REJECTED", "ROW_LOCAL_WORK_REJECTED",
        "ROW_LOCAL_ENCLOSURE_CERTIFICATE_CANDIDATE",
        "EFT_ACCUMULATION_RESEARCH_REQUIRED",
        "STAGED_ENCLOSURE_RESEARCH_REQUIRED",
        "ROW_LOCAL_ENCLOSURE_NO_IMPROVEMENT"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t route_index = 0U;
         route_index < expected_routes.size(); ++route_index) {
        std::array<bool, 12U> gates{};
        gates.fill(true);
        if (route_index < gates.size()) gates[route_index] = false;
        const std::string observed = al_r59_route(
            gates[0], gates[1], gates[2], gates[3], gates[4], gates[5],
            gates[6], gates[7], gates[8], gates[9], gates[10], gates[11],
            route_index == 12U, route_index == 13U, route_index == 14U);
        routes_exact = routes_exact && observed == expected_routes[route_index];
        route_material << expected_routes[route_index] << ':' << observed << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r59_route(parent_exact, source_exact,
        workspace_exact, topology_exact, master_exact, witness_exact,
        scan.audit_exact, scan.degree_exact, scan.global_exact,
        scan.local_monotone, scan.comparison_exact, work_exact && rollback,
        certified, eft, partial);
    const bool passed = identity && parent_exact && source_exact
        && workspace_exact && topology_exact && master_exact && witness_exact
        && scan.audit_exact && scan.degree_exact && scan.global_exact
        && scan.local_monotone && scan.comparison_exact && work_exact
        && rollback && routes_exact;

    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R59_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << witness_before << ':' << upper_before << '|'
             << scan.minimum_degree << ':' << scan.maximum_degree << ':'
             << scan.distinct_degrees << ':' << scan.rows_below_maximum << '|'
             << scan.current_positive << ':' << scan.local_positive << ':'
             << scan.current_positive_at_maximum_degree << ':'
             << scan.local_positive_at_maximum_degree << ':'
             << scan.strict_improved_rows << ':'
             << scan.closed_current_positive << '|'
             << binary64_bits(scan.current_maximum) << ':'
             << scan.current_maximum_row << ':'
             << scan.current_maximum_degree << ':'
             << binary64_bits(scan.local_maximum) << ':'
             << scan.local_maximum_row << ':'
             << scan.local_maximum_degree << '|'
             << scan.current_active_root << ':' << scan.local_active_root << ':'
             << scan.histogram_root << ':' << scan.comparison_root << '|'
             << scan.row_scans << ':' << certified << ':' << eft << ':'
             << partial << ':' << work_exact << ':' << rollback << ':'
             << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r59_row_local_enclosure.v2\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R59_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source\":{\"state_root\":\"" << r58.state_root
           << "\",\"witness_root\":\"" << witness_before
           << "\",\"upper_root\":\"" << upper_before
           << "\",\"comparison_root\":\""
           << capture.decomposition.comparison_root
           << "\",\"master_root_before\":\"" << master_root_before
           << "\",\"master_root_after\":\"" << r51.union_root
           << "\",\"master_root_before_size\":" << master_root_before.size()
           << ",\"master_root_after_size\":" << r51.union_root.size()
           << ",\"check_mask\":\"" << source_check_mask << "\"}"
           << ",\"degree\":{\"minimum\":" << scan.minimum_degree
           << ",\"maximum\":" << scan.maximum_degree
           << ",\"distinct\":" << scan.distinct_degrees
           << ",\"rows_below_maximum\":" << scan.rows_below_maximum
           << ",\"histogram_root\":\"" << scan.histogram_root << "\"}"
           << ",\"rows\":{\"current_positive\":"
           << scan.current_positive << ",\"local_positive\":"
           << scan.local_positive
           << ",\"current_positive_at_maximum_degree\":"
           << scan.current_positive_at_maximum_degree
           << ",\"local_positive_at_maximum_degree\":"
           << scan.local_positive_at_maximum_degree
           << ",\"strict_improved\":" << scan.strict_improved_rows
           << ",\"closed_current_positive\":"
           << scan.closed_current_positive << "}"
           << ",\"maxima\":{\"current_upper\":"
           << scan.current_maximum << ",\"current_row\":"
           << scan.current_maximum_row << ",\"current_degree\":"
           << scan.current_maximum_degree << ",\"local_upper\":"
           << scan.local_maximum << ",\"local_row\":"
           << scan.local_maximum_row << ",\"local_degree\":"
           << scan.local_maximum_degree << "}"
           << ",\"roots\":{\"current_active\":\""
           << scan.current_active_root << "\",\"local_active\":\""
           << scan.local_active_root << "\",\"comparison\":\""
           << scan.comparison_root << "\"}"
           << ",\"gates\":{\"audit_exact\":"
           << (scan.audit_exact ? "true" : "false")
           << ",\"degree_exact\":"
           << (scan.degree_exact ? "true" : "false")
           << ",\"global_exact\":"
           << (scan.global_exact ? "true" : "false")
           << ",\"local_monotone\":"
           << (scan.local_monotone ? "true" : "false")
           << ",\"comparison_exact\":"
           << (scan.comparison_exact ? "true" : "false") << "}"
           << ",\"work\":{\"row_scans\":" << scan.row_scans
           << ",\"new_pair_passes\":0,\"new_quad_row_passes\":0"
           << ",\"workspace_builds\":" << workspace_work.workspace_builds
           << ",\"workspace_releases\":"
           << workspace_work.workspace_releases
           << ",\"exact\":" << (work_exact ? "true" : "false") << "}"
           << ",\"route_cases\":" << expected_routes.size()
           << ",\"route_root\":\"" << route_root
           << "\",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback ? "true" : "false")
           << ",\"route\":\"" << route
           << "\",\"row_local_certificate_private_only\":true"
              ",\"gamma_definition_unchanged\":true"
              ",\"operator_arithmetic_unchanged\":true"
              ",\"new_binary128_passes\":0"
              ",\"third_fixed_point_outer_executed\":false"
              ",\"normal_witness_applied\":false"
              ",\"r43_restoration_committed\":false"
              ",\"restoration_exit_authorized\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    return {passed, report.str()};
}
