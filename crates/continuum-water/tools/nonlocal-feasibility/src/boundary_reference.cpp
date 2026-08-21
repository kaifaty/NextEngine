#include "boundary_reference.hpp"

#include "math.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <iomanip>
#include <limits>
#include <numeric>
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
    Vec3 displacement;
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
    bool closed_box_contact = false;
    Vec3 contact_low{RADIUS, RADIUS, RADIUS};
    Vec3 contact_high{RADIUS, RADIUS, RADIUS};
    int macro_frames = SMOKE_FRAMES;
    std::size_t maximum_participants = 512U;
    std::size_t maximum_pairs = 0U;
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
    std::vector<Vec3> displacement;
    std::vector<Vec3> last_step;
    Evaluation support;
    int outer_trials = 0;
    int accepted_trials = 0;
    int rejected_trials = 0;
    int hvp_calls = 0;
    int negative_curvature_exits = 0;
    int floor_stops = 0;
    int floor_merit_trials = 0;
    int floor_merit_accepts = 0;
    double minimum_accepted_ratio = std::numeric_limits<double>::infinity();
    double final_gradient_norm = 0.0;
    double final_scaled_displacement_residual = 0.0;
    double reaction_stationarity_defect = 0.0;
    double reaction_mixed_limit = 0.0;
    double numerical_energy_floor = 0.0;
    double last_predicted_reduction = 0.0;
    double last_trust_radius = 0.0;
    double floor_trial_reaction_defect = 0.0;
    double floor_trial_reaction_limit = 0.0;
    double floor_trial_residual_ratio = 0.0;
    bool floor_trial_topology_exact = false;
    std::string convergence_stop;
};

struct SmokeStep {
    bool passed = false;
    std::string failure;
    std::vector<Vec3> position;
    std::vector<Vec3> velocity;
    std::vector<Vec3> displacement;
    Vec3 fluid_support_impulse;
    Vec3 support_reaction;
    Vec3 fluid_contact_impulse;
    Vec3 contact_reaction;
    Vec3 gravity_impulse;
    Vec3 stationarity_defect;
    Vec3 translation_defect;
    Vec3 reconstruction_defect;
    Vec3 contact_defect;
    double reconstruction_fp_bound = 0.0;
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
    int inactive_steps = 0;
    int contact_events = 0;
    int floor_merit_trials = 0;
    int floor_merit_accepts = 0;
    int maximum_floor_accepts_per_solve = 0;
    bool floor_conditions_exact = true;
    int cache_invalidations = 0;
    std::size_t maximum_pairs = 0;
    double maximum_penetration = 0.0;
    double maximum_ledger_residual = 0.0;
    double maximum_support_reaction_closure = 0.0;
    double maximum_active_mixed_ratio = 0.0;
    double maximum_inactive_bound_ratio = 0.0;
    double cumulative_fp_bound = 0.0;
    double first_contact_time = std::numeric_limits<double>::infinity();
    Vec3 fluid_support_impulse;
    Vec3 support_reaction;
    Vec3 fluid_contact_impulse;
    Vec3 contact_reaction;
    Vec3 gravity_impulse;
    double maximum_ledger_absolute = 0.0;
    std::vector<std::pair<std::size_t, int>> terminal_contacts;
    double maximum_mechanical_energy = -std::numeric_limits<double>::infinity();
    double maximum_positive_density_strain = 0.0;
    double maximum_speed = 0.0;
    int precontact_steps = 0;
    int precontact_pressure_violations = 0;
    double maximum_precontact_support_reaction = 0.0;
    double maximum_precontact_position_error = 0.0;
    double maximum_precontact_velocity_error = 0.0;
    double maximum_precontact_velocity_spread = 0.0;
    double failure_reaction_defect = 0.0;
    double failure_reaction_limit = 0.0;
    double failure_predicted_reduction = 0.0;
    double failure_energy_floor = 0.0;
    double failure_scaled_residual = 0.0;
    double failure_step_dx = 0.0;
    int failure_floor_merit_trials = 0;
    int failure_floor_merit_accepts = 0;
    double failure_floor_trial_defect = 0.0;
    double failure_floor_trial_limit = 0.0;
    double failure_floor_residual_ratio = 0.0;
    bool failure_floor_topology_exact = false;
    int projected_trials = 0;
    int active_set_changes = 0;
    std::array<int, 6> face_active_axes{};
    std::array<double, 6> face_multiplier_sum{};
    std::array<double, 6> face_fluid_impulse{};
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
    std::vector<Vec3> position;
    std::vector<Vec3> velocity;
    std::array<int, 6> face_active_axes{};
    std::array<double, 6> face_multiplier_sum{};
    std::array<double, 6> face_fluid_impulse{};
    std::string spectrum_source;
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
    int outer_trials = 0;
    int rejected_trials = 0;
    int floor_merit_trials = 0;
    int floor_merit_accepts = 0;
    int maximum_floor_accepts_per_solve = 0;
    int active_steps = 0;
    int inactive_steps = 0;
    bool floor_conditions_exact = true;
    double maximum_penetration = 0.0;
    double maximum_ledger_residual = 0.0;
    double maximum_support_reaction_closure = 0.0;
    double maximum_active_mixed_ratio = 0.0;
    double maximum_inactive_bound_ratio = 0.0;
    double cumulative_fp_bound = 0.0;
    double failure_ledger_absolute = 0.0;
    double failure_ledger_residual = 0.0;
    int failure_active_centers = 0;
    int failure_outer_trials = 0;
    int failure_hvp_calls = 0;
    int failure_contact_events = 0;
    double first_contact_time = std::numeric_limits<double>::infinity();
    std::vector<std::pair<std::size_t, int>> terminal_contacts;
    int accepted_active_steps = 0;
    int accepted_inactive_steps = 0;
    std::size_t maximum_pairs = 0;
    Vec3 support_reaction;
    Vec3 contact_reaction;
    Vec3 gravity_impulse;
    double maximum_mechanical_energy = -std::numeric_limits<double>::infinity();
    double maximum_positive_density_strain = 0.0;
    double maximum_speed = 0.0;
    int precontact_steps = 0;
    int precontact_pressure_violations = 0;
    double maximum_precontact_support_reaction = 0.0;
    double maximum_precontact_position_error = 0.0;
    double maximum_precontact_velocity_error = 0.0;
    double maximum_precontact_velocity_spread = 0.0;
    int projected_trials = 0;
    int active_set_changes = 0;
    std::array<int, 6> face_active_axes{};
    std::array<double, 6> face_multiplier_sum{};
    std::array<double, 6> face_fluid_impulse{};
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

double gamma_factor(std::size_t operations) {
    const double value = static_cast<double>(operations)
        * std::numeric_limits<double>::epsilon();
    if (value >= 1.0) {
        throw std::runtime_error("B3D gamma operation count overflow");
    }
    return value / (1.0 - value);
}

double reconstruction_forward_bound(
    const std::vector<Vec3>& position,
    const std::vector<Vec3>& velocity,
    Vec3 gravity,
    double time_step) {
    const double local_gamma = gamma_factor(6U);
    const double summation_gamma = gamma_factor(
        std::max<std::size_t>(position.size(), 2U) - 1U);
    Vec3 component_bound;
    Vec3 component_magnitude;
    for (std::size_t i = 0; i < position.size(); ++i) {
        const Vec3 v_star = velocity[i] + time_step * gravity;
        const Vec3 predicted = position[i] + time_step * v_star;
        const Vec3 computed = MASS
            * ((predicted - position[i]) / time_step - v_star);
        component_bound.x += MASS * local_gamma
            * (std::abs(predicted.x) / time_step
                + std::abs(position[i].x) / time_step
                + std::abs(v_star.x));
        component_bound.y += MASS * local_gamma
            * (std::abs(predicted.y) / time_step
                + std::abs(position[i].y) / time_step
                + std::abs(v_star.y));
        component_bound.z += MASS * local_gamma
            * (std::abs(predicted.z) / time_step
                + std::abs(position[i].z) / time_step
                + std::abs(v_star.z));
        component_magnitude.x += std::abs(computed.x);
        component_magnitude.y += std::abs(computed.y);
        component_magnitude.z += std::abs(computed.z);
    }
    component_bound.x += summation_gamma * component_magnitude.x;
    component_bound.y += summation_gamma * component_magnitude.y;
    component_bound.z += summation_gamma * component_magnitude.z;
    return norm(component_bound);
}

double displacement_forward_bound(
    const std::vector<Vec3>& velocity,
    Vec3 gravity,
    double time_step) {
    const double local_gamma = gamma_factor(4U);
    const double summation_gamma = gamma_factor(
        std::max<std::size_t>(velocity.size(), 2U) - 1U);
    Vec3 component_bound;
    Vec3 component_magnitude;
    for (Vec3 value : velocity) {
        const Vec3 v_star = value + time_step * gravity;
        const Vec3 displacement = time_step * v_star;
        const Vec3 computed = MASS * (displacement / time_step - v_star);
        component_bound.x += MASS * local_gamma
            * (std::abs(displacement.x) / time_step
                + std::abs(v_star.x));
        component_bound.y += MASS * local_gamma
            * (std::abs(displacement.y) / time_step
                + std::abs(v_star.y));
        component_bound.z += MASS * local_gamma
            * (std::abs(displacement.z) / time_step
                + std::abs(v_star.z));
        component_magnitude.x += std::abs(computed.x);
        component_magnitude.y += std::abs(computed.y);
        component_magnitude.z += std::abs(computed.z);
    }
    component_bound.x += summation_gamma * component_magnitude.x;
    component_bound.y += summation_gamma * component_magnitude.y;
    component_bound.z += summation_gamma * component_magnitude.z;
    return norm(component_bound);
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

SmoothEvaluation smooth_evaluate_owned(
    const std::vector<Vec3>& y,
    const std::vector<Vec3>& displacement,
    const std::vector<Vec3>& predicted_displacement,
    const std::vector<Vec3>& boundary,
    double time_step) {
    if (y.size() != displacement.size()
        || y.size() != predicted_displacement.size()) {
        throw std::invalid_argument("owned smooth state size mismatch");
    }
    SmoothEvaluation result;
    result.support = evaluate(y, boundary);
    result.total = result.support.energy;
    result.gradient = fluid_part(result.support.gradient, y.size());
    const double inertia_scale = MASS / (time_step * time_step);
    for (std::size_t i = 0; i < y.size(); ++i) {
        const Vec3 error = displacement[i] - predicted_displacement[i];
        result.total += 0.5 * inertia_scale * norm_squared(error);
        result.gradient[i] += inertia_scale * error;
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
    double time_step,
    bool reaction_aware = false,
    bool owned_displacement = false,
    bool floor_stationarity_merit = false,
    bool fully_owned_inertia = false) {
    std::vector<Vec3> y_star(position.size());
    std::vector<Vec3> predicted_displacement(position.size());
    for (std::size_t i = 0; i < position.size(); ++i) {
        predicted_displacement[i] = time_step
            * (velocity[i] + time_step * gravity);
        y_star[i] = position[i] + predicted_displacement[i];
    }
    SmoothSolve result;
    result.position = y_star;
    result.displacement = predicted_displacement;
    SmoothEvaluation current = fully_owned_inertia
        ? smooth_evaluate_owned(result.position, result.displacement,
            predicted_displacement, boundary, time_step)
        : smooth_evaluate(result.position, y_star, boundary, time_step);
    if (current.support.active_centers == 0U) {
        result.passed = true;
        result.support = std::move(current.support);
        result.minimum_accepted_ratio = 1.0;
        result.convergence_stop = "INACTIVE_EXACT";
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
        result.final_scaled_displacement_residual =
            scaled_residual(current.gradient);
        const Vec3 fluid_gradient = sum_values(
            current.support.gradient, 0U, position.size());
        Vec3 actual_impulse;
        for (std::size_t i = 0; i < position.size(); ++i) {
            const Vec3 smooth_velocity = owned_displacement
                ? result.displacement[i] / time_step
                : (result.position[i] - position[i]) / time_step;
            actual_impulse += MASS * (
                smooth_velocity - velocity[i] - time_step * gravity);
        }
        const Vec3 model_impulse = -time_step * fluid_gradient;
        result.reaction_stationarity_defect = norm(
            actual_impulse - model_impulse);
        const double impulse_scale = std::max({
            norm(actual_impulse) + norm(model_impulse),
            static_cast<double>(position.size()) * MASS * time_step
                * norm(gravity),
            1.0e-12,
        });
        result.reaction_mixed_limit = 1.0e-9 * impulse_scale
            + (owned_displacement
                    ? displacement_forward_bound(
                        velocity, gravity, time_step)
                    : reconstruction_forward_bound(
                        position, velocity, gravity, time_step));
        const bool ordinary_converged =
            result.final_gradient_norm <= 1.0e-10
            || result.final_scaled_displacement_residual <= 1.0e-8;
        const bool reaction_converged =
            result.reaction_stationarity_defect
                <= result.reaction_mixed_limit;
        if ((!reaction_aware && ordinary_converged)
            || (reaction_aware && reaction_converged)) {
            result.passed = true;
            result.support = std::move(current.support);
            result.convergence_stop = reaction_aware
                ? "REACTION_MIXED_IMPULSE" : "SCALED_OR_RAW";
            return result;
        }
        bool negative_curvature = false;
        result.last_trust_radius = trust_radius;
        const std::vector<Vec3> step = trust_step(
            result.position, boundary, current.gradient, time_step,
            trust_radius, result.hvp_calls, negative_curvature);
        result.last_step = step;
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
        result.numerical_energy_floor = energy_floor;
        result.last_predicted_reduction = predicted;
        if (predicted > 0.0 && predicted <= energy_floor
            && scaled_residual(current.gradient) <= 1.0e-7
            && vector_norm(step) / SPACING <= 1.0e-7) {
            if (reaction_aware) {
                if (floor_stationarity_merit && owned_displacement
                    && !negative_curvature) {
                    ++result.floor_merit_trials;
                    std::vector<Vec3> trial_displacement = add_scaled(
                        result.displacement, step, 1.0);
                    std::vector<Vec3> trial_position(position.size());
                    for (std::size_t i = 0; i < position.size(); ++i) {
                        trial_position[i] = position[i]
                            + trial_displacement[i];
                    }
                    SmoothEvaluation trial = fully_owned_inertia
                        ? smooth_evaluate_owned(trial_position,
                            trial_displacement, predicted_displacement,
                            boundary, time_step)
                        : smooth_evaluate(trial_position, y_star,
                            boundary, time_step);
                    result.floor_trial_topology_exact =
                        current.support.active_centers
                            == trial.support.active_centers
                        && current.support.fluid_pairs
                            == trial.support.fluid_pairs
                        && current.support.boundary_pairs
                            == trial.support.boundary_pairs;
                    const Vec3 trial_fluid_gradient = sum_values(
                        trial.support.gradient, 0U, position.size());
                    Vec3 trial_actual_impulse;
                    for (std::size_t i = 0; i < position.size(); ++i) {
                        const Vec3 trial_velocity =
                            trial_displacement[i] / time_step;
                        trial_actual_impulse += MASS * (
                            trial_velocity - velocity[i]
                            - time_step * gravity);
                    }
                    const Vec3 trial_model_impulse =
                        -time_step * trial_fluid_gradient;
                    result.floor_trial_reaction_defect = norm(
                        trial_actual_impulse - trial_model_impulse);
                    const double trial_impulse_scale = std::max({
                        norm(trial_actual_impulse)
                            + norm(trial_model_impulse),
                        static_cast<double>(position.size()) * MASS
                            * time_step * norm(gravity),
                        1.0e-12,
                    });
                    result.floor_trial_reaction_limit =
                        1.0e-9 * trial_impulse_scale
                        + displacement_forward_bound(
                            velocity, gravity, time_step);
                    result.floor_trial_residual_ratio =
                        result.floor_trial_reaction_defect
                        / std::max(result.reaction_stationarity_defect,
                            1.0e-300);
                    const bool finite_trial = std::isfinite(
                            result.floor_trial_reaction_defect)
                        && std::isfinite(result.floor_trial_reaction_limit)
                        && std::all_of(trial_position.begin(),
                            trial_position.end(),
                            [](Vec3 value) { return finite(value); });
                    if (result.floor_trial_topology_exact && finite_trial
                        && result.floor_trial_reaction_defect
                            < result.reaction_stationarity_defect
                        && (fully_owned_inertia
                            || result.floor_trial_reaction_defect
                                <= result.floor_trial_reaction_limit)) {
                        if (fully_owned_inertia
                            && result.floor_merit_accepts >= 4) {
                            result.failure =
                                "FLOOR_STATIONARITY_ITERATION_LIMIT";
                            result.support = std::move(current.support);
                            return result;
                        }
                        ++result.floor_merit_accepts;
                        ++result.accepted_trials;
                        result.position = std::move(trial_position);
                        result.displacement = std::move(trial_displacement);
                        result.final_gradient_norm = vector_norm(
                            trial.gradient);
                        result.final_scaled_displacement_residual =
                            scaled_residual(trial.gradient);
                        result.reaction_stationarity_defect =
                            result.floor_trial_reaction_defect;
                        result.reaction_mixed_limit =
                            result.floor_trial_reaction_limit;
                        if (result.floor_trial_reaction_defect
                            <= result.floor_trial_reaction_limit) {
                            result.support = std::move(trial.support);
                            result.passed = true;
                            result.convergence_stop =
                                "FLOOR_STATIONARITY_MERIT";
                            return result;
                        }
                        current = std::move(trial);
                        continue;
                    }
                }
                result.failure = "REACTION_BELOW_ENERGY_RESOLUTION";
                result.support = std::move(current.support);
                return result;
            }
            ++result.floor_stops;
            result.passed = true;
            result.support = std::move(current.support);
            result.convergence_stop = "NUMERICAL_ENERGY_FLOOR";
            return result;
        }
        std::vector<Vec3> trial_displacement = result.displacement;
        std::vector<Vec3> trial_position;
        if (owned_displacement) {
            trial_displacement = add_scaled(
                result.displacement, step, 1.0);
            trial_position.resize(position.size());
            for (std::size_t i = 0; i < position.size(); ++i) {
                trial_position[i] = position[i] + trial_displacement[i];
            }
        } else {
            trial_position = add_scaled(result.position, step, 1.0);
        }
        SmoothEvaluation trial = fully_owned_inertia
            ? smooth_evaluate_owned(trial_position, trial_displacement,
                predicted_displacement, boundary, time_step)
            : smooth_evaluate(trial_position, y_star, boundary, time_step);
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
            if (owned_displacement) {
                result.displacement = trial_displacement;
            }
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

SweepResult sweep_lower_planes_owned(
    Vec3 start,
    Vec3 smooth_displacement,
    const std::array<bool, 3>& lower_contact,
    double time_step) {
    SweepResult result;
    result.displacement = smooth_displacement;
    const Vec3 tentative = start + smooth_displacement;
    const Vec3 incoming_velocity = smooth_displacement / time_step;
    constexpr std::array<int, 3> feature = {0, 2, 4};
    for (int axis = 0; axis < 3; ++axis) {
        if (!lower_contact[static_cast<std::size_t>(axis)]) {
            continue;
        }
        const double start_value = component(start, axis);
        const double tentative_value = component(tentative, axis);
        const double displacement = component(smooth_displacement, axis);
        if (tentative_value < RADIUS && displacement < 0.0) {
            const double toi = (RADIUS - start_value) / displacement;
            if (!std::isfinite(toi) || toi < 0.0 || toi > 1.0) {
                throw std::runtime_error("invalid D1 lower-plane TOI");
            }
            set_component(result.displacement, axis,
                RADIUS - start_value);
            result.features.push_back(feature[static_cast<std::size_t>(axis)]);
            result.earliest_time_of_impact = std::min(
                result.earliest_time_of_impact, toi);
        }
    }
    std::sort(result.features.begin(), result.features.end());
    result.position = start + result.displacement;
    result.velocity = result.displacement / time_step;
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

SweepResult sweep_box_owned(
    Vec3 start, Vec3 smooth_displacement, Vec3 low, Vec3 high,
    double time_step);

double rms_difference(
    const std::vector<Vec3>& lhs, const std::vector<Vec3>& rhs);

double mechanical_energy(
    const SmokeFixture& fixture,
    const std::vector<Vec3>& position,
    const std::vector<Vec3>& velocity) {
    double potential = 0.0;
    for (Vec3 value : position) {
        potential += MASS * (-fixture.gravity.y) * value.y;
    }
    return kinetic_energy(velocity)
        + evaluate(position, fixture.boundary).energy + potential;
}

double maximum_positive_density_strain(
    const SmokeFixture& fixture,
    const std::vector<Vec3>& position) {
    const Evaluation state = evaluate(position, fixture.boundary);
    double result = 0.0;
    for (double value : state.density) {
        result = std::max(result, value / REST_DENSITY - 1.0);
    }
    return result;
}

double maximum_speed(const std::vector<Vec3>& velocity) {
    double result = 0.0;
    for (Vec3 value : velocity) {
        result = std::max(result, norm(value));
    }
    return result;
}

SmokeStep execute_smoke_step(
    const SmokeFixture& fixture,
    const std::vector<Vec3>& position,
    const std::vector<Vec3>& velocity,
    double time_step,
    bool enforce_ledger = true,
    bool reaction_aware = false,
    bool owned_displacement = false,
    bool floor_stationarity_merit = false,
    bool fully_owned_inertia = false) {
    SmokeStep result;
    result.position = position;
    result.velocity = velocity;
    result.displacement.resize(position.size());
    result.smooth = solve_smooth_step(position, velocity,
        fixture.boundary, fixture.gravity, time_step,
        reaction_aware, owned_displacement, floor_stationarity_merit,
        fully_owned_inertia);
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
    result.translation_defect =
        result.fluid_support_impulse + result.support_reaction;
    const double support_scale = norm(result.fluid_support_impulse)
        + norm(result.support_reaction);
    result.support_reaction_closure = norm(
        result.fluid_support_impulse + result.support_reaction)
        / std::max(support_scale, 1.0e-30);

    std::vector<Vec3> smooth_velocity(position.size());
    Vec3 actual_support_impulse;
    Vec3 reconstruction_defect;
    for (std::size_t i = 0; i < position.size(); ++i) {
        smooth_velocity[i] = owned_displacement
            ? result.smooth.displacement[i] / time_step
            : (result.smooth.position[i] - position[i]) / time_step;
        const Vec3 v_star = velocity[i] + time_step * fixture.gravity;
        actual_support_impulse += MASS * (smooth_velocity[i] - v_star);
        if (owned_displacement) {
            const Vec3 predicted_displacement = time_step * v_star;
            reconstruction_defect += MASS
                * (predicted_displacement / time_step - v_star);
        } else {
            const Vec3 predicted = position[i] + time_step * v_star;
            reconstruction_defect += MASS
                * ((predicted - position[i]) / time_step - v_star);
        }
        const SweepResult contact = fixture.closed_box_contact
            ? sweep_box_owned(position[i],
                owned_displacement
                    ? result.smooth.displacement[i]
                    : result.smooth.position[i] - position[i],
                fixture.contact_low, fixture.contact_high, time_step)
            : (owned_displacement
                ? sweep_lower_planes_owned(position[i],
                    result.smooth.displacement[i],
                    fixture.lower_contact, time_step)
                : sweep_lower_planes(position[i], result.smooth.position[i],
                    fixture.lower_contact, time_step));
        result.position[i] = contact.position;
        result.velocity[i] = contact.velocity;
        result.displacement[i] = owned_displacement
            ? contact.displacement : contact.position - position[i];
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
    result.stationarity_defect =
        actual_support_impulse - result.fluid_support_impulse;
    result.reconstruction_defect = reconstruction_defect;
    result.reconstruction_fp_bound = owned_displacement
        ? displacement_forward_bound(velocity, fixture.gravity, time_step)
        : reconstruction_forward_bound(
            position, velocity, fixture.gravity, time_step);
    result.contact_defect =
        result.fluid_contact_impulse + result.contact_reaction;
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
        if (owned_displacement) {
            reconstructed = reconstructed
                && result.position[i].x
                    == position[i].x + result.displacement[i].x
                && result.position[i].y
                    == position[i].y + result.displacement[i].y
                && result.position[i].z
                    == position[i].z + result.displacement[i].z
                && result.velocity[i].x
                    == result.displacement[i].x / time_step
                && result.velocity[i].y
                    == result.displacement[i].y / time_step
                && result.velocity[i].z
                    == result.displacement[i].z / time_step;
        } else {
            reconstructed = reconstructed
                && result.velocity[i].x
                    == (result.position[i].x - position[i].x) / time_step
                && result.velocity[i].y
                    == (result.position[i].y - position[i].y) / time_step
                && result.velocity[i].z
                    == (result.position[i].z - position[i].z) / time_step;
        }
    }
    const bool finite_state =
        std::all_of(result.position.begin(), result.position.end(),
            [](Vec3 value) { return finite(value); })
        && std::all_of(result.velocity.begin(), result.velocity.end(),
            [](Vec3 value) { return finite(value); });
    result.passed = result.maximum_penetration <= 1.0e-12
        && result.support_reaction_closure <= 1.0e-10
        && contact_impulse_closed
        && (!enforce_ledger || result.ledger_residual <= 1.0e-9)
        && reconstructed && finite_state;
    if (result.maximum_penetration > 1.0e-12) {
        result.failure = "PENETRATION";
    } else if (result.support_reaction_closure > 1.0e-10) {
        result.failure = "SUPPORT_REACTION_CLOSURE";
    } else if (!contact_impulse_closed) {
        result.failure = "CONTACT_IMPULSE_CLOSURE";
    } else if (enforce_ledger && result.ledger_residual > 1.0e-9) {
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
    double interval_duration,
    bool enforce_ledger = true,
    bool reaction_aware = false,
    bool owned_displacement = false,
    bool floor_stationarity_merit = false,
    bool fully_owned_inertia = false) {
    SmokeRun result;
    result.position = start_position;
    result.velocity = start_velocity;
    result.substeps = substeps;
    result.maximum_mechanical_energy = mechanical_energy(
        fixture, result.position, result.velocity);
    result.maximum_positive_density_strain =
        maximum_positive_density_strain(fixture, result.position);
    result.maximum_speed = maximum_speed(result.velocity);
    std::vector<Vec3> expected_position = start_position;
    std::vector<Vec3> expected_velocity = start_velocity;
    bool precontact = true;
    const double time_step = interval_duration / static_cast<double>(substeps);
    for (int substep = 0; substep < substeps; ++substep) {
        const SmokeStep step = execute_smoke_step(
            fixture, result.position, result.velocity, time_step,
            enforce_ledger, reaction_aware, owned_displacement,
            floor_stationarity_merit, fully_owned_inertia);
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
            result.failure_reaction_defect =
                step.smooth.reaction_stationarity_defect;
            result.failure_reaction_limit =
                step.smooth.reaction_mixed_limit;
            result.failure_predicted_reduction =
                step.smooth.last_predicted_reduction;
            result.failure_energy_floor =
                step.smooth.numerical_energy_floor;
            result.failure_scaled_residual =
                step.smooth.final_scaled_displacement_residual;
            result.failure_step_dx = vector_norm(
                step.smooth.last_step) / SPACING;
            result.failure_floor_merit_trials =
                step.smooth.floor_merit_trials;
            result.failure_floor_merit_accepts =
                step.smooth.floor_merit_accepts;
            result.failure_floor_trial_defect =
                step.smooth.floor_trial_reaction_defect;
            result.failure_floor_trial_limit =
                step.smooth.floor_trial_reaction_limit;
            result.failure_floor_residual_ratio =
                step.smooth.floor_trial_residual_ratio;
            result.failure_floor_topology_exact =
                step.smooth.floor_trial_topology_exact;
            result.failure = "SUBSTEP_" + std::to_string(substep)
                + ':' + step.failure;
            return result;
        }
        result.position = step.position;
        result.velocity = step.velocity;
        result.maximum_mechanical_energy = std::max(
            result.maximum_mechanical_energy,
            mechanical_energy(fixture, result.position, result.velocity));
        result.maximum_positive_density_strain = std::max(
            result.maximum_positive_density_strain,
            maximum_positive_density_strain(fixture, result.position));
        result.maximum_speed = std::max(
            result.maximum_speed, maximum_speed(result.velocity));
        result.outer_trials += step.smooth.outer_trials;
        result.rejected_trials += step.smooth.rejected_trials;
        result.hvp_calls += step.smooth.hvp_calls;
        result.negative_curvature_exits +=
            step.smooth.negative_curvature_exits;
        result.floor_stops += step.smooth.floor_stops;
        const bool active = step.active_centers > 0;
        result.active_steps += active ? 1 : 0;
        result.inactive_steps += active ? 0 : 1;
        result.floor_merit_trials += step.smooth.floor_merit_trials;
        result.floor_merit_accepts += step.smooth.floor_merit_accepts;
        result.maximum_floor_accepts_per_solve = std::max(
            result.maximum_floor_accepts_per_solve,
            step.smooth.floor_merit_accepts);
        if (step.smooth.floor_merit_trials > 0) {
            result.floor_conditions_exact = result.floor_conditions_exact
                && step.smooth.floor_trial_topology_exact
                && step.smooth.floor_trial_residual_ratio < 1.0
                && step.smooth.floor_merit_trials
                    == step.smooth.floor_merit_accepts;
        }
        result.cumulative_fp_bound += step.reconstruction_fp_bound;
        if (active) {
            result.maximum_active_mixed_ratio = std::max(
                result.maximum_active_mixed_ratio,
                norm(step.stationarity_defect) / std::max(
                    step.smooth.reaction_mixed_limit, 1.0e-300));
        } else {
            result.maximum_inactive_bound_ratio = std::max(
                result.maximum_inactive_bound_ratio,
                norm(step.reconstruction_defect) / std::max(
                    step.reconstruction_fp_bound, 1.0e-300));
        }
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
        if (precontact && step.contact_features.empty()) {
            for (std::size_t i = 0; i < expected_position.size(); ++i) {
                expected_velocity[i] += time_step * fixture.gravity;
                expected_position[i] += time_step * expected_velocity[i];
            }
            ++result.precontact_steps;
            result.precontact_pressure_violations += active ? 1 : 0;
            result.maximum_precontact_support_reaction = std::max(
                result.maximum_precontact_support_reaction,
                norm(step.support_reaction));
            result.maximum_precontact_position_error = std::max(
                result.maximum_precontact_position_error,
                rms_difference(result.position, expected_position));
            result.maximum_precontact_velocity_error = std::max(
                result.maximum_precontact_velocity_error,
                rms_difference(result.velocity, expected_velocity));
            const Vec3 mean_velocity = average_values(result.velocity);
            for (Vec3 value : result.velocity) {
                result.maximum_precontact_velocity_spread = std::max(
                    result.maximum_precontact_velocity_spread,
                    norm(value - mean_velocity));
            }
        }
        if (!step.contact_features.empty()) {
            precontact = false;
        }
        if (!step.contact_features.empty()
            && !std::isfinite(result.first_contact_time)) {
            result.first_contact_time = interval_start
                + (static_cast<double>(substep)
                    + step.earliest_contact_fraction) * time_step;
        }
    }
    const std::size_t maximum_pairs = fixture.maximum_pairs > 0U
        ? fixture.maximum_pairs
        : 80U * (fixture.position.size() + fixture.boundary.size());
    result.passed = result.contact_events == result.cache_invalidations
        && result.maximum_pairs <= maximum_pairs
        && fixture.position.size() + fixture.boundary.size()
            <= fixture.maximum_participants;
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

SmokeController run_smoke_controller(
    const SmokeFixture& fixture,
    bool enforce_ledger = true,
    bool owned_residual_candidate = false) {
    SmokeController result;
    result.position = fixture.position;
    result.velocity = fixture.velocity;
    bool global_precontact = true;
    for (int frame_index = 0; frame_index < fixture.macro_frames;
         ++frame_index) {
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
                SMOKE_FRAME_TIME, enforce_ledger,
                owned_residual_candidate,
                owned_residual_candidate,
                owned_residual_candidate,
                owned_residual_candidate));
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
            result.outer_trials += run.outer_trials;
            result.rejected_trials += run.rejected_trials;
            result.floor_merit_trials += run.floor_merit_trials;
            result.floor_merit_accepts += run.floor_merit_accepts;
            result.maximum_floor_accepts_per_solve = std::max(
                result.maximum_floor_accepts_per_solve,
                run.maximum_floor_accepts_per_solve);
            result.floor_conditions_exact = result.floor_conditions_exact
                && run.floor_conditions_exact;
            result.active_steps += run.active_steps;
            result.inactive_steps += run.inactive_steps;
            result.maximum_active_mixed_ratio = std::max(
                result.maximum_active_mixed_ratio,
                run.maximum_active_mixed_ratio);
            result.maximum_inactive_bound_ratio = std::max(
                result.maximum_inactive_bound_ratio,
                run.maximum_inactive_bound_ratio);
            result.cumulative_fp_bound += run.cumulative_fp_bound;
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
        result.maximum_support_reaction_closure = std::max(
            result.maximum_support_reaction_closure,
            accepted.maximum_support_reaction_closure);
        result.first_contact_time = std::min(
            result.first_contact_time, accepted.first_contact_time);
        result.terminal_contacts = accepted.terminal_contacts;
        result.accepted_active_steps += accepted.active_steps;
        result.accepted_inactive_steps += accepted.inactive_steps;
        result.maximum_pairs = std::max(
            result.maximum_pairs, accepted.maximum_pairs);
        result.support_reaction += accepted.support_reaction;
        result.contact_reaction += accepted.contact_reaction;
        result.gravity_impulse += accepted.gravity_impulse;
        result.maximum_mechanical_energy = std::max(
            result.maximum_mechanical_energy,
            accepted.maximum_mechanical_energy);
        result.maximum_positive_density_strain = std::max(
            result.maximum_positive_density_strain,
            accepted.maximum_positive_density_strain);
        result.maximum_speed = std::max(
            result.maximum_speed, accepted.maximum_speed);
        if (global_precontact) {
            result.precontact_steps += accepted.precontact_steps;
            result.precontact_pressure_violations +=
                accepted.precontact_pressure_violations;
            result.maximum_precontact_support_reaction = std::max(
                result.maximum_precontact_support_reaction,
                accepted.maximum_precontact_support_reaction);
            result.maximum_precontact_position_error = std::max(
                result.maximum_precontact_position_error,
                accepted.maximum_precontact_position_error);
            result.maximum_precontact_velocity_error = std::max(
                result.maximum_precontact_velocity_error,
                accepted.maximum_precontact_velocity_error);
            result.maximum_precontact_velocity_spread = std::max(
                result.maximum_precontact_velocity_spread,
                accepted.maximum_precontact_velocity_spread);
            global_precontact = !std::isfinite(
                accepted.first_contact_time);
        }
        frame.position = result.position;
        frame.velocity = result.velocity;
        frame.passed = true;
        result.frames.push_back(frame);
    }
    result.passed = result.contact_events > 0
        && result.maximum_penetration <= 1.0e-12
        && (!enforce_ledger || result.maximum_ledger_residual <= 1.0e-9);
    if (!result.passed) {
        result.failure = "CONTROLLER_PHYSICAL_GATE";
    }
    return result;
}

SmokeRun run_fixed_smoke(
    const SmokeFixture& fixture,
    int substeps_per_frame,
    bool owned_residual_candidate = false) {
    return run_smoke_interval(fixture, fixture.position, fixture.velocity,
        fixture.macro_frames * substeps_per_frame, 0.0,
        fixture.macro_frames * SMOKE_FRAME_TIME, true,
        owned_residual_candidate, owned_residual_candidate,
        owned_residual_candidate, owned_residual_candidate);
}

SmokeReference run_smoke_reference(
    const SmokeFixture& fixture,
    bool owned_residual_candidate = false) {
    constexpr std::array<int, 3> counts = {96, 192, 384};
    SmokeReference result;
    for (std::size_t i = 0; i < counts.size(); ++i) {
        result.levels[i] = run_fixed_smoke(
            fixture, counts[i], owned_residual_candidate);
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

SmokeCase run_smoke_case(
    SmokeFixture fixture,
    bool owned_residual_candidate = false) {
    SmokeCase result;
    result.fixture = std::move(fixture);
    result.controller = run_smoke_controller(
        result.fixture, true, owned_residual_candidate);
    result.reference = run_smoke_reference(
        result.fixture, owned_residual_candidate);
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

struct ReactionCapture {
    bool valid = false;
    std::vector<Vec3> position;
    std::vector<Vec3> velocity;
    double time_step = 0.0;
    SmokeStep ordinary;
    std::string state_sha256;
};

struct ReactionTrace {
    bool passed = false;
    std::string failure;
    bool inactive_certified = false;
    int substeps_per_frame = 0;
    int steps = 0;
    int inactive_steps = 0;
    int active_steps = 0;
    int contact_steps = 0;
    int outer_trials = 0;
    int hvp_calls = 0;
    int rejected_trials = 0;
    int floor_merit_trials = 0;
    int floor_merit_accepts = 0;
    int maximum_floor_accepts_per_solve = 0;
    double maximum_floor_residual_ratio = 0.0;
    bool floor_conditions_exact = true;
    double maximum_stationarity_defect = 0.0;
    double maximum_translation_defect = 0.0;
    double maximum_reconstruction_defect = 0.0;
    double maximum_contact_defect = 0.0;
    double maximum_inactive_bound_ratio = 0.0;
    double maximum_active_mixed_ratio = 0.0;
    double maximum_fp_bound = 0.0;
    double cumulative_fp_bound = 0.0;
    Vec3 signed_cumulative_defect;
    double cumulative_l1_defect = 0.0;
    double direct_terminal_defect = 0.0;
    double failure_reaction_defect = 0.0;
    double failure_reaction_limit = 0.0;
    double failure_predicted_reduction = 0.0;
    double failure_energy_floor = 0.0;
    double failure_scaled_residual = 0.0;
    int failure_floor_merit_trials = 0;
    int failure_floor_merit_accepts = 0;
    int failure_negative_curvature_exits = 0;
    double failure_floor_trial_defect = 0.0;
    double failure_floor_trial_limit = 0.0;
    double failure_floor_residual_ratio = 0.0;
    bool failure_floor_topology_exact = false;
    Vec3 support_reaction;
    Vec3 contact_reaction;
    Vec3 gravity_impulse;
    std::vector<Vec3> final_position;
    std::vector<Vec3> final_velocity;
    ReactionCapture precontact;
    ReactionCapture contact;
    ReactionCapture energy_floor;
};

struct ReactionReplay {
    bool passed = false;
    std::string name;
    std::string failure;
    std::string state_sha256;
    int ordinary_outer_trials = 0;
    int ordinary_hvp_calls = 0;
    int reaction_outer_trials = 0;
    int reaction_hvp_calls = 0;
    double ordinary_stationarity_defect = 0.0;
    double reaction_stationarity_defect = 0.0;
    double reaction_mixed_limit = 0.0;
    double position_change_dx = 0.0;
    double velocity_change_c = 0.0;
    std::string reaction_stop;
};

std::string hash_smoke_state(
    const std::vector<Vec3>& position,
    const std::vector<Vec3>& velocity) {
    std::ostringstream material;
    material << std::setprecision(17);
    for (std::size_t i = 0; i < position.size(); ++i) {
        material << position[i].x << ',' << position[i].y << ','
                 << position[i].z << ':' << velocity[i].x << ','
                 << velocity[i].y << ',' << velocity[i].z << ';';
    }
    return sha256_hex(material.str());
}

ReactionTrace run_reaction_trace(
    const SmokeFixture& fixture,
    int substeps_per_frame,
    bool owned_displacement = false,
    bool reaction_aware = false,
    bool floor_stationarity_merit = false,
    bool fully_owned_inertia = false) {
    ReactionTrace result;
    result.substeps_per_frame = substeps_per_frame;
    result.final_position = fixture.position;
    result.final_velocity = fixture.velocity;
    const std::vector<Vec3> initial_velocity = result.final_velocity;
    const int total_substeps = SMOKE_FRAMES * substeps_per_frame;
    const double time_step = SMOKE_FRAME_TIME
        / static_cast<double>(substeps_per_frame);
    for (int substep = 0; substep < total_substeps; ++substep) {
        const SmokeStep step = execute_smoke_step(fixture,
            result.final_position, result.final_velocity,
            time_step, false, reaction_aware, owned_displacement,
            floor_stationarity_merit, fully_owned_inertia);
        if (!step.passed) {
            result.failure = "SUBSTEP_" + std::to_string(substep)
                + ':' + step.failure;
            result.failure_reaction_defect =
                step.smooth.reaction_stationarity_defect;
            result.failure_reaction_limit =
                step.smooth.reaction_mixed_limit;
            result.failure_predicted_reduction =
                step.smooth.last_predicted_reduction;
            result.failure_energy_floor =
                step.smooth.numerical_energy_floor;
            result.failure_scaled_residual =
                step.smooth.final_scaled_displacement_residual;
            result.failure_floor_merit_trials =
                step.smooth.floor_merit_trials;
            result.failure_floor_merit_accepts =
                step.smooth.floor_merit_accepts;
            result.failure_negative_curvature_exits =
                step.smooth.negative_curvature_exits;
            result.failure_floor_trial_defect =
                step.smooth.floor_trial_reaction_defect;
            result.failure_floor_trial_limit =
                step.smooth.floor_trial_reaction_limit;
            result.failure_floor_residual_ratio =
                step.smooth.floor_trial_residual_ratio;
            result.failure_floor_topology_exact =
                step.smooth.floor_trial_topology_exact;
            if (step.smooth.failure
                == "REACTION_BELOW_ENERGY_RESOLUTION") {
                result.energy_floor.valid = true;
                result.energy_floor.position = result.final_position;
                result.energy_floor.velocity = result.final_velocity;
                result.energy_floor.time_step = time_step;
                result.energy_floor.ordinary = step;
                result.energy_floor.state_sha256 = hash_smoke_state(
                    result.final_position, result.final_velocity);
            }
            return result;
        }
        ++result.steps;
        result.outer_trials += step.smooth.outer_trials;
        result.hvp_calls += step.smooth.hvp_calls;
        result.rejected_trials += step.smooth.rejected_trials;
        result.floor_merit_trials += step.smooth.floor_merit_trials;
        result.floor_merit_accepts += step.smooth.floor_merit_accepts;
        result.maximum_floor_accepts_per_solve = std::max(
            result.maximum_floor_accepts_per_solve,
            step.smooth.floor_merit_accepts);
        if (step.smooth.floor_merit_trials > 0) {
            result.maximum_floor_residual_ratio = std::max(
                result.maximum_floor_residual_ratio,
                step.smooth.floor_trial_residual_ratio);
            result.floor_conditions_exact = result.floor_conditions_exact
                && step.smooth.floor_trial_topology_exact
                && step.smooth.floor_trial_residual_ratio < 1.0
                && step.smooth.floor_trial_reaction_defect
                    <= step.smooth.floor_trial_reaction_limit
                && step.smooth.floor_merit_accepts
                    == step.smooth.floor_merit_trials;
        }
        const bool active = step.active_centers > 0;
        const bool contact = !step.contact_features.empty();
        result.inactive_steps += active ? 0 : 1;
        result.active_steps += active ? 1 : 0;
        result.contact_steps += contact ? 1 : 0;
        const double stationarity = norm(step.stationarity_defect);
        const double translation = norm(step.translation_defect);
        const double reconstruction = norm(step.reconstruction_defect);
        const double contact_defect = norm(step.contact_defect);
        result.maximum_stationarity_defect = std::max(
            result.maximum_stationarity_defect, stationarity);
        result.maximum_translation_defect = std::max(
            result.maximum_translation_defect, translation);
        result.maximum_reconstruction_defect = std::max(
            result.maximum_reconstruction_defect, reconstruction);
        result.maximum_contact_defect = std::max(
            result.maximum_contact_defect, contact_defect);
        result.maximum_fp_bound = std::max(
            result.maximum_fp_bound, step.reconstruction_fp_bound);
        result.cumulative_fp_bound += step.reconstruction_fp_bound;
        if (!active) {
            result.maximum_inactive_bound_ratio = std::max(
                result.maximum_inactive_bound_ratio,
                reconstruction
                    / std::max(step.reconstruction_fp_bound, 1.0e-300));
        } else {
            result.maximum_active_mixed_ratio = std::max(
                result.maximum_active_mixed_ratio,
                stationarity / std::max(
                    step.smooth.reaction_mixed_limit, 1.0e-300));
        }
        const Vec3 complete_defect = step.stationarity_defect
            + step.translation_defect + step.contact_defect;
        result.signed_cumulative_defect += complete_defect;
        result.cumulative_l1_defect += norm(complete_defect);
        result.support_reaction += step.support_reaction;
        result.contact_reaction += step.contact_reaction;
        result.gravity_impulse += step.gravity_impulse;
        if (active && !contact && !result.precontact.valid) {
            result.precontact.valid = true;
            result.precontact.position = result.final_position;
            result.precontact.velocity = result.final_velocity;
            result.precontact.time_step = time_step;
            result.precontact.ordinary = step;
            result.precontact.state_sha256 = hash_smoke_state(
                result.final_position, result.final_velocity);
        }
        if (active && contact && !result.contact.valid) {
            result.contact.valid = true;
            result.contact.position = result.final_position;
            result.contact.velocity = result.final_velocity;
            result.contact.time_step = time_step;
            result.contact.ordinary = step;
            result.contact.state_sha256 = hash_smoke_state(
                result.final_position, result.final_velocity);
        }
        result.final_position = step.position;
        result.final_velocity = step.velocity;
    }
    const Vec3 direct = momentum(result.final_velocity)
        - momentum(initial_velocity) - result.gravity_impulse
        + result.support_reaction + result.contact_reaction;
    result.direct_terminal_defect = norm(direct);
    const double momentum_budget = static_cast<double>(fixture.position.size())
        * MASS * std::sqrt(KAPPA / MASS);
    result.inactive_certified = result.maximum_inactive_bound_ratio <= 1.0
        && result.cumulative_fp_bound <= 1.0e-10 * momentum_budget;
    result.passed = result.steps == total_substeps
        && result.active_steps > 0 && result.contact_steps > 0
        && result.maximum_translation_defect <= 1.0e-12
        && result.maximum_contact_defect <= 1.0e-12
        && norm(result.signed_cumulative_defect)
            <= result.cumulative_l1_defect
                + 64.0 * std::numeric_limits<double>::epsilon()
        && result.direct_terminal_defect
            <= result.cumulative_l1_defect + result.cumulative_fp_bound;
    if (!result.passed) {
        result.failure = "TRACE_DEFECT_GATE";
    }
    return result;
}

ReactionReplay replay_reaction_aware(
    std::string name,
    const SmokeFixture& fixture,
    const ReactionCapture& capture,
    bool owned_displacement = false) {
    ReactionReplay result;
    result.name = std::move(name);
    result.state_sha256 = capture.state_sha256;
    if (!capture.valid) {
        result.failure = "CAPTURE_MISSING";
        return result;
    }
    result.ordinary_outer_trials = capture.ordinary.smooth.outer_trials;
    result.ordinary_hvp_calls = capture.ordinary.smooth.hvp_calls;
    result.ordinary_stationarity_defect = norm(
        capture.ordinary.stationarity_defect);
    const SmokeStep reaction = execute_smoke_step(fixture,
        capture.position, capture.velocity, capture.time_step,
        false, true, owned_displacement);
    result.reaction_outer_trials = reaction.smooth.outer_trials;
    result.reaction_hvp_calls = reaction.smooth.hvp_calls;
    result.reaction_stationarity_defect =
        reaction.smooth.reaction_stationarity_defect;
    result.reaction_mixed_limit = reaction.smooth.reaction_mixed_limit;
    result.reaction_stop = reaction.smooth.convergence_stop.empty()
        ? reaction.smooth.failure : reaction.smooth.convergence_stop;
    if (!reaction.smooth.passed) {
        result.failure = reaction.smooth.failure;
        return result;
    }
    result.position_change_dx = rms_difference(
        capture.ordinary.smooth.position,
        reaction.smooth.position) / SPACING;
    std::vector<Vec3> ordinary_velocity(capture.position.size());
    std::vector<Vec3> reaction_velocity(capture.position.size());
    for (std::size_t i = 0; i < capture.position.size(); ++i) {
        ordinary_velocity[i] = (capture.ordinary.smooth.position[i]
            - capture.position[i]) / capture.time_step;
        reaction_velocity[i] = owned_displacement
            ? reaction.smooth.displacement[i] / capture.time_step
            : (reaction.smooth.position[i]
                - capture.position[i]) / capture.time_step;
    }
    result.velocity_change_c = rms_difference(
        ordinary_velocity, reaction_velocity) / std::sqrt(KAPPA / MASS);
    const int hvp_limit = static_cast<int>(std::floor(
        2.5 * static_cast<double>(result.ordinary_hvp_calls) + 2.0));
    result.passed = result.reaction_stationarity_defect
            <= result.reaction_mixed_limit
        && result.position_change_dx <= 1.0e-5
        && result.velocity_change_c <= 1.0e-5
        && result.reaction_hvp_calls <= hvp_limit
        && reaction.smooth.rejected_trials <= 8;
    if (!result.passed) {
        result.failure = "REACTION_REPLAY_GATE";
    }
    return result;
}

void append_reaction_trace(
    std::ostringstream& output, const ReactionTrace& value) {
    output << std::setprecision(17)
           << "{\"substeps_per_frame\":" << value.substeps_per_frame
           << ",\"status\":\"" << (value.passed ? "PASS" : "FAIL") << '"'
           << ",\"failure\":\"" << value.failure << '"'
           << ",\"inactive_certified\":"
           << (value.inactive_certified ? "true" : "false")
           << ",\"steps\":" << value.steps
           << ",\"inactive_steps\":" << value.inactive_steps
           << ",\"active_steps\":" << value.active_steps
           << ",\"contact_steps\":" << value.contact_steps
           << ",\"maximum_stationarity_defect\":"
           << value.maximum_stationarity_defect
           << ",\"maximum_translation_defect\":"
           << value.maximum_translation_defect
           << ",\"maximum_reconstruction_defect\":"
           << value.maximum_reconstruction_defect
           << ",\"maximum_contact_defect\":"
           << value.maximum_contact_defect
           << ",\"maximum_inactive_bound_ratio\":"
           << value.maximum_inactive_bound_ratio
           << ",\"maximum_active_mixed_ratio\":"
           << value.maximum_active_mixed_ratio
           << ",\"maximum_fp_bound\":" << value.maximum_fp_bound
           << ",\"cumulative_fp_bound\":" << value.cumulative_fp_bound
           << ",\"signed_cumulative_defect\":";
    append_vec3(output, value.signed_cumulative_defect);
    output << ",\"cumulative_l1_defect\":"
           << value.cumulative_l1_defect
           << ",\"direct_terminal_defect\":"
           << value.direct_terminal_defect
           << ",\"precontact_state_sha256\":\""
           << value.precontact.state_sha256 << '"'
           << ",\"contact_state_sha256\":\""
           << value.contact.state_sha256 << "\"}";
}

void append_reaction_replay(
    std::ostringstream& output, const ReactionReplay& value) {
    output << std::setprecision(17)
           << "{\"name\":\"" << value.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL") << '"'
           << ",\"failure\":\"" << value.failure << '"'
           << ",\"state_sha256\":\"" << value.state_sha256 << '"'
           << ",\"ordinary_outer_trials\":"
           << value.ordinary_outer_trials
           << ",\"ordinary_hvp_calls\":" << value.ordinary_hvp_calls
           << ",\"reaction_outer_trials\":"
           << value.reaction_outer_trials
           << ",\"reaction_hvp_calls\":" << value.reaction_hvp_calls
           << ",\"ordinary_stationarity_defect\":"
           << value.ordinary_stationarity_defect
           << ",\"reaction_stationarity_defect\":"
           << value.reaction_stationarity_defect
           << ",\"reaction_mixed_limit\":"
           << value.reaction_mixed_limit
           << ",\"position_change_dx\":" << value.position_change_dx
           << ",\"velocity_change_c\":" << value.velocity_change_c
           << ",\"reaction_stop\":\"" << value.reaction_stop << "\"}";
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

bool owned_smoke_accounting_valid(const SmokeCase& value) {
    const double momentum_budget = static_cast<double>(
        value.fixture.position.size()) * MASS * std::sqrt(KAPPA / MASS);
    bool references_valid = true;
    for (const SmokeRun& run : value.reference.levels) {
        references_valid = references_valid
            && run.passed
            && run.floor_merit_accepts > 0
            && run.floor_merit_trials == run.floor_merit_accepts
            && run.maximum_floor_accepts_per_solve <= 4
            && run.floor_conditions_exact
            && run.maximum_active_mixed_ratio <= 1.0
            && run.maximum_inactive_bound_ratio <= 1.0
            && run.cumulative_fp_bound <= 1.0e-10 * momentum_budget;
    }
    return value.passed
        && value.controller.floor_merit_accepts > 0
        && value.controller.floor_merit_trials
            == value.controller.floor_merit_accepts
        && value.controller.maximum_floor_accepts_per_solve <= 4
        && value.controller.floor_conditions_exact
        && value.controller.maximum_active_mixed_ratio <= 1.0
        && value.controller.maximum_inactive_bound_ratio <= 1.0
        && references_valid;
}

void append_smoke_run_owned_accounting(
    std::ostringstream& output, const SmokeRun& value) {
    output << std::setprecision(17)
           << "{\"substeps\":" << value.substeps
           << ",\"active_steps\":" << value.active_steps
           << ",\"inactive_steps\":" << value.inactive_steps
           << ",\"outer_trials\":" << value.outer_trials
           << ",\"rejected_trials\":" << value.rejected_trials
           << ",\"hvp_calls\":" << value.hvp_calls
           << ",\"floor_merit_trials\":" << value.floor_merit_trials
           << ",\"floor_merit_accepts\":" << value.floor_merit_accepts
           << ",\"maximum_floor_accepts_per_solve\":"
           << value.maximum_floor_accepts_per_solve
           << ",\"floor_conditions_exact\":"
           << (value.floor_conditions_exact ? "true" : "false")
           << ",\"maximum_active_mixed_ratio\":"
           << value.maximum_active_mixed_ratio
           << ",\"maximum_inactive_bound_ratio\":"
           << value.maximum_inactive_bound_ratio
           << ",\"cumulative_fp_bound\":"
           << value.cumulative_fp_bound << '}';
}

void append_smoke_case_owned_accounting(
    std::ostringstream& output, const SmokeCase& value) {
    output << std::setprecision(17)
           << "{\"name\":\"" << value.fixture.name << '\"'
           << ",\"valid\":"
           << (owned_smoke_accounting_valid(value) ? "true" : "false")
           << ",\"controller\":{\"active_steps\":"
           << value.controller.active_steps
           << ",\"inactive_steps\":" << value.controller.inactive_steps
           << ",\"outer_trials\":" << value.controller.outer_trials
           << ",\"rejected_trials\":"
           << value.controller.rejected_trials
           << ",\"hvp_calls\":"
           << value.controller.nonlinear_hvp_calls
           << ",\"floor_merit_trials\":"
           << value.controller.floor_merit_trials
           << ",\"floor_merit_accepts\":"
           << value.controller.floor_merit_accepts
           << ",\"maximum_floor_accepts_per_solve\":"
           << value.controller.maximum_floor_accepts_per_solve
           << ",\"floor_conditions_exact\":"
           << (value.controller.floor_conditions_exact ? "true" : "false")
           << ",\"maximum_active_mixed_ratio\":"
           << value.controller.maximum_active_mixed_ratio
           << ",\"maximum_inactive_bound_ratio\":"
           << value.controller.maximum_inactive_bound_ratio
           << ",\"cumulative_fp_bound\":"
           << value.controller.cumulative_fp_bound
           << "},\"reference\":[";
    for (std::size_t i = 0; i < value.reference.levels.size(); ++i) {
        if (i != 0) {
            output << ',';
        }
        append_smoke_run_owned_accounting(
            output, value.reference.levels[i]);
    }
    output << "]}";
}

struct DisplacementLevel {
    bool passed = false;
    int substeps_per_frame = 0;
    ReactionTrace baseline;
    ReactionTrace candidate;
    double final_position_difference_dx = 0.0;
    double final_velocity_difference_c = 0.0;
};

DisplacementLevel displacement_level(
    const SmokeFixture& fixture, int substeps_per_frame) {
    DisplacementLevel result;
    result.substeps_per_frame = substeps_per_frame;
    result.baseline = run_reaction_trace(fixture, substeps_per_frame);
    result.candidate = run_reaction_trace(
        fixture, substeps_per_frame, true, true);
    if (result.baseline.steps == result.candidate.steps
        && result.baseline.steps
            == SMOKE_FRAMES * substeps_per_frame) {
        result.final_position_difference_dx = rms_difference(
            result.baseline.final_position,
            result.candidate.final_position) / SPACING;
        result.final_velocity_difference_c = rms_difference(
            result.baseline.final_velocity,
            result.candidate.final_velocity)
            / std::sqrt(KAPPA / MASS);
    } else {
        result.final_position_difference_dx =
            std::numeric_limits<double>::infinity();
        result.final_velocity_difference_c =
            std::numeric_limits<double>::infinity();
    }
    result.passed = result.baseline.passed && result.candidate.passed
        && result.candidate.inactive_certified
        && result.candidate.maximum_active_mixed_ratio <= 1.0
        && result.final_position_difference_dx <= 1.0e-5
        && result.final_velocity_difference_c <= 1.0e-5;
    return result;
}

void append_displacement_level(
    std::ostringstream& output, const DisplacementLevel& value) {
    output << std::setprecision(17)
           << "{\"substeps_per_frame\":" << value.substeps_per_frame
           << ",\"status\":\"" << (value.passed ? "PASS" : "FAIL") << '"'
           << ",\"final_position_difference_dx\":"
           << value.final_position_difference_dx
           << ",\"final_velocity_difference_c\":"
           << value.final_velocity_difference_c
           << ",\"baseline\":";
    append_reaction_trace(output, value.baseline);
    output << ",\"candidate\":";
    append_reaction_trace(output, value.candidate);
    output << ",\"candidate_failure_diagnostic\":{\"reaction_defect\":"
           << value.candidate.failure_reaction_defect
           << ",\"reaction_limit\":"
           << value.candidate.failure_reaction_limit
           << ",\"predicted_reduction\":"
           << value.candidate.failure_predicted_reduction
           << ",\"energy_floor\":"
           << value.candidate.failure_energy_floor
           << ",\"scaled_residual\":"
           << value.candidate.failure_scaled_residual << '}';
    output << '}';
}

struct DifferenceProbe {
    bool passed = false;
    std::string name;
    std::string failure;
    std::string state_sha256;
    int substeps_per_frame = 0;
    int failing_substep = 0;
    double predicted_reduction = 0.0;
    double inherited_energy_floor = 0.0;
    double correction_norm_dx = 0.0;
    int changed_position_components = 0;
    int nonzero_correction_components = 0;
    double minimum_step_ulp_ratio = std::numeric_limits<double>::infinity();
    double maximum_step_ulp_ratio = 0.0;
    int current_active_centers = 0;
    int trial_active_centers = 0;
    std::size_t current_pairs = 0;
    std::size_t trial_pairs = 0;
    bool topology_exact = false;
    double maximum_density_change = 0.0;
    double maximum_compression_change = 0.0;
    double legacy_total_difference = 0.0;
    double factored_potential_difference = 0.0;
    double factored_inertia_difference = 0.0;
    double factored_total_difference = 0.0;
    long double extended_total_difference = 0.0L;
    double factored_error_bound = 0.0;
    bool factored_correspondence = false;
    bool positive_sign_certified = false;
    double current_reaction_defect = 0.0;
    double current_reaction_limit = 0.0;
    double trial_reaction_defect = 0.0;
    double trial_reaction_limit = 0.0;
    double residual_reduction_ratio = 0.0;
    bool trial_reaction_converged = false;
};

double compression_from_density(double density) {
    return std::max(density / REST_DENSITY - 1.0, 0.0);
}

Vec3 owned_reaction_defect(
    const std::vector<Vec3>& displacement,
    const std::vector<Vec3>& predicted_displacement,
    const Evaluation& support,
    double time_step) {
    Vec3 actual;
    for (std::size_t i = 0; i < displacement.size(); ++i) {
        actual += MASS / time_step
            * (displacement[i] - predicted_displacement[i]);
    }
    const Vec3 fluid_gradient = sum_values(
        support.gradient, 0U, displacement.size());
    return actual + time_step * fluid_gradient;
}

double owned_reaction_limit(
    const std::vector<Vec3>& displacement,
    const std::vector<Vec3>& predicted_displacement,
    const Evaluation& support,
    const std::vector<Vec3>& velocity,
    Vec3 gravity,
    double time_step) {
    Vec3 actual;
    for (std::size_t i = 0; i < displacement.size(); ++i) {
        actual += MASS / time_step
            * (displacement[i] - predicted_displacement[i]);
    }
    const Vec3 model = -time_step * sum_values(
        support.gradient, 0U, displacement.size());
    const double scale = std::max({
        norm(actual) + norm(model),
        static_cast<double>(displacement.size()) * MASS * time_step
            * norm(gravity),
        1.0e-12,
    });
    return 1.0e-9 * scale
        + displacement_forward_bound(velocity, gravity, time_step);
}

DifferenceProbe difference_probe(
    const std::string& name,
    const SmokeFixture& fixture,
    const ReactionTrace& trace) {
    DifferenceProbe result;
    result.name = name;
    result.substeps_per_frame = trace.substeps_per_frame;
    result.failing_substep = trace.steps;
    if (!trace.energy_floor.valid) {
        result.failure = "MISSING_ENERGY_FLOOR_CAPTURE";
        return result;
    }
    const ReactionCapture& capture = trace.energy_floor;
    const SmoothSolve& smooth = capture.ordinary.smooth;
    result.state_sha256 = capture.state_sha256;
    result.predicted_reduction = smooth.last_predicted_reduction;
    result.inherited_energy_floor = smooth.numerical_energy_floor;
    result.current_reaction_defect = smooth.reaction_stationarity_defect;
    result.current_reaction_limit = smooth.reaction_mixed_limit;
    if (smooth.displacement.size() != capture.position.size()
        || smooth.last_step.size() != capture.position.size()) {
        result.failure = "CAPTURE_SHAPE";
        return result;
    }

    std::vector<Vec3> predicted_displacement(capture.position.size());
    std::vector<Vec3> trial_displacement = smooth.displacement;
    std::vector<Vec3> trial_position(capture.position.size());
    for (std::size_t i = 0; i < capture.position.size(); ++i) {
        predicted_displacement[i] = capture.time_step
            * (capture.velocity[i]
                + capture.time_step * fixture.gravity);
        trial_displacement[i] += smooth.last_step[i];
        trial_position[i] = capture.position[i] + trial_displacement[i];
        for (int axis = 0; axis < 3; ++axis) {
            const double p = component(smooth.last_step[i], axis);
            if (p == 0.0) {
                continue;
            }
            ++result.nonzero_correction_components;
            const double old_value = component(smooth.position[i], axis);
            const double new_value = component(trial_position[i], axis);
            result.changed_position_components += old_value != new_value
                ? 1 : 0;
            double ulp = std::nextafter(old_value,
                std::numeric_limits<double>::infinity()) - old_value;
            if (!(ulp > 0.0) || !std::isfinite(ulp)) {
                ulp = std::numeric_limits<double>::denorm_min();
            }
            const double ratio = std::abs(p) / ulp;
            result.minimum_step_ulp_ratio = std::min(
                result.minimum_step_ulp_ratio, ratio);
            result.maximum_step_ulp_ratio = std::max(
                result.maximum_step_ulp_ratio, ratio);
        }
    }
    if (result.nonzero_correction_components == 0) {
        result.minimum_step_ulp_ratio = 0.0;
    }
    result.correction_norm_dx = vector_norm(smooth.last_step) / SPACING;

    std::vector<Vec3> y_star(capture.position.size());
    for (std::size_t i = 0; i < capture.position.size(); ++i) {
        y_star[i] = capture.position[i] + predicted_displacement[i];
    }
    const SmoothEvaluation current = smooth_evaluate(
        smooth.position, y_star, fixture.boundary, capture.time_step);
    const SmoothEvaluation trial = smooth_evaluate(
        trial_position, y_star, fixture.boundary, capture.time_step);
    result.legacy_total_difference = current.total - trial.total;
    result.current_active_centers = static_cast<int>(
        current.support.active_centers);
    result.trial_active_centers = static_cast<int>(
        trial.support.active_centers);
    result.current_pairs = current.support.fluid_pairs
        + current.support.boundary_pairs;
    result.trial_pairs = trial.support.fluid_pairs
        + trial.support.boundary_pairs;
    result.topology_exact = result.current_active_centers
            == result.trial_active_centers
        && result.current_pairs == result.trial_pairs;

    const double inertia_scale = MASS
        / (capture.time_step * capture.time_step);
    double absolute_term_sum = 0.0;
    double local_error_sum = 0.0;
    long double extended = 0.0L;
    for (std::size_t i = 0; i < capture.position.size(); ++i) {
        const double c0 = compression_from_density(
            current.support.density[i]);
        const double c1 = compression_from_density(
            trial.support.density[i]);
        result.maximum_density_change = std::max(
            result.maximum_density_change,
            std::abs(current.support.density[i]
                - trial.support.density[i]));
        result.maximum_compression_change = std::max(
            result.maximum_compression_change, std::abs(c0 - c1));
        const double term = 0.5 * KAPPA * (c0 - c1) * (c0 + c1);
        result.factored_potential_difference += term;
        const double magnitude = 0.5 * std::abs(KAPPA)
            * (std::abs(c0) + std::abs(c1))
            * (std::abs(c0) + std::abs(c1));
        absolute_term_sum += std::abs(term);
        local_error_sum += gamma_factor(4U) * magnitude;
        extended += 0.5L * static_cast<long double>(KAPPA)
            * (static_cast<long double>(c0) - static_cast<long double>(c1))
            * (static_cast<long double>(c0) + static_cast<long double>(c1));

        for (int axis = 0; axis < 3; ++axis) {
            const double p = component(smooth.last_step[i], axis);
            const double d = component(smooth.displacement[i], axis)
                - component(predicted_displacement[i], axis);
            const double inertia_term = -0.5 * inertia_scale * p
                * (2.0 * d + p);
            result.factored_inertia_difference += inertia_term;
            const double inertia_magnitude = 0.5 * inertia_scale
                * std::abs(p) * (2.0 * std::abs(d) + std::abs(p));
            absolute_term_sum += std::abs(inertia_term);
            local_error_sum += gamma_factor(5U) * inertia_magnitude;
            extended += -0.5L * static_cast<long double>(inertia_scale)
                * static_cast<long double>(p)
                * (2.0L * static_cast<long double>(d)
                    + static_cast<long double>(p));
        }
    }
    result.factored_total_difference =
        result.factored_potential_difference
        + result.factored_inertia_difference;
    result.extended_total_difference = extended;
    const std::size_t terms = 4U * capture.position.size();
    result.factored_error_bound = local_error_sum
        + gamma_factor(std::max<std::size_t>(terms, 2U) - 1U)
            * absolute_term_sum;
    result.factored_correspondence = std::abs(
        static_cast<long double>(result.factored_total_difference)
            - result.extended_total_difference)
        <= static_cast<long double>(result.factored_error_bound);
    result.positive_sign_certified = result.factored_correspondence
        && result.factored_total_difference
            - result.factored_error_bound > 0.0
        && result.extended_total_difference > 0.0L;

    result.trial_reaction_defect = norm(owned_reaction_defect(
        trial_displacement, predicted_displacement,
        trial.support, capture.time_step));
    result.trial_reaction_limit = owned_reaction_limit(
        trial_displacement, predicted_displacement, trial.support,
        capture.velocity, fixture.gravity, capture.time_step);
    result.residual_reduction_ratio = result.trial_reaction_defect
        / std::max(result.current_reaction_defect, 1.0e-300);
    result.trial_reaction_converged = std::isfinite(
        result.trial_reaction_defect)
        && result.trial_reaction_defect <= result.trial_reaction_limit;
    result.passed = result.predicted_reduction > 0.0
        && result.predicted_reduction <= result.inherited_energy_floor
        && result.current_reaction_defect > result.current_reaction_limit
        && result.factored_correspondence
        && result.topology_exact
        && std::isfinite(result.residual_reduction_ratio);
    if (!result.passed) {
        result.failure = "PROBE_INVARIANT";
    }
    return result;
}

void append_difference_probe(
    std::ostringstream& output, const DifferenceProbe& value) {
    output << std::setprecision(17)
           << "{\"name\":\"" << value.name << '\"'
           << ",\"status\":\"" << (value.passed ? "PASS" : "FAIL") << '\"'
           << ",\"failure\":\"" << value.failure << '\"'
           << ",\"state_sha256\":\"" << value.state_sha256 << '\"'
           << ",\"substeps_per_frame\":" << value.substeps_per_frame
           << ",\"failing_substep\":" << value.failing_substep
           << ",\"predicted_reduction\":" << value.predicted_reduction
           << ",\"inherited_energy_floor\":"
           << value.inherited_energy_floor
           << ",\"representation\":{\"correction_norm_dx\":"
           << value.correction_norm_dx
           << ",\"nonzero_correction_components\":"
           << value.nonzero_correction_components
           << ",\"changed_position_components\":"
           << value.changed_position_components
           << ",\"minimum_step_ulp_ratio\":"
           << value.minimum_step_ulp_ratio
           << ",\"maximum_step_ulp_ratio\":"
           << value.maximum_step_ulp_ratio << '}'
           << ",\"topology\":{\"current_active_centers\":"
           << value.current_active_centers
           << ",\"trial_active_centers\":" << value.trial_active_centers
           << ",\"current_pairs\":" << value.current_pairs
           << ",\"trial_pairs\":" << value.trial_pairs
           << ",\"exact\":" << (value.topology_exact ? "true" : "false")
           << ",\"maximum_density_change\":"
           << value.maximum_density_change
           << ",\"maximum_compression_change\":"
           << value.maximum_compression_change << '}'
           << ",\"energy\":{\"legacy_total_difference\":"
           << value.legacy_total_difference
           << ",\"factored_potential_difference\":"
           << value.factored_potential_difference
           << ",\"factored_inertia_difference\":"
           << value.factored_inertia_difference
           << ",\"factored_total_difference\":"
           << value.factored_total_difference
           << ",\"extended_total_difference\":"
           << value.extended_total_difference
           << ",\"factored_error_bound\":"
           << value.factored_error_bound
           << ",\"correspondence\":"
           << (value.factored_correspondence ? "true" : "false")
           << ",\"positive_sign_certified\":"
           << (value.positive_sign_certified ? "true" : "false") << '}'
           << ",\"stationarity\":{\"current_defect\":"
           << value.current_reaction_defect
           << ",\"current_limit\":" << value.current_reaction_limit
           << ",\"trial_defect\":" << value.trial_reaction_defect
           << ",\"trial_limit\":" << value.trial_reaction_limit
           << ",\"reduction_ratio\":" << value.residual_reduction_ratio
           << ",\"trial_converged\":"
           << (value.trial_reaction_converged ? "true" : "false")
           << "}}";
}

struct StationarityTrajectoryLevel {
    bool passed = false;
    int substeps_per_frame = 0;
    ReactionTrace ordinary;
    ReactionTrace candidate;
    double final_position_difference_dx = 0.0;
    double final_velocity_difference_c = 0.0;
    int hvp_limit = 0;
    std::string final_state_sha256;
};

StationarityTrajectoryLevel stationarity_trajectory_level(
    const SmokeFixture& fixture, int substeps_per_frame) {
    StationarityTrajectoryLevel result;
    result.substeps_per_frame = substeps_per_frame;
    result.ordinary = run_reaction_trace(fixture, substeps_per_frame);
    result.candidate = run_reaction_trace(
        fixture, substeps_per_frame, true, true, true);
    result.hvp_limit = static_cast<int>(std::floor(
        2.5 * static_cast<double>(result.ordinary.hvp_calls)
            + 2.0 * static_cast<double>(result.candidate.active_steps)));
    if (result.ordinary.steps == result.candidate.steps
        && result.candidate.steps
            == SMOKE_FRAMES * substeps_per_frame) {
        result.final_position_difference_dx = rms_difference(
            result.ordinary.final_position,
            result.candidate.final_position) / SPACING;
        result.final_velocity_difference_c = rms_difference(
            result.ordinary.final_velocity,
            result.candidate.final_velocity)
            / std::sqrt(KAPPA / MASS);
        result.final_state_sha256 = hash_smoke_state(
            result.candidate.final_position,
            result.candidate.final_velocity);
    } else {
        result.final_position_difference_dx =
            std::numeric_limits<double>::infinity();
        result.final_velocity_difference_c =
            std::numeric_limits<double>::infinity();
    }
    result.passed = result.ordinary.passed && result.candidate.passed
        && result.candidate.inactive_certified
        && result.candidate.maximum_active_mixed_ratio <= 1.0
        && result.final_position_difference_dx <= 1.0e-5
        && result.final_velocity_difference_c <= 1.0e-5
        && result.candidate.floor_merit_trials > 0
        && result.candidate.floor_merit_trials
            == result.candidate.floor_merit_accepts
        && result.candidate.floor_conditions_exact
        && result.candidate.hvp_calls <= result.hvp_limit;
    return result;
}

void append_stationarity_trajectory_level(
    std::ostringstream& output,
    const StationarityTrajectoryLevel& value) {
    output << std::setprecision(17)
           << "{\"substeps_per_frame\":" << value.substeps_per_frame
           << ",\"status\":\"" << (value.passed ? "PASS" : "FAIL") << '\"'
           << ",\"final_position_difference_dx\":"
           << value.final_position_difference_dx
           << ",\"final_velocity_difference_c\":"
           << value.final_velocity_difference_c
           << ",\"final_state_sha256\":\""
           << value.final_state_sha256 << '\"'
           << ",\"ordinary_work\":{\"outer_trials\":"
           << value.ordinary.outer_trials
           << ",\"hvp_calls\":" << value.ordinary.hvp_calls
           << ",\"rejected_trials\":"
           << value.ordinary.rejected_trials << '}'
           << ",\"candidate_work\":{\"outer_trials\":"
           << value.candidate.outer_trials
           << ",\"hvp_calls\":" << value.candidate.hvp_calls
           << ",\"hvp_limit\":" << value.hvp_limit
           << ",\"rejected_trials\":"
           << value.candidate.rejected_trials
           << ",\"floor_merit_trials\":"
           << value.candidate.floor_merit_trials
           << ",\"floor_merit_accepts\":"
           << value.candidate.floor_merit_accepts
           << ",\"maximum_floor_residual_ratio\":"
           << value.candidate.maximum_floor_residual_ratio
           << ",\"floor_conditions_exact\":"
           << (value.candidate.floor_conditions_exact ? "true" : "false")
           << '}'
           << ",\"candidate_trace\":";
    append_reaction_trace(output, value.candidate);
    output << ",\"failure_diagnostic\":{\"floor_merit_trials\":"
           << value.candidate.failure_floor_merit_trials
           << ",\"floor_merit_accepts\":"
           << value.candidate.failure_floor_merit_accepts
           << ",\"negative_curvature_exits\":"
           << value.candidate.failure_negative_curvature_exits
           << ",\"current_defect\":"
           << value.candidate.failure_reaction_defect
           << ",\"current_limit\":"
           << value.candidate.failure_reaction_limit
           << ",\"predicted_reduction\":"
           << value.candidate.failure_predicted_reduction
           << ",\"energy_floor\":"
           << value.candidate.failure_energy_floor
           << ",\"trial_defect\":"
           << value.candidate.failure_floor_trial_defect
           << ",\"trial_limit\":"
           << value.candidate.failure_floor_trial_limit
           << ",\"residual_ratio\":"
           << value.candidate.failure_floor_residual_ratio
           << ",\"topology_exact\":"
           << (value.candidate.failure_floor_topology_exact
                   ? "true" : "false") << '}'
           << '}';
}

struct OwnedGradientProbe {
    bool passed = false;
    std::string name;
    std::string failure;
    std::string state_sha256;
    int substeps_per_frame = 0;
    int failing_substep = 0;
    Vec3 direct_residual;
    Vec3 legacy_gradient_residual;
    Vec3 owned_gradient_residual;
    Vec3 legacy_identity_difference;
    Vec3 owned_identity_difference;
    double direct_residual_norm = 0.0;
    double legacy_identity_error = 0.0;
    double owned_identity_error = 0.0;
    double owned_identity_bound = 0.0;
    bool owned_identity_certified = false;
    double captured_trial_defect = 0.0;
    double trust_radius_dx = 0.0;
    double owned_step_norm_dx = 0.0;
    int owned_hvp_calls = 0;
    bool owned_negative_curvature = false;
    bool topology_exact = false;
    double owned_trial_defect = 0.0;
    double owned_trial_limit = 0.0;
    double owned_trial_ratio = 0.0;
    bool owned_trial_converged = false;
};

OwnedGradientProbe owned_gradient_probe(
    const std::string& name,
    const SmokeFixture& fixture,
    const ReactionTrace& trace) {
    OwnedGradientProbe result;
    result.name = name;
    result.substeps_per_frame = trace.substeps_per_frame;
    result.failing_substep = trace.steps;
    if (!trace.energy_floor.valid) {
        result.failure = "MISSING_D3_FAILURE_CAPTURE";
        return result;
    }
    const ReactionCapture& capture = trace.energy_floor;
    const SmoothSolve& smooth = capture.ordinary.smooth;
    result.state_sha256 = capture.state_sha256;
    result.captured_trial_defect = trace.failure_floor_trial_defect;
    result.trust_radius_dx = smooth.last_trust_radius / SPACING;
    if (smooth.displacement.size() != capture.position.size()) {
        result.failure = "CAPTURE_SHAPE";
        return result;
    }
    std::vector<Vec3> predicted_displacement(capture.position.size());
    std::vector<Vec3> y_star(capture.position.size());
    for (std::size_t i = 0; i < capture.position.size(); ++i) {
        predicted_displacement[i] = capture.time_step
            * (capture.velocity[i]
                + capture.time_step * fixture.gravity);
        y_star[i] = capture.position[i] + predicted_displacement[i];
    }
    const SmoothEvaluation legacy = smooth_evaluate(
        smooth.position, y_star, fixture.boundary, capture.time_step);
    std::vector<Vec3> owned_gradient = fluid_part(
        legacy.support.gradient, capture.position.size());
    const double inertia_scale = MASS
        / (capture.time_step * capture.time_step);
    Vec3 owned_component_magnitude;
    for (std::size_t i = 0; i < owned_gradient.size(); ++i) {
        owned_gradient[i] += inertia_scale
            * (smooth.displacement[i] - predicted_displacement[i]);
        owned_component_magnitude.x += std::abs(owned_gradient[i].x);
        owned_component_magnitude.y += std::abs(owned_gradient[i].y);
        owned_component_magnitude.z += std::abs(owned_gradient[i].z);
    }
    result.direct_residual = owned_reaction_defect(
        smooth.displacement, predicted_displacement,
        legacy.support, capture.time_step);
    result.legacy_gradient_residual = capture.time_step
        * sum_values(legacy.gradient, 0U, legacy.gradient.size());
    result.owned_gradient_residual = capture.time_step
        * sum_values(owned_gradient, 0U, owned_gradient.size());
    result.legacy_identity_difference =
        result.legacy_gradient_residual - result.direct_residual;
    result.owned_identity_difference =
        result.owned_gradient_residual - result.direct_residual;
    result.direct_residual_norm = norm(result.direct_residual);
    result.legacy_identity_error = norm(result.legacy_identity_difference);
    result.owned_identity_error = norm(result.owned_identity_difference);
    const double sum_bound = capture.time_step
        * gamma_factor(std::max<std::size_t>(
            owned_gradient.size(), 2U) - 1U)
        * norm(owned_component_magnitude);
    double local_inertia_magnitude = 0.0;
    for (std::size_t i = 0; i < smooth.displacement.size(); ++i) {
        local_inertia_magnitude += MASS / capture.time_step
            * (norm(smooth.displacement[i])
                + norm(predicted_displacement[i]));
    }
    result.owned_identity_bound = displacement_forward_bound(
        capture.velocity, fixture.gravity, capture.time_step)
        + sum_bound + gamma_factor(6U) * local_inertia_magnitude;
    result.owned_identity_certified = result.owned_identity_error
        <= result.owned_identity_bound;

    const std::vector<Vec3> step = trust_step(
        smooth.position, fixture.boundary, owned_gradient,
        capture.time_step, smooth.last_trust_radius,
        result.owned_hvp_calls, result.owned_negative_curvature);
    result.owned_step_norm_dx = vector_norm(step) / SPACING;
    std::vector<Vec3> trial_displacement = add_scaled(
        smooth.displacement, step, 1.0);
    std::vector<Vec3> trial_position(capture.position.size());
    for (std::size_t i = 0; i < capture.position.size(); ++i) {
        trial_position[i] = capture.position[i] + trial_displacement[i];
    }
    const Evaluation trial_support = evaluate(
        trial_position, fixture.boundary);
    result.topology_exact = legacy.support.active_centers
            == trial_support.active_centers
        && legacy.support.fluid_pairs == trial_support.fluid_pairs
        && legacy.support.boundary_pairs == trial_support.boundary_pairs;
    result.owned_trial_defect = norm(owned_reaction_defect(
        trial_displacement, predicted_displacement,
        trial_support, capture.time_step));
    result.owned_trial_limit = owned_reaction_limit(
        trial_displacement, predicted_displacement, trial_support,
        capture.velocity, fixture.gravity, capture.time_step);
    result.owned_trial_ratio = result.owned_trial_defect
        / std::max(result.direct_residual_norm, 1.0e-300);
    result.owned_trial_converged = std::isfinite(result.owned_trial_defect)
        && result.owned_trial_defect <= result.owned_trial_limit;
    result.passed = result.owned_identity_certified
        && result.topology_exact
        && std::isfinite(result.legacy_identity_error)
        && std::isfinite(result.owned_trial_ratio)
        && result.owned_hvp_calls > 0;
    if (!result.passed) {
        result.failure = "OWNED_GRADIENT_PROBE";
    }
    return result;
}

void append_owned_gradient_probe(
    std::ostringstream& output, const OwnedGradientProbe& value) {
    output << std::setprecision(17)
           << "{\"name\":\"" << value.name << '\"'
           << ",\"status\":\"" << (value.passed ? "PASS" : "FAIL") << '\"'
           << ",\"failure\":\"" << value.failure << '\"'
           << ",\"state_sha256\":\"" << value.state_sha256 << '\"'
           << ",\"substeps_per_frame\":" << value.substeps_per_frame
           << ",\"failing_substep\":" << value.failing_substep
           << ",\"identity\":{\"direct_residual\":";
    append_vec3(output, value.direct_residual);
    output << ",\"legacy_gradient_residual\":";
    append_vec3(output, value.legacy_gradient_residual);
    output << ",\"owned_gradient_residual\":";
    append_vec3(output, value.owned_gradient_residual);
    output << ",\"legacy_difference\":";
    append_vec3(output, value.legacy_identity_difference);
    output << ",\"owned_difference\":";
    append_vec3(output, value.owned_identity_difference);
    output << ",\"direct_norm\":" << value.direct_residual_norm
           << ",\"legacy_error\":" << value.legacy_identity_error
           << ",\"owned_error\":" << value.owned_identity_error
           << ",\"owned_bound\":" << value.owned_identity_bound
           << ",\"owned_certified\":"
           << (value.owned_identity_certified ? "true" : "false") << '}'
           << ",\"trust_trial\":{\"captured_legacy_trial_defect\":"
           << value.captured_trial_defect
           << ",\"trust_radius_dx\":" << value.trust_radius_dx
           << ",\"step_norm_dx\":" << value.owned_step_norm_dx
           << ",\"hvp_calls\":" << value.owned_hvp_calls
           << ",\"negative_curvature\":"
           << (value.owned_negative_curvature ? "true" : "false")
           << ",\"topology_exact\":"
           << (value.topology_exact ? "true" : "false")
           << ",\"trial_defect\":" << value.owned_trial_defect
           << ",\"trial_limit\":" << value.owned_trial_limit
           << ",\"residual_ratio\":" << value.owned_trial_ratio
           << ",\"trial_converged\":"
           << (value.owned_trial_converged ? "true" : "false")
           << "}}";
}

struct OwnedResidualLevel {
    bool passed = false;
    int substeps_per_frame = 0;
    ReactionTrace ordinary;
    ReactionTrace candidate;
    double final_position_difference_dx = 0.0;
    double final_velocity_difference_c = 0.0;
    int hvp_limit = 0;
    std::string final_state_sha256;
};

OwnedResidualLevel owned_residual_level(
    const SmokeFixture& fixture, int substeps_per_frame) {
    OwnedResidualLevel result;
    result.substeps_per_frame = substeps_per_frame;
    result.ordinary = run_reaction_trace(fixture, substeps_per_frame);
    result.candidate = run_reaction_trace(
        fixture, substeps_per_frame, true, true, true, true);
    result.hvp_limit = static_cast<int>(std::floor(
        2.5 * static_cast<double>(result.ordinary.hvp_calls)
            + 4.0 * static_cast<double>(result.candidate.active_steps)));
    if (result.ordinary.steps == result.candidate.steps
        && result.candidate.steps
            == SMOKE_FRAMES * substeps_per_frame) {
        result.final_position_difference_dx = rms_difference(
            result.ordinary.final_position,
            result.candidate.final_position) / SPACING;
        result.final_velocity_difference_c = rms_difference(
            result.ordinary.final_velocity,
            result.candidate.final_velocity)
            / std::sqrt(KAPPA / MASS);
        result.final_state_sha256 = hash_smoke_state(
            result.candidate.final_position,
            result.candidate.final_velocity);
    } else {
        result.final_position_difference_dx =
            std::numeric_limits<double>::infinity();
        result.final_velocity_difference_c =
            std::numeric_limits<double>::infinity();
    }
    result.passed = result.ordinary.passed && result.candidate.passed
        && result.candidate.inactive_certified
        && result.candidate.maximum_active_mixed_ratio <= 1.0
        && result.final_position_difference_dx <= 1.0e-5
        && result.final_velocity_difference_c <= 1.0e-5
        && result.candidate.floor_merit_accepts > 0
        && result.candidate.floor_merit_trials
            == result.candidate.floor_merit_accepts
        && result.candidate.maximum_floor_accepts_per_solve <= 4
        && result.candidate.floor_conditions_exact
        && result.candidate.hvp_calls <= result.hvp_limit;
    return result;
}

void append_owned_residual_level(
    std::ostringstream& output, const OwnedResidualLevel& value) {
    output << std::setprecision(17)
           << "{\"substeps_per_frame\":" << value.substeps_per_frame
           << ",\"status\":\"" << (value.passed ? "PASS" : "FAIL") << '\"'
           << ",\"final_position_difference_dx\":"
           << value.final_position_difference_dx
           << ",\"final_velocity_difference_c\":"
           << value.final_velocity_difference_c
           << ",\"final_state_sha256\":\""
           << value.final_state_sha256 << '\"'
           << ",\"ordinary_work\":{\"outer_trials\":"
           << value.ordinary.outer_trials
           << ",\"hvp_calls\":" << value.ordinary.hvp_calls
           << ",\"rejected_trials\":"
           << value.ordinary.rejected_trials << '}'
           << ",\"candidate_work\":{\"outer_trials\":"
           << value.candidate.outer_trials
           << ",\"hvp_calls\":" << value.candidate.hvp_calls
           << ",\"hvp_limit\":" << value.hvp_limit
           << ",\"rejected_trials\":"
           << value.candidate.rejected_trials
           << ",\"floor_merit_trials\":"
           << value.candidate.floor_merit_trials
           << ",\"floor_merit_accepts\":"
           << value.candidate.floor_merit_accepts
           << ",\"maximum_floor_accepts_per_solve\":"
           << value.candidate.maximum_floor_accepts_per_solve
           << ",\"maximum_floor_residual_ratio\":"
           << value.candidate.maximum_floor_residual_ratio
           << ",\"floor_conditions_exact\":"
           << (value.candidate.floor_conditions_exact ? "true" : "false")
           << '}'
           << ",\"candidate_trace\":";
    append_reaction_trace(output, value.candidate);
    output << '}';
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

SplitBoundaryReport run_boundary_reaction_accuracy_controls() {
    const SmokeFixture face_fixture = make_face_smoke_fixture();
    const SmokeFixture corner_fixture = make_corner_smoke_fixture();
    const SmokeController face_adaptive =
        run_smoke_controller(face_fixture, false);
    const SmokeController corner_adaptive =
        run_smoke_controller(corner_fixture, false);
    constexpr std::array<int, 3> counts = {96, 192, 384};
    std::array<ReactionTrace, 3> face_traces;
    std::array<ReactionTrace, 3> corner_traces;
    bool traces_valid = true;
    for (std::size_t i = 0; i < counts.size(); ++i) {
        face_traces[i] = run_reaction_trace(face_fixture, counts[i]);
        corner_traces[i] = run_reaction_trace(corner_fixture, counts[i]);
        traces_valid = traces_valid
            && face_traces[i].passed && corner_traces[i].passed;
    }
    const ReactionReplay face_replay = replay_reaction_aware(
        "face-precontact-192", face_fixture, face_traces[1].precontact);
    const ReactionReplay corner_replay = replay_reaction_aware(
        "corner-contact-192", corner_fixture, corner_traces[1].contact);
    const SplitBoundaryReport parent =
        run_boundary_composition_smoke_controls();
    const bool parent_exact = !parent.passed
        && sha256_hex(parent.json)
            == "1d7372c7bcf4cbf3eddc4a6f82842e520535170b8182c9c66f95e486a539fe63";
    const bool diagnostic_valid = parent_exact
        && face_adaptive.passed && corner_adaptive.passed
        && traces_valid;
    bool inactive_certified = true;
    for (std::size_t i = 0; i < counts.size(); ++i) {
        inactive_certified = inactive_certified
            && face_traces[i].inactive_certified
            && corner_traces[i].inactive_certified;
    }
    const bool candidate_selected = diagnostic_valid && inactive_certified
        && face_replay.passed && corner_replay.passed;
    std::string first_failure;
    if (!parent_exact) {
        first_failure = "NSR3B3D_PARENT_REPLAY";
    } else if (!face_adaptive.passed) {
        first_failure = "NSR3B3D_FACE_ADAPTIVE:" + face_adaptive.failure;
    } else if (!corner_adaptive.passed) {
        first_failure = "NSR3B3D_CORNER_ADAPTIVE:"
            + corner_adaptive.failure;
    } else if (!traces_valid) {
        first_failure = "NSR3B3D_TRACE";
    }
    const std::string disposition = candidate_selected
        ? "REACTION_AWARE_STOP_CANDIDATE"
        : !inactive_certified
            ? "INACTIVE_FORWARD_ERROR_CERTIFICATE_REJECTED"
            : "REACTION_AUTHORITY_TOO_EXPENSIVE_OR_UNRESOLVED";

    std::ostringstream material;
    material << std::setprecision(17)
             << (diagnostic_valid ? "PASS|" : "FAIL|")
             << first_failure << '|' << disposition << '|'
             << face_replay.passed << ':'
             << face_replay.reaction_stationarity_defect << ':'
             << face_replay.reaction_hvp_calls << '|'
             << corner_replay.passed << ':'
             << corner_replay.reaction_stationarity_defect << ':'
             << corner_replay.reaction_hvp_calls;
    for (const ReactionTrace& trace : face_traces) {
        material << '|' << trace.substeps_per_frame << ':'
                 << trace.maximum_inactive_bound_ratio << ':'
                 << trace.maximum_active_mixed_ratio << ':'
                 << trace.direct_terminal_defect;
    }
    for (const ReactionTrace& trace : corner_traces) {
        material << '|' << trace.substeps_per_frame << ':'
                 << trace.maximum_inactive_bound_ratio << ':'
                 << trace.maximum_active_mixed_ratio << ':'
                 << trace.direct_terminal_defect;
    }

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b3d_reaction_accuracy.v1\""
           << ",\"identity\":\"boundary-reaction-accuracy-diagnostic-r0\""
           << ",\"parent_b3_result_sha256\":\"9e0eb0baf63c6bf1ae0dd5288cd8809c1722ddf354a1fff86f3d70935f2a56fe\""
           << ",\"parent_b3_raw_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"status\":\""
           << (diagnostic_valid ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"disposition\":\"" << disposition << '"'
           << ",\"adaptive_nonaborting\":{\"face_status\":\""
           << (face_adaptive.passed ? "PASS" : "FAIL") << '"'
           << ",\"face_maximum_ledger_residual\":"
           << face_adaptive.maximum_ledger_residual
           << ",\"corner_status\":\""
           << (corner_adaptive.passed ? "PASS" : "FAIL") << '"'
           << ",\"corner_maximum_ledger_residual\":"
           << corner_adaptive.maximum_ledger_residual << '}'
           << ",\"fixed_traces\":{\"face\":[";
    for (std::size_t i = 0; i < face_traces.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_reaction_trace(report, face_traces[i]);
    }
    report << "],\"corner\":[";
    for (std::size_t i = 0; i < corner_traces.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_reaction_trace(report, corner_traces[i]);
    }
    report << "]},\"reaction_aware_replays\":[";
    append_reaction_replay(report, face_replay);
    report << ',';
    append_reaction_replay(report, corner_replay);
    report << "]"
           << ",\"candidate_selected\":"
           << (candidate_selected ? "true" : "false")
           << ",\"b3_retry_authorized\":"
           << (candidate_selected ? "true" : "false")
           << ",\"physical_corpus_execution_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(material.str()) << "\"}";
    return {diagnostic_valid, report.str()};
}

SplitBoundaryReport run_displacement_ownership_controls() {
    const SplitBoundaryReport parent =
        run_boundary_reaction_accuracy_controls();
    const bool parent_exact = parent.passed
        && sha256_hex(parent.json)
            == "d514d3085585c58ab61aa40c03d9b96c5d56c44bdcf40c6e0b72817f6ea1c16e";
    const SmokeFixture face_fixture = make_face_smoke_fixture();
    const SmokeFixture corner_fixture = make_corner_smoke_fixture();
    constexpr std::array<int, 3> counts = {96, 192, 384};
    std::array<DisplacementLevel, 3> face;
    std::array<DisplacementLevel, 3> corner;
    bool levels_passed = true;
    for (std::size_t i = 0; i < counts.size(); ++i) {
        face[i] = displacement_level(face_fixture, counts[i]);
        corner[i] = displacement_level(corner_fixture, counts[i]);
        levels_passed = levels_passed
            && face[i].passed && corner[i].passed;
    }
    const ReactionReplay face_replay = replay_reaction_aware(
        "face-precontact-192-owned", face_fixture,
        face[1].baseline.precontact, true);
    const ReactionReplay corner_replay = replay_reaction_aware(
        "corner-contact-192-owned", corner_fixture,
        corner[1].baseline.contact, true);
    const bool storage_valid = 24U * face_fixture.position.size()
            == sizeof(Vec3) * face_fixture.position.size()
        && 24U * corner_fixture.position.size()
            == sizeof(Vec3) * corner_fixture.position.size();
    const bool passed = parent_exact && levels_passed
        && face_replay.passed && corner_replay.passed
        && storage_valid;
    std::string first_failure;
    if (!parent_exact) {
        first_failure = "NSR3B3D1_PARENT";
    } else if (!levels_passed) {
        first_failure = "NSR3B3D1_DISPLACEMENT_CERTIFICATE";
    } else if (!face_replay.passed) {
        first_failure = "NSR3B3D1_FACE_REACTION";
    } else if (!corner_replay.passed) {
        first_failure = "NSR3B3D1_CORNER_REACTION";
    } else if (!storage_valid) {
        first_failure = "NSR3B3D1_STORAGE";
    }
    const std::string disposition = passed
        ? "DISPLACEMENT_OWNED_REACTION_CANDIDATE"
        : !levels_passed
            ? "DISPLACEMENT_CERTIFICATE_REJECTED"
            : "REACTION_AWARE_DISPLACEMENT_REJECTED";

    std::ostringstream material;
    material << std::setprecision(17)
             << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
             << disposition << '|' << face_replay.passed << ':'
             << face_replay.reaction_stationarity_defect << '|'
             << corner_replay.passed << ':'
             << corner_replay.reaction_stationarity_defect;
    for (const DisplacementLevel& value : face) {
        material << '|' << value.substeps_per_frame << ':' << value.passed
                 << ':' << value.candidate.cumulative_fp_bound << ':'
                 << value.final_position_difference_dx << ':'
                 << value.final_velocity_difference_c;
    }
    for (const DisplacementLevel& value : corner) {
        material << '|' << value.substeps_per_frame << ':' << value.passed
                 << ':' << value.candidate.cumulative_fp_bound << ':'
                 << value.final_position_difference_dx << ':'
                 << value.final_velocity_difference_c;
    }

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b3d1_displacement.v1\""
           << ",\"identity\":\"owned-substep-displacement-r0\""
           << ",\"parent_b3d_result_sha256\":\"e99cda02f945e8402e6f4ca30742853c1fa29f1c845ed5c8a24db180877fa3b1\""
           << ",\"parent_b3d_raw_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"disposition\":\"" << disposition << '"'
           << ",\"transient_storage\":{\"bytes_per_fluid_sample\":24"
           << ",\"face_bytes\":"
           << 24U * face_fixture.position.size()
           << ",\"corner_bytes\":"
           << 24U * corner_fixture.position.size()
           << ",\"valid\":" << (storage_valid ? "true" : "false") << '}'
           << ",\"levels\":{\"face\":[";
    for (std::size_t i = 0; i < face.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_displacement_level(report, face[i]);
    }
    report << "],\"corner\":[";
    for (std::size_t i = 0; i < corner.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_displacement_level(report, corner[i]);
    }
    report << "]},\"reaction_aware_replays\":[";
    append_reaction_replay(report, face_replay);
    report << ',';
    append_reaction_replay(report, corner_replay);
    report << "]"
           << ",\"candidate_selected\":" << (passed ? "true" : "false")
           << ",\"b3r_design_authorized\":" << (passed ? "true" : "false")
           << ",\"runtime_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(material.str()) << "\"}";
    return {passed, report.str()};
}

SplitBoundaryReport run_finite_precision_merit_controls() {
    const SplitBoundaryReport parent = run_displacement_ownership_controls();
    const bool parent_exact = !parent.passed
        && sha256_hex(parent.json)
            == "6dede55270ca21c583ba83f5eebb740b0c0adf0e4b2a61cd00eb941906e4d259";
    const SmokeFixture face_fixture = make_face_smoke_fixture();
    const SmokeFixture corner_fixture = make_corner_smoke_fixture();
    constexpr std::array<int, 3> counts = {96, 192, 384};
    std::array<DifferenceProbe, 6> probes;
    for (std::size_t i = 0; i < counts.size(); ++i) {
        const ReactionTrace trace = run_reaction_trace(
            face_fixture, counts[i], true, true);
        probes[i] = difference_probe(
            "face-" + std::to_string(counts[i]), face_fixture, trace);
    }
    for (std::size_t i = 0; i < counts.size(); ++i) {
        const ReactionTrace trace = run_reaction_trace(
            corner_fixture, counts[i], true, true);
        probes[i + counts.size()] = difference_probe(
            "corner-" + std::to_string(counts[i]), corner_fixture, trace);
    }

    bool probes_valid = true;
    bool all_factored = true;
    bool all_stationarity = true;
    bool any_unmaterialized = false;
    for (const DifferenceProbe& probe : probes) {
        probes_valid = probes_valid && probe.passed;
        all_factored = all_factored && probe.positive_sign_certified;
        all_stationarity = all_stationarity
            && probe.topology_exact
            && probe.trial_reaction_defect < probe.current_reaction_defect
            && probe.trial_reaction_converged;
        any_unmaterialized = any_unmaterialized
            || probe.changed_position_components
                < probe.nonzero_correction_components
            || (probe.correction_norm_dx > 0.0
                && probe.maximum_density_change == 0.0);
    }
    const std::string classification = all_factored
        ? "FACTORED_OBJECTIVE_DIFFERENCE_CANDIDATE"
        : all_stationarity
            ? "FLOOR_STATIONARITY_MERIT_CANDIDATE"
            : any_unmaterialized
                ? "LOCAL_GEOMETRY_OWNERSHIP_REQUIRED"
                : "NUMERICAL_GLOBALIZATION_STOP";
    const bool passed = parent_exact && probes_valid;
    std::string first_failure;
    if (!parent_exact) {
        first_failure = "NSR3B3D2_PARENT";
    } else if (!probes_valid) {
        first_failure = "NSR3B3D2_PROBE_INVARIANT";
    }

    std::ostringstream material;
    material << std::setprecision(17)
             << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
             << classification;
    for (const DifferenceProbe& probe : probes) {
        material << '|' << probe.name << ':'
                 << probe.state_sha256 << ':'
                 << probe.factored_total_difference << ':'
                 << probe.factored_error_bound << ':'
                 << probe.positive_sign_certified << ':'
                 << probe.changed_position_components << ':'
                 << probe.nonzero_correction_components << ':'
                 << probe.trial_reaction_defect << ':'
                 << probe.trial_reaction_limit;
    }

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b3d2_merit.v1\""
           << ",\"identity\":\"finite-precision-merit-discriminator-r0\""
           << ",\"parent_b3d1_result_sha256\":\"f9ed6a83170eff4c965b8c573eac0dd6f308beb41a6f7fcc365f8ad2b11a2ec7\""
           << ",\"parent_b3d1_raw_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '\"'
           << ",\"first_failure\":\"" << first_failure << '\"'
           << ",\"classification\":\"" << classification << '\"'
           << ",\"probes\":[";
    for (std::size_t i = 0; i < probes.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_difference_probe(report, probes[i]);
    }
    report << ']'
           << ",\"factored_candidate_selected\":"
           << (all_factored && passed ? "true" : "false")
           << ",\"stationarity_candidate_selected\":"
           << (!all_factored && all_stationarity && passed
                   ? "true" : "false")
           << ",\"b3_retry_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(material.str()) << "\"}";
    return {passed, report.str()};
}

SplitBoundaryReport run_floor_stationarity_trajectory_controls() {
    const SplitBoundaryReport parent = run_finite_precision_merit_controls();
    const bool parent_exact = parent.passed
        && sha256_hex(parent.json)
            == "573bf5a943d338bbbed4c05919bea2a0255b6d71e98d85c195a3ee38adf82f6e";
    const SmokeFixture face_fixture = make_face_smoke_fixture();
    const SmokeFixture corner_fixture = make_corner_smoke_fixture();
    constexpr std::array<int, 3> counts = {96, 192, 384};
    std::array<StationarityTrajectoryLevel, 3> face;
    std::array<StationarityTrajectoryLevel, 3> corner;
    bool levels_passed = true;
    for (std::size_t i = 0; i < counts.size(); ++i) {
        face[i] = stationarity_trajectory_level(face_fixture, counts[i]);
        corner[i] = stationarity_trajectory_level(
            corner_fixture, counts[i]);
        levels_passed = levels_passed
            && face[i].passed && corner[i].passed;
    }
    const bool passed = parent_exact && levels_passed;
    std::string first_failure;
    if (!parent_exact) {
        first_failure = "NSR3B3D3_PARENT";
    } else if (!levels_passed) {
        first_failure = "NSR3B3D3_TRAJECTORY_GATE";
    }
    const std::string disposition = passed
        ? "FLOOR_STATIONARITY_TRAJECTORY_CANDIDATE"
        : "FLOOR_STATIONARITY_TRAJECTORY_REJECTED";

    std::ostringstream material;
    material << std::setprecision(17)
             << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
             << disposition;
    for (const StationarityTrajectoryLevel& value : face) {
        material << "|F:" << value.substeps_per_frame << ':'
                 << value.passed << ':'
                 << value.candidate.floor_merit_accepts << ':'
                 << value.candidate.hvp_calls << ':'
                 << value.final_position_difference_dx << ':'
                 << value.final_velocity_difference_c << ':'
                 << value.final_state_sha256;
    }
    for (const StationarityTrajectoryLevel& value : corner) {
        material << "|C:" << value.substeps_per_frame << ':'
                 << value.passed << ':'
                 << value.candidate.floor_merit_accepts << ':'
                 << value.candidate.hvp_calls << ':'
                 << value.final_position_difference_dx << ':'
                 << value.final_velocity_difference_c << ':'
                 << value.final_state_sha256;
    }

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b3d3_trajectory.v1\""
           << ",\"identity\":\"floor-stationarity-trajectory-r0\""
           << ",\"parent_b3d2_result_sha256\":\"da1da7a0fcd4adf29c01a60a5d72f97c4d7a4c9794484bc90f575b9d271d4601\""
           << ",\"parent_b3d2_raw_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '\"'
           << ",\"first_failure\":\"" << first_failure << '\"'
           << ",\"disposition\":\"" << disposition << '\"'
           << ",\"levels\":{\"face\":[";
    for (std::size_t i = 0; i < face.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_stationarity_trajectory_level(report, face[i]);
    }
    report << "],\"corner\":[";
    for (std::size_t i = 0; i < corner.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_stationarity_trajectory_level(report, corner[i]);
    }
    report << "]}"
           << ",\"candidate_selected\":"
           << (passed ? "true" : "false")
           << ",\"b3r_design_authorized\":"
           << (passed ? "true" : "false")
           << ",\"physical_corpus_execution_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(material.str()) << "\"}";
    return {passed, report.str()};
}

SplitBoundaryReport run_owned_gradient_controls() {
    const SplitBoundaryReport parent =
        run_floor_stationarity_trajectory_controls();
    const bool parent_exact = !parent.passed
        && sha256_hex(parent.json)
            == "65737fcabbe8a7d4e926e38bd39b0c4cef20c933b07c19505195641a8a3d1719";
    const SmokeFixture face_fixture = make_face_smoke_fixture();
    const SmokeFixture corner_fixture = make_corner_smoke_fixture();
    constexpr std::array<int, 3> counts = {96, 192, 384};
    std::array<OwnedGradientProbe, 6> probes;
    for (std::size_t i = 0; i < counts.size(); ++i) {
        const ReactionTrace trace = run_reaction_trace(
            face_fixture, counts[i], true, true, true);
        probes[i] = owned_gradient_probe(
            "face-" + std::to_string(counts[i]), face_fixture, trace);
    }
    for (std::size_t i = 0; i < counts.size(); ++i) {
        const ReactionTrace trace = run_reaction_trace(
            corner_fixture, counts[i], true, true, true);
        probes[i + counts.size()] = owned_gradient_probe(
            "corner-" + std::to_string(counts[i]), corner_fixture, trace);
    }
    bool probes_valid = true;
    bool all_converged = true;
    bool all_improved = true;
    for (const OwnedGradientProbe& probe : probes) {
        probes_valid = probes_valid && probe.passed;
        all_converged = all_converged
            && probe.owned_trial_converged;
        all_improved = all_improved
            && probe.owned_trial_defect < probe.direct_residual_norm;
    }
    const std::string classification = !probes_valid
        ? "OWNED_GRADIENT_IDENTITY_REJECTED"
        : all_converged
            ? "OWNED_INERTIA_GRADIENT_CANDIDATE"
            : all_improved
                ? "BOUNDED_OWNED_RESIDUAL_ITERATION_REQUIRED"
                : "OWNED_RESIDUAL_LINE_SEARCH_REQUIRED";
    const bool passed = parent_exact && probes_valid;
    std::string first_failure;
    if (!parent_exact) {
        first_failure = "NSR3B3D4_PARENT";
    } else if (!probes_valid) {
        first_failure = "NSR3B3D4_IDENTITY";
    }
    std::ostringstream material;
    material << std::setprecision(17)
             << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
             << classification;
    for (const OwnedGradientProbe& probe : probes) {
        material << '|' << probe.name << ':' << probe.state_sha256 << ':'
                 << probe.legacy_identity_error << ':'
                 << probe.owned_identity_error << ':'
                 << probe.owned_identity_bound << ':'
                 << probe.owned_trial_defect << ':'
                 << probe.owned_trial_limit << ':'
                 << probe.owned_hvp_calls;
    }
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b3d4_gradient.v1\""
           << ",\"identity\":\"displacement-owned-inertia-gradient-r0\""
           << ",\"parent_b3d3_result_sha256\":\"a550a2e9cd6783bd74a022c612236e47542a1e229f2ec3e4432b7663ef3a0071\""
           << ",\"parent_b3d3_raw_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '\"'
           << ",\"first_failure\":\"" << first_failure << '\"'
           << ",\"classification\":\"" << classification << '\"'
           << ",\"probes\":[";
    for (std::size_t i = 0; i < probes.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_owned_gradient_probe(report, probes[i]);
    }
    report << ']'
           << ",\"candidate_selected\":"
           << (passed && classification
                   == "OWNED_INERTIA_GRADIENT_CANDIDATE"
                   ? "true" : "false")
           << ",\"b3_retry_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(material.str()) << "\"}";
    return {passed, report.str()};
}

SplitBoundaryReport run_owned_residual_trajectory_controls() {
    const SplitBoundaryReport parent = run_owned_gradient_controls();
    const bool parent_exact = parent.passed
        && sha256_hex(parent.json)
            == "36f91802d114ba87124d00303c6ac413fbc09d4cd792f4262636e9816a5680ca";
    const SmokeFixture face_fixture = make_face_smoke_fixture();
    const SmokeFixture corner_fixture = make_corner_smoke_fixture();
    constexpr std::array<int, 3> counts = {96, 192, 384};
    std::array<OwnedResidualLevel, 3> face;
    std::array<OwnedResidualLevel, 3> corner;
    bool levels_passed = true;
    for (std::size_t i = 0; i < counts.size(); ++i) {
        face[i] = owned_residual_level(face_fixture, counts[i]);
        corner[i] = owned_residual_level(corner_fixture, counts[i]);
        levels_passed = levels_passed
            && face[i].passed && corner[i].passed;
    }
    const bool passed = parent_exact && levels_passed;
    std::string first_failure;
    if (!parent_exact) {
        first_failure = "NSR3B3D5_PARENT";
    } else if (!levels_passed) {
        first_failure = "NSR3B3D5_TRAJECTORY_GATE";
    }
    const std::string disposition = passed
        ? "OWNED_RESIDUAL_TRAJECTORY_CANDIDATE"
        : "OWNED_RESIDUAL_TRAJECTORY_REJECTED";
    std::ostringstream material;
    material << std::setprecision(17)
             << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
             << disposition;
    for (const OwnedResidualLevel& value : face) {
        material << "|F:" << value.substeps_per_frame << ':'
                 << value.passed << ':'
                 << value.candidate.floor_merit_accepts << ':'
                 << value.candidate.maximum_floor_accepts_per_solve << ':'
                 << value.candidate.hvp_calls << ':'
                 << value.final_position_difference_dx << ':'
                 << value.final_velocity_difference_c << ':'
                 << value.final_state_sha256;
    }
    for (const OwnedResidualLevel& value : corner) {
        material << "|C:" << value.substeps_per_frame << ':'
                 << value.passed << ':'
                 << value.candidate.floor_merit_accepts << ':'
                 << value.candidate.maximum_floor_accepts_per_solve << ':'
                 << value.candidate.hvp_calls << ':'
                 << value.final_position_difference_dx << ':'
                 << value.final_velocity_difference_c << ':'
                 << value.final_state_sha256;
    }
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b3d5_trajectory.v1\""
           << ",\"identity\":\"owned-residual-trajectory-r0\""
           << ",\"parent_b3d4_result_sha256\":\"bdf5b7eaa1f215e14ebb05500ffa2495b8cb3fccd19aa4b369514464086aae34\""
           << ",\"parent_b3d4_raw_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '\"'
           << ",\"first_failure\":\"" << first_failure << '\"'
           << ",\"disposition\":\"" << disposition << '\"'
           << ",\"levels\":{\"face\":[";
    for (std::size_t i = 0; i < face.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_owned_residual_level(report, face[i]);
    }
    report << "],\"corner\":[";
    for (std::size_t i = 0; i < corner.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_owned_residual_level(report, corner[i]);
    }
    report << "]}"
           << ",\"candidate_selected\":"
           << (passed ? "true" : "false")
           << ",\"b3r_design_authorized\":"
           << (passed ? "true" : "false")
           << ",\"physical_corpus_execution_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(material.str()) << "\"}";
    return {passed, report.str()};
}

SplitBoundaryReport run_owned_boundary_composition_controls() {
    const SplitBoundaryReport parent =
        run_owned_residual_trajectory_controls();
    const bool parent_exact = parent.passed
        && sha256_hex(parent.json)
            == "fd7627b22734b6be1183e0bbd53f03a99170bda35d2f37585c4da066516c2b91";
    const SmokeCase face = run_smoke_case(
        make_face_smoke_fixture(), true);
    const SmokeCase corner = run_smoke_case(
        make_corner_smoke_fixture(), true);
    const bool face_accounting = owned_smoke_accounting_valid(face);
    const bool corner_accounting = owned_smoke_accounting_valid(corner);
    const bool passed = parent_exact && face.passed && corner.passed
        && face_accounting && corner_accounting;
    std::string first_failure;
    if (!parent_exact) {
        first_failure = "NSR3B3R_PARENT";
    } else if (!face.passed) {
        first_failure = "NSR3B3R_FACE:" + face.failure;
    } else if (!corner.passed) {
        first_failure = "NSR3B3R_CORNER:" + corner.failure;
    } else if (!face_accounting) {
        first_failure = "NSR3B3R_FACE_ACCOUNTING";
    } else if (!corner_accounting) {
        first_failure = "NSR3B3R_CORNER_ACCOUNTING";
    }
    const std::string disposition = passed
        ? "STATIC_BOUNDARY_SMOKE_CANDIDATE"
        : "OWNED_BOUNDARY_COMPOSITION_REJECTED";
    std::ostringstream material;
    material << std::setprecision(17)
             << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
             << disposition << '|'
             << face.controller.accepted_substeps << ':'
             << face.controller.executed_substeps << ':'
             << face.controller.nonlinear_hvp_calls << ':'
             << face.controller.floor_merit_accepts << ':'
             << face.controller.maximum_ledger_residual << ':'
             << face.final_position_error_dx << ':'
             << face.final_velocity_error_c << '|'
             << corner.controller.accepted_substeps << ':'
             << corner.controller.executed_substeps << ':'
             << corner.controller.nonlinear_hvp_calls << ':'
             << corner.controller.floor_merit_accepts << ':'
             << corner.controller.maximum_ledger_residual << ':'
             << corner.final_position_error_dx << ':'
             << corner.final_velocity_error_c;
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b3r_boundary.v1\""
           << ",\"identity\":\"post-solve-swept-contact-composition-r1-owned-residual\""
           << ",\"parent_b3d5_result_sha256\":\"c716675fd75c4c7ecaa2410eb2f26154d7c31f36264a1d41bd9ad5b3f0aee40c\""
           << ",\"parent_b3d5_raw_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '\"'
           << ",\"first_failure\":\"" << first_failure << '\"'
           << ",\"disposition\":\"" << disposition << '\"'
           << ",\"cases\":[";
    append_smoke_case(report, face);
    report << ',';
    append_smoke_case(report, corner);
    report << "] ,\"owned_accounting\":[";
    append_smoke_case_owned_accounting(report, face);
    report << ',';
    append_smoke_case_owned_accounting(report, corner);
    report << ']'
           << ",\"candidate_selected\":"
           << (passed ? "true" : "false")
           << ",\"b4_contract_design_authorized\":"
           << (passed ? "true" : "false")
           << ",\"physical_corpus_execution_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(material.str()) << "\"}";
    return {passed, report.str()};
}

namespace {

constexpr std::array<int, 3> B4A_BOX_CELLS = {6, 6, 6};
constexpr Vec3 B4A_BOX_MIN{0.0, 0.0, 0.0};
constexpr Vec3 B4A_BOX_MAX{0.3, 0.3, 0.3};

std::vector<Vec3> make_box_owned_shell(
    std::array<int, 3> cells, int layers) {
    if (layers <= 0) {
        throw std::invalid_argument("B4A shell layers must be positive");
    }
    std::vector<Vec3> result;
    for (int z = -layers; z < cells[2] + layers; ++z) {
        for (int y = -layers; y < cells[1] + layers; ++y) {
            for (int x = -layers; x < cells[0] + layers; ++x) {
                if (!inside_cells(x, y, z, cells)) {
                    result.push_back(lattice_position(x, y, z));
                }
            }
        }
    }
    return result;
}

std::vector<Vec3> make_lattice_fluid(std::array<int, 3> cells) {
    std::vector<Vec3> result;
    for (int z = 0; z < cells[2]; ++z) {
        for (int y = 0; y < cells[1]; ++y) {
            for (int x = 0; x < cells[0]; ++x) {
                result.push_back(lattice_position(x, y, z));
            }
        }
    }
    return result;
}

bool point_inside_open_box(Vec3 value, Vec3 low, Vec3 high) {
    return value.x > low.x && value.x < high.x
        && value.y > low.y && value.y < high.y
        && value.z > low.z && value.z < high.z;
}

bool contains_exact(const std::vector<Vec3>& values, Vec3 candidate) {
    return std::any_of(values.begin(), values.end(),
        [candidate](Vec3 value) {
            return value.x == candidate.x
                && value.y == candidate.y
                && value.z == candidate.z;
        });
}

struct B4ALayerControl {
    bool passed = false;
    std::size_t two_layer_count = 0;
    std::size_t three_layer_count = 0;
    double density_error = 0.0;
    double energy_error = 0.0;
    double gradient_error = 0.0;
    double hvp_error = 0.0;
    double reaction_error = 0.0;
};

B4ALayerControl b4a_layer_control() {
    Fixture two;
    two.name = "b4a-filled-box-two-layer";
    two.cells = B4A_BOX_CELLS;
    two.fluid = make_lattice_fluid(B4A_BOX_CELLS);
    two.boundary = make_box_owned_shell(B4A_BOX_CELLS, 2);
    Fixture three = two;
    three.name = "b4a-filled-box-three-layer";
    three.boundary = make_box_owned_shell(B4A_BOX_CELLS, 3);

    const std::vector<Vec3> compressed = compressed_fluid(two, 0.99);
    const Evaluation value_two = evaluate(compressed, two.boundary);
    const Evaluation value_three = evaluate(compressed, three.boundary);
    const std::vector<Vec3> direction =
        deterministic_direction(compressed.size());
    std::vector<Vec3> direction_two(
        compressed.size() + two.boundary.size());
    std::vector<Vec3> direction_three(
        compressed.size() + three.boundary.size());
    std::copy(direction.begin(), direction.end(), direction_two.begin());
    std::copy(direction.begin(), direction.end(), direction_three.begin());
    const std::vector<Vec3> hvp_two = fluid_part(
        apply_hessian(compressed, two.boundary, direction_two),
        compressed.size());
    const std::vector<Vec3> hvp_three = fluid_part(
        apply_hessian(compressed, three.boundary, direction_three),
        compressed.size());

    B4ALayerControl result;
    result.two_layer_count = two.boundary.size();
    result.three_layer_count = three.boundary.size();
    for (std::size_t i = 0; i < value_two.density.size(); ++i) {
        result.density_error = std::max(result.density_error,
            relative_error(value_two.density[i], value_three.density[i]));
    }
    result.energy_error = relative_error(
        value_two.energy, value_three.energy);
    result.gradient_error = vector_relative_error(
        fluid_part(value_two.gradient, compressed.size()),
        fluid_part(value_three.gradient, compressed.size()));
    result.hvp_error = vector_relative_error(hvp_two, hvp_three);
    const Vec3 reaction_two = sum_values(value_two.gradient,
        compressed.size(), value_two.gradient.size());
    const Vec3 reaction_three = sum_values(value_three.gradient,
        compressed.size(), value_three.gradient.size());
    result.reaction_error = norm(reaction_two - reaction_three)
        / std::max({norm(reaction_two), norm(reaction_three), 1.0e-30});
    result.passed = result.two_layer_count == 784U
        && result.three_layer_count == 1512U
        && value_two.active_centers > 0U
        && value_three.active_centers == value_two.active_centers
        && result.density_error <= LAYER_LIMIT
        && result.energy_error <= LAYER_LIMIT
        && result.gradient_error <= LAYER_LIMIT
        && result.hvp_error <= LAYER_LIMIT
        && result.reaction_error <= LAYER_LIMIT;
    return result;
}

struct B4AFreeSurfaceControl {
    bool passed = false;
    std::size_t fluid_count = 0;
    std::size_t support_count = 0;
    std::size_t missing_air_count = 0;
    std::size_t support_inside_box = 0;
    std::size_t missing_air_as_support = 0;
    std::size_t active_centers = 0;
    double maximum_density_ratio = 0.0;
    double bottom_density_error = 0.0;
    double minimum_top_density_ratio = std::numeric_limits<double>::infinity();
};

B4AFreeSurfaceControl b4a_free_surface_control() {
    constexpr std::array<int, 3> fluid_cells = {6, 3, 6};
    const std::vector<Vec3> fluid = make_lattice_fluid(fluid_cells);
    const std::vector<Vec3> support =
        make_box_owned_shell(B4A_BOX_CELLS, 2);
    const Evaluation value = evaluate(fluid, support);

    B4AFreeSurfaceControl result;
    result.fluid_count = fluid.size();
    result.support_count = support.size();
    result.active_centers = value.active_centers;
    for (Vec3 boundary : support) {
        result.support_inside_box += point_inside_open_box(
            boundary, B4A_BOX_MIN, B4A_BOX_MAX) ? 1U : 0U;
    }
    for (int z = 0; z < B4A_BOX_CELLS[2]; ++z) {
        for (int y = fluid_cells[1]; y < B4A_BOX_CELLS[1]; ++y) {
            for (int x = 0; x < B4A_BOX_CELLS[0]; ++x) {
                ++result.missing_air_count;
                result.missing_air_as_support += contains_exact(
                    support, lattice_position(x, y, z)) ? 1U : 0U;
            }
        }
    }
    for (std::size_t i = 0; i < value.density.size(); ++i) {
        const std::size_t y = (i / static_cast<std::size_t>(fluid_cells[0]))
            % static_cast<std::size_t>(fluid_cells[1]);
        const double ratio = value.density[i] / REST_DENSITY;
        result.maximum_density_ratio = std::max(
            result.maximum_density_ratio, ratio);
        if (y == 0U) {
            result.bottom_density_error = std::max(
                result.bottom_density_error, std::abs(ratio - 1.0));
        }
        if (y == 2U) {
            result.minimum_top_density_ratio = std::min(
                result.minimum_top_density_ratio, ratio);
        }
    }
    result.passed = result.fluid_count == 108U
        && result.support_count == 784U
        && result.missing_air_count == 108U
        && result.support_inside_box == 0U
        && result.missing_air_as_support == 0U
        && result.active_centers == 0U
        && result.maximum_density_ratio <= 1.0 + 1.0e-12
        && result.bottom_density_error <= 1.0e-12
        && result.minimum_top_density_ratio < 1.0;
    return result;
}

SweepResult sweep_box_owned(
    Vec3 start, Vec3 smooth_displacement, Vec3 low, Vec3 high,
    double time_step) {
    SweepResult result;
    result.displacement = smooth_displacement;
    const Vec3 tentative = start + smooth_displacement;
    const Vec3 incoming_velocity = smooth_displacement / time_step;
    constexpr std::array<int, 3> low_feature = {0, 2, 4};
    constexpr std::array<int, 3> high_feature = {1, 3, 5};
    for (int axis = 0; axis < 3; ++axis) {
        const double start_value = component(start, axis);
        const double displacement = component(smooth_displacement, axis);
        const double tentative_value = component(tentative, axis);
        const double low_value = component(low, axis);
        const double high_value = component(high, axis);
        if (tentative_value < low_value && displacement < 0.0) {
            const double toi = (low_value - start_value) / displacement;
            if (!std::isfinite(toi) || toi < 0.0 || toi > 1.0) {
                throw std::runtime_error("invalid B4A lower box TOI");
            }
            set_component(result.displacement, axis,
                low_value - start_value);
            result.features.push_back(
                low_feature[static_cast<std::size_t>(axis)]);
            result.earliest_time_of_impact = std::min(
                result.earliest_time_of_impact, toi);
        } else if (tentative_value > high_value && displacement > 0.0) {
            const double toi = (high_value - start_value) / displacement;
            if (!std::isfinite(toi) || toi < 0.0 || toi > 1.0) {
                throw std::runtime_error("invalid B4A upper box TOI");
            }
            set_component(result.displacement, axis,
                high_value - start_value);
            result.features.push_back(
                high_feature[static_cast<std::size_t>(axis)]);
            result.earliest_time_of_impact = std::min(
                result.earliest_time_of_impact, toi);
        }
    }
    std::sort(result.features.begin(), result.features.end());
    result.position = start + result.displacement;
    result.velocity = result.displacement / time_step;
    result.fluid_impulse = MASS * (result.velocity - incoming_velocity);
    result.reaction = -result.fluid_impulse;
    for (int axis = 0; axis < 3; ++axis) {
        result.maximum_penetration = std::max(result.maximum_penetration,
            std::max(component(low, axis) - component(result.position, axis),
                component(result.position, axis) - component(high, axis)));
    }
    result.maximum_penetration = std::max(result.maximum_penetration, 0.0);
    return result;
}

struct B4AContactCase {
    std::string name;
    bool passed = false;
    std::vector<int> expected;
    SweepResult sweep;
};

B4AContactCase b4a_contact_case(
    std::string name, Vec3 start, Vec3 displacement,
    std::vector<int> expected) {
    B4AContactCase result;
    result.name = std::move(name);
    result.expected = std::move(expected);
    const Vec3 low{RADIUS, RADIUS, RADIUS};
    const Vec3 high{
        B4A_BOX_MAX.x - RADIUS,
        B4A_BOX_MAX.y - RADIUS,
        B4A_BOX_MAX.z - RADIUS,
    };
    result.sweep = sweep_box_owned(
        start, displacement, low, high, TIME_STEP);
    const bool active = !result.expected.empty();
    result.passed = result.sweep.features == result.expected
        && result.sweep.maximum_penetration <= 1.0e-12
        && norm(result.sweep.fluid_impulse + result.sweep.reaction)
            <= 1.0e-12
        && finite(result.sweep.position)
        && finite(result.sweep.velocity)
        && (!active
            || (result.sweep.earliest_time_of_impact >= 0.0
                && result.sweep.earliest_time_of_impact <= 1.0));
    return result;
}

std::array<B4AContactCase, 11> b4a_contact_controls() {
    return {
        b4a_contact_case("x-minus", {0.075, 0.15, 0.15}, {-0.1, 0.0, 0.0}, {0}),
        b4a_contact_case("x-plus", {0.225, 0.15, 0.15}, {0.1, 0.0, 0.0}, {1}),
        b4a_contact_case("y-minus", {0.15, 0.075, 0.15}, {0.0, -0.1, 0.0}, {2}),
        b4a_contact_case("y-plus", {0.15, 0.225, 0.15}, {0.0, 0.1, 0.0}, {3}),
        b4a_contact_case("z-minus", {0.15, 0.15, 0.075}, {0.0, 0.0, -0.1}, {4}),
        b4a_contact_case("z-plus", {0.15, 0.15, 0.225}, {0.0, 0.0, 0.1}, {5}),
        b4a_contact_case("lower-corner", {0.075, 0.075, 0.075}, {-0.1, -0.1, -0.1}, {0, 2, 4}),
        b4a_contact_case("upper-corner", {0.225, 0.225, 0.225}, {0.1, 0.1, 0.1}, {1, 3, 5}),
        b4a_contact_case("exact-graze", {0.1, RADIUS, 0.1}, {0.05, 0.0, 0.0}, {}),
        b4a_contact_case("moving-away", {RADIUS, 0.15, 0.15}, {0.05, 0.0, 0.0}, {}),
        b4a_contact_case("complete-box-crossing", {0.15, 0.15, 0.15}, {0.4, 0.0, 0.0}, {1}),
    };
}

struct B4ACostProjection {
    std::string name;
    std::uint64_t fluid = 0;
    std::array<std::uint64_t, 3> box_cells{};
    std::uint64_t support = 0;
    std::uint64_t checks = 0;
    std::uint64_t expected_support = 0;
    std::uint64_t expected_checks = 0;
    bool passed = false;
};

std::uint64_t checked_product(std::array<std::uint64_t, 3> values) {
    return values[0] * values[1] * values[2];
}

B4ACostProjection b4a_cost_projection(
    std::string name, std::uint64_t fluid,
    std::array<std::uint64_t, 3> cells,
    std::uint64_t expected_support,
    std::uint64_t expected_checks) {
    B4ACostProjection result;
    result.name = std::move(name);
    result.fluid = fluid;
    result.box_cells = cells;
    result.support = checked_product({
        cells[0] + 4U, cells[1] + 4U, cells[2] + 4U})
        - checked_product(cells);
    result.checks = fluid * (fluid - 1U) / 2U
        + fluid * result.support;
    result.expected_support = expected_support;
    result.expected_checks = expected_checks;
    result.passed = result.support == expected_support
        && result.checks == expected_checks;
    return result;
}

void append_b4a_contact(
    std::ostringstream& output, const B4AContactCase& value) {
    output << std::setprecision(17)
           << "{\"name\":\"" << value.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL") << "\",\"features\":[";
    for (std::size_t i = 0; i < value.sweep.features.size(); ++i) {
        if (i != 0U) {
            output << ',';
        }
        output << value.sweep.features[i];
    }
    output << "],\"expected\":[";
    for (std::size_t i = 0; i < value.expected.size(); ++i) {
        if (i != 0U) {
            output << ',';
        }
        output << value.expected[i];
    }
    output << "],\"time_of_impact\":"
           << value.sweep.earliest_time_of_impact
           << ",\"maximum_penetration_m\":"
           << value.sweep.maximum_penetration
           << ",\"impulse_closure\":"
           << norm(value.sweep.fluid_impulse + value.sweep.reaction) << '}';
}

void append_b4a_cost(
    std::ostringstream& output, const B4ACostProjection& value) {
    output << "{\"name\":\"" << value.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL") << "\",\"fluid\":"
           << value.fluid << ",\"box_cells\":["
           << value.box_cells[0] << ',' << value.box_cells[1] << ','
           << value.box_cells[2] << "],\"outer_support\":"
           << value.support << ",\"candidate_checks_per_evaluation\":"
           << value.checks << '}';
}

} // namespace

SplitBoundaryReport run_closed_box_eligibility_controls() {
    const SplitBoundaryReport parent =
        run_owned_boundary_composition_controls();
    const bool parent_exact = parent.passed
        && sha256_hex(parent.json)
            == "792dceb540d7998d62f9c290b4e4215832ec9e1a6ba6ecf9e16b3200f1e30da9";
    const B4ALayerControl layer = b4a_layer_control();
    const B4AFreeSurfaceControl free_surface =
        b4a_free_surface_control();
    const std::array<B4AContactCase, 11> contacts =
        b4a_contact_controls();
    const std::array<B4ACostProjection, 4> costs = {
        b4a_cost_projection("hydro", 6000U, {20U, 20U, 20U},
            5824U, 52941000U),
        b4a_cost_projection("dam-break", 6000U, {80U, 20U, 20U},
            16384U, 116301000U),
        b4a_cost_projection("orifice-outer-only", 6000U,
            {40U, 20U, 20U}, 9344U, 74061000U),
        b4a_cost_projection("sealed-product", 48000U,
            {80U, 20U, 40U}, 24704U, 2337768000U),
    };
    const bool contacts_passed = std::all_of(
        contacts.begin(), contacts.end(),
        [](const B4AContactCase& value) { return value.passed; });
    const bool costs_passed = std::all_of(
        costs.begin(), costs.end(),
        [](const B4ACostProjection& value) { return value.passed; });
    const bool passed = parent_exact && layer.passed
        && free_surface.passed && contacts_passed && costs_passed;
    std::string first_failure;
    if (!parent_exact) {
        first_failure = "NSR3B4A_PARENT";
    } else if (!layer.passed) {
        first_failure = "NSR3B4A_LAYER_TOPOLOGY";
    } else if (!free_surface.passed) {
        first_failure = "NSR3B4A_FREE_SURFACE_SEPARATION";
    } else if (!contacts_passed) {
        first_failure = "NSR3B4A_BOX_CONTACT";
    } else if (!costs_passed) {
        first_failure = "NSR3B4A_COST_PROJECTION";
    }
    const std::string disposition = passed
        ? "CLOSED_BOX_FREE_SURFACE_ELIGIBLE"
        : "CLOSED_BOX_FREE_SURFACE_REJECTED";
    std::ostringstream material;
    material << std::setprecision(17)
             << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
             << disposition << '|'
             << layer.two_layer_count << ':' << layer.three_layer_count
             << ':' << layer.density_error << ':' << layer.energy_error
             << ':' << layer.gradient_error << ':' << layer.hvp_error
             << ':' << layer.reaction_error << '|'
             << free_surface.fluid_count << ':'
             << free_surface.support_count << ':'
             << free_surface.missing_air_count << ':'
             << free_surface.maximum_density_ratio << ':'
             << free_surface.bottom_density_error << ':'
             << free_surface.minimum_top_density_ratio;
    for (const B4AContactCase& value : contacts) {
        material << "|C:" << value.name << ':' << value.passed << ':'
                 << value.sweep.earliest_time_of_impact;
    }
    for (const B4ACostProjection& value : costs) {
        material << "|K:" << value.name << ':' << value.support << ':'
                 << value.checks;
    }

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4a_eligibility.v1\""
           << ",\"identity\":\"free-surface-closed-box-eligibility-r0\""
           << ",\"parent_b3r_result_sha256\":\"2310c531dc7ab8883e83ab015742ce439d7923c21687eef9fac8f6b0014a0d10\""
           << ",\"parent_b3r_raw_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"disposition\":\"" << disposition << '"'
           << ",\"terms\":{\"incompressibility\":true,\"viscosity\":false,\"surface_tension\":false}"
           << ",\"layer_control\":{\"status\":\""
           << (layer.passed ? "PASS" : "FAIL")
           << "\",\"two_layer_support\":" << layer.two_layer_count
           << ",\"three_layer_support\":" << layer.three_layer_count
           << ",\"density_error\":" << layer.density_error
           << ",\"energy_error\":" << layer.energy_error
           << ",\"gradient_error\":" << layer.gradient_error
           << ",\"hvp_error\":" << layer.hvp_error
           << ",\"reaction_error\":" << layer.reaction_error << '}'
           << ",\"free_surface_control\":{\"status\":\""
           << (free_surface.passed ? "PASS" : "FAIL")
           << "\",\"fluid_samples\":" << free_surface.fluid_count
           << ",\"support_samples\":" << free_surface.support_count
           << ",\"missing_air_samples\":"
           << free_surface.missing_air_count
           << ",\"support_inside_box\":"
           << free_surface.support_inside_box
           << ",\"missing_air_as_support\":"
           << free_surface.missing_air_as_support
           << ",\"active_pressure_centers\":"
           << free_surface.active_centers
           << ",\"maximum_density_ratio\":"
           << free_surface.maximum_density_ratio
           << ",\"bottom_density_error\":"
           << free_surface.bottom_density_error
           << ",\"minimum_top_density_ratio\":"
           << free_surface.minimum_top_density_ratio << '}'
           << ",\"contact_controls\":[";
    for (std::size_t i = 0; i < contacts.size(); ++i) {
        if (i != 0U) {
            report << ',';
        }
        append_b4a_contact(report, contacts[i]);
    }
    report << "],\"nominal_cost_projection\":[";
    for (std::size_t i = 0; i < costs.size(); ++i) {
        if (i != 0U) {
            report << ',';
        }
        append_b4a_cost(report, costs[i]);
    }
    report << ']'
           << ",\"nominal_runner_requirement\":\"JOINT_CELL_NEIGHBORHOOD_REQUIRED_BEFORE_NOMINAL\""
           << ",\"candidate_selected\":"
           << (passed ? "true" : "false")
           << ",\"b4b_contract_design_authorized\":"
           << (passed ? "true" : "false")
           << ",\"physical_trajectory_execution_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(material.str()) << "\"}";
    return {passed, report.str()};
}

namespace {

constexpr std::array<int, 3> B4B_REFERENCE_COUNTS = {48, 96, 192};

SmokeFixture make_b4b_supported_column_fixture() {
    SmokeFixture result;
    result.name = "b4b-supported-column-startup";
    result.position = make_lattice_fluid({4, 3, 4});
    result.velocity.resize(result.position.size());
    result.boundary = make_box_owned_shell({4, 6, 4}, 2);
    result.closed_box_contact = true;
    result.contact_low = {RADIUS, RADIUS, RADIUS};
    result.contact_high = {0.2 - RADIUS, 0.3 - RADIUS, 0.2 - RADIUS};
    result.macro_frames = 8;
    result.maximum_participants = 1280U;
    result.maximum_pairs = 160U * result.position.size();
    result.geometry_sha256 = geometry_hash(result);
    return result;
}

SmokeFixture make_b4b_released_block_fixture() {
    SmokeFixture result;
    result.name = "b4b-released-block-floor-impact";
    constexpr std::array<double, 3> horizontal = {0.125, 0.175, 0.225};
    constexpr std::array<double, 3> vertical = {0.035, 0.085, 0.135};
    for (double z : horizontal) {
        for (double y : vertical) {
            for (double x : horizontal) {
                result.position.push_back({x, y, z});
            }
        }
    }
    result.velocity.resize(result.position.size());
    result.boundary = make_box_owned_shell({8, 8, 8}, 2);
    result.closed_box_contact = true;
    result.contact_low = {RADIUS, RADIUS, RADIUS};
    result.contact_high = {0.4 - RADIUS, 0.4 - RADIUS, 0.4 - RADIUS};
    result.macro_frames = 16;
    result.maximum_participants = 1280U;
    result.maximum_pairs = 160U * result.position.size();
    result.geometry_sha256 = geometry_hash(result);
    return result;
}

struct B4BAggregate {
    Vec3 center;
    Vec3 momentum_value;
    double q99_height = 0.0;
    double q99_front = 0.0;
    double kinetic = 0.0;
    double pressure = 0.0;
    double gravitational = 0.0;
    double mechanical = 0.0;
    double maximum_density_ratio = 0.0;
    double maximum_positive_strain = 0.0;
    double maximum_speed_value = 0.0;
    bool finite_values = false;
};

double nearest_rank_q99(std::vector<double> values) {
    std::sort(values.begin(), values.end());
    const std::size_t rank = static_cast<std::size_t>(
        std::ceil(0.99 * static_cast<double>(values.size())));
    return values[std::max<std::size_t>(rank, 1U) - 1U];
}

B4BAggregate b4b_aggregate(
    const SmokeFixture& fixture,
    const std::vector<Vec3>& position,
    const std::vector<Vec3>& velocity) {
    B4BAggregate result;
    result.center = average_values(position);
    result.momentum_value = momentum(velocity);
    result.kinetic = kinetic_energy(velocity);
    const Evaluation state = evaluate(position, fixture.boundary);
    result.pressure = state.energy;
    std::vector<double> heights;
    std::vector<double> fronts;
    heights.reserve(position.size());
    fronts.reserve(position.size());
    for (std::size_t i = 0; i < position.size(); ++i) {
        heights.push_back(position[i].y);
        fronts.push_back(position[i].x);
        result.gravitational += MASS * (-fixture.gravity.y) * position[i].y;
        result.maximum_speed_value = std::max(
            result.maximum_speed_value, norm(velocity[i]));
        const double ratio = state.density[i] / REST_DENSITY;
        result.maximum_density_ratio = std::max(
            result.maximum_density_ratio, ratio);
        result.maximum_positive_strain = std::max(
            result.maximum_positive_strain, ratio - 1.0);
    }
    result.q99_height = nearest_rank_q99(std::move(heights));
    result.q99_front = nearest_rank_q99(std::move(fronts));
    result.mechanical = result.kinetic + result.pressure
        + result.gravitational;
    result.finite_values = finite(result.center)
        && finite(result.momentum_value)
        && std::isfinite(result.q99_height)
        && std::isfinite(result.q99_front)
        && std::isfinite(result.kinetic)
        && std::isfinite(result.pressure)
        && std::isfinite(result.gravitational)
        && std::isfinite(result.mechanical)
        && std::isfinite(result.maximum_density_ratio)
        && std::isfinite(result.maximum_speed_value);
    return result;
}

struct B4BFixedTrajectory {
    bool passed = false;
    int substeps_per_frame = 0;
    std::string failure;
    std::vector<SmokeRun> runs;
    std::vector<B4BAggregate> aggregates;
    std::vector<Vec3> position;
    std::vector<Vec3> velocity;
    int active_steps = 0;
    int contact_events = 0;
    int outer_trials = 0;
    int rejected_trials = 0;
    int hvp_calls = 0;
    int floor_merit_accepts = 0;
    std::size_t maximum_pairs = 0;
    double maximum_penetration = 0.0;
    double maximum_ledger_residual = 0.0;
    double maximum_support_reaction_closure = 0.0;
    double maximum_mechanical_energy = -std::numeric_limits<double>::infinity();
    double first_contact_time = std::numeric_limits<double>::infinity();
    std::vector<std::pair<std::size_t, int>> terminal_contacts;
};

B4BFixedTrajectory run_b4b_fixed_trajectory(
    const SmokeFixture& fixture, int substeps_per_frame) {
    B4BFixedTrajectory result;
    result.substeps_per_frame = substeps_per_frame;
    result.position = fixture.position;
    result.velocity = fixture.velocity;
    for (int frame = 0; frame < fixture.macro_frames; ++frame) {
        SmokeRun run = run_smoke_interval(fixture,
            result.position, result.velocity, substeps_per_frame,
            static_cast<double>(frame) * SMOKE_FRAME_TIME,
            SMOKE_FRAME_TIME, true, true, true, true, true);
        result.runs.push_back(run);
        if (!run.passed) {
            result.failure = "FRAME_" + std::to_string(frame)
                + ':' + run.failure;
            return result;
        }
        result.position = run.position;
        result.velocity = run.velocity;
        result.aggregates.push_back(b4b_aggregate(
            fixture, result.position, result.velocity));
        result.active_steps += run.active_steps;
        result.contact_events += run.contact_events;
        result.outer_trials += run.outer_trials;
        result.rejected_trials += run.rejected_trials;
        result.hvp_calls += run.hvp_calls;
        result.floor_merit_accepts += run.floor_merit_accepts;
        result.maximum_pairs = std::max(
            result.maximum_pairs, run.maximum_pairs);
        result.maximum_penetration = std::max(
            result.maximum_penetration, run.maximum_penetration);
        result.maximum_ledger_residual = std::max(
            result.maximum_ledger_residual, run.maximum_ledger_residual);
        result.maximum_support_reaction_closure = std::max(
            result.maximum_support_reaction_closure,
            run.maximum_support_reaction_closure);
        result.maximum_mechanical_energy = std::max(
            result.maximum_mechanical_energy,
            run.maximum_mechanical_energy);
        result.first_contact_time = std::min(
            result.first_contact_time, run.first_contact_time);
        result.terminal_contacts = run.terminal_contacts;
    }
    result.passed = true;
    return result;
}

double b4b_rms_floor(
    const std::vector<Vec3>& lhs, const std::vector<Vec3>& rhs) {
    double magnitude = 0.0;
    for (std::size_t i = 0; i < lhs.size(); ++i) {
        magnitude += norm_squared(lhs[i]) + norm_squared(rhs[i]);
    }
    magnitude = std::sqrt(
        magnitude / static_cast<double>(2U * lhs.size()));
    return gamma_factor(16U + 6U * lhs.size())
        * std::max(magnitude, std::numeric_limits<double>::min());
}

struct B4BConvergence {
    bool passed = false;
    std::array<double, 2> position_difference{};
    std::array<double, 2> velocity_difference{};
    std::array<double, 2> position_floor{};
    std::array<double, 2> velocity_floor{};
    double position_ratio = 0.0;
    double velocity_ratio = 0.0;
    bool position_floor_overlap = false;
    bool velocity_floor_overlap = false;
};

struct B4BReference {
    bool passed = false;
    std::array<B4BFixedTrajectory, 3> levels;
    B4BConvergence convergence;
};

B4BReference run_b4b_reference(const SmokeFixture& fixture) {
    B4BReference result;
    for (std::size_t i = 0; i < B4B_REFERENCE_COUNTS.size(); ++i) {
        result.levels[i] = run_b4b_fixed_trajectory(
            fixture, B4B_REFERENCE_COUNTS[i]);
    }
    B4BConvergence& value = result.convergence;
    for (std::size_t i = 0; i < 2U; ++i) {
        value.position_difference[i] = rms_difference(
            result.levels[i].position, result.levels[i + 1U].position);
        value.velocity_difference[i] = rms_difference(
            result.levels[i].velocity, result.levels[i + 1U].velocity);
        value.position_floor[i] = b4b_rms_floor(
            result.levels[i].position, result.levels[i + 1U].position);
        value.velocity_floor[i] = b4b_rms_floor(
            result.levels[i].velocity, result.levels[i + 1U].velocity);
    }
    value.position_floor_overlap =
        value.position_difference[0] <= value.position_floor[0]
        && value.position_difference[1] <= value.position_floor[1];
    value.velocity_floor_overlap =
        value.velocity_difference[0] <= value.velocity_floor[0]
        && value.velocity_difference[1] <= value.velocity_floor[1];
    if (value.position_difference[1] > 0.0) {
        value.position_ratio = value.position_difference[0]
            / value.position_difference[1];
    }
    if (value.velocity_difference[1] > 0.0) {
        value.velocity_ratio = value.velocity_difference[0]
            / value.velocity_difference[1];
    }
    const bool position_order = value.position_difference[0] > 0.0
        && value.position_difference[1] > 0.0
        && value.position_ratio >= 1.25 && value.position_ratio <= 2.75;
    const bool velocity_order = value.velocity_difference[0] > 0.0
        && value.velocity_difference[1] > 0.0
        && value.velocity_ratio >= 1.25 && value.velocity_ratio <= 2.75;
    value.passed = (position_order || value.position_floor_overlap)
        && (velocity_order || value.velocity_floor_overlap);
    result.passed = std::all_of(result.levels.begin(), result.levels.end(),
            [](const B4BFixedTrajectory& level) { return level.passed; })
        && value.passed;
    return result;
}

struct B4BFrameComparison {
    int frame = 0;
    double position_dx = 0.0;
    double velocity_c = 0.0;
    double center_dx = 0.0;
    double q99_height_dx = 0.0;
    double q99_front_dx = 0.0;
    double kinetic_relative = 0.0;
    double kinetic_absolute = 0.0;
    double kinetic_floor = 0.0;
    bool kinetic_floor_overlap = false;
    bool passed = false;
};

double maximum_component_abs(Vec3 value) {
    return std::max({std::abs(value.x), std::abs(value.y),
        std::abs(value.z)});
}

B4BFrameComparison compare_b4b_frame(
    int frame, const SmokeFixture& fixture,
    const SmokeFrame& candidate,
    const SmokeRun& reference_run,
    const B4BAggregate& reference) {
    B4BFrameComparison result;
    result.frame = frame;
    const B4BAggregate candidate_value = b4b_aggregate(
        fixture, candidate.position, candidate.velocity);
    result.position_dx = rms_difference(
        candidate.position, reference_run.position) / SPACING;
    result.velocity_c = rms_difference(
        candidate.velocity, reference_run.velocity)
        / std::sqrt(KAPPA / MASS);
    result.center_dx = maximum_component_abs(
        candidate_value.center - reference.center) / SPACING;
    result.q99_height_dx = std::abs(
        candidate_value.q99_height - reference.q99_height) / SPACING;
    result.q99_front_dx = std::abs(
        candidate_value.q99_front - reference.q99_front) / SPACING;
    result.kinetic_absolute = std::abs(
        candidate_value.kinetic - reference.kinetic);
    const double kinetic_scale = std::max(
        candidate_value.kinetic, reference.kinetic);
    result.kinetic_floor = gamma_factor(
        32U + 12U * candidate.velocity.size())
        * std::max(kinetic_scale, std::numeric_limits<double>::min());
    result.kinetic_floor_overlap = kinetic_scale <= 1.0e-12
        && result.kinetic_absolute <= result.kinetic_floor;
    if (kinetic_scale > 1.0e-12) {
        result.kinetic_relative = result.kinetic_absolute / kinetic_scale;
    }
    result.passed = candidate_value.finite_values
        && reference.finite_values
        && result.position_dx <= 0.05
        && result.velocity_c <= 0.001
        && result.center_dx <= 0.05
        && result.q99_height_dx <= 0.10
        && result.q99_front_dx <= 0.10
        && (kinetic_scale > 1.0e-12
            ? result.kinetic_relative <= 0.15
            : result.kinetic_floor_overlap);
    return result;
}

struct B4BCase {
    bool passed = false;
    bool physical_passed = false;
    bool comparison_passed = false;
    bool work_passed = false;
    std::string failure;
    SmokeFixture fixture;
    SmokeController candidate;
    B4BReference reference;
    std::vector<B4BFrameComparison> comparisons;
    B4BAggregate initial;
    B4BAggregate final;
    double lateral_drift = 0.0;
    double vertical_center_change = 0.0;
    double energy_allowance = 0.0;
    double energy_creation = 0.0;
    double contact_time_error = 0.0;
    double contact_time_limit = 0.0;
    bool terminal_contacts_exact = false;
};

double b4b_contact_time_limit(const SmokeController& candidate) {
    if (!std::isfinite(candidate.first_contact_time)
        || candidate.frames.empty()) {
        return 0.0;
    }
    const int frame = std::clamp(
        static_cast<int>(candidate.first_contact_time / SMOKE_FRAME_TIME),
        0, static_cast<int>(candidate.frames.size()) - 1);
    return SMOKE_FRAME_TIME / static_cast<double>(
        candidate.frames[static_cast<std::size_t>(frame)].accepted_substeps)
        + 64.0 * std::numeric_limits<double>::epsilon();
}

bool b4b_work_gate(const B4BCase& value) {
    bool passed = value.fixture.position.size()
            + value.fixture.boundary.size() <= 1280U
        && value.candidate.maximum_pairs
            <= 160U * value.fixture.position.size();
    for (const SmokeFrame& frame : value.candidate.frames) {
        const int largest_level = frame.initial_substeps
            * (1 << (frame.refinement_depth + 1));
        passed = passed && frame.accepted_substeps <= 192
            && largest_level <= 768;
    }
    for (const B4BFixedTrajectory& level : value.reference.levels) {
        passed = passed && level.maximum_pairs
            <= 160U * value.fixture.position.size();
    }
    return passed;
}

B4BCase run_b4b_case(SmokeFixture fixture, bool released_block) {
    B4BCase result;
    result.fixture = std::move(fixture);
    result.initial = b4b_aggregate(result.fixture,
        result.fixture.position, result.fixture.velocity);
    result.candidate = run_smoke_controller(result.fixture, true, true);
    if (!result.candidate.passed) {
        result.failure = "CANDIDATE:" + result.candidate.failure;
        return result;
    }
    result.reference = run_b4b_reference(result.fixture);
    const bool reference_runs_passed = std::all_of(
        result.reference.levels.begin(), result.reference.levels.end(),
        [](const B4BFixedTrajectory& level) { return level.passed; });
    if (!reference_runs_passed) {
        result.failure = "REFERENCE_RUN";
        return result;
    }
    if (!result.reference.convergence.passed) {
        result.failure = "REFERENCE_CONVERGENCE";
        return result;
    }
    const B4BFixedTrajectory& fine = result.reference.levels[2];
    for (std::size_t frame = 0; frame < result.candidate.frames.size();
         ++frame) {
        result.comparisons.push_back(compare_b4b_frame(
            static_cast<int>(frame), result.fixture,
            result.candidate.frames[frame], fine.runs[frame],
            fine.aggregates[frame]));
    }
    result.comparison_passed = std::all_of(
        result.comparisons.begin(), result.comparisons.end(),
        [](const B4BFrameComparison& value) { return value.passed; });
    result.final = b4b_aggregate(result.fixture,
        result.candidate.position, result.candidate.velocity);
    result.lateral_drift = std::max(
        std::abs(result.final.center.x - result.initial.center.x),
        std::abs(result.final.center.z - result.initial.center.z));
    result.vertical_center_change = std::abs(
        result.final.center.y - result.initial.center.y);
    result.energy_allowance = 0.01 * std::max({
        std::abs(result.initial.mechanical),
        static_cast<double>(result.fixture.position.size())
            * MASS * (-result.fixture.gravity.y) * SPACING,
        1.0e-12,
    });
    result.energy_creation = std::max(0.0,
        result.candidate.maximum_mechanical_energy
            - result.initial.mechanical);
    result.contact_time_error = event_time_error(
        result.candidate.first_contact_time, fine.first_contact_time);
    result.contact_time_limit = b4b_contact_time_limit(result.candidate);
    result.terminal_contacts_exact = result.candidate.terminal_contacts
        == fine.terminal_contacts;

    const bool common_physical = result.initial.finite_values
        && result.final.finite_values
        && result.fixture.velocity.size() == result.fixture.position.size()
        && result.lateral_drift <= 1.0e-10
        && result.candidate.contact_events > 0
        && result.candidate.accepted_active_steps > 0
        && result.candidate.maximum_penetration <= 1.0e-12
        && result.candidate.maximum_ledger_residual <= 1.0e-9
        && result.candidate.maximum_support_reaction_closure <= 1.0e-10
        && result.energy_creation <= result.energy_allowance
        && result.terminal_contacts_exact;
    if (released_block) {
        result.physical_passed = common_physical
            && result.fixture.position.size() == 27U
            && static_cast<double>(result.fixture.position.size()) * MASS
                == 3.375
            && result.candidate.precontact_steps > 0
            && result.candidate.precontact_pressure_violations == 0
            && result.candidate.maximum_precontact_support_reaction
                <= 1.0e-12
            && result.candidate.maximum_precontact_velocity_spread
                <= 1.0e-12
            && result.candidate.maximum_precontact_position_error
                <= 1.0e-12
            && result.candidate.maximum_precontact_velocity_error
                <= 1.0e-12
            && result.contact_time_error <= result.contact_time_limit;
    } else {
        result.physical_passed = common_physical
            && result.fixture.position.size() == 48U
            && static_cast<double>(result.fixture.position.size()) * MASS
                == 6.0
            && result.vertical_center_change <= 0.05 * SPACING
            && result.candidate.maximum_positive_density_strain <= 1.0e-3
            && result.candidate.maximum_speed
                <= 0.01 * std::sqrt(KAPPA / MASS);
    }
    result.work_passed = b4b_work_gate(result);
    result.passed = result.comparison_passed
        && result.physical_passed && result.work_passed;
    if (!result.comparison_passed) {
        result.failure = "FRAME_COMPARISON";
    } else if (!result.physical_passed) {
        result.failure = "PHYSICAL_GATE";
    } else if (!result.work_passed) {
        result.failure = "WORK_GATE";
    }
    return result;
}

void append_b4b_number(std::ostringstream& output, double value) {
    if (std::isfinite(value)) {
        output << value;
    } else {
        output << "null";
    }
}

void append_b4b_aggregate(
    std::ostringstream& output, const B4BAggregate& value) {
    output << "{\"center_m\":";
    append_vec3(output, value.center);
    output << ",\"momentum_kg_m_s\":";
    append_vec3(output, value.momentum_value);
    output << ",\"q99_height_m\":" << value.q99_height
           << ",\"q99_front_m\":" << value.q99_front
           << ",\"kinetic_j\":" << value.kinetic
           << ",\"pressure_j\":" << value.pressure
           << ",\"gravitational_j\":" << value.gravitational
           << ",\"mechanical_j\":" << value.mechanical
           << ",\"maximum_density_ratio\":"
           << value.maximum_density_ratio
           << ",\"maximum_positive_density_strain\":"
           << value.maximum_positive_strain
           << ",\"maximum_speed_m_s\":"
           << value.maximum_speed_value
           << ",\"finite\":"
           << (value.finite_values ? "true" : "false") << '}';
}

void append_b4b_convergence(
    std::ostringstream& output, const B4BConvergence& value) {
    output << "{\"status\":\"" << (value.passed ? "PASS" : "FAIL")
           << "\",\"position_difference_m\":["
           << value.position_difference[0] << ','
           << value.position_difference[1]
           << "],\"velocity_difference_m_s\":["
           << value.velocity_difference[0] << ','
           << value.velocity_difference[1]
           << "],\"position_floor_m\":["
           << value.position_floor[0] << ',' << value.position_floor[1]
           << "],\"velocity_floor_m_s\":["
           << value.velocity_floor[0] << ',' << value.velocity_floor[1]
           << "],\"position_ratio\":";
    append_b4b_number(output, value.position_ratio);
    output << ",\"velocity_ratio\":";
    append_b4b_number(output, value.velocity_ratio);
    output << ",\"position_floor_overlap\":"
           << (value.position_floor_overlap ? "true" : "false")
           << ",\"velocity_floor_overlap\":"
           << (value.velocity_floor_overlap ? "true" : "false") << '}';
}

void append_b4b_case(std::ostringstream& output, const B4BCase& value) {
    output << "{\"name\":\"" << value.fixture.name
           << "\",\"status\":\"" << (value.passed ? "PASS" : "FAIL")
           << "\",\"failure\":\"" << value.failure
           << "\",\"geometry_sha256\":\""
           << value.fixture.geometry_sha256
           << "\",\"fluid_samples\":" << value.fixture.position.size()
           << ",\"support_samples\":" << value.fixture.boundary.size()
           << ",\"mass_kg\":"
           << static_cast<double>(value.fixture.position.size()) * MASS
           << ",\"macro_frames\":" << value.fixture.macro_frames
           << ",\"initial\":";
    append_b4b_aggregate(output, value.initial);
    output << ",\"final\":";
    append_b4b_aggregate(output, value.final);
    output << ",\"candidate\":{\"passed\":"
           << (value.candidate.passed ? "true" : "false")
           << ",\"failure\":\"" << value.candidate.failure
           << "\",\"accepted_substeps\":"
           << value.candidate.accepted_substeps
           << ",\"executed_substeps\":"
           << value.candidate.executed_substeps
           << ",\"discarded_substeps\":"
           << value.candidate.discarded_substeps
           << ",\"spectral_hvp_calls\":"
           << value.candidate.spectral_hvp_calls
           << ",\"nonlinear_hvp_calls\":"
           << value.candidate.nonlinear_hvp_calls
           << ",\"outer_trials\":" << value.candidate.outer_trials
           << ",\"rejected_trials\":"
           << value.candidate.rejected_trials
           << ",\"floor_merit_trials\":"
           << value.candidate.floor_merit_trials
           << ",\"floor_merit_accepts\":"
           << value.candidate.floor_merit_accepts
           << ",\"accepted_active_steps\":"
           << value.candidate.accepted_active_steps
           << ",\"accepted_inactive_steps\":"
           << value.candidate.accepted_inactive_steps
           << ",\"contact_events\":"
           << value.candidate.contact_events
           << ",\"maximum_pairs\":" << value.candidate.maximum_pairs
           << ",\"maximum_penetration_m\":"
           << value.candidate.maximum_penetration
           << ",\"maximum_ledger_residual\":"
           << value.candidate.maximum_ledger_residual
           << ",\"maximum_support_reaction_closure\":"
           << value.candidate.maximum_support_reaction_closure
           << ",\"maximum_mechanical_energy_j\":";
    append_b4b_number(output, value.candidate.maximum_mechanical_energy);
    output << ",\"maximum_positive_density_strain\":"
           << value.candidate.maximum_positive_density_strain
           << ",\"maximum_speed_m_s\":"
           << value.candidate.maximum_speed
           << ",\"first_contact_time_s\":";
    append_b4b_number(output, value.candidate.first_contact_time);
    output << ",\"support_reaction_impulse_n_s\":";
    append_vec3(output, value.candidate.support_reaction);
    output << ",\"contact_reaction_impulse_n_s\":";
    append_vec3(output, value.candidate.contact_reaction);
    output << ",\"gravity_impulse_n_s\":";
    append_vec3(output, value.candidate.gravity_impulse);
    output << ",\"precontact\":{\"steps\":"
           << value.candidate.precontact_steps
           << ",\"pressure_violations\":"
           << value.candidate.precontact_pressure_violations
           << ",\"maximum_support_reaction_n_s\":"
           << value.candidate.maximum_precontact_support_reaction
           << ",\"maximum_position_error_m\":"
           << value.candidate.maximum_precontact_position_error
           << ",\"maximum_velocity_error_m_s\":"
           << value.candidate.maximum_precontact_velocity_error
           << ",\"maximum_velocity_spread_m_s\":"
           << value.candidate.maximum_precontact_velocity_spread
           << "},\"frames\":[";
    for (std::size_t i = 0; i < value.candidate.frames.size(); ++i) {
        if (i != 0U) {
            output << ',';
        }
        const SmokeFrame& frame = value.candidate.frames[i];
        output << "{\"frame\":" << frame.frame
               << ",\"active_centers_at_start\":"
               << frame.active_centers
               << ",\"spectral_hvp_calls\":"
               << frame.spectral_hvp_calls
               << ",\"maximum_eigenvalue\":"
               << frame.maximum_eigenvalue
               << ",\"initial_substeps\":"
               << frame.initial_substeps
               << ",\"refinement_depth\":"
               << frame.refinement_depth
               << ",\"accepted_substeps\":"
               << frame.accepted_substeps
               << ",\"executed_substeps\":"
               << frame.executed_substeps
               << ",\"discarded_substeps\":"
               << frame.discarded_substeps << '}';
    }
    output << "]},\"reference\":{";
    output << "\"levels\":[";
    for (std::size_t i = 0; i < value.reference.levels.size(); ++i) {
        if (i != 0U) {
            output << ',';
        }
        const B4BFixedTrajectory& level = value.reference.levels[i];
        output << "{\"substeps_per_frame\":"
               << level.substeps_per_frame
               << ",\"passed\":" << (level.passed ? "true" : "false")
               << ",\"failure\":\"" << level.failure
               << "\",\"active_steps\":" << level.active_steps
               << ",\"contact_events\":" << level.contact_events
               << ",\"outer_trials\":" << level.outer_trials
               << ",\"rejected_trials\":" << level.rejected_trials
               << ",\"hvp_calls\":" << level.hvp_calls
               << ",\"floor_merit_accepts\":"
               << level.floor_merit_accepts
               << ",\"maximum_pairs\":" << level.maximum_pairs
               << ",\"maximum_penetration_m\":"
               << level.maximum_penetration
               << ",\"maximum_ledger_residual\":"
               << level.maximum_ledger_residual
               << ",\"failure_diagnostic\":{\"reaction_defect_n_s\":"
               << (level.runs.empty()
                    ? 0.0 : level.runs.back().failure_reaction_defect)
               << ",\"reaction_limit_n_s\":"
               << (level.runs.empty()
                    ? 0.0 : level.runs.back().failure_reaction_limit)
               << ",\"predicted_reduction_j\":"
               << (level.runs.empty()
                    ? 0.0 : level.runs.back().failure_predicted_reduction)
               << ",\"energy_floor_j\":"
               << (level.runs.empty()
                    ? 0.0 : level.runs.back().failure_energy_floor)
               << ",\"scaled_residual\":"
               << (level.runs.empty()
                    ? 0.0 : level.runs.back().failure_scaled_residual)
               << ",\"step_dx\":"
               << (level.runs.empty()
                    ? 0.0 : level.runs.back().failure_step_dx)
               << ",\"floor_trials\":"
               << (level.runs.empty()
                    ? 0 : level.runs.back().failure_floor_merit_trials)
               << ",\"floor_accepts\":"
               << (level.runs.empty()
                    ? 0 : level.runs.back().failure_floor_merit_accepts)
               << ",\"floor_trial_defect_n_s\":"
               << (level.runs.empty()
                    ? 0.0 : level.runs.back().failure_floor_trial_defect)
               << ",\"floor_trial_limit_n_s\":"
               << (level.runs.empty()
                    ? 0.0 : level.runs.back().failure_floor_trial_limit)
               << ",\"floor_residual_ratio\":"
               << (level.runs.empty()
                    ? 0.0 : level.runs.back().failure_floor_residual_ratio)
               << ",\"floor_topology_exact\":"
               << (!level.runs.empty()
                    && level.runs.back().failure_floor_topology_exact
                    ? "true" : "false") << '}'
               << ",\"first_contact_time_s\":";
        append_b4b_number(output, level.first_contact_time);
        output << '}';
    }
    output << "],\"convergence\":";
    append_b4b_convergence(output, value.reference.convergence);
    output << "},\"frame_comparisons\":[";
    for (std::size_t i = 0; i < value.comparisons.size(); ++i) {
        if (i != 0U) {
            output << ',';
        }
        const B4BFrameComparison& frame = value.comparisons[i];
        output << "{\"frame\":" << frame.frame
               << ",\"status\":\"" << (frame.passed ? "PASS" : "FAIL")
               << "\",\"position_dx\":" << frame.position_dx
               << ",\"velocity_c\":" << frame.velocity_c
               << ",\"center_dx\":" << frame.center_dx
               << ",\"q99_height_dx\":" << frame.q99_height_dx
               << ",\"q99_front_dx\":" << frame.q99_front_dx
               << ",\"kinetic_relative\":"
               << frame.kinetic_relative
               << ",\"kinetic_absolute_j\":"
               << frame.kinetic_absolute
               << ",\"kinetic_floor_j\":" << frame.kinetic_floor
               << ",\"kinetic_floor_overlap\":"
               << (frame.kinetic_floor_overlap ? "true" : "false")
               << '}';
    }
    output << "],\"physical\":{";
    output << "\"status\":\"" << (value.physical_passed ? "PASS" : "FAIL")
           << "\",\"lateral_center_drift_m\":" << value.lateral_drift
           << ",\"vertical_center_change_m\":"
           << value.vertical_center_change
           << ",\"energy_creation_j\":" << value.energy_creation
           << ",\"energy_allowance_j\":" << value.energy_allowance
           << ",\"contact_time_error_s\":";
    append_b4b_number(output, value.contact_time_error);
    output << ",\"contact_time_limit_s\":" << value.contact_time_limit
           << ",\"terminal_contacts_exact\":"
           << (value.terminal_contacts_exact ? "true" : "false")
           << ",\"bottom_to_top_density_ratio\":[";
    if (value.fixture.position.size() == 48U
        && value.candidate.position.size() == 48U) {
        const Evaluation density = evaluate(
            value.candidate.position, value.fixture.boundary);
        std::array<double, 3> sum{};
        std::array<int, 3> count{};
        for (std::size_t i = 0; i < density.density.size(); ++i) {
            const std::size_t row = (i / 4U) % 3U;
            sum[row] += density.density[i] / REST_DENSITY;
            ++count[row];
        }
        for (std::size_t row = 0; row < 3U; ++row) {
            if (row != 0U) {
                output << ',';
            }
            output << sum[row] / static_cast<double>(count[row]);
        }
    }
    output << ']'
           << "},\"comparison_status\":\""
           << (value.comparison_passed ? "PASS" : "FAIL")
           << "\",\"work_status\":\""
           << (value.work_passed ? "PASS" : "FAIL") << "\"}";
}

} // namespace

SplitBoundaryReport run_tiny_pressure_corpus_controls() {
    const SplitBoundaryReport parent = run_closed_box_eligibility_controls();
    const bool parent_exact = parent.passed
        && sha256_hex(parent.json)
            == "7282f6ab736d4bea423cc20a1dd20092a156afb3eb5d61741d6b8b4e10674c51";
    std::vector<B4BCase> cases;
    std::string first_failure;
    if (!parent_exact) {
        first_failure = "NSR3B4A_PARENT";
    } else {
        cases.push_back(run_b4b_case(
            make_b4b_supported_column_fixture(), false));
        if (!cases.back().passed) {
            first_failure = "P1_SUPPORTED_COLUMN:" + cases.back().failure;
        } else {
            cases.push_back(run_b4b_case(
                make_b4b_released_block_fixture(), true));
            if (!cases.back().passed) {
                first_failure = "P2_RELEASED_BLOCK:" + cases.back().failure;
            }
        }
    }
    const bool passed = parent_exact && cases.size() == 2U
        && std::all_of(cases.begin(), cases.end(),
            [](const B4BCase& value) { return value.passed; });
    const std::string disposition = passed
        ? "TINY_PRESSURE_CORPUS_CANDIDATE"
        : "TINY_PRESSURE_CORPUS_REJECTED";
    std::ostringstream material;
    material << std::setprecision(17)
             << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
             << disposition;
    for (const B4BCase& value : cases) {
        material << '|' << value.fixture.name << ':' << value.passed
                 << ':' << value.candidate.accepted_substeps
                 << ':' << value.candidate.executed_substeps
                 << ':' << value.candidate.nonlinear_hvp_calls
                 << ':' << value.candidate.maximum_pairs
                 << ':' << value.lateral_drift
                 << ':' << value.vertical_center_change
                 << ':' << value.energy_creation
                 << ':' << value.contact_time_error;
    }
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4b_pressure.v1\""
           << ",\"identity\":\"tiny-pressure-water-corpus-r0\""
           << ",\"parent_b4a_result_sha256\":\"5e069b7e7c86da39aef944d31fc184a564eaa8e794c54f9e63d0e855852567b1\""
           << ",\"parent_b4a_raw_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"disposition\":\"" << disposition << '"'
           << ",\"terms\":{"
           << "\"incompressibility\":true,\"viscosity\":false,"
           << "\"surface_tension\":false}"
           << ",\"cases\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0U) {
            report << ',';
        }
        append_b4b_case(report, cases[i]);
    }
    report << ']'
           << ",\"candidate_selected\":"
           << (passed ? "true" : "false")
           << ",\"b4c_neighborhood_design_authorized\":"
           << (passed ? "true" : "false")
           << ",\"nominal_corpus_execution_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"production_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(material.str()) << "\"}";
    return {passed, report.str()};
}

namespace {

struct BoxKktState {
    SmoothEvaluation smooth;
    std::vector<Vec3> projected_gradient;
    std::vector<Vec3> active_gradient;
    std::vector<std::array<bool, 3>> active_axis;
    int lower_axes = 0;
    int upper_axes = 0;
    int lower_y_axes = 0;
    int lateral_or_upper_axes = 0;
    std::array<int, 6> face_counts{};
    std::array<double, 6> face_multiplier_sum{};
    std::array<double, 6> face_fluid_impulse{};
    double projected_impulse_residual = 0.0;
    double reaction_limit = 0.0;
    double complementarity = 0.0;
    double minimum_multiplier = std::numeric_limits<double>::infinity();
    double maximum_penetration = 0.0;
    double support_translation_closure = 0.0;
    double contact_closure = 0.0;
    double ledger_absolute = 0.0;
    double ledger_residual = 0.0;
    Vec3 actual_impulse;
    Vec3 fluid_pressure_impulse;
    Vec3 support_reaction;
    Vec3 fluid_contact_impulse;
    Vec3 contact_reaction;
    Vec3 gravity_impulse;
    bool finite_values = false;
    bool passed = false;
};

BoxKktState evaluate_box_kkt(
    const SmokeFixture& fixture,
    const std::vector<Vec3>& start_position,
    const std::vector<Vec3>& position,
    const std::vector<Vec3>& velocity,
    const std::vector<Vec3>& displacement,
    const std::vector<Vec3>& predicted_displacement,
    double time_step) {
    BoxKktState result;
    result.smooth = smooth_evaluate_owned(position, displacement,
        predicted_displacement, fixture.boundary, time_step);
    result.projected_gradient = result.smooth.gradient;
    result.active_gradient.resize(position.size());
    result.active_axis.resize(position.size());
    for (std::size_t i = 0; i < position.size(); ++i) {
        for (int axis = 0; axis < 3; ++axis) {
            const double low = component(fixture.contact_low, axis)
                - component(start_position[i], axis);
            const double high = component(fixture.contact_high, axis)
                - component(start_position[i], axis);
            const double value = component(displacement[i], axis);
            const double gradient = component(result.smooth.gradient[i], axis);
            const bool lower = value == low && gradient >= 0.0;
            const bool upper = value == high && gradient <= 0.0;
            if (lower || upper) {
                result.active_axis[i][static_cast<std::size_t>(axis)] = true;
                set_component(result.projected_gradient[i], axis, 0.0);
                set_component(result.active_gradient[i], axis, gradient);
                const double multiplier = lower ? gradient : -gradient;
                const std::size_t face = static_cast<std::size_t>(
                    2 * axis + (upper ? 1 : 0));
                ++result.face_counts[face];
                result.face_multiplier_sum[face] += multiplier;
                result.face_fluid_impulse[face] += time_step * gradient;
                result.minimum_multiplier = std::min(
                    result.minimum_multiplier, multiplier);
                result.complementarity = std::max(result.complementarity,
                    multiplier * std::abs(lower ? value - low : high - value));
                result.lower_axes += lower ? 1 : 0;
                result.upper_axes += upper ? 1 : 0;
                result.lower_y_axes += lower && axis == 1 ? 1 : 0;
                result.lateral_or_upper_axes +=
                    upper || (lower && axis != 1) ? 1 : 0;
            }
            result.maximum_penetration = std::max(
                result.maximum_penetration,
                std::max(low - value, value - high));
        }
        const Vec3 v_star = velocity[i] + time_step * fixture.gravity;
        result.actual_impulse += MASS
            * (displacement[i] / time_step - v_star);
    }
    result.maximum_penetration = std::max(
        result.maximum_penetration, 0.0);
    const Vec3 fluid_gradient = sum_values(
        result.smooth.support.gradient, 0U, position.size());
    const Vec3 boundary_gradient = sum_values(
        result.smooth.support.gradient, position.size(),
        result.smooth.support.gradient.size());
    result.fluid_pressure_impulse = -time_step * fluid_gradient;
    result.support_reaction = -time_step * boundary_gradient;
    result.fluid_contact_impulse = time_step
        * sum_values(result.active_gradient, 0U,
            result.active_gradient.size());
    result.contact_reaction = -result.fluid_contact_impulse;
    result.gravity_impulse = static_cast<double>(position.size())
        * MASS * time_step * fixture.gravity;
    result.projected_impulse_residual = time_step
        * vector_norm(result.projected_gradient);
    const double support_scale = norm(result.fluid_pressure_impulse)
        + norm(result.support_reaction);
    result.support_translation_closure = norm(
        result.fluid_pressure_impulse + result.support_reaction)
        / std::max(support_scale, 1.0e-30);
    result.contact_closure = norm(
        result.fluid_contact_impulse + result.contact_reaction);
    const Vec3 stationarity = result.actual_impulse
        - result.fluid_pressure_impulse - result.fluid_contact_impulse;
    const double impulse_scale = std::max({
        norm(result.actual_impulse)
            + norm(result.fluid_pressure_impulse)
            + norm(result.fluid_contact_impulse),
        static_cast<double>(position.size()) * MASS * time_step
            * norm(fixture.gravity),
        1.0e-12,
    });
    result.reaction_limit = 1.0e-9 * impulse_scale
        + displacement_forward_bound(velocity, fixture.gravity, time_step);
    const Vec3 new_momentum = [&]() {
        Vec3 value;
        for (Vec3 delta : displacement) {
            value += MASS * delta / time_step;
        }
        return value;
    }();
    const Vec3 ledger = new_momentum - momentum(velocity)
        - result.gravity_impulse + result.support_reaction
        + result.contact_reaction;
    const double ledger_scale = norm(new_momentum - momentum(velocity))
        + norm(result.gravity_impulse) + norm(result.support_reaction)
        + norm(result.contact_reaction);
    result.ledger_absolute = norm(ledger);
    result.ledger_residual = result.ledger_absolute
        / std::max(ledger_scale, 1.0e-30);
    if (!std::isfinite(result.minimum_multiplier)) {
        result.minimum_multiplier = 0.0;
    }
    result.finite_values = std::isfinite(result.smooth.total)
        && std::isfinite(result.projected_impulse_residual)
        && std::isfinite(result.reaction_limit)
        && std::isfinite(result.minimum_multiplier)
        && finite(result.actual_impulse)
        && finite(stationarity);
    result.passed = result.finite_values
        && result.maximum_penetration <= 1.0e-12
        && result.minimum_multiplier >= 0.0
        && result.complementarity == 0.0
        && result.projected_impulse_residual <= result.reaction_limit
        && norm(stationarity) <= result.reaction_limit
        && result.support_translation_closure <= 1.0e-10
        && result.contact_closure <= 1.0e-12
        && result.ledger_residual <= 1.0e-9;
    return result;
}

void zero_active_components(
    std::vector<Vec3>& values,
    const std::vector<std::array<bool, 3>>& active_axis) {
    for (std::size_t i = 0; i < values.size(); ++i) {
        for (int axis = 0; axis < 3; ++axis) {
            if (active_axis[i][static_cast<std::size_t>(axis)]) {
                set_component(values[i], axis, 0.0);
            }
        }
    }
}

std::vector<Vec3> box_kkt_trust_step(
    const std::vector<Vec3>& position,
    const std::vector<Vec3>& boundary,
    const BoxKktState& state,
    double time_step,
    double radius,
    int& hvp_calls,
    bool& negative_curvature) {
    std::vector<Vec3> point(state.projected_gradient.size());
    std::vector<Vec3> residual = state.projected_gradient;
    std::vector<Vec3> direction = residual;
    for (Vec3& value : direction) {
        value = -value;
    }
    double residual_squared = flat_dot(residual, residual);
    const double initial_residual = std::sqrt(residual_squared);
    if (initial_residual == 0.0) {
        return point;
    }
    for (std::size_t iteration = 0;
         iteration < 3U * direction.size(); ++iteration) {
        std::vector<Vec3> image = smooth_hvp(
            position, boundary, direction, time_step);
        ++hvp_calls;
        zero_active_components(image, state.active_axis);
        const double curvature = flat_dot(direction, image);
        if (!std::isfinite(curvature) || curvature <= 0.0) {
            negative_curvature = true;
            return add_scaled(point, direction,
                trust_boundary_tau(point, direction, radius));
        }
        const double alpha = residual_squared / curvature;
        const std::vector<Vec3> candidate = add_scaled(
            point, direction, alpha);
        if (vector_norm(candidate) >= radius) {
            return add_scaled(point, direction,
                trust_boundary_tau(point, direction, radius));
        }
        point = candidate;
        std::vector<Vec3> next_residual = residual;
        for (std::size_t i = 0; i < next_residual.size(); ++i) {
            next_residual[i] += alpha * image[i];
        }
        zero_active_components(next_residual, state.active_axis);
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
        zero_active_components(direction, state.active_axis);
        residual = std::move(next_residual);
        residual_squared = next_squared;
    }
    return point;
}

struct BoxKktSolve {
    bool passed = false;
    std::string failure;
    std::vector<Vec3> position;
    std::vector<Vec3> velocity;
    std::vector<Vec3> displacement;
    BoxKktState state;
    int outer_trials = 0;
    int accepted_trials = 0;
    int rejected_trials = 0;
    int hvp_calls = 0;
    int negative_curvature_exits = 0;
    int projected_trials = 0;
    int active_set_changes = 0;
    int floor_merit_trials = 0;
    int floor_merit_accepts = 0;
    double initial_objective = 0.0;
    double objective_forward_bound = 0.0;
};

std::vector<Vec3> clamp_box_displacement(
    const SmokeFixture& fixture,
    const std::vector<Vec3>& start_position,
    const std::vector<Vec3>& displacement) {
    std::vector<Vec3> result = displacement;
    for (std::size_t i = 0; i < result.size(); ++i) {
        for (int axis = 0; axis < 3; ++axis) {
            const double low = component(fixture.contact_low, axis)
                - component(start_position[i], axis);
            const double high = component(fixture.contact_high, axis)
                - component(start_position[i], axis);
            set_component(result[i], axis, std::clamp(
                component(result[i], axis), low, high));
        }
    }
    return result;
}

std::vector<Vec3> materialize_displacement(
    const std::vector<Vec3>& start,
    const std::vector<Vec3>& displacement) {
    std::vector<Vec3> result(start.size());
    for (std::size_t i = 0; i < start.size(); ++i) {
        result[i] = start[i] + displacement[i];
    }
    return result;
}

bool same_active_set(const BoxKktState& lhs, const BoxKktState& rhs) {
    return lhs.active_axis == rhs.active_axis;
}

BoxKktSolve solve_box_kkt_step(
    const SmokeFixture& fixture,
    const std::vector<Vec3>& start_position,
    const std::vector<Vec3>& start_velocity,
    double time_step) {
    BoxKktSolve result;
    std::vector<Vec3> predicted(start_position.size());
    for (std::size_t i = 0; i < predicted.size(); ++i) {
        predicted[i] = time_step
            * (start_velocity[i] + time_step * fixture.gravity);
    }
    result.displacement = clamp_box_displacement(
        fixture, start_position, predicted);
    result.position = materialize_displacement(
        start_position, result.displacement);
    result.state = evaluate_box_kkt(fixture, start_position, result.position,
        start_velocity, result.displacement, predicted, time_step);
    result.initial_objective = result.state.smooth.total;
    result.objective_forward_bound = 1024.0
        * std::numeric_limits<double>::epsilon()
        * std::max(std::abs(result.initial_objective), 1.0);
    double trust_radius = 0.25 * SPACING;
    for (int outer = 0; outer < 64; ++outer) {
        result.outer_trials = outer + 1;
        if (result.state.passed) {
            result.velocity.resize(result.displacement.size());
            for (std::size_t i = 0; i < result.velocity.size(); ++i) {
                result.velocity[i] = result.displacement[i] / time_step;
            }
            result.passed = result.state.smooth.total
                <= result.initial_objective + result.objective_forward_bound;
            if (!result.passed) {
                result.failure = "OBJECTIVE_ABOVE_FEASIBLE_PREDICTOR";
            }
            return result;
        }
        bool negative_curvature = false;
        const std::vector<Vec3> raw_step = box_kkt_trust_step(
            result.position, fixture.boundary, result.state, time_step,
            trust_radius, result.hvp_calls, negative_curvature);
        result.negative_curvature_exits += negative_curvature ? 1 : 0;
        std::vector<Vec3> trial_displacement = add_scaled(
            result.displacement, raw_step, 1.0);
        trial_displacement = clamp_box_displacement(
            fixture, start_position, trial_displacement);
        ++result.projected_trials;
        std::vector<Vec3> actual_step(trial_displacement.size());
        for (std::size_t i = 0; i < actual_step.size(); ++i) {
            actual_step[i] = trial_displacement[i] - result.displacement[i];
        }
        if (vector_norm(actual_step) == 0.0) {
            result.failure = "ZERO_PROJECTED_STEP";
            return result;
        }
        const std::vector<Vec3> trial_position = materialize_displacement(
            start_position, trial_displacement);
        const BoxKktState trial = evaluate_box_kkt(fixture, start_position,
            trial_position, start_velocity, trial_displacement,
            predicted, time_step);
        const std::vector<Vec3> image = smooth_hvp(
            result.position, fixture.boundary, actual_step, time_step);
        ++result.hvp_calls;
        const double predicted_reduction = -flat_dot(
            result.state.smooth.gradient, actual_step)
            - 0.5 * flat_dot(actual_step, image);
        const double actual_reduction = result.state.smooth.total
            - trial.smooth.total;
        const double energy_floor = 1024.0
            * std::numeric_limits<double>::epsilon()
            * std::max(std::abs(result.state.smooth.total), 1.0);
        bool accept = false;
        if (predicted_reduction > energy_floor
            && actual_reduction > 0.0) {
            const double ratio = actual_reduction / predicted_reduction;
            accept = ratio >= 0.1;
            if (ratio < 0.25) {
                trust_radius *= 0.25;
            } else if (ratio > 0.75
                && vector_norm(actual_step) >= 0.9 * trust_radius) {
                trust_radius = std::min(
                    2.0 * trust_radius, 2.0 * SPACING);
            }
        } else if (predicted_reduction > 0.0
            && predicted_reduction <= energy_floor) {
            ++result.floor_merit_trials;
            accept = trial.finite_values
                && trial.minimum_multiplier >= 0.0
                && trial.projected_impulse_residual
                    < result.state.projected_impulse_residual
                && trial.ledger_absolute < result.state.ledger_absolute
                && result.floor_merit_accepts < 4;
            if (accept) {
                ++result.floor_merit_accepts;
            }
        }
        if (accept) {
            result.active_set_changes += same_active_set(
                result.state, trial) ? 0 : 1;
            result.displacement = std::move(trial_displacement);
            result.position = std::move(trial_position);
            result.state = trial;
            ++result.accepted_trials;
        } else {
            ++result.rejected_trials;
            trust_radius *= 0.25;
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

BoxKktSolve solve_box_kkt_step(
    const SmokeFixture& fixture, double time_step) {
    return solve_box_kkt_step(fixture,
        fixture.position, fixture.velocity, time_step);
}

struct B4BKReplay {
    int substeps_per_frame = 0;
    double time_step = 0.0;
    SmokeStep split;
    BoxKktSolve constrained;
    bool split_exact = false;
    bool constrained_gate = false;
};

B4BKReplay run_b4bk_replay(
    const SmokeFixture& fixture, int substeps_per_frame) {
    B4BKReplay result;
    result.substeps_per_frame = substeps_per_frame;
    result.time_step = SMOKE_FRAME_TIME
        / static_cast<double>(substeps_per_frame);
    result.split = execute_smoke_step(fixture,
        fixture.position, fixture.velocity, result.time_step,
        true, true, true, true, true);
    result.constrained = solve_box_kkt_step(fixture, result.time_step);
    if (substeps_per_frame == 48) {
        result.split_exact = result.split.passed;
    } else if (substeps_per_frame == 96) {
        result.split_exact = !result.split.passed
            && result.split.smooth.failure
                == "REACTION_BELOW_ENERGY_RESOLUTION"
            && result.split.smooth.reaction_stationarity_defect
                == 5.9689936631574676e-7
            && result.split.smooth.reaction_mixed_limit
                == 2.5546920380366131e-12
            && result.split.smooth.last_predicted_reduction
                == 1.1151393208186398e-13
            && result.split.smooth.numerical_energy_floor
                == 2.2737367544323206e-13
            && vector_norm(result.split.smooth.last_step) / SPACING
                == 1.1589205474866491e-9
            && result.split.smooth.floor_trial_residual_ratio
                == 1.2778566964207925e-4
            && !result.split.smooth.floor_trial_topology_exact;
    } else if (substeps_per_frame == 192) {
        result.split_exact = !result.split.passed
            && result.split.smooth.failure
                == "REACTION_BELOW_ENERGY_RESOLUTION"
            && result.split.smooth.reaction_stationarity_defect
                == 7.4612397140137767e-8
            && result.split.smooth.reaction_mixed_limit
                == 1.2773460190183065e-12
            && result.split.smooth.last_predicted_reduction
                == 1.7437192547456636e-15
            && result.split.smooth.numerical_energy_floor
                == 2.2737367544323206e-13
            && vector_norm(result.split.smooth.last_step) / SPACING
                == 7.2487174912679562e-11
            && result.split.smooth.floor_trial_residual_ratio
                == 3.1986895733209341e-5
            && !result.split.smooth.floor_trial_topology_exact;
    }
    const BoxKktState& state = result.constrained.state;
    bool reconstruction = result.constrained.passed;
    for (std::size_t i = 0;
         reconstruction && i < result.constrained.velocity.size(); ++i) {
        reconstruction = result.constrained.velocity[i].x
                == result.constrained.displacement[i].x / result.time_step
            && result.constrained.velocity[i].y
                == result.constrained.displacement[i].y / result.time_step
            && result.constrained.velocity[i].z
                == result.constrained.displacement[i].z / result.time_step;
    }
    result.constrained_gate = result.constrained.passed
        && state.lower_y_axes > 0
        && state.lateral_or_upper_axes == 0
        && reconstruction;
    return result;
}

struct B4BKDetached {
    BoxKktSolve constrained;
    bool passed = false;
    double position_error = 0.0;
    double velocity_error = 0.0;
};

B4BKDetached run_b4bk_detached() {
    B4BKDetached result;
    const SmokeFixture fixture = make_b4b_released_block_fixture();
    const double time_step = SMOKE_FRAME_TIME / 48.0;
    result.constrained = solve_box_kkt_step(fixture, time_step);
    std::vector<Vec3> expected_position = fixture.position;
    std::vector<Vec3> expected_velocity = fixture.velocity;
    for (std::size_t i = 0; i < expected_position.size(); ++i) {
        expected_velocity[i] += time_step * fixture.gravity;
        expected_position[i] += time_step * expected_velocity[i];
    }
    if (result.constrained.position.size() == expected_position.size()) {
        result.position_error = rms_difference(
            result.constrained.position, expected_position);
        result.velocity_error = rms_difference(
            result.constrained.velocity, expected_velocity);
    }
    const BoxKktState& state = result.constrained.state;
    result.passed = result.constrained.passed
        && result.position_error == 0.0
        && result.velocity_error == 0.0
        && state.smooth.support.active_centers == 0U
        && state.lower_axes == 0 && state.upper_axes == 0
        && norm(state.support_reaction) == 0.0
        && norm(state.contact_reaction) == 0.0
        && state.maximum_penetration == 0.0;
    return result;
}

void append_b4bk_state(std::ostringstream& output, const BoxKktState& value) {
    output << "{\"passed\":" << (value.passed ? "true" : "false")
           << ",\"objective_j\":" << value.smooth.total
           << ",\"pressure_active_centers\":"
           << value.smooth.support.active_centers
           << ",\"lower_axes\":" << value.lower_axes
           << ",\"upper_axes\":" << value.upper_axes
           << ",\"lower_y_axes\":" << value.lower_y_axes
           << ",\"lateral_or_upper_axes\":"
           << value.lateral_or_upper_axes
           << ",\"projected_impulse_residual_n_s\":"
           << value.projected_impulse_residual
           << ",\"reaction_limit_n_s\":" << value.reaction_limit
           << ",\"minimum_multiplier_n\":" << value.minimum_multiplier
           << ",\"complementarity_n_m\":" << value.complementarity
           << ",\"maximum_penetration_m\":" << value.maximum_penetration
           << ",\"support_translation_closure\":"
           << value.support_translation_closure
           << ",\"contact_closure_n_s\":" << value.contact_closure
           << ",\"ledger_absolute_n_s\":" << value.ledger_absolute
           << ",\"ledger_residual\":" << value.ledger_residual
           << ",\"actual_impulse_n_s\":";
    append_vec3(output, value.actual_impulse);
    output << ",\"fluid_pressure_impulse_n_s\":";
    append_vec3(output, value.fluid_pressure_impulse);
    output << ",\"support_reaction_n_s\":";
    append_vec3(output, value.support_reaction);
    output << ",\"fluid_contact_impulse_n_s\":";
    append_vec3(output, value.fluid_contact_impulse);
    output << ",\"contact_reaction_n_s\":";
    append_vec3(output, value.contact_reaction);
    output << '}';
}

void append_b4bk_solve(std::ostringstream& output, const BoxKktSolve& value) {
    output << "{\"passed\":" << (value.passed ? "true" : "false")
           << ",\"failure\":\"" << value.failure
           << "\",\"outer_trials\":" << value.outer_trials
           << ",\"accepted_trials\":" << value.accepted_trials
           << ",\"rejected_trials\":" << value.rejected_trials
           << ",\"hvp_calls\":" << value.hvp_calls
           << ",\"negative_curvature_exits\":"
           << value.negative_curvature_exits
           << ",\"projected_trials\":" << value.projected_trials
           << ",\"active_set_changes\":" << value.active_set_changes
           << ",\"floor_merit_trials\":" << value.floor_merit_trials
           << ",\"floor_merit_accepts\":" << value.floor_merit_accepts
           << ",\"initial_objective_j\":" << value.initial_objective
           << ",\"objective_forward_bound_j\":"
           << value.objective_forward_bound
           << ",\"state\":";
    append_b4bk_state(output, value.state);
    output << '}';
}

} // namespace

SplitBoundaryReport run_box_contact_kkt_controls() {
    const SplitBoundaryReport parent = run_tiny_pressure_corpus_controls();
    const bool parent_exact = !parent.passed
        && sha256_hex(parent.json)
            == "fef0035a7e0d005358d08d16ed134a656d73127c5ba215d2663f9feb95bdb5e3";
    const SmokeFixture fixture = make_b4b_supported_column_fixture();
    constexpr std::array<int, 3> counts = {48, 96, 192};
    std::array<B4BKReplay, 3> replay;
    bool replay_exact = true;
    bool constrained_passed = true;
    if (parent_exact) {
        for (std::size_t i = 0; i < counts.size(); ++i) {
            replay[i] = run_b4bk_replay(fixture, counts[i]);
            replay_exact = replay_exact && replay[i].split_exact;
            constrained_passed = constrained_passed
                && replay[i].constrained_gate;
        }
    } else {
        replay_exact = false;
        constrained_passed = false;
    }
    const B4BKDetached detached = parent_exact
        ? run_b4bk_detached() : B4BKDetached{};
    const bool passed = parent_exact && replay_exact
        && constrained_passed && detached.passed;
    std::string first_failure;
    if (!parent_exact) {
        first_failure = "NSR3B4B_PARENT";
    } else if (!replay_exact) {
        first_failure = "SPLIT_REPLAY";
    } else if (!constrained_passed) {
        for (std::size_t i = 0; i < replay.size(); ++i) {
            if (!replay[i].constrained_gate) {
                first_failure = "P1_KKT_"
                    + std::to_string(replay[i].substeps_per_frame)
                    + ':' + replay[i].constrained.failure;
                break;
            }
        }
    } else if (!detached.passed) {
        first_failure = "P2_DETACHED_NEGATIVE:"
            + detached.constrained.failure;
    }
    const std::string disposition = passed
        ? "BOX_CONTACT_KKT_CANDIDATE"
        : "BOX_CONTACT_KKT_REJECTED";
    std::ostringstream material;
    material << std::setprecision(17)
             << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
             << disposition;
    for (const B4BKReplay& value : replay) {
        material << '|' << value.substeps_per_frame << ':'
                 << value.split_exact << ':' << value.constrained_gate << ':'
                 << value.constrained.state.projected_impulse_residual << ':'
                 << value.constrained.state.ledger_residual << ':'
                 << value.constrained.state.lower_y_axes;
    }
    material << "|D:" << detached.passed << ':'
             << detached.position_error << ':' << detached.velocity_error;
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4bk_contact_kkt.v1\""
           << ",\"identity\":\"box-contact-kkt-discriminator-r0\""
           << ",\"parent_b4b_result_sha256\":\"59d7f2436f8bbbea974b47f4109d038ed09251012340accd3b0e0664b6ee2b16\""
           << ",\"parent_b4b_raw_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"disposition\":\"" << disposition << '"'
           << ",\"p1_replays\":[";
    for (std::size_t i = 0; i < replay.size(); ++i) {
        if (i != 0U) {
            report << ',';
        }
        const B4BKReplay& value = replay[i];
        report << "{\"substeps_per_frame\":"
               << value.substeps_per_frame
               << ",\"time_step_s\":" << value.time_step
               << ",\"split_exact\":"
               << (value.split_exact ? "true" : "false")
               << ",\"split\":{\"passed\":"
               << (value.split.passed ? "true" : "false")
               << ",\"failure\":\"" << value.split.failure
               << "\",\"reaction_defect_n_s\":"
               << value.split.smooth.reaction_stationarity_defect
               << ",\"reaction_limit_n_s\":"
               << value.split.smooth.reaction_mixed_limit
               << ",\"predicted_reduction_j\":"
               << value.split.smooth.last_predicted_reduction
               << ",\"energy_floor_j\":"
               << value.split.smooth.numerical_energy_floor
               << ",\"floor_residual_ratio\":"
               << value.split.smooth.floor_trial_residual_ratio
               << ",\"floor_topology_exact\":"
               << (value.split.smooth.floor_trial_topology_exact
                    ? "true" : "false") << "},\"constrained_gate\":"
               << (value.constrained_gate ? "true" : "false")
               << ",\"constrained\":";
        append_b4bk_solve(report, value.constrained);
        report << '}';
    }
    report << "],\"p2_detached\":{\"status\":\""
           << (detached.passed ? "PASS" : "FAIL")
           << "\",\"position_error_m\":" << detached.position_error
           << ",\"velocity_error_m_s\":" << detached.velocity_error
           << ",\"constrained\":";
    append_b4bk_solve(report, detached.constrained);
    report << "},\"candidate_selected\":"
           << (passed ? "true" : "false")
           << ",\"b4b_r1_contract_design_authorized\":"
           << (passed ? "true" : "false")
           << ",\"full_trajectory_execution_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"production_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(material.str()) << "\"}";
    return {passed, report.str()};
}

namespace {

bool b4bk1_face_gate(const B4BKReplay& value) {
    constexpr std::array<int, 6> expected = {12, 12, 16, 0, 12, 12};
    const BoxKktState& state = value.constrained.state;
    const double x_pair = state.face_fluid_impulse[0]
        + state.face_fluid_impulse[1];
    const double z_pair = state.face_fluid_impulse[4]
        + state.face_fluid_impulse[5];
    const Vec3 reconstructed{
        x_pair,
        state.face_fluid_impulse[2] + state.face_fluid_impulse[3],
        z_pair,
    };
    return value.split_exact && value.constrained.passed
        && state.face_counts == expected
        && state.face_counts[3] == 0
        && state.face_multiplier_sum[3] == 0.0
        && std::abs(x_pair) <= 1.0e-12
        && std::abs(z_pair) <= 1.0e-12
        && norm(reconstructed - state.fluid_contact_impulse) <= 1.0e-12
        && state.face_fluid_impulse[2] >= 0.0;
}

void append_b4bk1_faces(
    std::ostringstream& output, const BoxKktState& state) {
    output << "{\"counts\":[";
    for (std::size_t i = 0; i < state.face_counts.size(); ++i) {
        if (i != 0U) {
            output << ',';
        }
        output << state.face_counts[i];
    }
    output << "],\"multiplier_sum_n\":[";
    for (std::size_t i = 0; i < state.face_multiplier_sum.size(); ++i) {
        if (i != 0U) {
            output << ',';
        }
        output << state.face_multiplier_sum[i];
    }
    output << "],\"fluid_impulse_n_s\":[";
    for (std::size_t i = 0; i < state.face_fluid_impulse.size(); ++i) {
        if (i != 0U) {
            output << ',';
        }
        output << state.face_fluid_impulse[i];
    }
    const Vec3 reconstructed{
        state.face_fluid_impulse[0] + state.face_fluid_impulse[1],
        state.face_fluid_impulse[2] + state.face_fluid_impulse[3],
        state.face_fluid_impulse[4] + state.face_fluid_impulse[5],
    };
    output << "],\"reconstructed_impulse_n_s\":";
    append_vec3(output, reconstructed);
    output << ",\"aggregate_error_n_s\":"
           << norm(reconstructed - state.fluid_contact_impulse)
           << ",\"x_pair_closure_n_s\":" << std::abs(reconstructed.x)
           << ",\"z_pair_closure_n_s\":" << std::abs(reconstructed.z)
           << '}';
}

} // namespace

SplitBoundaryReport run_box_contact_kkt_face_controls() {
    const SplitBoundaryReport parent = run_box_contact_kkt_controls();
    const bool parent_exact = !parent.passed
        && sha256_hex(parent.json)
            == "b110585a9e8894126666c4f5c941b447361b15e3ea771deebdf0e760afbec716";
    const SmokeFixture fixture = make_b4b_supported_column_fixture();
    constexpr std::array<int, 3> counts = {48, 96, 192};
    std::array<B4BKReplay, 3> replay;
    bool face_passed = parent_exact;
    if (parent_exact) {
        for (std::size_t i = 0; i < counts.size(); ++i) {
            replay[i] = run_b4bk_replay(fixture, counts[i]);
            face_passed = face_passed && b4bk1_face_gate(replay[i]);
        }
    }
    const B4BKDetached detached = parent_exact
        ? run_b4bk_detached() : B4BKDetached{};
    const bool passed = parent_exact && face_passed && detached.passed;
    std::string first_failure;
    if (!parent_exact) {
        first_failure = "NSR3B4BK_PARENT";
    } else if (!face_passed) {
        for (std::size_t i = 0; i < replay.size(); ++i) {
            if (!b4bk1_face_gate(replay[i])) {
                first_failure = "P1_FACE_"
                    + std::to_string(replay[i].substeps_per_frame);
                break;
            }
        }
    } else if (!detached.passed) {
        first_failure = "P2_DETACHED_NEGATIVE";
    }
    const std::string disposition = passed
        ? "BOX_CONTACT_KKT_CANDIDATE"
        : "BOX_CONTACT_KKT_FACE_REPAIR_REJECTED";
    std::ostringstream material;
    material << std::setprecision(17)
             << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
             << disposition;
    for (const B4BKReplay& value : replay) {
        material << '|' << value.substeps_per_frame << ':'
                 << b4bk1_face_gate(value);
        for (std::size_t face = 0; face < 6U; ++face) {
            material << ':' << value.constrained.state.face_counts[face]
                     << ':'
                     << value.constrained.state.face_fluid_impulse[face];
        }
    }
    material << "|D:" << detached.passed;
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4bk1_contact_face.v1\""
           << ",\"identity\":\"box-contact-kkt-discriminator-r1-face-symmetry\""
           << ",\"parent_b4bk_result_sha256\":\"7cb256e5b1c62db865c7230a03a94a9553b07f1112a7e9552c78bd6b273a9f93\""
           << ",\"parent_b4bk_raw_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"disposition\":\"" << disposition << '"'
           << ",\"p1_face_controls\":[";
    for (std::size_t i = 0; i < replay.size(); ++i) {
        if (i != 0U) {
            report << ',';
        }
        report << "{\"substeps_per_frame\":"
               << replay[i].substeps_per_frame
               << ",\"status\":\""
               << (b4bk1_face_gate(replay[i]) ? "PASS" : "FAIL")
               << "\",\"faces\":";
        append_b4bk1_faces(report, replay[i].constrained.state);
        report << ",\"kkt\":";
        append_b4bk_state(report, replay[i].constrained.state);
        report << '}';
    }
    report << "],\"p2_detached_exact\":"
           << (detached.passed ? "true" : "false")
           << ",\"candidate_selected\":"
           << (passed ? "true" : "false")
           << ",\"b4b_r1_contract_design_authorized\":"
           << (passed ? "true" : "false")
           << ",\"full_trajectory_execution_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"production_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(material.str()) << "\"}";
    return {passed, report.str()};
}

namespace {

SmokeRun run_b4b1_interval(
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
    result.maximum_mechanical_energy = mechanical_energy(
        fixture, result.position, result.velocity);
    result.maximum_positive_density_strain =
        maximum_positive_density_strain(fixture, result.position);
    result.maximum_speed = maximum_speed(result.velocity);
    std::vector<Vec3> expected_position = start_position;
    std::vector<Vec3> expected_velocity = start_velocity;
    bool precontact = true;
    const double time_step = interval_duration / static_cast<double>(substeps);
    for (int substep = 0; substep < substeps; ++substep) {
        const BoxKktSolve solve = solve_box_kkt_step(
            fixture, result.position, result.velocity, time_step);
        if (!solve.passed) {
            result.failure = "SUBSTEP_" + std::to_string(substep)
                + ":KKT_SOLVE:" + solve.failure;
            result.outer_trials = solve.outer_trials;
            result.rejected_trials = solve.rejected_trials;
            result.hvp_calls = solve.hvp_calls;
            result.floor_merit_trials = solve.floor_merit_trials;
            result.floor_merit_accepts = solve.floor_merit_accepts;
            result.projected_trials = solve.projected_trials;
            result.active_set_changes = solve.active_set_changes;
            result.maximum_penetration = solve.state.maximum_penetration;
            result.maximum_ledger_residual = solve.state.ledger_residual;
            result.maximum_support_reaction_closure =
                solve.state.support_translation_closure;
            return result;
        }
        result.position = solve.position;
        result.velocity = solve.velocity;
        result.outer_trials += solve.outer_trials;
        result.rejected_trials += solve.rejected_trials;
        result.hvp_calls += solve.hvp_calls;
        result.negative_curvature_exits += solve.negative_curvature_exits;
        result.floor_merit_trials += solve.floor_merit_trials;
        result.floor_merit_accepts += solve.floor_merit_accepts;
        result.maximum_floor_accepts_per_solve = std::max(
            result.maximum_floor_accepts_per_solve,
            solve.floor_merit_accepts);
        result.projected_trials += solve.projected_trials;
        result.active_set_changes += solve.active_set_changes;
        const bool pressure_active =
            solve.state.smooth.support.active_centers > 0U;
        result.active_steps += pressure_active ? 1 : 0;
        result.inactive_steps += pressure_active ? 0 : 1;
        result.maximum_active_mixed_ratio = std::max(
            result.maximum_active_mixed_ratio,
            solve.state.projected_impulse_residual
                / std::max(solve.state.reaction_limit, 1.0e-300));
        result.maximum_pairs = std::max(result.maximum_pairs,
            solve.state.smooth.support.fluid_pairs
                + solve.state.smooth.support.boundary_pairs);
        result.maximum_penetration = std::max(
            result.maximum_penetration,
            solve.state.maximum_penetration);
        result.maximum_ledger_residual = std::max(
            result.maximum_ledger_residual,
            solve.state.ledger_residual);
        result.maximum_ledger_absolute = std::max(
            result.maximum_ledger_absolute,
            solve.state.ledger_absolute);
        result.maximum_support_reaction_closure = std::max(
            result.maximum_support_reaction_closure,
            solve.state.support_translation_closure);
        result.fluid_support_impulse +=
            solve.state.fluid_pressure_impulse;
        result.support_reaction += solve.state.support_reaction;
        result.fluid_contact_impulse +=
            solve.state.fluid_contact_impulse;
        result.contact_reaction += solve.state.contact_reaction;
        result.gravity_impulse += solve.state.gravity_impulse;
        result.maximum_mechanical_energy = std::max(
            result.maximum_mechanical_energy,
            mechanical_energy(fixture, result.position, result.velocity));
        result.maximum_positive_density_strain = std::max(
            result.maximum_positive_density_strain,
            maximum_positive_density_strain(fixture, result.position));
        result.maximum_speed = std::max(
            result.maximum_speed, maximum_speed(result.velocity));

        std::vector<std::pair<std::size_t, int>> contacts;
        for (std::size_t i = 0; i < solve.state.active_axis.size(); ++i) {
            for (int axis = 0; axis < 3; ++axis) {
                if (!solve.state.active_axis[i][static_cast<std::size_t>(axis)]) {
                    continue;
                }
                const double gradient = component(
                    solve.state.active_gradient[i], axis);
                if (gradient != 0.0) {
                    contacts.emplace_back(i,
                        2 * axis + (gradient < 0.0 ? 1 : 0));
                }
            }
        }
        std::sort(contacts.begin(), contacts.end());
        result.contact_events += static_cast<int>(contacts.size());
        result.cache_invalidations += static_cast<int>(contacts.size());
        result.terminal_contacts = contacts;
        for (std::size_t face = 0; face < 6U; ++face) {
            result.face_active_axes[face] += solve.state.face_counts[face];
            result.face_multiplier_sum[face] +=
                solve.state.face_multiplier_sum[face];
            result.face_fluid_impulse[face] +=
                solve.state.face_fluid_impulse[face];
        }
        if (precontact && contacts.empty()) {
            for (std::size_t i = 0; i < expected_position.size(); ++i) {
                expected_velocity[i] += time_step * fixture.gravity;
                expected_position[i] += time_step * expected_velocity[i];
            }
            ++result.precontact_steps;
            result.precontact_pressure_violations += pressure_active ? 1 : 0;
            result.maximum_precontact_support_reaction = std::max(
                result.maximum_precontact_support_reaction,
                norm(solve.state.support_reaction));
            result.maximum_precontact_position_error = std::max(
                result.maximum_precontact_position_error,
                rms_difference(result.position, expected_position));
            result.maximum_precontact_velocity_error = std::max(
                result.maximum_precontact_velocity_error,
                rms_difference(result.velocity, expected_velocity));
            const Vec3 mean_velocity = average_values(result.velocity);
            for (Vec3 value : result.velocity) {
                result.maximum_precontact_velocity_spread = std::max(
                    result.maximum_precontact_velocity_spread,
                    norm(value - mean_velocity));
            }
        }
        if (!contacts.empty()) {
            if (!std::isfinite(result.first_contact_time)) {
                result.first_contact_time = interval_start
                    + static_cast<double>(substep + 1) * time_step;
            }
            precontact = false;
        }
    }
    const std::size_t maximum_pairs = fixture.maximum_pairs > 0U
        ? fixture.maximum_pairs
        : 80U * (fixture.position.size() + fixture.boundary.size());
    result.passed = result.contact_events == result.cache_invalidations
        && result.maximum_pairs <= maximum_pairs
        && fixture.position.size() + fixture.boundary.size()
            <= fixture.maximum_participants
        && result.maximum_penetration <= 1.0e-12
        && result.maximum_ledger_residual <= 1.0e-9
        && result.maximum_support_reaction_closure <= 1.0e-10;
    if (!result.passed) {
        result.failure = "RUN_KKT_GATE";
    }
    return result;
}

SmokeController run_b4b1_controller(
    const SmokeFixture& fixture, bool contact_forecast = false) {
    SmokeController result;
    result.position = fixture.position;
    result.velocity = fixture.velocity;
    bool global_precontact = true;
    for (int frame_index = 0; frame_index < fixture.macro_frames;
         ++frame_index) {
        SmokeFrame frame;
        frame.frame = frame_index;
        const Evaluation frame_state = evaluate(
            result.position, fixture.boundary);
        frame.active_centers = static_cast<int>(frame_state.active_centers);
        std::vector<Vec3> spectrum_position;
        if (frame.active_centers > 0) {
            frame.spectrum_source = "START_ACTIVE";
            spectrum_position = result.position;
        } else if (contact_forecast) {
            std::vector<Vec3> predicted(result.position.size());
            for (std::size_t i = 0; i < predicted.size(); ++i) {
                predicted[i] = SMOKE_FRAME_TIME
                    * (result.velocity[i]
                        + SMOKE_FRAME_TIME * fixture.gravity);
            }
            predicted = clamp_box_displacement(
                fixture, result.position, predicted);
            const std::vector<Vec3> projected = materialize_displacement(
                result.position, predicted);
            const Evaluation forecast_state = evaluate(
                projected, fixture.boundary);
            if (forecast_state.active_centers > 0U) {
                frame.spectrum_source = "FORECAST_ACTIVE";
                spectrum_position = projected;
            } else {
                frame.spectrum_source = "INACTIVE_EXACT";
            }
        } else {
            frame.spectrum_source = "START_INACTIVE";
        }
        if (!spectrum_position.empty()) {
            const SpectralEstimate spectrum = boundary_pressure_spectrum(
                spectrum_position, fixture.boundary);
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
            levels.push_back(run_b4b1_interval(fixture,
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
                frame.gate = smoke_gate(
                    levels[static_cast<std::size_t>(level - 1)],
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
            result.outer_trials += run.outer_trials;
            result.rejected_trials += run.rejected_trials;
            result.floor_merit_trials += run.floor_merit_trials;
            result.floor_merit_accepts += run.floor_merit_accepts;
            result.projected_trials += run.projected_trials;
            result.active_set_changes += run.active_set_changes;
            result.active_steps += run.active_steps;
            result.inactive_steps += run.inactive_steps;
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
        result.accepted_active_steps += accepted.active_steps;
        result.accepted_inactive_steps += accepted.inactive_steps;
        result.maximum_pairs = std::max(
            result.maximum_pairs, accepted.maximum_pairs);
        result.maximum_penetration = std::max(
            result.maximum_penetration, accepted.maximum_penetration);
        result.maximum_ledger_residual = std::max(
            result.maximum_ledger_residual,
            accepted.maximum_ledger_residual);
        result.maximum_support_reaction_closure = std::max(
            result.maximum_support_reaction_closure,
            accepted.maximum_support_reaction_closure);
        result.maximum_active_mixed_ratio = std::max(
            result.maximum_active_mixed_ratio,
            accepted.maximum_active_mixed_ratio);
        result.support_reaction += accepted.support_reaction;
        result.contact_reaction += accepted.contact_reaction;
        result.gravity_impulse += accepted.gravity_impulse;
        result.maximum_mechanical_energy = std::max(
            result.maximum_mechanical_energy,
            accepted.maximum_mechanical_energy);
        result.maximum_positive_density_strain = std::max(
            result.maximum_positive_density_strain,
            accepted.maximum_positive_density_strain);
        result.maximum_speed = std::max(
            result.maximum_speed, accepted.maximum_speed);
        for (std::size_t face = 0; face < 6U; ++face) {
            frame.face_active_axes[face] =
                accepted.face_active_axes[face];
            frame.face_multiplier_sum[face] =
                accepted.face_multiplier_sum[face];
            frame.face_fluid_impulse[face] =
                accepted.face_fluid_impulse[face];
            result.face_active_axes[face] +=
                accepted.face_active_axes[face];
            result.face_multiplier_sum[face] +=
                accepted.face_multiplier_sum[face];
            result.face_fluid_impulse[face] +=
                accepted.face_fluid_impulse[face];
        }
        result.first_contact_time = std::min(
            result.first_contact_time, accepted.first_contact_time);
        result.terminal_contacts = accepted.terminal_contacts;
        if (global_precontact) {
            result.precontact_steps += accepted.precontact_steps;
            result.precontact_pressure_violations +=
                accepted.precontact_pressure_violations;
            result.maximum_precontact_support_reaction = std::max(
                result.maximum_precontact_support_reaction,
                accepted.maximum_precontact_support_reaction);
            result.maximum_precontact_position_error = std::max(
                result.maximum_precontact_position_error,
                accepted.maximum_precontact_position_error);
            result.maximum_precontact_velocity_error = std::max(
                result.maximum_precontact_velocity_error,
                accepted.maximum_precontact_velocity_error);
            result.maximum_precontact_velocity_spread = std::max(
                result.maximum_precontact_velocity_spread,
                accepted.maximum_precontact_velocity_spread);
            global_precontact = !std::isfinite(
                accepted.first_contact_time);
        }
        frame.position = result.position;
        frame.velocity = result.velocity;
        frame.passed = true;
        result.frames.push_back(frame);
    }
    result.passed = result.contact_events > 0
        && result.maximum_penetration <= 1.0e-12
        && result.maximum_ledger_residual <= 1.0e-9
        && result.maximum_support_reaction_closure <= 1.0e-10;
    if (!result.passed) {
        result.failure = "CONTROLLER_PHYSICAL_GATE";
    }
    return result;
}

B4BFixedTrajectory run_b4b1_fixed_trajectory(
    const SmokeFixture& fixture, int substeps_per_frame) {
    B4BFixedTrajectory result;
    result.substeps_per_frame = substeps_per_frame;
    result.position = fixture.position;
    result.velocity = fixture.velocity;
    for (int frame = 0; frame < fixture.macro_frames; ++frame) {
        SmokeRun run = run_b4b1_interval(fixture,
            result.position, result.velocity, substeps_per_frame,
            static_cast<double>(frame) * SMOKE_FRAME_TIME,
            SMOKE_FRAME_TIME);
        result.runs.push_back(run);
        if (!run.passed) {
            result.failure = "FRAME_" + std::to_string(frame)
                + ':' + run.failure;
            return result;
        }
        result.position = run.position;
        result.velocity = run.velocity;
        result.aggregates.push_back(b4b_aggregate(
            fixture, result.position, result.velocity));
        result.active_steps += run.active_steps;
        result.contact_events += run.contact_events;
        result.outer_trials += run.outer_trials;
        result.rejected_trials += run.rejected_trials;
        result.hvp_calls += run.hvp_calls;
        result.floor_merit_accepts += run.floor_merit_accepts;
        result.maximum_pairs = std::max(
            result.maximum_pairs, run.maximum_pairs);
        result.maximum_penetration = std::max(
            result.maximum_penetration, run.maximum_penetration);
        result.maximum_ledger_residual = std::max(
            result.maximum_ledger_residual, run.maximum_ledger_residual);
        result.maximum_support_reaction_closure = std::max(
            result.maximum_support_reaction_closure,
            run.maximum_support_reaction_closure);
        result.maximum_mechanical_energy = std::max(
            result.maximum_mechanical_energy,
            run.maximum_mechanical_energy);
        result.first_contact_time = std::min(
            result.first_contact_time, run.first_contact_time);
        result.terminal_contacts = run.terminal_contacts;
    }
    result.passed = true;
    return result;
}

B4BReference run_b4b1_reference(const SmokeFixture& fixture) {
    B4BReference result;
    for (std::size_t i = 0; i < B4B_REFERENCE_COUNTS.size(); ++i) {
        result.levels[i] = run_b4b1_fixed_trajectory(
            fixture, B4B_REFERENCE_COUNTS[i]);
    }
    B4BConvergence& value = result.convergence;
    for (std::size_t i = 0; i < 2U; ++i) {
        value.position_difference[i] = rms_difference(
            result.levels[i].position, result.levels[i + 1U].position);
        value.velocity_difference[i] = rms_difference(
            result.levels[i].velocity, result.levels[i + 1U].velocity);
        value.position_floor[i] = b4b_rms_floor(
            result.levels[i].position, result.levels[i + 1U].position);
        value.velocity_floor[i] = b4b_rms_floor(
            result.levels[i].velocity, result.levels[i + 1U].velocity);
    }
    value.position_floor_overlap =
        value.position_difference[0] <= value.position_floor[0]
        && value.position_difference[1] <= value.position_floor[1];
    value.velocity_floor_overlap =
        value.velocity_difference[0] <= value.velocity_floor[0]
        && value.velocity_difference[1] <= value.velocity_floor[1];
    if (value.position_difference[1] > 0.0) {
        value.position_ratio = value.position_difference[0]
            / value.position_difference[1];
    }
    if (value.velocity_difference[1] > 0.0) {
        value.velocity_ratio = value.velocity_difference[0]
            / value.velocity_difference[1];
    }
    const bool position_order = value.position_difference[0] > 0.0
        && value.position_difference[1] > 0.0
        && value.position_ratio >= 1.25 && value.position_ratio <= 2.75;
    const bool velocity_order = value.velocity_difference[0] > 0.0
        && value.velocity_difference[1] > 0.0
        && value.velocity_ratio >= 1.25 && value.velocity_ratio <= 2.75;
    value.passed = (position_order || value.position_floor_overlap)
        && (velocity_order || value.velocity_floor_overlap);
    result.passed = std::all_of(result.levels.begin(), result.levels.end(),
            [](const B4BFixedTrajectory& level) { return level.passed; })
        && value.passed;
    return result;
}

B4BCase run_b4b1_case(
    SmokeFixture fixture, bool released_block,
    bool contact_forecast = false) {
    B4BCase result;
    result.fixture = std::move(fixture);
    result.initial = b4b_aggregate(result.fixture,
        result.fixture.position, result.fixture.velocity);
    result.candidate = run_b4b1_controller(
        result.fixture, contact_forecast);
    if (!result.candidate.passed) {
        result.failure = "CANDIDATE:" + result.candidate.failure;
        return result;
    }
    result.reference = run_b4b1_reference(result.fixture);
    const bool reference_runs_passed = std::all_of(
        result.reference.levels.begin(), result.reference.levels.end(),
        [](const B4BFixedTrajectory& level) { return level.passed; });
    if (!reference_runs_passed) {
        result.failure = "REFERENCE_RUN";
        return result;
    }
    if (!result.reference.convergence.passed) {
        result.failure = "REFERENCE_CONVERGENCE";
        return result;
    }
    const B4BFixedTrajectory& fine = result.reference.levels[2];
    for (std::size_t frame = 0; frame < result.candidate.frames.size();
         ++frame) {
        result.comparisons.push_back(compare_b4b_frame(
            static_cast<int>(frame), result.fixture,
            result.candidate.frames[frame], fine.runs[frame],
            fine.aggregates[frame]));
    }
    result.comparison_passed = std::all_of(
        result.comparisons.begin(), result.comparisons.end(),
        [](const B4BFrameComparison& value) { return value.passed; });
    result.final = b4b_aggregate(result.fixture,
        result.candidate.position, result.candidate.velocity);
    result.lateral_drift = std::max(
        std::abs(result.final.center.x - result.initial.center.x),
        std::abs(result.final.center.z - result.initial.center.z));
    result.vertical_center_change = std::abs(
        result.final.center.y - result.initial.center.y);
    result.energy_allowance = 0.01 * std::max({
        std::abs(result.initial.mechanical),
        static_cast<double>(result.fixture.position.size())
            * MASS * (-result.fixture.gravity.y) * SPACING,
        1.0e-12,
    });
    result.energy_creation = std::max(0.0,
        result.candidate.maximum_mechanical_energy
            - result.initial.mechanical);
    result.contact_time_error = event_time_error(
        result.candidate.first_contact_time, fine.first_contact_time);
    result.contact_time_limit = b4b_contact_time_limit(result.candidate);
    result.terminal_contacts_exact = result.candidate.terminal_contacts
        == fine.terminal_contacts;
    const bool common_physical = result.initial.finite_values
        && result.final.finite_values
        && result.lateral_drift <= 1.0e-10
        && result.candidate.contact_events > 0
        && result.candidate.accepted_active_steps > 0
        && result.candidate.maximum_penetration <= 1.0e-12
        && result.candidate.maximum_ledger_residual <= 1.0e-9
        && result.candidate.maximum_support_reaction_closure <= 1.0e-10
        && result.energy_creation <= result.energy_allowance
        && result.terminal_contacts_exact;
    if (released_block) {
        result.physical_passed = common_physical
            && result.fixture.position.size() == 27U
            && result.candidate.precontact_steps > 0
            && result.candidate.precontact_pressure_violations == 0
            && result.candidate.maximum_precontact_support_reaction
                <= 1.0e-12
            && result.candidate.maximum_precontact_velocity_spread
                <= 1.0e-12
            && result.candidate.maximum_precontact_position_error
                <= 1.0e-12
            && result.candidate.maximum_precontact_velocity_error
                <= 1.0e-12
            && result.contact_time_error <= result.contact_time_limit;
    } else {
        result.physical_passed = common_physical
            && result.fixture.position.size() == 48U
            && result.vertical_center_change <= 0.05 * SPACING
            && result.candidate.maximum_positive_density_strain <= 1.0e-3
            && result.candidate.maximum_speed
                <= 0.01 * std::sqrt(KAPPA / MASS);
    }
    result.work_passed = b4b_work_gate(result);
    result.passed = result.comparison_passed
        && result.physical_passed && result.work_passed;
    if (!result.comparison_passed) {
        result.failure = "FRAME_COMPARISON";
    } else if (!result.physical_passed) {
        result.failure = "PHYSICAL_GATE";
    } else if (!result.work_passed) {
        result.failure = "WORK_GATE";
    }
    return result;
}

void append_b4b1_kkt_work(
    std::ostringstream& output, const B4BCase& value) {
    output << "{\"name\":\"" << value.fixture.name
           << "\",\"candidate\":{\"projected_trials\":"
           << value.candidate.projected_trials
           << ",\"active_set_changes\":"
           << value.candidate.active_set_changes
           << ",\"face_active_axes\":[";
    for (std::size_t face = 0; face < 6U; ++face) {
        if (face != 0U) {
            output << ',';
        }
        output << value.candidate.face_active_axes[face];
    }
    output << "],\"face_multiplier_sum_n\":[";
    for (std::size_t face = 0; face < 6U; ++face) {
        if (face != 0U) {
            output << ',';
        }
        output << value.candidate.face_multiplier_sum[face];
    }
    output << "],\"face_fluid_impulse_n_s\":[";
    for (std::size_t face = 0; face < 6U; ++face) {
        if (face != 0U) {
            output << ',';
        }
        output << value.candidate.face_fluid_impulse[face];
    }
    output << "],\"frames\":[";
    for (std::size_t i = 0; i < value.candidate.frames.size(); ++i) {
        if (i != 0U) {
            output << ',';
        }
        output << "{\"frame\":" << i << ",\"face_active_axes\":[";
        for (std::size_t face = 0; face < 6U; ++face) {
            if (face != 0U) {
                output << ',';
            }
            output << value.candidate.frames[i].face_active_axes[face];
        }
        output << "],\"embedded_gate\":{\"position_dx\":"
               << value.candidate.frames[i].gate.normalized_position_error
               << ",\"velocity_c\":"
               << value.candidate.frames[i].gate.normalized_velocity_error
               << ",\"kinetic_relative\":"
               << value.candidate.frames[i].gate.relative_kinetic_error
               << ",\"contact_time_error_s\":"
               << value.candidate.frames[i].gate.contact_time_error
               << ",\"passed\":"
               << (value.candidate.frames[i].gate.passed
                    ? "true" : "false")
               << "},\"face_multiplier_sum_n\":[";
        for (std::size_t face = 0; face < 6U; ++face) {
            if (face != 0U) {
                output << ',';
            }
            output << value.candidate.frames[i].face_multiplier_sum[face];
        }
        output << "],\"face_fluid_impulse_n_s\":[";
        for (std::size_t face = 0; face < 6U; ++face) {
            if (face != 0U) {
                output << ',';
            }
            output << value.candidate.frames[i].face_fluid_impulse[face];
        }
        output << "]}";
    }
    output << "]},\"reference\":[";
    for (std::size_t level = 0; level < value.reference.levels.size();
         ++level) {
        if (level != 0U) {
            output << ',';
        }
        int projected_trials = 0;
        int active_set_changes = 0;
        for (const SmokeRun& run : value.reference.levels[level].runs) {
            projected_trials += run.projected_trials;
            active_set_changes += run.active_set_changes;
        }
        output << "{\"substeps_per_frame\":"
               << value.reference.levels[level].substeps_per_frame
               << ",\"projected_trials\":" << projected_trials
               << ",\"active_set_changes\":" << active_set_changes
               << '}';
    }
    output << "]}";
}

} // namespace

SplitBoundaryReport run_tiny_pressure_contact_kkt_controls() {
    const SplitBoundaryReport parent = run_box_contact_kkt_face_controls();
    const bool parent_exact = parent.passed
        && sha256_hex(parent.json)
            == "eed7934a81dc451edc2eeaaee81dcb403e2bb94e6799edfdc1646cb7970b4bdd";
    std::vector<B4BCase> cases;
    std::string first_failure;
    if (!parent_exact) {
        first_failure = "NSR3B4BK1_PARENT";
    } else {
        cases.push_back(run_b4b1_case(
            make_b4b_supported_column_fixture(), false));
        if (!cases.back().passed) {
            first_failure = "P1_SUPPORTED_COLUMN:" + cases.back().failure;
        } else {
            cases.push_back(run_b4b1_case(
                make_b4b_released_block_fixture(), true));
            if (!cases.back().passed) {
                first_failure = "P2_RELEASED_BLOCK:" + cases.back().failure;
            }
        }
    }
    const bool passed = parent_exact && cases.size() == 2U
        && std::all_of(cases.begin(), cases.end(),
            [](const B4BCase& value) { return value.passed; });
    const std::string disposition = passed
        ? "TINY_PRESSURE_CONTACT_KKT_CANDIDATE"
        : "TINY_PRESSURE_CONTACT_KKT_REJECTED";
    std::ostringstream material;
    material << std::setprecision(17)
             << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
             << disposition;
    for (const B4BCase& value : cases) {
        material << '|' << value.fixture.name << ':' << value.passed
                 << ':' << value.candidate.accepted_substeps
                 << ':' << value.candidate.executed_substeps
                 << ':' << value.candidate.nonlinear_hvp_calls
                 << ':' << value.candidate.projected_trials
                 << ':' << value.candidate.active_set_changes
                 << ':' << value.candidate.maximum_ledger_residual
                 << ':' << value.energy_creation;
    }
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4b1_pressure_kkt.v1\""
           << ",\"identity\":\"tiny-pressure-water-corpus-r1-contact-kkt\""
           << ",\"parent_b4bk1_result_sha256\":\"48db28247059f4f61870ee8ff9bc680ebbb5e672039c5c199b11e14dbe980197\""
           << ",\"parent_b4bk1_raw_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"disposition\":\"" << disposition << '"'
           << ",\"cases\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0U) {
            report << ',';
        }
        append_b4b_case(report, cases[i]);
    }
    report << "],\"kkt_work\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0U) {
            report << ',';
        }
        append_b4b1_kkt_work(report, cases[i]);
    }
    report << ']'
           << ",\"candidate_selected\":"
           << (passed ? "true" : "false")
           << ",\"b4c_neighborhood_design_authorized\":"
           << (passed ? "true" : "false")
           << ",\"nominal_corpus_execution_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"production_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(material.str()) << "\"}";
    return {passed, report.str()};
}

namespace {

constexpr std::array<int, 9> B4BF_LEVELS = {
    1, 2, 4, 8, 16, 32, 48, 96, 192,
};

struct B4BFForecast {
    bool passed = false;
    int pressure_active_centers = 0;
    int spectral_hvp_calls = 0;
    double maximum_eigenvalue = 0.0;
    double maximum_eigenfrequency = 0.0;
    int substeps = 1;
    std::vector<Vec3> projected_position;
};

B4BFForecast b4bf_forecast(const SmokeFixture& fixture) {
    B4BFForecast result;
    std::vector<Vec3> predicted(fixture.position.size());
    for (std::size_t i = 0; i < predicted.size(); ++i) {
        predicted[i] = SMOKE_FRAME_TIME
            * (fixture.velocity[i]
                + SMOKE_FRAME_TIME * fixture.gravity);
    }
    const std::vector<Vec3> projected = clamp_box_displacement(
        fixture, fixture.position, predicted);
    result.projected_position = materialize_displacement(
        fixture.position, projected);
    const Evaluation state = evaluate(
        result.projected_position, fixture.boundary);
    result.pressure_active_centers = static_cast<int>(state.active_centers);
    if (result.pressure_active_centers > 0) {
        const SpectralEstimate spectrum = boundary_pressure_spectrum(
            result.projected_position, fixture.boundary);
        result.spectral_hvp_calls = spectrum.calls;
        result.maximum_eigenvalue = spectrum.maximum_eigenvalue;
        result.maximum_eigenfrequency = std::sqrt(
            std::max(result.maximum_eigenvalue, 0.0) / MASS);
        if (!spectrum.passed) {
            return result;
        }
        result.substeps = std::max(1,
            static_cast<int>(std::ceil(SMOKE_FRAME_TIME
                * result.maximum_eigenfrequency / SPECTRAL_TARGET)));
    }
    result.passed = result.substeps >= 1 && result.substeps <= 192
        && (result.pressure_active_centers == 0
            ? result.spectral_hvp_calls == 0 && result.substeps == 1
            : result.spectral_hvp_calls == 48
                && result.maximum_eigenvalue > 0.0);
    return result;
}

B4BFrameComparison b4bf_compare(
    int label, const SmokeFixture& fixture,
    const SmokeRun& candidate, const SmokeRun& reference) {
    SmokeFrame frame;
    frame.position = candidate.position;
    frame.velocity = candidate.velocity;
    return compare_b4b_frame(label, fixture, frame, reference,
        b4b_aggregate(fixture, reference.position, reference.velocity));
}

struct B4BFResult {
    bool passed = false;
    std::string failure;
    SmokeFixture p1;
    B4BFForecast p1_forecast;
    std::array<SmokeRun, 9> curve;
    std::array<SmokeGate, 8> adjacent;
    std::array<B4BFrameComparison, 9> comparison;
    SmokeRun forecast_coarse;
    SmokeRun forecast_fine;
    SmokeGate forecast_gate;
    B4BFrameComparison forecast_reference;
    SmokeFixture p2;
    B4BFForecast p2_forecast;
    SmokeRun p2_one_step;
    double p2_position_error = 0.0;
    double p2_velocity_error = 0.0;
    bool p2_exact = false;
};

B4BFResult run_b4bf() {
    B4BFResult result;
    result.p1 = make_b4b_supported_column_fixture();
    for (std::size_t i = 0; i < B4BF_LEVELS.size(); ++i) {
        result.curve[i] = run_b4b1_interval(result.p1,
            result.p1.position, result.p1.velocity,
            B4BF_LEVELS[i], 0.0, SMOKE_FRAME_TIME);
    }
    const SmokeRun& reference = result.curve.back();
    for (std::size_t i = 0; i + 1U < B4BF_LEVELS.size(); ++i) {
        result.adjacent[i] = smoke_gate(
            result.curve[i], result.curve[i + 1U]);
    }
    for (std::size_t i = 0; i < B4BF_LEVELS.size(); ++i) {
        result.comparison[i] = b4bf_compare(
            B4BF_LEVELS[i], result.p1, result.curve[i], reference);
    }
    result.p1_forecast = b4bf_forecast(result.p1);
    if (!result.p1_forecast.passed
        || 2 * result.p1_forecast.substeps > 192) {
        result.failure = "P1_FORECAST";
        return result;
    }
    result.forecast_coarse = run_b4b1_interval(result.p1,
        result.p1.position, result.p1.velocity,
        result.p1_forecast.substeps, 0.0, SMOKE_FRAME_TIME);
    result.forecast_fine = run_b4b1_interval(result.p1,
        result.p1.position, result.p1.velocity,
        2 * result.p1_forecast.substeps, 0.0, SMOKE_FRAME_TIME);
    result.forecast_gate = smoke_gate(
        result.forecast_coarse, result.forecast_fine);
    result.forecast_reference = b4bf_compare(
        result.p1_forecast.substeps * 2,
        result.p1, result.forecast_fine, reference);

    result.p2 = make_b4b_released_block_fixture();
    result.p2_forecast = b4bf_forecast(result.p2);
    result.p2_one_step = run_b4b1_interval(result.p2,
        result.p2.position, result.p2.velocity, 1, 0.0,
        SMOKE_FRAME_TIME);
    std::vector<Vec3> expected_position = result.p2.position;
    std::vector<Vec3> expected_velocity = result.p2.velocity;
    for (std::size_t i = 0; i < expected_position.size(); ++i) {
        expected_velocity[i] += SMOKE_FRAME_TIME * result.p2.gravity;
        expected_position[i] += SMOKE_FRAME_TIME * expected_velocity[i];
    }
    result.p2_position_error = rms_difference(
        result.p2_one_step.position, expected_position);
    result.p2_velocity_error = rms_difference(
        result.p2_one_step.velocity, expected_velocity);
    result.p2_exact = result.p2_forecast.passed
        && result.p2_forecast.pressure_active_centers == 0
        && result.p2_forecast.spectral_hvp_calls == 0
        && result.p2_forecast.substeps == 1
        && result.p2_one_step.passed
        && result.p2_position_error == 0.0
        && result.p2_velocity_error == 0.0
        && result.p2_one_step.contact_events == 0
        && result.p2_one_step.active_steps == 0
        && norm(result.p2_one_step.support_reaction) == 0.0
        && norm(result.p2_one_step.contact_reaction) == 0.0;
    const bool curve_passed = std::all_of(
        result.curve.begin(), result.curve.end(),
        [](const SmokeRun& value) { return value.passed; });
    result.passed = curve_passed
        && result.p1_forecast.pressure_active_centers > 0
        && result.p1_forecast.spectral_hvp_calls == 48
        && result.forecast_coarse.passed && result.forecast_fine.passed
        && result.forecast_gate.passed
        && result.forecast_reference.passed
        && result.p2_exact;
    if (!curve_passed) {
        result.failure = "P1_LEVEL_CURVE";
    } else if (!result.forecast_gate.passed) {
        result.failure = "P1_EMBEDDED_GATE";
    } else if (!result.forecast_reference.passed) {
        result.failure = "P1_REFERENCE_GATE";
    } else if (!result.p2_exact) {
        result.failure = "P2_DETACHED_NEGATIVE";
    }
    return result;
}

void append_b4bf_comparison(
    std::ostringstream& output, const B4BFrameComparison& value) {
    output << "{\"status\":\"" << (value.passed ? "PASS" : "FAIL")
           << "\",\"position_dx\":" << value.position_dx
           << ",\"velocity_c\":" << value.velocity_c
           << ",\"center_dx\":" << value.center_dx
           << ",\"q99_height_dx\":" << value.q99_height_dx
           << ",\"q99_front_dx\":" << value.q99_front_dx
           << ",\"kinetic_relative\":" << value.kinetic_relative
           << ",\"kinetic_absolute_j\":" << value.kinetic_absolute
           << '}';
}

void append_b4bf_gate(std::ostringstream& output, const SmokeGate& value) {
    output << "{\"status\":\"" << (value.passed ? "PASS" : "FAIL")
           << "\",\"position_dx\":" << value.normalized_position_error
           << ",\"velocity_c\":" << value.normalized_velocity_error
           << ",\"kinetic_relative\":" << value.relative_kinetic_error
           << ",\"contact_time_error_s\":" << value.contact_time_error
           << '}';
}

void append_b4bf_forecast(
    std::ostringstream& output, const B4BFForecast& value) {
    output << "{\"status\":\"" << (value.passed ? "PASS" : "FAIL")
           << "\",\"pressure_active_centers\":"
           << value.pressure_active_centers
           << ",\"spectral_hvp_calls\":" << value.spectral_hvp_calls
           << ",\"maximum_eigenvalue\":" << value.maximum_eigenvalue
           << ",\"maximum_eigenfrequency_rad_s\":"
           << value.maximum_eigenfrequency
           << ",\"selected_substeps\":" << value.substeps << '}';
}

} // namespace

SplitBoundaryReport run_contact_onset_forecast_controls() {
    const SplitBoundaryReport parent =
        run_tiny_pressure_contact_kkt_controls();
    const bool parent_exact = !parent.passed
        && sha256_hex(parent.json)
            == "2b40f5662f9aa38f2bc6bae35d4e9d2e21c9b0f967af40c1831562d7fa4c98c9";
    const B4BFResult value = parent_exact ? run_b4bf() : B4BFResult{};
    const bool passed = parent_exact && value.passed;
    std::string first_failure;
    if (!parent_exact) {
        first_failure = "NSR3B4B1_PARENT";
    } else {
        first_failure = value.failure;
    }
    const std::string disposition = passed
        ? "CONTACT_ONSET_SPECTRAL_FORECAST_CANDIDATE"
        : "CONTACT_ONSET_SPECTRAL_FORECAST_REJECTED";
    std::ostringstream material;
    material << std::setprecision(17)
             << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
             << disposition << '|' << value.p1_forecast.substeps << ':'
             << value.p1_forecast.maximum_eigenvalue << ':'
             << value.forecast_gate.relative_kinetic_error << ':'
             << value.forecast_reference.kinetic_relative << ':'
             << value.p2_exact;
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4bf_contact_forecast.v1\""
           << ",\"identity\":\"feasible-contact-onset-spectrum-r0\""
           << ",\"parent_b4b1_result_sha256\":\"11302033bacf1a3656c3b584f9db68f573e088b786c56ceea10b32dfee509a2e\""
           << ",\"parent_b4b1_raw_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"disposition\":\"" << disposition << '"'
           << ",\"p1_level_curve\":[";
    for (std::size_t i = 0; i < B4BF_LEVELS.size(); ++i) {
        if (i != 0U) {
            report << ',';
        }
        report << "{\"substeps\":" << B4BF_LEVELS[i]
               << ",\"run_passed\":"
               << (value.curve[i].passed ? "true" : "false")
               << ",\"outer_trials\":" << value.curve[i].outer_trials
               << ",\"hvp_calls\":" << value.curve[i].hvp_calls
               << ",\"projected_trials\":"
               << value.curve[i].projected_trials
               << ",\"versus_fixed192\":";
        append_b4bf_comparison(report, value.comparison[i]);
        if (i + 1U < B4BF_LEVELS.size()) {
            report << ",\"adjacent_gate\":";
            append_b4bf_gate(report, value.adjacent[i]);
        }
        report << '}';
    }
    report << "],\"p1_forecast\":";
    append_b4bf_forecast(report, value.p1_forecast);
    report << ",\"p1_forecast_pair\":{\"coarse_substeps\":"
           << value.p1_forecast.substeps
           << ",\"fine_substeps\":"
           << 2 * value.p1_forecast.substeps
           << ",\"embedded\":";
    append_b4bf_gate(report, value.forecast_gate);
    report << ",\"fine_versus_fixed192\":";
    append_b4bf_comparison(report, value.forecast_reference);
    report << "},\"p2_forecast\":";
    append_b4bf_forecast(report, value.p2_forecast);
    report << ",\"p2_detached\":{\"status\":\""
           << (value.p2_exact ? "PASS" : "FAIL")
           << "\",\"position_error_m\":" << value.p2_position_error
           << ",\"velocity_error_m_s\":" << value.p2_velocity_error
           << "},\"candidate_selected\":"
           << (passed ? "true" : "false")
           << ",\"b4b2_contract_design_authorized\":"
           << (passed ? "true" : "false")
           << ",\"full_trajectory_execution_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"production_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(material.str()) << "\"}";
    return {passed, report.str()};
}

namespace {

void append_b4b2_forecast_work(
    std::ostringstream& output, const B4BCase& value) {
    output << "{\"name\":\"" << value.fixture.name
           << "\",\"frames\":[";
    for (std::size_t i = 0; i < value.candidate.frames.size(); ++i) {
        if (i != 0U) {
            output << ',';
        }
        const SmokeFrame& frame = value.candidate.frames[i];
        output << "{\"frame\":" << frame.frame
               << ",\"spectrum_source\":\"" << frame.spectrum_source
               << "\",\"pressure_active_at_start\":"
               << frame.active_centers
               << ",\"spectral_hvp_calls\":"
               << frame.spectral_hvp_calls
               << ",\"maximum_eigenvalue\":"
               << frame.maximum_eigenvalue
               << ",\"initial_substeps\":" << frame.initial_substeps
               << ",\"accepted_substeps\":"
               << frame.accepted_substeps
               << ",\"refinement_depth\":"
               << frame.refinement_depth << '}';
    }
    output << "]}";
}

} // namespace

SplitBoundaryReport run_tiny_pressure_contact_forecast_controls() {
    const SplitBoundaryReport parent =
        run_contact_onset_forecast_controls();
    const bool parent_exact = parent.passed
        && sha256_hex(parent.json)
            == "dfd6b39d8c12da4e494ff2ed4de5590d7a08b2be598c2d411c076ad6f8f1b550";
    std::vector<B4BCase> cases;
    std::string first_failure;
    if (!parent_exact) {
        first_failure = "NSR3B4BF_PARENT";
    } else {
        cases.push_back(run_b4b1_case(
            make_b4b_supported_column_fixture(), false, true));
        if (!cases.back().passed) {
            first_failure = "P1_SUPPORTED_COLUMN:" + cases.back().failure;
        } else {
            cases.push_back(run_b4b1_case(
                make_b4b_released_block_fixture(), true, true));
            if (!cases.back().passed) {
                first_failure = "P2_RELEASED_BLOCK:" + cases.back().failure;
            }
        }
    }
    const bool passed = parent_exact && cases.size() == 2U
        && std::all_of(cases.begin(), cases.end(),
            [](const B4BCase& value) { return value.passed; });
    const std::string disposition = passed
        ? "TINY_PRESSURE_CONTACT_FORECAST_CANDIDATE"
        : "TINY_PRESSURE_CONTACT_FORECAST_REJECTED";
    std::ostringstream material;
    material << std::setprecision(17)
             << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
             << disposition;
    for (const B4BCase& value : cases) {
        material << '|' << value.fixture.name << ':' << value.passed
                 << ':' << value.candidate.accepted_substeps
                 << ':' << value.candidate.executed_substeps
                 << ':' << value.candidate.spectral_hvp_calls
                 << ':' << value.candidate.nonlinear_hvp_calls
                 << ':' << value.candidate.maximum_ledger_residual
                 << ':' << value.contact_time_error;
        for (const SmokeFrame& frame : value.candidate.frames) {
            material << ':' << frame.spectrum_source
                     << ':' << frame.initial_substeps
                     << ':' << frame.accepted_substeps;
        }
    }
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b4b2_pressure_forecast.v1\""
           << ",\"identity\":\"tiny-pressure-water-corpus-r2-contact-forecast\""
           << ",\"parent_b4bf_result_sha256\":\"c6d53131786bcfacdae1bbb1a4ee2076846019d037b62b167b2a76b4d28e146f\""
           << ",\"parent_b4bf_raw_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"disposition\":\"" << disposition << '"'
           << ",\"cases\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0U) {
            report << ',';
        }
        append_b4b_case(report, cases[i]);
    }
    report << "],\"forecast_work\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0U) {
            report << ',';
        }
        append_b4b2_forecast_work(report, cases[i]);
    }
    report << "],\"kkt_work\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0U) {
            report << ',';
        }
        append_b4b1_kkt_work(report, cases[i]);
    }
    report << ']'
           << ",\"candidate_selected\":"
           << (passed ? "true" : "false")
           << ",\"b4c_neighborhood_design_authorized\":"
           << (passed ? "true" : "false")
           << ",\"nominal_corpus_execution_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"production_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(material.str()) << "\"}";
    return {passed, report.str()};
}

namespace {

constexpr std::size_t B4C0_MAX_FLUID = 50000U;
constexpr std::size_t B4C0_MAX_SUPPORT = 32768U;
constexpr std::size_t B4C0_MAX_NEIGHBORS = 160U;

struct JointPoint {
    std::uint32_t id = 0;
    Vec3 position;
};

struct JointPair {
    std::uint32_t fluid = 0;
    std::uint32_t participant = 0;
};

bool operator==(const JointPair& lhs, const JointPair& rhs) {
    return lhs.fluid == rhs.fluid
        && lhs.participant == rhs.participant;
}

struct JointCellRecord {
    std::int64_t x = 0;
    std::int64_t y = 0;
    std::int64_t z = 0;
    std::size_t participant = 0;
};

struct JointCellRange {
    std::int64_t x = 0;
    std::int64_t y = 0;
    std::int64_t z = 0;
    std::size_t begin = 0;
    std::size_t end = 0;
};

struct JointNeighborhood {
    bool passed = false;
    std::string failure;
    std::vector<JointPoint> fluid;
    std::vector<JointPoint> support;
    std::vector<JointPair> pairs;
    std::vector<std::vector<std::uint32_t>> adjacency;
    std::size_t fluid_pairs = 0;
    std::size_t support_pairs = 0;
    std::size_t maximum_degree = 0;
    std::size_t distance_tests_per_pass = 0;
    std::size_t construction_distance_tests = 0;
    std::size_t pair_payload_capacity_bytes = 0;
    std::size_t adjacency_payload_capacity_bytes = 0;
};

struct JointCase {
    std::string name;
    bool passed = false;
    std::string failure;
    std::size_t fluid_samples = 0;
    std::size_t support_samples = 0;
    std::size_t fluid_pairs = 0;
    std::size_t support_pairs = 0;
    std::size_t maximum_degree = 0;
    std::size_t all_pair_candidate_checks = 0;
    std::size_t cell_distance_tests = 0;
    bool pair_exact = false;
    bool evaluation_exact = false;
    bool joint_hvp_exact = false;
    bool fluid_hvp_exact = false;
    bool repeat_exact = false;
    bool permutation_exact = false;
    bool work_reduced = false;
    std::string pair_sha256;
};

struct JointNegative {
    std::string name;
    std::string expected;
    std::string observed;
    std::size_t partial_pairs = 0;
    std::size_t partial_adjacency_rows = 0;
    bool passed = false;
};

bool joint_cell_less(
    std::int64_t ax, std::int64_t ay, std::int64_t az,
    std::int64_t bx, std::int64_t by, std::int64_t bz) {
    if (ax != bx) {
        return ax < bx;
    }
    if (ay != by) {
        return ay < by;
    }
    return az < bz;
}

bool joint_cell_coordinate(double value, std::int64_t& output) {
    const double coordinate = std::floor(value / HORIZON);
    const double low = static_cast<double>(
        std::numeric_limits<std::int64_t>::min() + 1);
    const double high = static_cast<double>(
        std::numeric_limits<std::int64_t>::max() - 1);
    if (!std::isfinite(coordinate)
        || coordinate < low || coordinate > high) {
        return false;
    }
    output = static_cast<std::int64_t>(coordinate);
    return true;
}

bool canonicalize_joint_points(
    const std::vector<JointPoint>& input,
    std::vector<JointPoint>& output) {
    output = input;
    std::sort(output.begin(), output.end(),
        [](const JointPoint& lhs, const JointPoint& rhs) {
            return lhs.id < rhs.id;
        });
    for (std::size_t i = 0; i < output.size(); ++i) {
        if (!finite(output[i].position)) {
            return false;
        }
        if (i > 0U && output[i - 1U].id == output[i].id) {
            return false;
        }
    }
    return true;
}

const Vec3& joint_position(
    const JointNeighborhood& value, std::size_t participant) {
    if (participant < value.fluid.size()) {
        return value.fluid[participant].position;
    }
    return value.support[participant - value.fluid.size()].position;
}

std::size_t checked_joint_pair_limit(std::size_t fluid) {
    if (fluid > std::numeric_limits<std::size_t>::max()
            / B4C0_MAX_NEIGHBORS) {
        return 0U;
    }
    return fluid * B4C0_MAX_NEIGHBORS;
}

JointNeighborhood build_joint_neighborhood(
    const std::vector<JointPoint>& fluid_input,
    const std::vector<JointPoint>& support_input,
    bool one_pass = false) {
    JointNeighborhood result;
    if (fluid_input.empty() || fluid_input.size() > B4C0_MAX_FLUID) {
        result.failure = "JOINT_FLUID_CAPACITY";
        return result;
    }
    if (support_input.size() > B4C0_MAX_SUPPORT) {
        result.failure = "JOINT_SUPPORT_CAPACITY";
        return result;
    }
    const std::size_t pair_limit = checked_joint_pair_limit(
        fluid_input.size());
    if (pair_limit == 0U) {
        result.failure = "JOINT_PAIR_CAPACITY";
        return result;
    }
    result.pair_payload_capacity_bytes = pair_limit * sizeof(JointPair);
    result.adjacency_payload_capacity_bytes = pair_limit
        * sizeof(std::uint32_t);
    if (!canonicalize_joint_points(fluid_input, result.fluid)
        || !canonicalize_joint_points(support_input, result.support)) {
        bool all_finite = true;
        const auto finite_point = [&](const JointPoint& value) {
            all_finite = all_finite && finite(value.position);
        };
        std::for_each(fluid_input.begin(), fluid_input.end(), finite_point);
        std::for_each(support_input.begin(), support_input.end(), finite_point);
        result.failure = all_finite
            ? "JOINT_DUPLICATE_ID" : "JOINT_POSITION_INVALID";
        result.fluid.clear();
        result.support.clear();
        return result;
    }
    if (result.fluid.size() > std::numeric_limits<std::size_t>::max()
            - result.support.size()) {
        result.failure = "JOINT_PAIR_CAPACITY";
        result.fluid.clear();
        result.support.clear();
        return result;
    }
    const std::size_t total = result.fluid.size() + result.support.size();
    std::vector<JointCellRecord> records;
    records.reserve(total);
    for (std::size_t participant = 0; participant < total; ++participant) {
        const Vec3& position = joint_position(result, participant);
        JointCellRecord record;
        record.participant = participant;
        if (!joint_cell_coordinate(position.x, record.x)
            || !joint_cell_coordinate(position.y, record.y)
            || !joint_cell_coordinate(position.z, record.z)) {
            result.failure = "JOINT_POSITION_INVALID";
            result.fluid.clear();
            result.support.clear();
            return result;
        }
        records.push_back(record);
    }
    std::sort(records.begin(), records.end(),
        [](const JointCellRecord& lhs, const JointCellRecord& rhs) {
            if (joint_cell_less(
                    lhs.x, lhs.y, lhs.z, rhs.x, rhs.y, rhs.z)) {
                return true;
            }
            if (joint_cell_less(
                    rhs.x, rhs.y, rhs.z, lhs.x, lhs.y, lhs.z)) {
                return false;
            }
            return lhs.participant < rhs.participant;
        });
    std::vector<JointCellRange> ranges;
    ranges.reserve(records.size());
    for (std::size_t begin = 0; begin < records.size();) {
        std::size_t end = begin + 1U;
        while (end < records.size()
            && records[end].x == records[begin].x
            && records[end].y == records[begin].y
            && records[end].z == records[begin].z) {
            ++end;
        }
        ranges.push_back({records[begin].x, records[begin].y,
            records[begin].z, begin, end});
        begin = end;
    }

    const auto visit = [&](const auto& callback,
                           std::size_t& distance_tests) {
        for (std::size_t center = 0; center < result.fluid.size(); ++center) {
            std::int64_t cx = 0;
            std::int64_t cy = 0;
            std::int64_t cz = 0;
            if (!joint_cell_coordinate(result.fluid[center].position.x, cx)
                || !joint_cell_coordinate(result.fluid[center].position.y, cy)
                || !joint_cell_coordinate(result.fluid[center].position.z, cz)) {
                return false;
            }
            for (int dx = -1; dx <= 1; ++dx) {
                for (int dy = -1; dy <= 1; ++dy) {
                    for (int dz = -1; dz <= 1; ++dz) {
                        const std::int64_t qx = cx + dx;
                        const std::int64_t qy = cy + dy;
                        const std::int64_t qz = cz + dz;
                        const auto range = std::lower_bound(
                            ranges.begin(), ranges.end(),
                            JointCellRange{qx, qy, qz, 0U, 0U},
                            [](const JointCellRange& lhs,
                               const JointCellRange& rhs) {
                                return joint_cell_less(
                                    lhs.x, lhs.y, lhs.z,
                                    rhs.x, rhs.y, rhs.z);
                            });
                        if (range == ranges.end()
                            || range->x != qx || range->y != qy
                            || range->z != qz) {
                            continue;
                        }
                        for (std::size_t slot = range->begin;
                             slot < range->end; ++slot) {
                            const std::size_t participant =
                                records[slot].participant;
                            if (participant < result.fluid.size()
                                && participant <= center) {
                                continue;
                            }
                            ++distance_tests;
                            if (norm(result.fluid[center].position
                                    - joint_position(result, participant))
                                    <= HORIZON
                                && !callback(center, participant)) {
                                return false;
                            }
                        }
                    }
                }
            }
        }
        return true;
    };

    std::vector<std::size_t> degree(result.fluid.size());
    std::size_t pair_count = 0U;
    if (one_pass) {
        result.pairs.reserve(pair_limit);
        const bool filled = visit(
            [&](std::size_t center, std::size_t participant) {
                if (pair_count == pair_limit) {
                    result.failure = "JOINT_PAIR_CAPACITY";
                    return false;
                }
                if (degree[center] == B4C0_MAX_NEIGHBORS
                    || (participant < result.fluid.size()
                        && degree[participant] == B4C0_MAX_NEIGHBORS)) {
                    result.failure = "JOINT_NEIGHBOR_CAPACITY";
                    return false;
                }
                ++pair_count;
                ++degree[center];
                if (participant < result.fluid.size()) {
                    ++degree[participant];
                    ++result.fluid_pairs;
                } else {
                    ++result.support_pairs;
                }
                result.pairs.push_back({
                    static_cast<std::uint32_t>(center),
                    static_cast<std::uint32_t>(participant)});
                return true;
            }, result.distance_tests_per_pass);
        if (!filled) {
            if (result.failure.empty()) {
                result.failure = "JOINT_POSITION_INVALID";
            }
            result.pairs.clear();
            result.fluid.clear();
            result.support.clear();
            return result;
        }
        result.construction_distance_tests =
            result.distance_tests_per_pass;
    } else {
        const bool counted = visit(
            [&](std::size_t center, std::size_t participant) {
                if (pair_count == pair_limit) {
                    result.failure = "JOINT_PAIR_CAPACITY";
                    return false;
                }
                ++pair_count;
                if (++degree[center] > B4C0_MAX_NEIGHBORS) {
                    result.failure = "JOINT_NEIGHBOR_CAPACITY";
                    return false;
                }
                if (participant < result.fluid.size()
                    && ++degree[participant] > B4C0_MAX_NEIGHBORS) {
                    result.failure = "JOINT_NEIGHBOR_CAPACITY";
                    return false;
                }
                return true;
            }, result.distance_tests_per_pass);
        if (!counted) {
            if (result.failure.empty()) {
                result.failure = "JOINT_POSITION_INVALID";
            }
            result.fluid.clear();
            result.support.clear();
            return result;
        }
        result.pairs.reserve(pair_count);
        std::size_t fill_tests = 0U;
        const bool filled = visit(
            [&](std::size_t center, std::size_t participant) {
                result.pairs.push_back({
                    static_cast<std::uint32_t>(center),
                    static_cast<std::uint32_t>(participant)});
                if (participant < result.fluid.size()) {
                    ++result.fluid_pairs;
                } else {
                    ++result.support_pairs;
                }
                return true;
            }, fill_tests);
        if (!filled || result.pairs.size() != pair_count) {
            result.failure = "JOINT_POSITION_INVALID";
            result.pairs.clear();
            result.fluid.clear();
            result.support.clear();
            return result;
        }
        result.construction_distance_tests =
            result.distance_tests_per_pass + fill_tests;
    }
    result.maximum_degree = *std::max_element(degree.begin(), degree.end());
    std::sort(result.pairs.begin(), result.pairs.end(),
        [](const JointPair& lhs, const JointPair& rhs) {
            return lhs.fluid < rhs.fluid
                || (lhs.fluid == rhs.fluid
                    && lhs.participant < rhs.participant);
        });
    if (std::adjacent_find(result.pairs.begin(), result.pairs.end())
        != result.pairs.end()) {
        result.failure = "JOINT_DUPLICATE_PAIR";
        result.pairs.clear();
        result.fluid.clear();
        result.support.clear();
        return result;
    }
    result.adjacency.resize(result.fluid.size());
    for (std::size_t center = 0; center < result.fluid.size(); ++center) {
        result.adjacency[center].reserve(degree[center]);
    }
    for (const JointPair pair : result.pairs) {
        result.adjacency[pair.fluid].push_back(pair.participant);
        if (pair.participant < result.fluid.size()) {
            result.adjacency[pair.participant].push_back(pair.fluid);
        }
    }
    for (std::vector<std::uint32_t>& row : result.adjacency) {
        std::sort(row.begin(), row.end());
    }
    result.passed = true;
    return result;
}

std::vector<JointPair> all_joint_pairs(
    const std::vector<JointPoint>& fluid,
    const std::vector<JointPoint>& support) {
    std::vector<JointPair> result;
    for (std::size_t i = 0; i < fluid.size(); ++i) {
        for (std::size_t j = i + 1U; j < fluid.size(); ++j) {
            if (norm(fluid[i].position - fluid[j].position) <= HORIZON) {
                result.push_back({static_cast<std::uint32_t>(i),
                    static_cast<std::uint32_t>(j)});
            }
        }
        for (std::size_t b = 0; b < support.size(); ++b) {
            if (norm(fluid[i].position - support[b].position) <= HORIZON) {
                result.push_back({static_cast<std::uint32_t>(i),
                    static_cast<std::uint32_t>(fluid.size() + b)});
            }
        }
    }
    return result;
}

std::string joint_pair_hash(const JointNeighborhood& value) {
    std::ostringstream material;
    material << "joint-fluid-support-r0|";
    for (const JointPair pair : value.pairs) {
        material << value.fluid[pair.fluid].id << ':';
        if (pair.participant < value.fluid.size()) {
            material << 'F' << ':'
                     << value.fluid[pair.participant].id;
        } else {
            material << 'S' << ':'
                     << value.support[
                            pair.participant - value.fluid.size()].id;
        }
        material << '|';
    }
    return sha256_hex(material.str());
}

Evaluation evaluate_joint(const JointNeighborhood& neighborhood) {
    Evaluation result;
    const std::size_t fluid_count = neighborhood.fluid.size();
    result.gradient.resize(fluid_count + neighborhood.support.size());
    result.density.assign(fluid_count, MASS * weight(0.0));
    for (const JointPair pair : neighborhood.pairs) {
        const double radius = norm(
            neighborhood.fluid[pair.fluid].position
            - joint_position(neighborhood, pair.participant));
        const double contribution = MASS * weight(radius);
        result.density[pair.fluid] += contribution;
        if (pair.participant < fluid_count) {
            result.density[pair.participant] += contribution;
            ++result.fluid_pairs;
        } else {
            ++result.boundary_pairs;
        }
    }
    for (std::size_t center = 0; center < fluid_count; ++center) {
        const double compression =
            result.density[center] / REST_DENSITY - 1.0;
        result.minimum_branch_margin = std::min(
            result.minimum_branch_margin, std::abs(compression));
        if (compression <= 0.0) {
            continue;
        }
        ++result.active_centers;
        result.energy += 0.5 * KAPPA * compression * compression;
        const double scale = KAPPA * compression
            * MASS / REST_DENSITY;
        for (std::size_t participant : neighborhood.adjacency[center]) {
            const Vec3 displacement =
                neighborhood.fluid[center].position
                - joint_position(neighborhood, participant);
            const double radius = norm(displacement);
            if (radius <= 1.0e-15 || radius > HORIZON) {
                continue;
            }
            const Vec3 pair = scale * weight_gradient(radius)
                * (displacement / radius);
            result.gradient[center] += pair;
            result.gradient[participant] += -pair;
        }
    }
    return result;
}

std::vector<Vec3> apply_joint_hessian(
    const JointNeighborhood& neighborhood,
    const std::vector<Vec3>& direction) {
    const std::size_t fluid_count = neighborhood.fluid.size();
    const std::size_t total = fluid_count + neighborhood.support.size();
    if (direction.size() != total) {
        throw std::invalid_argument("B4C0 HVP direction size mismatch");
    }
    const Evaluation state = evaluate_joint(neighborhood);
    std::vector<Vec3> result(total);
    for (std::size_t center = 0; center < fluid_count; ++center) {
        const double compression =
            state.density[center] / REST_DENSITY - 1.0;
        if (compression <= 0.0) {
            continue;
        }
        double compression_direction = 0.0;
        for (std::size_t participant : neighborhood.adjacency[center]) {
            const Vec3 displacement =
                neighborhood.fluid[center].position
                - joint_position(neighborhood, participant);
            const double radius = norm(displacement);
            if (radius <= 1.0e-15 || radius > HORIZON) {
                continue;
            }
            const Vec3 jacobian = MASS / REST_DENSITY
                * weight_gradient(radius) * (displacement / radius);
            compression_direction += dot(
                jacobian, direction[center] - direction[participant]);
        }
        for (std::size_t participant : neighborhood.adjacency[center]) {
            const Vec3 displacement =
                neighborhood.fluid[center].position
                - joint_position(neighborhood, participant);
            const double radius = norm(displacement);
            if (radius <= 1.0e-15 || radius > HORIZON) {
                continue;
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
        }
    }
    return result;
}

bool exact_vec3_values(
    const std::vector<Vec3>& lhs, const std::vector<Vec3>& rhs) {
    if (lhs.size() != rhs.size()) {
        return false;
    }
    for (std::size_t i = 0; i < lhs.size(); ++i) {
        if (lhs[i].x != rhs[i].x
            || lhs[i].y != rhs[i].y
            || lhs[i].z != rhs[i].z) {
            return false;
        }
    }
    return true;
}

bool exact_evaluation_values(
    const Evaluation& lhs, const Evaluation& rhs) {
    return lhs.energy == rhs.energy
        && lhs.density == rhs.density
        && exact_vec3_values(lhs.gradient, rhs.gradient)
        && lhs.active_centers == rhs.active_centers
        && lhs.fluid_pairs == rhs.fluid_pairs
        && lhs.boundary_pairs == rhs.boundary_pairs
        && lhs.minimum_branch_margin == rhs.minimum_branch_margin;
}

std::vector<JointPoint> tagged_points(const std::vector<Vec3>& position) {
    std::vector<JointPoint> result(position.size());
    for (std::size_t i = 0; i < position.size(); ++i) {
        result[i] = {static_cast<std::uint32_t>(i), position[i]};
    }
    return result;
}

std::vector<JointPoint> permute_joint_points(
    const std::vector<JointPoint>& input, int mode) {
    if (mode == 0 || input.size() < 2U) {
        return input;
    }
    std::vector<JointPoint> result;
    result.reserve(input.size());
    if (mode == 1) {
        result.assign(input.rbegin(), input.rend());
        return result;
    }
    std::size_t multiplier = 2U;
    while (std::gcd(multiplier, input.size()) != 1U) {
        ++multiplier;
    }
    for (std::size_t slot = 0; slot < input.size(); ++slot) {
        result.push_back(input[
            (multiplier * slot + 1U) % input.size()]);
    }
    return result;
}

std::size_t all_joint_candidate_checks(
    std::size_t fluid, std::size_t support) {
    return fluid * (fluid - 1U) / 2U + fluid * support;
}

JointCase run_joint_case(
    std::string name,
    const std::vector<JointPoint>& fluid,
    const std::vector<JointPoint>& support,
    bool require_work_reduction,
    bool one_pass = false) {
    JointCase result;
    result.name = std::move(name);
    const JointNeighborhood value =
        build_joint_neighborhood(fluid, support, one_pass);
    if (!value.passed) {
        result.failure = value.failure;
        return result;
    }
    result.fluid_samples = value.fluid.size();
    result.support_samples = value.support.size();
    result.fluid_pairs = value.fluid_pairs;
    result.support_pairs = value.support_pairs;
    result.maximum_degree = value.maximum_degree;
    result.all_pair_candidate_checks = all_joint_candidate_checks(
        value.fluid.size(), value.support.size());
    result.cell_distance_tests = value.construction_distance_tests;
    result.pair_sha256 = joint_pair_hash(value);
    const std::vector<JointPair> oracle_pairs = all_joint_pairs(
        value.fluid, value.support);
    result.pair_exact = value.pairs == oracle_pairs;
    std::vector<Vec3> canonical_fluid;
    std::vector<Vec3> canonical_support;
    canonical_fluid.reserve(value.fluid.size());
    canonical_support.reserve(value.support.size());
    for (const JointPoint& point : value.fluid) {
        canonical_fluid.push_back(point.position);
    }
    for (const JointPoint& point : value.support) {
        canonical_support.push_back(point.position);
    }
    const Evaluation oracle_evaluation = evaluate(
        canonical_fluid, canonical_support);
    const Evaluation cell_evaluation = evaluate_joint(value);
    result.evaluation_exact = exact_evaluation_values(
        oracle_evaluation, cell_evaluation);
    const std::vector<Vec3> joint_direction =
        deterministic_direction(value.fluid.size() + value.support.size());
    const std::vector<Vec3> oracle_joint_hvp = apply_hessian(
        canonical_fluid, canonical_support, joint_direction);
    const std::vector<Vec3> cell_joint_hvp = apply_joint_hessian(
        value, joint_direction);
    result.joint_hvp_exact = exact_vec3_values(
        oracle_joint_hvp, cell_joint_hvp);
    std::vector<Vec3> fluid_direction(
        value.fluid.size() + value.support.size());
    const std::vector<Vec3> fluid_only =
        deterministic_direction(value.fluid.size());
    std::copy(fluid_only.begin(), fluid_only.end(), fluid_direction.begin());
    result.fluid_hvp_exact = exact_vec3_values(
        apply_hessian(canonical_fluid, canonical_support, fluid_direction),
        apply_joint_hessian(value, fluid_direction));
    const JointNeighborhood repeated =
        build_joint_neighborhood(fluid, support, one_pass);
    result.repeat_exact = repeated.passed
        && repeated.pairs == value.pairs
        && joint_pair_hash(repeated) == result.pair_sha256;
    result.permutation_exact = true;
    for (int mode = 1; mode <= 2; ++mode) {
        const JointNeighborhood permuted = build_joint_neighborhood(
            permute_joint_points(fluid, mode),
            permute_joint_points(support, mode), one_pass);
        if (!permuted.passed || permuted.pairs != value.pairs
            || joint_pair_hash(permuted) != result.pair_sha256
            || !exact_evaluation_values(
                evaluate_joint(permuted), cell_evaluation)
            || !exact_vec3_values(
                apply_joint_hessian(permuted, joint_direction),
                cell_joint_hvp)) {
            result.permutation_exact = false;
        }
    }
    result.work_reduced = !require_work_reduction
        || result.cell_distance_tests < result.all_pair_candidate_checks;
    result.passed = result.pair_exact && result.evaluation_exact
        && result.joint_hvp_exact && result.fluid_hvp_exact
        && result.repeat_exact && result.permutation_exact
        && result.work_reduced
        && result.maximum_degree <= B4C0_MAX_NEIGHBORS
        && value.pairs.size() <= checked_joint_pair_limit(value.fluid.size());
    if (!result.passed) {
        result.failure = "CORRESPONDENCE_GATE";
    }
    return result;
}

JointNegative joint_negative(
    std::string name, std::string expected,
    const std::vector<JointPoint>& fluid,
    const std::vector<JointPoint>& support,
    bool one_pass = false) {
    const JointNeighborhood value =
        build_joint_neighborhood(fluid, support, one_pass);
    JointNegative result;
    result.name = std::move(name);
    result.expected = std::move(expected);
    result.observed = value.failure;
    result.partial_pairs = value.pairs.size();
    result.partial_adjacency_rows = value.adjacency.size();
    result.passed = !value.passed
        && result.observed == result.expected
        && result.partial_pairs == 0U
        && result.partial_adjacency_rows == 0U;
    return result;
}

std::array<JointNegative, 6> run_joint_negatives(bool one_pass = false) {
    std::array<JointNegative, 6> result;
    result[0] = joint_negative(
        "zero-fluid", "JOINT_FLUID_CAPACITY", {}, {}, one_pass);
    std::vector<JointPoint> too_many_fluid(B4C0_MAX_FLUID + 1U);
    result[1] = joint_negative(
        "fluid-capacity", "JOINT_FLUID_CAPACITY",
        too_many_fluid, {}, one_pass);
    std::vector<JointPoint> one_fluid(1U);
    std::vector<JointPoint> too_much_support(B4C0_MAX_SUPPORT + 1U);
    result[2] = joint_negative(
        "support-capacity", "JOINT_SUPPORT_CAPACITY",
        one_fluid, too_much_support, one_pass);
    std::vector<JointPoint> duplicate = {
        {7U, {0.0, 0.0, 0.0}},
        {7U, {SPACING, 0.0, 0.0}},
    };
    result[3] = joint_negative(
        "duplicate-fluid-id", "JOINT_DUPLICATE_ID",
        duplicate, {}, one_pass);
    std::vector<JointPoint> nonfinite = {
        {0U, {std::numeric_limits<double>::quiet_NaN(), 0.0, 0.0}},
    };
    result[4] = joint_negative(
        "nonfinite-position", "JOINT_POSITION_INVALID",
        nonfinite, {}, one_pass);
    std::vector<JointPoint> dense(B4C0_MAX_NEIGHBORS + 2U);
    for (std::size_t i = 0; i < dense.size(); ++i) {
        dense[i].id = static_cast<std::uint32_t>(i);
    }
    result[5] = joint_negative(
        "neighbor-capacity", "JOINT_NEIGHBOR_CAPACITY",
        dense, {}, one_pass);
    return result;
}

void append_joint_case(std::ostringstream& output, const JointCase& value) {
    output << "{\"name\":\"" << value.name
           << "\",\"status\":\"" << (value.passed ? "PASS" : "FAIL")
           << "\",\"failure\":\"" << value.failure
           << "\",\"fluid_samples\":" << value.fluid_samples
           << ",\"support_samples\":" << value.support_samples
           << ",\"fluid_pairs\":" << value.fluid_pairs
           << ",\"fluid_support_pairs\":" << value.support_pairs
           << ",\"maximum_degree\":" << value.maximum_degree
           << ",\"all_pair_candidate_checks\":"
           << value.all_pair_candidate_checks
           << ",\"cell_construction_distance_tests\":"
           << value.cell_distance_tests
           << ",\"pair_sha256\":\"" << value.pair_sha256
           << "\",\"pair_exact\":"
           << (value.pair_exact ? "true" : "false")
           << ",\"evaluation_exact\":"
           << (value.evaluation_exact ? "true" : "false")
           << ",\"joint_hvp_exact\":"
           << (value.joint_hvp_exact ? "true" : "false")
           << ",\"fluid_hvp_exact\":"
           << (value.fluid_hvp_exact ? "true" : "false")
           << ",\"repeat_exact\":"
           << (value.repeat_exact ? "true" : "false")
           << ",\"permutation_exact\":"
           << (value.permutation_exact ? "true" : "false")
           << ",\"work_reduced\":"
           << (value.work_reduced ? "true" : "false") << '}';
}

void append_joint_negative(
    std::ostringstream& output, const JointNegative& value) {
    output << "{\"name\":\"" << value.name
           << "\",\"status\":\"" << (value.passed ? "PASS" : "FAIL")
           << "\",\"expected\":\"" << value.expected
           << "\",\"observed\":\"" << value.observed
           << "\",\"partial_pairs\":" << value.partial_pairs
           << ",\"partial_adjacency_rows\":"
           << value.partial_adjacency_rows << '}';
}

} // namespace

SplitBoundaryReport run_joint_neighborhood_controls() {
    const SplitBoundaryReport parent =
        run_tiny_pressure_contact_forecast_controls();
    const bool parent_exact = parent.passed
        && sha256_hex(parent.json)
            == "8f9c0fb92216770a11f8f0b603bb66f5ce27858815d73b90b2e09266ec19e64b";
    std::vector<JointCase> cases;
    std::array<JointNegative, 6> negatives{};
    std::string first_failure;
    if (!parent_exact) {
        first_failure = "NSR3B4B2_PARENT";
    } else {
        const SmokeFixture p1 = make_b4b_supported_column_fixture();
        cases.push_back(run_joint_case(
            "p1-initial", tagged_points(p1.position),
            tagged_points(p1.boundary), true));
        std::vector<Vec3> prediction(p1.position.size());
        for (std::size_t i = 0; i < prediction.size(); ++i) {
            prediction[i] = SMOKE_FRAME_TIME
                * (p1.velocity[i] + SMOKE_FRAME_TIME * p1.gravity);
        }
        prediction = clamp_box_displacement(
            p1, p1.position, prediction);
        const std::vector<Vec3> forecast = materialize_displacement(
            p1.position, prediction);
        cases.push_back(run_joint_case(
            "p1-feasible-forecast", tagged_points(forecast),
            tagged_points(p1.boundary), true));
        const SmokeFixture p2 = make_b4b_released_block_fixture();
        cases.push_back(run_joint_case(
            "p2-detached-initial", tagged_points(p2.position),
            tagged_points(p2.boundary), true));
        const double below = std::nextafter(HORIZON, 0.0);
        const double above = std::nextafter(
            HORIZON, std::numeric_limits<double>::infinity());
        const std::vector<JointPoint> signed_fluid = {
            {9U, {0.0, 0.0, 0.0}},
            {2U, {below, 0.0, 0.0}},
            {5U, {-0.31, -0.15, 0.07}},
        };
        const std::vector<JointPoint> signed_support = {
            {9U, {-HORIZON, 0.0, 0.0}},
            {1U, {0.0, HORIZON, 0.0}},
            {4U, {above, 0.0, 0.0}},
        };
        cases.push_back(run_joint_case(
            "signed-cutoff", signed_fluid, signed_support, false));
        for (const JointCase& value : cases) {
            if (!value.passed && first_failure.empty()) {
                first_failure = value.name + ':' + value.failure;
            }
        }
        negatives = run_joint_negatives();
        for (const JointNegative& value : negatives) {
            if (!value.passed && first_failure.empty()) {
                first_failure = value.name + ":FAILURE_CONTROL";
            }
        }
    }
    const bool cases_passed = cases.size() == 4U
        && std::all_of(cases.begin(), cases.end(),
            [](const JointCase& value) { return value.passed; });
    const bool negatives_passed = parent_exact
        && std::all_of(negatives.begin(), negatives.end(),
            [](const JointNegative& value) { return value.passed; });
    const bool passed = parent_exact && cases_passed && negatives_passed;
    const std::string disposition = passed
        ? "JOINT_PRESSURE_NEIGHBORHOOD_CANDIDATE"
        : "JOINT_PRESSURE_NEIGHBORHOOD_REJECTED";
    std::ostringstream material;
    material << (passed ? "PASS|" : "FAIL|") << first_failure
             << '|' << disposition;
    for (const JointCase& value : cases) {
        material << '|' << value.name << ':' << value.pair_sha256
                 << ':' << value.fluid_pairs << ':' << value.support_pairs
                 << ':' << value.maximum_degree
                 << ':' << value.cell_distance_tests;
    }
    for (const JointNegative& value : negatives) {
        material << '|' << value.name << ':' << value.observed
                 << ':' << value.partial_pairs
                 << ':' << value.partial_adjacency_rows;
    }
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4c0_joint_neighborhood.v1\""
           << ",\"identity\":\"joint-fluid-support-cell-pairs-r0\""
           << ",\"parent_b4b2_result_sha256\":\"b79e537d44519391d9ec65f56134f409122c10ca7135d62f797b026d062b7286\""
           << ",\"parent_b4b2_raw_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"disposition\":\"" << disposition << '"'
           << ",\"limits\":{\"fluid_samples\":" << B4C0_MAX_FLUID
           << ",\"support_samples\":" << B4C0_MAX_SUPPORT
           << ",\"participants_per_fluid\":" << B4C0_MAX_NEIGHBORS
           << ",\"pairs_per_fluid\":" << B4C0_MAX_NEIGHBORS
           << "},\"cases\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0U) {
            report << ',';
        }
        append_joint_case(report, cases[i]);
    }
    report << "],\"failure_controls\":[";
    for (std::size_t i = 0; i < negatives.size(); ++i) {
        if (i != 0U) {
            report << ',';
        }
        append_joint_negative(report, negatives[i]);
    }
    report << "],\"candidate_selected\":"
           << (passed ? "true" : "false")
           << ",\"b4c1_pressure_tape_design_authorized\":"
           << (passed ? "true" : "false")
           << ",\"trajectory_substitution_authorized\":false"
           << ",\"canonical_continuation_authorized\":false"
           << ",\"nominal_corpus_execution_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"production_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(material.str()) << "\"}";
    return {passed, report.str()};
}

SplitBoundaryReport run_joint_neighborhood_one_pass_controls() {
    const SplitBoundaryReport parent = run_joint_neighborhood_controls();
    const bool parent_exact = !parent.passed
        && sha256_hex(parent.json)
            == "d811c8d55fd3ed1dad803d489b70eb9f2b1ca690296d4b4420c42ad44908f761";
    std::vector<JointCase> cases;
    std::array<JointNegative, 6> negatives{};
    std::string first_failure;
    if (!parent_exact) {
        first_failure = "NSR3B4C0_PARENT";
    } else {
        const SmokeFixture p1 = make_b4b_supported_column_fixture();
        cases.push_back(run_joint_case(
            "p1-initial", tagged_points(p1.position),
            tagged_points(p1.boundary), true, true));
        std::vector<Vec3> prediction(p1.position.size());
        for (std::size_t i = 0; i < prediction.size(); ++i) {
            prediction[i] = SMOKE_FRAME_TIME
                * (p1.velocity[i] + SMOKE_FRAME_TIME * p1.gravity);
        }
        prediction = clamp_box_displacement(
            p1, p1.position, prediction);
        cases.push_back(run_joint_case(
            "p1-feasible-forecast",
            tagged_points(materialize_displacement(
                p1.position, prediction)),
            tagged_points(p1.boundary), true, true));
        const SmokeFixture p2 = make_b4b_released_block_fixture();
        cases.push_back(run_joint_case(
            "p2-detached-initial", tagged_points(p2.position),
            tagged_points(p2.boundary), true, true));
        const double below = std::nextafter(HORIZON, 0.0);
        const double above = std::nextafter(
            HORIZON, std::numeric_limits<double>::infinity());
        cases.push_back(run_joint_case(
            "signed-cutoff",
            {{9U, {0.0, 0.0, 0.0}},
             {2U, {below, 0.0, 0.0}},
             {5U, {-0.31, -0.15, 0.07}}},
            {{9U, {-HORIZON, 0.0, 0.0}},
             {1U, {0.0, HORIZON, 0.0}},
             {4U, {above, 0.0, 0.0}}},
            false, true));
        for (const JointCase& value : cases) {
            if (!value.passed && first_failure.empty()) {
                first_failure = value.name + ':' + value.failure;
            }
        }
        negatives = run_joint_negatives(true);
        for (const JointNegative& value : negatives) {
            if (!value.passed && first_failure.empty()) {
                first_failure = value.name + ":FAILURE_CONTROL";
            }
        }
    }
    const bool cases_passed = cases.size() == 4U
        && std::all_of(cases.begin(), cases.end(),
            [](const JointCase& value) { return value.passed; });
    const bool negatives_passed = parent_exact
        && std::all_of(negatives.begin(), negatives.end(),
            [](const JointNegative& value) { return value.passed; });
    const std::size_t maximum_pair_payload = checked_joint_pair_limit(
        B4C0_MAX_FLUID) * sizeof(JointPair);
    const std::size_t maximum_adjacency_payload = checked_joint_pair_limit(
        B4C0_MAX_FLUID) * sizeof(std::uint32_t);
    const std::size_t maximum_row_headers = B4C0_MAX_FLUID
        * sizeof(std::vector<std::uint32_t>);
    const bool workspace_passed = sizeof(JointPair) == 8U
        && maximum_pair_payload == 64000000U
        && maximum_adjacency_payload == 32000000U;
    const bool passed = parent_exact && cases_passed
        && negatives_passed && workspace_passed;
    const std::string disposition = passed
        ? "JOINT_PRESSURE_NEIGHBORHOOD_CANDIDATE"
        : "JOINT_PRESSURE_NEIGHBORHOOD_REJECTED";
    if (!workspace_passed && first_failure.empty()) {
        first_failure = "WORKSPACE_LAYOUT";
    }
    std::ostringstream material;
    material << (passed ? "PASS|" : "FAIL|") << first_failure
             << '|' << disposition << '|' << maximum_pair_payload
             << ':' << maximum_adjacency_payload
             << ':' << maximum_row_headers;
    for (const JointCase& value : cases) {
        material << '|' << value.name << ':' << value.pair_sha256
                 << ':' << value.fluid_pairs << ':' << value.support_pairs
                 << ':' << value.maximum_degree
                 << ':' << value.cell_distance_tests;
    }
    for (const JointNegative& value : negatives) {
        material << '|' << value.name << ':' << value.observed
                 << ':' << value.partial_pairs
                 << ':' << value.partial_adjacency_rows;
    }
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4c0r_one_pass_neighborhood.v1\""
           << ",\"identity\":\"joint-fluid-support-cell-pairs-r1-one-pass\""
           << ",\"parent_b4c0_result_sha256\":\"44aec304e5fd7d3a54a3d74d9512630d4a76595b5d437fda42751fc4187e8a89\""
           << ",\"parent_b4c0_raw_exact\":"
           << (parent_exact ? "true" : "false")
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"disposition\":\"" << disposition << '"'
           << ",\"workspace\":{\"index_bytes\":" << sizeof(std::uint32_t)
           << ",\"pair_record_bytes\":" << sizeof(JointPair)
           << ",\"maximum_pair_payload_bytes\":"
           << maximum_pair_payload
           << ",\"maximum_adjacency_payload_bytes\":"
           << maximum_adjacency_payload
           << ",\"diagnostic_nested_row_header_bytes\":"
           << maximum_row_headers
           << ",\"compact_csr_required_before_nominal\":true}"
           << ",\"cases\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0U) {
            report << ',';
        }
        append_joint_case(report, cases[i]);
    }
    report << "],\"failure_controls\":[";
    for (std::size_t i = 0; i < negatives.size(); ++i) {
        if (i != 0U) {
            report << ',';
        }
        append_joint_negative(report, negatives[i]);
    }
    report << "],\"candidate_selected\":"
           << (passed ? "true" : "false")
           << ",\"b4c1_pressure_tape_design_authorized\":"
           << (passed ? "true" : "false")
           << ",\"trajectory_substitution_authorized\":false"
           << ",\"canonical_continuation_authorized\":false"
           << ",\"nominal_corpus_execution_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"production_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(material.str()) << "\"}";
    return {passed, report.str()};
}

} // namespace nextengine::nonlocal::fcr
