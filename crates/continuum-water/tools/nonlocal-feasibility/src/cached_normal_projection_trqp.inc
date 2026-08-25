namespace {

constexpr const char* B4E2D7R19R63_IDENTITY_SHA256 =
    "7cc6eb487c099cce6c08c4ef52c7474f304fdf4e8a127cc49c2926a22b4c38d8";
constexpr const char* B4E2D7R19R63_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r63-cached-normal-projection-trqp|v1|parent=e2e19892:acbfa4befbc91a1c59e0ab75937665e7747112fd118a66eea6753f1206195fae:61de28c308a1f9199c954a4c785fe88e767cabd96e71223d3503b0111145da43:RESTORATION_EXIT_TRANSACTION_CANDIDATE|source=payload-5995a2cca7c24b99b328d4c66af7f9c7a710cfe4e684d0af305f81f1e162b41c;position-0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50;normal-040cc9f0dc57f543c8c3f9e1324b2e68dcc827e120782f9813a7aa0652d884bd;filter-19c1311bf84aaa236a8ce4ec82359ffadb6db404a721ea804f66daabef472c19;radius0.0625;certificate-78472fcb347565bdcc4be443a3cbf220cc5459fd5959a1bcc1c8bd1d428e55bf;active-82086595b4ed35abd2bf7f7cf6107359eaa9fe92be80df7aad1174dbf043bdb3;master-38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28;cache-f4e9368971d4b6feeb13874d0279dc73fd54ebbb523c75c5c2dd303acfc58cd5|model=f=0.5*norm2x-minus-predicted;dimensionless-s;target=predicted-minus-r43-divided-by-spacing;hessian=spacing-squared-identity;trqp=min0.5*norm2s-minus-target;density=c-r43-plus-A-s<=0;contact-box-at-r43;global-l2-radius0.0625;composite-s=normal+tangential|consumption=r62-payload-consumed-once;normal-used-as-feasible-anchor-and-model-reference;normal-not-applied-alone;tangential=composite-minus-normal|solver=cyclic-hildreth-density-plus-dykstra-ball-box;initial-composite=target;density-lambda0;box-dual0;stable-r51-master494;captured-basis-and-gram;fresh-pair-once-residual-after-each-box;cycles64;checkpoints8,16,32,64;no-tolerance-stop|certificate=unchanged-r61-topology-owned-row-local-audit-each-checkpoint;binary64;candidate-positive0-only;raw-and-candidate-outside-master-counted;first-certified-model-reducing-checkpoint;else-terminal-classification|geometry=all-r43-contact-intervals;projection-ball-intersection-box-each-cycle;selected-or-terminal-composite-endpoint-contact-tests36000;trust-and-component-direct-audit|model-audit=normal-endpoint-inertia;checkpoint-composite-inertia;strict-reduction-no-fitted-tolerance;target-identity;no-phr-objective;no-hvp|controls=parent;source;capture;workspace;topology;master;model;geometry;dykstra;projection;certificate;contact;consumption;work;rollback;route-precedence|routes=cached-trqp-parent-rejected;cached-trqp-source-rejected;cached-trqp-capture-rejected;cached-trqp-workspace-rejected;cached-trqp-topology-rejected;cached-trqp-master-rejected;cached-trqp-model-rejected;cached-trqp-geometry-rejected;cached-trqp-dykstra-rejected;cached-trqp-projection-rejected;cached-trqp-certificate-audit-rejected;cached-trqp-contact-rejected;cached-trqp-consumption-rejected;cached-trqp-work-rejected;tangential-master-expansion-required;tangential-model-reduction-rejected;cached-normal-projected-trqp-candidate;tangential-projection-depth-required;tangential-certificate-refinement-required;cached-trqp-reference-retained|precedence=parent,source,capture,workspace,topology,master,model,geometry,dykstra,projection,audit,contact,consumption,work,outside-master,certified-model,certified,raw-positive,bound-only,reference|work=parent-r62-replays1;new-static-index1;new-r43-workspace1;workspace-release1;captured-row-vjp0;captured-gram-jvp0;density-cycles64;ball-box-projections64;fresh-pair-once-jvp64;checkpoint-directed-jvp4;new-pairpasses68;selected-contact-tests36000;new-hvp0;new-model-hvp0;new-nonlinear-density-trial0;new-switching0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53-r54-r55-r56-r57-r58-r59-r60-r61-r62=unchanged;r62-public-output=byte-exact;private-trqp-only;runtime-position-commit=none;normal-apply=none;tangential-apply=none;filter-runtime-commit=none;switching=none;trust-update=none;multiplier-update=none;hessian-update=none;following-outer=none;tolerance=none;capacity-change=none;binary128=none;timing=none;runtime=none;production=none|credit=one-private-cached-normal-projection-trqp-classification-only)IDENTITY";

constexpr std::array<std::size_t, 4U> AL_R63_CHECKPOINTS{{8U, 16U, 32U, 64U}};

bool al_r63_checkpoint(std::size_t cycle) {
    return std::find(AL_R63_CHECKPOINTS.begin(), AL_R63_CHECKPOINTS.end(), cycle)
        != AL_R63_CHECKPOINTS.end();
}

double al_r63_inertia(const std::vector<Vec3>& position,
    const std::vector<Vec3>& predicted) {
    if (position.size() != predicted.size())
        return std::numeric_limits<double>::infinity();
    double result = 0.0;
    for (std::size_t i = 0U; i < position.size(); ++i)
        result += 0.5 * norm_squared(position[i] - predicted[i]);
    return result;
}

struct ALR63ModelAudit {
    bool exact = false;
    double direct = std::numeric_limits<double>::infinity();
    double projection = std::numeric_limits<double>::infinity();
    double error = std::numeric_limits<double>::infinity();
    double bound = 0.0;
};

ALR63ModelAudit al_r63_model_audit(const std::vector<Vec3>& composite,
    const std::vector<Vec3>& target, const std::vector<Vec3>& endpoint,
    const std::vector<Vec3>& predicted) {
    ALR63ModelAudit result;
    if (composite.size() != target.size() || composite.size() != endpoint.size()
        || endpoint.size() != predicted.size()) return result;
    result.direct = al_r63_inertia(endpoint, predicted);
    double squared = 0.0;
    for (std::size_t i = 0U; i < composite.size(); ++i)
        squared += norm_squared(composite[i] - target[i]);
    result.projection = 0.5 * SPACING * SPACING * squared;
    result.error = std::abs(result.direct - result.projection);
    result.bound = gamma_factor(32U * composite.size() + 64U)
        * (std::abs(result.direct) + std::abs(result.projection));
    result.exact = std::isfinite(result.direct)
        && std::isfinite(result.projection) && std::isfinite(result.error)
        && std::isfinite(result.bound) && result.error <= result.bound;
    return result;
}

struct ALR63Record {
    bool projection_exact = false;
    bool audit_exact = false;
    bool model_exact = false;
    bool certified = false;
    bool model_reducing = false;
    std::size_t cycle = 0U;
    std::size_t raw_outside_master = 0U;
    std::size_t candidate_outside_master = 0U;
    double composite_norm = 0.0;
    double inertia = 0.0;
    double model_reduction = 0.0;
    double master_maximum = 0.0;
    ALR61Audit audit;
    std::vector<Vec3> composite;
    std::string composite_root;
    std::string root;
};

struct ALR63Result {
    bool dykstra_exact = false;
    bool projection_exact = false;
    bool audit_exact = false;
    bool selection_exact = false;
    bool any_outside_master = false;
    bool any_certified = false;
    bool selected = false;
    std::size_t selected_index = 0U;
    std::size_t density_cycles = 0U;
    std::size_t projections = 0U;
    std::size_t projection_scans = 0U;
    std::size_t pair_jvp = 0U;
    std::size_t directed_jvp = 0U;
    std::vector<ALR63Record> records;
    std::vector<Vec3> terminal;
    std::string state_root;
};

ALR63Result al_r63_project(const ALNormalizedWorkspace& workspace,
    const ALR51MasterResult& master, const std::vector<Vec3>& target,
    const std::vector<Vec3>& lower, const std::vector<Vec3>& upper,
    double normal_inertia) {
    ALR63Result result;
    const std::size_t row_count = workspace.support.constraint.size();
    const std::size_t particle_count = workspace.neighborhood.fluid.size();
    if (!workspace.passed || target.size() != particle_count
        || lower.size() != particle_count || upper.size() != particle_count
        || master.rows.size() != 494U || master.cache.size() != 494U)
        return result;
    std::vector<std::size_t> basis_index(master.rows.size());
    bool exact = true;
    for (std::size_t i = 0U; i < master.rows.size(); ++i) {
        basis_index[i] = al_r50_find_basis(master.cache, master.rows[i]);
        exact = exact && basis_index[i] < master.cache.size();
    }
    if (!exact) return result;

    std::vector<std::uint8_t> master_mask(row_count, 0U);
    for (std::size_t row : master.rows) master_mask[row] = 1U;
    std::vector<Vec3> composite = target;
    std::vector<Vec3> box_dual(particle_count);
    std::vector<double> lambda(master.rows.size(), 0.0);
    std::vector<double> residual(master.rows.size());
    for (std::size_t i = 0U; i < master.rows.size(); ++i) {
        const ALR50RowBasis& basis = master.cache[basis_index[i]];
        residual[i] = workspace.support.constraint[master.rows[i]]
            + static_cast<double>(al_r35_dot_vec3(basis.gradient, composite));
    }

    for (std::size_t cycle = 1U; cycle <= 64U && exact; ++cycle) {
        for (std::size_t i = 0U; i < master.rows.size(); ++i) {
            const ALR50RowBasis& basis = master.cache[basis_index[i]];
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
                composite[particle] = composite[particle]
                    - delta * basis.gradient[particle];
            for (std::size_t j = 0U; j < master.rows.size(); ++j)
                residual[j] -= delta
                    * basis.gram_column[master.rows[j]];
        }
        if (!exact || !al_r48_vec_finite(composite)
            || !al_r30_finite(residual)) break;
        ++result.density_cycles;

        std::vector<Vec3> projected_input(particle_count);
        for (std::size_t i = 0U; i < particle_count; ++i)
            projected_input[i] = composite[i] + box_dual[i];
        const ALR48Projection projection = al_r48_project_ball_box(
            projected_input, lower, upper, AL_R62_NEXT_RADIUS);
        ++result.projections;
        result.projection_scans += projection.scans;
        if (!projection.passed) {
            exact = false;
            break;
        }
        composite = projection.value;
        for (std::size_t i = 0U; i < particle_count; ++i)
            box_dual[i] = projected_input[i] - composite[i];
        exact = exact && al_r48_vec_finite(composite)
            && al_r48_vec_finite(box_dual)
            && al_r32_vec3_norm(composite) <= AL_R62_NEXT_RADIUS
            && al_r49_feasible(composite, lower, upper);
        if (!exact) break;

        const ALLinearizedJvp refreshed =
            al_r29_pair_once_jvp(workspace, composite);
        ++result.pair_jvp;
        if (!refreshed.passed || refreshed.value.size() != row_count) {
            exact = false;
            break;
        }
        for (std::size_t i = 0U; i < master.rows.size(); ++i)
            residual[i] = workspace.support.constraint[master.rows[i]]
                + refreshed.value[master.rows[i]];

        if (!al_r63_checkpoint(cycle)) continue;
        ALR63Record record;
        record.cycle = cycle;
        record.projection_exact = projection.passed
            && al_r49_feasible(composite, lower, upper);
        record.composite_norm = al_r32_vec3_norm(composite);
        record.master_maximum = *std::max_element(
            residual.begin(), residual.end());
        record.audit = al_r61_topology_owned_audit(workspace, composite);
        ++result.directed_jvp;
        record.audit_exact = record.audit.owner_exact;
        for (std::size_t row = 0U; row < row_count; ++row) {
            if (master_mask[row] == 0U && record.audit.raw[row] > 0.0)
                ++record.raw_outside_master;
            if (master_mask[row] == 0U
                && record.audit.candidate_upper[row] > 0.0)
                ++record.candidate_outside_master;
        }
        record.composite = composite;
        record.composite_root = al_binary64_vec3_root(composite);
        record.inertia = 0.5 * SPACING * SPACING
            * static_cast<double>(al_r35_dot_vec3(
                al_r48_difference(composite, target),
                al_r48_difference(composite, target)));
        record.model_reduction = normal_inertia - record.inertia;
        record.model_exact = std::isfinite(record.inertia)
            && std::isfinite(record.model_reduction);
        record.model_reducing = record.model_reduction > 0.0;
        record.certified = record.audit_exact
            && record.audit.candidate_positive == 0U;
        std::ostringstream material;
        material << cycle << ':' << record.composite_root << ':'
                 << binary64_bits(record.composite_norm) << ':'
                 << binary64_bits(record.master_maximum) << ':'
                 << record.audit.raw_positive << ':'
                 << record.audit.candidate_positive << ':'
                 << record.raw_outside_master << ':'
                 << record.candidate_outside_master << ':'
                 << binary64_bits(record.inertia) << ':'
                 << binary64_bits(record.model_reduction) << ':'
                 << record.certified << ':' << record.model_reducing;
        record.root = sha256_hex(material.str());
        result.any_outside_master = result.any_outside_master
            || record.raw_outside_master > 0U
            || record.candidate_outside_master > 0U;
        result.any_certified = result.any_certified || record.certified;
        if (!result.selected && record.certified && record.model_reducing) {
            result.selected = true;
            result.selected_index = result.records.size();
        }
        result.records.push_back(std::move(record));
    }
    result.dykstra_exact = exact && result.density_cycles == 64U
        && result.projections == 64U && result.pair_jvp == 64U
        && result.records.size() == AL_R63_CHECKPOINTS.size();
    result.projection_exact = result.dykstra_exact
        && std::all_of(result.records.begin(), result.records.end(),
            [](const ALR63Record& record) {
                return record.projection_exact;
            });
    result.audit_exact = result.projection_exact
        && result.directed_jvp == AL_R63_CHECKPOINTS.size()
        && std::all_of(result.records.begin(), result.records.end(),
            [](const ALR63Record& record) { return record.audit_exact; });
    result.selection_exact = result.audit_exact
        && (!result.selected || (result.selected_index < result.records.size()
            && result.records[result.selected_index].certified
            && result.records[result.selected_index].model_reducing));
    if (!result.records.empty()) {
        const std::size_t terminal_index = result.selected
            ? result.selected_index : result.records.size() - 1U;
        result.terminal = result.records[terminal_index].composite;
    }
    std::ostringstream state;
    state << result.dykstra_exact << ':' << result.projection_exact << ':'
          << result.audit_exact << ':' << result.selected << ':'
          << result.selected_index << ':' << result.any_outside_master << ':';
    for (const ALR63Record& record : result.records) state << record.root << ':';
    result.state_root = sha256_hex(state.str());
    return result;
}

std::string al_r63_route(bool parent, bool source, bool capture,
    bool workspace, bool topology, bool master, bool model, bool geometry,
    bool dykstra, bool projection, bool audit, bool contact,
    bool consumption, bool work, bool outside_master, bool any_certified,
    bool certified_model, bool selected, bool raw_positive,
    bool bound_only) {
    if (!parent) return "CACHED_TRQP_PARENT_REJECTED";
    if (!source) return "CACHED_TRQP_SOURCE_REJECTED";
    if (!capture) return "CACHED_TRQP_CAPTURE_REJECTED";
    if (!workspace) return "CACHED_TRQP_WORKSPACE_REJECTED";
    if (!topology) return "CACHED_TRQP_TOPOLOGY_REJECTED";
    if (!master) return "CACHED_TRQP_MASTER_REJECTED";
    if (!model) return "CACHED_TRQP_MODEL_REJECTED";
    if (!geometry) return "CACHED_TRQP_GEOMETRY_REJECTED";
    if (!dykstra) return "CACHED_TRQP_DYKSTRA_REJECTED";
    if (!projection) return "CACHED_TRQP_PROJECTION_REJECTED";
    if (!audit) return "CACHED_TRQP_CERTIFICATE_AUDIT_REJECTED";
    if (!contact) return "CACHED_TRQP_CONTACT_REJECTED";
    if (!consumption) return "CACHED_TRQP_CONSUMPTION_REJECTED";
    if (!work) return "CACHED_TRQP_WORK_REJECTED";
    if (outside_master) return "TANGENTIAL_MASTER_EXPANSION_REQUIRED";
    if (any_certified && !certified_model)
        return "TANGENTIAL_MODEL_REDUCTION_REJECTED";
    if (selected) return "CACHED_NORMAL_PROJECTED_TRQP_CANDIDATE";
    if (raw_positive) return "TANGENTIAL_PROJECTION_DEPTH_REQUIRED";
    if (bound_only) return "TANGENTIAL_CERTIFICATE_REFINEMENT_REQUIRED";
    return "CACHED_TRQP_REFERENCE_RETAINED";
}

} // namespace

SplitBoundaryReport run_al_cached_normal_projection_trqp_controls() {
    const bool identity = sha256_hex(B4E2D7R19R63_IDENTITY_PROJECTION)
        == B4E2D7R19R63_IDENTITY_SHA256;
    ALR62ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_restoration_exit_transaction_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "acbfa4befbc91a1c59e0ab75937665e7747112fd118a66eea6753f1206195fae"
        && parent.json.find(
            "61de28c308a1f9199c954a4c785fe88e767cabd96e71223d3503b0111145da43")
            != std::string::npos
        && parent.json.find("RESTORATION_EXIT_TRANSACTION_CANDIDATE")
            != std::string::npos;

    ALR61ReplayCapture& r61 = capture.r61;
    ALR58ReplayCapture& r58_capture = r61.r60.r59.r58;
    ALR57ReplayCapture& r57_capture = r58_capture.r57;
    ALR56ReplayCapture& r56_capture = r57_capture.r56;
    ALR55ReplayCapture& r55_capture = r56_capture.r55;
    ALR54ReplayCapture& r54_capture = r55_capture.r54;
    ALR53ReplayCapture& r53_capture = r54_capture.r53;
    ALR51ReplayCapture& r51_capture = r53_capture.r52.r51;
    ALR51MasterResult& master = r51_capture.master;
    ALR43ReplayCapture& r43 =
        r51_capture.r50.r49.r48.r47.r46.r45.r44.r43;
    ALR40ReplayCapture& r40 = r43.r42.r40;
    ALR30RangeProjectionCapture& r30 =
        r40.r39.r38.r37.r36.r35.r34.r33.r32.r31.r30;

    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string trial_before = al_binary64_vec3_root(r43.projected_trial);
    const std::string normal_before =
        al_binary64_vec3_root(capture.publication.cached_normal);
    const std::string payload_before =
        al_r62_payload_root(capture.publication.payload);
    const bool source_exact = parent_exact && capture.publication.published
        && capture.publication.publish_count == 1U
        && payload_before
            == "5995a2cca7c24b99b328d4c66af7f9c7a710cfe4e684d0af305f81f1e162b41c"
        && trial_before
            == "0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50"
        && normal_before
            == "040cc9f0dc57f543c8c3f9e1324b2e68dcc827e120782f9813a7aa0652d884bd"
        && capture.publication.payload.filter_root
            == "19c1311bf84aaa236a8ce4ec82359ffadb6db404a721ea804f66daabef472c19"
        && binary64_bits(capture.publication.payload.radius)
            == binary64_bits(0.0625)
        && capture.publication.payload.certificate_root
            == "78472fcb347565bdcc4be443a3cbf220cc5459fd5959a1bcc1c8bd1d428e55bf";
    std::size_t consumption_count = 0U;
    std::vector<Vec3> cached_normal;
    if (source_exact) {
        cached_normal = capture.publication.cached_normal;
        ++consumption_count;
    }
    const bool capture_exact = source_exact && consumption_count == 1U
        && exact_vec3_values(capture.publication.position, r43.projected_trial)
        && al_binary64_vec3_root(cached_normal) == normal_before;

    ALSparseWorkTrace workspace_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    const SmokeFixture fixture = make_b4e2d_dam_fixture();
    const JointStaticSupportIndex index = build_joint_static_support_index(
        tagged_points(fixture.boundary), &static_work);
    const JointStaticSupportBinding binding = bind_joint_static_support_index(
        &index, index.identity_sha256);
    ALNormalizedWorkspace workspace = build_al_normalized_sparse_workspace(
        r43.projected_trial, binding, r30.dual, r30.theta,
        &workspace_work, &static_work, &adjacency_work);
    const bool workspace_exact = capture_exact && index.passed
        && binding.passed && workspace.passed;
    const std::string topology_root = al_r29_topology_root(workspace);
    const bool topology_exact = workspace_exact
        && al_r42_superset_root(workspace.neighborhood)
            == capture.payload.workspace_topology_root;
    const bool master_exact = topology_exact && master.rows.size() == 494U
        && master.cache.size() == 494U
        && std::is_sorted(master.rows.begin(), master.rows.end())
        && master.union_root
            == "38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28"
        && al_r51_cache_root(master.cache)
            == "f4e9368971d4b6feeb13874d0279dc73fd54ebbb523c75c5c2dd303acfc58cd5";

    std::vector<Vec3> target(r43.projected_trial.size());
    std::vector<Vec3> normal_endpoint(r43.projected_trial.size());
    for (std::size_t i = 0U; i < target.size(); ++i) {
        target[i] = (1.0 / SPACING)
            * (r30.predicted_position[i] - r43.projected_trial[i]);
        normal_endpoint[i] = r43.projected_trial[i]
            + SPACING * cached_normal[i];
    }
    const ALR63ModelAudit normal_model = al_r63_model_audit(
        cached_normal, target, normal_endpoint, r30.predicted_position);
    const bool model_exact = master_exact && al_r48_vec_finite(target)
        && al_r48_vec_finite(normal_endpoint) && normal_model.exact;

    std::vector<Vec3> lower(r43.projected_trial.size());
    std::vector<Vec3> upper(r43.projected_trial.size());
    bool intervals_exact = model_exact;
    for (std::size_t i = 0U; i < target.size(); ++i)
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
            const double normal = al_r41_component(cached_normal[i], axis);
            intervals_exact = intervals_exact && std::isfinite(low)
                && std::isfinite(high) && low <= normal && normal <= high;
        }
    const bool geometry_exact = intervals_exact
        && al_r32_vec3_norm(cached_normal) <= AL_R62_NORMAL_RADIUS;

    ALR63Result result;
    if (geometry_exact)
        result = al_r63_project(workspace, master, target, lower, upper,
            normal_model.projection);
    const bool dykstra_exact = result.dykstra_exact;
    const bool projection_exact = result.projection_exact;
    const bool audit_exact = result.audit_exact;
    const std::size_t terminal_index = result.records.empty() ? 0U
        : (result.selected ? result.selected_index : result.records.size() - 1U);
    ALR63Record* terminal = result.records.empty()
        ? nullptr : &result.records[terminal_index];
    std::vector<Vec3> endpoint(r43.projected_trial.size());
    std::vector<Vec3> tangential(r43.projected_trial.size());
    if (terminal != nullptr)
        for (std::size_t i = 0U; i < endpoint.size(); ++i) {
            endpoint[i] = r43.projected_trial[i]
                + SPACING * terminal->composite[i];
            tangential[i] = terminal->composite[i] - cached_normal[i];
        }
    const ALR41ContactAudit contact = al_r41_contact_audit(
        r43.projected_trial, endpoint, fixture);
    const ALR63ModelAudit terminal_model = terminal == nullptr
        ? ALR63ModelAudit{} : al_r63_model_audit(terminal->composite,
            target, endpoint, r30.predicted_position);
    const bool contact_exact = terminal != nullptr && contact.finite
        && contact.tests == 36000U && contact.newly_penetrating == 0U
        && contact.worsened == 0U
        && contact.trial_maximum <= contact.source_maximum;
    const bool consumption_exact = capture_exact && consumption_count == 1U
        && terminal != nullptr && al_r48_vec_finite(tangential)
        && exact_vec3_values(cached_normal, capture.publication.cached_normal)
        && terminal_model.exact;

    if (workspace.passed)
        release_al_normalized_workspace(workspace, &workspace_work);
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
        && result.density_cycles == 64U && result.projections == 64U
        && result.pair_jvp == 64U && result.directed_jvp == 4U
        && contact.tests == 36000U;
    const bool rollback = position_before == al_binary64_vec3_root(r30.position)
        && trial_before == al_binary64_vec3_root(r43.projected_trial)
        && normal_before == al_binary64_vec3_root(
            capture.publication.cached_normal)
        && payload_before == al_r62_payload_root(capture.publication.payload)
        && capture.publication.publish_count == 1U;

    const bool raw_positive = terminal != nullptr
        && terminal->audit.raw_positive > 0U;
    const bool bound_only = terminal != nullptr && !raw_positive
        && terminal->audit.candidate_positive > 0U;
    const bool certified_model = terminal != nullptr && terminal->certified
        && terminal->model_reducing;

    const std::array<const char*, 20U> expected_routes{{
        "CACHED_TRQP_PARENT_REJECTED", "CACHED_TRQP_SOURCE_REJECTED",
        "CACHED_TRQP_CAPTURE_REJECTED", "CACHED_TRQP_WORKSPACE_REJECTED",
        "CACHED_TRQP_TOPOLOGY_REJECTED", "CACHED_TRQP_MASTER_REJECTED",
        "CACHED_TRQP_MODEL_REJECTED", "CACHED_TRQP_GEOMETRY_REJECTED",
        "CACHED_TRQP_DYKSTRA_REJECTED", "CACHED_TRQP_PROJECTION_REJECTED",
        "CACHED_TRQP_CERTIFICATE_AUDIT_REJECTED",
        "CACHED_TRQP_CONTACT_REJECTED", "CACHED_TRQP_CONSUMPTION_REJECTED",
        "CACHED_TRQP_WORK_REJECTED", "TANGENTIAL_MASTER_EXPANSION_REQUIRED",
        "TANGENTIAL_MODEL_REDUCTION_REJECTED",
        "CACHED_NORMAL_PROJECTED_TRQP_CANDIDATE",
        "TANGENTIAL_PROJECTION_DEPTH_REQUIRED",
        "TANGENTIAL_CERTIFICATE_REFINEMENT_REQUIRED",
        "CACHED_TRQP_REFERENCE_RETAINED"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t route_index = 0U;
         route_index < expected_routes.size(); ++route_index) {
        std::array<bool, 14U> gates{};
        gates.fill(true);
        if (route_index < gates.size()) gates[route_index] = false;
        const std::string observed = al_r63_route(
            gates[0], gates[1], gates[2], gates[3], gates[4], gates[5],
            gates[6], gates[7], gates[8], gates[9], gates[10], gates[11],
            gates[12], gates[13], route_index == 14U,
            route_index == 15U || route_index == 16U,
            route_index == 16U, route_index == 16U,
            route_index == 17U, route_index == 18U);
        routes_exact = routes_exact && observed == expected_routes[route_index];
        route_material << expected_routes[route_index] << ':' << observed << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r63_route(parent_exact, source_exact,
        capture_exact, workspace_exact, topology_exact, master_exact,
        model_exact, geometry_exact, dykstra_exact, projection_exact,
        audit_exact, contact_exact, consumption_exact, work_exact && rollback,
        result.any_outside_master, result.any_certified, certified_model,
        result.selected, raw_positive, bound_only);
    const bool passed = identity && parent_exact && source_exact
        && capture_exact && workspace_exact && topology_exact && master_exact
        && model_exact && geometry_exact && dykstra_exact && projection_exact
        && audit_exact && contact_exact && consumption_exact && work_exact
        && rollback && routes_exact;

    const std::string target_root = al_binary64_vec3_root(target);
    const std::string endpoint_root = al_binary64_vec3_root(endpoint);
    const std::string tangential_root = al_binary64_vec3_root(tangential);
    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R63_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << topology_root << ':' << target_root << ':'
             << normal_model.direct << ':' << normal_model.projection << ':'
             << normal_model.error << ':' << normal_model.bound << '|'
             << result.state_root << ':' << result.records.size() << ':'
             << result.selected << ':' << result.selected_index << ':'
             << result.any_outside_master << ':' << result.any_certified << '|';
    for (const ALR63Record& record : result.records)
        semantic << record.root << ':';
    semantic << '|' << endpoint_root << ':' << tangential_root << ':'
             << terminal_model.direct << ':' << terminal_model.projection << ':'
             << contact.root << ':' << consumption_count << '|'
             << work_exact << ':' << rollback << ':' << route_root << ':'
             << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r63_cached_normal_projection_trqp.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R63_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source\":{\"payload_root\":\"" << payload_before
           << "\",\"normal_root\":\"" << normal_before
           << "\",\"consume_count\":" << consumption_count << "}"
           << ",\"model\":{\"target_root\":\"" << target_root
           << "\",\"normal_direct\":" << normal_model.direct
           << ",\"normal_projection\":" << normal_model.projection
           << ",\"identity_error\":" << normal_model.error
           << ",\"identity_bound\":" << normal_model.bound << "}"
           << ",\"records\":[";
    for (std::size_t i = 0U; i < result.records.size(); ++i) {
        if (i != 0U) report << ',';
        const ALR63Record& record = result.records[i];
        report << "{\"cycle\":" << record.cycle
               << ",\"composite_norm\":" << record.composite_norm
               << ",\"master_maximum\":" << record.master_maximum
               << ",\"raw_positive\":" << record.audit.raw_positive
               << ",\"candidate_positive\":"
               << record.audit.candidate_positive
               << ",\"raw_outside_master\":" << record.raw_outside_master
               << ",\"candidate_outside_master\":"
               << record.candidate_outside_master
               << ",\"inertia\":" << record.inertia
               << ",\"model_reduction\":" << record.model_reduction
               << ",\"certified\":" << (record.certified ? "true" : "false")
               << ",\"model_reducing\":"
               << (record.model_reducing ? "true" : "false")
               << ",\"composite_root\":\"" << record.composite_root
               << "\",\"root\":\"" << record.root << "\"}";
    }
    report << "],\"selection\":{\"selected\":"
           << (result.selected ? "true" : "false")
           << ",\"selected_index\":" << result.selected_index
           << ",\"outside_master\":"
           << (result.any_outside_master ? "true" : "false")
           << ",\"any_certified\":"
           << (result.any_certified ? "true" : "false") << "}"
           << ",\"terminal\":{\"endpoint_root\":\"" << endpoint_root
           << "\",\"tangential_root\":\"" << tangential_root
           << "\",\"direct_inertia\":" << terminal_model.direct
           << ",\"projection_inertia\":" << terminal_model.projection
           << ",\"contact_tests\":" << contact.tests
           << ",\"contact_new\":" << contact.newly_penetrating
           << ",\"contact_worsened\":" << contact.worsened << "}"
           << ",\"work\":{\"density_cycles\":" << result.density_cycles
           << ",\"ball_box_projections\":" << result.projections
           << ",\"projection_scans\":" << result.projection_scans
           << ",\"pair_once_jvp\":" << result.pair_jvp
           << ",\"directed_jvp\":" << result.directed_jvp
           << ",\"new_row_vjp\":0,\"new_gram_jvp\":0,\"new_hvp\":0"
           << ",\"exact\":" << (work_exact ? "true" : "false") << "}"
           << ",\"state_root\":\"" << result.state_root
           << "\",\"route_cases\":" << expected_routes.size()
           << ",\"route_root\":\"" << route_root
           << "\",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback ? "true" : "false")
           << ",\"route\":\"" << route
           << "\",\"normal_applied_alone\":false"
              ",\"runtime_state_mutated\":false"
              ",\"nonlinear_density_trial_executed\":false"
              ",\"switching_executed\":false"
              ",\"following_outer_executed\":false"
              ",\"binary128_runtime\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    return {passed, report.str()};
}
