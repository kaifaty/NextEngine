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

std::vector<double> densities(
    const Config& config, const std::vector<Vec3>& y) {
    std::vector<double> result(
        y.size(), config.mass * cubic_weight(0.0, config.horizon));
    for (std::size_t i = 0; i < y.size(); ++i) {
        for (std::size_t j = i + 1; j < y.size(); ++j) {
            const double radius = norm(y[i] - y[j]);
            if (radius <= config.horizon) {
                const double contribution =
                    config.mass * cubic_weight(radius, config.horizon);
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
                    * cubic_gradient(radius, config.horizon) * normal;
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
                    cubic_second_derivative(radius, config.horizon);
                const double tangential =
                    cubic_gradient(radius, config.horizon) / radius;
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
                * (-cubic_gradient(radius, config.horizon))
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
        y.size(), config.mass * cubic_weight(0.0, config.horizon));
    for (const ParticlePair pair : pairs) {
        const double radius = norm(y[pair.i] - y[pair.j]);
        if (radius <= config.horizon) {
            const double contribution =
                config.mass * cubic_weight(radius, config.horizon);
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
            * cubic_gradient(radius, config.horizon);
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
        const double omega = -cubic_gradient(radius, config.horizon);
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
                    * cubic_gradient(radius, config.horizon) * normal;
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
                        cubic_second_derivative(radius, config.horizon),
                        cubic_gradient(radius, config.horizon) / radius,
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
            * (-cubic_gradient(radius, config.horizon))
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
            * (-cubic_gradient(radius, config.horizon))
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
                * cubic_gradient(radius, config.horizon)
                * coefficient.normal;
            coefficient.density_radial =
                cubic_second_derivative(radius, config.horizon);
            coefficient.density_tangential =
                cubic_gradient(radius, config.horizon) / radius;
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
        y.size(), config.mass * cubic_weight(0.0, config.horizon));
    for (const CurrentPairHessianCoefficient& coefficient :
         tape.current_pairs) {
        if (!coefficient.density_active) {
            continue;
        }
        const double radius = norm(
            y[coefficient.pair.i] - y[coefficient.pair.j]);
        const double contribution =
            config.mass * cubic_weight(radius, config.horizon);
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
            config.mass * cubic_weight(0.0, config.horizon));
        for (const ParticlePair pair : current_pairs) {
            const double radius = norm(y[pair.i] - y[pair.j]);
            if (radius <= config.horizon) {
                const double contribution =
                    config.mass * cubic_weight(radius, config.horizon);
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
                    * cubic_gradient(radius, config.horizon) * normal;
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
                        cubic_second_derivative(radius, config.horizon),
                        cubic_gradient(radius, config.horizon) / radius,
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
            * (-cubic_gradient(radius, config.horizon))
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
                    * cubic_gradient(radius, config.horizon) / radius;
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
                    * (-cubic_gradient(radius, config.horizon))
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
    std::string& reason) {
    if (evaluation.gradient_norm <= 1.0e-10) {
        reason = "RAW_GRADIENT";
        return true;
    }
    if (scale_aware
        && scaled_displacement_residual(config, evaluation) <= 1.0e-8) {
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
    };
    std::vector<TrialTrace> trace;
};

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
    bool verify_tape_hvp = false) {
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
                config, current, true, result.solve.convergence_stop)) {
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
            config, current, true, result.solve.convergence_stop)
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

} // namespace nextengine::nonlocal::fcr
