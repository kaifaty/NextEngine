namespace {

constexpr const char* AL_R20_PREFLIGHT_SHA256 =
    "16a24ef3040411ef0bd29294f0887b29a1cce7690c7d8a8bee511ea58880b1f6";
constexpr std::array<const char*, 8U> AL_R20_PROBLEM_SHA256{{
    "05ccf39a0c52ef5413da8564a84155284ca20cd05efdf7253440aaf99cd79a09",
    "62cd8d6070aa79e5694a170e8e454b95c180bcc48e35f7bfdb1211f68da792a3",
    "24ab412215d6444c2adf50074c13609b50bde486daee3af0922ffcb8efbbfaba",
    "cb3779458eb1c3d020b956faee9e6348867185a5b247ebe94963589d8ed09ae6",
    "e9ee8cf9bd5ca61018e424313f89d57d8c5603918a2c5c45e9b860452ec9cec0",
    "fbf5eedcecc0ea95d266616393404114b28f3354b9ddd4240baf73ee64da980d",
    "336c12bdd8d14a670475016a51ea0b211d01012233d5c0accb3d9541fa62e65f",
    "594b80926812eb1219091d7e3b5c7013497ce4b53ffc80818155280d71182bde",
}};
constexpr std::array<std::size_t, 6U> AL_R20_ORACLE_CHECKPOINTS{{
    1U << 8U,
    1U << 10U,
    1U << 12U,
    1U << 14U,
    1U << 16U,
    1U << 18U,
}};

Binary128Vec3 al_r20_q_add(Binary128Vec3 lhs, Binary128Vec3 rhs) {
    return {lhs.x + rhs.x, lhs.y + rhs.y, lhs.z + rhs.z};
}

Binary128Vec3 al_r20_q_scale(Binary128 scale, Binary128Vec3 value) {
    return {scale * value.x, scale * value.y, scale * value.z};
}

Binary128 al_r20_q_dot(Binary128Vec3 lhs, Binary128Vec3 rhs) {
    return lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z;
}

Binary128 al_r20_q_absmax(Binary128Vec3 value) {
    return fmaxq(fabsq(value.x), fmaxq(fabsq(value.y), fabsq(value.z)));
}

Binary128 al_r20_q_component(Binary128Vec3 value, int axis) {
    return axis == 0 ? value.x : (axis == 1 ? value.y : value.z);
}

void al_r20_q_set_component(
    Binary128Vec3* value, int axis, Binary128 scalar) {
    if (axis == 0) value->x = scalar;
    else if (axis == 1) value->y = scalar;
    else value->z = scalar;
}

Binary128 al_r20_q_clamp(
    Binary128 value, Binary128 lower, Binary128 upper) {
    return fminq(upper, fmaxq(lower, value));
}

Binary128 al_r20_q_gamma(std::size_t operations) {
    const Binary128 value = static_cast<Binary128>(operations)
        * static_cast<Binary128>(ldexpq(static_cast<Binary128>(1.0), -112));
    return value / (static_cast<Binary128>(1.0) - value);
}

std::vector<Binary128Vec3> al_r20_q_promote(
    const std::vector<Vec3>& values) {
    std::vector<Binary128Vec3> result;
    result.reserve(values.size());
    for (Vec3 value : values) result.push_back(al_binary128_vec3(value));
    return result;
}

std::string al_r20_q_vec_root(
    const std::vector<Binary128Vec3>& values) {
    std::ostringstream material;
    material << values.size();
    for (Binary128Vec3 value : values)
        material << ':' << al_binary128_hex(value.x)
                 << ':' << al_binary128_hex(value.y)
                 << ':' << al_binary128_hex(value.z);
    return sha256_hex(material.str());
}

std::string al_r20_q_scalar_root(const std::vector<Binary128>& values) {
    std::ostringstream material;
    material << values.size();
    for (Binary128 value : values)
        material << ':' << al_binary128_hex(value);
    return sha256_hex(material.str());
}

std::vector<Binary128> al_r20_q_diagonal(const ALR64Operator& op) {
    const std::size_t rows = op.entry_offsets.size() - 1U;
    std::vector<Binary128> result(rows);
    for (std::size_t row = 0U; row < rows; ++row) {
        Binary128Accumulator sum;
        for (std::size_t index = op.entry_offsets[row];
             index < op.entry_offsets[row + 1U]; ++index) {
            const Binary128Vec3 value =
                al_binary128_vec3(op.entries[index].coefficient);
            sum.add(al_r20_q_dot(value, value));
        }
        result[row] = sum.value();
    }
    return result;
}

std::vector<Binary128> al_r20_q_action(const ALR64Operator& op,
    const std::vector<Binary128Vec3>& value,
    const std::vector<double>& constraint,
    std::vector<Binary128>* error_bound = nullptr) {
    const std::size_t rows = constraint.size();
    std::vector<Binary128> result(rows);
    if (error_bound != nullptr) error_bound->assign(rows, static_cast<Binary128>(0.0));
    for (std::size_t row = 0U; row < rows; ++row) {
        Binary128Accumulator sum;
        Binary128 absolute = fabsq(static_cast<Binary128>(constraint[row]));
        sum.add(static_cast<Binary128>(constraint[row]));
        for (std::size_t index = op.entry_offsets[row];
             index < op.entry_offsets[row + 1U]; ++index) {
            const ALR64Entry& entry = op.entries[index];
            const Binary128 term = al_r20_q_dot(
                al_binary128_vec3(entry.coefficient), value[entry.particle]);
            sum.add(term);
            absolute += fabsq(term);
        }
        result[row] = sum.value();
        if (error_bound != nullptr) {
            const std::size_t terms = op.entry_offsets[row + 1U]
                - op.entry_offsets[row];
            (*error_bound)[row] = al_r20_q_gamma(4U * terms + 32U)
                * absolute;
        }
    }
    return result;
}

std::vector<Binary128Vec3> al_r20_q_transpose(
    const ALR64Operator& op, const std::vector<Binary128>& lambda,
    std::size_t particle_count) {
    std::vector<Binary128Vec3> result(particle_count);
    for (std::size_t row = 0U; row < lambda.size(); ++row)
        for (std::size_t index = op.entry_offsets[row];
             index < op.entry_offsets[row + 1U]; ++index) {
            const ALR64Entry& entry = op.entries[index];
            result[entry.particle] = al_r20_q_add(
                result[entry.particle],
                al_r20_q_scale(lambda[row],
                    al_binary128_vec3(entry.coefficient)));
        }
    return result;
}

std::vector<Binary128Vec3> al_r20_q_project_domain(
    const std::vector<Binary128Vec3>& input,
    const std::vector<Vec3>& lower,
    const std::vector<Vec3>& upper) {
    std::vector<Binary128Vec3> result(input.size());
    Binary128 norm_squared = static_cast<Binary128>(0.0);
    for (std::size_t particle = 0U; particle < input.size(); ++particle) {
        for (int axis = 0; axis < 3; ++axis) {
            const Binary128 value = al_r20_q_clamp(
                al_r20_q_component(input[particle], axis),
                static_cast<Binary128>(al_r41_component(lower[particle], axis)),
                static_cast<Binary128>(al_r41_component(upper[particle], axis)));
            al_r20_q_set_component(&result[particle], axis, value);
            norm_squared += value * value;
        }
    }
    const Binary128 radius = static_cast<Binary128>(AL_R20_TRUST_RADIUS);
    if (norm_squared <= radius * radius) return result;

    const auto norm_at = [&](Binary128 tau) {
        Binary128 value = static_cast<Binary128>(0.0);
        const Binary128 divisor = static_cast<Binary128>(1.0) + tau;
        for (std::size_t particle = 0U;
             particle < input.size(); ++particle)
            for (int axis = 0; axis < 3; ++axis) {
                const Binary128 component = al_r20_q_clamp(
                    al_r20_q_component(input[particle], axis) / divisor,
                    static_cast<Binary128>(
                        al_r41_component(lower[particle], axis)),
                    static_cast<Binary128>(
                        al_r41_component(upper[particle], axis)));
                value += component * component;
            }
        return value;
    };
    Binary128 low_tau = static_cast<Binary128>(0.0);
    Binary128 high_tau = static_cast<Binary128>(1.0);
    for (std::size_t expansion = 0U;
         expansion < 512U && norm_at(high_tau) > radius * radius;
         ++expansion)
        high_tau *= static_cast<Binary128>(2.0);
    for (std::size_t iteration = 0U; iteration < 256U; ++iteration) {
        const Binary128 middle = static_cast<Binary128>(0.5) * (low_tau + high_tau);
        if (norm_at(middle) > radius * radius) low_tau = middle;
        else high_tau = middle;
    }
    const Binary128 divisor = static_cast<Binary128>(1.0) + high_tau;
    for (std::size_t particle = 0U; particle < input.size(); ++particle)
        for (int axis = 0; axis < 3; ++axis)
            al_r20_q_set_component(&result[particle], axis,
                al_r20_q_clamp(
                    al_r20_q_component(input[particle], axis) / divisor,
                    static_cast<Binary128>(
                        al_r41_component(lower[particle], axis)),
                    static_cast<Binary128>(
                        al_r41_component(upper[particle], axis))));
    return result;
}

Binary128 al_r20_q_objective(
    const std::vector<Binary128Vec3>& value,
    const std::vector<Binary128Vec3>& target) {
    Binary128Accumulator sum;
    for (std::size_t particle = 0U; particle < value.size(); ++particle) {
        const Binary128Vec3 delta = value[particle] - target[particle];
        sum.add(static_cast<Binary128>(0.5) * al_r20_q_dot(delta, delta));
    }
    return sum.value();
}

struct ALR20OracleMetrics {
    bool finite = false;
    bool certified = false;
    std::size_t cycle = 0U;
    Binary128 primal = static_cast<Binary128>(0.0);
    Binary128 dual_mapping = static_cast<Binary128>(0.0);
    Binary128 complementarity = static_cast<Binary128>(0.0);
    Binary128 stationarity = static_cast<Binary128>(0.0);
    Binary128 gap = static_cast<Binary128>(0.0);
    Binary128 gap_bound = static_cast<Binary128>(0.0);
    Binary128 gap_scaled = static_cast<Binary128>(0.0);
    Binary128 objective = static_cast<Binary128>(0.0);
    Binary128 dual = static_cast<Binary128>(0.0);
    std::string primal_root;
    std::string lambda_root;
    std::string root;
};

ALR20OracleMetrics al_r20_q_metrics(const ALR20ProblemData& problem,
    const std::vector<Binary128Vec3>& primal,
    const std::vector<Binary128>& lambda,
    const std::vector<Binary128>& diagonal,
    std::size_t cycle) {
    ALR20OracleMetrics result;
    result.cycle = cycle;
    const std::vector<Binary128Vec3> target = al_r20_q_promote(problem.target);
    std::vector<Binary128> primal_bound;
    const std::vector<Binary128> primal_raw = al_r20_q_action(
        problem.op, primal, problem.workspace.support.constraint, &primal_bound);
    const std::vector<Binary128Vec3> at =
        al_r20_q_transpose(problem.op, lambda, target.size());
    std::vector<Binary128Vec3> z(target.size());
    for (std::size_t particle = 0U; particle < target.size(); ++particle)
        z[particle] = target[particle] - at[particle];
    const std::vector<Binary128Vec3> dual_primal =
        al_r20_q_project_domain(z, problem.lower, problem.upper);
    std::vector<Binary128> dual_bound;
    const std::vector<Binary128> dual_raw = al_r20_q_action(
        problem.op, dual_primal, problem.workspace.support.constraint,
        &dual_bound);
    result.objective = al_r20_q_objective(primal, target);
    const Binary128 dual_objective = al_r20_q_objective(dual_primal, target);
    Binary128Accumulator lambda_raw;
    Binary128 multiplier_scale = static_cast<Binary128>(0.0);
    for (std::size_t row = 0U; row < lambda.size(); ++row) {
        const Binary128 sigma = static_cast<Binary128>(problem.row_scale[row]);
        const Binary128 primal_positive =
            fmaxq(static_cast<Binary128>(0.0), primal_raw[row] + primal_bound[row]) / sigma;
        result.primal = fmaxq(result.primal, primal_positive);
        const Binary128 next = fmaxq(
            static_cast<Binary128>(0.0), lambda[row] + dual_raw[row] / diagonal[row]);
        const Binary128 mapping = diagonal[row] * (lambda[row] - next);
        result.dual_mapping = fmaxq(
            result.dual_mapping, fabsq(mapping) / sigma);
        lambda_raw.add(lambda[row] * dual_raw[row]);
        multiplier_scale += fabsq(lambda[row]) * sigma;
    }
    result.dual = dual_objective + lambda_raw.value();
    const Binary128 energy_scale = fmaxq(
        static_cast<Binary128>(0.5) * static_cast<Binary128>(AL_R20_TRUST_RADIUS)
            * static_cast<Binary128>(AL_R20_TRUST_RADIUS),
        fmaxq(fabsq(result.objective),
            fmaxq(fabsq(result.dual),
                fmaxq(multiplier_scale, ldexpq(static_cast<Binary128>(1.0), -16382)))));
    for (std::size_t row = 0U; row < lambda.size(); ++row)
        result.complementarity = fmaxq(result.complementarity,
            fabsq(lambda[row] * dual_raw[row]) / energy_scale);
    Binary128 primal_scale = static_cast<Binary128>(AL_R20_TRUST_RADIUS);
    Binary128 stationarity_max = static_cast<Binary128>(0.0);
    for (std::size_t particle = 0U; particle < primal.size(); ++particle) {
        stationarity_max = fmaxq(stationarity_max,
            al_r20_q_absmax(primal[particle] - dual_primal[particle]));
        primal_scale = fmaxq(primal_scale,
            al_r20_q_absmax(dual_primal[particle]));
    }
    result.stationarity = stationarity_max / primal_scale;
    result.gap = result.objective - result.dual;
    result.gap_bound = al_r20_q_gamma(
        64U * primal.size() + 32U * lambda.size() + 512U)
        * (fabsq(result.objective) + fabsq(result.dual)
            + multiplier_scale + energy_scale);
    result.gap_scaled = fmaxq(static_cast<Binary128>(0.0), result.gap + result.gap_bound)
        / energy_scale;
    const Binary128 tolerance = ldexpq(static_cast<Binary128>(1.0), -70);
    result.finite = finiteq(result.primal) != 0
        && finiteq(result.dual_mapping) != 0
        && finiteq(result.complementarity) != 0
        && finiteq(result.stationarity) != 0
        && finiteq(result.gap) != 0 && finiteq(result.gap_bound) != 0
        && finiteq(result.gap_scaled) != 0;
    result.certified = result.finite
        && result.primal <= tolerance
        && result.dual_mapping <= tolerance
        && result.complementarity <= tolerance
        && result.stationarity <= tolerance
        && result.gap >= -result.gap_bound
        && result.gap_scaled <= tolerance;
    result.primal_root = al_r20_q_vec_root(primal);
    result.lambda_root = al_r20_q_scalar_root(lambda);
    std::ostringstream material;
    material << cycle << ':' << result.certified << ':'
             << al_binary128_hex(result.primal) << ':'
             << al_binary128_hex(result.dual_mapping) << ':'
             << al_binary128_hex(result.complementarity) << ':'
             << al_binary128_hex(result.stationarity) << ':'
             << al_binary128_hex(result.gap) << ':'
             << al_binary128_hex(result.gap_bound) << ':'
             << al_binary128_hex(result.gap_scaled) << ':'
             << result.primal_root << ':' << result.lambda_root;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20OracleCase {
    bool exact = false;
    bool certified = false;
    std::size_t cycles = 0U;
    std::size_t density_projections = 0U;
    std::size_t box_projections = 0U;
    std::size_t ball_projections = 0U;
    std::size_t coefficient_terms = 0U;
    std::vector<ALR20OracleMetrics> checkpoints;
    ALR20OracleMetrics selected;
    std::string root;
};

ALR20OracleCase al_r20_run_q_oracle(const ALR20ProblemData& problem) {
    ALR20OracleCase result;
    const std::size_t rows = problem.diagonal.size();
    const std::vector<Binary128Vec3> target = al_r20_q_promote(problem.target);
    std::vector<Binary128Vec3> primal = al_r20_q_project_domain(
        target, problem.lower, problem.upper);
    std::vector<Binary128> lambda(rows, static_cast<Binary128>(0.0));
    const std::vector<Binary128> diagonal = al_r20_q_diagonal(problem.op);
    bool exact = diagonal.size() == rows;
    for (Binary128 value : diagonal)
        exact = exact && finiteq(value) != 0 && value > static_cast<Binary128>(0.0);
    if (!exact) return result;

    ALR20OracleMetrics initial =
        al_r20_q_metrics(problem, primal, lambda, diagonal, 0U);
    result.checkpoints.push_back(initial);
    if (initial.certified) {
        result.exact = true;
        result.certified = true;
        result.selected = initial;
        result.root = initial.root;
        return result;
    }

    std::vector<Binary128> density_correction(rows, static_cast<Binary128>(0.0));
    std::vector<Binary128Vec3> box_correction(primal.size());
    std::vector<Binary128Vec3> ball_correction(primal.size());
    const Binary128 radius = static_cast<Binary128>(AL_R20_TRUST_RADIUS);
    std::size_t checkpoint_index = 0U;
    for (std::size_t cycle = 1U;
         cycle <= AL_R20_ORACLE_CHECKPOINTS.back() && exact; ++cycle) {
        for (std::size_t row = 0U; row < rows; ++row) {
            const Binary128 correction = density_correction[row];
            for (std::size_t index = problem.op.entry_offsets[row];
                 index < problem.op.entry_offsets[row + 1U]; ++index) {
                const ALR64Entry& entry = problem.op.entries[index];
                primal[entry.particle] = al_r20_q_add(
                    primal[entry.particle],
                    al_r20_q_scale(correction,
                        al_binary128_vec3(entry.coefficient)));
                ++result.coefficient_terms;
            }
            Binary128Accumulator residual;
            residual.add(static_cast<Binary128>(
                problem.workspace.support.constraint[row]));
            for (std::size_t index = problem.op.entry_offsets[row];
                 index < problem.op.entry_offsets[row + 1U]; ++index) {
                const ALR64Entry& entry = problem.op.entries[index];
                residual.add(al_r20_q_dot(
                    al_binary128_vec3(entry.coefficient),
                    primal[entry.particle]));
                ++result.coefficient_terms;
            }
            const Binary128 next = fmaxq(static_cast<Binary128>(0.0),
                residual.value() / diagonal[row]);
            for (std::size_t index = problem.op.entry_offsets[row];
                 index < problem.op.entry_offsets[row + 1U]; ++index) {
                const ALR64Entry& entry = problem.op.entries[index];
                primal[entry.particle] = primal[entry.particle]
                    - al_r20_q_scale(next,
                        al_binary128_vec3(entry.coefficient));
                ++result.coefficient_terms;
            }
            density_correction[row] = next;
            ++result.density_projections;
        }

        for (std::size_t particle = 0U; particle < primal.size(); ++particle) {
            const Binary128Vec3 input = al_r20_q_add(
                primal[particle], box_correction[particle]);
            Binary128Vec3 projected;
            for (int axis = 0; axis < 3; ++axis)
                al_r20_q_set_component(&projected, axis,
                    al_r20_q_clamp(al_r20_q_component(input, axis),
                        static_cast<Binary128>(al_r41_component(
                            problem.lower[particle], axis)),
                        static_cast<Binary128>(al_r41_component(
                            problem.upper[particle], axis))));
            box_correction[particle] = input - projected;
            primal[particle] = projected;
        }
        ++result.box_projections;

        Binary128 norm_squared = static_cast<Binary128>(0.0);
        std::vector<Binary128Vec3> ball_input(primal.size());
        for (std::size_t particle = 0U; particle < primal.size(); ++particle) {
            ball_input[particle] = al_r20_q_add(
                primal[particle], ball_correction[particle]);
            norm_squared += al_r20_q_dot(
                ball_input[particle], ball_input[particle]);
        }
        const Binary128 scale = norm_squared > radius * radius
            ? radius / sqrtq(norm_squared) : static_cast<Binary128>(1.0);
        for (std::size_t particle = 0U; particle < primal.size(); ++particle) {
            const Binary128Vec3 projected =
                al_r20_q_scale(scale, ball_input[particle]);
            ball_correction[particle] = ball_input[particle] - projected;
            primal[particle] = projected;
            exact = exact && finiteq(primal[particle].x) != 0
                && finiteq(primal[particle].y) != 0
                && finiteq(primal[particle].z) != 0;
        }
        ++result.ball_projections;
        result.cycles = cycle;

        if (checkpoint_index < AL_R20_ORACLE_CHECKPOINTS.size()
            && cycle == AL_R20_ORACLE_CHECKPOINTS[checkpoint_index]) {
            ALR20OracleMetrics metrics = al_r20_q_metrics(
                problem, primal, density_correction, diagonal, cycle);
            exact = exact && metrics.finite;
            result.checkpoints.push_back(metrics);
            if (metrics.certified) {
                result.certified = true;
                result.selected = metrics;
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
             << result.cycles << ':' << result.density_projections << ':'
             << result.box_projections << ':' << result.ball_projections
             << ':' << result.coefficient_terms << '|';
    for (const ALR20OracleMetrics& metrics : result.checkpoints)
        material << metrics.root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_binary128_oracle_controls() {
#if defined(__linux__) && defined(__x86_64__) && defined(__GNUC__) \
    && defined(__SIZEOF_FLOAT128__)
    constexpr bool frozen_platform = true;
#else
    constexpr bool frozen_platform = false;
#endif
    const bool profile_exact = frozen_platform
        && sizeof(Binary128) == 16U && FLT_RADIX == 2
        && FLT128_MANT_DIG == 113;
    std::vector<ALR20ManifestCase> cases = al_r20_manifest_cases();
    const bool parent_exact = profile_exact
        && al_r20_manifest_semantic_root(cases) == AL_R20_MANIFEST_SHA256;
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    std::vector<ALR20ProblemData> problems;
    std::vector<ALR20OracleCase> oracle;
    bool preflight_exact = parent_exact
        && cases.size() == AL_R20_PROBLEM_SHA256.size();
    bool exact = preflight_exact;
    bool all_certified = true;
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        const ALR20ManifestCase& source = cases[index];
        ALR20ProblemData problem = al_r20_materialize_problem(
            source, &sparse_work, &static_work, &adjacency_work);
        const bool problem_exact = index < AL_R20_PROBLEM_SHA256.size()
            && problem.problem_root == AL_R20_PROBLEM_SHA256[index];
        preflight_exact = preflight_exact && problem_exact;
        exact = exact && problem.exact;
        ALR20OracleCase result = problem.exact && problem_exact
            ? al_r20_run_q_oracle(problem) : ALR20OracleCase{};
        exact = exact && result.exact;
        all_certified = all_certified && result.certified;
        oracle.push_back(std::move(result));
        problems.push_back(std::move(problem));
    }
    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    exact = exact && sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    exact = exact && preflight_exact;
    const std::string route = !parent_exact || !preflight_exact
        ? "GENERALIZATION_PARENT_REJECTED"
        : (!exact ? "GENERALIZATION_ORACLE_REJECTED"
                  : (all_certified ? "BINARY128_ORACLE_CERTIFIED"
                                   : "ORACLE_UNRESOLVED"));
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20-binary128-oracle.v1|"
             << AL_R20_PREFLIGHT_SHA256 << ':' << route << '|';
    for (const ALR20OracleCase& value : oracle)
        semantic << value.root << ';';
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20_binary128_oracle.v1\""
           << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
           << ",\"route\":\"" << route << "\""
           << ",\"parent_manifest_sha256\":\""
           << AL_R20_MANIFEST_SHA256 << "\",\"parent_preflight_sha256\":\""
           << AL_R20_PREFLIGHT_SHA256 << "\",\"profile_exact\":"
           << (profile_exact ? "true" : "false")
           << ",\"preflight_exact\":"
           << (preflight_exact ? "true" : "false")
           << ",\"all_certified\":" << (all_certified ? "true" : "false")
           << ",\"cases\":[";
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20OracleCase& value = oracle[index];
        const ALR20OracleMetrics& selected = value.selected;
        report << "{\"name\":\"" << cases[index].fixture.name
               << "\",\"role\":\"" << cases[index].role
               << "\",\"certified\":"
               << (value.certified ? "true" : "false")
               << ",\"selected_cycle\":" << selected.cycle
               << ",\"cycles\":" << value.cycles
               << ",\"checkpoint_count\":" << value.checkpoints.size()
               << ",\"primal\":\"" << al_binary128_text(selected.primal)
               << "\",\"dual_mapping\":\""
               << al_binary128_text(selected.dual_mapping)
               << "\",\"complementarity\":\""
               << al_binary128_text(selected.complementarity)
               << "\",\"stationarity\":\""
               << al_binary128_text(selected.stationarity)
               << "\",\"gap\":\"" << al_binary128_text(selected.gap)
               << "\",\"gap_bound\":\""
               << al_binary128_text(selected.gap_bound)
               << "\",\"gap_scaled\":\""
               << al_binary128_text(selected.gap_scaled)
               << "\",\"objective\":\""
               << al_binary128_text(selected.objective)
               << "\",\"dual\":\"" << al_binary128_text(selected.dual)
               << "\",\"work\":{\"density_projections\":"
               << value.density_projections << ",\"box_projections\":"
               << value.box_projections << ",\"ball_projections\":"
               << value.ball_projections << ",\"coefficient_terms\":"
               << value.coefficient_terms << "},\"root\":\""
               << value.root << "\"}";
    }
    report << "]"
              ",\"candidate_iterations\":0,\"timing_admitted\":false"
              ",\"runtime_binary128_authorized\":false"
              ",\"runtime_authority\":false,\"production_authority\":false"
              ",\"result_sha256\":\""
           << result_sha << "\"}";
    return {exact, report.str()};
}
