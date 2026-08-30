#include "corrected_cuda_full_step.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <cstdint>
#include <cstring>
#include <fstream>
#include <iomanip>
#include <iostream>
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

std::string file_root(const std::string& path) {
    std::ifstream stream(path, std::ios::binary);
    if (!stream) return {};
    std::ostringstream bytes;
    bytes << stream.rdbuf();
    return nextengine::nonlocal::sha256_hex(bytes.str());
}

std::string binary_root() { return file_root("/proc/self/exe"); }

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

} // namespace

int main(int argc, char** argv) {
    if (argc == 2 && std::string(argv[1]) == "--graph-self-test") {
        return run_graph_self_test();
    }
    std::cerr << "usage: nonlocal-corrected-cuda-compensated-scale "
                 "--graph-self-test\n";
    return 2;
}
