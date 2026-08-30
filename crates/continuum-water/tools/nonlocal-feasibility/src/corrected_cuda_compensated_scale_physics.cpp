#include "corrected_cuda_compensated_scale_physics.hpp"

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
#include <vector>

#ifndef NCGP3_CONTRACT_ROOT
#define NCGP3_CONTRACT_ROOT "unconfigured"
#endif
#ifndef NCGP3_SOURCE_ROOT
#define NCGP3_SOURCE_ROOT "unconfigured"
#endif
#ifndef NCGP4_CONTRACT_ROOT
#define NCGP4_CONTRACT_ROOT NCGP3_CONTRACT_ROOT
#endif
#ifndef NCGP4_SOURCE_ROOT
#define NCGP4_SOURCE_ROOT NCGP3_SOURCE_ROOT
#endif
#ifndef NCGP4_SOURCE_COMMIT
#define NCGP4_SOURCE_COMMIT "unconfigured"
#endif
#ifndef NCGP4_SOURCE_TREE
#define NCGP4_SOURCE_TREE "unconfigured"
#endif
#ifndef NCGP4_COMPILER_FLAGS
#define NCGP4_COMPILER_FLAGS "unconfigured"
#endif

namespace {

using namespace nextengine::nonlocal::gpu_full_step;

constexpr std::uint32_t kBudget = 128U;

std::string physics_binary_root() {
    std::ifstream stream("/proc/self/exe", std::ios::binary);
    if (!stream) return {};
    std::ostringstream bytes;
    bytes << stream.rdbuf();
    return nextengine::nonlocal::sha256_hex(bytes.str());
}

double squared_norm(Vec3d value) {
    return value.x * value.x + value.y * value.y + value.z * value.z;
}

Vec3d subtract(Vec3d lhs, Vec3d rhs) {
    return {lhs.x - rhs.x, lhs.y - rhs.y, lhs.z - rhs.z};
}

double distance(Vec3d lhs, Vec3d rhs) {
    return std::sqrt(squared_norm(subtract(lhs, rhs)));
}

std::uint32_t float_bits(float value) {
    std::uint32_t bits = 0U;
    static_assert(sizeof(bits) == sizeof(value));
    std::memcpy(&bits, &value, sizeof(bits));
    return bits;
}

std::string snapshot_state_root(const NonlocalGpuPublicSnapshot& snapshot) {
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp3.physics-state.v1\n"
             << static_cast<std::uint32_t>(snapshot.failure) << '\n';
    for (const NonlocalGpuSample& sample : snapshot.state) {
        material << sample.sample_id << ':' << std::hex
                 << float_bits(static_cast<float>(sample.current.x)) << ','
                 << float_bits(static_cast<float>(sample.current.y)) << ','
                 << float_bits(static_cast<float>(sample.current.z)) << ';'
                 << float_bits(static_cast<float>(sample.velocity.x)) << ','
                 << float_bits(static_cast<float>(sample.velocity.y)) << ','
                 << float_bits(static_cast<float>(sample.velocity.z)) << '\n'
                 << std::dec;
    }
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string compensated_state_root(
    const NonlocalGpuCompensatedStateSnapshot& snapshot) {
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp3.physics-pair.v1\n"
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
            const Vec3d value = (*field)[index];
            material << std::hex
                     << float_bits(static_cast<float>(value.x)) << ','
                     << float_bits(static_cast<float>(value.y)) << ','
                     << float_bits(static_cast<float>(value.z)) << ';'
                     << std::dec;
        }
        material << '\n';
    }
    return nextengine::nonlocal::sha256_hex(material.str());
}

struct FreeFallResult {
    bool passed = false;
    double maximum_position_error = std::numeric_limits<double>::infinity();
    double maximum_velocity_error = std::numeric_limits<double>::infinity();
    double positive_energy_excess = std::numeric_limits<double>::infinity();
    double reverse_position_error = std::numeric_limits<double>::infinity();
    double reverse_velocity_error = std::numeric_limits<double>::infinity();
    double reversible_energy_drift = std::numeric_limits<double>::infinity();
    double momentum_residual = std::numeric_limits<double>::infinity();
    std::string input_root;
    std::string receipt_root;
};

FreeFallResult run_free_fall() {
    FreeFallResult result;
    NonlocalGpuProfile profile = nonlocal_water_corrected_profile();
    const Vec3d initial_position{0.75, 0.75, 1.0};
    const Vec3d initial_velocity{0.03125, -0.015625, 0.0625};
    const auto input = canonicalize_samples_binary32({
        {101U, initial_position, initial_position, initial_velocity}});
    result.input_root = input_semantic_root(profile, input, {});
    NonlocalGpuWorkspace workspace(profile);
    if (workspace.upload(input, {}, true) != NonlocalGpuFailure::None) {
        return result;
    }
    const double initial_energy = 0.5 * profile.mass
            * squared_norm(input[0].velocity)
        - profile.mass * (profile.gravity.x * input[0].current.x
            + profile.gravity.y * input[0].current.y
            + profile.gravity.z * input[0].current.z);
    double maximum_energy_delta = 0.0;
    double maximum_momentum = 0.0;
    double maximum_position = 0.0;
    double maximum_velocity = 0.0;
    std::vector<NonlocalGpuSample> forward_state;
    std::ostringstream receipts;
    receipts << "nextengine.nonlocal.ncgp3.free-fall.v1\n";
    for (std::uint32_t step = 1U; step <= 32U; ++step) {
        const auto solved = workspace.step(kBudget,
            NonlocalGpuSolverProfile::Jacobi,
            NonlocalGpuVariant::CompensatedScaleF32, false, false);
        const auto snapshot = workspace.capture_public_snapshot();
        receipts << step_work_semantic_root(profile, solved) << ':'
                 << work_semantic_root(snapshot.work) << '\n';
        if (solved.failure != NonlocalGpuFailure::None
            || snapshot.failure != NonlocalGpuFailure::None
            || snapshot.state.size() != 1U) {
            result.receipt_root = nextengine::nonlocal::sha256_hex(receipts.str());
            return result;
        }
        const double n = static_cast<double>(step);
        const double dt = profile.dt;
        const Vec3d expected_velocity{
            input[0].velocity.x + n * dt * profile.gravity.x,
            input[0].velocity.y + n * dt * profile.gravity.y,
            input[0].velocity.z + n * dt * profile.gravity.z};
        const double position_factor = 0.5 * n * (n + 1.0) * dt * dt;
        const Vec3d expected_position{
            input[0].current.x + n * dt * input[0].velocity.x
                + position_factor * profile.gravity.x,
            input[0].current.y + n * dt * input[0].velocity.y
                + position_factor * profile.gravity.y,
            input[0].current.z + n * dt * input[0].velocity.z
                + position_factor * profile.gravity.z};
        maximum_position = std::max(maximum_position,
            distance(snapshot.state[0].current, expected_position));
        maximum_velocity = std::max(maximum_velocity,
            distance(snapshot.state[0].velocity, expected_velocity));
        const double energy = 0.5 * profile.mass
                * squared_norm(snapshot.state[0].velocity)
            - profile.mass * (profile.gravity.x * snapshot.state[0].current.x
                + profile.gravity.y * snapshot.state[0].current.y
                + profile.gravity.z * snapshot.state[0].current.z);
        maximum_energy_delta = std::max(maximum_energy_delta,
            std::max(energy - initial_energy, 0.0));
        const Vec3d expected_momentum{
            profile.mass * expected_velocity.x,
            profile.mass * expected_velocity.y,
            profile.mass * expected_velocity.z};
        const Vec3d actual_momentum{
            profile.mass * snapshot.state[0].velocity.x,
            profile.mass * snapshot.state[0].velocity.y,
            profile.mass * snapshot.state[0].velocity.z};
        maximum_momentum = std::max(maximum_momentum,
            distance(actual_momentum, expected_momentum));
        forward_state = snapshot.state;
    }
    result.maximum_position_error = maximum_position;
    result.maximum_velocity_error = maximum_velocity;
    result.positive_energy_excess = maximum_energy_delta
        / std::max(std::abs(initial_energy), 1.0);
    if (forward_state.size() != 1U) {
        result.receipt_root = nextengine::nonlocal::sha256_hex(receipts.str());
        return result;
    }
    auto reverse_input = forward_state;
    reverse_input[0].reference = reverse_input[0].current;
    reverse_input[0].velocity = {
        -reverse_input[0].velocity.x - profile.dt * profile.gravity.x,
        -reverse_input[0].velocity.y - profile.dt * profile.gravity.y,
        -reverse_input[0].velocity.z - profile.dt * profile.gravity.z};
    reverse_input = canonicalize_samples_binary32(reverse_input);
    NonlocalGpuWorkspace reverse_workspace(profile);
    if (reverse_workspace.upload(reverse_input, {}, true)
        != NonlocalGpuFailure::None) {
        result.receipt_root = nextengine::nonlocal::sha256_hex(receipts.str());
        return result;
    }
    NonlocalGpuPublicSnapshot reverse_snapshot;
    for (std::uint32_t step = 1U; step <= 32U; ++step) {
        const auto solved = reverse_workspace.step(kBudget,
            NonlocalGpuSolverProfile::Jacobi,
            NonlocalGpuVariant::CompensatedScaleF32, false, false);
        reverse_snapshot = reverse_workspace.capture_public_snapshot();
        receipts << "reverse:" << step << ':'
                 << step_work_semantic_root(profile, solved) << ':'
                 << work_semantic_root(reverse_snapshot.work) << '\n';
        if (solved.failure != NonlocalGpuFailure::None
            || reverse_snapshot.failure != NonlocalGpuFailure::None
            || reverse_snapshot.state.size() != 1U) {
            result.receipt_root = nextengine::nonlocal::sha256_hex(
                receipts.str());
            return result;
        }
    }
    const Vec3d reconstructed_velocity{
        -reverse_snapshot.state[0].velocity.x
            - profile.dt * profile.gravity.x,
        -reverse_snapshot.state[0].velocity.y
            - profile.dt * profile.gravity.y,
        -reverse_snapshot.state[0].velocity.z
            - profile.dt * profile.gravity.z};
    result.reverse_position_error = distance(
        reverse_snapshot.state[0].current, input[0].current);
    result.reverse_velocity_error = distance(
        reconstructed_velocity, input[0].velocity);
    const double reconstructed_energy = 0.5 * profile.mass
            * squared_norm(reconstructed_velocity)
        - profile.mass * (profile.gravity.x
                * reverse_snapshot.state[0].current.x
            + profile.gravity.y * reverse_snapshot.state[0].current.y
            + profile.gravity.z * reverse_snapshot.state[0].current.z);
    result.reversible_energy_drift = std::abs(
        reconstructed_energy - initial_energy)
        / std::max(std::abs(initial_energy), 1.0);
    const double momentum_scale = std::max({
        profile.mass * std::sqrt(squared_norm(input[0].velocity)),
        profile.dt * profile.mass * std::sqrt(squared_norm(profile.gravity)),
        profile.spacing * profile.mass / profile.dt});
    result.momentum_residual = maximum_momentum / momentum_scale;
    result.receipt_root = nextengine::nonlocal::sha256_hex(receipts.str());
    result.passed = maximum_position <= 5.0e-6
        && maximum_velocity <= 5.0e-6 / profile.dt
        && result.positive_energy_excess <= 0.01
        && result.reverse_position_error <= 5.0e-6
        && result.reverse_velocity_error <= 5.0e-6 / profile.dt
        && result.reversible_energy_drift <= 0.01
        && result.momentum_residual <= 0.01;
    return result;
}

std::vector<NonlocalGpuSample> invariant_block() {
    std::vector<NonlocalGpuSample> result;
    for (std::uint32_t z = 0U; z < 4U; ++z) {
        for (std::uint32_t y = 0U; y < 4U; ++y) {
            for (std::uint32_t x = 0U; x < 4U; ++x) {
                const Vec3d position{0.65 + 0.05 * x, 0.65 + 0.05 * y,
                    0.65 + 0.05 * z};
                const std::uint32_t id = 1000U + x + 4U * y + 16U * z;
                result.push_back({id, position, position, {}});
            }
        }
    }
    return canonicalize_samples_binary32(result);
}

struct InvarianceResult {
    bool passed = false;
    double translation_error = std::numeric_limits<double>::infinity();
    double rotation_error = std::numeric_limits<double>::infinity();
    std::uint64_t permutation_mismatches = 0U;
    std::string receipt_root;
};

InvarianceResult run_invariance() {
    InvarianceResult result;
    const NonlocalGpuProfile profile = nonlocal_water_corrected_profile();
    const auto base = invariant_block();
    const Vec3d translation{0.5, 0.25, 0.25};
    const Vec3d centre{0.725, 0.725, 0.725};
    auto translated = base;
    auto rotated = base;
    auto permuted = base;
    for (NonlocalGpuSample& sample : translated) {
        sample.reference = {sample.reference.x + translation.x,
            sample.reference.y + translation.y,
            sample.reference.z + translation.z};
        sample.current = sample.reference;
    }
    for (NonlocalGpuSample& sample : rotated) {
        const double x = sample.reference.x - centre.x;
        const double y = sample.reference.y - centre.y;
        sample.reference = {centre.x - y, centre.y + x, sample.reference.z};
        sample.current = sample.reference;
    }
    std::reverse(permuted.begin(), permuted.end());
    translated = canonicalize_samples_binary32(translated);
    rotated = canonicalize_samples_binary32(rotated);
    permuted = canonicalize_samples_binary32(permuted);
    NonlocalGpuWorkspace base_gpu(profile);
    NonlocalGpuWorkspace translation_gpu(profile);
    NonlocalGpuWorkspace rotation_gpu(profile);
    NonlocalGpuWorkspace permutation_gpu(profile);
    if (base_gpu.upload(base, {}, true) != NonlocalGpuFailure::None
        || translation_gpu.upload(translated, {}, true) != NonlocalGpuFailure::None
        || rotation_gpu.upload(rotated, {}, true) != NonlocalGpuFailure::None
        || permutation_gpu.upload(permuted, {}, true) != NonlocalGpuFailure::None) {
        return result;
    }
    double maximum_translation = 0.0;
    double maximum_rotation = 0.0;
    std::ostringstream receipts;
    receipts << "nextengine.nonlocal.ncgp3.invariance.v1\n";
    for (std::uint32_t step = 0U; step < 8U; ++step) {
        const auto a = base_gpu.step(kBudget, NonlocalGpuSolverProfile::Jacobi,
            NonlocalGpuVariant::CompensatedScaleF32, false, false);
        const auto b = translation_gpu.step(kBudget,
            NonlocalGpuSolverProfile::Jacobi,
            NonlocalGpuVariant::CompensatedScaleF32, false, false);
        const auto c = rotation_gpu.step(kBudget,
            NonlocalGpuSolverProfile::Jacobi,
            NonlocalGpuVariant::CompensatedScaleF32, false, false);
        const auto d = permutation_gpu.step(kBudget,
            NonlocalGpuSolverProfile::Jacobi,
            NonlocalGpuVariant::CompensatedScaleF32, false, false);
        const auto as = base_gpu.capture_public_snapshot();
        const auto bs = translation_gpu.capture_public_snapshot();
        const auto cs = rotation_gpu.capture_public_snapshot();
        const auto ds = permutation_gpu.capture_public_snapshot();
        receipts << step_work_semantic_root(profile, a) << ':'
                 << step_work_semantic_root(profile, b) << ':'
                 << step_work_semantic_root(profile, c) << ':'
                 << step_work_semantic_root(profile, d) << ':'
                 << work_semantic_root(as.work) << ':'
                 << work_semantic_root(bs.work) << ':'
                 << work_semantic_root(cs.work) << ':'
                 << work_semantic_root(ds.work) << '\n';
        if (a.failure != NonlocalGpuFailure::None
            || b.failure != NonlocalGpuFailure::None
            || c.failure != NonlocalGpuFailure::None
            || d.failure != NonlocalGpuFailure::None
            || as.state.size() != base.size()
            || bs.state.size() != base.size()
            || cs.state.size() != base.size()
            || ds.state.size() != base.size()) {
            result.receipt_root = nextengine::nonlocal::sha256_hex(receipts.str());
            return result;
        }
        if (snapshot_state_root(as) != snapshot_state_root(ds)
            || as.active_pressure_ids != ds.active_pressure_ids) {
            ++result.permutation_mismatches;
        }
        for (std::size_t index = 0U; index < as.state.size(); ++index) {
            const Vec3d translated_back{
                bs.state[index].current.x - translation.x,
                bs.state[index].current.y - translation.y,
                bs.state[index].current.z - translation.z};
            maximum_translation = std::max(maximum_translation,
                distance(as.state[index].current, translated_back));
            const double x = cs.state[index].current.x - centre.x;
            const double y = cs.state[index].current.y - centre.y;
            const Vec3d rotated_back{centre.x + y, centre.y - x,
                cs.state[index].current.z};
            maximum_rotation = std::max(maximum_rotation,
                distance(as.state[index].current, rotated_back));
        }
    }
    result.translation_error = maximum_translation;
    result.rotation_error = maximum_rotation;
    result.receipt_root = nextengine::nonlocal::sha256_hex(receipts.str());
    result.passed = maximum_translation <= 5.0e-6
        && maximum_rotation <= 5.0e-6
        && result.permutation_mismatches == 0U;
    return result;
}

double kinetic_energy(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& state) {
    double energy = 0.0;
    for (const auto& sample : state) {
        energy += 0.5 * profile.mass * squared_norm(sample.velocity);
    }
    return energy;
}

struct ViscosityResult {
    bool passed = false;
    double maximum_relative_energy_increase =
        std::numeric_limits<double>::infinity();
    double final_relative_energy_change =
        std::numeric_limits<double>::infinity();
    std::string receipt_root;
};

ViscosityResult run_viscosity() {
    ViscosityResult result;
    NonlocalGpuProfile profile = nonlocal_water_corrected_profile();
    profile.gravity = {};
    const auto input = canonicalize_samples_binary32({
        {101U, {0.725, 0.75, 0.75}, {0.725, 0.75, 0.75}, {0.0, 0.001, 0.0}},
        {202U, {0.775, 0.75, 0.75}, {0.775, 0.75, 0.75}, {0.0, -0.001, 0.0}},
    });
    auto permuted = input;
    std::reverse(permuted.begin(), permuted.end());
    NonlocalGpuWorkspace gpu(profile);
    NonlocalGpuWorkspace permutation_gpu(profile);
    if (gpu.upload(input, {}, true) != NonlocalGpuFailure::None
        || permutation_gpu.upload(permuted, {}, true)
            != NonlocalGpuFailure::None) return result;
    const double initial = kinetic_energy(profile, input);
    double previous = initial;
    double maximum_increase = 0.0;
    double final = initial;
    std::ostringstream receipts;
    receipts << "nextengine.nonlocal.ncgp3.viscosity.v1\n";
    for (std::uint32_t step = 0U; step < 32U; ++step) {
        const auto a = gpu.step(kBudget, NonlocalGpuSolverProfile::Jacobi,
            NonlocalGpuVariant::CompensatedScaleF32, false, false);
        const auto b = permutation_gpu.step(kBudget,
            NonlocalGpuSolverProfile::Jacobi,
            NonlocalGpuVariant::CompensatedScaleF32, false, false);
        const auto as = gpu.capture_public_snapshot();
        const auto bs = permutation_gpu.capture_public_snapshot();
        receipts << step_work_semantic_root(profile, a) << ':'
                 << step_work_semantic_root(profile, b) << ':'
                 << work_semantic_root(as.work) << ':'
                 << work_semantic_root(bs.work) << '\n';
        if (a.failure != NonlocalGpuFailure::None
            || b.failure != NonlocalGpuFailure::None
            || snapshot_state_root(as) != snapshot_state_root(bs)) {
            result.receipt_root = nextengine::nonlocal::sha256_hex(receipts.str());
            return result;
        }
        final = kinetic_energy(profile, as.state);
        maximum_increase = std::max(maximum_increase, final - previous);
        previous = final;
    }
    const double energy_scale = std::max(std::abs(initial), 1.0);
    result.maximum_relative_energy_increase = maximum_increase / energy_scale;
    result.final_relative_energy_change = std::abs(final - initial)
        / energy_scale;
    result.receipt_root = nextengine::nonlocal::sha256_hex(receipts.str());
    result.passed = result.maximum_relative_energy_increase <= 1.0e-6
        && result.final_relative_energy_change <= 1.0e-6;
    return result;
}

long double surface_potential(long double radius, long double spacing) {
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

long double surface_mechanical_energy(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& state) {
    long double energy = 0.0L;
    for (const auto& sample : state) {
        energy += 0.5L * profile.mass
            * static_cast<long double>(squared_norm(sample.velocity));
    }
    for (std::size_t a = 0U; a < state.size(); ++a) {
        for (std::size_t b = a + 1U; b < state.size(); ++b) {
            const long double radius = distance(
                state[a].current, state[b].current);
            if (radius <= profile.horizon) {
                energy += 2.0L * profile.gamma * profile.mass * profile.mass
                    * surface_potential(radius, profile.spacing);
            }
        }
    }
    return energy;
}

std::vector<NonlocalGpuSample> tetrahedron_state() {
    const double edge = 0.04;
    const double sqrt3 = std::sqrt(3.0);
    const double sqrt23 = std::sqrt(2.0 / 3.0);
    const std::array<Vec3d, 4> local{{{0.0, 0.0, 0.0}, {edge, 0.0, 0.0},
        {0.5 * edge, 0.5 * sqrt3 * edge, 0.0},
        {0.5 * edge, edge * sqrt3 / 6.0, edge * sqrt23}}};
    Vec3d centroid{};
    for (const Vec3d value : local) {
        centroid.x += value.x / 4.0;
        centroid.y += value.y / 4.0;
        centroid.z += value.z / 4.0;
    }
    std::vector<NonlocalGpuSample> result;
    for (std::size_t index = 0U; index < local.size(); ++index) {
        const Vec3d position{0.75 + local[index].x - centroid.x,
            0.75 + local[index].y - centroid.y,
            0.75 + local[index].z - centroid.z};
        const Vec3d radial = subtract(position, {0.75, 0.75, 0.75});
        const double inverse_radius = 1.0 / std::sqrt(squared_norm(radial));
        const Vec3d velocity{0.01 * radial.x * inverse_radius,
            0.01 * radial.y * inverse_radius,
            0.01 * radial.z * inverse_radius};
        result.push_back({static_cast<std::uint32_t>(101U + 101U * index),
            position, position, velocity});
    }
    return canonicalize_samples_binary32(result);
}

struct SurfaceResult {
    bool passed = false;
    double maximum_position_error = std::numeric_limits<double>::infinity();
    double energy_decrease = -std::numeric_limits<double>::infinity();
    std::string receipt_root;
};

SurfaceResult run_surface_relaxation() {
    SurfaceResult result;
    NonlocalGpuProfile profile = nonlocal_water_corrected_profile();
    profile.gravity = {};
    auto cpu_state = tetrahedron_state();
    const auto input = cpu_state;
    NonlocalGpuWorkspace gpu(profile);
    if (gpu.upload(input, {}, true) != NonlocalGpuFailure::None) return result;
    const long double initial_energy = surface_mechanical_energy(profile, input);
    long double prior_energy = initial_energy;
    double maximum_error = 0.0;
    bool monotone = true;
    std::ostringstream receipts;
    receipts << "nextengine.nonlocal.ncgp3.surface.v1\n";
    for (std::uint32_t step = 0U; step < 32U; ++step) {
        const auto gpu_step = gpu.step(kBudget,
            NonlocalGpuSolverProfile::Jacobi,
            NonlocalGpuVariant::CompensatedScaleF32, false, false);
        const auto snapshot = gpu.capture_public_snapshot();
        const auto cpu_step = step_reference(profile, cpu_state, {}, kBudget,
            NonlocalGpuVariant::Corrected, true);
        receipts << step_work_semantic_root(profile, gpu_step) << ':'
                 << step_work_semantic_root(profile, cpu_step) << ':'
                 << work_semantic_root(snapshot.work) << '\n';
        if (gpu_step.failure != NonlocalGpuFailure::None
            || snapshot.failure != NonlocalGpuFailure::None
            || cpu_step.failure != NonlocalGpuFailure::None
            || snapshot.state.size() != cpu_step.state.size()
            || !snapshot.active_pressure_ids.empty()
            || !cpu_step.active_pressure_ids.empty()) {
            result.receipt_root = nextengine::nonlocal::sha256_hex(receipts.str());
            return result;
        }
        for (std::size_t index = 0U; index < snapshot.state.size(); ++index) {
            maximum_error = std::max(maximum_error,
                distance(snapshot.state[index].current,
                    cpu_step.state[index].current));
        }
        const long double energy = surface_mechanical_energy(
            profile, cpu_step.state);
        if (energy > prior_energy + 1.0e-12L) monotone = false;
        prior_energy = energy;
        cpu_state = cpu_step.state;
    }
    result.maximum_position_error = maximum_error;
    result.energy_decrease = static_cast<double>(initial_energy - prior_energy);
    result.receipt_root = nextengine::nonlocal::sha256_hex(receipts.str());
    result.passed = maximum_error <= 5.0e-6 && monotone
        && result.energy_decrease > 0.0;
    return result;
}

struct HvpResult {
    bool passed = false;
    double relative_l2 = std::numeric_limits<double>::infinity();
    double cosine_loss = std::numeric_limits<double>::infinity();
    std::uint32_t active = 0U;
    std::string input_root;
    std::string work_root;
    std::string reference_work_root;
};

HvpResult run_hvp() {
    HvpResult result;
    const NonlocalGpuProfile profile = nonlocal_water_corrected_profile();
    std::vector<NonlocalGpuSample> input;
    std::vector<Vec3d> direction;
    for (std::uint32_t z = 0U; z < 3U; ++z) {
        for (std::uint32_t y = 0U; y < 3U; ++y) {
            for (std::uint32_t x = 0U; x < 3U; ++x) {
                const std::uint32_t lane = x + 3U * y + 9U * z;
                const Vec3d reference{0.7 + 0.05 * x, 0.7 + 0.05 * y,
                    0.7 + 0.05 * z};
                const Vec3d current{0.7 + 0.045 * x, 0.7 + 0.045 * y,
                    0.7 + 0.045 * z};
                input.push_back({1000U + lane, reference, current, {}});
                direction.push_back({
                    (static_cast<int>(lane % 5U) - 2) * 0.03125,
                    (static_cast<int>(lane % 7U) - 3) * 0.015625,
                    (static_cast<int>(lane % 11U) - 5) * 0.0078125});
            }
        }
    }
    input = canonicalize_samples_binary32(input);
    result.input_root = input_semantic_root(profile, input, {});
    NonlocalGpuWorkspace gpu(profile);
    if (gpu.upload(input, {}, true) != NonlocalGpuFailure::None) return result;
    const auto actual = gpu.evaluate(&direction,
        NonlocalGpuVariant::CompensatedScaleF32, true, false);
    const auto expected = evaluate_reference(profile, input, {}, &direction,
        NonlocalGpuVariant::CompensatedScaleF32);
    if (actual.failure != NonlocalGpuFailure::None
        || expected.failure != NonlocalGpuFailure::None
        || actual.hvp.size() != expected.hvp.size()) return result;
    long double difference_squared = 0.0L;
    long double expected_squared = 0.0L;
    long double actual_squared = 0.0L;
    long double product = 0.0L;
    for (std::size_t index = 0U; index < actual.hvp.size(); ++index) {
        const Vec3d delta = subtract(actual.hvp[index], expected.hvp[index]);
        difference_squared += squared_norm(delta);
        expected_squared += squared_norm(expected.hvp[index]);
        actual_squared += squared_norm(actual.hvp[index]);
        product += actual.hvp[index].x * expected.hvp[index].x
            + actual.hvp[index].y * expected.hvp[index].y
            + actual.hvp[index].z * expected.hvp[index].z;
    }
    result.relative_l2 = static_cast<double>(std::sqrt(difference_squared)
        / std::max(std::sqrt(expected_squared), 1.0e-30L));
    result.cosine_loss = static_cast<double>(1.0L
        - product / std::max(std::sqrt(actual_squared * expected_squared),
              1.0e-30L));
    result.active = actual.active_pressure_centers;
    result.work_root = work_semantic_root(actual.work);
    result.reference_work_root = work_semantic_root(expected.work);
    result.passed = result.relative_l2 <= 1.0e-3
        && result.cosine_loss <= 1.0e-6
        && actual.active_pressure_ids == expected.active_pressure_ids
        && !actual.active_pressure_ids.empty()
        && actual.active_pressure_ids.size() < input.size();
    return result;
}

struct PredictorResult {
    bool passed = false;
    std::uint64_t correct_eft_components = 0U;
    std::uint64_t ordinary_eft_components = 0U;
    std::string correct_root;
    std::string ordinary_root;
    std::string receipt_root;
};

PredictorResult run_predictor_negative() {
    PredictorResult result;
    const NonlocalGpuProfile profile = nonlocal_water_corrected_profile();
    const auto input = canonicalize_samples_binary32({
        {101U, {0.75, 0.75, 1.0}, {0.75, 0.75, 1.0},
            {0.12345, -0.06789, 0.0314159}}});
    NonlocalGpuWorkspace correct(profile);
    NonlocalGpuWorkspace ordinary(profile);
    if (correct.upload(input, {}, true) != NonlocalGpuFailure::None
        || ordinary.upload(input, {}, true) != NonlocalGpuFailure::None) {
        return result;
    }
    const auto correct_step = correct.step(kBudget,
        NonlocalGpuSolverProfile::Jacobi,
        NonlocalGpuVariant::CompensatedScaleF32, false, false);
    const auto ordinary_step = ordinary.step(kBudget,
        NonlocalGpuSolverProfile::Jacobi,
        NonlocalGpuVariant::CompensatedScaleOrdinaryPredictor, false, false);
    const auto correct_state = correct.capture_compensated_state();
    const auto ordinary_state = ordinary.capture_compensated_state();
    result.correct_eft_components =
        correct_step.work.compensated_prediction_eft_components;
    result.ordinary_eft_components =
        ordinary_step.work.compensated_prediction_eft_components;
    result.correct_root = compensated_state_root(correct_state);
    result.ordinary_root = compensated_state_root(ordinary_state);
    std::ostringstream receipt;
    receipt << "nextengine.nonlocal.ncgp4.predictor-receipt.v1\n"
            << step_work_semantic_root(profile, correct_step) << '\n'
            << step_work_semantic_root(profile, ordinary_step) << '\n'
            << work_semantic_root(correct_state.work) << '\n'
            << work_semantic_root(ordinary_state.work) << '\n';
    result.receipt_root = nextengine::nonlocal::sha256_hex(receipt.str());
    bool correct_has_low = false;
    bool ordinary_all_zero = true;
    for (const Vec3d value : correct_state.predicted_low) {
        correct_has_low = correct_has_low || value.x != 0.0 || value.y != 0.0
            || value.z != 0.0;
    }
    for (const Vec3d value : ordinary_state.predicted_low) {
        ordinary_all_zero = ordinary_all_zero && value.x == 0.0
            && value.y == 0.0 && value.z == 0.0;
    }
    result.passed = correct_step.failure == NonlocalGpuFailure::None
        && ordinary_step.failure == NonlocalGpuFailure::None
        && correct_state.failure == NonlocalGpuFailure::None
        && ordinary_state.failure == NonlocalGpuFailure::None
        && correct_has_low && ordinary_all_zero
        && result.correct_eft_components == 6U
        && result.ordinary_eft_components == 0U
        && !result.correct_root.empty()
        && result.correct_root != result.ordinary_root
        && !result.receipt_root.empty();
    return result;
}

} // namespace

int run_ncgp3_physics_self_test(bool emit_result) {
    const FreeFallResult free_fall = run_free_fall();
    const InvarianceResult invariance = run_invariance();
    const ViscosityResult viscosity = run_viscosity();
    const SurfaceResult surface = run_surface_relaxation();
    const HvpResult hvp = run_hvp();
    const PredictorResult predictor = run_predictor_negative();
    const bool controls_passed = free_fall.passed && invariance.passed
        && viscosity.passed && surface.passed && hvp.passed
        && predictor.passed;
#if defined(NCGP4_EXPERIMENTAL)
    constexpr const char* kSchema = "nextengine.nonlocal.ncgp4.physics.v1";
    constexpr const char* kSuiteDomain =
        "nextengine.nonlocal.ncgp4.physics-suite.v1\n";
    constexpr const char* kContractRoot = NCGP4_CONTRACT_ROOT;
    constexpr const char* kSourceRoot = NCGP4_SOURCE_ROOT;
    constexpr const char* kSourceCommit = NCGP4_SOURCE_COMMIT;
    constexpr const char* kSourceTree = NCGP4_SOURCE_TREE;
    constexpr const char* kCompilerFlags = NCGP4_COMPILER_FLAGS;
#else
    constexpr const char* kSchema = "nextengine.nonlocal.ncgp3.physics.v2";
    constexpr const char* kSuiteDomain =
        "nextengine.nonlocal.ncgp3.physics-suite.v2\n";
    constexpr const char* kContractRoot = NCGP3_CONTRACT_ROOT;
    constexpr const char* kSourceRoot = NCGP3_SOURCE_ROOT;
    constexpr const char* kSourceCommit = "unconfigured";
    constexpr const char* kSourceTree = "unconfigured";
    constexpr const char* kCompilerFlags = "unconfigured";
#endif
    NonlocalGpuWorkspace environment_probe(nonlocal_water_corrected_profile());
    const std::string environment = environment_probe.environment_json();
    const std::string executable_root = physics_binary_root();
#if defined(NCGP4_EXPERIMENTAL)
    const bool passed = controls_passed && !executable_root.empty()
        && std::string(kContractRoot) != "unconfigured"
        && std::string(kSourceRoot) != "unconfigured"
        && std::string(kSourceCommit) != "unconfigured"
        && std::string(kSourceTree) != "unconfigured"
        && std::string(kCompilerFlags) != "unconfigured";
#else
    const bool passed = controls_passed;
#endif
    std::ostringstream roots;
    roots << std::setprecision(17) << kSuiteDomain
          << passed << '\n'
          << free_fall.passed << '\n'
          << free_fall.maximum_position_error << '\n'
          << free_fall.maximum_velocity_error << '\n'
          << free_fall.positive_energy_excess << '\n'
          << free_fall.reverse_position_error << '\n'
          << free_fall.reverse_velocity_error << '\n'
          << free_fall.reversible_energy_drift << '\n'
          << free_fall.momentum_residual << '\n'
          << free_fall.input_root << '\n' << free_fall.receipt_root << '\n'
          << invariance.passed << '\n' << invariance.translation_error << '\n'
          << invariance.rotation_error << '\n'
          << invariance.permutation_mismatches << '\n'
          << invariance.receipt_root << '\n'
          << viscosity.passed << '\n'
          << viscosity.maximum_relative_energy_increase << '\n'
          << viscosity.final_relative_energy_change << '\n'
          << viscosity.receipt_root << '\n'
          << surface.passed << '\n' << surface.maximum_position_error << '\n'
          << surface.energy_decrease << '\n' << surface.receipt_root << '\n'
          << hvp.passed << '\n' << hvp.relative_l2 << '\n'
          << hvp.cosine_loss << '\n' << hvp.active << '\n'
          << hvp.input_root << '\n' << hvp.work_root << '\n'
          << hvp.reference_work_root << '\n'
          << predictor.passed << '\n'
          << predictor.correct_eft_components << '\n'
          << predictor.ordinary_eft_components << '\n'
          << predictor.correct_root << '\n' << predictor.ordinary_root << '\n'
          << predictor.receipt_root << '\n'
          << kContractRoot << '\n' << kSourceRoot << '\n'
          << kSourceCommit << '\n' << kSourceTree << '\n'
          << kCompilerFlags << '\n' << executable_root << '\n'
          << environment << '\n';
    const std::string suite_root = nextengine::nonlocal::sha256_hex(
        roots.str());
    if (emit_result) std::cout << std::setprecision(17)
              << "{\"schema\":\"" << kSchema << "\""
              << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
              << ",\"free_fall_pass\":" << (free_fall.passed ? "true" : "false")
              << ",\"free_fall_position_max_m\":"
              << free_fall.maximum_position_error
              << ",\"free_fall_velocity_max_m_s\":"
              << free_fall.maximum_velocity_error
              << ",\"positive_energy_excess_fraction\":"
              << free_fall.positive_energy_excess
              << ",\"reverse_position_error_m\":"
              << free_fall.reverse_position_error
              << ",\"reverse_velocity_error_m_s\":"
              << free_fall.reverse_velocity_error
              << ",\"reversible_energy_drift_fraction\":"
              << free_fall.reversible_energy_drift
              << ",\"free_fall_momentum_residual\":"
              << free_fall.momentum_residual
              << ",\"invariance_pass\":" << (invariance.passed ? "true" : "false")
              << ",\"translation_error_max_m\":" << invariance.translation_error
              << ",\"rotation_error_max_m\":" << invariance.rotation_error
              << ",\"invariance_permutation_mismatches\":"
              << invariance.permutation_mismatches
              << ",\"viscosity_pass\":" << (viscosity.passed ? "true" : "false")
              << ",\"viscosity_energy_increase_max_fraction\":"
              << viscosity.maximum_relative_energy_increase
              << ",\"viscosity_final_energy_change_fraction\":"
              << viscosity.final_relative_energy_change
              << ",\"surface_pass\":" << (surface.passed ? "true" : "false")
              << ",\"surface_position_error_max_m\":"
              << surface.maximum_position_error
              << ",\"surface_energy_decrease_j\":" << surface.energy_decrease
              << ",\"hvp_pass\":" << (hvp.passed ? "true" : "false")
              << ",\"hvp_relative_l2\":" << hvp.relative_l2
              << ",\"hvp_cosine_loss\":" << hvp.cosine_loss
              << ",\"hvp_active_centers\":" << hvp.active
              << ",\"predictor_negative_pass\":"
              << (predictor.passed ? "true" : "false")
              << ",\"predictor_eft_components\":"
              << predictor.correct_eft_components
              << ",\"ordinary_predictor_eft_components\":"
              << predictor.ordinary_eft_components
              << ",\"suite_root\":\"" << suite_root << "\""
              << ",\"contract_root\":\"" << kContractRoot << "\""
              << ",\"source_root\":\"" << kSourceRoot << "\""
              << ",\"source_commit\":\"" << kSourceCommit << "\""
              << ",\"source_tree\":\"" << kSourceTree << "\""
              << ",\"compiler_flags\":\"" << kCompilerFlags << "\""
              << ",\"binary_root\":\"" << executable_root << "\""
              << ",\"environment\":" << environment << "}\n";
    return passed ? 0 : 4;
}
