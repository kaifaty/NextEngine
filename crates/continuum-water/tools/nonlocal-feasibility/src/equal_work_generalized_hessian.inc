namespace {

constexpr const char* B4E2D7R19R35_IDENTITY_SHA256 =
    "9462749f5b1f1113537e9bc1b57883198571721e88e4a2ddaf8f255ebedc4708";
constexpr const char* B4E2D7R19R35_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r35-equal-work-generalized-hessian|v1|parent=7db0857e:5150f1f697072a21a81ce7d6fc7aa4548ce603e7e27796143483fa0ad796f345:00dcf602a9266d8170a8c84af1f338903650f4c767a24d7cf00249a9837583b7|source=v8-from-r33;prefix-response-7dd519932782f9177be354418c9ecaaa1fed3baac1e8255305675da7cb59dff3;baseline-checkpoint-35db4ad52c1fbd8d99e4ada888ecc5589df1cdfb39e3f9a73cd1323494326dc3;baseline-response-ed311a3272f1cd21049db2fa57cb40d6a9b3dd23a9e629b0cfd9f5f514d62193;baseline-gradient-49edafddd1de7d26262831752dc14269a6c6a72b317983c5b75c626939b66996;baseline-mapping-f536d409dc5679e2343d05a418d0a33637f9ec1db65c52e25f8eb49a1e0d1c07;particles6000;pairs340340;max-degree113|model=phi(v)=0.5*norm(max(c+A*v,0))2;A=SPACING*Jc;all-rows;support-fixed;H=At*Dactive*A|trust=global-l2;delta0.25;fixed|curvature=outer4;cg-max5;unpreconditioned;undamped;zero-initial-direction;consistent-range;hvp=jvp-mask-vjp;zero-or-nonpositive-curvature-stop|globalization=z=project-ball(v+p,delta);d=(z-v)/norm(z-v);exact-r32-piecewise-line;all-rows|comparison=same-v8;pair-pass-cap51;strict-terminal-phi-violation-mapping-dominance-over-r34|terminal=fresh-jvp-v;fresh-vjp-positive;maintained-direct-response-relative<=1e-12;projected-mapping|controls=dense-spd-newton;dense-singular-consistent;dense-active-switch;dense-trust-projection;parent;source;workspace;prefix;cg;globalization;direct;work;rollback|observations=outer-cg-residual-curvature-step-active;terminal-roots;baseline-candidate-ratios;trust-use|nullspace-stops|routes=curvature-parent-rejected;curvature-source-rejected;curvature-dense-rejected;curvature-workspace-rejected;curvature-prefix-rejected;curvature-cg-rejected;curvature-globalization-rejected;curvature-direct-rejected;curvature-work-rejected;all-inequality-projected-stationary;first-order-reference-retained;equal-work-generalized-hessian-candidate|precedence=parent,source,dense,workspace,prefix,cg,globalization,direct,work,classification|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-r34-once;diagnostic-workspaces1;prefix-passes1;outer4*(gradient1+cg5*hvp2+line1)=48;terminal-passes2;new-pair-passes<=51;new-hvp<=20;new-model0;new-trial0;new-outer0|correction=none;nonlinear-evaluation=none;floor-classification=none;state-mutation=none;following-outer=none;substep=none;macro=none;trajectory=none;timing=none;public-schema=none;runtime=none;production=none|credit=one-private-equal-work-generalized-hessian-classification-only)IDENTITY";

struct ALR35DenseCg {
    bool passed = false;
    std::vector<double> step;
    double residual_norm = 0.0;
    std::size_t iterations = 0U;
    std::size_t curvature_stops = 0U;
    std::string root;
};

struct ALR35OuterRecord {
    std::size_t outer = 0U;
    std::size_t cg_iterations = 0U;
    std::size_t curvature_stops = 0U;
    double cg_initial_norm = 0.0;
    double cg_final_norm = 0.0;
    double descent = 0.0;
    double line_alpha = 0.0;
    double phi = 0.0;
    double violation_norm = 0.0;
    std::size_t active = 0U;
    std::string line_root;
    std::string root;
};

struct ALR35CurvatureCapture {
    bool captured = false;
    ALR34ReferenceCapture r34;
    std::vector<ALR35OuterRecord> outer_records;
};

long double al_r35_dot_vec3(
    const std::vector<Vec3>& lhs,
    const std::vector<Vec3>& rhs) {
    if (lhs.size() != rhs.size()) {
        return std::numeric_limits<long double>::quiet_NaN();
    }
    long double result = 0.0L;
    for (std::size_t index = 0U; index < lhs.size(); ++index) {
        result += static_cast<long double>(dot(lhs[index], rhs[index]));
    }
    return result;
}

std::vector<double> al_r35_dense_matvec(
    const std::vector<std::vector<double>>& matrix,
    const std::vector<double>& value) {
    std::vector<double> result(matrix.size(), 0.0);
    for (std::size_t row = 0U; row < matrix.size(); ++row) {
        long double item = 0.0L;
        for (std::size_t column = 0U; column < value.size(); ++column) {
            item += static_cast<long double>(matrix[row][column])
                * value[column];
        }
        result[row] = static_cast<double>(item);
    }
    return result;
}

ALR35DenseCg al_r35_dense_cg(
    const std::vector<std::vector<double>>& hessian,
    const std::vector<double>& gradient) {
    ALR35DenseCg result;
    if (hessian.empty() || hessian.size() != gradient.size()) return result;
    std::vector<double> residual(gradient.size(), 0.0);
    std::vector<double> direction(gradient.size(), 0.0);
    result.step.assign(gradient.size(), 0.0);
    for (std::size_t index = 0U; index < gradient.size(); ++index) {
        residual[index] = -gradient[index];
        direction[index] = residual[index];
    }
    long double rr = al_r30_dot(residual, residual);
    const long double initial = rr;
    bool exact = std::isfinite(static_cast<double>(rr));
    for (std::size_t iteration = 0U; iteration < 5U && rr != 0.0L;
         ++iteration) {
        const std::vector<double> image =
            al_r35_dense_matvec(hessian, direction);
        const long double curvature = al_r30_dot(direction, image);
        if (!std::isfinite(static_cast<double>(curvature))) {
            exact = false;
            break;
        }
        if (curvature <= 0.0L) {
            ++result.curvature_stops;
            break;
        }
        const long double alpha = rr / curvature;
        for (std::size_t index = 0U; index < gradient.size(); ++index) {
            result.step[index] += static_cast<double>(alpha) * direction[index];
            residual[index] -= static_cast<double>(alpha) * image[index];
        }
        const long double rr_next = al_r30_dot(residual, residual);
        ++result.iterations;
        if (!std::isfinite(static_cast<double>(rr_next)) || rr_next < 0.0L) {
            exact = false;
            break;
        }
        if (rr_next == 0.0L) {
            rr = rr_next;
            break;
        }
        const long double beta = rr_next / rr;
        for (std::size_t index = 0U; index < gradient.size(); ++index) {
            direction[index] = residual[index]
                + static_cast<double>(beta) * direction[index];
        }
        rr = rr_next;
    }
    result.residual_norm = std::sqrt(static_cast<double>(rr));
    result.passed = exact && initial >= rr
        && std::all_of(result.step.begin(), result.step.end(),
            [](double value) { return std::isfinite(value); });
    std::ostringstream projection;
    projection << result.passed << ':' << result.iterations << ':'
               << result.curvature_stops << ':'
               << binary64_bits(result.residual_norm) << ':';
    for (double value : result.step) projection << binary64_bits(value) << ':';
    result.root = sha256_hex(projection.str());
    return result;
}

bool al_r35_dense_controls(std::string* output_root) {
    const ALR35DenseCg spd = al_r35_dense_cg(
        {{2.0, 0.0}, {0.0, 4.0}}, {2.0, 4.0});
    const ALR35DenseCg singular = al_r35_dense_cg(
        {{1.0, 0.0}, {0.0, 0.0}}, {1.0, 0.0});
    const ALR35DenseCg active_cg = al_r35_dense_cg({{1.0}}, {-1.0});
    const ALR32LineResult active_line = al_r32_piecewise_line(
        {1.0, -0.1}, {-1.0, 1.0}, 1.0);
    const std::vector<double> projected = al_r33_dense_project({10.9}, 1.0);
    const bool exact = spd.passed && spd.iterations <= 5U
        && spd.residual_norm <= 1.0e-14
        && std::abs(spd.step[0] + 1.0) <= 1.0e-14
        && std::abs(spd.step[1] + 1.0) <= 1.0e-14
        && singular.passed && singular.iterations == 1U
        && singular.residual_norm == 0.0
        && binary64_bits(singular.step[0]) == binary64_bits(-1.0)
        && singular.step[1] == 0.0
        && active_cg.passed && active_cg.iterations == 1U
        && active_line.passed
        && binary64_bits(active_line.alpha) == binary64_bits(0.55)
        && active_line.final.active == 2U
        && projected.size() == 1U
        && std::abs(projected[0] - 1.0) <= 1.0e-15;
    std::ostringstream projection;
    projection << spd.root << ':' << singular.root << ':' << active_cg.root
               << ':' << active_line.root << ':'
               << (projected.empty() ? 0ULL : binary64_bits(projected[0]))
               << ':' << exact;
    if (output_root != nullptr) *output_root = sha256_hex(projection.str());
    return exact;
}

std::string al_r35_route(
    bool parent, bool source, bool dense, bool workspace, bool prefix, bool cg,
    bool globalization, bool direct, bool work, bool stationary,
    bool dominates) {
    if (!parent) return "CURVATURE_PARENT_REJECTED";
    if (!source) return "CURVATURE_SOURCE_REJECTED";
    if (!dense) return "CURVATURE_DENSE_REJECTED";
    if (!workspace) return "CURVATURE_WORKSPACE_REJECTED";
    if (!prefix) return "CURVATURE_PREFIX_REJECTED";
    if (!cg) return "CURVATURE_CG_REJECTED";
    if (!globalization) return "CURVATURE_GLOBALIZATION_REJECTED";
    if (!direct) return "CURVATURE_DIRECT_REJECTED";
    if (!work) return "CURVATURE_WORK_REJECTED";
    if (stationary) return "ALL_INEQUALITY_PROJECTED_STATIONARY";
    return dominates
        ? "EQUAL_WORK_GENERALIZED_HESSIAN_CANDIDATE"
        : "FIRST_ORDER_REFERENCE_RETAINED";
}

} // namespace

SplitBoundaryReport run_al_equal_work_generalized_hessian_controls_impl(
    ALR35CurvatureCapture* output_capture) {
    const bool identity_exact = sha256_hex(B4E2D7R19R35_IDENTITY_PROJECTION)
        == B4E2D7R19R35_IDENTITY_SHA256;
    ALR34ReferenceCapture capture;
    const SplitBoundaryReport parent =
        run_al_first_order_reference_extension_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "00dcf602a9266d8170a8c84af1f338903650f4c767a24d7cf00249a9837583b7"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && parent_semantic && parent_stdout
            == "5150f1f697072a21a81ce7d6fc7aa4548ce603e7e27796143483fa0ad796f345";
    ALR33NormalStepCapture& r33 = capture.r33;
    ALR30RangeProjectionCapture& r30 = r33.r32.r31.r30;
    const std::string position_before = al_binary64_vec3_root(r30.position);
    const std::string dual_before = al_r10_dual_root(r30.dual);
    const std::string constraint_before = al_r10_dual_root(r30.constraint);
    std::ostringstream baseline_checkpoint_projection;
    for (const ALR34Checkpoint& checkpoint : capture.checkpoints) {
        baseline_checkpoint_projection << checkpoint.root << '|';
    }
    const std::string baseline_checkpoint_root =
        sha256_hex(baseline_checkpoint_projection.str());
    const bool source_exact = parent_exact
        && baseline_checkpoint_root
            == "35db4ad52c1fbd8d99e4ada888ecc5589df1cdfb39e3f9a73cd1323494326dc3"
        && al_r10_dual_root(capture.terminal_response)
            == "ed311a3272f1cd21049db2fa57cb40d6a9b3dd23a9e629b0cfd9f5f514d62193"
        && al_binary64_vec3_root(capture.terminal_gradient)
            == "49edafddd1de7d26262831752dc14269a6c6a72b317983c5b75c626939b66996"
        && al_binary64_vec3_root(capture.projected_mapping)
            == "f536d409dc5679e2343d05a418d0a33637f9ec1db65c52e25f8eb49a1e0d1c07"
        && al_r10_dual_root(r33.terminal_response)
            == "7dd519932782f9177be354418c9ecaaa1fed3baac1e8255305675da7cb59dff3"
        && r30.state_root
            == "851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7"
        && r33.iterate.size() == 6000U
        && r33.maintained_response.size() == 6000U;

    std::string dense_root;
    const bool dense_exact = al_r35_dense_controls(&dense_root);
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
    std::vector<ALR35OuterRecord> outer_records;
    bool cg_exact = prefix_exact;
    bool globalization_exact = prefix_exact;
    bool projected_stationary = false;
    std::size_t curvature_stops = 0U;
    for (std::size_t outer = 0U; outer < 4U && cg_exact
            && globalization_exact && !projected_stationary; ++outer) {
        std::vector<double> positive(residual.size(), 0.0);
        for (std::size_t row = 0U; row < residual.size(); ++row) {
            positive[row] = std::max(residual[row], 0.0);
        }
        const std::vector<Vec3> gradient =
            al_r29_pair_once_vjp(workspace, positive);
        ++pair_passes;
        const double gradient_norm = al_r32_vec3_norm(gradient);
        if (!al_r32_vec3_finite(gradient) || !std::isfinite(gradient_norm)) {
            cg_exact = false;
            break;
        }
        if (gradient_norm == 0.0) {
            projected_stationary = true;
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
                cg_exact = false;
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
                cg_exact = false;
                break;
            }
            const long double curvature =
                al_r35_dot_vec3(direction, hessian_direction);
            if (!std::isfinite(static_cast<double>(curvature))) {
                cg_exact = false;
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
                step[particle] += static_cast<double>(alpha) * direction[particle];
                cg_residual[particle] += -static_cast<double>(alpha)
                    * hessian_direction[particle];
            }
            const long double rr_next =
                al_r35_dot_vec3(cg_residual, cg_residual);
            ++cg_iterations;
            if (!std::isfinite(static_cast<double>(rr_next)) || rr_next < 0.0L) {
                cg_exact = false;
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
        if (!cg_exact) break;
        const double cg_final_norm = std::sqrt(static_cast<double>(rr));
        const double descent = static_cast<double>(
            al_r35_dot_vec3(gradient, step));
        cg_exact = al_r32_vec3_finite(step) && std::isfinite(cg_final_norm)
            && std::isfinite(descent) && descent < 0.0
            && cg_iterations <= 5U;
        if (!cg_exact) break;

        std::vector<Vec3> trial(iterate.size());
        for (std::size_t particle = 0U; particle < iterate.size(); ++particle) {
            trial[particle] = iterate[particle] + step[particle];
        }
        const std::vector<Vec3> target = al_r33_project_vec3(
            trial, AL_R32_TRUST_DELTA);
        if (target.size() != iterate.size()) {
            globalization_exact = false;
            break;
        }
        std::vector<Vec3> line_direction(iterate.size());
        for (std::size_t particle = 0U; particle < iterate.size(); ++particle) {
            line_direction[particle] = target[particle] - iterate[particle];
        }
        const double chord_norm = al_r32_vec3_norm(line_direction);
        if (chord_norm == 0.0) {
            projected_stationary = true;
            break;
        }
        for (Vec3& value : line_direction) {
            value = (1.0 / chord_norm) * value;
        }
        const ALLinearizedJvp line_response =
            al_r29_pair_once_jvp(workspace, line_direction);
        ++pair_passes;
        if (!line_response.passed) {
            globalization_exact = false;
            break;
        }
        const ALR32LineResult line = al_r32_piecewise_line(
            residual, line_response.value, chord_norm);
        const double derivative_scale = std::max(
            std::abs(line.initial.derivative), 1.0e-300);
        const bool line_kkt = line.trust_active
            ? line.final.derivative <= 0.0
            : std::abs(line.final.derivative) / derivative_scale <= 1.0e-10;
        globalization_exact = line.passed && line_kkt
            && line.final.phi < line.initial.phi;
        if (!globalization_exact) break;
        for (std::size_t particle = 0U; particle < iterate.size(); ++particle) {
            iterate[particle] += line.alpha * line_direction[particle];
        }
        for (std::size_t row = 0U; row < residual.size(); ++row) {
            maintained_response[row] += line.alpha * line_response.value[row];
            residual[row] = r30.constraint[row] + maintained_response[row];
        }
        globalization_exact = al_r32_vec3_norm(iterate)
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
        outer_records.push_back(std::move(record));
    }
    const bool outer_exact = cg_exact && globalization_exact
        && (projected_stationary || outer_records.size() == 4U);

    ALLinearizedJvp terminal_response;
    if (workspace_exact && outer_exact) {
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
    const double baseline_mapping_norm =
        al_r32_vec3_norm(capture.projected_mapping);
    const bool dominates = direct_exact
        && terminal.phi < capture.terminal.phi
        && terminal.positive_norm < capture.terminal.positive_norm
        && projected_mapping_norm < baseline_mapping_norm;

    release_al_normalized_workspace(workspace, &work);
    const bool work_exact = pair_passes <= 51U && hvp_count <= 20U
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
        {"EQUAL_WORK_GENERALIZED_HESSIAN_CANDIDATE", al_r35_route(1,1,1,1,1,1,1,1,1,0,1)},
        {"FIRST_ORDER_REFERENCE_RETAINED", al_r35_route(1,1,1,1,1,1,1,1,1,0,0)},
        {"ALL_INEQUALITY_PROJECTED_STATIONARY", al_r35_route(1,1,1,1,1,1,1,1,1,1,0)},
        {"CURVATURE_PARENT_REJECTED", al_r35_route(0,1,1,1,1,1,1,1,1,0,0)},
        {"CURVATURE_SOURCE_REJECTED", al_r35_route(1,0,1,1,1,1,1,1,1,0,0)},
        {"CURVATURE_DENSE_REJECTED", al_r35_route(1,1,0,1,1,1,1,1,1,0,0)},
        {"CURVATURE_WORKSPACE_REJECTED", al_r35_route(1,1,1,0,1,1,1,1,1,0,0)},
        {"CURVATURE_PREFIX_REJECTED", al_r35_route(1,1,1,1,0,1,1,1,1,0,0)},
        {"CURVATURE_CG_REJECTED", al_r35_route(1,1,1,1,1,0,1,1,1,0,0)},
        {"CURVATURE_GLOBALIZATION_REJECTED", al_r35_route(1,1,1,1,1,1,0,1,1,0,0)},
        {"CURVATURE_DIRECT_REJECTED", al_r35_route(1,1,1,1,1,1,1,0,1,0,0)},
        {"CURVATURE_WORK_REJECTED", al_r35_route(1,1,1,1,1,1,1,1,0,0,0)},
    }};
    bool routes_exact = true;
    std::ostringstream route_projection;
    for (const auto& value : routes) {
        routes_exact = routes_exact && value.first == value.second;
        route_projection << value.first << ':' << value.second << '|';
    }
    const std::string route_root = sha256_hex(route_projection.str());
    const std::string route = al_r35_route(
        parent_exact, source_exact, dense_exact, workspace_exact, prefix_exact,
        cg_exact, globalization_exact, direct_exact && rollback_exact,
        work_exact, projected_stationary, dominates);
    const bool passed = identity_exact && parent_exact && source_exact
        && dense_exact && workspace_exact && prefix_exact && outer_exact
        && direct_exact && work_exact && rollback_exact && routes_exact;
    std::string first_failure;
    if (!identity_exact) first_failure = "IDENTITY";
    else if (!parent_exact) first_failure = "PARENT";
    else if (!source_exact) first_failure = "SOURCE";
    else if (!dense_exact) first_failure = "DENSE";
    else if (!workspace_exact) first_failure = "WORKSPACE";
    else if (!prefix_exact) first_failure = "PREFIX";
    else if (!cg_exact) first_failure = "CG";
    else if (!globalization_exact) first_failure = "GLOBALIZATION";
    else if (!outer_exact) first_failure = "OUTER";
    else if (!direct_exact) first_failure = "DIRECT";
    else if (!work_exact) first_failure = "WORK";
    else if (!rollback_exact) first_failure = "ROLLBACK";
    else if (!routes_exact) first_failure = "ROUTES";

    std::ostringstream outer_projection;
    for (const ALR35OuterRecord& record : outer_records) {
        outer_projection << record.root << '|';
    }
    const std::string outer_root = sha256_hex(outer_projection.str());
    const std::string terminal_response_root = terminal_response.passed
        ? al_r10_dual_root(terminal_response.value) : sha256_hex("missing");
    const std::string terminal_gradient_root =
        al_binary64_vec3_root(terminal_gradient);
    const std::string projected_mapping_root =
        al_binary64_vec3_root(projected_mapping);
    const double phi_ratio = terminal.phi / capture.terminal.phi;
    const double violation_ratio =
        terminal.positive_norm / capture.terminal.positive_norm;
    const double mapping_ratio = projected_mapping_norm / baseline_mapping_norm;
    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R35_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << dense_root << ':' << outer_root << ':'
             << terminal_response_root << ':' << terminal_gradient_root << ':'
             << projected_mapping_root << '|' << binary64_bits(phi_ratio) << ':'
             << binary64_bits(violation_ratio) << ':'
             << binary64_bits(mapping_ratio) << ':' << pair_passes << ':'
             << hvp_count << '|' << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4e2d7r19r35_equal_work_generalized_hessian.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R35_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source\":{\"baseline_checkpoint_root\":\""
           << baseline_checkpoint_root << "\",\"prefix_response_root\":\""
           << (prefix_response.passed
               ? al_r10_dual_root(prefix_response.value) : sha256_hex("missing"))
           << "\",\"prefix_response_relative\":" << prefix_relative << "}"
           << ",\"dense\":{\"passed\":"
           << (dense_exact ? "true" : "false")
           << ",\"root\":\"" << dense_root << "\"}"
           << ",\"curvature\":{\"outer_count\":" << outer_records.size()
           << ",\"hvp_count\":" << hvp_count
           << ",\"curvature_stops\":" << curvature_stops
           << ",\"records\":[";
    for (std::size_t index_value = 0U;
         index_value < outer_records.size(); ++index_value) {
        if (index_value != 0U) report << ',';
        const ALR35OuterRecord& record = outer_records[index_value];
        report << "{\"outer\":" << record.outer
               << ",\"cg_iterations\":" << record.cg_iterations
               << ",\"curvature_stops\":" << record.curvature_stops
               << ",\"cg_initial_norm\":" << record.cg_initial_norm
               << ",\"cg_final_norm\":" << record.cg_final_norm
               << ",\"descent\":" << record.descent
               << ",\"line_alpha\":" << record.line_alpha
               << ",\"phi\":" << record.phi
               << ",\"violation_norm\":" << record.violation_norm
               << ",\"active\":" << record.active
               << ",\"root\":\"" << record.root << "\"}";
    }
    report << "],\"root\":\"" << outer_root << "\"}"
           << ",\"comparison\":{\"baseline_phi\":" << capture.terminal.phi
           << ",\"candidate_phi\":" << terminal.phi
           << ",\"phi_ratio\":" << phi_ratio
           << ",\"baseline_violation_norm\":"
           << capture.terminal.positive_norm
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
           << ",\"work\":{\"parent_r34_replays\":1"
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
              ",\"floor_classification\":false,\"state_mutation\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false"
           << ",\"result_sha256\":\"" << result_sha << "\"}";
    if (output_capture != nullptr) {
        output_capture->captured = passed;
        output_capture->r34 = std::move(capture);
        output_capture->outer_records = std::move(outer_records);
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_equal_work_generalized_hessian_controls() {
    return run_al_equal_work_generalized_hessian_controls_impl(nullptr);
}
