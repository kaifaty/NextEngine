#include "formula_discriminators.hpp"

#include "math.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <iomanip>
#include <sstream>
#include <string>
#include <vector>

namespace nextengine::nonlocal::fcr {
namespace {

constexpr double PI = 3.141592653589793238462643383279502884;
constexpr double LIMIT = 1.0e-12;
constexpr double REJECTION_FLOOR = 0.49;
constexpr double HORIZON = 0.15;
constexpr double SPACING = 0.05;
constexpr double MASS = 0.125;
constexpr double REST_DENSITY = 1000.0;
constexpr double TIME_STEP = 1.0 / 240.0;
constexpr double KAPPA = 9196.875;

double vector_relative_error(Vec3 lhs, Vec3 rhs) {
    return norm(lhs - rhs) / std::max({norm(lhs), norm(rhs), 1.0e-30});
}

Vec3 normalized(Vec3 value) {
    return value / norm(value);
}

Vec3 project_normal(Vec3 value, Vec3 normal) {
    return dot(value, normal) * normal;
}

Vec3 project_tangent(Vec3 value, Vec3 normal) {
    return value - project_normal(value, normal);
}

double cubic_weight(double radius) {
    const double q = 2.0 * radius / HORIZON;
    const double alpha = 3.0 / (2.0 * PI * HORIZON * HORIZON * HORIZON);
    if (q > 2.0) {
        return 0.0;
    }
    if (q >= 1.0) {
        const double delta = 2.0 - q;
        return alpha * delta * delta * delta / 6.0;
    }
    return alpha * (2.0 / 3.0 - q * q + 0.5 * q * q * q);
}

double cubic_gradient(double radius) {
    const double q = 2.0 * radius / HORIZON;
    const double alpha = 3.0 / (2.0 * PI * HORIZON * HORIZON * HORIZON);
    if (q > 2.0) {
        return 0.0;
    }
    const double derivative_q = q >= 1.0
        ? -0.5 * alpha * (2.0 - q) * (2.0 - q)
        : alpha * (-2.0 * q + 1.5 * q * q);
    return derivative_q * (2.0 / HORIZON);
}

double surface_spline(double radius) {
    const double q = radius / SPACING;
    if (q <= 1.0) {
        return q * q - 1.0;
    }
    if (q < 3.0) {
        return 1.0 - (q - 2.0) * (q - 2.0);
    }
    return 0.0;
}

struct EnumerationResult {
    Vec3 expected_viscosity;
    Vec3 unique_full_viscosity;
    Vec3 directed_half_viscosity;
    Vec3 directed_full_viscosity;
    Vec3 unique_half_viscosity;
    Vec3 expected_surface;
    Vec3 unique_full_surface;
    Vec3 directed_half_surface;
    Vec3 directed_full_surface;
    Vec3 source_gamma_surface;
    Vec3 source_gamma_mass_surface;
    double maximum_closure_residual = 0.0;
};

std::array<Vec3, 2> accumulate_pair(
    Vec3 full_first_update,
    int visits,
    double per_visit_scale) {
    std::array<Vec3, 2> result{};
    for (int visit = 0; visit < visits; ++visit) {
        const Vec3 contribution = per_visit_scale * full_first_update;
        result[0] += contribution;
        result[1] += -contribution;
    }
    return result;
}

EnumerationResult enumeration_control() {
    constexpr double lambda = 2.5;
    constexpr double mu = 1.7;
    constexpr double gamma = 3.5;
    const Vec3 reference_first{-0.025, 0.015, -0.01};
    const Vec3 reference_second{0.055, -0.012, 0.018};
    const Vec3 first_velocity{1.2, 0.4, -0.1};
    const Vec3 second_velocity{-0.2, 0.1, 0.5};
    const Vec3 first = reference_first + TIME_STEP * first_velocity;
    const Vec3 second = reference_second + TIME_STEP * second_velocity;
    const Vec3 reference_delta = reference_first - reference_second;
    const Vec3 increment = (first - second) - reference_delta;
    const double radius = norm(reference_delta);
    const Vec3 normal = reference_delta / radius;
    const double omega = -cubic_gradient(radius);
    const Vec3 full_viscosity = -TIME_STEP * omega / REST_DENSITY
        * (2.0 * mu * project_tangent(increment, normal)
            + lambda * project_normal(increment, normal));

    const double surface_radius = 1.7 * SPACING;
    const Vec3 surface_normal = normalized({0.91, -0.21, 0.356});
    const double c = surface_spline(surface_radius);
    const Vec3 full_surface = -2.0 * gamma * MASS * TIME_STEP * TIME_STEP
        * c * surface_normal;
    const Vec3 source_gamma_one_directed = -gamma * TIME_STEP * TIME_STEP
        * c * surface_normal;
    const Vec3 source_gamma_mass_one_directed =
        -gamma * MASS * TIME_STEP * TIME_STEP * c * surface_normal;

    const auto unique_viscosity = accumulate_pair(full_viscosity, 1, 1.0);
    const auto directed_half_viscosity = accumulate_pair(full_viscosity, 2, 0.5);
    const auto directed_full_viscosity = accumulate_pair(full_viscosity, 2, 1.0);
    const auto unique_half_viscosity = accumulate_pair(full_viscosity, 1, 0.5);
    const auto unique_surface = accumulate_pair(full_surface, 1, 1.0);
    const auto directed_half_surface = accumulate_pair(full_surface, 2, 0.5);
    const auto directed_full_surface = accumulate_pair(full_surface, 2, 1.0);
    const auto source_gamma_surface =
        accumulate_pair(source_gamma_one_directed, 2, 1.0);
    const auto source_gamma_mass_surface =
        accumulate_pair(source_gamma_mass_one_directed, 2, 1.0);

    EnumerationResult result;
    result.expected_viscosity = full_viscosity;
    result.unique_full_viscosity = unique_viscosity[0];
    result.directed_half_viscosity = directed_half_viscosity[0];
    result.directed_full_viscosity = directed_full_viscosity[0];
    result.unique_half_viscosity = unique_half_viscosity[0];
    result.expected_surface = full_surface;
    result.unique_full_surface = unique_surface[0];
    result.directed_half_surface = directed_half_surface[0];
    result.directed_full_surface = directed_full_surface[0];
    result.source_gamma_surface = source_gamma_surface[0];
    result.source_gamma_mass_surface = source_gamma_mass_surface[0];
    const std::array<std::array<Vec3, 2>, 9> accumulated = {
        unique_viscosity,
        directed_half_viscosity,
        directed_full_viscosity,
        unique_half_viscosity,
        unique_surface,
        directed_half_surface,
        directed_full_surface,
        source_gamma_surface,
        source_gamma_mass_surface,
    };
    for (const auto& endpoints : accumulated) {
        const double scale = std::max(
            norm(endpoints[0]) + norm(endpoints[1]), 1.0e-30);
        result.maximum_closure_residual = std::max(
            result.maximum_closure_residual,
            norm(endpoints[0] + endpoints[1]) / scale);
    }
    return result;
}

struct PressureResult {
    double minimum_density = 0.0;
    double maximum_density = 0.0;
    double maximum_force = 0.0;
    double corner_inward_force = 0.0;
    double normalized_momentum_residual = 0.0;
};

PressureResult pressure_forces(
    const std::vector<Vec3>& positions,
    bool compression_only) {
    std::vector<double> density(positions.size(), MASS * cubic_weight(0.0));
    for (std::size_t i = 0; i < positions.size(); ++i) {
        for (std::size_t j = i + 1; j < positions.size(); ++j) {
            const double radius = norm(positions[i] - positions[j]);
            if (radius <= HORIZON) {
                const double contribution = MASS * cubic_weight(radius);
                density[i] += contribution;
                density[j] += contribution;
            }
        }
    }
    std::vector<double> error(positions.size());
    for (std::size_t i = 0; i < positions.size(); ++i) {
        const double raw = density[i] / REST_DENSITY - 1.0;
        error[i] = compression_only ? std::max(raw, 0.0) : raw;
    }
    std::vector<Vec3> force(positions.size());
    for (std::size_t i = 0; i < positions.size(); ++i) {
        for (std::size_t j = i + 1; j < positions.size(); ++j) {
            const Vec3 displacement = positions[i] - positions[j];
            const double radius = norm(displacement);
            if (radius <= 1.0e-12 || radius > HORIZON) {
                continue;
            }
            const Vec3 normal = displacement / radius;
            const double coefficient = KAPPA * MASS / REST_DENSITY
                * (error[i] + error[j]) * cubic_gradient(radius);
            const Vec3 pair_force = -coefficient * normal;
            force[i] += pair_force;
            force[j] += -pair_force;
        }
    }
    Vec3 total_force;
    double force_sum = 0.0;
    double maximum_force = 0.0;
    for (Vec3 value : force) {
        total_force += value;
        force_sum += norm(value);
        maximum_force = std::max(maximum_force, norm(value));
    }
    Vec3 center;
    for (Vec3 position : positions) {
        center += position;
    }
    center = center / static_cast<double>(positions.size());
    const Vec3 inward = normalized(center - positions.front());

    PressureResult result;
    result.minimum_density = *std::min_element(density.begin(), density.end());
    result.maximum_density = *std::max_element(density.begin(), density.end());
    result.maximum_force = maximum_force;
    result.corner_inward_force = dot(force.front(), inward);
    result.normalized_momentum_residual =
        norm(total_force) / std::max(force_sum, 1.0e-30);
    return result;
}

std::vector<Vec3> make_patch() {
    std::vector<Vec3> positions;
    positions.reserve(27);
    for (int z = -1; z <= 1; ++z) {
        for (int y = -1; y <= 1; ++y) {
            for (int x = -1; x <= 1; ++x) {
                positions.push_back({x * SPACING, y * SPACING, z * SPACING});
            }
        }
    }
    return positions;
}

} // namespace

DiscriminatorReport run_pair_pressure_discriminator() {
    const EnumerationResult enumeration = enumeration_control();
    const double unique_viscosity_error = vector_relative_error(
        enumeration.unique_full_viscosity, enumeration.expected_viscosity);
    const double directed_half_viscosity_error = vector_relative_error(
        enumeration.directed_half_viscosity, enumeration.expected_viscosity);
    const double directed_full_viscosity_error = vector_relative_error(
        enumeration.directed_full_viscosity, enumeration.expected_viscosity);
    const double unique_half_viscosity_error = vector_relative_error(
        enumeration.unique_half_viscosity, enumeration.expected_viscosity);
    const double unique_surface_error = vector_relative_error(
        enumeration.unique_full_surface, enumeration.expected_surface);
    const double directed_half_surface_error = vector_relative_error(
        enumeration.directed_half_surface, enumeration.expected_surface);
    const double directed_full_surface_error = vector_relative_error(
        enumeration.directed_full_surface, enumeration.expected_surface);
    const double source_gamma_error = vector_relative_error(
        enumeration.source_gamma_surface, enumeration.expected_surface);
    const double source_gamma_mass_error = vector_relative_error(
        enumeration.source_gamma_mass_surface, enumeration.expected_surface);

    const std::vector<Vec3> pair = {
        {-0.04, 0.0, 0.0},
        {0.04, 0.0, 0.0},
    };
    const PressureResult pair_selected = pressure_forces(pair, true);
    const PressureResult pair_two_sided = pressure_forces(pair, false);
    const std::vector<Vec3> patch = make_patch();
    const PressureResult patch_selected = pressure_forces(patch, true);
    const PressureResult patch_two_sided = pressure_forces(patch, false);

    const bool enumeration_passed = unique_viscosity_error <= LIMIT
        && directed_half_viscosity_error <= LIMIT
        && directed_full_viscosity_error >= REJECTION_FLOOR
        && unique_half_viscosity_error >= REJECTION_FLOOR
        && unique_surface_error <= LIMIT
        && directed_half_surface_error <= LIMIT
        && directed_full_surface_error >= REJECTION_FLOOR
        && enumeration.maximum_closure_residual <= LIMIT;
    const bool pressure_passed = pair_selected.maximum_force <= LIMIT
        && patch_selected.maximum_force <= LIMIT
        && pair_two_sided.corner_inward_force > 1.0e-6
        && patch_two_sided.corner_inward_force > 1.0e-6
        && pair_two_sided.normalized_momentum_residual <= LIMIT
        && patch_two_sided.normalized_momentum_residual <= LIMIT;
    const bool surface_mapping_passed = source_gamma_mass_error <= LIMIT
        && source_gamma_error >= REJECTION_FLOOR;

    std::string first_failure;
    if (!enumeration_passed) {
        first_failure = "FCR1_PAIR_ENUMERATION_MISMATCH";
    } else if (!pressure_passed) {
        first_failure = "FCR1_PRESSURE_SEMANTICS_MISMATCH";
    } else if (!surface_mapping_passed) {
        first_failure = "FCR1_SURFACE_PARAMETER_MAPPING_MISMATCH";
    }
    const bool passed = enumeration_passed && pressure_passed
        && surface_mapping_passed;

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.formula_reclosure_fcr1.v1\""
           << ",\"identity\":\"nuv-variational-fcr1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"thresholds\":{\"correspondence\":" << LIMIT
           << ",\"rejection_floor\":" << REJECTION_FLOOR
           << ",\"minimum_inward_force\":1e-6}"
           << ",\"pair_enumeration\":{\"status\":\""
           << (enumeration_passed ? "PASS" : "FAIL")
           << "\",\"selected\":\"unique-undirected-full-endpoint-scatter\""
           << ",\"unique_full_viscosity_error\":" << unique_viscosity_error
           << ",\"directed_half_viscosity_error\":"
           << directed_half_viscosity_error
           << ",\"directed_full_viscosity_error\":"
           << directed_full_viscosity_error
           << ",\"unique_half_viscosity_error\":" << unique_half_viscosity_error
           << ",\"unique_full_surface_error\":" << unique_surface_error
           << ",\"directed_half_surface_error\":" << directed_half_surface_error
           << ",\"directed_full_surface_error\":" << directed_full_surface_error
           << ",\"maximum_closure_residual\":"
           << enumeration.maximum_closure_residual
           << '}'
           << ",\"pressure_semantics\":{\"status\":\""
           << (pressure_passed ? "PASS" : "FAIL")
           << "\",\"selected\":\"compression-only\""
           << ",\"pair\":{\"minimum_density\":" << pair_selected.minimum_density
           << ",\"maximum_density\":" << pair_selected.maximum_density
           << ",\"selected_maximum_force\":" << pair_selected.maximum_force
           << ",\"two_sided_corner_inward_force\":"
           << pair_two_sided.corner_inward_force
           << ",\"two_sided_momentum_residual\":"
           << pair_two_sided.normalized_momentum_residual << '}'
           << ",\"patch_3x3x3\":{\"minimum_density\":"
           << patch_selected.minimum_density
           << ",\"maximum_density\":" << patch_selected.maximum_density
           << ",\"selected_maximum_force\":" << patch_selected.maximum_force
           << ",\"two_sided_corner_inward_force\":"
           << patch_two_sided.corner_inward_force
           << ",\"two_sided_maximum_force\":" << patch_two_sided.maximum_force
           << ",\"two_sided_momentum_residual\":"
           << patch_two_sided.normalized_momentum_residual << "}}"
           << ",\"surface_parameter_mapping\":{\"status\":\""
           << (surface_mapping_passed ? "PASS" : "FAIL")
           << "\",\"selected\":\"strength=gamma*m\""
           << ",\"strength_gamma_error\":" << source_gamma_error
           << ",\"strength_gamma_mass_error\":" << source_gamma_mass_error << '}'
           << ",\"fcr2_authorized\":" << (passed ? "true" : "false")
           << ",\"runtime_authority\":false";
    const std::string result_material = std::string(passed ? "PASS|" : "FAIL|")
        + first_failure + '|' + std::to_string(directed_full_viscosity_error) + '|'
        + std::to_string(patch_two_sided.corner_inward_force) + '|'
        + std::to_string(source_gamma_error);
    report << ",\"result_sha256\":\"" << sha256_hex(result_material) << "\"}";
    return {passed, report.str()};
}

} // namespace nextengine::nonlocal::fcr
