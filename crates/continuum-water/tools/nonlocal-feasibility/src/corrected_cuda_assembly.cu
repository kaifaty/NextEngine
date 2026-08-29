#include "corrected_cuda_assembly.hpp"

#include <cuda_runtime.h>

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <sstream>
#include <stdexcept>
#include <string>
#include <unordered_set>
#include <vector>

namespace nextengine::nonlocal::gpu_assembly_audit {
namespace {

constexpr int THREADS = 128;
constexpr long long COORDINATE_LIMIT_UM = 1000000000LL;

struct DeviceSample {
    unsigned int id;
    long long reference[3];
    long long predicted[3];
    long long current[3];
    int direction[3];
};

struct DeviceProfile {
    float horizon;
    float spacing;
    float mass;
    float time_step;
    float rest_density;
    float kappa;
    float lambda;
    float mu;
    float gamma;
    unsigned int terms;
    unsigned int variant;
};

struct DeviceVec3 {
    float x;
    float y;
    float z;
};

void check_cuda(cudaError_t value, const char* operation) {
    if (value != cudaSuccess) {
        throw std::runtime_error(
            std::string(operation) + ": " + cudaGetErrorString(value));
    }
}

int blocks_for(std::size_t count) {
    return static_cast<int>((count + static_cast<std::size_t>(THREADS) - 1U)
        / static_cast<std::size_t>(THREADS));
}

template <typename T>
class DeviceBuffer {
public:
    DeviceBuffer() = default;
    explicit DeviceBuffer(std::size_t count) { allocate(count); }
    DeviceBuffer(const DeviceBuffer&) = delete;
    DeviceBuffer& operator=(const DeviceBuffer&) = delete;
    ~DeviceBuffer() {
        if (data_ != nullptr) cudaFree(data_);
    }
    void allocate(std::size_t count) {
        if (count == 0U) return;
        check_cuda(cudaMalloc(reinterpret_cast<void**>(&data_), count * sizeof(T)),
            "cudaMalloc NCGA2 buffer");
    }
    T* get() { return data_; }
    const T* get() const { return data_; }
private:
    T* data_ = nullptr;
};

__device__ DeviceVec3 add(DeviceVec3 lhs, DeviceVec3 rhs) {
    return {lhs.x + rhs.x, lhs.y + rhs.y, lhs.z + rhs.z};
}

__device__ DeviceVec3 subtract(DeviceVec3 lhs, DeviceVec3 rhs) {
    return {lhs.x - rhs.x, lhs.y - rhs.y, lhs.z - rhs.z};
}

__device__ DeviceVec3 multiply(float scalar, DeviceVec3 value) {
    return {scalar * value.x, scalar * value.y, scalar * value.z};
}

__device__ float dot(DeviceVec3 lhs, DeviceVec3 rhs) {
    return lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z;
}

__device__ float length(DeviceVec3 value) { return sqrtf(dot(value, value)); }

__device__ float component(DeviceVec3 value, int axis) {
    return axis == 0 ? value.x : (axis == 1 ? value.y : value.z);
}

__device__ DeviceVec3 position(const DeviceSample& sample, int kind) {
    const long long* value = kind == 0 ? sample.reference
        : (kind == 1 ? sample.predicted : sample.current);
    constexpr float scale = 1.0e-6F;
    return {scale * static_cast<float>(value[0]),
        scale * static_cast<float>(value[1]),
        scale * static_cast<float>(value[2])};
}

__device__ DeviceVec3 direction(const DeviceSample& sample) {
    constexpr float scale = 1.0e-3F;
    return {scale * static_cast<float>(sample.direction[0]),
        scale * static_cast<float>(sample.direction[1]),
        scale * static_cast<float>(sample.direction[2])};
}

__device__ unsigned long long magnitude(long long value) {
    return static_cast<unsigned long long>(value < 0 ? -value : value);
}

__device__ bool inside_support(const long long* lhs, const long long* rhs) {
    unsigned long long squared = 0ULL;
    for (int axis = 0; axis < 3; ++axis) {
        const unsigned long long value = magnitude(lhs[axis] - rhs[axis]);
        if (value > static_cast<unsigned long long>(ASSEMBLY_SUPPORT_UM)) {
            return false;
        }
        squared += value * value;
    }
    return squared <= static_cast<unsigned long long>(
        ASSEMBLY_SUPPORT_UM * ASSEMBLY_SUPPORT_UM);
}

__device__ float cubic_weight(float radius, float horizon) {
    constexpr float pi = 3.14159265358979323846F;
    const float q = 2.0F * radius / horizon;
    const float alpha = 3.0F / (2.0F * pi * horizon * horizon * horizon);
    if (q > 2.0F) return 0.0F;
    if (q >= 1.0F) {
        const float delta = 2.0F - q;
        return alpha * delta * delta * delta / 6.0F;
    }
    return alpha * (2.0F / 3.0F - q * q + 0.5F * q * q * q);
}

__device__ float cubic_gradient(float radius, const DeviceProfile& profile) {
    constexpr float pi = 3.14159265358979323846F;
    const float q = 2.0F * radius / profile.horizon;
    const float h = profile.horizon;
    const float alpha = 3.0F / (2.0F * pi * h * h * h);
    if (q > 2.0F) return 0.0F;
    const float derivative_q = q >= 1.0F
        ? -0.5F * alpha * (2.0F - q) * (2.0F - q)
        : alpha * (-2.0F * q + 1.5F * q * q);
    float value = derivative_q * 2.0F / h;
    if (profile.variant
        == static_cast<unsigned int>(AssemblyVariant::SourceShapedGradient)) {
        value *= 0.5F * h;
    }
    return value;
}

__device__ float cubic_second(float radius, const DeviceProfile& profile) {
    constexpr float pi = 3.14159265358979323846F;
    const float q = 2.0F * radius / profile.horizon;
    const float h = profile.horizon;
    const float alpha = 3.0F / (2.0F * pi * h * h * h);
    if (q > 2.0F) return 0.0F;
    const float second_q = q >= 1.0F
        ? alpha * (2.0F - q) : alpha * (-2.0F + 3.0F * q);
    float value = second_q * 4.0F / (h * h);
    if (profile.variant
        == static_cast<unsigned int>(AssemblyVariant::SourceShapedGradient)) {
        value *= 0.5F * h;
    }
    return value;
}

__device__ float surface_spline(float radius, float spacing) {
    const float q = radius / spacing;
    if (q <= 1.0F) return q * q - 1.0F;
    if (q < 3.0F) return 1.0F - (q - 2.0F) * (q - 2.0F);
    return 0.0F;
}

__device__ float surface_derivative(float radius, float spacing) {
    const float q = radius / spacing;
    if (q <= 1.0F) return 2.0F * q / spacing;
    if (q < 3.0F) return -2.0F * (q - 2.0F) / spacing;
    return 0.0F;
}

__device__ float surface_potential(float radius, float spacing) {
    const float q = radius / spacing;
    if (q <= 1.0F) {
        return spacing * (q * q * q / 3.0F - q - 2.0F / 3.0F);
    }
    if (q < 3.0F) {
        const float value = q - 2.0F;
        return spacing * (q - value * value * value / 3.0F - 8.0F / 3.0F);
    }
    return 0.0F;
}

__device__ float radial_entry(DeviceVec3 normal, float radial,
    float tangential, int row_axis, int column_axis) {
    return tangential * (row_axis == column_axis ? 1.0F : 0.0F)
        + (radial - tangential) * component(normal, row_axis)
            * component(normal, column_axis);
}

__global__ void sort_owner_indices(const DeviceSample* samples,
    unsigned int* owner_indices, unsigned int* owner_ids, int count,
    unsigned long long* comparisons) {
    if (blockIdx.x != 0 || threadIdx.x != 0) return;
    for (int i = 0; i < count; ++i) owner_indices[i] = static_cast<unsigned int>(i);
    unsigned long long work = 0ULL;
    for (int first = 0; first < count; ++first) {
        int selected = first;
        for (int candidate = first + 1; candidate < count; ++candidate) {
            ++work;
            if (samples[owner_indices[candidate]].id
                < samples[owner_indices[selected]].id) selected = candidate;
        }
        if (selected != first) {
            const unsigned int value = owner_indices[first];
            owner_indices[first] = owner_indices[selected];
            owner_indices[selected] = value;
        }
    }
    for (int i = 0; i < count; ++i) owner_ids[i] = samples[owner_indices[i]].id;
    *comparisons = work;
}

__global__ void build_graphs(const DeviceSample* samples,
    const unsigned int* owner_indices, unsigned int* current_counts,
    unsigned int* current_rows, unsigned int* reference_counts,
    unsigned int* reference_rows, int count) {
    const int row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= count) return;
    const DeviceSample owner = samples[owner_indices[row]];
    unsigned int current_count = 0U;
    unsigned int reference_count = 0U;
    for (int candidate = 0; candidate < count; ++candidate) {
        const DeviceSample other = samples[owner_indices[candidate]];
        if (inside_support(owner.current, other.current)) {
            current_rows[row * count + static_cast<int>(current_count)]
                = static_cast<unsigned int>(candidate);
            ++current_count;
        }
        if (inside_support(owner.reference, other.reference)) {
            reference_rows[row * count + static_cast<int>(reference_count)]
                = static_cast<unsigned int>(candidate);
            ++reference_count;
        }
    }
    current_counts[row] = current_count;
    reference_counts[row] = reference_count;
}

__device__ bool row_contains(const unsigned int* counts,
    const unsigned int* rows, int count, int owner, int candidate) {
    const unsigned int degree = counts[owner];
    for (unsigned int slot = 0U; slot < degree; ++slot) {
        if (rows[owner * count + static_cast<int>(slot)]
            == static_cast<unsigned int>(candidate)) return true;
    }
    return false;
}

__global__ void compute_density(const DeviceSample* samples,
    const unsigned int* owner_indices, const unsigned int* current_counts,
    const unsigned int* current_rows, int count, DeviceProfile profile,
    float* density, float* compression, unsigned char* active) {
    const int row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= count) return;
    const DeviceVec3 owner = position(samples[owner_indices[row]], 2);
    float rho = 0.0F;
    for (unsigned int slot = 0U; slot < current_counts[row]; ++slot) {
        const int neighbor = static_cast<int>(
            current_rows[row * count + static_cast<int>(slot)]);
        const DeviceVec3 other = position(samples[owner_indices[neighbor]], 2);
        rho += profile.mass * cubic_weight(length(subtract(owner, other)),
            profile.horizon);
    }
    density[row] = rho;
    compression[row] = fmaxf(rho / profile.rest_density - 1.0F, 0.0F);
    active[row] = compression[row] > 0.0F ? 1U : 0U;
}

__global__ void compute_energy_gradient(const DeviceSample* samples,
    const unsigned int* owner_indices,
    const unsigned int* current_counts, const unsigned int* current_rows,
    const unsigned int* reference_counts, const unsigned int* reference_rows,
    const float* compression, int count, DeviceProfile profile,
    float* row_energy, float* gradient) {
    const int row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= count) return;
    const DeviceSample sample = samples[owner_indices[row]];
    const DeviceVec3 x = position(sample, 0);
    const DeviceVec3 predicted = position(sample, 1);
    const DeviceVec3 current = position(sample, 2);
    DeviceVec3 value{0.0F, 0.0F, 0.0F};
    float inertia_energy = 0.0F;
    float pressure_energy = 0.0F;
    float viscosity_energy = 0.0F;
    float surface_energy = 0.0F;
    if ((profile.terms & 1U) != 0U) {
        const float scale = profile.mass / (profile.time_step * profile.time_step);
        const DeviceVec3 displacement = subtract(current, predicted);
        inertia_energy = 0.5F * scale * dot(displacement, displacement);
        value = add(value, multiply(scale, displacement));
    }
    if ((profile.terms & 2U) != 0U) {
        pressure_energy = 0.5F * profile.kappa
            * compression[row] * compression[row];
        for (unsigned int slot = 0U; slot < current_counts[row]; ++slot) {
            const int neighbor = static_cast<int>(
                current_rows[row * count + static_cast<int>(slot)]);
            if (neighbor == row) continue;
            const DeviceVec3 other = position(samples[owner_indices[neighbor]], 2);
            const DeviceVec3 displacement = subtract(current, other);
            const float radius = length(displacement);
            if (radius <= 1.0e-15F) continue;
            const float neighbor_compression = profile.variant
                    == static_cast<unsigned int>(AssemblyVariant::OwnerPressureOnly)
                ? 0.0F : compression[neighbor];
            const float scale = profile.kappa * profile.mass / profile.rest_density
                * (compression[row] + neighbor_compression)
                * cubic_gradient(radius, profile) / radius;
            value = add(value, multiply(scale, displacement));
        }
    }
    const bool current_viscosity = profile.variant
        == static_cast<unsigned int>(AssemblyVariant::CurrentGraphViscosity);
    const unsigned int* viscosity_counts = current_viscosity
        ? current_counts : reference_counts;
    const unsigned int* viscosity_rows = current_viscosity
        ? current_rows : reference_rows;
    if ((profile.terms & 4U) != 0U) {
        for (unsigned int slot = 0U; slot < viscosity_counts[row]; ++slot) {
            const int neighbor = static_cast<int>(
                viscosity_rows[row * count + static_cast<int>(slot)]);
            if (neighbor == row) continue;
            const DeviceSample other_sample = samples[owner_indices[neighbor]];
            const DeviceVec3 reference = subtract(x, position(other_sample, 0));
            const float radius = length(reference);
            if (radius <= 1.0e-15F) continue;
            const DeviceVec3 normal = multiply(1.0F / radius, reference);
            const DeviceVec3 increment = subtract(
                subtract(current, position(other_sample, 2)), reference);
            const DeviceVec3 normal_part = multiply(dot(increment, normal), normal);
            const DeviceVec3 tangent_part = subtract(increment, normal_part);
            const float omega = -cubic_gradient(radius, profile);
            float coefficient_scale = 1.0F;
            if (profile.variant == static_cast<unsigned int>(
                    AssemblyVariant::DirectedEdgeViscosity)) {
                coefficient_scale = 0.5F;
            }
            const DeviceVec3 pair = multiply(profile.mass * omega
                    / (profile.rest_density * profile.time_step),
                add(multiply(coefficient_scale * 2.0F * profile.mu, tangent_part),
                    multiply(coefficient_scale * profile.lambda, normal_part)));
            value = add(value, pair);
            if (neighbor > row) {
                viscosity_energy += profile.mass * omega
                    / (profile.rest_density * profile.time_step)
                    * (profile.mu * dot(tangent_part, tangent_part)
                        + 0.5F * profile.lambda * dot(normal_part, normal_part));
            }
        }
    }
    if ((profile.terms & 8U) != 0U) {
        for (unsigned int slot = 0U; slot < current_counts[row]; ++slot) {
            const int neighbor = static_cast<int>(
                current_rows[row * count + static_cast<int>(slot)]);
            if (neighbor == row) continue;
            const DeviceVec3 other = position(samples[owner_indices[neighbor]], 2);
            const DeviceVec3 displacement = subtract(current, other);
            const float radius = length(displacement);
            if (radius <= 1.0e-15F || radius >= 3.0F * profile.spacing) continue;
            const DeviceVec3 normal = multiply(1.0F / radius, displacement);
            value = add(value, multiply(2.0F * profile.gamma * profile.mass
                * profile.mass * surface_spline(radius, profile.spacing), normal));
            if (neighbor > row) {
                surface_energy += 2.0F * profile.gamma * profile.mass * profile.mass
                    * surface_potential(radius, profile.spacing);
            }
        }
    }
    row_energy[4 * row] = inertia_energy;
    row_energy[4 * row + 1] = pressure_energy;
    row_energy[4 * row + 2] = viscosity_energy;
    row_energy[4 * row + 3] = surface_energy;
    gradient[3 * row] = value.x;
    gradient[3 * row + 1] = value.y;
    gradient[3 * row + 2] = value.z;
}

__global__ void reduce_energy(const float* row_energy, int count, float* energy) {
    if (blockIdx.x != 0 || threadIdx.x != 0) return;
    for (int term = 0; term < 4; ++term) {
        float value = 0.0F;
        for (int row = 0; row < count; ++row) value += row_energy[4 * row + term];
        energy[term] = value;
    }
}

__global__ void build_pressure_jacobian(const DeviceSample* samples,
    const unsigned int* owner_indices, const unsigned int* current_counts,
    const unsigned int* current_rows, const unsigned char* active,
    int count, DeviceProfile profile, float* jacobian) {
    const int dimension = 3 * count;
    const int scalar = blockIdx.x * blockDim.x + threadIdx.x;
    if (scalar >= count * dimension) return;
    const int center = scalar / dimension;
    const int degree = scalar % dimension;
    if (active[center] == 0U || (profile.terms & 2U) == 0U) {
        jacobian[scalar] = 0.0F;
        return;
    }
    const int participant = degree / 3;
    const int axis = degree % 3;
    const DeviceVec3 center_position = position(samples[owner_indices[center]], 2);
    float value = 0.0F;
    if (participant == center) {
        for (unsigned int slot = 0U; slot < current_counts[center]; ++slot) {
            const int neighbor = static_cast<int>(
                current_rows[center * count + static_cast<int>(slot)]);
            if (neighbor == center) continue;
            const DeviceVec3 displacement = subtract(center_position,
                position(samples[owner_indices[neighbor]], 2));
            const float radius = length(displacement);
            if (radius <= 1.0e-15F) continue;
            value += profile.mass / profile.rest_density
                * cubic_gradient(radius, profile) / radius
                * component(displacement, axis);
        }
    } else if (row_contains(current_counts, current_rows,
                   count, center, participant)) {
        const DeviceVec3 displacement = subtract(center_position,
            position(samples[owner_indices[participant]], 2));
        const float radius = length(displacement);
        if (radius > 1.0e-15F) {
            value = -profile.mass / profile.rest_density
                * cubic_gradient(radius, profile) / radius
                * component(displacement, axis);
        }
    }
    jacobian[scalar] = value;
}

__device__ float pair_curvature_entry(const DeviceSample* samples,
    const unsigned int* owner_indices, int first, int second,
    int row_axis, int column_axis, DeviceProfile profile, int kind) {
    const int position_kind = kind == 0 ? 0 : 2;
    const DeviceVec3 displacement = subtract(
        position(samples[owner_indices[first]], position_kind),
        position(samples[owner_indices[second]], position_kind));
    const float radius = length(displacement);
    if (radius <= 1.0e-15F) return 0.0F;
    const DeviceVec3 normal = multiply(1.0F / radius, displacement);
    if (kind == 0) {
        float coefficient_scale = 1.0F;
        if (profile.variant == static_cast<unsigned int>(
                AssemblyVariant::DirectedEdgeViscosity)) {
            coefficient_scale = 0.5F;
        }
        const float scale = profile.mass * (-cubic_gradient(radius, profile))
            / (profile.rest_density * profile.time_step);
        return radial_entry(normal, coefficient_scale * scale * profile.lambda,
            coefficient_scale * scale * 2.0F * profile.mu,
            row_axis, column_axis);
    }
    const float scale = 2.0F * profile.gamma * profile.mass * profile.mass;
    return radial_entry(normal,
        scale * surface_derivative(radius, profile.spacing),
        scale * surface_spline(radius, profile.spacing) / radius,
        row_axis, column_axis);
}

__device__ float sissm_diagonal_entry(const DeviceSample* samples,
    const unsigned int* owner_indices, const unsigned int* current_counts,
    const unsigned int* current_rows, const unsigned int* reference_counts,
    const unsigned int* reference_rows, const float* compression,
    int count, int particle, int row_axis, int column_axis,
    DeviceProfile profile) {
    const float inertia_scale = profile.mass
        / (profile.time_step * profile.time_step);
    float local = row_axis == column_axis ? 1.0F : 0.0F;
    if ((profile.terms & 2U) != 0U) {
        for (int center = 0; center < count; ++center) {
            if (compression[center] <= 0.0F) continue;
            for (unsigned int slot = 0U; slot < current_counts[center]; ++slot) {
                const int neighbor = static_cast<int>(
                    current_rows[center * count + static_cast<int>(slot)]);
                if (neighbor == center || (particle != center && particle != neighbor)) {
                    continue;
                }
                const DeviceVec3 displacement = subtract(
                    position(samples[owner_indices[center]], 2),
                    position(samples[owner_indices[neighbor]], 2));
                const float radius = length(displacement);
                if (radius <= 1.0e-15F) continue;
                const float positive = -profile.kappa * profile.time_step
                    * profile.time_step / profile.rest_density
                    * cubic_gradient(radius, profile) / radius;
                if (row_axis == column_axis) local += positive;
            }
        }
    }
    if ((profile.terms & 4U) != 0U) {
        for (unsigned int slot = 0U; slot < reference_counts[particle]; ++slot) {
            const int neighbor = static_cast<int>(
                reference_rows[particle * count + static_cast<int>(slot)]);
            if (neighbor == particle) continue;
            const DeviceVec3 reference = subtract(
                position(samples[owner_indices[particle]], 0),
                position(samples[owner_indices[neighbor]], 0));
            const float radius = length(reference);
            if (radius <= 1.0e-15F) continue;
            const DeviceVec3 normal = multiply(1.0F / radius, reference);
            const float scale = profile.time_step * (-cubic_gradient(radius, profile))
                / profile.rest_density;
            local += radial_entry(normal, scale * profile.lambda,
                scale * 2.0F * profile.mu, row_axis, column_axis);
        }
    }
    if ((profile.terms & 8U) != 0U) {
        for (unsigned int slot = 0U; slot < current_counts[particle]; ++slot) {
            const int neighbor = static_cast<int>(
                current_rows[particle * count + static_cast<int>(slot)]);
            if (neighbor == particle) continue;
            const DeviceVec3 displacement = subtract(
                position(samples[owner_indices[particle]], 2),
                position(samples[owner_indices[neighbor]], 2));
            const float radius = length(displacement);
            if (radius <= 1.0e-15F) continue;
            const float coefficient = 2.0F * profile.gamma * profile.mass
                * profile.time_step * profile.time_step
                * surface_spline(radius, profile.spacing) / radius;
            if (coefficient >= 0.0F && row_axis == column_axis) {
                local += coefficient;
            }
        }
    }
    return inertia_scale * local;
}

__global__ void assemble_hessian(const DeviceSample* samples,
    const unsigned int* owner_indices,
    const unsigned int* current_counts, const unsigned int* current_rows,
    const unsigned int* reference_counts, const unsigned int* reference_rows,
    const float* compression, const unsigned char* active,
    const float* jacobian, int count, DeviceProfile profile, float* hessian) {
    const int dimension = 3 * count;
    const int entry = blockIdx.x * blockDim.x + threadIdx.x;
    if (entry >= dimension * dimension) return;
    const int row = entry / dimension;
    const int column = entry % dimension;
    const int row_particle = row / 3;
    const int column_particle = column / 3;
    const int row_axis = row % 3;
    const int column_axis = column % 3;
    if (profile.variant
        == static_cast<unsigned int>(AssemblyVariant::SissmLocalMatrix)) {
        hessian[entry] = row_particle == column_particle
            ? sissm_diagonal_entry(samples, owner_indices, current_counts,
                current_rows, reference_counts, reference_rows, compression,
                count, row_particle, row_axis, column_axis, profile)
            : 0.0F;
        return;
    }
    float value = (profile.terms & 1U) != 0U && row == column
        ? profile.mass / (profile.time_step * profile.time_step) : 0.0F;
    if ((profile.terms & 2U) != 0U) {
        for (int center = 0; center < count; ++center) {
            if (active[center] == 0U) continue;
            value += profile.kappa * jacobian[center * dimension + row]
                * jacobian[center * dimension + column];
            if (profile.variant == static_cast<unsigned int>(
                    AssemblyVariant::GaussNewtonPressureOnly)) continue;
            const bool row_center = row_particle == center;
            const bool column_center = column_particle == center;
            if (row_center && column_center) {
                for (unsigned int slot = 0U; slot < current_counts[center]; ++slot) {
                    const int neighbor = static_cast<int>(
                        current_rows[center * count + static_cast<int>(slot)]);
                    if (neighbor == center) continue;
                    const DeviceVec3 displacement = subtract(
                        position(samples[owner_indices[center]], 2),
                        position(samples[owner_indices[neighbor]], 2));
                    const float radius = length(displacement);
                    if (radius <= 1.0e-15F) continue;
                    const DeviceVec3 normal = multiply(1.0F / radius, displacement);
                    value += profile.kappa * compression[center]
                        * profile.mass / profile.rest_density
                        * radial_entry(normal, cubic_second(radius, profile),
                            cubic_gradient(radius, profile) / radius,
                            row_axis, column_axis);
                }
            } else {
                int neighbor = -1;
                float sign = 0.0F;
                if (row_center && row_contains(current_counts, current_rows,
                        count, center, column_particle)) {
                    neighbor = column_particle;
                    sign = -1.0F;
                } else if (column_center && row_contains(current_counts, current_rows,
                               count, center, row_particle)) {
                    neighbor = row_particle;
                    sign = -1.0F;
                } else if (row_particle == column_particle
                    && row_contains(current_counts, current_rows,
                        count, center, row_particle)) {
                    neighbor = row_particle;
                    sign = 1.0F;
                }
                if (neighbor >= 0 && neighbor != center) {
                    const DeviceVec3 displacement = subtract(
                        position(samples[owner_indices[center]], 2),
                        position(samples[owner_indices[neighbor]], 2));
                    const float radius = length(displacement);
                    if (radius > 1.0e-15F) {
                        const DeviceVec3 normal = multiply(1.0F / radius, displacement);
                        value += sign * profile.kappa * compression[center]
                            * profile.mass / profile.rest_density
                            * radial_entry(normal, cubic_second(radius, profile),
                                cubic_gradient(radius, profile) / radius,
                                row_axis, column_axis);
                    }
                }
            }
        }
    }
    const bool current_viscosity = profile.variant
        == static_cast<unsigned int>(AssemblyVariant::CurrentGraphViscosity);
    const unsigned int* viscosity_counts = current_viscosity
        ? current_counts : reference_counts;
    const unsigned int* viscosity_rows = current_viscosity
        ? current_rows : reference_rows;
    if ((profile.terms & 4U) != 0U) {
        if (row_particle == column_particle) {
            for (unsigned int slot = 0U; slot < viscosity_counts[row_particle]; ++slot) {
                const int neighbor = static_cast<int>(
                    viscosity_rows[row_particle * count + static_cast<int>(slot)]);
                if (neighbor == row_particle) continue;
                value += pair_curvature_entry(samples, owner_indices,
                    row_particle, neighbor, row_axis, column_axis, profile, 0);
            }
        } else if (row_contains(viscosity_counts, viscosity_rows,
                       count, row_particle, column_particle)) {
            value -= pair_curvature_entry(samples, owner_indices,
                row_particle, column_particle, row_axis, column_axis, profile, 0);
        }
    }
    if ((profile.terms & 8U) != 0U) {
        if (row_particle == column_particle) {
            for (unsigned int slot = 0U; slot < current_counts[row_particle]; ++slot) {
                const int neighbor = static_cast<int>(
                    current_rows[row_particle * count + static_cast<int>(slot)]);
                if (neighbor == row_particle) continue;
                value += pair_curvature_entry(samples, owner_indices,
                    row_particle, neighbor, row_axis, column_axis, profile, 1);
            }
        } else if (row_contains(current_counts, current_rows,
                       count, row_particle, column_particle)) {
            value -= pair_curvature_entry(samples, owner_indices,
                row_particle, column_particle, row_axis, column_axis, profile, 1);
        }
    }
    hessian[entry] = value;
}

__global__ void extract_blocks(const float* hessian,
    int count, float* blocks) {
    const int entry = blockIdx.x * blockDim.x + threadIdx.x;
    if (entry >= 9 * count) return;
    const int particle = entry / 9;
    const int local = entry % 9;
    const int row_axis = local / 3;
    const int column_axis = local % 3;
    const int dimension = 3 * count;
    blocks[entry] = hessian[(3 * particle + row_axis) * dimension
        + 3 * particle + column_axis];
}

__global__ void direct_hvp(const DeviceSample* samples,
    const unsigned int* owner_indices,
    const unsigned int* current_counts, const unsigned int* current_rows,
    const unsigned int* reference_counts, const unsigned int* reference_rows,
    const float* compression, const unsigned char* active,
    const float* jacobian, int count, DeviceProfile profile, float* hvp) {
    const int row = blockIdx.x * blockDim.x + threadIdx.x;
    const int dimension = 3 * count;
    if (row >= dimension) return;
    const int particle = row / 3;
    const int axis = row % 3;
    float value = (profile.terms & 1U) != 0U
        ? profile.mass / (profile.time_step * profile.time_step)
            * component(direction(samples[owner_indices[particle]]), axis)
        : 0.0F;
    if ((profile.terms & 2U) != 0U) {
        for (int center = 0; center < count; ++center) {
            if (active[center] == 0U) continue;
            float density_direction = 0.0F;
            for (int column = 0; column < dimension; ++column) {
                density_direction += jacobian[center * dimension + column]
                    * component(direction(samples[owner_indices[column / 3]]),
                        column % 3);
            }
            value += profile.kappa * jacobian[center * dimension + row]
                * density_direction;
            if (profile.variant == static_cast<unsigned int>(
                    AssemblyVariant::GaussNewtonPressureOnly)) continue;
            if (particle == center) {
                for (unsigned int slot = 0U; slot < current_counts[center]; ++slot) {
                    const int neighbor = static_cast<int>(
                        current_rows[center * count + static_cast<int>(slot)]);
                    if (neighbor == center) continue;
                    const DeviceVec3 displacement = subtract(
                        position(samples[owner_indices[center]], 2),
                        position(samples[owner_indices[neighbor]], 2));
                    const float radius = length(displacement);
                    if (radius <= 1.0e-15F) continue;
                    const DeviceVec3 normal = multiply(1.0F / radius, displacement);
                    const DeviceVec3 relative_direction = subtract(
                        direction(samples[owner_indices[center]]),
                        direction(samples[owner_indices[neighbor]]));
                    float product = 0.0F;
                    for (int column_axis = 0; column_axis < 3; ++column_axis) {
                        product += radial_entry(normal, cubic_second(radius, profile),
                            cubic_gradient(radius, profile) / radius,
                            axis, column_axis)
                            * component(relative_direction, column_axis);
                    }
                    value += profile.kappa * compression[center]
                        * profile.mass / profile.rest_density * product;
                }
            } else if (row_contains(current_counts, current_rows,
                           count, center, particle)) {
                const DeviceVec3 displacement = subtract(
                    position(samples[owner_indices[center]], 2),
                    position(samples[owner_indices[particle]], 2));
                const float radius = length(displacement);
                if (radius > 1.0e-15F) {
                    const DeviceVec3 normal = multiply(1.0F / radius, displacement);
                    const DeviceVec3 relative_direction = subtract(
                        direction(samples[owner_indices[center]]),
                        direction(samples[owner_indices[particle]]));
                    float product = 0.0F;
                    for (int column_axis = 0; column_axis < 3; ++column_axis) {
                        product += radial_entry(normal, cubic_second(radius, profile),
                            cubic_gradient(radius, profile) / radius,
                            axis, column_axis)
                            * component(relative_direction, column_axis);
                    }
                    value -= profile.kappa * compression[center]
                        * profile.mass / profile.rest_density * product;
                }
            }
        }
    }
    const bool current_viscosity = profile.variant
        == static_cast<unsigned int>(AssemblyVariant::CurrentGraphViscosity);
    const unsigned int* viscosity_counts = current_viscosity
        ? current_counts : reference_counts;
    const unsigned int* viscosity_rows = current_viscosity
        ? current_rows : reference_rows;
    if ((profile.terms & 4U) != 0U) {
        for (unsigned int slot = 0U; slot < viscosity_counts[particle]; ++slot) {
            const int neighbor = static_cast<int>(
                viscosity_rows[particle * count + static_cast<int>(slot)]);
            if (neighbor == particle) continue;
            const DeviceVec3 relative_direction = subtract(
                direction(samples[owner_indices[particle]]),
                direction(samples[owner_indices[neighbor]]));
            for (int column_axis = 0; column_axis < 3; ++column_axis) {
                value += pair_curvature_entry(samples, owner_indices,
                    particle, neighbor, axis, column_axis, profile, 0)
                    * component(relative_direction, column_axis);
            }
        }
    }
    if ((profile.terms & 8U) != 0U) {
        for (unsigned int slot = 0U; slot < current_counts[particle]; ++slot) {
            const int neighbor = static_cast<int>(
                current_rows[particle * count + static_cast<int>(slot)]);
            if (neighbor == particle) continue;
            const DeviceVec3 relative_direction = subtract(
                direction(samples[owner_indices[particle]]),
                direction(samples[owner_indices[neighbor]]));
            for (int column_axis = 0; column_axis < 3; ++column_axis) {
                value += pair_curvature_entry(samples, owner_indices,
                    particle, neighbor, axis, column_axis, profile, 1)
                    * component(relative_direction, column_axis);
            }
        }
    }
    hvp[row] = value;
}

bool admitted(const AssemblyProfile& profile, const AssemblyFixture& fixture,
    AssemblyFailure& failure) {
    if (fixture.samples.empty()) {
        failure = AssemblyFailure::InvalidInput;
        return false;
    }
    if (fixture.samples.size() > ASSEMBLY_MAX_SAMPLES) {
        failure = AssemblyFailure::CapacityExceeded;
        return false;
    }
    if (!std::isfinite(profile.horizon) || profile.horizon != 0.15
        || !std::isfinite(profile.spacing) || profile.spacing != 0.05
        || !std::isfinite(profile.mass) || profile.mass <= 0.0
        || !std::isfinite(profile.time_step) || profile.time_step <= 0.0
        || !std::isfinite(profile.rest_density) || profile.rest_density <= 0.0
        || !std::isfinite(profile.kappa) || profile.kappa < 0.0
        || !std::isfinite(profile.lambda) || profile.lambda < 0.0
        || !std::isfinite(profile.mu) || profile.mu < 0.0
        || !std::isfinite(profile.gamma) || profile.gamma < 0.0) {
        failure = AssemblyFailure::InvalidInput;
        return false;
    }
    std::unordered_set<std::uint32_t> ids;
    for (const AssemblySample& sample : fixture.samples) {
        if (!ids.insert(sample.sample_id).second) {
            failure = AssemblyFailure::DuplicateSampleId;
            return false;
        }
        for (const auto* coordinates : {&sample.reference_um, &sample.predicted_um,
                 &sample.current_um}) {
            for (std::int64_t value : *coordinates) {
                if (value < -COORDINATE_LIMIT_UM || value > COORDINATE_LIMIT_UM) {
                    failure = AssemblyFailure::InvalidInput;
                    return false;
                }
            }
        }
    }
    failure = AssemblyFailure::None;
    return true;
}

std::uint32_t term_bits(const AssemblyTerms& terms) {
    return (terms.inertia ? 1U : 0U) | (terms.pressure ? 2U : 0U)
        | (terms.viscosity ? 4U : 0U) | (terms.surface ? 8U : 0U);
}

void build_csr(const std::vector<unsigned int>& counts,
    const std::vector<unsigned int>& rows, const std::vector<unsigned int>& owner_ids,
    std::vector<std::uint32_t>& offsets,
    std::vector<std::uint32_t>& neighbors) {
    const std::size_t count = counts.size();
    offsets.reserve(count + 1U);
    offsets.push_back(0U);
    for (std::size_t row = 0; row < count; ++row) {
        for (std::size_t slot = 0; slot < counts[row]; ++slot) {
            neighbors.push_back(owner_ids[rows[row * count + slot]]);
        }
        offsets.push_back(static_cast<std::uint32_t>(neighbors.size()));
    }
}

void compute_margins(const AssemblyFixture& fixture, AssemblyResult& result) {
    if (fixture.samples.size() < 2U) return;
    result.minimum_current_support_margin_um = std::numeric_limits<std::int64_t>::max();
    result.minimum_reference_support_margin_um = std::numeric_limits<std::int64_t>::max();
    result.minimum_surface_branch_margin_um = std::numeric_limits<std::int64_t>::max();
    const auto radius_um = [](const auto& lhs, const auto& rhs) {
        long double squared = 0.0L;
        for (std::size_t axis = 0; axis < 3U; ++axis) {
            const long double delta = static_cast<long double>(lhs[axis] - rhs[axis]);
            squared += delta * delta;
        }
        return std::sqrt(squared);
    };
    for (std::size_t first = 0; first < fixture.samples.size(); ++first) {
        for (std::size_t second = first + 1U; second < fixture.samples.size(); ++second) {
            const long double current = radius_um(fixture.samples[first].current_um,
                fixture.samples[second].current_um);
            const long double reference = radius_um(
                fixture.samples[first].reference_um,
                fixture.samples[second].reference_um);
            result.minimum_current_support_margin_um = std::min(
                result.minimum_current_support_margin_um,
                static_cast<std::int64_t>(std::llround(std::abs(
                    current - static_cast<long double>(ASSEMBLY_SUPPORT_UM)))));
            result.minimum_reference_support_margin_um = std::min(
                result.minimum_reference_support_margin_um,
                static_cast<std::int64_t>(std::llround(std::abs(
                    reference - static_cast<long double>(ASSEMBLY_SUPPORT_UM)))));
            for (long double branch : {50000.0L, 150000.0L}) {
                result.minimum_surface_branch_margin_um = std::min(
                    result.minimum_surface_branch_margin_um,
                    static_cast<std::int64_t>(std::llround(std::abs(current - branch))));
            }
        }
    }
}

} // namespace

AssemblyResult evaluate_gpu_assembly(const AssemblyProfile& profile,
    const AssemblyFixture& fixture, AssemblyVariant variant) {
    AssemblyResult output;
    if (!admitted(profile, fixture, output.failure)) return output;
    const int count = static_cast<int>(fixture.samples.size());
    const int dimension = 3 * count;
    const std::size_t dense_entries = static_cast<std::size_t>(dimension) * dimension;
    std::vector<DeviceSample> host_samples;
    host_samples.reserve(fixture.samples.size());
    for (const AssemblySample& sample : fixture.samples) {
        DeviceSample value{};
        value.id = sample.sample_id;
        for (std::size_t axis = 0; axis < 3U; ++axis) {
            value.reference[axis] = sample.reference_um[axis];
            value.predicted[axis] = sample.predicted_um[axis];
            value.current[axis] = sample.current_um[axis];
            value.direction[axis] = sample.direction_milli[axis];
        }
        host_samples.push_back(value);
    }
    const DeviceProfile device_profile{
        static_cast<float>(profile.horizon), static_cast<float>(profile.spacing),
        static_cast<float>(profile.mass), static_cast<float>(profile.time_step),
        static_cast<float>(profile.rest_density), static_cast<float>(profile.kappa),
        static_cast<float>(profile.lambda), static_cast<float>(profile.mu),
        static_cast<float>(profile.gamma), term_bits(fixture.terms),
        static_cast<unsigned int>(variant)};

    DeviceBuffer<DeviceSample> device_samples(host_samples.size());
    DeviceBuffer<unsigned int> owner_indices(host_samples.size());
    DeviceBuffer<unsigned int> owner_ids(host_samples.size());
    DeviceBuffer<unsigned long long> sort_comparisons(1U);
    DeviceBuffer<unsigned int> current_counts(host_samples.size());
    DeviceBuffer<unsigned int> reference_counts(host_samples.size());
    DeviceBuffer<unsigned int> current_rows(host_samples.size() * host_samples.size());
    DeviceBuffer<unsigned int> reference_rows(host_samples.size() * host_samples.size());
    DeviceBuffer<float> density(host_samples.size());
    DeviceBuffer<float> compression(host_samples.size());
    DeviceBuffer<unsigned char> active(host_samples.size());
    DeviceBuffer<float> row_energy(4U * host_samples.size());
    DeviceBuffer<float> energy(4U);
    DeviceBuffer<float> gradient(static_cast<std::size_t>(dimension));
    DeviceBuffer<float> jacobian(static_cast<std::size_t>(count) * dimension);
    DeviceBuffer<float> hessian(dense_entries);
    DeviceBuffer<float> diagonal_blocks(9U * host_samples.size());
    DeviceBuffer<float> hvp(static_cast<std::size_t>(dimension));

    check_cuda(cudaMemcpy(device_samples.get(), host_samples.data(),
                   host_samples.size() * sizeof(DeviceSample), cudaMemcpyHostToDevice),
        "upload NCGA2 samples");
    check_cuda(cudaMemset(sort_comparisons.get(), 0, sizeof(unsigned long long)),
        "reset NCGA2 sort work");
    sort_owner_indices<<<1, 1>>>(device_samples.get(), owner_indices.get(),
        owner_ids.get(), count, sort_comparisons.get());
    check_cuda(cudaGetLastError(), "launch NCGA2 owner sort");
    build_graphs<<<blocks_for(host_samples.size()), THREADS>>>(device_samples.get(),
        owner_indices.get(), current_counts.get(), current_rows.get(),
        reference_counts.get(), reference_rows.get(), count);
    check_cuda(cudaGetLastError(), "launch NCGA2 graph build");
    compute_density<<<blocks_for(host_samples.size()), THREADS>>>(
        device_samples.get(), owner_indices.get(), current_counts.get(),
        current_rows.get(), count, device_profile, density.get(), compression.get(),
        active.get());
    check_cuda(cudaGetLastError(), "launch NCGA2 density");
    compute_energy_gradient<<<blocks_for(host_samples.size()), THREADS>>>(
        device_samples.get(), owner_indices.get(), current_counts.get(),
        current_rows.get(), reference_counts.get(), reference_rows.get(),
        compression.get(), count, device_profile, row_energy.get(), gradient.get());
    check_cuda(cudaGetLastError(), "launch NCGA2 energy/gradient");
    reduce_energy<<<1, 1>>>(row_energy.get(), count, energy.get());
    check_cuda(cudaGetLastError(), "launch NCGA2 energy reduction");
    build_pressure_jacobian<<<blocks_for(static_cast<std::size_t>(count) * dimension),
        THREADS>>>(device_samples.get(), owner_indices.get(), current_counts.get(),
        current_rows.get(), active.get(), count, device_profile, jacobian.get());
    check_cuda(cudaGetLastError(), "launch NCGA2 pressure Jacobian");
    assemble_hessian<<<blocks_for(dense_entries), THREADS>>>(device_samples.get(),
        owner_indices.get(), current_counts.get(), current_rows.get(),
        reference_counts.get(), reference_rows.get(), compression.get(), active.get(),
        jacobian.get(), count, device_profile, hessian.get());
    check_cuda(cudaGetLastError(), "launch NCGA2 Hessian");
    extract_blocks<<<blocks_for(9U * host_samples.size()), THREADS>>>(hessian.get(),
        count, diagonal_blocks.get());
    check_cuda(cudaGetLastError(), "launch NCGA2 diagonal blocks");
    direct_hvp<<<blocks_for(static_cast<std::size_t>(dimension)), THREADS>>>(
        device_samples.get(), owner_indices.get(), current_counts.get(),
        current_rows.get(), reference_counts.get(), reference_rows.get(),
        compression.get(), active.get(), jacobian.get(), count, device_profile,
        hvp.get());
    check_cuda(cudaGetLastError(), "launch NCGA2 HVP");
    check_cuda(cudaDeviceSynchronize(), "synchronize NCGA2 assembly");

    std::vector<unsigned int> host_owner_ids(host_samples.size());
    std::vector<unsigned int> host_current_counts(host_samples.size());
    std::vector<unsigned int> host_reference_counts(host_samples.size());
    std::vector<unsigned int> host_current_rows(host_samples.size() * host_samples.size());
    std::vector<unsigned int> host_reference_rows(host_samples.size() * host_samples.size());
    std::vector<float> host_density(host_samples.size());
    std::vector<float> host_compression(host_samples.size());
    std::vector<unsigned char> host_active(host_samples.size());
    std::array<float, 4> host_energy{};
    std::vector<float> host_gradient(static_cast<std::size_t>(dimension));
    std::vector<float> host_jacobian(static_cast<std::size_t>(count) * dimension);
    std::vector<float> host_hessian(dense_entries);
    std::vector<float> host_blocks(9U * host_samples.size());
    std::vector<float> host_hvp(static_cast<std::size_t>(dimension));
    unsigned long long host_sort_comparisons = 0ULL;
#define NCGA2_COPY(destination, source, bytes, label) \
    check_cuda(cudaMemcpy((destination), (source), (bytes), cudaMemcpyDeviceToHost), (label))
    NCGA2_COPY(host_owner_ids.data(), owner_ids.get(),
        host_owner_ids.size() * sizeof(unsigned int), "copy NCGA2 owner IDs");
    NCGA2_COPY(host_current_counts.data(), current_counts.get(),
        host_current_counts.size() * sizeof(unsigned int), "copy NCGA2 current counts");
    NCGA2_COPY(host_reference_counts.data(), reference_counts.get(),
        host_reference_counts.size() * sizeof(unsigned int), "copy NCGA2 reference counts");
    NCGA2_COPY(host_current_rows.data(), current_rows.get(),
        host_current_rows.size() * sizeof(unsigned int), "copy NCGA2 current rows");
    NCGA2_COPY(host_reference_rows.data(), reference_rows.get(),
        host_reference_rows.size() * sizeof(unsigned int), "copy NCGA2 reference rows");
    NCGA2_COPY(host_density.data(), density.get(), host_density.size() * sizeof(float),
        "copy NCGA2 density");
    NCGA2_COPY(host_compression.data(), compression.get(),
        host_compression.size() * sizeof(float), "copy NCGA2 compression");
    NCGA2_COPY(host_active.data(), active.get(),
        host_active.size() * sizeof(unsigned char), "copy NCGA2 active flags");
    NCGA2_COPY(host_energy.data(), energy.get(), sizeof(host_energy), "copy NCGA2 energy");
    NCGA2_COPY(host_gradient.data(), gradient.get(),
        host_gradient.size() * sizeof(float), "copy NCGA2 gradient");
    NCGA2_COPY(host_jacobian.data(), jacobian.get(),
        host_jacobian.size() * sizeof(float), "copy NCGA2 Jacobian");
    NCGA2_COPY(host_hessian.data(), hessian.get(),
        host_hessian.size() * sizeof(float), "copy NCGA2 Hessian");
    NCGA2_COPY(host_blocks.data(), diagonal_blocks.get(),
        host_blocks.size() * sizeof(float), "copy NCGA2 diagonal blocks");
    NCGA2_COPY(host_hvp.data(), hvp.get(), host_hvp.size() * sizeof(float),
        "copy NCGA2 HVP");
    NCGA2_COPY(&host_sort_comparisons, sort_comparisons.get(),
        sizeof(host_sort_comparisons), "copy NCGA2 sort work");
#undef NCGA2_COPY

    output.owner_ids.assign(host_owner_ids.begin(), host_owner_ids.end());
    build_csr(host_current_counts, host_current_rows, host_owner_ids,
        output.current_offsets, output.current_neighbors);
    build_csr(host_reference_counts, host_reference_rows, host_owner_ids,
        output.reference_offsets, output.reference_neighbors);
    output.density.assign(host_density.begin(), host_density.end());
    for (std::size_t index = 0; index < 4U; ++index) output.energy[index] = host_energy[index];
    output.gradient.assign(host_gradient.begin(), host_gradient.end());
    output.hessian.assign(host_hessian.begin(), host_hessian.end());
    output.diagonal_blocks.assign(host_blocks.begin(), host_blocks.end());
    output.hvp.assign(host_hvp.begin(), host_hvp.end());
    output.pressure_active.assign(host_active.begin(), host_active.end());
    output.minimum_density_clamp_margin = std::numeric_limits<double>::infinity();
    for (float value : host_density) {
        output.minimum_density_clamp_margin = std::min(
            output.minimum_density_clamp_margin,
            std::abs(static_cast<double>(value) / profile.rest_density - 1.0));
    }
    compute_margins(fixture, output);
    output.work.owner_sort_comparisons = host_sort_comparisons;
    output.work.graph_distance_predicates = 2ULL * host_samples.size()
        * host_samples.size();
    output.work.density_kernel_evaluations = output.current_neighbors.size();
    output.work.active_pressure_centers = static_cast<std::uint64_t>(
        std::count(host_active.begin(), host_active.end(), static_cast<unsigned char>(1U)));
    const auto unique_edges = [](const std::vector<unsigned int>& counts,
        const std::vector<unsigned int>& rows, std::size_t count_value) {
        std::uint64_t result = 0U;
        for (std::size_t row = 0; row < count_value; ++row) {
            for (std::size_t slot = 0; slot < counts[row]; ++slot) {
                if (rows[row * count_value + slot] > row) ++result;
            }
        }
        return result;
    };
    const std::uint64_t current_edges = unique_edges(
        host_current_counts, host_current_rows, host_samples.size());
    const std::uint64_t reference_edges = unique_edges(
        host_reference_counts, host_reference_rows, host_samples.size());
    if (fixture.terms.pressure) {
        output.work.energy_pair_visits += current_edges;
        output.work.gradient_pair_visits += current_edges;
    }
    if (fixture.terms.viscosity) {
        output.work.energy_pair_visits += reference_edges;
        output.work.gradient_pair_visits += reference_edges;
        output.work.viscosity_curvature_blocks = 4U * reference_edges;
    }
    if (fixture.terms.surface) {
        output.work.energy_pair_visits += current_edges;
        output.work.gradient_pair_visits += current_edges;
        output.work.surface_curvature_blocks = 4U * current_edges;
    }
    for (std::size_t center = 0; center < host_samples.size(); ++center) {
        if (host_active[center] == 0U || !fixture.terms.pressure) continue;
        std::uint64_t nonzero = 0U;
        for (int column = 0; column < dimension; ++column) {
            if (host_jacobian[center * static_cast<std::size_t>(dimension)
                    + static_cast<std::size_t>(column)] != 0.0F) ++nonzero;
        }
        output.work.pressure_outer_products += nonzero * nonzero;
        output.work.pressure_geometric_blocks += 4ULL
            * (host_current_counts[center] - 1U);
    }
    output.work.dense_entries_written = dense_entries;
    output.work.hvp_products = dense_entries;
    output.work.host_device_scalar_transfers = host_owner_ids.size()
        + host_current_counts.size() + host_reference_counts.size()
        + host_current_rows.size() + host_reference_rows.size()
        + host_density.size() + host_compression.size() + host_active.size()
        + host_energy.size() + host_gradient.size() + host_jacobian.size()
        + host_hessian.size() + host_blocks.size() + host_hvp.size() + 1U;
    output.failure = AssemblyFailure::None;
    return output;
}

std::string gpu_assembly_environment_json() {
    int device = 0;
    check_cuda(cudaGetDevice(&device), "cudaGetDevice NCGA2");
    cudaDeviceProp properties{};
    check_cuda(cudaGetDeviceProperties(&properties, device),
        "cudaGetDeviceProperties NCGA2");
    int driver = 0;
    int runtime = 0;
    check_cuda(cudaDriverGetVersion(&driver), "cudaDriverGetVersion NCGA2");
    check_cuda(cudaRuntimeGetVersion(&runtime), "cudaRuntimeGetVersion NCGA2");
    std::ostringstream output;
    output << "{\"name\":\"" << properties.name << "\",\"major\":"
           << properties.major << ",\"minor\":" << properties.minor
           << ",\"driver\":" << driver << ",\"runtime\":" << runtime
           << ",\"strict_f32\":true,\"floating_atomics\":false}";
    return output.str();
}

} // namespace nextengine::nonlocal::gpu_assembly_audit
