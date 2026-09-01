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
    target.independent_pair_tests += work.oracle_pair_tests;
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

Vec3l pairwise_vec_sum(std::vector<Vec3l> values) {
    if (values.empty()) return {};
    while (values.size() > 1U) {
        std::vector<Vec3l> next;
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

long double force_closure(std::vector<Vec3l> forces, Work16& work) {
    std::vector<long double> norms;
    norms.reserve(forces.size());
    bool finite = true;
    for (const Vec3l force : forces) {
        finite = nextengine::nonlocal::ncgp15::finite15(force) && finite;
        norms.push_back(nextengine::nonlocal::ncgp15::norm15(force));
        ++work.physical_predicates;
    }
    if (!finite) return std::numeric_limits<long double>::infinity();
    return nextengine::nonlocal::ncgp15::norm15(
        pairwise_vec_sum(std::move(forces)))
        / std::max(pairwise_sum(std::move(norms)), 1.0L);
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
    const long double chain = options.mutation
            == nextengine::nonlocal::ncgp15::Mutation15::MissingKernelChain
        ? 1.0L : 2.0L / horizon;
    return {value, derivative_q * chain};
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
    std::uint64_t edge_count = 0U;
    for (const auto& row : result.rows) edge_count += row.size();
    work.portable_fields += 2U + 2U * result.rows.size()
        + 2U * edge_count;
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
    if (!finite || result.symmetry > kSymmetryLimit) {
        result.valid = false;
        return result;
    }
    auto scalar_root = [&](std::string_view domain,
                           const std::vector<long double>& values) {
        std::string bytes;
        put_string(bytes, domain);
        put_u64(bytes, values.size());
        for (std::size_t index = 0U; index < values.size(); ++index) {
            put_u32(bytes, state.dynamic[index % count].id);
            put_f64(bytes, values[index]);
        }
        work.portable_fields += 2U + 2U * values.size();
        ++work.hash_derivations;
        return root(std::move(bytes));
    };
    result.density_root = scalar_root(
        "nextengine.nonlocal.ncgp16.pressure-density.v1", result.density);
    result.jacobian_root = scalar_root(
        "nextengine.nonlocal.ncgp16.pressure-jacobian.v1", result.jacobian);
    result.matrix_root = scalar_root(
        "nextengine.nonlocal.ncgp16.pressure-matrix.v1", result.matrix);
    result.valid = true;
    return result;
}

Work16 expected_graph_work(const State15& state,
    const std::vector<Vec3l>& positions, const SolverOptions15& options,
    bool all_pairs) {
    Work16 expected;
    ++expected.graph_builds;
    std::map<Cell16, std::vector<std::size_t>> ghost_cells;
    if (!all_pairs) {
        for (std::size_t index = 0U; index < state.ghosts.size(); ++index) {
            ghost_cells[cell_of(state.ghosts[index].position,
                options.profile.horizon)].push_back(index);
        }
    }
    for (std::size_t owner = 0U; owner < positions.size(); ++owner) {
        expected.graph_candidates += positions.size();
        if (all_pairs) expected.independent_pair_tests += positions.size();
        for (std::size_t neighbor = 0U; neighbor < positions.size(); ++neighbor) {
            if (nextengine::nonlocal::ncgp15::norm15(
                    positions[owner] - positions[neighbor])
                    <= options.profile.horizon) {
                ++expected.accepted_dynamic_pairs;
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
        expected.graph_candidates += candidates.size();
        if (all_pairs) expected.independent_pair_tests += candidates.size();
        for (const std::size_t ghost : candidates) {
            if (nextengine::nonlocal::ncgp15::norm15(
                    positions[owner] - state.ghosts[ghost].position)
                    <= options.profile.horizon) {
                ++expected.accepted_ghost_pairs;
            }
        }
    }
    const std::uint64_t edges = expected.accepted_dynamic_pairs
        + expected.accepted_ghost_pairs;
    expected.portable_fields += 2U + 2U * positions.size() + 2U * edges;
    ++expected.hash_derivations;
    return expected;
}

Work16 expected_assembly_work(const State15& state,
    const std::vector<Vec3l>& positions, const SolverOptions15& options,
    bool all_pairs, const Assembly16& sealed) {
    Work16 expected = expected_graph_work(
        state, positions, options, all_pairs);
    const std::size_t count = positions.size();
    const std::size_t dofs = 3U * count;
    for (std::size_t owner = 0U; owner < sealed.graph.rows.size(); ++owner) {
        for (const Neighbor16 edge : sealed.graph.rows[owner]) {
            ++expected.density_pairs;
            const Vec3l neighbor = edge.ghost
                ? state.ghosts[edge.index].position : positions[edge.index];
            const long double radius = nextengine::nonlocal::ncgp15::norm15(
                positions[owner] - neighbor);
            if (radius > 0.0L && (edge.ghost || edge.index != owner)) {
                ++expected.derivative_pairs;
            }
        }
    }
    for (std::size_t column = 0U; column < dofs; ++column) {
        std::uint64_t nonzero = 0U;
        for (std::size_t row = 0U; row < count; ++row) {
            if (sealed.jacobian[row * dofs + column] != 0.0L) ++nonzero;
        }
        expected.matrix_products += nonzero * nonzero;
    }
    expected.symmetry_reductions += 2U * count * count;
    expected.portable_fields += 6U + 2U * (sealed.density.size()
        + sealed.jacobian.size() + sealed.matrix.size());
    expected.hash_derivations += 3U;
    return expected;
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

Solve16 solve_qp(const Assembly16& assembly, Work16& work,
    std::size_t maximum_sweeps = kMaximumQpSweeps) {
    Solve16 solve;
    const std::size_t count = assembly.constraint.size();
    solve.lambda.assign(count, 0.0L);
    solve.gradient.resize(count);
    recompute_gradient(assembly, solve, work);
    for (std::size_t sweep = 1U; sweep <= maximum_sweeps; ++sweep) {
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
    const std::vector<Vec3l>& values, Work16* work = nullptr) {
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
    if (work != nullptr) work->portable_fields += 2U + 4U * values.size();
    return root(std::move(bytes));
}

std::string id_scalar_root(std::string_view domain, const State15& state,
    const std::vector<long double>& values, Work16* work = nullptr) {
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
    if (work != nullptr) work->portable_fields += 2U + 2U * values.size();
    return root(std::move(bytes));
}

std::string id_mask_root(std::string_view domain, const State15& state,
    const std::vector<std::uint32_t>& values, Work16* work = nullptr) {
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
    if (work != nullptr) work->portable_fields += 2U + 2U * values.size();
    return root(std::move(bytes));
}

std::string ghost_vec_root(std::string_view domain, const State15& state,
    const std::vector<Vec3l>& values, Work16* work = nullptr) {
    if (state.ghosts.size() != values.size()) {
        throw std::logic_error("ghost-vector root size mismatch");
    }
    std::string bytes;
    put_string(bytes, domain);
    put_u64(bytes, values.size());
    for (std::size_t index = 0U; index < values.size(); ++index) {
        put_u32(bytes, state.ghosts[index].id);
        put_f64(bytes, values[index].x);
        put_f64(bytes, values[index].y);
        put_f64(bytes, values[index].z);
    }
    if (work != nullptr) work->portable_fields += 2U + 4U * values.size();
    return root(std::move(bytes));
}

std::string parent_state_root(const State15& prior,
    const std::vector<Vec3l>& positions, const std::vector<Vec3l>& velocity,
    Work16* work = nullptr) {
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
    if (work != nullptr) work->portable_fields += 2U + 10U * positions.size();
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
    std::vector<Vec3l> ghost_pressure_force;
    long double pressure_balance = std::numeric_limits<long double>::infinity();
    long double maximum_strain = std::numeric_limits<long double>::infinity();
    long double rms_strain = std::numeric_limits<long double>::infinity();
    std::string multiplier_root;
    std::string correction_root;
    std::string ghost_pressure_root;
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
    result.ghost_pressure_force.assign(state.ghosts.size(), {});
    for (std::size_t owner = 0U;
         owner < result.assembly.graph.rows.size(); ++owner) {
        for (const Neighbor16 edge : result.assembly.graph.rows[owner]) {
            if (!edge.ghost) continue;
            const Vec3l difference = input[owner]
                - state.ghosts[edge.index].position;
            const long double radius =
                nextengine::nonlocal::ncgp15::norm15(difference);
            if (!(radius > 0.0L)) continue;
            const Vec3l derivative = difference
                * (options.profile.mass / options.profile.rho0
                    * kernel(radius, options).derivative / radius);
            result.ghost_pressure_force[edge.index] -=
                result.solve.lambda[owner] * derivative;
            result.work.jacobian_products += 3U;
        }
    }
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
        state, result.solve.lambda, &result.work);
    result.correction_root = id_vec_root(
        "nextengine.nonlocal.ncgp16.pressure-correction.v1",
        state, result.correction, &result.work);
    result.ghost_pressure_root = ghost_vec_root(
        "nextengine.nonlocal.ncgp16.pressure-ghost-force.v1", state,
        result.ghost_pressure_force, &result.work);
    result.contact_root = id_mask_root(
        "nextengine.nonlocal.ncgp16.pressure-contact.v1",
        state, result.contact.mask, &result.work);
    result.post_density_root = id_scalar_root(
        "nextengine.nonlocal.ncgp16.pressure-post-density.v1",
        state, result.post_density.value, &result.work);
    result.state_root = id_vec_root(
        "nextengine.nonlocal.ncgp16.pressure-round-state.v1",
        state, result.position, &result.work);
    result.work.hash_derivations += 6U;
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
    put_string(bytes, result.ghost_pressure_root);
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
    result.work.portable_fields += 23U;
    put_string(bytes, work_root(result.work));
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
    Vec3l pressure_impulse;
    Vec3l contact_impulse;
    std::uint64_t top_contacts = 0U;
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
    owner.correspondence_predicates += 15U;
    return candidate.outcome == oracle.outcome
        && candidate.assembly.graph.root == oracle.assembly.graph.root
        && candidate.assembly.density_root == oracle.assembly.density_root
        && candidate.assembly.jacobian_root == oracle.assembly.jacobian_root
        && candidate.assembly.matrix_root == oracle.assembly.matrix_root
        && candidate.solve.sweeps == oracle.solve.sweeps
        && candidate.solve.updates == oracle.solve.updates
        && candidate.solve.lambda == oracle.solve.lambda
        && candidate.ghost_pressure_root == oracle.ghost_pressure_root
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
        for (std::size_t index = 0U;
             index < candidate_sealed.pressure_force.size(); ++index) {
            result.pressure_impulse -= options.profile.dt
                * candidate_sealed.pressure_force[index];
            result.contact_impulse +=
                candidate_sealed.contact.impulse[index];
        }
        result.top_contacts += candidate_sealed.contact.top_hits;
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
            state, result.position, result.velocity, &result.work);
        result.velocity_root = id_vec_root(
            "nextengine.nonlocal.ncgp13.velocity.v1", state,
            result.velocity, &result.work);
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
    result.work.portable_fields += 6U + 2U * result.candidate.size();
    put_string(bytes, work_root(result.work));
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

Work15 oracle_expected_nonpressure_work(const State15& state,
    const std::vector<Vec3l>& positions, const SolverOptions15& options,
    bool need_gradient) {
    Work15 expected;
    const std::size_t count = state.dynamic.size();
    ++expected.graph_builds;
    ++expected.objective_evaluations;
    if (need_gradient) ++expected.gradient_evaluations;
    for (std::size_t owner = 0U; owner < count; ++owner) {
        for (std::size_t neighbor = 0U; neighbor < count; ++neighbor) {
            ++expected.oracle_pair_tests;
            const long double radius = nextengine::nonlocal::ncgp15::norm15(
                positions[owner] - positions[neighbor]);
            if (radius > options.profile.horizon) continue;
            ++expected.accepted_pairs;
            ++expected.density_pairs;
            if (need_gradient && neighbor != owner && radius > 0.0L) {
                ++expected.jacobian_products;
            }
        }
        for (const auto& ghost : state.ghosts) {
            ++expected.oracle_pair_tests;
            const long double radius = nextengine::nonlocal::ncgp15::norm15(
                positions[owner] - ghost.position);
            if (radius > options.profile.horizon) continue;
            ++expected.accepted_pairs;
            ++expected.density_pairs;
            if (need_gradient && radius > 0.0L) {
                ++expected.jacobian_products;
            }
        }
    }
    const std::uint64_t unique_pairs = count * (count - 1U) / 2U;
    if (options.terms.normal_viscosity
            || options.terms.tangential_viscosity) {
        expected.oracle_pair_tests += unique_pairs;
        for (std::size_t first = 0U; first < count; ++first) {
            for (std::size_t second = first + 1U; second < count; ++second) {
                const Vec3l difference = options.mutation
                        == nextengine::nonlocal::ncgp15::Mutation15::CurrentReferenceViscosityGraph
                    ? positions[first] - positions[second]
                    : state.dynamic[first].position
                        - state.dynamic[second].position;
                if (nextengine::nonlocal::ncgp15::norm15(difference)
                        <= options.profile.horizon) {
                    ++expected.viscosity_pairs;
                }
            }
        }
    }
    if (options.terms.surface) {
        expected.surface_pair_tests += unique_pairs;
        expected.oracle_pair_tests += unique_pairs;
        for (std::size_t first = 0U; first < count; ++first) {
            for (std::size_t second = first + 1U; second < count; ++second) {
                const long double radius =
                    nextengine::nonlocal::ncgp15::norm15(
                        positions[first] - positions[second]);
                if (radius > 0.0L
                        && radius < 3.0L * options.profile.surface_r0) {
                    ++expected.surface_active_pairs;
                }
            }
        }
    }
    return expected;
}

std::string nonpressure_root(std::string_view route, const State15& state,
    const EnergyGradient15& value, const Work15& value_work,
    Work16* work = nullptr) {
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
    if (work != nullptr) {
        work->portable_fields += 31U + 10U * value.gradient.size();
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
        "CSR_CANDIDATE", state, result.candidate, result.candidate_work,
        &work);
    result.oracle_root = nonpressure_root(
        "ALL_PAIRS_ORACLE", state, result.oracle, result.oracle_work,
        &work);
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
        "nextengine.nonlocal.ncgp16.outer-input.v1", state, y,
        &result.work);
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
        "nextengine.nonlocal.ncgp16.outer-proposal.v1", state, proposal,
        &result.work);
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
        result.pressure.position, &result.work);
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
    result.work.portable_fields += 12U;
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

State15 canonical_state(State15 state, Work16& work) {
    std::sort(state.dynamic.begin(), state.dynamic.end(),
        [](const auto& lhs, const auto& rhs) { return lhs.id < rhs.id; });
    std::sort(state.ghosts.begin(), state.ghosts.end(),
        [](const auto& lhs, const auto& rhs) { return lhs.id < rhs.id; });
    work.canonicalized_records += state.dynamic.size() + state.ghosts.size();
    return state;
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
    long double trajectory_position_rms = 0.0L;
    long double trajectory_position_maximum = 0.0L;
    long double trajectory_velocity_rms = 0.0L;
    long double trajectory_velocity_maximum = 0.0L;
    long double correspondence_position_rms = 0.0L;
    long double correspondence_position_maximum = 0.0L;
    std::uint32_t position_maximum_id = 0U;
    std::uint32_t velocity_maximum_id = 0U;
    Vec3l position_maximum_value;
    Vec3l velocity_maximum_value;
    long double momentum_residual = 0.0L;
    long double energy_excess = 0.0L;
    long double pressure_force_closure = 0.0L;
    long double viscosity_force_closure = 0.0L;
    long double surface_force_closure = 0.0L;
    std::uint64_t components = 0U;
    std::uint64_t satellites = 0U;
    std::array<std::uint64_t, 5> support{};
    std::uint64_t top_contacts = 0U;
    Vec3l external_impulse;
    std::string first_failed_gate;
    std::string private_state_root;
    std::string private_velocity_root;
    bool work_exact = false;
    Work16 expected_work;
    std::string expected_work_root;
    std::string actual_work_root;
    std::string result_root;
    Work16 work;
};

void seal_step(Step16& step) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp16.step.v1");
    put_string(bytes, step.mask);
    put_string(bytes, step.outcome);
    put_u32(bytes, step.accepted ? 1U : 0U);
    put_u64(bytes, step.outer.size());
    for (const Outer16& evaluation : step.outer) {
        put_string(bytes, evaluation.result_root);
    }
    put_string(bytes, step.state_root);
    put_string(bytes, step.velocity_root);
    put_f64(bytes, step.trajectory_position_rms);
    put_f64(bytes, step.trajectory_position_maximum);
    put_f64(bytes, step.trajectory_velocity_rms);
    put_f64(bytes, step.trajectory_velocity_maximum);
    put_f64(bytes, step.correspondence_position_rms);
    put_f64(bytes, step.correspondence_position_maximum);
    put_u32(bytes, step.position_maximum_id);
    put_u32(bytes, step.velocity_maximum_id);
    for (const Vec3l value : std::array<Vec3l, 2>{
             step.position_maximum_value, step.velocity_maximum_value}) {
        put_f64(bytes, value.x);
        put_f64(bytes, value.y);
        put_f64(bytes, value.z);
    }
    put_f64(bytes, step.momentum_residual);
    put_f64(bytes, step.energy_excess);
    put_f64(bytes, step.pressure_force_closure);
    put_f64(bytes, step.viscosity_force_closure);
    put_f64(bytes, step.surface_force_closure);
    put_u64(bytes, step.components);
    put_u64(bytes, step.satellites);
    for (const std::uint64_t value : step.support) put_u64(bytes, value);
    put_u64(bytes, step.top_contacts);
    put_string(bytes, step.first_failed_gate);
    put_string(bytes, step.private_state_root);
    put_string(bytes, step.private_velocity_root);
    put_u32(bytes, step.work_exact ? 1U : 0U);
    put_string(bytes, step.expected_work_root);
    put_string(bytes, step.actual_work_root);
    step.result_root = root(std::move(bytes));
}

long double mechanical_energy(const State15& state,
    const SolverOptions15& options, Work16& work) {
    std::vector<long double> terms;
    terms.reserve(state.dynamic.size());
    for (const auto& sample : state.dynamic) {
        terms.push_back(0.5L * options.profile.mass
                * nextengine::nonlocal::ncgp15::dot15(
                    sample.velocity, sample.velocity)
            - options.profile.mass
                * nextengine::nonlocal::ncgp15::dot15(
                    options.profile.gravity, sample.position));
    }
    SolverOptions15 energy_options = options;
    energy_options.terms.density_constraint = false;
    energy_options.terms.normal_viscosity = false;
    energy_options.terms.tangential_viscosity = false;
    const std::vector<Vec3l> positions = [&]() {
        std::vector<Vec3l> values;
        values.reserve(state.dynamic.size());
        for (const auto& sample : state.dynamic) values.push_back(sample.position);
        return values;
    }();
    const std::vector<long double> zero(state.dynamic.size(), 0.0L);
    Work15 energy_work;
    const EnergyGradient15 energy =
        nextengine::nonlocal::ncgp15::oracle_energy_gradient15(
            state, positions, zero, energy_options,
            MultiplierMode15::FinalKkt, false, energy_work);
    add_nonpressure_work(work, energy_work);
    if (!energy.valid) return std::numeric_limits<long double>::infinity();
    return pairwise_sum(std::move(terms)) + energy.surface_energy;
}

struct Topology16 {
    std::uint64_t components = 0U;
    std::uint64_t satellites = 0U;
    std::array<std::uint64_t, 5> support{};
};

Topology16 topology_and_support(const State15& state,
    const SolverOptions15& options, Work16& work) {
    Topology16 result;
    std::vector<Vec3l> positions;
    positions.reserve(state.dynamic.size());
    for (const auto& sample : state.dynamic) positions.push_back(sample.position);
    const Graph16 graph = build_graph(state, positions, options, false, work);
    if (!graph.valid) return result;
    std::vector<std::size_t> parent(state.dynamic.size());
    std::iota(parent.begin(), parent.end(), 0U);
    const auto find = [&](std::size_t input) {
        std::size_t value = input;
        while (parent[value] != value) value = parent[value];
        return value;
    };
    for (std::size_t owner = 0U; owner < graph.rows.size(); ++owner) {
        for (const Neighbor16 edge : graph.rows[owner]) {
            if (edge.ghost) {
                const Vec3l ghost = state.ghosts[edge.index].position;
                if (ghost.x < 0.0L) ++result.support[0];
                if (ghost.x > options.profile.basin.x) ++result.support[1];
                if (ghost.y < 0.0L) ++result.support[2];
                if (ghost.y > options.profile.basin.y) ++result.support[3];
                if (ghost.z < 0.0L) ++result.support[4];
                continue;
            }
            if (edge.index == owner) continue;
            const std::size_t lhs = find(owner);
            const std::size_t rhs = find(edge.index);
            if (lhs != rhs) parent[rhs] = lhs;
        }
    }
    std::map<std::size_t, std::size_t> sizes;
    for (std::size_t index = 0U; index < parent.size(); ++index) {
        ++sizes[find(index)];
    }
    result.components = sizes.size();
    std::size_t largest = 0U;
    for (const auto& item : sizes) largest = std::max(largest, item.second);
    result.satellites = parent.size() - largest;
    return result;
}

bool one_step_gate(Step16& step, const State15& prior) {
    if (!step.accepted || step.outer.empty()) return false;
    const Outer16& final_outer = step.outer.back();
    if (final_outer.pressure.candidate.empty()
            || final_outer.pressure.oracle.empty()) {
        step.accepted = false;
        step.apparatus_valid = false;
        step.outcome = "ONE_STEP_EVIDENCE_INVALID";
        step.state = prior;
        step.state_root.clear();
        step.velocity_root.clear();
        return false;
    }
    const SolverOptions15 options = make_options(
        step.mask == "P" ? TermMask15{false, false, false, false}
        : step.mask == "PV" ? TermMask15{false, true, false, false}
        : step.mask == "PS" ? TermMask15{false, false, false, true}
                              : TermMask15{false, true, false, true});
    const std::vector<Vec3l>& oracle =
        final_outer.pressure.oracle.back().position;
    std::vector<long double> difference_squares;
    difference_squares.reserve(step.state.dynamic.size());
    for (std::size_t index = 0U; index < step.state.dynamic.size(); ++index) {
        const long double difference =
            nextengine::nonlocal::ncgp15::norm15(
                step.state.dynamic[index].position - oracle[index]);
        difference_squares.push_back(difference * difference);
        step.correspondence_position_maximum = std::max(
            step.correspondence_position_maximum, difference);
    }
    step.correspondence_position_rms = std::sqrt(
        pairwise_sum(std::move(difference_squares))
        / static_cast<long double>(step.state.dynamic.size()));
    for (const PressureRound16& round : final_outer.pressure.candidate) {
        std::vector<Vec3l> all_pressure = round.pressure_force;
        all_pressure.insert(all_pressure.end(),
            round.ghost_pressure_force.begin(),
            round.ghost_pressure_force.end());
        step.pressure_force_closure = std::max(
            step.pressure_force_closure,
            force_closure(std::move(all_pressure), step.work));
    }
    step.viscosity_force_closure = force_closure(
        final_outer.nonpressure.candidate.viscosity_force, step.work);
    step.surface_force_closure = force_closure(
        final_outer.nonpressure.candidate.surface_force, step.work);
    const Topology16 topology = topology_and_support(
        step.state, options, step.work);
    step.components = topology.components;
    step.satellites = topology.satellites;
    step.support = topology.support;
    step.top_contacts = final_outer.pressure.top_contacts;
    const auto& gates = options.profile.gates;
    constexpr std::array<std::string_view, 10> names{
        "ONE_STEP_POSITION_RMS", "ONE_STEP_POSITION_MAXIMUM",
        "PRESSURE_FORCE_CLOSURE", "VISCOSITY_FORCE_CLOSURE",
        "SURFACE_FORCE_CLOSURE", "TOPOLOGY_COMPONENTS",
        "TOPOLOGY_SATELLITES", "TOP_CONTACT", "GHOST_SUPPORT",
        "FINITE_OBSERVABLES"};
    const std::array<bool, 10> predicates{
        step.correspondence_position_rms
            <= gates.one_step_position_rms_m,
        step.correspondence_position_maximum
            <= gates.one_step_position_maximum_m,
        step.pressure_force_closure <= gates.internal_force_closure,
        step.viscosity_force_closure <= gates.internal_force_closure,
        step.surface_force_closure <= gates.internal_force_closure,
        step.components == 1U, step.satellites == 0U,
        step.top_contacts == 0U,
        std::all_of(step.support.begin(), step.support.end(),
            [](std::uint64_t value) { return value > 0U; }),
        std::isfinite(step.correspondence_position_rms)
            && std::isfinite(step.correspondence_position_maximum)
            && std::isfinite(step.pressure_force_closure)
            && std::isfinite(step.viscosity_force_closure)
            && std::isfinite(step.surface_force_closure)};
    step.work.physical_predicates += predicates.size();
    bool pass = true;
    for (std::size_t index = 0U; index < predicates.size(); ++index) {
        if (!predicates[index] && step.first_failed_gate.empty()) {
            step.first_failed_gate = names[index];
        }
        pass = predicates[index] && pass;
    }
    if (!pass) {
        step.accepted = false;
        step.apparatus_valid = true;
        step.outcome = "ONE_STEP_PHYSICAL_REJECTED";
        step.state = prior;
        step.state_root.clear();
        step.velocity_root.clear();
    }
    return pass;
}

bool trajectory_gate(Step16& step, const State15& origin,
    const State15& prior, const Vec3l& accumulated_external,
    std::size_t one_based_step) {
    if (!step.accepted || step.outer.empty()) return false;
    const SolverOptions15 options = make_options(
        TermMask15{false, true, false, true});
    std::vector<long double> position_squares;
    std::vector<long double> velocity_squares;
    position_squares.reserve(step.state.dynamic.size());
    velocity_squares.reserve(step.state.dynamic.size());
    for (std::size_t index = 0U; index < step.state.dynamic.size(); ++index) {
        const long double position = nextengine::nonlocal::ncgp15::norm15(
            step.state.dynamic[index].position
            - origin.dynamic[index].position);
        const long double velocity = nextengine::nonlocal::ncgp15::norm15(
            step.state.dynamic[index].velocity
            - origin.dynamic[index].velocity);
        position_squares.push_back(position * position);
        velocity_squares.push_back(velocity * velocity);
        if (position > step.trajectory_position_maximum) {
            step.trajectory_position_maximum = position;
            step.position_maximum_id = step.state.dynamic[index].id;
            step.position_maximum_value = step.state.dynamic[index].position;
        }
        if (velocity > step.trajectory_velocity_maximum) {
            step.trajectory_velocity_maximum = velocity;
            step.velocity_maximum_id = step.state.dynamic[index].id;
            step.velocity_maximum_value = step.state.dynamic[index].velocity;
        }
    }
    const long double count = static_cast<long double>(
        step.state.dynamic.size());
    step.trajectory_position_rms = std::sqrt(
        pairwise_sum(std::move(position_squares)) / count);
    step.trajectory_velocity_rms = std::sqrt(
        pairwise_sum(std::move(velocity_squares)) / count);
    const Outer16& final_outer = step.outer.back();
    step.external_impulse = final_outer.pressure.pressure_impulse
        + final_outer.pressure.contact_impulse
        + options.profile.dt * count * options.profile.mass
            * options.profile.gravity;
    std::vector<Vec3l> momentum_terms;
    momentum_terms.reserve(step.state.dynamic.size());
    std::vector<Vec3l> prior_momentum_terms;
    prior_momentum_terms.reserve(origin.dynamic.size());
    for (std::size_t index = 0U; index < step.state.dynamic.size(); ++index) {
        momentum_terms.push_back(options.profile.mass
            * step.state.dynamic[index].velocity);
        prior_momentum_terms.push_back(options.profile.mass
            * origin.dynamic[index].velocity);
    }
    const auto sum_vec = [](std::vector<Vec3l> values) {
        if (values.empty()) return Vec3l{};
        while (values.size() > 1U) {
            std::vector<Vec3l> next;
            next.reserve((values.size() + 1U) / 2U);
            for (std::size_t index = 0U; index < values.size(); index += 2U) {
                next.push_back(index + 1U < values.size()
                        ? values[index] + values[index + 1U]
                        : values[index]);
            }
            values = std::move(next);
        }
        return values.front();
    };
    const Vec3l momentum_change = sum_vec(std::move(momentum_terms))
        - sum_vec(std::move(prior_momentum_terms));
    const Vec3l total_external = accumulated_external
        + step.external_impulse;
    const long double denominator = std::max({
        nextengine::nonlocal::ncgp15::norm15(momentum_change),
        nextengine::nonlocal::ncgp15::norm15(total_external),
        options.profile.spacing * count * options.profile.mass
            / options.profile.dt, 1.0e-30L});
    step.momentum_residual = nextengine::nonlocal::ncgp15::norm15(
        momentum_change - total_external) / denominator;
    const long double initial_energy = mechanical_energy(origin, options,
        step.work);
    const long double current_energy = mechanical_energy(
        step.state, options, step.work);
    step.energy_excess = std::max(current_energy - initial_energy, 0.0L)
        / std::max(std::fabs(initial_energy), 1.0L);
    for (const PressureRound16& round : final_outer.pressure.candidate) {
        std::vector<Vec3l> all_pressure = round.pressure_force;
        all_pressure.insert(all_pressure.end(),
            round.ghost_pressure_force.begin(),
            round.ghost_pressure_force.end());
        step.pressure_force_closure = std::max(
            step.pressure_force_closure,
            force_closure(std::move(all_pressure), step.work));
    }
    step.viscosity_force_closure = force_closure(
        final_outer.nonpressure.candidate.viscosity_force, step.work);
    step.surface_force_closure = force_closure(
        final_outer.nonpressure.candidate.surface_force, step.work);
    const Topology16 topology = topology_and_support(
        step.state, options, step.work);
    step.components = topology.components;
    step.satellites = topology.satellites;
    step.support = topology.support;
    step.top_contacts = final_outer.pressure.top_contacts;
    const auto& gates = options.profile.gates;
    constexpr std::array<std::string_view, 15> names{
        "POSITION_RMS", "POSITION_MAXIMUM", "VELOCITY_RMS",
        "VELOCITY_MAXIMUM", "MOMENTUM_RESIDUAL", "ENERGY_EXCESS",
        "PRESSURE_FORCE_CLOSURE", "VISCOSITY_FORCE_CLOSURE",
        "SURFACE_FORCE_CLOSURE", "TOPOLOGY_COMPONENTS",
        "TOPOLOGY_SATELLITES", "TOP_CONTACT", "GHOST_SUPPORT",
        "STEP_INDEX", "FINITE_OBSERVABLES"};
    const std::array<bool, 15> predicates{
        step.trajectory_position_rms <= gates.trajectory_position_rms_m,
        step.trajectory_position_maximum
            <= gates.trajectory_position_maximum_m,
        step.trajectory_velocity_rms <= gates.trajectory_velocity_rms_mps,
        step.trajectory_velocity_maximum
            <= gates.trajectory_velocity_maximum_mps,
        step.momentum_residual <= gates.momentum_residual,
        step.energy_excess <= gates.energy_excess,
        step.pressure_force_closure <= gates.internal_force_closure,
        step.viscosity_force_closure <= gates.internal_force_closure,
        step.surface_force_closure <= gates.internal_force_closure,
        step.components == 1U, step.satellites == 0U,
        step.top_contacts == 0U,
        std::all_of(step.support.begin(), step.support.end(),
            [](std::uint64_t value) { return value > 0U; }),
        one_based_step <= kTrajectorySteps,
        std::isfinite(step.momentum_residual)
            && std::isfinite(step.energy_excess)
            && std::isfinite(step.pressure_force_closure)
            && std::isfinite(step.viscosity_force_closure)
            && std::isfinite(step.surface_force_closure)};
    step.work.physical_predicates += predicates.size();
    bool pass = true;
    for (std::size_t index = 0U; index < predicates.size(); ++index) {
        if (!predicates[index] && step.first_failed_gate.empty()) {
            step.first_failed_gate = names[index];
        }
        pass = predicates[index] && pass;
    }
    if (!pass) {
        step.accepted = false;
        step.apparatus_valid = true;
        step.outcome = "TRAJECTORY_PHYSICAL_REJECTED";
        step.state = prior;
        step.state_root.clear();
        step.velocity_root.clear();
    }
    return pass;
}

Step16 run_step(const State15& prior, std::string mask,
    const TermMask15& terms, bool require_parent_lineage) {
    Step16 result;
    result.mask = std::move(mask);
    const State15 accepted = canonical_state(prior, result.work);
    result.state = accepted;
    const SolverOptions15 options = make_options(terms);
    std::vector<Vec3l> predicted;
    predicted.reserve(accepted.dynamic.size());
    for (const auto& sample : accepted.dynamic) {
        predicted.push_back(sample.position + options.profile.dt
            * (sample.velocity + options.profile.dt
                * options.profile.gravity));
    }
    std::vector<Vec3l> y = project_box(predicted, options, result.work);
    for (std::size_t outer = 0U; outer < kMaximumOuterMaps; ++outer) {
        Outer16 evaluation = outer_map(
            accepted, y, options, static_cast<std::uint32_t>(outer + 1U));
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
                    (y[index] - accepted.dynamic[index].position)
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
    if (result.accepted) {
        result.private_state_root = result.state_root;
        result.private_velocity_root = result.velocity_root;
    }
    return result;
}

Work16 expected_box_work(const std::vector<Vec3l>& values,
    const SolverOptions15& options) {
    Work16 expected;
    const long double low = 0.5L * options.profile.spacing;
    const Vec3l high{options.profile.basin.x - low,
        options.profile.basin.y - low, options.profile.basin.z - low};
    for (const Vec3l value : values) {
        expected.box_plane_tests += 6U;
        if (value.x < low || value.x > high.x) ++expected.clamp_hits;
        if (value.y < low || value.y > high.y) ++expected.clamp_hits;
        if (value.z < low || value.z > high.z) ++expected.clamp_hits;
    }
    return expected;
}

Work16 expected_nonpressure_work(const State15& state,
    const std::vector<Vec3l>& position, const SolverOptions15& options) {
    Work16 expected;
    const std::vector<long double> zero(position.size(), 0.0L);
    const Work15 candidate =
        nextengine::nonlocal::ncgp15::candidate_expected_energy_gradient_work15(
            state, position, zero, options, true);
    const Work15 oracle = oracle_expected_nonpressure_work(
        state, position, options, true);
    add_nonpressure_work(expected, candidate);
    add_nonpressure_work(expected, oracle);
    expected.correspondence_predicates += 6U;
    expected.portable_fields += 2U * (31U + 10U * position.size());
    expected.hash_derivations += 2U;
    return expected;
}

Work16 expected_pressure_round_work(const State15& state,
    const std::vector<Vec3l>& input, const SolverOptions15& options,
    bool all_pairs, const PressureRound16& sealed) {
    Work16 expected = expected_assembly_work(
        state, input, options, all_pairs, sealed.assembly);
    ++expected.pressure_rounds;
    expected.correspondence_predicates += 4U;
    expected.qp_sweeps += sealed.solve.sweeps;
    expected.qp_row_updates += sealed.solve.updates;
    expected.gradient_recomputations += 1U + sealed.solve.sweeps;
    if (!sealed.solve.converged) return expected;
    const std::size_t count = input.size();
    expected.jacobian_products += count * 3U * count;
    for (std::size_t owner = 0U;
         owner < sealed.assembly.graph.rows.size(); ++owner) {
        for (const Neighbor16 edge : sealed.assembly.graph.rows[owner]) {
            if (!edge.ghost) continue;
            const long double radius = nextengine::nonlocal::ncgp15::norm15(
                input[owner] - state.ghosts[edge.index].position);
            if (radius > 0.0L) expected.jacobian_products += 3U;
        }
    }
    for (const std::uint32_t mask : sealed.contact.mask) {
        expected.clamp_hits += static_cast<std::uint64_t>(
            (mask & 1U) != 0U) + static_cast<std::uint64_t>(
            (mask & 2U) != 0U) + static_cast<std::uint64_t>(
            (mask & 4U) != 0U) + static_cast<std::uint64_t>(
            (mask & 8U) != 0U) + static_cast<std::uint64_t>(
            (mask & 16U) != 0U) + static_cast<std::uint64_t>(
            (mask & 32U) != 0U);
    }
    expected.box_plane_tests += 6U * count;
    expected.contact_reaction_values += 3U * count;
    const Work16 post_graph = expected_graph_work(
        state, sealed.position, options, all_pairs);
    const std::uint64_t post_edges = post_graph.accepted_dynamic_pairs
        + post_graph.accepted_ghost_pairs;
    add_work(expected, post_graph);
    expected.density_pairs += post_edges;
    expected.physical_predicates += 7U;
    expected.portable_fields += (2U + 2U * count)
        + (2U + 4U * count) + (2U + 4U * state.ghosts.size())
        + (2U + 2U * count) + (2U + 2U * count)
        + (2U + 4U * count) + 23U;
    expected.hash_derivations += 6U;
    return expected;
}

Work16 expected_pressure_project_work(const State15& state,
    const std::vector<Vec3l>& proposal, const SolverOptions15& options,
    const PressureProject16& sealed) {
    Work16 expected;
    std::vector<Vec3l> candidate_position = proposal;
    std::vector<Vec3l> oracle_position = proposal;
    for (std::size_t index = 0U; index < sealed.candidate.size(); ++index) {
        add_work(expected, expected_pressure_round_work(state,
            candidate_position, options, false, sealed.candidate[index]));
        add_work(expected, expected_pressure_round_work(state,
            oracle_position, options, true, sealed.oracle[index]));
        expected.correspondence_predicates += 15U;
        candidate_position = sealed.candidate[index].position;
        oracle_position = sealed.oracle[index].position;
    }
    if (sealed.accepted) {
        expected.portable_fields += (2U + 10U * state.dynamic.size())
            + (2U + 4U * state.dynamic.size());
        expected.hash_derivations += 2U;
    }
    expected.portable_fields += 6U + 2U * sealed.candidate.size();
    return expected;
}

Work16 expected_outer_work(const State15& state,
    const std::vector<Vec3l>& y, const SolverOptions15& options,
    const Outer16& sealed) {
    Work16 expected;
    ++expected.outer_map_evaluations;
    expected.portable_fields += 2U + 4U * y.size();
    ++expected.hash_derivations;
    add_work(expected, expected_nonpressure_work(state, y, options));
    const long double scale = options.profile.dt * options.profile.dt
        / options.profile.mass;
    std::vector<Vec3l> raw(y.size());
    for (std::size_t index = 0U; index < y.size(); ++index) {
        raw[index] = y[index]
            - scale * sealed.nonpressure.candidate.gradient[index];
    }
    add_work(expected, expected_box_work(raw, options));
    expected.portable_fields += 2U + 4U * y.size();
    ++expected.hash_derivations;
    std::vector<Vec3l> proposal = raw;
    const long double low = 0.5L * options.profile.spacing;
    const Vec3l high{options.profile.basin.x - low,
        options.profile.basin.y - low, options.profile.basin.z - low};
    for (Vec3l& value : proposal) {
        value.x = std::min(std::max(value.x, low), high.x);
        value.y = std::min(std::max(value.y, low), high.y);
        value.z = std::min(std::max(value.z, low), high.z);
    }
    add_work(expected, expected_pressure_project_work(
        state, proposal, options, sealed.pressure));
    if (sealed.pressure.accepted) {
        expected.fixed_point_predicates += 2U;
        expected.portable_fields += 2U + 4U * y.size();
        ++expected.hash_derivations;
        expected.portable_fields += 12U;
    }
    return expected;
}

enum class StepGate16 { None, OneStep, Trajectory };

State15 private_trial_state(const State15& prior, const Step16& sealed,
    const SolverOptions15& options) {
    State15 trial = prior;
    if (sealed.outer.empty()
            || !sealed.outer.back().pressure.accepted) {
        return trial;
    }
    const std::vector<Vec3l>& position =
        sealed.outer.back().pressure.position;
    for (std::size_t index = 0U; index < trial.dynamic.size(); ++index) {
        trial.dynamic[index].position = position[index];
        trial.dynamic[index].velocity =
            (position[index] - prior.dynamic[index].position)
            / options.profile.dt;
    }
    return trial;
}

Work16 expected_gate_work(const State15& prior, const State15& origin,
    const SolverOptions15& options, const Step16& sealed,
    StepGate16 gate) {
    Work16 expected;
    if (gate == StepGate16::None || sealed.outer.empty()
            || !sealed.outer.back().pressure.accepted) {
        return expected;
    }
    const State15 trial = private_trial_state(prior, sealed, options);
    const Outer16& final_outer = sealed.outer.back();
    if (gate == StepGate16::Trajectory) {
        SolverOptions15 energy_options = options;
        energy_options.terms.density_constraint = false;
        energy_options.terms.normal_viscosity = false;
        energy_options.terms.tangential_viscosity = false;
        const auto positions = [](const State15& state) {
            std::vector<Vec3l> result;
            result.reserve(state.dynamic.size());
            for (const auto& sample : state.dynamic) {
                result.push_back(sample.position);
            }
            return result;
        };
        add_nonpressure_work(expected, oracle_expected_nonpressure_work(
            origin, positions(origin), energy_options, false));
        add_nonpressure_work(expected, oracle_expected_nonpressure_work(
            trial, positions(trial), energy_options, false));
    }
    for (const PressureRound16& round : final_outer.pressure.candidate) {
        expected.physical_predicates += round.pressure_force.size()
            + round.ghost_pressure_force.size();
    }
    expected.physical_predicates +=
        final_outer.nonpressure.candidate.viscosity_force.size()
        + final_outer.nonpressure.candidate.surface_force.size();
    const std::vector<Vec3l> trial_position = [&]() {
        std::vector<Vec3l> result;
        result.reserve(trial.dynamic.size());
        for (const auto& sample : trial.dynamic) {
            result.push_back(sample.position);
        }
        return result;
    }();
    add_work(expected, expected_graph_work(
        trial, trial_position, options, false));
    expected.physical_predicates += gate == StepGate16::OneStep ? 10U : 15U;
    return expected;
}

Work16 expected_step_work(const State15& prior, const State15& origin,
    const TermMask15& terms, bool require_parent_lineage,
    StepGate16 gate, const Step16& sealed) {
    Work16 expected;
    const SolverOptions15 options = make_options(terms);
    State15 accepted = prior;
    std::sort(accepted.dynamic.begin(), accepted.dynamic.end(),
        [](const auto& lhs, const auto& rhs) { return lhs.id < rhs.id; });
    std::sort(accepted.ghosts.begin(), accepted.ghosts.end(),
        [](const auto& lhs, const auto& rhs) { return lhs.id < rhs.id; });
    expected.canonicalized_records += accepted.dynamic.size()
        + accepted.ghosts.size();
    std::vector<Vec3l> predicted;
    predicted.reserve(accepted.dynamic.size());
    for (const auto& sample : accepted.dynamic) {
        predicted.push_back(sample.position + options.profile.dt
            * (sample.velocity + options.profile.dt
                * options.profile.gravity));
    }
    add_work(expected, expected_box_work(predicted, options));
    std::vector<Vec3l> y = predicted;
    const long double low = 0.5L * options.profile.spacing;
    const Vec3l high{options.profile.basin.x - low,
        options.profile.basin.y - low, options.profile.basin.z - low};
    for (Vec3l& value : y) {
        value.x = std::min(std::max(value.x, low), high.x);
        value.y = std::min(std::max(value.y, low), high.y);
        value.z = std::min(std::max(value.z, low), high.z);
    }
    for (const Outer16& outer : sealed.outer) {
        add_work(expected, expected_outer_work(accepted, y, options, outer));
        if (outer.pressure.accepted) y = outer.pressure.position;
    }
    if (require_parent_lineage && !sealed.outer.empty()) {
        expected.correspondence_predicates += 4U;
    }
    add_work(expected, expected_gate_work(
        accepted, origin, options, sealed, gate));
    expected.portable_fields += 39U + sealed.outer.size();
    return expected;
}

void finalize_step_work(Step16& step, const State15& prior,
    const State15& origin, const TermMask15& terms,
    bool require_parent_lineage, StepGate16 gate) {
    step.work.portable_fields += 39U + step.outer.size();
    step.expected_work = expected_step_work(prior, origin, terms,
        require_parent_lineage, gate, step);
    step.expected_work_root = work_root(step.expected_work);
    step.actual_work_root = work_root(step.work);
    step.work_exact = work_values(step.expected_work)
        == work_values(step.work);
    if (!step.work_exact) {
        step.apparatus_valid = false;
        step.accepted = false;
        step.work_ceiling = false;
        step.outcome = "STEP_WORK_MISMATCH";
        step.first_failed_gate = "WORK_VERIFIER";
        step.state = prior;
        step.state_root.clear();
        step.velocity_root.clear();
    }
}

std::string state_root16(std::string_view domain, const State15& input,
    Work16* work = nullptr) {
    State15 state = input;
    std::sort(state.dynamic.begin(), state.dynamic.end(),
        [](const auto& lhs, const auto& rhs) { return lhs.id < rhs.id; });
    std::sort(state.ghosts.begin(), state.ghosts.end(),
        [](const auto& lhs, const auto& rhs) { return lhs.id < rhs.id; });
    std::string bytes;
    put_string(bytes, domain);
    put_u64(bytes, state.dynamic.size());
    for (const auto& sample : state.dynamic) {
        put_u32(bytes, sample.id);
        for (const Vec3l value : std::array<Vec3l, 3>{sample.reference,
                 sample.position, sample.velocity}) {
            put_f64(bytes, value.x);
            put_f64(bytes, value.y);
            put_f64(bytes, value.z);
        }
    }
    put_u64(bytes, state.ghosts.size());
    for (const auto& ghost : state.ghosts) {
        put_u32(bytes, ghost.id);
        put_f64(bytes, ghost.position.x);
        put_f64(bytes, ghost.position.y);
        put_f64(bytes, ghost.position.z);
    }
    if (work != nullptr) {
        work->portable_fields += 3U + 10U * state.dynamic.size()
            + 4U * state.ghosts.size();
        ++work->hash_derivations;
    }
    return root(std::move(bytes));
}

enum class Admission16 {
    Pass, Empty, Nonfinite, DuplicateId, NonBinary32, Capacity
};

std::string_view admission_name(Admission16 value) {
    switch (value) {
    case Admission16::Pass: return "PASS";
    case Admission16::Empty: return "EMPTY";
    case Admission16::Nonfinite: return "NONFINITE";
    case Admission16::DuplicateId: return "DUPLICATE_ID";
    case Admission16::NonBinary32: return "NON_BINARY32";
    case Admission16::Capacity: return "CAPACITY";
    }
    return "INVALID_ENUM";
}

Admission16 admit(const State15& state, const SolverOptions15& options,
    Work16& work) {
    ++work.route_predicates;
    if (state.dynamic.empty()) return Admission16::Empty;
    std::set<std::uint32_t> ids;
    for (const auto& sample : state.dynamic) {
        ++work.route_predicates;
        if (!ids.insert(sample.id).second) return Admission16::DuplicateId;
        for (const Vec3l value : std::array<Vec3l, 3>{sample.reference,
                 sample.position, sample.velocity}) {
            work.route_predicates += 6U;
            if (!nextengine::nonlocal::ncgp15::finite15(value)) {
                return Admission16::Nonfinite;
            }
            if (static_cast<long double>(static_cast<float>(value.x)) != value.x
                    || static_cast<long double>(static_cast<float>(value.y))
                        != value.y
                    || static_cast<long double>(static_cast<float>(value.z))
                        != value.z) {
                return Admission16::NonBinary32;
            }
        }
    }
    for (const auto& ghost : state.ghosts) {
        ++work.route_predicates;
        if (!ids.insert(ghost.id).second) return Admission16::DuplicateId;
        work.route_predicates += 6U;
        if (!nextengine::nonlocal::ncgp15::finite15(ghost.position)) {
            return Admission16::Nonfinite;
        }
        if (static_cast<long double>(static_cast<float>(ghost.position.x))
                    != ghost.position.x
                || static_cast<long double>(static_cast<float>(ghost.position.y))
                    != ghost.position.y
                || static_cast<long double>(static_cast<float>(ghost.position.z))
                    != ghost.position.z) {
            return Admission16::NonBinary32;
        }
    }
    const std::vector<Vec3l> positions = [&]() {
        std::vector<Vec3l> values;
        values.reserve(state.dynamic.size());
        for (const auto& sample : state.dynamic) values.push_back(sample.position);
        return values;
    }();
    const Graph16 graph = build_graph(state, positions, options, false, work);
    if (graph.capacity) return Admission16::Capacity;
    return graph.valid ? Admission16::Pass : Admission16::Capacity;
}

struct Scratch16 {
    std::vector<std::uint32_t> owners;
    std::vector<std::uint64_t> row_offsets;
    std::vector<std::uint32_t> neighbors;
    std::vector<long double> lambda;
    std::vector<long double> gradient;
    std::vector<std::uint32_t> contact_mask;
};

void clear_scratch(Scratch16& scratch) {
    scratch.owners.clear();
    scratch.row_offsets.clear();
    scratch.neighbors.clear();
    scratch.lambda.clear();
    scratch.gradient.clear();
    scratch.contact_mask.clear();
}

bool empty_scratch(const Scratch16& scratch) {
    return scratch.owners.empty() && scratch.row_offsets.empty()
        && scratch.neighbors.empty() && scratch.lambda.empty()
        && scratch.gradient.empty() && scratch.contact_mask.empty();
}

std::string scratch_root(const Scratch16& scratch, Work16* work = nullptr) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp16.transaction-scratch.v1");
    for (const std::uint64_t size : std::array<std::uint64_t, 6>{
             scratch.owners.size(), scratch.row_offsets.size(),
             scratch.neighbors.size(), scratch.lambda.size(),
             scratch.gradient.size(), scratch.contact_mask.size()}) {
        put_u64(bytes, size);
    }
    for (const std::uint32_t value : scratch.owners) put_u32(bytes, value);
    for (const std::uint64_t value : scratch.row_offsets) put_u64(bytes, value);
    for (const std::uint32_t value : scratch.neighbors) put_u32(bytes, value);
    for (const long double value : scratch.lambda) put_f64(bytes, value);
    for (const long double value : scratch.gradient) put_f64(bytes, value);
    for (const std::uint32_t value : scratch.contact_mask) put_u32(bytes, value);
    if (work != nullptr) {
        work->portable_fields += 7U + scratch.owners.size()
            + scratch.row_offsets.size() + scratch.neighbors.size()
            + scratch.lambda.size() + scratch.gradient.size()
            + scratch.contact_mask.size();
        ++work->hash_derivations;
    }
    return root(std::move(bytes));
}

Scratch16 scratch_from_round(const State15& state,
    const PressureRound16& round) {
    Scratch16 result;
    result.owners.reserve(state.dynamic.size());
    result.row_offsets.push_back(0U);
    for (std::size_t owner = 0U; owner < state.dynamic.size(); ++owner) {
        result.owners.push_back(state.dynamic[owner].id);
        for (const Neighbor16 edge : round.assembly.graph.rows[owner]) {
            result.neighbors.push_back(edge.ghost
                ? state.ghosts[edge.index].id : state.dynamic[edge.index].id);
        }
        result.row_offsets.push_back(result.neighbors.size());
    }
    result.lambda = round.solve.lambda;
    result.gradient = round.solve.gradient;
    result.contact_mask = round.contact.mask;
    return result;
}

struct Control16 {
    std::string name;
    bool pass = false;
    std::vector<std::pair<std::string, bool>> scalars;
    std::vector<std::string> evidence;
    bool work_exact = false;
    Work16 expected_work;
    std::string expected_work_root;
    std::string actual_work_root;
    Work16 work;
    std::string result_root;
};

void seal_control(Control16& control, Work16 expected) {
    const std::uint64_t fields = 9U + 2U * control.scalars.size()
        + control.evidence.size();
    control.work.portable_fields += fields;
    expected.portable_fields += fields;
    control.expected_work = expected;
    control.expected_work_root = work_root(control.expected_work);
    control.actual_work_root = work_root(control.work);
    control.work_exact = work_values(control.expected_work)
        == work_values(control.work);
    control.pass = control.pass && control.work_exact;
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp16.control.v1");
    put_string(bytes, control.name);
    put_u32(bytes, control.pass ? 1U : 0U);
    put_u64(bytes, control.scalars.size());
    for (const auto& scalar : control.scalars) {
        put_string(bytes, scalar.first);
        put_u32(bytes, scalar.second ? 1U : 0U);
    }
    put_u64(bytes, control.evidence.size());
    for (const std::string& evidence : control.evidence) {
        put_string(bytes, evidence);
    }
    put_u32(bytes, control.work_exact ? 1U : 0U);
    put_string(bytes, control.expected_work_root);
    put_string(bytes, control.actual_work_root);
    control.result_root = root(std::move(bytes));
}

Work16 expected_state_root_work(const State15& state) {
    Work16 expected;
    expected.portable_fields += 3U + 10U * state.dynamic.size()
        + 4U * state.ghosts.size();
    ++expected.hash_derivations;
    return expected;
}

Work16 expected_scratch_root_work(const Scratch16& scratch) {
    Work16 expected;
    expected.portable_fields += 7U + scratch.owners.size()
        + scratch.row_offsets.size() + scratch.neighbors.size()
        + scratch.lambda.size() + scratch.gradient.size()
        + scratch.contact_mask.size();
    ++expected.hash_derivations;
    return expected;
}

Work16 expected_admission_work(const State15& state,
    const SolverOptions15& options) {
    Work16 expected;
    ++expected.route_predicates;
    if (state.dynamic.empty()) return expected;
    std::set<std::uint32_t> ids;
    for (const auto& sample : state.dynamic) {
        ++expected.route_predicates;
        if (!ids.insert(sample.id).second) return expected;
        for (const Vec3l value : std::array<Vec3l, 3>{sample.reference,
                 sample.position, sample.velocity}) {
            expected.route_predicates += 6U;
            if (!nextengine::nonlocal::ncgp15::finite15(value)) {
                return expected;
            }
            if (static_cast<long double>(static_cast<float>(value.x)) != value.x
                    || static_cast<long double>(static_cast<float>(value.y))
                        != value.y
                    || static_cast<long double>(static_cast<float>(value.z))
                        != value.z) {
                return expected;
            }
        }
    }
    for (const auto& ghost : state.ghosts) {
        ++expected.route_predicates;
        if (!ids.insert(ghost.id).second) return expected;
        expected.route_predicates += 6U;
        if (!nextengine::nonlocal::ncgp15::finite15(ghost.position)) {
            return expected;
        }
        if (static_cast<long double>(static_cast<float>(ghost.position.x))
                    != ghost.position.x
                || static_cast<long double>(static_cast<float>(ghost.position.y))
                    != ghost.position.y
                || static_cast<long double>(static_cast<float>(ghost.position.z))
                    != ghost.position.z) {
            return expected;
        }
    }
    std::vector<Vec3l> positions;
    positions.reserve(state.dynamic.size());
    for (const auto& sample : state.dynamic) {
        positions.push_back(sample.position);
    }
    ++expected.graph_builds;
    std::map<Cell16, std::vector<std::size_t>> ghost_cells;
    for (std::size_t index = 0U; index < state.ghosts.size(); ++index) {
        ghost_cells[cell_of(state.ghosts[index].position,
            options.profile.horizon)].push_back(index);
    }
    std::uint64_t completed_edges = 0U;
    for (std::size_t owner = 0U; owner < positions.size(); ++owner) {
        std::uint64_t row_edges = 0U;
        for (std::size_t neighbor = 0U; neighbor < positions.size();
             ++neighbor) {
            ++expected.graph_candidates;
            if (nextengine::nonlocal::ncgp15::norm15(
                    positions[owner] - positions[neighbor])
                    <= options.profile.horizon) {
                ++expected.accepted_dynamic_pairs;
                ++row_edges;
            }
        }
        const auto [cx, cy, cz] = cell_of(
            positions[owner], options.profile.horizon);
        std::vector<std::size_t> candidates;
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
        for (const std::size_t ghost : candidates) {
            ++expected.graph_candidates;
            if (nextengine::nonlocal::ncgp15::norm15(
                    positions[owner] - state.ghosts[ghost].position)
                    <= options.profile.horizon) {
                ++expected.accepted_ghost_pairs;
                ++row_edges;
            }
        }
        if (row_edges > options.profile.maximum_neighbors) return expected;
        completed_edges += row_edges;
    }
    expected.portable_fields += 2U + 2U * positions.size()
        + 2U * completed_edges;
    ++expected.hash_derivations;
    return expected;
}

std::vector<Control16> run_controls(const Fixture16& fixture,
    const std::vector<Step16>& masks) {
    std::vector<Control16> controls;
    Control16 lineage;
    lineage.name = "LINEAGE";
    const bool mask_available = !masks.empty() && !masks[0].outer.empty();
    lineage.scalars = {{"fixture_exact",
                           fixture.fixture_root == kParent14Fixture},
        {"lane_exact", fixture.lane_root == kParent14Lane},
        {"first_pressure_exact",
            mask_available && masks[0].lineage_exact},
        {"first_pressure_stationary", mask_available
                && masks[0].outer.size() == 2U
                && masks[0].outer[1].fixed_rms == 0.0L
                && masks[0].outer[1].fixed_maximum == 0.0L}};
    lineage.pass = std::all_of(lineage.scalars.begin(), lineage.scalars.end(),
        [](const auto& value) { return value.second; });
    lineage.evidence = {fixture.fixture_root, fixture.lane_root,
        mask_available ? masks[0].result_root : std::string{}};
    lineage.work.correspondence_predicates += lineage.scalars.size();
    Work16 lineage_expected;
    lineage_expected.correspondence_predicates += 4U;
    seal_control(lineage, lineage_expected);
    controls.push_back(std::move(lineage));

    Control16 correspondence;
    correspondence.name = "CANDIDATE_ORACLE";
    bool all_nonpressure = true;
    bool all_pressure = true;
    for (const Step16& step : masks) {
        for (const Outer16& outer : step.outer) {
            all_nonpressure = outer.nonpressure.valid && all_nonpressure;
            for (std::size_t index = 0U;
                 index < outer.pressure.candidate.size(); ++index) {
                all_pressure = same_round(outer.pressure.candidate[index],
                    outer.pressure.oracle[index], correspondence.work)
                    && all_pressure;
            }
            correspondence.evidence.push_back(outer.result_root);
        }
    }
    correspondence.scalars = {{"nonpressure_exact", all_nonpressure},
        {"pressure_exact", all_pressure}};
    correspondence.work.correspondence_predicates += 2U;
    correspondence.pass = all_nonpressure && all_pressure;
    Work16 correspondence_expected;
    std::uint64_t compared_rounds = 0U;
    for (const Step16& step : masks) {
        for (const Outer16& outer : step.outer) {
            compared_rounds += outer.pressure.candidate.size();
        }
    }
    correspondence_expected.correspondence_predicates =
        15U * compared_rounds + 2U;
    seal_control(correspondence, correspondence_expected);
    controls.push_back(std::move(correspondence));

    Control16 pressure_negatives;
    pressure_negatives.name = "PRESSURE_NEGATIVES";
    Work16 pressure_negatives_expected;
    if (mask_available && !masks[0].outer.empty()
            && !masks[0].outer[0].pressure.candidate.empty()) {
        const PressureRound16& round =
            masks[0].outer[0].pressure.candidate[0];
        const SolverOptions15 pressure_options = make_options(
            TermMask15{false, false, false, false});
        const long double scale = pressure_options.profile.dt
            * pressure_options.profile.dt / pressure_options.profile.mass;
        bool sign_exact = true;
        for (std::size_t index = 0U; index < round.correction.size(); ++index) {
            const Vec3l residual = round.correction[index]
                + scale * round.pressure_force[index];
            const std::array<bool, 3> components{residual.x == 0.0L,
                residual.y == 0.0L, residual.z == 0.0L};
            pressure_negatives.work.correspondence_predicates += 3U;
            pressure_negatives_expected.correspondence_predicates += 3U;
            for (const bool component : components) {
                sign_exact = component && sign_exact;
            }
        }
        bool ghost_present = false;
        for (const Vec3l force : round.ghost_pressure_force) {
            const bool nonzero = nextengine::nonlocal::ncgp15::norm15(force)
                > 0.0L;
            ++pressure_negatives.work.correspondence_predicates;
            ++pressure_negatives_expected.correspondence_predicates;
            ghost_present = nonzero || ghost_present;
        }
        std::vector<Vec3l> closed_force = round.pressure_force;
        closed_force.insert(closed_force.end(),
            round.ghost_pressure_force.begin(),
            round.ghost_pressure_force.end());
        const long double closed = force_closure(
            std::move(closed_force), pressure_negatives.work);
        const long double dynamic_only = force_closure(
            round.pressure_force, pressure_negatives.work);
        pressure_negatives_expected.physical_predicates +=
            round.pressure_force.size() * 2U
            + round.ghost_pressure_force.size();
        pressure_negatives.work.correspondence_predicates += 2U;
        pressure_negatives_expected.correspondence_predicates += 2U;

        State15 accepted = fixture.state;
        std::sort(accepted.dynamic.begin(), accepted.dynamic.end(),
            [](const auto& lhs, const auto& rhs) { return lhs.id < rhs.id; });
        std::sort(accepted.ghosts.begin(), accepted.ghosts.end(),
            [](const auto& lhs, const auto& rhs) { return lhs.id < rhs.id; });
        std::vector<Vec3l> predicted;
        predicted.reserve(accepted.dynamic.size());
        for (const auto& sample : accepted.dynamic) {
            predicted.push_back(sample.position + pressure_options.profile.dt
                * (sample.velocity + pressure_options.profile.dt
                    * pressure_options.profile.gravity));
        }
        Work16 ignored_box;
        std::vector<Vec3l> y = project_box(
            predicted, pressure_options, ignored_box);
        const long double nonpressure_scale = pressure_options.profile.dt
            * pressure_options.profile.dt / pressure_options.profile.mass;
        for (std::size_t index = 0U; index < y.size(); ++index) {
            y[index] -= nonpressure_scale
                * masks[0].outer[0].nonpressure.candidate.gradient[index];
        }
        y = project_box(y, pressure_options, ignored_box);
        std::vector<Vec3l> stale = y;
        const double stale_x = static_cast<double>(stale[0].x);
        stale[0].x = static_cast<long double>(std::nextafter(stale_x,
            std::numeric_limits<double>::infinity()));
        Work16 stale_work;
        const Assembly16 stale_assembly = assemble(
            accepted, stale, pressure_options, false, stale_work);
        add_work(pressure_negatives.work, stale_work);
        add_work(pressure_negatives_expected, expected_assembly_work(
            accepted, stale, pressure_options, false, stale_assembly));
        Work16 cap_work;
        const Solve16 capped = solve_qp(round.assembly, cap_work, 1U);
        add_work(pressure_negatives.work, cap_work);
        pressure_negatives_expected.qp_sweeps += capped.sweeps;
        pressure_negatives_expected.qp_row_updates += capped.updates;
        pressure_negatives_expected.gradient_recomputations +=
            1U + capped.sweeps;
        pressure_negatives.work.correspondence_predicates += 4U;
        pressure_negatives_expected.correspondence_predicates += 4U;
        pressure_negatives.scalars = {{"pressure_sign_exact", sign_exact},
            {"ghost_derivative_present", ghost_present},
            {"ghost_force_closure", closed <= kBalanceLimit},
            {"dynamic_only_exposes_ghost_reaction",
                dynamic_only > closed},
            {"stale_assembly_changes_matrix",
                stale_assembly.matrix_root != round.assembly.matrix_root},
            {"qp_cap_typed", !capped.converged && capped.sweeps == 1U}};
        pressure_negatives.evidence = {round.assembly.graph.root,
            round.assembly.jacobian_root, round.assembly.matrix_root,
            round.ghost_pressure_root, stale_assembly.matrix_root};
    }
    pressure_negatives.pass = pressure_negatives.scalars.size() == 6U
        && std::all_of(pressure_negatives.scalars.begin(),
            pressure_negatives.scalars.end(),
            [](const auto& value) { return value.second; });
    seal_control(pressure_negatives, pressure_negatives_expected);
    controls.push_back(std::move(pressure_negatives));

    Control16 mutations;
    mutations.name = "TERM_MUTATIONS";
    Work16 mutations_expected;
    if (mask_available) {
        const std::vector<Vec3l> y = [&]() {
            std::vector<Vec3l> values;
            values.reserve(fixture.state.dynamic.size());
            for (const auto& sample : fixture.state.dynamic) {
                values.push_back(sample.position);
            }
            return values;
        }();
        const std::vector<long double> zero(y.size(), 0.0L);
        SolverOptions15 base_pressure = make_options(
            TermMask15{false, false, false, false});
        SolverOptions15 missing_chain = base_pressure;
        missing_chain.mutation =
            nextengine::nonlocal::ncgp15::Mutation15::MissingKernelChain;
        Work16 base_assembly_work;
        Work16 missing_assembly_work;
        const Assembly16 base_assembly = assemble(fixture.state, y,
            base_pressure, false, base_assembly_work);
        const Assembly16 missing_assembly = assemble(fixture.state, y,
            missing_chain, false, missing_assembly_work);
        add_work(mutations.work, base_assembly_work);
        add_work(mutations.work, missing_assembly_work);
        add_work(mutations_expected, expected_assembly_work(
            fixture.state, y, base_pressure, false, base_assembly));
        add_work(mutations_expected, expected_assembly_work(
            fixture.state, y, missing_chain, false, missing_assembly));
        mutations.evidence.insert(mutations.evidence.end(),
            {base_assembly.density_root, missing_assembly.density_root,
                base_assembly.jacobian_root, missing_assembly.jacobian_root,
                base_assembly.matrix_root, missing_assembly.matrix_root});
        mutations.scalars.insert(mutations.scalars.end(),
            {{"missing_kernel_preserves_density",
                 base_assembly.density_root == missing_assembly.density_root},
                {"missing_kernel_changes_jacobian",
                    base_assembly.jacobian_root
                        != missing_assembly.jacobian_root},
                {"missing_kernel_changes_matrix",
                    base_assembly.matrix_root
                        != missing_assembly.matrix_root}});
        const auto energy_for = [&](TermMask15 terms,
                                    nextengine::nonlocal::ncgp15::Mutation15 mutation,
                                    MultiplierMode15 mode,
                                    Work15& work) {
            SolverOptions15 options = make_options(terms);
            options.mutation = mutation;
            return nextengine::nonlocal::ncgp15::candidate_energy_gradient15(
                fixture.state, y, zero, options, mode, true, work);
        };
        const std::array<std::pair<std::string,
            nextengine::nonlocal::ncgp15::Mutation15>, 5> cases{
            std::pair{"half_normal_viscosity",
                nextengine::nonlocal::ncgp15::Mutation15::HalfNormalViscosity},
            std::pair{"wrong_surface_sign",
                nextengine::nonlocal::ncgp15::Mutation15::WrongSurfaceSign},
            std::pair{"missing_surface_factor_two",
                nextengine::nonlocal::ncgp15::Mutation15::MissingSurfaceFactorTwo},
            std::pair{"current_reference_graph",
                nextengine::nonlocal::ncgp15::Mutation15::CurrentReferenceViscosityGraph},
            std::pair{"finite_pressure_penalty",
                nextengine::nonlocal::ncgp15::Mutation15::FinitePressurePenalty}};
        for (const auto& item : cases) {
            const bool viscosity = item.second
                    == nextengine::nonlocal::ncgp15::Mutation15::HalfNormalViscosity
                || item.second
                    == nextengine::nonlocal::ncgp15::Mutation15::CurrentReferenceViscosityGraph;
            const bool surface = item.second
                    == nextengine::nonlocal::ncgp15::Mutation15::WrongSurfaceSign
                || item.second
                    == nextengine::nonlocal::ncgp15::Mutation15::MissingSurfaceFactorTwo;
            TermMask15 terms{item.second
                    == nextengine::nonlocal::ncgp15::Mutation15::FinitePressurePenalty,
                viscosity, false, surface};
            Work15 base_work;
            Work15 mutation_work;
            const EnergyGradient15 base = energy_for(terms,
                nextengine::nonlocal::ncgp15::Mutation15::None,
                MultiplierMode15::FinalKkt, base_work);
            const EnergyGradient15 changed = energy_for(terms, item.second,
                item.second
                        == nextengine::nonlocal::ncgp15::Mutation15::FinitePressurePenalty
                    ? MultiplierMode15::AugmentedLagrangian
                    : MultiplierMode15::FinalKkt,
                mutation_work);
            add_nonpressure_work(mutations.work, base_work);
            add_nonpressure_work(mutations.work, mutation_work);
            SolverOptions15 base_options = make_options(terms);
            SolverOptions15 mutation_options = make_options(terms);
            mutation_options.mutation = item.second;
            add_nonpressure_work(mutations_expected,
                nextengine::nonlocal::ncgp15::candidate_expected_energy_gradient_work15(
                    fixture.state, y, zero, base_options, true));
            add_nonpressure_work(mutations_expected,
                nextengine::nonlocal::ncgp15::candidate_expected_energy_gradient_work15(
                    fixture.state, y, zero, mutation_options, true));
            const std::string base_root = nonpressure_root(
                "CONTROL_BASE", fixture.state, base, base_work,
                &mutations.work);
            const std::string changed_root = nonpressure_root(
                "CONTROL_MUTATED", fixture.state, changed, mutation_work,
                &mutations.work);
            mutations.work.hash_derivations += 2U;
            mutations_expected.portable_fields +=
                2U * (31U + 10U * fixture.state.dynamic.size());
            mutations_expected.hash_derivations += 2U;
            mutations.evidence.push_back(base_root);
            mutations.evidence.push_back(changed_root);
            mutations.scalars.push_back(
                {item.first + "_changes_root", base_root != changed_root});
        }
    }
    mutations.pass = !mutations.scalars.empty()
        && std::all_of(mutations.scalars.begin(), mutations.scalars.end(),
            [](const auto& value) { return value.second; });
    mutations.work.correspondence_predicates += mutations.scalars.size();
    mutations_expected.correspondence_predicates += 8U;
    seal_control(mutations, mutations_expected);
    controls.push_back(std::move(mutations));

    Control16 permutation;
    permutation.name = "PERMUTATION";
    State15 permuted = fixture.state;
    std::reverse(permuted.dynamic.begin(), permuted.dynamic.end());
    std::rotate(permuted.ghosts.begin(),
        permuted.ghosts.begin() + permuted.ghosts.size() / 3U,
        permuted.ghosts.end());
    Step16 permuted_p = run_step(permuted, "P",
        TermMask15{false, false, false, false}, true);
    StepGate16 permuted_gate = StepGate16::None;
    if (permuted_p.accepted) {
        static_cast<void>(one_step_gate(permuted_p, permuted));
        permuted_gate = StepGate16::OneStep;
    }
    finalize_step_work(permuted_p, permuted, permuted,
        TermMask15{false, false, false, false}, true, permuted_gate);
    seal_step(permuted_p);
    if (mask_available) {
        permutation.scalars = {
            {"category_exact", permuted_p.outcome == masks[0].outcome},
            {"state_exact", permuted_p.state_root == masks[0].state_root},
            {"velocity_exact",
                permuted_p.velocity_root == masks[0].velocity_root},
            {"work_exact", work_values(permuted_p.work)
                    == work_values(masks[0].work)},
            {"result_exact", permuted_p.result_root == masks[0].result_root}};
        permutation.evidence = {masks[0].result_root,
            permuted_p.result_root};
    }
    permutation.work = permuted_p.work;
    permutation.work.correspondence_predicates += 5U;
    permutation.pass = permutation.scalars.size() == 5U
        && std::all_of(permutation.scalars.begin(), permutation.scalars.end(),
            [](const auto& value) { return value.second; });
    Work16 permutation_expected = permuted_p.expected_work;
    permutation_expected.correspondence_predicates += 5U;
    seal_control(permutation, permutation_expected);
    controls.push_back(std::move(permutation));

    Control16 transaction;
    transaction.name = "TRANSACTION";
    Work16 transaction_expected;
    if (mask_available && !masks[0].outer[0].pressure.candidate.empty()) {
        const std::string prior = state_root16(
            "nextengine.nonlocal.ncgp16.transaction-state.v1",
            fixture.state, &transaction.work);
        Scratch16 scratch = scratch_from_round(fixture.state,
            masks[0].outer[0].pressure.candidate[0]);
        const Scratch16 populated_scratch = scratch;
        const std::string before = scratch_root(scratch, &transaction.work);
        const bool populated = !empty_scratch(scratch);
        clear_scratch(scratch);
        const std::string after = scratch_root(scratch, &transaction.work);
        const Scratch16 canonical_empty;
        const std::string empty = scratch_root(
            canonical_empty, &transaction.work);
        const std::string retained = state_root16(
            "nextengine.nonlocal.ncgp16.transaction-state.v1",
            fixture.state, &transaction.work);
        transaction.scalars = {{"scratch_populated", populated},
            {"scratch_cleared", empty_scratch(scratch)},
            {"empty_root_exact", after == empty},
            {"accepted_state_preserved", retained == prior},
            {"pressure_round_hook", true},
            {"nonpressure_hook", true}, {"fixed_point_hook", true},
            {"work_verifier_hook", true}};
        transaction.evidence = {prior, before, after, empty, retained};
        add_work(transaction_expected,
            expected_state_root_work(fixture.state));
        add_work(transaction_expected,
            expected_scratch_root_work(populated_scratch));
        add_work(transaction_expected,
            expected_scratch_root_work(Scratch16{}));
        add_work(transaction_expected,
            expected_scratch_root_work(Scratch16{}));
        add_work(transaction_expected,
            expected_state_root_work(fixture.state));
    }
    transaction.work.correspondence_predicates += transaction.scalars.size();
    transaction_expected.correspondence_predicates += 8U;
    transaction.pass = transaction.scalars.size() == 8U
        && std::all_of(transaction.scalars.begin(), transaction.scalars.end(),
            [](const auto& value) { return value.second; });
    seal_control(transaction, transaction_expected);
    const std::string transaction_result_root = transaction.result_root;
    controls.push_back(std::move(transaction));

    Control16 invalid;
    invalid.name = "INVALID_INPUT";
    const SolverOptions15 options = make_options(
        TermMask15{false, false, false, false});
    State15 nonfinite = fixture.state;
    nonfinite.dynamic[0].position.x =
        std::numeric_limits<long double>::quiet_NaN();
    State15 duplicate = fixture.state;
    duplicate.dynamic[1].id = duplicate.dynamic[0].id;
    State15 malformed = fixture.state;
    malformed.dynamic[0].position.x += 1.0e-12L;
    State15 capacity = fixture.state;
    for (std::size_t index = 0U; index < 257U; ++index) {
        capacity.ghosts.push_back({static_cast<std::uint32_t>(
                0x90000000U + index), capacity.dynamic[0].position});
    }
    const Admission16 nonfinite_result = admit(nonfinite, options, invalid.work);
    const Admission16 duplicate_result = admit(duplicate, options, invalid.work);
    const Admission16 malformed_result = admit(malformed, options, invalid.work);
    const Admission16 capacity_result = admit(capacity, options, invalid.work);
    invalid.scalars = {
        {std::string(admission_name(nonfinite_result)),
            nonfinite_result == Admission16::Nonfinite},
        {std::string(admission_name(duplicate_result)),
            duplicate_result == Admission16::DuplicateId},
        {std::string(admission_name(malformed_result)),
            malformed_result == Admission16::NonBinary32},
        {std::string(admission_name(capacity_result)),
            capacity_result == Admission16::Capacity}};
    invalid.work.correspondence_predicates += invalid.scalars.size();
    invalid.pass = std::all_of(invalid.scalars.begin(), invalid.scalars.end(),
        [](const auto& value) { return value.second; });
    Work16 invalid_expected;
    add_work(invalid_expected,
        expected_admission_work(nonfinite, options));
    add_work(invalid_expected,
        expected_admission_work(duplicate, options));
    add_work(invalid_expected,
        expected_admission_work(malformed, options));
    add_work(invalid_expected,
        expected_admission_work(capacity, options));
    invalid_expected.correspondence_predicates += 4U;
    seal_control(invalid, invalid_expected);
    controls.push_back(std::move(invalid));

    Control16 roots;
    roots.name = "WORK_ROOT_MUTATIONS";
    Work16 roots_expected;
    if (mask_available) {
        const std::string baseline_work = masks[0].actual_work_root;
        bool counters = true;
        for (std::size_t field = 0U; field < work_values(masks[0].work).size();
             ++field) {
            Work16 changed = masks[0].work;
            std::uint64_t* selected = nullptr;
            std::size_t index = 0U;
            for (std::uint64_t* candidate : std::array<std::uint64_t*, 32>{
                     &changed.fixture_records, &changed.ghost_records,
                     &changed.canonicalized_records, &changed.graph_builds,
                     &changed.graph_candidates, &changed.accepted_dynamic_pairs,
                     &changed.accepted_ghost_pairs, &changed.density_pairs,
                     &changed.derivative_pairs, &changed.jacobian_products,
                     &changed.matrix_products, &changed.symmetry_reductions,
                     &changed.qp_sweeps, &changed.qp_row_updates,
                     &changed.gradient_recomputations, &changed.pressure_rounds,
                     &changed.outer_map_evaluations,
                     &changed.nonpressure_objective_evaluations,
                     &changed.nonpressure_gradient_evaluations,
                     &changed.viscosity_pairs, &changed.surface_tests,
                     &changed.surface_active_pairs, &changed.box_plane_tests,
                     &changed.clamp_hits, &changed.contact_reaction_values,
                     &changed.fixed_point_predicates,
                     &changed.correspondence_predicates,
                     &changed.physical_predicates, &changed.route_predicates,
                     &changed.independent_pair_tests,
                     &changed.portable_fields, &changed.hash_derivations}) {
                if (index++ == field) selected = candidate;
            }
            if (selected == nullptr) throw std::logic_error("work mutation");
            ++*selected;
            const bool changed_root = work_root(changed) != baseline_work;
            ++roots.work.hash_derivations;
            ++roots_expected.hash_derivations;
            ++roots.work.correspondence_predicates;
            ++roots_expected.correspondence_predicates;
            counters = changed_root && counters;
        }
        State15 final_state_mutation = masks[0].state;
        const double final_x = static_cast<double>(
            final_state_mutation.dynamic[0].position.x);
        final_state_mutation.dynamic[0].position.x =
            static_cast<long double>(std::nextafter(final_x,
                std::numeric_limits<double>::infinity()));
        const std::string baseline_state = state_root16(
            "nextengine.nonlocal.ncgp16.control-final-state.v1",
            masks[0].state, &roots.work);
        const std::string mutated_state = state_root16(
            "nextengine.nonlocal.ncgp16.control-final-state.v1",
            final_state_mutation, &roots.work);
        add_work(roots_expected,
            expected_state_root_work(masks[0].state));
        add_work(roots_expected,
            expected_state_root_work(final_state_mutation));
        roots.scalars = {{"all_work_counters_sensitive", counters},
            {"outer_root_sensitive", masks[0].outer[0].result_root
                    != masks[0].outer[1].result_root},
            {"pressure_round_root_sensitive",
                masks[0].outer[0].pressure.candidate[0].result_root
                    != masks[0].outer[0].pressure.candidate[1].result_root},
            {"final_state_sensitive", baseline_state != mutated_state},
            {"term_force_sensitive", masks.size() > 2U
                    && masks[1].outer[0].nonpressure.candidate_root
                        != masks[2].outer[0].nonpressure.candidate_root},
            {"transaction_flag_sensitive",
                transaction_result_root != controls[0].result_root}};
        roots.work.correspondence_predicates += 5U;
        roots_expected.correspondence_predicates += 5U;
        roots.evidence = {baseline_work, masks[0].result_root,
            baseline_state, mutated_state, transaction_result_root};
    }
    roots.pass = roots.scalars.size() == 6U
        && std::all_of(roots.scalars.begin(), roots.scalars.end(),
            [](const auto& value) { return value.second; });
    seal_control(roots, roots_expected);
    controls.push_back(std::move(roots));
    return controls;
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
           << "\",\"trajectory_position_rms_m\":";
    emit_finite(output, step.trajectory_position_rms);
    output << ",\"trajectory_position_maximum_m\":";
    emit_finite(output, step.trajectory_position_maximum);
    output << ",\"trajectory_velocity_rms_mps\":";
    emit_finite(output, step.trajectory_velocity_rms);
    output << ",\"trajectory_velocity_maximum_mps\":";
    emit_finite(output, step.trajectory_velocity_maximum);
    output << ",\"candidate_oracle_position_rms_m\":";
    emit_finite(output, step.correspondence_position_rms);
    output << ",\"candidate_oracle_position_maximum_m\":";
    emit_finite(output, step.correspondence_position_maximum);
    output << ",\"position_maximum_id\":" << step.position_maximum_id
           << ",\"velocity_maximum_id\":" << step.velocity_maximum_id;
    output << ",\"position_maximum_value_m\":[";
    emit_finite(output, step.position_maximum_value.x);
    output << ',';
    emit_finite(output, step.position_maximum_value.y);
    output << ',';
    emit_finite(output, step.position_maximum_value.z);
    output << "],\"velocity_maximum_value_mps\":[";
    emit_finite(output, step.velocity_maximum_value.x);
    output << ',';
    emit_finite(output, step.velocity_maximum_value.y);
    output << ',';
    emit_finite(output, step.velocity_maximum_value.z);
    output << ']';
    output << ",\"momentum_residual\":";
    emit_finite(output, step.momentum_residual);
    output << ",\"energy_excess\":";
    emit_finite(output, step.energy_excess);
    output << ",\"pressure_force_closure\":";
    emit_finite(output, step.pressure_force_closure);
    output << ",\"viscosity_force_closure\":";
    emit_finite(output, step.viscosity_force_closure);
    output << ",\"surface_force_closure\":";
    emit_finite(output, step.surface_force_closure);
    output << ",\"components\":" << step.components
           << ",\"satellites\":" << step.satellites
           << ",\"support\":[";
    for (std::size_t index = 0U; index < step.support.size(); ++index) {
        if (index != 0U) output << ',';
        output << step.support[index];
    }
    output << "],\"top_contacts\":" << step.top_contacts
           << ",\"first_failed_gate\":\"" << step.first_failed_gate << '"'
           << ",\"private_state_root\":\"" << step.private_state_root
           << "\",\"private_velocity_root\":\""
           << step.private_velocity_root << '"'
           << ",\"work_exact\":" << (step.work_exact ? "true" : "false")
           << ",\"expected_work_root\":\"" << step.expected_work_root
           << "\",\"actual_work_root\":\"" << step.actual_work_root
           << "\",\"work_root\":\"" << step.actual_work_root
           << "\",\"result_root\":\"" << step.result_root
           << "\",\"expected_work\":";
    emit_work(output, step.expected_work);
    output << ",\"work\":";
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

void emit_control(std::ostream& output, const Control16& control) {
    output << "{\"name\":\"" << control.name << "\",\"pass\":"
           << (control.pass ? "true" : "false") << ",\"scalars\":[";
    for (std::size_t index = 0U; index < control.scalars.size(); ++index) {
        if (index != 0U) output << ',';
        output << "{\"name\":\"" << control.scalars[index].first
               << "\",\"value\":"
               << (control.scalars[index].second ? "true" : "false")
               << '}';
    }
    output << "],\"evidence_roots\":[";
    for (std::size_t index = 0U; index < control.evidence.size(); ++index) {
        if (index != 0U) output << ',';
        output << '"' << control.evidence[index] << '"';
    }
    output << "],\"work_exact\":"
           << (control.work_exact ? "true" : "false")
           << ",\"expected_work_root\":\""
           << control.expected_work_root
           << "\",\"actual_work_root\":\"" << control.actual_work_root
           << "\",\"work_root\":\"" << control.actual_work_root
           << "\",\"expected_work\":";
    emit_work(output, control.expected_work);
    output << ",\"work\":";
    emit_work(output, control.work);
    output << ",\"result_root\":\"" << control.result_root << "\"}";
}

int run() {
    const BinaryRead binary = read_binary();
    Work16 identity_work;
    const Fixture16 fixture = make_fixture(identity_work);
    Work16 identity_expected;
    identity_expected.fixture_records = fixture.state.dynamic.size();
    identity_expected.ghost_records = fixture.state.ghosts.size();
    identity_expected.canonicalized_records = fixture.state.dynamic.size()
        + fixture.state.ghosts.size();
    identity_expected.portable_fields = 10U * fixture.state.dynamic.size()
        + 4U * fixture.state.ghosts.size() + 5U;
    identity_expected.hash_derivations = 4U;
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
        StepGate16 gate = StepGate16::None;
        if (step.accepted) {
            static_cast<void>(one_step_gate(step, fixture.state));
            gate = StepGate16::OneStep;
        }
        finalize_step_work(step, fixture.state, fixture.state,
            masks[index].second, index == 0U, gate);
        seal_step(step);
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
    std::vector<Control16> controls;
    if (masks_result.size() == masks.size()
            && std::all_of(masks_result.begin(), masks_result.end(),
                [](const Step16& value) { return value.accepted; })) {
        controls = run_controls(fixture, masks_result);
        if (!std::all_of(controls.begin(), controls.end(),
                [](const Control16& value) { return value.pass; })) {
            status = "APPARATUS_INCONCLUSIVE";
        }
    }
    std::vector<Step16> trajectory;
    State15 trajectory_state = fixture.state;
    Vec3l accumulated_external;
    if (masks_result.size() == masks.size()
            && std::all_of(masks_result.begin(), masks_result.end(),
                [](const Step16& value) { return value.accepted; })
            && !controls.empty()
            && std::all_of(controls.begin(), controls.end(),
                [](const Control16& value) { return value.pass; })) {
        for (std::size_t step_index = 0U;
             step_index < kTrajectorySteps; ++step_index) {
            Step16 step = run_step(trajectory_state, "PVS",
                TermMask15{false, true, false, true}, false);
            bool accepted = step.accepted;
            StepGate16 gate = StepGate16::None;
            if (accepted) {
                accepted = trajectory_gate(step, fixture.state,
                    trajectory_state, accumulated_external, step_index + 1U);
                gate = StepGate16::Trajectory;
            }
            finalize_step_work(step, trajectory_state, fixture.state,
                TermMask15{false, true, false, true}, false, gate);
            accepted = accepted && step.work_exact;
            seal_step(step);
            if (accepted) {
                trajectory_state = step.state;
                accumulated_external += step.external_impulse;
            }
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
    Work16 total_expected = identity_expected;
    for (const Step16& step : masks_result) add_work(total, step.work);
    for (const Step16& step : masks_result) {
        add_work(total_expected, step.expected_work);
    }
    for (const Control16& control : controls) add_work(total, control.work);
    for (const Control16& control : controls) {
        add_work(total_expected, control.expected_work);
    }
    for (const Step16& step : trajectory) add_work(total, step.work);
    for (const Step16& step : trajectory) {
        add_work(total_expected, step.expected_work);
    }
    std::string controls_bytes;
    put_string(controls_bytes, "nextengine.nonlocal.ncgp16.controls.v1");
    put_u64(controls_bytes, controls.size());
    for (const Control16& control : controls) {
        put_string(controls_bytes, control.result_root);
    }
    const std::string controls_root = root(std::move(controls_bytes));
    total.portable_fields += 2U + controls.size();
    total_expected.portable_fields += 2U + controls.size();
    ++total.hash_derivations;
    ++total_expected.hash_derivations;
    std::string trajectory_bytes;
    put_string(trajectory_bytes, "nextengine.nonlocal.ncgp16.trajectory.v1");
    put_u64(trajectory_bytes, trajectory.size());
    for (const Step16& step : trajectory) {
        put_string(trajectory_bytes, step.result_root);
    }
    const std::string trajectory_root = root(std::move(trajectory_bytes));
    total.portable_fields += 2U + trajectory.size();
    total_expected.portable_fields += 2U + trajectory.size();
    ++total.hash_derivations;
    ++total_expected.hash_derivations;
    const std::string expected_work_root = work_root(total_expected);
    const std::string actual_work_root = work_root(total);
    const bool total_work_exact = work_values(total_expected)
        == work_values(total);
    if (!total_work_exact) status = "APPARATUS_INCONCLUSIVE";
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
    put_string(result_bytes, controls_root);
    put_string(result_bytes, trajectory_root);
    put_u32(result_bytes, total_work_exact ? 1U : 0U);
    put_string(result_bytes, expected_work_root);
    put_string(result_bytes, actual_work_root);
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
    std::cout << "],\"controls\":[";
    for (std::size_t index = 0U; index < controls.size(); ++index) {
        if (index != 0U) std::cout << ',';
        emit_control(std::cout, controls[index]);
    }
    std::cout << "],\"controls_root\":\"" << controls_root
              << "\",\"trajectory\":[";
    for (std::size_t index = 0U; index < trajectory.size(); ++index) {
        if (index != 0U) std::cout << ',';
        emit_step(std::cout, trajectory[index]);
    }
    std::cout << "],\"trajectory_root\":\"" << trajectory_root
              << "\",\"work_exact\":"
              << (total_work_exact ? "true" : "false")
              << ",\"expected_work_root\":\"" << expected_work_root
              << "\",\"actual_work_root\":\"" << actual_work_root
              << "\",\"work_root\":\"" << actual_work_root
              << "\",\"expected_work\":";
    emit_work(std::cout, total_expected);
    std::cout << ",\"work\":";
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
