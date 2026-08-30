#include "corrected_cuda_full_step.hpp"

#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <limits>
#include <stdexcept>
#include <unordered_set>
#include <utility>

namespace nextengine::nonlocal::gpu_full_step {
namespace {

constexpr double kMicrometresPerMetre = 1000000.0;

std::int64_t quantize(double value) {
    if (!std::isfinite(value)
        || std::abs(value) > static_cast<double>(std::numeric_limits<std::int64_t>::max())
            / kMicrometresPerMetre) {
        throw std::invalid_argument("position is not quantizable");
    }
    return static_cast<std::int64_t>(std::llround(value * kMicrometresPerMetre));
}

std::uint64_t square(std::int64_t value) {
    const auto magnitude = value < 0
        ? static_cast<std::uint64_t>(-value)
        : static_cast<std::uint64_t>(value);
    return magnitude * magnitude;
}

void append_u32(std::string& bytes, std::uint32_t value) {
    for (unsigned int shift = 0U; shift < 32U; shift += 8U) {
        bytes.push_back(static_cast<char>((value >> shift) & 0xffU));
    }
}

void append_u64(std::string& bytes, std::uint64_t value) {
    for (unsigned int shift = 0U; shift < 64U; shift += 8U) {
        bytes.push_back(static_cast<char>((value >> shift) & 0xffU));
    }
}

std::uint64_t splitmix64(std::uint64_t value) {
    value += 0x9e3779b97f4a7c15ULL;
    value = (value ^ (value >> 30U)) * 0xbf58476d1ce4e5b9ULL;
    value = (value ^ (value >> 27U)) * 0x94d049bb133111ebULL;
    return value ^ (value >> 31U);
}

bool valid_profile(const NonlocalGpuProfile& profile) {
    return profile.id == "nonlocal-water-50k-v1"
        && profile.dt > 0.0 && profile.spacing > 0.0
        && profile.horizon > 0.0 && profile.mass > 0.0
        && profile.rest_density > 0.0 && profile.kappa >= 0.0
        && profile.lambda >= 0.0 && profile.mu >= 0.0
        && profile.gamma >= 0.0 && profile.ghost_layers == 3U
        && profile.maximum_dynamic_samples == kMaximumDynamicSamples
        && profile.maximum_neighbors == kMaximumNeighbors;
}

} // namespace

NonlocalGpuProfile nonlocal_water_profile() {
    NonlocalGpuProfile profile;
    profile.id = "nonlocal-water-50k-v1";
    return profile;
}

std::vector<NonlocalGpuSample> make_lattice_state(
    const NonlocalGpuProfile& profile,
    std::uint32_t nx,
    std::uint32_t ny,
    std::uint32_t nz,
    bool permuted,
    bool advected) {
    if (!valid_profile(profile) || nx == 0U || ny == 0U || nz == 0U) {
        throw std::invalid_argument("invalid lattice profile");
    }
    const std::uint64_t count64 = static_cast<std::uint64_t>(nx) * ny * nz;
    if (count64 > profile.maximum_dynamic_samples) {
        throw std::invalid_argument("lattice exceeds dynamic capacity");
    }
    const std::size_t count = static_cast<std::size_t>(count64);
    std::vector<NonlocalGpuSample> result;
    result.reserve(count);
    std::vector<bool> seen(count, false);
    for (std::size_t input = 0U; input < count; ++input) {
        const std::size_t logical = permuted
            ? (23449ULL * input + 7919ULL) % count
            : input;
        if (seen[logical]) {
            throw std::logic_error("permutation is not bijective");
        }
        seen[logical] = true;
        const std::size_t ix = logical % nx;
        const std::size_t iy = (logical / nx) % ny;
        const std::size_t iz = logical / (static_cast<std::size_t>(nx) * ny);
        Vec3d position{
            0.275 + profile.spacing * static_cast<double>(ix),
            0.275 + profile.spacing * static_cast<double>(iy),
            0.025 + profile.spacing * static_cast<double>(iz),
        };
        if (advected) {
            const auto displacement = [&](std::uint64_t lane) {
                return (static_cast<double>(splitmix64(logical + lane) % 16001ULL)
                           - 8000.0)
                    * 1.0e-6;
            };
            position.x += displacement(0x123456789abcdef0ULL);
            position.y += displacement(0x0fedcba987654321ULL);
            position.z += displacement(0xa5a5a5a55a5a5a5aULL);
        }
        result.push_back({static_cast<std::uint32_t>(1000U + 17U * logical),
            position, position, {0.0, 0.0, 0.0}});
    }
    return result;
}

std::vector<NonlocalGpuGhost> make_basin_ghosts(
    const NonlocalGpuProfile& profile,
    std::uint32_t first_sample_id) {
    if (!valid_profile(profile)) {
        throw std::invalid_argument("invalid ghost profile");
    }
    const auto cell_count = [&](double extent) {
        return static_cast<int>(std::llround(extent / profile.spacing));
    };
    const int nx = cell_count(profile.basin_extent.x);
    const int ny = cell_count(profile.basin_extent.y);
    const int nz = cell_count(profile.basin_extent.z);
    const int layers = static_cast<int>(profile.ghost_layers);
    std::vector<NonlocalGpuGhost> ghosts;
    for (int z = -layers; z < nz + layers; ++z) {
        for (int y = -layers; y < ny + layers; ++y) {
            for (int x = -layers; x < nx + layers; ++x) {
                if (x >= 0 && x < nx && y >= 0 && y < ny && z >= 0 && z < nz) {
                    continue;
                }
                const std::uint64_t ordinal = ghosts.size();
                if (ordinal > std::numeric_limits<std::uint32_t>::max()
                        - first_sample_id) {
                    throw std::overflow_error("ghost ID range exceeded");
                }
                ghosts.push_back({first_sample_id + static_cast<std::uint32_t>(ordinal),
                    {(static_cast<double>(x) + 0.5) * profile.spacing,
                        (static_cast<double>(y) + 0.5) * profile.spacing,
                        (static_cast<double>(z) + 0.5) * profile.spacing}});
            }
        }
    }
    return ghosts;
}

NonlocalGpuGraphResult build_reference_graph(
    const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& samples,
    const std::vector<NonlocalGpuGhost>& ghosts,
    bool strict_radius) {
    NonlocalGpuGraphResult result;
    if (!valid_profile(profile) || samples.empty()
        || samples.size() > profile.maximum_dynamic_samples) {
        result.failure = NonlocalGpuFailure::InvalidProfile;
        return result;
    }
    struct Point {
        std::uint32_t id;
        std::array<std::int64_t, 3> p;
    };
    std::vector<Point> dynamic;
    std::vector<Point> all;
    dynamic.reserve(samples.size());
    all.reserve(samples.size() + ghosts.size());
    std::unordered_set<std::uint32_t> ids;
    try {
        for (const auto& sample : samples) {
            if (!ids.insert(sample.sample_id).second) {
                result.failure = NonlocalGpuFailure::DuplicateSampleId;
                return result;
            }
            dynamic.push_back({sample.sample_id,
                {quantize(sample.current.x), quantize(sample.current.y),
                    quantize(sample.current.z)}});
        }
        for (const auto& ghost : ghosts) {
            if (!ids.insert(ghost.sample_id).second) {
                result.failure = NonlocalGpuFailure::DuplicateSampleId;
                return result;
            }
            all.push_back({ghost.sample_id,
                {quantize(ghost.position.x), quantize(ghost.position.y),
                    quantize(ghost.position.z)}});
        }
    } catch (const std::exception&) {
        result.failure = NonlocalGpuFailure::InvalidState;
        return result;
    }
    std::sort(dynamic.begin(), dynamic.end(),
        [](const Point& lhs, const Point& rhs) { return lhs.id < rhs.id; });
    all.insert(all.end(), dynamic.begin(), dynamic.end());
    std::sort(all.begin(), all.end(),
        [](const Point& lhs, const Point& rhs) { return lhs.id < rhs.id; });
    result.dynamic_samples = static_cast<std::uint32_t>(dynamic.size());
    result.ghost_samples = static_cast<std::uint32_t>(ghosts.size());
    result.offsets.push_back(0U);
    const std::int64_t radius = quantize(profile.horizon);
    const std::uint64_t limit = static_cast<std::uint64_t>(radius)
        * static_cast<std::uint64_t>(radius);
    for (const Point& owner : dynamic) {
        result.owner_ids.push_back(owner.id);
        std::vector<std::uint32_t> row;
        for (const Point& candidate : all) {
            ++result.work.distance_predicates;
            const std::uint64_t distance = square(owner.p[0] - candidate.p[0])
                + square(owner.p[1] - candidate.p[1])
                + square(owner.p[2] - candidate.p[2]);
            if ((strict_radius && distance < limit)
                || (!strict_radius && distance <= limit)) {
                row.push_back(candidate.id);
            }
        }
        if (row.size() > profile.maximum_neighbors) {
            result.failure = NonlocalGpuFailure::CapacityExceeded;
            return result;
        }
        std::sort(row.begin(), row.end());
        result.maximum_degree = std::max(result.maximum_degree,
            static_cast<std::uint32_t>(row.size()));
        result.neighbor_ids.insert(result.neighbor_ids.end(), row.begin(), row.end());
        result.offsets.push_back(static_cast<std::uint32_t>(result.neighbor_ids.size()));
    }
    result.directed_pairs = static_cast<std::uint32_t>(result.neighbor_ids.size());
    result.work.graph_builds = 1U;
    result.work.key_evaluations = all.size();
    result.work.emitted_directed_pairs = result.neighbor_ids.size();
    result.work.row_sort_items = result.neighbor_ids.size();
    return result;
}

std::string graph_semantic_root(const NonlocalGpuGraphResult& graph) {
    std::string bytes = "nextengine.nonlocal.ncgp1.graph.v1\0";
    append_u32(bytes, static_cast<std::uint32_t>(graph.failure));
    append_u32(bytes, graph.dynamic_samples);
    append_u32(bytes, graph.ghost_samples);
    append_u32(bytes, static_cast<std::uint32_t>(graph.owner_ids.size()));
    for (const auto value : graph.owner_ids) append_u32(bytes, value);
    append_u32(bytes, static_cast<std::uint32_t>(graph.offsets.size()));
    for (const auto value : graph.offsets) append_u32(bytes, value);
    append_u32(bytes, static_cast<std::uint32_t>(graph.neighbor_ids.size()));
    for (const auto value : graph.neighbor_ids) append_u32(bytes, value);
    return sha256_hex(bytes);
}

std::string work_semantic_root(const NonlocalGpuWorkReceipt& work) {
    std::string bytes = "nextengine.nonlocal.ncgp1.work.v1\0";
    const std::array<std::uint64_t, 21> values{
        work.uploads, work.graph_builds, work.key_evaluations,
        work.radix_sort_items, work.cell_probes, work.distance_predicates,
        work.emitted_directed_pairs, work.row_sort_items,
        work.density_kernel_evaluations, work.energy_pair_visits,
        work.gradient_pair_visits, work.hvp_pair_visits,
        work.hvp_applications, work.reduction_values, work.outer_trials,
        work.accepted_trials, work.rejected_trials, work.contact_projections,
        work.host_to_device_bytes, work.device_to_host_bytes, 0U};
    for (const auto value : values) append_u64(bytes, value);
    return sha256_hex(bytes);
}

NonlocalGpuEvaluationResult evaluate_reference(
    const NonlocalGpuProfile&,
    const std::vector<NonlocalGpuSample>&,
    const std::vector<NonlocalGpuGhost>&,
    const std::vector<Vec3d>*,
    NonlocalGpuVariant) {
    NonlocalGpuEvaluationResult result;
    result.failure = NonlocalGpuFailure::InvalidState;
    return result;
}

} // namespace nextengine::nonlocal::gpu_full_step
