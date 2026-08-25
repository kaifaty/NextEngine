namespace {

constexpr const char* AL_R20_ORACLE_SHA256 =
    "885dc7bdb628e4d435b3cd6966c87b6866dd9ca33fec3d8b9718b8477328c60f";
constexpr std::array<std::size_t, 7U> AL_R20_ADMM_CHECKPOINTS{{
    1U << 4U,
    1U << 6U,
    1U << 8U,
    1U << 10U,
    1U << 12U,
    1U << 14U,
    1U << 16U,
}};

struct ALR20ADMMWork {
    std::size_t matrix_outer_terms = 0U;
    std::size_t factor_terms = 0U;
    std::size_t solve_terms = 0U;
    std::size_t normalized_action_terms = 0U;
    std::size_t normalized_transpose_terms = 0U;
    std::size_t domain_projections = 0U;
};

bool al_r20_admm_finite(const std::vector<Binary128>& value) {
    for (Binary128 scalar : value)
        if (finiteq(scalar) == 0) return false;
    return true;
}

bool al_r20_admm_finite(const std::vector<Binary128Vec3>& value) {
    for (Binary128Vec3 vector : value)
        if (finiteq(vector.x) == 0 || finiteq(vector.y) == 0
            || finiteq(vector.z) == 0) return false;
    return true;
}

std::vector<Binary128> al_r20_admm_flatten(
    const std::vector<Binary128Vec3>& value) {
    std::vector<Binary128> result(3U * value.size());
    for (std::size_t particle = 0U; particle < value.size(); ++particle) {
        result[3U * particle] = value[particle].x;
        result[3U * particle + 1U] = value[particle].y;
        result[3U * particle + 2U] = value[particle].z;
    }
    return result;
}

std::vector<Binary128Vec3> al_r20_admm_unflatten(
    const std::vector<Binary128>& value) {
    if (value.size() % 3U != 0U) return {};
    std::vector<Binary128Vec3> result(value.size() / 3U);
    for (std::size_t particle = 0U; particle < result.size(); ++particle)
        result[particle] = {value[3U * particle],
            value[3U * particle + 1U], value[3U * particle + 2U]};
    return result;
}

std::vector<Binary128> al_r20_admm_action(
    const ALR20ProblemData& problem,
    const std::vector<Binary128Vec3>& value,
    ALR20ADMMWork* work) {
    const std::size_t rows = problem.row_scale.size();
    std::vector<Binary128> result(rows);
    for (std::size_t row = 0U; row < rows; ++row) {
        Binary128Accumulator sum;
        sum.add(static_cast<Binary128>(
            problem.workspace.support.constraint[row]));
        for (std::size_t index = problem.op.entry_offsets[row];
             index < problem.op.entry_offsets[row + 1U]; ++index) {
            const ALR64Entry& entry = problem.op.entries[index];
            sum.add(al_r20_q_dot(
                al_binary128_vec3(entry.coefficient), value[entry.particle]));
            if (work != nullptr) ++work->normalized_action_terms;
        }
        result[row] = sum.value()
            / static_cast<Binary128>(problem.row_scale[row]);
    }
    return result;
}

std::vector<Binary128Vec3> al_r20_admm_transpose(
    const ALR20ProblemData& problem,
    const std::vector<Binary128>& value,
    ALR20ADMMWork* work) {
    std::vector<Binary128Vec3> result(problem.target.size());
    for (std::size_t row = 0U; row < value.size(); ++row) {
        const Binary128 weight = value[row]
            / static_cast<Binary128>(problem.row_scale[row]);
        for (std::size_t index = problem.op.entry_offsets[row];
             index < problem.op.entry_offsets[row + 1U]; ++index) {
            const ALR64Entry& entry = problem.op.entries[index];
            result[entry.particle] = al_r20_q_add(
                result[entry.particle], al_r20_q_scale(
                    weight, al_binary128_vec3(entry.coefficient)));
            if (work != nullptr) ++work->normalized_transpose_terms;
        }
    }
    return result;
}

struct ALR20ADMMFactor {
    bool passed = false;
    std::size_t dimension = 0U;
    Binary128 maximum_reconstruction = static_cast<Binary128>(0.0);
    Binary128 maximum_reconstruction_bound = static_cast<Binary128>(0.0);
    std::vector<Binary128> matrix;
    std::vector<Binary128> lower;
    std::string matrix_root;
    std::string factor_root;
};

ALR20ADMMFactor al_r20_admm_factor_matrix(
    std::vector<Binary128> matrix, std::size_t dimension,
    ALR20ADMMWork* work) {
    ALR20ADMMFactor result;
    result.dimension = dimension;
    result.matrix = std::move(matrix);
    if (dimension == 0U
        || result.matrix.size() != dimension * dimension
        || !al_r20_admm_finite(result.matrix)) return result;
    result.lower.assign(dimension * dimension, static_cast<Binary128>(0.0));
    bool exact = true;
    for (std::size_t row = 0U; row < dimension && exact; ++row)
        for (std::size_t column = 0U; column <= row; ++column) {
            Binary128Accumulator product;
            for (std::size_t inner = 0U; inner < column; ++inner) {
                product.add(result.lower[row * dimension + inner]
                    * result.lower[column * dimension + inner]);
                if (work != nullptr) ++work->factor_terms;
            }
            const Binary128 remainder =
                result.matrix[row * dimension + column] - product.value();
            if (row == column) {
                exact = finiteq(remainder) != 0
                    && remainder > static_cast<Binary128>(0.0);
                if (exact)
                    result.lower[row * dimension + column] = sqrtq(remainder);
            } else {
                const Binary128 pivot =
                    result.lower[column * dimension + column];
                exact = finiteq(pivot) != 0
                    && pivot > static_cast<Binary128>(0.0);
                if (exact)
                    result.lower[row * dimension + column] = remainder / pivot;
            }
        }
    for (std::size_t row = 0U; row < dimension && exact; ++row)
        for (std::size_t column = 0U; column <= row; ++column) {
            Binary128Accumulator reconstructed;
            Binary128 absolute = fabsq(
                result.matrix[row * dimension + column]);
            for (std::size_t inner = 0U; inner <= column; ++inner) {
                const Binary128 term = result.lower[row * dimension + inner]
                    * result.lower[column * dimension + inner];
                reconstructed.add(term);
                absolute += fabsq(term);
            }
            const Binary128 gap = fabsq(
                result.matrix[row * dimension + column]
                    - reconstructed.value());
            const Binary128 bound = al_r20_q_gamma(4U * column + 64U)
                * fmaxq(absolute, ldexpq(static_cast<Binary128>(1.0), -16382));
            result.maximum_reconstruction = fmaxq(
                result.maximum_reconstruction, gap);
            result.maximum_reconstruction_bound = fmaxq(
                result.maximum_reconstruction_bound, bound);
            exact = gap <= bound;
        }
    result.matrix_root = al_r20_q_scalar_root(result.matrix);
    result.factor_root = al_r20_q_scalar_root(result.lower);
    result.passed = exact && al_r20_admm_finite(result.lower);
    return result;
}

ALR20ADMMFactor al_r20_admm_build_factor(
    const ALR20ProblemData& problem, ALR20ADMMWork* work) {
    const std::size_t dimension = 3U * problem.target.size();
    std::vector<Binary128> matrix(
        dimension * dimension, static_cast<Binary128>(0.0));
    for (std::size_t index = 0U; index < dimension; ++index)
        matrix[index * dimension + index] = static_cast<Binary128>(2.0);
    for (std::size_t row = 0U; row < problem.row_scale.size(); ++row) {
        const Binary128 inverse_scale = static_cast<Binary128>(1.0)
            / static_cast<Binary128>(problem.row_scale[row]);
        const std::size_t begin = problem.op.entry_offsets[row];
        const std::size_t end = problem.op.entry_offsets[row + 1U];
        for (std::size_t left = begin; left < end; ++left)
            for (int left_axis = 0; left_axis < 3; ++left_axis) {
                const std::size_t left_scalar =
                    3U * problem.op.entries[left].particle
                    + static_cast<std::size_t>(left_axis);
                const Binary128 left_value = inverse_scale * al_r20_q_component(
                    al_binary128_vec3(problem.op.entries[left].coefficient),
                    left_axis);
                for (std::size_t right = begin; right <= left; ++right)
                    for (int right_axis = 0; right_axis < 3; ++right_axis) {
                        const std::size_t right_scalar =
                            3U * problem.op.entries[right].particle
                            + static_cast<std::size_t>(right_axis);
                        if (right_scalar > left_scalar) continue;
                        const Binary128 right_value = inverse_scale
                            * al_r20_q_component(al_binary128_vec3(
                                problem.op.entries[right].coefficient),
                                right_axis);
                        matrix[left_scalar * dimension + right_scalar]
                            += left_value * right_value;
                        if (work != nullptr) ++work->matrix_outer_terms;
                    }
            }
    }
    for (std::size_t row = 0U; row < dimension; ++row)
        for (std::size_t column = 0U; column < row; ++column)
            matrix[column * dimension + row] =
                matrix[row * dimension + column];
    return al_r20_admm_factor_matrix(
        std::move(matrix), dimension, work);
}

bool al_r20_admm_solve(const ALR20ADMMFactor& factor,
    const std::vector<Binary128>& rhs, std::vector<Binary128>* solution,
    Binary128* maximum_residual, Binary128* maximum_bound,
    ALR20ADMMWork* work) {
    const std::size_t dimension = factor.dimension;
    if (!factor.passed || rhs.size() != dimension || solution == nullptr
        || !al_r20_admm_finite(rhs)) return false;
    std::vector<Binary128> forward(dimension);
    solution->assign(dimension, static_cast<Binary128>(0.0));
    for (std::size_t row = 0U; row < dimension; ++row) {
        Binary128Accumulator product;
        for (std::size_t column = 0U; column < row; ++column) {
            product.add(factor.lower[row * dimension + column]
                * forward[column]);
            if (work != nullptr) ++work->solve_terms;
        }
        forward[row] = (rhs[row] - product.value())
            / factor.lower[row * dimension + row];
    }
    for (std::size_t reverse = dimension; reverse-- > 0U;) {
        Binary128Accumulator product;
        for (std::size_t row = reverse + 1U; row < dimension; ++row) {
            product.add(factor.lower[row * dimension + reverse]
                * (*solution)[row]);
            if (work != nullptr) ++work->solve_terms;
        }
        (*solution)[reverse] = (forward[reverse] - product.value())
            / factor.lower[reverse * dimension + reverse];
    }
    bool exact = al_r20_admm_finite(*solution);
    Binary128 residual_max = static_cast<Binary128>(0.0);
    Binary128 bound_max = static_cast<Binary128>(0.0);
    for (std::size_t row = 0U; row < dimension && exact; ++row) {
        Binary128Accumulator image;
        Binary128 absolute = fabsq(rhs[row]);
        for (std::size_t column = 0U; column < dimension; ++column) {
            const Binary128 term = factor.matrix[row * dimension + column]
                * (*solution)[column];
            image.add(term);
            absolute += fabsq(term);
        }
        const Binary128 residual = fabsq(image.value() - rhs[row]);
        const Binary128 bound = al_r20_q_gamma(4U * dimension + 128U)
            * fmaxq(absolute, ldexpq(static_cast<Binary128>(1.0), -16382));
        residual_max = fmaxq(residual_max, residual);
        bound_max = fmaxq(bound_max, bound);
        exact = residual <= bound;
    }
    if (maximum_residual != nullptr) *maximum_residual = residual_max;
    if (maximum_bound != nullptr) *maximum_bound = bound_max;
    return exact;
}

bool al_r20_admm_dense_controls() {
    ALR20ADMMWork work;
    std::vector<Binary128> matrix{
        static_cast<Binary128>(4.0), static_cast<Binary128>(1.0),
        static_cast<Binary128>(1.0), static_cast<Binary128>(3.0),
    };
    const ALR20ADMMFactor factor = al_r20_admm_factor_matrix(
        matrix, 2U, &work);
    std::vector<Binary128> solution;
    Binary128 residual = static_cast<Binary128>(0.0);
    Binary128 bound = static_cast<Binary128>(0.0);
    const bool solved = al_r20_admm_solve(factor,
        {static_cast<Binary128>(1.0), static_cast<Binary128>(2.0)},
        &solution, &residual, &bound, &work);
    const Binary128 tolerance = ldexpq(static_cast<Binary128>(1.0), -100);
    std::vector<Binary128> corrupted{
        static_cast<Binary128>(-1.0), static_cast<Binary128>(0.0),
        static_cast<Binary128>(0.0), static_cast<Binary128>(1.0),
    };
    const ALR20ADMMFactor rejected = al_r20_admm_factor_matrix(
        corrupted, 2U, nullptr);
    return factor.passed && solved && solution.size() == 2U
        && fabsq(solution[0] - static_cast<Binary128>(1.0)
                / static_cast<Binary128>(11.0))
            <= tolerance
        && fabsq(solution[1] - static_cast<Binary128>(7.0)
                / static_cast<Binary128>(11.0))
            <= tolerance
        && residual <= bound && !rejected.passed;
}

struct ALR20ADMMCheckpoint {
    ALR20OracleMetrics metrics;
    Binary128 primal_consensus = static_cast<Binary128>(0.0);
    Binary128 dual_consensus = static_cast<Binary128>(0.0);
    Binary128 linear_residual = static_cast<Binary128>(0.0);
    Binary128 linear_bound = static_cast<Binary128>(0.0);
    std::string root;
};

struct ALR20ADMMCase {
    bool exact = false;
    bool certified = false;
    std::size_t cycles = 0U;
    ALR20ADMMFactor factor;
    ALR20ADMMWork work;
    std::vector<ALR20ADMMCheckpoint> checkpoints;
    ALR20ADMMCheckpoint selected;
    std::string root;
};

ALR20ADMMCheckpoint al_r20_admm_checkpoint(
    const ALR20ProblemData& problem,
    const std::vector<Binary128Vec3>& primal,
    const std::vector<Binary128>& normalized_lambda,
    const std::vector<Binary128>& q,
    const std::vector<Binary128>& u,
    const std::vector<Binary128Vec3>& s,
    const std::vector<Binary128Vec3>& previous_v,
    const std::vector<Binary128>& previous_u,
    const std::vector<Binary128>& diagonal,
    std::size_t cycle, Binary128 linear_residual, Binary128 linear_bound,
    ALR20ADMMWork* work) {
    ALR20ADMMCheckpoint result;
    std::vector<Binary128> lambda(normalized_lambda.size());
    for (std::size_t row = 0U; row < lambda.size(); ++row)
        lambda[row] = normalized_lambda[row]
            / static_cast<Binary128>(problem.row_scale[row]);
    result.metrics = al_r20_q_metrics(
        problem, primal, lambda, diagonal, cycle);
    for (std::size_t row = 0U; row < q.size(); ++row)
        result.primal_consensus = fmaxq(result.primal_consensus,
            fabsq(q[row] - u[row]));
    Binary128 displacement = static_cast<Binary128>(0.0);
    for (std::size_t particle = 0U; particle < s.size(); ++particle)
        displacement = fmaxq(displacement,
            al_r20_q_absmax(s[particle] - primal[particle])
                / static_cast<Binary128>(AL_R20_TRUST_RADIUS));
    result.primal_consensus = fmaxq(
        result.primal_consensus, displacement);
    std::vector<Binary128> u_change(u.size());
    for (std::size_t row = 0U; row < u.size(); ++row)
        u_change[row] = u[row] - previous_u[row];
    const std::vector<Binary128Vec3> bt_change =
        al_r20_admm_transpose(problem, u_change, work);
    for (std::size_t particle = 0U; particle < primal.size(); ++particle)
        result.dual_consensus = fmaxq(result.dual_consensus,
            al_r20_q_absmax(al_r20_q_add(bt_change[particle],
                primal[particle] - previous_v[particle]))
                / static_cast<Binary128>(AL_R20_TRUST_RADIUS));
    result.linear_residual = linear_residual;
    result.linear_bound = linear_bound;
    std::ostringstream material;
    material << result.metrics.root << ':'
             << al_binary128_hex(result.primal_consensus) << ':'
             << al_binary128_hex(result.dual_consensus) << ':'
             << al_binary128_hex(result.linear_residual) << ':'
             << al_binary128_hex(result.linear_bound);
    result.root = sha256_hex(material.str());
    return result;
}

ALR20ADMMCase al_r20_run_admm_oracle(const ALR20ProblemData& problem) {
    ALR20ADMMCase result;
    const std::vector<Binary128> diagonal = al_r20_q_diagonal(problem.op);
    std::vector<Binary128Vec3> initial = al_r20_q_project_domain(
        al_r20_q_promote(problem.target), problem.lower, problem.upper);
    std::vector<Binary128> zero_lambda(
        problem.row_scale.size(), static_cast<Binary128>(0.0));
    ALR20ADMMCheckpoint initial_checkpoint;
    initial_checkpoint.metrics = al_r20_q_metrics(
        problem, initial, zero_lambda, diagonal, 0U);
    initial_checkpoint.root = initial_checkpoint.metrics.root;
    result.checkpoints.push_back(initial_checkpoint);
    if (initial_checkpoint.metrics.certified) {
        result.exact = true;
        result.certified = true;
        result.selected = initial_checkpoint;
        result.root = initial_checkpoint.root;
        return result;
    }

    result.factor = al_r20_admm_build_factor(problem, &result.work);
    if (!result.factor.passed) return result;
    const std::size_t rows = problem.row_scale.size();
    const std::size_t particles = problem.target.size();
    std::vector<Binary128Vec3> s(particles);
    std::vector<Binary128Vec3> v(particles);
    std::vector<Binary128Vec3> z(particles);
    std::vector<Binary128> u(rows);
    std::vector<Binary128> y(rows, static_cast<Binary128>(0.0));
    bool exact = true;
    for (std::size_t row = 0U; row < rows; ++row) {
        u[row] = static_cast<Binary128>(
            problem.workspace.support.constraint[row])
            / static_cast<Binary128>(problem.row_scale[row]);
        exact = exact && u[row] <= static_cast<Binary128>(0.0);
    }
    std::size_t checkpoint_index = 0U;
    for (std::size_t cycle = 1U;
         cycle <= AL_R20_ADMM_CHECKPOINTS.back() && exact; ++cycle) {
        std::vector<Binary128> row_rhs(rows);
        for (std::size_t row = 0U; row < rows; ++row) {
            const Binary128 d = static_cast<Binary128>(
                problem.workspace.support.constraint[row])
                / static_cast<Binary128>(problem.row_scale[row]);
            row_rhs[row] = u[row] - d - y[row];
        }
        const std::vector<Binary128Vec3> bt =
            al_r20_admm_transpose(problem, row_rhs, &result.work);
        const std::vector<Binary128Vec3> target =
            al_r20_q_promote(problem.target);
        std::vector<Binary128Vec3> rhs(particles);
        for (std::size_t particle = 0U; particle < particles; ++particle)
            rhs[particle] = al_r20_q_add(al_r20_q_add(
                target[particle], bt[particle]), v[particle] - z[particle]);
        std::vector<Binary128> flat_solution;
        Binary128 linear_residual = static_cast<Binary128>(0.0);
        Binary128 linear_bound = static_cast<Binary128>(0.0);
        exact = al_r20_admm_solve(result.factor,
            al_r20_admm_flatten(rhs), &flat_solution,
            &linear_residual, &linear_bound, &result.work);
        if (!exact) break;
        s = al_r20_admm_unflatten(flat_solution);
        const std::vector<Binary128> q =
            al_r20_admm_action(problem, s, &result.work);
        const std::vector<Binary128> previous_u = u;
        const std::vector<Binary128Vec3> previous_v = v;
        for (std::size_t row = 0U; row < rows; ++row)
            u[row] = fminq(static_cast<Binary128>(0.0), q[row] + y[row]);
        std::vector<Binary128Vec3> domain_input(particles);
        for (std::size_t particle = 0U; particle < particles; ++particle)
            domain_input[particle] = al_r20_q_add(s[particle], z[particle]);
        v = al_r20_q_project_domain(
            domain_input, problem.lower, problem.upper);
        ++result.work.domain_projections;
        for (std::size_t row = 0U; row < rows; ++row)
            y[row] += q[row] - u[row];
        for (std::size_t particle = 0U; particle < particles; ++particle)
            z[particle] = al_r20_q_add(
                z[particle], s[particle] - v[particle]);
        exact = al_r20_admm_finite(s) && al_r20_admm_finite(v)
            && al_r20_admm_finite(z) && al_r20_admm_finite(u)
            && al_r20_admm_finite(y);
        result.cycles = cycle;
        if (checkpoint_index < AL_R20_ADMM_CHECKPOINTS.size()
            && cycle == AL_R20_ADMM_CHECKPOINTS[checkpoint_index]) {
            ALR20ADMMCheckpoint checkpoint = al_r20_admm_checkpoint(
                problem, v, y, q, u, s, previous_v, previous_u,
                diagonal, cycle, linear_residual, linear_bound, &result.work);
            exact = exact && checkpoint.metrics.finite
                && checkpoint.linear_residual <= checkpoint.linear_bound;
            result.checkpoints.push_back(checkpoint);
            if (checkpoint.metrics.certified) {
                result.certified = true;
                result.selected = checkpoint;
                break;
            }
            ++checkpoint_index;
        }
    }
    result.exact = exact;
    if (!result.certified && !result.checkpoints.empty())
        result.selected = result.checkpoints.back();
    std::ostringstream material;
    material << result.exact << ':' << result.certified << ':'
             << result.cycles << ':' << result.factor.matrix_root << ':'
             << result.factor.factor_root << '|';
    for (const ALR20ADMMCheckpoint& checkpoint : result.checkpoints)
        material << checkpoint.root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_admm_oracle_controls() {
#if defined(__linux__) && defined(__x86_64__) && defined(__GNUC__) \
    && defined(__SIZEOF_FLOAT128__)
    constexpr bool frozen_platform = true;
#else
    constexpr bool frozen_platform = false;
#endif
    const bool profile_exact = frozen_platform
        && sizeof(Binary128) == 16U && FLT_RADIX == 2
        && FLT128_MANT_DIG == 113;
    const bool dense_controls = profile_exact && al_r20_admm_dense_controls();
    std::vector<ALR20ManifestCase> cases = al_r20_manifest_cases();
    const bool parent_exact = dense_controls
        && al_r20_manifest_semantic_root(cases) == AL_R20_MANIFEST_SHA256
        && cases.size() == AL_R20_PROBLEM_SHA256.size();
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    std::vector<ALR20ProblemData> problems;
    std::vector<ALR20ADMMCase> oracle;
    bool exact = parent_exact;
    bool all_certified = true;
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        ALR20ProblemData problem = al_r20_materialize_problem(
            cases[index], &sparse_work, &static_work, &adjacency_work);
        const bool problem_exact = problem.exact
            && index < AL_R20_PROBLEM_SHA256.size()
            && problem.problem_root == AL_R20_PROBLEM_SHA256[index]
            && problem.source_positive == 0U;
        ALR20ADMMCase result = problem_exact
            ? al_r20_run_admm_oracle(problem) : ALR20ADMMCase{};
        exact = exact && problem_exact && result.exact;
        all_certified = all_certified && result.certified;
        oracle.push_back(std::move(result));
        problems.push_back(std::move(problem));
    }
    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    exact = exact && sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const std::string route = !parent_exact
        ? "GLOBAL_ORACLE_PARENT_REJECTED"
        : (!exact ? "GLOBAL_ORACLE_ITERATION_REJECTED"
                  : (all_certified ? "GLOBAL_ADMM_ORACLE_CERTIFIED"
                                   : "GLOBAL_ADMM_ORACLE_UNRESOLVED"));
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r2-global-admm-oracle.v1|"
             << AL_R20_ORACLE_SHA256 << ':' << route << '|';
    for (const ALR20ADMMCase& value : oracle)
        semantic << value.root << ';';
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r2_global_admm_oracle.v1\""
           << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
           << ",\"route\":\"" << route << "\""
           << ",\"parent_oracle_sha256\":\"" << AL_R20_ORACLE_SHA256
           << "\",\"profile_exact\":" << (profile_exact ? "true" : "false")
           << ",\"dense_controls\":" << (dense_controls ? "true" : "false")
           << ",\"all_certified\":" << (all_certified ? "true" : "false")
           << ",\"cases\":[";
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20ADMMCase& value = oracle[index];
        const ALR20ADMMCheckpoint& selected = value.selected;
        report << "{\"name\":\"" << cases[index].fixture.name
               << "\",\"role\":\"" << cases[index].role
               << "\",\"certified\":"
               << (value.certified ? "true" : "false")
               << ",\"selected_cycle\":" << selected.metrics.cycle
               << ",\"cycles\":" << value.cycles
               << ",\"checkpoint_count\":" << value.checkpoints.size()
               << ",\"primal\":\"" << al_binary128_text(selected.metrics.primal)
               << "\",\"dual_mapping\":\""
               << al_binary128_text(selected.metrics.dual_mapping)
               << "\",\"complementarity\":\""
               << al_binary128_text(selected.metrics.complementarity)
               << "\",\"stationarity\":\""
               << al_binary128_text(selected.metrics.stationarity)
               << "\",\"gap_scaled\":\""
               << al_binary128_text(selected.metrics.gap_scaled)
               << "\",\"primal_consensus\":\""
               << al_binary128_text(selected.primal_consensus)
               << "\",\"dual_consensus\":\""
               << al_binary128_text(selected.dual_consensus)
               << "\",\"factor_dimension\":" << value.factor.dimension
               << ",\"work\":{\"matrix_outer_terms\":"
               << value.work.matrix_outer_terms << ",\"factor_terms\":"
               << value.work.factor_terms << ",\"solve_terms\":"
               << value.work.solve_terms << ",\"action_terms\":"
               << value.work.normalized_action_terms
               << ",\"transpose_terms\":"
               << value.work.normalized_transpose_terms
               << ",\"domain_projections\":"
               << value.work.domain_projections << "},\"root\":\""
               << value.root << "\"}";
    }
    report << "]"
              ",\"candidate_iterations\":0,\"old_oracle_reruns\":0"
              ",\"timing_admitted\":false"
              ",\"runtime_binary128_authorized\":false"
              ",\"runtime_authority\":false,\"production_authority\":false"
              ",\"result_sha256\":\""
           << result_sha << "\"}";
    return {exact, report.str()};
}
