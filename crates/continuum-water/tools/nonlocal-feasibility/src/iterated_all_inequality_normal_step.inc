namespace {

constexpr const char* B4E2D7R19R33_IDENTITY_SHA256 =
    "7e4a70ffc56b82d73f8244a1f7096135419638842b27fd17004e7bea8e3f6aab";
constexpr const char* B4E2D7R19R33_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r33-iterated-all-inequality-normal-step|v1|parent=f535f109:243a115d66a33667d57f2fec73ca58553c57f00d83a235dcb8640d04f1d701db:2417db008424792e6b000beebf50999c2e4b71493d0cfb0d7df69ae2e14ba695|source=state-851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7;constraint-c567599060d7d00ffec4928009772327d081d3ef294d9030bfc5499b82e533c9;topology-070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5;particles6000;pairs340340;max-degree113|model=phi(v)=0.5*norm(max(c+A*v,0))2;A=SPACING*Jc;all-rows;support-fixed|trust=global-l2;delta0.25;fixed|iteration=projected-normalized-steepest;z=project-ball(v-g/norm(g),delta);d=(z-v)/norm(z-v);alpha-in-[0,norm(z-v)];exact-r32-piecewise-line;update-v-response;max8;checkpoints1,2,4,8|terminal=fresh-jvp-v;fresh-vjp-positive;maintained-direct-response-relative<=1e-12;projected-mapping;strict-monotone;first-step-r32-exact|controls=dense-conditioned;dense-active-switch;dense-boundary-stationary;dense-feasible-stationary;parent;source;workspace;first-step;iteration;direct;work;rollback|observations=checkpoint-phi-violation-active-step;accepted-iterations;terminal-response-gradient-mapping-roots;norm-ratios;trust-use|routes=iterated-parent-rejected;iterated-source-rejected;iterated-dense-rejected;iterated-workspace-rejected;iterated-first-step-rejected;iterated-iteration-rejected;iterated-direct-rejected;iterated-work-rejected;all-inequality-projected-stationary;iterated-cauchy-no-additional-progress;iterated-all-inequality-normal-step-candidate|precedence=parent,source,dense,workspace,first-step,iteration,direct,work,classification|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-r32-once;diagnostic-workspaces1;new-pair-passes<=18;new-hvp0;new-model0;new-trial0;new-outer0|correction=none;nonlinear-evaluation=none;floor-classification=none;state-mutation=none;following-outer=none;substep=none;macro=none;trajectory=none;timing=none;public-schema=none;runtime=none;production=none|credit=one-private-iterated-all-inequality-normal-step-classification-only)IDENTITY";

constexpr std::size_t AL_R33_MAX_ITERATIONS = 8U;

struct ALR33Checkpoint {
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

struct ALR33DenseControl {
    std::string name;
    bool passed = false;
    bool stationary = false;
    std::size_t accepted = 0U;
    double phi = 0.0;
    double iterate_norm = 0.0;
    std::size_t active = 0U;
    std::string root;
};

double al_r33_vec_norm(const std::vector<double>& value) {
    long double squared = 0.0L;
    for (double item : value) squared += static_cast<long double>(item) * item;
    return std::sqrt(static_cast<double>(squared));
}

double al_r33_vec_diff_relative(
    const std::vector<double>& lhs,
    const std::vector<double>& rhs) {
    if (lhs.size() != rhs.size() || lhs.empty()) {
        return std::numeric_limits<double>::infinity();
    }
    std::vector<double> difference(lhs.size(), 0.0);
    for (std::size_t index = 0U; index < lhs.size(); ++index) {
        difference[index] = lhs[index] - rhs[index];
    }
    return al_r33_vec_norm(difference) / std::max(
        {al_r33_vec_norm(lhs), al_r33_vec_norm(rhs), 1.0e-300});
}

std::vector<double> al_r33_dense_jvp(
    const std::vector<std::vector<double>>& matrix,
    const std::vector<double>& direction) {
    std::vector<double> result(matrix.size(), 0.0);
    for (std::size_t row = 0U; row < matrix.size(); ++row) {
        long double value = 0.0L;
        for (std::size_t column = 0U; column < direction.size(); ++column) {
            value += static_cast<long double>(matrix[row][column])
                * direction[column];
        }
        result[row] = static_cast<double>(value);
    }
    return result;
}

std::vector<double> al_r33_dense_vjp(
    const std::vector<std::vector<double>>& matrix,
    const std::vector<double>& weight) {
    const std::size_t columns = matrix.empty() ? 0U : matrix.front().size();
    std::vector<double> result(columns, 0.0);
    for (std::size_t column = 0U; column < columns; ++column) {
        long double value = 0.0L;
        for (std::size_t row = 0U; row < matrix.size(); ++row) {
            value += static_cast<long double>(matrix[row][column])
                * weight[row];
        }
        result[column] = static_cast<double>(value);
    }
    return result;
}

std::vector<double> al_r33_dense_project(
    std::vector<double> value,
    double delta) {
    const double value_norm = al_r33_vec_norm(value);
    if (value_norm <= delta) return value;
    double scale = delta / value_norm;
    std::vector<double> projected(value.size(), 0.0);
    for (std::size_t repair = 0U; repair <= 64U; ++repair) {
        for (std::size_t index = 0U; index < value.size(); ++index) {
            projected[index] = scale * value[index];
        }
        if (al_r33_vec_norm(projected) <= delta) return projected;
        scale = std::nextafter(scale, 0.0);
    }
    return {};
}

ALR33DenseControl al_r33_dense_run(
    std::string name,
    const std::vector<std::vector<double>>& matrix,
    const std::vector<double>& constraint,
    double delta,
    std::size_t maximum_iterations = AL_R33_MAX_ITERATIONS) {
    ALR33DenseControl result;
    result.name = std::move(name);
    if (matrix.empty() || matrix.size() != constraint.size()
        || matrix.front().empty()) return result;
    const std::size_t columns = matrix.front().size();
    std::vector<double> iterate(columns, 0.0);
    std::vector<double> residual = constraint;
    const std::vector<double> zero(constraint.size(), 0.0);
    bool exact = true;
    for (std::size_t iteration = 0U;
         iteration < maximum_iterations; ++iteration) {
        std::vector<double> positive(residual.size(), 0.0);
        for (std::size_t row = 0U; row < residual.size(); ++row) {
            positive[row] = std::max(residual[row], 0.0);
        }
        const std::vector<double> gradient =
            al_r33_dense_vjp(matrix, positive);
        const double gradient_norm = al_r33_vec_norm(gradient);
        if (gradient_norm <= 1.0e-14) {
            result.stationary = true;
            break;
        }
        std::vector<double> target(columns, 0.0);
        for (std::size_t column = 0U; column < columns; ++column) {
            target[column] = iterate[column]
                - gradient[column] / gradient_norm;
        }
        target = al_r33_dense_project(std::move(target), delta);
        if (target.size() != columns) {
            exact = false;
            break;
        }
        std::vector<double> chord(columns, 0.0);
        for (std::size_t column = 0U; column < columns; ++column) {
            chord[column] = target[column] - iterate[column];
        }
        const double chord_norm = al_r33_vec_norm(chord);
        if (chord_norm <= 1.0e-14) {
            result.stationary = true;
            break;
        }
        std::vector<double> direction(columns, 0.0);
        for (std::size_t column = 0U; column < columns; ++column) {
            direction[column] = chord[column] / chord_norm;
        }
        const std::vector<double> response =
            al_r33_dense_jvp(matrix, direction);
        const ALR32LineResult line = al_r32_piecewise_line(
            residual, response, chord_norm);
        const double derivative_scale = std::max(
            std::abs(line.initial.derivative), 1.0e-300);
        const bool kkt = line.trust_active
            ? line.final.derivative <= 0.0
            : std::abs(line.final.derivative) / derivative_scale <= 1.0e-10;
        exact = exact && line.passed && line.final.phi < line.initial.phi
            && kkt;
        if (!exact) break;
        for (std::size_t column = 0U; column < columns; ++column) {
            iterate[column] += line.alpha * direction[column];
        }
        for (std::size_t row = 0U; row < residual.size(); ++row) {
            residual[row] += line.alpha * response[row];
        }
        ++result.accepted;
        exact = exact && al_r33_vec_norm(iterate)
            <= std::nextafter(delta, std::numeric_limits<double>::infinity());
    }
    const ALR32Evaluation final = al_r32_evaluate(residual, zero, 0.0);
    result.phi = final.phi;
    result.active = final.active;
    result.iterate_norm = al_r33_vec_norm(iterate);
    result.passed = exact && final.finite;
    std::ostringstream projection;
    projection << result.name << ':' << result.passed << ':'
               << result.stationary << ':' << result.accepted << ':'
               << binary64_bits(result.phi) << ':'
               << binary64_bits(result.iterate_norm) << ':' << result.active;
    result.root = sha256_hex(projection.str());
    return result;
}

std::array<ALR33DenseControl, 4U> al_r33_dense_controls() {
    std::array<ALR33DenseControl, 4U> result{{
        al_r33_dense_run("conditioned",
            {{1.0, 0.0}, {0.0, 2.0}}, {1.0, 1.0}, 2.0),
        al_r33_dense_run("active-switch",
            {{-1.0}, {1.0}}, {1.0, -0.1}, 1.0),
        al_r33_dense_run("boundary-stationary",
            {{-0.25}}, {1.0}, 1.0),
        al_r33_dense_run("feasible-stationary",
            {{1.0}}, {-1.0}, 1.0),
    }};
    result[0].passed = result[0].passed && result[0].stationary
        && result[0].accepted == 2U && result[0].phi <= 1.0e-28;
    result[1].passed = result[1].passed && result[1].stationary
        && result[1].accepted == 1U && result[1].active == 2U;
    result[2].passed = result[2].passed && result[2].stationary
        && result[2].accepted == 1U
        && binary64_bits(result[2].iterate_norm) == binary64_bits(1.0)
        && binary64_bits(result[2].phi) == binary64_bits(0.28125);
    result[3].passed = result[3].passed && result[3].stationary
        && result[3].accepted == 0U && result[3].phi == 0.0;
    return result;
}

std::vector<Vec3> al_r33_project_vec3(
    const std::vector<Vec3>& value,
    double delta) {
    const double value_norm = al_r32_vec3_norm(value);
    if (value_norm <= delta) return value;
    double scale = delta / value_norm;
    std::vector<Vec3> projected(value.size());
    for (std::size_t repair = 0U; repair <= 64U; ++repair) {
        for (std::size_t index = 0U; index < value.size(); ++index) {
            projected[index] = scale * value[index];
        }
        if (al_r32_vec3_norm(projected) <= delta) return projected;
        scale = std::nextafter(scale, 0.0);
    }
    return {};
}

std::string al_r33_route(
    bool parent, bool source, bool dense, bool workspace, bool first_step,
    bool iteration, bool direct, bool work, bool stationary,
    bool additional_progress) {
    if (!parent) return "ITERATED_PARENT_REJECTED";
    if (!source) return "ITERATED_SOURCE_REJECTED";
    if (!dense) return "ITERATED_DENSE_REJECTED";
    if (!workspace) return "ITERATED_WORKSPACE_REJECTED";
    if (!first_step) return "ITERATED_FIRST_STEP_REJECTED";
    if (!iteration) return "ITERATED_ITERATION_REJECTED";
    if (!direct) return "ITERATED_DIRECT_REJECTED";
    if (!work) return "ITERATED_WORK_REJECTED";
    if (stationary) return "ALL_INEQUALITY_PROJECTED_STATIONARY";
    return additional_progress
        ? "ITERATED_ALL_INEQUALITY_NORMAL_STEP_CANDIDATE"
        : "ITERATED_CAUCHY_NO_ADDITIONAL_PROGRESS";
}

struct ALR33NormalStepCapture {
    bool captured = false;
    ALR32CauchyCapture r32;
    std::vector<Vec3> iterate;
    std::vector<double> maintained_response;
    std::vector<double> terminal_response;
    std::vector<Vec3> terminal_gradient;
    std::vector<ALR33Checkpoint> checkpoints;
    ALR32Evaluation terminal;
    bool projected_stationary = false;
};

} // namespace

SplitBoundaryReport run_al_iterated_all_inequality_normal_step_controls_impl(
    ALR33NormalStepCapture* output_capture) {
    const bool identity_exact = sha256_hex(B4E2D7R19R33_IDENTITY_PROJECTION)
        == B4E2D7R19R33_IDENTITY_SHA256;
    ALR32CauchyCapture capture;
    const SplitBoundaryReport parent =
        run_al_all_inequality_cauchy_step_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "2417db008424792e6b000beebf50999c2e4b71493d0cfb0d7df69ae2e14ba695"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && parent_semantic && parent_stdout
            == "243a115d66a33667d57f2fec73ca58553c57f00d83a235dcb8640d04f1d701db";
    ALR30RangeProjectionCapture& r30 = capture.r31.r30;
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
        && r30.position.size() == 6000U
        && r30.constraint.size() == r30.position.size();

    const std::array<ALR33DenseControl, 4U> dense_controls =
        al_r33_dense_controls();
    const bool dense_exact = std::all_of(
        dense_controls.begin(), dense_controls.end(),
        [](const ALR33DenseControl& value) { return value.passed; });
    std::ostringstream dense_projection;
    for (const ALR33DenseControl& value : dense_controls) {
        dense_projection << value.name << ':' << value.passed << ':'
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

    std::vector<Vec3> iterate(r30.position.size());
    std::vector<double> maintained_response(r30.constraint.size(), 0.0);
    std::vector<double> residual = r30.constraint;
    std::vector<ALR33Checkpoint> checkpoints;
    std::size_t pair_passes = 0U;
    std::size_t accepted = 0U;
    bool iteration_exact = workspace_exact;
    bool projected_stationary = false;
    bool first_step_exact = false;
    std::string first_gradient_root;
    std::string first_direction_root;
    std::string first_response_root;
    std::string first_line_root;
    double first_phi = capture.line.final.phi;

    for (std::size_t iteration = 0U;
         iteration < AL_R33_MAX_ITERATIONS && iteration_exact; ++iteration) {
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

        std::vector<Vec3> direction(gradient.size());
        double chord_norm = 0.0;
        if (iteration == 0U) {
            for (std::size_t particle = 0U;
                 particle < gradient.size(); ++particle) {
                direction[particle] =
                    -(1.0 / gradient_norm) * gradient[particle];
            }
            chord_norm = AL_R32_TRUST_DELTA;
        } else {
            std::vector<Vec3> trial(iterate.size());
            for (std::size_t particle = 0U;
                 particle < iterate.size(); ++particle) {
                trial[particle] = iterate[particle]
                    - (1.0 / gradient_norm) * gradient[particle];
            }
            const std::vector<Vec3> target = al_r33_project_vec3(
                trial, AL_R32_TRUST_DELTA);
            if (target.size() != iterate.size()) {
                iteration_exact = false;
                break;
            }
            for (std::size_t particle = 0U;
                 particle < iterate.size(); ++particle) {
                direction[particle] = target[particle] - iterate[particle];
            }
            chord_norm = al_r32_vec3_norm(direction);
            if (chord_norm == 0.0) {
                projected_stationary = true;
                break;
            }
            for (Vec3& value : direction) value = (1.0 / chord_norm) * value;
        }
        const double direction_norm = al_r32_vec3_norm(direction);
        iteration_exact = iteration_exact && al_r32_vec3_finite(direction)
            && std::isfinite(chord_norm) && chord_norm > 0.0
            && std::abs(direction_norm - 1.0) <= 1.0e-12;
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
        for (std::size_t particle = 0U;
             particle < gradient.size(); ++particle) {
            adjoint_right += static_cast<long double>(dot(
                gradient[particle], direction[particle]));
        }
        const double adjoint_relative = static_cast<double>(
            std::abs(adjoint_left - adjoint_right)
            / std::max(std::abs(adjoint_right), 1.0e-300L));
        const double derivative_scale = std::max(
            std::abs(line.initial.derivative), 1.0e-300);
        const bool line_kkt = line.trust_active
            ? line.final.derivative <= 0.0
            : std::abs(line.final.derivative) / derivative_scale <= 1.0e-10;
        iteration_exact = iteration_exact && line.passed
            && line.final.phi < line.initial.phi && line_kkt
            && std::isfinite(adjoint_relative) && adjoint_relative <= 1.0e-12;
        if (!iteration_exact) break;

        if (iteration == 0U) {
            first_gradient_root = al_binary64_vec3_root(gradient);
            first_direction_root = al_binary64_vec3_root(direction);
            first_response_root = al_r10_dual_root(response.value);
            first_line_root = line.root;
            first_step_exact = first_gradient_root
                    == al_binary64_vec3_root(capture.gradient)
                && first_direction_root
                    == al_binary64_vec3_root(capture.direction)
                && first_response_root == al_r10_dual_root(capture.response)
                && first_line_root == capture.line.root
                && binary64_bits(line.alpha)
                    == binary64_bits(capture.line.alpha)
                && binary64_bits(line.final.phi)
                    == binary64_bits(capture.line.final.phi);
            iteration_exact = iteration_exact && first_step_exact;
            if (!iteration_exact) break;
        }

        for (std::size_t particle = 0U;
             particle < iterate.size(); ++particle) {
            iterate[particle] += line.alpha * direction[particle];
        }
        for (std::size_t row = 0U; row < residual.size(); ++row) {
            maintained_response[row] += line.alpha * response.value[row];
            residual[row] = r30.constraint[row] + maintained_response[row];
        }
        ++accepted;
        const double iterate_norm = al_r32_vec3_norm(iterate);
        iteration_exact = iteration_exact
            && iterate_norm <= std::nextafter(
                AL_R32_TRUST_DELTA,
                std::numeric_limits<double>::infinity());
        const std::size_t checkpoint_iteration = iteration + 1U;
        if (checkpoint_iteration == 1U || checkpoint_iteration == 2U
            || checkpoint_iteration == 4U || checkpoint_iteration == 8U) {
            ALR33Checkpoint checkpoint;
            checkpoint.iteration = checkpoint_iteration;
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
    mapping_target = al_r33_project_vec3(
        mapping_target, AL_R32_TRUST_DELTA);
    std::vector<Vec3> projected_mapping(iterate.size());
    if (mapping_target.size() == iterate.size()) {
        for (std::size_t particle = 0U; particle < iterate.size(); ++particle) {
            projected_mapping[particle] = iterate[particle]
                - mapping_target[particle];
        }
    }
    const double projected_mapping_norm =
        al_r32_vec3_norm(projected_mapping);
    const bool direct_exact = terminal_response.passed
        && terminal_gradient.size() == iterate.size()
        && al_r32_vec3_finite(terminal_gradient)
        && terminal.finite && std::isfinite(response_relative)
        && response_relative <= 1.0e-12
        && std::isfinite(terminal_gradient_norm)
        && std::isfinite(projected_mapping_norm)
        && mapping_target.size() == iterate.size();
    projected_stationary = projected_stationary
        || (direct_exact && projected_mapping_norm == 0.0);
    const bool additional_progress = accepted >= 2U
        && terminal.phi < first_phi;

    release_al_normalized_workspace(workspace, &work);
    const bool work_exact = pair_passes <= 18U
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
        {"ITERATED_ALL_INEQUALITY_NORMAL_STEP_CANDIDATE", al_r33_route(1,1,1,1,1,1,1,1,0,1)},
        {"ITERATED_CAUCHY_NO_ADDITIONAL_PROGRESS", al_r33_route(1,1,1,1,1,1,1,1,0,0)},
        {"ALL_INEQUALITY_PROJECTED_STATIONARY", al_r33_route(1,1,1,1,1,1,1,1,1,0)},
        {"ITERATED_PARENT_REJECTED", al_r33_route(0,1,1,1,1,1,1,1,0,1)},
        {"ITERATED_SOURCE_REJECTED", al_r33_route(1,0,1,1,1,1,1,1,0,1)},
        {"ITERATED_DENSE_REJECTED", al_r33_route(1,1,0,1,1,1,1,1,0,1)},
        {"ITERATED_WORKSPACE_REJECTED", al_r33_route(1,1,1,0,1,1,1,1,0,1)},
        {"ITERATED_FIRST_STEP_REJECTED", al_r33_route(1,1,1,1,0,1,1,1,0,1)},
        {"ITERATED_ITERATION_REJECTED", al_r33_route(1,1,1,1,1,0,1,1,0,1)},
        {"ITERATED_DIRECT_REJECTED", al_r33_route(1,1,1,1,1,1,0,1,0,1)},
        {"ITERATED_WORK_REJECTED", al_r33_route(1,1,1,1,1,1,1,0,0,1)},
    }};
    bool routes_exact = true;
    std::ostringstream route_projection;
    for (const auto& value : routes) {
        routes_exact = routes_exact && value.first == value.second;
        route_projection << value.first << ':' << value.second << '|';
    }
    const std::string route_root = sha256_hex(route_projection.str());
    const std::string route = al_r33_route(
        parent_exact, source_exact, dense_exact, workspace_exact,
        first_step_exact, iteration_exact, direct_exact && rollback_exact,
        work_exact, projected_stationary, additional_progress);
    const bool passed = identity_exact && parent_exact && source_exact
        && dense_exact && workspace_exact && first_step_exact
        && iteration_exact && direct_exact && work_exact && rollback_exact
        && routes_exact;
    std::string first_failure;
    if (!identity_exact) first_failure = "IDENTITY";
    else if (!parent_exact) first_failure = "PARENT";
    else if (!source_exact) first_failure = "SOURCE";
    else if (!dense_exact) first_failure = "DENSE";
    else if (!workspace_exact) first_failure = "WORKSPACE";
    else if (!first_step_exact) first_failure = "FIRST_STEP";
    else if (!iteration_exact) first_failure = "ITERATION";
    else if (!direct_exact) first_failure = "DIRECT";
    else if (!work_exact) first_failure = "WORK";
    else if (!rollback_exact) first_failure = "ROLLBACK";
    else if (!routes_exact) first_failure = "ROUTES";

    std::ostringstream checkpoint_projection;
    for (const ALR33Checkpoint& checkpoint : checkpoints) {
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
             << B4E2D7R19R33_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << dense_root << ':' << checkpoint_root << ':'
             << terminal_response_root << ':' << terminal_gradient_root << ':'
             << projected_mapping_root << '|' << accepted << ':'
             << binary64_bits(terminal.phi) << ':'
             << binary64_bits(terminal.positive_norm) << ':'
             << binary64_bits(response_relative) << ':' << pair_passes << '|'
             << route_root << ':' << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4e2d7r19r33_iterated_all_inequality_normal_step.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R33_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"source\":{\"constraint_root\":\"" << constraint_before
           << "\",\"topology_root\":\"" << r30.topology_root
           << "\",\"particles\":" << r30.position.size()
           << ",\"pairs\":340340,\"maximum_degree\":113}"
           << ",\"dense\":{\"passed\":"
           << (dense_exact ? "true" : "false")
           << ",\"root\":\"" << dense_root << "\"}"
           << ",\"iteration\":{\"accepted\":" << accepted
           << ",\"stationary\":"
           << (projected_stationary ? "true" : "false")
           << ",\"additional_progress\":"
           << (additional_progress ? "true" : "false")
           << ",\"first_gradient_root\":\"" << first_gradient_root
           << "\",\"first_direction_root\":\"" << first_direction_root
           << "\",\"first_response_root\":\"" << first_response_root
           << "\",\"first_line_root\":\"" << first_line_root
           << "\",\"checkpoints\":[";
    for (std::size_t index_value = 0U;
         index_value < checkpoints.size(); ++index_value) {
        if (index_value != 0U) report << ',';
        const ALR33Checkpoint& checkpoint = checkpoints[index_value];
        report << "{\"iteration\":" << checkpoint.iteration
               << ",\"phi\":" << checkpoint.phi
               << ",\"violation_norm\":" << checkpoint.violation_norm
               << ",\"step_norm\":" << checkpoint.step_norm
               << ",\"iterate_norm\":" << checkpoint.iterate_norm
               << ",\"gradient_norm\":" << checkpoint.gradient_norm
               << ",\"active\":" << checkpoint.active
               << ",\"root\":\"" << checkpoint.root << "\"}";
    }
    report << "],\"checkpoint_root\":\"" << checkpoint_root << "\"}"
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
           << ",\"work\":{\"parent_r32_replays\":1"
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
    if (output_capture != nullptr) {
        output_capture->captured = passed;
        output_capture->r32 = std::move(capture);
        output_capture->iterate = std::move(iterate);
        output_capture->maintained_response = std::move(maintained_response);
        output_capture->terminal_response = std::move(terminal_response.value);
        output_capture->terminal_gradient = std::move(terminal_gradient);
        output_capture->checkpoints = std::move(checkpoints);
        output_capture->terminal = terminal;
        output_capture->projected_stationary = projected_stationary;
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_iterated_all_inequality_normal_step_controls() {
    return run_al_iterated_all_inequality_normal_step_controls_impl(nullptr);
}
