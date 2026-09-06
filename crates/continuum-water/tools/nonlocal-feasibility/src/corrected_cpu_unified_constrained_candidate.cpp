#include "corrected_cpu_unified_constrained.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <limits>
#include <map>
#include <set>
#include <tuple>
#include <utility>

#if __cplusplus != 201703L
#error "NCGP15 requires exact ISO C++17 mode"
#endif

namespace nextengine::nonlocal::ncgp15 {
namespace {

struct Kernel15 {
    long double value = 0.0L;
    long double derivative = 0.0L;
};

struct Neighbor15 {
    std::uint32_t id = 0U;
    std::size_t index = 0U;
    bool ghost = false;
};

struct Graph15 {
    bool valid = true;
    bool capacity_overflow = false;
    std::uint32_t maximum_degree = 0U;
    std::vector<std::vector<Neighbor15>> rows;
    std::vector<std::pair<std::size_t, std::size_t>> dynamic_pairs;
};

struct ReferencePair15 {
    std::size_t first = 0U;
    std::size_t second = 0U;
    Vec3l normal;
    long double omega = 0.0L;
};

struct Cell15 {
    std::int64_t x = 0;
    std::int64_t y = 0;
    std::int64_t z = 0;
    friend bool operator<(const Cell15& lhs, const Cell15& rhs) {
        return std::tie(lhs.x, lhs.y, lhs.z)
            < std::tie(rhs.x, rhs.y, rhs.z);
    }
};

bool finite_scalar15(long double value) {
    return std::isfinite(value);
}

bool finite_positions15(const std::vector<Vec3l>& positions) {
    bool result = true;
    for (const Vec3l position : positions) {
        result = finite15(position) && result;
    }
    return result;
}

bool counted15(Work15& work, bool value);

constexpr std::uint64_t kProfilePredicateCount15 = 55U;
constexpr std::uint64_t kOptionPredicateCount15 = 3U;

bool valid_profile15(const SolverOptions15& options, Work15& work) {
    const Profile15 frozen = frozen_profile15();
    bool valid = true;
    const auto check = [&](bool value) {
        valid = counted15(work, value) && valid;
    };
    check(options.profile.id == frozen.id);
    for (const auto& values : std::array<std::pair<long double, long double>, 21>{
             {{options.profile.dt, frozen.dt},
                 {options.profile.spacing, frozen.spacing},
                 {options.profile.horizon, frozen.horizon},
                 {options.profile.surface_r0, frozen.surface_r0},
                 {options.profile.mass, frozen.mass},
                 {options.profile.rho0, frozen.rho0},
                 {options.profile.kernel_scale, frozen.kernel_scale},
                 {options.profile.beta, frozen.beta},
                 {options.profile.lambda_v, frozen.lambda_v},
                 {options.profile.mu_v, frozen.mu_v},
                 {options.profile.gamma, frozen.gamma},
                 {options.profile.gravity.x, frozen.gravity.x},
                 {options.profile.gravity.y, frozen.gravity.y},
                 {options.profile.gravity.z, frozen.gravity.z},
                 {options.profile.basin.x, frozen.basin.x},
                 {options.profile.basin.y, frozen.basin.y},
                 {options.profile.basin.z, frozen.basin.z},
                 {static_cast<long double>(options.profile.ghost_layers),
                     static_cast<long double>(frozen.ghost_layers)},
                 {static_cast<long double>(
                      options.profile.maximum_dynamic_samples),
                     static_cast<long double>(
                         frozen.maximum_dynamic_samples)},
                 {static_cast<long double>(options.profile.maximum_neighbors),
                     static_cast<long double>(frozen.maximum_neighbors)},
                 {static_cast<long double>(
                      options.profile.maximum_outer_updates),
                     static_cast<long double>(
                         frozen.maximum_outer_updates)}}}) {
        check(values.first == values.second);
    }
    check(options.profile.maximum_accepted_inner_iterations
        == frozen.maximum_accepted_inner_iterations);
    check(options.profile.maximum_line_search_backtracks
        == frozen.maximum_line_search_backtracks);
    for (const auto& values : std::array<std::pair<long double, long double>, 4>{
             {{options.profile.armijo, frozen.armijo},
                 {options.profile.initial_alpha, frozen.initial_alpha},
                 {options.profile.line_search_shrink,
                     frozen.line_search_shrink},
                 {options.profile.finite_difference_scale,
                     frozen.finite_difference_scale}}}) {
        check(values.first == values.second);
    }
    const auto gate_values = [](const Gates15& gates) {
        return std::array<long double, 27>{gates.inner_rms_relative,
            gates.inner_max_relative, gates.density_maximum, gates.density_rms,
            gates.kkt_rms_m, gates.kkt_maximum_m, gates.complementarity,
            gates.multiplier_fixed_point, gates.density_relative_l2,
            gates.gradient_relative_l2, gates.gradient_maximum_n,
            gates.pair_position_m, gates.normal_decay_relative,
            gates.tangential_change_absolute,
            gates.center_of_mass_relative,
            gates.surface_energy_relative_excess,
            gates.surface_energy_absolute_excess_j,
            gates.reversible_energy_drift,
            gates.one_step_position_rms_m,
            gates.one_step_position_maximum_m,
            gates.trajectory_position_rms_m,
            gates.trajectory_position_maximum_m,
            gates.trajectory_velocity_rms_mps,
            gates.trajectory_velocity_maximum_mps,
            gates.momentum_residual, gates.energy_excess,
            gates.internal_force_closure};
    };
    const auto actual_gates = gate_values(options.profile.gates);
    const auto frozen_gates = gate_values(frozen.gates);
    for (std::size_t index = 0U; index < actual_gates.size(); ++index) {
        check(actual_gates[index] == frozen_gates[index]);
    }
    check(options.gravity_mode == GravityMode15::Profile
        || options.gravity_mode == GravityMode15::Zero);
    check(options.boundary_mode == BoundaryMode15::AnalyticBox
        || options.boundary_mode == BoundaryMode15::UnboundedManufactured);
    check(options.mutation >= Mutation15::None
        && options.mutation <= Mutation15::FinitePressurePenalty);
    return valid;
}

Cell15 cell_of15(Vec3l position, long double width) {
    return {static_cast<std::int64_t>(std::floor(position.x / width)),
        static_cast<std::int64_t>(std::floor(position.y / width)),
        static_cast<std::int64_t>(std::floor(position.z / width))};
}

Kernel15 kernel15(long double distance, const SolverOptions15& options) {
    const Profile15& profile = options.profile;
    if (!(distance >= 0.0L) || distance > profile.horizon) return {};
    const long double q = 2.0L * distance / profile.horizon;
    const long double alpha = profile.kernel_scale * 3.0L
        / (2.0L * std::acos(-1.0L) * profile.horizon * profile.horizon
            * profile.horizon);
    const long double chain = options.mutation == Mutation15::MissingKernelChain
        ? 1.0L : 2.0L / profile.horizon;
    if (q < 1.0L) {
        return {alpha * (2.0L / 3.0L - q * q
                    + q * q * q / 2.0L),
            alpha * (-2.0L * q + 1.5L * q * q) * chain};
    }
    const long double tail = 2.0L - q;
    return {alpha * tail * tail * tail / 6.0L,
        -alpha * tail * tail * 0.5L * chain};
}

long double surface_c15(long double q) {
    if (q <= 1.0L) return q * q - 1.0L;
    if (q < 3.0L) {
        const long double shifted = q - 2.0L;
        return 1.0L - shifted * shifted;
    }
    return 0.0L;
}

long double surface_C15(long double distance, long double r0) {
    const long double q = distance / r0;
    if (q <= 1.0L) {
        return r0 * (q * q * q / 3.0L - q - 2.0L / 3.0L);
    }
    if (q < 3.0L) {
        const long double shifted = q - 2.0L;
        return r0 * (q - shifted * shifted * shifted / 3.0L
            - 8.0L / 3.0L);
    }
    return 0.0L;
}

std::vector<Vec3l> positions15(const State15& state) {
    std::vector<Vec3l> result;
    result.reserve(state.dynamic.size());
    for (const Dynamic15& record : state.dynamic) {
        result.push_back(record.position);
    }
    return result;
}

std::vector<Vec3l> prediction15(const State15& accepted,
    const SolverOptions15& options) {
    const Vec3l gravity = options.gravity_mode == GravityMode15::Profile
        ? options.profile.gravity : Vec3l{};
    std::vector<Vec3l> result;
    result.reserve(accepted.dynamic.size());
    for (const Dynamic15& record : accepted.dynamic) {
        result.push_back(record.position + options.profile.dt
            * (record.velocity + options.profile.dt * gravity));
    }
    return result;
}

Vec3l project_one15(Vec3l value, const SolverOptions15& options,
    Work15& work) {
    if (options.boundary_mode == BoundaryMode15::UnboundedManufactured) {
        return value;
    }
    const long double low = 0.5L * options.profile.spacing;
    const Vec3l high{options.profile.basin.x - low,
        options.profile.basin.y - low, options.profile.basin.z - low};
    std::array<long double, 3> input{value.x, value.y, value.z};
    const std::array<long double, 3> upper{high.x, high.y, high.z};
    for (std::size_t axis = 0U; axis < input.size(); ++axis) {
        work.box_plane_tests += 2U;
        input[axis] = std::max(low, std::min(upper[axis], input[axis]));
    }
    return {input[0], input[1], input[2]};
}

bool project15(const std::vector<Vec3l>& values,
    const SolverOptions15& options, std::vector<Vec3l>& result,
    Work15& work) {
    bool finite = true;
    for (const Vec3l value : values) {
        finite = counted15(work, finite15(value)) && finite;
    }
    result.clear();
    if (!finite) return false;
    result.reserve(values.size());
    for (const Vec3l value : values) {
        result.push_back(project_one15(value, options, work));
    }
    return true;
}

Graph15 build_graph15(const State15& accepted,
    const std::vector<Vec3l>& positions, const SolverOptions15& options,
    Work15& work) {
    Graph15 result;
    const std::size_t count = accepted.dynamic.size();
    result.rows.resize(count);
    ++work.graph_builds;

    std::map<Cell15, std::vector<Neighbor15>> cells;
    for (std::size_t index = 0U; index < count; ++index) {
        cells[cell_of15(positions[index], options.profile.horizon)]
            .push_back({accepted.dynamic[index].id, index, false});
    }
    for (std::size_t index = 0U; index < accepted.ghosts.size(); ++index) {
        cells[cell_of15(accepted.ghosts[index].position,
                options.profile.horizon)]
            .push_back({accepted.ghosts[index].id, index, true});
    }
    for (auto& item : cells) {
        auto& records = item.second;
        std::sort(records.begin(), records.end(),
            [](const Neighbor15& lhs, const Neighbor15& rhs) {
                if (lhs.id != rhs.id) return lhs.id < rhs.id;
                return lhs.ghost < rhs.ghost;
            });
    }

    for (std::size_t owner = 0U; owner < count; ++owner) {
        const Cell15 base = cell_of15(
            positions[owner], options.profile.horizon);
        auto& row = result.rows[owner];
        bool stop = false;
        for (std::int64_t dz = -1; dz <= 1 && !stop; ++dz) {
            for (std::int64_t dy = -1; dy <= 1 && !stop; ++dy) {
                for (std::int64_t dx = -1; dx <= 1 && !stop; ++dx) {
                    const auto found = cells.find(
                        {base.x + dx, base.y + dy, base.z + dz});
                    if (found == cells.end()) continue;
                    for (const Neighbor15 neighbor : found->second) {
                        ++work.graph_candidates;
                        const Vec3l other = neighbor.ghost
                            ? accepted.ghosts[neighbor.index].position
                            : positions[neighbor.index];
                        const long double distance = norm15(
                            positions[owner] - other);
                        if (distance > options.profile.horizon) continue;
                        if ((neighbor.ghost || neighbor.index != owner)
                                && distance == 0.0L) {
                            result.valid = false;
                            stop = true;
                            break;
                        }
                        row.push_back(neighbor);
                        ++work.accepted_pairs;
                        if (row.size()
                                > options.profile.maximum_neighbors) {
                            result.valid = false;
                            result.capacity_overflow = true;
                            stop = true;
                            break;
                        }
                    }
                }
            }
        }
        std::sort(row.begin(), row.end(),
            [](const Neighbor15& lhs, const Neighbor15& rhs) {
                if (lhs.id != rhs.id) return lhs.id < rhs.id;
                return lhs.ghost < rhs.ghost;
            });
        result.maximum_degree = std::max(result.maximum_degree,
            static_cast<std::uint32_t>(row.size()));
        if (!result.valid) return result;
    }

    for (std::size_t first = 0U; first < count; ++first) {
        for (const Neighbor15 neighbor : result.rows[first]) {
            if (neighbor.ghost || first == neighbor.index
                    || accepted.dynamic[first].id
                        >= accepted.dynamic[neighbor.index].id) {
                continue;
            }
            result.dynamic_pairs.emplace_back(first, neighbor.index);
        }
    }
    std::sort(result.dynamic_pairs.begin(), result.dynamic_pairs.end(),
        [&](const auto& lhs, const auto& rhs) {
            return std::tie(accepted.dynamic[lhs.first].id,
                       accepted.dynamic[lhs.second].id)
                < std::tie(accepted.dynamic[rhs.first].id,
                    accepted.dynamic[rhs.second].id);
        });
    return result;
}

std::vector<ReferencePair15> reference_pairs15(const State15& accepted,
    const SolverOptions15& options, Work15& work, bool& valid,
    bool& capacity_overflow) {
    std::vector<Vec3l> reference;
    reference.reserve(accepted.dynamic.size());
    for (const Dynamic15& item : accepted.dynamic) {
        reference.push_back(item.position);
    }
    State15 dynamic_only = accepted;
    dynamic_only.ghosts.clear();
    const Graph15 graph = build_graph15(
        dynamic_only, reference, options, work);
    valid = graph.valid;
    capacity_overflow = graph.capacity_overflow;
    std::vector<ReferencePair15> result;
    if (!valid) return result;
    for (const auto pair : graph.dynamic_pairs) {
        const Vec3l difference = reference[pair.first]
            - reference[pair.second];
        const long double distance = norm15(difference);
        if (!(distance > 0.0L)
                || distance > options.profile.horizon) continue;
        result.push_back({pair.first, pair.second, difference / distance,
            -kernel15(distance, options).derivative});
    }
    return result;
}

long double pairwise_sum15(std::vector<long double> values) {
    if (values.empty()) return 0.0L;
    while (values.size() > 1U) {
        std::vector<long double> next;
        next.reserve((values.size() + 1U) / 2U);
        for (std::size_t index = 0U; index < values.size(); index += 2U) {
            next.push_back(index + 1U == values.size()
                ? values[index] : values[index] + values[index + 1U]);
        }
        values = std::move(next);
    }
    return values.front();
}

long double vector_norm15(const std::vector<Vec3l>& values) {
    std::vector<long double> squares;
    squares.reserve(values.size());
    for (const Vec3l value : values) squares.push_back(dot15(value, value));
    return std::sqrt(std::max(0.0L, pairwise_sum15(std::move(squares))));
}

long double max_component15(const std::vector<Vec3l>& values) {
    long double result = 0.0L;
    for (const Vec3l value : values) {
        result = std::max(result, std::fabs(value.x));
        result = std::max(result, std::fabs(value.y));
        result = std::max(result, std::fabs(value.z));
    }
    return result;
}

bool finite_evaluation15(const EnergyGradient15& value) {
    bool finite = finite_scalar15(value.objective)
        && finite_scalar15(value.nonpressure_energy)
        && finite_scalar15(value.inertia_energy)
        && finite_scalar15(value.viscosity_energy)
        && finite_scalar15(value.surface_energy);
    for (const long double item : value.density.rho) {
        finite = finite_scalar15(item) && finite;
    }
    for (const long double item : value.density.constraint) {
        finite = finite_scalar15(item) && finite;
    }
    for (const Vec3l item : value.gradient) {
        finite = finite15(item) && finite;
    }
    for (const long double item : value.effective_multiplier) {
        finite = finite_scalar15(item) && finite;
    }
    for (const auto* forces : std::array<const std::vector<Vec3l>*, 4>{
             &value.dynamic_pressure_force, &value.ghost_pressure_force,
             &value.viscosity_force, &value.surface_force}) {
        for (const Vec3l item : *forces) finite = finite15(item) && finite;
    }
    return finite;
}

Density15 density_from_graph15(const State15& accepted,
    const std::vector<Vec3l>& positions, const SolverOptions15& options,
    const Graph15& graph, Work15& work) {
    Density15 result;
    result.valid = graph.valid;
    result.capacity_overflow = graph.capacity_overflow;
    result.outcome = graph.valid ? "PASS"
        : (graph.capacity_overflow ? "CAPACITY_OVERFLOW" : "INVALID_GRAPH");
    result.maximum_degree = graph.maximum_degree;
    result.accepted_pair_count = 0U;
    result.rho.assign(positions.size(), 0.0L);
    result.constraint.assign(positions.size(), 0.0L);
    if (!graph.valid) return result;
    for (std::size_t owner = 0U; owner < graph.rows.size(); ++owner) {
        std::vector<long double> terms;
        terms.reserve(graph.rows[owner].size());
        for (const Neighbor15 neighbor : graph.rows[owner]) {
            const Vec3l other = neighbor.ghost
                ? accepted.ghosts[neighbor.index].position
                : positions[neighbor.index];
            const long double distance = norm15(positions[owner] - other);
            terms.push_back(options.profile.mass
                * kernel15(distance, options).value);
            ++work.density_pairs;
            ++result.accepted_pair_count;
        }
        result.rho[owner] = pairwise_sum15(std::move(terms));
        result.constraint[owner] = result.rho[owner] / options.profile.rho0
            - 1.0L;
        if (!finite_scalar15(result.rho[owner])
                || !finite_scalar15(result.constraint[owner])) {
            result.valid = false;
            result.outcome = "NONFINITE_DENSITY";
        }
    }
    return result;
}

EnergyGradient15 evaluate15(const State15& accepted,
    const std::vector<Vec3l>& positions,
    const std::vector<long double>& multipliers,
    const SolverOptions15& options, MultiplierMode15 multiplier_mode,
    bool need_gradient, const std::vector<ReferencePair15>* frozen_pairs,
    Work15& work) {
    EnergyGradient15 result;
    ++work.objective_evaluations;
    if (need_gradient) ++work.gradient_evaluations;
    const std::size_t count = accepted.dynamic.size();
    const bool positions_size = counted15(work, positions.size() == count);
    const bool multipliers_size = counted15(
        work, multipliers.size() == count);
    const bool nonempty = counted15(work, count != 0U);
    bool positions_finite = true;
    for (const Vec3l position : positions) {
        positions_finite = counted15(work, finite15(position))
            && positions_finite;
    }
    bool multipliers_finite = true;
    for (const long double multiplier : multipliers) {
        multipliers_finite = counted15(work, finite_scalar15(multiplier))
            && multipliers_finite;
    }
    if (!positions_size || !multipliers_size || !nonempty
            || !positions_finite || !multipliers_finite) {
        result.outcome = "INVALID_INPUT";
        return result;
    }
    const std::vector<Vec3l> predicted = prediction15(accepted, options);
    result.gradient.assign(count, {});
    result.dynamic_pressure_force.assign(count, {});
    result.ghost_pressure_force.assign(count, {});
    result.viscosity_force.assign(count, {});
    result.surface_force.assign(count, {});

    std::vector<long double> inertia_terms;
    inertia_terms.reserve(count);
    const long double inertia_coefficient = options.profile.mass
        / (options.profile.dt * options.profile.dt);
    for (std::size_t index = 0U; index < count; ++index) {
        const Vec3l displacement = positions[index] - predicted[index];
        inertia_terms.push_back(0.5L * inertia_coefficient
            * dot15(displacement, displacement));
        if (need_gradient) {
            result.gradient[index] += inertia_coefficient * displacement;
        }
    }
    result.inertia_energy = pairwise_sum15(std::move(inertia_terms));

    const Graph15 graph = build_graph15(accepted, positions, options, work);
    result.density = density_from_graph15(
        accepted, positions, options, graph, work);
    if (!result.density.valid) {
        result.outcome = result.density.outcome;
        return result;
    }

    std::vector<ReferencePair15> local_reference_pairs;
    const std::vector<ReferencePair15>* reference_pairs = frozen_pairs;
    bool reference_valid = true;
    bool reference_capacity_overflow = false;
    const bool viscosity_enabled = options.terms.normal_viscosity
        || options.terms.tangential_viscosity;
    if (!viscosity_enabled) {
        reference_pairs = &local_reference_pairs;
    } else if (options.mutation
            == Mutation15::CurrentReferenceViscosityGraph) {
        State15 current_reference = accepted;
        for (std::size_t index = 0U; index < positions.size(); ++index) {
            current_reference.dynamic[index].position = positions[index];
        }
        SolverOptions15 reference_options = options;
        reference_options.mutation = Mutation15::None;
        local_reference_pairs = reference_pairs15(current_reference,
            reference_options, work, reference_valid,
            reference_capacity_overflow);
        reference_pairs = &local_reference_pairs;
    } else if (reference_pairs == nullptr) {
        local_reference_pairs = reference_pairs15(
            accepted, options, work, reference_valid,
            reference_capacity_overflow);
        reference_pairs = &local_reference_pairs;
    }
    if (!reference_valid) {
        result.outcome = reference_capacity_overflow
            ? "CAPACITY_OVERFLOW" : "INVALID_REFERENCE_GRAPH";
        return result;
    }

    std::vector<long double> viscosity_terms;
    if (options.terms.normal_viscosity
            || options.terms.tangential_viscosity) {
        viscosity_terms.reserve(reference_pairs->size());
        const long double lambda = (options.terms.normal_viscosity
                ? options.profile.lambda_v : 0.0L)
            * (options.mutation == Mutation15::HalfNormalViscosity
                ? 0.5L : 1.0L);
        const long double mu = options.terms.tangential_viscosity
            ? options.profile.mu_v : 0.0L;
        const long double factor = options.profile.mass
            / (options.profile.rho0 * options.profile.dt);
        for (const ReferencePair15& pair : *reference_pairs) {
            const Vec3l delta = (positions[pair.first]
                    - positions[pair.second])
                - (accepted.dynamic[pair.first].position
                    - accepted.dynamic[pair.second].position);
            const Vec3l normal_delta = pair.normal
                * dot15(pair.normal, delta);
            const Vec3l tangent_delta = delta - normal_delta;
            viscosity_terms.push_back(factor * pair.omega
                * (mu * dot15(tangent_delta, tangent_delta)
                    + 0.5L * lambda
                        * dot15(normal_delta, normal_delta)));
            if (need_gradient) {
                const Vec3l endpoint = factor * pair.omega
                    * (2.0L * mu * tangent_delta
                        + lambda * normal_delta);
                result.gradient[pair.first] += endpoint;
                result.gradient[pair.second] -= endpoint;
                result.viscosity_force[pair.first] -= endpoint;
                result.viscosity_force[pair.second] += endpoint;
            }
            ++work.viscosity_pairs;
            ++result.viscosity_pair_count;
        }
    }
    result.viscosity_energy = pairwise_sum15(std::move(viscosity_terms));

    std::vector<long double> surface_terms;
    if (options.terms.surface) {
        State15 surface_state = accepted;
        surface_state.ghosts.clear();
        SolverOptions15 surface_options = options;
        surface_options.profile.horizon = 3.0L
            * options.profile.surface_r0;
        const Graph15 surface_graph = build_graph15(surface_state,
            positions, surface_options, work);
        if (!surface_graph.valid) {
            result.outcome = surface_graph.capacity_overflow
                ? "CAPACITY_OVERFLOW" : "INVALID_SURFACE_GRAPH";
            return result;
        }
        surface_terms.reserve(surface_graph.dynamic_pairs.size());
        const long double sign = options.mutation == Mutation15::WrongSurfaceSign
            ? -1.0L : 1.0L;
        const long double pair_factor =
            options.mutation == Mutation15::MissingSurfaceFactorTwo
            ? 1.0L : 2.0L;
        for (const auto pair : surface_graph.dynamic_pairs) {
            ++work.surface_pair_tests;
            const Vec3l difference = positions[pair.first]
                - positions[pair.second];
            const long double distance = norm15(difference);
            if (!(distance > 0.0L)) {
                result.outcome = "COINCIDENT_SURFACE_PAIR";
                return result;
            }
            if (!(distance < 3.0L * options.profile.surface_r0)) continue;
            ++work.surface_active_pairs;
            ++result.surface_pair_count;
            surface_terms.push_back(sign * pair_factor * options.profile.gamma
                * options.profile.mass * options.profile.mass
                * surface_C15(distance, options.profile.surface_r0));
            if (need_gradient) {
                const Vec3l endpoint = sign * pair_factor
                    * options.profile.gamma * options.profile.mass
                    * options.profile.mass
                    * surface_c15(distance / options.profile.surface_r0)
                    * (difference / distance);
                result.gradient[pair.first] += endpoint;
                result.gradient[pair.second] -= endpoint;
                result.surface_force[pair.first] -= endpoint;
                result.surface_force[pair.second] += endpoint;
            }
        }
    }
    result.surface_energy = pairwise_sum15(std::move(surface_terms));
    result.nonpressure_energy = result.inertia_energy
        + result.viscosity_energy + result.surface_energy;

    result.effective_multiplier.assign(count, 0.0L);
    std::vector<long double> pressure_terms;
    pressure_terms.reserve(count);
    for (std::size_t owner = 0U; owner < count; ++owner) {
        long double effective = multipliers[owner];
        if (options.mutation == Mutation15::FinitePressurePenalty
                && multiplier_mode
                    == MultiplierMode15::AugmentedLagrangian) {
            const long double positive = std::max(
                0.0L, result.density.constraint[owner]);
            effective = options.profile.beta * positive;
            result.effective_multiplier[owner] = effective;
            pressure_terms.push_back(0.5L * options.profile.beta
                * positive * positive);
        } else if (options.terms.density_constraint
                && multiplier_mode == MultiplierMode15::AugmentedLagrangian) {
            effective = std::max(0.0L, multipliers[owner]
                + options.profile.beta * result.density.constraint[owner]);
            result.effective_multiplier[owner] = effective;
        } else if (!options.terms.density_constraint) {
            effective = 0.0L;
            result.effective_multiplier[owner] = effective;
        } else {
            result.effective_multiplier[owner] = effective;
        }
        if (!(options.mutation == Mutation15::FinitePressurePenalty
                    && multiplier_mode
                        == MultiplierMode15::AugmentedLagrangian)
                && options.terms.density_constraint
                && multiplier_mode == MultiplierMode15::AugmentedLagrangian) {
            pressure_terms.push_back((effective * effective
                    - multipliers[owner] * multipliers[owner])
                / (2.0L * options.profile.beta));
        }
        if (!need_gradient || !options.terms.density_constraint) continue;
        for (const Neighbor15 neighbor : graph.rows[owner]) {
            const Vec3l other = neighbor.ghost
                ? accepted.ghosts[neighbor.index].position
                : positions[neighbor.index];
            const Vec3l difference = positions[owner] - other;
            const long double distance = norm15(difference);
            if (!(distance > 0.0L)) continue;
            const Vec3l derivative = options.profile.mass
                / options.profile.rho0
                * kernel15(distance, options).derivative
                * (difference / distance);
            const Vec3l endpoint = effective * derivative;
            result.gradient[owner] += endpoint;
            if (neighbor.ghost) {
                result.ghost_pressure_force[owner] -= endpoint;
            } else if (neighbor.index != owner) {
                result.dynamic_pressure_force[owner] -= endpoint;
                result.gradient[neighbor.index] -= endpoint;
                result.dynamic_pressure_force[neighbor.index] += endpoint;
            }
            ++work.jacobian_products;
        }
    }
    result.objective = result.nonpressure_energy
        + pairwise_sum15(std::move(pressure_terms));
    result.valid = finite_evaluation15(result);
    result.outcome = result.valid ? "PASS" : "NONFINITE_EVALUATION";
    return result;
}

} // namespace

Density15 candidate_density15(const State15& accepted,
    const std::vector<Vec3l>& positions, const SolverOptions15& options,
    Work15& work) {
    const Graph15 graph = build_graph15(accepted, positions, options, work);
    return density_from_graph15(accepted, positions, options, graph, work);
}

EnergyGradient15 candidate_energy_gradient15(const State15& accepted,
    const std::vector<Vec3l>& positions,
    const std::vector<long double>& multipliers,
    const SolverOptions15& options, MultiplierMode15 multiplier_mode,
    bool need_gradient, Work15& work) {
    return evaluate15(accepted, positions, multipliers, options,
        multiplier_mode, need_gradient, nullptr, work);
}

namespace {

struct GraphAudit15 {
    std::uint64_t candidates = 0U;
    std::uint64_t accepted = 0U;
    std::uint64_t density_pairs = 0U;
    std::uint64_t dynamic_pairs = 0U;
    std::uint64_t surface_tests = 0U;
    std::uint64_t surface_active = 0U;
    bool complete = true;
};

GraphAudit15 independently_enumerate_graph15(const State15& accepted,
    const std::vector<Vec3l>& positions, const SolverOptions15& options) {
    GraphAudit15 result;
    struct Record {
        std::uint32_t id = 0U;
        std::size_t index = 0U;
        bool ghost = false;
        Vec3l position;
    };
    std::map<Cell15, std::vector<Record>> cells;
    for (std::size_t index = 0U; index < positions.size(); ++index) {
        cells[cell_of15(positions[index], options.profile.horizon)]
            .push_back({accepted.dynamic[index].id, index, false,
                positions[index]});
    }
    for (std::size_t index = 0U; index < accepted.ghosts.size(); ++index) {
        const Ghost15& ghost = accepted.ghosts[index];
        cells[cell_of15(ghost.position, options.profile.horizon)]
            .push_back({ghost.id, index, true, ghost.position});
    }
    for (auto& item : cells) {
        std::sort(item.second.begin(), item.second.end(),
            [](const Record& lhs, const Record& rhs) {
                if (lhs.id != rhs.id) return lhs.id < rhs.id;
                return lhs.ghost < rhs.ghost;
            });
    }
    std::set<std::pair<std::size_t, std::size_t>> unique_pairs;
    for (std::size_t owner = 0U; owner < positions.size(); ++owner) {
        const Cell15 base = cell_of15(
            positions[owner], options.profile.horizon);
        std::uint32_t degree = 0U;
        for (std::int64_t dz = -1; dz <= 1; ++dz) {
            for (std::int64_t dy = -1; dy <= 1; ++dy) {
                for (std::int64_t dx = -1; dx <= 1; ++dx) {
                    const auto found = cells.find(
                        {base.x + dx, base.y + dy, base.z + dz});
                    if (found == cells.end()) continue;
                    for (const Record& record : found->second) {
                        ++result.candidates;
                        const long double distance = norm15(
                            positions[owner] - record.position);
                        if (distance > options.profile.horizon) continue;
                        if ((record.ghost || record.index != owner)
                                && distance == 0.0L) {
                            result.complete = false;
                            return result;
                        }
                        ++result.accepted;
                        ++result.density_pairs;
                        ++degree;
                        if (!record.ghost && record.index != owner) {
                            const std::uint32_t owner_id =
                                accepted.dynamic[owner].id;
                            const std::uint32_t neighbor_id =
                                accepted.dynamic[record.index].id;
                            if (owner_id < neighbor_id) {
                                unique_pairs.emplace(owner, record.index);
                            }
                        }
                        if (degree > options.profile.maximum_neighbors) {
                            result.complete = false;
                            return result;
                        }
                    }
                }
            }
        }
    }
    result.dynamic_pairs = unique_pairs.size();
    for (const auto pair : unique_pairs) {
        ++result.surface_tests;
        if (norm15(positions[pair.first] - positions[pair.second])
                < 3.0L * options.profile.surface_r0) {
            ++result.surface_active;
        }
    }
    return result;
}

} // namespace

Work15 candidate_expected_density_work15(const State15& accepted,
    const std::vector<Vec3l>& positions, const SolverOptions15& options) {
    Work15 expected;
    const GraphAudit15 graph = independently_enumerate_graph15(
        accepted, positions, options);
    ++expected.graph_builds;
    expected.graph_candidates = graph.candidates;
    expected.accepted_pairs = graph.accepted;
    expected.density_pairs = graph.complete ? graph.density_pairs : 0U;
    return expected;
}

namespace {

std::uint64_t stage_count15(const StepTrace15& trace,
    std::string_view stage) {
    return static_cast<std::uint64_t>(std::count(
        trace.reached_stages.begin(), trace.reached_stages.end(), stage));
}

Work15 expected_solver_work15(const State15& accepted,
    const SolverOptions15& options, const StepTrace15& trace) {
    Work15 expected;
    expected.fixture_records = accepted.dynamic.size();
    expected.ghost_records = accepted.ghosts.size();

    State15 dynamic_only = accepted;
    dynamic_only.ghosts.clear();
    GraphAudit15 reference;
    const bool viscosity_enabled = options.terms.normal_viscosity
        || options.terms.tangential_viscosity;
    if (viscosity_enabled
            && options.mutation
                != Mutation15::CurrentReferenceViscosityGraph) {
        reference = independently_enumerate_graph15(
            dynamic_only, positions15(accepted), options);
        ++expected.graph_builds;
        expected.graph_candidates += reference.candidates;
        expected.accepted_pairs += reference.accepted;
    }

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
        ++expected.objective_evaluations;
        if (gradient) {
            ++expected.gradient_evaluations;
        }
        expected.scalar_predicates += 3U + evaluation.size()
            + multipliers.size();
        if (evaluation.size() != accepted.dynamic.size()
                || multipliers.size() != accepted.dynamic.size()
                || accepted.dynamic.empty()
                || !finite_positions15(evaluation)
                || !std::all_of(multipliers.begin(), multipliers.end(),
                    finite_scalar15)) {
            continue;
        }
        const GraphAudit15 graph = independently_enumerate_graph15(
            accepted, evaluation, options);
        ++expected.graph_builds;
        expected.graph_candidates += graph.candidates;
        expected.accepted_pairs += graph.accepted;
        if (!graph.complete) continue;
        expected.density_pairs += graph.density_pairs;
        if (gradient && options.terms.density_constraint) {
            expected.jacobian_products += graph.accepted
                - accepted.dynamic.size();
        }
        GraphAudit15 current_reference;
        if (viscosity_enabled && options.mutation
                == Mutation15::CurrentReferenceViscosityGraph) {
            current_reference = independently_enumerate_graph15(dynamic_only,
                evaluation, options);
            ++expected.graph_builds;
            expected.graph_candidates += current_reference.candidates;
            expected.accepted_pairs += current_reference.accepted;
            if (!current_reference.complete) continue;
        }
        if (options.terms.normal_viscosity
                || options.terms.tangential_viscosity) {
            if (options.mutation
                    == Mutation15::CurrentReferenceViscosityGraph) {
                expected.viscosity_pairs += current_reference.dynamic_pairs;
            } else {
                expected.viscosity_pairs += reference.dynamic_pairs;
            }
        }
        if (options.terms.surface) {
            SolverOptions15 surface_options = options;
            surface_options.profile.horizon = 3.0L
                * options.profile.surface_r0;
            const GraphAudit15 surface = independently_enumerate_graph15(
                dynamic_only, evaluation, surface_options);
            ++expected.graph_builds;
            expected.graph_candidates += surface.candidates;
            expected.accepted_pairs += surface.accepted;
            expected.surface_pair_tests += surface.surface_tests;
            expected.surface_active_pairs += surface.surface_active;
        }
    }
    const std::uint64_t projections = stage_count15(trace, "PROJECTION");
    const std::uint64_t box_projections =
        stage_count15(trace, "PROJECTION_BOX");
    const std::uint64_t inner_gates = stage_count15(trace, "INNER_GATE");
    const std::uint64_t final_gates = stage_count15(trace, "FINAL_GATE");
    const std::uint64_t contact_batches = stage_count15(trace, "CONTACT");
    expected.projected_directions = (inner_gates + final_gates)
        * accepted.dynamic.size();
    expected.line_search_trials = stage_count15(trace, "LINE_TRIAL");
    expected.accepted_inner_iterations = trace.accepted_inner_iterations;
    expected.outer_multiplier_updates = trace.outer_updates;
    if (options.boundary_mode == BoundaryMode15::AnalyticBox) {
        expected.box_plane_tests = 6U * accepted.dynamic.size()
            * (box_projections + final_gates + contact_batches);
    }
    if (options.boundary_mode == BoundaryMode15::AnalyticBox) {
        expected.contact_reaction_values = 3U * accepted.dynamic.size()
            * contact_batches;
    }
    if (stage_count15(trace, "TOPOLOGY") != 0U) {
        const GraphAudit15 topology = independently_enumerate_graph15(accepted,
            trace.evaluation_positions.empty() ? positions15(accepted)
                                               : trace.evaluation_positions.back(),
            options);
        ++expected.graph_builds;
        expected.graph_candidates += topology.candidates;
        expected.accepted_pairs += topology.accepted;
        expected.topology_edges = accepted.dynamic.size()
            + 2U * topology.dynamic_pairs;
    }
    expected.scalar_predicates += kProfilePredicateCount15
        + kOptionPredicateCount15 + 2U + 4U * accepted.dynamic.size()
        + 2U * accepted.ghosts.size() + 2U * inner_gates
        + accepted.dynamic.size() * projections
        + stage_count15(trace, "SLOPE")
        + stage_count15(trace, "ARMIJO") + accepted.dynamic.size()
            * stage_count15(trace, "MULTIPLIER_UPDATE")
        + (8U + accepted.dynamic.size()
            + (options.boundary_mode == BoundaryMode15::AnalyticBox
                    ? 1U : 0U))
            * final_gates;
    return expected;
}

bool counted15(Work15& work, bool value) {
    ++work.scalar_predicates;
    return value;
}

bool kkt_gates15(Step15& step, const State15& accepted,
    const std::vector<Vec3l>& y, const std::vector<long double>& pi,
    const EnergyGradient15& final, const SolverOptions15& options,
    StepTrace15& trace) {
    const std::size_t count = y.size();
    const long double scale = options.profile.dt * options.profile.dt
        / options.profile.mass;
    step.kkt_gradient = final.gradient;
    std::vector<Vec3l> proposal(count);
    for (std::size_t index = 0U; index < count; ++index) {
        proposal[index] = y[index] - scale * final.gradient[index];
    }
    std::vector<Vec3l> projected;
    Work15 projection_work;
    const bool projection_valid = project15(
        proposal, options, projected, projection_work);
    add_work15(step.work, projection_work);
    trace.reached_stages.push_back("PROJECTION");
    if (projection_valid
            && options.boundary_mode == BoundaryMode15::AnalyticBox) {
        trace.reached_stages.push_back("PROJECTION_BOX");
    }
    if (!projection_valid) {
        step.outcome = "NONFINITE";
        return false;
    }
    step.kkt_direction.resize(count);
    for (std::size_t index = 0U; index < count; ++index) {
        step.kkt_direction[index] = projected[index] - y[index];
    }
    step.work.projected_directions += count;
    step.metrics.objective = final.objective;
    step.metrics.nonpressure_energy = final.nonpressure_energy;
    step.metrics.inertia_energy = final.inertia_energy;
    step.metrics.viscosity_energy = final.viscosity_energy;
    step.metrics.surface_energy = final.surface_energy;
    std::vector<long double> strain_squares;
    std::vector<long double> fixed_numerator_terms;
    std::vector<long double> fixed_denominator_terms;
    strain_squares.reserve(count);
    fixed_numerator_terms.reserve(count);
    fixed_denominator_terms.reserve(count);
    long double maximum_strain = 0.0L;
    long double complementarity = 0.0L;
    bool multipliers_nonnegative = true;
    step.metrics.active_multiplier_count = 0U;
    step.active_multiplier_ids.clear();
    for (std::size_t index = 0U; index < count; ++index) {
        const long double strain = std::max(
            0.0L, final.density.constraint[index]);
        strain_squares.push_back(strain * strain);
        maximum_strain = std::max(maximum_strain, strain);
        complementarity = std::max(complementarity,
            std::fabs(pi[index] * final.density.constraint[index]));
        const long double updated = std::max(0.0L, pi[index]
            + options.profile.beta * final.density.constraint[index]);
        const long double difference = updated - pi[index];
        fixed_numerator_terms.push_back(difference * difference);
        fixed_denominator_terms.push_back(pi[index] * pi[index]);
        multipliers_nonnegative = counted15(step.work, pi[index] >= 0.0L)
            && multipliers_nonnegative;
        if (pi[index] > 0.0L) {
            ++step.metrics.active_multiplier_count;
            step.active_multiplier_ids.push_back(accepted.dynamic[index].id);
        }
    }
    step.metrics.maximum_positive_density_strain = maximum_strain;
    step.metrics.rms_positive_density_strain = std::sqrt(
        pairwise_sum15(std::move(strain_squares))
            / static_cast<long double>(count));
    step.metrics.kkt_rms_m = vector_norm15(step.kkt_direction)
        / std::sqrt(static_cast<long double>(count));
    step.metrics.kkt_maximum_m = 0.0L;
    for (const Vec3l value : step.kkt_direction) {
        step.metrics.kkt_maximum_m = std::max(
            step.metrics.kkt_maximum_m, norm15(value));
    }
    const long double max_pi = pi.empty() ? 0.0L
        : *std::max_element(pi.begin(), pi.end());
    step.metrics.complementarity = complementarity
        / std::max(1.0L, max_pi);
    step.metrics.multiplier_fixed_point = std::sqrt(
        pairwise_sum15(std::move(fixed_numerator_terms)))
        / std::max(std::sqrt(pairwise_sum15(
            std::move(fixed_denominator_terms))), 1.0L);
    step.contact.clear();
    const long double low = 0.5L * options.profile.spacing;
    const Vec3l high{options.profile.basin.x - low,
        options.profile.basin.y - low, options.profile.basin.z - low};
    if (options.boundary_mode == BoundaryMode15::AnalyticBox) {
        step.contact.reserve(count);
        trace.reached_stages.push_back("CONTACT");
        for (std::size_t index = 0U; index < count; ++index) {
            Contact15 contact;
            contact.id = accepted.dynamic[index].id;
            const std::array<long double, 3> value{
                y[index].x, y[index].y, y[index].z};
            const std::array<long double, 3> maximum{
                high.x, high.y, high.z};
            const std::array<long double, 3> gradient{
                step.kkt_gradient[index].x, step.kkt_gradient[index].y,
                step.kkt_gradient[index].z};
            std::array<long double*, 3> impulse{
                &contact.impulse.x, &contact.impulse.y,
                &contact.impulse.z};
            for (std::size_t axis = 0U; axis < 3U; ++axis) {
                step.work.box_plane_tests += 2U;
                ++step.work.contact_reaction_values;
                if (value[axis] == low) {
                    contact.mask |= static_cast<std::uint8_t>(
                        1U << (2U * axis));
                    *impulse[axis] = options.profile.dt
                        * std::max(gradient[axis], 0.0L);
                } else if (value[axis] == maximum[axis]) {
                    contact.mask |= static_cast<std::uint8_t>(
                        1U << (2U * axis + 1U));
                    *impulse[axis] = options.profile.dt
                        * std::min(gradient[axis], 0.0L);
                }
            }
            step.contact.push_back(contact);
        }
    }

    trace.reached_stages.push_back("FINAL_GATE");
    bool finite_multipliers = true;
    for (const long double multiplier : pi) {
        finite_multipliers = finite_scalar15(multiplier)
            && finite_multipliers;
    }
    bool derived_finite = finite_evaluation15(final)
        && finite_positions15(y) && finite_multipliers
        && finite_scalar15(step.metrics.objective)
        && finite_scalar15(step.metrics.nonpressure_energy)
        && finite_scalar15(step.metrics.inertia_energy)
        && finite_scalar15(step.metrics.viscosity_energy)
        && finite_scalar15(step.metrics.surface_energy)
        && finite_scalar15(step.metrics.maximum_positive_density_strain)
        && finite_scalar15(step.metrics.rms_positive_density_strain)
        && finite_scalar15(step.metrics.kkt_rms_m)
        && finite_scalar15(step.metrics.kkt_maximum_m)
        && finite_scalar15(step.metrics.complementarity)
        && finite_scalar15(step.metrics.multiplier_fixed_point);
    for (const Dynamic15& item : step.state.dynamic) {
        derived_finite = finite15(item.reference) && finite15(item.position)
            && finite15(item.velocity) && derived_finite;
    }
    for (const Vec3l value : step.kkt_gradient) {
        derived_finite = finite15(value) && derived_finite;
    }
    for (const Vec3l value : step.kkt_direction) {
        derived_finite = finite15(value) && derived_finite;
    }
    for (const Contact15& item : step.contact) {
        derived_finite = finite15(item.impulse) && derived_finite;
    }
    const bool finite = counted15(step.work, derived_finite);
    if (!finite) step.outcome = "NONFINITE";
    const bool density_maximum = counted15(step.work, maximum_strain
        <= options.profile.gates.density_maximum);
    const bool density_rms = counted15(step.work,
        step.metrics.rms_positive_density_strain
            <= options.profile.gates.density_rms);
    const bool kkt_rms = counted15(step.work, step.metrics.kkt_rms_m
        <= options.profile.gates.kkt_rms_m);
    const bool kkt_maximum = counted15(step.work,
        step.metrics.kkt_maximum_m
            <= options.profile.gates.kkt_maximum_m);
    const bool complementarity_pass = counted15(step.work,
        step.metrics.complementarity
            <= options.profile.gates.complementarity);
    const bool fixed_point = counted15(step.work,
        step.metrics.multiplier_fixed_point
            <= options.profile.gates.multiplier_fixed_point);
    bool contained = true;
    if (options.boundary_mode == BoundaryMode15::AnalyticBox) {
        for (const Vec3l value : y) {
            step.work.box_plane_tests += 6U;
            contained = value.x >= low && value.x <= high.x && contained;
            contained = value.y >= low && value.y <= high.y && contained;
            contained = value.z >= low && value.z <= high.z && contained;
        }
    }
    const bool containment = options.boundary_mode
            == BoundaryMode15::AnalyticBox
        ? counted15(step.work, contained) : true;
    const bool capacity = counted15(
        step.work, !final.density.capacity_overflow);
    return finite && multipliers_nonnegative && density_maximum
        && density_rms && kkt_rms && kkt_maximum
        && complementarity_pass && fixed_point && containment && capacity;
}

void publish_final_records15(Step15& step, const State15& accepted,
    const std::vector<Vec3l>& y, const std::vector<long double>& pi,
    const EnergyGradient15& final, const SolverOptions15& options) {
    step.state = accepted;
    step.pi = pi;
    step.rho = final.density.rho;
    step.dynamic_pressure_force = final.dynamic_pressure_force;
    step.ghost_pressure_force = final.ghost_pressure_force;
    step.viscosity_force = final.viscosity_force;
    step.surface_force = final.surface_force;
    for (std::size_t index = 0U; index < y.size(); ++index) {
        Dynamic15& record = step.state.dynamic[index];
        record.reference = y[index];
        record.position = y[index];
        record.velocity = (y[index] - accepted.dynamic[index].position)
            / options.profile.dt;
    }
}

void publish_private_state15(Step15& step, const State15& accepted,
    const std::vector<Vec3l>& y, const std::vector<long double>& pi,
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
    step.state = accepted;
    step.pi = pi;
    for (std::size_t index = 0U; index < y.size(); ++index) {
        step.state.dynamic[index].reference = y[index];
        step.state.dynamic[index].position = y[index];
        step.state.dynamic[index].velocity = (y[index]
            - accepted.dynamic[index].position) / options.profile.dt;
    }
    if (evaluation != nullptr) {
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
}

} // namespace

Work15 candidate_expected_energy_gradient_work15(
    const State15& accepted, const std::vector<Vec3l>& positions,
    const std::vector<long double>& multipliers,
    const SolverOptions15& options, bool need_gradient) {
    Work15 expected;
    ++expected.objective_evaluations;
    if (need_gradient) ++expected.gradient_evaluations;
    expected.scalar_predicates = 3U + positions.size() + multipliers.size();
    if (accepted.dynamic.empty()
            || positions.size() != accepted.dynamic.size()
            || multipliers.size() != accepted.dynamic.size()
            || !finite_positions15(positions)
            || !std::all_of(multipliers.begin(), multipliers.end(),
                finite_scalar15)) {
        return expected;
    }
    const GraphAudit15 density = independently_enumerate_graph15(
        accepted, positions, options);
    ++expected.graph_builds;
    expected.graph_candidates += density.candidates;
    expected.accepted_pairs += density.accepted;
    if (!density.complete) return expected;
    expected.density_pairs += density.density_pairs;
    if (need_gradient && options.terms.density_constraint) {
        expected.jacobian_products += density.accepted
            - accepted.dynamic.size();
    }
    if (options.terms.normal_viscosity
            || options.terms.tangential_viscosity) {
        State15 dynamic_only = accepted;
        dynamic_only.ghosts.clear();
        const std::vector<Vec3l>& reference_positions =
            options.mutation == Mutation15::CurrentReferenceViscosityGraph
            ? positions : positions15(accepted);
        const GraphAudit15 reference = independently_enumerate_graph15(
            dynamic_only, reference_positions, options);
        ++expected.graph_builds;
        expected.graph_candidates += reference.candidates;
        expected.accepted_pairs += reference.accepted;
        if (!reference.complete) return expected;
        expected.viscosity_pairs += reference.dynamic_pairs;
    }
    if (options.terms.surface) {
        State15 dynamic_only = accepted;
        dynamic_only.ghosts.clear();
        SolverOptions15 surface_options = options;
        surface_options.profile.horizon =
            3.0L * options.profile.surface_r0;
        const GraphAudit15 surface = independently_enumerate_graph15(
            dynamic_only, positions, surface_options);
        ++expected.graph_builds;
        expected.graph_candidates += surface.candidates;
        expected.accepted_pairs += surface.accepted;
        expected.surface_pair_tests += surface.surface_tests;
        expected.surface_active_pairs += surface.surface_active;
    }
    return expected;
}

Step15 candidate_step15(const State15& input,
    const SolverOptions15& options) {
    Step15 step;
    State15 accepted = input;
    std::sort(accepted.dynamic.begin(), accepted.dynamic.end(),
        [](const Dynamic15& lhs, const Dynamic15& rhs) {
            return lhs.id < rhs.id;
        });
    std::sort(accepted.ghosts.begin(), accepted.ghosts.end(),
        [](const Ghost15& lhs, const Ghost15& rhs) {
            return lhs.id < rhs.id;
        });
    step.work.fixture_records = accepted.dynamic.size();
    step.work.ghost_records = accepted.ghosts.size();
    const bool profile_valid = valid_profile15(options, step.work);
    bool input_valid = counted15(step.work, !accepted.dynamic.empty());
    input_valid = counted15(step.work,
        accepted.dynamic.size() <= options.profile.maximum_dynamic_samples)
        && input_valid;
    std::uint32_t prior_id = 0U;
    bool first_id = true;
    for (const Dynamic15& record : accepted.dynamic) {
        input_valid = counted15(step.work, finite15(record.reference))
            && input_valid;
        input_valid = counted15(step.work, finite15(record.position))
            && input_valid;
        input_valid = counted15(step.work, finite15(record.velocity))
            && input_valid;
        const bool ordered = first_id || record.id > prior_id;
        input_valid = counted15(step.work, ordered) && input_valid;
        prior_id = record.id;
        first_id = false;
    }
    for (const Ghost15& record : accepted.ghosts) {
        input_valid = counted15(step.work, finite15(record.position))
            && input_valid;
        const bool ordered = first_id || record.id > prior_id;
        input_valid = counted15(step.work, ordered) && input_valid;
        prior_id = record.id;
        first_id = false;
    }
    if (!profile_valid || !input_valid) {
        step.outcome = profile_valid ? "INVALID_INPUT" : "INVALID_PROFILE";
        step.expected_work.fixture_records = accepted.dynamic.size();
        step.expected_work.ghost_records = accepted.ghosts.size();
        step.expected_work.scalar_predicates = kProfilePredicateCount15
            + kOptionPredicateCount15 + 2U
            + 4U * accepted.dynamic.size()
            + 2U * accepted.ghosts.size();
        return step;
    }

    bool reference_valid = true;
    bool reference_capacity_overflow = false;
    std::vector<ReferencePair15> frozen_pairs;
    if ((options.terms.normal_viscosity
            || options.terms.tangential_viscosity)
            && options.mutation
                != Mutation15::CurrentReferenceViscosityGraph) {
        frozen_pairs = reference_pairs15(
            accepted, options, step.work, reference_valid,
            reference_capacity_overflow);
    }
    if (!reference_valid) {
        step.outcome = reference_capacity_overflow
            ? "CAPACITY_OVERFLOW" : "INVALID_REFERENCE_GRAPH";
        step.expected_work = expected_solver_work15(
            accepted, options, step.trace);
        return step;
    }

    const std::size_t count = accepted.dynamic.size();
    std::vector<long double> pi(count, 0.0L);
    std::vector<Vec3l> y;
    {
        Work15 projection_work;
        const bool projection_valid = project15(prediction15(accepted,
            options), options, y, projection_work);
        add_work15(step.work, projection_work);
        step.trace.reached_stages.push_back("PROJECTION");
        if (projection_valid
                && options.boundary_mode == BoundaryMode15::AnalyticBox) {
            step.trace.reached_stages.push_back("PROJECTION_BOX");
        }
        if (!projection_valid) {
            step.outcome = "NONFINITE";
            step.expected_work = expected_solver_work15(
                accepted, options, step.trace);
            return step;
        }
    }
    std::uint64_t accepted_inner = 0U;

    auto evaluate_recorded = [&](const std::vector<Vec3l>& point,
                                 const std::vector<long double>& multiplier,
                                 MultiplierMode15 mode, bool gradient) {
        step.trace.evaluation_positions.push_back(point);
        step.trace.evaluation_multipliers.push_back(multiplier);
        step.trace.evaluation_modes.push_back(mode
                == MultiplierMode15::FinalKkt
            ? std::uint8_t{1U} : std::uint8_t{0U});
        step.trace.evaluation_gradient.push_back(
            gradient ? std::uint8_t{1U} : std::uint8_t{0U});
        return evaluate15(accepted, point, multiplier, options, mode,
            gradient, &frozen_pairs, step.work);
    };

    for (std::uint32_t outer = 0U;
            outer < options.profile.maximum_outer_updates; ++outer) {
        bool inner_converged = false;
        EnergyGradient15 at_inner;
        while (!inner_converged) {
            EnergyGradient15 current = evaluate_recorded(y, pi,
                MultiplierMode15::AugmentedLagrangian, true);
            if (!current.valid) {
                step.outcome = current.outcome;
                publish_private_state15(
                    step, accepted, y, pi, nullptr, options);
                step.expected_work = expected_solver_work15(
                    accepted, options, step.trace);
                return step;
            }
            const long double scale = options.profile.dt
                * options.profile.dt / options.profile.mass;
            std::vector<Vec3l> raw(y.size());
            for (std::size_t index = 0U; index < y.size(); ++index) {
                raw[index] = y[index] - scale * current.gradient[index];
            }
            Work15 projection_work;
            std::vector<Vec3l> projected;
            const bool projection_valid = project15(
                raw, options, projected, projection_work);
            add_work15(step.work, projection_work);
            step.trace.reached_stages.push_back("PROJECTION");
            if (projection_valid
                    && options.boundary_mode == BoundaryMode15::AnalyticBox) {
                step.trace.reached_stages.push_back("PROJECTION_BOX");
            }
            if (!projection_valid) {
                step.outcome = "NONFINITE";
                publish_private_state15(
                    step, accepted, y, pi, &current, options);
                step.expected_work = expected_solver_work15(
                    accepted, options, step.trace);
                return step;
            }
            ++step.work.projected_directions;
            step.work.projected_directions += y.size() - 1U;
            std::vector<Vec3l> direction(y.size());
            for (std::size_t index = 0U; index < y.size(); ++index) {
                direction[index] = projected[index] - y[index];
            }
            step.trace.reached_stages.push_back("INNER_GATE");
            const long double relative = vector_norm15(direction)
                / std::max(options.profile.spacing
                    * std::sqrt(3.0L * static_cast<long double>(count)),
                    1.0e-30L);
            const long double maximum = max_component15(direction)
                / options.profile.spacing;
            const bool rms_pass = counted15(step.work,
                relative <= options.profile.gates.inner_rms_relative);
            const bool maximum_pass = counted15(step.work,
                maximum <= options.profile.gates.inner_max_relative);
            if (rms_pass && maximum_pass) {
                at_inner = std::move(current);
                inner_converged = true;
                break;
            }
            step.trace.reached_stages.push_back("SLOPE");
            const long double slope = [&]() {
                std::vector<long double> terms;
                terms.reserve(count);
                for (std::size_t index = 0U; index < count; ++index) {
                    terms.push_back(dot15(
                        current.gradient[index], direction[index]));
                }
                return pairwise_sum15(std::move(terms));
            }();
            if (!counted15(step.work, slope < 0.0L)) {
                step.outcome = "NON_DESCENT";
                publish_private_state15(
                    step, accepted, y, pi, &current, options);
                step.expected_work = expected_solver_work15(
                    accepted, options, step.trace);
                return step;
            }
            if (accepted_inner
                    >= options.profile.maximum_accepted_inner_iterations) {
                step.outcome = "SOLVER_WORK_CEILING_INCONCLUSIVE";
                step.work_ceiling = true;
                step.apparatus_valid = true;
                publish_private_state15(
                    step, accepted, y, pi, &current, options);
                step.expected_work = expected_solver_work15(
                    accepted, options, step.trace);
                return step;
            }

            bool line_accepted = false;
            std::vector<Vec3l> trial(y.size());
            long double accepted_objective = 0.0L;
            std::uint32_t accepted_backtracks = 0U;
            long double alpha = options.profile.initial_alpha;
            for (std::uint32_t backtrack = 0U;
                    backtrack <= options.profile.maximum_line_search_backtracks;
                    ++backtrack) {
                for (std::size_t index = 0U; index < y.size(); ++index) {
                    trial[index] = y[index] + alpha * direction[index];
                }
                EnergyGradient15 trial_value = evaluate_recorded(trial, pi,
                    MultiplierMode15::AugmentedLagrangian, false);
                ++step.work.line_search_trials;
                step.trace.reached_stages.push_back("LINE_TRIAL");
                if (!trial_value.valid) {
                    step.outcome = trial_value.outcome;
                    publish_private_state15(
                        step, accepted, trial, pi, nullptr, options);
                    step.expected_work = expected_solver_work15(
                        accepted, options, step.trace);
                    return step;
                }
                const bool armijo = trial_value.objective
                    <= current.objective
                        + options.profile.armijo * alpha * slope;
                step.trace.reached_stages.push_back("ARMIJO");
                if (counted15(step.work, armijo)) {
                    line_accepted = true;
                    accepted_objective = trial_value.objective;
                    accepted_backtracks = backtrack;
                    break;
                }
                if (backtrack
                        < options.profile.maximum_line_search_backtracks) {
                    alpha *= options.profile.line_search_shrink;
                }
            }
            if (!line_accepted) {
                step.outcome = "LINE_SEARCH_EXHAUSTED";
                publish_private_state15(
                    step, accepted, y, pi, &current, options);
                step.expected_work = expected_solver_work15(
                    accepted, options, step.trace);
                return step;
            }
            y = trial;
            ++accepted_inner;
            ++step.work.accepted_inner_iterations;
            step.trace.accepted_inner_iterations = accepted_inner;
            step.trace.backtracks.push_back(accepted_backtracks);
            step.trace.accepted_objectives.push_back(accepted_objective);
        }

        step.trace.reached_stages.push_back("MULTIPLIER_UPDATE");
        for (std::size_t index = 0U; index < count; ++index) {
            pi[index] = options.mutation
                        == Mutation15::FinitePressurePenalty
                    || !options.terms.density_constraint
                ? 0.0L
                : std::max(0.0L, pi[index]
                    + options.profile.beta
                        * at_inner.density.constraint[index]);
            (void)counted15(step.work, finite_scalar15(pi[index]));
        }
        ++step.work.outer_multiplier_updates;
        step.trace.outer_updates = outer + 1U;
        step.trace.outer_inner_offsets.push_back(accepted_inner);

        EnergyGradient15 final = evaluate_recorded(y, pi,
            MultiplierMode15::FinalKkt, true);
        if (!final.valid) {
            step.outcome = final.outcome;
            publish_private_state15(
                step, accepted, y, pi, nullptr, options);
            step.expected_work = expected_solver_work15(
                accepted, options, step.trace);
            return step;
        }
        publish_final_records15(
            step, accepted, y, pi, final, options);
        if (kkt_gates15(step, accepted, y, pi, final, options, step.trace)) {
            const Graph15 topology = build_graph15(
                accepted, y, options, step.work);
            if (!topology.valid) {
                step.outcome = "FINAL_GRAPH_INVALID";
                step.expected_work = expected_solver_work15(
                    accepted, options, step.trace);
                return step;
            }
            step.trace.reached_stages.push_back("TOPOLOGY");
            step.work.topology_edges += accepted.dynamic.size()
                + 2U * topology.dynamic_pairs.size();
            step.outcome = "PASS";
            step.apparatus_valid = true;
            step.accepted = true;
            step.expected_work = expected_solver_work15(
                accepted, options, step.trace);
            return step;
        }
        if (step.outcome == "NONFINITE") {
            publish_private_state15(
                step, accepted, y, pi, &final, options);
            step.expected_work = expected_solver_work15(
                accepted, options, step.trace);
            return step;
        }
    }

    step.outcome = "SOLVER_WORK_CEILING_INCONCLUSIVE";
    step.apparatus_valid = true;
    step.work_ceiling = true;
    step.expected_work = expected_solver_work15(
        accepted, options, step.trace);
    return step;
}

} // namespace nextengine::nonlocal::ncgp15
