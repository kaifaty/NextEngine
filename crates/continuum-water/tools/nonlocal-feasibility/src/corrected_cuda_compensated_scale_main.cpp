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

std::string public_state_root(
    const std::vector<NonlocalGpuSample>& unordered_state) {
    std::vector<NonlocalGpuSample> state = unordered_state;
    std::sort(state.begin(), state.end(), [](const auto& lhs, const auto& rhs) {
        return lhs.sample_id < rhs.sample_id;
    });
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp3.public-state.v1\n";
    for (const NonlocalGpuSample& sample : state) {
        material << sample.sample_id << ':' << std::hex
                 << float_bits(static_cast<float>(sample.current.x)) << ','
                 << float_bits(static_cast<float>(sample.current.y)) << ','
                 << float_bits(static_cast<float>(sample.current.z)) << ';'
                 << float_bits(static_cast<float>(sample.velocity.x)) << ','
                 << float_bits(static_cast<float>(sample.velocity.y)) << ','
                 << float_bits(static_cast<float>(sample.velocity.z)) << '\n'
                 << std::dec;
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

int run_correspondence_4k(const std::string& scenario,
    std::uint32_t steps,
    std::uint32_t budget,
    NonlocalGpuVariant gpu_variant,
    const char* arithmetic_profile) {
    if (steps == 0U || steps > 240U
        || (budget != 32U && budget != 64U && budget != 128U)) return 2;
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
    std::string prior_gpu_state_root = public_state_root(cpu_state);
    std::string prior_permuted_state_root = public_state_root(permuted_input);
    std::string snapshot_receipt_material =
        "nextengine.nonlocal.ncgp3.snapshot-receipts.v1\n";
    std::uint32_t completed = 0U;
    for (std::uint32_t step_index = 0U; step_index < steps; ++step_index) {
        const auto gpu_result = gpu.step(budget,
            NonlocalGpuSolverProfile::Jacobi,
            gpu_variant, false, false);
        const auto permuted_result = permuted.step(budget,
            NonlocalGpuSolverProfile::Jacobi,
            gpu_variant, false, false);
        const auto gpu_snapshot = gpu.capture_public_snapshot();
        const auto permuted_snapshot = permuted.capture_public_snapshot();
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
            + ':' + work_semantic_root(permuted_snapshot.work) + '\n';
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
            failure_restored_state_root = public_state_root(gpu_snapshot.state);
            failure_permuted_restored_state_root = public_state_root(
                permuted_snapshot.state);
        }
        if (gpu_failure != NonlocalGpuFailure::None
            || cpu_failure != NonlocalGpuFailure::None
            || gpu_snapshot.failure != NonlocalGpuFailure::None
            || permuted_snapshot.failure != NonlocalGpuFailure::None
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
            maximum_position_error = std::max(
                maximum_position_error, distance);
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
        if (gpu_snapshot.active_pressure_ids
            != permuted_snapshot.active_pressure_ids) {
            ++permutation_mismatch_steps;
        }
        prior_gpu_state_root = public_state_root(gpu_snapshot.state);
        prior_permuted_state_root = public_state_root(permuted_snapshot.state);
        cpu_state = cpu_result.state;
        ++completed;
        if (maximum_position_rmse > 0.0025
            || maximum_position_error > 0.005
            || maximum_density_rmse > 0.05
            || maximum_density_error > 0.10
            || maximum_compression_rmse > 0.05
            || maximum_compression_error > 0.10
            || maximum_momentum_residual > 0.01
            || maximum_positive_energy_excess > 0.01
            || maximum_penetration > 0.0025
            || permutation_mismatch_steps != 0U) break;
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
    const bool passed = completed == steps
        && gpu_failure == NonlocalGpuFailure::None
        && cpu_failure == NonlocalGpuFailure::None
        && maximum_position_rmse <= 0.0025
        && maximum_position_error <= 0.005
        && maximum_density_rmse <= 0.05
        && maximum_density_error <= 0.10
        && maximum_compression_rmse <= 0.05
        && maximum_compression_error <= 0.10
        && maximum_momentum_residual <= 0.01
        && maximum_positive_energy_excess <= 0.01
        && maximum_penetration <= 0.0025
        && closed_basin_bounds
        && permutation_mismatch_steps == 0U;
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
        && (maximum_compression_rmse > 0.05
            || maximum_compression_error > 0.10
            || maximum_momentum_residual > 0.01
            || maximum_positive_energy_excess > 0.01
            || maximum_penetration > 0.0025
            || !closed_basin_bounds));
    const long double lattice_density = infinite_lattice_density(profile);
    const char* status = passed ? "PASS"
        : (physical_refuted ? "PHYSICS_REFUTED" : "INCONCLUSIVE");
    const std::string receipt_root = nextengine::nonlocal::sha256_hex(
        receipt_material);
    const std::string snapshot_receipt_root =
        nextengine::nonlocal::sha256_hex(snapshot_receipt_material);
    const std::string executable_root = binary_root();
    const std::string environment = gpu.environment_json();
    std::ostringstream result_material;
    result_material << std::setprecision(17)
                    << "nextengine.nonlocal.ncgp3.correspondence-result.v3\n"
                    << status << '\n' << scenario << '\n' << steps << '\n'
                    << budget << '\n' << input_root << '\n' << receipt_root
                    << '\n' << snapshot_receipt_root << '\n'
                    << prior_gpu_state_root << '\n'
                    << prior_permuted_state_root << '\n'
                    << failure_restored_state_root << '\n'
                    << maximum_position_rmse << '\n'
                    << maximum_position_error << '\n'
                    << maximum_density_rmse << '\n'
                    << maximum_density_error << '\n'
                    << maximum_momentum_residual << '\n'
                    << maximum_positive_energy_excess << '\n'
                    << maximum_penetration << '\n'
                    << NCGP3_CONTRACT_ROOT << '\n' << NCGP3_SOURCE_ROOT << '\n'
                    << NCGP3_SOURCE_COMMIT << '\n' << NCGP3_SOURCE_TREE << '\n'
                    << NCGP3_COMPILER_FLAGS << '\n' << executable_root << '\n'
                    << environment << '\n';
    const std::string result_root = nextengine::nonlocal::sha256_hex(
        result_material.str());
    std::cout << std::setprecision(17)
              << "{\"schema\":\"nextengine.nonlocal.ncgp3.correspondence.v3\""
              << ",\"status\":\"" << status << "\""
              << ",\"scenario\":\"" << scenario << "\",\"steps\":" << steps
              << ",\"arithmetic_profile\":\"" << arithmetic_profile << "\""
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
              << ",\"contract_root\":\"" << NCGP3_CONTRACT_ROOT
              << "\",\"source_root\":\"" << NCGP3_SOURCE_ROOT
              << "\",\"source_commit\":\"" << NCGP3_SOURCE_COMMIT
              << "\",\"source_tree\":\"" << NCGP3_SOURCE_TREE
              << "\",\"compiler_flags\":\"" << NCGP3_COMPILER_FLAGS
              << "\",\"environment\":" << environment
              << ",\"exact_command\":\"--correspondence-4k "
              << scenario << ' ' << steps << ' ' << budget << "\""
              << ",\"timing_status\":\"NOT_RUN\""
              << ",\"binary_root\":\"" << executable_root
              << "\"}\n";
    return passed ? 0 : (physical_refuted ? 37 : 53);
}

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
    std::cerr << "usage: nonlocal-corrected-cuda-compensated-scale "
                 "--profile-self-test|--graph-self-test|--boundary-self-test|"
                 "--transaction-self-test|--physics-self-test|"
                 "--correspondence-4k SCENARIO STEPS BUDGET|"
                 "--correspondence-4k-pressure-f64 SCENARIO STEPS BUDGET\n";
    return 2;
}
