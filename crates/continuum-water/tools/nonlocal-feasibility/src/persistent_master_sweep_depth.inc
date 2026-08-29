namespace {

constexpr const char* B4E2D7R19R52_IDENTITY_SHA256 =
    "fde05c9b766f5bf81283adcfa9374cc60b490bd75c850e1fba599de70707aafd";
constexpr const char* B4E2D7R19R52_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r52-persistent-master-sweep-depth|v1|parent=8bf5863f:c0fc53e3c1637a84c0a9cc0494a8b9fc7bc6e60b627afe032337a37c14cbf960:65df360418fba870089415b922b50e96c3bba55e904b2f9eec4fadd1e096361a:PERSISTENT_MASTER_CLOSURE_CANDIDATE|source=r50-terminal-witness-e2ccfb4281b2b598b0a9864a8c9bb92f5c2512e018501147ca82b39b14b67592;r51-witness-72c570b9bb981625ee2036b5c88ff19e5a88bca662efb3474bd62ce1130a6e34;union-root-38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28;cache-root-f4e9368971d4b6feeb13874d0279dc73fd54ebbb523c75c5c2dd303acfc58cd5;gram-root-7a0358af3fda7c90c89a5cfb9abe125fadf9142fb5ce0215e94ee2e70a94dc48;master494;r51-psi1.3460782774625938e-25;r51-h5.1885995749577626e-13;r51-maximum-upper8.9325740894562227e-14|problem=unchanged-r51-fixed-master-same-r50-terminal-anchor-upper-and-ball-box;independent-zero-dual-depth-probes;no-new-row|solver=stable-master-order;exact-sweeps16,32,64;independent-zero-dual-each;checkpoints1,2,4,8-plus-terminal-state;no-tolerance-stop;dense-gram-reused|globalization=each-depth-target=r50-terminal+correction;unchanged-r48-ball-box-projection;fresh-r48-all-row-directed-jvp;evaluate-all-three|selection=smallest-sweep-certified-first;else-smallest-sweep-strict-simultaneous-r51-psi-h-maximum-upper-dominance;else-r51-reference-retained|diagnostic=predicted-maximum-at8,16,32,64;strict-monotone-depth-contraction;cg-polish-required-only-if-no-selected-candidate-and-contraction-valid|controls=r51-parent-bytes;source;workspace;master-union;basis-reuse;gram;depth-sweeps;projection;directed-audit;selection;work;rollback;route-precedence|routes=sweep-parent-rejected;sweep-source-rejected;sweep-workspace-rejected;sweep-master-rejected;sweep-basis-rejected;sweep-gram-rejected;sweep-depth-rejected;sweep-projection-rejected;sweep-primal-audit-rejected;sweep-selection-rejected;sweep-work-rejected;restoration-next-trqp-compatible-candidate;persistent-master-sweep-depth-candidate;persistent-master-cg-polish-required;persistent-master-r51-reference-retained|precedence=parent,source,workspace,master,basis,gram,depth,projection,primal-audit,selection,work,certificate,sweep-candidate,cg-polish,reference|work=parent-r51-replays1;moved-workspace1;workspace-release1;captured-cache494-reused;new-row-vjp0;new-gram-jvp0;candidate-directed-jvp3;pairpasses3;independent-sweeps112;projection-calls3;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51=unchanged;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;capacity-change=none;penalty-change=none;support-expansion=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-fixed-master-sweep-depth-classification-only)IDENTITY";

constexpr std::array<std::size_t, 3U> AL_R52_DEPTHS{{16U, 32U, 64U}};
constexpr double AL_R52_R51_PSI = 1.3460782774625938e-25;
constexpr double AL_R52_R51_H = 5.1885995749577626e-13;
constexpr double AL_R52_R51_MAXIMUM_UPPER = 8.9325740894562227e-14;

struct ALR52Record {
    bool passed = false;
    bool strict = false;
    bool certified = false;
    std::size_t sweeps = 0U;
    std::size_t raw_outside_master = 0U;
    std::size_t directed_outside_master = 0U;
    ALR50Solve solve;
    ALR50Audit audit;
    std::vector<Vec3> witness;
    double witness_norm = 0.0;
    std::string witness_root;
    std::string root;
};

struct ALR52Result {
    bool master_exact = false;
    bool basis_exact = false;
    bool gram_exact = false;
    bool depth_exact = false;
    bool projection_exact = false;
    bool primal_exact = false;
    bool selection_exact = false;
    bool contraction = false;
    bool selected = false;
    bool certified = false;
    std::size_t selected_index = 0U;
    std::size_t selected_sweeps = 0U;
    std::size_t audit_jvp = 0U;
    std::size_t pair_passes = 0U;
    std::size_t projection_calls = 0U;
    std::size_t dense_sweeps = 0U;
    std::vector<ALR52Record> records;
    ALR50Audit terminal;
    std::vector<Vec3> witness;
};

ALR52Result al_r52_generate(const ALNormalizedWorkspace& workspace,
    const ALR50Generator& r50, const ALR51MasterResult& r51,
    const std::vector<Vec3>& lower, const std::vector<Vec3>& upper) {
    ALR52Result result;
    result.master_exact = workspace.passed && r50.terminal.passed
        && r51.master_rows == 494U && r51.rows.size() == 494U
        && r51.cache.size() == 494U
        && std::is_sorted(r51.rows.begin(), r51.rows.end())
        && al_r49_feasible(r50.witness, lower, upper);
    result.basis_exact = result.master_exact
        && al_r51_cache_root(r51.cache) == r51.extended_cache_root;
    if (!result.basis_exact) return result;
    ALR50Audit master = r50.terminal;
    master.active = r51.rows;
    master.active_root = r51.union_root;
    std::vector<std::uint8_t> master_mask(
        workspace.support.constraint.size(), 0U);
    for (std::size_t row : r51.rows) master_mask[row] = 1U;

    bool all_depth = true;
    bool all_gram = true;
    bool all_projection = true;
    bool all_primal = true;
    for (std::size_t depth : AL_R52_DEPTHS) {
        ALR52Record record;
        record.sweeps = depth;
        record.solve = al_r50_solve(r51.cache, master, depth);
        result.dense_sweeps += depth;
        all_depth = all_depth && record.solve.passed
            && record.solve.core.sweeps == depth;
        all_gram = all_gram && record.solve.gram_exact;
        if (!record.solve.passed) {
            result.records.push_back(std::move(record));
            continue;
        }
        std::vector<Vec3> target(r50.witness.size());
        for (std::size_t i = 0U; i < target.size(); ++i)
            target[i] = r50.witness[i] + record.solve.correction[i];
        const ALR48Projection projection = al_r48_project_ball_box(
            target, lower, upper, AL_R48_NORMAL_RADIUS);
        ++result.projection_calls;
        all_projection = all_projection && projection.passed;
        if (!projection.passed) {
            result.records.push_back(std::move(record));
            continue;
        }
        record.witness = projection.value;
        record.audit = al_r50_audit(workspace, record.witness);
        ++result.audit_jvp;
        all_primal = all_primal && record.audit.passed;
        record.certified = record.audit.passed && record.audit.active.empty();
        for (std::size_t row = 0U; row < master_mask.size(); ++row)
            if (master_mask[row] == 0U
                && workspace.support.constraint[row]
                    + record.audit.image.value[row] > 0.0)
                ++record.raw_outside_master;
        for (std::size_t row : record.audit.active)
            if (master_mask[row] == 0U) ++record.directed_outside_master;
        record.strict = record.audit.passed
            && record.audit.primal.psi < AL_R52_R51_PSI
            && record.audit.primal.h < AL_R52_R51_H
            && record.audit.primal.maximum_upper
                < AL_R52_R51_MAXIMUM_UPPER;
        record.witness_norm = al_r32_vec3_norm(record.witness);
        record.witness_root = al_binary64_vec3_root(record.witness);
        std::ostringstream material;
        material << std::setprecision(17) << depth << ':'
                 << record.solve.root << ':'
                 << record.solve.core.predicted_maximum << ':'
                 << record.audit.primal.psi << ':' << record.audit.primal.h
                 << ':' << record.audit.primal.maximum_upper << ':'
                 << record.audit.primal.raw_active << ':'
                 << record.audit.active.size() << ':'
                 << record.raw_outside_master << ':'
                 << record.directed_outside_master << ':'
                 << record.strict << ':'
                 << record.certified << ':' << record.witness_norm << ':'
                 << record.witness_root;
        record.root = sha256_hex(material.str());
        record.passed = record.solve.passed && projection.passed
            && record.audit.passed && std::isfinite(record.witness_norm);
        result.records.push_back(std::move(record));
    }
    result.depth_exact = all_depth && result.records.size() == AL_R52_DEPTHS.size()
        && result.dense_sweeps == 112U;
    result.gram_exact = all_gram;
    result.projection_exact = all_projection
        && result.projection_calls == AL_R52_DEPTHS.size();
    result.primal_exact = all_primal
        && result.audit_jvp == AL_R52_DEPTHS.size()
        && std::all_of(result.records.begin(), result.records.end(),
            [](const ALR52Record& record) { return record.passed; });
    if (!result.depth_exact || !result.projection_exact || !result.primal_exact)
        return result;

    double previous = r51.solve.core.predicted_maximum;
    result.contraction = std::isfinite(previous);
    for (const ALR52Record& record : result.records) {
        result.contraction = result.contraction
            && record.solve.core.predicted_maximum < previous;
        previous = record.solve.core.predicted_maximum;
    }
    for (std::size_t i = 0U; i < result.records.size(); ++i)
        if (result.records[i].certified) {
            result.selected = true;
            result.certified = true;
            result.selected_index = i;
            break;
        }
    if (!result.selected)
        for (std::size_t i = 0U; i < result.records.size(); ++i)
            if (result.records[i].strict) {
                result.selected = true;
                result.selected_index = i;
                break;
            }
    if (result.selected) {
        result.selected_sweeps = result.records[result.selected_index].sweeps;
        result.terminal = result.records[result.selected_index].audit;
        result.witness = result.records[result.selected_index].witness;
    } else {
        result.terminal = r51.terminal;
        result.witness = r51.witness;
    }
    result.selection_exact = result.terminal.passed
        && (!result.certified || result.terminal.active.empty());
    result.pair_passes = result.audit_jvp;
    return result;
}

bool al_r52_dense_controls(std::string* output_root) {
    const std::vector<double> gram{2.0, 1.0, 1.0, 2.0};
    const std::vector<double> upper{1.0, 1.0};
    const ALR50Core d16 = al_r50_hildreth_core(gram, upper, 16U);
    const ALR50Core d32 = al_r50_hildreth_core(gram, upper, 32U);
    const ALR50Core d64 = al_r50_hildreth_core(gram, upper, 64U);
    const bool passed = d16.passed && d32.passed && d64.passed
        && d16.sweeps == 16U && d32.sweeps == 32U && d64.sweeps == 64U
        && d32.predicted_maximum <= d16.predicted_maximum
        && d64.predicted_maximum <= d32.predicted_maximum;
    std::ostringstream material;
    material << passed << ':' << d16.predicted_maximum << ':'
             << d32.predicted_maximum << ':' << d64.predicted_maximum << ':'
             << al_r10_dual_root(d64.lambda);
    if (output_root != nullptr) *output_root = sha256_hex(material.str());
    return passed;
}

std::string al_r52_route(bool parent, bool source, bool workspace,
    bool master, bool basis, bool gram, bool depth, bool projection,
    bool primal, bool selection, bool work, bool certified,
    bool sweep_candidate, bool cg_polish) {
    if (!parent) return "SWEEP_PARENT_REJECTED";
    if (!source) return "SWEEP_SOURCE_REJECTED";
    if (!workspace) return "SWEEP_WORKSPACE_REJECTED";
    if (!master) return "SWEEP_MASTER_REJECTED";
    if (!basis) return "SWEEP_BASIS_REJECTED";
    if (!gram) return "SWEEP_GRAM_REJECTED";
    if (!depth) return "SWEEP_DEPTH_REJECTED";
    if (!projection) return "SWEEP_PROJECTION_REJECTED";
    if (!primal) return "SWEEP_PRIMAL_AUDIT_REJECTED";
    if (!selection) return "SWEEP_SELECTION_REJECTED";
    if (!work) return "SWEEP_WORK_REJECTED";
    if (certified) return "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE";
    if (sweep_candidate) return "PERSISTENT_MASTER_SWEEP_DEPTH_CANDIDATE";
    if (cg_polish) return "PERSISTENT_MASTER_CG_POLISH_REQUIRED";
    return "PERSISTENT_MASTER_R51_REFERENCE_RETAINED";
}

struct ALR52ReplayCapture {
    bool captured = false;
    ALR51ReplayCapture r51;
    ALR52Result depth;
};

} // namespace

SplitBoundaryReport run_al_persistent_master_sweep_depth_controls_impl(
    ALR52ReplayCapture* replay_capture) {
    const bool identity = sha256_hex(B4E2D7R19R52_IDENTITY_PROJECTION)
        == B4E2D7R19R52_IDENTITY_SHA256;
    ALR51ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_persistent_master_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "c0fc53e3c1637a84c0a9cc0494a8b9fc7bc6e60b627afe032337a37c14cbf960"
        && parent.json.find(
            "65df360418fba870089415b922b50e96c3bba55e904b2f9eec4fadd1e096361a")
            != std::string::npos
        && parent.json.find("PERSISTENT_MASTER_CLOSURE_CANDIDATE")
            != std::string::npos;
    ALR50Generator& r50 = capture.r50.generator;
    ALR51MasterResult& r51 = capture.master;
    ALR43ReplayCapture& r43 =
        capture.r50.r49.r48.r47.r46.r45.r44.r43;
    ALR40ReplayCapture& r40 = r43.r42.r40;
    ALR30RangeProjectionCapture& r30 =
        r40.r39.r38.r37.r36.r35.r34.r33.r32.r31.r30;
    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string trial_before = al_binary64_vec3_root(r43.projected_trial);
    const std::string owner_before = al_r29_mask_root(r43.owners);
    const std::string superset_before = al_r42_superset_root(r43.r42.superset);
    const std::string r50_witness_before = al_binary64_vec3_root(r50.witness);
    const std::string r51_witness_before = al_binary64_vec3_root(r51.witness);
    const bool source_exact = parent_exact && r30.captured
        && r50_witness_before
            == "e2ccfb4281b2b598b0a9864a8c9bb92f5c2512e018501147ca82b39b14b67592"
        && r51_witness_before
            == "72c570b9bb981625ee2036b5c88ff19e5a88bca662efb3474bd62ce1130a6e34"
        && r51.union_root
            == "38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28"
        && r51.extended_cache_root
            == "f4e9368971d4b6feeb13874d0279dc73fd54ebbb523c75c5c2dd303acfc58cd5"
        && r51.solve.gram_root
            == "7a0358af3fda7c90c89a5cfb9abe125fadf9142fb5ce0215e94ee2e70a94dc48"
        && r51.master_rows == 494U && r51.terminal.active.size() == 481U
        && binary64_bits(r51.terminal.primal.psi)
            == binary64_bits(AL_R52_R51_PSI)
        && binary64_bits(r51.terminal.primal.h)
            == binary64_bits(AL_R52_R51_H)
        && binary64_bits(r51.terminal.primal.maximum_upper)
            == binary64_bits(AL_R52_R51_MAXIMUM_UPPER);

    std::string dense_root;
    const bool dense = identity && al_r52_dense_controls(&dense_root);
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
        && binding.passed && superset.passed && moved.passed
        && al_r42_superset_root(superset) == superset_before
        && moved_filtered.pairs == moved.neighborhood.pairs;

    std::vector<Vec3> lower(r43.projected_trial.size());
    std::vector<Vec3> upper(r43.projected_trial.size());
    bool geometry_exact = workspace_exact;
    for (std::size_t i = 0U; i < r43.projected_trial.size(); ++i)
        for (int axis = 0; axis < 3; ++axis) {
            const double y = al_r41_component(r43.projected_trial[i], axis);
            const double low_face = al_r41_component(fixture.contact_low, axis);
            const double high_face = al_r41_component(fixture.contact_high, axis);
            double low = (low_face - y) / SPACING;
            double high = (high_face - y) / SPACING;
            if (y < low_face) low = 0.0;
            else if (y > high_face) high = 0.0;
            al_r43_component(lower[i], axis) = low;
            al_r43_component(upper[i], axis) = high;
            geometry_exact = geometry_exact && std::isfinite(low)
                && std::isfinite(high) && low <= 0.0 && high >= 0.0;
        }
    geometry_exact = geometry_exact
        && al_r49_feasible(r50.witness, lower, upper);

    ALR52Result depth;
    if (geometry_exact) depth = al_r52_generate(moved, r50, r51, lower, upper);
    if (moved.passed) release_al_normalized_workspace(moved, &workspace_work);
    const bool sweep_candidate = depth.selected && !depth.certified;
    const bool cg_polish = !depth.selected && depth.contraction;
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
        && depth.audit_jvp == 3U && depth.pair_passes == 3U
        && depth.projection_calls == 3U && depth.dense_sweeps == 112U;
    const bool rollback = position_before == al_binary64_vec3_root(r30.position)
        && trial_before == al_binary64_vec3_root(r43.projected_trial)
        && owner_before == al_r29_mask_root(r43.owners)
        && superset_before == al_r42_superset_root(r43.r42.superset)
        && r50_witness_before == al_binary64_vec3_root(r50.witness)
        && r51_witness_before == al_binary64_vec3_root(r51.witness)
        && r51.extended_cache_root == al_r51_cache_root(r51.cache);

    const std::array<const char*, 15U> expected_routes{{
        "SWEEP_PARENT_REJECTED", "SWEEP_SOURCE_REJECTED",
        "SWEEP_WORKSPACE_REJECTED", "SWEEP_MASTER_REJECTED",
        "SWEEP_BASIS_REJECTED", "SWEEP_GRAM_REJECTED",
        "SWEEP_DEPTH_REJECTED", "SWEEP_PROJECTION_REJECTED",
        "SWEEP_PRIMAL_AUDIT_REJECTED", "SWEEP_SELECTION_REJECTED",
        "SWEEP_WORK_REJECTED", "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE",
        "PERSISTENT_MASTER_SWEEP_DEPTH_CANDIDATE",
        "PERSISTENT_MASTER_CG_POLISH_REQUIRED",
        "PERSISTENT_MASTER_R51_REFERENCE_RETAINED"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t route_index = 0U;
         route_index < expected_routes.size(); ++route_index) {
        std::array<bool, 11U> gates{};
        gates.fill(true);
        if (route_index < gates.size()) gates[route_index] = false;
        const std::string observed = al_r52_route(
            gates[0], gates[1], gates[2], gates[3], gates[4], gates[5],
            gates[6], gates[7], gates[8], gates[9], gates[10],
            route_index == 11U, route_index == 12U, route_index == 13U);
        routes_exact = routes_exact && observed == expected_routes[route_index];
        route_material << expected_routes[route_index] << ':' << observed << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r52_route(parent_exact, source_exact,
        workspace_exact && geometry_exact, depth.master_exact,
        depth.basis_exact, depth.gram_exact, depth.depth_exact,
        depth.projection_exact, depth.primal_exact, depth.selection_exact,
        work_exact && rollback, depth.certified, sweep_candidate, cg_polish);
    const bool passed = identity && parent_exact && source_exact && dense
        && workspace_exact && geometry_exact && depth.master_exact
        && depth.basis_exact && depth.gram_exact && depth.depth_exact
        && depth.projection_exact && depth.primal_exact
        && depth.selection_exact && work_exact && rollback && routes_exact;

    std::ostringstream record_material;
    for (const ALR52Record& record : depth.records)
        record_material << record.root << ':';
    const std::string record_root = sha256_hex(record_material.str());
    const std::string witness_root = al_binary64_vec3_root(depth.witness);
    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R52_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << r50_witness_before << ':' << r51_witness_before << ':'
             << r51.union_root << ':' << r51.extended_cache_root << ':'
             << r51.solve.gram_root << '|' << depth.records.size() << ':'
             << record_root << ':' << depth.contraction << '|'
             << depth.selected << ':' << depth.selected_sweeps << ':'
             << depth.certified << ':' << depth.terminal.primal.psi << ':'
             << depth.terminal.primal.h << ':'
             << depth.terminal.primal.maximum_upper << ':'
             << depth.terminal.primal.raw_active << ':'
             << depth.terminal.active.size() << ':' << witness_root << '|'
             << depth.audit_jvp << ':' << depth.pair_passes << ':'
             << depth.projection_calls << ':' << depth.dense_sweeps << ':'
             << sweep_candidate << ':' << cg_polish << ':' << work_exact << ':'
             << rollback << ':' << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r52_persistent_master_sweep_depth.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R52_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"master\":{\"rows\":" << r51.master_rows
           << ",\"union_root\":\"" << r51.union_root
           << "\",\"cache_root\":\"" << r51.extended_cache_root
           << "\",\"gram_root\":\"" << r51.solve.gram_root << "\"}"
           << ",\"depth\":{\"records\":[";
    for (std::size_t i = 0U; i < depth.records.size(); ++i) {
        if (i != 0U) report << ',';
        const ALR52Record& record = depth.records[i];
        report << "{\"sweeps\":" << record.sweeps
               << ",\"predicted_maximum\":"
               << record.solve.core.predicted_maximum
               << ",\"psi\":" << record.audit.primal.psi
               << ",\"h\":" << record.audit.primal.h
               << ",\"maximum_upper\":"
               << record.audit.primal.maximum_upper
               << ",\"raw_active\":" << record.audit.primal.raw_active
               << ",\"certified_positive\":" << record.audit.active.size()
               << ",\"raw_outside_master\":"
               << record.raw_outside_master
               << ",\"directed_outside_master\":"
               << record.directed_outside_master
               << ",\"strict\":" << (record.strict ? "true" : "false")
               << ",\"certified\":"
               << (record.certified ? "true" : "false")
               << ",\"root\":\"" << record.root << "\"}";
    }
    report << "],\"record_root\":\"" << record_root
           << "\",\"contraction\":" << (depth.contraction ? "true" : "false")
           << ",\"selected\":" << (depth.selected ? "true" : "false")
           << ",\"selected_sweeps\":" << depth.selected_sweeps << "}"
           << ",\"primal\":{\"psi\":" << depth.terminal.primal.psi
           << ",\"h\":" << depth.terminal.primal.h
           << ",\"maximum_upper\":"
           << depth.terminal.primal.maximum_upper
           << ",\"raw_active\":" << depth.terminal.primal.raw_active
           << ",\"certified_positive\":" << depth.terminal.active.size()
           << ",\"witness_root\":\"" << witness_root
           << "\",\"certified\":"
           << (depth.certified ? "true" : "false") << "}"
           << ",\"work\":{\"workspace_builds\":"
           << workspace_work.workspace_builds << ",\"workspace_releases\":"
           << workspace_work.workspace_releases
           << ",\"row_vjp\":0,\"gram_jvp\":0,\"audit_jvp\":"
           << depth.audit_jvp << ",\"pair_passes\":" << depth.pair_passes
           << ",\"dense_sweeps\":" << depth.dense_sweeps
           << ",\"projection_calls\":" << depth.projection_calls
           << ",\"exact\":" << (work_exact ? "true" : "false") << "}"
           << ",\"dense_root\":\"" << dense_root << "\""
           << ",\"route_cases\":" << expected_routes.size()
           << ",\"route_root\":\"" << route_root
           << "\",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback ? "true" : "false")
           << ",\"route\":\"" << route
           << "\",\"normal_witness_applied\":false"
              ",\"r43_restoration_committed\":false"
              ",\"filter_runtime_committed\":false"
              ",\"restoration_exit_authorized\":false"
              ",\"following_outer_executed\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    if (replay_capture != nullptr) {
        replay_capture->captured = passed;
        replay_capture->r51 = std::move(capture);
        replay_capture->depth = std::move(depth);
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_persistent_master_sweep_depth_controls() {
    return run_al_persistent_master_sweep_depth_controls_impl(nullptr);
}
