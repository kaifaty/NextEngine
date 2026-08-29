#include "corrected_cuda_neighbors.hpp"

#include <cub/cub.cuh>
#include <cuda_runtime.h>

#include <algorithm>
#include <cstdint>
#include <sstream>
#include <stdexcept>
#include <string>
#include <unordered_set>
#include <vector>

namespace nextengine::nonlocal::gpu_neighbor_audit {
namespace {

constexpr long long SUPPORT_UM = 100000;
constexpr long long COORDINATE_LIMIT_UM = 1000000000;
constexpr long long CELL_BIAS = 1LL << 20;
constexpr long long CELL_MIN = -CELL_BIAS;
constexpr long long CELL_MAX = CELL_BIAS - 1;
constexpr int MAX_SAMPLES = 256;
constexpr int THREADS = 128;

struct DeviceSample {
    unsigned int sample_id;
    long long x_um;
    long long y_um;
    long long z_um;
};

struct DeviceWork {
    unsigned long long cell_probes;
    unsigned long long range_search_comparisons;
    unsigned long long distance_predicates;
    unsigned long long fill_writes;
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
        if (data_ != nullptr) {
            cudaFree(data_);
        }
    }

    void allocate(std::size_t count) {
        if (count == 0U) {
            return;
        }
        check_cuda(cudaMalloc(reinterpret_cast<void**>(&data_), count * sizeof(T)),
            "cudaMalloc NCGA1 buffer");
    }

    T* get() { return data_; }
    const T* get() const { return data_; }

private:
    T* data_ = nullptr;
};

__device__ long long cell_axis(long long coordinate, bool truncate) {
    const long long quotient = coordinate / SUPPORT_UM;
    if (truncate) {
        return quotient;
    }
    const long long remainder = coordinate % SUPPORT_UM;
    return quotient - (remainder < 0 ? 1 : 0);
}

__device__ unsigned long long pack_cell(
    long long x, long long y, long long z, int* error_flag) {
    if (x < CELL_MIN || x > CELL_MAX || y < CELL_MIN || y > CELL_MAX
        || z < CELL_MIN || z > CELL_MAX) {
        atomicExch(error_flag, 1);
        return 0ULL;
    }
    return (static_cast<unsigned long long>(x + CELL_BIAS) << 42U)
        | (static_cast<unsigned long long>(y + CELL_BIAS) << 21U)
        | static_cast<unsigned long long>(z + CELL_BIAS);
}

__global__ void compute_keys(const DeviceSample* samples,
    unsigned long long* cell_keys,
    unsigned int* owner_keys,
    unsigned int* indices,
    int count,
    unsigned int variant,
    int* error_flag) {
    const int index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= count) {
        return;
    }
    const bool truncate = variant
        == static_cast<unsigned int>(GpuNeighborVariant::TruncateSignedCell);
    const DeviceSample sample = samples[index];
    const long long x = cell_axis(sample.x_um, truncate);
    const long long y = cell_axis(sample.y_um, truncate);
    const long long z = cell_axis(sample.z_um, truncate);
    cell_keys[index] = pack_cell(x, y, z, error_flag);
    owner_keys[index] = variant
            == static_cast<unsigned int>(GpuNeighborVariant::ArrayIndexIdentity)
        ? static_cast<unsigned int>(index)
        : sample.sample_id;
    indices[index] = static_cast<unsigned int>(index);
}

__global__ void gather_cell_sort_input(const unsigned int* owner_indices,
    const unsigned long long* raw_cell_keys,
    unsigned long long* cell_keys,
    unsigned int* cell_indices,
    int count) {
    const int row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= count) {
        return;
    }
    const unsigned int input_index = owner_indices[row];
    cell_keys[row] = raw_cell_keys[input_index];
    cell_indices[row] = input_index;
}

__global__ void gather_cell_ids(const DeviceSample* samples,
    const unsigned int* cell_indices,
    unsigned int* cell_ids,
    int count,
    unsigned int variant) {
    const int slot = blockIdx.x * blockDim.x + threadIdx.x;
    if (slot >= count) {
        return;
    }
    const unsigned int input_index = cell_indices[slot];
    cell_ids[slot] = variant
            == static_cast<unsigned int>(GpuNeighborVariant::ArrayIndexIdentity)
        ? input_index
        : samples[input_index].sample_id;
}

__device__ int lower_bound_key(const unsigned long long* values,
    int count,
    unsigned long long key,
    unsigned long long& comparisons) {
    int first = 0;
    int length = count;
    while (length > 0) {
        const int half = length / 2;
        const int middle = first + half;
        ++comparisons;
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
    unsigned long long key,
    unsigned long long& comparisons) {
    int first = 0;
    int length = count;
    while (length > 0) {
        const int half = length / 2;
        const int middle = first + half;
        ++comparisons;
        if (key < values[middle]) {
            length = half;
        } else {
            first = middle + 1;
            length -= half + 1;
        }
    }
    return first;
}

__device__ unsigned long long square(long long value) {
    const unsigned long long magnitude = value < 0
        ? static_cast<unsigned long long>(-value)
        : static_cast<unsigned long long>(value);
    return magnitude * magnitude;
}

template <bool Fill>
__global__ void visit_neighbors(const DeviceSample* samples,
    const unsigned int* owner_indices,
    const unsigned long long* sorted_cell_keys,
    const unsigned int* sorted_cell_indices,
    unsigned int* counts,
    const unsigned int* offsets,
    unsigned int* neighbors,
    int count,
    unsigned int variant,
    DeviceWork* work,
    int* error_flag) {
    const int row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= count) {
        return;
    }
    const unsigned int owner_index = owner_indices[row];
    const DeviceSample owner = samples[owner_index];
    const bool truncate = variant
        == static_cast<unsigned int>(GpuNeighborVariant::TruncateSignedCell);
    const bool same_cell_only = variant
        == static_cast<unsigned int>(GpuNeighborVariant::SameCellOnly);
    const bool strict = variant
        == static_cast<unsigned int>(GpuNeighborVariant::StrictRadius);
    const bool array_identity = variant
        == static_cast<unsigned int>(GpuNeighborVariant::ArrayIndexIdentity);
    const long long own_x = cell_axis(owner.x_um, truncate);
    const long long own_y = cell_axis(owner.y_um, truncate);
    const long long own_z = cell_axis(owner.z_um, truncate);
    unsigned int local_ids[MAX_SAMPLES];
    int degree = 0;
    unsigned long long local_cell_probes = 0ULL;
    unsigned long long local_range_comparisons = 0ULL;
    unsigned long long local_distance_predicates = 0ULL;

    const int radius = same_cell_only ? 0 : 1;
    for (int dz = -radius; dz <= radius; ++dz) {
        for (int dy = -radius; dy <= radius; ++dy) {
            for (int dx = -radius; dx <= radius; ++dx) {
                ++local_cell_probes;
                const unsigned long long key = pack_cell(
                    own_x + dx, own_y + dy, own_z + dz, error_flag);
                const int begin = lower_bound_key(
                    sorted_cell_keys, count, key, local_range_comparisons);
                const int end = upper_bound_key(
                    sorted_cell_keys, count, key, local_range_comparisons);
                for (int slot = begin; slot < end; ++slot) {
                    ++local_distance_predicates;
                    const unsigned int candidate_index = sorted_cell_indices[slot];
                    const DeviceSample candidate = samples[candidate_index];
                    const unsigned long long squared =
                        square(owner.x_um - candidate.x_um)
                        + square(owner.y_um - candidate.y_um)
                        + square(owner.z_um - candidate.z_um);
                    const unsigned long long limit = static_cast<unsigned long long>(
                        SUPPORT_UM * SUPPORT_UM);
                    if ((strict && squared < limit) || (!strict && squared <= limit)) {
                        if constexpr (Fill) {
                            if (degree >= MAX_SAMPLES) {
                                atomicExch(error_flag, 1);
                                continue;
                            }
                            local_ids[degree] = array_identity
                                ? candidate_index
                                : candidate.sample_id;
                        }
                        ++degree;
                    }
                }
            }
        }
    }

    if constexpr (Fill) {
        for (int index = 1; index < degree; ++index) {
            const unsigned int value = local_ids[index];
            int cursor = index;
            while (cursor > 0 && local_ids[cursor - 1] > value) {
                local_ids[cursor] = local_ids[cursor - 1];
                --cursor;
            }
            local_ids[cursor] = value;
        }
        const unsigned int begin = offsets[row];
        const unsigned int end = offsets[row + 1];
        if (end - begin != static_cast<unsigned int>(degree)) {
            atomicExch(error_flag, 1);
        } else {
            for (int index = 0; index < degree; ++index) {
                neighbors[begin + static_cast<unsigned int>(index)] = local_ids[index];
            }
            atomicAdd(&work->fill_writes,
                static_cast<unsigned long long>(degree));
        }
    } else {
        counts[row] = static_cast<unsigned int>(degree);
    }
    atomicAdd(&work->cell_probes, local_cell_probes);
    atomicAdd(&work->range_search_comparisons, local_range_comparisons);
    atomicAdd(&work->distance_predicates, local_distance_predicates);
}

__global__ void finish_offsets(
    const unsigned int* counts, unsigned int* offsets, int count) {
    if (blockIdx.x == 0 && threadIdx.x == 0) {
        offsets[count] = offsets[count - 1] + counts[count - 1];
    }
}

bool input_admitted(const std::vector<CanonicalSample>& samples,
    NeighborFailure& failure) {
    if (samples.empty() || samples.size() > static_cast<std::size_t>(MAX_SAMPLES)) {
        failure = samples.empty() ? NeighborFailure::InvalidInput
                                  : NeighborFailure::CapacityExceeded;
        return false;
    }
    std::unordered_set<std::uint32_t> ids;
    ids.reserve(samples.size());
    for (const CanonicalSample& sample : samples) {
        if (sample.x_um < -COORDINATE_LIMIT_UM
            || sample.x_um > COORDINATE_LIMIT_UM
            || sample.y_um < -COORDINATE_LIMIT_UM
            || sample.y_um > COORDINATE_LIMIT_UM
            || sample.z_um < -COORDINATE_LIMIT_UM
            || sample.z_um > COORDINATE_LIMIT_UM) {
            failure = NeighborFailure::InvalidInput;
            return false;
        }
        if (!ids.insert(sample.sample_id).second) {
            failure = NeighborFailure::DuplicateSampleId;
            return false;
        }
    }
    return true;
}

} // namespace

NeighborhoodResult build_gpu_neighborhood(
    const std::vector<CanonicalSample>& samples,
    GpuNeighborVariant variant) {
    NeighborhoodResult result;
    if (!input_admitted(samples, result.failure)) {
        return result;
    }
    const int count = static_cast<int>(samples.size());
    std::vector<DeviceSample> host_samples;
    host_samples.reserve(samples.size());
    for (const CanonicalSample& sample : samples) {
        host_samples.push_back({sample.sample_id,
            static_cast<long long>(sample.x_um),
            static_cast<long long>(sample.y_um),
            static_cast<long long>(sample.z_um)});
    }

    DeviceBuffer<DeviceSample> device_samples(samples.size());
    DeviceBuffer<unsigned long long> raw_cell_keys(samples.size());
    DeviceBuffer<unsigned int> owner_keys_input(samples.size());
    DeviceBuffer<unsigned int> owner_keys_sorted(samples.size());
    DeviceBuffer<unsigned int> indices_input(samples.size());
    DeviceBuffer<unsigned int> owner_indices(samples.size());
    DeviceBuffer<unsigned long long> cell_keys_input(samples.size());
    DeviceBuffer<unsigned long long> cell_keys_sorted(samples.size());
    DeviceBuffer<unsigned int> cell_indices_input(samples.size());
    DeviceBuffer<unsigned int> cell_indices_sorted(samples.size());
    DeviceBuffer<unsigned int> cell_ids_sorted(samples.size());
    DeviceBuffer<unsigned int> counts(samples.size());
    DeviceBuffer<unsigned int> offsets(samples.size() + 1U);
    DeviceBuffer<unsigned int> neighbors(samples.size() * samples.size());
    DeviceBuffer<DeviceWork> work(1U);
    DeviceBuffer<int> error_flag(1U);

    check_cuda(cudaMemcpy(device_samples.get(), host_samples.data(),
                   host_samples.size() * sizeof(DeviceSample), cudaMemcpyHostToDevice),
        "upload NCGA1 samples");
    check_cuda(cudaMemset(work.get(), 0, sizeof(DeviceWork)), "reset NCGA1 work");
    check_cuda(cudaMemset(error_flag.get(), 0, sizeof(int)), "reset NCGA1 error");
    compute_keys<<<blocks_for(samples.size()), THREADS>>>(device_samples.get(),
        raw_cell_keys.get(), owner_keys_input.get(), indices_input.get(), count,
        static_cast<unsigned int>(variant), error_flag.get());
    check_cuda(cudaGetLastError(), "launch NCGA1 key computation");

    std::size_t owner_sort_bytes = 0U;
    check_cuda(cub::DeviceRadixSort::SortPairs(nullptr, owner_sort_bytes,
                   owner_keys_input.get(), owner_keys_sorted.get(), indices_input.get(),
                   owner_indices.get(), count),
        "query NCGA1 owner sort");
    DeviceBuffer<unsigned char> owner_sort_storage(owner_sort_bytes);
    check_cuda(cub::DeviceRadixSort::SortPairs(owner_sort_storage.get(), owner_sort_bytes,
                   owner_keys_input.get(), owner_keys_sorted.get(), indices_input.get(),
                   owner_indices.get(), count),
        "execute NCGA1 owner sort");

    gather_cell_sort_input<<<blocks_for(samples.size()), THREADS>>>(
        owner_indices.get(), raw_cell_keys.get(), cell_keys_input.get(),
        cell_indices_input.get(), count);
    check_cuda(cudaGetLastError(), "launch NCGA1 cell sort gather");
    std::size_t cell_sort_bytes = 0U;
    check_cuda(cub::DeviceRadixSort::SortPairs(nullptr, cell_sort_bytes,
                   cell_keys_input.get(), cell_keys_sorted.get(), cell_indices_input.get(),
                   cell_indices_sorted.get(), count),
        "query NCGA1 cell sort");
    DeviceBuffer<unsigned char> cell_sort_storage(cell_sort_bytes);
    check_cuda(cub::DeviceRadixSort::SortPairs(cell_sort_storage.get(), cell_sort_bytes,
                   cell_keys_input.get(), cell_keys_sorted.get(), cell_indices_input.get(),
                   cell_indices_sorted.get(), count),
        "execute NCGA1 cell sort");
    gather_cell_ids<<<blocks_for(samples.size()), THREADS>>>(device_samples.get(),
        cell_indices_sorted.get(), cell_ids_sorted.get(), count,
        static_cast<unsigned int>(variant));
    check_cuda(cudaGetLastError(), "launch NCGA1 cell id gather");

    visit_neighbors<false><<<blocks_for(samples.size()), THREADS>>>(
        device_samples.get(), owner_indices.get(), cell_keys_sorted.get(),
        cell_indices_sorted.get(), counts.get(), nullptr, nullptr, count,
        static_cast<unsigned int>(variant), work.get(), error_flag.get());
    check_cuda(cudaGetLastError(), "launch NCGA1 neighbor count");
    std::size_t scan_bytes = 0U;
    check_cuda(cub::DeviceScan::ExclusiveSum(nullptr, scan_bytes, counts.get(),
                   offsets.get(), count),
        "query NCGA1 scan");
    DeviceBuffer<unsigned char> scan_storage(scan_bytes);
    check_cuda(cub::DeviceScan::ExclusiveSum(scan_storage.get(), scan_bytes,
                   counts.get(), offsets.get(), count),
        "execute NCGA1 scan");
    finish_offsets<<<1, 1>>>(counts.get(), offsets.get(), count);
    check_cuda(cudaGetLastError(), "launch NCGA1 offset finish");
    visit_neighbors<true><<<blocks_for(samples.size()), THREADS>>>(
        device_samples.get(), owner_indices.get(), cell_keys_sorted.get(),
        cell_indices_sorted.get(), nullptr, offsets.get(), neighbors.get(), count,
        static_cast<unsigned int>(variant), work.get(), error_flag.get());
    check_cuda(cudaGetLastError(), "launch NCGA1 neighbor fill");
    check_cuda(cudaDeviceSynchronize(), "synchronize NCGA1 candidate");

    int device_error = 0;
    check_cuda(cudaMemcpy(&device_error, error_flag.get(), sizeof(int),
                   cudaMemcpyDeviceToHost),
        "copy NCGA1 error");
    if (device_error != 0) {
        result.failure = NeighborFailure::DeviceFailure;
        return result;
    }

    std::vector<unsigned long long> host_cell_keys(samples.size());
    std::vector<unsigned int> host_cell_ids(samples.size());
    result.owner_ids.resize(samples.size());
    result.offsets.resize(samples.size() + 1U);
    check_cuda(cudaMemcpy(host_cell_keys.data(), cell_keys_sorted.get(),
                   host_cell_keys.size() * sizeof(unsigned long long),
                   cudaMemcpyDeviceToHost),
        "copy NCGA1 cell keys");
    check_cuda(cudaMemcpy(host_cell_ids.data(), cell_ids_sorted.get(),
                   host_cell_ids.size() * sizeof(unsigned int), cudaMemcpyDeviceToHost),
        "copy NCGA1 cell ids");
    check_cuda(cudaMemcpy(result.owner_ids.data(), owner_keys_sorted.get(),
                   result.owner_ids.size() * sizeof(std::uint32_t), cudaMemcpyDeviceToHost),
        "copy NCGA1 owner ids");
    check_cuda(cudaMemcpy(result.offsets.data(), offsets.get(),
                   result.offsets.size() * sizeof(std::uint32_t), cudaMemcpyDeviceToHost),
        "copy NCGA1 offsets");
    const std::size_t pair_count = result.offsets.back();
    if (pair_count > samples.size() * samples.size()) {
        result.failure = NeighborFailure::CapacityExceeded;
        result.cells.clear();
        result.owner_ids.clear();
        result.offsets.clear();
        return result;
    }
    result.neighbor_ids.resize(pair_count);
    if (pair_count != 0U) {
        check_cuda(cudaMemcpy(result.neighbor_ids.data(), neighbors.get(),
                       pair_count * sizeof(std::uint32_t), cudaMemcpyDeviceToHost),
            "copy NCGA1 neighbors");
    }
    result.cells.reserve(samples.size());
    for (std::size_t index = 0; index < samples.size(); ++index) {
        result.cells.push_back(
            {static_cast<std::uint64_t>(host_cell_keys[index]), host_cell_ids[index]});
    }
    DeviceWork host_work{};
    check_cuda(cudaMemcpy(&host_work, work.get(), sizeof(DeviceWork),
                   cudaMemcpyDeviceToHost),
        "copy NCGA1 work");
    result.work.key_evaluations = samples.size();
    result.work.radix_sort_items = samples.size();
    result.work.owner_sort_items = samples.size();
    result.work.cell_probes = host_work.cell_probes;
    result.work.range_search_comparisons = host_work.range_search_comparisons;
    result.work.distance_predicates = host_work.distance_predicates;
    result.work.count_writes = samples.size();
    result.work.fill_writes = host_work.fill_writes;
    result.work.emitted_directed_pairs = pair_count;
    result.failure = NeighborFailure::None;
    return result;
}

std::string gpu_neighbor_environment_json() {
    int device = 0;
    check_cuda(cudaGetDevice(&device), "cudaGetDevice NCGA1");
    cudaDeviceProp properties{};
    check_cuda(cudaGetDeviceProperties(&properties, device),
        "cudaGetDeviceProperties NCGA1");
    int driver = 0;
    int runtime = 0;
    check_cuda(cudaDriverGetVersion(&driver), "cudaDriverGetVersion NCGA1");
    check_cuda(cudaRuntimeGetVersion(&runtime), "cudaRuntimeGetVersion NCGA1");
    std::ostringstream output;
    output << "{\"name\":\"" << properties.name << "\",\"major\":"
           << properties.major << ",\"minor\":" << properties.minor
           << ",\"driver\":" << driver << ",\"runtime\":" << runtime
           << ",\"integer_membership\":true,\"cub_radix_sort\":true}";
    return output.str();
}

} // namespace nextengine::nonlocal::gpu_neighbor_audit
