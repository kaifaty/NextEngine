namespace {

constexpr const char* B4E2D7R19R51_IDENTITY_SHA256 =
    "9785a2168fdcb436cc06ff7c20277284c9e8fd5d27288374469f936f95ef5fcf";
constexpr const char* B4E2D7R19R51_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r51-persistent-active-face-master|v2|parent=717ffe6c:1ee396be61be90534573b8c476256d80b46ee803584617a6ab34900b8727b56d:a97b34d19b19b6d39725de8fdaf9ce38db924b7d39bfc823e07ea9bee86c7d8d:ACTIVE_FACE_HILDRETH_CLOSURE_CANDIDATE|source=r50-witness-e2ccfb4281b2b598b0a9864a8c9bb92f5c2512e018501147ca82b39b14b67592;cache-root-31a6713866e951e716b8fd77e40ae532c3d9d281d3b40b0d16cd8039c2c5112e;outer-root-04a91d0379f71f9c6dfb711bf385c8771837bffde1f176c995ebe7d814c5b20b;cache492;terminal-positive488;maximum-upper1.0297972392379541e-12|master=stable-ascending-union-of-r50-cache-rows-and-terminal-directed-positive-rows;include-currently-negative-cached-rows;capacity512;overflow-before-partial-add;no-row-drop|basis=reuse-r50-captured-gradient-and-gram-column-bit-exact;missing-row-one-vjp-plus-one-jvp;fixed-operator|solver=one-outer;hildreth-zero-dual;exact-eight-sweeps;stable-master-order;checkpoints1,2,4,8;inactive-release-via-lambda-zero;no-tolerance-stop|globalization=target=current+correction;unchanged-r48-ball-box-projection;feasible-chord;dyadic-alpha1-through2^-8-largest-first;fresh-directed-audit;accept-first-certified-or-strict-psi-h-maximum-upper-decrease|certificate=unchanged-r48-all-row-directed-jvp;zero-positive-only;outside-master-terminal-positive-count-exact;no-infeasibility-claim|selection=certificate-first;else-persistent-expansion-required-if-strict-progress-and-outside-master-positive>0;else-persistent-master-closure-candidate-if-strict-progress;else-r50-reference-retained|controls=r50-parent-bytes;cache-root;stable-union;capacity;basis-reuse;missing-basis;gram;hildreth;projection;dyadic-globalization;directed-certificate;outside-master;work;rollback;route-precedence|routes=persistent-parent-rejected;persistent-source-rejected;persistent-workspace-rejected;persistent-union-rejected;persistent-capacity-rejected;persistent-basis-rejected;persistent-gram-rejected;persistent-hildreth-rejected;persistent-projection-rejected;persistent-globalization-rejected;persistent-primal-audit-rejected;persistent-work-rejected;restoration-next-trqp-compatible-candidate;persistent-constraint-generation-expansion-required;persistent-master-closure-candidate;persistent-r50-reference-retained|precedence=parent,source,workspace,union,capacity,basis,gram,hildreth,projection,globalization,primal-audit,work,certificate,outside-master,selection|work=parent-r50-replays1;moved-workspace1-new;workspace-release1;captured-cache-reused;missing-rows<=20;row-vjp<=20;gram-jvp<=20;candidate-directed-jvp<=9;pairpasses<=49;sweeps8;projection-calls1;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50=unchanged;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;capacity-change=none;penalty-change=none;support-expansion=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-persistent-master-classification-only)IDENTITY";

std::string al_r51_identity_projection_v3() {
    std::string identity = B4E2D7R19R51_IDENTITY_PROJECTION;
    const std::size_t version = identity.find("|v2|");
    if (version != std::string::npos) identity.replace(version, 4U, "|v3|");
    return identity;
}

struct ALR51MasterResult {
    bool union_exact = false;
    bool capacity_exact = false;
    bool basis_exact = false;
    bool gram_exact = false;
    bool hildreth_exact = false;
    bool projection_exact = false;
    bool globalization_exact = false;
    bool primal_exact = false;
    bool selected = false;
    bool certified = false;
    std::size_t source_cache_rows = 0U;
    std::size_t source_positive_rows = 0U;
    std::size_t master_rows = 0U;
    std::size_t missing_rows = 0U;
    std::size_t row_vjp = 0U;
    std::size_t gram_jvp = 0U;
    std::size_t audit_jvp = 0U;
    std::size_t pair_passes = 0U;
    std::size_t selected_dyadic = 0U;
    std::size_t outside_master_positive = 0U;
    double alpha = 0.0;
    double witness_norm = 0.0;
    std::string union_root;
    std::string source_cache_root;
    std::string extended_cache_root;
    ALR50Solve solve;
    ALR50Audit terminal;
    std::vector<Vec3> witness;
};

std::string al_r51_cache_root(const std::vector<ALR50RowBasis>& cache) {
    std::ostringstream material;
    for (const ALR50RowBasis& basis : cache) material << basis.root << ':';
    return sha256_hex(material.str());
}

ALR51MasterResult al_r51_generate(const ALNormalizedWorkspace& workspace,
    ALR50Generator* parent, const std::string& expected_cache_root,
    const std::vector<Vec3>& lower, const std::vector<Vec3>& upper) {
    ALR51MasterResult result;
    if (parent == nullptr || !workspace.passed || !parent->passed
        || !parent->terminal.passed
        || !al_r49_feasible(parent->witness, lower, upper)) return result;

    result.source_cache_rows = parent->cache.size();
    result.source_positive_rows = parent->terminal.active.size();
    result.source_cache_root = al_r51_cache_root(parent->cache);
    const std::size_t row_count = workspace.support.constraint.size();
    std::vector<std::uint8_t> master_mask(row_count, 0U);
    bool rows_valid = true;
    for (const ALR50RowBasis& basis : parent->cache) {
        rows_valid = rows_valid && basis.row < row_count
            && master_mask[basis.row] == 0U;
        if (basis.row < row_count) master_mask[basis.row] = 1U;
    }
    for (std::size_t row : parent->terminal.active) {
        rows_valid = rows_valid && row < row_count;
        if (row < row_count) master_mask[row] = 1U;
    }
    std::vector<std::size_t> master_rows;
    for (std::size_t row = 0U; row < row_count; ++row)
        if (master_mask[row] != 0U) master_rows.push_back(row);
    result.master_rows = master_rows.size();
    result.union_root = al_r29_mask_root(master_mask);
    result.union_exact = rows_valid
        && result.source_cache_rows == 492U
        && result.source_positive_rows == 488U
        && result.source_cache_root == expected_cache_root
        && std::is_sorted(master_rows.begin(), master_rows.end());
    result.capacity_exact = result.union_exact
        && result.master_rows <= AL_R50_ROW_CAPACITY;
    if (!result.capacity_exact) return result;

    std::vector<ALR50RowBasis> cache = std::move(parent->cache);
    const std::size_t old_cache = cache.size();
    const bool cached = al_r50_cache_rows(workspace, master_rows, &cache,
        &result.row_vjp, &result.gram_jvp);
    result.missing_rows = cache.size() - old_cache;
    result.extended_cache_root = al_r51_cache_root(cache);
    result.basis_exact = cached && old_cache == 492U
        && result.missing_rows <= 20U
        && result.row_vjp == result.missing_rows
        && result.gram_jvp == result.missing_rows
        && result.source_cache_root == expected_cache_root;
    if (!result.basis_exact) return result;

    ALR50Audit master = parent->terminal;
    master.active = master_rows;
    master.active_root = result.union_root;
    result.solve = al_r50_solve(cache, master);
    result.gram_exact = result.solve.gram_exact;
    result.hildreth_exact = result.solve.passed
        && result.solve.core.sweeps == AL_R50_SWEEPS
        && result.solve.core.checkpoint_roots.size() == 4U;
    if (!result.hildreth_exact) return result;

    const std::vector<Vec3>& current = parent->witness;
    std::vector<Vec3> target(current.size());
    for (std::size_t i = 0U; i < current.size(); ++i)
        target[i] = current[i] + result.solve.correction[i];
    const ALR48Projection projection = al_r48_project_ball_box(
        target, lower, upper, AL_R48_NORMAL_RADIUS);
    result.projection_exact = projection.passed;
    if (!projection.passed) return result;
    std::vector<Vec3> chord(current.size());
    for (std::size_t i = 0U; i < current.size(); ++i)
        chord[i] = projection.value[i] - current[i];
    const double chord_norm = al_r32_vec3_norm(chord);
    if (!std::isfinite(chord_norm) || chord_norm == 0.0) return result;

    bool ladder_exact = true;
    for (std::size_t dyadic = 0U; dyadic < AL_R50_DYADIC_TRIALS; ++dyadic) {
        const double alpha = std::ldexp(1.0, -static_cast<int>(dyadic));
        std::vector<Vec3> candidate(current.size());
        for (std::size_t i = 0U; i < current.size(); ++i)
            candidate[i] = current[i] + alpha * chord[i];
        if (!al_r49_feasible(candidate, lower, upper)) {
            ladder_exact = false;
            break;
        }
        ALR50Audit audit = al_r50_audit(workspace, candidate);
        ++result.audit_jvp;
        if (!audit.passed) {
            ladder_exact = false;
            break;
        }
        const bool certified = audit.active.empty();
        const bool strict = audit.primal.psi < parent->terminal.primal.psi
            && audit.primal.h < parent->terminal.primal.h
            && audit.primal.maximum_upper
                < parent->terminal.primal.maximum_upper;
        if (certified || strict) {
            result.selected = true;
            result.certified = certified;
            result.selected_dyadic = dyadic;
            result.alpha = alpha;
            result.terminal = std::move(audit);
            result.witness = std::move(candidate);
            break;
        }
    }
    result.globalization_exact = ladder_exact
        && (result.selected || result.audit_jvp == AL_R50_DYADIC_TRIALS);
    if (!result.selected) {
        result.terminal = parent->terminal;
        result.witness = parent->witness;
    }
    result.primal_exact = result.globalization_exact && result.terminal.passed
        && (!result.certified || result.terminal.active.empty());
    for (std::size_t row : result.terminal.active)
        if (row >= master_mask.size() || master_mask[row] == 0U)
            ++result.outside_master_positive;
    result.witness_norm = al_r32_vec3_norm(result.witness);
    result.pair_passes = result.row_vjp + result.gram_jvp + result.audit_jvp;
    return result;
}

bool al_r51_dense_controls(std::string* output_root) {
    std::vector<std::uint8_t> mask(8U, 0U);
    for (std::size_t row : std::array<std::size_t, 3U>{{5U, 1U, 5U}})
        mask[row] = 1U;
    for (std::size_t row : std::array<std::size_t, 2U>{{2U, 1U}})
        mask[row] = 1U;
    std::vector<std::size_t> stable;
    for (std::size_t row = 0U; row < mask.size(); ++row)
        if (mask[row] != 0U) stable.push_back(row);
    const ALR50Core release = al_r50_hildreth_core(
        {2.0, 1.0, 1.0, 2.0}, {-1.0, 1.0});
    const bool passed = stable == std::vector<std::size_t>({1U, 2U, 5U})
        && release.passed && release.lambda.front() == 0.0
        && 492U + 20U == AL_R50_ROW_CAPACITY
        && 492U + 21U > AL_R50_ROW_CAPACITY;
    std::ostringstream material;
    material << passed << ':' << al_r29_mask_root(mask) << ':'
             << al_r10_dual_root(release.lambda) << ':'
             << al_r10_dual_root(release.predicted);
    if (output_root != nullptr) *output_root = sha256_hex(material.str());
    return passed;
}

std::string al_r51_route(bool parent, bool source, bool workspace,
    bool union_exact, bool capacity, bool basis, bool gram, bool hildreth,
    bool projection, bool globalization, bool primal, bool work,
    bool certified, bool expansion, bool closure) {
    if (!parent) return "PERSISTENT_PARENT_REJECTED";
    if (!source) return "PERSISTENT_SOURCE_REJECTED";
    if (!workspace) return "PERSISTENT_WORKSPACE_REJECTED";
    if (!union_exact) return "PERSISTENT_UNION_REJECTED";
    if (!capacity) return "PERSISTENT_CAPACITY_REJECTED";
    if (!basis) return "PERSISTENT_BASIS_REJECTED";
    if (!gram) return "PERSISTENT_GRAM_REJECTED";
    if (!hildreth) return "PERSISTENT_HILDRETH_REJECTED";
    if (!projection) return "PERSISTENT_PROJECTION_REJECTED";
    if (!globalization) return "PERSISTENT_GLOBALIZATION_REJECTED";
    if (!primal) return "PERSISTENT_PRIMAL_AUDIT_REJECTED";
    if (!work) return "PERSISTENT_WORK_REJECTED";
    if (certified) return "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE";
    if (expansion)
        return "PERSISTENT_CONSTRAINT_GENERATION_EXPANSION_REQUIRED";
    if (closure) return "PERSISTENT_MASTER_CLOSURE_CANDIDATE";
    return "PERSISTENT_R50_REFERENCE_RETAINED";
}

} // namespace

SplitBoundaryReport run_al_persistent_master_controls() {
    const bool identity = sha256_hex(al_r51_identity_projection_v3())
        == B4E2D7R19R51_IDENTITY_SHA256;
    ALR50ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_active_face_closure_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "1ee396be61be90534573b8c476256d80b46ee803584617a6ab34900b8727b56d"
        && parent.json.find(
            "a97b34d19b19b6d39725de8fdaf9ce38db924b7d39bfc823e07ea9bee86c7d8d")
            != std::string::npos
        && parent.json.find("ACTIVE_FACE_HILDRETH_CLOSURE_CANDIDATE")
            != std::string::npos;
    ALR50Generator& parent_generator = capture.generator;
    ALR43ReplayCapture& r43 =
        capture.r49.r48.r47.r46.r45.r44.r43;
    ALR40ReplayCapture& r40 = r43.r42.r40;
    ALR30RangeProjectionCapture& r30 =
        r40.r39.r38.r37.r36.r35.r34.r33.r32.r31.r30;
    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string trial_before = al_binary64_vec3_root(r43.projected_trial);
    const std::string owner_before = al_r29_mask_root(r43.owners);
    const std::string superset_before = al_r42_superset_root(r43.r42.superset);
    const bool source_exact = parent_exact && r30.captured
        && capture.cache_root
            == "31a6713866e951e716b8fd77e40ae532c3d9d281d3b40b0d16cd8039c2c5112e"
        && capture.outer_root
            == "04a91d0379f71f9c6dfb711bf385c8771837bffde1f176c995ebe7d814c5b20b"
        && capture.witness_root
            == "e2ccfb4281b2b598b0a9864a8c9bb92f5c2512e018501147ca82b39b14b67592"
        && parent_generator.cache.size() == 492U
        && parent_generator.terminal.active.size() == 488U
        && binary64_bits(parent_generator.terminal.primal.maximum_upper)
            == binary64_bits(1.0297972392379541e-12)
        && position_before
            == "e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828"
        && trial_before
            == "0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50"
        && owner_before
            == "baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187"
        && superset_before
            == "355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01";

    std::string dense_root;
    const bool dense = identity && al_r51_dense_controls(&dense_root);
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
    std::vector<std::uint8_t> moved_owners(
        r43.projected_trial.size() * 3U, 0U);
    std::size_t moved_active_faces = 0U;
    bool intervals_exact = workspace_exact;
    for (std::size_t i = 0U; i < r43.projected_trial.size(); ++i)
        for (int axis = 0; axis < 3; ++axis) {
            const double y = al_r41_component(r43.projected_trial[i], axis);
            const double low_face = al_r41_component(fixture.contact_low, axis);
            const double high_face = al_r41_component(fixture.contact_high, axis);
            double low = (low_face - y) / SPACING;
            double high = (high_face - y) / SPACING;
            const std::size_t component =
                3U * i + static_cast<std::size_t>(axis);
            if (y < low_face) {
                low = 0.0;
                moved_owners[component] = 1U;
                ++moved_active_faces;
            } else if (y > high_face) {
                high = 0.0;
                moved_owners[component] = 2U;
                ++moved_active_faces;
            }
            al_r43_component(lower[i], axis) = low;
            al_r43_component(upper[i], axis) = high;
            intervals_exact = intervals_exact && std::isfinite(low)
                && std::isfinite(high) && low <= 0.0 && high >= 0.0;
        }
    const double moved_step_norm = al_r32_vec3_norm(r43.projected_endpoint);
    const double half_skin = 0.5 * B4EP3_SKIN
        * std::sqrt(1.0 - 2.0e-12) / SPACING;
    const std::string moved_owner_root = al_r29_mask_root(moved_owners);
    const bool geometry_exact = intervals_exact
        && moved_step_norm + AL_R48_NORMAL_RADIUS <= half_skin
        && moved_active_faces == 1290U && moved_owner_root == owner_before
        && al_r49_feasible(parent_generator.witness, lower, upper);

    ALR51MasterResult master;
    if (geometry_exact)
        master = al_r51_generate(moved, &parent_generator,
            capture.cache_root, lower, upper);
    if (moved.passed) release_al_normalized_workspace(moved, &workspace_work);

    const bool selected_strict = master.selected && master.primal_exact
        && master.terminal.primal.psi
            < 4.4684544709411563e-24
        && master.terminal.primal.h
            < 2.9894663306152676e-12
        && master.terminal.primal.maximum_upper
            < 1.0297972392379541e-12;
    const bool expansion = selected_strict && !master.certified
        && master.outside_master_positive > 0U;
    const bool closure = selected_strict && !master.certified
        && master.outside_master_positive == 0U;
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
        && master.missing_rows <= 20U
        && master.row_vjp <= 20U && master.gram_jvp <= 20U
        && master.audit_jvp <= 9U && master.pair_passes <= 49U
        && master.solve.core.sweeps == AL_R50_SWEEPS;
    const bool rollback = position_before == al_binary64_vec3_root(r30.position)
        && trial_before == al_binary64_vec3_root(r43.projected_trial)
        && owner_before == al_r29_mask_root(r43.owners)
        && superset_before == al_r42_superset_root(r43.r42.superset)
        && capture.witness_root
            == al_binary64_vec3_root(capture.generator.witness);

    const std::array<const char*, 16U> expected_routes{{
        "PERSISTENT_PARENT_REJECTED", "PERSISTENT_SOURCE_REJECTED",
        "PERSISTENT_WORKSPACE_REJECTED", "PERSISTENT_UNION_REJECTED",
        "PERSISTENT_CAPACITY_REJECTED", "PERSISTENT_BASIS_REJECTED",
        "PERSISTENT_GRAM_REJECTED", "PERSISTENT_HILDRETH_REJECTED",
        "PERSISTENT_PROJECTION_REJECTED",
        "PERSISTENT_GLOBALIZATION_REJECTED",
        "PERSISTENT_PRIMAL_AUDIT_REJECTED", "PERSISTENT_WORK_REJECTED",
        "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE",
        "PERSISTENT_CONSTRAINT_GENERATION_EXPANSION_REQUIRED",
        "PERSISTENT_MASTER_CLOSURE_CANDIDATE",
        "PERSISTENT_R50_REFERENCE_RETAINED"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t route_index = 0U;
         route_index < expected_routes.size(); ++route_index) {
        std::array<bool, 12U> gates{};
        gates.fill(true);
        if (route_index < gates.size()) gates[route_index] = false;
        const std::string observed = al_r51_route(
            gates[0], gates[1], gates[2], gates[3], gates[4], gates[5],
            gates[6], gates[7], gates[8], gates[9], gates[10], gates[11],
            route_index == 12U, route_index == 13U, route_index == 14U);
        routes_exact = routes_exact && observed == expected_routes[route_index];
        route_material << expected_routes[route_index] << ':' << observed << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r51_route(parent_exact, source_exact,
        workspace_exact && geometry_exact, master.union_exact,
        master.capacity_exact, master.basis_exact, master.gram_exact,
        master.hildreth_exact, master.projection_exact,
        master.globalization_exact, master.primal_exact,
        work_exact && rollback, master.certified, expansion, closure);
    const bool passed = identity && parent_exact && source_exact && dense
        && workspace_exact && geometry_exact && master.union_exact
        && master.capacity_exact && master.basis_exact && master.gram_exact
        && master.hildreth_exact && master.projection_exact
        && master.globalization_exact && master.primal_exact
        && work_exact && rollback && routes_exact;

    const std::string witness_root = al_binary64_vec3_root(master.witness);
    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R51_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << capture.cache_root << ':' << capture.outer_root << ':'
             << capture.witness_root << '|' << master.source_cache_rows << ':'
             << master.source_positive_rows << ':' << master.master_rows << ':'
             << master.missing_rows << ':' << master.union_root << ':'
             << master.extended_cache_root << '|' << master.solve.gram_root << ':'
             << master.solve.root << ':' << master.solve.core.predicted_maximum
             << '|' << master.selected << ':' << master.selected_dyadic << ':'
             << master.alpha << ':' << master.terminal.primal.psi << ':'
             << master.terminal.primal.h << ':'
             << master.terminal.primal.maximum_upper << ':'
             << master.terminal.active.size() << ':'
             << master.outside_master_positive << ':' << master.witness_norm
             << ':' << witness_root << '|' << master.row_vjp << ':'
             << master.gram_jvp << ':' << master.audit_jvp << ':'
             << master.pair_passes << ':' << selected_strict << ':'
             << master.certified << ':' << expansion << ':' << closure << ':'
             << work_exact << ':' << rollback << ':' << route_root << ':'
             << route;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r51_persistent_active_face_master.v3\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R51_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"master\":{\"source_cache_rows\":"
           << master.source_cache_rows << ",\"source_positive_rows\":"
           << master.source_positive_rows << ",\"rows\":"
           << master.master_rows << ",\"missing_rows\":"
           << master.missing_rows << ",\"union_root\":\""
           << master.union_root << "\",\"source_cache_root\":\""
           << master.source_cache_root << "\",\"extended_cache_root\":\""
           << master.extended_cache_root << "\",\"gram_root\":\""
           << master.solve.gram_root << "\",\"solve_root\":\""
           << master.solve.root << "\",\"predicted_maximum\":"
           << master.solve.core.predicted_maximum << "}"
           << ",\"primal\":{\"selected\":"
           << (master.selected ? "true" : "false")
           << ",\"selected_dyadic\":" << master.selected_dyadic
           << ",\"alpha\":" << master.alpha << ",\"psi\":"
           << master.terminal.primal.psi << ",\"h\":"
           << master.terminal.primal.h << ",\"maximum_upper\":"
           << master.terminal.primal.maximum_upper
           << ",\"certified_positive\":" << master.terminal.active.size()
           << ",\"outside_master_positive\":"
           << master.outside_master_positive << ",\"witness_norm\":"
           << master.witness_norm << ",\"witness_root\":\""
           << witness_root << "\",\"certified\":"
           << (master.certified ? "true" : "false") << "}"
           << ",\"work\":{\"workspace_builds\":"
           << workspace_work.workspace_builds << ",\"workspace_releases\":"
           << workspace_work.workspace_releases << ",\"row_vjp\":"
           << master.row_vjp << ",\"gram_jvp\":" << master.gram_jvp
           << ",\"audit_jvp\":" << master.audit_jvp
           << ",\"pair_passes\":" << master.pair_passes
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
    return {passed, report.str()};
}
