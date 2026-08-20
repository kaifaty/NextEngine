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

} // namespace nextengine::nonlocal::fcr
