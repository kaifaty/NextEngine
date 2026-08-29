#include "corrected_cuda_neighbors.hpp"

#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cstdint>
#include <iostream>
#include <limits>
#include <sstream>
#include <stdexcept>
#include <string>
#include <unordered_map>
#include <utility>
#include <vector>

namespace nextengine::nonlocal::gpu_neighbor_audit {
namespace {

constexpr int COLD_REPEATS = 10;

struct NamedFixture {
    const char* name;
    std::vector<CanonicalSample> samples;
};

void append_u32(std::string& bytes, std::uint32_t value) {
    for (unsigned int shift = 0; shift < 32U; shift += 8U) {
        bytes.push_back(static_cast<char>((value >> shift) & 0xffU));
    }
}

void append_u64(std::string& bytes, std::uint64_t value) {
    for (unsigned int shift = 0; shift < 64U; shift += 8U) {
        bytes.push_back(static_cast<char>((value >> shift) & 0xffU));
    }
}

void append_i64(std::string& bytes, std::int64_t value) {
    append_u64(bytes, static_cast<std::uint64_t>(value));
}

std::vector<NamedFixture> make_fixtures() {
    std::vector<NamedFixture> fixtures;
    fixtures.push_back({"isolated_nonzero_id", {{42U, 0, 0, 0}}});
    fixtures.push_back({"support_edge",
        {
            {90U, 0, 0, 0},
            {4U, 100000, 0, 0},
            {55U, 100001, 0, 0},
            {801U, 0, 100001, 0},
        }});
    fixtures.push_back({"negative_cell_floor",
        {
            {700U, -1, -1, -1},
            {31U, -100000, 0, 0},
            {99U, -100001, 0, 0},
            {12U, 0, 0, 0},
            {444U, 99999, 0, 0},
            {6U, -99999, 1, 0},
        }});
    fixtures.push_back({"duplicate_positions_distinct_ids",
        {
            {71U, 25000, -25000, 50000},
            {8U, 25000, -25000, 50000},
            {991U, 125000, -25000, 50000},
            {15U, 125001, -25000, 50000},
        }});
    fixtures.push_back({"adjacent_and_diagonal_cells",
        {
            {501U, 99999, 99999, 99999},
            {3U, 100000, 99999, 99999},
            {80U, 100000, 100000, 99999},
            {42U, 100000, 100000, 100000},
            {600U, 170710, 170710, 100000},
            {18U, 170711, 170711, 100000},
            {777U, 200001, 99999, 99999},
        }});

    const std::vector<CanonicalSample> cloud{
        {9001U, -175000, 25000, 50000},
        {17U, 50000, 50000, 50000},
        {404U, -75000, 25000, 50000},
        {88U, 150000, -25000, 0},
        {7007U, 50001, 50000, 50000},
        {29U, -75000, 125000, 50000},
        {100U, 0, 0, 0},
        {65530U, 100000, 0, 0},
        {321U, -1, 0, 0},
        {5U, 0, -100000, 0},
        {1234U, 70710, 70710, 0},
        {66U, 70711, 70711, 0},
    };
    const std::array<std::size_t, 12> scramble{
        7U, 2U, 10U, 0U, 5U, 9U, 3U, 11U, 1U, 8U, 6U, 4U};
    std::vector<CanonicalSample> scrambled;
    scrambled.reserve(cloud.size());
    for (std::size_t index : scramble) {
        scrambled.push_back(cloud[index]);
    }
    fixtures.push_back({"scrambled_cloud", scrambled});
    std::reverse(scrambled.begin(), scrambled.end());
    fixtures.push_back({"scrambled_cloud_reversed", scrambled});

    std::vector<CanonicalSample> lattice;
    lattice.reserve(64U);
    for (std::size_t input = 0; input < 64U; ++input) {
        const std::size_t logical = (37U * input + 11U) % 64U;
        const std::int64_t x = static_cast<std::int64_t>(logical % 4U) * 50000 - 75000;
        const std::int64_t y = static_cast<std::int64_t>((logical / 4U) % 4U) * 50000 - 75000;
        const std::int64_t z = static_cast<std::int64_t>(logical / 16U) * 50000 - 75000;
        lattice.push_back({static_cast<std::uint32_t>(1000U + 17U * logical), x, y, z});
    }
    fixtures.push_back({"water_lattice_4x4x4_scrambled", lattice});
    return fixtures;
}

std::string fixture_material(const std::vector<NamedFixture>& fixtures) {
    std::string bytes = "nextengine.nonlocal.ncga1.fixtures.v1\0";
    append_u32(bytes, 100000U);
    append_u32(bytes, static_cast<std::uint32_t>(fixtures.size()));
    for (const NamedFixture& fixture : fixtures) {
        const std::string name = fixture.name;
        append_u32(bytes, static_cast<std::uint32_t>(name.size()));
        bytes.append(name);
        append_u32(bytes, static_cast<std::uint32_t>(fixture.samples.size()));
        for (const CanonicalSample& sample : fixture.samples) {
            append_u32(bytes, sample.sample_id);
            append_i64(bytes, sample.x_um);
            append_i64(bytes, sample.y_um);
            append_i64(bytes, sample.z_um);
        }
    }
    return bytes;
}

std::string graph_material(const NeighborhoodResult& result) {
    std::string bytes = "nextengine.nonlocal.ncga1.graph.v1\0";
    append_u32(bytes, static_cast<std::uint32_t>(result.failure));
    append_u32(bytes, static_cast<std::uint32_t>(result.cells.size()));
    for (const CellEntry& cell : result.cells) {
        append_u64(bytes, cell.packed_key);
        append_u32(bytes, cell.sample_id);
    }
    append_u32(bytes, static_cast<std::uint32_t>(result.owner_ids.size()));
    for (std::uint32_t value : result.owner_ids) append_u32(bytes, value);
    append_u32(bytes, static_cast<std::uint32_t>(result.offsets.size()));
    for (std::uint32_t value : result.offsets) append_u32(bytes, value);
    append_u32(bytes, static_cast<std::uint32_t>(result.neighbor_ids.size()));
    for (std::uint32_t value : result.neighbor_ids) append_u32(bytes, value);
    return bytes;
}

std::string work_material(const NeighborWorkReceipt& work) {
    std::string bytes = "nextengine.nonlocal.ncga1.work.v1\0";
    append_u64(bytes, work.key_evaluations);
    append_u64(bytes, work.radix_sort_items);
    append_u64(bytes, work.owner_sort_items);
    append_u64(bytes, work.cell_probes);
    append_u64(bytes, work.range_search_comparisons);
    append_u64(bytes, work.distance_predicates);
    append_u64(bytes, work.count_writes);
    append_u64(bytes, work.fill_writes);
    append_u64(bytes, work.emitted_directed_pairs);
    return bytes;
}

bool same_graph(const NeighborhoodResult& lhs, const NeighborhoodResult& rhs) {
    if (lhs.failure != rhs.failure || lhs.cells.size() != rhs.cells.size()
        || lhs.owner_ids != rhs.owner_ids || lhs.offsets != rhs.offsets
        || lhs.neighbor_ids != rhs.neighbor_ids) {
        return false;
    }
    for (std::size_t index = 0; index < lhs.cells.size(); ++index) {
        if (lhs.cells[index].packed_key != rhs.cells[index].packed_key
            || lhs.cells[index].sample_id != rhs.cells[index].sample_id) {
            return false;
        }
    }
    return true;
}

bool same_work(const NeighborWorkReceipt& lhs, const NeighborWorkReceipt& rhs) {
    return work_material(lhs) == work_material(rhs);
}

struct InvariantReport {
    bool passed = false;
    std::size_t self_count = 0U;
    std::size_t maximum_degree = 0U;
};

InvariantReport check_invariants(const NeighborhoodResult& result) {
    InvariantReport report;
    if (result.failure != NeighborFailure::None
        || result.offsets.size() != result.owner_ids.size() + 1U
        || result.offsets.empty() || result.offsets.front() != 0U
        || result.offsets.back() != result.neighbor_ids.size()
        || !std::is_sorted(result.owner_ids.begin(), result.owner_ids.end())) {
        return report;
    }
    for (std::size_t index = 1; index < result.owner_ids.size(); ++index) {
        if (result.owner_ids[index - 1] >= result.owner_ids[index]) return report;
    }
    for (std::size_t index = 1; index < result.cells.size(); ++index) {
        const CellEntry& previous = result.cells[index - 1];
        const CellEntry& current = result.cells[index];
        if (previous.packed_key > current.packed_key
            || (previous.packed_key == current.packed_key
                && previous.sample_id >= current.sample_id)) {
            return report;
        }
    }
    std::unordered_map<std::uint32_t, std::size_t> row_by_id;
    for (std::size_t row = 0; row < result.owner_ids.size(); ++row) {
        row_by_id.emplace(result.owner_ids[row], row);
    }
    for (std::size_t row = 0; row < result.owner_ids.size(); ++row) {
        const std::size_t begin = result.offsets[row];
        const std::size_t end = result.offsets[row + 1U];
        report.maximum_degree = std::max(report.maximum_degree, end - begin);
        for (std::size_t slot = begin; slot < end; ++slot) {
            if (slot > begin && result.neighbor_ids[slot - 1U]
                    >= result.neighbor_ids[slot]) {
                return report;
            }
            const std::uint32_t neighbor = result.neighbor_ids[slot];
            if (neighbor == result.owner_ids[row]) ++report.self_count;
            const auto reverse_row = row_by_id.find(neighbor);
            if (reverse_row == row_by_id.end()) return report;
            const std::size_t reverse_begin = result.offsets[reverse_row->second];
            const std::size_t reverse_end = result.offsets[reverse_row->second + 1U];
            if (!std::binary_search(result.neighbor_ids.begin() + reverse_begin,
                    result.neighbor_ids.begin() + reverse_end,
                    result.owner_ids[row])) {
                return report;
            }
        }
    }
    report.passed = report.self_count == result.owner_ids.size();
    return report;
}

std::string combined_graph_material(const std::vector<NeighborhoodResult>& results) {
    std::string bytes = "nextengine.nonlocal.ncga1.graph-set.v1\0";
    append_u32(bytes, static_cast<std::uint32_t>(results.size()));
    for (const NeighborhoodResult& result : results) {
        const std::string graph = graph_material(result);
        append_u64(bytes, graph.size());
        bytes.append(graph);
    }
    return bytes;
}

std::string combined_work_material(const std::vector<NeighborhoodResult>& results) {
    std::string bytes = "nextengine.nonlocal.ncga1.work-set.v1\0";
    append_u32(bytes, static_cast<std::uint32_t>(results.size()));
    for (const NeighborhoodResult& result : results) {
        const std::string work = work_material(result.work);
        append_u64(bytes, work.size());
        bytes.append(work);
    }
    return bytes;
}

const char* variant_name(GpuNeighborVariant variant) {
    switch (variant) {
    case GpuNeighborVariant::Corrected: return "corrected";
    case GpuNeighborVariant::StrictRadius: return "strict_radius";
    case GpuNeighborVariant::TruncateSignedCell: return "truncate_signed_cell";
    case GpuNeighborVariant::ArrayIndexIdentity: return "array_index_identity";
    case GpuNeighborVariant::SameCellOnly: return "same_cell_only";
    }
    return "unknown";
}

struct ControlDefinition {
    GpuNeighborVariant variant;
    std::size_t fixture_index;
};

bool admission_controls_pass() {
    const std::vector<std::pair<std::vector<CanonicalSample>, NeighborFailure>> controls{
        {{}, NeighborFailure::InvalidInput},
        {{{7U, 0, 0, 0}, {7U, 1, 0, 0}}, NeighborFailure::DuplicateSampleId},
        {{{9U, 1000000001LL, 0, 0}}, NeighborFailure::InvalidInput},
    };
    for (const auto& [samples, expected] : controls) {
        const NeighborhoodResult reference = build_reference_neighborhood(samples);
        const NeighborhoodResult candidate = build_gpu_neighborhood(
            samples, GpuNeighborVariant::Corrected);
        if (reference.failure != expected || candidate.failure != expected
            || !same_graph(reference, candidate)) {
            return false;
        }
    }
    std::vector<CanonicalSample> excess;
    excess.reserve(257U);
    for (std::uint32_t index = 0; index < 257U; ++index) {
        excess.push_back({1000U + index, static_cast<std::int64_t>(index), 0, 0});
    }
    const NeighborhoodResult reference = build_reference_neighborhood(excess);
    const NeighborhoodResult candidate = build_gpu_neighborhood(
        excess, GpuNeighborVariant::Corrected);
    return reference.failure == NeighborFailure::CapacityExceeded
        && candidate.failure == NeighborFailure::CapacityExceeded
        && same_graph(reference, candidate);
}

int run() {
    const std::vector<NamedFixture> fixtures = make_fixtures();
    std::vector<NeighborhoodResult> references;
    std::vector<NeighborhoodResult> candidates;
    std::vector<InvariantReport> invariants;
    std::vector<bool> case_passed;
    bool cold_repeat_exact = true;
    bool positive_passed = true;
    references.reserve(fixtures.size());
    candidates.reserve(fixtures.size());
    invariants.reserve(fixtures.size());
    case_passed.reserve(fixtures.size());

    for (const NamedFixture& fixture : fixtures) {
        references.push_back(build_reference_neighborhood(fixture.samples));
        std::vector<NeighborhoodResult> runs;
        runs.reserve(COLD_REPEATS);
        for (int repeat = 0; repeat < COLD_REPEATS; ++repeat) {
            runs.push_back(build_gpu_neighborhood(
                fixture.samples, GpuNeighborVariant::Corrected));
        }
        for (int repeat = 1; repeat < COLD_REPEATS; ++repeat) {
            cold_repeat_exact = cold_repeat_exact
                && same_graph(runs.front(), runs[repeat])
                && same_work(runs.front().work, runs[repeat].work);
        }
        candidates.push_back(std::move(runs.front()));
        invariants.push_back(check_invariants(candidates.back()));
        case_passed.push_back(same_graph(references.back(), candidates.back())
            && invariants.back().passed
            && candidates.back().work.fill_writes
                == candidates.back().neighbor_ids.size()
            && candidates.back().work.emitted_directed_pairs
                == candidates.back().neighbor_ids.size());
        positive_passed = positive_passed && case_passed.back();
    }

    const bool permutation_exact = same_graph(candidates[5], candidates[6])
        && same_graph(references[5], references[6]);
    positive_passed = positive_passed && permutation_exact;

    const std::array<ControlDefinition, 4> controls{{
        {GpuNeighborVariant::StrictRadius, 1U},
        {GpuNeighborVariant::TruncateSignedCell, 2U},
        {GpuNeighborVariant::ArrayIndexIdentity, 5U},
        {GpuNeighborVariant::SameCellOnly, 4U},
    }};
    bool negative_passed = true;
    std::vector<bool> control_rejected;
    std::vector<NeighborhoodResult> control_results;
    for (const ControlDefinition& control : controls) {
        control_results.push_back(build_gpu_neighborhood(
            fixtures[control.fixture_index].samples, control.variant));
        control_rejected.push_back(!same_graph(
            references[control.fixture_index], control_results.back()));
        negative_passed = negative_passed && control_rejected.back();
    }
    const bool admission_passed = admission_controls_pass();
    const bool passed = positive_passed && cold_repeat_exact
        && negative_passed && admission_passed;

    const std::string fixture_root = sha256_hex(fixture_material(fixtures));
    const std::string reference_root = sha256_hex(
        combined_graph_material(references));
    const std::string candidate_root = sha256_hex(
        combined_graph_material(candidates));
    const std::string work_root = sha256_hex(
        combined_work_material(candidates));
    const std::string control_root = sha256_hex(
        combined_graph_material(control_results));

    std::ostringstream output;
    output << "{\"schema\":\"nextengine.nonlocal.corrected_cuda_neighbors.ncga1.v1\","
              "\"status\":\""
           << (passed ? "PASS" : "FAIL")
           << "\",\"claim\":\"EXACT_NEIGHBORHOOD_INDEX_CORRESPONDENCE_"
           << (passed ? "SUPPORTED_BOUNDED" : "REJECTED")
           << "\",\"fixture_root\":\"" << fixture_root
           << "\",\"reference_root\":\"" << reference_root
           << "\",\"candidate_root\":\"" << candidate_root
           << "\",\"work_root\":\"" << work_root
           << "\",\"control_root\":\"" << control_root
           << "\",\"environment\":" << gpu_neighbor_environment_json()
           << ",\"cold_repeats\":" << COLD_REPEATS
           << ",\"cold_repeat_exact\":"
           << (cold_repeat_exact ? "true" : "false")
           << ",\"permutation_exact\":"
           << (permutation_exact ? "true" : "false")
           << ",\"admission_controls_passed\":"
           << (admission_passed ? "true" : "false")
           << ",\"cases\":[";
    for (std::size_t index = 0; index < fixtures.size(); ++index) {
        if (index != 0U) output << ',';
        const NeighborhoodResult& candidate = candidates[index];
        output << "{\"name\":\"" << fixtures[index].name
               << "\",\"status\":\"" << (case_passed[index] ? "PASS" : "FAIL")
               << "\",\"samples\":" << candidate.owner_ids.size()
               << ",\"directed_pairs\":" << candidate.neighbor_ids.size()
               << ",\"maximum_degree\":" << invariants[index].maximum_degree
               << ",\"self_count\":" << invariants[index].self_count
               << ",\"graph_root\":\"" << sha256_hex(graph_material(candidate))
               << "\",\"work_root\":\"" << sha256_hex(work_material(candidate.work))
               << "\"}";
    }
    output << "],\"negative_controls\":[";
    for (std::size_t index = 0; index < controls.size(); ++index) {
        if (index != 0U) output << ',';
        output << "{\"name\":\"" << variant_name(controls[index].variant)
               << "\",\"fixture\":\""
               << fixtures[controls[index].fixture_index].name
               << "\",\"rejected\":"
               << (control_rejected[index] ? "true" : "false") << '}';
    }
    output << "]}";
    std::cout << output.str() << '\n';
    return passed ? 0 : 1;
}

} // namespace
} // namespace nextengine::nonlocal::gpu_neighbor_audit

int main(int argc, char** argv) {
    try {
        if (argc != 2 || std::string(argv[1]) != "--self-test") {
            std::cerr << "usage: nonlocal-corrected-cuda-neighbors --self-test\n";
            return 2;
        }
        return nextengine::nonlocal::gpu_neighbor_audit::run();
    } catch (const std::exception& error) {
        std::cerr << "nonlocal-corrected-cuda-neighbors: " << error.what() << '\n';
        return 1;
    }
}
