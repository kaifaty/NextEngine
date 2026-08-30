#include "corrected_cuda_consequence.hpp"

#include <cuda_runtime.h>

#include <algorithm>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <stdexcept>
#include <string>
#include <vector>

namespace nextengine::nonlocal::gpu_assembly_audit {
namespace {

constexpr int THREADS = 128;

struct DeviceMixedSample {
    unsigned int id;
    long long current[3];
};

struct DeviceMixedProfile {
    float horizon;
    float mass;
    float rest_density;
    float kappa;
    unsigned int arithmetic;
};

struct DeviceMixedWork {
    unsigned long long density_terms;
    unsigned long long jacobian_terms;
    unsigned long long outer_products;
    unsigned long long geometric_products;
};

struct DeviceVec3f {
    float x;
    float y;
    float z;
};

struct DeviceVec3d {
    double x;
    double y;
    double z;
};

struct F32Accumulator {
    float sum = 0.0F;
    float correction = 0.0F;

    __device__ void add(float value) {
        const float adjusted = value - correction;
        const float next = sum + adjusted;
        correction = (next - sum) - adjusted;
        sum = next;
    }
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
            "cudaMalloc NCGA3 buffer");
    }
    T* get() { return data_; }
    const T* get() const { return data_; }

private:
    T* data_ = nullptr;
};

__device__ unsigned long long magnitude(long long value) {
    return static_cast<unsigned long long>(value < 0 ? -value : value);
}

__device__ bool inside_support(
    const DeviceMixedSample& lhs, const DeviceMixedSample& rhs) {
    unsigned long long squared = 0ULL;
    for (int axis = 0; axis < 3; ++axis) {
        const unsigned long long value = magnitude(lhs.current[axis] - rhs.current[axis]);
        if (value > static_cast<unsigned long long>(ASSEMBLY_SUPPORT_UM)) return false;
        squared += value * value;
    }
    return squared <= static_cast<unsigned long long>(
        ASSEMBLY_SUPPORT_UM * ASSEMBLY_SUPPORT_UM);
}

__device__ DeviceVec3f position_f32(const DeviceMixedSample& sample) {
    constexpr float scale = 1.0e-6F;
    return {scale * static_cast<float>(sample.current[0]),
        scale * static_cast<float>(sample.current[1]),
        scale * static_cast<float>(sample.current[2])};
}

__device__ DeviceVec3d position_f64_from_f32(const DeviceMixedSample& sample) {
    const DeviceVec3f value = position_f32(sample);
    return {static_cast<double>(value.x), static_cast<double>(value.y),
        static_cast<double>(value.z)};
}

__device__ DeviceVec3f subtract(DeviceVec3f lhs, DeviceVec3f rhs) {
    return {lhs.x - rhs.x, lhs.y - rhs.y, lhs.z - rhs.z};
}

__device__ DeviceVec3d subtract(DeviceVec3d lhs, DeviceVec3d rhs) {
    return {lhs.x - rhs.x, lhs.y - rhs.y, lhs.z - rhs.z};
}

__device__ float component(DeviceVec3f value, int axis) {
    return axis == 0 ? value.x : (axis == 1 ? value.y : value.z);
}

__device__ double component(DeviceVec3d value, int axis) {
    return axis == 0 ? value.x : (axis == 1 ? value.y : value.z);
}

__device__ float dot(DeviceVec3f lhs, DeviceVec3f rhs) {
    return lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z;
}

__device__ double dot(DeviceVec3d lhs, DeviceVec3d rhs) {
    return lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z;
}

__device__ float length(DeviceVec3f value) { return sqrtf(dot(value, value)); }
__device__ double length(DeviceVec3d value) { return sqrt(dot(value, value)); }

__device__ float cubic_weight_f32(float radius, float horizon) {
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

__device__ float cubic_gradient_f32(float radius, float horizon) {
    constexpr float pi = 3.14159265358979323846F;
    const float q = 2.0F * radius / horizon;
    const float alpha = 3.0F / (2.0F * pi * horizon * horizon * horizon);
    if (q > 2.0F) return 0.0F;
    const float derivative_q = q >= 1.0F
        ? -0.5F * alpha * (2.0F - q) * (2.0F - q)
        : alpha * (-2.0F * q + 1.5F * q * q);
    return derivative_q * 2.0F / horizon;
}

__device__ float cubic_second_f32(float radius, float horizon) {
    constexpr float pi = 3.14159265358979323846F;
    const float q = 2.0F * radius / horizon;
    const float alpha = 3.0F / (2.0F * pi * horizon * horizon * horizon);
    if (q > 2.0F) return 0.0F;
    const float second_q = q >= 1.0F
        ? alpha * (2.0F - q) : alpha * (-2.0F + 3.0F * q);
    return second_q * 4.0F / (horizon * horizon);
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

__device__ double cubic_second_f64(double radius, double horizon) {
    constexpr float pi_f32 = 3.14159265358979323846F;
    const double pi = static_cast<double>(pi_f32);
    const double q = 2.0 * radius / horizon;
    const double alpha = 3.0 / (2.0 * pi * horizon * horizon * horizon);
    if (q > 2.0) return 0.0;
    const double second_q = q >= 1.0
        ? alpha * (2.0 - q) : alpha * (-2.0 + 3.0 * q);
    return second_q * 4.0 / (horizon * horizon);
}

__device__ float radial_entry(DeviceVec3f normal, float radial,
    float tangential, int row_axis, int column_axis) {
    return tangential * (row_axis == column_axis ? 1.0F : 0.0F)
        + (radial - tangential) * component(normal, row_axis)
            * component(normal, column_axis);
}

__device__ double radial_entry(DeviceVec3d normal, double radial,
    double tangential, int row_axis, int column_axis) {
    return tangential * (row_axis == column_axis ? 1.0 : 0.0)
        + (radial - tangential) * component(normal, row_axis)
            * component(normal, column_axis);
}

__global__ void compute_mixed_density(const DeviceMixedSample* samples, int count,
    DeviceMixedProfile profile, double* density, double* compression,
    unsigned char* active, DeviceMixedWork* work) {
    const int row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= count) return;
    unsigned long long terms = 0ULL;
    if (profile.arithmetic
        == static_cast<unsigned int>(PressureArithmetic::F32ProductsF64Reduction)) {
        const DeviceVec3f owner = position_f32(samples[row]);
        F32Accumulator sum;
        for (int neighbor = 0; neighbor < count; ++neighbor) {
            if (!inside_support(samples[row], samples[neighbor])) continue;
            sum.add(profile.mass * cubic_weight_f32(
                length(subtract(owner, position_f32(samples[neighbor]))), profile.horizon));
            ++terms;
        }
        density[row] = static_cast<double>(sum.sum);
        const float value = fmaxf(sum.sum / profile.rest_density - 1.0F, 0.0F);
        compression[row] = static_cast<double>(value);
        active[row] = value > 0.0F ? 1U : 0U;
    } else {
        const double horizon = static_cast<double>(profile.horizon);
        const double mass = static_cast<double>(profile.mass);
        const double rest_density = static_cast<double>(profile.rest_density);
        const DeviceVec3d owner = position_f64_from_f32(samples[row]);
        double sum = 0.0;
        for (int neighbor = 0; neighbor < count; ++neighbor) {
            if (!inside_support(samples[row], samples[neighbor])) continue;
            sum += mass * cubic_weight_f64(
                length(subtract(owner, position_f64_from_f32(samples[neighbor]))), horizon);
            ++terms;
        }
        density[row] = sum;
        const double value = fmax(sum / rest_density - 1.0, 0.0);
        compression[row] = value;
        active[row] = value > 0.0 ? 1U : 0U;
    }
    atomicAdd(&work->density_terms, terms);
}

__global__ void compute_mixed_jacobian(const DeviceMixedSample* samples,
    const unsigned char* active, int count, DeviceMixedProfile profile,
    double* jacobian, DeviceMixedWork* work) {
    const int dimension = 3 * count;
    const int scalar = blockIdx.x * blockDim.x + threadIdx.x;
    if (scalar >= count * dimension) return;
    const int center = scalar / dimension;
    const int degree = scalar % dimension;
    if (active[center] == 0U) {
        jacobian[scalar] = 0.0;
        return;
    }
    const int participant = degree / 3;
    const int axis = degree % 3;
    unsigned long long terms = 0ULL;
    if (profile.arithmetic
        == static_cast<unsigned int>(PressureArithmetic::F32ProductsF64Reduction)) {
        const DeviceVec3f center_position = position_f32(samples[center]);
        F32Accumulator value;
        if (participant == center) {
            for (int neighbor = 0; neighbor < count; ++neighbor) {
                if (neighbor == center || !inside_support(samples[center], samples[neighbor])) {
                    continue;
                }
                const DeviceVec3f displacement = subtract(
                    center_position, position_f32(samples[neighbor]));
                const float radius = length(displacement);
                if (radius <= 1.0e-15F) continue;
                value.add(profile.mass / profile.rest_density
                    * cubic_gradient_f32(radius, profile.horizon) / radius
                    * component(displacement, axis));
                ++terms;
            }
        } else if (inside_support(samples[center], samples[participant])) {
            const DeviceVec3f displacement = subtract(
                center_position, position_f32(samples[participant]));
            const float radius = length(displacement);
            if (radius > 1.0e-15F) {
                value.add(-profile.mass / profile.rest_density
                    * cubic_gradient_f32(radius, profile.horizon) / radius
                    * component(displacement, axis));
                ++terms;
            }
        }
        jacobian[scalar] = static_cast<double>(value.sum);
    } else {
        const double horizon = static_cast<double>(profile.horizon);
        const double mass = static_cast<double>(profile.mass);
        const double rest_density = static_cast<double>(profile.rest_density);
        const DeviceVec3d center_position = position_f64_from_f32(samples[center]);
        double value = 0.0;
        if (participant == center) {
            for (int neighbor = 0; neighbor < count; ++neighbor) {
                if (neighbor == center || !inside_support(samples[center], samples[neighbor])) {
                    continue;
                }
                const DeviceVec3d displacement = subtract(
                    center_position, position_f64_from_f32(samples[neighbor]));
                const double radius = length(displacement);
                if (radius <= 1.0e-15) continue;
                value += mass / rest_density * cubic_gradient_f64(radius, horizon)
                    / radius * component(displacement, axis);
                ++terms;
            }
        } else if (inside_support(samples[center], samples[participant])) {
            const DeviceVec3d displacement = subtract(
                center_position, position_f64_from_f32(samples[participant]));
            const double radius = length(displacement);
            if (radius > 1.0e-15) {
                value += -mass / rest_density * cubic_gradient_f64(radius, horizon)
                    / radius * component(displacement, axis);
                ++terms;
            }
        }
        jacobian[scalar] = value;
    }
    atomicAdd(&work->jacobian_terms, terms);
}

__global__ void assemble_mixed_pressure(const DeviceMixedSample* samples,
    const double* compression, const unsigned char* active,
    const double* jacobian, const float* base_hessian, int count,
    DeviceMixedProfile profile, float* hessian, DeviceMixedWork* work) {
    const int dimension = 3 * count;
    const int entry = blockIdx.x * blockDim.x + threadIdx.x;
    if (entry >= dimension * dimension) return;
    const int row = entry / dimension;
    const int column = entry % dimension;
    const int row_particle = row / 3;
    const int column_particle = column / 3;
    const int row_axis = row % 3;
    const int column_axis = column % 3;
    double sum = 0.0;
    unsigned long long outer = 0ULL;
    unsigned long long geometric = 0ULL;
    if (profile.arithmetic
        == static_cast<unsigned int>(PressureArithmetic::F32ProductsF64Reduction)) {
        for (int center = 0; center < count; ++center) {
            if (active[center] == 0U) continue;
            const float outer_product = profile.kappa
                * static_cast<float>(jacobian[center * dimension + row])
                * static_cast<float>(jacobian[center * dimension + column]);
            sum += static_cast<double>(outer_product);
            ++outer;
            const bool row_center = row_particle == center;
            const bool column_center = column_particle == center;
            if (row_center && column_center) {
                const DeviceVec3f center_position = position_f32(samples[center]);
                for (int neighbor = 0; neighbor < count; ++neighbor) {
                    if (neighbor == center
                        || !inside_support(samples[center], samples[neighbor])) continue;
                    const DeviceVec3f displacement = subtract(
                        center_position, position_f32(samples[neighbor]));
                    const float radius = length(displacement);
                    if (radius <= 1.0e-15F) continue;
                    const float inverse = 1.0F / radius;
                    const DeviceVec3f normal{inverse * displacement.x,
                        inverse * displacement.y, inverse * displacement.z};
                    const float completed = profile.kappa
                        * static_cast<float>(compression[center])
                        * profile.mass / profile.rest_density
                        * radial_entry(normal,
                            cubic_second_f32(radius, profile.horizon),
                            cubic_gradient_f32(radius, profile.horizon) / radius,
                            row_axis, column_axis);
                    sum += static_cast<double>(completed);
                    ++geometric;
                }
            } else {
                int neighbor = -1;
                float sign = 0.0F;
                if (row_center && column_particle != center
                    && inside_support(samples[center], samples[column_particle])) {
                    neighbor = column_particle;
                    sign = -1.0F;
                } else if (column_center && row_particle != center
                    && inside_support(samples[center], samples[row_particle])) {
                    neighbor = row_particle;
                    sign = -1.0F;
                } else if (row_particle == column_particle && row_particle != center
                    && inside_support(samples[center], samples[row_particle])) {
                    neighbor = row_particle;
                    sign = 1.0F;
                }
                if (neighbor >= 0) {
                    const DeviceVec3f displacement = subtract(
                        position_f32(samples[center]), position_f32(samples[neighbor]));
                    const float radius = length(displacement);
                    if (radius > 1.0e-15F) {
                        const float inverse = 1.0F / radius;
                        const DeviceVec3f normal{inverse * displacement.x,
                            inverse * displacement.y, inverse * displacement.z};
                        const float completed = sign * profile.kappa
                            * static_cast<float>(compression[center])
                            * profile.mass / profile.rest_density
                            * radial_entry(normal,
                                cubic_second_f32(radius, profile.horizon),
                                cubic_gradient_f32(radius, profile.horizon) / radius,
                                row_axis, column_axis);
                        sum += static_cast<double>(completed);
                        ++geometric;
                    }
                }
            }
        }
    } else {
        const double horizon = static_cast<double>(profile.horizon);
        const double mass = static_cast<double>(profile.mass);
        const double rest_density = static_cast<double>(profile.rest_density);
        const double kappa = static_cast<double>(profile.kappa);
        for (int center = 0; center < count; ++center) {
            if (active[center] == 0U) continue;
            sum += kappa * jacobian[center * dimension + row]
                * jacobian[center * dimension + column];
            ++outer;
            const bool row_center = row_particle == center;
            const bool column_center = column_particle == center;
            if (row_center && column_center) {
                const DeviceVec3d center_position = position_f64_from_f32(samples[center]);
                for (int neighbor = 0; neighbor < count; ++neighbor) {
                    if (neighbor == center
                        || !inside_support(samples[center], samples[neighbor])) continue;
                    const DeviceVec3d displacement = subtract(
                        center_position, position_f64_from_f32(samples[neighbor]));
                    const double radius = length(displacement);
                    if (radius <= 1.0e-15) continue;
                    const double inverse = 1.0 / radius;
                    const DeviceVec3d normal{inverse * displacement.x,
                        inverse * displacement.y, inverse * displacement.z};
                    sum += kappa * compression[center] * mass / rest_density
                        * radial_entry(normal, cubic_second_f64(radius, horizon),
                            cubic_gradient_f64(radius, horizon) / radius,
                            row_axis, column_axis);
                    ++geometric;
                }
            } else {
                int neighbor = -1;
                double sign = 0.0;
                if (row_center && column_particle != center
                    && inside_support(samples[center], samples[column_particle])) {
                    neighbor = column_particle;
                    sign = -1.0;
                } else if (column_center && row_particle != center
                    && inside_support(samples[center], samples[row_particle])) {
                    neighbor = row_particle;
                    sign = -1.0;
                } else if (row_particle == column_particle && row_particle != center
                    && inside_support(samples[center], samples[row_particle])) {
                    neighbor = row_particle;
                    sign = 1.0;
                }
                if (neighbor >= 0) {
                    const DeviceVec3d displacement = subtract(
                        position_f64_from_f32(samples[center]),
                        position_f64_from_f32(samples[neighbor]));
                    const double radius = length(displacement);
                    if (radius > 1.0e-15) {
                        const double inverse = 1.0 / radius;
                        const DeviceVec3d normal{inverse * displacement.x,
                            inverse * displacement.y, inverse * displacement.z};
                        sum += sign * kappa * compression[center] * mass / rest_density
                            * radial_entry(normal, cubic_second_f64(radius, horizon),
                                cubic_gradient_f64(radius, horizon) / radius,
                                row_axis, column_axis);
                        ++geometric;
                    }
                }
            }
        }
    }
    hessian[entry] = static_cast<float>(static_cast<double>(base_hessian[entry]) + sum);
    atomicAdd(&work->outer_products, outer);
    atomicAdd(&work->geometric_products, geometric);
}

} // namespace

MixedPressureResult evaluate_gpu_mixed_pressure_hessian(
    const AssemblyProfile& profile, const AssemblyFixture& fixture,
    PressureArithmetic arithmetic) {
    MixedPressureResult output;
    const AssemblyResult strict = evaluate_gpu_assembly(
        profile, fixture, AssemblyVariant::Corrected);
    if (strict.failure != AssemblyFailure::None) {
        output.failure = strict.failure;
        return output;
    }
    AssemblyFixture no_pressure = fixture;
    no_pressure.terms.pressure = false;
    const AssemblyResult base = evaluate_gpu_assembly(
        profile, no_pressure, AssemblyVariant::Corrected);
    if (base.failure != AssemblyFailure::None) {
        output.failure = base.failure;
        return output;
    }

    std::vector<const AssemblySample*> ordered;
    ordered.reserve(fixture.samples.size());
    for (const AssemblySample& sample : fixture.samples) ordered.push_back(&sample);
    std::sort(ordered.begin(), ordered.end(), [](const AssemblySample* lhs,
        const AssemblySample* rhs) { return lhs->sample_id < rhs->sample_id; });
    std::vector<DeviceMixedSample> samples;
    samples.reserve(ordered.size());
    for (const AssemblySample* sample : ordered) {
        DeviceMixedSample value{};
        value.id = sample->sample_id;
        for (std::size_t axis = 0; axis < 3U; ++axis) {
            value.current[axis] = sample->current_um[axis];
        }
        samples.push_back(value);
    }
    const int count = static_cast<int>(samples.size());
    const int dimension = 3 * count;
    const std::size_t dense_entries = static_cast<std::size_t>(dimension) * dimension;
    std::vector<float> base_hessian;
    base_hessian.reserve(base.hessian.size());
    for (double value : base.hessian) base_hessian.push_back(static_cast<float>(value));

    const DeviceMixedProfile device_profile{
        static_cast<float>(profile.horizon), static_cast<float>(profile.mass),
        static_cast<float>(profile.rest_density), static_cast<float>(profile.kappa),
        static_cast<unsigned int>(arithmetic)};
    DeviceBuffer<DeviceMixedSample> device_samples(samples.size());
    DeviceBuffer<double> density(samples.size());
    DeviceBuffer<double> compression(samples.size());
    DeviceBuffer<unsigned char> active(samples.size());
    DeviceBuffer<double> jacobian(static_cast<std::size_t>(count) * dimension);
    DeviceBuffer<float> device_base(dense_entries);
    DeviceBuffer<float> hessian(dense_entries);
    DeviceBuffer<DeviceMixedWork> work(1U);
    check_cuda(cudaMemcpy(device_samples.get(), samples.data(),
                   samples.size() * sizeof(DeviceMixedSample), cudaMemcpyHostToDevice),
        "upload NCGA3 samples");
    check_cuda(cudaMemcpy(device_base.get(), base_hessian.data(),
                   base_hessian.size() * sizeof(float), cudaMemcpyHostToDevice),
        "upload NCGA3 base Hessian");
    check_cuda(cudaMemset(work.get(), 0, sizeof(DeviceMixedWork)),
        "reset NCGA3 work");
    compute_mixed_density<<<blocks_for(samples.size()), THREADS>>>(
        device_samples.get(), count, device_profile, density.get(), compression.get(),
        active.get(), work.get());
    check_cuda(cudaGetLastError(), "launch NCGA3 density");
    compute_mixed_jacobian<<<blocks_for(static_cast<std::size_t>(count) * dimension),
        THREADS>>>(device_samples.get(), active.get(), count, device_profile,
        jacobian.get(), work.get());
    check_cuda(cudaGetLastError(), "launch NCGA3 Jacobian");
    assemble_mixed_pressure<<<blocks_for(dense_entries), THREADS>>>(
        device_samples.get(), compression.get(), active.get(), jacobian.get(),
        device_base.get(), count, device_profile, hessian.get(), work.get());
    check_cuda(cudaGetLastError(), "launch NCGA3 pressure Hessian");
    check_cuda(cudaDeviceSynchronize(), "synchronize NCGA3 pressure Hessian");

    std::vector<float> host_hessian(dense_entries);
    std::vector<unsigned char> host_active(samples.size());
    DeviceMixedWork host_work{};
    check_cuda(cudaMemcpy(host_hessian.data(), hessian.get(),
                   host_hessian.size() * sizeof(float), cudaMemcpyDeviceToHost),
        "copy NCGA3 Hessian");
    check_cuda(cudaMemcpy(host_active.data(), active.get(),
                   host_active.size() * sizeof(unsigned char), cudaMemcpyDeviceToHost),
        "copy NCGA3 active flags");
    check_cuda(cudaMemcpy(&host_work, work.get(), sizeof(host_work),
                   cudaMemcpyDeviceToHost),
        "copy NCGA3 work");
    output.hessian.assign(host_hessian.begin(), host_hessian.end());
    output.pressure_active.assign(host_active.begin(), host_active.end());
    output.work.density_terms = host_work.density_terms;
    output.work.jacobian_terms = host_work.jacobian_terms;
    output.work.pressure_outer_products = host_work.outer_products;
    output.work.pressure_geometric_products = host_work.geometric_products;
    const std::uint64_t products = host_work.outer_products
        + host_work.geometric_products;
    if (arithmetic == PressureArithmetic::F32ProductsF64Reduction) {
        output.work.f32_completed_products = products;
    } else {
        output.work.f64_completed_products = products;
    }
    output.work.f64_accumulations = products;
    output.work.output_rounds_to_f32 = dense_entries;
    output.failure = AssemblyFailure::None;
    return output;
}

} // namespace nextengine::nonlocal::gpu_assembly_audit
