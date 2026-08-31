#define main ncgp8_embedded_main
#include "corrected_cuda_visible_surface_main.cpp"
#undef main

#include "corrected_cuda_compensated_scale_physics.hpp"

#include <unordered_map>

#ifndef NCGP9_CONTRACT_ROOT
#define NCGP9_CONTRACT_ROOT "unconfigured"
#endif
#ifndef NCGP9_SOURCE_ROOT
#define NCGP9_SOURCE_ROOT "unconfigured"
#endif
#ifndef NCGP9_SOURCE_COMMIT
#define NCGP9_SOURCE_COMMIT "unconfigured"
#endif
#ifndef NCGP9_SOURCE_TREE
#define NCGP9_SOURCE_TREE "unconfigured"
#endif
#ifndef NCGP9_COMPILER_FLAGS
#define NCGP9_COMPILER_FLAGS "unconfigured"
#endif
#ifndef NCGP10_CONTRACT_ROOT
#define NCGP10_CONTRACT_ROOT "unconfigured"
#endif
#ifndef NCGP10_SOURCE_ROOT
#define NCGP10_SOURCE_ROOT "unconfigured"
#endif
#ifndef NCGP10_SOURCE_COMMIT
#define NCGP10_SOURCE_COMMIT "unconfigured"
#endif
#ifndef NCGP10_SOURCE_TREE
#define NCGP10_SOURCE_TREE "unconfigured"
#endif
#ifndef NCGP10_COMPILER_FLAGS
#define NCGP10_COMPILER_FLAGS "unconfigured"
#endif

namespace {

constexpr std::uint32_t kCorpusSteps = 240U;
constexpr std::uint32_t kCorpusBudget = 128U;
constexpr const char* kReviewedNcgp8ResultRoot =
    "ad982ab8cf5363b5222b4a5014076d55df393cba0c1facf01e2361d1e9cbce84";

struct PhysicsMetrics {
    bool valid = false;
    Vec3d momentum;
    Vec3d ghost_pressure_force;
    double mechanical_energy = 0.0;
};

struct OperatorGate {
    bool valid = false;
    double relative_l2 = 0.0;
    double cosine_loss = 0.0;
    bool active_signature_exact = false;
    std::uint32_t gpu_active_count = 0U;
    std::uint32_t cpu_active_count = 0U;
    std::uint32_t first_active_mismatch_id = 0U;
    double first_gpu_mismatch_density = 0.0;
    double first_cpu_mismatch_density = 0.0;
    double density_relative_rmse = 0.0;
    double density_relative_maximum = 0.0;
    std::string gpu_work_root;
    std::string cpu_work_root;
    std::string root;
};

struct ScenarioResult {
    std::string name;
    std::string arithmetic_profile;
    std::string status = "APPARATUS_INCONCLUSIVE";
    std::string first_gate;
    std::uint32_t first_gate_step = 0U;
    std::uint32_t completed_steps = 0U;
    std::uint32_t visible_steps = 0U;
    std::uint32_t bulk_checkpoints = 0U;
    double first_step_position_max = 0.0;
    double maximum_position_rmse = 0.0;
    double maximum_position_p99 = 0.0;
    double maximum_position_max = 0.0;
    double maximum_density_rmse = 0.0;
    double maximum_density_max = 0.0;
    double maximum_momentum_residual = 0.0;
    double maximum_positive_energy_excess = 0.0;
    double maximum_penetration = 0.0;
    double maximum_silhouette = 0.0;
    double maximum_depth_rmse = 0.0;
    double maximum_depth_p95 = 0.0;
    double maximum_depth_p99 = 0.0;
    double maximum_depth_max = 0.0;
    double maximum_component_fraction_difference = 0.0;
    double maximum_satellite_fraction_difference = 0.0;
    double maximum_cpu_satellite_fraction = 0.0;
    double maximum_gpu_satellite_fraction = 0.0;
    double failure_cpu_satellite_fraction = 0.0;
    double failure_gpu_satellite_fraction = 0.0;
    double failure_cpu_largest_component_fraction = 0.0;
    double failure_gpu_largest_component_fraction = 0.0;
    std::uint32_t failure_cpu_components = 0U;
    std::uint32_t failure_gpu_components = 0U;
    std::uint64_t failure_cpu_wet_pixels = 0U;
    std::uint64_t failure_gpu_wet_pixels = 0U;
    std::uint32_t maximum_gpu_hvp = 0U;
    std::uint32_t maximum_cpu_hvp = 0U;
    std::uint32_t maximum_degree = 0U;
    std::uint32_t maximum_directed_pairs = 0U;
    std::uint64_t hot_host_to_device_bytes = 0U;
    std::uint64_t hot_device_to_host_bytes = 0U;
    std::uint64_t snapshot_device_to_host_bytes = 0U;
    double operator_relative_l2 = 0.0;
    double operator_cosine_loss = 0.0;
    bool operator_active_signature_exact = false;
    std::uint32_t operator_gpu_active_count = 0U;
    std::uint32_t operator_cpu_active_count = 0U;
    std::uint32_t operator_first_active_mismatch_id = 0U;
    double operator_first_gpu_mismatch_density = 0.0;
    double operator_first_cpu_mismatch_density = 0.0;
    double operator_density_relative_rmse = 0.0;
    double operator_density_relative_maximum = 0.0;
    std::string input_root;
    std::string operator_root;
    std::string operator_gpu_work_root;
    std::string operator_cpu_work_root;
    std::string receipt_root;
    std::string final_gpu_state_root;
    std::string final_cpu_state_root;
    std::string final_permuted_state_root;
    std::string failure_gpu_state_root;
    std::string failure_cpu_state_root;
    std::string failure_permuted_state_root;
    std::string failure_gpu_image_root;
    std::string failure_cpu_image_root;
    std::string failure_permuted_image_root;
    bool failure_visible_present = false;
    VisibleSurfaceImage failure_gpu_image;
    VisibleSurfaceImage failure_cpu_image;
    VisibleSurfaceImage failure_permuted_image;
    VisibleSurfaceComparison failure_visible;
    std::string result_root;
};

void seal_scenario_4k(
    ScenarioResult& result, const std::string& receipt_material) {
    std::ostringstream failure_receipt;
    failure_receipt << "nextengine.nonlocal.ncgp9.failure-receipt.v1\n"
                    << result.name << ':' << result.first_gate << ':'
                    << result.first_gate_step << ':' << result.input_root << ':'
                    << result.operator_root << '\n';
    result.receipt_root = nextengine::nonlocal::sha256_hex(
        receipt_material.empty() ? failure_receipt.str() : receipt_material);
    std::ostringstream material;
    material << std::setprecision(17)
             << "nextengine.nonlocal.ncgp9.scenario-result.v1\n"
             << result.name << ':' << result.arithmetic_profile << ':'
             << result.status << ':'
             << result.first_gate << ':' << result.first_gate_step << ':'
             << result.completed_steps << ':' << result.visible_steps << ':'
             << result.bulk_checkpoints << '\n'
             << result.first_step_position_max << ':'
             << result.maximum_position_rmse << ':'
             << result.maximum_position_p99 << ':'
             << result.maximum_position_max << ':'
             << result.maximum_density_rmse << ':'
             << result.maximum_density_max << ':'
             << result.maximum_momentum_residual << ':'
             << result.maximum_positive_energy_excess << ':'
             << result.maximum_penetration << '\n'
             << result.maximum_silhouette << ':' << result.maximum_depth_rmse
             << ':' << result.maximum_depth_p95 << ':'
             << result.maximum_depth_p99 << ':' << result.maximum_depth_max
             << ':' << result.maximum_component_fraction_difference << ':'
             << result.maximum_satellite_fraction_difference << ':'
             << result.maximum_cpu_satellite_fraction << ':'
             << result.maximum_gpu_satellite_fraction << ':'
             << result.failure_cpu_satellite_fraction << ':'
             << result.failure_gpu_satellite_fraction << ':'
             << result.failure_cpu_largest_component_fraction << ':'
             << result.failure_gpu_largest_component_fraction << ':'
             << result.failure_cpu_components << ':'
             << result.failure_gpu_components << ':'
             << result.failure_cpu_wet_pixels << ':'
             << result.failure_gpu_wet_pixels << '\n'
             << result.operator_relative_l2 << ':'
             << result.operator_cosine_loss << ':'
             << result.operator_active_signature_exact << ':'
             << result.operator_gpu_active_count << ':'
             << result.operator_cpu_active_count << ':'
             << result.operator_first_active_mismatch_id << ':'
             << result.operator_first_gpu_mismatch_density << ':'
             << result.operator_first_cpu_mismatch_density << ':'
             << result.operator_density_relative_rmse << ':'
             << result.operator_density_relative_maximum << ':'
             << result.maximum_gpu_hvp << ':' << result.maximum_cpu_hvp << ':'
             << result.maximum_degree << ':' << result.maximum_directed_pairs
             << ':' << result.hot_host_to_device_bytes << ':'
             << result.hot_device_to_host_bytes << ':'
             << result.snapshot_device_to_host_bytes << '\n'
             << result.input_root << ':' << result.operator_root << ':'
             << result.operator_gpu_work_root << ':'
             << result.operator_cpu_work_root << ':' << result.receipt_root
             << ':' << result.final_gpu_state_root << ':'
             << result.final_cpu_state_root << ':'
             << result.final_permuted_state_root << ':'
             << result.failure_gpu_state_root << ':'
             << result.failure_cpu_state_root << ':'
             << result.failure_permuted_state_root << ':'
             << result.failure_gpu_image_root << ':'
             << result.failure_cpu_image_root << ':'
             << result.failure_permuted_image_root << '\n';
    result.result_root = nextengine::nonlocal::sha256_hex(material.str());
}

std::uint32_t float_bits_4k(float value) {
    std::uint32_t result = 0U;
    static_assert(sizeof(result) == sizeof(value));
    std::memcpy(&result, &value, sizeof(value));
    return result;
}

std::string compensated_state_root_4k(
    const NonlocalGpuCompensatedStateSnapshot& snapshot) {
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp9.device-state.v1\n"
             << static_cast<std::uint32_t>(snapshot.failure) << '\n';
    const std::array<const std::vector<Vec3d>*, 8> fields{
        &snapshot.reference_high, &snapshot.reference_low,
        &snapshot.current_high, &snapshot.current_low,
        &snapshot.predicted_high, &snapshot.predicted_low,
        &snapshot.velocity_high, &snapshot.velocity_low};
    for (std::size_t index = 0U; index < snapshot.ids.size(); ++index) {
        material << snapshot.ids[index] << ':';
        for (const auto* field : fields) {
            if (index >= field->size()) return {};
            const Vec3d& value = (*field)[index];
            material << std::hex
                     << float_bits_4k(static_cast<float>(value.x)) << ','
                     << float_bits_4k(static_cast<float>(value.y)) << ','
                     << float_bits_4k(static_cast<float>(value.z)) << ';'
                     << std::dec;
        }
        material << '\n';
    }
    return nextengine::nonlocal::sha256_hex(material.str());
}

long double cubic_weight_first_4k(
    long double radius, const NonlocalGpuProfile& profile) {
    const long double h = profile.horizon;
    const long double q = 2.0L * radius / h;
    const long double alpha = static_cast<long double>(profile.kernel_scale)
        * 3.0L / (2.0L * acosl(-1.0L) * h * h * h);
    long double derivative_q = 0.0L;
    if (q < 1.0L) {
        derivative_q = alpha * (-2.0L * q + 1.5L * q * q);
    } else if (q <= 2.0L) {
        const long double tail = 2.0L - q;
        derivative_q = -0.5L * alpha * tail * tail;
    }
    return derivative_q * 2.0L / h;
}

long double surface_potential_4k(long double radius, long double spacing) {
    const long double q = radius / spacing;
    if (q <= 1.0L) {
        return spacing * (q * q * q / 3.0L - q - 2.0L / 3.0L);
    }
    if (q < 3.0L) {
        const long double shifted = q - 2.0L;
        return spacing
            * (q - shifted * shifted * shifted / 3.0L - 8.0L / 3.0L);
    }
    return 0.0L;
}

PhysicsMetrics physics_metrics_4k(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& state,
    const std::vector<NonlocalGpuGhost>& ghosts,
    const std::vector<double>& density) {
    PhysicsMetrics result;
    if (state.empty() || density.size() != state.size()) return result;
    const auto graph = build_reference_graph(profile, state, ghosts, false);
    if (graph.failure != NonlocalGpuFailure::None
        || graph.offsets.size() != state.size() + 1U) {
        return result;
    }
    std::unordered_map<std::uint32_t, std::size_t> dynamic;
    std::unordered_map<std::uint32_t, Vec3d> ghost_positions;
    dynamic.reserve(state.size());
    ghost_positions.reserve(ghosts.size());
    for (std::size_t index = 0U; index < state.size(); ++index) {
        dynamic.emplace(state[index].sample_id, index);
        result.momentum.x += profile.mass * state[index].velocity.x;
        result.momentum.y += profile.mass * state[index].velocity.y;
        result.momentum.z += profile.mass * state[index].velocity.z;
        result.mechanical_energy += 0.5 * profile.mass
                * (state[index].velocity.x * state[index].velocity.x
                    + state[index].velocity.y * state[index].velocity.y
                    + state[index].velocity.z * state[index].velocity.z)
            - profile.mass * (profile.gravity.x * state[index].current.x
                + profile.gravity.y * state[index].current.y
                + profile.gravity.z * state[index].current.z);
        const double excess = std::max(
            density[index] / profile.rest_density - 1.0, 0.0);
        result.mechanical_energy += 0.5 * profile.kappa * excess * excess;
    }
    for (const auto& ghost : ghosts) {
        ghost_positions.emplace(ghost.sample_id, ghost.position);
    }
    for (std::size_t owner = 0U; owner < state.size(); ++owner) {
        const double excess = std::max(
            density[owner] / profile.rest_density - 1.0, 0.0);
        for (std::uint32_t slot = graph.offsets[owner];
             slot < graph.offsets[owner + 1U]; ++slot) {
            const std::uint32_t neighbor_id = graph.neighbor_ids[slot];
            const auto dynamic_neighbor = dynamic.find(neighbor_id);
            Vec3d neighbor{};
            if (dynamic_neighbor != dynamic.end()) {
                if (dynamic_neighbor->second == owner) continue;
                neighbor = state[dynamic_neighbor->second].current;
            } else {
                const auto ghost = ghost_positions.find(neighbor_id);
                if (ghost == ghost_positions.end()) return result;
                neighbor = ghost->second;
            }
            const Vec3d difference{state[owner].current.x - neighbor.x,
                state[owner].current.y - neighbor.y,
                state[owner].current.z - neighbor.z};
            const long double radius = std::sqrt(
                static_cast<long double>(difference.x) * difference.x
                + static_cast<long double>(difference.y) * difference.y
                + static_cast<long double>(difference.z) * difference.z);
            if (!(radius > 0.0L)) continue;
            if (dynamic_neighbor != dynamic.end()) {
                if (dynamic_neighbor->second > owner) {
                    result.mechanical_energy += static_cast<double>(
                        2.0L * profile.gamma * profile.mass * profile.mass
                        * surface_potential_4k(radius, profile.spacing));
                }
            } else if (excess > 0.0) {
                const long double coefficient = -static_cast<long double>(
                    profile.kappa * profile.mass / profile.rest_density
                    * excess)
                    * cubic_weight_first_4k(radius, profile) / radius;
                result.ghost_pressure_force.x +=
                    static_cast<double>(coefficient * difference.x);
                result.ghost_pressure_force.y +=
                    static_cast<double>(coefficient * difference.y);
                result.ghost_pressure_force.z +=
                    static_cast<double>(coefficient * difference.z);
            }
        }
    }
    result.valid = std::isfinite(result.mechanical_energy)
        && std::isfinite(result.momentum.x)
        && std::isfinite(result.momentum.y)
        && std::isfinite(result.momentum.z)
        && std::isfinite(result.ghost_pressure_force.x)
        && std::isfinite(result.ghost_pressure_force.y)
        && std::isfinite(result.ghost_pressure_force.z);
    return result;
}

std::vector<NonlocalGpuSample> corpus_initial(
    const NonlocalGpuProfile& profile, const std::string& scenario,
    bool permuted) {
    if (scenario == "hydrostatic-hold") {
        return make_lattice_state(profile, 20U, 20U, 10U, permuted, false);
    }
    if (scenario == "dam-break") {
        return make_lattice_state(profile, 10U, 20U, 20U, permuted, false);
    }
    if (scenario == "orifice-jet") {
        auto result =
            make_lattice_state(profile, 20U, 20U, 10U, permuted, false);
        for (auto& sample : result) {
            const double dy = sample.current.y - 0.75;
            const double dz = sample.current.z - 0.25;
            if (dy * dy + dz * dz <= 0.15 * 0.15) {
                sample.velocity.x = 1.5;
            }
        }
        return canonicalize_samples_binary32(result);
    }
    throw std::invalid_argument("unknown NCGP9 scenario");
}

double vector_norm_4k(Vec3d value) {
    return std::sqrt(value.x * value.x + value.y * value.y
        + value.z * value.z);
}

std::vector<Vec3d> operator_direction_4k(
    const std::vector<NonlocalGpuSample>& state) {
    std::vector<Vec3d> result;
    result.reserve(state.size());
    for (const auto& sample : state) {
        const std::int32_t id = static_cast<std::int32_t>(sample.sample_id);
        result.push_back({std::ldexp(static_cast<double>(id % 17 - 8), -10),
            std::ldexp(static_cast<double>(id % 13 - 6), -10),
            std::ldexp(static_cast<double>(id % 11 - 5), -10)});
    }
    return result;
}

OperatorGate operator_gate_4k(const NonlocalGpuProfile& profile,
    NonlocalGpuWorkspace& gpu, const std::vector<NonlocalGpuSample>& state,
    const std::vector<NonlocalGpuGhost>& ghosts, NonlocalGpuVariant variant) {
    OperatorGate result;
    auto canonical_state = state;
    std::sort(canonical_state.begin(), canonical_state.end(),
        [](const NonlocalGpuSample& lhs, const NonlocalGpuSample& rhs) {
            return lhs.sample_id < rhs.sample_id;
        });
    const auto direction = operator_direction_4k(canonical_state);
    const auto gpu_result = gpu.evaluate(&direction, variant, true, false);
    const auto cpu_result = evaluate_reference(
        profile, canonical_state, ghosts, &direction,
        NonlocalGpuVariant::Corrected);
    if (gpu_result.failure != NonlocalGpuFailure::None
        || cpu_result.failure != NonlocalGpuFailure::None
        || gpu_result.hvp.size() != cpu_result.hvp.size()
        || gpu_result.hvp.empty()) {
        return result;
    }
    long double difference_squared = 0.0L;
    long double gpu_squared = 0.0L;
    long double cpu_squared = 0.0L;
    long double product = 0.0L;
    for (std::size_t index = 0U; index < gpu_result.hvp.size(); ++index) {
        const Vec3d delta{gpu_result.hvp[index].x - cpu_result.hvp[index].x,
            gpu_result.hvp[index].y - cpu_result.hvp[index].y,
            gpu_result.hvp[index].z - cpu_result.hvp[index].z};
        difference_squared += static_cast<long double>(delta.x) * delta.x
            + static_cast<long double>(delta.y) * delta.y
            + static_cast<long double>(delta.z) * delta.z;
        gpu_squared += static_cast<long double>(gpu_result.hvp[index].x)
                * gpu_result.hvp[index].x
            + static_cast<long double>(gpu_result.hvp[index].y)
                * gpu_result.hvp[index].y
            + static_cast<long double>(gpu_result.hvp[index].z)
                * gpu_result.hvp[index].z;
        cpu_squared += static_cast<long double>(cpu_result.hvp[index].x)
                * cpu_result.hvp[index].x
            + static_cast<long double>(cpu_result.hvp[index].y)
                * cpu_result.hvp[index].y
            + static_cast<long double>(cpu_result.hvp[index].z)
                * cpu_result.hvp[index].z;
        product += static_cast<long double>(gpu_result.hvp[index].x)
                * cpu_result.hvp[index].x
            + static_cast<long double>(gpu_result.hvp[index].y)
                * cpu_result.hvp[index].y
            + static_cast<long double>(gpu_result.hvp[index].z)
                * cpu_result.hvp[index].z;
    }
    result.relative_l2 = static_cast<double>(
        std::sqrt(difference_squared / std::max(cpu_squared, 1.0e-300L)));
    result.cosine_loss = static_cast<double>(1.0L
        - product / std::sqrt(std::max(gpu_squared * cpu_squared, 1.0e-300L)));
    result.active_signature_exact = gpu_result.active_pressure_ids
        == cpu_result.active_pressure_ids;
    result.gpu_active_count = static_cast<std::uint32_t>(
        gpu_result.active_pressure_ids.size());
    result.cpu_active_count = static_cast<std::uint32_t>(
        cpu_result.active_pressure_ids.size());
    if (gpu_result.density.size() == cpu_result.density.size()
        && gpu_result.density.size() == canonical_state.size()) {
        long double density_squared = 0.0L;
        for (std::size_t index = 0U; index < gpu_result.density.size(); ++index) {
            const double relative = std::abs(
                gpu_result.density[index] - cpu_result.density[index])
                / profile.rest_density;
            density_squared += static_cast<long double>(relative) * relative;
            result.density_relative_maximum = std::max(
                result.density_relative_maximum, relative);
        }
        result.density_relative_rmse = static_cast<double>(std::sqrt(
            density_squared / static_cast<long double>(canonical_state.size())));
    }
    std::size_t gpu_active = 0U;
    std::size_t cpu_active = 0U;
    while (gpu_active < gpu_result.active_pressure_ids.size()
        || cpu_active < cpu_result.active_pressure_ids.size()) {
        const std::uint32_t gpu_id = gpu_active
                < gpu_result.active_pressure_ids.size()
            ? gpu_result.active_pressure_ids[gpu_active]
            : std::numeric_limits<std::uint32_t>::max();
        const std::uint32_t cpu_id = cpu_active
                < cpu_result.active_pressure_ids.size()
            ? cpu_result.active_pressure_ids[cpu_active]
            : std::numeric_limits<std::uint32_t>::max();
        if (gpu_id == cpu_id) {
            ++gpu_active;
            ++cpu_active;
            continue;
        }
        result.first_active_mismatch_id = std::min(gpu_id, cpu_id);
        const auto sample = std::lower_bound(canonical_state.begin(),
            canonical_state.end(), result.first_active_mismatch_id,
            [](const NonlocalGpuSample& value, std::uint32_t id) {
                return value.sample_id < id;
            });
        if (sample != canonical_state.end()
            && sample->sample_id == result.first_active_mismatch_id) {
            const std::size_t index = static_cast<std::size_t>(
                sample - canonical_state.begin());
            result.first_gpu_mismatch_density = gpu_result.density[index];
            result.first_cpu_mismatch_density = cpu_result.density[index];
        }
        break;
    }
    result.gpu_work_root = work_semantic_root(gpu_result.work);
    result.cpu_work_root = work_semantic_root(cpu_result.work);
    result.valid = result.relative_l2 <= 1.0e-3
        && result.cosine_loss <= 1.0e-6 && result.active_signature_exact;
    std::ostringstream material;
    material << std::hex << "nextengine.nonlocal.ncgp9.operator.v1\n"
             << result.valid << ':' << bits(result.relative_l2) << ':'
             << bits(result.cosine_loss) << ':'
             << result.active_signature_exact << ':'
             << result.gpu_active_count << ':' << result.cpu_active_count << ':'
             << result.first_active_mismatch_id << ':'
             << bits(result.first_gpu_mismatch_density) << ':'
             << bits(result.first_cpu_mismatch_density) << ':'
             << bits(result.density_relative_rmse) << ':'
             << bits(result.density_relative_maximum) << '\n'
             << result.gpu_work_root << ':' << result.cpu_work_root << '\n';
    result.root = nextengine::nonlocal::sha256_hex(material.str());
    return result;
}

bool bulk_checkpoint_valid(
    const nextengine::nonlocal::ncgp8::RetainedBulkClosure& value) {
    return value.gpu_coarse.valid && value.cpu_coarse.valid
        && value.permuted_coarse.valid && value.gpu_fine.valid
        && value.cpu_fine.valid && value.permuted_fine.valid
        && value.coarse.valid && value.fine.valid && value.bulk_bands_passed
        && value.permutation_exact;
}

bool is_bulk_checkpoint(std::uint32_t step) {
    return step == 60U || step == 120U || step == 180U || step == 240U;
}

ScenarioResult run_scenario(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuGhost>& ghosts, const std::string& scenario,
    NonlocalGpuVariant variant, const char* arithmetic_profile) {
    ScenarioResult result;
    result.name = scenario;
    result.arithmetic_profile = arithmetic_profile;
    auto cpu_state = corpus_initial(profile, scenario, false);
    const auto permuted_input = corpus_initial(profile, scenario, true);
    result.input_root = input_semantic_root(profile, cpu_state, ghosts);
    if (result.input_root.empty()
        || result.input_root
            != input_semantic_root(profile, permuted_input, ghosts)) {
        result.first_gate = "input_identity";
        seal_scenario_4k(result, {});
        return result;
    }
    NonlocalGpuWorkspace gpu(profile);
    NonlocalGpuWorkspace permuted(profile);
    const auto gpu_upload = gpu.upload(cpu_state, ghosts, true);
    const auto permuted_upload = permuted.upload(permuted_input, ghosts, true);
    if (gpu_upload != NonlocalGpuFailure::None
        || permuted_upload != NonlocalGpuFailure::None) {
        result.first_gate = "upload";
        seal_scenario_4k(result, {});
        return result;
    }
    const OperatorGate operator_gate =
        operator_gate_4k(profile, gpu, cpu_state, ghosts,
            variant);
    result.operator_root = operator_gate.root;
    result.operator_relative_l2 = operator_gate.relative_l2;
    result.operator_cosine_loss = operator_gate.cosine_loss;
    result.operator_active_signature_exact =
        operator_gate.active_signature_exact;
    result.operator_gpu_active_count = operator_gate.gpu_active_count;
    result.operator_cpu_active_count = operator_gate.cpu_active_count;
    result.operator_first_active_mismatch_id =
        operator_gate.first_active_mismatch_id;
    result.operator_first_gpu_mismatch_density =
        operator_gate.first_gpu_mismatch_density;
    result.operator_first_cpu_mismatch_density =
        operator_gate.first_cpu_mismatch_density;
    result.operator_density_relative_rmse =
        operator_gate.density_relative_rmse;
    result.operator_density_relative_maximum =
        operator_gate.density_relative_maximum;
    result.operator_gpu_work_root = operator_gate.gpu_work_root;
    result.operator_cpu_work_root = operator_gate.cpu_work_root;
    if (!operator_gate.valid) {
        result.first_gate = "same_state_operator";
        result.status = "PHYSICS_REFUTED_BOUNDED";
        seal_scenario_4k(result, {});
        return result;
    }
    const auto initial_evaluation = evaluate_reference(
        profile, cpu_state, ghosts, nullptr, NonlocalGpuVariant::Corrected);
    PhysicsMetrics previous_physics = physics_metrics_4k(
        profile, cpu_state, ghosts, initial_evaluation.density);
    const PhysicsMetrics initial_physics = previous_physics;
    if (initial_evaluation.failure != NonlocalGpuFailure::None
        || !initial_physics.valid) {
        result.first_gate = "cpu_initial_oracle";
        seal_scenario_4k(result, {});
        return result;
    }
    std::ostringstream receipts;
    receipts << "nextengine.nonlocal.ncgp9.scenario-receipts.v1\n"
             << scenario << ':' << result.input_root << ':'
             << result.operator_root << '\n';
    const double exact_mass = profile.mass * 4000.0;
    for (std::uint32_t step = 1U; step <= kCorpusSteps; ++step) {
        const auto gpu_step = gpu.step(kCorpusBudget,
            NonlocalGpuSolverProfile::Unpreconditioned,
            variant, false, false);
        const auto permuted_step = permuted.step(kCorpusBudget,
            NonlocalGpuSolverProfile::Unpreconditioned,
            variant, false, false);
        const auto gpu_snapshot = gpu.capture_public_snapshot();
        const auto permuted_snapshot = permuted.capture_public_snapshot();
        const auto gpu_compensated = gpu.capture_compensated_state();
        const auto permuted_compensated = permuted.capture_compensated_state();
        const auto cpu_step = step_reference(profile, cpu_state, ghosts,
            kCorpusBudget, NonlocalGpuVariant::Corrected, true);
        result.maximum_gpu_hvp = std::max(
            result.maximum_gpu_hvp, gpu_step.hvp_used);
        result.maximum_cpu_hvp = std::max(
            result.maximum_cpu_hvp, cpu_step.hvp_used);
        result.maximum_degree = std::max(
            result.maximum_degree, gpu_step.maximum_degree);
        result.maximum_directed_pairs = std::max(
            result.maximum_directed_pairs, gpu_step.maximum_directed_pairs);
        result.hot_host_to_device_bytes += gpu_step.work.host_to_device_bytes
            + permuted_step.work.host_to_device_bytes;
        result.hot_device_to_host_bytes += gpu_step.work.device_to_host_bytes
            + permuted_step.work.device_to_host_bytes;
        result.snapshot_device_to_host_bytes +=
            gpu_snapshot.work.device_to_host_bytes
            + permuted_snapshot.work.device_to_host_bytes;
        const bool gpu_failed = gpu_step.failure != NonlocalGpuFailure::None
            || permuted_step.failure != NonlocalGpuFailure::None;
        if (cpu_step.failure != NonlocalGpuFailure::None) {
            result.first_gate = "cpu_oracle";
            result.first_gate_step = step;
            break;
        }
        if (gpu_failed) {
            const bool matching = gpu_step.failure == permuted_step.failure
                && step_work_semantic_root(profile, gpu_step)
                    == step_work_semantic_root(profile, permuted_step)
                && compensated_state_root_4k(gpu_compensated)
                    == compensated_state_root_4k(permuted_compensated);
            result.first_gate = matching ? "gpu_solver_work" : "gpu_mismatch";
            result.first_gate_step = step;
            result.status = matching ? "PHYSICS_REFUTED_BOUNDED"
                                     : "APPARATUS_INCONCLUSIVE";
            break;
        }
        if (gpu_snapshot.failure != NonlocalGpuFailure::None
            || permuted_snapshot.failure != NonlocalGpuFailure::None
            || gpu_compensated.failure != NonlocalGpuFailure::None
            || permuted_compensated.failure != NonlocalGpuFailure::None
            || gpu_snapshot.state.size() != 4000U
            || cpu_step.state.size() != 4000U
            || permuted_snapshot.state.size() != 4000U
            || gpu_snapshot.density.size() != 4000U
            || cpu_step.density.size() != 4000U) {
            result.first_gate = "snapshot_or_count";
            result.first_gate_step = step;
            break;
        }
        const std::string gpu_state = state_root(gpu_snapshot.state,
            gpu_snapshot.density, gpu_snapshot.active_pressure_ids);
        const std::string permuted_state = state_root(permuted_snapshot.state,
            permuted_snapshot.density,
            permuted_snapshot.active_pressure_ids);
        if (gpu_state.empty() || gpu_state != permuted_state
            || step_work_semantic_root(profile, gpu_step)
                != step_work_semantic_root(profile, permuted_step)) {
            result.first_gate = "permutation";
            result.first_gate_step = step;
            break;
        }

        long double position_squared = 0.0L;
        long double density_squared = 0.0L;
        std::vector<double> position_errors;
        position_errors.reserve(4000U);
        for (std::size_t index = 0U; index < 4000U; ++index) {
            if (gpu_snapshot.state[index].sample_id
                    != cpu_step.state[index].sample_id
                || gpu_snapshot.state[index].sample_id
                    != permuted_snapshot.state[index].sample_id) {
                result.first_gate = "sample_identity";
                result.first_gate_step = step;
                break;
            }
            const Vec3d delta{
                gpu_snapshot.state[index].current.x
                    - cpu_step.state[index].current.x,
                gpu_snapshot.state[index].current.y
                    - cpu_step.state[index].current.y,
                gpu_snapshot.state[index].current.z
                    - cpu_step.state[index].current.z};
            const double error = vector_norm_4k(delta);
            position_errors.push_back(error);
            position_squared += static_cast<long double>(error) * error;
            result.maximum_position_max = std::max(
                result.maximum_position_max, error);
            const double density_error = std::abs(
                gpu_snapshot.density[index] - cpu_step.density[index])
                / profile.rest_density;
            density_squared += static_cast<long double>(density_error)
                * density_error;
            result.maximum_density_max = std::max(
                result.maximum_density_max, density_error);
        }
        if (!result.first_gate.empty()) break;
        std::sort(position_errors.begin(), position_errors.end());
        const double position_rmse = static_cast<double>(
            std::sqrt(position_squared / 4000.0L));
        const double position_p99 = position_errors[
            nearest_rank_index(position_errors.size(), 99U, 100U)];
        result.maximum_position_rmse = std::max(
            result.maximum_position_rmse, position_rmse);
        result.maximum_position_p99 = std::max(
            result.maximum_position_p99, position_p99);
        const double density_rmse = static_cast<double>(
            std::sqrt(density_squared / 4000.0L));
        result.maximum_density_rmse = std::max(
            result.maximum_density_rmse, density_rmse);
        if (step == 1U) {
            result.first_step_position_max = position_errors.back();
            if (result.first_step_position_max > 5.0e-6
                || gpu_snapshot.active_pressure_ids
                    != cpu_step.active_pressure_ids) {
                result.first_gate = result.first_step_position_max > 5.0e-6
                    ? "same_state_position"
                    : "same_state_active_signature";
            }
        }

        const PhysicsMetrics physics = physics_metrics_4k(
            profile, gpu_snapshot.state, ghosts, gpu_snapshot.density);
        if (!physics.valid) {
            result.first_gate = "physics_oracle";
        } else {
            const Vec3d momentum_residual{
                physics.momentum.x - previous_physics.momentum.x
                    - profile.dt
                        * (exact_mass * profile.gravity.x
                            + physics.ghost_pressure_force.x)
                    - gpu_step.boundary_impulse.x,
                physics.momentum.y - previous_physics.momentum.y
                    - profile.dt
                        * (exact_mass * profile.gravity.y
                            + physics.ghost_pressure_force.y)
                    - gpu_step.boundary_impulse.y,
                physics.momentum.z - previous_physics.momentum.z
                    - profile.dt
                        * (exact_mass * profile.gravity.z
                            + physics.ghost_pressure_force.z)
                    - gpu_step.boundary_impulse.z};
            const double momentum_scale = std::max({
                vector_norm_4k(previous_physics.momentum),
                profile.dt * exact_mass * vector_norm_4k(profile.gravity),
                profile.spacing * exact_mass / profile.dt});
            result.maximum_momentum_residual = std::max(
                result.maximum_momentum_residual,
                vector_norm_4k(momentum_residual) / momentum_scale);
            result.maximum_positive_energy_excess = std::max(
                result.maximum_positive_energy_excess,
                std::max(physics.mechanical_energy
                        - initial_physics.mechanical_energy,
                    0.0)
                    / std::max(std::abs(initial_physics.mechanical_energy), 1.0));
            previous_physics = physics;
        }
        result.maximum_penetration = std::max(
            result.maximum_penetration, gpu_step.maximum_penetration_m);

        const auto gpu_image = build_visible_surface(
            profile, gpu_snapshot.state, 4000U, exact_mass);
        const auto cpu_image = build_visible_surface(
            profile, cpu_step.state, 4000U, exact_mass);
        const auto permuted_image = build_visible_surface(
            profile, permuted_snapshot.state, 4000U, exact_mass);
        const auto visible = compare_visible_surfaces(gpu_image, cpu_image);
        if (!gpu_image.valid || !cpu_image.valid || !permuted_image.valid
            || !visible.valid
            || gpu_image.image_root != permuted_image.image_root) {
            result.first_gate = "visible_apparatus";
        } else {
            ++result.visible_steps;
            result.maximum_silhouette = std::max(result.maximum_silhouette,
                visible.silhouette_symmetric_difference);
            result.maximum_depth_rmse = std::max(
                result.maximum_depth_rmse, visible.depth_rmse);
            result.maximum_depth_p95 = std::max(
                result.maximum_depth_p95, visible.depth_p95);
            result.maximum_depth_p99 = std::max(
                result.maximum_depth_p99, visible.depth_p99);
            result.maximum_depth_max = std::max(
                result.maximum_depth_max, visible.depth_maximum);
            result.maximum_component_fraction_difference = std::max(
                result.maximum_component_fraction_difference,
                visible.largest_component_fraction_difference);
            const double satellite_difference = std::abs(
                visible.gpu_satellite_area_fraction
                - visible.cpu_satellite_area_fraction);
            result.maximum_satellite_fraction_difference = std::max(
                result.maximum_satellite_fraction_difference,
                satellite_difference);
            result.maximum_cpu_satellite_fraction = std::max(
                result.maximum_cpu_satellite_fraction,
                visible.cpu_satellite_area_fraction);
            result.maximum_gpu_satellite_fraction = std::max(
                result.maximum_gpu_satellite_fraction,
                visible.gpu_satellite_area_fraction);
            if (visible.silhouette_symmetric_difference > 0.01) {
                result.first_gate = "visible_silhouette";
            } else if (visible.depth_rmse > 0.00625) {
                result.first_gate = "visible_depth_rmse";
            } else if (visible.depth_p95 > 0.0125) {
                result.first_gate = "visible_depth_p95";
            } else if (visible.depth_p99 > 0.025) {
                result.first_gate = "visible_depth_p99";
            } else if (visible.cpu_material_components
                != visible.gpu_material_components) {
                result.first_gate = "visible_components";
            } else if (visible.largest_component_fraction_difference > 0.01) {
                result.first_gate = "visible_largest_component";
            } else if (satellite_difference > 0.01) {
                result.first_gate = "visible_satellite_difference";
            } else if (scenario == "hydrostatic-hold"
                && (visible.cpu_satellite_area_fraction > 0.01
                    || visible.gpu_satellite_area_fraction > 0.01)) {
                result.first_gate = "visible_hydro_satellite";
            }
        }

        std::string bulk_root = "none";
        if (result.first_gate.empty() && is_bulk_checkpoint(step)) {
            const auto bulk =
                nextengine::nonlocal::ncgp8::evaluate_retained_bulk(profile,
                    gpu_snapshot.state, gpu_snapshot.density, cpu_step.state,
                    cpu_step.density, permuted_snapshot.state,
                    permuted_snapshot.density);
            bulk_root = bulk.closure_root;
            if (!bulk_checkpoint_valid(bulk)) {
                result.first_gate = "bulk_field";
            } else {
                ++result.bulk_checkpoints;
            }
        }

        if (result.first_gate.empty()) {
            if (density_rmse > 0.05) {
                result.first_gate = "density_rmse";
            } else if (result.maximum_density_max > 0.10) {
                result.first_gate = "density_max";
            } else if (result.maximum_momentum_residual > 0.01) {
                result.first_gate = "momentum";
            } else if (result.maximum_positive_energy_excess > 0.01) {
                result.first_gate = "energy_excess";
            } else if (result.maximum_penetration > 0.0025) {
                result.first_gate = "penetration";
            }
        }
        receipts << std::setprecision(17) << step << ':'
                 << step_work_semantic_root(profile, gpu_step) << ':'
                 << step_work_semantic_root(profile, permuted_step) << ':'
                 << step_work_semantic_root(profile, cpu_step) << ':'
                 << gpu_state << ':'
                 << state_root(cpu_step.state, cpu_step.density,
                        cpu_step.active_pressure_ids)
                 << ':' << gpu_image.image_root << ':' << cpu_image.image_root
                 << ':' << visible.metrics_root << ':' << bulk_root << ':'
                 << bits(position_rmse) << ':' << bits(position_p99) << ':'
                 << bits(density_rmse) << ':'
                 << bits(result.maximum_momentum_residual) << ':'
                 << bits(result.maximum_positive_energy_excess) << '\n';
        if (!result.first_gate.empty()) {
            result.first_gate_step = step;
            result.failure_gpu_state_root = gpu_state;
            result.failure_cpu_state_root = state_root(cpu_step.state,
                cpu_step.density, cpu_step.active_pressure_ids);
            result.failure_permuted_state_root = permuted_state;
            if (visible.valid && gpu_image.valid && cpu_image.valid
                && permuted_image.valid) {
                result.failure_cpu_satellite_fraction =
                    visible.cpu_satellite_area_fraction;
                result.failure_gpu_satellite_fraction =
                    visible.gpu_satellite_area_fraction;
                result.failure_cpu_largest_component_fraction =
                    cpu_image.largest_component_fraction;
                result.failure_gpu_largest_component_fraction =
                    gpu_image.largest_component_fraction;
                result.failure_cpu_components =
                    visible.cpu_material_components;
                result.failure_gpu_components =
                    visible.gpu_material_components;
                result.failure_cpu_wet_pixels = cpu_image.work.wet_pixels;
                result.failure_gpu_wet_pixels = gpu_image.work.wet_pixels;
                result.failure_gpu_image_root = gpu_image.image_root;
                result.failure_cpu_image_root = cpu_image.image_root;
                result.failure_permuted_image_root =
                    permuted_image.image_root;
                result.failure_visible_present = true;
                result.failure_gpu_image = gpu_image;
                result.failure_cpu_image = cpu_image;
                result.failure_permuted_image = permuted_image;
                result.failure_visible = visible;
            }
            if (result.status != "APPARATUS_INCONCLUSIVE") {
                result.status = "PHYSICS_REFUTED_BOUNDED";
            } else if (result.first_gate.find("apparatus") == std::string::npos
                && result.first_gate != "sample_identity"
                && result.first_gate != "permutation") {
                result.status = "PHYSICS_REFUTED_BOUNDED";
            }
            break;
        }
        cpu_state = cpu_step.state;
        ++result.completed_steps;
        result.final_gpu_state_root = gpu_state;
        result.final_cpu_state_root = state_root(
            cpu_step.state, cpu_step.density, cpu_step.active_pressure_ids);
        result.final_permuted_state_root = permuted_state;
    }
    if (result.completed_steps == kCorpusSteps && result.visible_steps == 240U
        && result.bulk_checkpoints == 4U && result.first_gate.empty()) {
        result.status = "PASS";
    }
    seal_scenario_4k(result, receipts.str());
    return result;
}

void emit_scenario(const ScenarioResult& value) {
    std::cout << std::setprecision(17) << "{\"name\":\"" << value.name
              << "\",\"arithmetic_profile\":\""
              << value.arithmetic_profile
              << "\",\"status\":\"" << value.status
              << "\",\"first_gate\":\"" << value.first_gate
              << "\",\"first_gate_step\":" << value.first_gate_step
              << ",\"completed_steps\":" << value.completed_steps
              << ",\"visible_steps\":" << value.visible_steps
              << ",\"bulk_checkpoints\":" << value.bulk_checkpoints
              << ",\"first_step_position_max_m\":"
              << value.first_step_position_max
              << ",\"maximum_position_rmse_m\":"
              << value.maximum_position_rmse
              << ",\"maximum_position_p99_m\":"
              << value.maximum_position_p99
              << ",\"maximum_position_max_m\":"
              << value.maximum_position_max
              << ",\"maximum_density_rmse\":"
              << value.maximum_density_rmse
              << ",\"maximum_density_max\":" << value.maximum_density_max
              << ",\"maximum_momentum_residual\":"
              << value.maximum_momentum_residual
              << ",\"maximum_positive_energy_excess\":"
              << value.maximum_positive_energy_excess
              << ",\"maximum_penetration_m\":" << value.maximum_penetration
              << ",\"maximum_silhouette\":" << value.maximum_silhouette
              << ",\"maximum_depth_rmse_m\":" << value.maximum_depth_rmse
              << ",\"maximum_depth_p95_m\":" << value.maximum_depth_p95
              << ",\"maximum_depth_p99_m\":" << value.maximum_depth_p99
              << ",\"maximum_depth_max_m\":" << value.maximum_depth_max
              << ",\"maximum_component_fraction_difference\":"
              << value.maximum_component_fraction_difference
              << ",\"maximum_satellite_fraction_difference\":"
              << value.maximum_satellite_fraction_difference
              << ",\"maximum_cpu_satellite_fraction\":"
              << value.maximum_cpu_satellite_fraction
              << ",\"maximum_gpu_satellite_fraction\":"
              << value.maximum_gpu_satellite_fraction
              << ",\"failure_cpu_satellite_fraction\":"
              << value.failure_cpu_satellite_fraction
              << ",\"failure_gpu_satellite_fraction\":"
              << value.failure_gpu_satellite_fraction
              << ",\"failure_cpu_largest_component_fraction\":"
              << value.failure_cpu_largest_component_fraction
              << ",\"failure_gpu_largest_component_fraction\":"
              << value.failure_gpu_largest_component_fraction
              << ",\"failure_cpu_components\":"
              << value.failure_cpu_components
              << ",\"failure_gpu_components\":"
              << value.failure_gpu_components
              << ",\"failure_cpu_wet_pixels\":"
              << value.failure_cpu_wet_pixels
              << ",\"failure_gpu_wet_pixels\":"
              << value.failure_gpu_wet_pixels
              << ",\"maximum_gpu_hvp\":" << value.maximum_gpu_hvp
              << ",\"maximum_cpu_hvp\":" << value.maximum_cpu_hvp
              << ",\"maximum_degree\":" << value.maximum_degree
              << ",\"maximum_directed_pairs\":"
              << value.maximum_directed_pairs
              << ",\"hot_host_to_device_bytes\":"
              << value.hot_host_to_device_bytes
              << ",\"hot_device_to_host_bytes\":"
              << value.hot_device_to_host_bytes
              << ",\"snapshot_device_to_host_bytes\":"
              << value.snapshot_device_to_host_bytes
              << ",\"operator_relative_l2\":"
              << value.operator_relative_l2
              << ",\"operator_cosine_loss\":"
              << value.operator_cosine_loss
              << ",\"operator_active_signature_exact\":"
              << (value.operator_active_signature_exact ? "true" : "false")
              << ",\"operator_gpu_active_count\":"
              << value.operator_gpu_active_count
              << ",\"operator_cpu_active_count\":"
              << value.operator_cpu_active_count
              << ",\"operator_first_active_mismatch_id\":"
              << value.operator_first_active_mismatch_id
              << ",\"operator_first_gpu_mismatch_density\":"
              << value.operator_first_gpu_mismatch_density
              << ",\"operator_first_cpu_mismatch_density\":"
              << value.operator_first_cpu_mismatch_density
              << ",\"operator_density_relative_rmse\":"
              << value.operator_density_relative_rmse
              << ",\"operator_density_relative_maximum\":"
              << value.operator_density_relative_maximum
              << ",\"input_root\":\"" << value.input_root
              << "\",\"operator_root\":\"" << value.operator_root
              << "\",\"operator_gpu_work_root\":\""
              << value.operator_gpu_work_root
              << "\",\"operator_cpu_work_root\":\""
              << value.operator_cpu_work_root
              << "\",\"receipt_root\":\"" << value.receipt_root
              << "\",\"final_gpu_state_root\":\""
              << value.final_gpu_state_root
              << "\",\"final_cpu_state_root\":\""
              << value.final_cpu_state_root
              << "\",\"final_permuted_state_root\":\""
              << value.final_permuted_state_root
              << "\",\"failure_gpu_state_root\":\""
              << value.failure_gpu_state_root
              << "\",\"failure_cpu_state_root\":\""
              << value.failure_cpu_state_root
              << "\",\"failure_permuted_state_root\":\""
              << value.failure_permuted_state_root
              << "\",\"failure_gpu_image_root\":\""
              << value.failure_gpu_image_root
              << "\",\"failure_cpu_image_root\":\""
              << value.failure_cpu_image_root
              << "\",\"failure_permuted_image_root\":\""
              << value.failure_permuted_image_root
              << "\",\"result_root\":\"" << value.result_root << "\"";
    if (value.failure_visible_present) {
        std::cout << ",\"failure_visible\":{";
        std::cout << "\"gpu_image\":";
        emit_image(value.failure_gpu_image);
        std::cout << ",\"cpu_image\":";
        emit_image(value.failure_cpu_image);
        std::cout << ",\"permuted_image\":";
        emit_image(value.failure_permuted_image);
        std::cout << ",\"comparison\":";
        emit_comparison(value.failure_visible);
        std::cout << '}';
    }
    std::cout << '}';
}

void emit_operator_diagnostic_4k(
    const char* name, const OperatorGate& value);

int run_complete_corpus(bool pressure_f64) {
    const NonlocalGpuProfile profile = nonlocal_water_corrected_profile();
    const auto ghosts = canonicalize_ghosts_binary32(make_basin_ghosts(profile));
    const bool physics_controls = run_ncgp3_physics_self_test(false) == 0;
    OperatorGate primary_discriminator;
    OperatorGate pressure_discriminator;
    bool discriminator_passed = !pressure_f64;
    if (pressure_f64 && physics_controls) {
        const auto state = corpus_initial(profile, "hydrostatic-hold", false);
        NonlocalGpuWorkspace primary_workspace(profile);
        NonlocalGpuWorkspace pressure_workspace(profile);
        const bool uploaded = primary_workspace.upload(state, ghosts, true)
                == NonlocalGpuFailure::None
            && pressure_workspace.upload(state, ghosts, true)
                == NonlocalGpuFailure::None;
        if (uploaded) {
            primary_discriminator = operator_gate_4k(profile,
                primary_workspace, state, ghosts,
                NonlocalGpuVariant::CompensatedScaleF32);
            pressure_discriminator = operator_gate_4k(profile,
                pressure_workspace, state, ghosts,
                NonlocalGpuVariant::CompensatedScalePressureF64);
            discriminator_passed = !primary_discriminator.valid
                && pressure_discriminator.valid
                && primary_discriminator.gpu_active_count == 1362U
                && primary_discriminator.cpu_active_count == 978U
                && pressure_discriminator.gpu_active_count == 978U
                && pressure_discriminator.cpu_active_count == 978U;
        }
    }
    const NonlocalGpuVariant variant = pressure_f64
        ? NonlocalGpuVariant::CompensatedScalePressureF64
        : NonlocalGpuVariant::CompensatedScaleF32;
    const char* arithmetic_profile = pressure_f64
        ? "f32-state-f64-pressure" : "f32-primary";
    std::vector<ScenarioResult> scenarios;
    const std::array<std::string, 3> order{
        "hydrostatic-hold", "dam-break", "orifice-jet"};
    for (const auto& name : order) {
        if (!physics_controls || !discriminator_passed) break;
        scenarios.push_back(run_scenario(
            profile, ghosts, name, variant, arithmetic_profile));
        if (scenarios.back().status != "PASS") break;
    }
    std::string status = "APPARATUS_INCONCLUSIVE";
    if (physics_controls && discriminator_passed
        && scenarios.size() == order.size()
        && std::all_of(scenarios.begin(), scenarios.end(), [](const auto& value) {
               return value.status == "PASS";
           })) {
        status = "CORPUS_PASS";
    } else if (!scenarios.empty()
        && scenarios.back().status == "PHYSICS_REFUTED_BOUNDED") {
        status = "PHYSICS_REFUTED_BOUNDED";
    }
    const std::string executable_root = binary_root();
    NonlocalGpuWorkspace environment_workspace(profile);
    const std::string environment = environment_workspace.environment_json();
    std::ostringstream material;
    material << (pressure_f64
            ? "nextengine.nonlocal.ncgp10.corpus-result.v1\n"
            : "nextengine.nonlocal.ncgp9.corpus-result.v1\n")
             << status << ':' << arithmetic_profile << ':' << physics_controls
             << ':' << discriminator_passed << ':' << scenarios.size() << '\n';
    for (const auto& scenario : scenarios) {
        material << scenario.name << ':' << scenario.result_root << '\n';
    }
    material << primary_discriminator.root << ':'
             << pressure_discriminator.root << ':'
             << kReviewedNcgp8ResultRoot << ':'
             << (pressure_f64 ? NCGP10_CONTRACT_ROOT : NCGP9_CONTRACT_ROOT)
             << ':'
             << (pressure_f64 ? NCGP10_SOURCE_ROOT : NCGP9_SOURCE_ROOT) << ':'
             << (pressure_f64 ? NCGP10_SOURCE_COMMIT : NCGP9_SOURCE_COMMIT)
             << ':' << (pressure_f64 ? NCGP10_SOURCE_TREE : NCGP9_SOURCE_TREE)
             << ':'
             << (pressure_f64 ? NCGP10_COMPILER_FLAGS : NCGP9_COMPILER_FLAGS)
             << ':'
             << executable_root << ':' << environment << '\n';
    const std::string result_root =
        nextengine::nonlocal::sha256_hex(material.str());
    std::cout << "{\"schema\":\""
              << (pressure_f64
                      ? "nextengine.nonlocal.ncgp10.complete-4k.v1"
                      : "nextengine.nonlocal.ncgp9.complete-4k.v1")
              << "\""
              << ",\"status\":\"" << status << "\""
              << ",\"arithmetic_profile\":\"" << arithmetic_profile << "\""
              << ",\"physics_controls\":"
              << (physics_controls ? "true" : "false")
              << ",\"discriminator_passed\":"
              << (discriminator_passed ? "true" : "false");
    if (pressure_f64) {
        std::cout << ",\"operator_discriminator\":{";
        emit_operator_diagnostic_4k("primary_f32", primary_discriminator);
        std::cout << ',';
        emit_operator_diagnostic_4k(
            "pressure_f64", pressure_discriminator);
        std::cout << '}';
    }
    std::cout
              << ",\"scenarios\":[";
    for (std::size_t index = 0U; index < scenarios.size(); ++index) {
        if (index != 0U) std::cout << ',';
        emit_scenario(scenarios[index]);
    }
    std::cout << "]\n,\"parent_ncgp8_result_root\":\""
              << kReviewedNcgp8ResultRoot << "\",\"result_root\":\""
              << result_root << "\",\"contract_root\":\""
              << (pressure_f64 ? NCGP10_CONTRACT_ROOT : NCGP9_CONTRACT_ROOT)
              << "\",\"source_root\":\""
              << (pressure_f64 ? NCGP10_SOURCE_ROOT : NCGP9_SOURCE_ROOT)
              << "\",\"source_commit\":\""
              << (pressure_f64 ? NCGP10_SOURCE_COMMIT : NCGP9_SOURCE_COMMIT)
              << "\",\"source_tree\":\""
              << (pressure_f64 ? NCGP10_SOURCE_TREE : NCGP9_SOURCE_TREE)
              << "\",\"compiler_flags\":\""
              << (pressure_f64 ? NCGP10_COMPILER_FLAGS : NCGP9_COMPILER_FLAGS)
              << "\",\"binary_root\":\""
              << executable_root << "\",\"environment\":" << environment
              << ",\"allocated_device_bytes\":"
              << environment_workspace.allocated_device_bytes()
              << ",\"exact_command\":\""
              << (pressure_f64 ? "--complete-4k-pressure-f64-corpus"
                               : "--complete-4k-corpus")
              << "\""
              << ",\"performance_status\":\"NOT_RUN\"}\n";
    return status == "CORPUS_PASS" ? 0
        : (status == "PHYSICS_REFUTED_BOUNDED" ? 37 : 4);
}

int emit_constructor_failure(const char* message);

void emit_operator_diagnostic_4k(
    const char* name, const OperatorGate& value) {
    std::cout << std::setprecision(17) << "\"" << name << "\":{";
    std::cout << "\"valid\":" << (value.valid ? "true" : "false")
              << ",\"relative_l2\":" << value.relative_l2
              << ",\"cosine_loss\":" << value.cosine_loss
              << ",\"active_signature_exact\":"
              << (value.active_signature_exact ? "true" : "false")
              << ",\"gpu_active_count\":" << value.gpu_active_count
              << ",\"cpu_active_count\":" << value.cpu_active_count
              << ",\"first_active_mismatch_id\":"
              << value.first_active_mismatch_id
              << ",\"first_gpu_mismatch_density\":"
              << value.first_gpu_mismatch_density
              << ",\"first_cpu_mismatch_density\":"
              << value.first_cpu_mismatch_density
              << ",\"density_relative_rmse\":"
              << value.density_relative_rmse
              << ",\"density_relative_maximum\":"
              << value.density_relative_maximum
              << ",\"gpu_work_root\":\"" << value.gpu_work_root
              << "\",\"cpu_work_root\":\"" << value.cpu_work_root
              << "\",\"root\":\"" << value.root << "\"}";
}

int run_operator_discriminator_4k() {
    const NonlocalGpuProfile profile = nonlocal_water_corrected_profile();
    const auto ghosts = canonicalize_ghosts_binary32(make_basin_ghosts(profile));
    const auto state = corpus_initial(profile, "hydrostatic-hold", false);
    NonlocalGpuWorkspace primary_workspace(profile);
    NonlocalGpuWorkspace pressure_workspace(profile);
    if (primary_workspace.upload(state, ghosts, true)
            != NonlocalGpuFailure::None
        || pressure_workspace.upload(state, ghosts, true)
            != NonlocalGpuFailure::None) {
        return emit_constructor_failure("operator-discriminator-upload");
    }
    const OperatorGate primary = operator_gate_4k(profile, primary_workspace,
        state, ghosts, NonlocalGpuVariant::CompensatedScaleF32);
    const OperatorGate pressure = operator_gate_4k(profile, pressure_workspace,
        state, ghosts, NonlocalGpuVariant::CompensatedScalePressureF64);
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp9.operator-discriminator.v1\n"
             << primary.root << ':' << pressure.root << ':'
             << NCGP9_CONTRACT_ROOT << ':' << NCGP9_SOURCE_ROOT << ':'
             << binary_root() << '\n';
    std::cout << "{\"schema\":\"nextengine.nonlocal.ncgp9.operator-discriminator.v1\",";
    emit_operator_diagnostic_4k("primary_f32", primary);
    std::cout << ',';
    emit_operator_diagnostic_4k("pressure_f64", pressure);
    std::cout << ",\"result_root\":\""
              << nextengine::nonlocal::sha256_hex(material.str())
              << "\",\"contract_root\":\"" << NCGP9_CONTRACT_ROOT
              << "\",\"source_root\":\"" << NCGP9_SOURCE_ROOT
              << "\",\"binary_root\":\"" << binary_root() << "\"}\n";
    return pressure.valid ? 0 : 37;
}

int emit_constructor_failure(const char* message) {
    const std::string executable_root = binary_root();
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp9.failure.v1\nconstructor:"
             << message << ':' << NCGP9_CONTRACT_ROOT << ':'
             << NCGP9_SOURCE_ROOT << ':' << NCGP9_SOURCE_COMMIT << ':'
             << NCGP9_SOURCE_TREE << ':' << executable_root << '\n';
    const std::string result_root =
        nextengine::nonlocal::sha256_hex(material.str());
    std::cout << "{\"schema\":\"nextengine.nonlocal.ncgp9.failure.v1\""
              << ",\"status\":\"APPARATUS_INCONCLUSIVE\""
              << ",\"stage\":\"constructor\",\"message\":\"" << message
              << "\",\"result_root\":\"" << result_root
              << "\",\"contract_root\":\"" << NCGP9_CONTRACT_ROOT
              << "\",\"source_root\":\"" << NCGP9_SOURCE_ROOT
              << "\",\"source_commit\":\"" << NCGP9_SOURCE_COMMIT
              << "\",\"source_tree\":\"" << NCGP9_SOURCE_TREE
              << "\",\"binary_root\":\"" << executable_root << "\"}\n";
    return 4;
}

}  // namespace

int main(int argc, char** argv) {
    if (argc == 2
        && std::string(argv[1]) == "--diagnose-4k-operator-pressure") {
        try {
            return run_operator_discriminator_4k();
        } catch (const std::exception& exception) {
            return emit_constructor_failure(exception.what());
        } catch (...) {
            return emit_constructor_failure("unknown");
        }
    }
    if (argc == 2
        && std::string(argv[1]) == "--complete-4k-pressure-f64-corpus") {
        try {
            return run_complete_corpus(true);
        } catch (const std::exception& exception) {
            return emit_constructor_failure(exception.what());
        } catch (...) {
            return emit_constructor_failure("unknown");
        }
    }
    if (argc != 2 || std::string(argv[1]) != "--complete-4k-corpus") {
        std::cerr << "usage: nonlocal-corrected-cuda-complete-4k "
                     "--complete-4k-corpus|"
                     "--complete-4k-pressure-f64-corpus|"
                     "--diagnose-4k-operator-pressure\n";
        return 2;
    }
    try {
        return run_complete_corpus(false);
    } catch (const std::exception& exception) {
        return emit_constructor_failure(exception.what());
    } catch (...) {
        return emit_constructor_failure("unknown");
    }
}
