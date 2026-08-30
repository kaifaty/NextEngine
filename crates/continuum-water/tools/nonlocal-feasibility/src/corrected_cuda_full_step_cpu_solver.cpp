#include "corrected_cuda_full_step.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <limits>
#include <map>
#include <stdexcept>
#include <tuple>
#include <unordered_set>
#include <utility>
#include <vector>

namespace nextengine::nonlocal::gpu_full_step {
namespace {

constexpr long double kUmPerMetre = 1000000.0L;

struct CpuVec3 {
    long double x = 0.0L;
    long double y = 0.0L;
    long double z = 0.0L;
};

CpuVec3 make_vec(const Vec3d& value) {
    return {value.x, value.y, value.z};
}

Vec3d narrow(CpuVec3 value) {
    return {static_cast<double>(value.x), static_cast<double>(value.y),
        static_cast<double>(value.z)};
}

CpuVec3 add(CpuVec3 lhs, CpuVec3 rhs) {
    return {lhs.x + rhs.x, lhs.y + rhs.y, lhs.z + rhs.z};
}

CpuVec3 subtract(CpuVec3 lhs, CpuVec3 rhs) {
    return {lhs.x - rhs.x, lhs.y - rhs.y, lhs.z - rhs.z};
}

CpuVec3 scale(CpuVec3 value, long double factor) {
    return {value.x * factor, value.y * factor, value.z * factor};
}

long double dot(CpuVec3 lhs, CpuVec3 rhs) {
    return lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z;
}

long double length(CpuVec3 value) {
    return std::sqrt(std::max(dot(value, value), 0.0L));
}

bool finite(CpuVec3 value) {
    return std::isfinite(value.x) && std::isfinite(value.y)
        && std::isfinite(value.z);
}

CpuVec3 radial_apply(CpuVec3 normal,
    long double radial,
    long double tangential,
    CpuVec3 value) {
    return add(scale(value, tangential),
        scale(normal, (radial - tangential) * dot(normal, value)));
}

struct KernelValues {
    long double value = 0.0L;
    long double first = 0.0L;
    long double second = 0.0L;
};

KernelValues kernel(long double radius,
    const NonlocalGpuProfile& profile,
    bool missing_chain) {
    const long double pi = std::acos(-1.0L);
    const long double h = profile.horizon;
    const long double q = 2.0L * radius / h;
    const long double alpha = 3.0L / (2.0L * pi * h * h * h);
    long double value = 0.0L;
    long double first_q = 0.0L;
    long double second_q = 0.0L;
    if (q < 1.0L) {
        value = alpha * (2.0L / 3.0L - q * q + 0.5L * q * q * q);
        first_q = alpha * (-2.0L * q + 1.5L * q * q);
        second_q = alpha * (-2.0L + 3.0L * q);
    } else if (q <= 2.0L) {
        const long double tail = 2.0L - q;
        value = alpha * tail * tail * tail / 6.0L;
        first_q = -0.5L * alpha * tail * tail;
        second_q = alpha * tail;
    }
    const long double chain = 2.0L / h;
    return {value, first_q * (missing_chain ? 1.0L : chain),
        second_q * (missing_chain ? chain : chain * chain)};
}

void surface(long double radius,
    long double spacing,
    long double& potential,
    long double& force,
    long double& derivative) {
    const long double q = radius / spacing;
    if (q <= 1.0L) {
        force = q * q - 1.0L;
        derivative = 2.0L * q / spacing;
        potential = spacing * (q * q * q / 3.0L - q - 2.0L / 3.0L);
    } else if (q < 3.0L) {
        const long double shifted = q - 2.0L;
        force = 1.0L - shifted * shifted;
        derivative = -2.0L * shifted / spacing;
        potential = spacing
            * (q - shifted * shifted * shifted / 3.0L - 8.0L / 3.0L);
    } else {
        potential = 0.0L;
        force = 0.0L;
        derivative = 0.0L;
    }
}

std::int64_t quantize(long double value) {
    if (!std::isfinite(value)) throw std::invalid_argument("nonfinite CPU point");
    return static_cast<std::int64_t>(std::llround(value * kUmPerMetre));
}

std::int64_t floor_division(std::int64_t numerator, std::int64_t denominator) {
    const std::int64_t quotient = numerator / denominator;
    const std::int64_t remainder = numerator % denominator;
    return quotient - (remainder < 0 ? 1 : 0);
}

std::uint64_t square(std::int64_t value) {
    const auto magnitude = value < 0
        ? static_cast<std::uint64_t>(-value)
        : static_cast<std::uint64_t>(value);
    return magnitude * magnitude;
}

struct Cell {
    std::int64_t x = 0;
    std::int64_t y = 0;
    std::int64_t z = 0;

    bool operator<(const Cell& other) const {
        return std::tie(x, y, z) < std::tie(other.x, other.y, other.z);
    }
};

struct CpuGraph {
    NonlocalGpuFailure failure = NonlocalGpuFailure::None;
    std::vector<std::uint32_t> offsets;
    std::vector<std::uint32_t> neighbors;
    std::uint32_t maximum_degree = 0U;
};

CpuGraph build_cpu_direct_graph(const NonlocalGpuProfile& profile,
    const std::vector<std::uint32_t>& dynamic_ids,
    const std::vector<CpuVec3>& dynamic_positions,
    const std::vector<std::uint32_t>& ghost_ids,
    const std::vector<CpuVec3>& ghost_positions,
    bool strict) {
    CpuGraph result;
    std::vector<std::uint32_t> ids = dynamic_ids;
    std::vector<CpuVec3> positions = dynamic_positions;
    ids.insert(ids.end(), ghost_ids.begin(), ghost_ids.end());
    positions.insert(positions.end(), ghost_positions.begin(), ghost_positions.end());
    std::vector<std::array<std::int64_t, 3>> micrometres;
    micrometres.reserve(positions.size());
    for (const CpuVec3& position : positions) {
        micrometres.push_back({quantize(position.x), quantize(position.y),
            quantize(position.z)});
    }
    const std::int64_t support = quantize(profile.horizon);
    const std::uint64_t support_squared = static_cast<std::uint64_t>(support)
        * static_cast<std::uint64_t>(support);
    for (std::size_t owner = 0U; owner < dynamic_positions.size(); ++owner) {
        std::vector<std::uint32_t> row;
        for (std::size_t candidate = 0U; candidate < positions.size(); ++candidate) {
            const std::uint64_t distance = square(
                    micrometres[owner][0] - micrometres[candidate][0])
                + square(micrometres[owner][1] - micrometres[candidate][1])
                + square(micrometres[owner][2] - micrometres[candidate][2]);
            if ((strict && distance < support_squared)
                || (!strict && distance <= support_squared)) {
                row.push_back(static_cast<std::uint32_t>(candidate));
            }
        }
        if (row.size() > profile.maximum_neighbors) {
            result.failure = NonlocalGpuFailure::CapacityExceeded;
            return result;
        }
        std::sort(row.begin(), row.end(), [&](std::uint32_t lhs,
                                                  std::uint32_t rhs) {
            return ids[lhs] < ids[rhs];
        });
        result.maximum_degree = std::max(result.maximum_degree,
            static_cast<std::uint32_t>(row.size()));
        result.neighbors.insert(result.neighbors.end(), row.begin(), row.end());
        result.offsets.push_back(static_cast<std::uint32_t>(
            result.neighbors.size()));
    }
    result.offsets.insert(result.offsets.begin(), 0U);
    return result;
}

CpuGraph build_cpu_graph(const NonlocalGpuProfile& profile,
    const std::vector<std::uint32_t>& dynamic_ids,
    const std::vector<CpuVec3>& dynamic_positions,
    const std::vector<std::uint32_t>& ghost_ids,
    const std::vector<CpuVec3>& ghost_positions,
    bool strict) {
    CpuGraph result;
    const std::size_t dynamic_count = dynamic_positions.size();
    const std::size_t total_count = dynamic_count + ghost_positions.size();
    std::vector<std::uint32_t> ids = dynamic_ids;
    std::vector<CpuVec3> positions = dynamic_positions;
    ids.insert(ids.end(), ghost_ids.begin(), ghost_ids.end());
    positions.insert(positions.end(), ghost_positions.begin(), ghost_positions.end());
    std::vector<std::array<std::int64_t, 3>> micrometres(total_count);
    const std::int64_t support = quantize(profile.horizon);
    const std::uint64_t support_squared = static_cast<std::uint64_t>(support)
        * static_cast<std::uint64_t>(support);
    std::map<Cell, std::vector<std::uint32_t>> cells;
    try {
        for (std::size_t index = 0U; index < total_count; ++index) {
            micrometres[index] = {quantize(positions[index].x),
                quantize(positions[index].y), quantize(positions[index].z)};
            const auto& p = micrometres[index];
            cells[{floor_division(p[0], support),
                floor_division(p[1], support),
                floor_division(p[2], support)}]
                .push_back(static_cast<std::uint32_t>(index));
        }
    } catch (const std::exception&) {
        result.failure = NonlocalGpuFailure::Nonfinite;
        return result;
    }
    result.offsets.reserve(dynamic_count + 1U);
    result.offsets.push_back(0U);
    for (std::size_t owner = 0U; owner < dynamic_count; ++owner) {
        const auto& p = micrometres[owner];
        const Cell cell{floor_division(p[0], support),
            floor_division(p[1], support), floor_division(p[2], support)};
        std::vector<std::uint32_t> row;
        for (std::int64_t dz = -1; dz <= 1; ++dz) {
            for (std::int64_t dy = -1; dy <= 1; ++dy) {
                for (std::int64_t dx = -1; dx <= 1; ++dx) {
                    const auto found = cells.find(
                        {cell.x + dx, cell.y + dy, cell.z + dz});
                    if (found == cells.end()) continue;
                    for (const std::uint32_t candidate : found->second) {
                        const auto& q = micrometres[candidate];
                        const std::uint64_t distance = square(p[0] - q[0])
                            + square(p[1] - q[1]) + square(p[2] - q[2]);
                        if ((strict && distance < support_squared)
                            || (!strict && distance <= support_squared)) {
                            row.push_back(candidate);
                        }
                    }
                }
            }
        }
        if (row.size() > profile.maximum_neighbors) {
            result.failure = NonlocalGpuFailure::CapacityExceeded;
            return result;
        }
        std::sort(row.begin(), row.end(), [&](std::uint32_t lhs,
                                                  std::uint32_t rhs) {
            return ids[lhs] < ids[rhs];
        });
        result.maximum_degree = std::max(result.maximum_degree,
            static_cast<std::uint32_t>(row.size()));
        result.neighbors.insert(result.neighbors.end(), row.begin(), row.end());
        result.offsets.push_back(static_cast<std::uint32_t>(
            result.neighbors.size()));
    }
    return result;
}

struct CpuEvaluation {
    NonlocalGpuFailure failure = NonlocalGpuFailure::None;
    long double energy = 0.0L;
    std::vector<CpuVec3> gradient;
    std::vector<long double> density;
    std::vector<long double> excess;
    CpuGraph current_graph;
    CpuGraph reference_graph;
    std::uint32_t active = 0U;
};

struct CpuFixture {
    NonlocalGpuProfile profile;
    std::vector<std::uint32_t> ids;
    std::vector<CpuVec3> reference;
    std::vector<CpuVec3> current;
    std::vector<CpuVec3> velocity;
    std::vector<std::uint32_t> ghost_ids;
    std::vector<CpuVec3> ghosts;
};

CpuFixture canonical_fixture(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& input,
    const std::vector<NonlocalGpuGhost>& input_ghosts) {
    CpuFixture result;
    result.profile = profile;
    std::vector<std::size_t> order(input.size());
    for (std::size_t index = 0U; index < order.size(); ++index) order[index] = index;
    std::sort(order.begin(), order.end(), [&](std::size_t lhs, std::size_t rhs) {
        return input[lhs].sample_id < input[rhs].sample_id;
    });
    std::unordered_set<std::uint32_t> seen;
    for (const std::size_t index : order) {
        if (!seen.insert(input[index].sample_id).second) {
            throw std::invalid_argument("duplicate CPU sample ID");
        }
        result.ids.push_back(input[index].sample_id);
        const auto binary32 = [](const Vec3d& value) {
            return CpuVec3{static_cast<long double>(static_cast<float>(value.x)),
                static_cast<long double>(static_cast<float>(value.y)),
                static_cast<long double>(static_cast<float>(value.z))};
        };
        result.reference.push_back(binary32(input[index].reference));
        result.current.push_back(binary32(input[index].current));
        result.velocity.push_back(binary32(input[index].velocity));
    }
    std::vector<NonlocalGpuGhost> ghosts = input_ghosts;
    std::sort(ghosts.begin(), ghosts.end(), [](const auto& lhs, const auto& rhs) {
        return lhs.sample_id < rhs.sample_id;
    });
    for (const auto& ghost : ghosts) {
        if (!seen.insert(ghost.sample_id).second) {
            throw std::invalid_argument("duplicate CPU ghost ID");
        }
        result.ghost_ids.push_back(ghost.sample_id);
        result.ghosts.push_back({
            static_cast<long double>(static_cast<float>(ghost.position.x)),
            static_cast<long double>(static_cast<float>(ghost.position.y)),
            static_cast<long double>(static_cast<float>(ghost.position.z))});
    }
    return result;
}

CpuVec3 graph_position(const CpuFixture& fixture, std::uint32_t index) {
    if (index < fixture.current.size()) return fixture.current[index];
    return fixture.ghosts[index - fixture.current.size()];
}

CpuEvaluation evaluate_cpu(const CpuFixture& fixture,
    NonlocalGpuVariant variant) {
    CpuEvaluation result;
    const std::size_t count = fixture.current.size();
    constexpr std::size_t kDirectTinyLimit = 4U;
    const bool direct = count <= kDirectTinyLimit;
    result.current_graph = direct
        ? build_cpu_direct_graph(fixture.profile, fixture.ids, fixture.current,
              fixture.ghost_ids, fixture.ghosts,
              variant == NonlocalGpuVariant::StrictRadius)
        : build_cpu_graph(fixture.profile, fixture.ids, fixture.current,
              fixture.ghost_ids, fixture.ghosts,
              variant == NonlocalGpuVariant::StrictRadius);
    result.reference_graph = direct
        ? build_cpu_direct_graph(fixture.profile, fixture.ids, fixture.reference,
              fixture.ghost_ids, fixture.ghosts, false)
        : build_cpu_graph(fixture.profile, fixture.ids, fixture.reference,
              fixture.ghost_ids, fixture.ghosts, false);
    if (result.current_graph.failure != NonlocalGpuFailure::None
        || result.reference_graph.failure != NonlocalGpuFailure::None) {
        result.failure = result.current_graph.failure != NonlocalGpuFailure::None
            ? result.current_graph.failure : result.reference_graph.failure;
        return result;
    }
    const long double mass = fixture.profile.mass;
    const long double dt = fixture.profile.dt;
    const long double rho0 = fixture.profile.rest_density;
    const long double kappa = fixture.profile.kappa;
    const long double lambda = fixture.profile.lambda;
    const long double mu = fixture.profile.mu;
    const long double gamma = fixture.profile.gamma;
    const long double inertia = mass / (dt * dt);
    const bool missing_chain = variant == NonlocalGpuVariant::MissingKernelChain;
    const bool half_viscosity = variant == NonlocalGpuVariant::HalfViscosity;
    const bool wrong_surface = variant == NonlocalGpuVariant::WrongSurfaceSign;
    const bool owner_only = variant == NonlocalGpuVariant::OwnerOnlyPressure;
    const bool swap_graph = variant == NonlocalGpuVariant::CurrentReferenceSwap;
    result.density.assign(count, 0.0L);
    result.excess.assign(count, 0.0L);
    result.gradient.assign(count, {});
    for (std::size_t row = 0U; row < count; ++row) {
        const CpuVec3 owner = fixture.current[row];
        for (std::uint32_t slot = result.current_graph.offsets[row];
             slot < result.current_graph.offsets[row + 1U]; ++slot) {
            const CpuVec3 candidate = graph_position(
                fixture, result.current_graph.neighbors[slot]);
            result.density[row] += mass * kernel(
                length(subtract(owner, candidate)), fixture.profile, false).value;
        }
        result.excess[row] = std::max(result.density[row] / rho0 - 1.0L, 0.0L);
        if (result.excess[row] > 0.0L) ++result.active;
    }
    const CpuVec3 gravity = make_vec(fixture.profile.gravity);
    for (std::size_t row = 0U; row < count; ++row) {
        const CpuVec3 x = fixture.reference[row];
        const CpuVec3 y = fixture.current[row];
        const CpuVec3 predicted = add(x, add(scale(fixture.velocity[row], dt),
            scale(gravity, dt * dt)));
        const CpuVec3 inertial_delta = subtract(y, predicted);
        result.gradient[row] = scale(inertial_delta, inertia);
        result.energy += 0.5L * inertia * dot(inertial_delta, inertial_delta)
            + 0.5L * kappa * result.excess[row] * result.excess[row];
        for (std::uint32_t slot = result.current_graph.offsets[row];
             slot < result.current_graph.offsets[row + 1U]; ++slot) {
            const std::uint32_t neighbor = result.current_graph.neighbors[slot];
            if (neighbor == row) continue;
            const CpuVec3 difference = subtract(y,
                graph_position(fixture, neighbor));
            const long double radius = length(difference);
            if (!(radius > 0.0L)) continue;
            const CpuVec3 normal = scale(difference, 1.0L / radius);
            const KernelValues values = kernel(
                radius, fixture.profile, missing_chain);
            const long double neighbor_excess = neighbor < count && !owner_only
                ? result.excess[neighbor] : 0.0L;
            result.gradient[row] = add(result.gradient[row], scale(normal,
                kappa * mass / rho0
                    * (result.excess[row] + neighbor_excess) * values.first));
            if (neighbor < count) {
                long double potential = 0.0L;
                long double force = 0.0L;
                long double derivative = 0.0L;
                surface(radius, fixture.profile.spacing,
                    potential, force, derivative);
                const long double sign = wrong_surface ? -1.0L : 1.0L;
                result.gradient[row] = add(result.gradient[row], scale(normal,
                    sign * 2.0L * gamma * mass * mass * force));
                if (neighbor > row) {
                    result.energy += 2.0L * gamma * mass * mass * potential;
                }
            }
        }
        const CpuGraph& viscosity_graph = swap_graph
            ? result.current_graph : result.reference_graph;
        for (std::uint32_t slot = viscosity_graph.offsets[row];
             slot < viscosity_graph.offsets[row + 1U]; ++slot) {
            const std::uint32_t neighbor = viscosity_graph.neighbors[slot];
            if (neighbor == row || neighbor >= count) continue;
            const CpuVec3 reference_delta = subtract(x,
                fixture.reference[neighbor]);
            const long double radius = length(reference_delta);
            if (!(radius > 0.0L)) continue;
            const CpuVec3 normal = scale(reference_delta, 1.0L / radius);
            const CpuVec3 delta = subtract(
                subtract(y, fixture.current[neighbor]), reference_delta);
            const long double normal_delta = dot(normal, delta);
            const CpuVec3 tangent_delta = subtract(delta,
                scale(normal, normal_delta));
            const long double factor = mass
                * (-kernel(radius, fixture.profile, missing_chain).first)
                / (rho0 * dt) * (half_viscosity ? 0.5L : 1.0L);
            result.gradient[row] = add(result.gradient[row], scale(add(
                scale(tangent_delta, 2.0L * mu),
                scale(normal, lambda * normal_delta)), factor));
            if (neighbor > row) {
                result.energy += factor * (mu * dot(tangent_delta, tangent_delta)
                    + 0.5L * lambda * normal_delta * normal_delta);
            }
        }
        if (!finite(result.gradient[row]) || !std::isfinite(result.energy)) {
            result.failure = NonlocalGpuFailure::Nonfinite;
            return result;
        }
    }
    return result;
}

std::vector<CpuVec3> apply_cpu_hvp(const CpuFixture& fixture,
    const CpuEvaluation& evaluation,
    const std::vector<CpuVec3>& direction,
    NonlocalGpuVariant variant,
    NonlocalGpuFailure& failure) {
    const std::size_t count = fixture.current.size();
    std::vector<CpuVec3> result(count);
    std::vector<long double> pressure_q(count, 0.0L);
    const long double mass = fixture.profile.mass;
    const long double dt = fixture.profile.dt;
    const long double rho0 = fixture.profile.rest_density;
    const long double kappa = fixture.profile.kappa;
    const long double lambda = fixture.profile.lambda;
    const long double mu = fixture.profile.mu;
    const long double gamma = fixture.profile.gamma;
    const long double inertia = mass / (dt * dt);
    const bool missing_chain = variant == NonlocalGpuVariant::MissingKernelChain;
    const bool half_viscosity = variant == NonlocalGpuVariant::HalfViscosity;
    const bool wrong_surface = variant == NonlocalGpuVariant::WrongSurfaceSign;
    const bool owner_only = variant == NonlocalGpuVariant::OwnerOnlyPressure;
    const bool swap_graph = variant == NonlocalGpuVariant::CurrentReferenceSwap;
    for (std::size_t row = 0U; row < count; ++row) {
        if (!(evaluation.excess[row] > 0.0L)) continue;
        const CpuVec3 owner = fixture.current[row];
        for (std::uint32_t slot = evaluation.current_graph.offsets[row];
             slot < evaluation.current_graph.offsets[row + 1U]; ++slot) {
            const std::uint32_t neighbor = evaluation.current_graph.neighbors[slot];
            if (neighbor == row) continue;
            const CpuVec3 difference = subtract(owner,
                graph_position(fixture, neighbor));
            const long double radius = length(difference);
            if (!(radius > 0.0L)) continue;
            const CpuVec3 normal = scale(difference, 1.0L / radius);
            const CpuVec3 neighbor_direction = neighbor < count
                ? direction[neighbor] : CpuVec3{};
            pressure_q[row] += mass / rho0
                * kernel(radius, fixture.profile, missing_chain).first
                * dot(normal, subtract(direction[row], neighbor_direction));
        }
    }
    for (std::size_t row = 0U; row < count; ++row) {
        const CpuVec3 y = fixture.current[row];
        const CpuVec3 x = fixture.reference[row];
        result[row] = scale(direction[row], inertia);
        for (std::uint32_t slot = evaluation.current_graph.offsets[row];
             slot < evaluation.current_graph.offsets[row + 1U]; ++slot) {
            const std::uint32_t neighbor = evaluation.current_graph.neighbors[slot];
            if (neighbor == row) continue;
            const CpuVec3 difference = subtract(y,
                graph_position(fixture, neighbor));
            const long double radius = length(difference);
            if (!(radius > 0.0L)) continue;
            const CpuVec3 normal = scale(difference, 1.0L / radius);
            const CpuVec3 neighbor_direction = neighbor < count
                ? direction[neighbor] : CpuVec3{};
            const CpuVec3 dv = subtract(direction[row], neighbor_direction);
            const KernelValues values = kernel(
                radius, fixture.profile, missing_chain);
            const CpuVec3 b = scale(normal, mass / rho0 * values.first);
            const long double neighbor_q = neighbor < count && !owner_only
                ? pressure_q[neighbor] : 0.0L;
            const long double neighbor_excess = neighbor < count && !owner_only
                ? evaluation.excess[neighbor] : 0.0L;
            result[row] = add(result[row], scale(b,
                kappa * (pressure_q[row] + neighbor_q)));
            result[row] = add(result[row], scale(radial_apply(normal,
                values.second, values.first / radius, dv),
                kappa * mass / rho0
                    * (evaluation.excess[row] + neighbor_excess)));
            if (neighbor < count) {
                long double potential = 0.0L;
                long double force = 0.0L;
                long double derivative = 0.0L;
                surface(radius, fixture.profile.spacing,
                    potential, force, derivative);
                result[row] = add(result[row], scale(radial_apply(normal,
                    derivative, force / radius, dv),
                    (wrong_surface ? -1.0L : 1.0L)
                        * 2.0L * gamma * mass * mass));
            }
        }
        const CpuGraph& viscosity_graph = swap_graph
            ? evaluation.current_graph : evaluation.reference_graph;
        for (std::uint32_t slot = viscosity_graph.offsets[row];
             slot < viscosity_graph.offsets[row + 1U]; ++slot) {
            const std::uint32_t neighbor = viscosity_graph.neighbors[slot];
            if (neighbor == row || neighbor >= count) continue;
            const CpuVec3 reference_delta = subtract(x,
                fixture.reference[neighbor]);
            const long double radius = length(reference_delta);
            if (!(radius > 0.0L)) continue;
            const CpuVec3 normal = scale(reference_delta, 1.0L / radius);
            const long double factor = mass
                * (-kernel(radius, fixture.profile, missing_chain).first)
                / (rho0 * dt) * (half_viscosity ? 0.5L : 1.0L);
            result[row] = add(result[row], radial_apply(normal,
                factor * lambda, factor * 2.0L * mu,
                subtract(direction[row], direction[neighbor])));
        }
        if (variant == NonlocalGpuVariant::HvpSignFlip) {
            result[row] = scale(result[row], -1.0L);
        }
        if (!finite(result[row])) failure = NonlocalGpuFailure::Nonfinite;
    }
    return result;
}

long double vector_dot(const std::vector<CpuVec3>& lhs,
    const std::vector<CpuVec3>& rhs) {
    if (lhs.size() != rhs.size()) throw std::logic_error("CPU dot size");
    long double result = 0.0L;
    for (std::size_t index = 0U; index < lhs.size(); ++index) {
        result += dot(lhs[index], rhs[index]);
    }
    return result;
}

long double vector_norm(const std::vector<CpuVec3>& values) {
    return std::sqrt(std::max(vector_dot(values, values), 0.0L));
}

long double maximum_norm(const std::vector<CpuVec3>& values) {
    long double result = 0.0L;
    for (const CpuVec3 value : values) result = std::max(result, length(value));
    return result;
}

std::vector<CpuVec3> projected_gradient(const CpuFixture& fixture,
    const std::vector<CpuVec3>& input,
    std::vector<CpuVec3>& mask,
    bool disable_boundary,
    std::uint64_t& projected) {
    std::vector<CpuVec3> result = input;
    mask.assign(result.size(), {1.0L, 1.0L, 1.0L});
    const long double low = 0.5L * fixture.profile.spacing;
    const CpuVec3 high{fixture.profile.basin_extent.x - low,
        fixture.profile.basin_extent.y - low,
        fixture.profile.basin_extent.z - low};
    if (disable_boundary) return result;
    for (std::size_t index = 0U; index < result.size(); ++index) {
        const CpuVec3 p = fixture.current[index];
        if ((p.x == low && result[index].x > 0.0L)
            || (p.x == high.x && result[index].x < 0.0L)) {
            result[index].x = 0.0L;
            mask[index].x = 0.0L;
            ++projected;
        }
        if ((p.y == low && result[index].y > 0.0L)
            || (p.y == high.y && result[index].y < 0.0L)) {
            result[index].y = 0.0L;
            mask[index].y = 0.0L;
            ++projected;
        }
        if ((p.z == low && result[index].z > 0.0L)
            || (p.z == high.z && result[index].z < 0.0L)) {
            result[index].z = 0.0L;
            mask[index].z = 0.0L;
            ++projected;
        }
    }
    return result;
}

void apply_mask(std::vector<CpuVec3>& values,
    const std::vector<CpuVec3>& mask) {
    for (std::size_t index = 0U; index < values.size(); ++index) {
        values[index].x *= mask[index].x;
        values[index].y *= mask[index].y;
        values[index].z *= mask[index].z;
    }
}

long double boundary_intersection(const std::vector<CpuVec3>& point,
    const std::vector<CpuVec3>& direction,
    long double radius) {
    const long double a = vector_dot(direction, direction);
    const long double pd = vector_dot(point, direction);
    const long double p2 = vector_dot(point, point);
    if (!(a > 0.0L)) return std::numeric_limits<long double>::quiet_NaN();
    const long double discriminant = std::max(
        pd * pd + a * (radius * radius - p2), 0.0L);
    return (-pd + std::sqrt(discriminant)) / a;
}

long double penetration(const CpuFixture& fixture) {
    const long double low = 0.5L * fixture.profile.spacing;
    const CpuVec3 high{fixture.profile.basin_extent.x - low,
        fixture.profile.basin_extent.y - low,
        fixture.profile.basin_extent.z - low};
    long double result = 0.0L;
    for (const CpuVec3 value : fixture.current) {
        result = std::max(result, std::max({low - value.x, value.x - high.x,
            low - value.y, value.y - high.y, low - value.z,
            value.z - high.z, 0.0L}));
    }
    return result;
}

} // namespace

NonlocalGpuStepResult step_reference(
    const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& samples,
    const std::vector<NonlocalGpuGhost>& ghosts,
    std::uint32_t total_hvp_budget,
    NonlocalGpuVariant variant,
    bool capture_state) {
    NonlocalGpuStepResult result;
    result.hvp_budget = total_hvp_budget;
    result.solver_profile = NonlocalGpuSolverProfile::Unpreconditioned;
    result.variant = variant;
    const NonlocalGpuFailure admission = validate_nonlocal_input(
        profile, samples, ghosts);
    if (admission != NonlocalGpuFailure::None) {
        result.failure = admission;
        return result;
    }
    if (total_hvp_budget != 32U
        && total_hvp_budget != 64U && total_hvp_budget != 128U) {
        result.failure = NonlocalGpuFailure::InvalidState;
        return result;
    }
    CpuFixture fixture;
    try {
        fixture = canonical_fixture(profile, samples, ghosts);
    } catch (const std::exception&) {
        result.failure = NonlocalGpuFailure::InvalidState;
        return result;
    }
    result.maximum_penetration_m = static_cast<double>(penetration(fixture));
    if (result.maximum_penetration_m != 0.0) {
        result.failure = NonlocalGpuFailure::InvalidState;
        return result;
    }
    const bool disable_boundary = variant == NonlocalGpuVariant::DisableBoundary;
    long double radius = profile.spacing;
    const long double minimum_radius = std::ldexp(
        static_cast<long double>(profile.spacing), -40);
    const long double maximum_radius = 4.0L * profile.spacing;
    bool accepted_once = false;
    bool succeeded = false;
    for (std::uint32_t outer = 0U; outer < 64U; ++outer) {
        const CpuEvaluation evaluation = evaluate_cpu(fixture, variant);
        if (evaluation.failure != NonlocalGpuFailure::None) {
            result.failure = evaluation.failure;
            break;
        }
        if (outer == 0U) result.initial_energy = static_cast<double>(evaluation.energy);
        result.final_energy = static_cast<double>(evaluation.energy);
        result.active_pressure_centers = evaluation.active;
        std::vector<CpuVec3> mask;
        std::uint64_t projected = 0U;
        std::vector<CpuVec3> gradient = projected_gradient(fixture,
            evaluation.gradient, mask, disable_boundary, projected);
        result.work.projected_gradient_components += projected;
        result.gradient_norm = static_cast<double>(vector_norm(gradient));
        result.scaled_displacement_residual = static_cast<double>(
            static_cast<long double>(profile.dt) * profile.dt / profile.mass
            * maximum_norm(gradient) / profile.spacing);
        if (!std::isfinite(result.gradient_norm)
            || !std::isfinite(result.scaled_displacement_residual)) {
            result.failure = NonlocalGpuFailure::Nonfinite;
            break;
        }
        if (result.scaled_displacement_residual <= 1.0e-5) {
            if (outer == 0U || accepted_once) succeeded = true;
            else result.failure = NonlocalGpuFailure::PhysicsGateFailed;
            break;
        }
        std::vector<CpuVec3> residual = gradient;
        std::vector<CpuVec3> direction(gradient.size());
        std::vector<CpuVec3> step(gradient.size());
        for (std::size_t index = 0U; index < gradient.size(); ++index) {
            direction[index] = scale(residual[index], -1.0L);
        }
        long double residual_squared = vector_dot(residual, residual);
        const long double initial_residual = std::sqrt(
            std::max(residual_squared, 0.0L));
        const long double forcing = std::min(0.5L, std::sqrt(initial_residual));
        bool inner_complete = false;
        bool at_radius = false;
        for (std::uint64_t inner = 0U;
             inner < 9ULL * fixture.current.size(); ++inner) {
            if (result.hvp_used + 1U >= total_hvp_budget) {
                result.failure = NonlocalGpuFailure::WorkBudgetExceeded;
                break;
            }
            NonlocalGpuFailure hvp_failure = NonlocalGpuFailure::None;
            std::vector<CpuVec3> product = apply_cpu_hvp(fixture,
                evaluation, direction, variant, hvp_failure);
            ++result.hvp_used;
            ++result.work.hvp_applications;
            apply_mask(product, mask);
            if (hvp_failure != NonlocalGpuFailure::None) {
                result.failure = hvp_failure;
                break;
            }
            const long double curvature = vector_dot(direction, product);
            if (!std::isfinite(curvature)) {
                result.failure = NonlocalGpuFailure::Nonfinite;
                break;
            }
            if (curvature <= 0.0L) {
                const long double tau = boundary_intersection(
                    step, direction, radius);
                ++result.work.boundary_intersections;
                if (!std::isfinite(tau)) {
                    result.failure = NonlocalGpuFailure::Nonfinite;
                    break;
                }
                for (std::size_t index = 0U; index < step.size(); ++index) {
                    step[index] = add(step[index], scale(direction[index], tau));
                }
                inner_complete = true;
                at_radius = true;
                break;
            }
            const long double alpha = residual_squared / curvature;
            std::vector<CpuVec3> candidate = step;
            for (std::size_t index = 0U; index < candidate.size(); ++index) {
                candidate[index] = add(candidate[index],
                    scale(direction[index], alpha));
            }
            if (vector_norm(candidate) >= radius) {
                const long double tau = boundary_intersection(
                    step, direction, radius);
                ++result.work.boundary_intersections;
                if (!std::isfinite(tau)) {
                    result.failure = NonlocalGpuFailure::Nonfinite;
                    break;
                }
                for (std::size_t index = 0U; index < step.size(); ++index) {
                    step[index] = add(step[index], scale(direction[index], tau));
                }
                inner_complete = true;
                at_radius = true;
                break;
            }
            step = std::move(candidate);
            std::vector<CpuVec3> next_residual = residual;
            for (std::size_t index = 0U; index < residual.size(); ++index) {
                next_residual[index] = add(next_residual[index],
                    scale(product[index], alpha));
            }
            const long double next_squared = vector_dot(
                next_residual, next_residual);
            if (std::sqrt(std::max(next_squared, 0.0L))
                <= forcing * initial_residual) {
                residual = std::move(next_residual);
                inner_complete = true;
                break;
            }
            const long double beta = next_squared / residual_squared;
            for (std::size_t index = 0U; index < direction.size(); ++index) {
                direction[index] = add(scale(next_residual[index], -1.0L),
                    scale(direction[index], beta));
            }
            residual = std::move(next_residual);
            residual_squared = next_squared;
        }
        if (result.failure != NonlocalGpuFailure::None) break;
        if (!inner_complete) {
            result.failure = NonlocalGpuFailure::WorkBudgetExceeded;
            break;
        }
        const long double low = 0.5L * profile.spacing;
        const CpuVec3 high{profile.basin_extent.x - low,
            profile.basin_extent.y - low, profile.basin_extent.z - low};
        std::vector<CpuVec3> trial = fixture.current;
        std::vector<CpuVec3> actual(step.size());
        CpuVec3 trial_boundary_impulse{};
        std::uint64_t trial_face_mask_xor = 0U;
        for (std::size_t index = 0U; index < trial.size(); ++index) {
            const CpuVec3 origin = trial[index];
            CpuVec3 value = add(origin, step[index]);
            std::uint32_t face_mask = 0U;
            if (!disable_boundary) {
                result.work.boundary_face_tests += 6U;
                long double hit = 1.0L;
                const auto consider = [&](long double base, long double delta,
                                          long double plane,
                                          std::uint32_t bit) {
                    if (delta == 0.0L) return;
                    const long double candidate = (plane - base) / delta;
                    if (!(candidate >= 0.0L && candidate <= 1.0L)) return;
                    if (candidate < hit) {
                        hit = candidate;
                        face_mask = bit;
                    } else if (candidate == hit) {
                        face_mask |= bit;
                    }
                };
                if (value.x < low) consider(origin.x, step[index].x, low, 1U);
                if (value.x > high.x) consider(origin.x, step[index].x, high.x, 2U);
                if (value.y < low) consider(origin.y, step[index].y, low, 4U);
                if (value.y > high.y) consider(origin.y, step[index].y, high.y, 8U);
                if (value.z < low) consider(origin.z, step[index].z, low, 16U);
                if (value.z > high.z) consider(origin.z, step[index].z, high.z, 32U);
                if (face_mask != 0U) {
                    value = add(origin, scale(step[index], hit));
                    if ((face_mask & 1U) != 0U) value.x = low;
                    if ((face_mask & 2U) != 0U) value.x = high.x;
                    if ((face_mask & 4U) != 0U) value.y = low;
                    if ((face_mask & 8U) != 0U) value.y = high.y;
                    if ((face_mask & 16U) != 0U) value.z = low;
                    if ((face_mask & 32U) != 0U) value.z = high.z;
                    for (std::uint32_t bits = face_mask; bits != 0U;
                         bits >>= 1U) {
                        result.work.boundary_face_hits += bits & 1U;
                    }
                    result.work.contact_projections += 1U;
                    trial_face_mask_xor ^= (static_cast<std::uint64_t>(
                        fixture.ids[index]) << 8U) ^ face_mask;
                    const CpuVec3 impulse = scale(subtract(
                        subtract(value, origin), step[index]),
                        profile.mass / profile.dt);
                    trial_boundary_impulse = add(
                        trial_boundary_impulse, impulse);
                }
            }
            actual[index] = subtract(value, origin);
            trial[index] = value;
        }
        result.work.boundary_face_mask_xor ^= trial_face_mask_xor;
        if (result.hvp_used >= total_hvp_budget) {
            result.failure = NonlocalGpuFailure::WorkBudgetExceeded;
            break;
        }
        NonlocalGpuFailure prediction_failure = NonlocalGpuFailure::None;
        std::vector<CpuVec3> hessian_step = apply_cpu_hvp(fixture,
            evaluation, actual, variant, prediction_failure);
        ++result.hvp_used;
        ++result.work.hvp_applications;
        apply_mask(hessian_step, mask);
        if (prediction_failure != NonlocalGpuFailure::None) {
            result.failure = prediction_failure;
            break;
        }
        const long double predicted_reduction = -(
            vector_dot(gradient, actual)
            + 0.5L * vector_dot(actual, hessian_step));
        ++result.outer_trials;
        ++result.work.outer_trials;
        bool valid = std::isfinite(predicted_reduction)
            && predicted_reduction > 0.0L;
        long double ratio = -std::numeric_limits<long double>::infinity();
        CpuEvaluation trial_evaluation;
        if (valid) {
            CpuFixture trial_fixture = fixture;
            trial_fixture.current = trial;
            trial_evaluation = evaluate_cpu(trial_fixture, variant);
            if (trial_evaluation.failure != NonlocalGpuFailure::None) {
                result.failure = trial_evaluation.failure;
                break;
            }
            const long double actual_reduction = evaluation.energy
                - trial_evaluation.energy;
            valid = std::isfinite(actual_reduction) && actual_reduction > 0.0L;
            if (valid) ratio = actual_reduction / predicted_reduction;
        }
        const bool accepted = valid && std::isfinite(ratio) && ratio >= 0.1L;
        if (accepted) {
            fixture.current = std::move(trial);
            ++result.work.accepted_trials;
            accepted_once = true;
            const Vec3d impulse = narrow(trial_boundary_impulse);
            result.boundary_impulse.x += impulse.x;
            result.boundary_impulse.y += impulse.y;
            result.boundary_impulse.z += impulse.z;
            result.boundary_face_mask_xor ^= trial_face_mask_xor;
            result.final_energy = static_cast<double>(trial_evaluation.energy);
            result.active_pressure_centers = trial_evaluation.active;
        } else {
            ++result.work.rejected_trials;
        }
        if (!valid || ratio < 0.25L) {
            radius *= 0.25L;
            ++result.work.radius_shrinks;
        } else if (ratio > 0.75L && at_radius) {
            const long double expanded = std::min(2.0L * radius, maximum_radius);
            if (expanded != radius) ++result.work.radius_expands;
            radius = expanded;
        }
        if (!(radius >= minimum_radius)) {
            result.failure = NonlocalGpuFailure::PhysicsGateFailed;
            break;
        }
    }
    if (!succeeded && result.failure == NonlocalGpuFailure::None) {
        result.failure = result.hvp_used >= total_hvp_budget
            ? NonlocalGpuFailure::WorkBudgetExceeded
            : NonlocalGpuFailure::PhysicsGateFailed;
    }
    if (succeeded) {
        result.maximum_penetration_m = static_cast<double>(penetration(fixture));
        if (result.maximum_penetration_m > 0.0025) {
            result.failure = NonlocalGpuFailure::PhysicsGateFailed;
            return result;
        }
        const CpuEvaluation terminal = evaluate_cpu(fixture, variant);
        if (terminal.failure != NonlocalGpuFailure::None) {
            result.failure = terminal.failure;
            return result;
        }
        const long double inverse_dt = 1.0L / profile.dt;
        if (capture_state) result.state.reserve(fixture.current.size());
        if (capture_state) result.density.reserve(fixture.current.size());
        for (std::size_t index = 0U; index < fixture.current.size(); ++index) {
            const CpuVec3 velocity = scale(subtract(
                fixture.current[index], fixture.reference[index]), inverse_dt);
            if (capture_state) {
                const Vec3d position = narrow(fixture.current[index]);
                result.state.push_back({fixture.ids[index], position, position,
                    narrow(velocity)});
                result.density.push_back(static_cast<double>(terminal.density[index]));
            }
        }
        result.work.state_updates += fixture.current.size();
        result.failure = NonlocalGpuFailure::None;
    }
    return result;
}

} // namespace nextengine::nonlocal::gpu_full_step
