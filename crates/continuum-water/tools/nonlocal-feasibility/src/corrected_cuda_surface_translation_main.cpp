#include "corrected_cuda_full_step.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <limits>
#include <sstream>
#include <string>
#include <vector>

#ifndef NCGP2_CONTRACT_ROOT
#define NCGP2_CONTRACT_ROOT "unconfigured"
#endif
#ifndef NCGP2_SOURCE_ROOT
#define NCGP2_SOURCE_ROOT "unconfigured"
#endif
#ifndef NCGP2_SOURCE_COMMIT
#define NCGP2_SOURCE_COMMIT "unconfigured"
#endif
#ifndef NCGP2_SOURCE_TREE
#define NCGP2_SOURCE_TREE "unconfigured"
#endif
#ifndef NCGP2_COMPILER_FLAGS
#define NCGP2_COMPILER_FLAGS "unconfigured"
#endif

namespace {

using namespace nextengine::nonlocal::gpu_full_step;

std::string file_root(const std::string& path) {
    std::ifstream stream(path, std::ios::binary);
    if (!stream) return {};
    std::ostringstream bytes;
    bytes << stream.rdbuf();
    return nextengine::nonlocal::sha256_hex(bytes.str());
}

std::string binary_root() { return file_root("/proc/self/exe"); }

bool binary_identity_control_passes() {
    return binary_root().size() == 64U
        && file_root("/proc/self/nextengine-ncgp2-missing").empty();
}

std::uint32_t float_bits(float value) {
    std::uint32_t bits = 0U;
    static_assert(sizeof(bits) == sizeof(value));
    std::memcpy(&bits, &value, sizeof(bits));
    return bits;
}

float upward_ulp(float value) {
    return std::nextafter(value, std::numeric_limits<float>::infinity()) - value;
}

long double cubic_weight(long double radius, long double horizon) {
    const long double pi = acosl(-1.0L);
    const long double q = 2.0L * radius / horizon;
    const long double alpha = 3.0L
        / (2.0L * pi * horizon * horizon * horizon);
    if (q < 1.0L) {
        return alpha * (2.0L / 3.0L - q * q + 0.5L * q * q * q);
    }
    if (q <= 2.0L) {
        const long double tail = 2.0L - q;
        return alpha * tail * tail * tail / 6.0L;
    }
    return 0.0L;
}

NonlocalGpuProfile pair_profile(double gamma = 3.5) {
    NonlocalGpuProfile profile = nonlocal_water_profile();
    profile.gravity = {};
    profile.kappa = 500.0;
    profile.gamma = gamma;
    profile.rest_density = static_cast<double>(
        static_cast<long double>(profile.mass)
        * (cubic_weight(0.0L, static_cast<long double>(profile.horizon))
            + cubic_weight(0.05L, static_cast<long double>(profile.horizon)))
        / 1.1L);
    return profile;
}

std::vector<NonlocalGpuSample> pair_state(double center) {
    return canonicalize_samples_binary32({
        {101U, {center - 0.025, 0.75, 0.75},
            {center - 0.025, 0.75, 0.75}, {}},
        {202U, {center + 0.025, 0.75, 0.75},
            {center + 0.025, 0.75, 0.75}, {}},
    });
}

const NonlocalGpuSample* find_sample(
    const std::vector<NonlocalGpuSample>& samples, std::uint32_t sample_id) {
    const auto found = std::find_if(samples.begin(), samples.end(),
        [sample_id](const NonlocalGpuSample& sample) {
            return sample.sample_id == sample_id;
        });
    return found == samples.end() ? nullptr : &*found;
}

struct Route {
    NonlocalGpuStepResult gpu;
    NonlocalGpuStepResult permuted;
    NonlocalGpuStepResult cpu;
    std::string input_root;
    std::string gpu_work_root;
    std::string permuted_work_root;
    std::string cpu_work_root;
    std::string gpu_result_root;
    std::string permuted_result_root;
    std::string cpu_result_root;
    std::string environment;
    std::string arithmetic_input_root;
    std::string high_root;
    std::string low_root;
    std::string representation_root;
    std::string gpu_active_root;
    std::string permuted_active_root;
    std::string cpu_active_root;
    bool permutation_exact = false;
};

std::string active_pressure_root(const std::vector<std::uint32_t>& ids) {
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp2.active-pressure.v2\n"
             << ids.size() << '\n';
    for (const std::uint32_t id : ids) material << id << '\n';
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string ids_json(const std::vector<std::uint32_t>& ids) {
    std::ostringstream output;
    output << '[';
    for (std::size_t index = 0U; index < ids.size(); ++index) {
        if (index != 0U) output << ',';
        output << ids[index];
    }
    output << ']';
    return output.str();
}

std::string zero_low_root(const std::vector<NonlocalGpuSample>& state) {
    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp2.binary32-low-input.v2\n"
             << state.size() << '\n';
    for (const NonlocalGpuSample& sample : state) {
        material << sample.sample_id << '\n';
        for (unsigned int component = 0U; component < 9U; ++component) {
            material << 0U << '\n';
        }
    }
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string close_representation_root(
    const std::string& semantic_root, const std::string& representation_root) {
    return nextengine::nonlocal::sha256_hex(
        std::string("nextengine.nonlocal.ncgp2.closed-result.v2\n")
        + semantic_root + "\n" + representation_root + "\n");
}

Route run_route(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& state,
    NonlocalGpuVariant gpu_variant = NonlocalGpuVariant::Corrected) {
    const std::vector<NonlocalGpuGhost> no_ghosts;
    Route route;
    route.input_root = input_semantic_root(profile, state, no_ghosts);
    route.arithmetic_input_root = route.input_root;
    NonlocalGpuWorkspace workspace(profile);
    if (workspace.upload(state, no_ghosts) != NonlocalGpuFailure::None) {
        route.gpu.failure = NonlocalGpuFailure::InvalidState;
        return route;
    }
    route.gpu = workspace.step(128U,
        NonlocalGpuSolverProfile::Unpreconditioned,
        gpu_variant, true, false);
    route.environment = workspace.environment_json();
    route.cpu = step_reference(profile, state, no_ghosts, 128U,
        NonlocalGpuVariant::Corrected, true);

    auto permuted_state = state;
    std::reverse(permuted_state.begin(), permuted_state.end());
    NonlocalGpuWorkspace permuted_workspace(profile);
    if (permuted_workspace.upload(permuted_state, no_ghosts)
        != NonlocalGpuFailure::None) {
        route.permuted.failure = NonlocalGpuFailure::InvalidState;
        return route;
    }
    route.permuted = permuted_workspace.step(128U,
        NonlocalGpuSolverProfile::Unpreconditioned,
        gpu_variant, true, false);
    route.gpu_work_root = step_work_semantic_root(profile, route.gpu);
    route.permuted_work_root = step_work_semantic_root(profile, route.permuted);
    route.cpu_work_root = step_work_semantic_root(profile, route.cpu);
    route.gpu_result_root = step_semantic_root(
        profile, route.input_root, route.gpu);
    route.permuted_result_root = step_semantic_root(
        profile, route.input_root, route.permuted);
    route.cpu_result_root = step_semantic_root(
        profile, route.input_root, route.cpu);
    route.gpu_active_root = active_pressure_root(route.gpu.active_pressure_ids);
    route.permuted_active_root = active_pressure_root(
        route.permuted.active_pressure_ids);
    route.cpu_active_root = active_pressure_root(route.cpu.active_pressure_ids);
    route.permutation_exact = route.gpu.failure == route.permuted.failure
        && route.gpu_work_root == route.permuted_work_root
        && route.gpu_result_root == route.permuted_result_root
        && route.gpu.active_pressure_ids == route.permuted.active_pressure_ids;
    return route;
}

Route run_compensated_route(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& state,
    bool& input_exact,
    NonlocalGpuVariant variant = NonlocalGpuVariant::CompensatedStateF32) {
    const std::vector<NonlocalGpuGhost> no_ghosts;
    input_exact = std::all_of(state.begin(), state.end(),
        [](const NonlocalGpuSample& sample) {
            const auto exact = [](double value) {
                return static_cast<double>(static_cast<float>(value)) == value;
            };
            return exact(sample.reference.x) && exact(sample.reference.y)
                && exact(sample.reference.z) && exact(sample.current.x)
                && exact(sample.current.y) && exact(sample.current.z)
                && exact(sample.velocity.x) && exact(sample.velocity.y)
                && exact(sample.velocity.z);
        });
    Route route;
    route.input_root = input_semantic_root(profile, state, no_ghosts);
    route.high_root = route.input_root;
    route.low_root = zero_low_root(state);
    route.arithmetic_input_root = nextengine::nonlocal::sha256_hex(
        std::string("nextengine.nonlocal.ncgp2.arithmetic-input.v2\n")
        + route.high_root + "\n" + route.low_root + "\n");
    route.representation_root = nextengine::nonlocal::sha256_hex(
        std::string("nextengine.nonlocal.ncgp2.compensated-input.v2\n")
        + route.input_root + "\n" + route.arithmetic_input_root + "\n"
        + route.high_root + "\n" + route.low_root + "\n");
    NonlocalGpuWorkspace workspace(profile);
    if (workspace.upload(state, no_ghosts, true) != NonlocalGpuFailure::None) {
        route.gpu.failure = NonlocalGpuFailure::InvalidState;
        return route;
    }
    route.gpu = workspace.step(128U,
        NonlocalGpuSolverProfile::Unpreconditioned,
        variant, true, false);
    route.environment = workspace.environment_json();
    route.cpu = step_reference(profile, state, no_ghosts, 128U,
        NonlocalGpuVariant::Corrected, true);

    auto permuted_state = state;
    std::reverse(permuted_state.begin(), permuted_state.end());
    NonlocalGpuWorkspace permuted_workspace(profile);
    if (permuted_workspace.upload(permuted_state, no_ghosts, true)
        != NonlocalGpuFailure::None) {
        route.permuted.failure = NonlocalGpuFailure::InvalidState;
        return route;
    }
    route.permuted = permuted_workspace.step(128U,
        NonlocalGpuSolverProfile::Unpreconditioned,
        variant, true, false);
    route.gpu_work_root = step_work_semantic_root(profile, route.gpu);
    route.permuted_work_root = step_work_semantic_root(profile, route.permuted);
    route.cpu_work_root = step_work_semantic_root(profile, route.cpu);
    route.gpu_result_root = close_representation_root(step_semantic_root(
        profile, route.input_root, route.gpu), route.representation_root);
    route.permuted_result_root = close_representation_root(step_semantic_root(
        profile, route.input_root, route.permuted), route.representation_root);
    route.cpu_result_root = step_semantic_root(
        profile, route.input_root, route.cpu);
    route.gpu_active_root = active_pressure_root(route.gpu.active_pressure_ids);
    route.permuted_active_root = active_pressure_root(
        route.permuted.active_pressure_ids);
    route.cpu_active_root = active_pressure_root(route.cpu.active_pressure_ids);
    route.permutation_exact = route.gpu.failure == route.permuted.failure
        && route.gpu_work_root == route.permuted_work_root
        && route.gpu_result_root == route.permuted_result_root
        && route.gpu.active_pressure_ids == route.permuted.active_pressure_ids;
    return route;
}

void emit_center(double center,
    const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& state,
    const Route& route) {
    const float low = static_cast<float>(state[0].current.x);
    const float high = static_cast<float>(state[1].current.x);
    const float separation = high - low;
    const long double radius = static_cast<long double>(high)
        - static_cast<long double>(low);
    const long double q = radius / static_cast<long double>(profile.spacing);
    long double surface_force = 0.0L;
    if (q <= 1.0L) {
        surface_force = q * q - 1.0L;
    } else if (q < 3.0L) {
        const long double shifted = q - 2.0L;
        surface_force = 1.0L - shifted * shifted;
    }
    const long double radial_gradient = 2.0L
        * static_cast<long double>(profile.gamma)
        * static_cast<long double>(profile.mass)
        * static_cast<long double>(profile.mass) * surface_force;
    double maximum_cpu_displacement = 0.0;
    double maximum_cpu_displacement_ulps = 0.0;
    double maximum_once_round_error = 0.0;
    std::uint32_t once_round_unchanged = 0U;
    std::ostringstream endpoint_json;
    endpoint_json << '[';
    bool first_endpoint = true;
    if (route.cpu.failure == NonlocalGpuFailure::None) {
        for (const NonlocalGpuSample& initial : state) {
            const NonlocalGpuSample* final = find_sample(
                route.cpu.state, initial.sample_id);
            if (final == nullptr) continue;
            const double displacement = std::abs(
                final->current.x - initial.current.x);
            const float initial_float = static_cast<float>(initial.current.x);
            const double ulp = static_cast<double>(upward_ulp(initial_float));
            maximum_cpu_displacement = std::max(
                maximum_cpu_displacement, displacement);
            maximum_cpu_displacement_ulps = std::max(
                maximum_cpu_displacement_ulps, displacement / ulp);
            const float rounded_final = static_cast<float>(final->current.x);
            maximum_once_round_error = std::max(maximum_once_round_error,
                std::abs(static_cast<double>(rounded_final) - final->current.x));
            if (float_bits(rounded_final) == float_bits(initial_float)) {
                ++once_round_unchanged;
            }
            if (!first_endpoint) endpoint_json << ',';
            first_endpoint = false;
            endpoint_json << std::setprecision(17)
                          << "{\"sample_id\":" << initial.sample_id
                          << ",\"initial_bits\":" << float_bits(initial_float)
                          << ",\"initial_ulp_m\":" << ulp
                          << ",\"cpu_displacement_m\":" << displacement
                          << ",\"cpu_displacement_ulps\":"
                          << displacement / ulp
                          << ",\"once_round_error_m\":"
                          << std::abs(static_cast<double>(rounded_final)
                                - final->current.x)
                          << ",\"rounded_bits\":" << float_bits(rounded_final)
                          << ",\"once_round_unchanged\":"
                          << (float_bits(rounded_final) == float_bits(initial_float)
                                  ? "true" : "false")
                          << '}';
        }
    }
    endpoint_json << ']';

    std::cout << std::setprecision(17)
              << "{\"schema\":\"nextengine.nonlocal.ncgp2.phase-a.v2\""
              << ",\"kind\":\"center\",\"center_m\":" << center
              << ",\"profile_root\":\"" << profile_semantic_root(profile)
              << "\",\"input_root\":\"" << route.input_root
              << "\",\"low_bits\":" << float_bits(low)
              << ",\"high_bits\":" << float_bits(high)
              << ",\"low_ulp_m\":" << upward_ulp(low)
              << ",\"high_ulp_m\":" << upward_ulp(high)
              << ",\"encoded_separation_m\":" << separation
              << ",\"separation_error_m\":"
              << static_cast<double>(radius
                    - static_cast<long double>(profile.spacing))
              << ",\"surface_q\":" << static_cast<double>(q)
              << ",\"surface_c\":" << static_cast<double>(surface_force)
              << ",\"surface_radial_gradient\":"
              << static_cast<double>(radial_gradient)
              << ",\"cpu_failure\":"
              << static_cast<std::uint32_t>(route.cpu.failure)
              << ",\"cpu_scaled_residual\":"
              << route.cpu.scaled_displacement_residual
              << ",\"cpu_max_displacement_m\":"
              << maximum_cpu_displacement
              << ",\"cpu_max_displacement_ulps\":"
              << maximum_cpu_displacement_ulps
              << ",\"cpu_once_round_error_m\":"
              << maximum_once_round_error
              << ",\"cpu_once_round_unchanged_endpoints\":"
              << once_round_unchanged
              << ",\"cpu_endpoints\":" << endpoint_json.str()
              << ",\"gpu_failure\":"
              << static_cast<std::uint32_t>(route.gpu.failure)
              << ",\"gpu_scaled_residual\":"
              << route.gpu.scaled_displacement_residual
              << ",\"gpu_gradient_norm\":" << route.gpu.gradient_norm
              << ",\"gpu_hvp_used\":" << route.gpu.hvp_used
              << ",\"gpu_outer_trials\":" << route.gpu.outer_trials
              << ",\"gpu_accepted\":" << route.gpu.work.accepted_trials
              << ",\"gpu_rejected\":" << route.gpu.work.rejected_trials
              << ",\"gpu_work_root\":\"" << route.gpu_work_root
              << "\",\"gpu_result_root\":\"" << route.gpu_result_root
              << "\",\"gpu_active_root\":\"" << route.gpu_active_root
              << "\",\"gpu_active_ids\":"
              << ids_json(route.gpu.active_pressure_ids)
              << ",\"permuted_failure\":"
              << static_cast<std::uint32_t>(route.permuted.failure)
              << ",\"permutation_exact\":"
              << (route.permutation_exact ? "true" : "false")
              << ",\"permuted_work_root\":\""
              << route.permuted_work_root
              << "\",\"permuted_result_root\":\""
              << route.permuted_result_root
              << "\",\"permuted_active_root\":\""
              << route.permuted_active_root
              << "\",\"permuted_active_ids\":"
              << ids_json(route.permuted.active_pressure_ids)
              << ",\"cpu_work_root\":\"" << route.cpu_work_root
              << "\",\"cpu_result_root\":\"" << route.cpu_result_root
              << "\",\"cpu_active_root\":\"" << route.cpu_active_root
              << "\",\"cpu_active_ids\":"
              << ids_json(route.cpu.active_pressure_ids)
              << ",\"arithmetic_input_root\":\""
              << route.arithmetic_input_root
              << "\",\"high_root\":\"" << route.high_root
              << "\",\"low_root\":\"" << route.low_root
              << "\",\"representation_root\":\""
              << route.representation_root
              << "\",\"anchor_root\":\"not-applicable-no-anchor\""
              << "}\n";
}

int run_phase_a() {
    constexpr std::array<double, 9> centers{
        0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 1.0, 1.5, 2.0};
    const NonlocalGpuProfile profile = pair_profile();
    bool cpu_all_pass = true;
    bool permutations_all_exact = true;
    bool control_quarter_pass = false;
    bool primary_reproduced = false;
    std::string environment;
    for (double center : centers) {
        const auto state = pair_state(center);
        const Route route = run_route(profile, state);
        if (environment.empty()) environment = route.environment;
        emit_center(center, profile, state, route);
        cpu_all_pass = cpu_all_pass
            && route.cpu.failure == NonlocalGpuFailure::None;
        permutations_all_exact = permutations_all_exact
            && route.permutation_exact;
        if (center == 0.25) {
            control_quarter_pass = route.gpu.failure == NonlocalGpuFailure::None;
        }
        if (center == 0.75) {
            primary_reproduced = route.gpu.failure
                == NonlocalGpuFailure::PhysicsGateFailed;
        }
    }

    const NonlocalGpuProfile gamma_zero_profile = pair_profile(0.0);
    const auto gamma_zero_state = pair_state(0.75);
    const Route gamma_zero = run_route(gamma_zero_profile, gamma_zero_state);
    const bool gamma_zero_pass = gamma_zero.gpu.failure
            == NonlocalGpuFailure::None
        && gamma_zero.cpu.failure == NonlocalGpuFailure::None
        && gamma_zero.permutation_exact;
    emit_center(0.75, gamma_zero_profile, gamma_zero_state, gamma_zero);

    const bool binary_control = binary_identity_control_passes();
    const bool passed = cpu_all_pass && permutations_all_exact
        && control_quarter_pass && primary_reproduced && gamma_zero_pass
        && binary_control;
    const std::string executable_root = binary_root();
    std::cout << "{\"schema\":\"nextengine.nonlocal.ncgp2.phase-a.v2\""
              << ",\"kind\":\"summary\",\"research_id\":\"NCGP2\""
              << ",\"status\":\""
              << (passed ? "PHASE_A_PASS" : "INCONCLUSIVE") << "\""
              << ",\"contract_root\":\"" << NCGP2_CONTRACT_ROOT
              << "\",\"source_root\":\"" << NCGP2_SOURCE_ROOT
              << "\",\"source_commit\":\"" << NCGP2_SOURCE_COMMIT
              << "\",\"source_tree\":\"" << NCGP2_SOURCE_TREE
              << "\",\"binary_root\":\"" << executable_root
              << "\",\"compiler_flags\":\"" << NCGP2_COMPILER_FLAGS
              << "\",\"command\":\"nonlocal-corrected-cuda-surface-translation --phase-a\""
              << ",\"cpu_all_pass\":"
              << (cpu_all_pass ? "true" : "false")
              << ",\"permutations_all_exact\":"
              << (permutations_all_exact ? "true" : "false")
              << ",\"quarter_control_pass\":"
              << (control_quarter_pass ? "true" : "false")
              << ",\"primary_failure_reproduced\":"
              << (primary_reproduced ? "true" : "false")
              << ",\"gamma_zero_control_pass\":"
              << (gamma_zero_pass ? "true" : "false")
              << ",\"binary_identity_control_pass\":"
              << (binary_control ? "true" : "false")
              << ",\"environment\":" << environment << "}\n";
    return passed ? 0 : 2;
}

int run_surface_f64() {
    constexpr std::array<double, 9> centers{
        0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 1.0, 1.5, 2.0};
    const NonlocalGpuProfile profile = pair_profile();
    bool all_pass = true;
    bool cpu_all_pass = true;
    bool permutations_all_exact = true;
    bool primary_pass = false;
    double maximum_cpu_error = 0.0;
    std::string environment;
    for (double center : centers) {
        const auto state = pair_state(center);
        const Route route = run_route(
            profile, state, NonlocalGpuVariant::SurfaceF64);
        if (environment.empty()) environment = route.environment;
        emit_center(center, profile, state, route);
        const bool route_pass = route.gpu.failure == NonlocalGpuFailure::None
            && route.cpu.failure == NonlocalGpuFailure::None
            && route.permutation_exact
            && route.gpu.work.accepted_trials > 0U
            && route.gpu.active_pressure_centers
                == route.cpu.active_pressure_centers;
        double route_cpu_error = 0.0;
        if (route_pass) {
            for (const NonlocalGpuSample& gpu_sample : route.gpu.state) {
                const NonlocalGpuSample* cpu_sample = find_sample(
                    route.cpu.state, gpu_sample.sample_id);
                if (cpu_sample == nullptr) {
                    route_cpu_error = std::numeric_limits<double>::infinity();
                    break;
                }
                route_cpu_error = std::max(route_cpu_error,
                    std::abs(gpu_sample.current.x - cpu_sample->current.x));
                route_cpu_error = std::max(route_cpu_error,
                    std::abs(gpu_sample.current.y - cpu_sample->current.y));
                route_cpu_error = std::max(route_cpu_error,
                    std::abs(gpu_sample.current.z - cpu_sample->current.z));
            }
        }
        maximum_cpu_error = std::max(maximum_cpu_error, route_cpu_error);
        const bool center_pass = route_pass && route_cpu_error <= 5.0e-6;
        all_pass = all_pass && center_pass;
        cpu_all_pass = cpu_all_pass
            && route.cpu.failure == NonlocalGpuFailure::None;
        permutations_all_exact = permutations_all_exact
            && route.permutation_exact;
        if (center == 0.75) primary_pass = center_pass;
    }
    const bool binary_control = binary_identity_control_passes();
    const bool passed = all_pass && primary_pass && cpu_all_pass
        && permutations_all_exact && binary_control;
    std::cout << std::setprecision(17)
              << "{\"schema\":\"nextengine.nonlocal.ncgp2.phase-b.v2\""
              << ",\"kind\":\"summary\",\"counterfactual\":\"surface-f64\""
              << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
              << ",\"contract_root\":\"" << NCGP2_CONTRACT_ROOT
              << "\",\"source_root\":\"" << NCGP2_SOURCE_ROOT
              << "\",\"source_commit\":\"" << NCGP2_SOURCE_COMMIT
              << "\",\"source_tree\":\"" << NCGP2_SOURCE_TREE
              << "\",\"binary_root\":\"" << binary_root()
              << "\",\"compiler_flags\":\"" << NCGP2_COMPILER_FLAGS
              << "\",\"command\":\"nonlocal-corrected-cuda-surface-translation --surface-f64\""
              << ",\"primary_pass\":" << (primary_pass ? "true" : "false")
              << ",\"all_centers_pass\":" << (all_pass ? "true" : "false")
              << ",\"cpu_all_pass\":" << (cpu_all_pass ? "true" : "false")
              << ",\"permutations_all_exact\":"
              << (permutations_all_exact ? "true" : "false")
              << ",\"maximum_cpu_position_error_m\":" << maximum_cpu_error
              << ",\"binary_identity_control_pass\":"
              << (binary_control ? "true" : "false")
              << ",\"environment\":" << environment << "}\n";
    return passed ? 0 : 4;
}

double vector_relative_l2(const std::vector<Vec3d>& observed,
    const std::vector<Vec3d>& expected) {
    if (observed.size() != expected.size() || observed.empty()) {
        return std::numeric_limits<double>::infinity();
    }
    double numerator = 0.0;
    double denominator = 0.0;
    for (std::size_t index = 0U; index < observed.size(); ++index) {
        const double dx = observed[index].x - expected[index].x;
        const double dy = observed[index].y - expected[index].y;
        const double dz = observed[index].z - expected[index].z;
        numerator += dx * dx + dy * dy + dz * dz;
        denominator += expected[index].x * expected[index].x
            + expected[index].y * expected[index].y
            + expected[index].z * expected[index].z;
    }
    return std::sqrt(numerator / std::max(denominator, 1.0e-30));
}

struct SurfaceControl {
    bool passed = false;
    double gradient_relative_l2 = 0.0;
    double hvp_relative_l2 = 0.0;
    double directional_relative_error = 0.0;
    double equal_opposite_closure = 0.0;
};

struct CompensatedControls {
    bool canonical_pair_pass = false;
    bool malformed_pair_rejected = false;
    bool omitted_low_rejected = false;
    bool broken_eft_rejected = false;
    bool input_root_mismatch_rejected = false;
    bool work_mismatch_rejected = false;
    bool transaction_mismatch_rejected = false;
    bool publish_mismatch_rejected = false;
    bool permutation_identity_loss_rejected = false;
    bool representation_root_mismatch_rejected = false;
    NonlocalGpuFailure omitted_low_failure = NonlocalGpuFailure::None;
    NonlocalGpuFailure broken_eft_failure = NonlocalGpuFailure::None;

    bool passed() const {
        return canonical_pair_pass && malformed_pair_rejected
            && omitted_low_rejected && broken_eft_rejected
            && input_root_mismatch_rejected && work_mismatch_rejected
            && transaction_mismatch_rejected && publish_mismatch_rejected
            && permutation_identity_loss_rejected
            && representation_root_mismatch_rejected;
    }
};

bool canonical_binary32_pair(float high, float low) {
    return std::isfinite(high) && std::isfinite(low)
        && high + low == high
        && std::abs(low) <= 0.5F * upward_ulp(std::abs(high));
}

CompensatedControls run_compensated_negative_controls(
    const NonlocalGpuProfile& profile, const Route& baseline) {
    CompensatedControls controls;
    controls.canonical_pair_pass = canonical_binary32_pair(0.75F, 0.0F);
    controls.malformed_pair_rejected = !canonical_binary32_pair(0.75F, 0.25F);

    const auto state = pair_state(0.75);
    bool input_exact = false;
    const Route omitted = run_compensated_route(profile, state, input_exact,
        NonlocalGpuVariant::CompensatedOmitLow);
    const Route broken = run_compensated_route(profile, state, input_exact,
        NonlocalGpuVariant::CompensatedBrokenEft);
    controls.omitted_low_failure = omitted.gpu.failure;
    controls.broken_eft_failure = broken.gpu.failure;
    controls.omitted_low_rejected = omitted.gpu_result_root
        != baseline.gpu_result_root;
    controls.broken_eft_rejected = broken.gpu_result_root
        != baseline.gpu_result_root;

    auto input_mutation = state;
    input_mutation[0].current.x = static_cast<double>(std::nextafter(
        static_cast<float>(input_mutation[0].current.x),
        std::numeric_limits<float>::infinity()));
    const std::vector<NonlocalGpuGhost> no_ghosts;
    controls.input_root_mismatch_rejected = input_semantic_root(
        profile, input_mutation, no_ghosts) != baseline.input_root;

    NonlocalGpuStepResult work_mutation = baseline.gpu;
    ++work_mutation.work.compensated_difference_components;
    controls.work_mismatch_rejected = step_work_semantic_root(
        profile, work_mutation) != baseline.gpu_work_root;

    NonlocalGpuStepResult transaction_mutation = baseline.gpu;
    ++transaction_mutation.work.compensated_transaction_components;
    controls.transaction_mismatch_rejected = step_semantic_root(
        profile, baseline.input_root, transaction_mutation)
        != step_semantic_root(profile, baseline.input_root, baseline.gpu);

    NonlocalGpuStepResult publish_mutation = baseline.gpu;
    if (!publish_mutation.state.empty()) {
        publish_mutation.state[0].current.x = static_cast<double>(std::nextafter(
            static_cast<float>(publish_mutation.state[0].current.x),
            std::numeric_limits<float>::infinity()));
    }
    controls.publish_mismatch_rejected = step_semantic_root(
        profile, baseline.input_root, publish_mutation)
        != step_semantic_root(profile, baseline.input_root, baseline.gpu);

    NonlocalGpuStepResult permutation_mutation = baseline.gpu;
    std::reverse(permutation_mutation.active_pressure_ids.begin(),
        permutation_mutation.active_pressure_ids.end());
    controls.permutation_identity_loss_rejected = step_semantic_root(
        profile, baseline.input_root, permutation_mutation)
        != step_semantic_root(profile, baseline.input_root, baseline.gpu);

    std::string representation_mutation = baseline.representation_root;
    if (!representation_mutation.empty()) representation_mutation[0] =
        representation_mutation[0] == '0' ? '1' : '0';
    controls.representation_root_mismatch_rejected = close_representation_root(
        step_semantic_root(profile, baseline.input_root, baseline.gpu),
        representation_mutation) != baseline.gpu_result_root;
    return controls;
}

SurfaceControl run_compensated_surface_control() {
    NonlocalGpuProfile profile = pair_profile();
    profile.kappa = 0.0;
    profile.lambda = 0.0;
    profile.mu = 0.0;
    auto state = canonicalize_samples_binary32({
        {101U, {0.73, 0.75, 0.75}, {0.73, 0.75, 0.75}, {}},
        {202U, {0.77, 0.75, 0.75}, {0.77, 0.75, 0.75}, {}},
    });
    const std::vector<Vec3d> direction{
        {-0.3, 0.2, -0.1}, {0.3, -0.2, 0.1}};
    const std::vector<NonlocalGpuGhost> no_ghosts;
    NonlocalGpuWorkspace workspace(profile);
    SurfaceControl control;
    if (workspace.upload(state, no_ghosts, true) != NonlocalGpuFailure::None) {
        return control;
    }
    const auto gpu = workspace.evaluate(&direction,
        NonlocalGpuVariant::CompensatedStateF32, true, false);
    const auto cpu = evaluate_reference(profile, state, no_ghosts, &direction,
        NonlocalGpuVariant::Corrected);
    if (gpu.failure != NonlocalGpuFailure::None
        || cpu.failure != NonlocalGpuFailure::None
        || gpu.gradient.size() != 2U || gpu.hvp.size() != 2U) {
        return control;
    }
    control.gradient_relative_l2 = vector_relative_l2(gpu.gradient, cpu.gradient);
    control.hvp_relative_l2 = vector_relative_l2(gpu.hvp, cpu.hvp);
    const Vec3d closure{gpu.gradient[0].x + gpu.gradient[1].x,
        gpu.gradient[0].y + gpu.gradient[1].y,
        gpu.gradient[0].z + gpu.gradient[1].z};
    control.equal_opposite_closure = std::sqrt(closure.x * closure.x
        + closure.y * closure.y + closure.z * closure.z);
    long double gradient_dot = 0.0L;
    for (std::size_t index = 0U; index < direction.size(); ++index) {
        gradient_dot += static_cast<long double>(gpu.gradient[index].x)
                * static_cast<long double>(direction[index].x)
            + static_cast<long double>(gpu.gradient[index].y)
                * static_cast<long double>(direction[index].y)
            + static_cast<long double>(gpu.gradient[index].z)
                * static_cast<long double>(direction[index].z);
    }
    const long double dx = static_cast<long double>(state[0].current.x)
        - static_cast<long double>(state[1].current.x);
    const long double dy = static_cast<long double>(state[0].current.y)
        - static_cast<long double>(state[1].current.y);
    const long double dz = static_cast<long double>(state[0].current.z)
        - static_cast<long double>(state[1].current.z);
    const long double radius = std::sqrt(dx * dx + dy * dy + dz * dz);
    const long double q = radius / static_cast<long double>(profile.spacing);
    long double force = 0.0L;
    if (q <= 1.0L) {
        force = q * q - 1.0L;
    } else if (q < 3.0L) {
        const long double shifted = q - 2.0L;
        force = 1.0L - shifted * shifted;
    }
    const long double scale = 2.0L
        * static_cast<long double>(profile.gamma)
        * static_cast<long double>(profile.mass)
        * static_cast<long double>(profile.mass) * force / radius;
    const std::array<long double, 3> expected_first{
        scale * dx, scale * dy, scale * dz};
    const long double expected_dot = expected_first[0]
            * static_cast<long double>(direction[0].x - direction[1].x)
        + expected_first[1]
            * static_cast<long double>(direction[0].y - direction[1].y)
        + expected_first[2]
            * static_cast<long double>(direction[0].z - direction[1].z);
    control.directional_relative_error = static_cast<double>(std::abs(
        expected_dot - gradient_dot)
        / std::max(std::abs(expected_dot), 1.0e-30L));
    control.passed = control.gradient_relative_l2 <= 1.0e-5
        && control.hvp_relative_l2 <= 1.0e-3
        && control.directional_relative_error <= 1.0e-3
        && control.equal_opposite_closure <= 1.0e-6;
    return control;
}

int run_compensated_state_f32() {
    constexpr std::array<double, 9> centers{
        0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 1.0, 1.5, 2.0};
    const NonlocalGpuProfile profile = pair_profile();
    bool all_pass = true;
    bool input_all_exact = true;
    bool cpu_all_pass = true;
    bool permutations_all_exact = true;
    bool primary_pass = false;
    double maximum_cpu_error = 0.0;
    std::string representation_material;
    std::uint64_t input_components = 0U;
    std::uint64_t decomposition_components = 0U;
    std::uint64_t reconstruction_components = 0U;
    std::uint64_t canonical_checks = 0U;
    std::uint64_t difference_components = 0U;
    std::uint64_t inertia_components = 0U;
    std::uint64_t trial_eft_components = 0U;
    std::uint64_t transaction_components = 0U;
    std::uint64_t publish_components = 0U;
    std::string environment;
    Route primary_route;
    for (double center : centers) {
        const auto state = pair_state(center);
        bool input_exact = false;
        const Route route = run_compensated_route(
            profile, state, input_exact);
        representation_material += route.representation_root + "\n";
        const auto accumulate_representation_work = [&](const auto& work) {
            input_components += work.compensated_input_components;
            decomposition_components += work.compensated_decomposition_components;
            reconstruction_components +=
                work.compensated_reconstruction_components;
            canonical_checks += work.compensated_canonical_checks;
            difference_components += work.compensated_difference_components;
            inertia_components += work.compensated_inertia_components;
            trial_eft_components += work.compensated_trial_eft_components;
            transaction_components += work.compensated_transaction_components;
            publish_components += work.compensated_publish_components;
        };
        accumulate_representation_work(route.gpu.work);
        accumulate_representation_work(route.permuted.work);
        if (environment.empty()) environment = route.environment;
        emit_center(center, profile, state, route);
        const bool route_pass = input_exact
            && route.gpu.failure == NonlocalGpuFailure::None
            && route.cpu.failure == NonlocalGpuFailure::None
            && route.permutation_exact
            && route.gpu.work.accepted_trials > 0U
            && route.gpu.active_pressure_ids == route.cpu.active_pressure_ids;
        double route_cpu_error = 0.0;
        if (route_pass) {
            for (const NonlocalGpuSample& gpu_sample : route.gpu.state) {
                const NonlocalGpuSample* cpu_sample = find_sample(
                    route.cpu.state, gpu_sample.sample_id);
                if (cpu_sample == nullptr) {
                    route_cpu_error = std::numeric_limits<double>::infinity();
                    break;
                }
                route_cpu_error = std::max(route_cpu_error,
                    std::abs(gpu_sample.current.x - cpu_sample->current.x));
                route_cpu_error = std::max(route_cpu_error,
                    std::abs(gpu_sample.current.y - cpu_sample->current.y));
                route_cpu_error = std::max(route_cpu_error,
                    std::abs(gpu_sample.current.z - cpu_sample->current.z));
            }
        }
        maximum_cpu_error = std::max(maximum_cpu_error, route_cpu_error);
        const bool center_pass = route_pass && route_cpu_error <= 5.0e-6;
        all_pass = all_pass && center_pass;
        input_all_exact = input_all_exact && input_exact;
        cpu_all_pass = cpu_all_pass
            && route.cpu.failure == NonlocalGpuFailure::None;
        permutations_all_exact = permutations_all_exact
            && route.permutation_exact;
        if (center == 0.75) {
            primary_pass = center_pass;
            primary_route = route;
        }
    }
    const SurfaceControl surface_control = run_compensated_surface_control();
    const CompensatedControls controls = run_compensated_negative_controls(
        profile, primary_route);
    const std::string representation_work_root =
        nextengine::nonlocal::sha256_hex(
            std::string("nextengine.nonlocal.ncgp2.compensated-work.v2\n")
            + std::to_string(input_components) + "\n"
            + std::to_string(decomposition_components) + "\n"
            + std::to_string(reconstruction_components) + "\n"
            + std::to_string(canonical_checks) + "\n"
            + std::to_string(difference_components) + "\n"
            + std::to_string(inertia_components) + "\n"
            + std::to_string(trial_eft_components) + "\n"
            + std::to_string(transaction_components) + "\n"
            + std::to_string(publish_components) + "\n"
            + representation_material);
    const bool binary_control = binary_identity_control_passes();
    const bool passed = all_pass && primary_pass && input_all_exact
        && cpu_all_pass && permutations_all_exact && surface_control.passed
        && binary_control && controls.passed();
    std::cout << std::setprecision(17)
              << "{\"schema\":\"nextengine.nonlocal.ncgp2.phase-b.v2\""
              << ",\"kind\":\"summary\",\"counterfactual\":\"compensated-state-f32\""
              << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
              << ",\"contract_root\":\"" << NCGP2_CONTRACT_ROOT
              << "\",\"source_root\":\"" << NCGP2_SOURCE_ROOT
              << "\",\"source_commit\":\"" << NCGP2_SOURCE_COMMIT
              << "\",\"source_tree\":\"" << NCGP2_SOURCE_TREE
              << "\",\"binary_root\":\"" << binary_root()
              << "\",\"compiler_flags\":\"" << NCGP2_COMPILER_FLAGS
              << "\",\"command\":\"nonlocal-corrected-cuda-surface-translation --compensated-state-f32\""
              << ",\"primary_pass\":" << (primary_pass ? "true" : "false")
              << ",\"all_centers_pass\":" << (all_pass ? "true" : "false")
              << ",\"input_all_exact\":" << (input_all_exact ? "true" : "false")
              << ",\"cpu_all_pass\":" << (cpu_all_pass ? "true" : "false")
              << ",\"permutations_all_exact\":"
              << (permutations_all_exact ? "true" : "false")
              << ",\"maximum_cpu_position_error_m\":" << maximum_cpu_error
              << ",\"representation\":\"canonical-binary32-hi-lo-device-state\""
              << ",\"compensated_input_components\":" << input_components
              << ",\"compensated_decomposition_components\":"
              << decomposition_components
              << ",\"compensated_reconstruction_components\":"
              << reconstruction_components
              << ",\"compensated_canonical_checks\":" << canonical_checks
              << ",\"compensated_difference_components\":"
              << difference_components
              << ",\"compensated_inertia_components\":" << inertia_components
              << ",\"compensated_trial_eft_components\":"
              << trial_eft_components
              << ",\"compensated_transaction_components\":"
              << transaction_components
              << ",\"compensated_publish_components\":" << publish_components
              << ",\"representation_work_root\":\""
              << representation_work_root << "\""
              << ",\"surface_control_pass\":"
              << (surface_control.passed ? "true" : "false")
              << ",\"surface_gradient_relative_l2\":"
              << surface_control.gradient_relative_l2
              << ",\"surface_hvp_relative_l2\":"
              << surface_control.hvp_relative_l2
              << ",\"surface_directional_relative_error\":"
              << surface_control.directional_relative_error
              << ",\"surface_equal_opposite_closure\":"
              << surface_control.equal_opposite_closure
              << ",\"binary_identity_control_pass\":"
              << (binary_control ? "true" : "false")
              << ",\"compensated_controls_pass\":"
              << (controls.passed() ? "true" : "false")
              << ",\"canonical_pair_control_pass\":"
              << (controls.canonical_pair_pass ? "true" : "false")
              << ",\"malformed_pair_control_pass\":"
              << (controls.malformed_pair_rejected ? "true" : "false")
              << ",\"omitted_low_control_pass\":"
              << (controls.omitted_low_rejected ? "true" : "false")
              << ",\"omitted_low_failure\":"
              << static_cast<std::uint32_t>(controls.omitted_low_failure)
              << ",\"broken_eft_control_pass\":"
              << (controls.broken_eft_rejected ? "true" : "false")
              << ",\"broken_eft_failure\":"
              << static_cast<std::uint32_t>(controls.broken_eft_failure)
              << ",\"input_root_control_pass\":"
              << (controls.input_root_mismatch_rejected ? "true" : "false")
              << ",\"work_root_control_pass\":"
              << (controls.work_mismatch_rejected ? "true" : "false")
              << ",\"transaction_root_control_pass\":"
              << (controls.transaction_mismatch_rejected ? "true" : "false")
              << ",\"publish_root_control_pass\":"
              << (controls.publish_mismatch_rejected ? "true" : "false")
              << ",\"permutation_identity_control_pass\":"
              << (controls.permutation_identity_loss_rejected
                      ? "true" : "false")
              << ",\"representation_root_control_pass\":"
              << (controls.representation_root_mismatch_rejected
                      ? "true" : "false")
              << ",\"environment\":" << environment << "}\n";
    return passed ? 0 : 5;
}

} // namespace

int main(int argc, char** argv) {
    if (argc == 2 && std::string(argv[1]) == "--phase-a") {
        try {
            return run_phase_a();
        } catch (const std::exception& error) {
            std::cerr << "NCGP2 CUDA failure: " << error.what() << '\n';
            return 3;
        }
    }
    if (argc == 2 && std::string(argv[1]) == "--surface-f64") {
        try {
            return run_surface_f64();
        } catch (const std::exception& error) {
            std::cerr << "NCGP2 CUDA failure: " << error.what() << '\n';
            return 3;
        }
    }
    if (argc == 2 && std::string(argv[1]) == "--compensated-state-f32") {
        try {
            return run_compensated_state_f32();
        } catch (const std::exception& error) {
            std::cerr << "NCGP2 CUDA failure: " << error.what() << '\n';
            return 3;
        }
    }
    std::cerr << "usage: nonlocal-corrected-cuda-surface-translation "
                 "--phase-a|--surface-f64|--compensated-state-f32\n";
    return 64;
}
