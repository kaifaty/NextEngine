namespace {

constexpr const char* B4E2D7R19R30_IDENTITY_SHA256 =
    "c1e548cc5dba9f4467ee932fa4596e787a11b43d8ba6836c2ab6101ee967a0eb";
constexpr const char* B4E2D7R19R30_IDENTITY_PROJECTION =
    R"IDENTITY(nextengine.nonlocal.nsr3b4e2d7r19r30-linearized-range-projection|v1|parent=f3185b23:596c81d4979bed80181d65429e5d253a6464d61f39ec76d8330fe0ad464dfe7b:f6736b14927c8daedf751649a32cdf3f5ac527116b3037c9367ed921c369c835|source=state-851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7;position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;dual-cd0100c30eb55ad7f91f1b94c00ee4281ff3c75e90ca287be038e87ab7a5215d;topology-070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5;violated-bd06f7b7c5e1f1943e5c269bbfb192229504091381b8f20f0f4988eecb021c70;active-b84e502736ee7eed3239fe4dfcd86ec92c54dbffb0ce78854087318275d57029;boundary-25ed05cd3b2703de3f2434c825dc4d8894fcab76e824298421f0416e36df32fc;interior-72515617c229e4db8ef84aaa96455c3ba8a4f7cdd2060315e1aff5d406d94f2d;particles6000;pairs340340;max-degree113;violated1420|problem=B=restrict-violated(SPACING*Jc);b=-restrict-violated(c);min||Bv-b||2;support-fixed;no-row-scaling|scaling=exact-scalar-column-l2;D=reciprocal-nonzero;zero-columns-derived;C=BD;range-preserved|solver=LSMR;golub-kahan;lambda0;y0zero;max-iterations512;atol1e-10;btol1e-10;conlim1e12;reorth-none|direct-stop=compatible-or-least-squares;fresh-Cy;fresh-Ct-residual;orthogonality<=1e-10;pythagorean-relative<=1e-10|controls=dense-identity;dense-underdetermined-compatible;dense-overdetermined-inconsistent;dense-rank-deficient-inconsistent;parent;source;operator;partition;column-scaling;lsmr;direct-residual;orthogonality;work;rollback|observations=rhs-root;column-root;iterate-root;projection-root;residual-root;normal-root;residual-ratio;normal-ratio;condition-estimate;boundary-interior-decomposition;full-row-response;new-positive-count;dimensionless-preimage-rms-max;no-physical-threshold|routes=range-projection-parent-rejected;range-projection-source-rejected;range-projection-operator-rejected;range-projection-partition-rejected;range-projection-dense-control-rejected;range-projection-scaling-rejected;range-projection-not-converged;range-projection-residual-rejected;range-projection-orthogonality-rejected;range-projection-work-rejected;linearized-range-projection-candidate|precedence=parent,source,operator,partition,dense-control,scaling,convergence,residual,orthogonality,work,candidate|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;diagnostic-workspaces1;column-passes1;initial-adjoint-passes1;iteration-passes<=1024;final-passes2;total-pair-passes<=1028;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0|correction-application=none;nonlinear-evaluation=none;floor-classification=none;state-mutation=none;following-outer=none;substep=none;macro=none;trajectory=none;timing=none;public-schema=none;runtime=none;production=none|credit=one-private-linearized-range-projection-candidate-only)IDENTITY";

constexpr double AL_R30_ATOL = 1.0e-10;
constexpr double AL_R30_BTOL = 1.0e-10;
constexpr double AL_R30_CONLIM = 1.0e12;
constexpr std::size_t AL_R30_MAX_ITERATIONS = 512U;

struct ALR30Rotation {
    double c = 1.0;
    double s = 0.0;
    double r = 0.0;
};

struct ALR30LSMRResult {
    bool finite = false;
    std::vector<double> value;
    int stop = 0;
    std::size_t iterations = 0U;
    std::size_t forward_calls = 0U;
    std::size_t adjoint_calls = 0U;
    std::size_t estimated_normal_increases = 0U;
    double residual_norm = 0.0;
    double normal_norm = 0.0;
    double operator_norm = 0.0;
    double condition_estimate = 1.0;
    double iterate_norm = 0.0;
};

struct ALR30DenseControl {
    std::string name;
    bool passed = false;
    int stop = 0;
    std::size_t iterations = 0U;
    double solution_error = 0.0;
    double projection_error = 0.0;
    double residual_error = 0.0;
    double normal_error = 0.0;
    std::string root;
};

struct ALR30ColumnScaling {
    bool passed = false;
    std::vector<double> norm;
    std::vector<double> reciprocal;
    std::size_t nonzero_columns = 0U;
    std::size_t zero_columns = 0U;
    double scaled_frobenius = 0.0;
    std::string norm_root;
    std::string reciprocal_root;
    std::string root;
};

double al_r30_norm(const std::vector<double>& values) {
    long double sum = 0.0L;
    for (double value : values) {
        sum += static_cast<long double>(value)
            * static_cast<long double>(value);
    }
    return std::sqrt(static_cast<double>(sum));
}

long double al_r30_dot(
    const std::vector<double>& lhs,
    const std::vector<double>& rhs) {
    if (lhs.size() != rhs.size()) {
        return std::numeric_limits<long double>::quiet_NaN();
    }
    long double result = 0.0L;
    for (std::size_t index = 0U; index < lhs.size(); ++index) {
        result += static_cast<long double>(lhs[index])
            * static_cast<long double>(rhs[index]);
    }
    return result;
}

bool al_r30_finite(const std::vector<double>& values) {
    return std::all_of(values.begin(), values.end(),
        [](double value) { return std::isfinite(value); });
}

double al_r30_relative_vector_error(
    const std::vector<double>& value,
    const std::vector<double>& expected) {
    if (value.size() != expected.size()) {
        return std::numeric_limits<double>::infinity();
    }
    std::vector<double> difference(value.size());
    for (std::size_t index = 0U; index < value.size(); ++index) {
        difference[index] = value[index] - expected[index];
    }
    return al_r30_norm(difference)
        / std::max({al_r30_norm(expected), 1.0});
}

ALR30Rotation al_r30_sym_ortho(double a, double b) {
    if (b == 0.0) {
        return {a == 0.0 ? 1.0 : std::copysign(1.0, a), 0.0,
            std::abs(a)};
    }
    if (a == 0.0) {
        return {0.0, std::copysign(1.0, b), std::abs(b)};
    }
    if (std::abs(b) > std::abs(a)) {
        const double tau = a / b;
        const double s = std::copysign(1.0, b)
            / std::sqrt(1.0 + tau * tau);
        return {s * tau, s, b / s};
    }
    const double tau = b / a;
    const double c = std::copysign(1.0, a)
        / std::sqrt(1.0 + tau * tau);
    return {c, c * tau, a / c};
}

template <typename Forward, typename Adjoint>
ALR30LSMRResult al_r30_lsmr(
    const std::vector<double>& rhs,
    std::size_t column_count,
    Forward&& forward,
    Adjoint&& adjoint,
    double atol = AL_R30_ATOL,
    double btol = AL_R30_BTOL,
    double conlim = AL_R30_CONLIM,
    std::size_t maximum_iterations = AL_R30_MAX_ITERATIONS) {
    ALR30LSMRResult result;
    result.value.assign(column_count, 0.0);
    if (rhs.empty() || column_count == 0U || !al_r30_finite(rhs)) {
        result.stop = 9;
        return result;
    }

    auto apply_forward = [&](const std::vector<double>& value) {
        ++result.forward_calls;
        return forward(value);
    };
    auto apply_adjoint = [&](const std::vector<double>& value) {
        ++result.adjoint_calls;
        return adjoint(value);
    };

    std::vector<double> u = rhs;
    double beta = al_r30_norm(u);
    std::vector<double> v(column_count, 0.0);
    double alpha = 0.0;
    if (beta > 0.0) {
        for (double& value : u) value /= beta;
        v = apply_adjoint(u);
        if (v.size() != column_count || !al_r30_finite(v)) {
            result.stop = 9;
            return result;
        }
        alpha = al_r30_norm(v);
        if (alpha > 0.0) {
            for (double& value : v) value /= alpha;
        }
    }

    double zetabar = alpha * beta;
    double alphabar = alpha;
    double rho = 1.0;
    double rhobar = 1.0;
    double cbar = 1.0;
    double sbar = 0.0;
    std::vector<double> h = v;
    std::vector<double> hbar(column_count, 0.0);

    double betadd = beta;
    double betad = 0.0;
    double rhodold = 1.0;
    double tautildeold = 0.0;
    double thetatilde = 0.0;
    double zeta = 0.0;
    double residual_accumulator = 0.0;

    double operator_norm_squared = alpha * alpha;
    double maximum_rhobar = 0.0;
    double minimum_rhobar = std::numeric_limits<double>::infinity();
    result.operator_norm = std::sqrt(operator_norm_squared);
    result.condition_estimate = 1.0;
    result.residual_norm = beta;
    result.normal_norm = alpha * beta;
    const double norm_rhs = beta;
    const double condition_tolerance = conlim > 0.0 ? 1.0 / conlim : 0.0;
    double previous_normal = result.normal_norm;

    if (result.normal_norm == 0.0) {
        result.finite = true;
        return result;
    }

    for (std::size_t iteration = 1U;
         iteration <= maximum_iterations; ++iteration) {
        result.iterations = iteration;
        std::vector<double> next_u = apply_forward(v);
        if (next_u.size() != rhs.size() || !al_r30_finite(next_u)) {
            result.stop = 9;
            break;
        }
        for (std::size_t index = 0U; index < next_u.size(); ++index) {
            next_u[index] -= alpha * u[index];
        }
        u = std::move(next_u);
        beta = al_r30_norm(u);
        const bool beta_breakdown = beta == 0.0;
        if (beta > 0.0) {
            for (double& value : u) value /= beta;
            std::vector<double> next_v = apply_adjoint(u);
            if (next_v.size() != column_count || !al_r30_finite(next_v)) {
                result.stop = 9;
                break;
            }
            for (std::size_t index = 0U; index < next_v.size(); ++index) {
                next_v[index] -= beta * v[index];
            }
            v = std::move(next_v);
            alpha = al_r30_norm(v);
            if (alpha > 0.0) {
                for (double& value : v) value /= alpha;
            }
        } else {
            alpha = 0.0;
            std::fill(v.begin(), v.end(), 0.0);
        }
        const bool alpha_breakdown = alpha == 0.0;

        const ALR30Rotation damp_rotation = al_r30_sym_ortho(alphabar, 0.0);
        const double rho_old = rho;
        const ALR30Rotation primary_rotation =
            al_r30_sym_ortho(damp_rotation.r, beta);
        rho = primary_rotation.r;
        const double theta_new = primary_rotation.s * alpha;
        alphabar = primary_rotation.c * alpha;

        const double rhobar_old = rhobar;
        const double zeta_old = zeta;
        const double theta_bar = sbar * rho;
        const double rho_temp = cbar * rho;
        const ALR30Rotation secondary_rotation =
            al_r30_sym_ortho(rho_temp, theta_new);
        cbar = secondary_rotation.c;
        sbar = secondary_rotation.s;
        rhobar = secondary_rotation.r;
        zeta = cbar * zetabar;
        zetabar = -sbar * zetabar;

        const double hbar_denominator = rho_old * rhobar_old;
        const double value_denominator = rho * rhobar;
        if (hbar_denominator == 0.0 || value_denominator == 0.0
            || !std::isfinite(hbar_denominator)
            || !std::isfinite(value_denominator)) {
            result.stop = 9;
            break;
        }
        const double hbar_scale = theta_bar * rho / hbar_denominator;
        for (std::size_t index = 0U; index < column_count; ++index) {
            hbar[index] = h[index] - hbar_scale * hbar[index];
            result.value[index] += zeta / value_denominator * hbar[index];
            h[index] = v[index] - theta_new / rho * h[index];
        }

        const double beta_acute = damp_rotation.c * betadd;
        const double beta_check = -damp_rotation.s * betadd;
        const double beta_hat = primary_rotation.c * beta_acute;
        betadd = -primary_rotation.s * beta_acute;
        const double theta_tilde_old = thetatilde;
        const ALR30Rotation residual_rotation =
            al_r30_sym_ortho(rhodold, theta_bar);
        thetatilde = residual_rotation.s * rhobar;
        rhodold = residual_rotation.c * rhobar;
        betad = -residual_rotation.s * betad
            + residual_rotation.c * beta_hat;
        if (residual_rotation.r == 0.0 || rhodold == 0.0) {
            result.stop = 9;
            break;
        }
        tautildeold = (zeta_old - theta_tilde_old * tautildeold)
            / residual_rotation.r;
        const double tau_d = (zeta - thetatilde * tautildeold) / rhodold;
        residual_accumulator += beta_check * beta_check;
        result.residual_norm = std::sqrt(std::max(0.0,
            residual_accumulator + (betad - tau_d) * (betad - tau_d)
                + betadd * betadd));

        operator_norm_squared += beta * beta;
        result.operator_norm = std::sqrt(operator_norm_squared);
        operator_norm_squared += alpha * alpha;
        maximum_rhobar = std::max(maximum_rhobar, std::abs(rhobar_old));
        if (iteration > 1U) {
            minimum_rhobar = std::min(
                minimum_rhobar, std::abs(rhobar_old));
        }
        const double condition_denominator = std::min(
            minimum_rhobar, std::abs(rho_temp));
        result.condition_estimate = condition_denominator > 0.0
            ? std::max(maximum_rhobar, std::abs(rho_temp))
                / condition_denominator
            : std::numeric_limits<double>::infinity();
        result.normal_norm = std::abs(zetabar);
        result.iterate_norm = al_r30_norm(result.value);

        const double monotone_slack = 256.0
            * std::numeric_limits<double>::epsilon()
            * std::max(previous_normal, 1.0e-300);
        if (result.normal_norm > previous_normal + monotone_slack) {
            ++result.estimated_normal_increases;
        }
        previous_normal = result.normal_norm;

        const double test1 = norm_rhs > 0.0
            ? result.residual_norm / norm_rhs : 0.0;
        const double normal_denominator =
            result.operator_norm * result.residual_norm;
        const double test2 = normal_denominator > 0.0
            ? result.normal_norm / normal_denominator
            : (result.normal_norm == 0.0 ? 0.0
                : std::numeric_limits<double>::infinity());
        const double test3 = result.condition_estimate > 0.0
            ? 1.0 / result.condition_estimate : 0.0;
        const double normalized_compatible = norm_rhs > 0.0
            ? test1 / (1.0 + result.operator_norm
                * result.iterate_norm / norm_rhs)
            : 0.0;
        const double requested_compatible = norm_rhs > 0.0
            ? btol + atol * result.operator_norm
                * result.iterate_norm / norm_rhs
            : btol;

        int stop = 0;
        if (iteration >= maximum_iterations) stop = 7;
        if (1.0 + test3 <= 1.0) stop = 6;
        if (1.0 + test2 <= 1.0) stop = 5;
        if (1.0 + normalized_compatible <= 1.0) stop = 4;
        if (test3 <= condition_tolerance) stop = 3;
        if (test2 <= atol) stop = 2;
        if (test1 <= requested_compatible) stop = 1;
        if (stop == 0 && (beta_breakdown || alpha_breakdown)) stop = 8;
        result.stop = stop;
        if (stop != 0) break;
    }

    result.finite = al_r30_finite(result.value)
        && std::isfinite(result.residual_norm)
        && std::isfinite(result.normal_norm)
        && std::isfinite(result.operator_norm)
        && std::isfinite(result.condition_estimate)
        && std::isfinite(result.iterate_norm)
        && result.stop != 9;
    return result;
}

std::vector<double> al_r30_dense_forward(
    const std::vector<double>& matrix,
    std::size_t rows,
    std::size_t columns,
    const std::vector<double>& value) {
    if (matrix.size() != rows * columns || value.size() != columns) return {};
    std::vector<double> result(rows, 0.0);
    for (std::size_t row = 0U; row < rows; ++row) {
        long double sum = 0.0L;
        for (std::size_t column = 0U; column < columns; ++column) {
            sum += static_cast<long double>(matrix[row * columns + column])
                * static_cast<long double>(value[column]);
        }
        result[row] = static_cast<double>(sum);
    }
    return result;
}

std::vector<double> al_r30_dense_adjoint(
    const std::vector<double>& matrix,
    std::size_t rows,
    std::size_t columns,
    const std::vector<double>& value) {
    if (matrix.size() != rows * columns || value.size() != rows) return {};
    std::vector<double> result(columns, 0.0);
    for (std::size_t column = 0U; column < columns; ++column) {
        long double sum = 0.0L;
        for (std::size_t row = 0U; row < rows; ++row) {
            sum += static_cast<long double>(matrix[row * columns + column])
                * static_cast<long double>(value[row]);
        }
        result[column] = static_cast<double>(sum);
    }
    return result;
}

ALR30DenseControl al_r30_dense_control(
    std::string name,
    std::size_t rows,
    std::size_t columns,
    std::vector<double> matrix,
    std::vector<double> rhs,
    std::vector<double> expected_solution,
    std::vector<double> expected_projection,
    std::vector<double> expected_residual) {
    ALR30DenseControl result;
    result.name = std::move(name);
    const auto forward = [&](const std::vector<double>& value) {
        return al_r30_dense_forward(matrix, rows, columns, value);
    };
    const auto adjoint = [&](const std::vector<double>& value) {
        return al_r30_dense_adjoint(matrix, rows, columns, value);
    };
    const ALR30LSMRResult solve = al_r30_lsmr(
        rhs, columns, forward, adjoint);
    const std::vector<double> projection = forward(solve.value);
    std::vector<double> residual(rows, 0.0);
    if (projection.size() == rows) {
        for (std::size_t row = 0U; row < rows; ++row) {
            residual[row] = rhs[row] - projection[row];
        }
    }
    const std::vector<double> normal = adjoint(residual);
    result.stop = solve.stop;
    result.iterations = solve.iterations;
    result.solution_error = al_r30_relative_vector_error(
        solve.value, expected_solution);
    result.projection_error = al_r30_relative_vector_error(
        projection, expected_projection);
    result.residual_error = al_r30_relative_vector_error(
        residual, expected_residual);
    result.normal_error = al_r30_norm(normal)
        / std::max({al_r30_norm(rhs), 1.0});
    result.passed = solve.finite && projection.size() == rows
        && normal.size() == columns
        && result.solution_error <= 1.0e-12
        && result.projection_error <= 1.0e-12
        && result.residual_error <= 1.0e-12
        && result.normal_error <= 1.0e-12;
    std::ostringstream projection_text;
    projection_text << result.name << ':' << result.passed << ':'
                    << result.stop << ':' << result.iterations << ':'
                    << binary64_bits(result.solution_error) << ':'
                    << binary64_bits(result.projection_error) << ':'
                    << binary64_bits(result.residual_error) << ':'
                    << binary64_bits(result.normal_error) << ':'
                    << al_r10_dual_root(solve.value) << ':'
                    << al_r10_dual_root(projection) << ':'
                    << al_r10_dual_root(residual) << ':'
                    << al_r10_dual_root(normal);
    result.root = sha256_hex(projection_text.str());
    return result;
}

std::array<ALR30DenseControl, 4U> al_r30_dense_controls() {
    return {{
        al_r30_dense_control(
            "identity", 3U, 3U,
            {1.0,0.0,0.0, 0.0,1.0,0.0, 0.0,0.0,1.0},
            {1.0,-2.0,0.5}, {1.0,-2.0,0.5},
            {1.0,-2.0,0.5}, {0.0,0.0,0.0}),
        al_r30_dense_control(
            "underdetermined-compatible", 2U, 3U,
            {1.0,0.0,1.0, 0.0,1.0,1.0},
            {1.0,2.0}, {0.0,1.0,1.0},
            {1.0,2.0}, {0.0,0.0}),
        al_r30_dense_control(
            "overdetermined-inconsistent", 3U, 2U,
            {1.0,0.0, 0.0,1.0, 1.0,1.0},
            {1.0,2.0,4.0}, {4.0/3.0,7.0/3.0},
            {4.0/3.0,7.0/3.0,11.0/3.0},
            {-1.0/3.0,-1.0/3.0,1.0/3.0}),
        al_r30_dense_control(
            "rank-deficient-inconsistent", 3U, 2U,
            {1.0,2.0, 2.0,4.0, 3.0,6.0},
            {1.0,0.0,1.0}, {2.0/35.0,4.0/35.0},
            {2.0/7.0,4.0/7.0,6.0/7.0},
            {5.0/7.0,-4.0/7.0,1.0/7.0}),
    }};
}

void al_r30_add_column_square(
    std::vector<long double>& squared,
    std::size_t particle,
    Vec3 value) {
    const std::size_t base = 3U * particle;
    squared[base] += static_cast<long double>(value.x) * value.x;
    squared[base + 1U] += static_cast<long double>(value.y) * value.y;
    squared[base + 2U] += static_cast<long double>(value.z) * value.z;
}

ALR30ColumnScaling al_r30_column_scaling(
    const ALNormalizedWorkspace& workspace,
    const std::vector<std::uint8_t>& selected) {
    ALR30ColumnScaling result;
    const std::size_t count = workspace.neighborhood.fluid.size();
    if (!workspace.passed || selected.size() != count) return result;
    std::vector<long double> squared(3U * count, 0.0L);
    std::vector<Vec3> center(count);
    for (std::size_t pair_index = 0U;
         pair_index < workspace.neighborhood.pairs.size(); ++pair_index) {
        const JointPair pair = workspace.neighborhood.pairs[pair_index];
        const Vec3 jacobian = SPACING
            * al_r29_pair_jacobian(workspace, pair_index);
        if (selected[pair.fluid]) {
            center[pair.fluid] += jacobian;
            if (pair.participant < count) {
                al_r30_add_column_square(
                    squared, pair.participant, -jacobian);
            }
        }
        if (pair.participant < count && selected[pair.participant]) {
            center[pair.participant] += -jacobian;
            al_r30_add_column_square(squared, pair.fluid, jacobian);
        }
    }
    for (std::size_t particle = 0U; particle < count; ++particle) {
        if (selected[particle]) {
            al_r30_add_column_square(squared, particle, center[particle]);
        }
    }
    result.norm.resize(squared.size());
    result.reciprocal.resize(squared.size());
    result.passed = true;
    for (std::size_t index = 0U; index < squared.size(); ++index) {
        result.norm[index] = static_cast<double>(std::sqrt(squared[index]));
        if (result.norm[index] > 0.0) {
            result.reciprocal[index] = 1.0 / result.norm[index];
            ++result.nonzero_columns;
            result.passed = result.passed
                && std::isfinite(result.reciprocal[index])
                && result.reciprocal[index] > 0.0;
        } else {
            result.reciprocal[index] = 0.0;
            ++result.zero_columns;
        }
    }
    result.passed = result.passed && result.nonzero_columns > 0U
        && result.nonzero_columns + result.zero_columns == squared.size();
    result.scaled_frobenius = std::sqrt(
        static_cast<double>(result.nonzero_columns));
    result.norm_root = al_r10_dual_root(result.norm);
    result.reciprocal_root = al_r10_dual_root(result.reciprocal);
    result.root = sha256_hex(result.norm_root + ':' + result.reciprocal_root
        + ':' + std::to_string(result.nonzero_columns) + ':'
        + std::to_string(result.zero_columns));
    return result;
}

std::vector<Vec3> al_r30_unflatten_scaled(
    const std::vector<double>& value,
    const std::vector<double>& reciprocal) {
    if (value.size() != reciprocal.size() || value.size() % 3U != 0U) {
        return {};
    }
    std::vector<Vec3> result(value.size() / 3U);
    for (std::size_t particle = 0U; particle < result.size(); ++particle) {
        result[particle] = {
            reciprocal[3U * particle] * value[3U * particle],
            reciprocal[3U * particle + 1U] * value[3U * particle + 1U],
            reciprocal[3U * particle + 2U] * value[3U * particle + 2U]};
    }
    return result;
}

std::vector<double> al_r30_flatten_scaled(
    const std::vector<Vec3>& value,
    const std::vector<double>& reciprocal) {
    if (3U * value.size() != reciprocal.size()) return {};
    std::vector<double> result(reciprocal.size());
    for (std::size_t particle = 0U; particle < value.size(); ++particle) {
        result[3U * particle] = reciprocal[3U * particle] * value[particle].x;
        result[3U * particle + 1U] = reciprocal[3U * particle + 1U]
            * value[particle].y;
        result[3U * particle + 2U] = reciprocal[3U * particle + 2U]
            * value[particle].z;
    }
    return result;
}

double al_r30_masked_norm(
    const std::vector<double>& value,
    const std::vector<std::uint8_t>& mask,
    const std::vector<std::size_t>& rows) {
    if (value.size() != rows.size()) {
        return std::numeric_limits<double>::infinity();
    }
    long double sum = 0.0L;
    for (std::size_t index = 0U; index < rows.size(); ++index) {
        if (mask[rows[index]]) {
            sum += static_cast<long double>(value[index]) * value[index];
        }
    }
    return std::sqrt(static_cast<double>(sum));
}

const char* al_r30_stop_name(int stop) {
    switch (stop) {
    case 0: return "ZERO_SOLUTION";
    case 1: return "COMPATIBLE";
    case 2: return "LEAST_SQUARES";
    case 3: return "CONDITION_LIMIT";
    case 4: return "MACHINE_COMPATIBLE";
    case 5: return "MACHINE_LEAST_SQUARES";
    case 6: return "MACHINE_CONDITION";
    case 7: return "ITERATION_LIMIT";
    case 8: return "BIDIAGONAL_BREAKDOWN";
    default: return "INVALID";
    }
}

std::string al_r30_route(
    bool parent, bool source, bool operator_exact, bool partition,
    bool dense, bool scaling, bool convergence, bool residual,
    bool orthogonality, bool work) {
    if (!parent) return "RANGE_PROJECTION_PARENT_REJECTED";
    if (!source) return "RANGE_PROJECTION_SOURCE_REJECTED";
    if (!operator_exact) return "RANGE_PROJECTION_OPERATOR_REJECTED";
    if (!partition) return "RANGE_PROJECTION_PARTITION_REJECTED";
    if (!dense) return "RANGE_PROJECTION_DENSE_CONTROL_REJECTED";
    if (!scaling) return "RANGE_PROJECTION_SCALING_REJECTED";
    if (!convergence) return "RANGE_PROJECTION_NOT_CONVERGED";
    if (!residual) return "RANGE_PROJECTION_RESIDUAL_REJECTED";
    if (!orthogonality) return "RANGE_PROJECTION_ORTHOGONALITY_REJECTED";
    if (!work) return "RANGE_PROJECTION_WORK_REJECTED";
    return "LINEARIZED_RANGE_PROJECTION_CANDIDATE";
}

struct ALR30RangeProjectionCapture {
    bool captured = false;
    std::string state_root;
    std::vector<Vec3> position;
    std::vector<Vec3> predicted_position;
    std::vector<double> dual;
    double theta = 0.0;
    std::vector<double> constraint;
    std::vector<std::uint8_t> violated;
    std::vector<Vec3> preimage;
    std::vector<double> full_response;
    std::string topology_root;
    std::string violated_root;
    std::string rhs_root;
    std::string preimage_root;
    std::string response_root;
    double preimage_rms = 0.0;
    double preimage_maximum = 0.0;
};

} // namespace

SplitBoundaryReport run_al_linearized_range_projection_controls_impl(
    ALR30RangeProjectionCapture* output_capture) {
    const bool identity_exact = sha256_hex(B4E2D7R19R30_IDENTITY_PROJECTION)
        == B4E2D7R19R30_IDENTITY_SHA256;
    ALShadowOuter11ExecutionCapture capture;
    const SplitBoundaryReport parent =
        run_al_linearized_feasibility_operator_controls_impl(&capture);
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const bool parent_semantic = parent.json.find(
        "\"result_sha256\":\""
        "f6736b14927c8daedf751649a32cdf3f5ac527116b3037c9367ed921c369c835"
        "\"") != std::string::npos;
    const bool parent_exact = parent.passed && capture.captured
        && parent_semantic && parent_stdout
            == "596c81d4979bed80181d65429e5d253a6464d61f39ec76d8330fe0ad464dfe7b";

    const std::vector<Vec3>& position = capture.candidate.update.position;
    const std::vector<double>& dual = capture.candidate.update.u;
    const auto& r26 = capture.r27.r26;
    const auto& r25 = r26.r25;
    const auto& r24 = r25.r24;
    const auto& r23 = r24.r23;
    const auto& r22 = r23.r22;
    const auto& r21 = r22.r21;
    const auto& r20 = r21.r20;
    const auto& r19 = r20.r19;
    const auto& r18 = r19.r18;
    const auto& r17 = r18.r17;
    const ALV2EnvelopeCapture& r16 = r17.r16;
    const ALSoftCapSuspensionCapture& r14 = r16.r15.r14;
    const ALNormalizedTotalHvpBoundaryCapture& boundary =
        r14.r13.r12.r11.r10.total_hvp_boundary;
    const bool source_exact = capture.state_root
            == "851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7"
        && al_binary64_vec3_root(position)
            == "e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828"
        && al_r10_dual_root(dual)
            == "cd0100c30eb55ad7f91f1b94c00ee4281ff3c75e90ca287be038e87ab7a5215d"
        && position.size() == 6000U && dual.size() == position.size()
        && binary64_bits(boundary.theta) == 0x3fc5cccccccccccdULL;
    const std::string position_before = al_binary64_vec3_root(position);
    const std::string dual_before = al_r10_dual_root(dual);
    const std::string constraint_before = al_r10_dual_root(
        capture.candidate.update.support.constraint);

    ALSparseWorkTrace work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    const SmokeFixture fixture = make_b4e2d_dam_fixture();
    const JointStaticSupportIndex index = build_joint_static_support_index(
        tagged_points(fixture.boundary), &static_work);
    const JointStaticSupportBinding binding = bind_joint_static_support_index(
        &index, index.identity_sha256);
    ALNormalizedWorkspace workspace;
    if (identity_exact && parent_exact && source_exact
        && index.passed && binding.passed) {
        workspace = build_al_normalized_sparse_workspace(
            position, binding, dual, boundary.theta, &work,
            &static_work, &adjacency_work);
    }
    const bool workspace_exact = workspace.passed
        && workspace.support.density
            == capture.candidate.update.support.density
        && workspace.support.constraint
            == capture.candidate.update.support.constraint;
    const std::string topology_root = al_r29_topology_root(workspace);
    const bool operator_exact = workspace_exact
        && topology_root
            == "070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5"
        && workspace.neighborhood.pairs.size() == 340340U
        && workspace.neighborhood.maximum_degree == 113U;

    const std::size_t count = position.size();
    std::vector<std::uint8_t> violated(count, 0U);
    std::vector<std::uint8_t> active(count, 0U);
    std::vector<std::uint8_t> boundary_rows(count, 0U);
    std::vector<std::uint8_t> interior(count, 1U);
    std::vector<std::size_t> selected_rows;
    for (std::size_t row = 0U; row < count; ++row) {
        violated[row] = workspace_exact
            && workspace.support.constraint[row] > 0.0;
        active[row] = workspace_exact
            && dual[row] + workspace.support.constraint[row] > 0.0;
        if (violated[row]) selected_rows.push_back(row);
    }
    if (workspace_exact) {
        for (const JointPair pair : workspace.neighborhood.pairs) {
            if (pair.participant >= count) boundary_rows[pair.fluid] = 1U;
        }
    }
    for (std::size_t row = 0U; row < count; ++row) {
        interior[row] = boundary_rows[row] ? 0U : 1U;
    }
    const std::string violated_root = al_r29_mask_root(violated);
    const std::string active_root = al_r29_mask_root(active);
    const std::string boundary_root = al_r29_mask_root(boundary_rows);
    const std::string interior_root = al_r29_mask_root(interior);
    const bool partition_exact = operator_exact
        && selected_rows.size() == 1420U
        && violated_root
            == "bd06f7b7c5e1f1943e5c269bbfb192229504091381b8f20f0f4988eecb021c70"
        && active_root
            == "b84e502736ee7eed3239fe4dfcd86ec92c54dbffb0ce78854087318275d57029"
        && boundary_root
            == "25ed05cd3b2703de3f2434c825dc4d8894fcab76e824298421f0416e36df32fc"
        && interior_root
            == "72515617c229e4db8ef84aaa96455c3ba8a4f7cdd2060315e1aff5d406d94f2d";

    const std::array<ALR30DenseControl, 4U> dense_controls =
        al_r30_dense_controls();
    const bool dense_exact = std::all_of(
        dense_controls.begin(), dense_controls.end(),
        [](const ALR30DenseControl& value) { return value.passed; });
    std::ostringstream dense_projection;
    for (const ALR30DenseControl& value : dense_controls) {
        dense_projection << value.name << ':' << value.root << ':'
                         << value.passed << '|';
    }
    const std::string dense_root = sha256_hex(dense_projection.str());

    std::size_t pair_passes = 0U;
    ALR30ColumnScaling scaling;
    if (partition_exact && dense_exact) {
        scaling = al_r30_column_scaling(workspace, violated);
        ++pair_passes;
    }
    const bool scaling_exact = scaling.passed
        && scaling.reciprocal.size() == 3U * count
        && scaling.nonzero_columns + scaling.zero_columns == 3U * count
        && std::isfinite(scaling.scaled_frobenius)
        && scaling.scaled_frobenius > 0.0;

    std::vector<double> rhs(selected_rows.size(), 0.0);
    for (std::size_t index_value = 0U;
         index_value < selected_rows.size(); ++index_value) {
        rhs[index_value] = -workspace.support.constraint[
            selected_rows[index_value]];
    }
    std::vector<double> full_response;
    const auto forward = [&](const std::vector<double>& value) {
        const std::vector<Vec3> direction = al_r30_unflatten_scaled(
            value, scaling.reciprocal);
        const ALLinearizedJvp image = al_r29_pair_once_jvp(
            workspace, direction);
        ++pair_passes;
        full_response = image.value;
        std::vector<double> result(selected_rows.size(), 0.0);
        if (!image.passed) return std::vector<double>{};
        for (std::size_t row = 0U; row < selected_rows.size(); ++row) {
            result[row] = image.value[selected_rows[row]];
        }
        return result;
    };
    const auto adjoint = [&](const std::vector<double>& value) {
        if (value.size() != selected_rows.size()) {
            return std::vector<double>{};
        }
        std::vector<double> weights(count, 0.0);
        for (std::size_t row = 0U; row < selected_rows.size(); ++row) {
            weights[selected_rows[row]] = value[row];
        }
        const std::vector<Vec3> image = al_r29_pair_once_vjp(
            workspace, weights);
        ++pair_passes;
        return al_r30_flatten_scaled(image, scaling.reciprocal);
    };

    ALR30LSMRResult solve;
    if (scaling_exact) {
        solve = al_r30_lsmr(rhs, 3U * count, forward, adjoint);
    }
    std::vector<double> projection;
    std::vector<double> residual;
    std::vector<double> normal;
    if (solve.finite) {
        projection = forward(solve.value);
        if (projection.size() == rhs.size()) {
            residual.resize(rhs.size());
            for (std::size_t row = 0U; row < rhs.size(); ++row) {
                residual[row] = rhs[row] - projection[row];
            }
            normal = adjoint(residual);
        }
    }

    const double rhs_norm = al_r30_norm(rhs);
    const double projection_norm = al_r30_norm(projection);
    const double residual_norm = al_r30_norm(residual);
    const double normal_norm = al_r30_norm(normal);
    const double iterate_norm = al_r30_norm(solve.value);
    const double compatible_limit = AL_R30_BTOL * rhs_norm
        + AL_R30_ATOL * scaling.scaled_frobenius * iterate_norm;
    const bool compatible = residual.size() == rhs.size()
        && residual_norm <= compatible_limit;
    const double normal_denominator =
        scaling.scaled_frobenius * residual_norm;
    const double normal_ratio = normal_denominator > 0.0
        ? normal_norm / normal_denominator
        : (normal_norm == 0.0 ? 0.0
            : std::numeric_limits<double>::infinity());
    const bool least_squares = normal.size() == 3U * count
        && normal_ratio <= AL_R30_ATOL;
    const bool convergence_exact = solve.finite
        && solve.iterations <= AL_R30_MAX_ITERATIONS
        && (compatible || least_squares);

    const bool residual_exact = convergence_exact
        && projection.size() == rhs.size()
        && residual.size() == rhs.size()
        && normal.size() == 3U * count
        && al_r30_finite(projection) && al_r30_finite(residual)
        && al_r30_finite(normal) && std::isfinite(normal_ratio);
    const long double projection_residual = al_r30_dot(
        projection, residual);
    const double orthogonality_angle = projection_norm > 0.0
            && residual_norm > 0.0
        ? static_cast<double>(std::abs(projection_residual)
            / (static_cast<long double>(projection_norm) * residual_norm))
        : 0.0;
    const long double rhs_squared = al_r30_dot(rhs, rhs);
    const double orthogonality = static_cast<double>(
        std::abs(projection_residual)
        / std::max(std::abs(rhs_squared), 1.0e-300L));
    const long double pythagorean_difference = std::abs(
        rhs_squared - al_r30_dot(projection, projection)
            - al_r30_dot(residual, residual));
    const double pythagorean = static_cast<double>(
        pythagorean_difference
        / std::max(std::abs(rhs_squared), 1.0e-300L));
    const bool orthogonality_exact = residual_exact
        && std::isfinite(orthogonality) && std::isfinite(pythagorean)
        && orthogonality <= 1.0e-10 && pythagorean <= 1.0e-10;

    const double boundary_rhs = al_r30_masked_norm(
        rhs, boundary_rows, selected_rows);
    const double interior_rhs = al_r30_masked_norm(
        rhs, interior, selected_rows);
    const double boundary_projection = al_r30_masked_norm(
        projection, boundary_rows, selected_rows);
    const double interior_projection = al_r30_masked_norm(
        projection, interior, selected_rows);
    const double boundary_residual = al_r30_masked_norm(
        residual, boundary_rows, selected_rows);
    const double interior_residual = al_r30_masked_norm(
        residual, interior, selected_rows);

    const std::vector<Vec3> preimage = al_r30_unflatten_scaled(
        solve.value, scaling.reciprocal);
    long double preimage_squared = 0.0L;
    double preimage_maximum = 0.0;
    for (Vec3 value : preimage) {
        preimage_squared += static_cast<long double>(norm_squared(value));
        preimage_maximum = std::max(preimage_maximum, norm(value));
    }
    const double preimage_rms = preimage.empty() ? 0.0 : std::sqrt(
        static_cast<double>(preimage_squared)
            / static_cast<double>(preimage.size()));
    std::size_t newly_positive = 0U;
    double maximum_predicted_positive = 0.0;
    if (full_response.size() == count) {
        for (std::size_t row = 0U; row < count; ++row) {
            const double predicted = workspace.support.constraint[row]
                + full_response[row];
            if (!violated[row] && predicted > 0.0) {
                ++newly_positive;
                maximum_predicted_positive = std::max(
                    maximum_predicted_positive, predicted);
            }
        }
    }

    const std::size_t expected_pair_passes = scaling_exact
        ? 1U + solve.forward_calls + solve.adjoint_calls
            + (solve.finite ? 2U : 0U)
        : pair_passes;
    release_al_normalized_workspace(workspace, &work);
    const bool work_exact = pair_passes == expected_pair_passes
        && pair_passes <= 1028U
        && work.workspace_builds == 1U && work.workspace_releases == 1U
        && work.live_workspaces == 0U && work.maximum_live_workspaces == 1U
        && !work.lifecycle_underflow && work.all_pair_candidate_calls == 0U
        && static_work.static_index_builds == 1U
        && static_work.support_canonicalizations == 1U
        && static_work.workspace_builds == 1U
        && adjacency_work.workspace_builds == 1U;
    const bool rollback_exact = position_before == al_binary64_vec3_root(position)
        && dual_before == al_r10_dual_root(dual)
        && constraint_before == al_r10_dual_root(
            capture.candidate.update.support.constraint);

    struct RouteCase {
        const char* expected;
        std::string observed;
    };
    const std::array<RouteCase, 11U> routes{{
        {"LINEARIZED_RANGE_PROJECTION_CANDIDATE", al_r30_route(1,1,1,1,1,1,1,1,1,1)},
        {"RANGE_PROJECTION_PARENT_REJECTED", al_r30_route(0,1,1,1,1,1,1,1,1,1)},
        {"RANGE_PROJECTION_SOURCE_REJECTED", al_r30_route(1,0,1,1,1,1,1,1,1,1)},
        {"RANGE_PROJECTION_OPERATOR_REJECTED", al_r30_route(1,1,0,1,1,1,1,1,1,1)},
        {"RANGE_PROJECTION_PARTITION_REJECTED", al_r30_route(1,1,1,0,1,1,1,1,1,1)},
        {"RANGE_PROJECTION_DENSE_CONTROL_REJECTED", al_r30_route(1,1,1,1,0,1,1,1,1,1)},
        {"RANGE_PROJECTION_SCALING_REJECTED", al_r30_route(1,1,1,1,1,0,1,1,1,1)},
        {"RANGE_PROJECTION_NOT_CONVERGED", al_r30_route(1,1,1,1,1,1,0,1,1,1)},
        {"RANGE_PROJECTION_RESIDUAL_REJECTED", al_r30_route(1,1,1,1,1,1,1,0,1,1)},
        {"RANGE_PROJECTION_ORTHOGONALITY_REJECTED", al_r30_route(1,1,1,1,1,1,1,1,0,1)},
        {"RANGE_PROJECTION_WORK_REJECTED", al_r30_route(1,1,1,1,1,1,1,1,1,0)},
    }};
    bool routes_exact = true;
    std::ostringstream route_projection;
    for (const RouteCase& value : routes) {
        routes_exact = routes_exact && value.expected == value.observed;
        route_projection << value.expected << ':' << value.observed << '|';
    }
    const std::string route_root = sha256_hex(route_projection.str());
    const bool passed = identity_exact && parent_exact && source_exact
        && operator_exact && partition_exact && dense_exact && scaling_exact
        && convergence_exact && residual_exact && orthogonality_exact
        && work_exact && rollback_exact && routes_exact;
    const std::string route = passed
        ? "LINEARIZED_RANGE_PROJECTION_CANDIDATE" : std::string{};
    std::string first_failure;
    if (!identity_exact) first_failure = "IDENTITY";
    else if (!parent_exact) first_failure = "PARENT";
    else if (!source_exact) first_failure = "SOURCE";
    else if (!operator_exact) first_failure = "OPERATOR";
    else if (!partition_exact) first_failure = "PARTITION";
    else if (!dense_exact) first_failure = "DENSE_CONTROL";
    else if (!scaling_exact) first_failure = "SCALING";
    else if (!convergence_exact) first_failure = "CONVERGENCE";
    else if (!residual_exact) first_failure = "RESIDUAL";
    else if (!orthogonality_exact) first_failure = "ORTHOGONALITY";
    else if (!work_exact) first_failure = "WORK";
    else if (!rollback_exact) first_failure = "ROLLBACK";
    else if (!routes_exact) first_failure = "ROUTES";

    const std::string rhs_root = al_r10_dual_root(rhs);
    const std::string iterate_root = al_r10_dual_root(solve.value);
    const std::string preimage_root = al_binary64_vec3_root(preimage);
    const std::string projection_root = al_r10_dual_root(projection);
    const std::string residual_root = al_r10_dual_root(residual);
    const std::string normal_root = al_r10_dual_root(normal);
    const std::string response_root = al_r10_dual_root(full_response);
    const double residual_ratio = rhs_norm > 0.0
        ? residual_norm / rhs_norm : 0.0;
    std::ostringstream semantic;
    semantic << (passed ? "PASS|" : "FAIL|")
             << B4E2D7R19R30_IDENTITY_SHA256 << '|' << parent_stdout << '|'
             << topology_root << ':' << violated_root << '|'
             << dense_root << ':' << scaling.root << '|'
             << rhs_root << ':' << iterate_root << ':' << preimage_root << ':'
             << projection_root << ':' << residual_root << ':' << normal_root
             << ':' << response_root << '|'
             << solve.stop << ':' << solve.iterations << ':' << pair_passes
             << ':' << binary64_bits(residual_ratio) << ':'
             << binary64_bits(normal_ratio) << ':'
             << binary64_bits(orthogonality) << ':'
             << binary64_bits(pythagorean) << ':' << newly_positive << '|'
             << route_root << ':' << routes_exact << '|' << route;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4e2d7r19r30_linearized_range_projection.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"identity_sha256\":\"" << B4E2D7R19R30_IDENTITY_SHA256
           << "\",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
           << ",\"topology_root\":\"" << topology_root << "\""
           << ",\"rows\":" << selected_rows.size()
           << ",\"columns\":" << 3U * count
           << ",\"dense\":{\"passed\":"
           << (dense_exact ? "true" : "false")
           << ",\"cases\":" << dense_controls.size()
           << ",\"root\":\"" << dense_root << "\"}"
           << ",\"scaling\":{\"nonzero_columns\":"
           << scaling.nonzero_columns << ",\"zero_columns\":"
           << scaling.zero_columns << ",\"frobenius\":"
           << scaling.scaled_frobenius << ",\"norm_root\":\""
           << scaling.norm_root << "\",\"reciprocal_root\":\""
           << scaling.reciprocal_root << "\",\"root\":\""
           << scaling.root << "\"}"
           << ",\"solver\":{\"method\":\"LSMR\",\"stop\":\""
           << al_r30_stop_name(solve.stop) << "\",\"stop_code\":"
           << solve.stop << ",\"iterations\":" << solve.iterations
           << ",\"forward_calls\":" << solve.forward_calls
           << ",\"adjoint_calls\":" << solve.adjoint_calls
           << ",\"estimated_normal_increases\":"
           << solve.estimated_normal_increases
           << ",\"operator_norm_estimate\":" << solve.operator_norm
           << ",\"condition_estimate\":" << solve.condition_estimate << "}"
           << ",\"projection\":{\"rhs_root\":\"" << rhs_root
           << "\",\"iterate_root\":\"" << iterate_root
           << "\",\"preimage_root\":\"" << preimage_root
           << "\",\"projection_root\":\"" << projection_root
           << "\",\"residual_root\":\"" << residual_root
           << "\",\"normal_root\":\"" << normal_root
           << "\",\"full_response_root\":\"" << response_root
           << "\",\"rhs_norm\":" << rhs_norm
           << ",\"projection_norm\":" << projection_norm
           << ",\"residual_norm\":" << residual_norm
           << ",\"residual_ratio\":" << residual_ratio
           << ",\"normal_norm\":" << normal_norm
           << ",\"normal_ratio\":" << normal_ratio
           << ",\"compatible_limit\":" << compatible_limit
           << ",\"compatible\":" << (compatible ? "true" : "false")
           << ",\"least_squares\":" << (least_squares ? "true" : "false")
           << ",\"orthogonality\":" << orthogonality
           << ",\"orthogonality_angle_cosine\":" << orthogonality_angle
           << ",\"pythagorean_relative\":" << pythagorean << "}"
           << ",\"localization\":{\"boundary_rhs\":" << boundary_rhs
           << ",\"interior_rhs\":" << interior_rhs
           << ",\"boundary_projection\":" << boundary_projection
           << ",\"interior_projection\":" << interior_projection
           << ",\"boundary_residual\":" << boundary_residual
           << ",\"interior_residual\":" << interior_residual
           << ",\"newly_positive\":" << newly_positive
           << ",\"maximum_predicted_positive\":"
           << maximum_predicted_positive
           << ",\"preimage_rms\":" << preimage_rms
           << ",\"preimage_maximum\":" << preimage_maximum << "}"
           << ",\"work\":{\"diagnostic_workspaces\":1,\"pair_passes\":"
           << pair_passes << ",\"new_hvp\":0,\"new_model\":0"
              ",\"new_trials\":0,\"new_precision\":0,\"new_outer\":0}"
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
    if (output_capture != nullptr) {
        output_capture->captured = passed;
        output_capture->state_root = capture.state_root;
        output_capture->position = position;
        output_capture->predicted_position = boundary.predicted_position;
        output_capture->dual = dual;
        output_capture->theta = boundary.theta;
        output_capture->constraint =
            capture.candidate.update.support.constraint;
        output_capture->violated = std::move(violated);
        output_capture->preimage = std::move(preimage);
        output_capture->full_response = std::move(full_response);
        output_capture->topology_root = topology_root;
        output_capture->violated_root = violated_root;
        output_capture->rhs_root = rhs_root;
        output_capture->preimage_root = preimage_root;
        output_capture->response_root = response_root;
        output_capture->preimage_rms = preimage_rms;
        output_capture->preimage_maximum = preimage_maximum;
    }
    return {passed, report.str()};
}

SplitBoundaryReport run_al_linearized_range_projection_controls() {
    return run_al_linearized_range_projection_controls_impl(nullptr);
}
