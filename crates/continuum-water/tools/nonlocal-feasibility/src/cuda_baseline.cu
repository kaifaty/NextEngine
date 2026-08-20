#include "cuda_baseline.hpp"

#include "sha256.hpp"

#include <cub/cub.cuh>
#include <cuda_runtime.h>

#include <algorithm>
#include <array>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <fstream>
#include <iomanip>
#include <iterator>
#include <limits>
#include <numeric>
#include <sstream>
#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

namespace nextengine::nonlocal {
namespace {

constexpr float CUDA_PI = 3.14159265358979323846F;
constexpr float CUDA_PAIR_EPSILON = 1.0e-12F;
constexpr int THREADS = 256;

void check_cuda(cudaError_t result, const char* operation) {
    if (result != cudaSuccess) {
        throw std::runtime_error(std::string(operation) + ": " + cudaGetErrorString(result));
    }
}

int blocks_for(std::size_t count) {
    return static_cast<int>((count + static_cast<std::size_t>(THREADS) - 1U)
        / static_cast<std::size_t>(THREADS));
}

__device__ float3 add3(float3 a, float3 b) {
    return make_float3(a.x + b.x, a.y + b.y, a.z + b.z);
}

__device__ float3 subtract3(float3 a, float3 b) {
    return make_float3(a.x - b.x, a.y - b.y, a.z - b.z);
}

__device__ float3 multiply3(float scalar, float3 a) {
    return make_float3(scalar * a.x, scalar * a.y, scalar * a.z);
}

__device__ float dot3(float3 a, float3 b) { return a.x * b.x + a.y * b.y + a.z * b.z; }

__device__ float length3(float3 a) { return sqrtf(dot3(a, a)); }

__device__ float device_cubic_weight(float radius, float horizon, float scale) {
    // Independent CUDA transcription of the pinned CubicKernel::weight.
    const float q = 2.0F * radius / horizon;
    const float alpha = 3.0F * scale / (2.0F * CUDA_PI * horizon * horizon * horizon);
    if (q > 2.0F) {
        return 0.0F;
    }
    if (q >= 1.0F) {
        const float delta = 2.0F - q;
        return alpha * delta * delta * delta / 6.0F;
    }
    return alpha * (2.0F / 3.0F - q * q + 0.5F * q * q * q);
}

__device__ float device_cubic_gradient(float radius, float horizon, float scale) {
    // Independent CUDA transcription of the pinned CubicKernel::gradient.
    const float q = 2.0F * radius / horizon;
    const float alpha = 3.0F * scale / (2.0F * CUDA_PI * horizon * horizon * horizon);
    if (q > 2.0F) {
        return 0.0F;
    }
    if (q >= 1.0F) {
        const float delta = 2.0F - q;
        return -0.5F * alpha * delta * delta;
    }
    return alpha * (-2.0F * q + 1.5F * q * q);
}

__device__ int cell_key(float3 position, float3 origin, float cell_size, int3 dimensions) {
    const int x = max(0, min(dimensions.x - 1, static_cast<int>(floorf((position.x - origin.x) / cell_size))));
    const int y = max(0, min(dimensions.y - 1, static_cast<int>(floorf((position.y - origin.y) / cell_size))));
    const int z = max(0, min(dimensions.z - 1, static_cast<int>(floorf((position.z - origin.z) / cell_size))));
    return x + dimensions.x * (y + dimensions.y * z);
}

__global__ void compute_grid_keys(
    const float3* position,
    int* keys,
    int* indices,
    int count,
    float3 origin,
    float cell_size,
    int3 dimensions,
    int* error_flag) {
    const int index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= count) {
        return;
    }
    const float3 point = position[index];
    const float x = (point.x - origin.x) / cell_size;
    const float y = (point.y - origin.y) / cell_size;
    const float z = (point.z - origin.z) / cell_size;
    if (!isfinite(x) || !isfinite(y) || !isfinite(z)
        || x < 0.0F || x >= static_cast<float>(dimensions.x)
        || y < 0.0F || y >= static_cast<float>(dimensions.y)
        || z < 0.0F || z >= static_cast<float>(dimensions.z)) {
        atomicExch(error_flag, 1);
    }
    keys[index] = cell_key(point, origin, cell_size, dimensions);
    indices[index] = index;
}

__global__ void mark_cell_ranges(
    const int* sorted_keys,
    int* cell_start,
    int* cell_end,
    int count) {
    const int index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= count) {
        return;
    }
    const int key = sorted_keys[index];
    if (index == 0 || sorted_keys[index - 1] != key) {
        cell_start[key] = index;
    }
    if (index == count - 1 || sorted_keys[index + 1] != key) {
        cell_end[key] = index + 1;
    }
}

template <bool Fill, typename NeighborIndex = int>
__global__ void visit_grid_neighbors(
    const float3* position,
    const int* sorted_indices,
    const int* cell_start,
    const int* cell_end,
    int* counts,
    const int* offsets,
    NeighborIndex* neighbors,
    int count,
    float3 origin,
    float horizon,
    int3 dimensions,
    int* error_flag) {
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }
    const float3 point = position[particle];
    const int own_key = cell_key(point, origin, horizon, dimensions);
    const int own_x = own_key % dimensions.x;
    const int own_y = (own_key / dimensions.x) % dimensions.y;
    const int own_z = own_key / (dimensions.x * dimensions.y);
    int cursor = Fill ? offsets[particle] : 0;
    int degree = 0;
    const float support_squared = horizon * horizon * (1.0F + 2.0e-6F);
    for (int dz = -1; dz <= 1; ++dz) {
        const int z = own_z + dz;
        if (z < 0 || z >= dimensions.z) {
            continue;
        }
        for (int dy = -1; dy <= 1; ++dy) {
            const int y = own_y + dy;
            if (y < 0 || y >= dimensions.y) {
                continue;
            }
            for (int dx = -1; dx <= 1; ++dx) {
                const int x = own_x + dx;
                if (x < 0 || x >= dimensions.x) {
                    continue;
                }
                const int key = x + dimensions.x * (y + dimensions.y * z);
                const int begin = cell_start[key];
                if (begin < 0) {
                    continue;
                }
                for (int slot = begin; slot < cell_end[key]; ++slot) {
                    const int candidate = sorted_indices[slot];
                    const float3 delta = subtract3(point, position[candidate]);
                    if (dot3(delta, delta) <= support_squared) {
                        if constexpr (Fill) {
                            if (cursor < offsets[particle + 1]) {
                                neighbors[cursor++] = static_cast<NeighborIndex>(candidate);
                            } else {
                                atomicExch(error_flag, 1);
                            }
                        }
                        ++degree;
                    }
                }
            }
        }
    }
    if constexpr (!Fill) {
        counts[particle] = degree;
    }
}

__global__ void initialize_storage_map(
    const int* sorted_keys,
    const int* sorted_indices,
    int* storage_to_sample,
    int* sample_to_storage,
    int* error_flag,
    int count) {
    const int storage = blockIdx.x * blockDim.x + threadIdx.x;
    if (storage >= count) {
        return;
    }
    const int sample = sorted_indices[storage];
    if (sample < 0 || sample >= count) {
        atomicExch(error_flag, 1);
        return;
    }
    if (storage > 0) {
        const int previous_key = sorted_keys[storage - 1];
        const int previous_sample = sorted_indices[storage - 1];
        if (previous_key > sorted_keys[storage]
            || (previous_key == sorted_keys[storage] && previous_sample >= sample)) {
            atomicExch(error_flag, 1);
        }
    }
    storage_to_sample[storage] = sample;
    sample_to_storage[sample] = storage;
}

__global__ void validate_storage_map(
    const int* sorted_keys,
    const int* sorted_indices,
    const int* storage_to_sample,
    const int* sample_to_storage,
    int* error_flag,
    int count) {
    const int storage = blockIdx.x * blockDim.x + threadIdx.x;
    if (storage >= count) {
        return;
    }
    const int sample = sorted_indices[storage];
    if (sample < 0 || sample >= count || storage_to_sample[storage] != sample
        || sample_to_storage[sample] != storage) {
        atomicExch(error_flag, 1);
        return;
    }
    if (storage > 0) {
        const int previous_key = sorted_keys[storage - 1];
        const int previous_sample = sorted_indices[storage - 1];
        if (previous_key > sorted_keys[storage]
            || (previous_key == sorted_keys[storage] && previous_sample >= sample)) {
            atomicExch(error_flag, 1);
        }
    }
}

__global__ void gather_immutable_storage(
    const float3* reference,
    const float3* initial_velocity,
    const std::uint8_t* fixed,
    const int* storage_to_sample,
    float3* sorted_reference,
    float3* sorted_initial_velocity,
    std::uint8_t* sorted_fixed,
    int count) {
    const int storage = blockIdx.x * blockDim.x + threadIdx.x;
    if (storage >= count) {
        return;
    }
    const int sample = storage_to_sample[storage];
    sorted_reference[storage] = reference[sample];
    sorted_initial_velocity[storage] = initial_velocity[sample];
    sorted_fixed[storage] = fixed[sample];
}

template <bool Fill>
__global__ void visit_grid_neighbors_cell_sorted(
    const float3* logical_position,
    const int* sorted_indices,
    const int* storage_to_sample,
    const int* sample_to_storage,
    const int* cell_start,
    const int* cell_end,
    int* counts,
    const int* offsets,
    int* neighbors,
    int count,
    float3 origin,
    float horizon,
    int3 dimensions,
    int* error_flag) {
    const int storage_owner = blockIdx.x * blockDim.x + threadIdx.x;
    if (storage_owner >= count) {
        return;
    }
    const int sample_owner = storage_to_sample[storage_owner];
    const float3 point = logical_position[sample_owner];
    const int own_key = cell_key(point, origin, horizon, dimensions);
    const int own_x = own_key % dimensions.x;
    const int own_y = (own_key / dimensions.x) % dimensions.y;
    const int own_z = own_key / (dimensions.x * dimensions.y);
    int cursor = Fill ? offsets[storage_owner] : 0;
    int degree = 0;
    const float support_squared = horizon * horizon * (1.0F + 2.0e-6F);
    for (int dz = -1; dz <= 1; ++dz) {
        const int z = own_z + dz;
        if (z < 0 || z >= dimensions.z) {
            continue;
        }
        for (int dy = -1; dy <= 1; ++dy) {
            const int y = own_y + dy;
            if (y < 0 || y >= dimensions.y) {
                continue;
            }
            for (int dx = -1; dx <= 1; ++dx) {
                const int x = own_x + dx;
                if (x < 0 || x >= dimensions.x) {
                    continue;
                }
                const int key = x + dimensions.x * (y + dimensions.y * z);
                const int begin = cell_start[key];
                if (begin < 0) {
                    continue;
                }
                for (int slot = begin; slot < cell_end[key]; ++slot) {
                    const int candidate_sample = sorted_indices[slot];
                    const float3 delta =
                        subtract3(point, logical_position[candidate_sample]);
                    if (dot3(delta, delta) <= support_squared) {
                        if constexpr (Fill) {
                            if (cursor < offsets[storage_owner + 1]) {
                                neighbors[cursor++] = sample_to_storage[candidate_sample];
                            } else {
                                atomicExch(error_flag, 1);
                            }
                        }
                        ++degree;
                    }
                }
            }
        }
    }
    if constexpr (!Fill) {
        counts[storage_owner] = degree;
    }
}

__global__ void finish_offsets(const int* counts, int* offsets, int count) {
    if (blockIdx.x == 0 && threadIdx.x == 0) {
        offsets[count] = count == 0 ? 0 : offsets[count - 1] + counts[count - 1];
    }
}

__global__ void clamp_neighbor_offsets(
    int* offsets,
    int count,
    int capacity,
    int* error_flag) {
    const int index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index > count) {
        return;
    }
    if (offsets[index] > capacity) {
        offsets[index] = capacity;
        atomicExch(error_flag, 1);
    }
}

struct PairFragment {
    float source[3];
    float matrix[9];
};

static_assert(sizeof(PairFragment) == 48, "O3 fragment accounting requires 48-byte fragments");

__device__ void clear_fragment(PairFragment* fragments, int slot) {
    for (int component = 0; component < 3; ++component) {
        fragments[slot].source[component] = 0.0F;
    }
    for (int component = 0; component < 9; ++component) {
        fragments[slot].matrix[component] = 0.0F;
    }
}

__device__ void store_fragment(
    PairFragment* fragments,
    int slot,
    float3 source,
    const float* matrix) {
    fragments[slot].source[0] = source.x;
    fragments[slot].source[1] = source.y;
    fragments[slot].source[2] = source.z;
    for (int component = 0; component < 9; ++component) {
        fragments[slot].matrix[component] = matrix[component];
    }
}

__global__ void initialize_reverse_slots(
    const int* offsets,
    const int* neighbors,
    int* reverse_slots,
    int* error_flag,
    int count) {
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }
    for (int slot = offsets[particle]; slot < offsets[particle + 1]; ++slot) {
        const int neighbor = neighbors[slot];
        if (neighbor < 0 || neighbor >= count) {
            atomicExch(error_flag, 1);
            continue;
        }
        int reverse = -1;
        int matches = 0;
        for (int candidate = offsets[neighbor]; candidate < offsets[neighbor + 1]; ++candidate) {
            if (neighbors[candidate] == particle) {
                reverse = candidate;
                ++matches;
            }
        }
        if (matches != 1) {
            atomicExch(error_flag, 1);
        }
        reverse_slots[slot] = reverse;
    }
}

__global__ void validate_reverse_slots(
    const int* offsets,
    const int* neighbors,
    const int* reverse_slots,
    int* error_flag,
    int count) {
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }
    for (int slot = offsets[particle]; slot < offsets[particle + 1]; ++slot) {
        const int neighbor = neighbors[slot];
        if (neighbor < 0 || neighbor >= count) {
            atomicExch(error_flag, 1);
            continue;
        }
        const int reverse = reverse_slots[slot];
        if (reverse < offsets[neighbor] || reverse >= offsets[neighbor + 1]
            || neighbors[reverse] != particle || reverse_slots[reverse] != slot) {
            atomicExch(error_flag, 1);
        }
    }
}

__global__ void reduce_segmented_fragments(
    const int* offsets,
    const PairFragment* fragments,
    float* source,
    float* matrix,
    int count) {
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }
    float3 local_source = make_float3(0.0F, 0.0F, 0.0F);
    float local_matrix[9] = {};
    for (int slot = offsets[particle]; slot < offsets[particle + 1]; ++slot) {
        local_source.x += fragments[slot].source[0];
        local_source.y += fragments[slot].source[1];
        local_source.z += fragments[slot].source[2];
        for (int component = 0; component < 9; ++component) {
            local_matrix[component] += fragments[slot].matrix[component];
        }
    }
    source[3 * particle] += local_source.x;
    source[3 * particle + 1] += local_source.y;
    source[3 * particle + 2] += local_source.z;
    for (int component = 0; component < 9; ++component) {
        matrix[9 * particle + component] += local_matrix[component];
    }
}

__global__ void predict_positions(
    const float3* reference,
    const float3* velocity,
    float3* predicted,
    float3* current,
    int count,
    float3 gravity,
    float time_step) {
    const int index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= count) {
        return;
    }
    // Paper Eq. (4), independently implemented from SIUFS_PredictVelocity.
    const float3 predicted_velocity = add3(velocity[index], multiply3(time_step, gravity));
    predicted[index] = add3(reference[index], multiply3(time_step, predicted_velocity));
    current[index] = predicted[index];
}

template <typename NeighborIndex>
__global__ void compute_density(
    const float3* position,
    const int* offsets,
    const NeighborIndex* neighbors,
    float* density,
    int count,
    float mass,
    float horizon,
    float scale) {
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }
    float total = 0.0F;
    for (int slot = offsets[particle]; slot < offsets[particle + 1]; ++slot) {
        const int neighbor = static_cast<int>(neighbors[slot]);
        total += mass * device_cubic_weight(
            length3(subtract3(position[particle], position[neighbor])), horizon, scale);
    }
    density[particle] = total;
}

__device__ void atomic_add_vec(float* target, int particle, float3 value) {
    atomicAdd(&target[3 * particle], value.x);
    atomicAdd(&target[3 * particle + 1], value.y);
    atomicAdd(&target[3 * particle + 2], value.z);
}

__device__ void atomic_add_diagonal(float* matrix, int particle, float value) {
    atomicAdd(&matrix[9 * particle], value);
    atomicAdd(&matrix[9 * particle + 4], value);
    atomicAdd(&matrix[9 * particle + 8], value);
}

__global__ void accumulate_density_term(
    const float3* current,
    const float* density,
    const int* offsets,
    const int* neighbors,
    float* source,
    float* matrix,
    int count,
    float rest_density,
    float kappa,
    float horizon,
    float time_step,
    float scale) {
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }
    // Paper Eqs. (7, 26), independently transcribed from the pinned source.
    const float ratio = fmaxf(density[particle], rest_density) / rest_density;
    const float coefficient = kappa * time_step * time_step / rest_density;
    float3 local_source = make_float3(0.0F, 0.0F, 0.0F);
    float local_diagonal = 0.0F;
    const float3 own = current[particle];
    for (int slot = offsets[particle]; slot < offsets[particle + 1]; ++slot) {
        const int neighbor = neighbors[slot];
        const float3 other = current[neighbor];
        const float radius = length3(subtract3(own, other));
        if (radius <= CUDA_PAIR_EPSILON) {
            continue;
        }
        const float a = coefficient * device_cubic_gradient(radius, horizon, scale) / radius;
        const float diagonal = -a;
        const float3 local = add3(
            multiply3(-a, other), multiply3(ratio * a, subtract3(other, own)));
        const float3 reverse = add3(
            multiply3(-a, own), multiply3(ratio * a, subtract3(own, other)));
        local_source = add3(local_source, local);
        local_diagonal += diagonal;
        atomic_add_vec(source, neighbor, reverse);
        atomic_add_diagonal(matrix, neighbor, diagonal);
    }
    atomic_add_vec(source, particle, local_source);
    atomic_add_diagonal(matrix, particle, local_diagonal);
}

__global__ void accumulate_density_term_gather_directed(
    const float3* current,
    const float* density,
    const int* offsets,
    const int* neighbors,
    float* source,
    float* matrix,
    int count,
    float rest_density,
    float kappa,
    float horizon,
    float time_step,
    float scale) {
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }
    // NR1-RC1: one thread owns particle i and reconstructs both directed
    // endpoint contributions in the frozen CSR order. The incoming reverse
    // term is reverse(j,i), so its density ratio belongs to j.
    const float own_ratio = fmaxf(density[particle], rest_density) / rest_density;
    const float coefficient = kappa * time_step * time_step / rest_density;
    const float3 own = current[particle];
    float3 local_source = make_float3(0.0F, 0.0F, 0.0F);
    float local_diagonal = 0.0F;
    for (int slot = offsets[particle]; slot < offsets[particle + 1]; ++slot) {
        const int neighbor = neighbors[slot];
        const float3 other = current[neighbor];
        const float radius = length3(subtract3(own, other));
        if (radius <= CUDA_PAIR_EPSILON) {
            continue;
        }
        const float neighbor_ratio = fmaxf(density[neighbor], rest_density) / rest_density;
        const float a = coefficient * device_cubic_gradient(radius, horizon, scale) / radius;
        const float diagonal = -a;
        const float3 local = add3(
            multiply3(-a, other), multiply3(own_ratio * a, subtract3(other, own)));
        const float3 incoming_reverse = add3(
            multiply3(-a, other), multiply3(neighbor_ratio * a, subtract3(other, own)));
        local_source = add3(local_source, local);
        local_source = add3(local_source, incoming_reverse);
        local_diagonal += diagonal;
        local_diagonal += diagonal;
    }
    source[3 * particle] += local_source.x;
    source[3 * particle + 1] += local_source.y;
    source[3 * particle + 2] += local_source.z;
    matrix[9 * particle] += local_diagonal;
    matrix[9 * particle + 4] += local_diagonal;
    matrix[9 * particle + 8] += local_diagonal;
}

__global__ void accumulate_density_term_unique_pairs(
    const float3* current,
    const float* density,
    const int* offsets,
    const int* neighbors,
    const int* reverse_slots,
    PairFragment* fragments,
    int count,
    float rest_density,
    float kappa,
    float horizon,
    float time_step,
    float scale) {
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }
    const float own_ratio = fmaxf(density[particle], rest_density) / rest_density;
    const float coefficient = kappa * time_step * time_step / rest_density;
    const float3 own = current[particle];
    for (int slot = offsets[particle]; slot < offsets[particle + 1]; ++slot) {
        const int neighbor = neighbors[slot];
        if (neighbor == particle) {
            clear_fragment(fragments, slot);
            continue;
        }
        if (particle > neighbor) {
            continue;
        }
        const int reverse = reverse_slots[slot];
        const float3 other = current[neighbor];
        const float radius = length3(subtract3(own, other));
        if (radius <= CUDA_PAIR_EPSILON) {
            clear_fragment(fragments, slot);
            clear_fragment(fragments, reverse);
            continue;
        }
        const float neighbor_ratio = fmaxf(density[neighbor], rest_density) / rest_density;
        const float a = coefficient * device_cubic_gradient(radius, horizon, scale) / radius;
        const float diagonal = -a;
        const float3 other_minus_own = subtract3(other, own);
        const float3 own_local = add3(
            multiply3(-a, other), multiply3(own_ratio * a, other_minus_own));
        const float3 own_incoming = add3(
            multiply3(-a, other), multiply3(neighbor_ratio * a, other_minus_own));
        const float3 own_fragment = add3(own_local, own_incoming);
        const float3 own_minus_other = subtract3(own, other);
        const float3 other_local = add3(
            multiply3(-a, own), multiply3(neighbor_ratio * a, own_minus_other));
        const float3 other_incoming = add3(
            multiply3(-a, own), multiply3(own_ratio * a, own_minus_other));
        const float3 other_fragment = add3(other_local, other_incoming);
        float pair_matrix[9] = {};
        pair_matrix[0] = diagonal + diagonal;
        pair_matrix[4] = diagonal + diagonal;
        pair_matrix[8] = diagonal + diagonal;
        store_fragment(fragments, slot, own_fragment, pair_matrix);
        store_fragment(fragments, reverse, other_fragment, pair_matrix);
    }
}

__device__ void build_normal_tangent(float3 direction, float* normal, float* tangent) {
    const float components[3] = {direction.x, direction.y, direction.z};
    for (int row = 0; row < 3; ++row) {
        for (int column = 0; column < 3; ++column) {
            const int index = 3 * row + column;
            normal[index] = components[row] * components[column];
            tangent[index] = (row == column ? 1.0F : 0.0F) - normal[index];
        }
    }
}

__device__ float3 matrix_vector(const float* matrix, float3 vector) {
    return make_float3(
        matrix[0] * vector.x + matrix[1] * vector.y + matrix[2] * vector.z,
        matrix[3] * vector.x + matrix[4] * vector.y + matrix[5] * vector.z,
        matrix[6] * vector.x + matrix[7] * vector.y + matrix[8] * vector.z);
}

__global__ void accumulate_viscosity_term(
    const float3* reference,
    const float3* current,
    const int* offsets,
    const int* neighbors,
    float* source,
    float* matrix,
    int count,
    float rest_density,
    float lambda,
    float mu,
    float horizon,
    float time_step,
    float scale,
    bool bulk_enabled,
    bool shear_enabled) {
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }
    // Paper Eqs. (10, 26), independently transcribed from the pinned source.
    const float alpha = bulk_enabled ? lambda * time_step / rest_density : 0.0F;
    const float beta = shear_enabled ? mu * time_step / rest_density : 0.0F;
    float3 local_source = make_float3(0.0F, 0.0F, 0.0F);
    float local_matrix[9] = {};
    for (int slot = offsets[particle]; slot < offsets[particle + 1]; ++slot) {
        const int neighbor = neighbors[slot];
        const float3 reference_delta = subtract3(reference[neighbor], reference[particle]);
        const float radius = length3(reference_delta);
        if (radius <= CUDA_PAIR_EPSILON) {
            continue;
        }
        const float3 direction = multiply3(1.0F / radius, reference_delta);
        float normal[9];
        float tangent[9];
        build_normal_tangent(direction, normal, tangent);
        const float weight = device_cubic_weight(radius, horizon, scale);
        float pair_matrix[9];
        for (int component = 0; component < 9; ++component) {
            pair_matrix[component] =
                alpha * weight * normal[component] + beta * weight * tangent[component];
            local_matrix[component] += pair_matrix[component];
            atomicAdd(&matrix[9 * neighbor + component], pair_matrix[component]);
        }
        const float3 local = subtract3(
            matrix_vector(pair_matrix, current[neighbor]),
            matrix_vector(pair_matrix, reference_delta));
        const float3 reverse = add3(
            matrix_vector(pair_matrix, current[particle]),
            matrix_vector(pair_matrix, reference_delta));
        local_source = add3(local_source, local);
        atomic_add_vec(source, neighbor, reverse);
    }
    atomic_add_vec(source, particle, local_source);
    for (int component = 0; component < 9; ++component) {
        atomicAdd(&matrix[9 * particle + component], local_matrix[component]);
    }
}

__global__ void accumulate_viscosity_term_gather_directed(
    const float3* reference,
    const float3* current,
    const int* offsets,
    const int* neighbors,
    float* source,
    float* matrix,
    int count,
    float rest_density,
    float lambda,
    float mu,
    float horizon,
    float time_step,
    float scale,
    bool bulk_enabled,
    bool shear_enabled) {
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }
    // NR1-RC1 independent CUDA transcription of the owner-only Eq. (10)
    // reconstruction. Keep the two named endpoint evaluations explicit.
    const float alpha = bulk_enabled ? lambda * time_step / rest_density : 0.0F;
    const float beta = shear_enabled ? mu * time_step / rest_density : 0.0F;
    float3 local_source = make_float3(0.0F, 0.0F, 0.0F);
    float local_matrix[9] = {};
    for (int slot = offsets[particle]; slot < offsets[particle + 1]; ++slot) {
        const int neighbor = neighbors[slot];
        const float3 reference_delta = subtract3(reference[neighbor], reference[particle]);
        const float radius = length3(reference_delta);
        if (radius <= CUDA_PAIR_EPSILON) {
            continue;
        }
        const float3 direction = multiply3(1.0F / radius, reference_delta);
        float normal[9];
        float tangent[9];
        build_normal_tangent(direction, normal, tangent);
        const float weight = device_cubic_weight(radius, horizon, scale);
        float pair_matrix[9];
        for (int component = 0; component < 9; ++component) {
            pair_matrix[component] =
                alpha * weight * normal[component] + beta * weight * tangent[component];
            local_matrix[component] += pair_matrix[component];
            local_matrix[component] += pair_matrix[component];
        }
        const float3 local = subtract3(
            matrix_vector(pair_matrix, current[neighbor]),
            matrix_vector(pair_matrix, reference_delta));
        const float3 incoming_reference_delta =
            subtract3(reference[particle], reference[neighbor]);
        const float3 incoming_reverse = add3(
            matrix_vector(pair_matrix, current[neighbor]),
            matrix_vector(pair_matrix, incoming_reference_delta));
        local_source = add3(local_source, local);
        local_source = add3(local_source, incoming_reverse);
    }
    source[3 * particle] += local_source.x;
    source[3 * particle + 1] += local_source.y;
    source[3 * particle + 2] += local_source.z;
    for (int component = 0; component < 9; ++component) {
        matrix[9 * particle + component] += local_matrix[component];
    }
}

template <bool BulkEnabled, bool ShearEnabled>
__global__ void accumulate_viscosity_term_gather_specialized(
    const float3* reference,
    const float3* current,
    const int* offsets,
    const int* neighbors,
    float* source,
    float* matrix,
    int count,
    float rest_density,
    float lambda,
    float mu,
    float horizon,
    float time_step,
    float scale) {
    static_assert(BulkEnabled || ShearEnabled, "inactive viscosity has no kernel launch");
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }
    // NR2-O2 closes the viscosity mask at compilation. The combined variant
    // deliberately preserves the runtime kernel's expression order.
    const float alpha = BulkEnabled ? lambda * time_step / rest_density : 0.0F;
    const float beta = ShearEnabled ? mu * time_step / rest_density : 0.0F;
    float3 local_source = make_float3(0.0F, 0.0F, 0.0F);
    float local_matrix[9] = {};
    for (int slot = offsets[particle]; slot < offsets[particle + 1]; ++slot) {
        const int neighbor = neighbors[slot];
        const float3 reference_delta = subtract3(reference[neighbor], reference[particle]);
        const float radius = length3(reference_delta);
        if (radius <= CUDA_PAIR_EPSILON) {
            continue;
        }
        const float3 direction = multiply3(1.0F / radius, reference_delta);
        const float components[3] = {direction.x, direction.y, direction.z};
        const float weight = device_cubic_weight(radius, horizon, scale);
        float pair_matrix[9];
        for (int row = 0; row < 3; ++row) {
            for (int column = 0; column < 3; ++column) {
                const int component = 3 * row + column;
                const float normal = components[row] * components[column];
                if constexpr (BulkEnabled && ShearEnabled) {
                    const float tangent = (row == column ? 1.0F : 0.0F) - normal;
                    pair_matrix[component] =
                        alpha * weight * normal + beta * weight * tangent;
                } else if constexpr (BulkEnabled) {
                    pair_matrix[component] = alpha * weight * normal;
                } else {
                    const float tangent = (row == column ? 1.0F : 0.0F) - normal;
                    pair_matrix[component] = beta * weight * tangent;
                }
                local_matrix[component] += pair_matrix[component];
                local_matrix[component] += pair_matrix[component];
            }
        }
        const float3 local = subtract3(
            matrix_vector(pair_matrix, current[neighbor]),
            matrix_vector(pair_matrix, reference_delta));
        const float3 incoming_reference_delta =
            subtract3(reference[particle], reference[neighbor]);
        const float3 incoming_reverse = add3(
            matrix_vector(pair_matrix, current[neighbor]),
            matrix_vector(pair_matrix, incoming_reference_delta));
        local_source = add3(local_source, local);
        local_source = add3(local_source, incoming_reverse);
    }
    source[3 * particle] += local_source.x;
    source[3 * particle + 1] += local_source.y;
    source[3 * particle + 2] += local_source.z;
    for (int component = 0; component < 9; ++component) {
        matrix[9 * particle + component] += local_matrix[component];
    }
}

template <bool BulkEnabled, bool ShearEnabled>
__global__ void accumulate_viscosity_term_unique_pairs_specialized(
    const float3* reference,
    const float3* current,
    const int* offsets,
    const int* neighbors,
    const int* reverse_slots,
    PairFragment* fragments,
    int count,
    float rest_density,
    float lambda,
    float mu,
    float horizon,
    float time_step,
    float scale) {
    static_assert(BulkEnabled || ShearEnabled, "inactive viscosity has no kernel launch");
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }
    const float alpha = BulkEnabled ? lambda * time_step / rest_density : 0.0F;
    const float beta = ShearEnabled ? mu * time_step / rest_density : 0.0F;
    for (int slot = offsets[particle]; slot < offsets[particle + 1]; ++slot) {
        const int neighbor = neighbors[slot];
        if (neighbor == particle) {
            clear_fragment(fragments, slot);
            continue;
        }
        if (particle > neighbor) {
            continue;
        }
        const int reverse = reverse_slots[slot];
        const float3 reference_delta = subtract3(reference[neighbor], reference[particle]);
        const float radius = length3(reference_delta);
        if (radius <= CUDA_PAIR_EPSILON) {
            clear_fragment(fragments, slot);
            clear_fragment(fragments, reverse);
            continue;
        }
        const float3 direction = multiply3(1.0F / radius, reference_delta);
        const float components[3] = {direction.x, direction.y, direction.z};
        const float weight = device_cubic_weight(radius, horizon, scale);
        float pair_matrix[9];
        float endpoint_matrix[9];
        for (int row = 0; row < 3; ++row) {
            for (int column = 0; column < 3; ++column) {
                const int component = 3 * row + column;
                const float normal = components[row] * components[column];
                if constexpr (BulkEnabled && ShearEnabled) {
                    const float tangent = (row == column ? 1.0F : 0.0F) - normal;
                    pair_matrix[component] =
                        alpha * weight * normal + beta * weight * tangent;
                } else if constexpr (BulkEnabled) {
                    pair_matrix[component] = alpha * weight * normal;
                } else {
                    const float tangent = (row == column ? 1.0F : 0.0F) - normal;
                    pair_matrix[component] = beta * weight * tangent;
                }
                endpoint_matrix[component] = pair_matrix[component] + pair_matrix[component];
            }
        }
        const float3 own_local = subtract3(
            matrix_vector(pair_matrix, current[neighbor]),
            matrix_vector(pair_matrix, reference_delta));
        const float3 reverse_reference_delta =
            subtract3(reference[particle], reference[neighbor]);
        const float3 own_incoming = add3(
            matrix_vector(pair_matrix, current[neighbor]),
            matrix_vector(pair_matrix, reverse_reference_delta));
        const float3 own_fragment = add3(own_local, own_incoming);
        const float3 other_local = subtract3(
            matrix_vector(pair_matrix, current[particle]),
            matrix_vector(pair_matrix, reverse_reference_delta));
        const float3 other_incoming = add3(
            matrix_vector(pair_matrix, current[particle]),
            matrix_vector(pair_matrix, reference_delta));
        const float3 other_fragment = add3(other_local, other_incoming);
        store_fragment(fragments, slot, own_fragment, endpoint_matrix);
        store_fragment(fragments, reverse, other_fragment, endpoint_matrix);
    }
}

__device__ float surface_positive_cuda(float radius, float rest_spacing) {
    const float q = radius / rest_spacing;
    if (q <= 1.0F) {
        return q * q;
    }
    if (q <= 3.0F) {
        return 1.0F - (q - 2.0F) * (q - 2.0F);
    }
    return 0.0F;
}

__device__ float surface_negative_cuda(float radius, float rest_spacing) {
    return radius / rest_spacing <= 1.0F ? -1.0F : 0.0F;
}

__device__ float surface_potential_cuda(float radius, float rest_spacing) {
    const float q = radius / rest_spacing;
    if (q <= 1.0F) {
        return q * q * q / 3.0F - q + 2.0F / 3.0F;
    }
    if (q <= 3.0F) {
        const float shifted = q - 2.0F;
        return q - shifted * shifted * shifted / 3.0F - 4.0F / 3.0F;
    }
    return 0.0F;
}

__global__ void accumulate_surface_term(
    const float3* current,
    const int* offsets,
    const int* neighbors,
    float* source,
    float* matrix,
    int count,
    float gamma,
    float rest_spacing,
    float time_step) {
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }
    // Paper Eqs. (14, 26), default bidirectional pair function.
    const float coefficient = gamma * time_step * time_step;
    const float3 own = current[particle];
    float3 local_source = make_float3(0.0F, 0.0F, 0.0F);
    float local_diagonal = 0.0F;
    for (int slot = offsets[particle]; slot < offsets[particle + 1]; ++slot) {
        const int neighbor = neighbors[slot];
        const float3 other = current[neighbor];
        const float radius = length3(subtract3(own, other));
        if (radius <= CUDA_PAIR_EPSILON) {
            continue;
        }
        const float positive = surface_positive_cuda(radius, rest_spacing);
        const float negative = surface_negative_cuda(radius, rest_spacing);
        const float diagonal = coefficient * positive / radius;
        const float3 local = add3(
            multiply3(diagonal, other),
            multiply3(coefficient * negative / radius, subtract3(other, own)));
        const float3 reverse = add3(
            multiply3(diagonal, own),
            multiply3(coefficient * negative / radius, subtract3(own, other)));
        local_source = add3(local_source, local);
        local_diagonal += diagonal;
        atomic_add_vec(source, neighbor, reverse);
        atomic_add_diagonal(matrix, neighbor, diagonal);
    }
    atomic_add_vec(source, particle, local_source);
    atomic_add_diagonal(matrix, particle, local_diagonal);
}

__global__ void accumulate_surface_term_gather_directed(
    const float3* current,
    const int* offsets,
    const int* neighbors,
    float* source,
    float* matrix,
    int count,
    float gamma,
    float rest_spacing,
    float time_step) {
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }
    // NR1-RC1 owner-only Eq. (14) reconstruction. The local and incoming
    // reverse additions intentionally remain separate rather than multiplied.
    const float coefficient = gamma * time_step * time_step;
    const float3 own = current[particle];
    float3 local_source = make_float3(0.0F, 0.0F, 0.0F);
    float local_diagonal = 0.0F;
    for (int slot = offsets[particle]; slot < offsets[particle + 1]; ++slot) {
        const int neighbor = neighbors[slot];
        const float3 other = current[neighbor];
        const float radius = length3(subtract3(own, other));
        if (radius <= CUDA_PAIR_EPSILON) {
            continue;
        }
        const float positive = surface_positive_cuda(radius, rest_spacing);
        const float negative = surface_negative_cuda(radius, rest_spacing);
        const float diagonal = coefficient * positive / radius;
        const float signed_coefficient = coefficient * negative / radius;
        const float3 local = add3(
            multiply3(diagonal, other),
            multiply3(signed_coefficient, subtract3(other, own)));
        const float3 incoming_reverse = add3(
            multiply3(diagonal, other),
            multiply3(signed_coefficient, subtract3(other, own)));
        local_source = add3(local_source, local);
        local_source = add3(local_source, incoming_reverse);
        local_diagonal += diagonal;
        local_diagonal += diagonal;
    }
    source[3 * particle] += local_source.x;
    source[3 * particle + 1] += local_source.y;
    source[3 * particle + 2] += local_source.z;
    matrix[9 * particle] += local_diagonal;
    matrix[9 * particle + 4] += local_diagonal;
    matrix[9 * particle + 8] += local_diagonal;
}

template <typename NeighborIndex, bool BulkEnabled, bool ShearEnabled, bool SurfaceEnabled>
__global__ void accumulate_fused_owner_terms_p1(
    const float3* reference,
    const float3* current,
    const float* density,
    const int* offsets,
    const NeighborIndex* neighbors,
    float* source,
    float* matrix,
    int count,
    float rest_density,
    float kappa,
    float lambda,
    float mu,
    float gamma,
    float spacing,
    float horizon,
    float time_step,
    float scale) {
    static_assert(
        BulkEnabled || ShearEnabled || SurfaceEnabled,
        "P1 fusion requires a second active post-density term");
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }

    const float density_ratio = fmaxf(density[particle], rest_density) / rest_density;
    const float density_coefficient = kappa * time_step * time_step / rest_density;
    const float viscosity_alpha = BulkEnabled ? lambda * time_step / rest_density : 0.0F;
    const float viscosity_beta = ShearEnabled ? mu * time_step / rest_density : 0.0F;
    const float surface_coefficient = SurfaceEnabled ? gamma * time_step * time_step : 0.0F;
    const float3 own = current[particle];

    float3 density_source = make_float3(0.0F, 0.0F, 0.0F);
    float density_diagonal = 0.0F;
    float3 viscosity_source = make_float3(0.0F, 0.0F, 0.0F);
    float viscosity_matrix[9] = {};
    float3 surface_source = make_float3(0.0F, 0.0F, 0.0F);
    float surface_diagonal = 0.0F;

    for (int slot = offsets[particle]; slot < offsets[particle + 1]; ++slot) {
        const int neighbor = static_cast<int>(neighbors[slot]);
        const float3 other = current[neighbor];

        // Retained incompressibility owner association and add order.
        const float density_radius = length3(subtract3(own, other));
        if (density_radius > CUDA_PAIR_EPSILON) {
            const float neighbor_ratio =
                fmaxf(density[neighbor], rest_density) / rest_density;
            const float a = density_coefficient
                * device_cubic_gradient(density_radius, horizon, scale) / density_radius;
            const float diagonal = -a;
            const float3 local = add3(
                multiply3(-a, other),
                multiply3(density_ratio * a, subtract3(other, own)));
            const float3 incoming_reverse = add3(
                multiply3(-a, other),
                multiply3(neighbor_ratio * a, subtract3(other, own)));
            density_source = add3(density_source, local);
            density_source = add3(density_source, incoming_reverse);
            density_diagonal += diagonal;
            density_diagonal += diagonal;
        }

        if constexpr (BulkEnabled || ShearEnabled) {
            // Retained O2 specialized viscosity association and add order.
            const float3 reference_delta =
                subtract3(reference[neighbor], reference[particle]);
            const float viscosity_radius = length3(reference_delta);
            if (viscosity_radius > CUDA_PAIR_EPSILON) {
                const float3 direction = multiply3(1.0F / viscosity_radius, reference_delta);
                const float components[3] = {direction.x, direction.y, direction.z};
                const float weight = device_cubic_weight(viscosity_radius, horizon, scale);
                float pair_matrix[9];
                for (int row = 0; row < 3; ++row) {
                    for (int column = 0; column < 3; ++column) {
                        const int component = 3 * row + column;
                        const float normal = components[row] * components[column];
                        if constexpr (BulkEnabled && ShearEnabled) {
                            const float tangent =
                                (row == column ? 1.0F : 0.0F) - normal;
                            pair_matrix[component] = viscosity_alpha * weight * normal
                                + viscosity_beta * weight * tangent;
                        } else if constexpr (BulkEnabled) {
                            pair_matrix[component] = viscosity_alpha * weight * normal;
                        } else {
                            const float tangent =
                                (row == column ? 1.0F : 0.0F) - normal;
                            pair_matrix[component] = viscosity_beta * weight * tangent;
                        }
                        viscosity_matrix[component] += pair_matrix[component];
                        viscosity_matrix[component] += pair_matrix[component];
                    }
                }
                const float3 local = subtract3(
                    matrix_vector(pair_matrix, current[neighbor]),
                    matrix_vector(pair_matrix, reference_delta));
                const float3 incoming_reference_delta =
                    subtract3(reference[particle], reference[neighbor]);
                const float3 incoming_reverse = add3(
                    matrix_vector(pair_matrix, current[neighbor]),
                    matrix_vector(pair_matrix, incoming_reference_delta));
                viscosity_source = add3(viscosity_source, local);
                viscosity_source = add3(viscosity_source, incoming_reverse);
            }
        }

        if constexpr (SurfaceEnabled) {
            // Retained surface owner association and add order.
            const float surface_radius = length3(subtract3(own, other));
            if (surface_radius > CUDA_PAIR_EPSILON) {
                const float positive = surface_positive_cuda(surface_radius, spacing);
                const float negative = surface_negative_cuda(surface_radius, spacing);
                const float diagonal = surface_coefficient * positive / surface_radius;
                const float signed_coefficient =
                    surface_coefficient * negative / surface_radius;
                const float3 local = add3(
                    multiply3(diagonal, other),
                    multiply3(signed_coefficient, subtract3(other, own)));
                const float3 incoming_reverse = add3(
                    multiply3(diagonal, other),
                    multiply3(signed_coefficient, subtract3(other, own)));
                surface_source = add3(surface_source, local);
                surface_source = add3(surface_source, incoming_reverse);
                surface_diagonal += diagonal;
                surface_diagonal += diagonal;
            }
        }
    }

    // Preserve the retained cross-term commit order and f32 roundings.
    source[3 * particle] += density_source.x;
    source[3 * particle + 1] += density_source.y;
    source[3 * particle + 2] += density_source.z;
    matrix[9 * particle] += density_diagonal;
    matrix[9 * particle + 4] += density_diagonal;
    matrix[9 * particle + 8] += density_diagonal;
    if constexpr (BulkEnabled || ShearEnabled) {
        source[3 * particle] += viscosity_source.x;
        source[3 * particle + 1] += viscosity_source.y;
        source[3 * particle + 2] += viscosity_source.z;
        for (int component = 0; component < 9; ++component) {
            matrix[9 * particle + component] += viscosity_matrix[component];
        }
    }
    if constexpr (SurfaceEnabled) {
        source[3 * particle] += surface_source.x;
        source[3 * particle + 1] += surface_source.y;
        source[3 * particle + 2] += surface_source.z;
        matrix[9 * particle] += surface_diagonal;
        matrix[9 * particle + 4] += surface_diagonal;
        matrix[9 * particle + 8] += surface_diagonal;
    }
}

__global__ void accumulate_surface_term_unique_pairs(
    const float3* current,
    const int* offsets,
    const int* neighbors,
    const int* reverse_slots,
    PairFragment* fragments,
    int count,
    float gamma,
    float rest_spacing,
    float time_step) {
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }
    const float coefficient = gamma * time_step * time_step;
    const float3 own = current[particle];
    for (int slot = offsets[particle]; slot < offsets[particle + 1]; ++slot) {
        const int neighbor = neighbors[slot];
        if (neighbor == particle) {
            clear_fragment(fragments, slot);
            continue;
        }
        if (particle > neighbor) {
            continue;
        }
        const int reverse = reverse_slots[slot];
        const float3 other = current[neighbor];
        const float radius = length3(subtract3(own, other));
        if (radius <= CUDA_PAIR_EPSILON) {
            clear_fragment(fragments, slot);
            clear_fragment(fragments, reverse);
            continue;
        }
        const float positive = surface_positive_cuda(radius, rest_spacing);
        const float negative = surface_negative_cuda(radius, rest_spacing);
        const float diagonal = coefficient * positive / radius;
        const float signed_coefficient = coefficient * negative / radius;
        const float3 other_minus_own = subtract3(other, own);
        const float3 own_local = add3(
            multiply3(diagonal, other), multiply3(signed_coefficient, other_minus_own));
        const float3 own_incoming = add3(
            multiply3(diagonal, other), multiply3(signed_coefficient, other_minus_own));
        const float3 own_fragment = add3(own_local, own_incoming);
        const float3 own_minus_other = subtract3(own, other);
        const float3 other_local = add3(
            multiply3(diagonal, own), multiply3(signed_coefficient, own_minus_other));
        const float3 other_incoming = add3(
            multiply3(diagonal, own), multiply3(signed_coefficient, own_minus_other));
        const float3 other_fragment = add3(other_local, other_incoming);
        float pair_matrix[9] = {};
        pair_matrix[0] = diagonal + diagonal;
        pair_matrix[4] = diagonal + diagonal;
        pair_matrix[8] = diagonal + diagonal;
        store_fragment(fragments, slot, own_fragment, pair_matrix);
        store_fragment(fragments, reverse, other_fragment, pair_matrix);
    }
}

__device__ bool inverse_matrix_vector(const float* raw, float3 rhs, float3& output) {
    const float a00 = 1.0F + raw[0];
    const float a01 = raw[1];
    const float a02 = raw[2];
    const float a10 = raw[3];
    const float a11 = 1.0F + raw[4];
    const float a12 = raw[5];
    const float a20 = raw[6];
    const float a21 = raw[7];
    const float a22 = 1.0F + raw[8];
    const float determinant = a00 * (a11 * a22 - a12 * a21)
        - a01 * (a10 * a22 - a12 * a20) + a02 * (a10 * a21 - a11 * a20);
    if (!isfinite(determinant) || fabsf(determinant) <= 1.0e-18F) {
        return false;
    }
    const float inverse[9] = {
        (a11 * a22 - a12 * a21) / determinant,
        (a02 * a21 - a01 * a22) / determinant,
        (a01 * a12 - a02 * a11) / determinant,
        (a12 * a20 - a10 * a22) / determinant,
        (a00 * a22 - a02 * a20) / determinant,
        (a02 * a10 - a00 * a12) / determinant,
        (a10 * a21 - a11 * a20) / determinant,
        (a01 * a20 - a00 * a21) / determinant,
        (a00 * a11 - a01 * a10) / determinant,
    };
    output = matrix_vector(inverse, rhs);
    return isfinite(output.x) && isfinite(output.y) && isfinite(output.z);
}

__global__ void update_positions(
    const float3* reference,
    const float3* predicted,
    const std::uint8_t* fixed,
    const float* source,
    const float* matrix,
    float3* next,
    int* error_flag,
    int count) {
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }
    if (fixed[particle] != 0U) {
        next[particle] = reference[particle];
        return;
    }
    const float3 rhs = make_float3(
        predicted[particle].x + source[3 * particle],
        predicted[particle].y + source[3 * particle + 1],
        predicted[particle].z + source[3 * particle + 2]);
    // Paper Eq. (26), with the same unregularized local inverse rule as source.
    if (!inverse_matrix_vector(&matrix[9 * particle], rhs, next[particle])) {
        atomicExch(error_flag, 1);
    }
}

__global__ void reconstruct_velocity(
    const float3* reference,
    const float3* current,
    const std::uint8_t* fixed,
    float3* velocity,
    int count,
    float time_step) {
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }
    velocity[particle] = fixed[particle] != 0U
        ? make_float3(0.0F, 0.0F, 0.0F)
        : multiply3(1.0F / time_step, subtract3(current[particle], reference[particle]));
}

template <typename NeighborIndex>
__global__ void compute_energy_components(
    const float3* reference,
    const float3* predicted,
    const float3* current,
    const float* density,
    const int* offsets,
    const NeighborIndex* neighbors,
    float* energy,
    int count,
    float rest_density,
    float spacing,
    float mass,
    float horizon,
    float time_step,
    float kappa,
    float lambda,
    float mu,
    float gamma,
    float scale,
    bool incompressibility_enabled,
    bool bulk_enabled,
    bool shear_enabled,
    bool surface_enabled) {
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }
    float values[5] = {};
    values[0] = mass * dot3(subtract3(current[particle], predicted[particle]),
                              subtract3(current[particle], predicted[particle]))
        / (2.0F * time_step * time_step);
    if (incompressibility_enabled) {
        const float error = density[particle] / rest_density - 1.0F;
        values[1] = 0.5F * kappa * error * error;
    }
    for (int slot = offsets[particle]; slot < offsets[particle + 1]; ++slot) {
        const int neighbor = static_cast<int>(neighbors[slot]);
        const float3 reference_delta = subtract3(reference[particle], reference[neighbor]);
        const float radius = length3(reference_delta);
        if (radius <= CUDA_PAIR_EPSILON) {
            continue;
        }
        if (bulk_enabled || shear_enabled) {
            const float3 direction = multiply3(1.0F / radius, reference_delta);
            const float3 relative_velocity = multiply3(
                1.0F / time_step,
                subtract3(subtract3(current[particle], current[neighbor]), reference_delta));
            const float3 bulk = multiply3(dot3(relative_velocity, direction), direction);
            const float3 shear = subtract3(relative_velocity, bulk);
            const float weight = device_cubic_weight(radius, horizon, scale);
            if (bulk_enabled) {
                values[2] += mass / rest_density * lambda * dot3(bulk, bulk) * weight / 4.0F;
            }
            if (shear_enabled) {
                values[3] += mass / rest_density * mu * dot3(shear, shear) * weight / 2.0F;
            }
        }
        if (surface_enabled) {
            values[4] += gamma * mass * mass
                * surface_potential_cuda(length3(subtract3(current[particle], current[neighbor])), spacing);
        }
    }
    for (int component = 0; component < 5; ++component) {
        energy[5 * particle + component] = values[component];
    }
}

struct GridDescription {
    float3 origin{};
    int3 dimensions{};
    int cells = 0;
};

struct StageTiming {
    double prediction = 0.0;
    double neighbor_construction = 0.0;
    double density = 0.0;
    double buffer_reset = 0.0;
    double incompressibility = 0.0;
    double viscosity = 0.0;
    double surface_tension = 0.0;
    double fused_owner_terms = 0.0;
    double local_update = 0.0;
    double state_handoff = 0.0;
    double total = 0.0;
};

struct CapturedRun {
    OracleResult state;
    StageTiming timing;
    std::vector<int> offsets;
    std::vector<int> neighbors;
    std::size_t device_memory_bytes = 0;
    std::size_t neighbor_id_bytes = sizeof(int);
    std::size_t layout_memory_bytes = 0;
    std::size_t storage_memory_bytes = 0;
    std::size_t self_slots = 0;
    std::size_t nonself_directed_samples = 0;
    std::size_t unique_pairs = 0;
    std::size_t pair_evaluations_per_active_term = 0;
    std::size_t endpoint_fragments_per_active_term = 0;
    double layout_setup_ms = 0.0;
    double storage_setup_ms = 0.0;
    double mean_neighbor_storage_distance = 0.0;
    double p95_neighbor_storage_distance = 0.0;
    bool local_solve_failed = false;
    bool reverse_map_valid = true;
    bool storage_map_valid = true;
    bool neighbor_build_valid = true;
    std::string storage_to_sample_sha256;
    std::string sample_to_storage_sha256;
    std::string physical_csr_sha256;
};

struct EventInterval {
    enum class Stage {
        Prediction,
        Neighbor,
        Density,
        Reset,
        Incompressibility,
        Viscosity,
        Surface,
        FusedOwnerTerms,
        Update,
        Handoff,
    };

    Stage stage;
    cudaEvent_t begin{};
    cudaEvent_t end{};
};

GridDescription describe_grid(const Fixture& fixture) {
    Vec3 minimum = fixture.particles.front().position;
    Vec3 maximum = minimum;
    for (const Particle& particle : fixture.particles) {
        minimum.x = std::min(minimum.x, particle.position.x);
        minimum.y = std::min(minimum.y, particle.position.y);
        minimum.z = std::min(minimum.z, particle.position.z);
        maximum.x = std::max(maximum.x, particle.position.x);
        maximum.y = std::max(maximum.y, particle.position.y);
        maximum.z = std::max(maximum.z, particle.position.z);
    }
    minimum.x -= fixture.grid_margin;
    minimum.y -= fixture.grid_margin;
    minimum.z -= fixture.grid_margin;
    maximum.x += fixture.grid_margin;
    maximum.y += fixture.grid_margin;
    maximum.z += fixture.grid_margin;
    GridDescription grid;
    grid.origin = make_float3(
        static_cast<float>(minimum.x - fixture.horizon),
        static_cast<float>(minimum.y - fixture.horizon),
        static_cast<float>(minimum.z - fixture.horizon));
    const auto dimension = [&](double low, double high) {
        return std::max(3, static_cast<int>(std::floor((high - low) / fixture.horizon)) + 3);
    };
    grid.dimensions = make_int3(
        dimension(minimum.x, maximum.x),
        dimension(minimum.y, maximum.y),
        dimension(minimum.z, maximum.z));
    const std::int64_t cells = static_cast<std::int64_t>(grid.dimensions.x)
        * static_cast<std::int64_t>(grid.dimensions.y)
        * static_cast<std::int64_t>(grid.dimensions.z);
    if (cells <= 0 || cells > std::numeric_limits<int>::max()) {
        throw std::runtime_error("neighbor grid packed-key range exceeds signed i32");
    }
    grid.cells = static_cast<int>(cells);
    return grid;
}

float host_cuda_kernel_weight(float radius, float horizon) {
    const float q = 2.0F * radius / horizon;
    const float alpha = 3.0F / (2.0F * CUDA_PI * horizon * horizon * horizon);
    if (q > 2.0F) {
        return 0.0F;
    }
    if (q >= 1.0F) {
        const float delta = 2.0F - q;
        return alpha * delta * delta * delta / 6.0F;
    }
    return alpha * (2.0F / 3.0F - q * q + 0.5F * q * q * q);
}

float host_cuda_kernel_scale(float spacing, float horizon) {
    const int half_resolution = static_cast<int>(horizon / spacing + 1.0F);
    const float volume = spacing * spacing * spacing;
    float total = 0.0F;
    for (int z = -half_resolution; z <= half_resolution; ++z) {
        for (int y = -half_resolution; y <= half_resolution; ++y) {
            for (int x = -half_resolution; x <= half_resolution; ++x) {
                const float radius = spacing
                    * std::sqrt(static_cast<float>(x * x + y * y + z * z));
                total += volume * host_cuda_kernel_weight(radius, horizon);
            }
        }
    }
    if (!std::isfinite(total) || total <= 0.0F) {
        throw std::runtime_error("invalid CUDA cubic-kernel scaling factor");
    }
    return 1.0F / total;
}

Fixture performance_fixture(const Profile& profile, int iterations) {
    if (profile.id == "nuv-tiny-oracle.v0") {
        throw std::invalid_argument("tiny profile is a fixture matrix, not one performance block");
    }
    Fixture fixture;
    fixture.name = profile.id;
    fixture.rest_density = profile.rest_density;
    fixture.spacing = profile.spacing;
    fixture.mass = profile.mass;
    fixture.horizon = profile.horizon;
    fixture.time_step = profile.time_step;
    fixture.gravity = profile.gravity;
    fixture.kappa = profile.kappa;
    fixture.lambda = profile.lambda;
    fixture.mu = profile.mu;
    fixture.gamma = profile.gamma;
    fixture.terms = profile.terms;
    fixture.iterations = iterations;
    fixture.pair_capacity = profile.max_directed_pairs;
    fixture.grid_margin = profile.grid_margin;
    fixture.advected = profile.advected;
    fixture.trace_length = profile.trace_length;
    fixture.particles.reserve(profile.samples);
    if (profile.record_version >= 1) {
        fixture.lattice_index_by_sample.reserve(profile.samples);
    }
    const double center_x = 0.5 * static_cast<double>(profile.lattice_x - 1) * profile.spacing;
    const double center_y = 0.5 * static_cast<double>(profile.lattice_y - 1) * profile.spacing;
    const double center_z = 0.5 * static_cast<double>(profile.lattice_z - 1) * profile.spacing;
    for (std::size_t sample = 0; sample < profile.samples; ++sample) {
        const std::size_t lattice =
            profile.initialization_order == InitializationOrder::AffinePermutation
            ? (profile.permutation_multiplier * sample + profile.permutation_offset)
                % profile.samples
            : sample;
        const int x = static_cast<int>(lattice % static_cast<std::size_t>(profile.lattice_x));
        const int y = static_cast<int>(
            (lattice / static_cast<std::size_t>(profile.lattice_x))
            % static_cast<std::size_t>(profile.lattice_y));
        const int z = static_cast<int>(
            lattice / (static_cast<std::size_t>(profile.lattice_x)
                * static_cast<std::size_t>(profile.lattice_y)));
        Vec3 position = profile.record_version == 0
            ? Vec3{x * profile.spacing, y * profile.spacing, z * profile.spacing}
            : Vec3{
                  center_x + profile.lattice_scale * (x * profile.spacing - center_x),
                  center_y + profile.lattice_scale * (y * profile.spacing - center_y),
                  center_z + profile.lattice_scale * (z * profile.spacing - center_z)};
        Vec3 velocity{};
        if (profile.advected) {
            const double nx = (position.x - center_x) / std::max(center_x, profile.spacing);
            const double ny = (position.y - center_y) / std::max(center_y, profile.spacing);
            const double nz = (position.z - center_z) / std::max(center_z, profile.spacing);
            constexpr double PI = 3.141592653589793238462643383279502884;
            velocity = {
                -0.20 * ny,
                0.20 * nx,
                0.05 * std::sin(PI * nx) * std::sin(PI * ny) * std::cos(PI * nz)};
        }
        fixture.particles.push_back({position, velocity, false});
        if (profile.record_version >= 1) {
            fixture.lattice_index_by_sample.push_back(static_cast<int>(lattice));
        }
    }
    return fixture;
}

std::string fixture_input_hash(const Fixture& fixture) {
    std::ostringstream data;
    data << std::setprecision(17) << fixture.name << '|' << fixture.rest_density << '|'
         << fixture.spacing << '|' << fixture.mass << '|' << fixture.horizon << '|'
         << fixture.time_step << '|' << fixture.gravity.x << ',' << fixture.gravity.y << ','
         << fixture.gravity.z << '|' << fixture.kappa << '|' << fixture.lambda << '|'
         << fixture.mu << '|' << fixture.gamma << '|' << fixture.iterations << '|'
         << fixture.terms.incompressibility << fixture.terms.bulk_viscosity
         << fixture.terms.shear_viscosity << fixture.terms.surface_tension << '|';
    for (const Particle& particle : fixture.particles) {
        data << particle.position.x << ',' << particle.position.y << ',' << particle.position.z
             << ';' << particle.velocity.x << ',' << particle.velocity.y << ','
             << particle.velocity.z << ';' << particle.fixed << '|';
    }
    if (!fixture.lattice_index_by_sample.empty()) {
        data << "v1|pair_capacity:" << fixture.pair_capacity << "|grid_margin:"
             << fixture.grid_margin << "|advected:" << fixture.advected
             << "|trace_length:" << fixture.trace_length << "|lattice_index:";
        for (int index : fixture.lattice_index_by_sample) {
            data << index << ',';
        }
    }
    return sha256_hex(data.str());
}

std::string executable_hash() {
    std::ifstream input("/proc/self/exe", std::ios::binary);
    if (!input) {
        throw std::runtime_error("cannot open /proc/self/exe for binary hash");
    }
    const std::string bytes{
        std::istreambuf_iterator<char>(input), std::istreambuf_iterator<char>()};
    if (input.bad()) {
        throw std::runtime_error("cannot read /proc/self/exe for binary hash");
    }
    return sha256_hex(bytes);
}

std::uint64_t executable_bytes() {
    std::ifstream input("/proc/self/exe", std::ios::binary | std::ios::ate);
    if (!input) {
        throw std::runtime_error("cannot open /proc/self/exe for binary size");
    }
    const std::streampos size = input.tellg();
    if (size < 0) {
        throw std::runtime_error("cannot determine /proc/self/exe size");
    }
    return static_cast<std::uint64_t>(size);
}

std::string ordered_output_digest(const OracleResult& state) {
    std::ostringstream data;
    data << std::setprecision(17) << "density:" << state.density.size() << '|';
    for (double value : state.density) {
        data << value << ',';
    }
    data << "|source:" << state.source.size() << '|';
    for (Vec3 value : state.source) {
        data << value.x << ',' << value.y << ',' << value.z << ';';
    }
    data << "|matrix:" << state.local_matrix.size() << '|';
    for (const Mat3& value : state.local_matrix) {
        for (double component : value.v) {
            data << component << ',';
        }
        data << ';';
    }
    data << "|position:" << state.next_position.size() << '|';
    for (Vec3 value : state.next_position) {
        data << value.x << ',' << value.y << ',' << value.z << ';';
    }
    data << "|velocity:" << state.final_velocity.size() << '|';
    for (Vec3 value : state.final_velocity) {
        data << value.x << ',' << value.y << ',' << value.z << ';';
    }
    return sha256_hex(data.str());
}

std::string csr_vectors_digest(
    const std::vector<int>& offsets,
    const std::vector<int>& neighbors) {
    std::ostringstream data;
    data << "offsets:" << offsets.size() << '|';
    for (int value : offsets) {
        data << value << ',';
    }
    data << "|neighbors:" << neighbors.size() << '|';
    for (int value : neighbors) {
        data << value << ',';
    }
    return sha256_hex(data.str());
}

std::string csr_digest(const CapturedRun& run) {
    return csr_vectors_digest(run.offsets, run.neighbors);
}

std::string integer_vector_digest(const char* label, const std::vector<int>& values) {
    std::ostringstream data;
    data << label << ':' << values.size() << '|';
    for (int value : values) {
        data << value << ',';
    }
    return sha256_hex(data.str());
}

enum class PairTraversalMode {
    SeparateRetained,
    FusedOwnerTermsP1,
};

enum class NeighborEncodingMode {
    U32Retained,
    CompactU16P2,
};

class CudaBaseline {
public:
    CudaBaseline(
        const Fixture& fixture,
        AccumulationMode accumulation_mode,
        HandoffMode handoff_mode,
        TermKernelMode term_kernel_mode,
        StorageMode storage_mode = StorageMode::StableSampleV0,
        PairTraversalMode pair_traversal_mode = PairTraversalMode::SeparateRetained,
        NeighborEncodingMode neighbor_encoding_mode = NeighborEncodingMode::U32Retained)
        : fixture_(fixture),
          accumulation_mode_(accumulation_mode),
          handoff_mode_(handoff_mode),
          term_kernel_mode_(term_kernel_mode),
          storage_mode_(storage_mode),
          pair_traversal_mode_(pair_traversal_mode),
          neighbor_encoding_mode_(neighbor_encoding_mode),
          count_(static_cast<int>(fixture.particles.size())),
          grid_(describe_grid(fixture)),
          pair_capacity_(fixture.pair_capacity != 0U
                  ? fixture.pair_capacity
                  : (fixture.particles.size() <= 256U
                          ? fixture.particles.size() * fixture.particles.size()
                          : fixture.particles.size() * 123U)),
          kernel_scale_(host_cuda_kernel_scale(
              static_cast<float>(fixture.spacing), static_cast<float>(fixture.horizon))) {
        if (count_ <= 0) {
            throw std::invalid_argument("CUDA fixture is empty");
        }
        if (handoff_mode_ == HandoffMode::PointerSwapO1
            && accumulation_mode_ != AccumulationMode::GatherDirectedR0
            && accumulation_mode_ != AccumulationMode::UniquePairSegmentedO3) {
            throw std::invalid_argument(
                "pointer-swap-o1 requires gather or segmented accumulation");
        }
        if (term_kernel_mode_ == TermKernelMode::SpecializedO2
            && (accumulation_mode_ != AccumulationMode::GatherDirectedR0
                && accumulation_mode_ != AccumulationMode::UniquePairSegmentedO3)) {
            throw std::invalid_argument(
                "nuv-terms-specialized-o2 requires gather or segmented accumulation");
        }
        if (term_kernel_mode_ == TermKernelMode::SpecializedO2
            && handoff_mode_ != HandoffMode::PointerSwapO1) {
            throw std::invalid_argument(
                "nuv-terms-specialized-o2 requires pointer-swap-o1");
        }
        if (accumulation_mode_ == AccumulationMode::UniquePairSegmentedO3
            && (handoff_mode_ != HandoffMode::PointerSwapO1
                || term_kernel_mode_ != TermKernelMode::SpecializedO2)) {
            throw std::invalid_argument(
                "nuv-unique-pair-segmented-o3 requires pointer-swap-o1 and specialized-o2");
        }
        if (storage_mode_ == StorageMode::CellSortedO4
            && (accumulation_mode_ != AccumulationMode::GatherDirectedR0
                || handoff_mode_ != HandoffMode::PointerSwapO1
                || term_kernel_mode_ != TermKernelMode::SpecializedO2)) {
            throw std::invalid_argument(
                "cell-sorted-o4 requires gather, pointer-swap-o1 and specialized-o2");
        }
        if (fixture_.advected && storage_mode_ == StorageMode::CellSortedO4) {
            throw std::invalid_argument(
                "advected cell-sorted storage requires the later NP1-P3 dynamic remap path");
        }
        if (pair_traversal_mode_ == PairTraversalMode::FusedOwnerTermsP1
            && (accumulation_mode_ != AccumulationMode::GatherDirectedR0
                || handoff_mode_ != HandoffMode::PointerSwapO1
                || term_kernel_mode_ != TermKernelMode::SpecializedO2
                || storage_mode_ != StorageMode::StableSampleV0)) {
            throw std::invalid_argument(
                "fused-owner-terms-p1 requires retained gather/pointer/O2/stable identity");
        }
        compact_neighbor_ids_ = neighbor_encoding_mode_ == NeighborEncodingMode::CompactU16P2
            && count_ <= static_cast<int>(std::numeric_limits<std::uint16_t>::max());
        if (neighbor_encoding_mode_ == NeighborEncodingMode::CompactU16P2
            && pair_traversal_mode_ != PairTraversalMode::FusedOwnerTermsP1) {
            throw std::invalid_argument("compact-csr-u16-p2 requires fused-owner-terms-p1");
        }
        allocate_device_storage();
        upload_fixture();
        if (storage_mode_ == StorageMode::CellSortedO4) {
            initialize_cell_sorted_storage();
        }
        if (accumulation_mode_ == AccumulationMode::UniquePairSegmentedO3) {
            initialize_segmented_layout();
        }
    }

    CudaBaseline(const CudaBaseline&) = delete;
    CudaBaseline& operator=(const CudaBaseline&) = delete;

    ~CudaBaseline() {
        cudaFree(reference_);
        cudaFree(initial_velocity_);
        cudaFree(fixed_);
        cudaFree(predicted_);
        cudaFree(current_);
        cudaFree(linearization_);
        cudaFree(next_);
        cudaFree(final_velocity_);
        cudaFree(density_);
        cudaFree(source_);
        cudaFree(matrix_);
        cudaFree(energy_);
        cudaFree(keys_input_);
        cudaFree(keys_sorted_);
        cudaFree(indices_input_);
        cudaFree(indices_sorted_);
        cudaFree(cell_start_);
        cudaFree(cell_end_);
        cudaFree(neighbor_counts_);
        cudaFree(neighbor_offsets_);
        cudaFree(neighbors_);
        cudaFree(compact_neighbors_);
        cudaFree(sort_scratch_);
        cudaFree(scan_scratch_);
        cudaFree(error_flag_);
        cudaFree(reverse_slots_);
        cudaFree(pair_fragments_);
        cudaFree(layout_error_flag_);
        cudaFree(sorted_reference_);
        cudaFree(sorted_initial_velocity_);
        cudaFree(sorted_fixed_);
        cudaFree(storage_to_sample_);
        cudaFree(sample_to_storage_);
        cudaFree(storage_error_flag_);
        cudaFree(captured_reference_);
        cudaFree(neighbor_error_flag_);
    }

    void reset_seed() {
        upload_fixture();
    }

    CapturedRun execute(bool capture, bool advance = false) {
        if (advance && !fixture_.advected) {
            throw std::invalid_argument("state advance requires an advected fixture");
        }
        const float3* solver_reference = storage_mode_ == StorageMode::CellSortedO4
            ? sorted_reference_ : reference_;
        const float3* solver_initial_velocity = storage_mode_ == StorageMode::CellSortedO4
            ? sorted_initial_velocity_ : initial_velocity_;
        const std::uint8_t* solver_fixed = storage_mode_ == StorageMode::CellSortedO4
            ? sorted_fixed_ : fixed_;
        std::vector<EventInterval> intervals;
        intervals.reserve(static_cast<std::size_t>(4 + 7 * fixture_.iterations));
        cudaEvent_t total_begin{};
        cudaEvent_t total_end{};
        check_cuda(cudaEventCreate(&total_begin), "cudaEventCreate(total begin)");
        check_cuda(cudaEventCreate(&total_end), "cudaEventCreate(total end)");
        check_cuda(cudaEventRecord(total_begin), "cudaEventRecord(total begin)");

        timed(intervals, EventInterval::Stage::Prediction, [&] {
            check_cuda(cudaMemsetAsync(error_flag_, 0, sizeof(int)), "reset local solve flag");
            predict_positions<<<blocks_for(count_), THREADS>>>(
                solver_reference, solver_initial_velocity, predicted_, current_, count_,
                make_float3(
                    static_cast<float>(fixture_.gravity.x),
                    static_cast<float>(fixture_.gravity.y),
                    static_cast<float>(fixture_.gravity.z)),
                static_cast<float>(fixture_.time_step));
        });
        timed(intervals, EventInterval::Stage::Neighbor, [&] {
            enqueue_neighbor_construction();
            if (accumulation_mode_ == AccumulationMode::UniquePairSegmentedO3) {
                check_cuda(
                    cudaMemsetAsync(layout_error_flag_, 0, sizeof(int)), "reset O3 layout flag");
                validate_reverse_slots<<<blocks_for(count_), THREADS>>>(
                    neighbor_offsets_, neighbors_, reverse_slots_, layout_error_flag_, count_);
            }
        });

        const bool fuse_owner_terms =
            pair_traversal_mode_ == PairTraversalMode::FusedOwnerTermsP1
            && fixture_.terms.incompressibility
            && (fixture_.terms.bulk_viscosity || fixture_.terms.shear_viscosity
                || fixture_.terms.surface_tension);
        for (int iteration = 0; iteration < fixture_.iterations; ++iteration) {
            timed(intervals, EventInterval::Stage::Density, [&] {
                if (compact_neighbor_ids_) {
                    compute_density<<<blocks_for(count_), THREADS>>>(
                        current_, neighbor_offsets_, compact_neighbors_, density_, count_,
                        static_cast<float>(fixture_.mass), static_cast<float>(fixture_.horizon),
                        kernel_scale_);
                } else {
                    compute_density<<<blocks_for(count_), THREADS>>>(
                        current_, neighbor_offsets_, neighbors_, density_, count_,
                        static_cast<float>(fixture_.mass), static_cast<float>(fixture_.horizon),
                        kernel_scale_);
                }
            });
            timed(intervals, EventInterval::Stage::Reset, [&] {
                check_cuda(cudaMemsetAsync(source_, 0, 3U * count_ * sizeof(float)), "reset source");
                check_cuda(cudaMemsetAsync(matrix_, 0, 9U * count_ * sizeof(float)), "reset matrix");
            });
            if (fuse_owner_terms) {
                timed(intervals, EventInterval::Stage::FusedOwnerTerms, [&] {
                    if (fixture_.terms.bulk_viscosity && fixture_.terms.shear_viscosity) {
                        if (fixture_.terms.surface_tension) {
                            enqueue_fused_owner_terms<true, true, true>(solver_reference);
                        } else {
                            enqueue_fused_owner_terms<true, true, false>(solver_reference);
                        }
                    } else if (fixture_.terms.bulk_viscosity) {
                        if (fixture_.terms.surface_tension) {
                            enqueue_fused_owner_terms<true, false, true>(solver_reference);
                        } else {
                            enqueue_fused_owner_terms<true, false, false>(solver_reference);
                        }
                    } else if (fixture_.terms.shear_viscosity) {
                        if (fixture_.terms.surface_tension) {
                            enqueue_fused_owner_terms<false, true, true>(solver_reference);
                        } else {
                            enqueue_fused_owner_terms<false, true, false>(solver_reference);
                        }
                    } else {
                        enqueue_fused_owner_terms<false, false, true>(solver_reference);
                    }
                });
            }
            if (fixture_.terms.incompressibility && !fuse_owner_terms) {
                timed(intervals, EventInterval::Stage::Incompressibility, [&] {
                    if (accumulation_mode_ == AccumulationMode::UniquePairSegmentedO3) {
                        accumulate_density_term_unique_pairs<<<blocks_for(count_), THREADS>>>(
                            current_, density_, neighbor_offsets_, neighbors_, reverse_slots_,
                            pair_fragments_, count_, static_cast<float>(fixture_.rest_density),
                            static_cast<float>(fixture_.kappa),
                            static_cast<float>(fixture_.horizon),
                            static_cast<float>(fixture_.time_step), kernel_scale_);
                        reduce_segmented_fragments<<<blocks_for(count_), THREADS>>>(
                            neighbor_offsets_, pair_fragments_, source_, matrix_, count_);
                    } else if (accumulation_mode_ == AccumulationMode::GatherDirectedR0) {
                        accumulate_density_term_gather_directed<<<blocks_for(count_), THREADS>>>(
                            current_, density_, neighbor_offsets_, neighbors_, source_, matrix_, count_,
                            static_cast<float>(fixture_.rest_density),
                            static_cast<float>(fixture_.kappa),
                            static_cast<float>(fixture_.horizon),
                            static_cast<float>(fixture_.time_step), kernel_scale_);
                    } else {
                        accumulate_density_term<<<blocks_for(count_), THREADS>>>(
                            current_, density_, neighbor_offsets_, neighbors_, source_, matrix_, count_,
                            static_cast<float>(fixture_.rest_density),
                            static_cast<float>(fixture_.kappa),
                            static_cast<float>(fixture_.horizon),
                            static_cast<float>(fixture_.time_step), kernel_scale_);
                    }
                });
            }
            if ((fixture_.terms.bulk_viscosity || fixture_.terms.shear_viscosity)
                && !fuse_owner_terms) {
                timed(intervals, EventInterval::Stage::Viscosity, [&] {
                    if (accumulation_mode_ == AccumulationMode::UniquePairSegmentedO3) {
                        if (fixture_.terms.bulk_viscosity && fixture_.terms.shear_viscosity) {
                            accumulate_viscosity_term_unique_pairs_specialized<true, true>
                                <<<blocks_for(count_), THREADS>>>(
                                    solver_reference, current_, neighbor_offsets_, neighbors_,
                                    reverse_slots_, pair_fragments_, count_,
                                    static_cast<float>(fixture_.rest_density),
                                    static_cast<float>(fixture_.lambda),
                                    static_cast<float>(fixture_.mu),
                                    static_cast<float>(fixture_.horizon),
                                    static_cast<float>(fixture_.time_step), kernel_scale_);
                        } else if (fixture_.terms.bulk_viscosity) {
                            accumulate_viscosity_term_unique_pairs_specialized<true, false>
                                <<<blocks_for(count_), THREADS>>>(
                                    solver_reference, current_, neighbor_offsets_, neighbors_,
                                    reverse_slots_, pair_fragments_, count_,
                                    static_cast<float>(fixture_.rest_density),
                                    static_cast<float>(fixture_.lambda),
                                    static_cast<float>(fixture_.mu),
                                    static_cast<float>(fixture_.horizon),
                                    static_cast<float>(fixture_.time_step), kernel_scale_);
                        } else {
                            accumulate_viscosity_term_unique_pairs_specialized<false, true>
                                <<<blocks_for(count_), THREADS>>>(
                                    solver_reference, current_, neighbor_offsets_, neighbors_,
                                    reverse_slots_, pair_fragments_, count_,
                                    static_cast<float>(fixture_.rest_density),
                                    static_cast<float>(fixture_.lambda),
                                    static_cast<float>(fixture_.mu),
                                    static_cast<float>(fixture_.horizon),
                                    static_cast<float>(fixture_.time_step), kernel_scale_);
                        }
                        reduce_segmented_fragments<<<blocks_for(count_), THREADS>>>(
                            neighbor_offsets_, pair_fragments_, source_, matrix_, count_);
                    } else if (accumulation_mode_ == AccumulationMode::GatherDirectedR0) {
                        if (term_kernel_mode_ == TermKernelMode::SpecializedO2) {
                            if (fixture_.terms.bulk_viscosity
                                && fixture_.terms.shear_viscosity) {
                                accumulate_viscosity_term_gather_specialized<true, true>
                                    <<<blocks_for(count_), THREADS>>>(
                                        solver_reference, current_, neighbor_offsets_, neighbors_, source_,
                                        matrix_, count_, static_cast<float>(fixture_.rest_density),
                                        static_cast<float>(fixture_.lambda),
                                        static_cast<float>(fixture_.mu),
                                        static_cast<float>(fixture_.horizon),
                                        static_cast<float>(fixture_.time_step), kernel_scale_);
                            } else if (fixture_.terms.bulk_viscosity) {
                                accumulate_viscosity_term_gather_specialized<true, false>
                                    <<<blocks_for(count_), THREADS>>>(
                                        solver_reference, current_, neighbor_offsets_, neighbors_, source_,
                                        matrix_, count_, static_cast<float>(fixture_.rest_density),
                                        static_cast<float>(fixture_.lambda),
                                        static_cast<float>(fixture_.mu),
                                        static_cast<float>(fixture_.horizon),
                                        static_cast<float>(fixture_.time_step), kernel_scale_);
                            } else {
                                accumulate_viscosity_term_gather_specialized<false, true>
                                    <<<blocks_for(count_), THREADS>>>(
                                        solver_reference, current_, neighbor_offsets_, neighbors_, source_,
                                        matrix_, count_, static_cast<float>(fixture_.rest_density),
                                        static_cast<float>(fixture_.lambda),
                                        static_cast<float>(fixture_.mu),
                                        static_cast<float>(fixture_.horizon),
                                        static_cast<float>(fixture_.time_step), kernel_scale_);
                            }
                        } else {
                            accumulate_viscosity_term_gather_directed
                                <<<blocks_for(count_), THREADS>>>(
                                    solver_reference, current_, neighbor_offsets_, neighbors_, source_,
                                    matrix_, count_, static_cast<float>(fixture_.rest_density),
                                    static_cast<float>(fixture_.lambda),
                                    static_cast<float>(fixture_.mu),
                                    static_cast<float>(fixture_.horizon),
                                    static_cast<float>(fixture_.time_step), kernel_scale_,
                                    fixture_.terms.bulk_viscosity,
                                    fixture_.terms.shear_viscosity);
                        }
                    } else {
                        accumulate_viscosity_term<<<blocks_for(count_), THREADS>>>(
                            solver_reference, current_, neighbor_offsets_, neighbors_, source_, matrix_,
                            count_, static_cast<float>(fixture_.rest_density),
                            static_cast<float>(fixture_.lambda), static_cast<float>(fixture_.mu),
                            static_cast<float>(fixture_.horizon),
                            static_cast<float>(fixture_.time_step), kernel_scale_,
                            fixture_.terms.bulk_viscosity, fixture_.terms.shear_viscosity);
                    }
                });
            }
            if (fixture_.terms.surface_tension && !fuse_owner_terms) {
                timed(intervals, EventInterval::Stage::Surface, [&] {
                    if (accumulation_mode_ == AccumulationMode::UniquePairSegmentedO3) {
                        accumulate_surface_term_unique_pairs<<<blocks_for(count_), THREADS>>>(
                            current_, neighbor_offsets_, neighbors_, reverse_slots_, pair_fragments_,
                            count_, static_cast<float>(fixture_.gamma),
                            static_cast<float>(fixture_.spacing),
                            static_cast<float>(fixture_.time_step));
                        reduce_segmented_fragments<<<blocks_for(count_), THREADS>>>(
                            neighbor_offsets_, pair_fragments_, source_, matrix_, count_);
                    } else if (accumulation_mode_ == AccumulationMode::GatherDirectedR0) {
                        accumulate_surface_term_gather_directed<<<blocks_for(count_), THREADS>>>(
                            current_, neighbor_offsets_, neighbors_, source_, matrix_, count_,
                            static_cast<float>(fixture_.gamma),
                            static_cast<float>(fixture_.spacing),
                            static_cast<float>(fixture_.time_step));
                    } else {
                        accumulate_surface_term<<<blocks_for(count_), THREADS>>>(
                            current_, neighbor_offsets_, neighbors_, source_, matrix_, count_,
                            static_cast<float>(fixture_.gamma),
                            static_cast<float>(fixture_.spacing),
                            static_cast<float>(fixture_.time_step));
                    }
                });
            }
            timed(intervals, EventInterval::Stage::Update, [&] {
                check_cuda(
                    cudaMemcpyAsync(
                        linearization_, current_, count_ * sizeof(float3), cudaMemcpyDeviceToDevice),
                    "capture linearization position");
                update_positions<<<blocks_for(count_), THREADS>>>(
                    solver_reference, predicted_, solver_fixed, source_, matrix_, next_, error_flag_, count_);
            });
            timed(intervals, EventInterval::Stage::Handoff, [&] {
                if (handoff_mode_ == HandoffMode::PointerSwapO1) {
                    std::swap(current_, next_);
                } else {
                    check_cuda(cudaMemcpyAsync(
                                   current_, next_, count_ * sizeof(float3),
                                   cudaMemcpyDeviceToDevice),
                        "handoff next position");
                }
            });
        }
        timed(intervals, EventInterval::Stage::Density, [&] {
            if (compact_neighbor_ids_) {
                compute_density<<<blocks_for(count_), THREADS>>>(
                    current_, neighbor_offsets_, compact_neighbors_, density_, count_,
                    static_cast<float>(fixture_.mass), static_cast<float>(fixture_.horizon),
                    kernel_scale_);
            } else {
                compute_density<<<blocks_for(count_), THREADS>>>(
                    current_, neighbor_offsets_, neighbors_, density_, count_,
                    static_cast<float>(fixture_.mass), static_cast<float>(fixture_.horizon),
                    kernel_scale_);
            }
        });
        timed(intervals, EventInterval::Stage::Handoff, [&] {
            reconstruct_velocity<<<blocks_for(count_), THREADS>>>(
                solver_reference, current_, solver_fixed, final_velocity_, count_,
                static_cast<float>(fixture_.time_step));
        });
        const float3* captured_reference = solver_reference;
        if (capture && advance) {
            check_cuda(cudaMemcpyAsync(
                           captured_reference_, solver_reference,
                           static_cast<std::size_t>(count_) * sizeof(float3),
                           cudaMemcpyDeviceToDevice),
                "capture dynamic reference positions");
            captured_reference = captured_reference_;
        }
        if (advance) {
            timed(intervals, EventInterval::Stage::Handoff, [&] {
                check_cuda(cudaMemcpyAsync(
                               reference_, current_, static_cast<std::size_t>(count_) * sizeof(float3),
                               cudaMemcpyDeviceToDevice),
                    "publish dynamic reference positions");
                check_cuda(cudaMemcpyAsync(
                               initial_velocity_, final_velocity_,
                               static_cast<std::size_t>(count_) * sizeof(float3),
                               cudaMemcpyDeviceToDevice),
                    "publish dynamic velocity");
            });
        }
        check_cuda(cudaGetLastError(), "enqueue CUDA baseline");
        check_cuda(cudaEventRecord(total_end), "cudaEventRecord(total end)");
        check_cuda(cudaEventSynchronize(total_end), "cudaEventSynchronize(total end)");

        CapturedRun result;
        result.device_memory_bytes = device_memory_bytes_;
        result.neighbor_id_bytes = compact_neighbor_ids_ ? sizeof(std::uint16_t) : sizeof(int);
        result.layout_memory_bytes = layout_memory_bytes_;
        result.layout_setup_ms = layout_setup_ms_;
        result.storage_memory_bytes = storage_memory_bytes_;
        result.storage_setup_ms = storage_setup_ms_;
        result.storage_to_sample_sha256 = storage_to_sample_sha256_;
        result.sample_to_storage_sha256 = sample_to_storage_sha256_;
        int local_solve_failed = 0;
        check_cuda(cudaMemcpy(
                       &local_solve_failed, error_flag_, sizeof(int), cudaMemcpyDeviceToHost),
            "copy local solve flag");
        result.local_solve_failed = local_solve_failed != 0;
        if (accumulation_mode_ == AccumulationMode::UniquePairSegmentedO3) {
            int layout_failed = 0;
            check_cuda(cudaMemcpy(
                           &layout_failed, layout_error_flag_, sizeof(int), cudaMemcpyDeviceToHost),
                "copy O3 layout flag");
            result.reverse_map_valid = layout_failed == 0;
        }
        if (storage_mode_ == StorageMode::CellSortedO4) {
            int storage_failed = 0;
            check_cuda(cudaMemcpy(
                           &storage_failed, storage_error_flag_, sizeof(int), cudaMemcpyDeviceToHost),
                "copy O4 storage flag");
            result.storage_map_valid = storage_failed == 0;
        }
        int neighbor_failed = 0;
        check_cuda(cudaMemcpy(
                       &neighbor_failed, neighbor_error_flag_, sizeof(int), cudaMemcpyDeviceToHost),
            "copy neighbor build flag");
        result.neighbor_build_valid = neighbor_failed == 0;
        result.local_solve_failed = result.local_solve_failed || neighbor_failed != 0;
        float total_ms = 0.0F;
        check_cuda(cudaEventElapsedTime(&total_ms, total_begin, total_end), "elapsed total");
        result.timing.total = total_ms;
        collect_stage_timings(intervals, result.timing);
        destroy_events(intervals, total_begin, total_end);
        if (capture) {
            capture_result(result, captured_reference);
        }
        return result;
    }

private:
    template <bool Bulk, bool Shear, bool Surface>
    void enqueue_fused_owner_terms(const float3* solver_reference) {
        if (compact_neighbor_ids_) {
            accumulate_fused_owner_terms_p1<std::uint16_t, Bulk, Shear, Surface>
                <<<blocks_for(count_), THREADS>>>(
                    solver_reference, current_, density_, neighbor_offsets_, compact_neighbors_,
                    source_, matrix_, count_, static_cast<float>(fixture_.rest_density),
                    static_cast<float>(fixture_.kappa), static_cast<float>(fixture_.lambda),
                    static_cast<float>(fixture_.mu), static_cast<float>(fixture_.gamma),
                    static_cast<float>(fixture_.spacing), static_cast<float>(fixture_.horizon),
                    static_cast<float>(fixture_.time_step), kernel_scale_);
        } else {
            accumulate_fused_owner_terms_p1<int, Bulk, Shear, Surface>
                <<<blocks_for(count_), THREADS>>>(
                    solver_reference, current_, density_, neighbor_offsets_, neighbors_, source_,
                    matrix_, count_, static_cast<float>(fixture_.rest_density),
                    static_cast<float>(fixture_.kappa), static_cast<float>(fixture_.lambda),
                    static_cast<float>(fixture_.mu), static_cast<float>(fixture_.gamma),
                    static_cast<float>(fixture_.spacing), static_cast<float>(fixture_.horizon),
                    static_cast<float>(fixture_.time_step), kernel_scale_);
        }
    }

    template <typename T>
    void allocate(T*& pointer, std::size_t count) {
        check_cuda(cudaMalloc(reinterpret_cast<void**>(&pointer), count * sizeof(T)), "cudaMalloc");
        device_memory_bytes_ += count * sizeof(T);
    }

    void allocate_device_storage() {
        const std::size_t count = static_cast<std::size_t>(count_);
        allocate(reference_, count);
        allocate(initial_velocity_, count);
        allocate(fixed_, count);
        allocate(predicted_, count);
        allocate(current_, count);
        allocate(linearization_, count);
        allocate(next_, count);
        allocate(final_velocity_, count);
        allocate(density_, count);
        allocate(source_, 3U * count);
        allocate(matrix_, 9U * count);
        allocate(energy_, 5U * count);
        allocate(keys_input_, count);
        allocate(keys_sorted_, count);
        allocate(indices_input_, count);
        allocate(indices_sorted_, count);
        allocate(cell_start_, static_cast<std::size_t>(grid_.cells));
        allocate(cell_end_, static_cast<std::size_t>(grid_.cells));
        allocate(neighbor_counts_, count);
        allocate(neighbor_offsets_, count + 1U);
        if (compact_neighbor_ids_) {
            allocate(compact_neighbors_, pair_capacity_);
        } else {
            allocate(neighbors_, pair_capacity_);
        }
        allocate(error_flag_, 1U);
        allocate(neighbor_error_flag_, 1U);
        if (fixture_.advected) {
            allocate(captured_reference_, count);
        }
        if (storage_mode_ == StorageMode::CellSortedO4) {
            const std::size_t before_storage = device_memory_bytes_;
            allocate(sorted_reference_, count);
            allocate(sorted_initial_velocity_, count);
            allocate(sorted_fixed_, count);
            allocate(storage_to_sample_, count);
            allocate(sample_to_storage_, count);
            allocate(storage_error_flag_, 1U);
            storage_memory_bytes_ = device_memory_bytes_ - before_storage;
        }
        if (accumulation_mode_ == AccumulationMode::UniquePairSegmentedO3) {
            const std::size_t before_layout = device_memory_bytes_;
            allocate(reverse_slots_, pair_capacity_);
            allocate(pair_fragments_, pair_capacity_);
            allocate(layout_error_flag_, 1U);
            layout_memory_bytes_ = device_memory_bytes_ - before_layout;
        }

        check_cuda(cub::DeviceRadixSort::SortPairs(
                       nullptr, sort_scratch_bytes_, keys_input_, keys_sorted_, indices_input_,
                       indices_sorted_, count_),
            "query CUB radix-sort storage");
        check_cuda(cub::DeviceScan::ExclusiveSum(
                       nullptr, scan_scratch_bytes_, neighbor_counts_, neighbor_offsets_, count_),
            "query CUB scan storage");
        check_cuda(cudaMalloc(&sort_scratch_, sort_scratch_bytes_), "cudaMalloc(sort scratch)");
        check_cuda(cudaMalloc(&scan_scratch_, scan_scratch_bytes_), "cudaMalloc(scan scratch)");
        device_memory_bytes_ += sort_scratch_bytes_ + scan_scratch_bytes_;
    }

    void initialize_cell_sorted_storage() {
        cudaEvent_t begin{};
        cudaEvent_t end{};
        check_cuda(cudaEventCreate(&begin), "cudaEventCreate(O4 setup begin)");
        check_cuda(cudaEventCreate(&end), "cudaEventCreate(O4 setup end)");
        check_cuda(cudaEventRecord(begin), "cudaEventRecord(O4 setup begin)");
        check_cuda(cudaMemsetAsync(neighbor_error_flag_, 0, sizeof(int)),
            "reset O4 setup neighbor flag");
        enqueue_grid_sort();
        check_cuda(cudaMemsetAsync(storage_error_flag_, 0, sizeof(int)), "reset O4 storage flag");
        initialize_storage_map<<<blocks_for(count_), THREADS>>>(
            keys_sorted_, indices_sorted_, storage_to_sample_, sample_to_storage_,
            storage_error_flag_, count_);
        gather_immutable_storage<<<blocks_for(count_), THREADS>>>(
            reference_, initial_velocity_, fixed_, storage_to_sample_, sorted_reference_,
            sorted_initial_velocity_, sorted_fixed_, count_);
        check_cuda(cudaEventRecord(end), "cudaEventRecord(O4 setup end)");
        check_cuda(cudaEventSynchronize(end), "cudaEventSynchronize(O4 setup end)");
        float milliseconds = 0.0F;
        check_cuda(cudaEventElapsedTime(&milliseconds, begin, end), "elapsed O4 setup");
        storage_setup_ms_ = milliseconds;
        cudaEventDestroy(begin);
        cudaEventDestroy(end);

        int storage_failed = 0;
        check_cuda(cudaMemcpy(
                       &storage_failed, storage_error_flag_, sizeof(int), cudaMemcpyDeviceToHost),
            "copy O4 setup flag");
        if (storage_failed != 0) {
            throw std::runtime_error("O4 storage-map construction failed");
        }
        storage_to_sample_host_.resize(static_cast<std::size_t>(count_));
        sample_to_storage_host_.resize(static_cast<std::size_t>(count_));
        check_cuda(cudaMemcpy(
                       storage_to_sample_host_.data(), storage_to_sample_,
                       storage_to_sample_host_.size() * sizeof(int), cudaMemcpyDeviceToHost),
            "copy O4 storage-to-sample map");
        check_cuda(cudaMemcpy(
                       sample_to_storage_host_.data(), sample_to_storage_,
                       sample_to_storage_host_.size() * sizeof(int), cudaMemcpyDeviceToHost),
            "copy O4 sample-to-storage map");
        for (int storage = 0; storage < count_; ++storage) {
            const int sample = storage_to_sample_host_[static_cast<std::size_t>(storage)];
            if (sample < 0 || sample >= count_
                || sample_to_storage_host_[static_cast<std::size_t>(sample)] != storage) {
                throw std::runtime_error("O4 host map is not a bounded bijection");
            }
        }
        storage_to_sample_sha256_ =
            integer_vector_digest("storage_to_sample", storage_to_sample_host_);
        sample_to_storage_sha256_ =
            integer_vector_digest("sample_to_storage", sample_to_storage_host_);
    }

    void initialize_segmented_layout() {
        enqueue_neighbor_construction();
        check_cuda(cudaMemsetAsync(layout_error_flag_, 0, sizeof(int)), "reset O3 layout flag");
        cudaEvent_t begin{};
        cudaEvent_t end{};
        check_cuda(cudaEventCreate(&begin), "cudaEventCreate(O3 setup begin)");
        check_cuda(cudaEventCreate(&end), "cudaEventCreate(O3 setup end)");
        check_cuda(cudaEventRecord(begin), "cudaEventRecord(O3 setup begin)");
        initialize_reverse_slots<<<blocks_for(count_), THREADS>>>(
            neighbor_offsets_, neighbors_, reverse_slots_, layout_error_flag_, count_);
        check_cuda(cudaEventRecord(end), "cudaEventRecord(O3 setup end)");
        check_cuda(cudaEventSynchronize(end), "cudaEventSynchronize(O3 setup end)");
        float milliseconds = 0.0F;
        check_cuda(cudaEventElapsedTime(&milliseconds, begin, end), "elapsed O3 setup");
        layout_setup_ms_ = milliseconds;
        cudaEventDestroy(begin);
        cudaEventDestroy(end);
        validate_reverse_slots<<<blocks_for(count_), THREADS>>>(
            neighbor_offsets_, neighbors_, reverse_slots_, layout_error_flag_, count_);
        check_cuda(cudaGetLastError(), "initialize O3 reverse map");
        check_cuda(cudaDeviceSynchronize(), "synchronize O3 reverse map");
        int layout_failed = 0;
        check_cuda(cudaMemcpy(
                       &layout_failed, layout_error_flag_, sizeof(int), cudaMemcpyDeviceToHost),
            "copy O3 setup flag");
        if (layout_failed != 0) {
            throw std::runtime_error("O3 reverse-map construction failed");
        }
    }

    void upload_fixture() {
        std::vector<float3> positions;
        std::vector<float3> velocities;
        std::vector<std::uint8_t> fixed;
        positions.reserve(fixture_.particles.size());
        velocities.reserve(fixture_.particles.size());
        fixed.reserve(fixture_.particles.size());
        for (const Particle& particle : fixture_.particles) {
            positions.push_back(make_float3(
                static_cast<float>(particle.position.x),
                static_cast<float>(particle.position.y),
                static_cast<float>(particle.position.z)));
            velocities.push_back(make_float3(
                static_cast<float>(particle.velocity.x),
                static_cast<float>(particle.velocity.y),
                static_cast<float>(particle.velocity.z)));
            fixed.push_back(particle.fixed ? 1U : 0U);
        }
        check_cuda(cudaMemcpy(
                       reference_, positions.data(), positions.size() * sizeof(float3),
                       cudaMemcpyHostToDevice),
            "upload reference positions");
        check_cuda(cudaMemcpy(
                       initial_velocity_, velocities.data(), velocities.size() * sizeof(float3),
                       cudaMemcpyHostToDevice),
            "upload velocities");
        check_cuda(cudaMemcpy(
                       fixed_, fixed.data(), fixed.size() * sizeof(std::uint8_t),
                       cudaMemcpyHostToDevice),
            "upload fixed flags");
    }

    template <typename Enqueue>
    void timed(
        std::vector<EventInterval>& intervals,
        EventInterval::Stage stage,
        Enqueue enqueue) {
        EventInterval interval;
        interval.stage = stage;
        check_cuda(cudaEventCreate(&interval.begin), "cudaEventCreate(stage begin)");
        check_cuda(cudaEventCreate(&interval.end), "cudaEventCreate(stage end)");
        check_cuda(cudaEventRecord(interval.begin), "cudaEventRecord(stage begin)");
        enqueue();
        check_cuda(cudaEventRecord(interval.end), "cudaEventRecord(stage end)");
        intervals.push_back(interval);
    }

    void enqueue_grid_sort() {
        compute_grid_keys<<<blocks_for(count_), THREADS>>>(
            reference_, keys_input_, indices_input_, count_, grid_.origin,
            static_cast<float>(fixture_.horizon), grid_.dimensions, neighbor_error_flag_);
        check_cuda(cub::DeviceRadixSort::SortPairs(
                       sort_scratch_, sort_scratch_bytes_, keys_input_, keys_sorted_, indices_input_,
                       indices_sorted_, count_),
            "CUB radix sort pairs");
        check_cuda(
            cudaMemsetAsync(cell_start_, 0xff, static_cast<std::size_t>(grid_.cells) * sizeof(int)),
            "reset cell starts");
        check_cuda(
            cudaMemsetAsync(cell_end_, 0xff, static_cast<std::size_t>(grid_.cells) * sizeof(int)),
            "reset cell ends");
        mark_cell_ranges<<<blocks_for(count_), THREADS>>>(
            keys_sorted_, cell_start_, cell_end_, count_);
    }

    void enqueue_neighbor_construction() {
        check_cuda(cudaMemsetAsync(neighbor_error_flag_, 0, sizeof(int)),
            "reset neighbor build flag");
        enqueue_grid_sort();
        if (storage_mode_ == StorageMode::CellSortedO4) {
            check_cuda(cudaMemsetAsync(storage_error_flag_, 0, sizeof(int)),
                "reset O4 storage flag");
            validate_storage_map<<<blocks_for(count_), THREADS>>>(
                keys_sorted_, indices_sorted_, storage_to_sample_, sample_to_storage_,
                storage_error_flag_, count_);
            visit_grid_neighbors_cell_sorted<false><<<blocks_for(count_), THREADS>>>(
                reference_, indices_sorted_, storage_to_sample_, sample_to_storage_, cell_start_,
                cell_end_, neighbor_counts_, nullptr, nullptr, count_, grid_.origin,
                static_cast<float>(fixture_.horizon), grid_.dimensions,
                neighbor_error_flag_);
        } else {
            if (compact_neighbor_ids_) {
                visit_grid_neighbors<false, std::uint16_t><<<blocks_for(count_), THREADS>>>(
                    reference_, indices_sorted_, cell_start_, cell_end_, neighbor_counts_, nullptr,
                    nullptr, count_, grid_.origin, static_cast<float>(fixture_.horizon),
                    grid_.dimensions, neighbor_error_flag_);
            } else {
                visit_grid_neighbors<false, int><<<blocks_for(count_), THREADS>>>(
                    reference_, indices_sorted_, cell_start_, cell_end_, neighbor_counts_, nullptr,
                    nullptr, count_, grid_.origin, static_cast<float>(fixture_.horizon),
                    grid_.dimensions, neighbor_error_flag_);
            }
        }
        check_cuda(cub::DeviceScan::ExclusiveSum(
                       scan_scratch_, scan_scratch_bytes_, neighbor_counts_, neighbor_offsets_, count_),
            "CUB exclusive neighbor scan");
        finish_offsets<<<1, 1>>>(neighbor_counts_, neighbor_offsets_, count_);
        clamp_neighbor_offsets<<<blocks_for(count_ + 1), THREADS>>>(
            neighbor_offsets_, count_, static_cast<int>(pair_capacity_), neighbor_error_flag_);
        if (storage_mode_ == StorageMode::CellSortedO4) {
            visit_grid_neighbors_cell_sorted<true><<<blocks_for(count_), THREADS>>>(
                reference_, indices_sorted_, storage_to_sample_, sample_to_storage_, cell_start_,
                cell_end_, nullptr, neighbor_offsets_, neighbors_, count_, grid_.origin,
                static_cast<float>(fixture_.horizon), grid_.dimensions,
                neighbor_error_flag_);
        } else {
            if (compact_neighbor_ids_) {
                visit_grid_neighbors<true, std::uint16_t><<<blocks_for(count_), THREADS>>>(
                    reference_, indices_sorted_, cell_start_, cell_end_, nullptr,
                    neighbor_offsets_, compact_neighbors_, count_, grid_.origin,
                    static_cast<float>(fixture_.horizon), grid_.dimensions,
                    neighbor_error_flag_);
            } else {
                visit_grid_neighbors<true, int><<<blocks_for(count_), THREADS>>>(
                    reference_, indices_sorted_, cell_start_, cell_end_, nullptr,
                    neighbor_offsets_, neighbors_, count_, grid_.origin,
                    static_cast<float>(fixture_.horizon), grid_.dimensions,
                    neighbor_error_flag_);
            }
        }
    }

    static void add_stage_time(StageTiming& timing, EventInterval::Stage stage, double milliseconds) {
        switch (stage) {
        case EventInterval::Stage::Prediction:
            timing.prediction += milliseconds;
            break;
        case EventInterval::Stage::Neighbor:
            timing.neighbor_construction += milliseconds;
            break;
        case EventInterval::Stage::Density:
            timing.density += milliseconds;
            break;
        case EventInterval::Stage::Reset:
            timing.buffer_reset += milliseconds;
            break;
        case EventInterval::Stage::Incompressibility:
            timing.incompressibility += milliseconds;
            break;
        case EventInterval::Stage::Viscosity:
            timing.viscosity += milliseconds;
            break;
        case EventInterval::Stage::Surface:
            timing.surface_tension += milliseconds;
            break;
        case EventInterval::Stage::FusedOwnerTerms:
            timing.fused_owner_terms += milliseconds;
            break;
        case EventInterval::Stage::Update:
            timing.local_update += milliseconds;
            break;
        case EventInterval::Stage::Handoff:
            timing.state_handoff += milliseconds;
            break;
        }
    }

    static void collect_stage_timings(
        const std::vector<EventInterval>& intervals,
        StageTiming& timing) {
        for (const EventInterval& interval : intervals) {
            float milliseconds = 0.0F;
            check_cuda(
                cudaEventElapsedTime(&milliseconds, interval.begin, interval.end),
                "elapsed stage");
            add_stage_time(timing, interval.stage, milliseconds);
        }
    }

    static void destroy_events(
        const std::vector<EventInterval>& intervals,
        cudaEvent_t total_begin,
        cudaEvent_t total_end) {
        for (const EventInterval& interval : intervals) {
            cudaEventDestroy(interval.begin);
            cudaEventDestroy(interval.end);
        }
        cudaEventDestroy(total_begin);
        cudaEventDestroy(total_end);
    }

    void capture_result(CapturedRun& result, const float3* solver_reference) {
        int directed_pairs = 0;
        check_cuda(cudaMemcpy(
                       &directed_pairs, neighbor_offsets_ + count_, sizeof(int), cudaMemcpyDeviceToHost),
            "copy directed-pair count");
        if (directed_pairs < 0 || static_cast<std::size_t>(directed_pairs) > pair_capacity_) {
            throw std::runtime_error("neighbor pair capacity exceeded");
        }
        if (compact_neighbor_ids_) {
            compute_energy_components<<<blocks_for(count_), THREADS>>>(
                solver_reference, predicted_, current_, density_, neighbor_offsets_,
                compact_neighbors_, energy_, count_, static_cast<float>(fixture_.rest_density),
                static_cast<float>(fixture_.spacing), static_cast<float>(fixture_.mass),
                static_cast<float>(fixture_.horizon), static_cast<float>(fixture_.time_step),
                static_cast<float>(fixture_.kappa), static_cast<float>(fixture_.lambda),
                static_cast<float>(fixture_.mu), static_cast<float>(fixture_.gamma), kernel_scale_,
                fixture_.terms.incompressibility, fixture_.terms.bulk_viscosity,
                fixture_.terms.shear_viscosity, fixture_.terms.surface_tension);
        } else {
            compute_energy_components<<<blocks_for(count_), THREADS>>>(
                solver_reference, predicted_, current_, density_, neighbor_offsets_, neighbors_,
                energy_, count_, static_cast<float>(fixture_.rest_density),
                static_cast<float>(fixture_.spacing), static_cast<float>(fixture_.mass),
                static_cast<float>(fixture_.horizon), static_cast<float>(fixture_.time_step),
                static_cast<float>(fixture_.kappa), static_cast<float>(fixture_.lambda),
                static_cast<float>(fixture_.mu), static_cast<float>(fixture_.gamma), kernel_scale_,
                fixture_.terms.incompressibility, fixture_.terms.bulk_viscosity,
                fixture_.terms.shear_viscosity, fixture_.terms.surface_tension);
        }
        check_cuda(cudaGetLastError(), "compute CUDA energy diagnostics");

        std::vector<float3> predicted(static_cast<std::size_t>(count_));
        std::vector<float3> linearization(static_cast<std::size_t>(count_));
        std::vector<float3> current(static_cast<std::size_t>(count_));
        std::vector<float3> velocity(static_cast<std::size_t>(count_));
        std::vector<float> density(static_cast<std::size_t>(count_));
        std::vector<float> source(3U * static_cast<std::size_t>(count_));
        std::vector<float> matrix(9U * static_cast<std::size_t>(count_));
        std::vector<float> energy(5U * static_cast<std::size_t>(count_));
        std::vector<int> physical_offsets(static_cast<std::size_t>(count_) + 1U);
        std::vector<int> physical_neighbors(static_cast<std::size_t>(directed_pairs));

        check_cuda(cudaMemcpy(
                       predicted.data(), predicted_, predicted.size() * sizeof(float3),
                       cudaMemcpyDeviceToHost),
            "copy predicted positions");
        check_cuda(cudaMemcpy(
                       linearization.data(), linearization_, linearization.size() * sizeof(float3),
                       cudaMemcpyDeviceToHost),
            "copy linearization positions");
        check_cuda(cudaMemcpy(
                       current.data(), current_, current.size() * sizeof(float3), cudaMemcpyDeviceToHost),
            "copy final positions");
        check_cuda(cudaMemcpy(
                       velocity.data(), final_velocity_, velocity.size() * sizeof(float3),
                       cudaMemcpyDeviceToHost),
            "copy final velocities");
        check_cuda(cudaMemcpy(
                       density.data(), density_, density.size() * sizeof(float), cudaMemcpyDeviceToHost),
            "copy densities");
        check_cuda(cudaMemcpy(
                       source.data(), source_, source.size() * sizeof(float), cudaMemcpyDeviceToHost),
            "copy source");
        check_cuda(cudaMemcpy(
                       matrix.data(), matrix_, matrix.size() * sizeof(float), cudaMemcpyDeviceToHost),
            "copy matrix");
        check_cuda(cudaMemcpy(
                       energy.data(), energy_, energy.size() * sizeof(float), cudaMemcpyDeviceToHost),
            "copy energies");
        check_cuda(cudaMemcpy(
                       physical_offsets.data(), neighbor_offsets_,
                       physical_offsets.size() * sizeof(int), cudaMemcpyDeviceToHost),
            "copy neighbor offsets");
        if (directed_pairs != 0) {
            if (compact_neighbor_ids_) {
                std::vector<std::uint16_t> compact_physical_neighbors(
                    static_cast<std::size_t>(directed_pairs));
                check_cuda(cudaMemcpy(
                               compact_physical_neighbors.data(), compact_neighbors_,
                               compact_physical_neighbors.size() * sizeof(std::uint16_t),
                               cudaMemcpyDeviceToHost),
                    "copy compact neighbors");
                std::transform(
                    compact_physical_neighbors.begin(), compact_physical_neighbors.end(),
                    physical_neighbors.begin(), [](std::uint16_t neighbor) {
                        return static_cast<int>(neighbor);
                    });
            } else {
                check_cuda(cudaMemcpy(
                               physical_neighbors.data(), neighbors_,
                               physical_neighbors.size() * sizeof(int), cudaMemcpyDeviceToHost),
                    "copy neighbors");
            }
        }
        result.physical_csr_sha256 =
            csr_vectors_digest(physical_offsets, physical_neighbors);
        if (storage_mode_ == StorageMode::CellSortedO4) {
            result.offsets.resize(static_cast<std::size_t>(count_) + 1U);
            result.offsets[0] = 0;
            for (int sample = 0; sample < count_; ++sample) {
                const int storage = sample_to_storage_host_[static_cast<std::size_t>(sample)];
                const int degree = physical_offsets[static_cast<std::size_t>(storage + 1)]
                    - physical_offsets[static_cast<std::size_t>(storage)];
                result.offsets[static_cast<std::size_t>(sample + 1)] =
                    result.offsets[static_cast<std::size_t>(sample)] + degree;
            }
            result.neighbors.reserve(static_cast<std::size_t>(directed_pairs));
            for (int sample = 0; sample < count_; ++sample) {
                const int storage = sample_to_storage_host_[static_cast<std::size_t>(sample)];
                for (int slot = physical_offsets[static_cast<std::size_t>(storage)];
                     slot < physical_offsets[static_cast<std::size_t>(storage + 1)]; ++slot) {
                    const int neighbor_storage =
                        physical_neighbors[static_cast<std::size_t>(slot)];
                    result.neighbors.push_back(
                        storage_to_sample_host_[static_cast<std::size_t>(neighbor_storage)]);
                }
            }
        } else {
            result.offsets = physical_offsets;
            result.neighbors = physical_neighbors;
        }

        std::vector<double> storage_distances;
        storage_distances.reserve(static_cast<std::size_t>(directed_pairs));
        double storage_distance_sum = 0.0;
        for (int owner = 0; owner < count_; ++owner) {
            for (int slot = physical_offsets[static_cast<std::size_t>(owner)];
                 slot < physical_offsets[static_cast<std::size_t>(owner + 1)]; ++slot) {
                const int neighbor = physical_neighbors[static_cast<std::size_t>(slot)];
                if (neighbor != owner) {
                    const double distance = static_cast<double>(std::abs(neighbor - owner));
                    storage_distances.push_back(distance);
                    storage_distance_sum += distance;
                }
            }
        }
        if (!storage_distances.empty()) {
            result.mean_neighbor_storage_distance =
                storage_distance_sum / static_cast<double>(storage_distances.size());
            const std::size_t p95_index = static_cast<std::size_t>(std::ceil(
                0.95 * static_cast<double>(storage_distances.size()))) - 1U;
            std::nth_element(
                storage_distances.begin(), storage_distances.begin() + p95_index,
                storage_distances.end());
            result.p95_neighbor_storage_distance = storage_distances[p95_index];
        }

        OracleResult& state = result.state;
        state.density.resize(static_cast<std::size_t>(count_));
        state.predicted_position.reserve(static_cast<std::size_t>(count_));
        state.linearization_position.reserve(static_cast<std::size_t>(count_));
        state.next_position.reserve(static_cast<std::size_t>(count_));
        state.final_velocity.reserve(static_cast<std::size_t>(count_));
        state.source.resize(static_cast<std::size_t>(count_));
        state.local_matrix.resize(static_cast<std::size_t>(count_));
        Vec3 total_pair_impulse{};
        double pair_impulse_scale = 0.0;
        for (int particle = 0; particle < count_; ++particle) {
            const int physical = storage_mode_ == StorageMode::CellSortedO4
                ? sample_to_storage_host_[static_cast<std::size_t>(particle)] : particle;
            const auto convert = [](float3 value) {
                return Vec3{value.x, value.y, value.z};
            };
            state.density[static_cast<std::size_t>(particle)] =
                density[static_cast<std::size_t>(physical)];
            state.predicted_position.push_back(convert(predicted[physical]));
            state.linearization_position.push_back(convert(linearization[physical]));
            state.next_position.push_back(convert(current[physical]));
            state.final_velocity.push_back(convert(velocity[physical]));
            state.source[particle] = {
                source[3 * physical], source[3 * physical + 1], source[3 * physical + 2]};
            for (int component = 0; component < 9; ++component) {
                state.local_matrix[particle].v[component] = matrix[9 * physical + component];
            }
            const Vec3 matrix_position =
                state.local_matrix[particle] * state.linearization_position[particle];
            const Vec3 impulse = state.source[particle] - matrix_position;
            total_pair_impulse += impulse;
            pair_impulse_scale += norm(state.source[particle]) + norm(matrix_position);
        }
        state.normalized_momentum_residual =
            norm(total_pair_impulse) / std::max(pair_impulse_scale, 1.0e-30);
        state.directed_pairs = static_cast<std::size_t>(directed_pairs);
        for (int particle = 0; particle < count_; ++particle) {
            for (int slot = result.offsets[particle]; slot < result.offsets[particle + 1]; ++slot) {
                const int neighbor = result.neighbors[static_cast<std::size_t>(slot)];
                if (neighbor == particle) {
                    ++result.self_slots;
                } else {
                    ++result.nonself_directed_samples;
                    if (particle < neighbor) {
                        ++result.unique_pairs;
                    }
                }
            }
            state.maximum_degree = std::max(
                state.maximum_degree,
                static_cast<std::size_t>(result.offsets[particle + 1] - result.offsets[particle]));
            const int physical = storage_mode_ == StorageMode::CellSortedO4
                ? sample_to_storage_host_[static_cast<std::size_t>(particle)] : particle;
            state.energy.inertia += energy[5 * physical];
            state.energy.incompressibility += energy[5 * physical + 1];
            state.energy.bulk_viscosity += energy[5 * physical + 2];
            state.energy.shear_viscosity += energy[5 * physical + 3];
            state.energy.surface_tension += energy[5 * physical + 4];
        }
        if (accumulation_mode_ == AccumulationMode::UniquePairSegmentedO3) {
            result.pair_evaluations_per_active_term = result.unique_pairs;
            result.endpoint_fragments_per_active_term = result.nonself_directed_samples;
        } else {
            result.pair_evaluations_per_active_term = result.nonself_directed_samples;
        }
    }

    Fixture fixture_;
    AccumulationMode accumulation_mode_ = AccumulationMode::SourceAtomicV0;
    HandoffMode handoff_mode_ = HandoffMode::CopyV0;
    TermKernelMode term_kernel_mode_ = TermKernelMode::RuntimeV0;
    StorageMode storage_mode_ = StorageMode::StableSampleV0;
    PairTraversalMode pair_traversal_mode_ = PairTraversalMode::SeparateRetained;
    NeighborEncodingMode neighbor_encoding_mode_ = NeighborEncodingMode::U32Retained;
    bool compact_neighbor_ids_ = false;
    int count_ = 0;
    GridDescription grid_;
    std::size_t pair_capacity_ = 0;
    float kernel_scale_ = 1.0F;
    std::size_t device_memory_bytes_ = 0;
    std::size_t layout_memory_bytes_ = 0;
    std::size_t storage_memory_bytes_ = 0;
    double layout_setup_ms_ = 0.0;
    double storage_setup_ms_ = 0.0;
    std::string storage_to_sample_sha256_;
    std::string sample_to_storage_sha256_;
    std::vector<int> storage_to_sample_host_;
    std::vector<int> sample_to_storage_host_;
    float3* reference_ = nullptr;
    float3* initial_velocity_ = nullptr;
    std::uint8_t* fixed_ = nullptr;
    float3* predicted_ = nullptr;
    float3* current_ = nullptr;
    float3* linearization_ = nullptr;
    float3* next_ = nullptr;
    float3* final_velocity_ = nullptr;
    float* density_ = nullptr;
    float* source_ = nullptr;
    float* matrix_ = nullptr;
    float* energy_ = nullptr;
    int* keys_input_ = nullptr;
    int* keys_sorted_ = nullptr;
    int* indices_input_ = nullptr;
    int* indices_sorted_ = nullptr;
    int* cell_start_ = nullptr;
    int* cell_end_ = nullptr;
    int* neighbor_counts_ = nullptr;
    int* neighbor_offsets_ = nullptr;
    int* neighbors_ = nullptr;
    std::uint16_t* compact_neighbors_ = nullptr;
    int* reverse_slots_ = nullptr;
    PairFragment* pair_fragments_ = nullptr;
    void* sort_scratch_ = nullptr;
    std::size_t sort_scratch_bytes_ = 0;
    void* scan_scratch_ = nullptr;
    std::size_t scan_scratch_bytes_ = 0;
    int* error_flag_ = nullptr;
    int* layout_error_flag_ = nullptr;
    float3* sorted_reference_ = nullptr;
    float3* sorted_initial_velocity_ = nullptr;
    std::uint8_t* sorted_fixed_ = nullptr;
    int* storage_to_sample_ = nullptr;
    int* sample_to_storage_ = nullptr;
    int* storage_error_flag_ = nullptr;
    float3* captured_reference_ = nullptr;
    int* neighbor_error_flag_ = nullptr;
};

bool finite_state(const OracleResult& state) {
    const auto finite_vector = [](const std::vector<Vec3>& values) {
        return std::all_of(values.begin(), values.end(), [](Vec3 value) { return finite(value); });
    };
    const auto finite_matrix = [](const std::vector<Mat3>& values) {
        return std::all_of(
            values.begin(), values.end(), [](const Mat3& value) { return finite(value); });
    };
    return std::all_of(state.density.begin(), state.density.end(), [](double value) {
               return std::isfinite(value);
           })
        && finite_vector(state.source) && finite_matrix(state.local_matrix)
        && finite_vector(state.predicted_position) && finite_vector(state.linearization_position)
        && finite_vector(state.next_position) && finite_vector(state.final_velocity)
        && std::isfinite(state.energy.inertia) && std::isfinite(state.energy.incompressibility)
        && std::isfinite(state.energy.bulk_viscosity)
        && std::isfinite(state.energy.shear_viscosity)
        && std::isfinite(state.energy.surface_tension)
        && std::isfinite(state.normalized_momentum_residual);
}

bool valid_symmetric_neighbors(const CapturedRun& run, std::size_t sample_count) {
    if (run.offsets.size() != sample_count + 1U || run.offsets.front() != 0
        || run.offsets.back() != static_cast<int>(run.neighbors.size())) {
        return false;
    }
    if (sample_count > 1024U) {
        std::vector<std::uint64_t> directed;
        directed.reserve(run.neighbors.size());
        for (std::size_t particle = 0; particle < sample_count; ++particle) {
            bool has_self = false;
            for (int slot = run.offsets[particle]; slot < run.offsets[particle + 1U]; ++slot) {
                const int neighbor = run.neighbors[slot];
                if (neighbor < 0 || neighbor >= static_cast<int>(sample_count)) {
                    return false;
                }
                has_self = has_self || neighbor == static_cast<int>(particle);
                directed.push_back((static_cast<std::uint64_t>(particle) << 32U)
                    | static_cast<std::uint32_t>(neighbor));
            }
            if (!has_self) {
                return false;
            }
        }
        std::sort(directed.begin(), directed.end());
        for (std::uint64_t pair : directed) {
            const std::uint64_t reverse = (pair << 32U) | (pair >> 32U);
            if (!std::binary_search(directed.begin(), directed.end(), reverse)) {
                return false;
            }
        }
        return true;
    }
    for (std::size_t particle = 0; particle < sample_count; ++particle) {
        const int begin = run.offsets[particle];
        const int end = run.offsets[particle + 1U];
        if (begin < 0 || end < begin || end > static_cast<int>(run.neighbors.size())) {
            return false;
        }
        bool has_self = false;
        for (int slot = begin; slot < end; ++slot) {
            const int neighbor = run.neighbors[slot];
            if (neighbor < 0 || neighbor >= static_cast<int>(sample_count)) {
                return false;
            }
            has_self = has_self || neighbor == static_cast<int>(particle);
            const int reverse_begin = run.offsets[static_cast<std::size_t>(neighbor)];
            const int reverse_end = run.offsets[static_cast<std::size_t>(neighbor) + 1U];
            if (std::find(
                    run.neighbors.begin() + reverse_begin,
                    run.neighbors.begin() + reverse_end,
                    static_cast<int>(particle))
                == run.neighbors.begin() + reverse_end) {
                return false;
            }
        }
        if (!has_self) {
            return false;
        }
    }
    return true;
}

bool exact_fixture_neighbors(const Fixture& fixture, const CapturedRun& run) {
    if (run.offsets.size() != fixture.particles.size() + 1U) {
        return false;
    }
    const double support = fixture.horizon * (1.0 + 1.0e-12);
    for (std::size_t i = 0; i < fixture.particles.size(); ++i) {
        for (std::size_t j = 0; j < fixture.particles.size(); ++j) {
            const bool expected = norm(fixture.particles[i].position - fixture.particles[j].position)
                <= support;
            const bool found = std::find(
                                   run.neighbors.begin() + run.offsets[i],
                                   run.neighbors.begin() + run.offsets[i + 1U],
                                   static_cast<int>(j))
                != run.neighbors.begin() + run.offsets[i + 1U];
            if (expected != found) {
                return false;
            }
        }
    }
    return true;
}

struct Discrepancy {
    double maximum_absolute = 0.0;
    double maximum_relative = 0.0;
    bool passed = true;
};

void observe(
    Discrepancy& discrepancy,
    double expected,
    double actual,
    double absolute_limit,
    double relative_limit,
    double normalization = 1.0) {
    const double absolute = std::abs(actual - expected) / normalization;
    const double relative = std::abs(actual - expected) / std::max(std::abs(expected), 1.0e-30);
    discrepancy.maximum_absolute = std::max(discrepancy.maximum_absolute, absolute);
    discrepancy.maximum_relative = std::max(discrepancy.maximum_relative, relative);
    discrepancy.passed = discrepancy.passed
        && (absolute <= absolute_limit || relative <= relative_limit);
}

struct Comparison {
    Discrepancy density;
    Discrepancy energy;
    Discrepancy source;
    Discrepancy matrix;
    Discrepancy position;
    Discrepancy velocity;
    double energy_scale = 1.0;
    double source_scale = 1.0;
    double matrix_scale = 1.0;
    bool passed = false;
};

Comparison compare_results(
    const Fixture& fixture,
    const OracleResult& cpu,
    const OracleResult& gpu,
    const Tolerances& tolerances) {
    Comparison result;
    result.energy_scale = std::max(
        static_cast<double>(fixture.particles.size()) * fixture.mass * fixture.spacing
            * fixture.spacing / (fixture.time_step * fixture.time_step),
        1.0e-30);
    result.source_scale = fixture.spacing;
    result.matrix_scale = 1.0;
    const double cpu_energy_for_scale[5] = {
        cpu.energy.inertia,
        cpu.energy.incompressibility,
        cpu.energy.bulk_viscosity,
        cpu.energy.shear_viscosity,
        cpu.energy.surface_tension,
    };
    for (double value : cpu_energy_for_scale) {
        result.energy_scale = std::max(result.energy_scale, std::abs(value));
    }
    for (std::size_t i = 0; i < cpu.source.size(); ++i) {
        result.source_scale = std::max(
            result.source_scale,
            std::max({std::abs(cpu.source[i].x), std::abs(cpu.source[i].y),
                std::abs(cpu.source[i].z)}));
        for (double value : cpu.local_matrix[i].v) {
            result.matrix_scale = std::max(result.matrix_scale, std::abs(value));
        }
    }
    if (cpu.density.size() != gpu.density.size() || cpu.source.size() != gpu.source.size()
        || cpu.local_matrix.size() != gpu.local_matrix.size()
        || cpu.next_position.size() != gpu.next_position.size()
        || cpu.final_velocity.size() != gpu.final_velocity.size()) {
        result.density.passed = false;
        result.energy.passed = false;
        result.source.passed = false;
        result.matrix.passed = false;
        result.position.passed = false;
        result.velocity.passed = false;
        return result;
    }
    for (std::size_t i = 0; i < cpu.density.size(); ++i) {
        observe(result.density, cpu.density[i], gpu.density[i],
            tolerances.density_absolute, tolerances.density_relative);
        const double cpu_source[3] = {cpu.source[i].x, cpu.source[i].y, cpu.source[i].z};
        const double gpu_source[3] = {gpu.source[i].x, gpu.source[i].y, gpu.source[i].z};
        const double cpu_position[3] = {
            cpu.next_position[i].x, cpu.next_position[i].y, cpu.next_position[i].z};
        const double gpu_position[3] = {
            gpu.next_position[i].x, gpu.next_position[i].y, gpu.next_position[i].z};
        const double cpu_velocity[3] = {
            cpu.final_velocity[i].x, cpu.final_velocity[i].y, cpu.final_velocity[i].z};
        const double gpu_velocity[3] = {
            gpu.final_velocity[i].x, gpu.final_velocity[i].y, gpu.final_velocity[i].z};
        for (int component = 0; component < 3; ++component) {
            observe(result.source, cpu_source[component], gpu_source[component],
                tolerances.normalized_absolute, tolerances.normalized_relative,
                result.source_scale);
            observe(result.position, cpu_position[component], gpu_position[component],
                tolerances.position_absolute, 0.0);
            observe(result.velocity, cpu_velocity[component], gpu_velocity[component],
                tolerances.velocity_absolute, 0.0);
        }
        for (int component = 0; component < 9; ++component) {
            observe(result.matrix, cpu.local_matrix[i].v[component], gpu.local_matrix[i].v[component],
                tolerances.normalized_absolute, tolerances.normalized_relative, result.matrix_scale);
        }
    }
    const double cpu_energy[5] = {
        cpu.energy.inertia,
        cpu.energy.incompressibility,
        cpu.energy.bulk_viscosity,
        cpu.energy.shear_viscosity,
        cpu.energy.surface_tension,
    };
    const double gpu_energy[5] = {
        gpu.energy.inertia,
        gpu.energy.incompressibility,
        gpu.energy.bulk_viscosity,
        gpu.energy.shear_viscosity,
        gpu.energy.surface_tension,
    };
    for (int component = 0; component < 5; ++component) {
        observe(result.energy, cpu_energy[component], gpu_energy[component],
            tolerances.normalized_absolute, tolerances.normalized_relative, result.energy_scale);
    }
    result.passed = result.density.passed && result.energy.passed && result.source.passed
        && result.matrix.passed && result.position.passed && result.velocity.passed;
    return result;
}

void append_discrepancy(std::ostringstream& output, const Discrepancy& discrepancy) {
    output << "{\"maximum_absolute\":" << discrepancy.maximum_absolute
           << ",\"maximum_relative\":" << discrepancy.maximum_relative
           << ",\"passed\":" << (discrepancy.passed ? "true" : "false") << '}';
}

void append_timing(std::ostringstream& output, const StageTiming& timing) {
    output << "{\"prediction_ms\":" << timing.prediction
           << ",\"neighbor_construction_ms\":" << timing.neighbor_construction
           << ",\"density_ms\":" << timing.density
           << ",\"buffer_reset_ms\":" << timing.buffer_reset
           << ",\"incompressibility_ms\":" << timing.incompressibility
           << ",\"viscosity_ms\":" << timing.viscosity
           << ",\"surface_tension_ms\":" << timing.surface_tension
           << ",\"fused_owner_terms_ms\":" << timing.fused_owner_terms
           << ",\"local_update_ms\":" << timing.local_update
           << ",\"state_handoff_and_velocity_ms\":" << timing.state_handoff
           << ",\"total_ms\":" << timing.total << '}';
}

struct Statistics {
    double minimum = 0.0;
    double median = 0.0;
    double p95 = 0.0;
    double p99 = 0.0;
    double mean = 0.0;
};

Statistics statistics(std::vector<double> values) {
    if (values.empty()) {
        throw std::invalid_argument("statistics input is empty");
    }
    const double sum = std::accumulate(values.begin(), values.end(), 0.0);
    std::sort(values.begin(), values.end());
    const auto percentile = [&](double probability) {
        const std::size_t rank = static_cast<std::size_t>(
            std::ceil(probability * static_cast<double>(values.size())));
        return values[std::max<std::size_t>(1U, rank) - 1U];
    };
    return {values.front(), percentile(0.5), percentile(0.95), percentile(0.99),
        sum / static_cast<double>(values.size())};
}

void append_statistics(std::ostringstream& output, const Statistics& value) {
    output << "{\"minimum_ms\":" << value.minimum << ",\"median_ms\":" << value.median
           << ",\"p95_ms\":" << value.p95 << ",\"p99_ms\":" << value.p99
           << ",\"mean_ms\":" << value.mean << '}';
}

Statistics collect_statistics(
    const std::vector<StageTiming>& timings,
    double StageTiming::*member) {
    std::vector<double> values;
    values.reserve(timings.size());
    for (const StageTiming& timing : timings) {
        values.push_back(timing.*member);
    }
    return statistics(std::move(values));
}

void append_stage_statistics(
    std::ostringstream& output,
    const std::vector<StageTiming>& timings) {
    output << "{\"prediction\":";
    append_statistics(output, collect_statistics(timings, &StageTiming::prediction));
    output << ",\"neighbor_construction\":";
    append_statistics(output, collect_statistics(timings, &StageTiming::neighbor_construction));
    output << ",\"density\":";
    append_statistics(output, collect_statistics(timings, &StageTiming::density));
    output << ",\"buffer_reset\":";
    append_statistics(output, collect_statistics(timings, &StageTiming::buffer_reset));
    output << ",\"incompressibility\":";
    append_statistics(output, collect_statistics(timings, &StageTiming::incompressibility));
    output << ",\"viscosity\":";
    append_statistics(output, collect_statistics(timings, &StageTiming::viscosity));
    output << ",\"surface_tension\":";
    append_statistics(output, collect_statistics(timings, &StageTiming::surface_tension));
    output << ",\"fused_owner_terms\":";
    append_statistics(output, collect_statistics(timings, &StageTiming::fused_owner_terms));
    output << ",\"local_update\":";
    append_statistics(output, collect_statistics(timings, &StageTiming::local_update));
    output << ",\"state_handoff_and_velocity\":";
    append_statistics(output, collect_statistics(timings, &StageTiming::state_handoff));
    output << ",\"total\":";
    append_statistics(output, collect_statistics(timings, &StageTiming::total));
    output << '}';
}

void append_raw_totals(
    std::ostringstream& output,
    const std::vector<StageTiming>& timings) {
    output << '[';
    for (std::size_t index = 0; index < timings.size(); ++index) {
        if (index != 0) {
            output << ',';
        }
        output << timings[index].total;
    }
    output << ']';
}

std::string device_json() {
    int device = 0;
    int driver = 0;
    int runtime = 0;
    check_cuda(cudaGetDevice(&device), "cudaGetDevice");
    check_cuda(cudaDriverGetVersion(&driver), "cudaDriverGetVersion");
    check_cuda(cudaRuntimeGetVersion(&runtime), "cudaRuntimeGetVersion");
    cudaDeviceProp properties{};
    check_cuda(cudaGetDeviceProperties(&properties, device), "cudaGetDeviceProperties");
    std::ostringstream output;
    output << "{\"name\":\"" << properties.name << "\",\"compute_capability\":\""
           << properties.major << '.' << properties.minor << "\",\"multiprocessors\":"
           << properties.multiProcessorCount << ",\"global_memory_bytes\":"
           << properties.totalGlobalMem << ",\"driver_version\":" << driver
           << ",\"runtime_version\":" << runtime << ",\"cccl_cub_version\":\""
           << CUB_VERSION / 100000 << '.' << (CUB_VERSION / 100) % 1000 << '.'
           << CUB_VERSION % 100 << "\",\"compiler_version\":\""
           << __CUDACC_VER_MAJOR__ << '.' << __CUDACC_VER_MINOR__ << '.'
           << __CUDACC_VER_BUILD__ << "\"}";
    return output.str();
}

} // namespace

const char* accumulation_identity(AccumulationMode mode) {
    switch (mode) {
    case AccumulationMode::SourceAtomicV0:
        return "source-atomic-v0";
    case AccumulationMode::GatherDirectedR0:
        return "nuv-gather-directed-r0";
    case AccumulationMode::UniquePairSegmentedO3:
        return "nuv-unique-pair-segmented-o3";
    }
    throw std::invalid_argument("unknown accumulation mode");
}

AccumulationMode parse_accumulation_identity(const std::string& identity) {
    if (identity == "source-atomic-v0") {
        return AccumulationMode::SourceAtomicV0;
    }
    if (identity == "nuv-gather-directed-r0") {
        return AccumulationMode::GatherDirectedR0;
    }
    if (identity == "nuv-unique-pair-segmented-o3") {
        return AccumulationMode::UniquePairSegmentedO3;
    }
    throw std::invalid_argument("unknown accumulation identity: " + identity);
}

const char* handoff_identity(HandoffMode mode) {
    switch (mode) {
    case HandoffMode::CopyV0:
        return "copy-v0";
    case HandoffMode::PointerSwapO1:
        return "pointer-swap-o1";
    }
    throw std::invalid_argument("unknown handoff mode");
}

HandoffMode parse_handoff_identity(const std::string& identity) {
    if (identity == "copy-v0") {
        return HandoffMode::CopyV0;
    }
    if (identity == "pointer-swap-o1") {
        return HandoffMode::PointerSwapO1;
    }
    throw std::invalid_argument("unknown handoff identity: " + identity);
}

const char* term_kernel_identity(TermKernelMode mode) {
    switch (mode) {
    case TermKernelMode::RuntimeV0:
        return "nuv-terms-runtime-v0";
    case TermKernelMode::SpecializedO2:
        return "nuv-terms-specialized-o2";
    }
    throw std::invalid_argument("unknown term-kernel mode");
}

TermKernelMode parse_term_kernel_identity(const std::string& identity) {
    if (identity == "nuv-terms-runtime-v0") {
        return TermKernelMode::RuntimeV0;
    }
    if (identity == "nuv-terms-specialized-o2") {
        return TermKernelMode::SpecializedO2;
    }
    throw std::invalid_argument("unknown term-kernel identity: " + identity);
}

const char* storage_identity(StorageMode mode) {
    switch (mode) {
    case StorageMode::StableSampleV0:
        return "stable-sample-v0";
    case StorageMode::CellSortedO4:
        return "cell-sorted-o4";
    }
    throw std::invalid_argument("unknown storage mode");
}

StorageMode parse_storage_identity(const std::string& identity) {
    if (identity == "stable-sample-v0") {
        return StorageMode::StableSampleV0;
    }
    if (identity == "cell-sorted-o4") {
        return StorageMode::CellSortedO4;
    }
    throw std::invalid_argument("unknown storage identity: " + identity);
}

CommandReport run_cuda_self_test(
    AccumulationMode mode,
    HandoffMode handoff,
    TermKernelMode term_kernels,
    StorageMode storage) {
    const Profile& profile = find_profile("nuv-tiny-oracle.v0");
    const Tolerances tolerances = profile.tolerances;
    const std::vector<Fixture> fixtures = oracle_fixtures();
    std::ostringstream output;
    output << std::setprecision(17);
    output << "{\"schema\":\"nextengine.nonlocal.cuda_oracle_check.v0\""
           << ",\"accumulation_identity\":\"" << accumulation_identity(mode)
           << "\",\"handoff_identity\":\"" << handoff_identity(handoff)
           << "\",\"term_kernel_identity\":\"" << term_kernel_identity(term_kernels)
           << "\",\"storage_identity\":\"" << storage_identity(storage)
           << "\",\"binary_sha256\":\"" << executable_hash()
           << "\",\"command\":\"nonlocal-feasibility --self-test --accumulation "
           << accumulation_identity(mode) << " --handoff " << handoff_identity(handoff)
           << " --term-kernels " << term_kernel_identity(term_kernels)
           << " --storage " << storage_identity(storage)
           << "\",\"profile_id\":\"" << profile.id
           << "\",\"profile_sha256\":\"" << sha256_hex(canonical_profile_json(profile))
           << "\",\"device\":" << device_json()
           << ",\"normalization\":{\"energy\":\"max(N*m*dx^2/dt^2,max_abs_cpu_energy)\""
           << ",\"source\":\"max(dx,max_abs_cpu_source)\""
           << ",\"matrix\":\"max(1,max_abs_cpu_matrix)\"},\"cases\":[";
    bool all_passed = true;
    for (std::size_t index = 0; index < fixtures.size(); ++index) {
        const Fixture& fixture = fixtures[index];
        const OracleResult cpu = run_cpu_oracle(fixture);
        CudaBaseline baseline(fixture, mode, handoff, term_kernels, storage);
        const CapturedRun gpu = baseline.execute(true);
        CapturedRun reused_second = gpu;
        CapturedRun reused_third = gpu;
        if (handoff == HandoffMode::PointerSwapO1) {
            reused_second = baseline.execute(true);
            reused_third = baseline.execute(true);
        }
        const Comparison comparison = compare_results(fixture, cpu, gpu.state, tolerances);
        const Comparison reuse_second =
            compare_results(fixture, gpu.state, reused_second.state, tolerances);
        const Comparison reuse_third =
            compare_results(fixture, gpu.state, reused_third.state, tolerances);
        const std::string output_digest = ordered_output_digest(gpu.state);
        const std::string second_digest = ordered_output_digest(reused_second.state);
        const std::string third_digest = ordered_output_digest(reused_third.state);
        const std::string topology_digest = csr_digest(gpu);
        const bool reused_instance_exact = output_digest == second_digest
            && output_digest == third_digest && gpu.offsets == reused_second.offsets
            && gpu.offsets == reused_third.offsets && gpu.neighbors == reused_second.neighbors
            && gpu.neighbors == reused_third.neighbors && reuse_second.passed
            && reuse_third.passed;

        CapturedRun copy_reference = gpu;
        if (handoff == HandoffMode::PointerSwapO1
            && term_kernels == TermKernelMode::RuntimeV0) {
            CudaBaseline copy_baseline(
                fixture, mode, HandoffMode::CopyV0, TermKernelMode::RuntimeV0);
            copy_reference = copy_baseline.execute(true);
        }
        const Comparison copy_comparison =
            compare_results(fixture, copy_reference.state, gpu.state, tolerances);
        const bool copy_baseline_exact = ordered_output_digest(copy_reference.state) == output_digest
            && copy_reference.offsets == gpu.offsets && copy_reference.neighbors == gpu.neighbors
            && copy_reference.device_memory_bytes == gpu.device_memory_bytes
            && copy_comparison.passed;

        const bool layout_candidate = mode == AccumulationMode::UniquePairSegmentedO3;
        const bool storage_candidate = storage == StorageMode::CellSortedO4;
        CapturedRun term_reference = gpu;
        if (term_kernels == TermKernelMode::SpecializedO2
            && !layout_candidate && !storage_candidate) {
            CudaBaseline runtime_baseline(
                fixture, mode, handoff, TermKernelMode::RuntimeV0);
            term_reference = runtime_baseline.execute(true);
        }
        const Comparison term_correspondence =
            compare_results(fixture, term_reference.state, gpu.state, tolerances);
        const bool term_baseline_exact =
            ordered_output_digest(term_reference.state) == output_digest;
        const bool term_baseline_correspondence = term_correspondence.passed
            && term_reference.offsets == gpu.offsets
            && term_reference.neighbors == gpu.neighbors
            && term_reference.device_memory_bytes == gpu.device_memory_bytes;
        CapturedRun layout_reference = gpu;
        if (layout_candidate) {
            CudaBaseline gather_baseline(
                fixture, AccumulationMode::GatherDirectedR0,
                HandoffMode::PointerSwapO1, TermKernelMode::SpecializedO2);
            layout_reference = gather_baseline.execute(true);
        }
        const Comparison layout_comparison =
            compare_results(fixture, layout_reference.state, gpu.state, tolerances);
        const bool layout_baseline_exact =
            ordered_output_digest(layout_reference.state) == output_digest;
        const bool layout_baseline_correspondence = layout_comparison.passed
            && layout_reference.offsets == gpu.offsets
            && layout_reference.neighbors == gpu.neighbors;
        CapturedRun storage_reference = gpu;
        if (storage_candidate) {
            CudaBaseline stable_baseline(
                fixture, AccumulationMode::GatherDirectedR0,
                HandoffMode::PointerSwapO1, TermKernelMode::SpecializedO2,
                StorageMode::StableSampleV0);
            storage_reference = stable_baseline.execute(true);
        }
        const bool storage_baseline_exact = !storage_candidate
            || (ordered_output_digest(storage_reference.state) == output_digest
                && storage_reference.offsets == gpu.offsets
                && storage_reference.neighbors == gpu.neighbors);
        const bool neighbors_passed = exact_fixture_neighbors(fixture, gpu)
            && valid_symmetric_neighbors(gpu, fixture.particles.size())
            && exact_fixture_neighbors(fixture, reused_second)
            && exact_fixture_neighbors(fixture, reused_third);
        const bool momentum_passed = gpu.state.normalized_momentum_residual
                <= tolerances.normalized_momentum_residual
            && reused_second.state.normalized_momentum_residual
                <= tolerances.normalized_momentum_residual
            && reused_third.state.normalized_momentum_residual
                <= tolerances.normalized_momentum_residual;
        const bool finite_passed = finite_state(gpu.state) && finite_state(reused_second.state)
            && finite_state(reused_third.state);
        const bool local_solve_passed = !gpu.local_solve_failed
            && !reused_second.local_solve_failed && !reused_third.local_solve_failed;
        const bool reverse_map_passed = gpu.reverse_map_valid
            && reused_second.reverse_map_valid && reused_third.reverse_map_valid;
        const bool storage_map_passed = gpu.storage_map_valid
            && reused_second.storage_map_valid && reused_third.storage_map_valid;
        const bool passed = comparison.passed && neighbors_passed && momentum_passed
            && finite_passed && local_solve_passed && reverse_map_passed && storage_map_passed
            && (handoff != HandoffMode::PointerSwapO1 || reused_instance_exact)
            && (term_kernels != TermKernelMode::RuntimeV0
                || handoff != HandoffMode::PointerSwapO1 || copy_baseline_exact)
            && (term_kernels != TermKernelMode::SpecializedO2 || layout_candidate
                || storage_candidate
                || term_baseline_correspondence);
        const bool final_passed = passed
            && (!layout_candidate || layout_baseline_correspondence)
            && storage_baseline_exact;
        all_passed = all_passed && final_passed;
        if (index != 0) {
            output << ',';
        }
        output << "{\"name\":\"" << fixture.name << "\",\"input_sha256\":\""
               << fixture_input_hash(fixture) << "\",\"passed\":"
               << (final_passed ? "true" : "false") << ",\"neighbors_passed\":"
               << (neighbors_passed ? "true" : "false");
        if (handoff == HandoffMode::PointerSwapO1) {
            output << ",\"reused_instance_runs\":3,\"reused_instance_exact\":"
                   << (reused_instance_exact ? "true" : "false");
            if (term_kernels == TermKernelMode::RuntimeV0) {
                output << ",\"copy_baseline_exact\":"
                       << (copy_baseline_exact ? "true" : "false");
            } else {
                output << ",\"copy_baseline_exact\":null";
            }
            output << ",\"ordered_output_sha256\":[\"" << output_digest << "\",\""
                   << second_digest << "\",\"" << third_digest << "\"]";
        } else {
            output << ",\"reused_instance_runs\":1,\"reused_instance_exact\":null"
                      ",\"copy_baseline_exact\":null,\"ordered_output_sha256\":[\""
                   << output_digest << "\"]";
        }
        if (term_kernels == TermKernelMode::SpecializedO2
            && !layout_candidate && !storage_candidate) {
            output << ",\"term_baseline_correspondence\":"
                   << (term_baseline_correspondence ? "true" : "false")
                   << ",\"term_baseline_exact\":"
                   << (term_baseline_exact ? "true" : "false")
                   << ",\"term_baseline_output_sha256\":\""
                   << ordered_output_digest(term_reference.state) << "\"";
        } else {
            output << ",\"term_baseline_correspondence\":null"
                      ",\"term_baseline_exact\":null"
                      ",\"term_baseline_output_sha256\":null";
        }
        output << ",\"layout_baseline_correspondence\":";
        if (layout_candidate) {
            output << (layout_baseline_correspondence ? "true" : "false")
                   << ",\"layout_baseline_exact\":"
                   << (layout_baseline_exact ? "true" : "false")
                   << ",\"layout_baseline_output_sha256\":\""
                   << ordered_output_digest(layout_reference.state) << "\"";
        } else {
            output << "null,\"layout_baseline_exact\":null"
                      ",\"layout_baseline_output_sha256\":null";
        }
        output << ",\"storage_baseline_exact\":";
        if (storage_candidate) {
            output << (storage_baseline_exact ? "true" : "false")
                   << ",\"storage_baseline_output_sha256\":\""
                   << ordered_output_digest(storage_reference.state) << "\"";
        } else {
            output << "null,\"storage_baseline_output_sha256\":null";
        }
        output << ",\"csr_sha256\":\"" << topology_digest << "\",\"finite\":"
               << (finite_passed ? "true" : "false")
               << ",\"local_solve_failed\":"
               << (local_solve_passed ? "false" : "true")
               << ",\"directed_pairs\":" << gpu.state.directed_pairs
               << ",\"self_slots\":" << gpu.self_slots
               << ",\"nonself_directed_samples\":" << gpu.nonself_directed_samples
               << ",\"unique_pairs\":" << gpu.unique_pairs
               << ",\"pair_evaluations_per_active_term\":"
               << gpu.pair_evaluations_per_active_term
               << ",\"endpoint_fragments_per_active_term\":"
               << gpu.endpoint_fragments_per_active_term
               << ",\"reverse_map_valid\":"
               << (gpu.reverse_map_valid ? "true" : "false")
               << ",\"layout_memory_bytes\":" << gpu.layout_memory_bytes
               << ",\"layout_setup_ms\":" << gpu.layout_setup_ms
               << ",\"storage_map_valid\":"
               << (gpu.storage_map_valid ? "true" : "false")
               << ",\"storage_memory_bytes\":" << gpu.storage_memory_bytes
               << ",\"storage_setup_ms\":" << gpu.storage_setup_ms
               << ",\"storage_to_sample_sha256\":\""
               << gpu.storage_to_sample_sha256
               << "\",\"sample_to_storage_sha256\":\""
               << gpu.sample_to_storage_sha256
               << "\",\"physical_csr_sha256\":\"" << gpu.physical_csr_sha256
               << "\",\"mean_neighbor_storage_distance\":"
               << gpu.mean_neighbor_storage_distance
               << ",\"p95_neighbor_storage_distance\":"
               << gpu.p95_neighbor_storage_distance
               << ",\"maximum_degree\":" << gpu.state.maximum_degree
               << ",\"normalized_momentum_residual\":"
               << gpu.state.normalized_momentum_residual << ",\"scales\":{\"energy\":"
               << comparison.energy_scale << ",\"source\":" << comparison.source_scale
               << ",\"matrix\":" << comparison.matrix_scale << "},\"discrepancy\":{\"density\":";
        append_discrepancy(output, comparison.density);
        output << ",\"energy\":";
        append_discrepancy(output, comparison.energy);
        output << ",\"source\":";
        append_discrepancy(output, comparison.source);
        output << ",\"matrix\":";
        append_discrepancy(output, comparison.matrix);
        output << ",\"position\":";
        append_discrepancy(output, comparison.position);
        output << ",\"velocity\":";
        append_discrepancy(output, comparison.velocity);
        output << "},\"timing\":";
        append_timing(output, gpu.timing);
        output << '}';
    }
    output << ']';
    if (mode == AccumulationMode::UniquePairSegmentedO3) {
        Fixture bulk_only = fixtures.at(6);
        bulk_only.name = "o3_bulk_only_specialization_smoke";
        bulk_only.terms = {false, true, false, false};
        bulk_only.lambda = 1.5;
        bulk_only.mu = 0.0;
        const OracleResult bulk_cpu = run_cpu_oracle(bulk_only);
        CudaBaseline bulk_baseline(bulk_only, mode, handoff, term_kernels);
        const CapturedRun bulk_gpu = bulk_baseline.execute(true);
        const bool bulk_passed = compare_results(
            bulk_only, bulk_cpu, bulk_gpu.state, tolerances).passed
            && exact_fixture_neighbors(bulk_only, bulk_gpu)
            && bulk_gpu.reverse_map_valid && !bulk_gpu.local_solve_failed;
        all_passed = all_passed && bulk_passed;
        output << ",\"specialization_smoke\":{\"bulk_only\":"
               << (bulk_passed ? "true" : "false")
               << ",\"shear_only_case\":\"viscosity_shear_pair\""
               << ",\"bulk_shear_cases\":\"tiny_closed_box_iter1..4\""
               << ",\"bulk_only_output_sha256\":\""
               << ordered_output_digest(bulk_gpu.state) << "\"}";
    } else {
        output << ",\"specialization_smoke\":null";
    }
    output << ",\"status\":\"" << (all_passed ? "PASS" : "FAIL") << "\"}";
    return {all_passed, output.str()};
}

CommandReport run_cuda_check(
    const Profile& profile,
    int iterations,
    AccumulationMode mode,
    HandoffMode handoff,
    TermKernelMode term_kernels,
    StorageMode storage) {
    if (profile.id == "nuv-tiny-oracle.v0") {
        return run_cuda_self_test(mode, handoff, term_kernels, storage);
    }
    if (iterations < 1 || iterations > 100) {
        throw std::invalid_argument("check iterations must be inside 1..=100");
    }
    const Fixture fixture = performance_fixture(profile, iterations);
    CudaBaseline baseline(fixture, mode, handoff, term_kernels, storage);
    const CapturedRun first = baseline.execute(true);
    const CapturedRun second = baseline.execute(true);
    const Comparison repeated =
        compare_results(fixture, first.state, second.state, profile.tolerances);
    const bool repeated_neighbors_identical =
        first.offsets == second.offsets && first.neighbors == second.neighbors;
    const bool neighbors_passed = repeated_neighbors_identical
        && valid_symmetric_neighbors(first, fixture.particles.size())
        && first.state.directed_pairs <= profile.max_directed_pairs
        && first.state.maximum_degree <= profile.max_neighbors;
    const std::string first_digest = ordered_output_digest(first.state);
    const std::string second_digest = ordered_output_digest(second.state);
    const bool reused_instance_exact = first_digest == second_digest
        && repeated_neighbors_identical && repeated.passed;

    CapturedRun copy_reference = first;
    if (handoff == HandoffMode::PointerSwapO1
        && term_kernels == TermKernelMode::RuntimeV0) {
        CudaBaseline copy_baseline(
            fixture, mode, HandoffMode::CopyV0, TermKernelMode::RuntimeV0);
        copy_reference = copy_baseline.execute(true);
    }
    const Comparison copy_correspondence =
        compare_results(fixture, copy_reference.state, first.state, profile.tolerances);
    const bool copy_baseline_exact = ordered_output_digest(copy_reference.state) == first_digest
        && copy_reference.offsets == first.offsets && copy_reference.neighbors == first.neighbors
        && copy_reference.device_memory_bytes == first.device_memory_bytes
        && copy_correspondence.passed;
    const bool layout_candidate = mode == AccumulationMode::UniquePairSegmentedO3;
    const bool storage_candidate = storage == StorageMode::CellSortedO4;
    CapturedRun term_reference = first;
    if (term_kernels == TermKernelMode::SpecializedO2
        && !layout_candidate && !storage_candidate) {
        CudaBaseline runtime_baseline(fixture, mode, handoff, TermKernelMode::RuntimeV0);
        term_reference = runtime_baseline.execute(true);
    }
    const Comparison term_correspondence =
        compare_results(fixture, term_reference.state, first.state, profile.tolerances);
    const bool term_baseline_exact =
        ordered_output_digest(term_reference.state) == first_digest;
    const bool term_baseline_correspondence = term_correspondence.passed
        && term_reference.offsets == first.offsets
        && term_reference.neighbors == first.neighbors
        && term_reference.device_memory_bytes == first.device_memory_bytes;
    CapturedRun layout_reference = first;
    if (layout_candidate) {
        CudaBaseline gather_baseline(
            fixture, AccumulationMode::GatherDirectedR0,
            HandoffMode::PointerSwapO1, TermKernelMode::SpecializedO2);
        layout_reference = gather_baseline.execute(true);
    }
    const Comparison layout_comparison =
        compare_results(fixture, layout_reference.state, first.state, profile.tolerances);
    const bool layout_baseline_exact =
        ordered_output_digest(layout_reference.state) == first_digest;
    const bool layout_baseline_correspondence = layout_comparison.passed
        && layout_reference.offsets == first.offsets
        && layout_reference.neighbors == first.neighbors;
    CapturedRun storage_reference = first;
    if (storage_candidate) {
        CudaBaseline stable_baseline(
            fixture, AccumulationMode::GatherDirectedR0,
            HandoffMode::PointerSwapO1, TermKernelMode::SpecializedO2,
            StorageMode::StableSampleV0);
        storage_reference = stable_baseline.execute(true);
    }
    const bool storage_baseline_exact = !storage_candidate
        || (ordered_output_digest(storage_reference.state) == first_digest
            && storage_reference.offsets == first.offsets
            && storage_reference.neighbors == first.neighbors);
    const bool passed = finite_state(first.state) && finite_state(second.state)
        && !first.local_solve_failed && !second.local_solve_failed && neighbors_passed
        && first.reverse_map_valid && second.reverse_map_valid
        && first.storage_map_valid && second.storage_map_valid
        && first.state.normalized_momentum_residual
            <= profile.tolerances.normalized_momentum_residual
        && second.state.normalized_momentum_residual
            <= profile.tolerances.normalized_momentum_residual
        && repeated.passed
        && (handoff != HandoffMode::PointerSwapO1 || reused_instance_exact)
        && (term_kernels != TermKernelMode::RuntimeV0
            || handoff != HandoffMode::PointerSwapO1 || copy_baseline_exact)
        && (term_kernels != TermKernelMode::SpecializedO2 || layout_candidate
            || storage_candidate
            || term_baseline_correspondence)
        && (!layout_candidate || layout_baseline_correspondence)
        && storage_baseline_exact;

    std::ostringstream output;
    output << std::setprecision(17);
    output << "{\"schema\":\"nextengine.nonlocal.cuda_full_control.v0\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"profile_id\":\"" << profile.id
           << "\",\"accumulation_identity\":\"" << accumulation_identity(mode)
           << "\",\"handoff_identity\":\"" << handoff_identity(handoff)
           << "\",\"term_kernel_identity\":\"" << term_kernel_identity(term_kernels)
           << "\",\"storage_identity\":\"" << storage_identity(storage)
           << "\",\"binary_sha256\":\"" << executable_hash()
           << "\",\"command\":\"nonlocal-feasibility --check " << profile.id
           << " --iterations " << iterations << " --accumulation "
           << accumulation_identity(mode) << " --handoff " << handoff_identity(handoff)
           << " --term-kernels " << term_kernel_identity(term_kernels)
           << " --storage " << storage_identity(storage)
           << "\",\"profile_sha256\":\"" << sha256_hex(canonical_profile_json(profile))
           << "\",\"input_sha256\":\"" << fixture_input_hash(fixture)
           << "\",\"iterations\":" << iterations << ",\"samples\":"
           << fixture.particles.size() << ",\"directed_pairs\":"
           << first.state.directed_pairs << ",\"maximum_degree\":"
           << first.state.maximum_degree << ",\"neighbors_passed\":"
           << (neighbors_passed ? "true" : "false")
           << ",\"repeated_neighbors_identical\":"
           << (repeated_neighbors_identical ? "true" : "false") << ",\"finite\":"
           << (finite_state(first.state) && finite_state(second.state) ? "true" : "false")
           << ",\"local_solve_failed\":"
           << (first.local_solve_failed || second.local_solve_failed ? "true" : "false")
           << ",\"normalized_momentum_residual\":["
           << first.state.normalized_momentum_residual << ','
           << second.state.normalized_momentum_residual
           << "],\"reused_instance_exact\":"
           << (reused_instance_exact ? "true" : "false") << ",\"copy_baseline_exact\":";
    if (term_kernels == TermKernelMode::RuntimeV0) {
        output << (copy_baseline_exact ? "true" : "false");
    } else {
        output << "null";
    }
    output << ",\"term_baseline_correspondence\":";
    if (term_kernels == TermKernelMode::SpecializedO2
        && !layout_candidate && !storage_candidate) {
        output << (term_baseline_correspondence ? "true" : "false");
    } else {
        output << "null";
    }
    output << ",\"term_baseline_exact\":";
    if (term_kernels == TermKernelMode::SpecializedO2
        && !layout_candidate && !storage_candidate) {
        output << (term_baseline_exact ? "true" : "false");
    } else {
        output << "null";
    }
    output << ",\"term_baseline_output_sha256\":";
    if (term_kernels == TermKernelMode::SpecializedO2
        && !layout_candidate && !storage_candidate) {
        output << '"' << ordered_output_digest(term_reference.state) << '"';
    } else {
        output << "null";
    }
    output << ",\"layout_baseline_correspondence\":";
    if (layout_candidate) {
        output << (layout_baseline_correspondence ? "true" : "false")
               << ",\"layout_baseline_exact\":"
               << (layout_baseline_exact ? "true" : "false")
               << ",\"layout_baseline_output_sha256\":\""
               << ordered_output_digest(layout_reference.state) << "\"";
    } else {
        output << "null,\"layout_baseline_exact\":null"
                  ",\"layout_baseline_output_sha256\":null";
    }
    output << ",\"storage_baseline_exact\":";
    if (storage_candidate) {
        output << (storage_baseline_exact ? "true" : "false")
               << ",\"storage_baseline_output_sha256\":\""
               << ordered_output_digest(storage_reference.state) << "\"";
    } else {
        output << "null,\"storage_baseline_output_sha256\":null";
    }
    output << ",\"layout_baseline_discrepancy\":{\"density\":";
    append_discrepancy(output, layout_comparison.density);
    output << ",\"energy\":";
    append_discrepancy(output, layout_comparison.energy);
    output << ",\"source\":";
    append_discrepancy(output, layout_comparison.source);
    output << ",\"matrix\":";
    append_discrepancy(output, layout_comparison.matrix);
    output << ",\"position\":";
    append_discrepancy(output, layout_comparison.position);
    output << ",\"velocity\":";
    append_discrepancy(output, layout_comparison.velocity);
    output << '}';
    output << ",\"ordered_output_sha256\":[\"" << first_digest << "\",\""
           << second_digest
           << "\"],\"repeated_output_discrepancy\":{\"density\":";
    append_discrepancy(output, repeated.density);
    output << ",\"energy\":";
    append_discrepancy(output, repeated.energy);
    output << ",\"source\":";
    append_discrepancy(output, repeated.source);
    output << ",\"matrix\":";
    append_discrepancy(output, repeated.matrix);
    output << ",\"position\":";
    append_discrepancy(output, repeated.position);
    output << ",\"velocity\":";
    append_discrepancy(output, repeated.velocity);
    output << "},\"device_memory_bytes\":" << first.device_memory_bytes
           << ",\"layout_memory_bytes\":" << first.layout_memory_bytes
           << ",\"layout_setup_ms\":" << first.layout_setup_ms
           << ",\"storage_map_valid\":"
           << (first.storage_map_valid && second.storage_map_valid ? "true" : "false")
           << ",\"storage_memory_bytes\":" << first.storage_memory_bytes
           << ",\"storage_setup_ms\":" << first.storage_setup_ms
           << ",\"storage_to_sample_sha256\":\""
           << first.storage_to_sample_sha256
           << "\",\"sample_to_storage_sha256\":\""
           << first.sample_to_storage_sha256
           << "\",\"physical_csr_sha256\":\"" << first.physical_csr_sha256
           << "\",\"mean_neighbor_storage_distance\":"
           << first.mean_neighbor_storage_distance
           << ",\"p95_neighbor_storage_distance\":"
           << first.p95_neighbor_storage_distance
           << ",\"reverse_map_valid\":"
           << (first.reverse_map_valid && second.reverse_map_valid ? "true" : "false")
           << ",\"self_slots\":" << first.self_slots
           << ",\"nonself_directed_samples\":" << first.nonself_directed_samples
           << ",\"unique_pairs\":" << first.unique_pairs
           << ",\"pair_evaluations_per_active_term\":"
           << first.pair_evaluations_per_active_term
           << ",\"endpoint_fragments_per_active_term\":"
           << first.endpoint_fragments_per_active_term
           << ",\"device\":" << device_json() << ",\"first_timing\":";
    append_timing(output, first.timing);
    output << ",\"second_timing\":";
    append_timing(output, second.timing);
    output << '}';
    return {passed, output.str()};
}

CommandReport run_cuda_repeatability(
    const Profile& profile,
    int iterations,
    int runs,
    AccumulationMode mode,
    HandoffMode handoff,
    TermKernelMode term_kernels,
    StorageMode storage) {
    if (profile.id == "nuv-tiny-oracle.v0") {
        throw std::invalid_argument("repeatability requires a fixed performance profile");
    }
    if (iterations < 1 || iterations > 100) {
        throw std::invalid_argument("repeatability iterations must be inside 1..=100");
    }
    if (runs < 2 || runs > 100) {
        throw std::invalid_argument("repeatability runs must be inside 2..=100");
    }

    const Fixture fixture = performance_fixture(profile, iterations);
    CapturedRun first;
    std::string first_output_digest;
    std::string first_csr_digest;
    std::size_t first_memory_bytes = 0;
    std::vector<std::string> output_digests;
    std::vector<std::string> csr_digests;
    std::vector<double> total_timings;
    std::vector<double> momentum_residuals;
    output_digests.reserve(static_cast<std::size_t>(runs));
    csr_digests.reserve(static_cast<std::size_t>(runs));
    total_timings.reserve(static_cast<std::size_t>(runs));
    momentum_residuals.reserve(static_cast<std::size_t>(runs));

    bool exact_output_digest = true;
    bool exact_csr = true;
    bool memory_identical = true;
    bool correspondence_passed = true;
    bool finite_passed = true;
    bool local_solve_passed = true;
    bool momentum_passed = true;
    bool topology_passed = true;
    bool reverse_map_passed = true;
    bool storage_map_passed = true;
    for (int run = 0; run < runs; ++run) {
        CapturedRun captured;
        {
            // Deliberately reconstruct and reallocate the complete baseline for
            // every RC1 cold repeat, then reset from the immutable fixture.
            CudaBaseline baseline(fixture, mode, handoff, term_kernels, storage);
            captured = baseline.execute(true);
        }
        const std::string output_digest = ordered_output_digest(captured.state);
        const std::string current_csr_digest = csr_digest(captured);
        output_digests.push_back(output_digest);
        csr_digests.push_back(current_csr_digest);
        total_timings.push_back(captured.timing.total);
        momentum_residuals.push_back(captured.state.normalized_momentum_residual);

        finite_passed = finite_passed && finite_state(captured.state);
        local_solve_passed = local_solve_passed && !captured.local_solve_failed;
        momentum_passed = momentum_passed
            && captured.state.normalized_momentum_residual
                <= profile.tolerances.normalized_momentum_residual;
        topology_passed = topology_passed
            && valid_symmetric_neighbors(captured, fixture.particles.size())
            && captured.state.directed_pairs <= profile.max_directed_pairs
            && captured.state.maximum_degree <= profile.max_neighbors;
        reverse_map_passed = reverse_map_passed && captured.reverse_map_valid;
        storage_map_passed = storage_map_passed && captured.storage_map_valid;

        if (run == 0) {
            first = captured;
            first_output_digest = output_digest;
            first_csr_digest = current_csr_digest;
            first_memory_bytes = captured.device_memory_bytes;
        } else {
            exact_output_digest = exact_output_digest && output_digest == first_output_digest;
            exact_csr = exact_csr && current_csr_digest == first_csr_digest
                && captured.offsets == first.offsets && captured.neighbors == first.neighbors;
            memory_identical =
                memory_identical && captured.device_memory_bytes == first_memory_bytes;
            correspondence_passed = correspondence_passed
                && compare_results(fixture, first.state, captured.state, profile.tolerances).passed;
        }
    }

    CapturedRun copy_reference = first;
    if (handoff == HandoffMode::PointerSwapO1
        && term_kernels == TermKernelMode::RuntimeV0) {
        CudaBaseline copy_baseline(
            fixture, mode, HandoffMode::CopyV0, TermKernelMode::RuntimeV0);
        copy_reference = copy_baseline.execute(true);
    }
    const Comparison copy_correspondence =
        compare_results(fixture, copy_reference.state, first.state, profile.tolerances);
    const bool copy_baseline_exact = ordered_output_digest(copy_reference.state)
            == first_output_digest
        && copy_reference.offsets == first.offsets && copy_reference.neighbors == first.neighbors
        && copy_reference.device_memory_bytes == first.device_memory_bytes
        && copy_correspondence.passed;

    const bool layout_candidate = mode == AccumulationMode::UniquePairSegmentedO3;
    const bool storage_candidate = storage == StorageMode::CellSortedO4;
    CapturedRun term_reference = first;
    if (term_kernels == TermKernelMode::SpecializedO2
        && !layout_candidate && !storage_candidate) {
        CudaBaseline runtime_baseline(fixture, mode, handoff, TermKernelMode::RuntimeV0);
        term_reference = runtime_baseline.execute(true);
    }
    const Comparison term_correspondence =
        compare_results(fixture, term_reference.state, first.state, profile.tolerances);
    const bool term_baseline_exact =
        ordered_output_digest(term_reference.state) == first_output_digest;
    const bool term_baseline_correspondence = term_correspondence.passed
        && term_reference.offsets == first.offsets
        && term_reference.neighbors == first.neighbors
        && term_reference.device_memory_bytes == first.device_memory_bytes;
    CapturedRun layout_reference = first;
    if (layout_candidate) {
        CudaBaseline gather_baseline(
            fixture, AccumulationMode::GatherDirectedR0,
            HandoffMode::PointerSwapO1, TermKernelMode::SpecializedO2);
        layout_reference = gather_baseline.execute(true);
    }
    const Comparison layout_comparison =
        compare_results(fixture, layout_reference.state, first.state, profile.tolerances);
    const bool layout_baseline_exact =
        ordered_output_digest(layout_reference.state) == first_output_digest;
    const bool layout_baseline_correspondence = layout_comparison.passed
        && layout_reference.offsets == first.offsets
        && layout_reference.neighbors == first.neighbors;
    CapturedRun storage_reference = first;
    if (storage_candidate) {
        CudaBaseline stable_baseline(
            fixture, AccumulationMode::GatherDirectedR0,
            HandoffMode::PointerSwapO1, TermKernelMode::SpecializedO2,
            StorageMode::StableSampleV0);
        storage_reference = stable_baseline.execute(true);
    }
    const bool storage_baseline_exact = !storage_candidate
        || (ordered_output_digest(storage_reference.state) == first_output_digest
            && storage_reference.offsets == first.offsets
            && storage_reference.neighbors == first.neighbors);

    const bool passed = exact_output_digest && exact_csr && memory_identical
        && correspondence_passed && finite_passed && local_solve_passed && momentum_passed
        && topology_passed && reverse_map_passed && storage_map_passed
        && (term_kernels != TermKernelMode::RuntimeV0
            || handoff != HandoffMode::PointerSwapO1 || copy_baseline_exact)
        && (term_kernels != TermKernelMode::SpecializedO2 || layout_candidate
            || storage_candidate
            || term_baseline_correspondence)
        && (!layout_candidate || layout_baseline_correspondence)
        && storage_baseline_exact;
    std::ostringstream output;
    output << std::setprecision(17);
    output << "{\"schema\":\"nextengine.nonlocal.cuda_repeatability.v0\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"profile_id\":\"" << profile.id
           << "\",\"accumulation_identity\":\"" << accumulation_identity(mode)
           << "\",\"handoff_identity\":\"" << handoff_identity(handoff)
           << "\",\"term_kernel_identity\":\"" << term_kernel_identity(term_kernels)
           << "\",\"storage_identity\":\"" << storage_identity(storage)
           << "\",\"binary_sha256\":\"" << executable_hash()
           << "\",\"command\":\"nonlocal-feasibility --repeatability " << profile.id
           << " --iterations " << iterations << " --runs " << runs << " --accumulation "
           << accumulation_identity(mode) << " --handoff " << handoff_identity(handoff)
           << " --term-kernels " << term_kernel_identity(term_kernels)
           << " --storage " << storage_identity(storage)
           << "\",\"profile_sha256\":\"" << sha256_hex(canonical_profile_json(profile))
           << "\",\"input_sha256\":\"" << fixture_input_hash(fixture)
           << "\",\"iterations\":" << iterations << ",\"runs\":" << runs
           << ",\"reset_to_fixture_each_run\":true,\"samples\":"
           << fixture.particles.size() << ",\"directed_pairs\":"
           << first.state.directed_pairs << ",\"maximum_degree\":"
           << first.state.maximum_degree << ",\"device_memory_bytes\":"
           << first_memory_bytes << ",\"exact_output_digest\":"
           << (exact_output_digest ? "true" : "false") << ",\"exact_csr\":"
           << (exact_csr ? "true" : "false") << ",\"memory_identical\":"
           << (memory_identical ? "true" : "false") << ",\"copy_baseline_exact\":";
    if (term_kernels == TermKernelMode::RuntimeV0) {
        output << (copy_baseline_exact ? "true" : "false");
    } else {
        output << "null";
    }
    output << ",\"term_baseline_correspondence\":";
    if (term_kernels == TermKernelMode::SpecializedO2
        && !layout_candidate && !storage_candidate) {
        output << (term_baseline_correspondence ? "true" : "false");
    } else {
        output << "null";
    }
    output << ",\"term_baseline_exact\":";
    if (term_kernels == TermKernelMode::SpecializedO2
        && !layout_candidate && !storage_candidate) {
        output << (term_baseline_exact ? "true" : "false");
    } else {
        output << "null";
    }
    output << ",\"term_baseline_output_sha256\":";
    if (term_kernels == TermKernelMode::SpecializedO2
        && !layout_candidate && !storage_candidate) {
        output << '"' << ordered_output_digest(term_reference.state) << '"';
    } else {
        output << "null";
    }
    output << ",\"layout_baseline_correspondence\":";
    if (layout_candidate) {
        output << (layout_baseline_correspondence ? "true" : "false")
               << ",\"layout_baseline_exact\":"
               << (layout_baseline_exact ? "true" : "false")
               << ",\"layout_baseline_output_sha256\":\""
               << ordered_output_digest(layout_reference.state) << "\"";
    } else {
        output << "null,\"layout_baseline_exact\":null"
                  ",\"layout_baseline_output_sha256\":null";
    }
    output << ",\"storage_baseline_exact\":";
    if (storage_candidate) {
        output << (storage_baseline_exact ? "true" : "false")
               << ",\"storage_baseline_output_sha256\":\""
               << ordered_output_digest(storage_reference.state) << "\"";
    } else {
        output << "null,\"storage_baseline_output_sha256\":null";
    }
    output << ",\"correspondence_passed\":"
           << (correspondence_passed ? "true" : "false") << ",\"finite\":"
           << (finite_passed ? "true" : "false") << ",\"local_solve_failed\":"
           << (local_solve_passed ? "false" : "true") << ",\"momentum_passed\":"
           << (momentum_passed ? "true" : "false") << ",\"topology_passed\":"
           << (topology_passed ? "true" : "false") << ",\"reverse_map_valid\":"
           << (reverse_map_passed ? "true" : "false")
           << ",\"storage_map_valid\":" << (storage_map_passed ? "true" : "false")
           << ",\"layout_memory_bytes\":" << first.layout_memory_bytes
           << ",\"layout_setup_ms\":" << first.layout_setup_ms
           << ",\"storage_memory_bytes\":" << first.storage_memory_bytes
           << ",\"storage_setup_ms\":" << first.storage_setup_ms
           << ",\"storage_to_sample_sha256\":\""
           << first.storage_to_sample_sha256
           << "\",\"sample_to_storage_sha256\":\""
           << first.sample_to_storage_sha256
           << "\",\"physical_csr_sha256\":\"" << first.physical_csr_sha256
           << "\",\"mean_neighbor_storage_distance\":"
           << first.mean_neighbor_storage_distance
           << ",\"p95_neighbor_storage_distance\":"
           << first.p95_neighbor_storage_distance
           << ",\"self_slots\":" << first.self_slots
           << ",\"nonself_directed_samples\":" << first.nonself_directed_samples
           << ",\"unique_pairs\":" << first.unique_pairs
           << ",\"pair_evaluations_per_active_term\":"
           << first.pair_evaluations_per_active_term
           << ",\"endpoint_fragments_per_active_term\":"
           << first.endpoint_fragments_per_active_term << ",\"csr_sha256\":\""
           << first_csr_digest
           << "\",\"ordered_output_digest_fields\":[\"density\",\"source\",\"matrix\","
              "\"final_position\",\"final_velocity\"],\"ordered_output_sha256\":[";
    for (std::size_t index = 0; index < output_digests.size(); ++index) {
        if (index != 0) {
            output << ',';
        }
        output << '\"' << output_digests[index] << '\"';
    }
    output << "],\"csr_sha256_per_run\":[";
    for (std::size_t index = 0; index < csr_digests.size(); ++index) {
        if (index != 0) {
            output << ',';
        }
        output << '\"' << csr_digests[index] << '\"';
    }
    output << "],\"normalized_momentum_residual\":[";
    for (std::size_t index = 0; index < momentum_residuals.size(); ++index) {
        if (index != 0) {
            output << ',';
        }
        output << momentum_residuals[index];
    }
    output << "],\"raw_total_ms\":[";
    for (std::size_t index = 0; index < total_timings.size(); ++index) {
        if (index != 0) {
            output << ',';
        }
        output << total_timings[index];
    }
    output << "],\"device\":" << device_json() << '}';
    return {passed, output.str()};
}

CommandReport run_cuda_benchmark(
    const Profile& profile,
    int warmup,
    int runs,
    AccumulationMode mode,
    HandoffMode handoff,
    TermKernelMode term_kernels,
    StorageMode storage) {
    if (profile.id == "nuv-tiny-oracle.v0") {
        throw std::invalid_argument("benchmark requires a fixed performance profile");
    }
    if (warmup < 0 || warmup > 100 || runs < 1 || runs > 1000) {
        throw std::invalid_argument("benchmark counts exceed bounded limits");
    }
    const CommandReport self_test = run_cuda_self_test(mode, handoff, term_kernels, storage);
    if (!self_test.passed) {
        std::ostringstream failure;
        failure << "{\"schema\":\"nextengine.nonlocal.cuda_benchmark.v0\","
                   "\"status\":\"FAIL\",\"reason\":\"self_test_preflight_failed\","
                   "\"accumulation_identity\":\""
                << accumulation_identity(mode) << "\",\"handoff_identity\":\""
                << handoff_identity(handoff) << "\",\"term_kernel_identity\":\""
                << term_kernel_identity(term_kernels) << "\",\"storage_identity\":\""
                << storage_identity(storage) << "\",\"binary_sha256\":\""
                << executable_hash() << "\"}";
        return {false, failure.str()};
    }

    const Fixture fixture = performance_fixture(profile, profile.fixed_iterations);
    CudaBaseline baseline(fixture, mode, handoff, term_kernels, storage);
    const CapturedRun before = baseline.execute(true);
    for (int run = 0; run < warmup; ++run) {
        const CapturedRun warm = baseline.execute(false);
        if (warm.local_solve_failed || !warm.reverse_map_valid || !warm.storage_map_valid) {
            throw std::runtime_error("solver, O3 layout or O4 storage failed during benchmark warmup");
        }
    }

    std::vector<StageTiming> timings;
    timings.reserve(static_cast<std::size_t>(runs));
    for (int run = 0; run < runs; ++run) {
        CapturedRun measured = baseline.execute(false);
        if (measured.local_solve_failed || !measured.reverse_map_valid
            || !measured.storage_map_valid) {
            throw std::runtime_error(
                "solver, O3 layout or O4 storage failed during measured benchmark run");
        }
        timings.push_back(measured.timing);
    }
    const CapturedRun after = baseline.execute(true);
    const Comparison repeated =
        compare_results(fixture, before.state, after.state, profile.tolerances);
    const bool repeated_neighbors_identical =
        before.offsets == after.offsets && before.neighbors == after.neighbors;
    const bool neighbors_passed = repeated_neighbors_identical
        && valid_symmetric_neighbors(before, fixture.particles.size())
        && before.state.directed_pairs <= profile.max_directed_pairs
        && before.state.maximum_degree <= profile.max_neighbors;
    const bool repeated_output_exact = ordered_output_digest(before.state)
        == ordered_output_digest(after.state);
    const bool passed = finite_state(before.state) && finite_state(after.state)
        && !before.local_solve_failed && !after.local_solve_failed && neighbors_passed
        && before.reverse_map_valid && after.reverse_map_valid
        && before.storage_map_valid && after.storage_map_valid
        && repeated.passed
        && before.state.normalized_momentum_residual
            <= profile.tolerances.normalized_momentum_residual
        && after.state.normalized_momentum_residual
            <= profile.tolerances.normalized_momentum_residual
        && (handoff != HandoffMode::PointerSwapO1 || repeated_output_exact);

    const auto collect = [&](auto member) {
        std::vector<double> values;
        values.reserve(timings.size());
        for (const StageTiming& timing : timings) {
            values.push_back(timing.*member);
        }
        return statistics(std::move(values));
    };

    std::ostringstream output;
    output << std::setprecision(17);
    output << "{\"schema\":\"nextengine.nonlocal.cuda_benchmark.v0\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"profile_id\":\"" << profile.id
           << "\",\"accumulation_identity\":\"" << accumulation_identity(mode)
           << "\",\"handoff_identity\":\"" << handoff_identity(handoff)
           << "\",\"term_kernel_identity\":\"" << term_kernel_identity(term_kernels)
           << "\",\"storage_identity\":\"" << storage_identity(storage)
           << "\",\"binary_sha256\":\"" << executable_hash()
           << "\",\"binary_bytes\":" << executable_bytes()
           << ",\"command\":\"nonlocal-feasibility --benchmark " << profile.id
           << " --warmup " << warmup << " --runs " << runs << " --accumulation "
           << accumulation_identity(mode) << " --handoff " << handoff_identity(handoff)
           << " --term-kernels " << term_kernel_identity(term_kernels)
           << " --storage " << storage_identity(storage)
           << "\",\"profile_sha256\":\"" << sha256_hex(canonical_profile_json(profile))
           << "\",\"input_sha256\":\"" << fixture_input_hash(fixture)
           << "\",\"fixed_iterations\":" << profile.fixed_iterations
           << ",\"warmup_runs\":" << warmup << ",\"measured_runs\":" << runs
           << ",\"samples\":" << fixture.particles.size() << ",\"directed_pairs\":"
           << before.state.directed_pairs << ",\"maximum_degree\":"
           << before.state.maximum_degree << ",\"device_memory_bytes\":"
           << before.device_memory_bytes << ",\"neighbors_passed\":"
           << (neighbors_passed ? "true" : "false")
           << ",\"layout_memory_bytes\":" << before.layout_memory_bytes
           << ",\"layout_setup_ms\":" << before.layout_setup_ms
           << ",\"storage_map_valid\":"
           << (before.storage_map_valid && after.storage_map_valid ? "true" : "false")
           << ",\"storage_memory_bytes\":" << before.storage_memory_bytes
           << ",\"storage_setup_ms\":" << before.storage_setup_ms
           << ",\"storage_to_sample_sha256\":\""
           << before.storage_to_sample_sha256
           << "\",\"sample_to_storage_sha256\":\""
           << before.sample_to_storage_sha256
           << "\",\"physical_csr_sha256\":\"" << before.physical_csr_sha256
           << "\",\"mean_neighbor_storage_distance\":"
           << before.mean_neighbor_storage_distance
           << ",\"p95_neighbor_storage_distance\":"
           << before.p95_neighbor_storage_distance
           << ",\"reverse_map_valid\":"
           << (before.reverse_map_valid && after.reverse_map_valid ? "true" : "false")
           << ",\"self_slots\":" << before.self_slots
           << ",\"nonself_directed_samples\":" << before.nonself_directed_samples
           << ",\"unique_pairs\":" << before.unique_pairs
           << ",\"pair_evaluations_per_active_term\":"
           << before.pair_evaluations_per_active_term
           << ",\"endpoint_fragments_per_active_term\":"
           << before.endpoint_fragments_per_active_term
           << ",\"repeated_neighbors_identical\":"
           << (repeated_neighbors_identical ? "true" : "false")
           << ",\"repeated_output_passed\":" << (repeated.passed ? "true" : "false")
           << ",\"repeated_output_exact\":"
           << (repeated_output_exact ? "true" : "false")
           << ",\"normalized_momentum_residual\":["
           << before.state.normalized_momentum_residual << ','
           << after.state.normalized_momentum_residual << "],\"device\":" << device_json()
           << ",\"statistics\":{\"prediction\":";
    append_statistics(output, collect(&StageTiming::prediction));
    output << ",\"neighbor_construction\":";
    append_statistics(output, collect(&StageTiming::neighbor_construction));
    output << ",\"density\":";
    append_statistics(output, collect(&StageTiming::density));
    output << ",\"buffer_reset\":";
    append_statistics(output, collect(&StageTiming::buffer_reset));
    output << ",\"incompressibility\":";
    append_statistics(output, collect(&StageTiming::incompressibility));
    output << ",\"viscosity\":";
    append_statistics(output, collect(&StageTiming::viscosity));
    output << ",\"surface_tension\":";
    append_statistics(output, collect(&StageTiming::surface_tension));
    output << ",\"fused_owner_terms\":";
    append_statistics(output, collect(&StageTiming::fused_owner_terms));
    output << ",\"local_update\":";
    append_statistics(output, collect(&StageTiming::local_update));
    output << ",\"state_handoff_and_velocity\":";
    append_statistics(output, collect(&StageTiming::state_handoff));
    output << ",\"total\":";
    append_statistics(output, collect(&StageTiming::total));
    output << "},\"raw_total_ms\":[";
    for (std::size_t index = 0; index < timings.size(); ++index) {
        if (index != 0) {
            output << ',';
        }
        output << timings[index].total;
    }
    output << "]}";
    return {passed, output.str()};
}

CommandReport run_cuda_layout_tournament(
    const Profile& profile,
    int warmup,
    int runs) {
    if (profile.id == "nuv-tiny-oracle.v0") {
        throw std::invalid_argument("layout tournament requires a fixed performance profile");
    }
    if (warmup != 32 || runs != 96) {
        throw std::invalid_argument("O3 layout tournament requires --warmup 32 --runs 96");
    }

    const CommandReport atomic_self = run_cuda_self_test(
        AccumulationMode::SourceAtomicV0, HandoffMode::CopyV0, TermKernelMode::RuntimeV0);
    const CommandReport gather_self = run_cuda_self_test(
        AccumulationMode::GatherDirectedR0,
        HandoffMode::PointerSwapO1, TermKernelMode::SpecializedO2);
    const CommandReport segmented_self = run_cuda_self_test(
        AccumulationMode::UniquePairSegmentedO3,
        HandoffMode::PointerSwapO1, TermKernelMode::SpecializedO2);
    if (!atomic_self.passed || !gather_self.passed || !segmented_self.passed) {
        std::ostringstream failure;
        failure << "{\"schema\":\"nextengine.nonlocal.cuda_layout_tournament.v0\","
                   "\"status\":\"FAIL\",\"reason\":\"self_test_preflight_failed\","
                   "\"preflight\":{\"atomic\":"
                << (atomic_self.passed ? "true" : "false") << ",\"gather\":"
                << (gather_self.passed ? "true" : "false") << ",\"segmented\":"
                << (segmented_self.passed ? "true" : "false") << "}}";
        return {false, failure.str()};
    }

    const Fixture fixture = performance_fixture(profile, profile.fixed_iterations);
    CudaBaseline atomic(
        fixture, AccumulationMode::SourceAtomicV0,
        HandoffMode::CopyV0, TermKernelMode::RuntimeV0);
    CudaBaseline gather(
        fixture, AccumulationMode::GatherDirectedR0,
        HandoffMode::PointerSwapO1, TermKernelMode::SpecializedO2);
    CudaBaseline segmented(
        fixture, AccumulationMode::UniquePairSegmentedO3,
        HandoffMode::PointerSwapO1, TermKernelMode::SpecializedO2);

    const CapturedRun atomic_before = atomic.execute(true);
    const CapturedRun gather_before = gather.execute(true);
    const CapturedRun segmented_before = segmented.execute(true);
    const std::size_t co_resident_memory = atomic_before.device_memory_bytes
        + gather_before.device_memory_bytes + segmented_before.device_memory_bytes;
    constexpr std::size_t CO_RESIDENT_CEILING = 420000000U;
    if (co_resident_memory > CO_RESIDENT_CEILING) {
        std::ostringstream failure;
        failure << "{\"schema\":\"nextengine.nonlocal.cuda_layout_tournament.v0\","
                   "\"status\":\"CAPACITY_REJECT\",\"reason\":\"co_resident_memory_ceiling\","
                   "\"three_instance_co_resident_bytes\":"
                << co_resident_memory << ",\"ceiling_bytes\":" << CO_RESIDENT_CEILING << '}';
        return {false, failure.str()};
    }

    std::array<CudaBaseline*, 3> layouts = {&atomic, &gather, &segmented};
    const auto execute_rotated = [&](int round, bool capture, std::vector<StageTiming>* timings) {
        for (int position = 0; position < 3; ++position) {
            const int layout = (round + position) % 3;
            CapturedRun run = layouts[static_cast<std::size_t>(layout)]->execute(capture);
            if (run.local_solve_failed || !run.reverse_map_valid) {
                throw std::runtime_error("solver or O3 layout failed during layout tournament");
            }
            if (timings != nullptr) {
                timings[layout].push_back(run.timing);
            }
        }
    };
    for (int round = 0; round < warmup; ++round) {
        execute_rotated(round, false, nullptr);
    }
    std::vector<StageTiming> timings[3];
    for (auto& layout_timings : timings) {
        layout_timings.reserve(static_cast<std::size_t>(runs));
    }
    for (int round = 0; round < runs; ++round) {
        execute_rotated(round, false, timings);
    }

    const CapturedRun atomic_after = atomic.execute(true);
    const CapturedRun gather_after = gather.execute(true);
    const CapturedRun segmented_after = segmented.execute(true);
    const std::array<const CapturedRun*, 3> before = {
        &atomic_before, &gather_before, &segmented_before};
    const std::array<const CapturedRun*, 3> after = {
        &atomic_after, &gather_after, &segmented_after};

    bool topology_passed = true;
    bool finite_passed = true;
    bool solve_passed = true;
    bool momentum_passed = true;
    bool repeated_correspondence[3] = {};
    bool repeated_exact[3] = {};
    for (int layout = 0; layout < 3; ++layout) {
        const CapturedRun& first = *before[static_cast<std::size_t>(layout)];
        const CapturedRun& second = *after[static_cast<std::size_t>(layout)];
        const bool same_topology = first.offsets == second.offsets
            && first.neighbors == second.neighbors
            && valid_symmetric_neighbors(first, fixture.particles.size())
            && first.state.directed_pairs <= profile.max_directed_pairs
            && first.state.maximum_degree <= profile.max_neighbors;
        topology_passed = topology_passed && same_topology;
        finite_passed = finite_passed && finite_state(first.state) && finite_state(second.state);
        solve_passed = solve_passed && !first.local_solve_failed && !second.local_solve_failed
            && first.reverse_map_valid && second.reverse_map_valid;
        momentum_passed = momentum_passed
            && first.state.normalized_momentum_residual
                <= profile.tolerances.normalized_momentum_residual
            && second.state.normalized_momentum_residual
                <= profile.tolerances.normalized_momentum_residual;
        repeated_correspondence[layout] =
            compare_results(fixture, first.state, second.state, profile.tolerances).passed;
        repeated_exact[layout] = ordered_output_digest(first.state)
            == ordered_output_digest(second.state);
    }

    const Comparison layout_comparison = compare_results(
        fixture, gather_before.state, segmented_before.state, profile.tolerances);
    const bool layout_correspondence = layout_comparison.passed
        && gather_before.offsets == segmented_before.offsets
        && gather_before.neighbors == segmented_before.neighbors;
    const bool layout_exact = ordered_output_digest(gather_before.state)
        == ordered_output_digest(segmented_before.state);
    const std::size_t expected_pair_capacity = fixture.particles.size() * 123U;
    const std::size_t expected_layout_memory = expected_pair_capacity * 52U + sizeof(int);
    const std::size_t candidate_ceiling = fixture.particles.size() == 16000U
        ? 113250058U
        : (fixture.particles.size() == 48000U ? 339733770U : 0U);
    const bool capacity_passed = segmented_before.layout_memory_bytes == expected_layout_memory
        && candidate_ceiling != 0U
        && segmented_before.device_memory_bytes <= candidate_ceiling
        && co_resident_memory <= CO_RESIDENT_CEILING;
    const bool work_passed = segmented_before.self_slots == fixture.particles.size()
        && segmented_before.nonself_directed_samples == 2U * segmented_before.unique_pairs
        && segmented_before.pair_evaluations_per_active_term == segmented_before.unique_pairs
        && segmented_before.endpoint_fragments_per_active_term
            == segmented_before.nonself_directed_samples
        && gather_before.pair_evaluations_per_active_term
            == gather_before.nonself_directed_samples;
    const bool passed = topology_passed && finite_passed && solve_passed && momentum_passed
        && repeated_correspondence[0] && repeated_correspondence[1]
        && repeated_correspondence[2] && repeated_exact[1] && repeated_exact[2]
        && layout_correspondence && capacity_passed && work_passed;

    const Statistics gather_total =
        collect_statistics(timings[1], &StageTiming::total);
    const Statistics segmented_total =
        collect_statistics(timings[2], &StageTiming::total);
    const double p95_speedup = gather_total.p95 / segmented_total.p95;
    const std::array<const char*, 3> names = {
        "historical_atomic", "retained_gather", "candidate_segmented"};
    const std::array<AccumulationMode, 3> accumulations = {
        AccumulationMode::SourceAtomicV0,
        AccumulationMode::GatherDirectedR0,
        AccumulationMode::UniquePairSegmentedO3};
    const std::array<HandoffMode, 3> handoffs = {
        HandoffMode::CopyV0, HandoffMode::PointerSwapO1, HandoffMode::PointerSwapO1};
    const std::array<TermKernelMode, 3> terms = {
        TermKernelMode::RuntimeV0, TermKernelMode::SpecializedO2,
        TermKernelMode::SpecializedO2};

    std::ostringstream output;
    output << std::setprecision(17);
    output << "{\"schema\":\"nextengine.nonlocal.cuda_layout_tournament.v0\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"profile_id\":\"" << profile.id
           << "\",\"profile_sha256\":\"" << sha256_hex(canonical_profile_json(profile))
           << "\",\"input_sha256\":\"" << fixture_input_hash(fixture)
           << "\",\"binary_sha256\":\"" << executable_hash()
           << "\",\"binary_bytes\":" << executable_bytes()
           << ",\"command\":\"nonlocal-feasibility --layout-tournament " << profile.id
           << " --warmup " << warmup << " --runs " << runs
           << "\",\"fixed_iterations\":" << profile.fixed_iterations
           << ",\"warmup_rounds\":" << warmup << ",\"measured_rounds\":" << runs
           << ",\"samples_per_layout\":" << runs
           << ",\"rotation\":[[\"A\",\"G\",\"S\"],[\"G\",\"S\",\"A\"],"
              "[\"S\",\"A\",\"G\"]],\"each_measured_position_count\":32"
           << ",\"preflight\":{\"atomic\":true,\"gather\":true,\"segmented\":true}"
           << ",\"correctness\":{\"topology_passed\":"
           << (topology_passed ? "true" : "false") << ",\"finite_passed\":"
           << (finite_passed ? "true" : "false") << ",\"solve_passed\":"
           << (solve_passed ? "true" : "false") << ",\"momentum_passed\":"
           << (momentum_passed ? "true" : "false")
           << ",\"layout_baseline_correspondence\":"
           << (layout_correspondence ? "true" : "false")
           << ",\"layout_baseline_exact\":" << (layout_exact ? "true" : "false")
           << ",\"work_accounting_passed\":" << (work_passed ? "true" : "false")
           << ",\"capacity_passed\":" << (capacity_passed ? "true" : "false") << '}'
           << ",\"memory\":{\"three_instance_co_resident_bytes\":" << co_resident_memory
           << ",\"three_instance_ceiling_bytes\":" << CO_RESIDENT_CEILING
           << ",\"candidate_ceiling_bytes\":" << candidate_ceiling
           << ",\"candidate_expected_layout_bytes\":" << expected_layout_memory << '}'
           << ",\"work\":{\"directed_samples\":" << segmented_before.state.directed_pairs
           << ",\"self_slots\":" << segmented_before.self_slots
           << ",\"nonself_directed_samples\":" << segmented_before.nonself_directed_samples
           << ",\"unique_pairs\":" << segmented_before.unique_pairs
           << ",\"gather_pair_evaluations_per_active_term\":"
           << gather_before.pair_evaluations_per_active_term
           << ",\"candidate_pair_evaluations_per_active_term\":"
           << segmented_before.pair_evaluations_per_active_term
           << ",\"candidate_endpoint_fragments_per_active_term\":"
           << segmented_before.endpoint_fragments_per_active_term << '}'
           << ",\"candidate_vs_retained\":{\"total_p95_speedup\":" << p95_speedup
           << ",\"retained_total_p95_ms\":" << gather_total.p95
           << ",\"candidate_total_p95_ms\":" << segmented_total.p95 << '}'
           << ",\"layouts\":{";
    for (int layout = 0; layout < 3; ++layout) {
        if (layout != 0) {
            output << ',';
        }
        const CapturedRun& first = *before[static_cast<std::size_t>(layout)];
        output << '\"' << names[static_cast<std::size_t>(layout)] << "\":{"
               << "\"accumulation_identity\":\""
               << accumulation_identity(accumulations[static_cast<std::size_t>(layout)])
               << "\",\"handoff_identity\":\""
               << handoff_identity(handoffs[static_cast<std::size_t>(layout)])
               << "\",\"term_kernel_identity\":\""
               << term_kernel_identity(terms[static_cast<std::size_t>(layout)])
               << "\",\"device_memory_bytes\":" << first.device_memory_bytes
               << ",\"layout_memory_bytes\":" << first.layout_memory_bytes
               << ",\"layout_setup_ms\":" << first.layout_setup_ms
               << ",\"reverse_map_valid\":"
               << (first.reverse_map_valid ? "true" : "false")
               << ",\"repeated_output_correspondence\":"
               << (repeated_correspondence[layout] ? "true" : "false")
               << ",\"repeated_output_exact\":"
               << (repeated_exact[layout] ? "true" : "false")
               << ",\"ordered_output_sha256\":\"" << ordered_output_digest(first.state)
               << "\",\"csr_sha256\":\"" << csr_digest(first) << "\",\"statistics\":";
        append_stage_statistics(output, timings[layout]);
        output << ",\"raw_total_ms\":";
        append_raw_totals(output, timings[layout]);
        output << '}';
    }
    output << "},\"device\":" << device_json() << '}';
    return {passed, output.str()};
}

CommandReport run_cuda_locality_tournament(
    const Profile& profile,
    int warmup,
    int runs) {
    if (profile.id == "nuv-tiny-oracle.v0") {
        throw std::invalid_argument("locality tournament requires a fixed performance profile");
    }
    if (warmup != 32 || runs != 96) {
        throw std::invalid_argument("O4 locality tournament requires --warmup 32 --runs 96");
    }

    const CommandReport stable_self = run_cuda_self_test(
        AccumulationMode::GatherDirectedR0,
        HandoffMode::PointerSwapO1, TermKernelMode::SpecializedO2,
        StorageMode::StableSampleV0);
    const CommandReport sorted_self = run_cuda_self_test(
        AccumulationMode::GatherDirectedR0,
        HandoffMode::PointerSwapO1, TermKernelMode::SpecializedO2,
        StorageMode::CellSortedO4);
    if (!stable_self.passed || !sorted_self.passed) {
        std::ostringstream failure;
        failure << "{\"schema\":\"nextengine.nonlocal.cuda_locality_tournament.v0\","
                   "\"status\":\"FAIL\",\"reason\":\"self_test_preflight_failed\","
                   "\"preflight\":{\"stable_sample\":"
                << (stable_self.passed ? "true" : "false") << ",\"cell_sorted\":"
                << (sorted_self.passed ? "true" : "false") << "}}";
        return {false, failure.str()};
    }

    const Fixture fixture = performance_fixture(profile, profile.fixed_iterations);
    CudaBaseline stable(
        fixture, AccumulationMode::GatherDirectedR0,
        HandoffMode::PointerSwapO1, TermKernelMode::SpecializedO2,
        StorageMode::StableSampleV0);
    CudaBaseline sorted(
        fixture, AccumulationMode::GatherDirectedR0,
        HandoffMode::PointerSwapO1, TermKernelMode::SpecializedO2,
        StorageMode::CellSortedO4);

    const CapturedRun stable_before = stable.execute(true);
    const CapturedRun sorted_before = sorted.execute(true);
    const std::size_t co_resident_memory =
        stable_before.device_memory_bytes + sorted_before.device_memory_bytes;
    constexpr std::size_t CO_RESIDENT_CEILING = 70000000U;
    const std::size_t expected_storage_memory = fixture.particles.size() * 33U + sizeof(int);
    const std::size_t candidate_ceiling = fixture.particles.size() == 16000U
        ? 11442058U
        : (fixture.particles.size() == 48000U ? 34309770U : 0U);
    const bool capacity_passed = candidate_ceiling != 0U
        && sorted_before.storage_memory_bytes == expected_storage_memory
        && sorted_before.device_memory_bytes <= candidate_ceiling
        && co_resident_memory <= CO_RESIDENT_CEILING;
    if (!capacity_passed) {
        std::ostringstream failure;
        failure << "{\"schema\":\"nextengine.nonlocal.cuda_locality_tournament.v0\","
                   "\"status\":\"CAPACITY_REJECT\",\"profile_id\":\""
                << profile.id << "\",\"candidate_bytes\":"
                << sorted_before.device_memory_bytes << ",\"candidate_ceiling_bytes\":"
                << candidate_ceiling << ",\"storage_bytes\":"
                << sorted_before.storage_memory_bytes << ",\"expected_storage_bytes\":"
                << expected_storage_memory << ",\"two_instance_co_resident_bytes\":"
                << co_resident_memory << ",\"co_resident_ceiling_bytes\":"
                << CO_RESIDENT_CEILING << '}';
        return {false, failure.str()};
    }

    std::array<CudaBaseline*, 2> layouts = {&stable, &sorted};
    const auto execute_alternating =
        [&](int round, bool capture, std::vector<StageTiming>* timings) {
            for (int position = 0; position < 2; ++position) {
                const int layout = (round + position) % 2;
                CapturedRun run = layouts[static_cast<std::size_t>(layout)]->execute(capture);
                if (run.local_solve_failed || !run.reverse_map_valid
                    || !run.storage_map_valid) {
                    throw std::runtime_error(
                        "solver or storage identity failed during O4 locality tournament");
                }
                if (timings != nullptr) {
                    timings[layout].push_back(run.timing);
                }
            }
        };
    for (int round = 0; round < warmup; ++round) {
        execute_alternating(round, false, nullptr);
    }
    std::vector<StageTiming> timings[2];
    for (auto& values : timings) {
        values.reserve(static_cast<std::size_t>(runs));
    }
    for (int round = 0; round < runs; ++round) {
        execute_alternating(round, false, timings);
    }

    const CapturedRun stable_after = stable.execute(true);
    const CapturedRun sorted_after = sorted.execute(true);
    const bool logical_correspondence_exact =
        ordered_output_digest(stable_before.state) == ordered_output_digest(sorted_before.state)
        && stable_before.offsets == sorted_before.offsets
        && stable_before.neighbors == sorted_before.neighbors;
    const bool stable_repeated_exact =
        ordered_output_digest(stable_before.state) == ordered_output_digest(stable_after.state)
        && stable_before.offsets == stable_after.offsets
        && stable_before.neighbors == stable_after.neighbors;
    const bool sorted_repeated_exact =
        ordered_output_digest(sorted_before.state) == ordered_output_digest(sorted_after.state)
        && sorted_before.offsets == sorted_after.offsets
        && sorted_before.neighbors == sorted_after.neighbors;
    const bool topology_passed = valid_symmetric_neighbors(
            stable_before, fixture.particles.size())
        && valid_symmetric_neighbors(sorted_before, fixture.particles.size())
        && stable_before.state.directed_pairs <= profile.max_directed_pairs
        && sorted_before.state.directed_pairs <= profile.max_directed_pairs
        && stable_before.state.maximum_degree <= profile.max_neighbors
        && sorted_before.state.maximum_degree <= profile.max_neighbors;
    const bool finite_passed = finite_state(stable_before.state)
        && finite_state(stable_after.state) && finite_state(sorted_before.state)
        && finite_state(sorted_after.state);
    const bool solve_passed = !stable_before.local_solve_failed
        && !stable_after.local_solve_failed && !sorted_before.local_solve_failed
        && !sorted_after.local_solve_failed && sorted_before.storage_map_valid
        && sorted_after.storage_map_valid;
    const bool momentum_passed = stable_before.state.normalized_momentum_residual
            <= profile.tolerances.normalized_momentum_residual
        && stable_after.state.normalized_momentum_residual
            <= profile.tolerances.normalized_momentum_residual
        && sorted_before.state.normalized_momentum_residual
            <= profile.tolerances.normalized_momentum_residual
        && sorted_after.state.normalized_momentum_residual
            <= profile.tolerances.normalized_momentum_residual;
    const bool passed = logical_correspondence_exact && stable_repeated_exact
        && sorted_repeated_exact && topology_passed && finite_passed && solve_passed
        && momentum_passed && capacity_passed;

    const Statistics stable_total = collect_statistics(timings[0], &StageTiming::total);
    const Statistics sorted_total = collect_statistics(timings[1], &StageTiming::total);
    const double stable_pair_p95 =
        collect_statistics(timings[0], &StageTiming::density).p95
        + collect_statistics(timings[0], &StageTiming::incompressibility).p95
        + collect_statistics(timings[0], &StageTiming::viscosity).p95
        + collect_statistics(timings[0], &StageTiming::surface_tension).p95;
    const double sorted_pair_p95 =
        collect_statistics(timings[1], &StageTiming::density).p95
        + collect_statistics(timings[1], &StageTiming::incompressibility).p95
        + collect_statistics(timings[1], &StageTiming::viscosity).p95
        + collect_statistics(timings[1], &StageTiming::surface_tension).p95;
    const double total_p95_speedup = stable_total.p95 / sorted_total.p95;
    const bool total_profile_rule = profile.id == "nuv-water-16k.v0"
        ? sorted_total.p95 <= stable_total.p95 * 1.02
        : sorted_total.p95 < stable_total.p95;
    const bool pair_profile_rule = sorted_pair_p95 <= stable_pair_p95;

    std::ostringstream output;
    output << std::setprecision(17);
    output << "{\"schema\":\"nextengine.nonlocal.cuda_locality_tournament.v0\","
           << "\"status\":\"" << (passed ? "PASS" : "FAIL")
           << "\",\"profile_id\":\"" << profile.id
           << "\",\"profile_sha256\":\"" << sha256_hex(canonical_profile_json(profile))
           << "\",\"input_sha256\":\"" << fixture_input_hash(fixture)
           << "\",\"binary_sha256\":\"" << executable_hash()
           << "\",\"binary_bytes\":" << executable_bytes()
           << ",\"command\":\"nonlocal-feasibility --locality-tournament " << profile.id
           << " --warmup " << warmup << " --runs " << runs
           << "\",\"fixed_iterations\":" << profile.fixed_iterations
           << ",\"warmup_rounds\":" << warmup << ",\"measured_rounds\":" << runs
           << ",\"samples_per_layout\":" << runs
           << ",\"rotation\":[[\"stable\",\"sorted\"],[\"sorted\",\"stable\"]]"
           << ",\"each_measured_position_count\":48"
           << ",\"correctness\":{\"logical_correspondence_exact\":"
           << (logical_correspondence_exact ? "true" : "false")
           << ",\"stable_repeated_exact\":"
           << (stable_repeated_exact ? "true" : "false")
           << ",\"sorted_repeated_exact\":"
           << (sorted_repeated_exact ? "true" : "false")
           << ",\"topology_passed\":" << (topology_passed ? "true" : "false")
           << ",\"finite_passed\":" << (finite_passed ? "true" : "false")
           << ",\"solve_and_map_passed\":" << (solve_passed ? "true" : "false")
           << ",\"momentum_passed\":" << (momentum_passed ? "true" : "false")
           << ",\"capacity_passed\":" << (capacity_passed ? "true" : "false") << '}'
           << ",\"memory\":{\"stable_bytes\":" << stable_before.device_memory_bytes
           << ",\"candidate_bytes\":" << sorted_before.device_memory_bytes
           << ",\"candidate_storage_bytes\":" << sorted_before.storage_memory_bytes
           << ",\"expected_storage_bytes\":" << expected_storage_memory
           << ",\"candidate_ceiling_bytes\":" << candidate_ceiling
           << ",\"two_instance_co_resident_bytes\":" << co_resident_memory
           << ",\"co_resident_ceiling_bytes\":" << CO_RESIDENT_CEILING << '}'
           << ",\"locality\":{\"stable_mean_neighbor_distance\":"
           << stable_before.mean_neighbor_storage_distance
           << ",\"stable_p95_neighbor_distance\":"
           << stable_before.p95_neighbor_storage_distance
           << ",\"sorted_mean_neighbor_distance\":"
           << sorted_before.mean_neighbor_storage_distance
           << ",\"sorted_p95_neighbor_distance\":"
           << sorted_before.p95_neighbor_storage_distance << '}'
           << ",\"candidate_vs_retained\":{\"total_p95_speedup\":"
           << total_p95_speedup << ",\"retained_total_p95_ms\":" << stable_total.p95
           << ",\"candidate_total_p95_ms\":" << sorted_total.p95
           << ",\"retained_pair_stage_p95_sum_ms\":" << stable_pair_p95
           << ",\"candidate_pair_stage_p95_sum_ms\":" << sorted_pair_p95
           << ",\"profile_total_rule_passed\":"
           << (total_profile_rule ? "true" : "false")
           << ",\"profile_pair_rule_passed\":"
           << (pair_profile_rule ? "true" : "false") << '}'
           << ",\"layouts\":{\"retained_stable\":{\"storage_identity\":\""
           << storage_identity(StorageMode::StableSampleV0)
           << "\",\"ordered_output_sha256\":\""
           << ordered_output_digest(stable_before.state) << "\",\"logical_csr_sha256\":\""
           << csr_digest(stable_before) << "\",\"physical_csr_sha256\":\""
           << stable_before.physical_csr_sha256 << "\",\"statistics\":";
    append_stage_statistics(output, timings[0]);
    output << ",\"raw_total_ms\":";
    append_raw_totals(output, timings[0]);
    output << "},\"candidate_sorted\":{\"storage_identity\":\""
           << storage_identity(StorageMode::CellSortedO4)
           << "\",\"ordered_output_sha256\":\""
           << ordered_output_digest(sorted_before.state) << "\",\"logical_csr_sha256\":\""
           << csr_digest(sorted_before) << "\",\"physical_csr_sha256\":\""
           << sorted_before.physical_csr_sha256
           << "\",\"storage_to_sample_sha256\":\""
           << sorted_before.storage_to_sample_sha256
           << "\",\"sample_to_storage_sha256\":\""
           << sorted_before.sample_to_storage_sha256
           << "\",\"storage_setup_ms\":" << sorted_before.storage_setup_ms
           << ",\"statistics\":";
    append_stage_statistics(output, timings[1]);
    output << ",\"raw_total_ms\":";
    append_raw_totals(output, timings[1]);
    output << "}},\"device\":" << device_json() << '}';
    return {passed, output.str()};
}

CommandReport run_cuda_retained_tournament(
    const Profile& profile,
    int warmup,
    int runs) {
    if (profile.id == "nuv-tiny-oracle.v0" || profile.id == "nuv-surface-16k.v0") {
        throw std::invalid_argument(
            "retained tournament requires a correctness-valid HN-3 denominator profile");
    }
    if (warmup != 32 || runs != 96) {
        throw std::invalid_argument("NR2 retained tournament requires --warmup 32 --runs 96");
    }

    const CommandReport atomic_self = run_cuda_self_test(
        AccumulationMode::SourceAtomicV0, HandoffMode::CopyV0,
        TermKernelMode::RuntimeV0, StorageMode::StableSampleV0);
    const CommandReport retained_self = run_cuda_self_test(
        AccumulationMode::GatherDirectedR0, HandoffMode::PointerSwapO1,
        TermKernelMode::SpecializedO2, StorageMode::StableSampleV0);
    if (!atomic_self.passed || !retained_self.passed) {
        std::ostringstream failure;
        failure << "{\"schema\":\"nextengine.nonlocal.cuda_retained_tournament.v0\","
                   "\"status\":\"FAIL\",\"reason\":\"self_test_preflight_failed\","
                   "\"preflight\":{\"source_atomic\":"
                << (atomic_self.passed ? "true" : "false") << ",\"retained\":"
                << (retained_self.passed ? "true" : "false") << "}}";
        return {false, failure.str()};
    }

    const Fixture fixture = performance_fixture(profile, profile.fixed_iterations);
    CudaBaseline atomic(
        fixture, AccumulationMode::SourceAtomicV0, HandoffMode::CopyV0,
        TermKernelMode::RuntimeV0, StorageMode::StableSampleV0);
    CudaBaseline retained(
        fixture, AccumulationMode::GatherDirectedR0, HandoffMode::PointerSwapO1,
        TermKernelMode::SpecializedO2, StorageMode::StableSampleV0);
    const CapturedRun atomic_before = atomic.execute(true);
    const CapturedRun retained_before = retained.execute(true);
    const std::size_t co_resident_memory =
        atomic_before.device_memory_bytes + retained_before.device_memory_bytes;
    constexpr std::size_t CO_RESIDENT_CEILING = 70000000U;
    if (co_resident_memory > CO_RESIDENT_CEILING) {
        std::ostringstream failure;
        failure << "{\"schema\":\"nextengine.nonlocal.cuda_retained_tournament.v0\","
                   "\"status\":\"CAPACITY_REJECT\",\"profile_id\":\""
                << profile.id << "\",\"two_instance_co_resident_bytes\":"
                << co_resident_memory << ",\"ceiling_bytes\":"
                << CO_RESIDENT_CEILING << '}';
        return {false, failure.str()};
    }

    std::array<CudaBaseline*, 2> implementations = {&atomic, &retained};
    const auto execute_alternating =
        [&](int round, bool capture, std::vector<StageTiming>* timings) {
            for (int position = 0; position < 2; ++position) {
                const int implementation = (round + position) % 2;
                CapturedRun run =
                    implementations[static_cast<std::size_t>(implementation)]->execute(capture);
                if (run.local_solve_failed || !run.reverse_map_valid
                    || !run.storage_map_valid) {
                    throw std::runtime_error(
                        "solver identity failed during retained NR2 tournament");
                }
                if (timings != nullptr) {
                    timings[implementation].push_back(run.timing);
                }
            }
        };
    for (int round = 0; round < warmup; ++round) {
        execute_alternating(round, false, nullptr);
    }
    std::vector<StageTiming> timings[2];
    for (auto& values : timings) {
        values.reserve(static_cast<std::size_t>(runs));
    }
    for (int round = 0; round < runs; ++round) {
        execute_alternating(round, false, timings);
    }

    const CapturedRun atomic_after = atomic.execute(true);
    const CapturedRun retained_after = retained.execute(true);
    const bool atomic_repeated_correspondence = compare_results(
        fixture, atomic_before.state, atomic_after.state, profile.tolerances).passed;
    const bool retained_repeated_exact =
        ordered_output_digest(retained_before.state)
            == ordered_output_digest(retained_after.state)
        && retained_before.offsets == retained_after.offsets
        && retained_before.neighbors == retained_after.neighbors;
    const bool adjacent_correspondence = compare_results(
        fixture, atomic_before.state, retained_before.state, profile.tolerances).passed;
    const bool topology_passed = valid_symmetric_neighbors(
            atomic_before, fixture.particles.size())
        && valid_symmetric_neighbors(retained_before, fixture.particles.size())
        && atomic_before.offsets == retained_before.offsets
        && atomic_before.neighbors == retained_before.neighbors
        && atomic_before.state.directed_pairs <= profile.max_directed_pairs
        && retained_before.state.maximum_degree <= profile.max_neighbors;
    const bool finite_passed = finite_state(atomic_before.state)
        && finite_state(atomic_after.state) && finite_state(retained_before.state)
        && finite_state(retained_after.state);
    const bool solve_passed = !atomic_before.local_solve_failed
        && !atomic_after.local_solve_failed && !retained_before.local_solve_failed
        && !retained_after.local_solve_failed;
    const bool momentum_passed = atomic_before.state.normalized_momentum_residual
            <= profile.tolerances.normalized_momentum_residual
        && atomic_after.state.normalized_momentum_residual
            <= profile.tolerances.normalized_momentum_residual
        && retained_before.state.normalized_momentum_residual
            <= profile.tolerances.normalized_momentum_residual
        && retained_after.state.normalized_momentum_residual
            <= profile.tolerances.normalized_momentum_residual;
    // HN-3 admits the atomic denominator by its own frozen full-control gate.
    // Atomic/gather cross-trajectory correspondence is diagnostic: endpoint
    // association differs by construction and was never an HN-3 timing gate.
    const bool passed = atomic_repeated_correspondence && retained_repeated_exact
        && topology_passed && finite_passed && solve_passed && momentum_passed;

    const Statistics atomic_total = collect_statistics(timings[0], &StageTiming::total);
    const Statistics retained_total = collect_statistics(timings[1], &StageTiming::total);
    const double total_p95_speedup = atomic_total.p95 / retained_total.p95;

    std::ostringstream output;
    output << std::setprecision(17);
    output << "{\"schema\":\"nextengine.nonlocal.cuda_retained_tournament.v0\","
           << "\"status\":\"" << (passed ? "PASS" : "FAIL")
           << "\",\"profile_id\":\"" << profile.id
           << "\",\"profile_sha256\":\"" << sha256_hex(canonical_profile_json(profile))
           << "\",\"input_sha256\":\"" << fixture_input_hash(fixture)
           << "\",\"binary_sha256\":\"" << executable_hash()
           << "\",\"binary_bytes\":" << executable_bytes()
           << ",\"command\":\"nonlocal-feasibility --retained-tournament " << profile.id
           << " --warmup " << warmup << " --runs " << runs
           << "\",\"fixed_iterations\":" << profile.fixed_iterations
           << ",\"warmup_rounds\":" << warmup << ",\"measured_rounds\":" << runs
           << ",\"samples_per_identity\":" << runs
           << ",\"rotation\":[[\"atomic\",\"retained\"],[\"retained\",\"atomic\"]]"
           << ",\"each_measured_position_count\":48"
           << ",\"correctness\":{\"atomic_repeated_correspondence\":"
           << (atomic_repeated_correspondence ? "true" : "false")
           << ",\"retained_repeated_exact\":"
           << (retained_repeated_exact ? "true" : "false")
           << ",\"adjacent_correspondence\":"
           << (adjacent_correspondence ? "true" : "false")
           << ",\"adjacent_correspondence_is_diagnostic\":true"
           << ",\"denominator_admitted_by_frozen_hn3_profile\":true"
           << ",\"topology_passed\":" << (topology_passed ? "true" : "false")
           << ",\"finite_passed\":" << (finite_passed ? "true" : "false")
           << ",\"solve_passed\":" << (solve_passed ? "true" : "false")
           << ",\"momentum_passed\":" << (momentum_passed ? "true" : "false") << '}'
           << ",\"memory\":{\"source_atomic_bytes\":"
           << atomic_before.device_memory_bytes << ",\"retained_bytes\":"
           << retained_before.device_memory_bytes
           << ",\"two_instance_co_resident_bytes\":" << co_resident_memory
           << ",\"co_resident_ceiling_bytes\":" << CO_RESIDENT_CEILING << '}'
           << ",\"retained_vs_source_atomic\":{\"total_p95_speedup\":"
           << total_p95_speedup << ",\"source_atomic_total_p95_ms\":"
           << atomic_total.p95 << ",\"retained_total_p95_ms\":"
           << retained_total.p95 << '}'
           << ",\"identities\":{\"source_atomic\":{\"accumulation_identity\":\""
           << accumulation_identity(AccumulationMode::SourceAtomicV0)
           << "\",\"handoff_identity\":\"" << handoff_identity(HandoffMode::CopyV0)
           << "\",\"term_kernel_identity\":\""
           << term_kernel_identity(TermKernelMode::RuntimeV0)
           << "\",\"storage_identity\":\"" << storage_identity(StorageMode::StableSampleV0)
           << "\",\"statistics\":";
    append_stage_statistics(output, timings[0]);
    output << ",\"raw_total_ms\":";
    append_raw_totals(output, timings[0]);
    output << "},\"retained\":{\"accumulation_identity\":\""
           << accumulation_identity(AccumulationMode::GatherDirectedR0)
           << "\",\"handoff_identity\":\""
           << handoff_identity(HandoffMode::PointerSwapO1)
           << "\",\"term_kernel_identity\":\""
           << term_kernel_identity(TermKernelMode::SpecializedO2)
           << "\",\"storage_identity\":\"" << storage_identity(StorageMode::StableSampleV0)
           << "\",\"ordered_output_sha256\":\""
           << ordered_output_digest(retained_before.state)
           << "\",\"logical_csr_sha256\":\"" << csr_digest(retained_before)
           << "\",\"statistics\":";
    append_stage_statistics(output, timings[1]);
    output << ",\"raw_total_ms\":";
    append_raw_totals(output, timings[1]);
    output << "}},\"device\":" << device_json() << '}';
    return {passed, output.str()};
}

namespace {

struct DegreeDistribution {
    std::size_t minimum = 0;
    double mean = 0.0;
    std::size_t p95 = 0;
    std::size_t maximum = 0;
};

DegreeDistribution degree_distribution(const CapturedRun& run) {
    if (run.offsets.size() < 2U) {
        throw std::invalid_argument("degree distribution requires a non-empty CSR");
    }
    std::vector<std::size_t> degree;
    degree.reserve(run.offsets.size() - 1U);
    double sum = 0.0;
    for (std::size_t owner = 0; owner + 1U < run.offsets.size(); ++owner) {
        const std::size_t value = static_cast<std::size_t>(
            run.offsets[owner + 1U] - run.offsets[owner]);
        degree.push_back(value);
        sum += static_cast<double>(value);
    }
    const auto bounds = std::minmax_element(degree.begin(), degree.end());
    const std::size_t minimum = *bounds.first;
    const std::size_t maximum = *bounds.second;
    const std::size_t p95_index = static_cast<std::size_t>(
        std::ceil(0.95 * static_cast<double>(degree.size()))) - 1U;
    std::nth_element(degree.begin(), degree.begin() + p95_index, degree.end());
    return {minimum, sum / static_cast<double>(degree.size()), degree[p95_index], maximum};
}

void append_degree_distribution(
    std::ostringstream& output,
    const DegreeDistribution& value) {
    output << "{\"minimum\":" << value.minimum << ",\"mean\":" << value.mean
           << ",\"p95\":" << value.p95 << ",\"maximum\":" << value.maximum << '}';
}

std::string handoff_digest(const OracleResult& state) {
    std::ostringstream data;
    data << std::setprecision(17) << "handoff:" << state.next_position.size() << '|';
    for (std::size_t index = 0; index < state.next_position.size(); ++index) {
        const Vec3& position = state.next_position[index];
        const Vec3& velocity = state.final_velocity[index];
        data << position.x << ',' << position.y << ',' << position.z << ';'
             << velocity.x << ',' << velocity.y << ',' << velocity.z << '|';
    }
    return sha256_hex(data.str());
}

OracleResult remap_state_to_lattice(const Fixture& fixture, const OracleResult& state) {
    if (fixture.lattice_index_by_sample.size() != fixture.particles.size()) {
        throw std::invalid_argument("lattice remap requires one mapping per sample");
    }
    OracleResult result;
    const std::size_t count = fixture.particles.size();
    result.density.resize(count);
    result.source.resize(count);
    result.local_matrix.resize(count);
    result.predicted_position.resize(count);
    result.linearization_position.resize(count);
    result.next_position.resize(count);
    result.final_velocity.resize(count);
    for (std::size_t sample = 0; sample < count; ++sample) {
        const std::size_t lattice = static_cast<std::size_t>(
            fixture.lattice_index_by_sample[sample]);
        result.density[lattice] = state.density[sample];
        result.source[lattice] = state.source[sample];
        result.local_matrix[lattice] = state.local_matrix[sample];
        result.predicted_position[lattice] = state.predicted_position[sample];
        result.linearization_position[lattice] = state.linearization_position[sample];
        result.next_position[lattice] = state.next_position[sample];
        result.final_velocity[lattice] = state.final_velocity[sample];
    }
    result.energy = state.energy;
    result.directed_pairs = state.directed_pairs;
    result.maximum_degree = state.maximum_degree;
    result.normalized_momentum_residual = state.normalized_momentum_residual;
    return result;
}

std::string canonical_lattice_csr_digest(
    const Fixture& fixture,
    const CapturedRun& run) {
    const std::size_t count = fixture.particles.size();
    if (fixture.lattice_index_by_sample.size() != count) {
        throw std::invalid_argument("canonical lattice CSR requires a complete mapping");
    }
    std::vector<int> sample_by_lattice(count, -1);
    for (std::size_t sample = 0; sample < count; ++sample) {
        const int lattice = fixture.lattice_index_by_sample[sample];
        if (lattice < 0 || static_cast<std::size_t>(lattice) >= count
            || sample_by_lattice[static_cast<std::size_t>(lattice)] != -1) {
            throw std::runtime_error("sample-to-lattice mapping is not bijective");
        }
        sample_by_lattice[static_cast<std::size_t>(lattice)] = static_cast<int>(sample);
    }
    std::ostringstream data;
    data << "canonical_lattice_csr:" << count << '|';
    std::vector<int> row;
    for (std::size_t lattice = 0; lattice < count; ++lattice) {
        const int sample = sample_by_lattice[lattice];
        row.clear();
        row.reserve(static_cast<std::size_t>(
            run.offsets[static_cast<std::size_t>(sample + 1)]
            - run.offsets[static_cast<std::size_t>(sample)]));
        for (int slot = run.offsets[static_cast<std::size_t>(sample)];
             slot < run.offsets[static_cast<std::size_t>(sample + 1)]; ++slot) {
            const int neighbor = run.neighbors[static_cast<std::size_t>(slot)];
            row.push_back(fixture.lattice_index_by_sample[static_cast<std::size_t>(neighbor)]);
        }
        std::sort(row.begin(), row.end());
        data << lattice << ':';
        for (int neighbor : row) {
            data << neighbor << ',';
        }
        data << '|';
    }
    return sha256_hex(data.str());
}

bool run_v1_oracle_subset(const Profile& profile) {
    Profile subset = profile;
    subset.id = profile.id + ".oracle-subset";
    subset.lattice_x = profile.terms.surface_tension ? 4 : 6;
    subset.lattice_y = profile.terms.surface_tension ? 3 : 6;
    subset.lattice_z = profile.terms.surface_tension ? 2 : 6;
    subset.samples = static_cast<std::size_t>(subset.lattice_x)
        * static_cast<std::size_t>(subset.lattice_y)
        * static_cast<std::size_t>(subset.lattice_z);
    subset.max_samples = subset.samples;
    subset.max_neighbors = subset.samples;
    subset.max_directed_pairs = subset.samples * subset.samples;
    subset.trace_length = 1;
    subset.grid_margin = 0.0;
    subset.permutation_offset %= subset.samples;
    if (profile.terms.surface_tension) {
        subset.terms = {false, false, false, true};
        subset.kappa = 0.0;
        subset.lambda = 0.0;
        subset.mu = 0.0;
    }
    Fixture fixture = performance_fixture(subset, 1);
    fixture.advected = false;
    fixture.trace_length = 1;
    if (profile.terms.surface_tension) {
        const Vec3 center{
            0.5 * static_cast<double>(subset.lattice_x - 1) * subset.spacing,
            0.5 * static_cast<double>(subset.lattice_y - 1) * subset.spacing,
            0.5 * static_cast<double>(subset.lattice_z - 1) * subset.spacing};
        for (Particle& particle : fixture.particles) {
            particle.position = particle.position - center;
        }
    }
    const OracleResult cpu = run_cpu_gather_oracle(fixture);
    CudaBaseline gpu(
        fixture, AccumulationMode::GatherDirectedR0, HandoffMode::PointerSwapO1,
        TermKernelMode::SpecializedO2, StorageMode::StableSampleV0);
    const CapturedRun captured = gpu.execute(true);
    return !captured.local_solve_failed && captured.neighbor_build_valid
        && finite_state(captured.state) && exact_fixture_neighbors(fixture, captured)
        && valid_symmetric_neighbors(captured, fixture.particles.size())
        && compare_results(fixture, cpu, captured.state, profile.tolerances).passed;
}

bool run_surface_i2_preflight(const Profile& profile) {
    if (!profile.terms.surface_tension) {
        return true;
    }
    const Profile& stiff = find_profile("nuv-surface-stiff-16k-i2.v1");
    Fixture fixture = performance_fixture(stiff, 2);
    CudaBaseline baseline(
        fixture, AccumulationMode::GatherDirectedR0, HandoffMode::PointerSwapO1,
        TermKernelMode::SpecializedO2, StorageMode::StableSampleV0);
    const CapturedRun first = baseline.execute(true);
    const CapturedRun second = baseline.execute(true);
    return !first.local_solve_failed && !second.local_solve_failed
        && first.neighbor_build_valid && second.neighbor_build_valid
        && finite_state(first.state) && finite_state(second.state)
        && ordered_output_digest(first.state) == ordered_output_digest(second.state)
        && first.offsets == second.offsets && first.neighbors == second.neighbors;
}

void append_raw_stage_timings(
    std::ostringstream& output,
    const std::vector<StageTiming>& timings) {
    output << '[';
    for (std::size_t index = 0; index < timings.size(); ++index) {
        if (index != 0) {
            output << ',';
        }
        append_timing(output, timings[index]);
    }
    output << ']';
}

} // namespace

CommandReport run_cuda_np0_baseline(
    const Profile& profile,
    int warmup,
    int runs) {
    if (profile.record_version != 1) {
        throw std::invalid_argument("NP0 baseline requires a v1 profile");
    }
    if (warmup < 0 || warmup > 64 || runs < 1 || runs > 1000) {
        throw std::invalid_argument("NP0 baseline counts exceed bounded limits");
    }
    if (profile.advected
        && (warmup % profile.trace_length != 0 || runs % profile.trace_length != 0)) {
        throw std::invalid_argument(
            "advected NP0 warmup and run counts must contain whole trace epochs");
    }

    const CommandReport self_test = run_cuda_self_test(
        AccumulationMode::GatherDirectedR0, HandoffMode::PointerSwapO1,
        TermKernelMode::SpecializedO2, StorageMode::StableSampleV0);
    const bool oracle_subset_passed = self_test.passed && run_v1_oracle_subset(profile);
    const bool surface_i2_passed = oracle_subset_passed && run_surface_i2_preflight(profile);
    const Fixture fixture = performance_fixture(profile, profile.fixed_iterations);
    CudaBaseline baseline(
        fixture, AccumulationMode::GatherDirectedR0, HandoffMode::PointerSwapO1,
        TermKernelMode::SpecializedO2, StorageMode::StableSampleV0);

    baseline.reset_seed();
    const CapturedRun repeated_first = baseline.execute(true, profile.advected);
    baseline.reset_seed();
    const CapturedRun repeated_second = baseline.execute(true, profile.advected);
    const bool repeated_exact =
        ordered_output_digest(repeated_first.state)
            == ordered_output_digest(repeated_second.state)
        && repeated_first.offsets == repeated_second.offsets
        && repeated_first.neighbors == repeated_second.neighbors;

    bool coherent_permuted_correspondence = true;
    bool coherent_permuted_topology = true;
    std::string coherent_csr_sha256;
    std::string permuted_csr_sha256;
    if (profile.id == "nuv-water-50k-coherent.v1"
        || profile.id == "nuv-water-50k-permuted.v1") {
        const Profile& coherent_profile = find_profile("nuv-water-50k-coherent.v1");
        const Profile& permuted_profile = find_profile("nuv-water-50k-permuted.v1");
        const Fixture coherent_fixture = performance_fixture(
            coherent_profile, coherent_profile.fixed_iterations);
        const Fixture permuted_fixture = performance_fixture(
            permuted_profile, permuted_profile.fixed_iterations);
        CudaBaseline coherent_baseline(
            coherent_fixture, AccumulationMode::GatherDirectedR0,
            HandoffMode::PointerSwapO1, TermKernelMode::SpecializedO2,
            StorageMode::StableSampleV0);
        CudaBaseline permuted_baseline(
            permuted_fixture, AccumulationMode::GatherDirectedR0,
            HandoffMode::PointerSwapO1, TermKernelMode::SpecializedO2,
            StorageMode::StableSampleV0);
        const CapturedRun coherent = coherent_baseline.execute(true);
        const CapturedRun permuted = permuted_baseline.execute(true);
        const OracleResult remapped = remap_state_to_lattice(permuted_fixture, permuted.state);
        coherent_permuted_correspondence = compare_results(
            coherent_fixture, coherent.state, remapped, coherent_profile.tolerances).passed;
        coherent_csr_sha256 = canonical_lattice_csr_digest(coherent_fixture, coherent);
        permuted_csr_sha256 = canonical_lattice_csr_digest(permuted_fixture, permuted);
        coherent_permuted_topology = coherent_csr_sha256 == permuted_csr_sha256;
    }

    bool trace_finite = true;
    bool trace_topology = true;
    bool trace_capacity = true;
    bool trace_momentum = true;
    int first_preserving_interval = -1;
    int first_changing_interval = -1;
    std::ostringstream trace_material;
    std::ostringstream trace_steps;
    trace_steps << '[';
    std::string prior_state_digest = fixture_input_hash(fixture);
    std::string prior_membership;
    baseline.reset_seed();
    const int trace_length = profile.advected ? profile.trace_length : 1;
    for (int step = 0; step < trace_length; ++step) {
        CapturedRun captured = baseline.execute(true, profile.advected);
        const std::string output_sha256 = ordered_output_digest(captured.state);
        const std::string state_sha256 = handoff_digest(captured.state);
        const std::string logical_csr_sha256 = csr_digest(captured);
        const std::string active_membership_sha256 =
            canonical_lattice_csr_digest(fixture, captured);
        const DegreeDistribution degrees = degree_distribution(captured);
        const bool finite = finite_state(captured.state);
        const bool topology = valid_symmetric_neighbors(captured, fixture.particles.size());
        const bool capacity = captured.neighbor_build_valid
            && captured.state.directed_pairs <= profile.max_directed_pairs
            && captured.state.maximum_degree <= profile.max_neighbors;
        const bool momentum = captured.state.normalized_momentum_residual
            <= profile.tolerances.normalized_momentum_residual;
        trace_finite = trace_finite && finite && !captured.local_solve_failed;
        trace_topology = trace_topology && topology;
        trace_capacity = trace_capacity && capacity;
        trace_momentum = trace_momentum && momentum;
        if (step > 0 && active_membership_sha256 == prior_membership
            && first_preserving_interval < 0) {
            first_preserving_interval = step - 1;
        }
        if (step > 0 && active_membership_sha256 != prior_membership
            && first_changing_interval < 0) {
            first_changing_interval = step - 1;
        }
        trace_material << step << '|' << prior_state_digest << '|' << logical_csr_sha256
                       << '|' << active_membership_sha256 << '|' << output_sha256 << '|'
                       << state_sha256 << '|';
        if (step != 0) {
            trace_steps << ',';
        }
        trace_steps << "{\"step\":" << step << ",\"input_state_sha256\":\""
                    << prior_state_digest << "\",\"logical_csr_sha256\":\""
                    << logical_csr_sha256 << "\",\"active_membership_sha256\":\""
                    << active_membership_sha256 << "\",\"ordered_output_sha256\":\""
                    << output_sha256 << "\",\"handoff_state_sha256\":\""
                    << state_sha256 << "\",\"directed_pairs\":"
                    << captured.state.directed_pairs << ",\"degree\":";
        append_degree_distribution(trace_steps, degrees);
        trace_steps << ",\"mean_neighbor_storage_distance\":"
                    << captured.mean_neighbor_storage_distance
                    << ",\"p95_neighbor_storage_distance\":"
                    << captured.p95_neighbor_storage_distance
                    << ",\"normalized_momentum_residual\":"
                    << captured.state.normalized_momentum_residual
                    << ",\"finite\":" << (finite ? "true" : "false")
                    << ",\"topology_valid\":" << (topology ? "true" : "false")
                    << ",\"capacity_valid\":" << (capacity ? "true" : "false") << '}';
        prior_state_digest = state_sha256;
        prior_membership = active_membership_sha256;
    }
    trace_steps << ']';
    const std::string trace_sha256 = sha256_hex(trace_material.str());
    const bool requires_interval_classes =
        profile.id == "nuv-water-50k-advected.v1";
    const bool interval_classes_passed = !requires_interval_classes
        || (first_preserving_interval >= 0 && first_changing_interval >= 0);

    constexpr int CONDITIONING_RUNS = 256;
    baseline.reset_seed();
    for (int run = 0; run < CONDITIONING_RUNS; ++run) {
        const CapturedRun conditioned = baseline.execute(false, profile.advected);
        if (conditioned.local_solve_failed || !conditioned.neighbor_build_valid) {
            throw std::runtime_error("NP0 solver failed during GPU conditioning");
        }
        if (profile.advected && (run + 1) % profile.trace_length == 0) {
            baseline.reset_seed();
        }
    }
    baseline.reset_seed();
    bool measurement_valid = true;
    int failed_warmup = -1;
    int failed_measurement = -1;
    for (int run = 0; run < warmup; ++run) {
        const CapturedRun discarded = baseline.execute(false, profile.advected);
        if (discarded.local_solve_failed || !discarded.neighbor_build_valid) {
            measurement_valid = false;
            failed_warmup = run;
            break;
        }
        if (profile.advected && (run + 1) % profile.trace_length == 0) {
            baseline.reset_seed();
        }
    }
    if (profile.advected) {
        baseline.reset_seed();
    }
    std::vector<StageTiming> timings;
    timings.reserve(static_cast<std::size_t>(runs));
    for (int run = 0; run < runs; ++run) {
        const CapturedRun measured = baseline.execute(false, profile.advected);
        if (measured.local_solve_failed || !measured.neighbor_build_valid) {
            measurement_valid = false;
            failed_measurement = run;
            break;
        }
        timings.push_back(measured.timing);
        if (profile.advected && (run + 1) % profile.trace_length == 0) {
            baseline.reset_seed();
        }
    }

    const bool passed = self_test.passed && oracle_subset_passed && surface_i2_passed
        && repeated_exact && coherent_permuted_correspondence
        && coherent_permuted_topology && trace_finite && trace_topology
        && trace_capacity && trace_momentum && interval_classes_passed
        && measurement_valid && !timings.empty();
    const std::size_t directed_pairs = repeated_first.state.directed_pairs;
    const double bytes_per_sample = static_cast<double>(repeated_first.device_memory_bytes)
        / static_cast<double>(fixture.particles.size());
    const double bytes_per_edge = directed_pairs == 0U ? 0.0
        : static_cast<double>(repeated_first.device_memory_bytes)
            / static_cast<double>(directed_pairs);
    const std::string tier = warmup == 32 && runs == 96 ? "adjacent"
        : (warmup == 64 && runs >= 512 ? "decision" : "exploratory");

    std::ostringstream output;
    output << std::setprecision(17);
    output << "{\"schema\":\"nextengine.nonlocal.np0_baseline.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"candidate_class\":\"CORPUS\""
           << ",\"profile_id\":\"" << profile.id << "\",\"profile_sha256\":\""
           << sha256_hex(canonical_profile_json(profile)) << "\",\"seed_input_sha256\":\""
           << fixture_input_hash(fixture) << "\",\"binary_sha256\":\""
           << executable_hash() << "\",\"binary_bytes\":" << executable_bytes()
           << ",\"command\":\"nonlocal-feasibility --np0-baseline " << profile.id
           << " --warmup " << warmup << " --runs " << runs << "\""
           << ",\"measurement_tier\":\"" << tier << "\",\"warmup_runs\":"
           << warmup << ",\"measured_runs\":" << runs
           << ",\"conditioning_runs\":" << CONDITIONING_RUNS
           << ",\"completed_measured_runs\":" << timings.size()
           << ",\"trace_length\":" << trace_length
           << ",\"trace_sha256\":\"" << trace_sha256 << "\""
           << ",\"rebuild_reason\":\""
           << (profile.advected ? "substep_reference_changed" : "fixed_input_replay")
           << "\",\"reset_count\":"
           << (profile.advected
                   ? (CONDITIONING_RUNS + warmup + runs) / profile.trace_length + 6 : 0)
           << ",\"identity\":{\"accumulation\":\""
           << accumulation_identity(AccumulationMode::GatherDirectedR0)
           << "\",\"handoff\":\"" << handoff_identity(HandoffMode::PointerSwapO1)
           << "\",\"term_kernels\":\""
           << term_kernel_identity(TermKernelMode::SpecializedO2)
           << "\",\"storage\":\"" << storage_identity(StorageMode::StableSampleV0)
           << "\"},\"correctness\":{\"retained_self_test_passed\":"
           << (self_test.passed ? "true" : "false")
           << ",\"v1_oracle_subset_passed\":"
           << (oracle_subset_passed ? "true" : "false")
           << ",\"surface_i2_passed\":" << (surface_i2_passed ? "true" : "false")
           << ",\"fixed_frame_repeated_exact\":" << (repeated_exact ? "true" : "false")
           << ",\"coherent_permuted_correspondence\":"
           << (coherent_permuted_correspondence ? "true" : "false")
           << ",\"coherent_permuted_topology\":"
           << (coherent_permuted_topology ? "true" : "false")
           << ",\"coherent_canonical_csr_sha256\":\"" << coherent_csr_sha256
           << "\",\"permuted_canonical_csr_sha256\":\"" << permuted_csr_sha256
           << "\",\"trace_finite\":" << (trace_finite ? "true" : "false")
           << ",\"trace_topology_valid\":" << (trace_topology ? "true" : "false")
           << ",\"trace_capacity_valid\":" << (trace_capacity ? "true" : "false")
           << ",\"trace_momentum_valid\":" << (trace_momentum ? "true" : "false")
           << ",\"first_topology_preserving_interval\":"
           << first_preserving_interval << ",\"first_topology_changing_interval\":"
           << first_changing_interval << ",\"interval_classes_passed\":"
           << (interval_classes_passed ? "true" : "false")
           << ",\"interval_classes_required\":"
           << (requires_interval_classes ? "true" : "false")
           << ",\"measurement_valid\":" << (measurement_valid ? "true" : "false")
           << ",\"failed_warmup_index\":" << failed_warmup
           << ",\"failed_measurement_index\":" << failed_measurement << '}'
           << ",\"capacity\":{\"samples\":" << fixture.particles.size()
           << ",\"max_directed_pairs\":" << profile.max_directed_pairs
           << ",\"seed_directed_pairs\":" << directed_pairs
           << ",\"seed_maximum_degree\":" << repeated_first.state.maximum_degree
           << ",\"device_memory_bytes\":" << repeated_first.device_memory_bytes
           << ",\"bytes_per_sample\":" << bytes_per_sample
           << ",\"bytes_per_seed_edge\":" << bytes_per_edge << '}'
           << ",\"trace_steps\":" << trace_steps.str() << ",\"statistics\":";
    append_stage_statistics(output, timings);
    output << ",\"raw_stage_timings\":";
    append_raw_stage_timings(output, timings);
    output << ",\"device\":" << device_json() << '}';
    return {passed, output.str()};
}

namespace {

constexpr AccumulationMode P1_ACCUMULATION = AccumulationMode::GatherDirectedR0;
constexpr HandoffMode P1_HANDOFF = HandoffMode::PointerSwapO1;
constexpr TermKernelMode P1_TERMS = TermKernelMode::SpecializedO2;
constexpr StorageMode P1_STORAGE = StorageMode::StableSampleV0;

bool exact_p1_correspondence(
    const Fixture& fixture,
    const CapturedRun& retained,
    const CapturedRun& candidate,
    const Tolerances& tolerances) {
    return !retained.local_solve_failed && !candidate.local_solve_failed
        && retained.neighbor_build_valid && candidate.neighbor_build_valid
        && retained.reverse_map_valid && candidate.reverse_map_valid
        && retained.storage_map_valid && candidate.storage_map_valid
        && finite_state(retained.state) && finite_state(candidate.state)
        && valid_symmetric_neighbors(retained, fixture.particles.size())
        && valid_symmetric_neighbors(candidate, fixture.particles.size())
        && retained.offsets == candidate.offsets
        && retained.neighbors == candidate.neighbors
        && ordered_output_digest(retained.state) == ordered_output_digest(candidate.state)
        && compare_results(fixture, retained.state, candidate.state, tolerances).passed
        && retained.state.normalized_momentum_residual
            <= tolerances.normalized_momentum_residual
        && candidate.state.normalized_momentum_residual
            <= tolerances.normalized_momentum_residual;
}

struct P1Preflight {
    bool passed = true;
    std::string first_failure;
};

P1Preflight p1_tiny_preflight() {
    P1Preflight result;
    const Tolerances tolerances;
    for (const Fixture& fixture : oracle_fixtures()) {
        const OracleResult cpu = run_cpu_gather_oracle(fixture);
        CudaBaseline retained(fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE);
        CudaBaseline candidate(
            fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
            PairTraversalMode::FusedOwnerTermsP1);
        const CapturedRun retained_run = retained.execute(true);
        const CapturedRun candidate_run = candidate.execute(true);
        const bool passed = exact_p1_correspondence(
                fixture, retained_run, candidate_run, tolerances)
            && compare_results(fixture, cpu, candidate_run.state, tolerances).passed
            && exact_fixture_neighbors(fixture, candidate_run);
        if (!passed) {
            result.passed = false;
            if (result.first_failure.empty()) {
                result.first_failure = fixture.name;
            }
        }
    }
    return result;
}

bool p1_stiff_i2_preflight() {
    const Profile& profile = find_profile("nuv-surface-stiff-16k-i2.v1");
    const Fixture fixture = performance_fixture(profile, 2);
    CudaBaseline retained(fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE);
    CudaBaseline candidate(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1);
    return exact_p1_correspondence(
        fixture, retained.execute(true), candidate.execute(true), profile.tolerances);
}

double retained_pair_stage(const StageTiming& timing) {
    return timing.incompressibility + timing.viscosity + timing.surface_tension;
}

std::vector<double> pair_stage_values(
    const std::vector<StageTiming>& timings,
    bool fused) {
    std::vector<double> values;
    values.reserve(timings.size());
    for (const StageTiming& timing : timings) {
        values.push_back(fused ? timing.fused_owner_terms : retained_pair_stage(timing));
    }
    return values;
}

std::size_t fixture_pair_capacity(const Fixture& fixture) {
    return fixture.pair_capacity != 0U
        ? fixture.pair_capacity
        : (fixture.particles.size() <= 256U
                ? fixture.particles.size() * fixture.particles.size()
                : fixture.particles.size() * 123U);
}

bool compact_memory_correspondence(
    const Fixture& fixture,
    const CapturedRun& retained,
    const CapturedRun& candidate) {
    const bool eligible = fixture.particles.size()
        <= static_cast<std::size_t>(std::numeric_limits<std::uint16_t>::max());
    if (!eligible) {
        return retained.neighbor_id_bytes == sizeof(int)
            && candidate.neighbor_id_bytes == sizeof(int)
            && retained.device_memory_bytes == candidate.device_memory_bytes;
    }
    const std::size_t expected_savings = fixture_pair_capacity(fixture)
        * (sizeof(int) - sizeof(std::uint16_t));
    return retained.neighbor_id_bytes == sizeof(int)
        && candidate.neighbor_id_bytes == sizeof(std::uint16_t)
        && retained.device_memory_bytes >= candidate.device_memory_bytes
        && retained.device_memory_bytes - candidate.device_memory_bytes == expected_savings;
}

P1Preflight p2_tiny_preflight() {
    P1Preflight result;
    const Tolerances tolerances;
    for (const Fixture& fixture : oracle_fixtures()) {
        if (!fixture.terms.incompressibility
            || !(fixture.terms.bulk_viscosity || fixture.terms.shear_viscosity
                || fixture.terms.surface_tension)) {
            continue;
        }
        const OracleResult cpu = run_cpu_gather_oracle(fixture);
        CudaBaseline retained(
            fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
            PairTraversalMode::FusedOwnerTermsP1);
        CudaBaseline candidate(
            fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
            PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2);
        const CapturedRun retained_run = retained.execute(true);
        const CapturedRun candidate_run = candidate.execute(true);
        const bool passed = exact_p1_correspondence(
                fixture, retained_run, candidate_run, tolerances)
            && compare_results(fixture, cpu, candidate_run.state, tolerances).passed
            && exact_fixture_neighbors(fixture, candidate_run)
            && compact_memory_correspondence(fixture, retained_run, candidate_run);
        if (!passed) {
            result.passed = false;
            if (result.first_failure.empty()) {
                result.first_failure = fixture.name;
            }
        }
    }
    return result;
}

bool p2_stiff_i2_preflight() {
    const Profile& profile = find_profile("nuv-surface-stiff-16k-i2.v1");
    const Fixture fixture = performance_fixture(profile, 2);
    CudaBaseline retained(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1);
    CudaBaseline candidate(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2);
    const CapturedRun retained_run = retained.execute(true);
    const CapturedRun candidate_run = candidate.execute(true);
    return exact_p1_correspondence(
               fixture, retained_run, candidate_run, profile.tolerances)
        && compact_memory_correspondence(fixture, retained_run, candidate_run);
}

} // namespace

CommandReport run_cuda_p1_check(
    const Profile& profile,
    int iterations) {
    if (profile.record_version != 1 || iterations < 1 || iterations > 100) {
        throw std::invalid_argument("P1 check requires a v1 profile and 1..=100 iterations");
    }
    const CommandReport retained_self = run_cuda_self_test(
        P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE);
    const P1Preflight tiny = p1_tiny_preflight();
    const bool stiff_i2 = p1_stiff_i2_preflight();
    const Fixture fixture = performance_fixture(profile, iterations);
    CudaBaseline retained(fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE);
    CudaBaseline candidate(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1);
    const CapturedRun retained_run = retained.execute(true);
    const CapturedRun candidate_run = candidate.execute(true);
    const bool exact = exact_p1_correspondence(
        fixture, retained_run, candidate_run, profile.tolerances);
    const bool passed = retained_self.passed && tiny.passed && stiff_i2 && exact;

    std::ostringstream output;
    output << std::setprecision(17);
    output << "{\"schema\":\"nextengine.nonlocal.p1_check.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"candidate_class\":\"EXACT_WORK\""
           << ",\"candidate_identity\":\"fused-owner-terms-p1\",\"profile_id\":\""
           << profile.id << "\",\"iterations\":" << iterations
           << ",\"profile_sha256\":\"" << sha256_hex(canonical_profile_json(profile))
           << "\",\"input_sha256\":\"" << fixture_input_hash(fixture)
           << "\",\"binary_sha256\":\"" << executable_hash()
           << "\",\"correctness\":{\"retained_self_test_passed\":"
           << (retained_self.passed ? "true" : "false")
           << ",\"tiny_cpu_oracle_passed\":" << (tiny.passed ? "true" : "false")
           << ",\"tiny_first_failure\":\"" << tiny.first_failure
           << "\",\"stiff_surface_i2_passed\":" << (stiff_i2 ? "true" : "false")
           << ",\"target_exact\":" << (exact ? "true" : "false") << '}'
           << ",\"retained\":{\"ordered_output_sha256\":\""
           << ordered_output_digest(retained_run.state) << "\",\"logical_csr_sha256\":\""
           << csr_digest(retained_run) << "\",\"timing\":";
    append_timing(output, retained_run.timing);
    output << "},\"candidate\":{\"ordered_output_sha256\":\""
           << ordered_output_digest(candidate_run.state)
           << "\",\"logical_csr_sha256\":\"" << csr_digest(candidate_run)
           << "\",\"timing\":";
    append_timing(output, candidate_run.timing);
    output << "},\"device\":" << device_json() << '}';
    return {passed, output.str()};
}

CommandReport run_cuda_p1_tournament(
    const Profile& profile,
    int warmup,
    int runs) {
    if (profile.record_version != 1 || warmup != 32 || runs != 96) {
        throw std::invalid_argument("P1 tournament requires v1 --warmup 32 --runs 96");
    }
    if (profile.id == "nuv-surface-stiff-16k-i2.v1"
        || profile.id == "nuv-water-100k-report.v1") {
        throw std::invalid_argument("P1 tournament requires an adjacent decision profile");
    }
    const CommandReport check = run_cuda_p1_check(profile, profile.fixed_iterations);
    const Fixture fixture = performance_fixture(profile, profile.fixed_iterations);
    CudaBaseline retained(fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE);
    CudaBaseline candidate(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1);

    bool trace_exact = check.passed;
    int first_trace_mismatch = -1;
    std::ostringstream trace_material;
    std::string prior_state_digest = fixture_input_hash(fixture);
    retained.reset_seed();
    candidate.reset_seed();
    const int trace_length = profile.advected ? profile.trace_length : 1;
    CapturedRun retained_seed;
    CapturedRun candidate_seed;
    for (int step = 0; step < trace_length; ++step) {
        CapturedRun retained_run = retained.execute(true, profile.advected);
        CapturedRun candidate_run = candidate.execute(true, profile.advected);
        const bool step_exact = exact_p1_correspondence(
            fixture, retained_run, candidate_run, profile.tolerances);
        if (!step_exact && first_trace_mismatch < 0) {
            first_trace_mismatch = step;
        }
        trace_exact = trace_exact && step_exact;
        const std::string logical_csr_sha256 = csr_digest(retained_run);
        const std::string membership_sha256 =
            canonical_lattice_csr_digest(fixture, retained_run);
        const std::string output_sha256 = ordered_output_digest(retained_run.state);
        const std::string state_sha256 = handoff_digest(retained_run.state);
        trace_material << step << '|' << prior_state_digest << '|' << logical_csr_sha256
                       << '|' << membership_sha256 << '|' << output_sha256 << '|'
                       << state_sha256 << '|';
        prior_state_digest = state_sha256;
        if (step == 0) {
            retained_seed = std::move(retained_run);
            candidate_seed = std::move(candidate_run);
        }
    }
    const std::string trace_sha256 = sha256_hex(trace_material.str());
    constexpr int CONDITIONING_RUNS = 256;
    std::array<CudaBaseline*, 2> implementations = {&retained, &candidate};
    const auto execute_round = [&](int round, bool capture, std::vector<StageTiming>* timings) {
        for (int position = 0; position < 2; ++position) {
            const int identity = (round + position) % 2;
            CapturedRun run = implementations[static_cast<std::size_t>(identity)]->execute(
                capture, profile.advected);
            if (run.local_solve_failed || !run.neighbor_build_valid) {
                throw std::runtime_error("P1 identity failed during tournament");
            }
            if (timings != nullptr) {
                timings[identity].push_back(run.timing);
            }
        }
        if (profile.advected && (round + 1) % profile.trace_length == 0) {
            retained.reset_seed();
            candidate.reset_seed();
        }
    };

    retained.reset_seed();
    candidate.reset_seed();
    for (int round = 0; round < CONDITIONING_RUNS; ++round) {
        execute_round(round, false, nullptr);
    }
    retained.reset_seed();
    candidate.reset_seed();
    for (int round = 0; round < warmup; ++round) {
        execute_round(round, false, nullptr);
    }
    if (profile.advected) {
        retained.reset_seed();
        candidate.reset_seed();
    }
    std::vector<StageTiming> timings[2];
    timings[0].reserve(static_cast<std::size_t>(runs));
    timings[1].reserve(static_cast<std::size_t>(runs));
    for (int round = 0; round < runs; ++round) {
        execute_round(round, false, timings);
    }

    const Statistics retained_total = collect_statistics(timings[0], &StageTiming::total);
    const Statistics candidate_total = collect_statistics(timings[1], &StageTiming::total);
    const Statistics retained_pair = statistics(pair_stage_values(timings[0], false));
    const Statistics candidate_pair = statistics(pair_stage_values(timings[1], true));
    const double total_speedup = retained_total.p95 / candidate_total.p95;
    const double pair_speedup = retained_pair.p95 / candidate_pair.p95;
    const bool target_gate = pair_speedup >= 1.10 || total_speedup >= 1.05;
    const bool regression_gate = candidate_total.p95 <= retained_total.p95 * 1.02;
    const bool retain_candidate = trace_exact && target_gate && regression_gate;

    std::ostringstream output;
    output << std::setprecision(17);
    output << "{\"schema\":\"nextengine.nonlocal.p1_tournament.v1\",\"status\":\""
           << (trace_exact ? "PASS" : "FAIL") << "\",\"candidate_class\":\"EXACT_WORK\""
           << ",\"candidate_identity\":\"fused-owner-terms-p1\",\"profile_id\":\""
           << profile.id << "\",\"profile_sha256\":\""
           << sha256_hex(canonical_profile_json(profile)) << "\",\"input_sha256\":\""
           << fixture_input_hash(fixture) << "\",\"trace_sha256\":\"" << trace_sha256
           << "\",\"binary_sha256\":\"" << executable_hash()
           << "\",\"command\":\"nonlocal-feasibility --p1-tournament " << profile.id
           << " --warmup 32 --runs 96\",\"conditioning_runs\":" << CONDITIONING_RUNS
           << ",\"warmup_rounds\":" << warmup << ",\"measured_rounds\":" << runs
           << ",\"rotation\":[[\"retained\",\"candidate\"],[\"candidate\",\"retained\"]]"
           << ",\"correctness\":{\"check_passed\":" << (check.passed ? "true" : "false")
           << ",\"trace_exact\":" << (trace_exact ? "true" : "false")
           << ",\"first_trace_mismatch\":" << first_trace_mismatch
           << ",\"retained_seed_output_sha256\":\""
           << ordered_output_digest(retained_seed.state)
           << "\",\"candidate_seed_output_sha256\":\""
           << ordered_output_digest(candidate_seed.state)
           << "\",\"logical_csr_sha256\":\"" << csr_digest(retained_seed) << "\"}"
           << ",\"retention\":{\"target_gate\":" << (target_gate ? "true" : "false")
           << ",\"regression_gate\":" << (regression_gate ? "true" : "false")
           << ",\"retain_candidate\":" << (retain_candidate ? "true" : "false")
           << ",\"total_p95_speedup\":" << total_speedup
           << ",\"pair_stage_p95_speedup\":" << pair_speedup
           << ",\"retained_total_p95_ms\":" << retained_total.p95
           << ",\"candidate_total_p95_ms\":" << candidate_total.p95
           << ",\"retained_pair_stage_p95_ms\":" << retained_pair.p95
           << ",\"candidate_pair_stage_p95_ms\":" << candidate_pair.p95 << '}'
           << ",\"memory\":{\"retained_bytes\":" << retained_seed.device_memory_bytes
           << ",\"candidate_bytes\":" << candidate_seed.device_memory_bytes
           << ",\"additional_candidate_bytes\":"
           << (candidate_seed.device_memory_bytes - retained_seed.device_memory_bytes) << '}'
           << ",\"identities\":{\"retained\":{\"statistics\":";
    append_stage_statistics(output, timings[0]);
    output << ",\"raw_total_ms\":";
    append_raw_totals(output, timings[0]);
    output << "},\"candidate\":{\"statistics\":";
    append_stage_statistics(output, timings[1]);
    output << ",\"raw_total_ms\":";
    append_raw_totals(output, timings[1]);
    output << "}},\"device\":" << device_json() << '}';
    return {trace_exact, output.str()};
}

CommandReport run_cuda_p2_check(
    const Profile& profile,
    int iterations) {
    if (profile.record_version != 1 || iterations < 1 || iterations > 100) {
        throw std::invalid_argument("P2 check requires a v1 profile and 1..=100 iterations");
    }
    const CommandReport retained_self = run_cuda_self_test(
        P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE);
    const P1Preflight p1_tiny = p1_tiny_preflight();
    const bool p1_stiff = p1_stiff_i2_preflight();
    const P1Preflight p2_tiny = p2_tiny_preflight();
    const bool p2_stiff = p2_stiff_i2_preflight();
    const Fixture fixture = performance_fixture(profile, iterations);
    CudaBaseline retained(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1);
    CudaBaseline candidate(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2);
    const CapturedRun retained_run = retained.execute(true);
    const CapturedRun candidate_run = candidate.execute(true);
    const bool exact = exact_p1_correspondence(
        fixture, retained_run, candidate_run, profile.tolerances);
    const bool memory_exact = compact_memory_correspondence(
        fixture, retained_run, candidate_run);
    const bool compact_eligible = fixture.particles.size()
        <= static_cast<std::size_t>(std::numeric_limits<std::uint16_t>::max());
    const bool passed = retained_self.passed && p1_tiny.passed && p1_stiff
        && p2_tiny.passed && p2_stiff && exact && memory_exact;

    std::ostringstream output;
    output << std::setprecision(17);
    output << "{\"schema\":\"nextengine.nonlocal.p2_check.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"candidate_class\":\"EXACT_WORK\""
           << ",\"candidate_identity\":\"compact-csr-u16-p2\",\"profile_id\":\""
           << profile.id << "\",\"iterations\":" << iterations
           << ",\"profile_sha256\":\"" << sha256_hex(canonical_profile_json(profile))
           << "\",\"input_sha256\":\"" << fixture_input_hash(fixture)
           << "\",\"binary_sha256\":\"" << executable_hash()
           << "\",\"correctness\":{\"retained_self_test_passed\":"
           << (retained_self.passed ? "true" : "false")
           << ",\"p1_tiny_cpu_oracle_passed\":" << (p1_tiny.passed ? "true" : "false")
           << ",\"p1_stiff_surface_i2_passed\":" << (p1_stiff ? "true" : "false")
           << ",\"compact_tiny_cpu_oracle_passed\":"
           << (p2_tiny.passed ? "true" : "false")
           << ",\"compact_tiny_first_failure\":\"" << p2_tiny.first_failure
           << "\",\"compact_stiff_surface_i2_passed\":"
           << (p2_stiff ? "true" : "false")
           << ",\"target_exact\":" << (exact ? "true" : "false")
           << ",\"memory_exact\":" << (memory_exact ? "true" : "false") << '}'
           << ",\"representation\":{\"compact_eligible\":"
           << (compact_eligible ? "true" : "false")
           << ",\"selected_neighbor_id_bytes\":" << candidate_run.neighbor_id_bytes
           << ",\"fallback_u32\":" << (!compact_eligible ? "true" : "false") << '}'
           << ",\"retained\":{\"ordered_output_sha256\":\""
           << ordered_output_digest(retained_run.state) << "\",\"logical_csr_sha256\":\""
           << csr_digest(retained_run) << "\",\"device_memory_bytes\":"
           << retained_run.device_memory_bytes << ",\"timing\":";
    append_timing(output, retained_run.timing);
    output << "},\"candidate\":{\"ordered_output_sha256\":\""
           << ordered_output_digest(candidate_run.state)
           << "\",\"logical_csr_sha256\":\"" << csr_digest(candidate_run)
           << "\",\"device_memory_bytes\":" << candidate_run.device_memory_bytes
           << ",\"timing\":";
    append_timing(output, candidate_run.timing);
    output << "},\"device\":" << device_json() << '}';
    return {passed, output.str()};
}

CommandReport run_cuda_p2_tournament(
    const Profile& profile,
    int warmup,
    int runs) {
    if (profile.record_version != 1 || warmup != 32 || runs != 96) {
        throw std::invalid_argument("P2 tournament requires v1 --warmup 32 --runs 96");
    }
    if (profile.id == "nuv-surface-stiff-16k-i2.v1"
        || profile.id == "nuv-water-100k-report.v1") {
        throw std::invalid_argument("P2 tournament requires an adjacent decision profile");
    }
    const CommandReport check = run_cuda_p2_check(profile, profile.fixed_iterations);
    const Fixture fixture = performance_fixture(profile, profile.fixed_iterations);
    CudaBaseline retained(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1);
    CudaBaseline candidate(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2);

    bool trace_exact = check.passed;
    bool trace_memory_exact = true;
    int first_trace_mismatch = -1;
    std::ostringstream trace_material;
    std::string prior_state_digest = fixture_input_hash(fixture);
    retained.reset_seed();
    candidate.reset_seed();
    const int trace_length = profile.advected ? profile.trace_length : 1;
    CapturedRun retained_seed;
    CapturedRun candidate_seed;
    for (int step = 0; step < trace_length; ++step) {
        CapturedRun retained_run = retained.execute(true, profile.advected);
        CapturedRun candidate_run = candidate.execute(true, profile.advected);
        const bool step_exact = exact_p1_correspondence(
            fixture, retained_run, candidate_run, profile.tolerances);
        const bool step_memory = compact_memory_correspondence(
            fixture, retained_run, candidate_run);
        if ((!step_exact || !step_memory) && first_trace_mismatch < 0) {
            first_trace_mismatch = step;
        }
        trace_exact = trace_exact && step_exact;
        trace_memory_exact = trace_memory_exact && step_memory;
        const std::string logical_csr_sha256 = csr_digest(retained_run);
        const std::string membership_sha256 =
            canonical_lattice_csr_digest(fixture, retained_run);
        const std::string output_sha256 = ordered_output_digest(retained_run.state);
        const std::string state_sha256 = handoff_digest(retained_run.state);
        trace_material << step << '|' << prior_state_digest << '|' << logical_csr_sha256
                       << '|' << membership_sha256 << '|' << output_sha256 << '|'
                       << state_sha256 << '|';
        prior_state_digest = state_sha256;
        if (step == 0) {
            retained_seed = std::move(retained_run);
            candidate_seed = std::move(candidate_run);
        }
    }
    const std::string trace_sha256 = sha256_hex(trace_material.str());
    constexpr int CONDITIONING_RUNS = 256;
    std::array<CudaBaseline*, 2> implementations = {&retained, &candidate};
    const auto execute_round = [&](int round, bool capture, std::vector<StageTiming>* timings) {
        for (int position = 0; position < 2; ++position) {
            const int identity = (round + position) % 2;
            CapturedRun run = implementations[static_cast<std::size_t>(identity)]->execute(
                capture, profile.advected);
            if (run.local_solve_failed || !run.neighbor_build_valid) {
                throw std::runtime_error("P2 identity failed during tournament");
            }
            if (timings != nullptr) {
                timings[identity].push_back(run.timing);
            }
        }
        if (profile.advected && (round + 1) % profile.trace_length == 0) {
            retained.reset_seed();
            candidate.reset_seed();
        }
    };

    retained.reset_seed();
    candidate.reset_seed();
    for (int round = 0; round < CONDITIONING_RUNS; ++round) {
        execute_round(round, false, nullptr);
    }
    retained.reset_seed();
    candidate.reset_seed();
    for (int round = 0; round < warmup; ++round) {
        execute_round(round, false, nullptr);
    }
    if (profile.advected) {
        retained.reset_seed();
        candidate.reset_seed();
    }
    std::vector<StageTiming> timings[2];
    timings[0].reserve(static_cast<std::size_t>(runs));
    timings[1].reserve(static_cast<std::size_t>(runs));
    for (int round = 0; round < runs; ++round) {
        execute_round(round, false, timings);
    }

    const Statistics retained_total = collect_statistics(timings[0], &StageTiming::total);
    const Statistics candidate_total = collect_statistics(timings[1], &StageTiming::total);
    const Statistics retained_pair = statistics(pair_stage_values(timings[0], true));
    const Statistics candidate_pair = statistics(pair_stage_values(timings[1], true));
    const double total_speedup = retained_total.p95 / candidate_total.p95;
    const double pair_speedup = retained_pair.p95 / candidate_pair.p95;
    const bool target_gate = pair_speedup >= 1.10 || total_speedup >= 1.05;
    const bool regression_gate = candidate_total.p95 <= retained_total.p95 * 1.02;
    const bool retain_candidate = trace_exact && trace_memory_exact
        && target_gate && regression_gate;
    const std::size_t memory_saved = retained_seed.device_memory_bytes
        - candidate_seed.device_memory_bytes;

    std::ostringstream output;
    output << std::setprecision(17);
    output << "{\"schema\":\"nextengine.nonlocal.p2_tournament.v1\",\"status\":\""
           << (trace_exact && trace_memory_exact ? "PASS" : "FAIL")
           << "\",\"candidate_class\":\"EXACT_WORK\""
           << ",\"candidate_identity\":\"compact-csr-u16-p2\",\"profile_id\":\""
           << profile.id << "\",\"profile_sha256\":\""
           << sha256_hex(canonical_profile_json(profile)) << "\",\"input_sha256\":\""
           << fixture_input_hash(fixture) << "\",\"trace_sha256\":\"" << trace_sha256
           << "\",\"binary_sha256\":\"" << executable_hash()
           << "\",\"command\":\"nonlocal-feasibility --p2-tournament " << profile.id
           << " --warmup 32 --runs 96\",\"conditioning_runs\":" << CONDITIONING_RUNS
           << ",\"warmup_rounds\":" << warmup << ",\"measured_rounds\":" << runs
           << ",\"rotation\":[[\"retained-p1\",\"candidate-p2\"],"
              "[\"candidate-p2\",\"retained-p1\"]]"
           << ",\"correctness\":{\"check_passed\":" << (check.passed ? "true" : "false")
           << ",\"trace_exact\":" << (trace_exact ? "true" : "false")
           << ",\"trace_memory_exact\":" << (trace_memory_exact ? "true" : "false")
           << ",\"first_trace_mismatch\":" << first_trace_mismatch
           << ",\"retained_seed_output_sha256\":\""
           << ordered_output_digest(retained_seed.state)
           << "\",\"candidate_seed_output_sha256\":\""
           << ordered_output_digest(candidate_seed.state)
           << "\",\"logical_csr_sha256\":\"" << csr_digest(retained_seed) << "\"}"
           << ",\"retention\":{\"target_gate\":" << (target_gate ? "true" : "false")
           << ",\"regression_gate\":" << (regression_gate ? "true" : "false")
           << ",\"retain_candidate\":" << (retain_candidate ? "true" : "false")
           << ",\"total_p95_speedup\":" << total_speedup
           << ",\"pair_stage_p95_speedup\":" << pair_speedup
           << ",\"retained_total_p95_ms\":" << retained_total.p95
           << ",\"candidate_total_p95_ms\":" << candidate_total.p95
           << ",\"retained_pair_stage_p95_ms\":" << retained_pair.p95
           << ",\"candidate_pair_stage_p95_ms\":" << candidate_pair.p95 << '}'
           << ",\"memory\":{\"retained_bytes\":" << retained_seed.device_memory_bytes
           << ",\"candidate_bytes\":" << candidate_seed.device_memory_bytes
           << ",\"saved_bytes\":" << memory_saved
           << ",\"retained_neighbor_id_bytes\":" << retained_seed.neighbor_id_bytes
           << ",\"candidate_neighbor_id_bytes\":" << candidate_seed.neighbor_id_bytes << '}'
           << ",\"identities\":{\"retained_p1\":{\"statistics\":";
    append_stage_statistics(output, timings[0]);
    output << ",\"raw_total_ms\":";
    append_raw_totals(output, timings[0]);
    output << "},\"candidate_p2\":{\"statistics\":";
    append_stage_statistics(output, timings[1]);
    output << ",\"raw_total_ms\":";
    append_raw_totals(output, timings[1]);
    output << "}},\"device\":" << device_json() << '}';
    return {trace_exact && trace_memory_exact, output.str()};
}

} // namespace nextengine::nonlocal
