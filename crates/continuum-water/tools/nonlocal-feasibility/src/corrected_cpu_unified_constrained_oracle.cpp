#include "corrected_cpu_unified_constrained.hpp"

#include <algorithm>
#include <array>
#include <stdexcept>
#include <utility>

#if __cplusplus != 201703L
#error "NCGP15 oracle requires exact ISO C++17 mode"
#endif

namespace nextengine::nonlocal::ncgp15 {
namespace {

constexpr long double kTinyDenominator = 1.0e-30L;

struct Kernel15 {
    long double value = 0.0L;
    long double first = 0.0L;
};

struct DensityEdge15 {
    std::size_t index = 0U;
    bool ghost = false;
};

struct DensityPayload15 {
    Density15 public_value;
    std::vector<std::vector<DensityEdge15>> rows;
};

template <typename T>
T pairwise_sum15(std::vector<T> values) {
    if (values.empty()) return T{};
    while (values.size() > 1U) {
        std::vector<T> next;
        next.reserve((values.size() + 1U) / 2U);
        for (std::size_t index = 0U; index + 1U < values.size(); index += 2U) {
            next.push_back(values[index] + values[index + 1U]);
        }
        if ((values.size() & 1U) != 0U) next.push_back(values.back());
        values = std::move(next);
    }
    return values.front();
}

long double vector_norm15(const std::vector<Vec3l>& values) {
    std::vector<long double> squares;
    squares.reserve(values.size());
    for (const Vec3l value : values) squares.push_back(dot15(value, value));
    return std::sqrt(std::max(pairwise_sum15(std::move(squares)), 0.0L));
}

long double scalar_norm15(const std::vector<long double>& values) {
    std::vector<long double> squares;
    squares.reserve(values.size());
    for (const long double value : values) squares.push_back(value * value);
    return std::sqrt(std::max(pairwise_sum15(std::move(squares)), 0.0L));
}

long double maximum_vector_norm15(const std::vector<Vec3l>& values) {
    long double result = 0.0L;
    for (const Vec3l value : values) result = std::max(result, norm15(value));
    return result;
}

long double maximum_abs_component15(const std::vector<Vec3l>& values) {
    long double result = 0.0L;
    for (const Vec3l value : values) {
        result = std::max({result, std::abs(value.x), std::abs(value.y),
            std::abs(value.z)});
    }
    return result;
}

bool finite_scalar15(long double value) {
    return std::isfinite(value);
}

bool finite_vector15(const std::vector<Vec3l>& values) {
    return std::all_of(values.begin(), values.end(), finite15);
}

bool finite_scalars15(const std::vector<long double>& values) {
    return std::all_of(values.begin(), values.end(), finite_scalar15);
}

bool counted_predicate15(Work15& work, bool value) {
    ++work.scalar_predicates;
    return value;
}

constexpr std::uint64_t kProfilePredicateCount15 = 55U;
constexpr std::uint64_t kOptionPredicateCount15 = 3U;

bool valid_profile15(const Profile15& profile, Work15* work = nullptr) {
    const Profile15 frozen = frozen_profile15();
    bool valid = true;
    const auto check = [&](bool predicate) {
        if (work != nullptr) ++work->scalar_predicates;
        valid = predicate && valid;
    };
    check(profile.id == frozen.id);
    check(profile.dt == frozen.dt);
    check(profile.spacing == frozen.spacing);
    check(profile.horizon == frozen.horizon);
    check(profile.surface_r0 == frozen.surface_r0);
    check(profile.mass == frozen.mass);
    check(profile.rho0 == frozen.rho0);
    check(profile.kernel_scale == frozen.kernel_scale);
    check(profile.beta == frozen.beta);
    check(profile.lambda_v == frozen.lambda_v);
    check(profile.mu_v == frozen.mu_v);
    check(profile.gamma == frozen.gamma);
    check(profile.gravity.x == frozen.gravity.x);
    check(profile.gravity.y == frozen.gravity.y);
    check(profile.gravity.z == frozen.gravity.z);
    check(profile.basin.x == frozen.basin.x);
    check(profile.basin.y == frozen.basin.y);
    check(profile.basin.z == frozen.basin.z);
    check(profile.ghost_layers == frozen.ghost_layers);
    check(profile.maximum_dynamic_samples == frozen.maximum_dynamic_samples);
    check(profile.maximum_neighbors == frozen.maximum_neighbors);
    check(profile.maximum_outer_updates == frozen.maximum_outer_updates);
    check(profile.maximum_accepted_inner_iterations
        == frozen.maximum_accepted_inner_iterations);
    check(profile.maximum_line_search_backtracks
        == frozen.maximum_line_search_backtracks);
    check(profile.armijo == frozen.armijo);
    check(profile.initial_alpha == frozen.initial_alpha);
    check(profile.line_search_shrink == frozen.line_search_shrink);
    check(profile.finite_difference_scale == frozen.finite_difference_scale);
    check(profile.gates.inner_rms_relative
        == frozen.gates.inner_rms_relative);
    check(profile.gates.inner_max_relative
        == frozen.gates.inner_max_relative);
    check(profile.gates.density_maximum == frozen.gates.density_maximum);
    check(profile.gates.density_rms == frozen.gates.density_rms);
    check(profile.gates.kkt_rms_m == frozen.gates.kkt_rms_m);
    check(profile.gates.kkt_maximum_m == frozen.gates.kkt_maximum_m);
    check(profile.gates.complementarity == frozen.gates.complementarity);
    check(profile.gates.multiplier_fixed_point
        == frozen.gates.multiplier_fixed_point);
    check(profile.gates.density_relative_l2
        == frozen.gates.density_relative_l2);
    check(profile.gates.gradient_relative_l2
        == frozen.gates.gradient_relative_l2);
    check(profile.gates.gradient_maximum_n
        == frozen.gates.gradient_maximum_n);
    check(profile.gates.pair_position_m == frozen.gates.pair_position_m);
    check(profile.gates.normal_decay_relative
        == frozen.gates.normal_decay_relative);
    check(profile.gates.tangential_change_absolute
        == frozen.gates.tangential_change_absolute);
    check(profile.gates.center_of_mass_relative
        == frozen.gates.center_of_mass_relative);
    check(profile.gates.surface_energy_relative_excess
        == frozen.gates.surface_energy_relative_excess);
    check(profile.gates.surface_energy_absolute_excess_j
        == frozen.gates.surface_energy_absolute_excess_j);
    check(profile.gates.reversible_energy_drift
        == frozen.gates.reversible_energy_drift);
    check(profile.gates.one_step_position_rms_m
        == frozen.gates.one_step_position_rms_m);
    check(profile.gates.one_step_position_maximum_m
        == frozen.gates.one_step_position_maximum_m);
    check(profile.gates.trajectory_position_rms_m
        == frozen.gates.trajectory_position_rms_m);
    check(profile.gates.trajectory_position_maximum_m
        == frozen.gates.trajectory_position_maximum_m);
    check(profile.gates.trajectory_velocity_rms_mps
        == frozen.gates.trajectory_velocity_rms_mps);
    check(profile.gates.trajectory_velocity_maximum_mps
        == frozen.gates.trajectory_velocity_maximum_mps);
    check(profile.gates.momentum_residual
        == frozen.gates.momentum_residual);
    check(profile.gates.energy_excess == frozen.gates.energy_excess);
    check(profile.gates.internal_force_closure
        == frozen.gates.internal_force_closure);
    return valid;
}

std::uint64_t canonical_predicate_count15(const State15& state) {
    return 1U + 4U * state.dynamic.size() + 2U * state.ghosts.size();
}

bool canonical_state15(const State15& state, Work15* work = nullptr) {
    bool valid = true;
    const auto check = [&](bool predicate) {
        if (work != nullptr) ++work->scalar_predicates;
        valid = predicate && valid;
    };
    check(!state.dynamic.empty());
    std::uint32_t prior_id = 0U;
    bool first_id = true;
    for (std::size_t index = 0U; index < state.dynamic.size(); ++index) {
        const Dynamic15& item = state.dynamic[index];
        check(finite15(item.reference));
        check(finite15(item.position));
        check(finite15(item.velocity));
        check(first_id || item.id > prior_id);
        prior_id = item.id;
        first_id = false;
    }
    for (std::size_t index = 0U; index < state.ghosts.size(); ++index) {
        const Ghost15& item = state.ghosts[index];
        check(finite15(item.position));
        check(first_id || item.id > prior_id);
        prior_id = item.id;
        first_id = false;
    }
    return valid;
}

State15 sorted_state15(State15 state) {
    std::sort(state.dynamic.begin(), state.dynamic.end(),
        [](const Dynamic15& lhs, const Dynamic15& rhs) {
            return lhs.id < rhs.id;
        });
    std::sort(state.ghosts.begin(), state.ghosts.end(),
        [](const Ghost15& lhs, const Ghost15& rhs) {
            return lhs.id < rhs.id;
        });
    return state;
}

Kernel15 kernel15(long double radius, const SolverOptions15& options) {
    const Profile15& profile = options.profile;
    const long double pi = std::acos(-1.0L);
    const long double q = 2.0L * radius / profile.horizon;
    const long double alpha = profile.kernel_scale * 3.0L
        / (2.0L * pi * profile.horizon * profile.horizon * profile.horizon);
    long double value = 0.0L;
    long double first_q = 0.0L;
    if (q < 1.0L) {
        value = alpha * (2.0L / 3.0L - q * q
            + 0.5L * q * q * q);
        first_q = alpha * (-2.0L * q + 1.5L * q * q);
    } else if (q <= 2.0L) {
        const long double tail = 2.0L - q;
        value = alpha * tail * tail * tail / 6.0L;
        first_q = -0.5L * alpha * tail * tail;
    }
    const long double chain = options.mutation == Mutation15::MissingKernelChain
        ? 1.0L : 2.0L / profile.horizon;
    return {value, first_q * chain};
}

long double surface_coefficient15(long double q) {
    if (q <= 1.0L) return q * q - 1.0L;
    if (q < 3.0L) {
        const long double shifted = q - 2.0L;
        return 1.0L - shifted * shifted;
    }
    return 0.0L;
}

long double surface_potential15(long double radius,
    const Profile15& profile) {
    const long double q = radius / profile.surface_r0;
    if (q <= 1.0L) {
        return profile.surface_r0
            * (q * q * q / 3.0L - q - 2.0L / 3.0L);
    }
    if (q < 3.0L) {
        const long double shifted = q - 2.0L;
        return profile.surface_r0
            * (q - shifted * shifted * shifted / 3.0L - 8.0L / 3.0L);
    }
    return 0.0L;
}

long double surface_factor15(const SolverOptions15& options) {
    long double factor = options.mutation == Mutation15::MissingSurfaceFactorTwo
        ? 1.0L : 2.0L;
    if (options.mutation == Mutation15::WrongSurfaceSign) factor = -factor;
    return factor * options.profile.gamma * options.profile.mass
        * options.profile.mass;
}

Vec3l selected_gravity15(const SolverOptions15& options) {
    return options.gravity_mode == GravityMode15::Zero
        ? Vec3l{} : options.profile.gravity;
}

bool project_all15(const std::vector<Vec3l>& input,
    const SolverOptions15& options, std::vector<Vec3l>& output,
    Work15& work) {
    bool finite = true;
    for (const Vec3l value : input) {
        finite = counted_predicate15(work, finite15(value)) && finite;
    }
    output.resize(input.size());
    if (!finite) return false;
    if (options.boundary_mode == BoundaryMode15::UnboundedManufactured) {
        output = input;
        return true;
    }
    const Profile15& profile = options.profile;
    const long double lower = 0.5L * profile.spacing;
    const Vec3l upper{profile.basin.x - lower, profile.basin.y - lower,
        profile.basin.z - lower};
    const auto clamp = [&](long double value, long double maximum) {
        work.box_plane_tests += 2U;
        return std::min(std::max(value, lower), maximum);
    };
    for (std::size_t index = 0U; index < input.size(); ++index) {
        output[index] = {clamp(input[index].x, upper.x),
            clamp(input[index].y, upper.y),
            clamp(input[index].z, upper.z)};
    }
    return true;
}

DensityPayload15 density_payload15(const State15& accepted,
    const std::vector<Vec3l>& positions, const SolverOptions15& options,
    Work15& work) {
    DensityPayload15 result;
    Density15& published = result.public_value;
    published.outcome = "INVALID_STATE";
    const std::size_t count = accepted.dynamic.size();
    const bool canonical = canonical_state15(accepted, &work);
    const bool size_valid = counted_predicate15(
        work, positions.size() == count);
    bool positions_finite = true;
    for (const Vec3l position : positions) {
        positions_finite = counted_predicate15(work, finite15(position))
            && positions_finite;
    }
    if (!canonical || !size_valid || !positions_finite) {
        return result;
    }
    ++work.graph_builds;
    published.rho.assign(count, 0.0L);
    published.constraint.assign(count, 0.0L);
    result.rows.resize(count);
    for (std::size_t owner = 0U; owner < count; ++owner) {
        std::uint32_t degree = 0U;
        std::vector<long double> contributions;
        contributions.reserve(count + accepted.ghosts.size());
        for (std::size_t neighbor = 0U; neighbor < count; ++neighbor) {
            ++work.oracle_pair_tests;
            const Vec3l difference = positions[owner] - positions[neighbor];
            const long double radius = norm15(difference);
            if (neighbor != owner && !(radius > 0.0L)) {
                published.outcome = "COINCIDENT_DISTINCT_PAIR";
                return result;
            }
            if (radius > options.profile.horizon) continue;
            result.rows[owner].push_back({neighbor, false});
            ++degree;
            ++work.accepted_pairs;
            ++work.density_pairs;
            contributions.push_back(options.profile.mass
                * kernel15(radius, options).value);
        }
        for (std::size_t ghost = 0U; ghost < accepted.ghosts.size(); ++ghost) {
            ++work.oracle_pair_tests;
            const Vec3l difference = positions[owner]
                - accepted.ghosts[ghost].position;
            const long double radius = norm15(difference);
            if (!(radius > 0.0L)) {
                published.outcome = "COINCIDENT_DISTINCT_PAIR";
                return result;
            }
            if (radius > options.profile.horizon) continue;
            result.rows[owner].push_back({ghost, true});
            ++degree;
            ++work.accepted_pairs;
            ++work.density_pairs;
            contributions.push_back(options.profile.mass
                * kernel15(radius, options).value);
        }
        if (degree > options.profile.maximum_neighbors) {
            published.capacity_overflow = true;
            published.outcome = "CAPACITY_OVERFLOW";
            return result;
        }
        published.maximum_degree = std::max(published.maximum_degree, degree);
        published.accepted_pair_count += degree;
        published.rho[owner] = pairwise_sum15(std::move(contributions));
        published.constraint[owner] =
            published.rho[owner] / options.profile.rho0 - 1.0L;
        if (published.constraint[owner] > 0.0L) {
            published.active_ids.push_back(accepted.dynamic[owner].id);
        }
    }
    published.valid = finite_scalars15(published.rho)
        && finite_scalars15(published.constraint);
    published.outcome = published.valid ? "PASS" : "NONFINITE";
    return result;
}

EnergyGradient15 invalid_evaluation15(std::string outcome) {
    EnergyGradient15 result;
    result.outcome = std::move(outcome);
    return result;
}

bool finite_evaluation15(const EnergyGradient15& value) {
    return finite_scalar15(value.objective)
        && finite_scalar15(value.nonpressure_energy)
        && finite_scalar15(value.inertia_energy)
        && finite_scalar15(value.viscosity_energy)
        && finite_scalar15(value.surface_energy)
        && finite_vector15(value.gradient)
        && finite_scalars15(value.effective_multiplier)
        && finite_vector15(value.dynamic_pressure_force)
        && finite_vector15(value.ghost_pressure_force)
        && finite_vector15(value.viscosity_force)
        && finite_vector15(value.surface_force);
}

EnergyGradient15 energy_gradient_impl15(const State15& accepted,
    const std::vector<Vec3l>& positions,
    const std::vector<long double>& multipliers,
    const SolverOptions15& options, MultiplierMode15 multiplier_mode,
    bool need_gradient, Work15& work) {
    const std::size_t count = accepted.dynamic.size();
    const bool profile_valid = valid_profile15(options.profile, &work);
    const bool canonical = canonical_state15(accepted, &work);
    const bool positions_size = counted_predicate15(
        work, positions.size() == count);
    const bool multipliers_size = counted_predicate15(
        work, multipliers.size() == count);
    bool positions_finite = true;
    for (const Vec3l position : positions) {
        positions_finite = counted_predicate15(work, finite15(position))
            && positions_finite;
    }
    bool multipliers_finite = true;
    for (const long double multiplier : multipliers) {
        multipliers_finite = counted_predicate15(
            work, finite_scalar15(multiplier)) && multipliers_finite;
    }
    if (!profile_valid) {
        return invalid_evaluation15("INVALID_PROFILE");
    }
    if (!canonical || !positions_size || !multipliers_size
        || !positions_finite || !multipliers_finite) {
        return invalid_evaluation15("INVALID_STATE");
    }
    ++work.objective_evaluations;
    if (need_gradient) ++work.gradient_evaluations;
    const DensityPayload15 density = density_payload15(
        accepted, positions, options, work);
    if (!density.public_value.valid) {
        return invalid_evaluation15(density.public_value.outcome);
    }

    EnergyGradient15 result;
    result.density = density.public_value;
    result.gradient.assign(count, {});
    result.dynamic_pressure_force.assign(count, {});
    result.ghost_pressure_force.assign(count, {});
    result.viscosity_force.assign(count, {});
    result.surface_force.assign(count, {});
    result.effective_multiplier.assign(count, 0.0L);
    std::vector<long double> inertia_terms;
    std::vector<long double> viscosity_terms;
    std::vector<long double> surface_terms;
    std::vector<long double> augmented_terms;
    std::vector<long double> penalty_terms;
    inertia_terms.reserve(count);
    augmented_terms.reserve(count);
    penalty_terms.reserve(count);
    const Profile15& profile = options.profile;
    const Vec3l gravity = selected_gravity15(options);
    const long double inertia_scale = profile.mass
        / (profile.dt * profile.dt);

    for (std::size_t index = 0U; index < count; ++index) {
        const Dynamic15& particle = accepted.dynamic[index];
        const Vec3l predicted = particle.position
            + (particle.velocity + gravity * profile.dt) * profile.dt;
        const Vec3l displacement = positions[index] - predicted;
        inertia_terms.push_back(
            0.5L * inertia_scale * dot15(displacement, displacement));
        if (need_gradient) result.gradient[index] += displacement * inertia_scale;

        const long double c = result.density.constraint[index];
        if (options.mutation == Mutation15::FinitePressurePenalty
                && multiplier_mode
                    == MultiplierMode15::AugmentedLagrangian) {
            const long double positive = std::max(c, 0.0L);
            result.effective_multiplier[index] = profile.beta * positive;
            penalty_terms.push_back(0.5L * profile.beta * positive * positive);
        } else if (!options.terms.density_constraint) {
            result.effective_multiplier[index] = 0.0L;
        } else if (multiplier_mode == MultiplierMode15::AugmentedLagrangian) {
            const long double shifted = multipliers[index] + profile.beta * c;
            const long double effective = std::max(0.0L, shifted);
            result.effective_multiplier[index] = effective;
            augmented_terms.push_back((effective * effective
                - multipliers[index] * multipliers[index])
                / (2.0L * profile.beta));
        } else {
            result.effective_multiplier[index] = multipliers[index];
        }
    }

    if (options.terms.normal_viscosity || options.terms.tangential_viscosity) {
        const long double normal_mutation =
            options.mutation == Mutation15::HalfNormalViscosity ? 0.5L : 1.0L;
        for (std::size_t first = 0U; first < count; ++first) {
            for (std::size_t second = first + 1U; second < count; ++second) {
                ++work.oracle_pair_tests;
                const Vec3l accepted_difference =
                    accepted.dynamic[first].position
                    - accepted.dynamic[second].position;
                const Vec3l graph_difference = options.mutation
                        == Mutation15::CurrentReferenceViscosityGraph
                    ? positions[first] - positions[second]
                    : accepted_difference;
                const long double radius = norm15(graph_difference);
                if (!(radius > 0.0L)) {
                    return invalid_evaluation15("COINCIDENT_DISTINCT_PAIR");
                }
                if (radius > profile.horizon) continue;
                ++work.viscosity_pairs;
                ++result.viscosity_pair_count;
                const Vec3l normal = graph_difference / radius;
                const Vec3l delta = (positions[first] - positions[second])
                    - accepted_difference;
                const long double normal_scalar = dot15(normal, delta);
                const Vec3l normal_delta = normal * normal_scalar;
                const Vec3l tangent_delta = delta - normal_delta;
                const long double omega = -kernel15(radius, options).first;
                const long double factor = profile.mass * omega
                    / (profile.rho0 * profile.dt);
                const long double normal_coefficient = options.terms.normal_viscosity
                    ? profile.lambda_v * normal_mutation : 0.0L;
                const long double tangent_coefficient =
                    options.terms.tangential_viscosity ? profile.mu_v : 0.0L;
                viscosity_terms.push_back(factor
                    * (tangent_coefficient * dot15(tangent_delta, tangent_delta)
                        + 0.5L * normal_coefficient
                            * dot15(normal_delta, normal_delta)));
                if (need_gradient) {
                    const Vec3l pair_gradient = factor
                        * (2.0L * tangent_coefficient * tangent_delta
                            + normal_coefficient * normal_delta);
                    result.gradient[first] += pair_gradient;
                    result.gradient[second] -= pair_gradient;
                    result.viscosity_force[first] -= pair_gradient;
                    result.viscosity_force[second] += pair_gradient;
                }
            }
        }
    }

    if (options.terms.surface) {
        const long double factor = surface_factor15(options);
        for (std::size_t first = 0U; first < count; ++first) {
            for (std::size_t second = first + 1U; second < count; ++second) {
                ++work.surface_pair_tests;
                ++work.oracle_pair_tests;
                const Vec3l difference = positions[first] - positions[second];
                const long double radius = norm15(difference);
                if (!(radius > 0.0L)) {
                    return invalid_evaluation15("COINCIDENT_DISTINCT_PAIR");
                }
                if (!(radius < 3.0L * profile.surface_r0)) continue;
                ++work.surface_active_pairs;
                ++result.surface_pair_count;
                surface_terms.push_back(factor
                    * surface_potential15(radius, profile));
                if (need_gradient) {
                    const Vec3l pair_gradient = difference / radius
                        * (factor * surface_coefficient15(
                            radius / profile.surface_r0));
                    result.gradient[first] += pair_gradient;
                    result.gradient[second] -= pair_gradient;
                    result.surface_force[first] -= pair_gradient;
                    result.surface_force[second] += pair_gradient;
                }
            }
        }
    }

    if (need_gradient) {
        for (std::size_t owner = 0U; owner < count; ++owner) {
            const long double multiplier = result.effective_multiplier[owner];
            for (const DensityEdge15 edge : density.rows[owner]) {
                if (!edge.ghost && edge.index == owner) continue;
                const Vec3l endpoint = edge.ghost
                    ? accepted.ghosts[edge.index].position
                    : positions[edge.index];
                const Vec3l difference = positions[owner] - endpoint;
                const long double radius = norm15(difference);
                if (!(radius > 0.0L)) {
                    return invalid_evaluation15("COINCIDENT_DISTINCT_PAIR");
                }
                const Vec3l jacobian = difference / radius
                    * (profile.mass / profile.rho0
                        * kernel15(radius, options).first);
                const Vec3l endpoint_gradient = multiplier * jacobian;
                result.gradient[owner] += endpoint_gradient;
                ++work.jacobian_products;
                if (edge.ghost) {
                    result.ghost_pressure_force[owner] -= endpoint_gradient;
                } else {
                    result.gradient[edge.index] -= endpoint_gradient;
                    result.dynamic_pressure_force[owner] -= endpoint_gradient;
                    result.dynamic_pressure_force[edge.index]
                        += endpoint_gradient;
                }
            }
        }
    }

    result.inertia_energy = pairwise_sum15(std::move(inertia_terms));
    result.viscosity_energy = pairwise_sum15(std::move(viscosity_terms));
    result.surface_energy = pairwise_sum15(std::move(surface_terms));
    const long double pressure_penalty = pairwise_sum15(std::move(penalty_terms));
    result.nonpressure_energy = result.inertia_energy + result.viscosity_energy
        + result.surface_energy;
    result.objective = result.nonpressure_energy + pressure_penalty
        + pairwise_sum15(std::move(augmented_terms));
    result.valid = result.density.valid && finite_evaluation15(result);
    result.outcome = result.valid ? "PASS" : "NONFINITE";
    return result;
}

State15 state_from_trial15(const State15& accepted,
    const std::vector<Vec3l>& positions, const Profile15& profile) {
    State15 result = accepted;
    for (std::size_t index = 0U; index < result.dynamic.size(); ++index) {
        const Vec3l old_position = accepted.dynamic[index].position;
        result.dynamic[index].reference = positions[index];
        result.dynamic[index].position = positions[index];
        result.dynamic[index].velocity =
            (positions[index] - old_position) / profile.dt;
    }
    return result;
}

bool contained15(const std::vector<Vec3l>& positions,
    const SolverOptions15& options, Work15& work) {
    if (options.boundary_mode == BoundaryMode15::UnboundedManufactured) {
        return true;
    }
    const long double lower = 0.5L * options.profile.spacing;
    const Vec3l upper{options.profile.basin.x - lower,
        options.profile.basin.y - lower, options.profile.basin.z - lower};
    bool result = true;
    for (const Vec3l position : positions) {
        work.box_plane_tests += 6U;
        result = (position.x >= lower && position.x <= upper.x
            && position.y >= lower && position.y <= upper.y
            && position.z >= lower && position.z <= upper.z) && result;
    }
    return result;
}

std::vector<Contact15> contact15(const State15& accepted,
    const std::vector<Vec3l>& positions,
    const std::vector<Vec3l>& kkt_gradient,
    const SolverOptions15& options, Work15& work) {
    std::vector<Contact15> result;
    if (options.boundary_mode == BoundaryMode15::UnboundedManufactured) {
        return result;
    }
    result.reserve(positions.size());
    const long double lower = 0.5L * options.profile.spacing;
    const Vec3l upper{options.profile.basin.x - lower,
        options.profile.basin.y - lower, options.profile.basin.z - lower};
    for (std::size_t index = 0U; index < positions.size(); ++index) {
        Contact15 record;
        record.id = accepted.dynamic[index].id;
        const std::array<long double, 3> value{
            positions[index].x, positions[index].y, positions[index].z};
        const std::array<long double, 3> maximum{upper.x, upper.y, upper.z};
        const std::array<long double, 3> gradient{kkt_gradient[index].x,
            kkt_gradient[index].y, kkt_gradient[index].z};
        std::array<long double*, 3> impulse{
            &record.impulse.x, &record.impulse.y, &record.impulse.z};
        for (std::size_t axis = 0U; axis < 3U; ++axis) {
            work.box_plane_tests += 2U;
            ++work.contact_reaction_values;
            if (value[axis] == lower) {
                record.mask |= static_cast<std::uint8_t>(1U << (2U * axis));
                *impulse[axis] = options.profile.dt
                    * std::max(gradient[axis], 0.0L);
            } else if (value[axis] == maximum[axis]) {
                record.mask |= static_cast<std::uint8_t>(
                    1U << (2U * axis + 1U));
                *impulse[axis] = options.profile.dt
                    * std::min(gradient[axis], 0.0L);
            }
        }
        result.push_back(record);
    }
    return result;
}

void set_step_failure15(Step15& step, std::string outcome,
    bool apparatus_valid, bool work_ceiling) {
    step.outcome = std::move(outcome);
    step.apparatus_valid = apparatus_valid;
    step.work_ceiling = work_ceiling;
    step.accepted = false;
}

void publish_private_state15(Step15& step, const State15& accepted,
    const std::vector<Vec3l>& positions,
    const EnergyGradient15* evaluation, const SolverOptions15& options) {
    step.metrics = StepMetrics15{};
    step.rho.clear();
    step.active_multiplier_ids.clear();
    step.contact.clear();
    step.kkt_gradient.clear();
    step.kkt_direction.clear();
    step.dynamic_pressure_force.clear();
    step.ghost_pressure_force.clear();
    step.viscosity_force.clear();
    step.surface_force.clear();
    step.state = state_from_trial15(accepted, positions, options.profile);
    if (evaluation == nullptr) return;
    step.metrics.objective = evaluation->objective;
    step.metrics.nonpressure_energy = evaluation->nonpressure_energy;
    step.metrics.inertia_energy = evaluation->inertia_energy;
    step.metrics.viscosity_energy = evaluation->viscosity_energy;
    step.metrics.surface_energy = evaluation->surface_energy;
    step.rho = evaluation->density.rho;
    step.dynamic_pressure_force = evaluation->dynamic_pressure_force;
    step.ghost_pressure_force = evaluation->ghost_pressure_force;
    step.viscosity_force = evaluation->viscosity_force;
    step.surface_force = evaluation->surface_force;
}

bool final_kkt_gates15(Step15& result,
    const EnergyGradient15& final_evaluation,
    const std::vector<Vec3l>& positions, const SolverOptions15& options,
    StepTrace15& trace) {
    Work15& work = result.work;
    const std::size_t count = positions.size();
    result.kkt_gradient = final_evaluation.gradient;
    std::vector<Vec3l> projected;
    std::vector<Vec3l> proposal(count);
    const long double scale = options.profile.dt * options.profile.dt
        / options.profile.mass;
    for (std::size_t index = 0U; index < count; ++index) {
        proposal[index] = positions[index]
            - final_evaluation.gradient[index] * scale;
    }
    const bool projection_valid = project_all15(
        proposal, options, projected, work);
    trace.reached_stages.push_back("PROJECTION");
    if (projection_valid
            && options.boundary_mode == BoundaryMode15::AnalyticBox) {
        trace.reached_stages.push_back("PROJECTION_BOX");
    }
    if (!projection_valid) {
        result.outcome = "NONFINITE";
        return false;
    }
    work.projected_directions += count;
    result.kkt_direction.resize(count);
    for (std::size_t index = 0U; index < count; ++index) {
        result.kkt_direction[index] = projected[index] - positions[index];
    }
    result.metrics.objective = final_evaluation.objective;
    result.metrics.nonpressure_energy = final_evaluation.nonpressure_energy;
    result.metrics.inertia_energy = final_evaluation.inertia_energy;
    result.metrics.viscosity_energy = final_evaluation.viscosity_energy;
    result.metrics.surface_energy = final_evaluation.surface_energy;
    long double strain_maximum = 0.0L;
    std::vector<long double> strain_squares;
    strain_squares.reserve(count);
    long double complementarity_maximum = 0.0L;
    long double multiplier_maximum = 0.0L;
    bool multipliers_nonnegative = true;
    std::vector<long double> fixed_point_difference;
    fixed_point_difference.reserve(count);
    for (std::size_t index = 0U; index < count; ++index) {
        const long double c = final_evaluation.density.constraint[index];
        const long double strain = std::max(c, 0.0L);
        strain_maximum = std::max(strain_maximum, strain);
        strain_squares.push_back(strain * strain);
        complementarity_maximum = std::max(
            complementarity_maximum, std::abs(result.pi[index] * c));
        multiplier_maximum = std::max(multiplier_maximum, result.pi[index]);
        const long double fixed = std::max(0.0L,
            result.pi[index] + options.profile.beta * c);
        fixed_point_difference.push_back(fixed - result.pi[index]);
        multipliers_nonnegative = counted_predicate15(
            work, result.pi[index] >= 0.0L) && multipliers_nonnegative;
        if (result.pi[index] > 0.0L) {
            result.active_multiplier_ids.push_back(
                result.state.dynamic[index].id);
        }
    }
    result.metrics.maximum_positive_density_strain = strain_maximum;
    result.metrics.rms_positive_density_strain = std::sqrt(std::max(
        pairwise_sum15(std::move(strain_squares))
            / static_cast<long double>(count), 0.0L));
    result.metrics.complementarity = complementarity_maximum
        / std::max(1.0L, multiplier_maximum);
    result.metrics.multiplier_fixed_point = scalar_norm15(
        fixed_point_difference)
        / std::max(scalar_norm15(result.pi), 1.0L);
    result.metrics.active_multiplier_count =
        result.active_multiplier_ids.size();
    result.metrics.kkt_rms_m = vector_norm15(result.kkt_direction)
        / std::sqrt(static_cast<long double>(count));
    result.metrics.kkt_maximum_m = maximum_vector_norm15(result.kkt_direction);
    result.contact = contact15(result.state, positions,
        result.kkt_gradient, options, work);
    if (options.boundary_mode == BoundaryMode15::AnalyticBox) {
        trace.reached_stages.push_back("CONTACT");
    }
    trace.reached_stages.push_back("FINAL_GATE");
    bool derived_finite = finite_evaluation15(final_evaluation)
        && finite_scalars15(final_evaluation.density.rho)
        && finite_scalars15(final_evaluation.density.constraint)
        && finite_scalars15(result.pi) && finite_vector15(positions)
        && finite_vector15(result.kkt_gradient)
        && finite_vector15(result.kkt_direction)
        && finite_vector15(result.dynamic_pressure_force)
        && finite_vector15(result.ghost_pressure_force)
        && finite_vector15(result.viscosity_force)
        && finite_vector15(result.surface_force)
        && finite_scalar15(result.metrics.objective)
        && finite_scalar15(result.metrics.nonpressure_energy)
        && finite_scalar15(result.metrics.inertia_energy)
        && finite_scalar15(result.metrics.viscosity_energy)
        && finite_scalar15(result.metrics.surface_energy)
        && finite_scalar15(result.metrics.maximum_positive_density_strain)
        && finite_scalar15(result.metrics.rms_positive_density_strain)
        && finite_scalar15(result.metrics.kkt_rms_m)
        && finite_scalar15(result.metrics.kkt_maximum_m)
        && finite_scalar15(result.metrics.complementarity)
        && finite_scalar15(result.metrics.multiplier_fixed_point);
    for (const Dynamic15& item : result.state.dynamic) {
        derived_finite = finite15(item.reference) && finite15(item.position)
            && finite15(item.velocity) && derived_finite;
    }
    for (const Contact15& item : result.contact) {
        derived_finite = finite15(item.impulse) && derived_finite;
    }
    const bool finite = counted_predicate15(work, derived_finite);
    if (!finite) result.outcome = "NONFINITE";
    const bool density_max = counted_predicate15(work,
        result.metrics.maximum_positive_density_strain
            <= options.profile.gates.density_maximum);
    const bool density_rms = counted_predicate15(work,
        result.metrics.rms_positive_density_strain
            <= options.profile.gates.density_rms);
    const bool kkt_rms = counted_predicate15(work,
        result.metrics.kkt_rms_m <= options.profile.gates.kkt_rms_m);
    const bool kkt_max = counted_predicate15(work,
        result.metrics.kkt_maximum_m
            <= options.profile.gates.kkt_maximum_m);
    const bool complementarity = counted_predicate15(work,
        result.metrics.complementarity
            <= options.profile.gates.complementarity);
    const bool fixed_point = counted_predicate15(work,
        result.metrics.multiplier_fixed_point
            <= options.profile.gates.multiplier_fixed_point);
    const bool containment = options.boundary_mode
            == BoundaryMode15::AnalyticBox
        ? counted_predicate15(work, contained15(positions, options, work))
        : true;
    const bool capacity = counted_predicate15(work,
        !final_evaluation.density.capacity_overflow);
    return finite && multipliers_nonnegative && density_max && density_rms && kkt_rms
        && kkt_max && complementarity && fixed_point && containment && capacity;
}

struct Membership15 {
    bool valid = false;
    std::vector<std::uint8_t> flags;
};

struct OracleEnumeration15 {
    std::uint64_t pair_tests = 0U;
    std::uint64_t accepted_density_pairs = 0U;
    std::uint64_t viscosity_pairs = 0U;
    std::uint64_t surface_tests = 0U;
    std::uint64_t surface_active = 0U;
    bool density_complete = true;
};

OracleEnumeration15 enumerate_expected_evaluation15(
    const State15& accepted, const std::vector<Vec3l>& positions,
    const SolverOptions15& options) {
    OracleEnumeration15 result;
    const std::size_t count = accepted.dynamic.size();
    for (std::size_t owner = 0U; owner < count; ++owner) {
        std::uint32_t degree = 0U;
        for (std::size_t neighbor = 0U; neighbor < count; ++neighbor) {
            ++result.pair_tests;
            const long double radius = norm15(
                positions[owner] - positions[neighbor]);
            if (neighbor != owner && !(radius > 0.0L)) {
                result.density_complete = false;
                return result;
            }
            if (radius > options.profile.horizon) continue;
            ++degree;
            ++result.accepted_density_pairs;
        }
        for (const Ghost15& ghost : accepted.ghosts) {
            ++result.pair_tests;
            const long double radius = norm15(positions[owner] - ghost.position);
            if (!(radius > 0.0L)) {
                result.density_complete = false;
                return result;
            }
            if (radius > options.profile.horizon) continue;
            ++degree;
            ++result.accepted_density_pairs;
        }
        if (degree > options.profile.maximum_neighbors) {
            result.density_complete = false;
            return result;
        }
    }
    if (options.terms.normal_viscosity
        || options.terms.tangential_viscosity) {
        for (std::size_t first = 0U; first < count; ++first) {
            for (std::size_t second = first + 1U; second < count; ++second) {
                ++result.pair_tests;
                const Vec3l accepted_difference =
                    accepted.dynamic[first].position
                    - accepted.dynamic[second].position;
                const Vec3l graph_difference = options.mutation
                        == Mutation15::CurrentReferenceViscosityGraph
                    ? positions[first] - positions[second]
                    : accepted_difference;
                const long double radius = norm15(graph_difference);
                if (!(radius > 0.0L)) return result;
                if (radius <= options.profile.horizon) {
                    ++result.viscosity_pairs;
                }
            }
        }
    }
    if (options.terms.surface) {
        for (std::size_t first = 0U; first < count; ++first) {
            for (std::size_t second = first + 1U; second < count; ++second) {
                ++result.pair_tests;
                ++result.surface_tests;
                const long double radius = norm15(
                    positions[first] - positions[second]);
                if (!(radius > 0.0L)) return result;
                if (radius < 3.0L * options.profile.surface_r0) {
                    ++result.surface_active;
                }
            }
        }
    }
    return result;
}

std::uint64_t stage_count15(const StepTrace15& trace,
    std::string_view stage) {
    return static_cast<std::uint64_t>(std::count(
        trace.reached_stages.begin(), trace.reached_stages.end(), stage));
}

Work15 expected_oracle_work15(const State15& accepted,
    const SolverOptions15& options, const StepTrace15& trace) {
    Work15 expected;
    expected.fixture_records = accepted.dynamic.size();
    expected.ghost_records = accepted.ghosts.size();
    expected.scalar_predicates = kProfilePredicateCount15
        + kOptionPredicateCount15 + canonical_predicate_count15(accepted)
        + 1U;
    for (std::size_t evaluation_index = 0U;
         evaluation_index < trace.evaluation_positions.size();
         ++evaluation_index) {
        const std::vector<Vec3l>& evaluation =
            trace.evaluation_positions[evaluation_index];
        const std::vector<long double>& multipliers = evaluation_index
                < trace.evaluation_multipliers.size()
            ? trace.evaluation_multipliers[evaluation_index]
            : std::vector<long double>{};
        const bool gradient = evaluation_index
                < trace.evaluation_gradient.size()
            && trace.evaluation_gradient[evaluation_index] != 0U;
        expected.scalar_predicates += kProfilePredicateCount15
            + canonical_predicate_count15(accepted) + 2U
            + evaluation.size() + multipliers.size();
        const bool evaluation_valid = evaluation.size()
                == accepted.dynamic.size()
            && multipliers.size() == accepted.dynamic.size()
            && finite_vector15(evaluation) && finite_scalars15(multipliers);
        if (!evaluation_valid) continue;
        const OracleEnumeration15 counts = enumerate_expected_evaluation15(
            accepted, evaluation, options);
        ++expected.objective_evaluations;
        expected.scalar_predicates += canonical_predicate_count15(accepted)
            + 1U + evaluation.size();
        ++expected.graph_builds;
        expected.oracle_pair_tests += counts.pair_tests;
        expected.accepted_pairs += counts.accepted_density_pairs;
        expected.density_pairs += counts.accepted_density_pairs;
        if (gradient) ++expected.gradient_evaluations;
        if (!counts.density_complete) continue;
        expected.viscosity_pairs += counts.viscosity_pairs;
        expected.surface_pair_tests += counts.surface_tests;
        expected.surface_active_pairs += counts.surface_active;
        if (gradient) {
            expected.jacobian_products += counts.accepted_density_pairs
                - accepted.dynamic.size();
        }
    }
    const std::uint64_t projections = stage_count15(trace, "PROJECTION");
    const std::uint64_t box_projections =
        stage_count15(trace, "PROJECTION_BOX");
    const std::uint64_t inner_gates = stage_count15(trace, "INNER_GATE");
    const std::uint64_t final_gates = stage_count15(trace, "FINAL_GATE");
    const std::uint64_t contact_batches = stage_count15(trace, "CONTACT");
    expected.projected_directions = accepted.dynamic.size()
        * (inner_gates + final_gates);
    expected.line_search_trials = stage_count15(trace, "LINE_TRIAL");
    expected.accepted_inner_iterations = trace.accepted_inner_iterations;
    expected.outer_multiplier_updates = trace.outer_updates;
    if (options.boundary_mode == BoundaryMode15::AnalyticBox) {
        expected.box_plane_tests = 6U * accepted.dynamic.size()
            * (box_projections + final_gates + contact_batches);
        expected.contact_reaction_values = 3U * accepted.dynamic.size()
            * contact_batches;
    }
    if (stage_count15(trace, "TOPOLOGY") != 0U
        && !trace.evaluation_positions.empty()) {
        const std::vector<Vec3l>& final_positions =
            trace.evaluation_positions.back();
        ++expected.graph_builds;
        for (std::size_t owner = 0U; owner < accepted.dynamic.size(); ++owner) {
            for (std::size_t neighbor = 0U;
                 neighbor < accepted.dynamic.size(); ++neighbor) {
                ++expected.oracle_pair_tests;
                const long double radius = norm15(
                    final_positions[owner] - final_positions[neighbor]);
                if (radius > options.profile.horizon) continue;
                ++expected.accepted_pairs;
                ++expected.topology_edges;
            }
            for (const Ghost15& ghost : accepted.ghosts) {
                ++expected.oracle_pair_tests;
                if (norm15(final_positions[owner] - ghost.position)
                        <= options.profile.horizon) {
                    ++expected.accepted_pairs;
                }
            }
        }
    }
    expected.scalar_predicates += 2U * inner_gates
        + stage_count15(trace, "SLOPE") + stage_count15(trace, "ARMIJO")
        + accepted.dynamic.size() * projections
        + accepted.dynamic.size()
            * stage_count15(trace, "MULTIPLIER_UPDATE")
        + (accepted.dynamic.size() + 8U
            + (options.boundary_mode == BoundaryMode15::AnalyticBox
                    ? 1U : 0U))
            * final_gates;
    return expected;
}

void topology_work15(const State15& accepted,
    const std::vector<Vec3l>& positions, const SolverOptions15& options,
    Work15& work) {
    ++work.graph_builds;
    for (std::size_t owner = 0U; owner < accepted.dynamic.size(); ++owner) {
        for (std::size_t neighbor = 0U;
             neighbor < accepted.dynamic.size(); ++neighbor) {
            ++work.oracle_pair_tests;
            const long double radius = norm15(
                positions[owner] - positions[neighbor]);
            if (radius > options.profile.horizon) continue;
            ++work.accepted_pairs;
            ++work.topology_edges;
        }
        for (const Ghost15& ghost : accepted.ghosts) {
            ++work.oracle_pair_tests;
            if (norm15(positions[owner] - ghost.position)
                    <= options.profile.horizon) {
                ++work.accepted_pairs;
            }
        }
    }
}

Membership15 membership15(const State15& accepted,
    const std::vector<Vec3l>& positions, const SolverOptions15& options,
    Work15& work) {
    Membership15 result;
    const std::size_t count = accepted.dynamic.size();
    for (std::size_t owner = 0U; owner < count; ++owner) {
        for (std::size_t neighbor = 0U; neighbor < count; ++neighbor) {
            ++work.oracle_pair_tests;
            const long double radius = norm15(
                positions[owner] - positions[neighbor]);
            if (neighbor != owner && !(radius > 0.0L)) return result;
            result.flags.push_back(radius <= options.profile.horizon ? 1U : 0U);
        }
        for (const Ghost15& ghost : accepted.ghosts) {
            ++work.oracle_pair_tests;
            const long double radius = norm15(positions[owner] - ghost.position);
            if (!(radius > 0.0L)) return result;
            result.flags.push_back(radius <= options.profile.horizon ? 1U : 0U);
        }
    }
    if (options.terms.surface) {
        for (std::size_t first = 0U; first < count; ++first) {
            for (std::size_t second = first + 1U; second < count; ++second) {
                ++work.surface_pair_tests;
                ++work.oracle_pair_tests;
                const long double radius = norm15(
                    positions[first] - positions[second]);
                if (!(radius > 0.0L)) return result;
                result.flags.push_back(
                    radius < 3.0L * options.profile.surface_r0 ? 1U : 0U);
            }
        }
    }
    result.valid = true;
    return result;
}

Work15 expected_oracle_evaluation_work15(const State15& accepted,
    const std::vector<Vec3l>& positions,
    const std::vector<long double>& multipliers,
    const SolverOptions15& options, bool need_gradient) {
    Work15 expected;
    expected.scalar_predicates = kProfilePredicateCount15
        + canonical_predicate_count15(accepted) + 2U
        + positions.size() + multipliers.size();
    if (positions.size() != accepted.dynamic.size()
            || multipliers.size() != accepted.dynamic.size()
            || !finite_vector15(positions)
            || !finite_scalars15(multipliers)) {
        return expected;
    }
    ++expected.objective_evaluations;
    if (need_gradient) ++expected.gradient_evaluations;
    expected.scalar_predicates += canonical_predicate_count15(accepted)
        + 1U + positions.size();
    const OracleEnumeration15 counts = enumerate_expected_evaluation15(
        accepted, positions, options);
    ++expected.graph_builds;
    expected.oracle_pair_tests += counts.pair_tests;
    expected.accepted_pairs += counts.accepted_density_pairs;
    expected.density_pairs += counts.accepted_density_pairs;
    if (!counts.density_complete) return expected;
    expected.viscosity_pairs += counts.viscosity_pairs;
    expected.surface_pair_tests += counts.surface_tests;
    expected.surface_active_pairs += counts.surface_active;
    if (need_gradient) {
        expected.jacobian_products += counts.accepted_density_pairs
            - accepted.dynamic.size();
    }
    return expected;
}

Work15 expected_gradient_check_work15(const State15& accepted,
    const std::vector<Vec3l>& positions,
    const std::vector<long double>& multipliers,
    const SolverOptions15& options,
    const std::vector<Vec3l>& analytical_gradient) {
    Work15 expected;
    const std::size_t count = accepted.dynamic.size();
    expected.scalar_predicates = kProfilePredicateCount15
        + canonical_predicate_count15(accepted) + 3U + positions.size()
        + multipliers.size() + analytical_gradient.size();
    if (positions.size() != count || multipliers.size() != count
            || analytical_gradient.size() != count
            || !finite_vector15(positions)
            || !finite_scalars15(multipliers)
            || !finite_vector15(analytical_gradient)) {
        return expected;
    }
    const auto add_membership = [&](const std::vector<Vec3l>& point) {
        static_cast<void>(point);
        expected.oracle_pair_tests += count
            * (count + accepted.ghosts.size());
        if (options.terms.surface) {
            const std::uint64_t pairs = count * (count - 1U) / 2U;
            expected.oracle_pair_tests += pairs;
            expected.surface_pair_tests += pairs;
        }
    };
    add_membership(positions);
    ++expected.scalar_predicates;
    const long double epsilon = options.profile.spacing
        * options.profile.finite_difference_scale;
    for (std::size_t particle = 0U; particle < count; ++particle) {
        for (std::size_t axis = 0U; axis < 3U; ++axis) {
            std::vector<Vec3l> plus = positions;
            std::vector<Vec3l> minus = positions;
            long double* const plus_value = axis == 0U ? &plus[particle].x
                : axis == 1U ? &plus[particle].y : &plus[particle].z;
            long double* const minus_value = axis == 0U ? &minus[particle].x
                : axis == 1U ? &minus[particle].y : &minus[particle].z;
            *plus_value += epsilon;
            *minus_value -= epsilon;
            if (options.boundary_mode == BoundaryMode15::AnalyticBox) {
                expected.box_plane_tests += 4U;
            }
            add_membership(plus);
            add_membership(minus);
            expected.scalar_predicates += 4U;
            expected.finite_difference_evaluations += 2U;
            add_work15(expected, expected_oracle_evaluation_work15(
                accepted, plus, multipliers, options, false));
            add_work15(expected, expected_oracle_evaluation_work15(
                accepted, minus, multipliers, options, false));
        }
    }
    expected.scalar_predicates += 2U;
    return expected;
}

} // namespace

Profile15 frozen_profile15() {
    return Profile15{};
}

Density15 oracle_density15(const State15& accepted,
    const std::vector<Vec3l>& positions, const SolverOptions15& options,
    Work15& work) {
    if (!valid_profile15(options.profile, &work)) {
        Density15 result;
        result.outcome = "INVALID_PROFILE";
        return result;
    }
    return density_payload15(accepted, positions, options, work).public_value;
}

Work15 oracle_expected_density_work15(const State15& accepted,
    const std::vector<Vec3l>& positions, const SolverOptions15& options) {
    Work15 expected;
    expected.scalar_predicates = kProfilePredicateCount15;
    const bool profile_valid = valid_profile15(options.profile);
    if (!profile_valid) return expected;
    expected.scalar_predicates += canonical_predicate_count15(accepted)
        + 1U + positions.size();
    const bool canonical = canonical_state15(accepted);
    if (!canonical
            || positions.size() != accepted.dynamic.size()
            || !finite_vector15(positions)) {
        return expected;
    }
    ++expected.graph_builds;
    for (std::size_t owner = 0U; owner < positions.size(); ++owner) {
        std::uint32_t degree = 0U;
        for (std::size_t neighbor = 0U; neighbor < positions.size();
             ++neighbor) {
            ++expected.oracle_pair_tests;
            const long double radius = norm15(
                positions[owner] - positions[neighbor]);
            if (neighbor != owner && !(radius > 0.0L)) return expected;
            if (radius > options.profile.horizon) continue;
            ++expected.accepted_pairs;
            ++expected.density_pairs;
            ++degree;
        }
        for (const Ghost15& ghost : accepted.ghosts) {
            ++expected.oracle_pair_tests;
            const long double radius = norm15(
                positions[owner] - ghost.position);
            if (!(radius > 0.0L)) return expected;
            if (radius > options.profile.horizon) continue;
            ++expected.accepted_pairs;
            ++expected.density_pairs;
            ++degree;
        }
        if (degree > options.profile.maximum_neighbors) return expected;
    }
    return expected;
}

EnergyGradient15 oracle_energy_gradient15(const State15& accepted,
    const std::vector<Vec3l>& positions,
    const std::vector<long double>& multipliers,
    const SolverOptions15& options, MultiplierMode15 multiplier_mode,
    bool need_gradient, Work15& work) {
    return energy_gradient_impl15(accepted, positions, multipliers, options,
        multiplier_mode, need_gradient, work);
}

Step15 oracle_step15(const State15& input,
    const SolverOptions15& options) {
    Step15 result;
    const State15 accepted_state = sorted_state15(input);
    result.state = accepted_state;
    result.work.fixture_records = accepted_state.dynamic.size();
    result.work.ghost_records = accepted_state.ghosts.size();
    const auto close_expected = [&]() {
        result.expected_work = expected_oracle_work15(
            accepted_state, options, result.trace);
    };
    const bool profile_valid = valid_profile15(options.profile, &result.work);
    const bool gravity_mode_valid = counted_predicate15(result.work,
        options.gravity_mode == GravityMode15::Profile
            || options.gravity_mode == GravityMode15::Zero);
    const bool boundary_mode_valid = counted_predicate15(result.work,
        options.boundary_mode == BoundaryMode15::AnalyticBox
            || options.boundary_mode == BoundaryMode15::UnboundedManufactured);
    const bool mutation_valid = counted_predicate15(result.work,
        options.mutation >= Mutation15::None
            && options.mutation <= Mutation15::FinitePressurePenalty);
    const bool canonical = canonical_state15(accepted_state, &result.work);
    const bool capacity = counted_predicate15(result.work,
        accepted_state.dynamic.size()
            <= options.profile.maximum_dynamic_samples);
    if (!profile_valid || !gravity_mode_valid || !boundary_mode_valid
        || !mutation_valid) {
        set_step_failure15(result, "INVALID_PROFILE", false, false);
        close_expected();
        return result;
    }
    if (!capacity || !canonical) {
        set_step_failure15(result, "INVALID_STATE", false, false);
        close_expected();
        return result;
    }
    const std::size_t count = accepted_state.dynamic.size();
    std::vector<Vec3l> predictor(count);
    const Vec3l gravity = selected_gravity15(options);
    for (std::size_t index = 0U; index < count; ++index) {
        const Dynamic15& item = accepted_state.dynamic[index];
        predictor[index] = item.position
            + (item.velocity + gravity * options.profile.dt)
                * options.profile.dt;
    }
    std::vector<Vec3l> trial;
    const bool predictor_projection_valid = project_all15(
        predictor, options, trial, result.work);
    result.trace.reached_stages.push_back("PROJECTION");
    if (predictor_projection_valid
            && options.boundary_mode == BoundaryMode15::AnalyticBox) {
        result.trace.reached_stages.push_back("PROJECTION_BOX");
    }
    if (!predictor_projection_valid) {
        set_step_failure15(result, "NONFINITE", false, false);
        close_expected();
        return result;
    }
    result.pi.assign(count, 0.0L);
    EnergyGradient15 last_final;
    const auto evaluate_recorded = [&](const std::vector<Vec3l>& point,
                                       MultiplierMode15 mode,
                                       bool need_gradient) {
        result.trace.evaluation_positions.push_back(point);
        result.trace.evaluation_multipliers.push_back(result.pi);
        result.trace.evaluation_modes.push_back(mode
                == MultiplierMode15::FinalKkt
            ? std::uint8_t{1U} : std::uint8_t{0U});
        result.trace.evaluation_gradient.push_back(
            need_gradient ? std::uint8_t{1U} : std::uint8_t{0U});
        return oracle_energy_gradient15(accepted_state, point, result.pi,
            options, mode, need_gradient, result.work);
    };
    for (std::uint32_t outer = 0U;
         outer < options.profile.maximum_outer_updates; ++outer) {
        EnergyGradient15 converged_inner;
        while (true) {
            EnergyGradient15 current = evaluate_recorded(trial,
                MultiplierMode15::AugmentedLagrangian, true);
            if (!current.valid) {
                set_step_failure15(result, current.outcome, false, false);
                publish_private_state15(
                    result, accepted_state, trial, nullptr, options);
                close_expected();
                return result;
            }
            const long double scale = options.profile.dt * options.profile.dt
                / options.profile.mass;
            std::vector<Vec3l> proposal(count);
            for (std::size_t index = 0U; index < count; ++index) {
                proposal[index] = trial[index] - current.gradient[index] * scale;
            }
            std::vector<Vec3l> projected;
            const bool projection_valid = project_all15(
                proposal, options, projected, result.work);
            result.trace.reached_stages.push_back("PROJECTION");
            if (projection_valid
                    && options.boundary_mode == BoundaryMode15::AnalyticBox) {
                result.trace.reached_stages.push_back("PROJECTION_BOX");
            }
            if (!projection_valid) {
                set_step_failure15(result, "NONFINITE", false, false);
                publish_private_state15(
                    result, accepted_state, trial, &current, options);
                close_expected();
                return result;
            }
            result.work.projected_directions += count;
            result.trace.reached_stages.push_back("INNER_GATE");
            std::vector<Vec3l> direction(count);
            for (std::size_t index = 0U; index < count; ++index) {
                direction[index] = projected[index] - trial[index];
            }
            const long double relative = vector_norm15(direction)
                / std::max(options.profile.spacing
                    * std::sqrt(3.0L * static_cast<long double>(count)),
                    kTinyDenominator);
            const long double maximum = maximum_abs_component15(direction)
                / options.profile.spacing;
            const bool relative_converged = counted_predicate15(result.work,
                relative <= options.profile.gates.inner_rms_relative);
            const bool maximum_converged = counted_predicate15(result.work,
                maximum <= options.profile.gates.inner_max_relative);
            if (relative_converged && maximum_converged) {
                converged_inner = std::move(current);
                break;
            }
            std::vector<long double> slope_terms;
            slope_terms.reserve(count);
            for (std::size_t index = 0U; index < count; ++index) {
                slope_terms.push_back(dot15(current.gradient[index],
                    direction[index]));
            }
            const long double slope = pairwise_sum15(std::move(slope_terms));
            result.trace.reached_stages.push_back("SLOPE");
            if (counted_predicate15(result.work, !(slope < 0.0L))) {
                set_step_failure15(result, "NON_DESCENT", false, false);
                publish_private_state15(
                    result, accepted_state, trial, &current, options);
                close_expected();
                return result;
            }
            if (result.trace.accepted_inner_iterations
                    >= options.profile.maximum_accepted_inner_iterations) {
                set_step_failure15(result,
                    "SOLVER_WORK_CEILING_INCONCLUSIVE", true, true);
                publish_private_state15(
                    result, accepted_state, trial, &current, options);
                close_expected();
                return result;
            }
            long double alpha = options.profile.initial_alpha;
            bool accepted_line = false;
            std::uint32_t accepted_backtracks = 0U;
            std::vector<Vec3l> line_trial(count);
            long double accepted_objective = 0.0L;
            for (std::uint32_t backtrack = 0U;
                 backtrack <= options.profile.maximum_line_search_backtracks;
                 ++backtrack) {
                for (std::size_t index = 0U; index < count; ++index) {
                    line_trial[index] = trial[index]
                        + direction[index] * alpha;
                }
                ++result.work.line_search_trials;
                result.trace.reached_stages.push_back("LINE_TRIAL");
                EnergyGradient15 line = evaluate_recorded(line_trial,
                    MultiplierMode15::AugmentedLagrangian, false);
                if (!line.valid) {
                    set_step_failure15(result, line.outcome, false, false);
                    publish_private_state15(result, accepted_state,
                        line_trial, nullptr, options);
                    close_expected();
                    return result;
                }
                const long double bound = current.objective
                    + options.profile.armijo * alpha * slope;
                result.trace.reached_stages.push_back("ARMIJO");
                const bool armijo_pass = counted_predicate15(result.work,
                    !(line.objective > bound));
                if (armijo_pass) {
                    accepted_line = true;
                    accepted_backtracks = backtrack;
                    accepted_objective = line.objective;
                    break;
                }
                if (backtrack
                        == options.profile.maximum_line_search_backtracks) {
                    break;
                }
                alpha *= options.profile.line_search_shrink;
            }
            if (!accepted_line) {
                set_step_failure15(result,
                    "LINE_SEARCH_EXHAUSTED", false, false);
                publish_private_state15(
                    result, accepted_state, trial, &current, options);
                close_expected();
                return result;
            }
            trial = line_trial;
            ++result.trace.accepted_inner_iterations;
            ++result.work.accepted_inner_iterations;
            result.trace.backtracks.push_back(accepted_backtracks);
            result.trace.accepted_objectives.push_back(accepted_objective);
        }

        result.trace.reached_stages.push_back("MULTIPLIER_UPDATE");
        for (std::size_t index = 0U; index < count; ++index) {
            const long double next = options.mutation
                        == Mutation15::FinitePressurePenalty
                    || !options.terms.density_constraint
                ? 0.0L
                : std::max(0.0L, result.pi[index]
                    + options.profile.beta
                        * converged_inner.density.constraint[index]);
            result.pi[index] = next;
            static_cast<void>(counted_predicate15(
                result.work, finite_scalar15(next)));
        }
        ++result.trace.outer_updates;
        ++result.work.outer_multiplier_updates;
        result.trace.outer_inner_offsets.push_back(
            result.trace.accepted_inner_iterations);
        last_final = evaluate_recorded(
            trial, MultiplierMode15::FinalKkt, true);
        if (!last_final.valid) {
            set_step_failure15(result, last_final.outcome, false, false);
            publish_private_state15(
                result, accepted_state, trial, nullptr, options);
            close_expected();
            return result;
        }
        result.state = state_from_trial15(
            accepted_state, trial, options.profile);
        result.rho = last_final.density.rho;
        result.dynamic_pressure_force = last_final.dynamic_pressure_force;
        result.ghost_pressure_force = last_final.ghost_pressure_force;
        result.viscosity_force = last_final.viscosity_force;
        result.surface_force = last_final.surface_force;
        result.active_multiplier_ids.clear();
        if (final_kkt_gates15(
                result, last_final, trial, options, result.trace)) {
            topology_work15(
                accepted_state, trial, options, result.work);
            result.trace.reached_stages.push_back("TOPOLOGY");
            result.outcome = "PASS";
            result.apparatus_valid = true;
            result.accepted = true;
            result.work_ceiling = false;
            close_expected();
            return result;
        }
        if (result.outcome == "NONFINITE") {
            set_step_failure15(result, "NONFINITE", false, false);
            publish_private_state15(
                result, accepted_state, trial, &last_final, options);
            close_expected();
            return result;
        }
        if (result.trace.outer_updates
                == options.profile.maximum_outer_updates) {
            set_step_failure15(result,
                "SOLVER_WORK_CEILING_INCONCLUSIVE", true, true);
            close_expected();
            return result;
        }
    }
    set_step_failure15(result,
        "SOLVER_WORK_CEILING_INCONCLUSIVE", true, true);
    close_expected();
    return result;
}

GradientCheck15 oracle_central_difference_gradient15(
    const State15& accepted, const std::vector<Vec3l>& positions,
    const std::vector<long double>& multipliers,
    const SolverOptions15& options,
    const std::vector<Vec3l>& analytical_gradient) {
    GradientCheck15 result;
    result.outcome = "FINITE_DIFFERENCE_INVALID";
    const std::size_t count = accepted.dynamic.size();
    result.expected_work = expected_gradient_check_work15(accepted,
        positions, multipliers, options, analytical_gradient);
    const bool profile_valid = valid_profile15(options.profile, &result.work);
    const bool canonical = canonical_state15(accepted, &result.work);
    const bool positions_size = counted_predicate15(
        result.work, positions.size() == count);
    const bool multipliers_size = counted_predicate15(
        result.work, multipliers.size() == count);
    const bool analytical_size = counted_predicate15(
        result.work, analytical_gradient.size() == count);
    bool finite_inputs = true;
    for (const Vec3l position : positions) {
        finite_inputs = counted_predicate15(result.work, finite15(position))
            && finite_inputs;
    }
    for (const long double multiplier : multipliers) {
        finite_inputs = counted_predicate15(
            result.work, finite_scalar15(multiplier)) && finite_inputs;
    }
    for (const Vec3l gradient : analytical_gradient) {
        finite_inputs = counted_predicate15(result.work, finite15(gradient))
            && finite_inputs;
    }
    if (!profile_valid || !canonical || !positions_size || !multipliers_size
        || !analytical_size || !finite_inputs) {
        return result;
    }
    const Membership15 base = membership15(
        accepted, positions, options, result.work);
    if (!counted_predicate15(result.work, base.valid)) return result;
    result.finite_difference.assign(count, {});
    const long double epsilon = options.profile.spacing
        * options.profile.finite_difference_scale;
    for (std::size_t particle = 0U; particle < count; ++particle) {
        for (std::size_t axis = 0U; axis < 3U; ++axis) {
            std::vector<Vec3l> plus = positions;
            std::vector<Vec3l> minus = positions;
            long double* const plus_value = axis == 0U ? &plus[particle].x
                : axis == 1U ? &plus[particle].y : &plus[particle].z;
            long double* const minus_value = axis == 0U ? &minus[particle].x
                : axis == 1U ? &minus[particle].y : &minus[particle].z;
            *plus_value += epsilon;
            *minus_value -= epsilon;
            if (options.boundary_mode == BoundaryMode15::AnalyticBox) {
                const long double lower = 0.5L * options.profile.spacing;
                const std::array<long double, 3> upper{
                    options.profile.basin.x - lower,
                    options.profile.basin.y - lower,
                    options.profile.basin.z - lower};
                result.work.box_plane_tests += 4U;
                if (!(*minus_value > lower && *plus_value < upper[axis])) {
                    return result;
                }
            }
            const Membership15 plus_membership = membership15(
                accepted, plus, options, result.work);
            const Membership15 minus_membership = membership15(
                accepted, minus, options, result.work);
            const bool plus_valid = counted_predicate15(
                result.work, plus_membership.valid);
            const bool minus_valid = counted_predicate15(
                result.work, minus_membership.valid);
            const bool plus_same = counted_predicate15(
                result.work, plus_membership.flags == base.flags);
            const bool minus_same = counted_predicate15(
                result.work, minus_membership.flags == base.flags);
            if (!plus_valid || !minus_valid || !plus_same || !minus_same) {
                return result;
            }
            ++result.work.finite_difference_evaluations;
            const EnergyGradient15 plus_value_result =
                oracle_energy_gradient15(accepted, plus, multipliers, options,
                    MultiplierMode15::AugmentedLagrangian, false, result.work);
            ++result.work.finite_difference_evaluations;
            const EnergyGradient15 minus_value_result =
                oracle_energy_gradient15(accepted, minus, multipliers, options,
                    MultiplierMode15::AugmentedLagrangian, false, result.work);
            if (!plus_value_result.valid || !minus_value_result.valid) {
                return result;
            }
            const long double derivative = (plus_value_result.objective
                - minus_value_result.objective) / (2.0L * epsilon);
            if (axis == 0U) result.finite_difference[particle].x = derivative;
            else if (axis == 1U) {
                result.finite_difference[particle].y = derivative;
            } else {
                result.finite_difference[particle].z = derivative;
            }
        }
    }
    std::vector<Vec3l> difference(count);
    for (std::size_t index = 0U; index < count; ++index) {
        difference[index] = analytical_gradient[index]
            - result.finite_difference[index];
    }
    result.relative_l2 = vector_norm15(difference)
        / std::max(vector_norm15(result.finite_difference), 1.0L);
    result.maximum_absolute_n = maximum_abs_component15(difference);
    const bool relative_pass = counted_predicate15(result.work,
        result.relative_l2 <= options.profile.gates.gradient_relative_l2);
    const bool maximum_pass = counted_predicate15(result.work,
        result.maximum_absolute_n
            <= options.profile.gates.gradient_maximum_n);
    result.valid = relative_pass && maximum_pass;
    result.outcome = result.valid ? "PASS" : "GRADIENT_MISMATCH";
    return result;
}

long double oracle_surface_pair_root15(long double initial_separation,
    const SolverOptions15& options) {
    if (!valid_profile15(options.profile)
        || !(initial_separation > 0.0L)
        || !(initial_separation < 3.0L * options.profile.surface_r0)) {
        throw std::invalid_argument("invalid NCGP15 surface-pair root input");
    }
    const Profile15& profile = options.profile;
    if (initial_separation == profile.surface_r0) return initial_separation;
    const long double surface_factor = 2.0L * profile.gamma
        * profile.mass * profile.mass;
    const auto equation = [&](long double radius) {
        return profile.mass / (2.0L * profile.dt * profile.dt)
                * (radius - initial_separation)
            + surface_factor
                * surface_coefficient15(radius / profile.surface_r0);
    };
    long double lower = std::min(initial_separation, profile.surface_r0);
    long double upper = std::max(initial_separation, profile.surface_r0);
    long double lower_value = equation(lower);
    long double upper_value = equation(upper);
    if (!(lower_value <= 0.0L && upper_value >= 0.0L)) {
        throw std::runtime_error("NCGP15 surface-pair root is not bracketed");
    }
    for (std::uint32_t iteration = 0U; iteration < 256U; ++iteration) {
        const long double middle = 0.5L * (lower + upper);
        const long double value = equation(middle);
        if (value <= 0.0L) {
            lower = middle;
            lower_value = value;
        } else {
            upper = middle;
            upper_value = value;
        }
    }
    static_cast<void>(lower_value);
    static_cast<void>(upper_value);
    return 0.5L * (lower + upper);
}

std::string_view gravity_mode_name15(GravityMode15 mode) {
    switch (mode) {
    case GravityMode15::Profile: return "PROFILE_GRAVITY";
    case GravityMode15::Zero: return "ZERO_GRAVITY";
    }
    return "INVALID_GRAVITY_MODE";
}

std::string_view boundary_mode_name15(BoundaryMode15 mode) {
    switch (mode) {
    case BoundaryMode15::AnalyticBox: return "ANALYTIC_BOX";
    case BoundaryMode15::UnboundedManufactured:
        return "UNBOUNDED_MANUFACTURED";
    }
    return "INVALID_BOUNDARY_MODE";
}

std::string_view mutation_name15(Mutation15 mutation) {
    switch (mutation) {
    case Mutation15::None: return "NONE";
    case Mutation15::MissingKernelChain: return "MISSING_KERNEL_CHAIN";
    case Mutation15::HalfNormalViscosity: return "HALF_NORMAL_VISCOSITY";
    case Mutation15::WrongSurfaceSign: return "WRONG_SURFACE_SIGN";
    case Mutation15::MissingSurfaceFactorTwo:
        return "MISSING_SURFACE_FACTOR_TWO";
    case Mutation15::CurrentReferenceViscosityGraph:
        return "CURRENT_REFERENCE_VISCOSITY_GRAPH";
    case Mutation15::FinitePressurePenalty: return "FINITE_PRESSURE_PENALTY";
    }
    return "INVALID_MUTATION";
}

} // namespace nextengine::nonlocal::ncgp15
