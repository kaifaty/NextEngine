#include "term_controls.hpp"

#include "math.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <iomanip>
#include <sstream>
#include <string>
#include <vector>

namespace nextengine::nonlocal::npr1 {
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
    double implementation_force_projection = 0.0;
    double repair_probe_force_projection = 0.0;
    double derivative_relative_error = 0.0;
    double implementation_relative_error = 0.0;
    double repair_probe_relative_error = 0.0;
    double translation_relative_error = 0.0;
    double closure_relative_error = 0.0;
    bool passed = false;
};

double relative_error(double lhs, double rhs) {
    return std::abs(lhs - rhs)
        / std::max({std::abs(lhs), std::abs(rhs), 1.0e-30});
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

double source_cubic_gradient(double radius, double horizon) {
    // Independent transcription of the selected PeriDyno/NR0 arithmetic.
    // It returns dW/dq. NPR1-B deliberately does not repair it to dW/dr.
    const double q = 2.0 * radius / horizon;
    const double alpha = 3.0 / (2.0 * PI * horizon * horizon * horizon);
    if (q > 2.0) {
        return 0.0;
    }
    if (q >= 1.0) {
        const double delta = 2.0 - q;
        return -0.5 * alpha * delta * delta;
    }
    return alpha * (-2.0 * q + 1.5 * q * q);
}

double energy_cubic_gradient(double radius, double horizon) {
    // W(r) is parameterized by q=2r/h, hence dW/dr=(dW/dq)*(2/h).
    return source_cubic_gradient(radius, horizon) * (2.0 / horizon);
}

double surface_positive(double radius) {
    const double q = radius / SPACING;
    if (q <= 1.0) {
        return q * q;
    }
    if (q <= 3.0) {
        return 1.0 - (q - 2.0) * (q - 2.0);
    }
    return 0.0;
}

double surface_negative(double radius) {
    return radius / SPACING <= 1.0 ? -1.0 : 0.0;
}

double surface_potential(double radius) {
    const double q = radius / SPACING;
    if (q <= 1.0) {
        return q * q * q / 3.0 - q + 2.0 / 3.0;
    }
    if (q <= 3.0) {
        return q - (q - 2.0) * (q - 2.0) * (q - 2.0) / 3.0 - 4.0 / 3.0;
    }
    return 0.0;
}

Vec3 normalized(Vec3 value) {
    return value / norm(value);
}

Vec3 cross(Vec3 lhs, Vec3 rhs) {
    return {
        lhs.y * rhs.z - lhs.z * rhs.y,
        lhs.z * rhs.x - lhs.x * rhs.z,
        lhs.x * rhs.y - lhs.y * rhs.x,
    };
}

Vec3 project_normal(Vec3 value, Vec3 normal) {
    return dot(value, normal) * normal;
}

Vec3 project_tangent(Vec3 value, Vec3 normal) {
    return value - project_normal(value, normal);
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

DirectionalCheck incompressibility_check() {
    constexpr double kappa = 9196.875;
    const Vec3 first{0.0, 0.0, 0.0};
    const Vec3 second{0.061, 0.017, -0.009};
    const Vec3 direction = normalized({0.43, -0.31, 0.847});
    const double initial_radius = norm(first - second);
    const double initial_density =
        MASS * (cubic_weight(0.0, HORIZON) + cubic_weight(initial_radius, HORIZON));
    const double control_rest_density = initial_density / 1.1;
    const auto energy = [=](Vec3 a, Vec3 b) {
        const double density = MASS
            * (cubic_weight(0.0, HORIZON)
                + cubic_weight(norm(a - b), HORIZON));
        const double error = density / control_rest_density - 1.0;
        return kappa * error * error;
    };
    const double finite_difference = central_directional_derivative(
        energy, first, second, direction, HORIZON * 1.0e-7);
    const Vec3 pair_direction = (first - second) / initial_radius;
    const double error = initial_density / control_rest_density - 1.0;
    const Vec3 analytic_gradient = 2.0 * kappa * error * MASS
        / control_rest_density * energy_cubic_gradient(initial_radius, HORIZON)
        * pair_direction;

    // m/dt^2 times (source-K*y) from the selected Eq. 26 arithmetic.
    const Vec3 implementation_force = -2.0 * MASS * kappa * error
        / control_rest_density * source_cubic_gradient(initial_radius, HORIZON)
        * pair_direction;
    const Vec3 repair_probe_force = -2.0 * MASS * kappa * error
        / control_rest_density * energy_cubic_gradient(initial_radius, HORIZON)
        * pair_direction;
    const double force_projection = dot(implementation_force, direction);
    const double translated_energy =
        energy(first + Vec3{1.25, -0.75, 0.5}, second + Vec3{1.25, -0.75, 0.5});
    const double translation_error = relative_error(energy(first, second), translated_energy);
    const double closure_error = norm(implementation_force + (-implementation_force))
        / std::max(2.0 * norm(implementation_force), 1.0e-30);

    DirectionalCheck result;
    result.name = "incompressibility";
    result.finite_difference = finite_difference;
    result.analytic_derivative = dot(analytic_gradient, direction);
    result.implementation_force_projection = force_projection;
    result.repair_probe_force_projection = dot(repair_probe_force, direction);
    result.derivative_relative_error =
        relative_error(result.finite_difference, result.analytic_derivative);
    result.implementation_relative_error =
        relative_error(-result.finite_difference, result.implementation_force_projection);
    result.repair_probe_relative_error =
        relative_error(-result.finite_difference, result.repair_probe_force_projection);
    result.translation_relative_error = translation_error;
    result.closure_relative_error = closure_error;
    result.passed = result.derivative_relative_error <= DERIVATIVE_LIMIT
        && result.implementation_relative_error <= DERIVATIVE_LIMIT
        && translation_error <= CLOSURE_LIMIT && closure_error <= CLOSURE_LIMIT;
    return result;
}

DirectionalCheck viscosity_check(bool shear) {
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
    const double influence = -energy_cubic_gradient(radius, HORIZON);
    const auto energy = [=](Vec3 a, Vec3 b) {
        const Vec3 velocity_delta = ((a - b) - reference_delta) / TIME_STEP;
        if (shear) {
            const Vec3 component = project_tangent(velocity_delta, normal);
            // dt times the directed-pair total in paper Eq. 10.
            return TIME_STEP * MASS / REST_DENSITY * norm_squared(component) * influence;
        }
        const Vec3 component = project_normal(velocity_delta, normal);
        return TIME_STEP * MASS / REST_DENSITY * 0.5
            * norm_squared(component) * influence;
    };
    const double finite_difference = central_directional_derivative(
        energy, first, second, direction, HORIZON * 1.0e-8);
    const Vec3 increment = (first - second) - reference_delta;
    const Vec3 component = shear
        ? project_tangent(increment, normal)
        : project_normal(increment, normal);
    const double analytic_factor = shear ? 2.0 : 1.0;
    const Vec3 analytic_gradient = analytic_factor * MASS / REST_DENSITY
        * influence / TIME_STEP * component;

    // The selected source path uses zeroth-order W rather than paper Eq. 10's
    // omega=-dW/dr, and applies the same endpoint factor to bulk and shear.
    const double source_factor = TIME_STEP / REST_DENSITY
        * cubic_weight(radius, HORIZON);
    const Vec3 implementation_force = -2.0 * MASS / (TIME_STEP * TIME_STEP)
        * source_factor * component;
    const double repair_source_factor = TIME_STEP / REST_DENSITY
        * influence * (shear ? 1.0 : 0.5);
    const Vec3 repair_probe_force = -2.0 * MASS / (TIME_STEP * TIME_STEP)
        * repair_source_factor * component;
    const double base_energy = energy(first, second);
    const Vec3 shift{0.7, -1.1, 0.3};
    const auto translated_energy = [=]() {
        const Vec3 shifted_reference_delta =
            (reference_first + shift) - (reference_second + shift);
        const Vec3 velocity_delta =
            (((first + shift) - (second + shift)) - shifted_reference_delta) / TIME_STEP;
        const Vec3 shifted_normal = shifted_reference_delta / norm(shifted_reference_delta);
        const Vec3 shifted_component = shear
            ? project_tangent(velocity_delta, shifted_normal)
            : project_normal(velocity_delta, shifted_normal);
        return TIME_STEP * MASS / REST_DENSITY * (shear ? 1.0 : 0.5)
            * norm_squared(shifted_component) * influence;
    }();
    const double closure_error = norm(implementation_force + (-implementation_force))
        / std::max(2.0 * norm(implementation_force), 1.0e-30);

    DirectionalCheck result;
    result.name = shear ? "shear_viscosity" : "bulk_viscosity";
    result.finite_difference = finite_difference;
    result.analytic_derivative = dot(analytic_gradient, direction);
    result.implementation_force_projection = dot(implementation_force, direction);
    result.repair_probe_force_projection = dot(repair_probe_force, direction);
    result.derivative_relative_error =
        relative_error(result.finite_difference, result.analytic_derivative);
    result.implementation_relative_error =
        relative_error(-result.finite_difference, result.implementation_force_projection);
    result.repair_probe_relative_error =
        relative_error(-result.finite_difference, result.repair_probe_force_projection);
    result.translation_relative_error = relative_error(base_energy, translated_energy);
    result.closure_relative_error = closure_error;
    result.passed = result.derivative_relative_error <= DERIVATIVE_LIMIT
        && result.implementation_relative_error <= DERIVATIVE_LIMIT
        && result.translation_relative_error <= CLOSURE_LIMIT
        && result.closure_relative_error <= CLOSURE_LIMIT;
    return result;
}

DirectionalCheck surface_check() {
    constexpr double gamma = 3.5;
    const Vec3 first{0.0, 0.0, 0.0};
    const Vec3 second{0.081, -0.006, 0.004};
    const Vec3 direction = normalized({0.61, 0.22, -0.761});
    const auto energy = [=](Vec3 a, Vec3 b) {
        return 2.0 * gamma * MASS * MASS * surface_potential(norm(a - b));
    };
    const double radius = norm(first - second);
    const Vec3 pair_direction = (first - second) / radius;
    const double pair_value = surface_positive(radius) + surface_negative(radius);
    const double finite_difference = central_directional_derivative(
        energy, first, second, direction, SPACING * 1.0e-7);
    const Vec3 analytic_gradient = 2.0 * gamma * MASS * MASS
        * pair_value / SPACING * pair_direction;

    // The selected source coefficient is gamma*dt^2, so conversion back to
    // force contributes m but not the m/spacing factor in paper Eq. 14.
    const Vec3 implementation_force =
        -2.0 * gamma * MASS * pair_value * pair_direction;
    const Vec3 repair_probe_force = -2.0 * gamma * MASS * MASS / SPACING
        * pair_value * pair_direction;
    const Vec3 shift{-0.45, 0.8, 1.2};
    const double closure_error = norm(implementation_force + (-implementation_force))
        / std::max(2.0 * norm(implementation_force), 1.0e-30);

    DirectionalCheck result;
    result.name = "surface_tension";
    result.finite_difference = finite_difference;
    result.analytic_derivative = dot(analytic_gradient, direction);
    result.implementation_force_projection = dot(implementation_force, direction);
    result.repair_probe_force_projection = dot(repair_probe_force, direction);
    result.derivative_relative_error =
        relative_error(result.finite_difference, result.analytic_derivative);
    result.implementation_relative_error =
        relative_error(-result.finite_difference, result.implementation_force_projection);
    result.repair_probe_relative_error =
        relative_error(-result.finite_difference, result.repair_probe_force_projection);
    result.translation_relative_error =
        relative_error(energy(first, second), energy(first + shift, second + shift));
    result.closure_relative_error = closure_error;
    result.passed = result.derivative_relative_error <= DERIVATIVE_LIMIT
        && result.implementation_relative_error <= DERIVATIVE_LIMIT
        && result.translation_relative_error <= CLOSURE_LIMIT
        && result.closure_relative_error <= CLOSURE_LIMIT;
    return result;
}

struct RotationResult {
    bool passed = false;
    double rigid_translation_shear = 0.0;
    std::vector<double> mu_values;
    std::vector<double> kinetic_energies;
    std::vector<double> angular_momentum_ratios;
};

RotationResult rotating_sphere_check() {
    const std::vector<Vec3> reference = {
        {0.03, 0.0, 0.0}, {-0.03, 0.0, 0.0},
        {0.0, 0.03, 0.0}, {0.0, -0.03, 0.0},
        {0.0, 0.0, 0.03}, {0.0, 0.0, -0.03},
    };
    const Vec3 omega{0.0, 0.0, 20.0};
    std::vector<Vec3> predicted;
    predicted.reserve(reference.size());
    for (Vec3 position : reference) {
        predicted.push_back(position + TIME_STEP * cross(omega, position));
    }

    const auto solve = [&](double mu) {
        std::vector<Vec3> velocity(reference.size());
        for (std::size_t i = 0; i < reference.size(); ++i) {
            Mat3 matrix;
            Vec3 source;
            for (std::size_t j = 0; j < reference.size(); ++j) {
                if (i == j) {
                    continue;
                }
                const Vec3 reference_delta = reference[j] - reference[i];
                const double radius = norm(reference_delta);
                if (radius > HORIZON) {
                    continue;
                }
                const Vec3 normal = reference_delta / radius;
                const Mat3 tangent = Mat3::identity() - outer(normal, normal);
                const Mat3 pair_matrix = mu * TIME_STEP / REST_DENSITY
                    * cubic_weight(radius, HORIZON) * tangent;
                matrix += 2.0 * pair_matrix;
                source += 2.0 * (pair_matrix
                    * (predicted[j] - reference_delta));
            }
            const Vec3 next = inverse_without_regularization(Mat3::identity() + matrix)
                * (predicted[i] + source);
            velocity[i] = (next - reference[i]) / TIME_STEP;
        }
        double kinetic = 0.0;
        Vec3 angular;
        for (std::size_t i = 0; i < reference.size(); ++i) {
            kinetic += 0.5 * MASS * norm_squared(velocity[i]);
            angular += MASS * cross(reference[i], velocity[i]);
        }
        return std::array<double, 2>{kinetic, norm(angular)};
    };

    RotationResult result;
    result.mu_values = {0.0, 10.0, 100.0, 1000.0};
    std::vector<double> angular;
    for (double mu : result.mu_values) {
        const auto observables = solve(mu);
        result.kinetic_energies.push_back(observables[0]);
        angular.push_back(observables[1]);
    }
    const double angular_base = std::max(angular.front(), 1.0e-30);
    for (double value : angular) {
        result.angular_momentum_ratios.push_back(value / angular_base);
    }

    const Vec3 rigid_velocity{0.7, -0.4, 0.2};
    const Vec3 pair_a{-0.02, 0.01, 0.0};
    const Vec3 pair_b{0.04, -0.015, 0.008};
    const Vec3 translated_a = pair_a + TIME_STEP * rigid_velocity;
    const Vec3 translated_b = pair_b + TIME_STEP * rigid_velocity;
    const Vec3 reference_delta = pair_a - pair_b;
    const Vec3 normal = reference_delta / norm(reference_delta);
    result.rigid_translation_shear = norm(project_tangent(
        ((translated_a - translated_b) - reference_delta) / TIME_STEP, normal));

    bool monotonic = true;
    for (std::size_t i = 1; i < result.kinetic_energies.size(); ++i) {
        monotonic = monotonic
            && result.kinetic_energies[i]
                <= result.kinetic_energies[i - 1] * (1.0 + CLOSURE_LIMIT);
    }
    result.passed = monotonic && result.rigid_translation_shear <= CLOSURE_LIMIT;
    return result;
}

void append_directional(std::ostringstream& output, const DirectionalCheck& value) {
    output << "{\"name\":\"" << value.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL")
           << "\",\"finite_difference\":" << value.finite_difference
           << ",\"analytic_derivative\":" << value.analytic_derivative
           << ",\"implementation_force_projection\":"
           << value.implementation_force_projection
           << ",\"repair_probe_force_projection\":"
           << value.repair_probe_force_projection
           << ",\"derivative_relative_error\":" << value.derivative_relative_error
           << ",\"implementation_relative_error\":"
           << value.implementation_relative_error
           << ",\"repair_probe_relative_error\":"
           << value.repair_probe_relative_error
           << ",\"translation_relative_error\":"
           << value.translation_relative_error
           << ",\"closure_relative_error\":" << value.closure_relative_error << '}';
}

} // namespace

TermControlReport run_term_controls() {
    constexpr double golden_weight_zero = 94.3140403507528;
    constexpr double golden_weight_half_support = 23.5785100876882;
    constexpr double golden_source_gradient_half_support = -70.7355302630646;
    constexpr double golden_energy_gradient_half_support = -943.140403507528;
    const double derivative_radius = 0.06;
    const double derivative_epsilon = HORIZON * 1.0e-7;
    const double finite_difference_gradient =
        (cubic_weight(derivative_radius + derivative_epsilon, HORIZON)
            - cubic_weight(derivative_radius - derivative_epsilon, HORIZON))
        / (2.0 * derivative_epsilon);
    const double source_gradient = source_cubic_gradient(derivative_radius, HORIZON);
    const double energy_gradient = energy_cubic_gradient(derivative_radius, HORIZON);
    const bool value_goldens =
        relative_error(cubic_weight(0.0, HORIZON), golden_weight_zero) <= 1.0e-15
        && relative_error(cubic_weight(0.5 * HORIZON, HORIZON),
               golden_weight_half_support)
            <= 1.0e-15
        && cubic_weight(HORIZON, HORIZON) == 0.0
        && cubic_weight(HORIZON * 1.01, HORIZON) == 0.0;
    const bool gradient_goldens =
        relative_error(source_cubic_gradient(0.5 * HORIZON, HORIZON),
            golden_source_gradient_half_support)
            <= 1.0e-15
        && relative_error(energy_cubic_gradient(0.5 * HORIZON, HORIZON),
               golden_energy_gradient_half_support)
            <= 1.0e-15;
    const double energy_gradient_error =
        relative_error(finite_difference_gradient, energy_gradient);
    const double source_gradient_error =
        relative_error(finite_difference_gradient, source_gradient);
    const bool kernel_passed = value_goldens && gradient_goldens
        && energy_gradient_error <= DERIVATIVE_LIMIT
        && source_gradient_error <= DERIVATIVE_LIMIT;

    const std::array<DirectionalCheck, 4> directional = {
        incompressibility_check(),
        viscosity_check(false),
        viscosity_check(true),
        surface_check(),
    };
    const RotationResult rotation = rotating_sphere_check();
    bool terms_passed = true;
    std::string first_failure;
    if (!kernel_passed) {
        first_failure = "KERNEL_SOURCE_GRADIENT_MISMATCH";
    }
    for (const DirectionalCheck& check : directional) {
        terms_passed = terms_passed && check.passed;
        if (first_failure.empty() && !check.passed) {
            first_failure = "TERM_IMPLEMENTATION_ENERGY_MISMATCH:" + check.name;
        }
    }
    if (first_failure.empty() && !rotation.passed) {
        first_failure = "VISCOSITY_ROTATION_CONTROL_FAILED";
    }
    const bool passed = kernel_passed && terms_passed && rotation.passed;

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.npr1_term_controls.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"thresholds\":{\"derivative_relative\":" << DERIVATIVE_LIMIT
           << ",\"translation_and_closure\":" << CLOSURE_LIMIT << '}'
           << ",\"kernel\":{\"status\":\"" << (kernel_passed ? "PASS" : "FAIL")
           << "\",\"value_goldens\":" << (value_goldens ? "true" : "false")
           << ",\"gradient_goldens\":" << (gradient_goldens ? "true" : "false")
           << ",\"finite_difference_gradient\":" << finite_difference_gradient
           << ",\"energy_gradient\":" << energy_gradient
           << ",\"source_gradient\":" << source_gradient
           << ",\"energy_gradient_relative_error\":" << energy_gradient_error
           << ",\"source_gradient_relative_error\":" << source_gradient_error
           << ",\"source_to_energy_gradient_ratio\":"
           << source_gradient / energy_gradient << '}'
           << ",\"directional_terms\":[";
    for (std::size_t i = 0; i < directional.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_directional(report, directional[i]);
    }
    report << "],\"rotating_sphere\":{\"status\":\""
           << (rotation.passed ? "PASS" : "FAIL")
           << "\",\"rigid_translation_shear\":" << rotation.rigid_translation_shear
           << ",\"mu\":[";
    for (std::size_t i = 0; i < rotation.mu_values.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        report << rotation.mu_values[i];
    }
    report << "],\"kinetic_energy\":[";
    for (std::size_t i = 0; i < rotation.kinetic_energies.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        report << rotation.kinetic_energies[i];
    }
    report << "],\"angular_momentum_ratio\":[";
    for (std::size_t i = 0; i < rotation.angular_momentum_ratios.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        report << rotation.angular_momentum_ratios[i];
    }
    report << "]},\"poiseuille\":{\"status\":\"NOT_RUN_AFTER_FIRST_FAILURE\"}"
           << ",\"convergence_16_32_64\":{\"status\":\"NOT_RUN_AFTER_FIRST_FAILURE\"}"
           << ",\"runtime_authority\":false,\"npr2_authorized\":false";
    const std::string result_material = std::string(passed ? "PASS|" : "FAIL|")
        + first_failure + '|' + std::to_string(source_gradient_error) + '|'
        + std::to_string(directional[0].implementation_relative_error) + '|'
        + std::to_string(directional[1].implementation_relative_error) + '|'
        + std::to_string(directional[2].implementation_relative_error) + '|'
        + std::to_string(directional[3].implementation_relative_error);
    report << ",\"result_sha256\":\"" << sha256_hex(result_material) << "\"}";
    return {passed, report.str()};
}

} // namespace nextengine::nonlocal::npr1
