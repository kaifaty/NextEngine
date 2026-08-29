namespace {

constexpr const char* B4E2D7R19R53_IDENTITY_SHA256 =
    "3dbdcd223d53816eeb3c196e03c20606aefe6c0427f3356157d869b7401aacc6";
constexpr const char* B4E2D7R19R53_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r53-operator-consistency-decomposition|v1|parent=2549bdd5:b1d439950f174b66b837041d077ed24fd19524bf2880ded37fc6392f9c65cf4e:ad1efbe449d809039ad41404dc94736f8fcb536c48d724f38598f70d99cc744b:PERSISTENT_MASTER_SWEEP_DEPTH_CANDIDATE|source=r52-master494;union-38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28;cache-f4e9368971d4b6feeb13874d0279dc73fd54ebbb523c75c5c2dd303acfc58cd5;gram-7a0358af3fda7c90c89a5cfb9abe125fadf9142fb5ce0215e94ee2e70a94dc48;record64-2e0afdb4dbdac10cef5ba591b5764f7cce40ca414458e7f5b9c61cdbc83425f0;predicted1.8707447849842052e-20;raw-active219;directed-positive340;outside0|oracle=unchanged-r52-64-witness;captured-directed-binary64;fresh-pair-once-binary64;directed-binary64-terms-compensated-binary128-fold;frozen-coefficient-directed-binary128-recompute-compensated;stable-topology|rows=raw-positive-count;maximum-raw-and-row-per-representation;pair-full128-sign-disagreements;pair-direct-maximum-difference;direct-term128-maximum-difference;term128-full128-maximum-difference;worst-row-roots|enclosure=current-gamma-unchanged;raw-plus-bound;maximum-bound-and-row;bound-only-positive-count;binary128-forward-envelope-and-resolved-sign-counts|selection=pair-full128-resolved-sign-disagreement-first;else-full128-resolved-positive-high-precision-raw-residual;else-current-certified-positive-enclosure-tightening;else-restoration-next-trqp-compatible|controls=r52-parent-bytes;source;workspace;topology;master;witness;pair-once;directed-capture;term128-fold;full128-recompute;row-comparison;enclosure;work;rollback;route-precedence|routes=operator-parent-rejected;operator-source-rejected;operator-workspace-rejected;operator-topology-rejected;operator-master-rejected;operator-witness-rejected;operator-pair-rejected;operator-directed-rejected;operator-binary128-rejected;operator-row-comparison-rejected;operator-enclosure-rejected;operator-work-rejected;pair-directed-operator-alignment-required;high-precision-raw-residual-confirmed;directed-certificate-enclosure-tightening-required;restoration-next-trqp-compatible-candidate|precedence=parent,source,workspace,topology,master,witness,pair,directed,binary128,row-comparison,enclosure,work,alignment,raw-residual,enclosure-tightening,compatibility|work=parent-r52-replays1;moved-workspace1;workspace-release1;fresh-pair-once-jvp1;binary128-directed-row-traversals1;captured-directed-reused;pairpasses1;quad-row-passes1;new-row-vjp0;new-gram-jvp0;new-audit-jvp0;new-sweeps0;new-projection0;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52=unchanged;certificate=current-directed-unchanged;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;gamma-change=none;capacity-change=none;penalty-change=none;support-expansion=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-operator-consistency-classification-only)IDENTITY";

struct ALR53Maximum {
    Binary128 value = -HUGE_VALQ;
    std::size_t row = 0U;
};

struct ALR53Difference {
    Binary128 value = static_cast<Binary128>(0.0);
    std::size_t row = 0U;
};

struct ALR53Result {
    bool pair_exact = false;
    bool directed_exact = false;
    bool binary128_exact = false;
    bool comparison_exact = false;
    bool enclosure_exact = false;
    std::size_t pair_positive = 0U;
    std::size_t directed_positive = 0U;
    std::size_t term128_positive = 0U;
    std::size_t full128_resolved_positive = 0U;
    std::size_t full128_resolved_negative = 0U;
    std::size_t full128_unresolved = 0U;
    std::size_t pair_full_sign_disagreements = 0U;
    std::size_t directed_bound_only_positive = 0U;
    std::size_t pair_passes = 0U;
    std::size_t quad_row_passes = 0U;
    ALR53Maximum pair_maximum;
    ALR53Maximum directed_maximum;
    ALR53Maximum term128_maximum;
    ALR53Maximum full128_maximum;
    ALR53Maximum full128_lower_maximum;
    ALR53Maximum full128_upper_maximum;
    ALR53Maximum bound_maximum;
    ALR53Maximum upper_maximum;
    ALR53Difference pair_direct_difference;
    ALR53Difference direct_term128_difference;
    ALR53Difference term128_full_difference;
    std::string pair_root;
    std::string directed_root;
    std::string term128_root;
    std::string full128_root;
    std::string comparison_root;
};

void al_r53_update_maximum(
    ALR53Maximum* maximum, Binary128 value, std::size_t row) {
    if (maximum != nullptr && value > maximum->value) {
        maximum->value = value;
        maximum->row = row;
    }
}

void al_r53_update_difference(
    ALR53Difference* maximum, Binary128 value, std::size_t row) {
    if (maximum != nullptr && value > maximum->value) {
        maximum->value = value;
        maximum->row = row;
    }
}

Binary128 al_r53_gamma(std::size_t operations) {
    const Binary128 epsilon = scalbnq(static_cast<Binary128>(1.0), -112);
    const Binary128 value = static_cast<Binary128>(operations) * epsilon;
    return value / (static_cast<Binary128>(1.0) - value);
}

ALR53Result al_r53_decompose(const ALNormalizedWorkspace& workspace,
    const std::vector<Vec3>& witness, const ALR50Audit& directed) {
    ALR53Result result;
    if (!workspace.passed || witness.size() != workspace.neighborhood.fluid.size()
        || !directed.passed
        || directed.image.value.size() != workspace.support.constraint.size())
        return result;
    const std::size_t count = workspace.support.constraint.size();
    const ALLinearizedJvp pair = al_r29_pair_once_jvp(workspace, witness);
    ++result.pair_passes;
    result.pair_exact = pair.passed && pair.value.size() == count;
    result.directed_exact = directed.image.passed
        && directed.image.value.size() == count
        && directed.image.absolute_sum.size() == count;
    if (!result.pair_exact || !result.directed_exact) return result;

    std::vector<Binary128Accumulator> term_fold(count);
    std::vector<Binary128Accumulator> full_fold(count);
    std::vector<Binary128Accumulator> full_absolute(count);
    for (std::size_t center = 0U; center < count; ++center) {
        const std::size_t begin = workspace.neighborhood.flat_offsets[center];
        const std::size_t end = workspace.neighborhood.flat_offsets[center + 1U];
        for (std::size_t slot = begin; slot < end; ++slot) {
            const std::size_t pair_index =
                workspace.neighborhood.flat_directed_pair_indices[slot];
            const std::size_t participant = joint_pair_participant(
                workspace.neighborhood,
                workspace.neighborhood.pairs[pair_index], center);
            const double radius = workspace.radius[pair_index];
            if (radius <= 1.0e-15 || radius > HORIZON) continue;
            const Vec3 center_position =
                workspace.neighborhood.fluid[center].position;
            const Vec3 participant_position = joint_position(
                workspace.neighborhood, participant);
            const Vec3 displacement = center_position - participant_position;
            const Vec3 jacobian = MASS / REST_DENSITY
                * workspace.weight_first[pair_index]
                * (displacement / radius);
            const Vec3 relative = participant < count
                ? witness[center] - witness[participant]
                : witness[center];
            const double term = SPACING * dot(jacobian, relative);
            term_fold[center].add(static_cast<Binary128>(term));

            const Binary128 dx = static_cast<Binary128>(center_position.x)
                - static_cast<Binary128>(participant_position.x);
            const Binary128 dy = static_cast<Binary128>(center_position.y)
                - static_cast<Binary128>(participant_position.y);
            const Binary128 dz = static_cast<Binary128>(center_position.z)
                - static_cast<Binary128>(participant_position.z);
            const Binary128 rx = static_cast<Binary128>(relative.x);
            const Binary128 ry = static_cast<Binary128>(relative.y);
            const Binary128 rz = static_cast<Binary128>(relative.z);
            const Binary128 scale = static_cast<Binary128>(MASS)
                / static_cast<Binary128>(REST_DENSITY)
                * static_cast<Binary128>(workspace.weight_first[pair_index])
                / static_cast<Binary128>(radius);
            const Binary128 full_term = static_cast<Binary128>(SPACING)
                * scale * (dx * rx + dy * ry + dz * rz);
            full_fold[center].add(full_term);
            full_absolute[center].add(fabsq(full_term));
        }
    }
    ++result.quad_row_passes;

    std::ostringstream term_material;
    std::ostringstream full_material;
    const std::vector<double>& constraint = workspace.support.constraint;
    const Binary128 quad_gamma = al_r53_gamma(
        16U * workspace.neighborhood.maximum_degree + 66U);
    bool finite = true;
    for (std::size_t row = 0U; row < count; ++row) {
        const Binary128 c = static_cast<Binary128>(constraint[row]);
        const Binary128 pair_raw = static_cast<Binary128>(
            constraint[row] + pair.value[row]);
        const Binary128 directed_raw = static_cast<Binary128>(
            constraint[row] + directed.image.value[row]);
        const Binary128 term_raw = c + term_fold[row].value();
        const Binary128 full_raw = c + full_fold[row].value();
        const Binary128 full_bound = quad_gamma
            * (fabsq(c) + full_absolute[row].value());
        const double directed_bound_double = gamma_factor(
            16U * workspace.neighborhood.maximum_degree + 66U)
            * (std::abs(constraint[row])
                + directed.image.absolute_sum[row]);
        const Binary128 directed_bound =
            static_cast<Binary128>(directed_bound_double);
        const Binary128 directed_upper =
            static_cast<Binary128>(directed.upper[row]);

        if (pair_raw > static_cast<Binary128>(0.0)) ++result.pair_positive;
        if (directed_raw > static_cast<Binary128>(0.0))
            ++result.directed_positive;
        if (term_raw > static_cast<Binary128>(0.0)) ++result.term128_positive;
        const bool full_positive = full_raw - full_bound
            > static_cast<Binary128>(0.0);
        const bool full_negative = full_raw + full_bound
            < static_cast<Binary128>(0.0);
        if (full_positive) ++result.full128_resolved_positive;
        else if (full_negative) ++result.full128_resolved_negative;
        else ++result.full128_unresolved;
        const bool pair_sign = pair_raw > static_cast<Binary128>(0.0);
        if ((full_positive && !pair_sign) || (full_negative && pair_sign))
            ++result.pair_full_sign_disagreements;
        if (directed_raw <= static_cast<Binary128>(0.0)
            && directed_upper > static_cast<Binary128>(0.0))
            ++result.directed_bound_only_positive;

        al_r53_update_maximum(&result.pair_maximum, pair_raw, row);
        al_r53_update_maximum(&result.directed_maximum, directed_raw, row);
        al_r53_update_maximum(&result.term128_maximum, term_raw, row);
        al_r53_update_maximum(&result.full128_maximum, full_raw, row);
        al_r53_update_maximum(
            &result.full128_lower_maximum, full_raw - full_bound, row);
        al_r53_update_maximum(
            &result.full128_upper_maximum, full_raw + full_bound, row);
        al_r53_update_maximum(&result.bound_maximum, directed_bound, row);
        al_r53_update_maximum(&result.upper_maximum, directed_upper, row);
        al_r53_update_difference(&result.pair_direct_difference,
            fabsq(pair_raw - directed_raw), row);
        al_r53_update_difference(&result.direct_term128_difference,
            fabsq(directed_raw - term_raw), row);
        al_r53_update_difference(&result.term128_full_difference,
            fabsq(term_raw - full_raw), row);
        term_material << al_binary128_hex(term_raw) << ':';
        full_material << al_binary128_hex(full_raw) << ':'
                      << al_binary128_hex(full_bound) << ':';
        finite = finite && finiteq(pair_raw) != 0 && finiteq(directed_raw) != 0
            && finiteq(term_raw) != 0 && finiteq(full_raw) != 0
            && finiteq(full_bound) != 0;
    }
    result.pair_root = al_r10_dual_root(pair.value);
    result.directed_root = al_r10_dual_root(directed.image.value);
    result.term128_root = sha256_hex(term_material.str());
    result.full128_root = sha256_hex(full_material.str());
    std::ostringstream comparison;
    comparison << result.pair_positive << ':' << result.directed_positive << ':'
               << result.term128_positive << ':'
               << result.full128_resolved_positive << ':'
               << result.full128_resolved_negative << ':'
               << result.full128_unresolved << ':'
               << result.pair_full_sign_disagreements << ':'
               << result.directed_bound_only_positive << '|'
               << al_binary128_hex(result.pair_maximum.value) << ':'
               << result.pair_maximum.row << ':'
               << al_binary128_hex(result.directed_maximum.value) << ':'
               << result.directed_maximum.row << ':'
               << al_binary128_hex(result.term128_maximum.value) << ':'
               << result.term128_maximum.row << ':'
               << al_binary128_hex(result.full128_maximum.value) << ':'
               << result.full128_maximum.row << ':'
               << al_binary128_hex(result.bound_maximum.value) << ':'
               << result.bound_maximum.row << ':'
               << al_binary128_hex(result.upper_maximum.value) << ':'
               << result.upper_maximum.row << '|'
               << al_binary128_hex(result.pair_direct_difference.value) << ':'
               << result.pair_direct_difference.row << ':'
               << al_binary128_hex(result.direct_term128_difference.value) << ':'
               << result.direct_term128_difference.row << ':'
               << al_binary128_hex(result.term128_full_difference.value) << ':'
               << result.term128_full_difference.row << '|'
               << result.pair_root << ':' << result.directed_root << ':'
               << result.term128_root << ':' << result.full128_root;
    result.comparison_root = sha256_hex(comparison.str());
    result.binary128_exact = finite && result.quad_row_passes == 1U
        && result.full128_resolved_positive + result.full128_resolved_negative
            + result.full128_unresolved == count;
    result.comparison_exact = result.binary128_exact
        && result.pair_positive <= count && result.directed_positive <= count
        && result.term128_positive <= count;
    result.enclosure_exact = result.comparison_exact
        && result.directed_positive == directed.primal.raw_active
        && result.upper_maximum.row < count
        && binary64_bits(static_cast<double>(result.upper_maximum.value))
            == binary64_bits(directed.primal.maximum_upper)
        && result.directed_positive + result.directed_bound_only_positive
            == directed.active.size();
    return result;
}

bool al_r53_dense_controls(std::string* output_root) {
    Binary128Accumulator sum;
    sum.add(static_cast<Binary128>(1.0e30));
    sum.add(static_cast<Binary128>(1.0));
    sum.add(static_cast<Binary128>(-1.0e30));
    const Binary128 gamma = al_r53_gamma(1024U);
    const bool passed = sum.value() == static_cast<Binary128>(1.0)
        && gamma > static_cast<Binary128>(0.0)
        && gamma < static_cast<Binary128>(1.0e-28);
    std::ostringstream material;
    material << passed << ':' << al_binary128_hex(sum.value()) << ':'
             << al_binary128_hex(gamma);
    if (output_root != nullptr) *output_root = sha256_hex(material.str());
    return passed;
}

std::string al_r53_route(bool parent, bool source, bool workspace,
    bool topology, bool master, bool witness, bool pair, bool directed,
    bool binary128, bool comparison, bool enclosure, bool work,
    bool alignment, bool raw_residual, bool tightening) {
    if (!parent) return "OPERATOR_PARENT_REJECTED";
    if (!source) return "OPERATOR_SOURCE_REJECTED";
    if (!workspace) return "OPERATOR_WORKSPACE_REJECTED";
    if (!topology) return "OPERATOR_TOPOLOGY_REJECTED";
    if (!master) return "OPERATOR_MASTER_REJECTED";
    if (!witness) return "OPERATOR_WITNESS_REJECTED";
    if (!pair) return "OPERATOR_PAIR_REJECTED";
    if (!directed) return "OPERATOR_DIRECTED_REJECTED";
    if (!binary128) return "OPERATOR_BINARY128_REJECTED";
    if (!comparison) return "OPERATOR_ROW_COMPARISON_REJECTED";
    if (!enclosure) return "OPERATOR_ENCLOSURE_REJECTED";
    if (!work) return "OPERATOR_WORK_REJECTED";
    if (alignment) return "PAIR_DIRECTED_OPERATOR_ALIGNMENT_REQUIRED";
    if (raw_residual) return "HIGH_PRECISION_RAW_RESIDUAL_CONFIRMED";
    if (tightening) return "DIRECTED_CERTIFICATE_ENCLOSURE_TIGHTENING_REQUIRED";
    return "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE";
}

struct ALR53ReplayCapture {
    bool captured = false;
    ALR52ReplayCapture r52;
    ALR53Result decomposition;
};

} // namespace

SplitBoundaryReport run_al_operator_consistency_decomposition_controls_impl(
    ALR53ReplayCapture* replay_capture) {
    const bool identity = sha256_hex(B4E2D7R19R53_IDENTITY_PROJECTION)
        == B4E2D7R19R53_IDENTITY_SHA256;
    ALR52ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_persistent_master_sweep_depth_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "b1d439950f174b66b837041d077ed24fd19524bf2880ded37fc6392f9c65cf4e"
        && parent.json.find(
            "ad1efbe449d809039ad41404dc94736f8fcb536c48d724f38598f70d99cc744b")
            != std::string::npos
        && parent.json.find("PERSISTENT_MASTER_SWEEP_DEPTH_CANDIDATE")
            != std::string::npos;
    ALR52Result& r52 = capture.depth;
    ALR51MasterResult& r51 = capture.r51.master;
    ALR50Generator& r50 = capture.r51.r50.generator;
    ALR43ReplayCapture& r43 =
        capture.r51.r50.r49.r48.r47.r46.r45.r44.r43;
    ALR40ReplayCapture& r40 = r43.r42.r40;
    ALR30RangeProjectionCapture& r30 =
        r40.r39.r38.r37.r36.r35.r34.r33.r32.r31.r30;
    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string trial_before = al_binary64_vec3_root(r43.projected_trial);
    const std::string owner_before = al_r29_mask_root(r43.owners);
    const std::string superset_before = al_r42_superset_root(r43.r42.superset);
    const bool source_exact = parent_exact && r30.captured
        && r51.master_rows == 494U
        && r51.union_root
            == "38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28"
        && r51.extended_cache_root
            == "f4e9368971d4b6feeb13874d0279dc73fd54ebbb523c75c5c2dd303acfc58cd5"
        && r51.solve.gram_root
            == "7a0358af3fda7c90c89a5cfb9abe125fadf9142fb5ce0215e94ee2e70a94dc48"
        && r52.records.size() == 3U && r52.records[2].sweeps == 64U
        && r52.records[2].root
            == "2e0afdb4dbdac10cef5ba591b5764f7cce40ca414458e7f5b9c61cdbc83425f0"
        && r52.records[2].audit.primal.raw_active == 219U
        && r52.records[2].audit.active.size() == 340U
        && r52.records[2].directed_outside_master == 0U;
    const std::string witness_before = source_exact
        ? al_binary64_vec3_root(r52.records[2].witness) : std::string{};

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
    const bool witness_exact = master_exact && !witness_before.empty()
        && r52.records[2].witness.size()
            == moved.neighborhood.fluid.size()
        && al_r48_vec_finite(r52.records[2].witness);

    ALR53Result decomposition;
    if (witness_exact) decomposition = al_r53_decompose(
        moved, r52.records[2].witness, r52.records[2].audit);
    if (moved.passed) release_al_normalized_workspace(moved, &workspace_work);
    const bool alignment = decomposition.pair_full_sign_disagreements > 0U;
    const bool raw_residual = !alignment
        && decomposition.full128_resolved_positive > 0U;
    const bool tightening = !alignment && !raw_residual
        && r52.records[2].audit.active.size() > 0U;
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
    const bool rollback = position_before == al_binary64_vec3_root(r30.position)
        && trial_before == al_binary64_vec3_root(r43.projected_trial)
        && owner_before == al_r29_mask_root(r43.owners)
        && superset_before == al_r42_superset_root(r43.r42.superset)
        && witness_before == al_binary64_vec3_root(r52.records[2].witness)
        && r50.witness.size() == r52.records[2].witness.size();

    const std::array<const char*, 16U> expected_routes{{
        "OPERATOR_PARENT_REJECTED", "OPERATOR_SOURCE_REJECTED",
        "OPERATOR_WORKSPACE_REJECTED", "OPERATOR_TOPOLOGY_REJECTED",
        "OPERATOR_MASTER_REJECTED", "OPERATOR_WITNESS_REJECTED",
        "OPERATOR_PAIR_REJECTED", "OPERATOR_DIRECTED_REJECTED",
        "OPERATOR_BINARY128_REJECTED", "OPERATOR_ROW_COMPARISON_REJECTED",
        "OPERATOR_ENCLOSURE_REJECTED", "OPERATOR_WORK_REJECTED",
        "PAIR_DIRECTED_OPERATOR_ALIGNMENT_REQUIRED",
        "HIGH_PRECISION_RAW_RESIDUAL_CONFIRMED",
        "DIRECTED_CERTIFICATE_ENCLOSURE_TIGHTENING_REQUIRED",
        "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t route_index = 0U;
         route_index < expected_routes.size(); ++route_index) {
        std::array<bool, 12U> gates{};
        gates.fill(true);
        if (route_index < gates.size()) gates[route_index] = false;
        const std::string observed = al_r53_route(
            gates[0], gates[1], gates[2], gates[3], gates[4], gates[5],
            gates[6], gates[7], gates[8], gates[9], gates[10], gates[11],
            route_index == 12U, route_index == 13U, route_index == 14U);
        routes_exact = routes_exact && observed == expected_routes[route_index];
        route_material << expected_routes[route_index] << ':' << observed << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r53_route(parent_exact, source_exact,
        workspace_exact, topology_exact, master_exact, witness_exact,
        decomposition.pair_exact, decomposition.directed_exact,
        decomposition.binary128_exact, decomposition.comparison_exact,
        decomposition.enclosure_exact, work_exact && rollback,
        alignment, raw_residual, tightening);
    const bool passed = identity && parent_exact && source_exact && dense
        && workspace_exact && topology_exact && master_exact && witness_exact
        && decomposition.pair_exact && decomposition.directed_exact
        && decomposition.binary128_exact && decomposition.comparison_exact
        && decomposition.enclosure_exact && work_exact && rollback
        && routes_exact;

    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R53_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << witness_before << ':' << r52.records[2].root << '|'
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
             << raw_residual << ':' << tightening << ':' << work_exact << ':'
             << rollback << ':' << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r53_operator_consistency_decomposition.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R53_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source\":{\"witness_root\":\"" << witness_before
           << "\",\"record_root\":\"" << r52.records[2].root << "\"}"
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
           << ",\"gamma_bound\":"
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
           << "\",\"comparison\":\"" << decomposition.comparison_root << "\"}"
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
              ",\"normal_witness_applied\":false"
              ",\"r43_restoration_committed\":false"
              ",\"restoration_exit_authorized\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    if (replay_capture != nullptr) {
        replay_capture->captured = passed;
        replay_capture->r52 = std::move(capture);
        replay_capture->decomposition = std::move(decomposition);
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_operator_consistency_decomposition_controls() {
    return run_al_operator_consistency_decomposition_controls_impl(nullptr);
}
