#pragma once

#include <cstdint>
#include <string>
#include <vector>

namespace nextengine::nonlocal::gpu_neighbor_audit {

enum class GpuNeighborVariant : std::uint32_t {
    Corrected = 0,
    StrictRadius = 1,
    TruncateSignedCell = 2,
    ArrayIndexIdentity = 3,
    SameCellOnly = 4,
};

enum class NeighborFailure : std::uint32_t {
    None = 0,
    InvalidInput = 1,
    DuplicateSampleId = 2,
    CapacityExceeded = 3,
    CellRangeExceeded = 4,
    DeviceFailure = 5,
};

struct CanonicalSample {
    std::uint32_t sample_id = 0;
    std::int64_t x_um = 0;
    std::int64_t y_um = 0;
    std::int64_t z_um = 0;
};

struct CellEntry {
    std::uint64_t packed_key = 0;
    std::uint32_t sample_id = 0;
};

struct NeighborWorkReceipt {
    std::uint64_t key_evaluations = 0;
    std::uint64_t radix_sort_items = 0;
    std::uint64_t owner_sort_items = 0;
    std::uint64_t cell_probes = 0;
    std::uint64_t range_search_comparisons = 0;
    std::uint64_t distance_predicates = 0;
    std::uint64_t count_writes = 0;
    std::uint64_t fill_writes = 0;
    std::uint64_t emitted_directed_pairs = 0;
};

struct NeighborhoodResult {
    NeighborFailure failure = NeighborFailure::None;
    std::vector<CellEntry> cells;
    std::vector<std::uint32_t> owner_ids;
    std::vector<std::uint32_t> offsets;
    std::vector<std::uint32_t> neighbor_ids;
    NeighborWorkReceipt work;
};

NeighborhoodResult build_reference_neighborhood(
    const std::vector<CanonicalSample>& samples);

NeighborhoodResult build_gpu_neighborhood(
    const std::vector<CanonicalSample>& samples,
    GpuNeighborVariant variant);

std::string gpu_neighbor_environment_json();

} // namespace nextengine::nonlocal::gpu_neighbor_audit
