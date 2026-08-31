#include "corrected_cuda_full_step.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <limits>
#include <numeric>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace {

#ifndef NCGP12_CONTRACT_ROOT
#define NCGP12_CONTRACT_ROOT "unconfigured"
#endif
#ifndef NCGP12_SOURCE_ROOT
#define NCGP12_SOURCE_ROOT "unconfigured"
#endif
#ifndef NCGP12_SOURCE_COMMIT
#define NCGP12_SOURCE_COMMIT "unconfigured"
#endif
#ifndef NCGP12_SOURCE_TREE
#define NCGP12_SOURCE_TREE "unconfigured"
#endif
#ifndef NCGP12_COMPILER_FLAGS
#define NCGP12_COMPILER_FLAGS "unconfigured"
#endif

using namespace nextengine::nonlocal::gpu_full_step;

constexpr std::size_t kNx = 8U;
constexpr std::size_t kNy = 8U;
constexpr std::size_t kNz = 8U;
constexpr std::size_t kMaximumSweeps = 4096U;
constexpr long double kJacobianLimit = 2.0e-7L;
constexpr long double kSymmetryLimit = 2.0e-12L;
constexpr long double kPrimalLimit = 1.0e-8L;
constexpr long double kKktLimit = 1.0e-8L;
constexpr long double kComplementarityLimit = 1.0e-10L;
constexpr long double kLinearizedLimit = 2.0e-12L;
constexpr long double kStationarityLimit = 1.0e-8L;
constexpr long double kMaximumStrainLimit = 1.0e-3L;
constexpr long double kRmsStrainLimit = 2.5e-4L;
constexpr long double kPermutationLimit = 2.0e-12L;

struct Vec3l {
    long double x = 0.0L;
    long double y = 0.0L;
    long double z = 0.0L;
};

Vec3l add(Vec3l lhs, Vec3l rhs) {
    return {lhs.x + rhs.x, lhs.y + rhs.y, lhs.z + rhs.z};
}

Vec3l subtract(Vec3l lhs, Vec3l rhs) {
    return {lhs.x - rhs.x, lhs.y - rhs.y, lhs.z - rhs.z};
}

Vec3l scale(Vec3l value, long double factor) {
    return {value.x * factor, value.y * factor, value.z * factor};
}

long double dot(Vec3l lhs, Vec3l rhs) {
    return lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z;
}

long double norm(Vec3l value) {
    return std::sqrt(std::max(dot(value, value), 0.0L));
}

Vec3l widen(Vec3d value) {
    return {static_cast<long double>(value.x),
        static_cast<long double>(value.y),
        static_cast<long double>(value.z)};
}

struct Kernel {
    long double value = 0.0L;
    long double first = 0.0L;
};

Kernel kernel(long double radius, const NonlocalGpuProfile& profile) {
    const long double pi = std::acos(-1.0L);
    const long double horizon = profile.horizon;
    const long double q = 2.0L * radius / horizon;
    const long double alpha = static_cast<long double>(profile.kernel_scale)
        * 3.0L / (2.0L * pi * horizon * horizon * horizon);
    long double value = 0.0L;
    long double first_q = 0.0L;
    if (q < 1.0L) {
        value = alpha * (2.0L / 3.0L - q * q + 0.5L * q * q * q);
        first_q = alpha * (-2.0L * q + 1.5L * q * q);
    } else if (q <= 2.0L) {
        const long double tail = 2.0L - q;
        value = alpha * tail * tail * tail / 6.0L;
        first_q = -0.5L * alpha * tail * tail;
    }
    return {value, first_q * 2.0L / horizon};
}

void append_u64(std::string& bytes, std::uint64_t value) {
    for (std::uint32_t index = 0U; index < 8U; ++index) {
        bytes.push_back(static_cast<char>((value >> (8U * index)) & 0xffU));
    }
}

void append_f64(std::string& bytes, double value) {
    std::uint64_t bits = 0U;
    static_assert(sizeof(bits) == sizeof(value));
    std::memcpy(&bits, &value, sizeof(bits));
    append_u64(bytes, bits);
}

void append_string(std::string& bytes, std::string_view value) {
    append_u64(bytes, value.size());
    bytes.append(value.data(), value.size());
}

std::string file_root(const std::string& path) {
    std::ifstream stream(path, std::ios::binary);
    if (!stream) return {};
    std::ostringstream bytes;
    bytes << stream.rdbuf();
    if (!stream.good() && !stream.eof()) return {};
    return nextengine::nonlocal::sha256_hex(bytes.str());
}

std::string binary_root() {
    return file_root("/proc/self/exe");
}

long double vector_norm(const std::vector<long double>& values) {
    long double sum = 0.0L;
    for (const long double value : values) sum += value * value;
    return std::sqrt(std::max(sum, 0.0L));
}

std::vector<NonlocalGpuSample> canonical_state(
    const NonlocalGpuProfile& profile, bool permuted) {
    auto state = make_lattice_state(profile, static_cast<std::uint32_t>(kNx),
        static_cast<std::uint32_t>(kNy), static_cast<std::uint32_t>(kNz),
        permuted, false);
    std::sort(state.begin(), state.end(),
        [](const NonlocalGpuSample& lhs, const NonlocalGpuSample& rhs) {
            return lhs.sample_id < rhs.sample_id;
        });
    return state;
}

std::vector<NonlocalGpuGhost> canonical_ghosts(
    const NonlocalGpuProfile& profile) {
    auto ghosts = make_basin_ghosts(profile);
    std::sort(ghosts.begin(), ghosts.end(),
        [](const NonlocalGpuGhost& lhs, const NonlocalGpuGhost& rhs) {
            return lhs.sample_id < rhs.sample_id;
        });
    return ghosts;
}

struct Work {
    std::uint64_t density_candidates = 0U;
    std::uint64_t accepted_pairs = 0U;
    std::uint64_t derivative_pairs = 0U;
    std::uint64_t matrix_products = 0U;
    std::uint64_t coordinate_sweeps = 0U;
    std::uint64_t coordinate_updates = 0U;
    std::uint64_t gradient_recomputations = 0U;
    std::uint64_t finite_difference_density_candidates = 0U;
    std::uint64_t control_density_candidates = 0U;
};

void add_work(Work& target, const Work& source) {
    target.density_candidates += source.density_candidates;
    target.accepted_pairs += source.accepted_pairs;
    target.derivative_pairs += source.derivative_pairs;
    target.matrix_products += source.matrix_products;
    target.coordinate_sweeps += source.coordinate_sweeps;
    target.coordinate_updates += source.coordinate_updates;
    target.gradient_recomputations += source.gradient_recomputations;
    target.finite_difference_density_candidates +=
        source.finite_difference_density_candidates;
    target.control_density_candidates += source.control_density_candidates;
}

std::string work_root(const Work& work) {
    std::string bytes = "nextengine.nonlocal.ncgp12.work.v1\0";
    for (const std::uint64_t value : std::array<std::uint64_t, 9>{
             work.density_candidates, work.accepted_pairs,
             work.derivative_pairs, work.matrix_products,
             work.coordinate_sweeps, work.coordinate_updates,
             work.gradient_recomputations,
             work.finite_difference_density_candidates,
             work.control_density_candidates}) {
        append_u64(bytes, value);
    }
    return nextengine::nonlocal::sha256_hex(bytes);
}

struct DensityResult {
    bool valid = true;
    std::vector<long double> density;
    std::uint64_t candidates = 0U;
};

DensityResult density_only(const NonlocalGpuProfile& profile,
    const std::vector<Vec3l>& positions,
    const std::vector<NonlocalGpuGhost>& ghosts) {
    DensityResult result;
    result.density.assign(positions.size(), 0.0L);
    const long double horizon = profile.horizon;
    const long double mass = profile.mass;
    for (std::size_t owner = 0U; owner < positions.size(); ++owner) {
        for (const Vec3l candidate : positions) {
            ++result.candidates;
            const long double radius = norm(subtract(positions[owner], candidate));
            if (radius <= horizon) {
                result.density[owner] += mass * kernel(radius, profile).value;
            }
        }
        for (const NonlocalGpuGhost& ghost : ghosts) {
            ++result.candidates;
            const long double radius = norm(subtract(
                positions[owner], widen(ghost.position)));
            if (radius <= horizon) {
                result.density[owner] += mass * kernel(radius, profile).value;
            }
        }
        result.valid = result.valid && std::isfinite(result.density[owner]);
    }
    return result;
}

struct Assembly {
    bool valid = true;
    std::size_t count = 0U;
    std::size_t dofs = 0U;
    std::vector<long double> density;
    std::vector<long double> constraint;
    std::vector<long double> jacobian;
    std::vector<long double> matrix;
    std::vector<long double> right_hand_side;
    long double symmetry_error = std::numeric_limits<long double>::infinity();
    Work work;
};

void add_jacobian_component(std::vector<long double>& jacobian,
    std::size_t dofs, std::size_t row, std::size_t sample, Vec3l value,
    long double sign) {
    const std::size_t base = row * dofs + 3U * sample;
    jacobian[base] += sign * value.x;
    jacobian[base + 1U] += sign * value.y;
    jacobian[base + 2U] += sign * value.z;
}

Assembly assemble(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& state,
    const std::vector<NonlocalGpuGhost>& ghosts,
    bool include_ghost_derivatives) {
    Assembly result;
    result.count = state.size();
    result.dofs = 3U * result.count;
    result.density.assign(result.count, 0.0L);
    result.constraint.assign(result.count, 0.0L);
    result.jacobian.assign(result.count * result.dofs, 0.0L);
    result.matrix.assign(result.count * result.count, 0.0L);
    result.right_hand_side.assign(result.count, 0.0L);
    std::vector<Vec3l> positions;
    positions.reserve(result.count);
    for (const NonlocalGpuSample& sample : state) {
        positions.push_back(widen(sample.current));
    }
    const long double horizon = profile.horizon;
    const long double mass = profile.mass;
    const long double rho0 = profile.rest_density;
    for (std::size_t owner = 0U; owner < result.count; ++owner) {
        for (std::size_t neighbor = 0U; neighbor < result.count; ++neighbor) {
            ++result.work.density_candidates;
            const Vec3l difference = subtract(positions[owner], positions[neighbor]);
            const long double radius = norm(difference);
            if (radius > horizon) continue;
            ++result.work.accepted_pairs;
            const Kernel values = kernel(radius, profile);
            result.density[owner] += mass * values.value;
            if (neighbor == owner || !(radius > 0.0L)) continue;
            ++result.work.derivative_pairs;
            const Vec3l derivative = scale(difference,
                mass / rho0 * values.first / radius);
            add_jacobian_component(result.jacobian, result.dofs, owner,
                owner, derivative, 1.0L);
            add_jacobian_component(result.jacobian, result.dofs, owner,
                neighbor, derivative, -1.0L);
        }
        for (const NonlocalGpuGhost& ghost : ghosts) {
            ++result.work.density_candidates;
            const Vec3l difference = subtract(positions[owner], widen(ghost.position));
            const long double radius = norm(difference);
            if (radius > horizon) continue;
            ++result.work.accepted_pairs;
            const Kernel values = kernel(radius, profile);
            result.density[owner] += mass * values.value;
            if (!include_ghost_derivatives || !(radius > 0.0L)) continue;
            ++result.work.derivative_pairs;
            const Vec3l derivative = scale(difference,
                mass / rho0 * values.first / radius);
            add_jacobian_component(result.jacobian, result.dofs, owner,
                owner, derivative, 1.0L);
        }
        result.constraint[owner] = result.density[owner] / rho0 - 1.0L;
    }

    const long double dt2 = static_cast<long double>(profile.dt) * profile.dt;
    const long double matrix_scale = dt2 / mass;
    for (std::size_t column = 0U; column < result.dofs; ++column) {
        std::vector<std::pair<std::size_t, long double>> entries;
        for (std::size_t row = 0U; row < result.count; ++row) {
            const long double value = result.jacobian[row * result.dofs + column];
            if (value != 0.0L) entries.emplace_back(row, value);
        }
        for (const auto& lhs : entries) {
            for (const auto& rhs : entries) {
                result.matrix[lhs.first * result.count + rhs.first] +=
                    matrix_scale * lhs.second * rhs.second;
                ++result.work.matrix_products;
            }
        }
    }

    std::vector<long double> gravity_displacement(result.dofs, 0.0L);
    for (std::size_t sample = 0U; sample < result.count; ++sample) {
        gravity_displacement[3U * sample] = dt2 * profile.gravity.x;
        gravity_displacement[3U * sample + 1U] = dt2 * profile.gravity.y;
        gravity_displacement[3U * sample + 2U] = dt2 * profile.gravity.z;
    }
    for (std::size_t row = 0U; row < result.count; ++row) {
        long double projected = result.constraint[row];
        for (std::size_t column = 0U; column < result.dofs; ++column) {
            projected += result.jacobian[row * result.dofs + column]
                * gravity_displacement[column];
        }
        result.right_hand_side[row] = projected;
    }

    long double symmetry_difference = 0.0L;
    long double symmetry_reference = 0.0L;
    for (std::size_t row = 0U; row < result.count; ++row) {
        const long double diagonal = result.matrix[row * result.count + row];
        result.valid = result.valid && std::isfinite(result.density[row])
            && std::isfinite(result.constraint[row])
            && std::isfinite(result.right_hand_side[row])
            && std::isfinite(diagonal) && diagonal > 0.0L;
        for (std::size_t column = 0U; column < result.count; ++column) {
            const long double lhs = result.matrix[row * result.count + column];
            const long double rhs = result.matrix[column * result.count + row];
            symmetry_difference += (lhs - rhs) * (lhs - rhs);
            symmetry_reference += lhs * lhs + rhs * rhs;
            result.valid = result.valid && std::isfinite(lhs);
        }
    }
    result.symmetry_error = std::sqrt(symmetry_difference)
        / std::max(std::sqrt(0.5L * symmetry_reference), 1.0e-30L);
    return result;
}

struct Solve {
    bool valid = true;
    bool converged = false;
    std::vector<long double> multiplier;
    std::vector<long double> gradient;
    std::size_t sweeps = 0U;
    std::uint64_t coordinate_updates = 0U;
    long double primal_residual = std::numeric_limits<long double>::infinity();
    long double projected_kkt = std::numeric_limits<long double>::infinity();
    long double complementarity = std::numeric_limits<long double>::infinity();
};

void recompute_gradient(const Assembly& assembly, Solve& solve) {
    for (std::size_t row = 0U; row < assembly.count; ++row) {
        long double value = -assembly.right_hand_side[row];
        for (std::size_t column = 0U; column < assembly.count; ++column) {
            value += assembly.matrix[row * assembly.count + column]
                * solve.multiplier[column];
        }
        solve.gradient[row] = value;
    }
}

void update_residuals(const Assembly& assembly, Solve& solve) {
    std::vector<long double> positive_violation(assembly.count, 0.0L);
    std::vector<long double> source_violation(assembly.count, 0.0L);
    std::vector<long double> projected(assembly.count, 0.0L);
    solve.complementarity = 0.0L;
    for (std::size_t row = 0U; row < assembly.count; ++row) {
        positive_violation[row] = std::max(-solve.gradient[row], 0.0L);
        source_violation[row] = std::max(assembly.right_hand_side[row], 0.0L);
        const long double diagonal = assembly.matrix[row * assembly.count + row];
        projected[row] = std::min(diagonal * solve.multiplier[row],
            solve.gradient[row]);
        solve.complementarity = std::max(solve.complementarity,
            std::abs(solve.multiplier[row] * solve.gradient[row]));
    }
    solve.primal_residual = vector_norm(positive_violation)
        / std::max(vector_norm(source_violation), 1.0e-30L);
    solve.projected_kkt = vector_norm(projected)
        / std::max(vector_norm(assembly.right_hand_side), 1.0e-30L);
}

Solve solve_pressure(const Assembly& assembly, Work& work) {
    Solve solve;
    solve.multiplier.assign(assembly.count, 0.0L);
    solve.gradient.resize(assembly.count);
    recompute_gradient(assembly, solve);
    ++work.gradient_recomputations;
    for (std::size_t sweep = 1U; sweep <= kMaximumSweeps; ++sweep) {
        for (std::size_t row = 0U; row < assembly.count; ++row) {
            const long double diagonal = assembly.matrix[row * assembly.count + row];
            if (!(diagonal > 0.0L) || !std::isfinite(diagonal)) {
                solve.valid = false;
                return solve;
            }
            const long double prior = solve.multiplier[row];
            const long double next = std::max(0.0L,
                prior - solve.gradient[row] / diagonal);
            const long double delta = next - prior;
            if (delta == 0.0L) continue;
            solve.multiplier[row] = next;
            ++solve.coordinate_updates;
            for (std::size_t affected = 0U; affected < assembly.count; ++affected) {
                solve.gradient[affected] +=
                    assembly.matrix[affected * assembly.count + row] * delta;
            }
        }
        solve.sweeps = sweep;
        recompute_gradient(assembly, solve);
        ++work.gradient_recomputations;
        update_residuals(assembly, solve);
        solve.valid = solve.valid && std::isfinite(solve.primal_residual)
            && std::isfinite(solve.projected_kkt)
            && std::isfinite(solve.complementarity)
            && std::all_of(solve.multiplier.begin(), solve.multiplier.end(),
                [](long double value) {
                    return std::isfinite(value) && value >= 0.0L;
                });
        if (!solve.valid) return solve;
        if (solve.primal_residual <= kPrimalLimit
            && solve.projected_kkt <= kKktLimit
            && solve.complementarity <= kComplementarityLimit) {
            solve.converged = true;
            break;
        }
    }
    work.coordinate_sweeps += solve.sweeps;
    work.coordinate_updates += solve.coordinate_updates;
    return solve;
}

std::vector<long double> jacobian_times(const Assembly& assembly,
    const std::vector<long double>& vector) {
    std::vector<long double> result(assembly.count, 0.0L);
    for (std::size_t row = 0U; row < assembly.count; ++row) {
        for (std::size_t column = 0U; column < assembly.dofs; ++column) {
            result[row] += assembly.jacobian[row * assembly.dofs + column]
                * vector[column];
        }
    }
    return result;
}

std::vector<long double> jacobian_transpose_times(const Assembly& assembly,
    const std::vector<long double>& vector) {
    std::vector<long double> result(assembly.dofs, 0.0L);
    for (std::size_t row = 0U; row < assembly.count; ++row) {
        for (std::size_t column = 0U; column < assembly.dofs; ++column) {
            result[column] += assembly.jacobian[row * assembly.dofs + column]
                * vector[row];
        }
    }
    return result;
}

struct Metrics {
    bool valid = true;
    bool solver_converged = false;
    std::size_t sweeps = 0U;
    std::uint64_t coordinate_updates = 0U;
    std::size_t positive_multipliers = 0U;
    long double jacobian_error = std::numeric_limits<long double>::infinity();
    long double symmetry_error = std::numeric_limits<long double>::infinity();
    long double primal_residual = std::numeric_limits<long double>::infinity();
    long double projected_kkt = std::numeric_limits<long double>::infinity();
    long double complementarity = std::numeric_limits<long double>::infinity();
    long double bottom_median = 0.0L;
    long double top_median = 0.0L;
    long double linearized_error = std::numeric_limits<long double>::infinity();
    long double maximum_positive_strain = std::numeric_limits<long double>::infinity();
    long double rms_positive_strain = std::numeric_limits<long double>::infinity();
    long double stationarity = std::numeric_limits<long double>::infinity();
    long double maximum_penetration = std::numeric_limits<long double>::infinity();
    long double zero_control_ratio = 0.0L;
    long double negated_maximum_strain = 0.0L;
    long double negated_rms_strain = 0.0L;
    long double negated_penetration = 0.0L;
    std::vector<long double> multiplier;
    std::vector<Vec3l> trial;
    std::string multiplier_root;
    std::string trial_root;
    std::string result_root;
    Work work;
};

long double median(std::vector<long double> values) {
    if (values.empty()) return 0.0L;
    std::sort(values.begin(), values.end());
    const std::size_t middle = values.size() / 2U;
    if ((values.size() & 1U) != 0U) return values[middle];
    return 0.5L * (values[middle - 1U] + values[middle]);
}

std::string vector_root(std::string_view domain,
    const std::vector<long double>& values) {
    std::string bytes;
    append_string(bytes, domain);
    append_u64(bytes, values.size());
    for (const long double value : values) append_f64(bytes, static_cast<double>(value));
    return nextengine::nonlocal::sha256_hex(bytes);
}

std::string position_root(std::string_view domain,
    const std::vector<Vec3l>& values) {
    std::string bytes;
    append_string(bytes, domain);
    append_u64(bytes, values.size());
    for (const Vec3l value : values) {
        append_f64(bytes, static_cast<double>(value.x));
        append_f64(bytes, static_cast<double>(value.y));
        append_f64(bytes, static_cast<double>(value.z));
    }
    return nextengine::nonlocal::sha256_hex(bytes);
}

std::pair<long double, long double> positive_strain_metrics(
    const std::vector<long double>& density, long double rho0) {
    long double maximum = 0.0L;
    long double squared = 0.0L;
    for (const long double value : density) {
        const long double positive = std::max(value / rho0 - 1.0L, 0.0L);
        maximum = std::max(maximum, positive);
        squared += positive * positive;
    }
    return {maximum, std::sqrt(squared / density.size())};
}

long double maximum_penetration(const NonlocalGpuProfile& profile,
    const std::vector<Vec3l>& positions) {
    const long double low = 0.5L * profile.spacing;
    const Vec3l high{profile.basin_extent.x - low,
        profile.basin_extent.y - low, profile.basin_extent.z - low};
    long double maximum = 0.0L;
    for (const Vec3l value : positions) {
        maximum = std::max(maximum, low - value.x);
        maximum = std::max(maximum, low - value.y);
        maximum = std::max(maximum, low - value.z);
        maximum = std::max(maximum, value.x - high.x);
        maximum = std::max(maximum, value.y - high.y);
        maximum = std::max(maximum, value.z - high.z);
    }
    return std::max(maximum, 0.0L);
}

long double jacobian_correspondence(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& state,
    const std::vector<NonlocalGpuGhost>& ghosts,
    const Assembly& assembly, Work& work) {
    std::vector<long double> direction(assembly.dofs, 0.0L);
    for (std::size_t index = 0U; index < state.size(); ++index) {
        const std::int64_t id = state[index].sample_id;
        direction[3U * index] = static_cast<long double>(id % 17 - 8);
        direction[3U * index + 1U] = static_cast<long double>(id % 13 - 6);
        direction[3U * index + 2U] = static_cast<long double>(id % 11 - 5);
    }
    const long double direction_norm = vector_norm(direction);
    for (long double& value : direction) value /= direction_norm;
    const std::vector<long double> analytic = jacobian_times(assembly, direction);
    const long double epsilon = std::ldexp(
        static_cast<long double>(profile.spacing), -20);
    std::vector<Vec3l> plus;
    std::vector<Vec3l> minus;
    plus.reserve(state.size());
    minus.reserve(state.size());
    for (std::size_t index = 0U; index < state.size(); ++index) {
        const Vec3l current = widen(state[index].current);
        const Vec3l delta{epsilon * direction[3U * index],
            epsilon * direction[3U * index + 1U],
            epsilon * direction[3U * index + 2U]};
        plus.push_back(add(current, delta));
        minus.push_back(subtract(current, delta));
    }
    const DensityResult plus_density = density_only(profile, plus, ghosts);
    const DensityResult minus_density = density_only(profile, minus, ghosts);
    work.finite_difference_density_candidates +=
        plus_density.candidates + minus_density.candidates;
    if (!plus_density.valid || !minus_density.valid) {
        return std::numeric_limits<long double>::infinity();
    }
    std::vector<long double> finite_difference(assembly.count, 0.0L);
    std::vector<long double> difference(assembly.count, 0.0L);
    for (std::size_t row = 0U; row < assembly.count; ++row) {
        finite_difference[row] = (plus_density.density[row]
            - minus_density.density[row])
            / (2.0L * epsilon * profile.rest_density);
        difference[row] = analytic[row] - finite_difference[row];
    }
    return vector_norm(difference)
        / std::max({vector_norm(analytic), vector_norm(finite_difference),
            1.0e-30L});
}

std::string metrics_root(const Metrics& metrics) {
    std::string bytes = "nextengine.nonlocal.ncgp12.metrics.v1\0";
    append_u64(bytes, metrics.valid ? 1U : 0U);
    append_u64(bytes, metrics.solver_converged ? 1U : 0U);
    append_u64(bytes, metrics.sweeps);
    append_u64(bytes, metrics.coordinate_updates);
    append_u64(bytes, metrics.positive_multipliers);
    for (const long double value : std::array<long double, 16>{
             metrics.jacobian_error, metrics.symmetry_error,
             metrics.primal_residual, metrics.projected_kkt,
             metrics.complementarity, metrics.bottom_median,
             metrics.top_median, metrics.linearized_error,
             metrics.maximum_positive_strain, metrics.rms_positive_strain,
             metrics.stationarity, metrics.maximum_penetration,
             metrics.zero_control_ratio, metrics.negated_maximum_strain,
             metrics.negated_rms_strain, metrics.negated_penetration}) {
        append_f64(bytes, static_cast<double>(value));
    }
    append_string(bytes, metrics.multiplier_root);
    append_string(bytes, metrics.trial_root);
    append_string(bytes, work_root(metrics.work));
    return nextengine::nonlocal::sha256_hex(bytes);
}

Metrics run_case(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& state,
    const std::vector<NonlocalGpuGhost>& ghosts,
    bool include_ghost_derivatives,
    bool run_derivative_control,
    bool run_negative_control) {
    Metrics metrics;
    const Assembly assembly = assemble(
        profile, state, ghosts, include_ghost_derivatives);
    add_work(metrics.work, assembly.work);
    metrics.valid = assembly.valid;
    metrics.symmetry_error = assembly.symmetry_error;
    metrics.jacobian_error = run_derivative_control
        ? jacobian_correspondence(profile, state, ghosts, assembly, metrics.work)
        : 0.0L;
    Solve solve = solve_pressure(assembly, metrics.work);
    metrics.valid = metrics.valid && solve.valid;
    metrics.solver_converged = solve.converged;
    metrics.sweeps = solve.sweeps;
    metrics.coordinate_updates = solve.coordinate_updates;
    metrics.primal_residual = solve.primal_residual;
    metrics.projected_kkt = solve.projected_kkt;
    metrics.complementarity = solve.complementarity;
    metrics.multiplier = solve.multiplier;
    metrics.positive_multipliers = static_cast<std::size_t>(std::count_if(
        solve.multiplier.begin(), solve.multiplier.end(),
        [](long double value) { return value > 0.0L; }));

    std::vector<long double> bottom;
    std::vector<long double> top;
    for (std::size_t index = 0U; index < state.size(); ++index) {
        const std::size_t layer = index / (kNx * kNy);
        if (layer == 0U) bottom.push_back(solve.multiplier[index]);
        if (layer + 1U == kNz) top.push_back(solve.multiplier[index]);
    }
    metrics.bottom_median = median(std::move(bottom));
    metrics.top_median = median(std::move(top));

    const std::vector<long double> jt_lambda =
        jacobian_transpose_times(assembly, solve.multiplier);
    const long double dt2 = static_cast<long double>(profile.dt) * profile.dt;
    const long double correction_scale = dt2 / profile.mass;
    std::vector<long double> displacement(assembly.dofs, 0.0L);
    std::vector<long double> negated_displacement(assembly.dofs, 0.0L);
    metrics.trial.reserve(state.size());
    std::vector<Vec3l> negated_trial;
    negated_trial.reserve(state.size());
    for (std::size_t index = 0U; index < state.size(); ++index) {
        const Vec3l gravity{dt2 * profile.gravity.x, dt2 * profile.gravity.y,
            dt2 * profile.gravity.z};
        const Vec3l correction{correction_scale * jt_lambda[3U * index],
            correction_scale * jt_lambda[3U * index + 1U],
            correction_scale * jt_lambda[3U * index + 2U]};
        const Vec3l step = subtract(gravity, correction);
        const Vec3l wrong_step = add(gravity, correction);
        displacement[3U * index] = step.x;
        displacement[3U * index + 1U] = step.y;
        displacement[3U * index + 2U] = step.z;
        negated_displacement[3U * index] = wrong_step.x;
        negated_displacement[3U * index + 1U] = wrong_step.y;
        negated_displacement[3U * index + 2U] = wrong_step.z;
        metrics.trial.push_back(add(widen(state[index].current), step));
        negated_trial.push_back(add(widen(state[index].current), wrong_step));
    }

    const std::vector<long double> linearized =
        jacobian_times(assembly, displacement);
    std::vector<long double> dual_constraint(assembly.count, 0.0L);
    std::vector<long double> linear_difference(assembly.count, 0.0L);
    for (std::size_t row = 0U; row < assembly.count; ++row) {
        long double matrix_action = 0.0L;
        for (std::size_t column = 0U; column < assembly.count; ++column) {
            matrix_action += assembly.matrix[row * assembly.count + column]
                * solve.multiplier[column];
        }
        dual_constraint[row] = assembly.right_hand_side[row] - matrix_action;
        linear_difference[row] = assembly.constraint[row] + linearized[row]
            - dual_constraint[row];
    }
    metrics.linearized_error = vector_norm(linear_difference)
        / std::max(vector_norm(dual_constraint), 1.0e-30L);

    const DensityResult trial_density = density_only(profile, metrics.trial, ghosts);
    metrics.work.control_density_candidates += trial_density.candidates;
    metrics.valid = metrics.valid && trial_density.valid;
    const auto strains = positive_strain_metrics(
        trial_density.density, profile.rest_density);
    metrics.maximum_positive_strain = strains.first;
    metrics.rms_positive_strain = strains.second;
    metrics.maximum_penetration = maximum_penetration(profile, metrics.trial);

    std::vector<long double> stationarity(assembly.dofs, 0.0L);
    std::vector<long double> gravity_force(assembly.dofs, 0.0L);
    for (std::size_t index = 0U; index < state.size(); ++index) {
        for (std::size_t component = 0U; component < 3U; ++component) {
            const std::size_t dof = 3U * index + component;
            const long double gravity_component = component == 0U
                ? profile.gravity.x : component == 1U
                ? profile.gravity.y : profile.gravity.z;
            stationarity[dof] = profile.mass / dt2
                    * (displacement[dof] - dt2 * gravity_component)
                + jt_lambda[dof];
            gravity_force[dof] = profile.mass * gravity_component;
        }
    }
    metrics.stationarity = vector_norm(stationarity)
        / std::max(vector_norm(gravity_force), 1.0e-30L);

    std::vector<long double> source_violation(assembly.count, 0.0L);
    for (std::size_t row = 0U; row < assembly.count; ++row) {
        source_violation[row] = std::max(assembly.right_hand_side[row], 0.0L);
    }
    metrics.zero_control_ratio = vector_norm(source_violation)
        / std::max(vector_norm(source_violation), 1.0e-30L);

    if (run_negative_control) {
        const DensityResult negated_density = density_only(profile, negated_trial, ghosts);
        metrics.work.control_density_candidates += negated_density.candidates;
        metrics.valid = metrics.valid && negated_density.valid;
        const auto negated_strains = positive_strain_metrics(
            negated_density.density, profile.rest_density);
        metrics.negated_maximum_strain = negated_strains.first;
        metrics.negated_rms_strain = negated_strains.second;
        metrics.negated_penetration = maximum_penetration(profile, negated_trial);
    }

    metrics.multiplier_root = vector_root(
        "nextengine.nonlocal.ncgp12.multiplier.v1", metrics.multiplier);
    metrics.trial_root = position_root(
        "nextengine.nonlocal.ncgp12.trial.v1", metrics.trial);
    metrics.result_root = metrics_root(metrics);
    return metrics;
}

bool scalar_close(long double lhs, long double rhs) {
    return std::abs(lhs - rhs)
        <= kPermutationLimit * std::max({std::abs(lhs), std::abs(rhs), 1.0L});
}

bool core_gates(const Metrics& metrics) {
    return metrics.valid && metrics.solver_converged
        && metrics.jacobian_error <= kJacobianLimit
        && metrics.symmetry_error <= kSymmetryLimit
        && metrics.primal_residual <= kPrimalLimit
        && metrics.projected_kkt <= kKktLimit
        && metrics.complementarity <= kComplementarityLimit
        && metrics.positive_multipliers > 0U
        && metrics.bottom_median > metrics.top_median
        && metrics.linearized_error <= kLinearizedLimit
        && metrics.stationarity <= kStationarityLimit;
}

bool nonlinear_gates(const Metrics& metrics) {
    return metrics.maximum_positive_strain <= kMaximumStrainLimit
        && metrics.rms_positive_strain <= kRmsStrainLimit
        && metrics.maximum_penetration == 0.0L;
}

std::string final_root(std::string_view classification,
    const Metrics& candidate, const Metrics& permuted,
    const Metrics& omitted, bool zero_control, bool negated_control,
    bool omitted_control, bool permutation_control, bool mutation_control,
    std::string_view input_root, std::string_view binary) {
    std::string bytes = "nextengine.nonlocal.ncgp12.result.v1\0";
    append_string(bytes, NCGP12_CONTRACT_ROOT);
    append_string(bytes, NCGP12_SOURCE_ROOT);
    append_string(bytes, NCGP12_SOURCE_COMMIT);
    append_string(bytes, NCGP12_SOURCE_TREE);
    append_string(bytes, input_root);
    append_string(bytes, binary);
    append_string(bytes, classification);
    append_string(bytes, candidate.result_root);
    append_string(bytes, permuted.result_root);
    append_string(bytes, omitted.result_root);
    append_u64(bytes, zero_control ? 1U : 0U);
    append_u64(bytes, negated_control ? 1U : 0U);
    append_u64(bytes, omitted_control ? 1U : 0U);
    append_u64(bytes, permutation_control ? 1U : 0U);
    append_u64(bytes, mutation_control ? 1U : 0U);
    return nextengine::nonlocal::sha256_hex(bytes);
}

void emit_metrics(std::ostream& output, const Metrics& metrics,
    const NonlocalGpuProfile& profile) {
    output << "{\"valid\":" << (metrics.valid ? "true" : "false")
           << ",\"solver_converged\":"
           << (metrics.solver_converged ? "true" : "false")
           << ",\"sweeps\":" << metrics.sweeps
           << ",\"coordinate_updates\":" << metrics.coordinate_updates
           << ",\"positive_multipliers\":" << metrics.positive_multipliers
           << ",\"jacobian_relative_l2\":"
           << static_cast<double>(metrics.jacobian_error)
           << ",\"matrix_symmetry_relative_l2\":"
           << static_cast<double>(metrics.symmetry_error)
           << ",\"primal_residual\":"
           << static_cast<double>(metrics.primal_residual)
           << ",\"projected_kkt\":"
           << static_cast<double>(metrics.projected_kkt)
           << ",\"complementarity_j\":"
           << static_cast<double>(metrics.complementarity)
           << ",\"bottom_median_lambda_j\":"
           << static_cast<double>(metrics.bottom_median)
           << ",\"top_median_lambda_j\":"
           << static_cast<double>(metrics.top_median)
           << ",\"bottom_median_pressure_pa\":"
           << static_cast<double>(profile.rest_density / profile.mass
               * metrics.bottom_median)
           << ",\"top_median_pressure_pa\":"
           << static_cast<double>(profile.rest_density / profile.mass
               * metrics.top_median)
           << ",\"linearized_relative_l2\":"
           << static_cast<double>(metrics.linearized_error)
           << ",\"maximum_positive_strain\":"
           << static_cast<double>(metrics.maximum_positive_strain)
           << ",\"rms_positive_strain\":"
           << static_cast<double>(metrics.rms_positive_strain)
           << ",\"stationarity_residual\":"
           << static_cast<double>(metrics.stationarity)
           << ",\"maximum_penetration_m\":"
           << static_cast<double>(metrics.maximum_penetration)
           << ",\"zero_control_ratio\":"
           << static_cast<double>(metrics.zero_control_ratio)
           << ",\"negated_maximum_strain\":"
           << static_cast<double>(metrics.negated_maximum_strain)
           << ",\"negated_rms_strain\":"
           << static_cast<double>(metrics.negated_rms_strain)
           << ",\"negated_penetration_m\":"
           << static_cast<double>(metrics.negated_penetration)
           << ",\"multiplier_root\":\"" << metrics.multiplier_root
           << "\",\"trial_root\":\"" << metrics.trial_root
           << "\",\"work_root\":\"" << work_root(metrics.work)
           << "\",\"result_root\":\"" << metrics.result_root
           << "\",\"work\":{\"density_candidates\":"
           << metrics.work.density_candidates
           << ",\"accepted_pairs\":" << metrics.work.accepted_pairs
           << ",\"derivative_pairs\":" << metrics.work.derivative_pairs
           << ",\"matrix_products\":" << metrics.work.matrix_products
           << ",\"coordinate_sweeps\":" << metrics.work.coordinate_sweeps
           << ",\"coordinate_updates\":" << metrics.work.coordinate_updates
           << ",\"gradient_recomputations\":"
           << metrics.work.gradient_recomputations
           << ",\"finite_difference_density_candidates\":"
           << metrics.work.finite_difference_density_candidates
           << ",\"control_density_candidates\":"
           << metrics.work.control_density_candidates << "}}";
}

int run() {
    NonlocalGpuProfile profile = nonlocal_water_corrected_profile();
    profile.lambda = 0.0;
    profile.mu = 0.0;
    profile.gamma = 0.0;
    const auto state = canonical_state(profile, false);
    const auto permuted_state = canonical_state(profile, true);
    const auto ghosts = canonical_ghosts(profile);
    if (validate_nonlocal_input(profile, state, ghosts)
            != NonlocalGpuFailure::None
        || validate_nonlocal_input(profile, permuted_state, ghosts)
            != NonlocalGpuFailure::None) {
        throw std::runtime_error("NCGP12 input admission failed");
    }

    Metrics candidate = run_case(profile, state, ghosts, true, true, true);
    Metrics permuted = run_case(
        profile, permuted_state, ghosts, true, true, true);
    Metrics omitted = run_case(profile, state, ghosts, false, false, false);

    const bool zero_control = candidate.zero_control_ratio >= 0.99L;
    const bool negated_control =
        candidate.negated_maximum_strain > kMaximumStrainLimit
        || candidate.negated_rms_strain > kRmsStrainLimit;
    const bool omitted_control = !core_gates(omitted)
        || !nonlinear_gates(omitted);
    const bool permutation_metrics =
        scalar_close(candidate.jacobian_error, permuted.jacobian_error)
        && scalar_close(candidate.symmetry_error, permuted.symmetry_error)
        && scalar_close(candidate.primal_residual, permuted.primal_residual)
        && scalar_close(candidate.projected_kkt, permuted.projected_kkt)
        && scalar_close(candidate.maximum_positive_strain,
            permuted.maximum_positive_strain)
        && scalar_close(candidate.rms_positive_strain,
            permuted.rms_positive_strain)
        && scalar_close(candidate.stationarity, permuted.stationarity);
    const bool permutation_control = permutation_metrics
        && candidate.multiplier_root == permuted.multiplier_root
        && candidate.trial_root == permuted.trial_root
        && candidate.result_root == permuted.result_root;

    std::vector<long double> mutated = candidate.multiplier;
    const auto positive = std::find_if(mutated.begin(), mutated.end(),
        [](long double value) { return value > 0.0L; });
    bool mutation_control = positive != mutated.end();
    if (mutation_control) {
        const double narrowed = static_cast<double>(*positive);
        *positive = std::nextafter(narrowed,
            std::numeric_limits<double>::infinity());
        mutation_control = vector_root(
            "nextengine.nonlocal.ncgp12.multiplier.v1", mutated)
            != candidate.multiplier_root;
    }

    const bool controls = zero_control && negated_control && omitted_control
        && permutation_control && mutation_control;
    const bool apparatus = candidate.valid && candidate.solver_converged
        && candidate.jacobian_error <= kJacobianLimit
        && candidate.symmetry_error <= kSymmetryLimit
        && permuted.valid && permuted.solver_converged && controls;
    std::string classification;
    if (!apparatus) {
        classification = "APPARATUS_INCONCLUSIVE";
    } else if (!core_gates(candidate)) {
        classification = "NONLOCAL_SUPPORT_REDESIGN_REQUIRED";
    } else if (!nonlinear_gates(candidate)) {
        classification = "NONLINEAR_PROJECTION_REQUIRED";
    } else {
        classification = "PRESSURE_STATE_LINEARIZED_SUPPORTED";
    }

    const std::string binary = binary_root();
    if (binary.empty()) throw std::runtime_error("NCGP12 binary identity failed");
    const std::string input_root = input_semantic_root(profile, state, ghosts);
    const std::string result = final_root(classification, candidate, permuted,
        omitted, zero_control, negated_control, omitted_control,
        permutation_control, mutation_control, input_root, binary);

    std::cout << std::setprecision(17)
              << "{\"schema\":\"nextengine.nonlocal.ncgp12_pressure_state.v2\""
              << ",\"status\":\"" << classification << "\""
              << ",\"contract_root\":\"" << NCGP12_CONTRACT_ROOT << "\""
              << ",\"source_root\":\"" << NCGP12_SOURCE_ROOT << "\""
              << ",\"source_commit\":\"" << NCGP12_SOURCE_COMMIT << "\""
              << ",\"source_tree\":\"" << NCGP12_SOURCE_TREE << "\""
              << ",\"compiler_flags\":\"" << NCGP12_COMPILER_FLAGS << "\""
              << ",\"binary_root\":\"" << binary << "\""
              << ",\"input_root\":\"" << input_root << "\""
              << ",\"dynamic_samples\":" << state.size()
              << ",\"ghost_samples\":" << ghosts.size()
              << ",\"candidate\":";
    emit_metrics(std::cout, candidate, profile);
    std::cout << ",\"permuted\":";
    emit_metrics(std::cout, permuted, profile);
    std::cout << ",\"omitted_ghost_derivative\":";
    emit_metrics(std::cout, omitted, profile);
    std::cout << ",\"controls\":{\"zero_multiplier\":"
              << (zero_control ? "true" : "false")
              << ",\"negated_pressure\":"
              << (negated_control ? "true" : "false")
              << ",\"omitted_ghost_derivative\":"
              << (omitted_control ? "true" : "false")
              << ",\"permutation\":"
              << (permutation_control ? "true" : "false")
              << ",\"multiplier_mutation\":"
              << (mutation_control ? "true" : "false")
              << "},\"result_root\":\"" << result << "\"}\n";
    return classification == "APPARATUS_INCONCLUSIVE" ? 2 : 0;
}

} // namespace

int main(int argc, char** argv) {
    try {
        if (argc != 2
            || std::string_view(argv[1]) != "--pressure-state-discriminator") {
            std::cerr << "usage: nonlocal-corrected-cpu-pressure-state "
                         "--pressure-state-discriminator\n";
            return 64;
        }
        return run();
    } catch (const std::exception& error) {
        std::cerr << "NCGP12 apparatus failure: " << error.what() << '\n';
        return 3;
    }
}
