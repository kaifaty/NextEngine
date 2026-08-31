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
#include <tuple>
#include <utility>
#include <vector>

#ifndef NCGP8_CONTRACT_ROOT
#define NCGP8_CONTRACT_ROOT "unconfigured"
#endif
#ifndef NCGP8_SOURCE_ROOT
#define NCGP8_SOURCE_ROOT "unconfigured"
#endif
#ifndef NCGP8_SOURCE_COMMIT
#define NCGP8_SOURCE_COMMIT "unconfigured"
#endif
#ifndef NCGP8_SOURCE_TREE
#define NCGP8_SOURCE_TREE "unconfigured"
#endif
#ifndef NCGP8_COMPILER_FLAGS
#define NCGP8_COMPILER_FLAGS "unconfigured"
#endif

namespace {

using namespace nextengine::nonlocal::gpu_full_step;

constexpr std::size_t kWitnessSamples = 4000U;
constexpr std::uint32_t kWitnessStep = 112U;
constexpr std::uint32_t kWitnessBudget = 128U;
constexpr std::uint32_t kVerticalAxisZ = 2U;
constexpr std::uint32_t kMinimumMaterialComponentPixels = 4U;
constexpr double kSphereRadius = 0.025;
constexpr double kPixelPitch = 0.0125;
std::uint64_t bits(double value) {
    std::uint64_t result = 0U;
    static_assert(sizeof(result) == sizeof(value));
    std::memcpy(&result, &value, sizeof(value));
    return result;
}

double norm(Vec3d value) {
    return std::sqrt(value.x * value.x + value.y * value.y
        + value.z * value.z);
}

std::string file_root(const std::string& path) {
    std::ifstream stream(path, std::ios::binary);
    if (!stream) return {};
    std::ostringstream bytes;
    bytes << stream.rdbuf();
    return nextengine::nonlocal::sha256_hex(bytes.str());
}

std::string binary_root() { return file_root("/proc/self/exe"); }

struct VisibleSurfaceWork {
    std::uint64_t samples_validated = 0U;
    std::uint64_t candidate_pixel_tests = 0U;
    std::uint64_t depth_records_emitted = 0U;
    std::uint64_t depth_records_sorted = 0U;
    std::uint64_t depth_records_reduced = 0U;
    std::uint64_t wet_pixels = 0U;
    std::uint64_t flood_pixels = 0U;
    std::uint64_t flood_neighbor_tests = 0U;
    std::uint64_t components_reduced = 0U;
    std::uint64_t root_derivations = 0U;
};

struct VisibleSurfaceImage {
    bool valid = false;
    std::uint32_t width = 0U;
    std::uint32_t height = 0U;
    std::uint32_t vertical_axis = kVerticalAxisZ;
    double pixel_pitch = 0.0;
    double sphere_radius = 0.0;
    std::size_t expected_samples = 0U;
    double expected_mass = 0.0;
    std::vector<std::uint8_t> wet;
    std::vector<double> depth;
    std::vector<std::uint32_t> component_label;
    std::vector<std::uint32_t> component_sizes;
    std::uint32_t material_component_count = 0U;
    double largest_component_fraction = 0.0;
    double satellite_area_fraction = 0.0;
    VisibleSurfaceWork work;
    std::string mask_root;
    std::string depth_root;
    std::string component_root;
    std::string work_root;
    std::string image_root;
};

struct VisibleComparisonWork {
    std::uint64_t image_validations = 0U;
    std::uint64_t validation_pixel_reads = 0U;
    std::uint64_t validation_component_reads = 0U;
    std::uint64_t pixel_comparisons = 0U;
    std::uint64_t depth_error_records = 0U;
    std::uint64_t depth_error_records_sorted = 0U;
    std::uint64_t quantile_reads = 0U;
    std::uint64_t topology_comparisons = 0U;
    std::uint64_t root_derivations = 0U;
};

struct VisibleSurfaceComparison {
    bool valid = false;
    double silhouette_symmetric_difference =
        std::numeric_limits<double>::infinity();
    double depth_rmse = std::numeric_limits<double>::infinity();
    double depth_p95 = std::numeric_limits<double>::infinity();
    double depth_p99 = std::numeric_limits<double>::infinity();
    double depth_maximum = std::numeric_limits<double>::infinity();
    std::uint64_t common_wet_pixels = 0U;
    std::uint64_t union_wet_pixels = 0U;
    std::uint32_t cpu_material_components = 0U;
    std::uint32_t gpu_material_components = 0U;
    double largest_component_fraction_difference =
        std::numeric_limits<double>::infinity();
    double cpu_satellite_area_fraction =
        std::numeric_limits<double>::infinity();
    double gpu_satellite_area_fraction =
        std::numeric_limits<double>::infinity();
    VisibleComparisonWork work;
    std::string work_root;
    std::string metrics_root;
};

struct DepthContribution {
    std::uint32_t pixel = 0U;
    std::array<std::uint64_t, 4> order{};
    double depth = 0.0;
};

std::string surface_work_root(const VisibleSurfaceWork& work) {
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp8.surface-work.v1\n"
             << work.samples_validated << ':'
             << work.candidate_pixel_tests << ':'
             << work.depth_records_emitted << ':'
             << work.depth_records_sorted << ':'
             << work.depth_records_reduced << ':' << work.wet_pixels << ':'
             << work.flood_pixels << ':' << work.flood_neighbor_tests << ':'
             << work.components_reduced << ':' << work.root_derivations
             << '\n';
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string surface_mask_root(const VisibleSurfaceImage& image) {
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp8.surface-mask.v1\n"
             << image.width << ':' << image.height << '\n';
    for (std::uint8_t value : image.wet) {
        material << static_cast<unsigned>(value);
    }
    material << '\n';
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string surface_depth_root(const VisibleSurfaceImage& image) {
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp8.surface-depth.v1\n" << std::hex;
    for (std::size_t pixel = 0U; pixel < image.wet.size(); ++pixel) {
        if (image.wet[pixel] != 0U) {
            material << pixel << ':' << bits(image.depth[pixel]) << '\n';
        }
    }
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string surface_component_root(const VisibleSurfaceImage& image) {
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp8.surface-components.v1\n"
             << image.material_component_count << ':' << std::hex
             << bits(image.largest_component_fraction) << ':'
             << bits(image.satellite_area_fraction) << std::dec << '\n';
    for (std::uint32_t label : image.component_label) material << label << ',';
    material << '\n';
    for (std::uint32_t size : image.component_sizes) material << size << ',';
    material << '\n';
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string surface_image_root(const VisibleSurfaceImage& image) {
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp8.surface-image.v1\n"
             << image.valid << ':' << image.width << ':' << image.height << ':'
             << image.vertical_axis << ':' << std::hex
             << bits(image.pixel_pitch) << ':' << bits(image.sphere_radius)
             << ':' << std::dec << image.expected_samples << ':' << std::hex
             << bits(image.expected_mass) << std::dec << '\n'
             << image.mask_root << ':' << image.depth_root << ':'
             << image.component_root << ':' << image.work_root << '\n';
    return nextengine::nonlocal::sha256_hex(material.str());
}

void seal_image(VisibleSurfaceImage& image) {
    image.work.root_derivations = 5U;
    image.valid = true;
    image.mask_root = surface_mask_root(image);
    image.depth_root = surface_depth_root(image);
    image.component_root = surface_component_root(image);
    image.work_root = surface_work_root(image.work);
    image.image_root = surface_image_root(image);
}

bool vertical_axis_valid(
    const NonlocalGpuProfile& profile, std::uint32_t axis) {
    return axis == kVerticalAxisZ && profile.gravity.x == 0.0
        && profile.gravity.y == 0.0 && profile.gravity.z < 0.0
        && std::isfinite(profile.gravity.z);
}

std::pair<double, double> horizontal_coordinates(
    const Vec3d& value, std::uint32_t axis) {
    if (axis == 2U) return {value.x, value.y};
    if (axis == 1U) return {value.x, value.z};
    return {value.y, value.z};
}

double vertical_coordinate(const Vec3d& value, std::uint32_t axis) {
    if (axis == 2U) return value.z;
    if (axis == 1U) return value.y;
    return value.x;
}

std::pair<double, double> horizontal_extents(
    const NonlocalGpuProfile& profile, std::uint32_t axis) {
    if (axis == 2U) return {profile.basin_extent.x, profile.basin_extent.y};
    if (axis == 1U) return {profile.basin_extent.x, profile.basin_extent.z};
    return {profile.basin_extent.y, profile.basin_extent.z};
}

bool finite_sample(const NonlocalGpuSample& sample) {
    const std::array<double, 9> values{sample.reference.x, sample.reference.y,
        sample.reference.z, sample.current.x, sample.current.y,
        sample.current.z, sample.velocity.x, sample.velocity.y,
        sample.velocity.z};
    return std::all_of(values.begin(), values.end(),
        [](double value) { return std::isfinite(value); });
}

void derive_components(VisibleSurfaceImage& image) {
    const std::uint32_t absent = std::numeric_limits<std::uint32_t>::max();
    image.component_label.assign(image.wet.size(), absent);
    image.component_sizes.clear();
    std::vector<std::uint32_t> pending;
    for (std::uint32_t seed = 0U; seed < image.wet.size(); ++seed) {
        if (image.wet[seed] == 0U || image.component_label[seed] != absent) {
            continue;
        }
        const std::uint32_t label =
            static_cast<std::uint32_t>(image.component_sizes.size());
        image.component_sizes.push_back(0U);
        pending.push_back(seed);
        image.component_label[seed] = label;
        while (!pending.empty()) {
            const std::uint32_t pixel = pending.back();
            pending.pop_back();
            ++image.component_sizes[label];
            ++image.work.flood_pixels;
            const std::int32_t x = static_cast<std::int32_t>(
                pixel % image.width);
            const std::int32_t y = static_cast<std::int32_t>(
                pixel / image.width);
            for (std::int32_t dy = -1; dy <= 1; ++dy) {
                for (std::int32_t dx = -1; dx <= 1; ++dx) {
                    if (dx == 0 && dy == 0) continue;
                    ++image.work.flood_neighbor_tests;
                    const std::int32_t nx = x + dx;
                    const std::int32_t ny = y + dy;
                    if (nx < 0 || ny < 0
                        || nx >= static_cast<std::int32_t>(image.width)
                        || ny >= static_cast<std::int32_t>(image.height)) {
                        continue;
                    }
                    const std::uint32_t neighbor =
                        static_cast<std::uint32_t>(ny) * image.width
                        + static_cast<std::uint32_t>(nx);
                    if (image.wet[neighbor] != 0U
                        && image.component_label[neighbor] == absent) {
                        image.component_label[neighbor] = label;
                        pending.push_back(neighbor);
                    }
                }
            }
        }
    }
    image.work.components_reduced = image.component_sizes.size();
    image.material_component_count = 0U;
    std::uint32_t largest = 0U;
    for (std::uint32_t size : image.component_sizes) {
        if (size >= kMinimumMaterialComponentPixels) {
            ++image.material_component_count;
        }
        largest = std::max(largest, size);
    }
    if (image.work.wet_pixels != 0U) {
        image.largest_component_fraction = static_cast<double>(largest)
            / static_cast<double>(image.work.wet_pixels);
        image.satellite_area_fraction =
            1.0 - image.largest_component_fraction;
    }
}

VisibleSurfaceImage build_visible_surface(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& state, std::size_t expected_samples,
    double expected_mass, double pixel_pitch = kPixelPitch,
    double sphere_radius = kSphereRadius, std::uint32_t vertical_axis = 2U,
    bool strict_radius = false) {
    VisibleSurfaceImage image;
    image.vertical_axis = vertical_axis;
    image.pixel_pitch = pixel_pitch;
    image.sphere_radius = sphere_radius;
    image.expected_samples = expected_samples;
    image.expected_mass = expected_mass;
    if (!vertical_axis_valid(profile, vertical_axis)
        || !(pixel_pitch > 0.0) || !(sphere_radius > 0.0)
        || !std::isfinite(pixel_pitch) || !std::isfinite(sphere_radius)
        || state.size() != expected_samples
        || profile.mass * static_cast<double>(state.size()) != expected_mass) {
        return image;
    }
    const auto [extent_u, extent_v] = horizontal_extents(profile, vertical_axis);
    const double cells_u = extent_u / pixel_pitch;
    const double cells_v = extent_v / pixel_pitch;
    image.width = static_cast<std::uint32_t>(std::llround(cells_u));
    image.height = static_cast<std::uint32_t>(std::llround(cells_v));
    if (image.width == 0U || image.height == 0U
        || std::abs(cells_u - image.width) > 1.0e-12
        || std::abs(cells_v - image.height) > 1.0e-12) {
        return image;
    }
    const std::size_t pixels =
        static_cast<std::size_t>(image.width) * image.height;
    image.wet.assign(pixels, 0U);
    image.depth.assign(pixels, 0.0);
    std::vector<DepthContribution> records;
    records.reserve(state.size() * 20U);
    const long double radius_squared =
        static_cast<long double>(sphere_radius) * sphere_radius;
    for (const NonlocalGpuSample& sample : state) {
        if (!finite_sample(sample) || sample.current.x < 0.0
            || sample.current.y < 0.0 || sample.current.z < 0.0
            || sample.current.x > profile.basin_extent.x
            || sample.current.y > profile.basin_extent.y
            || sample.current.z > profile.basin_extent.z) {
            return {};
        }
        ++image.work.samples_validated;
        const auto [u, v] = horizontal_coordinates(sample.current, vertical_axis);
        const double height = vertical_coordinate(sample.current, vertical_axis);
        const auto minimum_index = [pixel_pitch, sphere_radius](double value) {
            return static_cast<std::int64_t>(std::ceil(
                (value - sphere_radius) / pixel_pitch - 0.5));
        };
        const auto maximum_index = [pixel_pitch, sphere_radius](double value) {
            return static_cast<std::int64_t>(std::floor(
                (value + sphere_radius) / pixel_pitch - 0.5));
        };
        const std::int64_t min_x = std::max<std::int64_t>(0, minimum_index(u));
        const std::int64_t min_y = std::max<std::int64_t>(0, minimum_index(v));
        const std::int64_t max_x = std::min<std::int64_t>(
            image.width - 1U, maximum_index(u));
        const std::int64_t max_y = std::min<std::int64_t>(
            image.height - 1U, maximum_index(v));
        for (std::int64_t iy = min_y; iy <= max_y; ++iy) {
            for (std::int64_t ix = min_x; ix <= max_x; ++ix) {
                ++image.work.candidate_pixel_tests;
                const long double pixel_u =
                    (static_cast<long double>(ix) + 0.5L) * pixel_pitch;
                const long double pixel_v =
                    (static_cast<long double>(iy) + 0.5L) * pixel_pitch;
                const long double du = static_cast<long double>(u) - pixel_u;
                const long double dv = static_cast<long double>(v) - pixel_v;
                const long double distance_squared = du * du + dv * dv;
                const bool covered = strict_radius
                    ? distance_squared < radius_squared
                    : distance_squared <= radius_squared;
                if (!covered) continue;
                const long double radial =
                    std::sqrt(std::max(0.0L, radius_squared - distance_squared));
                const double depth = static_cast<double>(
                    static_cast<long double>(height) + radial);
                if (!std::isfinite(depth)) return {};
                DepthContribution record;
                record.pixel = static_cast<std::uint32_t>(iy) * image.width
                    + static_cast<std::uint32_t>(ix);
                record.order = {bits(depth), bits(sample.current.x),
                    bits(sample.current.y), bits(sample.current.z)};
                record.depth = depth;
                records.push_back(record);
            }
        }
    }
    image.work.depth_records_emitted = records.size();
    std::sort(records.begin(), records.end(), [](const auto& lhs, const auto& rhs) {
        return std::tie(lhs.pixel, lhs.order)
            < std::tie(rhs.pixel, rhs.order);
    });
    image.work.depth_records_sorted = records.size();
    std::size_t cursor = 0U;
    while (cursor < records.size()) {
        const std::uint32_t pixel = records[cursor].pixel;
        double maximum = -std::numeric_limits<double>::infinity();
        while (cursor < records.size() && records[cursor].pixel == pixel) {
            maximum = std::max(maximum, records[cursor].depth);
            ++cursor;
            ++image.work.depth_records_reduced;
        }
        image.wet[pixel] = 1U;
        image.depth[pixel] = maximum;
        ++image.work.wet_pixels;
    }
    if (image.work.wet_pixels == 0U) return {};
    derive_components(image);
    seal_image(image);
    return image;
}

bool image_consistent(const VisibleSurfaceImage& image,
    VisibleComparisonWork* work = nullptr) {
    if (!image.valid || image.width == 0U || image.height == 0U
        || image.wet.size() != image.depth.size()
        || image.wet.size() != image.component_label.size()
        || image.wet.size() != static_cast<std::size_t>(image.width) * image.height
        || image.work.wet_pixels
            != static_cast<std::uint64_t>(std::count(
                image.wet.begin(), image.wet.end(), std::uint8_t{1U}))) {
        return false;
    }
    if (work != nullptr) {
        ++work->image_validations;
        work->validation_pixel_reads += image.wet.size();
        work->validation_component_reads += image.component_sizes.size();
    }
    VisibleSurfaceImage derived = image;
    derived.work.flood_pixels = 0U;
    derived.work.flood_neighbor_tests = 0U;
    derived.work.components_reduced = 0U;
    derive_components(derived);
    return derived.component_label == image.component_label
        && derived.component_sizes == image.component_sizes
        && derived.material_component_count == image.material_component_count
        && bits(derived.largest_component_fraction)
            == bits(image.largest_component_fraction)
        && bits(derived.satellite_area_fraction)
            == bits(image.satellite_area_fraction)
        && surface_mask_root(image) == image.mask_root
        && surface_depth_root(image) == image.depth_root
        && surface_component_root(image) == image.component_root
        && surface_work_root(image.work) == image.work_root
        && surface_image_root(image) == image.image_root;
}

std::size_t nearest_rank_index(
    std::size_t size, std::size_t numerator, std::size_t denominator) {
    return (numerator * size + denominator - 1U) / denominator - 1U;
}

std::string comparison_work_root(const VisibleComparisonWork& work) {
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp8.comparison-work.v1\n"
             << work.image_validations << ':' << work.validation_pixel_reads
             << ':' << work.validation_component_reads << ':'
             << work.pixel_comparisons << ':' << work.depth_error_records
             << ':' << work.depth_error_records_sorted << ':'
             << work.quantile_reads << ':' << work.topology_comparisons << ':'
             << work.root_derivations << '\n';
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string comparison_metrics_root(
    const VisibleSurfaceComparison& comparison) {
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp8.comparison.v1\n"
             << comparison.valid << ':' << std::hex
             << bits(comparison.silhouette_symmetric_difference) << ':'
             << bits(comparison.depth_rmse) << ':' << bits(comparison.depth_p95)
             << ':' << bits(comparison.depth_p99) << ':'
             << bits(comparison.depth_maximum) << ':' << std::dec
             << comparison.common_wet_pixels << ':'
             << comparison.union_wet_pixels << ':'
             << comparison.cpu_material_components << ':'
             << comparison.gpu_material_components << ':' << std::hex
             << bits(comparison.largest_component_fraction_difference) << ':'
             << bits(comparison.cpu_satellite_area_fraction) << ':'
             << bits(comparison.gpu_satellite_area_fraction) << std::dec
             << '\n' << comparison.work_root << '\n';
    return nextengine::nonlocal::sha256_hex(material.str());
}

VisibleSurfaceComparison compare_visible_surfaces(
    const VisibleSurfaceImage& gpu, const VisibleSurfaceImage& cpu) {
    VisibleSurfaceComparison result;
    if (gpu.width != cpu.width || gpu.height != cpu.height
        || bits(gpu.pixel_pitch) != bits(cpu.pixel_pitch)
        || bits(gpu.sphere_radius) != bits(cpu.sphere_radius)
        || !image_consistent(gpu, &result.work)
        || !image_consistent(cpu, &result.work)) {
        return result;
    }
    std::uint64_t different = 0U;
    long double squared = 0.0L;
    std::vector<double> errors;
    for (std::size_t pixel = 0U; pixel < gpu.wet.size(); ++pixel) {
        ++result.work.pixel_comparisons;
        const bool gpu_wet = gpu.wet[pixel] != 0U;
        const bool cpu_wet = cpu.wet[pixel] != 0U;
        if (gpu_wet || cpu_wet) ++result.union_wet_pixels;
        if (gpu_wet != cpu_wet) ++different;
        if (gpu_wet && cpu_wet) {
            ++result.common_wet_pixels;
            const double error = std::abs(gpu.depth[pixel] - cpu.depth[pixel]);
            errors.push_back(error);
            squared += static_cast<long double>(error) * error;
            ++result.work.depth_error_records;
        }
    }
    if (result.union_wet_pixels == 0U || errors.empty()) return result;
    std::sort(errors.begin(), errors.end());
    result.work.depth_error_records_sorted = errors.size();
    result.silhouette_symmetric_difference = static_cast<double>(different)
        / static_cast<double>(result.union_wet_pixels);
    result.depth_rmse = std::sqrt(static_cast<double>(
        squared / static_cast<long double>(errors.size())));
    result.depth_p95 = errors[nearest_rank_index(errors.size(), 95U, 100U)];
    result.depth_p99 = errors[nearest_rank_index(errors.size(), 99U, 100U)];
    result.depth_maximum = errors.back();
    result.work.quantile_reads = 3U;
    result.cpu_material_components = cpu.material_component_count;
    result.gpu_material_components = gpu.material_component_count;
    result.largest_component_fraction_difference = std::abs(
        gpu.largest_component_fraction - cpu.largest_component_fraction);
    result.cpu_satellite_area_fraction = cpu.satellite_area_fraction;
    result.gpu_satellite_area_fraction = gpu.satellite_area_fraction;
    result.work.topology_comparisons = 4U;
    result.work.root_derivations = 2U;
    result.valid = true;
    result.work_root = comparison_work_root(result.work);
    result.metrics_root = comparison_metrics_root(result);
    return result;
}

bool h8a_band(const VisibleSurfaceComparison& value) {
    return value.valid && value.silhouette_symmetric_difference <= 0.01
        && value.depth_rmse <= 0.00625 && value.depth_p95 <= 0.0125
        && value.depth_p99 <= 0.025
        && value.cpu_material_components == value.gpu_material_components
        && value.largest_component_fraction_difference <= 0.01
        && value.cpu_satellite_area_fraction <= 0.01
        && value.gpu_satellite_area_fraction <= 0.01;
}

bool h8b_band(const VisibleSurfaceComparison& value) {
    return value.valid
        && (value.silhouette_symmetric_difference >= 0.05
            || value.depth_rmse >= 0.025 || value.depth_p95 >= 0.05
            || value.depth_p99 >= 0.10
            || (value.cpu_material_components != value.gpu_material_components
                && (value.cpu_satellite_area_fraction >= 0.05
                    || value.gpu_satellite_area_fraction >= 0.05)));
}

std::string classification(
    const VisibleSurfaceComparison& value, bool apparatus_valid) {
    if (!apparatus_valid || !value.valid) return "APPARATUS_INCONCLUSIVE";
    if (h8a_band(value)) return "H8A_VISIBLE_SURFACE_SUPPORTED_BOUNDED";
    if (h8b_band(value)) return "H8B_VISIBLE_SURFACE_DIVERGENCE_SUPPORTED_BOUNDED";
    return "H8C_VISIBLE_SURFACE_INCONCLUSIVE";
}

std::string state_root(const std::vector<NonlocalGpuSample>& unordered_state,
    const std::vector<double>& unordered_density,
    const std::vector<std::uint32_t>& active_ids) {
    if (unordered_state.size() != unordered_density.size()) return {};
    struct Record {
        NonlocalGpuSample sample;
        double density = 0.0;
    };
    std::vector<Record> records;
    records.reserve(unordered_state.size());
    for (std::size_t index = 0U; index < unordered_state.size(); ++index) {
        records.push_back({unordered_state[index], unordered_density[index]});
    }
    std::sort(records.begin(), records.end(), [](const auto& lhs, const auto& rhs) {
        return lhs.sample.sample_id < rhs.sample.sample_id;
    });
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp7.state.v1\n" << std::hex;
    for (const auto& record : records) {
        material << record.sample.sample_id << ':'
                 << bits(record.sample.reference.x) << ':'
                 << bits(record.sample.reference.y) << ':'
                 << bits(record.sample.reference.z) << ':'
                 << bits(record.sample.current.x) << ':'
                 << bits(record.sample.current.y) << ':'
                 << bits(record.sample.current.z) << ':'
                 << bits(record.sample.velocity.x) << ':'
                 << bits(record.sample.velocity.y) << ':'
                 << bits(record.sample.velocity.z) << ':'
                 << bits(record.density) << '\n';
    }
    material << "active:";
    for (std::uint32_t id : active_ids) material << id << ',';
    material << '\n';
    return nextengine::nonlocal::sha256_hex(material.str());
}

Vec3d total_momentum(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& state) {
    std::array<long double, 3> value{};
    for (const auto& sample : state) {
        value[0] += static_cast<long double>(profile.mass) * sample.velocity.x;
        value[1] += static_cast<long double>(profile.mass) * sample.velocity.y;
        value[2] += static_cast<long double>(profile.mass) * sample.velocity.z;
    }
    return {static_cast<double>(value[0]), static_cast<double>(value[1]),
        static_cast<double>(value[2])};
}

std::pair<Vec3d, Vec3d> position_bounds(
    const std::vector<NonlocalGpuSample>& state) {
    Vec3d minimum{std::numeric_limits<double>::infinity(),
        std::numeric_limits<double>::infinity(),
        std::numeric_limits<double>::infinity()};
    Vec3d maximum{-std::numeric_limits<double>::infinity(),
        -std::numeric_limits<double>::infinity(),
        -std::numeric_limits<double>::infinity()};
    for (const auto& sample : state) {
        minimum.x = std::min(minimum.x, sample.current.x);
        minimum.y = std::min(minimum.y, sample.current.y);
        minimum.z = std::min(minimum.z, sample.current.z);
        maximum.x = std::max(maximum.x, sample.current.x);
        maximum.y = std::max(maximum.y, sample.current.y);
        maximum.z = std::max(maximum.z, sample.current.z);
    }
    return {minimum, maximum};
}

bool identity_valid(const std::string& executable_root) {
    return !executable_root.empty()
        && std::string(NCGP8_CONTRACT_ROOT) != "unconfigured"
        && std::string(NCGP8_SOURCE_ROOT) != "unconfigured"
        && std::string(NCGP8_SOURCE_COMMIT) != "unconfigured"
        && std::string(NCGP8_SOURCE_TREE) != "unconfigured"
        && std::string(NCGP8_COMPILER_FLAGS) != "unconfigured";
}

void emit_comparison(const VisibleSurfaceComparison& value) {
    std::cout << std::setprecision(17)
              << "{\"valid\":" << (value.valid ? "true" : "false")
              << ",\"silhouette_symmetric_difference\":"
              << value.silhouette_symmetric_difference
              << ",\"depth_rmse_m\":" << value.depth_rmse
              << ",\"depth_p95_m\":" << value.depth_p95
              << ",\"depth_p99_m\":" << value.depth_p99
              << ",\"depth_maximum_m\":" << value.depth_maximum
              << ",\"common_wet_pixels\":" << value.common_wet_pixels
              << ",\"union_wet_pixels\":" << value.union_wet_pixels
              << ",\"cpu_material_components\":"
              << value.cpu_material_components
              << ",\"gpu_material_components\":"
              << value.gpu_material_components
              << ",\"largest_component_fraction_difference\":"
              << value.largest_component_fraction_difference
              << ",\"cpu_satellite_area_fraction\":"
              << value.cpu_satellite_area_fraction
              << ",\"gpu_satellite_area_fraction\":"
              << value.gpu_satellite_area_fraction
              << ",\"work_root\":\"" << value.work_root
              << "\",\"metrics_root\":\"" << value.metrics_root << "\"}";
}

void reseal_image(VisibleSurfaceImage& image) {
    image.mask_root = surface_mask_root(image);
    image.depth_root = surface_depth_root(image);
    image.component_root = surface_component_root(image);
    image.work_root = surface_work_root(image.work);
    image.image_root = surface_image_root(image);
}

int run_self_test() {
    const NonlocalGpuProfile profile = nonlocal_water_corrected_profile();
    const auto state = make_lattice_state(profile, 4U, 4U, 4U, false, false);
    const double mass = profile.mass * static_cast<double>(state.size());
    const auto base = build_visible_surface(profile, state, state.size(), mass);
    const auto exact = compare_visible_surfaces(base, base);
    auto relabelled = state;
    for (std::size_t index = 0U; index < relabelled.size(); ++index) {
        relabelled[index].sample_id =
            state[(index + 17U) % state.size()].sample_id;
    }
    const auto relabelled_image = build_visible_surface(
        profile, relabelled, state.size(), mass);
    auto z_translated = state;
    for (auto& sample : z_translated) {
        sample.reference.z += 0.05;
        sample.current.z += 0.05;
    }
    const auto z_image = build_visible_surface(
        profile, z_translated, state.size(), mass);
    const auto z_comparison = compare_visible_surfaces(z_image, base);
    auto x_translated = state;
    for (auto& sample : x_translated) {
        sample.reference.x += 0.05;
        sample.current.x += 0.05;
    }
    const auto x_image = build_visible_surface(
        profile, x_translated, state.size(), mass);
    const auto x_comparison = compare_visible_surfaces(x_image, base);
    const auto wrong_axis = build_visible_surface(profile, z_translated,
        state.size(), mass, kPixelPitch, kSphereRadius, 1U);
    auto deleted = state;
    deleted.pop_back();
    const auto deleted_image = build_visible_surface(
        profile, deleted, state.size(), mass);

    auto sparse_tail = base;
    const auto first_wet = std::find(
        sparse_tail.wet.begin(), sparse_tail.wet.end(), std::uint8_t{1U});
    if (first_wet == sparse_tail.wet.end()) return 4;
    const std::size_t tail_pixel =
        static_cast<std::size_t>(first_wet - sparse_tail.wet.begin());
    sparse_tail.depth[tail_pixel] += 0.05;
    reseal_image(sparse_tail);
    const auto sparse_comparison = compare_visible_surfaces(sparse_tail, base);
    const auto sheet_comparison = z_comparison;

    std::vector<NonlocalGpuSample> tangent_state{
        {1U, {0.03125, 0.00625, 0.10}, {0.03125, 0.00625, 0.10}, {}}};
    const double tangent_mass = profile.mass;
    const auto inclusive = build_visible_surface(profile, tangent_state, 1U,
        tangent_mass, kPixelPitch, kSphereRadius, 2U, false);
    const auto strict = build_visible_surface(profile, tangent_state, 1U,
        tangent_mass, kPixelPitch, kSphereRadius, 2U, true);
    const auto changed_pitch = build_visible_surface(
        profile, state, state.size(), mass, 0.025, kSphereRadius);

    auto omitted_topology = base;
    omitted_topology.component_sizes.clear();
    const bool omitted_topology_rejected = !image_consistent(omitted_topology);
    auto depth_mutation = base;
    depth_mutation.depth[tail_pixel] += 0.001;
    const bool depth_mutation_rejected = !image_consistent(depth_mutation);
    auto component_mutation = base;
    component_mutation.component_label[tail_pixel] += 1U;
    const bool component_mutation_rejected = !image_consistent(component_mutation);
    auto work_mutation = base;
    ++work_mutation.work.depth_records_reduced;
    const bool work_mutation_rejected = !image_consistent(work_mutation);

    const bool exact_zero = exact.valid
        && exact.silhouette_symmetric_difference == 0.0
        && exact.depth_rmse == 0.0 && exact.depth_p95 == 0.0
        && exact.depth_p99 == 0.0 && exact.depth_maximum == 0.0;
    const bool relabel_preserved = base.image_root == relabelled_image.image_root;
    const bool z_translation_detected = z_comparison.valid
        && z_comparison.silhouette_symmetric_difference == 0.0
        && !h8a_band(z_comparison) && h8b_band(z_comparison);
    const bool x_translation_detected = x_comparison.valid
        && !h8a_band(x_comparison)
        && x_comparison.silhouette_symmetric_difference > 0.01;
    const bool wrong_axis_rejected = !wrong_axis.valid;
    const bool deletion_rejected = !deleted_image.valid;
    const bool sparse_tail_robust = h8a_band(sparse_comparison)
        && sparse_comparison.depth_maximum > sparse_comparison.depth_p99;
    const bool sheet_rejected = !h8a_band(sheet_comparison)
        && h8b_band(sheet_comparison);
    const bool strict_radius_rejected = inclusive.valid && strict.valid
        && inclusive.image_root != strict.image_root;
    const bool pitch_mutation_rejected = changed_pitch.valid
        && changed_pitch.image_root != base.image_root;
    const std::string executable_root = binary_root();
    std::ostringstream result_material;
    result_material << "nextengine.nonlocal.ncgp8.surface-self-test.v1\n"
                    << exact_zero << ':' << relabel_preserved << ':'
                    << z_translation_detected << ':' << x_translation_detected
                    << ':' << wrong_axis_rejected << ':' << deletion_rejected
                    << ':' << sparse_tail_robust << ':' << sheet_rejected << ':'
                    << strict_radius_rejected << ':' << pitch_mutation_rejected
                    << ':' << omitted_topology_rejected << ':'
                    << depth_mutation_rejected << ':'
                    << component_mutation_rejected << ':'
                    << work_mutation_rejected << '\n'
                    << base.image_root << ':' << relabelled_image.image_root
                    << ':' << z_image.image_root << ':' << x_image.image_root
                    << ':' << sparse_tail.image_root << ':'
                    << inclusive.image_root << ':' << strict.image_root << ':'
                    << changed_pitch.image_root << '\n'
                    << exact.metrics_root << ':' << z_comparison.metrics_root
                    << ':' << x_comparison.metrics_root << ':'
                    << sparse_comparison.metrics_root << '\n'
                    << NCGP8_CONTRACT_ROOT << ':' << NCGP8_SOURCE_ROOT << ':'
                    << NCGP8_SOURCE_COMMIT << ':' << NCGP8_SOURCE_TREE << ':'
                    << NCGP8_COMPILER_FLAGS << ':' << executable_root << '\n';
    const std::string result_root = nextengine::nonlocal::sha256_hex(
        result_material.str());
    const bool result_mutation_rejected = result_root
        != nextengine::nonlocal::sha256_hex(result_material.str() + "mutation");
    const bool passed = base.valid && exact_zero && relabel_preserved
        && z_translation_detected && x_translation_detected
        && wrong_axis_rejected && deletion_rejected && sparse_tail_robust
        && sheet_rejected && strict_radius_rejected && pitch_mutation_rejected
        && omitted_topology_rejected && depth_mutation_rejected
        && component_mutation_rejected && work_mutation_rejected
        && result_mutation_rejected && identity_valid(executable_root);
    std::cout << std::setprecision(17)
              << "{\"schema\":\"nextengine.nonlocal.ncgp8.surface-self-test.v1\""
              << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
              << ",\"vertical_axis\":\"z\""
              << ",\"exact_zero\":" << (exact_zero ? "true" : "false")
              << ",\"relabel_preserved\":"
              << (relabel_preserved ? "true" : "false")
              << ",\"z_translation_detected\":"
              << (z_translation_detected ? "true" : "false")
              << ",\"x_translation_detected\":"
              << (x_translation_detected ? "true" : "false")
              << ",\"wrong_axis_rejected\":"
              << (wrong_axis_rejected ? "true" : "false")
              << ",\"deletion_rejected\":"
              << (deletion_rejected ? "true" : "false")
              << ",\"sparse_tail_robust\":"
              << (sparse_tail_robust ? "true" : "false")
              << ",\"sheet_rejected\":"
              << (sheet_rejected ? "true" : "false")
              << ",\"strict_radius_rejected\":"
              << (strict_radius_rejected ? "true" : "false")
              << ",\"pitch_mutation_rejected\":"
              << (pitch_mutation_rejected ? "true" : "false")
              << ",\"omitted_topology_rejected\":"
              << (omitted_topology_rejected ? "true" : "false")
              << ",\"depth_mutation_rejected\":"
              << (depth_mutation_rejected ? "true" : "false")
              << ",\"component_mutation_rejected\":"
              << (component_mutation_rejected ? "true" : "false")
              << ",\"work_mutation_rejected\":"
              << (work_mutation_rejected ? "true" : "false")
              << ",\"result_mutation_rejected\":"
              << (result_mutation_rejected ? "true" : "false")
              << ",\"sparse_tail\":";
    emit_comparison(sparse_comparison);
    std::cout << ",\"sheet\":";
    emit_comparison(sheet_comparison);
    std::cout << ",\"result_root\":\"" << result_root
              << "\",\"contract_root\":\"" << NCGP8_CONTRACT_ROOT
              << "\",\"source_root\":\"" << NCGP8_SOURCE_ROOT
              << "\",\"source_commit\":\"" << NCGP8_SOURCE_COMMIT
              << "\",\"source_tree\":\"" << NCGP8_SOURCE_TREE
              << "\",\"compiler_flags\":\"" << NCGP8_COMPILER_FLAGS
              << "\",\"binary_root\":\"" << executable_root
              << "\",\"exact_command\":\"--visible-surface-self-test\"}\n";
    return passed ? 0 : 4;
}

int run_witness() {
    constexpr const char* expected_input_root =
        "f96ad2a8bd7c7fcee2917ff6168c1aac59d7da56d2f5844a3cfcdab396eda80f";
    constexpr const char* expected_receipt_root =
        "5dcff7ae2f7f9f6f6758a42aea9806b4631d71fe2aeb1b03cb78dab5e2c19b9b";
    constexpr const char* expected_gpu_state_root =
        "cd2d44238d08968de263bfc6c4495e6e3535e49d05f0ab82f449a687dc9a4222";
    constexpr const char* expected_cpu_state_root =
        "4bb62cc6c006d489b65fad6a6a87d8d81e0a8ade77a349fd1b03e2c8e1301af1";
    constexpr const char* ncgp7_result_root =
        "d11a79d7360b9ea4b1fc3cc85c974d4e97592bc29cdc09483a6dde7f6fe70abc";
    const NonlocalGpuProfile profile = nonlocal_water_corrected_profile();
    const auto ghosts = canonicalize_ghosts_binary32(make_basin_ghosts(profile));
    auto cpu_state = make_lattice_state(profile, 20U, 20U, 10U, false, false);
    const auto initial_state = cpu_state;
    const auto permuted_input =
        make_lattice_state(profile, 20U, 20U, 10U, true, false);
    const std::string input_root = input_semantic_root(
        profile, initial_state, ghosts);
    NonlocalGpuWorkspace gpu(profile);
    NonlocalGpuWorkspace permuted(profile);
    if (gpu.upload(initial_state, ghosts, true) != NonlocalGpuFailure::None
        || permuted.upload(permuted_input, ghosts, true)
            != NonlocalGpuFailure::None) return 51;
    NonlocalGpuPublicSnapshot final_gpu;
    NonlocalGpuPublicSnapshot final_permuted;
    NonlocalGpuStepResult final_gpu_step;
    NonlocalGpuStepResult final_permuted_step;
    NonlocalGpuStepResult final_cpu;
    std::ostringstream receipts;
    receipts << "nextengine.nonlocal.ncgp7.step112-receipts.v1\n";
    bool replay_valid = true;
    std::uint32_t completed = 0U;
    std::uint32_t maximum_gpu_hvp = 0U;
    std::uint32_t maximum_cpu_hvp = 0U;
    std::uint64_t snapshot_bytes = 0U;
    double maximum_penetration = 0.0;
    for (std::uint32_t step = 1U; step <= kWitnessStep; ++step) {
        const auto gpu_step = gpu.step(kWitnessBudget,
            NonlocalGpuSolverProfile::Unpreconditioned,
            NonlocalGpuVariant::CompensatedScaleF32, false, false);
        const auto permuted_step = permuted.step(kWitnessBudget,
            NonlocalGpuSolverProfile::Unpreconditioned,
            NonlocalGpuVariant::CompensatedScaleF32, false, false);
        const auto gpu_snapshot = gpu.capture_public_snapshot();
        const auto permuted_snapshot = permuted.capture_public_snapshot();
        const auto cpu_step = step_reference(profile, cpu_state, ghosts,
            kWitnessBudget, NonlocalGpuVariant::Corrected, true);
        receipts << step << ':' << step_work_semantic_root(profile, gpu_step)
                 << ':' << step_work_semantic_root(profile, permuted_step)
                 << ':' << step_work_semantic_root(profile, cpu_step) << ':'
                 << work_semantic_root(gpu_snapshot.work) << ':'
                 << work_semantic_root(permuted_snapshot.work) << '\n';
        maximum_gpu_hvp = std::max(maximum_gpu_hvp, gpu_step.hvp_used);
        maximum_cpu_hvp = std::max(maximum_cpu_hvp, cpu_step.hvp_used);
        maximum_penetration = std::max({maximum_penetration,
            gpu_step.maximum_penetration_m,
            permuted_step.maximum_penetration_m});
        snapshot_bytes += gpu_snapshot.work.device_to_host_bytes
            + permuted_snapshot.work.device_to_host_bytes;
        if (gpu_step.failure != NonlocalGpuFailure::None
            || permuted_step.failure != NonlocalGpuFailure::None
            || cpu_step.failure != NonlocalGpuFailure::None
            || gpu_snapshot.failure != NonlocalGpuFailure::None
            || permuted_snapshot.failure != NonlocalGpuFailure::None
            || gpu_snapshot.state.size() != kWitnessSamples
            || gpu_snapshot.state.size() != cpu_step.state.size()
            || gpu_snapshot.state.size() != permuted_snapshot.state.size()
            || gpu_snapshot.density.size() != cpu_step.density.size()
            || state_root(gpu_snapshot.state, gpu_snapshot.density,
                   gpu_snapshot.active_pressure_ids)
                != state_root(permuted_snapshot.state,
                    permuted_snapshot.density,
                    permuted_snapshot.active_pressure_ids)) {
            replay_valid = false;
            break;
        }
        cpu_state = cpu_step.state;
        ++completed;
        if (step == kWitnessStep) {
            final_gpu = gpu_snapshot;
            final_permuted = permuted_snapshot;
            final_gpu_step = gpu_step;
            final_permuted_step = permuted_step;
            final_cpu = cpu_step;
        }
    }
    std::vector<double> position_errors;
    double position_squared = 0.0;
    if (replay_valid && completed == kWitnessStep) {
        position_errors.reserve(kWitnessSamples);
        for (std::size_t index = 0U; index < kWitnessSamples; ++index) {
            const Vec3d delta{
                final_gpu.state[index].current.x
                    - final_cpu.state[index].current.x,
                final_gpu.state[index].current.y
                    - final_cpu.state[index].current.y,
                final_gpu.state[index].current.z
                    - final_cpu.state[index].current.z};
            const double error = norm(delta);
            position_errors.push_back(error);
            position_squared += error * error;
        }
        std::sort(position_errors.begin(), position_errors.end());
    }
    const double position_rmse = position_errors.empty()
        ? std::numeric_limits<double>::infinity()
        : std::sqrt(position_squared / position_errors.size());
    const double position_p99 = position_errors.empty()
        ? std::numeric_limits<double>::infinity()
        : position_errors[nearest_rank_index(kWitnessSamples, 99U, 100U)];
    const double position_maximum = position_errors.empty()
        ? std::numeric_limits<double>::infinity() : position_errors.back();
    const bool witness_exact = bits(position_rmse)
            == bits(0.00078099627929678129)
        && bits(position_p99) == bits(0.0025768828657373294)
        && bits(position_maximum) == bits(0.020810782527689854);
    const std::string receipt_root = nextengine::nonlocal::sha256_hex(
        receipts.str());
    const std::string gpu_state_root = state_root(final_gpu.state,
        final_gpu.density, final_gpu.active_pressure_ids);
    const std::string cpu_state_root_value = state_root(final_cpu.state,
        final_cpu.density, final_cpu.active_pressure_ids);
    const std::string permuted_state_root = state_root(final_permuted.state,
        final_permuted.density, final_permuted.active_pressure_ids);
    const bool parent_exact = input_root == expected_input_root
        && receipt_root == expected_receipt_root
        && gpu_state_root == expected_gpu_state_root
        && cpu_state_root_value == expected_cpu_state_root
        && permuted_state_root == expected_gpu_state_root && witness_exact;

    const double exact_mass = profile.mass * static_cast<double>(kWitnessSamples);
    const auto gpu_image = build_visible_surface(
        profile, final_gpu.state, kWitnessSamples, exact_mass);
    const auto cpu_image = build_visible_surface(
        profile, final_cpu.state, kWitnessSamples, exact_mass);
    const auto permuted_image = build_visible_surface(
        profile, final_permuted.state, kWitnessSamples, exact_mass);
    const auto comparison = compare_visible_surfaces(gpu_image, cpu_image);
    const bool permutation_exact = gpu_image.image_root
        == permuted_image.image_root;
    const Vec3d gpu_momentum = total_momentum(profile, final_gpu.state);
    const Vec3d cpu_momentum = total_momentum(profile, final_cpu.state);
    const Vec3d momentum_delta{gpu_momentum.x - cpu_momentum.x,
        gpu_momentum.y - cpu_momentum.y, gpu_momentum.z - cpu_momentum.z};
    const double momentum_correspondence = norm(momentum_delta)
        / std::max(exact_mass * profile.spacing / profile.dt, 1.0);
    const auto [minimum, maximum] = position_bounds(final_gpu.state);
    const bool contained = minimum.x >= 0.0 && minimum.y >= 0.0
        && minimum.z >= 0.0 && maximum.x <= profile.basin_extent.x
        && maximum.y <= profile.basin_extent.y
        && maximum.z <= profile.basin_extent.z;
    const bool retained_passed = parent_exact && exact_mass == 500.0
        && final_gpu.state.size() == kWitnessSamples
        && final_cpu.state.size() == kWitnessSamples
        && final_permuted.state.size() == kWitnessSamples
        && momentum_correspondence <= 0.01
        && maximum_penetration <= 0.0025 && contained
        && step_work_semantic_root(profile, final_gpu_step)
            == step_work_semantic_root(profile, final_permuted_step);
    const std::string executable_root = binary_root();
    const bool apparatus_valid = replay_valid && completed == kWitnessStep
        && vertical_axis_valid(profile, kVerticalAxisZ) && parent_exact
        && gpu_image.valid && cpu_image.valid && permuted_image.valid
        && comparison.valid && permutation_exact && retained_passed
        && identity_valid(executable_root);
    const std::string verdict = classification(comparison, apparatus_valid);
    std::ostringstream result_material;
    result_material << std::setprecision(17)
                    << "nextengine.nonlocal.ncgp8.step112-result.v1\n"
                    << verdict << ':' << completed << ':' << witness_exact
                    << ':' << position_rmse << ':' << position_p99 << ':'
                    << position_maximum << ':' << maximum_gpu_hvp << ':'
                    << maximum_cpu_hvp << ':' << snapshot_bytes << ':'
                    << maximum_penetration << ':' << momentum_correspondence
                    << ':' << contained << ':' << retained_passed << '\n'
                    << input_root << ':' << receipt_root << ':'
                    << gpu_state_root << ':' << cpu_state_root_value << ':'
                    << permuted_state_root << ':' << ncgp7_result_root << '\n'
                    << gpu_image.image_root << ':' << cpu_image.image_root
                    << ':' << permuted_image.image_root << ':'
                    << comparison.metrics_root << '\n'
                    << gpu_image.work_root << ':' << cpu_image.work_root << ':'
                    << permuted_image.work_root << ':' << comparison.work_root
                    << '\n' << "--diagnose-hydro-step112-visible-surface" << '\n'
                    << NCGP8_CONTRACT_ROOT << ':' << NCGP8_SOURCE_ROOT << ':'
                    << NCGP8_SOURCE_COMMIT << ':' << NCGP8_SOURCE_TREE << ':'
                    << NCGP8_COMPILER_FLAGS << ':' << executable_root << ':'
                    << gpu.environment_json() << '\n';
    const std::string result_root = nextengine::nonlocal::sha256_hex(
        result_material.str());
    std::cout << std::setprecision(17)
              << "{\"schema\":\"nextengine.nonlocal.ncgp8.step112-visible-surface.v1\""
              << ",\"status\":\"" << verdict << "\""
              << ",\"completed_steps\":" << completed
              << ",\"vertical_axis\":\"z\""
              << ",\"image_plane\":\"x-y\""
              << ",\"sphere_radius_m\":" << kSphereRadius
              << ",\"pixel_pitch_m\":" << kPixelPitch
              << ",\"witness_exact\":"
              << (witness_exact ? "true" : "false")
              << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
              << ",\"position_rmse_m\":" << position_rmse
              << ",\"position_p99_m\":" << position_p99
              << ",\"position_maximum_m\":" << position_maximum
              << ",\"maximum_gpu_hvp\":" << maximum_gpu_hvp
              << ",\"maximum_cpu_hvp\":" << maximum_cpu_hvp
              << ",\"snapshot_device_to_host_bytes\":" << snapshot_bytes
              << ",\"particle_count\":" << final_gpu.state.size()
              << ",\"total_mass_kg\":" << exact_mass
              << ",\"momentum_correspondence_fraction\":"
              << momentum_correspondence
              << ",\"maximum_penetration_m\":" << maximum_penetration
              << ",\"closed_basin_bounds\":"
              << (contained ? "true" : "false")
              << ",\"permutation_exact\":"
              << (permutation_exact ? "true" : "false")
              << ",\"comparison\":";
    emit_comparison(comparison);
    std::cout << ",\"gpu_image_root\":\"" << gpu_image.image_root
              << "\",\"cpu_image_root\":\"" << cpu_image.image_root
              << "\",\"permuted_image_root\":\""
              << permuted_image.image_root
              << "\",\"gpu_state_root\":\"" << gpu_state_root
              << "\",\"cpu_state_root\":\"" << cpu_state_root_value
              << "\",\"permuted_state_root\":\"" << permuted_state_root
              << "\",\"input_root\":\"" << input_root
              << "\",\"receipt_root\":\"" << receipt_root
              << "\",\"parent_ncgp7_result_root\":\"" << ncgp7_result_root
              << "\",\"result_root\":\"" << result_root
              << "\",\"contract_root\":\"" << NCGP8_CONTRACT_ROOT
              << "\",\"source_root\":\"" << NCGP8_SOURCE_ROOT
              << "\",\"source_commit\":\"" << NCGP8_SOURCE_COMMIT
              << "\",\"source_tree\":\"" << NCGP8_SOURCE_TREE
              << "\",\"compiler_flags\":\"" << NCGP8_COMPILER_FLAGS
              << "\",\"binary_root\":\"" << executable_root
              << "\",\"environment\":" << gpu.environment_json()
              << ",\"allocated_device_bytes\":" << gpu.allocated_device_bytes()
              << ",\"exact_command\":\"--diagnose-hydro-step112-visible-surface\""
              << ",\"performance_status\":\"NOT_RUN\"}\n";
    if (verdict == "H8A_VISIBLE_SURFACE_SUPPORTED_BOUNDED") return 0;
    if (verdict == "H8B_VISIBLE_SURFACE_DIVERGENCE_SUPPORTED_BOUNDED") return 37;
    return verdict == "H8C_VISIBLE_SURFACE_INCONCLUSIVE" ? 38 : 4;
}

} // namespace

int main(int argc, char** argv) {
    if (argc == 2 && std::string(argv[1]) == "--visible-surface-self-test") {
        return run_self_test();
    }
    if (argc == 2
        && std::string(argv[1])
            == "--diagnose-hydro-step112-visible-surface") {
        return run_witness();
    }
    std::cerr << "usage: nonlocal-corrected-cuda-visible-surface "
                 "--visible-surface-self-test|"
                 "--diagnose-hydro-step112-visible-surface\n";
    return 2;
}
