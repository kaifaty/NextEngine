#include "oracle.hpp"

#include "sha256.hpp"

#include <algorithm>
#include <cmath>
#include <iomanip>
#include <limits>
#include <sstream>
#include <stdexcept>

namespace nextengine::nonlocal {
namespace {

constexpr double PI = 3.141592653589793238462643383279502884;
constexpr double PAIR_EPSILON = 1.0e-12;

using NeighborLists = std::vector<std::vector<std::size_t>>;

double cubic_weight(double radius, double horizon, double scale) {
    // PeriDyno Kernel.h CubicKernel::weight at the pinned NR0 reference commit.
    const double q = 2.0 * radius / horizon;
    const double alpha = 3.0 * scale / (2.0 * PI * horizon * horizon * horizon);
    if (q > 2.0) {
        return 0.0;
    }
    if (q >= 1.0) {
        const double delta = 2.0 - q;
        return alpha * delta * delta * delta / 6.0;
    }
    return alpha * (2.0 / 3.0 - q * q + 0.5 * q * q * q);
}

double cubic_gradient(double radius, double horizon, double scale) {
    // PeriDyno Kernel.h CubicKernel::gradient. The reference intentionally
    // omits the analytical 2/h chain factor, so this oracle does too.
    const double q = 2.0 * radius / horizon;
    const double alpha = 3.0 * scale / (2.0 * PI * horizon * horizon * horizon);
    if (q > 2.0) {
        return 0.0;
    }
    if (q >= 1.0) {
        const double delta = 2.0 - q;
        return -0.5 * alpha * delta * delta;
    }
    return alpha * (-2.0 * q + 1.5 * q * q);
}

double cubic_scale(double spacing, double horizon) {
    // PeriDyno ParticleApproximation::calculateScalingFactor.
    const int half_resolution = static_cast<int>(horizon / spacing + 1.0);
    const double particle_volume = spacing * spacing * spacing;
    double total = 0.0;
    for (int z = -half_resolution; z <= half_resolution; ++z) {
        for (int y = -half_resolution; y <= half_resolution; ++y) {
            for (int x = -half_resolution; x <= half_resolution; ++x) {
                const Vec3 offset{
                    x * spacing,
                    y * spacing,
                    z * spacing,
                };
                total += particle_volume * cubic_weight(norm(offset), horizon, 1.0);
            }
        }
    }
    if (!std::isfinite(total) || total <= 0.0) {
        throw std::runtime_error("invalid cubic-kernel scaling factor");
    }
    return 1.0 / total;
}

NeighborLists build_neighbors(const std::vector<Vec3>& positions, double horizon) {
    NeighborLists neighbors(positions.size());
    // NEXTENGINE_RESEARCH_CHOICE: the relative comparison allowance is used
    // only to make exact support-edge membership stable across host compilers.
    const double support_limit = horizon * (1.0 + 1.0e-12);
    for (std::size_t i = 0; i < positions.size(); ++i) {
        for (std::size_t j = 0; j < positions.size(); ++j) {
            if (norm(positions[i] - positions[j]) <= support_limit) {
                neighbors[i].push_back(j);
            }
        }
    }
    return neighbors;
}

bool symmetric_neighbors(const NeighborLists& neighbors) {
    for (std::size_t i = 0; i < neighbors.size(); ++i) {
        for (std::size_t j : neighbors[i]) {
            if (j >= neighbors.size()
                || std::find(neighbors[j].begin(), neighbors[j].end(), i)
                    == neighbors[j].end()) {
                return false;
            }
        }
    }
    return true;
}

std::vector<double> summation_density(
    const std::vector<Vec3>& positions,
    const NeighborLists& neighbors,
    double mass,
    double horizon,
    double scale) {
    std::vector<double> density(positions.size(), 0.0);
    for (std::size_t i = 0; i < positions.size(); ++i) {
        for (std::size_t j : neighbors[i]) {
            // Paper Section 4.1 summation density, interpreted through the
            // pinned PeriDyno SummationDensity cubic-kernel path.
            density[i] += mass * cubic_weight(norm(positions[i] - positions[j]), horizon, scale);
        }
    }
    return density;
}

void accumulate_incompressibility(
    const Fixture& fixture,
    const NeighborLists& neighbors,
    const std::vector<Vec3>& current,
    const std::vector<double>& density,
    double scale,
    std::vector<Vec3>& source,
    std::vector<Mat3>& matrix) {
    // Paper Eq. (7), linearized into the pairwise SISSM update of Eq. (26),
    // with coefficients interpreted through SIUFS_CalculateSourceForIncompressibility.
    const double coefficient =
        fixture.kappa * fixture.time_step * fixture.time_step / fixture.rest_density;
    for (std::size_t i = 0; i < current.size(); ++i) {
        const double density_ratio = std::max(density[i], fixture.rest_density)
            / fixture.rest_density;
        for (std::size_t j : neighbors[i]) {
            const double radius = norm(current[i] - current[j]);
            if (radius <= PAIR_EPSILON) {
                continue;
            }
            const double a = coefficient * cubic_gradient(radius, fixture.horizon, scale) / radius;
            const double diagonal = -a;
            const Vec3 local = (-a) * current[j]
                + (density_ratio * a) * (current[j] - current[i]);
            const Vec3 reverse = (-a) * current[i]
                + (density_ratio * a) * (current[i] - current[j]);
            source[i] += local;
            source[j] += reverse;
            for (int axis = 0; axis < 3; ++axis) {
                matrix[i](axis, axis) += diagonal;
                matrix[j](axis, axis) += diagonal;
            }
        }
    }
}

void accumulate_viscosity(
    const Fixture& fixture,
    const NeighborLists& neighbors,
    const std::vector<Vec3>& reference,
    const std::vector<Vec3>& current,
    double scale,
    std::vector<Vec3>& source,
    std::vector<Mat3>& matrix) {
    // Paper Eq. (10), interpreted through SIUFS_CalculateSourceForViscosity.
    const double alpha = fixture.terms.bulk_viscosity
        ? fixture.lambda * fixture.time_step / fixture.rest_density
        : 0.0;
    const double beta = fixture.terms.shear_viscosity
        ? fixture.mu * fixture.time_step / fixture.rest_density
        : 0.0;
    for (std::size_t i = 0; i < current.size(); ++i) {
        for (std::size_t j : neighbors[i]) {
            const Vec3 reference_delta = reference[j] - reference[i];
            const double radius = norm(reference_delta);
            if (radius <= PAIR_EPSILON) {
                continue;
            }
            const Vec3 direction = reference_delta / radius;
            const Mat3 normal = outer(direction, direction);
            const Mat3 tangent = Mat3::identity() - normal;
            const double weight = cubic_weight(radius, fixture.horizon, scale);
            const Mat3 pair_matrix = (alpha * weight) * normal + (beta * weight) * tangent;
            const Vec3 local = pair_matrix * current[j] - pair_matrix * reference_delta;
            const Vec3 reverse = pair_matrix * current[i] + pair_matrix * reference_delta;
            source[i] += local;
            source[j] += reverse;
            matrix[i] += pair_matrix;
            matrix[j] += pair_matrix;
        }
    }
}

double surface_positive(double radius, double rest_spacing) {
    const double q = radius / rest_spacing;
    if (q <= 1.0) {
        return q * q;
    }
    if (q <= 3.0) {
        return 1.0 - (q - 2.0) * (q - 2.0);
    }
    return 0.0;
}

double surface_negative(double radius, double rest_spacing) {
    return radius / rest_spacing <= 1.0 ? -1.0 : 0.0;
}

double surface_potential(double radius, double rest_spacing) {
    // PeriDyno K_SIUFS_EnergyForSimpleBidirectionFuncNegative, the potential
    // associated with its default bidirectional surface pair function.
    const double q = radius / rest_spacing;
    if (q <= 1.0) {
        return q * q * q / 3.0 - q + 2.0 / 3.0;
    }
    if (q <= 3.0) {
        return q - (q - 2.0) * (q - 2.0) * (q - 2.0) / 3.0 - 4.0 / 3.0;
    }
    return 0.0;
}

void accumulate_surface_tension(
    const Fixture& fixture,
    const NeighborLists& neighbors,
    const std::vector<Vec3>& current,
    std::vector<Vec3>& source,
    std::vector<Mat3>& matrix) {
    // Paper Eq. (14), interpreted through the default bidirectional
    // SIUFS_ComputeSourceForSurfaceTension pair function.
    const double coefficient = fixture.gamma * fixture.time_step * fixture.time_step;
    for (std::size_t i = 0; i < current.size(); ++i) {
        for (std::size_t j : neighbors[i]) {
            const double radius = norm(current[i] - current[j]);
            if (radius <= PAIR_EPSILON) {
                continue;
            }
            const double positive = surface_positive(radius, fixture.spacing);
            const double negative = surface_negative(radius, fixture.spacing);
            const double diagonal = coefficient * positive / radius;
            const Vec3 local = diagonal * current[j]
                + (coefficient * negative / radius) * (current[j] - current[i]);
            const Vec3 reverse = diagonal * current[i]
                + (coefficient * negative / radius) * (current[i] - current[j]);
            source[i] += local;
            source[j] += reverse;
            for (int axis = 0; axis < 3; ++axis) {
                matrix[i](axis, axis) += diagonal;
                matrix[j](axis, axis) += diagonal;
            }
        }
    }
}

EnergyComponents calculate_energies(
    const Fixture& fixture,
    const NeighborLists& neighbors,
    const std::vector<Vec3>& reference,
    const std::vector<Vec3>& predicted,
    const std::vector<Vec3>& current,
    const std::vector<double>& density,
    double scale) {
    EnergyComponents result;
    for (std::size_t i = 0; i < current.size(); ++i) {
        // Paper Eq. (4) inertial energy.
        result.inertia += fixture.mass * norm_squared(current[i] - predicted[i])
            / (2.0 * fixture.time_step * fixture.time_step);
        if (fixture.terms.incompressibility) {
            // Paper Eq. (7) incompressibility potential.
            const double ratio_error = density[i] / fixture.rest_density - 1.0;
            result.incompressibility += 0.5 * fixture.kappa * ratio_error * ratio_error;
        }
        for (std::size_t j : neighbors[i]) {
            const Vec3 reference_delta = reference[i] - reference[j];
            const double radius = norm(reference_delta);
            if (radius <= PAIR_EPSILON) {
                continue;
            }
            if (fixture.terms.bulk_viscosity || fixture.terms.shear_viscosity) {
                // Paper Eq. (10) directional bulk/shear decomposition.
                const Vec3 direction = reference_delta / radius;
                const Vec3 relative_velocity =
                    ((current[i] - current[j]) - reference_delta) / fixture.time_step;
                const Vec3 bulk_component = direction * dot(relative_velocity, direction);
                const Vec3 shear_component = relative_velocity - bulk_component;
                const double weight = cubic_weight(radius, fixture.horizon, scale);
                if (fixture.terms.bulk_viscosity) {
                    result.bulk_viscosity += fixture.mass / fixture.rest_density
                        * fixture.lambda * norm_squared(bulk_component) * weight / 4.0;
                }
                if (fixture.terms.shear_viscosity) {
                    result.shear_viscosity += fixture.mass / fixture.rest_density
                        * fixture.mu * norm_squared(shear_component) * weight / 2.0;
                }
            }
            if (fixture.terms.surface_tension) {
                // Paper Eq. (14), using the pinned source's default potential.
                result.surface_tension += fixture.gamma * fixture.mass * fixture.mass
                    * surface_potential(norm(current[i] - current[j]), fixture.spacing);
            }
        }
    }
    return result;
}

Fixture base_fixture(std::string name) {
    Fixture fixture;
    fixture.name = std::move(name);
    return fixture;
}

std::vector<Particle> centered_lattice(
    int x_count,
    int y_count,
    int z_count,
    double spacing,
    bool fixed_shell) {
    std::vector<Particle> particles;
    particles.reserve(static_cast<std::size_t>(x_count * y_count * z_count));
    for (int z = 0; z < z_count; ++z) {
        for (int y = 0; y < y_count; ++y) {
            for (int x = 0; x < x_count; ++x) {
                const bool fixed = fixed_shell
                    && (x == 0 || y == 0 || z == 0 || x == x_count - 1
                        || y == y_count - 1 || z == z_count - 1);
                particles.push_back({
                    {
                        (x - 0.5 * (x_count - 1)) * spacing,
                        (y - 0.5 * (y_count - 1)) * spacing,
                        (z - 0.5 * (z_count - 1)) * spacing,
                    },
                    {},
                    fixed,
                });
            }
        }
    }
    return particles;
}

bool result_is_finite(const OracleResult& result) {
    for (double value : result.density) {
        if (!std::isfinite(value)) {
            return false;
        }
    }
    for (Vec3 value : result.source) {
        if (!finite(value)) {
            return false;
        }
    }
    for (const Mat3& value : result.local_matrix) {
        if (!finite(value)) {
            return false;
        }
    }
    for (Vec3 value : result.next_position) {
        if (!finite(value)) {
            return false;
        }
    }
    for (Vec3 value : result.final_velocity) {
        if (!finite(value)) {
            return false;
        }
    }
    const EnergyComponents& energy = result.energy;
    return std::isfinite(energy.inertia) && std::isfinite(energy.incompressibility)
        && std::isfinite(energy.bulk_viscosity) && std::isfinite(energy.shear_viscosity)
        && std::isfinite(energy.surface_tension)
        && std::isfinite(result.normalized_momentum_residual);
}

void append_vec3(std::ostringstream& output, Vec3 value) {
    output << '[' << value.x << ',' << value.y << ',' << value.z << ']';
}

template <typename Value, typename Append>
void append_array(std::ostringstream& output, const std::vector<Value>& values, Append append) {
    output << '[';
    for (std::size_t index = 0; index < values.size(); ++index) {
        if (index != 0) {
            output << ',';
        }
        append(output, values[index]);
    }
    output << ']';
}

} // namespace

std::vector<Fixture> oracle_fixtures() {
    std::vector<Fixture> fixtures;

    Fixture isolated = base_fixture("isolated_particle");
    isolated.particles = {{{0.0, 0.0, 0.0}, {}, false}};
    isolated.terms.incompressibility = true;
    isolated.kappa = 1.0;
    fixtures.push_back(isolated);

    Fixture pair = base_fixture("symmetric_pair");
    pair.particles = {
        {{-0.0025, 0.0, 0.0}, {}, false},
        {{0.0025, 0.0, 0.0}, {}, false},
    };
    pair.terms.incompressibility = true;
    pair.kappa = 1.0;
    fixtures.push_back(pair);

    Fixture triplet = base_fixture("collinear_triplet");
    triplet.particles = {
        {{-0.005, 0.0, 0.0}, {}, false},
        {{0.0, 0.0, 0.0}, {}, false},
        {{0.005, 0.0, 0.0}, {}, false},
    };
    triplet.terms.incompressibility = true;
    triplet.kappa = 1.0;
    fixtures.push_back(triplet);

    Fixture tetrahedron = base_fixture("tetrahedral_neighborhood");
    const double tetra_height = std::sqrt(2.0 / 3.0) * tetrahedron.spacing;
    tetrahedron.particles = {
        {{0.0, 0.0, 0.0}, {}, false},
        {{tetrahedron.spacing, 0.0, 0.0}, {}, false},
        {{0.5 * tetrahedron.spacing, 0.5 * std::sqrt(3.0) * tetrahedron.spacing, 0.0}, {}, false},
        {{0.5 * tetrahedron.spacing, tetrahedron.spacing / (2.0 * std::sqrt(3.0)), tetra_height}, {}, false},
    };
    tetrahedron.terms.incompressibility = true;
    tetrahedron.kappa = 1.0;
    fixtures.push_back(tetrahedron);

    Fixture block = base_fixture("uniform_interior_block");
    block.horizon = 0.010;
    block.particles = centered_lattice(5, 5, 5, block.spacing, false);
    block.terms.incompressibility = true;
    block.kappa = 1.0;
    fixtures.push_back(block);

    Fixture surface = base_fixture("free_surface_patch");
    surface.particles = centered_lattice(4, 3, 2, surface.spacing, false);
    surface.terms.surface_tension = true;
    surface.gamma = 1000.0;
    fixtures.push_back(surface);

    Fixture shear = base_fixture("viscosity_shear_pair");
    shear.particles = {
        {{-0.0025, 0.0, 0.0}, {0.0, 0.5, 0.0}, false},
        {{0.0025, 0.0, 0.0}, {0.0, -0.5, 0.0}, false},
    };
    shear.terms.shear_viscosity = true;
    shear.mu = 1.0;
    fixtures.push_back(shear);

    for (int iterations = 1; iterations <= 4; ++iterations) {
        Fixture box = base_fixture("tiny_closed_box_iter" + std::to_string(iterations));
        box.particles = centered_lattice(3, 3, 3, box.spacing, true);
        box.gravity = {0.0, -9.81, 0.0};
        box.terms = {true, true, true, false};
        box.kappa = 1.0;
        box.lambda = 1.5;
        box.mu = 0.2;
        box.iterations = iterations;
        fixtures.push_back(std::move(box));
    }

    return fixtures;
}

OracleResult run_cpu_oracle(const Fixture& fixture) {
    if (fixture.particles.empty() || fixture.particles.size() > 256) {
        throw std::invalid_argument("oracle fixture sample count is outside 1..=256");
    }
    if (fixture.iterations < 1 || fixture.iterations > 4) {
        throw std::invalid_argument("oracle fixture iteration count is outside 1..=4");
    }

    std::vector<Vec3> reference;
    std::vector<Vec3> predicted;
    reference.reserve(fixture.particles.size());
    predicted.reserve(fixture.particles.size());
    for (const Particle& particle : fixture.particles) {
        reference.push_back(particle.position);
        // Paper Eq. (4) inertial prediction; matches SIUFS_PredictVelocity.
        predicted.push_back(particle.position
            + (particle.velocity + fixture.gravity * fixture.time_step) * fixture.time_step);
    }

    const NeighborLists neighbors = build_neighbors(reference, fixture.horizon);
    if (!symmetric_neighbors(neighbors)) {
        throw std::runtime_error("fixture neighbor membership is not symmetric");
    }
    const double scale = cubic_scale(fixture.spacing, fixture.horizon);
    std::vector<Vec3> current = predicted;
    std::vector<double> density;
    std::vector<Vec3> source;
    std::vector<Mat3> matrix;
    std::vector<Vec3> linearization_position;

    for (int iteration = 0; iteration < fixture.iterations; ++iteration) {
        linearization_position = current;
        density = summation_density(current, neighbors, fixture.mass, fixture.horizon, scale);
        source.assign(current.size(), {});
        matrix.assign(current.size(), {});
        if (fixture.terms.incompressibility) {
            accumulate_incompressibility(
                fixture, neighbors, current, density, scale, source, matrix);
        }
        if (fixture.terms.bulk_viscosity || fixture.terms.shear_viscosity) {
            accumulate_viscosity(
                fixture, neighbors, reference, current, scale, source, matrix);
        }
        if (fixture.terms.surface_tension) {
            accumulate_surface_tension(fixture, neighbors, current, source, matrix);
        }

        std::vector<Vec3> next(current.size());
        for (std::size_t i = 0; i < current.size(); ++i) {
            if (fixture.particles[i].fixed) {
                // PeriDyno's attribute overload pins fixed samples without
                // introducing a hidden regularization term.
                next[i] = reference[i];
            } else {
                // Paper Eq. (26): local SISSM 3x3 update.
                next[i] = inverse_without_regularization(Mat3::identity() + matrix[i])
                    * (predicted[i] + source[i]);
            }
        }
        current = std::move(next);
    }

    const std::vector<double> final_density =
        summation_density(current, neighbors, fixture.mass, fixture.horizon, scale);
    OracleResult result;
    result.density = final_density;
    result.source = std::move(source);
    result.local_matrix = std::move(matrix);
    result.predicted_position = predicted;
    result.linearization_position = linearization_position;
    result.next_position = current;
    result.energy = calculate_energies(
        fixture, neighbors, reference, predicted, current, final_density, scale);
    result.final_velocity.resize(current.size());
    Vec3 total_pair_impulse{};
    double pair_impulse_scale = 0.0;
    for (std::size_t i = 0; i < current.size(); ++i) {
        // Eq. (26) stores each internal pair force as source - K*y. Summing
        // that quantity, including fixed endpoints, tests pair impulse closure
        // without conflating it with the local inverse or boundary reaction.
        const Vec3 matrix_position = result.local_matrix[i] * linearization_position[i];
        const Vec3 pair_impulse = result.source[i] - matrix_position;
        total_pair_impulse += pair_impulse;
        pair_impulse_scale += norm(result.source[i]) + norm(matrix_position);
        if (fixture.particles[i].fixed) {
            result.final_velocity[i] = {};
            continue;
        }
        result.final_velocity[i] = (current[i] - reference[i]) / fixture.time_step;
    }
    result.normalized_momentum_residual =
        norm(total_pair_impulse) / std::max(pair_impulse_scale, 1.0e-30);
    for (const auto& list : neighbors) {
        result.directed_pairs += list.size();
        result.maximum_degree = std::max(result.maximum_degree, list.size());
    }
    return result;
}

std::vector<OracleCaseReport> run_cpu_self_test() {
    std::vector<OracleCaseReport> reports;
    for (const Fixture& fixture : oracle_fixtures()) {
        OracleCaseReport report;
        report.name = fixture.name;
        try {
            report.result = run_cpu_oracle(fixture);
            if (!result_is_finite(report.result)) {
                report.failures.push_back("nonfinite_output");
            }
            for (std::size_t i = 0; i < fixture.particles.size(); ++i) {
                if (fixture.particles[i].fixed
                    && norm(report.result.next_position[i] - fixture.particles[i].position)
                        > 1.0e-15) {
                    report.failures.push_back("fixed_sample_moved");
                    break;
                }
            }
            if (fixture.name == "isolated_particle") {
                if (norm(report.result.source[0]) > 1.0e-15
                    || norm(report.result.next_position[0] - report.result.predicted_position[0])
                        > 1.0e-15) {
                    report.failures.push_back("isolated_sample_has_pair_correction");
                }
            }
            if (fixture.name == "symmetric_pair"
                && report.result.normalized_momentum_residual > 1.0e-12) {
                report.failures.push_back("symmetric_pair_momentum_closure");
            }
            if (report.result.normalized_momentum_residual
                > find_profile("nuv-tiny-oracle.v0").tolerances.normalized_momentum_residual) {
                report.failures.push_back("pair_momentum_closure");
            }
            if (fixture.name == "uniform_interior_block") {
                const std::size_t center = 2U + 5U * (2U + 5U * 2U);
                const double relative_error =
                    std::abs(report.result.density[center] / fixture.rest_density - 1.0);
                if (relative_error > 1.0e-12) {
                    report.failures.push_back("uniform_interior_density_not_rest");
                }
            }
            if (fixture.name == "free_surface_patch"
                && report.result.energy.surface_tension <= 0.0) {
                report.failures.push_back("surface_potential_not_positive");
            }
            if (fixture.name == "viscosity_shear_pair") {
                if (report.result.energy.shear_viscosity <= 0.0
                    || std::abs(report.result.energy.bulk_viscosity) > 1.0e-30) {
                    report.failures.push_back("viscosity_shear_bulk_separation");
                }
            }
        } catch (const std::exception& error) {
            report.failures.push_back(std::string("exception:") + error.what());
        }
        report.passed = report.failures.empty();
        reports.push_back(std::move(report));
    }
    return reports;
}

std::string cpu_self_test_json(const std::vector<OracleCaseReport>& reports) {
    bool all_passed = sha256_hex("")
            == "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        && sha256_hex("abc")
            == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    std::ostringstream output;
    output << std::setprecision(17);
    output << "{\"schema\":\"nextengine.nonlocal.cpu_oracle_self_test.v0\""
           << ",\"profile_sha256\":\""
           << sha256_hex(canonical_profile_json(find_profile("nuv-tiny-oracle.v0"))) << "\""
           << ",\"sha256_known_vectors_passed\":" << (all_passed ? "true" : "false")
           << ",\"cases\":[";
    for (std::size_t case_index = 0; case_index < reports.size(); ++case_index) {
        const OracleCaseReport& report = reports[case_index];
        all_passed = all_passed && report.passed;
        if (case_index != 0) {
            output << ',';
        }
        output << "{\"name\":\"" << report.name << "\",\"passed\":"
               << (report.passed ? "true" : "false") << ",\"failures\":[";
        for (std::size_t failure_index = 0; failure_index < report.failures.size(); ++failure_index) {
            if (failure_index != 0) {
                output << ',';
            }
            output << '\"' << report.failures[failure_index] << '\"';
        }
        const OracleResult& result = report.result;
        output << "],\"directed_pairs\":" << result.directed_pairs
               << ",\"maximum_degree\":" << result.maximum_degree
               << ",\"normalized_momentum_residual\":"
               << result.normalized_momentum_residual << ",\"energy\":{\"inertia\":"
               << result.energy.inertia << ",\"incompressibility\":"
               << result.energy.incompressibility << ",\"bulk_viscosity\":"
               << result.energy.bulk_viscosity << ",\"shear_viscosity\":"
               << result.energy.shear_viscosity << ",\"surface_tension\":"
               << result.energy.surface_tension << "},\"density\":";
        append_array(output, result.density, [](std::ostringstream& stream, double value) {
            stream << value;
        });
        output << ",\"source\":";
        append_array(output, result.source, append_vec3);
        output << ",\"local_matrix_row_major\":";
        append_array(output, result.local_matrix, [](std::ostringstream& stream, const Mat3& value) {
            stream << '[';
            for (std::size_t index = 0; index < value.v.size(); ++index) {
                if (index != 0) {
                    stream << ',';
                }
                stream << value.v[index];
            }
            stream << ']';
        });
        output << ",\"predicted_position\":";
        append_array(output, result.predicted_position, append_vec3);
        output << ",\"linearization_position\":";
        append_array(output, result.linearization_position, append_vec3);
        output << ",\"next_position\":";
        append_array(output, result.next_position, append_vec3);
        output << ",\"final_velocity\":";
        append_array(output, result.final_velocity, append_vec3);
        output << '}';
    }
    output << "],\"status\":\"" << (all_passed ? "PASS" : "FAIL") << "\"}";
    return output.str();
}

} // namespace nextengine::nonlocal
