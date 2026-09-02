#include "cuda_baseline.hpp"

#include "sha256.hpp"

#include <cub/cub.cuh>
#include <cuda_runtime.h>

#include <algorithm>
#include <array>
#include <chrono>
#include <cmath>
#include <condition_variable>
#include <memory>
#include <mutex>
#include <optional>
#include <thread>
#include <cstdint>
#include <cstring>
#include <fstream>
#include <iomanip>
#include <iterator>
#include <limits>
#include <map>
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

__global__ void measure_max_displacement_squared(
    const float3* reference,
    const float3* anchor,
    unsigned long long* maximum_bits,
    int count) {
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }
    const double dx = static_cast<double>(reference[particle].x)
        - static_cast<double>(anchor[particle].x);
    const double dy = static_cast<double>(reference[particle].y)
        - static_cast<double>(anchor[particle].y);
    const double dz = static_cast<double>(reference[particle].z)
        - static_cast<double>(anchor[particle].z);
    double squared = dx * dx + dy * dy + dz * dz;
    if (!isfinite(squared)) {
        squared = INFINITY;
    }
    atomicMax(maximum_bits, static_cast<unsigned long long>(__double_as_longlong(squared)));
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

template <bool Fill, typename NeighborIndex>
__global__ void visit_grid_neighbors_superset_p4(
    const float3* anchor,
    const int* sorted_indices,
    const int* cell_start,
    const int* cell_end,
    int* counts,
    const int* offsets,
    NeighborIndex* neighbors,
    int count,
    float3 origin,
    float cell_size,
    float superset_horizon,
    int3 dimensions,
    int cell_radius,
    int* error_flag) {
    const int particle = blockIdx.x * blockDim.x + threadIdx.x;
    if (particle >= count) {
        return;
    }
    const float3 point = anchor[particle];
    const int own_key = cell_key(point, origin, cell_size, dimensions);
    const int own_x = own_key % dimensions.x;
    const int own_y = (own_key / dimensions.x) % dimensions.y;
    const int own_z = own_key / (dimensions.x * dimensions.y);
    int cursor = Fill ? offsets[particle] : 0;
    int degree = 0;
    const float support_squared =
        superset_horizon * superset_horizon * (1.0F + 2.0e-6F);
    for (int dz = -cell_radius; dz <= cell_radius; ++dz) {
        const int z = own_z + dz;
        if (z < 0 || z >= dimensions.z) {
            continue;
        }
        for (int dy = -cell_radius; dy <= cell_radius; ++dy) {
            const int y = own_y + dy;
            if (y < 0 || y >= dimensions.y) {
                continue;
            }
            for (int dx = -cell_radius; dx <= cell_radius; ++dx) {
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
                    const float3 delta = subtract3(point, anchor[candidate]);
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

template <bool Fill, typename NeighborIndex = int>
__global__ void visit_grid_neighbors_cell_sorted(
    const float3* logical_position,
    const int* sorted_indices,
    const int* storage_to_sample,
    const int* sample_to_storage,
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
                                neighbors[cursor++] = static_cast<NeighborIndex>(
                                    sample_to_storage[candidate_sample]);
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

__global__ void scatter_dynamic_state(
    const float3* physical_position,
    const float3* physical_velocity,
    const int* storage_to_sample,
    float3* logical_position,
    float3* logical_velocity,
    int count) {
    const int storage = blockIdx.x * blockDim.x + threadIdx.x;
    if (storage >= count) {
        return;
    }
    const int sample = storage_to_sample[storage];
    logical_position[sample] = physical_position[storage];
    logical_velocity[sample] = physical_velocity[storage];
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

__global__ void clamp_analytic_box_contact(
    float3* position,
    const std::uint8_t* fixed,
    int count,
    float3 lower,
    float3 upper) {
    const int index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= count || fixed[index] != 0U) {
        return;
    }
    position[index].x = fminf(fmaxf(position[index].x, lower.x), upper.x);
    position[index].y = fminf(fmaxf(position[index].y, lower.y), upper.y);
    position[index].z = fminf(fmaxf(position[index].z, lower.z), upper.z);
}

// NGQ8 spill clamp: interior divider slab with one opening plus a shelf
// left of the divider. Runs after the outer-box clamp; positional only.
__global__ void clamp_spill_contact(
    float3* position,
    const std::uint8_t* fixed,
    int count,
    float radius,
    float shelf_top,
    float wall_x0,
    float wall_x1,
    float opening_y0,
    float opening_y1,
    float opening_z0,
    float opening_z1,
    float lip_margin) {
    const int index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= count || fixed[index] != 0U) {
        return;
    }
    float3 p = position[index];
    // NGQ8 revision 8: the opening floor always keeps one radius so it stays
    // level with the shelf lift; `flush` applies only to the free faces.
    if (p.x >= wall_x0 && p.x <= wall_x1) {
        // NGQ8 revision 6: inside the slab a sample can only be in the pipe;
        // keep it in the opening window instead of pushing it through a face.
        p.y = fminf(fmaxf(p.y, opening_y0 + radius), opening_y1 - lip_margin);
        p.z = fminf(fmaxf(p.z, opening_z0 + lip_margin), opening_z1 - lip_margin);
    } else if (p.x > wall_x0 - radius && p.x < wall_x1 + radius) {
        const bool window = p.y > opening_y0 && p.y < opening_y1 && p.z > opening_z0
            && p.z < opening_z1;
        if (window) {
            p.y = fminf(fmaxf(p.y, opening_y0 + radius), opening_y1 - lip_margin);
            p.z = fminf(fmaxf(p.z, opening_z0 + lip_margin), opening_z1 - lip_margin);
        } else if (p.x < wall_x0) {
            p.x = wall_x0 - radius;
        } else {
            p.x = wall_x1 + radius;
        }
    }
    if (p.x < wall_x0 && p.y < shelf_top + radius) {
        p.y = shelf_top + radius;
    }
    position[index] = p;
}

// NGQ8 revision 9: under-floor pipe scene in device units (metres).
struct SpillPipeDevice {
    float shelf_top;
    float wall_x0;
    float wall_x1;
    float shaft_x0;
    float shaft_x1;
    float duct_y0;
    float duct_y1;
    float z0;
    float z1;
    float pipe_x1;
    float pipe_wall;
};

__device__ inline bool spill_pipe_in_box(float3 p, float3 lo, float3 hi) {
    return p.x >= lo.x && p.x <= hi.x && p.y >= lo.y && p.y <= hi.y && p.z >= lo.z && p.z <= hi.z;
}

__device__ inline float3 spill_pipe_clamp_box(float3 p, float3 lo, float3 hi) {
    return make_float3(
        fminf(fmaxf(p.x, lo.x), hi.x), fminf(fmaxf(p.y, lo.y), hi.y),
        fminf(fmaxf(p.z, lo.z), hi.z));
}

__device__ inline float spill_pipe_dist2(float3 a, float3 b) {
    const float dx = a.x - b.x;
    const float dy = a.y - b.y;
    const float dz = a.z - b.z;
    return dx * dx + dy * dy + dz * dz;
}

// Allowed sample centres: inside a channel shrunk by one radius, or outside
// every solid expanded by one radius. Channels end one radius past their
// mouths so they join the free region without a gap.
__device__ inline bool spill_pipe_allowed(float3 p, const SpillPipeDevice& g, float r) {
    const float3 shaft_lo = make_float3(g.shaft_x0 + r, g.duct_y0 + r, g.z0 + r);
    const float3 shaft_hi = make_float3(g.shaft_x1 - r, g.shelf_top + r, g.z1 - r);
    const float3 duct_lo = make_float3(g.shaft_x0 + r, g.duct_y0 + r, g.z0 + r);
    const float3 duct_hi = make_float3(g.pipe_x1 + r, g.duct_y1 - r, g.z1 - r);
    if (spill_pipe_in_box(p, shaft_lo, shaft_hi) || spill_pipe_in_box(p, duct_lo, duct_hi)) {
        return true;
    }
    const bool shelf = p.x < g.wall_x0 + r && p.y < g.shelf_top + r;
    const bool wall = p.x > g.wall_x0 - r && p.x < g.wall_x1 + r;
    const bool pipe = p.x > g.wall_x1 - r && p.x < g.pipe_x1 + r
        && p.y > g.duct_y0 - g.pipe_wall - r && p.y < g.duct_y1 + g.pipe_wall + r
        && p.z > g.z0 - g.pipe_wall - r && p.z < g.z1 + g.pipe_wall + r;
    return !(shelf || wall || pipe);
}

// NGQ8 revision 9 clamp: a sample inside a solid moves to the nearest of
// the channel interiors or the exit faces of the solids that contain it,
// repeated a few times so a concave corner (shelf under the divider) is
// left through two short moves instead of one long one. Faces glued to
// another solid (shelf +x, pipe body -x) are never exits. Positional only.
__global__ void clamp_spill_pipe_contact(
    float3* position,
    const std::uint8_t* fixed,
    int count,
    float radius,
    SpillPipeDevice g) {
    const int index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= count || fixed[index] != 0U) {
        return;
    }
    float3 p = position[index];
    const float r = radius;
    const float3 shaft_lo = make_float3(g.shaft_x0 + r, g.duct_y0 + r, g.z0 + r);
    const float3 shaft_hi = make_float3(g.shaft_x1 - r, g.shelf_top + r, g.z1 - r);
    const float3 duct_lo = make_float3(g.shaft_x0 + r, g.duct_y0 + r, g.z0 + r);
    const float3 duct_hi = make_float3(g.pipe_x1 + r, g.duct_y1 - r, g.z1 - r);
    for (int pass = 0; pass < 4 && !spill_pipe_allowed(p, g, r); ++pass) {
        float3 best = p;
        float best_dist = 3.4e38F;
        const auto consider = [&](float3 candidate) {
            const float dist = spill_pipe_dist2(p, candidate);
            if (dist < best_dist) {
                best_dist = dist;
                best = candidate;
            }
        };
        consider(spill_pipe_clamp_box(p, shaft_lo, shaft_hi));
        consider(spill_pipe_clamp_box(p, duct_lo, duct_hi));
        if (p.x < g.wall_x0 + r && p.y < g.shelf_top + r) {
            consider(make_float3(p.x, g.shelf_top + r, p.z));
        }
        if (p.x > g.wall_x0 - r && p.x < g.wall_x1 + r) {
            consider(make_float3(g.wall_x0 - r, p.y, p.z));
            consider(make_float3(g.wall_x1 + r, p.y, p.z));
        }
        const float pipe_y0 = g.duct_y0 - g.pipe_wall - r;
        const float pipe_y1 = g.duct_y1 + g.pipe_wall + r;
        const float pipe_z0 = g.z0 - g.pipe_wall - r;
        const float pipe_z1 = g.z1 + g.pipe_wall + r;
        if (p.x > g.wall_x1 - r && p.x < g.pipe_x1 + r && p.y > pipe_y0 && p.y < pipe_y1
            && p.z > pipe_z0 && p.z < pipe_z1) {
            consider(make_float3(g.pipe_x1 + r, p.y, p.z));
            consider(make_float3(p.x, pipe_y0, p.z));
            consider(make_float3(p.x, pipe_y1, p.z));
            consider(make_float3(p.x, p.y, pipe_z0));
            consider(make_float3(p.x, p.y, pipe_z1));
        }
        p = best;
    }
    position[index] = p;
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

template <typename NeighborIndex>
__global__ void compute_density_filtered_p4(
    const float3* position,
    const float3* reference,
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
    const float active_squared = horizon * horizon * (1.0F + 2.0e-6F);
    for (int slot = offsets[particle]; slot < offsets[particle + 1]; ++slot) {
        const int neighbor = static_cast<int>(neighbors[slot]);
        const float3 reference_delta = subtract3(reference[particle], reference[neighbor]);
        if (dot3(reference_delta, reference_delta) > active_squared) {
            continue;
        }
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

template <typename NeighborIndex, bool BulkEnabled, bool ShearEnabled, bool SurfaceEnabled,
    bool FilterActiveP4 = false>
__global__ void accumulate_fused_owner_terms_p1(
    const float3* reference,
    const float3* current,
    const float* density,
    const int* offsets,
    const NeighborIndex* neighbors,
    const std::uint8_t* fixed,
    bool density_only_support,
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
    // NGQ7 revision 4: a fixed owner's row is never read back (the update
    // keeps it at its reference), so density-only support skips it.
    if (density_only_support && fixed[particle] != 0U) {
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
        if constexpr (FilterActiveP4) {
            const float3 active_delta = subtract3(reference[particle], reference[neighbor]);
            const float active_squared = horizon * horizon * (1.0F + 2.0e-6F);
            if (dot3(active_delta, active_delta) > active_squared) {
                continue;
            }
        }
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
        // NGQ7 revision 2: a fixed boundary sample supports density only.
        const bool density_only_neighbor = density_only_support && fixed[neighbor] != 0U;

        if constexpr (BulkEnabled || ShearEnabled) {
            // Retained O2 specialized viscosity association and add order.
            const float3 reference_delta =
                subtract3(reference[neighbor], reference[particle]);
            const float viscosity_radius = length3(reference_delta);
            if (!density_only_neighbor && viscosity_radius > CUDA_PAIR_EPSILON) {
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
            if (!density_only_neighbor && surface_radius > CUDA_PAIR_EPSILON) {
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

template <typename NeighborIndex, bool FilterActiveP4 = false>
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
        if constexpr (FilterActiveP4) {
            const float active_squared = horizon * horizon * (1.0F + 2.0e-6F);
            if (dot3(reference_delta, reference_delta) > active_squared) {
                continue;
            }
        }
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
    double storage_remap = 0.0;
    double prediction = 0.0;
    double neighbor_construction = 0.0;
    double density = 0.0;
    double buffer_reset = 0.0;
    double incompressibility = 0.0;
    double viscosity = 0.0;
    double surface_tension = 0.0;
    double fused_owner_terms = 0.0;
    double local_update = 0.0;
    double contact = 0.0;
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
    std::size_t candidate_directed_pairs = 0;
    double layout_setup_ms = 0.0;
    double storage_setup_ms = 0.0;
    double mean_neighbor_storage_distance = 0.0;
    double p95_neighbor_storage_distance = 0.0;
    bool local_solve_failed = false;
    bool reverse_map_valid = true;
    bool storage_map_valid = true;
    bool neighbor_build_valid = true;
    bool neighbor_rebuilt = false;
    bool neighbor_reused = false;
    double maximum_anchor_displacement = 0.0;
    std::string storage_to_sample_sha256;
    std::string sample_to_storage_sha256;
    std::string physical_csr_sha256;
};

struct EventInterval {
    enum class Stage {
        StorageRemap,
        Prediction,
        Neighbor,
        Density,
        Reset,
        Incompressibility,
        Viscosity,
        Surface,
        FusedOwnerTerms,
        Update,
        Contact,
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

int exact_lattice_span(double minimum, double maximum, double spacing) {
    const double cells = (maximum - minimum) / spacing;
    const long long rounded = std::llround(cells);
    if (rounded <= 0 || std::abs(cells - static_cast<double>(rounded)) > 1.0e-12
        || rounded > std::numeric_limits<int>::max()) {
        throw std::invalid_argument("static boundary extent is not an exact lattice span");
    }
    return static_cast<int>(rounded);
}

void append_static_outer_complement(const Profile& profile, Fixture& fixture) {
    if (profile.static_boundary_samples == 0U) {
        return;
    }
    if (profile.record_version < 3 || profile.static_boundary_layers <= 0
        || profile.static_boundary_samples > profile.max_static_boundary_samples) {
        throw std::invalid_argument("invalid v3 static boundary profile");
    }
    const int x_count = exact_lattice_span(
        profile.basin_min.x, profile.basin_max.x, profile.spacing);
    const int y_count = exact_lattice_span(
        profile.basin_min.y, profile.basin_max.y, profile.spacing);
    const int z_count = exact_lattice_span(
        profile.basin_min.z, profile.basin_max.z, profile.spacing);
    const int layers = profile.static_boundary_layers;
    const std::size_t before = fixture.particles.size();
    for (int x = -layers; x < x_count + layers; ++x) {
        for (int y = -layers; y < y_count + layers; ++y) {
            for (int z = -layers; z < z_count + layers; ++z) {
                if (x >= 0 && x < x_count && y >= 0 && y < y_count
                    && z >= 0 && z < z_count) {
                    continue;
                }
                const Vec3 position{
                    profile.basin_min.x + profile.particle_radius + x * profile.spacing,
                    profile.basin_min.y + profile.particle_radius + y * profile.spacing,
                    profile.basin_min.z + profile.particle_radius + z * profile.spacing,
                };
                const int lattice_index = static_cast<int>(fixture.particles.size());
                fixture.particles.push_back({position, {}, true});
                fixture.lattice_index_by_sample.push_back(lattice_index);
            }
        }
    }
    if (fixture.particles.size() - before != profile.static_boundary_samples) {
        throw std::runtime_error("static boundary sample count does not match profile");
    }
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
    fixture.analytic_box_contact = profile.contact == "analytic_box_clamp_gpu_v1";
    fixture.contact_minimum = profile.basin_min;
    fixture.contact_maximum = profile.basin_max;
    fixture.particle_radius = profile.particle_radius;
    fixture.advected = profile.advected;
    fixture.trace_length = profile.trace_length;
    fixture.particles.reserve(profile.samples + profile.static_boundary_samples);
    if (profile.record_version >= 1) {
        fixture.lattice_index_by_sample.reserve(
            profile.samples + profile.static_boundary_samples);
    }
    const double center_x = profile.origin.x
        + 0.5 * static_cast<double>(profile.lattice_x - 1) * profile.spacing;
    const double center_y = profile.origin.y
        + 0.5 * static_cast<double>(profile.lattice_y - 1) * profile.spacing;
    const double center_z = profile.origin.z
        + 0.5 * static_cast<double>(profile.lattice_z - 1) * profile.spacing;
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
            ? Vec3{
                  profile.origin.x + x * profile.spacing,
                  profile.origin.y + y * profile.spacing,
                  profile.origin.z + z * profile.spacing}
            : Vec3{
                  center_x + profile.lattice_scale
                      * (profile.origin.x + x * profile.spacing - center_x),
                  center_y + profile.lattice_scale
                      * (profile.origin.y + y * profile.spacing - center_y),
                  center_z + profile.lattice_scale
                      * (profile.origin.z + z * profile.spacing - center_z)};
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
    append_static_outer_complement(profile, fixture);
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
    if (fixture.analytic_box_contact) {
        data << "|analytic_box_contact:" << fixture.contact_minimum.x << ','
             << fixture.contact_minimum.y << ',' << fixture.contact_minimum.z << ';'
             << fixture.contact_maximum.x << ',' << fixture.contact_maximum.y << ','
             << fixture.contact_maximum.z << ';' << fixture.particle_radius;
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

enum class DynamicStorageMode {
    Retained,
    DynamicCellLocalP3,
};

enum class NeighborReuseMode {
    RebuildEverySolve,
    CertifiedVerletP4,
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
        NeighborEncodingMode neighbor_encoding_mode = NeighborEncodingMode::U32Retained,
        DynamicStorageMode dynamic_storage_mode = DynamicStorageMode::Retained,
        NeighborReuseMode neighbor_reuse_mode = NeighborReuseMode::RebuildEverySolve)
        : fixture_(fixture),
          accumulation_mode_(accumulation_mode),
          handoff_mode_(handoff_mode),
          term_kernel_mode_(term_kernel_mode),
          storage_mode_(storage_mode),
          pair_traversal_mode_(pair_traversal_mode),
          neighbor_encoding_mode_(neighbor_encoding_mode),
          dynamic_storage_mode_(dynamic_storage_mode),
          neighbor_reuse_mode_(neighbor_reuse_mode),
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
        if (fixture_.analytic_box_contact
            && (!finite(fixture_.contact_minimum) || !finite(fixture_.contact_maximum)
                || !std::isfinite(fixture_.particle_radius)
                || fixture_.particle_radius <= 0.0
                || fixture_.contact_maximum.x - fixture_.contact_minimum.x
                    < 2.0 * fixture_.particle_radius
                || fixture_.contact_maximum.y - fixture_.contact_minimum.y
                    < 2.0 * fixture_.particle_radius
                || fixture_.contact_maximum.z - fixture_.contact_minimum.z
                    < 2.0 * fixture_.particle_radius)) {
            throw std::invalid_argument("invalid analytic box contact fixture");
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
        if (dynamic_storage_mode_ == DynamicStorageMode::DynamicCellLocalP3
            && (storage_mode_ != StorageMode::StableSampleV0
                || pair_traversal_mode_ != PairTraversalMode::FusedOwnerTermsP1
                || neighbor_encoding_mode_ != NeighborEncodingMode::CompactU16P2
                || !compact_neighbor_ids_)) {
            throw std::invalid_argument(
                "dynamic-cell-local-p3 requires stable P1 plus eligible compact P2");
        }
        if (neighbor_reuse_mode_ == NeighborReuseMode::CertifiedVerletP4
            && (storage_mode_ != StorageMode::StableSampleV0
                || dynamic_storage_mode_ != DynamicStorageMode::Retained
                || pair_traversal_mode_ != PairTraversalMode::FusedOwnerTermsP1
                || neighbor_encoding_mode_ != NeighborEncodingMode::CompactU16P2
                || !compact_neighbor_ids_)) {
            throw std::invalid_argument(
                "verlet-skin-p4 requires stable P1 plus eligible compact P2");
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
        cudaFree(neighbor_anchor_);
        cudaFree(max_displacement_bits_);
    }

    /// Copies the current published sample positions (the advected reference
    /// after `execute(_, true)`) without the diagnostic capture path.
    void download_positions(std::vector<float3>& positions) const {
        positions.resize(static_cast<std::size_t>(count_));
        const float3* source = uses_cell_local_storage() ? sorted_reference_ : reference_;
        check_cuda(cudaMemcpy(
                       positions.data(), source,
                       static_cast<std::size_t>(count_) * sizeof(float3),
                       cudaMemcpyDeviceToHost),
            "download published positions");
    }

    void reset_seed() {
        upload_fixture();
        if (neighbor_reuse_mode_ == NeighborReuseMode::CertifiedVerletP4) {
            neighbor_cache_valid_ = false;
        }
    }

    CapturedRun execute(bool capture, bool advance = false) {
        if (advance && !fixture_.advected) {
            throw std::invalid_argument("state advance requires an advected fixture");
        }
        const float3* solver_reference = uses_cell_local_storage()
            ? sorted_reference_ : reference_;
        const float3* solver_initial_velocity = uses_cell_local_storage()
            ? sorted_initial_velocity_ : initial_velocity_;
        const std::uint8_t* solver_fixed = uses_cell_local_storage()
            ? sorted_fixed_ : fixed_;
        std::vector<EventInterval> intervals;
        intervals.reserve(static_cast<std::size_t>(5 + 7 * fixture_.iterations));
        cudaEvent_t total_begin{};
        cudaEvent_t total_end{};
        check_cuda(cudaEventCreate(&total_begin), "cudaEventCreate(total begin)");
        check_cuda(cudaEventCreate(&total_end), "cudaEventCreate(total end)");
        check_cuda(cudaEventRecord(total_begin), "cudaEventRecord(total begin)");

        if (dynamic_storage_mode_ == DynamicStorageMode::DynamicCellLocalP3) {
            timed(intervals, EventInterval::Stage::StorageRemap, [&] {
                enqueue_dynamic_storage_remap();
            });
        }

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
            if (neighbor_reuse_mode_ == NeighborReuseMode::CertifiedVerletP4) {
                enqueue_certified_neighbor_construction_p4();
            } else {
                enqueue_neighbor_construction();
            }
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
                enqueue_density(solver_reference);
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
        if (fixture_.analytic_box_contact) {
            timed(intervals, EventInterval::Stage::Contact, [&] {
                const float radius = static_cast<float>(fixture_.particle_radius);
                clamp_analytic_box_contact<<<blocks_for(count_), THREADS>>>(
                    current_, solver_fixed, count_,
                    make_float3(
                        static_cast<float>(fixture_.contact_minimum.x) + radius,
                        static_cast<float>(fixture_.contact_minimum.y) + radius,
                        static_cast<float>(fixture_.contact_minimum.z) + radius),
                    make_float3(
                        static_cast<float>(fixture_.contact_maximum.x) - radius,
                        static_cast<float>(fixture_.contact_maximum.y) - radius,
                        static_cast<float>(fixture_.contact_maximum.z) - radius));
                if (fixture_.spill.enabled && fixture_.spill.under_floor) {
                    const Fixture::Spill& s = fixture_.spill;
                    const SpillPipeDevice geometry{
                        static_cast<float>(s.shelf_top),  static_cast<float>(s.wall_x0),
                        static_cast<float>(s.wall_x1),    static_cast<float>(s.shaft_x0),
                        static_cast<float>(s.shaft_x1),   static_cast<float>(s.opening_y0),
                        static_cast<float>(s.opening_y1), static_cast<float>(s.opening_z0),
                        static_cast<float>(s.opening_z1), static_cast<float>(s.pipe_x1),
                        static_cast<float>(s.pipe_wall)};
                    clamp_spill_pipe_contact<<<blocks_for(count_), THREADS>>>(
                        current_, solver_fixed, count_, radius, geometry);
                } else if (fixture_.spill.enabled) {
                    clamp_spill_contact<<<blocks_for(count_), THREADS>>>(
                        current_, solver_fixed, count_, radius,
                        static_cast<float>(fixture_.spill.shelf_top),
                        static_cast<float>(fixture_.spill.wall_x0),
                        static_cast<float>(fixture_.spill.wall_x1),
                        static_cast<float>(fixture_.spill.opening_y0),
                        static_cast<float>(fixture_.spill.opening_y1),
                        static_cast<float>(fixture_.spill.opening_z0),
                        static_cast<float>(fixture_.spill.opening_z1),
                        fixture_.spill.flush ? 0.0F : radius);
                }
            });
        }
        timed(intervals, EventInterval::Stage::Density, [&] {
            enqueue_density(solver_reference);
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
                if (dynamic_storage_mode_ == DynamicStorageMode::DynamicCellLocalP3) {
                    scatter_dynamic_state<<<blocks_for(count_), THREADS>>>(
                        current_, final_velocity_, storage_to_sample_, reference_,
                        initial_velocity_, count_);
                } else {
                    check_cuda(cudaMemcpyAsync(
                                   reference_, current_,
                                   static_cast<std::size_t>(count_) * sizeof(float3),
                                   cudaMemcpyDeviceToDevice),
                        "publish dynamic reference positions");
                    check_cuda(cudaMemcpyAsync(
                                   initial_velocity_, final_velocity_,
                                   static_cast<std::size_t>(count_) * sizeof(float3),
                                   cudaMemcpyDeviceToDevice),
                        "publish dynamic velocity");
                }
            });
        }
        check_cuda(cudaGetLastError(), "enqueue CUDA baseline");
        check_cuda(cudaEventRecord(total_end), "cudaEventRecord(total end)");
        check_cuda(cudaEventSynchronize(total_end), "cudaEventSynchronize(total end)");

        if (capture && uses_cell_local_storage()) {
            capture_storage_map();
        }

        CapturedRun result;
        result.device_memory_bytes = device_memory_bytes_;
        result.neighbor_id_bytes = compact_neighbor_ids_ ? sizeof(std::uint16_t) : sizeof(int);
        result.layout_memory_bytes = layout_memory_bytes_;
        result.layout_setup_ms = layout_setup_ms_;
        result.storage_memory_bytes = storage_memory_bytes_;
        result.storage_setup_ms = storage_setup_ms_;
        result.storage_to_sample_sha256 = storage_to_sample_sha256_;
        result.sample_to_storage_sha256 = sample_to_storage_sha256_;
        result.neighbor_rebuilt = current_neighbor_rebuilt_;
        result.neighbor_reused = neighbor_reuse_mode_ == NeighborReuseMode::CertifiedVerletP4
            && !current_neighbor_rebuilt_;
        result.maximum_anchor_displacement = current_max_anchor_displacement_;
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
        if (uses_cell_local_storage()) {
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
    bool uses_cell_local_storage() const {
        return storage_mode_ == StorageMode::CellSortedO4
            || dynamic_storage_mode_ == DynamicStorageMode::DynamicCellLocalP3;
    }

    void enqueue_density(const float3* solver_reference) {
        if (neighbor_reuse_mode_ == NeighborReuseMode::CertifiedVerletP4) {
            compute_density_filtered_p4<<<blocks_for(count_), THREADS>>>(
                current_, solver_reference, neighbor_offsets_, compact_neighbors_, density_, count_,
                static_cast<float>(fixture_.mass), static_cast<float>(fixture_.horizon),
                kernel_scale_);
        } else if (compact_neighbor_ids_) {
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
    }

    template <bool Bulk, bool Shear, bool Surface>
    void enqueue_fused_owner_terms(const float3* solver_reference) {
        const std::uint8_t* solver_fixed = uses_cell_local_storage() ? sorted_fixed_ : fixed_;
        const bool density_only_support = fixture_.boundary_density_only;
        if (neighbor_reuse_mode_ == NeighborReuseMode::CertifiedVerletP4) {
            accumulate_fused_owner_terms_p1<std::uint16_t, Bulk, Shear, Surface, true>
                <<<blocks_for(count_), THREADS>>>(
                    solver_reference, current_, density_, neighbor_offsets_, compact_neighbors_,
                    solver_fixed, density_only_support, source_, matrix_, count_, static_cast<float>(fixture_.rest_density),
                    static_cast<float>(fixture_.kappa), static_cast<float>(fixture_.lambda),
                    static_cast<float>(fixture_.mu), static_cast<float>(fixture_.gamma),
                    static_cast<float>(fixture_.spacing), static_cast<float>(fixture_.horizon),
                    static_cast<float>(fixture_.time_step), kernel_scale_);
        } else if (compact_neighbor_ids_) {
            accumulate_fused_owner_terms_p1<std::uint16_t, Bulk, Shear, Surface>
                <<<blocks_for(count_), THREADS>>>(
                    solver_reference, current_, density_, neighbor_offsets_, compact_neighbors_,
                    solver_fixed, density_only_support, source_, matrix_, count_, static_cast<float>(fixture_.rest_density),
                    static_cast<float>(fixture_.kappa), static_cast<float>(fixture_.lambda),
                    static_cast<float>(fixture_.mu), static_cast<float>(fixture_.gamma),
                    static_cast<float>(fixture_.spacing), static_cast<float>(fixture_.horizon),
                    static_cast<float>(fixture_.time_step), kernel_scale_);
        } else {
            accumulate_fused_owner_terms_p1<int, Bulk, Shear, Surface>
                <<<blocks_for(count_), THREADS>>>(
                    solver_reference, current_, density_, neighbor_offsets_, neighbors_,
                    solver_fixed, density_only_support, source_, matrix_, count_, static_cast<float>(fixture_.rest_density),
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
        if (uses_cell_local_storage()) {
            const std::size_t before_storage = device_memory_bytes_;
            allocate(sorted_reference_, count);
            allocate(sorted_initial_velocity_, count);
            allocate(sorted_fixed_, count);
            allocate(storage_to_sample_, count);
            allocate(sample_to_storage_, count);
            allocate(storage_error_flag_, 1U);
            storage_memory_bytes_ = device_memory_bytes_ - before_storage;
        }
        if (neighbor_reuse_mode_ == NeighborReuseMode::CertifiedVerletP4) {
            allocate(neighbor_anchor_, count);
            allocate(max_displacement_bits_, 1U);
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
        capture_storage_map();
    }

    void capture_storage_map() {
        storage_to_sample_host_.resize(static_cast<std::size_t>(count_));
        sample_to_storage_host_.resize(static_cast<std::size_t>(count_));
        check_cuda(cudaMemcpy(
                       storage_to_sample_host_.data(), storage_to_sample_,
                       storage_to_sample_host_.size() * sizeof(int), cudaMemcpyDeviceToHost),
            "copy storage-to-sample map");
        check_cuda(cudaMemcpy(
                       sample_to_storage_host_.data(), sample_to_storage_,
                       sample_to_storage_host_.size() * sizeof(int), cudaMemcpyDeviceToHost),
            "copy sample-to-storage map");
        for (int storage = 0; storage < count_; ++storage) {
            const int sample = storage_to_sample_host_[static_cast<std::size_t>(storage)];
            if (sample < 0 || sample >= count_
                || sample_to_storage_host_[static_cast<std::size_t>(sample)] != storage) {
                throw std::runtime_error("storage map is not a bounded bijection");
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

    void enqueue_dynamic_storage_remap() {
        check_cuda(cudaMemsetAsync(neighbor_error_flag_, 0, sizeof(int)),
            "reset P3 grid flag");
        check_cuda(cudaMemsetAsync(storage_error_flag_, 0, sizeof(int)),
            "reset P3 storage flag");
        enqueue_grid_sort();
        initialize_storage_map<<<blocks_for(count_), THREADS>>>(
            keys_sorted_, indices_sorted_, storage_to_sample_, sample_to_storage_,
            storage_error_flag_, count_);
        gather_immutable_storage<<<blocks_for(count_), THREADS>>>(
            reference_, initial_velocity_, fixed_, storage_to_sample_, sorted_reference_,
            sorted_initial_velocity_, sorted_fixed_, count_);
    }

    void enqueue_certified_neighbor_construction_p4() {
        constexpr double SKIN_FRACTION = 0.04;
        constexpr double CERTIFICATE_SAFETY = 1.0 - 2.0e-12;
        check_cuda(cudaMemsetAsync(neighbor_error_flag_, 0, sizeof(int)),
            "reset P4 neighbor flag");
        current_neighbor_rebuilt_ = !neighbor_cache_valid_;
        current_max_anchor_displacement_ = 0.0;
        if (neighbor_cache_valid_) {
            check_cuda(cudaMemsetAsync(
                           max_displacement_bits_, 0, sizeof(unsigned long long)),
                "reset P4 displacement maximum");
            measure_max_displacement_squared<<<blocks_for(count_), THREADS>>>(
                reference_, neighbor_anchor_, max_displacement_bits_, count_);
            double maximum_squared = 0.0;
            check_cuda(cudaMemcpy(
                           &maximum_squared, max_displacement_bits_, sizeof(double),
                           cudaMemcpyDeviceToHost),
                "copy P4 displacement maximum");
            current_max_anchor_displacement_ = std::sqrt(maximum_squared);
            const double skin = SKIN_FRACTION * fixture_.horizon;
            current_neighbor_rebuilt_ = !std::isfinite(maximum_squared)
                || 4.0 * maximum_squared > skin * skin * CERTIFICATE_SAFETY;
        }
        if (!current_neighbor_rebuilt_) {
            return;
        }

        check_cuda(cudaMemcpyAsync(
                       neighbor_anchor_, reference_,
                       static_cast<std::size_t>(count_) * sizeof(float3),
                       cudaMemcpyDeviceToDevice),
            "capture P4 neighbor anchor");
        enqueue_grid_sort();
        constexpr int CELL_RADIUS = 2;
        const float superset_horizon = static_cast<float>(
            fixture_.horizon * (1.0 + SKIN_FRACTION));
        visit_grid_neighbors_superset_p4<false, std::uint16_t>
            <<<blocks_for(count_), THREADS>>>(
                neighbor_anchor_, indices_sorted_, cell_start_, cell_end_, neighbor_counts_,
                nullptr, nullptr, count_, grid_.origin, static_cast<float>(fixture_.horizon),
                superset_horizon, grid_.dimensions, CELL_RADIUS, neighbor_error_flag_);
        check_cuda(cub::DeviceScan::ExclusiveSum(
                       scan_scratch_, scan_scratch_bytes_, neighbor_counts_, neighbor_offsets_, count_),
            "CUB P4 exclusive neighbor scan");
        finish_offsets<<<1, 1>>>(neighbor_counts_, neighbor_offsets_, count_);
        clamp_neighbor_offsets<<<blocks_for(count_ + 1), THREADS>>>(
            neighbor_offsets_, count_, static_cast<int>(pair_capacity_), neighbor_error_flag_);
        visit_grid_neighbors_superset_p4<true, std::uint16_t>
            <<<blocks_for(count_), THREADS>>>(
                neighbor_anchor_, indices_sorted_, cell_start_, cell_end_, nullptr,
                neighbor_offsets_, compact_neighbors_, count_, grid_.origin,
                static_cast<float>(fixture_.horizon), superset_horizon, grid_.dimensions,
                CELL_RADIUS, neighbor_error_flag_);
        neighbor_cache_valid_ = true;
    }

    void enqueue_neighbor_construction() {
        if (dynamic_storage_mode_ != DynamicStorageMode::DynamicCellLocalP3) {
            check_cuda(cudaMemsetAsync(neighbor_error_flag_, 0, sizeof(int)),
                "reset neighbor build flag");
            enqueue_grid_sort();
        }
        if (uses_cell_local_storage()) {
            if (storage_mode_ == StorageMode::CellSortedO4) {
                check_cuda(cudaMemsetAsync(storage_error_flag_, 0, sizeof(int)),
                    "reset O4 storage flag");
                validate_storage_map<<<blocks_for(count_), THREADS>>>(
                    keys_sorted_, indices_sorted_, storage_to_sample_, sample_to_storage_,
                    storage_error_flag_, count_);
            }
            if (compact_neighbor_ids_) {
                visit_grid_neighbors_cell_sorted<false, std::uint16_t>
                    <<<blocks_for(count_), THREADS>>>(
                        reference_, indices_sorted_, storage_to_sample_, sample_to_storage_,
                        cell_start_, cell_end_, neighbor_counts_, nullptr, nullptr, count_,
                        grid_.origin, static_cast<float>(fixture_.horizon), grid_.dimensions,
                        neighbor_error_flag_);
            } else {
                visit_grid_neighbors_cell_sorted<false, int><<<blocks_for(count_), THREADS>>>(
                    reference_, indices_sorted_, storage_to_sample_, sample_to_storage_, cell_start_,
                    cell_end_, neighbor_counts_, nullptr, nullptr, count_, grid_.origin,
                    static_cast<float>(fixture_.horizon), grid_.dimensions,
                    neighbor_error_flag_);
            }
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
        if (uses_cell_local_storage()) {
            if (compact_neighbor_ids_) {
                visit_grid_neighbors_cell_sorted<true, std::uint16_t>
                    <<<blocks_for(count_), THREADS>>>(
                        reference_, indices_sorted_, storage_to_sample_, sample_to_storage_,
                        cell_start_, cell_end_, nullptr, neighbor_offsets_, compact_neighbors_,
                        count_, grid_.origin, static_cast<float>(fixture_.horizon),
                        grid_.dimensions, neighbor_error_flag_);
            } else {
                visit_grid_neighbors_cell_sorted<true, int><<<blocks_for(count_), THREADS>>>(
                    reference_, indices_sorted_, storage_to_sample_, sample_to_storage_, cell_start_,
                    cell_end_, nullptr, neighbor_offsets_, neighbors_, count_, grid_.origin,
                    static_cast<float>(fixture_.horizon), grid_.dimensions,
                    neighbor_error_flag_);
            }
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
        case EventInterval::Stage::StorageRemap:
            timing.storage_remap += milliseconds;
            break;
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
        case EventInterval::Stage::Contact:
            timing.contact += milliseconds;
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
        result.candidate_directed_pairs = static_cast<std::size_t>(directed_pairs);
        if (neighbor_reuse_mode_ == NeighborReuseMode::CertifiedVerletP4) {
            compute_energy_components<std::uint16_t, true><<<blocks_for(count_), THREADS>>>(
                solver_reference, predicted_, current_, density_, neighbor_offsets_,
                compact_neighbors_, energy_, count_, static_cast<float>(fixture_.rest_density),
                static_cast<float>(fixture_.spacing), static_cast<float>(fixture_.mass),
                static_cast<float>(fixture_.horizon), static_cast<float>(fixture_.time_step),
                static_cast<float>(fixture_.kappa), static_cast<float>(fixture_.lambda),
                static_cast<float>(fixture_.mu), static_cast<float>(fixture_.gamma), kernel_scale_,
                fixture_.terms.incompressibility, fixture_.terms.bulk_viscosity,
                fixture_.terms.shear_viscosity, fixture_.terms.surface_tension);
        } else if (compact_neighbor_ids_) {
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
        if (neighbor_reuse_mode_ == NeighborReuseMode::CertifiedVerletP4) {
            std::vector<float3> active_reference(static_cast<std::size_t>(count_));
            check_cuda(cudaMemcpy(
                           active_reference.data(), solver_reference,
                           active_reference.size() * sizeof(float3), cudaMemcpyDeviceToHost),
                "copy P4 active reference");
            std::vector<int> active_offsets(static_cast<std::size_t>(count_) + 1U, 0);
            std::vector<int> active_neighbors;
            active_neighbors.reserve(physical_neighbors.size());
            const float horizon = static_cast<float>(fixture_.horizon);
            const float active_squared = horizon * horizon * (1.0F + 2.0e-6F);
            for (int owner = 0; owner < count_; ++owner) {
                for (int slot = physical_offsets[static_cast<std::size_t>(owner)];
                     slot < physical_offsets[static_cast<std::size_t>(owner + 1)]; ++slot) {
                    const int neighbor = physical_neighbors[static_cast<std::size_t>(slot)];
                    const float3 own = active_reference[static_cast<std::size_t>(owner)];
                    const float3 other = active_reference[static_cast<std::size_t>(neighbor)];
                    const float dx = own.x - other.x;
                    const float dy = own.y - other.y;
                    const float dz = own.z - other.z;
                    if (dx * dx + dy * dy + dz * dz <= active_squared) {
                        active_neighbors.push_back(neighbor);
                    }
                }
                active_offsets[static_cast<std::size_t>(owner + 1)] =
                    static_cast<int>(active_neighbors.size());
            }
            physical_offsets = std::move(active_offsets);
            physical_neighbors = std::move(active_neighbors);
            directed_pairs = static_cast<int>(physical_neighbors.size());
        }
        if (uses_cell_local_storage()) {
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
            const int physical = uses_cell_local_storage()
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
            const int physical = uses_cell_local_storage()
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
    DynamicStorageMode dynamic_storage_mode_ = DynamicStorageMode::Retained;
    NeighborReuseMode neighbor_reuse_mode_ = NeighborReuseMode::RebuildEverySolve;
    bool compact_neighbor_ids_ = false;
    bool neighbor_cache_valid_ = false;
    bool current_neighbor_rebuilt_ = false;
    double current_max_anchor_displacement_ = 0.0;
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
    float3* neighbor_anchor_ = nullptr;
    unsigned long long* max_displacement_bits_ = nullptr;
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
    output << "{\"storage_remap_ms\":" << timing.storage_remap
           << ",\"prediction_ms\":" << timing.prediction
           << ",\"neighbor_construction_ms\":" << timing.neighbor_construction
           << ",\"density_ms\":" << timing.density
           << ",\"buffer_reset_ms\":" << timing.buffer_reset
           << ",\"incompressibility_ms\":" << timing.incompressibility
           << ",\"viscosity_ms\":" << timing.viscosity
           << ",\"surface_tension_ms\":" << timing.surface_tension
           << ",\"fused_owner_terms_ms\":" << timing.fused_owner_terms
           << ",\"local_update_ms\":" << timing.local_update
           << ",\"analytic_contact_ms\":" << timing.contact
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
    output << "{\"storage_remap\":";
    append_statistics(output, collect_statistics(timings, &StageTiming::storage_remap));
    output << ",\"prediction\":";
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
    output << ",\"analytic_contact\":";
    append_statistics(output, collect_statistics(timings, &StageTiming::contact));
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

bool p3_memory_correspondence(
    const Fixture& fixture,
    const CapturedRun& retained,
    const CapturedRun& candidate) {
    const std::size_t expected_storage = 33U * fixture.particles.size() + sizeof(int);
    return candidate.neighbor_id_bytes == sizeof(std::uint16_t)
        && candidate.storage_memory_bytes == expected_storage
        && candidate.device_memory_bytes == retained.device_memory_bytes + expected_storage;
}

P1Preflight p3_tiny_preflight() {
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
            PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2);
        CudaBaseline candidate(
            fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
            PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2,
            DynamicStorageMode::DynamicCellLocalP3);
        const CapturedRun retained_run = retained.execute(true);
        const CapturedRun candidate_run = candidate.execute(true);
        const bool passed = exact_p1_correspondence(
                fixture, retained_run, candidate_run, tolerances)
            && compare_results(fixture, cpu, candidate_run.state, tolerances).passed
            && exact_fixture_neighbors(fixture, candidate_run)
            && p3_memory_correspondence(fixture, retained_run, candidate_run)
            && !candidate_run.storage_to_sample_sha256.empty()
            && !candidate_run.sample_to_storage_sha256.empty();
        if (!passed) {
            result.passed = false;
            if (result.first_failure.empty()) {
                result.first_failure = fixture.name;
            }
        }
    }
    return result;
}

bool p3_stiff_i2_preflight() {
    const Profile& profile = find_profile("nuv-surface-stiff-16k-i2.v1");
    const Fixture fixture = performance_fixture(profile, 2);
    CudaBaseline retained(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2);
    CudaBaseline candidate(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2,
        DynamicStorageMode::DynamicCellLocalP3);
    const CapturedRun retained_run = retained.execute(true);
    const CapturedRun candidate_run = candidate.execute(true);
    return exact_p1_correspondence(
               fixture, retained_run, candidate_run, profile.tolerances)
        && p3_memory_correspondence(fixture, retained_run, candidate_run);
}

bool p4_memory_correspondence(
    const Fixture& fixture,
    const CapturedRun& retained,
    const CapturedRun& candidate) {
    const std::size_t expected_cache = 12U * fixture.particles.size()
        + sizeof(unsigned long long);
    return candidate.neighbor_id_bytes == sizeof(std::uint16_t)
        && candidate.device_memory_bytes == retained.device_memory_bytes + expected_cache;
}

P1Preflight p4_tiny_preflight() {
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
            PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2);
        CudaBaseline candidate(
            fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
            PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2,
            DynamicStorageMode::Retained, NeighborReuseMode::CertifiedVerletP4);
        const CapturedRun retained_cold = retained.execute(true);
        const CapturedRun candidate_cold = candidate.execute(true);
        const CapturedRun retained_reused = retained.execute(true);
        const CapturedRun candidate_reused = candidate.execute(true);
        const bool passed = exact_p1_correspondence(
                fixture, retained_cold, candidate_cold, tolerances)
            && exact_p1_correspondence(fixture, retained_reused, candidate_reused, tolerances)
            && compare_results(fixture, cpu, candidate_reused.state, tolerances).passed
            && exact_fixture_neighbors(fixture, candidate_reused)
            && p4_memory_correspondence(fixture, retained_cold, candidate_cold)
            && candidate_cold.neighbor_rebuilt && candidate_reused.neighbor_reused
            && candidate_reused.maximum_anchor_displacement == 0.0
            && candidate_cold.candidate_directed_pairs >= candidate_cold.state.directed_pairs;
        if (!passed) {
            result.passed = false;
            if (result.first_failure.empty()) {
                result.first_failure = fixture.name;
            }
        }
    }
    return result;
}

bool p4_stiff_i2_preflight() {
    const Profile& profile = find_profile("nuv-surface-stiff-16k-i2.v1");
    const Fixture fixture = performance_fixture(profile, 2);
    CudaBaseline retained(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2);
    CudaBaseline candidate(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2,
        DynamicStorageMode::Retained, NeighborReuseMode::CertifiedVerletP4);
    const CapturedRun retained_cold = retained.execute(true);
    const CapturedRun candidate_cold = candidate.execute(true);
    const CapturedRun retained_reused = retained.execute(true);
    const CapturedRun candidate_reused = candidate.execute(true);
    return exact_p1_correspondence(
               fixture, retained_cold, candidate_cold, profile.tolerances)
        && exact_p1_correspondence(
            fixture, retained_reused, candidate_reused, profile.tolerances)
        && p4_memory_correspondence(fixture, retained_cold, candidate_cold)
        && candidate_cold.neighbor_rebuilt && candidate_reused.neighbor_reused;
}

} // namespace

CommandReport run_cuda_p1_check(
    const Profile& profile,
    int iterations) {
    if (profile.record_version < 1 || profile.record_version > 4
        || iterations < 1 || iterations > 100) {
        throw std::invalid_argument(
            "P1 check requires a v1/v2/v3/v4 profile and 1..=100 iterations");
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
    if ((profile.record_version != 1 && profile.record_version != 2
            && profile.record_version != 3 && profile.record_version != 4
            && profile.record_version != 5 && profile.record_version != 6)
        || iterations < 1 || iterations > 100) {
        throw std::invalid_argument(
            "P2 check requires a v1/v2/v3/v4/v5/v6 profile and 1..=100 iterations");
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

CommandReport run_cuda_p3_check(
    const Profile& profile,
    int iterations) {
    if (profile.record_version != 1 || iterations < 1 || iterations > 100
        || profile.id == "nuv-water-100k-report.v1") {
        throw std::invalid_argument(
            "P3 check requires an eligible v1 profile and 1..=100 iterations");
    }
    const CommandReport p2_check = run_cuda_p2_check(profile, iterations);
    const P1Preflight tiny = p3_tiny_preflight();
    const bool stiff_i2 = p3_stiff_i2_preflight();
    const Fixture fixture = performance_fixture(profile, iterations);
    CudaBaseline retained(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2);
    CudaBaseline candidate(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2,
        DynamicStorageMode::DynamicCellLocalP3);
    const CapturedRun retained_run = retained.execute(true);
    const CapturedRun candidate_run = candidate.execute(true);
    const bool exact = exact_p1_correspondence(
        fixture, retained_run, candidate_run, profile.tolerances);
    const bool memory_exact = p3_memory_correspondence(
        fixture, retained_run, candidate_run);
    const bool maps_valid = candidate_run.storage_map_valid
        && !candidate_run.storage_to_sample_sha256.empty()
        && !candidate_run.sample_to_storage_sha256.empty();
    const bool passed = p2_check.passed && tiny.passed && stiff_i2
        && exact && memory_exact && maps_valid;

    std::ostringstream output;
    output << std::setprecision(17);
    output << "{\"schema\":\"nextengine.nonlocal.p3_check.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"candidate_class\":\"EXACT_WORK\""
           << ",\"candidate_identity\":\"dynamic-cell-local-p3\",\"profile_id\":\""
           << profile.id << "\",\"iterations\":" << iterations
           << ",\"profile_sha256\":\"" << sha256_hex(canonical_profile_json(profile))
           << "\",\"input_sha256\":\"" << fixture_input_hash(fixture)
           << "\",\"binary_sha256\":\"" << executable_hash()
           << "\",\"correctness\":{\"p2_check_passed\":"
           << (p2_check.passed ? "true" : "false")
           << ",\"tiny_cpu_oracle_passed\":" << (tiny.passed ? "true" : "false")
           << ",\"tiny_first_failure\":\"" << tiny.first_failure
           << "\",\"stiff_surface_i2_passed\":" << (stiff_i2 ? "true" : "false")
           << ",\"target_exact\":" << (exact ? "true" : "false")
           << ",\"memory_exact\":" << (memory_exact ? "true" : "false")
           << ",\"maps_valid\":" << (maps_valid ? "true" : "false") << '}'
           << ",\"retained\":{\"ordered_output_sha256\":\""
           << ordered_output_digest(retained_run.state) << "\",\"logical_csr_sha256\":\""
           << csr_digest(retained_run) << "\",\"device_memory_bytes\":"
           << retained_run.device_memory_bytes << ",\"timing\":";
    append_timing(output, retained_run.timing);
    output << "},\"candidate\":{\"ordered_output_sha256\":\""
           << ordered_output_digest(candidate_run.state)
           << "\",\"logical_csr_sha256\":\"" << csr_digest(candidate_run)
           << "\",\"device_memory_bytes\":" << candidate_run.device_memory_bytes
           << ",\"storage_memory_bytes\":" << candidate_run.storage_memory_bytes
           << ",\"storage_to_sample_sha256\":\""
           << candidate_run.storage_to_sample_sha256
           << "\",\"sample_to_storage_sha256\":\""
           << candidate_run.sample_to_storage_sha256 << "\",\"timing\":";
    append_timing(output, candidate_run.timing);
    output << "},\"device\":" << device_json() << '}';
    return {passed, output.str()};
}

CommandReport run_cuda_p3_tournament(
    const Profile& profile,
    int warmup,
    int runs) {
    if (profile.record_version != 1 || warmup != 32 || runs != 96
        || profile.id == "nuv-water-100k-report.v1") {
        throw std::invalid_argument(
            "P3 tournament requires eligible v1 --warmup 32 --runs 96");
    }
    if (profile.id == "nuv-surface-stiff-16k-i2.v1") {
        throw std::invalid_argument("P3 tournament requires an adjacent decision profile");
    }
    const CommandReport check = run_cuda_p3_check(profile, profile.fixed_iterations);
    const Fixture fixture = performance_fixture(profile, profile.fixed_iterations);
    CudaBaseline retained(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2);
    CudaBaseline candidate(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2,
        DynamicStorageMode::DynamicCellLocalP3);

    bool trace_exact = check.passed;
    bool trace_memory_exact = true;
    bool trace_maps_valid = true;
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
        const bool step_memory = p3_memory_correspondence(
            fixture, retained_run, candidate_run);
        const bool step_maps = candidate_run.storage_map_valid
            && !candidate_run.storage_to_sample_sha256.empty()
            && !candidate_run.sample_to_storage_sha256.empty();
        if ((!step_exact || !step_memory || !step_maps) && first_trace_mismatch < 0) {
            first_trace_mismatch = step;
        }
        trace_exact = trace_exact && step_exact;
        trace_memory_exact = trace_memory_exact && step_memory;
        trace_maps_valid = trace_maps_valid && step_maps;
        const std::string logical_csr_sha256 = csr_digest(retained_run);
        const std::string membership_sha256 =
            canonical_lattice_csr_digest(fixture, retained_run);
        const std::string output_sha256 = ordered_output_digest(retained_run.state);
        const std::string state_sha256 = handoff_digest(retained_run.state);
        trace_material << step << '|' << prior_state_digest << '|' << logical_csr_sha256
                       << '|' << membership_sha256 << '|' << output_sha256 << '|'
                       << state_sha256 << '|' << candidate_run.storage_to_sample_sha256 << '|'
                       << candidate_run.sample_to_storage_sha256 << '|';
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
            if (run.local_solve_failed || !run.neighbor_build_valid
                || !run.storage_map_valid) {
                throw std::runtime_error("P3 identity failed during tournament");
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
    const bool improvement_gate = total_speedup >= 1.05;
    const bool regression_gate = candidate_total.p95 <= retained_total.p95 * 1.02;
    const bool permuted_profile = profile.id == "nuv-water-50k-permuted.v1";
    const bool coherent_control = profile.id == "nuv-water-50k-coherent.v1";
    const bool profile_gate = permuted_profile ? improvement_gate
        : (coherent_control ? true : regression_gate);
    const bool select_candidate_for_profile = improvement_gate;
    const std::size_t added_memory = candidate_seed.device_memory_bytes
        - retained_seed.device_memory_bytes;

    std::ostringstream output;
    output << std::setprecision(17);
    output << "{\"schema\":\"nextengine.nonlocal.p3_tournament.v1\",\"status\":\""
           << (trace_exact && trace_memory_exact && trace_maps_valid ? "PASS" : "FAIL")
           << "\",\"candidate_class\":\"EXACT_WORK\""
           << ",\"candidate_identity\":\"dynamic-cell-local-p3\",\"profile_id\":\""
           << profile.id << "\",\"profile_sha256\":\""
           << sha256_hex(canonical_profile_json(profile)) << "\",\"input_sha256\":\""
           << fixture_input_hash(fixture) << "\",\"trace_sha256\":\"" << trace_sha256
           << "\",\"binary_sha256\":\"" << executable_hash()
           << "\",\"command\":\"nonlocal-feasibility --p3-tournament " << profile.id
           << " --warmup 32 --runs 96\",\"conditioning_runs\":" << CONDITIONING_RUNS
           << ",\"warmup_rounds\":" << warmup << ",\"measured_rounds\":" << runs
           << ",\"rotation\":[[\"retained-p2\",\"candidate-p3\"],"
              "[\"candidate-p3\",\"retained-p2\"]]"
           << ",\"correctness\":{\"check_passed\":" << (check.passed ? "true" : "false")
           << ",\"trace_exact\":" << (trace_exact ? "true" : "false")
           << ",\"trace_memory_exact\":" << (trace_memory_exact ? "true" : "false")
           << ",\"trace_maps_valid\":" << (trace_maps_valid ? "true" : "false")
           << ",\"first_trace_mismatch\":" << first_trace_mismatch
           << ",\"retained_seed_output_sha256\":\""
           << ordered_output_digest(retained_seed.state)
           << "\",\"candidate_seed_output_sha256\":\""
           << ordered_output_digest(candidate_seed.state)
           << "\",\"logical_csr_sha256\":\"" << csr_digest(retained_seed)
           << "\",\"storage_to_sample_sha256\":\""
           << candidate_seed.storage_to_sample_sha256
           << "\",\"sample_to_storage_sha256\":\""
           << candidate_seed.sample_to_storage_sha256 << "\"}"
           << ",\"retention\":{\"coherent_negative_control\":"
           << (coherent_control ? "true" : "false")
           << ",\"improvement_gate\":" << (improvement_gate ? "true" : "false")
           << ",\"regression_gate\":" << (regression_gate ? "true" : "false")
           << ",\"profile_gate\":" << (profile_gate ? "true" : "false")
           << ",\"select_candidate_for_profile\":"
           << (select_candidate_for_profile ? "true" : "false")
           << ",\"total_p95_speedup\":" << total_speedup
           << ",\"pair_stage_p95_speedup\":" << pair_speedup
           << ",\"retained_total_p95_ms\":" << retained_total.p95
           << ",\"candidate_total_p95_ms\":" << candidate_total.p95
           << ",\"retained_pair_stage_p95_ms\":" << retained_pair.p95
           << ",\"candidate_pair_stage_p95_ms\":" << candidate_pair.p95 << '}'
           << ",\"memory\":{\"retained_bytes\":" << retained_seed.device_memory_bytes
           << ",\"candidate_bytes\":" << candidate_seed.device_memory_bytes
           << ",\"added_storage_bytes\":" << added_memory << '}'
           << ",\"identities\":{\"retained_p2\":{\"statistics\":";
    append_stage_statistics(output, timings[0]);
    output << ",\"raw_total_ms\":";
    append_raw_totals(output, timings[0]);
    output << "},\"candidate_p3\":{\"statistics\":";
    append_stage_statistics(output, timings[1]);
    output << ",\"raw_total_ms\":";
    append_raw_totals(output, timings[1]);
    output << "}},\"device\":" << device_json() << '}';
    return {trace_exact && trace_memory_exact && trace_maps_valid, output.str()};
}

CommandReport run_cuda_p4_check(
    const Profile& profile,
    int iterations) {
    if (profile.record_version != 1 || iterations < 1 || iterations > 100
        || profile.id == "nuv-water-100k-report.v1") {
        throw std::invalid_argument(
            "P4 check requires an eligible v1 profile and 1..=100 iterations");
    }
    const CommandReport p2_check = run_cuda_p2_check(profile, iterations);
    const P1Preflight tiny = p4_tiny_preflight();
    const bool stiff_i2 = p4_stiff_i2_preflight();
    const Fixture fixture = performance_fixture(profile, iterations);
    CudaBaseline retained(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2);
    CudaBaseline candidate(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2,
        DynamicStorageMode::Retained, NeighborReuseMode::CertifiedVerletP4);
    retained.reset_seed();
    candidate.reset_seed();
    bool exact = true;
    bool memory_exact = true;
    int rebuilt = 0;
    int reused = 0;
    int first_mismatch = -1;
    double maximum_displacement = 0.0;
    std::ostringstream step_receipts;
    step_receipts << '[';
    CapturedRun retained_first;
    CapturedRun candidate_first;
    for (int step = 0; step < 3; ++step) {
        CapturedRun retained_run = retained.execute(true, profile.advected);
        CapturedRun candidate_run = candidate.execute(true, profile.advected);
        const bool step_exact = exact_p1_correspondence(
            fixture, retained_run, candidate_run, profile.tolerances);
        if (!step_exact && first_mismatch < 0) {
            first_mismatch = step;
        }
        exact = exact && step_exact;
        memory_exact = memory_exact
            && p4_memory_correspondence(fixture, retained_run, candidate_run);
        rebuilt += candidate_run.neighbor_rebuilt ? 1 : 0;
        reused += candidate_run.neighbor_reused ? 1 : 0;
        maximum_displacement = std::max(
            maximum_displacement, candidate_run.maximum_anchor_displacement);
        if (step != 0) {
            step_receipts << ',';
        }
        step_receipts << "{\"step\":" << step
                      << ",\"exact\":" << (step_exact ? "true" : "false")
                      << ",\"rebuilt\":"
                      << (candidate_run.neighbor_rebuilt ? "true" : "false")
                      << ",\"maximum_anchor_displacement_m\":"
                      << candidate_run.maximum_anchor_displacement
                      << ",\"retained_output_sha256\":\""
                      << ordered_output_digest(retained_run.state)
                      << "\",\"candidate_output_sha256\":\""
                      << ordered_output_digest(candidate_run.state)
                      << "\",\"retained_csr_sha256\":\"" << csr_digest(retained_run)
                      << "\",\"candidate_csr_sha256\":\"" << csr_digest(candidate_run)
                      << "\",\"active_pairs\":" << retained_run.state.directed_pairs
                      << ",\"candidate_pairs\":" << candidate_run.candidate_directed_pairs
                      << '}';
        if (step == 0) {
            retained_first = std::move(retained_run);
            candidate_first = std::move(candidate_run);
        }
    }
    step_receipts << ']';
    const bool transitions_valid = rebuilt >= 1 && rebuilt + reused == 3
        && (!profile.advected || reused >= 1);
    const bool passed = p2_check.passed && tiny.passed && stiff_i2
        && exact && memory_exact && transitions_valid;

    std::ostringstream output;
    output << std::setprecision(17);
    output << "{\"schema\":\"nextengine.nonlocal.p4_check.v1\",\"status\":\""
           << (passed ? "PASS" : "FAIL") << "\",\"candidate_class\":\"EXACT_RESULT_REUSE\""
           << ",\"candidate_identity\":\"verlet-skin-p4\",\"profile_id\":\""
           << profile.id << "\",\"iterations\":" << iterations
           << ",\"profile_sha256\":\"" << sha256_hex(canonical_profile_json(profile))
           << "\",\"input_sha256\":\"" << fixture_input_hash(fixture)
           << "\",\"binary_sha256\":\"" << executable_hash()
           << "\",\"skin_fraction\":0.040000000000000001"
           << ",\"correctness\":{\"p2_check_passed\":"
           << (p2_check.passed ? "true" : "false")
           << ",\"tiny_cpu_oracle_passed\":" << (tiny.passed ? "true" : "false")
           << ",\"tiny_first_failure\":\"" << tiny.first_failure
           << "\",\"stiff_surface_i2_passed\":" << (stiff_i2 ? "true" : "false")
           << ",\"three_step_exact\":" << (exact ? "true" : "false")
           << ",\"memory_exact\":" << (memory_exact ? "true" : "false")
           << ",\"transitions_valid\":" << (transitions_valid ? "true" : "false") << '}'
           << ",\"reuse\":{\"steps\":3,\"rebuild_count\":" << rebuilt
           << ",\"reuse_count\":" << reused
           << ",\"maximum_anchor_displacement_m\":" << maximum_displacement
           << ",\"seed_active_pairs\":" << retained_first.state.directed_pairs
           << ",\"seed_candidate_pairs\":" << candidate_first.candidate_directed_pairs
           << ",\"first_mismatch\":" << first_mismatch
           << ",\"step_receipts\":" << step_receipts.str() << '}'
           << ",\"retained\":{\"ordered_output_sha256\":\""
           << ordered_output_digest(retained_first.state) << "\",\"logical_csr_sha256\":\""
           << csr_digest(retained_first) << "\",\"device_memory_bytes\":"
           << retained_first.device_memory_bytes << "},\"candidate\":{\"ordered_output_sha256\":\""
           << ordered_output_digest(candidate_first.state)
           << "\",\"logical_csr_sha256\":\"" << csr_digest(candidate_first)
           << "\",\"superset_csr_sha256\":\"" << candidate_first.physical_csr_sha256
           << "\",\"device_memory_bytes\":" << candidate_first.device_memory_bytes
           << "},\"device\":" << device_json() << '}';
    return {passed, output.str()};
}

CommandReport run_cuda_p4_tournament(
    const Profile& profile,
    int warmup,
    int runs) {
    if (profile.record_version != 1 || warmup != 32 || runs != 96
        || profile.id == "nuv-water-100k-report.v1") {
        throw std::invalid_argument(
            "P4 tournament requires eligible v1 --warmup 32 --runs 96");
    }
    if (profile.id == "nuv-surface-stiff-16k-i2.v1"
        || profile.id == "nuv-water-50k-permuted.v1") {
        throw std::invalid_argument("P4 tournament requires a retained-stack control profile");
    }
    const CommandReport check = run_cuda_p4_check(profile, profile.fixed_iterations);
    const Fixture fixture = performance_fixture(profile, profile.fixed_iterations);
    CudaBaseline retained(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2);
    CudaBaseline candidate(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2,
        DynamicStorageMode::Retained, NeighborReuseMode::CertifiedVerletP4);

    bool trace_exact = check.passed;
    bool trace_memory_exact = true;
    int first_trace_mismatch = -1;
    int trace_rebuilds = 0;
    int trace_reuses = 0;
    std::size_t maximum_candidate_pairs = 0;
    double maximum_displacement = 0.0;
    std::ostringstream trace_material;
    std::string prior_state_digest = fixture_input_hash(fixture);
    retained.reset_seed();
    candidate.reset_seed();
    const int trace_length = profile.advected ? profile.trace_length : 3;
    CapturedRun retained_seed;
    CapturedRun candidate_seed;
    for (int step = 0; step < trace_length; ++step) {
        CapturedRun retained_run = retained.execute(true, profile.advected);
        CapturedRun candidate_run = candidate.execute(true, profile.advected);
        const bool step_exact = exact_p1_correspondence(
            fixture, retained_run, candidate_run, profile.tolerances);
        const bool step_memory = p4_memory_correspondence(
            fixture, retained_run, candidate_run);
        if ((!step_exact || !step_memory) && first_trace_mismatch < 0) {
            first_trace_mismatch = step;
        }
        trace_exact = trace_exact && step_exact;
        trace_memory_exact = trace_memory_exact && step_memory;
        trace_rebuilds += candidate_run.neighbor_rebuilt ? 1 : 0;
        trace_reuses += candidate_run.neighbor_reused ? 1 : 0;
        maximum_candidate_pairs = std::max(
            maximum_candidate_pairs, candidate_run.candidate_directed_pairs);
        maximum_displacement = std::max(
            maximum_displacement, candidate_run.maximum_anchor_displacement);
        const std::string logical_csr_sha256 = csr_digest(retained_run);
        const std::string membership_sha256 =
            canonical_lattice_csr_digest(fixture, retained_run);
        const std::string output_sha256 = ordered_output_digest(retained_run.state);
        const std::string state_sha256 = handoff_digest(retained_run.state);
        trace_material << step << '|' << prior_state_digest << '|' << logical_csr_sha256
                       << '|' << membership_sha256 << '|' << output_sha256 << '|'
                       << state_sha256 << '|' << candidate_run.physical_csr_sha256 << '|'
                       << (candidate_run.neighbor_rebuilt ? 'R' : 'U') << '|'
                       << candidate_run.maximum_anchor_displacement << '|';
        prior_state_digest = state_sha256;
        if (step == 0) {
            retained_seed = std::move(retained_run);
            candidate_seed = std::move(candidate_run);
        }
    }
    const std::string trace_sha256 = sha256_hex(trace_material.str());
    constexpr int CONDITIONING_RUNS = 256;
    std::array<CudaBaseline*, 2> implementations = {&retained, &candidate};
    int measured_rebuilds = 0;
    int measured_reuses = 0;
    const auto execute_round = [&](int round, bool capture, std::vector<StageTiming>* timings) {
        for (int position = 0; position < 2; ++position) {
            const int identity = (round + position) % 2;
            CapturedRun run = implementations[static_cast<std::size_t>(identity)]->execute(
                capture, profile.advected);
            if (run.local_solve_failed || !run.neighbor_build_valid) {
                throw std::runtime_error("P4 identity failed during tournament");
            }
            if (timings != nullptr) {
                timings[identity].push_back(run.timing);
                if (identity == 1) {
                    measured_rebuilds += run.neighbor_rebuilt ? 1 : 0;
                    measured_reuses += run.neighbor_reused ? 1 : 0;
                }
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
    retained.reset_seed();
    candidate.reset_seed();
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
    const bool improvement_gate = total_speedup >= 1.05;
    const bool regression_gate = candidate_total.p95 <= retained_total.p95 * 1.02;
    const bool advected_water = profile.id == "nuv-water-50k-advected.v1";
    const bool profile_gate = advected_water ? improvement_gate : regression_gate;
    const bool transitions_valid = trace_rebuilds >= 1 && trace_reuses >= 1
        && trace_rebuilds + trace_reuses == trace_length
        && measured_rebuilds >= 1 && measured_reuses >= 1;
    const std::size_t added_memory = candidate_seed.device_memory_bytes
        - retained_seed.device_memory_bytes;

    std::ostringstream output;
    output << std::setprecision(17);
    output << "{\"schema\":\"nextengine.nonlocal.p4_tournament.v1\",\"status\":\""
           << (trace_exact && trace_memory_exact && transitions_valid ? "PASS" : "FAIL")
           << "\",\"candidate_class\":\"EXACT_RESULT_REUSE\""
           << ",\"candidate_identity\":\"verlet-skin-p4\",\"profile_id\":\""
           << profile.id << "\",\"profile_sha256\":\""
           << sha256_hex(canonical_profile_json(profile)) << "\",\"input_sha256\":\""
           << fixture_input_hash(fixture) << "\",\"trace_sha256\":\"" << trace_sha256
           << "\",\"binary_sha256\":\"" << executable_hash()
           << "\",\"command\":\"nonlocal-feasibility --p4-tournament " << profile.id
           << " --warmup 32 --runs 96\",\"conditioning_runs\":" << CONDITIONING_RUNS
           << ",\"warmup_rounds\":" << warmup << ",\"measured_rounds\":" << runs
           << ",\"rotation\":[[\"retained-p2\",\"candidate-p4\"],"
              "[\"candidate-p4\",\"retained-p2\"]]"
           << ",\"correctness\":{\"check_passed\":" << (check.passed ? "true" : "false")
           << ",\"trace_exact\":" << (trace_exact ? "true" : "false")
           << ",\"trace_memory_exact\":" << (trace_memory_exact ? "true" : "false")
           << ",\"transitions_valid\":" << (transitions_valid ? "true" : "false")
           << ",\"first_trace_mismatch\":" << first_trace_mismatch
           << ",\"retained_seed_output_sha256\":\""
           << ordered_output_digest(retained_seed.state)
           << "\",\"candidate_seed_output_sha256\":\""
           << ordered_output_digest(candidate_seed.state)
           << "\",\"active_logical_csr_sha256\":\"" << csr_digest(retained_seed)
           << "\",\"seed_superset_csr_sha256\":\""
           << candidate_seed.physical_csr_sha256 << "\"}"
           << ",\"reuse\":{\"skin_fraction\":0.040000000000000001"
           << ",\"trace_rebuild_count\":" << trace_rebuilds
           << ",\"trace_reuse_count\":" << trace_reuses
           << ",\"measured_rebuild_count\":" << measured_rebuilds
           << ",\"measured_reuse_count\":" << measured_reuses
           << ",\"maximum_anchor_displacement_m\":" << maximum_displacement
           << ",\"seed_active_pairs\":" << retained_seed.state.directed_pairs
           << ",\"seed_candidate_pairs\":" << candidate_seed.candidate_directed_pairs
           << ",\"maximum_trace_candidate_pairs\":" << maximum_candidate_pairs << '}'
           << ",\"retention\":{\"advected_water_target\":"
           << (advected_water ? "true" : "false")
           << ",\"improvement_gate\":" << (improvement_gate ? "true" : "false")
           << ",\"regression_gate\":" << (regression_gate ? "true" : "false")
           << ",\"profile_gate\":" << (profile_gate ? "true" : "false")
           << ",\"total_p95_speedup\":" << total_speedup
           << ",\"pair_stage_p95_speedup\":" << pair_speedup
           << ",\"retained_total_p95_ms\":" << retained_total.p95
           << ",\"candidate_total_p95_ms\":" << candidate_total.p95
           << ",\"retained_pair_stage_p95_ms\":" << retained_pair.p95
           << ",\"candidate_pair_stage_p95_ms\":" << candidate_pair.p95 << '}'
           << ",\"memory\":{\"retained_bytes\":" << retained_seed.device_memory_bytes
           << ",\"candidate_bytes\":" << candidate_seed.device_memory_bytes
           << ",\"added_cache_bytes\":" << added_memory << '}'
           << ",\"identities\":{\"retained_p2\":{\"statistics\":";
    append_stage_statistics(output, timings[0]);
    output << ",\"raw_total_ms\":";
    append_raw_totals(output, timings[0]);
    output << "},\"candidate_p4\":{\"statistics\":";
    append_stage_statistics(output, timings[1]);
    output << ",\"raw_total_ms\":";
    append_raw_totals(output, timings[1]);
    output << "}},\"device\":" << device_json() << '}';
    return {trace_exact && trace_memory_exact && transitions_valid, output.str()};
}

CommandReport run_cuda_p2_decision(
    const Profile& profile,
    int warmup,
    int runs) {
    if ((profile.id != "nuv-water-50k-coherent.v1"
            && profile.id != "nuv-water-50k-advected.v1"
            && profile.id != "nuv-basin-48k-analytic-contact-game.v5"
            && profile.id !=
                "nuv-basin-48k-analytic-contact-game-cap160.v6")
        || warmup != 64 || runs != 512) {
        throw std::invalid_argument(
            "P2 decision requires an admitted coherent/advected/game-contact profile "
            "with --warmup 64 --runs 512");
    }
    const CommandReport check = run_cuda_p2_check(profile, profile.fixed_iterations);
    const Fixture fixture = performance_fixture(profile, profile.fixed_iterations);
    bool trace_exact = check.passed;
    bool trace_memory_exact = true;
    int first_trace_mismatch = -1;
    std::ostringstream trace_material;
    std::string prior_state_digest = fixture_input_hash(fixture);
    CapturedRun finalist_seed;
    {
        CudaBaseline retained(
            fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
            PairTraversalMode::FusedOwnerTermsP1);
        CudaBaseline finalist(
            fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
            PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2);
        retained.reset_seed();
        finalist.reset_seed();
        const int trace_length = profile.advected ? profile.trace_length : 1;
        for (int step = 0; step < trace_length; ++step) {
            CapturedRun retained_run = retained.execute(true, profile.advected);
            CapturedRun finalist_run = finalist.execute(true, profile.advected);
            const bool step_exact = exact_p1_correspondence(
                fixture, retained_run, finalist_run, profile.tolerances);
            const bool step_memory = compact_memory_correspondence(
                fixture, retained_run, finalist_run);
            if ((!step_exact || !step_memory) && first_trace_mismatch < 0) {
                first_trace_mismatch = step;
            }
            trace_exact = trace_exact && step_exact;
            trace_memory_exact = trace_memory_exact && step_memory;
            const std::string logical_csr_sha256 = csr_digest(finalist_run);
            const std::string membership_sha256 =
                canonical_lattice_csr_digest(fixture, finalist_run);
            const std::string output_sha256 = ordered_output_digest(finalist_run.state);
            const std::string state_sha256 = handoff_digest(finalist_run.state);
            trace_material << step << '|' << prior_state_digest << '|' << logical_csr_sha256
                           << '|' << membership_sha256 << '|' << output_sha256 << '|'
                           << state_sha256 << '|';
            prior_state_digest = state_sha256;
            if (step == 0) {
                finalist_seed = std::move(finalist_run);
            }
        }
    }
    const std::string trace_sha256 = sha256_hex(trace_material.str());

    CudaBaseline finalist(
        fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
        PairTraversalMode::FusedOwnerTermsP1, NeighborEncodingMode::CompactU16P2);
    constexpr int CONDITIONING_RUNS = 256;
    const auto execute_epoch_step = [&](int index, bool capture) {
        CapturedRun run = finalist.execute(capture, profile.advected);
        if (run.local_solve_failed || !run.neighbor_build_valid) {
            throw std::runtime_error("P2 finalist failed during decision campaign");
        }
        if (profile.advected && (index + 1) % profile.trace_length == 0) {
            finalist.reset_seed();
        }
        return run;
    };
    finalist.reset_seed();
    for (int run = 0; run < CONDITIONING_RUNS; ++run) {
        execute_epoch_step(run, false);
    }
    finalist.reset_seed();
    for (int run = 0; run < warmup; ++run) {
        execute_epoch_step(run, false);
    }
    finalist.reset_seed();
    std::vector<StageTiming> timings;
    timings.reserve(static_cast<std::size_t>(runs));
    bool measurement_valid = true;
    int failed_measurement = -1;
    for (int run = 0; run < runs; ++run) {
        const CapturedRun measured = execute_epoch_step(run, false);
        if (measured.local_solve_failed || !measured.neighbor_build_valid) {
            measurement_valid = false;
            failed_measurement = run;
            break;
        }
        timings.push_back(measured.timing);
    }
    const bool correctness = trace_exact && trace_memory_exact
        && measurement_valid && timings.size() == static_cast<std::size_t>(runs);
    const Statistics total = collect_statistics(timings, &StageTiming::total);
    const bool p95_gate = total.p95 <= 4.0;
    const bool p99_gate = total.p99 <= 6.0;
    const bool decision_gate = correctness && p95_gate && p99_gate;

    std::ostringstream output;
    output << std::setprecision(17);
    output << "{\"schema\":\"nextengine.nonlocal.p2_decision.v1\",\"status\":\""
           << (correctness ? "PASS" : "FAIL") << "\",\"candidate_class\":\"EXACT_WORK\""
           << ",\"candidate_identity\":\"fused-owner-terms-p1+compact-csr-u16-p2\""
           << ",\"profile_id\":\"" << profile.id << "\",\"profile_sha256\":\""
           << sha256_hex(canonical_profile_json(profile)) << "\",\"input_sha256\":\""
           << fixture_input_hash(fixture) << "\",\"trace_sha256\":\"" << trace_sha256
           << "\",\"binary_sha256\":\"" << executable_hash()
           << "\",\"command\":\"nonlocal-feasibility --p2-decision " << profile.id
           << " --warmup 64 --runs 512\",\"conditioning_runs\":" << CONDITIONING_RUNS
           << ",\"analytic_contact_in_primary_timing\":"
           << (fixture.analytic_box_contact ? "true" : "false")
           << ",\"warmup_runs\":" << warmup << ",\"measured_runs\":" << runs
           << ",\"correctness\":{\"p2_check_passed\":"
           << (check.passed ? "true" : "false")
           << ",\"trace_exact\":" << (trace_exact ? "true" : "false")
           << ",\"trace_memory_exact\":" << (trace_memory_exact ? "true" : "false")
           << ",\"first_trace_mismatch\":" << first_trace_mismatch
           << ",\"measurement_valid\":" << (measurement_valid ? "true" : "false")
           << ",\"failed_measurement_index\":" << failed_measurement
           << ",\"seed_output_sha256\":\"" << ordered_output_digest(finalist_seed.state)
           << "\",\"seed_logical_csr_sha256\":\"" << csr_digest(finalist_seed) << "\"}"
           << ",\"decision\":{\"p95_limit_ms\":4,\"p99_limit_ms\":6"
           << ",\"p95_gate\":" << (p95_gate ? "true" : "false")
           << ",\"p99_gate\":" << (p99_gate ? "true" : "false")
           << ",\"decision_gate\":" << (decision_gate ? "true" : "false")
           << ",\"p95_ms\":" << total.p95 << ",\"p99_ms\":" << total.p99 << '}'
           << ",\"capacity\":{\"device_memory_bytes\":" << finalist_seed.device_memory_bytes
           << ",\"neighbor_id_bytes\":" << finalist_seed.neighbor_id_bytes
           << ",\"directed_pairs\":" << finalist_seed.state.directed_pairs
           << ",\"maximum_degree\":" << finalist_seed.state.maximum_degree << '}'
           << ",\"statistics\":";
    append_stage_statistics(output, timings);
    output << ",\"raw_total_ms\":";
    append_raw_totals(output, timings);
    output << ",\"device\":" << device_json() << '}';
    return {correctness, output.str()};
}

namespace {

constexpr double GAME_SPACING = 0.05;
constexpr double GAME_RADIUS = 0.025;
constexpr double GAME_MASS = 0.125;
constexpr double GAME_TIME_STEP = 1.0 / 240.0;

struct GameQualityBox {
    Vec3 minimum;
    Vec3 maximum;
    std::array<int, 3> cells;
};

struct GameContactResult {
    Vec3 position;
    Vec3 velocity;
    std::vector<int> features;
    double penetration = 0.0;
};

struct GameScenarioResult {
    std::string id;
    bool apparatus_passed = true;
    bool quality_passed = false;
    int steps = 0;
    std::size_t dynamic_samples = 0;
    std::size_t boundary_samples = 0;
    std::size_t components = 0;
    std::size_t satellites = 0;
    double maximum_penetration = 0.0;
    double maximum_fixed_displacement = 0.0;
    double maximum_contact_velocity_error = 0.0;
    double maximum_speed = 0.0;
    double maximum_positive_compression = 0.0;
    double horizontal_com_drift = 0.0;
    double vertical_com_drift = 0.0;
    double forward_com_travel = 0.0;
    double solver_total_ms = 0.0;
    double solver_maximum_ms = 0.0;
    Vec3 final_com{};
    std::string trace_sha256;
    std::string first_failure;
};

double game_axis(Vec3 value, int component) {
    if (component == 0) {
        return value.x;
    }
    if (component == 1) {
        return value.y;
    }
    return value.z;
}

void set_game_axis(Vec3& value, int component, double scalar) {
    if (component == 0) {
        value.x = scalar;
    } else if (component == 1) {
        value.y = scalar;
    } else {
        value.z = scalar;
    }
}

GameContactResult game_sweep_box(
    Vec3 start,
    Vec3 tentative,
    const GameQualityBox& box) {
    GameContactResult result;
    result.position = tentative;
    const float radius = static_cast<float>(GAME_RADIUS);
    const std::array<double, 3> lower = {
        static_cast<double>(static_cast<float>(box.minimum.x) + radius),
        static_cast<double>(static_cast<float>(box.minimum.y) + radius),
        static_cast<double>(static_cast<float>(box.minimum.z) + radius),
    };
    const std::array<double, 3> upper = {
        static_cast<double>(static_cast<float>(box.maximum.x) - radius),
        static_cast<double>(static_cast<float>(box.maximum.y) - radius),
        static_cast<double>(static_cast<float>(box.maximum.z) - radius),
    };
    const std::array<int, 3> lower_feature = {0, 2, 4};
    const std::array<int, 3> upper_feature = {1, 3, 5};
    for (int component = 0; component < 3; ++component) {
        const double start_value = game_axis(start, component);
        const double tentative_value = game_axis(tentative, component);
        const double displacement = tentative_value - start_value;
        if (tentative_value <= lower[component] && displacement < 0.0) {
            set_game_axis(result.position, component, lower[component]);
            result.features.push_back(lower_feature[component]);
        } else if (tentative_value >= upper[component] && displacement > 0.0) {
            set_game_axis(result.position, component, upper[component]);
            result.features.push_back(upper_feature[component]);
        }
    }
    std::sort(result.features.begin(), result.features.end());
    result.velocity = (result.position - start) / GAME_TIME_STEP;
    for (int component = 0; component < 3; ++component) {
        result.penetration = std::max(result.penetration,
            std::max(lower[component] - game_axis(result.position, component),
                game_axis(result.position, component) - upper[component]));
    }
    result.penetration = std::max(result.penetration, 0.0);
    return result;
}

std::size_t append_game_boundary(
    Fixture& fixture,
    const GameQualityBox& box,
    int layers,
    bool with_lid = true) {
    const std::size_t before = fixture.particles.size();
    const int top = with_lid ? box.cells[1] + layers : box.cells[1];
    for (int x = -layers; x < box.cells[0] + layers; ++x) {
        for (int y = -layers; y < top; ++y) {
            for (int z = -layers; z < box.cells[2] + layers; ++z) {
                if (x >= 0 && x < box.cells[0] && y >= 0 && y < box.cells[1]
                    && z >= 0 && z < box.cells[2]) {
                    continue;
                }
                fixture.particles.push_back({
                    {
                        box.minimum.x + GAME_RADIUS + x * GAME_SPACING,
                        box.minimum.y + GAME_RADIUS + y * GAME_SPACING,
                        box.minimum.z + GAME_RADIUS + z * GAME_SPACING,
                    },
                    {},
                    true,
                });
            }
        }
    }
    return fixture.particles.size() - before;
}

Fixture game_fixture(
    const Profile& profile,
    const std::string& name,
    const std::vector<Particle>& fluid,
    const GameQualityBox& box,
    int iterations,
    std::size_t neighbor_capacity = 123U,
    int boundary_layers = 0,
    bool boundary_lid = true) {
    Fixture fixture;
    fixture.name = name;
    fixture.rest_density = profile.rest_density;
    fixture.spacing = GAME_SPACING;
    fixture.mass = GAME_MASS;
    fixture.horizon = profile.horizon;
    fixture.time_step = GAME_TIME_STEP;
    fixture.gravity = profile.gravity;
    fixture.kappa = profile.kappa;
    fixture.lambda = profile.lambda;
    fixture.mu = profile.mu;
    fixture.gamma = profile.gamma;
    fixture.terms = profile.terms;
    fixture.iterations = iterations;
    fixture.particles = fluid;
    const std::size_t boundary =
        append_game_boundary(fixture, box, boundary_layers, boundary_lid);
    fixture.pair_capacity = (fluid.size() + boundary) * neighbor_capacity;
    fixture.analytic_box_contact = true;
    fixture.contact_minimum = box.minimum;
    fixture.contact_maximum = box.maximum;
    fixture.particle_radius = GAME_RADIUS;
    return fixture;
}

// NGQ8 revision 9: solid material of the under-floor pipe scene (shelf,
// divider, protruding pipe body) minus the shaft and duct channels.
bool spill_pipe_solid(const Vec3& p, const Fixture::Spill& spill) {
    const bool shaft = p.x > spill.shaft_x0 && p.x < spill.shaft_x1 && p.z > spill.opening_z0
        && p.z < spill.opening_z1 && p.y > spill.opening_y0;
    const bool duct = p.y > spill.opening_y0 && p.y < spill.opening_y1 && p.z > spill.opening_z0
        && p.z < spill.opening_z1 && p.x > spill.shaft_x0 && p.x < spill.pipe_x1;
    if (shaft || duct) {
        return false;
    }
    const bool shelf = p.x < spill.wall_x0 && p.y < spill.shelf_top;
    const bool wall = p.x > spill.wall_x0 && p.x < spill.wall_x1;
    const bool pipe = p.x > spill.wall_x1 && p.x < spill.pipe_x1
        && p.y > spill.opening_y0 - spill.pipe_wall && p.y < spill.opening_y1 + spill.pipe_wall
        && p.z > spill.opening_z0 - spill.pipe_wall && p.z < spill.opening_z1 + spill.pipe_wall;
    return shelf || wall || pipe;
}

// NGQ8 revision 9: fixed density-only samples in every solid lattice cell
// within two cells of a non-solid cell (shelf top layers, shaft and duct
// linings, the whole divider, the pipe body).
std::size_t append_spill_pipe_solids(
    Fixture& fixture,
    const GameQualityBox& box,
    const Fixture::Spill& spill) {
    const std::size_t before = fixture.particles.size();
    const auto centre = [&](int x, int y, int z) {
        return Vec3{
            box.minimum.x + GAME_RADIUS + x * GAME_SPACING,
            box.minimum.y + GAME_RADIUS + y * GAME_SPACING,
            box.minimum.z + GAME_RADIUS + z * GAME_SPACING,
        };
    };
    const auto solid = [&](int x, int y, int z) {
        return spill_pipe_solid(centre(x, y, z), spill);
    };
    for (int x = 0; x < box.cells[0]; ++x) {
        for (int y = 0; y < box.cells[1]; ++y) {
            for (int z = 0; z < box.cells[2]; ++z) {
                if (!solid(x, y, z)) {
                    continue;
                }
                bool near_free = false;
                for (int dx = -2; dx <= 2 && !near_free; ++dx) {
                    for (int dy = -2; dy <= 2 && !near_free; ++dy) {
                        for (int dz = -2; dz <= 2 && !near_free; ++dz) {
                            const int nx = x + dx;
                            const int ny = y + dy;
                            const int nz = z + dz;
                            if (nx < 0 || ny < 0 || nz < 0 || nx >= box.cells[0]
                                || ny >= box.cells[1] || nz >= box.cells[2]) {
                                continue;
                            }
                            near_free = !solid(nx, ny, nz);
                        }
                    }
                }
                if (near_free) {
                    fixture.particles.push_back({centre(x, y, z), {}, true});
                }
            }
        }
    }
    return fixture.particles.size() - before;
}

// NGQ8 revision 9: depth of one fluid sample inside the pipe-scene solids
// (distance to the nearest exit face), zero inside a channel or free space.
double spill_pipe_penetration(const Vec3& p, const Fixture::Spill& spill) {
    constexpr double e = 1e-5;
    const bool z_window = p.z >= spill.opening_z0 - e && p.z <= spill.opening_z1 + e;
    const bool shaft = p.x >= spill.shaft_x0 - e && p.x <= spill.shaft_x1 + e && z_window
        && p.y >= spill.opening_y0 - e;
    const bool duct = p.y >= spill.opening_y0 - e && p.y <= spill.opening_y1 + e && z_window
        && p.x >= spill.shaft_x0 - e && p.x <= spill.pipe_x1 + e;
    if (shaft || duct) {
        return 0.0;
    }
    double penetration = 0.0;
    if (p.x < spill.wall_x0 && p.y < spill.shelf_top) {
        penetration = std::max(penetration, spill.shelf_top - p.y);
    }
    if (p.x > spill.wall_x0 && p.x < spill.wall_x1) {
        penetration = std::max(penetration, std::min(p.x - spill.wall_x0, spill.wall_x1 - p.x));
    }
    const double py0 = spill.opening_y0 - spill.pipe_wall;
    const double py1 = spill.opening_y1 + spill.pipe_wall;
    const double pz0 = spill.opening_z0 - spill.pipe_wall;
    const double pz1 = spill.opening_z1 + spill.pipe_wall;
    if (p.x > spill.wall_x1 && p.x < spill.pipe_x1 && p.y > py0 && p.y < py1 && p.z > pz0
        && p.z < pz1) {
        const double depth = std::min(
            {spill.pipe_x1 - p.x, p.y - py0, py1 - p.y, p.z - pz0, pz1 - p.z});
        penetration = std::max(penetration, depth);
    }
    return penetration;
}

// NGQ8: fixed density-only samples filling the top two shelf layers and the
// whole divider slab minus the opening, on the same lattice as the box.
std::size_t append_spill_solids(
    Fixture& fixture,
    const GameQualityBox& box,
    const Fixture::Spill& spill) {
    if (spill.under_floor) {
        return append_spill_pipe_solids(fixture, box, spill);
    }
    const std::size_t before = fixture.particles.size();
    const auto cell = [&](double value, double origin) {
        return static_cast<int>(std::llround((value - origin) / GAME_SPACING));
    };
    const int shelf_y = cell(spill.shelf_top, box.minimum.y);
    const int wall_x0 = cell(spill.wall_x0, box.minimum.x);
    const int wall_x1 = cell(spill.wall_x1, box.minimum.x);
    const int opening_y0 = cell(spill.opening_y0, box.minimum.y);
    const int opening_y1 = cell(spill.opening_y1, box.minimum.y);
    const int opening_z0 = cell(spill.opening_z0, box.minimum.z);
    const int opening_z1 = cell(spill.opening_z1, box.minimum.z);
    const auto push = [&](int x, int y, int z) {
        fixture.particles.push_back({
            {
                box.minimum.x + GAME_RADIUS + x * GAME_SPACING,
                box.minimum.y + GAME_RADIUS + y * GAME_SPACING,
                box.minimum.z + GAME_RADIUS + z * GAME_SPACING,
            },
            {},
            true,
        });
    };
    for (int x = 0; x < wall_x0; ++x) {
        for (int y = std::max(0, shelf_y - 2); y < shelf_y; ++y) {
            for (int z = 0; z < box.cells[2]; ++z) {
                push(x, y, z);
            }
        }
    }
    const int ring = spill.open_ring ? 1 : 0;
    for (int x = wall_x0; x < wall_x1; ++x) {
        for (int y = 0; y < box.cells[1]; ++y) {
            for (int z = 0; z < box.cells[2]; ++z) {
                if (y >= opening_y0 - ring && y < opening_y1 + ring && z >= opening_z0 - ring
                    && z < opening_z1 + ring) {
                    continue;
                }
                push(x, y, z);
            }
        }
    }
    return fixture.particles.size() - before;
}

// Penetration of one fluid sample into the shelf or the divider (outside the
// opening), zero when the sample is in free space.
double spill_penetration(const Vec3& p, const Fixture::Spill& spill) {
    if (spill.under_floor) {
        return spill_pipe_penetration(p, spill);
    }
    double penetration = 0.0;
    if (p.x < spill.wall_x0 && p.y < spill.shelf_top) {
        penetration = std::max(penetration, spill.shelf_top - p.y);
    }
    if (p.x > spill.wall_x0 && p.x < spill.wall_x1) {
        // Inclusive with a float-rounding allowance: a flush-clamped sample
        // sits on the opening face at binary32 precision.
        constexpr double face_epsilon = 1e-5;
        const bool window = p.y >= spill.opening_y0 - face_epsilon
            && p.y <= spill.opening_y1 + face_epsilon && p.z >= spill.opening_z0 - face_epsilon
            && p.z <= spill.opening_z1 + face_epsilon;
        if (!window) {
            penetration = std::max(
                penetration, std::min(p.x - spill.wall_x0, spill.wall_x1 - p.x));
        }
    }
    return penetration;
}

Vec3 game_center_of_mass(const std::vector<Particle>& fluid) {
    Vec3 result{};
    for (const Particle& particle : fluid) {
        result += particle.position;
    }
    return result / static_cast<double>(fluid.size());
}

std::pair<std::size_t, std::size_t> game_topology(
    const std::vector<Particle>& fluid) {
    const std::size_t count = fluid.size();
    std::vector<std::uint8_t> visited(count, 0U);
    std::vector<std::size_t> stack;
    std::size_t components = 0;
    std::size_t largest = 0;
    const double link_distance = 1.75 * GAME_SPACING;
    for (std::size_t seed = 0; seed < count; ++seed) {
        if (visited[seed] != 0U) {
            continue;
        }
        ++components;
        std::size_t component_size = 0;
        visited[seed] = 1U;
        stack.clear();
        stack.push_back(seed);
        while (!stack.empty()) {
            const std::size_t owner = stack.back();
            stack.pop_back();
            ++component_size;
            for (std::size_t neighbor = 0; neighbor < count; ++neighbor) {
                if (visited[neighbor] == 0U
                    && norm(fluid[owner].position - fluid[neighbor].position)
                        <= link_distance) {
                    visited[neighbor] = 1U;
                    stack.push_back(neighbor);
                }
            }
        }
        largest = std::max(largest, component_size);
    }
    return {components, count - largest};
}

bool game_finite(const CapturedRun& run, std::size_t fluid_count) {
    if (run.local_solve_failed || !run.neighbor_build_valid
        || run.state.next_position.size() < fluid_count
        || run.state.final_velocity.size() < fluid_count
        || run.state.density.size() < fluid_count) {
        return false;
    }
    for (std::size_t index = 0; index < fluid_count; ++index) {
        if (!finite(run.state.next_position[index])
            || !finite(run.state.final_velocity[index])
            || !std::isfinite(run.state.density[index])) {
            return false;
        }
    }
    return true;
}

Vec3 game_uploaded_position(const Particle& particle) {
    return {
        static_cast<double>(static_cast<float>(particle.position.x)),
        static_cast<double>(static_cast<float>(particle.position.y)),
        static_cast<double>(static_cast<float>(particle.position.z)),
    };
}

void game_accumulate_step(
    GameScenarioResult& result,
    const Fixture& fixture,
    const CapturedRun& run,
    std::vector<Particle>& fluid,
    const GameQualityBox& box,
    std::ostringstream& trace) {
    const bool step_valid = game_finite(run, fluid.size())
        && run.state.next_position.size() == fixture.particles.size();
    result.apparatus_passed = result.apparatus_passed && step_valid;
    result.solver_total_ms += run.timing.total;
    result.solver_maximum_ms = std::max(result.solver_maximum_ms, run.timing.total);
    if (!step_valid) {
        trace << "INVALID_STEP|";
        return;
    }
    for (std::size_t index = fluid.size(); index < fixture.particles.size(); ++index) {
        result.maximum_fixed_displacement = std::max(
            result.maximum_fixed_displacement,
            norm(run.state.next_position[index]
                - game_uploaded_position(fixture.particles[index])));
    }
    for (std::size_t index = 0; index < fluid.size(); ++index) {
        const GameContactResult contact = game_sweep_box(
            fluid[index].position, run.state.next_position[index], box);
        result.maximum_penetration = std::max(
            result.maximum_penetration, contact.penetration);
        result.maximum_speed = std::max(result.maximum_speed, norm(contact.velocity));
        result.maximum_contact_velocity_error = std::max(
            result.maximum_contact_velocity_error,
            norm(run.state.final_velocity[index] - contact.velocity));
        const double positive_compression = std::max(
            run.state.density[index] / fixture.rest_density - 1.0, 0.0);
        result.maximum_positive_compression = std::max(
            result.maximum_positive_compression, positive_compression);
        fluid[index].position = run.state.next_position[index];
        fluid[index].velocity = run.state.final_velocity[index];
    }
    trace << ordered_output_digest(run.state) << '|';
    for (const Particle& particle : fluid) {
        trace << std::setprecision(17)
              << particle.position.x << ',' << particle.position.y << ','
              << particle.position.z << ';' << particle.velocity.x << ','
              << particle.velocity.y << ',' << particle.velocity.z << '|';
    }
}

GameScenarioResult run_game_hold(const Profile& profile, int iterations) {
    const GameQualityBox box{{0.0, 0.0, 0.0}, {0.1, 0.2, 0.1}, {2, 4, 2}};
    std::vector<Particle> fluid;
    for (int x = 0; x < 2; ++x) {
        for (int y = 0; y < 2; ++y) {
            for (int z = 0; z < 2; ++z) {
                fluid.push_back({
                    {GAME_RADIUS + x * GAME_SPACING,
                        GAME_RADIUS + y * GAME_SPACING,
                        GAME_RADIUS + z * GAME_SPACING},
                    {},
                    false,
                });
            }
        }
    }
    const Vec3 initial_com = game_center_of_mass(fluid);
    GameScenarioResult result;
    result.id = "confined_hold";
    result.steps = 24;
    result.dynamic_samples = fluid.size();
    std::ostringstream trace;
    for (int step = 0; step < result.steps && result.apparatus_passed; ++step) {
        const Fixture fixture = game_fixture(
            profile, "game-confined-hold", fluid, box, iterations);
        result.boundary_samples = fixture.particles.size() - fluid.size();
        CudaBaseline gpu(
            fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
            PairTraversalMode::FusedOwnerTermsP1,
            NeighborEncodingMode::CompactU16P2);
        const CapturedRun run = gpu.execute(true);
        game_accumulate_step(result, fixture, run, fluid, box, trace);
    }
    result.final_com = game_center_of_mass(fluid);
    result.horizontal_com_drift = std::hypot(
        result.final_com.x - initial_com.x, result.final_com.z - initial_com.z);
    result.vertical_com_drift = std::abs(result.final_com.y - initial_com.y);
    std::tie(result.components, result.satellites) = game_topology(fluid);
    result.trace_sha256 = sha256_hex(trace.str());
    result.quality_passed = result.apparatus_passed
        && result.maximum_penetration <= 1.0e-12
        && result.maximum_fixed_displacement <= 1.0e-12
        && result.maximum_contact_velocity_error <= 1.0e-4
        && result.horizontal_com_drift <= 0.005
        && result.vertical_com_drift <= GAME_RADIUS
        && result.maximum_speed <= 1.0
        && result.components == 1U && result.satellites == 0U;
    if (!result.apparatus_passed) {
        result.first_failure = "apparatus";
    } else if (result.maximum_penetration > 1.0e-12) {
        result.first_failure = "containment";
    } else if (result.maximum_fixed_displacement > 1.0e-12) {
        result.first_failure = "fixed_boundary_moved";
    } else if (result.maximum_contact_velocity_error > 1.0e-4) {
        result.first_failure = "contact_velocity_correspondence";
    } else if (result.horizontal_com_drift > 0.005) {
        result.first_failure = "horizontal_drift";
    } else if (result.vertical_com_drift > GAME_RADIUS) {
        result.first_failure = "vertical_drift";
    } else if (result.maximum_speed > 1.0) {
        result.first_failure = "speed";
    } else if (result.components != 1U || result.satellites != 0U) {
        result.first_failure = "topology";
    }
    return result;
}

GameScenarioResult run_game_release(const Profile& profile, int iterations) {
    const GameQualityBox box{{0.0, 0.0, 0.0}, {0.4, 0.3, 0.3}, {8, 6, 6}};
    std::vector<Particle> fluid;
    for (int x = 0; x < 4; ++x) {
        for (int y = 0; y < 4; ++y) {
            for (int z = 0; z < 4; ++z) {
                fluid.push_back({
                    {GAME_RADIUS + x * GAME_SPACING,
                        GAME_RADIUS + y * GAME_SPACING,
                        GAME_RADIUS + GAME_SPACING + z * GAME_SPACING},
                    {0.75, 0.0, 0.0},
                    false,
                });
            }
        }
    }
    const Vec3 initial_com = game_center_of_mass(fluid);
    GameScenarioResult result;
    result.id = "release_contact";
    result.steps = 48;
    result.dynamic_samples = fluid.size();
    std::ostringstream trace;
    for (int step = 0; step < result.steps && result.apparatus_passed; ++step) {
        const Fixture fixture = game_fixture(
            profile, "game-release-contact", fluid, box, iterations);
        result.boundary_samples = fixture.particles.size() - fluid.size();
        CudaBaseline gpu(
            fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
            PairTraversalMode::FusedOwnerTermsP1,
            NeighborEncodingMode::CompactU16P2);
        const CapturedRun run = gpu.execute(true);
        game_accumulate_step(result, fixture, run, fluid, box, trace);
    }
    result.final_com = game_center_of_mass(fluid);
    result.forward_com_travel = result.final_com.x - initial_com.x;
    std::tie(result.components, result.satellites) = game_topology(fluid);
    result.trace_sha256 = sha256_hex(trace.str());
    result.quality_passed = result.apparatus_passed
        && result.maximum_penetration <= 1.0e-12
        && result.maximum_fixed_displacement <= 1.0e-12
        && result.maximum_contact_velocity_error <= 1.0e-4
        && result.forward_com_travel >= 0.05
        && result.maximum_speed <= 3.0
        && result.components <= 2U && result.satellites <= 4U;
    if (!result.apparatus_passed) {
        result.first_failure = "apparatus";
    } else if (result.maximum_penetration > 1.0e-12) {
        result.first_failure = "containment";
    } else if (result.maximum_fixed_displacement > 1.0e-12) {
        result.first_failure = "fixed_boundary_moved";
    } else if (result.maximum_contact_velocity_error > 1.0e-4) {
        result.first_failure = "contact_velocity_correspondence";
    } else if (result.forward_com_travel < 0.05) {
        result.first_failure = "release_motion";
    } else if (result.maximum_speed > 3.0) {
        result.first_failure = "speed";
    } else if (result.components > 2U || result.satellites > 4U) {
        result.first_failure = "topology";
    }
    return result;
}

GameScenarioResult run_game_contact(const Profile& profile, int iterations) {
    const GameQualityBox box{{0.0, 0.0, 0.0}, {0.1, 0.1, 0.1}, {2, 2, 2}};
    const std::array<Vec3, 2> starts = {
        Vec3{GAME_RADIUS, 0.075, GAME_RADIUS},
        Vec3{0.075, 0.075, 0.075},
    };
    const std::array<Vec3, 2> velocities = {
        Vec3{0.0, -120.0, 0.0},
        Vec3{-120.0, -120.0, -120.0},
    };
    const std::array<std::vector<int>, 2> expected = {
        std::vector<int>{2},
        std::vector<int>{0, 2, 4},
    };
    GameScenarioResult result;
    result.id = "face_corner_contact";
    result.steps = 2;
    result.dynamic_samples = 1;
    result.components = 1;
    std::ostringstream trace;
    bool exact_features = true;
    for (std::size_t index = 0; index < starts.size(); ++index) {
        std::vector<Particle> fluid = {{starts[index], velocities[index], false}};
        const Fixture fixture = game_fixture(
            profile, "game-face-corner-contact", fluid, box, iterations);
        result.boundary_samples = fixture.particles.size() - fluid.size();
        CudaBaseline gpu(
            fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
            PairTraversalMode::FusedOwnerTermsP1,
            NeighborEncodingMode::CompactU16P2);
        const CapturedRun run = gpu.execute(true);
        const bool step_valid = game_finite(run, 1U)
            && run.state.next_position.size() == fixture.particles.size();
        result.apparatus_passed = result.apparatus_passed && step_valid;
        if (!step_valid) {
            trace << "INVALID_STEP|";
            continue;
        }
        const GameContactResult contact = game_sweep_box(
            starts[index], run.state.next_position.front(), box);
        exact_features = exact_features && contact.features == expected[index];
        result.maximum_penetration = std::max(
            result.maximum_penetration, contact.penetration);
        result.maximum_speed = std::max(result.maximum_speed, norm(contact.velocity));
        result.maximum_contact_velocity_error = std::max(
            result.maximum_contact_velocity_error,
            norm(run.state.final_velocity.front() - contact.velocity));
        result.solver_total_ms += run.timing.total;
        result.solver_maximum_ms = std::max(result.solver_maximum_ms, run.timing.total);
        for (std::size_t fixed = 1; fixed < fixture.particles.size(); ++fixed) {
            result.maximum_fixed_displacement = std::max(
                result.maximum_fixed_displacement,
                norm(run.state.next_position[fixed]
                    - game_uploaded_position(fixture.particles[fixed])));
        }
        trace << ordered_output_digest(run.state) << '|';
        for (int feature : contact.features) {
            trace << feature << ',';
        }
        trace << '|';
    }
    result.trace_sha256 = sha256_hex(trace.str());
    result.quality_passed = result.apparatus_passed && exact_features
        && result.maximum_penetration <= 1.0e-12
        && result.maximum_fixed_displacement <= 1.0e-12
        && result.maximum_contact_velocity_error <= 1.0e-4;
    if (!result.apparatus_passed) {
        result.first_failure = "apparatus";
    } else if (!exact_features) {
        result.first_failure = "contact_features";
    } else if (result.maximum_penetration > 1.0e-12) {
        result.first_failure = "containment";
    } else if (result.maximum_fixed_displacement > 1.0e-12) {
        result.first_failure = "fixed_boundary_moved";
    } else if (result.maximum_contact_velocity_error > 1.0e-4) {
        result.first_failure = "contact_velocity_correspondence";
    }
    return result;
}

void append_game_scenario(
    std::ostringstream& output,
    const GameScenarioResult& result) {
    output << std::setprecision(17)
           << "{\"id\":\"" << result.id << "\",\"apparatus_passed\":"
           << (result.apparatus_passed ? "true" : "false")
           << ",\"quality_passed\":"
           << (result.quality_passed ? "true" : "false")
           << ",\"first_failure\":\"" << result.first_failure << "\""
           << ",\"steps\":" << result.steps
           << ",\"dynamic_samples\":" << result.dynamic_samples
           << ",\"boundary_samples\":" << result.boundary_samples
           << ",\"components\":" << result.components
           << ",\"satellites\":" << result.satellites
           << ",\"maximum_penetration_m\":" << result.maximum_penetration
           << ",\"maximum_fixed_displacement_m\":"
           << result.maximum_fixed_displacement
           << ",\"maximum_contact_velocity_error_m_s\":"
           << result.maximum_contact_velocity_error
           << ",\"maximum_speed_m_s\":" << result.maximum_speed
           << ",\"maximum_positive_compression\":"
           << result.maximum_positive_compression
           << ",\"horizontal_com_drift_m\":" << result.horizontal_com_drift
           << ",\"vertical_com_drift_m\":" << result.vertical_com_drift
           << ",\"forward_com_travel_m\":" << result.forward_com_travel
           << ",\"final_com_m\":[" << result.final_com.x << ','
           << result.final_com.y << ',' << result.final_com.z << ']'
           << ",\"solver_total_ms\":" << result.solver_total_ms
           << ",\"solver_mean_ms\":"
           << result.solver_total_ms / static_cast<double>(result.steps)
           << ",\"solver_maximum_ms\":" << result.solver_maximum_ms
           << ",\"trace_sha256\":\"" << result.trace_sha256 << "\"}";
}

} // namespace

CommandReport run_cuda_game_quality_smoke() {
    const Profile& profile = find_profile("nuv-basin-48k-static-support-h3-physical.v4");
    constexpr std::array<int, 2> ITERATION_LANES = {5, 16};
    struct Lane {
        int iterations = 0;
        GameScenarioResult hold;
        GameScenarioResult release;
        GameScenarioResult contact;
        bool apparatus_passed = false;
        bool quality_passed = false;
    };
    std::array<Lane, ITERATION_LANES.size()> lanes;
    bool apparatus_passed = true;
    int selected_iterations = 0;
    std::ostringstream root;
    for (std::size_t index = 0; index < lanes.size(); ++index) {
        Lane& lane = lanes[index];
        lane.iterations = ITERATION_LANES[index];
        lane.hold = run_game_hold(profile, lane.iterations);
        lane.release = run_game_release(profile, lane.iterations);
        lane.contact = run_game_contact(profile, lane.iterations);
        lane.apparatus_passed = lane.hold.apparatus_passed
            && lane.release.apparatus_passed && lane.contact.apparatus_passed;
        lane.quality_passed = lane.hold.quality_passed
            && lane.release.quality_passed && lane.contact.quality_passed;
        apparatus_passed = apparatus_passed && lane.apparatus_passed;
        if (selected_iterations == 0 && lane.quality_passed) {
            selected_iterations = lane.iterations;
        }
        root << lane.iterations << '|' << lane.apparatus_passed << '|'
             << lane.quality_passed << '|' << lane.hold.trace_sha256 << '|'
             << lane.release.trace_sha256 << '|' << lane.contact.trace_sha256 << '|';
    }
    root << selected_iterations;
    const bool quality_passed = apparatus_passed && selected_iterations != 0;
    const char* semantic_status = !apparatus_passed ? "APPARATUS_INCONCLUSIVE"
        : (selected_iterations == 5 ? "ORIGINAL_WORK_GAME_QUALITY_CANDIDATE"
            : (selected_iterations == 16 ? "MORE_ITERATIONS_REQUIRED"
                                         : "GAME_QUALITY_REFUTED"));

    std::ostringstream output;
    output << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.cuda_game_quality_smoke.v1\""
           << ",\"status\":\"" << (quality_passed ? "PASS" : "FAIL") << "\""
           << ",\"apparatus_passed\":"
           << (apparatus_passed ? "true" : "false")
           << ",\"quality_passed\":" << (quality_passed ? "true" : "false")
           << ",\"semantic_status\":\"" << semantic_status
           << "\",\"backend_identity\":"
              "\"fused-owner-terms-p1+compact-csr-u16-p2\""
           << ",\"profile_id\":\"" << profile.id << "\""
           << ",\"profile_sha256\":\""
           << sha256_hex(canonical_profile_json(profile)) << "\""
           << ",\"binary_sha256\":\"" << executable_hash() << "\""
           << ",\"observer_timing_in_primary_gpu_step\":false"
           << ",\"contact_timing_in_primary_gpu_step\":true"
           << ",\"gates\":{\"hold_horizontal_com_drift_m\":0.005"
           << ",\"hold_vertical_com_drift_m\":0.025"
           << ",\"hold_maximum_speed_m_s\":1"
           << ",\"release_forward_com_travel_m\":0.05"
           << ",\"release_maximum_speed_m_s\":3"
           << ",\"maximum_penetration_m\":1e-12"
           << ",\"maximum_fixed_displacement_m\":1e-12"
           << ",\"maximum_contact_velocity_error_m_s\":1e-4}"
           << ",\"lanes\":[";
    for (std::size_t index = 0; index < lanes.size(); ++index) {
        if (index != 0U) {
            output << ',';
        }
        const Lane& lane = lanes[index];
        output << "{\"iterations\":" << lane.iterations
               << ",\"apparatus_passed\":"
               << (lane.apparatus_passed ? "true" : "false")
               << ",\"quality_passed\":"
               << (lane.quality_passed ? "true" : "false")
               << ",\"scenarios\":[";
        append_game_scenario(output, lane.hold);
        output << ',';
        append_game_scenario(output, lane.release);
        output << ',';
        append_game_scenario(output, lane.contact);
        output << "]}";
    }
    output << "],\"selection\":{\"selected_iterations\":"
           << selected_iterations
           << ",\"quality_passed\":"
           << (selected_iterations != 0 ? "true" : "false")
           << ",\"performance_campaign_authorized\":"
           << (quality_passed ? "true" : "false") << '}'
           << ",\"result_sha256\":\"" << sha256_hex(root.str()) << "\""
           << ",\"device\":" << device_json() << '}';
    return {quality_passed, output.str()};
}

namespace {

constexpr double GAME_VISUAL_PIXEL_PITCH = GAME_SPACING / 4.0;
constexpr std::array<int, 5> GAME_VISUAL_FRAME_STEPS = {0, 24, 48, 72, 96};

struct GameSurfaceFrame {
    bool valid = false;
    int step = 0;
    std::uint32_t width = 0;
    std::uint32_t height = 0;
    std::vector<std::uint8_t> wet;
    std::vector<double> depth;
    std::uint64_t wet_pixels = 0;
    std::uint32_t material_components = 0;
    double largest_component_fraction = 0.0;
    double satellite_area_fraction = 1.0;
    double depth_p50 = 0.0;
    double depth_p95 = 0.0;
    double minimum_depth = 0.0;
    double maximum_depth = 0.0;
    std::string root;
};

struct GameVisualLaneResult {
    std::string id;
    bool executed = false;
    bool apparatus_passed = false;
    bool quality_passed = false;
    int requested_steps = 96;
    int completed_steps = 0;
    std::size_t dynamic_samples = 0;
    std::size_t neighbor_capacity_per_sample = 160U;
    std::size_t maximum_directed_pairs = 0;
    std::size_t maximum_degree = 0;
    std::size_t neighbor_id_bytes = 0;
    double maximum_penetration = 0.0;
    double maximum_contact_velocity_error = 0.0;
    double maximum_speed = 0.0;
    double maximum_positive_compression = 0.0;
    double vertical_com_drop = 0.0;
    double front_advance = 0.0;
    double wet_area_ratio = 0.0;
    double depth_p95_drop = 0.0;
    std::size_t particle_components = 0;
    std::size_t particle_satellites = 0;
    double particle_largest_component_fraction = 0.0;
    std::vector<GameSurfaceFrame> frames;
    std::vector<double> total_timings;
    std::vector<double> contact_timings;
    std::string trace_root;
    std::string result_root;
    std::string first_failure;
    bool montage_written = false;
};

struct GameVisualObserverControls {
    bool empty_rejected = false;
    bool nonfinite_rejected = false;
    bool mask_mutation_changed_root = false;
    bool depth_mutation_changed_root = false;
    std::string root;
};

struct PresentationSurfaceFrame {
    bool valid = false;
    bool passed = false;
    int step = 0;
    std::uint32_t width = 0;
    std::uint32_t height = 0;
    std::vector<std::uint8_t> wet;
    std::vector<double> depth;
    std::uint64_t raw_wet_pixels = 0;
    /// NGQ9: raw wet pixels kept after the component policy.
    std::uint64_t retained_pixels = 0;
    /// NGQ9: raw components kept after the component policy.
    std::uint64_t retained_components = 0;
    std::uint64_t wet_pixels = 0;
    std::uint64_t common_pixels = 0;
    std::uint64_t filled_pixels = 0;
    std::uint64_t culled_pixels = 0;
    std::uint32_t components = 0;
    bool local_fill_only = false;
    std::array<std::uint32_t, 4> bounding_box_expansion_pixels{};
    double area_ratio = 0.0;
    double common_coverage = 0.0;
    double depth_rmse = 0.0;
    double depth_p95_change = 0.0;
    double maximum_depth_change = 0.0;
    double isotropic_depth_rmse = 0.0;
    double isotropic_depth_p95_change = 0.0;
    double isotropic_maximum_depth_change = 0.0;
    std::uint64_t mesh_vertices = 0;
    std::uint64_t mesh_triangles = 0;
    /// 0 = frozen NGQ5 sphere-cap heights; 1 = NGQ6 revision-2 heights: a
    /// 5x5 grayscale closing of the sphere-cap height over the unchanged
    /// sphere mask, before the unchanged NGQ5 pipeline.
    int model = 0;
    /// NGQ6 metrics: signed change against the raw sphere height on common
    /// pixels (median, p95, minimum) and the count of pixels above the local
    /// 5x5 raw maximum (structurally zero for a closing).
    double lift_p50 = 0.0;
    double lift_p95 = 0.0;
    double lift_min = 0.0;
    std::uint64_t ceiling_violations = 0;
    double extraction_ms = 0.0;
    std::string input_root;
    std::string root;
};

/// NGQ6 revision-2 presentation height source. Sphere-cap projection leaves
/// pits wherever the top layer's circles (radius `r` at spacing `2r`) do not
/// cover a pixel and a deeper particle shows through. A grayscale closing
/// (dilation then erosion over the raw wet mask) with a 5x5 element, i.e.
/// +-2 pixels = +-25 mm, fills pits narrower than the element and leaves
/// wider structures and fronts unchanged. `height` is the closed field and
/// `ceiling` the 5x5 local raw maximum, which bounds it structurally.
/// Revision 1 (a broad dome envelope of support `2 * spacing`) was refuted
/// by its own gate: it bridged real vertical gaps by up to `0.27 m`.
struct GameDomeField {
    bool valid = false;
    std::vector<double> height;
    std::vector<double> ceiling;
};

constexpr int GAME_CLOSING_RADIUS_PIXELS = 2;
constexpr double GAME_CLOSING_LIFT_P50_LIMIT = GAME_RADIUS;

struct PresentationSurfaceLane {
    std::string id;
    bool executed = false;
    bool passed = false;
    bool montage_written = false;
    bool mesh_written = false;
    double total_extraction_ms = 0.0;
    std::vector<PresentationSurfaceFrame> frames;
    std::string raw_trace_root;
    std::string raw_result_root;
    std::string result_root;
    std::string first_failure;
};

struct PresentationSurfaceControls {
    bool empty_rejected = false;
    bool nonfinite_rejected = false;
    bool mask_mutation_changed_root = false;
    bool depth_mutation_changed_root = false;
    std::string root;
};

double game_percentile(std::vector<double> values, double probability) {
    if (values.empty()) {
        return std::numeric_limits<double>::quiet_NaN();
    }
    std::sort(values.begin(), values.end());
    const std::size_t rank = static_cast<std::size_t>(
        std::ceil(probability * static_cast<double>(values.size())));
    return values[std::max<std::size_t>(1U, rank) - 1U];
}

std::string game_surface_root(const GameSurfaceFrame& frame) {
    std::ostringstream material;
    material << "nextengine.nonlocal.game-surface-frame.v1\n"
             << frame.valid << ':' << frame.step << ':' << frame.width << ':'
             << frame.height << ':' << frame.wet_pixels << ':'
             << frame.material_components << ':' << std::hexfloat
             << frame.largest_component_fraction << ':'
             << frame.satellite_area_fraction << ':' << frame.depth_p50 << ':'
             << frame.depth_p95 << ':' << frame.minimum_depth << ':'
             << frame.maximum_depth << '\n';
    for (std::size_t pixel = 0; pixel < frame.wet.size(); ++pixel) {
        material << static_cast<unsigned>(frame.wet[pixel]);
        if (frame.wet[pixel] != 0U) {
            material << ':' << frame.depth[pixel];
        }
        material << ';';
    }
    return sha256_hex(material.str());
}

GameSurfaceFrame game_surface_frame(
    const std::vector<Particle>& fluid,
    const GameQualityBox& box,
    int step) {
    GameSurfaceFrame frame;
    frame.step = step;
    if (fluid.empty()) {
        return frame;
    }
    const double extent_x = box.maximum.x - box.minimum.x;
    const double extent_z = box.maximum.z - box.minimum.z;
    const double cells_x = extent_x / GAME_VISUAL_PIXEL_PITCH;
    const double cells_z = extent_z / GAME_VISUAL_PIXEL_PITCH;
    frame.width = static_cast<std::uint32_t>(std::llround(cells_x));
    frame.height = static_cast<std::uint32_t>(std::llround(cells_z));
    if (frame.width == 0U || frame.height == 0U
        || std::abs(cells_x - static_cast<double>(frame.width)) > 1.0e-12
        || std::abs(cells_z - static_cast<double>(frame.height)) > 1.0e-12) {
        return frame;
    }
    const std::size_t pixels = static_cast<std::size_t>(frame.width) * frame.height;
    frame.wet.assign(pixels, 0U);
    frame.depth.assign(pixels, 0.0);
    const long double radius_squared =
        static_cast<long double>(GAME_RADIUS) * GAME_RADIUS;
    const float radius_f32 = static_cast<float>(GAME_RADIUS);
    const std::array<double, 3> lower_center = {
        static_cast<double>(static_cast<float>(box.minimum.x) + radius_f32),
        static_cast<double>(static_cast<float>(box.minimum.y) + radius_f32),
        static_cast<double>(static_cast<float>(box.minimum.z) + radius_f32),
    };
    const std::array<double, 3> upper_center = {
        static_cast<double>(static_cast<float>(box.maximum.x) - radius_f32),
        static_cast<double>(static_cast<float>(box.maximum.y) - radius_f32),
        static_cast<double>(static_cast<float>(box.maximum.z) - radius_f32),
    };
    const std::array<double, 3> observer_lower = {
        std::min(box.minimum.x + GAME_RADIUS, lower_center[0]),
        std::min(box.minimum.y + GAME_RADIUS, lower_center[1]),
        std::min(box.minimum.z + GAME_RADIUS, lower_center[2]),
    };
    const std::array<double, 3> observer_upper = {
        std::max(box.maximum.x - GAME_RADIUS, upper_center[0]),
        std::max(box.maximum.y - GAME_RADIUS, upper_center[1]),
        std::max(box.maximum.z - GAME_RADIUS, upper_center[2]),
    };
    for (const Particle& particle : fluid) {
        constexpr double observer_boundary_tolerance = 1.0e-12;
        if (!finite(particle.position)
            || particle.position.x
                < observer_lower[0] - observer_boundary_tolerance
            || particle.position.y
                < observer_lower[1] - observer_boundary_tolerance
            || particle.position.z
                < observer_lower[2] - observer_boundary_tolerance
            || particle.position.x
                > observer_upper[0] + observer_boundary_tolerance
            || particle.position.y
                > observer_upper[1] + observer_boundary_tolerance
            || particle.position.z
                > observer_upper[2] + observer_boundary_tolerance) {
            return frame;
        }
        const auto minimum_index = [](double value, double origin) {
            return static_cast<std::int64_t>(std::ceil(
                (value - GAME_RADIUS - origin) / GAME_VISUAL_PIXEL_PITCH - 0.5));
        };
        const auto maximum_index = [](double value, double origin) {
            return static_cast<std::int64_t>(std::floor(
                (value + GAME_RADIUS - origin) / GAME_VISUAL_PIXEL_PITCH - 0.5));
        };
        const std::int64_t min_x = std::max<std::int64_t>(
            0, minimum_index(particle.position.x, box.minimum.x));
        const std::int64_t max_x = std::min<std::int64_t>(
            frame.width - 1U, maximum_index(particle.position.x, box.minimum.x));
        const std::int64_t min_z = std::max<std::int64_t>(
            0, minimum_index(particle.position.z, box.minimum.z));
        const std::int64_t max_z = std::min<std::int64_t>(
            frame.height - 1U, maximum_index(particle.position.z, box.minimum.z));
        for (std::int64_t iz = min_z; iz <= max_z; ++iz) {
            for (std::int64_t ix = min_x; ix <= max_x; ++ix) {
                const long double pixel_x = static_cast<long double>(box.minimum.x)
                    + (static_cast<long double>(ix) + 0.5L)
                        * GAME_VISUAL_PIXEL_PITCH;
                const long double pixel_z = static_cast<long double>(box.minimum.z)
                    + (static_cast<long double>(iz) + 0.5L)
                        * GAME_VISUAL_PIXEL_PITCH;
                const long double dx =
                    static_cast<long double>(particle.position.x) - pixel_x;
                const long double dz =
                    static_cast<long double>(particle.position.z) - pixel_z;
                const long double distance_squared = dx * dx + dz * dz;
                if (distance_squared > radius_squared) {
                    continue;
                }
                const double depth = static_cast<double>(
                    static_cast<long double>(particle.position.y)
                    + std::sqrt(std::max(0.0L, radius_squared - distance_squared)));
                if (!std::isfinite(depth)) {
                    return frame;
                }
                const std::size_t pixel = static_cast<std::size_t>(iz) * frame.width
                    + static_cast<std::size_t>(ix);
                if (frame.wet[pixel] == 0U || depth > frame.depth[pixel]) {
                    frame.wet[pixel] = 1U;
                    frame.depth[pixel] = depth;
                }
            }
        }
    }
    frame.wet_pixels = static_cast<std::uint64_t>(std::count(
        frame.wet.begin(), frame.wet.end(), std::uint8_t{1U}));
    if (frame.wet_pixels == 0U) {
        return frame;
    }

    const std::uint32_t absent = std::numeric_limits<std::uint32_t>::max();
    std::vector<std::uint32_t> labels(pixels, absent);
    std::vector<std::uint32_t> stack;
    std::vector<std::uint32_t> component_sizes;
    for (std::uint32_t seed = 0; seed < pixels; ++seed) {
        if (frame.wet[seed] == 0U || labels[seed] != absent) {
            continue;
        }
        const std::uint32_t label =
            static_cast<std::uint32_t>(component_sizes.size());
        component_sizes.push_back(0U);
        labels[seed] = label;
        stack.push_back(seed);
        while (!stack.empty()) {
            const std::uint32_t pixel = stack.back();
            stack.pop_back();
            ++component_sizes[label];
            const std::int32_t x = static_cast<std::int32_t>(pixel % frame.width);
            const std::int32_t z = static_cast<std::int32_t>(pixel / frame.width);
            for (std::int32_t dz = -1; dz <= 1; ++dz) {
                for (std::int32_t dx = -1; dx <= 1; ++dx) {
                    if (dx == 0 && dz == 0) {
                        continue;
                    }
                    const std::int32_t nx = x + dx;
                    const std::int32_t nz = z + dz;
                    if (nx < 0 || nz < 0
                        || nx >= static_cast<std::int32_t>(frame.width)
                        || nz >= static_cast<std::int32_t>(frame.height)) {
                        continue;
                    }
                    const std::uint32_t neighbor =
                        static_cast<std::uint32_t>(nz) * frame.width
                        + static_cast<std::uint32_t>(nx);
                    if (frame.wet[neighbor] != 0U && labels[neighbor] == absent) {
                        labels[neighbor] = label;
                        stack.push_back(neighbor);
                    }
                }
            }
        }
    }
    std::uint32_t largest = 0U;
    for (std::uint32_t size : component_sizes) {
        if (size >= 4U) {
            ++frame.material_components;
        }
        largest = std::max(largest, size);
    }
    frame.largest_component_fraction = static_cast<double>(largest)
        / static_cast<double>(frame.wet_pixels);
    frame.satellite_area_fraction = 1.0 - frame.largest_component_fraction;
    std::vector<double> depths;
    depths.reserve(frame.wet_pixels);
    for (std::size_t pixel = 0; pixel < pixels; ++pixel) {
        if (frame.wet[pixel] != 0U) {
            depths.push_back(frame.depth[pixel]);
        }
    }
    frame.depth_p50 = game_percentile(depths, 0.50);
    frame.depth_p95 = game_percentile(depths, 0.95);
    const auto [minimum, maximum] = std::minmax_element(depths.begin(), depths.end());
    frame.minimum_depth = *minimum;
    frame.maximum_depth = *maximum;
    frame.valid = std::isfinite(frame.depth_p50) && std::isfinite(frame.depth_p95)
        && frame.minimum_depth >= box.minimum.y
        && frame.maximum_depth <= observer_upper[1] + GAME_RADIUS + 1.0e-12;
    frame.root = game_surface_root(frame);
    return frame;
}

std::tuple<std::size_t, std::size_t, double> game_csr_topology(
    const CapturedRun& run,
    std::size_t count) {
    if (run.offsets.size() != count + 1U || run.neighbors.empty()) {
        return {count, count, 0.0};
    }
    std::vector<std::uint8_t> visited(count, 0U);
    std::vector<std::size_t> stack;
    std::size_t components = 0U;
    std::size_t largest = 0U;
    for (std::size_t seed = 0; seed < count; ++seed) {
        if (visited[seed] != 0U) {
            continue;
        }
        ++components;
        std::size_t component_size = 0U;
        visited[seed] = 1U;
        stack.push_back(seed);
        while (!stack.empty()) {
            const std::size_t owner = stack.back();
            stack.pop_back();
            ++component_size;
            const int begin = run.offsets[owner];
            const int end = run.offsets[owner + 1U];
            if (begin < 0 || end < begin
                || static_cast<std::size_t>(end) > run.neighbors.size()) {
                return {count, count, 0.0};
            }
            for (int cursor = begin; cursor < end; ++cursor) {
                const int neighbor = run.neighbors[static_cast<std::size_t>(cursor)];
                if (neighbor < 0 || static_cast<std::size_t>(neighbor) >= count) {
                    return {count, count, 0.0};
                }
                if (visited[static_cast<std::size_t>(neighbor)] == 0U) {
                    visited[static_cast<std::size_t>(neighbor)] = 1U;
                    stack.push_back(static_cast<std::size_t>(neighbor));
                }
            }
        }
        largest = std::max(largest, component_size);
    }
    return {components, count - largest,
        static_cast<double>(largest) / static_cast<double>(count)};
}

bool game_frame_prefix_valid(const std::string& prefix) {
    return std::all_of(prefix.begin(), prefix.end(), [](unsigned char value) {
        return (value >= 'a' && value <= 'z') || (value >= 'A' && value <= 'Z')
            || (value >= '0' && value <= '9') || value == '/' || value == '.'
            || value == '_' || value == '-';
    });
}

bool write_game_surface_montage(
    const std::string& path,
    const std::vector<GameSurfaceFrame>& frames,
    const GameQualityBox& box) {
    if (frames.empty() || !frames.front().valid) {
        return false;
    }
    const std::uint32_t frame_width = frames.front().width;
    const std::uint32_t frame_height = frames.front().height;
    if (!std::all_of(frames.begin(), frames.end(), [&](const auto& frame) {
            return frame.valid && frame.width == frame_width
                && frame.height == frame_height;
        })) {
        return false;
    }
    std::ofstream stream(path, std::ios::binary);
    if (!stream) {
        return false;
    }
    const std::uint32_t width =
        frame_width * static_cast<std::uint32_t>(frames.size());
    stream << "P6\n" << width << ' ' << frame_height << "\n255\n";
    for (std::uint32_t row = 0; row < frame_height; ++row) {
        for (const GameSurfaceFrame& frame : frames) {
            for (std::uint32_t column = 0; column < frame_width; ++column) {
                const std::size_t pixel =
                    static_cast<std::size_t>(row) * frame_width + column;
                unsigned char red = 12U;
                unsigned char green = 18U;
                unsigned char blue = 28U;
                if (frame.wet[pixel] != 0U) {
                    const double normalized = std::clamp(
                        (frame.depth[pixel] - box.minimum.y)
                            / (box.maximum.y - box.minimum.y),
                        0.0, 1.0);
                    red = static_cast<unsigned char>(20.0 + 45.0 * normalized);
                    green = static_cast<unsigned char>(85.0 + 125.0 * normalized);
                    blue = static_cast<unsigned char>(160.0 + 95.0 * normalized);
                }
                stream.put(static_cast<char>(red));
                stream.put(static_cast<char>(green));
                stream.put(static_cast<char>(blue));
            }
        }
    }
    return static_cast<bool>(stream);
}

struct SurfaceMaskComponents {
    std::vector<std::uint32_t> labels;
    std::vector<std::uint32_t> sizes;
};

SurfaceMaskComponents label_surface_mask(
    const std::vector<std::uint8_t>& wet,
    std::uint32_t width,
    std::uint32_t height) {
    SurfaceMaskComponents result;
    const std::size_t pixels = static_cast<std::size_t>(width) * height;
    if (width == 0U || height == 0U || wet.size() != pixels) {
        return result;
    }
    const std::uint32_t absent = std::numeric_limits<std::uint32_t>::max();
    result.labels.assign(pixels, absent);
    std::vector<std::uint32_t> stack;
    for (std::uint32_t seed = 0; seed < pixels; ++seed) {
        if (wet[seed] == 0U || result.labels[seed] != absent) {
            continue;
        }
        const std::uint32_t label = static_cast<std::uint32_t>(result.sizes.size());
        result.sizes.push_back(0U);
        result.labels[seed] = label;
        stack.push_back(seed);
        while (!stack.empty()) {
            const std::uint32_t pixel = stack.back();
            stack.pop_back();
            ++result.sizes[label];
            const std::int32_t x = static_cast<std::int32_t>(pixel % width);
            const std::int32_t z = static_cast<std::int32_t>(pixel / width);
            for (std::int32_t dz = -1; dz <= 1; ++dz) {
                for (std::int32_t dx = -1; dx <= 1; ++dx) {
                    if (dx == 0 && dz == 0) {
                        continue;
                    }
                    const std::int32_t nx = x + dx;
                    const std::int32_t nz = z + dz;
                    if (nx < 0 || nz < 0 || nx >= static_cast<std::int32_t>(width)
                        || nz >= static_cast<std::int32_t>(height)) {
                        continue;
                    }
                    const std::uint32_t neighbor =
                        static_cast<std::uint32_t>(nz) * width
                        + static_cast<std::uint32_t>(nx);
                    if (wet[neighbor] != 0U && result.labels[neighbor] == absent) {
                        result.labels[neighbor] = label;
                        stack.push_back(neighbor);
                    }
                }
            }
        }
    }
    return result;
}

std::string presentation_surface_root(const PresentationSurfaceFrame& frame) {
    std::ostringstream material;
    if (frame.model != 0) {
        material << "nextengine.nonlocal.presentation-surface-frame-closing.v2\n"
                 << std::hexfloat << frame.lift_p50 << ':' << frame.lift_p95 << ':'
                 << frame.lift_min << ':' << std::defaultfloat
                 << frame.ceiling_violations << '\n';
    }
    material << "nextengine.nonlocal.presentation-surface-frame.v3\n"
             << frame.valid << ':' << frame.passed << ':' << frame.step << ':'
             << frame.width << ':' << frame.height << ':' << frame.raw_wet_pixels
             << ':' << frame.wet_pixels << ':' << frame.common_pixels << ':'
             << frame.filled_pixels << ':' << frame.culled_pixels << ':'
             << frame.components << ':' << frame.local_fill_only << ':'
             << frame.bounding_box_expansion_pixels[0] << ':'
             << frame.bounding_box_expansion_pixels[1] << ':'
             << frame.bounding_box_expansion_pixels[2] << ':'
             << frame.bounding_box_expansion_pixels[3] << ':' << std::hexfloat
             << frame.area_ratio << ':'
             << frame.common_coverage << ':' << frame.depth_rmse << ':'
             << frame.depth_p95_change << ':' << frame.maximum_depth_change << ':'
             << frame.isotropic_depth_rmse << ':'
             << frame.isotropic_depth_p95_change << ':'
             << frame.isotropic_maximum_depth_change << ':'
             << frame.mesh_vertices << ':' << frame.mesh_triangles << '\n'
             << frame.input_root << '\n';
    for (std::size_t pixel = 0; pixel < frame.wet.size(); ++pixel) {
        material << static_cast<unsigned>(frame.wet[pixel]);
        if (frame.wet[pixel] != 0U) {
            material << ':' << frame.depth[pixel];
        }
        material << ';';
    }
    return sha256_hex(material.str());
}

GameDomeField game_surface_dome_field(
    const std::vector<Particle>& fluid,
    const GameQualityBox& box,
    const GameSurfaceFrame& raw) {
    (void)fluid;
    (void)box;
    GameDomeField field;
    const std::size_t pixels = static_cast<std::size_t>(raw.width) * raw.height;
    if (!raw.valid || raw.wet.size() != pixels || raw.depth.size() != pixels) {
        return field;
    }
    const auto filter = [&](const std::vector<double>& input, bool maximum) {
        std::vector<double> output(pixels, 0.0);
        for (std::int32_t z = 0; z < static_cast<std::int32_t>(raw.height); ++z) {
            for (std::int32_t x = 0; x < static_cast<std::int32_t>(raw.width); ++x) {
                const std::size_t pixel = static_cast<std::size_t>(z) * raw.width
                    + static_cast<std::size_t>(x);
                if (raw.wet[pixel] == 0U) {
                    continue;
                }
                double value = input[pixel];
                for (std::int32_t dz = -GAME_CLOSING_RADIUS_PIXELS;
                     dz <= GAME_CLOSING_RADIUS_PIXELS; ++dz) {
                    for (std::int32_t dx = -GAME_CLOSING_RADIUS_PIXELS;
                         dx <= GAME_CLOSING_RADIUS_PIXELS; ++dx) {
                        const std::int32_t nx = x + dx;
                        const std::int32_t nz = z + dz;
                        if (nx < 0 || nz < 0 || nx >= static_cast<std::int32_t>(raw.width)
                            || nz >= static_cast<std::int32_t>(raw.height)) {
                            continue;
                        }
                        const std::size_t neighbor = static_cast<std::size_t>(nz)
                            * raw.width + static_cast<std::size_t>(nx);
                        if (raw.wet[neighbor] == 0U) {
                            continue;
                        }
                        value = maximum ? std::max(value, input[neighbor])
                                        : std::min(value, input[neighbor]);
                    }
                }
                output[pixel] = value;
            }
        }
        return output;
    };
    field.ceiling = filter(raw.depth, true);
    field.height = filter(field.ceiling, false);
    field.valid = true;
    return field;
}

// NGQ9: components with at least this many raw wet pixels survive under the
// `all` policy (one sphere-cap footprint is ~13 pixels at the 12.5 mm pitch).
constexpr std::uint32_t GAME_SURFACE_MIN_COMPONENT_PIXELS = 9U;

PresentationSurfaceFrame extract_presentation_surface(
    const GameSurfaceFrame& raw,
    const GameQualityBox& box,
    const GameDomeField* dome = nullptr,
    bool keep_all_components = false) {
    const auto begin = std::chrono::steady_clock::now();
    PresentationSurfaceFrame frame;
    frame.step = raw.step;
    frame.width = raw.width;
    frame.height = raw.height;
    frame.raw_wet_pixels = raw.wet_pixels;
    frame.input_root = raw.root;
    frame.model = dome != nullptr ? 1 : 0;
    const std::size_t pixels = static_cast<std::size_t>(raw.width) * raw.height;
    const bool dimensions_valid = raw.width != 0U && raw.height != 0U
        && raw.wet.size() == pixels && raw.depth.size() == pixels;
    bool input_valid = raw.valid && dimensions_valid && raw.wet_pixels != 0U
        && game_surface_root(raw) == raw.root
        && (dome == nullptr
            || (dome->valid && dome->height.size() == pixels && dome->ceiling.size() == pixels));
    const std::vector<double>& height_source = dome != nullptr ? dome->height : raw.depth;
    if (input_valid) {
        for (std::size_t pixel = 0; pixel < pixels; ++pixel) {
            input_valid = input_valid && (raw.wet[pixel] == 0U
                || (std::isfinite(raw.depth[pixel])
                    && raw.depth[pixel] >= box.minimum.y
                    && raw.depth[pixel] <= box.maximum.y + 1.0e-12));
        }
    }
    if (!input_valid) {
        frame.extraction_ms = std::chrono::duration<double, std::milli>(
            std::chrono::steady_clock::now() - begin).count();
        frame.root = presentation_surface_root(frame);
        return frame;
    }

    const SurfaceMaskComponents raw_components =
        label_surface_mask(raw.wet, raw.width, raw.height);
    if (raw_components.sizes.empty()) {
        frame.extraction_ms = std::chrono::duration<double, std::milli>(
            std::chrono::steady_clock::now() - begin).count();
        frame.root = presentation_surface_root(frame);
        return frame;
    }
    const std::uint32_t largest_label = static_cast<std::uint32_t>(std::distance(
        raw_components.sizes.begin(), std::max_element(
            raw_components.sizes.begin(), raw_components.sizes.end())));
    std::vector<std::uint8_t> retained(pixels, 0U);
    for (std::size_t pixel = 0; pixel < pixels; ++pixel) {
        const std::uint32_t label = raw_components.labels[pixel];
        const bool kept = keep_all_components
            ? (raw.wet[pixel] != 0U
                && raw_components.sizes[label] >= GAME_SURFACE_MIN_COMPONENT_PIXELS)
            : label == largest_label;
        retained[pixel] = static_cast<std::uint8_t>(kept);
        frame.retained_pixels += kept ? 1U : 0U;
    }
    frame.retained_components = keep_all_components
        ? static_cast<std::uint64_t>(std::count_if(
              raw_components.sizes.begin(), raw_components.sizes.end(),
              [](std::uint32_t size) { return size >= GAME_SURFACE_MIN_COMPONENT_PIXELS; }))
        : 1U;
    std::uint32_t raw_min_x = raw.width;
    std::uint32_t raw_max_x = 0U;
    std::uint32_t raw_min_z = raw.height;
    std::uint32_t raw_max_z = 0U;
    for (std::uint32_t z = 0; z < raw.height; ++z) {
        for (std::uint32_t x = 0; x < raw.width; ++x) {
            const std::size_t pixel = static_cast<std::size_t>(z) * raw.width + x;
            if (retained[pixel] != 0U) {
                raw_min_x = std::min(raw_min_x, x);
                raw_max_x = std::max(raw_max_x, x);
                raw_min_z = std::min(raw_min_z, z);
                raw_max_z = std::max(raw_max_z, z);
            }
        }
    }

    std::vector<std::uint8_t> dilated(pixels, 0U);
    for (std::int32_t z = 0; z < static_cast<std::int32_t>(raw.height); ++z) {
        for (std::int32_t x = 0; x < static_cast<std::int32_t>(raw.width); ++x) {
            bool any = false;
            for (std::int32_t dz = -1; dz <= 1; ++dz) {
                for (std::int32_t dx = -1; dx <= 1; ++dx) {
                    const std::int32_t nx = x + dx;
                    const std::int32_t nz = z + dz;
                    if (nx >= 0 && nz >= 0
                        && nx < static_cast<std::int32_t>(raw.width)
                        && nz < static_cast<std::int32_t>(raw.height)) {
                        const std::size_t neighbor = static_cast<std::size_t>(nz)
                            * raw.width + static_cast<std::size_t>(nx);
                        any = any || retained[neighbor] != 0U;
                    }
                }
            }
            dilated[static_cast<std::size_t>(z) * raw.width
                + static_cast<std::size_t>(x)] = static_cast<std::uint8_t>(any);
        }
    }
    frame.wet.assign(pixels, 0U);
    for (std::int32_t z = 0; z < static_cast<std::int32_t>(raw.height); ++z) {
        for (std::int32_t x = 0; x < static_cast<std::int32_t>(raw.width); ++x) {
            bool all = true;
            for (std::int32_t dz = -1; dz <= 1; ++dz) {
                for (std::int32_t dx = -1; dx <= 1; ++dx) {
                    const std::int32_t nx = x + dx;
                    const std::int32_t nz = z + dz;
                    if (nx < 0 || nz < 0 || nx >= static_cast<std::int32_t>(raw.width)
                        || nz >= static_cast<std::int32_t>(raw.height)) {
                        continue;
                    }
                    const std::size_t neighbor = static_cast<std::size_t>(nz)
                        * raw.width + static_cast<std::size_t>(nx);
                    all = all && dilated[neighbor] != 0U;
                }
            }
            frame.wet[static_cast<std::size_t>(z) * raw.width
                + static_cast<std::size_t>(x)] = static_cast<std::uint8_t>(all);
        }
    }

    std::vector<double> base_depth(pixels, 0.0);
    for (std::int32_t z = 0; z < static_cast<std::int32_t>(raw.height); ++z) {
        for (std::int32_t x = 0; x < static_cast<std::int32_t>(raw.width); ++x) {
            const std::size_t pixel = static_cast<std::size_t>(z) * raw.width
                + static_cast<std::size_t>(x);
            if (frame.wet[pixel] == 0U) {
                continue;
            }
            if (retained[pixel] != 0U) {
                base_depth[pixel] = height_source[pixel];
                continue;
            }
            double sum = 0.0;
            std::uint32_t count = 0U;
            for (std::int32_t dz = -1; dz <= 1; ++dz) {
                for (std::int32_t dx = -1; dx <= 1; ++dx) {
                    const std::int32_t nx = x + dx;
                    const std::int32_t nz = z + dz;
                    if (nx < 0 || nz < 0 || nx >= static_cast<std::int32_t>(raw.width)
                        || nz >= static_cast<std::int32_t>(raw.height)) {
                        continue;
                    }
                    const std::size_t neighbor = static_cast<std::size_t>(nz)
                        * raw.width + static_cast<std::size_t>(nx);
                    if (retained[neighbor] != 0U) {
                        sum += height_source[neighbor];
                        ++count;
                    }
                }
            }
            if (count == 0U) {
                frame.extraction_ms = std::chrono::duration<double, std::milli>(
                    std::chrono::steady_clock::now() - begin).count();
                frame.root = presentation_surface_root(frame);
                return frame;
            }
            base_depth[pixel] = sum / static_cast<double>(count);
        }
    }

    frame.depth.assign(pixels, 0.0);
    std::vector<double> isotropic_depth(pixels, 0.0);
    for (std::int32_t z = 0; z < static_cast<std::int32_t>(raw.height); ++z) {
        for (std::int32_t x = 0; x < static_cast<std::int32_t>(raw.width); ++x) {
            const std::size_t pixel = static_cast<std::size_t>(z) * raw.width
                + static_cast<std::size_t>(x);
            if (frame.wet[pixel] == 0U) {
                continue;
            }
            double weighted_sum = 0.0;
            double weight_sum = 0.0;
            double bilateral_sum = 0.0;
            double bilateral_weight_sum = 0.0;
            for (std::int32_t dz = -1; dz <= 1; ++dz) {
                for (std::int32_t dx = -1; dx <= 1; ++dx) {
                    const std::int32_t nx = x + dx;
                    const std::int32_t nz = z + dz;
                    if (nx < 0 || nz < 0 || nx >= static_cast<std::int32_t>(raw.width)
                        || nz >= static_cast<std::int32_t>(raw.height)) {
                        continue;
                    }
                    const std::size_t neighbor = static_cast<std::size_t>(nz)
                        * raw.width + static_cast<std::size_t>(nx);
                    if (frame.wet[neighbor] == 0U) {
                        continue;
                    }
                    const double weight = static_cast<double>(dx == 0 ? 2 : 1)
                        * static_cast<double>(dz == 0 ? 2 : 1);
                    weighted_sum += weight * base_depth[neighbor];
                    weight_sum += weight;
                    const double range = base_depth[neighbor] - base_depth[pixel];
                    const double range_weight = std::exp(
                        -0.5 * range * range / (GAME_RADIUS * GAME_RADIUS));
                    bilateral_sum += weight * range_weight * base_depth[neighbor];
                    bilateral_weight_sum += weight * range_weight;
                }
            }
            isotropic_depth[pixel] = std::clamp(
                weighted_sum / weight_sum, box.minimum.y, box.maximum.y);
            frame.depth[pixel] = std::clamp(bilateral_sum / bilateral_weight_sum,
                box.minimum.y, box.maximum.y);
        }
    }

    std::vector<double> depth_changes;
    std::vector<double> isotropic_depth_changes;
    std::vector<double> lifts;
    frame.local_fill_only = true;
    std::uint32_t surface_min_x = frame.width;
    std::uint32_t surface_max_x = 0U;
    std::uint32_t surface_min_z = frame.height;
    std::uint32_t surface_max_z = 0U;
    for (std::size_t pixel = 0; pixel < pixels; ++pixel) {
        const bool raw_wet = raw.wet[pixel] != 0U;
        const bool surface_wet = frame.wet[pixel] != 0U;
        frame.wet_pixels += static_cast<std::uint64_t>(surface_wet);
        frame.common_pixels += static_cast<std::uint64_t>(raw_wet && surface_wet);
        frame.filled_pixels += static_cast<std::uint64_t>(!raw_wet && surface_wet);
        frame.culled_pixels += static_cast<std::uint64_t>(raw_wet && !surface_wet);
        if (surface_wet) {
            const std::uint32_t x = static_cast<std::uint32_t>(pixel % frame.width);
            const std::uint32_t z = static_cast<std::uint32_t>(pixel / frame.width);
            surface_min_x = std::min(surface_min_x, x);
            surface_max_x = std::max(surface_max_x, x);
            surface_min_z = std::min(surface_min_z, z);
            surface_max_z = std::max(surface_max_z, z);
            if (retained[pixel] == 0U) {
                bool retained_neighbor = false;
                for (std::int32_t dz = -1; dz <= 1; ++dz) {
                    for (std::int32_t dx = -1; dx <= 1; ++dx) {
                        const std::int32_t nx = static_cast<std::int32_t>(x) + dx;
                        const std::int32_t nz = static_cast<std::int32_t>(z) + dz;
                        if (nx >= 0 && nz >= 0
                            && nx < static_cast<std::int32_t>(frame.width)
                            && nz < static_cast<std::int32_t>(frame.height)) {
                            const std::size_t neighbor = static_cast<std::size_t>(nz)
                                * frame.width + static_cast<std::size_t>(nx);
                            retained_neighbor = retained_neighbor
                                || retained[neighbor] != 0U;
                        }
                    }
                }
                frame.local_fill_only = frame.local_fill_only && retained_neighbor;
            }
        }
        if (raw_wet && surface_wet && dome != nullptr) {
            lifts.push_back(frame.depth[pixel] - raw.depth[pixel]);
            frame.ceiling_violations += static_cast<std::uint64_t>(
                frame.depth[pixel] > dome->ceiling[pixel] + 1.0e-9);
        }
        if (raw_wet && surface_wet) {
            const double change = std::abs(frame.depth[pixel] - raw.depth[pixel]);
            const double isotropic_change =
                std::abs(isotropic_depth[pixel] - raw.depth[pixel]);
            depth_changes.push_back(change);
            isotropic_depth_changes.push_back(isotropic_change);
            frame.depth_rmse += change * change;
            frame.maximum_depth_change = std::max(frame.maximum_depth_change, change);
            frame.isotropic_depth_rmse += isotropic_change * isotropic_change;
            frame.isotropic_maximum_depth_change = std::max(
                frame.isotropic_maximum_depth_change, isotropic_change);
        }
    }
    if (frame.raw_wet_pixels != 0U) {
        frame.area_ratio = static_cast<double>(frame.wet_pixels)
            / static_cast<double>(frame.raw_wet_pixels);
        frame.common_coverage = static_cast<double>(frame.common_pixels)
            / static_cast<double>(frame.raw_wet_pixels);
    }
    if (!depth_changes.empty()) {
        frame.depth_rmse = std::sqrt(
            frame.depth_rmse / static_cast<double>(depth_changes.size()));
        frame.depth_p95_change = game_percentile(depth_changes, 0.95);
        frame.isotropic_depth_rmse = std::sqrt(frame.isotropic_depth_rmse
            / static_cast<double>(isotropic_depth_changes.size()));
        frame.isotropic_depth_p95_change =
            game_percentile(isotropic_depth_changes, 0.95);
    }
    if (frame.wet_pixels != 0U) {
        frame.bounding_box_expansion_pixels = {
            raw_min_x > surface_min_x ? raw_min_x - surface_min_x : 0U,
            surface_max_x > raw_max_x ? surface_max_x - raw_max_x : 0U,
            raw_min_z > surface_min_z ? raw_min_z - surface_min_z : 0U,
            surface_max_z > raw_max_z ? surface_max_z - raw_max_z : 0U,
        };
    }
    frame.components = static_cast<std::uint32_t>(
        label_surface_mask(frame.wet, frame.width, frame.height).sizes.size());
    frame.mesh_vertices = frame.wet_pixels;
    if (frame.width > 1U && frame.height > 1U) {
        for (std::uint32_t z = 0; z + 1U < frame.height; ++z) {
            for (std::uint32_t x = 0; x + 1U < frame.width; ++x) {
                const std::size_t a = static_cast<std::size_t>(z) * frame.width + x;
                const std::size_t b = a + 1U;
                const std::size_t d = static_cast<std::size_t>(z + 1U)
                    * frame.width + x;
                const std::size_t c = d + 1U;
                if (frame.wet[a] != 0U && frame.wet[b] != 0U
                    && frame.wet[c] != 0U && frame.wet[d] != 0U) {
                    frame.mesh_triangles += 2U;
                }
            }
        }
    }
    if (!lifts.empty()) {
        frame.lift_p50 = game_percentile(lifts, 0.50);
        frame.lift_p95 = game_percentile(lifts, 0.95);
        frame.lift_min = *std::min_element(lifts.begin(), lifts.end());
    }
    frame.valid = frame.wet_pixels != 0U && frame.common_pixels != 0U
        && std::all_of(frame.depth.begin(), frame.depth.end(), [](double value) {
            return std::isfinite(value);
        });
    // NGQ9: under the `all` policy the closed mask may hold as many bodies
    // as the raw mask kept (bodies may merge under the close, never split).
    const bool mask_passed = frame.valid && frame.components >= 1U
        && frame.components <= std::max<std::uint64_t>(1U, frame.retained_components)
        && frame.local_fill_only
        && *std::max_element(frame.bounding_box_expansion_pixels.begin(),
               frame.bounding_box_expansion_pixels.end()) <= 1U
        && frame.area_ratio >= 0.95 && frame.area_ratio <= 1.40
        && frame.common_coverage >= 0.95 && frame.mesh_vertices != 0U
        && frame.mesh_triangles != 0U;
    frame.passed = dome == nullptr
        ? (mask_passed && frame.depth_rmse <= 0.025 && frame.depth_p95_change <= 0.050)
        : (mask_passed && frame.lift_p50 <= GAME_CLOSING_LIFT_P50_LIMIT
            && frame.ceiling_violations == 0U);
    frame.extraction_ms = std::chrono::duration<double, std::milli>(
        std::chrono::steady_clock::now() - begin).count();
    frame.root = presentation_surface_root(frame);
    return frame;
}

PresentationSurfaceControls presentation_surface_controls(
    const GameSurfaceFrame& raw,
    const GameQualityBox& box) {
    PresentationSurfaceControls controls;
    controls.empty_rejected = !extract_presentation_surface({}, box).valid;
    GameSurfaceFrame nonfinite = raw;
    const auto wet = std::find(nonfinite.wet.begin(), nonfinite.wet.end(), 1U);
    if (wet != nonfinite.wet.end()) {
        const std::size_t pixel = static_cast<std::size_t>(
            std::distance(nonfinite.wet.begin(), wet));
        nonfinite.depth[pixel] = std::numeric_limits<double>::infinity();
        nonfinite.root = game_surface_root(nonfinite);
        controls.nonfinite_rejected =
            !extract_presentation_surface(nonfinite, box).valid;
    }
    const PresentationSurfaceFrame base = extract_presentation_surface(raw, box);
    if (base.valid) {
        PresentationSurfaceFrame mask_mutation = base;
        const auto surface_wet =
            std::find(mask_mutation.wet.begin(), mask_mutation.wet.end(), 1U);
        if (surface_wet != mask_mutation.wet.end()) {
            *surface_wet = 0U;
            controls.mask_mutation_changed_root =
                presentation_surface_root(mask_mutation) != base.root;
        }
        PresentationSurfaceFrame depth_mutation = base;
        const auto depth_wet =
            std::find(depth_mutation.wet.begin(), depth_mutation.wet.end(), 1U);
        if (depth_wet != depth_mutation.wet.end()) {
            const std::size_t pixel = static_cast<std::size_t>(
                std::distance(depth_mutation.wet.begin(), depth_wet));
            depth_mutation.depth[pixel] = std::nextafter(
                depth_mutation.depth[pixel], std::numeric_limits<double>::infinity());
            controls.depth_mutation_changed_root =
                presentation_surface_root(depth_mutation) != base.root;
        }
    }
    std::ostringstream material;
    material << "nextengine.nonlocal.presentation-surface-controls.v3\n"
             << controls.empty_rejected << ':' << controls.nonfinite_rejected << ':'
             << controls.mask_mutation_changed_root << ':'
             << controls.depth_mutation_changed_root << '\n';
    controls.root = sha256_hex(material.str());
    return controls;
}

bool write_presentation_surface_montage(
    const std::string& path,
    const std::vector<PresentationSurfaceFrame>& frames,
    const GameQualityBox& box) {
    if (frames.empty() || !frames.front().valid) {
        return false;
    }
    const std::uint32_t frame_width = frames.front().width;
    const std::uint32_t frame_height = frames.front().height;
    if (!std::all_of(frames.begin(), frames.end(), [&](const auto& frame) {
            return frame.valid && frame.width == frame_width
                && frame.height == frame_height;
        })) {
        return false;
    }
    std::ofstream stream(path, std::ios::binary);
    if (!stream) {
        return false;
    }
    stream << "P6\n" << frame_width * frames.size() << ' ' << frame_height
           << "\n255\n";
    constexpr double light_x = -0.30151134457776363;
    constexpr double light_y = 0.90453403373329089;
    constexpr double light_z = -0.30151134457776363;
    for (std::uint32_t z = 0; z < frame_height; ++z) {
        for (const PresentationSurfaceFrame& frame : frames) {
            for (std::uint32_t x = 0; x < frame_width; ++x) {
                const std::size_t pixel = static_cast<std::size_t>(z) * frame_width + x;
                unsigned char red = 9U;
                unsigned char green = 16U;
                unsigned char blue = 27U;
                if (frame.wet[pixel] != 0U) {
                    const auto sample = [&](std::int32_t sx, std::int32_t sz) {
                        const std::int32_t cx = std::clamp<std::int32_t>(
                            sx, 0, static_cast<std::int32_t>(frame_width) - 1);
                        const std::int32_t cz = std::clamp<std::int32_t>(
                            sz, 0, static_cast<std::int32_t>(frame_height) - 1);
                        const std::size_t index = static_cast<std::size_t>(cz)
                            * frame_width + static_cast<std::size_t>(cx);
                        return frame.wet[index] != 0U
                            ? frame.depth[index] : frame.depth[pixel];
                    };
                    const double slope_x = (sample(static_cast<std::int32_t>(x) + 1,
                        static_cast<std::int32_t>(z))
                        - sample(static_cast<std::int32_t>(x) - 1,
                            static_cast<std::int32_t>(z)))
                        / (2.0 * GAME_VISUAL_PIXEL_PITCH);
                    const double slope_z = (sample(static_cast<std::int32_t>(x),
                        static_cast<std::int32_t>(z) + 1)
                        - sample(static_cast<std::int32_t>(x),
                            static_cast<std::int32_t>(z) - 1))
                        / (2.0 * GAME_VISUAL_PIXEL_PITCH);
                    const double normal_length =
                        std::sqrt(slope_x * slope_x + 1.0 + slope_z * slope_z);
                    const double diffuse = std::clamp(
                        (-slope_x * light_x + light_y - slope_z * light_z)
                            / normal_length,
                        0.22, 1.0);
                    const double height = std::clamp(
                        (frame.depth[pixel] - box.minimum.y)
                            / (box.maximum.y - box.minimum.y),
                        0.0, 1.0);
                    red = static_cast<unsigned char>(std::clamp(
                        (20.0 + 38.0 * height) * diffuse, 0.0, 255.0));
                    green = static_cast<unsigned char>(std::clamp(
                        (105.0 + 105.0 * height) * diffuse, 0.0, 255.0));
                    blue = static_cast<unsigned char>(std::clamp(
                        (185.0 + 65.0 * height) * diffuse, 0.0, 255.0));
                }
                stream.put(static_cast<char>(red));
                stream.put(static_cast<char>(green));
                stream.put(static_cast<char>(blue));
            }
        }
    }
    return static_cast<bool>(stream);
}

bool write_presentation_surface_obj(
    const std::string& path,
    const PresentationSurfaceFrame& frame,
    const GameQualityBox& box) {
    if (!frame.valid || frame.mesh_vertices == 0U || frame.mesh_triangles == 0U) {
        return false;
    }
    std::ofstream stream(path);
    if (!stream) {
        return false;
    }
    stream << std::setprecision(17) << "# NextEngine presentation-only water surface\n";
    std::vector<std::uint64_t> indices(frame.wet.size(), 0U);
    std::uint64_t vertex = 0U;
    for (std::uint32_t z = 0; z < frame.height; ++z) {
        for (std::uint32_t x = 0; x < frame.width; ++x) {
            const std::size_t pixel = static_cast<std::size_t>(z) * frame.width + x;
            if (frame.wet[pixel] == 0U) {
                continue;
            }
            indices[pixel] = ++vertex;
            const double world_x = box.minimum.x
                + (static_cast<double>(x) + 0.5) * GAME_VISUAL_PIXEL_PITCH;
            const double world_z = box.minimum.z
                + (static_cast<double>(z) + 0.5) * GAME_VISUAL_PIXEL_PITCH;
            stream << "v " << world_x << ' ' << frame.depth[pixel] << ' '
                   << world_z << '\n';
        }
    }
    std::uint64_t triangles = 0U;
    for (std::uint32_t z = 0; z + 1U < frame.height; ++z) {
        for (std::uint32_t x = 0; x + 1U < frame.width; ++x) {
            const std::size_t a = static_cast<std::size_t>(z) * frame.width + x;
            const std::size_t b = a + 1U;
            const std::size_t d = static_cast<std::size_t>(z + 1U) * frame.width + x;
            const std::size_t c = d + 1U;
            if (indices[a] != 0U && indices[b] != 0U && indices[c] != 0U
                && indices[d] != 0U) {
                stream << "f " << indices[a] << ' ' << indices[d] << ' '
                       << indices[c] << '\n';
                stream << "f " << indices[a] << ' ' << indices[c] << ' '
                       << indices[b] << '\n';
                triangles += 2U;
            }
        }
    }
    return static_cast<bool>(stream) && vertex == frame.mesh_vertices
        && triangles == frame.mesh_triangles;
}

PresentationSurfaceLane make_presentation_surface_lane(
    const GameVisualLaneResult& raw,
    const GameQualityBox& box,
    const std::string& frame_prefix) {
    PresentationSurfaceLane lane;
    lane.id = raw.id;
    lane.executed = raw.executed;
    lane.raw_trace_root = raw.trace_root;
    lane.raw_result_root = raw.result_root;
    if (!raw.executed || !raw.apparatus_passed || !raw.quality_passed) {
        lane.first_failure = "PARENT_LANE_NOT_ACCEPTED";
    } else {
        for (const GameSurfaceFrame& raw_frame : raw.frames) {
            PresentationSurfaceFrame frame =
                extract_presentation_surface(raw_frame, box);
            lane.total_extraction_ms += frame.extraction_ms;
            lane.frames.push_back(std::move(frame));
            if (!lane.frames.back().passed) {
                lane.first_failure = "FRAME_" + std::to_string(raw_frame.step);
                break;
            }
        }
        lane.passed = lane.frames.size() == GAME_VISUAL_FRAME_STEPS.size()
            && std::all_of(lane.frames.begin(), lane.frames.end(), [](const auto& frame) {
                return frame.passed;
            });
        if (lane.passed && !frame_prefix.empty()) {
            lane.montage_written = write_presentation_surface_montage(
                frame_prefix + "-" + lane.id + "-surface.ppm", lane.frames, box);
            lane.mesh_written = write_presentation_surface_obj(
                frame_prefix + "-" + lane.id + "-surface.obj", lane.frames.back(), box);
            // Every accepted keyframe is also exported so the engine-side
            // presentation bridge can replay a bounded dynamic sequence.
            for (const PresentationSurfaceFrame& frame : lane.frames) {
                lane.mesh_written = lane.mesh_written
                    && write_presentation_surface_obj(
                        frame_prefix + "-" + lane.id + "-step"
                            + std::to_string(frame.step) + "-surface.obj",
                        frame, box);
            }
            if (!lane.montage_written || !lane.mesh_written) {
                lane.passed = false;
                lane.first_failure = "FILE_OUTPUT";
            }
        }
    }
    std::ostringstream material;
    material << "nextengine.nonlocal.presentation-surface-lane.v3\n"
             << lane.id << ':' << lane.executed << ':' << lane.passed << '\n'
             << lane.raw_trace_root << '\n' << lane.raw_result_root << '\n';
    for (const PresentationSurfaceFrame& frame : lane.frames) {
        material << frame.root << '\n';
    }
    lane.result_root = sha256_hex(material.str());
    return lane;
}

std::vector<Particle> game_visual_particles(int lattice_x, int lattice_z, int lattice_y) {
    std::vector<Particle> fluid;
    fluid.reserve(static_cast<std::size_t>(lattice_x) * static_cast<std::size_t>(lattice_y)
        * lattice_z);
    for (int x = 0; x < lattice_x; ++x) {
        for (int y = 0; y < lattice_y; ++y) {
            for (int z = 0; z < lattice_z; ++z) {
                fluid.push_back({
                    {GAME_RADIUS + x * GAME_SPACING,
                        GAME_RADIUS + 2.0 * GAME_SPACING + y * GAME_SPACING,
                        GAME_RADIUS + z * GAME_SPACING},
                    {},
                    false,
                });
            }
        }
    }
    return fluid;
}

std::vector<Particle> game_visual_particles(int lattice_x, int lattice_z) {
    return game_visual_particles(lattice_x, lattice_z, 10);
}

GameVisualObserverControls game_visual_observer_controls(
    const std::vector<Particle>& initial,
    const GameQualityBox& box) {
    GameVisualObserverControls controls;
    const GameSurfaceFrame base = game_surface_frame(initial, box, 0);
    controls.empty_rejected = !game_surface_frame({}, box, 0).valid;
    std::vector<Particle> nonfinite = initial;
    nonfinite.front().position.y = std::numeric_limits<double>::quiet_NaN();
    controls.nonfinite_rejected = !game_surface_frame(nonfinite, box, 0).valid;
    if (base.valid) {
        GameSurfaceFrame mask_mutation = base;
        const auto wet = std::find(mask_mutation.wet.begin(), mask_mutation.wet.end(),
            std::uint8_t{1U});
        if (wet != mask_mutation.wet.end()) {
            *wet = 0U;
            controls.mask_mutation_changed_root =
                game_surface_root(mask_mutation) != base.root;
        }
        GameSurfaceFrame depth_mutation = base;
        const auto depth_pixel = std::find(depth_mutation.wet.begin(),
            depth_mutation.wet.end(), std::uint8_t{1U});
        if (depth_pixel != depth_mutation.wet.end()) {
            const std::size_t index = static_cast<std::size_t>(
                std::distance(depth_mutation.wet.begin(), depth_pixel));
            depth_mutation.depth[index] = std::nextafter(
                depth_mutation.depth[index], std::numeric_limits<double>::infinity());
            controls.depth_mutation_changed_root =
                game_surface_root(depth_mutation) != base.root;
        }
    }
    std::ostringstream material;
    material << "nextengine.nonlocal.game-surface-controls.v1\n"
             << controls.empty_rejected << ':' << controls.nonfinite_rejected << ':'
             << controls.mask_mutation_changed_root << ':'
             << controls.depth_mutation_changed_root << '\n';
    controls.root = sha256_hex(material.str());
    return controls;
}

GameVisualLaneResult run_game_visual_lane(
    const Profile& profile,
    const std::string& id,
    int lattice_x,
    int lattice_z,
    const GameQualityBox& box,
    const std::string& frame_prefix) {
    GameVisualLaneResult result;
    result.id = id;
    result.executed = true;
    result.apparatus_passed = true;
    result.neighbor_capacity_per_sample = profile.max_neighbors;
    std::vector<Particle> fluid = game_visual_particles(lattice_x, lattice_z);
    result.dynamic_samples = fluid.size();
    const Vec3 initial_com = game_center_of_mass(fluid);
    const double initial_front = std::max_element(fluid.begin(), fluid.end(),
        [](const Particle& lhs, const Particle& rhs) {
            return lhs.position.x < rhs.position.x;
        })->position.x;
    result.frames.push_back(game_surface_frame(fluid, box, 0));
    result.apparatus_passed = result.frames.back().valid;
    std::ostringstream trace;
    CapturedRun final_run;
    std::size_t next_frame = 1U;
    for (int step = 1; step <= result.requested_steps && result.apparatus_passed;
         ++step) {
        const Fixture fixture = game_fixture(
            profile, "game-visual-" + id, fluid, box, profile.fixed_iterations,
            result.neighbor_capacity_per_sample);
        CudaBaseline gpu(
            fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
            PairTraversalMode::FusedOwnerTermsP1,
            NeighborEncodingMode::CompactU16P2);
        const CapturedRun run = gpu.execute(true);
        result.maximum_directed_pairs = std::max(
            result.maximum_directed_pairs, run.state.directed_pairs);
        result.maximum_degree = std::max(result.maximum_degree, run.state.maximum_degree);
        result.neighbor_id_bytes = run.neighbor_id_bytes;
        const bool finite_state_passed = game_finite(run, fluid.size());
        const bool output_size_passed =
            run.state.next_position.size() == fluid.size();
        const bool compact_ids_passed =
            run.neighbor_id_bytes == sizeof(std::uint16_t);
        const bool pair_capacity_passed =
            run.state.directed_pairs <= fixture.pair_capacity;
        const bool degree_capacity_passed =
            run.state.maximum_degree <= result.neighbor_capacity_per_sample;
        const bool step_valid = finite_state_passed && output_size_passed
            && compact_ids_passed && pair_capacity_passed && degree_capacity_passed;
        if (!step_valid) {
            result.apparatus_passed = false;
            if (run.local_solve_failed) {
                result.first_failure = "local_solve";
            } else if (!run.neighbor_build_valid) {
                result.first_failure = "neighbor_build";
            } else if (!finite_state_passed) {
                result.first_failure = "nonfinite_state";
            } else if (!output_size_passed) {
                result.first_failure = "output_size";
            } else if (!compact_ids_passed) {
                result.first_failure = "compact_neighbor_ids";
            } else if (!pair_capacity_passed) {
                result.first_failure = "directed_pair_capacity";
            } else if (!degree_capacity_passed) {
                result.first_failure = "neighbor_degree_capacity";
            } else {
                result.first_failure = "step_apparatus";
            }
            trace << step << ":INVALID:" << result.first_failure << ':'
                  << run.neighbor_build_valid << ':' << run.local_solve_failed
                  << ':' << run.state.directed_pairs << ':'
                  << run.state.maximum_degree << '|';
            break;
        }
        result.total_timings.push_back(run.timing.total);
        result.contact_timings.push_back(run.timing.contact);
        for (std::size_t index = 0; index < fluid.size(); ++index) {
            const GameContactResult contact = game_sweep_box(
                fluid[index].position, run.state.next_position[index], box);
            result.maximum_penetration = std::max(
                result.maximum_penetration, contact.penetration);
            result.maximum_contact_velocity_error = std::max(
                result.maximum_contact_velocity_error,
                norm(run.state.final_velocity[index] - contact.velocity));
            result.maximum_speed = std::max(
                result.maximum_speed, norm(run.state.final_velocity[index]));
            result.maximum_positive_compression = std::max(
                result.maximum_positive_compression,
                std::max(run.state.density[index] / fixture.rest_density - 1.0, 0.0));
            fluid[index].position = run.state.next_position[index];
            fluid[index].velocity = run.state.final_velocity[index];
        }
        ++result.completed_steps;
        trace << step << ':' << ordered_output_digest(run.state) << '|';
        final_run = run;
        if (next_frame < GAME_VISUAL_FRAME_STEPS.size()
            && step == GAME_VISUAL_FRAME_STEPS[next_frame]) {
            result.frames.push_back(game_surface_frame(fluid, box, step));
            if (!result.frames.back().valid) {
                result.apparatus_passed = false;
                result.first_failure = "surface_observer";
                break;
            }
            ++next_frame;
        }
    }
    result.trace_root = sha256_hex(trace.str());
    if (result.apparatus_passed) {
        result.apparatus_passed = result.completed_steps == result.requested_steps
            && result.frames.size() == GAME_VISUAL_FRAME_STEPS.size()
            && result.maximum_penetration <= 1.0e-12
            && result.maximum_contact_velocity_error <= 1.0e-4;
        if (!result.apparatus_passed && result.first_failure.empty()) {
            result.first_failure = "apparatus_closure";
        }
    }
    if (result.completed_steps != 0) {
        const Vec3 final_com = game_center_of_mass(fluid);
        result.vertical_com_drop = initial_com.y - final_com.y;
        const double final_front = std::max_element(fluid.begin(), fluid.end(),
            [](const Particle& lhs, const Particle& rhs) {
                return lhs.position.x < rhs.position.x;
            })->position.x;
        result.front_advance = final_front - initial_front;
        std::tie(result.particle_components, result.particle_satellites,
            result.particle_largest_component_fraction) =
            game_csr_topology(final_run, fluid.size());
    }
    if (result.frames.size() == GAME_VISUAL_FRAME_STEPS.size()) {
        result.wet_area_ratio = static_cast<double>(result.frames.back().wet_pixels)
            / static_cast<double>(result.frames.front().wet_pixels);
        result.depth_p95_drop =
            result.frames.front().depth_p95 - result.frames.back().depth_p95;
    }
    bool frames_coherent = result.frames.size() == GAME_VISUAL_FRAME_STEPS.size();
    for (const GameSurfaceFrame& frame : result.frames) {
        frames_coherent = frames_coherent && frame.valid
            && frame.largest_component_fraction >= 0.98
            && frame.satellite_area_fraction <= 0.02
            && frame.material_components <= 4U;
    }
    const double particle_satellite_fraction = result.dynamic_samples == 0U
        ? 1.0
        : static_cast<double>(result.particle_satellites)
            / static_cast<double>(result.dynamic_samples);
    result.quality_passed = result.apparatus_passed
        && result.maximum_speed <= 10.0
        && result.vertical_com_drop >= 0.075
        && result.front_advance >= 0.10
        && result.wet_area_ratio >= 1.05 && result.wet_area_ratio <= 3.0
        && frames_coherent
        && result.particle_largest_component_fraction >= 0.98
        && particle_satellite_fraction <= 0.02
        && result.depth_p95_drop >= 0.025;
    if (result.apparatus_passed && !result.quality_passed) {
        if (result.maximum_speed > 10.0) {
            result.first_failure = "speed";
        } else if (result.vertical_com_drop < 0.075) {
            result.first_failure = "vertical_motion";
        } else if (result.front_advance < 0.10) {
            result.first_failure = "front_advance";
        } else if (result.wet_area_ratio < 1.05 || result.wet_area_ratio > 3.0) {
            result.first_failure = "wet_area";
        } else if (!frames_coherent) {
            result.first_failure = "surface_topology";
        } else if (result.particle_largest_component_fraction < 0.98
            || particle_satellite_fraction > 0.02) {
            result.first_failure = "particle_topology";
        } else if (result.depth_p95_drop < 0.025) {
            result.first_failure = "surface_settling";
        }
    }
    if (!frame_prefix.empty()) {
        result.montage_written = write_game_surface_montage(
            frame_prefix + "-" + result.id + ".ppm", result.frames, box);
    }
    std::ostringstream root;
    root << "nextengine.nonlocal.game-visual-lane.v1\n" << result.id << ':'
         << result.executed << ':' << result.apparatus_passed << ':'
         << result.quality_passed << ':' << result.completed_steps << ':'
         << result.dynamic_samples << ':' << result.neighbor_capacity_per_sample
         << ':' << result.maximum_directed_pairs << ':'
         << result.maximum_degree << ':' << result.neighbor_id_bytes << ':'
         << std::hexfloat << result.maximum_penetration << ':'
         << result.maximum_contact_velocity_error << ':' << result.maximum_speed << ':'
         << result.maximum_positive_compression << ':' << result.vertical_com_drop << ':'
         << result.front_advance << ':' << result.wet_area_ratio << ':'
         << result.depth_p95_drop << ':' << result.particle_components << ':'
         << result.particle_satellites << ':'
         << result.particle_largest_component_fraction << '\n'
         << result.trace_root << '\n';
    for (const GameSurfaceFrame& frame : result.frames) {
        root << frame.root << '\n';
    }
    result.result_root = sha256_hex(root.str());
    return result;
}

void append_game_surface_frame(
    std::ostringstream& output,
    const GameSurfaceFrame& frame) {
    output << std::setprecision(17)
           << "{\"valid\":" << (frame.valid ? "true" : "false")
           << ",\"step\":" << frame.step << ",\"width\":" << frame.width
           << ",\"height\":" << frame.height
           << ",\"wet_pixels\":" << frame.wet_pixels
           << ",\"material_components\":" << frame.material_components
           << ",\"largest_component_fraction\":"
           << frame.largest_component_fraction
           << ",\"satellite_area_fraction\":" << frame.satellite_area_fraction
           << ",\"depth_p50_m\":" << frame.depth_p50
           << ",\"depth_p95_m\":" << frame.depth_p95
           << ",\"minimum_depth_m\":" << frame.minimum_depth
           << ",\"maximum_depth_m\":" << frame.maximum_depth
           << ",\"frame_root\":\"" << frame.root << "\"}";
}

void append_game_visual_lane(
    std::ostringstream& output,
    const GameVisualLaneResult& lane) {
    output << std::setprecision(17)
           << "{\"id\":\"" << lane.id << "\",\"executed\":"
           << (lane.executed ? "true" : "false")
           << ",\"apparatus_passed\":"
           << (lane.apparatus_passed ? "true" : "false")
           << ",\"quality_passed\":"
           << (lane.quality_passed ? "true" : "false")
           << ",\"first_failure\":\"" << lane.first_failure << "\""
           << ",\"requested_steps\":" << lane.requested_steps
           << ",\"completed_steps\":" << lane.completed_steps
           << ",\"dynamic_samples\":" << lane.dynamic_samples
           << ",\"neighbor_capacity_per_sample\":"
           << lane.neighbor_capacity_per_sample
           << ",\"maximum_directed_pairs\":" << lane.maximum_directed_pairs
           << ",\"maximum_degree\":" << lane.maximum_degree
           << ",\"neighbor_id_bytes\":" << lane.neighbor_id_bytes
           << ",\"maximum_penetration_m\":" << lane.maximum_penetration
           << ",\"maximum_contact_velocity_error_m_s\":"
           << lane.maximum_contact_velocity_error
           << ",\"maximum_speed_m_s\":" << lane.maximum_speed
           << ",\"maximum_positive_compression\":"
           << lane.maximum_positive_compression
           << ",\"vertical_com_drop_m\":" << lane.vertical_com_drop
           << ",\"front_advance_m\":" << lane.front_advance
           << ",\"wet_area_ratio\":" << lane.wet_area_ratio
           << ",\"depth_p95_drop_m\":" << lane.depth_p95_drop
           << ",\"particle_components\":" << lane.particle_components
           << ",\"particle_satellites\":" << lane.particle_satellites
           << ",\"particle_largest_component_fraction\":"
           << lane.particle_largest_component_fraction
           << ",\"trace_root\":\"" << lane.trace_root << "\""
           << ",\"result_root\":\"" << lane.result_root << "\""
           << ",\"montage_written\":"
           << (lane.montage_written ? "true" : "false")
           << ",\"gpu_timing\":";
    if (lane.total_timings.empty()) {
        output << "null";
    } else {
        const Statistics total = statistics(lane.total_timings);
        const Statistics contact = statistics(lane.contact_timings);
        output << "{\"total\":";
        append_statistics(output, total);
        output << ",\"analytic_contact\":";
        append_statistics(output, contact);
        output << '}';
    }
    output << ",\"frames\":[";
    for (std::size_t index = 0; index < lane.frames.size(); ++index) {
        if (index != 0U) {
            output << ',';
        }
        append_game_surface_frame(output, lane.frames[index]);
    }
    output << "]}";
}

void append_presentation_surface_frame(
    std::ostringstream& output,
    const PresentationSurfaceFrame& frame) {
    output << std::setprecision(17)
           << "{\"valid\":" << (frame.valid ? "true" : "false")
           << ",\"passed\":" << (frame.passed ? "true" : "false")
           << ",\"step\":" << frame.step << ",\"width\":" << frame.width
           << ",\"height\":" << frame.height
           << ",\"raw_wet_pixels\":" << frame.raw_wet_pixels
           << ",\"presentation_wet_pixels\":" << frame.wet_pixels
           << ",\"common_pixels\":" << frame.common_pixels
           << ",\"filled_pixels\":" << frame.filled_pixels
           << ",\"culled_pixels\":" << frame.culled_pixels
           << ",\"components\":" << frame.components
           << ",\"local_fill_only\":"
           << (frame.local_fill_only ? "true" : "false")
           << ",\"bounding_box_expansion_pixels\":["
           << frame.bounding_box_expansion_pixels[0] << ','
           << frame.bounding_box_expansion_pixels[1] << ','
           << frame.bounding_box_expansion_pixels[2] << ','
           << frame.bounding_box_expansion_pixels[3] << ']'
           << ",\"area_ratio\":" << frame.area_ratio
           << ",\"common_coverage\":" << frame.common_coverage
           << ",\"depth_rmse_m\":" << frame.depth_rmse
           << ",\"depth_p95_change_m\":" << frame.depth_p95_change
           << ",\"maximum_depth_change_m\":" << frame.maximum_depth_change
           << ",\"isotropic_control\":{\"depth_rmse_m\":"
           << frame.isotropic_depth_rmse
           << ",\"depth_p95_change_m\":" << frame.isotropic_depth_p95_change
           << ",\"maximum_depth_change_m\":"
           << frame.isotropic_maximum_depth_change << '}'
           << ",\"mesh_vertices\":" << frame.mesh_vertices
           << ",\"mesh_triangles\":" << frame.mesh_triangles
           << ",\"extraction_ms\":" << frame.extraction_ms
           << ",\"input_root\":\"" << frame.input_root << "\""
           << ",\"surface_root\":\"" << frame.root << "\"}";
}

void append_presentation_surface_lane(
    std::ostringstream& output,
    const PresentationSurfaceLane& lane) {
    output << std::setprecision(17)
           << "{\"id\":\"" << lane.id << "\",\"executed\":"
           << (lane.executed ? "true" : "false")
           << ",\"passed\":" << (lane.passed ? "true" : "false")
           << ",\"first_failure\":\"" << lane.first_failure << "\""
           << ",\"raw_trace_root\":\"" << lane.raw_trace_root << "\""
           << ",\"raw_result_root\":\"" << lane.raw_result_root << "\""
           << ",\"surface_result_root\":\"" << lane.result_root << "\""
           << ",\"total_extraction_ms\":" << lane.total_extraction_ms
           << ",\"montage_written\":"
           << (lane.montage_written ? "true" : "false")
           << ",\"mesh_written\":" << (lane.mesh_written ? "true" : "false")
           << ",\"frames\":[";
    for (std::size_t index = 0; index < lane.frames.size(); ++index) {
        if (index != 0U) {
            output << ',';
        }
        append_presentation_surface_frame(output, lane.frames[index]);
    }
    output << "]}";
}

} // namespace

CommandReport run_cuda_game_visual_corpus(const std::string& frame_prefix) {
    if (!game_frame_prefix_valid(frame_prefix)) {
        throw std::invalid_argument("frame prefix contains unsupported characters");
    }
    const Profile& profile =
        find_profile("nuv-basin-48k-analytic-contact-game-cap160.v6");
    const GameQualityBox box4k{
        {0.0, 0.0, 0.0}, {2.0, 0.75, 1.0}, {40, 15, 20}};
    const GameQualityBox box16k{
        {0.0, 0.0, 0.0}, {4.0, 0.75, 2.0}, {80, 15, 40}};
    const std::vector<Particle> control_particles = game_visual_particles(20, 20);
    const GameVisualObserverControls controls =
        game_visual_observer_controls(control_particles, box4k);
    const bool controls_passed = controls.empty_rejected
        && controls.nonfinite_rejected && controls.mask_mutation_changed_root
        && controls.depth_mutation_changed_root;
    GameVisualLaneResult lane4k = run_game_visual_lane(
        profile, "falling-dam-4k", 20, 20, box4k, frame_prefix);
    GameVisualLaneResult lane16k;
    lane16k.id = "falling-dam-16k";
    lane16k.first_failure = "NOT_RUN_4K_REJECTED";
    if (controls_passed && lane4k.apparatus_passed && lane4k.quality_passed) {
        lane16k = run_game_visual_lane(
            profile, "falling-dam-16k", 40, 40, box16k, frame_prefix);
    }
    const bool apparatus_passed = controls_passed && lane4k.apparatus_passed
        && (!lane16k.executed || lane16k.apparatus_passed);
    const bool quality_passed = apparatus_passed && lane4k.quality_passed
        && lane16k.quality_passed;
    const char* semantic_status = !controls_passed ? "APPARATUS_INCONCLUSIVE"
        : (!lane4k.apparatus_passed ? "APPARATUS_INCONCLUSIVE"
            : (!lane4k.quality_passed ? "DYNAMIC_VISUAL_4K_REFUTED"
                : (!lane16k.apparatus_passed ? "APPARATUS_INCONCLUSIVE"
                    : (!lane16k.quality_passed ? "DYNAMIC_VISUAL_16K_REFUTED"
                        : "ORIGINAL_GPU_DYNAMIC_VISUAL_SUPPORTED_BOUNDED"))));
    std::ostringstream root;
    root << "nextengine.nonlocal.game-visual-corpus.v1\n"
         << controls.root << '\n' << lane4k.result_root << '\n'
         << lane16k.result_root << '\n' << semantic_status << '\n';
    const std::string result_root = sha256_hex(root.str());

    std::ostringstream output;
    output << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.cuda_game_visual_corpus.v1\""
           << ",\"status\":\"" << (quality_passed ? "PASS" : "FAIL") << "\""
           << ",\"semantic_status\":\"" << semantic_status << "\""
           << ",\"apparatus_passed\":"
           << (apparatus_passed ? "true" : "false")
           << ",\"quality_passed\":" << (quality_passed ? "true" : "false")
           << ",\"claim_ceiling\":\"finite_game_quality_tool_only\""
           << ",\"backend_identity\":"
              "\"fused-owner-terms-p1+compact-csr-u16-p2\""
           << ",\"profile_id\":\"" << profile.id << "\""
           << ",\"profile_sha256\":\""
           << sha256_hex(canonical_profile_json(profile)) << "\""
           << ",\"binary_sha256\":\"" << executable_hash() << "\""
           << ",\"observer_timing_in_primary_gpu_step\":false"
           << ",\"contact_timing_in_primary_gpu_step\":true"
           << ",\"gates\":{\"maximum_speed_m_s\":10"
           << ",\"minimum_vertical_com_drop_m\":0.075"
           << ",\"minimum_front_advance_m\":0.10"
           << ",\"wet_area_ratio_minimum\":1.05"
           << ",\"wet_area_ratio_maximum\":3.0"
           << ",\"minimum_largest_component_fraction\":0.98"
           << ",\"maximum_satellite_fraction\":0.02"
           << ",\"maximum_material_components\":4"
           << ",\"minimum_depth_p95_drop_m\":0.025"
           << ",\"maximum_penetration_m\":1e-12"
           << ",\"maximum_contact_velocity_error_m_s\":1e-4}"
           << ",\"observer_controls\":{\"passed\":"
           << (controls_passed ? "true" : "false")
           << ",\"empty_rejected\":"
           << (controls.empty_rejected ? "true" : "false")
           << ",\"nonfinite_rejected\":"
           << (controls.nonfinite_rejected ? "true" : "false")
           << ",\"mask_mutation_changed_root\":"
           << (controls.mask_mutation_changed_root ? "true" : "false")
           << ",\"depth_mutation_changed_root\":"
           << (controls.depth_mutation_changed_root ? "true" : "false")
           << ",\"root\":\"" << controls.root << "\"}"
           << ",\"lanes\":[";
    append_game_visual_lane(output, lane4k);
    output << ',';
    append_game_visual_lane(output, lane16k);
    output << "]"
           << ",\"result_root\":\"" << result_root << "\""
           << ",\"device\":" << device_json() << '}';
    return {quality_passed, output.str()};
}

namespace {

constexpr char GAME_SURFACE_STREAM_MAGIC[4] = {'N', 'E', 'W', 'S'};
// Version 2 (NGQ10) appends the fluid particle positions after the indices.
constexpr std::uint32_t GAME_SURFACE_STREAM_VERSION = 2U;
constexpr std::uint64_t GAME_SURFACE_STREAM_AUDIT_FRAMES = 60U;

template <typename T>
void write_stream_pod(std::ostream& out, T value) {
    out.write(reinterpret_cast<const char*>(&value), sizeof(T));
}

// Binary little-endian presentation surface frame consumed by the engine
// developer bridge. Vertex and triangle order match the OBJ writer exactly;
// indices are zero-based. Host byte order is little-endian on every supported
// x86_64 host; the reader validates the magic and version.
bool write_presentation_surface_stream_frame(
    std::ostream& out,
    const PresentationSurfaceFrame& frame,
    const GameQualityBox& box,
    int cycle,
    double simulation_seconds,
    double physics_ms,
    const std::vector<Particle>& particles) {
    if (!frame.valid || frame.mesh_vertices == 0U || frame.mesh_triangles == 0U) {
        return false;
    }
    out.write(GAME_SURFACE_STREAM_MAGIC, sizeof(GAME_SURFACE_STREAM_MAGIC));
    write_stream_pod<std::uint32_t>(out, GAME_SURFACE_STREAM_VERSION);
    write_stream_pod<std::int32_t>(out, frame.step);
    write_stream_pod<std::int32_t>(out, cycle);
    for (double value : {box.minimum.x, box.minimum.y, box.minimum.z,
             box.maximum.x, box.maximum.y, box.maximum.z}) {
        write_stream_pod<double>(out, value);
    }
    write_stream_pod<std::uint64_t>(out, frame.mesh_vertices);
    write_stream_pod<std::uint64_t>(out, frame.mesh_triangles);
    write_stream_pod<double>(out, simulation_seconds);
    write_stream_pod<double>(out, frame.extraction_ms);
    write_stream_pod<double>(out, physics_ms);
    std::vector<std::uint32_t> indices(frame.wet.size(), 0U);
    std::uint64_t vertex = 0U;
    for (std::uint32_t z = 0; z < frame.height; ++z) {
        for (std::uint32_t x = 0; x < frame.width; ++x) {
            const std::size_t pixel = static_cast<std::size_t>(z) * frame.width + x;
            if (frame.wet[pixel] == 0U) {
                continue;
            }
            indices[pixel] = static_cast<std::uint32_t>(++vertex);
            const double world_x = box.minimum.x
                + (static_cast<double>(x) + 0.5) * GAME_VISUAL_PIXEL_PITCH;
            const double world_z = box.minimum.z
                + (static_cast<double>(z) + 0.5) * GAME_VISUAL_PIXEL_PITCH;
            write_stream_pod<double>(out, world_x);
            write_stream_pod<double>(out, frame.depth[pixel]);
            write_stream_pod<double>(out, world_z);
        }
    }
    std::uint64_t triangles = 0U;
    for (std::uint32_t z = 0; z + 1U < frame.height; ++z) {
        for (std::uint32_t x = 0; x + 1U < frame.width; ++x) {
            const std::size_t a = static_cast<std::size_t>(z) * frame.width + x;
            const std::size_t b = a + 1U;
            const std::size_t d = static_cast<std::size_t>(z + 1U) * frame.width + x;
            const std::size_t c = d + 1U;
            if (indices[a] != 0U && indices[b] != 0U && indices[c] != 0U
                && indices[d] != 0U) {
                for (std::uint32_t index : {indices[a], indices[d], indices[c],
                         indices[a], indices[c], indices[b]}) {
                    write_stream_pod<std::uint32_t>(out, index - 1U);
                }
                triangles += 2U;
            }
        }
    }
    // Version 2: the fluid particle set (binary32 metres) for the ADR-102
    // screen-space pass; the reader bounds the count.
    write_stream_pod<std::uint64_t>(out, static_cast<std::uint64_t>(particles.size()));
    for (const Particle& particle : particles) {
        write_stream_pod<float>(out, static_cast<float>(particle.position.x));
        write_stream_pod<float>(out, static_cast<float>(particle.position.y));
        write_stream_pod<float>(out, static_cast<float>(particle.position.z));
    }
    out.flush();
    return static_cast<bool>(out) && vertex == frame.mesh_vertices
        && triangles == frame.mesh_triangles;
}

/// Depth tolerance for the GPU extractor against the CPU reference. The CPU
/// observer accumulates in `long double` and the GPU in `double`; at pixels
/// whose centre is exactly tangent to a particle sphere (the seed lattice
/// produces such ties) `sqrt(r^2 - d^2)` amplifies rounding noise to tens of
/// nanometres in either reference, so the frozen equivalence gate is
/// identical masks and mesh counts plus one micrometre of depth, the
/// engine's own position quantum.
constexpr double GAME_SURFACE_GPU_DEPTH_TOLERANCE = 1.0e-6;
constexpr double GAME_SURFACE_OBSERVER_TOLERANCE = 1.0e-12;

struct GpuSurfaceParams {
    double box_min[3];
    double box_max[3];
    double observer_lower[3];
    double observer_upper[3];
    unsigned width;
    unsigned height;
};

__global__ void surface_splat_kernel(
    const float3* positions,
    int count,
    GpuSurfaceParams params,
    unsigned long long* depth_bits,
    unsigned char* wet,
    int* error) {
    const int index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= count) {
        return;
    }
    const float3 raw = positions[index];
    const double px = raw.x;
    const double py = raw.y;
    const double pz = raw.z;
    if (!isfinite(px) || !isfinite(py) || !isfinite(pz)
        || px < params.observer_lower[0] - GAME_SURFACE_OBSERVER_TOLERANCE
        || py < params.observer_lower[1] - GAME_SURFACE_OBSERVER_TOLERANCE
        || pz < params.observer_lower[2] - GAME_SURFACE_OBSERVER_TOLERANCE
        || px > params.observer_upper[0] + GAME_SURFACE_OBSERVER_TOLERANCE
        || py > params.observer_upper[1] + GAME_SURFACE_OBSERVER_TOLERANCE
        || pz > params.observer_upper[2] + GAME_SURFACE_OBSERVER_TOLERANCE) {
        atomicOr(error, 1);
        return;
    }
    const double radius_squared = GAME_RADIUS * GAME_RADIUS;
    const long long min_x = max(0LL, static_cast<long long>(ceil(
        (px - GAME_RADIUS - params.box_min[0]) / GAME_VISUAL_PIXEL_PITCH - 0.5)));
    const long long max_x = min(static_cast<long long>(params.width) - 1LL,
        static_cast<long long>(floor(
            (px + GAME_RADIUS - params.box_min[0]) / GAME_VISUAL_PIXEL_PITCH - 0.5)));
    const long long min_z = max(0LL, static_cast<long long>(ceil(
        (pz - GAME_RADIUS - params.box_min[2]) / GAME_VISUAL_PIXEL_PITCH - 0.5)));
    const long long max_z = min(static_cast<long long>(params.height) - 1LL,
        static_cast<long long>(floor(
            (pz + GAME_RADIUS - params.box_min[2]) / GAME_VISUAL_PIXEL_PITCH - 0.5)));
    for (long long iz = min_z; iz <= max_z; ++iz) {
        const double pixel_z = params.box_min[2]
            + (static_cast<double>(iz) + 0.5) * GAME_VISUAL_PIXEL_PITCH;
        const double dz = pz - pixel_z;
        for (long long ix = min_x; ix <= max_x; ++ix) {
            const double pixel_x = params.box_min[0]
                + (static_cast<double>(ix) + 0.5) * GAME_VISUAL_PIXEL_PITCH;
            const double dx = px - pixel_x;
            const double distance_squared = dx * dx + dz * dz;
            if (distance_squared > radius_squared) {
                continue;
            }
            const double depth = py + sqrt(max(0.0, radius_squared - distance_squared));
            if (!isfinite(depth) || depth < 0.0) {
                atomicOr(error, 2);
                return;
            }
            const unsigned pixel = static_cast<unsigned>(iz) * params.width
                + static_cast<unsigned>(ix);
            // Non-negative doubles order like their bit patterns, so the
            // maximum height per pixel is one 64-bit atomic.
            atomicMax(&depth_bits[pixel],
                static_cast<unsigned long long>(__double_as_longlong(depth)));
            wet[pixel] = 1U;
        }
    }
}

/// NGQ6 revision-2 grayscale filters over the raw wet mask: `maximum`
/// selects the dilation, otherwise the erosion. Two passes form the closing.
__global__ void surface_closing_filter_kernel(
    const unsigned char* wet,
    const unsigned long long* input_bits,
    unsigned width,
    unsigned height,
    bool maximum,
    unsigned long long* output_bits) {
    const unsigned pixel = blockIdx.x * blockDim.x + threadIdx.x;
    if (pixel >= width * height) {
        return;
    }
    if (wet[pixel] == 0U) {
        output_bits[pixel] = 0ULL;
        return;
    }
    const int x = static_cast<int>(pixel % width);
    const int z = static_cast<int>(pixel / width);
    // Non-negative doubles order like their bit patterns, so max/min on the
    // bits equals max/min on the heights.
    unsigned long long value = input_bits[pixel];
    for (int dz = -GAME_CLOSING_RADIUS_PIXELS; dz <= GAME_CLOSING_RADIUS_PIXELS; ++dz) {
        for (int dx = -GAME_CLOSING_RADIUS_PIXELS; dx <= GAME_CLOSING_RADIUS_PIXELS; ++dx) {
            const int nx = x + dx;
            const int nz = z + dz;
            if (nx < 0 || nz < 0 || nx >= static_cast<int>(width)
                || nz >= static_cast<int>(height)) {
                continue;
            }
            const unsigned neighbor = static_cast<unsigned>(nz) * width
                + static_cast<unsigned>(nx);
            if (wet[neighbor] == 0U) {
                continue;
            }
            const unsigned long long candidate = input_bits[neighbor];
            value = maximum ? max(value, candidate) : min(value, candidate);
        }
    }
    output_bits[pixel] = value;
}

__device__ bool surface_neighbor_all(
    const unsigned char* mask, unsigned width, unsigned height, int x, int z) {
    bool all = true;
    for (int dz = -1; dz <= 1; ++dz) {
        for (int dx = -1; dx <= 1; ++dx) {
            const int nx = x + dx;
            const int nz = z + dz;
            if (nx < 0 || nz < 0 || nx >= static_cast<int>(width)
                || nz >= static_cast<int>(height)) {
                continue;
            }
            all = all && mask[static_cast<unsigned>(nz) * width + static_cast<unsigned>(nx)] != 0U;
        }
    }
    return all;
}

__device__ bool surface_neighbor_any(
    const unsigned char* mask, unsigned width, unsigned height, int x, int z) {
    bool any = false;
    for (int dz = -1; dz <= 1; ++dz) {
        for (int dx = -1; dx <= 1; ++dx) {
            const int nx = x + dx;
            const int nz = z + dz;
            if (nx >= 0 && nz >= 0 && nx < static_cast<int>(width)
                && nz < static_cast<int>(height)) {
                any = any
                    || mask[static_cast<unsigned>(nz) * width + static_cast<unsigned>(nx)] != 0U;
            }
        }
    }
    return any;
}

__global__ void surface_dilate_kernel(
    const unsigned char* retained, unsigned width, unsigned height, unsigned char* dilated) {
    const unsigned pixel = blockIdx.x * blockDim.x + threadIdx.x;
    if (pixel >= width * height) {
        return;
    }
    dilated[pixel] = surface_neighbor_any(
        retained, width, height, static_cast<int>(pixel % width), static_cast<int>(pixel / width));
}

__global__ void surface_erode_kernel(
    const unsigned char* dilated, unsigned width, unsigned height, unsigned char* closed) {
    const unsigned pixel = blockIdx.x * blockDim.x + threadIdx.x;
    if (pixel >= width * height) {
        return;
    }
    closed[pixel] = surface_neighbor_all(
        dilated, width, height, static_cast<int>(pixel % width), static_cast<int>(pixel / width));
}

__global__ void surface_base_depth_kernel(
    const unsigned char* closed,
    const unsigned char* retained,
    const unsigned long long* height_bits,
    unsigned width,
    unsigned height,
    double* base_depth,
    int* error) {
    const unsigned pixel = blockIdx.x * blockDim.x + threadIdx.x;
    if (pixel >= width * height) {
        return;
    }
    if (closed[pixel] == 0U) {
        base_depth[pixel] = 0.0;
        return;
    }
    if (retained[pixel] != 0U) {
        base_depth[pixel] = __longlong_as_double(static_cast<long long>(height_bits[pixel]));
        return;
    }
    const int x = static_cast<int>(pixel % width);
    const int z = static_cast<int>(pixel / width);
    double sum = 0.0;
    unsigned count = 0U;
    for (int dz = -1; dz <= 1; ++dz) {
        for (int dx = -1; dx <= 1; ++dx) {
            const int nx = x + dx;
            const int nz = z + dz;
            if (nx < 0 || nz < 0 || nx >= static_cast<int>(width)
                || nz >= static_cast<int>(height)) {
                continue;
            }
            const unsigned neighbor = static_cast<unsigned>(nz) * width
                + static_cast<unsigned>(nx);
            if (retained[neighbor] != 0U) {
                sum += __longlong_as_double(static_cast<long long>(height_bits[neighbor]));
                ++count;
            }
        }
    }
    if (count == 0U) {
        atomicOr(error, 4);
        base_depth[pixel] = 0.0;
        return;
    }
    base_depth[pixel] = sum / static_cast<double>(count);
}

__global__ void surface_bilateral_kernel(
    const unsigned char* closed,
    const double* base_depth,
    unsigned width,
    unsigned height,
    double floor_depth,
    double ceiling_depth,
    double* depth) {
    const unsigned pixel = blockIdx.x * blockDim.x + threadIdx.x;
    if (pixel >= width * height) {
        return;
    }
    if (closed[pixel] == 0U) {
        depth[pixel] = 0.0;
        return;
    }
    const int x = static_cast<int>(pixel % width);
    const int z = static_cast<int>(pixel / width);
    double bilateral_sum = 0.0;
    double bilateral_weight_sum = 0.0;
    // Same neighbour order as the CPU reference so the double sums match.
    for (int dz = -1; dz <= 1; ++dz) {
        for (int dx = -1; dx <= 1; ++dx) {
            const int nx = x + dx;
            const int nz = z + dz;
            if (nx < 0 || nz < 0 || nx >= static_cast<int>(width)
                || nz >= static_cast<int>(height)) {
                continue;
            }
            const unsigned neighbor = static_cast<unsigned>(nz) * width
                + static_cast<unsigned>(nx);
            if (closed[neighbor] == 0U) {
                continue;
            }
            const double weight = static_cast<double>(dx == 0 ? 2 : 1)
                * static_cast<double>(dz == 0 ? 2 : 1);
            const double range = base_depth[neighbor] - base_depth[pixel];
            const double range_weight = exp(-0.5 * range * range / (GAME_RADIUS * GAME_RADIUS));
            bilateral_sum += weight * range_weight * base_depth[neighbor];
            bilateral_weight_sum += weight * range_weight;
        }
    }
    const double value = bilateral_sum / bilateral_weight_sum;
    depth[pixel] = min(max(value, floor_depth), ceiling_depth);
}

/// GPU port of `game_surface_frame` + `extract_presentation_surface` for the
/// presentation output only: raw height splat, largest 8-connected component
/// (labelled on the host from the downloaded mask), 3x3 close, local fill and
/// the one-pass bilateral. Diagnostics, roots and gates of the CPU reference
/// are not computed; `verify` mode compares against that reference instead.
class GpuSurfaceExtractor {
public:
    GpuSurfaceExtractor(
        const GameQualityBox& box,
        std::size_t particle_capacity,
        int model,
        bool keep_all_components)
        : box_(box), particle_capacity_(particle_capacity), model_(model),
          keep_all_components_(keep_all_components) {
        const double cells_x = (box.maximum.x - box.minimum.x) / GAME_VISUAL_PIXEL_PITCH;
        const double cells_z = (box.maximum.z - box.minimum.z) / GAME_VISUAL_PIXEL_PITCH;
        width_ = static_cast<unsigned>(std::llround(cells_x));
        height_ = static_cast<unsigned>(std::llround(cells_z));
        if (width_ == 0U || height_ == 0U
            || std::abs(cells_x - static_cast<double>(width_)) > 1.0e-12
            || std::abs(cells_z - static_cast<double>(height_)) > 1.0e-12
            || box.minimum.y < 0.0) {
            throw std::invalid_argument("GPU surface extractor requires a pixel-aligned box above y=0");
        }
        pixels_ = static_cast<std::size_t>(width_) * height_;
        const float radius_f32 = static_cast<float>(GAME_RADIUS);
        const double lower_center[3] = {
            static_cast<double>(static_cast<float>(box.minimum.x) + radius_f32),
            static_cast<double>(static_cast<float>(box.minimum.y) + radius_f32),
            static_cast<double>(static_cast<float>(box.minimum.z) + radius_f32)};
        const double upper_center[3] = {
            static_cast<double>(static_cast<float>(box.maximum.x) - radius_f32),
            static_cast<double>(static_cast<float>(box.maximum.y) - radius_f32),
            static_cast<double>(static_cast<float>(box.maximum.z) - radius_f32)};
        const double box_min[3] = {box.minimum.x, box.minimum.y, box.minimum.z};
        const double box_max[3] = {box.maximum.x, box.maximum.y, box.maximum.z};
        for (int axis = 0; axis < 3; ++axis) {
            params_.box_min[axis] = box_min[axis];
            params_.box_max[axis] = box_max[axis];
            params_.observer_lower[axis] = std::min(box_min[axis] + GAME_RADIUS, lower_center[axis]);
            params_.observer_upper[axis] = std::max(box_max[axis] - GAME_RADIUS, upper_center[axis]);
        }
        params_.width = width_;
        params_.height = height_;
        check_cuda(cudaStreamCreateWithFlags(&stream_, cudaStreamNonBlocking),
            "create surface stream");
        check_cuda(cudaMalloc(&positions_, particle_capacity * sizeof(float3)), "surface positions");
        check_cuda(cudaMalloc(&depth_bits_, pixels_ * sizeof(unsigned long long)), "surface depth bits");
        check_cuda(cudaMalloc(&dome_bits_, pixels_ * sizeof(unsigned long long)), "surface dome bits");
        check_cuda(cudaMalloc(&ceiling_bits_, pixels_ * sizeof(unsigned long long)), "surface ceiling bits");
        check_cuda(cudaMalloc(&wet_, pixels_), "surface wet");
        check_cuda(cudaMalloc(&retained_, pixels_), "surface retained");
        check_cuda(cudaMalloc(&dilated_, pixels_), "surface dilated");
        check_cuda(cudaMalloc(&closed_, pixels_), "surface closed");
        check_cuda(cudaMalloc(&base_depth_, pixels_ * sizeof(double)), "surface base depth");
        check_cuda(cudaMalloc(&depth_, pixels_ * sizeof(double)), "surface depth");
        check_cuda(cudaMalloc(&error_, sizeof(int)), "surface error flag");
        host_positions_.reserve(particle_capacity);
    }

    GpuSurfaceExtractor(const GpuSurfaceExtractor&) = delete;
    GpuSurfaceExtractor& operator=(const GpuSurfaceExtractor&) = delete;

    ~GpuSurfaceExtractor() {
        cudaStreamSynchronize(stream_);
        cudaFree(positions_);
        cudaFree(depth_bits_);
        cudaFree(dome_bits_);
        cudaFree(ceiling_bits_);
        cudaFree(wet_);
        cudaFree(retained_);
        cudaFree(dilated_);
        cudaFree(closed_);
        cudaFree(base_depth_);
        cudaFree(depth_);
        cudaFree(error_);
        cudaStreamDestroy(stream_);
    }

    /// Fills `frame` (wet, depth, sizes, mesh counts, valid) and `raw_wet`
    /// with the pre-close observer mask. Returns false with a reason on any
    /// bounded failure; nothing is retried.
    bool extract(
        const std::vector<Particle>& fluid,
        int step,
        PresentationSurfaceFrame& frame,
        std::vector<std::uint8_t>& raw_wet,
        std::string& failure,
        std::vector<double>* raw_depth = nullptr) {
        const auto begin = std::chrono::steady_clock::now();
        frame = PresentationSurfaceFrame{};
        frame.step = step;
        frame.width = width_;
        frame.height = height_;
        frame.model = model_;
        if (fluid.empty() || fluid.size() > particle_capacity_) {
            failure = "gpu_surface_particle_capacity";
            return false;
        }
        host_positions_.clear();
        for (const Particle& particle : fluid) {
            host_positions_.push_back(make_float3(
                static_cast<float>(particle.position.x),
                static_cast<float>(particle.position.y),
                static_cast<float>(particle.position.z)));
        }
        const int count = static_cast<int>(fluid.size());
        check_cuda(cudaMemcpyAsync(positions_, host_positions_.data(),
                       fluid.size() * sizeof(float3), cudaMemcpyHostToDevice, stream_),
            "upload surface positions");
        check_cuda(cudaMemsetAsync(depth_bits_, 0, pixels_ * sizeof(unsigned long long), stream_),
            "reset surface depth bits");
        check_cuda(cudaMemsetAsync(wet_, 0, pixels_, stream_), "reset surface wet");
        check_cuda(cudaMemsetAsync(error_, 0, sizeof(int), stream_), "reset surface error");
        surface_splat_kernel<<<blocks_for(fluid.size()), THREADS, 0, stream_>>>(
            positions_, count, params_, depth_bits_, wet_, error_);
        raw_wet.resize(pixels_);
        int error = 0;
        check_cuda(cudaMemcpyAsync(raw_wet.data(), wet_, pixels_, cudaMemcpyDeviceToHost, stream_),
            "download surface wet");
        check_cuda(cudaMemcpyAsync(&error, error_, sizeof(int), cudaMemcpyDeviceToHost, stream_),
            "download surface error");
        check_cuda(cudaStreamSynchronize(stream_), "surface splat");
        if (error != 0) {
            failure = "gpu_surface_observer";
            return false;
        }
        frame.raw_wet_pixels = static_cast<std::uint64_t>(
            std::count(raw_wet.begin(), raw_wet.end(), std::uint8_t{1U}));
        if (frame.raw_wet_pixels == 0U) {
            failure = "gpu_surface_empty";
            return false;
        }
        const SurfaceMaskComponents components = label_surface_mask(raw_wet, width_, height_);
        if (components.sizes.empty()) {
            failure = "gpu_surface_components";
            return false;
        }
        const std::uint32_t largest_label = static_cast<std::uint32_t>(std::distance(
            components.sizes.begin(),
            std::max_element(components.sizes.begin(), components.sizes.end())));
        host_retained_.resize(pixels_);
        frame.retained_pixels = 0U;
        for (std::size_t pixel = 0; pixel < pixels_; ++pixel) {
            const std::uint32_t label = components.labels[pixel];
            const bool kept = keep_all_components_
                ? (raw_wet[pixel] != 0U
                    && components.sizes[label] >= GAME_SURFACE_MIN_COMPONENT_PIXELS)
                : label == largest_label;
            host_retained_[pixel] = static_cast<std::uint8_t>(kept);
            frame.retained_pixels += kept ? 1U : 0U;
        }
        check_cuda(cudaMemcpyAsync(retained_, host_retained_.data(), pixels_,
                       cudaMemcpyHostToDevice, stream_),
            "upload surface retained");
        const int blocks = blocks_for(pixels_);
        if (model_ != 0) {
            surface_closing_filter_kernel<<<blocks, THREADS, 0, stream_>>>(
                wet_, depth_bits_, width_, height_, true, ceiling_bits_);
            surface_closing_filter_kernel<<<blocks, THREADS, 0, stream_>>>(
                wet_, ceiling_bits_, width_, height_, false, dome_bits_);
        }
        surface_dilate_kernel<<<blocks, THREADS, 0, stream_>>>(retained_, width_, height_, dilated_);
        surface_erode_kernel<<<blocks, THREADS, 0, stream_>>>(dilated_, width_, height_, closed_);
        surface_base_depth_kernel<<<blocks, THREADS, 0, stream_>>>(
            closed_, retained_, model_ != 0 ? dome_bits_ : depth_bits_, width_, height_,
            base_depth_, error_);
        surface_bilateral_kernel<<<blocks, THREADS, 0, stream_>>>(
            closed_, base_depth_, width_, height_, box_.minimum.y, box_.maximum.y, depth_);
        frame.wet.resize(pixels_);
        frame.depth.resize(pixels_);
        if (raw_depth != nullptr || model_ != 0) {
            host_depth_bits_.resize(pixels_);
            check_cuda(cudaMemcpyAsync(host_depth_bits_.data(), depth_bits_,
                           pixels_ * sizeof(unsigned long long), cudaMemcpyDeviceToHost, stream_),
                "download surface raw depth");
        }
        if (model_ != 0) {
            host_ceiling_bits_.resize(pixels_);
            check_cuda(cudaMemcpyAsync(host_ceiling_bits_.data(), ceiling_bits_,
                           pixels_ * sizeof(unsigned long long), cudaMemcpyDeviceToHost, stream_),
                "download surface ceiling");
        }
        check_cuda(cudaMemcpyAsync(frame.wet.data(), closed_, pixels_, cudaMemcpyDeviceToHost, stream_),
            "download surface closed");
        check_cuda(cudaMemcpyAsync(frame.depth.data(), depth_, pixels_ * sizeof(double),
                       cudaMemcpyDeviceToHost, stream_),
            "download surface depth");
        check_cuda(cudaMemcpyAsync(&error, error_, sizeof(int), cudaMemcpyDeviceToHost, stream_),
            "download surface fill error");
        check_cuda(cudaStreamSynchronize(stream_), "surface extraction");
        if (error != 0) {
            failure = "gpu_surface_fill";
            return false;
        }
        const auto bits_to_double = [](std::uint64_t bits) {
            double value = 0.0;
            std::memcpy(&value, &bits, sizeof(value));
            return value;
        };
        if (raw_depth != nullptr) {
            raw_depth->resize(pixels_);
            for (std::size_t pixel = 0; pixel < pixels_; ++pixel) {
                (*raw_depth)[pixel] = bits_to_double(host_depth_bits_[pixel]);
            }
        }
        if (model_ != 0) {
            std::vector<double> lifts;
            lifts.reserve(frame.raw_wet_pixels);
            for (std::size_t pixel = 0; pixel < pixels_; ++pixel) {
                if (frame.wet[pixel] == 0U || raw_wet[pixel] == 0U) {
                    continue;
                }
                lifts.push_back(frame.depth[pixel] - bits_to_double(host_depth_bits_[pixel]));
                frame.ceiling_violations += static_cast<std::uint64_t>(
                    frame.depth[pixel] > bits_to_double(host_ceiling_bits_[pixel]) + 1.0e-9);
            }
            if (!lifts.empty()) {
                // Same nearest-rank definition as `game_percentile` without
                // the full sort; this runs on every live frame.
                const auto nearest_rank = [&lifts](double probability) {
                    const std::size_t rank = static_cast<std::size_t>(
                        std::ceil(probability * static_cast<double>(lifts.size())));
                    const std::size_t index = std::max<std::size_t>(1U, rank) - 1U;
                    std::nth_element(lifts.begin(), lifts.begin() + static_cast<std::ptrdiff_t>(index),
                        lifts.end());
                    return lifts[index];
                };
                frame.lift_min = *std::min_element(lifts.begin(), lifts.end());
                frame.lift_p50 = nearest_rank(0.50);
                frame.lift_p95 = nearest_rank(0.95);
            }
        }
        bool finite_depths = true;
        for (std::size_t pixel = 0; pixel < pixels_; ++pixel) {
            const bool surface_wet = frame.wet[pixel] != 0U;
            frame.wet_pixels += static_cast<std::uint64_t>(surface_wet);
            frame.common_pixels += static_cast<std::uint64_t>(surface_wet && raw_wet[pixel] != 0U);
            finite_depths = finite_depths && (!surface_wet || std::isfinite(frame.depth[pixel]));
        }
        frame.mesh_vertices = frame.wet_pixels;
        for (std::uint32_t z = 0; z + 1U < height_; ++z) {
            for (std::uint32_t x = 0; x + 1U < width_; ++x) {
                const std::size_t a = static_cast<std::size_t>(z) * width_ + x;
                const std::size_t b = a + 1U;
                const std::size_t d = static_cast<std::size_t>(z + 1U) * width_ + x;
                const std::size_t c = d + 1U;
                if (frame.wet[a] != 0U && frame.wet[b] != 0U && frame.wet[c] != 0U
                    && frame.wet[d] != 0U) {
                    frame.mesh_triangles += 2U;
                }
            }
        }
        frame.valid = frame.wet_pixels != 0U && frame.common_pixels != 0U && finite_depths
            && frame.mesh_triangles != 0U;
        frame.extraction_ms = std::chrono::duration<double, std::milli>(
            std::chrono::steady_clock::now() - begin).count();
        if (!frame.valid) {
            failure = "gpu_surface_invalid";
            return false;
        }
        if (model_ != 0
            && (frame.lift_p50 > GAME_CLOSING_LIFT_P50_LIMIT || frame.ceiling_violations != 0U)) {
            failure = "gpu_surface_closing_gate";
            return false;
        }
        return true;
    }

private:
    GameQualityBox box_;
    std::size_t particle_capacity_;
    int model_ = 0;
    bool keep_all_components_ = false;
    unsigned width_ = 0U;
    unsigned height_ = 0U;
    std::size_t pixels_ = 0U;
    GpuSurfaceParams params_{};
    cudaStream_t stream_{};
    float3* positions_ = nullptr;
    unsigned long long* depth_bits_ = nullptr;
    unsigned long long* dome_bits_ = nullptr;
    unsigned long long* ceiling_bits_ = nullptr;
    unsigned char* wet_ = nullptr;
    unsigned char* retained_ = nullptr;
    unsigned char* dilated_ = nullptr;
    unsigned char* closed_ = nullptr;
    double* base_depth_ = nullptr;
    double* depth_ = nullptr;
    int* error_ = nullptr;
    std::vector<float3> host_positions_;
    std::vector<std::uint8_t> host_retained_;
    std::vector<unsigned long long> host_depth_bits_;
    std::vector<unsigned long long> host_ceiling_bits_;
};

enum class StreamExtractorMode { Cpu, Gpu, Verify };

StreamExtractorMode parse_stream_extractor_mode(const std::string& value) {
    if (value == "cpu") {
        return StreamExtractorMode::Cpu;
    }
    if (value == "gpu") {
        return StreamExtractorMode::Gpu;
    }
    if (value == "verify") {
        return StreamExtractorMode::Verify;
    }
    throw std::invalid_argument("stream extractor must be cpu, gpu or verify");
}

/// Verification accumulators for `verify` mode.
struct GpuSurfaceVerification {
    std::uint64_t frames = 0U;
    std::uint64_t raw_mask_mismatch_pixels = 0U;
    std::uint64_t closed_mask_mismatch_pixels = 0U;
    std::uint64_t mesh_count_mismatches = 0U;
    double maximum_depth_difference = 0.0;
    double maximum_raw_depth_difference = 0.0;
    double cpu_total_ms = 0.0;
    double gpu_total_ms = 0.0;
};

/// NGQ6 accumulators over streamed frames (CPU or GPU, whichever was emitted).
struct DomeSurfaceStats {
    std::uint64_t frames = 0U;
    std::uint64_t gate_failures = 0U;
    /// NGQ9: retained raw wet pixels over raw wet pixels, minimum and sum.
    double minimum_retained_fraction = 1.0;
    double retained_fraction_sum = 0.0;
    std::uint64_t retained_frames = 0U;
    double maximum_lift_p50 = 0.0;
    double maximum_lift_p95 = 0.0;
    double minimum_lift = 0.0;
    std::uint64_t ceiling_violations = 0U;
};

struct StreamExtractionJob {
    std::vector<Particle> fluid;
    int step = 0;
    int cycle = 0;
    double physics_ms = 0.0;
    std::uint64_t sequence = 0U;
};

/// Runs observer, edge-aware extraction and frame serialization on a small
/// pool of worker threads so the solver keeps stepping. Jobs carry a
/// sequence number; finished frames are written strictly in sequence order
/// by whichever worker completes the next expected frame, so parallelism
/// never reorders or drops a frame. The bounded queue applies back-pressure
/// to the solver thread.
class StreamExtractor {
public:
    StreamExtractor(
        std::ostream& out,
        const GameQualityBox& box,
        unsigned workers,
        StreamExtractorMode mode,
        std::size_t particle_capacity,
        int surface_model,
        bool keep_all_components)
        : out_(out), box_(box), capacity_(std::max(1U, workers) * 2U), mode_(mode),
          particle_capacity_(particle_capacity), surface_model_(surface_model),
          keep_all_components_(keep_all_components) {
        for (unsigned index = 0; index < std::max(1U, workers); ++index) {
            workers_.emplace_back([this] { run(); });
        }
    }

    GpuSurfaceVerification verification() const {
        std::lock_guard<std::mutex> lock(mutex_);
        return verification_;
    }

    DomeSurfaceStats dome_stats() const {
        std::lock_guard<std::mutex> lock(mutex_);
        return dome_stats_;
    }

    StreamExtractor(const StreamExtractor&) = delete;
    StreamExtractor& operator=(const StreamExtractor&) = delete;

    ~StreamExtractor() {
        {
            std::lock_guard<std::mutex> lock(mutex_);
            stop_ = true;
        }
        condition_.notify_all();
        for (std::thread& worker : workers_) {
            if (worker.joinable()) {
                worker.join();
            }
        }
    }

    bool submit(StreamExtractionJob job) {
        std::unique_lock<std::mutex> lock(mutex_);
        condition_.wait(lock, [this] { return queue_.size() < capacity_ || !failure_.empty(); });
        if (!failure_.empty()) {
            return false;
        }
        job.sequence = next_sequence_++;
        queue_.push_back(std::move(job));
        lock.unlock();
        condition_.notify_all();
        return true;
    }

    /// Waits until every submitted frame has been written.
    bool drain() {
        std::unique_lock<std::mutex> lock(mutex_);
        condition_.wait(lock, [this] { return next_write_ == next_sequence_ || !failure_.empty(); });
        return failure_.empty();
    }

    std::string failure() const {
        std::lock_guard<std::mutex> lock(mutex_);
        return failure_;
    }

    std::uint64_t frames_written() const {
        std::lock_guard<std::mutex> lock(mutex_);
        return frames_written_;
    }

    double observer_total_ms() const {
        std::lock_guard<std::mutex> lock(mutex_);
        return observer_total_ms_;
    }

    double extraction_total_ms() const {
        std::lock_guard<std::mutex> lock(mutex_);
        return extraction_total_ms_;
    }

    double frame_wall_max_ms() const {
        std::lock_guard<std::mutex> lock(mutex_);
        return frame_wall_max_ms_;
    }

    std::size_t worker_count() const {
        return workers_.size();
    }

private:
    void run() {
        std::unique_ptr<GpuSurfaceExtractor> gpu;
        if (mode_ != StreamExtractorMode::Cpu) {
            try {
                gpu = std::make_unique<GpuSurfaceExtractor>(
                    box_, particle_capacity_, surface_model_, keep_all_components_);
            } catch (const std::exception& error) {
                std::lock_guard<std::mutex> lock(mutex_);
                if (failure_.empty()) {
                    failure_ = std::string("gpu_surface_setup:") + error.what();
                }
                condition_.notify_all();
                return;
            }
        }
        while (true) {
            StreamExtractionJob job;
            {
                std::unique_lock<std::mutex> lock(mutex_);
                condition_.wait(lock, [this] { return !queue_.empty() || stop_; });
                if (queue_.empty()) {
                    return;
                }
                job = std::move(queue_.front());
                queue_.erase(queue_.begin());
            }
            condition_.notify_all();
            const auto frame_begin = std::chrono::steady_clock::now();
            std::string failure;
            double observer_ms = 0.0;
            double extraction_ms = 0.0;
            GpuSurfaceVerification verified;
            std::ostringstream serialized;
            PresentationSurfaceFrame frame;
            bool frame_ready = false;
            if (mode_ != StreamExtractorMode::Gpu) {
                const GameSurfaceFrame raw = game_surface_frame(job.fluid, box_, job.step);
                observer_ms = std::chrono::duration<double, std::milli>(
                    std::chrono::steady_clock::now() - frame_begin).count();
                if (!raw.valid) {
                    failure = "surface_observer";
                } else {
                    GameDomeField dome;
                    if (surface_model_ != 0) {
                        const auto dome_begin = std::chrono::steady_clock::now();
                        dome = game_surface_dome_field(job.fluid, box_, raw);
                        observer_ms += std::chrono::duration<double, std::milli>(
                            std::chrono::steady_clock::now() - dome_begin).count();
                    }
                    if (surface_model_ != 0 && !dome.valid) {
                        failure = "surface_dome_field";
                    } else {
                        frame = extract_presentation_surface(
                            raw, box_, surface_model_ != 0 ? &dome : nullptr,
                            keep_all_components_);
                        extraction_ms = frame.extraction_ms;
                        frame_ready = frame.valid;
                        if (!frame.valid) {
                            failure = "surface_extraction";
                        } else if (surface_model_ != 0 && !frame.passed) {
                            failure = "surface_closing_gate step=" + std::to_string(frame.step)
                                + " components=" + std::to_string(frame.components)
                                + " retained_components="
                                + std::to_string(frame.retained_components)
                                + " local_fill_only=" + std::to_string(frame.local_fill_only)
                                + " expansion="
                                + std::to_string(*std::max_element(
                                    frame.bounding_box_expansion_pixels.begin(),
                                    frame.bounding_box_expansion_pixels.end()))
                                + " area_ratio=" + std::to_string(frame.area_ratio)
                                + " coverage=" + std::to_string(frame.common_coverage)
                                + " lift_p50=" + std::to_string(frame.lift_p50)
                                + " ceiling=" + std::to_string(frame.ceiling_violations);
                            frame_ready = false;
                        }
                    }
                }
                if (failure.empty() && mode_ == StreamExtractorMode::Verify) {
                    PresentationSurfaceFrame gpu_frame;
                    std::vector<std::uint8_t> gpu_raw_wet;
                    std::vector<double> gpu_raw_depth;
                    if (!gpu->extract(
                            job.fluid, job.step, gpu_frame, gpu_raw_wet, failure, &gpu_raw_depth)) {
                        frame_ready = false;
                    } else {
                        verified.frames = 1U;
                        verified.cpu_total_ms = observer_ms + extraction_ms;
                        verified.gpu_total_ms = gpu_frame.extraction_ms;
                        for (std::size_t pixel = 0; pixel < gpu_raw_wet.size(); ++pixel) {
                            verified.raw_mask_mismatch_pixels +=
                                static_cast<std::uint64_t>(gpu_raw_wet[pixel] != raw.wet[pixel]);
                            if (gpu_raw_wet[pixel] != 0U && raw.wet[pixel] != 0U) {
                                verified.maximum_raw_depth_difference = std::max(
                                    verified.maximum_raw_depth_difference,
                                    std::abs(raw.depth[pixel] - gpu_raw_depth[pixel]));
                            }
                            const bool cpu_wet = frame.wet[pixel] != 0U;
                            const bool gpu_wet = gpu_frame.wet[pixel] != 0U;
                            verified.closed_mask_mismatch_pixels +=
                                static_cast<std::uint64_t>(cpu_wet != gpu_wet);
                            if (cpu_wet && gpu_wet) {
                                verified.maximum_depth_difference = std::max(
                                    verified.maximum_depth_difference,
                                    std::abs(frame.depth[pixel] - gpu_frame.depth[pixel]));
                            }
                        }
                        verified.mesh_count_mismatches = static_cast<std::uint64_t>(
                            gpu_frame.mesh_vertices != frame.mesh_vertices
                            || gpu_frame.mesh_triangles != frame.mesh_triangles);
                        if (verified.closed_mask_mismatch_pixels != 0U
                            || verified.mesh_count_mismatches != 0U
                            || !(verified.maximum_depth_difference
                                <= GAME_SURFACE_GPU_DEPTH_TOLERANCE)) {
                            failure = "gpu_surface_mismatch";
                            frame_ready = false;
                        } else {
                            // Stream the GPU result so verify mode exercises
                            // the same bytes that gpu mode would emit.
                            frame = std::move(gpu_frame);
                        }
                    }
                }
            } else {
                std::vector<std::uint8_t> gpu_raw_wet;
                frame_ready = gpu->extract(job.fluid, job.step, frame, gpu_raw_wet, failure);
                extraction_ms = frame.extraction_ms;
            }
            if (failure.empty() && frame_ready
                && !write_presentation_surface_stream_frame(
                    serialized, frame, box_, job.cycle,
                    static_cast<double>(job.step) * GAME_TIME_STEP, job.physics_ms,
                    job.fluid)) {
                failure = "surface_serialization";
            }
            const double frame_wall_ms = std::chrono::duration<double, std::milli>(
                std::chrono::steady_clock::now() - frame_begin).count();
            {
                std::unique_lock<std::mutex> lock(mutex_);
                observer_total_ms_ += observer_ms;
                extraction_total_ms_ += extraction_ms;
                frame_wall_max_ms_ = std::max(frame_wall_max_ms_, frame_wall_ms);
                if (frame_ready && frame.raw_wet_pixels > 0U) {
                    const double retained = static_cast<double>(frame.retained_pixels)
                        / static_cast<double>(frame.raw_wet_pixels);
                    dome_stats_.minimum_retained_fraction =
                        std::min(dome_stats_.minimum_retained_fraction, retained);
                    dome_stats_.retained_fraction_sum += retained;
                    ++dome_stats_.retained_frames;
                }
                if (surface_model_ != 0
                    && (frame_ready || failure.rfind("surface_closing_gate", 0) == 0
                        || failure == "gpu_surface_closing_gate")) {
                    ++dome_stats_.frames;
                    dome_stats_.gate_failures += static_cast<std::uint64_t>(!frame_ready);
                    dome_stats_.maximum_lift_p50 =
                        std::max(dome_stats_.maximum_lift_p50, frame.lift_p50);
                    dome_stats_.maximum_lift_p95 =
                        std::max(dome_stats_.maximum_lift_p95, frame.lift_p95);
                    dome_stats_.minimum_lift = dome_stats_.frames == 1U
                        ? frame.lift_min
                        : std::min(dome_stats_.minimum_lift, frame.lift_min);
                    dome_stats_.ceiling_violations += frame.ceiling_violations;
                }
                verification_.frames += verified.frames;
                verification_.raw_mask_mismatch_pixels += verified.raw_mask_mismatch_pixels;
                verification_.closed_mask_mismatch_pixels +=
                    verified.closed_mask_mismatch_pixels;
                verification_.mesh_count_mismatches += verified.mesh_count_mismatches;
                verification_.maximum_depth_difference = std::max(
                    verification_.maximum_depth_difference, verified.maximum_depth_difference);
                verification_.maximum_raw_depth_difference = std::max(
                    verification_.maximum_raw_depth_difference,
                    verified.maximum_raw_depth_difference);
                verification_.cpu_total_ms += verified.cpu_total_ms;
                verification_.gpu_total_ms += verified.gpu_total_ms;
                if (!failure.empty()) {
                    if (failure_.empty()) {
                        failure_ = failure;
                    }
                } else {
                    finished_.emplace(job.sequence, serialized.str());
                    // Flush every frame that is now contiguous with the
                    // write cursor; the writer holds the lock, so frames
                    // reach stdout strictly in sequence order.
                    while (failure_.empty()) {
                        const auto next = finished_.find(next_write_);
                        if (next == finished_.end()) {
                            break;
                        }
                        out_.write(next->second.data(),
                            static_cast<std::streamsize>(next->second.size()));
                        out_.flush();
                        finished_.erase(next);
                        if (!out_) {
                            failure_ = "stream_closed";
                            break;
                        }
                        ++frames_written_;
                        ++next_write_;
                    }
                }
            }
            condition_.notify_all();
        }
    }

    std::ostream& out_;
    GameQualityBox box_;
    std::size_t capacity_;
    StreamExtractorMode mode_;
    std::size_t particle_capacity_;
    int surface_model_ = 0;
    bool keep_all_components_ = false;
    GpuSurfaceVerification verification_;
    DomeSurfaceStats dome_stats_;
    mutable std::mutex mutex_;
    std::condition_variable condition_;
    std::vector<StreamExtractionJob> queue_;
    std::map<std::uint64_t, std::string> finished_;
    std::uint64_t next_sequence_ = 0U;
    std::uint64_t next_write_ = 0U;
    bool stop_ = false;
    std::string failure_;
    std::uint64_t frames_written_ = 0U;
    double observer_total_ms_ = 0.0;
    double extraction_total_ms_ = 0.0;
    double frame_wall_max_ms_ = 0.0;
    std::vector<std::thread> workers_;
};

} // namespace

CommandReport run_cuda_game_surface_stream(
    const std::string& lane,
    int steps,
    int every,
    int cycles,
    int workers,
    const std::string& extractor_name,
    const std::string& surface_model_name,
    std::ostream& frames,
    const std::string& particle_dump_prefix,
    int boundary_layers,
    const std::string& boundary_support,
    bool boundary_lid,
    const std::string& spill_lip,
    int iterations_override,
    const std::string& surface_components) {
    if (surface_components != "largest" && surface_components != "all") {
        throw std::invalid_argument("stream surface components must be largest or all");
    }
    const bool keep_all_components = surface_components == "all";
    if (boundary_support != "full" && boundary_support != "density") {
        throw std::invalid_argument("stream boundary support must be full or density");
    }
    if (iterations_override < 0 || iterations_override > 50) {
        throw std::invalid_argument("stream iterations override must be 0..50");
    }
    if (spill_lip != "margin" && spill_lip != "flush" && spill_lip != "open") {
        throw std::invalid_argument("stream spill lip must be margin, flush or open");
    }
    const StreamExtractorMode extractor_mode = parse_stream_extractor_mode(extractor_name);
    if (boundary_layers < 0 || boundary_layers > 2) {
        throw std::invalid_argument("stream boundary layers must be 0, 1 or 2");
    }
    if (!particle_dump_prefix.empty() && !game_frame_prefix_valid(particle_dump_prefix)) {
        throw std::invalid_argument("particle dump prefix contains unsupported characters");
    }
    int surface_model = 0;
    if (surface_model_name == "closing") {
        surface_model = 1;
    } else if (surface_model_name != "sphere") {
        throw std::invalid_argument("stream surface model must be sphere or closing");
    }
    if (steps < 1 || steps > 100000 || every < 1 || every > steps || cycles < 0
        || cycles > 1000000 || workers < 1 || workers > 16) {
        throw std::invalid_argument(
            "stream steps/every/cycles/workers are outside the bounded range");
    }
    const Profile& profile =
        find_profile("nuv-basin-48k-analytic-contact-game-cap160.v6");
    GameQualityBox box;
    int lattice_x = 0;
    int lattice_y = 10;
    int lattice_z = 0;
    Fixture::Spill spill;
    if (lane == "4k") {
        box = GameQualityBox{{0.0, 0.0, 0.0}, {2.0, 0.75, 1.0}, {40, 15, 20}};
        lattice_x = 20;
        lattice_z = 20;
    } else if (lane == "16k") {
        box = GameQualityBox{{0.0, 0.0, 0.0}, {4.0, 0.75, 2.0}, {80, 15, 40}};
        lattice_x = 40;
        lattice_z = 40;
    } else if (lane == "48k") {
        // SPEC-38 production fixture extent: the 4 x 1 x 2 m sealed basin
        // with the accepted 80 x 15 x 40 fill (0.75 m deep), released from
        // the same 0.1 m lift as the visual lanes so it slams and sloshes.
        box = GameQualityBox{{0.0, 0.0, 0.0}, {4.0, 1.0, 2.0}, {80, 20, 40}};
        lattice_x = 80;
        lattice_y = 15;
        lattice_z = 40;
    } else if (lane == "48k-dam") {
        // Same sample count as the production fixture, released as a 2 m
        // column in a taller 4 x 2 x 2 m box so the dam-break front is
        // visible at 48k.
        box = GameQualityBox{{0.0, 0.0, 0.0}, {4.0, 2.0, 2.0}, {80, 40, 40}};
        lattice_x = 40;
        lattice_y = 30;
        lattice_z = 40;
    } else if (lane == "spill-pipe") {
        // NGQ8 revision 9 (plan 22): the same two tanks, but the divider is
        // closed and the upper tank drains through a 0.2 x 0.2 m shaft at
        // its floor centre into a horizontal duct under the floor that
        // leaves the divider inside a 0.4 m pipe body (0.1 m walls) with
        // its open end 0.6 m above the lower floor.
        box = GameQualityBox{{0.0, 0.0, 0.0}, {5.0, 2.0, 1.5}, {100, 40, 30}};
        lattice_x = 40;
        lattice_y = 10;
        lattice_z = 30;
        spill.enabled = true;
        spill.under_floor = true;
        spill.shelf_top = 1.0;
        spill.wall_x0 = 2.0;
        spill.wall_x1 = 2.2;
        spill.shaft_x0 = 0.9;
        spill.shaft_x1 = 1.1;
        spill.opening_y0 = 0.5;
        spill.opening_y1 = 0.7;
        spill.opening_z0 = 0.65;
        spill.opening_z1 = 0.85;
        spill.pipe_x1 = 2.6;
        spill.pipe_wall = 0.1;
        spill.flush = true;
        spill.open_ring = false;
    } else if (lane == "spill" || lane == "spill-narrow") {
        // NGQ8 two-tank spillway (plan 22): the upper tank sits on a 1 m
        // shelf left of a 0.2 m divider whose opening drains into the
        // empty lower tank. Revision 2 narrows the opening to 0.15 x 0.2 m.
        box = GameQualityBox{{0.0, 0.0, 0.0}, {5.0, 2.0, 1.5}, {100, 40, 30}};
        lattice_x = 40;
        lattice_y = 10;
        lattice_z = 30;
        spill.enabled = true;
        spill.shelf_top = 1.0;
        spill.wall_x0 = 2.0;
        spill.wall_x1 = 2.2;
        spill.opening_y0 = 1.0;
        spill.flush = spill_lip != "margin";
        spill.open_ring = spill_lip == "open";
        if (lane == "spill") {
            spill.opening_y1 = 1.3;
            spill.opening_z0 = 0.5;
            spill.opening_z1 = 1.0;
        } else {
            spill.opening_y1 = 1.15;
            spill.opening_z0 = 0.65;
            spill.opening_z1 = 0.85;
        }
    } else {
        throw std::invalid_argument(
            "stream lane must be 4k, 16k, 48k, 48k-dam, spill, spill-narrow or spill-pipe");
    }
    double spill_maximum_penetration = 0.0;
    double spill_upper_fraction = 1.0;
    double spill_upper_at_2s = -1.0;
    bool spill_arrived_by_480 = false;
    // Exit-speed diagnostic: per second, head above the shelf, mean speed of
    // samples just past the divider (from emitted-frame differences), the
    // free-fall speed for that head and the maximum sample speed anywhere.
    std::string spill_exit_curve;
    std::vector<Vec3> spill_previous_positions;
    double spill_maximum_exit_ratio = 0.0;
    std::string spill_fast_curve;
    std::string spill_drain_curve;
    std::uint64_t completed_steps = 0U;
    std::uint64_t audits = 0U;
    int completed_cycles = 0;
    double total_physics_ms = 0.0;
    double total_execute_wall_ms = 0.0;
    double total_wall_ms = 0.0;
    double maximum_step_wall_ms = 0.0;
    std::size_t maximum_degree = 0U;
    std::size_t dynamic_samples = 0U;
    std::size_t boundary_samples = 0U;
    std::string first_failure;
    const auto stream_begin = std::chrono::steady_clock::now();
    StreamExtractor extractor(
        frames, box, static_cast<unsigned>(workers), extractor_mode,
        static_cast<std::size_t>(lattice_x) * static_cast<std::size_t>(lattice_y) * lattice_z,
        surface_model, keep_all_components);
    for (int cycle = 0; (cycles == 0 || cycle < cycles) && first_failure.empty();
         ++cycle) {
        std::vector<Particle> fluid = game_visual_particles(lattice_x, lattice_z, lattice_y);
        if (spill.enabled) {
            for (Particle& particle : fluid) {
                particle.position.y += spill.shelf_top;
            }
        }
        dynamic_samples = fluid.size();
        double physics_since_frame_ms = 0.0;
        const auto emit = [&](int step) {
            if (!particle_dump_prefix.empty()) {
                // Research diagnostic only: sample-ordered float positions of
                // this emitted frame, so velocities follow from differences.
                std::ofstream dump(
                    particle_dump_prefix + "-cycle" + std::to_string(cycle) + "-step"
                        + std::to_string(step) + ".bin",
                    std::ios::binary);
                const std::uint64_t count = fluid.size();
                dump.write(reinterpret_cast<const char*>(&count), sizeof(count));
                for (const Particle& particle : fluid) {
                    const float values[3] = {
                        static_cast<float>(particle.position.x),
                        static_cast<float>(particle.position.y),
                        static_cast<float>(particle.position.z)};
                    dump.write(reinterpret_cast<const char*>(values), sizeof(values));
                }
                if (!dump) {
                    first_failure = "particle_dump";
                    return false;
                }
            }
            StreamExtractionJob job;
            job.fluid = fluid;
            job.step = step;
            job.cycle = cycle;
            job.physics_ms = physics_since_frame_ms;
            physics_since_frame_ms = 0.0;
            if (!extractor.submit(std::move(job))) {
                first_failure = extractor.failure();
                return false;
            }
            return true;
        };
        if (!emit(0)) {
            break;
        }
        // One persistent solver per cycle: the advected fixture keeps the
        // particle state on the device and `execute(capture, advance)` hands
        // the published positions/velocities to the next step without a
        // host round trip. Host state is refreshed only on emitted frames.
        // NGQ7: optional fixed lattice complement on all box faces gives the
        // floor and wall neighbourhoods their density support (D-047).
        // NGQ8 revision 4: an explicit iteration count is a research
        // override of the accepted profile, reported in the summary.
        Fixture fixture = game_fixture(
            profile, "game-stream-" + lane, fluid, box,
            iterations_override > 0 ? iterations_override : profile.fixed_iterations,
            profile.max_neighbors, boundary_layers, boundary_lid && !spill.enabled);
        if (spill.enabled) {
            append_spill_solids(fixture, box, spill);
            fixture.spill = spill;
            fixture.pair_capacity = fixture.particles.size() * profile.max_neighbors;
        }
        boundary_samples = fixture.particles.size() - fluid.size();
        fixture.boundary_density_only = boundary_support == "density";
        fixture.advected = true;
        // The persistent grid is sized once from the initial column; give it
        // the whole basin so particles never leave the neighbor grid.
        fixture.grid_margin = std::max(
            {box.maximum.x - box.minimum.x, box.maximum.y - box.minimum.y,
                box.maximum.z - box.minimum.z});
        CudaBaseline gpu(
            fixture, P1_ACCUMULATION, P1_HANDOFF, P1_TERMS, P1_STORAGE,
            PairTraversalMode::FusedOwnerTermsP1,
            NeighborEncodingMode::CompactU16P2);
        std::vector<float3> published;
        std::uint64_t emitted_in_cycle = 0U;
        for (int step = 1; step <= steps; ++step) {
            const auto step_begin = std::chrono::steady_clock::now();
            const bool frame_step = step % every == 0;
            // The full diagnostic capture (neighbors, energies, topology)
            // costs far more than the step; audit it once per
            // GAME_SURFACE_STREAM_AUDIT_FRAMES emitted frames and otherwise
            // download only the published positions.
            const bool audit = frame_step
                && emitted_in_cycle % GAME_SURFACE_STREAM_AUDIT_FRAMES == 0U;
            const CapturedRun run = gpu.execute(audit, true);
            total_execute_wall_ms += std::chrono::duration<double, std::milli>(
                std::chrono::steady_clock::now() - step_begin).count();
            bool step_valid = !run.local_solve_failed && run.neighbor_build_valid
                && run.neighbor_id_bytes == sizeof(std::uint16_t);
            if (audit) {
                maximum_degree = std::max(maximum_degree, run.state.maximum_degree);
                step_valid = step_valid && game_finite(run, fluid.size())
                    && run.state.next_position.size() == fluid.size() + boundary_samples
                    && run.state.directed_pairs <= fixture.pair_capacity
                    && run.state.maximum_degree <= profile.max_neighbors;
                ++audits;
            }
            if (!step_valid) {
                first_failure = run.local_solve_failed ? "local_solve"
                    : (!run.neighbor_build_valid ? "neighbor_build" : "step_apparatus");
                break;
            }
            total_physics_ms += run.timing.total;
            physics_since_frame_ms += run.timing.total;
            ++completed_steps;
            if (frame_step) {
                if (audit) {
                    for (std::size_t index = 0; index < fluid.size(); ++index) {
                        fluid[index].position = run.state.next_position[index];
                        fluid[index].velocity = run.state.final_velocity[index];
                    }
                } else {
                    gpu.download_positions(published);
                    if (published.size() != fluid.size() + boundary_samples) {
                        first_failure = "output_size";
                        break;
                    }
                    bool finite = true;
                    for (std::size_t index = 0; index < fluid.size(); ++index) {
                        const float3 value = published[index];
                        finite = finite && std::isfinite(value.x) && std::isfinite(value.y)
                            && std::isfinite(value.z);
                        fluid[index].position = Vec3{value.x, value.y, value.z};
                    }
                    if (!finite) {
                        first_failure = "nonfinite_state";
                        break;
                    }
                }
                if (spill.enabled) {
                    std::size_t upper = 0U;
                    for (const Particle& particle : fluid) {
                        spill_maximum_penetration = std::max(
                            spill_maximum_penetration,
                            spill_penetration(particle.position, spill));
                        upper += particle.position.x < spill.wall_x0 ? 1U : 0U;
                        spill_arrived_by_480 = spill_arrived_by_480
                            || (step <= 480 && particle.position.x > spill.exit_x()
                                && particle.position.y < 0.2);
                    }
                    spill_upper_fraction =
                        static_cast<double>(upper) / static_cast<double>(fluid.size());
                    if (step % 240 == 0) {
                        spill_drain_curve += (spill_drain_curve.empty() ? "" : ",")
                            + std::to_string(spill_upper_fraction);
                    }
                    if (step == 480) {
                        spill_upper_at_2s = spill_upper_fraction;
                    }
                    if (spill_previous_positions.size() == fluid.size()) {
                        const double frame_seconds = every * GAME_TIME_STEP;
                        double head_sum = 0.0;
                        std::size_t head_count = 0U;
                        double exit_sum = 0.0;
                        std::size_t exit_count = 0U;
                        double maximum_speed = 0.0;
                        for (std::size_t index = 0; index < fluid.size(); ++index) {
                            const Vec3& p = fluid[index].position;
                            const Vec3 delta = p - spill_previous_positions[index];
                            const double speed = norm(delta) / frame_seconds;
                            maximum_speed = std::max(maximum_speed, speed);
                            if (p.x < spill.wall_x0) {
                                head_sum += p.y - spill.shelf_top;
                                ++head_count;
                            }
                            if (p.x > spill.exit_x() && p.x < spill.exit_x() + 0.3
                                && p.y > spill.opening_y0 - 0.3) {
                                exit_sum += delta.x / frame_seconds;
                                ++exit_count;
                            }
                        }
                        if (step % 240 == 0) {
                            // Revision 5 diagnostic: the five fastest samples
                            // with their neighbourhoods, plus the sheet degree.
                            std::vector<std::pair<double, std::size_t>> ranked;
                            ranked.reserve(fluid.size());
                            for (std::size_t index = 0; index < fluid.size(); ++index) {
                                ranked.emplace_back(
                                    norm(fluid[index].position - spill_previous_positions[index])
                                        / frame_seconds,
                                    index);
                            }
                            std::partial_sort(
                                ranked.begin(), ranked.begin() + std::min<std::size_t>(5U, ranked.size()),
                                ranked.end(), [](const auto& a, const auto& b) { return a.first > b.first; });
                            const double horizon = fixture.horizon;
                            const auto degrees = [&](const Vec3& centre) {
                                std::size_t fluid_degree = 0U;
                                std::size_t fixed_degree = 0U;
                                for (const Particle& other : fluid) {
                                    if (norm(other.position - centre) < horizon) {
                                        ++fluid_degree;
                                    }
                                }
                                for (std::size_t index = fluid.size(); index < fixture.particles.size(); ++index) {
                                    if (norm(fixture.particles[index].position - centre) < horizon) {
                                        ++fixed_degree;
                                    }
                                }
                                return std::make_pair(fluid_degree > 0U ? fluid_degree - 1U : 0U, fixed_degree);
                            };
                            const auto region = [&](const Vec3& p) {
                                if (spill.under_floor && p.y < spill.shelf_top
                                    && p.x < spill.exit_x()) {
                                    return "pipe";
                                }
                                if (p.x < spill.wall_x0) {
                                    return p.y < spill.shelf_top + 0.2 ? "sheet" : "upper";
                                }
                                if (!spill.under_floor && p.x <= spill.wall_x1) {
                                    return "pipe";
                                }
                                if (p.x < spill.exit_x() + 0.3 && p.y > spill.opening_y0 - 0.3) {
                                    return "exit";
                                }
                                return p.y < 0.3 ? "lower" : "air";
                            };
                            std::string fast;
                            for (std::size_t rank = 0; rank < std::min<std::size_t>(5U, ranked.size()); ++rank) {
                                const Vec3& p = fluid[ranked[rank].second].position;
                                const auto [fluid_degree, fixed_degree] = degrees(p);
                                fast += (fast.empty() ? "" : ",") + std::string("{\"speed\":")
                                    + std::to_string(ranked[rank].first) + ",\"region\":\"" + region(p)
                                    + "\",\"fluid_degree\":" + std::to_string(fluid_degree)
                                    + ",\"fixed_degree\":" + std::to_string(fixed_degree)
                                    + ",\"y\":" + std::to_string(p.y) + "}";
                            }
                            double sheet_degree_sum = 0.0;
                            std::size_t sheet_count = 0U;
                            for (std::size_t index = 0; index < fluid.size() && sheet_count < 64U; index += 97U) {
                                const Vec3& p = fluid[index].position;
                                if (p.x < spill.wall_x0 && p.y < spill.shelf_top + 0.2) {
                                    sheet_degree_sum += static_cast<double>(degrees(p).first);
                                    ++sheet_count;
                                }
                            }
                            spill_fast_curve += (spill_fast_curve.empty() ? "" : ",")
                                + std::string("{\"second\":") + std::to_string(step / 240)
                                + ",\"sheet_mean_fluid_degree\":"
                                + std::to_string(sheet_count > 0U ? sheet_degree_sum / static_cast<double>(sheet_count) : 0.0)
                                + ",\"fastest\":[" + fast + "]}";
                            // Mean sample height above the shelf is half the
                            // sheet depth; head = twice the mean, measured
                            // down to the exit centre for the under-floor pipe.
                            const double head = (head_count > 0U
                                ? 2.0 * head_sum / static_cast<double>(head_count) : 0.0)
                                + (spill.under_floor ? spill.shelf_top - spill.exit_centre_y() : 0.0);
                            const double free_fall = head > 0.0 ? std::sqrt(2.0 * 9.81 * head) : 0.0;
                            const double exit_speed = exit_count > 0U
                                ? exit_sum / static_cast<double>(exit_count) : 0.0;
                            const double ratio = free_fall > 0.0 ? exit_speed / free_fall : 0.0;
                            spill_maximum_exit_ratio = std::max(spill_maximum_exit_ratio, ratio);
                            spill_exit_curve += (spill_exit_curve.empty() ? "" : ",")
                                + std::string("{\"second\":") + std::to_string(step / 240)
                                + ",\"head_m\":" + std::to_string(head)
                                + ",\"exit_speed_mps\":" + std::to_string(exit_speed)
                                + ",\"free_fall_mps\":" + std::to_string(free_fall)
                                + ",\"ratio\":" + std::to_string(ratio)
                                + ",\"exit_samples\":" + std::to_string(exit_count)
                                + ",\"max_speed_mps\":" + std::to_string(maximum_speed) + "}";
                        }
                    }
                    spill_previous_positions.resize(fluid.size());
                    for (std::size_t index = 0; index < fluid.size(); ++index) {
                        spill_previous_positions[index] = fluid[index].position;
                    }
                }
                ++emitted_in_cycle;
                if (!emit(step)) {
                    break;
                }
            }
            const double step_wall_ms = std::chrono::duration<double, std::milli>(
                std::chrono::steady_clock::now() - step_begin).count();
            total_wall_ms += step_wall_ms;
            maximum_step_wall_ms = std::max(maximum_step_wall_ms, step_wall_ms);
        }
        if (first_failure.empty()) {
            ++completed_cycles;
        }
    }
    if (first_failure.empty() && !extractor.drain()) {
        first_failure = extractor.failure();
    }
    const std::uint64_t frames_written = extractor.frames_written();
    const GpuSurfaceVerification verification = extractor.verification();
    const DomeSurfaceStats dome_stats = extractor.dome_stats();
    const double total_extraction_ms = extractor.extraction_total_ms();
    const double total_observer_ms = extractor.observer_total_ms();
    const double frame_wall_max_ms = extractor.frame_wall_max_ms();
    const double stream_wall_ms = std::chrono::duration<double, std::milli>(
        std::chrono::steady_clock::now() - stream_begin).count();
    const bool passed = first_failure.empty() || first_failure == "stream_closed";
    std::ostringstream output;
    output << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.game_surface_stream.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"authority\":\"PRESENTATION_ONLY_TOOL\""
           << ",\"lane\":\"" << lane << "\""
           << ",\"profile_id\":\"" << profile.id << "\""
           << ",\"profile_sha256\":\"" << sha256_hex(canonical_profile_json(profile)) << "\""
           << ",\"binary_sha256\":\"" << executable_hash() << "\""
           << ",\"dynamic_samples\":" << dynamic_samples
           << ",\"iterations\":" << (iterations_override > 0 ? iterations_override : profile.fixed_iterations)
           << ",\"boundary_layers\":" << boundary_layers
           << ",\"boundary_support\":\"" << boundary_support << "\""
           << ",\"boundary_lid\":" << (boundary_lid ? "true" : "false")
           << ",\"boundary_samples\":" << boundary_samples
           << ",\"requested_steps\":" << steps
           << ",\"frame_every_steps\":" << every
           << ",\"extraction_workers\":" << extractor.worker_count()
           << ",\"extractor\":\"" << extractor_name << "\""
           << ",\"surface_model\":\"" << surface_model_name << "\""
           << ",\"surface_components\":\"" << surface_components << "\""
           << ",\"retained_wet_fraction\":{\"frames\":" << dome_stats.retained_frames
           << ",\"minimum\":" << dome_stats.minimum_retained_fraction
           << ",\"mean\":" << (dome_stats.retained_frames > 0U
                  ? dome_stats.retained_fraction_sum / static_cast<double>(dome_stats.retained_frames)
                  : 0.0) << '}'
           << ",\"closing_surface\":{\"frames\":" << dome_stats.frames
           << ",\"gate_failures\":" << dome_stats.gate_failures
           << ",\"maximum_lift_p50_m\":" << dome_stats.maximum_lift_p50
           << ",\"maximum_lift_p95_m\":" << dome_stats.maximum_lift_p95
           << ",\"minimum_lift_m\":" << dome_stats.minimum_lift
           << ",\"ceiling_violations\":" << dome_stats.ceiling_violations
           << ",\"lift_p50_limit_m\":" << GAME_CLOSING_LIFT_P50_LIMIT
           << ",\"closing_radius_pixels\":" << GAME_CLOSING_RADIUS_PIXELS << '}'
           << ",\"gpu_verification\":{\"frames\":" << verification.frames
           << ",\"raw_mask_mismatch_pixels\":" << verification.raw_mask_mismatch_pixels
           << ",\"closed_mask_mismatch_pixels\":" << verification.closed_mask_mismatch_pixels
           << ",\"mesh_count_mismatches\":" << verification.mesh_count_mismatches
           << ",\"maximum_depth_difference_m\":" << verification.maximum_depth_difference
           << ",\"maximum_raw_depth_difference_m\":" << verification.maximum_raw_depth_difference
           << ",\"depth_tolerance_m\":" << GAME_SURFACE_GPU_DEPTH_TOLERANCE
           << ",\"cpu_total_ms\":" << verification.cpu_total_ms
           << ",\"gpu_total_ms\":" << verification.gpu_total_ms << '}'
           << ",\"requested_cycles\":" << cycles
           << ",\"completed_cycles\":" << completed_cycles
           << ",\"completed_steps\":" << completed_steps
           << ",\"frames_written\":" << frames_written
           << ",\"audits\":" << audits
           << ",\"audit_every_frames\":" << GAME_SURFACE_STREAM_AUDIT_FRAMES
           << ",\"maximum_degree\":" << maximum_degree
           << ",\"physics_total_ms\":" << total_physics_ms
           << ",\"execute_wall_total_ms\":" << total_execute_wall_ms
           << ",\"observer_total_ms\":" << total_observer_ms
           << ",\"extraction_total_ms\":" << total_extraction_ms
           << ",\"frame_wall_max_ms\":" << frame_wall_max_ms
           << ",\"step_wall_total_ms\":" << total_wall_ms
           << ",\"step_wall_max_ms\":" << maximum_step_wall_ms
           << ",\"stream_wall_ms\":" << stream_wall_ms
           << ",\"first_failure\":\"" << first_failure << "\""
           << ",\"simulation_feedback\":false";
    if (spill.enabled) {
        // Discharge coefficient over the first two seconds (480 steps):
        // drained volume against a free orifice at the mean head 0.45 m.
        const double opening_area = (spill.opening_y1 - spill.opening_y0)
            * (spill.opening_z1 - spill.opening_z0);
        const double drained_fraction_2s = spill_upper_at_2s >= 0.0 ? 1.0 - spill_upper_at_2s : 0.0;
        const double fluid_volume = static_cast<double>(dynamic_samples) * GAME_SPACING
            * GAME_SPACING * GAME_SPACING;
        // Under-floor pipe: mean head over the same two seconds from the
        // shelf top plus half the mean sheet depth down to the exit centre.
        const double head_reference = spill.under_floor
            ? spill.shelf_top + 0.25 * (1.0 + std::max(spill_upper_at_2s, 0.0))
                - spill.exit_centre_y()
            : 0.45;
        const double torricelli_flow = opening_area * std::sqrt(2.0 * 9.81 * head_reference);
        const double discharge_coefficient =
            torricelli_flow > 0.0 ? drained_fraction_2s * fluid_volume / 2.0 / torricelli_flow : 0.0;
        output << ",\"spill\":{\"lip\":\"" << spill_lip << "\""
               << ",\"under_floor\":" << (spill.under_floor ? "true" : "false")
               << ",\"head_reference_m\":" << head_reference
               << ",\"opening_area_m2\":" << opening_area
               << ",\"discharge_coefficient_2s\":" << discharge_coefficient
               << ",\"maximum_penetration_m\":" << spill_maximum_penetration
               << ",\"upper_fraction_final\":" << spill_upper_fraction
               << ",\"arrived_by_step_480\":" << (spill_arrived_by_480 ? "true" : "false")
               << ",\"upper_fraction_per_second\":[" << spill_drain_curve << ']'
               << ",\"maximum_exit_ratio\":" << spill_maximum_exit_ratio
               << ",\"exit_per_second\":[" << spill_exit_curve << ']'
               << ",\"fast_per_second\":[" << spill_fast_curve << ']'
               << ",\"gates\":{\"g2_penetration\":\""
               << (spill_maximum_penetration <= 1e-4 ? "PASS" : "FAIL")
               << "\",\"g3_drainage\":\"" << (spill_upper_fraction <= 0.6 ? "PASS" : "FAIL")
               << "\",\"g4_arrival\":\"" << (spill_arrived_by_480 ? "PASS" : "FAIL")
               << "\"}}";
    }
    output << ",\"device\":" << device_json() << '}';
    return {passed, output.str()};
}

CommandReport run_cuda_game_surface_prototype(const std::string& frame_prefix) {
    if (!game_frame_prefix_valid(frame_prefix)) {
        throw std::invalid_argument("frame prefix contains unsupported characters");
    }
    const Profile& profile =
        find_profile("nuv-basin-48k-analytic-contact-game-cap160.v6");
    const GameQualityBox box4k{
        {0.0, 0.0, 0.0}, {2.0, 0.75, 1.0}, {40, 15, 20}};
    const GameQualityBox box16k{
        {0.0, 0.0, 0.0}, {4.0, 0.75, 2.0}, {80, 15, 40}};
    const std::vector<Particle> control_particles = game_visual_particles(20, 20);
    const GameVisualObserverControls raw_controls =
        game_visual_observer_controls(control_particles, box4k);
    const bool raw_controls_passed = raw_controls.empty_rejected
        && raw_controls.nonfinite_rejected && raw_controls.mask_mutation_changed_root
        && raw_controls.depth_mutation_changed_root;
    GameVisualLaneResult raw4k = run_game_visual_lane(
        profile, "falling-dam-4k", 20, 20, box4k, {});
    GameVisualLaneResult raw16k;
    raw16k.id = "falling-dam-16k";
    raw16k.first_failure = "NOT_RUN_4K_REJECTED";
    if (raw_controls_passed && raw4k.apparatus_passed && raw4k.quality_passed) {
        raw16k = run_game_visual_lane(
            profile, "falling-dam-16k", 40, 40, box16k, {});
    }
    const bool parent_passed = raw_controls_passed && raw4k.apparatus_passed
        && raw4k.quality_passed && raw16k.apparatus_passed && raw16k.quality_passed;
    std::ostringstream parent_material;
    parent_material << "nextengine.nonlocal.game-visual-corpus.v1\n"
                    << raw_controls.root << '\n' << raw4k.result_root << '\n'
                    << raw16k.result_root << '\n'
                    << "ORIGINAL_GPU_DYNAMIC_VISUAL_SUPPORTED_BOUNDED\n";
    const std::string parent_result_root = sha256_hex(parent_material.str());
    const bool parent_exact = parent_passed
        && raw4k.trace_root
            == "369ac07d797d755abbee8b965e8d6fe635693534938c00cfbc684f75e5a71bbb"
        && raw4k.result_root
            == "18f48386fb1c178e8bf22cf1fa74315abf0be4f8decb31ffa15cda97a35278aa"
        && raw16k.trace_root
            == "61c45089cf57d41045d7ca1be59657b93845cc4a266173dd121ada2f84578a05"
        && raw16k.result_root
            == "8e3e9f9584f91622b274f6635452abe65230cf24b2bf24d1636e735d3fd74f85"
        && parent_result_root
            == "c2f1f6e75f1238409298cb6db6f16c2aeb0baca3ad4ebbeff216ba3c39f98274";

    PresentationSurfaceControls controls;
    if (!raw4k.frames.empty()) {
        controls = presentation_surface_controls(raw4k.frames.front(), box4k);
    }
    const bool controls_passed = controls.empty_rejected
        && controls.nonfinite_rejected && controls.mask_mutation_changed_root
        && controls.depth_mutation_changed_root;
    PresentationSurfaceLane surface4k =
        make_presentation_surface_lane(raw4k, box4k, frame_prefix);
    PresentationSurfaceLane surface16k;
    surface16k.id = "falling-dam-16k";
    surface16k.first_failure = "NOT_RUN_4K_SURFACE_REJECTED";
    if (parent_exact && controls_passed && surface4k.passed) {
        surface16k = make_presentation_surface_lane(raw16k, box16k, frame_prefix);
    }
    const bool passed = parent_exact && controls_passed
        && surface4k.passed && surface16k.passed;
    const char* semantic_status = !parent_passed || !parent_exact || !controls_passed
        ? "APPARATUS_INCONCLUSIVE"
        : (!surface4k.passed || !surface16k.passed
                ? "PRESENTATION_SURFACE_REFUTED_BOUNDED"
                : "PRESENTATION_SURFACE_SUPPORTED_BOUNDED");
    std::ostringstream root_material;
    root_material << "nextengine.nonlocal.presentation-surface-corpus.v3\n"
                  << sha256_hex(canonical_profile_json(profile)) << '\n'
                  << parent_result_root << '\n' << controls.root << '\n'
                  << surface4k.result_root << '\n' << surface16k.result_root << '\n'
                  << semantic_status << '\n';
    const std::string result_root = sha256_hex(root_material.str());

    std::ostringstream output;
    output << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.presentation_surface_corpus.v3\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"semantic_status\":\"" << semantic_status << "\""
           << ",\"claim_ceiling\":\"presentation_only_finite_frames\""
           << ",\"profile_id\":\"" << profile.id << "\""
           << ",\"profile_sha256\":\""
           << sha256_hex(canonical_profile_json(profile)) << "\""
           << ",\"binary_sha256\":\"" << executable_hash() << "\""
           << ",\"simulation_feedback\":false"
           << ",\"extraction_in_primary_gpu_timing\":false"
           << ",\"filter\":{\"component_policy\":\"largest_8_connected\""
           << ",\"binary_close\":\"3x3_one_iteration\""
           << ",\"height_filter\":\"masked_bilateral_3x3_one_pass\""
           << ",\"range_sigma_m\":" << GAME_RADIUS
           << ",\"pixel_pitch_m\":" << GAME_VISUAL_PIXEL_PITCH << "}"
           << ",\"gates\":{\"area_ratio_minimum\":0.95"
           << ",\"area_ratio_maximum\":1.40"
           << ",\"maximum_bounding_box_expansion_pixels\":1"
           << ",\"local_fill_only_required\":true"
           << ",\"common_coverage_minimum\":0.95"
           << ",\"depth_rmse_maximum_m\":0.025"
           << ",\"depth_p95_change_maximum_m\":0.05"
           << ",\"components\":1}"
           << ",\"parent\":{\"passed\":"
           << (parent_passed ? "true" : "false")
           << ",\"exact\":" << (parent_exact ? "true" : "false")
           << ",\"result_root\":\"" << parent_result_root << "\"}"
           << ",\"controls\":{\"passed\":"
           << (controls_passed ? "true" : "false")
           << ",\"empty_rejected\":"
           << (controls.empty_rejected ? "true" : "false")
           << ",\"nonfinite_rejected\":"
           << (controls.nonfinite_rejected ? "true" : "false")
           << ",\"mask_mutation_changed_root\":"
           << (controls.mask_mutation_changed_root ? "true" : "false")
           << ",\"depth_mutation_changed_root\":"
           << (controls.depth_mutation_changed_root ? "true" : "false")
           << ",\"root\":\"" << controls.root << "\"}"
           << ",\"lanes\":[";
    append_presentation_surface_lane(output, surface4k);
    output << ',';
    append_presentation_surface_lane(output, surface16k);
    output << "]"
           << ",\"result_root\":\"" << result_root << "\""
           << ",\"device\":" << device_json() << '}';
    return {passed, output.str()};
}

} // namespace nextengine::nonlocal
