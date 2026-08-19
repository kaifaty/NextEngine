#include <cub/cub.cuh>
#include <cuda_runtime.h>

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <cstdlib>
#include <iomanip>
#include <iostream>
#include <limits>
#include <stdexcept>
#include <string>
#include <type_traits>
#include <utility>
#include <vector>

namespace {

constexpr int kFluidX = 80;
constexpr int kFluidY = 15;
constexpr int kFluidZ = 40;
constexpr int kBasinX = 80;
constexpr int kBasinY = 20;
constexpr int kBasinZ = 40;
constexpr int kExteriorLayers = 2;
constexpr int kFluidCount = kFluidX * kFluidY * kFluidZ;
constexpr int kExpectedBoundaryCount = 24704;
constexpr std::uint32_t kExpectedFluidEdges = 1427612;
constexpr std::uint32_t kExpectedBoundaryEdges = 73904;
constexpr std::int64_t kSpacingUm = 50000;
constexpr std::int64_t kRadiusUm = 100000;
constexpr std::int64_t kRadiusSquaredUm = kRadiusUm * kRadiusUm;
constexpr int kBlockSize = 256;

constexpr int kCellMinX = -21;
constexpr int kCellMinY = -1;
constexpr int kCellMinZ = -11;
constexpr int kCellCountX = 42;
constexpr int kCellCountY = 12;
constexpr int kCellCountZ = 22;
constexpr int kCellCount = kCellCountX * kCellCountY * kCellCountZ;

constexpr double kRestVolume = 0.000125;
constexpr double kSupportRadius = 0.1;
constexpr double kPi = 3.14159265358979323846264338327950288;
constexpr double kKernelL = 48.0 / (kPi * kSupportRadius * kSupportRadius * kSupportRadius);
constexpr double kDt = 1.0 / 240.0;
constexpr double kDtSquared = kDt * kDt;
constexpr double kProjectedGradientStep = 0.25;
constexpr double kSolverEpsilon = 1.0e-5;

struct Options {
    int warmup = 5;
    int runs = 20;
    int iterations = 40;
    bool self_test = false;
};

struct Int3 {
    std::int32_t x;
    std::int32_t y;
    std::int32_t z;
};

template <typename T>
struct DeviceBuffer {
    DeviceBuffer() = default;
    explicit DeviceBuffer(std::size_t count) { allocate(count); }
    DeviceBuffer(const DeviceBuffer&) = delete;
    DeviceBuffer& operator=(const DeviceBuffer&) = delete;
    DeviceBuffer(DeviceBuffer&& other) noexcept : pointer(other.pointer), count(other.count) {
        other.pointer = nullptr;
        other.count = 0;
    }
    DeviceBuffer& operator=(DeviceBuffer&& other) noexcept {
        if (this != &other) {
            release();
            pointer = other.pointer;
            count = other.count;
            other.pointer = nullptr;
            other.count = 0;
        }
        return *this;
    }
    ~DeviceBuffer() { release(); }

    void allocate(std::size_t next_count) {
        release();
        count = next_count;
        if (count != 0) {
            check(cudaMalloc(reinterpret_cast<void**>(&pointer), sizeof(T) * count), "cudaMalloc");
        }
    }

    void release() {
        if (pointer != nullptr) {
            cudaFree(pointer);
            pointer = nullptr;
        }
        count = 0;
    }

    T* get() { return pointer; }
    const T* get() const { return pointer; }

    static void check(cudaError_t result, const char* operation) {
        if (result != cudaSuccess) {
            throw std::runtime_error(std::string(operation) + ": " + cudaGetErrorString(result));
        }
    }

    T* pointer = nullptr;
    std::size_t count = 0;
};

void check_cuda(cudaError_t result, const char* operation) {
    DeviceBuffer<std::uint8_t>::check(result, operation);
}

void check_launch(const char* operation) {
    check_cuda(cudaGetLastError(), operation);
}

class EventTimer {
public:
    EventTimer() {
        check_cuda(cudaEventCreate(&start_), "cudaEventCreate(start)");
        check_cuda(cudaEventCreate(&stop_), "cudaEventCreate(stop)");
    }
    EventTimer(const EventTimer&) = delete;
    EventTimer& operator=(const EventTimer&) = delete;
    ~EventTimer() {
        cudaEventDestroy(start_);
        cudaEventDestroy(stop_);
    }

    template <typename Function>
    float measure(Function&& function) {
        check_cuda(cudaEventRecord(start_), "cudaEventRecord(start)");
        function();
        check_cuda(cudaEventRecord(stop_), "cudaEventRecord(stop)");
        check_cuda(cudaEventSynchronize(stop_), "cudaEventSynchronize(stop)");
        float milliseconds = 0.0F;
        check_cuda(cudaEventElapsedTime(&milliseconds, start_, stop_), "cudaEventElapsedTime");
        return milliseconds;
    }

private:
    cudaEvent_t start_{};
    cudaEvent_t stop_{};
};

int blocks_for(std::size_t count) {
    return static_cast<int>((count + kBlockSize - 1) / kBlockSize);
}

Int3 lattice_position(int ix, int iy, int iz) {
    return Int3{
        static_cast<std::int32_t>(-2000000 + 25000 + ix * kSpacingUm),
        static_cast<std::int32_t>(25000 + iy * kSpacingUm),
        static_cast<std::int32_t>(-1000000 + 25000 + iz * kSpacingUm),
    };
}

std::vector<Int3> make_fluid_positions() {
    std::vector<Int3> result;
    result.reserve(kFluidCount);
    for (int ix = 0; ix < kFluidX; ++ix) {
        for (int iy = 0; iy < kFluidY; ++iy) {
            for (int iz = 0; iz < kFluidZ; ++iz) {
                result.push_back(lattice_position(ix, iy, iz));
            }
        }
    }
    return result;
}

std::vector<Int3> make_boundary_positions() {
    std::vector<Int3> result;
    result.reserve(kExpectedBoundaryCount);
    for (int ix = -kExteriorLayers; ix <= kBasinX + kExteriorLayers - 1; ++ix) {
        for (int iy = -kExteriorLayers; iy <= kBasinY + kExteriorLayers - 1; ++iy) {
            for (int iz = -kExteriorLayers; iz <= kBasinZ + kExteriorLayers - 1; ++iz) {
                const bool outside = ix < 0 || ix >= kBasinX || iy < 0 || iy >= kBasinY ||
                                     iz < 0 || iz >= kBasinZ;
                if (outside) {
                    result.push_back(lattice_position(ix, iy, iz));
                }
            }
        }
    }
    if (result.size() != kExpectedBoundaryCount) {
        throw std::runtime_error("boundary lattice count mismatch");
    }
    return result;
}

__device__ int device_floor_div(std::int32_t value, std::int32_t divisor) {
    int quotient = value / divisor;
    int remainder = value % divisor;
    if (remainder != 0 && value < 0) {
        --quotient;
    }
    return quotient;
}

__host__ __device__ int cell_key_from_coordinates(int x, int y, int z) {
    if (x < kCellMinX || x >= kCellMinX + kCellCountX || y < kCellMinY ||
        y >= kCellMinY + kCellCountY || z < kCellMinZ ||
        z >= kCellMinZ + kCellCountZ) {
        return -1;
    }
    const int local_x = x - kCellMinX;
    const int local_y = y - kCellMinY;
    const int local_z = z - kCellMinZ;
    return (local_x * kCellCountY + local_y) * kCellCountZ + local_z;
}

__global__ void initialize_entries(
    const Int3* positions,
    std::uint32_t count,
    std::uint32_t* keys,
    std::uint32_t* ids) {
    const std::uint32_t index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= count) {
        return;
    }
    const Int3 position = positions[index];
    const int cell_x = device_floor_div(position.x, static_cast<std::int32_t>(kRadiusUm));
    const int cell_y = device_floor_div(position.y, static_cast<std::int32_t>(kRadiusUm));
    const int cell_z = device_floor_div(position.z, static_cast<std::int32_t>(kRadiusUm));
    const int key = cell_key_from_coordinates(cell_x, cell_y, cell_z);
    keys[index] = static_cast<std::uint32_t>(key);
    ids[index] = index;
}

__global__ void mark_cell_ranges(
    const std::uint32_t* sorted_keys,
    std::uint32_t count,
    std::int32_t* starts,
    std::int32_t* ends) {
    const std::uint32_t index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= count) {
        return;
    }
    const std::uint32_t key = sorted_keys[index];
    if (index == 0 || sorted_keys[index - 1] != key) {
        starts[key] = static_cast<std::int32_t>(index);
    }
    if (index + 1 == count || sorted_keys[index + 1] != key) {
        ends[key] = static_cast<std::int32_t>(index + 1);
    }
}

__device__ bool inside_support(Int3 left, Int3 right) {
    const std::int64_t dx = static_cast<std::int64_t>(left.x) - right.x;
    const std::int64_t dy = static_cast<std::int64_t>(left.y) - right.y;
    const std::int64_t dz = static_cast<std::int64_t>(left.z) - right.z;
    return dx * dx + dy * dy + dz * dz <= kRadiusSquaredUm;
}

template <bool Boundary>
__global__ void count_neighbors(
    const Int3* fluid_positions,
    std::uint32_t fluid_count,
    const Int3* other_positions,
    const std::uint32_t* sorted_ids,
    const std::int32_t* starts,
    const std::int32_t* ends,
    std::uint32_t* counts) {
    const std::uint32_t row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= fluid_count) {
        return;
    }
    const Int3 position = fluid_positions[row];
    const int centre_x = device_floor_div(position.x, static_cast<std::int32_t>(kRadiusUm));
    const int centre_y = device_floor_div(position.y, static_cast<std::int32_t>(kRadiusUm));
    const int centre_z = device_floor_div(position.z, static_cast<std::int32_t>(kRadiusUm));
    std::uint32_t count = 0;
    for (int dz = -1; dz <= 1; ++dz) {
        for (int dy = -1; dy <= 1; ++dy) {
            for (int dx = -1; dx <= 1; ++dx) {
                const int key = cell_key_from_coordinates(centre_x + dx, centre_y + dy, centre_z + dz);
                if (key < 0) {
                    continue;
                }
                const std::int32_t begin = starts[key];
                if (begin < 0) {
                    continue;
                }
                const std::int32_t end = ends[key];
                for (std::int32_t cursor = begin; cursor < end; ++cursor) {
                    const std::uint32_t other = sorted_ids[cursor];
                    if (!Boundary && other == row) {
                        continue;
                    }
                    if (inside_support(position, other_positions[other])) {
                        ++count;
                    }
                }
            }
        }
    }
    counts[row] = count;
}

template <typename Scalar>
__device__ void sample_gradient(Int3 left, Int3 right, Scalar& gx, Scalar& gy, Scalar& gz) {
    const Scalar dx = static_cast<Scalar>(static_cast<double>(left.x - right.x) / 1000000.0);
    const Scalar dy = static_cast<Scalar>(static_cast<double>(left.y - right.y) / 1000000.0);
    const Scalar dz = static_cast<Scalar>(static_cast<double>(left.z - right.z) / 1000000.0);
    const Scalar r = sqrt(dx * dx + dy * dy + dz * dz);
    if (r == Scalar(0)) {
        gx = Scalar(0);
        gy = Scalar(0);
        gz = Scalar(0);
        return;
    }
    const Scalar q = r / static_cast<Scalar>(kSupportRadius);
    Scalar coefficient;
    if (q <= Scalar(0.5)) {
        coefficient = static_cast<Scalar>(kKernelL) * q * (Scalar(3) * q - Scalar(2));
    } else {
        const Scalar t = Scalar(1) - q;
        coefficient = -static_cast<Scalar>(kKernelL) * t * t;
    }
    const Scalar inverse = Scalar(1) / (r * static_cast<Scalar>(kSupportRadius));
    gx = dx * inverse * coefficient;
    gy = dy * inverse * coefficient;
    gz = dz * inverse * coefficient;
}

template <typename Scalar, bool Boundary>
__global__ void fill_neighbors(
    const Int3* fluid_positions,
    std::uint32_t fluid_count,
    const Int3* other_positions,
    const std::uint32_t* sorted_ids,
    const std::int32_t* starts,
    const std::int32_t* ends,
    const std::uint32_t* offsets,
    std::uint32_t* other_indices,
    Scalar* gradient_x,
    Scalar* gradient_y,
    Scalar* gradient_z) {
    const std::uint32_t row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= fluid_count) {
        return;
    }
    const Int3 position = fluid_positions[row];
    const int centre_x = device_floor_div(position.x, static_cast<std::int32_t>(kRadiusUm));
    const int centre_y = device_floor_div(position.y, static_cast<std::int32_t>(kRadiusUm));
    const int centre_z = device_floor_div(position.z, static_cast<std::int32_t>(kRadiusUm));
    std::uint32_t write = offsets[row];
    for (int dz = -1; dz <= 1; ++dz) {
        for (int dy = -1; dy <= 1; ++dy) {
            for (int dx = -1; dx <= 1; ++dx) {
                const int key = cell_key_from_coordinates(centre_x + dx, centre_y + dy, centre_z + dz);
                if (key < 0) {
                    continue;
                }
                const std::int32_t begin = starts[key];
                if (begin < 0) {
                    continue;
                }
                const std::int32_t end = ends[key];
                for (std::int32_t cursor = begin; cursor < end; ++cursor) {
                    const std::uint32_t other = sorted_ids[cursor];
                    if (!Boundary && other == row) {
                        continue;
                    }
                    const Int3 other_position = other_positions[other];
                    if (!inside_support(position, other_position)) {
                        continue;
                    }
                    other_indices[write] = other;
                    sample_gradient(position, other_position, gradient_x[write], gradient_y[write], gradient_z[write]);
                    ++write;
                }
            }
        }
    }
}

struct GridWorkspace {
    DeviceBuffer<std::uint32_t> input_keys;
    DeviceBuffer<std::uint32_t> input_ids;
    DeviceBuffer<std::uint32_t> sorted_keys;
    DeviceBuffer<std::uint32_t> sorted_ids;
    DeviceBuffer<std::int32_t> starts;
    DeviceBuffer<std::int32_t> ends;
    DeviceBuffer<std::uint8_t> sort_scratch;
    std::size_t sort_scratch_bytes = 0;

    explicit GridWorkspace(std::size_t count)
        : input_keys(count),
          input_ids(count),
          sorted_keys(count),
          sorted_ids(count),
          starts(kCellCount),
          ends(kCellCount) {
        check_cuda(
            cub::DeviceRadixSort::SortPairs(
                nullptr,
                sort_scratch_bytes,
                input_keys.get(),
                sorted_keys.get(),
                input_ids.get(),
                sorted_ids.get(),
                count),
            "cub radix-sort sizing");
        sort_scratch.allocate(sort_scratch_bytes);
    }
};

template <typename Scalar>
struct NeighborGraph {
    DeviceBuffer<std::uint32_t> fluid_offsets{kFluidCount + 1};
    DeviceBuffer<std::uint32_t> boundary_offsets{kFluidCount + 1};
    DeviceBuffer<std::uint32_t> fluid_other;
    DeviceBuffer<std::uint32_t> boundary_other;
    DeviceBuffer<Scalar> fluid_gx;
    DeviceBuffer<Scalar> fluid_gy;
    DeviceBuffer<Scalar> fluid_gz;
    DeviceBuffer<Scalar> boundary_gx;
    DeviceBuffer<Scalar> boundary_gy;
    DeviceBuffer<Scalar> boundary_gz;
    std::uint32_t fluid_edges = 0;
    std::uint32_t boundary_edges = 0;

    void allocate_edges(std::uint32_t next_fluid_edges, std::uint32_t next_boundary_edges) {
        fluid_edges = next_fluid_edges;
        boundary_edges = next_boundary_edges;
        fluid_other.allocate(fluid_edges);
        boundary_other.allocate(boundary_edges);
        fluid_gx.allocate(fluid_edges);
        fluid_gy.allocate(fluid_edges);
        fluid_gz.allocate(fluid_edges);
        boundary_gx.allocate(boundary_edges);
        boundary_gy.allocate(boundary_edges);
        boundary_gz.allocate(boundary_edges);
    }
};

struct ReconstructionWorkspace {
    GridWorkspace fluid_grid{kFluidCount};
    GridWorkspace boundary_grid{kExpectedBoundaryCount};
    DeviceBuffer<std::uint32_t> fluid_counts{kFluidCount + 1};
    DeviceBuffer<std::uint32_t> boundary_counts{kFluidCount + 1};
    DeviceBuffer<std::uint8_t> scan_scratch;
    std::size_t scan_scratch_bytes = 0;

    ReconstructionWorkspace() {
        std::size_t fluid_bytes = 0;
        std::size_t boundary_bytes = 0;
        check_cuda(
            cub::DeviceScan::ExclusiveSum(
                nullptr,
                fluid_bytes,
                fluid_counts.get(),
                static_cast<std::uint32_t*>(nullptr),
                kFluidCount + 1),
            "cub fluid scan sizing");
        check_cuda(
            cub::DeviceScan::ExclusiveSum(
                nullptr,
                boundary_bytes,
                boundary_counts.get(),
                static_cast<std::uint32_t*>(nullptr),
                kFluidCount + 1),
            "cub boundary scan sizing");
        scan_scratch_bytes = std::max(fluid_bytes, boundary_bytes);
        scan_scratch.allocate(scan_scratch_bytes);
    }
};

void build_grid(const Int3* positions, std::uint32_t count, GridWorkspace& grid) {
    initialize_entries<<<blocks_for(count), kBlockSize>>>(
        positions, count, grid.input_keys.get(), grid.input_ids.get());
    check_launch("initialize_entries");
    check_cuda(
        cub::DeviceRadixSort::SortPairs(
            grid.sort_scratch.get(),
            grid.sort_scratch_bytes,
            grid.input_keys.get(),
            grid.sorted_keys.get(),
            grid.input_ids.get(),
            grid.sorted_ids.get(),
            count),
        "cub radix-sort");
    check_cuda(cudaMemset(grid.starts.get(), 0xff, sizeof(std::int32_t) * kCellCount), "clear cell starts");
    check_cuda(cudaMemset(grid.ends.get(), 0xff, sizeof(std::int32_t) * kCellCount), "clear cell ends");
    mark_cell_ranges<<<blocks_for(count), kBlockSize>>>(
        grid.sorted_keys.get(), count, grid.starts.get(), grid.ends.get());
    check_launch("mark_cell_ranges");
}

template <typename Scalar>
void reconstruct(
    const DeviceBuffer<Int3>& fluid_positions,
    const DeviceBuffer<Int3>& boundary_positions,
    ReconstructionWorkspace& workspace,
    NeighborGraph<Scalar>& graph,
    bool allocate) {
    build_grid(fluid_positions.get(), kFluidCount, workspace.fluid_grid);
    build_grid(boundary_positions.get(), kExpectedBoundaryCount, workspace.boundary_grid);

    check_cuda(cudaMemset(workspace.fluid_counts.get(), 0, sizeof(std::uint32_t) * (kFluidCount + 1)), "clear fluid counts");
    check_cuda(cudaMemset(workspace.boundary_counts.get(), 0, sizeof(std::uint32_t) * (kFluidCount + 1)), "clear boundary counts");
    count_neighbors<false><<<blocks_for(kFluidCount), kBlockSize>>>(
        fluid_positions.get(),
        kFluidCount,
        fluid_positions.get(),
        workspace.fluid_grid.sorted_ids.get(),
        workspace.fluid_grid.starts.get(),
        workspace.fluid_grid.ends.get(),
        workspace.fluid_counts.get());
    count_neighbors<true><<<blocks_for(kFluidCount), kBlockSize>>>(
        fluid_positions.get(),
        kFluidCount,
        boundary_positions.get(),
        workspace.boundary_grid.sorted_ids.get(),
        workspace.boundary_grid.starts.get(),
        workspace.boundary_grid.ends.get(),
        workspace.boundary_counts.get());
    check_launch("count_neighbors");

    check_cuda(
        cub::DeviceScan::ExclusiveSum(
            workspace.scan_scratch.get(),
            workspace.scan_scratch_bytes,
            workspace.fluid_counts.get(),
            graph.fluid_offsets.get(),
            kFluidCount + 1),
        "cub fluid exclusive scan");
    check_cuda(
        cub::DeviceScan::ExclusiveSum(
            workspace.scan_scratch.get(),
            workspace.scan_scratch_bytes,
            workspace.boundary_counts.get(),
            graph.boundary_offsets.get(),
            kFluidCount + 1),
        "cub boundary exclusive scan");

    if (allocate) {
        std::uint32_t fluid_edges = 0;
        std::uint32_t boundary_edges = 0;
        check_cuda(
            cudaMemcpy(
                &fluid_edges,
                graph.fluid_offsets.get() + kFluidCount,
                sizeof(fluid_edges),
                cudaMemcpyDeviceToHost),
            "copy fluid edge count");
        check_cuda(
            cudaMemcpy(
                &boundary_edges,
                graph.boundary_offsets.get() + kFluidCount,
                sizeof(boundary_edges),
                cudaMemcpyDeviceToHost),
            "copy boundary edge count");
        graph.allocate_edges(fluid_edges, boundary_edges);
    }

    fill_neighbors<Scalar, false><<<blocks_for(kFluidCount), kBlockSize>>>(
        fluid_positions.get(),
        kFluidCount,
        fluid_positions.get(),
        workspace.fluid_grid.sorted_ids.get(),
        workspace.fluid_grid.starts.get(),
        workspace.fluid_grid.ends.get(),
        graph.fluid_offsets.get(),
        graph.fluid_other.get(),
        graph.fluid_gx.get(),
        graph.fluid_gy.get(),
        graph.fluid_gz.get());
    fill_neighbors<Scalar, true><<<blocks_for(kFluidCount), kBlockSize>>>(
        fluid_positions.get(),
        kFluidCount,
        boundary_positions.get(),
        workspace.boundary_grid.sorted_ids.get(),
        workspace.boundary_grid.starts.get(),
        workspace.boundary_grid.ends.get(),
        graph.boundary_offsets.get(),
        graph.boundary_other.get(),
        graph.boundary_gx.get(),
        graph.boundary_gy.get(),
        graph.boundary_gz.get());
    check_launch("fill_neighbors");
}

template <typename Scalar>
__global__ void initialize_solve_vectors(
    Scalar* scale,
    Scalar* right_hand_side,
    Scalar* previous,
    Scalar* iterate,
    Scalar* previous_gradient,
    Scalar* gradient,
    Scalar* velocity_x,
    Scalar* velocity_y,
    Scalar* velocity_z,
    std::uint32_t count) {
    const std::uint32_t index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= count) {
        return;
    }
    const Scalar rhs = static_cast<Scalar>(0.00001 * (1.0 + static_cast<double>(index % 7)));
    scale[index] = Scalar(1);
    right_hand_side[index] = rhs;
    previous[index] = Scalar(0);
    iterate[index] = Scalar(0);
    previous_gradient[index] = -rhs;
    gradient[index] = -rhs;
    velocity_x[index] = Scalar(0);
    velocity_y[index] = Scalar(0);
    velocity_z[index] = Scalar(0);
}

template <typename Scalar>
__global__ void accelerated_candidate(
    const Scalar* previous,
    const Scalar* iterate,
    const Scalar* previous_gradient,
    const Scalar* gradient,
    Scalar* next,
    Scalar momentum,
    std::uint32_t count) {
    const std::uint32_t index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= count) {
        return;
    }
    const Scalar extrapolated = iterate[index] + momentum * (iterate[index] - previous[index]);
    const Scalar extrapolated_gradient =
        gradient[index] + momentum * (gradient[index] - previous_gradient[index]);
    const Scalar candidate = extrapolated - static_cast<Scalar>(kProjectedGradientStep) * extrapolated_gradient;
    next[index] = candidate > Scalar(0) ? candidate : Scalar(0);
}

template <typename Scalar>
__global__ void scale_vector(
    const Scalar* scale,
    const Scalar* input,
    Scalar* output,
    std::uint32_t count) {
    const std::uint32_t index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index < count) {
        output[index] = scale[index] * input[index];
    }
}

template <typename Scalar>
__global__ void pressure_acceleration(
    const std::uint32_t* fluid_offsets,
    const std::uint32_t* fluid_other,
    const Scalar* fluid_gx,
    const Scalar* fluid_gy,
    const Scalar* fluid_gz,
    const std::uint32_t* boundary_offsets,
    const Scalar* boundary_gx,
    const Scalar* boundary_gy,
    const Scalar* boundary_gz,
    const Scalar* multiplier,
    Scalar* acceleration_x,
    Scalar* acceleration_y,
    Scalar* acceleration_z,
    std::uint32_t count) {
    const std::uint32_t row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= count) {
        return;
    }
    Scalar x = Scalar(0);
    Scalar y = Scalar(0);
    Scalar z = Scalar(0);
    const Scalar own = multiplier[row];
    for (std::uint32_t edge = fluid_offsets[row]; edge < fluid_offsets[row + 1]; ++edge) {
        const Scalar pressure_sum = own + multiplier[fluid_other[edge]];
        if (abs(pressure_sum) > static_cast<Scalar>(kSolverEpsilon)) {
            const Scalar factor = -static_cast<Scalar>(kRestVolume) * pressure_sum;
            x += fluid_gx[edge] * factor;
            y += fluid_gy[edge] * factor;
            z += fluid_gz[edge] * factor;
        }
    }
    if (abs(own) > static_cast<Scalar>(kSolverEpsilon)) {
        const Scalar factor = -static_cast<Scalar>(kRestVolume) * own;
        for (std::uint32_t edge = boundary_offsets[row]; edge < boundary_offsets[row + 1]; ++edge) {
            x += boundary_gx[edge] * factor;
            y += boundary_gy[edge] * factor;
            z += boundary_gz[edge] * factor;
        }
    }
    acceleration_x[row] = x;
    acceleration_y[row] = y;
    acceleration_z[row] = z;
}

template <typename Scalar>
__global__ void matrix_action(
    const std::uint32_t* fluid_offsets,
    const std::uint32_t* fluid_other,
    const Scalar* fluid_gx,
    const Scalar* fluid_gy,
    const Scalar* fluid_gz,
    const std::uint32_t* boundary_offsets,
    const Scalar* boundary_gx,
    const Scalar* boundary_gy,
    const Scalar* boundary_gz,
    const Scalar* acceleration_x,
    const Scalar* acceleration_y,
    const Scalar* acceleration_z,
    Scalar* result,
    std::uint32_t count) {
    const std::uint32_t row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= count) {
        return;
    }
    const Scalar own_x = acceleration_x[row];
    const Scalar own_y = acceleration_y[row];
    const Scalar own_z = acceleration_z[row];
    Scalar value = Scalar(0);
    for (std::uint32_t edge = fluid_offsets[row]; edge < fluid_offsets[row + 1]; ++edge) {
        const std::uint32_t other = fluid_other[edge];
        const Scalar relative_x = own_x - acceleration_x[other];
        const Scalar relative_y = own_y - acceleration_y[other];
        const Scalar relative_z = own_z - acceleration_z[other];
        const Scalar dot = relative_x * fluid_gx[edge] + relative_y * fluid_gy[edge] +
                           relative_z * fluid_gz[edge];
        value += static_cast<Scalar>(kRestVolume) * dot;
    }
    for (std::uint32_t edge = boundary_offsets[row]; edge < boundary_offsets[row + 1]; ++edge) {
        const Scalar dot = own_x * boundary_gx[edge] + own_y * boundary_gy[edge] +
                           own_z * boundary_gz[edge];
        value += static_cast<Scalar>(kRestVolume) * dot;
    }
    result[row] = -static_cast<Scalar>(kDtSquared) * value;
}

template <typename Scalar>
__global__ void finish_iteration(
    const Scalar* previous,
    const Scalar* iterate,
    const Scalar* previous_gradient,
    const Scalar* gradient,
    const Scalar* next,
    const Scalar* matrix,
    const Scalar* right_hand_side,
    const Scalar* scale,
    Scalar* next_gradient,
    double* density_terms,
    double* kkt_terms,
    double* displacement_squared_terms,
    double* displacement_action_terms,
    Scalar momentum,
    std::uint32_t count) {
    const std::uint32_t index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= count) {
        return;
    }
    const Scalar next_g = scale[index] * matrix[index] - right_hand_side[index];
    next_gradient[index] = next_g;
    const Scalar original_gradient = next_g / scale[index];
    density_terms[index] =
        static_cast<double>(original_gradient < Scalar(0) ? -original_gradient : Scalar(0));
    const Scalar projected =
        next[index] > Scalar(0) ? abs(original_gradient) : max(-original_gradient, Scalar(0));
    kkt_terms[index] = static_cast<double>(projected);

    const Scalar extrapolated = iterate[index] + momentum * (iterate[index] - previous[index]);
    const Scalar extrapolated_gradient =
        gradient[index] + momentum * (gradient[index] - previous_gradient[index]);
    const Scalar displacement = next[index] - extrapolated;
    const Scalar action = next_g - extrapolated_gradient;
    displacement_squared_terms[index] = static_cast<double>(displacement) * displacement;
    displacement_action_terms[index] = static_cast<double>(displacement) * action;
}

__global__ void validate_reductions(const double* reductions, double* checksum) {
    if (threadIdx.x == 0 && blockIdx.x == 0) {
        const double value = reductions[0] + reductions[1] + reductions[2] + reductions[3];
        checksum[0] += value * 1.0e-12;
    }
}

template <typename Scalar>
__global__ void apply_final_acceleration(
    const Scalar* acceleration_x,
    const Scalar* acceleration_y,
    const Scalar* acceleration_z,
    Scalar* velocity_x,
    Scalar* velocity_y,
    Scalar* velocity_z,
    double* checksum,
    std::uint32_t count) {
    const std::uint32_t index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= count) {
        return;
    }
    velocity_x[index] += acceleration_x[index];
    velocity_y[index] += acceleration_y[index];
    velocity_z[index] += acceleration_z[index];
    if (index == 0) {
        checksum[0] += static_cast<double>(velocity_x[0] + velocity_y[0] + velocity_z[0]);
    }
}

template <typename Scalar>
struct SolveWorkspace {
    DeviceBuffer<Scalar> scale{kFluidCount};
    DeviceBuffer<Scalar> right_hand_side{kFluidCount};
    DeviceBuffer<Scalar> previous{kFluidCount};
    DeviceBuffer<Scalar> iterate{kFluidCount};
    DeviceBuffer<Scalar> next{kFluidCount};
    DeviceBuffer<Scalar> previous_gradient{kFluidCount};
    DeviceBuffer<Scalar> gradient{kFluidCount};
    DeviceBuffer<Scalar> next_gradient{kFluidCount};
    DeviceBuffer<Scalar> unscaled{kFluidCount};
    DeviceBuffer<Scalar> acceleration_x{kFluidCount};
    DeviceBuffer<Scalar> acceleration_y{kFluidCount};
    DeviceBuffer<Scalar> acceleration_z{kFluidCount};
    DeviceBuffer<Scalar> matrix{kFluidCount};
    DeviceBuffer<Scalar> velocity_x{kFluidCount};
    DeviceBuffer<Scalar> velocity_y{kFluidCount};
    DeviceBuffer<Scalar> velocity_z{kFluidCount};
    DeviceBuffer<double> density_terms{kFluidCount};
    DeviceBuffer<double> kkt_terms{kFluidCount};
    DeviceBuffer<double> displacement_squared_terms{kFluidCount};
    DeviceBuffer<double> displacement_action_terms{kFluidCount};
    DeviceBuffer<double> reductions{4};
    DeviceBuffer<double> checksum{1};
    DeviceBuffer<std::uint8_t> reduce_scratch;
    std::size_t reduce_scratch_bytes = 0;

    SolveWorkspace() {
        check_cuda(
            cub::DeviceReduce::Sum(
                nullptr,
                reduce_scratch_bytes,
                density_terms.get(),
                reductions.get(),
                kFluidCount),
            "cub reduce sizing");
        reduce_scratch.allocate(reduce_scratch_bytes);
    }
};

template <typename Scalar>
void run_solve(const NeighborGraph<Scalar>& graph, SolveWorkspace<Scalar>& workspace, int iterations) {
    check_cuda(cudaMemset(workspace.checksum.get(), 0, sizeof(double)), "clear solve checksum");
    initialize_solve_vectors<<<blocks_for(kFluidCount), kBlockSize>>>(
        workspace.scale.get(),
        workspace.right_hand_side.get(),
        workspace.previous.get(),
        workspace.iterate.get(),
        workspace.previous_gradient.get(),
        workspace.gradient.get(),
        workspace.velocity_x.get(),
        workspace.velocity_y.get(),
        workspace.velocity_z.get(),
        kFluidCount);
    check_launch("initialize_solve_vectors");

    Scalar* previous = workspace.previous.get();
    Scalar* iterate = workspace.iterate.get();
    Scalar* next = workspace.next.get();
    Scalar* previous_gradient = workspace.previous_gradient.get();
    Scalar* gradient = workspace.gradient.get();
    Scalar* next_gradient = workspace.next_gradient.get();

    for (int iteration = 1; iteration <= iterations; ++iteration) {
        const Scalar momentum = static_cast<Scalar>(
            static_cast<double>(iteration - 1) / static_cast<double>(iteration + 2));
        accelerated_candidate<<<blocks_for(kFluidCount), kBlockSize>>>(
            previous,
            iterate,
            previous_gradient,
            gradient,
            next,
            momentum,
            kFluidCount);
        scale_vector<<<blocks_for(kFluidCount), kBlockSize>>>(
            workspace.scale.get(), next, workspace.unscaled.get(), kFluidCount);
        pressure_acceleration<<<blocks_for(kFluidCount), kBlockSize>>>(
            graph.fluid_offsets.get(),
            graph.fluid_other.get(),
            graph.fluid_gx.get(),
            graph.fluid_gy.get(),
            graph.fluid_gz.get(),
            graph.boundary_offsets.get(),
            graph.boundary_gx.get(),
            graph.boundary_gy.get(),
            graph.boundary_gz.get(),
            workspace.unscaled.get(),
            workspace.acceleration_x.get(),
            workspace.acceleration_y.get(),
            workspace.acceleration_z.get(),
            kFluidCount);
        matrix_action<<<blocks_for(kFluidCount), kBlockSize>>>(
            graph.fluid_offsets.get(),
            graph.fluid_other.get(),
            graph.fluid_gx.get(),
            graph.fluid_gy.get(),
            graph.fluid_gz.get(),
            graph.boundary_offsets.get(),
            graph.boundary_gx.get(),
            graph.boundary_gy.get(),
            graph.boundary_gz.get(),
            workspace.acceleration_x.get(),
            workspace.acceleration_y.get(),
            workspace.acceleration_z.get(),
            workspace.matrix.get(),
            kFluidCount);
        finish_iteration<<<blocks_for(kFluidCount), kBlockSize>>>(
            previous,
            iterate,
            previous_gradient,
            gradient,
            next,
            workspace.matrix.get(),
            workspace.right_hand_side.get(),
            workspace.scale.get(),
            next_gradient,
            workspace.density_terms.get(),
            workspace.kkt_terms.get(),
            workspace.displacement_squared_terms.get(),
            workspace.displacement_action_terms.get(),
            momentum,
            kFluidCount);
        check_launch("APG iteration kernels");

        check_cuda(
            cub::DeviceReduce::Sum(
                workspace.reduce_scratch.get(),
                workspace.reduce_scratch_bytes,
                workspace.density_terms.get(),
                workspace.reductions.get(),
                kFluidCount),
            "density reduction");
        check_cuda(
            cub::DeviceReduce::Sum(
                workspace.reduce_scratch.get(),
                workspace.reduce_scratch_bytes,
                workspace.kkt_terms.get(),
                workspace.reductions.get() + 1,
                kFluidCount),
            "KKT reduction");
        check_cuda(
            cub::DeviceReduce::Sum(
                workspace.reduce_scratch.get(),
                workspace.reduce_scratch_bytes,
                workspace.displacement_squared_terms.get(),
                workspace.reductions.get() + 2,
                kFluidCount),
            "displacement reduction");
        check_cuda(
            cub::DeviceReduce::Sum(
                workspace.reduce_scratch.get(),
                workspace.reduce_scratch_bytes,
                workspace.displacement_action_terms.get(),
                workspace.reductions.get() + 3,
                kFluidCount),
            "curvature reduction");
        validate_reductions<<<1, 1>>>(workspace.reductions.get(), workspace.checksum.get());
        check_launch("validate_reductions");

        std::swap(previous, iterate);
        std::swap(iterate, next);
        std::swap(previous_gradient, gradient);
        std::swap(gradient, next_gradient);
    }

    scale_vector<<<blocks_for(kFluidCount), kBlockSize>>>(
        workspace.scale.get(), iterate, workspace.unscaled.get(), kFluidCount);
    pressure_acceleration<<<blocks_for(kFluidCount), kBlockSize>>>(
        graph.fluid_offsets.get(),
        graph.fluid_other.get(),
        graph.fluid_gx.get(),
        graph.fluid_gy.get(),
        graph.fluid_gz.get(),
        graph.boundary_offsets.get(),
        graph.boundary_gx.get(),
        graph.boundary_gy.get(),
        graph.boundary_gz.get(),
        workspace.unscaled.get(),
        workspace.acceleration_x.get(),
        workspace.acceleration_y.get(),
        workspace.acceleration_z.get(),
        kFluidCount);
    apply_final_acceleration<<<blocks_for(kFluidCount), kBlockSize>>>(
        workspace.acceleration_x.get(),
        workspace.acceleration_y.get(),
        workspace.acceleration_z.get(),
        workspace.velocity_x.get(),
        workspace.velocity_y.get(),
        workspace.velocity_z.get(),
        workspace.checksum.get(),
        kFluidCount);
    check_launch("final acceleration");
}

double copy_checksum(const DeviceBuffer<double>& checksum) {
    double value = 0.0;
    check_cuda(cudaMemcpy(&value, checksum.get(), sizeof(value), cudaMemcpyDeviceToHost), "copy checksum");
    if (!std::isfinite(value)) {
        throw std::runtime_error("non-finite solve checksum");
    }
    return value;
}

struct Distribution {
    double minimum = 0.0;
    double median = 0.0;
    double p95 = 0.0;
    double maximum = 0.0;
    double mean = 0.0;
};

Distribution summarize(std::vector<float> samples) {
    if (samples.empty()) {
        throw std::runtime_error("cannot summarize an empty sample set");
    }
    double sum = 0.0;
    for (float value : samples) {
        if (!std::isfinite(value) || value < 0.0F) {
            throw std::runtime_error("invalid timing sample");
        }
        sum += value;
    }
    std::sort(samples.begin(), samples.end());
    const auto percentile = [&](double p) {
        const std::size_t index = static_cast<std::size_t>(
            std::ceil(p * static_cast<double>(samples.size())) - 1.0);
        return static_cast<double>(samples[std::min(index, samples.size() - 1)]);
    };
    return Distribution{
        static_cast<double>(samples.front()),
        percentile(0.5),
        percentile(0.95),
        static_cast<double>(samples.back()),
        sum / static_cast<double>(samples.size()),
    };
}

struct ModeResult {
    std::string name;
    Distribution reconstruction;
    Distribution solve;
    Distribution proxy_total;
    double checksum = 0.0;
};

template <typename Scalar>
ModeResult benchmark_mode(
    const std::string& name,
    const DeviceBuffer<Int3>& fluid_positions,
    const DeviceBuffer<Int3>& boundary_positions,
    const Options& options) {
    ReconstructionWorkspace reconstruction_workspace;
    NeighborGraph<Scalar> graph;
    SolveWorkspace<Scalar> solve_workspace;
    reconstruct(
        fluid_positions,
        boundary_positions,
        reconstruction_workspace,
        graph,
        true);
    check_cuda(cudaDeviceSynchronize(), "initial reconstruction synchronization");
    if (graph.fluid_edges != kExpectedFluidEdges || graph.boundary_edges != kExpectedBoundaryEdges) {
        throw std::runtime_error(
            "sealed neighbor graph mismatch: fluid=" + std::to_string(graph.fluid_edges) +
            ", boundary=" + std::to_string(graph.boundary_edges));
    }

    for (int index = 0; index < options.warmup; ++index) {
        reconstruct(
            fluid_positions,
            boundary_positions,
            reconstruction_workspace,
            graph,
            false);
        run_solve(graph, solve_workspace, options.iterations);
    }
    check_cuda(cudaDeviceSynchronize(), "warmup synchronization");

    EventTimer timer;
    std::vector<float> reconstruction_samples;
    std::vector<float> solve_samples;
    std::vector<float> total_samples;
    reconstruction_samples.reserve(options.runs);
    solve_samples.reserve(options.runs);
    total_samples.reserve(options.runs);
    for (int index = 0; index < options.runs; ++index) {
        const float reconstruction_ms = timer.measure([&] {
            reconstruct(
                fluid_positions,
                boundary_positions,
                reconstruction_workspace,
                graph,
                false);
        });
        const float solve_ms = timer.measure([&] {
            run_solve(graph, solve_workspace, options.iterations);
        });
        reconstruction_samples.push_back(reconstruction_ms);
        solve_samples.push_back(solve_ms);
        total_samples.push_back(reconstruction_ms + solve_ms);
    }

    return ModeResult{
        name,
        summarize(std::move(reconstruction_samples)),
        summarize(std::move(solve_samples)),
        summarize(std::move(total_samples)),
        copy_checksum(solve_workspace.checksum),
    };
}

Options parse_options(int argc, char** argv) {
    Options options;
    for (int index = 1; index < argc; ++index) {
        const std::string argument = argv[index];
        auto parse_positive = [&](const char* label) {
            if (index + 1 >= argc) {
                throw std::runtime_error(std::string("missing value for ") + label);
            }
            const long value = std::strtol(argv[++index], nullptr, 10);
            if (value <= 0 || value > 1000) {
                throw std::runtime_error(std::string(label) + " must be in 1..=1000");
            }
            return static_cast<int>(value);
        };
        if (argument == "--self-test") {
            options.self_test = true;
            options.warmup = 1;
            options.runs = 1;
            options.iterations = 1;
        } else if (argument == "--warmup") {
            options.warmup = parse_positive("--warmup");
        } else if (argument == "--runs") {
            options.runs = parse_positive("--runs");
        } else if (argument == "--iterations") {
            options.iterations = parse_positive("--iterations");
        } else {
            throw std::runtime_error("unknown argument: " + argument);
        }
    }
    return options;
}

void print_distribution(const Distribution& value) {
    std::cout << "{\"minimum_ms\":" << value.minimum << ",\"median_ms\":" << value.median
              << ",\"p95_ms\":" << value.p95 << ",\"maximum_ms\":" << value.maximum
              << ",\"mean_ms\":" << value.mean << "}";
}

void print_mode(const ModeResult& value) {
    std::cout << "{\"precision_profile\":\"" << value.name << "\",\"reconstruction\":";
    print_distribution(value.reconstruction);
    std::cout << ",\"cold_apg_solve\":";
    print_distribution(value.solve);
    std::cout << ",\"proxy_total\":";
    print_distribution(value.proxy_total);
    std::cout << ",\"finite_checksum\":" << value.checksum << "}";
}

}  // namespace

int main(int argc, char** argv) {
    try {
        const Options options = parse_options(argc, argv);
        int device = 0;
        check_cuda(cudaGetDevice(&device), "cudaGetDevice");
        cudaDeviceProp properties{};
        check_cuda(cudaGetDeviceProperties(&properties, device), "cudaGetDeviceProperties");
        if (properties.major != 8 || properties.minor != 6) {
            throw std::runtime_error("this bounded spike requires the declared compute-capability 8.6 host");
        }

        const std::vector<Int3> fluid = make_fluid_positions();
        const std::vector<Int3> boundary = make_boundary_positions();
        DeviceBuffer<Int3> device_fluid(fluid.size());
        DeviceBuffer<Int3> device_boundary(boundary.size());
        check_cuda(
            cudaMemcpy(
                device_fluid.get(),
                fluid.data(),
                sizeof(Int3) * fluid.size(),
                cudaMemcpyHostToDevice),
            "copy fluid positions");
        check_cuda(
            cudaMemcpy(
                device_boundary.get(),
                boundary.data(),
                sizeof(Int3) * boundary.size(),
                cudaMemcpyHostToDevice),
            "copy boundary positions");

        const ModeResult f64 = benchmark_mode<double>(
            "f64-cold-apg-gpu-tree",
            device_fluid,
            device_boundary,
            options);
        const ModeResult mixed32 = benchmark_mode<float>(
            "mixed32-f32-operator-f64-reductions-cold-apg",
            device_fluid,
            device_boundary,
            options);

        std::cout << std::fixed << std::setprecision(6);
        std::cout << "{\"schema_version\":1,\"status\":\"REPORT_ONLY\",\"invocation\":\""
                  << (options.self_test ? "SELF_TEST" : "MEASUREMENT") << "\","
                     "\"classification\":\"GPU_FEASIBILITY_PROXY_NO_AUTHORITY_NO_PRODUCTCHECK_CREDIT\","
                     "\"scope\":{\"windows\":\"OUT_OF_SCOPE_BY_USER\","
                     "\"canonical_roots\":\"NOT_EVALUATED\","
                     "\"synthetic_solve_vectors\":true,"
                     "\"omitted_stages\":[\"density-diagonal-assembly\","
                     "\"data-dependent-convergence-branch\",\"divergence\",\"contact\","
                     "\"integration\",\"canonical-publication\"]},"
                     "\"device\":{\"name\":\""
                  << properties.name << "\",\"compute_capability\":\"" << properties.major << "."
                  << properties.minor << "\",\"global_memory_bytes\":" << properties.totalGlobalMem
                  << ",\"sm_count\":" << properties.multiProcessorCount << "},"
                     "\"toolchain\":{\"cuda_runtime_version\":"
                  << CUDART_VERSION << ",\"cuda_compiler_major\":" << __CUDACC_VER_MAJOR__
                  << ",\"cuda_compiler_minor\":" << __CUDACC_VER_MINOR__ << "},"
                     "\"workload\":{\"scenario\":\"CW-SEALED-001-initial-lattice-proxy\","
                     "\"fluid_samples\":"
                  << kFluidCount << ",\"boundary_samples\":" << kExpectedBoundaryCount
                  << ",\"fluid_directed_edges\":" << kExpectedFluidEdges
                  << ",\"boundary_directed_edges\":" << kExpectedBoundaryEdges
                  << ",\"apg_iterations\":" << options.iterations << ",\"warmup_runs\":"
                  << options.warmup << ",\"measured_runs\":" << options.runs << "},\"modes\":[";
        print_mode(f64);
        std::cout << ',';
        print_mode(mixed32);
        std::cout << "]}\n";
        return 0;
    } catch (const std::exception& error) {
        std::cerr << "continuum-water GPU feasibility failed: " << error.what() << '\n';
        return 1;
    }
}
