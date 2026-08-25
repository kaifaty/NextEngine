namespace {

constexpr const char* B4E2D7R19R55_IDENTITY_SHA256 =
    "4195f54a1b18cbe53222b4d3a9da3c9004f401bcb9b528162b38d20b537cbe28";
constexpr const char* B4E2D7R19R55_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r55-contact-constrained-dykstra|v1|parent=76e72ba9:b2f32c0aa9e8dd689e3924d8a88db5c4c971404fbc3835bd313cf7512994edff:9d0b1b8cc165ae84463386751414fe5614f9a62af7117609cd575f4573f14e74:BALL_BOX_PROJECTION_MODEL_REQUIRED|source=r54-target-7bf5b35fa9cedb0aca615e994e1252b61c3b8a0d69db1d168a07d04f5b584203;projected-ccc1084988fb56a829649f60c292210f24723a4932874923640d5bdbf99e8729;displacement-bdb3ec8e28529c4ddfe02f5b6c6297e75e7361ac72428fed1af3f324263cce45;box-violations1125;target-positive2;projected-positive219;master-38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28;cache-f4e9368971d4b6feeb13874d0279dc73fd54ebbb523c75c5c2dd303acfc58cd5|problem=p0;min0.5*norm2p;density-upper-r50-plus-Ap<=0;box=lower-minus-r50-anchor<=p<=upper-minus-r50-anchor;normal-ball-audit-only|solver=dykstra-cyclic;sets=494-density-halfspaces-stable-master-order-then-one-box-block;lambda0;box-correction0;primal-correction0;retain-density-and-box-duals|operator=recursive-gram-inside-density-sweep;fresh-pair-once-residual-after-every-box;captured-basis-and-gram;no-post-terminal-projection|depth=single-continuation64;checkpoints8,16,32,64;no-tolerance-stop;directed-certificate-at-checkpoints|rows=master-predicted-maximum-before-box;fresh-master-upper-after-box;all-raw-positive;directed-positive;outside-master;box-feasibility;ball-norm;correction-and-box-dual-norms;roots|selection=first-certified-checkpoint;else-smallest-checkpoint-strict-r52-64-psi-h-maximum-upper-dominance;outside-master-before-candidate;ball-before-candidate;else-reference|controls=r54-parent-bytes;source;workspace;topology;master;geometry;dense-dykstra;lambda;gram;box-block;residual-refresh;checkpoints;directed-certificate;selection;work;rollback;route-precedence|routes=contact-parent-rejected;contact-source-rejected;contact-workspace-rejected;contact-topology-rejected;contact-master-rejected;contact-geometry-rejected;contact-dykstra-rejected;contact-box-rejected;contact-ball-rejected;contact-primal-audit-rejected;contact-selection-rejected;contact-work-rejected;restoration-next-trqp-compatible-candidate;contact-constrained-master-expansion-required;contact-constrained-ball-integration-required;contact-constrained-dykstra-candidate;contact-constrained-reference-retained|precedence=parent,source,workspace,topology,master,geometry,dykstra,box,ball,primal,selection,work,certificate,outside-master,ball-integration,candidate,reference|work=parent-r54-replays1;moved-workspace1;workspace-release1;captured-cache494;density-sweeps64;box-blocks64;fresh-pair-once-jvp64;checkpoint-directed-jvp4;pairpasses68;new-row-vjp0;new-gram-jvp0;new-projection0;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53-r54=unchanged;certificate=current-directed-unchanged;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;residual-replacement=none;post-projection=none;gamma-change=none;capacity-change=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-contact-constrained-dykstra-classification-only)IDENTITY";

constexpr std::array<std::size_t, 4U> AL_R55_CHECKPOINTS{{8U, 16U, 32U, 64U}};
constexpr double AL_R55_R52_PSI = 7.9325157688588131e-26;
constexpr double AL_R55_R52_H = 3.9830932122808307e-13;
constexpr double AL_R55_R52_MAXIMUM_UPPER = 7.7539986039447153e-14;

struct ALR55Record {
    bool passed = false;
    bool box_exact = false;
    bool ball_exact = false;
    bool strict = false;
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
    std::string witness_root;
    std::string correction_root;
    std::string box_dual_root;
    std::string root;
};

struct ALR55Result {
    bool master_exact = false;
    bool geometry_exact = false;
    bool dykstra_exact = false;
    bool box_exact = false;
    bool ball_exact = false;
    bool primal_exact = false;
    bool selection_exact = false;
    bool contraction = false;
    bool selected = false;
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
    std::vector<ALR55Record> records;
    std::vector<Vec3> witness;
    std::vector<Vec3> diagnostic_witness;
    std::vector<Vec3> diagnostic_correction;
    std::vector<Vec3> diagnostic_box_dual;
    ALR50Audit diagnostic_audit;
    ALR50Audit terminal;
    std::string state_root;
};

bool al_r55_checkpoint(std::size_t cycle) {
    return std::find(AL_R55_CHECKPOINTS.begin(), AL_R55_CHECKPOINTS.end(), cycle)
        != AL_R55_CHECKPOINTS.end();
}

ALR55Result al_r55_generate(const ALNormalizedWorkspace& workspace,
    const ALR50Generator& r50, const ALR51MasterResult& r51,
    const std::vector<Vec3>& lower, const std::vector<Vec3>& upper) {
    ALR55Result result;
    const std::size_t row_count = workspace.support.constraint.size();
    const std::size_t particle_count = workspace.neighborhood.fluid.size();
    const std::size_t master_count = r51.rows.size();
    result.master_exact = workspace.passed && r50.terminal.passed
        && master_count == 494U && r51.cache.size() == master_count
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
    result.geometry_exact = lower.size() == particle_count
        && upper.size() == particle_count
        && r50.witness.size() == particle_count;
    for (std::size_t i = 0U; i < particle_count && result.geometry_exact; ++i)
        for (int axis = 0; axis < 3; ++axis) {
            const double anchor = al_r41_component(r50.witness[i], axis);
            const double low = al_r41_component(lower[i], axis) - anchor;
            const double high = al_r41_component(upper[i], axis) - anchor;
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
        residual[j] = r50.terminal.upper[r51.rows[j]];
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
            for (std::size_t particle = 0U;
                 particle < particle_count; ++particle)
                correction[particle] = correction[particle]
                    - delta * basis.gradient[particle];
            for (std::size_t j = 0U; j < master_count; ++j)
                residual[j] -= delta
                    * basis.gram_column[r51.rows[j]];
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
                    std::min(z,
                        al_r41_component(correction_upper[i], axis)));
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
            residual[j] = r50.terminal.upper[r51.rows[j]]
                + refreshed.value[r51.rows[j]];

        if (!al_r55_checkpoint(cycle)) continue;
        ALR55Record record;
        record.cycle = cycle;
        record.before_box_maximum = before_box;
        record.after_box_master_maximum =
            *std::max_element(residual.begin(), residual.end());
        std::vector<Vec3> witness(particle_count);
        for (std::size_t i = 0U; i < particle_count; ++i)
            witness[i] = r50.witness[i] + correction[i];
        record.box_exact = al_r49_feasible(witness, lower, upper);
        record.witness_norm = al_r32_vec3_norm(witness);
        record.ball_exact = std::isfinite(record.witness_norm)
            && record.witness_norm <= AL_R48_NORMAL_RADIUS;
        record.correction_norm = al_r32_vec3_norm(correction);
        record.box_dual_norm = al_r32_vec3_norm(box_dual);
        record.audit = al_r50_audit(workspace, witness);
        ++result.audit_jvp;
        record.certified = record.audit.passed && record.audit.active.empty();
        for (std::size_t row = 0U; row < row_count; ++row)
            if (master_mask[row] == 0U
                && workspace.support.constraint[row]
                    + record.audit.image.value[row] > 0.0)
                ++record.raw_outside_master;
        for (std::size_t row : record.audit.active)
            if (master_mask[row] == 0U) ++record.directed_outside_master;
        record.strict = record.audit.passed
            && record.audit.primal.psi < AL_R55_R52_PSI
            && record.audit.primal.h < AL_R55_R52_H
            && record.audit.primal.maximum_upper
                < AL_R55_R52_MAXIMUM_UPPER;
        record.witness_root = al_binary64_vec3_root(witness);
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
                 << record.strict << ':' << record.certified << ':'
                 << record.witness_norm << ':' << record.correction_norm << ':'
                 << record.box_dual_norm << ':' << record.witness_root << ':'
                 << record.correction_root << ':' << record.box_dual_root;
        record.root = sha256_hex(material.str());
        record.passed = record.audit.passed && record.box_exact
            && std::isfinite(record.correction_norm)
            && std::isfinite(record.box_dual_norm);
        if (cycle == AL_R55_CHECKPOINTS.back()) {
            result.diagnostic_witness = witness;
            result.diagnostic_correction = correction;
            result.diagnostic_box_dual = box_dual;
            result.diagnostic_audit = record.audit;
        }
        result.records.push_back(std::move(record));
    }
    result.dykstra_exact = exact && result.density_sweeps == 64U
        && result.box_blocks == 64U && result.residual_jvp == 64U
        && result.records.size() == AL_R55_CHECKPOINTS.size();
    result.box_exact = result.dykstra_exact
        && std::all_of(result.records.begin(), result.records.end(),
            [](const ALR55Record& record) {
                return record.passed && record.box_exact;
            });
    result.ball_exact = result.box_exact
        && std::all_of(result.records.begin(), result.records.end(),
            [](const ALR55Record& record) {
                return std::isfinite(record.witness_norm);
            });
    result.primal_exact = result.ball_exact
        && result.audit_jvp == AL_R55_CHECKPOINTS.size();
    if (!result.primal_exact) return result;

    result.contraction = true;
    double previous = std::numeric_limits<double>::infinity();
    for (const ALR55Record& record : result.records) {
        result.contraction = result.contraction
            && record.after_box_master_maximum < previous;
        previous = record.after_box_master_maximum;
        result.any_outside_master = result.any_outside_master
            || record.raw_outside_master > 0U
            || record.directed_outside_master > 0U;
        result.any_ball_outside = result.any_ball_outside
            || !record.ball_exact;
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
        result.terminal = result.records[result.selected_index].audit;
        result.witness = r50.witness;
        const ALR55Record& selected = result.records[result.selected_index];
        (void)selected;
    } else {
        result.terminal = result.records.back().audit;
    }
    result.selection_exact = result.terminal.passed
        && (!result.certified || result.terminal.active.empty());
    result.pair_passes = result.residual_jvp + result.audit_jvp;
    std::ostringstream state;
    state << result.contraction << ':' << result.selected << ':'
          << result.certified << ':' << result.selected_index << ':'
          << result.total_box_changes << ':';
    for (const ALR55Record& record : result.records) state << record.root << ':';
    result.state_root = sha256_hex(state.str());
    return result;
}

bool al_r55_dense_controls(std::string* output_root) {
    double x = 0.0;
    double y = 0.0;
    double halfspace_dual = 0.0;
    double box_dual_x = 0.0;
    double box_dual_y = 0.0;
    for (std::size_t cycle = 0U; cycle < 64U; ++cycle) {
        const double residual = 1.0 - x - y;
        const double next = std::max(0.0, halfspace_dual + residual / 2.0);
        const double delta = next - halfspace_dual;
        halfspace_dual = next;
        x += delta;
        y += delta;
        const double zx = x + box_dual_x;
        const double zy = y + box_dual_y;
        x = std::max(0.0, std::min(zx, 0.4));
        y = std::max(0.0, std::min(zy, 1.0));
        box_dual_x = zx - x;
        box_dual_y = zy - y;
    }
    const bool passed = std::abs(x - 0.4) <= 1.0e-15
        && std::abs(y - 0.6) <= 1.0e-15
        && std::abs(x + y - 1.0) <= 1.0e-15 && box_dual_x > 0.0;
    std::ostringstream material;
    material << std::setprecision(17) << passed << ':' << x << ':' << y << ':'
             << halfspace_dual << ':' << box_dual_x << ':' << box_dual_y;
    if (output_root != nullptr) *output_root = sha256_hex(material.str());
    return passed;
}

std::string al_r55_route(bool parent, bool source, bool workspace,
    bool topology, bool master, bool geometry, bool dykstra, bool box,
    bool ball, bool primal, bool selection, bool work, bool certified,
    bool outside_master, bool ball_integration, bool candidate) {
    if (!parent) return "CONTACT_PARENT_REJECTED";
    if (!source) return "CONTACT_SOURCE_REJECTED";
    if (!workspace) return "CONTACT_WORKSPACE_REJECTED";
    if (!topology) return "CONTACT_TOPOLOGY_REJECTED";
    if (!master) return "CONTACT_MASTER_REJECTED";
    if (!geometry) return "CONTACT_GEOMETRY_REJECTED";
    if (!dykstra) return "CONTACT_DYKSTRA_REJECTED";
    if (!box) return "CONTACT_BOX_REJECTED";
    if (!ball) return "CONTACT_BALL_REJECTED";
    if (!primal) return "CONTACT_PRIMAL_AUDIT_REJECTED";
    if (!selection) return "CONTACT_SELECTION_REJECTED";
    if (!work) return "CONTACT_WORK_REJECTED";
    if (certified) return "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE";
    if (outside_master)
        return "CONTACT_CONSTRAINED_MASTER_EXPANSION_REQUIRED";
    if (ball_integration)
        return "CONTACT_CONSTRAINED_BALL_INTEGRATION_REQUIRED";
    if (candidate) return "CONTACT_CONSTRAINED_DYKSTRA_CANDIDATE";
    return "CONTACT_CONSTRAINED_REFERENCE_RETAINED";
}

struct ALR55ReplayCapture {
    bool captured = false;
    ALR54ReplayCapture r54;
    ALR55Result result;
};

} // namespace

SplitBoundaryReport run_al_contact_constrained_dykstra_controls_impl(
    ALR55ReplayCapture* replay_capture) {
    const bool identity = sha256_hex(B4E2D7R19R55_IDENTITY_PROJECTION)
        == B4E2D7R19R55_IDENTITY_SHA256;
    ALR54ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_model_projection_decomposition_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "b2f32c0aa9e8dd689e3924d8a88db5c4c971404fbc3835bd313cf7512994edff"
        && parent.json.find(
            "9d0b1b8cc165ae84463386751414fe5614f9a62af7117609cd575f4573f14e74")
            != std::string::npos
        && parent.json.find("BALL_BOX_PROJECTION_MODEL_REQUIRED")
            != std::string::npos;
    ALR53ReplayCapture& r53_capture = capture.r53;
    ALR52ReplayCapture& r52_capture = r53_capture.r52;
    ALR51ReplayCapture& r51_capture = r52_capture.r51;
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
    const bool source_exact = parent_exact && r30.captured
        && capture.decomposition.target_root
            == "7bf5b35fa9cedb0aca615e994e1252b61c3b8a0d69db1d168a07d04f5b584203"
        && capture.decomposition.projected_root
            == "ccc1084988fb56a829649f60c292210f24723a4932874923640d5bdbf99e8729"
        && capture.decomposition.displacement_root
            == "bdb3ec8e28529c4ddfe02f5b6c6297e75e7361ac72428fed1af3f324263cce45"
        && capture.decomposition.box_violations == 1125U
        && r51.union_root
            == "38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28"
        && r51.extended_cache_root
            == "f4e9368971d4b6feeb13874d0279dc73fd54ebbb523c75c5c2dd303acfc58cd5";

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

    std::vector<Vec3> lower(r43.projected_trial.size());
    std::vector<Vec3> upper(r43.projected_trial.size());
    bool geometry_exact = master_exact;
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
            al_r43_component(lower[i], axis) = low;
            al_r43_component(upper[i], axis) = high;
            geometry_exact = geometry_exact && std::isfinite(low)
                && std::isfinite(high) && low <= 0.0 && high >= 0.0;
        }

    ALR55Result result;
    if (geometry_exact)
        result = al_r55_generate(moved, r50, r51, lower, upper);
    if (moved.passed) release_al_normalized_workspace(moved, &workspace_work);
    const bool candidate = result.selected && !result.certified
        && !result.any_outside_master && !result.any_ball_outside;
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
        && result.pair_passes == 68U;
    const bool rollback = position_before == al_binary64_vec3_root(r30.position)
        && trial_before == al_binary64_vec3_root(r43.projected_trial)
        && owner_before == al_r29_mask_root(r43.owners)
        && superset_before == al_r42_superset_root(r43.r42.superset)
        && anchor_before == al_binary64_vec3_root(r50.witness)
        && capture.decomposition.box_violations == 1125U;

    const std::array<const char*, 17U> expected_routes{{
        "CONTACT_PARENT_REJECTED", "CONTACT_SOURCE_REJECTED",
        "CONTACT_WORKSPACE_REJECTED", "CONTACT_TOPOLOGY_REJECTED",
        "CONTACT_MASTER_REJECTED", "CONTACT_GEOMETRY_REJECTED",
        "CONTACT_DYKSTRA_REJECTED", "CONTACT_BOX_REJECTED",
        "CONTACT_BALL_REJECTED", "CONTACT_PRIMAL_AUDIT_REJECTED",
        "CONTACT_SELECTION_REJECTED", "CONTACT_WORK_REJECTED",
        "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE",
        "CONTACT_CONSTRAINED_MASTER_EXPANSION_REQUIRED",
        "CONTACT_CONSTRAINED_BALL_INTEGRATION_REQUIRED",
        "CONTACT_CONSTRAINED_DYKSTRA_CANDIDATE",
        "CONTACT_CONSTRAINED_REFERENCE_RETAINED"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t route_index = 0U;
         route_index < expected_routes.size(); ++route_index) {
        std::array<bool, 12U> gates{};
        gates.fill(true);
        if (route_index < gates.size()) gates[route_index] = false;
        const std::string observed = al_r55_route(
            gates[0], gates[1], gates[2], gates[3], gates[4], gates[5],
            gates[6], gates[7], gates[8], gates[9], gates[10], gates[11],
            route_index == 12U, route_index == 13U, route_index == 14U,
            route_index == 15U);
        routes_exact = routes_exact && observed == expected_routes[route_index];
        route_material << expected_routes[route_index] << ':' << observed << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r55_route(parent_exact, source_exact,
        workspace_exact, topology_exact, result.master_exact,
        result.geometry_exact, result.dykstra_exact, result.box_exact,
        result.ball_exact, result.primal_exact, result.selection_exact,
        work_exact && rollback, result.certified, result.any_outside_master,
        result.any_ball_outside, candidate);
    const bool passed = identity && parent_exact && source_exact && dense
        && workspace_exact && topology_exact && master_exact
        && result.master_exact && result.geometry_exact && result.dykstra_exact
        && result.box_exact && result.ball_exact && result.primal_exact
        && result.selection_exact && work_exact && rollback && routes_exact;

    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R55_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << result.state_root << ':' << result.records.size() << ':'
             << result.contraction << ':' << result.selected << ':'
             << result.certified << ':' << result.selected_index << ':'
             << result.any_outside_master << ':' << result.any_ball_outside
             << '|' << result.density_sweeps << ':' << result.box_blocks << ':'
             << result.residual_jvp << ':' << result.audit_jvp << ':'
             << result.pair_passes << ':' << result.total_box_changes << '|'
             << work_exact << ':' << rollback << ':' << route_root << ':'
             << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r55_contact_constrained_dykstra.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R55_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"records\":[";
    for (std::size_t i = 0U; i < result.records.size(); ++i) {
        if (i != 0U) report << ',';
        const ALR55Record& record = result.records[i];
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
               << ",\"witness_norm\":" << record.witness_norm
               << ",\"correction_norm\":" << record.correction_norm
               << ",\"box_dual_norm\":" << record.box_dual_norm
               << ",\"strict\":" << (record.strict ? "true" : "false")
               << ",\"certified\":"
               << (record.certified ? "true" : "false")
               << ",\"witness_root\":\"" << record.witness_root
               << "\",\"correction_root\":\"" << record.correction_root
               << "\",\"box_dual_root\":\"" << record.box_dual_root
               << "\",\"root\":\"" << record.root << "\"}";
    }
    report << "],\"selection\":{\"contraction\":"
           << (result.contraction ? "true" : "false")
           << ",\"selected\":" << (result.selected ? "true" : "false")
           << ",\"certified\":" << (result.certified ? "true" : "false")
           << ",\"selected_index\":" << result.selected_index
           << ",\"outside_master\":"
           << (result.any_outside_master ? "true" : "false")
           << ",\"ball_outside\":"
           << (result.any_ball_outside ? "true" : "false") << "}"
           << ",\"work\":{\"density_sweeps\":" << result.density_sweeps
           << ",\"box_blocks\":" << result.box_blocks
           << ",\"residual_jvp\":" << result.residual_jvp
           << ",\"audit_jvp\":" << result.audit_jvp
           << ",\"pair_passes\":" << result.pair_passes
           << ",\"total_box_changes\":" << result.total_box_changes
           << ",\"workspace_builds\":" << workspace_work.workspace_builds
           << ",\"workspace_releases\":"
           << workspace_work.workspace_releases
           << ",\"exact\":" << (work_exact ? "true" : "false") << "}"
           << ",\"state_root\":\"" << result.state_root << "\""
           << ",\"dense_root\":\"" << dense_root << "\""
           << ",\"route_cases\":" << expected_routes.size()
           << ",\"route_root\":\"" << route_root
           << "\",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback ? "true" : "false")
           << ",\"route\":\"" << route
           << "\",\"current_directed_certificate_unchanged\":true"
              ",\"post_terminal_projection_applied\":false"
              ",\"normal_witness_applied\":false"
              ",\"r43_restoration_committed\":false"
              ",\"restoration_exit_authorized\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    if (replay_capture != nullptr) {
        replay_capture->captured = passed;
        replay_capture->r54 = std::move(capture);
        replay_capture->result = std::move(result);
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_contact_constrained_dykstra_controls() {
    return run_al_contact_constrained_dykstra_controls_impl(nullptr);
}
