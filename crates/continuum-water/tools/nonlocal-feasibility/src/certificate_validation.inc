namespace {

constexpr const char* B4E2D7R19R60_IDENTITY_SHA256 =
    "15313184ff0f552c9731da249d135d1075ead0334988bf9834a0bf72e3ce42ca";
constexpr const char* B4E2D7R19R60_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r60-certificate-validation|v1|parent=e2553f04:495bbd213784c4e4f7500c33e2be32c05d41b859145282770364a8fc3a143a4b:e97d68233ce067e7fb948d6cd38bf79ecafb403e16961fadb9a3e9bcb0ce071d:ROW_LOCAL_ENCLOSURE_CERTIFICATE_CANDIDATE|source=r59-r58-state-a4343378e4055fdc06ccb134dd44f3322473c93138960ff31965f27e8fd22af2;witness-040cc9f0dc57f543c8c3f9e1324b2e68dcc827e120782f9813a7aa0652d884bd;upper-9e8ad22488a6a49f66f7d2734c9573692a807cbfdc6c3d09874b24a590151ff8;master-38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28;histogram-3d4a798f7d9ea0fc71816091e786778f2368d8c0ff944b45200b004fa09dabdd;comparison-5253b1c5a41fbb798cbe25df4a9c16afe663e4167b332aa9e0114c99b6de82de;degree44,113,53,5992;current234;local0;strict5992;closed234;current-max1.3678206846699582e-24-row2522-degree102;local-max-1.9354174334860891e-23-row4930-degree111|structural=claimed-degree-vector-equals-flat-offset-difference;derived-maximum-equals-neighborhood-maximum;nominal-pass;dense-pass;forced-one-row-undercount-reject;overcount-reject;decreasing-offset-reject;maximum-mismatch-reject|domain=stable-directed-row-slot-order;binary64-runtime-expression-replay;nonzero-subnormal-count;nonfinite-count;displacement,relative,jacobian,component-product,dot-partial,term,row-accumulator,absolute-sum,raw,bound,upper|oracle=fresh-full-binary128-directed-formula;binary64-owned-position,witness,radius,weight;compensated-row-fold;row-local-gamma128(16*degree+66);full-upper=full-raw+full-bound|dominance=binary128(local-binary64-upper)>=full-upper-all-rows;publish-failures;minimum-margin;worst-row-degree;roots|controls=r59-parent-bytes;source;workspace;topology;master;witness;r59-reproduction;dense-degree;binary128;comparison;work;rollback;route-precedence|routes=validation-parent-rejected;validation-source-rejected;validation-workspace-rejected;validation-topology-rejected;validation-master-rejected;validation-witness-rejected;validation-r59-reproduction-rejected;validation-degree-controls-rejected;validation-binary128-rejected;validation-comparison-rejected;validation-work-rejected;validation-arithmetic-domain-stronger-bound-required;validation-dominance-rejected;row-local-certificate-validation-candidate;validation-reference-retained|precedence=parent,source,workspace,topology,master,witness,r59,dense,binary128,comparison,work,domain,dominance,validation,reference|work=parent-r59-replays1;moved-workspace1;workspace-release1;row-local-scans1;dense-degree-controls5;fresh-binary128-row-traversals1;new-pairpasses0;new-quad-row-passes1;new-density-sweeps0;new-box-blocks0;new-jvp0;new-vjp0;new-basis0;new-gram0;new-projection0;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53-r54-r55-r56-r57-r58-r59=unchanged;integration=none;certificate-authority=private-validation-only;binary128=offline-only;gamma-definition=unchanged;operator-arithmetic=unchanged;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;capacity-change=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-row-local-certificate-validation-only)IDENTITY";

bool al_r60_validate_degrees(const std::vector<std::uint32_t>& offsets,
    const std::vector<std::size_t>& claimed, std::size_t declared_maximum) {
    if (offsets.size() != claimed.size() + 1U || claimed.empty()) return false;
    std::size_t observed_maximum = 0U;
    for (std::size_t row = 0U; row < claimed.size(); ++row) {
        if (offsets[row] > offsets[row + 1U]) return false;
        const std::size_t degree = static_cast<std::size_t>(
            offsets[row + 1U] - offsets[row]);
        if (claimed[row] != degree) return false;
        observed_maximum = std::max(observed_maximum, degree);
    }
    return observed_maximum == declared_maximum;
}

struct ALR60DegreeControls {
    bool nominal = false;
    bool dense_valid = false;
    bool undercount_rejected = false;
    bool overcount_rejected = false;
    bool decreasing_rejected = false;
    bool maximum_mismatch_rejected = false;
    std::size_t dense_cases = 0U;
    std::string root;
};

ALR60DegreeControls al_r60_degree_controls(
    const JointNeighborhood& neighborhood) {
    ALR60DegreeControls result;
    std::vector<std::size_t> nominal(neighborhood.fluid.size());
    for (std::size_t row = 0U; row < nominal.size(); ++row)
        nominal[row] = static_cast<std::size_t>(
            neighborhood.flat_offsets[row + 1U]
            - neighborhood.flat_offsets[row]);
    result.nominal = al_r60_validate_degrees(
        neighborhood.flat_offsets, nominal, neighborhood.maximum_degree);

    const std::vector<std::uint32_t> offsets{0U, 2U, 3U, 6U};
    const std::vector<std::size_t> valid{2U, 1U, 3U};
    result.dense_valid = al_r60_validate_degrees(offsets, valid, 3U);
    ++result.dense_cases;
    std::vector<std::size_t> undercount = valid;
    --undercount[2U];
    result.undercount_rejected =
        !al_r60_validate_degrees(offsets, undercount, 3U);
    ++result.dense_cases;
    std::vector<std::size_t> overcount = valid;
    ++overcount[0U];
    result.overcount_rejected =
        !al_r60_validate_degrees(offsets, overcount, 3U);
    ++result.dense_cases;
    std::vector<std::uint32_t> decreasing = offsets;
    decreasing[2U] = 1U;
    result.decreasing_rejected =
        !al_r60_validate_degrees(decreasing, valid, 3U);
    ++result.dense_cases;
    result.maximum_mismatch_rejected =
        !al_r60_validate_degrees(offsets, valid, 2U);
    ++result.dense_cases;

    std::ostringstream material;
    material << result.nominal << ':' << result.dense_valid << ':'
             << result.undercount_rejected << ':'
             << result.overcount_rejected << ':'
             << result.decreasing_rejected << ':'
             << result.maximum_mismatch_rejected << ':' << result.dense_cases;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR60Validation {
    bool runtime_replay_exact = false;
    bool binary128_exact = false;
    bool comparison_exact = false;
    std::size_t row_scans = 0U;
    std::size_t quad_row_passes = 0U;
    std::size_t subnormal_intermediates = 0U;
    std::size_t nonfinite_intermediates = 0U;
    std::size_t dominance_failures = 0U;
    Binary128 minimum_margin = HUGE_VALQ;
    std::size_t minimum_margin_row = 0U;
    std::size_t minimum_margin_degree = 0U;
    Binary128 full_upper_maximum = -HUGE_VALQ;
    std::size_t full_upper_maximum_row = 0U;
    std::string full_root;
    std::string comparison_root;
};

void al_r60_observe(double value, ALR60Validation* result) {
    if (result == nullptr) return;
    if (!std::isfinite(value)) {
        ++result->nonfinite_intermediates;
    } else if (value != 0.0 && std::fpclassify(value) == FP_SUBNORMAL) {
        ++result->subnormal_intermediates;
    }
}

ALR60Validation al_r60_validate_arithmetic(
    const ALNormalizedWorkspace& workspace, const std::vector<Vec3>& witness,
    const ALR50Audit& audit) {
    ALR60Validation result;
    const std::size_t count = workspace.neighborhood.fluid.size();
    if (!workspace.passed || witness.size() != count || !audit.passed
        || audit.image.value.size() != count
        || audit.image.absolute_sum.size() != count
        || audit.upper.size() != count) return result;

    std::vector<Binary128Accumulator> full_fold(count);
    std::vector<Binary128Accumulator> full_absolute(count);
    bool runtime_exact = true;
    for (std::size_t center = 0U; center < count; ++center) {
        double runtime_sum = 0.0;
        double runtime_absolute = 0.0;
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
            const double scalar = MASS / REST_DENSITY
                * workspace.weight_first[pair_index];
            const Vec3 divided = displacement / radius;
            const Vec3 jacobian = scalar * divided;
            const Vec3 relative = participant < count
                ? witness[center] - witness[participant]
                : witness[center];
            const double product_x = jacobian.x * relative.x;
            const double product_y = jacobian.y * relative.y;
            const double product_z = jacobian.z * relative.z;
            const double dot_xy = product_x + product_y;
            const double dot_value = dot_xy + product_z;
            const double term = SPACING * dot_value;
            runtime_sum += term;
            runtime_absolute += std::abs(term);
            al_r60_observe(radius, &result);
            al_r60_observe(workspace.weight_first[pair_index], &result);
            al_r60_observe(scalar, &result);
            al_r60_observe(displacement.x, &result);
            al_r60_observe(displacement.y, &result);
            al_r60_observe(displacement.z, &result);
            al_r60_observe(divided.x, &result);
            al_r60_observe(divided.y, &result);
            al_r60_observe(divided.z, &result);
            al_r60_observe(jacobian.x, &result);
            al_r60_observe(jacobian.y, &result);
            al_r60_observe(jacobian.z, &result);
            al_r60_observe(relative.x, &result);
            al_r60_observe(relative.y, &result);
            al_r60_observe(relative.z, &result);
            al_r60_observe(product_x, &result);
            al_r60_observe(product_y, &result);
            al_r60_observe(product_z, &result);
            al_r60_observe(dot_xy, &result);
            al_r60_observe(dot_value, &result);
            al_r60_observe(term, &result);
            al_r60_observe(runtime_sum, &result);
            al_r60_observe(runtime_absolute, &result);

            const Binary128 dx = static_cast<Binary128>(center_position.x)
                - static_cast<Binary128>(participant_position.x);
            const Binary128 dy = static_cast<Binary128>(center_position.y)
                - static_cast<Binary128>(participant_position.y);
            const Binary128 dz = static_cast<Binary128>(center_position.z)
                - static_cast<Binary128>(participant_position.z);
            const Binary128 rx = participant < count
                ? static_cast<Binary128>(witness[center].x)
                    - static_cast<Binary128>(witness[participant].x)
                : static_cast<Binary128>(witness[center].x);
            const Binary128 ry = participant < count
                ? static_cast<Binary128>(witness[center].y)
                    - static_cast<Binary128>(witness[participant].y)
                : static_cast<Binary128>(witness[center].y);
            const Binary128 rz = participant < count
                ? static_cast<Binary128>(witness[center].z)
                    - static_cast<Binary128>(witness[participant].z)
                : static_cast<Binary128>(witness[center].z);
            const Binary128 scale = static_cast<Binary128>(MASS)
                / static_cast<Binary128>(REST_DENSITY)
                * static_cast<Binary128>(workspace.weight_first[pair_index])
                / static_cast<Binary128>(radius);
            const Binary128 full_term = static_cast<Binary128>(SPACING)
                * scale * (dx * rx + dy * ry + dz * rz);
            full_fold[center].add(full_term);
            full_absolute[center].add(fabsq(full_term));
        }
        runtime_exact = runtime_exact
            && binary64_bits(runtime_sum)
                == binary64_bits(audit.image.value[center])
            && binary64_bits(runtime_absolute)
                == binary64_bits(audit.image.absolute_sum[center]);
    }
    ++result.quad_row_passes;

    std::ostringstream full_material;
    std::ostringstream comparison_material;
    bool finite = true;
    for (std::size_t row = 0U; row < count; ++row) {
        const std::size_t degree = static_cast<std::size_t>(
            workspace.neighborhood.flat_offsets[row + 1U]
            - workspace.neighborhood.flat_offsets[row]);
        const double raw = workspace.support.constraint[row]
            + audit.image.value[row];
        const double local_bound = gamma_factor(16U * degree + 66U)
            * (std::abs(workspace.support.constraint[row])
                + audit.image.absolute_sum[row]);
        const double local_upper = raw + local_bound;
        al_r60_observe(workspace.support.constraint[row], &result);
        al_r60_observe(raw, &result);
        al_r60_observe(local_bound, &result);
        al_r60_observe(local_upper, &result);

        const Binary128 c = static_cast<Binary128>(
            workspace.support.constraint[row]);
        const Binary128 full_raw = c + full_fold[row].value();
        const Binary128 full_bound = al_r53_gamma(16U * degree + 66U)
            * (fabsq(c) + full_absolute[row].value());
        const Binary128 full_upper = full_raw + full_bound;
        const Binary128 margin = static_cast<Binary128>(local_upper)
            - full_upper;
        if (margin < static_cast<Binary128>(0.0))
            ++result.dominance_failures;
        if (margin < result.minimum_margin) {
            result.minimum_margin = margin;
            result.minimum_margin_row = row;
            result.minimum_margin_degree = degree;
        }
        if (full_upper > result.full_upper_maximum) {
            result.full_upper_maximum = full_upper;
            result.full_upper_maximum_row = row;
        }
        finite = finite && finiteq(full_raw) != 0 && finiteq(full_bound) != 0
            && finiteq(full_upper) != 0 && finiteq(margin) != 0;
        full_material << al_binary128_hex(full_raw) << ':'
                      << al_binary128_hex(full_bound) << ':'
                      << al_binary128_hex(full_upper) << ';';
        comparison_material << row << ':' << degree << ':'
            << binary64_bits(local_upper) << ':'
            << al_binary128_hex(full_upper) << ':'
            << al_binary128_hex(margin) << ';';
    }
    ++result.row_scans;
    result.full_root = sha256_hex(full_material.str());
    result.comparison_root = sha256_hex(comparison_material.str());
    result.runtime_replay_exact = runtime_exact;
    result.binary128_exact = runtime_exact && finite
        && result.quad_row_passes == 1U;
    result.comparison_exact = result.binary128_exact
        && result.row_scans == 1U
        && result.minimum_margin_row < count
        && result.full_upper_maximum_row < count;
    return result;
}

std::string al_r60_route(bool parent, bool source, bool workspace,
    bool topology, bool master, bool witness, bool r59, bool degree,
    bool binary128, bool comparison, bool work, bool domain,
    bool dominance, bool validation) {
    if (!parent) return "VALIDATION_PARENT_REJECTED";
    if (!source) return "VALIDATION_SOURCE_REJECTED";
    if (!workspace) return "VALIDATION_WORKSPACE_REJECTED";
    if (!topology) return "VALIDATION_TOPOLOGY_REJECTED";
    if (!master) return "VALIDATION_MASTER_REJECTED";
    if (!witness) return "VALIDATION_WITNESS_REJECTED";
    if (!r59) return "VALIDATION_R59_REPRODUCTION_REJECTED";
    if (!degree) return "VALIDATION_DEGREE_CONTROLS_REJECTED";
    if (!binary128) return "VALIDATION_BINARY128_REJECTED";
    if (!comparison) return "VALIDATION_COMPARISON_REJECTED";
    if (!work) return "VALIDATION_WORK_REJECTED";
    if (domain)
        return "VALIDATION_ARITHMETIC_DOMAIN_STRONGER_BOUND_REQUIRED";
    if (dominance) return "VALIDATION_DOMINANCE_REJECTED";
    if (validation) return "ROW_LOCAL_CERTIFICATE_VALIDATION_CANDIDATE";
    return "VALIDATION_REFERENCE_RETAINED";
}

} // namespace

SplitBoundaryReport run_al_certificate_validation_controls() {
    const bool identity = sha256_hex(B4E2D7R19R60_IDENTITY_PROJECTION)
        == B4E2D7R19R60_IDENTITY_SHA256;
    ALR59ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_row_local_enclosure_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "495bbd213784c4e4f7500c33e2be32c05d41b859145282770364a8fc3a143a4b"
        && parent.json.find(
            "e97d68233ce067e7fb948d6cd38bf79ecafb403e16961fadb9a3e9bcb0ce071d")
            != std::string::npos
        && parent.json.find("ROW_LOCAL_ENCLOSURE_CERTIFICATE_CANDIDATE")
            != std::string::npos;

    ALR58ReplayCapture& r58_capture = capture.r58;
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
        && capture.scan.histogram_root
            == "3d4a798f7d9ea0fc71816091e786778f2368d8c0ff944b45200b004fa09dabdd"
        && capture.scan.comparison_root
            == "5253b1c5a41fbb798cbe25df4a9c16afe663e4167b332aa9e0114c99b6de82de"
        && capture.scan.minimum_degree == 44U
        && capture.scan.maximum_degree == 113U
        && capture.scan.distinct_degrees == 53U
        && capture.scan.rows_below_maximum == 5992U
        && capture.scan.current_positive == 234U
        && capture.scan.local_positive == 0U
        && capture.scan.strict_improved_rows == 5992U
        && capture.scan.closed_current_positive == 234U
        && binary64_bits(capture.scan.current_maximum)
            == binary64_bits(1.3678206846699582e-24)
        && capture.scan.current_maximum_row == 2522U
        && capture.scan.current_maximum_degree == 102U
        && binary64_bits(capture.scan.local_maximum)
            == binary64_bits(-1.9354174334860891e-23)
        && capture.scan.local_maximum_row == 4930U
        && capture.scan.local_maximum_degree == 111U;

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

    ALR59Scan reproduced;
    ALR60DegreeControls degree_controls;
    ALR60Validation validation;
    if (witness_exact) {
        reproduced = al_r59_scan(moved, r58.terminal);
        degree_controls = al_r60_degree_controls(moved.neighborhood);
        validation = al_r60_validate_arithmetic(
            moved, r58.witness, r58.terminal);
    }
    if (moved.passed) release_al_normalized_workspace(moved, &workspace_work);

    const bool r59_exact = reproduced.comparison_exact
        && reproduced.histogram_root == capture.scan.histogram_root
        && reproduced.comparison_root == capture.scan.comparison_root
        && reproduced.current_active_root == capture.scan.current_active_root
        && reproduced.local_active_root == capture.scan.local_active_root
        && reproduced.current_positive == capture.scan.current_positive
        && reproduced.local_positive == capture.scan.local_positive
        && binary64_bits(reproduced.current_maximum)
            == binary64_bits(capture.scan.current_maximum)
        && binary64_bits(reproduced.local_maximum)
            == binary64_bits(capture.scan.local_maximum);
    const bool degrees_exact = degree_controls.nominal
        && degree_controls.dense_valid
        && degree_controls.undercount_rejected
        && degree_controls.overcount_rejected
        && degree_controls.decreasing_rejected
        && degree_controls.maximum_mismatch_rejected
        && degree_controls.dense_cases == 5U;
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
        && reproduced.row_scans == 1U
        && degree_controls.dense_cases == 5U
        && validation.row_scans == 1U
        && validation.quad_row_passes == 1U;
    const bool rollback =
        position_before == al_binary64_vec3_root(r30.position)
        && trial_before == al_binary64_vec3_root(r43.projected_trial)
        && owner_before == al_r29_mask_root(r43.owners)
        && superset_before == al_r42_superset_root(r43.r42.superset)
        && anchor_before == al_binary64_vec3_root(r50.witness)
        && witness_before == al_binary64_vec3_root(r58.witness)
        && upper_before == al_r10_dual_root(r58.terminal.upper);

    const bool domain_issue = validation.subnormal_intermediates > 0U
        || validation.nonfinite_intermediates > 0U;
    const bool dominance_issue = !domain_issue
        && validation.dominance_failures > 0U;
    const bool validation_candidate = !domain_issue && !dominance_issue
        && validation.comparison_exact && reproduced.local_positive == 0U;

    const std::array<const char*, 15U> expected_routes{{
        "VALIDATION_PARENT_REJECTED", "VALIDATION_SOURCE_REJECTED",
        "VALIDATION_WORKSPACE_REJECTED", "VALIDATION_TOPOLOGY_REJECTED",
        "VALIDATION_MASTER_REJECTED", "VALIDATION_WITNESS_REJECTED",
        "VALIDATION_R59_REPRODUCTION_REJECTED",
        "VALIDATION_DEGREE_CONTROLS_REJECTED",
        "VALIDATION_BINARY128_REJECTED", "VALIDATION_COMPARISON_REJECTED",
        "VALIDATION_WORK_REJECTED",
        "VALIDATION_ARITHMETIC_DOMAIN_STRONGER_BOUND_REQUIRED",
        "VALIDATION_DOMINANCE_REJECTED",
        "ROW_LOCAL_CERTIFICATE_VALIDATION_CANDIDATE",
        "VALIDATION_REFERENCE_RETAINED"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t route_index = 0U;
         route_index < expected_routes.size(); ++route_index) {
        std::array<bool, 11U> gates{};
        gates.fill(true);
        if (route_index < gates.size()) gates[route_index] = false;
        const std::string observed = al_r60_route(
            gates[0], gates[1], gates[2], gates[3], gates[4], gates[5],
            gates[6], gates[7], gates[8], gates[9], gates[10],
            route_index == 11U, route_index == 12U, route_index == 13U);
        routes_exact = routes_exact && observed == expected_routes[route_index];
        route_material << expected_routes[route_index] << ':' << observed << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r60_route(parent_exact, source_exact,
        workspace_exact, topology_exact, master_exact, witness_exact,
        r59_exact, degrees_exact, validation.binary128_exact,
        validation.comparison_exact, work_exact && rollback, domain_issue,
        dominance_issue, validation_candidate);
    const bool passed = identity && parent_exact && source_exact
        && workspace_exact && topology_exact && master_exact && witness_exact
        && r59_exact && degrees_exact && validation.binary128_exact
        && validation.comparison_exact && work_exact && rollback && routes_exact;

    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R60_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << witness_before << ':' << upper_before << ':'
             << capture.scan.histogram_root << ':'
             << capture.scan.comparison_root << '|'
             << degree_controls.root << ':' << degree_controls.dense_cases
             << ':' << degrees_exact << '|'
             << validation.runtime_replay_exact << ':'
             << validation.subnormal_intermediates << ':'
             << validation.nonfinite_intermediates << ':'
             << validation.dominance_failures << ':'
             << al_binary128_hex(validation.minimum_margin) << ':'
             << validation.minimum_margin_row << ':'
             << validation.minimum_margin_degree << ':'
             << al_binary128_hex(validation.full_upper_maximum) << ':'
             << validation.full_upper_maximum_row << '|'
             << validation.full_root << ':' << validation.comparison_root << '|'
             << validation.row_scans << ':'
             << validation.quad_row_passes << ':' << domain_issue << ':'
             << dominance_issue << ':' << validation_candidate << ':'
             << work_exact << ':' << rollback << ':' << route_root << ':'
             << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r60_certificate_validation.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R60_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source\":{\"state_root\":\"" << r58.state_root
           << "\",\"witness_root\":\"" << witness_before
           << "\",\"upper_root\":\"" << upper_before
           << "\",\"histogram_root\":\"" << capture.scan.histogram_root
           << "\",\"comparison_root\":\"" << capture.scan.comparison_root
           << "\"}"
           << ",\"degree_controls\":{\"nominal\":"
           << (degree_controls.nominal ? "true" : "false")
           << ",\"dense_valid\":"
           << (degree_controls.dense_valid ? "true" : "false")
           << ",\"undercount_rejected\":"
           << (degree_controls.undercount_rejected ? "true" : "false")
           << ",\"overcount_rejected\":"
           << (degree_controls.overcount_rejected ? "true" : "false")
           << ",\"decreasing_rejected\":"
           << (degree_controls.decreasing_rejected ? "true" : "false")
           << ",\"maximum_mismatch_rejected\":"
           << (degree_controls.maximum_mismatch_rejected ? "true" : "false")
           << ",\"dense_cases\":" << degree_controls.dense_cases
           << ",\"root\":\"" << degree_controls.root << "\"}"
           << ",\"arithmetic_domain\":{\"runtime_replay_exact\":"
           << (validation.runtime_replay_exact ? "true" : "false")
           << ",\"subnormal_intermediates\":"
           << validation.subnormal_intermediates
           << ",\"nonfinite_intermediates\":"
           << validation.nonfinite_intermediates << "}"
           << ",\"dominance\":{\"failures\":"
           << validation.dominance_failures << ",\"minimum_margin\":"
           << static_cast<double>(validation.minimum_margin)
           << ",\"minimum_margin_hex\":\""
           << al_binary128_hex(validation.minimum_margin)
           << "\",\"minimum_margin_row\":"
           << validation.minimum_margin_row
           << ",\"minimum_margin_degree\":"
           << validation.minimum_margin_degree
           << ",\"full_upper_maximum\":"
           << static_cast<double>(validation.full_upper_maximum)
           << ",\"full_upper_maximum_row\":"
           << validation.full_upper_maximum_row << "}"
           << ",\"roots\":{\"full\":\"" << validation.full_root
           << "\",\"comparison\":\"" << validation.comparison_root
           << "\"}"
           << ",\"work\":{\"row_local_scans\":"
           << reproduced.row_scans << ",\"dense_degree_controls\":"
           << degree_controls.dense_cases
           << ",\"binary128_row_traversals\":"
           << validation.quad_row_passes
           << ",\"new_pair_passes\":0,\"workspace_builds\":"
           << workspace_work.workspace_builds
           << ",\"workspace_releases\":"
           << workspace_work.workspace_releases
           << ",\"exact\":" << (work_exact ? "true" : "false") << "}"
           << ",\"route_cases\":" << expected_routes.size()
           << ",\"route_root\":\"" << route_root
           << "\",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback ? "true" : "false")
           << ",\"route\":\"" << route
           << "\",\"certificate_integrated\":false"
              ",\"binary128_offline_only\":true"
              ",\"gamma_definition_unchanged\":true"
              ",\"operator_arithmetic_unchanged\":true"
              ",\"normal_witness_applied\":false"
              ",\"r43_restoration_committed\":false"
              ",\"restoration_exit_authorized\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    return {passed, report.str()};
}
