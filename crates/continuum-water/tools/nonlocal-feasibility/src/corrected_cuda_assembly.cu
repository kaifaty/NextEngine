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
    float reference_m[3];
    float predicted_m[3];
    float current_m[3];
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

struct DeviceCompensationWork {
    unsigned long long additions;
    unsigned long long initializations;
};

struct DeviceF64EnergyWork {
    unsigned long long density_terms;
    unsigned long long particle_terms;
    unsigned long long viscosity_pairs;
    unsigned long long surface_pairs;
    unsigned long long component_writes;
};

struct F32Accumulator {
    float sum = 0.0F;
    float correction = 0.0F;
    bool compensated = true;

    __device__ void add(float value) {
        if (!compensated) {
            sum += value;
            return;
        }
        const float adjusted = value - correction;
        const float next = sum + adjusted;
        correction = (next - sum) - adjusted;
        sum = next;
    }
};

struct DeviceVec3 {
    float x;
    float y;
    float z;
};

struct DeviceVec3d {
    double x;
    double y;
    double z;
};

__device__ bool use_compensation(const DeviceProfile& profile) {
    return profile.variant
        != static_cast<unsigned int>(AssemblyVariant::NaiveF32Pressure);
}

__device__ F32Accumulator make_accumulator(const DeviceProfile& profile) {
    F32Accumulator result;
    result.compensated = use_compensation(profile);
    return result;
}

__device__ void publish_compensation_work(DeviceCompensationWork* work,
    const DeviceProfile& profile, unsigned long long additions,
    unsigned long long initializations) {
    if (!use_compensation(profile)) return;
    atomicAdd(&work->additions, additions);
    atomicAdd(&work->initializations, initializations);
}

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

__device__ DeviceVec3d to_f64(DeviceVec3 value) {
    return {static_cast<double>(value.x), static_cast<double>(value.y),
        static_cast<double>(value.z)};
}

__device__ DeviceVec3d subtract(DeviceVec3d lhs, DeviceVec3d rhs) {
    return {lhs.x - rhs.x, lhs.y - rhs.y, lhs.z - rhs.z};
}

__device__ DeviceVec3d multiply(double scalar, DeviceVec3d value) {
    return {scalar * value.x, scalar * value.y, scalar * value.z};
}

__device__ double dot(DeviceVec3d lhs, DeviceVec3d rhs) {
    return lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z;
}

__device__ double length(DeviceVec3d value) { return sqrt(dot(value, value)); }

__device__ float component(DeviceVec3 value, int axis) {
    return axis == 0 ? value.x : (axis == 1 ? value.y : value.z);
}

__device__ DeviceVec3 position(const DeviceSample& sample, int kind) {
    if (kind == 0) {
        return {sample.reference_m[0], sample.reference_m[1],
            sample.reference_m[2]};
    }
    if (kind == 1) {
        return {sample.predicted_m[0], sample.predicted_m[1],
            sample.predicted_m[2]};
    }
    if (kind == 2) {
        return {sample.current_m[0], sample.current_m[1], sample.current_m[2]};
    }
    return {0.0F, 0.0F, 0.0F};
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

__device__ double cubic_weight_f64(double radius, double horizon) {
    constexpr float pi_f32 = 3.14159265358979323846F;
    const double pi = static_cast<double>(pi_f32);
    const double q = 2.0 * radius / horizon;
    const double alpha = 3.0 / (2.0 * pi * horizon * horizon * horizon);
    if (q > 2.0) return 0.0;
    if (q >= 1.0) {
        const double delta = 2.0 - q;
        return alpha * delta * delta * delta / 6.0;
    }
    return alpha * (2.0 / 3.0 - q * q + 0.5 * q * q * q);
}

__device__ double cubic_gradient_f64(double radius, double horizon) {
    constexpr float pi_f32 = 3.14159265358979323846F;
    const double pi = static_cast<double>(pi_f32);
    const double q = 2.0 * radius / horizon;
    const double alpha = 3.0 / (2.0 * pi * horizon * horizon * horizon);
    if (q > 2.0) return 0.0;
    const double derivative_q = q >= 1.0
        ? -0.5 * alpha * (2.0 - q) * (2.0 - q)
        : alpha * (-2.0 * q + 1.5 * q * q);
    return derivative_q * 2.0 / horizon;
}

__device__ double surface_potential_f64(double radius, double spacing) {
    const double q = radius / spacing;
    if (q <= 1.0) {
        return spacing * (q * q * q / 3.0 - q - 2.0 / 3.0);
    }
    if (q < 3.0) {
        return spacing * (q - (q - 2.0) * (q - 2.0) * (q - 2.0)
            / 3.0 - 8.0 / 3.0);
    }
    return 0.0;
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
    float* density, float* compression, unsigned char* active,
    DeviceCompensationWork* work) {
    const int row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= count) return;
    const DeviceVec3 owner = position(samples[owner_indices[row]], 2);
    F32Accumulator rho = make_accumulator(profile);
    unsigned long long additions = 0ULL;
    for (unsigned int slot = 0U; slot < current_counts[row]; ++slot) {
        const int neighbor = static_cast<int>(
            current_rows[row * count + static_cast<int>(slot)]);
        const DeviceVec3 other = position(samples[owner_indices[neighbor]], 2);
        rho.add(profile.mass * cubic_weight(length(subtract(owner, other)),
            profile.horizon));
        ++additions;
    }
    density[row] = rho.sum;
    compression[row] = fmaxf(rho.sum / profile.rest_density - 1.0F, 0.0F);
    active[row] = compression[row] > 0.0F ? 1U : 0U;
    publish_compensation_work(work, profile, additions, 1ULL);
}

__global__ void compute_energy_gradient(const DeviceSample* samples,
    const unsigned int* owner_indices,
    const unsigned int* current_counts, const unsigned int* current_rows,
    const unsigned int* reference_counts, const unsigned int* reference_rows,
    const float* compression, int count, DeviceProfile profile,
    float* row_energy, float* gradient, DeviceCompensationWork* work) {
    const int row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= count) return;
    const DeviceSample sample = samples[owner_indices[row]];
    const DeviceVec3 x = position(sample, 0);
    const DeviceVec3 predicted = position(sample, 1);
    const DeviceVec3 current = position(sample, 2);
    F32Accumulator inertia_energy = make_accumulator(profile);
    F32Accumulator pressure_energy = make_accumulator(profile);
    F32Accumulator viscosity_energy = make_accumulator(profile);
    F32Accumulator surface_energy = make_accumulator(profile);
    F32Accumulator gradient_x = make_accumulator(profile);
    F32Accumulator gradient_y = make_accumulator(profile);
    F32Accumulator gradient_z = make_accumulator(profile);
    unsigned long long additions = 0ULL;
    if ((profile.terms & 1U) != 0U) {
        const float scale = profile.mass / (profile.time_step * profile.time_step);
        const DeviceVec3 displacement = subtract(current, predicted);
        inertia_energy.add(0.5F * scale * dot(displacement, displacement));
        gradient_x.add(scale * displacement.x);
        gradient_y.add(scale * displacement.y);
        gradient_z.add(scale * displacement.z);
        additions += 4ULL;
    }
    if ((profile.terms & 2U) != 0U) {
        pressure_energy.add(0.5F * profile.kappa
            * compression[row] * compression[row]);
        ++additions;
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
            gradient_x.add(scale * displacement.x);
            gradient_y.add(scale * displacement.y);
            gradient_z.add(scale * displacement.z);
            additions += 3ULL;
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
            gradient_x.add(pair.x);
            gradient_y.add(pair.y);
            gradient_z.add(pair.z);
            additions += 3ULL;
            if (neighbor > row) {
                viscosity_energy.add(profile.mass * omega
                    / (profile.rest_density * profile.time_step)
                    * (profile.mu * dot(tangent_part, tangent_part)
                        + 0.5F * profile.lambda * dot(normal_part, normal_part)));
                ++additions;
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
            const DeviceVec3 pair = multiply(2.0F * profile.gamma * profile.mass
                * profile.mass * surface_spline(radius, profile.spacing), normal);
            gradient_x.add(pair.x);
            gradient_y.add(pair.y);
            gradient_z.add(pair.z);
            additions += 3ULL;
            if (neighbor > row) {
                surface_energy.add(2.0F * profile.gamma * profile.mass * profile.mass
                    * surface_potential(radius, profile.spacing));
                ++additions;
            }
        }
    }
    row_energy[4 * row] = inertia_energy.sum;
    row_energy[4 * row + 1] = pressure_energy.sum;
    row_energy[4 * row + 2] = viscosity_energy.sum;
    row_energy[4 * row + 3] = surface_energy.sum;
    gradient[3 * row] = gradient_x.sum;
    gradient[3 * row + 1] = gradient_y.sum;
    gradient[3 * row + 2] = gradient_z.sum;
    publish_compensation_work(work, profile, additions, 7ULL);
}

__global__ void reduce_energy(const float* row_energy, int count,
    DeviceProfile profile, float* energy, DeviceCompensationWork* work) {
    if (blockIdx.x != 0 || threadIdx.x != 0) return;
    for (int term = 0; term < 4; ++term) {
        F32Accumulator value = make_accumulator(profile);
        for (int row = 0; row < count; ++row) {
            value.add(row_energy[4 * row + term]);
        }
        energy[term] = value.sum;
    }
    publish_compensation_work(work, profile,
        4ULL * static_cast<unsigned long long>(count), 4ULL);
}

__global__ void compute_energy_f64(const DeviceSample* samples,
    const unsigned int* owner_indices,
    const unsigned int* current_counts, const unsigned int* current_rows,
    const unsigned int* reference_counts, const unsigned int* reference_rows,
    int count, DeviceProfile profile, double* energy,
    DeviceF64EnergyWork* work) {
    if (blockIdx.x != 0 || threadIdx.x != 0) return;
    double density[ASSEMBLY_MAX_SAMPLES];
    for (int row = 0; row < count; ++row) density[row] = 0.0;
    unsigned long long density_terms = 0ULL;
    unsigned long long particle_terms = 0ULL;
    unsigned long long viscosity_pairs = 0ULL;
    unsigned long long surface_pairs = 0ULL;
    const double horizon = static_cast<double>(profile.horizon);
    const double spacing = static_cast<double>(profile.spacing);
    const double mass = static_cast<double>(profile.mass);
    const double dt = static_cast<double>(profile.time_step);
    const double rest_density = static_cast<double>(profile.rest_density);
    const double kappa = static_cast<double>(profile.kappa);
    const double lambda = static_cast<double>(profile.lambda);
    const double mu = static_cast<double>(profile.mu);
    const double gamma = static_cast<double>(profile.gamma);
    double inertia = 0.0;
    double pressure = 0.0;
    double viscosity = 0.0;
    double surface = 0.0;
    if ((profile.terms & 2U) != 0U) {
        for (int row = 0; row < count; ++row) {
            const DeviceVec3d owner = to_f64(
                position(samples[owner_indices[row]], 2));
            for (unsigned int slot = 0U; slot < current_counts[row]; ++slot) {
                const int neighbor = static_cast<int>(
                    current_rows[row * count + static_cast<int>(slot)]);
                const DeviceVec3d other = to_f64(
                    position(samples[owner_indices[neighbor]], 2));
                density[row] += mass * cubic_weight_f64(
                    length(subtract(owner, other)), horizon);
                ++density_terms;
            }
            const double compression = fmax(
                density[row] / rest_density - 1.0, 0.0);
            pressure += 0.5 * kappa * compression * compression;
            ++particle_terms;
        }
    }
    if ((profile.terms & 1U) != 0U) {
        const double scale = mass / (dt * dt);
        for (int row = 0; row < count; ++row) {
            const DeviceSample sample = samples[owner_indices[row]];
            const DeviceVec3d displacement = subtract(
                to_f64(position(sample, 2)), to_f64(position(sample, 1)));
            inertia += 0.5 * scale * dot(displacement, displacement);
            ++particle_terms;
        }
    }
    if ((profile.terms & 4U) != 0U) {
        for (int row = 0; row < count; ++row) {
            const DeviceSample sample = samples[owner_indices[row]];
            for (unsigned int slot = 0U; slot < reference_counts[row]; ++slot) {
                const int neighbor = static_cast<int>(
                    reference_rows[row * count + static_cast<int>(slot)]);
                if (neighbor <= row) continue;
                const DeviceSample other = samples[owner_indices[neighbor]];
                const DeviceVec3d reference = subtract(
                    to_f64(position(sample, 0)), to_f64(position(other, 0)));
                const double radius = length(reference);
                if (radius <= 1.0e-15) continue;
                const DeviceVec3d normal = multiply(1.0 / radius, reference);
                const DeviceVec3d increment = subtract(subtract(
                    to_f64(position(sample, 2)), to_f64(position(other, 2))),
                    reference);
                const DeviceVec3d normal_part = multiply(
                    dot(increment, normal), normal);
                const DeviceVec3d tangent_part = subtract(
                    increment, normal_part);
                const double omega = -cubic_gradient_f64(radius, horizon);
                viscosity += mass * omega / (rest_density * dt)
                    * (mu * dot(tangent_part, tangent_part)
                        + 0.5 * lambda * dot(normal_part, normal_part));
                ++viscosity_pairs;
            }
        }
    }
    if ((profile.terms & 8U) != 0U) {
        for (int row = 0; row < count; ++row) {
            const DeviceVec3d owner = to_f64(
                position(samples[owner_indices[row]], 2));
            for (unsigned int slot = 0U; slot < current_counts[row]; ++slot) {
                const int neighbor = static_cast<int>(
                    current_rows[row * count + static_cast<int>(slot)]);
                if (neighbor <= row) continue;
                const DeviceVec3d other = to_f64(
                    position(samples[owner_indices[neighbor]], 2));
                const double radius = length(subtract(owner, other));
                if (radius <= 1.0e-15 || radius >= 3.0 * spacing) continue;
                surface += 2.0 * gamma * mass * mass
                    * surface_potential_f64(radius, spacing);
                ++surface_pairs;
            }
        }
    }
    energy[0] = inertia;
    energy[1] = pressure;
    energy[2] = viscosity;
    energy[3] = surface;
    work->density_terms = density_terms;
    work->particle_terms = particle_terms;
    work->viscosity_pairs = viscosity_pairs;
    work->surface_pairs = surface_pairs;
    work->component_writes = 4ULL;
}

__global__ void build_pressure_jacobian(const DeviceSample* samples,
    const unsigned int* owner_indices, const unsigned int* current_counts,
    const unsigned int* current_rows, const unsigned char* active,
    int count, DeviceProfile profile, float* jacobian,
    DeviceCompensationWork* work) {
    const int dimension = 3 * count;
    const int scalar = blockIdx.x * blockDim.x + threadIdx.x;
    if (scalar >= count * dimension) return;
    const int center = scalar / dimension;
    const int degree = scalar % dimension;
    if (active[center] == 0U || (profile.terms & 2U) == 0U) {
        jacobian[scalar] = 0.0F;
        publish_compensation_work(work, profile, 0ULL, 1ULL);
        return;
    }
    const int participant = degree / 3;
    const int axis = degree % 3;
    const DeviceVec3 center_position = position(samples[owner_indices[center]], 2);
    F32Accumulator value = make_accumulator(profile);
    unsigned long long additions = 0ULL;
    if (participant == center) {
        for (unsigned int slot = 0U; slot < current_counts[center]; ++slot) {
            const int neighbor = static_cast<int>(
                current_rows[center * count + static_cast<int>(slot)]);
            if (neighbor == center) continue;
            const DeviceVec3 displacement = subtract(center_position,
                position(samples[owner_indices[neighbor]], 2));
            const float radius = length(displacement);
            if (radius <= 1.0e-15F) continue;
            value.add(profile.mass / profile.rest_density
                * cubic_gradient(radius, profile) / radius
                * component(displacement, axis));
            ++additions;
        }
    } else if (row_contains(current_counts, current_rows,
                   count, center, participant)) {
        const DeviceVec3 displacement = subtract(center_position,
            position(samples[owner_indices[participant]], 2));
        const float radius = length(displacement);
        if (radius > 1.0e-15F) {
            value.add(-profile.mass / profile.rest_density
                * cubic_gradient(radius, profile) / radius
                * component(displacement, axis));
            ++additions;
        }
    }
    jacobian[scalar] = value.sum;
    publish_compensation_work(work, profile, additions, 1ULL);
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
    const float* jacobian, int count, DeviceProfile profile, float* hessian,
    DeviceCompensationWork* work) {
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
        publish_compensation_work(work, profile, 1ULL, 1ULL);
        return;
    }
    F32Accumulator value = make_accumulator(profile);
    unsigned long long additions = 0ULL;
    if ((profile.terms & 1U) != 0U && row == column) {
        value.add(profile.mass / (profile.time_step * profile.time_step));
        ++additions;
    }
    if ((profile.terms & 2U) != 0U) {
        for (int center = 0; center < count; ++center) {
            if (active[center] == 0U) continue;
            value.add(profile.kappa * jacobian[center * dimension + row]
                * jacobian[center * dimension + column]);
            ++additions;
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
                    value.add(profile.kappa * compression[center]
                        * profile.mass / profile.rest_density
                        * radial_entry(normal, cubic_second(radius, profile),
                            cubic_gradient(radius, profile) / radius,
                            row_axis, column_axis));
                    ++additions;
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
                        value.add(sign * profile.kappa * compression[center]
                            * profile.mass / profile.rest_density
                            * radial_entry(normal, cubic_second(radius, profile),
                                cubic_gradient(radius, profile) / radius,
                                row_axis, column_axis));
                        ++additions;
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
                value.add(pair_curvature_entry(samples, owner_indices,
                    row_particle, neighbor, row_axis, column_axis, profile, 0));
                ++additions;
            }
        } else if (row_contains(viscosity_counts, viscosity_rows,
                       count, row_particle, column_particle)) {
            value.add(-pair_curvature_entry(samples, owner_indices,
                row_particle, column_particle, row_axis, column_axis, profile, 0));
            ++additions;
        }
    }
    if ((profile.terms & 8U) != 0U) {
        if (row_particle == column_particle) {
            for (unsigned int slot = 0U; slot < current_counts[row_particle]; ++slot) {
                const int neighbor = static_cast<int>(
                    current_rows[row_particle * count + static_cast<int>(slot)]);
                if (neighbor == row_particle) continue;
                value.add(pair_curvature_entry(samples, owner_indices,
                    row_particle, neighbor, row_axis, column_axis, profile, 1));
                ++additions;
            }
        } else if (row_contains(current_counts, current_rows,
                       count, row_particle, column_particle)) {
            value.add(-pair_curvature_entry(samples, owner_indices,
                row_particle, column_particle, row_axis, column_axis, profile, 1));
            ++additions;
        }
    }
    hessian[entry] = value.sum;
    publish_compensation_work(work, profile, additions, 1ULL);
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
    const float* jacobian, int count, DeviceProfile profile, float* hvp,
    DeviceCompensationWork* work) {
    const int row = blockIdx.x * blockDim.x + threadIdx.x;
    const int dimension = 3 * count;
    if (row >= dimension) return;
    const int particle = row / 3;
    const int axis = row % 3;
    F32Accumulator value = make_accumulator(profile);
    unsigned long long additions = 0ULL;
    unsigned long long initializations = 1ULL;
    if ((profile.terms & 1U) != 0U) {
        value.add(profile.mass / (profile.time_step * profile.time_step)
            * component(direction(samples[owner_indices[particle]]), axis));
        ++additions;
    }
    if ((profile.terms & 2U) != 0U) {
        for (int center = 0; center < count; ++center) {
            if (active[center] == 0U) continue;
            F32Accumulator density_direction = make_accumulator(profile);
            ++initializations;
            for (int column = 0; column < dimension; ++column) {
                density_direction.add(jacobian[center * dimension + column]
                    * component(direction(samples[owner_indices[column / 3]]),
                        column % 3));
                ++additions;
            }
            value.add(profile.kappa * jacobian[center * dimension + row]
                * density_direction.sum);
            ++additions;
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
                    F32Accumulator product = make_accumulator(profile);
                    ++initializations;
                    for (int column_axis = 0; column_axis < 3; ++column_axis) {
                        product.add(radial_entry(normal, cubic_second(radius, profile),
                            cubic_gradient(radius, profile) / radius,
                            axis, column_axis)
                            * component(relative_direction, column_axis));
                        ++additions;
                    }
                    value.add(profile.kappa * compression[center]
                        * profile.mass / profile.rest_density * product.sum);
                    ++additions;
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
                    F32Accumulator product = make_accumulator(profile);
                    ++initializations;
                    for (int column_axis = 0; column_axis < 3; ++column_axis) {
                        product.add(radial_entry(normal, cubic_second(radius, profile),
                            cubic_gradient(radius, profile) / radius,
                            axis, column_axis)
                            * component(relative_direction, column_axis));
                        ++additions;
                    }
                    value.add(-profile.kappa * compression[center]
                        * profile.mass / profile.rest_density * product.sum);
                    ++additions;
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
                value.add(pair_curvature_entry(samples, owner_indices,
                    particle, neighbor, axis, column_axis, profile, 0)
                    * component(relative_direction, column_axis));
                ++additions;
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
                value.add(pair_curvature_entry(samples, owner_indices,
                    particle, neighbor, axis, column_axis, profile, 1)
                    * component(relative_direction, column_axis));
                ++additions;
            }
        }
    }
    hvp[row] = value.sum;
    publish_compensation_work(
        work, profile, additions, initializations);
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

AssemblyResult evaluate_gpu_assembly_impl(const AssemblyProfile& profile,
    const AssemblyFixture& fixture,
    const std::vector<double>* canonical_current_m,
    const AssemblyContinuousState* canonical_continuous_state,
    AssemblyVariant variant) {
    AssemblyResult output;
    if (!admitted(profile, fixture, output.failure)) return output;
    if (canonical_current_m != nullptr && canonical_continuous_state != nullptr) {
        output.failure = AssemblyFailure::InvalidInput;
        return output;
    }
    const std::size_t scalar_count = 3U * fixture.samples.size();
    const std::vector<double>* reference_m = canonical_continuous_state == nullptr
        ? nullptr : &canonical_continuous_state->reference_m;
    const std::vector<double>* predicted_m = canonical_continuous_state == nullptr
        ? nullptr : &canonical_continuous_state->predicted_m;
    const std::vector<double>* current_m = canonical_continuous_state == nullptr
        ? canonical_current_m : &canonical_continuous_state->current_m;
    if ((reference_m != nullptr && reference_m->size() != scalar_count)
        || (predicted_m != nullptr && predicted_m->size() != scalar_count)
        || (current_m != nullptr && current_m->size() != scalar_count)) {
        output.failure = AssemblyFailure::InvalidInput;
        return output;
    }
    std::vector<std::uint32_t> canonical_ids;
    canonical_ids.reserve(fixture.samples.size());
    for (const AssemblySample& sample : fixture.samples) {
        canonical_ids.push_back(sample.sample_id);
    }
    std::sort(canonical_ids.begin(), canonical_ids.end());
    const int count = static_cast<int>(fixture.samples.size());
    const int dimension = 3 * count;
    const std::size_t dense_entries = static_cast<std::size_t>(dimension) * dimension;
    std::vector<DeviceSample> host_samples;
    host_samples.reserve(fixture.samples.size());
    for (const AssemblySample& sample : fixture.samples) {
        DeviceSample value{};
        value.id = sample.sample_id;
        const std::size_t canonical_row = static_cast<std::size_t>(
            std::lower_bound(canonical_ids.begin(), canonical_ids.end(), sample.sample_id)
            - canonical_ids.begin());
        for (std::size_t axis = 0; axis < 3U; ++axis) {
            value.reference[axis] = sample.reference_um[axis];
            value.predicted[axis] = sample.predicted_um[axis];
            value.current[axis] = sample.current_um[axis];
            const double reference = reference_m == nullptr
                ? 1.0e-6 * static_cast<double>(sample.reference_um[axis])
                : (*reference_m)[3U * canonical_row + axis];
            const double predicted = predicted_m == nullptr
                ? 1.0e-6 * static_cast<double>(sample.predicted_um[axis])
                : (*predicted_m)[3U * canonical_row + axis];
            const double current = current_m == nullptr
                ? 1.0e-6 * static_cast<double>(sample.current_um[axis])
                : (*current_m)[3U * canonical_row + axis];
            if (!std::isfinite(reference) || std::abs(reference) > 1000.0
                || !std::isfinite(static_cast<float>(reference))
                || !std::isfinite(predicted) || std::abs(predicted) > 1000.0
                || !std::isfinite(static_cast<float>(predicted))
                || !std::isfinite(current) || std::abs(current) > 1000.0
                || !std::isfinite(static_cast<float>(current))) {
                output.failure = AssemblyFailure::InvalidInput;
                return output;
            }
            value.reference_m[axis] = reference_m == nullptr
                ? 1.0e-6F * static_cast<float>(sample.reference_um[axis])
                : static_cast<float>(reference);
            value.predicted_m[axis] = predicted_m == nullptr
                ? 1.0e-6F * static_cast<float>(sample.predicted_um[axis])
                : static_cast<float>(predicted);
            value.current_m[axis] = current_m == nullptr
                ? 1.0e-6F * static_cast<float>(sample.current_um[axis])
                : static_cast<float>(current);
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
    DeviceBuffer<DeviceCompensationWork> compensation_work(1U);
    const bool use_f64_energy = variant == AssemblyVariant::F64Energy;
    DeviceBuffer<double> energy_f64;
    DeviceBuffer<DeviceF64EnergyWork> energy_f64_work;
    if (use_f64_energy) {
        energy_f64.allocate(4U);
        energy_f64_work.allocate(1U);
    }

    check_cuda(cudaMemcpy(device_samples.get(), host_samples.data(),
                   host_samples.size() * sizeof(DeviceSample), cudaMemcpyHostToDevice),
        "upload NCGA2 samples");
    check_cuda(cudaMemset(sort_comparisons.get(), 0, sizeof(unsigned long long)),
        "reset NCGA2 sort work");
    check_cuda(cudaMemset(compensation_work.get(), 0,
                   sizeof(DeviceCompensationWork)),
        "reset NCGA2 compensation work");
    if (use_f64_energy) {
        check_cuda(cudaMemset(energy_f64_work.get(), 0,
                       sizeof(DeviceF64EnergyWork)),
            "reset NCGA6 f64 energy work");
    }
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
        active.get(), compensation_work.get());
    check_cuda(cudaGetLastError(), "launch NCGA2 density");
    compute_energy_gradient<<<blocks_for(host_samples.size()), THREADS>>>(
        device_samples.get(), owner_indices.get(), current_counts.get(),
        current_rows.get(), reference_counts.get(), reference_rows.get(),
        compression.get(), count, device_profile, row_energy.get(), gradient.get(),
        compensation_work.get());
    check_cuda(cudaGetLastError(), "launch NCGA2 energy/gradient");
    reduce_energy<<<1, 1>>>(row_energy.get(), count, device_profile,
        energy.get(), compensation_work.get());
    check_cuda(cudaGetLastError(), "launch NCGA2 energy reduction");
    if (use_f64_energy) {
        compute_energy_f64<<<1, 1>>>(device_samples.get(), owner_indices.get(),
            current_counts.get(), current_rows.get(), reference_counts.get(),
            reference_rows.get(), count, device_profile, energy_f64.get(),
            energy_f64_work.get());
        check_cuda(cudaGetLastError(), "launch NCGA6 f64 energy");
    }
    build_pressure_jacobian<<<blocks_for(static_cast<std::size_t>(count) * dimension),
        THREADS>>>(device_samples.get(), owner_indices.get(), current_counts.get(),
        current_rows.get(), active.get(), count, device_profile, jacobian.get(),
        compensation_work.get());
    check_cuda(cudaGetLastError(), "launch NCGA2 pressure Jacobian");
    assemble_hessian<<<blocks_for(dense_entries), THREADS>>>(device_samples.get(),
        owner_indices.get(), current_counts.get(), current_rows.get(),
        reference_counts.get(), reference_rows.get(), compression.get(), active.get(),
        jacobian.get(), count, device_profile, hessian.get(),
        compensation_work.get());
    check_cuda(cudaGetLastError(), "launch NCGA2 Hessian");
    extract_blocks<<<blocks_for(9U * host_samples.size()), THREADS>>>(hessian.get(),
        count, diagonal_blocks.get());
    check_cuda(cudaGetLastError(), "launch NCGA2 diagonal blocks");
    direct_hvp<<<blocks_for(static_cast<std::size_t>(dimension)), THREADS>>>(
        device_samples.get(), owner_indices.get(), current_counts.get(),
        current_rows.get(), reference_counts.get(), reference_rows.get(),
        compression.get(), active.get(), jacobian.get(), count, device_profile,
        hvp.get(), compensation_work.get());
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
    std::array<double, 4> host_energy_f64{};
    std::vector<float> host_gradient(static_cast<std::size_t>(dimension));
    std::vector<float> host_jacobian(static_cast<std::size_t>(count) * dimension);
    std::vector<float> host_hessian(dense_entries);
    std::vector<float> host_blocks(9U * host_samples.size());
    std::vector<float> host_hvp(static_cast<std::size_t>(dimension));
    unsigned long long host_sort_comparisons = 0ULL;
    DeviceCompensationWork host_compensation_work{};
    DeviceF64EnergyWork host_energy_f64_work{};
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
    if (use_f64_energy) {
        NCGA2_COPY(host_energy_f64.data(), energy_f64.get(),
            sizeof(host_energy_f64), "copy NCGA6 f64 energy");
        NCGA2_COPY(&host_energy_f64_work, energy_f64_work.get(),
            sizeof(host_energy_f64_work), "copy NCGA6 f64 energy work");
    }
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
    NCGA2_COPY(&host_compensation_work, compensation_work.get(),
        sizeof(host_compensation_work), "copy NCGA2 compensation work");
#undef NCGA2_COPY

    output.owner_ids.assign(host_owner_ids.begin(), host_owner_ids.end());
    build_csr(host_current_counts, host_current_rows, host_owner_ids,
        output.current_offsets, output.current_neighbors);
    build_csr(host_reference_counts, host_reference_rows, host_owner_ids,
        output.reference_offsets, output.reference_neighbors);
    output.density.assign(host_density.begin(), host_density.end());
    for (std::size_t index = 0; index < 4U; ++index) {
        output.energy[index] = use_f64_energy
            ? host_energy_f64[index] : static_cast<double>(host_energy[index]);
    }
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
        + host_hessian.size() + host_blocks.size() + host_hvp.size() + 3U;
    output.work.compensated_additions = host_compensation_work.additions;
    output.work.compensation_initializations
        = host_compensation_work.initializations;
    if (use_f64_energy) {
        output.work.f64_energy_density_terms
            = host_energy_f64_work.density_terms;
        output.work.f64_energy_particle_terms
            = host_energy_f64_work.particle_terms;
        output.work.f64_energy_viscosity_pairs
            = host_energy_f64_work.viscosity_pairs;
        output.work.f64_energy_surface_pairs
            = host_energy_f64_work.surface_pairs;
        output.work.f64_energy_component_writes
            = host_energy_f64_work.component_writes;
        output.work.host_device_scalar_transfers += 9U;
    }
    output.failure = AssemblyFailure::None;
    return output;
}

AssemblyResult evaluate_gpu_assembly(const AssemblyProfile& profile,
    const AssemblyFixture& fixture, AssemblyVariant variant) {
    return evaluate_gpu_assembly_impl(
        profile, fixture, nullptr, nullptr, variant);
}

AssemblyResult evaluate_gpu_assembly_at(const AssemblyProfile& profile,
    const AssemblyFixture& fixture,
    const std::vector<double>& canonical_current_m,
    AssemblyVariant variant) {
    return evaluate_gpu_assembly_impl(
        profile, fixture, &canonical_current_m, nullptr, variant);
}

AssemblyResult evaluate_gpu_assembly_state(const AssemblyProfile& profile,
    const AssemblyFixture& fixture,
    const AssemblyContinuousState& canonical_state,
    AssemblyVariant variant) {
    return evaluate_gpu_assembly_impl(
        profile, fixture, nullptr, &canonical_state, variant);
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
