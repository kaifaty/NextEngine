namespace {

constexpr const char* B4E2D7R19R34_IDENTITY_SHA256 =
    "f5350515798dfe3fd14cd533a2dae25516c7432d918fa7ef9863903f89bedbd2";
constexpr const char* B4E2D7R19R34_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r34-first-order-reference-extension|v1|parent=827e6092:6bab6bfb7e203ca45b387bbad04eefbf62aa03fdf187761c70f46d5fd7bc5464:61f21b041257c92222d61b4c454e44aa69c30b3da3e45d8ecc70f952db3e9ec0|source=checkpoint-e949afa7039e76b79deba5cacbbc3d87b29b68bb1a90324e0984b443b0183e55;response-7dd519932782f9177be354418c9ecaaa1fed3baac1e8255305675da7cb59dff3;gradient-295d66fc0b9b5b74e06200db48b11792005720477ad5291b2f6ea1147570679f;projected-mapping-11bade99d29986e5dc2a945bf00ffae3a7b5a69eb334143e69954dff570e1939;particles6000;pairs340340;max-degree113|model=phi(v)=0.5*norm(max(c+A*v,0))2;A=SPACING*Jc;all-rows;support-fixed|trust=global-l2;delta0.25;fixed|continuation=exact-r33-iterate-response;prefix-fresh-jvp;iterations9-through32;projected-normalized-steepest;exact-r32-piecewise-line;checkpoints16,32|slope=phi16/phi8;phi32/phi16;violation16/violation8;violation32/violation16;geometric-per-step;report-only|terminal=fresh-jvp-v;fresh-vjp-positive;maintained-direct-response-relative<=1e-12;projected-mapping;strict-monotone|controls=dense-conditioned-slope;dense-early-stationary;dense-boundary-stationary;parent;source;workspace;prefix;iteration;direct;work;rollback|observations=checkpoint-phi-violation-active-step;accepted-total;terminal-response-gradient-mapping-roots;norm-ratios;trust-use;block-contractions|routes=extended-parent-rejected;extended-source-rejected;extended-dense-rejected;extended-workspace-rejected;extended-prefix-rejected;extended-iteration-rejected;extended-direct-rejected;extended-work-rejected;all-inequality-projected-stationary;extended-first-order-saturated;extended-first-order-reference-candidate|precedence=parent,source,dense,workspace,prefix,iteration,direct,work,classification|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-r33-once;diagnostic-workspaces1;new-pair-passes<=51;new-hvp0;new-model0;new-trial0;new-outer0|correction=none;nonlinear-evaluation=none;floor-classification=none;state-mutation=none;following-outer=none;substep=none;macro=none;trajectory=none;timing=none;public-schema=none;runtime=none;production=none|credit=one-private-32-step-first-order-reference-classification-only)IDENTITY";

struct ALR34Checkpoint {
    std::size_t iteration = 0U;
    double phi = 0.0;
    double violation_norm = 0.0;
    double step_norm = 0.0;
    double iterate_norm = 0.0;
    double gradient_norm = 0.0;
    std::size_t active = 0U;
    std::string line_root;
    std::string root;
};

std::string al_r34_route(
    bool parent, bool source, bool dense, bool workspace, bool prefix,
    bool iteration, bool direct, bool work, bool stationary, bool saturated) {
    if (!parent) return "EXTENDED_PARENT_REJECTED";
    if (!source) return "EXTENDED_SOURCE_REJECTED";
    if (!dense) return "EXTENDED_DENSE_REJECTED";
    if (!workspace) return "EXTENDED_WORKSPACE_REJECTED";
    if (!prefix) return "EXTENDED_PREFIX_REJECTED";
    if (!iteration) return "EXTENDED_ITERATION_REJECTED";
    if (!direct) return "EXTENDED_DIRECT_REJECTED";
    if (!work) return "EXTENDED_WORK_REJECTED";
    if (stationary) return "ALL_INEQUALITY_PROJECTED_STATIONARY";
    return saturated
        ? "EXTENDED_FIRST_ORDER_SATURATED"
        : "EXTENDED_FIRST_ORDER_REFERENCE_CANDIDATE";
}

} // namespace

SplitBoundaryReport run_al_first_order_reference_extension_controls() {
    const bool identity_exact = sha256_hex(B4E2D7R19R34_IDENTITY_PROJECTION)
        == B4E2D7R19R34_IDENTITY_SHA256;
    ALR33NormalStepCapture capture;
    const SplitBoundaryReport parent =
        run_al_iterated_all_inequality_normal_step_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "61f21b041257c92222d61b4c454e44aa69c30b3da3e45d8ecc70f952db3e9ec0"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && parent_semantic && parent_stdout
            == "6bab6bfb7e203ca45b387bbad04eefbf62aa03fdf187761c70f46d5fd7bc5464";
    ALR30RangeProjectionCapture& r30 = capture.r32.r31.r30;
    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string dual_before = al_r10_dual_root(r30.dual);
    const std::string constraint_before = al_r10_dual_root(r30.constraint);
    std::ostringstream parent_checkpoint_projection;
    for (const ALR33Checkpoint& checkpoint : capture.checkpoints) {
        parent_checkpoint_projection << checkpoint.root << '|';
    }
    const std::string parent_checkpoint_root =
        sha256_hex(parent_checkpoint_projection.str());
    const bool source_exact = parent_exact
        && parent_checkpoint_root
            == "e949afa7039e76b79deba5cacbbc3d87b29b68bb1a90324e0984b443b0183e55"
        && al_r10_dual_root(capture.terminal_response)
            == "7dd519932782f9177be354418c9ecaaa1fed3baac1e8255305675da7cb59dff3"
        && al_binary64_vec3_root(capture.terminal_gradient)
            == "295d66fc0b9b5b74e06200db48b11792005720477ad5291b2f6ea1147570679f"
        && r30.state_root
            == "851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7"
        && constraint_before
            == "c567599060d7d00ffec4928009772327d081d3ef294d9030bfc5499b82e533c9"
        && r30.topology_root
            == "070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5"
        && capture.iterate.size() == 6000U
        && capture.maintained_response.size() == 6000U;

    const ALR33DenseControl dense_short = al_r33_dense_run(
        "conditioned-slope-8",
        {{1.0, 0.0}, {-1.0, 0.0}, {0.0, 8.0}, {0.0, -8.0}},
        {1.1, 0.9, 1.1, 0.9}, 2.0, 8U);
    const ALR33DenseControl dense_long = al_r33_dense_run(
        "conditioned-slope-32",
        {{1.0, 0.0}, {-1.0, 0.0}, {0.0, 8.0}, {0.0, -8.0}},
        {1.1, 0.9, 1.1, 0.9}, 2.0, 32U);
    const ALR33DenseControl dense_stationary = al_r33_dense_run(
        "early-stationary", {{-1.0}, {1.0}}, {1.0, -0.1}, 1.0, 32U);
    const ALR33DenseControl dense_boundary = al_r33_dense_run(
        "boundary-stationary", {{-0.25}}, {1.0}, 1.0, 32U);
    const bool dense_exact = dense_short.passed && dense_long.passed
        && dense_long.accepted == 32U && dense_long.phi < dense_short.phi
        && dense_stationary.passed && dense_stationary.stationary
        && dense_stationary.accepted == 1U
        && dense_boundary.passed && dense_boundary.stationary
        && dense_boundary.accepted == 1U;
    std::ostringstream dense_projection;
    dense_projection << dense_short.root << ':' << dense_long.root << ':'
                     << dense_stationary.root << ':' << dense_boundary.root
                     << ':' << dense_exact;
    const std::string dense_root = sha256_hex(dense_projection.str());

    ALSparseWorkTrace work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    const SmokeFixture fixture = make_b4e2d_dam_fixture();
    const JointStaticSupportIndex index = build_joint_static_support_index(
        tagged_points(fixture.boundary), &static_work);
    const JointStaticSupportBinding binding = bind_joint_static_support_index(
        &index, index.identity_sha256);
    ALNormalizedWorkspace workspace;
    if (source_exact && dense_exact && index.passed && binding.passed) {
        workspace = build_al_normalized_sparse_workspace(
            r30.position, binding, r30.dual, r30.theta, &work,
            &static_work, &adjacency_work);
    }
    const bool workspace_exact = workspace.passed
        && workspace.support.constraint == r30.constraint
        && al_r29_topology_root(workspace) == r30.topology_root
        && workspace.neighborhood.pairs.size() == 340340U
        && workspace.neighborhood.maximum_degree == 113U;

    std::size_t pair_passes = 0U;
    ALLinearizedJvp prefix_response;
    if (workspace_exact) {
        prefix_response = al_r29_pair_once_jvp(workspace, capture.iterate);
        ++pair_passes;
    }
    const double prefix_relative = prefix_response.passed
        ? al_r33_vec_diff_relative(
            capture.maintained_response, prefix_response.value)
        : std::numeric_limits<double>::infinity();
    const bool prefix_exact = prefix_response.passed
        && al_r10_dual_root(prefix_response.value)
            == "7dd519932782f9177be354418c9ecaaa1fed3baac1e8255305675da7cb59dff3"
        && std::isfinite(prefix_relative) && prefix_relative <= 1.0e-12;

    std::vector<Vec3> iterate = capture.iterate;
    std::vector<double> maintained_response = capture.maintained_response;
    std::vector<double> residual(r30.constraint.size(), 0.0);
    for (std::size_t row = 0U; row < residual.size(); ++row) {
        residual[row] = r30.constraint[row] + maintained_response[row];
    }
    std::vector<ALR34Checkpoint> checkpoints;
    std::size_t accepted_total = 8U;
    bool iteration_exact = prefix_exact;
    bool projected_stationary = capture.projected_stationary;
    bool saturated = false;
    for (std::size_t iteration = 8U;
         iteration < 32U && iteration_exact && !projected_stationary
             && !saturated;
         ++iteration) {
        std::vector<double> positive(residual.size(), 0.0);
        for (std::size_t row = 0U; row < residual.size(); ++row) {
            positive[row] = std::max(residual[row], 0.0);
        }
        const std::vector<Vec3> gradient =
            al_r29_pair_once_vjp(workspace, positive);
        ++pair_passes;
        const double gradient_norm = al_r32_vec3_norm(gradient);
        if (!al_r32_vec3_finite(gradient) || !std::isfinite(gradient_norm)) {
            iteration_exact = false;
            break;
        }
        if (gradient_norm == 0.0) {
            projected_stationary = true;
            break;
        }
        std::vector<Vec3> trial(iterate.size());
        for (std::size_t particle = 0U; particle < iterate.size(); ++particle) {
            trial[particle] = iterate[particle]
                - (1.0 / gradient_norm) * gradient[particle];
        }
        const std::vector<Vec3> target = al_r33_project_vec3(
            trial, AL_R32_TRUST_DELTA);
        if (target.size() != iterate.size()) {
            iteration_exact = false;
            break;
        }
        std::vector<Vec3> direction(iterate.size());
        for (std::size_t particle = 0U; particle < iterate.size(); ++particle) {
            direction[particle] = target[particle] - iterate[particle];
        }
        const double chord_norm = al_r32_vec3_norm(direction);
        if (chord_norm == 0.0) {
            projected_stationary = true;
            break;
        }
        for (Vec3& value : direction) value = (1.0 / chord_norm) * value;
        iteration_exact = al_r32_vec3_finite(direction)
            && std::abs(al_r32_vec3_norm(direction) - 1.0) <= 1.0e-12;
        if (!iteration_exact) break;
        const ALLinearizedJvp response =
            al_r29_pair_once_jvp(workspace, direction);
        ++pair_passes;
        if (!response.passed) {
            iteration_exact = false;
            break;
        }
        const ALR32LineResult line = al_r32_piecewise_line(
            residual, response.value, chord_norm);
        const long double adjoint_left = al_r30_dot(positive, response.value);
        long double adjoint_right = 0.0L;
        for (std::size_t particle = 0U; particle < gradient.size(); ++particle) {
            adjoint_right += static_cast<long double>(
                dot(gradient[particle], direction[particle]));
        }
        const double adjoint_relative = static_cast<double>(
            std::abs(adjoint_left - adjoint_right)
            / std::max(std::abs(adjoint_right), 1.0e-300L));
        const double derivative_scale = std::max(
            std::abs(line.initial.derivative), 1.0e-300);
        const bool line_kkt = line.trust_active
            ? line.final.derivative <= 0.0
            : std::abs(line.final.derivative) / derivative_scale <= 1.0e-10;
        iteration_exact = line.passed && line_kkt
            && std::isfinite(adjoint_relative) && adjoint_relative <= 1.0e-12;
        if (!iteration_exact) break;
        if (!(line.final.phi < line.initial.phi)) {
            saturated = true;
            break;
        }
        for (std::size_t particle = 0U; particle < iterate.size(); ++particle) {
            iterate[particle] += line.alpha * direction[particle];
        }
        for (std::size_t row = 0U; row < residual.size(); ++row) {
            maintained_response[row] += line.alpha * response.value[row];
            residual[row] = r30.constraint[row] + maintained_response[row];
        }
        ++accepted_total;
        const double iterate_norm = al_r32_vec3_norm(iterate);
        iteration_exact = iterate_norm <= std::nextafter(
            AL_R32_TRUST_DELTA, std::numeric_limits<double>::infinity());
        if (accepted_total == 16U || accepted_total == 32U) {
            ALR34Checkpoint checkpoint;
            checkpoint.iteration = accepted_total;
            checkpoint.phi = line.final.phi;
            checkpoint.violation_norm = line.final.positive_norm;
            checkpoint.step_norm = line.alpha;
            checkpoint.iterate_norm = iterate_norm;
            checkpoint.gradient_norm = gradient_norm;
            checkpoint.active = line.final.active;
            checkpoint.line_root = line.root;
            std::ostringstream projection;
            projection << checkpoint.iteration << ':'
                       << binary64_bits(checkpoint.phi) << ':'
                       << binary64_bits(checkpoint.violation_norm) << ':'
                       << binary64_bits(checkpoint.step_norm) << ':'
                       << binary64_bits(checkpoint.iterate_norm) << ':'
                       << binary64_bits(checkpoint.gradient_norm) << ':'
                       << checkpoint.active << ':' << checkpoint.line_root;
            checkpoint.root = sha256_hex(projection.str());
            checkpoints.push_back(std::move(checkpoint));
        }
    }
    const bool continuation_exact = iteration_exact
        && (projected_stationary || saturated
            || (accepted_total == 32U && checkpoints.size() == 2U));

    ALLinearizedJvp terminal_response;
    if (workspace_exact && continuation_exact) {
        terminal_response = al_r29_pair_once_jvp(workspace, iterate);
        ++pair_passes;
    }
    const double response_relative = terminal_response.passed
        ? al_r33_vec_diff_relative(
            maintained_response, terminal_response.value)
        : std::numeric_limits<double>::infinity();
    std::vector<double> terminal_residual(r30.constraint.size(), 0.0);
    std::vector<double> terminal_positive(r30.constraint.size(), 0.0);
    if (terminal_response.passed) {
        for (std::size_t row = 0U; row < terminal_residual.size(); ++row) {
            terminal_residual[row] =
                r30.constraint[row] + terminal_response.value[row];
            terminal_positive[row] = std::max(terminal_residual[row], 0.0);
        }
    }
    std::vector<Vec3> terminal_gradient;
    if (terminal_response.passed) {
        terminal_gradient = al_r29_pair_once_vjp(workspace, terminal_positive);
        ++pair_passes;
    }
    const std::vector<double> zero(r30.constraint.size(), 0.0);
    const ALR32Evaluation terminal = al_r32_evaluate(
        terminal_residual, zero, 0.0);
    const double terminal_gradient_norm = al_r32_vec3_norm(terminal_gradient);
    std::vector<Vec3> mapping_target(iterate.size());
    for (std::size_t particle = 0U;
         particle < iterate.size() && particle < terminal_gradient.size();
         ++particle) {
        mapping_target[particle] = iterate[particle] - terminal_gradient[particle];
    }
    mapping_target = al_r33_project_vec3(mapping_target, AL_R32_TRUST_DELTA);
    std::vector<Vec3> projected_mapping(iterate.size());
    if (mapping_target.size() == iterate.size()) {
        for (std::size_t particle = 0U; particle < iterate.size(); ++particle) {
            projected_mapping[particle] = iterate[particle]
                - mapping_target[particle];
        }
    }
    const double projected_mapping_norm = al_r32_vec3_norm(projected_mapping);
    const bool direct_exact = terminal_response.passed
        && terminal_gradient.size() == iterate.size()
        && al_r32_vec3_finite(terminal_gradient) && terminal.finite
        && std::isfinite(response_relative) && response_relative <= 1.0e-12
        && std::isfinite(terminal_gradient_norm)
        && std::isfinite(projected_mapping_norm)
        && mapping_target.size() == iterate.size();
    projected_stationary = projected_stationary
        || (direct_exact && projected_mapping_norm == 0.0);

    const double phi8 = capture.terminal.phi;
    const double violation8 = capture.terminal.positive_norm;
    const double phi16 = checkpoints.size() >= 1U ? checkpoints[0].phi : phi8;
    const double violation16 = checkpoints.size() >= 1U
        ? checkpoints[0].violation_norm : violation8;
    const double phi32 = checkpoints.size() >= 2U
        ? checkpoints[1].phi : terminal.phi;
    const double violation32 = checkpoints.size() >= 2U
        ? checkpoints[1].violation_norm : terminal.positive_norm;
    const double phi_ratio_16_8 = phi16 / phi8;
    const double phi_ratio_32_16 = phi32 / phi16;
    const double violation_ratio_16_8 = violation16 / violation8;
    const double violation_ratio_32_16 = violation32 / violation16;
    const double phi_geometric_8 = std::pow(phi_ratio_16_8, 1.0 / 8.0);
    const double phi_geometric_16 = std::pow(phi_ratio_32_16, 1.0 / 16.0);

    release_al_normalized_workspace(workspace, &work);
    const bool work_exact = pair_passes <= 51U
        && work.workspace_builds == 1U && work.workspace_releases == 1U
        && work.live_workspaces == 0U && work.maximum_live_workspaces == 1U
        && !work.lifecycle_underflow && work.all_pair_candidate_calls == 0U
        && static_work.static_index_builds == 1U
        && static_work.support_canonicalizations == 1U
        && static_work.workspace_builds == 1U
        && adjacency_work.workspace_builds == 1U;
    const bool rollback_exact = position_before
            == al_binary64_vec3_root(r30.position)
        && dual_before == al_r10_dual_root(r30.dual)
        && constraint_before == al_r10_dual_root(r30.constraint)
        && r30.violated_root == al_r29_mask_root(r30.violated);

    const std::array<std::pair<const char*, std::string>, 11U> routes{{
        {"EXTENDED_FIRST_ORDER_REFERENCE_CANDIDATE", al_r34_route(1,1,1,1,1,1,1,1,0,0)},
        {"EXTENDED_FIRST_ORDER_SATURATED", al_r34_route(1,1,1,1,1,1,1,1,0,1)},
        {"ALL_INEQUALITY_PROJECTED_STATIONARY", al_r34_route(1,1,1,1,1,1,1,1,1,0)},
        {"EXTENDED_PARENT_REJECTED", al_r34_route(0,1,1,1,1,1,1,1,0,0)},
        {"EXTENDED_SOURCE_REJECTED", al_r34_route(1,0,1,1,1,1,1,1,0,0)},
        {"EXTENDED_DENSE_REJECTED", al_r34_route(1,1,0,1,1,1,1,1,0,0)},
        {"EXTENDED_WORKSPACE_REJECTED", al_r34_route(1,1,1,0,1,1,1,1,0,0)},
        {"EXTENDED_PREFIX_REJECTED", al_r34_route(1,1,1,1,0,1,1,1,0,0)},
        {"EXTENDED_ITERATION_REJECTED", al_r34_route(1,1,1,1,1,0,1,1,0,0)},
        {"EXTENDED_DIRECT_REJECTED", al_r34_route(1,1,1,1,1,1,0,1,0,0)},
        {"EXTENDED_WORK_REJECTED", al_r34_route(1,1,1,1,1,1,1,0,0,0)},
    }};
    bool routes_exact = true;
    std::ostringstream route_projection;
    for (const auto& value : routes) {
        routes_exact = routes_exact && value.first == value.second;
        route_projection << value.first << ':' << value.second << '|';
    }
    const std::string route_root = sha256_hex(route_projection.str());
    const std::string route = al_r34_route(
        parent_exact, source_exact, dense_exact, workspace_exact, prefix_exact,
        continuation_exact, direct_exact && rollback_exact, work_exact,
        projected_stationary, saturated);
    const bool passed = identity_exact && parent_exact && source_exact
        && dense_exact && workspace_exact && prefix_exact
        && continuation_exact && direct_exact && work_exact && rollback_exact
        && routes_exact;
    std::string first_failure;
    if (!identity_exact) first_failure = "IDENTITY";
    else if (!parent_exact) first_failure = "PARENT";
    else if (!source_exact) first_failure = "SOURCE";
    else if (!dense_exact) first_failure = "DENSE";
    else if (!workspace_exact) first_failure = "WORKSPACE";
    else if (!prefix_exact) first_failure = "PREFIX";
    else if (!continuation_exact) first_failure = "ITERATION";
    else if (!direct_exact) first_failure = "DIRECT";
    else if (!work_exact) first_failure = "WORK";
    else if (!rollback_exact) first_failure = "ROLLBACK";
    else if (!routes_exact) first_failure = "ROUTES";

    std::ostringstream checkpoint_projection;
    for (const ALR34Checkpoint& checkpoint : checkpoints) {
        checkpoint_projection << checkpoint.root << '|';
    }
    const std::string checkpoint_root =
        sha256_hex(checkpoint_projection.str());
    const std::string terminal_response_root = terminal_response.passed
        ? al_r10_dual_root(terminal_response.value) : sha256_hex("missing");
    const std::string terminal_gradient_root =
        al_binary64_vec3_root(terminal_gradient);
    const std::string projected_mapping_root =
        al_binary64_vec3_root(projected_mapping);
    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R34_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << dense_root << ':' << checkpoint_root << ':'
             << terminal_response_root << ':' << terminal_gradient_root << ':'
             << projected_mapping_root << '|' << accepted_total << ':'
             << binary64_bits(terminal.phi) << ':'
             << binary64_bits(terminal.positive_norm) << ':'
             << binary64_bits(response_relative) << ':' << pair_passes << '|'
             << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4e2d7r19r34_first_order_reference_extension.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R34_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source\":{\"checkpoint_root\":\""
           << parent_checkpoint_root << "\",\"prefix_response_root\":\""
           << (prefix_response.passed
               ? al_r10_dual_root(prefix_response.value) : sha256_hex("missing"))
           << "\",\"prefix_response_relative\":" << prefix_relative << "}"
           << ",\"dense\":{\"passed\":"
           << (dense_exact ? "true" : "false")
           << ",\"root\":\"" << dense_root << "\"}"
           << ",\"continuation\":{\"accepted_total\":" << accepted_total
           << ",\"stationary\":"
           << (projected_stationary ? "true" : "false")
           << ",\"saturated\":" << (saturated ? "true" : "false")
           << ",\"checkpoints\":[";
    for (std::size_t index_value = 0U;
         index_value < checkpoints.size(); ++index_value) {
        if (index_value != 0U) report << ',';
        const ALR34Checkpoint& checkpoint = checkpoints[index_value];
        report << "{\"iteration\":" << checkpoint.iteration
               << ",\"phi\":" << checkpoint.phi
               << ",\"violation_norm\":" << checkpoint.violation_norm
               << ",\"step_norm\":" << checkpoint.step_norm
               << ",\"iterate_norm\":" << checkpoint.iterate_norm
               << ",\"gradient_norm\":" << checkpoint.gradient_norm
               << ",\"active\":" << checkpoint.active
               << ",\"root\":\"" << checkpoint.root << "\"}";
    }
    report << "],\"checkpoint_root\":\"" << checkpoint_root
           << "\",\"phi_ratio_16_8\":" << phi_ratio_16_8
           << ",\"phi_ratio_32_16\":" << phi_ratio_32_16
           << ",\"violation_ratio_16_8\":" << violation_ratio_16_8
           << ",\"violation_ratio_32_16\":" << violation_ratio_32_16
           << ",\"phi_geometric_per_step_8\":" << phi_geometric_8
           << ",\"phi_geometric_per_step_16\":" << phi_geometric_16 << "}"
           << ",\"terminal\":{\"phi\":" << terminal.phi
           << ",\"violation_norm\":" << terminal.positive_norm
           << ",\"active\":" << terminal.active
           << ",\"iterate_norm\":" << al_r32_vec3_norm(iterate)
           << ",\"gradient_norm\":" << terminal_gradient_norm
           << ",\"projected_mapping_norm\":" << projected_mapping_norm
           << ",\"response_relative\":" << response_relative
           << ",\"response_root\":\"" << terminal_response_root
           << "\",\"gradient_root\":\"" << terminal_gradient_root
           << "\",\"projected_mapping_root\":\""
           << projected_mapping_root << "\"}"
           << ",\"work\":{\"parent_r33_replays\":1"
           << ",\"diagnostic_workspaces\":1,\"new_pair_passes\":"
           << pair_passes << ",\"new_hvp\":0,\"new_model\":0"
              ",\"new_trials\":0,\"new_outer\":0}"
           << ",\"route_cases\":" << routes.size()
           << ",\"route_root\":\"" << route_root << "\""
           << ",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":"
           << (rollback_exact ? "true" : "false")
           << ",\"first_failure\":\"" << first_failure << "\""
           << ",\"route\":\"" << route << "\""
              ",\"correction_applied\":false,\"nonlinear_evaluation\":false"
              ",\"floor_classification\":false,\"state_mutation\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false"
           << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {passed, report.str()};
}
