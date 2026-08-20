#include "boundary_reference.hpp"

#include "math.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstring>
#include <iomanip>
#include <limits>
#include <sstream>
#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

namespace nextengine::nonlocal::fcr {
namespace {

constexpr double PI = 3.141592653589793238462643383279502884;
constexpr double REST_DENSITY = 1000.0;
constexpr double SPACING = 0.05;
constexpr double HORIZON = 0.15;
constexpr double MASS = REST_DENSITY * SPACING * SPACING * SPACING;
constexpr double KAPPA = 1226.25;
constexpr double RADIUS = 0.025;
constexpr double TIME_STEP = 1.0 / 240.0;
constexpr double LAYER_LIMIT = 2.0e-12;
constexpr double GRADIENT_LIMIT = 1.0e-7;
constexpr double HVP_LIMIT = 2.0e-6;
constexpr double DENSE_LIMIT = 2.0e-12;
constexpr double CONSERVATION_LIMIT = 1.0e-12;

using nextengine::nonlocal::Mat3;
using nextengine::nonlocal::Vec3;
using nextengine::nonlocal::dot;
using nextengine::nonlocal::finite;
using nextengine::nonlocal::norm;
using nextengine::nonlocal::norm_squared;

struct Fixture {
    std::string name;
    std::array<int, 3> cells{};
    std::vector<Vec3> fluid;
    std::vector<Vec3> boundary;
};

struct Evaluation {
    double energy = 0.0;
    std::vector<double> density;
    std::vector<Vec3> gradient;
    std::size_t active_centers = 0;
    std::size_t fluid_pairs = 0;
    std::size_t boundary_pairs = 0;
    double minimum_branch_margin = std::numeric_limits<double>::infinity();
};

struct DerivativeControl {
    std::string name;
    bool passed = false;
    std::size_t fluid_samples = 0;
    std::size_t boundary_samples = 0;
    std::size_t unique_pairs = 0;
    std::size_t active_centers = 0;
    double minimum_density_ratio = 0.0;
    double maximum_density_ratio = 0.0;
    double minimum_branch_margin = 0.0;
    double directional_gradient_error = 0.0;
    double hvp_finite_difference_error = 0.0;
    double dense_symmetry_error = 0.0;
    double dense_product_error = 0.0;
    double minimum_eigenvalue = 0.0;
    double maximum_eigenvalue = 0.0;
    double translation_energy_error = 0.0;
    double reaction_closure = 0.0;
    double fixed_boundary_displacement = 0.0;
    bool active_set_stable = false;
    bool capacity_valid = false;
};

struct LayerControl {
    std::string name;
    bool passed = false;
    std::size_t two_layer_samples = 0;
    std::size_t three_layer_samples = 0;
    std::size_t two_layer_pairs = 0;
    std::size_t three_layer_pairs = 0;
    double rest_density_error = 0.0;
    double density_error = 0.0;
    double energy_error = 0.0;
    double gradient_error = 0.0;
    double hvp_error = 0.0;
    double reaction_error = 0.0;
};

struct SweepResult {
    Vec3 position;
    Vec3 velocity;
    Vec3 fluid_impulse;
    Vec3 reaction;
    std::vector<int> features;
    double earliest_time_of_impact = 1.0;
    double maximum_penetration = 0.0;
};

struct ContactCase {
    std::string name;
    std::vector<int> expected_features;
    SweepResult result;
    bool passed = false;
};

double relative_error(double lhs, double rhs) {
    return std::abs(lhs - rhs)
        / std::max({std::abs(lhs), std::abs(rhs), 1.0e-30});
}

double vector_norm(const std::vector<Vec3>& values) {
    double squared = 0.0;
    for (Vec3 value : values) {
        squared += norm_squared(value);
    }
    return std::sqrt(squared);
}

double vector_difference_norm(
    const std::vector<Vec3>& lhs, const std::vector<Vec3>& rhs) {
    if (lhs.size() != rhs.size()) {
        throw std::invalid_argument("vector size mismatch");
    }
    double squared = 0.0;
    for (std::size_t i = 0; i < lhs.size(); ++i) {
        squared += norm_squared(lhs[i] - rhs[i]);
    }
    return std::sqrt(squared);
}

double vector_relative_error(
    const std::vector<Vec3>& lhs, const std::vector<Vec3>& rhs) {
    return vector_difference_norm(lhs, rhs)
        / std::max({vector_norm(lhs), vector_norm(rhs), 1.0e-30});
}

double cubic_weight_raw(double radius) {
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

double cubic_gradient_raw(double radius) {
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

double cubic_second_raw(double radius) {
    const double q = 2.0 * radius / HORIZON;
    const double alpha = 3.0 / (2.0 * PI * HORIZON * HORIZON * HORIZON);
    if (q > 2.0) {
        return 0.0;
    }
    const double second_q = q >= 1.0
        ? alpha * (2.0 - q)
        : alpha * (-2.0 + 3.0 * q);
    const double scale = 2.0 / HORIZON;
    return second_q * scale * scale;
}

double kernel_scale() {
    static const double scale = [] {
        const int extent = static_cast<int>(std::ceil(HORIZON / SPACING));
        double ratio = 0.0;
        for (int z = -extent; z <= extent; ++z) {
            for (int y = -extent; y <= extent; ++y) {
                for (int x = -extent; x <= extent; ++x) {
                    const double radius = SPACING * std::sqrt(
                        static_cast<double>(x * x + y * y + z * z));
                    ratio += SPACING * SPACING * SPACING
                        * cubic_weight_raw(radius);
                }
            }
        }
        if (!std::isfinite(ratio) || ratio <= 0.0) {
            throw std::runtime_error("invalid B2 lattice kernel sum");
        }
        return 1.0 / ratio;
    }();
    return scale;
}

double weight(double radius) {
    return kernel_scale() * cubic_weight_raw(radius);
}

double weight_gradient(double radius) {
    return kernel_scale() * cubic_gradient_raw(radius);
}

double weight_second(double radius) {
    return kernel_scale() * cubic_second_raw(radius);
}

Vec3 lattice_position(int x, int y, int z) {
    return {
        RADIUS + static_cast<double>(x) * SPACING,
        RADIUS + static_cast<double>(y) * SPACING,
        RADIUS + static_cast<double>(z) * SPACING,
    };
}

bool inside_cells(int x, int y, int z, std::array<int, 3> cells) {
    return x >= 0 && x < cells[0]
        && y >= 0 && y < cells[1]
        && z >= 0 && z < cells[2];
}

Fixture make_box_fixture(
    std::string name, std::array<int, 3> cells, int layers) {
    Fixture result;
    result.name = std::move(name);
    result.cells = cells;
    for (int z = 0; z < cells[2]; ++z) {
        for (int y = 0; y < cells[1]; ++y) {
            for (int x = 0; x < cells[0]; ++x) {
                result.fluid.push_back(lattice_position(x, y, z));
            }
        }
    }
    for (int z = -layers; z < cells[2] + layers; ++z) {
        for (int y = -layers; y < cells[1] + layers; ++y) {
            for (int x = -layers; x < cells[0] + layers; ++x) {
                if (inside_cells(x, y, z, cells)) {
                    continue;
                }
                const Vec3 candidate = lattice_position(x, y, z);
                bool visible = false;
                for (Vec3 fluid : result.fluid) {
                    if (norm(candidate - fluid) <= HORIZON) {
                        visible = true;
                        break;
                    }
                }
                if (visible) {
                    result.boundary.push_back(candidate);
                }
            }
        }
    }
    return result;
}

std::vector<Vec3> compressed_fluid(const Fixture& fixture, double factor) {
    const Vec3 center{
        0.5 * static_cast<double>(fixture.cells[0]) * SPACING,
        0.5 * static_cast<double>(fixture.cells[1]) * SPACING,
        0.5 * static_cast<double>(fixture.cells[2]) * SPACING,
    };
    std::vector<Vec3> result = fixture.fluid;
    for (Vec3& value : result) {
        value = center + factor * (value - center);
    }
    return result;
}

Evaluation evaluate(
    const std::vector<Vec3>& fluid, const std::vector<Vec3>& boundary) {
    Evaluation result;
    const std::size_t fluid_count = fluid.size();
    result.gradient.resize(fluid_count + boundary.size());
    result.density.assign(fluid_count, MASS * weight(0.0));
    for (std::size_t i = 0; i < fluid_count; ++i) {
        for (std::size_t j = i + 1; j < fluid_count; ++j) {
            const double radius = norm(fluid[i] - fluid[j]);
            if (radius <= HORIZON) {
                const double contribution = MASS * weight(radius);
                result.density[i] += contribution;
                result.density[j] += contribution;
                ++result.fluid_pairs;
            }
        }
        for (std::size_t b = 0; b < boundary.size(); ++b) {
            const double radius = norm(fluid[i] - boundary[b]);
            if (radius <= HORIZON) {
                result.density[i] += MASS * weight(radius);
                ++result.boundary_pairs;
            }
        }
    }

    for (std::size_t center = 0; center < fluid_count; ++center) {
        const double signed_compression =
            result.density[center] / REST_DENSITY - 1.0;
        result.minimum_branch_margin = std::min(
            result.minimum_branch_margin, std::abs(signed_compression));
        if (signed_compression <= 0.0) {
            continue;
        }
        ++result.active_centers;
        result.energy += 0.5 * KAPPA
            * signed_compression * signed_compression;
        const double scale = KAPPA * signed_compression
            * MASS / REST_DENSITY;
        for (std::size_t neighbor = 0; neighbor < fluid_count; ++neighbor) {
            if (neighbor == center) {
                continue;
            }
            const Vec3 displacement = fluid[center] - fluid[neighbor];
            const double radius = norm(displacement);
            if (radius <= 1.0e-15 || radius > HORIZON) {
                continue;
            }
            const Vec3 pair = scale * weight_gradient(radius)
                * (displacement / radius);
            result.gradient[center] += pair;
            result.gradient[neighbor] += -pair;
        }
        for (std::size_t b = 0; b < boundary.size(); ++b) {
            const Vec3 displacement = fluid[center] - boundary[b];
            const double radius = norm(displacement);
            if (radius <= 1.0e-15 || radius > HORIZON) {
                continue;
            }
            const Vec3 pair = scale * weight_gradient(radius)
                * (displacement / radius);
            result.gradient[center] += pair;
            result.gradient[fluid_count + b] += -pair;
        }
    }
    return result;
}

Vec3 radial_hessian_product(
    Vec3 normal, double radial, double tangential, Vec3 value) {
    const Vec3 projected = dot(normal, value) * normal;
    return radial * projected + tangential * (value - projected);
}

std::vector<Vec3> apply_hessian(
    const std::vector<Vec3>& fluid,
    const std::vector<Vec3>& boundary,
    const std::vector<Vec3>& direction) {
    const std::size_t fluid_count = fluid.size();
    const std::size_t total_count = fluid_count + boundary.size();
    if (direction.size() != total_count) {
        throw std::invalid_argument("B2 HVP direction size mismatch");
    }
    const Evaluation state = evaluate(fluid, boundary);
    std::vector<Vec3> result(total_count);
    for (std::size_t center = 0; center < fluid_count; ++center) {
        const double compression =
            state.density[center] / REST_DENSITY - 1.0;
        if (compression <= 0.0) {
            continue;
        }
        double compression_direction = 0.0;
        for (std::size_t neighbor = 0; neighbor < fluid_count; ++neighbor) {
            if (neighbor == center) {
                continue;
            }
            const Vec3 displacement = fluid[center] - fluid[neighbor];
            const double radius = norm(displacement);
            if (radius <= 1.0e-15 || radius > HORIZON) {
                continue;
            }
            const Vec3 jacobian = MASS / REST_DENSITY
                * weight_gradient(radius) * (displacement / radius);
            compression_direction += dot(
                jacobian, direction[center] - direction[neighbor]);
        }
        for (std::size_t b = 0; b < boundary.size(); ++b) {
            const Vec3 displacement = fluid[center] - boundary[b];
            const double radius = norm(displacement);
            if (radius <= 1.0e-15 || radius > HORIZON) {
                continue;
            }
            const Vec3 jacobian = MASS / REST_DENSITY
                * weight_gradient(radius) * (displacement / radius);
            compression_direction += dot(jacobian,
                direction[center] - direction[fluid_count + b]);
        }

        const auto accumulate_pair = [&](std::size_t participant,
                                         Vec3 displacement) {
            const double radius = norm(displacement);
            if (radius <= 1.0e-15 || radius > HORIZON) {
                return;
            }
            const Vec3 normal = displacement / radius;
            const Vec3 jacobian = MASS / REST_DENSITY
                * weight_gradient(radius) * normal;
            const Vec3 relative_direction =
                direction[center] - direction[participant];
            const Vec3 curvature = MASS / REST_DENSITY
                * radial_hessian_product(normal,
                    weight_second(radius),
                    weight_gradient(radius) / radius,
                    relative_direction);
            const Vec3 pair = KAPPA
                * (compression_direction * jacobian
                    + compression * curvature);
            result[center] += pair;
            result[participant] += -pair;
        };
        for (std::size_t neighbor = 0; neighbor < fluid_count; ++neighbor) {
            if (neighbor != center) {
                accumulate_pair(
                    neighbor, fluid[center] - fluid[neighbor]);
            }
        }
        for (std::size_t b = 0; b < boundary.size(); ++b) {
            accumulate_pair(fluid_count + b,
                fluid[center] - boundary[b]);
        }
    }
    return result;
}

std::vector<Vec3> deterministic_direction(std::size_t count) {
    std::vector<Vec3> result(count);
    for (std::size_t i = 0; i < count; ++i) {
        const double index = static_cast<double>(i + 1U);
        result[i] = {
            std::sin(0.73 * index),
            std::cos(1.17 * index),
            std::sin(1.91 * index + 0.3),
        };
    }
    const double magnitude = vector_norm(result);
    for (Vec3& value : result) {
        value = value / magnitude;
    }
    return result;
}

std::vector<Vec3> displace(
    const std::vector<Vec3>& values,
    const std::vector<Vec3>& direction,
    double scale) {
    if (values.size() != direction.size()) {
        throw std::invalid_argument("B2 displacement size mismatch");
    }
    std::vector<Vec3> result = values;
    for (std::size_t i = 0; i < result.size(); ++i) {
        result[i] += scale * direction[i];
    }
    return result;
}

std::vector<Vec3> fluid_part(
    const std::vector<Vec3>& values, std::size_t fluid_count) {
    return {values.begin(), values.begin()
        + static_cast<std::ptrdiff_t>(fluid_count)};
}

Vec3 sum_values(
    const std::vector<Vec3>& values, std::size_t begin, std::size_t end) {
    Vec3 result;
    for (std::size_t i = begin; i < end; ++i) {
        result += values[i];
    }
    return result;
}

double component(Vec3 value, int axis) {
    if (axis == 0) {
        return value.x;
    }
    if (axis == 1) {
        return value.y;
    }
    return value.z;
}

void set_component(Vec3& value, int axis, double scalar) {
    if (axis == 0) {
        value.x = scalar;
    } else if (axis == 1) {
        value.y = scalar;
    } else {
        value.z = scalar;
    }
}

std::vector<double> flatten(const std::vector<Vec3>& values) {
    std::vector<double> result;
    result.reserve(3U * values.size());
    for (Vec3 value : values) {
        result.push_back(value.x);
        result.push_back(value.y);
        result.push_back(value.z);
    }
    return result;
}

std::pair<double, double> symmetric_eigenvalue_bounds(
    std::vector<double> matrix, std::size_t dimension) {
    if (matrix.size() != dimension * dimension) {
        throw std::invalid_argument("B2 eigensystem size mismatch");
    }
    const std::size_t maximum_sweeps = 40U * dimension * dimension;
    for (std::size_t sweep = 0; sweep < maximum_sweeps; ++sweep) {
        std::size_t p = 0;
        std::size_t q = 0;
        double maximum = 0.0;
        for (std::size_t row = 0; row < dimension; ++row) {
            for (std::size_t column = row + 1U; column < dimension; ++column) {
                const double value = std::abs(matrix[row * dimension + column]);
                if (value > maximum) {
                    maximum = value;
                    p = row;
                    q = column;
                }
            }
        }
        if (maximum <= 1.0e-13) {
            break;
        }
        const double app = matrix[p * dimension + p];
        const double aqq = matrix[q * dimension + q];
        const double apq = matrix[p * dimension + q];
        const double angle = 0.5 * std::atan2(2.0 * apq, aqq - app);
        const double cosine = std::cos(angle);
        const double sine = std::sin(angle);
        for (std::size_t k = 0; k < dimension; ++k) {
            if (k == p || k == q) {
                continue;
            }
            const double akp = matrix[k * dimension + p];
            const double akq = matrix[k * dimension + q];
            const double rotated_p = cosine * akp - sine * akq;
            const double rotated_q = sine * akp + cosine * akq;
            matrix[k * dimension + p] = rotated_p;
            matrix[p * dimension + k] = rotated_p;
            matrix[k * dimension + q] = rotated_q;
            matrix[q * dimension + k] = rotated_q;
        }
        matrix[p * dimension + p] = cosine * cosine * app
            - 2.0 * sine * cosine * apq + sine * sine * aqq;
        matrix[q * dimension + q] = sine * sine * app
            + 2.0 * sine * cosine * apq + cosine * cosine * aqq;
        matrix[p * dimension + q] = 0.0;
        matrix[q * dimension + p] = 0.0;
    }
    double minimum = std::numeric_limits<double>::infinity();
    double maximum = -std::numeric_limits<double>::infinity();
    for (std::size_t i = 0; i < dimension; ++i) {
        minimum = std::min(minimum, matrix[i * dimension + i]);
        maximum = std::max(maximum, matrix[i * dimension + i]);
    }
    return {minimum, maximum};
}

DerivativeControl derivative_control(
    const Fixture& fixture, double compression_factor) {
    DerivativeControl result;
    result.name = fixture.name;
    result.fluid_samples = fixture.fluid.size();
    result.boundary_samples = fixture.boundary.size();
    const std::vector<Vec3> boundary_before = fixture.boundary;
    const std::vector<Vec3> fluid =
        compressed_fluid(fixture, compression_factor);
    const Evaluation state = evaluate(fluid, fixture.boundary);
    result.unique_pairs = state.fluid_pairs + state.boundary_pairs;
    result.active_centers = state.active_centers;
    result.minimum_branch_margin = state.minimum_branch_margin;
    const auto density_bounds = std::minmax_element(
        state.density.begin(), state.density.end());
    result.minimum_density_ratio = *density_bounds.first / REST_DENSITY;
    result.maximum_density_ratio = *density_bounds.second / REST_DENSITY;

    const std::vector<Vec3> direction =
        deterministic_direction(fluid.size());
    constexpr double gradient_epsilon = 2.0e-7;
    const double plus_energy = evaluate(
        displace(fluid, direction, gradient_epsilon), fixture.boundary).energy;
    const double minus_energy = evaluate(
        displace(fluid, direction, -gradient_epsilon), fixture.boundary).energy;
    double analytic_direction = 0.0;
    for (std::size_t i = 0; i < fluid.size(); ++i) {
        analytic_direction += dot(state.gradient[i], direction[i]);
    }
    const double finite_direction =
        (plus_energy - minus_energy) / (2.0 * gradient_epsilon);
    result.directional_gradient_error = relative_error(
        analytic_direction, finite_direction);

    std::vector<Vec3> joint_direction(
        fluid.size() + fixture.boundary.size());
    std::copy(direction.begin(), direction.end(), joint_direction.begin());
    const std::vector<Vec3> analytic_hvp = fluid_part(
        apply_hessian(fluid, fixture.boundary, joint_direction), fluid.size());
    constexpr double hvp_epsilon = 2.0e-7;
    const Evaluation plus = evaluate(
        displace(fluid, direction, hvp_epsilon), fixture.boundary);
    const Evaluation minus = evaluate(
        displace(fluid, direction, -hvp_epsilon), fixture.boundary);
    std::vector<Vec3> finite_hvp(fluid.size());
    for (std::size_t i = 0; i < fluid.size(); ++i) {
        finite_hvp[i] = (plus.gradient[i] - minus.gradient[i])
            / (2.0 * hvp_epsilon);
    }
    result.hvp_finite_difference_error =
        vector_relative_error(analytic_hvp, finite_hvp);
    result.active_set_stable = plus.active_centers == state.active_centers
        && minus.active_centers == state.active_centers
        && plus.fluid_pairs == state.fluid_pairs
        && minus.fluid_pairs == state.fluid_pairs
        && plus.boundary_pairs == state.boundary_pairs
        && minus.boundary_pairs == state.boundary_pairs;

    const std::size_t dimension = 3U * fluid.size();
    std::vector<double> dense(dimension * dimension);
    for (std::size_t column = 0; column < dimension; ++column) {
        std::vector<Vec3> basis(fluid.size() + fixture.boundary.size());
        set_component(basis[column / 3U], static_cast<int>(column % 3U), 1.0);
        const std::vector<double> product = flatten(fluid_part(
            apply_hessian(fluid, fixture.boundary, basis), fluid.size()));
        for (std::size_t row = 0; row < dimension; ++row) {
            dense[row * dimension + column] = product[row];
        }
    }
    double maximum_asymmetry = 0.0;
    double maximum_dense = 0.0;
    for (std::size_t row = 0; row < dimension; ++row) {
        for (std::size_t column = 0; column < dimension; ++column) {
            maximum_asymmetry = std::max(maximum_asymmetry,
                std::abs(dense[row * dimension + column]
                    - dense[column * dimension + row]));
            maximum_dense = std::max(maximum_dense,
                std::abs(dense[row * dimension + column]));
        }
    }
    result.dense_symmetry_error = maximum_asymmetry
        / std::max(maximum_dense, 1.0e-30);
    const std::vector<double> flat_direction = flatten(direction);
    std::vector<double> dense_product(dimension);
    for (std::size_t row = 0; row < dimension; ++row) {
        for (std::size_t column = 0; column < dimension; ++column) {
            dense_product[row] += dense[row * dimension + column]
                * flat_direction[column];
        }
    }
    const std::vector<double> flat_hvp = flatten(analytic_hvp);
    double product_difference = 0.0;
    double product_scale = 0.0;
    for (std::size_t i = 0; i < dimension; ++i) {
        product_difference += (dense_product[i] - flat_hvp[i])
            * (dense_product[i] - flat_hvp[i]);
        product_scale = std::max(product_scale,
            std::max(std::abs(dense_product[i]), std::abs(flat_hvp[i])));
    }
    result.dense_product_error = std::sqrt(product_difference)
        / std::max(product_scale * std::sqrt(static_cast<double>(dimension)),
            1.0e-30);
    const auto spectrum = symmetric_eigenvalue_bounds(dense, dimension);
    result.minimum_eigenvalue = spectrum.first;
    result.maximum_eigenvalue = spectrum.second;

    const Vec3 translation{0.011, -0.007, 0.013};
    std::vector<Vec3> translated_fluid = fluid;
    std::vector<Vec3> translated_boundary = fixture.boundary;
    for (Vec3& value : translated_fluid) {
        value += translation;
    }
    for (Vec3& value : translated_boundary) {
        value += translation;
    }
    const double translated_energy =
        evaluate(translated_fluid, translated_boundary).energy;
    result.translation_energy_error = relative_error(
        state.energy, translated_energy);
    const Vec3 fluid_gradient_sum =
        sum_values(state.gradient, 0U, fluid.size());
    const Vec3 boundary_gradient_sum = sum_values(
        state.gradient, fluid.size(), state.gradient.size());
    double reaction_scale = 0.0;
    for (Vec3 value : state.gradient) {
        reaction_scale += norm(value);
    }
    result.reaction_closure = norm(
        fluid_gradient_sum + boundary_gradient_sum)
        / std::max(reaction_scale, 1.0e-30);
    for (std::size_t i = 0; i < fixture.boundary.size(); ++i) {
        if (std::memcmp(&boundary_before[i], &fixture.boundary[i],
                sizeof(Vec3)) != 0) {
            result.fixed_boundary_displacement =
                std::numeric_limits<double>::infinity();
        }
    }
    result.capacity_valid = fluid.size() + fixture.boundary.size() <= 512U
        && result.unique_pairs <= 80U
            * (fluid.size() + fixture.boundary.size());
    result.passed = result.active_centers > 0U
        && result.directional_gradient_error <= GRADIENT_LIMIT
        && result.hvp_finite_difference_error <= HVP_LIMIT
        && result.dense_symmetry_error <= DENSE_LIMIT
        && result.dense_product_error <= DENSE_LIMIT
        && result.active_set_stable
        && result.translation_energy_error <= CONSERVATION_LIMIT
        && result.reaction_closure <= CONSERVATION_LIMIT
        && result.fixed_boundary_displacement == 0.0
        && result.capacity_valid;
    return result;
}

LayerControl layer_control(
    std::string name, std::array<int, 3> cells) {
    const Fixture two = make_box_fixture(name + "-two-layer", cells, 2);
    const Fixture three = make_box_fixture(name + "-three-layer", cells, 3);
    const Evaluation rest_two = evaluate(two.fluid, two.boundary);
    const Evaluation rest_three = evaluate(three.fluid, three.boundary);
    const std::vector<Vec3> compressed = compressed_fluid(two, 0.99);
    const Evaluation value_two = evaluate(compressed, two.boundary);
    const Evaluation value_three = evaluate(compressed, three.boundary);
    const std::vector<Vec3> fluid_direction =
        deterministic_direction(compressed.size());
    std::vector<Vec3> two_direction(
        compressed.size() + two.boundary.size());
    std::vector<Vec3> three_direction(
        compressed.size() + three.boundary.size());
    std::copy(fluid_direction.begin(), fluid_direction.end(), two_direction.begin());
    std::copy(fluid_direction.begin(), fluid_direction.end(), three_direction.begin());
    const std::vector<Vec3> hvp_two = apply_hessian(
        compressed, two.boundary, two_direction);
    const std::vector<Vec3> hvp_three = apply_hessian(
        compressed, three.boundary, three_direction);

    LayerControl result;
    result.name = std::move(name);
    result.two_layer_samples = two.fluid.size() + two.boundary.size();
    result.three_layer_samples = three.fluid.size() + three.boundary.size();
    result.two_layer_pairs = value_two.fluid_pairs + value_two.boundary_pairs;
    result.three_layer_pairs = value_three.fluid_pairs + value_three.boundary_pairs;
    for (std::size_t i = 0; i < rest_two.density.size(); ++i) {
        result.rest_density_error = std::max(result.rest_density_error,
            std::abs(rest_two.density[i] / REST_DENSITY - 1.0));
        result.rest_density_error = std::max(result.rest_density_error,
            std::abs(rest_three.density[i] / REST_DENSITY - 1.0));
        result.density_error = std::max(result.density_error,
            relative_error(value_two.density[i], value_three.density[i]));
    }
    result.energy_error = relative_error(value_two.energy, value_three.energy);
    result.gradient_error = vector_relative_error(
        fluid_part(value_two.gradient, compressed.size()),
        fluid_part(value_three.gradient, compressed.size()));
    result.hvp_error = vector_relative_error(
        fluid_part(hvp_two, compressed.size()),
        fluid_part(hvp_three, compressed.size()));
    const Vec3 two_reaction = sum_values(
        value_two.gradient, compressed.size(), value_two.gradient.size());
    const Vec3 three_reaction = sum_values(
        value_three.gradient, compressed.size(), value_three.gradient.size());
    result.reaction_error = norm(two_reaction - three_reaction)
        / std::max({norm(two_reaction), norm(three_reaction), 1.0e-30});
    result.passed = result.rest_density_error <= 1.0e-12
        && result.density_error <= LAYER_LIMIT
        && result.energy_error <= LAYER_LIMIT
        && result.gradient_error <= LAYER_LIMIT
        && result.hvp_error <= LAYER_LIMIT
        && result.reaction_error <= LAYER_LIMIT
        && result.two_layer_samples <= 512U;
    return result;
}

SweepResult sweep_box(Vec3 start, Vec3 tentative) {
    SweepResult result;
    result.position = tentative;
    const Vec3 incoming_velocity = (tentative - start) / TIME_STEP;
    constexpr std::array<double, 3> lower = {RADIUS, RADIUS, RADIUS};
    constexpr std::array<double, 3> upper = {
        1.0 - RADIUS, 1.0 - RADIUS, 1.0 - RADIUS};
    constexpr std::array<int, 3> lower_feature = {0, 2, 4};
    constexpr std::array<int, 3> upper_feature = {1, 3, 5};
    for (int axis = 0; axis < 3; ++axis) {
        const double start_value = component(start, axis);
        const double tentative_value = component(tentative, axis);
        const double displacement = tentative_value - start_value;
        if (tentative_value < lower[axis] && displacement < 0.0) {
            const double toi = (lower[axis] - start_value) / displacement;
            if (!std::isfinite(toi) || toi < 0.0 || toi > 1.0) {
                throw std::runtime_error("invalid lower B2 swept-box TOI");
            }
            set_component(result.position, axis, lower[axis]);
            result.features.push_back(lower_feature[axis]);
            result.earliest_time_of_impact = std::min(
                result.earliest_time_of_impact, toi);
        } else if (tentative_value > upper[axis] && displacement > 0.0) {
            const double toi = (upper[axis] - start_value) / displacement;
            if (!std::isfinite(toi) || toi < 0.0 || toi > 1.0) {
                throw std::runtime_error("invalid upper B2 swept-box TOI");
            }
            set_component(result.position, axis, upper[axis]);
            result.features.push_back(upper_feature[axis]);
            result.earliest_time_of_impact = std::min(
                result.earliest_time_of_impact, toi);
        }
    }
    std::sort(result.features.begin(), result.features.end());
    result.velocity = (result.position - start) / TIME_STEP;
    result.fluid_impulse = MASS * (result.velocity - incoming_velocity);
    result.reaction = -result.fluid_impulse;
    for (int axis = 0; axis < 3; ++axis) {
        result.maximum_penetration = std::max(result.maximum_penetration,
            std::max(lower[axis] - component(result.position, axis),
                component(result.position, axis) - upper[axis]));
    }
    result.maximum_penetration = std::max(result.maximum_penetration, 0.0);
    return result;
}

ContactCase contact_case(
    std::string name,
    Vec3 start,
    Vec3 tentative,
    std::vector<int> expected_features) {
    ContactCase result;
    result.name = std::move(name);
    result.expected_features = std::move(expected_features);
    result.result = sweep_box(start, tentative);
    const bool expected_active = !result.expected_features.empty();
    const bool finite_toi = result.result.earliest_time_of_impact >= 0.0
        && result.result.earliest_time_of_impact <= 1.0;
    const bool impulse_closed = norm(
        result.result.fluid_impulse + result.result.reaction) <= 1.0e-12;
    result.passed = result.result.features == result.expected_features
        && result.result.maximum_penetration <= 1.0e-12
        && finite(result.result.position)
        && finite(result.result.velocity)
        && finite(result.result.fluid_impulse)
        && finite(result.result.reaction)
        && impulse_closed
        && (!expected_active || finite_toi);
    return result;
}

void append_vec3(std::ostringstream& output, Vec3 value) {
    output << '[' << value.x << ',' << value.y << ',' << value.z << ']';
}

void append_derivative(
    std::ostringstream& output, const DerivativeControl& value) {
    output << std::setprecision(17)
           << "{\"name\":\"" << value.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL") << '"'
           << ",\"fluid_samples\":" << value.fluid_samples
           << ",\"boundary_samples\":" << value.boundary_samples
           << ",\"unique_pairs\":" << value.unique_pairs
           << ",\"active_centers\":" << value.active_centers
           << ",\"minimum_density_ratio\":" << value.minimum_density_ratio
           << ",\"maximum_density_ratio\":" << value.maximum_density_ratio
           << ",\"minimum_branch_margin\":" << value.minimum_branch_margin
           << ",\"directional_gradient_error\":"
           << value.directional_gradient_error
           << ",\"hvp_finite_difference_error\":"
           << value.hvp_finite_difference_error
           << ",\"dense_symmetry_error\":" << value.dense_symmetry_error
           << ",\"dense_product_error\":" << value.dense_product_error
           << ",\"dense_spectrum\":[" << value.minimum_eigenvalue << ','
           << value.maximum_eigenvalue << ']'
           << ",\"translation_energy_error\":"
           << value.translation_energy_error
           << ",\"reaction_closure\":" << value.reaction_closure
           << ",\"fixed_boundary_displacement\":"
           << value.fixed_boundary_displacement
           << ",\"active_set_stable\":"
           << (value.active_set_stable ? "true" : "false")
           << ",\"capacity_valid\":"
           << (value.capacity_valid ? "true" : "false") << '}';
}

void append_layer(std::ostringstream& output, const LayerControl& value) {
    output << std::setprecision(17)
           << "{\"name\":\"" << value.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL") << '"'
           << ",\"two_layer_samples\":" << value.two_layer_samples
           << ",\"three_layer_samples\":" << value.three_layer_samples
           << ",\"two_layer_pairs\":" << value.two_layer_pairs
           << ",\"three_layer_pairs\":" << value.three_layer_pairs
           << ",\"rest_density_error\":" << value.rest_density_error
           << ",\"density_error\":" << value.density_error
           << ",\"energy_error\":" << value.energy_error
           << ",\"gradient_error\":" << value.gradient_error
           << ",\"hvp_error\":" << value.hvp_error
           << ",\"reaction_error\":" << value.reaction_error << '}';
}

void append_contact(std::ostringstream& output, const ContactCase& value) {
    output << std::setprecision(17)
           << "{\"name\":\"" << value.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL") << "\",\"features\":[";
    for (std::size_t i = 0; i < value.result.features.size(); ++i) {
        if (i != 0) {
            output << ',';
        }
        output << value.result.features[i];
    }
    output << "],\"time_of_impact_fraction\":"
           << value.result.earliest_time_of_impact
           << ",\"maximum_penetration_m\":"
           << value.result.maximum_penetration
           << ",\"accepted_position_m\":";
    append_vec3(output, value.result.position);
    output << ",\"accepted_velocity_m_s\":";
    append_vec3(output, value.result.velocity);
    output << ",\"fluid_impulse_kg_m_s\":";
    append_vec3(output, value.result.fluid_impulse);
    output << ",\"boundary_reaction_kg_m_s\":";
    append_vec3(output, value.result.reaction);
    output << '}';
}

} // namespace

SplitBoundaryReport run_split_static_boundary_controls() {
    const Fixture corner = make_box_fixture("corner-box-2x2x2", {2, 2, 2}, 2);
    const Fixture face = make_box_fixture("face-slab-5x5x2", {5, 5, 2}, 2);
    const DerivativeControl corner_derivative = derivative_control(corner, 0.99);
    const DerivativeControl face_derivative = derivative_control(face, 0.99);
    const LayerControl corner_layers = layer_control("corner", {2, 2, 2});
    const LayerControl face_layers = layer_control("face-slab", {5, 5, 2});
    const double horizon_weight = weight(HORIZON);
    const double horizon_gradient = weight_gradient(HORIZON);
    const double horizon_second = weight_second(HORIZON);
    const bool kernel_closure = horizon_weight == 0.0
        && horizon_gradient == 0.0 && horizon_second == 0.0;

    const std::array<ContactCase, 5> contacts = {
        contact_case("face", {0.5, 0.075, 0.5}, {0.5, -0.075, 0.5}, {2}),
        contact_case("edge", {0.075, 0.075, 0.5}, {-0.075, -0.075, 0.5}, {0, 2}),
        contact_case("corner", {0.075, 0.075, 0.075}, {-0.075, -0.075, -0.075}, {0, 2, 4}),
        contact_case("grazing", {0.4, RADIUS, 0.5}, {0.6, RADIUS, 0.5}, {}),
        contact_case("moving-away", {0.5, RADIUS, 0.5}, {0.5, 0.075, 0.5}, {}),
    };
    bool contacts_passed = true;
    for (const ContactCase& value : contacts) {
        contacts_passed = contacts_passed && value.passed;
    }
    const Vec3 ghost_start{0.5, 0.075, 0.5};
    const Vec3 ghost_velocity{0.0, -120.0, 0.0};
    const Vec3 ghost_tentative = ghost_start + TIME_STEP * ghost_velocity;
    const bool ghost_only_penetrates = ghost_tentative.y < RADIUS;

    const bool passed = kernel_closure
        && corner_derivative.passed && face_derivative.passed
        && corner_layers.passed && face_layers.passed
        && contacts_passed && ghost_only_penetrates;
    std::string first_failure;
    if (!kernel_closure) {
        first_failure = "NSR3B2_KERNEL_HORIZON_CLOSURE";
    } else if (!corner_layers.passed) {
        first_failure = "NSR3B2_CORNER_LAYER_CORRESPONDENCE";
    } else if (!face_layers.passed) {
        first_failure = "NSR3B2_FACE_LAYER_CORRESPONDENCE";
    } else if (!corner_derivative.passed) {
        first_failure = "NSR3B2_CORNER_DERIVATIVES";
    } else if (!face_derivative.passed) {
        first_failure = "NSR3B2_FACE_DERIVATIVES";
    } else if (!contacts_passed) {
        first_failure = "NSR3B2_HARD_CONTACT";
    } else if (!ghost_only_penetrates) {
        first_failure = "NSR3B2_GHOST_NEGATIVE";
    }

    std::ostringstream result_material;
    result_material << std::setprecision(17)
                    << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
                    << kernel_scale() << '|' << horizon_weight << '|'
                    << horizon_gradient << '|' << horizon_second << '|'
                    << corner_layers.density_error << '|'
                    << corner_layers.gradient_error << '|'
                    << corner_layers.hvp_error << '|'
                    << face_layers.density_error << '|'
                    << face_layers.gradient_error << '|'
                    << face_layers.hvp_error << '|'
                    << corner_derivative.directional_gradient_error << '|'
                    << corner_derivative.hvp_finite_difference_error << '|'
                    << corner_derivative.dense_symmetry_error << '|'
                    << face_derivative.directional_gradient_error << '|'
                    << face_derivative.hvp_finite_difference_error << '|'
                    << face_derivative.dense_symmetry_error;
    for (const ContactCase& value : contacts) {
        result_material << '|' << value.name << ':' << value.passed << ':';
        for (int feature : value.result.features) {
            result_material << feature << ',';
        }
    }

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b2_split_boundary.v1\""
           << ",\"identity\":\"nuv-variational-fcr2+split-static-boundary-r0\""
           << ",\"solver_identity\":\"nuv-newton-krylov-r0+outer-state-hessian-tape-v1\""
           << ",\"parent_b1r1_result_sha256\":\"e215b0facc30445541a6f2fa9446fd9d8140bf5f66983180cb435de9863f535e\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"kernel\":{\"scale\":" << kernel_scale()
           << ",\"weight_at_horizon\":" << horizon_weight
           << ",\"gradient_at_horizon\":" << horizon_gradient
           << ",\"second_derivative_at_horizon\":" << horizon_second
           << ",\"closure_exact\":" << (kernel_closure ? "true" : "false")
           << "},\"layer_controls\":[";
    append_layer(report, corner_layers);
    report << ',';
    append_layer(report, face_layers);
    report << "],\"derivative_controls\":[";
    append_derivative(report, corner_derivative);
    report << ',';
    append_derivative(report, face_derivative);
    report << "],\"hard_contact\":{\"status\":\""
           << (contacts_passed ? "PASS" : "FAIL") << "\",\"cases\":[";
    for (std::size_t i = 0; i < contacts.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_contact(report, contacts[i]);
    }
    report << "]},\"ghost_support_negative\":{\"status\":\""
           << (ghost_only_penetrates ? "PASS" : "FAIL")
           << "\",\"start_position_m\":";
    append_vec3(report, ghost_start);
    report << ",\"velocity_m_s\":";
    append_vec3(report, ghost_velocity);
    report << ",\"tentative_position_m\":";
    append_vec3(report, ghost_tentative);
    report << ",\"penetrates_without_contact\":"
           << (ghost_only_penetrates ? "true" : "false") << '}'
           << ",\"candidate_selected\":" << (passed ? "true" : "false")
           << ",\"b3_design_authorized\":" << (passed ? "true" : "false")
           << ",\"trajectory_execution_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(result_material.str()) << "\"}";
    return {passed, report.str()};
}

namespace {

constexpr double SMOKE_FRAME_TIME = 1.0 / 240.0;
constexpr int SMOKE_FRAMES = 4;
constexpr double SPECTRAL_TARGET = 0.15;

struct SmokeFixture {
    std::string name;
    std::vector<Vec3> position;
    std::vector<Vec3> velocity;
    std::vector<Vec3> boundary;
    Vec3 gravity{0.0, -9.81, 0.0};
    std::array<bool, 3> lower_contact{};
    std::string geometry_sha256;
    double rejected_anchor_density_ratio = 0.0;
};

struct SmoothEvaluation {
    double total = 0.0;
    Evaluation support;
    std::vector<Vec3> gradient;
};

struct SmoothSolve {
    bool passed = false;
    std::string failure;
    std::vector<Vec3> position;
    Evaluation support;
    int outer_trials = 0;
    int accepted_trials = 0;
    int rejected_trials = 0;
    int hvp_calls = 0;
    int negative_curvature_exits = 0;
    int floor_stops = 0;
    double minimum_accepted_ratio = std::numeric_limits<double>::infinity();
    double final_gradient_norm = 0.0;
};

struct SmokeStep {
    bool passed = false;
    std::string failure;
    std::vector<Vec3> position;
    std::vector<Vec3> velocity;
    Vec3 fluid_support_impulse;
    Vec3 support_reaction;
    Vec3 fluid_contact_impulse;
    Vec3 contact_reaction;
    Vec3 gravity_impulse;
    double ledger_absolute = 0.0;
    double ledger_residual = 0.0;
    double support_reaction_closure = 0.0;
    double maximum_penetration = 0.0;
    double earliest_contact_fraction = 1.0;
    std::vector<std::pair<std::size_t, int>> contact_features;
    int active_centers = 0;
    std::size_t pairs = 0;
    int cache_invalidations = 0;
    SmoothSolve smooth;
};

struct SmokeRun {
    bool passed = false;
    std::string failure;
    std::vector<Vec3> position;
    std::vector<Vec3> velocity;
    int substeps = 0;
    int outer_trials = 0;
    int rejected_trials = 0;
    int hvp_calls = 0;
    int negative_curvature_exits = 0;
    int floor_stops = 0;
    int active_steps = 0;
    int contact_events = 0;
    int cache_invalidations = 0;
    std::size_t maximum_pairs = 0;
    double maximum_penetration = 0.0;
    double maximum_ledger_residual = 0.0;
    double maximum_support_reaction_closure = 0.0;
    double first_contact_time = std::numeric_limits<double>::infinity();
    Vec3 fluid_support_impulse;
    Vec3 support_reaction;
    Vec3 fluid_contact_impulse;
    Vec3 contact_reaction;
    Vec3 gravity_impulse;
    double maximum_ledger_absolute = 0.0;
    std::vector<std::pair<std::size_t, int>> terminal_contacts;
};

struct SmokeGate {
    bool passed = false;
    double position_difference = 0.0;
    double velocity_difference = 0.0;
    double normalized_position_error = 0.0;
    double normalized_velocity_error = 0.0;
    double relative_kinetic_error = 0.0;
    double contact_time_error = 0.0;
};

struct SmokeFrame {
    bool passed = false;
    int frame = 0;
    int active_centers = 0;
    int spectral_hvp_calls = 0;
    double maximum_eigenvalue = 0.0;
    double maximum_eigenfrequency = 0.0;
    int initial_substeps = 0;
    int refinement_depth = -1;
    int accepted_substeps = 0;
    int executed_substeps = 0;
    int discarded_substeps = 0;
    SmokeGate gate;
};

struct SmokeController {
    bool passed = false;
    std::string failure;
    std::vector<Vec3> position;
    std::vector<Vec3> velocity;
    std::vector<SmokeFrame> frames;
    int accepted_substeps = 0;
    int executed_substeps = 0;
    int discarded_substeps = 0;
    int spectral_hvp_calls = 0;
    int nonlinear_hvp_calls = 0;
    int contact_events = 0;
    int negative_curvature_exits = 0;
    double maximum_penetration = 0.0;
    double maximum_ledger_residual = 0.0;
    double failure_ledger_absolute = 0.0;
    double failure_ledger_residual = 0.0;
    int failure_active_centers = 0;
    int failure_outer_trials = 0;
    int failure_hvp_calls = 0;
    int failure_contact_events = 0;
    double first_contact_time = std::numeric_limits<double>::infinity();
    std::vector<std::pair<std::size_t, int>> terminal_contacts;
};

struct SmokeReference {
    bool passed = false;
    std::array<SmokeRun, 3> levels;
    std::array<double, 2> position_difference{};
    std::array<double, 2> velocity_difference{};
    double position_ratio = 0.0;
    double velocity_ratio = 0.0;
};

struct SmokeCase {
    bool passed = false;
    std::string failure;
    SmokeFixture fixture;
    SmokeController controller;
    SmokeReference reference;
    double final_position_error_dx = 0.0;
    double final_velocity_error_c = 0.0;
    double final_kinetic_error = 0.0;
    double final_contact_time_error = 0.0;
    bool terminal_contacts_exact = false;
};

Vec3 average_values(const std::vector<Vec3>& values) {
    Vec3 result;
    for (Vec3 value : values) {
        result += value;
    }
    return result / static_cast<double>(values.size());
}

Vec3 minimum_components(const std::vector<Vec3>& values) {
    Vec3 result{
        std::numeric_limits<double>::infinity(),
        std::numeric_limits<double>::infinity(),
        std::numeric_limits<double>::infinity(),
    };
    for (Vec3 value : values) {
        result.x = std::min(result.x, value.x);
        result.y = std::min(result.y, value.y);
        result.z = std::min(result.z, value.z);
    }
    return result;
}

double kinetic_energy(const std::vector<Vec3>& velocity) {
    double result = 0.0;
    for (Vec3 value : velocity) {
        result += 0.5 * MASS * norm_squared(value);
    }
    return result;
}

Vec3 momentum(const std::vector<Vec3>& velocity) {
    Vec3 result;
    for (Vec3 value : velocity) {
        result += MASS * value;
    }
    return result;
}

double flat_dot(
    const std::vector<Vec3>& lhs, const std::vector<Vec3>& rhs) {
    if (lhs.size() != rhs.size()) {
        throw std::invalid_argument("B3 flat dot size mismatch");
    }
    double result = 0.0;
    for (std::size_t i = 0; i < lhs.size(); ++i) {
        result += dot(lhs[i], rhs[i]);
    }
    return result;
}

std::vector<Vec3> add_scaled(
    const std::vector<Vec3>& lhs,
    const std::vector<Vec3>& rhs,
    double scale) {
    if (lhs.size() != rhs.size()) {
        throw std::invalid_argument("B3 vector update size mismatch");
    }
    std::vector<Vec3> result = lhs;
    for (std::size_t i = 0; i < result.size(); ++i) {
        result[i] += scale * rhs[i];
    }
    return result;
}

std::string geometry_hash(const SmokeFixture& fixture) {
    std::ostringstream material;
    material << std::setprecision(17) << fixture.name << '|';
    for (Vec3 value : fixture.position) {
        material << value.x << ',' << value.y << ',' << value.z << ';';
    }
    material << '|';
    for (Vec3 value : fixture.boundary) {
        material << value.x << ',' << value.y << ',' << value.z << ';';
    }
    return sha256_hex(material.str());
}

void compress_and_place(
    std::vector<Vec3>& values,
    std::array<bool, 3> placed_axes) {
    const Vec3 center = average_values(values);
    for (Vec3& value : values) {
        value = center + 0.99 * (value - center);
    }
    const Vec3 minimum = minimum_components(values);
    Vec3 offset;
    if (placed_axes[0]) {
        offset.x = 0.035 - minimum.x;
    }
    if (placed_axes[1]) {
        offset.y = 0.035 - minimum.y;
    }
    if (placed_axes[2]) {
        offset.z = 0.035 - minimum.z;
    }
    for (Vec3& value : values) {
        value += offset;
    }
}

SmokeFixture make_face_smoke_fixture() {
    SmokeFixture result;
    result.name = "face-slab-5x3x5";
    result.lower_contact = {false, true, false};
    for (int z = -2; z <= 2; ++z) {
        for (int y = 0; y < 3; ++y) {
            for (int x = -2; x <= 2; ++x) {
                result.position.push_back({
                    static_cast<double>(x) * SPACING,
                    0.035 + static_cast<double>(y) * SPACING,
                    static_cast<double>(z) * SPACING,
                });
            }
        }
    }
    compress_and_place(result.position, {false, true, false});
    result.velocity.assign(result.position.size(), {0.0, -1.0, 0.0});
    for (int z = -4; z <= 4; ++z) {
        for (int layer = 1; layer <= 2; ++layer) {
            for (int x = -4; x <= 4; ++x) {
                const Vec3 candidate{
                    static_cast<double>(x) * SPACING,
                    RADIUS - static_cast<double>(layer) * SPACING,
                    static_cast<double>(z) * SPACING,
                };
                if (std::any_of(result.position.begin(), result.position.end(),
                        [candidate](Vec3 fluid) {
                            return norm(fluid - candidate) <= HORIZON;
                        })) {
                    result.boundary.push_back(candidate);
                }
            }
        }
    }
    result.geometry_sha256 = geometry_hash(result);
    const double rejected_radius = RADIUS - (-0.115);
    result.rejected_anchor_density_ratio =
        MASS * weight(rejected_radius) / REST_DENSITY;
    return result;
}

SmokeFixture make_corner_smoke_fixture() {
    SmokeFixture result;
    result.name = "corner-column-3x3x5";
    result.lower_contact = {true, true, false};
    for (int z = -2; z <= 2; ++z) {
        for (int y = 0; y < 3; ++y) {
            for (int x = 0; x < 3; ++x) {
                result.position.push_back({
                    0.035 + static_cast<double>(x) * SPACING,
                    0.035 + static_cast<double>(y) * SPACING,
                    static_cast<double>(z) * SPACING,
                });
            }
        }
    }
    compress_and_place(result.position, {true, true, false});
    result.velocity.assign(result.position.size(), {-0.7, -0.7, 0.0});
    for (int z = -4; z <= 4; ++z) {
        for (int y = -2; y <= 4; ++y) {
            for (int x = -2; x <= 4; ++x) {
                if (x >= 0 && y >= 0) {
                    continue;
                }
                const Vec3 candidate{
                    x < 0
                        ? RADIUS - static_cast<double>(-x) * SPACING
                        : 0.035 + static_cast<double>(x) * SPACING,
                    y < 0
                        ? RADIUS - static_cast<double>(-y) * SPACING
                        : 0.035 + static_cast<double>(y) * SPACING,
                    static_cast<double>(z) * SPACING,
                };
                if (std::any_of(result.position.begin(), result.position.end(),
                        [candidate](Vec3 fluid) {
                            return norm(fluid - candidate) <= HORIZON;
                        })) {
                    const bool duplicate = std::any_of(
                        result.boundary.begin(), result.boundary.end(),
                        [candidate](Vec3 existing) {
                            return existing.x == candidate.x
                                && existing.y == candidate.y
                                && existing.z == candidate.z;
                        });
                    if (!duplicate) {
                        result.boundary.push_back(candidate);
                    }
                }
            }
        }
    }
    result.geometry_sha256 = geometry_hash(result);
    result.rejected_anchor_density_ratio =
        MASS * weight(RADIUS - (-0.115)) / REST_DENSITY;
    return result;
}

SmoothEvaluation smooth_evaluate(
    const std::vector<Vec3>& y,
    const std::vector<Vec3>& y_star,
    const std::vector<Vec3>& boundary,
    double time_step) {
    SmoothEvaluation result;
    result.support = evaluate(y, boundary);
    result.total = result.support.energy;
    result.gradient = fluid_part(result.support.gradient, y.size());
    const double inertia_scale = MASS / (time_step * time_step);
    for (std::size_t i = 0; i < y.size(); ++i) {
        const Vec3 displacement = y[i] - y_star[i];
        result.total += 0.5 * inertia_scale * norm_squared(displacement);
        result.gradient[i] += inertia_scale * displacement;
    }
    return result;
}

std::vector<Vec3> smooth_hvp(
    const std::vector<Vec3>& y,
    const std::vector<Vec3>& boundary,
    const std::vector<Vec3>& direction,
    double time_step) {
    std::vector<Vec3> joint_direction(y.size() + boundary.size());
    std::copy(direction.begin(), direction.end(), joint_direction.begin());
    std::vector<Vec3> result = fluid_part(
        apply_hessian(y, boundary, joint_direction), y.size());
    const double inertia_scale = MASS / (time_step * time_step);
    for (std::size_t i = 0; i < result.size(); ++i) {
        result[i] += inertia_scale * direction[i];
    }
    return result;
}

double trust_boundary_tau(
    const std::vector<Vec3>& point,
    const std::vector<Vec3>& direction,
    double radius) {
    const double a = flat_dot(direction, direction);
    const double b = 2.0 * flat_dot(point, direction);
    const double c = flat_dot(point, point) - radius * radius;
    const double discriminant = std::max(b * b - 4.0 * a * c, 0.0);
    return (-b + std::sqrt(discriminant)) / (2.0 * a);
}

std::vector<Vec3> trust_step(
    const std::vector<Vec3>& y,
    const std::vector<Vec3>& boundary,
    const std::vector<Vec3>& gradient,
    double time_step,
    double radius,
    int& hvp_calls,
    bool& negative_curvature) {
    std::vector<Vec3> point(gradient.size());
    std::vector<Vec3> residual = gradient;
    std::vector<Vec3> direction = gradient;
    for (Vec3& value : direction) {
        value = -value;
    }
    double residual_squared = flat_dot(residual, residual);
    const double initial_residual = std::sqrt(residual_squared);
    for (std::size_t iteration = 0;
         iteration < 3U * gradient.size(); ++iteration) {
        const std::vector<Vec3> image =
            smooth_hvp(y, boundary, direction, time_step);
        ++hvp_calls;
        const double curvature = flat_dot(direction, image);
        if (!std::isfinite(curvature) || curvature <= 0.0) {
            negative_curvature = true;
            return add_scaled(point, direction,
                trust_boundary_tau(point, direction, radius));
        }
        const double alpha = residual_squared / curvature;
        const std::vector<Vec3> candidate =
            add_scaled(point, direction, alpha);
        if (vector_norm(candidate) >= radius) {
            return add_scaled(point, direction,
                trust_boundary_tau(point, direction, radius));
        }
        point = candidate;
        std::vector<Vec3> next_residual = residual;
        for (std::size_t i = 0; i < next_residual.size(); ++i) {
            next_residual[i] += alpha * image[i];
        }
        const double next_squared = flat_dot(next_residual, next_residual);
        if (std::sqrt(next_squared)
            <= std::min(0.5, std::sqrt(initial_residual))
                * initial_residual) {
            return point;
        }
        const double beta = next_squared / residual_squared;
        for (std::size_t i = 0; i < direction.size(); ++i) {
            direction[i] = -next_residual[i] + beta * direction[i];
        }
        residual = std::move(next_residual);
        residual_squared = next_squared;
    }
    return point;
}

SmoothSolve solve_smooth_step(
    const std::vector<Vec3>& position,
    const std::vector<Vec3>& velocity,
    const std::vector<Vec3>& boundary,
    Vec3 gravity,
    double time_step) {
    std::vector<Vec3> y_star(position.size());
    for (std::size_t i = 0; i < position.size(); ++i) {
        y_star[i] = position[i]
            + time_step * (velocity[i] + time_step * gravity);
    }
    SmoothSolve result;
    result.position = y_star;
    SmoothEvaluation current = smooth_evaluate(
        result.position, y_star, boundary, time_step);
    if (current.support.active_centers == 0U) {
        result.passed = true;
        result.support = std::move(current.support);
        result.minimum_accepted_ratio = 1.0;
        return result;
    }
    double trust_radius = 0.25 * SPACING;
    const auto scaled_residual = [time_step](
        const std::vector<Vec3>& gradient) {
        double maximum = 0.0;
        for (Vec3 value : gradient) {
            maximum = std::max(maximum, norm(value));
        }
        return time_step * time_step / MASS * maximum / SPACING;
    };
    for (int outer = 0; outer < 64; ++outer) {
        result.outer_trials = outer + 1;
        result.final_gradient_norm = vector_norm(current.gradient);
        if (result.final_gradient_norm <= 1.0e-10
            || scaled_residual(current.gradient) <= 1.0e-8) {
            result.passed = true;
            result.support = std::move(current.support);
            return result;
        }
        bool negative_curvature = false;
        const std::vector<Vec3> step = trust_step(
            result.position, boundary, current.gradient, time_step,
            trust_radius, result.hvp_calls, negative_curvature);
        if (negative_curvature) {
            ++result.negative_curvature_exits;
        }
        const std::vector<Vec3> image = smooth_hvp(
            result.position, boundary, step, time_step);
        ++result.hvp_calls;
        const double predicted = -flat_dot(current.gradient, step)
            - 0.5 * flat_dot(step, image);
        const double energy_floor = 1024.0
            * std::numeric_limits<double>::epsilon()
            * std::max(std::abs(current.total), 1.0);
        if (predicted > 0.0 && predicted <= energy_floor
            && scaled_residual(current.gradient) <= 1.0e-7
            && vector_norm(step) / SPACING <= 1.0e-7) {
            ++result.floor_stops;
            result.passed = true;
            result.support = std::move(current.support);
            return result;
        }
        const std::vector<Vec3> trial_position =
            add_scaled(result.position, step, 1.0);
        SmoothEvaluation trial = smooth_evaluate(
            trial_position, y_star, boundary, time_step);
        const double actual = current.total - trial.total;
        const double ratio = predicted > 0.0 ? actual / predicted
                                             : -std::numeric_limits<double>::infinity();
        if (ratio < 0.25) {
            trust_radius *= 0.25;
        } else if (ratio > 0.75
            && vector_norm(step) >= 0.9 * trust_radius) {
            trust_radius = std::min(2.0 * trust_radius, 2.0 * SPACING);
        }
        if (predicted > 0.0 && actual > 0.0 && ratio >= 0.1) {
            ++result.accepted_trials;
            result.minimum_accepted_ratio = std::min(
                result.minimum_accepted_ratio, ratio);
            result.position = trial_position;
            current = std::move(trial);
        } else {
            ++result.rejected_trials;
            if (result.rejected_trials > 8) {
                result.failure = "REJECT_LIMIT";
                return result;
            }
        }
        if (trust_radius < 1.0e-14) {
            result.failure = "MINIMUM_TRUST_RADIUS";
            return result;
        }
    }
    result.failure = "OUTER_LIMIT";
    return result;
}

SweepResult sweep_lower_planes(
    Vec3 start,
    Vec3 tentative,
    const std::array<bool, 3>& lower_contact,
    double time_step) {
    SweepResult result;
    result.position = tentative;
    const Vec3 incoming_velocity = (tentative - start) / time_step;
    constexpr std::array<int, 3> feature = {0, 2, 4};
    for (int axis = 0; axis < 3; ++axis) {
        if (!lower_contact[static_cast<std::size_t>(axis)]) {
            continue;
        }
        const double start_value = component(start, axis);
        const double tentative_value = component(tentative, axis);
        const double displacement = tentative_value - start_value;
        if (tentative_value < RADIUS && displacement < 0.0) {
            const double toi = (RADIUS - start_value) / displacement;
            if (!std::isfinite(toi) || toi < 0.0 || toi > 1.0) {
                throw std::runtime_error("invalid B3 lower-plane TOI");
            }
            set_component(result.position, axis, RADIUS);
            result.features.push_back(feature[static_cast<std::size_t>(axis)]);
            result.earliest_time_of_impact = std::min(
                result.earliest_time_of_impact, toi);
        }
    }
    std::sort(result.features.begin(), result.features.end());
    result.velocity = (result.position - start) / time_step;
    result.fluid_impulse = MASS * (result.velocity - incoming_velocity);
    result.reaction = -result.fluid_impulse;
    for (int axis = 0; axis < 3; ++axis) {
        if (lower_contact[static_cast<std::size_t>(axis)]) {
            result.maximum_penetration = std::max(
                result.maximum_penetration,
                RADIUS - component(result.position, axis));
        }
    }
    result.maximum_penetration = std::max(result.maximum_penetration, 0.0);
    return result;
}

SmokeStep execute_smoke_step(
    const SmokeFixture& fixture,
    const std::vector<Vec3>& position,
    const std::vector<Vec3>& velocity,
    double time_step) {
    SmokeStep result;
    result.position = position;
    result.velocity = velocity;
    result.smooth = solve_smooth_step(position, velocity,
        fixture.boundary, fixture.gravity, time_step);
    if (!result.smooth.passed) {
        result.failure = "SMOOTH_SOLVE:" + result.smooth.failure;
        return result;
    }
    result.active_centers = static_cast<int>(
        result.smooth.support.active_centers);
    result.pairs = result.smooth.support.fluid_pairs
        + result.smooth.support.boundary_pairs;
    const Vec3 fluid_gradient = sum_values(
        result.smooth.support.gradient, 0U, position.size());
    const Vec3 boundary_gradient = sum_values(result.smooth.support.gradient,
        position.size(), result.smooth.support.gradient.size());
    result.fluid_support_impulse = -time_step * fluid_gradient;
    result.support_reaction = -time_step * boundary_gradient;
    const double support_scale = norm(result.fluid_support_impulse)
        + norm(result.support_reaction);
    result.support_reaction_closure = norm(
        result.fluid_support_impulse + result.support_reaction)
        / std::max(support_scale, 1.0e-30);

    std::vector<Vec3> smooth_velocity(position.size());
    for (std::size_t i = 0; i < position.size(); ++i) {
        smooth_velocity[i] = (result.smooth.position[i] - position[i])
            / time_step;
        const SweepResult contact = sweep_lower_planes(
            position[i], result.smooth.position[i],
            fixture.lower_contact, time_step);
        result.position[i] = contact.position;
        result.velocity[i] = contact.velocity;
        result.fluid_contact_impulse += contact.fluid_impulse;
        result.contact_reaction += contact.reaction;
        result.maximum_penetration = std::max(
            result.maximum_penetration, contact.maximum_penetration);
        if (!contact.features.empty()) {
            result.earliest_contact_fraction = std::min(
                result.earliest_contact_fraction,
                contact.earliest_time_of_impact);
            result.cache_invalidations += static_cast<int>(
                contact.features.size());
            for (int feature : contact.features) {
                result.contact_features.emplace_back(i, feature);
            }
        }
    }
    std::sort(result.contact_features.begin(), result.contact_features.end());
    result.gravity_impulse = static_cast<double>(position.size())
        * MASS * time_step * fixture.gravity;
    const Vec3 momentum_change =
        momentum(result.velocity) - momentum(velocity);
    const Vec3 ledger = momentum_change - result.gravity_impulse
        + result.support_reaction + result.contact_reaction;
    const double ledger_scale = norm(momentum_change)
        + norm(result.gravity_impulse) + norm(result.support_reaction)
        + norm(result.contact_reaction);
    result.ledger_absolute = norm(ledger);
    result.ledger_residual = result.ledger_absolute
        / std::max(ledger_scale, 1.0e-30);
    const bool contact_impulse_closed = norm(
        result.fluid_contact_impulse + result.contact_reaction) <= 1.0e-12;
    bool reconstructed = true;
    for (std::size_t i = 0; i < position.size(); ++i) {
        reconstructed = reconstructed
            && result.velocity[i].x
                == (result.position[i].x - position[i].x) / time_step
            && result.velocity[i].y
                == (result.position[i].y - position[i].y) / time_step
            && result.velocity[i].z
                == (result.position[i].z - position[i].z) / time_step;
    }
    const bool finite_state =
        std::all_of(result.position.begin(), result.position.end(),
            [](Vec3 value) { return finite(value); })
        && std::all_of(result.velocity.begin(), result.velocity.end(),
            [](Vec3 value) { return finite(value); });
    result.passed = result.maximum_penetration <= 1.0e-12
        && result.support_reaction_closure <= 1.0e-10
        && contact_impulse_closed && result.ledger_residual <= 1.0e-9
        && reconstructed && finite_state;
    if (result.maximum_penetration > 1.0e-12) {
        result.failure = "PENETRATION";
    } else if (result.support_reaction_closure > 1.0e-10) {
        result.failure = "SUPPORT_REACTION_CLOSURE";
    } else if (!contact_impulse_closed) {
        result.failure = "CONTACT_IMPULSE_CLOSURE";
    } else if (result.ledger_residual > 1.0e-9) {
        result.failure = "MOMENTUM_LEDGER";
    } else if (!reconstructed) {
        result.failure = "VELOCITY_RECONSTRUCTION";
    } else if (!finite_state) {
        result.failure = "NONFINITE_STATE";
    }
    return result;
}

SmokeRun run_smoke_interval(
    const SmokeFixture& fixture,
    const std::vector<Vec3>& start_position,
    const std::vector<Vec3>& start_velocity,
    int substeps,
    double interval_start,
    double interval_duration) {
    SmokeRun result;
    result.position = start_position;
    result.velocity = start_velocity;
    result.substeps = substeps;
    const double time_step = interval_duration / static_cast<double>(substeps);
    for (int substep = 0; substep < substeps; ++substep) {
        const SmokeStep step = execute_smoke_step(
            fixture, result.position, result.velocity, time_step);
        if (!step.passed) {
            result.maximum_ledger_residual = step.ledger_residual;
            result.maximum_ledger_absolute = step.ledger_absolute;
            result.maximum_support_reaction_closure =
                step.support_reaction_closure;
            result.maximum_penetration = step.maximum_penetration;
            result.active_steps = step.active_centers > 0 ? 1 : 0;
            result.contact_events = static_cast<int>(
                step.contact_features.size());
            result.outer_trials = step.smooth.outer_trials;
            result.rejected_trials = step.smooth.rejected_trials;
            result.hvp_calls = step.smooth.hvp_calls;
            result.negative_curvature_exits =
                step.smooth.negative_curvature_exits;
            result.floor_stops = step.smooth.floor_stops;
            result.failure = "SUBSTEP_" + std::to_string(substep)
                + ':' + step.failure;
            return result;
        }
        result.position = step.position;
        result.velocity = step.velocity;
        result.outer_trials += step.smooth.outer_trials;
        result.rejected_trials += step.smooth.rejected_trials;
        result.hvp_calls += step.smooth.hvp_calls;
        result.negative_curvature_exits +=
            step.smooth.negative_curvature_exits;
        result.floor_stops += step.smooth.floor_stops;
        result.active_steps += step.active_centers > 0 ? 1 : 0;
        result.contact_events += static_cast<int>(step.contact_features.size());
        result.cache_invalidations += step.cache_invalidations;
        result.maximum_pairs = std::max(result.maximum_pairs, step.pairs);
        result.maximum_penetration = std::max(
            result.maximum_penetration, step.maximum_penetration);
        result.maximum_ledger_residual = std::max(
            result.maximum_ledger_residual, step.ledger_residual);
        result.maximum_ledger_absolute = std::max(
            result.maximum_ledger_absolute, step.ledger_absolute);
        result.maximum_support_reaction_closure = std::max(
            result.maximum_support_reaction_closure,
            step.support_reaction_closure);
        result.fluid_support_impulse += step.fluid_support_impulse;
        result.support_reaction += step.support_reaction;
        result.fluid_contact_impulse += step.fluid_contact_impulse;
        result.contact_reaction += step.contact_reaction;
        result.gravity_impulse += step.gravity_impulse;
        result.terminal_contacts = step.contact_features;
        if (!step.contact_features.empty()
            && !std::isfinite(result.first_contact_time)) {
            result.first_contact_time = interval_start
                + (static_cast<double>(substep)
                    + step.earliest_contact_fraction) * time_step;
        }
    }
    result.passed = result.contact_events == result.cache_invalidations
        && result.maximum_pairs <= 80U
            * (fixture.position.size() + fixture.boundary.size())
        && fixture.position.size() + fixture.boundary.size() <= 512U;
    if (!result.passed) {
        result.failure = "RUN_CAPACITY_OR_EPOCH";
    }
    return result;
}

double rms_difference(
    const std::vector<Vec3>& lhs, const std::vector<Vec3>& rhs) {
    return vector_difference_norm(lhs, rhs)
        / std::sqrt(static_cast<double>(lhs.size()));
}

double event_time_error(double lhs, double rhs) {
    if (!std::isfinite(lhs) && !std::isfinite(rhs)) {
        return 0.0;
    }
    if (!std::isfinite(lhs) || !std::isfinite(rhs)) {
        return std::numeric_limits<double>::infinity();
    }
    return std::abs(lhs - rhs);
}

SmokeGate smoke_gate(
    const SmokeRun& coarse, const SmokeRun& fine) {
    SmokeGate result;
    result.position_difference = rms_difference(
        coarse.position, fine.position);
    result.velocity_difference = rms_difference(
        coarse.velocity, fine.velocity);
    result.normalized_position_error =
        result.position_difference / SPACING;
    result.normalized_velocity_error = result.velocity_difference
        / std::sqrt(KAPPA / MASS);
    result.relative_kinetic_error = std::abs(
        kinetic_energy(coarse.velocity) - kinetic_energy(fine.velocity))
        / std::max(kinetic_energy(fine.velocity), 1.0e-30);
    result.contact_time_error = event_time_error(
        coarse.first_contact_time, fine.first_contact_time);
    result.passed = coarse.passed && fine.passed
        && result.normalized_position_error <= 0.05
        && result.normalized_velocity_error <= 0.001
        && result.relative_kinetic_error <= 0.15
        && result.contact_time_error
            <= SMOKE_FRAME_TIME / static_cast<double>(coarse.substeps)
                + 64.0 * std::numeric_limits<double>::epsilon();
    return result;
}

struct SpectralEstimate {
    bool passed = false;
    int calls = 0;
    double maximum_eigenvalue = 0.0;
};

SpectralEstimate boundary_pressure_spectrum(
    const std::vector<Vec3>& position,
    const std::vector<Vec3>& boundary) {
    constexpr int iterations = 48;
    SpectralEstimate result;
    std::vector<std::vector<Vec3>> basis;
    basis.reserve(iterations);
    std::vector<Vec3> q = deterministic_direction(position.size());
    std::vector<Vec3> previous(position.size());
    double previous_beta = 0.0;
    std::vector<double> diagonal;
    std::vector<double> off_diagonal;
    for (int iteration = 0; iteration < iterations; ++iteration) {
        basis.push_back(q);
        std::vector<Vec3> joint(position.size() + boundary.size());
        std::copy(q.begin(), q.end(), joint.begin());
        std::vector<Vec3> image = fluid_part(
            apply_hessian(position, boundary, joint), position.size());
        ++result.calls;
        const double alpha = flat_dot(q, image);
        diagonal.push_back(alpha);
        for (std::size_t i = 0; i < image.size(); ++i) {
            image[i] += -alpha * q[i] - previous_beta * previous[i];
        }
        for (const std::vector<Vec3>& vector : basis) {
            const double projection = flat_dot(vector, image);
            for (std::size_t i = 0; i < image.size(); ++i) {
                image[i] += -projection * vector[i];
            }
        }
        const double beta = vector_norm(image);
        if (iteration + 1 < iterations) {
            if (!std::isfinite(beta) || beta <= 1.0e-18) {
                break;
            }
            off_diagonal.push_back(beta);
            previous = q;
            previous_beta = beta;
            q = image;
            for (Vec3& value : q) {
                value = value / beta;
            }
        }
    }
    const std::size_t dimension = diagonal.size();
    std::vector<double> tridiagonal(dimension * dimension);
    for (std::size_t i = 0; i < dimension; ++i) {
        tridiagonal[i * dimension + i] = diagonal[i];
        if (i + 1U < dimension) {
            tridiagonal[i * dimension + i + 1U] = off_diagonal[i];
            tridiagonal[(i + 1U) * dimension + i] = off_diagonal[i];
        }
    }
    result.maximum_eigenvalue =
        symmetric_eigenvalue_bounds(tridiagonal, dimension).second;
    result.passed = result.calls == iterations
        && std::isfinite(result.maximum_eigenvalue)
        && result.maximum_eigenvalue > 0.0;
    return result;
}

SmokeController run_smoke_controller(const SmokeFixture& fixture) {
    SmokeController result;
    result.position = fixture.position;
    result.velocity = fixture.velocity;
    for (int frame_index = 0; frame_index < SMOKE_FRAMES; ++frame_index) {
        SmokeFrame frame;
        frame.frame = frame_index;
        const Evaluation frame_state = evaluate(
            result.position, fixture.boundary);
        frame.active_centers = static_cast<int>(frame_state.active_centers);
        if (frame.active_centers > 0) {
            const SpectralEstimate spectrum = boundary_pressure_spectrum(
                result.position, fixture.boundary);
            frame.spectral_hvp_calls = spectrum.calls;
            frame.maximum_eigenvalue = spectrum.maximum_eigenvalue;
            frame.maximum_eigenfrequency = std::sqrt(
                std::max(spectrum.maximum_eigenvalue, 0.0) / MASS);
            if (!spectrum.passed) {
                result.failure = "FRAME_SPECTRUM";
                return result;
            }
            frame.initial_substeps = std::max(1,
                static_cast<int>(std::ceil(SMOKE_FRAME_TIME
                    * frame.maximum_eigenfrequency / SPECTRAL_TARGET)));
        } else {
            frame.initial_substeps = 1;
        }
        std::vector<SmokeRun> levels;
        for (int level = 0; level < 4; ++level) {
            levels.push_back(run_smoke_interval(fixture,
                result.position, result.velocity,
                frame.initial_substeps * (1 << level),
                static_cast<double>(frame_index) * SMOKE_FRAME_TIME,
                SMOKE_FRAME_TIME));
            if (!levels.back().passed) {
                result.failure_ledger_absolute =
                    levels.back().maximum_ledger_absolute;
                result.failure_ledger_residual =
                    levels.back().maximum_ledger_residual;
                result.failure_active_centers = levels.back().active_steps;
                result.failure_outer_trials = levels.back().outer_trials;
                result.failure_hvp_calls = levels.back().hvp_calls;
                result.failure_contact_events = levels.back().contact_events;
                result.failure = "FRAME_CANDIDATE:" + levels.back().failure;
                return result;
            }
            if (level > 0) {
                frame.gate = smoke_gate(levels[static_cast<std::size_t>(level - 1)],
                    levels[static_cast<std::size_t>(level)]);
                if (frame.gate.passed) {
                    frame.refinement_depth = level - 1;
                    frame.accepted_substeps =
                        levels[static_cast<std::size_t>(level)].substeps;
                    result.position =
                        levels[static_cast<std::size_t>(level)].position;
                    result.velocity =
                        levels[static_cast<std::size_t>(level)].velocity;
                    break;
                }
            }
        }
        if (frame.refinement_depth < 0) {
            result.failure = "FRAME_ERROR_GATE";
            return result;
        }
        for (int level = 0; level <= frame.refinement_depth + 1; ++level) {
            const SmokeRun& run = levels[static_cast<std::size_t>(level)];
            frame.executed_substeps += run.substeps;
            result.nonlinear_hvp_calls += run.hvp_calls;
        }
        frame.discarded_substeps = frame.executed_substeps
            - frame.accepted_substeps;
        const SmokeRun& accepted = levels[
            static_cast<std::size_t>(frame.refinement_depth + 1)];
        result.accepted_substeps += frame.accepted_substeps;
        result.executed_substeps += frame.executed_substeps;
        result.discarded_substeps += frame.discarded_substeps;
        result.spectral_hvp_calls += frame.spectral_hvp_calls;
        result.contact_events += accepted.contact_events;
        result.negative_curvature_exits +=
            accepted.negative_curvature_exits;
        result.maximum_penetration = std::max(
            result.maximum_penetration, accepted.maximum_penetration);
        result.maximum_ledger_residual = std::max(
            result.maximum_ledger_residual,
            accepted.maximum_ledger_residual);
        result.first_contact_time = std::min(
            result.first_contact_time, accepted.first_contact_time);
        result.terminal_contacts = accepted.terminal_contacts;
        frame.passed = true;
        result.frames.push_back(frame);
    }
    result.passed = result.contact_events > 0
        && result.maximum_penetration <= 1.0e-12
        && result.maximum_ledger_residual <= 1.0e-9;
    if (!result.passed) {
        result.failure = "CONTROLLER_PHYSICAL_GATE";
    }
    return result;
}

SmokeRun run_fixed_smoke(
    const SmokeFixture& fixture, int substeps_per_frame) {
    return run_smoke_interval(fixture, fixture.position, fixture.velocity,
        SMOKE_FRAMES * substeps_per_frame, 0.0,
        SMOKE_FRAMES * SMOKE_FRAME_TIME);
}

SmokeReference run_smoke_reference(const SmokeFixture& fixture) {
    constexpr std::array<int, 3> counts = {96, 192, 384};
    SmokeReference result;
    for (std::size_t i = 0; i < counts.size(); ++i) {
        result.levels[i] = run_fixed_smoke(fixture, counts[i]);
    }
    for (std::size_t i = 0; i < 2; ++i) {
        result.position_difference[i] = rms_difference(
            result.levels[i].position, result.levels[i + 1U].position);
        result.velocity_difference[i] = rms_difference(
            result.levels[i].velocity, result.levels[i + 1U].velocity);
    }
    result.position_ratio = result.position_difference[0]
        / result.position_difference[1];
    result.velocity_ratio = result.velocity_difference[0]
        / result.velocity_difference[1];
    result.passed = std::all_of(result.levels.begin(), result.levels.end(),
            [](const SmokeRun& run) { return run.passed; })
        && result.position_difference[0] > 0.0
        && result.position_difference[1] > 0.0
        && result.velocity_difference[0] > 0.0
        && result.velocity_difference[1] > 0.0
        && std::isfinite(result.position_ratio)
        && std::isfinite(result.velocity_ratio)
        && result.position_ratio >= 1.25 && result.position_ratio <= 2.75
        && result.velocity_ratio >= 1.25 && result.velocity_ratio <= 2.75;
    return result;
}

SmokeCase run_smoke_case(SmokeFixture fixture) {
    SmokeCase result;
    result.fixture = std::move(fixture);
    result.controller = run_smoke_controller(result.fixture);
    result.reference = run_smoke_reference(result.fixture);
    if (!result.controller.passed) {
        result.failure = "CONTROLLER:" + result.controller.failure;
        return result;
    }
    if (!result.reference.passed) {
        result.failure = "REFERENCE";
        return result;
    }
    const SmokeRun& reference = result.reference.levels[2];
    result.final_position_error_dx = rms_difference(
        result.controller.position, reference.position) / SPACING;
    result.final_velocity_error_c = rms_difference(
        result.controller.velocity, reference.velocity)
        / std::sqrt(KAPPA / MASS);
    result.final_kinetic_error = std::abs(
        kinetic_energy(result.controller.velocity)
            - kinetic_energy(reference.velocity))
        / std::max(kinetic_energy(reference.velocity), 1.0e-30);
    result.final_contact_time_error = event_time_error(
        result.controller.first_contact_time, reference.first_contact_time);
    result.terminal_contacts_exact = result.controller.terminal_contacts
        == reference.terminal_contacts;
    result.passed = result.final_position_error_dx <= 0.05
        && result.final_velocity_error_c <= 0.001
        && result.final_kinetic_error <= 0.15
        && result.final_contact_time_error <= SMOKE_FRAME_TIME
        && result.controller.contact_events > 0
        && result.terminal_contacts_exact;
    if (!result.passed) {
        result.failure = "FINAL_ACCURACY";
    }
    return result;
}

void append_smoke_run(std::ostringstream& output, const SmokeRun& value) {
    output << std::setprecision(17)
           << "{\"status\":\"" << (value.passed ? "PASS" : "FAIL") << '"'
           << ",\"failure\":\"" << value.failure << '"'
           << ",\"substeps\":" << value.substeps
           << ",\"outer_trials\":" << value.outer_trials
           << ",\"rejected_trials\":" << value.rejected_trials
           << ",\"hvp_calls\":" << value.hvp_calls
           << ",\"negative_curvature_exits\":"
           << value.negative_curvature_exits
           << ",\"floor_stops\":" << value.floor_stops
           << ",\"active_steps\":" << value.active_steps
           << ",\"contact_events\":" << value.contact_events
           << ",\"cache_invalidations\":" << value.cache_invalidations
           << ",\"maximum_pairs\":" << value.maximum_pairs
           << ",\"maximum_penetration_m\":" << value.maximum_penetration
           << ",\"maximum_ledger_residual\":"
           << value.maximum_ledger_residual
           << ",\"maximum_ledger_absolute\":"
           << value.maximum_ledger_absolute
           << ",\"maximum_support_reaction_closure\":"
           << value.maximum_support_reaction_closure
           << ",\"first_contact_time_s\":" << value.first_contact_time
           << ",\"impulses\":{\"fluid_support\":";
    append_vec3(output, value.fluid_support_impulse);
    output << ",\"support_reaction\":";
    append_vec3(output, value.support_reaction);
    output << ",\"fluid_contact\":";
    append_vec3(output, value.fluid_contact_impulse);
    output << ",\"contact_reaction\":";
    append_vec3(output, value.contact_reaction);
    output << ",\"gravity\":";
    append_vec3(output, value.gravity_impulse);
    output << "}}";
}

void append_smoke_frame(
    std::ostringstream& output, const SmokeFrame& value) {
    output << std::setprecision(17)
           << "{\"frame\":" << value.frame
           << ",\"status\":\"" << (value.passed ? "PASS" : "FAIL") << '"'
           << ",\"active_centers\":" << value.active_centers
           << ",\"spectral_hvp_calls\":" << value.spectral_hvp_calls
           << ",\"maximum_eigenvalue\":" << value.maximum_eigenvalue
           << ",\"maximum_eigenfrequency\":"
           << value.maximum_eigenfrequency
           << ",\"initial_substeps\":" << value.initial_substeps
           << ",\"refinement_depth\":" << value.refinement_depth
           << ",\"accepted_substeps\":" << value.accepted_substeps
           << ",\"executed_substeps\":" << value.executed_substeps
           << ",\"discarded_substeps\":" << value.discarded_substeps
           << ",\"gate\":{\"position_difference\":"
           << value.gate.position_difference
           << ",\"velocity_difference\":" << value.gate.velocity_difference
           << ",\"normalized_position_error_dx\":"
           << value.gate.normalized_position_error
           << ",\"normalized_velocity_error_c\":"
           << value.gate.normalized_velocity_error
           << ",\"relative_kinetic_error\":"
           << value.gate.relative_kinetic_error
           << ",\"contact_time_error_s\":" << value.gate.contact_time_error
           << ",\"passed\":" << (value.gate.passed ? "true" : "false")
           << "}}";
}

void append_smoke_case(
    std::ostringstream& output, const SmokeCase& value) {
    output << std::setprecision(17)
           << "{\"name\":\"" << value.fixture.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL") << '"'
           << ",\"failure\":\"" << value.failure << '"'
           << ",\"fixture\":{\"fluid_samples\":"
           << value.fixture.position.size()
           << ",\"boundary_samples\":" << value.fixture.boundary.size()
           << ",\"geometry_sha256\":\""
           << value.fixture.geometry_sha256 << '"'
           << ",\"rejected_fluid_anchor_density_ratio\":"
           << value.fixture.rejected_anchor_density_ratio << '}'
           << ",\"controller\":{\"status\":\""
           << (value.controller.passed ? "PASS" : "FAIL") << '"'
           << ",\"failure\":\"" << value.controller.failure << '"'
           << ",\"accepted_substeps\":"
           << value.controller.accepted_substeps
           << ",\"executed_substeps\":"
           << value.controller.executed_substeps
           << ",\"discarded_substeps\":"
           << value.controller.discarded_substeps
           << ",\"spectral_hvp_calls\":"
           << value.controller.spectral_hvp_calls
           << ",\"nonlinear_hvp_calls\":"
           << value.controller.nonlinear_hvp_calls
           << ",\"contact_events\":" << value.controller.contact_events
           << ",\"negative_curvature_exits\":"
           << value.controller.negative_curvature_exits
           << ",\"maximum_penetration_m\":"
           << value.controller.maximum_penetration
           << ",\"maximum_ledger_residual\":"
           << value.controller.maximum_ledger_residual
           << ",\"failure_ledger_absolute\":"
           << value.controller.failure_ledger_absolute
           << ",\"failure_ledger_residual\":"
           << value.controller.failure_ledger_residual
           << ",\"failure_active_centers\":"
           << value.controller.failure_active_centers
           << ",\"failure_outer_trials\":"
           << value.controller.failure_outer_trials
           << ",\"failure_hvp_calls\":"
           << value.controller.failure_hvp_calls
           << ",\"failure_contact_events\":"
           << value.controller.failure_contact_events
           << ",\"first_contact_time_s\":"
           << value.controller.first_contact_time << ",\"frames\":[";
    for (std::size_t i = 0; i < value.controller.frames.size(); ++i) {
        if (i != 0) {
            output << ',';
        }
        append_smoke_frame(output, value.controller.frames[i]);
    }
    output << "]},\"reference\":{\"status\":\""
           << (value.reference.passed ? "PASS" : "FAIL") << '"'
           << ",\"substeps_per_frame\":[96,192,384]"
           << ",\"position_difference\":["
           << value.reference.position_difference[0] << ','
           << value.reference.position_difference[1] << ']'
           << ",\"velocity_difference\":["
           << value.reference.velocity_difference[0] << ','
           << value.reference.velocity_difference[1] << ']'
           << ",\"position_ratio\":" << value.reference.position_ratio
           << ",\"velocity_ratio\":" << value.reference.velocity_ratio
           << ",\"levels\":[";
    for (std::size_t i = 0; i < value.reference.levels.size(); ++i) {
        if (i != 0) {
            output << ',';
        }
        append_smoke_run(output, value.reference.levels[i]);
    }
    output << "]},\"final_gate\":{\"position_error_dx\":"
           << value.final_position_error_dx
           << ",\"velocity_error_c\":" << value.final_velocity_error_c
           << ",\"relative_kinetic_error\":"
           << value.final_kinetic_error
           << ",\"contact_time_error_s\":"
           << value.final_contact_time_error
           << ",\"terminal_contacts_exact\":"
           << (value.terminal_contacts_exact ? "true" : "false") << "}}";
}

} // namespace

SplitBoundaryReport run_boundary_composition_smoke_controls() {
    const SmokeCase face = run_smoke_case(make_face_smoke_fixture());
    const SmokeCase corner = run_smoke_case(make_corner_smoke_fixture());
    const bool passed = face.passed && corner.passed;
    std::string first_failure;
    if (!face.passed) {
        first_failure = "NSR3B3_FACE:" + face.failure;
    } else if (!corner.passed) {
        first_failure = "NSR3B3_CORNER:" + corner.failure;
    }
    std::ostringstream material;
    material << std::setprecision(17)
             << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
             << face.fixture.geometry_sha256 << '|'
             << face.controller.accepted_substeps << '|'
             << face.controller.executed_substeps << '|'
             << face.final_position_error_dx << '|'
             << face.final_velocity_error_c << '|'
             << face.reference.position_ratio << '|'
             << face.reference.velocity_ratio << '|'
             << corner.fixture.geometry_sha256 << '|'
             << corner.controller.accepted_substeps << '|'
             << corner.controller.executed_substeps << '|'
             << corner.final_position_error_dx << '|'
             << corner.final_velocity_error_c << '|'
             << corner.reference.position_ratio << '|'
             << corner.reference.velocity_ratio;

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b3_boundary_smoke.v1\""
           << ",\"identity\":\"nuv-variational-fcr2+split-static-boundary-r0\""
           << ",\"composition\":\"post-solve-swept-contact-composition-r0\""
           << ",\"parent_b2_result_sha256\":\"80a01b2ed0cf844841da322233b121b33e273c71eace8682795d0cad4e1dfb80\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"cases\":[";
    append_smoke_case(report, face);
    report << ',';
    append_smoke_case(report, corner);
    report << "]"
           << ",\"candidate_selected\":" << (passed ? "true" : "false")
           << ",\"physical_corpus_design_authorized\":"
           << (passed ? "true" : "false")
           << ",\"physical_corpus_execution_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(material.str()) << "\"}";
    return {passed, report.str()};
}

} // namespace nextengine::nonlocal::fcr
