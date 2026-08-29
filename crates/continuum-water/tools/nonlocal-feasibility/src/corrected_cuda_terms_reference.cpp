#include "corrected_cuda_terms.hpp"

#include <algorithm>
#include <cmath>

namespace nextengine::nonlocal::gpu_audit {
namespace {

constexpr long double PI =
    3.141592653589793238462643383279502884L;

struct Vec3Long {
    long double x = 0.0L;
    long double y = 0.0L;
    long double z = 0.0L;
};

Vec3Long to_long(Vec3Input value) {
    return {value.x, value.y, value.z};
}

Vec3Long subtract(Vec3Long lhs, Vec3Long rhs) {
    return {lhs.x - rhs.x, lhs.y - rhs.y, lhs.z - rhs.z};
}

Vec3Long multiply(long double scalar, Vec3Long value) {
    return {scalar * value.x, scalar * value.y, scalar * value.z};
}

long double dot(Vec3Long lhs, Vec3Long rhs) {
    return lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z;
}

long double length(Vec3Long value) {
    return std::sqrt(dot(value, value));
}

Vec3Long normalized(Vec3Long value) {
    return multiply(1.0L / length(value), value);
}

long double cubic_weight(long double radius, long double horizon) {
    const long double q = 2.0L * radius / horizon;
    const long double alpha =
        3.0L / (2.0L * PI * horizon * horizon * horizon);
    if (q > 2.0L) {
        return 0.0L;
    }
    if (q >= 1.0L) {
        const long double delta = 2.0L - q;
        return alpha * delta * delta * delta / 6.0L;
    }
    return alpha * (2.0L / 3.0L - q * q + 0.5L * q * q * q);
}

long double cubic_gradient(long double radius, long double horizon) {
    const long double q = 2.0L * radius / horizon;
    const long double alpha =
        3.0L / (2.0L * PI * horizon * horizon * horizon);
    long double derivative_q = 0.0L;
    if (q > 2.0L) {
        return 0.0L;
    }
    if (q >= 1.0L) {
        const long double delta = 2.0L - q;
        derivative_q = -0.5L * alpha * delta * delta;
    } else {
        derivative_q = alpha * (-2.0L * q + 1.5L * q * q);
    }
    return derivative_q * (2.0L / horizon);
}

long double surface_force(long double radius, long double spacing) {
    const long double q = radius / spacing;
    if (q <= 1.0L) {
        return q * q - 1.0L;
    }
    if (q < 3.0L) {
        const long double shifted = q - 2.0L;
        return 1.0L - shifted * shifted;
    }
    return 0.0L;
}

Vec3Input to_output(Vec3Long value) {
    return {
        static_cast<double>(value.x),
        static_cast<double>(value.y),
        static_cast<double>(value.z),
    };
}

bool finite(Vec3Long value) {
    return std::isfinite(value.x) && std::isfinite(value.y)
        && std::isfinite(value.z);
}

} // namespace

ReferenceTermOutput evaluate_reference_term(
    const AuditProfile& profile,
    const TermInput& input) {
    const long double horizon = profile.horizon;
    const long double spacing = profile.spacing;
    const long double mass = profile.mass;
    const long double time_step = profile.time_step;
    const long double rest_density = profile.rest_density;
    const Vec3Long first = to_long(input.first);
    const Vec3Long second = to_long(input.second);

    ReferenceTermOutput result;
    Vec3Long first_force;
    Vec3Long second_force;

    if (input.kind == TermKind::KernelGradient) {
        result.scalar = static_cast<double>(
            cubic_gradient(input.parameter, horizon));
        result.finite = std::isfinite(result.scalar);
        return result;
    }

    if (input.kind == TermKind::Compression) {
        const Vec3Long delta = subtract(first, second);
        const long double radius = length(delta);
        const Vec3Long direction = normalized(delta);
        const long double density = mass
            * (cubic_weight(0.0L, horizon)
                + cubic_weight(radius, horizon));
        const long double control_rest_density =
            density / static_cast<long double>(input.parameter);
        const long double error = std::max(
            density / control_rest_density - 1.0L, 0.0L);
        const long double factor = -2.0L * profile.kappa * error * mass
            / control_rest_density * cubic_gradient(radius, horizon);
        first_force = multiply(factor, direction);
        const Vec3Long reverse_direction = normalized(subtract(second, first));
        second_force = multiply(factor, reverse_direction);
    } else if (input.kind == TermKind::BulkViscosity
        || input.kind == TermKind::ShearViscosity) {
        const Vec3Long reference_delta = subtract(
            to_long(input.reference_first),
            to_long(input.reference_second));
        const long double radius = length(reference_delta);
        const Vec3Long normal = normalized(reference_delta);
        const Vec3Long increment = subtract(subtract(first, second), reference_delta);
        const Vec3Long normal_component = multiply(dot(increment, normal), normal);
        const bool shear = input.kind == TermKind::ShearViscosity;
        const Vec3Long component = shear
            ? subtract(increment, normal_component)
            : normal_component;
        const long double influence = -cubic_gradient(radius, horizon);
        const long double coefficient = shear
            ? 2.0L * profile.mu
            : profile.lambda;
        const long double factor = -mass / (rest_density * time_step)
            * coefficient * influence;
        first_force = multiply(factor, component);

        const Vec3Long reverse_reference_delta = subtract(
            to_long(input.reference_second),
            to_long(input.reference_first));
        const Vec3Long reverse_normal = normalized(reverse_reference_delta);
        const Vec3Long reverse_increment = subtract(
            subtract(second, first), reverse_reference_delta);
        const Vec3Long reverse_normal_component = multiply(
            dot(reverse_increment, reverse_normal), reverse_normal);
        const Vec3Long reverse_component = shear
            ? subtract(reverse_increment, reverse_normal_component)
            : reverse_normal_component;
        second_force = multiply(factor, reverse_component);
    } else if (input.kind == TermKind::Surface) {
        const Vec3Long delta = subtract(first, second);
        const long double radius = length(delta);
        const long double pair_value = surface_force(radius, spacing);
        if (radius > 0.0L) {
            const long double factor = -2.0L * profile.gamma * mass * mass
                * pair_value;
            first_force = multiply(factor, normalized(delta));
            second_force = multiply(
                factor, normalized(subtract(second, first)));
        }
    }

    result.first_force = to_output(first_force);
    result.second_force = to_output(second_force);
    result.finite = finite(first_force) && finite(second_force);
    return result;
}

} // namespace nextengine::nonlocal::gpu_audit
