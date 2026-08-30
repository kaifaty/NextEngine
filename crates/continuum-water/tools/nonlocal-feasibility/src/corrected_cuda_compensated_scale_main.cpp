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
#include <sstream>
#include <stdexcept>
#include <string>
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

long double cubic_weight(long double radius, long double horizon) {
    const long double pi = acosl(-1.0L);
    const long double q = 2.0L * radius / horizon;
    const long double alpha = 3.0L
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
        * (cubic_weight(0.0L, profile.horizon)
            + cubic_weight(0.05L, profile.horizon))
        / 1.1L);
    return profile;
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

int run_correspondence_4k(const std::string& scenario,
    std::uint32_t steps,
    std::uint32_t budget) {
    if (steps == 0U || steps > 240U
        || (budget != 32U && budget != 64U && budget != 128U)) return 2;
    const NonlocalGpuProfile profile = nonlocal_water_profile();
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
    double maximum_position_rmse = 0.0;
    double maximum_position_error = 0.0;
    double maximum_density_rmse = 0.0;
    double maximum_density_error = 0.0;
    double maximum_rest_density_rmse = 0.0;
    double maximum_rest_density_error = 0.0;
    double observed_density_minimum = std::numeric_limits<double>::infinity();
    double observed_density_maximum = 0.0;
    double observed_density_mean = 0.0;
    std::uint32_t maximum_gpu_hvp = 0U;
    std::uint32_t maximum_cpu_hvp = 0U;
    std::uint64_t active_mismatch_steps = 0U;
    std::uint64_t permutation_mismatch_steps = 0U;
    std::string receipt_material =
        "nextengine.nonlocal.ncgp3.trajectory-receipts.v1\n";
    NonlocalGpuFailure gpu_failure = NonlocalGpuFailure::None;
    NonlocalGpuFailure cpu_failure = NonlocalGpuFailure::None;
    std::uint32_t completed = 0U;
    for (std::uint32_t step_index = 0U; step_index < steps; ++step_index) {
        const auto gpu_result = gpu.step(budget,
            NonlocalGpuSolverProfile::Jacobi,
            NonlocalGpuVariant::CompensatedScaleF32, true, false);
        const auto permuted_result = permuted.step(budget,
            NonlocalGpuSolverProfile::Jacobi,
            NonlocalGpuVariant::CompensatedScaleF32, true, false);
        const auto cpu_result = step_reference(profile, cpu_state, ghosts,
            128U, NonlocalGpuVariant::Corrected, true);
        gpu_failure = gpu_result.failure != NonlocalGpuFailure::None
            ? gpu_result.failure : permuted_result.failure;
        cpu_failure = cpu_result.failure;
        receipt_material += step_work_semantic_root(profile, gpu_result) + ':'
            + step_work_semantic_root(profile, permuted_result) + ':'
            + step_work_semantic_root(profile, cpu_result) + '\n';
        if (gpu_failure != NonlocalGpuFailure::None
            || cpu_failure != NonlocalGpuFailure::None
            || gpu_result.state.size() != cpu_result.state.size()
            || gpu_result.state.size() != permuted_result.state.size()
            || gpu_result.density.size() != cpu_result.density.size()) break;
        double position_squared = 0.0;
        double density_squared = 0.0;
        double rest_density_squared = 0.0;
        double density_sum = 0.0;
        for (std::size_t index = 0U; index < gpu_result.state.size(); ++index) {
            if (gpu_result.state[index].sample_id
                    != cpu_result.state[index].sample_id
                || gpu_result.state[index].sample_id
                    != permuted_result.state[index].sample_id) {
                ++permutation_mismatch_steps;
                break;
            }
            const Vec3d delta{
                gpu_result.state[index].current.x
                    - cpu_result.state[index].current.x,
                gpu_result.state[index].current.y
                    - cpu_result.state[index].current.y,
                gpu_result.state[index].current.z
                    - cpu_result.state[index].current.z};
            const double distance = std::sqrt(delta.x * delta.x
                + delta.y * delta.y + delta.z * delta.z);
            position_squared += distance * distance;
            maximum_position_error = std::max(
                maximum_position_error, distance);
            const double density_error = std::abs(
                gpu_result.density[index] - cpu_result.density[index])
                / profile.rest_density;
            density_squared += density_error * density_error;
            maximum_density_error = std::max(
                maximum_density_error, density_error);
            const double rest_density_error = std::abs(
                gpu_result.density[index] - profile.rest_density)
                / profile.rest_density;
            observed_density_minimum = std::min(observed_density_minimum,
                gpu_result.density[index]);
            observed_density_maximum = std::max(observed_density_maximum,
                gpu_result.density[index]);
            density_sum += gpu_result.density[index];
            rest_density_squared += rest_density_error * rest_density_error;
            maximum_rest_density_error = std::max(
                maximum_rest_density_error, rest_density_error);
            const Vec3d permutation_delta{
                gpu_result.state[index].current.x
                    - permuted_result.state[index].current.x,
                gpu_result.state[index].current.y
                    - permuted_result.state[index].current.y,
                gpu_result.state[index].current.z
                    - permuted_result.state[index].current.z};
            if (permutation_delta.x != 0.0 || permutation_delta.y != 0.0
                || permutation_delta.z != 0.0) {
                ++permutation_mismatch_steps;
                break;
            }
        }
        maximum_position_rmse = std::max(maximum_position_rmse,
            std::sqrt(position_squared
                / static_cast<double>(gpu_result.state.size())));
        maximum_density_rmse = std::max(maximum_density_rmse,
            std::sqrt(density_squared
                / static_cast<double>(gpu_result.state.size())));
        maximum_rest_density_rmse = std::max(maximum_rest_density_rmse,
            std::sqrt(rest_density_squared
                / static_cast<double>(gpu_result.state.size())));
        observed_density_mean = density_sum
            / static_cast<double>(gpu_result.state.size());
        if (gpu_result.active_pressure_ids != cpu_result.active_pressure_ids) {
            ++active_mismatch_steps;
        }
        if (gpu_result.active_pressure_ids
            != permuted_result.active_pressure_ids) {
            ++permutation_mismatch_steps;
        }
        maximum_gpu_hvp = std::max(maximum_gpu_hvp, gpu_result.hvp_used);
        maximum_cpu_hvp = std::max(maximum_cpu_hvp, cpu_result.hvp_used);
        cpu_state = cpu_result.state;
        ++completed;
        if (maximum_position_rmse > 0.0025
            || maximum_position_error > 0.005
            || maximum_density_rmse > 0.05
            || maximum_density_error > 0.10
            || maximum_rest_density_rmse > 0.05
            || maximum_rest_density_error > 0.10
            || active_mismatch_steps != 0U
            || permutation_mismatch_steps != 0U) break;
    }
    NonlocalGpuGraphResult failure_graph;
    if (gpu_failure == NonlocalGpuFailure::CapacityExceeded
        || cpu_failure == NonlocalGpuFailure::CapacityExceeded) {
        failure_graph = build_reference_graph(
            profile, cpu_state, ghosts, false);
    }
    Vec3d minimum{std::numeric_limits<double>::infinity(),
        std::numeric_limits<double>::infinity(),
        std::numeric_limits<double>::infinity()};
    Vec3d maximum{-std::numeric_limits<double>::infinity(),
        -std::numeric_limits<double>::infinity(),
        -std::numeric_limits<double>::infinity()};
    for (const auto& sample : cpu_state) {
        minimum.x = std::min(minimum.x, sample.current.x);
        minimum.y = std::min(minimum.y, sample.current.y);
        minimum.z = std::min(minimum.z, sample.current.z);
        maximum.x = std::max(maximum.x, sample.current.x);
        maximum.y = std::max(maximum.y, sample.current.y);
        maximum.z = std::max(maximum.z, sample.current.z);
    }
    const bool passed = completed == steps
        && gpu_failure == NonlocalGpuFailure::None
        && cpu_failure == NonlocalGpuFailure::None
        && maximum_position_rmse <= 0.0025
        && maximum_position_error <= 0.005
        && maximum_density_rmse <= 0.05
        && maximum_density_error <= 0.10
        && maximum_rest_density_rmse <= 0.05
        && maximum_rest_density_error <= 0.10
        && active_mismatch_steps == 0U
        && permutation_mismatch_steps == 0U;
    const bool physical_refuted = !passed && completed > 0U
        && gpu_failure == NonlocalGpuFailure::None
        && cpu_failure == NonlocalGpuFailure::None
        && (maximum_rest_density_rmse > 0.05
            || maximum_rest_density_error > 0.10);
    long double infinite_lattice_density = 0.0L;
    for (int z = -3; z <= 3; ++z) {
        for (int y = -3; y <= 3; ++y) {
            for (int x = -3; x <= 3; ++x) {
                const long double radius = static_cast<long double>(
                    profile.spacing)
                    * std::sqrt(static_cast<long double>(x * x + y * y + z * z));
                if (radius <= static_cast<long double>(profile.horizon)) {
                    infinite_lattice_density += static_cast<long double>(
                        profile.mass) * cubic_weight(
                            radius, static_cast<long double>(profile.horizon));
                }
            }
        }
    }
    const char* status = passed ? "PASS"
        : (physical_refuted ? "PHYSICS_REFUTED" : "INCONCLUSIVE");
    std::cout << std::setprecision(17)
              << "{\"schema\":\"nextengine.nonlocal.ncgp3.correspondence.v1\""
              << ",\"status\":\"" << status << "\""
              << ",\"scenario\":\"" << scenario << "\",\"steps\":" << steps
              << ",\"completed_steps\":" << completed
              << ",\"budget\":" << budget
              << ",\"gpu_failure\":" << static_cast<std::uint32_t>(gpu_failure)
              << ",\"cpu_failure\":" << static_cast<std::uint32_t>(cpu_failure)
              << ",\"position_rmse_max_m\":" << maximum_position_rmse
              << ",\"position_error_max_m\":" << maximum_position_error
              << ",\"density_correspondence_rmse_max_fraction\":"
              << maximum_density_rmse
              << ",\"density_correspondence_error_max_fraction\":"
              << maximum_density_error
              << ",\"density_rest_rmse_max_fraction\":"
              << maximum_rest_density_rmse
              << ",\"density_rest_error_max_fraction\":"
              << maximum_rest_density_error
              << ",\"density_observed_min_kg_m3\":"
              << observed_density_minimum
              << ",\"density_observed_max_kg_m3\":"
              << observed_density_maximum
              << ",\"density_observed_mean_kg_m3\":"
              << observed_density_mean
              << ",\"independent_infinite_lattice_density_kg_m3\":"
              << static_cast<double>(infinite_lattice_density)
              << ",\"independent_lattice_density_fraction\":"
              << static_cast<double>(infinite_lattice_density
                    / static_cast<long double>(profile.rest_density))
              << ",\"active_mismatch_steps\":" << active_mismatch_steps
              << ",\"permutation_mismatch_steps\":"
              << permutation_mismatch_steps
              << ",\"gpu_hvp_max\":" << maximum_gpu_hvp
              << ",\"cpu_hvp_max\":" << maximum_cpu_hvp
              << ",\"particle_count\":" << cpu_state.size()
              << ",\"mass_kg\":"
              << static_cast<double>(cpu_state.size()) * profile.mass
              << ",\"input_root\":\"" << input_root
              << "\",\"receipt_root\":\""
              << nextengine::nonlocal::sha256_hex(receipt_material)
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
              << ",\"state_bounds_min\":[" << minimum.x << ',' << minimum.y
              << ',' << minimum.z << "]"
              << ",\"state_bounds_max\":[" << maximum.x << ',' << maximum.y
              << ',' << maximum.z << ']'
              << ",\"contract_root\":\"" << NCGP3_CONTRACT_ROOT
              << "\",\"source_root\":\"" << NCGP3_SOURCE_ROOT
              << "\",\"binary_root\":\"" << binary_root()
              << "\"}\n";
    return passed ? 0 : (physical_refuted ? 37 : 53);
}

} // namespace

int main(int argc, char** argv) {
    if (argc == 2 && std::string(argv[1]) == "--graph-self-test") {
        return run_graph_self_test();
    }
    if (argc == 2 && std::string(argv[1]) == "--boundary-self-test") {
        return run_boundary_self_test();
    }
    if (argc == 2 && std::string(argv[1]) == "--transaction-self-test") {
        return run_transaction_self_test();
    }
    if (argc == 5 && std::string(argv[1]) == "--correspondence-4k") {
        return run_correspondence_4k(argv[2],
            static_cast<std::uint32_t>(std::stoul(argv[3])),
            static_cast<std::uint32_t>(std::stoul(argv[4])));
    }
    std::cerr << "usage: nonlocal-corrected-cuda-compensated-scale "
                 "--graph-self-test|--boundary-self-test|"
                 "--transaction-self-test|"
                 "--correspondence-4k SCENARIO STEPS BUDGET\n";
    return 2;
}
