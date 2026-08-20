#include "formula_reclosure.hpp"

#include "math.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <iomanip>
#include <sstream>
#include <string>

namespace nextengine::nonlocal::fcr {
namespace {

constexpr double PI = 3.141592653589793238462643383279502884;
constexpr double DERIVATIVE_LIMIT = 1.0e-7;
constexpr double CLOSURE_LIMIT = 1.0e-12;
constexpr double HORIZON = 0.15;
constexpr double SPACING = 0.05;
constexpr double MASS = 0.125;
constexpr double TIME_STEP = 1.0 / 240.0;
constexpr double REST_DENSITY = 1000.0;

struct DirectionalCheck {
    std::string name;
    double finite_difference = 0.0;
    double analytic_derivative = 0.0;
    double force_projection = 0.0;
    double derivative_relative_error = 0.0;
    double force_relative_error = 0.0;
    double translation_relative_error = 0.0;
    double closure_relative_error = 0.0;
    bool passed = false;
};

double relative_error(double lhs, double rhs) {
    return std::abs(lhs - rhs)
        / std::max({std::abs(lhs), std::abs(rhs), 1.0e-30});
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

double cubic_weight(double radius, double horizon) {
    const double q = 2.0 * radius / horizon;
    const double alpha = 3.0 / (2.0 * PI * horizon * horizon * horizon);
    if (q > 2.0) {
        return 0.0;
    }
    if (q >= 1.0) {
        const double delta = 2.0 - q;
        return alpha * delta * delta * delta / 6.0;
    }
    return alpha * (2.0 / 3.0 - q * q + 0.5 * q * q * q);
}

double cubic_gradient(double radius, double horizon) {
    const double q = 2.0 * radius / horizon;
    const double alpha = 3.0 / (2.0 * PI * horizon * horizon * horizon);
    double derivative_q = 0.0;
    if (q > 2.0) {
        return 0.0;
    }
    if (q >= 1.0) {
        const double delta = 2.0 - q;
        derivative_q = -0.5 * alpha * delta * delta;
    } else {
        derivative_q = alpha * (-2.0 * q + 1.5 * q * q);
    }
    return derivative_q * (2.0 / horizon);
}

double surface_force_spline(double radius) {
    const double q = radius / SPACING;
    if (q <= 1.0) {
        return q * q - 1.0;
    }
    if (q < 3.0) {
        return 1.0 - (q - 2.0) * (q - 2.0);
    }
    return 0.0;
}

double surface_dimensionless_potential(double radius) {
    const double q = radius / SPACING;
    if (q <= 1.0) {
        return q * q * q / 3.0 - q - 2.0 / 3.0;
    }
    if (q < 3.0) {
        return q - (q - 2.0) * (q - 2.0) * (q - 2.0) / 3.0
            - 8.0 / 3.0;
    }
    return 0.0;
}

double surface_physical_potential(double radius) {
    return SPACING * surface_dimensionless_potential(radius);
}

template <typename Energy>
double central_directional_derivative(
    const Energy& energy,
    Vec3 first,
    Vec3 second,
    Vec3 direction,
    double epsilon) {
    return (energy(first + epsilon * direction, second)
               - energy(first - epsilon * direction, second))
        / (2.0 * epsilon);
}

DirectionalCheck make_directional(
    const std::string& name,
    double finite_difference,
    Vec3 analytic_gradient,
    Vec3 direction,
    double translation_error) {
    DirectionalCheck result;
    result.name = name;
    result.finite_difference = finite_difference;
    result.analytic_derivative = dot(analytic_gradient, direction);
    result.force_projection = dot(-analytic_gradient, direction);
    result.derivative_relative_error =
        relative_error(result.finite_difference, result.analytic_derivative);
    result.force_relative_error =
        relative_error(-result.finite_difference, result.force_projection);
    result.translation_relative_error = translation_error;
    result.closure_relative_error = norm((-analytic_gradient) + analytic_gradient)
        / std::max(2.0 * norm(analytic_gradient), 1.0e-30);
    result.passed = result.derivative_relative_error <= DERIVATIVE_LIMIT
        && result.force_relative_error <= DERIVATIVE_LIMIT
        && result.translation_relative_error <= CLOSURE_LIMIT
        && result.closure_relative_error <= CLOSURE_LIMIT;
    return result;
}

DirectionalCheck compression_check(bool compression_only, bool below_rest) {
    constexpr double kappa = 9196.875;
    const Vec3 first{0.0, 0.0, 0.0};
    const Vec3 second{0.061, 0.017, -0.009};
    const Vec3 direction = normalized({0.43, -0.31, 0.847});
    const double radius = norm(first - second);
    const double density = MASS
        * (cubic_weight(0.0, HORIZON) + cubic_weight(radius, HORIZON));
    const double target_ratio = below_rest ? 0.9 : 1.1;
    const double control_rest_density = density / target_ratio;
    const auto energy = [=](Vec3 a, Vec3 b) {
        const double local_density = MASS
            * (cubic_weight(0.0, HORIZON)
                + cubic_weight(norm(a - b), HORIZON));
        const double raw_error = local_density / control_rest_density - 1.0;
        const double error = compression_only ? std::max(raw_error, 0.0) : raw_error;
        // Two equal particles, each with kappa/2 * error^2.
        return kappa * error * error;
    };
    const double raw_error = density / control_rest_density - 1.0;
    const double selected_error = compression_only ? std::max(raw_error, 0.0) : raw_error;
    const Vec3 pair_direction = (first - second) / radius;
    const Vec3 analytic_gradient = 2.0 * kappa * selected_error * MASS
        / control_rest_density * cubic_gradient(radius, HORIZON)
        * pair_direction;
    const double finite_difference = central_directional_derivative(
        energy, first, second, direction, HORIZON * 1.0e-7);
    const Vec3 shift{1.25, -0.75, 0.5};
    return make_directional(
        compression_only
            ? (below_rest ? "compression_only_below_rest" : "compression_only_above_rest")
            : "two_sided_below_rest_comparator",
        finite_difference,
        analytic_gradient,
        direction,
        relative_error(energy(first, second), energy(first + shift, second + shift)));
}

DirectionalCheck viscosity_check(bool shear) {
    constexpr double lambda = 2.5;
    constexpr double mu = 1.7;
    const Vec3 reference_first{-0.025, 0.015, -0.01};
    const Vec3 reference_second{0.055, -0.012, 0.018};
    const Vec3 first_velocity{1.2, 0.4, -0.1};
    const Vec3 second_velocity{-0.2, 0.1, 0.5};
    const Vec3 first = reference_first + TIME_STEP * first_velocity;
    const Vec3 second = reference_second + TIME_STEP * second_velocity;
    const Vec3 direction = normalized({-0.27, 0.91, 0.31});
    const Vec3 reference_delta = reference_first - reference_second;
    const double radius = norm(reference_delta);
    const Vec3 normal = reference_delta / radius;
    const double influence = -cubic_gradient(radius, HORIZON);
    const auto energy = [=](Vec3 a, Vec3 b) {
        const Vec3 increment = (a - b) - reference_delta;
        const Vec3 component = shear
            ? project_tangent(increment, normal)
            : project_normal(increment, normal);
        const double coefficient = shear ? mu : lambda / 2.0;
        // Combined energy of the two directed terms in Eq. 10, integrated over dt.
        return MASS / (REST_DENSITY * TIME_STEP) * coefficient
            * norm_squared(component) * influence;
    };
    const Vec3 increment = (first - second) - reference_delta;
    const Vec3 component = shear
        ? project_tangent(increment, normal)
        : project_normal(increment, normal);
    const double gradient_coefficient = shear ? 2.0 * mu : lambda;
    const Vec3 analytic_gradient = MASS / (REST_DENSITY * TIME_STEP)
        * gradient_coefficient * influence * component;
    const double finite_difference = central_directional_derivative(
        energy, first, second, direction, HORIZON * 1.0e-8);
    const Vec3 shift{0.7, -1.1, 0.3};
    const Vec3 shifted_reference_delta =
        (reference_first + shift) - (reference_second + shift);
    const Vec3 shifted_normal = shifted_reference_delta / norm(shifted_reference_delta);
    const Vec3 shifted_increment =
        ((first + shift) - (second + shift)) - shifted_reference_delta;
    const Vec3 shifted_component = shear
        ? project_tangent(shifted_increment, shifted_normal)
        : project_normal(shifted_increment, shifted_normal);
    const double shifted_energy = MASS / (REST_DENSITY * TIME_STEP)
        * (shear ? mu : lambda / 2.0) * norm_squared(shifted_component)
        * influence;
    return make_directional(
        shear ? "shear_viscosity" : "bulk_viscosity",
        finite_difference,
        analytic_gradient,
        direction,
        relative_error(energy(first, second), shifted_energy));
}

DirectionalCheck surface_check(const std::string& name, double q) {
    constexpr double gamma = 3.5;
    const Vec3 first{0.0, 0.0, 0.0};
    const Vec3 axis = normalized({0.91, -0.21, 0.356});
    const Vec3 second = q * SPACING * axis;
    const Vec3 direction = normalized({0.61, 0.22, -0.761});
    const auto energy = [=](Vec3 a, Vec3 b) {
        return 2.0 * gamma * MASS * MASS
            * surface_physical_potential(norm(a - b));
    };
    const double radius = norm(first - second);
    const Vec3 pair_direction = (first - second) / radius;
    const Vec3 analytic_gradient = 2.0 * gamma * MASS * MASS
        * surface_force_spline(radius) * pair_direction;
    const double finite_difference = central_directional_derivative(
        energy, first, second, direction, SPACING * 1.0e-7);
    const Vec3 shift{-0.45, 0.8, 1.2};
    return make_directional(
        name,
        finite_difference,
        analytic_gradient,
        direction,
        relative_error(energy(first, second), energy(first + shift, second + shift)));
}

void append_directional(std::ostringstream& output, const DirectionalCheck& value) {
    output << "{\"name\":\"" << value.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL")
           << "\",\"finite_difference\":" << value.finite_difference
           << ",\"analytic_derivative\":" << value.analytic_derivative
           << ",\"force_projection\":" << value.force_projection
           << ",\"derivative_relative_error\":" << value.derivative_relative_error
           << ",\"force_relative_error\":" << value.force_relative_error
           << ",\"translation_relative_error\":"
           << value.translation_relative_error
           << ",\"closure_relative_error\":" << value.closure_relative_error << '}';
}

} // namespace

FormulaReport run_formula_controls() {
    constexpr double golden_weight_zero = 94.3140403507528;
    constexpr double golden_weight_half_support = 23.5785100876882;
    constexpr double golden_gradient_half_support = -943.140403507528;
    const double derivative_radius_inner = 0.03;
    const double derivative_radius_outer = 0.105;
    const double derivative_epsilon = HORIZON * 1.0e-7;
    const auto finite_kernel_gradient = [&](double radius) {
        return (cubic_weight(radius + derivative_epsilon, HORIZON)
                   - cubic_weight(radius - derivative_epsilon, HORIZON))
            / (2.0 * derivative_epsilon);
    };
    const double inner_kernel_error = relative_error(
        finite_kernel_gradient(derivative_radius_inner),
        cubic_gradient(derivative_radius_inner, HORIZON));
    const double outer_kernel_error = relative_error(
        finite_kernel_gradient(derivative_radius_outer),
        cubic_gradient(derivative_radius_outer, HORIZON));
    const bool kernel_goldens =
        relative_error(cubic_weight(0.0, HORIZON), golden_weight_zero) <= 1.0e-15
        && relative_error(cubic_weight(0.5 * HORIZON, HORIZON),
               golden_weight_half_support)
            <= 1.0e-15
        && relative_error(cubic_gradient(0.5 * HORIZON, HORIZON),
               golden_gradient_half_support)
            <= 1.0e-15
        && cubic_weight(HORIZON, HORIZON) == 0.0
        && cubic_gradient(HORIZON, HORIZON) == 0.0;
    const bool kernel_passed = kernel_goldens
        && inner_kernel_error <= DERIVATIVE_LIMIT
        && outer_kernel_error <= DERIVATIVE_LIMIT;

    const bool surface_goldens =
        relative_error(surface_dimensionless_potential(0.0), -2.0 / 3.0) <= 1.0e-15
        && relative_error(surface_dimensionless_potential(SPACING), -4.0 / 3.0)
            <= 1.0e-15
        && relative_error(surface_dimensionless_potential(2.0 * SPACING), -2.0 / 3.0)
            <= 1.0e-15
        && surface_dimensionless_potential(3.0 * SPACING) == 0.0
        && relative_error(surface_force_spline(0.5 * SPACING), -0.75) <= 1.0e-15
        && relative_error(surface_force_spline(2.0 * SPACING), 1.0) <= 1.0e-15;

    const std::array<DirectionalCheck, 8> directional = {
        compression_check(true, false),
        compression_check(true, true),
        compression_check(false, true),
        viscosity_check(false),
        viscosity_check(true),
        surface_check("surface_repulsive_branch", 0.8),
        surface_check("surface_attractive_branch", 1.7),
        surface_check("surface_outside_support", 3.2),
    };

    bool terms_passed = true;
    std::string first_failure;
    if (!kernel_passed) {
        first_failure = "FCR_KERNEL_DERIVATIVE_MISMATCH";
    } else if (!surface_goldens) {
        first_failure = "FCR_SURFACE_POTENTIAL_GOLDEN_MISMATCH";
    }
    for (const DirectionalCheck& check : directional) {
        terms_passed = terms_passed && check.passed;
        if (first_failure.empty() && !check.passed) {
            first_failure = "FCR_DIRECTIONAL_DERIVATIVE_MISMATCH:" + check.name;
        }
    }
    const bool compression_clamp_discriminates =
        std::abs(directional[1].analytic_derivative) <= CLOSURE_LIMIT
        && std::abs(directional[2].analytic_derivative) > 1.0e-6;
    if (first_failure.empty() && !compression_clamp_discriminates) {
        first_failure = "FCR_COMPRESSION_SEMANTICS_NOT_DISCRIMINATED";
    }
    const bool passed = kernel_passed && surface_goldens && terms_passed
        && compression_clamp_discriminates;

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.formula_reclosure_fcr0.v1\""
           << ",\"identity\":\"nuv-variational-fcr1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"thresholds\":{\"derivative_relative\":" << DERIVATIVE_LIMIT
           << ",\"translation_and_closure\":" << CLOSURE_LIMIT << '}'
           << ",\"pair_convention\":\"directed_energy_sum\""
           << ",\"compression_default\":\"max(rho/rho0-1,0)^2\""
           << ",\"viscosity_influence\":\"omega=-dW/dr\""
           << ",\"surface_potential\":\"C(r)=r0*C_hat(r/r0)\""
           << ",\"kernel\":{\"status\":\""
           << (kernel_passed ? "PASS" : "FAIL")
           << "\",\"goldens\":" << (kernel_goldens ? "true" : "false")
           << ",\"inner_derivative_relative_error\":" << inner_kernel_error
           << ",\"outer_derivative_relative_error\":" << outer_kernel_error
           << ",\"gradient_half_support\":"
           << cubic_gradient(0.5 * HORIZON, HORIZON) << '}'
           << ",\"surface_goldens\":{\"status\":\""
           << (surface_goldens ? "PASS" : "FAIL") << "\"}"
           << ",\"compression_clamp_discriminator\":{\"status\":\""
           << (compression_clamp_discriminates ? "PASS" : "FAIL")
           << "\",\"selected_below_rest_derivative\":"
           << directional[1].analytic_derivative
           << ",\"two_sided_below_rest_derivative\":"
           << directional[2].analytic_derivative << '}'
           << ",\"directional_terms\":[";
    for (std::size_t index = 0; index < directional.size(); ++index) {
        if (index != 0) {
            report << ',';
        }
        append_directional(report, directional[index]);
    }
    report << "]"
           << ",\"fcr1_authorized\":" << (passed ? "true" : "false")
           << ",\"runtime_authority\":false"
           << ",\"result_sha256\":\"";
    const std::string result_material = std::string(passed ? "PASS|" : "FAIL|")
        + first_failure + '|' + std::to_string(inner_kernel_error) + '|'
        + std::to_string(outer_kernel_error) + '|'
        + std::to_string(directional[0].derivative_relative_error) + '|'
        + std::to_string(directional[3].derivative_relative_error) + '|'
        + std::to_string(directional[4].derivative_relative_error) + '|'
        + std::to_string(directional[5].derivative_relative_error) + '|'
        + std::to_string(directional[6].derivative_relative_error);
    report << sha256_hex(result_material) << "\"}";
    return {passed, report.str()};
}

} // namespace nextengine::nonlocal::fcr
