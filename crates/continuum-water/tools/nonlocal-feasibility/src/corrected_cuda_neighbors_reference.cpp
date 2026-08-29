#include "corrected_cuda_neighbors.hpp"

#include <algorithm>
#include <array>
#include <limits>
#include <stdexcept>
#include <unordered_set>

namespace nextengine::nonlocal::gpu_neighbor_audit {
namespace {

constexpr std::int64_t SUPPORT_UM = 100000;
constexpr std::int64_t COORDINATE_LIMIT_UM = 1000000000;
constexpr std::int64_t CELL_BIAS = 1LL << 20;
constexpr std::int64_t CELL_MIN = -CELL_BIAS;
constexpr std::int64_t CELL_MAX = CELL_BIAS - 1;
constexpr std::size_t MAX_SAMPLES = 256;

bool input_admitted(const std::vector<CanonicalSample>& samples,
    NeighborFailure& failure) {
    if (samples.empty() || samples.size() > MAX_SAMPLES) {
        failure = samples.empty() ? NeighborFailure::InvalidInput
                                  : NeighborFailure::CapacityExceeded;
        return false;
    }
    std::unordered_set<std::uint32_t> ids;
    ids.reserve(samples.size());
    for (const CanonicalSample& sample : samples) {
        if (sample.x_um < -COORDINATE_LIMIT_UM
            || sample.x_um > COORDINATE_LIMIT_UM
            || sample.y_um < -COORDINATE_LIMIT_UM
            || sample.y_um > COORDINATE_LIMIT_UM
            || sample.z_um < -COORDINATE_LIMIT_UM
            || sample.z_um > COORDINATE_LIMIT_UM) {
            failure = NeighborFailure::InvalidInput;
            return false;
        }
        if (!ids.insert(sample.sample_id).second) {
            failure = NeighborFailure::DuplicateSampleId;
            return false;
        }
    }
    return true;
}

std::int64_t floor_division(std::int64_t numerator, std::int64_t denominator) {
    const std::int64_t quotient = numerator / denominator;
    const std::int64_t remainder = numerator % denominator;
    return quotient - (remainder < 0 ? 1 : 0);
}

std::uint64_t packed_cell_key(const CanonicalSample& sample) {
    const std::array<std::int64_t, 3> cell{
        floor_division(sample.x_um, SUPPORT_UM),
        floor_division(sample.y_um, SUPPORT_UM),
        floor_division(sample.z_um, SUPPORT_UM),
    };
    for (std::int64_t axis : cell) {
        if (axis < CELL_MIN || axis > CELL_MAX) {
            throw std::overflow_error("NCGA1 reference cell range exceeded");
        }
    }
    return (static_cast<std::uint64_t>(cell[0] + CELL_BIAS) << 42U)
        | (static_cast<std::uint64_t>(cell[1] + CELL_BIAS) << 21U)
        | static_cast<std::uint64_t>(cell[2] + CELL_BIAS);
}

bool within_support(const CanonicalSample& first,
    const CanonicalSample& second) {
    const std::int64_t dx = first.x_um - second.x_um;
    const std::int64_t dy = first.y_um - second.y_um;
    const std::int64_t dz = first.z_um - second.z_um;
    const std::uint64_t squared = static_cast<std::uint64_t>(dx * dx)
        + static_cast<std::uint64_t>(dy * dy)
        + static_cast<std::uint64_t>(dz * dz);
    return squared <= static_cast<std::uint64_t>(SUPPORT_UM * SUPPORT_UM);
}

} // namespace

NeighborhoodResult build_reference_neighborhood(
    const std::vector<CanonicalSample>& samples) {
    NeighborhoodResult result;
    if (!input_admitted(samples, result.failure)) {
        return result;
    }

    std::vector<std::size_t> owner_order(samples.size());
    for (std::size_t index = 0; index < samples.size(); ++index) {
        owner_order[index] = index;
        result.cells.push_back(
            {packed_cell_key(samples[index]), samples[index].sample_id});
    }
    std::sort(result.cells.begin(), result.cells.end(),
        [](const CellEntry& lhs, const CellEntry& rhs) {
            return lhs.packed_key < rhs.packed_key
                || (lhs.packed_key == rhs.packed_key
                    && lhs.sample_id < rhs.sample_id);
        });
    std::sort(owner_order.begin(), owner_order.end(),
        [&](std::size_t lhs, std::size_t rhs) {
            return samples[lhs].sample_id < samples[rhs].sample_id;
        });

    result.offsets.reserve(samples.size() + 1U);
    result.offsets.push_back(0U);
    for (std::size_t owner_index : owner_order) {
        const CanonicalSample& owner = samples[owner_index];
        result.owner_ids.push_back(owner.sample_id);
        std::vector<std::uint32_t> row;
        row.reserve(samples.size());
        for (const CanonicalSample& candidate : samples) {
            if (within_support(owner, candidate)) {
                row.push_back(candidate.sample_id);
            }
        }
        std::sort(row.begin(), row.end());
        result.neighbor_ids.insert(
            result.neighbor_ids.end(), row.begin(), row.end());
        result.offsets.push_back(
            static_cast<std::uint32_t>(result.neighbor_ids.size()));
    }
    result.failure = NeighborFailure::None;
    return result;
}

} // namespace nextengine::nonlocal::gpu_neighbor_audit
