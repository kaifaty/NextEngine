namespace {

constexpr const char* B4E2D7R19R49_IDENTITY_SHA256 =
    "e61ba2c8ca22f43c09ca7ad6a8949af6437b91f13b1f45b5b5c17926dbae62b8";
constexpr const char* B4E2D7R19R49_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r49-restoration-hz-primal-accelerator|v1|parent=b2d160d7:70a66443d0a278697e3e4d8d4456f80f5f1ccb1269596d3e5ce569e87d9b90b6:7d83b855aeda6c32202e197c65c94f88235e6e997563f7847ddb5972d88a7e91:RESTORATION_CERTIFICATE_UNRESOLVED|source=r43-position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;trial-0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50;owner-baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187;superset-355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01;r48-witness-29c5da31f537a3bbecc8826c42266cb94b25ff2ed22bb8691333d825f30a7f6d|problem=unchanged-r48;y=r43-projected-trial;c=fresh-density-inequality-at-y;A=spacing*Jc-at-y;phi(d)=0.5*norm2(max(c+A*d,0));all-rows;fixed-stable-superset|geometry=unchanged-r48;next-radius0.0625;normal-radius0.03125;global-l2;current-contact-box;zero-in-every-interval;source-half-skin-implied|generator=guarded-raw-hager-zhang;d0=0;g0=At-max-c-positive;first-steepest;history=previous-gradient-and-accepted-feasible-chord;beta=r38-long-double-hz;raw-normalize;target=current-plus-unit-raw;project-target-with-r48-euclidean-ball-box;chord=target-current;restart-invalid-beta-or-nonfinite-or-raw-nondescent-or-empty-chord-or-projected-nondescent;restart-history-becomes-accepted-steepest|max-accepted16;checkpoints1,2,4,8,16;exact-projected-stationarity-only|line=alpha-in-0,1;exact-r32-piecewise-all-row-hinge-on-feasible-chord;binary128-event-order;long-double-fold;direct-kkt;strict-objective-decrease|certificate=after-every-accepted-step;fresh-r48-directed-jvp-row-upper;zero-certified-positive-only;unchanged-rounding;early-stop-only-on-certificate;parent-r48-dual-unchanged;no-new-infeasibility-claim|selection=primal-certificate-first;exact-stationary-unresolved-second;otherwise-accelerator-candidate-only-if-terminal-strict-psi-h-maximum-upper-dominance-over-r48-and-pairpasses<=48-and-memory-streak>=2;else-r48-reference-retained|controls=r48-parent-bytes;r39-dense-hz-root;dense-ball-box-chord;dense-active-switch-restart;dense-exact-line;dense-primal-rounding;source;workspace;geometry;projection;generator;line;certificate;comparison;work;rollback;route-precedence|routes=restoration-hz-parent-rejected;restoration-hz-source-rejected;restoration-hz-workspace-rejected;restoration-hz-geometry-rejected;restoration-hz-projection-rejected;restoration-hz-generator-rejected;restoration-hz-line-rejected;restoration-hz-primal-audit-rejected;restoration-hz-comparison-rejected;restoration-hz-work-rejected;restoration-next-trqp-compatible-candidate;restoration-hz-stationary-unresolved;restoration-hz-primal-accelerator-candidate;restoration-hz-reference-retained|precedence=parent,source,workspace,geometry,projection,generator,line,primal-audit,comparison,work,certificate,stationarity,selection|work=parent-r48-replays1;static-index1;superset-build1;moved-workspace1;releases1;accepted<=16;line-jvp<=16;directed-certificate-jvp<=16;vjp<=16;pairpasses<=48;projection-calls<=32;projection-scans-bounded;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48=unchanged;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;penalty-change=none;support-expansion=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-primal-accelerator-classification-only)IDENTITY";

constexpr std::size_t AL_R49_MAX_ACCEPTED = 16U;
constexpr double AL_R49_R48_PSI = 1.3258397316434649e-17;
constexpr double AL_R49_R48_H = 5.1494460510689202e-09;
constexpr double AL_R49_R48_MAXIMUM_UPPER = 4.6927711453206623e-10;

struct ALR49Checkpoint {
    std::size_t accepted = 0U;
    double psi = std::numeric_limits<double>::infinity();
    double h = std::numeric_limits<double>::infinity();
    double maximum_upper = std::numeric_limits<double>::infinity();
    std::size_t certified_positive = 0U;
    std::size_t memory_streak = 0U;
    std::string root;
};

struct ALR49Generator {
    bool passed = false;
    bool stationary = false;
    bool certified = false;
    bool projection_exact = true;
    bool line_exact = true;
    bool primal_exact = true;
    bool response_exact = true;
    std::size_t accepted = 0U;
    std::size_t restarts = 0U;
    std::size_t memory_steps = 0U;
    std::size_t memory_streak = 0U;
    std::size_t maximum_memory_streak = 0U;
    std::size_t line_jvp = 0U;
    std::size_t certificate_jvp = 0U;
    std::size_t vjp = 0U;
    std::size_t projection_calls = 0U;
    std::size_t projection_scans = 0U;
    std::size_t pair_passes = 0U;
    double witness_norm = 0.0;
    double terminal_response_relative = std::numeric_limits<double>::infinity();
    ALR48PrimalAudit primal;
    std::vector<Vec3> witness;
    std::vector<ALR49Checkpoint> checkpoints;
    std::vector<std::string> step_roots;
};

bool al_r49_checkpoint_requested(std::size_t accepted) {
    return accepted == 1U || accepted == 2U || accepted == 4U
        || accepted == 8U || accepted == 16U;
}

bool al_r49_feasible(const std::vector<Vec3>& value,
    const std::vector<Vec3>& lower, const std::vector<Vec3>& upper) {
    if (value.size() != lower.size() || value.size() != upper.size()
        || !al_r48_vec_finite(value)) return false;
    bool box = true;
    for (std::size_t i = 0U; i < value.size(); ++i)
        for (int axis = 0; axis < 3; ++axis) {
            const double item = al_r48_component(value[i], axis);
            const double low = al_r48_component(lower[i], axis);
            const double high = al_r48_component(upper[i], axis);
            box = box
                && item >= std::nextafter(
                    low, -std::numeric_limits<double>::infinity())
                && item <= std::nextafter(
                    high, std::numeric_limits<double>::infinity());
        }
    return box && al_r32_vec3_norm(value) <= std::nextafter(
        AL_R48_NORMAL_RADIUS, std::numeric_limits<double>::infinity());
}

struct ALR49Chord {
    bool passed = false;
    bool empty = false;
    std::vector<Vec3> value;
    double norm = 0.0;
    double descent = std::numeric_limits<double>::quiet_NaN();
    ALR48Projection projection;
};

ALR49Chord al_r49_project_chord(const std::vector<Vec3>& current,
    const std::vector<Vec3>& raw, const std::vector<Vec3>& gradient,
    const std::vector<Vec3>& lower, const std::vector<Vec3>& upper) {
    ALR49Chord result;
    if (current.size() != raw.size() || current.size() != gradient.size()
        || current.empty()) return result;
    const double raw_norm = al_r32_vec3_norm(raw);
    if (!std::isfinite(raw_norm) || raw_norm == 0.0) return result;
    std::vector<Vec3> target(current.size());
    for (std::size_t i = 0U; i < current.size(); ++i)
        target[i] = current[i] + (1.0 / raw_norm) * raw[i];
    result.projection = al_r48_project_ball_box(
        target, lower, upper, AL_R48_NORMAL_RADIUS);
    if (!result.projection.passed) return result;
    result.value.resize(current.size());
    for (std::size_t i = 0U; i < current.size(); ++i)
        result.value[i] = result.projection.value[i] - current[i];
    result.norm = al_r32_vec3_norm(result.value);
    result.empty = result.norm == 0.0;
    result.descent = static_cast<double>(
        al_r35_dot_vec3(gradient, result.value));
    result.passed = al_r49_feasible(result.projection.value, lower, upper)
        && al_r32_vec3_finite(result.value) && std::isfinite(result.norm)
        && std::isfinite(result.descent);
    return result;
}

ALR49Checkpoint al_r49_checkpoint(std::size_t accepted,
    const ALR48PrimalAudit& primal, std::size_t memory_streak) {
    ALR49Checkpoint result;
    result.accepted = accepted;
    result.psi = primal.psi;
    result.h = primal.h;
    result.maximum_upper = primal.maximum_upper;
    result.certified_positive = primal.certified_positive;
    result.memory_streak = memory_streak;
    std::ostringstream projection;
    projection << accepted << ':' << binary64_bits(primal.psi) << ':'
               << binary64_bits(primal.h) << ':'
               << binary64_bits(primal.maximum_upper) << ':'
               << primal.certified_positive << ':' << memory_streak;
    result.root = sha256_hex(projection.str());
    return result;
}

ALR49Generator al_r49_generate(const ALNormalizedWorkspace& workspace,
    const std::vector<Vec3>& lower, const std::vector<Vec3>& upper) {
    ALR49Generator result;
    const std::vector<double>& c = workspace.support.constraint;
    if (!workspace.passed || c.empty() || lower.size() != c.size()
        || upper.size() != c.size()) return result;
    std::vector<Vec3> current(c.size());
    std::vector<double> maintained_image(c.size(), 0.0);
    std::vector<double> residual = c;
    std::vector<double> positive(c.size());
    for (std::size_t row = 0U; row < c.size(); ++row)
        positive[row] = std::max(c[row], 0.0);
    std::vector<Vec3> gradient = al_r29_pair_once_vjp(workspace, positive);
    ++result.vjp;
    if (gradient.size() != current.size() || !al_r48_vec_finite(gradient))
        return result;
    std::vector<double> zero(c.size(), 0.0);
    result.primal = al_r48_primal_audit(c, {true, zero, zero},
        workspace.neighborhood.maximum_degree);
    if (!result.primal.passed) return result;
    std::vector<Vec3> previous_gradient;
    std::vector<Vec3> previous_direction;
    bool exact = true;
    while (exact && !result.stationary && !result.certified
           && result.accepted < AL_R49_MAX_ACCEPTED) {
        const bool has_history = !previous_gradient.empty()
            && previous_gradient.size() == gradient.size()
            && previous_direction.size() == gradient.size();
        ALR38Beta beta = al_r38_beta(
            has_history ? ALR38LaneKind::HagerZhang
                        : ALR38LaneKind::Steepest,
            gradient, previous_gradient, previous_direction);
        bool restarted = false;
        std::string restart_reason;
        std::vector<Vec3> raw(gradient.size());
        auto set_steepest = [&]() {
            for (std::size_t i = 0U; i < raw.size(); ++i)
                raw[i] = -gradient[i];
        };
        if (beta.valid) {
            for (std::size_t i = 0U; i < raw.size(); ++i)
                raw[i] = -gradient[i]
                    + (has_history ? beta.beta * previous_direction[i]
                                   : Vec3{});
        } else {
            restarted = has_history;
            restart_reason = "INVALID_BETA";
            set_steepest();
        }
        double raw_descent = static_cast<double>(
            al_r35_dot_vec3(gradient, raw));
        if (!std::isfinite(raw_descent) || raw_descent >= 0.0) {
            restarted = has_history;
            restart_reason = "RAW_NON_DESCENT";
            set_steepest();
            raw_descent = static_cast<double>(
                al_r35_dot_vec3(gradient, raw));
        }
        ALR49Chord chord = al_r49_project_chord(
            current, raw, gradient, lower, upper);
        ++result.projection_calls;
        result.projection_scans += chord.projection.scans;
        if ((!chord.passed || chord.empty || chord.descent >= 0.0)
            && has_history && !restarted) {
            restarted = true;
            restart_reason = chord.empty
                ? "EMPTY_CHORD" : "PROJECTED_NON_DESCENT";
            set_steepest();
            chord = al_r49_project_chord(
                current, raw, gradient, lower, upper);
            ++result.projection_calls;
            result.projection_scans += chord.projection.scans;
        }
        result.projection_exact = result.projection_exact && chord.passed;
        if (!chord.passed || !std::isfinite(raw_descent)
            || raw_descent >= 0.0) { exact = false; break; }
        if (chord.empty) { result.stationary = true; break; }
        if (chord.descent >= 0.0) { exact = false; break; }

        const ALLinearizedJvp line_image =
            al_r29_pair_once_jvp(workspace, chord.value);
        ++result.line_jvp;
        const ALR32LineResult line = al_r32_piecewise_line(
            residual, line_image.value, 1.0);
        const double derivative_scale = std::max(
            std::abs(line.initial.derivative), 1.0e-300);
        const bool line_kkt = line.trust_active
            ? line.final.derivative <= 0.0
            : std::abs(line.final.derivative) / derivative_scale <= 1.0e-10;
        const bool line_ok = line_image.passed && line.passed && line_kkt
            && line.alpha > 0.0 && line.final.phi < line.initial.phi;
        result.line_exact = result.line_exact && line_ok;
        if (!line_ok) { exact = false; break; }

        std::vector<Vec3> next(current.size());
        for (std::size_t i = 0U; i < current.size(); ++i)
            next[i] = current[i] + line.alpha * chord.value[i];
        if (!al_r49_feasible(next, lower, upper)) {
            result.projection_exact = false;
            exact = false;
            break;
        }
        for (std::size_t row = 0U; row < c.size(); ++row)
            maintained_image[row] += line.alpha * line_image.value[row];
        const std::vector<Vec3> accepted_gradient = gradient;
        current = std::move(next);
        previous_gradient = accepted_gradient;
        previous_direction = chord.value;
        ++result.accepted;
        if (has_history && !restarted) {
            ++result.memory_steps;
            ++result.memory_streak;
            result.maximum_memory_streak = std::max(
                result.maximum_memory_streak, result.memory_streak);
        } else {
            if (restarted) ++result.restarts;
            result.memory_streak = 0U;
        }

        const ALLinearizedJvp certificate_image =
            al_r29_directed_jvp(workspace, current);
        ++result.certificate_jvp;
        result.primal = al_r48_primal_audit(
            c, certificate_image, workspace.neighborhood.maximum_degree);
        const double relative = certificate_image.passed
            ? al_r33_vec_diff_relative(
                maintained_image, certificate_image.value)
            : std::numeric_limits<double>::infinity();
        result.terminal_response_relative = relative;
        const bool response_ok = certificate_image.passed
            && std::isfinite(relative) && relative <= 1.0e-12;
        result.response_exact = result.response_exact && response_ok;
        result.primal_exact = result.primal_exact && result.primal.passed;
        if (!response_ok || !result.primal.passed) {
            exact = false;
            break;
        }
        maintained_image = certificate_image.value;
        for (std::size_t row = 0U; row < c.size(); ++row)
            residual[row] = c[row] + maintained_image[row];
        result.certified = result.primal.certified;
        if (al_r49_checkpoint_requested(result.accepted))
            result.checkpoints.push_back(al_r49_checkpoint(
                result.accepted, result.primal, result.memory_streak));
        std::ostringstream step_projection;
        step_projection << result.accepted << ':' << beta.root << ':'
                        << restarted << ':' << restart_reason << ':'
                        << binary64_bits(raw_descent) << ':'
                        << binary64_bits(chord.descent) << ':'
                        << binary64_bits(chord.norm) << ':' << line.root << ':'
                        << binary64_bits(line.alpha) << ':'
                        << binary64_bits(result.primal.psi) << ':'
                        << binary64_bits(result.primal.maximum_upper) << ':'
                        << result.primal.certified_positive;
        result.step_roots.push_back(sha256_hex(step_projection.str()));

        if (!result.certified && result.accepted < AL_R49_MAX_ACCEPTED) {
            for (std::size_t row = 0U; row < c.size(); ++row)
                positive[row] = std::max(residual[row], 0.0);
            gradient = al_r29_pair_once_vjp(workspace, positive);
            ++result.vjp;
            if (gradient.size() != current.size()
                || !al_r48_vec_finite(gradient)) {
                exact = false;
                break;
            }
        }
    }
    result.witness = std::move(current);
    result.witness_norm = al_r32_vec3_norm(result.witness);
    result.pair_passes = result.line_jvp
        + result.certificate_jvp + result.vjp;
    result.passed = exact && result.projection_exact && result.line_exact
        && result.primal_exact && result.response_exact
        && (result.certified || result.stationary
            || result.accepted == AL_R49_MAX_ACCEPTED)
        && result.accepted <= AL_R49_MAX_ACCEPTED
        && result.line_jvp <= AL_R49_MAX_ACCEPTED
        && result.certificate_jvp <= AL_R49_MAX_ACCEPTED
        && result.vjp <= AL_R49_MAX_ACCEPTED
        && result.pair_passes <= 48U
        && result.projection_calls <= 32U
        && al_r49_feasible(result.witness, lower, upper)
        && std::isfinite(result.witness_norm);
    return result;
}

bool al_r49_dense_controls(std::string* output_root) {
    std::string r39_root;
    const bool r39 = al_r39_dense_controls(&r39_root);
    const std::vector<Vec3> lower{{-0.25, 0.0, 0.0}};
    const std::vector<Vec3> upper{{0.25, 0.0, 0.0}};
    const ALR49Chord chord = al_r49_project_chord(
        {{0.0, 0.0, 0.0}}, {{1.0, 0.0, 0.0}},
        {{-1.0, 0.0, 0.0}}, lower, upper);
    const ALR38Beta restart = al_r38_beta(
        ALR38LaneKind::HagerZhang,
        {{1.0, 0.0, 0.0}}, {{1.0, 0.0, 0.0}},
        {{-1.0, 0.0, 0.0}});
    const ALR32LineResult line = al_r32_piecewise_line(
        {1.0, -0.25}, {-2.0, 1.0}, 1.0);
    const ALR48PrimalAudit safe = al_r48_primal_audit(
        {-1.0}, {true, {-0.5}, {0.5}}, 1U);
    const ALR48PrimalAudit guarded = al_r48_primal_audit(
        {-1.0e-16}, {true, {0.0}, {1.0}}, 1U);
    const bool passed = r39 && chord.passed && !chord.empty
        && binary64_bits(chord.projection.value[0].x)
            == binary64_bits(AL_R48_NORMAL_RADIUS)
        && chord.descent < 0.0 && !restart.valid && line.passed
        && line.final.phi < line.initial.phi
        && safe.passed && safe.certified
        && guarded.passed && !guarded.certified;
    std::ostringstream projection;
    projection << r39_root << ':' << passed << ':'
               << binary64_bits(chord.norm) << ':'
               << binary64_bits(chord.descent) << ':' << restart.root << ':'
               << line.root << ':' << safe.certified << ':'
               << guarded.certified << ':'
               << binary64_bits(guarded.maximum_upper);
    if (output_root != nullptr) *output_root = sha256_hex(projection.str());
    return passed;
}

std::string al_r49_route(bool parent, bool source, bool workspace,
    bool geometry, bool projection, bool generator, bool line,
    bool primal_audit, bool comparison, bool work, bool certified,
    bool stationary, bool accelerator) {
    if (!parent) return "RESTORATION_HZ_PARENT_REJECTED";
    if (!source) return "RESTORATION_HZ_SOURCE_REJECTED";
    if (!workspace) return "RESTORATION_HZ_WORKSPACE_REJECTED";
    if (!geometry) return "RESTORATION_HZ_GEOMETRY_REJECTED";
    if (!projection) return "RESTORATION_HZ_PROJECTION_REJECTED";
    if (!generator) return "RESTORATION_HZ_GENERATOR_REJECTED";
    if (!line) return "RESTORATION_HZ_LINE_REJECTED";
    if (!primal_audit) return "RESTORATION_HZ_PRIMAL_AUDIT_REJECTED";
    if (!comparison) return "RESTORATION_HZ_COMPARISON_REJECTED";
    if (!work) return "RESTORATION_HZ_WORK_REJECTED";
    if (certified) return "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE";
    if (stationary) return "RESTORATION_HZ_STATIONARY_UNRESOLVED";
    if (accelerator) return "RESTORATION_HZ_PRIMAL_ACCELERATOR_CANDIDATE";
    return "RESTORATION_HZ_REFERENCE_RETAINED";
}

} // namespace

SplitBoundaryReport run_al_restoration_hz_accelerator_controls() {
    const bool identity = sha256_hex(B4E2D7R19R49_IDENTITY_PROJECTION)
        == B4E2D7R19R49_IDENTITY_SHA256;
    ALR48ReplayCapture capture;
    const SplitBoundaryReport parent =
        run_al_restoration_certificate_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_exact = parent.passed && capture.captured
        && parent_stdout
            == "70a66443d0a278697e3e4d8d4456f80f5f1ccb1269596d3e5ce569e87d9b90b6"
        && parent.json.find(
            "7d83b855aeda6c32202e197c65c94f88235e6e997563f7847ddb5972d88a7e91")
            != std::string::npos
        && parent.json.find("RESTORATION_CERTIFICATE_UNRESOLVED")
            != std::string::npos;
    ALR43ReplayCapture& r43 = capture.r47.r46.r45.r44.r43;
    ALR40ReplayCapture& r40 = r43.r42.r40;
    ALR30RangeProjectionCapture& r30 =
        r40.r39.r38.r37.r36.r35.r34.r33.r32.r31.r30;
    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string trial_before =
        al_binary64_vec3_root(r43.projected_trial);
    const std::string owner_before = al_r29_mask_root(r43.owners);
    const std::string superset_before =
        al_r42_superset_root(r43.r42.superset);
    const bool source_exact = parent_exact && r30.captured
        && position_before
            == "e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828"
        && trial_before
            == "0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50"
        && owner_before
            == "baf145ae5d293802b8a1915ec05b10781b79bfed2f30f520f0a960bb817e5187"
        && superset_before
            == "355ce6fa5ccaf7524e2f28fedf74c68baf0e85a60429a5442d3a4bd707130f01";

    std::string dense_root;
    const bool dense = identity && al_r49_dense_controls(&dense_root);
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
    const ALR40HingeMetric moved_metric =
        al_r40_hinge_metric(moved.support.constraint);
    const bool workspace_exact = source_exact && dense && index.passed
        && binding.passed && superset.passed && moved.passed
        && al_r42_superset_root(superset) == superset_before
        && moved_filtered.pairs == moved.neighborhood.pairs
        && moved_metric.finite
        && binary64_bits(moved_metric.psi)
            == binary64_bits(6.8540208964482797e-16)
        && moved_metric.active == 464U;

    std::vector<Vec3> lower(r43.projected_trial.size());
    std::vector<Vec3> upper(r43.projected_trial.size());
    std::vector<std::uint8_t> moved_owners(
        r43.projected_trial.size() * 3U, 0U);
    std::size_t moved_active_faces = 0U;
    bool intervals_exact = workspace_exact;
    for (std::size_t i = 0U; i < r43.projected_trial.size(); ++i)
        for (int axis = 0; axis < 3; ++axis) {
            const double y = al_r41_component(r43.projected_trial[i], axis);
            const double low_face = al_r41_component(fixture.contact_low, axis);
            const double high_face = al_r41_component(
                fixture.contact_high, axis);
            double low = (low_face - y) / SPACING;
            double high = (high_face - y) / SPACING;
            const std::size_t component =
                3U * i + static_cast<std::size_t>(axis);
            if (y < low_face) {
                low = 0.0;
                moved_owners[component] = 1U;
                ++moved_active_faces;
            } else if (y > high_face) {
                high = 0.0;
                moved_owners[component] = 2U;
                ++moved_active_faces;
            }
            al_r43_component(lower[i], axis) = low;
            al_r43_component(upper[i], axis) = high;
            intervals_exact = intervals_exact && std::isfinite(low)
                && std::isfinite(high) && low <= 0.0 && high >= 0.0;
        }
    const double moved_step_norm = al_r32_vec3_norm(r43.projected_endpoint);
    const double half_skin = 0.5 * B4EP3_SKIN
        * std::sqrt(1.0 - 2.0e-12) / SPACING;
    const std::string moved_owner_root = al_r29_mask_root(moved_owners);
    const bool geometry_exact = intervals_exact
        && AL_R48_NORMAL_RADIUS == 0.5 * AL_R48_NEXT_RADIUS
        && moved_step_norm + AL_R48_NORMAL_RADIUS <= half_skin
        && moved_active_faces == 1290U && moved_owner_root == owner_before;
    const ALR48Projection zero_projection = al_r48_project_ball_box(
        std::vector<Vec3>(lower.size()), lower, upper,
        AL_R48_NORMAL_RADIUS);
    const bool projection_source_exact = geometry_exact
        && zero_projection.passed && zero_projection.norm == 0.0;

    ALR49Generator generator;
    if (projection_source_exact)
        generator = al_r49_generate(moved, lower, upper);
    const bool generator_exact = projection_source_exact && generator.passed;
    const bool projection_exact = generator_exact && generator.projection_exact;
    const bool line_exact = generator_exact && generator.line_exact;
    const bool primal_exact = generator_exact && generator.primal_exact
        && generator.primal.passed
        && (!generator.certified
            || generator.primal.certified_positive == 0U);
    const bool dominates = primal_exact && !generator.stationary
        && generator.primal.psi < AL_R49_R48_PSI
        && generator.primal.h < AL_R49_R48_H
        && generator.primal.maximum_upper < AL_R49_R48_MAXIMUM_UPPER
        && generator.pair_passes <= 48U
        && generator.maximum_memory_streak >= 2U;
    const bool comparison_exact = primal_exact
        && std::isfinite(generator.primal.psi)
        && std::isfinite(generator.primal.h)
        && std::isfinite(generator.primal.maximum_upper);
    if (moved.passed) release_al_normalized_workspace(moved, &workspace_work);
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
        && generator.accepted <= AL_R49_MAX_ACCEPTED
        && generator.line_jvp <= 16U
        && generator.certificate_jvp <= 16U
        && generator.vjp <= 16U
        && generator.pair_passes <= 48U
        && generator.projection_calls <= 32U;
    const bool rollback = position_before == al_binary64_vec3_root(r30.position)
        && trial_before == al_binary64_vec3_root(r43.projected_trial)
        && owner_before == al_r29_mask_root(r43.owners)
        && superset_before == al_r42_superset_root(r43.r42.superset);

    const std::array<const char*, 14U> expected_routes{{
        "RESTORATION_HZ_PARENT_REJECTED",
        "RESTORATION_HZ_SOURCE_REJECTED",
        "RESTORATION_HZ_WORKSPACE_REJECTED",
        "RESTORATION_HZ_GEOMETRY_REJECTED",
        "RESTORATION_HZ_PROJECTION_REJECTED",
        "RESTORATION_HZ_GENERATOR_REJECTED",
        "RESTORATION_HZ_LINE_REJECTED",
        "RESTORATION_HZ_PRIMAL_AUDIT_REJECTED",
        "RESTORATION_HZ_COMPARISON_REJECTED",
        "RESTORATION_HZ_WORK_REJECTED",
        "RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE",
        "RESTORATION_HZ_STATIONARY_UNRESOLVED",
        "RESTORATION_HZ_PRIMAL_ACCELERATOR_CANDIDATE",
        "RESTORATION_HZ_REFERENCE_RETAINED"}};
    bool routes_exact = true;
    std::ostringstream route_material;
    for (std::size_t route_index = 0U;
         route_index < expected_routes.size(); ++route_index) {
        std::array<bool, 10U> gates{};
        gates.fill(true);
        if (route_index < gates.size()) gates[route_index] = false;
        const bool certified = route_index == 10U;
        const bool stationary = route_index == 11U;
        const bool accelerator = route_index == 12U;
        const std::string observed = al_r49_route(
            gates[0], gates[1], gates[2], gates[3], gates[4], gates[5],
            gates[6], gates[7], gates[8], gates[9], certified, stationary,
            accelerator);
        routes_exact = routes_exact && observed == expected_routes[route_index];
        route_material << expected_routes[route_index] << ':' << observed << '|';
    }
    const std::string route_root = sha256_hex(route_material.str());
    const std::string route = al_r49_route(parent_exact, source_exact,
        workspace_exact, geometry_exact, projection_exact, generator_exact,
        line_exact, primal_exact, comparison_exact, work_exact && rollback,
        generator.certified, generator.stationary, dominates);
    const bool passed = identity && parent_exact && source_exact && dense
        && workspace_exact && geometry_exact && projection_exact
        && generator_exact && line_exact && primal_exact && comparison_exact
        && work_exact && rollback && routes_exact;

    std::ostringstream checkpoint_material;
    for (const ALR49Checkpoint& checkpoint : generator.checkpoints)
        checkpoint_material << checkpoint.root << ':';
    const std::string checkpoint_root = sha256_hex(checkpoint_material.str());
    std::ostringstream step_material;
    for (const std::string& root : generator.step_roots)
        step_material << root << ':';
    const std::string step_root = sha256_hex(step_material.str());
    const std::string witness_root = al_binary64_vec3_root(generator.witness);
    std::ostringstream semantic;
    semantic << std::setprecision(17) << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R49_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << moved_metric.psi << ':' << moved_metric.norm << ':'
             << moved_metric.active << '|' << moved_owner_root << ':'
             << moved_active_faces << ':' << half_skin << '|'
             << generator.accepted << ':' << generator.restarts << ':'
             << generator.memory_steps << ':'
             << generator.maximum_memory_streak << ':'
             << generator.projection_calls << ':' << checkpoint_root << ':'
             << step_root << '|' << generator.primal.psi << ':'
             << generator.primal.h << ':' << generator.primal.maximum_upper
             << ':' << generator.primal.certified_positive << ':'
             << generator.witness_norm << ':' << witness_root << '|'
             << generator.line_jvp << ':' << generator.certificate_jvp << ':'
             << generator.vjp << ':' << generator.pair_passes << ':'
             << dominates << ':' << work_exact << ':' << rollback << ':'
             << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r19r49_restoration_hz_primal_accelerator.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"identity_sha256\":\""
           << B4E2D7R19R49_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source\":{\"psi\":" << moved_metric.psi
           << ",\"h\":" << moved_metric.norm
           << ",\"active\":" << moved_metric.active << "}"
           << ",\"geometry\":{\"next_radius\":" << AL_R48_NEXT_RADIUS
           << ",\"normal_radius\":" << AL_R48_NORMAL_RADIUS
           << ",\"half_skin\":" << half_skin
           << ",\"active_faces\":" << moved_active_faces
           << ",\"owner_root\":\"" << moved_owner_root << "\"}"
           << ",\"generator\":{\"accepted\":" << generator.accepted
           << ",\"restarts\":" << generator.restarts
           << ",\"memory_steps\":" << generator.memory_steps
           << ",\"maximum_memory_streak\":"
           << generator.maximum_memory_streak
           << ",\"stationary\":"
           << (generator.stationary ? "true" : "false")
           << ",\"projection_calls\":" << generator.projection_calls
           << ",\"projection_scans\":" << generator.projection_scans
           << ",\"checkpoint_root\":\"" << checkpoint_root
           << "\",\"step_root\":\"" << step_root << "\"}"
           << ",\"primal\":{\"psi\":" << generator.primal.psi
           << ",\"h\":" << generator.primal.h
           << ",\"maximum_upper\":" << generator.primal.maximum_upper
           << ",\"raw_active\":" << generator.primal.raw_active
           << ",\"certified_positive\":"
           << generator.primal.certified_positive
           << ",\"witness_norm\":" << generator.witness_norm
           << ",\"witness_root\":\"" << witness_root
           << "\",\"certified\":"
           << (generator.certified ? "true" : "false") << "}"
           << ",\"comparison\":{\"r48_psi\":" << AL_R49_R48_PSI
           << ",\"r48_h\":" << AL_R49_R48_H
           << ",\"r48_maximum_upper\":" << AL_R49_R48_MAXIMUM_UPPER
           << ",\"strict_terminal_dominance\":"
           << (dominates ? "true" : "false") << "}"
           << ",\"work\":{\"line_jvp\":" << generator.line_jvp
           << ",\"certificate_jvp\":" << generator.certificate_jvp
           << ",\"vjp\":" << generator.vjp
           << ",\"pair_passes\":" << generator.pair_passes
           << ",\"exact\":" << (work_exact ? "true" : "false") << "}"
           << ",\"dense_root\":\"" << dense_root << "\""
           << ",\"route_cases\":" << expected_routes.size()
           << ",\"route_root\":\"" << route_root
           << "\",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":" << (rollback ? "true" : "false")
           << ",\"route\":\"" << route
           << "\",\"normal_witness_applied\":false"
              ",\"r43_restoration_committed\":false"
              ",\"filter_runtime_committed\":false"
              ",\"restoration_exit_authorized\":false"
              ",\"following_outer_executed\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    return {passed, report.str()};
}
