#include "corrected_cuda_assembly.hpp"

#include <algorithm>
#include <cmath>
#include <limits>
#include <unordered_set>
#include <vector>

namespace nextengine::nonlocal::gpu_assembly_audit {
namespace {

constexpr long double PI =
    3.141592653589793238462643383279502884L;

struct Vec3Energy {
    long double x = 0.0L;
    long double y = 0.0L;
    long double z = 0.0L;
};

struct EnergyState {
    std::uint32_t id = 0;
    Vec3Energy reference;
    Vec3Energy predicted;
    Vec3Energy current;
    Vec3Energy direction;
};

Vec3Energy operator+(Vec3Energy lhs, Vec3Energy rhs) {
    return {lhs.x + rhs.x, lhs.y + rhs.y, lhs.z + rhs.z};
}

Vec3Energy operator-(Vec3Energy lhs, Vec3Energy rhs) {
    return {lhs.x - rhs.x, lhs.y - rhs.y, lhs.z - rhs.z};
}

Vec3Energy operator*(long double scalar, Vec3Energy value) {
    return {scalar * value.x, scalar * value.y, scalar * value.z};
}

long double dot(Vec3Energy lhs, Vec3Energy rhs) {
    return lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z;
}

long double norm(Vec3Energy value) { return std::sqrt(dot(value, value)); }

Vec3Energy from_um(const std::array<std::int64_t, 3>& value) {
    return {value[0] / 1000000.0L, value[1] / 1000000.0L,
        value[2] / 1000000.0L};
}

Vec3Energy from_milli(const std::array<std::int32_t, 3>& value) {
    return {value[0] / 1000.0L, value[1] / 1000.0L,
        value[2] / 1000.0L};
}

long double weight(long double radius, long double horizon) {
    const long double q = 2.0L * radius / horizon;
    const long double scale = 3.0L
        / (2.0L * PI * horizon * horizon * horizon);
    if (q > 2.0L) return 0.0L;
    if (q >= 1.0L) {
        const long double v = 2.0L - q;
        return scale * v * v * v / 6.0L;
    }
    return scale * (2.0L / 3.0L - q * q + q * q * q / 2.0L);
}

long double omega(long double radius, long double horizon) {
    const long double q = 2.0L * radius / horizon;
    const long double scale = 3.0L
        / (2.0L * PI * horizon * horizon * horizon);
    if (q > 2.0L) return 0.0L;
    const long double derivative_q = q >= 1.0L
        ? -scale * (2.0L - q) * (2.0L - q) / 2.0L
        : scale * (-2.0L * q + 1.5L * q * q);
    return -derivative_q * 2.0L / horizon;
}

long double potential(long double radius, long double spacing) {
    const long double q = radius / spacing;
    if (q <= 1.0L) {
        return spacing * (q * q * q / 3.0L - q - 2.0L / 3.0L);
    }
    if (q < 3.0L) {
        const long double v = q - 2.0L;
        return spacing * (q - v * v * v / 3.0L - 8.0L / 3.0L);
    }
    return 0.0L;
}

std::vector<EnergyState> make_state(const AssemblyFixture& fixture) {
    std::unordered_set<std::uint32_t> ids;
    std::vector<EnergyState> result;
    result.reserve(fixture.samples.size());
    for (const AssemblySample& sample : fixture.samples) {
        if (!ids.insert(sample.sample_id).second) return {};
        result.push_back({sample.sample_id, from_um(sample.reference_um),
            from_um(sample.predicted_um), from_um(sample.current_um),
            from_milli(sample.direction_milli)});
    }
    std::sort(result.begin(), result.end(), [](const EnergyState& lhs,
        const EnergyState& rhs) { return lhs.id < rhs.id; });
    return result;
}

long double energy_only(const AssemblyProfile& profile,
    const AssemblyTerms& terms, const std::vector<EnergyState>& state,
    long double epsilon) {
    const long double h = profile.horizon;
    const long double r0 = profile.spacing;
    const long double m = profile.mass;
    const long double dt = profile.time_step;
    const long double rho0 = profile.rest_density;
    std::vector<Vec3Energy> current(state.size());
    for (std::size_t i = 0; i < state.size(); ++i) {
        current[i] = state[i].current + epsilon * state[i].direction;
    }

    long double result = 0.0L;
    if (terms.inertia) {
        const long double scale = m / (2.0L * dt * dt);
        for (std::size_t i = 0; i < state.size(); ++i) {
            const Vec3Energy displacement = current[i] - state[i].predicted;
            result += scale * dot(displacement, displacement);
        }
    }
    if (terms.pressure) {
        std::vector<long double> density(state.size(), m * weight(0.0L, h));
        for (std::size_t i = 0; i < state.size(); ++i) {
            for (std::size_t j = i + 1U; j < state.size(); ++j) {
                const long double radius = norm(current[i] - current[j]);
                if (radius <= h) {
                    const long double value = m * weight(radius, h);
                    density[i] += value;
                    density[j] += value;
                }
            }
        }
        for (long double value : density) {
            const long double compression = std::max(value / rho0 - 1.0L, 0.0L);
            result += 0.5L * profile.kappa * compression * compression;
        }
    }
    if (terms.viscosity) {
        for (std::size_t i = 0; i < state.size(); ++i) {
            for (std::size_t j = i + 1U; j < state.size(); ++j) {
                const Vec3Energy reference = state[i].reference - state[j].reference;
                const long double radius = norm(reference);
                if (radius <= 1.0e-15L || radius > h) continue;
                const Vec3Energy normal = (1.0L / radius) * reference;
                const Vec3Energy increment = (current[i] - current[j]) - reference;
                const Vec3Energy normal_part = dot(increment, normal) * normal;
                const Vec3Energy tangent_part = increment - normal_part;
                result += m * omega(radius, h) / (rho0 * dt)
                    * (profile.mu * dot(tangent_part, tangent_part)
                        + 0.5L * profile.lambda * dot(normal_part, normal_part));
            }
        }
    }
    if (terms.surface) {
        for (std::size_t i = 0; i < state.size(); ++i) {
            for (std::size_t j = i + 1U; j < state.size(); ++j) {
                const long double radius = norm(current[i] - current[j]);
                if (radius <= 1.0e-15L || radius >= 3.0L * r0) continue;
                result += 2.0L * profile.gamma * m * m
                    * potential(radius, r0);
            }
        }
    }
    return result;
}

} // namespace

EnergyDerivativeResult evaluate_energy_derivatives(
    const AssemblyProfile& profile,
    const AssemblyFixture& fixture) {
    EnergyDerivativeResult output;
    const std::vector<EnergyState> state = make_state(fixture);
    if (state.size() != fixture.samples.size() || state.empty()) return output;
    const long double first_step = profile.horizon * 1.0e-7L;
    const long double second_step = profile.horizon * 1.0e-4L;
    const long double first_plus = energy_only(
        profile, fixture.terms, state, first_step);
    const long double first_minus = energy_only(
        profile, fixture.terms, state, -first_step);
    const long double second_plus = energy_only(
        profile, fixture.terms, state, second_step);
    const long double second_zero = energy_only(
        profile, fixture.terms, state, 0.0L);
    const long double second_minus = energy_only(
        profile, fixture.terms, state, -second_step);
    const long double first = (first_plus - first_minus) / (2.0L * first_step);
    const long double second = (second_plus - 2.0L * second_zero + second_minus)
        / (second_step * second_step);
    const AssemblyResult reference = evaluate_reference_assembly(profile, fixture);
    const bool singleton = state.size() == 1U;
    output.branch_safe = reference.failure == AssemblyFailure::None
        && reference.minimum_density_clamp_margin > 1.0e-5
        && (singleton || (reference.minimum_current_support_margin_um > 100
            && reference.minimum_reference_support_margin_um > 100
            && reference.minimum_surface_branch_margin_um > 100));
    output.finite = std::isfinite(first) && std::isfinite(second);
    output.first_directional = static_cast<double>(first);
    output.second_directional = static_cast<double>(second);
    output.gradient_step = static_cast<double>(first_step);
    output.hessian_step = static_cast<double>(second_step);
    return output;
}

} // namespace nextengine::nonlocal::gpu_assembly_audit
