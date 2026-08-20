#include "variational_reference.hpp"

#include "math.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <chrono>
#include <cmath>
#include <cstdint>
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
    double kernel_scale = 1.0;
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
    std::string failure;
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
    BlockThenInertialWarm,
};

enum class SissmAcceleration {
    None,
    PressureChebyshev,
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

double cubic_second_derivative(double radius, double horizon) {
    const double q = 2.0 * radius / horizon;
    const double alpha = 3.0 / (2.0 * PI * horizon * horizon * horizon);
    if (q > 2.0) {
        return 0.0;
    }
    const double second_q = q >= 1.0
        ? alpha * (2.0 - q)
        : alpha * (-2.0 + 3.0 * q);
    const double q_scale = 2.0 / horizon;
    return second_q * q_scale * q_scale;
}

double configured_cubic_weight(const Config& config, double radius) {
    return config.kernel_scale * cubic_weight(radius, config.horizon);
}

double configured_cubic_gradient(const Config& config, double radius) {
    return config.kernel_scale * cubic_gradient(radius, config.horizon);
}

double configured_cubic_second_derivative(
    const Config& config, double radius) {
    return config.kernel_scale
        * cubic_second_derivative(radius, config.horizon);
}

double reference_lattice_kernel_scale(double spacing, double horizon) {
    const int extent = static_cast<int>(std::ceil(horizon / spacing));
    double ratio = 0.0;
    for (int z = -extent; z <= extent; ++z) {
        for (int y = -extent; y <= extent; ++y) {
            for (int x = -extent; x <= extent; ++x) {
                const double radius = spacing * std::sqrt(
                    static_cast<double>(x * x + y * y + z * z));
                ratio += spacing * spacing * spacing
                    * cubic_weight(radius, horizon);
            }
        }
    }
    if (!std::isfinite(ratio) || ratio <= 0.0) {
        throw std::runtime_error("invalid reference lattice kernel sum");
    }
    const double scale = 1.0 / ratio;
    if (!std::isfinite(scale) || scale < 7.5 || scale > 8.5) {
        throw std::runtime_error("normalized kernel scale outside B0R bounds");
    }
    return scale;
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
        y.size(), config.mass * configured_cubic_weight(config, 0.0));
    for (std::size_t i = 0; i < y.size(); ++i) {
        for (std::size_t j = i + 1; j < y.size(); ++j) {
            const double radius = norm(y[i] - y[j]);
            if (radius <= config.horizon) {
                const double contribution =
                    config.mass * configured_cubic_weight(config, radius);
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
                * configured_cubic_gradient(config, radius);
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
            const double omega = -configured_cubic_gradient(config, radius);
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

std::vector<double> densities(
    const Config& config, const std::vector<Vec3>& y) {
    std::vector<double> result(
        y.size(), config.mass * configured_cubic_weight(config, 0.0));
    for (std::size_t i = 0; i < y.size(); ++i) {
        for (std::size_t j = i + 1; j < y.size(); ++j) {
            const double radius = norm(y[i] - y[j]);
            if (radius <= config.horizon) {
                const double contribution =
                    config.mass * configured_cubic_weight(config, radius);
                result[i] += contribution;
                result[j] += contribution;
            }
        }
    }
    return result;
}

Vec3 radial_hessian_product(
    Vec3 normal, double radial, double tangential, Vec3 value) {
    const Vec3 normal_value = project_normal(value, normal);
    return radial * normal_value + tangential * (value - normal_value);
}

std::vector<Vec3> apply_hessian(
    const Config& config,
    const std::vector<Vec3>& x,
    const std::vector<Vec3>& y,
    const std::vector<Vec3>& direction) {
    std::vector<Vec3> result(y.size());
    const double inertia_scale = config.mass
        / (config.time_step * config.time_step);
    for (std::size_t i = 0; i < y.size(); ++i) {
        result[i] += inertia_scale * direction[i];
    }

    if (config.kappa != 0.0) {
        const std::vector<double> density = densities(config, y);
        for (std::size_t center = 0; center < y.size(); ++center) {
            const double compression =
                density[center] / config.rest_density - 1.0;
            if (compression <= 0.0) {
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
                const Vec3 normal = displacement / radius;
                const Vec3 pair_jacobian = config.mass / config.rest_density
                    * configured_cubic_gradient(config, radius) * normal;
                jacobian[center] += pair_jacobian;
                jacobian[neighbor] += -pair_jacobian;
            }

            const double density_direction = vector_dot(jacobian, direction);
            for (std::size_t i = 0; i < y.size(); ++i) {
                result[i] += config.kappa * density_direction * jacobian[i];
            }

            for (std::size_t neighbor = 0; neighbor < y.size(); ++neighbor) {
                if (neighbor == center) {
                    continue;
                }
                const Vec3 displacement = y[center] - y[neighbor];
                const double radius = norm(displacement);
                if (radius <= 1.0e-15 || radius > config.horizon) {
                    continue;
                }
                const Vec3 normal = displacement / radius;
                const Vec3 relative_direction =
                    direction[center] - direction[neighbor];
                const double radial =
                    configured_cubic_second_derivative(config, radius);
                const double tangential =
                    configured_cubic_gradient(config, radius) / radius;
                const Vec3 pair = config.kappa * compression * config.mass
                    / config.rest_density
                    * radial_hessian_product(
                        normal, radial, tangential, relative_direction);
                result[center] += pair;
                result[neighbor] += -pair;
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
            const Vec3 relative_direction = direction[i] - direction[j];
            const double scale = config.mass
                * (-configured_cubic_gradient(config, radius))
                / (config.rest_density * config.time_step);
            const Vec3 pair = scale
                * (config.lambda * project_normal(relative_direction, normal)
                    + 2.0 * config.mu
                        * project_tangent(relative_direction, normal));
            result[i] += pair;
            result[j] += -pair;
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
            const Vec3 relative_direction = direction[i] - direction[j];
            const double scale = 2.0 * config.gamma
                * config.mass * config.mass;
            const double radial = scale
                * surface_spline_derivative(radius, config.spacing);
            const double tangential = scale
                * surface_spline(radius, config.spacing) / radius;
            const Vec3 pair = radial_hessian_product(
                normal, radial, tangential, relative_direction);
            result[i] += pair;
            result[j] += -pair;
        }
    }
    return result;
}

std::vector<double> flatten(const std::vector<Vec3>& values) {
    std::vector<double> result;
    result.reserve(3 * values.size());
    for (Vec3 value : values) {
        result.push_back(value.x);
        result.push_back(value.y);
        result.push_back(value.z);
    }
    return result;
}

std::vector<Vec3> unflatten(const std::vector<double>& values) {
    std::vector<Vec3> result(values.size() / 3);
    for (std::size_t i = 0; i < result.size(); ++i) {
        result[i] = {values[3 * i], values[3 * i + 1], values[3 * i + 2]};
    }
    return result;
}

double flat_norm(const std::vector<double>& values) {
    double squared = 0.0;
    for (double value : values) {
        squared += value * value;
    }
    return std::sqrt(squared);
}

bool all_finite(const std::vector<double>& values) {
    for (double value : values) {
        if (!std::isfinite(value)) {
            return false;
        }
    }
    return true;
}

struct ParticlePair {
    std::size_t i = 0;
    std::size_t j = 0;
};

bool operator==(const ParticlePair& lhs, const ParticlePair& rhs) {
    return lhs.i == rhs.i && lhs.j == rhs.j;
}

struct CellRecord {
    std::int64_t x = 0;
    std::int64_t y = 0;
    std::int64_t z = 0;
    std::size_t sample = 0;
};

struct CellRange {
    std::int64_t x = 0;
    std::int64_t y = 0;
    std::int64_t z = 0;
    std::size_t begin = 0;
    std::size_t end = 0;
};

bool cell_key_less(
    std::int64_t ax,
    std::int64_t ay,
    std::int64_t az,
    std::int64_t bx,
    std::int64_t by,
    std::int64_t bz) {
    if (ax != bx) {
        return ax < bx;
    }
    if (ay != by) {
        return ay < by;
    }
    return az < bz;
}

std::int64_t cell_coordinate(double value, double cell_edge) {
    const double coordinate = std::floor(value / cell_edge);
    const double low = static_cast<double>(std::numeric_limits<std::int64_t>::min());
    const double high = static_cast<double>(std::numeric_limits<std::int64_t>::max());
    if (!std::isfinite(coordinate) || coordinate < low || coordinate > high) {
        throw std::runtime_error("cell coordinate out of range");
    }
    return static_cast<std::int64_t>(coordinate);
}

std::vector<ParticlePair> all_pairs_inside(
    const std::vector<Vec3>& position, double support) {
    std::vector<ParticlePair> result;
    for (std::size_t i = 0; i < position.size(); ++i) {
        for (std::size_t j = i + 1; j < position.size(); ++j) {
            if (norm(position[i] - position[j]) <= support) {
                result.push_back({i, j});
            }
        }
    }
    return result;
}

std::vector<ParticlePair> build_cell_pairs(
    const std::vector<Vec3>& position, double support) {
    std::vector<CellRecord> records;
    records.reserve(position.size());
    for (std::size_t i = 0; i < position.size(); ++i) {
        if (!finite(position[i])) {
            throw std::runtime_error("nonfinite cell position");
        }
        records.push_back({
            cell_coordinate(position[i].x, support),
            cell_coordinate(position[i].y, support),
            cell_coordinate(position[i].z, support),
            i,
        });
    }
    std::sort(records.begin(), records.end(),
        [](const CellRecord& lhs, const CellRecord& rhs) {
            if (cell_key_less(
                    lhs.x, lhs.y, lhs.z, rhs.x, rhs.y, rhs.z)) {
                return true;
            }
            if (cell_key_less(
                    rhs.x, rhs.y, rhs.z, lhs.x, lhs.y, lhs.z)) {
                return false;
            }
            return lhs.sample < rhs.sample;
        });
    std::vector<CellRange> ranges;
    for (std::size_t begin = 0; begin < records.size();) {
        std::size_t end = begin + 1;
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

    std::vector<ParticlePair> result;
    for (std::size_t i = 0; i < position.size(); ++i) {
        const std::int64_t cx = cell_coordinate(position[i].x, support);
        const std::int64_t cy = cell_coordinate(position[i].y, support);
        const std::int64_t cz = cell_coordinate(position[i].z, support);
        for (int dx = -1; dx <= 1; ++dx) {
            for (int dy = -1; dy <= 1; ++dy) {
                for (int dz = -1; dz <= 1; ++dz) {
                    const std::int64_t qx = cx + dx;
                    const std::int64_t qy = cy + dy;
                    const std::int64_t qz = cz + dz;
                    const auto range = std::lower_bound(ranges.begin(), ranges.end(),
                        CellRange{qx, qy, qz, 0, 0},
                        [](const CellRange& lhs, const CellRange& rhs) {
                            return cell_key_less(lhs.x, lhs.y, lhs.z,
                                rhs.x, rhs.y, rhs.z);
                        });
                    if (range == ranges.end()
                        || range->x != qx || range->y != qy || range->z != qz) {
                        continue;
                    }
                    for (std::size_t slot = range->begin;
                         slot < range->end; ++slot) {
                        const std::size_t j = records[slot].sample;
                        if (j > i && norm(position[i] - position[j]) <= support) {
                            result.push_back({i, j});
                        }
                    }
                }
            }
        }
    }
    std::sort(result.begin(), result.end(),
        [](const ParticlePair& lhs, const ParticlePair& rhs) {
            return lhs.i < rhs.i || (lhs.i == rhs.i && lhs.j < rhs.j);
        });
    const auto duplicate = std::adjacent_find(result.begin(), result.end());
    if (duplicate != result.end()) {
        throw std::runtime_error("duplicate cell pair");
    }
    return result;
}

std::vector<double> densities_with_pairs(
    const Config& config,
    const std::vector<Vec3>& y,
    const std::vector<ParticlePair>& pairs) {
    std::vector<double> result(
        y.size(), config.mass * configured_cubic_weight(config, 0.0));
    for (const ParticlePair pair : pairs) {
        const double radius = norm(y[pair.i] - y[pair.j]);
        if (radius <= config.horizon) {
            const double contribution =
                config.mass * configured_cubic_weight(config, radius);
            result[pair.i] += contribution;
            result[pair.j] += contribution;
        }
    }
    return result;
}

Evaluation evaluate_with_pairs(
    const Config& config,
    const std::vector<Vec3>& x,
    const std::vector<Vec3>& y_star,
    const std::vector<Vec3>& y,
    const std::vector<ParticlePair>& current_pairs,
    const std::vector<ParticlePair>& reference_pairs) {
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

    const std::vector<double> density =
        densities_with_pairs(config, y, current_pairs);
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
    for (const ParticlePair pair : current_pairs) {
        const Vec3 displacement = y[pair.i] - y[pair.j];
        const double radius = norm(displacement);
        if (radius <= 1.0e-15 || radius > config.horizon) {
            continue;
        }
        const double coefficient = config.kappa * config.mass
            / config.rest_density
            * (compression[pair.i] + compression[pair.j])
            * configured_cubic_gradient(config, radius);
        const Vec3 pair_gradient = coefficient * (displacement / radius);
        result.gradient[pair.i] += pair_gradient;
        result.gradient[pair.j] += -pair_gradient;
    }

    for (const ParticlePair pair : reference_pairs) {
        const Vec3 reference = x[pair.i] - x[pair.j];
        const double radius = norm(reference);
        if (radius <= 1.0e-15 || radius > config.horizon) {
            continue;
        }
        const Vec3 normal = reference / radius;
        const Vec3 increment =
            (y[pair.i] - y[pair.j]) - reference;
        const Vec3 normal_increment = project_normal(increment, normal);
        const Vec3 tangent_increment = project_tangent(increment, normal);
        const double omega = -configured_cubic_gradient(config, radius);
        result.viscosity += config.mass / (config.rest_density * config.time_step)
            * (config.mu * norm_squared(tangent_increment)
                + 0.5 * config.lambda * norm_squared(normal_increment))
            * omega;
        const Vec3 pair_gradient = config.mass * omega
            / (config.rest_density * config.time_step)
            * (2.0 * config.mu * tangent_increment
                + config.lambda * normal_increment);
        result.gradient[pair.i] += pair_gradient;
        result.gradient[pair.j] += -pair_gradient;
    }

    const double surface_support = 3.0 * config.spacing;
    for (const ParticlePair pair : current_pairs) {
        const Vec3 displacement = y[pair.i] - y[pair.j];
        const double radius = norm(displacement);
        if (radius <= 1.0e-15 || radius >= surface_support) {
            continue;
        }
        result.surface += 2.0 * config.gamma * config.mass * config.mass
            * surface_potential(radius, config.spacing);
        const Vec3 pair_gradient = 2.0 * config.gamma * config.mass
            * config.mass * surface_spline(radius, config.spacing)
            * (displacement / radius);
        result.gradient[pair.i] += pair_gradient;
        result.gradient[pair.j] += -pair_gradient;
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

std::vector<std::vector<std::size_t>> build_pair_adjacency(
    std::size_t particle_count,
    const std::vector<ParticlePair>& pairs) {
    std::vector<std::vector<std::size_t>> result(particle_count);
    for (const ParticlePair pair : pairs) {
        result[pair.i].push_back(pair.j);
        result[pair.j].push_back(pair.i);
    }
    for (std::vector<std::size_t>& neighbors : result) {
        std::sort(neighbors.begin(), neighbors.end());
    }
    return result;
}

std::vector<Vec3> apply_hessian_with_adjacency(
    const Config& config,
    const std::vector<Vec3>& x,
    const std::vector<Vec3>& y,
    const std::vector<Vec3>& direction,
    const std::vector<ParticlePair>& current_pairs,
    const std::vector<ParticlePair>& reference_pairs,
    const std::vector<std::vector<std::size_t>>& adjacency) {
    std::vector<Vec3> result(y.size());
    const double inertia_scale = config.mass
        / (config.time_step * config.time_step);
    for (std::size_t i = 0; i < y.size(); ++i) {
        result[i] += inertia_scale * direction[i];
    }

    if (config.kappa != 0.0) {
        const std::vector<double> density =
            densities_with_pairs(config, y, current_pairs);
        for (std::size_t center = 0; center < y.size(); ++center) {
            const double compression =
                density[center] / config.rest_density - 1.0;
            if (compression <= 0.0) {
                continue;
            }
            Vec3 center_jacobian;
            std::vector<Vec3> neighbor_jacobian(adjacency[center].size());
            for (std::size_t slot = 0; slot < adjacency[center].size(); ++slot) {
                const std::size_t neighbor = adjacency[center][slot];
                const Vec3 displacement = y[center] - y[neighbor];
                const double radius = norm(displacement);
                if (radius <= 1.0e-15 || radius > config.horizon) {
                    continue;
                }
                const Vec3 normal = displacement / radius;
                const Vec3 pair_jacobian = config.mass / config.rest_density
                    * configured_cubic_gradient(config, radius) * normal;
                center_jacobian += pair_jacobian;
                neighbor_jacobian[slot] = -pair_jacobian;
            }
            std::vector<std::size_t> participants = adjacency[center];
            participants.push_back(center);
            std::sort(participants.begin(), participants.end());
            double density_direction = 0.0;
            for (std::size_t participant : participants) {
                if (participant == center) {
                    density_direction += dot(center_jacobian, direction[center]);
                } else {
                    const auto neighbor = std::lower_bound(
                        adjacency[center].begin(), adjacency[center].end(),
                        participant);
                    const std::size_t slot = static_cast<std::size_t>(
                        neighbor - adjacency[center].begin());
                    density_direction += dot(
                        neighbor_jacobian[slot], direction[participant]);
                }
            }
            for (std::size_t participant : participants) {
                const Vec3 jacobian = participant == center
                    ? center_jacobian
                    : neighbor_jacobian[static_cast<std::size_t>(
                        std::lower_bound(adjacency[center].begin(),
                            adjacency[center].end(), participant)
                        - adjacency[center].begin())];
                result[participant] +=
                    config.kappa * density_direction * jacobian;
            }
            for (std::size_t neighbor : adjacency[center]) {
                const Vec3 displacement = y[center] - y[neighbor];
                const double radius = norm(displacement);
                if (radius <= 1.0e-15 || radius > config.horizon) {
                    continue;
                }
                const Vec3 normal = displacement / radius;
                const Vec3 relative_direction =
                    direction[center] - direction[neighbor];
                const Vec3 pair = config.kappa * compression * config.mass
                    / config.rest_density
                    * radial_hessian_product(normal,
                        configured_cubic_second_derivative(config, radius),
                        configured_cubic_gradient(config, radius) / radius,
                        relative_direction);
                result[center] += pair;
                result[neighbor] += -pair;
            }
        }
    }

    for (const ParticlePair pair_index : reference_pairs) {
        const Vec3 reference = x[pair_index.i] - x[pair_index.j];
        const double radius = norm(reference);
        if (radius <= 1.0e-15 || radius > config.horizon) {
            continue;
        }
        const Vec3 normal = reference / radius;
        const Vec3 relative_direction =
            direction[pair_index.i] - direction[pair_index.j];
        const double scale = config.mass
            * (-configured_cubic_gradient(config, radius))
            / (config.rest_density * config.time_step);
        const Vec3 pair = scale
            * (config.lambda * project_normal(relative_direction, normal)
                + 2.0 * config.mu
                    * project_tangent(relative_direction, normal));
        result[pair_index.i] += pair;
        result[pair_index.j] += -pair;
    }

    const double surface_support = 3.0 * config.spacing;
    for (const ParticlePair pair_index : current_pairs) {
        const Vec3 displacement = y[pair_index.i] - y[pair_index.j];
        const double radius = norm(displacement);
        if (radius <= 1.0e-15 || radius >= surface_support) {
            continue;
        }
        const Vec3 normal = displacement / radius;
        const Vec3 relative_direction =
            direction[pair_index.i] - direction[pair_index.j];
        const double scale = 2.0 * config.gamma
            * config.mass * config.mass;
        const Vec3 pair = radial_hessian_product(normal,
            scale * surface_spline_derivative(radius, config.spacing),
            scale * surface_spline(radius, config.spacing) / radius,
            relative_direction);
        result[pair_index.i] += pair;
        result[pair_index.j] += -pair;
    }
    return result;
}

struct NeighborhoodHvpWorkspace {
    std::vector<Vec3> result;
    std::vector<double> density;
    std::vector<Vec3> neighbor_jacobian;
};

struct CurrentPairHessianCoefficient {
    ParticlePair pair;
    Vec3 normal;
    Vec3 pair_jacobian_i;
    double density_radial = 0.0;
    double density_tangential = 0.0;
    double surface_radial = 0.0;
    double surface_tangential = 0.0;
    bool density_active = false;
    bool surface_active = false;
};

struct ReferencePairHessianCoefficient {
    ParticlePair pair;
    Vec3 normal;
    double scale = 0.0;
    bool active = false;
};

struct PressureCenterHessianCoefficient {
    Vec3 center_jacobian;
    double compression = 0.0;
};

struct NeighborhoodHessianTape {
    double inertia_scale = 0.0;
    std::vector<CurrentPairHessianCoefficient> current_pairs;
    std::vector<ReferencePairHessianCoefficient> reference_pairs;
    std::vector<PressureCenterHessianCoefficient> pressure_centers;
    std::vector<std::vector<std::size_t>> adjacency_pair_indices;
};

std::size_t hessian_tape_storage_bytes(const NeighborhoodHessianTape& tape) {
    std::size_t result = tape.current_pairs.capacity()
        * sizeof(CurrentPairHessianCoefficient);
    result += tape.reference_pairs.capacity()
        * sizeof(ReferencePairHessianCoefficient);
    result += tape.pressure_centers.capacity()
        * sizeof(PressureCenterHessianCoefficient);
    result += tape.adjacency_pair_indices.capacity()
        * sizeof(std::vector<std::size_t>);
    for (const std::vector<std::size_t>& indices :
         tape.adjacency_pair_indices) {
        result += indices.capacity() * sizeof(std::size_t);
    }
    return result;
}

std::size_t hessian_tape_storage_limit(
    std::size_t particles, std::size_t maximum_pairs) {
    return 192 * maximum_pairs + 128 * particles;
}

std::size_t hessian_tape_required_upper_bytes(
    std::size_t particles,
    std::size_t current_pairs,
    std::size_t reference_pairs) {
    return current_pairs * sizeof(CurrentPairHessianCoefficient)
        + reference_pairs * sizeof(ReferencePairHessianCoefficient)
        + particles * sizeof(PressureCenterHessianCoefficient)
        + particles * sizeof(std::vector<std::size_t>)
        + 2 * current_pairs * sizeof(std::size_t);
}

void build_reference_hessian_tape(
    const Config& config,
    const std::vector<Vec3>& x,
    const std::vector<ParticlePair>& reference_pairs,
    NeighborhoodHessianTape& tape) {
    tape.inertia_scale = config.mass
        / (config.time_step * config.time_step);
    tape.reference_pairs.resize(reference_pairs.size());
    for (std::size_t index = 0; index < reference_pairs.size(); ++index) {
        ReferencePairHessianCoefficient& coefficient =
            tape.reference_pairs[index];
        coefficient = {};
        coefficient.pair = reference_pairs[index];
        const Vec3 reference =
            x[coefficient.pair.i] - x[coefficient.pair.j];
        const double radius = norm(reference);
        if (radius <= 1.0e-15 || radius > config.horizon) {
            continue;
        }
        coefficient.normal = reference / radius;
        coefficient.scale = config.mass
            * (-configured_cubic_gradient(config, radius))
            / (config.rest_density * config.time_step);
        coefficient.active = true;
    }
}

bool build_current_hessian_tape(
    const Config& config,
    const std::vector<Vec3>& y,
    const std::vector<ParticlePair>& current_pairs,
    const std::vector<std::vector<std::size_t>>& adjacency,
    NeighborhoodHessianTape& tape) {
    tape.current_pairs.resize(current_pairs.size());
    tape.adjacency_pair_indices.clear();
    tape.adjacency_pair_indices.resize(y.size());
    for (std::size_t particle = 0; particle < adjacency.size(); ++particle) {
        tape.adjacency_pair_indices[particle].reserve(
            adjacency[particle].size());
    }
    const double surface_support = 3.0 * config.spacing;
    for (std::size_t index = 0; index < current_pairs.size(); ++index) {
        CurrentPairHessianCoefficient& coefficient =
            tape.current_pairs[index];
        coefficient = {};
        coefficient.pair = current_pairs[index];
        tape.adjacency_pair_indices[coefficient.pair.i].push_back(index);
        tape.adjacency_pair_indices[coefficient.pair.j].push_back(index);
        const Vec3 displacement =
            y[coefficient.pair.i] - y[coefficient.pair.j];
        const double radius = norm(displacement);
        if (radius <= 1.0e-15) {
            continue;
        }
        coefficient.normal = displacement / radius;
        if (radius <= config.horizon) {
            coefficient.pair_jacobian_i = config.mass / config.rest_density
                * configured_cubic_gradient(config, radius)
                * coefficient.normal;
            coefficient.density_radial =
                configured_cubic_second_derivative(config, radius);
            coefficient.density_tangential =
                configured_cubic_gradient(config, radius) / radius;
            coefficient.density_active = true;
        }
        if (radius < surface_support) {
            const double scale = 2.0 * config.gamma
                * config.mass * config.mass;
            coefficient.surface_radial = scale
                * surface_spline_derivative(radius, config.spacing);
            coefficient.surface_tangential = scale
                * surface_spline(radius, config.spacing) / radius;
            coefficient.surface_active = true;
        }
    }
    for (std::size_t particle = 0; particle < adjacency.size(); ++particle) {
        if (adjacency[particle].size()
            != tape.adjacency_pair_indices[particle].size()) {
            return false;
        }
        for (std::size_t slot = 0; slot < adjacency[particle].size(); ++slot) {
            const CurrentPairHessianCoefficient& coefficient =
                tape.current_pairs[
                    tape.adjacency_pair_indices[particle][slot]];
            const std::size_t neighbor = coefficient.pair.i == particle
                ? coefficient.pair.j : coefficient.pair.i;
            if (neighbor != adjacency[particle][slot]) {
                return false;
            }
        }
    }
    tape.pressure_centers.resize(y.size());
    std::vector<double> density(
        y.size(), config.mass * configured_cubic_weight(config, 0.0));
    for (const CurrentPairHessianCoefficient& coefficient :
         tape.current_pairs) {
        if (!coefficient.density_active) {
            continue;
        }
        const double radius = norm(
            y[coefficient.pair.i] - y[coefficient.pair.j]);
        const double contribution =
            config.mass * configured_cubic_weight(config, radius);
        density[coefficient.pair.i] += contribution;
        density[coefficient.pair.j] += contribution;
    }
    for (std::size_t center = 0; center < y.size(); ++center) {
        PressureCenterHessianCoefficient& center_coefficient =
            tape.pressure_centers[center];
        center_coefficient = {};
        center_coefficient.compression =
            density[center] / config.rest_density - 1.0;
        if (center_coefficient.compression <= 0.0) {
            continue;
        }
        for (std::size_t pair_index :
             tape.adjacency_pair_indices[center]) {
            const CurrentPairHessianCoefficient& pair =
                tape.current_pairs[pair_index];
            if (!pair.density_active) {
                continue;
            }
            const Vec3 pair_jacobian = pair.pair.i == center
                ? pair.pair_jacobian_i : -pair.pair_jacobian_i;
            center_coefficient.center_jacobian += pair_jacobian;
        }
    }
    return true;
}

const std::vector<Vec3>& apply_hessian_with_tape(
    const Config& config,
    const std::vector<Vec3>& direction,
    const std::vector<std::vector<std::size_t>>& adjacency,
    const NeighborhoodHessianTape& tape,
    NeighborhoodHvpWorkspace& workspace) {
    workspace.result.resize(direction.size());
    std::fill(workspace.result.begin(), workspace.result.end(), Vec3{});
    for (std::size_t i = 0; i < direction.size(); ++i) {
        workspace.result[i] += tape.inertia_scale * direction[i];
    }
    if (config.kappa != 0.0) {
        for (std::size_t center = 0; center < direction.size(); ++center) {
            const PressureCenterHessianCoefficient& center_coefficient =
                tape.pressure_centers[center];
            if (center_coefficient.compression <= 0.0) {
                continue;
            }
            double density_direction = 0.0;
            bool center_visited = false;
            for (std::size_t slot = 0; slot < adjacency[center].size(); ++slot) {
                const std::size_t neighbor = adjacency[center][slot];
                if (!center_visited && center < neighbor) {
                    density_direction += dot(
                        center_coefficient.center_jacobian,
                        direction[center]);
                    center_visited = true;
                }
                const CurrentPairHessianCoefficient& pair =
                    tape.current_pairs[
                        tape.adjacency_pair_indices[center][slot]];
                const Vec3 neighbor_jacobian = pair.pair.i == center
                    ? -pair.pair_jacobian_i : pair.pair_jacobian_i;
                density_direction += dot(
                    neighbor_jacobian, direction[neighbor]);
            }
            if (!center_visited) {
                density_direction += dot(
                    center_coefficient.center_jacobian, direction[center]);
            }
            center_visited = false;
            for (std::size_t slot = 0; slot < adjacency[center].size(); ++slot) {
                const std::size_t neighbor = adjacency[center][slot];
                if (!center_visited && center < neighbor) {
                    workspace.result[center] += config.kappa
                        * density_direction
                        * center_coefficient.center_jacobian;
                    center_visited = true;
                }
                const CurrentPairHessianCoefficient& pair =
                    tape.current_pairs[
                        tape.adjacency_pair_indices[center][slot]];
                const Vec3 neighbor_jacobian = pair.pair.i == center
                    ? -pair.pair_jacobian_i : pair.pair_jacobian_i;
                workspace.result[neighbor] += config.kappa
                    * density_direction * neighbor_jacobian;
            }
            if (!center_visited) {
                workspace.result[center] += config.kappa
                    * density_direction
                    * center_coefficient.center_jacobian;
            }
            for (std::size_t slot = 0; slot < adjacency[center].size(); ++slot) {
                const std::size_t neighbor = adjacency[center][slot];
                const CurrentPairHessianCoefficient& coefficient =
                    tape.current_pairs[
                        tape.adjacency_pair_indices[center][slot]];
                if (!coefficient.density_active) {
                    continue;
                }
                const Vec3 normal = coefficient.pair.i == center
                    ? coefficient.normal : -coefficient.normal;
                const Vec3 relative_direction =
                    direction[center] - direction[neighbor];
                const Vec3 pair = config.kappa
                    * center_coefficient.compression * config.mass
                    / config.rest_density
                    * radial_hessian_product(normal,
                        coefficient.density_radial,
                        coefficient.density_tangential,
                        relative_direction);
                workspace.result[center] += pair;
                workspace.result[neighbor] += -pair;
            }
        }
    }
    for (const ReferencePairHessianCoefficient& coefficient :
         tape.reference_pairs) {
        if (!coefficient.active) {
            continue;
        }
        const Vec3 relative_direction = direction[coefficient.pair.i]
            - direction[coefficient.pair.j];
        const Vec3 pair = coefficient.scale
            * (config.lambda
                    * project_normal(relative_direction, coefficient.normal)
                + 2.0 * config.mu
                    * project_tangent(relative_direction, coefficient.normal));
        workspace.result[coefficient.pair.i] += pair;
        workspace.result[coefficient.pair.j] += -pair;
    }
    for (const CurrentPairHessianCoefficient& coefficient :
         tape.current_pairs) {
        if (!coefficient.surface_active) {
            continue;
        }
        const Vec3 relative_direction = direction[coefficient.pair.i]
            - direction[coefficient.pair.j];
        const Vec3 pair = radial_hessian_product(coefficient.normal,
            coefficient.surface_radial, coefficient.surface_tangential,
            relative_direction);
        workspace.result[coefficient.pair.i] += pair;
        workspace.result[coefficient.pair.j] += -pair;
    }
    return workspace.result;
}

const std::vector<Vec3>& apply_hessian_with_workspace(
    const Config& config,
    const std::vector<Vec3>& x,
    const std::vector<Vec3>& y,
    const std::vector<Vec3>& direction,
    const std::vector<ParticlePair>& current_pairs,
    const std::vector<ParticlePair>& reference_pairs,
    const std::vector<std::vector<std::size_t>>& adjacency,
    NeighborhoodHvpWorkspace& workspace) {
    workspace.result.resize(y.size());
    std::fill(workspace.result.begin(), workspace.result.end(), Vec3{});
    const double inertia_scale = config.mass
        / (config.time_step * config.time_step);
    for (std::size_t i = 0; i < y.size(); ++i) {
        workspace.result[i] += inertia_scale * direction[i];
    }

    if (config.kappa != 0.0) {
        workspace.density.resize(y.size());
        std::fill(workspace.density.begin(), workspace.density.end(),
            config.mass * configured_cubic_weight(config, 0.0));
        for (const ParticlePair pair : current_pairs) {
            const double radius = norm(y[pair.i] - y[pair.j]);
            if (radius <= config.horizon) {
                const double contribution =
                    config.mass * configured_cubic_weight(config, radius);
                workspace.density[pair.i] += contribution;
                workspace.density[pair.j] += contribution;
            }
        }
        for (std::size_t center = 0; center < y.size(); ++center) {
            const double compression =
                workspace.density[center] / config.rest_density - 1.0;
            if (compression <= 0.0) {
                continue;
            }
            Vec3 center_jacobian;
            workspace.neighbor_jacobian.resize(adjacency[center].size());
            std::fill(workspace.neighbor_jacobian.begin(),
                workspace.neighbor_jacobian.end(), Vec3{});
            for (std::size_t slot = 0; slot < adjacency[center].size(); ++slot) {
                const std::size_t neighbor = adjacency[center][slot];
                const Vec3 displacement = y[center] - y[neighbor];
                const double radius = norm(displacement);
                if (radius <= 1.0e-15 || radius > config.horizon) {
                    continue;
                }
                const Vec3 normal = displacement / radius;
                const Vec3 pair_jacobian = config.mass / config.rest_density
                    * configured_cubic_gradient(config, radius) * normal;
                center_jacobian += pair_jacobian;
                workspace.neighbor_jacobian[slot] = -pair_jacobian;
            }
            double density_direction = 0.0;
            bool center_visited = false;
            for (std::size_t slot = 0; slot < adjacency[center].size(); ++slot) {
                const std::size_t neighbor = adjacency[center][slot];
                if (!center_visited && center < neighbor) {
                    density_direction += dot(
                        center_jacobian, direction[center]);
                    center_visited = true;
                }
                density_direction += dot(
                    workspace.neighbor_jacobian[slot], direction[neighbor]);
            }
            if (!center_visited) {
                density_direction += dot(center_jacobian, direction[center]);
            }
            center_visited = false;
            for (std::size_t slot = 0; slot < adjacency[center].size(); ++slot) {
                const std::size_t neighbor = adjacency[center][slot];
                if (!center_visited && center < neighbor) {
                    workspace.result[center] += config.kappa
                        * density_direction * center_jacobian;
                    center_visited = true;
                }
                workspace.result[neighbor] += config.kappa
                    * density_direction * workspace.neighbor_jacobian[slot];
            }
            if (!center_visited) {
                workspace.result[center] += config.kappa
                    * density_direction * center_jacobian;
            }
            for (std::size_t neighbor : adjacency[center]) {
                const Vec3 displacement = y[center] - y[neighbor];
                const double radius = norm(displacement);
                if (radius <= 1.0e-15 || radius > config.horizon) {
                    continue;
                }
                const Vec3 normal = displacement / radius;
                const Vec3 relative_direction =
                    direction[center] - direction[neighbor];
                const Vec3 pair = config.kappa * compression * config.mass
                    / config.rest_density
                    * radial_hessian_product(normal,
                        configured_cubic_second_derivative(config, radius),
                        configured_cubic_gradient(config, radius) / radius,
                        relative_direction);
                workspace.result[center] += pair;
                workspace.result[neighbor] += -pair;
            }
        }
    }

    for (const ParticlePair pair_index : reference_pairs) {
        const Vec3 reference = x[pair_index.i] - x[pair_index.j];
        const double radius = norm(reference);
        if (radius <= 1.0e-15 || radius > config.horizon) {
            continue;
        }
        const Vec3 normal = reference / radius;
        const Vec3 relative_direction =
            direction[pair_index.i] - direction[pair_index.j];
        const double scale = config.mass
            * (-configured_cubic_gradient(config, radius))
            / (config.rest_density * config.time_step);
        const Vec3 pair = scale
            * (config.lambda * project_normal(relative_direction, normal)
                + 2.0 * config.mu
                    * project_tangent(relative_direction, normal));
        workspace.result[pair_index.i] += pair;
        workspace.result[pair_index.j] += -pair;
    }

    const double surface_support = 3.0 * config.spacing;
    for (const ParticlePair pair_index : current_pairs) {
        const Vec3 displacement = y[pair_index.i] - y[pair_index.j];
        const double radius = norm(displacement);
        if (radius <= 1.0e-15 || radius >= surface_support) {
            continue;
        }
        const Vec3 normal = displacement / radius;
        const Vec3 relative_direction =
            direction[pair_index.i] - direction[pair_index.j];
        const double scale = 2.0 * config.gamma
            * config.mass * config.mass;
        const Vec3 pair = radial_hessian_product(normal,
            scale * surface_spline_derivative(radius, config.spacing),
            scale * surface_spline(radius, config.spacing) / radius,
            relative_direction);
        workspace.result[pair_index.i] += pair;
        workspace.result[pair_index.j] += -pair;
    }
    return workspace.result;
}

std::vector<Vec3> apply_hessian_with_pairs(
    const Config& config,
    const std::vector<Vec3>& x,
    const std::vector<Vec3>& y,
    const std::vector<Vec3>& direction,
    const std::vector<ParticlePair>& current_pairs,
    const std::vector<ParticlePair>& reference_pairs) {
    const std::vector<std::vector<std::size_t>> adjacency =
        build_pair_adjacency(y.size(), current_pairs);
    return apply_hessian_with_adjacency(config, x, y, direction,
        current_pairs, reference_pairs, adjacency);
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
            y.size(), config.mass * configured_cubic_weight(config, 0.0));
        for (std::size_t i = 0; i < y.size(); ++i) {
            for (std::size_t j = i + 1; j < y.size(); ++j) {
                const double radius = norm(y[i] - y[j]);
                if (radius <= config.horizon) {
                    const double contribution =
                        config.mass * configured_cubic_weight(config, radius);
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
                    * configured_cubic_gradient(config, radius)
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
                * (-configured_cubic_gradient(config, radius))
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
    double previous_accepted_alpha = 1.0;
    for (int iteration = 0; iteration < config.maximum_iterations; ++iteration) {
        if (current.gradient_norm <= 1.0e-10) {
            break;
        }
        std::vector<Vec3> direction(current.gradient.size());
        const bool use_block = preconditioner_kind == Preconditioner::BlockGaussNewton
            || (preconditioner_kind == Preconditioner::BlockThenInertialWarm
                && iteration < 16);
        if (use_block) {
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
        double alpha = preconditioner_kind == Preconditioner::BlockThenInertialWarm
            ? std::min(1.0, 2.0 * previous_accepted_alpha)
            : 1.0;
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
        previous_accepted_alpha = alpha;
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

SolveResult solve_sissm(
    const Config& config,
    const std::vector<Vec3>& x,
    const std::vector<Vec3>& velocity,
    SissmAcceleration acceleration = SissmAcceleration::None) {
    constexpr double armijo = 1.0e-4;
    constexpr int maximum_backtracks = 40;
    constexpr double chebyshev_spectral_radius = 0.9;
    const std::vector<Vec3> y_star = predict(config, x, velocity);
    std::vector<Vec3> y = y_star;
    std::vector<Vec3> previous_y = y;
    double chebyshev_omega = 1.0;
    Evaluation current = evaluate(config, x, y_star, y);
    SolveResult result;
    result.initial = current;
    result.minimum_alpha = 1.0;
    if (!current.finite) {
        result.failure = "NONFINITE_INITIAL_STATE";
        result.final = current;
        return result;
    }

    for (int iteration = 0; iteration < config.maximum_iterations; ++iteration) {
        if (current.gradient_norm <= 1.0e-10) {
            break;
        }
        std::vector<Mat3> local_matrix(y.size());
        std::vector<Vec3> source(y.size());

        std::vector<double> density(
            y.size(), config.mass * configured_cubic_weight(config, 0.0));
        for (std::size_t i = 0; i < y.size(); ++i) {
            for (std::size_t j = i + 1; j < y.size(); ++j) {
                const double radius = norm(y[i] - y[j]);
                if (radius <= config.horizon) {
                    const double contribution =
                        config.mass * configured_cubic_weight(config, radius);
                    density[i] += contribution;
                    density[j] += contribution;
                }
            }
        }
        for (std::size_t center = 0; center < y.size(); ++center) {
            if (config.kappa == 0.0 || density[center] <= config.rest_density) {
                continue;
            }
            const double density_ratio = density[center] / config.rest_density;
            for (std::size_t neighbor = 0; neighbor < y.size(); ++neighbor) {
                if (neighbor == center) {
                    continue;
                }
                const Vec3 displacement = y[center] - y[neighbor];
                const double radius = norm(displacement);
                if (radius <= 1.0e-15 || radius > config.horizon) {
                    continue;
                }
                const double a = config.kappa * config.time_step * config.time_step
                    / config.rest_density
                    * configured_cubic_gradient(config, radius) / radius;
                const double positive_diagonal = -a;
                source[center] += -a * y[neighbor]
                    + density_ratio * a * (y[neighbor] - y[center]);
                source[neighbor] += -a * y[center]
                    + density_ratio * a * (y[center] - y[neighbor]);
                local_matrix[center] +=
                    positive_diagonal * Mat3::identity();
                local_matrix[neighbor] +=
                    positive_diagonal * Mat3::identity();
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
                const double scale = config.time_step
                    * (-configured_cubic_gradient(config, radius))
                    / config.rest_density;
                const Mat3 pair_matrix = scale
                    * (2.0 * config.mu * tangent_projection
                        + config.lambda * normal_projection);
                source[i] += pair_matrix * (y[j] + (x[i] - x[j]));
                source[j] += pair_matrix * (y[i] + (x[j] - x[i]));
                local_matrix[i] += pair_matrix;
                local_matrix[j] += pair_matrix;
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
                const double coefficient = 2.0 * config.gamma * config.mass
                    * config.time_step * config.time_step
                    * surface_spline(radius, config.spacing) / radius;
                if (coefficient >= 0.0) {
                    source[i] += coefficient * y[j];
                    source[j] += coefficient * y[i];
                    local_matrix[i] += coefficient * Mat3::identity();
                    local_matrix[j] += coefficient * Mat3::identity();
                } else {
                    source[i] += -coefficient * (y[i] - y[j]);
                    source[j] += -coefficient * (y[j] - y[i]);
                }
            }
        }

        std::vector<Vec3> raw_candidate(y.size());
        std::vector<Vec3> candidate(y.size());
        std::vector<Vec3> direction(y.size());
        for (std::size_t i = 0; i < y.size(); ++i) {
            raw_candidate[i] = inverse_without_regularization(
                Mat3::identity() + local_matrix[i])
                * (y_star[i] + source[i]);
        }
        const bool apply_chebyshev =
            acceleration == SissmAcceleration::PressureChebyshev
            && config.kappa > 0.0 && iteration > 0;
        if (apply_chebyshev) {
            const double radius_squared = chebyshev_spectral_radius
                * chebyshev_spectral_radius;
            chebyshev_omega = iteration == 1
                ? 2.0 / (2.0 - radius_squared)
                : 4.0 / (4.0 - radius_squared * chebyshev_omega);
        }
        for (std::size_t i = 0; i < y.size(); ++i) {
            candidate[i] = apply_chebyshev
                ? previous_y[i]
                    + chebyshev_omega * (raw_candidate[i] - previous_y[i])
                : raw_candidate[i];
            direction[i] = candidate[i] - y[i];
        }
        const double slope = vector_dot(current.gradient, direction);
        if (!std::isfinite(slope) || slope >= 0.0) {
            result.failure = "NON_DESCENT_SISSM_DIRECTION";
            result.final = current;
            result.position = y;
            return result;
        }

        double alpha = 1.0;
        bool accepted = false;
        Evaluation trial_evaluation;
        std::vector<Vec3> trial(y.size());
        for (int backtrack = 0; backtrack <= maximum_backtracks; ++backtrack) {
            for (std::size_t i = 0; i < y.size(); ++i) {
                trial[i] = y[i] + alpha * direction[i];
            }
            trial_evaluation = evaluate(config, x, y_star, trial);
            if (trial_evaluation.finite
                && trial_evaluation.total
                    <= current.total + armijo * alpha * slope) {
                accepted = true;
                break;
            }
            alpha *= 0.5;
            ++result.backtracks;
        }
        if (!accepted) {
            result.failure = "SISSM_LINE_SEARCH_EXHAUSTED";
            result.final = current;
            result.position = y;
            return result;
        }
        const double allowance = ENERGY_ALLOWANCE
            * std::max({std::abs(current.total), std::abs(trial_evaluation.total), 1.0});
        result.monotonic = result.monotonic
            && trial_evaluation.total <= current.total + allowance;
        previous_y = y;
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
        * (configured_cubic_weight(config, 0.0)
            + configured_cubic_weight(config, distance));
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
        * configured_cubic_weight(fixture.config, 0.0);
    for (std::size_t j = 1; j < fixture.x.size(); ++j) {
        density += fixture.config.mass
            * configured_cubic_weight(
                fixture.config, norm(fixture.x[0] - fixture.x[j]));
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

struct SpectralFixture {
    std::string name;
    Config config;
    std::vector<Vec3> x;
    std::vector<Vec3> velocity;
    std::vector<Vec3> direction;
};

struct SpectralCase {
    std::string name;
    bool passed = false;
    bool finite = false;
    bool fixed_branches = false;
    bool jacobi_converged = false;
    std::size_t dimension = 0;
    int active_pressure_count = 0;
    int negative_eigenvalue_count = 0;
    int jacobi_pivots = 0;
    double active_margin = 0.0;
    double hvp_fd_error = 0.0;
    double symmetry_error = 0.0;
    double dense_product_error = 0.0;
    double off_particle_block_ratio = 0.0;
    double minimum_eigenvalue = 0.0;
    double maximum_eigenvalue = 0.0;
    double positive_condition_estimate = 0.0;
};

double spectral_directional_gradient_error(const SpectralFixture& fixture) {
    std::vector<Vec3> direction = fixture.direction;
    const double direction_scale = vector_norm(direction);
    for (Vec3& value : direction) {
        value = value / direction_scale;
    }
    const std::vector<Vec3> y =
        predict(fixture.config, fixture.x, fixture.velocity);
    const Evaluation base = evaluate(fixture.config, fixture.x, y, y);
    const double epsilon = fixture.config.spacing * 1.0e-7;
    std::vector<Vec3> plus = y;
    std::vector<Vec3> minus = y;
    for (std::size_t i = 0; i < direction.size(); ++i) {
        plus[i] += epsilon * direction[i];
        minus[i] += -epsilon * direction[i];
    }
    const double finite_difference =
        (evaluate(fixture.config, fixture.x, y, plus).total
            - evaluate(fixture.config, fixture.x, y, minus).total)
        / (2.0 * epsilon);
    const double analytic = vector_dot(base.gradient, direction);
    return relative_error(finite_difference, analytic);
}

std::vector<int> branch_signature(
    const Config& config, const std::vector<Vec3>& y) {
    std::vector<int> result;
    for (std::size_t i = 0; i < y.size(); ++i) {
        for (std::size_t j = i + 1; j < y.size(); ++j) {
            const double radius = norm(y[i] - y[j]);
            const double density_q = 2.0 * radius / config.horizon;
            const int density_branch = radius <= 1.0e-15
                ? 0
                : (density_q < 1.0 ? 1 : (density_q <= 2.0 ? 2 : 3));
            const double surface_q = radius / config.spacing;
            const int surface_branch = radius <= 1.0e-15
                ? 0
                : (surface_q <= 1.0 ? 1 : (surface_q < 3.0 ? 2 : 3));
            result.push_back(density_branch);
            result.push_back(surface_branch);
        }
    }
    return result;
}

std::vector<int> pressure_active_signature(
    const Config& config, const std::vector<Vec3>& y) {
    const std::vector<double> density = densities(config, y);
    std::vector<int> result(density.size());
    for (std::size_t i = 0; i < density.size(); ++i) {
        result[i] = density[i] > config.rest_density ? 1 : 0;
    }
    return result;
}

bool jacobi_eigenvalues(
    const std::vector<double>& matrix,
    std::size_t dimension,
    std::vector<double>& eigenvalues,
    int& pivots) {
    std::vector<double> value(matrix.size());
    for (std::size_t row = 0; row < dimension; ++row) {
        for (std::size_t column = 0; column < dimension; ++column) {
            value[row * dimension + column] = 0.5
                * (matrix[row * dimension + column]
                    + matrix[column * dimension + row]);
        }
    }
    const int maximum_pivots = static_cast<int>(64 * dimension * dimension);
    pivots = 0;
    for (; pivots < maximum_pivots; ++pivots) {
        std::size_t p = 0;
        std::size_t q = 0;
        double largest = 0.0;
        for (std::size_t row = 0; row < dimension; ++row) {
            for (std::size_t column = row + 1; column < dimension; ++column) {
                const double candidate =
                    std::abs(value[row * dimension + column]);
                if (candidate > largest) {
                    largest = candidate;
                    p = row;
                    q = column;
                }
            }
        }
        double maximum_diagonal = 0.0;
        for (std::size_t i = 0; i < dimension; ++i) {
            maximum_diagonal = std::max(
                maximum_diagonal, std::abs(value[i * dimension + i]));
        }
        if (largest <= 1.0e-12 * std::max(maximum_diagonal, 1.0)) {
            eigenvalues.resize(dimension);
            for (std::size_t i = 0; i < dimension; ++i) {
                eigenvalues[i] = value[i * dimension + i];
            }
            std::sort(eigenvalues.begin(), eigenvalues.end());
            return all_finite(eigenvalues);
        }

        const double app = value[p * dimension + p];
        const double aqq = value[q * dimension + q];
        const double apq = value[p * dimension + q];
        const double tau = (aqq - app) / (2.0 * apq);
        const double t = tau >= 0.0
            ? 1.0 / (tau + std::sqrt(1.0 + tau * tau))
            : -1.0 / (-tau + std::sqrt(1.0 + tau * tau));
        const double cosine = 1.0 / std::sqrt(1.0 + t * t);
        const double sine = t * cosine;
        for (std::size_t k = 0; k < dimension; ++k) {
            if (k == p || k == q) {
                continue;
            }
            const double akp = value[k * dimension + p];
            const double akq = value[k * dimension + q];
            const double next_kp = cosine * akp - sine * akq;
            const double next_kq = sine * akp + cosine * akq;
            value[k * dimension + p] = next_kp;
            value[p * dimension + k] = next_kp;
            value[k * dimension + q] = next_kq;
            value[q * dimension + k] = next_kq;
        }
        value[p * dimension + p] = cosine * cosine * app
            - 2.0 * sine * cosine * apq + sine * sine * aqq;
        value[q * dimension + q] = sine * sine * app
            + 2.0 * sine * cosine * apq + cosine * cosine * aqq;
        value[p * dimension + q] = 0.0;
        value[q * dimension + p] = 0.0;
    }
    return false;
}

SpectralCase analyze_spectral_fixture(const SpectralFixture& fixture) {
    constexpr double hvp_fd_limit = 2.0e-6;
    constexpr double symmetry_limit = 2.0e-12;
    constexpr double dense_product_limit = 2.0e-12;
    SpectralCase result;
    result.name = fixture.name;
    std::vector<Vec3> direction = fixture.direction;
    const double direction_scale = vector_norm(direction);
    for (Vec3& value : direction) {
        value = value / direction_scale;
    }
    const std::vector<Vec3> y =
        predict(fixture.config, fixture.x, fixture.velocity);
    const std::vector<Vec3> analytic =
        apply_hessian(fixture.config, fixture.x, y, direction);
    const double epsilon = std::ldexp(1.0, -20)
        * std::max(1.0, vector_norm(y));
    std::vector<Vec3> plus = y;
    std::vector<Vec3> minus = y;
    for (std::size_t i = 0; i < y.size(); ++i) {
        plus[i] += epsilon * direction[i];
        minus[i] += -epsilon * direction[i];
    }
    const Evaluation base = evaluate(fixture.config, fixture.x, y, y);
    const Evaluation plus_evaluation =
        evaluate(fixture.config, fixture.x, y, plus);
    const Evaluation minus_evaluation =
        evaluate(fixture.config, fixture.x, y, minus);
    std::vector<Vec3> finite_difference(y.size());
    for (std::size_t i = 0; i < y.size(); ++i) {
        finite_difference[i] =
            (plus_evaluation.gradient[i] - minus_evaluation.gradient[i])
            / (2.0 * epsilon);
    }
    std::vector<Vec3> hvp_difference(y.size());
    for (std::size_t i = 0; i < y.size(); ++i) {
        hvp_difference[i] = analytic[i] - finite_difference[i];
    }
    result.hvp_fd_error = vector_norm(hvp_difference)
        / std::max({vector_norm(analytic), vector_norm(finite_difference), 1.0});

    result.dimension = 3 * y.size();
    std::vector<double> dense(result.dimension * result.dimension);
    for (std::size_t column = 0; column < result.dimension; ++column) {
        std::vector<double> basis(result.dimension);
        basis[column] = 1.0;
        const std::vector<double> image = flatten(apply_hessian(
            fixture.config, fixture.x, y, unflatten(basis)));
        for (std::size_t row = 0; row < result.dimension; ++row) {
            dense[row * result.dimension + column] = image[row];
        }
    }
    double asymmetry_squared = 0.0;
    double dense_squared = 0.0;
    double off_particle_squared = 0.0;
    for (std::size_t row = 0; row < result.dimension; ++row) {
        for (std::size_t column = 0; column < result.dimension; ++column) {
            const double entry = dense[row * result.dimension + column];
            const double difference = entry
                - dense[column * result.dimension + row];
            asymmetry_squared += difference * difference;
            dense_squared += entry * entry;
            if (row / 3 != column / 3) {
                off_particle_squared += entry * entry;
            }
        }
    }
    result.symmetry_error = std::sqrt(asymmetry_squared)
        / std::max(std::sqrt(dense_squared), 1.0);
    result.off_particle_block_ratio = std::sqrt(off_particle_squared)
        / std::max(std::sqrt(dense_squared), 1.0);

    const std::vector<double> flat_direction = flatten(direction);
    const std::vector<double> flat_analytic = flatten(analytic);
    std::vector<double> dense_product(result.dimension);
    for (std::size_t row = 0; row < result.dimension; ++row) {
        for (std::size_t column = 0; column < result.dimension; ++column) {
            dense_product[row] += dense[row * result.dimension + column]
                * flat_direction[column];
        }
    }
    std::vector<double> dense_difference(result.dimension);
    for (std::size_t i = 0; i < result.dimension; ++i) {
        dense_difference[i] = dense_product[i] - flat_analytic[i];
    }
    result.dense_product_error = flat_norm(dense_difference)
        / std::max({flat_norm(dense_product), flat_norm(flat_analytic), 1.0});

    std::vector<double> eigenvalues;
    result.jacobi_converged = jacobi_eigenvalues(
        dense, result.dimension, eigenvalues, result.jacobi_pivots);
    if (!eigenvalues.empty()) {
        result.minimum_eigenvalue = eigenvalues.front();
        result.maximum_eigenvalue = eigenvalues.back();
        const double eigen_scale = std::max(
            std::abs(result.minimum_eigenvalue),
            std::abs(result.maximum_eigenvalue));
        const double sign_threshold = 1.0e-10 * std::max(eigen_scale, 1.0);
        double smallest_positive = std::numeric_limits<double>::infinity();
        double largest_positive = 0.0;
        for (double value : eigenvalues) {
            if (value < -sign_threshold) {
                ++result.negative_eigenvalue_count;
            } else if (value > sign_threshold) {
                smallest_positive = std::min(smallest_positive, value);
                largest_positive = std::max(largest_positive, value);
            }
        }
        result.positive_condition_estimate = largest_positive > 0.0
            ? largest_positive / smallest_positive
            : 0.0;
    }

    const std::vector<double> base_density = densities(fixture.config, y);
    result.active_margin = std::numeric_limits<double>::infinity();
    for (double value : base_density) {
        const double ratio = value / fixture.config.rest_density;
        result.active_margin = std::min(result.active_margin,
            std::abs(ratio - 1.0));
        if (ratio > 1.0) {
            ++result.active_pressure_count;
        }
    }
    result.fixed_branches = branch_signature(fixture.config, y)
            == branch_signature(fixture.config, plus)
        && branch_signature(fixture.config, y)
            == branch_signature(fixture.config, minus)
        && pressure_active_signature(fixture.config, y)
            == pressure_active_signature(fixture.config, plus)
        && pressure_active_signature(fixture.config, y)
            == pressure_active_signature(fixture.config, minus);
    result.finite = base.finite && plus_evaluation.finite
        && minus_evaluation.finite && all_finite(analytic)
        && all_finite(finite_difference) && all_finite(dense)
        && all_finite(eigenvalues) && std::isfinite(result.active_margin)
        && std::isfinite(result.hvp_fd_error)
        && std::isfinite(result.symmetry_error)
        && std::isfinite(result.dense_product_error)
        && std::isfinite(result.positive_condition_estimate);
    result.passed = result.finite && result.fixed_branches
        && result.jacobi_converged && result.hvp_fd_error <= hvp_fd_limit
        && result.symmetry_error <= symmetry_limit
        && result.dense_product_error <= dense_product_limit;
    return result;
}

std::array<SpectralFixture, 4> spectral_fixtures() {
    SpectralFixture pressure;
    pressure.name = "compressed_pair";
    pressure.config.kappa = 500.0;
    pressure.x = {{-0.0225, 0.0, 0.0}, {0.0225, 0.0, 0.0}};
    pressure.velocity.resize(2);
    pressure.config.rest_density =
        pair_density(norm(pressure.x[0] - pressure.x[1]), pressure.config) / 1.1;
    pressure.direction = {
        {0.31, -0.27, 0.11}, {-0.19, 0.41, -0.23},
    };

    const CombinedFixture combined_source = combined_fixture();
    SpectralFixture combined;
    combined.name = "combined_tetrahedron";
    combined.config = combined_source.config;
    combined.x = combined_source.x;
    combined.velocity = combined_source.velocity;
    combined.direction = {
        {0.31, -0.27, 0.11}, {-0.19, 0.41, -0.23},
        {0.17, 0.07, -0.37}, {-0.29, -0.21, 0.49},
    };

    SpectralFixture repulsive;
    repulsive.name = "surface_repulsive_pair";
    repulsive.config.gamma = 1000.0;
    const double repulsive_distance = 0.8 * repulsive.config.spacing;
    repulsive.x = {
        {-0.5 * repulsive_distance, 0.0, 0.0},
        {0.5 * repulsive_distance, 0.0, 0.0},
    };
    repulsive.velocity.resize(2);
    repulsive.direction = {
        {0.23, -0.31, 0.17}, {-0.37, 0.19, 0.29},
    };

    SpectralFixture attractive;
    attractive.name = "surface_attractive_pair";
    attractive.config.gamma = 1000.0;
    const double attractive_distance = 1.7 * attractive.config.spacing;
    attractive.x = {
        {-0.5 * attractive_distance, 0.0, 0.0},
        {0.5 * attractive_distance, 0.0, 0.0},
    };
    attractive.velocity.resize(2);
    attractive.direction = {
        {0.23, -0.31, 0.17}, {-0.37, 0.19, 0.29},
    };
    return {pressure, combined, repulsive, attractive};
}

std::array<SpectralFixture, 6> normalized_spectral_fixtures() {
    const Config defaults;
    const double kernel_scale = reference_lattice_kernel_scale(
        defaults.spacing, defaults.horizon);

    SpectralFixture pressure;
    pressure.name = "normalized_compressed_pair";
    pressure.config.kappa = 500.0;
    pressure.config.kernel_scale = kernel_scale;
    pressure.x = {{-0.0225, 0.0, 0.0}, {0.0225, 0.0, 0.0}};
    pressure.velocity.resize(2);
    pressure.config.rest_density =
        pair_density(norm(pressure.x[0] - pressure.x[1]), pressure.config) / 1.1;
    pressure.direction = {
        {0.31, -0.27, 0.11}, {-0.19, 0.41, -0.23},
    };

    SpectralFixture normal;
    normal.name = "normalized_normal_viscosity_pair";
    normal.config.lambda = 100.0;
    normal.config.kernel_scale = kernel_scale;
    normal.x = {{-0.04, 0.0, 0.0}, {0.04, 0.0, 0.0}};
    normal.velocity = {{1.0, 0.0, 0.0}, {-1.0, 0.0, 0.0}};
    normal.direction = {
        {0.23, -0.31, 0.17}, {-0.37, 0.19, 0.29},
    };

    SpectralFixture tangent;
    tangent.name = "normalized_tangent_viscosity_pair";
    tangent.config.mu = 100.0;
    tangent.config.kernel_scale = kernel_scale;
    tangent.x = normal.x;
    tangent.velocity = {{0.0, 1.0, 0.0}, {0.0, -1.0, 0.0}};
    tangent.direction = normal.direction;

    SpectralFixture repulsive;
    repulsive.name = "normalized_surface_repulsive_pair";
    repulsive.config.gamma = 1000.0;
    repulsive.config.kernel_scale = kernel_scale;
    const double repulsive_distance = 0.8 * repulsive.config.spacing;
    repulsive.x = {
        {-0.5 * repulsive_distance, 0.0, 0.0},
        {0.5 * repulsive_distance, 0.0, 0.0},
    };
    repulsive.velocity.resize(2);
    repulsive.direction = normal.direction;

    SpectralFixture attractive;
    attractive.name = "normalized_surface_attractive_pair";
    attractive.config.gamma = 1000.0;
    attractive.config.kernel_scale = kernel_scale;
    const double attractive_distance = 1.7 * attractive.config.spacing;
    attractive.x = {
        {-0.5 * attractive_distance, 0.0, 0.0},
        {0.5 * attractive_distance, 0.0, 0.0},
    };
    attractive.velocity.resize(2);
    attractive.direction = normal.direction;

    const CombinedFixture combined_source = combined_fixture();
    SpectralFixture combined;
    combined.name = "normalized_combined_tetrahedron";
    combined.config = combined_source.config;
    combined.config.kernel_scale = kernel_scale;
    combined.x = combined_source.x;
    combined.velocity = combined_source.velocity;
    double center_density = combined.config.mass
        * configured_cubic_weight(combined.config, 0.0);
    for (std::size_t j = 1; j < combined.x.size(); ++j) {
        center_density += combined.config.mass * configured_cubic_weight(
            combined.config, norm(combined.x[0] - combined.x[j]));
    }
    combined.config.rest_density = center_density / 1.1;
    combined.direction = {
        {0.31, -0.27, 0.11}, {-0.19, 0.41, -0.23},
        {0.17, 0.07, -0.37}, {-0.29, -0.21, 0.49},
    };
    return {pressure, normal, tangent, repulsive, attractive, combined};
}

void append_spectral_case(std::ostringstream& output, const SpectralCase& value) {
    output << "{\"name\":\"" << value.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL")
           << "\",\"finite\":" << (value.finite ? "true" : "false")
           << ",\"fixed_branches\":"
           << (value.fixed_branches ? "true" : "false")
           << ",\"dimension\":" << value.dimension
           << ",\"active_pressure_count\":" << value.active_pressure_count
           << ",\"active_margin\":" << value.active_margin
           << ",\"hvp_fd_error\":" << value.hvp_fd_error
           << ",\"symmetry_error\":" << value.symmetry_error
           << ",\"dense_product_error\":" << value.dense_product_error
           << ",\"off_particle_block_ratio\":"
           << value.off_particle_block_ratio
           << ",\"jacobi_converged\":"
           << (value.jacobi_converged ? "true" : "false")
           << ",\"jacobi_pivots\":" << value.jacobi_pivots
           << ",\"minimum_eigenvalue\":" << value.minimum_eigenvalue
           << ",\"maximum_eigenvalue\":" << value.maximum_eigenvalue
           << ",\"negative_eigenvalue_count\":"
           << value.negative_eigenvalue_count
           << ",\"positive_condition_estimate\":"
           << value.positive_condition_estimate << '}';
}

struct TrustStep {
    std::vector<Vec3> value;
    std::string reason;
    int iterations = 0;
    int hvp_calls = 0;
    std::uint64_t hvp_nanoseconds = 0;
    bool boundary = false;
    bool finite = true;
};

struct TrustSolveResult {
    bool succeeded = false;
    bool monotonic = true;
    int outer_trials = 0;
    int accepted_trials = 0;
    int rejected_trials = 0;
    int objective_evaluations = 0;
    int hvp_calls = 0;
    int negative_curvature_stops = 0;
    int boundary_stops = 0;
    int residual_stops = 0;
    int dimension_stops = 0;
    int active_set_changes = 0;
    int numerical_floor_stops = 0;
    double minimum_radius = 0.0;
    double maximum_radius = 0.0;
    double minimum_accepted_ratio = 0.0;
    double maximum_accepted_ratio = 0.0;
    double minimum_active_margin = 0.0;
    double final_scaled_displacement_residual = 0.0;
    double numerical_energy_floor = 0.0;
    double numerical_predicted_reduction = 0.0;
    double numerical_scaled_step = 0.0;
    Evaluation initial;
    Evaluation final;
    std::vector<Vec3> position;
    std::string convergence_stop;
    std::string failure;
};

double scaled_displacement_residual(
    const Config& config, const Evaluation& evaluation);

double numerical_energy_floor(const Evaluation& evaluation) {
    const double energy_scale = std::max(
        std::abs(evaluation.inertia) + std::abs(evaluation.pressure)
            + std::abs(evaluation.viscosity) + std::abs(evaluation.surface),
        1.0);
    return 1024.0 * std::numeric_limits<double>::epsilon() * energy_scale;
}

bool numerical_energy_floor_reached(
    const Config& config,
    const Evaluation& evaluation,
    const TrustStep& step,
    double predicted_reduction,
    TrustSolveResult& result) {
    const double energy_floor = numerical_energy_floor(evaluation);
    const double scaled_step = vector_norm(step.value) / config.spacing;
    const bool reached = predicted_reduction > 0.0
        && predicted_reduction <= energy_floor
        && scaled_displacement_residual(config, evaluation) <= 1.0e-7
        && scaled_step <= 1.0e-7;
    if (reached) {
        ++result.numerical_floor_stops;
        result.numerical_energy_floor = energy_floor;
        result.numerical_predicted_reduction = predicted_reduction;
        result.numerical_scaled_step = scaled_step;
        result.convergence_stop = "NUMERICAL_ENERGY_FLOOR";
        result.succeeded = true;
    }
    return reached;
}

double scaled_displacement_residual(
    const Config& config, const Evaluation& evaluation) {
    double maximum = 0.0;
    for (Vec3 gradient : evaluation.gradient) {
        maximum = std::max(maximum, norm(gradient));
    }
    return config.time_step * config.time_step / config.mass
        * maximum / config.spacing;
}

bool trust_converged(
    const Config& config,
    const Evaluation& evaluation,
    bool scale_aware,
    std::string& reason,
    double scaled_displacement_limit = 1.0e-8) {
    if (evaluation.gradient_norm <= 1.0e-10) {
        reason = "RAW_GRADIENT";
        return true;
    }
    if (scale_aware
        && scaled_displacement_residual(config, evaluation)
            <= scaled_displacement_limit) {
        reason = "SCALED_DISPLACEMENT";
        return true;
    }
    return false;
}

double positive_boundary_intersection(
    const std::vector<Vec3>& point,
    const std::vector<Vec3>& direction,
    double radius) {
    const double a = vector_dot(direction, direction);
    const double b = 2.0 * vector_dot(point, direction);
    const double c = vector_dot(point, point) - radius * radius;
    const double discriminant = std::max(b * b - 4.0 * a * c, 0.0);
    return (-b + std::sqrt(discriminant)) / (2.0 * a);
}

double minimum_active_margin(
    const Config& config, const std::vector<Vec3>& y) {
    const std::vector<double> density = densities(config, y);
    double result = std::numeric_limits<double>::infinity();
    for (double value : density) {
        result = std::min(result,
            std::abs(value / config.rest_density - 1.0));
    }
    return result;
}

TrustStep truncated_trust_cg(
    const Config& config,
    const std::vector<Vec3>& x,
    const std::vector<Vec3>& y,
    const std::vector<Vec3>& gradient,
    double radius) {
    TrustStep result;
    result.value.resize(y.size());
    std::vector<Vec3> residual = gradient;
    std::vector<Vec3> direction(gradient.size());
    for (std::size_t i = 0; i < gradient.size(); ++i) {
        direction[i] = -residual[i];
    }
    double residual_squared = vector_dot(residual, residual);
    const double gradient_norm = std::sqrt(residual_squared);
    const double forcing = std::min(0.5, std::sqrt(gradient_norm));
    const int maximum_iterations = static_cast<int>(3 * y.size());
    for (int iteration = 0; iteration < maximum_iterations; ++iteration) {
        const std::vector<Vec3> hessian_direction =
            apply_hessian(config, x, y, direction);
        ++result.hvp_calls;
        const double curvature = vector_dot(direction, hessian_direction);
        if (!std::isfinite(curvature) || !all_finite(hessian_direction)) {
            result.finite = false;
            result.reason = "NONFINITE";
            return result;
        }
        if (curvature <= 0.0) {
            const double tau = positive_boundary_intersection(
                result.value, direction, radius);
            for (std::size_t i = 0; i < result.value.size(); ++i) {
                result.value[i] += tau * direction[i];
            }
            result.reason = "NEGATIVE_CURVATURE";
            result.boundary = true;
            result.iterations = iteration + 1;
            return result;
        }

        const double alpha = residual_squared / curvature;
        std::vector<Vec3> candidate = result.value;
        for (std::size_t i = 0; i < candidate.size(); ++i) {
            candidate[i] += alpha * direction[i];
        }
        if (vector_norm(candidate) >= radius) {
            const double tau = positive_boundary_intersection(
                result.value, direction, radius);
            for (std::size_t i = 0; i < result.value.size(); ++i) {
                result.value[i] += tau * direction[i];
            }
            result.reason = "BOUNDARY";
            result.boundary = true;
            result.iterations = iteration + 1;
            return result;
        }
        result.value = candidate;

        std::vector<Vec3> next_residual = residual;
        for (std::size_t i = 0; i < next_residual.size(); ++i) {
            next_residual[i] += alpha * hessian_direction[i];
        }
        const double next_squared = vector_dot(next_residual, next_residual);
        if (std::sqrt(next_squared) <= forcing * gradient_norm) {
            result.reason = "RESIDUAL";
            result.iterations = iteration + 1;
            return result;
        }
        const double beta = next_squared / residual_squared;
        for (std::size_t i = 0; i < direction.size(); ++i) {
            direction[i] = -next_residual[i] + beta * direction[i];
        }
        residual = next_residual;
        residual_squared = next_squared;
    }
    result.reason = "DIMENSION_LIMIT";
    result.iterations = maximum_iterations;
    return result;
}

TrustSolveResult solve_trust_region(
    const Config& config,
    const std::vector<Vec3>& x,
    const std::vector<Vec3>& velocity,
    bool scale_aware_stop = false,
    bool numerical_floor_stop = false) {
    constexpr int maximum_outer_trials = 64;
    constexpr double accept_ratio = 0.1;
    const std::vector<Vec3> y_star = predict(config, x, velocity);
    std::vector<Vec3> y = y_star;
    Evaluation current = evaluate(config, x, y_star, y);
    TrustSolveResult result;
    result.initial = current;
    result.objective_evaluations = 1;
    double radius = config.spacing;
    const double minimum_radius = std::ldexp(config.spacing, -40);
    const double maximum_radius = 4.0 * config.spacing;
    result.minimum_radius = radius;
    result.maximum_radius = radius;
    result.minimum_accepted_ratio = std::numeric_limits<double>::infinity();
    result.maximum_accepted_ratio = -std::numeric_limits<double>::infinity();
    result.minimum_active_margin = minimum_active_margin(config, y);
    std::vector<int> active_signature = pressure_active_signature(config, y);
    if (!current.finite) {
        result.failure = "NONFINITE_INITIAL_STATE";
        result.final = current;
        return result;
    }

    for (int outer = 0; outer < maximum_outer_trials; ++outer) {
        if (trust_converged(
                config, current, scale_aware_stop, result.convergence_stop)) {
            result.succeeded = true;
            break;
        }
        const TrustStep step = truncated_trust_cg(
            config, x, y, current.gradient, radius);
        result.hvp_calls += step.hvp_calls;
        if (step.reason == "NEGATIVE_CURVATURE") {
            ++result.negative_curvature_stops;
        } else if (step.reason == "BOUNDARY") {
            ++result.boundary_stops;
        } else if (step.reason == "RESIDUAL") {
            ++result.residual_stops;
        } else if (step.reason == "DIMENSION_LIMIT") {
            ++result.dimension_stops;
        }
        ++result.outer_trials;
        if (!step.finite || !all_finite(step.value)) {
            result.failure = "NONFINITE_INNER_STEP";
            break;
        }

        const std::vector<Vec3> hessian_step =
            apply_hessian(config, x, y, step.value);
        ++result.hvp_calls;
        const double predicted_reduction = -(
            vector_dot(current.gradient, step.value)
            + 0.5 * vector_dot(step.value, hessian_step));
        bool valid_model = std::isfinite(predicted_reduction)
            && predicted_reduction > 0.0;
        if (numerical_floor_stop && valid_model
            && numerical_energy_floor_reached(config, current, step,
                predicted_reduction, result)) {
            break;
        }
        double ratio = -std::numeric_limits<double>::infinity();
        Evaluation trial_evaluation;
        std::vector<Vec3> trial = y;
        if (valid_model) {
            for (std::size_t i = 0; i < trial.size(); ++i) {
                trial[i] += step.value[i];
            }
            trial_evaluation = evaluate(config, x, y_star, trial);
            ++result.objective_evaluations;
            result.minimum_active_margin = std::min(
                result.minimum_active_margin,
                minimum_active_margin(config, trial));
            const double actual_reduction = current.total - trial_evaluation.total;
            ratio = actual_reduction / predicted_reduction;
            valid_model = trial_evaluation.finite
                && std::isfinite(ratio) && actual_reduction > 0.0;
        }

        if (valid_model && ratio >= accept_ratio) {
            const double allowance = ENERGY_ALLOWANCE
                * std::max({std::abs(current.total),
                    std::abs(trial_evaluation.total), 1.0});
            result.monotonic = result.monotonic
                && trial_evaluation.total <= current.total + allowance;
            const std::vector<int> next_active =
                pressure_active_signature(config, trial);
            if (next_active != active_signature) {
                ++result.active_set_changes;
            }
            active_signature = next_active;
            y = trial;
            current = trial_evaluation;
            ++result.accepted_trials;
            result.minimum_accepted_ratio = std::min(
                result.minimum_accepted_ratio, ratio);
            result.maximum_accepted_ratio = std::max(
                result.maximum_accepted_ratio, ratio);
        } else {
            ++result.rejected_trials;
        }

        if (!valid_model || ratio < 0.25) {
            radius *= 0.25;
        } else if (ratio > 0.75 && step.boundary) {
            radius = std::min(2.0 * radius, maximum_radius);
        }
        result.minimum_radius = std::min(result.minimum_radius, radius);
        result.maximum_radius = std::max(result.maximum_radius, radius);
        if (radius < minimum_radius) {
            result.failure = "MINIMUM_TRUST_RADIUS";
            break;
        }
    }
    if (trust_converged(
            config, current, scale_aware_stop, result.convergence_stop)
        && result.failure.empty()) {
        result.succeeded = true;
    }
    if (!result.succeeded && result.failure.empty()) {
        result.failure = "OUTER_TRIAL_LIMIT";
    }
    if (!std::isfinite(result.minimum_accepted_ratio)) {
        result.minimum_accepted_ratio = 0.0;
        result.maximum_accepted_ratio = 0.0;
    }
    result.succeeded = result.succeeded && result.monotonic && current.finite;
    result.final = current;
    result.final_scaled_displacement_residual =
        scaled_displacement_residual(config, current);
    result.position = y;
    return result;
}

struct TrustCase {
    std::string name;
    SolveResult baseline;
    TrustSolveResult candidate;
    int baseline_evaluations = 0;
    int evaluation_limit = 0;
    bool passed = false;
};

bool trust_quality_passed(const TrustCase& value) {
    const double objective_allowance = 1.0e-10
        * std::max(std::abs(value.baseline.final.total), 1.0);
    const double gradient_limit = std::max(
        2.0 * value.baseline.final.gradient_norm, 1.0e-8);
    return value.candidate.succeeded && value.candidate.monotonic
        && value.candidate.failure.empty()
        && value.candidate.final.total
            <= value.baseline.final.total + objective_allowance
        && value.candidate.final.gradient_norm <= gradient_limit
        && value.candidate.final.internal_momentum_residual <= CONSERVATION_LIMIT
        && value.candidate.objective_evaluations <= value.evaluation_limit;
}

TrustCase make_trust_compression_case() {
    Config config;
    config.kappa = 500.0;
    const std::vector<Vec3> x = {{-0.025, 0.0, 0.0}, {0.025, 0.0, 0.0}};
    const std::vector<Vec3> velocity(2);
    config.rest_density = pair_density(norm(x[0] - x[1]), config) / 1.1;
    TrustCase result;
    result.name = "compressed_pair";
    result.baseline = solve(config, x, velocity);
    result.candidate = solve_trust_region(config, x, velocity);
    result.baseline_evaluations = 1 + result.baseline.iterations
        + result.baseline.backtracks;
    result.evaluation_limit = 454;
    result.passed = trust_quality_passed(result);
    return result;
}

TrustCase make_trust_combined_case() {
    const CombinedFixture fixture = combined_fixture();
    TrustCase result;
    result.name = "combined_tetrahedron";
    result.baseline = solve(fixture.config, fixture.x, fixture.velocity);
    result.candidate = solve_trust_region(
        fixture.config, fixture.x, fixture.velocity);
    result.baseline_evaluations = 1 + result.baseline.iterations
        + result.baseline.backtracks;
    result.evaluation_limit = 224;
    result.passed = trust_quality_passed(result);
    return result;
}

void append_trust_case(std::ostringstream& output, const TrustCase& value) {
    output << "{\"name\":\"" << value.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL")
           << "\",\"baseline\":{\"final_objective\":"
           << value.baseline.final.total
           << ",\"final_gradient_norm\":"
           << value.baseline.final.gradient_norm
           << ",\"objective_evaluations\":"
           << value.baseline_evaluations << '}'
           << ",\"candidate\":{\"failure\":\""
           << value.candidate.failure << '"'
           << ",\"initial_objective\":"
           << value.candidate.initial.total
           << ",\"final_objective\":" << value.candidate.final.total
           << ",\"initial_gradient_norm\":"
           << value.candidate.initial.gradient_norm
           << ",\"final_gradient_norm\":"
           << value.candidate.final.gradient_norm
           << ",\"outer_trials\":" << value.candidate.outer_trials
           << ",\"accepted_trials\":" << value.candidate.accepted_trials
           << ",\"rejected_trials\":" << value.candidate.rejected_trials
           << ",\"objective_evaluations\":"
           << value.candidate.objective_evaluations
           << ",\"hvp_calls\":" << value.candidate.hvp_calls
           << ",\"negative_curvature_stops\":"
           << value.candidate.negative_curvature_stops
           << ",\"boundary_stops\":" << value.candidate.boundary_stops
           << ",\"residual_stops\":" << value.candidate.residual_stops
           << ",\"dimension_stops\":" << value.candidate.dimension_stops
           << ",\"active_set_changes\":"
           << value.candidate.active_set_changes
           << ",\"minimum_active_margin\":"
           << value.candidate.minimum_active_margin
           << ",\"minimum_radius\":" << value.candidate.minimum_radius
           << ",\"maximum_radius\":" << value.candidate.maximum_radius
           << ",\"minimum_accepted_ratio\":"
           << value.candidate.minimum_accepted_ratio
           << ",\"maximum_accepted_ratio\":"
           << value.candidate.maximum_accepted_ratio
           << ",\"internal_momentum_residual\":"
           << value.candidate.final.internal_momentum_residual << '}'
           << ",\"evaluation_limit\":" << value.evaluation_limit << '}';
}

struct BlockMetric {
    std::vector<Mat3> value;
    std::vector<Mat3> inverse;
};

BlockMetric make_block_metric(
    const Config& config,
    const std::vector<Vec3>& x,
    const std::vector<Vec3>& y) {
    const double inertia_scale = config.mass
        / (config.time_step * config.time_step);
    BlockMetric result;
    result.value = block_preconditioner(config, x, y);
    result.inverse.resize(result.value.size());
    for (std::size_t i = 0; i < result.value.size(); ++i) {
        result.value[i] = result.value[i] * (1.0 / inertia_scale);
        result.inverse[i] = inverse_without_regularization(result.value[i]);
    }
    return result;
}

double metric_dot(
    const std::vector<Vec3>& lhs,
    const BlockMetric& metric,
    const std::vector<Vec3>& rhs) {
    double result = 0.0;
    for (std::size_t i = 0; i < lhs.size(); ++i) {
        result += dot(lhs[i], metric.value[i] * rhs[i]);
    }
    return result;
}

std::vector<Vec3> apply_inverse_metric(
    const BlockMetric& metric, const std::vector<Vec3>& value) {
    std::vector<Vec3> result(value.size());
    for (std::size_t i = 0; i < value.size(); ++i) {
        result[i] = metric.inverse[i] * value[i];
    }
    return result;
}

double metric_boundary_intersection(
    const std::vector<Vec3>& point,
    const std::vector<Vec3>& direction,
    const BlockMetric& metric,
    double radius) {
    const double a = metric_dot(direction, metric, direction);
    const double b = 2.0 * metric_dot(point, metric, direction);
    const double c = metric_dot(point, metric, point) - radius * radius;
    const double discriminant = std::max(b * b - 4.0 * a * c, 0.0);
    return (-b + std::sqrt(discriminant)) / (2.0 * a);
}

TrustStep truncated_preconditioned_trust_cg(
    const Config& config,
    const std::vector<Vec3>& x,
    const std::vector<Vec3>& y,
    const std::vector<Vec3>& gradient,
    const BlockMetric& metric,
    double radius) {
    TrustStep result;
    result.value.resize(y.size());
    std::vector<Vec3> residual = gradient;
    std::vector<Vec3> preconditioned =
        apply_inverse_metric(metric, residual);
    std::vector<Vec3> direction(preconditioned.size());
    for (std::size_t i = 0; i < direction.size(); ++i) {
        direction[i] = -preconditioned[i];
    }
    double residual_preconditioned = vector_dot(residual, preconditioned);
    const double initial_residual = std::sqrt(residual_preconditioned);
    const double forcing = std::min(0.5, std::sqrt(initial_residual));
    const int maximum_iterations = static_cast<int>(3 * y.size());
    for (int iteration = 0; iteration < maximum_iterations; ++iteration) {
        const std::vector<Vec3> hessian_direction =
            apply_hessian(config, x, y, direction);
        ++result.hvp_calls;
        const double curvature = vector_dot(direction, hessian_direction);
        if (!std::isfinite(curvature) || !all_finite(hessian_direction)) {
            result.finite = false;
            result.reason = "NONFINITE";
            return result;
        }
        if (curvature <= 0.0) {
            const double tau = metric_boundary_intersection(
                result.value, direction, metric, radius);
            for (std::size_t i = 0; i < result.value.size(); ++i) {
                result.value[i] += tau * direction[i];
            }
            result.reason = "NEGATIVE_CURVATURE";
            result.boundary = true;
            result.iterations = iteration + 1;
            return result;
        }

        const double alpha = residual_preconditioned / curvature;
        std::vector<Vec3> candidate = result.value;
        for (std::size_t i = 0; i < candidate.size(); ++i) {
            candidate[i] += alpha * direction[i];
        }
        if (metric_dot(candidate, metric, candidate) >= radius * radius) {
            const double tau = metric_boundary_intersection(
                result.value, direction, metric, radius);
            for (std::size_t i = 0; i < result.value.size(); ++i) {
                result.value[i] += tau * direction[i];
            }
            result.reason = "BOUNDARY";
            result.boundary = true;
            result.iterations = iteration + 1;
            return result;
        }
        result.value = candidate;

        std::vector<Vec3> next_residual = residual;
        for (std::size_t i = 0; i < next_residual.size(); ++i) {
            next_residual[i] += alpha * hessian_direction[i];
        }
        std::vector<Vec3> next_preconditioned =
            apply_inverse_metric(metric, next_residual);
        const double next_scalar =
            vector_dot(next_residual, next_preconditioned);
        if (std::sqrt(next_scalar) <= forcing * initial_residual) {
            result.reason = "RESIDUAL";
            result.iterations = iteration + 1;
            return result;
        }
        const double beta = next_scalar / residual_preconditioned;
        for (std::size_t i = 0; i < direction.size(); ++i) {
            direction[i] = -next_preconditioned[i] + beta * direction[i];
        }
        residual = next_residual;
        residual_preconditioned = next_scalar;
    }
    result.reason = "DIMENSION_LIMIT";
    result.iterations = maximum_iterations;
    return result;
}

TrustSolveResult solve_preconditioned_trust_region(
    const Config& config,
    const std::vector<Vec3>& x,
    const std::vector<Vec3>& velocity,
    bool scale_aware_stop = false) {
    constexpr int maximum_outer_trials = 64;
    constexpr double accept_ratio = 0.1;
    const std::vector<Vec3> y_star = predict(config, x, velocity);
    std::vector<Vec3> y = y_star;
    Evaluation current = evaluate(config, x, y_star, y);
    TrustSolveResult result;
    result.initial = current;
    result.objective_evaluations = 1;
    double radius = config.spacing;
    const double minimum_radius = std::ldexp(config.spacing, -40);
    const double maximum_radius = 4.0 * config.spacing;
    result.minimum_radius = radius;
    result.maximum_radius = radius;
    result.minimum_accepted_ratio = std::numeric_limits<double>::infinity();
    result.maximum_accepted_ratio = -std::numeric_limits<double>::infinity();
    result.minimum_active_margin = minimum_active_margin(config, y);
    std::vector<int> active_signature = pressure_active_signature(config, y);
    if (!current.finite) {
        result.failure = "NONFINITE_INITIAL_STATE";
        result.final = current;
        return result;
    }

    for (int outer = 0; outer < maximum_outer_trials; ++outer) {
        if (trust_converged(
                config, current, scale_aware_stop, result.convergence_stop)) {
            result.succeeded = true;
            break;
        }
        const BlockMetric metric = make_block_metric(config, x, y);
        const TrustStep step = truncated_preconditioned_trust_cg(
            config, x, y, current.gradient, metric, radius);
        result.hvp_calls += step.hvp_calls;
        if (step.reason == "NEGATIVE_CURVATURE") {
            ++result.negative_curvature_stops;
        } else if (step.reason == "BOUNDARY") {
            ++result.boundary_stops;
        } else if (step.reason == "RESIDUAL") {
            ++result.residual_stops;
        } else if (step.reason == "DIMENSION_LIMIT") {
            ++result.dimension_stops;
        }
        ++result.outer_trials;
        if (!step.finite || !all_finite(step.value)) {
            result.failure = "NONFINITE_INNER_STEP";
            break;
        }
        const std::vector<Vec3> hessian_step =
            apply_hessian(config, x, y, step.value);
        ++result.hvp_calls;
        const double predicted_reduction = -(
            vector_dot(current.gradient, step.value)
            + 0.5 * vector_dot(step.value, hessian_step));
        bool valid_model = std::isfinite(predicted_reduction)
            && predicted_reduction > 0.0;
        double ratio = -std::numeric_limits<double>::infinity();
        Evaluation trial_evaluation;
        std::vector<Vec3> trial = y;
        if (valid_model) {
            for (std::size_t i = 0; i < trial.size(); ++i) {
                trial[i] += step.value[i];
            }
            trial_evaluation = evaluate(config, x, y_star, trial);
            ++result.objective_evaluations;
            result.minimum_active_margin = std::min(
                result.minimum_active_margin,
                minimum_active_margin(config, trial));
            const double actual_reduction = current.total - trial_evaluation.total;
            ratio = actual_reduction / predicted_reduction;
            valid_model = trial_evaluation.finite
                && std::isfinite(ratio) && actual_reduction > 0.0;
        }
        if (valid_model && ratio >= accept_ratio) {
            const double allowance = ENERGY_ALLOWANCE
                * std::max({std::abs(current.total),
                    std::abs(trial_evaluation.total), 1.0});
            result.monotonic = result.monotonic
                && trial_evaluation.total <= current.total + allowance;
            const std::vector<int> next_active =
                pressure_active_signature(config, trial);
            if (next_active != active_signature) {
                ++result.active_set_changes;
            }
            active_signature = next_active;
            y = trial;
            current = trial_evaluation;
            ++result.accepted_trials;
            result.minimum_accepted_ratio = std::min(
                result.minimum_accepted_ratio, ratio);
            result.maximum_accepted_ratio = std::max(
                result.maximum_accepted_ratio, ratio);
        } else {
            ++result.rejected_trials;
        }
        if (!valid_model || ratio < 0.25) {
            radius *= 0.25;
        } else if (ratio > 0.75 && step.boundary) {
            radius = std::min(2.0 * radius, maximum_radius);
        }
        result.minimum_radius = std::min(result.minimum_radius, radius);
        result.maximum_radius = std::max(result.maximum_radius, radius);
        if (radius < minimum_radius) {
            result.failure = "MINIMUM_TRUST_RADIUS";
            break;
        }
    }
    if (trust_converged(
            config, current, scale_aware_stop, result.convergence_stop)
        && result.failure.empty()) {
        result.succeeded = true;
    }
    if (!result.succeeded && result.failure.empty()) {
        result.failure = "OUTER_TRIAL_LIMIT";
    }
    if (!std::isfinite(result.minimum_accepted_ratio)) {
        result.minimum_accepted_ratio = 0.0;
        result.maximum_accepted_ratio = 0.0;
    }
    result.succeeded = result.succeeded && result.monotonic && current.finite;
    result.final = current;
    result.final_scaled_displacement_residual =
        scaled_displacement_residual(config, current);
    result.position = y;
    return result;
}

struct BlockScalingCase {
    std::string name;
    std::size_t particle_count = 0;
    TrustSolveResult baseline;
    TrustSolveResult candidate;
    bool scale_aware = false;
    bool passed = false;
};

BlockScalingCase make_block_scaling_case(int side, bool scale_aware = false) {
    constexpr double pitch = 0.04;
    Config config;
    config.kappa = 200.0;
    config.lambda = 20.0;
    config.mu = 10.0;
    config.gamma = 100.0;
    std::vector<Vec3> x;
    std::vector<Vec3> velocity;
    const double center = 0.5 * static_cast<double>(side - 1);
    for (int iz = 0; iz < side; ++iz) {
        for (int iy = 0; iy < side; ++iy) {
            for (int ix = 0; ix < side; ++ix) {
                const Vec3 position = {
                    (static_cast<double>(ix) - center) * pitch,
                    (static_cast<double>(iy) - center) * pitch,
                    (static_cast<double>(iz) - center) * pitch,
                };
                x.push_back(position);
                velocity.push_back({
                    -0.35 * position.x + 0.08 * position.y,
                    -0.25 * position.y - 0.06 * position.z,
                    -0.30 * position.z + 0.05 * position.x,
                });
            }
        }
    }
    const std::vector<Vec3> y_star = predict(config, x, velocity);
    const std::vector<double> density = densities(config, y_star);
    config.rest_density =
        *std::max_element(density.begin(), density.end()) / 1.05;
    BlockScalingCase result;
    result.name = "lattice_" + std::to_string(side) + "x"
        + std::to_string(side) + "x" + std::to_string(side);
    result.particle_count = x.size();
    result.scale_aware = scale_aware;
    result.baseline = solve_trust_region(config, x, velocity, scale_aware);
    result.candidate = solve_preconditioned_trust_region(
        config, x, velocity, scale_aware);
    const double objective_allowance = 1.0e-10
        * std::max(std::abs(result.baseline.final.total), 1.0);
    const bool residual_quality = scale_aware
        ? result.candidate.final_scaled_displacement_residual
            <= std::max(
                2.0 * result.baseline.final_scaled_displacement_residual,
                1.0e-8)
        : result.candidate.final.gradient_norm
            <= std::max(2.0 * result.baseline.final.gradient_norm, 1.0e-8);
    result.passed = result.baseline.succeeded && result.candidate.succeeded
        && result.baseline.failure.empty() && result.candidate.failure.empty()
        && result.candidate.final.total
            <= result.baseline.final.total + objective_allowance
        && residual_quality
        && result.candidate.final.internal_momentum_residual <= CONSERVATION_LIMIT
        && result.candidate.objective_evaluations
            <= result.baseline.objective_evaluations
        && result.candidate.hvp_calls <= result.baseline.hvp_calls;
    return result;
}

void append_block_scaling_case(
    std::ostringstream& output,
    const BlockScalingCase& value,
    bool include_scaled = false) {
    const auto append_solver = [&](const TrustSolveResult& solver) {
        output << "{\"failure\":\"" << solver.failure << '"'
               << ",\"final_objective\":" << solver.final.total
               << ",\"final_gradient_norm\":" << solver.final.gradient_norm;
        if (include_scaled) {
            output << ",\"scaled_displacement_residual\":"
                   << solver.final_scaled_displacement_residual
                   << ",\"convergence_stop\":\""
                   << solver.convergence_stop << '"';
        }
        output
               << ",\"outer_trials\":" << solver.outer_trials
               << ",\"accepted_trials\":" << solver.accepted_trials
               << ",\"rejected_trials\":" << solver.rejected_trials
               << ",\"objective_evaluations\":"
               << solver.objective_evaluations
               << ",\"hvp_calls\":" << solver.hvp_calls
               << ",\"negative_curvature_stops\":"
               << solver.negative_curvature_stops
               << ",\"boundary_stops\":" << solver.boundary_stops
               << ",\"active_set_changes\":" << solver.active_set_changes
               << ",\"minimum_radius\":" << solver.minimum_radius
               << ",\"maximum_radius\":" << solver.maximum_radius
               << ",\"internal_momentum_residual\":"
               << solver.final.internal_momentum_residual << '}';
    };
    output << "{\"name\":\"" << value.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL")
           << "\",\"particles\":" << value.particle_count
           << ",\"baseline\":";
    append_solver(value.baseline);
    output << ",\"candidate\":";
    append_solver(value.candidate);
    output << '}';
}

struct NeighborhoodFixture {
    std::string name;
    Config config;
    std::vector<Vec3> x;
    std::vector<Vec3> velocity;
    std::vector<Vec3> direction;
};

struct NeighborhoodCase {
    std::string name;
    bool passed = false;
    bool pair_exact = false;
    bool evaluation_exact = false;
    bool hvp_exact = false;
    std::size_t particles = 0;
    std::size_t current_pairs = 0;
    std::size_t reference_pairs = 0;
    std::string pair_sha256;
};

bool exact_vec3(Vec3 lhs, Vec3 rhs) {
    return lhs.x == rhs.x && lhs.y == rhs.y && lhs.z == rhs.z;
}

bool exact_vectors(
    const std::vector<Vec3>& lhs, const std::vector<Vec3>& rhs) {
    if (lhs.size() != rhs.size()) {
        return false;
    }
    for (std::size_t i = 0; i < lhs.size(); ++i) {
        if (!exact_vec3(lhs[i], rhs[i])) {
            return false;
        }
    }
    return true;
}

bool exact_evaluation(const Evaluation& lhs, const Evaluation& rhs) {
    return lhs.total == rhs.total
        && lhs.inertia == rhs.inertia
        && lhs.pressure == rhs.pressure
        && lhs.viscosity == rhs.viscosity
        && lhs.surface == rhs.surface
        && lhs.gradient_norm == rhs.gradient_norm
        && lhs.internal_momentum_residual == rhs.internal_momentum_residual
        && lhs.minimum_density_ratio == rhs.minimum_density_ratio
        && lhs.maximum_density_ratio == rhs.maximum_density_ratio
        && lhs.finite == rhs.finite
        && exact_vectors(lhs.gradient, rhs.gradient);
}

std::string hash_pair_lists(
    const std::vector<ParticlePair>& current,
    const std::vector<ParticlePair>& reference) {
    std::string material = "current|";
    for (const ParticlePair pair : current) {
        material += std::to_string(pair.i) + ':' + std::to_string(pair.j) + '|';
    }
    material += "reference|";
    for (const ParticlePair pair : reference) {
        material += std::to_string(pair.i) + ':' + std::to_string(pair.j) + '|';
    }
    return sha256_hex(material);
}

NeighborhoodFixture make_neighborhood_lattice_fixture(int side) {
    constexpr double pitch = 0.04;
    NeighborhoodFixture fixture;
    fixture.name = "lattice_" + std::to_string(side) + "x"
        + std::to_string(side) + "x" + std::to_string(side);
    fixture.config.kappa = 200.0;
    fixture.config.lambda = 20.0;
    fixture.config.mu = 10.0;
    fixture.config.gamma = 100.0;
    const double center = 0.5 * static_cast<double>(side - 1);
    for (int iz = 0; iz < side; ++iz) {
        for (int iy = 0; iy < side; ++iy) {
            for (int ix = 0; ix < side; ++ix) {
                const Vec3 position = {
                    (static_cast<double>(ix) - center) * pitch,
                    (static_cast<double>(iy) - center) * pitch,
                    (static_cast<double>(iz) - center) * pitch,
                };
                const std::size_t index = fixture.x.size();
                fixture.x.push_back(position);
                fixture.velocity.push_back({
                    -0.35 * position.x + 0.08 * position.y,
                    -0.25 * position.y - 0.06 * position.z,
                    -0.30 * position.z + 0.05 * position.x,
                });
                fixture.direction.push_back({
                    0.31 + 0.01 * static_cast<double>(index),
                    -0.27 + 0.02 * static_cast<double>(index % 5),
                    0.11 - 0.015 * static_cast<double>(index % 7),
                });
            }
        }
    }
    const std::vector<Vec3> y_star =
        predict(fixture.config, fixture.x, fixture.velocity);
    const std::vector<double> density = densities(fixture.config, y_star);
    fixture.config.rest_density =
        *std::max_element(density.begin(), density.end()) / 1.05;
    const double direction_norm = vector_norm(fixture.direction);
    for (Vec3& value : fixture.direction) {
        value = value / direction_norm;
    }
    return fixture;
}

NeighborhoodCase analyze_neighborhood_fixture(
    const NeighborhoodFixture& fixture) {
    NeighborhoodCase result;
    result.name = fixture.name;
    result.particles = fixture.x.size();
    const std::vector<Vec3> y_star =
        predict(fixture.config, fixture.x, fixture.velocity);
    const double current_support = std::max(
        fixture.config.horizon, 3.0 * fixture.config.spacing);
    const std::vector<ParticlePair> current =
        build_cell_pairs(y_star, current_support);
    const std::vector<ParticlePair> reference =
        build_cell_pairs(fixture.x, fixture.config.horizon);
    const std::vector<ParticlePair> repeated_current =
        build_cell_pairs(y_star, current_support);
    const std::vector<ParticlePair> repeated_reference =
        build_cell_pairs(fixture.x, fixture.config.horizon);
    result.current_pairs = current.size();
    result.reference_pairs = reference.size();
    result.pair_sha256 = hash_pair_lists(current, reference);
    result.pair_exact = current == all_pairs_inside(y_star, current_support)
        && reference == all_pairs_inside(fixture.x, fixture.config.horizon)
        && current == repeated_current && reference == repeated_reference
        && result.pair_sha256
            == hash_pair_lists(repeated_current, repeated_reference);

    const Evaluation all_pair_evaluation = evaluate(
        fixture.config, fixture.x, y_star, y_star);
    const Evaluation neighborhood_evaluation = evaluate_with_pairs(
        fixture.config, fixture.x, y_star, y_star, current, reference);
    result.evaluation_exact = exact_evaluation(
        all_pair_evaluation, neighborhood_evaluation);
    const std::vector<Vec3> all_pair_hvp = apply_hessian(
        fixture.config, fixture.x, y_star, fixture.direction);
    const std::vector<Vec3> neighborhood_hvp = apply_hessian_with_pairs(
        fixture.config, fixture.x, y_star, fixture.direction,
        current, reference);
    result.hvp_exact = exact_vectors(all_pair_hvp, neighborhood_hvp);
    result.passed = result.pair_exact && result.evaluation_exact
        && result.hvp_exact && all_pair_evaluation.finite
        && neighborhood_evaluation.finite && all_finite(all_pair_hvp)
        && all_finite(neighborhood_hvp);
    return result;
}

std::array<NeighborhoodFixture, 7> neighborhood_fixtures() {
    const std::array<SpectralFixture, 4> spectral = spectral_fixtures();
    std::array<NeighborhoodFixture, 7> result;
    for (std::size_t i = 0; i < spectral.size(); ++i) {
        result[i].name = spectral[i].name;
        result[i].config = spectral[i].config;
        result[i].x = spectral[i].x;
        result[i].velocity = spectral[i].velocity;
        result[i].direction = spectral[i].direction;
        const double direction_norm = vector_norm(result[i].direction);
        for (Vec3& value : result[i].direction) {
            value = value / direction_norm;
        }
    }
    result[4] = make_neighborhood_lattice_fixture(2);
    result[5] = make_neighborhood_lattice_fixture(3);
    result[6] = make_neighborhood_lattice_fixture(4);
    return result;
}

void append_neighborhood_case(
    std::ostringstream& output, const NeighborhoodCase& value) {
    output << "{\"name\":\"" << value.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL")
           << "\",\"particles\":" << value.particles
           << ",\"current_pairs\":" << value.current_pairs
           << ",\"reference_pairs\":" << value.reference_pairs
           << ",\"pair_exact\":" << (value.pair_exact ? "true" : "false")
           << ",\"evaluation_exact\":"
           << (value.evaluation_exact ? "true" : "false")
           << ",\"hvp_exact\":" << (value.hvp_exact ? "true" : "false")
           << ",\"pair_sha256\":\"" << value.pair_sha256 << "\"}";
}

struct NeighborhoodTrustResult {
    TrustSolveResult solve;
    int pair_builds = 0;
    std::size_t initial_pairs = 0;
    std::size_t final_pairs = 0;
    std::size_t maximum_pairs = 0;
    std::size_t maximum_neighbors = 0;
    std::uint64_t total_nanoseconds = 0;
    std::uint64_t pair_and_adjacency_nanoseconds = 0;
    std::uint64_t objective_gradient_nanoseconds = 0;
    std::uint64_t hvp_nanoseconds = 0;
    std::uint64_t hessian_tape_build_nanoseconds = 0;
    std::size_t maximum_hessian_tape_bytes = 0;
    std::size_t hessian_tape_capacity_bytes = 0;
    std::size_t maximum_active_pressure_centers = 0;
    std::size_t maximum_directed_pressure_records = 0;
    std::size_t reference_viscosity_records = 0;
    std::size_t maximum_surface_records = 0;
    int hessian_tape_hvp_checks = 0;
    struct TrialTrace {
        int trial = 0;
        int inner_iterations = 0;
        int inner_hvp_calls = 0;
        int current_active = 0;
        int trial_active = 0;
        int active_additions = 0;
        int active_removals = 0;
        std::size_t current_pairs = 0;
        std::size_t trial_pairs = 0;
        std::size_t pair_additions = 0;
        std::size_t pair_removals = 0;
        double current_objective = 0.0;
        double current_scaled_residual = 0.0;
        double radius_before = 0.0;
        double radius_after = 0.0;
        double step_norm = 0.0;
        double predicted_reduction = 0.0;
        double energy_floor = 0.0;
        double actual_reduction = 0.0;
        double ratio = 0.0;
        bool valid_model = false;
        bool accepted = false;
        std::string inner_reason;
        std::string current_state_sha256;
    };
    std::vector<TrialTrace> trace;
};

std::string hash_positions(const std::vector<Vec3>& position) {
    std::ostringstream material;
    material << std::setprecision(17);
    for (Vec3 value : position) {
        material << value.x << ',' << value.y << ',' << value.z << '|';
    }
    return sha256_hex(material.str());
}

struct MembershipDelta {
    std::size_t additions = 0;
    std::size_t removals = 0;
};

bool pair_less(const ParticlePair& lhs, const ParticlePair& rhs) {
    return lhs.i < rhs.i || (lhs.i == rhs.i && lhs.j < rhs.j);
}

MembershipDelta pair_membership_delta(
    const std::vector<ParticlePair>& current,
    const std::vector<ParticlePair>& trial) {
    MembershipDelta result;
    std::size_t current_index = 0;
    std::size_t trial_index = 0;
    while (current_index < current.size() && trial_index < trial.size()) {
        if (current[current_index] == trial[trial_index]) {
            ++current_index;
            ++trial_index;
        } else if (pair_less(current[current_index], trial[trial_index])) {
            ++result.removals;
            ++current_index;
        } else {
            ++result.additions;
            ++trial_index;
        }
    }
    result.removals += current.size() - current_index;
    result.additions += trial.size() - trial_index;
    return result;
}

int active_count(const std::vector<int>& signature) {
    int result = 0;
    for (int value : signature) {
        result += value;
    }
    return result;
}

std::pair<int, int> active_membership_delta(
    const std::vector<int>& current,
    const std::vector<int>& trial) {
    int additions = 0;
    int removals = 0;
    for (std::size_t i = 0; i < current.size(); ++i) {
        additions += current[i] == 0 && trial[i] != 0 ? 1 : 0;
        removals += current[i] != 0 && trial[i] == 0 ? 1 : 0;
    }
    return {additions, removals};
}

std::size_t maximum_neighbor_count(
    const std::vector<std::vector<std::size_t>>& adjacency) {
    std::size_t result = 0;
    for (const std::vector<std::size_t>& neighbors : adjacency) {
        result = std::max(result, neighbors.size());
    }
    return result;
}

bool neighborhood_capacity_valid(
    std::size_t particles,
    const std::vector<ParticlePair>& pairs,
    const std::vector<std::vector<std::size_t>>& adjacency) {
    return pairs.size() <= 80 * particles
        && maximum_neighbor_count(adjacency) <= 160;
}

TrustStep truncated_neighborhood_trust_cg(
    const Config& config,
    const std::vector<Vec3>& x,
    const std::vector<Vec3>& y,
    const std::vector<Vec3>& gradient,
    const std::vector<ParticlePair>& current_pairs,
    const std::vector<ParticlePair>& reference_pairs,
    const std::vector<std::vector<std::size_t>>& adjacency,
    double radius,
    bool capture_timing,
    NeighborhoodHvpWorkspace* workspace,
    const NeighborhoodHessianTape* tape) {
    TrustStep result;
    result.value.resize(y.size());
    std::vector<Vec3> residual = gradient;
    std::vector<Vec3> direction(gradient.size());
    for (std::size_t i = 0; i < gradient.size(); ++i) {
        direction[i] = -residual[i];
    }
    double residual_squared = vector_dot(residual, residual);
    const double gradient_norm = std::sqrt(residual_squared);
    const double forcing = std::min(0.5, std::sqrt(gradient_norm));
    const int maximum_iterations = static_cast<int>(3 * y.size());
    for (int iteration = 0; iteration < maximum_iterations; ++iteration) {
        std::chrono::steady_clock::time_point hvp_begin;
        if (capture_timing) {
            hvp_begin = std::chrono::steady_clock::now();
        }
        std::vector<Vec3> baseline_hessian_direction;
        const std::vector<Vec3>* hessian_direction = nullptr;
        if (tape != nullptr && workspace != nullptr) {
            hessian_direction = &apply_hessian_with_tape(
                config, direction, adjacency, *tape, *workspace);
        } else if (workspace != nullptr) {
            hessian_direction = &apply_hessian_with_workspace(
                config, x, y, direction, current_pairs,
                reference_pairs, adjacency, *workspace);
        } else {
            baseline_hessian_direction = apply_hessian_with_adjacency(
                config, x, y, direction, current_pairs,
                reference_pairs, adjacency);
            hessian_direction = &baseline_hessian_direction;
        }
        if (capture_timing) {
            result.hvp_nanoseconds += static_cast<std::uint64_t>(
                std::chrono::duration_cast<std::chrono::nanoseconds>(
                    std::chrono::steady_clock::now() - hvp_begin).count());
        }
        ++result.hvp_calls;
        const double curvature = vector_dot(direction, *hessian_direction);
        if (!std::isfinite(curvature) || !all_finite(*hessian_direction)) {
            result.finite = false;
            result.reason = "NONFINITE";
            return result;
        }
        if (curvature <= 0.0) {
            const double tau = positive_boundary_intersection(
                result.value, direction, radius);
            for (std::size_t i = 0; i < result.value.size(); ++i) {
                result.value[i] += tau * direction[i];
            }
            result.reason = "NEGATIVE_CURVATURE";
            result.boundary = true;
            result.iterations = iteration + 1;
            return result;
        }
        const double alpha = residual_squared / curvature;
        std::vector<Vec3> candidate = result.value;
        for (std::size_t i = 0; i < candidate.size(); ++i) {
            candidate[i] += alpha * direction[i];
        }
        if (vector_norm(candidate) >= radius) {
            const double tau = positive_boundary_intersection(
                result.value, direction, radius);
            for (std::size_t i = 0; i < result.value.size(); ++i) {
                result.value[i] += tau * direction[i];
            }
            result.reason = "BOUNDARY";
            result.boundary = true;
            result.iterations = iteration + 1;
            return result;
        }
        result.value = candidate;
        std::vector<Vec3> next_residual = residual;
        for (std::size_t i = 0; i < next_residual.size(); ++i) {
            next_residual[i] += alpha * (*hessian_direction)[i];
        }
        const double next_squared = vector_dot(next_residual, next_residual);
        if (std::sqrt(next_squared) <= forcing * gradient_norm) {
            result.reason = "RESIDUAL";
            result.iterations = iteration + 1;
            return result;
        }
        const double beta = next_squared / residual_squared;
        for (std::size_t i = 0; i < direction.size(); ++i) {
            direction[i] = -next_residual[i] + beta * direction[i];
        }
        residual = next_residual;
        residual_squared = next_squared;
    }
    result.reason = "DIMENSION_LIMIT";
    result.iterations = maximum_iterations;
    return result;
}

NeighborhoodTrustResult solve_neighborhood_trust_region(
    const Config& config,
    const std::vector<Vec3>& x,
    const std::vector<Vec3>& velocity,
    bool capture_trace = false,
    bool numerical_floor_stop = false,
    bool capture_timing = false,
    bool optimized_hvp = false,
    bool coefficient_tape = false,
    bool verify_tape_hvp = false,
    double scaled_displacement_limit = 1.0e-8) {
    constexpr int maximum_outer_trials = 64;
    constexpr double accept_ratio = 0.1;
    std::chrono::steady_clock::time_point solve_begin;
    if (capture_timing) {
        solve_begin = std::chrono::steady_clock::now();
    }
    const std::vector<Vec3> y_star = predict(config, x, velocity);
    std::vector<Vec3> y = y_star;
    const double current_support = std::max(
        config.horizon, 3.0 * config.spacing);
    NeighborhoodTrustResult result;
    std::chrono::steady_clock::time_point initial_pair_begin;
    if (capture_timing) {
        initial_pair_begin = std::chrono::steady_clock::now();
    }
    std::vector<ParticlePair> reference_pairs =
        build_cell_pairs(x, config.horizon);
    std::vector<ParticlePair> current_pairs =
        build_cell_pairs(y, current_support);
    std::vector<std::vector<std::size_t>> adjacency =
        build_pair_adjacency(y.size(), current_pairs);
    if (capture_timing) {
        result.pair_and_adjacency_nanoseconds +=
            static_cast<std::uint64_t>(
                std::chrono::duration_cast<std::chrono::nanoseconds>(
                    std::chrono::steady_clock::now() - initial_pair_begin)
                    .count());
    }
    result.pair_builds = 2;
    result.initial_pairs = current_pairs.size();
    result.maximum_pairs = std::max(
        current_pairs.size(), reference_pairs.size());
    result.maximum_neighbors = maximum_neighbor_count(adjacency);
    if (!neighborhood_capacity_valid(y.size(), current_pairs, adjacency)
        || reference_pairs.size() > 80 * y.size()) {
        result.solve.failure = "PAIR_CAPACITY";
        return result;
    }
    std::chrono::steady_clock::time_point initial_evaluation_begin;
    if (capture_timing) {
        initial_evaluation_begin = std::chrono::steady_clock::now();
    }
    Evaluation current = evaluate_with_pairs(
        config, x, y_star, y, current_pairs, reference_pairs);
    if (capture_timing) {
        result.objective_gradient_nanoseconds +=
            static_cast<std::uint64_t>(
                std::chrono::duration_cast<std::chrono::nanoseconds>(
                    std::chrono::steady_clock::now()
                        - initial_evaluation_begin).count());
    }
    result.solve.initial = current;
    result.solve.objective_evaluations = 1;
    double radius = config.spacing;
    const double minimum_radius = std::ldexp(config.spacing, -40);
    const double maximum_radius = 4.0 * config.spacing;
    result.solve.minimum_radius = radius;
    result.solve.maximum_radius = radius;
    result.solve.minimum_accepted_ratio =
        std::numeric_limits<double>::infinity();
    result.solve.maximum_accepted_ratio =
        -std::numeric_limits<double>::infinity();
    result.solve.minimum_active_margin = minimum_active_margin(config, y);
    std::vector<int> active_signature = pressure_active_signature(config, y);
    NeighborhoodHvpWorkspace hvp_workspace;
    NeighborhoodHessianTape hessian_tape;
    bool current_tape_valid = false;
    if (coefficient_tape) {
        const std::size_t required = hessian_tape_required_upper_bytes(
            y.size(), current_pairs.size(), reference_pairs.size());
        result.hessian_tape_capacity_bytes = hessian_tape_storage_limit(
            y.size(), std::max(current_pairs.size(), reference_pairs.size()));
        if (required > result.hessian_tape_capacity_bytes) {
            result.solve.failure = "HESSIAN_TAPE_CAPACITY";
            result.solve.final = current;
            return result;
        }
        std::chrono::steady_clock::time_point tape_begin;
        if (capture_timing) {
            tape_begin = std::chrono::steady_clock::now();
        }
        build_reference_hessian_tape(
            config, x, reference_pairs, hessian_tape);
        if (capture_timing) {
            result.hessian_tape_build_nanoseconds +=
                static_cast<std::uint64_t>(
                    std::chrono::duration_cast<std::chrono::nanoseconds>(
                        std::chrono::steady_clock::now() - tape_begin)
                        .count());
        }
        result.reference_viscosity_records =
            hessian_tape.reference_pairs.size();
    }
    if (!current.finite) {
        result.solve.failure = "NONFINITE_INITIAL_STATE";
        result.solve.final = current;
        return result;
    }

    for (int outer = 0; outer < maximum_outer_trials; ++outer) {
        if (trust_converged(
                config, current, true, result.solve.convergence_stop,
                scaled_displacement_limit)) {
            result.solve.succeeded = true;
            break;
        }
        if (coefficient_tape && !current_tape_valid) {
            const std::size_t required = hessian_tape_required_upper_bytes(
                y.size(), current_pairs.size(), reference_pairs.size());
            const std::size_t capacity = hessian_tape_storage_limit(
                y.size(), result.maximum_pairs);
            result.hessian_tape_capacity_bytes = std::max(
                result.hessian_tape_capacity_bytes, capacity);
            if (required > capacity) {
                result.solve.failure = "HESSIAN_TAPE_CAPACITY";
                break;
            }
            std::chrono::steady_clock::time_point tape_begin;
            if (capture_timing) {
                tape_begin = std::chrono::steady_clock::now();
            }
            const bool tape_valid = build_current_hessian_tape(
                config, y, current_pairs, adjacency, hessian_tape);
            if (capture_timing) {
                result.hessian_tape_build_nanoseconds +=
                    static_cast<std::uint64_t>(
                        std::chrono::duration_cast<std::chrono::nanoseconds>(
                            std::chrono::steady_clock::now() - tape_begin)
                            .count());
            }
            if (!tape_valid) {
                result.solve.failure = "HESSIAN_TAPE_ADJACENCY";
                break;
            }
            const std::size_t tape_bytes =
                hessian_tape_storage_bytes(hessian_tape);
            result.maximum_hessian_tape_bytes = std::max(
                result.maximum_hessian_tape_bytes, tape_bytes);
            if (tape_bytes > capacity) {
                result.solve.failure = "HESSIAN_TAPE_CAPACITY";
                break;
            }
            std::size_t active_pressure_centers = 0;
            std::size_t directed_pressure_records = 0;
            for (std::size_t center = 0;
                 center < hessian_tape.pressure_centers.size(); ++center) {
                if (hessian_tape.pressure_centers[center].compression > 0.0) {
                    ++active_pressure_centers;
                    directed_pressure_records += adjacency[center].size();
                }
            }
            std::size_t surface_records = 0;
            for (const CurrentPairHessianCoefficient& coefficient :
                 hessian_tape.current_pairs) {
                surface_records += coefficient.surface_active ? 1 : 0;
            }
            result.maximum_active_pressure_centers = std::max(
                result.maximum_active_pressure_centers,
                active_pressure_centers);
            result.maximum_directed_pressure_records = std::max(
                result.maximum_directed_pressure_records,
                directed_pressure_records);
            result.maximum_surface_records = std::max(
                result.maximum_surface_records, surface_records);
            if (verify_tape_hvp) {
                NeighborhoodHvpWorkspace baseline_workspace;
                NeighborhoodHvpWorkspace tape_workspace;
                const std::vector<Vec3>& baseline_hvp =
                    apply_hessian_with_workspace(config, x, y,
                        current.gradient, current_pairs, reference_pairs,
                        adjacency, baseline_workspace);
                const std::vector<Vec3>& tape_hvp = apply_hessian_with_tape(
                    config, current.gradient, adjacency,
                    hessian_tape, tape_workspace);
                ++result.hessian_tape_hvp_checks;
                if (!exact_vectors(baseline_hvp, tape_hvp)) {
                    result.solve.failure = "HESSIAN_TAPE_HVP_MISMATCH";
                    break;
                }
            }
            current_tape_valid = true;
        }
        const TrustStep step = truncated_neighborhood_trust_cg(
            config, x, y, current.gradient, current_pairs,
            reference_pairs, adjacency, radius, capture_timing,
            optimized_hvp ? &hvp_workspace : nullptr,
            coefficient_tape ? &hessian_tape : nullptr);
        result.solve.hvp_calls += step.hvp_calls;
        result.hvp_nanoseconds += step.hvp_nanoseconds;
        if (step.reason == "NEGATIVE_CURVATURE") {
            ++result.solve.negative_curvature_stops;
        } else if (step.reason == "BOUNDARY") {
            ++result.solve.boundary_stops;
        } else if (step.reason == "RESIDUAL") {
            ++result.solve.residual_stops;
        } else if (step.reason == "DIMENSION_LIMIT") {
            ++result.solve.dimension_stops;
        }
        ++result.solve.outer_trials;
        if (!step.finite || !all_finite(step.value)) {
            result.solve.failure = "NONFINITE_INNER_STEP";
            break;
        }
        std::chrono::steady_clock::time_point model_hvp_begin;
        if (capture_timing) {
            model_hvp_begin = std::chrono::steady_clock::now();
        }
        std::vector<Vec3> baseline_hessian_step;
        const std::vector<Vec3>* hessian_step = nullptr;
        if (coefficient_tape) {
            hessian_step = &apply_hessian_with_tape(
                config, step.value, adjacency,
                hessian_tape, hvp_workspace);
        } else if (optimized_hvp) {
            hessian_step = &apply_hessian_with_workspace(
                config, x, y, step.value, current_pairs,
                reference_pairs, adjacency, hvp_workspace);
        } else {
            baseline_hessian_step = apply_hessian_with_adjacency(
                config, x, y, step.value, current_pairs,
                reference_pairs, adjacency);
            hessian_step = &baseline_hessian_step;
        }
        if (capture_timing) {
            result.hvp_nanoseconds += static_cast<std::uint64_t>(
                std::chrono::duration_cast<std::chrono::nanoseconds>(
                    std::chrono::steady_clock::now() - model_hvp_begin)
                    .count());
        }
        ++result.solve.hvp_calls;
        const double predicted_reduction = -(
            vector_dot(current.gradient, step.value)
            + 0.5 * vector_dot(step.value, *hessian_step));
        bool valid_model = std::isfinite(predicted_reduction)
            && predicted_reduction > 0.0;
        if (numerical_floor_stop && valid_model
            && numerical_energy_floor_reached(config, current, step,
                predicted_reduction, result.solve)) {
            break;
        }
        double ratio = -std::numeric_limits<double>::infinity();
        double actual_reduction = std::numeric_limits<double>::quiet_NaN();
        Evaluation trial_evaluation;
        std::vector<Vec3> trial = y;
        std::vector<ParticlePair> trial_pairs;
        std::vector<std::vector<std::size_t>> trial_adjacency;
        std::vector<int> trial_active_signature = active_signature;
        NeighborhoodTrustResult::TrialTrace trace;
        if (capture_trace) {
            trace.trial = outer + 1;
            trace.inner_iterations = step.iterations;
            trace.inner_hvp_calls = step.hvp_calls;
            trace.current_active = active_count(active_signature);
            trace.current_pairs = current_pairs.size();
            trace.current_objective = current.total;
            trace.current_scaled_residual =
                scaled_displacement_residual(config, current);
            trace.radius_before = radius;
            trace.step_norm = vector_norm(step.value);
            trace.predicted_reduction = predicted_reduction;
            trace.energy_floor = numerical_energy_floor(current);
            trace.inner_reason = step.reason;
            trace.current_state_sha256 = hash_positions(y);
        }
        if (valid_model) {
            for (std::size_t i = 0; i < trial.size(); ++i) {
                trial[i] += step.value[i];
            }
            std::chrono::steady_clock::time_point trial_pair_begin;
            if (capture_timing) {
                trial_pair_begin = std::chrono::steady_clock::now();
            }
            trial_pairs = build_cell_pairs(trial, current_support);
            trial_adjacency = build_pair_adjacency(trial.size(), trial_pairs);
            if (capture_timing) {
                result.pair_and_adjacency_nanoseconds +=
                    static_cast<std::uint64_t>(
                        std::chrono::duration_cast<std::chrono::nanoseconds>(
                            std::chrono::steady_clock::now() - trial_pair_begin)
                            .count());
            }
            ++result.pair_builds;
            result.maximum_pairs = std::max(
                result.maximum_pairs, trial_pairs.size());
            result.maximum_neighbors = std::max(result.maximum_neighbors,
                maximum_neighbor_count(trial_adjacency));
            if (!neighborhood_capacity_valid(
                    trial.size(), trial_pairs, trial_adjacency)) {
                result.solve.failure = "PAIR_CAPACITY";
                break;
            }
            std::chrono::steady_clock::time_point trial_evaluation_begin;
            if (capture_timing) {
                trial_evaluation_begin = std::chrono::steady_clock::now();
            }
            trial_evaluation = evaluate_with_pairs(config, x, y_star, trial,
                trial_pairs, reference_pairs);
            if (capture_timing) {
                result.objective_gradient_nanoseconds +=
                    static_cast<std::uint64_t>(
                        std::chrono::duration_cast<std::chrono::nanoseconds>(
                            std::chrono::steady_clock::now()
                                - trial_evaluation_begin).count());
            }
            ++result.solve.objective_evaluations;
            result.solve.minimum_active_margin = std::min(
                result.solve.minimum_active_margin,
                minimum_active_margin(config, trial));
            actual_reduction = current.total - trial_evaluation.total;
            ratio = actual_reduction / predicted_reduction;
            if (capture_trace) {
                trial_active_signature =
                    pressure_active_signature(config, trial);
            }
            valid_model = trial_evaluation.finite
                && std::isfinite(ratio) && actual_reduction > 0.0;
        }
        MembershipDelta trace_pair_delta;
        std::pair<int, int> trace_active_delta{0, 0};
        if (capture_trace) {
            trace_pair_delta = pair_membership_delta(
                current_pairs, trial_pairs);
            trace_active_delta = active_membership_delta(
                active_signature, trial_active_signature);
        }
        const bool accepted = valid_model && ratio >= accept_ratio;
        if (accepted) {
            const double allowance = ENERGY_ALLOWANCE
                * std::max({std::abs(current.total),
                    std::abs(trial_evaluation.total), 1.0});
            result.solve.monotonic = result.solve.monotonic
                && trial_evaluation.total <= current.total + allowance;
            const std::vector<int> next_active =
                pressure_active_signature(config, trial);
            if (next_active != active_signature) {
                ++result.solve.active_set_changes;
            }
            active_signature = next_active;
            y = trial;
            current = trial_evaluation;
            current_pairs = trial_pairs;
            adjacency = trial_adjacency;
            current_tape_valid = false;
            ++result.solve.accepted_trials;
            result.solve.minimum_accepted_ratio = std::min(
                result.solve.minimum_accepted_ratio, ratio);
            result.solve.maximum_accepted_ratio = std::max(
                result.solve.maximum_accepted_ratio, ratio);
        } else {
            ++result.solve.rejected_trials;
        }
        if (!valid_model || ratio < 0.25) {
            radius *= 0.25;
        } else if (ratio > 0.75 && step.boundary) {
            radius = std::min(2.0 * radius, maximum_radius);
        }
        result.solve.minimum_radius = std::min(
            result.solve.minimum_radius, radius);
        result.solve.maximum_radius = std::max(
            result.solve.maximum_radius, radius);
        if (capture_trace) {
            trace.trial_active = active_count(trial_active_signature);
            trace.active_additions = trace_active_delta.first;
            trace.active_removals = trace_active_delta.second;
            trace.trial_pairs = trial_pairs.size();
            trace.pair_additions = trace_pair_delta.additions;
            trace.pair_removals = trace_pair_delta.removals;
            trace.radius_after = radius;
            trace.actual_reduction = actual_reduction;
            trace.ratio = ratio;
            trace.valid_model = valid_model;
            trace.accepted = accepted;
            result.trace.push_back(trace);
        }
        if (radius < minimum_radius) {
            result.solve.failure = "MINIMUM_TRUST_RADIUS";
            break;
        }
    }
    if (trust_converged(
            config, current, true, result.solve.convergence_stop,
            scaled_displacement_limit)
        && result.solve.failure.empty()) {
        result.solve.succeeded = true;
    }
    if (!result.solve.succeeded && result.solve.failure.empty()) {
        result.solve.failure = "OUTER_TRIAL_LIMIT";
    }
    if (!std::isfinite(result.solve.minimum_accepted_ratio)) {
        result.solve.minimum_accepted_ratio = 0.0;
        result.solve.maximum_accepted_ratio = 0.0;
    }
    result.solve.succeeded = result.solve.succeeded
        && result.solve.monotonic && current.finite;
    result.solve.final = current;
    result.solve.final_scaled_displacement_residual =
        scaled_displacement_residual(config, current);
    result.solve.position = y;
    result.final_pairs = current_pairs.size();
    if (capture_timing) {
        result.total_nanoseconds = static_cast<std::uint64_t>(
            std::chrono::duration_cast<std::chrono::nanoseconds>(
                std::chrono::steady_clock::now() - solve_begin).count());
    }
    return result;
}

enum class RejectionClass {
    ArithmeticFloor,
    ActiveSetModelMismatch,
    SupportTopologyModelMismatch,
    SmoothModelConditioning,
};

const char* rejection_class_name(RejectionClass value) {
    switch (value) {
    case RejectionClass::ArithmeticFloor:
        return "ARITHMETIC_FLOOR";
    case RejectionClass::ActiveSetModelMismatch:
        return "ACTIVE_SET_MODEL_MISMATCH";
    case RejectionClass::SupportTopologyModelMismatch:
        return "SUPPORT_TOPOLOGY_MODEL_MISMATCH";
    case RejectionClass::SmoothModelConditioning:
        return "SMOOTH_MODEL_CONDITIONING";
    }
    return "UNREACHABLE";
}

RejectionClass classify_rejection(
    const NeighborhoodTrustResult::TrialTrace& trace) {
    const double arithmetic_floor = 1024.0
        * std::numeric_limits<double>::epsilon()
        * std::max(std::abs(trace.current_objective), 1.0);
    if (std::abs(trace.predicted_reduction) <= arithmetic_floor
        && std::abs(trace.actual_reduction) <= arithmetic_floor
        && trace.current_scaled_residual <= 1.0e-7) {
        return RejectionClass::ArithmeticFloor;
    }
    if (trace.active_additions != 0 || trace.active_removals != 0) {
        return RejectionClass::ActiveSetModelMismatch;
    }
    if (trace.pair_additions != 0 || trace.pair_removals != 0) {
        return RejectionClass::SupportTopologyModelMismatch;
    }
    return RejectionClass::SmoothModelConditioning;
}

void append_nullable_double(std::ostringstream& output, double value) {
    if (std::isfinite(value)) {
        output << value;
    } else {
        output << "null";
    }
}

void append_trial_trace(
    std::ostringstream& output,
    const NeighborhoodTrustResult::TrialTrace& value) {
    output << "{\"trial\":" << value.trial
           << ",\"accepted\":" << (value.accepted ? "true" : "false")
           << ",\"valid_model\":"
           << (value.valid_model ? "true" : "false")
           << ",\"current_objective\":" << value.current_objective
           << ",\"current_scaled_residual\":"
           << value.current_scaled_residual
           << ",\"radius_before\":" << value.radius_before
           << ",\"radius_after\":" << value.radius_after
           << ",\"inner_reason\":\"" << value.inner_reason << '"'
           << ",\"inner_iterations\":" << value.inner_iterations
           << ",\"inner_hvp_calls\":" << value.inner_hvp_calls
           << ",\"step_norm\":" << value.step_norm
           << ",\"predicted_reduction\":";
    append_nullable_double(output, value.predicted_reduction);
    output << ",\"actual_reduction\":";
    append_nullable_double(output, value.actual_reduction);
    output << ",\"ratio\":";
    append_nullable_double(output, value.ratio);
    output << ",\"current_pairs\":" << value.current_pairs
           << ",\"trial_pairs\":" << value.trial_pairs
           << ",\"pair_additions\":" << value.pair_additions
           << ",\"pair_removals\":" << value.pair_removals
           << ",\"current_active\":" << value.current_active
           << ",\"trial_active\":" << value.trial_active
           << ",\"active_additions\":" << value.active_additions
           << ",\"active_removals\":" << value.active_removals;
    if (!value.accepted) {
        output << ",\"classification\":\""
               << rejection_class_name(classify_rejection(value)) << '"';
    }
    output << '}';
}

struct NeighborhoodScalingCase {
    std::string name;
    std::size_t particles = 0;
    bool correspondence = false;
    bool passed = false;
    NeighborhoodTrustResult neighborhood;
};

bool exact_trust_result(
    const TrustSolveResult& lhs, const TrustSolveResult& rhs) {
    return lhs.succeeded == rhs.succeeded
        && lhs.monotonic == rhs.monotonic
        && lhs.outer_trials == rhs.outer_trials
        && lhs.accepted_trials == rhs.accepted_trials
        && lhs.rejected_trials == rhs.rejected_trials
        && lhs.objective_evaluations == rhs.objective_evaluations
        && lhs.hvp_calls == rhs.hvp_calls
        && lhs.negative_curvature_stops == rhs.negative_curvature_stops
        && lhs.boundary_stops == rhs.boundary_stops
        && lhs.active_set_changes == rhs.active_set_changes
        && lhs.numerical_floor_stops == rhs.numerical_floor_stops
        && lhs.minimum_radius == rhs.minimum_radius
        && lhs.maximum_radius == rhs.maximum_radius
        && lhs.final_scaled_displacement_residual
            == rhs.final_scaled_displacement_residual
        && lhs.numerical_energy_floor == rhs.numerical_energy_floor
        && lhs.numerical_predicted_reduction
            == rhs.numerical_predicted_reduction
        && lhs.numerical_scaled_step == rhs.numerical_scaled_step
        && lhs.convergence_stop == rhs.convergence_stop
        && lhs.failure == rhs.failure
        && exact_evaluation(lhs.final, rhs.final)
        && exact_vectors(lhs.position, rhs.position);
}

NeighborhoodScalingCase make_neighborhood_correspondence_case(int side) {
    const NeighborhoodFixture fixture = make_neighborhood_lattice_fixture(side);
    NeighborhoodScalingCase result;
    result.name = fixture.name + "_correspondence";
    result.particles = fixture.x.size();
    const TrustSolveResult all_pair = solve_trust_region(
        fixture.config, fixture.x, fixture.velocity, true);
    result.neighborhood = solve_neighborhood_trust_region(
        fixture.config, fixture.x, fixture.velocity);
    result.correspondence = exact_trust_result(
        all_pair, result.neighborhood.solve);
    result.passed = result.correspondence;
    return result;
}

NeighborhoodFixture make_neighborhood_scale_fixture(int side) {
    constexpr double pitch = 0.05;
    NeighborhoodFixture fixture;
    fixture.name = "scale_" + std::to_string(side) + "x"
        + std::to_string(side) + "x" + std::to_string(side);
    fixture.config.kappa = 200.0;
    fixture.config.lambda = 20.0;
    fixture.config.mu = 10.0;
    fixture.config.gamma = 100.0;
    const double center = 0.5 * static_cast<double>(side - 1);
    for (int iz = 0; iz < side; ++iz) {
        for (int iy = 0; iy < side; ++iy) {
            for (int ix = 0; ix < side; ++ix) {
                const Vec3 position = {
                    (static_cast<double>(ix) - center) * pitch,
                    (static_cast<double>(iy) - center) * pitch,
                    (static_cast<double>(iz) - center) * pitch,
                };
                fixture.x.push_back(position);
                fixture.velocity.push_back({
                    -0.35 * position.x + 0.08 * position.y,
                    -0.25 * position.y - 0.06 * position.z,
                    -0.30 * position.z + 0.05 * position.x,
                });
            }
        }
    }
    const std::vector<Vec3> y_star =
        predict(fixture.config, fixture.x, fixture.velocity);
    const double support = std::max(
        fixture.config.horizon, 3.0 * fixture.config.spacing);
    const std::vector<ParticlePair> pairs = build_cell_pairs(y_star, support);
    const std::vector<double> density =
        densities_with_pairs(fixture.config, y_star, pairs);
    fixture.config.rest_density =
        *std::max_element(density.begin(), density.end()) / 1.05;
    return fixture;
}

NeighborhoodScalingCase make_neighborhood_scale_case(int side) {
    const NeighborhoodFixture fixture = make_neighborhood_scale_fixture(side);
    NeighborhoodScalingCase result;
    result.name = fixture.name;
    result.particles = fixture.x.size();
    result.neighborhood = solve_neighborhood_trust_region(
        fixture.config, fixture.x, fixture.velocity);
    const TrustSolveResult& solve = result.neighborhood.solve;
    result.passed = solve.succeeded && solve.failure.empty()
        && solve.monotonic && solve.final.finite
        && solve.final.internal_momentum_residual <= CONSERVATION_LIMIT
        && solve.final_scaled_displacement_residual <= 1.0e-8
        && solve.outer_trials <= 32 && solve.rejected_trials <= 8
        && solve.hvp_calls <= 128
        && result.neighborhood.maximum_neighbors <= 160
        && result.neighborhood.maximum_pairs <= 80 * result.particles;
    return result;
}

void append_neighborhood_scaling_case(
    std::ostringstream& output, const NeighborhoodScalingCase& value) {
    const TrustSolveResult& solve = value.neighborhood.solve;
    output << "{\"name\":\"" << value.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL")
           << "\",\"particles\":" << value.particles
           << ",\"correspondence\":"
           << (value.correspondence ? "true" : "false")
           << ",\"failure\":\"" << solve.failure << '"'
           << ",\"initial_objective\":" << solve.initial.total
           << ",\"final_objective\":" << solve.final.total
           << ",\"final_gradient_norm\":" << solve.final.gradient_norm
           << ",\"scaled_displacement_residual\":"
           << solve.final_scaled_displacement_residual
           << ",\"convergence_stop\":\"" << solve.convergence_stop << '"'
           << ",\"outer_trials\":" << solve.outer_trials
           << ",\"accepted_trials\":" << solve.accepted_trials
           << ",\"rejected_trials\":" << solve.rejected_trials
           << ",\"objective_evaluations\":" << solve.objective_evaluations
           << ",\"hvp_calls\":" << solve.hvp_calls
           << ",\"pair_builds\":" << value.neighborhood.pair_builds
           << ",\"initial_pairs\":" << value.neighborhood.initial_pairs
           << ",\"final_pairs\":" << value.neighborhood.final_pairs
           << ",\"maximum_pairs\":" << value.neighborhood.maximum_pairs
           << ",\"maximum_neighbors\":"
           << value.neighborhood.maximum_neighbors
           << ",\"active_set_changes\":" << solve.active_set_changes
           << ",\"internal_momentum_residual\":"
           << solve.final.internal_momentum_residual << '}';
}

struct NumericalFloorCase {
    std::string name;
    std::size_t particles = 0;
    bool correspondence = false;
    bool no_floor_eligible_accept = true;
    bool passed = false;
    NeighborhoodTrustResult neighborhood;
};

bool valid_numerical_floor_stop(const TrustSolveResult& solve) {
    if (solve.convergence_stop != "NUMERICAL_ENERGY_FLOOR") {
        return solve.final_scaled_displacement_residual <= 1.0e-8
            && solve.numerical_floor_stops == 0;
    }
    return solve.numerical_floor_stops == 1
        && solve.numerical_predicted_reduction > 0.0
        && solve.numerical_predicted_reduction
            <= solve.numerical_energy_floor
        && solve.final_scaled_displacement_residual <= 1.0e-7
        && solve.numerical_scaled_step <= 1.0e-7;
}

bool no_floor_eligible_accepted_trial(
    const NeighborhoodTrustResult& result) {
    for (const NeighborhoodTrustResult::TrialTrace& trace : result.trace) {
        if (trace.accepted
            && trace.predicted_reduction > 0.0
            && trace.predicted_reduction <= trace.energy_floor
            && trace.current_scaled_residual <= 1.0e-7
            && trace.step_norm / 0.05 <= 1.0e-7) {
            return false;
        }
    }
    return true;
}

NumericalFloorCase make_numerical_floor_correspondence_case(int side) {
    const NeighborhoodFixture fixture = make_neighborhood_lattice_fixture(side);
    NumericalFloorCase result;
    result.name = fixture.name + "_correspondence";
    result.particles = fixture.x.size();
    const TrustSolveResult all_pair = solve_trust_region(
        fixture.config, fixture.x, fixture.velocity, true, true);
    result.neighborhood = solve_neighborhood_trust_region(
        fixture.config, fixture.x, fixture.velocity, false, true);
    result.correspondence = exact_trust_result(
        all_pair, result.neighborhood.solve);
    result.passed = result.correspondence
        && valid_numerical_floor_stop(result.neighborhood.solve);
    return result;
}

NumericalFloorCase make_numerical_floor_scale_case(int side) {
    const NeighborhoodFixture fixture = make_neighborhood_scale_fixture(side);
    NumericalFloorCase result;
    result.name = fixture.name;
    result.particles = fixture.x.size();
    result.neighborhood = solve_neighborhood_trust_region(
        fixture.config, fixture.x, fixture.velocity, true, true);
    const TrustSolveResult& solve = result.neighborhood.solve;
    result.no_floor_eligible_accept =
        no_floor_eligible_accepted_trial(result.neighborhood);
    result.passed = solve.succeeded && solve.failure.empty()
        && solve.monotonic && solve.final.finite
        && solve.final.internal_momentum_residual <= CONSERVATION_LIMIT
        && valid_numerical_floor_stop(solve)
        && result.no_floor_eligible_accept
        && solve.outer_trials <= 32 && solve.rejected_trials <= 8
        && solve.hvp_calls <= 128
        && result.neighborhood.maximum_neighbors <= 160
        && result.neighborhood.maximum_pairs <= 80 * result.particles;
    return result;
}

void append_numerical_floor_case(
    std::ostringstream& output, const NumericalFloorCase& value) {
    const TrustSolveResult& solve = value.neighborhood.solve;
    output << "{\"name\":\"" << value.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL")
           << "\",\"particles\":" << value.particles
           << ",\"correspondence\":"
           << (value.correspondence ? "true" : "false")
           << ",\"convergence_stop\":\"" << solve.convergence_stop << '"'
           << ",\"outer_trials\":" << solve.outer_trials
           << ",\"accepted_trials\":" << solve.accepted_trials
           << ",\"rejected_trials\":" << solve.rejected_trials
           << ",\"objective_evaluations\":" << solve.objective_evaluations
           << ",\"hvp_calls\":" << solve.hvp_calls
           << ",\"pair_builds\":" << value.neighborhood.pair_builds
           << ",\"maximum_pairs\":" << value.neighborhood.maximum_pairs
           << ",\"maximum_neighbors\":"
           << value.neighborhood.maximum_neighbors
           << ",\"scaled_displacement_residual\":"
           << solve.final_scaled_displacement_residual
           << ",\"numerical_floor_stops\":"
           << solve.numerical_floor_stops
           << ",\"numerical_energy_floor\":"
           << solve.numerical_energy_floor
           << ",\"numerical_predicted_reduction\":"
           << solve.numerical_predicted_reduction
           << ",\"numerical_scaled_step\":"
           << solve.numerical_scaled_step
           << ",\"no_floor_eligible_accept\":"
           << (value.no_floor_eligible_accept ? "true" : "false")
           << ",\"internal_momentum_residual\":"
           << solve.final.internal_momentum_residual << '}';
}

struct SerialTimingSample {
    std::uint64_t total = 0;
    std::uint64_t pair_and_adjacency = 0;
    std::uint64_t objective_gradient = 0;
    std::uint64_t hvp = 0;
    std::uint64_t hessian_tape_build = 0;
    std::uint64_t hvp_and_tape = 0;
    std::uint64_t control_and_vector = 0;
};

struct SerialBenchmarkCase {
    std::string name;
    std::size_t particles = 0;
    bool passed = false;
    bool exact_repeat = true;
    bool nsr2c2_overlap = true;
    NeighborhoodTrustResult reference;
    std::array<SerialTimingSample, 7> samples;
    std::string state_sha256;
};

bool exact_neighborhood_trust_result(
    const NeighborhoodTrustResult& lhs,
    const NeighborhoodTrustResult& rhs) {
    return exact_trust_result(lhs.solve, rhs.solve)
        && lhs.pair_builds == rhs.pair_builds
        && lhs.initial_pairs == rhs.initial_pairs
        && lhs.final_pairs == rhs.final_pairs
        && lhs.maximum_pairs == rhs.maximum_pairs
        && lhs.maximum_neighbors == rhs.maximum_neighbors;
}

std::string hash_neighborhood_trust_state(
    const NeighborhoodTrustResult& value) {
    std::ostringstream material;
    material << std::setprecision(17)
             << value.solve.convergence_stop << '|'
             << value.solve.final.total << '|'
             << value.solve.final.gradient_norm << '|'
             << value.solve.outer_trials << '|'
             << value.solve.accepted_trials << '|'
             << value.solve.rejected_trials << '|'
             << value.solve.objective_evaluations << '|'
             << value.solve.hvp_calls << '|'
             << value.pair_builds << '|'
             << value.initial_pairs << '|'
             << value.final_pairs << '|'
             << value.maximum_pairs << '|'
             << value.maximum_neighbors << '|';
    for (Vec3 position : value.solve.position) {
        material << position.x << ',' << position.y << ',' << position.z << '|';
    }
    return sha256_hex(material.str());
}

std::uint64_t timing_percentile(
    const std::array<SerialTimingSample, 7>& samples,
    std::uint64_t SerialTimingSample::*member,
    int percentile) {
    std::array<std::uint64_t, 7> values;
    for (std::size_t i = 0; i < samples.size(); ++i) {
        values[i] = samples[i].*member;
    }
    std::sort(values.begin(), values.end());
    const std::size_t rank = static_cast<std::size_t>(
        (percentile * static_cast<int>(values.size()) + 99) / 100 - 1);
    return values[rank];
}

bool matches_nsr2c2_overlap(
    int side, const NeighborhoodTrustResult& value) {
    const TrustSolveResult& solve = value.solve;
    if (side == 8) {
        return solve.convergence_stop == "NUMERICAL_ENERGY_FLOOR"
            && solve.outer_trials == 12 && solve.accepted_trials == 11
            && solve.rejected_trials == 0
            && solve.objective_evaluations == 12 && solve.hvp_calls == 46
            && value.pair_builds == 13 && value.maximum_pairs == 19492
            && value.maximum_neighbors == 122;
    }
    if (side == 10) {
        return solve.convergence_stop == "SCALED_DISPLACEMENT"
            && solve.outer_trials == 13 && solve.accepted_trials == 13
            && solve.rejected_trials == 0
            && solve.objective_evaluations == 14 && solve.hvp_calls == 45
            && value.pair_builds == 15 && value.maximum_pairs == 42144
            && value.maximum_neighbors == 122;
    }
    return true;
}

SerialBenchmarkCase make_serial_benchmark_case(int side) {
    const NeighborhoodFixture fixture = make_neighborhood_scale_fixture(side);
    SerialBenchmarkCase result;
    result.name = fixture.name;
    result.particles = fixture.x.size();
    result.reference = solve_neighborhood_trust_region(
        fixture.config, fixture.x, fixture.velocity, false, true, false);
    static_cast<void>(solve_neighborhood_trust_region(
        fixture.config, fixture.x, fixture.velocity, false, true, true));
    bool timing_valid = true;
    for (std::size_t run = 0; run < result.samples.size(); ++run) {
        const NeighborhoodTrustResult measured =
            solve_neighborhood_trust_region(
                fixture.config, fixture.x, fixture.velocity,
                false, true, true);
        result.exact_repeat = result.exact_repeat
            && exact_neighborhood_trust_result(result.reference, measured);
        SerialTimingSample& timing = result.samples[run];
        timing.total = measured.total_nanoseconds;
        timing.pair_and_adjacency =
            measured.pair_and_adjacency_nanoseconds;
        timing.objective_gradient =
            measured.objective_gradient_nanoseconds;
        timing.hvp = measured.hvp_nanoseconds;
        const std::uint64_t timed_sum = timing.pair_and_adjacency
            + timing.objective_gradient + timing.hvp;
        timing_valid = timing_valid && timing.total > 0
            && timing.pair_and_adjacency > 0
            && timing.objective_gradient > 0 && timing.hvp > 0
            && timed_sum <= timing.total;
        timing.control_and_vector = timing.total - timed_sum;
    }
    result.nsr2c2_overlap = matches_nsr2c2_overlap(side, result.reference);
    result.state_sha256 = hash_neighborhood_trust_state(result.reference);
    result.passed = result.reference.solve.succeeded
        && result.reference.solve.failure.empty()
        && result.reference.solve.final.finite
        && result.reference.solve.final.internal_momentum_residual
            <= CONSERVATION_LIMIT
        && result.reference.maximum_pairs <= 80 * result.particles
        && result.reference.maximum_neighbors <= 160
        && result.exact_repeat && result.nsr2c2_overlap && timing_valid;
    return result;
}

void append_timing_samples(
    std::ostringstream& output,
    const std::array<SerialTimingSample, 7>& samples,
    std::uint64_t SerialTimingSample::*member) {
    output << '[';
    for (std::size_t i = 0; i < samples.size(); ++i) {
        if (i != 0) {
            output << ',';
        }
        output << samples[i].*member;
    }
    output << ']';
}

void append_serial_benchmark_case(
    std::ostringstream& output, const SerialBenchmarkCase& value) {
    output << "{\"name\":\"" << value.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL")
           << "\",\"particles\":" << value.particles
           << ",\"exact_repeat\":"
           << (value.exact_repeat ? "true" : "false")
           << ",\"nsr2c2_overlap\":"
           << (value.nsr2c2_overlap ? "true" : "false")
           << ",\"state_sha256\":\"" << value.state_sha256 << '"'
           << ",\"operations\":{\"outer_trials\":"
           << value.reference.solve.outer_trials
           << ",\"objective_evaluations\":"
           << value.reference.solve.objective_evaluations
           << ",\"hvp_calls\":" << value.reference.solve.hvp_calls
           << ",\"pair_builds\":" << value.reference.pair_builds
           << ",\"maximum_pairs\":" << value.reference.maximum_pairs
           << ",\"maximum_neighbors\":"
           << value.reference.maximum_neighbors << "}"
           << ",\"median_ns\":{\"total\":"
           << timing_percentile(value.samples, &SerialTimingSample::total, 50)
           << ",\"pair_and_adjacency\":"
           << timing_percentile(value.samples,
                &SerialTimingSample::pair_and_adjacency, 50)
           << ",\"objective_gradient\":"
           << timing_percentile(value.samples,
                &SerialTimingSample::objective_gradient, 50)
           << ",\"hvp\":"
           << timing_percentile(value.samples, &SerialTimingSample::hvp, 50)
           << ",\"control_and_vector\":"
           << timing_percentile(value.samples,
                &SerialTimingSample::control_and_vector, 50) << "}"
           << ",\"p95_ns\":{\"total\":"
           << timing_percentile(value.samples, &SerialTimingSample::total, 95)
           << ",\"pair_and_adjacency\":"
           << timing_percentile(value.samples,
                &SerialTimingSample::pair_and_adjacency, 95)
           << ",\"objective_gradient\":"
           << timing_percentile(value.samples,
                &SerialTimingSample::objective_gradient, 95)
           << ",\"hvp\":"
           << timing_percentile(value.samples, &SerialTimingSample::hvp, 95)
           << ",\"control_and_vector\":"
           << timing_percentile(value.samples,
                &SerialTimingSample::control_and_vector, 95) << "}"
           << ",\"raw_ns\":{\"total\":";
    append_timing_samples(output, value.samples, &SerialTimingSample::total);
    output << ",\"pair_and_adjacency\":";
    append_timing_samples(output, value.samples,
        &SerialTimingSample::pair_and_adjacency);
    output << ",\"objective_gradient\":";
    append_timing_samples(output, value.samples,
        &SerialTimingSample::objective_gradient);
    output << ",\"hvp\":";
    append_timing_samples(output, value.samples, &SerialTimingSample::hvp);
    output << ",\"control_and_vector\":";
    append_timing_samples(output, value.samples,
        &SerialTimingSample::control_and_vector);
    output << "}}";
}

bool workspace_hvp_exact_controls() {
    const std::array<NeighborhoodFixture, 7> fixtures =
        neighborhood_fixtures();
    for (const NeighborhoodFixture& fixture : fixtures) {
        const std::vector<Vec3> y =
            predict(fixture.config, fixture.x, fixture.velocity);
        const double support = std::max(
            fixture.config.horizon, 3.0 * fixture.config.spacing);
        const std::vector<ParticlePair> current_pairs =
            build_cell_pairs(y, support);
        const std::vector<ParticlePair> reference_pairs =
            build_cell_pairs(fixture.x, fixture.config.horizon);
        const std::vector<std::vector<std::size_t>> adjacency =
            build_pair_adjacency(y.size(), current_pairs);
        const std::vector<Vec3> baseline = apply_hessian_with_adjacency(
            fixture.config, fixture.x, y, fixture.direction,
            current_pairs, reference_pairs, adjacency);
        NeighborhoodHvpWorkspace workspace;
        const std::vector<Vec3>& candidate = apply_hessian_with_workspace(
            fixture.config, fixture.x, y, fixture.direction,
            current_pairs, reference_pairs, adjacency, workspace);
        if (!exact_vectors(baseline, candidate)) {
            return false;
        }
    }
    return true;
}

bool hessian_tape_hvp_exact_controls() {
    const std::array<NeighborhoodFixture, 7> fixtures =
        neighborhood_fixtures();
    for (const NeighborhoodFixture& fixture : fixtures) {
        const std::vector<Vec3> y =
            predict(fixture.config, fixture.x, fixture.velocity);
        const double support = std::max(
            fixture.config.horizon, 3.0 * fixture.config.spacing);
        const std::vector<ParticlePair> current_pairs =
            build_cell_pairs(y, support);
        const std::vector<ParticlePair> reference_pairs =
            build_cell_pairs(fixture.x, fixture.config.horizon);
        const std::vector<std::vector<std::size_t>> adjacency =
            build_pair_adjacency(y.size(), current_pairs);
        NeighborhoodHessianTape tape;
        build_reference_hessian_tape(
            fixture.config, fixture.x, reference_pairs, tape);
        if (!build_current_hessian_tape(
                fixture.config, y, current_pairs, adjacency, tape)) {
            return false;
        }
        NeighborhoodHvpWorkspace baseline_workspace;
        NeighborhoodHvpWorkspace tape_workspace;
        const std::vector<Vec3>& baseline = apply_hessian_with_workspace(
            fixture.config, fixture.x, y, fixture.direction,
            current_pairs, reference_pairs, adjacency, baseline_workspace);
        const std::vector<Vec3>& candidate = apply_hessian_with_tape(
            fixture.config, fixture.direction, adjacency,
            tape, tape_workspace);
        if (!exact_vectors(baseline, candidate)) {
            return false;
        }
        const std::size_t memory_limit = hessian_tape_storage_limit(
            y.size(), std::max(current_pairs.size(), reference_pairs.size()));
        if (hessian_tape_storage_bytes(tape) > memory_limit) {
            return false;
        }
    }
    return true;
}

struct HvpTournamentCase {
    std::string name;
    std::size_t particles = 0;
    bool passed = false;
    bool exact_state = true;
    NeighborhoodTrustResult baseline_reference;
    NeighborhoodTrustResult candidate_reference;
    std::array<SerialTimingSample, 7> baseline_samples;
    std::array<SerialTimingSample, 7> candidate_samples;
    double hvp_speedup = 0.0;
    double total_speedup = 0.0;
};

SerialTimingSample timing_sample(const NeighborhoodTrustResult& value) {
    SerialTimingSample result;
    result.total = value.total_nanoseconds;
    result.pair_and_adjacency = value.pair_and_adjacency_nanoseconds;
    result.objective_gradient = value.objective_gradient_nanoseconds;
    result.hvp = value.hvp_nanoseconds;
    result.hessian_tape_build = value.hessian_tape_build_nanoseconds;
    result.hvp_and_tape = result.hvp + result.hessian_tape_build;
    const std::uint64_t timed_sum = result.pair_and_adjacency
        + result.objective_gradient + result.hvp
        + result.hessian_tape_build;
    result.control_and_vector = value.total_nanoseconds >= timed_sum
        ? value.total_nanoseconds - timed_sum : 0;
    return result;
}

bool valid_timing(const SerialTimingSample& value) {
    return value.total > 0 && value.pair_and_adjacency > 0
        && value.objective_gradient > 0 && value.hvp > 0
        && value.pair_and_adjacency + value.objective_gradient + value.hvp
            + value.hessian_tape_build
            <= value.total;
}

HvpTournamentCase make_hvp_tournament_case(int side) {
    const NeighborhoodFixture fixture = make_neighborhood_scale_fixture(side);
    HvpTournamentCase result;
    result.name = fixture.name;
    result.particles = fixture.x.size();
    result.baseline_reference = solve_neighborhood_trust_region(
        fixture.config, fixture.x, fixture.velocity,
        false, true, false, false);
    result.candidate_reference = solve_neighborhood_trust_region(
        fixture.config, fixture.x, fixture.velocity,
        false, true, false, true);
    result.exact_state = exact_neighborhood_trust_result(
        result.baseline_reference, result.candidate_reference);
    static_cast<void>(solve_neighborhood_trust_region(
        fixture.config, fixture.x, fixture.velocity,
        false, true, true, false));
    static_cast<void>(solve_neighborhood_trust_region(
        fixture.config, fixture.x, fixture.velocity,
        false, true, true, true));
    bool timings_valid = true;
    for (std::size_t run = 0; run < result.baseline_samples.size(); ++run) {
        NeighborhoodTrustResult baseline;
        NeighborhoodTrustResult candidate;
        if (run % 2 == 0) {
            baseline = solve_neighborhood_trust_region(
                fixture.config, fixture.x, fixture.velocity,
                false, true, true, false);
            candidate = solve_neighborhood_trust_region(
                fixture.config, fixture.x, fixture.velocity,
                false, true, true, true);
        } else {
            candidate = solve_neighborhood_trust_region(
                fixture.config, fixture.x, fixture.velocity,
                false, true, true, true);
            baseline = solve_neighborhood_trust_region(
                fixture.config, fixture.x, fixture.velocity,
                false, true, true, false);
        }
        result.exact_state = result.exact_state
            && exact_neighborhood_trust_result(
                result.baseline_reference, baseline)
            && exact_neighborhood_trust_result(
                result.candidate_reference, candidate)
            && exact_neighborhood_trust_result(baseline, candidate);
        result.baseline_samples[run] = timing_sample(baseline);
        result.candidate_samples[run] = timing_sample(candidate);
        timings_valid = timings_valid
            && valid_timing(result.baseline_samples[run])
            && valid_timing(result.candidate_samples[run]);
    }
    const double baseline_hvp = static_cast<double>(timing_percentile(
        result.baseline_samples, &SerialTimingSample::hvp, 50));
    const double candidate_hvp = static_cast<double>(timing_percentile(
        result.candidate_samples, &SerialTimingSample::hvp, 50));
    const double baseline_total = static_cast<double>(timing_percentile(
        result.baseline_samples, &SerialTimingSample::total, 50));
    const double candidate_total = static_cast<double>(timing_percentile(
        result.candidate_samples, &SerialTimingSample::total, 50));
    result.hvp_speedup = baseline_hvp / candidate_hvp;
    result.total_speedup = baseline_total / candidate_total;
    const bool total_gate = side >= 12
        ? result.total_speedup >= 1.10
        : result.total_speedup >= (1.0 / 1.02);
    result.passed = result.exact_state && timings_valid
        && result.hvp_speedup >= 1.20 && total_gate;
    return result;
}

void append_hvp_tournament_case(
    std::ostringstream& output, const HvpTournamentCase& value) {
    output << std::setprecision(17)
           << "{\"name\":\"" << value.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL")
           << "\",\"particles\":" << value.particles
           << ",\"exact_state\":"
           << (value.exact_state ? "true" : "false")
           << ",\"hvp_speedup\":" << value.hvp_speedup
           << ",\"total_speedup\":" << value.total_speedup
           << ",\"baseline_median_ns\":{\"total\":"
           << timing_percentile(value.baseline_samples,
                &SerialTimingSample::total, 50)
           << ",\"hvp\":" << timing_percentile(value.baseline_samples,
                &SerialTimingSample::hvp, 50) << "}"
           << ",\"candidate_median_ns\":{\"total\":"
           << timing_percentile(value.candidate_samples,
                &SerialTimingSample::total, 50)
           << ",\"hvp\":" << timing_percentile(value.candidate_samples,
                &SerialTimingSample::hvp, 50) << "}"
           << ",\"baseline_raw_total_ns\":";
    append_timing_samples(output, value.baseline_samples,
        &SerialTimingSample::total);
    output << ",\"candidate_raw_total_ns\":";
    append_timing_samples(output, value.candidate_samples,
        &SerialTimingSample::total);
    output << ",\"baseline_raw_hvp_ns\":";
    append_timing_samples(output, value.baseline_samples,
        &SerialTimingSample::hvp);
    output << ",\"candidate_raw_hvp_ns\":";
    append_timing_samples(output, value.candidate_samples,
        &SerialTimingSample::hvp);
    output << '}';
}

struct HessianTapeTournamentCase {
    std::string name;
    std::size_t particles = 0;
    bool passed = false;
    bool exact_state = true;
    bool capacity_valid = true;
    NeighborhoodTrustResult baseline_reference;
    NeighborhoodTrustResult candidate_reference;
    std::array<SerialTimingSample, 7> baseline_samples;
    std::array<SerialTimingSample, 7> candidate_samples;
    double combined_hvp_speedup = 0.0;
    double total_speedup = 0.0;
};

HessianTapeTournamentCase make_hessian_tape_tournament_case(int side) {
    const NeighborhoodFixture fixture = make_neighborhood_scale_fixture(side);
    HessianTapeTournamentCase result;
    result.name = fixture.name;
    result.particles = fixture.x.size();
    result.baseline_reference = solve_neighborhood_trust_region(
        fixture.config, fixture.x, fixture.velocity,
        false, true, false, true, false, false);
    result.candidate_reference = solve_neighborhood_trust_region(
        fixture.config, fixture.x, fixture.velocity,
        false, true, false, true, true, true);
    result.exact_state = exact_neighborhood_trust_result(
        result.baseline_reference, result.candidate_reference)
        && result.candidate_reference.hessian_tape_hvp_checks
            == result.candidate_reference.solve.outer_trials;
    result.capacity_valid =
        result.candidate_reference.maximum_hessian_tape_bytes
            <= hessian_tape_storage_limit(result.particles,
                result.candidate_reference.maximum_pairs)
        && result.candidate_reference.maximum_hessian_tape_bytes
            <= result.candidate_reference.hessian_tape_capacity_bytes;
    static_cast<void>(solve_neighborhood_trust_region(
        fixture.config, fixture.x, fixture.velocity,
        false, true, true, true, false, false));
    static_cast<void>(solve_neighborhood_trust_region(
        fixture.config, fixture.x, fixture.velocity,
        false, true, true, true, true, false));
    bool timings_valid = true;
    for (std::size_t run = 0; run < result.baseline_samples.size(); ++run) {
        NeighborhoodTrustResult baseline;
        NeighborhoodTrustResult candidate;
        if (run % 2 == 0) {
            baseline = solve_neighborhood_trust_region(
                fixture.config, fixture.x, fixture.velocity,
                false, true, true, true, false, false);
            candidate = solve_neighborhood_trust_region(
                fixture.config, fixture.x, fixture.velocity,
                false, true, true, true, true, false);
        } else {
            candidate = solve_neighborhood_trust_region(
                fixture.config, fixture.x, fixture.velocity,
                false, true, true, true, true, false);
            baseline = solve_neighborhood_trust_region(
                fixture.config, fixture.x, fixture.velocity,
                false, true, true, true, false, false);
        }
        result.exact_state = result.exact_state
            && exact_neighborhood_trust_result(
                result.baseline_reference, baseline)
            && exact_neighborhood_trust_result(
                result.candidate_reference, candidate)
            && exact_neighborhood_trust_result(baseline, candidate);
        result.capacity_valid = result.capacity_valid
            && candidate.maximum_hessian_tape_bytes
                <= hessian_tape_storage_limit(
                    result.particles, candidate.maximum_pairs)
            && candidate.maximum_hessian_tape_bytes
                <= candidate.hessian_tape_capacity_bytes;
        result.baseline_samples[run] = timing_sample(baseline);
        result.candidate_samples[run] = timing_sample(candidate);
        timings_valid = timings_valid
            && valid_timing(result.baseline_samples[run])
            && valid_timing(result.candidate_samples[run])
            && result.candidate_samples[run].hessian_tape_build > 0;
    }
    const double baseline_combined = static_cast<double>(timing_percentile(
        result.baseline_samples, &SerialTimingSample::hvp_and_tape, 50));
    const double candidate_combined = static_cast<double>(timing_percentile(
        result.candidate_samples, &SerialTimingSample::hvp_and_tape, 50));
    const double baseline_total = static_cast<double>(timing_percentile(
        result.baseline_samples, &SerialTimingSample::total, 50));
    const double candidate_total = static_cast<double>(timing_percentile(
        result.candidate_samples, &SerialTimingSample::total, 50));
    result.combined_hvp_speedup = baseline_combined / candidate_combined;
    result.total_speedup = baseline_total / candidate_total;
    const bool total_gate = side >= 12
        ? result.total_speedup >= 1.10
        : result.total_speedup >= (1.0 / 1.02);
    result.passed = result.exact_state && result.capacity_valid
        && timings_valid && result.combined_hvp_speedup >= 1.20
        && total_gate;
    return result;
}

void append_hessian_tape_tournament_case(
    std::ostringstream& output,
    const HessianTapeTournamentCase& value) {
    output << std::setprecision(17)
           << "{\"name\":\"" << value.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL")
           << "\",\"particles\":" << value.particles
           << ",\"exact_state\":"
           << (value.exact_state ? "true" : "false")
           << ",\"capacity_valid\":"
           << (value.capacity_valid ? "true" : "false")
           << ",\"combined_hvp_speedup\":"
           << value.combined_hvp_speedup
           << ",\"total_speedup\":" << value.total_speedup
           << ",\"maximum_hessian_tape_bytes\":"
           << value.candidate_reference.maximum_hessian_tape_bytes
           << ",\"hessian_tape_capacity_bytes\":"
           << value.candidate_reference.hessian_tape_capacity_bytes
           << ",\"maximum_active_pressure_centers\":"
           << value.candidate_reference.maximum_active_pressure_centers
           << ",\"maximum_directed_pressure_records\":"
           << value.candidate_reference.maximum_directed_pressure_records
           << ",\"reference_viscosity_records\":"
           << value.candidate_reference.reference_viscosity_records
           << ",\"maximum_surface_records\":"
           << value.candidate_reference.maximum_surface_records
           << ",\"baseline_median_ns\":{\"total\":"
           << timing_percentile(value.baseline_samples,
                &SerialTimingSample::total, 50)
           << ",\"hvp_and_tape\":"
           << timing_percentile(value.baseline_samples,
                &SerialTimingSample::hvp_and_tape, 50) << "}"
           << ",\"candidate_median_ns\":{\"total\":"
           << timing_percentile(value.candidate_samples,
                &SerialTimingSample::total, 50)
           << ",\"hvp\":"
           << timing_percentile(value.candidate_samples,
                &SerialTimingSample::hvp, 50)
           << ",\"hessian_tape_build\":"
           << timing_percentile(value.candidate_samples,
                &SerialTimingSample::hessian_tape_build, 50)
           << ",\"hvp_and_tape\":"
           << timing_percentile(value.candidate_samples,
                &SerialTimingSample::hvp_and_tape, 50) << "}"
           << ",\"baseline_raw_total_ns\":";
    append_timing_samples(output, value.baseline_samples,
        &SerialTimingSample::total);
    output << ",\"candidate_raw_total_ns\":";
    append_timing_samples(output, value.candidate_samples,
        &SerialTimingSample::total);
    output << ",\"baseline_raw_hvp_and_tape_ns\":";
    append_timing_samples(output, value.baseline_samples,
        &SerialTimingSample::hvp_and_tape);
    output << ",\"candidate_raw_hvp_and_tape_ns\":";
    append_timing_samples(output, value.candidate_samples,
        &SerialTimingSample::hvp_and_tape);
    output << '}';
}

struct NormalizedSpectralControl {
    SpectralCase spectral;
    double gradient_error = 0.0;
    double momentum_residual = 0.0;
    bool passed = false;
};

struct NormalizedDensityControl {
    std::size_t particles = 0;
    std::size_t pairs = 0;
    double kernel_scale = 0.0;
    double reference_center_ratio = 0.0;
    double compressed_center_ratio = 0.0;
    int compressed_active_particles = 0;
    double momentum_residual = 0.0;
    bool passed = false;
};

struct NormalizedTrustControl {
    TrustSolveResult pressure;
    TrustSolveResult combined;
    bool passed = false;
};

struct NormalizedTapeControl {
    NeighborhoodTrustResult a1;
    NeighborhoodTrustResult a2;
    bool exact = false;
    bool capacity_valid = false;
    bool passed = false;
};

NormalizedDensityControl normalized_density_control() {
    constexpr int side = 7;
    Config config;
    config.kappa = 1226.25;
    config.kernel_scale = reference_lattice_kernel_scale(
        config.spacing, config.horizon);
    std::vector<Vec3> reference;
    const double center = 0.5 * static_cast<double>(side - 1);
    for (int z = 0; z < side; ++z) {
        for (int y = 0; y < side; ++y) {
            for (int x = 0; x < side; ++x) {
                reference.push_back({
                    (static_cast<double>(x) - center) * config.spacing,
                    (static_cast<double>(y) - center) * config.spacing,
                    (static_cast<double>(z) - center) * config.spacing,
                });
            }
        }
    }
    std::vector<Vec3> compressed = reference;
    for (Vec3& position : compressed) {
        position = 0.99 * position;
    }
    const std::size_t center_index =
        static_cast<std::size_t>((side / 2) * side * side
            + (side / 2) * side + side / 2);
    const std::vector<double> reference_density =
        densities(config, reference);
    const std::vector<double> compressed_density =
        densities(config, compressed);
    const Evaluation compressed_evaluation =
        evaluate(config, reference, compressed, compressed);
    NormalizedDensityControl result;
    result.particles = reference.size();
    result.pairs = all_pairs_inside(compressed, config.horizon).size();
    result.kernel_scale = config.kernel_scale;
    result.reference_center_ratio =
        reference_density[center_index] / config.rest_density;
    result.compressed_center_ratio =
        compressed_density[center_index] / config.rest_density;
    result.momentum_residual =
        compressed_evaluation.internal_momentum_residual;
    for (double density : compressed_density) {
        if (density > config.rest_density) {
            ++result.compressed_active_particles;
        }
    }
    result.passed = result.particles <= 512
        && result.pairs <= 80 * result.particles
        && relative_error(result.reference_center_ratio, 1.0) <= 1.0e-12
        && result.compressed_center_ratio > 1.0
        && result.compressed_active_particles > 0
        && compressed_evaluation.finite
        && compressed_evaluation.pressure > 0.0
        && result.momentum_residual <= CONSERVATION_LIMIT;
    return result;
}

NormalizedTrustControl normalized_trust_control(
    const std::array<SpectralFixture, 6>& fixtures) {
    NormalizedTrustControl result;
    result.pressure = solve_trust_region(
        fixtures[0].config, fixtures[0].x, fixtures[0].velocity,
        true, true);
    result.combined = solve_trust_region(
        fixtures[5].config, fixtures[5].x, fixtures[5].velocity,
        true, true);
    const auto valid = [](const TrustSolveResult& solve) {
        return solve.succeeded && solve.monotonic && solve.failure.empty()
            && solve.accepted_trials > 0 && solve.rejected_trials <= 8
            && solve.minimum_accepted_ratio >= 0.1
            && solve.final.finite
            && solve.final.internal_momentum_residual <= CONSERVATION_LIMIT;
    };
    result.passed = valid(result.pressure) && valid(result.combined);
    return result;
}

NormalizedTapeControl normalized_tape_control() {
    NeighborhoodFixture fixture = make_neighborhood_scale_fixture(8);
    fixture.config.kernel_scale = reference_lattice_kernel_scale(
        fixture.config.spacing, fixture.config.horizon);
    fixture.config.rest_density *= fixture.config.kernel_scale;
    NormalizedTapeControl result;
    result.a1 = solve_neighborhood_trust_region(
        fixture.config, fixture.x, fixture.velocity,
        false, true, false, true, false, false);
    result.a2 = solve_neighborhood_trust_region(
        fixture.config, fixture.x, fixture.velocity,
        false, true, false, true, true, true);
    result.exact = exact_neighborhood_trust_result(result.a1, result.a2)
        && result.a2.hessian_tape_hvp_checks
            == result.a2.solve.outer_trials;
    result.capacity_valid = result.a2.maximum_pairs
            <= 80 * fixture.x.size()
        && result.a2.maximum_hessian_tape_bytes
            <= hessian_tape_storage_limit(
                fixture.x.size(), result.a2.maximum_pairs)
        && result.a2.maximum_hessian_tape_bytes
            <= result.a2.hessian_tape_capacity_bytes;
    result.passed = result.exact && result.capacity_valid
        && result.a1.solve.succeeded && result.a2.solve.succeeded;
    return result;
}

void append_normalized_spectral_control(
    std::ostringstream& output, const NormalizedSpectralControl& value) {
    output << "{\"name\":\"" << value.spectral.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL")
           << "\",\"gradient_error\":" << value.gradient_error
           << ",\"momentum_residual\":" << value.momentum_residual
           << ",\"hvp_fd_error\":" << value.spectral.hvp_fd_error
           << ",\"symmetry_error\":" << value.spectral.symmetry_error
           << ",\"dense_product_error\":"
           << value.spectral.dense_product_error
           << ",\"active_pressure_count\":"
           << value.spectral.active_pressure_count
           << ",\"active_margin\":" << value.spectral.active_margin
           << ",\"minimum_eigenvalue\":"
           << value.spectral.minimum_eigenvalue
           << ",\"maximum_eigenvalue\":"
           << value.spectral.maximum_eigenvalue << '}';
}

void append_normalized_trust_result(
    std::ostringstream& output, const TrustSolveResult& value) {
    output << "{\"succeeded\":" << (value.succeeded ? "true" : "false")
           << ",\"failure\":\"" << value.failure << '"'
           << ",\"stop\":\"" << value.convergence_stop << '"'
           << ",\"outer_trials\":" << value.outer_trials
           << ",\"accepted_trials\":" << value.accepted_trials
           << ",\"rejected_trials\":" << value.rejected_trials
           << ",\"objective_evaluations\":" << value.objective_evaluations
           << ",\"hvp_calls\":" << value.hvp_calls
           << ",\"minimum_accepted_ratio\":"
           << value.minimum_accepted_ratio
           << ",\"maximum_accepted_ratio\":"
           << value.maximum_accepted_ratio
           << ",\"final_objective\":" << value.final.total
           << ",\"final_gradient_norm\":" << value.final.gradient_norm
           << ",\"momentum_residual\":"
           << value.final.internal_momentum_residual << '}';
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
    SolveResult candidate;
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
    result.candidate = solve(
        config, x, velocity, Preconditioner::BlockThenInertialWarm);
    result.direction_preserved =
        norm(result.candidate.position[0] - result.candidate.position[1])
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
    result.candidate = solve(
        config, x, velocity, Preconditioner::BlockThenInertialWarm);
    result.direction_preserved =
        norm(result.candidate.position[0] - result.candidate.position[1])
        > initial_distance + OBSERVABLE_FLOOR;
    return result;
}

ConditioningCase combined_conditioning_case() {
    const CombinedFixture fixture = combined_fixture();
    ConditioningCase result;
    result.name = "combined_tetrahedron";
    result.baseline = solve(
        fixture.config, fixture.x, fixture.velocity, Preconditioner::Inertial);
    result.candidate = solve(fixture.config, fixture.x, fixture.velocity,
        Preconditioner::BlockThenInertialWarm);
    result.direction_preserved =
        result.candidate.final.total < result.candidate.initial.total;
    return result;
}

bool conditioning_quality_passed(const ConditioningCase& value) {
    const double objective_allowance = 1.0e-10
        * std::max({std::abs(value.baseline.final.total),
            std::abs(value.candidate.final.total), 1.0});
    const double gradient_limit =
        std::max(2.0 * value.baseline.final.gradient_norm, 1.0e-8);
    return value.baseline.succeeded && value.candidate.succeeded
        && value.baseline.monotonic && value.candidate.monotonic
        && value.candidate.final.total <= value.baseline.final.total + objective_allowance
        && value.candidate.final.gradient_norm <= gradient_limit
        && value.candidate.final.internal_momentum_residual <= CONSERVATION_LIMIT
        && value.direction_preserved;
}

void append_conditioning_case(
    std::ostringstream& output,
    const ConditioningCase& value) {
    const double alpha_ratio = value.candidate.minimum_alpha
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
           << ",\"candidate\":{\"final_objective\":"
           << value.candidate.final.total
           << ",\"final_gradient_norm\":" << value.candidate.final.gradient_norm
           << ",\"iterations\":" << value.candidate.iterations
           << ",\"backtracks\":" << value.candidate.backtracks
           << ",\"minimum_alpha\":" << value.candidate.minimum_alpha << '}'
           << ",\"minimum_alpha_improvement\":" << alpha_ratio << '}';
}

struct SissmCase {
    std::string name;
    SolveResult baseline;
    SolveResult candidate;
    bool direction_preserved = false;
    bool passed = false;
};

SissmCase make_sissm_compression_case(
    SissmAcceleration acceleration = SissmAcceleration::None) {
    Config config;
    config.kappa = 500.0;
    const std::vector<Vec3> x = {{-0.025, 0.0, 0.0}, {0.025, 0.0, 0.0}};
    const std::vector<Vec3> velocity(2);
    config.rest_density = pair_density(norm(x[0] - x[1]), config) / 1.1;
    SissmCase result;
    result.name = "compressed_pair";
    result.baseline = solve(config, x, velocity);
    result.candidate = solve_sissm(config, x, velocity, acceleration);
    result.direction_preserved =
        norm(result.candidate.position[0] - result.candidate.position[1])
        > norm(x[0] - x[1]) + OBSERVABLE_FLOOR;
    return result;
}

SissmCase make_sissm_viscosity_case(
    bool shear,
    SissmAcceleration acceleration = SissmAcceleration::None) {
    Config config;
    config.lambda = shear ? 0.0 : 100.0;
    config.mu = shear ? 100.0 : 0.0;
    const std::vector<Vec3> x = {{-0.04, 0.0, 0.0}, {0.04, 0.0, 0.0}};
    const std::vector<Vec3> velocity = shear
        ? std::vector<Vec3>{{0.0, 1.0, 0.0}, {0.0, -1.0, 0.0}}
        : std::vector<Vec3>{{1.0, 0.0, 0.0}, {-1.0, 0.0, 0.0}};
    const Vec3 normal = (x[0] - x[1]) / norm(x[0] - x[1]);
    const Vec3 initial_relative = velocity[0] - velocity[1];
    SissmCase result;
    result.name = shear ? "shear_viscosity_pair" : "normal_viscosity_pair";
    result.baseline = solve(config, x, velocity);
    result.candidate = solve_sissm(config, x, velocity, acceleration);
    const Vec3 final_relative =
        result.candidate.velocity[0] - result.candidate.velocity[1];
    const double initial_component = shear
        ? norm(project_tangent(initial_relative, normal))
        : norm(project_normal(initial_relative, normal));
    const double final_component = shear
        ? norm(project_tangent(final_relative, normal))
        : norm(project_normal(final_relative, normal));
    result.direction_preserved = final_component
        < initial_component - OBSERVABLE_FLOOR;
    return result;
}

SissmCase make_sissm_surface_case(
    bool attractive,
    SissmAcceleration acceleration = SissmAcceleration::None) {
    Config config;
    config.gamma = 1000.0;
    const double initial_distance = (attractive ? 1.7 : 0.8) * config.spacing;
    const std::vector<Vec3> x = {
        {-0.5 * initial_distance, 0.0, 0.0},
        {0.5 * initial_distance, 0.0, 0.0},
    };
    const std::vector<Vec3> velocity(2);
    SissmCase result;
    result.name = attractive
        ? "surface_attractive_pair" : "surface_repulsive_pair";
    result.baseline = solve(config, x, velocity);
    result.candidate = solve_sissm(config, x, velocity, acceleration);
    const double final_distance =
        norm(result.candidate.position[0] - result.candidate.position[1]);
    result.direction_preserved = attractive
        ? final_distance < initial_distance - OBSERVABLE_FLOOR
        : final_distance > initial_distance + OBSERVABLE_FLOOR;
    return result;
}

SissmCase make_sissm_combined_case(
    SissmAcceleration acceleration = SissmAcceleration::None) {
    const CombinedFixture fixture = combined_fixture();
    SissmCase result;
    result.name = "combined_tetrahedron";
    result.baseline = solve(fixture.config, fixture.x, fixture.velocity);
    result.candidate = solve_sissm(
        fixture.config, fixture.x, fixture.velocity, acceleration);
    result.direction_preserved =
        result.candidate.final.total < result.candidate.initial.total;
    return result;
}

SissmCase make_sissm_mask_case(
    const std::string& mask,
    SissmAcceleration acceleration = SissmAcceleration::None) {
    CombinedFixture fixture = combined_fixture();
    if (mask.find('P') == std::string::npos) {
        fixture.config.kappa = 0.0;
    }
    if (mask.find('V') == std::string::npos) {
        fixture.config.lambda = 0.0;
        fixture.config.mu = 0.0;
    }
    if (mask.find('S') == std::string::npos) {
        fixture.config.gamma = 0.0;
    }
    SissmCase result;
    result.name = mask;
    result.baseline = solve(fixture.config, fixture.x, fixture.velocity);
    result.candidate = solve_sissm(
        fixture.config, fixture.x, fixture.velocity, acceleration);
    result.direction_preserved =
        result.candidate.final.total < result.candidate.initial.total;
    return result;
}

bool sissm_quality_passed(const SissmCase& value) {
    const double objective_allowance = 1.0e-10
        * std::max({std::abs(value.baseline.final.total),
            std::abs(value.candidate.final.total), 1.0});
    const double gradient_limit =
        std::max(2.0 * value.baseline.final.gradient_norm, 1.0e-8);
    return value.baseline.succeeded && value.candidate.succeeded
        && value.candidate.failure.empty()
        && value.baseline.monotonic && value.candidate.monotonic
        && value.candidate.final.total <= value.baseline.final.total + objective_allowance
        && value.candidate.final.gradient_norm <= gradient_limit
        && value.candidate.final.internal_momentum_residual <= CONSERVATION_LIMIT
        && value.direction_preserved;
}

void append_sissm_case(std::ostringstream& output, const SissmCase& value) {
    output << "{\"name\":\"" << value.name << "\",\"status\":\""
           << (value.passed ? "PASS" : "FAIL")
           << "\",\"direction_preserved\":"
           << (value.direction_preserved ? "true" : "false")
           << ",\"failure\":\"" << value.candidate.failure << '"'
           << ",\"baseline\":{\"final_objective\":"
           << value.baseline.final.total
           << ",\"final_gradient_norm\":" << value.baseline.final.gradient_norm
           << ",\"iterations\":" << value.baseline.iterations
           << ",\"backtracks\":" << value.baseline.backtracks << '}'
           << ",\"candidate\":{\"final_objective\":"
           << value.candidate.final.total
           << ",\"final_gradient_norm\":" << value.candidate.final.gradient_norm
           << ",\"iterations\":" << value.candidate.iterations
           << ",\"backtracks\":" << value.candidate.backtracks
           << ",\"minimum_alpha\":" << value.candidate.minimum_alpha << "}}";
}

bool same_sissm_candidate(const SissmCase& lhs, const SissmCase& rhs) {
    if (lhs.candidate.succeeded != rhs.candidate.succeeded
        || lhs.candidate.monotonic != rhs.candidate.monotonic
        || lhs.candidate.iterations != rhs.candidate.iterations
        || lhs.candidate.backtracks != rhs.candidate.backtracks
        || lhs.candidate.minimum_alpha != rhs.candidate.minimum_alpha
        || lhs.candidate.final.total != rhs.candidate.final.total
        || lhs.candidate.final.gradient_norm != rhs.candidate.final.gradient_norm
        || lhs.candidate.failure != rhs.candidate.failure
        || lhs.candidate.position.size() != rhs.candidate.position.size()) {
        return false;
    }
    for (std::size_t i = 0; i < lhs.candidate.position.size(); ++i) {
        if (lhs.candidate.position[i].x != rhs.candidate.position[i].x
            || lhs.candidate.position[i].y != rhs.candidate.position[i].y
            || lhs.candidate.position[i].z != rhs.candidate.position[i].z) {
            return false;
        }
    }
    return true;
}

struct MultistepRun {
    bool completed = true;
    bool solver_valid = true;
    bool positive_reductions = true;
    bool rejected_state_immutable = true;
    bool capacity_valid = true;
    int requested_steps = 0;
    int completed_steps = 0;
    int total_outer_trials = 0;
    int total_accepted_trials = 0;
    int total_rejected_trials = 0;
    int total_objective_evaluations = 0;
    int total_hvp_calls = 0;
    int total_pair_builds = 0;
    int total_numerical_floor_stops = 0;
    int total_raw_gradient_stops = 0;
    int total_scaled_displacement_stops = 0;
    int maximum_outer_trials = 0;
    int maximum_rejected_trials = 0;
    int maximum_hvp_calls = 0;
    int initial_active_pressure_centers = 0;
    int final_active_pressure_centers = 0;
    int maximum_active_pressure_centers = 0;
    std::size_t maximum_pairs = 0;
    std::size_t maximum_neighbors = 0;
    std::size_t maximum_hessian_tape_bytes = 0;
    double total_mass = 0.0;
    double maximum_density_ratio = 0.0;
    double maximum_material_energy = 0.0;
    double accumulated_momentum_residual = 0.0;
    double maximum_final_scaled_displacement_residual = 0.0;
    double maximum_center_of_mass_drift = 0.0;
    std::string failure;
    std::vector<Vec3> position;
    std::vector<Vec3> velocity;
    std::vector<std::vector<Vec3>> position_history;
    std::vector<std::vector<Vec3>> velocity_history;
    std::vector<int> active_pressure_history;
    std::vector<std::string> step_signatures;
};

Config physical_multistep_config(double time_step = 1.0 / 240.0) {
    Config config;
    config.time_step = time_step;
    config.kernel_scale = reference_lattice_kernel_scale(
        config.spacing, config.horizon);
    config.kappa = 1226.25;
    config.lambda = 1.413823172873555e-5;
    config.mu = 0.0;
    config.gamma = 0.0;
    return config;
}

std::vector<Vec3> centered_lattice(int side, double spacing) {
    std::vector<Vec3> result;
    result.reserve(static_cast<std::size_t>(side * side * side));
    const double center = 0.5 * static_cast<double>(side - 1);
    for (int z = 0; z < side; ++z) {
        for (int y = 0; y < side; ++y) {
            for (int x = 0; x < side; ++x) {
                result.push_back({
                    (static_cast<double>(x) - center) * spacing,
                    (static_cast<double>(y) - center) * spacing,
                    (static_cast<double>(z) - center) * spacing,
                });
            }
        }
    }
    return result;
}

int pressure_active_count(
    const Config& config, const std::vector<Vec3>& position) {
    return active_count(pressure_active_signature(config, position));
}

double maximum_density_ratio(
    const Config& config, const std::vector<Vec3>& position) {
    const std::vector<double> value = densities(config, position);
    return *std::max_element(value.begin(), value.end())
        / config.rest_density;
}

bool trace_has_positive_reductions(
    const NeighborhoodTrustResult& result) {
    for (const NeighborhoodTrustResult::TrialTrace& trace : result.trace) {
        if (trace.accepted
            && (!trace.valid_model || trace.predicted_reduction <= 0.0
                || trace.actual_reduction <= 0.0 || trace.ratio < 0.1)) {
            return false;
        }
    }
    return true;
}

bool trace_preserves_rejected_state(
    const NeighborhoodTrustResult& result) {
    for (std::size_t i = 0; i < result.trace.size(); ++i) {
        if (result.trace[i].accepted) {
            continue;
        }
        const std::string next_state = i + 1 < result.trace.size()
            ? result.trace[i + 1].current_state_sha256
            : hash_positions(result.solve.position);
        if (result.trace[i].current_state_sha256 != next_state) {
            return false;
        }
    }
    return true;
}

std::string multistep_signature(
    const Config& config, const NeighborhoodTrustResult& result) {
    std::ostringstream material;
    material << result.solve.convergence_stop << '|'
             << result.solve.outer_trials << '|'
             << result.solve.accepted_trials << '|'
             << result.solve.rejected_trials << '|'
             << result.solve.hvp_calls << '|'
             << pressure_active_count(config, result.solve.position) << '|';
    for (const NeighborhoodTrustResult::TrialTrace& trace : result.trace) {
        material << trace.current_active << ':' << trace.trial_active << ':'
                 << (trace.accepted ? 1 : 0) << ':' << trace.inner_reason
                 << ':' << trace.inner_hvp_calls << '|';
    }
    return material.str();
}

std::string hash_phase_state(
    const std::vector<Vec3>& position,
    const std::vector<Vec3>& velocity) {
    std::ostringstream material;
    material << std::setprecision(17);
    for (std::size_t i = 0; i < position.size(); ++i) {
        material << position[i].x << ',' << position[i].y << ','
                 << position[i].z << ';' << velocity[i].x << ','
                 << velocity[i].y << ',' << velocity[i].z << '|';
    }
    return sha256_hex(material.str());
}

MultistepRun run_multistep(
    const Config& config,
    std::vector<Vec3> position,
    std::vector<Vec3> velocity,
    int steps,
    bool capture_history,
    bool numerical_floor_stop = true,
    double scaled_displacement_limit = 1.0e-8) {
    MultistepRun result;
    result.requested_steps = steps;
    result.total_mass = config.mass * static_cast<double>(position.size());
    const Vec3 initial_center = average(position);
    result.position = position;
    result.velocity = velocity;
    result.initial_active_pressure_centers =
        pressure_active_count(config, position);
    result.final_active_pressure_centers =
        result.initial_active_pressure_centers;
    result.maximum_active_pressure_centers =
        result.initial_active_pressure_centers;
    result.maximum_density_ratio = maximum_density_ratio(config, position);
    result.active_pressure_history.push_back(
        result.initial_active_pressure_centers);
    if (capture_history) {
        result.position_history.push_back(position);
        result.velocity_history.push_back(velocity);
    }
    for (int step = 0; step < steps; ++step) {
        const NeighborhoodTrustResult solve =
            solve_neighborhood_trust_region(config, position, velocity,
                true, numerical_floor_stop, false, true, true, false,
                scaled_displacement_limit);
        result.total_outer_trials += solve.solve.outer_trials;
        result.total_accepted_trials += solve.solve.accepted_trials;
        result.total_rejected_trials += solve.solve.rejected_trials;
        result.total_objective_evaluations +=
            solve.solve.objective_evaluations;
        result.total_hvp_calls += solve.solve.hvp_calls;
        result.total_pair_builds += solve.pair_builds;
        result.total_numerical_floor_stops +=
            solve.solve.numerical_floor_stops;
        result.total_raw_gradient_stops +=
            solve.solve.convergence_stop == "RAW_GRADIENT" ? 1 : 0;
        result.total_scaled_displacement_stops +=
            solve.solve.convergence_stop == "SCALED_DISPLACEMENT" ? 1 : 0;
        result.maximum_outer_trials = std::max(
            result.maximum_outer_trials, solve.solve.outer_trials);
        result.maximum_rejected_trials = std::max(
            result.maximum_rejected_trials, solve.solve.rejected_trials);
        result.maximum_hvp_calls = std::max(
            result.maximum_hvp_calls, solve.solve.hvp_calls);
        result.maximum_pairs = std::max(
            result.maximum_pairs, solve.maximum_pairs);
        result.maximum_neighbors = std::max(
            result.maximum_neighbors, solve.maximum_neighbors);
        result.maximum_hessian_tape_bytes = std::max(
            result.maximum_hessian_tape_bytes,
            solve.maximum_hessian_tape_bytes);
        result.maximum_final_scaled_displacement_residual = std::max(
            result.maximum_final_scaled_displacement_residual,
            solve.solve.final_scaled_displacement_residual);
        result.positive_reductions = result.positive_reductions
            && trace_has_positive_reductions(solve);
        result.rejected_state_immutable = result.rejected_state_immutable
            && trace_preserves_rejected_state(solve);
        result.capacity_valid = result.capacity_valid
            && solve.maximum_pairs <= 80 * position.size()
            && solve.maximum_neighbors <= 160
            && solve.maximum_hessian_tape_bytes
                <= solve.hessian_tape_capacity_bytes;
        result.solver_valid = result.solver_valid && solve.solve.succeeded
            && solve.solve.monotonic && solve.solve.failure.empty();
        if (!result.solver_valid || solve.solve.position.size() != position.size()) {
            result.completed = false;
            result.failure = solve.solve.failure.empty()
                ? "INVALID_SOLVER_RESULT" : solve.solve.failure;
            break;
        }
        std::vector<Vec3> next_velocity(position.size());
        for (std::size_t i = 0; i < position.size(); ++i) {
            next_velocity[i] =
                (solve.solve.position[i] - position[i]) / config.time_step;
        }
        result.maximum_material_energy = std::max(
            result.maximum_material_energy,
            std::abs(solve.solve.final.pressure)
                + std::abs(solve.solve.final.viscosity)
                + std::abs(solve.solve.final.surface));
        result.accumulated_momentum_residual +=
            solve.solve.final.internal_momentum_residual;
        position = solve.solve.position;
        velocity = next_velocity;
        result.maximum_center_of_mass_drift = std::max(
            result.maximum_center_of_mass_drift,
            norm(average(position) - initial_center));
        ++result.completed_steps;
        result.final_active_pressure_centers =
            pressure_active_count(config, position);
        result.maximum_active_pressure_centers = std::max(
            result.maximum_active_pressure_centers,
            result.final_active_pressure_centers);
        result.maximum_density_ratio = std::max(
            result.maximum_density_ratio,
            maximum_density_ratio(config, position));
        result.active_pressure_history.push_back(
            result.final_active_pressure_centers);
        result.step_signatures.push_back(
            multistep_signature(config, solve));
        if (capture_history) {
            result.position_history.push_back(position);
            result.velocity_history.push_back(velocity);
        }
    }
    result.position = position;
    result.velocity = velocity;
    result.completed = result.completed && result.completed_steps == steps;
    return result;
}

struct FreeFlightControl {
    MultistepRun run;
    double maximum_position_error = 0.0;
    double maximum_velocity_error = 0.0;
    bool passed = false;
};

FreeFlightControl free_flight_multistep_control() {
    Config config = physical_multistep_config();
    config.gravity = {0.0, -9.81, 0.0};
    const Vec3 initial_position = {0.25, -0.1, 0.4};
    const Vec3 initial_velocity = {1.2, 0.7, -0.35};
    FreeFlightControl result;
    result.run = run_multistep(
        config, {initial_position}, {initial_velocity}, 240, true);
    for (std::size_t step = 0;
         step < result.run.position_history.size(); ++step) {
        const double n = static_cast<double>(step);
        const Vec3 expected_velocity =
            initial_velocity + n * config.time_step * config.gravity;
        const Vec3 expected_position = initial_position
            + n * config.time_step * initial_velocity
            + 0.5 * n * (n + 1.0) * config.time_step
                * config.time_step * config.gravity;
        result.maximum_position_error = std::max(
            result.maximum_position_error,
            vector_relative_error(
                result.run.position_history[step][0], expected_position));
        result.maximum_velocity_error = std::max(
            result.maximum_velocity_error,
            vector_relative_error(
                result.run.velocity_history[step][0], expected_velocity));
    }
    result.passed = result.run.completed && result.run.solver_valid
        && result.run.total_outer_trials == 0
        && result.run.total_hvp_calls == 0
        && result.maximum_position_error <= 1.0e-11
        && result.maximum_velocity_error <= 1.0e-11;
    return result;
}

struct RigidTranslationControl {
    MultistepRun run;
    double maximum_position_error = 0.0;
    double maximum_velocity_error = 0.0;
    double maximum_center_of_mass_error = 0.0;
    bool passed = false;
};

RigidTranslationControl rigid_translation_multistep_control() {
    const Config config = physical_multistep_config();
    const std::vector<Vec3> initial = centered_lattice(4, config.spacing);
    const Vec3 translation_velocity = {0.37, -0.21, 0.13};
    std::vector<Vec3> velocity(initial.size(), translation_velocity);
    RigidTranslationControl result;
    result.run = run_multistep(config, initial, velocity, 240, true);
    const Vec3 initial_center = average(initial);
    for (std::size_t step = 0;
         step < result.run.position_history.size(); ++step) {
        const Vec3 offset = static_cast<double>(step)
            * config.time_step * translation_velocity;
        for (std::size_t i = 0; i < initial.size(); ++i) {
            result.maximum_position_error = std::max(
                result.maximum_position_error,
                norm(result.run.position_history[step][i]
                    - (initial[i] + offset)));
            result.maximum_velocity_error = std::max(
                result.maximum_velocity_error,
                norm(result.run.velocity_history[step][i]
                    - translation_velocity));
        }
        result.maximum_center_of_mass_error = std::max(
            result.maximum_center_of_mass_error,
            norm(average(result.run.position_history[step])
                - (initial_center + offset)));
    }
    result.passed = result.run.completed && result.run.solver_valid
        && result.run.positive_reductions
        && result.run.rejected_state_immutable
        && result.run.maximum_active_pressure_centers == 0
        && result.run.maximum_material_energy <= 1.0e-12
        && result.run.accumulated_momentum_residual <= 1.0e-12
        && result.maximum_position_error <= 1.0e-11
        && result.maximum_velocity_error <= 1.0e-11
        && result.maximum_center_of_mass_error <= 1.0e-11;
    return result;
}

struct GalileanControl {
    MultistepRun reference;
    MultistepRun boosted;
    double maximum_position_error = 0.0;
    double maximum_velocity_error = 0.0;
    bool signatures_exact = false;
    bool passed = false;
};

GalileanControl galilean_multistep_control() {
    const CombinedFixture source = combined_fixture();
    Config config = physical_multistep_config();
    config.kappa = 200.0;
    config.lambda = 20.0;
    double center_density = config.mass
        * configured_cubic_weight(config, 0.0);
    for (std::size_t i = 1; i < source.x.size(); ++i) {
        center_density += config.mass * configured_cubic_weight(
            config, norm(source.x[0] - source.x[i]));
    }
    config.rest_density = center_density / 1.1;
    const Vec3 boost = {0.6, -0.3, 0.2};
    std::vector<Vec3> boosted_velocity = source.velocity;
    for (Vec3& value : boosted_velocity) {
        value += boost;
    }
    GalileanControl result;
    result.reference = run_multistep(
        config, source.x, source.velocity, 32, true);
    result.boosted = run_multistep(
        config, source.x, boosted_velocity, 32, true);
    const std::size_t history_size = std::min(
        result.reference.position_history.size(),
        result.boosted.position_history.size());
    for (std::size_t step = 0; step < history_size; ++step) {
        const Vec3 offset = static_cast<double>(step)
            * config.time_step * boost;
        for (std::size_t i = 0; i < source.x.size(); ++i) {
            result.maximum_position_error = std::max(
                result.maximum_position_error,
                norm(result.boosted.position_history[step][i] - offset
                    - result.reference.position_history[step][i]));
            result.maximum_velocity_error = std::max(
                result.maximum_velocity_error,
                norm(result.boosted.velocity_history[step][i] - boost
                    - result.reference.velocity_history[step][i]));
        }
    }
    result.signatures_exact = result.reference.step_signatures
        == result.boosted.step_signatures;
    result.passed = result.reference.completed && result.boosted.completed
        && result.reference.solver_valid && result.boosted.solver_valid
        && result.reference.positive_reductions
        && result.boosted.positive_reductions
        && result.reference.rejected_state_immutable
        && result.boosted.rejected_state_immutable
        && result.signatures_exact
        && result.maximum_position_error <= 1.0e-10
        && result.maximum_velocity_error <= 1.0e-10;
    return result;
}

double mass_weighted_rms_position_error(
    const std::vector<Vec3>& lhs,
    const std::vector<Vec3>& rhs) {
    double squared = 0.0;
    for (std::size_t i = 0; i < lhs.size(); ++i) {
        squared += norm_squared(lhs[i] - rhs[i]);
    }
    return std::sqrt(squared / static_cast<double>(lhs.size()));
}

struct CompressionControl {
    std::array<MultistepRun, 3> runs;
    std::array<double, 3> time_steps{};
    int initial_active_pressure_centers = 0;
    double initial_maximum_density_ratio = 0.0;
    double coarse_error = 0.0;
    double fine_error = 0.0;
    double convergence_ratio = 0.0;
    double maximum_normalized_center_of_mass_drift = 0.0;
    double maximum_accumulated_momentum_residual = 0.0;
    double effective_bulk_modulus = 0.0;
    double acoustic_wave_speed = 0.0;
    std::array<double, 3> acoustic_courant{};
    bool passed = false;
};

CompressionControl compression_multistep_control() {
    CompressionControl result;
    result.time_steps = {
        1.0 / 240.0, 1.0 / 480.0, 1.0 / 960.0,
    };
    const Config anchor = physical_multistep_config();
    result.effective_bulk_modulus =
        anchor.kappa * anchor.rest_density / anchor.mass;
    result.acoustic_wave_speed = std::sqrt(
        result.effective_bulk_modulus / anchor.rest_density);
    for (std::size_t i = 0; i < result.acoustic_courant.size(); ++i) {
        result.acoustic_courant[i] = result.time_steps[i]
            * result.acoustic_wave_speed / anchor.spacing;
    }
    for (std::size_t level = 0; level < result.runs.size(); ++level) {
        const Config config = physical_multistep_config(
            result.time_steps[level]);
        std::vector<Vec3> position = centered_lattice(7, config.spacing);
        for (Vec3& value : position) {
            value = 0.99 * value;
        }
        if (level == 0) {
            result.initial_active_pressure_centers =
                pressure_active_count(config, position);
            result.initial_maximum_density_ratio =
                maximum_density_ratio(config, position);
        }
        const Vec3 initial_center = average(position);
        const int steps = static_cast<int>(std::llround(
            0.05 / config.time_step));
        result.runs[level] = run_multistep(
            config, position, std::vector<Vec3>(position.size()),
            steps, false);
        result.maximum_normalized_center_of_mass_drift = std::max(
            result.maximum_normalized_center_of_mass_drift,
            norm(average(result.runs[level].position) - initial_center)
                / config.spacing);
        result.maximum_accumulated_momentum_residual = std::max(
            result.maximum_accumulated_momentum_residual,
            result.runs[level].accumulated_momentum_residual);
    }
    result.coarse_error = mass_weighted_rms_position_error(
        result.runs[0].position, result.runs[1].position);
    result.fine_error = mass_weighted_rms_position_error(
        result.runs[1].position, result.runs[2].position);
    result.convergence_ratio = result.coarse_error / result.fine_error;
    bool run_gates = true;
    for (const MultistepRun& run : result.runs) {
        run_gates = run_gates && run.completed && run.solver_valid
            && run.positive_reductions && run.rejected_state_immutable
            && run.capacity_valid && run.maximum_outer_trials <= 32
            && run.maximum_rejected_trials <= 8
            && run.maximum_hvp_calls <= 128
            && run.maximum_density_ratio
                <= result.initial_maximum_density_ratio + 1.0e-6
            && run.final_active_pressure_centers
                < result.initial_active_pressure_centers;
    }
    result.passed = run_gates
        && result.maximum_normalized_center_of_mass_drift <= 1.0e-11
        && result.maximum_accumulated_momentum_residual <= 1.0e-10
        && result.fine_error > 1.0e-14
        && result.convergence_ratio >= 1.5;
    return result;
}

Vec3 rotate_about_z(Vec3 value, double angle) {
    const double cosine = std::cos(angle);
    const double sine = std::sin(angle);
    return {
        cosine * value.x - sine * value.y,
        sine * value.x + cosine * value.y,
        value.z,
    };
}

double accumulated_rotation_viscosity(double time_step, double mu) {
    Config config = physical_multistep_config(time_step);
    config.kappa = 0.0;
    config.mu = mu;
    const std::vector<Vec3> reference =
        centered_lattice(4, config.spacing);
    const int steps = static_cast<int>(std::llround(0.25 / time_step));
    double energy = 0.0;
    for (int step = 0; step < steps; ++step) {
        std::vector<Vec3> x(reference.size());
        std::vector<Vec3> y(reference.size());
        const double angle = 2.0 * static_cast<double>(step) * time_step;
        const double next_angle =
            2.0 * static_cast<double>(step + 1) * time_step;
        for (std::size_t i = 0; i < reference.size(); ++i) {
            x[i] = rotate_about_z(reference[i], angle);
            y[i] = rotate_about_z(reference[i], next_angle);
        }
        energy += evaluate(config, x, y, y).viscosity;
    }
    return energy;
}

struct RotationControl {
    std::array<double, 3> selected_energy{};
    std::array<double, 3> comparator_energy{};
    std::array<double, 2> selected_ratio{};
    std::array<double, 2> comparator_ratio{};
    bool passed = false;
};

RotationControl rotation_objectivity_control() {
    const std::array<double, 3> time_steps = {
        1.0 / 240.0, 1.0 / 480.0, 1.0 / 960.0,
    };
    constexpr double lambda = 1.413823172873555e-5;
    RotationControl result;
    for (std::size_t i = 0; i < time_steps.size(); ++i) {
        result.selected_energy[i] =
            accumulated_rotation_viscosity(time_steps[i], 0.0);
        result.comparator_energy[i] =
            accumulated_rotation_viscosity(time_steps[i], lambda);
    }
    for (std::size_t i = 0; i < result.selected_ratio.size(); ++i) {
        result.selected_ratio[i] =
            result.selected_energy[i] / result.selected_energy[i + 1];
        result.comparator_ratio[i] =
            result.comparator_energy[i] / result.comparator_energy[i + 1];
    }
    result.passed = std::all_of(
            result.selected_energy.begin(), result.selected_energy.end(),
            [](double value) { return std::isfinite(value) && value > 0.0; })
        && std::all_of(
            result.comparator_energy.begin(), result.comparator_energy.end(),
            [](double value) { return std::isfinite(value) && value > 0.0; })
        && std::all_of(
            result.selected_ratio.begin(), result.selected_ratio.end(),
            [](double value) { return value >= 3.5 && value <= 4.5; })
        && std::all_of(
            result.comparator_ratio.begin(), result.comparator_ratio.end(),
            [](double value) { return value >= 0.8 && value <= 1.2; });
    return result;
}

double rms_radius_about_center(const std::vector<Vec3>& position) {
    const Vec3 center = average(position);
    double squared = 0.0;
    for (Vec3 value : position) {
        squared += norm_squared(value - center);
    }
    return std::sqrt(squared / static_cast<double>(position.size()));
}

double rms_relative_speed(const std::vector<Vec3>& velocity) {
    const Vec3 center_velocity = average(velocity);
    double squared = 0.0;
    for (Vec3 value : velocity) {
        squared += norm_squared(value - center_velocity);
    }
    return std::sqrt(squared / static_cast<double>(velocity.size()));
}

double relative_kinetic_energy(
    const Config& config, const std::vector<Vec3>& velocity) {
    const double speed = rms_relative_speed(velocity);
    return 0.5 * config.mass * static_cast<double>(velocity.size())
        * speed * speed;
}

double pressure_active_exit_time(
    const MultistepRun& run, double time_step) {
    int last_active = -1;
    for (std::size_t i = 0; i < run.active_pressure_history.size(); ++i) {
        if (run.active_pressure_history[i] != 0) {
            last_active = static_cast<int>(i);
        }
    }
    return static_cast<double>(last_active + 1) * time_step;
}

double maximum_normalized_center_drift(
    const MultistepRun& run, Vec3 initial_center, double spacing) {
    double result = norm(average(run.position) - initial_center) / spacing;
    for (const std::vector<Vec3>& position : run.position_history) {
        result = std::max(result,
            norm(average(position) - initial_center) / spacing);
    }
    return result;
}

struct TemporalLevel {
    double time_step = 0.0;
    double acoustic_courant = 0.0;
    MultistepRun run;
    double rms_radius = 0.0;
    double rms_speed = 0.0;
    double kinetic_energy = 0.0;
    double pressure_exit_time = 0.0;
    double normalized_center_drift = 0.0;
};

struct TemporalStiffnessDiagnostic {
    std::array<TemporalLevel, 5> levels;
    std::array<TemporalLevel, 2> strict_levels;
    std::array<double, 4> position_difference{};
    std::array<double, 4> velocity_difference{};
    std::array<double, 3> position_ratio{};
    std::array<double, 3> velocity_ratio{};
    std::array<double, 4> radius_change{};
    std::array<double, 4> speed_change{};
    std::array<double, 4> kinetic_change{};
    std::array<double, 4> exit_time_change{};
    std::array<double, 2> strict_position_difference{};
    std::array<double, 2> strict_velocity_difference{};
    double initial_maximum_density_ratio = 0.0;
    bool overlap_exact = false;
    bool solver_sensitivity_valid = false;
    bool validity_passed = false;
    bool asymptotic_regime_observed = false;
    bool passed = false;
    std::string first_failure;
    std::string disposition;
};

TemporalLevel make_temporal_level(
    double time_step, bool strict, double acoustic_wave_speed) {
    const Config config = physical_multistep_config(time_step);
    std::vector<Vec3> position = centered_lattice(7, config.spacing);
    for (Vec3& value : position) {
        value = 0.99 * value;
    }
    const Vec3 initial_center = average(position);
    TemporalLevel result;
    result.time_step = time_step;
    result.acoustic_courant =
        time_step * acoustic_wave_speed / config.spacing;
    result.run = run_multistep(config, position,
        std::vector<Vec3>(position.size()),
        static_cast<int>(std::llround(0.05 / time_step)), !strict,
        !strict, strict ? 1.0e-10 : 1.0e-8);
    result.rms_radius = rms_radius_about_center(result.run.position);
    result.rms_speed = rms_relative_speed(result.run.velocity);
    result.kinetic_energy =
        relative_kinetic_energy(config, result.run.velocity);
    result.pressure_exit_time =
        pressure_active_exit_time(result.run, time_step);
    result.normalized_center_drift = maximum_normalized_center_drift(
        result.run, initial_center, config.spacing);
    return result;
}

bool temporal_level_valid(
    const TemporalLevel& level, double initial_maximum_density_ratio) {
    const MultistepRun& run = level.run;
    return run.completed && run.solver_valid && run.positive_reductions
        && run.rejected_state_immutable && run.capacity_valid
        && run.maximum_outer_trials <= 32
        && run.maximum_rejected_trials <= 8
        && run.maximum_hvp_calls <= 128
        && run.maximum_density_ratio
            <= initial_maximum_density_ratio + 1.0e-6
        && run.final_active_pressure_centers == 0
        && level.normalized_center_drift <= 1.0e-11
        && run.accumulated_momentum_residual <= 1.0e-10
        && std::isfinite(level.rms_radius)
        && std::isfinite(level.rms_speed)
        && std::isfinite(level.kinetic_energy)
        && std::isfinite(level.pressure_exit_time);
}

TemporalStiffnessDiagnostic temporal_stiffness_diagnostic() {
    TemporalStiffnessDiagnostic result;
    const Config anchor = physical_multistep_config();
    const double effective_bulk_modulus =
        anchor.kappa * anchor.rest_density / anchor.mass;
    const double acoustic_wave_speed = std::sqrt(
        effective_bulk_modulus / anchor.rest_density);
    const std::array<double, 5> time_steps = {
        1.0 / 960.0,
        1.0 / 1920.0,
        1.0 / 3840.0,
        1.0 / 7680.0,
        1.0 / 15360.0,
    };
    std::vector<Vec3> initial = centered_lattice(7, anchor.spacing);
    for (Vec3& value : initial) {
        value = 0.99 * value;
    }
    result.initial_maximum_density_ratio =
        maximum_density_ratio(anchor, initial);
    for (std::size_t i = 0; i < result.levels.size(); ++i) {
        result.levels[i] = make_temporal_level(
            time_steps[i], false, acoustic_wave_speed);
    }
    result.strict_levels[0] = make_temporal_level(
        time_steps[3], true, acoustic_wave_speed);
    result.strict_levels[1] = make_temporal_level(
        time_steps[4], true, acoustic_wave_speed);
    for (std::size_t i = 0; i < result.position_difference.size(); ++i) {
        result.position_difference[i] = mass_weighted_rms_position_error(
            result.levels[i].run.position,
            result.levels[i + 1].run.position);
        result.velocity_difference[i] = mass_weighted_rms_position_error(
            result.levels[i].run.velocity,
            result.levels[i + 1].run.velocity);
        result.radius_change[i] = std::abs(
            result.levels[i].rms_radius
                - result.levels[i + 1].rms_radius);
        result.speed_change[i] = std::abs(
            result.levels[i].rms_speed
                - result.levels[i + 1].rms_speed);
        result.kinetic_change[i] = std::abs(
            result.levels[i].kinetic_energy
                - result.levels[i + 1].kinetic_energy);
        result.exit_time_change[i] = std::abs(
            result.levels[i].pressure_exit_time
                - result.levels[i + 1].pressure_exit_time);
    }
    for (std::size_t i = 0; i < result.position_ratio.size(); ++i) {
        result.position_ratio[i] =
            result.position_difference[i]
            / result.position_difference[i + 1];
        result.velocity_ratio[i] =
            result.velocity_difference[i]
            / result.velocity_difference[i + 1];
    }
    for (std::size_t i = 0; i < result.strict_levels.size(); ++i) {
        result.strict_position_difference[i] =
            mass_weighted_rms_position_error(
                result.levels[i + 3].run.position,
                result.strict_levels[i].run.position);
        result.strict_velocity_difference[i] =
            mass_weighted_rms_position_error(
                result.levels[i + 3].run.velocity,
                result.strict_levels[i].run.velocity);
    }
    const MultistepRun& overlap = result.levels[0].run;
    result.overlap_exact =
        hash_phase_state(overlap.position, overlap.velocity)
            == "7f667eb41a86e1c840630ff8d81da5a258a759a6106442c2407f28ac5069028e"
        && overlap.total_outer_trials == 59
        && overlap.total_accepted_trials == 59
        && overlap.total_rejected_trials == 0
        && overlap.total_objective_evaluations == 107
        && overlap.total_hvp_calls == 120
        && overlap.total_pair_builds == 155
        && overlap.maximum_outer_trials == 10
        && overlap.maximum_rejected_trials == 0
        && overlap.maximum_hvp_calls == 22
        && overlap.maximum_pairs == 12111
        && overlap.maximum_neighbors == 122
        && overlap.maximum_hessian_tape_bytes == 2150744;
    result.solver_sensitivity_valid = true;
    for (std::size_t i = 0; i < result.strict_levels.size(); ++i) {
        result.solver_sensitivity_valid = result.solver_sensitivity_valid
            && result.strict_position_difference[i]
                <= 0.1 * result.position_difference[3]
            && result.strict_velocity_difference[i]
                <= 0.1 * result.velocity_difference[3];
    }
    bool levels_valid = true;
    for (const TemporalLevel& level : result.levels) {
        levels_valid = levels_valid && temporal_level_valid(
            level, result.initial_maximum_density_ratio);
    }
    for (const TemporalLevel& level : result.strict_levels) {
        levels_valid = levels_valid && temporal_level_valid(
            level, result.initial_maximum_density_ratio);
    }
    bool differences_valid = true;
    for (double value : result.position_difference) {
        differences_valid = differences_valid
            && std::isfinite(value) && value > 0.0;
    }
    for (double value : result.velocity_difference) {
        differences_valid = differences_valid
            && std::isfinite(value) && value > 0.0;
    }
    result.validity_passed = result.overlap_exact && levels_valid
        && differences_valid && result.solver_sensitivity_valid;
    if (!result.overlap_exact) {
        result.first_failure = "NSR3B1D_B1_OVERLAP";
    } else if (!levels_valid) {
        result.first_failure = "NSR3B1D_LEVEL_VALIDITY";
    } else if (!differences_valid) {
        result.first_failure = "NSR3B1D_DIFFERENCE_VALIDITY";
    } else if (!result.solver_sensitivity_valid) {
        result.first_failure = "NSR3B1D_SOLVER_SENSITIVITY";
    }
    const auto ratio_in_range = [](double value) {
        return value >= 1.5 && value <= 2.5;
    };
    result.asymptotic_regime_observed = result.validity_passed
        && ratio_in_range(result.position_ratio[1])
        && ratio_in_range(result.position_ratio[2])
        && ratio_in_range(result.velocity_ratio[1])
        && ratio_in_range(result.velocity_ratio[2])
        && result.position_difference[1] > result.position_difference[2]
        && result.position_difference[2] > result.position_difference[3]
        && result.velocity_difference[1] > result.velocity_difference[2]
        && result.velocity_difference[2] > result.velocity_difference[3]
        && result.radius_change[2] > result.radius_change[3]
        && result.speed_change[2] > result.speed_change[3]
        && result.kinetic_change[2] > result.kinetic_change[3];
    result.disposition = !result.validity_passed
        ? "INVALID_DIAGNOSTIC"
        : result.asymptotic_regime_observed
            ? "ASYMPTOTIC_REGIME_OBSERVED"
            : "NO_ASYMPTOTIC_REGIME_AT_C0P129";
    result.passed = result.validity_passed;
    return result;
}

TemporalLevel make_floor_limited_temporal_level(
    double time_step, double acoustic_wave_speed) {
    const Config config = physical_multistep_config(time_step);
    std::vector<Vec3> position = centered_lattice(7, config.spacing);
    for (Vec3& value : position) {
        value = 0.99 * value;
    }
    const Vec3 initial_center = average(position);
    TemporalLevel result;
    result.time_step = time_step;
    result.acoustic_courant =
        time_step * acoustic_wave_speed / config.spacing;
    result.run = run_multistep(config, position,
        std::vector<Vec3>(position.size()),
        static_cast<int>(std::llround(0.05 / time_step)), false,
        true, 0.0);
    result.rms_radius = rms_radius_about_center(result.run.position);
    result.rms_speed = rms_relative_speed(result.run.velocity);
    result.kinetic_energy =
        relative_kinetic_energy(config, result.run.velocity);
    result.pressure_exit_time =
        pressure_active_exit_time(result.run, time_step);
    result.normalized_center_drift = maximum_normalized_center_drift(
        result.run, initial_center, config.spacing);
    return result;
}

bool exact_b1d_main_overlap(const std::array<TemporalLevel, 5>& levels) {
    const std::array<const char*, 5> state = {
        "7f667eb41a86e1c840630ff8d81da5a258a759a6106442c2407f28ac5069028e",
        "3611e20b08b5b400599b4a46edfa765ef1783d033456d9221bb505b1f3eda1b5",
        "1ea70858651d4d548a49934be6cb3054945cc35af38b0d58e87807be33cf1bed",
        "7078e43076b2209bd99fdb879e9294869223fe0792aceaa3b31b668620bf7442",
        "3f320beae99be233c5675ff58656f38d6d533638e3104503410b10d7b0ed4c04",
    };
    const std::array<int, 5> steps = {48, 96, 192, 384, 768};
    const std::array<int, 5> outer = {59, 14, 20, 24, 46};
    const std::array<int, 5> accepted = {59, 14, 20, 24, 46};
    const std::array<int, 5> evaluations = {107, 110, 212, 408, 814};
    const std::array<int, 5> hvp = {120, 30, 40, 48, 92};
    const std::array<int, 5> pair_builds = {155, 206, 404, 792, 1582};
    const std::array<int, 5> max_outer = {10, 6, 4, 3, 2};
    const std::array<int, 5> max_hvp = {22, 13, 8, 6, 4};
    const std::array<double, 5> rms_radius = {
        0.18283038237912211,
        0.18689165663253796,
        0.19067902061536393,
        0.19318344728051465,
        0.19463469052832796,
    };
    const std::array<double, 5> rms_speed = {
        0.43353697919866402,
        0.56798440525792315,
        0.69379904189524166,
        0.77495570476642417,
        0.8215886220790517,
    };
    const std::array<double, 5> kinetic = {
        4.0292705706323177,
        6.9158722264597161,
        10.319093057088816,
        12.874426632003663,
        14.47048108299412,
    };
    const std::array<double, 5> exit_time = {
        0.0031250000000000002,
        0.0020833333333333333,
        0.0018229166666666667,
        0.0018229166666666667,
        0.0017578125,
    };
    const std::array<double, 5> center_drift = {
        5.157623052278392e-16,
        6.3416126392291555e-16,
        2.3745614744111542e-15,
        2.5166774225595801e-15,
        3.3325307369518189e-16,
    };
    const std::array<double, 5> scaled_residual = {
        6.4276490284564593e-9,
        7.7145634125635067e-9,
        7.7642953905452552e-9,
        7.9753874151301836e-9,
        4.26685749162269e-10,
    };
    const std::array<double, 5> momentum = {
        2.3171599749710689e-15,
        3.7163468417334267e-15,
        8.3184877664114232e-15,
        1.6620009271471234e-14,
        3.7925748872932176e-14,
    };
    for (std::size_t i = 0; i < levels.size(); ++i) {
        const MultistepRun& run = levels[i].run;
        if (hash_phase_state(run.position, run.velocity) != state[i]
            || run.completed_steps != steps[i]
            || run.total_outer_trials != outer[i]
            || run.total_accepted_trials != accepted[i]
            || run.total_rejected_trials != 0
            || run.total_objective_evaluations != evaluations[i]
            || run.total_hvp_calls != hvp[i]
            || run.total_pair_builds != pair_builds[i]
            || run.maximum_outer_trials != max_outer[i]
            || run.maximum_rejected_trials != 0
            || run.maximum_hvp_calls != max_hvp[i]
            || run.maximum_pairs != 12111
            || run.maximum_neighbors != 122
            || run.maximum_hessian_tape_bytes != 2150744
            || run.maximum_density_ratio != 1.0308294231753403
            || run.final_active_pressure_centers != 0
            || levels[i].rms_radius != rms_radius[i]
            || levels[i].rms_speed != rms_speed[i]
            || levels[i].kinetic_energy != kinetic[i]
            || levels[i].pressure_exit_time != exit_time[i]
            || levels[i].normalized_center_drift != center_drift[i]
            || run.maximum_final_scaled_displacement_residual
                != scaled_residual[i]
            || run.accumulated_momentum_residual != momentum[i]) {
            return false;
        }
    }
    return true;
}

struct FloorLimitedOracleDiagnostic {
    std::array<TemporalLevel, 5> main_levels;
    std::array<TemporalLevel, 2> oracle_levels;
    std::array<double, 4> position_difference{};
    std::array<double, 4> velocity_difference{};
    std::array<double, 3> position_ratio{};
    std::array<double, 3> velocity_ratio{};
    std::array<double, 4> radius_change{};
    std::array<double, 4> speed_change{};
    std::array<double, 4> kinetic_change{};
    std::array<double, 2> oracle_position_difference{};
    std::array<double, 2> oracle_velocity_difference{};
    double initial_maximum_density_ratio = 0.0;
    bool main_overlap_exact = false;
    bool oracle_valid = false;
    bool sensitivity_valid = false;
    bool asymptotic_regime_observed = false;
    bool passed = false;
    std::string first_failure;
    std::string disposition;
};

FloorLimitedOracleDiagnostic floor_limited_oracle_diagnostic() {
    FloorLimitedOracleDiagnostic result;
    const Config anchor = physical_multistep_config();
    const double effective_bulk_modulus =
        anchor.kappa * anchor.rest_density / anchor.mass;
    const double acoustic_wave_speed = std::sqrt(
        effective_bulk_modulus / anchor.rest_density);
    const std::array<double, 5> time_steps = {
        1.0 / 960.0,
        1.0 / 1920.0,
        1.0 / 3840.0,
        1.0 / 7680.0,
        1.0 / 15360.0,
    };
    std::vector<Vec3> initial = centered_lattice(7, anchor.spacing);
    for (Vec3& value : initial) {
        value = 0.99 * value;
    }
    result.initial_maximum_density_ratio =
        maximum_density_ratio(anchor, initial);
    for (std::size_t i = 0; i < result.main_levels.size(); ++i) {
        result.main_levels[i] = make_temporal_level(
            time_steps[i], false, acoustic_wave_speed);
    }
    result.oracle_levels[0] = make_floor_limited_temporal_level(
        time_steps[3], acoustic_wave_speed);
    result.oracle_levels[1] = make_floor_limited_temporal_level(
        time_steps[4], acoustic_wave_speed);
    for (std::size_t i = 0; i < result.position_difference.size(); ++i) {
        result.position_difference[i] = mass_weighted_rms_position_error(
            result.main_levels[i].run.position,
            result.main_levels[i + 1].run.position);
        result.velocity_difference[i] = mass_weighted_rms_position_error(
            result.main_levels[i].run.velocity,
            result.main_levels[i + 1].run.velocity);
        result.radius_change[i] = std::abs(
            result.main_levels[i].rms_radius
                - result.main_levels[i + 1].rms_radius);
        result.speed_change[i] = std::abs(
            result.main_levels[i].rms_speed
                - result.main_levels[i + 1].rms_speed);
        result.kinetic_change[i] = std::abs(
            result.main_levels[i].kinetic_energy
                - result.main_levels[i + 1].kinetic_energy);
    }
    for (std::size_t i = 0; i < result.position_ratio.size(); ++i) {
        result.position_ratio[i] = result.position_difference[i]
            / result.position_difference[i + 1];
        result.velocity_ratio[i] = result.velocity_difference[i]
            / result.velocity_difference[i + 1];
    }
    for (std::size_t i = 0; i < result.oracle_levels.size(); ++i) {
        result.oracle_position_difference[i] =
            mass_weighted_rms_position_error(
                result.main_levels[i + 3].run.position,
                result.oracle_levels[i].run.position);
        result.oracle_velocity_difference[i] =
            mass_weighted_rms_position_error(
                result.main_levels[i + 3].run.velocity,
                result.oracle_levels[i].run.velocity);
    }
    result.main_overlap_exact = exact_b1d_main_overlap(result.main_levels);
    result.oracle_valid = true;
    int numerical_floor_stops = 0;
    for (const TemporalLevel& level : result.oracle_levels) {
        result.oracle_valid = result.oracle_valid
            && temporal_level_valid(
                level, result.initial_maximum_density_ratio);
        numerical_floor_stops += level.run.total_numerical_floor_stops;
    }
    result.oracle_valid = result.oracle_valid
        && numerical_floor_stops > 0;
    constexpr double temporal_position_difference =
        0.0022880058867231971;
    constexpr double temporal_velocity_difference =
        0.04678768432527124;
    result.sensitivity_valid = result.oracle_valid;
    for (std::size_t i = 0; i < result.oracle_levels.size(); ++i) {
        result.sensitivity_valid = result.sensitivity_valid
            && result.oracle_position_difference[i]
                <= 0.1 * temporal_position_difference
            && result.oracle_velocity_difference[i]
                <= 0.1 * temporal_velocity_difference;
    }
    const auto ratio_in_range = [](double value) {
        return value >= 1.5 && value <= 2.5;
    };
    result.asymptotic_regime_observed = result.main_overlap_exact
        && ratio_in_range(result.position_ratio[1])
        && ratio_in_range(result.position_ratio[2])
        && ratio_in_range(result.velocity_ratio[1])
        && ratio_in_range(result.velocity_ratio[2])
        && result.position_difference[1] > result.position_difference[2]
        && result.position_difference[2] > result.position_difference[3]
        && result.velocity_difference[1] > result.velocity_difference[2]
        && result.velocity_difference[2] > result.velocity_difference[3]
        && result.radius_change[2] > result.radius_change[3]
        && result.speed_change[2] > result.speed_change[3]
        && result.kinetic_change[2] > result.kinetic_change[3];
    result.passed = result.main_overlap_exact
        && result.oracle_valid && result.sensitivity_valid;
    if (!result.main_overlap_exact) {
        result.first_failure = "NSR3B1D1_MAIN_OVERLAP";
    } else if (!result.oracle_valid) {
        result.first_failure = "NSR3B1D1_ORACLE_VALIDITY";
    } else if (!result.sensitivity_valid) {
        result.first_failure = "NSR3B1D1_SOLVER_SENSITIVITY";
    }
    result.disposition = !result.passed
        ? "INVALID_DIAGNOSTIC"
        : result.asymptotic_regime_observed
            ? "TEMPORAL_STIFFNESS_CONFIRMED"
            : "NO_ASYMPTOTIC_REGIME_AT_C0P129";
    return result;
}

int acoustic_substep_count(
    double frame_time, double spacing, double kappa, double mass) {
    const double wave_speed = kappa > 0.0
        ? std::sqrt(kappa / mass) : 0.0;
    const double frame_courant = frame_time * wave_speed / spacing;
    return std::max(1, static_cast<int>(std::ceil(frame_courant / 0.25)));
}

TemporalLevel make_policy_temporal_level(
    double compression_factor,
    double kappa_factor,
    int substeps_per_frame) {
    constexpr double frame_time = 1.0 / 240.0;
    Config config = physical_multistep_config(
        frame_time / static_cast<double>(substeps_per_frame));
    config.kappa *= kappa_factor;
    std::vector<Vec3> position = centered_lattice(7, config.spacing);
    for (Vec3& value : position) {
        value = compression_factor * value;
    }
    TemporalLevel result;
    result.time_step = config.time_step;
    const double wave_speed = std::sqrt(config.kappa / config.mass);
    result.acoustic_courant =
        config.time_step * wave_speed / config.spacing;
    result.run = run_multistep(config, position,
        std::vector<Vec3>(position.size()),
        3 * substeps_per_frame, false);
    result.rms_radius = rms_radius_about_center(result.run.position);
    result.rms_speed = rms_relative_speed(result.run.velocity);
    result.kinetic_energy =
        relative_kinetic_energy(config, result.run.velocity);
    result.pressure_exit_time =
        pressure_active_exit_time(result.run, config.time_step);
    result.normalized_center_drift =
        result.run.maximum_center_of_mass_drift / config.spacing;
    return result;
}

struct AcousticPolicyCase {
    double compression_factor = 0.0;
    double kappa_factor = 0.0;
    double wave_speed = 0.0;
    double initial_maximum_density_ratio = 0.0;
    int policy_substeps = 0;
    std::array<TemporalLevel, 3> levels;
    std::array<double, 2> position_difference{};
    std::array<double, 2> velocity_difference{};
    double position_ratio = 0.0;
    double velocity_ratio = 0.0;
    double normalized_policy_position_error = 0.0;
    double normalized_policy_velocity_error = 0.0;
    double relative_kinetic_error = 0.0;
    double pressure_exit_time_error = 0.0;
    bool passed = false;
};

AcousticPolicyCase make_acoustic_policy_case(
    double compression_factor, double kappa_factor) {
    constexpr double frame_time = 1.0 / 240.0;
    const Config anchor = physical_multistep_config();
    AcousticPolicyCase result;
    result.compression_factor = compression_factor;
    result.kappa_factor = kappa_factor;
    const double kappa = anchor.kappa * kappa_factor;
    result.wave_speed = std::sqrt(kappa / anchor.mass);
    result.policy_substeps = acoustic_substep_count(
        frame_time, anchor.spacing, kappa, anchor.mass);
    std::vector<Vec3> initial = centered_lattice(7, anchor.spacing);
    for (Vec3& value : initial) {
        value = compression_factor * value;
    }
    result.initial_maximum_density_ratio =
        maximum_density_ratio(anchor, initial);
    for (std::size_t i = 0; i < result.levels.size(); ++i) {
        result.levels[i] = make_policy_temporal_level(
            compression_factor, kappa_factor,
            result.policy_substeps * (1 << i));
    }
    for (std::size_t i = 0; i < result.position_difference.size(); ++i) {
        result.position_difference[i] = mass_weighted_rms_position_error(
            result.levels[i].run.position,
            result.levels[i + 1].run.position);
        result.velocity_difference[i] = mass_weighted_rms_position_error(
            result.levels[i].run.velocity,
            result.levels[i + 1].run.velocity);
    }
    result.position_ratio = result.position_difference[0]
        / result.position_difference[1];
    result.velocity_ratio = result.velocity_difference[0]
        / result.velocity_difference[1];
    result.normalized_policy_position_error =
        result.position_difference[0] / anchor.spacing;
    result.normalized_policy_velocity_error =
        result.velocity_difference[0] / result.wave_speed;
    result.relative_kinetic_error = std::abs(
        result.levels[0].kinetic_energy
            - result.levels[1].kinetic_energy)
        / std::max(std::abs(result.levels[1].kinetic_energy), 1.0e-30);
    result.pressure_exit_time_error = std::abs(
        result.levels[0].pressure_exit_time
            - result.levels[1].pressure_exit_time);
    bool levels_valid = true;
    for (const TemporalLevel& level : result.levels) {
        levels_valid = levels_valid && temporal_level_valid(
            level, result.initial_maximum_density_ratio);
    }
    result.passed = levels_valid && result.policy_substeps <= 96
        && result.position_difference[0] > 0.0
        && result.position_difference[1] > 0.0
        && result.velocity_difference[0] > 0.0
        && result.velocity_difference[1] > 0.0
        && std::isfinite(result.position_ratio)
        && std::isfinite(result.velocity_ratio)
        && result.position_ratio >= 1.5 && result.position_ratio <= 2.5
        && result.velocity_ratio >= 1.5 && result.velocity_ratio <= 2.5
        && result.normalized_policy_position_error <= 0.05
        && result.normalized_policy_velocity_error <= 0.001
        && result.relative_kinetic_error <= 0.15
        && result.pressure_exit_time_error
            <= result.levels[0].time_step;
    return result;
}

struct AcousticDegenerateControl {
    int free_flight_substeps = 0;
    int translation_substeps = 0;
    MultistepRun free_flight;
    MultistepRun translation;
    double free_position_error = 0.0;
    double free_velocity_error = 0.0;
    double translation_position_error = 0.0;
    double translation_velocity_error = 0.0;
    bool passed = false;
};

AcousticDegenerateControl acoustic_degenerate_control() {
    constexpr double frame_time = 1.0 / 240.0;
    Config config = physical_multistep_config(frame_time);
    config.kappa = 0.0;
    AcousticDegenerateControl result;
    result.free_flight_substeps = acoustic_substep_count(
        frame_time, config.spacing, config.kappa, config.mass);
    result.translation_substeps = result.free_flight_substeps;
    const Vec3 initial_position = {0.25, -0.1, 0.4};
    const Vec3 initial_velocity = {1.2, 0.7, -0.35};
    config.gravity = {0.0, -9.81, 0.0};
    result.free_flight = run_multistep(config,
        {initial_position}, {initial_velocity}, 1, false);
    const Vec3 expected_velocity =
        initial_velocity + frame_time * config.gravity;
    const Vec3 expected_position = initial_position
        + frame_time * expected_velocity;
    result.free_position_error = vector_relative_error(
        result.free_flight.position[0], expected_position);
    result.free_velocity_error = vector_relative_error(
        result.free_flight.velocity[0], expected_velocity);

    config.gravity = {};
    const std::vector<Vec3> initial =
        centered_lattice(4, config.spacing);
    const Vec3 translation_velocity = {0.37, -0.21, 0.13};
    result.translation = run_multistep(config, initial,
        std::vector<Vec3>(initial.size(), translation_velocity), 1, false);
    for (std::size_t i = 0; i < initial.size(); ++i) {
        result.translation_position_error = std::max(
            result.translation_position_error,
            norm(result.translation.position[i]
                - (initial[i] + frame_time * translation_velocity)));
        result.translation_velocity_error = std::max(
            result.translation_velocity_error,
            norm(result.translation.velocity[i] - translation_velocity));
    }
    result.passed = result.free_flight_substeps == 1
        && result.translation_substeps == 1
        && result.free_flight.completed && result.translation.completed
        && result.free_flight.total_hvp_calls == 0
        && result.translation.total_hvp_calls == 0
        && result.free_flight.maximum_material_energy == 0.0
        && result.translation.maximum_material_energy <= 1.0e-12
        && result.free_position_error <= 1.0e-11
        && result.free_velocity_error <= 1.0e-11
        && result.translation_position_error <= 1.0e-11
        && result.translation_velocity_error <= 1.0e-11;
    return result;
}

struct AcousticSubstepPolicyDiagnostic {
    std::array<AcousticPolicyCase, 6> cases;
    AcousticDegenerateControl degenerate;
    bool passed = false;
    std::string first_failure;
};

AcousticSubstepPolicyDiagnostic acoustic_substep_policy_diagnostic() {
    AcousticSubstepPolicyDiagnostic result;
    const std::array<double, 2> compression = {0.99, 0.98};
    const std::array<double, 3> stiffness = {0.25, 1.0, 4.0};
    std::size_t index = 0;
    for (double compression_factor : compression) {
        for (double kappa_factor : stiffness) {
            result.cases[index++] = make_acoustic_policy_case(
                compression_factor, kappa_factor);
        }
    }
    result.degenerate = acoustic_degenerate_control();
    result.passed = result.degenerate.passed;
    if (!result.degenerate.passed) {
        result.first_failure = "NSR3B1S_DEGENERATE";
    }
    for (const AcousticPolicyCase& value : result.cases) {
        result.passed = result.passed && value.passed;
        if (!value.passed && result.first_failure.empty()) {
            std::ostringstream name;
            name << std::setprecision(17)
                 << "NSR3B1S_MATRIX:c=" << value.compression_factor
                 << ":k=" << value.kappa_factor;
            result.first_failure = name.str();
        }
    }
    return result;
}

bool jacobi_eigensystem(
    const std::vector<double>& matrix,
    std::size_t dimension,
    std::vector<double>& eigenvalues,
    std::vector<double>& eigenvectors,
    int& pivots) {
    std::vector<double> value(matrix.size());
    eigenvectors.assign(dimension * dimension, 0.0);
    for (std::size_t row = 0; row < dimension; ++row) {
        eigenvectors[row * dimension + row] = 1.0;
        for (std::size_t column = 0; column < dimension; ++column) {
            value[row * dimension + column] = 0.5
                * (matrix[row * dimension + column]
                    + matrix[column * dimension + row]);
        }
    }
    const int maximum_pivots = static_cast<int>(64 * dimension * dimension);
    pivots = 0;
    for (; pivots < maximum_pivots; ++pivots) {
        std::size_t p = 0;
        std::size_t q = 0;
        double largest = 0.0;
        for (std::size_t row = 0; row < dimension; ++row) {
            for (std::size_t column = row + 1;
                 column < dimension; ++column) {
                const double candidate =
                    std::abs(value[row * dimension + column]);
                if (candidate > largest) {
                    largest = candidate;
                    p = row;
                    q = column;
                }
            }
        }
        double maximum_diagonal = 0.0;
        for (std::size_t i = 0; i < dimension; ++i) {
            maximum_diagonal = std::max(
                maximum_diagonal, std::abs(value[i * dimension + i]));
        }
        if (largest <= 1.0e-12 * std::max(maximum_diagonal, 1.0)) {
            std::vector<std::size_t> order(dimension);
            for (std::size_t i = 0; i < dimension; ++i) {
                order[i] = i;
            }
            std::sort(order.begin(), order.end(),
                [&](std::size_t lhs, std::size_t rhs) {
                    return value[lhs * dimension + lhs]
                        < value[rhs * dimension + rhs];
                });
            eigenvalues.resize(dimension);
            std::vector<double> sorted_vectors(
                dimension * dimension);
            for (std::size_t column = 0; column < dimension; ++column) {
                eigenvalues[column] = value[
                    order[column] * dimension + order[column]];
                for (std::size_t row = 0; row < dimension; ++row) {
                    sorted_vectors[row * dimension + column] =
                        eigenvectors[row * dimension + order[column]];
                }
            }
            eigenvectors = std::move(sorted_vectors);
            return all_finite(eigenvalues) && all_finite(eigenvectors);
        }
        const double app = value[p * dimension + p];
        const double aqq = value[q * dimension + q];
        const double apq = value[p * dimension + q];
        const double tau = (aqq - app) / (2.0 * apq);
        const double tangent = tau >= 0.0
            ? 1.0 / (tau + std::sqrt(1.0 + tau * tau))
            : -1.0 / (-tau + std::sqrt(1.0 + tau * tau));
        const double cosine = 1.0 / std::sqrt(1.0 + tangent * tangent);
        const double sine = tangent * cosine;
        for (std::size_t k = 0; k < dimension; ++k) {
            if (k != p && k != q) {
                const double akp = value[k * dimension + p];
                const double akq = value[k * dimension + q];
                const double next_kp = cosine * akp - sine * akq;
                const double next_kq = sine * akp + cosine * akq;
                value[k * dimension + p] = next_kp;
                value[p * dimension + k] = next_kp;
                value[k * dimension + q] = next_kq;
                value[q * dimension + k] = next_kq;
            }
            const double vector_kp =
                eigenvectors[k * dimension + p];
            const double vector_kq =
                eigenvectors[k * dimension + q];
            eigenvectors[k * dimension + p] =
                cosine * vector_kp - sine * vector_kq;
            eigenvectors[k * dimension + q] =
                sine * vector_kp + cosine * vector_kq;
        }
        value[p * dimension + p] = cosine * cosine * app
            - 2.0 * sine * cosine * apq + sine * sine * aqq;
        value[q * dimension + q] = sine * sine * app
            + 2.0 * sine * cosine * apq + cosine * cosine * aqq;
        value[p * dimension + q] = 0.0;
        value[q * dimension + p] = 0.0;
    }
    return false;
}

struct PressureTangentOperator {
    Config config;
    std::vector<Vec3> position;
    std::vector<ParticlePair> pairs;
    std::vector<std::vector<std::size_t>> adjacency;
    NeighborhoodHessianTape tape;
    NeighborhoodHvpWorkspace workspace;
    int calls = 0;
    bool valid = false;
};

PressureTangentOperator make_pressure_tangent_operator(
    Config config, std::vector<Vec3> position) {
    config.lambda = 0.0;
    config.mu = 0.0;
    config.gamma = 0.0;
    PressureTangentOperator result;
    result.config = config;
    result.position = std::move(position);
    result.pairs = build_cell_pairs(
        result.position, result.config.horizon);
    result.adjacency = build_pair_adjacency(
        result.position.size(), result.pairs);
    build_reference_hessian_tape(result.config, result.position,
        result.pairs, result.tape);
    result.valid = build_current_hessian_tape(result.config,
        result.position, result.pairs, result.adjacency, result.tape)
        && neighborhood_capacity_valid(
            result.position.size(), result.pairs, result.adjacency)
        && hessian_tape_storage_bytes(result.tape)
            <= hessian_tape_storage_limit(
                result.position.size(), result.pairs.size());
    return result;
}

std::vector<Vec3> apply_pressure_tangent(
    PressureTangentOperator& pressure,
    const std::vector<Vec3>& direction) {
    const std::vector<Vec3>& full = apply_hessian_with_tape(
        pressure.config, direction, pressure.adjacency,
        pressure.tape, pressure.workspace);
    std::vector<Vec3> result = full;
    for (std::size_t i = 0; i < result.size(); ++i) {
        result[i] += -pressure.tape.inertia_scale * direction[i];
    }
    ++pressure.calls;
    return result;
}

std::vector<Vec3> deterministic_lanczos_start(std::size_t particles) {
    std::vector<Vec3> result(particles);
    for (std::size_t particle = 0; particle < particles; ++particle) {
        double* component[3] = {
            &result[particle].x,
            &result[particle].y,
            &result[particle].z,
        };
        for (std::size_t axis = 0; axis < 3; ++axis) {
            const double r = static_cast<double>(3 * particle + axis + 1);
            *component[axis] = std::sin(r * std::sqrt(2.0))
                + std::cos(r * std::sqrt(3.0));
        }
    }
    const Vec3 translation = average(result);
    for (Vec3& value : result) {
        value = value - translation;
    }
    const double scale = vector_norm(result);
    for (Vec3& value : result) {
        value = value / scale;
    }
    return result;
}

struct LanczosMaximumEigenvalue {
    bool finite = false;
    bool jacobi_converged = false;
    int iterations = 0;
    int operator_calls = 0;
    int jacobi_pivots = 0;
    double eigenvalue = 0.0;
    double relative_ritz_residual = 0.0;
    double orthogonality_error = 0.0;
};

LanczosMaximumEigenvalue lanczos_maximum_eigenvalue(
    PressureTangentOperator& pressure, int maximum_iterations) {
    LanczosMaximumEigenvalue result;
    std::vector<std::vector<Vec3>> basis;
    basis.push_back(deterministic_lanczos_start(
        pressure.position.size()));
    std::vector<Vec3> previous(pressure.position.size());
    std::vector<double> alpha;
    std::vector<double> beta;
    double previous_beta = 0.0;
    for (int iteration = 0; iteration < maximum_iterations; ++iteration) {
        const std::vector<Vec3>& current = basis.back();
        std::vector<Vec3> residual =
            apply_pressure_tangent(pressure, current);
        if (iteration != 0) {
            for (std::size_t i = 0; i < residual.size(); ++i) {
                residual[i] += -previous_beta * previous[i];
            }
        }
        const double diagonal = vector_dot(current, residual);
        for (std::size_t i = 0; i < residual.size(); ++i) {
            residual[i] += -diagonal * current[i];
        }
        for (int pass = 0; pass < 2; ++pass) {
            for (const std::vector<Vec3>& vector : basis) {
                const double projection = vector_dot(vector, residual);
                for (std::size_t i = 0; i < residual.size(); ++i) {
                    residual[i] += -projection * vector[i];
                }
            }
        }
        const double next_beta = vector_norm(residual);
        alpha.push_back(diagonal);
        beta.push_back(next_beta);
        result.iterations = iteration + 1;
        if (!std::isfinite(diagonal) || !std::isfinite(next_beta)
            || !all_finite(residual)) {
            result.operator_calls = pressure.calls;
            return result;
        }
        if (next_beta <= 1.0e-14
            || iteration + 1 == maximum_iterations) {
            break;
        }
        previous = current;
        previous_beta = next_beta;
        for (Vec3& value : residual) {
            value = value / next_beta;
        }
        basis.push_back(std::move(residual));
    }
    const std::size_t dimension = alpha.size();
    std::vector<double> tridiagonal(dimension * dimension);
    for (std::size_t i = 0; i < dimension; ++i) {
        tridiagonal[i * dimension + i] = alpha[i];
        if (i + 1 < dimension) {
            tridiagonal[i * dimension + i + 1] = beta[i];
            tridiagonal[(i + 1) * dimension + i] = beta[i];
        }
    }
    std::vector<double> eigenvalues;
    std::vector<double> eigenvectors;
    result.jacobi_converged = jacobi_eigensystem(
        tridiagonal, dimension, eigenvalues, eigenvectors,
        result.jacobi_pivots);
    if (result.jacobi_converged && !eigenvalues.empty()) {
        result.eigenvalue = eigenvalues.back();
        const double last_component =
            eigenvectors[(dimension - 1) * dimension + dimension - 1];
        result.relative_ritz_residual =
            std::abs(beta.back() * last_component)
            / std::max(std::abs(result.eigenvalue), 1.0);
    }
    for (std::size_t i = 0; i < basis.size(); ++i) {
        for (std::size_t j = 0; j < basis.size(); ++j) {
            const double target = i == j ? 1.0 : 0.0;
            result.orthogonality_error = std::max(
                result.orthogonality_error,
                std::abs(vector_dot(basis[i], basis[j]) - target));
        }
    }
    result.operator_calls = pressure.calls;
    result.finite = result.jacobi_converged
        && std::isfinite(result.eigenvalue)
        && std::isfinite(result.relative_ritz_residual)
        && std::isfinite(result.orthogonality_error);
    return result;
}

struct PressureDenseOracle {
    bool passed = false;
    std::size_t dimension = 0;
    int active_pressure_centers = 0;
    double dense_maximum_eigenvalue = 0.0;
    double lanczos_maximum_eigenvalue = 0.0;
    double eigenvalue_error = 0.0;
    double symmetry_error = 0.0;
    double dense_product_error = 0.0;
    LanczosMaximumEigenvalue lanczos;
};

PressureDenseOracle pressure_dense_oracle() {
    const CombinedFixture source = combined_fixture();
    Config config = physical_multistep_config();
    config.kappa = 200.0;
    double center_density = config.mass
        * configured_cubic_weight(config, 0.0);
    for (std::size_t i = 1; i < source.x.size(); ++i) {
        center_density += config.mass * configured_cubic_weight(
            config, norm(source.x[0] - source.x[i]));
    }
    config.rest_density = center_density / 1.1;
    PressureTangentOperator dense_operator =
        make_pressure_tangent_operator(config, source.x);
    PressureDenseOracle result;
    result.dimension = 3 * source.x.size();
    result.active_pressure_centers =
        pressure_active_count(config, source.x);
    std::vector<double> dense(result.dimension * result.dimension);
    for (std::size_t column = 0; column < result.dimension; ++column) {
        std::vector<double> basis(result.dimension);
        basis[column] = 1.0;
        const std::vector<double> image = flatten(
            apply_pressure_tangent(dense_operator, unflatten(basis)));
        for (std::size_t row = 0; row < result.dimension; ++row) {
            dense[row * result.dimension + column] = image[row];
        }
    }
    double asymmetry_squared = 0.0;
    double dense_squared = 0.0;
    for (std::size_t row = 0; row < result.dimension; ++row) {
        for (std::size_t column = 0; column < result.dimension; ++column) {
            const double value = dense[row * result.dimension + column];
            const double difference = value
                - dense[column * result.dimension + row];
            asymmetry_squared += difference * difference;
            dense_squared += value * value;
        }
    }
    result.symmetry_error = std::sqrt(asymmetry_squared)
        / std::max(std::sqrt(dense_squared), 1.0);
    std::vector<double> eigenvalues;
    std::vector<double> eigenvectors;
    int pivots = 0;
    const bool dense_converged = jacobi_eigensystem(
        dense, result.dimension, eigenvalues, eigenvectors, pivots);
    if (!eigenvalues.empty()) {
        result.dense_maximum_eigenvalue = eigenvalues.back();
    }
    PressureTangentOperator lanczos_operator =
        make_pressure_tangent_operator(config, source.x);
    result.lanczos = lanczos_maximum_eigenvalue(
        lanczos_operator, static_cast<int>(result.dimension));
    result.lanczos_maximum_eigenvalue = result.lanczos.eigenvalue;
    result.eigenvalue_error = relative_error(
        result.dense_maximum_eigenvalue,
        result.lanczos_maximum_eigenvalue);
    const std::vector<Vec3> direction = deterministic_lanczos_start(
        source.x.size());
    const std::vector<double> flat_direction = flatten(direction);
    const std::vector<double> operator_product = flatten(
        apply_pressure_tangent(dense_operator, direction));
    std::vector<double> dense_product(result.dimension);
    for (std::size_t row = 0; row < result.dimension; ++row) {
        for (std::size_t column = 0; column < result.dimension; ++column) {
            dense_product[row] += dense[row * result.dimension + column]
                * flat_direction[column];
        }
    }
    std::vector<double> difference(result.dimension);
    for (std::size_t i = 0; i < result.dimension; ++i) {
        difference[i] = dense_product[i] - operator_product[i];
    }
    result.dense_product_error = flat_norm(difference)
        / std::max({flat_norm(dense_product),
            flat_norm(operator_product), 1.0});
    result.passed = dense_operator.valid && lanczos_operator.valid
        && dense_converged && result.lanczos.finite
        && result.active_pressure_centers > 0
        && result.symmetry_error <= 2.0e-12
        && result.dense_product_error <= 2.0e-12
        && result.eigenvalue_error <= 1.0e-10;
    return result;
}

struct PressureSpectrumCase {
    double compression_factor = 0.0;
    double kappa_factor = 0.0;
    int particles = 0;
    int active_pressure_centers = 0;
    std::size_t pairs = 0;
    std::size_t maximum_neighbors = 0;
    double maximum_eigenvalue = 0.0;
    double maximum_eigenfrequency = 0.0;
    double spectral_amplification = 0.0;
    double translation_null_residual = 0.0;
    LanczosMaximumEigenvalue lanczos;
    bool passed = false;
};

PressureSpectrumCase make_pressure_spectrum_case(
    double compression_factor, double kappa_factor) {
    Config config = physical_multistep_config();
    config.kappa *= kappa_factor;
    std::vector<Vec3> position = centered_lattice(7, config.spacing);
    for (Vec3& value : position) {
        value = compression_factor * value;
    }
    PressureTangentOperator pressure =
        make_pressure_tangent_operator(config, position);
    PressureSpectrumCase result;
    result.compression_factor = compression_factor;
    result.kappa_factor = kappa_factor;
    result.particles = static_cast<int>(position.size());
    result.active_pressure_centers =
        pressure_active_count(config, position);
    result.pairs = pressure.pairs.size();
    result.maximum_neighbors = maximum_neighbor_count(pressure.adjacency);
    std::vector<Vec3> translation(position.size(), {1.0, -0.5, 0.25});
    const std::vector<Vec3> translation_image =
        apply_pressure_tangent(pressure, translation);
    const double translation_image_norm = vector_norm(translation_image);
    const int calls_before = pressure.calls;
    result.lanczos = lanczos_maximum_eigenvalue(pressure, 48);
    result.lanczos.operator_calls = pressure.calls - calls_before;
    result.maximum_eigenvalue = result.lanczos.eigenvalue;
    result.translation_null_residual = translation_image_norm
        / (std::max(std::abs(result.maximum_eigenvalue), 1.0)
            * std::max(vector_norm(translation), 1.0));
    result.maximum_eigenfrequency = std::sqrt(
        std::max(result.maximum_eigenvalue, 0.0) / config.mass);
    result.spectral_amplification = config.spacing * std::sqrt(
        std::max(result.maximum_eigenvalue, 0.0) / config.kappa);
    result.passed = pressure.valid && result.active_pressure_centers > 0
        && result.pairs <= 80 * position.size()
        && result.maximum_neighbors <= 160
        && result.lanczos.finite
        && result.maximum_eigenvalue > 0.0
        && result.lanczos.relative_ritz_residual <= 1.0e-8
        && result.lanczos.orthogonality_error <= 1.0e-10
        && result.translation_null_residual <= 1.0e-12;
    return result;
}

struct PressureSpectrumNullControl {
    int active_pressure_centers = 0;
    double maximum_eigenvalue = 0.0;
    double hvp_norm = 0.0;
    LanczosMaximumEigenvalue lanczos;
    bool passed = false;
};

PressureSpectrumNullControl pressure_spectrum_null_control() {
    Config config = physical_multistep_config();
    const std::vector<Vec3> position =
        centered_lattice(7, config.spacing);
    PressureTangentOperator pressure =
        make_pressure_tangent_operator(config, position);
    PressureSpectrumNullControl result;
    result.active_pressure_centers =
        pressure_active_count(config, position);
    const std::vector<Vec3> direction =
        deterministic_lanczos_start(position.size());
    result.hvp_norm = vector_norm(
        apply_pressure_tangent(pressure, direction));
    result.lanczos = lanczos_maximum_eigenvalue(pressure, 48);
    result.maximum_eigenvalue = result.lanczos.eigenvalue;
    result.passed = pressure.valid && result.active_pressure_centers == 0
        && result.hvp_norm <= 1.0e-12
        && std::abs(result.maximum_eigenvalue) <= 1.0e-12
        && result.lanczos.finite;
    return result;
}

struct PressureSpectrumDiagnostic {
    PressureDenseOracle dense;
    std::array<PressureSpectrumCase, 6> cases;
    PressureSpectrumNullControl null_control;
    double amplification_099 = 0.0;
    double amplification_098 = 0.0;
    bool kappa_scaling_valid = false;
    bool passed = false;
    std::string first_failure;
};

PressureSpectrumDiagnostic pressure_spectrum_diagnostic() {
    PressureSpectrumDiagnostic result;
    result.dense = pressure_dense_oracle();
    const std::array<double, 2> compression = {0.99, 0.98};
    const std::array<double, 3> stiffness = {0.25, 1.0, 4.0};
    std::size_t index = 0;
    for (double compression_factor : compression) {
        for (double kappa_factor : stiffness) {
            result.cases[index++] = make_pressure_spectrum_case(
                compression_factor, kappa_factor);
        }
    }
    result.null_control = pressure_spectrum_null_control();
    result.kappa_scaling_valid = true;
    for (std::size_t amplitude = 0; amplitude < 2; ++amplitude) {
        const std::size_t base = 3 * amplitude;
        for (std::size_t i = 0; i < 2; ++i) {
            result.kappa_scaling_valid = result.kappa_scaling_valid
                && relative_error(
                    result.cases[base + i + 1].maximum_eigenvalue,
                    4.0 * result.cases[base + i].maximum_eigenvalue)
                    <= 1.0e-10
                && relative_error(
                    result.cases[base + i + 1].maximum_eigenfrequency,
                    2.0 * result.cases[base + i].maximum_eigenfrequency)
                    <= 1.0e-10
                && relative_error(
                    result.cases[base + i + 1].spectral_amplification,
                    result.cases[base + i].spectral_amplification)
                    <= 1.0e-10;
        }
    }
    for (std::size_t i = 0; i < 3; ++i) {
        result.amplification_099 +=
            result.cases[i].spectral_amplification / 3.0;
        result.amplification_098 +=
            result.cases[i + 3].spectral_amplification / 3.0;
    }
    result.passed = result.dense.passed && result.null_control.passed
        && result.kappa_scaling_valid
        && result.amplification_098 > result.amplification_099;
    if (!result.dense.passed) {
        result.first_failure = "NSR3B1S1_DENSE_ORACLE";
    }
    for (const PressureSpectrumCase& value : result.cases) {
        result.passed = result.passed && value.passed;
        if (!value.passed && result.first_failure.empty()) {
            std::ostringstream name;
            name << std::setprecision(17)
                 << "NSR3B1S1_MATRIX:c=" << value.compression_factor
                 << ":k=" << value.kappa_factor;
            result.first_failure = name.str();
        }
    }
    if (!result.null_control.passed && result.first_failure.empty()) {
        result.first_failure = "NSR3B1S1_NULL_CONTROL";
    } else if (!result.kappa_scaling_valid
        && result.first_failure.empty()) {
        result.first_failure = "NSR3B1S1_KAPPA_SCALING";
    } else if (result.amplification_098 <= result.amplification_099
        && result.first_failure.empty()) {
        result.first_failure = "NSR3B1S1_AMPLITUDE_ORDER";
    }
    return result;
}

void append_double_array(
    std::ostringstream& output, const double* begin, std::size_t size) {
    output << '[';
    for (std::size_t i = 0; i < size; ++i) {
        if (i != 0) {
            output << ',';
        }
        output << begin[i];
    }
    output << ']';
}

void append_multistep_work(
    std::ostringstream& output, const MultistepRun& value) {
    output << "{\"completed\":" << (value.completed ? "true" : "false")
           << ",\"solver_valid\":"
           << (value.solver_valid ? "true" : "false")
           << ",\"positive_reductions\":"
           << (value.positive_reductions ? "true" : "false")
           << ",\"rejected_state_immutable\":"
           << (value.rejected_state_immutable ? "true" : "false")
           << ",\"capacity_valid\":"
           << (value.capacity_valid ? "true" : "false")
           << ",\"failure\":\"" << value.failure << '"'
           << ",\"requested_steps\":" << value.requested_steps
           << ",\"completed_steps\":" << value.completed_steps
           << ",\"total_outer_trials\":" << value.total_outer_trials
           << ",\"total_accepted_trials\":"
           << value.total_accepted_trials
           << ",\"total_rejected_trials\":"
           << value.total_rejected_trials
           << ",\"total_objective_evaluations\":"
           << value.total_objective_evaluations
           << ",\"total_hvp_calls\":" << value.total_hvp_calls
           << ",\"total_pair_builds\":" << value.total_pair_builds
           << ",\"maximum_outer_trials\":"
           << value.maximum_outer_trials
           << ",\"maximum_rejected_trials\":"
           << value.maximum_rejected_trials
           << ",\"maximum_hvp_calls\":" << value.maximum_hvp_calls
           << ",\"initial_active_pressure_centers\":"
           << value.initial_active_pressure_centers
           << ",\"final_active_pressure_centers\":"
           << value.final_active_pressure_centers
           << ",\"maximum_active_pressure_centers\":"
           << value.maximum_active_pressure_centers
           << ",\"maximum_pairs\":" << value.maximum_pairs
           << ",\"maximum_neighbors\":" << value.maximum_neighbors
           << ",\"maximum_hessian_tape_bytes\":"
           << value.maximum_hessian_tape_bytes
           << ",\"total_mass\":" << value.total_mass
           << ",\"maximum_density_ratio\":"
           << value.maximum_density_ratio
           << ",\"maximum_material_energy\":"
           << value.maximum_material_energy
           << ",\"accumulated_momentum_residual\":"
           << value.accumulated_momentum_residual
           << ",\"state_sha256\":\""
           << hash_phase_state(value.position, value.velocity) << "\"}";
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
    int candidate_backtracks = 0;
    for (ConditioningCase& value : cases) {
        value.passed = conditioning_quality_passed(value);
        baseline_backtracks += value.baseline.backtracks;
        candidate_backtracks += value.candidate.backtracks;
    }
    const double compression_alpha_ratio = cases[0].candidate.minimum_alpha
        / std::max(cases[0].baseline.minimum_alpha, 1.0e-300);
    const double combined_alpha_ratio = cases[2].candidate.minimum_alpha
        / std::max(cases[2].baseline.minimum_alpha, 1.0e-300);
    const bool aggregate_backtracks_passed =
        4LL * static_cast<long long>(candidate_backtracks)
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
           << "{\"schema\":\"nextengine.nonlocal.formula_reclosure_fcr3a.v2\""
           << ",\"identity\":\"nuv-variational-fcr1\""
           << ",\"candidate\":\"block16-inertial64-warm-armijo-v2\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"thresholds\":{\"objective_relative\":1e-10"
           << ",\"gradient_ratio\":2,\"backtrack_reduction\":4"
           << ",\"minimum_alpha_improvement\":16}"
           << ",\"aggregate\":{\"baseline_backtracks\":"
           << baseline_backtracks
           << ",\"candidate_backtracks\":" << candidate_backtracks
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
        + std::to_string(candidate_backtracks) + '|'
        + std::to_string(compression_alpha_ratio) + '|'
        + std::to_string(combined_alpha_ratio);
    report << ",\"result_sha256\":\"" << sha256_hex(result_material) << "\"}";
    return {passed, report.str()};
}

ReferenceSolverReport run_sissm_controls() {
    std::array<SissmCase, 6> cases = {
        make_sissm_compression_case(),
        make_sissm_viscosity_case(false),
        make_sissm_viscosity_case(true),
        make_sissm_surface_case(false),
        make_sissm_surface_case(true),
        make_sissm_combined_case(),
    };
    for (SissmCase& value : cases) {
        value.passed = sissm_quality_passed(value);
    }
    const std::array<std::size_t, 3> stiff_indices = {0, 3, 5};
    int baseline_evaluations = 0;
    int candidate_evaluations = 0;
    for (std::size_t index : stiff_indices) {
        baseline_evaluations +=
            cases[index].baseline.iterations + cases[index].baseline.backtracks;
        candidate_evaluations +=
            cases[index].candidate.iterations + cases[index].candidate.backtracks;
    }
    const bool evaluation_gate =
        4LL * static_cast<long long>(candidate_evaluations)
        <= static_cast<long long>(baseline_evaluations);
    std::string first_failure;
    for (const SissmCase& value : cases) {
        if (!value.passed && first_failure.empty()) {
            first_failure = value.candidate.failure.empty()
                ? "FCR3B_SISSM_QUALITY_FAILED:" + value.name
                : "FCR3B_" + value.candidate.failure + ':' + value.name;
        }
    }
    if (first_failure.empty() && !evaluation_gate) {
        first_failure = "FCR3B_OBJECTIVE_EVALUATION_REDUCTION_FAILED";
    }
    const bool passed = first_failure.empty();

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.formula_reclosure_fcr3b.v1\""
           << ",\"identity\":\"nuv-variational-fcr1\""
           << ",\"candidate\":\"corrected-sissm-armijo-v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"thresholds\":{\"objective_relative\":1e-10"
           << ",\"gradient_ratio\":2,\"evaluation_reduction\":4}"
           << ",\"aggregate\":{\"baseline_stiff_evaluations\":"
           << baseline_evaluations
           << ",\"candidate_stiff_evaluations\":" << candidate_evaluations
           << ",\"evaluation_gate\":\""
           << (evaluation_gate ? "PASS" : "FAIL") << "\"},\"cases\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_sissm_case(report, cases[i]);
    }
    report << "]"
           << ",\"chebyshev_ab_authorized\":" << (passed ? "true" : "false")
           << ",\"profile_reclosure_authorized\":false"
           << ",\"runtime_authority\":false";
    std::string result_material = std::string(passed ? "PASS|" : "FAIL|")
        + first_failure + '|' + std::to_string(baseline_evaluations) + '|'
        + std::to_string(candidate_evaluations);
    for (const SissmCase& value : cases) {
        result_material += '|' + value.name + ':'
            + std::to_string(value.candidate.final.total) + ':'
            + std::to_string(value.candidate.final.gradient_norm);
    }
    report << ",\"result_sha256\":\"" << sha256_hex(result_material) << "\"}";
    return {passed, report.str()};
}

ReferenceSolverReport run_sissm_term_local_controls() {
    const std::array<std::string, 7> masks = {
        "P", "V", "S", "PV", "PS", "VS", "PVS",
    };
    std::array<SissmCase, 7> cases;
    for (std::size_t i = 0; i < masks.size(); ++i) {
        cases[i] = make_sissm_mask_case(masks[i]);
        cases[i].passed = sissm_quality_passed(cases[i]);
    }

    std::string selected_scope;
    std::string first_failing_mask;
    for (std::size_t i = 0; i < 3; ++i) {
        if (!cases[i].passed) {
            first_failing_mask = masks[i];
            selected_scope = "isolated-" + masks[i];
            break;
        }
    }
    if (selected_scope.empty()) {
        for (std::size_t i = 3; i < 6; ++i) {
            if (!cases[i].passed) {
                first_failing_mask = masks[i];
                selected_scope = "pairwise-" + masks[i];
                break;
            }
        }
    }
    if (selected_scope.empty() && !cases[6].passed) {
        first_failing_mask = "PVS";
        selected_scope = "three-way-composition";
    }
    const bool known_failure_reproduced = !cases[6].passed;
    const bool passed = known_failure_reproduced && !selected_scope.empty();
    const std::string first_failure = passed
        ? std::string()
        : "FCR3B1_EXPECTED_COUPLING_FAILURE_NOT_LOCALIZED";

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.formula_reclosure_fcr3b1.v1\""
           << ",\"identity\":\"nuv-variational-fcr1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"selected_scope\":\"" << selected_scope << '"'
           << ",\"first_failing_mask\":\"" << first_failing_mask << '"'
           << ",\"known_pvs_failure_reproduced\":"
           << (known_failure_reproduced ? "true" : "false")
           << ",\"cases\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_sissm_case(report, cases[i]);
    }
    report << "]"
           << ",\"term_local_remediation_authorized\":"
           << (passed ? "true" : "false")
           << ",\"chebyshev_ab_authorized\":false"
           << ",\"profile_reclosure_authorized\":false"
           << ",\"runtime_authority\":false";
    std::string result_material = std::string(passed ? "PASS|" : "FAIL|")
        + selected_scope + '|' + first_failing_mask;
    for (const SissmCase& value : cases) {
        result_material += '|' + value.name + ':'
            + (value.passed ? "PASS" : "FAIL") + ':'
            + std::to_string(value.candidate.final.gradient_norm);
    }
    report << ",\"result_sha256\":\"" << sha256_hex(result_material) << "\"}";
    return {passed, report.str()};
}

ReferenceSolverReport run_sissm_pressure_chebyshev_controls() {
    constexpr SissmAcceleration acceleration =
        SissmAcceleration::PressureChebyshev;
    std::array<SissmCase, 6> cases = {
        make_sissm_compression_case(acceleration),
        make_sissm_viscosity_case(false, acceleration),
        make_sissm_viscosity_case(true, acceleration),
        make_sissm_surface_case(false, acceleration),
        make_sissm_surface_case(true, acceleration),
        make_sissm_combined_case(acceleration),
    };
    for (SissmCase& value : cases) {
        value.passed = sissm_quality_passed(value);
    }

    const std::array<std::string, 7> masks = {
        "P", "V", "S", "PV", "PS", "VS", "PVS",
    };
    std::array<SissmCase, 7> mask_cases;
    for (std::size_t i = 0; i < masks.size(); ++i) {
        mask_cases[i] = make_sissm_mask_case(masks[i], acceleration);
        mask_cases[i].passed = sissm_quality_passed(mask_cases[i]);
    }

    const std::array<std::size_t, 3> nonpressure_indices = {1, 2, 5};
    bool nonpressure_unchanged = true;
    for (std::size_t index : nonpressure_indices) {
        const SissmCase v1 = make_sissm_mask_case(masks[index]);
        nonpressure_unchanged = nonpressure_unchanged
            && same_sissm_candidate(mask_cases[index], v1);
    }

    const std::array<std::size_t, 3> stiff_indices = {0, 3, 5};
    int baseline_evaluations = 0;
    int candidate_evaluations = 0;
    for (std::size_t index : stiff_indices) {
        baseline_evaluations +=
            cases[index].baseline.iterations + cases[index].baseline.backtracks;
        candidate_evaluations +=
            cases[index].candidate.iterations + cases[index].candidate.backtracks;
    }

    std::string first_failure;
    for (const SissmCase& value : cases) {
        if (!value.passed && first_failure.empty()) {
            first_failure = value.candidate.failure.empty()
                ? "FCR3B2_CASE_QUALITY_FAILED:" + value.name
                : "FCR3B2_" + value.candidate.failure + ':' + value.name;
        }
    }
    for (const SissmCase& value : mask_cases) {
        if (!value.passed && first_failure.empty()) {
            first_failure = value.candidate.failure.empty()
                ? "FCR3B2_MASK_QUALITY_FAILED:" + value.name
                : "FCR3B2_" + value.candidate.failure + ':' + value.name;
        }
    }
    if (first_failure.empty() && !nonpressure_unchanged) {
        first_failure = "FCR3B2_NONPRESSURE_PATH_CHANGED";
    }
    const bool passed = first_failure.empty();

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.formula_reclosure_fcr3b2.v1\""
           << ",\"identity\":\"nuv-variational-fcr1\""
           << ",\"candidate\":\"pressure-chebyshev-rho-0.9-armijo-v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"spectral_radius\":0.90000000000000002"
           << ",\"nonpressure_unchanged\":"
           << (nonpressure_unchanged ? "true" : "false")
           << ",\"performance_report\":{\"baseline_stiff_evaluations\":"
           << baseline_evaluations
           << ",\"candidate_stiff_evaluations\":" << candidate_evaluations
           << ",\"gating\":false},\"cases\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_sissm_case(report, cases[i]);
    }
    report << "],\"masks\":[";
    for (std::size_t i = 0; i < mask_cases.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_sissm_case(report, mask_cases[i]);
    }
    report << "]"
           << ",\"cost_localization_authorized\":"
           << (passed ? "true" : "false")
           << ",\"profile_reclosure_authorized\":false"
           << ",\"runtime_authority\":false";
    std::string result_material = std::string(passed ? "PASS|" : "FAIL|")
        + first_failure + '|' + std::to_string(baseline_evaluations) + '|'
        + std::to_string(candidate_evaluations) + '|'
        + (nonpressure_unchanged ? "UNCHANGED" : "CHANGED");
    for (const SissmCase& value : cases) {
        result_material += '|' + value.name + ':'
            + std::to_string(value.candidate.final.gradient_norm);
    }
    for (const SissmCase& value : mask_cases) {
        result_material += '|' + value.name + ':'
            + std::to_string(value.candidate.final.gradient_norm);
    }
    report << ",\"result_sha256\":\"" << sha256_hex(result_material) << "\"}";
    return {passed, report.str()};
}

ReferenceSolverReport run_spectral_hvp_controls() {
    const std::array<SpectralFixture, 4> fixtures = spectral_fixtures();
    std::array<SpectralCase, 4> cases;
    bool passed = true;
    std::string first_failure;
    for (std::size_t i = 0; i < fixtures.size(); ++i) {
        cases[i] = analyze_spectral_fixture(fixtures[i]);
        passed = passed && cases[i].passed;
        if (first_failure.empty() && !cases[i].passed) {
            if (!cases[i].finite) {
                first_failure = "NSR0_NONFINITE:" + cases[i].name;
            } else if (!cases[i].fixed_branches) {
                first_failure = "NSR0_BRANCH_CROSSING:" + cases[i].name;
            } else if (!cases[i].jacobi_converged) {
                first_failure = "NSR0_JACOBI_BUDGET:" + cases[i].name;
            } else if (cases[i].hvp_fd_error > 2.0e-6) {
                first_failure = "NSR0_HVP_FD_MISMATCH:" + cases[i].name;
            } else if (cases[i].symmetry_error > 2.0e-12) {
                first_failure = "NSR0_HESSIAN_ASYMMETRY:" + cases[i].name;
            } else {
                first_failure = "NSR0_DENSE_PRODUCT_MISMATCH:" + cases[i].name;
            }
        }
    }

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr0_spectral_hvp.v1\""
           << ",\"identity\":\"nuv-newton-krylov-r0\""
           << ",\"objective_parent\":\"nuv-variational-fcr1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"thresholds\":{\"fd_epsilon_scale\":"
           << std::ldexp(1.0, -20)
           << ",\"hvp_fd_relative\":2e-6"
           << ",\"symmetry_relative\":2e-12"
           << ",\"dense_product_relative\":2e-12"
           << ",\"jacobi_relative\":1e-12"
           << ",\"jacobi_pivot_factor\":64}"
           << ",\"cases\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_spectral_case(report, cases[i]);
    }
    report << "]"
           << ",\"nsr1_authorized\":" << (passed ? "true" : "false")
           << ",\"runtime_authority\":false";
    std::string result_material = std::string(passed ? "PASS|" : "FAIL|")
        + first_failure;
    for (const SpectralCase& value : cases) {
        result_material += '|' + value.name + ':'
            + std::to_string(value.hvp_fd_error) + ':'
            + std::to_string(value.minimum_eigenvalue) + ':'
            + std::to_string(value.maximum_eigenvalue) + ':'
            + std::to_string(value.off_particle_block_ratio);
    }
    report << ",\"result_sha256\":\"" << sha256_hex(result_material) << "\"}";
    return {passed, report.str()};
}

ReferenceSolverReport run_trust_region_controls() {
    std::array<TrustCase, 2> cases = {
        make_trust_compression_case(),
        make_trust_combined_case(),
    };
    bool passed = true;
    std::string first_failure;
    for (const TrustCase& value : cases) {
        passed = passed && value.passed;
        if (!value.passed && first_failure.empty()) {
            if (!value.candidate.failure.empty()) {
                first_failure = "NSR1_" + value.candidate.failure
                    + ':' + value.name;
            } else if (value.candidate.final.total
                > value.baseline.final.total
                    + 1.0e-10
                        * std::max(std::abs(value.baseline.final.total), 1.0)) {
                first_failure = "NSR1_OBJECTIVE_QUALITY:" + value.name;
            } else if (value.candidate.final.gradient_norm
                > std::max(2.0 * value.baseline.final.gradient_norm, 1.0e-8)) {
                first_failure = "NSR1_GRADIENT_QUALITY:" + value.name;
            } else if (value.candidate.final.internal_momentum_residual
                > CONSERVATION_LIMIT) {
                first_failure = "NSR1_MOMENTUM_CLOSURE:" + value.name;
            } else {
                first_failure = "NSR1_EVALUATION_REDUCTION:" + value.name;
            }
        }
    }

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr1_trust_region.v1\""
           << ",\"identity\":\"nuv-newton-krylov-r0\""
           << ",\"objective_parent\":\"nuv-variational-fcr1\""
           << ",\"solver\":\"steihaug-toint-unpreconditioned-v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"policy\":{\"initial_radius\":0.05"
           << ",\"minimum_radius_scale\":" << std::ldexp(1.0, -40)
           << ",\"maximum_radius_scale\":4"
           << ",\"accept_ratio\":0.10000000000000001"
           << ",\"shrink_ratio\":0.25,\"grow_ratio\":2"
           << ",\"maximum_outer_trials\":64"
           << ",\"gradient_tolerance\":1e-10}"
           << ",\"cases\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_trust_case(report, cases[i]);
    }
    report << "]"
           << ",\"nsr2_authorized\":" << (passed ? "true" : "false")
           << ",\"runtime_authority\":false";
    std::string result_material = std::string(passed ? "PASS|" : "FAIL|")
        + first_failure;
    for (const TrustCase& value : cases) {
        result_material += '|' + value.name + ':'
            + std::to_string(value.candidate.final.total) + ':'
            + std::to_string(value.candidate.final.gradient_norm) + ':'
            + std::to_string(value.candidate.objective_evaluations) + ':'
            + std::to_string(value.candidate.hvp_calls);
    }
    report << ",\"result_sha256\":\"" << sha256_hex(result_material) << "\"}";
    return {passed, report.str()};
}

ReferenceSolverReport run_block_preconditioner_controls() {
    std::array<BlockScalingCase, 3> cases = {
        make_block_scaling_case(2),
        make_block_scaling_case(3),
        make_block_scaling_case(4),
    };
    int baseline_large_hvps = 0;
    int candidate_large_hvps = 0;
    bool cases_passed = true;
    std::string first_failure;
    for (std::size_t i = 0; i < cases.size(); ++i) {
        cases_passed = cases_passed && cases[i].passed;
        if (!cases[i].passed && first_failure.empty()) {
            first_failure = "NSR2A_CASE_QUALITY:" + cases[i].name;
        }
        if (i > 0) {
            baseline_large_hvps += cases[i].baseline.hvp_calls;
            candidate_large_hvps += cases[i].candidate.hvp_calls;
        }
    }
    const bool aggregate_reduction =
        4LL * static_cast<long long>(candidate_large_hvps)
        <= 3LL * static_cast<long long>(baseline_large_hvps);
    if (first_failure.empty() && !aggregate_reduction) {
        first_failure = "NSR2A_AGGREGATE_HVP_REDUCTION";
    }
    const bool passed = cases_passed && aggregate_reduction;

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr2a_block_preconditioner.v1\""
           << ",\"identity\":\"nuv-newton-krylov-r0\""
           << ",\"candidate\":\"block-gn-metric-v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"thresholds\":{\"aggregate_hvp_ratio\":0.75"
           << ",\"objective_relative\":1e-10"
           << ",\"gradient_ratio\":2"
           << ",\"momentum\":1e-12}"
           << ",\"aggregate\":{\"baseline_large_hvps\":"
           << baseline_large_hvps
           << ",\"candidate_large_hvps\":" << candidate_large_hvps
           << ",\"reduction_gate\":\""
           << (aggregate_reduction ? "PASS" : "FAIL") << "\"}"
           << ",\"cases\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_block_scaling_case(report, cases[i]);
    }
    report << "]"
           << ",\"selected_preconditioner\":\""
           << (passed ? "block-gn-metric-v1" : "unpreconditioned") << '"'
           << ",\"nsr2b_authorized\":true"
           << ",\"runtime_authority\":false";
    std::string result_material = std::string(passed ? "PASS|" : "FAIL|")
        + first_failure + '|' + std::to_string(baseline_large_hvps) + '|'
        + std::to_string(candidate_large_hvps);
    for (const BlockScalingCase& value : cases) {
        result_material += '|' + value.name + ':'
            + std::to_string(value.baseline.hvp_calls) + ':'
            + std::to_string(value.candidate.hvp_calls) + ':'
            + std::to_string(value.candidate.final.total);
    }
    report << ",\"result_sha256\":\"" << sha256_hex(result_material) << "\"}";
    return {passed, report.str()};
}

ReferenceSolverReport run_scale_aware_block_preconditioner_controls() {
    std::array<BlockScalingCase, 3> cases = {
        make_block_scaling_case(2, true),
        make_block_scaling_case(3, true),
        make_block_scaling_case(4, true),
    };
    int baseline_large_hvps = 0;
    int candidate_large_hvps = 0;
    bool cases_passed = true;
    std::string first_failure;
    for (std::size_t i = 0; i < cases.size(); ++i) {
        cases_passed = cases_passed && cases[i].passed;
        if (!cases[i].passed && first_failure.empty()) {
            first_failure = "NSR2A1_CASE_QUALITY:" + cases[i].name;
        }
        if (i > 0) {
            baseline_large_hvps += cases[i].baseline.hvp_calls;
            candidate_large_hvps += cases[i].candidate.hvp_calls;
        }
    }
    const bool aggregate_reduction =
        4LL * static_cast<long long>(candidate_large_hvps)
        <= 3LL * static_cast<long long>(baseline_large_hvps);
    if (first_failure.empty() && !aggregate_reduction) {
        first_failure = "NSR2A1_AGGREGATE_HVP_REDUCTION";
    }
    const bool passed = cases_passed && aggregate_reduction;

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr2a1_scale_aware_block.v1\""
           << ",\"identity\":\"nuv-newton-krylov-r0\""
           << ",\"candidate\":\"block-gn-metric-v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"thresholds\":{\"scaled_displacement\":1e-8"
           << ",\"aggregate_hvp_ratio\":0.75"
           << ",\"objective_relative\":1e-10"
           << ",\"scaled_residual_ratio\":2"
           << ",\"momentum\":1e-12}"
           << ",\"aggregate\":{\"baseline_large_hvps\":"
           << baseline_large_hvps
           << ",\"candidate_large_hvps\":" << candidate_large_hvps
           << ",\"reduction_gate\":\""
           << (aggregate_reduction ? "PASS" : "FAIL") << "\"}"
           << ",\"cases\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_block_scaling_case(report, cases[i], true);
    }
    report << "]"
           << ",\"selected_preconditioner\":\""
           << (passed ? "block-gn-metric-v1" : "unpreconditioned") << '"'
           << ",\"nsr2b_authorized\":true"
           << ",\"runtime_authority\":false";
    std::string result_material = std::string(passed ? "PASS|" : "FAIL|")
        + first_failure + '|' + std::to_string(baseline_large_hvps) + '|'
        + std::to_string(candidate_large_hvps);
    for (const BlockScalingCase& value : cases) {
        result_material += '|' + value.name + ':'
            + std::to_string(value.baseline.hvp_calls) + ':'
            + std::to_string(value.candidate.hvp_calls) + ':'
            + std::to_string(value.candidate.final.total) + ':'
            + std::to_string(
                value.candidate.final_scaled_displacement_residual);
    }
    report << ",\"result_sha256\":\"" << sha256_hex(result_material) << "\"}";
    return {passed, report.str()};
}

ReferenceSolverReport run_neighborhood_hvp_controls() {
    const std::array<NeighborhoodFixture, 7> fixtures =
        neighborhood_fixtures();
    std::array<NeighborhoodCase, 7> cases;
    bool passed = true;
    std::string first_failure;
    for (std::size_t i = 0; i < fixtures.size(); ++i) {
        cases[i] = analyze_neighborhood_fixture(fixtures[i]);
        passed = passed && cases[i].passed;
        if (!cases[i].passed && first_failure.empty()) {
            if (!cases[i].pair_exact) {
                first_failure = "NSR2B_PAIR_MISMATCH:" + cases[i].name;
            } else if (!cases[i].evaluation_exact) {
                first_failure = "NSR2B_EVALUATION_MISMATCH:" + cases[i].name;
            } else {
                first_failure = "NSR2B_HVP_MISMATCH:" + cases[i].name;
            }
        }
    }
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr2b_neighborhood_hvp.v1\""
           << ",\"identity\":\"nuv-newton-krylov-r0\""
           << ",\"candidate\":\"canonical-cell-neighborhood-v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"cell_edge\":\"max(horizon,3*spacing)\""
           << ",\"comparison\":\"binary64-exact\""
           << ",\"cases\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_neighborhood_case(report, cases[i]);
    }
    report << "]"
           << ",\"nsr2c_authorized\":" << (passed ? "true" : "false")
           << ",\"runtime_authority\":false";
    std::string result_material = std::string(passed ? "PASS|" : "FAIL|")
        + first_failure;
    for (const NeighborhoodCase& value : cases) {
        result_material += '|' + value.name + ':'
            + std::to_string(value.current_pairs) + ':'
            + std::to_string(value.reference_pairs) + ':'
            + value.pair_sha256;
    }
    report << ",\"result_sha256\":\"" << sha256_hex(result_material) << "\"}";
    return {passed, report.str()};
}

ReferenceSolverReport run_neighborhood_trust_scaling_controls() {
    std::array<NeighborhoodScalingCase, 6> cases = {
        make_neighborhood_correspondence_case(2),
        make_neighborhood_correspondence_case(3),
        make_neighborhood_correspondence_case(4),
        make_neighborhood_scale_case(5),
        make_neighborhood_scale_case(8),
        make_neighborhood_scale_case(10),
    };
    bool passed = true;
    std::string first_failure;
    for (std::size_t i = 0; i < cases.size(); ++i) {
        passed = passed && cases[i].passed;
        if (!cases[i].passed && first_failure.empty()) {
            first_failure = i < 3
                ? "NSR2C_SOLVER_CORRESPONDENCE:" + cases[i].name
                : "NSR2C_SCALE_GATE:" + cases[i].name;
        }
    }
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr2c_neighborhood_trust.v1\""
           << ",\"identity\":\"nuv-newton-krylov-r0\""
           << ",\"candidate\":\"canonical-neighborhood-trust-v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"thresholds\":{\"scaled_displacement\":1e-8"
           << ",\"maximum_outer_trials\":32"
           << ",\"maximum_rejected_trials\":8"
           << ",\"maximum_hvp_calls\":128"
           << ",\"maximum_neighbors\":160"
           << ",\"maximum_pairs_per_particle\":80"
           << ",\"momentum\":1e-12}"
           << ",\"cases\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_neighborhood_scaling_case(report, cases[i]);
    }
    report << "]"
           << ",\"nsr3_authorized\":" << (passed ? "true" : "false")
           << ",\"runtime_authority\":false";
    std::string result_material = std::string(passed ? "PASS|" : "FAIL|")
        + first_failure;
    for (const NeighborhoodScalingCase& value : cases) {
        result_material += '|' + value.name + ':'
            + std::to_string(value.neighborhood.solve.final.total) + ':'
            + std::to_string(value.neighborhood.solve.hvp_calls) + ':'
            + std::to_string(value.neighborhood.maximum_pairs) + ':'
            + std::to_string(value.neighborhood.maximum_neighbors);
    }
    report << ",\"result_sha256\":\"" << sha256_hex(result_material) << "\"}";
    return {passed, report.str()};
}

ReferenceSolverReport run_neighborhood_trust_rejection_trace_controls() {
    const NeighborhoodFixture fixture = make_neighborhood_scale_fixture(8);
    const NeighborhoodTrustResult result = solve_neighborhood_trust_region(
        fixture.config, fixture.x, fixture.velocity, true);
    std::array<int, 4> rejection_counts{};
    for (const NeighborhoodTrustResult::TrialTrace& trace : result.trace) {
        if (!trace.accepted) {
            ++rejection_counts[static_cast<std::size_t>(
                classify_rejection(trace))];
        }
    }
    int observed_classes = 0;
    std::size_t selected_class = 0;
    for (std::size_t i = 0; i < rejection_counts.size(); ++i) {
        if (rejection_counts[i] != 0) {
            ++observed_classes;
            selected_class = i;
        }
    }
    const std::string classification = observed_classes == 1
        ? rejection_class_name(static_cast<RejectionClass>(selected_class))
        : "MIXED_OR_UNRESOLVED";
    const bool parent_exact = result.solve.outer_trials == 26
        && result.solve.accepted_trials == 13
        && result.solve.rejected_trials == 13
        && result.solve.hvp_calls == 153
        && result.maximum_pairs == 19492
        && result.maximum_neighbors == 122
        && result.trace.size()
            == static_cast<std::size_t>(result.solve.outer_trials);
    const bool passed = parent_exact && observed_classes != 0;
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr2c1_rejection_trace.v1\""
           << ",\"identity\":\"nuv-newton-krylov-r0\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"parent_exact\":" << (parent_exact ? "true" : "false")
           << ",\"classification\":\"" << classification << '"'
           << ",\"classification_counts\":{"
           << "\"arithmetic_floor\":" << rejection_counts[0]
           << ",\"active_set_model_mismatch\":" << rejection_counts[1]
           << ",\"support_topology_model_mismatch\":"
           << rejection_counts[2]
           << ",\"smooth_model_conditioning\":" << rejection_counts[3]
           << "},\"outer_trials\":" << result.solve.outer_trials
           << ",\"accepted_trials\":" << result.solve.accepted_trials
           << ",\"rejected_trials\":" << result.solve.rejected_trials
           << ",\"hvp_calls\":" << result.solve.hvp_calls
           << ",\"trace\":[";
    for (std::size_t i = 0; i < result.trace.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_trial_trace(report, result.trace[i]);
    }
    const std::string result_material = classification + '|'
        + std::to_string(result.solve.outer_trials) + '|'
        + std::to_string(result.solve.rejected_trials) + '|'
        + std::to_string(result.solve.hvp_calls) + '|'
        + std::to_string(rejection_counts[0]) + '|'
        + std::to_string(rejection_counts[1]) + '|'
        + std::to_string(rejection_counts[2]) + '|'
        + std::to_string(rejection_counts[3]);
    report << "]"
           << ",\"remediation_authority\":false"
           << ",\"runtime_authority\":false"
           << ",\"result_sha256\":\"" << sha256_hex(result_material)
           << "\"}";
    return {passed, report.str()};
}

ReferenceSolverReport run_numerical_floor_stop_controls() {
    std::array<NumericalFloorCase, 6> cases = {
        make_numerical_floor_correspondence_case(2),
        make_numerical_floor_correspondence_case(3),
        make_numerical_floor_correspondence_case(4),
        make_numerical_floor_scale_case(5),
        make_numerical_floor_scale_case(8),
        make_numerical_floor_scale_case(10),
    };
    bool passed = true;
    std::string first_failure;
    for (const NumericalFloorCase& value : cases) {
        passed = passed && value.passed;
        if (!value.passed && first_failure.empty()) {
            first_failure = "NSR2C2_GATE:" + value.name;
        }
    }
    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr2c2_numerical_floor.v1\""
           << ",\"identity\":\"nuv-newton-krylov-r0\""
           << ",\"candidate\":\"numerical-energy-floor-stop-v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"thresholds\":{\"epsilon_multiplier\":1024"
           << ",\"ordinary_scaled_displacement\":1e-8"
           << ",\"floor_scaled_displacement\":1e-7"
           << ",\"floor_scaled_step\":1e-7"
           << ",\"maximum_outer_trials\":32"
           << ",\"maximum_rejected_trials\":8"
           << ",\"maximum_hvp_calls\":128"
           << ",\"momentum\":1e-12}"
           << ",\"cases\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_numerical_floor_case(report, cases[i]);
    }
    std::string result_material = std::string(passed ? "PASS|" : "FAIL|")
        + first_failure;
    for (const NumericalFloorCase& value : cases) {
        result_material += '|' + value.name + ':'
            + value.neighborhood.solve.convergence_stop + ':'
            + std::to_string(value.neighborhood.solve.outer_trials) + ':'
            + std::to_string(value.neighborhood.solve.rejected_trials) + ':'
            + std::to_string(value.neighborhood.solve.hvp_calls);
    }
    report << "]"
           << ",\"nsr3_authorized\":" << (passed ? "true" : "false")
           << ",\"runtime_authority\":false"
           << ",\"result_sha256\":\"" << sha256_hex(result_material)
           << "\"}";
    return {passed, report.str()};
}

ReferenceSolverReport run_serial_cpu_baseline_controls() {
    std::array<SerialBenchmarkCase, 4> cases = {
        make_serial_benchmark_case(8),
        make_serial_benchmark_case(10),
        make_serial_benchmark_case(12),
        make_serial_benchmark_case(16),
    };
    bool passed = true;
    std::string first_failure;
    std::array<std::uint64_t, 4> bucket_median_sums{};
    for (const SerialBenchmarkCase& value : cases) {
        passed = passed && value.passed;
        if (!value.passed && first_failure.empty()) {
            first_failure = "NSR3A_GATE:" + value.name;
        }
        bucket_median_sums[0] += timing_percentile(value.samples,
            &SerialTimingSample::pair_and_adjacency, 50);
        bucket_median_sums[1] += timing_percentile(value.samples,
            &SerialTimingSample::objective_gradient, 50);
        bucket_median_sums[2] += timing_percentile(value.samples,
            &SerialTimingSample::hvp, 50);
        bucket_median_sums[3] += timing_percentile(value.samples,
            &SerialTimingSample::control_and_vector, 50);
    }
    const std::array<const char*, 4> optimization = {
        "PAIR_AND_ADJACENCY",
        "OBJECTIVE_GRADIENT",
        "HVP_ALLOCATION_AND_TRAVERSAL",
        "KRYLOV_CONTROL_AND_VECTOR",
    };
    const std::size_t dominant = static_cast<std::size_t>(
        std::distance(bucket_median_sums.begin(),
            std::max_element(
                bucket_median_sums.begin(), bucket_median_sums.end())));
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3a_serial_cpu.v1\""
           << ",\"identity\":\"nuv-newton-krylov-r0\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"compiler\":\"" << __VERSION__ << '"'
           << ",\"thread_model\":\"serial\""
           << ",\"warmups\":1,\"measured_runs\":7"
           << ",\"timings_are_environmental\":true"
           << ",\"cases\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_serial_benchmark_case(report, cases[i]);
    }
    std::string result_material = std::string(passed ? "PASS|" : "FAIL|")
        + first_failure;
    for (const SerialBenchmarkCase& value : cases) {
        result_material += '|' + value.name + ':' + value.state_sha256 + ':'
            + std::to_string(value.reference.solve.hvp_calls) + ':'
            + std::to_string(value.reference.maximum_pairs);
    }
    report << "]"
           << ",\"dominant_median_bucket\":\"" << optimization[dominant]
           << '"'
           << ",\"selected_next_optimization\":\""
           << optimization[dominant] << '"'
           << ",\"hardware_counters\":\"UNAVAILABLE\""
           << ",\"runtime_authority\":false"
           << ",\"result_sha256\":\"" << sha256_hex(result_material)
           << "\"}";
    return {passed, report.str()};
}

ReferenceSolverReport run_hvp_workspace_stream_controls() {
    const bool hvp_exact = workspace_hvp_exact_controls();
    std::array<HvpTournamentCase, 4> cases = {
        make_hvp_tournament_case(8),
        make_hvp_tournament_case(10),
        make_hvp_tournament_case(12),
        make_hvp_tournament_case(16),
    };
    bool passed = hvp_exact;
    std::string first_failure = hvp_exact ? "" : "NSR3A1_HVP_EXACT";
    for (const HvpTournamentCase& value : cases) {
        passed = passed && value.passed;
        if (!value.passed && first_failure.empty()) {
            first_failure = "NSR3A1_GATE:" + value.name;
        }
    }
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3a1_hvp_workspace.v1\""
           << ",\"identity\":\"nuv-newton-krylov-r0\""
           << ",\"candidate\":\"hvp-workspace-stream-v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"hvp_controls_exact\":"
           << (hvp_exact ? "true" : "false")
           << ",\"warmups_per_implementation\":1"
           << ",\"alternating_pairs\":7"
           << ",\"thresholds\":{\"minimum_hvp_speedup\":1.2"
           << ",\"minimum_large_total_speedup\":1.1"
           << ",\"maximum_small_total_regression\":0.02}"
           << ",\"cases\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_hvp_tournament_case(report, cases[i]);
    }
    std::string result_material = std::string(passed ? "PASS|" : "FAIL|")
        + first_failure + '|' + (hvp_exact ? "EXACT" : "MISMATCH");
    for (const HvpTournamentCase& value : cases) {
        result_material += '|' + value.name + ':'
            + hash_neighborhood_trust_state(value.candidate_reference) + ':'
            + (value.exact_state ? "EXACT" : "MISMATCH");
    }
    report << "]"
           << ",\"candidate_selected\":"
           << (passed ? "true" : "false")
           << ",\"runtime_authority\":false"
           << ",\"result_sha256\":\"" << sha256_hex(result_material)
           << "\"}";
    return {passed, report.str()};
}

ReferenceSolverReport run_hessian_tape_controls() {
    const bool hvp_exact = hessian_tape_hvp_exact_controls();
    if (!hvp_exact) {
        const std::string result_material = "FAIL|NSR3A2_HVP_EXACT|MISMATCH";
        std::ostringstream report;
        report << "{\"schema\":\"nextengine.nonlocal.nsr3a2_hessian_tape.v1\""
               << ",\"identity\":\"nuv-newton-krylov-r0\""
               << ",\"candidate\":\"outer-state-hessian-tape-v1\""
               << ",\"status\":\"FAIL\""
               << ",\"first_failure\":\"NSR3A2_HVP_EXACT\""
               << ",\"hvp_controls_exact\":false"
               << ",\"tournament_executed\":false"
               << ",\"candidate_selected\":false"
               << ",\"runtime_authority\":false"
               << ",\"result_sha256\":\""
               << sha256_hex(result_material) << "\"}";
        return {false, report.str()};
    }
    std::array<HessianTapeTournamentCase, 4> cases = {
        make_hessian_tape_tournament_case(8),
        make_hessian_tape_tournament_case(10),
        make_hessian_tape_tournament_case(12),
        make_hessian_tape_tournament_case(16),
    };
    bool passed = true;
    std::string first_failure;
    for (const HessianTapeTournamentCase& value : cases) {
        passed = passed && value.passed;
        if (!value.passed && first_failure.empty()) {
            first_failure = "NSR3A2_GATE:" + value.name;
        }
    }
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3a2_hessian_tape.v1\""
           << ",\"identity\":\"nuv-newton-krylov-r0\""
           << ",\"candidate\":\"outer-state-hessian-tape-v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"hvp_controls_exact\":true"
           << ",\"tournament_executed\":true"
           << ",\"warmups_per_implementation\":1"
           << ",\"alternating_pairs\":7"
           << ",\"thresholds\":{\"minimum_combined_hvp_speedup\":1.2"
           << ",\"minimum_large_total_speedup\":1.1"
           << ",\"maximum_small_total_regression\":0.02"
           << ",\"maximum_bytes_per_pair\":192"
           << ",\"maximum_bytes_per_particle\":128}"
           << ",\"cases\":[";
    for (std::size_t i = 0; i < cases.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_hessian_tape_tournament_case(report, cases[i]);
    }
    std::string result_material = std::string(passed ? "PASS|" : "FAIL|")
        + first_failure + "|EXACT";
    for (const HessianTapeTournamentCase& value : cases) {
        result_material += '|' + value.name + ':'
            + hash_neighborhood_trust_state(value.candidate_reference) + ':'
            + (value.exact_state ? "EXACT" : "MISMATCH") + ':'
            + (value.capacity_valid ? "CAPACITY_OK" : "CAPACITY_FAIL") + ':'
            + std::to_string(
                value.candidate_reference.maximum_hessian_tape_bytes);
    }
    report << "]"
           << ",\"candidate_selected\":"
           << (passed ? "true" : "false")
           << ",\"runtime_authority\":false"
           << ",\"result_sha256\":\"" << sha256_hex(result_material)
           << "\"}";
    return {passed, report.str()};
}

ReferenceSolverReport run_normalized_kernel_reclosure_controls() {
    const ReferenceSolverReport dimensional =
        run_dimensional_profile_controls();
    const std::array<SpectralFixture, 6> fixtures =
        normalized_spectral_fixtures();
    std::array<NormalizedSpectralControl, 6> spectral;
    bool spectral_passed = true;
    for (std::size_t i = 0; i < fixtures.size(); ++i) {
        spectral[i].spectral = analyze_spectral_fixture(fixtures[i]);
        spectral[i].gradient_error =
            spectral_directional_gradient_error(fixtures[i]);
        const std::vector<Vec3> y = predict(
            fixtures[i].config, fixtures[i].x, fixtures[i].velocity);
        spectral[i].momentum_residual = evaluate(
            fixtures[i].config, fixtures[i].x, y, y)
            .internal_momentum_residual;
        spectral[i].passed = spectral[i].spectral.passed
            && spectral[i].gradient_error <= DERIVATIVE_LIMIT
            && spectral[i].momentum_residual <= CONSERVATION_LIMIT;
        spectral_passed = spectral_passed && spectral[i].passed;
    }
    const NormalizedDensityControl density = normalized_density_control();
    const NormalizedTrustControl trust = normalized_trust_control(fixtures);
    const NormalizedTapeControl tape = normalized_tape_control();
    const Config defaults;
    const double kernel_scale = reference_lattice_kernel_scale(
        defaults.spacing, defaults.horizon);
    const bool scale_valid = relative_error(
        kernel_scale, 7.985668078772472) <= 1.0e-14;
    const bool passed = dimensional.passed && scale_valid && spectral_passed
        && density.passed && trust.passed && tape.passed;
    std::string first_failure;
    if (!dimensional.passed) {
        first_failure = "NSR3B0R_DIMENSIONAL_PARENT";
    } else if (!scale_valid) {
        first_failure = "NSR3B0R_KERNEL_SCALE";
    } else if (!spectral_passed) {
        for (const NormalizedSpectralControl& value : spectral) {
            if (!value.passed) {
                first_failure = "NSR3B0R_SPECTRAL:" + value.spectral.name;
                break;
            }
        }
    } else if (!density.passed) {
        first_failure = "NSR3B0R_PHYSICAL_DENSITY";
    } else if (!trust.passed) {
        first_failure = "NSR3B0R_TRUST";
    } else if (!tape.passed) {
        first_failure = "NSR3B0R_TAPE";
    }

    std::ostringstream result_material;
    result_material << std::setprecision(17)
                    << (passed ? "PASS|" : "FAIL|") << first_failure << '|'
                    << kernel_scale << '|' << density.reference_center_ratio
                    << '|' << density.compressed_center_ratio;
    for (const NormalizedSpectralControl& value : spectral) {
        result_material << '|' << value.spectral.name << ':'
                        << value.gradient_error << ':'
                        << value.spectral.hvp_fd_error << ':'
                        << value.spectral.dense_product_error;
    }
    result_material << '|' << trust.pressure.final.total << ':'
                    << trust.pressure.hvp_calls << '|'
                    << trust.combined.final.total << ':'
                    << trust.combined.hvp_calls << '|'
                    << hash_neighborhood_trust_state(tape.a2);

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b0r_normalized.v1\""
           << ",\"identity\":\"nuv-variational-fcr2\""
           << ",\"solver_identity\":\"nuv-newton-krylov-r0\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"candidate\":\"lattice-normalized-cubic-v1\""
           << ",\"kernel_scale\":" << kernel_scale
           << ",\"dimensional_parent_passed\":"
           << (dimensional.passed ? "true" : "false")
           << ",\"spectral_controls\":[";
    for (std::size_t i = 0; i < spectral.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        append_normalized_spectral_control(report, spectral[i]);
    }
    report << "]"
           << ",\"physical_density\":{\"status\":\""
           << (density.passed ? "PASS" : "FAIL")
           << "\",\"particles\":" << density.particles
           << ",\"pairs\":" << density.pairs
           << ",\"reference_center_ratio\":"
           << density.reference_center_ratio
           << ",\"compressed_center_ratio\":"
           << density.compressed_center_ratio
           << ",\"compressed_active_particles\":"
           << density.compressed_active_particles
           << ",\"momentum_residual\":" << density.momentum_residual << "}"
           << ",\"trust\":{\"status\":\""
           << (trust.passed ? "PASS" : "FAIL")
           << "\",\"pressure\":";
    append_normalized_trust_result(report, trust.pressure);
    report << ",\"combined\":";
    append_normalized_trust_result(report, trust.combined);
    report << "}"
           << ",\"tape\":{\"status\":\""
           << (tape.passed ? "PASS" : "FAIL")
           << "\",\"exact\":" << (tape.exact ? "true" : "false")
           << ",\"capacity_valid\":"
           << (tape.capacity_valid ? "true" : "false")
           << ",\"particles\":512"
           << ",\"pairs\":" << tape.a2.maximum_pairs
           << ",\"outer_trials\":" << tape.a2.solve.outer_trials
           << ",\"hvp_calls\":" << tape.a2.solve.hvp_calls
           << ",\"tape_hvp_checks\":" << tape.a2.hessian_tape_hvp_checks
           << ",\"state_sha256\":\""
           << hash_neighborhood_trust_state(tape.a2) << "\"}"
           << ",\"historical_hash_check_required\":true"
           << ",\"selected\":" << (passed ? "true" : "false")
           << ",\"runtime_authority\":false"
           << ",\"result_sha256\":\""
           << sha256_hex(result_material.str()) << "\"}";
    return {passed, report.str()};
}

ReferenceSolverReport run_manufactured_multistep_controls() {
    const FreeFlightControl free_flight =
        free_flight_multistep_control();
    const RigidTranslationControl translation =
        rigid_translation_multistep_control();
    const GalileanControl galilean = galilean_multistep_control();
    const CompressionControl compression =
        compression_multistep_control();
    const RotationControl rotation = rotation_objectivity_control();
    const bool passed = free_flight.passed && translation.passed
        && galilean.passed && compression.passed && rotation.passed;
    std::string first_failure;
    if (!free_flight.passed) {
        first_failure = "NSR3B1_FREE_FLIGHT";
    } else if (!translation.passed) {
        first_failure = "NSR3B1_RIGID_TRANSLATION";
    } else if (!galilean.passed) {
        first_failure = "NSR3B1_GALILEAN_COVARIANCE";
    } else if (!compression.passed) {
        first_failure = "NSR3B1_COMPRESSION_RELAXATION";
    } else if (!rotation.passed) {
        first_failure = "NSR3B1_ROTATION_OBJECTIVITY";
    }

    std::ostringstream result_material;
    result_material << std::setprecision(17)
                    << (passed ? "PASS|" : "FAIL|") << first_failure
                    << '|' << free_flight.maximum_position_error << ':'
                    << free_flight.maximum_velocity_error << ':'
                    << hash_phase_state(
                        free_flight.run.position, free_flight.run.velocity)
                    << '|' << translation.maximum_position_error << ':'
                    << translation.maximum_velocity_error << ':'
                    << hash_phase_state(
                        translation.run.position, translation.run.velocity)
                    << '|' << galilean.maximum_position_error << ':'
                    << galilean.maximum_velocity_error << ':'
                    << (galilean.signatures_exact ? "EXACT" : "MISMATCH")
                    << ':' << hash_phase_state(
                        galilean.reference.position,
                        galilean.reference.velocity)
                    << '|' << compression.coarse_error << ':'
                    << compression.fine_error << ':'
                    << compression.convergence_ratio;
    for (const MultistepRun& run : compression.runs) {
        result_material << ':'
                        << hash_phase_state(run.position, run.velocity);
    }
    for (double value : rotation.selected_energy) {
        result_material << '|' << value;
    }
    for (double value : rotation.comparator_energy) {
        result_material << '|' << value;
    }

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b1_multistep.v1\""
           << ",\"identity\":\"nuv-variational-fcr2\""
           << ",\"solver_identity\":\"nuv-newton-krylov-r0\""
           << ",\"solver_storage\":\"outer-state-hessian-tape-v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"thresholds\":{\"trajectory_error\":1e-11"
           << ",\"galilean_error\":1e-10"
           << ",\"center_of_mass_normalized\":1e-11"
           << ",\"accumulated_momentum\":1e-10"
           << ",\"maximum_outer_trials_per_step\":32"
           << ",\"maximum_rejected_trials_per_step\":8"
           << ",\"maximum_hvp_calls_per_step\":128"
           << ",\"minimum_step_doubling_ratio\":1.5}"
           << ",\"cases\":[{\"name\":\"B1-FF\",\"status\":\""
           << (free_flight.passed ? "PASS" : "FAIL")
           << "\",\"maximum_position_error\":"
           << free_flight.maximum_position_error
           << ",\"maximum_velocity_error\":"
           << free_flight.maximum_velocity_error << ",\"work\":";
    append_multistep_work(report, free_flight.run);
    report << "},{\"name\":\"B1-RT\",\"status\":\""
           << (translation.passed ? "PASS" : "FAIL")
           << "\",\"maximum_position_error\":"
           << translation.maximum_position_error
           << ",\"maximum_velocity_error\":"
           << translation.maximum_velocity_error
           << ",\"maximum_center_of_mass_error\":"
           << translation.maximum_center_of_mass_error << ",\"work\":";
    append_multistep_work(report, translation.run);
    report << "},{\"name\":\"B1-GC\",\"status\":\""
           << (galilean.passed ? "PASS" : "FAIL")
           << "\",\"maximum_position_error\":"
           << galilean.maximum_position_error
           << ",\"maximum_velocity_error\":"
           << galilean.maximum_velocity_error
           << ",\"signatures_exact\":"
           << (galilean.signatures_exact ? "true" : "false")
           << ",\"reference_work\":";
    append_multistep_work(report, galilean.reference);
    report << ",\"boosted_work\":";
    append_multistep_work(report, galilean.boosted);
    report << "},{\"name\":\"B1-CR\",\"status\":\""
           << (compression.passed ? "PASS" : "FAIL")
           << "\",\"initial_active_pressure_centers\":"
           << compression.initial_active_pressure_centers
           << ",\"initial_maximum_density_ratio\":"
           << compression.initial_maximum_density_ratio
           << ",\"coarse_error\":" << compression.coarse_error
           << ",\"fine_error\":" << compression.fine_error
           << ",\"convergence_ratio\":"
           << compression.convergence_ratio
           << ",\"maximum_normalized_center_of_mass_drift\":"
           << compression.maximum_normalized_center_of_mass_drift
           << ",\"maximum_accumulated_momentum_residual\":"
           << compression.maximum_accumulated_momentum_residual
           << ",\"effective_bulk_modulus_pa\":"
           << compression.effective_bulk_modulus
           << ",\"acoustic_wave_speed_m_per_s\":"
           << compression.acoustic_wave_speed
           << ",\"acoustic_courant\":["
           << compression.acoustic_courant[0] << ','
           << compression.acoustic_courant[1] << ','
           << compression.acoustic_courant[2] << ']'
           << ",\"levels\":[";
    for (std::size_t i = 0; i < compression.runs.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        report << "{\"dt\":" << compression.time_steps[i]
               << ",\"work\":";
        append_multistep_work(report, compression.runs[i]);
        report << '}';
    }
    report << "]},{\"name\":\"B1-RO\",\"status\":\""
           << (rotation.passed ? "PASS" : "FAIL")
           << "\",\"selected_energy\":[";
    for (std::size_t i = 0; i < rotation.selected_energy.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        report << rotation.selected_energy[i];
    }
    report << "],\"selected_halving_ratio\":["
           << rotation.selected_ratio[0] << ','
           << rotation.selected_ratio[1]
           << "],\"positive_mu_comparator_energy\":[";
    for (std::size_t i = 0; i < rotation.comparator_energy.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        report << rotation.comparator_energy[i];
    }
    report << "],\"positive_mu_comparator_halving_ratio\":["
           << rotation.comparator_ratio[0] << ','
           << rotation.comparator_ratio[1] << "]}]"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"candidate_selected\":"
           << (passed ? "true" : "false")
           << ",\"boundary_design_authorized\":"
           << (passed ? "true" : "false")
           << ",\"runtime_authority\":false"
           << ",\"result_sha256\":\""
           << sha256_hex(result_material.str()) << "\"}";
    return {passed, report.str()};
}

ReferenceSolverReport run_temporal_stiffness_diagnostic_controls() {
    const TemporalStiffnessDiagnostic diagnostic =
        temporal_stiffness_diagnostic();
    std::ostringstream result_material;
    result_material << std::setprecision(17)
                    << (diagnostic.passed ? "PASS|" : "FAIL|")
                    << diagnostic.first_failure << '|'
                    << diagnostic.disposition << '|'
                    << (diagnostic.overlap_exact ? "OVERLAP" : "MISMATCH");
    for (const TemporalLevel& level : diagnostic.levels) {
        result_material << '|' << level.time_step << ':'
                        << hash_phase_state(
                            level.run.position, level.run.velocity) << ':'
                        << level.rms_radius << ':' << level.rms_speed << ':'
                        << level.kinetic_energy << ':'
                        << level.pressure_exit_time;
    }
    for (double value : diagnostic.position_difference) {
        result_material << '|' << value;
    }
    for (double value : diagnostic.velocity_difference) {
        result_material << '|' << value;
    }
    for (double value : diagnostic.strict_position_difference) {
        result_material << '|' << value;
    }
    for (double value : diagnostic.strict_velocity_difference) {
        result_material << '|' << value;
    }

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b1d_temporal_stiffness.v1\""
           << ",\"identity\":\"nuv-variational-fcr2\""
           << ",\"solver_identity\":\"nuv-newton-krylov-r0\""
           << ",\"parent_b1_result_sha256\":\""
           << "0625dba917f0105b5629d7a2d5e6c8475e9aaa4a4b6b44b201c88122cd00187f\""
           << ",\"status\":\""
           << (diagnostic.passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << diagnostic.first_failure << '"'
           << ",\"disposition\":\"" << diagnostic.disposition << '"'
           << ",\"overlap_exact\":"
           << (diagnostic.overlap_exact ? "true" : "false")
           << ",\"solver_sensitivity_valid\":"
           << (diagnostic.solver_sensitivity_valid ? "true" : "false")
           << ",\"initial_maximum_density_ratio\":"
           << diagnostic.initial_maximum_density_ratio
           << ",\"thresholds\":{\"minimum_ratio\":1.5"
           << ",\"maximum_ratio\":2.5"
           << ",\"maximum_solver_sensitivity_fraction\":0.1"
           << ",\"maximum_outer_trials_per_step\":32"
           << ",\"maximum_rejected_trials_per_step\":8"
           << ",\"maximum_hvp_calls_per_step\":128"
           << ",\"maximum_pairs_per_particle\":80}"
           << ",\"levels\":[";
    for (std::size_t i = 0; i < diagnostic.levels.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        const TemporalLevel& level = diagnostic.levels[i];
        report << "{\"name\":\"D" << i << "\",\"dt\":"
               << level.time_step
               << ",\"acoustic_courant\":" << level.acoustic_courant
               << ",\"rms_radius\":" << level.rms_radius
               << ",\"rms_relative_speed\":" << level.rms_speed
               << ",\"relative_kinetic_energy\":"
               << level.kinetic_energy
               << ",\"pressure_active_exit_time\":"
               << level.pressure_exit_time
               << ",\"normalized_center_drift\":"
               << level.normalized_center_drift
               << ",\"maximum_final_scaled_displacement_residual\":"
               << level.run.maximum_final_scaled_displacement_residual
               << ",\"work\":";
        append_multistep_work(report, level.run);
        report << '}';
    }
    report << "],\"adjacent\":{\"position_difference\":";
    append_double_array(report, diagnostic.position_difference.data(),
        diagnostic.position_difference.size());
    report << ",\"velocity_difference\":";
    append_double_array(report, diagnostic.velocity_difference.data(),
        diagnostic.velocity_difference.size());
    report << ",\"position_ratio\":";
    append_double_array(report, diagnostic.position_ratio.data(),
        diagnostic.position_ratio.size());
    report << ",\"velocity_ratio\":";
    append_double_array(report, diagnostic.velocity_ratio.data(),
        diagnostic.velocity_ratio.size());
    report << ",\"rms_radius_change\":";
    append_double_array(report, diagnostic.radius_change.data(),
        diagnostic.radius_change.size());
    report << ",\"rms_speed_change\":";
    append_double_array(report, diagnostic.speed_change.data(),
        diagnostic.speed_change.size());
    report << ",\"kinetic_energy_change\":";
    append_double_array(report, diagnostic.kinetic_change.data(),
        diagnostic.kinetic_change.size());
    report << ",\"pressure_exit_time_change\":";
    append_double_array(report, diagnostic.exit_time_change.data(),
        diagnostic.exit_time_change.size());
    report << "},\"strict_sensitivity\":{\"levels\":[";
    for (std::size_t i = 0; i < diagnostic.strict_levels.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        const TemporalLevel& level = diagnostic.strict_levels[i];
        report << "{\"name\":\"D" << i + 3 << "-strict\",\"dt\":"
               << level.time_step
               << ",\"scaled_displacement_tolerance\":1e-10"
               << ",\"numerical_floor_stop\":false"
               << ",\"position_difference\":"
               << diagnostic.strict_position_difference[i]
               << ",\"velocity_difference\":"
               << diagnostic.strict_velocity_difference[i]
               << ",\"work\":";
        append_multistep_work(report, level.run);
        report << '}';
    }
    report << "]}"
           << ",\"b1_remains_failed\":true"
           << ",\"substep_design_authorized\":"
           << (diagnostic.asymptotic_regime_observed ? "true" : "false")
           << ",\"integration_reclosure_required\":"
           << (diagnostic.passed
                    && !diagnostic.asymptotic_regime_observed
                ? "true" : "false")
           << ",\"boundary_design_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(result_material.str()) << "\"}";
    return {diagnostic.passed, report.str()};
}

ReferenceSolverReport run_floor_limited_temporal_oracle_controls() {
    const FloorLimitedOracleDiagnostic diagnostic =
        floor_limited_oracle_diagnostic();
    std::ostringstream result_material;
    result_material << std::setprecision(17)
                    << (diagnostic.passed ? "PASS|" : "FAIL|")
                    << diagnostic.first_failure << '|'
                    << diagnostic.disposition << '|'
                    << (diagnostic.main_overlap_exact
                            ? "OVERLAP" : "MISMATCH");
    for (const TemporalLevel& level : diagnostic.main_levels) {
        result_material << '|'
                        << hash_phase_state(
                            level.run.position, level.run.velocity);
    }
    for (std::size_t i = 0; i < diagnostic.oracle_levels.size(); ++i) {
        const TemporalLevel& level = diagnostic.oracle_levels[i];
        result_material << '|'
                        << hash_phase_state(
                            level.run.position, level.run.velocity) << ':'
                        << diagnostic.oracle_position_difference[i] << ':'
                        << diagnostic.oracle_velocity_difference[i] << ':'
                        << level.run.total_numerical_floor_stops;
    }

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b1d1_floor_oracle.v1\""
           << ",\"identity\":\"nuv-variational-fcr2\""
           << ",\"solver_identity\":\"nuv-newton-krylov-r0\""
           << ",\"parent_b1d_result_sha256\":\""
           << "9efcae39dcb56868457ca2105796c8cbab661a7e62d5d33ede1f01dc36fadf28\""
           << ",\"status\":\""
           << (diagnostic.passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << diagnostic.first_failure << '"'
           << ",\"disposition\":\"" << diagnostic.disposition << '"'
           << ",\"main_overlap_exact\":"
           << (diagnostic.main_overlap_exact ? "true" : "false")
           << ",\"oracle_valid\":"
           << (diagnostic.oracle_valid ? "true" : "false")
           << ",\"sensitivity_valid\":"
           << (diagnostic.sensitivity_valid ? "true" : "false")
           << ",\"asymptotic_regime_observed\":"
           << (diagnostic.asymptotic_regime_observed ? "true" : "false")
           << ",\"thresholds\":{\"temporal_position_difference\":"
           << "0.0022880058867231971"
           << ",\"temporal_velocity_difference\":"
           << "0.04678768432527124"
           << ",\"maximum_solver_sensitivity_fraction\":0.1"
           << ",\"oracle_scaled_displacement_limit\":0"
           << ",\"oracle_numerical_floor_stop\":true}"
           << ",\"main_levels\":[";
    for (std::size_t i = 0; i < diagnostic.main_levels.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        const TemporalLevel& level = diagnostic.main_levels[i];
        report << "{\"name\":\"D" << i << "\",\"dt\":"
               << level.time_step
               << ",\"state_sha256\":\""
               << hash_phase_state(
                    level.run.position, level.run.velocity) << '"'
               << ",\"outer_trials\":"
               << level.run.total_outer_trials
               << ",\"accepted_trials\":"
               << level.run.total_accepted_trials
               << ",\"hvp_calls\":" << level.run.total_hvp_calls
               << '}';
    }
    report << "],\"adjacent\":{\"position_difference\":";
    append_double_array(report, diagnostic.position_difference.data(),
        diagnostic.position_difference.size());
    report << ",\"velocity_difference\":";
    append_double_array(report, diagnostic.velocity_difference.data(),
        diagnostic.velocity_difference.size());
    report << ",\"position_ratio\":";
    append_double_array(report, diagnostic.position_ratio.data(),
        diagnostic.position_ratio.size());
    report << ",\"velocity_ratio\":";
    append_double_array(report, diagnostic.velocity_ratio.data(),
        diagnostic.velocity_ratio.size());
    report << "},\"floor_limited_oracles\":[";
    for (std::size_t i = 0; i < diagnostic.oracle_levels.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        const TemporalLevel& level = diagnostic.oracle_levels[i];
        report << "{\"name\":\"D" << i + 3 << "-floor\",\"dt\":"
               << level.time_step
               << ",\"position_difference\":"
               << diagnostic.oracle_position_difference[i]
               << ",\"velocity_difference\":"
               << diagnostic.oracle_velocity_difference[i]
               << ",\"numerical_floor_stops\":"
               << level.run.total_numerical_floor_stops
               << ",\"raw_gradient_stops\":"
               << level.run.total_raw_gradient_stops
               << ",\"scaled_displacement_stops\":"
               << level.run.total_scaled_displacement_stops
               << ",\"work\":";
        append_multistep_work(report, level.run);
        report << '}';
    }
    report << ']'
           << ",\"b1_remains_failed\":true"
           << ",\"b1d_remains_invalid\":true"
           << ",\"substep_design_authorized\":"
           << (diagnostic.passed
                    && diagnostic.asymptotic_regime_observed
                ? "true" : "false")
           << ",\"boundary_design_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(result_material.str()) << "\"}";
    return {diagnostic.passed, report.str()};
}

ReferenceSolverReport run_acoustic_substep_policy_controls() {
    const AcousticSubstepPolicyDiagnostic diagnostic =
        acoustic_substep_policy_diagnostic();
    std::ostringstream result_material;
    result_material << std::setprecision(17)
                    << (diagnostic.passed ? "PASS|" : "FAIL|")
                    << diagnostic.first_failure << '|'
                    << (diagnostic.degenerate.passed
                            ? "DEGENERATE_PASS" : "DEGENERATE_FAIL");
    for (const AcousticPolicyCase& value : diagnostic.cases) {
        result_material << '|' << value.compression_factor << ':'
                        << value.kappa_factor << ':'
                        << value.policy_substeps << ':'
                        << value.position_ratio << ':'
                        << value.velocity_ratio << ':'
                        << value.normalized_policy_position_error << ':'
                        << value.normalized_policy_velocity_error;
        for (const TemporalLevel& level : value.levels) {
            result_material << ':' << hash_phase_state(
                level.run.position, level.run.velocity);
        }
    }

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b1s_acoustic_substeps.v1\""
           << ",\"identity\":\"nuv-variational-fcr2\""
           << ",\"solver_identity\":\"nuv-newton-krylov-r0\""
           << ",\"parent_b1d1_result_sha256\":\""
           << "64592cd38c3371decd14baee6517e85bd0407e9654255e2eaab03fba8fd770c1\""
           << ",\"candidate\":\"acoustic-courant-substeps-r0\""
           << ",\"status\":\""
           << (diagnostic.passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << diagnostic.first_failure << '"'
           << ",\"frame_time\":0.0041666666666666666"
           << ",\"terminal_time\":0.012500000000000001"
           << ",\"target_acoustic_courant\":0.25"
           << ",\"thresholds\":{\"minimum_ratio\":1.5"
           << ",\"maximum_ratio\":2.5"
           << ",\"maximum_position_error_dx\":0.05"
           << ",\"maximum_velocity_error_c\":0.001"
           << ",\"maximum_relative_kinetic_error\":0.15"
           << ",\"maximum_policy_substeps\":96}"
           << ",\"matrix\":[";
    for (std::size_t case_index = 0;
         case_index < diagnostic.cases.size(); ++case_index) {
        if (case_index != 0) {
            report << ',';
        }
        const AcousticPolicyCase& value = diagnostic.cases[case_index];
        report << "{\"name\":\"compression-" << value.compression_factor
               << "-kappa-" << value.kappa_factor << "\",\"status\":\""
               << (value.passed ? "PASS" : "FAIL") << '"'
               << ",\"compression_factor\":" << value.compression_factor
               << ",\"kappa_factor\":" << value.kappa_factor
               << ",\"wave_speed\":" << value.wave_speed
               << ",\"initial_maximum_density_ratio\":"
               << value.initial_maximum_density_ratio
               << ",\"policy_substeps_per_frame\":"
               << value.policy_substeps
               << ",\"position_difference\":";
        append_double_array(report, value.position_difference.data(),
            value.position_difference.size());
        report << ",\"velocity_difference\":";
        append_double_array(report, value.velocity_difference.data(),
            value.velocity_difference.size());
        report << ",\"position_ratio\":" << value.position_ratio
               << ",\"velocity_ratio\":" << value.velocity_ratio
               << ",\"normalized_policy_position_error\":"
               << value.normalized_policy_position_error
               << ",\"normalized_policy_velocity_error\":"
               << value.normalized_policy_velocity_error
               << ",\"relative_kinetic_error\":"
               << value.relative_kinetic_error
               << ",\"pressure_exit_time_error\":"
               << value.pressure_exit_time_error
               << ",\"levels\":[";
        for (std::size_t level_index = 0;
             level_index < value.levels.size(); ++level_index) {
            if (level_index != 0) {
                report << ',';
            }
            const TemporalLevel& level = value.levels[level_index];
            report << "{\"multiple\":" << (1 << level_index)
                   << ",\"substeps_per_frame\":"
                   << value.policy_substeps * (1 << level_index)
                   << ",\"dt\":" << level.time_step
                   << ",\"acoustic_courant\":"
                   << level.acoustic_courant
                   << ",\"rms_radius\":" << level.rms_radius
                   << ",\"rms_relative_speed\":" << level.rms_speed
                   << ",\"relative_kinetic_energy\":"
                   << level.kinetic_energy
                   << ",\"pressure_active_exit_time\":"
                   << level.pressure_exit_time
                   << ",\"normalized_center_drift\":"
                   << level.normalized_center_drift
                   << ",\"work\":";
            append_multistep_work(report, level.run);
            report << '}';
        }
        report << "]}";
    }
    report << "],\"degenerate\":{\"status\":\""
           << (diagnostic.degenerate.passed ? "PASS" : "FAIL") << '"'
           << ",\"free_flight_substeps\":"
           << diagnostic.degenerate.free_flight_substeps
           << ",\"translation_substeps\":"
           << diagnostic.degenerate.translation_substeps
           << ",\"free_position_error\":"
           << diagnostic.degenerate.free_position_error
           << ",\"free_velocity_error\":"
           << diagnostic.degenerate.free_velocity_error
           << ",\"translation_position_error\":"
           << diagnostic.degenerate.translation_position_error
           << ",\"translation_velocity_error\":"
           << diagnostic.degenerate.translation_velocity_error
           << ",\"free_work\":";
    append_multistep_work(report, diagnostic.degenerate.free_flight);
    report << ",\"translation_work\":";
    append_multistep_work(report, diagnostic.degenerate.translation);
    report << "}"
           << ",\"base_material_substeps_per_frame\":34"
           << ",\"high_stiffness_substeps_per_frame\":67"
           << ",\"candidate_selected\":"
           << (diagnostic.passed ? "true" : "false")
           << ",\"b1r_design_authorized\":"
           << (diagnostic.passed ? "true" : "false")
           << ",\"boundary_design_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(result_material.str()) << "\"}";
    return {diagnostic.passed, report.str()};
}

ReferenceSolverReport run_pressure_tangent_spectrum_controls() {
    const PressureSpectrumDiagnostic diagnostic =
        pressure_spectrum_diagnostic();
    std::ostringstream result_material;
    result_material << std::setprecision(17)
                    << (diagnostic.passed ? "PASS|" : "FAIL|")
                    << diagnostic.first_failure << '|'
                    << diagnostic.dense.dense_maximum_eigenvalue << ':'
                    << diagnostic.dense.lanczos_maximum_eigenvalue << '|'
                    << diagnostic.amplification_099 << ':'
                    << diagnostic.amplification_098;
    for (const PressureSpectrumCase& value : diagnostic.cases) {
        result_material << '|' << value.compression_factor << ':'
                        << value.kappa_factor << ':'
                        << value.maximum_eigenvalue << ':'
                        << value.maximum_eigenfrequency << ':'
                        << value.spectral_amplification << ':'
                        << value.lanczos.relative_ritz_residual << ':'
                        << value.lanczos.operator_calls;
    }

    std::ostringstream report;
    report << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.nsr3b1s1_pressure_spectrum.v1\""
           << ",\"identity\":\"nuv-variational-fcr2\""
           << ",\"parent_b1s_result_sha256\":\""
           << "1537a691dfa821c6ccc7ce168c4d4c74868b10db95075a615eaf887f443eabee\""
           << ",\"candidate\":\"pressure-tangent-spectral-premise-r0\""
           << ",\"status\":\""
           << (diagnostic.passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << diagnostic.first_failure << '"'
           << ",\"thresholds\":{\"maximum_lanczos_iterations\":48"
           << ",\"maximum_ritz_residual\":1e-8"
           << ",\"maximum_orthogonality_error\":1e-10"
           << ",\"dense_eigenvalue_error\":1e-10"
           << ",\"dense_symmetry_error\":2e-12"
           << ",\"dense_product_error\":2e-12}"
           << ",\"dense_oracle\":{\"status\":\""
           << (diagnostic.dense.passed ? "PASS" : "FAIL") << '"'
           << ",\"dimension\":" << diagnostic.dense.dimension
           << ",\"active_pressure_centers\":"
           << diagnostic.dense.active_pressure_centers
           << ",\"dense_maximum_eigenvalue\":"
           << diagnostic.dense.dense_maximum_eigenvalue
           << ",\"lanczos_maximum_eigenvalue\":"
           << diagnostic.dense.lanczos_maximum_eigenvalue
           << ",\"eigenvalue_error\":"
           << diagnostic.dense.eigenvalue_error
           << ",\"symmetry_error\":"
           << diagnostic.dense.symmetry_error
           << ",\"dense_product_error\":"
           << diagnostic.dense.dense_product_error
           << ",\"lanczos_iterations\":"
           << diagnostic.dense.lanczos.iterations
           << ",\"lanczos_operator_calls\":"
           << diagnostic.dense.lanczos.operator_calls
           << ",\"ritz_residual\":"
           << diagnostic.dense.lanczos.relative_ritz_residual
           << ",\"orthogonality_error\":"
           << diagnostic.dense.lanczos.orthogonality_error << "}"
           << ",\"matrix\":[";
    for (std::size_t i = 0; i < diagnostic.cases.size(); ++i) {
        if (i != 0) {
            report << ',';
        }
        const PressureSpectrumCase& value = diagnostic.cases[i];
        report << "{\"name\":\"compression-" << value.compression_factor
               << "-kappa-" << value.kappa_factor << "\",\"status\":\""
               << (value.passed ? "PASS" : "FAIL") << '"'
               << ",\"compression_factor\":" << value.compression_factor
               << ",\"kappa_factor\":" << value.kappa_factor
               << ",\"particles\":" << value.particles
               << ",\"active_pressure_centers\":"
               << value.active_pressure_centers
               << ",\"pairs\":" << value.pairs
               << ",\"maximum_neighbors\":"
               << value.maximum_neighbors
               << ",\"maximum_eigenvalue\":"
               << value.maximum_eigenvalue
               << ",\"maximum_eigenfrequency\":"
               << value.maximum_eigenfrequency
               << ",\"spectral_amplification\":"
               << value.spectral_amplification
               << ",\"translation_null_residual\":"
               << value.translation_null_residual
               << ",\"lanczos_iterations\":"
               << value.lanczos.iterations
               << ",\"operator_calls\":"
               << value.lanczos.operator_calls
               << ",\"ritz_residual\":"
               << value.lanczos.relative_ritz_residual
               << ",\"orthogonality_error\":"
               << value.lanczos.orthogonality_error << '}';
    }
    report << "],\"kappa_scaling_valid\":"
           << (diagnostic.kappa_scaling_valid ? "true" : "false")
           << ",\"mean_spectral_amplification_0p99\":"
           << diagnostic.amplification_099
           << ",\"mean_spectral_amplification_0p98\":"
           << diagnostic.amplification_098
           << ",\"null_control\":{\"status\":\""
           << (diagnostic.null_control.passed ? "PASS" : "FAIL") << '"'
           << ",\"active_pressure_centers\":"
           << diagnostic.null_control.active_pressure_centers
           << ",\"hvp_norm\":" << diagnostic.null_control.hvp_norm
           << ",\"maximum_eigenvalue\":"
           << diagnostic.null_control.maximum_eigenvalue
           << ",\"operator_calls\":"
           << diagnostic.null_control.lanczos.operator_calls << "}"
           << ",\"disposition\":\""
           << (diagnostic.passed
                    ? "SPECTRAL_POLICY_PREMISE_VALID"
                    : "SPECTRAL_POLICY_PREMISE_REJECTED") << '"'
           << ",\"b1s2_design_authorized\":"
           << (diagnostic.passed ? "true" : "false")
           << ",\"trajectory_policy_selected\":false"
           << ",\"boundary_design_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"historical_hash_check_required\":true"
           << ",\"repeatability_check_required\":true"
           << ",\"result_sha256\":\""
           << sha256_hex(result_material.str()) << "\"}";
    return {diagnostic.passed, report.str()};
}

} // namespace nextengine::nonlocal::fcr
