#pragma once

#include <cstdint>
#include <string>
#include <vector>

namespace nextengine::nonlocal::gpu_audit {

enum class TermKind : std::uint32_t {
    KernelGradient = 1,
    Compression = 2,
    BulkViscosity = 3,
    ShearViscosity = 4,
    Surface = 5,
};

struct Vec3Input {
    double x = 0.0;
    double y = 0.0;
    double z = 0.0;
};

struct AuditProfile {
    double horizon = 0.15;
    double spacing = 0.05;
    double mass = 0.125;
    double time_step = 1.0 / 240.0;
    double rest_density = 1000.0;
    double kappa = 9196.875;
    double lambda = 2.5;
    double mu = 1.7;
    double gamma = 3.5;
};

struct TermInput {
    TermKind kind = TermKind::KernelGradient;
    Vec3Input first;
    Vec3Input second;
    Vec3Input reference_first;
    Vec3Input reference_second;
    double parameter = 0.0;
};

struct ReferenceTermOutput {
    double scalar = 0.0;
    Vec3Input first_force;
    Vec3Input second_force;
    bool finite = false;
};

struct GpuTermOutput {
    float scalar = 0.0F;
    float first_force[3] = {};
    float second_force[3] = {};
    std::uint32_t finite = 0;
};

static_assert(sizeof(GpuTermOutput) == 32, "NCGA0 GPU payload layout must stay fixed");

ReferenceTermOutput evaluate_reference_term(
    const AuditProfile& profile,
    const TermInput& input);

std::vector<GpuTermOutput> evaluate_gpu_terms(
    const AuditProfile& profile,
    const std::vector<TermInput>& inputs,
    bool source_shaped_gradient);

std::string gpu_environment_json();

} // namespace nextengine::nonlocal::gpu_audit
