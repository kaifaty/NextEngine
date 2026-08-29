#include "corrected_cuda_terms.hpp"

#include <cuda_runtime.h>

#include <cmath>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

namespace nextengine::nonlocal::gpu_audit {
namespace {

constexpr float CUDA_PI = 3.14159265358979323846F;

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
};

struct DeviceInput {
    std::uint32_t kind;
    float3 first;
    float3 second;
    float3 reference_first;
    float3 reference_second;
    float parameter;
};

void check_cuda(cudaError_t result, const char* operation) {
    if (result != cudaSuccess) {
        throw std::runtime_error(
            std::string(operation) + ": " + cudaGetErrorString(result));
    }
}

__device__ float3 subtract3(float3 lhs, float3 rhs) {
    return make_float3(lhs.x - rhs.x, lhs.y - rhs.y, lhs.z - rhs.z);
}

__device__ float3 multiply3(float scalar, float3 value) {
    return make_float3(scalar * value.x, scalar * value.y, scalar * value.z);
}

__device__ float dot3(float3 lhs, float3 rhs) {
    return lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z;
}

__device__ float length3(float3 value) {
    return sqrtf(dot3(value, value));
}

__device__ float3 normalized3(float3 value) {
    return multiply3(1.0F / length3(value), value);
}

__device__ float corrected_weight(float radius, float horizon) {
    const float q = 2.0F * radius / horizon;
    const float alpha =
        3.0F / (2.0F * CUDA_PI * horizon * horizon * horizon);
    if (q > 2.0F) {
        return 0.0F;
    }
    if (q >= 1.0F) {
        const float delta = 2.0F - q;
        return alpha * delta * delta * delta / 6.0F;
    }
    return alpha * (2.0F / 3.0F - q * q + 0.5F * q * q * q);
}

__device__ float corrected_gradient(
    float radius,
    float horizon,
    bool source_shaped) {
    const float q = 2.0F * radius / horizon;
    const float alpha =
        3.0F / (2.0F * CUDA_PI * horizon * horizon * horizon);
    float derivative_q = 0.0F;
    if (q > 2.0F) {
        return 0.0F;
    }
    if (q >= 1.0F) {
        const float delta = 2.0F - q;
        derivative_q = -0.5F * alpha * delta * delta;
    } else {
        derivative_q = alpha * (-2.0F * q + 1.5F * q * q);
    }
    return source_shaped ? derivative_q : derivative_q * (2.0F / horizon);
}

__device__ float surface_force(float radius, float spacing) {
    const float q = radius / spacing;
    if (q <= 1.0F) {
        return q * q - 1.0F;
    }
    if (q < 3.0F) {
        const float shifted = q - 2.0F;
        return 1.0F - shifted * shifted;
    }
    return 0.0F;
}

__device__ void store_vec(float target[3], float3 value) {
    target[0] = value.x;
    target[1] = value.y;
    target[2] = value.z;
}

__device__ bool finite3(float3 value) {
    return isfinite(value.x) && isfinite(value.y) && isfinite(value.z);
}

__global__ void evaluate_terms(
    DeviceProfile profile,
    const DeviceInput* inputs,
    GpuTermOutput* outputs,
    int count,
    bool source_shaped) {
    const int index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= count) {
        return;
    }
    const DeviceInput input = inputs[index];
    GpuTermOutput output{};
    float3 first_force = make_float3(0.0F, 0.0F, 0.0F);
    float3 second_force = make_float3(0.0F, 0.0F, 0.0F);
    const TermKind kind = static_cast<TermKind>(input.kind);

    if (kind == TermKind::KernelGradient) {
        output.scalar = corrected_gradient(
            input.parameter, profile.horizon, source_shaped);
    } else if (kind == TermKind::Compression) {
        const float3 delta = subtract3(input.first, input.second);
        const float radius = length3(delta);
        const float direction_scale = 1.0F / radius;
        const float3 direction = multiply3(direction_scale, delta);
        const float density = profile.mass
            * (corrected_weight(0.0F, profile.horizon)
                + corrected_weight(radius, profile.horizon));
        const float control_rest_density = density / input.parameter;
        const float error = fmaxf(
            density / control_rest_density - 1.0F, 0.0F);
        const float factor = -2.0F * profile.kappa * error * profile.mass
            / control_rest_density
            * corrected_gradient(radius, profile.horizon, source_shaped);
        first_force = multiply3(factor, direction);
        const float3 reverse_direction = normalized3(
            subtract3(input.second, input.first));
        second_force = multiply3(factor, reverse_direction);
    } else if (kind == TermKind::BulkViscosity
        || kind == TermKind::ShearViscosity) {
        const float3 reference_delta = subtract3(
            input.reference_first, input.reference_second);
        const float radius = length3(reference_delta);
        const float3 normal = normalized3(reference_delta);
        const float3 increment = subtract3(
            subtract3(input.first, input.second), reference_delta);
        const float3 normal_component = multiply3(dot3(increment, normal), normal);
        const bool shear = kind == TermKind::ShearViscosity;
        const float3 component = shear
            ? subtract3(increment, normal_component)
            : normal_component;
        const float influence = -corrected_gradient(
            radius, profile.horizon, false);
        const float coefficient = shear ? 2.0F * profile.mu : profile.lambda;
        const float factor = -profile.mass
            / (profile.rest_density * profile.time_step)
            * coefficient * influence;
        first_force = multiply3(factor, component);

        const float3 reverse_reference_delta = subtract3(
            input.reference_second, input.reference_first);
        const float3 reverse_normal = normalized3(reverse_reference_delta);
        const float3 reverse_increment = subtract3(
            subtract3(input.second, input.first), reverse_reference_delta);
        const float3 reverse_normal_component = multiply3(
            dot3(reverse_increment, reverse_normal), reverse_normal);
        const float3 reverse_component = shear
            ? subtract3(reverse_increment, reverse_normal_component)
            : reverse_normal_component;
        second_force = multiply3(factor, reverse_component);
    } else if (kind == TermKind::Surface) {
        const float3 delta = subtract3(input.first, input.second);
        const float radius = length3(delta);
        const float pair_value = surface_force(radius, profile.spacing);
        if (radius > 0.0F) {
            const float factor = -2.0F * profile.gamma * profile.mass
                * profile.mass * pair_value;
            first_force = multiply3(factor, normalized3(delta));
            second_force = multiply3(
                factor, normalized3(subtract3(input.second, input.first)));
        }
    }

    store_vec(output.first_force, first_force);
    store_vec(output.second_force, second_force);
    output.finite = isfinite(output.scalar) && finite3(first_force)
        && finite3(second_force);
    outputs[index] = output;
}

float3 to_float3(Vec3Input value) {
    return make_float3(
        static_cast<float>(value.x),
        static_cast<float>(value.y),
        static_cast<float>(value.z));
}

} // namespace

std::vector<GpuTermOutput> evaluate_gpu_terms(
    const AuditProfile& profile,
    const std::vector<TermInput>& inputs,
    bool source_shaped_gradient) {
    if (inputs.empty()) {
        throw std::runtime_error("NCGA0 input table must be nonempty");
    }
    const DeviceProfile device_profile{
        static_cast<float>(profile.horizon),
        static_cast<float>(profile.spacing),
        static_cast<float>(profile.mass),
        static_cast<float>(profile.time_step),
        static_cast<float>(profile.rest_density),
        static_cast<float>(profile.kappa),
        static_cast<float>(profile.lambda),
        static_cast<float>(profile.mu),
        static_cast<float>(profile.gamma),
    };
    std::vector<DeviceInput> device_inputs;
    device_inputs.reserve(inputs.size());
    for (const TermInput& input : inputs) {
        device_inputs.push_back({
            static_cast<std::uint32_t>(input.kind),
            to_float3(input.first),
            to_float3(input.second),
            to_float3(input.reference_first),
            to_float3(input.reference_second),
            static_cast<float>(input.parameter),
        });
    }

    DeviceInput* device_input = nullptr;
    GpuTermOutput* device_output = nullptr;
    const std::size_t input_bytes = device_inputs.size() * sizeof(DeviceInput);
    const std::size_t output_bytes = inputs.size() * sizeof(GpuTermOutput);
    check_cuda(cudaMalloc(&device_input, input_bytes), "cudaMalloc input");
    try {
        check_cuda(cudaMalloc(&device_output, output_bytes), "cudaMalloc output");
        check_cuda(cudaMemcpy(device_input, device_inputs.data(), input_bytes,
                       cudaMemcpyHostToDevice),
            "cudaMemcpy input");
        evaluate_terms<<<1, 32>>>(device_profile, device_input, device_output,
            static_cast<int>(inputs.size()), source_shaped_gradient);
        check_cuda(cudaGetLastError(), "evaluate_terms launch");
        check_cuda(cudaDeviceSynchronize(), "evaluate_terms synchronize");
        std::vector<GpuTermOutput> output(inputs.size());
        check_cuda(cudaMemcpy(output.data(), device_output, output_bytes,
                       cudaMemcpyDeviceToHost),
            "cudaMemcpy output");
        check_cuda(cudaFree(device_output), "cudaFree output");
        device_output = nullptr;
        check_cuda(cudaFree(device_input), "cudaFree input");
        return output;
    } catch (...) {
        if (device_output != nullptr) {
            cudaFree(device_output);
        }
        cudaFree(device_input);
        throw;
    }
}

std::string gpu_environment_json() {
    int device = 0;
    check_cuda(cudaGetDevice(&device), "cudaGetDevice");
    cudaDeviceProp properties{};
    check_cuda(cudaGetDeviceProperties(&properties, device),
        "cudaGetDeviceProperties");
    int driver = 0;
    int runtime = 0;
    check_cuda(cudaDriverGetVersion(&driver), "cudaDriverGetVersion");
    check_cuda(cudaRuntimeGetVersion(&runtime), "cudaRuntimeGetVersion");
    std::ostringstream output;
    output << "{\"name\":\"" << properties.name << "\",\"major\":"
           << properties.major << ",\"minor\":" << properties.minor
           << ",\"driver\":" << driver << ",\"runtime\":" << runtime
           << ",\"fmad\":false,\"precise_div\":true,\"precise_sqrt\":true,"
              "\"ftz\":false}";
    return output.str();
}

} // namespace nextengine::nonlocal::gpu_audit
