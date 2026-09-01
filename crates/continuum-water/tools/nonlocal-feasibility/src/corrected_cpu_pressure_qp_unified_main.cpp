#include "corrected_cpu_unified_constrained.hpp"
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
#include <optional>
#include <set>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <tuple>
#include <utility>
#include <vector>

#if __cplusplus != 201703L
#error "NCGP16 requires exact ISO C++17 mode"
#endif

namespace ncgp16 {

using namespace nextengine::nonlocal::gpu_full_step;
using nextengine::nonlocal::ncgp15::EnergyGradient15;
using nextengine::nonlocal::ncgp15::MultiplierMode15;
using nextengine::nonlocal::ncgp15::SolverOptions15;
using nextengine::nonlocal::ncgp15::State15;
using nextengine::nonlocal::ncgp15::TermMask15;
using nextengine::nonlocal::ncgp15::Vec3l;
using nextengine::nonlocal::ncgp15::Work15;

#ifndef NCGP16_CONTRACT_ROOT
#define NCGP16_CONTRACT_ROOT "unconfigured"
#endif
#ifndef NCGP16_CONTRACT_SHA
#define NCGP16_CONTRACT_SHA "unconfigured"
#endif
#ifndef NCGP16_SOURCE_ROOT
#define NCGP16_SOURCE_ROOT "unconfigured"
#endif
#ifndef NCGP16_SOURCE_COMMIT
#define NCGP16_SOURCE_COMMIT "unconfigured"
#endif
#ifndef NCGP16_SOURCE_TREE
#define NCGP16_SOURCE_TREE "unconfigured"
#endif
#ifndef NCGP16_COMPILER_FAMILY
#define NCGP16_COMPILER_FAMILY "unconfigured"
#endif
#ifndef NCGP16_COMPILER_VERSION
#define NCGP16_COMPILER_VERSION "unconfigured"
#endif
#ifndef NCGP16_COMPILER_FLAGS
#define NCGP16_COMPILER_FLAGS "unconfigured"
#endif

constexpr std::string_view kSchema =
    "nextengine.nonlocal.ncgp16.result.v1";
constexpr std::string_view kInvocation =
    "nonlocal-corrected-cpu-pressure-qp-unified --pressure-qp-unified";
constexpr std::string_view kParent14Fixture =
    "6dbaddf563b825e28e37aee58606f25cf50b17479cc3260e919103be79f7c2b2";
constexpr std::string_view kParent14Lane =
    "8d13d0eefb7e27acafb0588c052ed252a1246ec8e3c5cd5ebe4c748b59796ba6";
constexpr std::string_view kParent14FirstState =
    "e8b9d51befb1444cf16021e12275585d0fca5e8a398bdce42c82f4a6b0920199";
constexpr std::string_view kParent14FirstVelocity =
    "ca8d7ab81a427256f734821ddfddcfc4ec684718eff30890488b6b7b2495e41c";
constexpr std::size_t kMaximumOuterMaps = 8U;
constexpr std::size_t kMaximumPressureRounds = 8U;
constexpr std::size_t kMaximumQpSweeps = 16384U;
constexpr std::size_t kTrajectorySteps = 16U;
constexpr long double kFixedRms = 1.0e-9L;
constexpr long double kFixedMaximum = 1.0e-8L;
constexpr long double kPrimalLimit = 1.0e-8L;
constexpr long double kKktLimit = 1.0e-8L;
constexpr long double kComplementarityLimit = 1.0e-10L;
constexpr long double kSymmetryLimit = 2.0e-12L;
constexpr long double kDensityMaximumLimit = 1.0e-3L;
constexpr long double kDensityRmsLimit = 2.5e-4L;
constexpr long double kBalanceLimit = 1.0e-8L;

void put_u32(std::string& bytes, std::uint32_t value) {
    for (std::uint32_t index = 0U; index < 4U; ++index) {
        bytes.push_back(static_cast<char>((value >> (8U * index)) & 0xffU));
    }
}
void put_u64(std::string& bytes, std::uint64_t value) {
    for (std::uint32_t index = 0U; index < 8U; ++index) {
        bytes.push_back(static_cast<char>((value >> (8U * index)) & 0xffU));
    }
}
void put_f32(std::string& bytes, float value) {
    std::uint32_t bits = 0U;
    static_assert(sizeof(bits) == sizeof(value));
    std::memcpy(&bits, &value, sizeof(bits));
    put_u32(bytes, bits);
}
void put_f64(std::string& bytes, long double value) {
    if (!std::isfinite(value)) throw std::runtime_error("nonfinite root value");
    const double narrowed = static_cast<double>(value);
    std::uint64_t bits = 0U;
    static_assert(sizeof(bits) == sizeof(narrowed));
    std::memcpy(&bits, &narrowed, sizeof(bits));
    put_u64(bytes, bits);
}
void put_string(std::string& bytes, std::string_view value) {
    put_u64(bytes, value.size());
    bytes.append(value.data(), value.size());
}
std::string root(std::string bytes) {
    return nextengine::nonlocal::sha256_hex(bytes);
}

struct BinaryRead {
    std::string root;
    std::uint64_t bytes = 0U;
};
BinaryRead read_binary() {
    std::ifstream stream("/proc/self/exe", std::ios::binary);
    if (!stream) throw std::runtime_error("binary open failed");
    std::ostringstream bytes;
    bytes << stream.rdbuf();
    if (!stream.good() && !stream.eof()) {
        throw std::runtime_error("binary read failed");
    }
    return {root(bytes.str()), static_cast<std::uint64_t>(bytes.str().size())};
}

struct Work16 {
    std::uint64_t fixture_records = 0U;
    std::uint64_t ghost_records = 0U;
    std::uint64_t canonicalized_records = 0U;
    std::uint64_t graph_builds = 0U;
    std::uint64_t graph_candidates = 0U;
    std::uint64_t accepted_dynamic_pairs = 0U;
    std::uint64_t accepted_ghost_pairs = 0U;
    std::uint64_t density_pairs = 0U;
    std::uint64_t derivative_pairs = 0U;
    std::uint64_t jacobian_products = 0U;
    std::uint64_t matrix_products = 0U;
    std::uint64_t symmetry_reductions = 0U;
    std::uint64_t qp_sweeps = 0U;
    std::uint64_t qp_row_updates = 0U;
    std::uint64_t gradient_recomputations = 0U;
    std::uint64_t pressure_rounds = 0U;
    std::uint64_t outer_map_evaluations = 0U;
    std::uint64_t nonpressure_objective_evaluations = 0U;
    std::uint64_t nonpressure_gradient_evaluations = 0U;
    std::uint64_t viscosity_pairs = 0U;
    std::uint64_t surface_tests = 0U;
    std::uint64_t surface_active_pairs = 0U;
    std::uint64_t box_plane_tests = 0U;
    std::uint64_t clamp_hits = 0U;
    std::uint64_t contact_reaction_values = 0U;
    std::uint64_t fixed_point_predicates = 0U;
    std::uint64_t correspondence_predicates = 0U;
    std::uint64_t physical_predicates = 0U;
    std::uint64_t route_predicates = 0U;
    std::uint64_t independent_pair_tests = 0U;
    std::uint64_t portable_fields = 0U;
    std::uint64_t hash_derivations = 0U;
};

std::array<std::uint64_t, 32> work_values(const Work16& work) {
    return {work.fixture_records, work.ghost_records,
        work.canonicalized_records, work.graph_builds,
        work.graph_candidates, work.accepted_dynamic_pairs,
        work.accepted_ghost_pairs, work.density_pairs,
        work.derivative_pairs, work.jacobian_products,
        work.matrix_products, work.symmetry_reductions, work.qp_sweeps,
        work.qp_row_updates, work.gradient_recomputations,
        work.pressure_rounds, work.outer_map_evaluations,
        work.nonpressure_objective_evaluations,
        work.nonpressure_gradient_evaluations, work.viscosity_pairs,
        work.surface_tests, work.surface_active_pairs, work.box_plane_tests,
        work.clamp_hits, work.contact_reaction_values,
        work.fixed_point_predicates, work.correspondence_predicates,
        work.physical_predicates, work.route_predicates,
        work.independent_pair_tests, work.portable_fields,
        work.hash_derivations};
}

void add_work(Work16& target, const Work16& value) {
    const auto values = work_values(value);
    std::size_t index = 0U;
    for (std::uint64_t* field : std::array<std::uint64_t*, 32>{
             &target.fixture_records, &target.ghost_records,
             &target.canonicalized_records, &target.graph_builds,
             &target.graph_candidates, &target.accepted_dynamic_pairs,
             &target.accepted_ghost_pairs, &target.density_pairs,
             &target.derivative_pairs, &target.jacobian_products,
             &target.matrix_products, &target.symmetry_reductions,
             &target.qp_sweeps, &target.qp_row_updates,
             &target.gradient_recomputations, &target.pressure_rounds,
             &target.outer_map_evaluations,
             &target.nonpressure_objective_evaluations,
             &target.nonpressure_gradient_evaluations,
             &target.viscosity_pairs, &target.surface_tests,
             &target.surface_active_pairs, &target.box_plane_tests,
             &target.clamp_hits, &target.contact_reaction_values,
             &target.fixed_point_predicates,
             &target.correspondence_predicates,
             &target.physical_predicates, &target.route_predicates,
             &target.independent_pair_tests, &target.portable_fields,
             &target.hash_derivations}) {
        *field += values[index++];
    }
}

void add_nonpressure_work(Work16& target, const Work15& work) {
    target.graph_builds += work.graph_builds;
    target.graph_candidates += work.graph_candidates;
    target.density_pairs += work.density_pairs;
    target.jacobian_products += work.jacobian_products;
    target.nonpressure_objective_evaluations += work.objective_evaluations;
    target.nonpressure_gradient_evaluations += work.gradient_evaluations;
    target.viscosity_pairs += work.viscosity_pairs;
    target.surface_tests += work.surface_pair_tests;
    target.surface_active_pairs += work.surface_active_pairs;
}

std::string work_root(const Work16& work) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp16.work.v1");
    for (const std::uint64_t value : work_values(work)) put_u64(bytes, value);
    return root(std::move(bytes));
}

long double pairwise_sum(std::vector<long double> values) {
    if (values.empty()) return 0.0L;
    while (values.size() > 1U) {
        std::vector<long double> next;
        next.reserve((values.size() + 1U) / 2U);
        for (std::size_t index = 0U; index < values.size(); index += 2U) {
            next.push_back(index + 1U < values.size()
                    ? values[index] + values[index + 1U]
                    : values[index]);
        }
        values = std::move(next);
    }
    return values.front();
}
long double vector_norm(const std::vector<long double>& values) {
    long double sum = 0.0L;
    for (const long double value : values) sum += value * value;
    return std::sqrt(std::max(sum, 0.0L));
}
long double vec_norm(const std::vector<Vec3l>& values) {
    std::vector<long double> terms;
    terms.reserve(values.size());
    for (const Vec3l value : values) {
        terms.push_back(nextengine::nonlocal::ncgp15::dot15(value, value));
    }
    return std::sqrt(pairwise_sum(std::move(terms)));
}

struct Fixture16 {
    NonlocalGpuProfile parent_profile;
    State15 state;
    std::vector<float> dynamic_raw;
    std::vector<float> ghost_raw;
    std::string profile_root;
    std::string fixture_root;
    std::string lane_root;
    std::string input_root;
};

std::string parent_profile_root(const NonlocalGpuProfile& profile) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.profile.v1");
    put_string(bytes, profile.id);
    for (const double value : std::array<double, 19>{profile.dt,
             profile.spacing, profile.horizon, profile.mass,
             profile.rest_density, profile.kernel_scale, profile.kappa,
             profile.lambda, profile.mu, profile.gamma, profile.gravity.x,
             profile.gravity.y, profile.gravity.z, profile.basin_extent.x,
             profile.basin_extent.y, profile.basin_extent.z,
             static_cast<double>(profile.ghost_layers),
             static_cast<double>(profile.maximum_dynamic_samples),
             static_cast<double>(profile.maximum_neighbors)}) {
        put_f64(bytes, value);
    }
    return root(std::move(bytes));
}

Fixture16 make_fixture(Work16& work) {
    Fixture16 result;
    result.parent_profile = nonlocal_water_corrected_profile();
    result.parent_profile.lambda = 0.0;
    result.parent_profile.mu = 0.0;
    result.parent_profile.gamma = 0.0;
    result.parent_profile.basin_extent = {0.2, 0.2, 0.6};
    result.profile_root = parent_profile_root(result.parent_profile);
    result.state.dynamic.reserve(128U);
    result.dynamic_raw.reserve(128U * 9U);
    for (std::uint32_t iz = 0U; iz < 8U; ++iz) {
        for (std::uint32_t iy = 0U; iy < 4U; ++iy) {
            for (std::uint32_t ix = 0U; ix < 4U; ++ix) {
                const std::uint32_t logical = ix + 4U * (iy + 4U * iz);
                const float x = static_cast<float>(0.025
                    + 0.05 * static_cast<double>(ix));
                const float y = static_cast<float>(0.025
                    + 0.05 * static_cast<double>(iy));
                const float z = static_cast<float>(0.025
                    + 0.05 * static_cast<double>(iz));
                const Vec3l value{static_cast<long double>(x),
                    static_cast<long double>(y), static_cast<long double>(z)};
                result.state.dynamic.push_back(
                    {1000U + 17U * logical, value, value, {}});
                for (const float field : std::array<float, 9>{x, y, z,
                         x, y, z, 0.0F, 0.0F, 0.0F}) {
                    result.dynamic_raw.push_back(field);
                }
            }
        }
    }
    const auto ghosts = make_basin_ghosts(result.parent_profile);
    result.state.ghosts.reserve(ghosts.size());
    result.ghost_raw.reserve(ghosts.size() * 3U);
    for (const NonlocalGpuGhost& ghost : ghosts) {
        const float x = static_cast<float>(ghost.position.x);
        const float y = static_cast<float>(ghost.position.y);
        const float z = static_cast<float>(ghost.position.z);
        result.state.ghosts.push_back({ghost.sample_id,
            {static_cast<long double>(x), static_cast<long double>(y),
                static_cast<long double>(z)}});
        result.ghost_raw.insert(result.ghost_raw.end(), {x, y, z});
    }
    work.fixture_records += result.state.dynamic.size();
    work.ghost_records += result.state.ghosts.size();
    work.canonicalized_records += result.state.dynamic.size()
        + result.state.ghosts.size();
    std::string fixture_bytes;
    put_string(fixture_bytes, "nextengine.nonlocal.ncgp14.fixture.v1");
    put_string(fixture_bytes, "TIGHT-128");
    put_string(fixture_bytes, result.profile_root);
    put_u64(fixture_bytes, result.state.dynamic.size());
    std::size_t raw = 0U;
    for (const auto& sample : result.state.dynamic) {
        put_u32(fixture_bytes, sample.id);
        for (std::size_t field = 0U; field < 9U; ++field) {
            put_f32(fixture_bytes, result.dynamic_raw[raw++]);
        }
    }
    put_u64(fixture_bytes, result.state.ghosts.size());
    raw = 0U;
    for (const auto& ghost : result.state.ghosts) {
        put_u32(fixture_bytes, ghost.id);
        for (std::size_t field = 0U; field < 3U; ++field) {
            put_f32(fixture_bytes, result.ghost_raw[raw++]);
        }
    }
    result.fixture_root = root(std::move(fixture_bytes));
    std::string lane_bytes;
    put_string(lane_bytes, "nextengine.nonlocal.ncgp14.lane.v1");
    put_string(lane_bytes, "TIGHT-128-CAP16384");
    put_string(lane_bytes, result.fixture_root);
    put_u32(lane_bytes, 2U);
    put_u32(lane_bytes, 8U);
    put_u64(lane_bytes, 16384U);
    for (const long double value : std::array<long double, 16>{2.0e-7L,
             2.0e-12L, 1.0e-8L, 1.0e-8L, 1.0e-10L, 1.0e-8L,
             1.0e-3L, 2.5e-4L, 0.0L, 1.0e-8L, 0.0025L, 0.005L,
             0.05L, 0.10L, 0.01L, 0.01L}) {
        put_f64(lane_bytes, value);
    }
    put_u32(lane_bytes, 1U);
    result.lane_root = root(std::move(lane_bytes));
    std::string input_bytes;
    put_string(input_bytes, "nextengine.nonlocal.ncgp16.input.v1");
    put_string(input_bytes, result.profile_root);
    put_string(input_bytes, result.fixture_root);
    put_string(input_bytes, result.lane_root);
    put_u64(input_bytes, result.state.dynamic.size());
    for (const auto& sample : result.state.dynamic) {
        put_u32(input_bytes, sample.id);
        for (const Vec3l value : std::array<Vec3l, 3>{sample.reference,
                 sample.position, sample.velocity}) {
            put_f64(input_bytes, value.x);
            put_f64(input_bytes, value.y);
            put_f64(input_bytes, value.z);
        }
    }
    put_u64(input_bytes, result.state.ghosts.size());
    for (const auto& ghost : result.state.ghosts) {
        put_u32(input_bytes, ghost.id);
        put_f64(input_bytes, ghost.position.x);
        put_f64(input_bytes, ghost.position.y);
        put_f64(input_bytes, ghost.position.z);
    }
    result.input_root = root(std::move(input_bytes));
    work.portable_fields += 10U * result.state.dynamic.size()
        + 4U * result.state.ghosts.size() + 5U;
    work.hash_derivations += 4U;
    if (result.fixture_root != kParent14Fixture
            || result.lane_root != kParent14Lane) {
        throw std::runtime_error("frozen NCGP14 fixture lineage mismatch");
    }
    return result;
}

struct Kernel16 {
    long double value = 0.0L;
    long double derivative = 0.0L;
};
Kernel16 kernel(long double radius, const SolverOptions15& options) {
    const long double horizon = options.profile.horizon;
    const long double q = 2.0L * radius / horizon;
    const long double alpha = options.profile.kernel_scale * 3.0L
        / (2.0L * std::acos(-1.0L) * horizon * horizon * horizon);
    long double value = 0.0L;
    long double derivative_q = 0.0L;
    if (q < 1.0L) {
        value = alpha * (2.0L / 3.0L - q * q + 0.5L * q * q * q);
        derivative_q = alpha * (-2.0L * q + 1.5L * q * q);
    } else if (q <= 2.0L) {
        const long double tail = 2.0L - q;
        value = alpha * tail * tail * tail / 6.0L;
        derivative_q = -0.5L * alpha * tail * tail;
    }
    return {value, derivative_q * 2.0L / horizon};
}

using Cell16 = std::tuple<int, int, int>;
Cell16 cell_of(Vec3l value, long double cell_size) {
    return {static_cast<int>(std::floor(value.x / cell_size)),
        static_cast<int>(std::floor(value.y / cell_size)),
        static_cast<int>(std::floor(value.z / cell_size))};
}

struct Neighbor16 {
    std::size_t index = 0U;
    bool ghost = false;
};
struct Graph16 {
    bool valid = true;
    bool capacity = false;
    std::vector<std::vector<Neighbor16>> rows;
    std::string root;
};

Graph16 build_graph(const State15& state, const std::vector<Vec3l>& positions,
    const SolverOptions15& options, bool all_pairs, Work16& work) {
    Graph16 result;
    result.rows.resize(positions.size());
    ++work.graph_builds;
    std::map<Cell16, std::vector<std::size_t>> ghost_cells;
    if (!all_pairs) {
        for (std::size_t index = 0U; index < state.ghosts.size(); ++index) {
            ghost_cells[cell_of(state.ghosts[index].position,
                options.profile.horizon)].push_back(index);
        }
    }
    for (std::size_t owner = 0U; owner < positions.size(); ++owner) {
        for (std::size_t neighbor = 0U; neighbor < positions.size(); ++neighbor) {
            ++work.graph_candidates;
            if (all_pairs) ++work.independent_pair_tests;
            if (nextengine::nonlocal::ncgp15::norm15(
                    positions[owner] - positions[neighbor])
                    <= options.profile.horizon) {
                result.rows[owner].push_back({neighbor, false});
                ++work.accepted_dynamic_pairs;
            }
        }
        std::vector<std::size_t> candidates;
        if (all_pairs) {
            candidates.resize(state.ghosts.size());
            std::iota(candidates.begin(), candidates.end(), 0U);
        } else {
            const auto [cx, cy, cz] = cell_of(
                positions[owner], options.profile.horizon);
            for (int dz = -2; dz <= 2; ++dz) {
                for (int dy = -2; dy <= 2; ++dy) {
                    for (int dx = -2; dx <= 2; ++dx) {
                        const auto found = ghost_cells.find(
                            {cx + dx, cy + dy, cz + dz});
                        if (found != ghost_cells.end()) {
                            candidates.insert(candidates.end(),
                                found->second.begin(), found->second.end());
                        }
                    }
                }
            }
            std::sort(candidates.begin(), candidates.end());
        }
        for (const std::size_t ghost : candidates) {
            ++work.graph_candidates;
            if (all_pairs) ++work.independent_pair_tests;
            if (nextengine::nonlocal::ncgp15::norm15(
                    positions[owner] - state.ghosts[ghost].position)
                    <= options.profile.horizon) {
                result.rows[owner].push_back({ghost, true});
                ++work.accepted_ghost_pairs;
            }
        }
        if (result.rows[owner].size() > options.profile.maximum_neighbors) {
            result.valid = false;
            result.capacity = true;
            return result;
        }
    }
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp13.graph.v1");
    put_u64(bytes, result.rows.size());
    for (std::size_t owner = 0U; owner < result.rows.size(); ++owner) {
        put_u32(bytes, state.dynamic[owner].id);
        put_u64(bytes, result.rows[owner].size());
        for (const Neighbor16 edge : result.rows[owner]) {
            put_u32(bytes, edge.ghost ? state.ghosts[edge.index].id
                                      : state.dynamic[edge.index].id);
            put_u32(bytes, edge.ghost ? 1U : 0U);
        }
    }
    result.root = root(std::move(bytes));
    ++work.hash_derivations;
    return result;
}

struct Assembly16 {
    bool valid = false;
    Graph16 graph;
    std::vector<long double> density;
    std::vector<long double> constraint;
    std::vector<long double> jacobian;
    std::vector<long double> matrix;
    long double symmetry = std::numeric_limits<long double>::infinity();
    std::string density_root;
    std::string jacobian_root;
    std::string matrix_root;
};

void add_jacobian(std::vector<long double>& jacobian, std::size_t dofs,
    std::size_t row, std::size_t sample, Vec3l value, long double sign) {
    const std::size_t base = row * dofs + 3U * sample;
    jacobian[base] += sign * value.x;
    jacobian[base + 1U] += sign * value.y;
    jacobian[base + 2U] += sign * value.z;
}

Assembly16 assemble(const State15& state,
    const std::vector<Vec3l>& positions, const SolverOptions15& options,
    bool all_pairs, Work16& work) {
    Assembly16 result;
    const std::size_t count = positions.size();
    const std::size_t dofs = 3U * count;
    result.graph = build_graph(state, positions, options, all_pairs, work);
    if (!result.graph.valid) return result;
    result.density.assign(count, 0.0L);
    result.constraint.assign(count, 0.0L);
    result.jacobian.assign(count * dofs, 0.0L);
    result.matrix.assign(count * count, 0.0L);
    for (std::size_t owner = 0U; owner < count; ++owner) {
        for (const Neighbor16 edge : result.graph.rows[owner]) {
            ++work.density_pairs;
            const Vec3l neighbor = edge.ghost
                ? state.ghosts[edge.index].position : positions[edge.index];
            const Vec3l difference = positions[owner] - neighbor;
            const long double radius =
                nextengine::nonlocal::ncgp15::norm15(difference);
            const Kernel16 values = kernel(radius, options);
            result.density[owner] += options.profile.mass * values.value;
            if (!(radius > 0.0L)
                    || (!edge.ghost && edge.index == owner)) {
                continue;
            }
            ++work.derivative_pairs;
            const Vec3l derivative = difference
                * (options.profile.mass / options.profile.rho0
                    * values.derivative / radius);
            add_jacobian(result.jacobian, dofs, owner, owner,
                derivative, 1.0L);
            if (!edge.ghost) {
                add_jacobian(result.jacobian, dofs, owner,
                    edge.index, derivative, -1.0L);
            }
        }
        result.constraint[owner] = result.density[owner]
            / options.profile.rho0 - 1.0L;
    }
    const long double scale = options.profile.dt * options.profile.dt
        / options.profile.mass;
    for (std::size_t column = 0U; column < dofs; ++column) {
        std::vector<std::pair<std::size_t, long double>> entries;
        for (std::size_t row = 0U; row < count; ++row) {
            const long double value = result.jacobian[row * dofs + column];
            if (value != 0.0L) entries.emplace_back(row, value);
        }
        for (const auto& lhs : entries) {
            for (const auto& rhs : entries) {
                result.matrix[lhs.first * count + rhs.first] +=
                    scale * lhs.second * rhs.second;
                ++work.matrix_products;
            }
        }
    }
    long double difference2 = 0.0L;
    long double reference2 = 0.0L;
    bool finite = true;
    for (std::size_t row = 0U; row < count; ++row) {
        const long double diagonal = result.matrix[row * count + row];
        finite = std::isfinite(result.density[row])
            && std::isfinite(result.constraint[row])
            && diagonal > 0.0L && std::isfinite(diagonal) && finite;
        for (std::size_t column = 0U; column < dofs; ++column) {
            finite = std::isfinite(result.jacobian[row * dofs + column])
                && finite;
        }
        for (std::size_t column = 0U; column < count; ++column) {
            const long double lhs = result.matrix[row * count + column];
            const long double rhs = result.matrix[column * count + row];
            difference2 += (lhs - rhs) * (lhs - rhs);
            reference2 += lhs * lhs + rhs * rhs;
            work.symmetry_reductions += 2U;
            finite = std::isfinite(lhs) && finite;
        }
    }
    result.symmetry = std::sqrt(difference2)
        / std::max(std::sqrt(0.5L * reference2), 1.0e-30L);
    finite = std::isfinite(result.symmetry) && finite;
    auto scalar_root = [&](std::string_view domain,
                           const std::vector<long double>& values) {
        std::string bytes;
        put_string(bytes, domain);
        put_u64(bytes, values.size());
        for (std::size_t index = 0U; index < values.size(); ++index) {
            put_u32(bytes, state.dynamic[index % count].id);
            put_f64(bytes, values[index]);
        }
        ++work.hash_derivations;
        return root(std::move(bytes));
    };
    result.density_root = scalar_root(
        "nextengine.nonlocal.ncgp16.pressure-density.v1", result.density);
    result.jacobian_root = scalar_root(
        "nextengine.nonlocal.ncgp16.pressure-jacobian.v1", result.jacobian);
    result.matrix_root = scalar_root(
        "nextengine.nonlocal.ncgp16.pressure-matrix.v1", result.matrix);
    result.valid = finite && result.symmetry <= kSymmetryLimit;
    return result;
}

struct Solve16 {
    bool valid = true;
    bool converged = false;
    std::vector<long double> lambda;
    std::vector<long double> gradient;
    std::size_t sweeps = 0U;
    std::uint64_t updates = 0U;
    long double primal = std::numeric_limits<long double>::infinity();
    long double kkt = std::numeric_limits<long double>::infinity();
    long double complementarity = std::numeric_limits<long double>::infinity();
};

void recompute_gradient(const Assembly16& assembly, Solve16& solve,
    Work16& work) {
    const std::size_t count = assembly.constraint.size();
    for (std::size_t row = 0U; row < count; ++row) {
        long double value = -assembly.constraint[row];
        for (std::size_t column = 0U; column < count; ++column) {
            value += assembly.matrix[row * count + column]
                * solve.lambda[column];
        }
        solve.gradient[row] = value;
    }
    ++work.gradient_recomputations;
}

void solve_residuals(const Assembly16& assembly, Solve16& solve) {
    const std::size_t count = assembly.constraint.size();
    std::vector<long double> positive(count);
    std::vector<long double> source(count);
    std::vector<long double> projected(count);
    solve.complementarity = 0.0L;
    for (std::size_t row = 0U; row < count; ++row) {
        positive[row] = std::max(-solve.gradient[row], 0.0L);
        source[row] = std::max(assembly.constraint[row], 0.0L);
        const long double diagonal = assembly.matrix[row * count + row];
        projected[row] = std::min(
            diagonal * solve.lambda[row], solve.gradient[row]);
        solve.complementarity = std::max(solve.complementarity,
            std::fabs(solve.lambda[row] * solve.gradient[row]));
    }
    solve.primal = vector_norm(positive)
        / std::max(vector_norm(source), 1.0e-30L);
    solve.kkt = vector_norm(projected)
        / std::max(vector_norm(assembly.constraint), 1.0e-30L);
}

Solve16 solve_qp(const Assembly16& assembly, Work16& work) {
    Solve16 solve;
    const std::size_t count = assembly.constraint.size();
    solve.lambda.assign(count, 0.0L);
    solve.gradient.resize(count);
    recompute_gradient(assembly, solve, work);
    for (std::size_t sweep = 1U; sweep <= kMaximumQpSweeps; ++sweep) {
        for (std::size_t row = 0U; row < count; ++row) {
            const long double diagonal = assembly.matrix[row * count + row];
            if (!(diagonal > 0.0L) || !std::isfinite(diagonal)) {
                solve.valid = false;
                return solve;
            }
            const long double prior = solve.lambda[row];
            const long double next = std::max(
                0.0L, prior - solve.gradient[row] / diagonal);
            const long double delta = next - prior;
            if (delta == 0.0L) continue;
            solve.lambda[row] = next;
            ++solve.updates;
            for (std::size_t affected = 0U; affected < count; ++affected) {
                solve.gradient[affected] +=
                    assembly.matrix[affected * count + row] * delta;
            }
        }
        solve.sweeps = sweep;
        recompute_gradient(assembly, solve, work);
        solve_residuals(assembly, solve);
        solve.valid = std::isfinite(solve.primal)
            && std::isfinite(solve.kkt)
            && std::isfinite(solve.complementarity)
            && std::all_of(solve.lambda.begin(), solve.lambda.end(),
                [](long double value) {
                    return std::isfinite(value) && value >= 0.0L;
                });
        if (!solve.valid) return solve;
        if (solve.primal <= kPrimalLimit && solve.kkt <= kKktLimit
                && solve.complementarity <= kComplementarityLimit) {
            solve.converged = true;
            break;
        }
    }
    work.qp_sweeps += solve.sweeps;
    work.qp_row_updates += solve.updates;
    return solve;
}

std::vector<long double> jt_times(const Assembly16& assembly,
    const std::vector<long double>& lambda, Work16& work) {
    const std::size_t count = assembly.constraint.size();
    const std::size_t dofs = 3U * count;
    std::vector<long double> result(dofs, 0.0L);
    for (std::size_t row = 0U; row < count; ++row) {
        for (std::size_t column = 0U; column < dofs; ++column) {
            result[column] += assembly.jacobian[row * dofs + column]
                * lambda[row];
            ++work.jacobian_products;
        }
    }
    return result;
}

struct Contact16 {
    bool valid = true;
    std::vector<Vec3l> endpoint;
    std::vector<Vec3l> correction;
    std::vector<Vec3l> impulse;
    std::vector<std::uint32_t> mask;
    std::uint64_t lower_hits = 0U;
    std::uint64_t top_hits = 0U;
};

Contact16 contact(const State15& state, const std::vector<Vec3l>& start,
    const std::vector<Vec3l>& proposal, const SolverOptions15& options,
    Work16& work) {
    Contact16 result;
    const long double low = 0.5L * options.profile.spacing;
    const Vec3l high{options.profile.basin.x - low,
        options.profile.basin.y - low, options.profile.basin.z - low};
    result.endpoint = proposal;
    result.correction.resize(proposal.size());
    result.impulse.resize(proposal.size());
    result.mask.assign(proposal.size(), 0U);
    for (std::size_t index = 0U; index < proposal.size(); ++index) {
        const std::array<long double, 3> a{
            start[index].x, start[index].y, start[index].z};
        const std::array<long double, 3> b{
            proposal[index].x, proposal[index].y, proposal[index].z};
        const std::array<long double, 3> maximum{high.x, high.y, high.z};
        std::array<long double*, 3> endpoint{&result.endpoint[index].x,
            &result.endpoint[index].y, &result.endpoint[index].z};
        bool finite_inside = nextengine::nonlocal::ncgp15::finite15(start[index])
            && nextengine::nonlocal::ncgp15::finite15(proposal[index]);
        for (std::size_t axis = 0U; axis < 3U; ++axis) {
            finite_inside = finite_inside && a[axis] >= low
                && a[axis] <= maximum[axis];
            for (std::size_t side = 0U; side < 2U; ++side) {
                ++work.box_plane_tests;
                const bool high_side = side != 0U;
                const bool hit = high_side ? b[axis] > maximum[axis]
                                           : b[axis] < low;
                if (!hit) continue;
                const long double plane = high_side ? maximum[axis] : low;
                const long double denominator = b[axis] - a[axis];
                const long double time = denominator == 0.0L
                    ? std::numeric_limits<long double>::infinity()
                    : (plane - a[axis]) / denominator;
                finite_inside = finite_inside && std::isfinite(time)
                    && time >= 0.0L && time <= 1.0L;
                *endpoint[axis] = plane;
                result.mask[index] |= 1U << (2U * axis + side);
                ++work.clamp_hits;
            }
        }
        result.valid = result.valid && finite_inside;
        result.correction[index] = result.endpoint[index] - proposal[index];
        result.impulse[index] = result.correction[index]
            * (options.profile.mass / options.profile.dt);
        work.contact_reaction_values += 3U;
        if ((result.mask[index] & (1U << 4U)) != 0U) ++result.lower_hits;
        if ((result.mask[index] & (1U << 5U)) != 0U) ++result.top_hits;
    }
    static_cast<void>(state);
    return result;
}

struct Density16 {
    bool valid = false;
    std::vector<long double> value;
    std::vector<long double> constraint;
};

Density16 density_from_graph(const State15& state,
    const std::vector<Vec3l>& positions, const SolverOptions15& options,
    const Graph16& graph, Work16& work) {
    Density16 result;
    result.value.assign(positions.size(), 0.0L);
    result.constraint.assign(positions.size(), 0.0L);
    result.valid = graph.valid;
    for (std::size_t owner = 0U; owner < positions.size(); ++owner) {
        for (const Neighbor16 edge : graph.rows[owner]) {
            ++work.density_pairs;
            const Vec3l neighbor = edge.ghost
                ? state.ghosts[edge.index].position : positions[edge.index];
            const long double radius = nextengine::nonlocal::ncgp15::norm15(
                positions[owner] - neighbor);
            result.value[owner] += options.profile.mass
                * kernel(radius, options).value;
        }
        result.constraint[owner] = result.value[owner]
            / options.profile.rho0 - 1.0L;
        result.valid = std::isfinite(result.value[owner])
            && std::isfinite(result.constraint[owner]) && result.valid;
    }
    return result;
}

std::pair<long double, long double> strain_metrics(
    const std::vector<long double>& constraints) {
    long double maximum = 0.0L;
    std::vector<long double> squares;
    squares.reserve(constraints.size());
    for (const long double constraint : constraints) {
        const long double positive = std::max(constraint, 0.0L);
        maximum = std::max(maximum, positive);
        squares.push_back(positive * positive);
    }
    return {maximum, std::sqrt(pairwise_sum(std::move(squares))
        / static_cast<long double>(constraints.size()))};
}

std::string id_vec_root(std::string_view domain, const State15& state,
    const std::vector<Vec3l>& values) {
    if (state.dynamic.size() != values.size()) {
        throw std::logic_error("id-vector root size mismatch");
    }
    std::string bytes;
    put_string(bytes, domain);
    put_u64(bytes, values.size());
    for (std::size_t index = 0U; index < values.size(); ++index) {
        put_u32(bytes, state.dynamic[index].id);
        put_f64(bytes, values[index].x);
        put_f64(bytes, values[index].y);
        put_f64(bytes, values[index].z);
    }
    return root(std::move(bytes));
}

std::string id_scalar_root(std::string_view domain, const State15& state,
    const std::vector<long double>& values) {
    if (state.dynamic.size() != values.size()) {
        throw std::logic_error("id-scalar root size mismatch");
    }
    std::string bytes;
    put_string(bytes, domain);
    put_u64(bytes, values.size());
    for (std::size_t index = 0U; index < values.size(); ++index) {
        put_u32(bytes, state.dynamic[index].id);
        put_f64(bytes, values[index]);
    }
    return root(std::move(bytes));
}

std::string id_mask_root(std::string_view domain, const State15& state,
    const std::vector<std::uint32_t>& values) {
    if (state.dynamic.size() != values.size()) {
        throw std::logic_error("id-mask root size mismatch");
    }
    std::string bytes;
    put_string(bytes, domain);
    put_u64(bytes, values.size());
    for (std::size_t index = 0U; index < values.size(); ++index) {
        put_u32(bytes, state.dynamic[index].id);
        put_u32(bytes, values[index]);
    }
    return root(std::move(bytes));
}

std::string parent_state_root(const State15& prior,
    const std::vector<Vec3l>& positions, const std::vector<Vec3l>& velocity) {
    if (prior.dynamic.size() != positions.size()
            || positions.size() != velocity.size()) {
        throw std::logic_error("parent state root size mismatch");
    }
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp13.accepted-state.v1");
    put_u64(bytes, positions.size());
    for (std::size_t index = 0U; index < positions.size(); ++index) {
        put_u32(bytes, prior.dynamic[index].id);
        for (const Vec3l value : std::array<Vec3l, 3>{
                 prior.dynamic[index].reference, positions[index],
                 velocity[index]}) {
            put_f64(bytes, value.x);
            put_f64(bytes, value.y);
            put_f64(bytes, value.z);
        }
    }
    return root(std::move(bytes));
}

struct PressureRound16 {
    bool apparatus_valid = false;
    bool qp_cap = false;
    bool closed = false;
    std::string outcome = "NOT_RUN";
    Assembly16 assembly;
    Solve16 solve;
    Contact16 contact;
    Density16 post_density;
    std::vector<Vec3l> position;
    std::vector<Vec3l> correction;
    std::vector<Vec3l> pressure_force;
    long double pressure_balance = std::numeric_limits<long double>::infinity();
    long double maximum_strain = std::numeric_limits<long double>::infinity();
    long double rms_strain = std::numeric_limits<long double>::infinity();
    std::string multiplier_root;
    std::string correction_root;
    std::string contact_root;
    std::string post_density_root;
    std::string state_root;
    std::string result_root;
    Work16 work;
};

PressureRound16 pressure_round(const State15& state,
    const std::vector<Vec3l>& input, const SolverOptions15& options,
    bool all_pairs, std::uint32_t round_index) {
    PressureRound16 result;
    ++result.work.pressure_rounds;
    result.assembly = assemble(state, input, options, all_pairs, result.work);
    result.work.correspondence_predicates += 2U;
    if (!result.assembly.valid
            || result.assembly.symmetry > kSymmetryLimit) {
        result.outcome = result.assembly.graph.capacity
            ? "ROW_NEIGHBOR_CAPACITY_EXCEEDED" : "ASSEMBLY_INVALID";
        return result;
    }
    result.solve = solve_qp(result.assembly, result.work);
    result.work.correspondence_predicates += 2U;
    if (!result.solve.valid) {
        result.outcome = "QP_INVALID";
        return result;
    }
    if (!result.solve.converged) {
        result.qp_cap = result.solve.sweeps == kMaximumQpSweeps;
        result.outcome = result.qp_cap
            ? "QP_SWEEP_CAP_EXHAUSTED" : "QP_INVALID";
        return result;
    }
    const std::vector<long double> jt = jt_times(
        result.assembly, result.solve.lambda, result.work);
    const long double scale = options.profile.dt * options.profile.dt
        / options.profile.mass;
    result.correction.resize(input.size());
    result.pressure_force.resize(input.size());
    std::vector<Vec3l> proposed(input.size());
    std::vector<Vec3l> residual(input.size());
    for (std::size_t index = 0U; index < input.size(); ++index) {
        result.pressure_force[index] = {jt[3U * index],
            jt[3U * index + 1U], jt[3U * index + 2U]};
        result.correction[index] = -scale * result.pressure_force[index];
        proposed[index] = input[index] + result.correction[index];
        residual[index] = result.correction[index]
                * (options.profile.mass
                    / (options.profile.dt * options.profile.dt))
            + result.pressure_force[index];
    }
    const long double residual_norm = vec_norm(residual);
    result.pressure_balance = residual_norm
        / std::max({vec_norm(result.pressure_force),
            options.profile.mass
                * nextengine::nonlocal::ncgp15::norm15(
                    options.profile.gravity)
                * std::sqrt(static_cast<long double>(input.size())),
            1.0e-30L * static_cast<long double>(input.size())});
    result.contact = contact(
        state, input, proposed, options, result.work);
    if (!result.contact.valid) {
        result.outcome = "CONTACT_INVALID";
        return result;
    }
    result.position = result.contact.endpoint;
    const Graph16 post_graph = build_graph(
        state, result.position, options, all_pairs, result.work);
    result.post_density = density_from_graph(
        state, result.position, options, post_graph, result.work);
    if (!result.post_density.valid) {
        result.outcome = "POST_DENSITY_INVALID";
        return result;
    }
    const auto strains = strain_metrics(result.post_density.constraint);
    result.maximum_strain = strains.first;
    result.rms_strain = strains.second;
    result.work.physical_predicates += 7U;
    const bool finite = std::isfinite(result.pressure_balance)
        && std::isfinite(result.maximum_strain)
        && std::isfinite(result.rms_strain);
    const bool inset = result.contact.top_hits == 0U
        && std::all_of(result.position.begin(), result.position.end(),
            [&](Vec3l value) {
                const long double low = 0.5L * options.profile.spacing;
                const Vec3l high{options.profile.basin.x - low,
                    options.profile.basin.y - low,
                    options.profile.basin.z - low};
                return value.x >= low && value.x <= high.x
                    && value.y >= low && value.y <= high.y
                    && value.z >= low && value.z <= high.z;
            });
    result.apparatus_valid = finite && inset
        && result.pressure_balance <= kBalanceLimit;
    result.closed = result.apparatus_valid
        && result.maximum_strain <= kDensityMaximumLimit
        && result.rms_strain <= kDensityRmsLimit;
    result.outcome = !result.apparatus_valid ? "ROUND_APPARATUS_INVALID"
        : result.closed ? "PASS" : "ROUND_OPEN";
    result.multiplier_root = id_scalar_root(
        "nextengine.nonlocal.ncgp16.pressure-multiplier.v1",
        state, result.solve.lambda);
    result.correction_root = id_vec_root(
        "nextengine.nonlocal.ncgp16.pressure-correction.v1",
        state, result.correction);
    result.contact_root = id_mask_root(
        "nextengine.nonlocal.ncgp16.pressure-contact.v1",
        state, result.contact.mask);
    result.post_density_root = id_scalar_root(
        "nextengine.nonlocal.ncgp16.pressure-post-density.v1",
        state, result.post_density.value);
    result.state_root = id_vec_root(
        "nextengine.nonlocal.ncgp16.pressure-round-state.v1",
        state, result.position);
    result.work.hash_derivations += 5U;
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp16.pressure-round.v1");
    put_u32(bytes, round_index);
    put_string(bytes, all_pairs ? "ALL_PAIRS_ORACLE" : "CSR_CANDIDATE");
    put_string(bytes, result.outcome);
    put_string(bytes, result.assembly.graph.root);
    put_string(bytes, result.assembly.density_root);
    put_string(bytes, result.assembly.jacobian_root);
    put_string(bytes, result.assembly.matrix_root);
    put_string(bytes, result.multiplier_root);
    put_string(bytes, result.correction_root);
    put_string(bytes, result.contact_root);
    put_string(bytes, result.post_density_root);
    put_string(bytes, result.state_root);
    put_u64(bytes, result.solve.sweeps);
    put_u64(bytes, result.solve.updates);
    put_f64(bytes, result.solve.primal);
    put_f64(bytes, result.solve.kkt);
    put_f64(bytes, result.solve.complementarity);
    put_f64(bytes, result.assembly.symmetry);
    put_f64(bytes, result.pressure_balance);
    put_f64(bytes, result.maximum_strain);
    put_f64(bytes, result.rms_strain);
    put_string(bytes, work_root(result.work));
    result.work.portable_fields += 22U;
    result.result_root = root(std::move(bytes));
    return result;
}

struct PressureProject16 {
    bool apparatus_valid = false;
    bool accepted = false;
    bool work_ceiling = false;
    std::string outcome = "NOT_RUN";
    std::vector<PressureRound16> candidate;
    std::vector<PressureRound16> oracle;
    std::vector<Vec3l> position;
    std::vector<Vec3l> velocity;
    std::string state_root;
    std::string velocity_root;
    std::string result_root;
    Work16 work;
};

bool same_round(const PressureRound16& candidate,
    const PressureRound16& oracle, Work16& owner) {
    std::vector<Vec3l> difference(candidate.position.size());
    for (std::size_t index = 0U; index < difference.size(); ++index) {
        difference[index] = candidate.position[index] - oracle.position[index];
    }
    owner.correspondence_predicates += 14U;
    return candidate.outcome == oracle.outcome
        && candidate.assembly.graph.root == oracle.assembly.graph.root
        && candidate.assembly.density_root == oracle.assembly.density_root
        && candidate.assembly.jacobian_root == oracle.assembly.jacobian_root
        && candidate.assembly.matrix_root == oracle.assembly.matrix_root
        && candidate.solve.sweeps == oracle.solve.sweeps
        && candidate.solve.updates == oracle.solve.updates
        && candidate.solve.lambda == oracle.solve.lambda
        && candidate.contact.mask == oracle.contact.mask
        && candidate.post_density.value == oracle.post_density.value
        && candidate.maximum_strain == oracle.maximum_strain
        && candidate.rms_strain == oracle.rms_strain
        && candidate.pressure_balance == oracle.pressure_balance
        && vec_norm(difference) == 0.0L;
}

PressureProject16 pressure_project(const State15& state,
    const std::vector<Vec3l>& proposal, const SolverOptions15& options) {
    PressureProject16 result;
    std::vector<Vec3l> candidate_position = proposal;
    std::vector<Vec3l> oracle_position = proposal;
    bool correspondence = true;
    for (std::size_t round_index = 0U;
         round_index < kMaximumPressureRounds; ++round_index) {
        PressureRound16 candidate = pressure_round(state,
            candidate_position, options, false,
            static_cast<std::uint32_t>(round_index + 1U));
        PressureRound16 oracle = pressure_round(state,
            oracle_position, options, true,
            static_cast<std::uint32_t>(round_index + 1U));
        add_work(result.work, candidate.work);
        add_work(result.work, oracle.work);
        correspondence = same_round(candidate, oracle, result.work)
            && correspondence;
        result.candidate.push_back(std::move(candidate));
        result.oracle.push_back(std::move(oracle));
        const PressureRound16& candidate_sealed = result.candidate.back();
        const PressureRound16& oracle_sealed = result.oracle.back();
        if (!correspondence) {
            result.outcome = "PRESSURE_CORRESPONDENCE_INVALID";
            break;
        }
        if (candidate_sealed.qp_cap || oracle_sealed.qp_cap) {
            result.work_ceiling = true;
            result.outcome = "QP_SWEEP_CAP_EXHAUSTED";
            break;
        }
        if (!candidate_sealed.apparatus_valid
                || !oracle_sealed.apparatus_valid) {
            result.outcome = candidate_sealed.outcome;
            break;
        }
        candidate_position = candidate_sealed.position;
        oracle_position = oracle_sealed.position;
        if (candidate_sealed.closed && oracle_sealed.closed) {
            result.apparatus_valid = true;
            result.accepted = true;
            result.outcome = "PASS";
            result.position = candidate_position;
            break;
        }
    }
    if (result.outcome == "NOT_RUN") {
        result.work_ceiling = true;
        result.outcome = "PROJECTION_ROUND_CAP_EXHAUSTED";
    }
    if (result.accepted) {
        result.velocity.resize(result.position.size());
        for (std::size_t index = 0U; index < result.position.size(); ++index) {
            result.velocity[index] = (result.position[index]
                    - state.dynamic[index].position)
                / options.profile.dt;
        }
        result.state_root = parent_state_root(
            state, result.position, result.velocity);
        result.velocity_root = id_vec_root(
            "nextengine.nonlocal.ncgp13.velocity.v1", state,
            result.velocity);
        result.work.hash_derivations += 2U;
    }
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp16.pressure-project.v1");
    put_string(bytes, result.outcome);
    put_u64(bytes, result.candidate.size());
    for (std::size_t index = 0U; index < result.candidate.size(); ++index) {
        put_string(bytes, result.candidate[index].result_root);
        put_string(bytes, result.oracle[index].result_root);
    }
    put_string(bytes, result.state_root);
    put_string(bytes, result.velocity_root);
    put_string(bytes, work_root(result.work));
    result.work.portable_fields += 6U + 2U * result.candidate.size();
    result.result_root = root(std::move(bytes));
    return result;
}

std::vector<Vec3l> project_box(const std::vector<Vec3l>& input,
    const SolverOptions15& options, Work16& work) {
    std::vector<Vec3l> result = input;
    const long double low = 0.5L * options.profile.spacing;
    const Vec3l high{options.profile.basin.x - low,
        options.profile.basin.y - low, options.profile.basin.z - low};
    for (Vec3l& value : result) {
        const std::array<long double, 3> source{value.x, value.y, value.z};
        const std::array<long double, 3> maximum{high.x, high.y, high.z};
        std::array<long double*, 3> target{&value.x, &value.y, &value.z};
        for (std::size_t axis = 0U; axis < 3U; ++axis) {
            work.box_plane_tests += 2U;
            const long double projected = std::min(
                std::max(source[axis], low), maximum[axis]);
            if (projected != source[axis]) ++work.clamp_hits;
            *target[axis] = projected;
        }
    }
    return result;
}

struct Nonpressure16 {
    bool valid = false;
    long double relative_l2 = std::numeric_limits<long double>::infinity();
    long double maximum = std::numeric_limits<long double>::infinity();
    EnergyGradient15 candidate;
    EnergyGradient15 oracle;
    Work15 candidate_work;
    Work15 oracle_work;
    std::string candidate_root;
    std::string oracle_root;
};

std::string nonpressure_root(std::string_view route, const State15& state,
    const EnergyGradient15& value, const Work15& value_work) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp16.nonpressure.v1");
    put_string(bytes, route);
    put_f64(bytes, value.objective);
    put_f64(bytes, value.inertia_energy);
    put_f64(bytes, value.viscosity_energy);
    put_f64(bytes, value.surface_energy);
    put_u64(bytes, value.gradient.size());
    for (std::size_t index = 0U; index < value.gradient.size(); ++index) {
        put_u32(bytes, state.dynamic[index].id);
        for (const Vec3l field : std::array<Vec3l, 3>{value.gradient[index],
                 value.viscosity_force[index], value.surface_force[index]}) {
            put_f64(bytes, field.x);
            put_f64(bytes, field.y);
            put_f64(bytes, field.z);
        }
    }
    for (const std::uint64_t field :
         nextengine::nonlocal::ncgp15::work_values15(value_work)) {
        put_u64(bytes, field);
    }
    return root(std::move(bytes));
}

Nonpressure16 evaluate_nonpressure(const State15& state,
    const std::vector<Vec3l>& position, const SolverOptions15& options,
    Work16& work) {
    Nonpressure16 result;
    const std::vector<long double> zero(position.size(), 0.0L);
    result.candidate =
        nextengine::nonlocal::ncgp15::candidate_energy_gradient15(
            state, position, zero, options, MultiplierMode15::FinalKkt,
            true, result.candidate_work);
    result.oracle = nextengine::nonlocal::ncgp15::oracle_energy_gradient15(
        state, position, zero, options, MultiplierMode15::FinalKkt,
        true, result.oracle_work);
    add_nonpressure_work(work, result.candidate_work);
    add_nonpressure_work(work, result.oracle_work);
    if (!result.candidate.valid || !result.oracle.valid
            || result.candidate.gradient.size() != position.size()
            || result.oracle.gradient.size() != position.size()) {
        return result;
    }
    std::vector<Vec3l> difference(position.size());
    std::vector<long double> reference;
    reference.reserve(2U * position.size());
    result.maximum = 0.0L;
    for (std::size_t index = 0U; index < position.size(); ++index) {
        difference[index] = result.candidate.gradient[index]
            - result.oracle.gradient[index];
        result.maximum = std::max(result.maximum,
            nextengine::nonlocal::ncgp15::norm15(difference[index]));
        reference.push_back(nextengine::nonlocal::ncgp15::dot15(
            result.candidate.gradient[index],
            result.candidate.gradient[index]));
        reference.push_back(nextengine::nonlocal::ncgp15::dot15(
            result.oracle.gradient[index], result.oracle.gradient[index]));
    }
    result.relative_l2 = vec_norm(difference)
        / std::max(std::sqrt(0.5L * pairwise_sum(std::move(reference))),
            1.0e-30L);
    work.correspondence_predicates += 6U;
    result.valid = std::isfinite(result.relative_l2)
        && std::isfinite(result.maximum)
        && result.relative_l2
            <= options.profile.gates.gradient_relative_l2
        && result.maximum <= options.profile.gates.gradient_maximum_n;
    result.candidate_root = nonpressure_root(
        "CSR_CANDIDATE", state, result.candidate, result.candidate_work);
    result.oracle_root = nonpressure_root(
        "ALL_PAIRS_ORACLE", state, result.oracle, result.oracle_work);
    work.hash_derivations += 2U;
    return result;
}

struct Outer16 {
    bool valid = false;
    bool fixed = false;
    long double fixed_rms = std::numeric_limits<long double>::infinity();
    long double fixed_maximum = std::numeric_limits<long double>::infinity();
    Nonpressure16 nonpressure;
    PressureProject16 pressure;
    std::string input_root;
    std::string proposal_root;
    std::string output_root;
    std::string result_root;
    Work16 work;
};

Outer16 outer_map(const State15& state, const std::vector<Vec3l>& y,
    const SolverOptions15& options, std::uint32_t outer_index) {
    Outer16 result;
    ++result.work.outer_map_evaluations;
    result.input_root = id_vec_root(
        "nextengine.nonlocal.ncgp16.outer-input.v1", state, y);
    ++result.work.hash_derivations;
    result.nonpressure = evaluate_nonpressure(state, y, options, result.work);
    if (!result.nonpressure.valid) return result;
    const long double scale = options.profile.dt * options.profile.dt
        / options.profile.mass;
    std::vector<Vec3l> raw(y.size());
    for (std::size_t index = 0U; index < y.size(); ++index) {
        raw[index] = y[index]
            - scale * result.nonpressure.candidate.gradient[index];
    }
    const std::vector<Vec3l> proposal = project_box(raw, options, result.work);
    result.proposal_root = id_vec_root(
        "nextengine.nonlocal.ncgp16.outer-proposal.v1", state, proposal);
    ++result.work.hash_derivations;
    result.pressure = pressure_project(state, proposal, options);
    add_work(result.work, result.pressure.work);
    if (!result.pressure.accepted) return result;
    std::vector<long double> squares;
    squares.reserve(y.size());
    result.fixed_maximum = 0.0L;
    for (std::size_t index = 0U; index < y.size(); ++index) {
        const long double distance = nextengine::nonlocal::ncgp15::norm15(
            result.pressure.position[index] - y[index]);
        squares.push_back(distance * distance);
        result.fixed_maximum = std::max(result.fixed_maximum, distance);
    }
    result.fixed_rms = std::sqrt(pairwise_sum(std::move(squares))
        / static_cast<long double>(y.size()));
    result.work.fixed_point_predicates += 2U;
    result.fixed = result.fixed_rms / options.profile.spacing <= kFixedRms
        && result.fixed_maximum / options.profile.spacing <= kFixedMaximum;
    result.output_root = id_vec_root(
        "nextengine.nonlocal.ncgp16.outer-output.v1", state,
        result.pressure.position);
    ++result.work.hash_derivations;
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp16.outer-map.v1");
    put_u32(bytes, outer_index);
    put_string(bytes, result.input_root);
    put_string(bytes, result.nonpressure.candidate_root);
    put_string(bytes, result.nonpressure.oracle_root);
    put_string(bytes, result.proposal_root);
    put_string(bytes, result.pressure.result_root);
    put_string(bytes, result.output_root);
    put_f64(bytes, result.fixed_rms);
    put_f64(bytes, result.fixed_maximum);
    put_u32(bytes, result.fixed ? 1U : 0U);
    put_string(bytes, work_root(result.work));
    result.result_root = root(std::move(bytes));
    result.valid = true;
    return result;
}

SolverOptions15 make_options(const TermMask15& terms) {
    SolverOptions15 options;
    options.profile = nextengine::nonlocal::ncgp15::frozen_profile15();
    options.terms = terms;
    options.terms.density_constraint = false;
    options.terms.tangential_viscosity = false;
    return options;
}

struct Step16 {
    std::string mask;
    std::string outcome = "NOT_RUN";
    bool apparatus_valid = false;
    bool work_ceiling = false;
    bool accepted = false;
    bool lineage_exact = true;
    std::vector<Outer16> outer;
    State15 state;
    std::string state_root;
    std::string velocity_root;
    std::string result_root;
    Work16 work;
};

Step16 run_step(const State15& prior, std::string mask,
    const TermMask15& terms, bool require_parent_lineage) {
    Step16 result;
    result.mask = std::move(mask);
    result.state = prior;
    const SolverOptions15 options = make_options(terms);
    std::vector<Vec3l> predicted;
    predicted.reserve(prior.dynamic.size());
    for (const auto& sample : prior.dynamic) {
        predicted.push_back(sample.position + options.profile.dt
            * (sample.velocity + options.profile.dt
                * options.profile.gravity));
    }
    std::vector<Vec3l> y = project_box(predicted, options, result.work);
    for (std::size_t outer = 0U; outer < kMaximumOuterMaps; ++outer) {
        Outer16 evaluation = outer_map(
            prior, y, options, static_cast<std::uint32_t>(outer + 1U));
        add_work(result.work, evaluation.work);
        result.outer.push_back(std::move(evaluation));
        const Outer16& sealed = result.outer.back();
        if (!sealed.valid) {
            if (sealed.pressure.work_ceiling) {
                result.work_ceiling = true;
                result.outcome = sealed.pressure.outcome;
            } else {
                result.outcome = sealed.nonpressure.valid
                    ? sealed.pressure.outcome
                    : "NONPRESSURE_CORRESPONDENCE_INVALID";
            }
            break;
        }
        y = sealed.pressure.position;
        if (require_parent_lineage && outer == 0U) {
            std::vector<std::uint64_t> sweeps;
            for (const PressureRound16& round : sealed.pressure.candidate) {
                sweeps.push_back(round.solve.sweeps);
            }
            result.work.correspondence_predicates += 4U;
            result.lineage_exact = sealed.pressure.state_root
                    == kParent14FirstState
                && sealed.pressure.velocity_root == kParent14FirstVelocity
                && sweeps == std::vector<std::uint64_t>{6887U, 7663U,
                    6759U, 6521U, 6042U}
                && sealed.pressure.candidate.size() == 5U;
            if (!result.lineage_exact) {
                result.outcome = "PARENT_PRESSURE_LINEAGE_INVALID";
                break;
            }
        }
        if (sealed.fixed) {
            result.apparatus_valid = true;
            result.accepted = true;
            result.outcome = "PASS";
            for (std::size_t index = 0U; index < y.size(); ++index) {
                result.state.dynamic[index].position = y[index];
                result.state.dynamic[index].velocity =
                    (y[index] - prior.dynamic[index].position)
                    / options.profile.dt;
            }
            result.state_root = sealed.pressure.state_root;
            result.velocity_root = sealed.pressure.velocity_root;
            break;
        }
    }
    if (result.outcome == "NOT_RUN") {
        result.work_ceiling = true;
        result.outcome = "OUTER_MAP_CAP_EXHAUSTED";
    }
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp16.step.v1");
    put_string(bytes, result.mask);
    put_string(bytes, result.outcome);
    put_u32(bytes, result.accepted ? 1U : 0U);
    put_u64(bytes, result.outer.size());
    for (const Outer16& evaluation : result.outer) {
        put_string(bytes, evaluation.result_root);
    }
    put_string(bytes, result.state_root);
    put_string(bytes, result.velocity_root);
    put_string(bytes, work_root(result.work));
    result.result_root = root(std::move(bytes));
    return result;
}

void emit_finite(std::ostream& output, long double value) {
    if (std::isfinite(value)) {
        output << static_cast<double>(value);
    } else {
        output << "null";
    }
}

void emit_work(std::ostream& output, const Work16& work) {
    constexpr std::array<std::string_view, 32> names{
        "fixture_records", "ghost_records", "canonicalized_records",
        "graph_builds", "graph_candidates", "accepted_dynamic_pairs",
        "accepted_ghost_pairs", "density_pairs", "derivative_pairs",
        "jacobian_products", "matrix_products", "symmetry_reductions",
        "qp_sweeps", "qp_row_updates", "gradient_recomputations",
        "pressure_rounds", "outer_map_evaluations",
        "nonpressure_objective_evaluations",
        "nonpressure_gradient_evaluations", "viscosity_pairs",
        "surface_tests", "surface_active_pairs", "box_plane_tests",
        "clamp_hits", "contact_reaction_values",
        "fixed_point_predicates", "correspondence_predicates",
        "physical_predicates", "route_predicates",
        "independent_pair_tests", "portable_fields", "hash_derivations"};
    const auto values = work_values(work);
    output << '{';
    for (std::size_t index = 0U; index < names.size(); ++index) {
        if (index != 0U) output << ',';
        output << '"' << names[index] << "\":" << values[index];
    }
    output << '}';
}

void emit_step(std::ostream& output, const Step16& step) {
    output << "{\"mask\":\"" << step.mask << "\",\"outcome\":\""
           << step.outcome << "\",\"apparatus_valid\":"
           << (step.apparatus_valid ? "true" : "false")
           << ",\"work_ceiling\":"
           << (step.work_ceiling ? "true" : "false")
           << ",\"accepted\":" << (step.accepted ? "true" : "false")
           << ",\"lineage_exact\":"
           << (step.lineage_exact ? "true" : "false")
           << ",\"outer_evaluations\":" << step.outer.size()
           << ",\"state_root\":\"" << step.state_root
           << "\",\"velocity_root\":\"" << step.velocity_root
           << "\",\"work_root\":\"" << work_root(step.work)
           << "\",\"result_root\":\"" << step.result_root
           << "\",\"work\":";
    emit_work(output, step.work);
    output << ",\"outer\":[";
    for (std::size_t index = 0U; index < step.outer.size(); ++index) {
        if (index != 0U) output << ',';
        const Outer16& evaluation = step.outer[index];
        output << "{\"index\":" << index + 1U
               << ",\"valid\":"
               << (evaluation.valid ? "true" : "false")
               << ",\"fixed\":"
               << (evaluation.fixed ? "true" : "false")
               << ",\"fixed_rms_m\":";
        emit_finite(output, evaluation.fixed_rms);
        output << ",\"fixed_maximum_m\":";
        emit_finite(output, evaluation.fixed_maximum);
        output << ",\"gradient_relative_l2\":";
        emit_finite(output, evaluation.nonpressure.relative_l2);
        output << ",\"gradient_maximum_n\":";
        emit_finite(output, evaluation.nonpressure.maximum);
        output << ",\"pressure_outcome\":\""
               << evaluation.pressure.outcome
               << "\",\"pressure_rounds\":"
               << evaluation.pressure.candidate.size()
               << ",\"pressure_sweeps\":[";
        for (std::size_t round = 0U;
             round < evaluation.pressure.candidate.size(); ++round) {
            if (round != 0U) output << ',';
            output << evaluation.pressure.candidate[round].solve.sweeps;
        }
        output << "],\"pressure_round_metrics\":[";
        for (std::size_t round = 0U;
             round < evaluation.pressure.candidate.size(); ++round) {
            if (round != 0U) output << ',';
            const PressureRound16& value =
                evaluation.pressure.candidate[round];
            output << "{\"outcome\":\"" << value.outcome
                   << "\",\"primal\":";
            emit_finite(output, value.solve.primal);
            output << ",\"projected_kkt\":";
            emit_finite(output, value.solve.kkt);
            output << ",\"complementarity\":";
            emit_finite(output, value.solve.complementarity);
            output << ",\"symmetry\":";
            emit_finite(output, value.assembly.symmetry);
            output << ",\"pressure_balance\":";
            emit_finite(output, value.pressure_balance);
            output << ",\"maximum_positive_strain\":";
            emit_finite(output, value.maximum_strain);
            output << ",\"rms_positive_strain\":";
            emit_finite(output, value.rms_strain);
            output << ",\"active_multiplier_count\":"
                   << std::count_if(value.solve.lambda.begin(),
                        value.solve.lambda.end(),
                        [](long double lambda) { return lambda > 0.0L; })
                   << ",\"result_root\":\"" << value.result_root
                   << "\"}";
        }
        output << "],\"input_root\":\"" << evaluation.input_root
               << "\",\"proposal_root\":\"" << evaluation.proposal_root
               << "\",\"pressure_result_root\":\""
               << evaluation.pressure.result_root
               << "\",\"output_root\":\"" << evaluation.output_root
               << "\",\"result_root\":\"" << evaluation.result_root
               << "\"}";
    }
    output << "]}";
}

int run() {
    const BinaryRead binary = read_binary();
    Work16 identity_work;
    const Fixture16 fixture = make_fixture(identity_work);
    const std::array<std::pair<std::string, TermMask15>, 4> masks{
        std::pair{"P", TermMask15{false, false, false, false}},
        std::pair{"PV", TermMask15{false, true, false, false}},
        std::pair{"PS", TermMask15{false, false, false, true}},
        std::pair{"PVS", TermMask15{false, true, false, true}}};
    std::vector<Step16> masks_result;
    masks_result.reserve(masks.size());
    std::string status = "PRESSURE_QP_UNIFIED_SUPPORTED";
    for (std::size_t index = 0U; index < masks.size(); ++index) {
        Step16 step = run_step(fixture.state, masks[index].first,
            masks[index].second, index == 0U);
        const bool accepted = step.accepted;
        const bool ceiling = step.work_ceiling;
        masks_result.push_back(std::move(step));
        if (!accepted) {
            if (ceiling) status = "SOLVER_WORK_CEILING_INCONCLUSIVE";
            else if (!masks_result.back().apparatus_valid) {
                status = "APPARATUS_INCONCLUSIVE";
            } else if (index == 0U) {
                status = "PRESSURE_BLOCK_BASELINE_REFUTED";
            } else if (index == 1U) {
                status = "VISCOSITY_COUPLING_REFUTED";
            } else if (index == 2U) {
                status = "SURFACE_COUPLING_REFUTED";
            } else {
                status = "APPARATUS_INCONCLUSIVE";
            }
            break;
        }
    }
    std::vector<Step16> trajectory;
    State15 trajectory_state = fixture.state;
    if (masks_result.size() == masks.size()
            && std::all_of(masks_result.begin(), masks_result.end(),
                [](const Step16& value) { return value.accepted; })) {
        for (std::size_t step_index = 0U;
             step_index < kTrajectorySteps; ++step_index) {
            Step16 step = run_step(trajectory_state, "PVS",
                TermMask15{false, true, false, true}, false);
            const bool accepted = step.accepted;
            if (accepted) trajectory_state = step.state;
            trajectory.push_back(std::move(step));
            if (!accepted) {
                status = trajectory.back().work_ceiling
                    ? "SOLVER_WORK_CEILING_INCONCLUSIVE"
                    : trajectory.back().apparatus_valid
                    ? "UNIFIED_TRAJECTORY_REFUTED"
                    : "APPARATUS_INCONCLUSIVE";
                break;
            }
        }
    }
    Work16 total = identity_work;
    for (const Step16& step : masks_result) add_work(total, step.work);
    for (const Step16& step : trajectory) add_work(total, step.work);
    std::string trajectory_bytes;
    put_string(trajectory_bytes, "nextengine.nonlocal.ncgp16.trajectory.v1");
    put_u64(trajectory_bytes, trajectory.size());
    for (const Step16& step : trajectory) {
        put_string(trajectory_bytes, step.result_root);
    }
    const std::string trajectory_root = root(std::move(trajectory_bytes));
    ++total.hash_derivations;
    std::string result_bytes;
    put_string(result_bytes, "nextengine.nonlocal.ncgp16.result.v1");
    put_string(result_bytes, NCGP16_CONTRACT_ROOT);
    put_string(result_bytes, NCGP16_SOURCE_ROOT);
    put_string(result_bytes, NCGP16_SOURCE_COMMIT);
    put_string(result_bytes, NCGP16_SOURCE_TREE);
    put_string(result_bytes, binary.root);
    put_string(result_bytes, fixture.input_root);
    put_string(result_bytes, status);
    put_u64(result_bytes, masks_result.size());
    for (const Step16& step : masks_result) {
        put_string(result_bytes, step.result_root);
    }
    put_string(result_bytes, trajectory_root);
    put_string(result_bytes, work_root(total));
    const std::string result_root = root(std::move(result_bytes));
    std::cout << std::setprecision(17)
              << "{\"schema\":\"" << kSchema << "\",\"status\":\""
              << status << "\",\"invocation\":\"" << kInvocation
              << "\",\"contract_root\":\"" << NCGP16_CONTRACT_ROOT
              << "\",\"contract_sha\":\"" << NCGP16_CONTRACT_SHA
              << "\",\"source_root\":\"" << NCGP16_SOURCE_ROOT
              << "\",\"source_commit\":\"" << NCGP16_SOURCE_COMMIT
              << "\",\"source_tree\":\"" << NCGP16_SOURCE_TREE
              << "\",\"compiler_family\":\"" << NCGP16_COMPILER_FAMILY
              << "\",\"compiler_version\":\"" << NCGP16_COMPILER_VERSION
              << "\",\"compiler_flags\":\"" << NCGP16_COMPILER_FLAGS
              << "\",\"binary_root\":\"" << binary.root
              << "\",\"binary_bytes\":" << binary.bytes
              << ",\"input_root\":\"" << fixture.input_root
              << "\",\"fixture_root\":\"" << fixture.fixture_root
              << "\",\"lane_root\":\"" << fixture.lane_root
              << "\",\"masks\":[";
    for (std::size_t index = 0U; index < masks_result.size(); ++index) {
        if (index != 0U) std::cout << ',';
        emit_step(std::cout, masks_result[index]);
    }
    std::cout << "],\"trajectory\":[";
    for (std::size_t index = 0U; index < trajectory.size(); ++index) {
        if (index != 0U) std::cout << ',';
        emit_step(std::cout, trajectory[index]);
    }
    std::cout << "],\"trajectory_root\":\"" << trajectory_root
              << "\",\"work_root\":\"" << work_root(total)
              << "\",\"work\":";
    emit_work(std::cout, total);
    std::cout << ",\"result_root\":\"" << result_root << "\"}\n";
    return status == "APPARATUS_INCONCLUSIVE" ? 2 : 0;
}

} // namespace ncgp16

int main(int argc, char** argv) {
    if (argc != 2 || std::string_view(argv[1]) != "--pressure-qp-unified") {
        std::cerr << "usage: " << ncgp16::kInvocation << '\n';
        return 64;
    }
    try {
        return ncgp16::run();
    } catch (const std::exception& error) {
        std::cerr << "NCGP16 identity/runtime failure: " << error.what()
                  << '\n';
        return 3;
    }
}
