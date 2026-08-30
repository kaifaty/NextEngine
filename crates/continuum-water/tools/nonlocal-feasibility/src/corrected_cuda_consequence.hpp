#pragma once

#include "corrected_cuda_assembly.hpp"

#include <cstdint>
#include <vector>

namespace nextengine::nonlocal::gpu_assembly_audit {

enum class PressureArithmetic : std::uint32_t {
    F32ProductsF64Reduction = 0,
    F64PressureProducts = 1,
};

struct PressureArithmeticWork {
    std::uint64_t density_terms = 0;
    std::uint64_t jacobian_terms = 0;
    std::uint64_t pressure_outer_products = 0;
    std::uint64_t pressure_geometric_products = 0;
    std::uint64_t f32_completed_products = 0;
    std::uint64_t f64_completed_products = 0;
    std::uint64_t f64_accumulations = 0;
    std::uint64_t output_rounds_to_f32 = 0;
};

struct MixedPressureResult {
    AssemblyFailure failure = AssemblyFailure::None;
    std::vector<double> hessian;
    std::vector<std::uint8_t> pressure_active;
    PressureArithmeticWork work;
};

MixedPressureResult evaluate_gpu_mixed_pressure_hessian(
    const AssemblyProfile& profile,
    const AssemblyFixture& fixture,
    PressureArithmetic arithmetic);

} // namespace nextengine::nonlocal::gpu_assembly_audit
