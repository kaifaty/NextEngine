#include "variational_reference.hpp"

#include "math.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <iomanip>
#include <limits>
#include <sstream>
#include <string>
#include <vector>

namespace nextengine::nonlocal::fcr {
namespace {

constexpr double PI = 3.141592653589793238462643383279502884;
constexpr double DERIVATIVE_LIMIT = 1.0e-7;
constexpr double CONSERVATION_LIMIT = 1.0e-12;
constexpr double ENERGY_ALLOWANCE = 1.0e-12;
constexpr double OBSERVABLE_FLOOR = 1.0e-9;

struct Config {
    double horizon = 0.15;
    double spacing = 0.05;
    double mass = 0.125;
    double rest_density = 1000.0;
    double time_step = 1.0 / 240.0;
    double kappa = 0.0;
    double lambda = 0.0;
    double mu = 0.0;
    double gamma = 0.0;
    Vec3 gravity{};
    int maximum_iterations = 80;
};

struct Evaluation {
    double total = 0.0;
    double inertia = 0.0;
    double pressure = 0.0;
    double viscosity = 0.0;
    double surface = 0.0;
    double gradient_norm = 0.0;
    double internal_momentum_residual = 0.0;
    double minimum_density_ratio = 0.0;
    double maximum_density_ratio = 0.0;
    std::vector<Vec3> gradient;
    bool finite = true;
};

struct SolveResult {
    bool succeeded = false;
    bool monotonic = true;
    int iterations = 0;
    int backtracks = 0;
    double minimum_alpha = 1.0;
    Evaluation initial;
    Evaluation final;
    std::vector<Vec3> position;
    std::vector<Vec3> velocity;
};

struct CaseResult {
    std::string name;
    bool passed = false;
    SolveResult solve;
    double primary_before = 0.0;
    double primary_after = 0.0;
    double observable_change = 0.0;
    double center_of_mass_error = 0.0;
};

enum class Preconditioner {
    Inertial,
    BlockGaussNewton,
};

double relative_error(double lhs, double rhs) {
    return std::abs(lhs - rhs)
        / std::max({std::abs(lhs), std::abs(rhs), 1.0e-30});
}

double vector_relative_error(Vec3 lhs, Vec3 rhs) {
    return norm(lhs - rhs) / std::max({norm(lhs), norm(rhs), 1.0});
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
    if (q > 2.0) {
        return 0.0;
    }
    const double derivative_q = q >= 1.0
        ? -0.5 * alpha * (2.0 - q) * (2.0 - q)
        : alpha * (-2.0 * q + 1.5 * q * q);
    return derivative_q * (2.0 / horizon);
}

double surface_spline(double radius, double spacing) {
    const double q = radius / spacing;
    if (q <= 1.0) {
        return q * q - 1.0;
    }
    if (q < 3.0) {
        return 1.0 - (q - 2.0) * (q - 2.0);
    }
    return 0.0;
}

double surface_spline_derivative(double radius, double spacing) {
    const double q = radius / spacing;
    if (q <= 1.0) {
        return 2.0 * q / spacing;
    }
    if (q < 3.0) {
        return -2.0 * (q - 2.0) / spacing;
    }
    return 0.0;
}

double surface_potential(double radius, double spacing) {
    const double q = radius / spacing;
    if (q <= 1.0) {
        return spacing * (q * q * q / 3.0 - q - 2.0 / 3.0);
    }
    if (q < 3.0) {
        return spacing
            * (q - (q - 2.0) * (q - 2.0) * (q - 2.0) / 3.0
                - 8.0 / 3.0);
    }
    return 0.0;
}

double vector_norm(const std::vector<Vec3>& values) {
    double squared = 0.0;
    for (Vec3 value : values) {
        squared += norm_squared(value);
    }
    return std::sqrt(squared);
}

double vector_dot(const std::vector<Vec3>& lhs, const std::vector<Vec3>& rhs) {
    double value = 0.0;
    for (std::size_t i = 0; i < lhs.size(); ++i) {
        value += dot(lhs[i], rhs[i]);
    }
    return value;
}

bool all_finite(const std::vector<Vec3>& values) {
    for (Vec3 value : values) {
        if (!finite(value)) {
            return false;
        }
    }
    return true;
}

Evaluation evaluate(
    const Config& config,
    const std::vector<Vec3>& x,
    const std::vector<Vec3>& y_star,
    const std::vector<Vec3>& y) {
    Evaluation result;
    result.gradient.resize(y.size());
    std::vector<Vec3> inertia_gradient(y.size());
    const double inertia_scale = config.mass
        / (config.time_step * config.time_step);
    for (std::size_t i = 0; i < y.size(); ++i) {
        const Vec3 displacement = y[i] - y_star[i];
        result.inertia += 0.5 * inertia_scale * norm_squared(displacement);
        inertia_gradient[i] = inertia_scale * displacement;
        result.gradient[i] += inertia_gradient[i];
    }

    std::vector<double> density(
        y.size(), config.mass * cubic_weight(0.0, config.horizon));
    for (std::size_t i = 0; i < y.size(); ++i) {
        for (std::size_t j = i + 1; j < y.size(); ++j) {
            const double radius = norm(y[i] - y[j]);
            if (radius <= config.horizon) {
                const double contribution =
                    config.mass * cubic_weight(radius, config.horizon);
                density[i] += contribution;
                density[j] += contribution;
            }
        }
    }
    std::vector<double> compression(y.size());
    result.minimum_density_ratio = std::numeric_limits<double>::infinity();
    result.maximum_density_ratio = 0.0;
    for (std::size_t i = 0; i < y.size(); ++i) {
        const double ratio = density[i] / config.rest_density;
        result.minimum_density_ratio = std::min(result.minimum_density_ratio, ratio);
        result.maximum_density_ratio = std::max(result.maximum_density_ratio, ratio);
        compression[i] = std::max(ratio - 1.0, 0.0);
        result.pressure += 0.5 * config.kappa
            * compression[i] * compression[i];
    }
    for (std::size_t i = 0; i < y.size(); ++i) {
        for (std::size_t j = i + 1; j < y.size(); ++j) {
            const Vec3 displacement = y[i] - y[j];
            const double radius = norm(displacement);
            if (radius <= 1.0e-15 || radius > config.horizon) {
                continue;
            }
            const double coefficient = config.kappa * config.mass
                / config.rest_density * (compression[i] + compression[j])
                * cubic_gradient(radius, config.horizon);
            const Vec3 pair_gradient = coefficient * (displacement / radius);
            result.gradient[i] += pair_gradient;
            result.gradient[j] += -pair_gradient;
        }
    }

    for (std::size_t i = 0; i < y.size(); ++i) {
        for (std::size_t j = i + 1; j < y.size(); ++j) {
            const Vec3 reference = x[i] - x[j];
            const double radius = norm(reference);
            if (radius <= 1.0e-15 || radius > config.horizon) {
                continue;
            }
            const Vec3 normal = reference / radius;
            const Vec3 increment = (y[i] - y[j]) - reference;
            const Vec3 normal_increment = project_normal(increment, normal);
            const Vec3 tangent_increment = project_tangent(increment, normal);
            const double omega = -cubic_gradient(radius, config.horizon);
            result.viscosity += config.mass / (config.rest_density * config.time_step)
                * (config.mu * norm_squared(tangent_increment)
                    + 0.5 * config.lambda * norm_squared(normal_increment))
                * omega;
            const Vec3 pair_gradient = config.mass * omega
                / (config.rest_density * config.time_step)
                * (2.0 * config.mu * tangent_increment
                    + config.lambda * normal_increment);
            result.gradient[i] += pair_gradient;
            result.gradient[j] += -pair_gradient;
        }
    }

    const double surface_support = 3.0 * config.spacing;
    for (std::size_t i = 0; i < y.size(); ++i) {
        for (std::size_t j = i + 1; j < y.size(); ++j) {
            const Vec3 displacement = y[i] - y[j];
            const double radius = norm(displacement);
            if (radius <= 1.0e-15 || radius >= surface_support) {
                continue;
            }
            result.surface += 2.0 * config.gamma * config.mass * config.mass
                * surface_potential(radius, config.spacing);
            const Vec3 pair_gradient = 2.0 * config.gamma * config.mass
                * config.mass * surface_spline(radius, config.spacing)
                * (displacement / radius);
            result.gradient[i] += pair_gradient;
            result.gradient[j] += -pair_gradient;
        }
    }

    result.total = result.inertia + result.pressure
        + result.viscosity + result.surface;
    result.gradient_norm = vector_norm(result.gradient);
    Vec3 internal_sum;
    double internal_scale = 0.0;
    for (std::size_t i = 0; i < y.size(); ++i) {
        const Vec3 internal = result.gradient[i] - inertia_gradient[i];
        internal_sum += internal;
        internal_scale += norm(internal);
    }
    result.internal_momentum_residual =
        norm(internal_sum) / std::max(internal_scale, 1.0e-30);
    result.finite = std::isfinite(result.total)
        && std::isfinite(result.gradient_norm)
        && std::isfinite(result.internal_momentum_residual)
        && all_finite(result.gradient);
    return result;
}

std::vector<Mat3> block_preconditioner(
    const Config& config,
    const std::vector<Vec3>& x,
    const std::vector<Vec3>& y) {
    const double inertia_scale = config.mass
        / (config.time_step * config.time_step);
    std::vector<Mat3> blocks(y.size(), inertia_scale * Mat3::identity());

    if (config.kappa != 0.0) {
        std::vector<double> density(
            y.size(), config.mass * cubic_weight(0.0, config.horizon));
        for (std::size_t i = 0; i < y.size(); ++i) {
            for (std::size_t j = i + 1; j < y.size(); ++j) {
                const double radius = norm(y[i] - y[j]);
                if (radius <= config.horizon) {
                    const double contribution =
                        config.mass * cubic_weight(radius, config.horizon);
                    density[i] += contribution;
                    density[j] += contribution;
                }
            }
        }
        for (std::size_t center = 0; center < y.size(); ++center) {
            if (density[center] <= config.rest_density) {
                continue;
            }
            std::vector<Vec3> jacobian(y.size());
            for (std::size_t neighbor = 0; neighbor < y.size(); ++neighbor) {
                if (neighbor == center) {
                    continue;
                }
                const Vec3 displacement = y[center] - y[neighbor];
                const double radius = norm(displacement);
                if (radius <= 1.0e-15 || radius > config.horizon) {
                    continue;
                }
                const Vec3 pair_jacobian = config.mass / config.rest_density
                    * cubic_gradient(radius, config.horizon)
                    * (displacement / radius);
                jacobian[center] += pair_jacobian;
                jacobian[neighbor] += -pair_jacobian;
            }
            for (std::size_t i = 0; i < y.size(); ++i) {
                blocks[i] += config.kappa * outer(jacobian[i], jacobian[i]);
            }
        }
    }

    for (std::size_t i = 0; i < y.size(); ++i) {
        for (std::size_t j = i + 1; j < y.size(); ++j) {
            const Vec3 reference = x[i] - x[j];
            const double radius = norm(reference);
            if (radius <= 1.0e-15 || radius > config.horizon) {
                continue;
            }
            const Vec3 normal = reference / radius;
            const Mat3 normal_projection = outer(normal, normal);
            const Mat3 tangent_projection = Mat3::identity() - normal_projection;
            const double scale = config.mass
                * (-cubic_gradient(radius, config.horizon))
                / (config.rest_density * config.time_step);
            const Mat3 curvature = scale
                * (2.0 * config.mu * tangent_projection
                    + config.lambda * normal_projection);
            blocks[i] += curvature;
            blocks[j] += curvature;
        }
    }

    const double surface_support = 3.0 * config.spacing;
    for (std::size_t i = 0; i < y.size(); ++i) {
        for (std::size_t j = i + 1; j < y.size(); ++j) {
            const Vec3 displacement = y[i] - y[j];
            const double radius = norm(displacement);
            if (radius <= 1.0e-15 || radius >= surface_support) {
                continue;
            }
            const Vec3 normal = displacement / radius;
            const Mat3 normal_projection = outer(normal, normal);
            const Mat3 tangent_projection = Mat3::identity() - normal_projection;
            const double energy_scale = 2.0 * config.gamma
                * config.mass * config.mass;
            const double radial = std::max(
                energy_scale * surface_spline_derivative(radius, config.spacing),
                0.0);
            const double tangential = std::max(
                energy_scale * surface_spline(radius, config.spacing) / radius,
                0.0);
            const Mat3 curvature = radial * normal_projection
                + tangential * tangent_projection;
            blocks[i] += curvature;
            blocks[j] += curvature;
        }
    }
    return blocks;
}

std::vector<Vec3> predict(
    const Config& config,
    const std::vector<Vec3>& x,
    const std::vector<Vec3>& velocity) {
    std::vector<Vec3> y_star(x.size());
    for (std::size_t i = 0; i < x.size(); ++i) {
        y_star[i] = x[i] + config.time_step
            * (velocity[i] + config.time_step * config.gravity);
    }
    return y_star;
}

SolveResult solve(
    const Config& config,
    const std::vector<Vec3>& x,
    const std::vector<Vec3>& velocity,
    Preconditioner preconditioner_kind = Preconditioner::Inertial) {
    constexpr double armijo = 1.0e-4;
    constexpr int maximum_backtracks = 40;
    const std::vector<Vec3> y_star = predict(config, x, velocity);
    std::vector<Vec3> y = y_star;
    Evaluation current = evaluate(config, x, y_star, y);

    SolveResult result;
    result.initial = current;
    result.minimum_alpha = 1.0;
    if (!current.finite) {
        result.final = current;
        return result;
    }
    const double inertial_preconditioner =
        config.time_step * config.time_step / config.mass;
    for (int iteration = 0; iteration < config.maximum_iterations; ++iteration) {
        if (current.gradient_norm <= 1.0e-10) {
            break;
        }
        std::vector<Vec3> direction(current.gradient.size());
        if (preconditioner_kind == Preconditioner::BlockGaussNewton) {
            const std::vector<Mat3> blocks = block_preconditioner(config, x, y);
            for (std::size_t i = 0; i < direction.size(); ++i) {
                direction[i] = -(inverse_without_regularization(blocks[i])
                    * current.gradient[i]);
            }
        } else {
            for (std::size_t i = 0; i < direction.size(); ++i) {
                direction[i] = -inertial_preconditioner * current.gradient[i];
            }
        }
        const double slope = vector_dot(current.gradient, direction);
        double alpha = 1.0;
        bool accepted = false;
        Evaluation trial_evaluation;
        std::vector<Vec3> trial(y.size());
        for (int backtrack = 0; backtrack <= maximum_backtracks; ++backtrack) {
            for (std::size_t i = 0; i < y.size(); ++i) {
                trial[i] = y[i] + alpha * direction[i];
            }
            trial_evaluation = evaluate(config, x, y_star, trial);
            const double armijo_bound = current.total + armijo * alpha * slope;
            if (trial_evaluation.finite && trial_evaluation.total <= armijo_bound) {
                accepted = true;
                break;
            }
            alpha *= 0.5;
            ++result.backtracks;
        }
        if (!accepted) {
            const double step_norm = alpha * vector_norm(direction);
            if (step_norm <= 1.0e-14) {
                break;
            }
            result.final = current;
            result.position = y;
            return result;
        }
        const double allowance = ENERGY_ALLOWANCE
            * std::max({std::abs(current.total), std::abs(trial_evaluation.total), 1.0});
        result.monotonic = result.monotonic
            && trial_evaluation.total <= current.total + allowance;
        y = trial;
        current = trial_evaluation;
        result.minimum_alpha = std::min(result.minimum_alpha, alpha);
        ++result.iterations;
    }
    result.succeeded = current.finite && result.monotonic;
    result.final = current;
    result.position = y;
    result.velocity.resize(y.size());
    for (std::size_t i = 0; i < y.size(); ++i) {
        result.velocity[i] = (y[i] - x[i]) / config.time_step;
    }
    return result;
}

Vec3 average(const std::vector<Vec3>& values) {
    Vec3 result;
    for (Vec3 value : values) {
        result += value;
    }
    return result / static_cast<double>(values.size());
}

double pair_density(double distance, const Config& config) {
    return config.mass
        * (cubic_weight(0.0, config.horizon)
            + cubic_weight(distance, config.horizon));
}

bool descent_passed(const SolveResult& solve_result) {
    const double allowance = ENERGY_ALLOWANCE
        * std::max({std::abs(solve_result.initial.total),
            std::abs(solve_result.final.total), 1.0});
    return solve_result.succeeded && solve_result.monotonic
        && solve_result.final.total <= solve_result.initial.total + allowance
        && solve_result.final.total < solve_result.initial.total
        && solve_result.final.gradient_norm < solve_result.initial.gradient_norm
        && solve_result.final.internal_momentum_residual <= CONSERVATION_LIMIT;
}

CaseResult free_fall_case() {
    Config config;
    config.gravity = {0.0, -9.81, 0.0};
    const std::vector<Vec3> x = {{0.1, 0.2, 0.3}};
    const std::vector<Vec3> velocity = {{1.0, -0.5, 0.2}};
    const Vec3 expected_position = x[0] + config.time_step
        * (velocity[0] + config.time_step * config.gravity);
    const Vec3 expected_velocity = velocity[0] + config.time_step * config.gravity;
    CaseResult result;
    result.name = "isolated_free_fall";
    result.solve = solve(config, x, velocity);
    const double position_error = norm(result.solve.position[0] - expected_position);
    const double velocity_error = norm(result.solve.velocity[0] - expected_velocity);
    result.primary_before = 0.0;
    result.primary_after = std::max(position_error, velocity_error);
    result.observable_change = 0.0;
    result.center_of_mass_error = velocity_error;
    result.passed = result.solve.succeeded && result.solve.iterations == 0
        && result.primary_after <= CONSERVATION_LIMIT;
    return result;
}

CaseResult compression_case() {
    Config config;
    config.kappa = 500.0;
    const std::vector<Vec3> x = {{-0.025, 0.0, 0.0}, {0.025, 0.0, 0.0}};
    const std::vector<Vec3> velocity(2);
    config.rest_density = pair_density(norm(x[0] - x[1]), config) / 1.1;
    CaseResult result;
    result.name = "compressed_pair";
    result.solve = solve(config, x, velocity);
    result.primary_before = result.solve.initial.maximum_density_ratio - 1.0;
    result.primary_after = result.solve.final.maximum_density_ratio - 1.0;
    result.observable_change = norm(result.solve.position[0] - result.solve.position[1])
        - norm(x[0] - x[1]);
    result.center_of_mass_error = norm(average(result.solve.position) - average(x));
    result.passed = descent_passed(result.solve)
        && result.primary_after < result.primary_before
        && result.observable_change > OBSERVABLE_FLOOR
        && result.center_of_mass_error <= CONSERVATION_LIMIT;
    return result;
}

CaseResult viscosity_case(bool shear) {
    Config config;
    config.lambda = shear ? 0.0 : 100.0;
    config.mu = shear ? 100.0 : 0.0;
    const std::vector<Vec3> x = {{-0.04, 0.0, 0.0}, {0.04, 0.0, 0.0}};
    const std::vector<Vec3> velocity = shear
        ? std::vector<Vec3>{{0.0, 1.0, 0.0}, {0.0, -1.0, 0.0}}
        : std::vector<Vec3>{{1.0, 0.0, 0.0}, {-1.0, 0.0, 0.0}};
    const Vec3 normal = (x[0] - x[1]) / norm(x[0] - x[1]);
    const Vec3 initial_relative = velocity[0] - velocity[1];
    CaseResult result;
    result.name = shear ? "shear_viscosity_pair" : "normal_viscosity_pair";
    result.solve = solve(config, x, velocity);
    const Vec3 final_relative = result.solve.velocity[0] - result.solve.velocity[1];
    result.primary_before = shear
        ? norm(project_tangent(initial_relative, normal))
        : norm(project_normal(initial_relative, normal));
    result.primary_after = shear
        ? norm(project_tangent(final_relative, normal))
        : norm(project_normal(final_relative, normal));
    result.observable_change = result.primary_before - result.primary_after;
    result.center_of_mass_error = vector_relative_error(
        average(result.solve.velocity), average(velocity));
    result.passed = descent_passed(result.solve)
        && result.observable_change > OBSERVABLE_FLOOR
        && result.center_of_mass_error <= CONSERVATION_LIMIT;
    return result;
}

CaseResult surface_case(bool attractive) {
    Config config;
    config.gamma = 1000.0;
    const double initial_distance = (attractive ? 1.7 : 0.8) * config.spacing;
    const std::vector<Vec3> x = {
        {-0.5 * initial_distance, 0.0, 0.0},
        {0.5 * initial_distance, 0.0, 0.0},
    };
    const std::vector<Vec3> velocity(2);
    CaseResult result;
    result.name = attractive
        ? "surface_attractive_pair" : "surface_repulsive_pair";
    result.solve = solve(config, x, velocity);
    const double final_distance =
        norm(result.solve.position[0] - result.solve.position[1]);
    result.primary_before = initial_distance;
    result.primary_after = final_distance;
    result.observable_change = attractive
        ? initial_distance - final_distance
        : final_distance - initial_distance;
    result.center_of_mass_error = norm(average(result.solve.position) - average(x));
    result.passed = descent_passed(result.solve)
        && result.observable_change > OBSERVABLE_FLOOR
        && result.center_of_mass_error <= CONSERVATION_LIMIT;
    return result;
}

struct CombinedFixture {
    Config config;
    std::vector<Vec3> x;
    std::vector<Vec3> velocity;
};

CombinedFixture combined_fixture() {
    CombinedFixture fixture;
    fixture.config.kappa = 200.0;
    fixture.config.lambda = 20.0;
    fixture.config.mu = 10.0;
    fixture.config.gamma = 100.0;
    fixture.x = {
        {0.0, 0.0, 0.0},
        {0.04, 0.0, 0.0},
        {0.02, 0.0346410161513775, 0.0},
        {0.02, 0.0115470053837925, 0.0326598632371090},
    };
    fixture.velocity = {
        {0.4, -0.2, 0.1},
        {-0.1, 0.3, -0.2},
        {0.2, 0.1, 0.3},
        {-0.3, -0.2, -0.2},
    };
    double density = fixture.config.mass
        * cubic_weight(0.0, fixture.config.horizon);
    for (std::size_t j = 1; j < fixture.x.size(); ++j) {
        density += fixture.config.mass
            * cubic_weight(norm(fixture.x[0] - fixture.x[j]),
                fixture.config.horizon);
    }
    fixture.config.rest_density = density / 1.1;
    return fixture;
}

CaseResult combined_case() {
    const CombinedFixture fixture = combined_fixture();
    CaseResult result;
    result.name = "combined_tetrahedron";
    result.solve = solve(fixture.config, fixture.x, fixture.velocity);
    result.primary_before = result.solve.initial.gradient_norm;
    result.primary_after = result.solve.final.gradient_norm;
    result.observable_change = result.primary_before - result.primary_after;
    result.center_of_mass_error = vector_relative_error(
        average(result.solve.velocity), average(fixture.velocity));
    result.passed = descent_passed(result.solve)
        && result.observable_change > OBSERVABLE_FLOOR
        && result.center_of_mass_error <= CONSERVATION_LIMIT;
    return result;
}

double combined_directional_error() {
    const CombinedFixture fixture = combined_fixture();
    const std::vector<Vec3> y_star =
        predict(fixture.config, fixture.x, fixture.velocity);
    const Evaluation base = evaluate(fixture.config, fixture.x, y_star, y_star);
    std::vector<Vec3> direction = {
        {0.31, -0.27, 0.11},
        {-0.19, 0.41, -0.23},
        {0.17, 0.07, -0.37},
        {-0.29, -0.21, 0.49},
    };
    const double direction_norm = vector_norm(direction);
    for (Vec3& value : direction) {
        value = value / direction_norm;
    }
    const double epsilon = fixture.config.spacing * 1.0e-7;
    std::vector<Vec3> plus = y_star;
    std::vector<Vec3> minus = y_star;
    for (std::size_t i = 0; i < direction.size(); ++i) {
        plus[i] += epsilon * direction[i];
        minus[i] += -epsilon * direction[i];
    }
    const double finite_difference =
        (evaluate(fixture.config, fixture.x, y_star, plus).total
            - evaluate(fixture.config, fixture.x, y_star, minus).total)
        / (2.0 * epsilon);
    const double analytic = vector_dot(base.gradient, direction);
    return relative_error(finite_difference, analytic);
}

void append_case(std::ostringstream& output, const CaseResult& value) {
    output << "{\"name\":\"" << value.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL")
           << "\",\"initial_objective\":" << value.solve.initial.total
           << ",\"final_objective\":" << value.solve.final.total
           << ",\"initial_gradient_norm\":" << value.solve.initial.gradient_norm
           << ",\"final_gradient_norm\":" << value.solve.final.gradient_norm
           << ",\"iterations\":" << value.solve.iterations
           << ",\"backtracks\":" << value.solve.backtracks
           << ",\"minimum_alpha\":" << value.solve.minimum_alpha
           << ",\"primary_before\":" << value.primary_before
           << ",\"primary_after\":" << value.primary_after
           << ",\"observable_change\":" << value.observable_change
           << ",\"center_of_mass_error\":" << value.center_of_mass_error
           << ",\"internal_momentum_residual\":"
           << value.solve.final.internal_momentum_residual << '}';
}

struct ConditioningCase {
    std::string name;
    SolveResult baseline;
    SolveResult block;
    bool direction_preserved = false;
    bool passed = false;
};

ConditioningCase compression_conditioning_case() {
    Config config;
    config.kappa = 500.0;
    const std::vector<Vec3> x = {{-0.025, 0.0, 0.0}, {0.025, 0.0, 0.0}};
    const std::vector<Vec3> velocity(2);
    config.rest_density = pair_density(norm(x[0] - x[1]), config) / 1.1;
    ConditioningCase result;
    result.name = "compressed_pair";
    result.baseline = solve(config, x, velocity, Preconditioner::Inertial);
    result.block = solve(config, x, velocity, Preconditioner::BlockGaussNewton);
    result.direction_preserved =
        norm(result.block.position[0] - result.block.position[1])
        > norm(x[0] - x[1]) + OBSERVABLE_FLOOR;
    return result;
}

ConditioningCase surface_conditioning_case() {
    Config config;
    config.gamma = 1000.0;
    const double initial_distance = 0.8 * config.spacing;
    const std::vector<Vec3> x = {
        {-0.5 * initial_distance, 0.0, 0.0},
        {0.5 * initial_distance, 0.0, 0.0},
    };
    const std::vector<Vec3> velocity(2);
    ConditioningCase result;
    result.name = "surface_repulsive_pair";
    result.baseline = solve(config, x, velocity, Preconditioner::Inertial);
    result.block = solve(config, x, velocity, Preconditioner::BlockGaussNewton);
    result.direction_preserved =
        norm(result.block.position[0] - result.block.position[1])
        > initial_distance + OBSERVABLE_FLOOR;
    return result;
}

ConditioningCase combined_conditioning_case() {
    const CombinedFixture fixture = combined_fixture();
    ConditioningCase result;
    result.name = "combined_tetrahedron";
    result.baseline = solve(
        fixture.config, fixture.x, fixture.velocity, Preconditioner::Inertial);
    result.block = solve(
        fixture.config, fixture.x, fixture.velocity, Preconditioner::BlockGaussNewton);
    result.direction_preserved = result.block.final.total < result.block.initial.total;
    return result;
}

bool conditioning_quality_passed(const ConditioningCase& value) {
    const double objective_allowance = 1.0e-10
        * std::max({std::abs(value.baseline.final.total),
            std::abs(value.block.final.total), 1.0});
    const double gradient_limit =
        std::max(2.0 * value.baseline.final.gradient_norm, 1.0e-8);
    return value.baseline.succeeded && value.block.succeeded
        && value.baseline.monotonic && value.block.monotonic
        && value.block.final.total <= value.baseline.final.total + objective_allowance
        && value.block.final.gradient_norm <= gradient_limit
        && value.block.final.internal_momentum_residual <= CONSERVATION_LIMIT
        && value.direction_preserved;
}

void append_conditioning_case(
    std::ostringstream& output,
    const ConditioningCase& value) {
    const double alpha_ratio = value.block.minimum_alpha
        / std::max(value.baseline.minimum_alpha, 1.0e-300);
    output << "{\"name\":\"" << value.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL")
           << "\",\"direction_preserved\":"
           << (value.direction_preserved ? "true" : "false")
           << ",\"baseline\":{\"final_objective\":"
           << value.baseline.final.total
           << ",\"final_gradient_norm\":" << value.baseline.final.gradient_norm
           << ",\"iterations\":" << value.baseline.iterations
           << ",\"backtracks\":" << value.baseline.backtracks
           << ",\"minimum_alpha\":" << value.baseline.minimum_alpha << '}'
           << ",\"block\":{\"final_objective\":" << value.block.final.total
           << ",\"final_gradient_norm\":" << value.block.final.gradient_norm
           << ",\"iterations\":" << value.block.iterations
           << ",\"backtracks\":" << value.block.backtracks
           << ",\"minimum_alpha\":" << value.block.minimum_alpha << '}'
           << ",\"minimum_alpha_improvement\":" << alpha_ratio << '}';
}

} // namespace

ReferenceSolverReport run_reference_solver_controls() {
    const std::array<CaseResult, 7> cases = {
        free_fall_case(),
        compression_case(),
        viscosity_case(false),
        viscosity_case(true),
        surface_case(false),
        surface_case(true),
        combined_case(),
    };
    const double derivative_error = combined_directional_error();
    bool cases_passed = true;
    std::string first_failure;
    if (derivative_error > DERIVATIVE_LIMIT) {
        first_failure = "FCR2_FULL_OBJECTIVE_DERIVATIVE_MISMATCH";
    }
    for (const CaseResult& value : cases) {
        cases_passed = cases_passed && value.passed;
        if (first_failure.empty() && !value.passed) {
            first_failure = "FCR2_REFERENCE_CASE_FAILED:" + value.name;
        }
    }
    const bool passed = derivative_error <= DERIVATIVE_LIMIT && cases_passed;
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.formula_reclosure_fcr2.v1\""
           << ",\"identity\":\"nuv-variational-fcr1\""
           << ",\"solver\":\"binary64-preconditioned-gradient-armijo-v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"thresholds\":{\"derivative_relative\":" << DERIVATIVE_LIMIT
           << ",\"conservation\":" << CONSERVATION_LIMIT
           << ",\"energy_allowance\":" << ENERGY_ALLOWANCE
           << ",\"observable_floor\":" << OBSERVABLE_FLOOR << '}'
           << ",\"combined_directional_derivative_error\":" << derivative_error
           << ",\"cases\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_case(report, cases[i]);
    }
    report << "]"
           << ",\"fcr3_authorized\":" << (passed ? "true" : "false")
           << ",\"runtime_authority\":false";
    std::string result_material = std::string(passed ? "PASS|" : "FAIL|")
        + first_failure + '|' + std::to_string(derivative_error);
    for (const CaseResult& value : cases) {
        result_material += '|' + value.name + ':'
            + std::to_string(value.solve.final.total) + ':'
            + std::to_string(value.observable_change);
    }
    report << ",\"result_sha256\":\"" << sha256_hex(result_material) << "\"}";
    return {passed, report.str()};
}

ReferenceSolverReport run_conditioning_controls() {
    std::array<ConditioningCase, 3> cases = {
        compression_conditioning_case(),
        surface_conditioning_case(),
        combined_conditioning_case(),
    };
    int baseline_backtracks = 0;
    int block_backtracks = 0;
    for (ConditioningCase& value : cases) {
        value.passed = conditioning_quality_passed(value);
        baseline_backtracks += value.baseline.backtracks;
        block_backtracks += value.block.backtracks;
    }
    const double compression_alpha_ratio = cases[0].block.minimum_alpha
        / std::max(cases[0].baseline.minimum_alpha, 1.0e-300);
    const double combined_alpha_ratio = cases[2].block.minimum_alpha
        / std::max(cases[2].baseline.minimum_alpha, 1.0e-300);
    const bool aggregate_backtracks_passed =
        4LL * static_cast<long long>(block_backtracks)
        <= static_cast<long long>(baseline_backtracks);
    const bool alpha_passed = compression_alpha_ratio >= 16.0
        && combined_alpha_ratio >= 16.0;
    std::string first_failure;
    for (const ConditioningCase& value : cases) {
        if (!value.passed && first_failure.empty()) {
            first_failure = "FCR3A_BLOCK_QUALITY_FAILED:" + value.name;
        }
    }
    if (first_failure.empty() && !aggregate_backtracks_passed) {
        first_failure = "FCR3A_BACKTRACK_REDUCTION_FAILED";
    }
    if (first_failure.empty() && !alpha_passed) {
        first_failure = "FCR3A_ACCEPTED_ALPHA_FAILED";
    }
    const bool passed = first_failure.empty();

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.formula_reclosure_fcr3a.v1\""
           << ",\"identity\":\"nuv-variational-fcr1\""
           << ",\"candidate\":\"block-jacobi-gn-armijo-v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"thresholds\":{\"objective_relative\":1e-10"
           << ",\"gradient_ratio\":2,\"backtrack_reduction\":4"
           << ",\"minimum_alpha_improvement\":16}"
           << ",\"aggregate\":{\"baseline_backtracks\":"
           << baseline_backtracks
           << ",\"block_backtracks\":" << block_backtracks
           << ",\"backtrack_gate\":\""
           << (aggregate_backtracks_passed ? "PASS" : "FAIL")
           << "\",\"compression_alpha_improvement\":"
           << compression_alpha_ratio
           << ",\"combined_alpha_improvement\":" << combined_alpha_ratio
           << ",\"alpha_gate\":\"" << (alpha_passed ? "PASS" : "FAIL")
           << "\"},\"cases\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_conditioning_case(report, cases[i]);
    }
    report << "]"
           << ",\"fcr3b_authorized\":" << (passed ? "true" : "false")
           << ",\"runtime_authority\":false";
    std::string result_material = std::string(passed ? "PASS|" : "FAIL|")
        + first_failure + '|' + std::to_string(baseline_backtracks) + '|'
        + std::to_string(block_backtracks) + '|'
        + std::to_string(compression_alpha_ratio) + '|'
        + std::to_string(combined_alpha_ratio);
    report << ",\"result_sha256\":\"" << sha256_hex(result_material) << "\"}";
    return {passed, report.str()};
}

} // namespace nextengine::nonlocal::fcr
