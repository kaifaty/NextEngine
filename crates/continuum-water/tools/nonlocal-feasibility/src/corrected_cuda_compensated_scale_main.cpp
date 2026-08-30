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
    std::cerr << "usage: nonlocal-corrected-cuda-compensated-scale "
                 "--graph-self-test|--boundary-self-test|"
                 "--transaction-self-test\n";
    return 2;
}
