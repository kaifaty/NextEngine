namespace {

constexpr const char* B4E2D7R19R37_IDENTITY_SHA256 =
    "746d699d1f724f584dcd156838ea1500118e81000ee19a9d49d85de1aa3a574d";
constexpr const char* B4E2D7R19R37_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r37-hybrid-polish-continuation|v1|parent=eb50f25c:13566a2dc1ed03336e906ba68a6a2c80af84cb98a38981e61e22595b50c07550:6efda6f07993f4e4cf067c894bd2ffee7b0bf3f10585ad043aba6054008bae99|source=r36-endpoint;response-3bcaf5f90958f743eda5a4dccad83cc1b844a3c12083ac15292cb8f79a08e84b;gradient-f4bf70ad53ee6410a21d756e80558fe01d1eb47423fbec8dcd277d138d0b0558;mapping-54d472d43715c3592cc1e01283ddc79709e810ed23994f8f0f26e87dbf450e78;phi-3.5527997925597766e-17;violation-8.4294718607511539e-9;mapping-norm-7.4709090069224081e-10|model=all-row-squared-hinge;trust-global-l2-delta0.25|continuation=unchanged-r33-projected-normalized-steepest;exact-r32-piecewise-line;additional-max24;checkpoints6,12,24;exact-zero-projected-stationarity|terminal=fresh-jvp;fresh-vjp;response-relative<=1e-12;checkpoint-objective-violation-mapping-block-contractions;no-fitted-tolerance|work=prefix1+polish48+terminal2;pair-pass-cap51;hvp0|controls=dense-conditioned-continuation;dense-active-switch;dense-stationary;parent;source;workspace;prefix;iteration;checkpoint;direct;work;rollback|routes=continuation-parent-rejected;continuation-source-rejected;continuation-dense-rejected;continuation-workspace-rejected;continuation-prefix-rejected;continuation-iteration-rejected;continuation-checkpoint-rejected;continuation-direct-rejected;continuation-work-rejected;hybrid-polish-projected-stationary;hybrid-polish-mapping-regressed;hybrid-polish-continuation-candidate|runs=2-clean-release-builds;1-process-each;byte-exact|correction=none;nonlinear-evaluation=none;tolerance-selection=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-hybrid-polish-termination-curve-only)IDENTITY";

struct ALR37Checkpoint {
    std::size_t additional_step = 0U;
    double phi = 0.0;
    double violation_norm = 0.0;
    double mapping_norm = 0.0;
    double gradient_norm = 0.0;
    double iterate_norm = 0.0;
    std::size_t active = 0U;
    std::string response_root;
    std::string gradient_root;
    std::string mapping_root;
    std::string root;
};

struct ALR37ReplayState {
    ALR37Checkpoint checkpoint;
    std::vector<Vec3> iterate;
    std::vector<double> maintained_response;
    std::vector<double> residual;
    std::vector<Vec3> gradient;
    std::vector<Vec3> previous_gradient;
    std::vector<Vec3> previous_direction;
};

struct ALR37ContinuationCapture {
    bool captured = false;
    ALR36HybridCapture r36;
    std::vector<ALR37ReplayState> replay_states;
};

std::vector<Vec3> al_r37_projected_mapping(
    const std::vector<Vec3>& iterate,
    const std::vector<Vec3>& gradient) {
    if (iterate.size() != gradient.size()) return {};
    std::vector<Vec3> target(iterate.size());
    for (std::size_t particle = 0U; particle < iterate.size(); ++particle) {
        target[particle] = iterate[particle] - gradient[particle];
    }
    target = al_r33_project_vec3(target, AL_R32_TRUST_DELTA);
    if (target.size() != iterate.size()) return {};
    std::vector<Vec3> mapping(iterate.size());
    for (std::size_t particle = 0U; particle < iterate.size(); ++particle) {
        mapping[particle] = iterate[particle] - target[particle];
    }
    return mapping;
}

ALR37Checkpoint al_r37_checkpoint(
    std::size_t additional_step,
    const std::vector<Vec3>& iterate,
    const std::vector<double>& response,
    const std::vector<double>& residual,
    const std::vector<Vec3>& gradient,
    const std::vector<Vec3>& mapping) {
    ALR37Checkpoint result;
    result.additional_step = additional_step;
    const std::vector<double> zero(residual.size(), 0.0);
    const ALR32Evaluation evaluation = al_r32_evaluate(residual, zero, 0.0);
    result.phi = evaluation.phi;
    result.violation_norm = evaluation.positive_norm;
    result.mapping_norm = al_r32_vec3_norm(mapping);
    result.gradient_norm = al_r32_vec3_norm(gradient);
    result.iterate_norm = al_r32_vec3_norm(iterate);
    result.active = evaluation.active;
    result.response_root = al_r10_dual_root(response);
    result.gradient_root = al_binary64_vec3_root(gradient);
    result.mapping_root = al_binary64_vec3_root(mapping);
    std::ostringstream projection;
    projection << result.additional_step << ':'
               << binary64_bits(result.phi) << ':'
               << binary64_bits(result.violation_norm) << ':'
               << binary64_bits(result.mapping_norm) << ':'
               << binary64_bits(result.gradient_norm) << ':'
               << binary64_bits(result.iterate_norm) << ':'
               << result.active << ':' << result.response_root << ':'
               << result.gradient_root << ':' << result.mapping_root;
    result.root = sha256_hex(projection.str());
    return result;
}

bool al_r37_dense_controls(std::string* output_root) {
    ALR33DenseControl conditioned_short = al_r33_dense_run(
        "r37-conditioned-short",
        {{1.0, 0.0}, {-1.0, 0.0}, {0.0, 8.0}, {0.0, -8.0}},
        {1.1, 0.9, 1.1, 0.9}, 2.0, 6U);
    ALR33DenseControl conditioned_long = al_r33_dense_run(
        "r37-conditioned-long",
        {{1.0, 0.0}, {-1.0, 0.0}, {0.0, 8.0}, {0.0, -8.0}},
        {1.1, 0.9, 1.1, 0.9}, 2.0, 30U);
    ALR33DenseControl active_switch = al_r33_dense_run(
        "r37-active-switch", {{-1.0}, {1.0}}, {1.0, -0.1}, 1.0, 24U);
    ALR33DenseControl stationary = al_r33_dense_run(
        "r37-exact-stationary", {{1.0}}, {-1.0}, 1.0, 24U);
    conditioned_long.passed = conditioned_short.passed
        && conditioned_long.passed && conditioned_short.accepted == 6U
        && conditioned_long.accepted == 30U
        && conditioned_long.phi < conditioned_short.phi;
    active_switch.passed = active_switch.passed && active_switch.stationary
        && active_switch.accepted == 1U && active_switch.active == 2U;
    stationary.passed = stationary.passed && stationary.stationary
        && stationary.accepted == 0U && stationary.phi == 0.0;
    const bool exact = conditioned_long.passed && active_switch.passed
        && stationary.passed;
    std::ostringstream projection;
    projection << conditioned_short.root << ':' << conditioned_long.root
               << ':' << active_switch.root << ':' << stationary.root << ':'
               << exact;
    if (output_root != nullptr) *output_root = sha256_hex(projection.str());
    return exact;
}

std::string al_r37_route(
    bool parent, bool source, bool dense, bool workspace, bool prefix,
    bool iteration, bool checkpoint, bool direct, bool work, bool stationary,
    bool mapping_improved) {
    if (!parent) return "CONTINUATION_PARENT_REJECTED";
    if (!source) return "CONTINUATION_SOURCE_REJECTED";
    if (!dense) return "CONTINUATION_DENSE_REJECTED";
    if (!workspace) return "CONTINUATION_WORKSPACE_REJECTED";
    if (!prefix) return "CONTINUATION_PREFIX_REJECTED";
    if (!iteration) return "CONTINUATION_ITERATION_REJECTED";
    if (!checkpoint) return "CONTINUATION_CHECKPOINT_REJECTED";
    if (!direct) return "CONTINUATION_DIRECT_REJECTED";
    if (!work) return "CONTINUATION_WORK_REJECTED";
    if (stationary) return "HYBRID_POLISH_PROJECTED_STATIONARY";
    return mapping_improved
        ? "HYBRID_POLISH_CONTINUATION_CANDIDATE"
        : "HYBRID_POLISH_MAPPING_REGRESSED";
}

} // namespace

SplitBoundaryReport run_al_hybrid_polish_continuation_controls_impl(
    ALR37ContinuationCapture* output_capture) {
    const bool identity_exact = sha256_hex(B4E2D7R19R37_IDENTITY_PROJECTION)
        == B4E2D7R19R37_IDENTITY_SHA256;
    ALR36HybridCapture capture;
    const SplitBoundaryReport parent =
        run_al_equal_work_curvature_polish_hybrid_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "6efda6f07993f4e4cf067c894bd2ffee7b0bf3f10585ad043aba6054008bae99"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && parent_semantic && parent_stdout
            == "13566a2dc1ed03336e906ba68a6a2c80af84cb98a38981e61e22595b50c07550";
    ALR30RangeProjectionCapture& r30 =
        capture.r35.r34.r33.r32.r31.r30;
    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string dual_before = al_r10_dual_root(r30.dual);
    const std::string constraint_before = al_r10_dual_root(r30.constraint);
    const double parent_mapping_norm =
        al_r32_vec3_norm(capture.projected_mapping);
    const bool source_exact = parent_exact
        && al_r10_dual_root(capture.terminal_response)
            == "3bcaf5f90958f743eda5a4dccad83cc1b844a3c12083ac15292cb8f79a08e84b"
        && al_binary64_vec3_root(capture.terminal_gradient)
            == "f4bf70ad53ee6410a21d756e80558fe01d1eb47423fbec8dcd277d138d0b0558"
        && al_binary64_vec3_root(capture.projected_mapping)
            == "54d472d43715c3592cc1e01283ddc79709e810ed23994f8f0f26e87dbf450e78"
        && binary64_bits(capture.terminal.phi)
            == binary64_bits(3.5527997925597766e-17)
        && binary64_bits(capture.terminal.positive_norm)
            == binary64_bits(8.4294718607511539e-9)
        && binary64_bits(parent_mapping_norm)
            == binary64_bits(7.4709090069224081e-10)
        && capture.iterate.size() == 6000U
        && capture.maintained_response.size() == 6000U;

    std::string dense_root;
    const bool dense_exact = al_r37_dense_controls(&dense_root);
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
            == "3bcaf5f90958f743eda5a4dccad83cc1b844a3c12083ac15292cb8f79a08e84b"
        && std::isfinite(prefix_relative) && prefix_relative <= 1.0e-12;

    std::vector<Vec3> iterate = capture.iterate;
    std::vector<double> maintained_response = capture.maintained_response;
    std::vector<double> residual(r30.constraint.size(), 0.0);
    for (std::size_t row = 0U; row < residual.size(); ++row) {
        residual[row] = r30.constraint[row] + maintained_response[row];
    }
    std::vector<ALR37Checkpoint> checkpoints;
    std::vector<ALR37ReplayState> replay_states;
    std::vector<Vec3> previous_gradient;
    std::vector<Vec3> previous_direction;
    std::size_t accepted = 0U;
    bool iteration_exact = prefix_exact;
    bool projected_stationary = false;
    for (std::size_t iteration = 0U;
         iteration < 24U && iteration_exact && !projected_stationary;
         ++iteration) {
        std::vector<double> positive(residual.size(), 0.0);
        for (std::size_t row = 0U; row < residual.size(); ++row) {
            positive[row] = std::max(residual[row], 0.0);
        }
        const std::vector<Vec3> gradient =
            al_r29_pair_once_vjp(workspace, positive);
        ++pair_passes;
        const double gradient_norm = al_r32_vec3_norm(gradient);
        const std::vector<Vec3> mapping =
            al_r37_projected_mapping(iterate, gradient);
        const double mapping_norm = al_r32_vec3_norm(mapping);
        if (!al_r32_vec3_finite(gradient) || !std::isfinite(gradient_norm)
            || mapping.size() != iterate.size()
            || !std::isfinite(mapping_norm)) {
            iteration_exact = false;
            break;
        }
        if (accepted == 6U || accepted == 12U) {
            ALR37Checkpoint checkpoint = al_r37_checkpoint(
                accepted, iterate, maintained_response, residual,
                gradient, mapping);
            checkpoints.push_back(checkpoint);
            replay_states.push_back({
                std::move(checkpoint), iterate, maintained_response, residual,
                gradient, previous_gradient, previous_direction});
        }
        if (mapping_norm == 0.0) {
            projected_stationary = true;
            break;
        }
        if (gradient_norm == 0.0) {
            iteration_exact = false;
            break;
        }
        std::vector<Vec3> target(iterate.size());
        for (std::size_t particle = 0U; particle < iterate.size(); ++particle) {
            target[particle] = iterate[particle]
                - (1.0 / gradient_norm) * gradient[particle];
        }
        target = al_r33_project_vec3(target, AL_R32_TRUST_DELTA);
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
        const ALLinearizedJvp line_response =
            al_r29_pair_once_jvp(workspace, direction);
        ++pair_passes;
        if (!line_response.passed) {
            iteration_exact = false;
            break;
        }
        const ALR32LineResult line = al_r32_piecewise_line(
            residual, line_response.value, chord_norm);
        const double derivative_scale = std::max(
            std::abs(line.initial.derivative), 1.0e-300);
        const bool line_kkt = line.trust_active
            ? line.final.derivative <= 0.0
            : std::abs(line.final.derivative) / derivative_scale <= 1.0e-10;
        iteration_exact = line.passed && line_kkt
            && line.final.phi < line.initial.phi;
        if (!iteration_exact) break;
        for (std::size_t particle = 0U; particle < iterate.size(); ++particle) {
            iterate[particle] += line.alpha * direction[particle];
        }
        for (std::size_t row = 0U; row < residual.size(); ++row) {
            maintained_response[row] += line.alpha * line_response.value[row];
            residual[row] = r30.constraint[row] + maintained_response[row];
        }
        previous_gradient = gradient;
        previous_direction = direction;
        ++accepted;
        iteration_exact = al_r32_vec3_norm(iterate)
            <= std::nextafter(AL_R32_TRUST_DELTA,
                std::numeric_limits<double>::infinity());
    }
    iteration_exact = iteration_exact
        && (projected_stationary || accepted == 24U);

    ALLinearizedJvp terminal_response;
    if (workspace_exact && iteration_exact) {
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
    const std::vector<Vec3> terminal_mapping =
        al_r37_projected_mapping(iterate, terminal_gradient);
    const double terminal_mapping_norm =
        al_r32_vec3_norm(terminal_mapping);
    const std::vector<double> zero(r30.constraint.size(), 0.0);
    const ALR32Evaluation terminal = al_r32_evaluate(
        terminal_residual, zero, 0.0);
    const double terminal_gradient_norm = al_r32_vec3_norm(terminal_gradient);
    const bool direct_exact = terminal_response.passed
        && terminal_gradient.size() == iterate.size()
        && terminal_mapping.size() == iterate.size()
        && al_r32_vec3_finite(terminal_gradient)
        && al_r32_vec3_finite(terminal_mapping) && terminal.finite
        && std::isfinite(response_relative) && response_relative <= 1.0e-12
        && std::isfinite(terminal_gradient_norm)
        && std::isfinite(terminal_mapping_norm);
    projected_stationary = direct_exact && terminal_mapping_norm == 0.0;
    if (direct_exact && accepted == 24U) {
        ALR37Checkpoint checkpoint = al_r37_checkpoint(
            accepted, iterate, terminal_response.value, terminal_residual,
            terminal_gradient, terminal_mapping);
        checkpoints.push_back(checkpoint);
        replay_states.push_back({
            std::move(checkpoint), iterate, terminal_response.value,
            terminal_residual, terminal_gradient, previous_gradient,
            previous_direction});
    }

    bool checkpoint_exact = direct_exact;
    if (projected_stationary && accepted < 24U) {
        checkpoint_exact = checkpoint_exact && !checkpoints.empty();
    } else {
        checkpoint_exact = checkpoint_exact && checkpoints.size() == 3U
            && checkpoints[0].additional_step == 6U
            && checkpoints[1].additional_step == 12U
            && checkpoints[2].additional_step == 24U;
    }
    double previous_phi = capture.terminal.phi;
    double previous_violation = capture.terminal.positive_norm;
    for (const ALR37Checkpoint& checkpoint : checkpoints) {
        checkpoint_exact = checkpoint_exact
            && std::isfinite(checkpoint.phi)
            && std::isfinite(checkpoint.violation_norm)
            && std::isfinite(checkpoint.mapping_norm)
            && checkpoint.phi < previous_phi
            && checkpoint.violation_norm < previous_violation;
        previous_phi = checkpoint.phi;
        previous_violation = checkpoint.violation_norm;
    }
    const bool mapping_improved = direct_exact
        && terminal_mapping_norm < parent_mapping_norm;

    release_al_normalized_workspace(workspace, &work);
    const bool work_exact = pair_passes <= 51U
        && (accepted < 24U || pair_passes == 51U)
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

    const std::array<std::pair<const char*, std::string>, 12U> routes{{
        {"HYBRID_POLISH_CONTINUATION_CANDIDATE", al_r37_route(1,1,1,1,1,1,1,1,1,0,1)},
        {"HYBRID_POLISH_MAPPING_REGRESSED", al_r37_route(1,1,1,1,1,1,1,1,1,0,0)},
        {"HYBRID_POLISH_PROJECTED_STATIONARY", al_r37_route(1,1,1,1,1,1,1,1,1,1,0)},
        {"CONTINUATION_PARENT_REJECTED", al_r37_route(0,1,1,1,1,1,1,1,1,0,1)},
        {"CONTINUATION_SOURCE_REJECTED", al_r37_route(1,0,1,1,1,1,1,1,1,0,1)},
        {"CONTINUATION_DENSE_REJECTED", al_r37_route(1,1,0,1,1,1,1,1,1,0,1)},
        {"CONTINUATION_WORKSPACE_REJECTED", al_r37_route(1,1,1,0,1,1,1,1,1,0,1)},
        {"CONTINUATION_PREFIX_REJECTED", al_r37_route(1,1,1,1,0,1,1,1,1,0,1)},
        {"CONTINUATION_ITERATION_REJECTED", al_r37_route(1,1,1,1,1,0,1,1,1,0,1)},
        {"CONTINUATION_CHECKPOINT_REJECTED", al_r37_route(1,1,1,1,1,1,0,1,1,0,1)},
        {"CONTINUATION_DIRECT_REJECTED", al_r37_route(1,1,1,1,1,1,1,0,1,0,1)},
        {"CONTINUATION_WORK_REJECTED", al_r37_route(1,1,1,1,1,1,1,1,0,0,1)},
    }};
    bool routes_exact = true;
    std::ostringstream route_projection;
    for (const auto& value : routes) {
        routes_exact = routes_exact && value.first == value.second;
        route_projection << value.first << ':' << value.second << '|';
    }
    const std::string route_root = sha256_hex(route_projection.str());
    const std::string route = al_r37_route(
        parent_exact, source_exact, dense_exact, workspace_exact, prefix_exact,
        iteration_exact, checkpoint_exact, direct_exact && rollback_exact,
        work_exact, projected_stationary, mapping_improved);
    const bool passed = identity_exact && parent_exact && source_exact
        && dense_exact && workspace_exact && prefix_exact && iteration_exact
        && checkpoint_exact && direct_exact && work_exact && rollback_exact
        && routes_exact;
    std::string first_failure;
    if (!identity_exact) first_failure = "IDENTITY";
    else if (!parent_exact) first_failure = "PARENT";
    else if (!source_exact) first_failure = "SOURCE";
    else if (!dense_exact) first_failure = "DENSE";
    else if (!workspace_exact) first_failure = "WORKSPACE";
    else if (!prefix_exact) first_failure = "PREFIX";
    else if (!iteration_exact) first_failure = "ITERATION";
    else if (!checkpoint_exact) first_failure = "CHECKPOINT";
    else if (!direct_exact) first_failure = "DIRECT";
    else if (!work_exact) first_failure = "WORK";
    else if (!rollback_exact) first_failure = "ROLLBACK";
    else if (!routes_exact) first_failure = "ROUTES";

    std::ostringstream checkpoint_projection;
    for (const ALR37Checkpoint& checkpoint : checkpoints) {
        checkpoint_projection << checkpoint.root << '|';
    }
    const std::string checkpoint_root =
        sha256_hex(checkpoint_projection.str());
    const std::string terminal_response_root = terminal_response.passed
        ? al_r10_dual_root(terminal_response.value) : sha256_hex("missing");
    const std::string terminal_gradient_root =
        al_binary64_vec3_root(terminal_gradient);
    const std::string terminal_mapping_root =
        al_binary64_vec3_root(terminal_mapping);
    const double phi_ratio = terminal.phi / capture.terminal.phi;
    const double violation_ratio =
        terminal.positive_norm / capture.terminal.positive_norm;
    const double mapping_ratio = terminal_mapping_norm / parent_mapping_norm;
    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R37_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << dense_root << ':' << checkpoint_root << ':'
             << terminal_response_root << ':' << terminal_gradient_root << ':'
             << terminal_mapping_root << '|' << accepted << ':'
             << binary64_bits(phi_ratio) << ':'
             << binary64_bits(violation_ratio) << ':'
             << binary64_bits(mapping_ratio) << ':' << pair_passes << '|'
             << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4e2d7r19r37_hybrid_polish_continuation.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R37_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source\":{\"prefix_response_root\":\""
           << (prefix_response.passed
               ? al_r10_dual_root(prefix_response.value) : sha256_hex("missing"))
           << "\",\"prefix_response_relative\":" << prefix_relative << "}"
           << ",\"dense\":{\"passed\":"
           << (dense_exact ? "true" : "false")
           << ",\"root\":\"" << dense_root << "\"}"
           << ",\"continuation\":{\"accepted\":" << accepted
           << ",\"projected_stationary\":"
           << (projected_stationary ? "true" : "false")
           << ",\"checkpoints\":[";
    for (std::size_t index_value = 0U;
         index_value < checkpoints.size(); ++index_value) {
        if (index_value != 0U) report << ',';
        const ALR37Checkpoint& checkpoint = checkpoints[index_value];
        report << "{\"additional_step\":" << checkpoint.additional_step
               << ",\"phi\":" << checkpoint.phi
               << ",\"violation_norm\":" << checkpoint.violation_norm
               << ",\"mapping_norm\":" << checkpoint.mapping_norm
               << ",\"gradient_norm\":" << checkpoint.gradient_norm
               << ",\"iterate_norm\":" << checkpoint.iterate_norm
               << ",\"active\":" << checkpoint.active
               << ",\"root\":\"" << checkpoint.root << "\"}";
    }
    report << "],\"checkpoint_root\":\"" << checkpoint_root << "\"}"
           << ",\"comparison\":{\"parent_phi\":" << capture.terminal.phi
           << ",\"terminal_phi\":" << terminal.phi
           << ",\"phi_ratio\":" << phi_ratio
           << ",\"parent_violation_norm\":"
           << capture.terminal.positive_norm
           << ",\"terminal_violation_norm\":" << terminal.positive_norm
           << ",\"violation_ratio\":" << violation_ratio
           << ",\"parent_mapping_norm\":" << parent_mapping_norm
           << ",\"terminal_mapping_norm\":" << terminal_mapping_norm
           << ",\"mapping_ratio\":" << mapping_ratio
           << ",\"mapping_improved\":"
           << (mapping_improved ? "true" : "false") << "}"
           << ",\"terminal\":{\"active\":" << terminal.active
           << ",\"iterate_norm\":" << al_r32_vec3_norm(iterate)
           << ",\"gradient_norm\":" << terminal_gradient_norm
           << ",\"response_relative\":" << response_relative
           << ",\"response_root\":\"" << terminal_response_root
           << "\",\"gradient_root\":\"" << terminal_gradient_root
           << "\",\"projected_mapping_root\":\""
           << terminal_mapping_root << "\"}"
           << ",\"work\":{\"parent_r36_replays\":1"
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
              ",\"tolerance_selected\":false,\"state_mutation\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false"
           << ",\"result_sha256\":\"" << result_sha << "\"}";
    if (output_capture != nullptr) {
        output_capture->captured = passed;
        output_capture->r36 = std::move(capture);
        output_capture->replay_states = std::move(replay_states);
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_hybrid_polish_continuation_controls() {
    return run_al_hybrid_polish_continuation_controls_impl(nullptr);
}
