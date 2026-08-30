#include "corrected_cuda_full_step.hpp"

#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstring>
#include <limits>
#include <unordered_map>
#include <stdexcept>
#include <unordered_set>
#include <utility>

namespace nextengine::nonlocal::gpu_full_step {
namespace {

constexpr double kMicrometresPerMetre = 1000000.0;
constexpr std::size_t kMaximumTotalSamples = 100000U;

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

void append_f32(std::string& bytes, float value) {
    std::uint32_t bits = 0U;
    static_assert(sizeof(bits) == sizeof(value));
    std::memcpy(&bits, &value, sizeof(bits));
    append_u32(bytes, bits);
}

void append_f64(std::string& bytes, double value) {
    std::uint64_t bits = 0U;
    static_assert(sizeof(bits) == sizeof(value));
    std::memcpy(&bits, &value, sizeof(bits));
    append_u64(bytes, bits);
}

void append_string(std::string& bytes, const std::string& value) {
    append_u64(bytes, value.size());
    bytes.append(value);
}

bool finite_vec(const Vec3d& value) {
    return std::isfinite(value.x) && std::isfinite(value.y)
        && std::isfinite(value.z);
}

bool finite_binary32_vec(const Vec3d& value) {
    if (!finite_vec(value)) return false;
    return std::isfinite(static_cast<float>(value.x))
        && std::isfinite(static_cast<float>(value.y))
        && std::isfinite(static_cast<float>(value.z));
}

Vec3d binary32_vec(const Vec3d& value) {
    return {static_cast<double>(static_cast<float>(value.x)),
        static_cast<double>(static_cast<float>(value.y)),
        static_cast<double>(static_cast<float>(value.z))};
}

std::uint64_t splitmix64(std::uint64_t value) {
    value += 0x9e3779b97f4a7c15ULL;
    value = (value ^ (value >> 30U)) * 0xbf58476d1ce4e5b9ULL;
    value = (value ^ (value >> 27U)) * 0x94d049bb133111ebULL;
    return value ^ (value >> 31U);
}

bool valid_profile(const NonlocalGpuProfile& profile) {
    const std::array<double, 9> scalars{profile.dt, profile.spacing,
        profile.horizon, profile.mass, profile.rest_density, profile.kappa,
        profile.lambda, profile.mu, profile.gamma};
    const bool finite_scalars = std::all_of(scalars.begin(), scalars.end(),
        [](double value) {
            return std::isfinite(value)
                && std::isfinite(static_cast<float>(value));
        });
    return profile.id == "nonlocal-water-50k-v1" && finite_scalars
        && profile.dt > 0.0 && profile.spacing > 0.0
        && profile.horizon > 0.0 && profile.mass > 0.0
        && profile.rest_density > 0.0 && profile.kappa >= 0.0
        && profile.lambda >= 0.0 && profile.mu >= 0.0
        && profile.gamma >= 0.0 && finite_binary32_vec(profile.gravity)
        && finite_binary32_vec(profile.basin_extent)
        && profile.basin_extent.x > profile.spacing
        && profile.basin_extent.y > profile.spacing
        && profile.basin_extent.z > profile.spacing
        && profile.ghost_layers == 3U
        && profile.maximum_dynamic_samples == kMaximumDynamicSamples
        && profile.maximum_neighbors == kMaximumNeighbors;
}

struct WideVec3 {
    long double x = 0.0L;
    long double y = 0.0L;
    long double z = 0.0L;
};

WideVec3 wide(const Vec3d& value) {
    return {static_cast<long double>(value.x),
        static_cast<long double>(value.y), static_cast<long double>(value.z)};
}

WideVec3 add(WideVec3 lhs, WideVec3 rhs) {
    return {lhs.x + rhs.x, lhs.y + rhs.y, lhs.z + rhs.z};
}

WideVec3 subtract(WideVec3 lhs, WideVec3 rhs) {
    return {lhs.x - rhs.x, lhs.y - rhs.y, lhs.z - rhs.z};
}

WideVec3 scale(WideVec3 value, long double factor) {
    return {value.x * factor, value.y * factor, value.z * factor};
}

long double dot(WideVec3 lhs, WideVec3 rhs) {
    return lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z;
}

long double norm(WideVec3 value) { return std::sqrt(dot(value, value)); }

WideVec3 radial_apply(WideVec3 normal,
    long double radial,
    long double tangential,
    WideVec3 value) {
    return add(scale(value, tangential),
        scale(normal, (radial - tangential) * dot(normal, value)));
}

struct WideKernel {
    long double value = 0.0L;
    long double first = 0.0L;
    long double second = 0.0L;
};

WideKernel kernel(long double radius, long double horizon) {
    const long double pi = std::acos(-1.0L);
    const long double q = 2.0L * radius / horizon;
    const long double alpha = 3.0L
        / (2.0L * pi * horizon * horizon * horizon);
    long double value = 0.0L;
    long double first_q = 0.0L;
    long double second_q = 0.0L;
    if (q < 1.0L) {
        value = alpha * (2.0L / 3.0L - q * q + 0.5L * q * q * q);
        first_q = alpha * (-2.0L * q + 1.5L * q * q);
        second_q = alpha * (-2.0L + 3.0L * q);
    } else if (q <= 2.0L) {
        const long double tail = 2.0L - q;
        value = alpha * tail * tail * tail / 6.0L;
        first_q = -0.5L * alpha * tail * tail;
        second_q = alpha * tail;
    }
    const long double chain = 2.0L / horizon;
    return {value, first_q * chain, second_q * chain * chain};
}

void surface(long double radius,
    long double spacing,
    long double& potential,
    long double& force,
    long double& derivative) {
    const long double q = radius / spacing;
    if (q <= 1.0L) {
        force = q * q - 1.0L;
        derivative = 2.0L * q / spacing;
        potential = spacing * (q * q * q / 3.0L - q - 2.0L / 3.0L);
    } else if (q < 3.0L) {
        const long double shifted = q - 2.0L;
        force = 1.0L - shifted * shifted;
        derivative = -2.0L * shifted / spacing;
        potential = spacing
            * (q - shifted * shifted * shifted / 3.0L - 8.0L / 3.0L);
    } else {
        potential = 0.0L;
        force = 0.0L;
        derivative = 0.0L;
    }
}

} // namespace

NonlocalGpuProfile nonlocal_water_profile() {
    NonlocalGpuProfile profile;
    profile.id = "nonlocal-water-50k-v1";
    return profile;
}

NonlocalGpuFailure validate_nonlocal_input(
    const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& samples,
    const std::vector<NonlocalGpuGhost>& ghosts) {
    if (!valid_profile(profile)) return NonlocalGpuFailure::InvalidProfile;
    if (samples.empty() || samples.size() > profile.maximum_dynamic_samples
        || samples.size() + ghosts.size()
            > kMaximumTotalSamples) {
        return NonlocalGpuFailure::CapacityExceeded;
    }
    std::unordered_set<std::uint32_t> ids;
    ids.reserve(samples.size() + ghosts.size());
    for (const NonlocalGpuSample& sample : samples) {
        if (!ids.insert(sample.sample_id).second) {
            return NonlocalGpuFailure::DuplicateSampleId;
        }
        if (!finite_binary32_vec(sample.reference)
            || !finite_binary32_vec(sample.current)
            || !finite_binary32_vec(sample.velocity)) {
            return NonlocalGpuFailure::Nonfinite;
        }
    }
    std::uint32_t prior = 0U;
    for (std::size_t index = 0U; index < ghosts.size(); ++index) {
        const NonlocalGpuGhost& ghost = ghosts[index];
        if (!ids.insert(ghost.sample_id).second
            || (index != 0U && ghost.sample_id <= prior)) {
            return NonlocalGpuFailure::DuplicateSampleId;
        }
        prior = ghost.sample_id;
        if (!finite_binary32_vec(ghost.position)) {
            return NonlocalGpuFailure::Nonfinite;
        }
    }
    return NonlocalGpuFailure::None;
}

std::vector<NonlocalGpuSample> canonicalize_samples_binary32(
    const std::vector<NonlocalGpuSample>& samples) {
    std::vector<NonlocalGpuSample> result = samples;
    for (NonlocalGpuSample& sample : result) {
        sample.reference = binary32_vec(sample.reference);
        sample.current = binary32_vec(sample.current);
        sample.velocity = binary32_vec(sample.velocity);
    }
    return result;
}

std::vector<NonlocalGpuGhost> canonicalize_ghosts_binary32(
    const std::vector<NonlocalGpuGhost>& ghosts) {
    std::vector<NonlocalGpuGhost> result = ghosts;
    for (NonlocalGpuGhost& ghost : result) {
        ghost.position = binary32_vec(ghost.position);
    }
    return result;
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
            const double low = static_cast<double>(
                static_cast<float>(0.5 * profile.spacing));
            const Vec3d high{
                static_cast<double>(static_cast<float>(
                    profile.basin_extent.x - 0.5 * profile.spacing)),
                static_cast<double>(static_cast<float>(
                    profile.basin_extent.y - 0.5 * profile.spacing)),
                static_cast<double>(static_cast<float>(
                    profile.basin_extent.z - 0.5 * profile.spacing))};
            position.x = std::clamp(position.x, low, high.x);
            position.y = std::clamp(position.y, low, high.y);
            position.z = std::clamp(position.z, low, high.z);
        }
        result.push_back({static_cast<std::uint32_t>(1000U + 17U * logical),
            position, position, {0.0, 0.0, 0.0}});
    }
    return canonicalize_samples_binary32(result);
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
    return canonicalize_ghosts_binary32(ghosts);
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
    const bool boundary_extended = work.boundary_face_tests != 0U
        || work.boundary_face_hits != 0U || work.boundary_face_mask_xor != 0U;
    const bool extended = work.diagonal_probes != 0U
        || work.scalar_reductions != 0U || work.vector_kernel_values != 0U
        || work.boundary_intersections != 0U || work.radius_shrinks != 0U
        || work.radius_expands != 0U
        || work.projected_gradient_components != 0U
        || work.contact_projections != 0U || work.state_updates != 0U;
    std::string bytes = boundary_extended
        ? "nextengine.nonlocal.ncgp1.work.v3\0"
        : (extended ? "nextengine.nonlocal.ncgp1.work.v2\0"
                    : "nextengine.nonlocal.ncgp1.work.v1\0");
    const std::vector<std::uint64_t> values = boundary_extended
        ? std::vector<std::uint64_t>{work.uploads, work.graph_builds,
              work.key_evaluations, work.radix_sort_items, work.cell_probes,
              work.distance_predicates, work.emitted_directed_pairs,
              work.row_sort_items, work.density_kernel_evaluations,
              work.energy_pair_visits, work.gradient_pair_visits,
              work.hvp_pair_visits, work.hvp_applications,
              work.diagonal_probes, work.reduction_values,
              work.scalar_reductions, work.vector_kernel_values,
              work.boundary_intersections, work.outer_trials,
              work.accepted_trials, work.rejected_trials,
              work.radius_shrinks, work.radius_expands,
              work.projected_gradient_components, work.contact_projections,
              work.boundary_face_tests, work.boundary_face_hits,
              work.boundary_face_mask_xor, work.state_updates,
              work.host_to_device_bytes, work.device_to_host_bytes}
        : extended
        ? std::vector<std::uint64_t>{work.uploads, work.graph_builds,
              work.key_evaluations, work.radix_sort_items, work.cell_probes,
              work.distance_predicates, work.emitted_directed_pairs,
              work.row_sort_items, work.density_kernel_evaluations,
              work.energy_pair_visits, work.gradient_pair_visits,
              work.hvp_pair_visits, work.hvp_applications,
              work.diagonal_probes, work.reduction_values,
              work.scalar_reductions, work.vector_kernel_values,
              work.boundary_intersections, work.outer_trials,
              work.accepted_trials, work.rejected_trials,
              work.radius_shrinks, work.radius_expands,
              work.projected_gradient_components, work.contact_projections,
              work.state_updates, work.host_to_device_bytes,
              work.device_to_host_bytes}
        : std::vector<std::uint64_t>{work.uploads, work.graph_builds,
              work.key_evaluations, work.radix_sort_items, work.cell_probes,
              work.distance_predicates, work.emitted_directed_pairs,
              work.row_sort_items, work.density_kernel_evaluations,
              work.energy_pair_visits, work.gradient_pair_visits,
              work.hvp_pair_visits, work.hvp_applications,
              work.reduction_values, work.outer_trials,
              work.accepted_trials, work.rejected_trials,
              work.contact_projections, work.host_to_device_bytes,
              work.device_to_host_bytes, 0U};
    for (const auto value : values) append_u64(bytes, value);
    return sha256_hex(bytes);
}

std::string profile_semantic_root(const NonlocalGpuProfile& profile) {
    std::string bytes = "nextengine.nonlocal.ncgp1.profile.v1\0";
    append_string(bytes, profile.id);
    for (const double value : std::array<double, 18>{profile.dt,
             profile.spacing, profile.horizon, profile.mass,
             profile.rest_density, profile.kappa, profile.lambda, profile.mu,
             profile.gamma, profile.gravity.x, profile.gravity.y,
             profile.gravity.z, profile.basin_extent.x, profile.basin_extent.y,
             profile.basin_extent.z, static_cast<double>(profile.ghost_layers),
             static_cast<double>(profile.maximum_dynamic_samples),
             static_cast<double>(profile.maximum_neighbors)}) {
        append_f64(bytes, value);
    }
    return sha256_hex(bytes);
}

std::string input_semantic_root(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& input_samples,
    const std::vector<NonlocalGpuGhost>& input_ghosts) {
    auto samples = canonicalize_samples_binary32(input_samples);
    auto ghosts = canonicalize_ghosts_binary32(input_ghosts);
    std::sort(samples.begin(), samples.end(), [](const auto& lhs, const auto& rhs) {
        return lhs.sample_id < rhs.sample_id;
    });
    std::sort(ghosts.begin(), ghosts.end(), [](const auto& lhs, const auto& rhs) {
        return lhs.sample_id < rhs.sample_id;
    });
    std::string bytes = "nextengine.nonlocal.ncgp1.input.v1\0";
    append_string(bytes, profile_semantic_root(profile));
    append_u64(bytes, samples.size());
    for (const NonlocalGpuSample& sample : samples) {
        append_u32(bytes, sample.sample_id);
        for (const double value : std::array<double, 9>{sample.reference.x,
                 sample.reference.y, sample.reference.z, sample.current.x,
                 sample.current.y, sample.current.z, sample.velocity.x,
                 sample.velocity.y, sample.velocity.z}) {
            append_f32(bytes, static_cast<float>(value));
        }
    }
    append_u64(bytes, ghosts.size());
    for (const NonlocalGpuGhost& ghost : ghosts) {
        append_u32(bytes, ghost.sample_id);
        append_f32(bytes, static_cast<float>(ghost.position.x));
        append_f32(bytes, static_cast<float>(ghost.position.y));
        append_f32(bytes, static_cast<float>(ghost.position.z));
    }
    return sha256_hex(bytes);
}

std::string step_work_semantic_root(const NonlocalGpuProfile& profile,
    const NonlocalGpuStepResult& result) {
    std::string bytes = "nextengine.nonlocal.ncgp1.step-work.v1\0";
    append_string(bytes, profile_semantic_root(profile));
    append_u32(bytes, result.hvp_budget);
    append_u32(bytes, static_cast<std::uint32_t>(result.solver_profile));
    append_u32(bytes, static_cast<std::uint32_t>(result.variant));
    append_string(bytes, work_semantic_root(result.work));
    if (result.variant == NonlocalGpuVariant::CompensatedStateF32
        || result.variant == NonlocalGpuVariant::CompensatedOmitLow
        || result.variant == NonlocalGpuVariant::CompensatedBrokenEft) {
        append_u64(bytes, result.work.compensated_input_components);
        append_u64(bytes, result.work.compensated_decomposition_components);
        append_u64(bytes, result.work.compensated_reconstruction_components);
        append_u64(bytes, result.work.compensated_canonical_checks);
        append_u64(bytes, result.work.compensated_difference_components);
        append_u64(bytes, result.work.compensated_inertia_components);
        append_u64(bytes, result.work.compensated_trial_eft_components);
        append_u64(bytes, result.work.compensated_transaction_components);
        append_u64(bytes, result.work.compensated_publish_components);
    }
    return sha256_hex(bytes);
}

std::string step_semantic_root(const NonlocalGpuProfile& profile,
    const std::string& input_root,
    const NonlocalGpuStepResult& result) {
    std::string bytes = "nextengine.nonlocal.ncgp1.step-result.v1\0";
    append_string(bytes, profile_semantic_root(profile));
    append_string(bytes, input_root);
    append_string(bytes, step_work_semantic_root(profile, result));
    append_u32(bytes, static_cast<std::uint32_t>(result.failure));
    append_u32(bytes, result.hvp_budget);
    append_u32(bytes, result.hvp_used);
    append_u32(bytes, result.outer_trials);
    append_u32(bytes, result.active_pressure_centers);
    append_u32(bytes, static_cast<std::uint32_t>(result.solver_profile));
    append_u32(bytes, static_cast<std::uint32_t>(result.variant));
    append_f64(bytes, result.initial_energy);
    append_f64(bytes, result.final_energy);
    append_f64(bytes, result.gradient_norm);
    append_f64(bytes, result.scaled_displacement_residual);
    append_f64(bytes, result.maximum_penetration_m);
    append_f64(bytes, result.boundary_impulse.x);
    append_f64(bytes, result.boundary_impulse.y);
    append_f64(bytes, result.boundary_impulse.z);
    append_u64(bytes, result.boundary_face_mask_xor);
    if (result.variant == NonlocalGpuVariant::CompensatedStateF32
        || result.variant == NonlocalGpuVariant::CompensatedOmitLow
        || result.variant == NonlocalGpuVariant::CompensatedBrokenEft) {
        append_u64(bytes, result.active_pressure_ids.size());
        for (const std::uint32_t id : result.active_pressure_ids) {
            append_u32(bytes, id);
        }
    }
    append_u64(bytes, result.state.size());
    for (const NonlocalGpuSample& sample : result.state) {
        append_u32(bytes, sample.sample_id);
        append_f32(bytes, static_cast<float>(sample.current.x));
        append_f32(bytes, static_cast<float>(sample.current.y));
        append_f32(bytes, static_cast<float>(sample.current.z));
        append_f32(bytes, static_cast<float>(sample.velocity.x));
        append_f32(bytes, static_cast<float>(sample.velocity.y));
        append_f32(bytes, static_cast<float>(sample.velocity.z));
    }
    return sha256_hex(bytes);
}

NonlocalGpuEvaluationResult evaluate_reference(
    const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& input_samples,
    const std::vector<NonlocalGpuGhost>& ghosts,
    const std::vector<Vec3d>* input_direction,
    NonlocalGpuVariant variant) {
    NonlocalGpuEvaluationResult result;
    if (!valid_profile(profile) || input_samples.empty()
        || (input_direction != nullptr
            && input_direction->size() != input_samples.size())) {
        result.failure = NonlocalGpuFailure::InvalidState;
        return result;
    }
    std::vector<std::size_t> order(input_samples.size());
    for (std::size_t index = 0U; index < order.size(); ++index) order[index] = index;
    std::sort(order.begin(), order.end(), [&](std::size_t lhs, std::size_t rhs) {
        return input_samples[lhs].sample_id < input_samples[rhs].sample_id;
    });
    std::vector<NonlocalGpuSample> samples;
    std::vector<WideVec3> direction;
    samples.reserve(input_samples.size());
    direction.reserve(input_samples.size());
    for (const std::size_t index : order) {
        samples.push_back(input_samples[index]);
        if (input_direction != nullptr) direction.push_back(wide((*input_direction)[index]));
    }
    std::vector<NonlocalGpuSample> reference_samples = samples;
    for (auto& sample : reference_samples) sample.current = sample.reference;
    const auto current_graph = build_reference_graph(profile, samples, ghosts,
        variant == NonlocalGpuVariant::StrictRadius);
    const auto reference_graph = build_reference_graph(
        profile, reference_samples, ghosts, false);
    if (current_graph.failure != NonlocalGpuFailure::None
        || reference_graph.failure != NonlocalGpuFailure::None) {
        result.failure = current_graph.failure != NonlocalGpuFailure::None
            ? current_graph.failure : reference_graph.failure;
        return result;
    }
    const std::size_t count = samples.size();
    std::unordered_map<std::uint32_t, std::size_t> dynamic_index;
    std::unordered_map<std::uint32_t, WideVec3> ghost_position;
    dynamic_index.reserve(count);
    ghost_position.reserve(ghosts.size());
    for (std::size_t index = 0U; index < count; ++index) {
        dynamic_index.emplace(samples[index].sample_id, index);
    }
    for (const auto& ghost : ghosts) {
        ghost_position.emplace(ghost.sample_id, wide(ghost.position));
    }
    const auto current_position = [&](std::uint32_t id) {
        const auto dynamic = dynamic_index.find(id);
        if (dynamic != dynamic_index.end()) return wide(samples[dynamic->second].current);
        const auto ghost = ghost_position.find(id);
        if (ghost == ghost_position.end()) throw std::logic_error("missing graph point");
        return ghost->second;
    };
    const long double mass = profile.mass;
    const long double dt = profile.dt;
    const long double rho0 = profile.rest_density;
    const long double kappa = profile.kappa;
    const long double lambda = profile.lambda;
    const long double mu = profile.mu;
    const long double gamma = profile.gamma;
    const long double inertia_scale = mass / (dt * dt);
    std::vector<long double> density(count, 0.0L);
    std::vector<long double> excess(count, 0.0L);
    std::vector<WideVec3> gradient(count);
    std::vector<WideVec3> hvp(count);
    std::vector<long double> pressure_q(count, 0.0L);
    long double total_energy = 0.0L;
    for (std::size_t row = 0U; row < count; ++row) {
        const WideVec3 owner = wide(samples[row].current);
        for (std::uint32_t slot = current_graph.offsets[row];
             slot < current_graph.offsets[row + 1U]; ++slot) {
            const WideVec3 candidate = current_position(current_graph.neighbor_ids[slot]);
            density[row] += mass * kernel(norm(subtract(owner, candidate)),
                profile.horizon).value;
            ++result.work.density_kernel_evaluations;
        }
        excess[row] = std::max(density[row] / rho0 - 1.0L, 0.0L);
        if (excess[row] > 0.0L) ++result.active_pressure_centers;
    }
    if (input_direction != nullptr) {
        for (std::size_t row = 0U; row < count; ++row) {
            if (!(excess[row] > 0.0L)) continue;
            const WideVec3 owner = wide(samples[row].current);
            for (std::uint32_t slot = current_graph.offsets[row];
                 slot < current_graph.offsets[row + 1U]; ++slot) {
                const auto id = current_graph.neighbor_ids[slot];
                const auto dynamic = dynamic_index.find(id);
                if (dynamic != dynamic_index.end() && dynamic->second == row) continue;
                const WideVec3 difference = subtract(owner, current_position(id));
                const long double radius = norm(difference);
                if (!(radius > 0.0L)) continue;
                const WideVec3 normal = scale(difference, 1.0L / radius);
                const WideVec3 neighbor_direction = dynamic != dynamic_index.end()
                    ? direction[dynamic->second] : WideVec3{};
                pressure_q[row] += mass / rho0
                    * kernel(radius, profile.horizon).first
                    * dot(normal, subtract(direction[row], neighbor_direction));
            }
        }
    }
    for (std::size_t row = 0U; row < count; ++row) {
        const WideVec3 x = wide(samples[row].reference);
        const WideVec3 y = wide(samples[row].current);
        const WideVec3 velocity = wide(samples[row].velocity);
        const WideVec3 predicted = add(x, add(scale(velocity, dt),
            scale(wide(profile.gravity), dt * dt)));
        const WideVec3 inertial_delta = subtract(y, predicted);
        gradient[row] = scale(inertial_delta, inertia_scale);
        total_energy += 0.5L * inertia_scale * dot(inertial_delta, inertial_delta)
            + 0.5L * kappa * excess[row] * excess[row];
        if (input_direction != nullptr) {
            hvp[row] = scale(direction[row], inertia_scale);
        }
        for (std::uint32_t slot = current_graph.offsets[row];
             slot < current_graph.offsets[row + 1U]; ++slot) {
            const std::uint32_t id = current_graph.neighbor_ids[slot];
            const auto dynamic = dynamic_index.find(id);
            if (dynamic != dynamic_index.end() && dynamic->second == row) continue;
            const WideVec3 difference = subtract(y, current_position(id));
            const long double radius = norm(difference);
            if (!(radius > 0.0L)) continue;
            const WideVec3 normal = scale(difference, 1.0L / radius);
            const WideKernel values = kernel(radius, profile.horizon);
            const long double neighbor_excess = dynamic != dynamic_index.end()
                    && variant != NonlocalGpuVariant::OwnerOnlyPressure
                ? excess[dynamic->second] : 0.0L;
            gradient[row] = add(gradient[row], scale(normal,
                kappa * mass / rho0 * (excess[row] + neighbor_excess)
                    * values.first));
            ++result.work.gradient_pair_visits;
            if (dynamic != dynamic_index.end()) {
                long double potential = 0.0L;
                long double force = 0.0L;
                long double derivative = 0.0L;
                surface(radius, profile.spacing, potential, force, derivative);
                const long double surface_sign = variant
                        == NonlocalGpuVariant::WrongSurfaceSign ? -1.0L : 1.0L;
                gradient[row] = add(gradient[row], scale(normal,
                    surface_sign * 2.0L * gamma * mass * mass * force));
                ++result.work.gradient_pair_visits;
                if (dynamic->second > row) {
                    total_energy += 2.0L * gamma * mass * mass * potential;
                    ++result.work.energy_pair_visits;
                }
            }
            if (input_direction != nullptr) {
                const WideVec3 neighbor_direction = dynamic != dynamic_index.end()
                    ? direction[dynamic->second] : WideVec3{};
                const WideVec3 dv = subtract(direction[row], neighbor_direction);
                const WideVec3 b = scale(normal, mass / rho0 * values.first);
                const long double neighbor_q = dynamic != dynamic_index.end()
                        && variant != NonlocalGpuVariant::OwnerOnlyPressure
                    ? pressure_q[dynamic->second] : 0.0L;
                hvp[row] = add(hvp[row], scale(b,
                    kappa * (pressure_q[row] + neighbor_q)));
                hvp[row] = add(hvp[row], scale(radial_apply(normal,
                    values.second, values.first / radius, dv),
                    kappa * mass / rho0 * (excess[row] + neighbor_excess)));
                if (dynamic != dynamic_index.end()) {
                    long double potential = 0.0L;
                    long double force = 0.0L;
                    long double derivative = 0.0L;
                    surface(radius, profile.spacing, potential, force, derivative);
                    const long double surface_sign = variant
                            == NonlocalGpuVariant::WrongSurfaceSign ? -1.0L : 1.0L;
                    hvp[row] = add(hvp[row], scale(radial_apply(normal,
                        derivative, force / radius, dv),
                        surface_sign * 2.0L * gamma * mass * mass));
                }
                ++result.work.hvp_pair_visits;
            }
        }

        const NonlocalGpuGraphResult& viscosity_graph =
            variant == NonlocalGpuVariant::CurrentReferenceSwap
            ? current_graph : reference_graph;
        for (std::uint32_t slot = viscosity_graph.offsets[row];
             slot < viscosity_graph.offsets[row + 1U]; ++slot) {
            const auto dynamic = dynamic_index.find(viscosity_graph.neighbor_ids[slot]);
            if (dynamic == dynamic_index.end() || dynamic->second == row) continue;
            const WideVec3 reference_delta = subtract(x,
                wide(samples[dynamic->second].reference));
            const long double radius = norm(reference_delta);
            if (!(radius > 0.0L)) continue;
            const WideVec3 normal = scale(reference_delta, 1.0L / radius);
            const WideVec3 delta = subtract(subtract(y,
                wide(samples[dynamic->second].current)), reference_delta);
            const long double normal_delta = dot(normal, delta);
            const WideVec3 tangent_delta = subtract(delta,
                scale(normal, normal_delta));
            long double factor = mass * (-kernel(radius, profile.horizon).first)
                / (rho0 * dt);
            const long double force_scale = variant
                    == NonlocalGpuVariant::HalfViscosity ? 0.5L : 1.0L;
            gradient[row] = add(gradient[row], scale(add(
                scale(tangent_delta, 2.0L * mu),
                scale(normal, lambda * normal_delta)), factor * force_scale));
            ++result.work.gradient_pair_visits;
            if (dynamic->second > row) {
                total_energy += factor * (mu * dot(tangent_delta, tangent_delta)
                    + 0.5L * lambda * normal_delta * normal_delta);
                ++result.work.energy_pair_visits;
            }
            if (input_direction != nullptr) {
                const long double tangential = factor * 2.0L * mu * force_scale;
                const long double radial = factor * lambda * force_scale;
                hvp[row] = add(hvp[row], radial_apply(normal, radial,
                    tangential, subtract(direction[row], direction[dynamic->second])));
                ++result.work.hvp_pair_visits;
            }
        }
        if (input_direction != nullptr
            && variant == NonlocalGpuVariant::HvpSignFlip) {
            hvp[row] = scale(hvp[row], -1.0L);
        }
    }
    long double gradient_norm_squared = 0.0L;
    result.gradient.reserve(count);
    result.density.reserve(count);
    if (input_direction != nullptr) result.hvp.reserve(count);
    for (std::size_t index = 0U; index < count; ++index) {
        gradient_norm_squared += dot(gradient[index], gradient[index]);
        result.gradient.push_back({static_cast<double>(gradient[index].x),
            static_cast<double>(gradient[index].y),
            static_cast<double>(gradient[index].z)});
        result.density.push_back(static_cast<double>(density[index]));
        if (input_direction != nullptr) {
            result.hvp.push_back({static_cast<double>(hvp[index].x),
                static_cast<double>(hvp[index].y),
                static_cast<double>(hvp[index].z)});
        }
    }
    result.energy = static_cast<double>(total_energy);
    result.gradient_norm = static_cast<double>(std::sqrt(gradient_norm_squared));
    result.work.graph_builds = 2U;
    result.work.hvp_applications = input_direction != nullptr ? 1U : 0U;
    result.work.reduction_values = count * 2U;
    result.failure = NonlocalGpuFailure::None;
    return result;
}

} // namespace nextengine::nonlocal::gpu_full_step
