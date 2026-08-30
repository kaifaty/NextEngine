#include "corrected_cuda_full_step.hpp"

#include <algorithm>
#include <cstdint>
#include <iomanip>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>

namespace {

using namespace nextengine::nonlocal::gpu_full_step;

bool same_graph(const NonlocalGpuGraphResult& lhs,
    const NonlocalGpuGraphResult& rhs) {
    return lhs.failure == rhs.failure && lhs.owner_ids == rhs.owner_ids
        && lhs.offsets == rhs.offsets && lhs.neighbor_ids == rhs.neighbor_ids;
}

std::vector<NonlocalGpuSample> support_edge_fixture() {
    return {
        {90U, {0.0, 0.0, 0.0}, {0.0, 0.0, 0.0}, {}},
        {4U, {0.15, 0.0, 0.0}, {0.15, 0.0, 0.0}, {}},
        {55U, {0.150001, 0.0, 0.0}, {0.150001, 0.0, 0.0}, {}},
        {801U, {0.0, 0.150001, 0.0}, {0.0, 0.150001, 0.0}, {}},
    };
}

int run_graph_self_test() {
    const NonlocalGpuProfile profile = nonlocal_water_profile();
    const std::vector<NonlocalGpuGhost> no_ghosts;
    const auto edge = support_edge_fixture();
    const auto edge_reference = build_reference_graph(profile, edge, no_ghosts, false);
    NonlocalGpuWorkspace edge_workspace(profile);
    if (edge_workspace.upload(edge, no_ghosts) != NonlocalGpuFailure::None) return 2;
    const auto edge_gpu = edge_workspace.build_current_graph(
        NonlocalGpuVariant::Corrected, true, false);
    const auto edge_strict = edge_workspace.build_current_graph(
        NonlocalGpuVariant::StrictRadius, true, false);
    if (!same_graph(edge_reference, edge_gpu)
        || same_graph(edge_reference, edge_strict)) return 3;

    const auto lattice = make_lattice_state(profile, 4U, 4U, 4U, false, false);
    const auto permuted = make_lattice_state(profile, 4U, 4U, 4U, true, false);
    NonlocalGpuWorkspace canonical_workspace(profile);
    NonlocalGpuWorkspace permuted_workspace(profile);
    if (canonical_workspace.upload(lattice, no_ghosts) != NonlocalGpuFailure::None
        || permuted_workspace.upload(permuted, no_ghosts)
            != NonlocalGpuFailure::None) return 4;
    const auto canonical_graph = canonical_workspace.build_current_graph(
        NonlocalGpuVariant::Corrected, true, false);
    const auto permuted_graph = permuted_workspace.build_current_graph(
        NonlocalGpuVariant::Corrected, true, false);
    const auto reference_graph = build_reference_graph(
        profile, lattice, no_ghosts, false);
    if (!same_graph(reference_graph, canonical_graph)
        || !same_graph(canonical_graph, permuted_graph)) return 5;

    std::vector<NonlocalGpuSample> overflow;
    overflow.reserve(kMaximumNeighbors + 1U);
    for (std::uint32_t index = 0U; index <= kMaximumNeighbors; ++index) {
        overflow.push_back({100000U + index, {}, {}, {}});
    }
    NonlocalGpuWorkspace overflow_workspace(profile);
    if (overflow_workspace.upload(overflow, no_ghosts)
            != NonlocalGpuFailure::None
        || overflow_workspace.build_current_graph(
               NonlocalGpuVariant::Corrected, false, false)
               .failure
            != NonlocalGpuFailure::CapacityExceeded) return 6;

    const auto samples16 = make_lattice_state(profile, 40U, 20U, 20U, false, false);
    const auto samples50 = make_lattice_state(profile, 50U, 40U, 25U, false, false);
    const auto samples50_permuted = make_lattice_state(
        profile, 50U, 40U, 25U, true, false);
    const auto ghosts = make_basin_ghosts(profile);
    NonlocalGpuWorkspace workspace(profile);
    if (workspace.upload(samples16, ghosts) != NonlocalGpuFailure::None) return 7;
    const auto graph16 = workspace.build_current_graph(
        NonlocalGpuVariant::Corrected, false, false);
    if (graph16.failure != NonlocalGpuFailure::None
        || graph16.maximum_degree > kMaximumNeighbors) return 8;
    if (workspace.upload(samples50, ghosts) != NonlocalGpuFailure::None) return 9;
    const auto graph50 = workspace.build_current_graph(
        NonlocalGpuVariant::Corrected, true, false);
    if (graph50.failure != NonlocalGpuFailure::None
        || graph50.maximum_degree > kMaximumNeighbors) return 10;
    NonlocalGpuWorkspace permuted50_workspace(profile);
    if (permuted50_workspace.upload(samples50_permuted, ghosts)
            != NonlocalGpuFailure::None) return 11;
    const auto graph50_permuted = permuted50_workspace.build_current_graph(
        NonlocalGpuVariant::Corrected, true, false);
    if (!same_graph(graph50, graph50_permuted)) return 12;

    std::cout << std::setprecision(9)
              << "{\"research_id\":\"NCGP1\",\"mode\":\"graph-self-test\""
              << ",\"status\":\"PASS\",\"edge_root\":\""
              << graph_semantic_root(edge_gpu) << "\",\"lattice_root\":\""
              << graph_semantic_root(canonical_graph)
              << "\",\"graph16\":{\"pairs\":" << graph16.directed_pairs
              << ",\"max_degree\":" << graph16.maximum_degree
              << "},\"graph50\":{\"pairs\":" << graph50.directed_pairs
              << ",\"max_degree\":" << graph50.maximum_degree
              << ",\"root\":\"" << graph_semantic_root(graph50)
              << "\"},\"ghosts\":" << ghosts.size()
              << ",\"device_bytes\":" << workspace.allocated_device_bytes()
              << ",\"environment\":" << workspace.environment_json() << "}\n";
    return 0;
}

} // namespace

int main(int argc, char** argv) {
    try {
        if (argc == 2 && std::string(argv[1]) == "--graph-self-test") {
            return run_graph_self_test();
        }
        std::cerr << "usage: nonlocal-corrected-cuda-full-step --graph-self-test\n";
        return 64;
    } catch (const std::exception& error) {
        std::cerr << "NCGP1 failure: " << error.what() << '\n';
        return 1;
    }
}
