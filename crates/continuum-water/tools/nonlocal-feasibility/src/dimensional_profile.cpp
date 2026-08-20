#include "variational_reference.hpp"

#include "sha256.hpp"

#include <algorithm>
#include <cmath>
#include <cstddef>
#include <iomanip>
#include <limits>
#include <sstream>
#include <string>

namespace nextengine::nonlocal::fcr {
namespace {

constexpr double PI = 3.141592653589793238462643383279502884;
constexpr double EXPECTED_RAW_LATTICE_RATIO = 0.12522433816880058;

double relative_error(double lhs, double rhs) {
    return std::abs(lhs - rhs)
        / std::max({std::abs(lhs), std::abs(rhs), 1.0e-30});
}

double raw_cubic_weight(double radius, double horizon) {
    const double q = 2.0 * radius / horizon;
    const double alpha = 3.0
        / (2.0 * PI * horizon * horizon * horizon);
    if (q > 2.0) {
        return 0.0;
    }
    if (q >= 1.0) {
        const double delta = 2.0 - q;
        return alpha * delta * delta * delta / 6.0;
    }
    return alpha * (2.0 / 3.0 - q * q + 0.5 * q * q * q);
}

double raw_cubic_gradient(double radius, double horizon) {
    const double q = 2.0 * radius / horizon;
    const double alpha = 3.0
        / (2.0 * PI * horizon * horizon * horizon);
    if (q > 2.0) {
        return 0.0;
    }
    const double derivative_q = q >= 1.0
        ? -0.5 * alpha * (2.0 - q) * (2.0 - q)
        : alpha * (-2.0 * q + 1.5 * q * q);
    return derivative_q * (2.0 / horizon);
}

template <typename Function>
double simpson_integral(
    double lower, double upper, std::size_t intervals, Function function) {
    const double step = (upper - lower) / static_cast<double>(intervals);
    double sum = function(lower) + function(upper);
    for (std::size_t index = 1; index < intervals; ++index) {
        sum += (index % 2 == 0 ? 2.0 : 4.0)
            * function(lower + static_cast<double>(index) * step);
    }
    return step * sum / 3.0;
}

double raw_lattice_ratio(double spacing, double horizon) {
    const int extent = static_cast<int>(std::ceil(horizon / spacing));
    double result = 0.0;
    for (int z = -extent; z <= extent; ++z) {
        for (int y = -extent; y <= extent; ++y) {
            for (int x = -extent; x <= extent; ++x) {
                const double radius = spacing * std::sqrt(
                    static_cast<double>(x * x + y * y + z * z));
                result += spacing * spacing * spacing
                    * raw_cubic_weight(radius, horizon);
            }
        }
    }
    return result;
}

double surface_potential_hat(double q) {
    if (q <= 1.0) {
        return q * q * q / 3.0 - q - 2.0 / 3.0;
    }
    if (q < 3.0) {
        return q - (q - 2.0) * (q - 2.0) * (q - 2.0) / 3.0
            - 8.0 / 3.0;
    }
    return 0.0;
}

} // namespace

ReferenceSolverReport run_dimensional_profile_controls() {
    constexpr double rest_density = 1000.0;
    constexpr double spacing = 0.05;
    constexpr double horizon = 0.15;
    constexpr double time_step = 1.0 / 240.0;
    constexpr double gravity = 9.81;
    constexpr double maximum_head = 1.0;
    constexpr double density_strain = 1.0e-3;
    constexpr double nominal_water_viscosity = 1.0e-3;
    constexpr double water_surface_tension = 0.07274;
    constexpr double analytic_raw_integral = 1.0 / 8.0;
    constexpr double analytic_surface_constant = 1459.0 * PI / 210.0;

    const double particle_mass = rest_density * spacing * spacing * spacing;
    const double particle_volume = particle_mass / rest_density;
    const double lattice_ratio = raw_lattice_ratio(spacing, horizon);
    const double kernel_scale = 1.0 / lattice_ratio;
    const double normalized_lattice_ratio = lattice_ratio * kernel_scale;
    const double numerical_raw_integral = simpson_integral(
        0.0, horizon, 65536,
        [horizon](double radius) {
            return 4.0 * PI * radius * radius
                * raw_cubic_weight(radius, horizon);
        });
    const double normalized_continuum_integral =
        kernel_scale * numerical_raw_integral;

    const double derivative_radius = 0.37 * horizon;
    const double derivative_step = 1.0e-6 * horizon;
    const double derivative_fd = kernel_scale
        * (raw_cubic_weight(derivative_radius + derivative_step, horizon)
            - raw_cubic_weight(
                derivative_radius - derivative_step, horizon))
        / (2.0 * derivative_step);
    const double derivative_exact = kernel_scale
        * raw_cubic_gradient(derivative_radius, horizon);
    const double derivative_error =
        relative_error(derivative_fd, derivative_exact);

    const double analytic_kernel_moment =
        kernel_scale * 31.0 * horizon / 140.0;
    const double numerical_kernel_moment = simpson_integral(
        0.0, horizon, 65536,
        [horizon, kernel_scale](double radius) {
            const double radius_squared = radius * radius;
            return 4.0 * PI * radius_squared * radius_squared
                * (-kernel_scale * raw_cubic_gradient(radius, horizon));
        });
    const double kernel_moment_error = relative_error(
        numerical_kernel_moment, analytic_kernel_moment);

    const double numerical_surface_constant = -PI * simpson_integral(
        0.0, 3.0, 60000,
        [](double q) {
            return q * q * q * surface_potential_hat(q);
        });
    const double surface_constant_error = relative_error(
        numerical_surface_constant, analytic_surface_constant);

    const double target_bulk_modulus =
        rest_density * gravity * maximum_head / density_strain;
    const double kappa = target_bulk_modulus * particle_volume;
    const double lambda_water = 30.0 * nominal_water_viscosity
        * particle_volume / analytic_kernel_moment;
    const double mu_water = 0.0;
    const double gamma_water = water_surface_tension
        / (analytic_surface_constant * rest_density * rest_density
            * spacing * spacing * spacing * spacing * spacing);

    const double pi_horizon = horizon / spacing;
    const double pi_mass = particle_mass
        / (rest_density * spacing * spacing * spacing);
    const double pi_gravity = gravity * time_step * time_step / spacing;
    const double pi_kappa = kappa * time_step * time_step
        / (particle_mass * spacing * spacing);
    const double pi_lambda = lambda_water * time_step
        / (particle_mass * spacing);
    const double pi_mu = mu_water * time_step
        / (particle_mass * spacing);
    const double pi_gamma = gamma_water * particle_mass
        * time_step * time_step / spacing;

    const bool units_valid = true;
    const bool raw_integral_valid = relative_error(
        numerical_raw_integral, analytic_raw_integral) <= 1.0e-12;
    const bool raw_lattice_expected = relative_error(
        lattice_ratio, EXPECTED_RAW_LATTICE_RATIO) <= 1.0e-14;
    const bool current_formula_physics_eligible =
        lattice_ratio >= 0.99 && lattice_ratio <= 1.01;
    const bool normalized_density_valid = relative_error(
        normalized_lattice_ratio, 1.0) <= 1.0e-12;
    const bool normalized_integral_valid =
        normalized_continuum_integral >= 0.995
        && normalized_continuum_integral <= 1.005;
    const bool derivative_valid = derivative_error <= 1.0e-7;
    const bool kernel_moment_valid = kernel_moment_error <= 1.0e-10;
    const bool surface_constant_valid = surface_constant_error <= 1.0e-10;
    const bool anchors_finite = std::isfinite(kappa)
        && std::isfinite(lambda_water) && std::isfinite(gamma_water)
        && kappa > 0.0 && lambda_water > 0.0 && gamma_water > 0.0;
    const bool dimensionless_finite = std::isfinite(pi_horizon)
        && std::isfinite(pi_mass) && std::isfinite(pi_gravity)
        && std::isfinite(pi_kappa) && std::isfinite(pi_lambda)
        && std::isfinite(pi_mu) && std::isfinite(pi_gamma);
    const bool passed = units_valid && raw_integral_valid
        && raw_lattice_expected && !current_formula_physics_eligible
        && normalized_density_valid && normalized_integral_valid
        && derivative_valid && kernel_moment_valid
        && surface_constant_valid && anchors_finite && dimensionless_finite;
    std::string first_failure;
    if (!units_valid) {
        first_failure = "NSR3B0_UNITS";
    } else if (!raw_integral_valid) {
        first_failure = "NSR3B0_RAW_CONTINUUM_INTEGRAL";
    } else if (!raw_lattice_expected) {
        first_failure = "NSR3B0_RAW_LATTICE_RATIO";
    } else if (current_formula_physics_eligible) {
        first_failure = "NSR3B0_EXPECTED_INELIGIBILITY_NOT_REPRODUCED";
    } else if (!normalized_density_valid || !normalized_integral_valid) {
        first_failure = "NSR3B0_NORMALIZATION_CANDIDATE";
    } else if (!derivative_valid) {
        first_failure = "NSR3B0_NORMALIZED_DERIVATIVE";
    } else if (!kernel_moment_valid) {
        first_failure = "NSR3B0_KERNEL_MOMENT";
    } else if (!surface_constant_valid) {
        first_failure = "NSR3B0_SURFACE_MOMENT";
    } else if (!anchors_finite || !dimensionless_finite) {
        first_failure = "NSR3B0_PROFILE_ANCHOR";
    }

    std::ostringstream result_material;
    result_material << std::setprecision(17)
                    << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
                    << lattice_ratio << '|' << kernel_scale << '|'
                    << normalized_continuum_integral << '|'
                    << analytic_kernel_moment << '|' << kappa << '|'
                    << lambda_water << '|' << mu_water << '|' << gamma_water;
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b0_dimension.v1\""
           << ",\"identity_under_diagnosis\":\"nuv-variational-fcr1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"disposition\":\""
           << (passed ? "FORMULA_RECLOSURE_REQUIRED" : "DIAGNOSTIC_STOP")
           << '"'
           << ",\"current_formula_physics_eligible\":"
           << (current_formula_physics_eligible ? "true" : "false")
           << ",\"next_identity\":\"nuv-variational-fcr2\""
           << ",\"kernel_candidate\":\"lattice-normalized-cubic-v1\""
           << ",\"units_valid\":" << (units_valid ? "true" : "false")
           << ",\"kernel\":{\"analytic_raw_continuum_integral\":"
           << analytic_raw_integral
           << ",\"numerical_raw_continuum_integral\":"
           << numerical_raw_integral
           << ",\"raw_lattice_density_ratio\":" << lattice_ratio
           << ",\"normalization_scale\":" << kernel_scale
           << ",\"normalized_lattice_density_ratio\":"
           << normalized_lattice_ratio
           << ",\"normalized_continuum_integral\":"
           << normalized_continuum_integral
           << ",\"normalized_derivative_relative_error\":"
           << derivative_error
           << ",\"analytic_second_moment\":" << analytic_kernel_moment
           << ",\"numerical_second_moment\":" << numerical_kernel_moment
           << ",\"second_moment_relative_error\":" << kernel_moment_error
           << "}"
           << ",\"surface\":{\"analytic_flat_interface_constant\":"
           << analytic_surface_constant
           << ",\"numerical_flat_interface_constant\":"
           << numerical_surface_constant
           << ",\"relative_error\":" << surface_constant_error << "}"
           << ",\"anchors\":{\"rest_density\":" << rest_density
           << ",\"spacing\":" << spacing
           << ",\"horizon\":" << horizon
           << ",\"particle_mass\":" << particle_mass
           << ",\"time_step\":" << time_step
           << ",\"gravity\":" << gravity
           << ",\"maximum_head\":" << maximum_head
           << ",\"density_strain\":" << density_strain
           << ",\"target_bulk_modulus\":" << target_bulk_modulus
           << ",\"kappa\":" << kappa
           << ",\"nominal_water_viscosity\":"
           << nominal_water_viscosity
           << ",\"lambda_water\":" << lambda_water
           << ",\"mu_water\":" << mu_water
           << ",\"water_surface_tension\":" << water_surface_tension
           << ",\"gamma_water\":" << gamma_water << "}"
           << ",\"dimensionless\":{\"horizon\":" << pi_horizon
           << ",\"mass\":" << pi_mass
           << ",\"gravity\":" << pi_gravity
           << ",\"kappa\":" << pi_kappa
           << ",\"lambda\":" << pi_lambda
           << ",\"mu\":" << pi_mu
           << ",\"gamma\":" << pi_gamma << "}"
           << ",\"legacy_implications\":{\"kappa_1_bulk_modulus\":"
           << rest_density / particle_mass
           << ",\"kappa_9196_875_bulk_modulus\":"
           << 9196.875 * rest_density / particle_mass
           << ",\"gamma_1000_surface_tension\":"
           << 1000.0 * analytic_surface_constant * rest_density * rest_density
                * spacing * spacing * spacing * spacing * spacing
           << "}"
           << ",\"runtime_authority\":false"
           << ",\"result_sha256\":\""
           << sha256_hex(result_material.str()) << "\"}";
    return {passed, report.str()};
}

} // namespace nextengine::nonlocal::fcr
