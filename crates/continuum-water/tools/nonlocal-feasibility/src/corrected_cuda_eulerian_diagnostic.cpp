#include "corrected_cuda_eulerian_diagnostic.hpp"

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
#include <unordered_map>
#include <utility>
#include <vector>

#ifndef NCGP7_CONTRACT_ROOT
#define NCGP7_CONTRACT_ROOT "unconfigured"
#endif
#ifndef NCGP7_SOURCE_ROOT
#define NCGP7_SOURCE_ROOT "unconfigured"
#endif
#ifndef NCGP7_SOURCE_COMMIT
#define NCGP7_SOURCE_COMMIT "unconfigured"
#endif
#ifndef NCGP7_SOURCE_TREE
#define NCGP7_SOURCE_TREE "unconfigured"
#endif
#ifndef NCGP7_COMPILER_FLAGS
#define NCGP7_COMPILER_FLAGS "unconfigured"
#endif

namespace {

using namespace nextengine::nonlocal::gpu_full_step;

constexpr std::size_t kWitnessSamples = 4000U;
constexpr std::uint32_t kWitnessStep = 112U;
constexpr std::uint32_t kWitnessBudget = 128U;
constexpr double kSurfaceQuantile = 0.99;
constexpr const char* kNcgp6ParentResultRoot =
    "4e72a97b1add42e7c58f839fc66dd918879b4325c23f75e5e5f1cf696ea57009";

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
    std::ostringstream bytes_stream;
    bytes_stream << stream.rdbuf();
    return nextengine::nonlocal::sha256_hex(bytes_stream.str());
}

std::string binary_root() { return file_root("/proc/self/exe"); }

struct EulerianGridSpec {
    double spacing = 0.0;
    std::uint32_t nx = 0U;
    std::uint32_t ny = 0U;
    std::uint32_t nz = 0U;
};

struct EulerianNode {
    double mass = 0.0;
    double density_moment = 0.0;
    Vec3d momentum;
};

struct SurfaceColumn {
    double mass = 0.0;
    double height = 0.0;
    bool wet = false;
};

struct EulerianWork {
    std::uint64_t samples_validated = 0U;
    std::uint64_t cic_records_emitted = 0U;
    std::uint64_t cic_records_sorted = 0U;
    std::uint64_t cic_records_reduced = 0U;
    std::uint64_t column_records_emitted = 0U;
    std::uint64_t column_records_sorted = 0U;
    std::uint64_t column_records_reduced = 0U;
    std::uint64_t quantile_reads = 0U;
    std::uint64_t node_comparisons = 0U;
    std::uint64_t column_comparisons = 0U;
    std::uint64_t root_derivations = 0U;
};

struct EulerianField {
    bool valid = false;
    EulerianGridSpec spec;
    std::vector<EulerianNode> nodes;
    std::vector<SurfaceColumn> columns;
    double input_mass = 0.0;
    double deposited_mass = 0.0;
    double deposited_column_mass = 0.0;
    double surface_quantile = 0.0;
    EulerianWork work;
    std::string node_root;
    std::string surface_root;
    std::string work_root;
    std::string field_root;
};

struct EulerianComparison {
    bool valid = false;
    double mass_tv = std::numeric_limits<double>::infinity();
    double density_rmse = std::numeric_limits<double>::infinity();
    double velocity_rmse_normalized = std::numeric_limits<double>::infinity();
    double center_of_mass_error = std::numeric_limits<double>::infinity();
    double maximum_node_mass_difference_particles =
        std::numeric_limits<double>::infinity();
    double wet_column_symmetric_difference =
        std::numeric_limits<double>::infinity();
    double surface_rmse = std::numeric_limits<double>::infinity();
    double surface_p95 = std::numeric_limits<double>::infinity();
    double surface_maximum = std::numeric_limits<double>::infinity();
    std::uint64_t common_nodes = 0U;
    std::uint64_t common_wet_columns = 0U;
    std::uint64_t union_wet_columns = 0U;
    EulerianWork work;
    std::string work_root;
    std::string metrics_root;
};

struct NodeContribution {
    std::size_t node = 0U;
    std::array<std::uint64_t, 7> order{};
    long double mass = 0.0L;
    long double density_moment = 0.0L;
    std::array<long double, 3> momentum{};
};

struct ColumnContribution {
    std::size_t column = 0U;
    std::array<std::uint64_t, 7> order{};
    double height = 0.0;
    long double mass = 0.0L;
};

EulerianGridSpec grid_spec(const NonlocalGpuProfile& profile, double spacing) {
    EulerianGridSpec result;
    result.spacing = spacing;
    if (!(spacing > 0.0) || !std::isfinite(spacing)) return result;
    const auto dimension = [spacing](double extent) {
        const double cells = extent / spacing;
        const auto rounded = static_cast<std::uint32_t>(std::llround(cells));
        if (std::abs(cells - static_cast<double>(rounded)) > 1.0e-12) {
            return 0U;
        }
        return rounded + 1U;
    };
    result.nx = dimension(profile.basin_extent.x);
    result.ny = dimension(profile.basin_extent.y);
    result.nz = dimension(profile.basin_extent.z);
    return result;
}

std::size_t node_index(
    const EulerianGridSpec& spec, std::uint32_t x, std::uint32_t y,
    std::uint32_t z) {
    return (static_cast<std::size_t>(z) * spec.ny + y) * spec.nx + x;
}

std::size_t column_index(
    const EulerianGridSpec& spec, std::uint32_t x, std::uint32_t z) {
    return static_cast<std::size_t>(z) * spec.nx + x;
}

std::string eulerian_work_root(const EulerianWork& work) {
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp7.eulerian-work.v1\n"
             << work.samples_validated << ':' << work.cic_records_emitted
             << ':' << work.cic_records_sorted << ':'
             << work.cic_records_reduced << ':'
             << work.column_records_emitted << ':'
             << work.column_records_sorted << ':'
             << work.column_records_reduced << ':' << work.quantile_reads
             << ':' << work.node_comparisons << ':'
             << work.column_comparisons << ':' << work.root_derivations
             << '\n';
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string node_record_root(const EulerianField& field) {
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp7.eulerian-nodes.v1\n" << std::hex
             << bits(field.spec.spacing) << ':' << field.spec.nx << ':'
             << field.spec.ny << ':' << field.spec.nz << ':'
             << field.nodes.size() << '\n';
    for (std::size_t index = 0U; index < field.nodes.size(); ++index) {
        const auto& node = field.nodes[index];
        material << index << ':' << bits(node.mass) << ':'
                 << bits(node.density_moment) << ':' << bits(node.momentum.x)
                 << ':' << bits(node.momentum.y) << ':'
                 << bits(node.momentum.z) << '\n';
    }
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string surface_record_root(const EulerianField& field) {
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp7.eulerian-surface.v1\n" << std::hex
             << bits(field.spec.spacing) << ':' << bits(field.surface_quantile)
             << ':' << field.spec.nx << ':' << field.spec.nz << ':'
             << field.columns.size() << '\n';
    for (std::size_t index = 0U; index < field.columns.size(); ++index) {
        const auto& column = field.columns[index];
        material << index << ':' << bits(column.mass) << ':'
                 << bits(column.height) << ':' << column.wet << '\n';
    }
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string complete_field_root(const EulerianField& field) {
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp7.eulerian-field.v1\n" << std::hex
             << bits(field.spec.spacing) << ':' << field.spec.nx << ':'
             << field.spec.ny << ':' << field.spec.nz << ':'
             << bits(field.input_mass) << ':' << bits(field.deposited_mass)
             << ':' << bits(field.deposited_column_mass) << ':'
             << bits(field.surface_quantile) << ':' << field.valid << '\n'
             << field.node_root << ':' << field.surface_root << ':'
             << field.work_root << '\n';
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::array<std::uint64_t, 7> order_key(
    const NonlocalGpuSample& sample, double density) {
    return {bits(sample.current.x), bits(sample.current.y),
        bits(sample.current.z), bits(density), bits(sample.velocity.x),
        bits(sample.velocity.y), bits(sample.velocity.z)};
}

EulerianField build_field(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& state,
    const std::vector<double>& density, double spacing,
    double surface_quantile = kSurfaceQuantile) {
    EulerianField result;
    result.spec = grid_spec(profile, spacing);
    result.surface_quantile = surface_quantile;
    if (state.empty() || state.size() != density.size()
        || result.spec.nx < 2U || result.spec.ny < 2U
        || result.spec.nz < 2U || !(surface_quantile > 0.0)
        || !(surface_quantile <= 1.0)) return result;
    result.input_mass = profile.mass * static_cast<double>(state.size());
    const std::size_t node_count = static_cast<std::size_t>(result.spec.nx)
        * result.spec.ny * result.spec.nz;
    const std::size_t column_count = static_cast<std::size_t>(result.spec.nx)
        * result.spec.nz;
    result.nodes.resize(node_count);
    result.columns.resize(column_count);
    std::vector<NodeContribution> node_records;
    std::vector<ColumnContribution> column_records;
    node_records.reserve(state.size() * 8U);
    column_records.reserve(state.size() * 4U);
    for (std::size_t sample_index = 0U; sample_index < state.size();
         ++sample_index) {
        const auto& sample = state[sample_index];
        const double rho = density[sample_index];
        if (!std::isfinite(sample.current.x)
            || !std::isfinite(sample.current.y)
            || !std::isfinite(sample.current.z)
            || !std::isfinite(sample.velocity.x)
            || !std::isfinite(sample.velocity.y)
            || !std::isfinite(sample.velocity.z) || !std::isfinite(rho)
            || rho < 0.0 || sample.current.x < 0.0
            || sample.current.y < 0.0 || sample.current.z < 0.0
            || sample.current.x > profile.basin_extent.x
            || sample.current.y > profile.basin_extent.y
            || sample.current.z > profile.basin_extent.z) return result;
        const std::array<double, 3> coordinate{sample.current.x,
            sample.current.y, sample.current.z};
        std::array<std::uint32_t, 3> lower{};
        std::array<long double, 3> fraction{};
        const std::array<std::uint32_t, 3> dimensions{result.spec.nx,
            result.spec.ny, result.spec.nz};
        for (std::size_t axis = 0U; axis < 3U; ++axis) {
            const long double scaled = static_cast<long double>(coordinate[axis])
                / static_cast<long double>(spacing);
            auto base = static_cast<std::uint32_t>(std::floor(scaled));
            long double local = scaled - static_cast<long double>(base);
            if (base + 1U >= dimensions[axis]) {
                base = dimensions[axis] - 2U;
                local = 1.0L;
            }
            lower[axis] = base;
            fraction[axis] = local;
        }
        const auto ordering = order_key(sample, rho);
        for (std::uint32_t dz = 0U; dz < 2U; ++dz) {
            const long double wz = dz == 0U ? 1.0L - fraction[2]
                                                   : fraction[2];
            for (std::uint32_t dy = 0U; dy < 2U; ++dy) {
                const long double wy = dy == 0U ? 1.0L - fraction[1]
                                                       : fraction[1];
                for (std::uint32_t dx = 0U; dx < 2U; ++dx) {
                    const long double wx = dx == 0U ? 1.0L - fraction[0]
                                                           : fraction[0];
                    const long double weight = wx * wy * wz;
                    NodeContribution contribution;
                    contribution.node = node_index(result.spec,
                        lower[0] + dx, lower[1] + dy, lower[2] + dz);
                    contribution.order = ordering;
                    contribution.mass = profile.mass * weight;
                    contribution.density_moment = profile.mass * rho * weight;
                    contribution.momentum = {
                        profile.mass * sample.velocity.x * weight,
                        profile.mass * sample.velocity.y * weight,
                        profile.mass * sample.velocity.z * weight};
                    node_records.push_back(contribution);
                }
            }
        }
        for (std::uint32_t dz = 0U; dz < 2U; ++dz) {
            const long double wz = dz == 0U ? 1.0L - fraction[2]
                                                   : fraction[2];
            for (std::uint32_t dx = 0U; dx < 2U; ++dx) {
                const long double wx = dx == 0U ? 1.0L - fraction[0]
                                                       : fraction[0];
                ColumnContribution contribution;
                contribution.column = column_index(result.spec,
                    lower[0] + dx, lower[2] + dz);
                contribution.order = ordering;
                contribution.height = sample.current.y;
                contribution.mass = profile.mass * wx * wz;
                column_records.push_back(contribution);
            }
        }
        ++result.work.samples_validated;
    }
    result.work.cic_records_emitted = node_records.size();
    result.work.column_records_emitted = column_records.size();
    std::sort(node_records.begin(), node_records.end(),
        [](const auto& lhs, const auto& rhs) {
            return std::tie(lhs.node, lhs.order)
                < std::tie(rhs.node, rhs.order);
        });
    result.work.cic_records_sorted = node_records.size();
    std::size_t cursor = 0U;
    while (cursor < node_records.size()) {
        const std::size_t node = node_records[cursor].node;
        long double mass = 0.0L;
        long double density_moment = 0.0L;
        std::array<long double, 3> momentum{};
        while (cursor < node_records.size()
            && node_records[cursor].node == node) {
            mass += node_records[cursor].mass;
            density_moment += node_records[cursor].density_moment;
            for (std::size_t axis = 0U; axis < 3U; ++axis) {
                momentum[axis] += node_records[cursor].momentum[axis];
            }
            ++cursor;
            ++result.work.cic_records_reduced;
        }
        result.nodes[node].mass = static_cast<double>(mass);
        result.nodes[node].density_moment =
            static_cast<double>(density_moment);
        result.nodes[node].momentum = {static_cast<double>(momentum[0]),
            static_cast<double>(momentum[1]),
            static_cast<double>(momentum[2])};
    }
    std::sort(column_records.begin(), column_records.end(),
        [](const auto& lhs, const auto& rhs) {
            return std::tie(lhs.column, lhs.height, lhs.order)
                < std::tie(rhs.column, rhs.height, rhs.order);
        });
    result.work.column_records_sorted = column_records.size();
    cursor = 0U;
    while (cursor < column_records.size()) {
        const std::size_t first = cursor;
        const std::size_t column = column_records[cursor].column;
        long double total = 0.0L;
        while (cursor < column_records.size()
            && column_records[cursor].column == column) {
            total += column_records[cursor].mass;
            ++cursor;
            ++result.work.column_records_reduced;
        }
        auto& output = result.columns[column];
        output.mass = static_cast<double>(total);
        output.wet = output.mass >= 0.25 * profile.mass;
        if (output.wet) {
            const long double target = surface_quantile * total;
            long double cumulative = 0.0L;
            for (std::size_t index = first; index < cursor; ++index) {
                cumulative += column_records[index].mass;
                ++result.work.quantile_reads;
                if (cumulative >= target) {
                    output.height = column_records[index].height;
                    break;
                }
            }
        }
    }
    long double node_mass = 0.0L;
    long double column_mass = 0.0L;
    for (const auto& node : result.nodes) node_mass += node.mass;
    for (const auto& column : result.columns) column_mass += column.mass;
    result.deposited_mass = static_cast<double>(node_mass);
    result.deposited_column_mass = static_cast<double>(column_mass);
    const double denominator = std::max(result.input_mass, 1.0);
    const double node_mass_error = std::abs(
        result.deposited_mass - result.input_mass) / denominator;
    const double column_mass_error = std::abs(
        result.deposited_column_mass - result.input_mass) / denominator;
    result.valid = node_mass_error
            <= 8.0 * std::numeric_limits<double>::epsilon()
        && column_mass_error
            <= 8.0 * std::numeric_limits<double>::epsilon();
    result.work.root_derivations = 4U;
    result.node_root = node_record_root(result);
    result.surface_root = surface_record_root(result);
    result.work_root = eulerian_work_root(result.work);
    result.field_root = complete_field_root(result);
    return result;
}

std::size_t nearest_rank_index(
    std::size_t size, std::size_t numerator, std::size_t denominator) {
    return (numerator * size + denominator - 1U) / denominator - 1U;
}

std::string comparison_root(
    double spacing, const EulerianComparison& comparison) {
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp7.eulerian-comparison.v1\n"
             << std::hex << bits(spacing) << ':' << bits(comparison.mass_tv)
             << ':' << bits(comparison.density_rmse) << ':'
             << bits(comparison.velocity_rmse_normalized) << ':'
             << bits(comparison.center_of_mass_error) << ':'
             << bits(comparison.maximum_node_mass_difference_particles) << ':'
             << bits(comparison.wet_column_symmetric_difference) << ':'
             << bits(comparison.surface_rmse) << ':'
             << bits(comparison.surface_p95) << ':'
             << bits(comparison.surface_maximum) << ':' << std::dec
             << comparison.common_nodes << ':'
             << comparison.common_wet_columns << ':'
             << comparison.union_wet_columns << ':' << comparison.valid
             << '\n' << comparison.work_root << '\n';
    return nextengine::nonlocal::sha256_hex(material.str());
}

EulerianComparison compare_fields(const NonlocalGpuProfile& profile,
    const EulerianField& gpu, const EulerianField& cpu) {
    EulerianComparison result;
    if (!gpu.valid || !cpu.valid || gpu.spec.spacing != cpu.spec.spacing
        || gpu.spec.nx != cpu.spec.nx || gpu.spec.ny != cpu.spec.ny
        || gpu.spec.nz != cpu.spec.nz || gpu.nodes.size() != cpu.nodes.size()
        || gpu.columns.size() != cpu.columns.size()) return result;
    long double absolute_mass = 0.0L;
    long double density_squared = 0.0L;
    long double velocity_squared = 0.0L;
    long double common_mass = 0.0L;
    long double gpu_mass = 0.0L;
    long double cpu_mass = 0.0L;
    std::array<long double, 3> gpu_first{};
    std::array<long double, 3> cpu_first{};
    const double common_threshold = 0.25 * profile.mass;
    for (std::size_t index = 0U; index < gpu.nodes.size(); ++index) {
        ++result.work.node_comparisons;
        const auto& g = gpu.nodes[index];
        const auto& c = cpu.nodes[index];
        const double mass_difference = std::abs(g.mass - c.mass);
        absolute_mass += mass_difference;
        result.maximum_node_mass_difference_particles = std::max(
            std::isfinite(result.maximum_node_mass_difference_particles)
                ? result.maximum_node_mass_difference_particles : 0.0,
            mass_difference / profile.mass);
        const std::uint32_t x = static_cast<std::uint32_t>(
            index % gpu.spec.nx);
        const std::size_t yz = index / gpu.spec.nx;
        const std::uint32_t y = static_cast<std::uint32_t>(
            yz % gpu.spec.ny);
        const std::uint32_t z = static_cast<std::uint32_t>(
            yz / gpu.spec.ny);
        const std::array<long double, 3> position{
            static_cast<long double>(x) * gpu.spec.spacing,
            static_cast<long double>(y) * gpu.spec.spacing,
            static_cast<long double>(z) * gpu.spec.spacing};
        gpu_mass += g.mass;
        cpu_mass += c.mass;
        for (std::size_t axis = 0U; axis < 3U; ++axis) {
            gpu_first[axis] += static_cast<long double>(g.mass)
                * position[axis];
            cpu_first[axis] += static_cast<long double>(c.mass)
                * position[axis];
        }
        if (g.mass >= common_threshold && c.mass >= common_threshold) {
            const long double weight = std::min(g.mass, c.mass);
            const double gpu_density = g.density_moment / g.mass;
            const double cpu_density = c.density_moment / c.mass;
            const Vec3d gpu_velocity{g.momentum.x / g.mass,
                g.momentum.y / g.mass, g.momentum.z / g.mass};
            const Vec3d cpu_velocity{c.momentum.x / c.mass,
                c.momentum.y / c.mass, c.momentum.z / c.mass};
            const Vec3d velocity_delta{gpu_velocity.x - cpu_velocity.x,
                gpu_velocity.y - cpu_velocity.y,
                gpu_velocity.z - cpu_velocity.z};
            density_squared += weight
                * (gpu_density - cpu_density) * (gpu_density - cpu_density);
            velocity_squared += weight
                * (velocity_delta.x * velocity_delta.x
                    + velocity_delta.y * velocity_delta.y
                    + velocity_delta.z * velocity_delta.z);
            common_mass += weight;
            ++result.common_nodes;
        }
    }
    result.mass_tv = static_cast<double>(0.5L * absolute_mass
        / static_cast<long double>(profile.mass * kWitnessSamples));
    if (common_mass > 0.0L) {
        result.density_rmse = static_cast<double>(std::sqrt(
            density_squared / common_mass)) / profile.rest_density;
        result.velocity_rmse_normalized = static_cast<double>(std::sqrt(
            velocity_squared / common_mass)) / (profile.spacing / profile.dt);
    }
    Vec3d center_difference;
    if (gpu_mass > 0.0L && cpu_mass > 0.0L) {
        center_difference = {
            static_cast<double>(gpu_first[0] / gpu_mass
                - cpu_first[0] / cpu_mass),
            static_cast<double>(gpu_first[1] / gpu_mass
                - cpu_first[1] / cpu_mass),
            static_cast<double>(gpu_first[2] / gpu_mass
                - cpu_first[2] / cpu_mass)};
        result.center_of_mass_error = norm(center_difference);
    }
    std::vector<double> surface_errors;
    std::uint64_t symmetric = 0U;
    for (std::size_t index = 0U; index < gpu.columns.size(); ++index) {
        ++result.work.column_comparisons;
        const bool g = gpu.columns[index].wet;
        const bool c = cpu.columns[index].wet;
        if (g || c) ++result.union_wet_columns;
        if (g != c) ++symmetric;
        if (g && c) {
            surface_errors.push_back(std::abs(
                gpu.columns[index].height - cpu.columns[index].height));
            ++result.common_wet_columns;
        }
    }
    if (result.union_wet_columns != 0U) {
        result.wet_column_symmetric_difference = static_cast<double>(symmetric)
            / static_cast<double>(result.union_wet_columns);
    }
    if (!surface_errors.empty()) {
        long double squared = 0.0L;
        for (double error : surface_errors) squared += error * error;
        std::sort(surface_errors.begin(), surface_errors.end());
        result.surface_rmse = static_cast<double>(std::sqrt(
            squared / static_cast<long double>(surface_errors.size())));
        result.surface_p95 = surface_errors[nearest_rank_index(
            surface_errors.size(), 95U, 100U)];
        result.surface_maximum = surface_errors.back();
    }
    result.valid = std::isfinite(result.mass_tv)
        && std::isfinite(result.density_rmse)
        && std::isfinite(result.velocity_rmse_normalized)
        && std::isfinite(result.center_of_mass_error)
        && std::isfinite(result.maximum_node_mass_difference_particles)
        && std::isfinite(result.wet_column_symmetric_difference)
        && std::isfinite(result.surface_rmse)
        && std::isfinite(result.surface_p95)
        && std::isfinite(result.surface_maximum) && result.common_nodes != 0U
        && result.common_wet_columns != 0U;
    result.work.root_derivations = 2U;
    result.work_root = eulerian_work_root(result.work);
    result.metrics_root = comparison_root(gpu.spec.spacing, result);
    return result;
}

bool h7a_band(const EulerianComparison& value) {
    return value.valid && value.mass_tv <= 0.01
        && value.density_rmse <= 0.01
        && value.velocity_rmse_normalized <= 0.01
        && value.center_of_mass_error <= 0.0025
        && value.wet_column_symmetric_difference <= 0.01
        && value.surface_rmse <= 0.0125 && value.surface_p95 <= 0.025;
}

bool h7b_band(const EulerianComparison& value) {
    return value.valid && (value.mass_tv >= 0.05
        || value.density_rmse >= 0.05
        || value.velocity_rmse_normalized >= 0.05
        || value.center_of_mass_error >= 0.025
        || value.wet_column_symmetric_difference >= 0.05
        || value.surface_rmse >= 0.05 || value.surface_p95 >= 0.10);
}

std::string classification(const EulerianComparison& coarse,
    const EulerianComparison& fine, bool apparatus_valid) {
    if (!apparatus_valid || !coarse.valid || !fine.valid) {
        return "APPARATUS_INCONCLUSIVE";
    }
    if (h7a_band(coarse) && h7a_band(fine)) {
        return "H7A_SUPPORTED_BOUNDED";
    }
    if (h7b_band(coarse)) return "H7B_SUPPORTED_BOUNDED";
    return "H7C_INCONCLUSIVE";
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

void emit_comparison(const EulerianComparison& value) {
    std::cout << std::setprecision(17)
              << "{\"valid\":" << (value.valid ? "true" : "false")
              << ",\"mass_tv\":" << value.mass_tv
              << ",\"density_rmse\":" << value.density_rmse
              << ",\"velocity_rmse_normalized\":"
              << value.velocity_rmse_normalized
              << ",\"center_of_mass_error_m\":"
              << value.center_of_mass_error
              << ",\"maximum_node_mass_difference_particles\":"
              << value.maximum_node_mass_difference_particles
              << ",\"wet_column_symmetric_difference\":"
              << value.wet_column_symmetric_difference
              << ",\"surface_rmse_m\":" << value.surface_rmse
              << ",\"surface_p95_m\":" << value.surface_p95
              << ",\"surface_maximum_m\":" << value.surface_maximum
              << ",\"common_nodes\":" << value.common_nodes
              << ",\"common_wet_columns\":"
              << value.common_wet_columns
              << ",\"union_wet_columns\":" << value.union_wet_columns
              << ",\"work_root\":\"" << value.work_root << "\""
              << ",\"metrics_root\":\"" << value.metrics_root << "\"}";
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

std::vector<NonlocalGpuSample> hydrostatic_initial(
    const NonlocalGpuProfile& profile, bool permuted) {
    return make_lattice_state(profile, 20U, 20U, 10U, permuted, false);
}

bool identity_valid(const std::string& executable_root) {
    return !executable_root.empty()
        && std::string(NCGP7_CONTRACT_ROOT) != "unconfigured"
        && std::string(NCGP7_SOURCE_ROOT) != "unconfigured"
        && std::string(NCGP7_SOURCE_COMMIT) != "unconfigured"
        && std::string(NCGP7_SOURCE_TREE) != "unconfigured"
        && std::string(NCGP7_COMPILER_FLAGS) != "unconfigured";
}

} // namespace

int run_ncgp7_eulerian_self_test() {
    const NonlocalGpuProfile profile = nonlocal_water_corrected_profile();
    const auto ghosts = canonicalize_ghosts_binary32(make_basin_ghosts(profile));
    const auto state = hydrostatic_initial(profile, false);
    const auto evaluation = evaluate_reference(
        profile, state, ghosts, nullptr, NonlocalGpuVariant::Corrected);
    if (evaluation.failure != NonlocalGpuFailure::None
        || evaluation.density.size() != state.size()) return 4;
    const EulerianField coarse = build_field(
        profile, state, evaluation.density, 0.05);
    const EulerianField fine = build_field(
        profile, state, evaluation.density, 0.025);
    const EulerianComparison zero_coarse = compare_fields(
        profile, coarse, coarse);
    const EulerianComparison zero_fine = compare_fields(profile, fine, fine);
    auto relabelled = state;
    for (std::size_t index = 0U; index < relabelled.size(); ++index) {
        relabelled[index].sample_id = state[(index + 137U) % state.size()].sample_id;
    }
    const EulerianField relabelled_coarse = build_field(
        profile, relabelled, evaluation.density, 0.05);
    const EulerianField relabelled_fine = build_field(
        profile, relabelled, evaluation.density, 0.025);
    const bool relabel_preserved = coarse.field_root
            == relabelled_coarse.field_root
        && fine.field_root == relabelled_fine.field_root;
    auto translated = state;
    for (auto& sample : translated) {
        sample.reference.y += 0.05;
        sample.current.y += 0.05;
    }
    const EulerianField translated_coarse = build_field(
        profile, translated, evaluation.density, 0.05);
    const EulerianField translated_fine = build_field(
        profile, translated, evaluation.density, 0.025);
    const EulerianComparison translation_coarse = compare_fields(
        profile, translated_coarse, coarse);
    const EulerianComparison translation_fine = compare_fields(
        profile, translated_fine, fine);
    const bool translation_detected = classification(translation_coarse,
        translation_fine, true) == "H7B_SUPPORTED_BOUNDED";
    auto deleted = state;
    auto deleted_density = evaluation.density;
    deleted.pop_back();
    deleted_density.pop_back();
    const EulerianField deleted_field = build_field(
        profile, deleted, deleted_density, 0.05);
    const bool deletion_rejected = deleted_field.valid
        && deleted.size() != kWitnessSamples
        && std::abs(deleted_field.deposited_mass
                - profile.mass * kWitnessSamples)
            > 8.0 * std::numeric_limits<double>::epsilon()
                * (profile.mass * kWitnessSamples);
    EulerianField omitted = coarse;
    for (auto& node : omitted.nodes) {
        node.density_moment = 0.0;
        node.momentum = {};
    }
    omitted.node_root = node_record_root(omitted);
    omitted.field_root = complete_field_root(omitted);
    const bool omitted_channels_rejected = omitted.field_root
        != coarse.field_root;
    const EulerianField changed_quantile = build_field(
        profile, state, evaluation.density, 0.05, 0.95);
    const bool quantile_mutation_rejected = changed_quantile.surface_root
        != coarse.surface_root;
    EulerianField mutated_field = coarse;
    mutated_field.nodes[0].mass = std::nextafter(
        mutated_field.nodes[0].mass, 1.0);
    mutated_field.node_root = node_record_root(mutated_field);
    mutated_field.field_root = complete_field_root(mutated_field);
    const bool field_mutation_rejected = mutated_field.field_root
        != coarse.field_root;
    EulerianWork mutated_work = coarse.work;
    ++mutated_work.cic_records_reduced;
    const bool work_mutation_rejected = eulerian_work_root(mutated_work)
        != coarse.work_root;
    const bool grid_mutation_rejected = coarse.field_root != fine.field_root;
    const bool exact_zero = zero_coarse.valid && zero_fine.valid
        && zero_coarse.mass_tv == 0.0 && zero_fine.mass_tv == 0.0
        && zero_coarse.density_rmse == 0.0
        && zero_fine.density_rmse == 0.0
        && zero_coarse.velocity_rmse_normalized == 0.0
        && zero_fine.velocity_rmse_normalized == 0.0
        && zero_coarse.center_of_mass_error == 0.0
        && zero_fine.center_of_mass_error == 0.0
        && zero_coarse.surface_rmse == 0.0
        && zero_fine.surface_rmse == 0.0;
    const std::string executable_root = binary_root();
    const bool passed = coarse.valid && fine.valid && exact_zero
        && relabel_preserved && translation_detected && deletion_rejected
        && omitted_channels_rejected && quantile_mutation_rejected
        && field_mutation_rejected && work_mutation_rejected
        && grid_mutation_rejected && identity_valid(executable_root);
    std::ostringstream result_material;
    result_material << "nextengine.nonlocal.ncgp7.self-test-result.v1\n"
                    << passed << ':' << exact_zero << ':'
                    << relabel_preserved << ':' << translation_detected << ':'
                    << deletion_rejected << ':' << omitted_channels_rejected
                    << ':' << quantile_mutation_rejected << ':'
                    << field_mutation_rejected << ':'
                    << work_mutation_rejected << ':'
                    << grid_mutation_rejected << '\n'
                    << coarse.field_root << ':' << fine.field_root << ':'
                    << translation_coarse.metrics_root << ':'
                    << translation_fine.metrics_root << '\n'
                    << NCGP7_CONTRACT_ROOT << ':' << NCGP7_SOURCE_ROOT << ':'
                    << NCGP7_SOURCE_COMMIT << ':' << NCGP7_SOURCE_TREE << ':'
                    << NCGP7_COMPILER_FLAGS << ':' << executable_root << '\n';
    const std::string result_root = nextengine::nonlocal::sha256_hex(
        result_material.str());
    std::cout << std::setprecision(17)
              << "{\"schema\":\"nextengine.nonlocal.ncgp7.eulerian-self-test.v1\""
              << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
              << ",\"exact_zero\":" << (exact_zero ? "true" : "false")
              << ",\"relabel_preserved\":"
              << (relabel_preserved ? "true" : "false")
              << ",\"translation_detected\":"
              << (translation_detected ? "true" : "false")
              << ",\"deletion_rejected\":"
              << (deletion_rejected ? "true" : "false")
              << ",\"omitted_channels_rejected\":"
              << (omitted_channels_rejected ? "true" : "false")
              << ",\"quantile_mutation_rejected\":"
              << (quantile_mutation_rejected ? "true" : "false")
              << ",\"field_mutation_rejected\":"
              << (field_mutation_rejected ? "true" : "false")
              << ",\"work_mutation_rejected\":"
              << (work_mutation_rejected ? "true" : "false")
              << ",\"grid_mutation_rejected\":"
              << (grid_mutation_rejected ? "true" : "false")
              << ",\"coarse_field_root\":\"" << coarse.field_root
              << "\",\"fine_field_root\":\"" << fine.field_root
              << "\",\"translation_coarse\":";
    emit_comparison(translation_coarse);
    std::cout << ",\"translation_fine\":";
    emit_comparison(translation_fine);
    std::cout << ",\"result_root\":\"" << result_root
              << "\",\"contract_root\":\"" << NCGP7_CONTRACT_ROOT
              << "\",\"source_root\":\"" << NCGP7_SOURCE_ROOT
              << "\",\"source_commit\":\"" << NCGP7_SOURCE_COMMIT
              << "\",\"source_tree\":\"" << NCGP7_SOURCE_TREE
              << "\",\"compiler_flags\":\"" << NCGP7_COMPILER_FLAGS
              << "\",\"binary_root\":\"" << executable_root
              << "\",\"exact_command\":\"--eulerian-field-self-test\"}\n";
    return passed ? 0 : 4;
}

int run_ncgp7_step112_eulerian_diagnostic() {
    const NonlocalGpuProfile profile = nonlocal_water_corrected_profile();
    const auto ghosts = canonicalize_ghosts_binary32(make_basin_ghosts(profile));
    auto cpu_state = hydrostatic_initial(profile, false);
    const auto initial_state = cpu_state;
    const auto permuted_input = hydrostatic_initial(profile, true);
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
    std::ostringstream step_receipts;
    step_receipts << "nextengine.nonlocal.ncgp7.step112-receipts.v1\n";
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
        step_receipts << step << ':'
                      << step_work_semantic_root(profile, gpu_step) << ':'
                      << step_work_semantic_root(profile, permuted_step) << ':'
                      << step_work_semantic_root(profile, cpu_step) << ':'
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
    double position_squared = 0.0;
    std::vector<double> position_errors;
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
    const EulerianField gpu_coarse = build_field(profile, final_gpu.state,
        final_gpu.density, 0.05);
    const EulerianField cpu_coarse = build_field(profile, final_cpu.state,
        final_cpu.density, 0.05);
    const EulerianField permuted_coarse = build_field(profile,
        final_permuted.state, final_permuted.density, 0.05);
    const EulerianField gpu_fine = build_field(profile, final_gpu.state,
        final_gpu.density, 0.025);
    const EulerianField cpu_fine = build_field(profile, final_cpu.state,
        final_cpu.density, 0.025);
    const EulerianField permuted_fine = build_field(profile,
        final_permuted.state, final_permuted.density, 0.025);
    const EulerianComparison coarse = compare_fields(
        profile, gpu_coarse, cpu_coarse);
    const EulerianComparison fine = compare_fields(profile, gpu_fine, cpu_fine);
    const bool permutation_exact = gpu_coarse.field_root
            == permuted_coarse.field_root
        && gpu_fine.field_root == permuted_fine.field_root;
    const double exact_mass = profile.mass
        * static_cast<double>(kWitnessSamples);
    const bool particle_and_mass_exact = final_gpu.state.size()
            == kWitnessSamples
        && final_cpu.state.size() == kWitnessSamples
        && final_permuted.state.size() == kWitnessSamples
        && exact_mass == 500.0;
    const Vec3d gpu_momentum = total_momentum(profile, final_gpu.state);
    const Vec3d cpu_momentum = total_momentum(profile, final_cpu.state);
    const Vec3d momentum_difference{gpu_momentum.x - cpu_momentum.x,
        gpu_momentum.y - cpu_momentum.y,
        gpu_momentum.z - cpu_momentum.z};
    const double momentum_correspondence = norm(momentum_difference)
        / std::max(exact_mass * profile.spacing / profile.dt, 1.0);
    const double final_energy_correspondence = std::abs(
        final_gpu_step.final_energy - final_cpu.final_energy)
        / std::max(std::abs(final_cpu.final_energy), 1.0);
    const auto [gpu_minimum, gpu_maximum] = position_bounds(final_gpu.state);
    const bool closed_basin_bounds = gpu_minimum.x >= 0.0
        && gpu_minimum.y >= 0.0 && gpu_minimum.z >= 0.0
        && gpu_maximum.x <= profile.basin_extent.x
        && gpu_maximum.y <= profile.basin_extent.y
        && gpu_maximum.z <= profile.basin_extent.z;
    const bool retained_observables_passed = particle_and_mass_exact
        && std::isfinite(momentum_correspondence)
        && momentum_correspondence <= 0.01
        && std::isfinite(final_energy_correspondence)
        && final_energy_correspondence <= 0.01
        && maximum_penetration <= 0.0025 && closed_basin_bounds
        && step_work_semantic_root(profile, final_gpu_step)
            == step_work_semantic_root(profile, final_permuted_step);
    const std::string executable_root = binary_root();
    const bool apparatus_valid = replay_valid && completed == kWitnessStep
        && witness_exact && gpu_coarse.valid && cpu_coarse.valid
        && permuted_coarse.valid && gpu_fine.valid && cpu_fine.valid
        && permuted_fine.valid && coarse.valid && fine.valid
        && permutation_exact && retained_observables_passed
        && identity_valid(executable_root);
    const std::string verdict = classification(coarse, fine, apparatus_valid);
    const std::string gpu_state_root = state_root(final_gpu.state,
        final_gpu.density, final_gpu.active_pressure_ids);
    const std::string cpu_state_root_value = state_root(final_cpu.state,
        final_cpu.density, final_cpu.active_pressure_ids);
    const std::string permuted_state_root = state_root(final_permuted.state,
        final_permuted.density, final_permuted.active_pressure_ids);
    const std::string receipt_root = nextengine::nonlocal::sha256_hex(
        step_receipts.str());
    std::ostringstream result_material;
    result_material << std::setprecision(17)
                    << "nextengine.nonlocal.ncgp7.step112-result.v1\n"
                    << verdict << ':' << completed << ':' << witness_exact
                    << ':' << position_rmse << ':' << position_p99 << ':'
                    << position_maximum << ':' << maximum_gpu_hvp << ':'
                    << maximum_cpu_hvp << ':' << snapshot_bytes << ':'
                    << maximum_penetration << ':' << particle_and_mass_exact
                    << ':' << momentum_correspondence << ':'
                    << final_energy_correspondence << ':'
                    << closed_basin_bounds << ':'
                    << retained_observables_passed << '\n'
                    << input_root << ':' << gpu_state_root << ':'
                    << cpu_state_root_value << ':' << permuted_state_root
                    << ':' << receipt_root << '\n'
                    << gpu_coarse.field_root << ':' << cpu_coarse.field_root
                    << ':' << permuted_coarse.field_root << ':'
                    << coarse.metrics_root << '\n'
                    << gpu_fine.field_root << ':' << cpu_fine.field_root << ':'
                    << permuted_fine.field_root << ':' << fine.metrics_root
                    << '\n' << gpu_coarse.work_root << ':'
                    << cpu_coarse.work_root << ':'
                    << permuted_coarse.work_root << ':' << coarse.work_root
                    << ':' << gpu_fine.work_root << ':' << cpu_fine.work_root
                    << ':' << permuted_fine.work_root << ':' << fine.work_root
                    << '\n' << kNcgp6ParentResultRoot << ':'
                    << "--diagnose-hydro-step112-eulerian" << '\n'
                    << NCGP7_CONTRACT_ROOT << ':' << NCGP7_SOURCE_ROOT
                    << ':' << NCGP7_SOURCE_COMMIT << ':' << NCGP7_SOURCE_TREE
                    << ':' << NCGP7_COMPILER_FLAGS << ':' << executable_root
                    << ':' << gpu.environment_json() << '\n';
    const std::string result_root = nextengine::nonlocal::sha256_hex(
        result_material.str());
    std::cout << std::setprecision(17)
              << "{\"schema\":\"nextengine.nonlocal.ncgp7.step112-eulerian.v1\""
              << ",\"status\":\"" << verdict << "\""
              << ",\"completed_steps\":" << completed
              << ",\"witness_exact\":"
              << (witness_exact ? "true" : "false")
              << ",\"position_rmse_m\":" << position_rmse
              << ",\"position_p99_m\":" << position_p99
              << ",\"position_maximum_m\":" << position_maximum
              << ",\"maximum_gpu_hvp\":" << maximum_gpu_hvp
              << ",\"maximum_cpu_hvp\":" << maximum_cpu_hvp
              << ",\"snapshot_device_to_host_bytes\":" << snapshot_bytes
              << ",\"particle_count\":" << final_gpu.state.size()
              << ",\"total_mass_kg\":" << exact_mass
              << ",\"particle_and_mass_exact\":"
              << (particle_and_mass_exact ? "true" : "false")
              << ",\"momentum_correspondence_fraction\":"
              << momentum_correspondence
              << ",\"final_solver_energy_correspondence_fraction\":"
              << final_energy_correspondence
              << ",\"maximum_penetration_m\":" << maximum_penetration
              << ",\"closed_basin_bounds\":"
              << (closed_basin_bounds ? "true" : "false")
              << ",\"retained_observables_passed\":"
              << (retained_observables_passed ? "true" : "false")
              << ",\"gpu_bounds_min_m\":[" << gpu_minimum.x << ','
              << gpu_minimum.y << ',' << gpu_minimum.z
              << "],\"gpu_bounds_max_m\":[" << gpu_maximum.x << ','
              << gpu_maximum.y << ',' << gpu_maximum.z << ']'
              << ",\"permutation_exact\":"
              << (permutation_exact ? "true" : "false")
              << ",\"coarse_spacing_m\":0.05,\"coarse\":";
    emit_comparison(coarse);
    std::cout << ",\"fine_spacing_m\":0.025,\"fine\":";
    emit_comparison(fine);
    std::cout << ",\"gpu_coarse_field_root\":\""
              << gpu_coarse.field_root
              << "\",\"cpu_coarse_field_root\":\""
              << cpu_coarse.field_root
              << "\",\"permuted_coarse_field_root\":\""
              << permuted_coarse.field_root
              << "\",\"gpu_fine_field_root\":\"" << gpu_fine.field_root
              << "\",\"cpu_fine_field_root\":\"" << cpu_fine.field_root
              << "\",\"permuted_fine_field_root\":\""
              << permuted_fine.field_root
              << "\",\"gpu_coarse_work_root\":\"" << gpu_coarse.work_root
              << "\",\"cpu_coarse_work_root\":\"" << cpu_coarse.work_root
              << "\",\"permuted_coarse_work_root\":\""
              << permuted_coarse.work_root
              << "\",\"gpu_fine_work_root\":\"" << gpu_fine.work_root
              << "\",\"cpu_fine_work_root\":\"" << cpu_fine.work_root
              << "\",\"permuted_fine_work_root\":\""
              << permuted_fine.work_root
              << "\",\"gpu_state_root\":\"" << gpu_state_root
              << "\",\"cpu_state_root\":\"" << cpu_state_root_value
              << "\",\"permuted_state_root\":\"" << permuted_state_root
              << "\",\"input_root\":\"" << input_root
              << "\",\"receipt_root\":\"" << receipt_root
              << "\",\"parent_ncgp6_result_root\":\""
              << kNcgp6ParentResultRoot
              << "\",\"result_root\":\"" << result_root
              << "\",\"contract_root\":\"" << NCGP7_CONTRACT_ROOT
              << "\",\"source_root\":\"" << NCGP7_SOURCE_ROOT
              << "\",\"source_commit\":\"" << NCGP7_SOURCE_COMMIT
              << "\",\"source_tree\":\"" << NCGP7_SOURCE_TREE
              << "\",\"compiler_flags\":\"" << NCGP7_COMPILER_FLAGS
              << "\",\"binary_root\":\"" << executable_root
              << "\",\"environment\":" << gpu.environment_json()
              << ",\"allocated_device_bytes\":"
              << gpu.allocated_device_bytes()
              << ",\"exact_command\":\"--diagnose-hydro-step112-eulerian\""
              << ",\"performance_status\":\"NOT_RUN\"}\n";
    return apparatus_valid ? 0 : 4;
}
