namespace {

constexpr const char* B4E2D7R19R57_IDENTITY_SHA256 =
    "2a41c16b73c553a966c2e73f2a435a70f99a6d1eb766f359e78e247da69b5d9e";
constexpr const char* B4E2D7R19R57_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r57-certificate-aware-refinement|v1|parent=6b346ff6:e0ececa2d57448248e86d623d4c1e15e69abe68576c503e1e802ef990b7c46aa:625f8db0a1be011dd36737743487ad9ee54e629bfb8926c0fb130120c5b9f7ca:JOINT_WITNESS_HIGH_PRECISION_RAW_RESIDUAL_CONFIRMED|source=r55-state-0edef9f324518fd77716b8fbc420a1c9b1ccd4187fef28af222a0d2ea840c1b8;cycle64-record-dd2db6e5ffe417ac6fc2b5a6e1eae884607cb8400aacd4d9f3e2959b5945bc6d;witness-19da2a335377f195bc0ce110977d19f6fa30223054d02d485a0b790123270406;correction-8ae05f144d760cfcfd3232dc0113365dff7e3a680769442cbdef1e0c6977b387;boxdual-512c77152fb3f4e8d55603cb765f540dd36cc30e18bb2465e910112a11aacb7d;master-38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28;upper-active476;raw-positive308;full128-positive308;negative5692;unresolved0;maximum-upper4.1425104588214889e-20|problem=d0;min0.5*norm2delta;cycle64-directed-upper-plus-A-delta<=0-on-master494;box=contact-lower-minus-cycle64-witness<=delta<=contact-upper-minus-cycle64-witness;normal-ball-audit-only|solver=zero-initialized-grouped-box-dykstra;density-lambda0;box-correction0;delta0;stable-master-order;fresh-residual-after-box|depth=64;checkpoints8,16,32,64;no-tolerance-stop;first-certified-else64|oracle=unchanged-directed-audit-each-checkpoint;selected-fresh-pair-once-binary64;selected-compensated-binary128-directed-traversal;frozen-coefficient-binary128-recompute;current-gamma-unchanged|rows=checkpoint-psi-h-maximum-upper-raw-directed-outside-box-ball-roots;selected-full128-positive-negative-unresolved;bound-only;maxima;roots|controls=r56-parent-bytes;source;workspace;topology;master;geometry;dense-refinement;dykstra;box;ball;primal;binary128;selection;work;rollback;route-precedence|routes=refinement-parent-rejected;refinement-source-rejected;refinement-workspace-rejected;refinement-topology-rejected;refinement-master-rejected;refinement-geometry-rejected;refinement-dykstra-rejected;refinement-box-rejected;refinement-ball-rejected;refinement-primal-audit-rejected;refinement-binary128-rejected;refinement-selection-rejected;refinement-work-rejected;certificate-refinement-master-expansion-required;certificate-refinement-ball-integration-required;restoration-next-trqp-compatible-candidate;certificate-refinement-solver-polish-required;certificate-refinement-sign-unresolved;certificate-refinement-enclosure-fixed-point-required;certificate-refinement-reference-retained|precedence=parent,source,workspace,topology,master,geometry,dykstra,box,ball,primal,binary128,selection,work,outside-master,ball-integration,certified,raw-positive,unresolved,enclosure,reference|work=parent-r56-replays1;moved-workspace1;workspace-release1;density-sweeps64;box-blocks64;fresh-residual-jvp64;checkpoint-directed-jvp4;selected-fresh-pair-jvp1;binary128-row-traversal1;new-pairpasses69;quad-row-passes1;new-row-vjp0;new-gram-jvp0;new-basis0;new-projection0;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53-r54-r55-r56=unchanged;certificate=current-directed-unchanged;binary128=offline-only;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;gamma-change=none;capacity-change=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-certificate-aware-refinement-classification-only)IDENTITY";

constexpr std::array<std::size_t, 4U> AL_R57_CHECKPOINTS{{8U, 16U, 32U, 64U}};

struct ALR57Record {
    bool passed = false;
    bool box_exact = false;
    bool ball_exact = false;
    bool certified = false;
    std::size_t cycle = 0U;
    std::size_t raw_outside_master = 0U;
    std::size_t directed_outside_master = 0U;
    double before_box_maximum = std::numeric_limits<double>::infinity();
    double after_box_master_maximum = std::numeric_limits<double>::infinity();
    double witness_norm = 0.0;
    double correction_norm = 0.0;
    double box_dual_norm = 0.0;
    ALR50Audit audit;
    std::vector<Vec3> witness;
    std::vector<Vec3> correction;
    std::string witness_root;
    std::string correction_root;
    std::string box_dual_root;
    std::string root;
};

struct ALR57Result {
    bool master_exact = false;
    bool geometry_exact = false;
    bool dykstra_exact = false;
    bool box_exact = false;
    bool ball_exact = false;
    bool primal_exact = false;
    bool selection_exact = false;
    bool contraction = false;
    bool certified = false;
    bool any_outside_master = false;
    bool any_ball_outside = false;
    std::size_t selected_index = 0U;
    std::size_t density_sweeps = 0U;
    std::size_t box_blocks = 0U;
    std::size_t residual_jvp = 0U;
    std::size_t audit_jvp = 0U;
    std::size_t pair_passes = 0U;
    std::size_t total_box_changes = 0U;
    std::vector<ALR57Record> records;
    std::vector<Vec3> witness;
    ALR50Audit terminal;
    std::string state_root;
};

bool al_r57_checkpoint(std::size_t cycle) {
    return std::find(AL_R57_CHECKPOINTS.begin(), AL_R57_CHECKPOINTS.end(), cycle)
        != AL_R57_CHECKPOINTS.end();
}

ALR57Result al_r57_generate(const ALNormalizedWorkspace& workspace,
    const std::vector<Vec3>& source_witness, const ALR50Audit& source_audit,
    const ALR51MasterResult& r51, const std::vector<Vec3>& absolute_lower,
    const std::vector<Vec3>& absolute_upper) {
    ALR57Result result;
    const std::size_t row_count = workspace.support.constraint.size();
    const std::size_t particle_count = workspace.neighborhood.fluid.size();
    const std::size_t master_count = r51.rows.size();
    result.master_exact = workspace.passed && source_audit.passed
        && source_audit.upper.size() == row_count && master_count == 494U
        && r51.cache.size() == master_count
        && std::is_sorted(r51.rows.begin(), r51.rows.end())
        && al_r51_cache_root(r51.cache) == r51.extended_cache_root;
    if (!result.master_exact) return result;

    std::vector<std::size_t> basis_index(master_count);
    for (std::size_t i = 0U; i < master_count; ++i) {
        basis_index[i] = al_r50_find_basis(r51.cache, r51.rows[i]);
        result.master_exact = result.master_exact
            && basis_index[i] < r51.cache.size();
    }
    if (!result.master_exact) return result;

    std::vector<Vec3> correction_lower(particle_count);
    std::vector<Vec3> correction_upper(particle_count);
    result.geometry_exact = source_witness.size() == particle_count
        && absolute_lower.size() == particle_count
        && absolute_upper.size() == particle_count;
    for (std::size_t i = 0U; i < particle_count && result.geometry_exact; ++i)
        for (int axis = 0; axis < 3; ++axis) {
            const double anchor = al_r41_component(source_witness[i], axis);
            const double low = al_r41_component(absolute_lower[i], axis) - anchor;
            const double high = al_r41_component(absolute_upper[i], axis) - anchor;
            al_r43_component(correction_lower[i], axis) = low;
            al_r43_component(correction_upper[i], axis) = high;
            result.geometry_exact = result.geometry_exact
                && std::isfinite(low) && std::isfinite(high)
                && low <= 0.0 && high >= 0.0;
        }
    if (!result.geometry_exact) return result;

    std::vector<Vec3> correction(particle_count);
    std::vector<Vec3> box_dual(particle_count);
    std::vector<double> lambda(master_count, 0.0);
    std::vector<double> residual(master_count);
    for (std::size_t j = 0U; j < master_count; ++j)
        residual[j] = source_audit.upper[r51.rows[j]];
    std::vector<std::uint8_t> master_mask(row_count, 0U);
    for (std::size_t row : r51.rows) master_mask[row] = 1U;

    bool exact = true;
    for (std::size_t cycle = 1U; cycle <= 64U && exact; ++cycle) {
        for (std::size_t i = 0U; i < master_count; ++i) {
            const ALR50RowBasis& basis = r51.cache[basis_index[i]];
            const double next = std::max(
                0.0, lambda[i] + residual[i] / basis.diagonal);
            const double delta = next - lambda[i];
            if (!std::isfinite(next) || !std::isfinite(delta)) {
                exact = false;
                break;
            }
            lambda[i] = next;
            if (delta == 0.0) continue;
            for (std::size_t particle = 0U; particle < particle_count; ++particle)
                correction[particle] = correction[particle]
                    - delta * basis.gradient[particle];
            for (std::size_t j = 0U; j < master_count; ++j)
                residual[j] -= delta * basis.gram_column[r51.rows[j]];
        }
        if (!exact || !al_r48_vec_finite(correction)
            || !al_r30_finite(residual)) break;
        ++result.density_sweeps;
        const double before_box =
            *std::max_element(residual.begin(), residual.end());

        std::size_t box_changes = 0U;
        for (std::size_t i = 0U; i < particle_count; ++i)
            for (int axis = 0; axis < 3; ++axis) {
                const double z = al_r41_component(correction[i], axis)
                    + al_r41_component(box_dual[i], axis);
                const double clipped = std::max(
                    al_r41_component(correction_lower[i], axis),
                    std::min(z, al_r41_component(correction_upper[i], axis)));
                if (binary64_bits(clipped) != binary64_bits(z)) ++box_changes;
                al_r43_component(correction[i], axis) = clipped;
                al_r43_component(box_dual[i], axis) = z - clipped;
            }
        ++result.box_blocks;
        result.total_box_changes += box_changes;
        exact = exact && al_r48_vec_finite(correction)
            && al_r48_vec_finite(box_dual);
        if (!exact) break;

        const ALLinearizedJvp refreshed =
            al_r29_pair_once_jvp(workspace, correction);
        ++result.residual_jvp;
        if (!refreshed.passed || refreshed.value.size() != row_count) {
            exact = false;
            break;
        }
        for (std::size_t j = 0U; j < master_count; ++j)
            residual[j] = source_audit.upper[r51.rows[j]]
                + refreshed.value[r51.rows[j]];

        if (!al_r57_checkpoint(cycle)) continue;
        ALR57Record record;
        record.cycle = cycle;
        record.before_box_maximum = before_box;
        record.after_box_master_maximum =
            *std::max_element(residual.begin(), residual.end());
        record.witness.resize(particle_count);
        for (std::size_t i = 0U; i < particle_count; ++i)
            record.witness[i] = source_witness[i] + correction[i];
        record.correction = correction;
        record.box_exact =
            al_r49_feasible(record.witness, absolute_lower, absolute_upper);
        record.witness_norm = al_r32_vec3_norm(record.witness);
        record.ball_exact = std::isfinite(record.witness_norm)
            && record.witness_norm <= AL_R48_NORMAL_RADIUS;
        record.correction_norm = al_r32_vec3_norm(correction);
        record.box_dual_norm = al_r32_vec3_norm(box_dual);
        record.audit = al_r50_audit(workspace, record.witness);
        ++result.audit_jvp;
        record.certified = record.audit.passed && record.audit.active.empty();
        for (std::size_t row = 0U; row < row_count; ++row)
            if (master_mask[row] == 0U
                && workspace.support.constraint[row]
                    + record.audit.image.value[row] > 0.0)
                ++record.raw_outside_master;
        for (std::size_t row : record.audit.active)
            if (master_mask[row] == 0U) ++record.directed_outside_master;
        record.witness_root = al_binary64_vec3_root(record.witness);
        record.correction_root = al_binary64_vec3_root(correction);
        record.box_dual_root = al_binary64_vec3_root(box_dual);
        std::ostringstream material;
        material << std::setprecision(17) << cycle << ':'
                 << record.before_box_maximum << ':'
                 << record.after_box_master_maximum << ':'
                 << record.audit.primal.psi << ':' << record.audit.primal.h
                 << ':' << record.audit.primal.maximum_upper << ':'
                 << record.audit.primal.raw_active << ':'
                 << record.audit.active.size() << ':'
                 << record.raw_outside_master << ':'
                 << record.directed_outside_master << ':'
                 << record.box_exact << ':' << record.ball_exact << ':'
                 << record.certified << ':' << record.witness_norm << ':'
                 << record.correction_norm << ':' << record.box_dual_norm << ':'
                 << record.witness_root << ':' << record.correction_root << ':'
                 << record.box_dual_root;
        record.root = sha256_hex(material.str());
        record.passed = record.audit.passed && record.box_exact
            && std::isfinite(record.correction_norm)
            && std::isfinite(record.box_dual_norm);
        result.records.push_back(std::move(record));
    }

    result.dykstra_exact = exact && result.density_sweeps == 64U
        && result.box_blocks == 64U && result.residual_jvp == 64U
        && result.records.size() == AL_R57_CHECKPOINTS.size();
    result.box_exact = result.dykstra_exact
        && std::all_of(result.records.begin(), result.records.end(),
            [](const ALR57Record& record) {
                return record.passed && record.box_exact;
            });
    result.ball_exact = result.box_exact
        && std::all_of(result.records.begin(), result.records.end(),
            [](const ALR57Record& record) {
                return std::isfinite(record.witness_norm);
            });
    result.primal_exact = result.ball_exact
        && result.audit_jvp == AL_R57_CHECKPOINTS.size();
    if (!result.primal_exact) return result;

    result.contraction = true;
    double previous = std::numeric_limits<double>::infinity();
    for (const ALR57Record& record : result.records) {
        result.contraction = result.contraction
            && record.after_box_master_maximum < previous;
        previous = record.after_box_master_maximum;
        result.any_outside_master = result.any_outside_master
            || record.raw_outside_master > 0U
            || record.directed_outside_master > 0U;
        result.any_ball_outside = result.any_ball_outside || !record.ball_exact;
    }
    result.selected_index = result.records.size() - 1U;
    for (std::size_t i = 0U; i < result.records.size(); ++i)
        if (result.records[i].certified) {
            result.certified = true;
            result.selected_index = i;
            break;
        }
    result.witness = result.records[result.selected_index].witness;
    result.terminal = result.records[result.selected_index].audit;
    result.selection_exact = result.terminal.passed
        && result.witness.size() == particle_count
        && result.records[result.selected_index].passed
        && (!result.certified || result.terminal.active.empty());
    result.pair_passes = result.residual_jvp + result.audit_jvp;
    std::ostringstream state;
    state << result.contraction << ':' << result.certified << ':'
          << result.selected_index << ':' << result.total_box_changes << ':';
    for (const ALR57Record& record : result.records) state << record.root << ':';
    result.state_root = sha256_hex(state.str());
    return result;
}

std::string al_r57_route(bool parent, bool source, bool workspace,
    bool topology, bool master, bool geometry, bool dykstra, bool box,
    bool ball, bool primal, bool binary128, bool selection, bool work,
    bool outside_master, bool ball_integration, bool certified,
    bool raw_positive, bool unresolved, bool enclosure) {
    if (!parent) return "REFINEMENT_PARENT_REJECTED";
    if (!source) return "REFINEMENT_SOURCE_REJECTED";
    if (!workspace) return "REFINEMENT_WORKSPACE_REJECTED";
    if (!topology) return "REFINEMENT_TOPOLOGY_REJECTED";
    if (!master) return "REFINEMENT_MASTER_REJECTED";
    if (!geometry) return "REFINEMENT_GEOMETRY_REJECTED";
    if (!dykstra) return "REFINEMENT_DYKSTRA_REJECTED";
    if (!box) return "REFINEMENT_BOX_REJECTED";
    if (!ball) return "REFINEMENT_BALL_REJECTED";
    if (!primal) return "REFINEMENT_PRIMAL_AUDIT_REJECTED";
    if (!binary128) return "REFINEMENT_BINARY128_REJECTED";
    if (!selection) return "REFINEMENT_SELECTION_REJECTED";
    if (!work) return "REFINEMENT_WORK_REJECTED";
    if (outside_master)
        return "CERTIFICATE_REFINEMENT_MASTER_EXPANSION_REQUIRED";
    if (ball_integration)
        return "CERTIFICATE_REFINEMENT_BALL_INTEGRATION_REQUIRED";
    if (certified) return "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE";
    if (raw_positive)
        return "CERTIFICATE_REFINEMENT_SOLVER_POLISH_REQUIRED";
    if (unresolved) return "CERTIFICATE_REFINEMENT_SIGN_UNRESOLVED";
    if (enclosure)
        return "CERTIFICATE_REFINEMENT_ENCLOSURE_FIXED_POINT_REQUIRED";
    return "CERTIFICATE_REFINEMENT_REFERENCE_RETAINED";
}

struct ALR57ReplayCapture {
    bool captured = false;
    ALR56ReplayCapture r56;
    ALR57Result result;
    ALR53Result decomposition;
};

} // namespace

SplitBoundaryReport run_al_certificate_aware_refinement_controls_impl(
    ALR57ReplayCapture* replay_capture) {
    const bool identity = sha256_hex(B4E2D7R19R57_IDENTITY_PROJECTION)
        == B4E2D7R19R57_IDENTITY_SHA256;
    ALR56ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_joint_witness_certificate_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "e0ececa2d57448248e86d623d4c1e15e69abe68576c503e1e802ef990b7c46aa"
        && parent.json.find(
            "625f8db0a1be011dd36737743487ad9ee54e629bfb8926c0fb130120c5b9f7ca")
            != std::string::npos
        && parent.json.find(
            "JOINT_WITNESS_HIGH_PRECISION_RAW_RESIDUAL_CONFIRMED")
            != std::string::npos;

    ALR55ReplayCapture& r55_capture = capture.r55;
    ALR55Result& r55 = r55_capture.result;
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
    const std::string witness_before =
        al_binary64_vec3_root(r55.diagnostic_witness);
    const std::string correction_before =
        al_binary64_vec3_root(r55.diagnostic_correction);
    const std::string box_dual_before =
        al_binary64_vec3_root(r55.diagnostic_box_dual);
    const std::string upper_before =
        al_r10_dual_root(r55.diagnostic_audit.upper);

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
        && r55.diagnostic_audit.passed
        && r55.diagnostic_audit.active.size() == 476U
        && r55.diagnostic_audit.primal.raw_active == 308U
        && binary64_bits(r55.diagnostic_audit.primal.maximum_upper)
            == binary64_bits(4.1425104588214889e-20)
        && capture.decomposition.full128_resolved_positive == 308U
        && capture.decomposition.full128_resolved_negative == 5692U
        && capture.decomposition.full128_unresolved == 0U
        && capture.decomposition.pair_full_sign_disagreements == 0U
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
        && r55.diagnostic_witness.size() == r43.projected_trial.size();
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
            const double witness = i < r55.diagnostic_witness.size()
                ? al_r41_component(r55.diagnostic_witness[i], axis)
                : 0.0;
            geometry_exact = geometry_exact && std::isfinite(low)
                && std::isfinite(high) && low <= witness && witness <= high;
        }

    ALR57Result result;
    if (geometry_exact)
        result = al_r57_generate(moved, r55.diagnostic_witness,
            r55.diagnostic_audit, r51, absolute_lower, absolute_upper);
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
        && witness_before == al_binary64_vec3_root(r55.diagnostic_witness)
        && correction_before == al_binary64_vec3_root(r55.diagnostic_correction)
        && box_dual_before == al_binary64_vec3_root(r55.diagnostic_box_dual)
        && upper_before == al_r10_dual_root(r55.diagnostic_audit.upper);

    const bool raw_positive = binary128_exact
        && decomposition.full128_resolved_positive > 0U;
    const bool unresolved = binary128_exact && !raw_positive
        && decomposition.full128_unresolved > 0U;
    const bool enclosure = result.selection_exact && !raw_positive
        && !unresolved && !result.terminal.active.empty();

    const std::array<const char*, 20U> expected_routes{{
        "REFINEMENT_PARENT_REJECTED",
        "REFINEMENT_SOURCE_REJECTED",
        "REFINEMENT_WORKSPACE_REJECTED",
        "REFINEMENT_TOPOLOGY_REJECTED",
        "REFINEMENT_MASTER_REJECTED",
        "REFINEMENT_GEOMETRY_REJECTED",
        "REFINEMENT_DYKSTRA_REJECTED",
        "REFINEMENT_BOX_REJECTED",
        "REFINEMENT_BALL_REJECTED",
        "REFINEMENT_PRIMAL_AUDIT_REJECTED",
        "REFINEMENT_BINARY128_REJECTED",
        "REFINEMENT_SELECTION_REJECTED",
        "REFINEMENT_WORK_REJECTED",
        "CERTIFICATE_REFINEMENT_MASTER_EXPANSION_REQUIRED",
        "CERTIFICATE_REFINEMENT_BALL_INTEGRATION_REQUIRED",
        "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE",
        "CERTIFICATE_REFINEMENT_SOLVER_POLISH_REQUIRED",
        "CERTIFICATE_REFINEMENT_SIGN_UNRESOLVED",
        "CERTIFICATE_REFINEMENT_ENCLOSURE_FIXED_POINT_REQUIRED",
        "CERTIFICATE_REFINEMENT_REFERENCE_RETAINED"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t route_index = 0U;
         route_index < expected_routes.size(); ++route_index) {
        std::array<bool, 13U> gates{};
        gates.fill(true);
        if (route_index < gates.size()) gates[route_index] = false;
        const std::string observed = al_r57_route(
            gates[0], gates[1], gates[2], gates[3], gates[4], gates[5],
            gates[6], gates[7], gates[8], gates[9], gates[10], gates[11],
            gates[12], route_index == 13U, route_index == 14U,
            route_index == 15U, route_index == 16U, route_index == 17U,
            route_index == 18U);
        routes_exact = routes_exact && observed == expected_routes[route_index];
        route_material << expected_routes[route_index] << ':' << observed << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r57_route(parent_exact, source_exact,
        workspace_exact, topology_exact, master_exact, geometry_exact,
        result.dykstra_exact, result.box_exact, result.ball_exact,
        result.primal_exact, binary128_exact, result.selection_exact,
        work_exact && rollback, result.any_outside_master,
        result.any_ball_outside, result.certified, raw_positive, unresolved,
        enclosure);
    const bool passed = identity && parent_exact && source_exact && dense
        && workspace_exact && topology_exact && master_exact && geometry_exact
        && result.master_exact && result.geometry_exact && result.dykstra_exact
        && result.box_exact && result.ball_exact && result.primal_exact
        && binary128_exact && result.selection_exact && work_exact && rollback
        && routes_exact;

    const ALR57Record* selected = result.selection_exact
        ? &result.records[result.selected_index]
        : nullptr;
    const std::string selected_root = selected != nullptr ? selected->root : "";
    const std::string selected_witness =
        selected != nullptr ? selected->witness_root : "";
    const std::string selected_correction =
        selected != nullptr ? selected->correction_root : "";
    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R57_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << upper_before << ':' << result.state_root << ':'
             << result.records.size() << ':' << result.contraction << ':'
             << result.certified << ':' << result.selected_index << ':'
             << result.any_outside_master << ':' << result.any_ball_outside
             << '|' << selected_root << ':' << selected_witness << ':'
             << selected_correction << '|' << decomposition.pair_positive << ':'
             << decomposition.full128_resolved_positive << ':'
             << decomposition.full128_resolved_negative << ':'
             << decomposition.full128_unresolved << ':'
             << decomposition.directed_bound_only_positive << '|'
             << decomposition.comparison_root << ':'
             << result.density_sweeps << ':' << result.box_blocks << ':'
             << result.residual_jvp << ':' << result.audit_jvp << ':'
             << decomposition.pair_passes << ':'
             << decomposition.quad_row_passes << ':' << work_exact << ':'
             << rollback << ':' << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r57_certificate_aware_refinement.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R57_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source\":{\"witness_root\":\"" << witness_before
           << "\",\"upper_root\":\"" << upper_before
           << "\",\"active\":" << r55.diagnostic_audit.active.size()
           << "},\"records\":[";
    for (std::size_t i = 0U; i < result.records.size(); ++i) {
        if (i != 0U) report << ',';
        const ALR57Record& record = result.records[i];
        report << "{\"cycle\":" << record.cycle
               << ",\"before_box_maximum\":" << record.before_box_maximum
               << ",\"after_box_master_maximum\":"
               << record.after_box_master_maximum
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
               << ",\"witness_norm\":" << record.witness_norm
               << ",\"correction_norm\":" << record.correction_norm
               << ",\"box_dual_norm\":" << record.box_dual_norm
               << ",\"witness_root\":\"" << record.witness_root
               << "\",\"correction_root\":\"" << record.correction_root
               << "\",\"box_dual_root\":\"" << record.box_dual_root
               << "\",\"root\":\"" << record.root << "\"}";
    }
    report << "],\"selection\":{\"contraction\":"
           << (result.contraction ? "true" : "false")
           << ",\"certified\":" << (result.certified ? "true" : "false")
           << ",\"selected_index\":" << result.selected_index << "}"
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
           << ",\"maximum_raw_row\":"
           << decomposition.full128_maximum.row
           << ",\"maximum_lower_hex\":\""
           << al_binary128_hex(decomposition.full128_lower_maximum.value)
           << "\",\"maximum_upper_hex\":\""
           << al_binary128_hex(decomposition.full128_upper_maximum.value)
           << "\",\"comparison_root\":\""
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
              ",\"normal_witness_applied\":false"
              ",\"r43_restoration_committed\":false"
              ",\"restoration_exit_authorized\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    if (replay_capture != nullptr) {
        replay_capture->captured = passed;
        replay_capture->r56 = std::move(capture);
        replay_capture->result = std::move(result);
        replay_capture->decomposition = std::move(decomposition);
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_certificate_aware_refinement_controls() {
    return run_al_certificate_aware_refinement_controls_impl(nullptr);
}
