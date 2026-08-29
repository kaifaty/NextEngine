namespace {

constexpr const char* B4E2D7R19R38_IDENTITY_SHA256 =
    "8ebdf87b7d9635b15d276f0b05e11cb5d81e5909ea96c12f80458c64cc1f6aa1";
constexpr const char* B4E2D7R19R38_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r38-direction-memory-replay|v1|parent=c30febef:f475c20302c444be1ca355a1c9f0ec39309bbe7cd33f4db73fad676bef1c3571:c6a254243c8d549f7f20322ed61321eb94ff3d551e75c850d9b1a549a2d14118|source=r37-checkpoints6,12,24;e404d5997578df09bab907e16f37e2344f4c754cb7951fad2aa01a0c43f31f4e;terminal-response-ff2ba63971c215a505b6b2c8b26b941410e631fa33110227e39560e67f3056de;gradient-a7cd95da69de32c3a50a6c1b856a7eda5248598ef835d460d38815c99819e675;mapping-64f004ed0238c57b1be68bb6b741673944a5a38a304da9e588e1fcc4d9549f85|lanes=steepest;prp-plus;dy-hs-plus;hager-zhang-raw|direction=raw-minus-gradient-plus-beta-previous-reference-direction;normalize;project-global-l2-ball;normalize-chord|restart=invalid-denominator-or-beta;nonfinite;raw-nondescent;projected-nondescent;fallback-steepest|line=exact-r32-piecewise-all-row-hinge;direct-kkt;no-update|selection=all-three-valid-no-restart-strict-objective-dominance-over-steepest;precedence-hz,dy-hs-plus,prp-plus,steepest|work=states3*(source-jvp1+lanes4*jvp1);pair-pass-cap15;vjp0;hvp0|controls=dense-spd-exact-line-equivalence;dense-active-switch-restart;parent;source;workspace;state;direction;line;work;rollback|routes=memory-parent-rejected;memory-source-rejected;memory-dense-rejected;memory-workspace-rejected;memory-state-rejected;memory-direction-rejected;memory-line-rejected;memory-work-rejected;hager-zhang-direction-candidate;dy-hs-plus-direction-candidate;prp-plus-direction-candidate;steepest-reference-retained|runs=2-clean-release-builds;1-process-each;byte-exact|recurrence=none;accepted-step=none;correction=none;nonlinear-evaluation=none;tolerance=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-direction-memory-replay-classification-only)IDENTITY";

enum class ALR38LaneKind {
    Steepest,
    PrpPlus,
    DyHsPlus,
    HagerZhang,
};

struct ALR38Beta {
    bool valid = false;
    double beta = 0.0;
    double denominator = 0.0;
    std::string root;
};

struct ALR38Direction {
    bool passed = false;
    bool restarted = false;
    std::string restart_reason;
    ALR38Beta beta;
    double raw_descent = 0.0;
    double projected_descent = 0.0;
    double chord_norm = 0.0;
    std::vector<Vec3> direction;
    std::string root;
};

struct ALR38LaneRecord {
    std::size_t state_step = 0U;
    std::string name;
    ALR38Direction direction;
    double line_alpha = 0.0;
    double initial_phi = 0.0;
    double final_phi = 0.0;
    double final_violation = 0.0;
    std::size_t final_active = 0U;
    std::string line_root;
    std::string root;
};

struct ALR38ReplayCapture {
    bool captured = false;
    ALR37ContinuationCapture r37;
};

const char* al_r38_lane_name(ALR38LaneKind kind) {
    switch (kind) {
    case ALR38LaneKind::Steepest: return "STEEPEST";
    case ALR38LaneKind::PrpPlus: return "PRP_PLUS";
    case ALR38LaneKind::DyHsPlus: return "DY_HS_PLUS";
    case ALR38LaneKind::HagerZhang: return "HAGER_ZHANG";
    }
    return "UNKNOWN";
}

ALR38Beta al_r38_beta(
    ALR38LaneKind kind,
    const std::vector<Vec3>& gradient,
    const std::vector<Vec3>& previous_gradient,
    const std::vector<Vec3>& previous_direction) {
    ALR38Beta result;
    if (kind == ALR38LaneKind::Steepest) {
        result.valid = true;
    } else if (gradient.size() == previous_gradient.size()
            && gradient.size() == previous_direction.size()
            && !gradient.empty()) {
        std::vector<Vec3> y(gradient.size());
        for (std::size_t index = 0U; index < gradient.size(); ++index) {
            y[index] = gradient[index] - previous_gradient[index];
        }
        const long double g_y = al_r35_dot_vec3(gradient, y);
        const long double g_g = al_r35_dot_vec3(gradient, gradient);
        const long double gp_gp =
            al_r35_dot_vec3(previous_gradient, previous_gradient);
        const long double d_y = al_r35_dot_vec3(previous_direction, y);
        const long double y_y = al_r35_dot_vec3(y, y);
        long double beta = std::numeric_limits<long double>::quiet_NaN();
        long double denominator = 0.0L;
        if (kind == ALR38LaneKind::PrpPlus) {
            denominator = gp_gp;
            if (denominator != 0.0L) {
                beta = std::max(0.0L, g_y / denominator);
            }
        } else if (kind == ALR38LaneKind::DyHsPlus) {
            denominator = d_y;
            if (denominator != 0.0L) {
                const long double hs = g_y / denominator;
                const long double dy = g_g / denominator;
                beta = std::max(0.0L, std::min(hs, dy));
            }
        } else if (kind == ALR38LaneKind::HagerZhang) {
            denominator = d_y;
            if (denominator != 0.0L) {
                std::vector<Vec3> corrected_y = y;
                const long double scale = 2.0L * y_y / denominator;
                for (std::size_t index = 0U; index < y.size(); ++index) {
                    corrected_y[index] = y[index]
                        - static_cast<double>(scale) * previous_direction[index];
                }
                beta = al_r35_dot_vec3(corrected_y, gradient) / denominator;
            }
        }
        result.denominator = static_cast<double>(denominator);
        result.beta = static_cast<double>(beta);
        result.valid = std::isfinite(result.denominator)
            && result.denominator != 0.0 && std::isfinite(result.beta);
    }
    std::ostringstream projection;
    projection << al_r38_lane_name(kind) << ':' << result.valid << ':'
               << binary64_bits(result.beta) << ':'
               << binary64_bits(result.denominator);
    result.root = sha256_hex(projection.str());
    return result;
}

bool al_r38_project_direction(
    const std::vector<Vec3>& iterate,
    const std::vector<Vec3>& raw,
    std::vector<Vec3>* output_direction,
    double* output_chord_norm) {
    if (iterate.size() != raw.size() || raw.empty()
        || output_direction == nullptr || output_chord_norm == nullptr) {
        return false;
    }
    const double raw_norm = al_r32_vec3_norm(raw);
    if (!std::isfinite(raw_norm) || raw_norm == 0.0) return false;
    std::vector<Vec3> target(iterate.size());
    for (std::size_t index = 0U; index < iterate.size(); ++index) {
        target[index] = iterate[index] + (1.0 / raw_norm) * raw[index];
    }
    target = al_r33_project_vec3(target, AL_R32_TRUST_DELTA);
    if (target.size() != iterate.size()) return false;
    std::vector<Vec3> chord(iterate.size());
    for (std::size_t index = 0U; index < iterate.size(); ++index) {
        chord[index] = target[index] - iterate[index];
    }
    const double chord_norm = al_r32_vec3_norm(chord);
    if (!std::isfinite(chord_norm) || chord_norm == 0.0) return false;
    for (Vec3& value : chord) value = (1.0 / chord_norm) * value;
    *output_direction = std::move(chord);
    *output_chord_norm = chord_norm;
    return true;
}

ALR38Direction al_r38_direction(
    ALR38LaneKind kind,
    const ALR37ReplayState& state) {
    ALR38Direction result;
    result.beta = al_r38_beta(
        kind, state.gradient, state.previous_gradient,
        state.previous_direction);
    std::vector<Vec3> raw(state.gradient.size());
    auto set_steepest = [&]() {
        for (std::size_t index = 0U; index < raw.size(); ++index) {
            raw[index] = -state.gradient[index];
        }
    };
    if (result.beta.valid) {
        for (std::size_t index = 0U; index < raw.size(); ++index) {
            raw[index] = -state.gradient[index]
                + result.beta.beta * state.previous_direction[index];
        }
    } else {
        result.restarted = kind != ALR38LaneKind::Steepest;
        result.restart_reason = "INVALID_BETA";
        set_steepest();
    }
    result.raw_descent = static_cast<double>(
        al_r35_dot_vec3(state.gradient, raw));
    if (!std::isfinite(result.raw_descent) || result.raw_descent >= 0.0) {
        result.restarted = kind != ALR38LaneKind::Steepest;
        result.restart_reason = "RAW_NON_DESCENT";
        set_steepest();
        result.raw_descent = static_cast<double>(
            al_r35_dot_vec3(state.gradient, raw));
    }
    bool projected = al_r38_project_direction(
        state.iterate, raw, &result.direction, &result.chord_norm);
    result.projected_descent = projected
        ? static_cast<double>(
            al_r35_dot_vec3(state.gradient, result.direction))
        : std::numeric_limits<double>::quiet_NaN();
    if ((!projected || !std::isfinite(result.projected_descent)
            || result.projected_descent >= 0.0)
        && kind != ALR38LaneKind::Steepest) {
        result.restarted = true;
        result.restart_reason = "PROJECTED_NON_DESCENT";
        set_steepest();
        projected = al_r38_project_direction(
            state.iterate, raw, &result.direction, &result.chord_norm);
        result.projected_descent = projected
            ? static_cast<double>(
                al_r35_dot_vec3(state.gradient, result.direction))
            : std::numeric_limits<double>::quiet_NaN();
    }
    result.passed = projected && al_r32_vec3_finite(result.direction)
        && std::isfinite(result.raw_descent) && result.raw_descent < 0.0
        && std::isfinite(result.projected_descent)
        && result.projected_descent < 0.0;
    std::ostringstream projection;
    projection << al_r38_lane_name(kind) << ':' << result.passed << ':'
               << result.restarted << ':' << result.restart_reason << ':'
               << result.beta.root << ':' << binary64_bits(result.raw_descent)
               << ':' << binary64_bits(result.projected_descent) << ':'
               << binary64_bits(result.chord_norm) << ':'
               << al_binary64_vec3_root(result.direction);
    result.root = sha256_hex(projection.str());
    return result;
}

bool al_r38_dense_controls(std::string* output_root) {
    const std::vector<Vec3> previous_gradient{{2.0, 4.0, 0.0}};
    const std::vector<Vec3> previous_direction{{-2.0, -4.0, 0.0}};
    const std::vector<Vec3> gradient{{8.0 / 9.0, -4.0 / 9.0, 0.0}};
    const ALR38Beta prp = al_r38_beta(
        ALR38LaneKind::PrpPlus, gradient, previous_gradient,
        previous_direction);
    const ALR38Beta hybrid = al_r38_beta(
        ALR38LaneKind::DyHsPlus, gradient, previous_gradient,
        previous_direction);
    const ALR38Beta hz = al_r38_beta(
        ALR38LaneKind::HagerZhang, gradient, previous_gradient,
        previous_direction);
    std::vector<Vec3> prp_direction(1U);
    std::vector<Vec3> hybrid_direction(1U);
    std::vector<Vec3> hz_direction(1U);
    prp_direction[0] = -gradient[0] + prp.beta * previous_direction[0];
    hybrid_direction[0] =
        -gradient[0] + hybrid.beta * previous_direction[0];
    hz_direction[0] = -gradient[0] + hz.beta * previous_direction[0];
    const double equivalence = std::max({
        al_r32_vec3_norm({prp_direction[0] - hybrid_direction[0]}),
        al_r32_vec3_norm({prp_direction[0] - hz_direction[0]}),
        al_r32_vec3_norm({hybrid_direction[0] - hz_direction[0]})});
    const ALR38Beta invalid = al_r38_beta(
        ALR38LaneKind::DyHsPlus,
        {{1.0, 0.0, 0.0}}, {{1.0, 0.0, 0.0}}, {{0.0, 1.0, 0.0}});
    const ALR32LineResult active_line = al_r32_piecewise_line(
        {1.0, -0.1}, {-1.0, 1.0}, 1.0);
    const bool exact = prp.valid && hybrid.valid && hz.valid
        && equivalence <= 1.0e-14
        && al_r35_dot_vec3(gradient, prp_direction) < 0.0L
        && !invalid.valid && active_line.passed
        && binary64_bits(active_line.alpha) == binary64_bits(0.55)
        && active_line.final.active == 2U;
    std::ostringstream projection;
    projection << prp.root << ':' << hybrid.root << ':' << hz.root << ':'
               << invalid.root << ':' << active_line.root << ':'
               << binary64_bits(equivalence) << ':' << exact;
    if (output_root != nullptr) *output_root = sha256_hex(projection.str());
    return exact;
}

std::string al_r38_route(
    bool parent, bool source, bool dense, bool workspace, bool state,
    bool direction, bool line, bool work, bool hz, bool hybrid, bool prp) {
    if (!parent) return "MEMORY_PARENT_REJECTED";
    if (!source) return "MEMORY_SOURCE_REJECTED";
    if (!dense) return "MEMORY_DENSE_REJECTED";
    if (!workspace) return "MEMORY_WORKSPACE_REJECTED";
    if (!state) return "MEMORY_STATE_REJECTED";
    if (!direction) return "MEMORY_DIRECTION_REJECTED";
    if (!line) return "MEMORY_LINE_REJECTED";
    if (!work) return "MEMORY_WORK_REJECTED";
    if (hz) return "HAGER_ZHANG_DIRECTION_CANDIDATE";
    if (hybrid) return "DY_HS_PLUS_DIRECTION_CANDIDATE";
    if (prp) return "PRP_PLUS_DIRECTION_CANDIDATE";
    return "STEEPEST_REFERENCE_RETAINED";
}

} // namespace

SplitBoundaryReport run_al_direction_memory_replay_controls_impl(
    ALR38ReplayCapture* output_capture) {
    const bool identity_exact = sha256_hex(B4E2D7R19R38_IDENTITY_PROJECTION)
        == B4E2D7R19R38_IDENTITY_SHA256;
    ALR37ContinuationCapture capture;
    const SplitBoundaryReport parent =
        run_al_hybrid_polish_continuation_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "c6a254243c8d549f7f20322ed61321eb94ff3d551e75c850d9b1a549a2d14118"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && parent_semantic && parent_stdout
            == "f475c20302c444be1ca355a1c9f0ec39309bbe7cd33f4db73fad676bef1c3571";
    ALR30RangeProjectionCapture& r30 =
        capture.r36.r35.r34.r33.r32.r31.r30;
    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string dual_before = al_r10_dual_root(r30.dual);
    const std::string constraint_before = al_r10_dual_root(r30.constraint);
    std::ostringstream source_checkpoint_projection;
    for (const ALR37ReplayState& state : capture.replay_states) {
        source_checkpoint_projection << state.checkpoint.root << '|';
    }
    const std::string source_checkpoint_root =
        sha256_hex(source_checkpoint_projection.str());
    const bool source_exact = parent_exact
        && capture.replay_states.size() == 3U
        && capture.replay_states[0].checkpoint.additional_step == 6U
        && capture.replay_states[1].checkpoint.additional_step == 12U
        && capture.replay_states[2].checkpoint.additional_step == 24U
        && source_checkpoint_root
            == "e404d5997578df09bab907e16f37e2344f4c754cb7951fad2aa01a0c43f31f4e"
        && capture.replay_states[2].checkpoint.response_root
            == "ff2ba63971c215a505b6b2c8b26b941410e631fa33110227e39560e67f3056de"
        && capture.replay_states[2].checkpoint.gradient_root
            == "a7cd95da69de32c3a50a6c1b856a7eda5248598ef835d460d38815c99819e675"
        && capture.replay_states[2].checkpoint.mapping_root
            == "64f004ed0238c57b1be68bb6b741673944a5a38a304da9e588e1fcc4d9549f85";

    std::string dense_root;
    const bool dense_exact = al_r38_dense_controls(&dense_root);
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

    constexpr std::array<ALR38LaneKind, 4U> kinds{{
        ALR38LaneKind::Steepest,
        ALR38LaneKind::PrpPlus,
        ALR38LaneKind::DyHsPlus,
        ALR38LaneKind::HagerZhang,
    }};
    std::size_t pair_passes = 0U;
    bool state_exact = workspace_exact;
    bool direction_exact = workspace_exact;
    bool line_exact = workspace_exact;
    std::vector<ALR38LaneRecord> records;
    for (const ALR37ReplayState& state : capture.replay_states) {
        const ALLinearizedJvp direct_response =
            al_r29_pair_once_jvp(workspace, state.iterate);
        ++pair_passes;
        const double response_relative = direct_response.passed
            ? al_r33_vec_diff_relative(
                state.maintained_response, direct_response.value)
            : std::numeric_limits<double>::infinity();
        const std::vector<Vec3> mapping =
            al_r37_projected_mapping(state.iterate, state.gradient);
        const ALR37Checkpoint captured_checkpoint = al_r37_checkpoint(
            state.checkpoint.additional_step, state.iterate,
            state.maintained_response, state.residual, state.gradient, mapping);
        state_exact = state_exact && direct_response.passed
            && std::isfinite(response_relative) && response_relative <= 1.0e-12
            && captured_checkpoint.root == state.checkpoint.root
            && state.previous_gradient.size() == state.gradient.size()
            && state.previous_direction.size() == state.gradient.size();
        for (ALR38LaneKind kind : kinds) {
            ALR38LaneRecord record;
            record.state_step = state.checkpoint.additional_step;
            record.name = al_r38_lane_name(kind);
            record.direction = al_r38_direction(kind, state);
            direction_exact = direction_exact && record.direction.passed;
            const ALLinearizedJvp line_response = al_r29_pair_once_jvp(
                workspace, record.direction.direction);
            ++pair_passes;
            const ALR32LineResult line = al_r32_piecewise_line(
                state.residual, line_response.value,
                record.direction.chord_norm);
            const double derivative_scale = std::max(
                std::abs(line.initial.derivative), 1.0e-300);
            const bool line_kkt = line.trust_active
                ? line.final.derivative <= 0.0
                : std::abs(line.final.derivative) / derivative_scale <= 1.0e-10;
            const bool lane_line_exact = line_response.passed && line.passed
                && line_kkt && line.final.phi < line.initial.phi;
            line_exact = line_exact && lane_line_exact;
            record.line_alpha = line.alpha;
            record.initial_phi = line.initial.phi;
            record.final_phi = line.final.phi;
            record.final_violation = line.final.positive_norm;
            record.final_active = line.final.active;
            record.line_root = line.root;
            std::ostringstream projection;
            projection << record.state_step << ':' << record.name << ':'
                       << record.direction.root << ':'
                       << binary64_bits(record.line_alpha) << ':'
                       << binary64_bits(record.initial_phi) << ':'
                       << binary64_bits(record.final_phi) << ':'
                       << binary64_bits(record.final_violation) << ':'
                       << record.final_active << ':' << record.line_root;
            record.root = sha256_hex(projection.str());
            records.push_back(std::move(record));
        }
    }

    auto lane_selected = [&](const std::string& name) {
        bool selected = records.size() == 12U;
        for (std::size_t state_index = 0U; state_index < 3U; ++state_index) {
            const ALR38LaneRecord& steepest = records[state_index * 4U];
            const auto found = std::find_if(
                records.begin() + static_cast<std::ptrdiff_t>(state_index * 4U),
                records.begin() + static_cast<std::ptrdiff_t>(state_index * 4U + 4U),
                [&](const ALR38LaneRecord& value) { return value.name == name; });
            selected = selected
                && found != records.begin()
                    + static_cast<std::ptrdiff_t>(state_index * 4U + 4U)
                && found->direction.beta.valid && !found->direction.restarted
                && found->direction.raw_descent < 0.0
                && found->direction.projected_descent < 0.0
                && found->final_phi < steepest.final_phi;
        }
        return selected;
    };
    const bool hz_selected = lane_selected("HAGER_ZHANG");
    const bool hybrid_selected = lane_selected("DY_HS_PLUS");
    const bool prp_selected = lane_selected("PRP_PLUS");

    release_al_normalized_workspace(workspace, &work);
    const bool work_exact = pair_passes == 15U
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
        {"HAGER_ZHANG_DIRECTION_CANDIDATE", al_r38_route(1,1,1,1,1,1,1,1,1,0,0)},
        {"DY_HS_PLUS_DIRECTION_CANDIDATE", al_r38_route(1,1,1,1,1,1,1,1,0,1,0)},
        {"PRP_PLUS_DIRECTION_CANDIDATE", al_r38_route(1,1,1,1,1,1,1,1,0,0,1)},
        {"STEEPEST_REFERENCE_RETAINED", al_r38_route(1,1,1,1,1,1,1,1,0,0,0)},
        {"MEMORY_PARENT_REJECTED", al_r38_route(0,1,1,1,1,1,1,1,0,0,0)},
        {"MEMORY_SOURCE_REJECTED", al_r38_route(1,0,1,1,1,1,1,1,0,0,0)},
        {"MEMORY_DENSE_REJECTED", al_r38_route(1,1,0,1,1,1,1,1,0,0,0)},
        {"MEMORY_WORKSPACE_REJECTED", al_r38_route(1,1,1,0,1,1,1,1,0,0,0)},
        {"MEMORY_STATE_REJECTED", al_r38_route(1,1,1,1,0,1,1,1,0,0,0)},
        {"MEMORY_DIRECTION_REJECTED", al_r38_route(1,1,1,1,1,0,1,1,0,0,0)},
        {"MEMORY_LINE_REJECTED", al_r38_route(1,1,1,1,1,1,0,1,0,0,0)},
        {"MEMORY_WORK_REJECTED", al_r38_route(1,1,1,1,1,1,1,0,0,0,0)},
    }};
    bool routes_exact = true;
    std::ostringstream route_projection;
    for (const auto& value : routes) {
        routes_exact = routes_exact && value.first == value.second;
        route_projection << value.first << ':' << value.second << '|';
    }
    const std::string route_root = sha256_hex(route_projection.str());
    const std::string route = al_r38_route(
        parent_exact, source_exact, dense_exact, workspace_exact, state_exact,
        direction_exact, line_exact, work_exact && rollback_exact,
        hz_selected, hybrid_selected, prp_selected);
    const bool passed = identity_exact && parent_exact && source_exact
        && dense_exact && workspace_exact && state_exact && direction_exact
        && line_exact && work_exact && rollback_exact && routes_exact;
    std::string first_failure;
    if (!identity_exact) first_failure = "IDENTITY";
    else if (!parent_exact) first_failure = "PARENT";
    else if (!source_exact) first_failure = "SOURCE";
    else if (!dense_exact) first_failure = "DENSE";
    else if (!workspace_exact) first_failure = "WORKSPACE";
    else if (!state_exact) first_failure = "STATE";
    else if (!direction_exact) first_failure = "DIRECTION";
    else if (!line_exact) first_failure = "LINE";
    else if (!work_exact) first_failure = "WORK";
    else if (!rollback_exact) first_failure = "ROLLBACK";
    else if (!routes_exact) first_failure = "ROUTES";

    std::ostringstream record_projection;
    for (const ALR38LaneRecord& record : records) {
        record_projection << record.root << '|';
    }
    const std::string record_root = sha256_hex(record_projection.str());
    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R38_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << dense_root << ':' << source_checkpoint_root << ':'
             << record_root << ':' << pair_passes << '|'
             << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4e2d7r19r38_direction_memory_replay.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R38_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source_checkpoint_root\":\""
           << source_checkpoint_root << "\""
           << ",\"dense\":{\"passed\":"
           << (dense_exact ? "true" : "false")
           << ",\"root\":\"" << dense_root << "\"}"
           << ",\"replay\":{\"states\":3,\"lanes\":4,\"records\":[";
    for (std::size_t index_value = 0U; index_value < records.size(); ++index_value) {
        if (index_value != 0U) report << ',';
        const ALR38LaneRecord& record = records[index_value];
        report << "{\"state_step\":" << record.state_step
               << ",\"name\":\"" << record.name << "\""
               << ",\"beta_valid\":"
               << (record.direction.beta.valid ? "true" : "false")
               << ",\"beta\":" << record.direction.beta.beta
               << ",\"denominator\":" << record.direction.beta.denominator
               << ",\"restarted\":"
               << (record.direction.restarted ? "true" : "false")
               << ",\"restart_reason\":\""
               << record.direction.restart_reason << "\""
               << ",\"raw_descent\":" << record.direction.raw_descent
               << ",\"projected_descent\":"
               << record.direction.projected_descent
               << ",\"line_alpha\":" << record.line_alpha
               << ",\"initial_phi\":" << record.initial_phi
               << ",\"final_phi\":" << record.final_phi
               << ",\"final_violation\":" << record.final_violation
               << ",\"final_active\":" << record.final_active
               << ",\"root\":\"" << record.root << "\"}";
    }
    report << "],\"record_root\":\"" << record_root << "\"}"
           << ",\"selection\":{\"hager_zhang\":"
           << (hz_selected ? "true" : "false")
           << ",\"dy_hs_plus\":"
           << (hybrid_selected ? "true" : "false")
           << ",\"prp_plus\":" << (prp_selected ? "true" : "false")
           << "}"
           << ",\"work\":{\"parent_r37_replays\":1"
           << ",\"diagnostic_workspaces\":1,\"new_pair_passes\":"
           << pair_passes << ",\"new_vjp\":0,\"new_hvp\":0"
              ",\"new_model\":0,\"new_trials\":0,\"new_outer\":0}"
           << ",\"route_cases\":" << routes.size()
           << ",\"route_root\":\"" << route_root << "\""
           << ",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":"
           << (rollback_exact ? "true" : "false")
           << ",\"first_failure\":\"" << first_failure << "\""
           << ",\"route\":\"" << route << "\""
              ",\"recurrence_executed\":false,\"accepted_step\":false"
              ",\"correction_applied\":false,\"nonlinear_evaluation\":false"
              ",\"tolerance_selected\":false,\"state_mutation\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false"
           << ",\"result_sha256\":\"" << result_sha << "\"}";
    if (output_capture != nullptr) {
        output_capture->captured = passed;
        output_capture->r37 = std::move(capture);
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_direction_memory_replay_controls() {
    return run_al_direction_memory_replay_controls_impl(nullptr);
}
