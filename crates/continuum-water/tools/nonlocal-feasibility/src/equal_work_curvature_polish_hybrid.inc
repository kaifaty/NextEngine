namespace {

constexpr const char* B4E2D7R19R36_IDENTITY_SHA256 =
    "2e5cd1cb1fc8722c7d75fdf81c59e0d999809d8a9c00886b5260de62b80e3054";
constexpr const char* B4E2D7R19R36_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r36-equal-work-curvature-polish-hybrid|v1|parent=a1b0f1e8:8f4161e6d4cf9ac24d1ee828dec7019ae0179bd5979c97ea8fec3f2910446fe9:bacfbc3d6a3a620d83bddac51cde4133169100e7300dd9850ae1851d6997c368|source=v8-from-r33;r35-outer-prefix-b35a0e69d29316e6a53836e5c5d689d1ad6b915ea46dda968149459f587b1fa2,6457aeb488feff2b4526a1d227ecd13a6f0c37f7b4045e70102ce07eae189391,9520017434f95d8c1cabd9217dcfa88a1ac589efb03526420313bce9ff708c53;baseline-r34-phi-1.6477454905319591e-16;violation-1.8153487216135412e-8;mapping-1.4988794937300478e-9|model=all-row-squared-hinge;trust-global-l2-delta0.25|hybrid=curvature-outers3;cg-max5;undamped;then-projected-exact-line-polish6|work=prefix1+curvature36+polish12+terminal2;pair-pass-cap51;hvp15|comparison=strict-terminal-phi-violation-mapping-dominance-over-r34|controls=dense-curvature-prefix;dense-polish-stationarity;dense-active-switch;parent;source;workspace;prefix;curvature;polish;direct;work;rollback|routes=hybrid-parent-rejected;hybrid-source-rejected;hybrid-dense-rejected;hybrid-workspace-rejected;hybrid-prefix-rejected;hybrid-curvature-rejected;hybrid-polish-rejected;hybrid-direct-rejected;hybrid-work-rejected;all-inequality-projected-stationary;first-order-reference-retained;equal-work-curvature-polish-hybrid-candidate|runs=2-clean-release-builds;1-process-each;byte-exact|correction=none;nonlinear-evaluation=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-equal-work-hybrid-classification-only)IDENTITY";

constexpr std::array<const char*, 3U> AL_R36_CURVATURE_ROOTS{{
    "b35a0e69d29316e6a53836e5c5d689d1ad6b915ea46dda968149459f587b1fa2",
    "6457aeb488feff2b4526a1d227ecd13a6f0c37f7b4045e70102ce07eae189391",
    "9520017434f95d8c1cabd9217dcfa88a1ac589efb03526420313bce9ff708c53",
}};

struct ALR36PolishRecord {
    std::size_t iteration = 0U;
    double gradient_norm = 0.0;
    double step_norm = 0.0;
    double line_alpha = 0.0;
    double phi = 0.0;
    double violation_norm = 0.0;
    std::size_t active = 0U;
    std::string line_root;
    std::string root;
};

bool al_r36_dense_controls(std::string* output_root) {
    std::string curvature_root;
    const bool curvature = al_r35_dense_controls(&curvature_root);
    ALR33DenseControl polish = al_r33_dense_run(
        "hybrid-polish-stationary", {{1.0, 0.0}, {0.0, 4.0}},
        {1.0, 1.0}, 2.0, 6U);
    ALR33DenseControl active_switch = al_r33_dense_run(
        "hybrid-active-switch", {{-1.0}, {1.0}}, {1.0, -0.1}, 1.0, 6U);
    polish.passed = polish.passed && polish.stationary
        && polish.accepted == 2U && polish.phi <= 1.0e-28;
    active_switch.passed = active_switch.passed && active_switch.stationary
        && active_switch.accepted == 1U && active_switch.active == 2U;
    const bool exact = curvature && polish.passed && active_switch.passed;
    std::ostringstream projection;
    projection << curvature_root << ':' << polish.root << ':'
               << active_switch.root << ':' << exact;
    if (output_root != nullptr) *output_root = sha256_hex(projection.str());
    return exact;
}

std::string al_r36_route(
    bool parent, bool source, bool dense, bool workspace, bool prefix,
    bool curvature, bool polish, bool direct, bool work, bool stationary,
    bool dominates) {
    if (!parent) return "HYBRID_PARENT_REJECTED";
    if (!source) return "HYBRID_SOURCE_REJECTED";
    if (!dense) return "HYBRID_DENSE_REJECTED";
    if (!workspace) return "HYBRID_WORKSPACE_REJECTED";
    if (!prefix) return "HYBRID_PREFIX_REJECTED";
    if (!curvature) return "HYBRID_CURVATURE_REJECTED";
    if (!polish) return "HYBRID_POLISH_REJECTED";
    if (!direct) return "HYBRID_DIRECT_REJECTED";
    if (!work) return "HYBRID_WORK_REJECTED";
    if (stationary) return "ALL_INEQUALITY_PROJECTED_STATIONARY";
    return dominates
        ? "EQUAL_WORK_CURVATURE_POLISH_HYBRID_CANDIDATE"
        : "FIRST_ORDER_REFERENCE_RETAINED";
}

} // namespace

SplitBoundaryReport run_al_equal_work_curvature_polish_hybrid_controls() {
    const bool identity_exact = sha256_hex(B4E2D7R19R36_IDENTITY_PROJECTION)
        == B4E2D7R19R36_IDENTITY_SHA256;
    ALR35CurvatureCapture parent_capture;
    const SplitBoundaryReport parent =
        run_al_equal_work_generalized_hessian_controls_impl(&parent_capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "bacfbc3d6a3a620d83bddac51cde4133169100e7300dd9850ae1851d6997c368"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && parent_capture.captured
        && parent_semantic && parent_stdout
            == "8f4161e6d4cf9ac24d1ee828dec7019ae0179bd5979c97ea8fec3f2910446fe9";
    ALR34ReferenceCapture& baseline = parent_capture.r34;
    ALR33NormalStepCapture& r33 = baseline.r33;
    ALR30RangeProjectionCapture& r30 = r33.r32.r31.r30;
    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string dual_before = al_r10_dual_root(r30.dual);
    const std::string constraint_before = al_r10_dual_root(r30.constraint);
    const double baseline_mapping_norm =
        al_r32_vec3_norm(baseline.projected_mapping);
    const bool source_exact = parent_exact
        && parent_capture.outer_records.size() == 4U
        && parent_capture.outer_records[0].root == AL_R36_CURVATURE_ROOTS[0]
        && parent_capture.outer_records[1].root == AL_R36_CURVATURE_ROOTS[1]
        && parent_capture.outer_records[2].root == AL_R36_CURVATURE_ROOTS[2]
        && binary64_bits(baseline.terminal.phi)
            == binary64_bits(1.6477454905319591e-16)
        && binary64_bits(baseline.terminal.positive_norm)
            == binary64_bits(1.8153487216135412e-8)
        && binary64_bits(baseline_mapping_norm)
            == binary64_bits(1.4988794937300478e-9)
        && al_r10_dual_root(r33.terminal_response)
            == "7dd519932782f9177be354418c9ecaaa1fed3baac1e8255305675da7cb59dff3"
        && r30.state_root
            == "851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7"
        && r33.iterate.size() == 6000U
        && r33.maintained_response.size() == 6000U;

    std::string dense_root;
    const bool dense_exact = al_r36_dense_controls(&dense_root);
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
    std::size_t hvp_count = 0U;
    ALLinearizedJvp prefix_response;
    if (workspace_exact) {
        prefix_response = al_r29_pair_once_jvp(workspace, r33.iterate);
        ++pair_passes;
    }
    const double prefix_relative = prefix_response.passed
        ? al_r33_vec_diff_relative(
            r33.maintained_response, prefix_response.value)
        : std::numeric_limits<double>::infinity();
    const bool prefix_exact = prefix_response.passed
        && al_r10_dual_root(prefix_response.value)
            == "7dd519932782f9177be354418c9ecaaa1fed3baac1e8255305675da7cb59dff3"
        && std::isfinite(prefix_relative) && prefix_relative <= 1.0e-12;

    std::vector<Vec3> iterate = r33.iterate;
    std::vector<double> maintained_response = r33.maintained_response;
    std::vector<double> residual(r30.constraint.size(), 0.0);
    for (std::size_t row = 0U; row < residual.size(); ++row) {
        residual[row] = r30.constraint[row] + maintained_response[row];
    }
    std::vector<ALR35OuterRecord> curvature_records;
    bool curvature_exact = prefix_exact;
    bool projected_stationary = false;
    std::size_t curvature_stops = 0U;
    for (std::size_t outer = 0U; outer < 3U && curvature_exact
            && !projected_stationary; ++outer) {
        std::vector<double> positive(residual.size(), 0.0);
        for (std::size_t row = 0U; row < residual.size(); ++row) {
            positive[row] = std::max(residual[row], 0.0);
        }
        const std::vector<Vec3> gradient =
            al_r29_pair_once_vjp(workspace, positive);
        ++pair_passes;
        const double gradient_norm = al_r32_vec3_norm(gradient);
        if (!al_r32_vec3_finite(gradient) || !std::isfinite(gradient_norm)
            || gradient_norm == 0.0) {
            curvature_exact = false;
            break;
        }
        std::vector<Vec3> step(iterate.size());
        std::vector<Vec3> cg_residual(iterate.size());
        std::vector<Vec3> direction(iterate.size());
        for (std::size_t particle = 0U; particle < iterate.size(); ++particle) {
            cg_residual[particle] = -gradient[particle];
            direction[particle] = cg_residual[particle];
        }
        long double rr = al_r35_dot_vec3(cg_residual, cg_residual);
        const double cg_initial_norm = std::sqrt(static_cast<double>(rr));
        std::size_t cg_iterations = 0U;
        std::size_t outer_curvature_stops = 0U;
        for (std::size_t inner = 0U; inner < 5U && rr != 0.0L; ++inner) {
            const ALLinearizedJvp image =
                al_r29_pair_once_jvp(workspace, direction);
            ++pair_passes;
            if (!image.passed) {
                curvature_exact = false;
                break;
            }
            std::vector<double> masked(image.value.size(), 0.0);
            for (std::size_t row = 0U; row < masked.size(); ++row) {
                if (residual[row] > 0.0) masked[row] = image.value[row];
            }
            const std::vector<Vec3> hessian_direction =
                al_r29_pair_once_vjp(workspace, masked);
            ++pair_passes;
            ++hvp_count;
            if (!al_r32_vec3_finite(hessian_direction)) {
                curvature_exact = false;
                break;
            }
            const long double curvature =
                al_r35_dot_vec3(direction, hessian_direction);
            if (!std::isfinite(static_cast<double>(curvature))) {
                curvature_exact = false;
                break;
            }
            if (curvature <= 0.0L) {
                ++outer_curvature_stops;
                ++curvature_stops;
                break;
            }
            const long double alpha = rr / curvature;
            for (std::size_t particle = 0U;
                 particle < iterate.size(); ++particle) {
                step[particle] += static_cast<double>(alpha)
                    * direction[particle];
                cg_residual[particle] += -static_cast<double>(alpha)
                    * hessian_direction[particle];
            }
            const long double rr_next =
                al_r35_dot_vec3(cg_residual, cg_residual);
            ++cg_iterations;
            if (!std::isfinite(static_cast<double>(rr_next)) || rr_next < 0.0L) {
                curvature_exact = false;
                break;
            }
            if (rr_next == 0.0L) {
                rr = rr_next;
                break;
            }
            const long double beta = rr_next / rr;
            for (std::size_t particle = 0U;
                 particle < iterate.size(); ++particle) {
                direction[particle] = cg_residual[particle]
                    + static_cast<double>(beta) * direction[particle];
            }
            rr = rr_next;
        }
        if (!curvature_exact) break;
        const double cg_final_norm = std::sqrt(static_cast<double>(rr));
        const double descent = static_cast<double>(
            al_r35_dot_vec3(gradient, step));
        curvature_exact = al_r32_vec3_finite(step)
            && std::isfinite(cg_final_norm) && std::isfinite(descent)
            && descent < 0.0 && cg_iterations == 5U
            && outer_curvature_stops == 0U;
        if (!curvature_exact) break;

        std::vector<Vec3> trial(iterate.size());
        for (std::size_t particle = 0U; particle < iterate.size(); ++particle) {
            trial[particle] = iterate[particle] + step[particle];
        }
        const std::vector<Vec3> target = al_r33_project_vec3(
            trial, AL_R32_TRUST_DELTA);
        if (target.size() != iterate.size()) {
            curvature_exact = false;
            break;
        }
        std::vector<Vec3> line_direction(iterate.size());
        for (std::size_t particle = 0U; particle < iterate.size(); ++particle) {
            line_direction[particle] = target[particle] - iterate[particle];
        }
        const double chord_norm = al_r32_vec3_norm(line_direction);
        if (chord_norm == 0.0) {
            curvature_exact = false;
            break;
        }
        for (Vec3& value : line_direction) {
            value = (1.0 / chord_norm) * value;
        }
        const ALLinearizedJvp line_response =
            al_r29_pair_once_jvp(workspace, line_direction);
        ++pair_passes;
        if (!line_response.passed) {
            curvature_exact = false;
            break;
        }
        const ALR32LineResult line = al_r32_piecewise_line(
            residual, line_response.value, chord_norm);
        const double derivative_scale = std::max(
            std::abs(line.initial.derivative), 1.0e-300);
        const bool line_kkt = line.trust_active
            ? line.final.derivative <= 0.0
            : std::abs(line.final.derivative) / derivative_scale <= 1.0e-10;
        curvature_exact = line.passed && line_kkt
            && line.final.phi < line.initial.phi;
        if (!curvature_exact) break;
        for (std::size_t particle = 0U; particle < iterate.size(); ++particle) {
            iterate[particle] += line.alpha * line_direction[particle];
        }
        for (std::size_t row = 0U; row < residual.size(); ++row) {
            maintained_response[row] += line.alpha * line_response.value[row];
            residual[row] = r30.constraint[row] + maintained_response[row];
        }
        curvature_exact = al_r32_vec3_norm(iterate)
            <= std::nextafter(AL_R32_TRUST_DELTA,
                std::numeric_limits<double>::infinity());
        ALR35OuterRecord record;
        record.outer = outer + 1U;
        record.cg_iterations = cg_iterations;
        record.curvature_stops = outer_curvature_stops;
        record.cg_initial_norm = cg_initial_norm;
        record.cg_final_norm = cg_final_norm;
        record.descent = descent;
        record.line_alpha = line.alpha;
        record.phi = line.final.phi;
        record.violation_norm = line.final.positive_norm;
        record.active = line.final.active;
        record.line_root = line.root;
        std::ostringstream projection;
        projection << record.outer << ':' << record.cg_iterations << ':'
                   << record.curvature_stops << ':'
                   << binary64_bits(record.cg_initial_norm) << ':'
                   << binary64_bits(record.cg_final_norm) << ':'
                   << binary64_bits(record.descent) << ':'
                   << binary64_bits(record.line_alpha) << ':'
                   << binary64_bits(record.phi) << ':'
                   << binary64_bits(record.violation_norm) << ':'
                   << record.active << ':' << record.line_root;
        record.root = sha256_hex(projection.str());
        curvature_exact = curvature_exact
            && record.root == AL_R36_CURVATURE_ROOTS[outer]
            && record.root == parent_capture.outer_records[outer].root;
        curvature_records.push_back(std::move(record));
    }
    curvature_exact = curvature_exact && curvature_records.size() == 3U
        && pair_passes == 37U && hvp_count == 15U
        && curvature_stops == 0U;

    std::vector<ALR36PolishRecord> polish_records;
    bool polish_exact = curvature_exact;
    for (std::size_t polish = 0U; polish < 6U && polish_exact; ++polish) {
        std::vector<double> positive(residual.size(), 0.0);
        for (std::size_t row = 0U; row < residual.size(); ++row) {
            positive[row] = std::max(residual[row], 0.0);
        }
        const std::vector<Vec3> gradient =
            al_r29_pair_once_vjp(workspace, positive);
        ++pair_passes;
        const double gradient_norm = al_r32_vec3_norm(gradient);
        if (!al_r32_vec3_finite(gradient) || !std::isfinite(gradient_norm)) {
            polish_exact = false;
            break;
        }
        if (gradient_norm == 0.0) {
            projected_stationary = true;
            polish_exact = false;
            break;
        }
        std::vector<Vec3> target(iterate.size());
        for (std::size_t particle = 0U; particle < iterate.size(); ++particle) {
            target[particle] = iterate[particle]
                - (1.0 / gradient_norm) * gradient[particle];
        }
        target = al_r33_project_vec3(target, AL_R32_TRUST_DELTA);
        if (target.size() != iterate.size()) {
            polish_exact = false;
            break;
        }
        std::vector<Vec3> direction(iterate.size());
        for (std::size_t particle = 0U; particle < iterate.size(); ++particle) {
            direction[particle] = target[particle] - iterate[particle];
        }
        const double step_norm = al_r32_vec3_norm(direction);
        if (step_norm == 0.0) {
            projected_stationary = true;
            polish_exact = false;
            break;
        }
        for (Vec3& value : direction) value = (1.0 / step_norm) * value;
        const ALLinearizedJvp line_response =
            al_r29_pair_once_jvp(workspace, direction);
        ++pair_passes;
        if (!line_response.passed) {
            polish_exact = false;
            break;
        }
        const ALR32LineResult line = al_r32_piecewise_line(
            residual, line_response.value, step_norm);
        const double derivative_scale = std::max(
            std::abs(line.initial.derivative), 1.0e-300);
        const bool line_kkt = line.trust_active
            ? line.final.derivative <= 0.0
            : std::abs(line.final.derivative) / derivative_scale <= 1.0e-10;
        polish_exact = line.passed && line_kkt
            && line.final.phi < line.initial.phi;
        if (!polish_exact) break;
        for (std::size_t particle = 0U; particle < iterate.size(); ++particle) {
            iterate[particle] += line.alpha * direction[particle];
        }
        for (std::size_t row = 0U; row < residual.size(); ++row) {
            maintained_response[row] += line.alpha * line_response.value[row];
            residual[row] = r30.constraint[row] + maintained_response[row];
        }
        polish_exact = al_r32_vec3_norm(iterate)
            <= std::nextafter(AL_R32_TRUST_DELTA,
                std::numeric_limits<double>::infinity());
        ALR36PolishRecord record;
        record.iteration = polish + 1U;
        record.gradient_norm = gradient_norm;
        record.step_norm = step_norm;
        record.line_alpha = line.alpha;
        record.phi = line.final.phi;
        record.violation_norm = line.final.positive_norm;
        record.active = line.final.active;
        record.line_root = line.root;
        std::ostringstream projection;
        projection << record.iteration << ':'
                   << binary64_bits(record.gradient_norm) << ':'
                   << binary64_bits(record.step_norm) << ':'
                   << binary64_bits(record.line_alpha) << ':'
                   << binary64_bits(record.phi) << ':'
                   << binary64_bits(record.violation_norm) << ':'
                   << record.active << ':' << record.line_root;
        record.root = sha256_hex(projection.str());
        polish_records.push_back(std::move(record));
    }
    polish_exact = polish_exact && polish_records.size() == 6U
        && pair_passes == 49U;

    ALLinearizedJvp terminal_response;
    if (workspace_exact && polish_exact) {
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
    projected_stationary = direct_exact && projected_mapping_norm == 0.0;
    const bool dominates = direct_exact
        && terminal.phi < baseline.terminal.phi
        && terminal.positive_norm < baseline.terminal.positive_norm
        && projected_mapping_norm < baseline_mapping_norm;

    release_al_normalized_workspace(workspace, &work);
    const bool work_exact = pair_passes == 51U && hvp_count == 15U
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
        {"EQUAL_WORK_CURVATURE_POLISH_HYBRID_CANDIDATE", al_r36_route(1,1,1,1,1,1,1,1,1,0,1)},
        {"FIRST_ORDER_REFERENCE_RETAINED", al_r36_route(1,1,1,1,1,1,1,1,1,0,0)},
        {"ALL_INEQUALITY_PROJECTED_STATIONARY", al_r36_route(1,1,1,1,1,1,1,1,1,1,0)},
        {"HYBRID_PARENT_REJECTED", al_r36_route(0,1,1,1,1,1,1,1,1,0,0)},
        {"HYBRID_SOURCE_REJECTED", al_r36_route(1,0,1,1,1,1,1,1,1,0,0)},
        {"HYBRID_DENSE_REJECTED", al_r36_route(1,1,0,1,1,1,1,1,1,0,0)},
        {"HYBRID_WORKSPACE_REJECTED", al_r36_route(1,1,1,0,1,1,1,1,1,0,0)},
        {"HYBRID_PREFIX_REJECTED", al_r36_route(1,1,1,1,0,1,1,1,1,0,0)},
        {"HYBRID_CURVATURE_REJECTED", al_r36_route(1,1,1,1,1,0,1,1,1,0,0)},
        {"HYBRID_POLISH_REJECTED", al_r36_route(1,1,1,1,1,1,0,1,1,0,0)},
        {"HYBRID_DIRECT_REJECTED", al_r36_route(1,1,1,1,1,1,1,0,1,0,0)},
        {"HYBRID_WORK_REJECTED", al_r36_route(1,1,1,1,1,1,1,1,0,0,0)},
    }};
    bool routes_exact = true;
    std::ostringstream route_projection;
    for (const auto& value : routes) {
        routes_exact = routes_exact && value.first == value.second;
        route_projection << value.first << ':' << value.second << '|';
    }
    const std::string route_root = sha256_hex(route_projection.str());
    const std::string route = al_r36_route(
        parent_exact, source_exact, dense_exact, workspace_exact, prefix_exact,
        curvature_exact, polish_exact, direct_exact && rollback_exact,
        work_exact, projected_stationary, dominates);
    const bool passed = identity_exact && parent_exact && source_exact
        && dense_exact && workspace_exact && prefix_exact && curvature_exact
        && polish_exact && direct_exact && work_exact && rollback_exact
        && routes_exact;
    std::string first_failure;
    if (!identity_exact) first_failure = "IDENTITY";
    else if (!parent_exact) first_failure = "PARENT";
    else if (!source_exact) first_failure = "SOURCE";
    else if (!dense_exact) first_failure = "DENSE";
    else if (!workspace_exact) first_failure = "WORKSPACE";
    else if (!prefix_exact) first_failure = "PREFIX";
    else if (!curvature_exact) first_failure = "CURVATURE";
    else if (!polish_exact) first_failure = "POLISH";
    else if (!direct_exact) first_failure = "DIRECT";
    else if (!work_exact) first_failure = "WORK";
    else if (!rollback_exact) first_failure = "ROLLBACK";
    else if (!routes_exact) first_failure = "ROUTES";

    std::ostringstream curvature_projection;
    for (const ALR35OuterRecord& record : curvature_records) {
        curvature_projection << record.root << '|';
    }
    const std::string curvature_root =
        sha256_hex(curvature_projection.str());
    std::ostringstream polish_projection;
    for (const ALR36PolishRecord& record : polish_records) {
        polish_projection << record.root << '|';
    }
    const std::string polish_root = sha256_hex(polish_projection.str());
    const std::string terminal_response_root = terminal_response.passed
        ? al_r10_dual_root(terminal_response.value) : sha256_hex("missing");
    const std::string terminal_gradient_root =
        al_binary64_vec3_root(terminal_gradient);
    const std::string projected_mapping_root =
        al_binary64_vec3_root(projected_mapping);
    const double phi_ratio = terminal.phi / baseline.terminal.phi;
    const double violation_ratio =
        terminal.positive_norm / baseline.terminal.positive_norm;
    const double mapping_ratio = projected_mapping_norm / baseline_mapping_norm;
    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R36_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << dense_root << ':' << curvature_root << ':' << polish_root
             << ':' << terminal_response_root << ':' << terminal_gradient_root
             << ':' << projected_mapping_root << '|'
             << binary64_bits(phi_ratio) << ':'
             << binary64_bits(violation_ratio) << ':'
             << binary64_bits(mapping_ratio) << ':' << pair_passes << ':'
             << hvp_count << '|' << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4e2d7r19r36_equal_work_curvature_polish_hybrid.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R36_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source\":{\"prefix_response_root\":\""
           << (prefix_response.passed
               ? al_r10_dual_root(prefix_response.value) : sha256_hex("missing"))
           << "\",\"prefix_response_relative\":" << prefix_relative << "}"
           << ",\"dense\":{\"passed\":"
           << (dense_exact ? "true" : "false")
           << ",\"root\":\"" << dense_root << "\"}"
           << ",\"curvature\":{\"outer_count\":" << curvature_records.size()
           << ",\"hvp_count\":" << hvp_count
           << ",\"curvature_stops\":" << curvature_stops
           << ",\"records\":[";
    for (std::size_t index_value = 0U;
         index_value < curvature_records.size(); ++index_value) {
        if (index_value != 0U) report << ',';
        const ALR35OuterRecord& record = curvature_records[index_value];
        report << "{\"outer\":" << record.outer
               << ",\"cg_iterations\":" << record.cg_iterations
               << ",\"cg_initial_norm\":" << record.cg_initial_norm
               << ",\"cg_final_norm\":" << record.cg_final_norm
               << ",\"line_alpha\":" << record.line_alpha
               << ",\"phi\":" << record.phi
               << ",\"violation_norm\":" << record.violation_norm
               << ",\"active\":" << record.active
               << ",\"root\":\"" << record.root << "\"}";
    }
    report << "],\"root\":\"" << curvature_root << "\"}"
           << ",\"polish\":{\"accepted\":" << polish_records.size()
           << ",\"records\":[";
    for (std::size_t index_value = 0U;
         index_value < polish_records.size(); ++index_value) {
        if (index_value != 0U) report << ',';
        const ALR36PolishRecord& record = polish_records[index_value];
        report << "{\"iteration\":" << record.iteration
               << ",\"gradient_norm\":" << record.gradient_norm
               << ",\"step_norm\":" << record.step_norm
               << ",\"line_alpha\":" << record.line_alpha
               << ",\"phi\":" << record.phi
               << ",\"violation_norm\":" << record.violation_norm
               << ",\"active\":" << record.active
               << ",\"root\":\"" << record.root << "\"}";
    }
    report << "],\"root\":\"" << polish_root << "\"}"
           << ",\"comparison\":{\"baseline_phi\":" << baseline.terminal.phi
           << ",\"candidate_phi\":" << terminal.phi
           << ",\"phi_ratio\":" << phi_ratio
           << ",\"baseline_violation_norm\":"
           << baseline.terminal.positive_norm
           << ",\"candidate_violation_norm\":" << terminal.positive_norm
           << ",\"violation_ratio\":" << violation_ratio
           << ",\"baseline_projected_mapping_norm\":"
           << baseline_mapping_norm
           << ",\"candidate_projected_mapping_norm\":"
           << projected_mapping_norm
           << ",\"mapping_ratio\":" << mapping_ratio
           << ",\"strict_dominance\":" << (dominates ? "true" : "false")
           << "}"
           << ",\"terminal\":{\"active\":" << terminal.active
           << ",\"iterate_norm\":" << al_r32_vec3_norm(iterate)
           << ",\"gradient_norm\":" << terminal_gradient_norm
           << ",\"response_relative\":" << response_relative
           << ",\"response_root\":\"" << terminal_response_root
           << "\",\"gradient_root\":\"" << terminal_gradient_root
           << "\",\"projected_mapping_root\":\""
           << projected_mapping_root << "\"}"
           << ",\"work\":{\"parent_r35_replays\":1"
           << ",\"diagnostic_workspaces\":1,\"new_pair_passes\":"
           << pair_passes << ",\"new_hvp\":" << hvp_count
           << ",\"new_model\":0,\"new_trials\":0,\"new_outer\":0}"
           << ",\"route_cases\":" << routes.size()
           << ",\"route_root\":\"" << route_root << "\""
           << ",\"routes_exact\":" << (routes_exact ? "true" : "false")
           << ",\"rollback_exact\":"
           << (rollback_exact ? "true" : "false")
           << ",\"first_failure\":\"" << first_failure << "\""
           << ",\"route\":\"" << route << "\""
              ",\"correction_applied\":false,\"nonlinear_evaluation\":false"
              ",\"state_mutation\":false,\"timing_admitted\":false"
              ",\"runtime_authority\":false,\"production_authority\":false"
           << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {passed, report.str()};
}
