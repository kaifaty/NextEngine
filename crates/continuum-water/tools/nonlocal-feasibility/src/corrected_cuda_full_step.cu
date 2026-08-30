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
    const DeviceVec3 x = current_input[input];
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
    const std::vector<Vec3d>*, NonlocalGpuVariant, bool, bool) {
    NonlocalGpuEvaluationResult result;
    result.failure = NonlocalGpuFailure::InvalidState;
    return result;
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
