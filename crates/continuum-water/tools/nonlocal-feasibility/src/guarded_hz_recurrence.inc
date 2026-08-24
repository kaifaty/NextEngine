namespace {

constexpr const char* B4E2D7R19R39_IDENTITY_SHA256 =
    "acff94da15e985dad6695c64883893cdf3de6d571e9332e48f90b37b812b5fc2";
constexpr const char* B4E2D7R19R39_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r39-guarded-hz-recurrence|v1|parent=b36e0c33:7f574289496e70fb76886381c2b1460227f15a3f6d07214b0772dabf7247abb1:cc030c6559643952faf70dd3545f7f62a01133527ad5fdf8427159ed994ab7d3|source=r37-step24;checkpoint-463fc296b6adf7eb690c7ad4bbf05e07238424abe83ab0243cb6f7c28ac84be0;response-ff2ba63971c215a505b6b2c8b26b941410e631fa33110227e39560e67f3056de;gradient-a7cd95da69de32c3a50a6c1b856a7eda5248598ef835d460d38815c99819e675;mapping-64f004ed0238c57b1be68bb6b741673944a5a38a304da9e588e1fcc4d9549f85;r38-hz-record-185635ed433b785f5df56533c467a0ff2a13ab61c89ce9e8cddca9632777e18b|lanes=steepest;guarded-hager-zhang|history=previous-gradient-and-accepted-feasible-normalized-direction;restart-history-becomes-accepted-steepest|direction=r38-raw-hz;normalize;project-global-l2-ball;normalize-chord;restart-invalid-beta-or-nonfinite-or-raw-nondescent-or-projected-nondescent|line=exact-r32-piecewise-all-row-hinge;direct-kkt;accepted-linearized-lane-state-only|trajectory=max8;checkpoints1,2,4,8;exact-zero-only|selection=all-checkpoints-strict-objective-violation-mapping-dominance;memory-streak-at-least2;precedence=projected-stationary,guarded-hz-recurrence,hz-one-step-benefit-only,steepest-reference-retained|work=lanes2*(source-jvp1+line-jvp8+intermediate-vjp7+terminal-jvp1+terminal-vjp1);pair-pass-cap36;vjp16;jvp20;hvp0|controls=r38-dense-root;dense-spd-two-step-hz;parent;source;workspace;first-step;trajectory;checkpoint;direct;work;rollback|routes=recurrence-parent-rejected;recurrence-source-rejected;recurrence-dense-rejected;recurrence-workspace-rejected;recurrence-first-step-rejected;recurrence-trajectory-rejected;recurrence-checkpoint-rejected;recurrence-direct-rejected;recurrence-work-rejected;hz-projected-stationary;guarded-hz-recurrence-candidate;hz-one-step-benefit-only;steepest-reference-retained|runs=2-clean-release-builds;1-process-each;byte-exact|correction=none;nonlinear-evaluation=none;tolerance=none;public-state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-guarded-direction-recurrence-classification-only)IDENTITY";

enum class ALR39LaneKind {
    Steepest,
    GuardedHagerZhang,
};

struct ALR39Checkpoint {
    std::size_t accepted = 0U;
    ALR37Checkpoint state;
    std::size_t restarts = 0U;
    std::size_t memory_streak = 0U;
    std::string root;
};

struct ALR39Trajectory {
    bool passed = false;
    bool stationary = false;
    bool source_exact = false;
    bool iteration_exact = false;
    bool checkpoint_exact = false;
    bool direct_exact = false;
    bool first_step_exact = false;
    std::string name;
    std::size_t accepted = 0U;
    std::size_t restarts = 0U;
    std::size_t memory_steps = 0U;
    std::size_t memory_streak = 0U;
    std::size_t maximum_memory_streak = 0U;
    std::size_t pair_passes = 0U;
    std::size_t jvp_passes = 0U;
    std::size_t vjp_passes = 0U;
    double source_relative = std::numeric_limits<double>::infinity();
    double terminal_response_relative =
        std::numeric_limits<double>::infinity();
    std::string source_response_root;
    std::string first_record_root;
    std::vector<std::string> step_roots;
    std::vector<ALR39Checkpoint> checkpoints;
    ALR37Checkpoint terminal;
    std::string terminal_response_root;
    std::string terminal_gradient_root;
    std::string terminal_mapping_root;
    std::string root;
};

const char* al_r39_lane_name(ALR39LaneKind kind) {
    return kind == ALR39LaneKind::Steepest
        ? "STEEPEST" : "GUARDED_HAGER_ZHANG";
}

bool al_r39_checkpoint_requested(std::size_t accepted) {
    return accepted == 1U || accepted == 2U
        || accepted == 4U || accepted == 8U;
}

bool al_r39_dense_controls(std::string* output_root) {
    std::string inherited_root;
    const bool inherited = al_r38_dense_controls(&inherited_root)
        && inherited_root
            == "597f83607e042b7b5d11891c66390396e8428db4cbdd017ef7a628c1d69e512c";
    const auto apply = [](Vec3 value) {
        return Vec3{value.x, 8.0 * value.y, 0.0};
    };
    Vec3 x0{1.0, 1.0, 0.0};
    const Vec3 g0 = apply(x0);
    const Vec3 d0{-g0.x, -g0.y, 0.0};
    const double denominator0 = dot(d0, apply(d0));
    const double alpha0 = -dot(g0, d0) / denominator0;
    const Vec3 x1 = x0 + alpha0 * d0;
    const Vec3 g1 = apply(x1);
    const ALR38Beta beta = al_r38_beta(
        ALR38LaneKind::HagerZhang, {g1}, {g0}, {d0});
    const Vec3 d1{
        -g1.x + beta.beta * d0.x,
        -g1.y + beta.beta * d0.y,
        0.0};
    const double raw_descent = dot(g1, d1);
    const double denominator1 = dot(d1, apply(d1));
    const double alpha1 = -dot(g1, d1) / denominator1;
    const Vec3 x2 = x1 + alpha1 * d1;
    const Vec3 g2 = apply(x2);
    const double solution_defect = norm(x2);
    const double gradient_defect = norm(g2);
    const bool exact = inherited && beta.valid
        && std::isfinite(alpha0) && std::isfinite(alpha1)
        && alpha0 > 0.0 && alpha1 > 0.0 && raw_descent < 0.0
        && solution_defect <= 1.0e-13 && gradient_defect <= 1.0e-13;
    std::ostringstream projection;
    projection << inherited_root << ':' << beta.root << ':'
               << binary64_bits(alpha0) << ':' << binary64_bits(alpha1)
               << ':' << binary64_bits(raw_descent) << ':'
               << binary64_bits(solution_defect) << ':'
               << binary64_bits(gradient_defect) << ':' << exact;
    if (output_root != nullptr) *output_root = sha256_hex(projection.str());
    return exact;
}

std::string al_r39_r38_record_root(
    const ALR38Direction& direction,
    const ALR32LineResult& line) {
    std::ostringstream projection;
    projection << 24U << ':' << "HAGER_ZHANG" << ':' << direction.root << ':'
               << binary64_bits(line.alpha) << ':'
               << binary64_bits(line.initial.phi) << ':'
               << binary64_bits(line.final.phi) << ':'
               << binary64_bits(line.final.positive_norm) << ':'
               << line.final.active << ':' << line.root;
    return sha256_hex(projection.str());
}

ALR39Checkpoint al_r39_make_checkpoint(
    std::size_t accepted,
    const std::vector<Vec3>& iterate,
    const std::vector<double>& response,
    const std::vector<double>& residual,
    const std::vector<Vec3>& gradient,
    std::size_t restarts,
    std::size_t memory_streak) {
    const std::vector<Vec3> mapping =
        al_r37_projected_mapping(iterate, gradient);
    ALR39Checkpoint result;
    result.accepted = accepted;
    result.state = al_r37_checkpoint(
        accepted, iterate, response, residual, gradient, mapping);
    result.restarts = restarts;
    result.memory_streak = memory_streak;
    std::ostringstream projection;
    projection << result.accepted << ':' << result.state.root << ':'
               << result.restarts << ':' << result.memory_streak;
    result.root = sha256_hex(projection.str());
    return result;
}

ALR39Trajectory al_r39_run_trajectory(
    ALR39LaneKind kind,
    const ALNormalizedWorkspace& workspace,
    const std::vector<double>& constraint,
    const ALR37ReplayState& source) {
    ALR39Trajectory result;
    result.name = al_r39_lane_name(kind);
    ALLinearizedJvp source_response =
        al_r29_pair_once_jvp(workspace, source.iterate);
    ++result.pair_passes;
    ++result.jvp_passes;
    result.source_response_root = source_response.passed
        ? al_r10_dual_root(source_response.value) : sha256_hex("missing");
    result.source_relative = source_response.passed
        ? al_r33_vec_diff_relative(
            source.maintained_response, source_response.value)
        : std::numeric_limits<double>::infinity();
    result.source_exact = source_response.passed
        && result.source_response_root
            == "ff2ba63971c215a505b6b2c8b26b941410e631fa33110227e39560e67f3056de"
        && std::isfinite(result.source_relative)
        && result.source_relative <= 1.0e-12;

    std::vector<Vec3> iterate = source.iterate;
    std::vector<double> maintained_response = source.maintained_response;
    std::vector<double> residual = source.residual;
    std::vector<Vec3> gradient = source.gradient;
    std::vector<Vec3> previous_gradient = source.previous_gradient;
    std::vector<Vec3> previous_direction = source.previous_direction;
    result.iteration_exact = result.source_exact;
    while (result.iteration_exact && !result.stationary
           && result.accepted < 8U) {
        const std::vector<Vec3> mapping =
            al_r37_projected_mapping(iterate, gradient);
        const double mapping_norm = al_r32_vec3_norm(mapping);
        if (mapping.size() != iterate.size()
            || !al_r32_vec3_finite(gradient)
            || !al_r32_vec3_finite(mapping)
            || !std::isfinite(mapping_norm)) {
            result.iteration_exact = false;
            break;
        }
        if (mapping_norm == 0.0) {
            result.stationary = true;
            break;
        }
        ALR37ReplayState state;
        state.iterate = iterate;
        state.maintained_response = maintained_response;
        state.residual = residual;
        state.gradient = gradient;
        state.previous_gradient = previous_gradient;
        state.previous_direction = previous_direction;
        const ALR38LaneKind direction_kind =
            kind == ALR39LaneKind::Steepest
            ? ALR38LaneKind::Steepest : ALR38LaneKind::HagerZhang;
        const ALR38Direction direction =
            al_r38_direction(direction_kind, state);
        if (!direction.passed) {
            result.iteration_exact = false;
            break;
        }
        if (kind == ALR39LaneKind::GuardedHagerZhang) {
            if (direction.restarted) {
                ++result.restarts;
                result.memory_streak = 0U;
            } else {
                ++result.memory_steps;
                ++result.memory_streak;
                result.maximum_memory_streak = std::max(
                    result.maximum_memory_streak, result.memory_streak);
            }
        }
        const ALLinearizedJvp line_response = al_r29_pair_once_jvp(
            workspace, direction.direction);
        ++result.pair_passes;
        ++result.jvp_passes;
        const ALR32LineResult line = al_r32_piecewise_line(
            residual, line_response.value, direction.chord_norm);
        const double derivative_scale = std::max(
            std::abs(line.initial.derivative), 1.0e-300);
        const bool line_kkt = line.trust_active
            ? line.final.derivative <= 0.0
            : std::abs(line.final.derivative) / derivative_scale <= 1.0e-10;
        const bool line_exact = line_response.passed && line.passed
            && line_kkt && line.final.phi < line.initial.phi;
        if (!line_exact) {
            result.iteration_exact = false;
            break;
        }
        if (kind == ALR39LaneKind::GuardedHagerZhang
            && result.accepted == 0U) {
            result.first_record_root =
                al_r39_r38_record_root(direction, line);
            result.first_step_exact = result.first_record_root
                    == "185635ed433b785f5df56533c467a0ff2a13ab61c89ce9e8cddca9632777e18b"
                && binary64_bits(line.final.phi)
                    == binary64_bits(1.3684753550918572e-20)
                && binary64_bits(line.final.positive_norm)
                    == binary64_bits(1.6543732076480548e-10)
                && line.final.active == 260U;
        }
        std::ostringstream step_projection;
        step_projection << result.name << ':' << (result.accepted + 1U)
                        << ':' << direction.root << ':' << line.root << ':'
                        << binary64_bits(line.alpha) << ':'
                        << binary64_bits(line.final.phi) << ':'
                        << line.final.active;
        result.step_roots.push_back(sha256_hex(step_projection.str()));
        for (std::size_t particle = 0U; particle < iterate.size(); ++particle) {
            iterate[particle] += line.alpha * direction.direction[particle];
        }
        for (std::size_t row = 0U; row < residual.size(); ++row) {
            maintained_response[row] += line.alpha * line_response.value[row];
            residual[row] = constraint[row] + maintained_response[row];
        }
        previous_gradient = gradient;
        previous_direction = direction.direction;
        ++result.accepted;
        result.iteration_exact = al_r32_vec3_norm(iterate)
            <= std::nextafter(
                AL_R32_TRUST_DELTA, std::numeric_limits<double>::infinity());
        if (!result.iteration_exact) break;
        if (result.accepted < 8U) {
            std::vector<double> positive(residual.size(), 0.0);
            for (std::size_t row = 0U; row < residual.size(); ++row) {
                positive[row] = std::max(residual[row], 0.0);
            }
            gradient = al_r29_pair_once_vjp(workspace, positive);
            ++result.pair_passes;
            ++result.vjp_passes;
            result.iteration_exact = gradient.size() == iterate.size()
                && al_r32_vec3_finite(gradient);
            if (result.iteration_exact
                && al_r39_checkpoint_requested(result.accepted)) {
                result.checkpoints.push_back(al_r39_make_checkpoint(
                    result.accepted, iterate, maintained_response, residual,
                    gradient, result.restarts, result.memory_streak));
            }
        }
    }
    result.iteration_exact = result.iteration_exact
        && (result.stationary || result.accepted == 8U);

    ALLinearizedJvp terminal_response;
    std::vector<double> terminal_residual(constraint.size(), 0.0);
    std::vector<double> terminal_positive(constraint.size(), 0.0);
    std::vector<Vec3> terminal_gradient;
    std::vector<Vec3> terminal_mapping;
    if (result.iteration_exact) {
        terminal_response = al_r29_pair_once_jvp(workspace, iterate);
        ++result.pair_passes;
        ++result.jvp_passes;
    }
    if (terminal_response.passed) {
        result.terminal_response_relative = al_r33_vec_diff_relative(
            maintained_response, terminal_response.value);
        for (std::size_t row = 0U; row < constraint.size(); ++row) {
            terminal_residual[row] =
                constraint[row] + terminal_response.value[row];
            terminal_positive[row] = std::max(terminal_residual[row], 0.0);
        }
        terminal_gradient =
            al_r29_pair_once_vjp(workspace, terminal_positive);
        ++result.pair_passes;
        ++result.vjp_passes;
        terminal_mapping =
            al_r37_projected_mapping(iterate, terminal_gradient);
    }
    result.terminal_response_root = terminal_response.passed
        ? al_r10_dual_root(terminal_response.value) : sha256_hex("missing");
    result.terminal_gradient_root = al_binary64_vec3_root(terminal_gradient);
    result.terminal_mapping_root = al_binary64_vec3_root(terminal_mapping);
    result.direct_exact = terminal_response.passed
        && terminal_gradient.size() == iterate.size()
        && terminal_mapping.size() == iterate.size()
        && al_r32_vec3_finite(terminal_gradient)
        && al_r32_vec3_finite(terminal_mapping)
        && std::isfinite(result.terminal_response_relative)
        && result.terminal_response_relative <= 1.0e-12;
    if (result.direct_exact) {
        result.terminal = al_r37_checkpoint(
            result.accepted, iterate, terminal_response.value,
            terminal_residual, terminal_gradient, terminal_mapping);
        result.stationary = result.terminal.mapping_norm == 0.0;
        if (al_r39_checkpoint_requested(result.accepted)) {
            ALR39Checkpoint terminal_checkpoint;
            terminal_checkpoint.accepted = result.accepted;
            terminal_checkpoint.state = result.terminal;
            terminal_checkpoint.restarts = result.restarts;
            terminal_checkpoint.memory_streak = result.memory_streak;
            std::ostringstream projection;
            projection << terminal_checkpoint.accepted << ':'
                       << terminal_checkpoint.state.root << ':'
                       << terminal_checkpoint.restarts << ':'
                       << terminal_checkpoint.memory_streak;
            terminal_checkpoint.root = sha256_hex(projection.str());
            const auto existing = std::find_if(
                result.checkpoints.begin(), result.checkpoints.end(),
                [&](const ALR39Checkpoint& value) {
                    return value.accepted == result.accepted;
                });
            if (existing == result.checkpoints.end()) {
                result.checkpoints.push_back(std::move(terminal_checkpoint));
            } else {
                *existing = std::move(terminal_checkpoint);
            }
        }
    }
    result.checkpoint_exact = result.direct_exact
        && ((result.accepted == 8U && result.checkpoints.size() == 4U
             && result.checkpoints[0].accepted == 1U
             && result.checkpoints[1].accepted == 2U
             && result.checkpoints[2].accepted == 4U
             && result.checkpoints[3].accepted == 8U)
            || (result.stationary && result.accepted < 8U));
    const bool work_exact = result.accepted == 8U
        ? result.pair_passes == 18U && result.jvp_passes == 10U
            && result.vjp_passes == 8U
        : result.stationary && result.pair_passes <= 18U;
    if (kind == ALR39LaneKind::Steepest) {
        result.first_step_exact = true;
    }
    result.passed = result.source_exact && result.iteration_exact
        && result.checkpoint_exact && result.direct_exact
        && result.first_step_exact && work_exact;

    std::ostringstream projection;
    projection << result.name << ':' << result.passed << ':'
               << result.stationary << ':' << result.accepted << ':'
               << result.restarts << ':' << result.memory_steps << ':'
               << result.maximum_memory_streak << ':' << result.pair_passes
               << ':' << result.jvp_passes << ':' << result.vjp_passes << ':'
               << binary64_bits(result.source_relative) << ':'
               << binary64_bits(result.terminal_response_relative) << ':'
               << result.source_response_root << ':'
               << result.first_record_root << '|';
    for (const std::string& root : result.step_roots) {
        projection << root << '|';
    }
    for (const ALR39Checkpoint& checkpoint : result.checkpoints) {
        projection << checkpoint.root << '|';
    }
    projection << result.terminal_response_root << ':'
               << result.terminal_gradient_root << ':'
               << result.terminal_mapping_root;
    result.root = sha256_hex(projection.str());
    return result;
}

bool al_r39_checkpoint_dominance(
    const ALR39Checkpoint& candidate,
    const ALR39Checkpoint& reference) {
    return candidate.accepted == reference.accepted
        && candidate.state.phi < reference.state.phi
        && candidate.state.violation_norm < reference.state.violation_norm
        && candidate.state.mapping_norm < reference.state.mapping_norm;
}

std::string al_r39_route(
    bool parent, bool source, bool dense, bool workspace, bool first_step,
    bool trajectory, bool checkpoint, bool direct, bool work,
    bool stationary, bool recurrence, bool one_step_only) {
    if (!parent) return "RECURRENCE_PARENT_REJECTED";
    if (!source) return "RECURRENCE_SOURCE_REJECTED";
    if (!dense) return "RECURRENCE_DENSE_REJECTED";
    if (!workspace) return "RECURRENCE_WORKSPACE_REJECTED";
    if (!first_step) return "RECURRENCE_FIRST_STEP_REJECTED";
    if (!trajectory) return "RECURRENCE_TRAJECTORY_REJECTED";
    if (!checkpoint) return "RECURRENCE_CHECKPOINT_REJECTED";
    if (!direct) return "RECURRENCE_DIRECT_REJECTED";
    if (!work) return "RECURRENCE_WORK_REJECTED";
    if (stationary) return "HZ_PROJECTED_STATIONARY";
    if (recurrence) return "GUARDED_HZ_RECURRENCE_CANDIDATE";
    if (one_step_only) return "HZ_ONE_STEP_BENEFIT_ONLY";
    return "STEEPEST_REFERENCE_RETAINED";
}

} // namespace

SplitBoundaryReport run_al_guarded_hz_recurrence_controls() {
    const bool identity_exact = sha256_hex(B4E2D7R19R39_IDENTITY_PROJECTION)
        == B4E2D7R19R39_IDENTITY_SHA256;
    ALR38ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_direction_memory_replay_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "cc030c6559643952faf70dd3545f7f62a01133527ad5fdf8427159ed994ab7d3"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && capture.r37.captured && parent_semantic
        && parent_stdout
            == "7f574289496e70fb76886381c2b1460227f15a3f6d07214b0772dabf7247abb1";
    ALR30RangeProjectionCapture& r30 =
        capture.r37.r36.r35.r34.r33.r32.r31.r30;
    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string dual_before = al_r10_dual_root(r30.dual);
    const std::string constraint_before = al_r10_dual_root(r30.constraint);
    const bool has_source = capture.r37.replay_states.size() == 3U;
    const ALR37ReplayState* source = has_source
        ? &capture.r37.replay_states[2] : nullptr;
    const std::vector<Vec3> source_mapping = source != nullptr
        ? al_r37_projected_mapping(source->iterate, source->gradient)
        : std::vector<Vec3>{};
    const ALR37Checkpoint source_checkpoint = source != nullptr
        ? al_r37_checkpoint(
            24U, source->iterate, source->maintained_response,
            source->residual, source->gradient, source_mapping)
        : ALR37Checkpoint{};
    const bool source_exact = parent_exact && source != nullptr
        && source->checkpoint.additional_step == 24U
        && source_checkpoint.root
            == "463fc296b6adf7eb690c7ad4bbf05e07238424abe83ab0243cb6f7c28ac84be0"
        && source->checkpoint.root == source_checkpoint.root
        && source_checkpoint.response_root
            == "ff2ba63971c215a505b6b2c8b26b941410e631fa33110227e39560e67f3056de"
        && source_checkpoint.gradient_root
            == "a7cd95da69de32c3a50a6c1b856a7eda5248598ef835d460d38815c99819e675"
        && source_checkpoint.mapping_root
            == "64f004ed0238c57b1be68bb6b741673944a5a38a304da9e588e1fcc4d9549f85";

    std::string dense_root;
    const bool dense_exact = al_r39_dense_controls(&dense_root);
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

    ALR39Trajectory steepest;
    ALR39Trajectory hz;
    if (workspace_exact && source != nullptr) {
        steepest = al_r39_run_trajectory(
            ALR39LaneKind::Steepest, workspace, r30.constraint, *source);
        hz = al_r39_run_trajectory(
            ALR39LaneKind::GuardedHagerZhang,
            workspace, r30.constraint, *source);
    }
    const bool first_step_exact = hz.first_step_exact;
    const bool trajectory_exact = steepest.passed && hz.passed;
    bool checkpoint_exact = trajectory_exact
        && steepest.checkpoints.size() == hz.checkpoints.size()
        && steepest.checkpoints.size() == 4U;
    bool all_checkpoint_dominance = checkpoint_exact;
    for (std::size_t index_value = 0U;
         index_value < std::min(
             steepest.checkpoints.size(), hz.checkpoints.size());
         ++index_value) {
        checkpoint_exact = checkpoint_exact
            && steepest.checkpoints[index_value].accepted
                == hz.checkpoints[index_value].accepted;
        all_checkpoint_dominance = all_checkpoint_dominance
            && al_r39_checkpoint_dominance(
                hz.checkpoints[index_value], steepest.checkpoints[index_value]);
    }
    const bool direct_exact = trajectory_exact
        && steepest.direct_exact && hz.direct_exact;
    const bool stationary = hz.stationary;
    const bool recurrence_candidate = !stationary
        && all_checkpoint_dominance
        && hz.maximum_memory_streak >= 2U
        && hz.pair_passes <= steepest.pair_passes;
    const bool first_dominance = checkpoint_exact
        && al_r39_checkpoint_dominance(
            hz.checkpoints.front(), steepest.checkpoints.front());
    const bool terminal_dominance = checkpoint_exact
        && al_r39_checkpoint_dominance(
            hz.checkpoints.back(), steepest.checkpoints.back());
    const bool one_step_only = !stationary && !recurrence_candidate
        && hz.maximum_memory_streak < 2U
        && first_dominance && terminal_dominance;

    release_al_normalized_workspace(workspace, &work);
    const std::size_t pair_passes = steepest.pair_passes + hz.pair_passes;
    const std::size_t jvp_passes = steepest.jvp_passes + hz.jvp_passes;
    const std::size_t vjp_passes = steepest.vjp_passes + hz.vjp_passes;
    const bool trajectory_stationary = steepest.stationary || hz.stationary;
    const bool work_exact = pair_passes <= 36U
        && (trajectory_stationary
            || (pair_passes == 36U && jvp_passes == 20U
                && vjp_passes == 16U
                && steepest.pair_passes == hz.pair_passes))
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

    const std::array<std::pair<const char*, std::string>, 13U> routes{{
        {"HZ_PROJECTED_STATIONARY", al_r39_route(1,1,1,1,1,1,1,1,1,1,0,0)},
        {"GUARDED_HZ_RECURRENCE_CANDIDATE", al_r39_route(1,1,1,1,1,1,1,1,1,0,1,0)},
        {"HZ_ONE_STEP_BENEFIT_ONLY", al_r39_route(1,1,1,1,1,1,1,1,1,0,0,1)},
        {"STEEPEST_REFERENCE_RETAINED", al_r39_route(1,1,1,1,1,1,1,1,1,0,0,0)},
        {"RECURRENCE_PARENT_REJECTED", al_r39_route(0,1,1,1,1,1,1,1,1,0,0,0)},
        {"RECURRENCE_SOURCE_REJECTED", al_r39_route(1,0,1,1,1,1,1,1,1,0,0,0)},
        {"RECURRENCE_DENSE_REJECTED", al_r39_route(1,1,0,1,1,1,1,1,1,0,0,0)},
        {"RECURRENCE_WORKSPACE_REJECTED", al_r39_route(1,1,1,0,1,1,1,1,1,0,0,0)},
        {"RECURRENCE_FIRST_STEP_REJECTED", al_r39_route(1,1,1,1,0,1,1,1,1,0,0,0)},
        {"RECURRENCE_TRAJECTORY_REJECTED", al_r39_route(1,1,1,1,1,0,1,1,1,0,0,0)},
        {"RECURRENCE_CHECKPOINT_REJECTED", al_r39_route(1,1,1,1,1,1,0,1,1,0,0,0)},
        {"RECURRENCE_DIRECT_REJECTED", al_r39_route(1,1,1,1,1,1,1,0,1,0,0,0)},
        {"RECURRENCE_WORK_REJECTED", al_r39_route(1,1,1,1,1,1,1,1,0,0,0,0)},
    }};
    bool routes_exact = true;
    std::ostringstream route_projection;
    for (const auto& value : routes) {
        routes_exact = routes_exact && value.first == value.second;
        route_projection << value.first << ':' << value.second << '|';
    }
    const std::string route_root = sha256_hex(route_projection.str());
    const std::string route = al_r39_route(
        parent_exact, source_exact, dense_exact, workspace_exact,
        first_step_exact, trajectory_exact, checkpoint_exact, direct_exact,
        work_exact && rollback_exact, stationary, recurrence_candidate,
        one_step_only);
    const bool passed = identity_exact && parent_exact && source_exact
        && dense_exact && workspace_exact && first_step_exact
        && trajectory_exact && checkpoint_exact && direct_exact
        && work_exact && rollback_exact && routes_exact;
    std::string first_failure;
    if (!identity_exact) first_failure = "IDENTITY";
    else if (!parent_exact) first_failure = "PARENT";
    else if (!source_exact) first_failure = "SOURCE";
    else if (!dense_exact) first_failure = "DENSE";
    else if (!workspace_exact) first_failure = "WORKSPACE";
    else if (!first_step_exact) first_failure = "FIRST_STEP";
    else if (!trajectory_exact) first_failure = "TRAJECTORY";
    else if (!checkpoint_exact) first_failure = "CHECKPOINT";
    else if (!direct_exact) first_failure = "DIRECT";
    else if (!work_exact) first_failure = "WORK";
    else if (!rollback_exact) first_failure = "ROLLBACK";
    else if (!routes_exact) first_failure = "ROUTES";

    std::ostringstream checkpoint_projection;
    for (const ALR39Checkpoint& value : steepest.checkpoints) {
        checkpoint_projection << value.root << '|';
    }
    for (const ALR39Checkpoint& value : hz.checkpoints) {
        checkpoint_projection << value.root << '|';
    }
    const std::string checkpoint_root =
        sha256_hex(checkpoint_projection.str());
    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R39_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << dense_root << ':' << steepest.root << ':' << hz.root << ':'
             << checkpoint_root << '|' << pair_passes << ':' << jvp_passes
             << ':' << vjp_passes << '|' << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());

    const auto emit_trajectory = [](std::ostringstream& report,
                                    const ALR39Trajectory& value) {
        report << "{\"name\":\"" << value.name << "\""
               << ",\"passed\":" << (value.passed ? "true" : "false")
               << ",\"stationary\":"
               << (value.stationary ? "true" : "false")
               << ",\"accepted\":" << value.accepted
               << ",\"restarts\":" << value.restarts
               << ",\"memory_steps\":" << value.memory_steps
               << ",\"maximum_memory_streak\":"
               << value.maximum_memory_streak
               << ",\"source_relative\":" << value.source_relative
               << ",\"first_record_root\":\""
               << value.first_record_root << "\""
               << ",\"checkpoints\":[";
        for (std::size_t index_value = 0U;
             index_value < value.checkpoints.size(); ++index_value) {
            if (index_value != 0U) report << ',';
            const ALR39Checkpoint& checkpoint =
                value.checkpoints[index_value];
            report << "{\"accepted\":" << checkpoint.accepted
                   << ",\"phi\":" << checkpoint.state.phi
                   << ",\"violation_norm\":"
                   << checkpoint.state.violation_norm
                   << ",\"mapping_norm\":"
                   << checkpoint.state.mapping_norm
                   << ",\"active\":" << checkpoint.state.active
                   << ",\"restarts\":" << checkpoint.restarts
                   << ",\"memory_streak\":"
                   << checkpoint.memory_streak
                   << ",\"root\":\"" << checkpoint.root << "\"}";
        }
        report << "]"
               << ",\"terminal_response_relative\":"
               << value.terminal_response_relative
               << ",\"terminal_response_root\":\""
               << value.terminal_response_root << "\""
               << ",\"terminal_gradient_root\":\""
               << value.terminal_gradient_root << "\""
               << ",\"terminal_mapping_root\":\""
               << value.terminal_mapping_root << "\""
               << ",\"work\":{\"pair_passes\":" << value.pair_passes
               << ",\"jvp\":" << value.jvp_passes
               << ",\"vjp\":" << value.vjp_passes << "}"
               << ",\"root\":\"" << value.root << "\"}";
    };
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4e2d7r19r39_guarded_hz_recurrence.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\""
           << B4E2D7R19R39_IDENTITY_SHA256 << "\""
           << ",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source_checkpoint_root\":\""
           << source_checkpoint.root << "\""
           << ",\"dense\":{\"passed\":"
           << (dense_exact ? "true" : "false")
           << ",\"root\":\"" << dense_root << "\"}"
           << ",\"trajectories\":[";
    emit_trajectory(report, steepest);
    report << ',';
    emit_trajectory(report, hz);
    report << "]"
           << ",\"comparison\":{\"all_checkpoint_dominance\":"
           << (all_checkpoint_dominance ? "true" : "false")
           << ",\"recurrence_candidate\":"
           << (recurrence_candidate ? "true" : "false")
           << ",\"one_step_only\":"
           << (one_step_only ? "true" : "false")
           << ",\"checkpoint_root\":\"" << checkpoint_root << "\"}"
           << ",\"work\":{\"parent_r38_replays\":1"
           << ",\"diagnostic_workspaces\":1,\"new_pair_passes\":"
           << pair_passes << ",\"new_jvp\":" << jvp_passes
           << ",\"new_vjp\":" << vjp_passes
           << ",\"new_hvp\":0,\"new_model\":0"
              ",\"new_trials\":0,\"new_outer\":0}"
           << ",\"route_cases\":" << routes.size()
           << ",\"route_root\":\"" << route_root << "\""
           << ",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":"
           << (rollback_exact ? "true" : "false")
           << ",\"first_failure\":\"" << first_failure << "\""
           << ",\"route\":\"" << route << "\""
              ",\"correction_applied\":false"
              ",\"nonlinear_evaluation\":false"
              ",\"tolerance_selected\":false"
              ",\"state_mutation\":false"
              ",\"timing_admitted\":false"
              ",\"runtime_authority\":false"
              ",\"production_authority\":false"
           << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {passed, report.str()};
}
