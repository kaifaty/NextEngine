#include "corrected_cuda_full_step.hpp"

#include <cub/cub.cuh>
#include <cuda_runtime.h>

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <limits>
#include <sstream>
#include <stdexcept>
#include <string>
#include <unordered_set>
#include <vector>

namespace nextengine::nonlocal::gpu_full_step {
namespace {

constexpr int kThreads = 128;
constexpr int kRowSortThreads = 64;
constexpr int kMaximumTotalSamples = 100000;
constexpr long long kCellBias = 1LL << 20;
constexpr long long kCellMinimum = -kCellBias;
constexpr long long kCellMaximum = kCellBias - 1LL;
constexpr double kMicrometresPerMetre = 1000000.0;

struct DeviceVec3 {
    float x;
    float y;
    float z;
};

struct DeviceGraphWork {
    unsigned long long cell_probes;
    unsigned long long distance_predicates;
    unsigned long long emitted_pairs;
};

struct DeviceEvaluationWork {
    unsigned long long density_kernel_evaluations;
    unsigned long long energy_pair_visits;
    unsigned long long gradient_pair_visits;
    unsigned long long hvp_pair_visits;
    unsigned long long active_pressure_centers;
};

struct DeviceProfile {
    float dt;
    float spacing;
    float horizon;
    float mass;
    float rest_density;
    float kappa;
    float lambda;
    float mu;
    float gamma;
};

struct KernelValues {
    float value;
    float first;
    float second;
};

__device__ DeviceVec3 add(DeviceVec3 lhs, DeviceVec3 rhs) {
    return {lhs.x + rhs.x, lhs.y + rhs.y, lhs.z + rhs.z};
}

__device__ DeviceVec3 subtract(DeviceVec3 lhs, DeviceVec3 rhs) {
    return {lhs.x - rhs.x, lhs.y - rhs.y, lhs.z - rhs.z};
}

__device__ DeviceVec3 scale(DeviceVec3 value, float factor) {
    return {value.x * factor, value.y * factor, value.z * factor};
}

__device__ float dot(DeviceVec3 lhs, DeviceVec3 rhs) {
    return lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z;
}

__device__ float norm(DeviceVec3 value) {
    return sqrtf(dot(value, value));
}

__device__ DeviceVec3 radial_apply(
    DeviceVec3 normal, float radial, float tangential, DeviceVec3 value) {
    const float projected = dot(normal, value);
    return add(scale(value, tangential),
        scale(normal, (radial - tangential) * projected));
}

__device__ KernelValues kernel_values(
    float radius, const DeviceProfile& profile, bool missing_chain) {
    constexpr float pi = 3.14159265358979323846F;
    const float q = 2.0F * radius / profile.horizon;
    const float alpha = 3.0F
        / (2.0F * pi * profile.horizon * profile.horizon * profile.horizon);
    float value = 0.0F;
    float first_q = 0.0F;
    float second_q = 0.0F;
    if (q < 1.0F) {
        value = alpha * (2.0F / 3.0F - q * q + 0.5F * q * q * q);
        first_q = alpha * (-2.0F * q + 1.5F * q * q);
        second_q = alpha * (-2.0F + 3.0F * q);
    } else if (q <= 2.0F) {
        const float tail = 2.0F - q;
        value = alpha * tail * tail * tail / 6.0F;
        first_q = -0.5F * alpha * tail * tail;
        second_q = alpha * tail;
    }
    const float chain = 2.0F / profile.horizon;
    return {value,
        first_q * (missing_chain ? 1.0F : chain),
        second_q * (missing_chain ? chain : chain * chain)};
}

__device__ double kernel_value_double(double radius, double horizon) {
    constexpr double pi = 3.141592653589793238462643383279502884;
    const double q = 2.0 * radius / horizon;
    const double alpha = 3.0 / (2.0 * pi * horizon * horizon * horizon);
    if (q < 1.0) return alpha * (2.0 / 3.0 - q * q + 0.5 * q * q * q);
    if (q <= 2.0) {
        const double tail = 2.0 - q;
        return alpha * tail * tail * tail / 6.0;
    }
    return 0.0;
}

__device__ double kernel_first_double(double radius, double horizon) {
    constexpr double pi = 3.141592653589793238462643383279502884;
    const double q = 2.0 * radius / horizon;
    const double alpha = 3.0 / (2.0 * pi * horizon * horizon * horizon);
    double first_q = 0.0;
    if (q < 1.0) {
        first_q = alpha * (-2.0 * q + 1.5 * q * q);
    } else if (q <= 2.0) {
        const double tail = 2.0 - q;
        first_q = -0.5 * alpha * tail * tail;
    }
    return first_q * 2.0 / horizon;
}

__device__ void surface_values(
    float radius, float spacing, float& potential, float& force, float& derivative) {
    const float q = radius / spacing;
    if (q <= 1.0F) {
        force = q * q - 1.0F;
        derivative = 2.0F * q / spacing;
        potential = spacing * (q * q * q / 3.0F - q - 2.0F / 3.0F);
    } else if (q < 3.0F) {
        const float shifted = q - 2.0F;
        force = 1.0F - shifted * shifted;
        derivative = -2.0F * shifted / spacing;
        potential = spacing
            * (q - shifted * shifted * shifted / 3.0F - 8.0F / 3.0F);
    } else {
        potential = 0.0F;
        force = 0.0F;
        derivative = 0.0F;
    }
}

__device__ double surface_potential_double(double radius, double spacing) {
    const double q = radius / spacing;
    if (q <= 1.0) return spacing * (q * q * q / 3.0 - q - 2.0 / 3.0);
    if (q < 3.0) {
        const double shifted = q - 2.0;
        return spacing * (q - shifted * shifted * shifted / 3.0 - 8.0 / 3.0);
    }
    return 0.0;
}

void cuda_check(cudaError_t status, const char* operation) {
    if (status != cudaSuccess) {
        throw std::runtime_error(std::string(operation) + ": "
            + cudaGetErrorString(status));
    }
}

int blocks_for(int count) {
    return (count + kThreads - 1) / kThreads;
}

template <typename T>
void allocate_device(T** pointer, std::size_t count, std::size_t& bytes) {
    cuda_check(cudaMalloc(reinterpret_cast<void**>(pointer), count * sizeof(T)),
        "cudaMalloc NCGP1");
    bytes += count * sizeof(T);
}

__device__ long long quantize_axis(float value, int* error) {
    if (!isfinite(value) || fabsf(value) > 1000000.0F) {
        atomicExch(error, static_cast<int>(NonlocalGpuFailure::Nonfinite));
        return 0LL;
    }
    return llrintf(value * static_cast<float>(kMicrometresPerMetre));
}

__device__ long long floor_division(long long numerator, long long denominator) {
    const long long quotient = numerator / denominator;
    const long long remainder = numerator % denominator;
    return quotient - (remainder < 0LL ? 1LL : 0LL);
}

__device__ unsigned long long pack_cell(
    long long x, long long y, long long z, int* error) {
    if (x < kCellMinimum || x > kCellMaximum || y < kCellMinimum
        || y > kCellMaximum || z < kCellMinimum || z > kCellMaximum) {
        atomicExch(error, static_cast<int>(NonlocalGpuFailure::CellRangeExceeded));
        return 0ULL;
    }
    return (static_cast<unsigned long long>(x + kCellBias) << 42U)
        | (static_cast<unsigned long long>(y + kCellBias) << 21U)
        | static_cast<unsigned long long>(z + kCellBias);
}

__device__ unsigned long long square(long long value) {
    const auto magnitude = value < 0LL
        ? static_cast<unsigned long long>(-value)
        : static_cast<unsigned long long>(value);
    return magnitude * magnitude;
}

__device__ int lower_bound_key(const unsigned long long* values,
    int count,
    unsigned long long key) {
    int first = 0;
    int length = count;
    while (length > 0) {
        const int half = length / 2;
        const int middle = first + half;
        if (values[middle] < key) {
            first = middle + 1;
            length -= half + 1;
        } else {
            length = half;
        }
    }
    return first;
}

__device__ int upper_bound_key(const unsigned long long* values,
    int count,
    unsigned long long key) {
    int first = 0;
    int length = count;
    while (length > 0) {
        const int half = length / 2;
        const int middle = first + half;
        if (key < values[middle]) {
            length = half;
        } else {
            first = middle + 1;
            length -= half + 1;
        }
    }
    return first;
}

__global__ void initialize_indices(unsigned int* indices, int count) {
    const int index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index < count) indices[index] = static_cast<unsigned int>(index);
}

__global__ void validate_sorted_ids(const unsigned int* ids, int count, int* error) {
    const int index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index < count && index > 0 && ids[index - 1] >= ids[index]) {
        atomicExch(error, static_cast<int>(NonlocalGpuFailure::DuplicateSampleId));
    }
}

__global__ void gather_canonical_state(const unsigned int* sorted_ids,
    const unsigned int* sorted_indices,
    const DeviceVec3* reference_input,
    const DeviceVec3* current_input,
    const DeviceVec3* velocity_input,
    unsigned int* ids,
    DeviceVec3* reference,
    DeviceVec3* current,
    DeviceVec3* predicted,
    DeviceVec3* velocity,
    int count,
    DeviceVec3 gravity,
    float dt) {
    const int row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= count) return;
    const unsigned int input = sorted_indices[row];
    ids[row] = sorted_ids[row];
    reference[row] = reference_input[input];
    current[row] = current_input[input];
    velocity[row] = velocity_input[input];
    const DeviceVec3 x = reference_input[input];
    const DeviceVec3 v = velocity_input[input];
    predicted[row] = {x.x + dt * v.x + dt * dt * gravity.x,
        x.y + dt * v.y + dt * dt * gravity.y,
        x.z + dt * v.z + dt * dt * gravity.z};
}

__global__ void append_ghosts(const unsigned int* ghost_ids_input,
    const DeviceVec3* ghost_positions_input,
    unsigned int* ids,
    DeviceVec3* positions,
    int dynamic_count,
    int ghost_count) {
    const int index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= ghost_count) return;
    ids[dynamic_count + index] = ghost_ids_input[index];
    positions[dynamic_count + index] = ghost_positions_input[index];
}

__global__ void copy_dynamic_positions(
    const DeviceVec3* current, DeviceVec3* all_positions, int count) {
    const int index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index < count) all_positions[index] = current[index];
}

__global__ void compute_cell_keys(const DeviceVec3* positions,
    unsigned long long* keys,
    unsigned int* indices,
    int count,
    long long cell_size_um,
    int* error) {
    const int index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= count) return;
    const DeviceVec3 p = positions[index];
    const long long x = quantize_axis(p.x, error);
    const long long y = quantize_axis(p.y, error);
    const long long z = quantize_axis(p.z, error);
    keys[index] = pack_cell(floor_division(x, cell_size_um),
        floor_division(y, cell_size_um), floor_division(z, cell_size_um), error);
    indices[index] = static_cast<unsigned int>(index);
}

template <bool Fill>
__global__ void visit_neighbors(const DeviceVec3* positions,
    const unsigned int* ids,
    const unsigned long long* sorted_keys,
    const unsigned int* sorted_indices,
    unsigned int* counts,
    const unsigned int* offsets,
    unsigned int* neighbors,
    int dynamic_count,
    int total_count,
    long long support_um,
    bool strict_radius,
    DeviceGraphWork* work,
    int* error) {
    const int row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= dynamic_count) return;
    const DeviceVec3 owner_position = positions[row];
    const long long owner_x_um = quantize_axis(owner_position.x, error);
    const long long owner_y_um = quantize_axis(owner_position.y, error);
    const long long owner_z_um = quantize_axis(owner_position.z, error);
    const long long cx = floor_division(owner_x_um, support_um);
    const long long cy = floor_division(owner_y_um, support_um);
    const long long cz = floor_division(owner_z_um, support_um);
    const unsigned long long limit = static_cast<unsigned long long>(support_um)
        * static_cast<unsigned long long>(support_um);
    unsigned int degree = 0U;
    unsigned long long local_probes = 0ULL;
    unsigned long long local_predicates = 0ULL;
    for (int dz = -1; dz <= 1; ++dz) {
        for (int dy = -1; dy <= 1; ++dy) {
            for (int dx = -1; dx <= 1; ++dx) {
                ++local_probes;
                const unsigned long long key = pack_cell(
                    cx + dx, cy + dy, cz + dz, error);
                const int begin = lower_bound_key(sorted_keys, total_count, key);
                const int end = upper_bound_key(sorted_keys, total_count, key);
                for (int slot = begin; slot < end; ++slot) {
                    ++local_predicates;
                    const unsigned int candidate = sorted_indices[slot];
                    const DeviceVec3 q = positions[candidate];
                    const long long qx = quantize_axis(q.x, error);
                    const long long qy = quantize_axis(q.y, error);
                    const long long qz = quantize_axis(q.z, error);
                    const unsigned long long distance = square(owner_x_um - qx)
                        + square(owner_y_um - qy) + square(owner_z_um - qz);
                    const bool member = strict_radius ? distance < limit : distance <= limit;
                    if (!member) continue;
                    if (degree >= kMaximumNeighbors) {
                        atomicExch(error,
                            static_cast<int>(NonlocalGpuFailure::CapacityExceeded));
                        continue;
                    }
                    if constexpr (Fill) {
                        neighbors[offsets[row] + degree] = candidate;
                    }
                    ++degree;
                }
            }
        }
    }
    if constexpr (!Fill) {
        counts[row] = degree;
    } else {
        atomicAdd(&work->emitted_pairs,
            static_cast<unsigned long long>(degree));
    }
    atomicAdd(&work->cell_probes, local_probes);
    atomicAdd(&work->distance_predicates, local_predicates);
    (void)ids;
}

__global__ void finish_offsets(
    const unsigned int* counts, unsigned int* offsets, int count) {
    if (blockIdx.x == 0 && threadIdx.x == 0) {
        offsets[count] = offsets[count - 1] + counts[count - 1];
    }
}

__global__ void sort_neighbor_rows(const unsigned int* offsets,
    unsigned int* neighbors,
    int count,
    int* error) {
    const int row = blockIdx.x;
    if (row >= count) return;
    const unsigned int begin = offsets[row];
    const unsigned int degree = offsets[row + 1] - begin;
    if (degree > kMaximumNeighbors) {
        if (threadIdx.x == 0) {
            atomicExch(error,
                static_cast<int>(NonlocalGpuFailure::CapacityExceeded));
        }
        return;
    }
    extern __shared__ unsigned int values[];
    for (unsigned int index = threadIdx.x; index < kMaximumNeighbors;
         index += blockDim.x) {
        values[index] = index < degree ? neighbors[begin + index] : 0xffffffffU;
    }
    __syncthreads();
    for (unsigned int width = 2U; width <= kMaximumNeighbors; width <<= 1U) {
        for (unsigned int stride = width >> 1U; stride > 0U; stride >>= 1U) {
            for (unsigned int index = threadIdx.x; index < kMaximumNeighbors;
                 index += blockDim.x) {
                const unsigned int other = index ^ stride;
                if (other > index) {
                    const bool ascending = (index & width) == 0U;
                    const unsigned int lhs = values[index];
                    const unsigned int rhs = values[other];
                    if ((ascending && lhs > rhs) || (!ascending && lhs < rhs)) {
                        values[index] = rhs;
                        values[other] = lhs;
                    }
                }
            }
            __syncthreads();
        }
    }
    for (unsigned int index = threadIdx.x; index < degree; index += blockDim.x) {
        neighbors[begin + index] = values[index];
    }
}

__global__ void maximum_degree_kernel(const unsigned int* offsets,
    int count,
    unsigned int* maximum_degree) {
    const int row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row < count) {
        atomicMax(maximum_degree, offsets[row + 1] - offsets[row]);
    }
}

__global__ void density_kernel(const DeviceVec3* current,
    const DeviceVec3* all_positions,
    const unsigned int* offsets,
    const unsigned int* neighbors,
    float* density,
    double* energy_density,
    float* pressure_excess,
    int dynamic_count,
    DeviceProfile profile,
    DeviceEvaluationWork* work,
    int* error) {
    const int row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= dynamic_count) return;
    const DeviceVec3 owner = current[row];
    float sum = 0.0F;
    double sum_double = 0.0;
    unsigned long long evaluations = 0ULL;
    for (unsigned int slot = offsets[row]; slot < offsets[row + 1]; ++slot) {
        const DeviceVec3 candidate = all_positions[neighbors[slot]];
        const DeviceVec3 difference = subtract(owner, candidate);
        const float radius = norm(difference);
        const KernelValues kernel = kernel_values(radius, profile, false);
        sum += profile.mass * kernel.value;
        const double dx = static_cast<double>(owner.x)
            - static_cast<double>(candidate.x);
        const double dy = static_cast<double>(owner.y)
            - static_cast<double>(candidate.y);
        const double dz = static_cast<double>(owner.z)
            - static_cast<double>(candidate.z);
        const double radius_double = sqrt(dx * dx + dy * dy + dz * dz);
        sum_double += static_cast<double>(profile.mass)
            * kernel_value_double(radius_double, static_cast<double>(profile.horizon));
        ++evaluations;
    }
    density[row] = sum;
    energy_density[row] = sum_double;
    pressure_excess[row] = fmaxf(sum / profile.rest_density - 1.0F, 0.0F);
    if (!isfinite(sum) || !isfinite(sum_double)) {
        atomicExch(error, static_cast<int>(NonlocalGpuFailure::Nonfinite));
    }
    if (pressure_excess[row] > 0.0F) {
        atomicAdd(&work->active_pressure_centers, 1ULL);
    }
    atomicAdd(&work->density_kernel_evaluations, evaluations);
}

__global__ void energy_gradient_kernel(const DeviceVec3* reference,
    const DeviceVec3* current,
    const DeviceVec3* predicted,
    const DeviceVec3* all_positions,
    const unsigned int* current_offsets,
    const unsigned int* current_neighbors,
    const unsigned int* reference_offsets,
    const unsigned int* reference_neighbors,
    const float* pressure_excess,
    const double* energy_density,
    DeviceVec3* gradient,
    double* energy_rows,
    int dynamic_count,
    DeviceProfile profile,
    unsigned int variant,
    DeviceEvaluationWork* work,
    int* error) {
    const int row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= dynamic_count) return;
    const bool missing_chain = variant
        == static_cast<unsigned int>(NonlocalGpuVariant::MissingKernelChain);
    const bool half_viscosity = variant
        == static_cast<unsigned int>(NonlocalGpuVariant::HalfViscosity);
    const bool wrong_surface = variant
        == static_cast<unsigned int>(NonlocalGpuVariant::WrongSurfaceSign);
    const bool owner_pressure_only = variant
        == static_cast<unsigned int>(NonlocalGpuVariant::OwnerOnlyPressure);
    const bool current_reference_swap = variant
        == static_cast<unsigned int>(NonlocalGpuVariant::CurrentReferenceSwap);
    const DeviceVec3 y = current[row];
    const DeviceVec3 x = reference[row];
    const DeviceVec3 inertial_delta = subtract(y, predicted[row]);
    const float inertia_scale = profile.mass / (profile.dt * profile.dt);
    DeviceVec3 result = scale(inertial_delta, inertia_scale);
    const double inertial_x = static_cast<double>(y.x)
        - static_cast<double>(predicted[row].x);
    const double inertial_y = static_cast<double>(y.y)
        - static_cast<double>(predicted[row].y);
    const double inertial_z = static_cast<double>(y.z)
        - static_cast<double>(predicted[row].z);
    const double inertia_scale_double = static_cast<double>(profile.mass)
        / (static_cast<double>(profile.dt) * profile.dt);
    double energy = 0.5 * inertia_scale_double
        * (inertial_x * inertial_x + inertial_y * inertial_y
            + inertial_z * inertial_z);
    const double excess_double = fmax(
        energy_density[row] / static_cast<double>(profile.rest_density) - 1.0, 0.0);
    energy += 0.5 * static_cast<double>(profile.kappa)
        * excess_double * excess_double;
    unsigned long long energy_visits = 0ULL;
    unsigned long long gradient_visits = 0ULL;

    const float own_excess = pressure_excess[row];
    for (unsigned int slot = current_offsets[row];
         slot < current_offsets[row + 1]; ++slot) {
        const unsigned int neighbor = current_neighbors[slot];
        if (neighbor == static_cast<unsigned int>(row)) continue;
        const DeviceVec3 difference = subtract(y, all_positions[neighbor]);
        const float radius = norm(difference);
        if (!(radius > 0.0F)) continue;
        const DeviceVec3 normal = scale(difference, 1.0F / radius);
        const KernelValues kernel = kernel_values(radius, profile, missing_chain);
        const float neighbor_excess = neighbor < static_cast<unsigned int>(dynamic_count)
                && !owner_pressure_only
            ? pressure_excess[neighbor]
            : 0.0F;
        const float pressure_factor = profile.kappa * profile.mass
            / profile.rest_density * (own_excess + neighbor_excess) * kernel.first;
        result = add(result, scale(normal, pressure_factor));
        ++gradient_visits;

        if (neighbor < static_cast<unsigned int>(dynamic_count)) {
            float potential = 0.0F;
            float surface_force = 0.0F;
            float derivative = 0.0F;
            surface_values(radius, profile.spacing, potential, surface_force, derivative);
            const float sign = wrong_surface ? -1.0F : 1.0F;
            result = add(result, scale(normal,
                sign * 2.0F * profile.gamma * profile.mass * profile.mass
                    * surface_force));
            ++gradient_visits;
            if (neighbor > static_cast<unsigned int>(row)) {
                const double dx = static_cast<double>(y.x)
                    - static_cast<double>(all_positions[neighbor].x);
                const double dy = static_cast<double>(y.y)
                    - static_cast<double>(all_positions[neighbor].y);
                const double dz = static_cast<double>(y.z)
                    - static_cast<double>(all_positions[neighbor].z);
                const double radius_double = sqrt(dx * dx + dy * dy + dz * dz);
                energy += 2.0 * static_cast<double>(profile.gamma)
                    * static_cast<double>(profile.mass) * profile.mass
                    * surface_potential_double(radius_double,
                        static_cast<double>(profile.spacing));
                ++energy_visits;
            }
        }
    }

    const unsigned int* viscosity_offsets = current_reference_swap
        ? current_offsets : reference_offsets;
    const unsigned int* viscosity_neighbors = current_reference_swap
        ? current_neighbors : reference_neighbors;
    for (unsigned int slot = viscosity_offsets[row];
         slot < viscosity_offsets[row + 1]; ++slot) {
        const unsigned int neighbor = viscosity_neighbors[slot];
        if (neighbor == static_cast<unsigned int>(row)
            || neighbor >= static_cast<unsigned int>(dynamic_count)) continue;
        const DeviceVec3 reference_delta = subtract(x, reference[neighbor]);
        const float radius = norm(reference_delta);
        if (!(radius > 0.0F)) continue;
        const DeviceVec3 normal = scale(reference_delta, 1.0F / radius);
        const DeviceVec3 delta = subtract(subtract(y, current[neighbor]),
            reference_delta);
        const float normal_delta = dot(normal, delta);
        const DeviceVec3 tangent_delta = subtract(delta, scale(normal, normal_delta));
        const KernelValues kernel = kernel_values(radius, profile, missing_chain);
        const float factor = profile.mass * (-kernel.first)
            / (profile.rest_density * profile.dt);
        DeviceVec3 viscous = add(scale(tangent_delta, 2.0F * profile.mu),
            scale(normal, profile.lambda * normal_delta));
        if (half_viscosity) viscous = scale(viscous, 0.5F);
        result = add(result, scale(viscous, factor));
        ++gradient_visits;
        if (neighbor > static_cast<unsigned int>(row)) {
            const double rx = static_cast<double>(x.x)
                - static_cast<double>(reference[neighbor].x);
            const double ry = static_cast<double>(x.y)
                - static_cast<double>(reference[neighbor].y);
            const double rz = static_cast<double>(x.z)
                - static_cast<double>(reference[neighbor].z);
            const double reference_radius = sqrt(rx * rx + ry * ry + rz * rz);
            const double nx = rx / reference_radius;
            const double ny = ry / reference_radius;
            const double nz = rz / reference_radius;
            const double dx = (static_cast<double>(y.x)
                    - static_cast<double>(current[neighbor].x)) - rx;
            const double dy = (static_cast<double>(y.y)
                    - static_cast<double>(current[neighbor].y)) - ry;
            const double dz = (static_cast<double>(y.z)
                    - static_cast<double>(current[neighbor].z)) - rz;
            const double normal_component = nx * dx + ny * dy + nz * dz;
            const double tx = dx - nx * normal_component;
            const double ty = dy - ny * normal_component;
            const double tz = dz - nz * normal_component;
            const double energy_factor = static_cast<double>(profile.mass)
                * (-kernel_first_double(reference_radius,
                    static_cast<double>(profile.horizon)))
                / (static_cast<double>(profile.rest_density) * profile.dt);
            energy += energy_factor
                * (static_cast<double>(profile.mu) * (tx * tx + ty * ty + tz * tz)
                    + 0.5 * static_cast<double>(profile.lambda)
                        * normal_component * normal_component);
            ++energy_visits;
        }
    }
    gradient[row] = result;
    energy_rows[row] = energy;
    if (!isfinite(result.x) || !isfinite(result.y) || !isfinite(result.z)
        || !isfinite(energy)) {
        atomicExch(error, static_cast<int>(NonlocalGpuFailure::Nonfinite));
    }
    atomicAdd(&work->energy_pair_visits, energy_visits);
    atomicAdd(&work->gradient_pair_visits, gradient_visits);
}

__global__ void pressure_directional_kernel(const DeviceVec3* current,
    const DeviceVec3* all_positions,
    const DeviceVec3* direction,
    const unsigned int* offsets,
    const unsigned int* neighbors,
    float* pressure_q,
    int dynamic_count,
    DeviceProfile profile,
    unsigned int variant,
    int* error) {
    const int row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= dynamic_count) return;
    const bool missing_chain = variant
        == static_cast<unsigned int>(NonlocalGpuVariant::MissingKernelChain);
    float value = 0.0F;
    const DeviceVec3 owner = current[row];
    for (unsigned int slot = offsets[row]; slot < offsets[row + 1]; ++slot) {
        const unsigned int neighbor = neighbors[slot];
        if (neighbor == static_cast<unsigned int>(row)) continue;
        const DeviceVec3 difference = subtract(owner, all_positions[neighbor]);
        const float radius = norm(difference);
        if (!(radius > 0.0F)) continue;
        const DeviceVec3 normal = scale(difference, 1.0F / radius);
        const DeviceVec3 neighbor_direction = neighbor
                < static_cast<unsigned int>(dynamic_count)
            ? direction[neighbor] : DeviceVec3{0.0F, 0.0F, 0.0F};
        const KernelValues kernel = kernel_values(radius, profile, missing_chain);
        value += profile.mass / profile.rest_density * kernel.first
            * dot(normal, subtract(direction[row], neighbor_direction));
    }
    pressure_q[row] = value;
    if (!isfinite(value)) {
        atomicExch(error, static_cast<int>(NonlocalGpuFailure::Nonfinite));
    }
}

__global__ void hvp_kernel(const DeviceVec3* reference,
    const DeviceVec3* current,
    const DeviceVec3* all_positions,
    const DeviceVec3* direction,
    const unsigned int* current_offsets,
    const unsigned int* current_neighbors,
    const unsigned int* reference_offsets,
    const unsigned int* reference_neighbors,
    const float* pressure_excess,
    const float* pressure_q,
    DeviceVec3* hvp,
    DeviceVec3* diagonal,
    int dynamic_count,
    DeviceProfile profile,
    unsigned int variant,
    DeviceEvaluationWork* work,
    int* error) {
    const int row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= dynamic_count) return;
    const bool missing_chain = variant
        == static_cast<unsigned int>(NonlocalGpuVariant::MissingKernelChain);
    const bool half_viscosity = variant
        == static_cast<unsigned int>(NonlocalGpuVariant::HalfViscosity);
    const bool wrong_surface = variant
        == static_cast<unsigned int>(NonlocalGpuVariant::WrongSurfaceSign);
    const bool owner_pressure_only = variant
        == static_cast<unsigned int>(NonlocalGpuVariant::OwnerOnlyPressure);
    const bool current_reference_swap = variant
        == static_cast<unsigned int>(NonlocalGpuVariant::CurrentReferenceSwap);
    const DeviceVec3 y = current[row];
    const DeviceVec3 x = reference[row];
    const DeviceVec3 v = direction[row];
    const float inertia_scale = profile.mass / (profile.dt * profile.dt);
    DeviceVec3 result = scale(v, inertia_scale);
    DeviceVec3 diag{inertia_scale, inertia_scale, inertia_scale};
    DeviceVec3 own_density_gradient{0.0F, 0.0F, 0.0F};
    unsigned long long visits = 0ULL;

    for (unsigned int slot = current_offsets[row];
         slot < current_offsets[row + 1]; ++slot) {
        const unsigned int neighbor = current_neighbors[slot];
        if (neighbor == static_cast<unsigned int>(row)) continue;
        const DeviceVec3 difference = subtract(y, all_positions[neighbor]);
        const float radius = norm(difference);
        if (!(radius > 0.0F)) continue;
        const DeviceVec3 normal = scale(difference, 1.0F / radius);
        const DeviceVec3 neighbor_direction = neighbor
                < static_cast<unsigned int>(dynamic_count)
            ? direction[neighbor] : DeviceVec3{0.0F, 0.0F, 0.0F};
        const DeviceVec3 dv = subtract(v, neighbor_direction);
        const KernelValues kernel = kernel_values(radius, profile, missing_chain);
        const float density_factor = profile.mass / profile.rest_density;
        const DeviceVec3 b = scale(normal, density_factor * kernel.first);
        own_density_gradient = add(own_density_gradient, b);
        const float neighbor_q = neighbor < static_cast<unsigned int>(dynamic_count)
                && !owner_pressure_only
            ? pressure_q[neighbor] : 0.0F;
        result = add(result, scale(b,
            profile.kappa * (pressure_q[row] + neighbor_q)));
        const float neighbor_excess = neighbor
                    < static_cast<unsigned int>(dynamic_count)
                && !owner_pressure_only
            ? pressure_excess[neighbor] : 0.0F;
        const float excess_sum = pressure_excess[row] + neighbor_excess;
        const float tangential = kernel.first / radius;
        result = add(result, scale(radial_apply(normal, kernel.second,
            tangential, dv), profile.kappa * density_factor * excess_sum));
        const float geometric_scale = profile.kappa * density_factor * excess_sum;
        diag.x += geometric_scale
            * (tangential + (kernel.second - tangential) * normal.x * normal.x);
        diag.y += geometric_scale
            * (tangential + (kernel.second - tangential) * normal.y * normal.y);
        diag.z += geometric_scale
            * (tangential + (kernel.second - tangential) * normal.z * normal.z);
        if (neighbor < static_cast<unsigned int>(dynamic_count)) {
            const DeviceVec3 neighbor_b = b;
            diag.x += profile.kappa * neighbor_b.x * neighbor_b.x;
            diag.y += profile.kappa * neighbor_b.y * neighbor_b.y;
            diag.z += profile.kappa * neighbor_b.z * neighbor_b.z;
            float potential = 0.0F;
            float surface_force = 0.0F;
            float surface_derivative = 0.0F;
            surface_values(radius, profile.spacing, potential, surface_force,
                surface_derivative);
            const float sign = wrong_surface ? -1.0F : 1.0F;
            const float surface_scale = sign * 2.0F * profile.gamma
                * profile.mass * profile.mass;
            result = add(result, scale(radial_apply(normal, surface_derivative,
                surface_force / radius, dv), surface_scale));
            diag.x += surface_scale * (surface_force / radius
                + (surface_derivative - surface_force / radius)
                    * normal.x * normal.x);
            diag.y += surface_scale * (surface_force / radius
                + (surface_derivative - surface_force / radius)
                    * normal.y * normal.y);
            diag.z += surface_scale * (surface_force / radius
                + (surface_derivative - surface_force / radius)
                    * normal.z * normal.z);
        }
        ++visits;
    }
    diag.x += profile.kappa * own_density_gradient.x * own_density_gradient.x;
    diag.y += profile.kappa * own_density_gradient.y * own_density_gradient.y;
    diag.z += profile.kappa * own_density_gradient.z * own_density_gradient.z;

    const unsigned int* viscosity_offsets = current_reference_swap
        ? current_offsets : reference_offsets;
    const unsigned int* viscosity_neighbors = current_reference_swap
        ? current_neighbors : reference_neighbors;
    for (unsigned int slot = viscosity_offsets[row];
         slot < viscosity_offsets[row + 1]; ++slot) {
        const unsigned int neighbor = viscosity_neighbors[slot];
        if (neighbor == static_cast<unsigned int>(row)
            || neighbor >= static_cast<unsigned int>(dynamic_count)) continue;
        const DeviceVec3 reference_delta = subtract(x, reference[neighbor]);
        const float radius = norm(reference_delta);
        if (!(radius > 0.0F)) continue;
        const DeviceVec3 normal = scale(reference_delta, 1.0F / radius);
        const KernelValues kernel = kernel_values(radius, profile, missing_chain);
        float factor = profile.mass * (-kernel.first)
            / (profile.rest_density * profile.dt);
        if (half_viscosity) factor *= 0.5F;
        const float tangential = factor * 2.0F * profile.mu;
        const float radial = factor * profile.lambda;
        const DeviceVec3 dv = subtract(v, direction[neighbor]);
        result = add(result, radial_apply(normal, radial, tangential, dv));
        diag.x += tangential + (radial - tangential) * normal.x * normal.x;
        diag.y += tangential + (radial - tangential) * normal.y * normal.y;
        diag.z += tangential + (radial - tangential) * normal.z * normal.z;
        ++visits;
    }
    if (variant == static_cast<unsigned int>(NonlocalGpuVariant::HvpSignFlip)) {
        result = scale(result, -1.0F);
    }
    hvp[row] = result;
    diagonal[row] = diag;
    if (!isfinite(result.x) || !isfinite(result.y) || !isfinite(result.z)) {
        atomicExch(error, static_cast<int>(NonlocalGpuFailure::Nonfinite));
    }
    atomicAdd(&work->hvp_pair_visits, visits);
}

__global__ void vector_norm_rows(
    const DeviceVec3* values, double* rows, int count) {
    const int index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= count) return;
    const DeviceVec3 value = values[index];
    rows[index] = static_cast<double>(value.x) * value.x
        + static_cast<double>(value.y) * value.y
        + static_cast<double>(value.z) * value.z;
}

__global__ void reduce_double_blocks(
    const double* values, double* blocks, int count) {
    __shared__ double shared[256];
    const int global = blockIdx.x * blockDim.x + threadIdx.x;
    shared[threadIdx.x] = global < count ? values[global] : 0.0;
    __syncthreads();
    for (unsigned int stride = blockDim.x / 2U; stride > 0U; stride >>= 1U) {
        if (threadIdx.x < stride) shared[threadIdx.x] += shared[threadIdx.x + stride];
        __syncthreads();
    }
    if (threadIdx.x == 0) blocks[blockIdx.x] = shared[0];
}

__global__ void reduce_double_final(
    const double* blocks, double* result, int count) {
    __shared__ double shared[256];
    double value = 0.0;
    for (int index = threadIdx.x; index < count; index += blockDim.x) {
        value += blocks[index];
    }
    shared[threadIdx.x] = value;
    __syncthreads();
    for (unsigned int stride = blockDim.x / 2U; stride > 0U; stride >>= 1U) {
        if (threadIdx.x < stride) shared[threadIdx.x] += shared[threadIdx.x + stride];
        __syncthreads();
    }
    if (threadIdx.x == 0) *result = shared[0];
}

bool valid_profile(const NonlocalGpuProfile& profile) {
    return profile.id == "nonlocal-water-50k-v1"
        && profile.dt > 0.0 && profile.spacing > 0.0
        && profile.horizon > 0.0 && profile.mass > 0.0
        && profile.rest_density > 0.0 && profile.maximum_dynamic_samples > 0U
        && profile.maximum_dynamic_samples <= kMaximumDynamicSamples
        && profile.maximum_neighbors == kMaximumNeighbors;
}

} // namespace

struct NonlocalGpuWorkspace::Impl {
    explicit Impl(const NonlocalGpuProfile& selected) : profile(selected) {
        if (!valid_profile(profile)) {
            throw std::invalid_argument("invalid NCGP1 profile");
        }
        const std::size_t dynamic_capacity = profile.maximum_dynamic_samples;
        const std::size_t total_capacity = kMaximumTotalSamples;
        const std::size_t pair_capacity = dynamic_capacity * profile.maximum_neighbors;

        allocate_device(&ids_input, dynamic_capacity, allocated_bytes);
        allocate_device(&ids_sorted, dynamic_capacity, allocated_bytes);
        allocate_device(&input_indices, dynamic_capacity, allocated_bytes);
        allocate_device(&sorted_indices, dynamic_capacity, allocated_bytes);
        allocate_device(&reference_input, dynamic_capacity, allocated_bytes);
        allocate_device(&current_input, dynamic_capacity, allocated_bytes);
        allocate_device(&velocity_input, dynamic_capacity, allocated_bytes);
        allocate_device(&ids, total_capacity, allocated_bytes);
        allocate_device(&reference, dynamic_capacity, allocated_bytes);
        allocate_device(&current, dynamic_capacity, allocated_bytes);
        allocate_device(&predicted, dynamic_capacity, allocated_bytes);
        allocate_device(&velocity, dynamic_capacity, allocated_bytes);
        allocate_device(&all_positions, total_capacity, allocated_bytes);
        allocate_device(&ghost_ids_input, total_capacity - dynamic_capacity,
            allocated_bytes);
        allocate_device(&ghost_positions_input, total_capacity - dynamic_capacity,
            allocated_bytes);
        allocate_device(&cell_keys_input, total_capacity, allocated_bytes);
        allocate_device(&cell_keys_sorted, total_capacity, allocated_bytes);
        allocate_device(&cell_indices_input, total_capacity, allocated_bytes);
        allocate_device(&cell_indices_sorted, total_capacity, allocated_bytes);
        allocate_device(&counts, dynamic_capacity, allocated_bytes);
        allocate_device(&offsets, dynamic_capacity + 1U, allocated_bytes);
        allocate_device(&neighbors, pair_capacity, allocated_bytes);
        allocate_device(&reference_offsets, dynamic_capacity + 1U, allocated_bytes);
        allocate_device(&reference_neighbors, pair_capacity, allocated_bytes);
        allocate_device(&density, dynamic_capacity, allocated_bytes);
        allocate_device(&energy_density, dynamic_capacity, allocated_bytes);
        allocate_device(&pressure_excess, dynamic_capacity, allocated_bytes);
        allocate_device(&pressure_q, dynamic_capacity, allocated_bytes);
        allocate_device(&gradient, dynamic_capacity, allocated_bytes);
        allocate_device(&hvp, dynamic_capacity, allocated_bytes);
        allocate_device(&diagonal, dynamic_capacity, allocated_bytes);
        allocate_device(&direction, dynamic_capacity, allocated_bytes);
        allocate_device(&energy_rows, dynamic_capacity, allocated_bytes);
        allocate_device(&norm_rows, dynamic_capacity, allocated_bytes);
        const std::size_t reduction_capacity =
            (dynamic_capacity + 255U) / 256U;
        allocate_device(&reduction_blocks, reduction_capacity, allocated_bytes);
        allocate_device(&energy_result, 1U, allocated_bytes);
        allocate_device(&norm_result, 1U, allocated_bytes);
        allocate_device(&evaluation_work, 1U, allocated_bytes);
        allocate_device(&error, 1U, allocated_bytes);
        allocate_device(&graph_work, 1U, allocated_bytes);
        allocate_device(&maximum_degree, 1U, allocated_bytes);

        cuda_check(cub::DeviceRadixSort::SortPairs(nullptr, id_sort_bytes,
            ids_input, ids_sorted, input_indices, sorted_indices,
            static_cast<int>(dynamic_capacity)), "query ID sort");
        cuda_check(cub::DeviceRadixSort::SortPairs(nullptr, cell_sort_bytes,
            cell_keys_input, cell_keys_sorted, cell_indices_input,
            cell_indices_sorted, static_cast<int>(total_capacity)),
            "query cell sort");
        cuda_check(cub::DeviceScan::ExclusiveSum(nullptr, scan_bytes,
            counts, offsets, static_cast<int>(dynamic_capacity)),
            "query graph scan");
        allocate_device(&id_sort_storage, id_sort_bytes, allocated_bytes);
        allocate_device(&cell_sort_storage, cell_sort_bytes, allocated_bytes);
        allocate_device(&scan_storage, scan_bytes, allocated_bytes);
        for (cudaEvent_t& event : events) {
            cuda_check(cudaEventCreate(&event), "create NCGP1 event");
        }
    }

    ~Impl() {
        for (cudaEvent_t event : events) cudaEventDestroy(event);
        cudaFree(scan_storage);
        cudaFree(cell_sort_storage);
        cudaFree(id_sort_storage);
        cudaFree(maximum_degree);
        cudaFree(graph_work);
        cudaFree(error);
        cudaFree(evaluation_work);
        cudaFree(norm_result);
        cudaFree(energy_result);
        cudaFree(reduction_blocks);
        cudaFree(norm_rows);
        cudaFree(energy_rows);
        cudaFree(direction);
        cudaFree(diagonal);
        cudaFree(hvp);
        cudaFree(gradient);
        cudaFree(pressure_q);
        cudaFree(pressure_excess);
        cudaFree(energy_density);
        cudaFree(density);
        cudaFree(reference_neighbors);
        cudaFree(reference_offsets);
        cudaFree(neighbors);
        cudaFree(offsets);
        cudaFree(counts);
        cudaFree(cell_indices_sorted);
        cudaFree(cell_indices_input);
        cudaFree(cell_keys_sorted);
        cudaFree(cell_keys_input);
        cudaFree(ghost_positions_input);
        cudaFree(ghost_ids_input);
        cudaFree(all_positions);
        cudaFree(velocity);
        cudaFree(predicted);
        cudaFree(current);
        cudaFree(reference);
        cudaFree(ids);
        cudaFree(velocity_input);
        cudaFree(current_input);
        cudaFree(reference_input);
        cudaFree(sorted_indices);
        cudaFree(input_indices);
        cudaFree(ids_sorted);
        cudaFree(ids_input);
    }

    NonlocalGpuProfile profile;
    unsigned int* ids_input = nullptr;
    unsigned int* ids_sorted = nullptr;
    unsigned int* input_indices = nullptr;
    unsigned int* sorted_indices = nullptr;
    DeviceVec3* reference_input = nullptr;
    DeviceVec3* current_input = nullptr;
    DeviceVec3* velocity_input = nullptr;
    unsigned int* ids = nullptr;
    DeviceVec3* reference = nullptr;
    DeviceVec3* current = nullptr;
    DeviceVec3* predicted = nullptr;
    DeviceVec3* velocity = nullptr;
    DeviceVec3* all_positions = nullptr;
    unsigned int* ghost_ids_input = nullptr;
    DeviceVec3* ghost_positions_input = nullptr;
    unsigned long long* cell_keys_input = nullptr;
    unsigned long long* cell_keys_sorted = nullptr;
    unsigned int* cell_indices_input = nullptr;
    unsigned int* cell_indices_sorted = nullptr;
    unsigned int* counts = nullptr;
    unsigned int* offsets = nullptr;
    unsigned int* neighbors = nullptr;
    unsigned int* reference_offsets = nullptr;
    unsigned int* reference_neighbors = nullptr;
    float* density = nullptr;
    double* energy_density = nullptr;
    float* pressure_excess = nullptr;
    float* pressure_q = nullptr;
    DeviceVec3* gradient = nullptr;
    DeviceVec3* hvp = nullptr;
    DeviceVec3* diagonal = nullptr;
    DeviceVec3* direction = nullptr;
    double* energy_rows = nullptr;
    double* norm_rows = nullptr;
    double* reduction_blocks = nullptr;
    double* energy_result = nullptr;
    double* norm_result = nullptr;
    DeviceEvaluationWork* evaluation_work = nullptr;
    int* error = nullptr;
    DeviceGraphWork* graph_work = nullptr;
    unsigned int* maximum_degree = nullptr;
    unsigned char* id_sort_storage = nullptr;
    unsigned char* cell_sort_storage = nullptr;
    unsigned char* scan_storage = nullptr;
    std::size_t id_sort_bytes = 0U;
    std::size_t cell_sort_bytes = 0U;
    std::size_t scan_bytes = 0U;
    std::size_t allocated_bytes = 0U;
    int dynamic_count = 0;
    int ghost_count = 0;
    std::vector<std::uint32_t> host_sorted_ids;
    std::vector<std::uint32_t> host_all_ids;
    std::array<cudaEvent_t, 8> events{};
};

NonlocalGpuWorkspace::NonlocalGpuWorkspace(const NonlocalGpuProfile& profile)
    : impl_(new Impl(profile)) {}

NonlocalGpuWorkspace::~NonlocalGpuWorkspace() { delete impl_; }

NonlocalGpuFailure NonlocalGpuWorkspace::upload(
    const std::vector<NonlocalGpuSample>& samples,
    const std::vector<NonlocalGpuGhost>& ghosts) {
    if (samples.empty() || samples.size() > impl_->profile.maximum_dynamic_samples
        || samples.size() + ghosts.size() > kMaximumTotalSamples) {
        return NonlocalGpuFailure::CapacityExceeded;
    }
    std::unordered_set<std::uint32_t> ids;
    std::vector<unsigned int> host_ids;
    std::vector<DeviceVec3> reference;
    std::vector<DeviceVec3> current;
    std::vector<DeviceVec3> velocity;
    host_ids.reserve(samples.size());
    reference.reserve(samples.size());
    current.reserve(samples.size());
    velocity.reserve(samples.size());
    const auto convert = [](const Vec3d& value) {
        return DeviceVec3{static_cast<float>(value.x), static_cast<float>(value.y),
            static_cast<float>(value.z)};
    };
    for (const auto& sample : samples) {
        if (!ids.insert(sample.sample_id).second) {
            return NonlocalGpuFailure::DuplicateSampleId;
        }
        const std::array<double, 9> values{sample.reference.x, sample.reference.y,
            sample.reference.z, sample.current.x, sample.current.y,
            sample.current.z, sample.velocity.x, sample.velocity.y,
            sample.velocity.z};
        if (!std::all_of(values.begin(), values.end(),
                [](double value) { return std::isfinite(value); })) {
            return NonlocalGpuFailure::Nonfinite;
        }
        host_ids.push_back(sample.sample_id);
        reference.push_back(convert(sample.reference));
        current.push_back(convert(sample.current));
        velocity.push_back(convert(sample.velocity));
    }
    std::vector<unsigned int> ghost_ids;
    std::vector<DeviceVec3> ghost_positions;
    ghost_ids.reserve(ghosts.size());
    ghost_positions.reserve(ghosts.size());
    std::uint32_t prior_ghost_id = 0U;
    for (std::size_t index = 0U; index < ghosts.size(); ++index) {
        const auto& ghost = ghosts[index];
        if (!ids.insert(ghost.sample_id).second
            || (index > 0U && ghost.sample_id <= prior_ghost_id)) {
            return NonlocalGpuFailure::DuplicateSampleId;
        }
        prior_ghost_id = ghost.sample_id;
        ghost_ids.push_back(ghost.sample_id);
        ghost_positions.push_back(convert(ghost.position));
    }
    try {
        impl_->dynamic_count = static_cast<int>(samples.size());
        impl_->ghost_count = static_cast<int>(ghosts.size());
        const std::size_t dynamic_bytes = samples.size() * sizeof(DeviceVec3);
        cuda_check(cudaMemset(impl_->error, 0, sizeof(int)), "reset upload error");
        cuda_check(cudaMemcpy(impl_->ids_input, host_ids.data(),
            samples.size() * sizeof(unsigned int), cudaMemcpyHostToDevice),
            "upload sample IDs");
        cuda_check(cudaMemcpy(impl_->reference_input, reference.data(), dynamic_bytes,
            cudaMemcpyHostToDevice), "upload reference positions");
        cuda_check(cudaMemcpy(impl_->current_input, current.data(), dynamic_bytes,
            cudaMemcpyHostToDevice), "upload current positions");
        cuda_check(cudaMemcpy(impl_->velocity_input, velocity.data(), dynamic_bytes,
            cudaMemcpyHostToDevice), "upload velocities");
        initialize_indices<<<blocks_for(impl_->dynamic_count), kThreads>>>(
            impl_->input_indices, impl_->dynamic_count);
        cuda_check(cudaGetLastError(), "initialize upload indices");
        cuda_check(cub::DeviceRadixSort::SortPairs(impl_->id_sort_storage,
            impl_->id_sort_bytes, impl_->ids_input, impl_->ids_sorted,
            impl_->input_indices, impl_->sorted_indices, impl_->dynamic_count),
            "sort sample IDs");
        validate_sorted_ids<<<blocks_for(impl_->dynamic_count), kThreads>>>(
            impl_->ids_sorted, impl_->dynamic_count, impl_->error);
        const DeviceVec3 gravity{static_cast<float>(impl_->profile.gravity.x),
            static_cast<float>(impl_->profile.gravity.y),
            static_cast<float>(impl_->profile.gravity.z)};
        gather_canonical_state<<<blocks_for(impl_->dynamic_count), kThreads>>>(
            impl_->ids_sorted, impl_->sorted_indices, impl_->reference_input,
            impl_->current_input, impl_->velocity_input, impl_->ids,
            impl_->reference, impl_->current, impl_->predicted, impl_->velocity,
            impl_->dynamic_count, gravity, static_cast<float>(impl_->profile.dt));
        cuda_check(cudaGetLastError(), "gather canonical state");
        if (impl_->ghost_count > 0) {
            cuda_check(cudaMemcpy(impl_->ghost_ids_input, ghost_ids.data(),
                ghosts.size() * sizeof(unsigned int), cudaMemcpyHostToDevice),
                "upload ghost IDs");
            cuda_check(cudaMemcpy(impl_->ghost_positions_input,
                ghost_positions.data(), ghosts.size() * sizeof(DeviceVec3),
                cudaMemcpyHostToDevice), "upload ghost positions");
            append_ghosts<<<blocks_for(impl_->ghost_count), kThreads>>>(
                impl_->ghost_ids_input, impl_->ghost_positions_input, impl_->ids,
                impl_->all_positions, impl_->dynamic_count, impl_->ghost_count);
            cuda_check(cudaGetLastError(), "append ghost state");
        }
        cuda_check(cudaDeviceSynchronize(), "synchronize NCGP1 upload");
        int error = 0;
        cuda_check(cudaMemcpy(&error, impl_->error, sizeof(int),
            cudaMemcpyDeviceToHost), "copy upload error");
        if (error != 0) return static_cast<NonlocalGpuFailure>(error);
        impl_->host_sorted_ids.resize(samples.size());
        cuda_check(cudaMemcpy(impl_->host_sorted_ids.data(), impl_->ids,
            samples.size() * sizeof(unsigned int), cudaMemcpyDeviceToHost),
            "capture canonical IDs");
        impl_->host_all_ids = impl_->host_sorted_ids;
        impl_->host_all_ids.insert(
            impl_->host_all_ids.end(), ghost_ids.begin(), ghost_ids.end());
        return NonlocalGpuFailure::None;
    } catch (const std::exception&) {
        return NonlocalGpuFailure::DeviceFailure;
    }
}

NonlocalGpuGraphResult NonlocalGpuWorkspace::build_current_graph(
    NonlocalGpuVariant variant,
    bool capture_payload,
    bool measure) {
    NonlocalGpuGraphResult result;
    result.dynamic_samples = static_cast<std::uint32_t>(impl_->dynamic_count);
    result.ghost_samples = static_cast<std::uint32_t>(impl_->ghost_count);
    if (impl_->dynamic_count <= 0) {
        result.failure = NonlocalGpuFailure::InvalidState;
        return result;
    }
    try {
        const int total_count = impl_->dynamic_count + impl_->ghost_count;
        cuda_check(cudaMemsetAsync(impl_->error, 0, sizeof(int)),
            "reset graph error");
        cuda_check(cudaMemsetAsync(impl_->graph_work, 0, sizeof(DeviceGraphWork)),
            "reset graph work");
        cuda_check(cudaMemsetAsync(impl_->maximum_degree, 0, sizeof(unsigned int)),
            "reset maximum degree");
        if (measure) cuda_check(cudaEventRecord(impl_->events[0]), "graph start");
        copy_dynamic_positions<<<blocks_for(impl_->dynamic_count), kThreads>>>(
            impl_->current, impl_->all_positions, impl_->dynamic_count);
        cuda_check(cudaGetLastError(), "copy current graph positions");
        compute_cell_keys<<<blocks_for(total_count), kThreads>>>(
            impl_->all_positions, impl_->cell_keys_input,
            impl_->cell_indices_input, total_count,
            static_cast<long long>(std::llround(
                impl_->profile.horizon * kMicrometresPerMetre)),
            impl_->error);
        cuda_check(cudaGetLastError(), "compute graph keys");
        cuda_check(cub::DeviceRadixSort::SortPairs(impl_->cell_sort_storage,
            impl_->cell_sort_bytes, impl_->cell_keys_input,
            impl_->cell_keys_sorted, impl_->cell_indices_input,
            impl_->cell_indices_sorted, total_count), "sort graph cells");
        const bool strict = variant == NonlocalGpuVariant::StrictRadius;
        const long long support_um = static_cast<long long>(std::llround(
            impl_->profile.horizon * kMicrometresPerMetre));
        visit_neighbors<false><<<blocks_for(impl_->dynamic_count), kThreads>>>(
            impl_->all_positions, impl_->ids, impl_->cell_keys_sorted,
            impl_->cell_indices_sorted, impl_->counts, nullptr, nullptr,
            impl_->dynamic_count, total_count, support_um, strict,
            impl_->graph_work, impl_->error);
        cuda_check(cudaGetLastError(), "count graph neighbors");
        cuda_check(cub::DeviceScan::ExclusiveSum(impl_->scan_storage,
            impl_->scan_bytes, impl_->counts, impl_->offsets,
            impl_->dynamic_count), "scan graph counts");
        finish_offsets<<<1, 1>>>(impl_->counts, impl_->offsets,
            impl_->dynamic_count);
        cuda_check(cudaGetLastError(), "finish graph offsets");
        visit_neighbors<true><<<blocks_for(impl_->dynamic_count), kThreads>>>(
            impl_->all_positions, impl_->ids, impl_->cell_keys_sorted,
            impl_->cell_indices_sorted, nullptr, impl_->offsets,
            impl_->neighbors, impl_->dynamic_count, total_count, support_um,
            strict, impl_->graph_work, impl_->error);
        cuda_check(cudaGetLastError(), "fill graph neighbors");
        sort_neighbor_rows<<<impl_->dynamic_count, kRowSortThreads,
            kMaximumNeighbors * sizeof(unsigned int)>>>(impl_->offsets,
            impl_->neighbors, impl_->dynamic_count, impl_->error);
        cuda_check(cudaGetLastError(), "sort graph rows");
        maximum_degree_kernel<<<blocks_for(impl_->dynamic_count), kThreads>>>(
            impl_->offsets, impl_->dynamic_count, impl_->maximum_degree);
        cuda_check(cudaGetLastError(), "find maximum graph degree");
        if (measure) {
            cuda_check(cudaEventRecord(impl_->events[1]), "graph stop");
            cuda_check(cudaEventSynchronize(impl_->events[1]),
                "synchronize measured graph");
            cuda_check(cudaEventElapsedTime(&result.timing.graph_ms,
                impl_->events[0], impl_->events[1]), "read graph time");
            result.timing.total_ms = result.timing.graph_ms;
        } else {
            cuda_check(cudaDeviceSynchronize(), "synchronize graph");
        }
        int error = 0;
        DeviceGraphWork device_work{};
        cuda_check(cudaMemcpy(&error, impl_->error, sizeof(int),
            cudaMemcpyDeviceToHost), "copy graph error");
        cuda_check(cudaMemcpy(&device_work, impl_->graph_work,
            sizeof(device_work), cudaMemcpyDeviceToHost), "copy graph work");
        cuda_check(cudaMemcpy(&result.maximum_degree, impl_->maximum_degree,
            sizeof(unsigned int), cudaMemcpyDeviceToHost),
            "copy maximum degree");
        cuda_check(cudaMemcpy(&result.directed_pairs,
            impl_->offsets + impl_->dynamic_count, sizeof(unsigned int),
            cudaMemcpyDeviceToHost), "copy directed pair count");
        result.failure = static_cast<NonlocalGpuFailure>(error);
        result.work.graph_builds = 1U;
        result.work.key_evaluations = static_cast<std::uint64_t>(total_count);
        result.work.radix_sort_items = static_cast<std::uint64_t>(total_count);
        result.work.cell_probes = device_work.cell_probes;
        result.work.distance_predicates = device_work.distance_predicates;
        result.work.emitted_directed_pairs = device_work.emitted_pairs;
        result.work.row_sort_items = result.directed_pairs;
        result.work.device_to_host_bytes = sizeof(error) + sizeof(device_work)
            + 2U * sizeof(unsigned int);
        if (capture_payload && result.failure == NonlocalGpuFailure::None) {
            result.owner_ids = impl_->host_sorted_ids;
            result.offsets.resize(static_cast<std::size_t>(impl_->dynamic_count) + 1U);
            std::vector<unsigned int> neighbor_indices(result.directed_pairs);
            cuda_check(cudaMemcpy(result.offsets.data(), impl_->offsets,
                result.offsets.size() * sizeof(unsigned int),
                cudaMemcpyDeviceToHost), "capture graph offsets");
            if (!neighbor_indices.empty()) {
                cuda_check(cudaMemcpy(neighbor_indices.data(), impl_->neighbors,
                    neighbor_indices.size() * sizeof(unsigned int),
                    cudaMemcpyDeviceToHost), "capture graph neighbors");
            }
            result.neighbor_ids.reserve(neighbor_indices.size());
            for (const auto index : neighbor_indices) {
                if (index >= impl_->host_all_ids.size()) {
                    result.failure = NonlocalGpuFailure::InvalidState;
                    result.neighbor_ids.clear();
                    break;
                }
                result.neighbor_ids.push_back(impl_->host_all_ids[index]);
            }
            result.work.device_to_host_bytes += result.offsets.size()
                    * sizeof(unsigned int)
                + neighbor_indices.size() * sizeof(unsigned int);
        }
        return result;
    } catch (const std::exception&) {
        result.failure = NonlocalGpuFailure::DeviceFailure;
        return result;
    }
}

NonlocalGpuEvaluationResult NonlocalGpuWorkspace::evaluate(
    const std::vector<Vec3d>* host_direction,
    NonlocalGpuVariant variant,
    bool capture_payload,
    bool measure) {
    NonlocalGpuEvaluationResult result;
    if (impl_->dynamic_count <= 0
        || (host_direction != nullptr
            && host_direction->size() != static_cast<std::size_t>(impl_->dynamic_count))) {
        result.failure = NonlocalGpuFailure::InvalidState;
        return result;
    }
    try {
        std::swap(impl_->current, impl_->reference);
        const auto reference_graph = build_current_graph(
            NonlocalGpuVariant::Corrected, false, measure);
        std::swap(impl_->current, impl_->reference);
        if (reference_graph.failure != NonlocalGpuFailure::None) {
            result.failure = reference_graph.failure;
            return result;
        }
        cuda_check(cudaMemcpy(impl_->reference_offsets, impl_->offsets,
            (static_cast<std::size_t>(impl_->dynamic_count) + 1U)
                * sizeof(unsigned int),
            cudaMemcpyDeviceToDevice), "retain reference offsets");
        cuda_check(cudaMemcpy(impl_->reference_neighbors, impl_->neighbors,
            static_cast<std::size_t>(reference_graph.directed_pairs)
                * sizeof(unsigned int),
            cudaMemcpyDeviceToDevice), "retain reference neighbors");
        const NonlocalGpuVariant graph_variant = variant
                == NonlocalGpuVariant::StrictRadius
            ? NonlocalGpuVariant::StrictRadius
            : NonlocalGpuVariant::Corrected;
        const auto current_graph = build_current_graph(
            graph_variant, false, measure);
        if (current_graph.failure != NonlocalGpuFailure::None) {
            result.failure = current_graph.failure;
            return result;
        }

        if (host_direction != nullptr) {
            std::vector<DeviceVec3> converted;
            converted.reserve(host_direction->size());
            for (const Vec3d& value : *host_direction) {
                if (!std::isfinite(value.x) || !std::isfinite(value.y)
                    || !std::isfinite(value.z)) {
                    result.failure = NonlocalGpuFailure::Nonfinite;
                    return result;
                }
                converted.push_back({static_cast<float>(value.x),
                    static_cast<float>(value.y), static_cast<float>(value.z)});
            }
            cuda_check(cudaMemcpy(impl_->direction, converted.data(),
                converted.size() * sizeof(DeviceVec3), cudaMemcpyHostToDevice),
                "upload HVP direction");
            result.work.host_to_device_bytes += converted.size()
                * sizeof(DeviceVec3);
        }
        cuda_check(cudaMemsetAsync(impl_->error, 0, sizeof(int)),
            "reset evaluation error");
        cuda_check(cudaMemsetAsync(impl_->evaluation_work, 0,
            sizeof(DeviceEvaluationWork)), "reset evaluation work");
        if (measure) cuda_check(cudaEventRecord(impl_->events[2]), "evaluation start");
        const DeviceProfile profile{static_cast<float>(impl_->profile.dt),
            static_cast<float>(impl_->profile.spacing),
            static_cast<float>(impl_->profile.horizon),
            static_cast<float>(impl_->profile.mass),
            static_cast<float>(impl_->profile.rest_density),
            static_cast<float>(impl_->profile.kappa),
            static_cast<float>(impl_->profile.lambda),
            static_cast<float>(impl_->profile.mu),
            static_cast<float>(impl_->profile.gamma)};
        density_kernel<<<blocks_for(impl_->dynamic_count), kThreads>>>(
            impl_->current, impl_->all_positions, impl_->offsets,
            impl_->neighbors, impl_->density, impl_->energy_density,
            impl_->pressure_excess, impl_->dynamic_count, profile,
            impl_->evaluation_work, impl_->error);
        cuda_check(cudaGetLastError(), "evaluate density");
        energy_gradient_kernel<<<blocks_for(impl_->dynamic_count), kThreads>>>(
            impl_->reference, impl_->current, impl_->predicted,
            impl_->all_positions, impl_->offsets, impl_->neighbors,
            impl_->reference_offsets, impl_->reference_neighbors,
            impl_->pressure_excess, impl_->energy_density, impl_->gradient,
            impl_->energy_rows, impl_->dynamic_count, profile,
            static_cast<unsigned int>(variant), impl_->evaluation_work,
            impl_->error);
        cuda_check(cudaGetLastError(), "evaluate energy and gradient");
        const int reduction_blocks = (impl_->dynamic_count + 255) / 256;
        reduce_double_blocks<<<reduction_blocks, 256>>>(impl_->energy_rows,
            impl_->reduction_blocks, impl_->dynamic_count);
        reduce_double_final<<<1, 256>>>(impl_->reduction_blocks,
            impl_->energy_result, reduction_blocks);
        vector_norm_rows<<<blocks_for(impl_->dynamic_count), kThreads>>>(
            impl_->gradient, impl_->norm_rows, impl_->dynamic_count);
        reduce_double_blocks<<<reduction_blocks, 256>>>(impl_->norm_rows,
            impl_->reduction_blocks, impl_->dynamic_count);
        reduce_double_final<<<1, 256>>>(impl_->reduction_blocks,
            impl_->norm_result, reduction_blocks);
        cuda_check(cudaGetLastError(), "reduce evaluation scalars");

        if (host_direction != nullptr) {
            pressure_directional_kernel<<<blocks_for(impl_->dynamic_count),
                kThreads>>>(impl_->current, impl_->all_positions,
                impl_->direction, impl_->offsets, impl_->neighbors,
                impl_->pressure_q, impl_->dynamic_count, profile,
                static_cast<unsigned int>(variant), impl_->error);
            cuda_check(cudaGetLastError(), "evaluate pressure directional");
            hvp_kernel<<<blocks_for(impl_->dynamic_count), kThreads>>>(
                impl_->reference, impl_->current, impl_->all_positions,
                impl_->direction, impl_->offsets, impl_->neighbors,
                impl_->reference_offsets, impl_->reference_neighbors,
                impl_->pressure_excess, impl_->pressure_q, impl_->hvp,
                impl_->diagonal, impl_->dynamic_count, profile,
                static_cast<unsigned int>(variant), impl_->evaluation_work,
                impl_->error);
            cuda_check(cudaGetLastError(), "evaluate matrix-free HVP");
        }
        if (measure) {
            cuda_check(cudaEventRecord(impl_->events[3]), "evaluation stop");
            cuda_check(cudaEventSynchronize(impl_->events[3]),
                "synchronize measured evaluation");
            cuda_check(cudaEventElapsedTime(
                &result.timing.density_energy_gradient_ms,
                impl_->events[2], impl_->events[3]), "read evaluation time");
        } else {
            cuda_check(cudaDeviceSynchronize(), "synchronize evaluation");
        }
        int error = 0;
        DeviceEvaluationWork work{};
        double energy = 0.0;
        double norm_squared = 0.0;
        cuda_check(cudaMemcpy(&error, impl_->error, sizeof(int),
            cudaMemcpyDeviceToHost), "copy evaluation error");
        cuda_check(cudaMemcpy(&work, impl_->evaluation_work, sizeof(work),
            cudaMemcpyDeviceToHost), "copy evaluation work");
        cuda_check(cudaMemcpy(&energy, impl_->energy_result, sizeof(double),
            cudaMemcpyDeviceToHost), "copy energy");
        cuda_check(cudaMemcpy(&norm_squared, impl_->norm_result, sizeof(double),
            cudaMemcpyDeviceToHost), "copy gradient norm");
        result.failure = static_cast<NonlocalGpuFailure>(error);
        result.energy = energy;
        result.gradient_norm = std::sqrt(norm_squared);
        result.active_pressure_centers = static_cast<std::uint32_t>(
            work.active_pressure_centers);
        result.work.graph_builds = reference_graph.work.graph_builds
            + current_graph.work.graph_builds;
        result.work.key_evaluations = reference_graph.work.key_evaluations
            + current_graph.work.key_evaluations;
        result.work.radix_sort_items = reference_graph.work.radix_sort_items
            + current_graph.work.radix_sort_items;
        result.work.cell_probes = reference_graph.work.cell_probes
            + current_graph.work.cell_probes;
        result.work.distance_predicates = reference_graph.work.distance_predicates
            + current_graph.work.distance_predicates;
        result.work.emitted_directed_pairs = reference_graph.work.emitted_directed_pairs
            + current_graph.work.emitted_directed_pairs;
        result.work.row_sort_items = reference_graph.work.row_sort_items
            + current_graph.work.row_sort_items;
        result.work.density_kernel_evaluations = work.density_kernel_evaluations;
        result.work.energy_pair_visits = work.energy_pair_visits;
        result.work.gradient_pair_visits = work.gradient_pair_visits;
        result.work.hvp_pair_visits = work.hvp_pair_visits;
        result.work.hvp_applications = host_direction != nullptr ? 1U : 0U;
        result.work.reduction_values = static_cast<std::uint64_t>(
            impl_->dynamic_count) * 2U;
        result.work.device_to_host_bytes += reference_graph.work.device_to_host_bytes
            + current_graph.work.device_to_host_bytes + sizeof(error)
            + sizeof(work) + 2U * sizeof(double);
        result.timing.graph_ms = reference_graph.timing.graph_ms
            + current_graph.timing.graph_ms;
        result.timing.total_ms = result.timing.graph_ms
            + result.timing.density_energy_gradient_ms;

        if (capture_payload && result.failure == NonlocalGpuFailure::None) {
            std::vector<DeviceVec3> gradient(impl_->dynamic_count);
            std::vector<float> density(impl_->dynamic_count);
            cuda_check(cudaMemcpy(gradient.data(), impl_->gradient,
                gradient.size() * sizeof(DeviceVec3), cudaMemcpyDeviceToHost),
                "capture gradient");
            cuda_check(cudaMemcpy(density.data(), impl_->density,
                density.size() * sizeof(float), cudaMemcpyDeviceToHost),
                "capture density");
            result.gradient.reserve(gradient.size());
            result.density.reserve(density.size());
            for (std::size_t index = 0U; index < gradient.size(); ++index) {
                result.gradient.push_back({gradient[index].x,
                    gradient[index].y, gradient[index].z});
                result.density.push_back(density[index]);
            }
            result.work.device_to_host_bytes += gradient.size()
                    * sizeof(DeviceVec3)
                + density.size() * sizeof(float);
            if (host_direction != nullptr) {
                std::vector<DeviceVec3> hvp(impl_->dynamic_count);
                cuda_check(cudaMemcpy(hvp.data(), impl_->hvp,
                    hvp.size() * sizeof(DeviceVec3), cudaMemcpyDeviceToHost),
                    "capture HVP");
                result.hvp.reserve(hvp.size());
                for (const auto& value : hvp) {
                    result.hvp.push_back({value.x, value.y, value.z});
                }
                result.work.device_to_host_bytes += hvp.size()
                    * sizeof(DeviceVec3);
            }
        }
        return result;
    } catch (const std::exception&) {
        if (impl_->current == impl_->reference) {
            // Both pointers are never intentionally aliased. This branch only
            // keeps the exception path side-effect free for static analysis.
        }
        result.failure = NonlocalGpuFailure::DeviceFailure;
        return result;
    }
}

NonlocalGpuStepResult NonlocalGpuWorkspace::step(
    std::uint32_t, NonlocalGpuVariant, bool, bool) {
    NonlocalGpuStepResult result;
    result.failure = NonlocalGpuFailure::InvalidState;
    return result;
}

std::string NonlocalGpuWorkspace::environment_json() const {
    cudaDeviceProp properties{};
    int device = 0;
    int runtime = 0;
    int driver = 0;
    cuda_check(cudaGetDevice(&device), "get CUDA device");
    cuda_check(cudaGetDeviceProperties(&properties, device),
        "get CUDA properties");
    cuda_check(cudaRuntimeGetVersion(&runtime), "get CUDA runtime");
    cuda_check(cudaDriverGetVersion(&driver), "get CUDA driver");
    std::ostringstream output;
    output << "{\"device\":\"" << properties.name << "\",\"major\":"
           << properties.major << ",\"minor\":" << properties.minor
           << ",\"runtime\":" << runtime << ",\"driver\":" << driver
           << ",\"allocated_device_bytes\":" << impl_->allocated_bytes << '}';
    return output.str();
}

std::size_t NonlocalGpuWorkspace::allocated_device_bytes() const {
    return impl_->allocated_bytes;
}

} // namespace nextengine::nonlocal::gpu_full_step
