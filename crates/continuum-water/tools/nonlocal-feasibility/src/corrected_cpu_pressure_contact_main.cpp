#include "corrected_cuda_full_step.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <limits>
#include <map>
#include <numeric>
#include <queue>
#include <set>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <tuple>
#include <utility>
#include <vector>

namespace {

#ifndef NCGP13_CONTRACT_ROOT
#define NCGP13_CONTRACT_ROOT "unconfigured"
#endif
#ifndef NCGP13_SOURCE_ROOT
#define NCGP13_SOURCE_ROOT "unconfigured"
#endif
#ifndef NCGP13_SOURCE_COMMIT
#define NCGP13_SOURCE_COMMIT "unconfigured"
#endif
#ifndef NCGP13_SOURCE_TREE
#define NCGP13_SOURCE_TREE "unconfigured"
#endif
#ifndef NCGP13_COMPILER_FLAGS
#define NCGP13_COMPILER_FLAGS "unconfigured"
#endif

using namespace nextengine::nonlocal::gpu_full_step;

constexpr std::size_t kMaximumSweeps = 4096U;
constexpr std::size_t kMaximumRounds = 8U;
constexpr std::size_t kPhaseBSteps = 240U;
constexpr long double kJacobianLimit = 2.0e-7L;
constexpr long double kSymmetryLimit = 2.0e-12L;
constexpr long double kPrimalLimit = 1.0e-8L;
constexpr long double kKktLimit = 1.0e-8L;
constexpr long double kComplementarityLimit = 1.0e-10L;
constexpr long double kDensityMaximumLimit = 1.0e-3L;
constexpr long double kDensityRmsLimit = 2.5e-4L;
constexpr long double kBalanceLimit = 1.0e-8L;
constexpr long double kPositionRmsLimit = 2.5e-3L;
constexpr long double kPositionMaximumLimit = 5.0e-3L;
constexpr long double kVelocityRmsLimit = 5.0e-2L;
constexpr long double kVelocityMaximumLimit = 1.0e-1L;
constexpr long double kEnergyLimit = 1.0e-2L;
constexpr long double kMomentumLimit = 1.0e-2L;
constexpr std::string_view kPhaseAInput =
    "82cf83cbfc5839026c8dff77c55b81be2e5597982c31493a7016a9f5fca2a8fa";
constexpr std::string_view kPhaseBInput =
    "f9dbf235d5e176efe29468eba26551a1958106a5eb9d6cffbcb74ee57b4b4d1e";
constexpr std::string_view kLegacyMultiplier =
    "e3a62beac5ce4ebb2d39e9baf8ce8a28d95f43ab9291c11f415ba20cb6975b6a";
constexpr std::string_view kLegacyTrial =
    "00f5d0ff3bc77716e4a3da4a209bef8ab1382de6c317710058495e9ecd27b0c5";

struct Vec3l {
    long double x = 0.0L;
    long double y = 0.0L;
    long double z = 0.0L;
};

Vec3l operator+(Vec3l lhs, Vec3l rhs) {
    return {lhs.x + rhs.x, lhs.y + rhs.y, lhs.z + rhs.z};
}
Vec3l operator-(Vec3l lhs, Vec3l rhs) {
    return {lhs.x - rhs.x, lhs.y - rhs.y, lhs.z - rhs.z};
}
Vec3l operator*(Vec3l value, long double scale) {
    return {value.x * scale, value.y * scale, value.z * scale};
}
Vec3l& operator+=(Vec3l& lhs, Vec3l rhs) {
    lhs = lhs + rhs;
    return lhs;
}
long double dot(Vec3l lhs, Vec3l rhs) {
    return lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z;
}
long double norm(Vec3l value) {
    return std::sqrt(std::max(dot(value, value), 0.0L));
}
bool finite(Vec3l value) {
    return std::isfinite(value.x) && std::isfinite(value.y)
        && std::isfinite(value.z);
}
Vec3l widen(Vec3d value) {
    return {static_cast<long double>(value.x),
        static_cast<long double>(value.y),
        static_cast<long double>(value.z)};
}

void append_u32(std::string& bytes, std::uint32_t value) {
    for (std::uint32_t index = 0U; index < 4U; ++index) {
        bytes.push_back(static_cast<char>((value >> (8U * index)) & 0xffU));
    }
}
void append_u64(std::string& bytes, std::uint64_t value) {
    for (std::uint32_t index = 0U; index < 8U; ++index) {
        bytes.push_back(static_cast<char>((value >> (8U * index)) & 0xffU));
    }
}
void append_f32(std::string& bytes, float value) {
    std::uint32_t bits = 0U;
    static_assert(sizeof(bits) == sizeof(value));
    std::memcpy(&bits, &value, sizeof(bits));
    append_u32(bytes, bits);
}
void append_f64(std::string& bytes, double value) {
    if (!std::isfinite(value)) throw std::runtime_error("nonfinite root value");
    std::uint64_t bits = 0U;
    static_assert(sizeof(bits) == sizeof(value));
    std::memcpy(&bits, &value, sizeof(bits));
    append_u64(bytes, bits);
}
void append_wide(std::string& bytes, long double value) {
    if (!std::isfinite(value)) throw std::runtime_error("nonfinite wide root value");
    append_f64(bytes, static_cast<double>(value));
}
void append_string(std::string& bytes, std::string_view value) {
    append_u64(bytes, value.size());
    bytes.append(value.data(), value.size());
}
std::string digest(std::string bytes) {
    return nextengine::nonlocal::sha256_hex(bytes);
}
std::string file_root(const std::string& path) {
    std::ifstream stream(path, std::ios::binary);
    if (!stream) return {};
    std::ostringstream bytes;
    bytes << stream.rdbuf();
    if (!stream.good() && !stream.eof()) return {};
    return digest(bytes.str());
}

std::string local_profile_root(const NonlocalGpuProfile& profile) {
    std::string bytes = "nextengine.nonlocal.ncgp3.profile.v1\0";
    append_string(bytes, profile.id);
    for (const double value : std::array<double, 19>{profile.dt,
             profile.spacing, profile.horizon, profile.mass,
             profile.rest_density, profile.kernel_scale, profile.kappa,
             profile.lambda, profile.mu, profile.gamma, profile.gravity.x,
             profile.gravity.y, profile.gravity.z, profile.basin_extent.x,
             profile.basin_extent.y, profile.basin_extent.z,
             static_cast<double>(profile.ghost_layers),
             static_cast<double>(profile.maximum_dynamic_samples),
             static_cast<double>(profile.maximum_neighbors)}) {
        append_f64(bytes, value);
    }
    return digest(std::move(bytes));
}

std::string local_input_root(const NonlocalGpuProfile& profile,
    std::vector<NonlocalGpuSample> samples,
    std::vector<NonlocalGpuGhost> ghosts) {
    std::sort(samples.begin(), samples.end(), [](const auto& lhs, const auto& rhs) {
        return lhs.sample_id < rhs.sample_id;
    });
    std::sort(ghosts.begin(), ghosts.end(), [](const auto& lhs, const auto& rhs) {
        return lhs.sample_id < rhs.sample_id;
    });
    std::string bytes = "nextengine.nonlocal.ncgp1.input.v1\0";
    append_string(bytes, local_profile_root(profile));
    append_u64(bytes, samples.size());
    for (const auto& sample : samples) {
        append_u32(bytes, sample.sample_id);
        for (const double value : std::array<double, 9>{sample.reference.x,
                 sample.reference.y, sample.reference.z, sample.current.x,
                 sample.current.y, sample.current.z, sample.velocity.x,
                 sample.velocity.y, sample.velocity.z}) {
            append_f32(bytes, static_cast<float>(value));
        }
    }
    append_u64(bytes, ghosts.size());
    for (const auto& ghost : ghosts) {
        append_u32(bytes, ghost.sample_id);
        append_f32(bytes, static_cast<float>(ghost.position.x));
        append_f32(bytes, static_cast<float>(ghost.position.y));
        append_f32(bytes, static_cast<float>(ghost.position.z));
    }
    return digest(std::move(bytes));
}

std::string ncgp13_profile_root(const NonlocalGpuProfile& profile) {
    std::string bytes;
    append_string(bytes, "nextengine.nonlocal.ncgp13.profile.v1");
    append_string(bytes, profile.id);
    for (const double value : std::array<double, 19>{profile.dt,
             profile.spacing, profile.horizon, profile.mass,
             profile.rest_density, profile.kernel_scale, profile.kappa,
             profile.lambda, profile.mu, profile.gamma, profile.gravity.x,
             profile.gravity.y, profile.gravity.z, profile.basin_extent.x,
             profile.basin_extent.y, profile.basin_extent.z,
             static_cast<double>(profile.ghost_layers),
             static_cast<double>(profile.maximum_dynamic_samples),
             static_cast<double>(profile.maximum_neighbors)}) {
        append_f64(bytes, value);
    }
    return digest(std::move(bytes));
}

std::string ncgp13_control_input_root(std::string_view profile_root,
    std::vector<NonlocalGpuSample> samples,
    std::vector<NonlocalGpuGhost> ghosts) {
    std::sort(samples.begin(), samples.end(), [](const auto& lhs, const auto& rhs) {
        return lhs.sample_id < rhs.sample_id;
    });
    std::sort(ghosts.begin(), ghosts.end(), [](const auto& lhs, const auto& rhs) {
        return lhs.sample_id < rhs.sample_id;
    });
    std::string bytes;
    append_string(bytes, "nextengine.nonlocal.ncgp13.control-input.v1");
    append_string(bytes, profile_root);
    append_u64(bytes, samples.size());
    for (const NonlocalGpuSample& sample : samples) {
        append_u32(bytes, sample.sample_id);
        for (const double value : std::array<double, 9>{sample.reference.x,
                 sample.reference.y, sample.reference.z, sample.current.x,
                 sample.current.y, sample.current.z, sample.velocity.x,
                 sample.velocity.y, sample.velocity.z}) {
            append_f32(bytes, static_cast<float>(value));
        }
    }
    append_u64(bytes, ghosts.size());
    for (const NonlocalGpuGhost& ghost : ghosts) {
        append_u32(bytes, ghost.sample_id);
        append_f32(bytes, static_cast<float>(ghost.position.x));
        append_f32(bytes, static_cast<float>(ghost.position.y));
        append_f32(bytes, static_cast<float>(ghost.position.z));
    }
    return digest(std::move(bytes));
}

enum class Admission { Ok, InvalidProfile, Capacity, DuplicateId, Nonfinite,
    NonBinary32 };

bool binary32_exact(double value) {
    return std::isfinite(value)
        && value == static_cast<double>(static_cast<float>(value));
}
bool binary32_exact(Vec3d value) {
    return binary32_exact(value.x) && binary32_exact(value.y)
        && binary32_exact(value.z);
}
bool finite_after_binary32(Vec3d value) {
    return finite(widen(value))
        && std::isfinite(static_cast<float>(value.x))
        && std::isfinite(static_cast<float>(value.y))
        && std::isfinite(static_cast<float>(value.z));
}
Admission admit(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& samples,
    const std::vector<NonlocalGpuGhost>& ghosts) {
    const std::array<double, 10> scalars{profile.dt, profile.spacing,
        profile.horizon, profile.mass, profile.rest_density,
        profile.kernel_scale, profile.kappa, profile.lambda, profile.mu,
        profile.gamma};
    if (profile.id != "nonlocal-water-50k-v1"
        || !std::all_of(scalars.begin(), scalars.end(), [](double value) {
            return std::isfinite(value);
        }) || !finite_after_binary32(profile.gravity)
        || !finite_after_binary32(profile.basin_extent) || profile.dt <= 0.0
        || profile.spacing <= 0.0 || profile.horizon <= 0.0
        || profile.mass <= 0.0 || profile.rest_density <= 0.0
        || profile.kernel_scale <= 0.0 || profile.kappa < 0.0
        || profile.lambda != 0.0 || profile.mu != 0.0 || profile.gamma != 0.0
        || profile.ghost_layers != 3U
        || profile.maximum_dynamic_samples != kMaximumDynamicSamples
        || profile.maximum_neighbors != kMaximumNeighbors) {
        return Admission::InvalidProfile;
    }
    if (samples.empty() || samples.size() > profile.maximum_dynamic_samples
        || samples.size() + ghosts.size() > 100000U) {
        return Admission::Capacity;
    }
    std::set<std::uint32_t> ids;
    for (const auto& sample : samples) {
        if (!ids.insert(sample.sample_id).second) return Admission::DuplicateId;
        if (!finite(widen(sample.reference)) || !finite(widen(sample.current))
            || !finite(widen(sample.velocity))) return Admission::Nonfinite;
        if (!binary32_exact(sample.reference) || !binary32_exact(sample.current)
            || !binary32_exact(sample.velocity)) return Admission::NonBinary32;
    }
    std::uint32_t prior = 0U;
    for (std::size_t index = 0U; index < ghosts.size(); ++index) {
        const auto& ghost = ghosts[index];
        if (!ids.insert(ghost.sample_id).second
            || (index != 0U && ghost.sample_id <= prior)) {
            return Admission::DuplicateId;
        }
        prior = ghost.sample_id;
        if (!finite(widen(ghost.position))) return Admission::Nonfinite;
        if (!binary32_exact(ghost.position)) return Admission::NonBinary32;
    }
    return Admission::Ok;
}

struct State {
    std::vector<std::uint32_t> id;
    std::vector<Vec3l> reference;
    std::vector<Vec3l> position;
    std::vector<Vec3l> velocity;
};

State working_state(std::vector<NonlocalGpuSample> samples) {
    std::sort(samples.begin(), samples.end(), [](const auto& lhs, const auto& rhs) {
        return lhs.sample_id < rhs.sample_id;
    });
    State result;
    result.id.reserve(samples.size());
    result.reference.reserve(samples.size());
    result.position.reserve(samples.size());
    result.velocity.reserve(samples.size());
    for (const auto& sample : samples) {
        result.id.push_back(sample.sample_id);
        result.reference.push_back(widen(sample.reference));
        result.position.push_back(widen(sample.current));
        result.velocity.push_back(widen(sample.velocity));
    }
    return result;
}

std::string state_root(std::string_view domain, const State& state) {
    std::string bytes;
    append_string(bytes, domain);
    append_u64(bytes, state.id.size());
    for (std::size_t index = 0U; index < state.id.size(); ++index) {
        append_u32(bytes, state.id[index]);
        for (const Vec3l value : std::array<Vec3l, 3>{state.reference[index],
                 state.position[index], state.velocity[index]}) {
            append_wide(bytes, value.x);
            append_wide(bytes, value.y);
            append_wide(bytes, value.z);
        }
    }
    return digest(std::move(bytes));
}

std::string id_scalar_root(std::string_view domain,
    const std::vector<std::uint32_t>& ids,
    const std::vector<long double>& values) {
    if (ids.size() != values.size()) throw std::logic_error("root size mismatch");
    std::string bytes;
    append_string(bytes, domain);
    append_u64(bytes, ids.size());
    for (std::size_t index = 0U; index < ids.size(); ++index) {
        append_u32(bytes, ids[index]);
        append_wide(bytes, values[index]);
    }
    return digest(std::move(bytes));
}

std::string id_vec_root(std::string_view domain,
    const std::vector<std::uint32_t>& ids,
    const std::vector<Vec3l>& values) {
    if (ids.size() != values.size()) throw std::logic_error("root size mismatch");
    std::string bytes;
    append_string(bytes, domain);
    append_u64(bytes, ids.size());
    for (std::size_t index = 0U; index < ids.size(); ++index) {
        append_u32(bytes, ids[index]);
        append_wide(bytes, values[index].x);
        append_wide(bytes, values[index].y);
        append_wide(bytes, values[index].z);
    }
    return digest(std::move(bytes));
}

struct Kernel {
    long double value = 0.0L;
    long double first = 0.0L;
};
Kernel kernel(long double radius, const NonlocalGpuProfile& profile) {
    const long double horizon = profile.horizon;
    const long double q = 2.0L * radius / horizon;
    const long double alpha = static_cast<long double>(profile.kernel_scale)
        * 3.0L / (2.0L * std::acos(-1.0L) * horizon * horizon * horizon);
    long double value = 0.0L;
    long double first_q = 0.0L;
    if (q < 1.0L) {
        value = alpha * (2.0L / 3.0L - q * q + 0.5L * q * q * q);
        first_q = alpha * (-2.0L * q + 1.5L * q * q);
    } else if (q <= 2.0L) {
        const long double tail = 2.0L - q;
        value = alpha * tail * tail * tail / 6.0L;
        first_q = -0.5L * alpha * tail * tail;
    }
    return {value, first_q * 2.0L / horizon};
}

using Cell = std::tuple<int, int, int>;
struct GhostGrid {
    long double cell_size = 0.0L;
    std::map<Cell, std::vector<std::size_t>> cells;
};
Cell cell_of(Vec3l value, long double cell_size) {
    return {static_cast<int>(std::floor(value.x / cell_size)),
        static_cast<int>(std::floor(value.y / cell_size)),
        static_cast<int>(std::floor(value.z / cell_size))};
}
GhostGrid make_ghost_grid(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuGhost>& ghosts) {
    GhostGrid grid;
    grid.cell_size = profile.horizon;
    for (std::size_t index = 0U; index < ghosts.size(); ++index) {
        grid.cells[cell_of(widen(ghosts[index].position), grid.cell_size)]
            .push_back(index);
    }
    return grid;
}
std::vector<std::size_t> nearby_ghosts(const GhostGrid& grid, Vec3l owner) {
    const auto [cx, cy, cz] = cell_of(owner, grid.cell_size);
    std::vector<std::size_t> result;
    // Binary32 fixture coordinates can put an exact-radius pair on opposite
    // sides of two floor-cell boundaries.  The wider stencil is still only a
    // candidate filter; the inclusive distance predicate remains authority.
    for (int dz = -2; dz <= 2; ++dz) {
        for (int dy = -2; dy <= 2; ++dy) {
            for (int dx = -2; dx <= 2; ++dx) {
                const auto found = grid.cells.find({cx + dx, cy + dy, cz + dz});
                if (found != grid.cells.end()) {
                    result.insert(result.end(), found->second.begin(), found->second.end());
                }
            }
        }
    }
    std::sort(result.begin(), result.end());
    return result;
}

struct Work {
    std::uint64_t graph_builds = 0U;
    std::uint64_t graph_candidates = 0U;
    std::uint64_t accepted_pairs = 0U;
    std::uint64_t density_pairs = 0U;
    std::uint64_t derivative_pairs = 0U;
    std::uint64_t matrix_products = 0U;
    std::uint64_t qp_sweeps = 0U;
    std::uint64_t qp_updates = 0U;
    std::uint64_t gradient_recomputations = 0U;
    std::uint64_t finite_difference_candidates = 0U;
    std::uint64_t independent_density_candidates = 0U;
    std::uint64_t plane_tests = 0U;
    std::uint64_t plane_hits = 0U;
    std::uint64_t contact_projections = 0U;
    std::uint64_t projection_rounds = 0U;
    std::uint64_t analytic_jv_multiply_adds = 0U;
    std::uint64_t topology_distance_tests = 0U;
    std::uint64_t topology_discoveries = 0U;
    std::uint64_t hash_derivations = 0U;
    std::uint64_t penalty_work = 0U;
};

std::array<std::uint64_t, 20> work_values(const Work& work) {
    return {work.graph_builds, work.graph_candidates, work.accepted_pairs,
        work.density_pairs, work.derivative_pairs, work.matrix_products,
        work.qp_sweeps, work.qp_updates, work.gradient_recomputations,
        work.finite_difference_candidates,
        work.independent_density_candidates, work.plane_tests,
        work.plane_hits, work.contact_projections, work.projection_rounds,
        work.analytic_jv_multiply_adds, work.topology_distance_tests,
        work.topology_discoveries, work.hash_derivations,
        work.penalty_work};
}

bool same_work(const Work& lhs, const Work& rhs) {
    return work_values(lhs) == work_values(rhs);
}

void add_work(Work& target, const Work& value) {
    target.graph_builds += value.graph_builds;
    target.graph_candidates += value.graph_candidates;
    target.accepted_pairs += value.accepted_pairs;
    target.density_pairs += value.density_pairs;
    target.derivative_pairs += value.derivative_pairs;
    target.matrix_products += value.matrix_products;
    target.qp_sweeps += value.qp_sweeps;
    target.qp_updates += value.qp_updates;
    target.gradient_recomputations += value.gradient_recomputations;
    target.finite_difference_candidates += value.finite_difference_candidates;
    target.independent_density_candidates += value.independent_density_candidates;
    target.plane_tests += value.plane_tests;
    target.plane_hits += value.plane_hits;
    target.contact_projections += value.contact_projections;
    target.projection_rounds += value.projection_rounds;
    target.analytic_jv_multiply_adds += value.analytic_jv_multiply_adds;
    target.topology_distance_tests += value.topology_distance_tests;
    target.topology_discoveries += value.topology_discoveries;
    target.hash_derivations += value.hash_derivations;
    target.penalty_work += value.penalty_work;
}
std::string work_root(const Work& work) {
    std::string bytes;
    append_string(bytes, "nextengine.nonlocal.ncgp13.work.v1");
    for (const std::uint64_t value : work_values(work)) {
        append_u64(bytes, value);
    }
    return digest(std::move(bytes));
}

struct Edge {
    std::size_t index = 0U;
    bool ghost = false;
};
struct Graph {
    bool valid = true;
    std::vector<std::vector<Edge>> row;
    std::string root;
};
Graph build_graph(const NonlocalGpuProfile& profile,
    const std::vector<std::uint32_t>& ids,
    const std::vector<Vec3l>& positions,
    const std::vector<NonlocalGpuGhost>& ghosts,
    const GhostGrid& grid, bool strict, Work& work) {
    Graph graph;
    graph.row.resize(positions.size());
    ++work.graph_builds;
    for (std::size_t owner = 0U; owner < positions.size(); ++owner) {
        for (std::size_t neighbor = 0U; neighbor < positions.size(); ++neighbor) {
            ++work.graph_candidates;
            const long double radius = norm(positions[owner] - positions[neighbor]);
            if (strict ? radius < profile.horizon : radius <= profile.horizon) {
                graph.row[owner].push_back({neighbor, false});
                ++work.accepted_pairs;
            }
        }
        for (const std::size_t ghost : nearby_ghosts(grid, positions[owner])) {
            ++work.graph_candidates;
            const long double radius = norm(
                positions[owner] - widen(ghosts[ghost].position));
            if (strict ? radius < profile.horizon : radius <= profile.horizon) {
                graph.row[owner].push_back({ghost, true});
                ++work.accepted_pairs;
            }
        }
    }
    std::string bytes;
    append_string(bytes, strict ? "nextengine.nonlocal.ncgp13.graph.strict.v1"
                                : "nextengine.nonlocal.ncgp13.graph.v1");
    append_u64(bytes, positions.size());
    for (std::size_t owner = 0U; owner < positions.size(); ++owner) {
        append_u32(bytes, ids[owner]);
        append_u64(bytes, graph.row[owner].size());
        for (const Edge edge : graph.row[owner]) {
            append_u32(bytes, edge.ghost ? ghosts[edge.index].sample_id
                                         : ids[edge.index]);
            append_u32(bytes, edge.ghost ? 1U : 0U);
        }
    }
    graph.root = digest(std::move(bytes));
    ++work.hash_derivations;
    return graph;
}

struct DensityResult {
    bool valid = true;
    std::vector<long double> value;
    std::uint64_t candidates = 0U;
    std::string graph_root;
};

DensityResult independent_density(const NonlocalGpuProfile& profile,
    const std::vector<Vec3l>& positions,
    const std::vector<NonlocalGpuGhost>& ghosts,
    const GhostGrid& grid, bool brute_ghosts) {
    DensityResult result;
    result.value.assign(positions.size(), 0.0L);
    for (std::size_t owner = 0U; owner < positions.size(); ++owner) {
        for (const Vec3l candidate : positions) {
            ++result.candidates;
            const long double radius = norm(positions[owner] - candidate);
            if (radius <= profile.horizon) {
                result.value[owner] += profile.mass * kernel(radius, profile).value;
            }
        }
        if (brute_ghosts) {
            for (const auto& ghost : ghosts) {
                ++result.candidates;
                const long double radius = norm(
                    positions[owner] - widen(ghost.position));
                if (radius <= profile.horizon) {
                    result.value[owner] +=
                        profile.mass * kernel(radius, profile).value;
                }
            }
        } else {
            for (const std::size_t ghost : nearby_ghosts(grid, positions[owner])) {
                ++result.candidates;
                const long double radius = norm(
                    positions[owner] - widen(ghosts[ghost].position));
                if (radius <= profile.horizon) {
                    result.value[owner] +=
                        profile.mass * kernel(radius, profile).value;
                }
            }
        }
        result.valid = result.valid && std::isfinite(result.value[owner]);
    }
    return result;
}

DensityResult candidate_graph_density(const NonlocalGpuProfile& profile,
    const std::vector<std::uint32_t>& ids,
    const std::vector<Vec3l>& positions,
    const std::vector<NonlocalGpuGhost>& ghosts,
    const GhostGrid& grid, Work& work) {
    DensityResult result;
    result.value.assign(positions.size(), 0.0L);
    const Graph graph = build_graph(profile, ids, positions, ghosts, grid,
        false, work);
    result.valid = graph.valid;
    result.graph_root = graph.root;
    for (std::size_t owner = 0U; owner < positions.size(); ++owner) {
        for (const Edge edge : graph.row[owner]) {
            ++work.density_pairs;
            const Vec3l neighbor = edge.ghost
                ? widen(ghosts[edge.index].position) : positions[edge.index];
            result.value[owner] += profile.mass
                * kernel(norm(positions[owner] - neighbor), profile).value;
        }
        result.valid = result.valid && std::isfinite(result.value[owner]);
    }
    return result;
}

struct Assembly {
    bool valid = true;
    std::size_t count = 0U;
    std::size_t dofs = 0U;
    std::vector<long double> density;
    std::vector<long double> constraint;
    std::vector<long double> jacobian;
    std::vector<long double> matrix;
    std::vector<long double> rhs;
    long double symmetry = std::numeric_limits<long double>::infinity();
    std::string input_root;
    std::string graph_root;
    std::string density_root;
    std::string jacobian_root;
    std::string matrix_root;
    Work work;
};

void add_jacobian(std::vector<long double>& jacobian, std::size_t dofs,
    std::size_t row, std::size_t sample, Vec3l value, long double sign) {
    const std::size_t base = row * dofs + 3U * sample;
    jacobian[base] += sign * value.x;
    jacobian[base + 1U] += sign * value.y;
    jacobian[base + 2U] += sign * value.z;
}

std::string position_state_root(const std::vector<std::uint32_t>& ids,
    const std::vector<Vec3l>& positions) {
    return id_vec_root("nextengine.nonlocal.ncgp13.assembly-state.v1", ids,
        positions);
}

Assembly assemble(const NonlocalGpuProfile& profile,
    const std::vector<std::uint32_t>& ids,
    const std::vector<Vec3l>& positions,
    const std::vector<NonlocalGpuGhost>& ghosts, const GhostGrid& grid,
    bool include_ghost_derivative,
    const std::vector<long double>* external_displacement) {
    Assembly result;
    result.count = positions.size();
    result.dofs = 3U * result.count;
    result.input_root = position_state_root(ids, positions);
    result.density.assign(result.count, 0.0L);
    result.constraint.assign(result.count, 0.0L);
    result.jacobian.assign(result.count * result.dofs, 0.0L);
    result.matrix.assign(result.count * result.count, 0.0L);
    result.rhs.assign(result.count, 0.0L);
    const Graph graph = build_graph(profile, ids, positions, ghosts, grid,
        false, result.work);
    result.graph_root = graph.root;
    const long double mass = profile.mass;
    const long double rho0 = profile.rest_density;
    for (std::size_t owner = 0U; owner < result.count; ++owner) {
        for (const Edge edge : graph.row[owner]) {
            ++result.work.density_pairs;
            const Vec3l neighbor = edge.ghost
                ? widen(ghosts[edge.index].position) : positions[edge.index];
            const Vec3l difference = positions[owner] - neighbor;
            const long double radius = norm(difference);
            const Kernel values = kernel(radius, profile);
            result.density[owner] += mass * values.value;
            if (!(radius > 0.0L) || (edge.ghost && !include_ghost_derivative)
                || (!edge.ghost && edge.index == owner)) continue;
            ++result.work.derivative_pairs;
            const Vec3l derivative = difference
                * (mass / rho0 * values.first / radius);
            add_jacobian(result.jacobian, result.dofs, owner, owner,
                derivative, 1.0L);
            if (!edge.ghost) {
                add_jacobian(result.jacobian, result.dofs, owner, edge.index,
                    derivative, -1.0L);
            }
        }
        result.constraint[owner] =
            result.density[owner] / profile.rest_density - 1.0L;
    }
    const long double scale = static_cast<long double>(profile.dt) * profile.dt
        / profile.mass;
    for (std::size_t column = 0U; column < result.dofs; ++column) {
        std::vector<std::pair<std::size_t, long double>> entries;
        for (std::size_t row = 0U; row < result.count; ++row) {
            const long double value = result.jacobian[row * result.dofs + column];
            if (value != 0.0L) entries.emplace_back(row, value);
        }
        for (const auto& lhs : entries) {
            for (const auto& rhs : entries) {
                result.matrix[lhs.first * result.count + rhs.first] +=
                    scale * lhs.second * rhs.second;
                ++result.work.matrix_products;
            }
        }
    }
    for (std::size_t row = 0U; row < result.count; ++row) {
        result.rhs[row] = result.constraint[row];
        if (external_displacement != nullptr) {
            for (std::size_t column = 0U; column < result.dofs; ++column) {
                result.rhs[row] += result.jacobian[row * result.dofs + column]
                    * (*external_displacement)[column];
            }
        }
    }
    long double difference2 = 0.0L;
    long double reference2 = 0.0L;
    for (std::size_t row = 0U; row < result.count; ++row) {
        const long double diagonal = result.matrix[row * result.count + row];
        result.valid = result.valid && std::isfinite(result.density[row])
            && std::isfinite(result.constraint[row])
            && std::isfinite(result.rhs[row]) && std::isfinite(diagonal)
            && diagonal > 0.0L;
        for (std::size_t column = 0U; column < result.count; ++column) {
            const long double lhs = result.matrix[row * result.count + column];
            const long double rhs = result.matrix[column * result.count + row];
            difference2 += (lhs - rhs) * (lhs - rhs);
            reference2 += lhs * lhs + rhs * rhs;
            result.valid = result.valid && std::isfinite(lhs);
        }
    }
    result.symmetry = std::sqrt(difference2)
        / std::max(std::sqrt(0.5L * reference2), 1.0e-30L);
    result.density_root = id_scalar_root(
        "nextengine.nonlocal.ncgp13.density.v1", ids, result.density);
    {
        std::string bytes;
        append_string(bytes, "nextengine.nonlocal.ncgp13.jacobian.v1");
        append_u64(bytes, result.count);
        append_u64(bytes, result.dofs);
        for (std::size_t row = 0U; row < result.count; ++row) {
            append_u32(bytes, ids[row]);
            for (std::size_t column = 0U; column < result.dofs; ++column) {
                append_u32(bytes, ids[column / 3U]);
                append_u32(bytes, static_cast<std::uint32_t>(column % 3U));
                append_wide(bytes, result.jacobian[row * result.dofs + column]);
            }
        }
        result.jacobian_root = digest(std::move(bytes));
    }
    {
        std::string bytes;
        append_string(bytes, "nextengine.nonlocal.ncgp13.matrix.v1");
        append_u64(bytes, result.count);
        for (std::size_t row = 0U; row < result.count; ++row) {
            append_u32(bytes, ids[row]);
            for (std::size_t column = 0U; column < result.count; ++column) {
                append_u32(bytes, ids[column]);
                append_wide(bytes, result.matrix[row * result.count + column]);
            }
        }
        result.matrix_root = digest(std::move(bytes));
    }
    result.work.hash_derivations += 4U;
    if (result.work.hash_derivations != 5U) {
        throw std::logic_error("assembly hash accounting mismatch");
    }
    return result;
}

long double vector_norm(const std::vector<long double>& values) {
    long double sum = 0.0L;
    for (const long double value : values) sum += value * value;
    return std::sqrt(std::max(sum, 0.0L));
}

std::vector<long double> j_times(const Assembly& assembly,
    const std::vector<long double>& values, Work& work) {
    const std::uint64_t before = work.analytic_jv_multiply_adds;
    std::vector<long double> result(assembly.count, 0.0L);
    for (std::size_t row = 0U; row < assembly.count; ++row) {
        for (std::size_t column = 0U; column < assembly.dofs; ++column) {
            result[row] += assembly.jacobian[row * assembly.dofs + column]
                * values[column];
            ++work.analytic_jv_multiply_adds;
        }
    }
    const std::uint64_t expected = static_cast<std::uint64_t>(assembly.count)
        * static_cast<std::uint64_t>(assembly.dofs);
    if (work.analytic_jv_multiply_adds - before != expected) {
        throw std::logic_error("analytic Jv work accounting mismatch");
    }
    return result;
}

std::vector<long double> jt_times(const Assembly& assembly,
    const std::vector<long double>& values) {
    std::vector<long double> result(assembly.dofs, 0.0L);
    for (std::size_t row = 0U; row < assembly.count; ++row) {
        for (std::size_t column = 0U; column < assembly.dofs; ++column) {
            result[column] += assembly.jacobian[row * assembly.dofs + column]
                * values[row];
        }
    }
    return result;
}

struct Solve {
    bool valid = true;
    bool converged = false;
    std::vector<long double> lambda;
    std::vector<long double> gradient;
    std::size_t sweeps = 0U;
    std::uint64_t updates = 0U;
    long double primal = std::numeric_limits<long double>::infinity();
    long double projected_kkt = std::numeric_limits<long double>::infinity();
    long double complementarity = std::numeric_limits<long double>::infinity();
};

void recompute_gradient(const Assembly& assembly, Solve& solve) {
    for (std::size_t row = 0U; row < assembly.count; ++row) {
        long double value = -assembly.rhs[row];
        for (std::size_t column = 0U; column < assembly.count; ++column) {
            value += assembly.matrix[row * assembly.count + column]
                * solve.lambda[column];
        }
        solve.gradient[row] = value;
    }
}

void residuals(const Assembly& assembly, Solve& solve) {
    std::vector<long double> positive(assembly.count, 0.0L);
    std::vector<long double> source(assembly.count, 0.0L);
    std::vector<long double> projected(assembly.count, 0.0L);
    solve.complementarity = 0.0L;
    for (std::size_t row = 0U; row < assembly.count; ++row) {
        positive[row] = std::max(-solve.gradient[row], 0.0L);
        source[row] = std::max(assembly.rhs[row], 0.0L);
        const long double diagonal = assembly.matrix[row * assembly.count + row];
        projected[row] = std::min(diagonal * solve.lambda[row],
            solve.gradient[row]);
        solve.complementarity = std::max(solve.complementarity,
            std::abs(solve.lambda[row] * solve.gradient[row]));
    }
    solve.primal = vector_norm(positive)
        / std::max(vector_norm(source), 1.0e-30L);
    solve.projected_kkt = vector_norm(projected)
        / std::max(vector_norm(assembly.rhs), 1.0e-30L);
}

Solve solve_qp(const Assembly& assembly, Work& work) {
    Solve solve;
    solve.lambda.assign(assembly.count, 0.0L);
    solve.gradient.resize(assembly.count);
    recompute_gradient(assembly, solve);
    ++work.gradient_recomputations;
    for (std::size_t sweep = 1U; sweep <= kMaximumSweeps; ++sweep) {
        for (std::size_t row = 0U; row < assembly.count; ++row) {
            const long double diagonal = assembly.matrix[row * assembly.count + row];
            if (!(diagonal > 0.0L) || !std::isfinite(diagonal)) {
                solve.valid = false;
                return solve;
            }
            const long double prior = solve.lambda[row];
            const long double next = std::max(0.0L,
                prior - solve.gradient[row] / diagonal);
            const long double delta = next - prior;
            if (delta == 0.0L) continue;
            solve.lambda[row] = next;
            ++solve.updates;
            for (std::size_t affected = 0U; affected < assembly.count;
                 ++affected) {
                solve.gradient[affected] +=
                    assembly.matrix[affected * assembly.count + row] * delta;
            }
        }
        solve.sweeps = sweep;
        recompute_gradient(assembly, solve);
        ++work.gradient_recomputations;
        residuals(assembly, solve);
        solve.valid = solve.valid && std::isfinite(solve.primal)
            && std::isfinite(solve.projected_kkt)
            && std::isfinite(solve.complementarity)
            && std::all_of(solve.lambda.begin(), solve.lambda.end(),
                [](long double value) {
                    return std::isfinite(value) && value >= 0.0L;
                });
        if (!solve.valid) return solve;
        if (solve.primal <= kPrimalLimit
            && solve.projected_kkt <= kKktLimit
            && solve.complementarity <= kComplementarityLimit) {
            solve.converged = true;
            break;
        }
    }
    work.qp_sweeps += solve.sweeps;
    work.qp_updates += solve.updates;
    return solve;
}

long double jacobian_check(const NonlocalGpuProfile& profile,
    const std::vector<std::uint32_t>& ids,
    const std::vector<Vec3l>& positions,
    const std::vector<NonlocalGpuGhost>& ghosts, const GhostGrid& grid,
    const Assembly& assembly, bool brute_ghosts, Work& work) {
    std::vector<long double> direction(assembly.dofs, 0.0L);
    for (std::size_t index = 0U; index < ids.size(); ++index) {
        const std::int64_t id = static_cast<std::int64_t>(ids[index]);
        direction[3U * index] = static_cast<long double>((id % 17) - 8);
        direction[3U * index + 1U] = static_cast<long double>((id % 13) - 6);
        direction[3U * index + 2U] = static_cast<long double>((id % 11) - 5);
    }
    const long double magnitude = vector_norm(direction);
    for (long double& value : direction) value /= magnitude;
    const auto analytic = j_times(assembly, direction, work);
    const long double epsilon = std::ldexp(
        static_cast<long double>(profile.spacing), -20);
    std::vector<Vec3l> plus = positions;
    std::vector<Vec3l> minus = positions;
    for (std::size_t index = 0U; index < positions.size(); ++index) {
        const Vec3l delta{epsilon * direction[3U * index],
            epsilon * direction[3U * index + 1U],
            epsilon * direction[3U * index + 2U]};
        plus[index] += delta;
        minus[index] = minus[index] - delta;
    }
    const auto positive = independent_density(
        profile, plus, ghosts, grid, brute_ghosts);
    const auto negative = independent_density(
        profile, minus, ghosts, grid, brute_ghosts);
    work.finite_difference_candidates +=
        positive.candidates + negative.candidates;
    if (!positive.valid || !negative.valid) {
        return std::numeric_limits<long double>::infinity();
    }
    std::vector<long double> finite_difference(assembly.count, 0.0L);
    std::vector<long double> error(assembly.count, 0.0L);
    for (std::size_t row = 0U; row < assembly.count; ++row) {
        finite_difference[row] = (positive.value[row] - negative.value[row])
            / (2.0L * epsilon * profile.rest_density);
        error[row] = analytic[row] - finite_difference[row];
    }
    return vector_norm(error) / std::max({vector_norm(analytic),
        vector_norm(finite_difference), 1.0e-30L});
}

std::pair<long double, long double> strain_metrics(
    const std::vector<long double>& density, long double rho0) {
    long double maximum = 0.0L;
    long double sum = 0.0L;
    for (const long double value : density) {
        const long double strain = std::max(value / rho0 - 1.0L, 0.0L);
        maximum = std::max(maximum, strain);
        sum += strain * strain;
    }
    return {maximum, std::sqrt(sum / density.size())};
}

struct Contact {
    bool valid = true;
    Vec3l endpoint;
    Vec3l correction;
    Vec3l impulse;
    std::uint32_t mask = 0U;
    std::uint32_t first_mask = 0U;
    long double t_first = std::numeric_limits<long double>::infinity();
    std::uint64_t tests = 0U;
    std::uint64_t hits = 0U;
};

long double component(Vec3l value, std::size_t axis) {
    return axis == 0U ? value.x : axis == 1U ? value.y : value.z;
}
void set_component(Vec3l& value, std::size_t axis, long double component_value) {
    if (axis == 0U) value.x = component_value;
    else if (axis == 1U) value.y = component_value;
    else value.z = component_value;
}

Contact swept_contact(const NonlocalGpuProfile& profile, Vec3l start,
    Vec3l proposed) {
    Contact result;
    result.endpoint = proposed;
    const long double low = 0.5L * profile.spacing;
    const Vec3l high{profile.basin_extent.x - low,
        profile.basin_extent.y - low, profile.basin_extent.z - low};
    const bool finite_segment = finite(start) && finite(proposed);
    bool start_inside = finite_segment;
    if (finite_segment) {
        for (std::size_t axis = 0U; axis < 3U; ++axis) {
            const long double a = component(start, axis);
            start_inside = start_inside && a >= low
                && a <= component(high, axis);
        }
    }
    result.valid = finite_segment && start_inside;
    for (std::size_t face = 0U; face < 6U; ++face) {
        ++result.tests;
        const std::size_t axis = face / 2U;
        const bool high_face = (face & 1U) != 0U;
        const long double a = component(start, axis);
        const long double b = component(proposed, axis);
        const long double hi = component(high, axis);
        const bool crosses_plane = high_face ? b > hi : b < low;
        const bool hit = result.valid && crosses_plane;
        if (!hit) continue;
        const long double plane = high_face ? hi : low;
        const long double denominator = b - a;
        if (denominator == 0.0L) {
            result.valid = false;
            continue;
        }
        const long double time = (plane - a) / denominator;
        if (!std::isfinite(time) || time < 0.0L || time > 1.0L) {
            result.valid = false;
            continue;
        }
        const std::uint32_t bit = static_cast<std::uint32_t>(face);
        result.mask |= 1U << bit;
        ++result.hits;
        set_component(result.endpoint, axis, plane);
        if (time < result.t_first) {
            result.t_first = time;
            result.first_mask = 1U << bit;
        } else if (time == result.t_first) {
            result.first_mask |= 1U << bit;
        }
    }
    for (std::size_t axis = 0U; axis < 3U; ++axis) {
        const std::uint32_t low_bit = 1U << (2U * axis);
        const std::uint32_t high_bit = 1U << (2U * axis + 1U);
        if ((result.mask & low_bit) != 0U && (result.mask & high_bit) != 0U) {
            result.valid = false;
        }
    }
    result.correction = result.endpoint - proposed;
    result.impulse = result.correction
        * (static_cast<long double>(profile.mass) / profile.dt);
    return result;
}

struct ContactBatch {
    bool valid = true;
    std::vector<Vec3l> endpoint;
    std::vector<Vec3l> correction;
    std::vector<Vec3l> impulse;
    std::vector<std::uint32_t> mask;
    std::vector<std::uint32_t> first_mask;
    std::size_t lower_hits = 0U;
    Work work;
    std::string root;
};

ContactBatch contact_batch(const NonlocalGpuProfile& profile,
    const std::vector<std::uint32_t>& ids,
    const std::vector<Vec3l>& start,
    const std::vector<Vec3l>& proposed) {
    if (ids.size() != start.size() || ids.size() != proposed.size()) {
        throw std::logic_error("contact size mismatch");
    }
    ContactBatch result;
    result.endpoint.reserve(ids.size());
    result.correction.reserve(ids.size());
    result.impulse.reserve(ids.size());
    result.mask.reserve(ids.size());
    result.first_mask.reserve(ids.size());
    std::string bytes;
    append_string(bytes, "nextengine.nonlocal.ncgp13.contact.v1");
    append_u64(bytes, ids.size());
    for (std::size_t index = 0U; index < ids.size(); ++index) {
        const Contact contact = swept_contact(profile, start[index], proposed[index]);
        result.valid = result.valid && contact.valid;
        result.endpoint.push_back(contact.endpoint);
        result.correction.push_back(contact.correction);
        result.impulse.push_back(contact.impulse);
        result.mask.push_back(contact.mask);
        result.first_mask.push_back(contact.first_mask);
        result.work.plane_tests += contact.tests;
        result.work.plane_hits += contact.hits;
        if (contact.mask != 0U) ++result.work.contact_projections;
        if ((contact.mask & (1U << 4U)) != 0U) ++result.lower_hits;
        append_u32(bytes, ids[index]);
        append_u32(bytes, contact.mask);
        append_u32(bytes, contact.first_mask);
        append_wide(bytes, contact.endpoint.x);
        append_wide(bytes, contact.endpoint.y);
        append_wide(bytes, contact.endpoint.z);
        append_wide(bytes, contact.correction.x);
        append_wide(bytes, contact.correction.y);
        append_wide(bytes, contact.correction.z);
        append_wide(bytes, contact.impulse.x);
        append_wide(bytes, contact.impulse.y);
        append_wide(bytes, contact.impulse.z);
        append_wide(bytes, std::isfinite(contact.t_first)
                ? contact.t_first : -1.0L);
    }
    result.root = digest(std::move(bytes));
    ++result.work.hash_derivations;
    if (result.work.hash_derivations != 1U) {
        throw std::logic_error("contact hash accounting mismatch");
    }
    return result;
}

bool inset_oracle(const NonlocalGpuProfile& profile,
    const std::vector<Vec3l>& positions, long double& penetration) {
    const long double low = 0.5L * profile.spacing;
    const Vec3l high{profile.basin_extent.x - low,
        profile.basin_extent.y - low, profile.basin_extent.z - low};
    penetration = 0.0L;
    for (const Vec3l value : positions) {
        if (!finite(value)) return false;
        penetration = std::max(penetration, low - value.x);
        penetration = std::max(penetration, low - value.y);
        penetration = std::max(penetration, low - value.z);
        penetration = std::max(penetration, value.x - high.x);
        penetration = std::max(penetration, value.y - high.y);
        penetration = std::max(penetration, value.z - high.z);
    }
    penetration = std::max(penetration, 0.0L);
    return penetration == 0.0L;
}

Contact componentwise_contact_oracle(const NonlocalGpuProfile& profile,
    Vec3l start, Vec3l proposed) {
    Contact result;
    result.endpoint = proposed;
    const long double low = 0.5L * profile.spacing;
    const Vec3l high{profile.basin_extent.x - low,
        profile.basin_extent.y - low, profile.basin_extent.z - low};
    result.valid = finite(start) && finite(proposed);
    if (result.valid) {
        for (std::size_t axis = 0U; axis < 3U; ++axis) {
            const long double a = component(start, axis);
            result.valid = result.valid && a >= low
                && a <= component(high, axis);
        }
    }
    for (std::size_t axis = 0U; axis < 3U; ++axis) {
        const long double a = component(start, axis);
        const long double b = component(proposed, axis);
        const long double hi = component(high, axis);
        ++result.tests;
        const bool below_low = b < low;
        const bool low_hit = result.valid && below_low;
        ++result.tests;
        const bool above_high = b > hi;
        const bool high_hit = result.valid && above_high;
        if (!low_hit && !high_hit) continue;
        if (low_hit && high_hit) {
            result.valid = false;
            continue;
        }
        const long double plane = low_hit ? low : hi;
        const long double denominator = b - a;
        const long double time = denominator == 0.0L
            ? std::numeric_limits<long double>::infinity()
            : (plane - a) / denominator;
        if (!std::isfinite(time) || time < 0.0L || time > 1.0L) {
            result.valid = false;
            continue;
        }
        const std::uint32_t bit = static_cast<std::uint32_t>(
            2U * axis + (high_hit ? 1U : 0U));
        result.mask |= 1U << bit;
        ++result.hits;
        set_component(result.endpoint, axis, plane);
        if (time < result.t_first) {
            result.t_first = time;
            result.first_mask = 1U << bit;
        } else if (time == result.t_first) {
            result.first_mask |= 1U << bit;
        }
    }
    result.correction = result.endpoint - proposed;
    result.impulse = result.correction
        * (static_cast<long double>(profile.mass) / profile.dt);
    return result;
}

bool same_scalar(long double lhs, long double rhs) {
    if (std::isnan(lhs) || std::isnan(rhs)) {
        return std::isnan(lhs) && std::isnan(rhs);
    }
    return lhs == rhs;
}

bool same_vec(Vec3l lhs, Vec3l rhs) {
    return same_scalar(lhs.x, rhs.x) && same_scalar(lhs.y, rhs.y)
        && same_scalar(lhs.z, rhs.z);
}

bool same_contact(const Contact& candidate, const Contact& expected) {
    return candidate.valid == expected.valid
        && candidate.mask == expected.mask
        && candidate.first_mask == expected.first_mask
        && candidate.tests == expected.tests && candidate.hits == expected.hits
        && same_scalar(candidate.t_first, expected.t_first)
        && same_vec(candidate.endpoint, expected.endpoint)
        && same_vec(candidate.correction, expected.correction)
        && same_vec(candidate.impulse, expected.impulse);
}

void append_classified_wide(std::string& bytes, long double value) {
    if (std::isfinite(value)) {
        append_u32(bytes, 0U);
        append_wide(bytes, value);
    } else if (std::isnan(value)) {
        append_u32(bytes, 1U);
        append_wide(bytes, 0.0L);
    } else if (value > 0.0L) {
        append_u32(bytes, 2U);
        append_wide(bytes, 0.0L);
    } else {
        append_u32(bytes, 3U);
        append_wide(bytes, 0.0L);
    }
}

void append_contact_record(std::string& bytes, const Contact& contact) {
    append_u64(bytes, contact.valid ? 1U : 0U);
    append_u32(bytes, contact.mask);
    append_u32(bytes, contact.first_mask);
    append_u64(bytes, contact.tests);
    append_u64(bytes, contact.hits);
    append_classified_wide(bytes, contact.t_first);
    for (const Vec3l value : std::array<Vec3l, 3>{contact.endpoint,
             contact.correction, contact.impulse}) {
        append_classified_wide(bytes, value.x);
        append_classified_wide(bytes, value.y);
        append_classified_wide(bytes, value.z);
    }
}

bool contact_oracle_control(const NonlocalGpuProfile& profile,
    std::string& root, Work& work) {
    const long double low = 0.5L * profile.spacing;
    const Vec3l high{profile.basin_extent.x - low,
        profile.basin_extent.y - low, profile.basin_extent.z - low};
    const Vec3l center{0.5L * (low + high.x), 0.5L * (low + high.y),
        0.5L * (low + high.z)};
    bool pass = true;
    struct CaseResult {
        std::string name;
        Vec3l start;
        Vec3l proposed;
        Contact candidate;
        Contact expected;
    };
    std::vector<CaseResult> cases;
    auto test = [&](std::string name, Vec3l start, Vec3l proposed) {
        CaseResult item{std::move(name), start, proposed,
            swept_contact(profile, start, proposed),
            componentwise_contact_oracle(profile, start, proposed)};
        work.plane_tests += item.candidate.tests;
        work.plane_hits += item.candidate.hits;
        if (item.candidate.mask != 0U) ++work.contact_projections;
        pass = pass && item.candidate.tests == 6U
            && same_contact(item.candidate, item.expected);
        cases.push_back(item);
        return item.candidate;
    };
    test("interior", center, center + Vec3l{0.01L, -0.01L, 0.01L});
    for (std::size_t face = 0U; face < 6U; ++face) {
        const std::size_t axis = face / 2U;
        const bool high_face = (face & 1U) != 0U;
        const long double plane = high_face ? component(high, axis) : low;
        const long double sign = high_face ? 1.0L : -1.0L;
        Vec3l single = center;
        set_component(single, axis, plane + sign * 0.01L);
        test("single-face-" + std::to_string(face), center, single);
        Vec3l on_plane = center;
        set_component(on_plane, axis, plane);
        Vec3l outward = on_plane;
        set_component(outward, axis, plane + sign * 0.01L);
        const Contact outward_result = test(
            "on-plane-outward-" + std::to_string(face), on_plane, outward);
        pass = pass && outward_result.t_first == 0.0L
            && sign * component(outward_result.impulse, axis) < 0.0L;
        Vec3l inward = on_plane;
        set_component(inward, axis, plane - sign * 0.01L);
        test("on-plane-inward-" + std::to_string(face), on_plane, inward);
    }
    test("low-corner-tie", {low, low, low},
        {low - 0.01L, low - 0.01L, low - 0.01L});
    test("high-corner-tie", {high.x, high.y, high.z},
        {high.x + 0.01L, high.y + 0.01L, high.z + 0.01L});
    Vec3l tangent_end = center + Vec3l{0.02L, 0.03L, 0.0L};
    tangent_end.z = low - 0.01L;
    const Contact tangent = test("tangential-retained", center, tangent_end);
    pass = pass && tangent.endpoint.x == tangent_end.x
        && tangent.endpoint.y == tangent_end.y;
    const Contact a = test("nonexpansive-a", center,
        center + Vec3l{0.0L, 0.0L, -10.0L});
    const Contact b = test("nonexpansive-b", center,
        center + Vec3l{0.01L, 0.0L, -20.0L});
    pass = pass && norm(a.endpoint - b.endpoint)
        <= norm((center + Vec3l{0.0L, 0.0L, -10.0L})
            - (center + Vec3l{0.01L, 0.0L, -20.0L}));
    Vec3l outside = center;
    outside.x = low - 0.01L;
    pass = pass && !test("starting-outside", outside, center).valid;
    Vec3l nonfinite_start = center;
    nonfinite_start.x = std::numeric_limits<long double>::quiet_NaN();
    pass = pass && !test("nonfinite-start", nonfinite_start, center).valid;
    Vec3l nonfinite_end = center;
    nonfinite_end.z = std::numeric_limits<long double>::infinity();
    pass = pass && !test("nonfinite-end", center, nonfinite_end).valid;
    std::string bytes;
    append_string(bytes, "nextengine.nonlocal.ncgp13.contact-control.v1");
    append_u64(bytes, pass ? 1U : 0U);
    append_u64(bytes, cases.size());
    for (const CaseResult& item : cases) {
        append_string(bytes, item.name);
        for (const Vec3l value : std::array<Vec3l, 2>{item.start,
                 item.proposed}) {
            append_classified_wide(bytes, value.x);
            append_classified_wide(bytes, value.y);
            append_classified_wide(bytes, value.z);
        }
        append_contact_record(bytes, item.candidate);
        append_contact_record(bytes, item.expected);
    }
    root = digest(std::move(bytes));
    ++work.hash_derivations;
    return pass;
}

std::size_t components(const NonlocalGpuProfile& profile,
    const std::vector<Vec3l>& positions, Work& work) {
    if (positions.empty()) return 0U;
    const std::uint64_t graph_builds_before = work.graph_builds;
    const std::uint64_t graph_candidates_before = work.graph_candidates;
    const std::uint64_t accepted_pairs_before = work.accepted_pairs;
    const std::uint64_t topology_tests_before = work.topology_distance_tests;
    const std::uint64_t topology_discoveries_before =
        work.topology_discoveries;
    ++work.graph_builds;
    std::vector<bool> visited(positions.size(), false);
    std::size_t count = 0U;
    for (std::size_t start = 0U; start < positions.size(); ++start) {
        if (visited[start]) continue;
        ++count;
        std::queue<std::size_t> pending;
        pending.push(start);
        visited[start] = true;
        ++work.topology_discoveries;
        while (!pending.empty()) {
            const std::size_t owner = pending.front();
            pending.pop();
            for (std::size_t neighbor = 0U; neighbor < positions.size();
                 ++neighbor) {
                if (visited[neighbor]) continue;
                ++work.graph_candidates;
                ++work.topology_distance_tests;
                if (norm(positions[owner] - positions[neighbor])
                    <= profile.horizon) {
                    visited[neighbor] = true;
                    ++work.accepted_pairs;
                    ++work.topology_discoveries;
                    pending.push(neighbor);
                }
            }
        }
    }
    const std::uint64_t topology_discoveries =
        work.topology_discoveries - topology_discoveries_before;
    if (work.graph_builds - graph_builds_before != 1U
        || work.graph_candidates - graph_candidates_before
            != work.topology_distance_tests - topology_tests_before
        || work.accepted_pairs - accepted_pairs_before + count
            != topology_discoveries) {
        throw std::logic_error("topology graph work accounting mismatch");
    }
    return count;
}

std::string legacy_scalar_root(std::string_view domain,
    const std::vector<long double>& values) {
    std::string bytes;
    append_string(bytes, domain);
    append_u64(bytes, values.size());
    for (const long double value : values) append_wide(bytes, value);
    return digest(std::move(bytes));
}

std::string legacy_position_root(std::string_view domain,
    const std::vector<Vec3l>& values) {
    std::string bytes;
    append_string(bytes, domain);
    append_u64(bytes, values.size());
    for (const Vec3l value : values) {
        append_wide(bytes, value.x);
        append_wide(bytes, value.y);
        append_wide(bytes, value.z);
    }
    return digest(std::move(bytes));
}

long double maximum_penetration(const NonlocalGpuProfile& profile,
    const std::vector<Vec3l>& positions) {
    long double penetration = 0.0L;
    static_cast<void>(inset_oracle(profile, positions, penetration));
    return penetration;
}

struct LegacyWitness {
    bool pass = false;
    std::size_t sweeps = 0U;
    std::uint64_t updates = 0U;
    std::size_t positive = 0U;
    long double jacobian = std::numeric_limits<long double>::infinity();
    long double symmetry = std::numeric_limits<long double>::infinity();
    long double primal = std::numeric_limits<long double>::infinity();
    long double kkt = std::numeric_limits<long double>::infinity();
    long double complementarity = std::numeric_limits<long double>::infinity();
    long double linearized = std::numeric_limits<long double>::infinity();
    long double maximum_strain = std::numeric_limits<long double>::infinity();
    long double rms_strain = std::numeric_limits<long double>::infinity();
    long double stationarity = std::numeric_limits<long double>::infinity();
    long double penetration = std::numeric_limits<long double>::infinity();
    std::string multiplier_root;
    std::string trial_root;
    std::string result_root;
    Work work;
};

bool relative_close(long double candidate, long double retained,
    long double limit = 2.0e-12L) {
    return std::abs(candidate - retained)
        / std::max({std::abs(candidate), std::abs(retained), 1.0e-30L})
        <= limit;
}

LegacyWitness legacy_witness(const NonlocalGpuProfile& profile,
    const State& state, const std::vector<NonlocalGpuGhost>& ghosts,
    const GhostGrid& grid, std::string_view input_root) {
    LegacyWitness result;
    const long double dt2 = static_cast<long double>(profile.dt) * profile.dt;
    std::vector<long double> gravity(3U * state.id.size(), 0.0L);
    for (std::size_t index = 0U; index < state.id.size(); ++index) {
        gravity[3U * index] = dt2 * profile.gravity.x;
        gravity[3U * index + 1U] = dt2 * profile.gravity.y;
        gravity[3U * index + 2U] = dt2 * profile.gravity.z;
    }
    const Assembly assembly = assemble(profile, state.id, state.position,
        ghosts, grid, true, &gravity);
    add_work(result.work, assembly.work);
    result.jacobian = jacobian_check(profile, state.id, state.position,
        ghosts, grid, assembly, true, result.work);
    result.symmetry = assembly.symmetry;
    Solve solve = solve_qp(assembly, result.work);
    result.sweeps = solve.sweeps;
    result.updates = solve.updates;
    result.primal = solve.primal;
    result.kkt = solve.projected_kkt;
    result.complementarity = solve.complementarity;
    result.positive = static_cast<std::size_t>(std::count_if(
        solve.lambda.begin(), solve.lambda.end(),
        [](long double value) { return value > 0.0L; }));
    const auto jt = jt_times(assembly, solve.lambda);
    const long double scale = dt2 / profile.mass;
    std::vector<long double> displacement(assembly.dofs, 0.0L);
    std::vector<Vec3l> trial;
    trial.reserve(state.id.size());
    for (std::size_t index = 0U; index < state.id.size(); ++index) {
        const Vec3l step{gravity[3U * index] - scale * jt[3U * index],
            gravity[3U * index + 1U] - scale * jt[3U * index + 1U],
            gravity[3U * index + 2U] - scale * jt[3U * index + 2U]};
        displacement[3U * index] = step.x;
        displacement[3U * index + 1U] = step.y;
        displacement[3U * index + 2U] = step.z;
        trial.push_back(state.position[index] + step);
    }
    const auto linear = j_times(assembly, displacement, result.work);
    std::vector<long double> dual(assembly.count, 0.0L);
    std::vector<long double> linear_error(assembly.count, 0.0L);
    for (std::size_t row = 0U; row < assembly.count; ++row) {
        long double action = 0.0L;
        for (std::size_t column = 0U; column < assembly.count; ++column) {
            action += assembly.matrix[row * assembly.count + column]
                * solve.lambda[column];
        }
        dual[row] = assembly.rhs[row] - action;
        linear_error[row] = assembly.constraint[row] + linear[row] - dual[row];
    }
    result.linearized = vector_norm(linear_error)
        / std::max(vector_norm(dual), 1.0e-30L);
    const auto density = independent_density(profile, trial, ghosts, grid, true);
    result.work.independent_density_candidates += density.candidates;
    const auto strains = strain_metrics(density.value, profile.rest_density);
    result.maximum_strain = strains.first;
    result.rms_strain = strains.second;
    result.penetration = maximum_penetration(profile, trial);
    std::vector<long double> stationarity(assembly.dofs, 0.0L);
    std::vector<long double> gravity_force(assembly.dofs, 0.0L);
    for (std::size_t index = 0U; index < state.id.size(); ++index) {
        for (std::size_t axis = 0U; axis < 3U; ++axis) {
            const std::size_t dof = 3U * index + axis;
            const long double acceleration = axis == 0U ? profile.gravity.x
                : axis == 1U ? profile.gravity.y : profile.gravity.z;
            stationarity[dof] = profile.mass / dt2
                    * (displacement[dof] - dt2 * acceleration)
                + jt[dof];
            gravity_force[dof] = profile.mass * acceleration;
        }
    }
    result.stationarity = vector_norm(stationarity)
        / std::max(vector_norm(gravity_force), 1.0e-30L);
    result.multiplier_root = legacy_scalar_root(
        "nextengine.nonlocal.ncgp12.multiplier.v1", solve.lambda);
    result.trial_root = legacy_position_root(
        "nextengine.nonlocal.ncgp12.trial.v1", trial);
    result.work.hash_derivations += 2U;
    result.pass = input_root == kPhaseAInput && assembly.valid && solve.valid
        && solve.converged && result.multiplier_root == kLegacyMultiplier
        && result.trial_root == kLegacyTrial && result.sweeps == 726U
        && result.updates == 43612U && result.positive == 60U
        && relative_close(result.jacobian, 2.8023154802122483e-11L)
        && relative_close(result.symmetry, 2.943682118562611e-20L)
        && relative_close(result.primal, 9.902735128437468e-9L)
        && relative_close(result.kkt, 1.559010213940065e-11L)
        && relative_close(result.complementarity, 5.949392072812528e-12L)
        && relative_close(result.linearized, 1.9226987759652108e-19L)
        && relative_close(result.maximum_strain, 3.574240623292345e-6L)
        && relative_close(result.rms_strain, 5.801039442456803e-7L)
        && relative_close(result.stationarity, 2.0784084315481946e-20L)
        && relative_close(result.penetration, 1.7878107032179183e-4L)
        && result.penetration > 0.0L;
    std::string bytes;
    append_string(bytes, "nextengine.nonlocal.ncgp13.legacy-witness.v1");
    append_u64(bytes, result.pass ? 1U : 0U);
    append_u64(bytes, result.sweeps);
    append_u64(bytes, result.updates);
    append_u64(bytes, result.positive);
    for (const long double value : std::array<long double, 10>{
             result.jacobian, result.symmetry, result.primal, result.kkt,
             result.complementarity, result.linearized,
             result.maximum_strain, result.rms_strain,
             result.stationarity, result.penetration}) {
        append_wide(bytes, value);
    }
    append_string(bytes, result.multiplier_root);
    append_string(bytes, result.trial_root);
    append_string(bytes, work_root(result.work));
    result.result_root = digest(std::move(bytes));
    return result;
}

long double directional_check(const NonlocalGpuProfile& profile,
    const std::vector<Vec3l>& positions,
    const std::vector<NonlocalGpuGhost>& ghosts, const GhostGrid& grid,
    const Assembly& assembly, Work& work) {
    std::vector<long double> direction(assembly.dofs, 0.0L);
    for (std::size_t index = 0U; index < positions.size(); ++index) {
        direction[3U * index + 2U] = -1.0L;
    }
    const auto analytic = j_times(assembly, direction, work);
    const long double epsilon = std::ldexp(
        static_cast<long double>(profile.spacing), -20);
    std::vector<Vec3l> plus = positions;
    std::vector<Vec3l> minus = positions;
    for (std::size_t index = 0U; index < positions.size(); ++index) {
        plus[index].z -= epsilon;
        minus[index].z += epsilon;
    }
    const auto positive = independent_density(profile, plus, ghosts, grid, true);
    const auto negative = independent_density(profile, minus, ghosts, grid, true);
    work.finite_difference_candidates +=
        positive.candidates + negative.candidates;
    std::vector<long double> finite_difference(assembly.count, 0.0L);
    std::vector<long double> difference(assembly.count, 0.0L);
    for (std::size_t row = 0U; row < assembly.count; ++row) {
        finite_difference[row] = (positive.value[row] - negative.value[row])
            / (2.0L * epsilon * profile.rest_density);
        difference[row] = analytic[row] - finite_difference[row];
    }
    return vector_norm(difference) / std::max({vector_norm(analytic),
        vector_norm(finite_difference), 1.0e-30L});
}

std::string id_mask_root(std::string_view domain,
    const std::vector<std::uint32_t>& ids,
    const std::vector<std::uint32_t>& values) {
    if (ids.size() != values.size()) throw std::logic_error("mask root mismatch");
    std::string bytes;
    append_string(bytes, domain);
    append_u64(bytes, ids.size());
    for (std::size_t index = 0U; index < ids.size(); ++index) {
        append_u32(bytes, ids[index]);
        append_u32(bytes, values[index]);
    }
    return digest(std::move(bytes));
}

struct RoundSummary {
    bool assembly_valid = false;
    bool solve_valid = false;
    bool converged = false;
    bool stale = false;
    std::size_t sweeps = 0U;
    std::uint64_t updates = 0U;
    std::size_t positive = 0U;
    long double jacobian = 0.0L;
    long double symmetry = 0.0L;
    long double primal = 0.0L;
    long double kkt = 0.0L;
    long double complementarity = 0.0L;
    long double pressure_balance = 0.0L;
    long double maximum_strain = 0.0L;
    long double rms_strain = 0.0L;
    long double penetration = 0.0L;
    long double density_correspondence = 0.0L;
    std::string assembly_input_root;
    std::string graph_root;
    std::string density_root;
    std::string jacobian_root;
    std::string matrix_root;
    std::string post_graph_root;
    std::string post_density_root;
    std::string independent_density_root;
    std::string multiplier_root;
    std::string contact_root;
    std::string state_root;
    std::string result_root;
    std::uint64_t expected_hash_derivations = 0U;
    bool hash_accounting_exact = false;
    Work work;
};

struct StepResult {
    bool apparatus_valid = true;
    bool physical_pass = false;
    bool accepted = false;
    bool identity_mass_exact = false;
    std::string failure;
    State state;
    std::vector<RoundSummary> rounds;
    std::vector<long double> lambda_sum;
    std::vector<Vec3l> pressure_jt_sum;
    std::vector<Vec3l> contact_delta_sum;
    std::vector<Vec3l> contact_impulse_sum;
    std::vector<std::uint32_t> contact_mask;
    std::size_t positive_multipliers = 0U;
    std::size_t lower_contacts = 0U;
    long double maximum_strain = 0.0L;
    long double rms_strain = 0.0L;
    long double penetration = 0.0L;
    long double independent_correspondence = 0.0L;
    long double balance = 0.0L;
    long double bottom_pressure_proxy = 0.0L;
    long double top_pressure_proxy = 0.0L;
    std::size_t component_count = 0U;
    std::string multiplier_root;
    std::string predictor_contact_root;
    std::string injected_private_multiplier_root;
    std::string injected_private_contact_root;
    std::string contact_mask_root;
    std::string contact_impulse_root;
    std::string state_root;
    std::string velocity_root;
    std::string result_root;
    std::uint64_t expected_hash_derivations = 0U;
    bool hash_accounting_exact = false;
    Work work;
};

long double dense_norm(const std::vector<Vec3l>& values) {
    long double sum = 0.0L;
    for (const Vec3l value : values) sum += dot(value, value);
    return std::sqrt(std::max(sum, 0.0L));
}

std::string round_root(const RoundSummary& round) {
    std::string bytes;
    append_string(bytes, "nextengine.nonlocal.ncgp13.round.v1");
    append_u64(bytes, round.assembly_valid ? 1U : 0U);
    append_u64(bytes, round.solve_valid ? 1U : 0U);
    append_u64(bytes, round.converged ? 1U : 0U);
    append_u64(bytes, round.stale ? 1U : 0U);
    append_u64(bytes, round.expected_hash_derivations);
    append_u64(bytes, round.hash_accounting_exact ? 1U : 0U);
    append_u64(bytes, round.sweeps);
    append_u64(bytes, round.updates);
    append_u64(bytes, round.positive);
    for (const long double value : std::array<long double, 10>{round.jacobian,
             round.symmetry, round.primal, round.kkt, round.complementarity,
             round.pressure_balance, round.maximum_strain, round.rms_strain,
             round.penetration, round.density_correspondence}) {
        append_wide(bytes, value);
    }
    append_string(bytes, round.assembly_input_root);
    append_string(bytes, round.graph_root);
    append_string(bytes, round.density_root);
    append_string(bytes, round.jacobian_root);
    append_string(bytes, round.matrix_root);
    append_string(bytes, round.post_graph_root);
    append_string(bytes, round.post_density_root);
    append_string(bytes, round.independent_density_root);
    append_string(bytes, round.multiplier_root);
    append_string(bytes, round.contact_root);
    append_string(bytes, round.state_root);
    append_string(bytes, work_root(round.work));
    return digest(std::move(bytes));
}

std::string step_root(const StepResult& step) {
    std::string bytes;
    append_string(bytes, "nextengine.nonlocal.ncgp13.step.v1");
    append_u64(bytes, step.apparatus_valid ? 1U : 0U);
    append_u64(bytes, step.physical_pass ? 1U : 0U);
    append_u64(bytes, step.accepted ? 1U : 0U);
    append_u64(bytes, step.identity_mass_exact ? 1U : 0U);
    append_u64(bytes, step.expected_hash_derivations);
    append_u64(bytes, step.hash_accounting_exact ? 1U : 0U);
    append_string(bytes, step.failure);
    append_u64(bytes, step.rounds.size());
    append_u64(bytes, step.positive_multipliers);
    append_u64(bytes, step.lower_contacts);
    append_u64(bytes, step.component_count);
    for (const long double value : std::array<long double, 7>{
             step.maximum_strain, step.rms_strain, step.penetration,
             step.independent_correspondence, step.balance,
             step.bottom_pressure_proxy, step.top_pressure_proxy}) {
        append_wide(bytes, value);
    }
    for (const RoundSummary& round : step.rounds) {
        append_string(bytes, round.result_root);
    }
    append_string(bytes, step.multiplier_root);
    append_string(bytes, step.predictor_contact_root);
    append_string(bytes, step.injected_private_multiplier_root);
    append_string(bytes, step.injected_private_contact_root);
    append_string(bytes, step.contact_mask_root);
    append_string(bytes, step.contact_impulse_root);
    append_string(bytes, step.state_root);
    append_string(bytes, step.velocity_root);
    append_string(bytes, work_root(step.work));
    return digest(std::move(bytes));
}

void seal_round(RoundSummary& round,
    std::uint64_t expected_hash_derivations) {
    round.expected_hash_derivations = expected_hash_derivations;
    round.hash_accounting_exact =
        round.work.hash_derivations == expected_hash_derivations;
    if (!round.hash_accounting_exact) {
        throw std::logic_error("round hash accounting mismatch");
    }
    round.result_root = round_root(round);
}

void seal_step(StepResult& result, const std::vector<std::uint32_t>& ids,
    std::uint64_t expected_before_standard_roots) {
    result.multiplier_root = id_scalar_root(
        "nextengine.nonlocal.ncgp13.step-multiplier.v1", ids,
        result.lambda_sum);
    result.contact_mask_root = id_mask_root(
        "nextengine.nonlocal.ncgp13.step-contact-mask.v1", ids,
        result.contact_mask);
    result.contact_impulse_root = id_vec_root(
        "nextengine.nonlocal.ncgp13.step-contact-impulse.v1", ids,
        result.contact_impulse_sum);
    result.state_root = state_root(
        "nextengine.nonlocal.ncgp13.accepted-state.v1", result.state);
    result.velocity_root = id_vec_root(
        "nextengine.nonlocal.ncgp13.velocity.v1", ids,
        result.state.velocity);
    result.work.hash_derivations += 5U;
    result.expected_hash_derivations = expected_before_standard_roots + 5U;
    result.hash_accounting_exact = result.work.hash_derivations
        == result.expected_hash_derivations;
    if (!result.hash_accounting_exact) {
        throw std::logic_error("step hash accounting mismatch");
    }
    result.result_root = step_root(result);
}

StepResult run_step(const NonlocalGpuProfile& profile, const State& input,
    const std::vector<NonlocalGpuGhost>& ghosts, const GhostGrid& grid,
    bool phase_a, bool inject_failure = false) {
    StepResult result;
    result.state = input;
    const std::size_t count = input.id.size();
    result.lambda_sum.assign(count, 0.0L);
    result.pressure_jt_sum.assign(count, {});
    result.contact_delta_sum.assign(count, {});
    result.contact_impulse_sum.assign(count, {});
    result.contact_mask.assign(count, 0U);
    const long double dt = profile.dt;
    const long double dt2 = dt * dt;
    const Vec3l gravity = widen(profile.gravity);
    std::vector<Vec3l> displacement(count);
    std::vector<Vec3l> proposed(count);
    for (std::size_t index = 0U; index < count; ++index) {
        displacement[index] = input.velocity[index] * dt + gravity * dt2;
        proposed[index] = input.position[index] + displacement[index];
    }
    ContactBatch predictor = contact_batch(
        profile, input.id, input.position, proposed);
    add_work(result.work, predictor.work);
    std::uint64_t expected_step_hashes = 1U;
    result.predictor_contact_root = predictor.root;
    if (!predictor.valid) {
        result.apparatus_valid = false;
        result.failure = "PREDICTOR_CONTACT_INVALID";
        seal_step(result, input.id, expected_step_hashes);
        return result;
    }
    std::vector<Vec3l> y = predictor.endpoint;
    for (std::size_t index = 0U; index < count; ++index) {
        result.contact_delta_sum[index] += predictor.correction[index];
        result.contact_impulse_sum[index] += predictor.impulse[index];
        result.contact_mask[index] |= predictor.mask[index];
    }
    result.lower_contacts += predictor.lower_hits;

    if (inject_failure) {
        const Assembly private_assembly = assemble(profile, input.id, y,
            ghosts, grid, true, nullptr);
        add_work(result.work, private_assembly.work);
        Solve private_solve = solve_qp(private_assembly, result.work);
        std::vector<Vec3l> private_proposed = y;
        if (private_solve.valid && private_solve.converged) {
            const auto jt = jt_times(private_assembly, private_solve.lambda);
            for (std::size_t index = 0U; index < count; ++index) {
                private_proposed[index] += Vec3l{jt[3U * index],
                    jt[3U * index + 1U], jt[3U * index + 2U]}
                    * (-dt2 / profile.mass);
            }
        }
        const ContactBatch private_contact = contact_batch(
            profile, input.id, y, private_proposed);
        add_work(result.work, private_contact.work);
        result.injected_private_multiplier_root = id_scalar_root(
            "nextengine.nonlocal.ncgp13.injected-multiplier.v1", input.id,
            private_solve.lambda);
        ++result.work.hash_derivations;
        expected_step_hashes += 7U;
        result.injected_private_contact_root = private_contact.root;
        result.apparatus_valid = private_assembly.valid && private_solve.valid
            && private_solve.converged && private_contact.valid;
        result.identity_mass_exact = true;
        result.failure = "INJECTED_AFTER_PRIVATE_ROUND";
        seal_step(result, input.id, expected_step_hashes);
        return result;
    }

    std::vector<long double> last_density;
    auto record_round = [&](RoundSummary& round,
                            std::uint64_t expected_hash_derivations) {
        seal_round(round, expected_hash_derivations);
        expected_step_hashes += expected_hash_derivations;
        result.rounds.push_back(round);
        add_work(result.work, round.work);
    };
    for (std::size_t round_index = 0U; round_index < kMaximumRounds;
         ++round_index) {
        RoundSummary round;
        ++round.work.projection_rounds;
        ++round.work.hash_derivations;
        const std::string expected_input = position_state_root(input.id, y);
        const Assembly assembly = assemble(profile, input.id, y, ghosts, grid,
            true, nullptr);
        add_work(round.work, assembly.work);
        round.assembly_valid = assembly.valid;
        round.assembly_input_root = assembly.input_root;
        round.graph_root = assembly.graph_root;
        round.density_root = assembly.density_root;
        round.jacobian_root = assembly.jacobian_root;
        round.matrix_root = assembly.matrix_root;
        round.stale = assembly.input_root != expected_input;
        round.symmetry = assembly.symmetry;
        round.jacobian = phase_a
            ? jacobian_check(profile, input.id, y, ghosts, grid, assembly,
                true, round.work)
            : 0.0L;
        if (!assembly.valid || round.stale
            || assembly.symmetry > kSymmetryLimit
            || (phase_a && round.jacobian > kJacobianLimit)) {
            result.apparatus_valid = false;
            result.failure = round.stale ? "STALE_ASSEMBLY_REJECTED"
                : "ASSEMBLY_OR_DERIVATIVE_INVALID";
            record_round(round, 6U);
            break;
        }
        Solve solve = solve_qp(assembly, round.work);
        round.solve_valid = solve.valid;
        round.converged = solve.converged;
        round.sweeps = solve.sweeps;
        round.updates = solve.updates;
        round.primal = solve.primal;
        round.kkt = solve.projected_kkt;
        round.complementarity = solve.complementarity;
        round.positive = static_cast<std::size_t>(std::count_if(
            solve.lambda.begin(), solve.lambda.end(),
            [](long double value) { return value > 0.0L; }));
        round.multiplier_root = id_scalar_root(
            "nextengine.nonlocal.ncgp13.round-multiplier.v1", input.id,
            solve.lambda);
        ++round.work.hash_derivations;
        if (!solve.valid) {
            result.apparatus_valid = false;
            result.failure = "QP_INVALID";
            record_round(round, 7U);
            break;
        }
        if (!solve.converged || solve.primal > kPrimalLimit
            || solve.projected_kkt > kKktLimit
            || solve.complementarity > kComplementarityLimit) {
            result.failure = "QP_BUDGET_OR_GATE_EXHAUSTED";
            record_round(round, 7U);
            break;
        }
        const auto jt = jt_times(assembly, solve.lambda);
        const long double correction_scale = dt2 / profile.mass;
        std::vector<Vec3l> pressure_delta(count);
        std::vector<Vec3l> pressure_force(count);
        std::vector<Vec3l> pressure_residual(count);
        std::vector<Vec3l> pressure_proposed(count);
        for (std::size_t index = 0U; index < count; ++index) {
            pressure_force[index] = {jt[3U * index], jt[3U * index + 1U],
                jt[3U * index + 2U]};
            pressure_delta[index] = pressure_force[index] * (-correction_scale);
            pressure_residual[index] = pressure_delta[index]
                    * (profile.mass / dt2)
                + pressure_force[index];
            pressure_proposed[index] = y[index] + pressure_delta[index];
            result.lambda_sum[index] += solve.lambda[index];
            result.pressure_jt_sum[index] += pressure_force[index];
        }
        round.pressure_balance = dense_norm(pressure_residual)
            / std::max({dense_norm(pressure_force),
                profile.mass * norm(gravity) * std::sqrt(
                    static_cast<long double>(count)),
                1.0e-30L * static_cast<long double>(count)});
        const ContactBatch contact = contact_batch(
            profile, input.id, y, pressure_proposed);
        add_work(round.work, contact.work);
        round.contact_root = contact.root;
        if (!contact.valid) {
            result.apparatus_valid = false;
            result.failure = "ROUND_CONTACT_INVALID";
            record_round(round, 8U);
            break;
        }
        y = contact.endpoint;
        for (std::size_t index = 0U; index < count; ++index) {
            result.contact_delta_sum[index] += contact.correction[index];
            result.contact_impulse_sum[index] += contact.impulse[index];
            result.contact_mask[index] |= contact.mask[index];
        }
        result.lower_contacts += contact.lower_hits;
        const DensityResult candidate_density = candidate_graph_density(
            profile, input.id, y, ghosts, grid, round.work);
        const DensityResult independent = independent_density(
            profile, y, ghosts, grid, true);
        round.work.independent_density_candidates += independent.candidates;
        round.post_graph_root = candidate_density.graph_root;
        round.post_density_root = id_scalar_root(
            "nextengine.nonlocal.ncgp13.post-density.v1", input.id,
            candidate_density.value);
        round.independent_density_root = id_scalar_root(
            "nextengine.nonlocal.ncgp13.independent-density.v1", input.id,
            independent.value);
        round.work.hash_derivations += 2U;
        last_density = independent.value;
        std::vector<long double> density_difference(count, 0.0L);
        for (std::size_t index = 0U; index < count; ++index) {
            density_difference[index] = candidate_density.value[index]
                - independent.value[index];
        }
        round.density_correspondence = vector_norm(density_difference)
            / std::max(vector_norm(independent.value), 1.0e-30L);
        const auto strains = strain_metrics(last_density, profile.rest_density);
        round.maximum_strain = strains.first;
        round.rms_strain = strains.second;
        static_cast<void>(inset_oracle(profile, y, round.penetration));
        round.state_root = id_vec_root(
            "nextengine.nonlocal.ncgp13.round-state.v1", input.id, y);
        ++round.work.hash_derivations;
        record_round(round, 12U);
        result.positive_multipliers += round.positive;
        const std::size_t minimum_rounds = phase_a ? 2U : 1U;
        if (candidate_density.valid && independent.valid
            && round.pressure_balance <= kBalanceLimit
            && round.maximum_strain <= kDensityMaximumLimit
            && round.rms_strain <= kDensityRmsLimit
            && round.penetration == 0.0L
            && round.density_correspondence <= 2.0e-12L
            && result.rounds.size() >= minimum_rounds) {
            result.accepted = true;
            break;
        }
    }

    if (!result.accepted && result.failure.empty()) {
        result.failure = "PROJECTION_ROUND_BUDGET_EXHAUSTED";
    }
    if (result.accepted) {
        result.state.position = y;
        for (std::size_t index = 0U; index < count; ++index) {
            result.state.velocity[index] =
                (y[index] - input.position[index]) * (1.0L / dt);
        }
        const DensityResult candidate_density = candidate_graph_density(
            profile, input.id, y, ghosts, grid, result.work);
        ++expected_step_hashes;
        const DensityResult independent = independent_density(
            profile, y, ghosts, grid, true);
        result.work.independent_density_candidates += independent.candidates;
        const auto strains = strain_metrics(independent.value,
            profile.rest_density);
        result.maximum_strain = strains.first;
        result.rms_strain = strains.second;
        static_cast<void>(inset_oracle(profile, y, result.penetration));
        std::vector<long double> density_error(count, 0.0L);
        for (std::size_t index = 0U; index < count; ++index) {
            density_error[index] = candidate_density.value[index]
                - independent.value[index];
        }
        result.independent_correspondence = vector_norm(density_error)
            / std::max(vector_norm(independent.value), 1.0e-30L);
        std::vector<Vec3l> inertial(count);
        std::vector<Vec3l> contact_term(count);
        std::vector<Vec3l> residual(count);
        for (std::size_t index = 0U; index < count; ++index) {
            inertial[index] = (y[index] - input.position[index]
                    - displacement[index]) * (profile.mass / dt2);
            contact_term[index] = result.contact_delta_sum[index]
                * (-profile.mass / dt2);
            residual[index] = inertial[index] + result.pressure_jt_sum[index]
                + contact_term[index];
        }
        result.balance = dense_norm(residual) / std::max({dense_norm(inertial),
            dense_norm(result.pressure_jt_sum), dense_norm(contact_term),
            1.0e-30L * static_cast<long double>(count)});
        long double bottom_sum = 0.0L;
        long double top_sum = 0.0L;
        std::size_t bottom_count = 0U;
        std::size_t top_count = 0U;
        const std::size_t nx = phase_a ? 8U : 4U;
        const std::size_t ny = phase_a ? 8U : 4U;
        const std::size_t nz = 8U;
        for (std::size_t index = 0U; index < count; ++index) {
            const std::size_t logical = (input.id[index] - 1000U) / 17U;
            const std::size_t layer = logical / (nx * ny);
            if (layer == 0U) {
                bottom_sum += result.lambda_sum[index];
                ++bottom_count;
            }
            if (layer + 1U == nz) {
                top_sum += result.lambda_sum[index];
                ++top_count;
            }
        }
        result.bottom_pressure_proxy = static_cast<long double>(
                profile.rest_density) / profile.mass
            * bottom_sum / bottom_count;
        result.top_pressure_proxy = static_cast<long double>(
                profile.rest_density) / profile.mass
            * top_sum / top_count;
        result.component_count = components(profile, y, result.work);
        result.identity_mass_exact = result.state.id.size() == input.id.size()
            && result.state.id == input.id
            && result.state.reference.size() == input.reference.size()
            && std::equal(result.state.reference.begin(),
                result.state.reference.end(), input.reference.begin(),
                [](Vec3l lhs, Vec3l rhs) {
                    return lhs.x == rhs.x && lhs.y == rhs.y && lhs.z == rhs.z;
                })
            && static_cast<long double>(result.state.id.size()) * profile.mass
                == static_cast<long double>(input.id.size()) * profile.mass;
        result.physical_pass = result.maximum_strain <= kDensityMaximumLimit
            && result.rms_strain <= kDensityRmsLimit
            && result.penetration == 0.0L
            && result.independent_correspondence <= 2.0e-12L
            && result.balance <= kBalanceLimit
            && result.identity_mass_exact
            && (!phase_a || (result.positive_multipliers > 0U
                    && result.lower_contacts > 0U
                    && result.bottom_pressure_proxy
                        > result.top_pressure_proxy))
            && result.component_count == 1U;
        if (!result.physical_pass) result.failure = "FINAL_STEP_GATE_FAILED";
    }
    seal_step(result, input.id, expected_step_hashes);
    return result;
}

struct Controls {
    bool legacy = false;
    bool contact_oracle = false;
    bool exact_radius = false;
    bool ghost_derivative = false;
    bool nonfinite = false;
    bool duplicate = false;
    bool transactional = false;
    bool no_pressure = false;
    bool negated_pressure = false;
    bool stale = false;
    bool input_order_loss = false;
    bool mutations = false;
    std::string contact_oracle_root;
    std::string exact_radius_root;
    std::string ghost_derivative_root;
    std::string admission_root;
    std::string transactional_root;
    std::string conditional_root;
    std::string result_root;
    struct Receipt {
        std::string number;
        std::string name;
        std::string status;
        bool pass = false;
        std::uint64_t expected_hash_derivations = 0U;
        bool hash_accounting_exact = false;
        std::string before_state_root;
        std::string after_state_root;
        std::string typed_outcome;
        std::vector<std::string> evidence_roots;
        std::vector<long double> scalar_evidence;
        std::vector<std::uint64_t> integer_evidence;
        Work work;
        std::string work_root;
        std::string result_root;
    };
    std::vector<Receipt> receipts;
    std::vector<Receipt> subreceipts;
    Work work;
};

// `hash_derivations` counts logical content roots owned by a work receipt.
// It deliberately excludes that receipt's own work_root/result_root and all
// enclosing aggregate/final roots, avoiding recursive accounting. Recomputed
// verification roots and mutation evidence roots are charged to the control
// that requested them; a control never charges the immutable baseline.
Controls::Receipt seal_control_receipt(std::string number, std::string name,
    std::string status, Work work, std::uint64_t expected_hash_derivations,
    std::vector<std::string> evidence_roots,
    std::string before_state_root = {}, std::string after_state_root = {},
    std::string typed_outcome = {},
    std::vector<long double> scalar_evidence = {},
    std::vector<std::uint64_t> integer_evidence = {}) {
    Controls::Receipt receipt;
    receipt.number = std::move(number);
    receipt.name = std::move(name);
    receipt.status = std::move(status);
    receipt.pass = receipt.status == "PASS";
    receipt.expected_hash_derivations = expected_hash_derivations;
    receipt.hash_accounting_exact =
        work.hash_derivations == expected_hash_derivations;
    if (!receipt.hash_accounting_exact) {
        throw std::logic_error("control hash accounting mismatch");
    }
    receipt.before_state_root = std::move(before_state_root);
    receipt.after_state_root = std::move(after_state_root);
    receipt.typed_outcome = std::move(typed_outcome);
    receipt.evidence_roots = std::move(evidence_roots);
    receipt.scalar_evidence = std::move(scalar_evidence);
    receipt.integer_evidence = std::move(integer_evidence);
    receipt.work = work;
    receipt.work_root = work_root(receipt.work);
    std::string bytes;
    append_string(bytes, "nextengine.nonlocal.ncgp13.control-receipt.v1");
    append_string(bytes, receipt.number);
    append_string(bytes, receipt.name);
    append_string(bytes, receipt.status);
    append_u64(bytes, receipt.pass ? 1U : 0U);
    append_u64(bytes, receipt.expected_hash_derivations);
    append_u64(bytes, receipt.hash_accounting_exact ? 1U : 0U);
    append_string(bytes, receipt.before_state_root);
    append_string(bytes, receipt.after_state_root);
    append_string(bytes, receipt.typed_outcome);
    append_u64(bytes, receipt.evidence_roots.size());
    for (const std::string& root : receipt.evidence_roots) {
        append_string(bytes, root);
    }
    append_u64(bytes, receipt.scalar_evidence.size());
    for (const long double value : receipt.scalar_evidence) {
        append_wide(bytes, value);
    }
    append_u64(bytes, receipt.integer_evidence.size());
    for (const std::uint64_t value : receipt.integer_evidence) {
        append_u64(bytes, value);
    }
    append_string(bytes, receipt.work_root);
    receipt.result_root = digest(std::move(bytes));
    return receipt;
}

Controls::Receipt skipped_control(std::string number, std::string name) {
    return seal_control_receipt(std::move(number), std::move(name),
        "NOT_RUN_BY_PRECEDENCE", {}, 0U, {});
}

void add_control(Controls& controls, Controls::Receipt receipt) {
    add_work(controls.work, receipt.work);
    controls.receipts.push_back(std::move(receipt));
}

bool receipts_exact(const Controls& controls) {
    const auto exact = [](const Controls::Receipt& receipt) {
        return receipt.hash_accounting_exact;
    };
    return std::all_of(controls.receipts.begin(), controls.receipts.end(), exact)
        && std::all_of(controls.subreceipts.begin(),
            controls.subreceipts.end(), exact);
}

std::string input_order_root(const std::vector<NonlocalGpuSample>& samples) {
    std::string bytes;
    append_string(bytes, "nextengine.nonlocal.ncgp13.input-order.v1");
    append_u64(bytes, samples.size());
    for (const auto& sample : samples) {
        append_u32(bytes, sample.sample_id);
        append_f32(bytes, static_cast<float>(sample.current.x));
        append_f32(bytes, static_cast<float>(sample.current.y));
        append_f32(bytes, static_cast<float>(sample.current.z));
    }
    return digest(std::move(bytes));
}

bool exact_radius_control(NonlocalGpuProfile profile,
    std::vector<std::string>& evidence_roots,
    std::vector<std::uint64_t>& integer_evidence, Work& work) {
    profile.horizon = 0.125;
    const std::vector<std::uint32_t> ids{9000001U, 9000018U};
    const std::vector<Vec3l> positions{{0.0L, 0.0L, 0.0L},
        {0.125L, 0.0L, 0.0L}};
    const std::vector<NonlocalGpuGhost> ghosts;
    const GhostGrid grid = make_ghost_grid(profile, ghosts);
    Work inclusive_work;
    Work strict_work;
    const Graph inclusive = build_graph(profile, ids, positions, ghosts, grid,
        false, inclusive_work);
    const Graph strict = build_graph(profile, ids, positions, ghosts, grid,
        true, strict_work);
    add_work(work, inclusive_work);
    add_work(work, strict_work);
    std::vector<NonlocalGpuSample> samples{
        {ids[0], {0.0, 0.0, 0.0}, {0.0, 0.0, 0.0}, {}},
        {ids[1], {0.125, 0.0, 0.0}, {0.125, 0.0, 0.0}, {}}};
    const std::string profile_root = ncgp13_profile_root(profile);
    const std::string input = ncgp13_control_input_root(
        profile_root, samples, ghosts);
    work.hash_derivations += 2U;
    const std::uint64_t inclusive_pairs = std::accumulate(
        inclusive.row.begin(), inclusive.row.end(), std::uint64_t{0U},
        [](std::uint64_t sum, const auto& row) { return sum + row.size(); });
    const std::uint64_t strict_pairs = std::accumulate(
        strict.row.begin(), strict.row.end(), std::uint64_t{0U},
        [](std::uint64_t sum, const auto& row) { return sum + row.size(); });
    const bool pass = inclusive.valid && strict.valid
        && inclusive_pairs == strict_pairs + 2U
        && inclusive.root != strict.root;
    evidence_roots = {profile_root, input, inclusive.root, strict.root};
    integer_evidence = {inclusive_pairs, strict_pairs};
    return pass;
}

bool ghost_derivative_control(const NonlocalGpuProfile& profile,
    const State& state, const std::vector<NonlocalGpuGhost>& ghosts,
    const GhostGrid& grid, std::vector<std::string>& evidence_roots,
    std::vector<long double>& scalar_evidence, Work& work) {
    const Assembly included = assemble(profile, state.id, state.position,
        ghosts, grid, true, nullptr);
    const Assembly omitted = assemble(profile, state.id, state.position,
        ghosts, grid, false, nullptr);
    add_work(work, included.work);
    add_work(work, omitted.work);
    const long double included_error = directional_check(profile,
        state.position, ghosts, grid, included, work);
    const long double omitted_error = directional_check(profile,
        state.position, ghosts, grid, omitted, work);
    const bool pass = included.valid && omitted.valid
        && included_error <= kJacobianLimit
        && omitted_error > kJacobianLimit;
    evidence_roots = {included.input_root, included.graph_root,
        included.density_root, included.jacobian_root, included.matrix_root,
        omitted.input_root, omitted.graph_root, omitted.density_root,
        omitted.jacobian_root, omitted.matrix_root};
    scalar_evidence = {included_error, omitted_error};
    return pass;
}

Controls unconditional_controls(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& raw,
    const std::vector<NonlocalGpuGhost>& ghosts, const State& state,
    const GhostGrid& grid, const LegacyWitness& legacy) {
    Controls controls;
    controls.legacy = legacy.pass;
    add_control(controls, seal_control_receipt("1", "retained-pressure-witness",
        legacy.pass ? "PASS" : "FAIL", legacy.work, 7U,
        {legacy.multiplier_root, legacy.trial_root, legacy.result_root}));

    Work contact_work;
    controls.contact_oracle = contact_oracle_control(profile,
        controls.contact_oracle_root, contact_work);
    add_control(controls, seal_control_receipt("2", "contact-oracle",
        controls.contact_oracle ? "PASS" : "FAIL", contact_work, 1U,
        {controls.contact_oracle_root}));

    Work radius_work;
    std::vector<std::string> radius_evidence;
    std::vector<std::uint64_t> radius_integers;
    controls.exact_radius = exact_radius_control(
        profile, radius_evidence, radius_integers, radius_work);
    Controls::Receipt radius_receipt = seal_control_receipt(
        "3", "exact-radius-graph",
        controls.exact_radius ? "PASS" : "FAIL", radius_work, 4U,
        std::move(radius_evidence), {}, {}, "INCLUSIVE_HAS_TWO_CROSS_PAIRS",
        {}, std::move(radius_integers));
    controls.exact_radius_root = radius_receipt.result_root;
    add_control(controls, std::move(radius_receipt));

    Work ghost_work;
    std::vector<std::string> ghost_evidence;
    std::vector<long double> ghost_scalars;
    controls.ghost_derivative = ghost_derivative_control(profile, state,
        ghosts, grid, ghost_evidence, ghost_scalars, ghost_work);
    Controls::Receipt ghost_receipt = seal_control_receipt(
        "4", "ghost-derivative",
        controls.ghost_derivative ? "PASS" : "FAIL", ghost_work, 10U,
        std::move(ghost_evidence), {}, {}, "GHOST_DERIVATIVE_DISCRIMINATED",
        std::move(ghost_scalars));
    controls.ghost_derivative_root = ghost_receipt.result_root;
    add_control(controls, std::move(ghost_receipt));

    auto nonfinite = raw;
    nonfinite.front().current.x = std::numeric_limits<double>::quiet_NaN();
    auto duplicate = raw;
    duplicate.back().sample_id = duplicate.front().sample_id;

    Work admission_work;
    const std::string admission_prior = state_root(
        "nextengine.nonlocal.ncgp13.admission-state.v1", state);
    ++admission_work.hash_derivations;

    Work nonfinite_work;
    const Admission nonfinite_failure = admit(profile, nonfinite, ghosts);
    const std::string nonfinite_after = state_root(
        "nextengine.nonlocal.ncgp13.admission-state.v1", state);
    ++nonfinite_work.hash_derivations;
    controls.nonfinite = nonfinite_failure == Admission::Nonfinite
        && admission_prior == nonfinite_after;
    Controls::Receipt nonfinite_receipt = seal_control_receipt(
        "5a", "nonfinite-admission",
        controls.nonfinite ? "PASS" : "FAIL", nonfinite_work, 1U,
        {admission_prior, nonfinite_after}, admission_prior, nonfinite_after,
        "NONFINITE_REJECTED", {},
        {static_cast<std::uint64_t>(nonfinite_failure)});
    controls.subreceipts.push_back(nonfinite_receipt);

    Work duplicate_work;
    const Admission duplicate_failure = admit(profile, duplicate, ghosts);
    const std::string duplicate_after = state_root(
        "nextengine.nonlocal.ncgp13.admission-state.v1", state);
    ++duplicate_work.hash_derivations;
    controls.duplicate = duplicate_failure == Admission::DuplicateId
        && admission_prior == duplicate_after;
    Controls::Receipt duplicate_receipt = seal_control_receipt(
        "5b", "duplicate-id-admission",
        controls.duplicate ? "PASS" : "FAIL", duplicate_work, 1U,
        {admission_prior, duplicate_after}, admission_prior, duplicate_after,
        "DUPLICATE_ID_REJECTED", {},
        {static_cast<std::uint64_t>(duplicate_failure)});
    controls.subreceipts.push_back(duplicate_receipt);

    add_work(admission_work, nonfinite_receipt.work);
    add_work(admission_work, duplicate_receipt.work);
    Controls::Receipt admission_receipt = seal_control_receipt(
        "5", "admission-failures",
        controls.nonfinite && controls.duplicate ? "PASS" : "FAIL",
        admission_work, 3U,
        {nonfinite_receipt.work_root, nonfinite_receipt.result_root,
            duplicate_receipt.work_root, duplicate_receipt.result_root,
            admission_prior, nonfinite_after, duplicate_after},
        admission_prior, duplicate_after, "BOTH_INPUTS_REJECTED");
    controls.admission_root = admission_receipt.result_root;
    add_control(controls, std::move(admission_receipt));

    Work transaction_work;
    const std::string transaction_before = state_root(
        "nextengine.nonlocal.ncgp13.transaction-state.v1", state);
    ++transaction_work.hash_derivations;
    const StepResult injected = run_step(profile, state, ghosts, grid, true, true);
    add_work(transaction_work, injected.work);
    // The injected child step envelope is evidence owned by control 6.
    ++transaction_work.hash_derivations;
    const std::string transaction_after = state_root(
        "nextengine.nonlocal.ncgp13.transaction-state.v1", injected.state);
    ++transaction_work.hash_derivations;
    controls.transactional = injected.apparatus_valid
        && injected.failure == "INJECTED_AFTER_PRIVATE_ROUND"
        && injected.hash_accounting_exact
        && transaction_before == transaction_after
        && !injected.state_root.empty() && !injected.velocity_root.empty()
        && !injected.injected_private_multiplier_root.empty()
        && !injected.injected_private_contact_root.empty();
    Controls::Receipt transaction_receipt = seal_control_receipt(
        "6", "transactional-injection",
        controls.transactional ? "PASS" : "FAIL", transaction_work, 16U,
        {injected.result_root, injected.state_root, injected.velocity_root,
            injected.injected_private_multiplier_root,
            injected.injected_private_contact_root},
        transaction_before, transaction_after,
        "INJECTED_AFTER_PRIVATE_ROUND");
    controls.transactional_root = transaction_receipt.result_root;
    add_control(controls, std::move(transaction_receipt));
    return controls;
}

bool unconditional_pass(const Controls& controls) {
    return controls.legacy && controls.contact_oracle && controls.exact_radius
        && controls.ghost_derivative && controls.nonfinite
        && controls.duplicate && controls.transactional
        && receipts_exact(controls);
}

struct StaleAttempt {
    bool rejected = false;
    std::string status;
    std::string current_input_root;
    Work attempted_work;
};

StaleAttempt attempt_assembly_reuse(std::string_view frozen_input_root,
    const std::vector<std::uint32_t>& ids,
    const std::vector<Vec3l>& current) {
    StaleAttempt result;
    ++result.attempted_work.hash_derivations;
    result.current_input_root = position_state_root(ids, current);
    if (frozen_input_root != result.current_input_root) {
        result.rejected = true;
        result.status = "STALE_ASSEMBLY_REJECTED";
    } else {
        result.status = "ASSEMBLY_INPUT_MATCHED";
    }
    return result;
}

void conditional_controls(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& raw,
    const std::vector<NonlocalGpuSample>& permuted_raw,
    const std::vector<NonlocalGpuGhost>& ghosts, const GhostGrid& grid,
    const State& state, const StepResult& baseline,
    const StepResult& permuted, std::string_view canonical_input_root,
    std::string_view canonical_permuted_input_root, Controls& controls) {
    const long double dt = profile.dt;
    const long double dt2 = dt * dt;
    const Vec3l gravity = widen(profile.gravity);

    Work no_pressure_work;
    std::vector<Vec3l> predictor(state.id.size());
    for (std::size_t index = 0U; index < state.id.size(); ++index) {
        predictor[index] = state.position[index] + state.velocity[index] * dt
            + gravity * dt2;
    }
    const ContactBatch contact = contact_batch(
        profile, state.id, state.position, predictor);
    add_work(no_pressure_work, contact.work);
    const DensityResult no_pressure_density = independent_density(
        profile, contact.endpoint, ghosts, grid, true);
    no_pressure_work.independent_density_candidates +=
        no_pressure_density.candidates;
    const auto no_pressure_strain = strain_metrics(no_pressure_density.value,
        profile.rest_density);
    controls.no_pressure = contact.valid && no_pressure_density.valid
        && (no_pressure_strain.first > kDensityMaximumLimit
            || no_pressure_strain.second > kDensityRmsLimit);
    const std::string no_pressure_density_root = id_scalar_root(
        "nextengine.nonlocal.ncgp13.no-pressure-density.v1", state.id,
        no_pressure_density.value);
    ++no_pressure_work.hash_derivations;
    add_control(controls, seal_control_receipt("7", "no-pressure",
        controls.no_pressure ? "PASS" : "FAIL", no_pressure_work, 2U,
        {contact.root, no_pressure_density_root}, {}, {},
        "CONTACT_ONLY_DENSITY_GATE", {no_pressure_strain.first,
            no_pressure_strain.second}));

    Work negated_work;
    const ContactBatch negated_predictor = contact_batch(
        profile, state.id, state.position, predictor);
    add_work(negated_work, negated_predictor.work);
    const Assembly assembly = assemble(profile, state.id,
        negated_predictor.endpoint,
        ghosts, grid, true, nullptr);
    add_work(negated_work, assembly.work);
    Solve solve = solve_qp(assembly, negated_work);
    const std::string negated_multiplier_root = id_scalar_root(
        "nextengine.nonlocal.ncgp13.negated-multiplier.v1", state.id,
        solve.lambda);
    ++negated_work.hash_derivations;
    std::vector<Vec3l> wrong = negated_predictor.endpoint;
    if (solve.valid && solve.converged) {
        const auto jt = jt_times(assembly, solve.lambda);
        for (std::size_t index = 0U; index < state.id.size(); ++index) {
            wrong[index] += Vec3l{jt[3U * index], jt[3U * index + 1U],
                jt[3U * index + 2U]} * (dt2 / profile.mass);
        }
    }
    const ContactBatch wrong_contact = contact_batch(
        profile, state.id, negated_predictor.endpoint, wrong);
    add_work(negated_work, wrong_contact.work);
    const DensityResult wrong_density = independent_density(
        profile, wrong_contact.endpoint, ghosts, grid, true);
    negated_work.independent_density_candidates += wrong_density.candidates;
    const auto wrong_strain = strain_metrics(wrong_density.value,
        profile.rest_density);
    const std::string wrong_density_root = id_scalar_root(
        "nextengine.nonlocal.ncgp13.negated-density.v1", state.id,
        wrong_density.value);
    ++negated_work.hash_derivations;
    long double wrong_penetration = 0.0L;
    static_cast<void>(inset_oracle(profile, wrong_contact.endpoint,
        wrong_penetration));
    controls.negated_pressure = assembly.valid && solve.valid
        && solve.converged && wrong_contact.valid && wrong_density.valid
        && (wrong_strain.first > kDensityMaximumLimit
            || wrong_strain.second > kDensityRmsLimit
            || wrong_penetration > 0.0L);
    add_control(controls, seal_control_receipt("8", "negated-pressure",
        controls.negated_pressure ? "PASS" : "FAIL", negated_work, 9U,
        {negated_predictor.root, assembly.input_root,
            assembly.graph_root, assembly.density_root, assembly.jacobian_root,
            assembly.matrix_root, negated_multiplier_root,
            wrong_contact.root, wrong_density_root}, {}, {},
        "NEGATED_PRESSURE_GATE", {wrong_strain.first, wrong_strain.second,
            wrong_penetration}));

    if (baseline.rounds.empty()) {
        Work stale_missing_work;
        const std::string current_input_root = position_state_root(
            state.id, baseline.state.position);
        ++stale_missing_work.hash_derivations;
        add_control(controls, seal_control_receipt("9", "stale-relinearization",
            "FAIL", stale_missing_work, 1U, {current_input_root}, {}, {},
            "NO_BASELINE_ASSEMBLY"));
        controls.stale = false;
    } else {
        const StaleAttempt stale = attempt_assembly_reuse(
            baseline.rounds.front().assembly_input_root, state.id,
            baseline.state.position);
        controls.stale = stale.rejected
            && stale.status == "STALE_ASSEMBLY_REJECTED"
            && stale.attempted_work.matrix_products == 0U
            && stale.attempted_work.qp_sweeps == 0U
            && stale.attempted_work.qp_updates == 0U
            && stale.attempted_work.gradient_recomputations == 0U;
        add_control(controls, seal_control_receipt("9",
            "stale-relinearization", controls.stale ? "PASS" : "FAIL",
            stale.attempted_work, 1U,
            {baseline.rounds.front().assembly_input_root,
                stale.current_input_root}, {}, {}, stale.status));
    }

    Work permutation_work;
    const std::string raw_order = input_order_root(raw);
    const std::string permuted_order = input_order_root(permuted_raw);
    permutation_work.hash_derivations += 2U;
    controls.input_order_loss = canonical_input_root
            == canonical_permuted_input_root
        && raw_order != permuted_order && baseline.result_root == permuted.result_root;
    add_control(controls, seal_control_receipt("10", "permutation-identity",
        controls.input_order_loss ? "PASS" : "FAIL", permutation_work, 2U,
        {std::string(canonical_input_root),
            std::string(canonical_permuted_input_root), raw_order,
            permuted_order, baseline.result_root, permuted.result_root},
        {}, {}, "CANONICAL_EQUAL_RAW_ORDER_DISTINCT"));

    Work mutation_work;
    StepResult pressure_changed = baseline;
    const auto positive = std::find_if(pressure_changed.lambda_sum.begin(),
        pressure_changed.lambda_sum.end(),
        [](long double value) { return value > 0.0L; });
    bool pressure_mutation = positive != pressure_changed.lambda_sum.end();
    if (pressure_mutation) {
        *positive = std::nextafter(static_cast<double>(*positive),
            std::numeric_limits<double>::infinity());
        pressure_changed.multiplier_root = id_scalar_root(
            "nextengine.nonlocal.ncgp13.step-multiplier.v1", state.id,
            pressure_changed.lambda_sum);
        ++mutation_work.hash_derivations;
        pressure_changed.result_root = step_root(pressure_changed);
        ++mutation_work.hash_derivations;
        pressure_mutation = pressure_changed.multiplier_root
                != baseline.multiplier_root
            && pressure_changed.result_root != baseline.result_root;
    }
    StepResult position_changed = baseline;
    position_changed.state.position.front().x = std::nextafter(
        static_cast<double>(position_changed.state.position.front().x),
        std::numeric_limits<double>::infinity());
    position_changed.state_root = state_root(
        "nextengine.nonlocal.ncgp13.accepted-state.v1", position_changed.state);
    ++mutation_work.hash_derivations;
    position_changed.result_root = step_root(position_changed);
    ++mutation_work.hash_derivations;
    const bool position_mutation = position_changed.state_root
            != baseline.state_root
        && position_changed.result_root != baseline.result_root;
    StepResult mask_changed = baseline;
    mask_changed.contact_mask.front() ^= 1U << 4U;
    mask_changed.contact_mask_root = id_mask_root(
        "nextengine.nonlocal.ncgp13.step-contact-mask.v1", state.id,
        mask_changed.contact_mask);
    ++mutation_work.hash_derivations;
    mask_changed.result_root = step_root(mask_changed);
    ++mutation_work.hash_derivations;
    const bool mask_mutation = mask_changed.contact_mask_root
            != baseline.contact_mask_root
        && mask_changed.result_root != baseline.result_root;
    const std::string baseline_work_root = work_root(baseline.work);
    std::vector<std::string> work_mutation_roots;
    bool work_mutation = true;
    auto mutate_work = [&](auto member) {
        StepResult changed = baseline;
        Work expected_changed_work = baseline.work;
        ++(changed.work.*member);
        ++(expected_changed_work.*member);
        const std::string changed_work_root = work_root(changed.work);
        ++mutation_work.hash_derivations;
        changed.result_root = step_root(changed);
        ++mutation_work.hash_derivations;
        work_mutation = work_mutation
            && same_work(changed.work, expected_changed_work)
            && changed_work_root != baseline_work_root
            && changed.result_root != baseline.result_root;
        work_mutation_roots.push_back(changed_work_root);
        work_mutation_roots.push_back(changed.result_root);
    };
    mutate_work(&Work::projection_rounds);
    mutate_work(&Work::analytic_jv_multiply_adds);
    mutate_work(&Work::topology_distance_tests);
    mutate_work(&Work::topology_discoveries);
    mutate_work(&Work::hash_derivations);
    controls.mutations = pressure_mutation && position_mutation
        && mask_mutation && work_mutation;
    std::vector<std::string> mutation_roots{pressure_changed.multiplier_root,
        pressure_changed.result_root, position_changed.state_root,
        position_changed.result_root, mask_changed.contact_mask_root,
        mask_changed.result_root};
    mutation_roots.insert(mutation_roots.end(), work_mutation_roots.begin(),
        work_mutation_roots.end());
    add_control(controls, seal_control_receipt("11", "typed-root-mutations",
        controls.mutations ? "PASS" : "FAIL", mutation_work, 16U,
        std::move(mutation_roots), {}, {}, "ALL_TYPED_MUTATIONS_CHANGED"));
}

bool conditional_pass(const Controls& controls) {
    return controls.no_pressure && controls.negated_pressure && controls.stale
        && controls.input_order_loss && controls.mutations
        && receipts_exact(controls);
}

void add_skipped_conditional_controls(Controls& controls) {
    add_control(controls, skipped_control("7", "no-pressure"));
    add_control(controls, skipped_control("8", "negated-pressure"));
    add_control(controls, skipped_control("9", "stale-relinearization"));
    add_control(controls, skipped_control("10", "permutation-identity"));
    add_control(controls, skipped_control("11", "typed-root-mutations"));
}

struct TrajectoryResult {
    bool apparatus_valid = true;
    bool physical_pass = false;
    bool failing_trial_published = false;
    bool failing_trial_observables_published = false;
    bool failing_trial_gate_evaluated = false;
    bool failing_trial_gate_pass = false;
    std::size_t accepted_steps = 0U;
    std::size_t failure_step = 0U;
    std::size_t failing_trial_step = 0U;
    std::string failure;
    std::string failing_trial_outcome;
    State final_state;
    std::vector<std::string> step_roots;
    std::vector<StepResult> steps;
    std::vector<bool> trial_committed;
    long double maximum_position_rms = 0.0L;
    long double maximum_position = 0.0L;
    long double maximum_velocity_rms = 0.0L;
    long double maximum_speed = 0.0L;
    long double energy_excess = 0.0L;
    long double momentum_residual = 0.0L;
    std::size_t maximum_components = 0U;
    long double failing_trial_position_rms = 0.0L;
    long double failing_trial_position_maximum = 0.0L;
    long double failing_trial_velocity_rms = 0.0L;
    long double failing_trial_maximum_speed = 0.0L;
    long double failing_trial_energy_excess = 0.0L;
    long double failing_trial_momentum_residual = 0.0L;
    std::size_t failing_trial_components = 0U;
    std::size_t pressure_occurrences = 0U;
    std::size_t lower_contact_occurrences = 0U;
    std::string state_root;
    std::string failing_trial_state_root;
    std::string failing_trial_work_root;
    std::string failing_trial_result_root;
    std::string trajectory_root;
    std::uint64_t expected_hash_derivations = 0U;
    bool hash_accounting_exact = false;
    Work work;
};

Vec3l momentum(const State& state, long double mass) {
    Vec3l result;
    for (const Vec3l velocity : state.velocity) result += velocity * mass;
    return result;
}

long double energy(const State& state, const NonlocalGpuProfile& profile) {
    const Vec3l gravity = widen(profile.gravity);
    long double result = 0.0L;
    for (std::size_t index = 0U; index < state.id.size(); ++index) {
        result += 0.5L * profile.mass
                * dot(state.velocity[index], state.velocity[index])
            - profile.mass * dot(gravity, state.position[index]);
    }
    return result;
}

std::string trajectory_root(const TrajectoryResult& trajectory) {
    std::string bytes;
    append_string(bytes, "nextengine.nonlocal.ncgp13.trajectory.v1");
    append_u64(bytes, trajectory.apparatus_valid ? 1U : 0U);
    append_u64(bytes, trajectory.physical_pass ? 1U : 0U);
    append_u64(bytes, trajectory.failing_trial_published ? 1U : 0U);
    append_u64(bytes,
        trajectory.failing_trial_observables_published ? 1U : 0U);
    append_u64(bytes, trajectory.failing_trial_gate_evaluated ? 1U : 0U);
    append_u64(bytes, trajectory.failing_trial_gate_pass ? 1U : 0U);
    append_u64(bytes, trajectory.accepted_steps);
    append_u64(bytes, trajectory.failure_step);
    append_u64(bytes, trajectory.failing_trial_step);
    append_string(bytes, trajectory.failure);
    append_string(bytes, trajectory.failing_trial_outcome);
    append_u64(bytes, trajectory.maximum_components);
    append_u64(bytes, trajectory.pressure_occurrences);
    append_u64(bytes, trajectory.lower_contact_occurrences);
    for (const long double value : std::array<long double, 6>{
             trajectory.maximum_position_rms, trajectory.maximum_position,
             trajectory.maximum_velocity_rms, trajectory.maximum_speed,
             trajectory.energy_excess, trajectory.momentum_residual}) {
        append_wide(bytes, value);
    }
    for (const long double value : std::array<long double, 6>{
             trajectory.failing_trial_position_rms,
             trajectory.failing_trial_position_maximum,
             trajectory.failing_trial_velocity_rms,
             trajectory.failing_trial_maximum_speed,
             trajectory.failing_trial_energy_excess,
             trajectory.failing_trial_momentum_residual}) {
        append_wide(bytes, value);
    }
    append_u64(bytes, trajectory.failing_trial_components);
    append_u64(bytes, trajectory.step_roots.size());
    for (std::size_t index = 0U; index < trajectory.step_roots.size(); ++index) {
        append_string(bytes, trajectory.step_roots[index]);
        append_u64(bytes, trajectory.trial_committed[index] ? 1U : 0U);
    }
    append_string(bytes, trajectory.state_root);
    append_string(bytes, trajectory.failing_trial_state_root);
    append_string(bytes, trajectory.failing_trial_work_root);
    append_string(bytes, trajectory.failing_trial_result_root);
    append_u64(bytes, trajectory.expected_hash_derivations);
    append_u64(bytes, trajectory.hash_accounting_exact ? 1U : 0U);
    append_string(bytes, work_root(trajectory.work));
    return digest(std::move(bytes));
}

TrajectoryResult run_trajectory(const NonlocalGpuProfile& profile,
    const State& initial, const std::vector<NonlocalGpuGhost>& ghosts,
    const GhostGrid& grid) {
    TrajectoryResult result;
    State state = initial;
    const Vec3l initial_momentum = momentum(initial, profile.mass);
    const long double initial_energy = energy(initial, profile);
    const long double total_mass = static_cast<long double>(profile.mass)
        * initial.id.size();
    const Vec3l gravity = widen(profile.gravity);
    Vec3l gravity_impulse;
    Vec3l pressure_impulse;
    Vec3l contact_impulse;
    std::uint64_t expected_hash_derivations = 0U;
    auto publish_failing_trial = [&](const StepResult& trial) {
        result.failing_trial_published = true;
        result.failing_trial_step = result.steps.size();
        result.failing_trial_outcome = result.failure;
        result.failing_trial_state_root = trial.state_root;
        result.failing_trial_work_root = work_root(trial.work);
        result.failing_trial_result_root = trial.result_root;
    };
    for (std::size_t step_index = 0U; step_index < kPhaseBSteps;
         ++step_index) {
        StepResult trial = run_step(profile, state, ghosts, grid, false);
        add_work(result.work, trial.work);
        expected_hash_derivations += trial.expected_hash_derivations;
        result.step_roots.push_back(trial.result_root);
        result.steps.push_back(trial);
        result.trial_committed.push_back(false);
        if (!trial.hash_accounting_exact || !trial.apparatus_valid) {
            result.apparatus_valid = false;
            result.failure = trial.hash_accounting_exact
                ? trial.failure : "STEP_HASH_ACCOUNTING_INVALID";
            result.failure_step = step_index + 1U;
            publish_failing_trial(trial);
            break;
        }
        if (!trial.accepted || !trial.physical_pass) {
            result.failure = trial.failure;
            result.failure_step = step_index + 1U;
            publish_failing_trial(trial);
            break;
        }
        long double position_sum = 0.0L;
        long double velocity_sum = 0.0L;
        long double position_maximum = 0.0L;
        long double speed_maximum = 0.0L;
        for (std::size_t index = 0U; index < trial.state.id.size(); ++index) {
            const long double displacement = norm(
                trial.state.position[index] - initial.position[index]);
            const long double speed = norm(trial.state.velocity[index]);
            position_sum += displacement * displacement;
            velocity_sum += speed * speed;
            position_maximum = std::max(position_maximum, displacement);
            speed_maximum = std::max(speed_maximum, speed);
            if (trial.state.id[index] != initial.id[index]
                || trial.state.reference[index].x
                    != initial.reference[index].x
                || trial.state.reference[index].y
                    != initial.reference[index].y
                || trial.state.reference[index].z
                    != initial.reference[index].z) {
                result.apparatus_valid = false;
                result.failure = "IDENTITY_OR_REFERENCE_CHANGED";
                result.failure_step = step_index + 1U;
                break;
            }
        }
        if (!result.apparatus_valid) {
            publish_failing_trial(trial);
            break;
        }
        const long double trial_position_rms = std::sqrt(
            position_sum / trial.state.id.size());
        const long double trial_velocity_rms = std::sqrt(
            velocity_sum / trial.state.id.size());
        const long double maximum_position_rms = std::max(
            result.maximum_position_rms, trial_position_rms);
        const long double maximum_position = std::max(
            result.maximum_position, position_maximum);
        const long double maximum_velocity_rms = std::max(
            result.maximum_velocity_rms, trial_velocity_rms);
        const long double maximum_speed = std::max(
            result.maximum_speed, speed_maximum);
        const long double current_energy = energy(trial.state, profile);
        const long double denominator = std::max({std::abs(initial_energy),
            total_mass * norm(gravity) * profile.spacing, 1.0e-30L});
        const long double energy_excess = std::max(result.energy_excess,
            std::max(current_energy - initial_energy, 0.0L) / denominator);
        const Vec3l trial_gravity_impulse = gravity_impulse
            + gravity * (total_mass * profile.dt);
        Vec3l step_pressure;
        Vec3l step_contact;
        for (std::size_t index = 0U; index < trial.state.id.size(); ++index) {
            step_pressure += trial.pressure_jt_sum[index] * (-profile.dt);
            step_contact += trial.contact_impulse_sum[index];
        }
        const Vec3l trial_pressure_impulse = pressure_impulse + step_pressure;
        const Vec3l trial_contact_impulse = contact_impulse + step_contact;
        const Vec3l momentum_change = momentum(trial.state, profile.mass)
            - initial_momentum;
        const Vec3l momentum_error = momentum_change - trial_gravity_impulse
            - trial_pressure_impulse - trial_contact_impulse;
        const long double momentum_residual = norm(momentum_error)
            / std::max({norm(momentum_change), norm(trial_gravity_impulse),
                norm(trial_pressure_impulse), norm(trial_contact_impulse),
                1.0e-30L});
        const std::size_t maximum_components = std::max(
            result.maximum_components, trial.component_count);

        const bool trajectory_gates =
            maximum_position_rms <= kPositionRmsLimit
            && maximum_position <= kPositionMaximumLimit
            && maximum_velocity_rms <= kVelocityRmsLimit
            && maximum_speed <= kVelocityMaximumLimit
            && energy_excess <= kEnergyLimit
            && momentum_residual <= kMomentumLimit
            && maximum_components == 1U;
        if (!trajectory_gates) {
            result.failure = "TRAJECTORY_GATE_FAILED";
            result.failure_step = step_index + 1U;
            publish_failing_trial(trial);
            result.failing_trial_observables_published = true;
            result.failing_trial_gate_evaluated = true;
            result.failing_trial_gate_pass = false;
            result.failing_trial_position_rms = trial_position_rms;
            result.failing_trial_position_maximum = position_maximum;
            result.failing_trial_velocity_rms = trial_velocity_rms;
            result.failing_trial_maximum_speed = speed_maximum;
            result.failing_trial_energy_excess = energy_excess;
            result.failing_trial_momentum_residual = momentum_residual;
            result.failing_trial_components = trial.component_count;
            break;
        }
        result.maximum_position_rms = maximum_position_rms;
        result.maximum_position = maximum_position;
        result.maximum_velocity_rms = maximum_velocity_rms;
        result.maximum_speed = maximum_speed;
        result.energy_excess = energy_excess;
        result.momentum_residual = momentum_residual;
        result.maximum_components = maximum_components;
        state = trial.state;
        gravity_impulse = trial_gravity_impulse;
        pressure_impulse = trial_pressure_impulse;
        contact_impulse = trial_contact_impulse;
        ++result.accepted_steps;
        result.trial_committed.back() = true;
        if (trial.positive_multipliers > 0U) {
            ++result.pressure_occurrences;
        }
        if (trial.lower_contacts > 0U) {
            ++result.lower_contact_occurrences;
        }
    }
    result.final_state = state;
    result.physical_pass = result.apparatus_valid
        && result.accepted_steps == kPhaseBSteps
        && result.failure.empty() && result.pressure_occurrences > 0U
        && result.lower_contact_occurrences > 0U;
    if (result.accepted_steps == kPhaseBSteps && result.failure.empty()
        && !result.physical_pass) {
        result.failure = "TRAJECTORY_PRESSURE_OR_CONTACT_WITNESS_MISSING";
        result.failure_step = kPhaseBSteps;
    }
    result.state_root = state_root(
        "nextengine.nonlocal.ncgp13.trajectory-state.v1", result.final_state);
    ++result.work.hash_derivations;
    result.expected_hash_derivations = expected_hash_derivations + 1U;
    result.hash_accounting_exact = result.work.hash_derivations
        == result.expected_hash_derivations;
    if (!result.hash_accounting_exact) {
        throw std::logic_error("trajectory hash accounting mismatch");
    }
    result.trajectory_root = trajectory_root(result);
    return result;
}

void seal_controls(Controls& controls) {
    constexpr std::array<std::string_view, 11> numbers{
        "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11"};
    constexpr std::array<std::uint64_t, 11> executed_hashes{
        7U, 1U, 4U, 10U, 3U, 16U, 2U, 9U, 1U, 2U, 16U};
    if (controls.receipts.size() != numbers.size()) {
        throw std::logic_error("control receipt count mismatch");
    }
    Work summed;
    for (std::size_t index = 0U; index < controls.receipts.size(); ++index) {
        const Controls::Receipt& receipt = controls.receipts[index];
        if (receipt.number != numbers[index]) {
            throw std::logic_error("control receipt order mismatch");
        }
        const std::uint64_t expected =
            receipt.status == "NOT_RUN_BY_PRECEDENCE"
            ? 0U : executed_hashes[index];
        if (receipt.expected_hash_derivations != expected
            || !receipt.hash_accounting_exact) {
            throw std::logic_error("control receipt expectation mismatch");
        }
        add_work(summed, receipt.work);
    }
    if (!same_work(summed, controls.work)) {
        throw std::logic_error("control aggregate work is not componentwise exact");
    }
    const bool conditional_skipped = std::all_of(
        controls.receipts.begin() + 6, controls.receipts.end(),
        [](const Controls::Receipt& receipt) {
            return receipt.status == "NOT_RUN_BY_PRECEDENCE";
        });
    const bool conditional_executed = std::none_of(
        controls.receipts.begin() + 6, controls.receipts.end(),
        [](const Controls::Receipt& receipt) {
            return receipt.status == "NOT_RUN_BY_PRECEDENCE";
        });
    if (!conditional_skipped && !conditional_executed) {
        throw std::logic_error("mixed conditional-control precedence");
    }
    if (controls.work.hash_derivations
        != (conditional_skipped ? 41U : 71U)) {
        throw std::logic_error("control aggregate hash total mismatch");
    }
    if (controls.subreceipts.size() != 2U
        || controls.subreceipts[0].number != "5a"
        || controls.subreceipts[1].number != "5b"
        || controls.subreceipts[0].expected_hash_derivations != 1U
        || controls.subreceipts[1].expected_hash_derivations != 1U
        || controls.subreceipts[0].work.hash_derivations != 1U
        || controls.subreceipts[1].work.hash_derivations != 1U
        || controls.subreceipts[0].status == "NOT_RUN_BY_PRECEDENCE"
        || controls.subreceipts[1].status == "NOT_RUN_BY_PRECEDENCE"
        || !receipts_exact(controls)) {
        throw std::logic_error("control subreceipt closure mismatch");
    }
    std::string conditional_bytes;
    append_string(conditional_bytes,
        "nextengine.nonlocal.ncgp13.conditional-controls.v1");
    for (std::size_t index = 6U; index < controls.receipts.size(); ++index) {
        append_string(conditional_bytes, controls.receipts[index].result_root);
    }
    controls.conditional_root = digest(std::move(conditional_bytes));
    std::string bytes;
    append_string(bytes, "nextengine.nonlocal.ncgp13.controls.v1");
    for (const bool pass : std::array<bool, 12>{controls.legacy,
             controls.contact_oracle, controls.exact_radius,
             controls.ghost_derivative, controls.nonfinite,
             controls.duplicate, controls.transactional,
             controls.no_pressure, controls.negated_pressure,
             controls.stale, controls.input_order_loss,
             controls.mutations}) append_u64(bytes, pass ? 1U : 0U);
    append_string(bytes, controls.contact_oracle_root);
    append_string(bytes, controls.exact_radius_root);
    append_string(bytes, controls.ghost_derivative_root);
    append_string(bytes, controls.admission_root);
    append_string(bytes, controls.transactional_root);
    append_string(bytes, controls.conditional_root);
    append_u64(bytes, controls.receipts.size());
    for (const Controls::Receipt& receipt : controls.receipts) {
        append_string(bytes, receipt.number);
        append_string(bytes, receipt.status);
        append_string(bytes, receipt.work_root);
        append_string(bytes, receipt.result_root);
    }
    append_u64(bytes, controls.subreceipts.size());
    for (const Controls::Receipt& receipt : controls.subreceipts) {
        append_string(bytes, receipt.number);
        append_string(bytes, receipt.status);
        append_string(bytes, receipt.work_root);
        append_string(bytes, receipt.result_root);
    }
    append_string(bytes, work_root(controls.work));
    controls.result_root = digest(std::move(bytes));
}

std::string final_result_root(std::string_view status,
    std::string_view binary, std::string_view profile_root,
    std::string_view phase_a_input,
    std::string_view phase_b_input, const LegacyWitness& legacy,
    const Controls& controls, const StepResult* phase_a,
    const StepResult* phase_a_permuted,
    const TrajectoryResult* trajectory,
    const TrajectoryResult* trajectory_permuted) {
    std::string bytes;
    append_string(bytes, "nextengine.nonlocal.ncgp13.result.v1");
    append_string(bytes, NCGP13_CONTRACT_ROOT);
    append_string(bytes, NCGP13_SOURCE_ROOT);
    append_string(bytes, NCGP13_SOURCE_COMMIT);
    append_string(bytes, NCGP13_SOURCE_TREE);
    append_string(bytes, NCGP13_COMPILER_FLAGS);
    append_string(bytes, binary);
    append_string(bytes, profile_root);
    append_string(bytes, phase_a_input);
    append_string(bytes, phase_b_input);
    append_string(bytes, status);
    append_string(bytes, legacy.result_root);
    append_string(bytes, controls.result_root);
    append_string(bytes, phase_a == nullptr ? "NOT_RUN_BY_PRECEDENCE"
                                            : phase_a->result_root);
    append_string(bytes, phase_a_permuted == nullptr
            ? "NOT_RUN_BY_PRECEDENCE" : phase_a_permuted->result_root);
    append_string(bytes, trajectory == nullptr ? "NOT_RUN_BY_PRECEDENCE"
                                               : trajectory->trajectory_root);
    append_string(bytes, trajectory_permuted == nullptr
            ? "NOT_RUN_BY_PRECEDENCE" : trajectory_permuted->trajectory_root);
    return digest(std::move(bytes));
}

void emit_work(std::ostream& output, const Work& work) {
    output << "{\"graph_builds\":" << work.graph_builds
           << ",\"graph_candidates\":" << work.graph_candidates
           << ",\"accepted_pairs\":" << work.accepted_pairs
           << ",\"density_pairs\":" << work.density_pairs
           << ",\"derivative_pairs\":" << work.derivative_pairs
           << ",\"matrix_products\":" << work.matrix_products
           << ",\"qp_sweeps\":" << work.qp_sweeps
           << ",\"qp_updates\":" << work.qp_updates
           << ",\"gradient_recomputations\":"
           << work.gradient_recomputations
           << ",\"finite_difference_candidates\":"
           << work.finite_difference_candidates
           << ",\"independent_density_candidates\":"
           << work.independent_density_candidates
           << ",\"plane_tests\":" << work.plane_tests
           << ",\"plane_hits\":" << work.plane_hits
           << ",\"contact_projections\":" << work.contact_projections
           << ",\"projection_rounds\":" << work.projection_rounds
           << ",\"analytic_jv_multiply_adds\":"
           << work.analytic_jv_multiply_adds
           << ",\"topology_distance_tests\":"
           << work.topology_distance_tests
           << ",\"topology_discoveries\":"
           << work.topology_discoveries
           << ",\"hash_derivations\":" << work.hash_derivations
           << ",\"penalty_work\":" << work.penalty_work << "}";
}

void emit_control_receipt(std::ostream& output,
    const Controls::Receipt& receipt) {
    output << "{\"number\":\"" << receipt.number
           << "\",\"name\":\"" << receipt.name
           << "\",\"status\":\"" << receipt.status
           << "\",\"pass\":" << (receipt.pass ? "true" : "false")
           << ",\"typed_outcome\":\"" << receipt.typed_outcome
           << "\",\"expected_hash_derivations\":"
           << receipt.expected_hash_derivations
           << ",\"actual_hash_derivations\":"
           << receipt.work.hash_derivations
           << ",\"hash_accounting_exact\":"
           << (receipt.hash_accounting_exact ? "true" : "false")
           << ",\"before_state_root\":\"" << receipt.before_state_root
           << "\",\"after_state_root\":\"" << receipt.after_state_root
           << "\",\"evidence_roots\":[";
    for (std::size_t index = 0U; index < receipt.evidence_roots.size();
         ++index) {
        if (index != 0U) output << ',';
        output << '\"' << receipt.evidence_roots[index] << '\"';
    }
    output << "],\"scalar_evidence\":[";
    for (std::size_t index = 0U; index < receipt.scalar_evidence.size();
         ++index) {
        if (index != 0U) output << ',';
        output << static_cast<double>(receipt.scalar_evidence[index]);
    }
    output << "],\"integer_evidence\":[";
    for (std::size_t index = 0U; index < receipt.integer_evidence.size();
         ++index) {
        if (index != 0U) output << ',';
        output << receipt.integer_evidence[index];
    }
    output << "],\"work_root\":\"" << receipt.work_root
           << "\",\"result_root\":\"" << receipt.result_root
           << "\",\"work\":";
    emit_work(output, receipt.work);
    output << "}";
}

void emit_legacy(std::ostream& output, const LegacyWitness& legacy) {
    output << "{\"pass\":" << (legacy.pass ? "true" : "false")
           << ",\"expected_hash_derivations\":7"
           << ",\"hash_accounting_exact\":"
           << (legacy.work.hash_derivations == 7U ? "true" : "false")
           << ",\"sweeps\":" << legacy.sweeps
           << ",\"updates\":" << legacy.updates
           << ",\"positive_multipliers\":" << legacy.positive
           << ",\"jacobian_relative_l2\":"
           << static_cast<double>(legacy.jacobian)
           << ",\"symmetry_relative_l2\":"
           << static_cast<double>(legacy.symmetry)
           << ",\"primal\":" << static_cast<double>(legacy.primal)
           << ",\"projected_kkt\":" << static_cast<double>(legacy.kkt)
           << ",\"complementarity_j\":"
           << static_cast<double>(legacy.complementarity)
           << ",\"linearized_relative_l2\":"
           << static_cast<double>(legacy.linearized)
           << ",\"maximum_positive_strain\":"
           << static_cast<double>(legacy.maximum_strain)
           << ",\"rms_positive_strain\":"
           << static_cast<double>(legacy.rms_strain)
           << ",\"stationarity\":"
           << static_cast<double>(legacy.stationarity)
           << ",\"penetration_m\":"
           << static_cast<double>(legacy.penetration)
           << ",\"multiplier_root\":\"" << legacy.multiplier_root
           << "\",\"trial_root\":\"" << legacy.trial_root
           << "\",\"work_root\":\"" << work_root(legacy.work)
           << "\",\"result_root\":\"" << legacy.result_root
           << "\",\"work\":";
    emit_work(output, legacy.work);
    output << "}";
}

void emit_step(std::ostream& output, const StepResult& step) {
    output << "{\"apparatus_valid\":"
           << (step.apparatus_valid ? "true" : "false")
           << ",\"physical_pass\":"
           << (step.physical_pass ? "true" : "false")
           << ",\"accepted\":" << (step.accepted ? "true" : "false")
           << ",\"identity_mass_exact\":"
           << (step.identity_mass_exact ? "true" : "false")
           << ",\"expected_hash_derivations\":"
           << step.expected_hash_derivations
           << ",\"hash_accounting_exact\":"
           << (step.hash_accounting_exact ? "true" : "false")
           << ",\"failure\":\"" << step.failure
           << "\",\"round_count\":" << step.rounds.size()
           << ",\"positive_multipliers\":" << step.positive_multipliers
           << ",\"lower_contacts\":" << step.lower_contacts
           << ",\"maximum_positive_strain\":"
           << static_cast<double>(step.maximum_strain)
           << ",\"rms_positive_strain\":"
           << static_cast<double>(step.rms_strain)
           << ",\"penetration_m\":" << static_cast<double>(step.penetration)
           << ",\"independent_density_relative_l2\":"
           << static_cast<double>(step.independent_correspondence)
           << ",\"balance_residual\":" << static_cast<double>(step.balance)
           << ",\"bottom_pressure_proxy_pa\":"
           << static_cast<double>(step.bottom_pressure_proxy)
           << ",\"top_pressure_proxy_pa\":"
           << static_cast<double>(step.top_pressure_proxy)
           << ",\"components\":" << step.component_count
           << ",\"multiplier_root\":\"" << step.multiplier_root
           << "\",\"predictor_contact_root\":\""
           << step.predictor_contact_root
           << "\",\"injected_private_multiplier_root\":\""
           << step.injected_private_multiplier_root
           << "\",\"injected_private_contact_root\":\""
           << step.injected_private_contact_root
           << "\",\"contact_mask_root\":\"" << step.contact_mask_root
           << "\",\"contact_impulse_root\":\""
           << step.contact_impulse_root
           << "\",\"state_root\":\"" << step.state_root
           << "\",\"velocity_root\":\"" << step.velocity_root
           << "\",\"work_root\":\"" << work_root(step.work)
           << "\",\"result_root\":\"" << step.result_root
           << "\",\"rounds\":[";
    for (std::size_t index = 0U; index < step.rounds.size(); ++index) {
        if (index != 0U) output << ',';
        const RoundSummary& round = step.rounds[index];
        output << "{\"index\":" << index
               << ",\"assembly_valid\":"
               << (round.assembly_valid ? "true" : "false")
               << ",\"solve_valid\":"
               << (round.solve_valid ? "true" : "false")
               << ",\"converged\":" << (round.converged ? "true" : "false")
               << ",\"stale\":" << (round.stale ? "true" : "false")
               << ",\"expected_hash_derivations\":"
               << round.expected_hash_derivations
               << ",\"hash_accounting_exact\":"
               << (round.hash_accounting_exact ? "true" : "false")
               << ",\"sweeps\":" << round.sweeps
               << ",\"updates\":" << round.updates
               << ",\"positive\":" << round.positive
               << ",\"jacobian_relative_l2\":"
               << static_cast<double>(round.jacobian)
               << ",\"symmetry_relative_l2\":"
               << static_cast<double>(round.symmetry)
               << ",\"primal\":" << static_cast<double>(round.primal)
               << ",\"projected_kkt\":" << static_cast<double>(round.kkt)
               << ",\"complementarity_j\":"
               << static_cast<double>(round.complementarity)
               << ",\"pressure_balance\":"
               << static_cast<double>(round.pressure_balance)
               << ",\"maximum_positive_strain\":"
               << static_cast<double>(round.maximum_strain)
               << ",\"rms_positive_strain\":"
               << static_cast<double>(round.rms_strain)
               << ",\"penetration_m\":"
               << static_cast<double>(round.penetration)
               << ",\"density_correspondence\":"
               << static_cast<double>(round.density_correspondence)
               << ",\"assembly_input_root\":\""
               << round.assembly_input_root
               << "\",\"graph_root\":\"" << round.graph_root
               << "\",\"density_root\":\"" << round.density_root
               << "\",\"jacobian_root\":\"" << round.jacobian_root
               << "\",\"matrix_root\":\"" << round.matrix_root
               << "\",\"post_graph_root\":\"" << round.post_graph_root
               << "\",\"post_density_root\":\"" << round.post_density_root
               << "\",\"independent_density_root\":\""
               << round.independent_density_root
               << "\",\"multiplier_root\":\"" << round.multiplier_root
               << "\",\"contact_root\":\"" << round.contact_root
               << "\",\"state_root\":\"" << round.state_root
               << "\",\"work_root\":\"" << work_root(round.work)
               << "\",\"result_root\":\"" << round.result_root
               << "\",\"work\":";
        emit_work(output, round.work);
        output << "}";
    }
    output << "],\"work\":";
    emit_work(output, step.work);
    output << "}";
}

void emit_trajectory(std::ostream& output,
    const TrajectoryResult& trajectory) {
    output << "{\"apparatus_valid\":"
           << (trajectory.apparatus_valid ? "true" : "false")
           << ",\"physical_pass\":"
           << (trajectory.physical_pass ? "true" : "false")
           << ",\"failing_trial_published\":"
           << (trajectory.failing_trial_published ? "true" : "false")
           << ",\"failing_trial_observables_published\":"
           << (trajectory.failing_trial_observables_published
                   ? "true" : "false")
           << ",\"failing_trial_gate_evaluated\":"
           << (trajectory.failing_trial_gate_evaluated ? "true" : "false")
           << ",\"failing_trial_gate_pass\":"
           << (trajectory.failing_trial_gate_pass ? "true" : "false")
           << ",\"accepted_steps\":" << trajectory.accepted_steps
           << ",\"failure_step\":" << trajectory.failure_step
           << ",\"failing_trial_step\":" << trajectory.failing_trial_step
           << ",\"failure_state_published\":false"
           << ",\"failing_trial_committed\":false"
           << ",\"failing_trial_outcome\":\""
           << trajectory.failing_trial_outcome << "\""
           << ",\"expected_hash_derivations\":"
           << trajectory.expected_hash_derivations
           << ",\"hash_accounting_exact\":"
           << (trajectory.hash_accounting_exact ? "true" : "false")
           << ",\"failure\":\"" << trajectory.failure
           << "\",\"maximum_position_rmse_m\":"
           << static_cast<double>(trajectory.maximum_position_rms)
           << ",\"maximum_position_m\":"
           << static_cast<double>(trajectory.maximum_position)
           << ",\"maximum_velocity_rms_mps\":"
           << static_cast<double>(trajectory.maximum_velocity_rms)
           << ",\"maximum_speed_mps\":"
           << static_cast<double>(trajectory.maximum_speed)
           << ",\"energy_positive_excess\":"
           << static_cast<double>(trajectory.energy_excess)
           << ",\"momentum_residual\":"
           << static_cast<double>(trajectory.momentum_residual)
           << ",\"maximum_components\":" << trajectory.maximum_components
           << ",\"failing_trial_position_rmse_m\":"
           << static_cast<double>(trajectory.failing_trial_position_rms)
           << ",\"failing_trial_position_maximum_m\":"
           << static_cast<double>(
                  trajectory.failing_trial_position_maximum)
           << ",\"failing_trial_velocity_rms_mps\":"
           << static_cast<double>(trajectory.failing_trial_velocity_rms)
           << ",\"failing_trial_maximum_speed_mps\":"
           << static_cast<double>(trajectory.failing_trial_maximum_speed)
           << ",\"failing_trial_energy_positive_excess\":"
           << static_cast<double>(trajectory.failing_trial_energy_excess)
           << ",\"failing_trial_momentum_residual\":"
           << static_cast<double>(
                  trajectory.failing_trial_momentum_residual)
           << ",\"failing_trial_components\":"
           << trajectory.failing_trial_components
           << ",\"pressure_occurrences\":"
           << trajectory.pressure_occurrences
           << ",\"lower_contact_occurrences\":"
           << trajectory.lower_contact_occurrences
           << ",\"state_root\":\"" << trajectory.state_root
           << "\",\"failing_trial_state_root\":\""
           << trajectory.failing_trial_state_root
           << "\",\"failing_trial_work_root\":\""
           << trajectory.failing_trial_work_root
           << "\",\"failing_trial_result_root\":\""
           << trajectory.failing_trial_result_root
           << "\",\"work_root\":\"" << work_root(trajectory.work)
           << "\",\"trajectory_root\":\"" << trajectory.trajectory_root
           << "\",\"step_roots\":[";
    for (std::size_t index = 0U; index < trajectory.step_roots.size(); ++index) {
        if (index != 0U) output << ',';
        output << '\"' << trajectory.step_roots[index] << '\"';
    }
    output << "],\"trial_committed\":[";
    for (std::size_t index = 0U; index < trajectory.trial_committed.size();
         ++index) {
        if (index != 0U) output << ',';
        output << (trajectory.trial_committed[index] ? "true" : "false");
    }
    output << "],\"steps\":[";
    for (std::size_t index = 0U; index < trajectory.steps.size(); ++index) {
        if (index != 0U) output << ',';
        emit_step(output, trajectory.steps[index]);
    }
    output << "],\"work\":";
    emit_work(output, trajectory.work);
    output << "}";
}

int run() {
    NonlocalGpuProfile profile = nonlocal_water_corrected_profile();
    profile.lambda = 0.0;
    profile.mu = 0.0;
    profile.gamma = 0.0;
    const std::string profile_root = ncgp13_profile_root(profile);
    const auto phase_a_raw = make_lattice_state(profile, 8U, 8U, 8U,
        false, false);
    const auto phase_a_permuted_raw = make_lattice_state(profile, 8U, 8U,
        8U, true, false);
    const auto phase_b_raw = make_lattice_state(profile, 4U, 4U, 8U,
        false, false);
    const auto phase_b_permuted_raw = make_lattice_state(profile, 4U, 4U,
        8U, true, false);
    auto ghosts = make_basin_ghosts(profile);
    std::sort(ghosts.begin(), ghosts.end(), [](const auto& lhs, const auto& rhs) {
        return lhs.sample_id < rhs.sample_id;
    });
    const std::string phase_a_input = local_input_root(
        profile, phase_a_raw, ghosts);
    const std::string phase_a_permuted_input = local_input_root(
        profile, phase_a_permuted_raw, ghosts);
    const std::string phase_b_input = local_input_root(
        profile, phase_b_raw, ghosts);
    const std::string phase_b_permuted_input = local_input_root(
        profile, phase_b_permuted_raw, ghosts);
    const bool identity = admit(profile, phase_a_raw, ghosts) == Admission::Ok
        && admit(profile, phase_a_permuted_raw, ghosts) == Admission::Ok
        && admit(profile, phase_b_raw, ghosts) == Admission::Ok
        && admit(profile, phase_b_permuted_raw, ghosts) == Admission::Ok
        && phase_a_input == kPhaseAInput
        && phase_a_permuted_input == kPhaseAInput
        && phase_b_input == kPhaseBInput
        && phase_b_permuted_input == kPhaseBInput;
    const State phase_a_state = working_state(phase_a_raw);
    const State phase_a_permuted_state = working_state(phase_a_permuted_raw);
    const State phase_b_state = working_state(phase_b_raw);
    const State phase_b_permuted_state = working_state(phase_b_permuted_raw);
    const GhostGrid grid = make_ghost_grid(profile, ghosts);
    const LegacyWitness legacy = legacy_witness(profile, phase_a_state,
        ghosts, grid, phase_a_input);
    Controls controls = unconditional_controls(profile, phase_a_raw, ghosts,
        phase_a_state, grid, legacy);

    std::string status;
    StepResult phase_a;
    StepResult phase_a_permuted;
    TrajectoryResult trajectory;
    TrajectoryResult trajectory_permuted;
    bool phase_a_ran = false;
    bool trajectory_ran = false;
    if (!identity || !unconditional_pass(controls)) {
        status = "APPARATUS_INCONCLUSIVE";
        add_skipped_conditional_controls(controls);
    } else {
        phase_a = run_step(profile, phase_a_state, ghosts, grid, true);
        phase_a_permuted = run_step(profile, phase_a_permuted_state, ghosts,
            grid, true);
        phase_a_ran = true;
        const bool phase_a_identity = phase_a.result_root
                == phase_a_permuted.result_root
            && phase_a.state_root == phase_a_permuted.state_root
            && phase_a.velocity_root == phase_a_permuted.velocity_root
            && phase_a.multiplier_root == phase_a_permuted.multiplier_root
            && phase_a.contact_mask_root
                == phase_a_permuted.contact_mask_root
            && phase_a.contact_impulse_root
                == phase_a_permuted.contact_impulse_root
            && phase_a.hash_accounting_exact
            && phase_a_permuted.hash_accounting_exact
            && work_root(phase_a.work) == work_root(phase_a_permuted.work);
        if (!phase_a.apparatus_valid || !phase_a_permuted.apparatus_valid
            || !phase_a_identity) {
            status = "APPARATUS_INCONCLUSIVE";
            add_skipped_conditional_controls(controls);
        } else if (!phase_a.physical_pass || !phase_a_permuted.physical_pass) {
            status = "PRESSURE_CONTACT_STEP_REFUTED";
            add_skipped_conditional_controls(controls);
        } else {
            conditional_controls(profile, phase_a_raw, phase_a_permuted_raw,
                ghosts, grid, phase_a_state, phase_a, phase_a_permuted,
                phase_a_input, phase_a_permuted_input, controls);
            if (!conditional_pass(controls)) {
                status = "APPARATUS_INCONCLUSIVE";
            } else {
                trajectory = run_trajectory(profile, phase_b_state, ghosts,
                    grid);
                trajectory_permuted = run_trajectory(profile,
                    phase_b_permuted_state, ghosts, grid);
                trajectory_ran = true;
                const bool trajectory_identity = trajectory.trajectory_root
                        == trajectory_permuted.trajectory_root
                    && trajectory.state_root == trajectory_permuted.state_root
                    && trajectory.step_roots == trajectory_permuted.step_roots
                    && trajectory.trial_committed
                        == trajectory_permuted.trial_committed
                    && trajectory.hash_accounting_exact
                    && trajectory_permuted.hash_accounting_exact
                    && work_root(trajectory.work)
                        == work_root(trajectory_permuted.work);
                if (!trajectory.apparatus_valid
                    || !trajectory_permuted.apparatus_valid
                    || !trajectory_identity) {
                    status = "APPARATUS_INCONCLUSIVE";
                } else if (!trajectory.physical_pass
                    || !trajectory_permuted.physical_pass) {
                    status = "PRESSURE_CONTACT_TRAJECTORY_REFUTED";
                } else {
                    status = "PRESSURE_CONTACT_TINY_SUPPORTED";
                }
            }
        }
    }
    seal_controls(controls);
    const std::string binary = file_root("/proc/self/exe");
    if (binary.empty()) throw std::runtime_error("binary identity failed");
    const std::string result_root = final_result_root(status, binary,
        profile_root, phase_a_input, phase_b_input, legacy, controls,
        phase_a_ran ? &phase_a : nullptr,
        phase_a_ran ? &phase_a_permuted : nullptr,
        trajectory_ran ? &trajectory : nullptr,
        trajectory_ran ? &trajectory_permuted : nullptr);
    std::cout << std::setprecision(17)
              << "{\"schema\":\"nextengine.nonlocal.ncgp13_pressure_contact.v1\""
              << ",\"status\":\"" << status << "\""
              << ",\"contract_root\":\"" << NCGP13_CONTRACT_ROOT << "\""
              << ",\"source_root\":\"" << NCGP13_SOURCE_ROOT << "\""
              << ",\"source_commit\":\"" << NCGP13_SOURCE_COMMIT << "\""
              << ",\"source_tree\":\"" << NCGP13_SOURCE_TREE << "\""
              << ",\"compiler_flags\":\"" << NCGP13_COMPILER_FLAGS << "\""
              << ",\"binary_root\":\"" << binary << "\""
              << ",\"profile_root\":\"" << profile_root << "\""
              << ",\"profile\":{"
              << "\"id\":\"" << profile.id << "\",\"dt\":" << profile.dt
              << ",\"spacing\":" << profile.spacing
              << ",\"horizon\":" << profile.horizon
              << ",\"mass\":" << profile.mass
              << ",\"rho0\":" << profile.rest_density
              << ",\"kernel_scale\":" << profile.kernel_scale
              << ",\"kappa_retained\":" << profile.kappa
              << ",\"lambda\":" << profile.lambda
              << ",\"mu\":" << profile.mu
              << ",\"gamma\":" << profile.gamma
              << ",\"gravity\":[" << profile.gravity.x << ','
              << profile.gravity.y << ',' << profile.gravity.z << ']'
              << ",\"basin_extent\":[" << profile.basin_extent.x << ','
              << profile.basin_extent.y << ',' << profile.basin_extent.z
              << ']'
              << ",\"ghost_layers\":" << profile.ghost_layers
              << ",\"maximum_dynamic_samples\":"
              << profile.maximum_dynamic_samples
              << ",\"maximum_neighbors\":" << profile.maximum_neighbors
              << "}"
              << ",\"phase_a_input_root\":\"" << phase_a_input << "\""
              << ",\"phase_a_permuted_input_root\":\""
              << phase_a_permuted_input << "\""
              << ",\"phase_b_input_root\":\"" << phase_b_input << "\""
              << ",\"phase_b_permuted_input_root\":\""
              << phase_b_permuted_input << "\""
              << ",\"dynamic_phase_a\":" << phase_a_state.id.size()
              << ",\"dynamic_phase_b\":" << phase_b_state.id.size()
              << ",\"ghost_samples\":" << ghosts.size()
              << ",\"identity_admission\":"
              << (identity ? "true" : "false")
              << ",\"legacy_witness\":";
    emit_legacy(std::cout, legacy);
    std::cout << ",\"controls\":{\"legacy\":"
              << (controls.legacy ? "true" : "false")
              << ",\"contact_oracle\":"
              << (controls.contact_oracle ? "true" : "false")
              << ",\"exact_radius\":"
              << (controls.exact_radius ? "true" : "false")
              << ",\"ghost_derivative\":"
              << (controls.ghost_derivative ? "true" : "false")
              << ",\"nonfinite\":" << (controls.nonfinite ? "true" : "false")
              << ",\"duplicate_id\":" << (controls.duplicate ? "true" : "false")
              << ",\"transactional\":"
              << (controls.transactional ? "true" : "false")
              << ",\"no_pressure\":"
              << (controls.no_pressure ? "true" : "false")
              << ",\"negated_pressure\":"
              << (controls.negated_pressure ? "true" : "false")
              << ",\"stale_relinearization\":"
              << (controls.stale ? "true" : "false")
              << ",\"input_order_loss\":"
              << (controls.input_order_loss ? "true" : "false")
              << ",\"typed_mutations\":"
              << (controls.mutations ? "true" : "false")
              << ",\"contact_oracle_root\":\""
              << controls.contact_oracle_root
              << "\",\"exact_radius_root\":\""
              << controls.exact_radius_root
              << "\",\"ghost_derivative_root\":\""
              << controls.ghost_derivative_root
              << "\",\"admission_root\":\"" << controls.admission_root
              << "\",\"transactional_root\":\""
              << controls.transactional_root
              << "\",\"conditional_root\":\""
              << controls.conditional_root
              << "\",\"work_root\":\"" << work_root(controls.work)
              << "\",\"result_root\":\"" << controls.result_root
              << "\",\"receipts\":[";
    for (std::size_t index = 0U; index < controls.receipts.size(); ++index) {
        if (index != 0U) std::cout << ',';
        emit_control_receipt(std::cout, controls.receipts[index]);
    }
    std::cout << "],\"subreceipts\":[";
    for (std::size_t index = 0U; index < controls.subreceipts.size(); ++index) {
        if (index != 0U) std::cout << ',';
        emit_control_receipt(std::cout, controls.subreceipts[index]);
    }
    std::cout << "],\"work\":";
    emit_work(std::cout, controls.work);
    std::cout << "},\"phase_a\":";
    if (phase_a_ran) emit_step(std::cout, phase_a);
    else std::cout << "\"NOT_RUN_BY_PRECEDENCE\"";
    std::cout << ",\"phase_a_permuted\":";
    if (phase_a_ran) emit_step(std::cout, phase_a_permuted);
    else std::cout << "\"NOT_RUN_BY_PRECEDENCE\"";
    std::cout << ",\"phase_b\":";
    if (trajectory_ran) emit_trajectory(std::cout, trajectory);
    else std::cout << "\"NOT_RUN_BY_PRECEDENCE\"";
    std::cout << ",\"phase_b_permuted\":";
    if (trajectory_ran) emit_trajectory(std::cout, trajectory_permuted);
    else std::cout << "\"NOT_RUN_BY_PRECEDENCE\"";
    std::cout << ",\"result_root\":\"" << result_root << "\"}\n";
    return status == "APPARATUS_INCONCLUSIVE" ? 2 : 0;
}

} // namespace

int main(int argc, char** argv) {
    try {
        if (argc != 2
            || std::string_view(argv[1]) != "--pressure-contact-trajectory") {
            std::cerr << "usage: nonlocal-corrected-cpu-pressure-contact "
                         "--pressure-contact-trajectory\n";
            return 64;
        }
        return run();
    } catch (const std::exception& error) {
        std::cerr << "NCGP13 apparatus failure: " << error.what() << '\n';
        return 3;
    }
}
