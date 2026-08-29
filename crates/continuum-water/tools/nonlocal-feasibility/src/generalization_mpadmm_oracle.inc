namespace {

constexpr const char* AL_R20_ADMM_SHA256 =
    "929a6676182dc7702a8950343833e56faad4ac46e4784fda1c67f169f6eff08c";
constexpr std::array<std::size_t, 6U> AL_R20_MPADMM_CHECKPOINTS{{
    1U << 4U,
    1U << 6U,
    1U << 8U,
    1U << 10U,
    1U << 12U,
    1U << 14U,
}};
constexpr std::size_t AL_R20_MPADMM_UPDATE_INTERVAL = 64U;

Binary128 al_r20_mpadmm_norm(const std::vector<Binary128>& value) {
    Binary128Accumulator sum;
    for (Binary128 scalar : value) sum.add(scalar * scalar);
    return sqrtq(fmaxq(static_cast<Binary128>(0.0), sum.value()));
}

Binary128 al_r20_mpadmm_norm(const std::vector<Binary128Vec3>& value) {
    Binary128Accumulator sum;
    for (Binary128Vec3 vector : value)
        sum.add(al_r20_q_dot(vector, vector));
    return sqrtq(fmaxq(static_cast<Binary128>(0.0), sum.value()));
}

ALR20ADMMFactor al_r20_mpadmm_build_factor(
    const ALR20ProblemData& problem,
    Binary128 rho_density, Binary128 rho_domain,
    ALR20ADMMWork* work) {
    const std::size_t dimension = 3U * problem.target.size();
    std::vector<Binary128> matrix(
        dimension * dimension, static_cast<Binary128>(0.0));
    for (std::size_t index = 0U; index < dimension; ++index)
        matrix[index * dimension + index] =
            static_cast<Binary128>(1.0) + rho_domain;
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
                            += rho_density * left_value * right_value;
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

bool al_r20_mpadmm_solve(const ALR20ADMMFactor& factor,
    const std::vector<Binary128>& rhs, std::vector<Binary128>* solution,
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
    return al_r20_admm_finite(*solution);
}

bool al_r20_mpadmm_linear_audit(const ALR20ADMMFactor& factor,
    const std::vector<Binary128>& rhs,
    const std::vector<Binary128>& solution,
    Binary128* maximum_residual, Binary128* maximum_bound) {
    const std::size_t dimension = factor.dimension;
    if (!factor.passed || rhs.size() != dimension
        || solution.size() != dimension) return false;
    Binary128 residual_max = static_cast<Binary128>(0.0);
    Binary128 bound_max = static_cast<Binary128>(0.0);
    bool exact = true;
    for (std::size_t row = 0U; row < dimension && exact; ++row) {
        Binary128Accumulator image;
        Binary128 absolute = fabsq(rhs[row]);
        for (std::size_t column = 0U; column < dimension; ++column) {
            const Binary128 term = factor.matrix[row * dimension + column]
                * solution[column];
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

Binary128 al_r20_mpadmm_penalty(
    Binary128 old_value, Binary128 numerator, Binary128 denominator) {
    const Binary128 ten = static_cast<Binary128>(10.0);
    Binary128 proposal = old_value;
    if (numerator == static_cast<Binary128>(0.0)
        && denominator > static_cast<Binary128>(0.0))
        proposal = old_value / ten;
    else if (numerator > static_cast<Binary128>(0.0)
        && denominator == static_cast<Binary128>(0.0))
        proposal = ten * old_value;
    else if (numerator > static_cast<Binary128>(0.0)
        && denominator > static_cast<Binary128>(0.0))
        proposal = numerator / denominator;
    const Binary128 lower = ldexpq(static_cast<Binary128>(1.0), -20);
    const Binary128 upper = ldexpq(static_cast<Binary128>(1.0), 20);
    return fminq(upper, fmaxq(lower, proposal));
}

bool al_r20_mpadmm_controls() {
    const Binary128 two = static_cast<Binary128>(2.0);
    const Binary128 four = static_cast<Binary128>(4.0);
    const Binary128 one = static_cast<Binary128>(1.0);
    return al_r20_mpadmm_penalty(one, four, two) == two
        && al_r20_mpadmm_penalty(one, static_cast<Binary128>(0.0), two)
            == one / static_cast<Binary128>(10.0)
        && al_r20_mpadmm_penalty(one, two, static_cast<Binary128>(0.0))
            == static_cast<Binary128>(10.0)
        && al_r20_mpadmm_penalty(one, static_cast<Binary128>(0.0),
                static_cast<Binary128>(0.0)) == one;
}

struct ALR20MPADMMUpdate {
    std::size_t cycle = 0U;
    Binary128 density_before = static_cast<Binary128>(1.0);
    Binary128 density_after = static_cast<Binary128>(1.0);
    Binary128 domain_before = static_cast<Binary128>(1.0);
    Binary128 domain_after = static_cast<Binary128>(1.0);
    Binary128 density_dual_change = static_cast<Binary128>(0.0);
    Binary128 density_prox_change = static_cast<Binary128>(0.0);
    Binary128 domain_dual_change = static_cast<Binary128>(0.0);
    Binary128 domain_prox_change = static_cast<Binary128>(0.0);
    std::string factor_root;
    std::string root;
};

struct ALR20MPADMMCheckpoint {
    ALR20OracleMetrics metrics;
    Binary128 density_primal = static_cast<Binary128>(0.0);
    Binary128 domain_primal = static_cast<Binary128>(0.0);
    Binary128 density_dual = static_cast<Binary128>(0.0);
    Binary128 domain_dual = static_cast<Binary128>(0.0);
    Binary128 linear_residual = static_cast<Binary128>(0.0);
    Binary128 linear_bound = static_cast<Binary128>(0.0);
    Binary128 rho_density = static_cast<Binary128>(1.0);
    Binary128 rho_domain = static_cast<Binary128>(1.0);
    std::string root;
};

struct ALR20MPADMMCase {
    bool exact = false;
    bool stable = true;
    bool certified = false;
    std::size_t cycles = 0U;
    std::size_t factor_builds = 0U;
    ALR20ADMMWork work;
    std::vector<ALR20MPADMMCheckpoint> checkpoints;
    std::vector<ALR20MPADMMUpdate> updates;
    ALR20MPADMMCheckpoint selected;
    std::string root;
};

ALR20MPADMMCheckpoint al_r20_mpadmm_checkpoint(
    const ALR20ProblemData& problem,
    const std::vector<Binary128Vec3>& v,
    const std::vector<Binary128Vec3>& s,
    const std::vector<Binary128>& q,
    const std::vector<Binary128>& u,
    const std::vector<Binary128>& y,
    const std::vector<Binary128>& previous_u,
    const std::vector<Binary128Vec3>& previous_v,
    const std::vector<Binary128>& diagonal,
    Binary128 rho_density, Binary128 rho_domain,
    std::size_t cycle, Binary128 linear_residual,
    Binary128 linear_bound, ALR20ADMMWork* work) {
    ALR20MPADMMCheckpoint result;
    std::vector<Binary128> lambda(y.size());
    for (std::size_t row = 0U; row < y.size(); ++row)
        lambda[row] = rho_density * y[row]
            / static_cast<Binary128>(problem.row_scale[row]);
    result.metrics = al_r20_q_metrics(problem, v, lambda, diagonal, cycle);
    for (std::size_t row = 0U; row < q.size(); ++row)
        result.density_primal = fmaxq(
            result.density_primal, fabsq(q[row] - u[row]));
    for (std::size_t particle = 0U; particle < s.size(); ++particle)
        result.domain_primal = fmaxq(result.domain_primal,
            al_r20_q_absmax(s[particle] - v[particle])
                / static_cast<Binary128>(AL_R20_TRUST_RADIUS));
    std::vector<Binary128> u_change(u.size());
    for (std::size_t row = 0U; row < u.size(); ++row)
        u_change[row] = u[row] - previous_u[row];
    const std::vector<Binary128Vec3> bt_change =
        al_r20_admm_transpose(problem, u_change, work);
    for (Binary128Vec3 value : bt_change)
        result.density_dual = fmaxq(result.density_dual,
            rho_density * al_r20_q_absmax(value)
                / static_cast<Binary128>(AL_R20_TRUST_RADIUS));
    for (std::size_t particle = 0U; particle < v.size(); ++particle)
        result.domain_dual = fmaxq(result.domain_dual,
            rho_domain * al_r20_q_absmax(
                v[particle] - previous_v[particle])
                / static_cast<Binary128>(AL_R20_TRUST_RADIUS));
    result.linear_residual = linear_residual;
    result.linear_bound = linear_bound;
    result.rho_density = rho_density;
    result.rho_domain = rho_domain;
    std::ostringstream material;
    material << result.metrics.root << ':'
             << al_binary128_hex(result.density_primal) << ':'
             << al_binary128_hex(result.domain_primal) << ':'
             << al_binary128_hex(result.density_dual) << ':'
             << al_binary128_hex(result.domain_dual) << ':'
             << al_binary128_hex(linear_residual) << ':'
             << al_binary128_hex(linear_bound) << ':'
             << al_binary128_hex(rho_density) << ':'
             << al_binary128_hex(rho_domain);
    result.root = sha256_hex(material.str());
    return result;
}

ALR20MPADMMCase al_r20_run_mpadmm_oracle(
    const ALR20ProblemData& problem) {
    ALR20MPADMMCase result;
    const std::vector<Binary128> diagonal = al_r20_q_diagonal(problem.op);
    const std::vector<Binary128Vec3> target =
        al_r20_q_promote(problem.target);
    const std::vector<Binary128Vec3> initial = al_r20_q_project_domain(
        target, problem.lower, problem.upper);
    std::vector<Binary128> zero_lambda(
        problem.row_scale.size(), static_cast<Binary128>(0.0));
    ALR20MPADMMCheckpoint initial_checkpoint;
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

    Binary128 rho_density = static_cast<Binary128>(1.0);
    Binary128 rho_domain = static_cast<Binary128>(1.0);
    ALR20ADMMFactor factor = al_r20_mpadmm_build_factor(
        problem, rho_density, rho_domain, &result.work);
    ++result.factor_builds;
    if (!factor.passed) return result;
    bool factor_fresh = true;
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
    std::vector<Binary128> reference_density_dual(rows,
        static_cast<Binary128>(0.0));
    std::vector<Binary128Vec3> reference_domain_dual(particles);
    std::vector<Binary128> reference_u = u;
    std::vector<Binary128Vec3> reference_v = v;
    std::size_t checkpoint_index = 0U;
    for (std::size_t cycle = 1U;
         cycle <= AL_R20_MPADMM_CHECKPOINTS.back() && exact; ++cycle) {
        std::vector<Binary128> row_rhs(rows);
        for (std::size_t row = 0U; row < rows; ++row) {
            const Binary128 d = static_cast<Binary128>(
                problem.workspace.support.constraint[row])
                / static_cast<Binary128>(problem.row_scale[row]);
            row_rhs[row] = rho_density * (u[row] - d - y[row]);
        }
        const std::vector<Binary128Vec3> bt =
            al_r20_admm_transpose(problem, row_rhs, &result.work);
        std::vector<Binary128Vec3> rhs(particles);
        for (std::size_t particle = 0U; particle < particles; ++particle)
            rhs[particle] = al_r20_q_add(al_r20_q_add(target[particle],
                bt[particle]), al_r20_q_scale(rho_domain,
                    v[particle] - z[particle]));
        const std::vector<Binary128> flat_rhs = al_r20_admm_flatten(rhs);
        std::vector<Binary128> flat_solution;
        exact = al_r20_mpadmm_solve(
            factor, flat_rhs, &flat_solution, &result.work);
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

        const bool checkpoint_due =
            checkpoint_index < AL_R20_MPADMM_CHECKPOINTS.size()
            && cycle == AL_R20_MPADMM_CHECKPOINTS[checkpoint_index];
        Binary128 linear_residual = static_cast<Binary128>(0.0);
        Binary128 linear_bound = static_cast<Binary128>(0.0);
        if (factor_fresh || checkpoint_due) {
            exact = exact && al_r20_mpadmm_linear_audit(factor,
                flat_rhs, flat_solution, &linear_residual, &linear_bound);
            factor_fresh = false;
        }
        if (checkpoint_due) {
            const ALR20MPADMMCheckpoint checkpoint =
                al_r20_mpadmm_checkpoint(problem, v, s, q, u, y,
                    previous_u, previous_v, diagonal, rho_density, rho_domain,
                    cycle, linear_residual, linear_bound, &result.work);
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

        if (cycle % AL_R20_MPADMM_UPDATE_INTERVAL == 0U) {
            std::vector<Binary128> density_dual(rows);
            std::vector<Binary128> density_dual_change(rows);
            std::vector<Binary128> u_change(rows);
            for (std::size_t row = 0U; row < rows; ++row) {
                density_dual[row] = rho_density * y[row];
                density_dual_change[row] = density_dual[row]
                    - reference_density_dual[row];
                u_change[row] = u[row] - reference_u[row];
            }
            std::vector<Binary128Vec3> domain_dual(particles);
            std::vector<Binary128Vec3> domain_dual_change(particles);
            std::vector<Binary128Vec3> v_change(particles);
            for (std::size_t particle = 0U; particle < particles; ++particle) {
                domain_dual[particle] =
                    al_r20_q_scale(rho_domain, z[particle]);
                domain_dual_change[particle] = domain_dual[particle]
                    - reference_domain_dual[particle];
                v_change[particle] = v[particle] - reference_v[particle];
            }
            ALR20MPADMMUpdate update;
            update.cycle = cycle;
            update.density_before = rho_density;
            update.domain_before = rho_domain;
            update.density_dual_change =
                al_r20_mpadmm_norm(density_dual_change);
            update.density_prox_change = al_r20_mpadmm_norm(u_change);
            update.domain_dual_change =
                al_r20_mpadmm_norm(domain_dual_change);
            update.domain_prox_change = al_r20_mpadmm_norm(v_change);
            update.density_after = al_r20_mpadmm_penalty(rho_density,
                update.density_dual_change, update.density_prox_change);
            update.domain_after = al_r20_mpadmm_penalty(rho_domain,
                update.domain_dual_change, update.domain_prox_change);
            result.stable = result.stable
                && finiteq(update.density_after) != 0
                && finiteq(update.domain_after) != 0
                && update.density_after > static_cast<Binary128>(0.0)
                && update.domain_after > static_cast<Binary128>(0.0);
            if (!result.stable) { exact = false; break; }
            reference_density_dual = density_dual;
            reference_domain_dual = domain_dual;
            reference_u = u;
            reference_v = v;
            if (update.density_after != rho_density) {
                const Binary128 scale = rho_density / update.density_after;
                for (Binary128& value : y) value *= scale;
            }
            if (update.domain_after != rho_domain) {
                const Binary128 scale = rho_domain / update.domain_after;
                for (Binary128Vec3& value : z)
                    value = al_r20_q_scale(scale, value);
            }
            rho_density = update.density_after;
            rho_domain = update.domain_after;
            factor = al_r20_mpadmm_build_factor(
                problem, rho_density, rho_domain, &result.work);
            ++result.factor_builds;
            exact = exact && factor.passed;
            factor_fresh = true;
            update.factor_root = factor.factor_root;
            std::ostringstream update_material;
            update_material << cycle << ':'
                << al_binary128_hex(update.density_before) << ':'
                << al_binary128_hex(update.density_after) << ':'
                << al_binary128_hex(update.domain_before) << ':'
                << al_binary128_hex(update.domain_after) << ':'
                << al_binary128_hex(update.density_dual_change) << ':'
                << al_binary128_hex(update.density_prox_change) << ':'
                << al_binary128_hex(update.domain_dual_change) << ':'
                << al_binary128_hex(update.domain_prox_change) << ':'
                << update.factor_root;
            update.root = sha256_hex(update_material.str());
            result.updates.push_back(std::move(update));
        }
    }
    result.exact = exact;
    if (!result.certified && !result.checkpoints.empty())
        result.selected = result.checkpoints.back();
    std::ostringstream material;
    material << result.exact << ':' << result.stable << ':'
             << result.certified << ':' << result.cycles << ':'
             << result.factor_builds << '|';
    for (const ALR20MPADMMCheckpoint& checkpoint : result.checkpoints)
        material << checkpoint.root << ';';
    material << '|';
    for (const ALR20MPADMMUpdate& update : result.updates)
        material << update.root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_mpadmm_oracle_controls() {
#if defined(__linux__) && defined(__x86_64__) && defined(__GNUC__) \
    && defined(__SIZEOF_FLOAT128__)
    constexpr bool frozen_platform = true;
#else
    constexpr bool frozen_platform = false;
#endif
    const bool profile_exact = frozen_platform
        && sizeof(Binary128) == 16U && FLT_RADIX == 2
        && FLT128_MANT_DIG == 113;
    const bool dense_controls = profile_exact
        && al_r20_admm_dense_controls() && al_r20_mpadmm_controls();
    std::vector<ALR20ManifestCase> cases = al_r20_manifest_cases();
    const bool parent_exact = dense_controls
        && al_r20_manifest_semantic_root(cases) == AL_R20_MANIFEST_SHA256
        && cases.size() == AL_R20_PROBLEM_SHA256.size();
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    std::vector<ALR20ProblemData> problems;
    std::vector<ALR20MPADMMCase> oracle;
    bool exact = parent_exact;
    bool all_stable = true;
    bool all_certified = true;
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        ALR20ProblemData problem = al_r20_materialize_problem(
            cases[index], &sparse_work, &static_work, &adjacency_work);
        const bool problem_exact = problem.exact
            && index < AL_R20_PROBLEM_SHA256.size()
            && problem.problem_root == AL_R20_PROBLEM_SHA256[index]
            && problem.source_positive == 0U;
        ALR20MPADMMCase result = problem_exact
            ? al_r20_run_mpadmm_oracle(problem) : ALR20MPADMMCase{};
        exact = exact && problem_exact && result.exact;
        all_stable = all_stable && result.stable;
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
        ? "MULTIPARAMETER_ADMM_PARENT_REJECTED"
        : (!exact || !all_stable ? "MULTIPARAMETER_ADMM_UNSTABLE"
            : (all_certified ? "MULTIPARAMETER_ADMM_DEVELOPMENT_CERTIFIED"
                : "MULTIPARAMETER_ADMM_ACTIVE_SET_REFINEMENT_REQUIRED"));
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r3-mpadmm-development.v1|"
             << AL_R20_ADMM_SHA256 << ':' << route << '|';
    for (const ALR20MPADMMCase& value : oracle)
        semantic << value.root << ';';
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r3_mpadmm_development.v1\""
           << ",\"status\":\"" << (parent_exact ? "PASS" : "FAIL") << "\""
           << ",\"route\":\"" << route << "\""
           << ",\"parent_admm_sha256\":\"" << AL_R20_ADMM_SHA256
           << "\",\"profile_exact\":" << (profile_exact ? "true" : "false")
           << ",\"dense_controls\":" << (dense_controls ? "true" : "false")
           << ",\"all_stable\":" << (all_stable ? "true" : "false")
           << ",\"all_certified\":" << (all_certified ? "true" : "false")
           << ",\"cases\":[";
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20MPADMMCase& value = oracle[index];
        const ALR20MPADMMCheckpoint& selected = value.selected;
        report << "{\"name\":\"" << cases[index].fixture.name
               << "\",\"role\":\"" << cases[index].role
               << "\",\"certified\":"
               << (value.certified ? "true" : "false")
               << ",\"selected_cycle\":" << selected.metrics.cycle
               << ",\"cycles\":" << value.cycles
               << ",\"checkpoint_count\":" << value.checkpoints.size()
               << ",\"factor_builds\":" << value.factor_builds
               << ",\"penalty_updates\":" << value.updates.size()
               << ",\"primal\":\"" << al_binary128_text(selected.metrics.primal)
               << "\",\"dual_mapping\":\""
               << al_binary128_text(selected.metrics.dual_mapping)
               << "\",\"complementarity\":\""
               << al_binary128_text(selected.metrics.complementarity)
               << "\",\"stationarity\":\""
               << al_binary128_text(selected.metrics.stationarity)
               << "\",\"gap\":\"" << al_binary128_text(selected.metrics.gap)
               << "\",\"gap_bound\":\""
               << al_binary128_text(selected.metrics.gap_bound)
               << "\",\"gap_scaled\":\""
               << al_binary128_text(selected.metrics.gap_scaled)
               << "\",\"density_primal\":\""
               << al_binary128_text(selected.density_primal)
               << "\",\"domain_primal\":\""
               << al_binary128_text(selected.domain_primal)
               << "\",\"density_dual\":\""
               << al_binary128_text(selected.density_dual)
               << "\",\"domain_dual\":\""
               << al_binary128_text(selected.domain_dual)
               << "\",\"rho_density\":\""
               << al_binary128_text(selected.rho_density)
               << "\",\"rho_domain\":\""
               << al_binary128_text(selected.rho_domain)
               << "\",\"updates\":[";
        for (std::size_t update_index = 0U;
             update_index < value.updates.size(); ++update_index) {
            if (update_index != 0U) report << ',';
            const ALR20MPADMMUpdate& update = value.updates[update_index];
            report << "{\"cycle\":" << update.cycle
                   << ",\"density\":\""
                   << al_binary128_text(update.density_after)
                   << "\",\"domain\":\""
                   << al_binary128_text(update.domain_after) << "\"}";
        }
        report << "],\"work\":{\"matrix_outer_terms\":"
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
              ",\"generalization_authority\":false"
              ",\"timing_admitted\":false"
              ",\"runtime_binary128_authorized\":false"
              ",\"runtime_authority\":false,\"production_authority\":false"
              ",\"result_sha256\":\""
           << result_sha << "\"}";
    return {parent_exact, report.str()};
}
