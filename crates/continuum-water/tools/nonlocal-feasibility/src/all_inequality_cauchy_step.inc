namespace {

constexpr const char* B4E2D7R19R32_IDENTITY_SHA256 =
    "5e69a197102f8b4e1d86d68ec49f1da55f03bf754fb96d25d056a8edd5588b1d";
constexpr const char* B4E2D7R19R32_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r32-all-inequality-cauchy-normal-step|v1|parent=546035f6:67adbee532f615b9014e93cb44fccd053bc887c54f5d2ffcd76a19f2c771f3cb:d57f2712573d8e8dda52bdf44a11f40e2a6cbca03b1ac30e07c13b20af8aaa98|source=state-851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7;constraint-c567599060d7d00ffec4928009772327d081d3ef294d9030bfc5499b82e533c9;topology-070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5;violated-bd06f7b7c5e1f1943e5c269bbfb192229504091381b8f20f0f4988eecb021c70;particles6000;pairs340340;max-degree113|model=phi(v)=0.5*norm(max(c+A*v,0))2;A=SPACING*Jc;all-rows;support-fixed|trust=global-l2;delta0.25;inherited-inner-initial-radius0.25*SPACING|direction=g=At*max(c,0);d=-g/norm(g);unit-l2|line=alpha-in-[0,delta];events=-c/(A*d);binary128-order;stable-row-tie;fixed-long-double-fold;exact-piecewise-quadratic-minimum|direct=fresh-gradient-response;phi0;phialpha;directional-derivative;interior-kkt-relative<=1e-10;boundary-derivative<=0;positive-reduction|controls=dense-interior;dense-boundary;dense-inactive-entry;dense-simultaneous-event;parent;source;workspace;gradient;event-order;direct-kkt;work;rollback|observations=gradient-root;direction-root;response-root;events-root;alpha;trust-active;phi-reduction;violation-norm-ratio;initial-final-active;entered-left;selected-inactive-positive;step-rms-max|routes=cauchy-parent-rejected;cauchy-source-rejected;cauchy-dense-rejected;cauchy-workspace-rejected;cauchy-gradient-rejected;cauchy-line-rejected;cauchy-kkt-rejected;cauchy-work-rejected;all-inequality-linearized-stationary;all-inequality-cauchy-normal-step-candidate|precedence=parent,source,dense,workspace,gradient,line,kkt,work,classification|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-r31-once;diagnostic-workspaces1;new-pair-passes2;new-hvp0;new-model0;new-trial0;new-outer0|correction=none;nonlinear-evaluation=none;floor-classification=none;state-mutation=none;following-outer=none;substep=none;macro=none;trajectory=none;timing=none;public-schema=none;runtime=none;production=none|credit=one-private-all-inequality-cauchy-normal-step-classification-only)IDENTITY";

constexpr double AL_R32_TRUST_DELTA = 0.25;

struct ALR32Event {
    Binary128 alpha = static_cast<Binary128>(0.0);
    std::size_t row = 0U;
    bool entering = false;
};

struct ALR32Evaluation {
    bool finite = false;
    double phi = 0.0;
    double derivative = 0.0;
    double positive_norm = 0.0;
    std::size_t active = 0U;
    std::size_t entered = 0U;
    std::size_t left = 0U;
};

struct ALR32LineResult {
    bool passed = false;
    bool event_order_exact = false;
    double alpha = 0.0;
    bool trust_active = false;
    std::size_t events = 0U;
    std::size_t event_groups = 0U;
    ALR32Evaluation initial;
    ALR32Evaluation final;
    std::string event_root;
    std::string root;
};

struct ALR32DenseControl {
    std::string name;
    bool passed = false;
    double alpha = 0.0;
    std::string root;
};

double al_r32_vec3_norm(const std::vector<Vec3>& value) {
    long double squared = 0.0L;
    for (Vec3 item : value) {
        squared += static_cast<long double>(item.x) * item.x
            + static_cast<long double>(item.y) * item.y
            + static_cast<long double>(item.z) * item.z;
    }
    return std::sqrt(static_cast<double>(squared));
}

bool al_r32_vec3_finite(const std::vector<Vec3>& value) {
    return std::all_of(value.begin(), value.end(), [](Vec3 item) {
        return finite(item);
    });
}

ALR32Evaluation al_r32_evaluate(
    const std::vector<double>& constraint,
    const std::vector<double>& response,
    double alpha) {
    ALR32Evaluation result;
    if (constraint.size() != response.size() || constraint.empty()
        || !std::isfinite(alpha) || alpha < 0.0) {
        return result;
    }
    long double phi = 0.0L;
    long double derivative = 0.0L;
    for (std::size_t row = 0U; row < constraint.size(); ++row) {
        const double predicted = constraint[row] + alpha * response[row];
        if (predicted <= 0.0) continue;
        ++result.active;
        const long double lifted = predicted;
        phi += static_cast<long double>(0.5) * lifted * lifted;
        derivative += lifted * static_cast<long double>(response[row]);
        if (constraint[row] <= 0.0) ++result.entered;
    }
    for (std::size_t row = 0U; row < constraint.size(); ++row) {
        if (constraint[row] > 0.0
            && constraint[row] + alpha * response[row] <= 0.0) {
            ++result.left;
        }
    }
    result.phi = static_cast<double>(phi);
    result.derivative = static_cast<double>(derivative);
    result.positive_norm = std::sqrt(2.0 * result.phi);
    result.finite = std::isfinite(result.phi)
        && std::isfinite(result.derivative)
        && std::isfinite(result.positive_norm);
    return result;
}

ALR32LineResult al_r32_piecewise_line(
    const std::vector<double>& constraint,
    const std::vector<double>& response,
    double delta = AL_R32_TRUST_DELTA) {
    ALR32LineResult result;
    if (constraint.size() != response.size() || constraint.empty()
        || !al_r30_finite(constraint) || !al_r30_finite(response)
        || !std::isfinite(delta) || delta <= 0.0) {
        return result;
    }
    const Binary128 delta_quad = static_cast<Binary128>(delta);
    std::vector<ALR32Event> events;
    long double intercept = 0.0L;
    long double curvature = 0.0L;
    for (std::size_t row = 0U; row < constraint.size(); ++row) {
        const double c = constraint[row];
        const double z = response[row];
        const bool active_right = c > 0.0 || (c == 0.0 && z > 0.0);
        if (active_right) {
            intercept += static_cast<long double>(c) * z;
            curvature += static_cast<long double>(z) * z;
        }
        const bool leaving = c > 0.0 && z < 0.0;
        const bool entering = c < 0.0 && z > 0.0;
        if (!leaving && !entering) continue;
        const Binary128 alpha = -static_cast<Binary128>(c)
            / static_cast<Binary128>(z);
        if (!finiteq(alpha) || alpha <= static_cast<Binary128>(0.0)
            || alpha > delta_quad) {
            continue;
        }
        events.push_back({alpha, row, entering});
    }
    std::sort(events.begin(), events.end(), [](const ALR32Event& lhs,
        const ALR32Event& rhs) {
        if (lhs.alpha != rhs.alpha) return lhs.alpha < rhs.alpha;
        return lhs.row < rhs.row;
    });
    result.events = events.size();
    result.event_order_exact = true;
    for (std::size_t index = 1U; index < events.size(); ++index) {
        result.event_order_exact = result.event_order_exact
            && (events[index - 1U].alpha < events[index].alpha
                || (events[index - 1U].alpha == events[index].alpha
                    && events[index - 1U].row < events[index].row));
    }
    std::ostringstream event_projection;
    for (const ALR32Event& event : events) {
        event_projection << al_binary128_hex(event.alpha) << ':'
                         << event.row << ':' << event.entering << '|';
    }
    result.event_root = sha256_hex(event_projection.str());

    long double current = 0.0L;
    bool found = false;
    std::size_t begin = 0U;
    while (begin < events.size()) {
        std::size_t end = begin + 1U;
        while (end < events.size()
            && events[end].alpha == events[begin].alpha) {
            ++end;
        }
        ++result.event_groups;
        const long double next = static_cast<long double>(events[begin].alpha);
        if (curvature > 0.0L) {
            const long double root = -intercept / curvature;
            if (root >= current && root <= next) {
                result.alpha = static_cast<double>(root);
                found = true;
                break;
            }
        }
        for (std::size_t index = begin; index < end; ++index) {
            const std::size_t row = events[index].row;
            const long double c = constraint[row];
            const long double z = response[row];
            if (events[index].entering) {
                intercept += c * z;
                curvature += z * z;
            } else {
                intercept -= c * z;
                curvature -= z * z;
            }
        }
        current = next;
        begin = end;
    }
    if (!found) {
        if (curvature > 0.0L) {
            const long double root = -intercept / curvature;
            if (root >= current && root <= static_cast<long double>(delta)) {
                result.alpha = static_cast<double>(root);
                found = true;
            }
        }
        if (!found) result.alpha = delta;
    }
    result.alpha = std::clamp(result.alpha, 0.0, delta);
    result.trust_active = binary64_bits(result.alpha) == binary64_bits(delta);
    result.initial = al_r32_evaluate(constraint, response, 0.0);
    result.final = al_r32_evaluate(constraint, response, result.alpha);
    result.passed = result.event_order_exact
        && result.initial.finite && result.final.finite
        && result.alpha >= 0.0 && result.alpha <= delta
        && result.final.phi <= result.initial.phi;
    std::ostringstream projection;
    projection << result.passed << ':' << binary64_bits(result.alpha) << ':'
               << result.trust_active << ':' << result.events << ':'
               << result.event_groups << ':' << result.event_root << ':'
               << binary64_bits(result.initial.phi) << ':'
               << binary64_bits(result.initial.derivative) << ':'
               << binary64_bits(result.final.phi) << ':'
               << binary64_bits(result.final.derivative) << ':'
               << result.initial.active << ':' << result.final.active << ':'
               << result.final.entered << ':' << result.final.left;
    result.root = sha256_hex(projection.str());
    return result;
}

ALR32DenseControl al_r32_dense_control(
    std::string name,
    std::vector<double> constraint,
    std::vector<double> response,
    double delta,
    double expected_alpha,
    bool expected_trust,
    std::size_t expected_entered,
    std::size_t expected_left) {
    ALR32DenseControl result;
    result.name = std::move(name);
    const ALR32LineResult line = al_r32_piecewise_line(
        constraint, response, delta);
    result.alpha = line.alpha;
    const double derivative_scale = std::max(
        std::abs(line.initial.derivative), 1.0);
    const bool kkt = line.trust_active
        ? line.final.derivative <= 0.0
        : std::abs(line.final.derivative) <= 1.0e-12 * derivative_scale;
    result.passed = line.passed
        && std::abs(line.alpha - expected_alpha) <= 1.0e-12
        && line.trust_active == expected_trust
        && line.final.entered == expected_entered
        && line.final.left == expected_left
        && line.final.phi < line.initial.phi && kkt;
    result.root = sha256_hex(result.name + ':'
        + (result.passed ? "1:" : "0:") + line.root);
    return result;
}

std::array<ALR32DenseControl, 4U> al_r32_dense_controls() {
    return {{
        al_r32_dense_control(
            "interior", {1.0}, {-2.0}, 1.0, 0.5, false, 0U, 1U),
        al_r32_dense_control(
            "boundary", {1.0}, {-0.25}, 1.0, 1.0, true, 0U, 0U),
        al_r32_dense_control(
            "inactive-entry", {1.0,-0.1}, {-1.0,1.0},
            1.0, 0.55, false, 1U, 0U),
        al_r32_dense_control(
            "simultaneous-event", {1.0,-1.0}, {-1.0,1.0},
            2.0, 1.0, false, 0U, 1U),
    }};
}

std::string al_r32_route(
    bool parent, bool source, bool dense, bool workspace,
    bool gradient, bool line, bool kkt, bool work, bool stationary) {
    if (!parent) return "CAUCHY_PARENT_REJECTED";
    if (!source) return "CAUCHY_SOURCE_REJECTED";
    if (!dense) return "CAUCHY_DENSE_REJECTED";
    if (!workspace) return "CAUCHY_WORKSPACE_REJECTED";
    if (!gradient) return "CAUCHY_GRADIENT_REJECTED";
    if (!line) return "CAUCHY_LINE_REJECTED";
    if (!kkt) return "CAUCHY_KKT_REJECTED";
    if (!work) return "CAUCHY_WORK_REJECTED";
    return stationary
        ? "ALL_INEQUALITY_LINEARIZED_STATIONARY"
        : "ALL_INEQUALITY_CAUCHY_NORMAL_STEP_CANDIDATE";
}

} // namespace

SplitBoundaryReport run_al_all_inequality_cauchy_step_controls() {
    const bool identity_exact = sha256_hex(B4E2D7R19R32_IDENTITY_PROJECTION)
        == B4E2D7R19R32_IDENTITY_SHA256;
    ALR31ScalarIntervalCapture capture;
    const SplitBoundaryReport parent =
        run_al_scalar_feasibility_interval_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "d57f2712573d8e8dda52bdf44a11f40e2a6cbca03b1ac30e07c13b20af8aaa98"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && parent_semantic && parent_stdout
            == "67adbee532f615b9014e93cb44fccd053bc887c54f5d2ffcd76a19f2c771f3cb";
    ALR30RangeProjectionCapture& r30 = capture.r30;
    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string dual_before = al_r10_dual_root(r30.dual);
    const std::string constraint_before = al_r10_dual_root(r30.constraint);
    const bool source_exact = parent_exact
        && r30.state_root
            == "851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7"
        && constraint_before
            == "c567599060d7d00ffec4928009772327d081d3ef294d9030bfc5499b82e533c9"
        && r30.topology_root
            == "070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5"
        && r30.violated_root
            == "bd06f7b7c5e1f1943e5c269bbfb192229504091381b8f20f0f4988eecb021c70"
        && position_before
            == "e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828"
        && dual_before
            == "cd0100c30eb55ad7f91f1b94c00ee4281ff3c75e90ca287be038e87ab7a5215d"
        && binary64_bits(r30.theta) == 0x3fc5cccccccccccdULL
        && r30.position.size() == 6000U
        && r30.dual.size() == r30.position.size()
        && r30.constraint.size() == r30.position.size();

    const std::array<ALR32DenseControl, 4U> dense_controls =
        al_r32_dense_controls();
    const bool dense_exact = std::all_of(
        dense_controls.begin(), dense_controls.end(),
        [](const ALR32DenseControl& value) { return value.passed; });
    std::ostringstream dense_projection;
    for (const ALR32DenseControl& value : dense_controls) {
        dense_projection << value.name << ':' << value.passed << ':'
                         << binary64_bits(value.alpha) << ':'
                         << value.root << '|';
    }
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
    std::vector<double> positive_weight(r30.constraint.size(), 0.0);
    for (std::size_t row = 0U; row < r30.constraint.size(); ++row) {
        positive_weight[row] = std::max(r30.constraint[row], 0.0);
    }
    std::vector<Vec3> gradient;
    if (workspace_exact) {
        gradient = al_r29_pair_once_vjp(workspace, positive_weight);
        ++pair_passes;
    }
    const double gradient_norm = al_r32_vec3_norm(gradient);
    const bool stationary = gradient.size() == r30.position.size()
        && gradient_norm == 0.0;
    std::vector<Vec3> direction(gradient.size());
    if (!stationary && gradient_norm > 0.0) {
        for (std::size_t particle = 0U; particle < gradient.size(); ++particle) {
            direction[particle] = -(1.0 / gradient_norm) * gradient[particle];
        }
    }
    const double direction_norm = al_r32_vec3_norm(direction);
    const bool gradient_exact = workspace_exact
        && gradient.size() == r30.position.size()
        && al_r32_vec3_finite(gradient) && std::isfinite(gradient_norm)
        && gradient_norm >= 0.0
        && (stationary || (gradient_norm > 0.0
            && al_r32_vec3_finite(direction)
            && std::abs(direction_norm - 1.0) <= 1.0e-12));

    ALLinearizedJvp response;
    if (gradient_exact && !stationary) {
        response = al_r29_pair_once_jvp(workspace, direction);
        ++pair_passes;
    }
    ALR32LineResult line;
    if (stationary) {
        std::vector<double> zero(r30.constraint.size(), 0.0);
        line = al_r32_piecewise_line(
            r30.constraint, zero, AL_R32_TRUST_DELTA);
        line.alpha = 0.0;
    } else if (response.passed) {
        line = al_r32_piecewise_line(
            r30.constraint, response.value, AL_R32_TRUST_DELTA);
    }
    const bool line_exact = line.passed && line.event_order_exact
        && (stationary || response.passed);

    const long double adjoint_dot = al_r30_dot(
        positive_weight, response.value);
    const double adjoint_relative = stationary ? 0.0 : static_cast<double>(
        std::abs(adjoint_dot + static_cast<long double>(gradient_norm))
        / std::max(static_cast<long double>(gradient_norm), 1.0e-300L));
    const double derivative_relative = stationary ? 0.0
        : std::abs(line.final.derivative)
            / std::max(gradient_norm, 1.0e-300);
    const double reduction = line.initial.phi - line.final.phi;
    const bool kkt_exact = line_exact && std::isfinite(reduction)
        && std::isfinite(adjoint_relative)
        && adjoint_relative <= 1.0e-12
        && (stationary
            ? line.initial.phi > 0.0
            : reduction > 0.0 && (line.trust_active
                ? line.final.derivative <= 0.0
                : derivative_relative <= 1.0e-10));

    std::size_t selected_positive = 0U;
    std::size_t inactive_positive = 0U;
    double step_maximum = 0.0;
    for (std::size_t row = 0U; row < r30.constraint.size(); ++row) {
        const double z = stationary ? 0.0 : response.value[row];
        if (r30.constraint[row] + line.alpha * z > 0.0) {
            if (r30.violated[row]) ++selected_positive;
            else ++inactive_positive;
        }
    }
    for (Vec3 value : direction) {
        step_maximum = std::max(step_maximum, line.alpha * norm(value));
    }
    const double step_rms = r30.position.empty() ? 0.0
        : line.alpha / std::sqrt(static_cast<double>(r30.position.size()));
    const double violation_ratio = line.initial.positive_norm > 0.0
        ? line.final.positive_norm / line.initial.positive_norm : 0.0;

    release_al_normalized_workspace(workspace, &work);
    const bool work_exact = pair_passes == (stationary ? 1U : 2U)
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

    struct RouteCase {
        const char* expected;
        std::string observed;
    };
    const std::array<RouteCase, 10U> routes{{
        {"ALL_INEQUALITY_CAUCHY_NORMAL_STEP_CANDIDATE", al_r32_route(1,1,1,1,1,1,1,1,0)},
        {"ALL_INEQUALITY_LINEARIZED_STATIONARY", al_r32_route(1,1,1,1,1,1,1,1,1)},
        {"CAUCHY_PARENT_REJECTED", al_r32_route(0,1,1,1,1,1,1,1,0)},
        {"CAUCHY_SOURCE_REJECTED", al_r32_route(1,0,1,1,1,1,1,1,0)},
        {"CAUCHY_DENSE_REJECTED", al_r32_route(1,1,0,1,1,1,1,1,0)},
        {"CAUCHY_WORKSPACE_REJECTED", al_r32_route(1,1,1,0,1,1,1,1,0)},
        {"CAUCHY_GRADIENT_REJECTED", al_r32_route(1,1,1,1,0,1,1,1,0)},
        {"CAUCHY_LINE_REJECTED", al_r32_route(1,1,1,1,1,0,1,1,0)},
        {"CAUCHY_KKT_REJECTED", al_r32_route(1,1,1,1,1,1,0,1,0)},
        {"CAUCHY_WORK_REJECTED", al_r32_route(1,1,1,1,1,1,1,0,0)},
    }};
    bool routes_exact = true;
    std::ostringstream route_projection;
    for (const RouteCase& value : routes) {
        routes_exact = routes_exact && value.expected == value.observed;
        route_projection << value.expected << ':' << value.observed << '|';
    }
    const std::string route_root = sha256_hex(route_projection.str());
    const bool passed = identity_exact && parent_exact && source_exact
        && dense_exact && workspace_exact && gradient_exact && line_exact
        && kkt_exact && work_exact && rollback_exact && routes_exact;
    const std::string route = passed ? al_r32_route(
        true,true,true,true,true,true,true,true,stationary) : std::string{};
    std::string first_failure;
    if (!identity_exact) first_failure = "IDENTITY";
    else if (!parent_exact) first_failure = "PARENT";
    else if (!source_exact) first_failure = "SOURCE";
    else if (!dense_exact) first_failure = "DENSE";
    else if (!workspace_exact) first_failure = "WORKSPACE";
    else if (!gradient_exact) first_failure = "GRADIENT";
    else if (!line_exact) first_failure = "LINE";
    else if (!kkt_exact) first_failure = "KKT";
    else if (!work_exact) first_failure = "WORK";
    else if (!rollback_exact) first_failure = "ROLLBACK";
    else if (!routes_exact) first_failure = "ROUTES";

    const std::string gradient_root = al_binary64_vec3_root(gradient);
    const std::string direction_root = al_binary64_vec3_root(direction);
    const std::string response_root = al_r10_dual_root(response.value);
    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R32_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << constraint_before << ':' << r30.topology_root << '|'
             << dense_root << ':' << gradient_root << ':' << direction_root
             << ':' << response_root << ':' << line.root << '|'
             << binary64_bits(gradient_norm) << ':'
             << binary64_bits(line.alpha) << ':' << binary64_bits(reduction)
             << ':' << selected_positive << ':' << inactive_positive << '|'
             << pair_passes << ':' << route_root << ':' << routes_exact << '|'
             << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4e2d7r19r32_all_inequality_cauchy_normal_step.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R32_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source\":{\"constraint_root\":\"" << constraint_before
           << "\",\"topology_root\":\"" << r30.topology_root
           << "\",\"particles\":" << r30.position.size()
           << ",\"pairs\":340340,\"maximum_degree\":113}"
           << ",\"dense\":{\"passed\":"
           << (dense_exact ? "true" : "false")
           << ",\"cases\":" << dense_controls.size()
           << ",\"root\":\"" << dense_root << "\"}"
           << ",\"gradient\":{\"root\":\"" << gradient_root
           << "\",\"norm\":" << gradient_norm
           << ",\"direction_root\":\"" << direction_root
           << "\",\"direction_norm\":" << direction_norm
           << ",\"adjoint_relative\":" << adjoint_relative
           << ",\"stationary\":" << (stationary ? "true" : "false") << "}"
           << ",\"line\":{\"response_root\":\"" << response_root
           << "\",\"events\":" << line.events
           << ",\"event_groups\":" << line.event_groups
           << ",\"event_root\":\"" << line.event_root
           << "\",\"alpha\":" << line.alpha
           << ",\"trust_delta\":" << AL_R32_TRUST_DELTA
           << ",\"trust_active\":" << (line.trust_active ? "true" : "false")
           << ",\"phi_initial\":" << line.initial.phi
           << ",\"phi_final\":" << line.final.phi
           << ",\"reduction\":" << reduction
           << ",\"derivative_initial\":" << line.initial.derivative
           << ",\"derivative_final\":" << line.final.derivative
           << ",\"derivative_relative\":" << derivative_relative
           << ",\"violation_norm_initial\":" << line.initial.positive_norm
           << ",\"violation_norm_final\":" << line.final.positive_norm
           << ",\"violation_norm_ratio\":" << violation_ratio
           << ",\"active_initial\":" << line.initial.active
           << ",\"active_final\":" << line.final.active
           << ",\"entered\":" << line.final.entered
           << ",\"left\":" << line.final.left
           << ",\"selected_positive\":" << selected_positive
           << ",\"inactive_positive\":" << inactive_positive
           << ",\"step_rms\":" << step_rms
           << ",\"step_maximum\":" << step_maximum
           << ",\"root\":\"" << line.root << "\"}"
           << ",\"work\":{\"parent_r31_replays\":1"
           << ",\"diagnostic_workspaces\":1,\"new_pair_passes\":"
           << pair_passes << ",\"new_hvp\":0,\"new_model\":0"
              ",\"new_trials\":0,\"new_outer\":0}"
           << ",\"route_cases\":" << routes.size()
           << ",\"route_root\":\"" << route_root << "\""
           << ",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback_exact ? "true" : "false")
           << ",\"first_failure\":\"" << first_failure << "\""
           << ",\"route\":\"" << route << "\""
              ",\"correction_applied\":false,\"nonlinear_evaluation\":false"
              ",\"floor_classification\":false,\"state_mutation\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false"
           << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {passed, report.str()};
}
