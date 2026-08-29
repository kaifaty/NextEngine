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

Vec3Long add(Vec3Long lhs, Vec3Long rhs) {
    return {lhs.x + rhs.x, lhs.y + rhs.y, lhs.z + rhs.z};
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

long double kernel_gradient(long double radius, long double horizon) {
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

long double viscosity_energy(
    const AuditProfile& profile,
    const TermInput& input,
    Vec3Long first,
    Vec3Long second) {
    const Vec3Long reference_delta = subtract(
        to_long(input.reference_first), to_long(input.reference_second));
    const Vec3Long normal = normalized(reference_delta);
    const Vec3Long increment = subtract(subtract(first, second), reference_delta);
    const Vec3Long normal_component = multiply(dot(increment, normal), normal);
    const bool shear = input.kind == TermKind::ShearViscosity;
    const Vec3Long component = shear
        ? subtract(increment, normal_component)
        : normal_component;
    const long double energy_coefficient = shear
        ? static_cast<long double>(profile.mu)
        : 0.5L * static_cast<long double>(profile.lambda);
    const long double influence = -kernel_gradient(
        length(reference_delta), profile.horizon);
    return static_cast<long double>(profile.mass)
        / (static_cast<long double>(profile.rest_density)
            * static_cast<long double>(profile.time_step))
        * energy_coefficient * dot(component, component) * influence;
}

} // namespace

EnergyDerivativeOutput evaluate_viscosity_energy_derivative(
    const AuditProfile& profile,
    const TermInput& input) {
    EnergyDerivativeOutput output;
    if (input.kind != TermKind::BulkViscosity
        && input.kind != TermKind::ShearViscosity) {
        return output;
    }
    const Vec3Long direction = normalized({-0.27L, 0.91L, 0.31L});
    const long double epsilon =
        static_cast<long double>(profile.horizon) * 1.0e-8L;
    const Vec3Long first = to_long(input.first);
    const Vec3Long second = to_long(input.second);
    const long double plus = viscosity_energy(
        profile, input, add(first, multiply(epsilon, direction)), second);
    const long double minus = viscosity_energy(
        profile, input, subtract(first, multiply(epsilon, direction)), second);
    const long double derivative = (plus - minus) / (2.0L * epsilon);
    output.direction = {
        static_cast<double>(direction.x),
        static_cast<double>(direction.y),
        static_cast<double>(direction.z),
    };
    output.directional_derivative = static_cast<double>(derivative);
    output.finite = std::isfinite(derivative);
    return output;
}

} // namespace nextengine::nonlocal::gpu_audit
