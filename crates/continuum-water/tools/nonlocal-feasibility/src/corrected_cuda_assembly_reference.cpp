#include "corrected_cuda_assembly.hpp"

#include <algorithm>
#include <cmath>
#include <limits>
#include <unordered_set>
#include <utility>

namespace nextengine::nonlocal::gpu_assembly_audit {
namespace {

constexpr long double PI =
    3.141592653589793238462643383279502884L;

struct Vec3 {
    long double x = 0.0L;
    long double y = 0.0L;
    long double z = 0.0L;
};

struct CanonicalState {
    std::uint32_t id = 0;
    std::array<std::int64_t, 3> reference_um{};
    std::array<std::int64_t, 3> predicted_um{};
    std::array<std::int64_t, 3> current_um{};
    Vec3 reference;
    Vec3 predicted;
    Vec3 current;
    Vec3 direction;
};

struct Graph {
    std::vector<std::uint32_t> offsets;
    std::vector<std::uint32_t> neighbor_rows;
    std::vector<std::uint32_t> neighbor_ids;
};

Vec3 operator+(Vec3 lhs, Vec3 rhs) {
    return {lhs.x + rhs.x, lhs.y + rhs.y, lhs.z + rhs.z};
}

Vec3 operator-(Vec3 lhs, Vec3 rhs) {
    return {lhs.x - rhs.x, lhs.y - rhs.y, lhs.z - rhs.z};
}

Vec3 operator-(Vec3 value) { return {-value.x, -value.y, -value.z}; }

Vec3 operator*(long double scalar, Vec3 value) {
    return {scalar * value.x, scalar * value.y, scalar * value.z};
}

Vec3& operator+=(Vec3& lhs, Vec3 rhs) {
    lhs = lhs + rhs;
    return lhs;
}

long double dot(Vec3 lhs, Vec3 rhs) {
    return lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z;
}

long double norm(Vec3 value) { return std::sqrt(dot(value, value)); }

long double component(Vec3 value, std::size_t axis) {
    return axis == 0U ? value.x : (axis == 1U ? value.y : value.z);
}

Vec3 from_um(const std::array<std::int64_t, 3>& value) {
    constexpr long double scale = 1.0L / 1000000.0L;
    return {scale * value[0], scale * value[1], scale * value[2]};
}

Vec3 from_milli(const std::array<std::int32_t, 3>& value) {
    constexpr long double scale = 1.0L / 1000.0L;
    return {scale * value[0], scale * value[1], scale * value[2]};
}

bool pair_inside(const std::array<std::int64_t, 3>& lhs,
    const std::array<std::int64_t, 3>& rhs) {
    std::uint64_t squared = 0U;
    for (std::size_t axis = 0; axis < 3U; ++axis) {
        const std::int64_t difference = lhs[axis] - rhs[axis];
        const std::uint64_t magnitude = static_cast<std::uint64_t>(
            difference < 0 ? -difference : difference);
        if (magnitude > static_cast<std::uint64_t>(ASSEMBLY_SUPPORT_UM)) {
            return false;
        }
        squared += magnitude * magnitude;
    }
    return squared <= static_cast<std::uint64_t>(
        ASSEMBLY_SUPPORT_UM * ASSEMBLY_SUPPORT_UM);
}

long double cubic_weight(long double radius, long double horizon) {
    const long double q = 2.0L * radius / horizon;
    const long double alpha = 3.0L / (2.0L * PI * horizon * horizon * horizon);
    if (q > 2.0L) return 0.0L;
    if (q >= 1.0L) {
        const long double delta = 2.0L - q;
        return alpha * delta * delta * delta / 6.0L;
    }
    return alpha * (2.0L / 3.0L - q * q + 0.5L * q * q * q);
}

long double cubic_gradient(long double radius, long double horizon) {
    const long double q = 2.0L * radius / horizon;
    const long double alpha = 3.0L / (2.0L * PI * horizon * horizon * horizon);
    if (q > 2.0L) return 0.0L;
    const long double derivative_q = q >= 1.0L
        ? -0.5L * alpha * (2.0L - q) * (2.0L - q)
        : alpha * (-2.0L * q + 1.5L * q * q);
    return derivative_q * 2.0L / horizon;
}

long double cubic_second(long double radius, long double horizon) {
    const long double q = 2.0L * radius / horizon;
    const long double alpha = 3.0L / (2.0L * PI * horizon * horizon * horizon);
    if (q > 2.0L) return 0.0L;
    const long double second_q = q >= 1.0L
        ? alpha * (2.0L - q)
        : alpha * (-2.0L + 3.0L * q);
    return second_q * 4.0L / (horizon * horizon);
}

long double surface_spline(long double radius, long double spacing) {
    const long double q = radius / spacing;
    if (q <= 1.0L) return q * q - 1.0L;
    if (q < 3.0L) return 1.0L - (q - 2.0L) * (q - 2.0L);
    return 0.0L;
}

long double surface_derivative(long double radius, long double spacing) {
    const long double q = radius / spacing;
    if (q <= 1.0L) return 2.0L * q / spacing;
    if (q < 3.0L) return -2.0L * (q - 2.0L) / spacing;
    return 0.0L;
}

long double surface_potential(long double radius, long double spacing) {
    const long double q = radius / spacing;
    if (q <= 1.0L) {
        return spacing * (q * q * q / 3.0L - q - 2.0L / 3.0L);
    }
    if (q < 3.0L) {
        return spacing * (q - (q - 2.0L) * (q - 2.0L) * (q - 2.0L)
            / 3.0L - 8.0L / 3.0L);
    }
    return 0.0L;
}

long double radial_entry(Vec3 normal, long double radial,
    long double tangential, std::size_t row_axis, std::size_t column_axis) {
    const long double nr = component(normal, row_axis);
    const long double nc = component(normal, column_axis);
    return tangential * (row_axis == column_axis ? 1.0L : 0.0L)
        + (radial - tangential) * nr * nc;
}

std::vector<CanonicalState> canonicalize(const AssemblyFixture& fixture,
    AssemblyFailure& failure, std::uint64_t& sort_comparisons) {
    if (fixture.samples.empty()) {
        failure = AssemblyFailure::InvalidInput;
        return {};
    }
    if (fixture.samples.size() > ASSEMBLY_MAX_SAMPLES) {
        failure = AssemblyFailure::CapacityExceeded;
        return {};
    }
    std::unordered_set<std::uint32_t> ids;
    std::vector<CanonicalState> result;
    result.reserve(fixture.samples.size());
    for (const AssemblySample& sample : fixture.samples) {
        if (!ids.insert(sample.sample_id).second) {
            failure = AssemblyFailure::DuplicateSampleId;
            return {};
        }
        const auto coordinates_valid = [](const auto& value) {
            for (const auto component_value : value) {
                if (component_value < -1000000000LL
                    || component_value > 1000000000LL) return false;
            }
            return true;
        };
        if (!coordinates_valid(sample.reference_um)
            || !coordinates_valid(sample.predicted_um)
            || !coordinates_valid(sample.current_um)) {
            failure = AssemblyFailure::InvalidInput;
            return {};
        }
        result.push_back({sample.sample_id, sample.reference_um,
            sample.predicted_um, sample.current_um,
            from_um(sample.reference_um), from_um(sample.predicted_um),
            from_um(sample.current_um), from_milli(sample.direction_milli)});
    }
    for (std::size_t first = 0; first < result.size(); ++first) {
        std::size_t selected = first;
        for (std::size_t candidate = first + 1U;
             candidate < result.size(); ++candidate) {
            ++sort_comparisons;
            if (result[candidate].id < result[selected].id) selected = candidate;
        }
        if (selected != first) std::swap(result[first], result[selected]);
    }
    failure = AssemblyFailure::None;
    return result;
}

Graph build_graph(const std::vector<CanonicalState>& state, bool current,
    std::uint64_t& predicates) {
    Graph graph;
    graph.offsets.reserve(state.size() + 1U);
    graph.offsets.push_back(0U);
    for (std::size_t row = 0; row < state.size(); ++row) {
        const auto& owner = current ? state[row].current_um : state[row].reference_um;
        for (std::size_t candidate = 0; candidate < state.size(); ++candidate) {
            ++predicates;
            const auto& other = current ? state[candidate].current_um
                                        : state[candidate].reference_um;
            if (pair_inside(owner, other)) {
                graph.neighbor_rows.push_back(static_cast<std::uint32_t>(candidate));
                graph.neighbor_ids.push_back(state[candidate].id);
            }
        }
        graph.offsets.push_back(static_cast<std::uint32_t>(graph.neighbor_rows.size()));
    }
    return graph;
}

bool profile_valid(const AssemblyProfile& p) {
    return std::isfinite(p.horizon) && std::isfinite(p.spacing)
        && std::isfinite(p.mass) && std::isfinite(p.time_step)
        && std::isfinite(p.rest_density) && std::isfinite(p.kappa)
        && std::isfinite(p.lambda) && std::isfinite(p.mu)
        && std::isfinite(p.gamma) && p.horizon == 0.15
        && p.spacing == 0.05 && p.mass > 0.0 && p.time_step > 0.0
        && p.rest_density > 0.0 && p.kappa >= 0.0 && p.lambda >= 0.0
        && p.mu >= 0.0 && p.gamma >= 0.0;
}

void add_block(std::vector<long double>& matrix, std::size_t dimension,
    std::size_t first, std::size_t second, Vec3 normal,
    long double radial, long double tangential, long double sign) {
    for (std::size_t row_axis = 0; row_axis < 3U; ++row_axis) {
        for (std::size_t column_axis = 0; column_axis < 3U; ++column_axis) {
            matrix[(3U * first + row_axis) * dimension
                + 3U * second + column_axis] += sign * radial_entry(
                    normal, radial, tangential, row_axis, column_axis);
        }
    }
}

} // namespace

AssemblyResult evaluate_reference_assembly_impl(
    const AssemblyProfile& profile,
    const AssemblyFixture& fixture,
    const std::vector<double>* canonical_current_m) {
    AssemblyResult output;
    if (!profile_valid(profile)) {
        output.failure = AssemblyFailure::InvalidInput;
        return output;
    }
    std::uint64_t sort_comparisons = 0U;
    std::vector<CanonicalState> state = canonicalize(
        fixture, output.failure, sort_comparisons);
    if (output.failure != AssemblyFailure::None) return output;
    if (canonical_current_m != nullptr) {
        if (canonical_current_m->size() != 3U * state.size()) {
            output.failure = AssemblyFailure::InvalidInput;
            return output;
        }
        for (std::size_t row = 0; row < state.size(); ++row) {
            const double x = (*canonical_current_m)[3U * row];
            const double y = (*canonical_current_m)[3U * row + 1U];
            const double z = (*canonical_current_m)[3U * row + 2U];
            if (!std::isfinite(x) || !std::isfinite(y) || !std::isfinite(z)
                || std::abs(x) > 1000.0 || std::abs(y) > 1000.0
                || std::abs(z) > 1000.0) {
                output.failure = AssemblyFailure::InvalidInput;
                return output;
            }
            state[row].current = {static_cast<long double>(x),
                static_cast<long double>(y), static_cast<long double>(z)};
        }
    }

    const std::size_t count = state.size();
    const std::size_t dimension = 3U * count;
    Graph current_graph = build_graph(
        state, true, output.work.graph_distance_predicates);
    Graph reference_graph = build_graph(
        state, false, output.work.graph_distance_predicates);
    output.work.owner_sort_comparisons = sort_comparisons;
    output.current_offsets = current_graph.offsets;
    output.current_neighbors = current_graph.neighbor_ids;
    output.reference_offsets = reference_graph.offsets;
    output.reference_neighbors = reference_graph.neighbor_ids;
    output.owner_ids.reserve(count);
    for (const CanonicalState& sample : state) output.owner_ids.push_back(sample.id);

    const long double h = profile.horizon;
    const long double spacing = profile.spacing;
    const long double mass = profile.mass;
    const long double dt = profile.time_step;
    const long double rho0 = profile.rest_density;
    const long double kappa = profile.kappa;
    const long double lambda = profile.lambda;
    const long double mu = profile.mu;
    const long double gamma = profile.gamma;

    std::vector<long double> density(count, 0.0L);
    for (std::size_t row = 0; row < count; ++row) {
        for (std::size_t slot = current_graph.offsets[row];
             slot < current_graph.offsets[row + 1U]; ++slot) {
            const std::size_t neighbor = current_graph.neighbor_rows[slot];
            density[row] += mass * cubic_weight(
                norm(state[row].current - state[neighbor].current), h);
            ++output.work.density_kernel_evaluations;
        }
    }
    std::vector<long double> compression(count, 0.0L);
    output.density.resize(count);
    output.pressure_active.resize(count);
    long double minimum_density_margin = std::numeric_limits<long double>::infinity();
    for (std::size_t row = 0; row < count; ++row) {
        compression[row] = std::max(density[row] / rho0 - 1.0L, 0.0L);
        output.density[row] = static_cast<double>(density[row]);
        output.pressure_active[row] = compression[row] > 0.0L ? 1U : 0U;
        minimum_density_margin = std::min(minimum_density_margin,
            std::abs(density[row] / rho0 - 1.0L));
        if (compression[row] > 0.0L) ++output.work.active_pressure_centers;
    }
    output.minimum_density_clamp_margin = static_cast<double>(minimum_density_margin);

    std::array<long double, 4> energy{};
    std::vector<Vec3> gradient(count);
    const long double inertia_scale = mass / (dt * dt);
    if (fixture.terms.inertia) {
        for (std::size_t row = 0; row < count; ++row) {
            const Vec3 displacement = state[row].current - state[row].predicted;
            energy[0] += 0.5L * inertia_scale * dot(displacement, displacement);
            gradient[row] += inertia_scale * displacement;
        }
    }
    if (fixture.terms.pressure) {
        for (long double value : compression) energy[1] += 0.5L * kappa * value * value;
        for (std::size_t first = 0; first < count; ++first) {
            for (std::size_t slot = current_graph.offsets[first];
                 slot < current_graph.offsets[first + 1U]; ++slot) {
                const std::size_t second = current_graph.neighbor_rows[slot];
                if (second <= first) continue;
                ++output.work.energy_pair_visits;
                ++output.work.gradient_pair_visits;
                const Vec3 displacement = state[first].current - state[second].current;
                const long double radius = norm(displacement);
                if (radius <= 1.0e-15L) continue;
                const Vec3 normal = (1.0L / radius) * displacement;
                const long double coefficient = kappa * mass / rho0
                    * (compression[first] + compression[second])
                    * cubic_gradient(radius, h);
                const Vec3 pair = coefficient * normal;
                gradient[first] += pair;
                gradient[second] += -pair;
            }
        }
    }
    if (fixture.terms.viscosity) {
        for (std::size_t first = 0; first < count; ++first) {
            for (std::size_t slot = reference_graph.offsets[first];
                 slot < reference_graph.offsets[first + 1U]; ++slot) {
                const std::size_t second = reference_graph.neighbor_rows[slot];
                if (second <= first) continue;
                ++output.work.energy_pair_visits;
                ++output.work.gradient_pair_visits;
                const Vec3 reference = state[first].reference - state[second].reference;
                const long double radius = norm(reference);
                if (radius <= 1.0e-15L) continue;
                const Vec3 normal = (1.0L / radius) * reference;
                const Vec3 increment = (state[first].current - state[second].current)
                    - reference;
                const Vec3 normal_increment = dot(increment, normal) * normal;
                const Vec3 tangent_increment = increment - normal_increment;
                const long double omega = -cubic_gradient(radius, h);
                energy[2] += mass * omega / (rho0 * dt)
                    * (mu * dot(tangent_increment, tangent_increment)
                        + 0.5L * lambda * dot(normal_increment, normal_increment));
                const Vec3 pair = mass * omega / (rho0 * dt)
                    * (2.0L * mu * tangent_increment + lambda * normal_increment);
                gradient[first] += pair;
                gradient[second] += -pair;
            }
        }
    }
    if (fixture.terms.surface) {
        for (std::size_t first = 0; first < count; ++first) {
            for (std::size_t slot = current_graph.offsets[first];
                 slot < current_graph.offsets[first + 1U]; ++slot) {
                const std::size_t second = current_graph.neighbor_rows[slot];
                if (second <= first) continue;
                ++output.work.energy_pair_visits;
                ++output.work.gradient_pair_visits;
                const Vec3 displacement = state[first].current - state[second].current;
                const long double radius = norm(displacement);
                if (radius <= 1.0e-15L || radius >= 3.0L * spacing) continue;
                const Vec3 normal = (1.0L / radius) * displacement;
                energy[3] += 2.0L * gamma * mass * mass
                    * surface_potential(radius, spacing);
                const Vec3 pair = 2.0L * gamma * mass * mass
                    * surface_spline(radius, spacing) * normal;
                gradient[first] += pair;
                gradient[second] += -pair;
            }
        }
    }

    output.gradient.resize(dimension);
    for (std::size_t row = 0; row < count; ++row) {
        for (std::size_t axis = 0; axis < 3U; ++axis) {
            output.gradient[3U * row + axis] = static_cast<double>(
                component(gradient[row], axis));
        }
    }
    for (std::size_t index = 0; index < energy.size(); ++index) {
        output.energy[index] = static_cast<double>(energy[index]);
    }

    std::vector<long double> hessian(dimension * dimension, 0.0L);
    if (fixture.terms.inertia) {
        for (std::size_t scalar = 0; scalar < dimension; ++scalar) {
            hessian[scalar * dimension + scalar] += inertia_scale;
        }
    }
    if (fixture.terms.pressure) {
        std::vector<long double> jacobian(count * dimension, 0.0L);
        for (std::size_t center = 0; center < count; ++center) {
            if (compression[center] <= 0.0L) continue;
            for (std::size_t slot = current_graph.offsets[center];
                 slot < current_graph.offsets[center + 1U]; ++slot) {
                const std::size_t neighbor = current_graph.neighbor_rows[slot];
                if (neighbor == center) continue;
                const Vec3 displacement = state[center].current - state[neighbor].current;
                const long double radius = norm(displacement);
                if (radius <= 1.0e-15L) continue;
                const Vec3 pair = mass / rho0 * cubic_gradient(radius, h)
                    * ((1.0L / radius) * displacement);
                for (std::size_t axis = 0; axis < 3U; ++axis) {
                    jacobian[center * dimension + 3U * center + axis]
                        += component(pair, axis);
                    jacobian[center * dimension + 3U * neighbor + axis]
                        -= component(pair, axis);
                }
            }
            for (std::size_t row = 0; row < dimension; ++row) {
                const long double jr = jacobian[center * dimension + row];
                if (jr == 0.0L) continue;
                for (std::size_t column = 0; column < dimension; ++column) {
                    const long double jc = jacobian[center * dimension + column];
                    if (jc == 0.0L) continue;
                    hessian[row * dimension + column] += kappa * jr * jc;
                    ++output.work.pressure_outer_products;
                }
            }
            for (std::size_t slot = current_graph.offsets[center];
                 slot < current_graph.offsets[center + 1U]; ++slot) {
                const std::size_t neighbor = current_graph.neighbor_rows[slot];
                if (neighbor == center) continue;
                const Vec3 displacement = state[center].current - state[neighbor].current;
                const long double radius = norm(displacement);
                if (radius <= 1.0e-15L) continue;
                const Vec3 normal = (1.0L / radius) * displacement;
                const long double radial = kappa * compression[center] * mass
                    / rho0 * cubic_second(radius, h);
                const long double tangential = kappa * compression[center] * mass
                    / rho0 * cubic_gradient(radius, h) / radius;
                add_block(hessian, dimension, center, center,
                    normal, radial, tangential, 1.0L);
                add_block(hessian, dimension, center, neighbor,
                    normal, radial, tangential, -1.0L);
                add_block(hessian, dimension, neighbor, center,
                    normal, radial, tangential, -1.0L);
                add_block(hessian, dimension, neighbor, neighbor,
                    normal, radial, tangential, 1.0L);
                output.work.pressure_geometric_blocks += 4U;
            }
        }
    }
    if (fixture.terms.viscosity) {
        for (std::size_t first = 0; first < count; ++first) {
            for (std::size_t slot = reference_graph.offsets[first];
                 slot < reference_graph.offsets[first + 1U]; ++slot) {
                const std::size_t second = reference_graph.neighbor_rows[slot];
                if (second <= first) continue;
                const Vec3 reference = state[first].reference - state[second].reference;
                const long double radius = norm(reference);
                if (radius <= 1.0e-15L) continue;
                const Vec3 normal = (1.0L / radius) * reference;
                const long double scale = mass * (-cubic_gradient(radius, h))
                    / (rho0 * dt);
                const long double radial = scale * lambda;
                const long double tangential = scale * 2.0L * mu;
                add_block(hessian, dimension, first, first,
                    normal, radial, tangential, 1.0L);
                add_block(hessian, dimension, first, second,
                    normal, radial, tangential, -1.0L);
                add_block(hessian, dimension, second, first,
                    normal, radial, tangential, -1.0L);
                add_block(hessian, dimension, second, second,
                    normal, radial, tangential, 1.0L);
                output.work.viscosity_curvature_blocks += 4U;
            }
        }
    }
    if (fixture.terms.surface) {
        for (std::size_t first = 0; first < count; ++first) {
            for (std::size_t slot = current_graph.offsets[first];
                 slot < current_graph.offsets[first + 1U]; ++slot) {
                const std::size_t second = current_graph.neighbor_rows[slot];
                if (second <= first) continue;
                const Vec3 displacement = state[first].current - state[second].current;
                const long double radius = norm(displacement);
                if (radius <= 1.0e-15L || radius >= 3.0L * spacing) continue;
                const Vec3 normal = (1.0L / radius) * displacement;
                const long double scale = 2.0L * gamma * mass * mass;
                const long double radial = scale * surface_derivative(radius, spacing);
                const long double tangential = scale
                    * surface_spline(radius, spacing) / radius;
                add_block(hessian, dimension, first, first,
                    normal, radial, tangential, 1.0L);
                add_block(hessian, dimension, first, second,
                    normal, radial, tangential, -1.0L);
                add_block(hessian, dimension, second, first,
                    normal, radial, tangential, -1.0L);
                add_block(hessian, dimension, second, second,
                    normal, radial, tangential, 1.0L);
                output.work.surface_curvature_blocks += 4U;
            }
        }
    }

    output.hessian.resize(dimension * dimension);
    output.diagonal_blocks.resize(count * 9U);
    output.hvp.assign(dimension, 0.0);
    for (std::size_t row = 0; row < dimension; ++row) {
        for (std::size_t column = 0; column < dimension; ++column) {
            const double value = static_cast<double>(
                hessian[row * dimension + column]);
            output.hessian[row * dimension + column] = value;
            output.hvp[row] += value * static_cast<double>(component(
                state[column / 3U].direction, column % 3U));
            ++output.work.hvp_products;
        }
    }
    output.work.dense_entries_written = dimension * dimension;
    for (std::size_t particle = 0; particle < count; ++particle) {
        for (std::size_t row_axis = 0; row_axis < 3U; ++row_axis) {
            for (std::size_t column_axis = 0; column_axis < 3U; ++column_axis) {
                output.diagonal_blocks[9U * particle + 3U * row_axis + column_axis]
                    = output.hessian[(3U * particle + row_axis) * dimension
                        + 3U * particle + column_axis];
            }
        }
    }

    auto minimum_margin = [&state](bool current) {
        std::int64_t margin = std::numeric_limits<std::int64_t>::max();
        for (std::size_t first = 0; first < state.size(); ++first) {
            for (std::size_t second = first + 1U; second < state.size(); ++second) {
                const Vec3 delta = current
                    ? state[first].current - state[second].current
                    : state[first].reference - state[second].reference;
                const long double radius_um = norm(delta) * 1000000.0L;
                margin = std::min(margin, static_cast<std::int64_t>(
                    std::llround(std::abs(radius_um - ASSEMBLY_SUPPORT_UM))));
            }
        }
        return margin == std::numeric_limits<std::int64_t>::max() ? 0 : margin;
    };
    output.minimum_current_support_margin_um = minimum_margin(true);
    output.minimum_reference_support_margin_um = minimum_margin(false);
    std::int64_t surface_margin = std::numeric_limits<std::int64_t>::max();
    for (std::size_t first = 0; first < count; ++first) {
        for (std::size_t second = first + 1U; second < count; ++second) {
            const long double radius_um = norm(
                state[first].current - state[second].current) * 1000000.0L;
            for (std::int64_t branch : {50000LL, 150000LL}) {
                surface_margin = std::min(surface_margin,
                    static_cast<std::int64_t>(std::llround(
                        std::abs(radius_um - static_cast<long double>(branch)))));
            }
        }
    }
    output.minimum_surface_branch_margin_um =
        surface_margin == std::numeric_limits<std::int64_t>::max()
        ? 0 : surface_margin;
    output.failure = AssemblyFailure::None;
    return output;
}

AssemblyResult evaluate_reference_assembly(
    const AssemblyProfile& profile,
    const AssemblyFixture& fixture) {
    return evaluate_reference_assembly_impl(profile, fixture, nullptr);
}

AssemblyResult evaluate_reference_assembly_at(
    const AssemblyProfile& profile,
    const AssemblyFixture& fixture,
    const std::vector<double>& canonical_current_m) {
    return evaluate_reference_assembly_impl(
        profile, fixture, &canonical_current_m);
}

} // namespace nextengine::nonlocal::gpu_assembly_audit
