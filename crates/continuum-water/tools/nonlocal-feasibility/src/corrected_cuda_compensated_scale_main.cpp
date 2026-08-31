#include "corrected_cuda_full_step.hpp"
#include "corrected_cuda_compensated_scale_physics.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <iterator>
#include <limits>
#include <sstream>
#include <stdexcept>
#include <string>
#include <unordered_map>
#include <vector>

#ifndef NCGP3_CONTRACT_ROOT
#define NCGP3_CONTRACT_ROOT "unconfigured"
#endif
#ifndef NCGP3_SOURCE_ROOT
#define NCGP3_SOURCE_ROOT "unconfigured"
#endif
#ifndef NCGP3_SOURCE_COMMIT
#define NCGP3_SOURCE_COMMIT "unconfigured"
#endif
#ifndef NCGP3_SOURCE_TREE
#define NCGP3_SOURCE_TREE "unconfigured"
#endif
#ifndef NCGP3_COMPILER_FLAGS
#define NCGP3_COMPILER_FLAGS "unconfigured"
#endif
#ifndef NCGP4_CONTRACT_ROOT
#define NCGP4_CONTRACT_ROOT "unconfigured"
#endif
#ifndef NCGP4_SOURCE_ROOT
#define NCGP4_SOURCE_ROOT "unconfigured"
#endif
#ifndef NCGP4_SOURCE_COMMIT
#define NCGP4_SOURCE_COMMIT "unconfigured"
#endif
#ifndef NCGP4_SOURCE_TREE
#define NCGP4_SOURCE_TREE "unconfigured"
#endif
#ifndef NCGP4_COMPILER_FLAGS
#define NCGP4_COMPILER_FLAGS "unconfigured"
#endif
#ifndef NCGP5_CONTRACT_ROOT
#define NCGP5_CONTRACT_ROOT "unconfigured"
#endif
#ifndef NCGP5_SOURCE_ROOT
#define NCGP5_SOURCE_ROOT "unconfigured"
#endif
#ifndef NCGP5_SOURCE_COMMIT
#define NCGP5_SOURCE_COMMIT "unconfigured"
#endif
#ifndef NCGP5_SOURCE_TREE
#define NCGP5_SOURCE_TREE "unconfigured"
#endif
#ifndef NCGP5_COMPILER_FLAGS
#define NCGP5_COMPILER_FLAGS "unconfigured"
#endif
#ifndef NCGP6_CONTRACT_ROOT
#define NCGP6_CONTRACT_ROOT "unconfigured"
#endif
#ifndef NCGP6_SOURCE_ROOT
#define NCGP6_SOURCE_ROOT "unconfigured"
#endif
#ifndef NCGP6_SOURCE_COMMIT
#define NCGP6_SOURCE_COMMIT "unconfigured"
#endif
#ifndef NCGP6_SOURCE_TREE
#define NCGP6_SOURCE_TREE "unconfigured"
#endif
#ifndef NCGP6_COMPILER_FLAGS
#define NCGP6_COMPILER_FLAGS "unconfigured"
#endif

namespace {

using namespace nextengine::nonlocal::gpu_full_step;

float float_from_bits(std::uint32_t bits) {
    float value = 0.0F;
    static_assert(sizeof(value) == sizeof(bits));
    std::memcpy(&value, &bits, sizeof(value));
    return value;
}

std::uint32_t float_bits(float value) {
    std::uint32_t bits = 0U;
    std::memcpy(&bits, &value, sizeof(bits));
    return bits;
}

std::uint64_t double_bits(double value) {
    std::uint64_t bits = 0U;
    static_assert(sizeof(value) == sizeof(bits));
    std::memcpy(&bits, &value, sizeof(value));
    return bits;
}

Vec3d subtract(Vec3d lhs, Vec3d rhs) {
    return {lhs.x - rhs.x, lhs.y - rhs.y, lhs.z - rhs.z};
}

double squared_norm(Vec3d value) {
    return value.x * value.x + value.y * value.y + value.z * value.z;
}

std::string file_root(const std::string& path) {
    std::ifstream stream(path, std::ios::binary);
    if (!stream) return {};
    std::ostringstream bytes;
    bytes << stream.rdbuf();
    return nextengine::nonlocal::sha256_hex(bytes.str());
}

std::string binary_root() { return file_root("/proc/self/exe"); }

long double cubic_weight(
    long double radius, long double horizon, long double kernel_scale) {
    const long double pi = acosl(-1.0L);
    const long double q = 2.0L * radius / horizon;
    const long double alpha = kernel_scale * 3.0L
        / (2.0L * pi * horizon * horizon * horizon);
    if (q < 1.0L) {
        return alpha * (2.0L / 3.0L - q * q + 0.5L * q * q * q);
    }
    if (q <= 2.0L) {
        const long double tail = 2.0L - q;
        return alpha * tail * tail * tail / 6.0L;
    }
    return 0.0L;
}

NonlocalGpuProfile pair_profile() {
    NonlocalGpuProfile profile = nonlocal_water_profile();
    profile.gravity = {};
    profile.kappa = 500.0;
    profile.rest_density = static_cast<double>(
        static_cast<long double>(profile.mass)
        * (cubic_weight(0.0L, profile.horizon, profile.kernel_scale)
            + cubic_weight(
                0.05L, profile.horizon, profile.kernel_scale))
        / 1.1L);
    return profile;
}

long double infinite_lattice_density(const NonlocalGpuProfile& profile) {
    long double density = 0.0L;
    for (int z = -3; z <= 3; ++z) {
        for (int y = -3; y <= 3; ++y) {
            for (int x = -3; x <= 3; ++x) {
                const long double radius = static_cast<long double>(
                    profile.spacing)
                    * std::sqrt(static_cast<long double>(
                        x * x + y * y + z * z));
                if (radius <= static_cast<long double>(profile.horizon)) {
                    density += static_cast<long double>(profile.mass)
                        * cubic_weight(radius,
                            static_cast<long double>(profile.horizon),
                            static_cast<long double>(profile.kernel_scale));
                }
            }
        }
    }
    return density;
}

long double cubic_weight_first(long double radius,
    const NonlocalGpuProfile& profile) {
    const long double h = profile.horizon;
    const long double q = 2.0L * radius / h;
    const long double alpha = static_cast<long double>(profile.kernel_scale)
        * 3.0L / (2.0L * acosl(-1.0L) * h * h * h);
    long double derivative_q = 0.0L;
    if (q < 1.0L) {
        derivative_q = alpha * (-2.0L * q + 1.5L * q * q);
    } else if (q <= 2.0L) {
        const long double tail = 2.0L - q;
        derivative_q = -0.5L * alpha * tail * tail;
    }
    return derivative_q * 2.0L / h;
}

long double trajectory_surface_potential(
    long double radius, long double spacing) {
    const long double q = radius / spacing;
    if (q <= 1.0L) {
        return spacing * (q * q * q / 3.0L - q - 2.0L / 3.0L);
    }
    if (q < 3.0L) {
        const long double shifted = q - 2.0L;
        return spacing
            * (q - shifted * shifted * shifted / 3.0L - 8.0L / 3.0L);
    }
    return 0.0L;
}

struct TrajectoryPhysicsMetrics {
    bool valid = false;
    Vec3d momentum;
    Vec3d ghost_pressure_force;
    double mechanical_energy = 0.0;
};

TrajectoryPhysicsMetrics trajectory_physics_metrics(
    const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& state,
    const std::vector<NonlocalGpuGhost>& ghosts,
    const std::vector<double>& density) {
    TrajectoryPhysicsMetrics result;
    if (state.empty() || density.size() != state.size()) return result;
    const auto graph = build_reference_graph(profile, state, ghosts, false);
    if (graph.failure != NonlocalGpuFailure::None
        || graph.offsets.size() != state.size() + 1U) return result;
    std::unordered_map<std::uint32_t, std::size_t> dynamic;
    std::unordered_map<std::uint32_t, Vec3d> ghost_positions;
    dynamic.reserve(state.size());
    ghost_positions.reserve(ghosts.size());
    for (std::size_t index = 0U; index < state.size(); ++index) {
        dynamic.emplace(state[index].sample_id, index);
        result.momentum.x += profile.mass * state[index].velocity.x;
        result.momentum.y += profile.mass * state[index].velocity.y;
        result.momentum.z += profile.mass * state[index].velocity.z;
        result.mechanical_energy += 0.5 * profile.mass
                * (state[index].velocity.x * state[index].velocity.x
                    + state[index].velocity.y * state[index].velocity.y
                    + state[index].velocity.z * state[index].velocity.z)
            - profile.mass * (profile.gravity.x * state[index].current.x
                + profile.gravity.y * state[index].current.y
                + profile.gravity.z * state[index].current.z);
        const double excess = std::max(
            density[index] / profile.rest_density - 1.0, 0.0);
        result.mechanical_energy += 0.5 * profile.kappa * excess * excess;
    }
    for (const NonlocalGpuGhost& ghost : ghosts) {
        ghost_positions.emplace(ghost.sample_id, ghost.position);
    }
    for (std::size_t owner = 0U; owner < state.size(); ++owner) {
        const double excess = std::max(
            density[owner] / profile.rest_density - 1.0, 0.0);
        for (std::uint32_t slot = graph.offsets[owner];
             slot < graph.offsets[owner + 1U]; ++slot) {
            const std::uint32_t neighbor_id = graph.neighbor_ids[slot];
            const auto dynamic_neighbor = dynamic.find(neighbor_id);
            Vec3d neighbor{};
            if (dynamic_neighbor != dynamic.end()) {
                if (dynamic_neighbor->second == owner) continue;
                neighbor = state[dynamic_neighbor->second].current;
            } else {
                const auto ghost = ghost_positions.find(neighbor_id);
                if (ghost == ghost_positions.end()) return result;
                neighbor = ghost->second;
            }
            const Vec3d difference{state[owner].current.x - neighbor.x,
                state[owner].current.y - neighbor.y,
                state[owner].current.z - neighbor.z};
            const long double radius = std::sqrt(
                static_cast<long double>(difference.x) * difference.x
                + static_cast<long double>(difference.y) * difference.y
                + static_cast<long double>(difference.z) * difference.z);
            if (!(radius > 0.0L)) continue;
            if (dynamic_neighbor != dynamic.end()) {
                if (dynamic_neighbor->second > owner) {
                    result.mechanical_energy += static_cast<double>(
                        2.0L * profile.gamma * profile.mass * profile.mass
                        * trajectory_surface_potential(radius,
                            profile.spacing));
                }
            } else if (excess > 0.0) {
                const long double coefficient = -static_cast<long double>(
                    profile.kappa * profile.mass / profile.rest_density
                    * excess) * cubic_weight_first(radius, profile) / radius;
                result.ghost_pressure_force.x += static_cast<double>(
                    coefficient * difference.x);
                result.ghost_pressure_force.y += static_cast<double>(
                    coefficient * difference.y);
                result.ghost_pressure_force.z += static_cast<double>(
                    coefficient * difference.z);
            }
        }
    }
    result.valid = std::isfinite(result.mechanical_energy)
        && std::isfinite(result.momentum.x)
        && std::isfinite(result.momentum.y)
        && std::isfinite(result.momentum.z)
        && std::isfinite(result.ghost_pressure_force.x)
        && std::isfinite(result.ghost_pressure_force.y)
        && std::isfinite(result.ghost_pressure_force.z);
    return result;
}

int run_profile_self_test() {
    const NonlocalGpuProfile raw = nonlocal_water_profile();
    const NonlocalGpuProfile corrected = nonlocal_water_corrected_profile();
    const long double raw_density = infinite_lattice_density(raw);
    const long double corrected_density = infinite_lattice_density(corrected);
    const long double raw_ratio = raw_density / raw.rest_density;
    const long double corrected_ratio = corrected_density
        / corrected.rest_density;
    const bool passed = validate_nonlocal_input(raw,
            canonicalize_samples_binary32({{1U, {0.5, 0.5, 0.5},
                {0.5, 0.5, 0.5}, {}}}), {}) == NonlocalGpuFailure::None
        && validate_nonlocal_input(corrected,
            canonicalize_samples_binary32({{1U, {0.5, 0.5, 0.5},
                {0.5, 0.5, 0.5}, {}}}), {}) == NonlocalGpuFailure::None
        && std::abs(raw_ratio - 0.12522433816880058L) <= 1.0e-15L
        && std::abs(corrected_ratio - 1.0L) <= 1.0e-12L
        && std::max(raw_ratio - 1.0L, 0.0L) == 0.0L;
    std::cout << std::setprecision(17)
              << "{\"schema\":\"nextengine.nonlocal.ncgp3.profile.v1\""
              << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
              << ",\"raw_profile_root\":\"" << profile_semantic_root(raw)
              << "\",\"corrected_profile_root\":\""
              << profile_semantic_root(corrected)
              << "\",\"raw_lattice_density_kg_m3\":"
              << static_cast<double>(raw_density)
              << ",\"raw_lattice_density_fraction\":"
              << static_cast<double>(raw_ratio)
              << ",\"raw_physical_admitted\":false"
              << ",\"corrected_lattice_density_kg_m3\":"
              << static_cast<double>(corrected_density)
              << ",\"corrected_lattice_density_fraction\":"
              << static_cast<double>(corrected_ratio)
              << ",\"kernel_scale\":" << corrected.kernel_scale
              << ",\"kappa\":" << corrected.kappa
              << ",\"lambda\":" << corrected.lambda
              << ",\"mu\":" << corrected.mu
              << ",\"gamma\":" << corrected.gamma
              << ",\"contract_root\":\"" << NCGP3_CONTRACT_ROOT
              << "\",\"source_root\":\"" << NCGP3_SOURCE_ROOT
              << "\",\"binary_root\":\"" << binary_root() << "\"}\n";
    return passed ? 0 : 4;
}

std::vector<NonlocalGpuSample> graph_fixture(bool permuted) {
    const float owner_high = float_from_bits(0x3ccccdd9U);
    const float owner_low = float_from_bits(0x30388db0U);
    const float candidate_high = float_from_bits(0x3e333376U);
    std::vector<NonlocalGpuSample> samples{
        {101U,
            {static_cast<double>(owner_high) + static_cast<double>(owner_low),
                0.75, 0.75},
            {static_cast<double>(owner_high) + static_cast<double>(owner_low),
                0.75, 0.75}, {}},
        {202U, {candidate_high, 0.75, 0.75},
            {candidate_high, 0.75, 0.75}, {}},
    };
    if (permuted) std::reverse(samples.begin(), samples.end());
    return samples;
}

NonlocalGpuGraphResult expected_graph(bool inclusive) {
    NonlocalGpuGraphResult result;
    result.dynamic_samples = 2U;
    result.owner_ids = {101U, 202U};
    if (inclusive) {
        result.directed_pairs = 4U;
        result.maximum_degree = 2U;
        result.offsets = {0U, 2U, 4U};
        result.neighbor_ids = {101U, 202U, 101U, 202U};
    } else {
        result.directed_pairs = 2U;
        result.maximum_degree = 1U;
        result.offsets = {0U, 1U, 2U};
        result.neighbor_ids = {101U, 202U};
    }
    return result;
}

NonlocalGpuGraphResult run_graph(
    NonlocalGpuVariant variant, bool permuted, std::string& environment) {
    NonlocalGpuProfile profile = nonlocal_water_profile();
    profile.gravity = {};
    NonlocalGpuWorkspace workspace(profile);
    const auto input = graph_fixture(permuted);
    const NonlocalGpuFailure upload = workspace.upload(input, {}, true);
    environment = workspace.environment_json();
    if (upload != NonlocalGpuFailure::None) {
        NonlocalGpuGraphResult result;
        result.failure = upload;
        return result;
    }
    return workspace.build_current_graph(variant, true, false);
}

int run_graph_self_test() {
    std::string environment;
    std::string ignored_environment;
    const auto correct = run_graph(
        NonlocalGpuVariant::CompensatedScaleF32, false, environment);
    const auto permuted = run_graph(
        NonlocalGpuVariant::CompensatedScaleF32, true, ignored_environment);
    const auto high_only = run_graph(
        NonlocalGpuVariant::CompensatedScaleHighOnlyGraph, false,
        ignored_environment);
    const auto strict = run_graph(
        NonlocalGpuVariant::CompensatedScaleStrictRadius, false,
        ignored_environment);
    const auto inclusive_expected = expected_graph(true);
    const auto exclusive_expected = expected_graph(false);
    const std::string correct_root = graph_semantic_root(correct);
    const std::string expected_root = graph_semantic_root(inclusive_expected);
    const std::string exclusive_root = graph_semantic_root(exclusive_expected);
    const std::string executable_root = binary_root();
    const bool passed = correct.failure == NonlocalGpuFailure::None
        && permuted.failure == NonlocalGpuFailure::None
        && high_only.failure == NonlocalGpuFailure::None
        && strict.failure == NonlocalGpuFailure::None
        && correct_root == expected_root
        && graph_semantic_root(permuted) == expected_root
        && graph_semantic_root(high_only) == exclusive_root
        && graph_semantic_root(strict) == exclusive_root
        && correct.work.compensated_graph_quantizations == 42U
        && strict.work.compensated_graph_quantizations == 42U
        && high_only.work.compensated_graph_quantizations == 42U
        && executable_root.size() == 64U;
    const std::string input_root = nextengine::nonlocal::sha256_hex(
        "nextengine.nonlocal.ncgp3.graph-input.v1\n"
        "owner-hi=3ccccdd9\nowner-lo=30388db0\n"
        "candidate-hi=3e333376\ncandidate-lo=00000000\n"
        "y=3f400000\nz=3f400000\n");
    std::cout << std::setprecision(17)
              << "{\"schema\":\"nextengine.nonlocal.ncgp3.graph.v1\""
              << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
              << ",\"contract_root\":\"" << NCGP3_CONTRACT_ROOT << "\""
              << ",\"source_root\":\"" << NCGP3_SOURCE_ROOT << "\""
              << ",\"source_commit\":\"" << NCGP3_SOURCE_COMMIT << "\""
              << ",\"source_tree\":\"" << NCGP3_SOURCE_TREE << "\""
              << ",\"binary_root\":\"" << executable_root << "\""
              << ",\"compiler_flags\":\"" << NCGP3_COMPILER_FLAGS << "\""
              << ",\"input_root\":\"" << input_root << "\""
              << ",\"correct_root\":\"" << correct_root << "\""
              << ",\"permuted_root\":\"" << graph_semantic_root(permuted)
              << "\",\"high_only_root\":\"" << graph_semantic_root(high_only)
              << "\",\"strict_root\":\"" << graph_semantic_root(strict) << "\""
              << ",\"correct_work_root\":\""
              << work_semantic_root(correct.work)
              << "\",\"high_only_work_root\":\""
              << work_semantic_root(high_only.work) << "\""
              << ",\"owner_high_um\":25000,\"owner_pair_um\":25001"
              << ",\"candidate_um\":175001"
              << ",\"correct_pairs\":" << correct.directed_pairs
              << ",\"high_only_pairs\":" << high_only.directed_pairs
              << ",\"strict_pairs\":" << strict.directed_pairs
              << ",\"pair_quantizations\":"
              << correct.work.compensated_graph_quantizations
              << ",\"environment\":" << environment << "}\n";
    return passed ? 0 : 4;
}

struct BoundaryFixtureResult {
    NonlocalGpuBoundaryProbeResult correct;
    NonlocalGpuBoundaryProbeResult high_only;
    std::uint32_t expected_mask = 0U;
    bool passed = false;
};

BoundaryFixtureResult run_boundary_fixture(std::uint32_t axes) {
    const float origin_high = float_from_bits(0x3cccccceU);
    const float origin_low = float_from_bits(0xb04ccccdU);
    const float proposal_value = float_from_bits(0xb1000000U);
    Vec3d origin{0.75, 0.75, 0.75};
    Vec3d proposal{};
    std::uint32_t expected_mask = 0U;
    if ((axes & 1U) != 0U) {
        origin.x = static_cast<double>(origin_high) + origin_low;
        proposal.x = proposal_value;
        expected_mask |= 1U;
    }
    if ((axes & 2U) != 0U) {
        origin.y = static_cast<double>(origin_high) + origin_low;
        proposal.y = proposal_value;
        expected_mask |= 4U;
    }
    if ((axes & 4U) != 0U) {
        origin.z = static_cast<double>(origin_high) + origin_low;
        proposal.z = proposal_value;
        expected_mask |= 16U;
    }
    const std::vector<NonlocalGpuSample> input{
        {303U, origin, origin, {}},
    };
    NonlocalGpuProfile profile = nonlocal_water_profile();
    profile.gravity = {};
    NonlocalGpuWorkspace correct_workspace(profile);
    NonlocalGpuWorkspace high_only_workspace(profile);
    BoundaryFixtureResult result;
    result.expected_mask = expected_mask;
    const bool uploads = correct_workspace.upload(input, {}, true)
            == NonlocalGpuFailure::None
        && high_only_workspace.upload(input, {}, true)
            == NonlocalGpuFailure::None;
    if (!uploads) return result;
    result.correct = correct_workspace.probe_boundary(
        {proposal}, NonlocalGpuVariant::CompensatedScaleF32, true);
    result.high_only = high_only_workspace.probe_boundary(
        {proposal}, NonlocalGpuVariant::CompensatedScaleHighOnlyBoundary, true);
    const std::uint64_t expected_receipt =
        (static_cast<std::uint64_t>(303U) << 8U) | expected_mask;
    bool contacted_bytes = result.correct.trial_high.size() == 1U
        && result.correct.trial_low.size() == 1U;
    const float lower = static_cast<float>(0.5 * profile.spacing);
    if ((axes & 1U) != 0U && contacted_bytes) {
        contacted_bytes = float_bits(
                static_cast<float>(result.correct.trial_high[0].x))
                == float_bits(lower)
            && float_bits(static_cast<float>(result.correct.trial_low[0].x))
                == 0U;
    }
    if ((axes & 2U) != 0U && contacted_bytes) {
        contacted_bytes = float_bits(
                static_cast<float>(result.correct.trial_high[0].y))
                == float_bits(lower)
            && float_bits(static_cast<float>(result.correct.trial_low[0].y))
                == 0U;
    }
    if ((axes & 4U) != 0U && contacted_bytes) {
        contacted_bytes = float_bits(
                static_cast<float>(result.correct.trial_high[0].z))
                == float_bits(lower)
            && float_bits(static_cast<float>(result.correct.trial_low[0].z))
                == 0U;
    }
    result.passed = result.correct.failure == NonlocalGpuFailure::None
        && result.high_only.failure == NonlocalGpuFailure::None
        && result.correct.face_mask_xor == expected_receipt
        && result.high_only.face_mask_xor == 0U
        && result.correct.work.boundary_face_tests == 6U
        && result.high_only.work.boundary_face_tests == 6U
        && result.correct.work.boundary_face_hits
            == static_cast<std::uint64_t>(__builtin_popcount(expected_mask))
        && result.correct.work.compensated_boundary_origin_components == 3U
        && result.correct.work.compensated_contact_canonicalizations
            == static_cast<std::uint64_t>(__builtin_popcount(expected_mask))
        && contacted_bytes;
    return result;
}

int run_boundary_self_test() {
    const BoundaryFixtureResult face = run_boundary_fixture(1U);
    const BoundaryFixtureResult edge = run_boundary_fixture(3U);
    const BoundaryFixtureResult corner = run_boundary_fixture(7U);
    const bool passed = face.passed && edge.passed && corner.passed;
    std::cout << "{\"schema\":\"nextengine.nonlocal.ncgp3.boundary.v1\""
              << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
              << ",\"contract_root\":\"" << NCGP3_CONTRACT_ROOT << "\""
              << ",\"source_root\":\"" << NCGP3_SOURCE_ROOT << "\""
              << ",\"source_commit\":\"" << NCGP3_SOURCE_COMMIT << "\""
              << ",\"source_tree\":\"" << NCGP3_SOURCE_TREE << "\""
              << ",\"binary_root\":\"" << binary_root() << "\""
              << ",\"face_mask\":" << (face.correct.face_mask_xor & 0xffU)
              << ",\"edge_mask\":" << (edge.correct.face_mask_xor & 0xffU)
              << ",\"corner_mask\":" << (corner.correct.face_mask_xor & 0xffU)
              << ",\"high_only_face_mask\":"
              << (face.high_only.face_mask_xor & 0xffU)
              << ",\"face_work_root\":\""
              << work_semantic_root(face.correct.work)
              << "\",\"edge_work_root\":\""
              << work_semantic_root(edge.correct.work)
              << "\",\"corner_work_root\":\""
              << work_semantic_root(corner.correct.work) << "\"}\n";
    return passed ? 0 : 4;
}

std::string compensated_state_root(
    const NonlocalGpuCompensatedStateSnapshot& snapshot) {
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp3.device-state.v1\n"
             << static_cast<std::uint32_t>(snapshot.failure) << '\n';
    for (std::size_t index = 0U; index < snapshot.ids.size(); ++index) {
        material << snapshot.ids[index] << ':';
        const std::array<const std::vector<Vec3d>*, 8> fields{
            &snapshot.reference_high, &snapshot.reference_low,
            &snapshot.current_high, &snapshot.current_low,
            &snapshot.predicted_high, &snapshot.predicted_low,
            &snapshot.velocity_high, &snapshot.velocity_low};
        for (const auto* field : fields) {
            if (index >= field->size()) return {};
            const Vec3d& value = (*field)[index];
            material << std::hex
                     << float_bits(static_cast<float>(value.x)) << ','
                     << float_bits(static_cast<float>(value.y)) << ','
                     << float_bits(static_cast<float>(value.z)) << ';'
                     << std::dec;
        }
        material << '\n';
    }
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::vector<NonlocalGpuSample> transaction_pair_state() {
    return canonicalize_samples_binary32({
        {101U, {0.725, 0.75, 0.75}, {0.725, 0.75, 0.75}, {}},
        {202U, {0.775, 0.75, 0.75}, {0.775, 0.75, 0.75}, {}},
    });
}

int run_transaction_self_test() {
    const NonlocalGpuProfile profile = pair_profile();
    const auto input = transaction_pair_state();
    const std::string input_root = input_semantic_root(profile, input, {});
    NonlocalGpuWorkspace workspace(profile);
    NonlocalGpuWorkspace fresh(profile);
    const bool uploaded = workspace.upload(input, {}, true)
            == NonlocalGpuFailure::None
        && fresh.upload(input, {}, true) == NonlocalGpuFailure::None;
    if (!uploaded) return 4;
    const auto before = workspace.capture_compensated_state();
    const auto failed = workspace.step(32U,
        NonlocalGpuSolverProfile::Unpreconditioned,
        NonlocalGpuVariant::CompensatedScalePostFinalizeFailure,
        true, false);
    const auto restored = workspace.capture_compensated_state();
    const auto retry = workspace.step(32U,
        NonlocalGpuSolverProfile::Unpreconditioned,
        NonlocalGpuVariant::CompensatedScaleF32, true, false);
    const auto fresh_result = fresh.step(32U,
        NonlocalGpuSolverProfile::Unpreconditioned,
        NonlocalGpuVariant::CompensatedScaleF32, true, false);
    const std::string before_root = compensated_state_root(before);
    const std::string restored_root = compensated_state_root(restored);
    const std::string retry_root = step_semantic_root(
        profile, input_root, retry);
    const std::string fresh_root = step_semantic_root(
        profile, input_root, fresh_result);
    const bool passed = before.failure == NonlocalGpuFailure::None
        && restored.failure == NonlocalGpuFailure::None
        && failed.failure == NonlocalGpuFailure::DeviceFailure
        && failed.work.compensated_fault_injection_components == 2U
        && failed.work.compensated_rollback_components == 24U
        && before_root.size() == 64U && before_root == restored_root
        && retry.failure == NonlocalGpuFailure::None
        && fresh_result.failure == NonlocalGpuFailure::None
        && retry_root == fresh_root
        && step_work_semantic_root(profile, retry)
            == step_work_semantic_root(profile, fresh_result);
    std::cout << "{\"schema\":\"nextengine.nonlocal.ncgp3.transaction.v1\""
              << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
              << ",\"contract_root\":\"" << NCGP3_CONTRACT_ROOT << "\""
              << ",\"source_root\":\"" << NCGP3_SOURCE_ROOT << "\""
              << ",\"source_commit\":\"" << NCGP3_SOURCE_COMMIT << "\""
              << ",\"source_tree\":\"" << NCGP3_SOURCE_TREE << "\""
              << ",\"binary_root\":\"" << binary_root() << "\""
              << ",\"before_root\":\"" << before_root
              << "\",\"restored_root\":\"" << restored_root
              << "\",\"failure\":"
              << static_cast<std::uint32_t>(failed.failure)
              << ",\"fault_components\":"
              << failed.work.compensated_fault_injection_components
              << ",\"rollback_components\":"
              << failed.work.compensated_rollback_components
              << ",\"failure_work_root\":\""
              << step_work_semantic_root(profile, failed)
              << "\",\"retry_result_root\":\"" << retry_root
              << "\",\"fresh_result_root\":\"" << fresh_root << "\"}\n";
    return passed ? 0 : 4;
}

std::vector<NonlocalGpuSample> trajectory_initial(
    const NonlocalGpuProfile& profile,
    const std::string& scenario,
    bool permuted) {
    std::vector<NonlocalGpuSample> result;
    if (scenario == "hydrostatic-hold") {
        result = make_lattice_state(profile, 20U, 20U, 10U, permuted, false);
    } else if (scenario == "dam-break") {
        result = make_lattice_state(profile, 10U, 20U, 20U, permuted, false);
    } else if (scenario == "orifice-jet") {
        result = make_lattice_state(profile, 20U, 20U, 10U, permuted, false);
        for (auto& sample : result) {
            const double dy = sample.current.y - 0.75;
            const double dz = sample.current.z - 0.25;
            if (dy * dy + dz * dz <= 0.15 * 0.15) {
                sample.velocity.x = 1.5;
            }
        }
        result = canonicalize_samples_binary32(result);
    } else {
        throw std::invalid_argument("unknown NCGP3 trajectory");
    }
    return result;
}

struct PositionDistribution {
    double rmse = std::numeric_limits<double>::infinity();
    double p50 = std::numeric_limits<double>::infinity();
    double p95 = std::numeric_limits<double>::infinity();
    double p99 = std::numeric_limits<double>::infinity();
    double maximum = std::numeric_limits<double>::infinity();
};

struct PositionMetricWork {
    std::uint64_t samples_loaded = 0U;
    std::uint64_t squared_accumulations = 0U;
    std::uint64_t sorted_values = 0U;
    std::uint64_t nearest_rank_reads = 0U;
    std::uint64_t maximum_reads = 0U;
    std::uint64_t hashed_sample_records = 0U;
    std::uint64_t root_derivations = 0U;
};

std::size_t nearest_rank_index(
    std::size_t size, std::size_t numerator, std::size_t denominator) {
    if (size == 0U || numerator == 0U || numerator > denominator) {
        throw std::invalid_argument("invalid nearest-rank request");
    }
    return (numerator * size + denominator - 1U) / denominator - 1U;
}

PositionDistribution position_distribution(
    const std::vector<double>& unsorted_errors) {
    PositionDistribution result;
    if (unsorted_errors.empty()) return result;
    std::vector<double> errors = unsorted_errors;
    long double squared = 0.0L;
    for (const double error : errors) {
        if (!std::isfinite(error) || error < 0.0) return result;
        squared += static_cast<long double>(error) * error;
    }
    std::sort(errors.begin(), errors.end());
    result.rmse = static_cast<double>(std::sqrt(
        squared / static_cast<long double>(errors.size())));
    result.p50 = errors[nearest_rank_index(errors.size(), 50U, 100U)];
    result.p95 = errors[nearest_rank_index(errors.size(), 95U, 100U)];
    result.p99 = errors[nearest_rank_index(errors.size(), 99U, 100U)];
    result.maximum = errors.back();
    return result;
}

bool product_position_gate(const PositionDistribution& distribution) {
    return std::isfinite(distribution.rmse)
        && std::isfinite(distribution.p99)
        && distribution.rmse <= 0.0025
        && distribution.p99 <= 0.0025;
}

std::string position_metric_work_root(const PositionMetricWork& work) {
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp6.position-metric-work.v1\n"
             << work.samples_loaded << ':' << work.squared_accumulations << ':'
             << work.sorted_values << ':' << work.nearest_rank_reads << ':'
             << work.maximum_reads << ':' << work.hashed_sample_records << ':'
             << work.root_derivations << '\n';
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string position_sample_records_root(std::uint32_t step,
    const std::vector<std::pair<std::uint32_t, double>>& records) {
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp6.position-samples.v1\n"
             << step << ':' << records.size() << '\n' << std::hex;
    for (const auto& [sample_id, error] : records) {
        material << sample_id << ':' << double_bits(error) << '\n';
    }
    return nextengine::nonlocal::sha256_hex(material.str());
}

#if defined(NCGP6_EXPERIMENTAL)
int run_ncgp6_product_gate_self_test() {
    constexpr std::size_t kSamples = 4000U;
    constexpr double kTail = 0.0052112092581191585;
    constexpr double kAboveP99Gate = 0.002500001;
    std::vector<double> one_tail(kSamples, 0.0);
    one_tail.back() = kTail;
    const PositionDistribution one = position_distribution(one_tail);
    std::vector<double> forty_tail(kSamples, 0.0);
    std::fill(forty_tail.end() - 40, forty_tail.end(), kAboveP99Gate);
    const PositionDistribution forty = position_distribution(forty_tail);
    std::vector<double> forty_one_tail(kSamples, 0.0);
    std::fill(forty_one_tail.end() - 41, forty_one_tail.end(),
        kAboveP99Gate);
    const PositionDistribution forty_one = position_distribution(
        forty_one_tail);
    const std::size_t p99_index = nearest_rank_index(kSamples, 99U, 100U);
    std::vector<double> sorted_forty = forty_tail;
    std::sort(sorted_forty.begin(), sorted_forty.end());
    const bool rank_mutation_rejected = forty.p99 <= 0.0025
        && sorted_forty[p99_index + 1U] > 0.0025;
    const bool old_maximum_rejects = one.maximum > 0.005;
    const bool one_tail_passes = product_position_gate(one)
        && old_maximum_rejects;
    const bool forty_one_tail_rejected = forty_one.rmse <= 0.0025
        && forty_one.p99 > 0.0025
        && !product_position_gate(forty_one);
    PositionMetricWork work{3U * kSamples, 3U * kSamples, 4U * kSamples,
        10U, 3U, 2U * kSamples, 7U};
    const std::string work_root = position_metric_work_root(work);
    PositionMetricWork mutated_work = work;
    ++mutated_work.sorted_values;
    const bool work_mutation_rejected = work_root
        != position_metric_work_root(mutated_work);
    std::vector<std::pair<std::uint32_t, double>> sample_records;
    sample_records.reserve(kSamples);
    for (std::size_t index = 0U; index < kSamples; ++index) {
        sample_records.emplace_back(static_cast<std::uint32_t>(index),
            one_tail[index]);
    }
    const std::string sample_root = position_sample_records_root(
        92U, sample_records);
    sample_records[0].second = std::nextafter(0.0, 1.0);
    const bool sample_mutation_rejected = sample_root
        != position_sample_records_root(92U, sample_records);
    const std::string executable_root = binary_root();
    std::ostringstream complete_material;
    complete_material << std::setprecision(17)
                      << "nextengine.nonlocal.ncgp6.gate-result.v1\n"
                      << one.rmse << ':' << one.p50 << ':' << one.p95 << ':'
                      << one.p99 << ':' << one.maximum << ':' << p99_index
                      << ':' << sample_root << ':' << work_root << '\n'
                      << old_maximum_rejects << ':' << one_tail_passes << ':'
                      << forty_one_tail_rejected << ':'
                      << rank_mutation_rejected << ':'
                      << sample_mutation_rejected << ':'
                      << work_mutation_rejected << ':' << forty.rmse << ':'
                      << forty.p99 << ':' << forty.maximum << ':'
                      << forty_one.rmse << ':' << forty_one.p99 << ':'
                      << forty_one.maximum << '\n'
                      << NCGP6_CONTRACT_ROOT << ':' << NCGP6_SOURCE_ROOT << ':'
                      << NCGP6_SOURCE_COMMIT << ':' << NCGP6_SOURCE_TREE << ':'
                      << NCGP6_COMPILER_FLAGS << ':' << executable_root << '\n';
    std::ostringstream omitted_material;
    omitted_material << std::setprecision(17)
                     << "nextengine.nonlocal.ncgp6.gate-result.v1\n"
                     << one.rmse << ':' << one.p50 << ':' << one.p95 << ':'
                     << one.maximum << ':' << p99_index << ':' << sample_root
                     << ':' << work_root << '\n'
                     << old_maximum_rejects << ':' << one_tail_passes << ':'
                     << forty_one_tail_rejected << ':'
                     << rank_mutation_rejected << ':'
                     << sample_mutation_rejected << ':'
                     << work_mutation_rejected << ':' << forty.rmse << ':'
                     << forty.p99 << ':' << forty.maximum << ':'
                     << forty_one.rmse << ':' << forty_one.p99 << ':'
                     << forty_one.maximum << '\n'
                     << NCGP6_CONTRACT_ROOT << ':' << NCGP6_SOURCE_ROOT << ':'
                     << NCGP6_SOURCE_COMMIT << ':' << NCGP6_SOURCE_TREE << ':'
                     << NCGP6_COMPILER_FLAGS << ':' << executable_root << '\n';
    const std::string result_root = nextengine::nonlocal::sha256_hex(
        complete_material.str());
    const bool omitted_p99_rejected = result_root
        != nextengine::nonlocal::sha256_hex(omitted_material.str());
    const bool identity_valid = !executable_root.empty()
        && std::string(NCGP6_CONTRACT_ROOT) != "unconfigured"
        && std::string(NCGP6_SOURCE_ROOT) != "unconfigured"
        && std::string(NCGP6_SOURCE_COMMIT) != "unconfigured"
        && std::string(NCGP6_SOURCE_TREE) != "unconfigured"
        && std::string(NCGP6_COMPILER_FLAGS) != "unconfigured";
    const bool passed = p99_index == 3959U && one_tail_passes
        && forty.p99 <= 0.0025 && forty_one_tail_rejected
        && rank_mutation_rejected && work_mutation_rejected
        && sample_mutation_rejected && omitted_p99_rejected && identity_valid;
    std::cout << std::setprecision(17)
              << "{\"schema\":\"nextengine.nonlocal.ncgp6.gate-self-test.v1\""
              << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
              << ",\"samples\":" << kSamples
              << ",\"p99_index\":" << p99_index
              << ",\"one_tail_rmse_m\":" << one.rmse
              << ",\"one_tail_p99_m\":" << one.p99
              << ",\"one_tail_max_m\":" << one.maximum
              << ",\"old_maximum_rejects\":"
              << (old_maximum_rejects ? "true" : "false")
              << ",\"one_tail_product_passes\":"
              << (one_tail_passes ? "true" : "false")
              << ",\"forty_tail_p99_m\":" << forty.p99
              << ",\"forty_one_tail_rmse_m\":" << forty_one.rmse
              << ",\"forty_one_tail_p99_m\":" << forty_one.p99
              << ",\"forty_one_tail_rejected\":"
              << (forty_one_tail_rejected ? "true" : "false")
              << ",\"rank_mutation_rejected\":"
              << (rank_mutation_rejected ? "true" : "false")
              << ",\"sample_mutation_rejected\":"
              << (sample_mutation_rejected ? "true" : "false")
              << ",\"work_mutation_rejected\":"
              << (work_mutation_rejected ? "true" : "false")
              << ",\"omitted_p99_rejected\":"
              << (omitted_p99_rejected ? "true" : "false")
              << ",\"sample_records_root\":\"" << sample_root
              << "\",\"work_root\":\"" << work_root
              << "\",\"result_root\":\"" << result_root
              << "\",\"contract_root\":\"" << NCGP6_CONTRACT_ROOT
              << "\",\"source_root\":\"" << NCGP6_SOURCE_ROOT
              << "\",\"source_commit\":\"" << NCGP6_SOURCE_COMMIT
              << "\",\"source_tree\":\"" << NCGP6_SOURCE_TREE
              << "\",\"compiler_flags\":\"" << NCGP6_COMPILER_FLAGS
              << "\",\"binary_root\":\"" << executable_root
              << "\",\"exact_command\":\"--product-gate-self-test\"}\n";
    return passed ? 0 : 54;
}
#endif

int run_correspondence_4k(const std::string& scenario,
    std::uint32_t steps,
    std::uint32_t budget,
    NonlocalGpuVariant gpu_variant,
    const char* arithmetic_profile,
    NonlocalGpuSolverProfile solver_profile =
        NonlocalGpuSolverProfile::Jacobi,
    bool product_trajectory_gate = false) {
    if (steps == 0U || steps > 240U
        || (budget != 32U && budget != 64U && budget != 128U)) return 2;
    if (product_trajectory_gate
        && (steps != 240U || budget != 128U
            || gpu_variant != NonlocalGpuVariant::CompensatedScaleF32
            || solver_profile != NonlocalGpuSolverProfile::Unpreconditioned)) {
        return 2;
    }
#if defined(NCGP6_EXPERIMENTAL)
    constexpr const char* kCorrespondenceSchema =
        "nextengine.nonlocal.ncgp6.product-correspondence.v1";
    constexpr const char* kResultDomain =
        "nextengine.nonlocal.ncgp6.product-correspondence-result.v1\n";
    constexpr const char* kContractRoot = NCGP6_CONTRACT_ROOT;
    constexpr const char* kSourceRoot = NCGP6_SOURCE_ROOT;
    constexpr const char* kSourceCommit = NCGP6_SOURCE_COMMIT;
    constexpr const char* kSourceTree = NCGP6_SOURCE_TREE;
    constexpr const char* kCompilerFlags = NCGP6_COMPILER_FLAGS;
#elif defined(NCGP4_EXPERIMENTAL)
    constexpr const char* kCorrespondenceSchema =
        "nextengine.nonlocal.ncgp4.correspondence.v1";
    constexpr const char* kResultDomain =
        "nextengine.nonlocal.ncgp4.correspondence-result.v1\n";
    constexpr const char* kContractRoot = NCGP4_CONTRACT_ROOT;
    constexpr const char* kSourceRoot = NCGP4_SOURCE_ROOT;
    constexpr const char* kSourceCommit = NCGP4_SOURCE_COMMIT;
    constexpr const char* kSourceTree = NCGP4_SOURCE_TREE;
    constexpr const char* kCompilerFlags = NCGP4_COMPILER_FLAGS;
#else
    constexpr const char* kCorrespondenceSchema =
        "nextengine.nonlocal.ncgp3.correspondence.v3";
    constexpr const char* kResultDomain =
        "nextengine.nonlocal.ncgp3.correspondence-result.v3\n";
    constexpr const char* kContractRoot = NCGP3_CONTRACT_ROOT;
    constexpr const char* kSourceRoot = NCGP3_SOURCE_ROOT;
    constexpr const char* kSourceCommit = NCGP3_SOURCE_COMMIT;
    constexpr const char* kSourceTree = NCGP3_SOURCE_TREE;
    constexpr const char* kCompilerFlags = NCGP3_COMPILER_FLAGS;
#endif
    if (run_ncgp3_physics_self_test(false) != 0) return 54;
    const NonlocalGpuProfile profile = nonlocal_water_corrected_profile();
    const auto ghosts = canonicalize_ghosts_binary32(
        make_basin_ghosts(profile));
    auto cpu_state = trajectory_initial(profile, scenario, false);
    const auto permuted_input = trajectory_initial(profile, scenario, true);
    const std::string input_root = input_semantic_root(
        profile, cpu_state, ghosts);
    if (input_root != input_semantic_root(profile, permuted_input, ghosts)) {
        return 50;
    }
    NonlocalGpuWorkspace gpu(profile);
    NonlocalGpuWorkspace permuted(profile);
    if (gpu.upload(cpu_state, ghosts, true) != NonlocalGpuFailure::None
        || permuted.upload(permuted_input, ghosts, true)
            != NonlocalGpuFailure::None) return 51;
    const auto initial_evaluation = evaluate_reference(profile, cpu_state,
        ghosts, nullptr, NonlocalGpuVariant::Corrected);
    const TrajectoryPhysicsMetrics initial_physics = trajectory_physics_metrics(
        profile, cpu_state, ghosts, initial_evaluation.density);
    if (initial_evaluation.failure != NonlocalGpuFailure::None
        || !initial_physics.valid) return 52;
    TrajectoryPhysicsMetrics previous_physics = initial_physics;
    double maximum_momentum_residual = 0.0;
    double maximum_positive_energy_excess = 0.0;
    double maximum_penetration = 0.0;
    Vec3d gpu_minimum{std::numeric_limits<double>::infinity(),
        std::numeric_limits<double>::infinity(),
        std::numeric_limits<double>::infinity()};
    Vec3d gpu_maximum{-std::numeric_limits<double>::infinity(),
        -std::numeric_limits<double>::infinity(),
        -std::numeric_limits<double>::infinity()};
    double maximum_position_rmse = 0.0;
    double maximum_position_error = 0.0;
    double maximum_position_p50 = 0.0;
    double maximum_position_p95 = 0.0;
    double maximum_position_p99 = 0.0;
    std::uint32_t maximum_position_error_id = 0U;
    std::uint32_t maximum_position_error_component = 0U;
    double same_state_first_step_rmse = 0.0;
    double same_state_first_step_maximum = 0.0;
    bool same_state_first_step_active_signature_equal = true;
    double maximum_density_rmse = 0.0;
    double maximum_density_error = 0.0;
    double maximum_compression_rmse = 0.0;
    double maximum_compression_error = 0.0;
    double observed_density_minimum = std::numeric_limits<double>::infinity();
    double observed_density_maximum = 0.0;
    double observed_density_mean = 0.0;
    std::uint32_t maximum_gpu_hvp = 0U;
    std::uint32_t maximum_cpu_hvp = 0U;
    std::uint32_t maximum_active_pressure_centers = 0U;
    std::uint32_t maximum_directed_pairs = 0U;
    std::uint32_t maximum_degree = 0U;
    std::uint64_t hot_host_to_device_bytes = 0U;
    std::uint64_t hot_device_to_host_bytes = 0U;
    std::uint64_t snapshot_device_to_host_bytes = 0U;
    std::uint64_t active_mismatch_steps = 0U;
    std::uint64_t permutation_mismatch_steps = 0U;
    std::uint32_t first_active_mismatch_step = 0U;
    std::uint32_t first_active_mismatch_id = 0U;
    std::uint32_t first_gpu_active_count = 0U;
    std::uint32_t first_cpu_active_count = 0U;
    double first_gpu_mismatch_density = 0.0;
    double first_cpu_mismatch_density = 0.0;
    std::uint32_t first_gate_failure_step = 0U;
    std::string first_gate_failure;
    std::string receipt_material =
        "nextengine.nonlocal.ncgp3.trajectory-receipts.v1\n";
    NonlocalGpuFailure gpu_failure = NonlocalGpuFailure::None;
    NonlocalGpuFailure corrected_gpu_failure = NonlocalGpuFailure::None;
    NonlocalGpuFailure permuted_gpu_failure = NonlocalGpuFailure::None;
    NonlocalGpuFailure cpu_failure = NonlocalGpuFailure::None;
    std::uint32_t failure_gpu_hvp = 0U;
    std::uint32_t failure_permuted_hvp = 0U;
    std::uint64_t failure_outer_trials = 0U;
    std::uint64_t failure_accepted_trials = 0U;
    std::uint64_t failure_rejected_trials = 0U;
    std::string failure_work_root;
    std::string failure_permuted_work_root;
    std::string failure_restored_state_root;
    std::string failure_permuted_restored_state_root;
    const auto initial_gpu_compensated = gpu.capture_compensated_state();
    const auto initial_permuted_compensated =
        permuted.capture_compensated_state();
    if (initial_gpu_compensated.failure != NonlocalGpuFailure::None
        || initial_permuted_compensated.failure != NonlocalGpuFailure::None) {
        return 51;
    }
    std::string prior_gpu_state_root = compensated_state_root(
        initial_gpu_compensated);
    std::string prior_permuted_state_root = compensated_state_root(
        initial_permuted_compensated);
    std::string snapshot_receipt_material =
        "nextengine.nonlocal.ncgp3.snapshot-receipts.v1\n";
    std::string position_receipt_material =
        "nextengine.nonlocal.ncgp6.position-receipts.v1\n";
    PositionMetricWork position_metric_work;
    std::uint32_t completed = 0U;
    for (std::uint32_t step_index = 0U; step_index < steps; ++step_index) {
        const auto gpu_result = gpu.step(budget,
            solver_profile,
            gpu_variant, false, false);
        const auto permuted_result = permuted.step(budget,
            solver_profile,
            gpu_variant, false, false);
        const auto gpu_snapshot = gpu.capture_public_snapshot();
        const auto permuted_snapshot = permuted.capture_public_snapshot();
        const auto gpu_compensated = gpu.capture_compensated_state();
        const auto permuted_compensated = permuted.capture_compensated_state();
        const auto cpu_result = step_reference(profile, cpu_state, ghosts,
            128U, NonlocalGpuVariant::Corrected, true);
        corrected_gpu_failure = gpu_result.failure;
        permuted_gpu_failure = permuted_result.failure;
        gpu_failure = corrected_gpu_failure != NonlocalGpuFailure::None
            ? gpu_result.failure : permuted_result.failure;
        cpu_failure = cpu_result.failure;
        receipt_material += step_work_semantic_root(profile, gpu_result) + ':'
            + step_work_semantic_root(profile, permuted_result) + ':'
            + step_work_semantic_root(profile, cpu_result) + '\n';
        snapshot_receipt_material += work_semantic_root(gpu_snapshot.work)
            + ':' + work_semantic_root(permuted_snapshot.work)
            + ':' + work_semantic_root(gpu_compensated.work)
            + ':' + work_semantic_root(permuted_compensated.work) + '\n';
        maximum_gpu_hvp = std::max(maximum_gpu_hvp, gpu_result.hvp_used);
        maximum_cpu_hvp = std::max(maximum_cpu_hvp, cpu_result.hvp_used);
        maximum_active_pressure_centers = std::max(
            maximum_active_pressure_centers,
            gpu_result.active_pressure_centers);
        maximum_directed_pairs = std::max(maximum_directed_pairs,
            gpu_result.maximum_directed_pairs);
        maximum_degree = std::max(maximum_degree,
            gpu_result.maximum_degree);
        hot_host_to_device_bytes += gpu_result.work.host_to_device_bytes
            + permuted_result.work.host_to_device_bytes;
        hot_device_to_host_bytes += gpu_result.work.device_to_host_bytes
            + permuted_result.work.device_to_host_bytes;
        snapshot_device_to_host_bytes +=
            gpu_snapshot.work.device_to_host_bytes
            + permuted_snapshot.work.device_to_host_bytes;
        if (gpu_failure != NonlocalGpuFailure::None) {
            failure_gpu_hvp = gpu_result.hvp_used;
            failure_permuted_hvp = permuted_result.hvp_used;
            failure_outer_trials = gpu_result.work.outer_trials;
            failure_accepted_trials = gpu_result.work.accepted_trials;
            failure_rejected_trials = gpu_result.work.rejected_trials;
            failure_work_root = step_work_semantic_root(profile, gpu_result);
            failure_permuted_work_root = step_work_semantic_root(
                profile, permuted_result);
            failure_restored_state_root = compensated_state_root(
                gpu_compensated);
            failure_permuted_restored_state_root = compensated_state_root(
                permuted_compensated);
        }
        if (gpu_failure != NonlocalGpuFailure::None
            || cpu_failure != NonlocalGpuFailure::None
            || gpu_snapshot.failure != NonlocalGpuFailure::None
            || permuted_snapshot.failure != NonlocalGpuFailure::None
            || gpu_compensated.failure != NonlocalGpuFailure::None
            || permuted_compensated.failure != NonlocalGpuFailure::None
            || gpu_snapshot.state.size() != cpu_result.state.size()
            || gpu_snapshot.state.size() != permuted_snapshot.state.size()
            || gpu_snapshot.density.size() != cpu_result.density.size()) break;
        const TrajectoryPhysicsMetrics physics = trajectory_physics_metrics(
            profile, gpu_snapshot.state, ghosts, gpu_snapshot.density);
        if (!physics.valid) {
            gpu_failure = NonlocalGpuFailure::InvalidState;
            break;
        }
        const double total_mass = profile.mass
            * static_cast<double>(gpu_snapshot.state.size());
        const Vec3d momentum_residual{
            physics.momentum.x - previous_physics.momentum.x
                - profile.dt * (total_mass * profile.gravity.x
                    + physics.ghost_pressure_force.x)
                - gpu_result.boundary_impulse.x,
            physics.momentum.y - previous_physics.momentum.y
                - profile.dt * (total_mass * profile.gravity.y
                    + physics.ghost_pressure_force.y)
                - gpu_result.boundary_impulse.y,
            physics.momentum.z - previous_physics.momentum.z
                - profile.dt * (total_mass * profile.gravity.z
                    + physics.ghost_pressure_force.z)
                - gpu_result.boundary_impulse.z};
        const auto vector_norm = [](Vec3d value) {
            return std::sqrt(value.x * value.x + value.y * value.y
                + value.z * value.z);
        };
        const double momentum_scale = std::max({
            vector_norm(previous_physics.momentum),
            profile.dt * total_mass * vector_norm(profile.gravity),
            profile.spacing * total_mass / profile.dt});
        maximum_momentum_residual = std::max(maximum_momentum_residual,
            vector_norm(momentum_residual) / momentum_scale);
        maximum_positive_energy_excess = std::max(
            maximum_positive_energy_excess,
            std::max(physics.mechanical_energy
                    - initial_physics.mechanical_energy,
                0.0)
                / std::max(std::abs(initial_physics.mechanical_energy), 1.0));
        maximum_penetration = std::max(maximum_penetration,
            gpu_result.maximum_penetration_m);
        previous_physics = physics;
        double position_squared = 0.0;
        double density_squared = 0.0;
        double compression_squared = 0.0;
        double density_sum = 0.0;
        std::vector<double> position_errors;
        position_errors.reserve(gpu_snapshot.state.size());
        std::vector<std::pair<std::uint32_t, double>> position_records;
        position_records.reserve(gpu_snapshot.state.size());
        for (std::size_t index = 0U; index < gpu_snapshot.state.size(); ++index) {
            if (gpu_snapshot.state[index].sample_id
                    != cpu_result.state[index].sample_id
                || gpu_snapshot.state[index].sample_id
                    != permuted_snapshot.state[index].sample_id) {
                ++permutation_mismatch_steps;
                break;
            }
            const Vec3d delta{
                gpu_snapshot.state[index].current.x
                    - cpu_result.state[index].current.x,
                gpu_snapshot.state[index].current.y
                    - cpu_result.state[index].current.y,
                gpu_snapshot.state[index].current.z
                    - cpu_result.state[index].current.z};
            const double distance = std::sqrt(delta.x * delta.x
                + delta.y * delta.y + delta.z * delta.z);
            position_squared += distance * distance;
            position_errors.push_back(distance);
            position_records.emplace_back(
                gpu_snapshot.state[index].sample_id, distance);
            if (distance > maximum_position_error) {
                maximum_position_error = distance;
                maximum_position_error_id =
                    gpu_snapshot.state[index].sample_id;
                const std::array<double, 3> components{
                    std::abs(delta.x), std::abs(delta.y), std::abs(delta.z)};
                maximum_position_error_component = static_cast<std::uint32_t>(
                    std::distance(components.begin(),
                        std::max_element(components.begin(), components.end())));
            }
            gpu_minimum.x = std::min(gpu_minimum.x,
                gpu_snapshot.state[index].current.x);
            gpu_minimum.y = std::min(gpu_minimum.y,
                gpu_snapshot.state[index].current.y);
            gpu_minimum.z = std::min(gpu_minimum.z,
                gpu_snapshot.state[index].current.z);
            gpu_maximum.x = std::max(gpu_maximum.x,
                gpu_snapshot.state[index].current.x);
            gpu_maximum.y = std::max(gpu_maximum.y,
                gpu_snapshot.state[index].current.y);
            gpu_maximum.z = std::max(gpu_maximum.z,
                gpu_snapshot.state[index].current.z);
            const double density_error = std::abs(
                gpu_snapshot.density[index] - cpu_result.density[index])
                / profile.rest_density;
            density_squared += density_error * density_error;
            maximum_density_error = std::max(
                maximum_density_error, density_error);
            const double compression_error = std::max(
                gpu_snapshot.density[index] / profile.rest_density - 1.0, 0.0);
            observed_density_minimum = std::min(observed_density_minimum,
                gpu_snapshot.density[index]);
            observed_density_maximum = std::max(observed_density_maximum,
                gpu_snapshot.density[index]);
            density_sum += gpu_snapshot.density[index];
            compression_squared += compression_error * compression_error;
            maximum_compression_error = std::max(
                maximum_compression_error, compression_error);
            const Vec3d permutation_delta{
                gpu_snapshot.state[index].current.x
                    - permuted_snapshot.state[index].current.x,
                gpu_snapshot.state[index].current.y
                    - permuted_snapshot.state[index].current.y,
                gpu_snapshot.state[index].current.z
                    - permuted_snapshot.state[index].current.z};
            if (permutation_delta.x != 0.0 || permutation_delta.y != 0.0
                || permutation_delta.z != 0.0) {
                ++permutation_mismatch_steps;
                break;
            }
        }
        maximum_position_rmse = std::max(maximum_position_rmse,
            std::sqrt(position_squared
                / static_cast<double>(gpu_snapshot.state.size())));
        const PositionDistribution position = position_distribution(
            position_errors);
        maximum_position_p50 = std::max(maximum_position_p50, position.p50);
        maximum_position_p95 = std::max(maximum_position_p95, position.p95);
        maximum_position_p99 = std::max(maximum_position_p99, position.p99);
        position_metric_work.samples_loaded += position_errors.size();
        position_metric_work.squared_accumulations += position_errors.size();
        position_metric_work.sorted_values += position_errors.size();
        position_metric_work.nearest_rank_reads += 3U;
        ++position_metric_work.maximum_reads;
        position_metric_work.hashed_sample_records += position_records.size();
        position_metric_work.root_derivations += 2U;
        const std::string position_samples_root = position_sample_records_root(
            step_index + 1U, position_records);
        position_receipt_material += std::to_string(step_index + 1U) + ':'
            + std::to_string(double_bits(position.rmse)) + ':'
            + std::to_string(double_bits(position.p50)) + ':'
            + std::to_string(double_bits(position.p95)) + ':'
            + std::to_string(double_bits(position.p99)) + ':'
            + std::to_string(double_bits(position.maximum)) + ':'
            + position_samples_root + ':'
            + position_metric_work_root(position_metric_work) + '\n';
        if (product_trajectory_gate && step_index == 0U) {
            same_state_first_step_rmse = position.rmse;
            same_state_first_step_maximum = position.maximum;
        }
        maximum_density_rmse = std::max(maximum_density_rmse,
            std::sqrt(density_squared
                / static_cast<double>(gpu_snapshot.state.size())));
        maximum_compression_rmse = std::max(maximum_compression_rmse,
            std::sqrt(compression_squared
                / static_cast<double>(gpu_snapshot.state.size())));
        observed_density_mean = density_sum
            / static_cast<double>(gpu_snapshot.state.size());
        if (gpu_snapshot.active_pressure_ids != cpu_result.active_pressure_ids) {
            if (first_active_mismatch_step == 0U) {
                first_active_mismatch_step = step_index + 1U;
                first_gpu_active_count = static_cast<std::uint32_t>(
                    gpu_snapshot.active_pressure_ids.size());
                first_cpu_active_count = static_cast<std::uint32_t>(
                    cpu_result.active_pressure_ids.size());
                for (std::size_t index = 0U;
                     index < gpu_snapshot.state.size(); ++index) {
                    const std::uint32_t id = gpu_snapshot.state[index].sample_id;
                    const bool gpu_active = std::binary_search(
                        gpu_snapshot.active_pressure_ids.begin(),
                        gpu_snapshot.active_pressure_ids.end(), id);
                    const bool cpu_active = std::binary_search(
                        cpu_result.active_pressure_ids.begin(),
                        cpu_result.active_pressure_ids.end(), id);
                    if (gpu_active != cpu_active) {
                        first_active_mismatch_id = id;
                        first_gpu_mismatch_density = gpu_snapshot.density[index];
                        first_cpu_mismatch_density = cpu_result.density[index];
                        break;
                    }
                }
            }
            ++active_mismatch_steps;
        }
        if (product_trajectory_gate && step_index == 0U) {
            same_state_first_step_active_signature_equal =
                gpu_snapshot.active_pressure_ids
                == cpu_result.active_pressure_ids;
        }
        if (gpu_snapshot.active_pressure_ids
            != permuted_snapshot.active_pressure_ids) {
            ++permutation_mismatch_steps;
        }
        prior_gpu_state_root = compensated_state_root(gpu_compensated);
        prior_permuted_state_root = compensated_state_root(
            permuted_compensated);
        if (permutation_mismatch_steps != 0U) {
            break;
        }
        cpu_state = cpu_result.state;
        ++completed;
        if (product_trajectory_gate && step_index == 0U
            && same_state_first_step_maximum > 5.0e-6) {
            first_gate_failure = "same_state_position_max";
        } else if (product_trajectory_gate && step_index == 0U
            && !same_state_first_step_active_signature_equal) {
            first_gate_failure = "same_state_active_signature";
        } else if (maximum_position_rmse > 0.0025) {
            first_gate_failure = "position_rmse";
        } else if (product_trajectory_gate && maximum_position_p99 > 0.0025) {
            first_gate_failure = "position_p99";
        } else if (!product_trajectory_gate && maximum_position_error > 0.005) {
            first_gate_failure = "position_max";
        } else if (maximum_density_rmse > 0.05) {
            first_gate_failure = "density_rmse";
        } else if (maximum_density_error > 0.10) {
            first_gate_failure = "density_max";
        } else if (maximum_compression_rmse > 0.05) {
            first_gate_failure = "compression_rmse";
        } else if (maximum_compression_error > 0.10) {
            first_gate_failure = "compression_max";
        } else if (maximum_momentum_residual > 0.01) {
            first_gate_failure = "momentum";
        } else if (maximum_positive_energy_excess > 0.01) {
            first_gate_failure = "energy_excess";
        } else if (maximum_penetration > 0.0025) {
            first_gate_failure = "penetration";
        } else if (permutation_mismatch_steps != 0U) {
            first_gate_failure = "permutation";
        }
        if (!first_gate_failure.empty()) {
            first_gate_failure_step = step_index + 1U;
            break;
        }
    }
    NonlocalGpuGraphResult failure_graph;
    if (gpu_failure == NonlocalGpuFailure::CapacityExceeded
        || cpu_failure == NonlocalGpuFailure::CapacityExceeded) {
        failure_graph = build_reference_graph(
            profile, cpu_state, ghosts, false);
    }
    const double inset = 0.5 * profile.spacing;
    const bool closed_basin_bounds = completed != 0U
        && gpu_minimum.x >= inset - 0.0025
        && gpu_minimum.y >= inset - 0.0025
        && gpu_minimum.z >= inset - 0.0025
        && gpu_maximum.x <= profile.basin_extent.x - inset + 0.0025
        && gpu_maximum.y <= profile.basin_extent.y - inset + 0.0025
        && gpu_maximum.z <= profile.basin_extent.z - inset + 0.0025;
    const std::string executable_root = binary_root();
    const std::string environment = gpu.environment_json();
    const bool identity_valid = !executable_root.empty()
        && std::string(kContractRoot) != "unconfigured"
        && std::string(kSourceRoot) != "unconfigured"
        && std::string(kSourceCommit) != "unconfigured"
        && std::string(kSourceTree) != "unconfigured"
        && std::string(kCompilerFlags) != "unconfigured";
    const bool long_trajectory_position_passed =
        maximum_position_rmse <= 0.0025
        && (product_trajectory_gate
                ? maximum_position_p99 <= 0.0025
                : maximum_position_error <= 0.005);
    const bool same_state_first_step_passed = !product_trajectory_gate
        || (same_state_first_step_rmse <= 0.0025
            && same_state_first_step_maximum <= 5.0e-6
            && same_state_first_step_active_signature_equal);
    const bool passed = completed == steps
        && gpu_failure == NonlocalGpuFailure::None
        && cpu_failure == NonlocalGpuFailure::None
        && long_trajectory_position_passed
        && same_state_first_step_passed
        && maximum_density_rmse <= 0.05
        && maximum_density_error <= 0.10
        && maximum_compression_rmse <= 0.05
        && maximum_compression_error <= 0.10
        && maximum_momentum_residual <= 0.01
        && maximum_positive_energy_excess <= 0.01
        && maximum_penetration <= 0.0025
        && closed_basin_bounds
        && permutation_mismatch_steps == 0U
        && identity_valid;
    const bool work_refuted = !passed && budget == 128U
        && corrected_gpu_failure == NonlocalGpuFailure::WorkBudgetExceeded
        && permuted_gpu_failure == NonlocalGpuFailure::WorkBudgetExceeded
        && cpu_failure == NonlocalGpuFailure::None
        && failure_gpu_hvp <= budget
        && failure_gpu_hvp == failure_permuted_hvp
        && !failure_work_root.empty()
        && failure_work_root == failure_permuted_work_root
        && failure_restored_state_root == prior_gpu_state_root
        && failure_permuted_restored_state_root
            == prior_permuted_state_root;
    const bool physical_refuted = work_refuted || (!passed && completed > 0U
        && gpu_failure == NonlocalGpuFailure::None
        && cpu_failure == NonlocalGpuFailure::None
        && (!long_trajectory_position_passed
            || !same_state_first_step_passed
            || maximum_density_rmse > 0.05
            || maximum_density_error > 0.10
            || maximum_compression_rmse > 0.05
            || maximum_compression_error > 0.10
            || maximum_momentum_residual > 0.01
            || maximum_positive_energy_excess > 0.01
            || maximum_penetration > 0.0025
            || !closed_basin_bounds));
    const long double lattice_density = infinite_lattice_density(profile);
#if defined(NCGP4_EXPERIMENTAL)
    const char* refuted_status = "REFUTED_BOUNDED";
#else
    const char* refuted_status = "PHYSICS_REFUTED";
#endif
    const char* status = passed ? "PASS"
        : (physical_refuted ? refuted_status : "INCONCLUSIVE");
    const std::string receipt_root = nextengine::nonlocal::sha256_hex(
        receipt_material);
    const std::string snapshot_receipt_root =
        nextengine::nonlocal::sha256_hex(snapshot_receipt_material);
    position_metric_work.root_derivations += 2U;
    const std::string position_receipt_root =
        nextengine::nonlocal::sha256_hex(position_receipt_material);
    const std::string position_work_root = position_metric_work_root(
        position_metric_work);
    std::ostringstream result_material;
    result_material << std::setprecision(17)
                    << kResultDomain
                    << "status=" << status << '\n'
                    << "scenario=" << scenario << '\n'
                    << "steps=" << steps << '\n'
                    << "completed=" << completed << '\n'
                    << "budget=" << budget << '\n'
                    << "solver=" << static_cast<std::uint32_t>(solver_profile)
                    << '\n'
                    << "variant=" << static_cast<std::uint32_t>(gpu_variant)
                    << '\n'
                    << "arithmetic=" << arithmetic_profile << '\n'
                    << "failures=" << static_cast<std::uint32_t>(gpu_failure)
                    << ',' << static_cast<std::uint32_t>(
                           corrected_gpu_failure)
                    << ',' << static_cast<std::uint32_t>(
                           permuted_gpu_failure)
                    << ',' << static_cast<std::uint32_t>(cpu_failure) << '\n'
                    << "position=" << maximum_position_rmse << ','
                    << maximum_position_error << '\n'
                    << "density=" << maximum_density_rmse << ','
                    << maximum_density_error << ','
                    << maximum_compression_rmse << ','
                    << maximum_compression_error << ','
                    << observed_density_minimum << ','
                    << observed_density_maximum << ','
                    << observed_density_mean << '\n'
                    << "physics=" << maximum_momentum_residual << ','
                    << maximum_positive_energy_excess << ','
                    << maximum_penetration << ',' << closed_basin_bounds << ','
                    << initial_physics.mechanical_energy << '\n'
                    << "bounds=" << gpu_minimum.x << ',' << gpu_minimum.y
                    << ',' << gpu_minimum.z << ',' << gpu_maximum.x << ','
                    << gpu_maximum.y << ',' << gpu_maximum.z << '\n'
                    << "mismatch=" << active_mismatch_steps << ','
                    << first_active_mismatch_step << ','
                    << first_active_mismatch_id << ','
                    << first_gpu_active_count << ',' << first_cpu_active_count
                    << ',' << first_gpu_mismatch_density << ','
                    << first_cpu_mismatch_density << ','
                    << permutation_mismatch_steps << '\n'
                    << "first-gate=" << first_gate_failure_step << ','
                    << first_gate_failure << '\n'
                    << "maximum-work=" << maximum_gpu_hvp << ','
                    << maximum_cpu_hvp << ','
                    << maximum_active_pressure_centers << ','
                    << maximum_directed_pairs << ',' << maximum_degree << ','
                    << gpu.allocated_device_bytes() << '\n'
                    << "transfers=" << hot_host_to_device_bytes << ','
                    << hot_device_to_host_bytes << ','
                    << snapshot_device_to_host_bytes << '\n'
                    << "failure-work=" << failure_gpu_hvp << ','
                    << failure_permuted_hvp << ',' << failure_outer_trials
                    << ',' << failure_accepted_trials << ','
                    << failure_rejected_trials << ',' << failure_work_root
                    << ',' << failure_permuted_work_root << '\n'
                    << "failure-graph="
                    << static_cast<std::uint32_t>(failure_graph.failure) << ','
                    << failure_graph.maximum_degree << ','
                    << failure_graph.overflow_owner_id << ','
                    << failure_graph.overflow_dynamic_neighbors << ','
                    << failure_graph.overflow_ghost_neighbors << '\n'
                    << "count-mass=" << cpu_state.size() << ','
                    << static_cast<double>(cpu_state.size()) * profile.mass
                    << '\n'
                    << "profile=" << profile_semantic_root(profile) << ','
                    << profile.kernel_scale << ',' << profile.kappa << ','
                    << profile.lambda << ',' << profile.mu << ','
                    << profile.gamma << '\n'
                    << "roots=" << input_root << ',' << receipt_root << ','
                    << snapshot_receipt_root << ',' << prior_gpu_state_root
                    << ',' << prior_permuted_state_root << ','
                    << failure_restored_state_root << ','
                    << failure_permuted_restored_state_root << '\n'
                    << "identity=" << kContractRoot << ',' << kSourceRoot
                    << ',' << kSourceCommit << ',' << kSourceTree << ','
                    << kCompilerFlags << ',' << executable_root << '\n'
                    << "environment=" << environment << '\n';
    if (product_trajectory_gate) {
        result_material << "product-position=" << maximum_position_p50 << ','
                        << maximum_position_p95 << ','
                        << maximum_position_p99 << ','
                        << maximum_position_error_id << ','
                        << maximum_position_error_component << ','
                        << same_state_first_step_rmse << ','
                        << same_state_first_step_maximum << ','
                        << same_state_first_step_active_signature_equal << ','
                        << nearest_rank_index(4000U, 99U, 100U) << ','
                        << position_receipt_root << ',' << position_work_root
                        << '\n';
    }
    const std::string result_root = nextengine::nonlocal::sha256_hex(
        result_material.str());
    std::cout << std::setprecision(17)
              << "{\"schema\":\"" << kCorrespondenceSchema << "\""
              << ",\"status\":\"" << status << "\""
              << ",\"scenario\":\"" << scenario << "\",\"steps\":" << steps
              << ",\"arithmetic_profile\":\"" << arithmetic_profile << "\""
              << ",\"solver_profile\":"
              << static_cast<std::uint32_t>(solver_profile)
              << ",\"completed_steps\":" << completed
              << ",\"budget\":" << budget
              << ",\"gpu_failure\":" << static_cast<std::uint32_t>(gpu_failure)
              << ",\"corrected_gpu_failure\":"
              << static_cast<std::uint32_t>(corrected_gpu_failure)
              << ",\"permuted_gpu_failure\":"
              << static_cast<std::uint32_t>(permuted_gpu_failure)
              << ",\"cpu_failure\":" << static_cast<std::uint32_t>(cpu_failure)
              << ",\"position_rmse_max_m\":" << maximum_position_rmse
              << ",\"position_error_max_m\":" << maximum_position_error
              << ",\"position_p50_max_m\":" << maximum_position_p50
              << ",\"position_p95_max_m\":" << maximum_position_p95
              << ",\"position_p99_max_m\":" << maximum_position_p99
              << ",\"position_error_max_sample_id\":"
              << maximum_position_error_id
              << ",\"position_error_max_component\":"
              << maximum_position_error_component
              << ",\"position_p99_rank_index\":"
              << nearest_rank_index(4000U, 99U, 100U)
              << ",\"product_trajectory_gate\":"
              << (product_trajectory_gate ? "true" : "false")
              << ",\"position_maximum_is_diagnostic\":"
              << (product_trajectory_gate ? "true" : "false")
              << ",\"same_state_first_step_rmse_m\":"
              << same_state_first_step_rmse
              << ",\"same_state_first_step_max_m\":"
              << same_state_first_step_maximum
              << ",\"same_state_first_step_active_signature_equal\":"
              << (same_state_first_step_active_signature_equal
                      ? "true" : "false")
              << ",\"density_correspondence_rmse_max_fraction\":"
              << maximum_density_rmse
              << ",\"density_correspondence_error_max_fraction\":"
              << maximum_density_error
              << ",\"density_compression_rmse_max_fraction\":"
              << maximum_compression_rmse
              << ",\"density_compression_error_max_fraction\":"
              << maximum_compression_error
              << ",\"momentum_residual_max_fraction\":"
              << maximum_momentum_residual
              << ",\"positive_mechanical_energy_excess_max_fraction\":"
              << maximum_positive_energy_excess
              << ",\"maximum_penetration_m\":" << maximum_penetration
              << ",\"closed_basin_bounds\":"
              << (closed_basin_bounds ? "true" : "false")
              << ",\"initial_mechanical_energy_j\":"
              << initial_physics.mechanical_energy
              << ",\"density_observed_min_kg_m3\":"
              << observed_density_minimum
              << ",\"density_observed_max_kg_m3\":"
              << observed_density_maximum
              << ",\"density_observed_mean_kg_m3\":"
              << observed_density_mean
              << ",\"independent_infinite_lattice_density_kg_m3\":"
              << static_cast<double>(lattice_density)
              << ",\"independent_lattice_density_fraction\":"
              << static_cast<double>(lattice_density
                    / static_cast<long double>(profile.rest_density))
              << ",\"profile_root\":\"" << profile_semantic_root(profile)
              << "\",\"kernel_scale\":" << profile.kernel_scale
              << ",\"kappa\":" << profile.kappa
              << ",\"lambda\":" << profile.lambda
              << ",\"mu\":" << profile.mu
              << ",\"gamma\":" << profile.gamma
              << ",\"active_mismatch_steps\":" << active_mismatch_steps
              << ",\"first_active_mismatch_step\":"
              << first_active_mismatch_step
              << ",\"first_active_mismatch_id\":"
              << first_active_mismatch_id
              << ",\"first_gpu_active_count\":"
              << first_gpu_active_count
              << ",\"first_cpu_active_count\":"
              << first_cpu_active_count
              << ",\"first_gpu_mismatch_density\":"
              << first_gpu_mismatch_density
              << ",\"first_cpu_mismatch_density\":"
              << first_cpu_mismatch_density
              << ",\"first_gate_failure_step\":"
              << first_gate_failure_step
              << ",\"first_gate_failure\":\""
              << first_gate_failure << "\""
              << ",\"permutation_mismatch_steps\":"
              << permutation_mismatch_steps
              << ",\"gpu_hvp_max\":" << maximum_gpu_hvp
              << ",\"cpu_hvp_max\":" << maximum_cpu_hvp
              << ",\"active_pressure_centers_max\":"
              << maximum_active_pressure_centers
              << ",\"directed_pairs_max\":" << maximum_directed_pairs
              << ",\"maximum_degree\":" << maximum_degree
              << ",\"allocated_device_bytes\":"
              << gpu.allocated_device_bytes()
              << ",\"hot_host_to_device_bytes\":"
              << hot_host_to_device_bytes
              << ",\"hot_device_to_host_bytes\":"
              << hot_device_to_host_bytes
              << ",\"snapshot_device_to_host_bytes\":"
              << snapshot_device_to_host_bytes
              << ",\"failure_gpu_hvp\":" << failure_gpu_hvp
              << ",\"failure_permuted_hvp\":" << failure_permuted_hvp
              << ",\"failure_outer_trials\":" << failure_outer_trials
              << ",\"failure_accepted_trials\":"
              << failure_accepted_trials
              << ",\"failure_rejected_trials\":"
              << failure_rejected_trials
              << ",\"failure_work_root\":\"" << failure_work_root
              << "\",\"failure_permuted_work_root\":\""
              << failure_permuted_work_root << "\""
              << ",\"failure_restored_state_root\":\""
              << failure_restored_state_root << "\""
              << ",\"failure_permuted_restored_state_root\":\""
              << failure_permuted_restored_state_root << "\""
              << ",\"particle_count\":" << cpu_state.size()
              << ",\"mass_kg\":"
              << static_cast<double>(cpu_state.size()) * profile.mass
              << ",\"input_root\":\"" << input_root
              << "\",\"receipt_root\":\""
              << receipt_root
              << "\",\"snapshot_receipt_root\":\""
              << snapshot_receipt_root
              << "\",\"position_receipt_root\":\""
              << position_receipt_root
              << "\",\"position_metric_work_root\":\""
              << position_work_root
              << "\",\"final_state_root\":\"" << prior_gpu_state_root
              << "\",\"final_permuted_state_root\":\""
              << prior_permuted_state_root
              << "\",\"result_root\":\"" << result_root
              << "\",\"pre_failure_graph_failure\":"
              << static_cast<std::uint32_t>(failure_graph.failure)
              << ",\"pre_failure_maximum_degree\":"
              << failure_graph.maximum_degree
              << ",\"overflow_owner_id\":"
              << failure_graph.overflow_owner_id
              << ",\"overflow_dynamic_neighbors\":"
              << failure_graph.overflow_dynamic_neighbors
              << ",\"overflow_ghost_neighbors\":"
              << failure_graph.overflow_ghost_neighbors
              << ",\"state_bounds_min\":[" << gpu_minimum.x << ','
              << gpu_minimum.y << ',' << gpu_minimum.z << "]"
              << ",\"state_bounds_max\":[" << gpu_maximum.x << ','
              << gpu_maximum.y << ',' << gpu_maximum.z << ']'
              << ",\"contract_root\":\"" << kContractRoot
              << "\",\"source_root\":\"" << kSourceRoot
              << "\",\"source_commit\":\"" << kSourceCommit
              << "\",\"source_tree\":\"" << kSourceTree
              << "\",\"compiler_flags\":\"" << kCompilerFlags
              << "\",\"environment\":" << environment
              << ",\"exact_command\":\""
              << (product_trajectory_gate
                      ? "--correspondence-4k-product "
                      : (solver_profile
                                == NonlocalGpuSolverProfile::Unpreconditioned
                              ? "--correspondence-4k-unpreconditioned "
                              : "--correspondence-4k "))
              << scenario << ' ' << steps << ' ' << budget << "\""
              << ",\"timing_status\":\"NOT_RUN\""
              << ",\"binary_root\":\"" << executable_root
              << "\"}\n";
    return passed ? 0 : (physical_refuted ? 37 : 53);
}

#if defined(NCGP4_EXPERIMENTAL)
struct SameStateOperatorError {
    bool valid = false;
    bool active_signature_equal = false;
    double gradient_relative_l2 = std::numeric_limits<double>::infinity();
    double hvp_relative_l2 = std::numeric_limits<double>::infinity();
    double hvp_cosine_loss = std::numeric_limits<double>::infinity();
};

SameStateOperatorError compare_same_state_operator(
    NonlocalGpuWorkspace& workspace,
    const NonlocalGpuProfile& profile,
    const NonlocalGpuPublicSnapshot& snapshot,
    const std::vector<NonlocalGpuGhost>& ghosts) {
    SameStateOperatorError result;
    std::vector<Vec3d> direction;
    direction.reserve(snapshot.state.size());
    for (const NonlocalGpuSample& sample : snapshot.state) {
        const int id = static_cast<int>(sample.sample_id);
        direction.push_back({
            static_cast<double>(id % 17 - 8) / 16.0,
            static_cast<double>(id % 13 - 6) / 8.0,
            static_cast<double>(id % 11 - 5) / 8.0});
    }
    const NonlocalGpuEvaluationResult actual = workspace.evaluate(
        &direction, NonlocalGpuVariant::CompensatedScaleF32, true, false);
    const NonlocalGpuEvaluationResult expected = evaluate_reference(
        profile, snapshot.state, ghosts, &direction,
        NonlocalGpuVariant::CompensatedScaleF32);
    if (actual.failure != NonlocalGpuFailure::None
        || expected.failure != NonlocalGpuFailure::None
        || actual.gradient.size() != expected.gradient.size()
        || actual.hvp.size() != expected.hvp.size()
        || actual.gradient.empty()) {
        return result;
    }
    long double gradient_difference = 0.0L;
    long double gradient_expected = 0.0L;
    long double hvp_difference = 0.0L;
    long double hvp_expected = 0.0L;
    long double hvp_actual = 0.0L;
    long double hvp_product = 0.0L;
    for (std::size_t index = 0U; index < actual.gradient.size(); ++index) {
        const auto accumulate = [](const Vec3d& lhs, const Vec3d& rhs,
                                    long double& difference,
                                    long double& expected_squared) {
            const long double dx = static_cast<long double>(lhs.x) - rhs.x;
            const long double dy = static_cast<long double>(lhs.y) - rhs.y;
            const long double dz = static_cast<long double>(lhs.z) - rhs.z;
            difference += dx * dx + dy * dy + dz * dz;
            expected_squared += static_cast<long double>(rhs.x) * rhs.x
                + static_cast<long double>(rhs.y) * rhs.y
                + static_cast<long double>(rhs.z) * rhs.z;
        };
        accumulate(actual.gradient[index], expected.gradient[index],
            gradient_difference, gradient_expected);
        accumulate(actual.hvp[index], expected.hvp[index],
            hvp_difference, hvp_expected);
        hvp_actual += static_cast<long double>(actual.hvp[index].x)
                * actual.hvp[index].x
            + static_cast<long double>(actual.hvp[index].y)
                * actual.hvp[index].y
            + static_cast<long double>(actual.hvp[index].z)
                * actual.hvp[index].z;
        hvp_product += static_cast<long double>(actual.hvp[index].x)
                * expected.hvp[index].x
            + static_cast<long double>(actual.hvp[index].y)
                * expected.hvp[index].y
            + static_cast<long double>(actual.hvp[index].z)
                * expected.hvp[index].z;
    }
    if (!(gradient_expected > 0.0L) || !(hvp_expected > 0.0L)
        || !(hvp_actual > 0.0L)) return result;
    result.gradient_relative_l2 = static_cast<double>(
        std::sqrt(gradient_difference / gradient_expected));
    result.hvp_relative_l2 = static_cast<double>(
        std::sqrt(hvp_difference / hvp_expected));
    result.hvp_cosine_loss = static_cast<double>(1.0L
        - hvp_product / std::sqrt(hvp_actual * hvp_expected));
    result.active_signature_equal = actual.active_pressure_ids
        == expected.active_pressure_ids;
    result.valid = std::isfinite(result.gradient_relative_l2)
        && std::isfinite(result.hvp_relative_l2)
        && std::isfinite(result.hvp_cosine_loss);
    return result;
}

int run_ncgp4_solver_diagnosis() {
    constexpr std::uint32_t kPrefixSteps = 38U;
    constexpr std::uint32_t kBudget = 128U;
    const NonlocalGpuProfile profile = nonlocal_water_corrected_profile();
    const auto ghosts = canonicalize_ghosts_binary32(make_basin_ghosts(profile));
    const auto coherent_input = trajectory_initial(
        profile, "hydrostatic-hold", false);
    const auto permuted_input = trajectory_initial(
        profile, "hydrostatic-hold", true);
    NonlocalGpuWorkspace corrected(profile);
    NonlocalGpuWorkspace permuted(profile);
    NonlocalGpuWorkspace unpreconditioned(profile);
    if (corrected.upload(coherent_input, ghosts, true)
            != NonlocalGpuFailure::None
        || permuted.upload(permuted_input, ghosts, true)
            != NonlocalGpuFailure::None
        || unpreconditioned.upload(coherent_input, ghosts, true)
            != NonlocalGpuFailure::None) return 51;
    for (std::uint32_t index = 0U; index < kPrefixSteps; ++index) {
        const auto a = corrected.step(kBudget, NonlocalGpuSolverProfile::Jacobi,
            NonlocalGpuVariant::CompensatedScaleF32, false, false);
        const auto b = permuted.step(kBudget, NonlocalGpuSolverProfile::Jacobi,
            NonlocalGpuVariant::CompensatedScaleF32, false, false);
        const auto c = unpreconditioned.step(kBudget,
            NonlocalGpuSolverProfile::Jacobi,
            NonlocalGpuVariant::CompensatedScaleF32, false, false);
        if (a.failure != NonlocalGpuFailure::None
            || b.failure != NonlocalGpuFailure::None
            || c.failure != NonlocalGpuFailure::None
            || step_work_semantic_root(profile, a)
                != step_work_semantic_root(profile, b)
            || step_work_semantic_root(profile, a)
                != step_work_semantic_root(profile, c)) return 52;
    }
    const auto corrected_before = corrected.capture_compensated_state();
    const auto permuted_before = permuted.capture_compensated_state();
    const auto unpreconditioned_before =
        unpreconditioned.capture_compensated_state();
    const auto public_before = corrected.capture_public_snapshot();
    if (corrected_before.failure != NonlocalGpuFailure::None
        || permuted_before.failure != NonlocalGpuFailure::None
        || unpreconditioned_before.failure != NonlocalGpuFailure::None
        || public_before.failure != NonlocalGpuFailure::None) return 53;
    const SameStateOperatorError operator_error = compare_same_state_operator(
        corrected, profile, public_before, ghosts);
    const auto corrected_result = corrected.step(kBudget,
        NonlocalGpuSolverProfile::Jacobi,
        NonlocalGpuVariant::CompensatedScaleF32, false, false, true);
    const auto permuted_result = permuted.step(kBudget,
        NonlocalGpuSolverProfile::Jacobi,
        NonlocalGpuVariant::CompensatedScaleF32, false, false, true);
    const auto unpreconditioned_result = unpreconditioned.step(kBudget,
        NonlocalGpuSolverProfile::Unpreconditioned,
        NonlocalGpuVariant::CompensatedScaleF32, false, false, true);
    const std::string corrected_trace_root = solver_trace_semantic_root(
        corrected_result);
    const std::string permuted_trace_root = solver_trace_semantic_root(
        permuted_result);
    const std::string unpreconditioned_trace_root = solver_trace_semantic_root(
        unpreconditioned_result);
    NonlocalGpuStepResult event_mutation = corrected_result;
    if (!event_mutation.trace.empty()) {
        event_mutation.trace.front().gradient_norm = std::nextafter(
            event_mutation.trace.front().gradient_norm,
            std::numeric_limits<double>::infinity());
    }
    NonlocalGpuStepResult work_mutation = corrected_result;
    ++work_mutation.hvp_used;
    const bool event_mutation_rejected = solver_trace_valid(event_mutation)
        && solver_trace_semantic_root(event_mutation) != corrected_trace_root;
    const bool work_mutation_rejected = !solver_trace_valid(work_mutation);
    std::uint64_t inner_events = 0U;
    std::uint64_t trial_events = 0U;
    std::uint64_t forcing_events = 0U;
    std::uint64_t radius_events = 0U;
    std::uint64_t negative_curvature_events = 0U;
    double diagonal_min = std::numeric_limits<double>::infinity();
    double diagonal_max = 0.0;
    std::uint64_t inertia_floor_components = 0U;
    for (const NonlocalGpuSolverTraceEvent& event : corrected_result.trace) {
        if (event.kind == NonlocalGpuTraceKind::Inner) ++inner_events;
        if (event.kind == NonlocalGpuTraceKind::Trial) ++trial_events;
        if (event.reason == NonlocalGpuTraceReason::ForcingConverged) {
            ++forcing_events;
        }
        if (event.reason == NonlocalGpuTraceReason::TrustRadius) ++radius_events;
        if (event.reason == NonlocalGpuTraceReason::NegativeCurvature) {
            ++negative_curvature_events;
        }
        if (event.kind == NonlocalGpuTraceKind::Diagonal) {
            diagonal_min = std::min(diagonal_min, event.diagonal_min_abs);
            diagonal_max = std::max(diagonal_max, event.diagonal_max_abs);
            inertia_floor_components += event.inertia_floor_components;
        }
    }
    if (!std::isfinite(diagonal_min)) diagonal_min = 0.0;
    const std::string before_root = compensated_state_root(corrected_before);
    const std::string permuted_before_root = compensated_state_root(
        permuted_before);
    const std::string unpreconditioned_before_root = compensated_state_root(
        unpreconditioned_before);
    const bool trace_valid = solver_trace_valid(corrected_result)
        && solver_trace_valid(permuted_result)
        && solver_trace_valid(unpreconditioned_result);
    const bool permutation_exact = corrected_trace_root == permuted_trace_root
        && before_root == permuted_before_root;
    const bool controls_pass = event_mutation_rejected
        && work_mutation_rejected;
    const bool apparatus_pass = trace_valid && permutation_exact
        && controls_pass && operator_error.valid;
    const std::string executable_root = binary_root();
    std::ostringstream result_material;
    result_material << std::setprecision(17)
                    << "nextengine.nonlocal.ncgp4.diagnosis-result.v1\n"
                    << NCGP4_CONTRACT_ROOT << '\n' << NCGP4_SOURCE_ROOT << '\n'
                    << NCGP4_SOURCE_COMMIT << '\n' << NCGP4_SOURCE_TREE << '\n'
                    << executable_root << '\n' << before_root << '\n'
                    << corrected_trace_root << '\n' << permuted_trace_root
                    << '\n' << unpreconditioned_trace_root << '\n'
                    << static_cast<std::uint32_t>(corrected_result.failure)
                    << '\n' << corrected_result.hvp_used << '\n'
                    << inner_events << '\n' << trial_events << '\n'
                    << diagonal_min << '\n' << diagonal_max << '\n'
                    << operator_error.gradient_relative_l2 << '\n'
                    << operator_error.hvp_relative_l2 << '\n'
                    << operator_error.hvp_cosine_loss << '\n';
    const std::string result_root = nextengine::nonlocal::sha256_hex(
        result_material.str());
    std::cout << std::setprecision(17)
              << "{\"schema\":\"nextengine.nonlocal.ncgp4.solver-diagnosis.v1\""
              << ",\"status\":\"" << (apparatus_pass ? "PASS" : "FAIL")
              << "\",\"prefix_steps\":" << kPrefixSteps
              << ",\"budget\":" << kBudget
              << ",\"corrected_failure\":"
              << static_cast<std::uint32_t>(corrected_result.failure)
              << ",\"corrected_hvp\":" << corrected_result.hvp_used
              << ",\"corrected_outer\":" << corrected_result.outer_trials
              << ",\"corrected_accepted\":"
              << corrected_result.work.accepted_trials
              << ",\"corrected_rejected\":"
              << corrected_result.work.rejected_trials
              << ",\"corrected_radius_shrinks\":"
              << corrected_result.work.radius_shrinks
              << ",\"corrected_radius_expands\":"
              << corrected_result.work.radius_expands
              << ",\"unpreconditioned_failure\":"
              << static_cast<std::uint32_t>(unpreconditioned_result.failure)
              << ",\"unpreconditioned_hvp\":"
              << unpreconditioned_result.hvp_used
              << ",\"unpreconditioned_outer\":"
              << unpreconditioned_result.outer_trials
              << ",\"inner_events\":" << inner_events
              << ",\"trial_events\":" << trial_events
              << ",\"forcing_events\":" << forcing_events
              << ",\"radius_events\":" << radius_events
              << ",\"negative_curvature_events\":"
              << negative_curvature_events
              << ",\"diagonal_min_abs\":" << diagonal_min
              << ",\"diagonal_max_abs\":" << diagonal_max
              << ",\"inertia_floor_components\":"
              << inertia_floor_components
              << ",\"operator_gradient_relative_l2\":"
              << operator_error.gradient_relative_l2
              << ",\"operator_hvp_relative_l2\":"
              << operator_error.hvp_relative_l2
              << ",\"operator_hvp_cosine_loss\":"
              << operator_error.hvp_cosine_loss
              << ",\"operator_active_signature_equal\":"
              << (operator_error.active_signature_equal ? "true" : "false")
              << ",\"trace_valid\":" << (trace_valid ? "true" : "false")
              << ",\"permutation_exact\":"
              << (permutation_exact ? "true" : "false")
              << ",\"event_mutation_rejected\":"
              << (event_mutation_rejected ? "true" : "false")
              << ",\"work_mutation_rejected\":"
              << (work_mutation_rejected ? "true" : "false")
              << ",\"pre_step_state_root\":\"" << before_root
              << "\",\"permuted_pre_step_state_root\":\""
              << permuted_before_root
              << "\",\"unpreconditioned_pre_step_state_root\":\""
              << unpreconditioned_before_root
              << "\",\"corrected_trace_root\":\"" << corrected_trace_root
              << "\",\"permuted_trace_root\":\"" << permuted_trace_root
              << "\",\"unpreconditioned_trace_root\":\""
              << unpreconditioned_trace_root
              << "\",\"result_root\":\"" << result_root
              << "\",\"contract_root\":\"" << NCGP4_CONTRACT_ROOT
              << "\",\"source_root\":\"" << NCGP4_SOURCE_ROOT
              << "\",\"source_commit\":\"" << NCGP4_SOURCE_COMMIT
              << "\",\"source_tree\":\"" << NCGP4_SOURCE_TREE
              << "\",\"compiler_flags\":\"" << NCGP4_COMPILER_FLAGS
              << "\",\"binary_root\":\"" << executable_root
              << "\",\"environment\":" << corrected.environment_json()
              << ",\"events\":[";
    for (std::size_t index = 0U; index < corrected_result.trace.size(); ++index) {
        const auto& event = corrected_result.trace[index];
        if (index != 0U) std::cout << ',';
        std::cout << "{\"sequence\":" << event.sequence
                  << ",\"outer\":" << event.outer
                  << ",\"inner\":" << event.inner
                  << ",\"kind\":" << static_cast<std::uint32_t>(event.kind)
                  << ",\"reason\":"
                  << static_cast<std::uint32_t>(event.reason)
                  << ",\"hvp\":" << event.hvp_used
                  << ",\"active\":" << event.active_pressure_centers
                  << ",\"pairs\":" << event.directed_pairs
                  << ",\"radius_before\":" << event.radius_before
                  << ",\"radius_after\":" << event.radius_after
                  << ",\"gradient_norm\":" << event.gradient_norm
                  << ",\"scaled_residual\":"
                  << event.scaled_displacement_residual
                  << ",\"diag_min\":" << event.diagonal_min_abs
                  << ",\"diag_max\":" << event.diagonal_max_abs
                  << ",\"floor_components\":"
                  << event.inertia_floor_components
                  << ",\"initial_true_residual\":"
                  << event.initial_true_residual
                  << ",\"initial_preconditioned_residual\":"
                  << event.initial_preconditioned_residual
                  << ",\"true_residual\":" << event.true_residual
                  << ",\"preconditioned_residual\":"
                  << event.preconditioned_residual
                  << ",\"forcing\":" << event.forcing
                  << ",\"curvature\":" << event.curvature
                  << ",\"alpha\":" << event.alpha
                  << ",\"beta\":" << event.beta
                  << ",\"step_norm\":" << event.step_norm
                  << ",\"predicted_reduction\":"
                  << event.predicted_reduction
                  << ",\"actual_reduction\":" << event.actual_reduction
                  << ",\"rho\":" << event.rho << '}';
    }
    std::cout << "]}\n";
    return apparatus_pass ? 0 : 54;
}
#endif

#if defined(NCGP5_EXPERIMENTAL)
struct Ncgp5StepRecord {
    std::uint32_t step = 0U;
    std::uint32_t maximum_id = 0U;
    std::uint32_t maximum_component = 0U;
    double p50_position_error = 0.0;
    double p95_position_error = 0.0;
    double p99_position_error = 0.0;
    double maximum_position_error = 0.0;
    double position_rmse = 0.0;
    double maximum_velocity_error = 0.0;
    double maximum_density_error = 0.0;
    double outlier_density_error = 0.0;
    double outlier_face_distance = 0.0;
    bool gpu_active = false;
    bool cpu_active = false;
    std::uint32_t inferred_face_mask = 0U;
    std::uint32_t contact_face_mask = 0U;
    Vec3d contact_impulse;
    Vec3d gpu_position;
    Vec3d cpu_position;
    Vec3d position_delta;
    Vec3d gpu_velocity;
    Vec3d cpu_velocity;
    Vec3d current_high;
    Vec3d current_low;
    Vec3d predicted_high;
    Vec3d predicted_low;
    std::uint32_t gpu_hvp = 0U;
    std::uint32_t cpu_hvp = 0U;
    std::uint64_t gpu_outer = 0U;
    std::uint64_t gpu_accepted = 0U;
    std::uint64_t gpu_rejected = 0U;
    std::uint64_t gpu_radius_shrinks = 0U;
    std::uint64_t contact_projections = 0U;
    std::uint64_t boundary_face_mask_xor = 0U;
    std::uint32_t neighbor_count = 0U;
    std::uint32_t active_neighbor_centers = 0U;
    std::string sample_records_root;
    std::string neighbor_root;
    std::string graph_work_root;
    std::string step_work_root;
    std::string permuted_step_work_root;
    std::string cpu_step_work_root;
    std::string trace_root;
    std::string state_root;
};

struct Ncgp5Discriminator {
    bool valid = false;
    bool active_signature_equal = false;
    SameStateOperatorError operator_error;
    std::uint32_t gpu_hvp = 0U;
    std::uint32_t cpu_hvp = 0U;
    std::uint32_t gpu_outer = 0U;
    std::uint32_t cpu_outer = 0U;
    double position_rmse = std::numeric_limits<double>::infinity();
    double position_maximum = std::numeric_limits<double>::infinity();
    std::string input_root;
    std::string gpu_result_root;
    std::string cpu_result_root;
    std::string trace_root;
};

std::string cpu_state_semantic_root(
    const std::vector<NonlocalGpuSample>& unordered_state) {
    std::vector<NonlocalGpuSample> state = unordered_state;
    std::sort(state.begin(), state.end(), [](const auto& lhs, const auto& rhs) {
        return lhs.sample_id < rhs.sample_id;
    });
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp5.cpu-state.v1\n";
    for (const NonlocalGpuSample& sample : state) {
        material << sample.sample_id << ':' << std::hex;
        for (const double value : std::array<double, 9>{sample.reference.x,
                 sample.reference.y, sample.reference.z, sample.current.x,
                 sample.current.y, sample.current.z, sample.velocity.x,
                 sample.velocity.y, sample.velocity.z}) {
            material << double_bits(value) << ',';
        }
        material << std::dec << '\n';
    }
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::uint32_t inferred_inset_face_mask(
    const NonlocalGpuProfile& profile, Vec3d position) {
    constexpr double kFaceTolerance = 2.0e-7;
    const double lower = 0.5 * profile.spacing;
    const Vec3d upper{profile.basin_extent.x - lower,
        profile.basin_extent.y - lower, profile.basin_extent.z - lower};
    std::uint32_t mask = 0U;
    if (std::abs(position.x - lower) <= kFaceTolerance) mask |= 1U;
    if (std::abs(position.x - upper.x) <= kFaceTolerance) mask |= 2U;
    if (std::abs(position.y - lower) <= kFaceTolerance) mask |= 4U;
    if (std::abs(position.y - upper.y) <= kFaceTolerance) mask |= 8U;
    if (std::abs(position.z - lower) <= kFaceTolerance) mask |= 16U;
    if (std::abs(position.z - upper.z) <= kFaceTolerance) mask |= 32U;
    return mask;
}

double inset_face_distance(
    const NonlocalGpuProfile& profile, Vec3d position) {
    const double lower = 0.5 * profile.spacing;
    const Vec3d upper{profile.basin_extent.x - lower,
        profile.basin_extent.y - lower, profile.basin_extent.z - lower};
    return std::min({position.x - lower, upper.x - position.x,
        position.y - lower, upper.y - position.y, position.z - lower,
        upper.z - position.z});
}

Ncgp5Discriminator run_ncgp5_discriminator(
    NonlocalGpuWorkspace& source_workspace,
    const NonlocalGpuProfile& profile,
    const NonlocalGpuPublicSnapshot& source_snapshot,
    const std::vector<NonlocalGpuGhost>& ghosts) {
    Ncgp5Discriminator result;
    result.operator_error = compare_same_state_operator(
        source_workspace, profile, source_snapshot, ghosts);
    result.input_root = input_semantic_root(
        profile, source_snapshot.state, ghosts);
    NonlocalGpuWorkspace synchronized_gpu(profile);
    if (synchronized_gpu.upload(source_snapshot.state, ghosts, true)
        != NonlocalGpuFailure::None) return result;
    const NonlocalGpuStepResult gpu_step = synchronized_gpu.step(128U,
        NonlocalGpuSolverProfile::Unpreconditioned,
        NonlocalGpuVariant::CompensatedScaleF32, false, false, true);
    const NonlocalGpuPublicSnapshot gpu_snapshot =
        synchronized_gpu.capture_public_snapshot();
    const NonlocalGpuStepResult cpu_step = step_reference(profile,
        source_snapshot.state, ghosts, 128U, NonlocalGpuVariant::Corrected,
        true);
    result.gpu_hvp = gpu_step.hvp_used;
    result.cpu_hvp = cpu_step.hvp_used;
    result.gpu_outer = gpu_step.outer_trials;
    result.cpu_outer = cpu_step.outer_trials;
    result.gpu_result_root = step_semantic_root(
        profile, result.input_root, gpu_step);
    result.cpu_result_root = step_semantic_root(
        profile, result.input_root, cpu_step);
    result.trace_root = solver_trace_semantic_root(gpu_step);
    if (gpu_step.failure != NonlocalGpuFailure::None
        || cpu_step.failure != NonlocalGpuFailure::None
        || gpu_snapshot.failure != NonlocalGpuFailure::None
        || gpu_snapshot.state.size() != cpu_step.state.size()
        || gpu_snapshot.state.empty()
        || !solver_trace_valid(gpu_step)) return result;
    long double squared = 0.0L;
    double maximum = 0.0;
    for (std::size_t index = 0U; index < gpu_snapshot.state.size(); ++index) {
        if (gpu_snapshot.state[index].sample_id
            != cpu_step.state[index].sample_id) return result;
        const Vec3d delta{subtract(gpu_snapshot.state[index].current,
            cpu_step.state[index].current)};
        const double error = std::sqrt(
            delta.x * delta.x + delta.y * delta.y + delta.z * delta.z);
        squared += static_cast<long double>(error) * error;
        maximum = std::max(maximum, error);
    }
    result.position_rmse = static_cast<double>(std::sqrt(
        squared / static_cast<long double>(gpu_snapshot.state.size())));
    result.position_maximum = maximum;
    result.active_signature_equal = gpu_snapshot.active_pressure_ids
        == cpu_step.active_pressure_ids;
    result.valid = result.operator_error.valid
        && result.operator_error.active_signature_equal
        && result.position_rmse <= 0.0025
        && result.position_maximum <= 0.005;
    return result;
}

int run_ncgp5_step92_diagnosis() {
    constexpr std::uint32_t kLastStep = 92U;
    constexpr std::uint32_t kRecordFirst = 80U;
    const NonlocalGpuProfile profile = nonlocal_water_corrected_profile();
    const auto ghosts = canonicalize_ghosts_binary32(make_basin_ghosts(profile));
    auto cpu_state = trajectory_initial(profile, "hydrostatic-hold", false);
    const auto coherent_input = cpu_state;
    const auto permuted_input = trajectory_initial(
        profile, "hydrostatic-hold", true);
    const std::string input_root = input_semantic_root(
        profile, coherent_input, ghosts);
    NonlocalGpuWorkspace gpu(profile);
    NonlocalGpuWorkspace permuted(profile);
    if (gpu.upload(coherent_input, ghosts, true) != NonlocalGpuFailure::None
        || permuted.upload(permuted_input, ghosts, true)
            != NonlocalGpuFailure::None) return 51;
    std::vector<Ncgp5StepRecord> records;
    Ncgp5Discriminator slope_discriminator;
    Ncgp5Discriminator pre_failure_discriminator;
    std::uint32_t slope_step = 0U;
    double prior_maximum = 0.0;
    double prior_slope = 0.0;
    bool apparatus_valid = true;
    bool permutation_exact = true;
    bool sample_mutation_rejected = true;
    bool neighbor_mutation_rejected = true;
    NonlocalGpuStepResult final_gpu_step;
    std::ostringstream all_receipts;
    all_receipts << "nextengine.nonlocal.ncgp5.step92-receipts.v1\n";
    for (std::uint32_t step = 1U; step <= kLastStep; ++step) {
        const bool diagnostic = step >= kRecordFirst;
        const NonlocalGpuStepResult gpu_step = gpu.step(128U,
            NonlocalGpuSolverProfile::Unpreconditioned,
            NonlocalGpuVariant::CompensatedScaleF32, false, false,
            diagnostic);
        const NonlocalGpuStepResult permuted_step = permuted.step(128U,
            NonlocalGpuSolverProfile::Unpreconditioned,
            NonlocalGpuVariant::CompensatedScaleF32, false, false,
            diagnostic);
        const NonlocalGpuPublicSnapshot gpu_snapshot =
            gpu.capture_public_snapshot();
        const NonlocalGpuPublicSnapshot permuted_snapshot =
            permuted.capture_public_snapshot();
        const NonlocalGpuCompensatedStateSnapshot compensated =
            gpu.capture_compensated_state();
        const NonlocalGpuCompensatedStateSnapshot permuted_compensated =
            permuted.capture_compensated_state();
        const NonlocalGpuStepResult cpu_step = step_reference(profile,
            cpu_state, ghosts, 128U, NonlocalGpuVariant::Corrected, true);
        all_receipts << step << ':'
                     << step_work_semantic_root(profile, gpu_step) << ':'
                     << step_work_semantic_root(profile, permuted_step) << ':'
                     << step_work_semantic_root(profile, cpu_step) << ':'
                     << work_semantic_root(gpu_snapshot.work) << ':'
                     << work_semantic_root(permuted_snapshot.work) << ':'
                     << work_semantic_root(compensated.work) << ':'
                     << work_semantic_root(permuted_compensated.work) << '\n';
        if (gpu_step.failure != NonlocalGpuFailure::None
            || permuted_step.failure != NonlocalGpuFailure::None
            || cpu_step.failure != NonlocalGpuFailure::None
            || gpu_snapshot.failure != NonlocalGpuFailure::None
            || permuted_snapshot.failure != NonlocalGpuFailure::None
            || compensated.failure != NonlocalGpuFailure::None
            || permuted_compensated.failure != NonlocalGpuFailure::None
            || gpu_snapshot.state.size() != cpu_step.state.size()
            || gpu_snapshot.state.size() != permuted_snapshot.state.size()) {
            apparatus_valid = false;
            break;
        }
        if (diagnostic
            && (gpu_step.diagnostic_contact_impulse.size()
                    != gpu_snapshot.state.size()
                || gpu_step.diagnostic_contact_face_masks.size()
                    != gpu_snapshot.state.size()
                || permuted_step.diagnostic_contact_impulse.size()
                    != permuted_snapshot.state.size()
                || permuted_step.diagnostic_contact_face_masks.size()
                    != permuted_snapshot.state.size())) {
            apparatus_valid = false;
            break;
        }
        if (compensated_state_root(compensated)
                != compensated_state_root(permuted_compensated)
            || gpu_snapshot.active_pressure_ids
                != permuted_snapshot.active_pressure_ids) {
            permutation_exact = false;
            apparatus_valid = false;
            break;
        }
        std::vector<double> position_errors;
        position_errors.reserve(gpu_snapshot.state.size());
        long double position_squared = 0.0L;
        double maximum_position = -1.0;
        double maximum_velocity = 0.0;
        double maximum_density = 0.0;
        std::size_t maximum_index = 0U;
        std::ostringstream sample_material;
        sample_material << std::setprecision(17)
                        << "nextengine.nonlocal.ncgp5.sample-records.v1\n"
                        << step << '\n';
        for (std::size_t index = 0U; index < gpu_snapshot.state.size(); ++index) {
            if (gpu_snapshot.state[index].sample_id
                    != cpu_step.state[index].sample_id
                || gpu_snapshot.state[index].sample_id
                    != permuted_snapshot.state[index].sample_id) {
                apparatus_valid = false;
                break;
            }
            const Vec3d position_delta = subtract(
                gpu_snapshot.state[index].current,
                cpu_step.state[index].current);
            const Vec3d velocity_delta = subtract(
                gpu_snapshot.state[index].velocity,
                cpu_step.state[index].velocity);
            const double position_error = std::sqrt(
                squared_norm(position_delta));
            const double velocity_error = std::sqrt(
                squared_norm(velocity_delta));
            const double density_error = std::abs(
                gpu_snapshot.density[index] - cpu_step.density[index])
                / profile.rest_density;
            position_errors.push_back(position_error);
            position_squared += static_cast<long double>(position_error)
                * position_error;
            if (position_error > maximum_position) {
                maximum_position = position_error;
                maximum_index = index;
            }
            maximum_velocity = std::max(maximum_velocity, velocity_error);
            maximum_density = std::max(maximum_density, density_error);
            const std::uint32_t id = gpu_snapshot.state[index].sample_id;
            const bool gpu_active = std::binary_search(
                gpu_snapshot.active_pressure_ids.begin(),
                gpu_snapshot.active_pressure_ids.end(), id);
            const bool cpu_active = std::binary_search(
                cpu_step.active_pressure_ids.begin(),
                cpu_step.active_pressure_ids.end(), id);
            sample_material << id << ':' << std::hex
                            << float_bits(static_cast<float>(
                                   gpu_snapshot.state[index].current.x)) << ','
                            << float_bits(static_cast<float>(
                                   gpu_snapshot.state[index].current.y)) << ','
                            << float_bits(static_cast<float>(
                                   gpu_snapshot.state[index].current.z)) << ':'
                            << double_bits(cpu_step.state[index].current.x) << ','
                            << double_bits(cpu_step.state[index].current.y) << ','
                            << double_bits(cpu_step.state[index].current.z) << ':'
                            << float_bits(static_cast<float>(
                                   gpu_snapshot.density[index])) << ','
                            << double_bits(cpu_step.density[index]) << ':'
                            << std::dec << gpu_active << ',' << cpu_active
                            << ',' << position_error << ',' << velocity_error
                            << ',' << density_error;
            if (diagnostic) {
                const Vec3d contact =
                    gpu_step.diagnostic_contact_impulse[index];
                sample_material << ','
                    << gpu_step.diagnostic_contact_face_masks[index] << ','
                    << std::hex
                    << float_bits(static_cast<float>(contact.x)) << ','
                    << float_bits(static_cast<float>(contact.y)) << ','
                    << float_bits(static_cast<float>(contact.z)) << ','
                    << std::dec << inset_face_distance(
                           profile, gpu_snapshot.state[index].current) << ','
                    << inferred_inset_face_mask(
                           profile, gpu_snapshot.state[index].current);
            }
            sample_material << '\n';
        }
        if (!apparatus_valid || position_errors.empty()) break;
        std::sort(position_errors.begin(), position_errors.end());
        const auto nearest_rank = [&](double fraction) {
            const std::size_t rank = static_cast<std::size_t>(
                std::ceil(fraction * position_errors.size()));
            return position_errors[std::max<std::size_t>(rank, 1U) - 1U];
        };
        const double slope = maximum_position - prior_maximum;
        if (step >= kRecordFirst && slope_step == 0U && prior_slope > 0.0
            && slope > 2.0 * prior_slope) {
            slope_step = step;
            slope_discriminator = run_ncgp5_discriminator(
                gpu, profile, gpu_snapshot, ghosts);
        }
        prior_maximum = maximum_position;
        prior_slope = slope;
        if (step == kLastStep - 1U) {
            pre_failure_discriminator = run_ncgp5_discriminator(
                gpu, profile, gpu_snapshot, ghosts);
        }
        if (diagnostic) {
            Ncgp5StepRecord record;
            record.step = step;
            record.maximum_id = gpu_snapshot.state[maximum_index].sample_id;
            record.p50_position_error = nearest_rank(0.50);
            record.p95_position_error = nearest_rank(0.95);
            record.p99_position_error = nearest_rank(0.99);
            record.maximum_position_error = maximum_position;
            record.position_rmse = static_cast<double>(std::sqrt(
                position_squared / static_cast<long double>(
                    gpu_snapshot.state.size())));
            record.maximum_velocity_error = maximum_velocity;
            record.maximum_density_error = maximum_density;
            record.gpu_position = gpu_snapshot.state[maximum_index].current;
            record.cpu_position = cpu_step.state[maximum_index].current;
            record.position_delta = subtract(
                record.gpu_position, record.cpu_position);
            const std::array<double, 3> components{
                std::abs(record.position_delta.x),
                std::abs(record.position_delta.y),
                std::abs(record.position_delta.z)};
            record.maximum_component = static_cast<std::uint32_t>(
                std::distance(components.begin(), std::max_element(
                    components.begin(), components.end())));
            record.gpu_velocity = gpu_snapshot.state[maximum_index].velocity;
            record.cpu_velocity = cpu_step.state[maximum_index].velocity;
            record.outlier_density_error = std::abs(
                gpu_snapshot.density[maximum_index]
                    - cpu_step.density[maximum_index])
                / profile.rest_density;
            record.gpu_active = std::binary_search(
                gpu_snapshot.active_pressure_ids.begin(),
                gpu_snapshot.active_pressure_ids.end(), record.maximum_id);
            record.cpu_active = std::binary_search(
                cpu_step.active_pressure_ids.begin(),
                cpu_step.active_pressure_ids.end(), record.maximum_id);
            record.outlier_face_distance = inset_face_distance(
                profile, record.gpu_position);
            record.inferred_face_mask = inferred_inset_face_mask(
                profile, record.gpu_position);
            record.contact_face_mask =
                gpu_step.diagnostic_contact_face_masks[maximum_index];
            record.contact_impulse =
                gpu_step.diagnostic_contact_impulse[maximum_index];
            record.current_high = compensated.current_high[maximum_index];
            record.current_low = compensated.current_low[maximum_index];
            record.predicted_high = compensated.predicted_high[maximum_index];
            record.predicted_low = compensated.predicted_low[maximum_index];
            record.gpu_hvp = gpu_step.hvp_used;
            record.cpu_hvp = cpu_step.hvp_used;
            record.gpu_outer = gpu_step.work.outer_trials;
            record.gpu_accepted = gpu_step.work.accepted_trials;
            record.gpu_rejected = gpu_step.work.rejected_trials;
            record.gpu_radius_shrinks = gpu_step.work.radius_shrinks;
            record.contact_projections = gpu_step.work.contact_projections;
            record.boundary_face_mask_xor =
                gpu_step.boundary_face_mask_xor;
            record.sample_records_root = nextengine::nonlocal::sha256_hex(
                sample_material.str());
            std::string mutated_samples = sample_material.str();
            if (!mutated_samples.empty()) mutated_samples.back() ^= 1;
            sample_mutation_rejected = sample_mutation_rejected
                && nextengine::nonlocal::sha256_hex(mutated_samples)
                    != record.sample_records_root;
            const NonlocalGpuGraphResult graph = gpu.build_current_graph(
                NonlocalGpuVariant::CompensatedScaleF32, true, false);
            if (graph.failure != NonlocalGpuFailure::None) {
                apparatus_valid = false;
                break;
            }
            const auto owner = std::lower_bound(graph.owner_ids.begin(),
                graph.owner_ids.end(), record.maximum_id);
            if (owner == graph.owner_ids.end() || *owner != record.maximum_id) {
                apparatus_valid = false;
                break;
            }
            const std::size_t row = static_cast<std::size_t>(
                std::distance(graph.owner_ids.begin(), owner));
            std::ostringstream neighbor_material;
            neighbor_material << "nextengine.nonlocal.ncgp5.outlier-row.v1\n"
                              << step << ':' << record.maximum_id << '\n';
            for (std::uint32_t offset = graph.offsets[row];
                 offset < graph.offsets[row + 1U]; ++offset) {
                const std::uint32_t neighbor = graph.neighbor_ids[offset];
                const bool active = std::binary_search(
                    gpu_snapshot.active_pressure_ids.begin(),
                    gpu_snapshot.active_pressure_ids.end(), neighbor);
                neighbor_material << neighbor << ':' << active << '\n';
                ++record.neighbor_count;
                if (active) ++record.active_neighbor_centers;
            }
            record.neighbor_root = nextengine::nonlocal::sha256_hex(
                neighbor_material.str());
            std::string mutated_neighbors = neighbor_material.str();
            if (!mutated_neighbors.empty()) mutated_neighbors.back() ^= 1;
            neighbor_mutation_rejected = neighbor_mutation_rejected
                && nextengine::nonlocal::sha256_hex(mutated_neighbors)
                    != record.neighbor_root;
            record.graph_work_root = work_semantic_root(graph.work);
            record.step_work_root = step_work_semantic_root(profile, gpu_step);
            record.permuted_step_work_root = step_work_semantic_root(
                profile, permuted_step);
            record.cpu_step_work_root = step_work_semantic_root(
                profile, cpu_step);
            record.trace_root = solver_trace_semantic_root(gpu_step);
            record.state_root = compensated_state_root(compensated);
            records.push_back(record);
        }
        cpu_state = cpu_step.state;
        final_gpu_step = gpu_step;
    }
    NonlocalGpuStepResult work_mutation = final_gpu_step;
    ++work_mutation.hvp_used;
    const bool work_mutation_rejected = !solver_trace_valid(work_mutation);
    const bool witness_reproduced = apparatus_valid && permutation_exact
        && records.size() == kLastStep - kRecordFirst + 1U
        && records.back().step == kLastStep
        && records.back().maximum_position_error > 0.005
        && records.back().position_rmse <= 0.0025;
    const bool discriminators_valid = pre_failure_discriminator.valid
        && (slope_step == 0U || slope_discriminator.valid);
    const bool controls_pass = sample_mutation_rejected
        && neighbor_mutation_rejected && work_mutation_rejected;
    const bool passed = witness_reproduced && discriminators_valid
        && controls_pass;
    const std::string receipt_root = nextengine::nonlocal::sha256_hex(
        all_receipts.str());
    const std::string executable_root = binary_root();
    const std::string environment = gpu.environment_json();
    std::ostringstream result_material;
    result_material << std::setprecision(17)
                    << "nextengine.nonlocal.ncgp5.step92-result.v1\n"
                    << passed << '\n' << input_root << '\n' << receipt_root
                    << '\n' << slope_step << '\n';
    for (const Ncgp5StepRecord& record : records) {
        result_material << record.step << ':' << record.maximum_id << ':'
                        << record.maximum_component << ':'
                        << record.p50_position_error << ':'
                        << record.p95_position_error << ':'
                        << record.p99_position_error << ':'
                        << record.maximum_position_error << ':'
                        << record.position_rmse << ':'
                        << record.maximum_velocity_error << ':'
                        << record.maximum_density_error << ':'
                        << record.outlier_density_error << ':'
                        << record.outlier_face_distance << ':'
                        << record.gpu_active << ':' << record.cpu_active << ':'
                        << record.inferred_face_mask << ':'
                        << record.contact_face_mask << ':'
                        << record.contact_impulse.x << ':'
                        << record.contact_impulse.y << ':'
                        << record.contact_impulse.z << ':'
                        << record.gpu_hvp << ':' << record.cpu_hvp << ':'
                        << record.gpu_outer << ':' << record.gpu_accepted << ':'
                        << record.gpu_rejected << ':'
                        << record.gpu_radius_shrinks << ':'
                        << record.contact_projections << ':'
                        << record.boundary_face_mask_xor << ':'
                        << record.neighbor_count << ':'
                        << record.active_neighbor_centers << ':'
                        << record.sample_records_root << ':'
                        << record.neighbor_root << ':'
                        << record.graph_work_root << ':'
                        << record.step_work_root << ':'
                        << record.permuted_step_work_root << ':'
                        << record.cpu_step_work_root << ':'
                        << record.trace_root << ':' << record.state_root << '\n';
    }
    const auto append_discriminator = [&](const char* label,
                                          const Ncgp5Discriminator& value) {
        result_material << label << ':' << value.valid << ':'
                        << value.active_signature_equal << ':'
                        << value.operator_error.gradient_relative_l2 << ':'
                        << value.operator_error.hvp_relative_l2 << ':'
                        << value.operator_error.hvp_cosine_loss << ':'
                        << value.position_rmse << ':' << value.position_maximum
                        << ':' << value.gpu_hvp << ':' << value.cpu_hvp << ':'
                        << value.gpu_outer << ':' << value.cpu_outer << ':'
                        << value.input_root << ':' << value.gpu_result_root << ':'
                        << value.cpu_result_root << ':' << value.trace_root
                        << '\n';
    };
    append_discriminator("slope", slope_discriminator);
    append_discriminator("pre-failure", pre_failure_discriminator);
    result_material << controls_pass << '\n' << NCGP5_CONTRACT_ROOT << '\n'
                    << NCGP5_SOURCE_ROOT << '\n' << NCGP5_SOURCE_COMMIT << '\n'
                    << NCGP5_SOURCE_TREE << '\n' << NCGP5_COMPILER_FLAGS << '\n'
                    << executable_root << '\n' << environment << '\n';
    const std::string result_root = nextengine::nonlocal::sha256_hex(
        result_material.str());
    const auto emit_vec = [](Vec3d value) {
        std::cout << '[' << value.x << ',' << value.y << ',' << value.z << ']';
    };
    const auto emit_discriminator = [&](const Ncgp5Discriminator& value) {
        std::cout << "{\"valid\":" << (value.valid ? "true" : "false")
                  << ",\"active_signature_equal\":"
                  << (value.active_signature_equal ? "true" : "false")
                  << ",\"gradient_relative_l2\":"
                  << value.operator_error.gradient_relative_l2
                  << ",\"hvp_relative_l2\":"
                  << value.operator_error.hvp_relative_l2
                  << ",\"hvp_cosine_loss\":"
                  << value.operator_error.hvp_cosine_loss
                  << ",\"position_rmse_m\":" << value.position_rmse
                  << ",\"position_max_m\":" << value.position_maximum
                  << ",\"gpu_hvp\":" << value.gpu_hvp
                  << ",\"cpu_hvp\":" << value.cpu_hvp
                  << ",\"gpu_outer\":" << value.gpu_outer
                  << ",\"cpu_outer\":" << value.cpu_outer
                  << ",\"input_root\":\"" << value.input_root
                  << "\",\"gpu_result_root\":\"" << value.gpu_result_root
                  << "\",\"cpu_result_root\":\"" << value.cpu_result_root
                  << "\",\"trace_root\":\"" << value.trace_root << "\"}";
    };
    std::cout << std::setprecision(17)
              << "{\"schema\":\"nextengine.nonlocal.ncgp5.step92.v1\""
              << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
              << ",\"witness_reproduced\":"
              << (witness_reproduced ? "true" : "false")
              << ",\"permutation_exact\":"
              << (permutation_exact ? "true" : "false")
              << ",\"slope_step\":" << slope_step
              << ",\"sample_mutation_rejected\":"
              << (sample_mutation_rejected ? "true" : "false")
              << ",\"neighbor_mutation_rejected\":"
              << (neighbor_mutation_rejected ? "true" : "false")
              << ",\"work_mutation_rejected\":"
              << (work_mutation_rejected ? "true" : "false")
              << ",\"records\":[";
    for (std::size_t index = 0U; index < records.size(); ++index) {
        const Ncgp5StepRecord& record = records[index];
        if (index != 0U) std::cout << ',';
        std::cout << "{\"step\":" << record.step
                  << ",\"maximum_id\":" << record.maximum_id
                  << ",\"maximum_component\":" << record.maximum_component
                  << ",\"p50_m\":" << record.p50_position_error
                  << ",\"p95_m\":" << record.p95_position_error
                  << ",\"p99_m\":" << record.p99_position_error
                  << ",\"max_m\":" << record.maximum_position_error
                  << ",\"rmse_m\":" << record.position_rmse
                  << ",\"velocity_max_m_s\":"
                  << record.maximum_velocity_error
                  << ",\"density_max_fraction\":"
                  << record.maximum_density_error
                  << ",\"outlier_density_fraction\":"
                  << record.outlier_density_error
                  << ",\"outlier_face_distance_m\":"
                  << record.outlier_face_distance
                  << ",\"gpu_active\":"
                  << (record.gpu_active ? "true" : "false")
                  << ",\"cpu_active\":"
                  << (record.cpu_active ? "true" : "false")
                  << ",\"inferred_face_mask\":"
                  << record.inferred_face_mask
                  << ",\"contact_face_mask\":"
                  << record.contact_face_mask
                  << ",\"contact_impulse\":";
        emit_vec(record.contact_impulse);
        std::cout
                  << ",\"gpu_position\":";
        emit_vec(record.gpu_position);
        std::cout << ",\"cpu_position\":";
        emit_vec(record.cpu_position);
        std::cout << ",\"position_delta\":";
        emit_vec(record.position_delta);
        std::cout << ",\"gpu_velocity\":";
        emit_vec(record.gpu_velocity);
        std::cout << ",\"cpu_velocity\":";
        emit_vec(record.cpu_velocity);
        std::cout << ",\"current_high\":";
        emit_vec(record.current_high);
        std::cout << ",\"current_low\":";
        emit_vec(record.current_low);
        std::cout << ",\"predicted_high\":";
        emit_vec(record.predicted_high);
        std::cout << ",\"predicted_low\":";
        emit_vec(record.predicted_low);
        std::cout << ",\"gpu_hvp\":" << record.gpu_hvp
                  << ",\"cpu_hvp\":" << record.cpu_hvp
                  << ",\"gpu_outer\":" << record.gpu_outer
                  << ",\"gpu_accepted\":" << record.gpu_accepted
                  << ",\"gpu_rejected\":" << record.gpu_rejected
                  << ",\"gpu_radius_shrinks\":"
                  << record.gpu_radius_shrinks
                  << ",\"contact_projections\":"
                  << record.contact_projections
                  << ",\"boundary_face_mask_xor\":"
                  << record.boundary_face_mask_xor
                  << ",\"neighbor_count\":" << record.neighbor_count
                  << ",\"active_neighbor_centers\":"
                  << record.active_neighbor_centers
                  << ",\"sample_records_root\":\""
                  << record.sample_records_root
                  << "\",\"neighbor_root\":\"" << record.neighbor_root
                  << "\",\"graph_work_root\":\""
                  << record.graph_work_root
                  << "\",\"step_work_root\":\"" << record.step_work_root
                  << "\",\"trace_root\":\"" << record.trace_root
                  << "\",\"state_root\":\"" << record.state_root << "\"}";
    }
    std::cout << "],\"slope_discriminator\":";
    emit_discriminator(slope_discriminator);
    std::cout << ",\"pre_failure_discriminator\":";
    emit_discriminator(pre_failure_discriminator);
    std::cout << ",\"cpu_final_state_root\":\""
              << cpu_state_semantic_root(cpu_state)
              << "\",\"input_root\":\"" << input_root
              << "\",\"receipt_root\":\"" << receipt_root
              << "\",\"result_root\":\"" << result_root
              << "\",\"contract_root\":\"" << NCGP5_CONTRACT_ROOT
              << "\",\"source_root\":\"" << NCGP5_SOURCE_ROOT
              << "\",\"source_commit\":\"" << NCGP5_SOURCE_COMMIT
              << "\",\"source_tree\":\"" << NCGP5_SOURCE_TREE
              << "\",\"compiler_flags\":\"" << NCGP5_COMPILER_FLAGS
              << "\",\"binary_root\":\"" << executable_root
              << "\",\"environment\":" << environment
              << ",\"exact_command\":\"--diagnose-hydro-step92-outlier\"}\n";
    return passed ? 0 : 54;
}
#endif

} // namespace

int main(int argc, char** argv) {
    if (argc == 2 && std::string(argv[1]) == "--profile-self-test") {
        return run_profile_self_test();
    }
    if (argc == 2 && std::string(argv[1]) == "--graph-self-test") {
        return run_graph_self_test();
    }
    if (argc == 2 && std::string(argv[1]) == "--boundary-self-test") {
        return run_boundary_self_test();
    }
    if (argc == 2 && std::string(argv[1]) == "--transaction-self-test") {
        return run_transaction_self_test();
    }
    if (argc == 2 && std::string(argv[1]) == "--physics-self-test") {
        return run_ncgp3_physics_self_test();
    }
    if (argc == 5 && std::string(argv[1]) == "--correspondence-4k") {
        return run_correspondence_4k(argv[2],
            static_cast<std::uint32_t>(std::stoul(argv[3])),
            static_cast<std::uint32_t>(std::stoul(argv[4])),
            NonlocalGpuVariant::CompensatedScaleF32,
            "f32-primary");
    }
    if (argc == 5
        && std::string(argv[1]) == "--correspondence-4k-pressure-f64") {
        return run_correspondence_4k(argv[2],
            static_cast<std::uint32_t>(std::stoul(argv[3])),
            static_cast<std::uint32_t>(std::stoul(argv[4])),
            NonlocalGpuVariant::CompensatedScalePressureF64,
            "f32-state-f64-pressure");
    }
#if defined(NCGP4_EXPERIMENTAL)
    if (argc == 2 && std::string(argv[1]) == "--diagnose-hydro-step39") {
        return run_ncgp4_solver_diagnosis();
    }
    if (argc == 5
        && std::string(argv[1]) == "--correspondence-4k-unpreconditioned") {
        return run_correspondence_4k(argv[2],
            static_cast<std::uint32_t>(std::stoul(argv[3])),
            static_cast<std::uint32_t>(std::stoul(argv[4])),
            NonlocalGpuVariant::CompensatedScaleF32,
            "f32-primary", NonlocalGpuSolverProfile::Unpreconditioned);
    }
#endif
#if defined(NCGP5_EXPERIMENTAL)
    if (argc == 2
        && std::string(argv[1]) == "--diagnose-hydro-step92-outlier") {
        return run_ncgp5_step92_diagnosis();
    }
#endif
#if defined(NCGP6_EXPERIMENTAL)
    if (argc == 2 && std::string(argv[1]) == "--product-gate-self-test") {
        return run_ncgp6_product_gate_self_test();
    }
    if (argc == 5
        && std::string(argv[1]) == "--correspondence-4k-product") {
        return run_correspondence_4k(argv[2],
            static_cast<std::uint32_t>(std::stoul(argv[3])),
            static_cast<std::uint32_t>(std::stoul(argv[4])),
            NonlocalGpuVariant::CompensatedScaleF32,
            "f32-primary", NonlocalGpuSolverProfile::Unpreconditioned, true);
    }
#endif
    std::cerr << "usage: nonlocal-corrected-cuda-compensated-scale "
                 "--profile-self-test|--graph-self-test|--boundary-self-test|"
                 "--transaction-self-test|--physics-self-test|"
                 "--diagnose-hydro-step39|"
                 "--diagnose-hydro-step92-outlier|"
                 "--product-gate-self-test|"
                 "--correspondence-4k SCENARIO STEPS BUDGET|"
                 "--correspondence-4k-unpreconditioned SCENARIO STEPS BUDGET|"
                 "--correspondence-4k-product SCENARIO 240 128|"
                 "--correspondence-4k-pressure-f64 SCENARIO STEPS BUDGET\n";
    return 2;
}
