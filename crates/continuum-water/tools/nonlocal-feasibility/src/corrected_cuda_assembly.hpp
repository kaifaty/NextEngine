#pragma once

#include <array>
#include <cstdint>
#include <string>
#include <vector>

namespace nextengine::nonlocal::gpu_assembly_audit {

constexpr std::int64_t ASSEMBLY_SUPPORT_UM = 150000;
constexpr std::size_t ASSEMBLY_MAX_SAMPLES = 128U;

enum class AssemblyFailure : std::uint32_t {
    None = 0,
    InvalidInput = 1,
    DuplicateSampleId = 2,
    CapacityExceeded = 3,
    DeviceFailure = 4,
};

enum class AssemblyVariant : std::uint32_t {
    Corrected = 0,
    SourceShapedGradient = 1,
    DirectedEdgeViscosity = 2,
    OwnerPressureOnly = 3,
    GaussNewtonPressureOnly = 4,
    SissmLocalMatrix = 5,
    CurrentGraphViscosity = 6,
    NaiveF32Pressure = 7,
};

struct AssemblyProfile {
    double horizon = 0.15;
    double spacing = 0.05;
    double mass = 0.125;
    double time_step = 1.0 / 240.0;
    double rest_density = 1000.0;
    double kappa = 9196.875;
    double lambda = 2.5;
    double mu = 1.7;
    double gamma = 3.5;
};

struct AssemblyTerms {
    bool inertia = true;
    bool pressure = true;
    bool viscosity = true;
    bool surface = true;
};

struct AssemblySample {
    std::uint32_t sample_id = 0;
    std::array<std::int64_t, 3> reference_um{};
    std::array<std::int64_t, 3> predicted_um{};
    std::array<std::int64_t, 3> current_um{};
    std::array<std::int32_t, 3> direction_milli{};
};

struct AssemblyFixture {
    std::string name;
    AssemblyTerms terms;
    std::vector<AssemblySample> samples;
};

struct AssemblyWorkReceipt {
    std::uint64_t owner_sort_comparisons = 0;
    std::uint64_t graph_distance_predicates = 0;
    std::uint64_t density_kernel_evaluations = 0;
    std::uint64_t energy_pair_visits = 0;
    std::uint64_t gradient_pair_visits = 0;
    std::uint64_t active_pressure_centers = 0;
    std::uint64_t pressure_outer_products = 0;
    std::uint64_t pressure_geometric_blocks = 0;
    std::uint64_t viscosity_curvature_blocks = 0;
    std::uint64_t surface_curvature_blocks = 0;
    std::uint64_t dense_entries_written = 0;
    std::uint64_t hvp_products = 0;
    std::uint64_t host_device_scalar_transfers = 0;
    std::uint64_t compensated_additions = 0;
    std::uint64_t compensation_initializations = 0;
};

struct AssemblyResult {
    AssemblyFailure failure = AssemblyFailure::None;
    std::vector<std::uint32_t> owner_ids;
    std::vector<std::uint32_t> current_offsets;
    std::vector<std::uint32_t> current_neighbors;
    std::vector<std::uint32_t> reference_offsets;
    std::vector<std::uint32_t> reference_neighbors;
    std::vector<double> density;
    std::array<double, 4> energy{};
    std::vector<double> gradient;
    std::vector<double> hessian;
    std::vector<double> diagonal_blocks;
    std::vector<double> hvp;
    std::vector<std::uint8_t> pressure_active;
    double minimum_density_clamp_margin = 0.0;
    std::int64_t minimum_current_support_margin_um = 0;
    std::int64_t minimum_reference_support_margin_um = 0;
    std::int64_t minimum_surface_branch_margin_um = 0;
    AssemblyWorkReceipt work;
};

struct EnergyDerivativeResult {
    bool finite = false;
    bool branch_safe = false;
    double first_directional = 0.0;
    double second_directional = 0.0;
    double gradient_step = 0.0;
    double hessian_step = 0.0;
};

AssemblyResult evaluate_reference_assembly(
    const AssemblyProfile& profile,
    const AssemblyFixture& fixture);

AssemblyResult evaluate_reference_assembly_at(
    const AssemblyProfile& profile,
    const AssemblyFixture& fixture,
    const std::vector<double>& canonical_current_m);

EnergyDerivativeResult evaluate_energy_derivatives(
    const AssemblyProfile& profile,
    const AssemblyFixture& fixture);

AssemblyResult evaluate_gpu_assembly(
    const AssemblyProfile& profile,
    const AssemblyFixture& fixture,
    AssemblyVariant variant);

AssemblyResult evaluate_gpu_assembly_at(
    const AssemblyProfile& profile,
    const AssemblyFixture& fixture,
    const std::vector<double>& canonical_current_m,
    AssemblyVariant variant);

std::string gpu_assembly_environment_json();

} // namespace nextengine::nonlocal::gpu_assembly_audit
