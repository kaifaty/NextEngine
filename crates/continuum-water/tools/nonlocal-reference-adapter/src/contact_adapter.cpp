#include "contact_adapter.hpp"

#include "sha256.hpp"

#include "SPlisHSPlasH/Common.h"
#include "SPlisHSPlasH/DFSPH/TimeStepDFSPH.h"

#include <algorithm>
#include <array>
#include <cfenv>
#include <clocale>
#include <cmath>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <iomanip>
#include <limits>
#include <optional>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

#include <omp.h>
#include <xmmintrin.h>

namespace nextengine::nonlocal_reference {
namespace {

constexpr std::string_view SCHEMA =
    "nextengine.nonlocal.nsr3b4dr1b-contact-adapter.v2";
constexpr std::string_view CONTRACT_IDENTITY =
    "c65346ec7b215a7a173eabfdd6c91e20869d4a0679b9d014a1e897369cb71f84";
constexpr std::string_view PARENT_IDENTITY =
    "ade621f889a08fd26ca713592625c50316b893af8c4f1797d25f4e4d4c96b86a";
constexpr std::string_view UPSTREAM_LIBRARY_SHA256 =
    "172e6777027564566d6282f8679f4d93cbd4cdc193fc236eb9fdaa210ea07d20";

constexpr std::uint64_t DT_BITS = UINT64_C(0x3f71111111111111);
constexpr std::uint64_t INV_DT_BITS = UINT64_C(0x406e000000000000);
constexpr std::uint64_t RADIUS_BITS = UINT64_C(0x3f9999999999999a);
constexpr std::size_t FEATURE_CAPACITY = 25;
constexpr std::size_t HIT_LIMIT = 8;
constexpr std::uint32_t MXCSR_DAZ = UINT32_C(1) << 6U;
constexpr std::uint32_t MXCSR_FTZ = UINT32_C(1) << 15U;

double from_bits(std::uint64_t bits) {
    double value = 0.0;
    static_assert(sizeof(value) == sizeof(bits));
    std::memcpy(&value, &bits, sizeof(value));
    return value;
}

std::uint64_t to_bits(double value) {
    std::uint64_t bits = 0;
    std::memcpy(&bits, &value, sizeof(bits));
    return bits;
}

const double DT = from_bits(DT_BITS);
const double INV_DT = from_bits(INV_DT_BITS);
const double RADIUS = from_bits(RADIUS_BITS);
const double DIRECTION_GUARD = 32.0 * std::numeric_limits<double>::epsilon();

struct Vec3 {
    double x;
    double y;
    double z;
};

Vec3 add(Vec3 left, Vec3 right) {
    return {left.x + right.x, left.y + right.y, left.z + right.z};
}

Vec3 subtract(Vec3 left, Vec3 right) {
    return {left.x - right.x, left.y - right.y, left.z - right.z};
}

Vec3 scale(Vec3 vector, double scalar) {
    return {vector.x * scalar, vector.y * scalar, vector.z * scalar};
}

double dot(Vec3 left, Vec3 right) {
    return ((left.x * right.x) + (left.y * right.y)) + (left.z * right.z);
}

bool is_zero(Vec3 vector) {
    return vector.x == 0.0 && vector.y == 0.0 && vector.z == 0.0;
}

void require_finite(double value, std::string_view phase) {
    if (!std::isfinite(value)) {
        throw std::runtime_error(std::string(phase) + " is not finite");
    }
}

void require_finite(Vec3 value, std::string_view phase) {
    require_finite(value.x, phase);
    require_finite(value.y, phase);
    require_finite(value.z, phase);
}

Vec3 normalize(Vec3 vector, std::string_view phase) {
    const double squared = dot(vector, vector);
    require_finite(squared, phase);
    if (squared <= 0.0) {
        throw std::runtime_error(std::string(phase) + " has no direction");
    }
    const double inverse = 1.0 / std::sqrt(squared);
    require_finite(inverse, phase);
    const Vec3 result = scale(vector, inverse);
    require_finite(result, phase);
    return result;
}

struct Hit {
    double time;
    Vec3 normal;
    std::uint32_t feature;
};

struct Projection {
    Vec3 velocity;
    Vec3 end;
    std::array<std::uint32_t, FEATURE_CAPACITY> feature_counts{};
    std::vector<std::uint32_t> hit_order;
};

void select_hit(std::optional<Hit> &best, Hit candidate) {
    if (!best.has_value() || candidate.time < best->time
        || (to_bits(candidate.time) == to_bits(best->time)
            && candidate.feature < best->feature)) {
        best = candidate;
    }
}

bool meaningfully_inward(Vec3 displacement, Vec3 normal) {
    const double direction = dot(displacement, normal);
    const double length = std::sqrt(dot(displacement, displacement));
    require_finite(direction, "contact approach direction");
    require_finite(length, "contact displacement length");
    return direction < -(DIRECTION_GUARD * length);
}

void consider_face(
    std::optional<Hit> &best,
    Vec3 start,
    Vec3 displacement,
    double plane,
    Vec3 normal) {
    const double time = (plane - start.x) / displacement.x;
    require_finite(time, "internal face hit time");
    if (time < 0.0 || time > 1.0) {
        return;
    }
    const Vec3 point = add(start, scale(displacement, time));
    require_finite(point, "internal face hit point");
    const bool in_patch = point.y >= 0.0 && point.y <= 1.0 && point.z >= 0.0
        && point.z <= 1.0;
    const bool in_closed_opening = point.y >= 0.2 && point.y <= 0.4
        && point.z >= 0.4 && point.z <= 0.6;
    if (in_patch && !in_closed_opening) {
        select_hit(best, {time, normal, 16});
    }
}

void consider_capsule(
    std::optional<Hit> &best,
    Vec3 start,
    Vec3 displacement,
    Vec3 segment_start,
    Vec3 segment_end,
    std::size_t axis,
    std::uint32_t feature) {
    const std::array<double, 3> s = {start.x, start.y, start.z};
    const std::array<double, 3> d = {displacement.x, displacement.y, displacement.z};
    const std::array<double, 3> e = {segment_start.x, segment_start.y, segment_start.z};
    const std::array<std::size_t, 2> perpendicular =
        axis == 1 ? std::array<std::size_t, 2>{0, 2}
                  : std::array<std::size_t, 2>{0, 1};
    if (axis != 1 && axis != 2) {
        throw std::runtime_error("unsupported capsule axis");
    }

    const double r0 = s[perpendicular[0]] - e[perpendicular[0]];
    const double r1 = s[perpendicular[1]] - e[perpendicular[1]];
    const double d0 = d[perpendicular[0]];
    const double d1 = d[perpendicular[1]];
    const double a = (d0 * d0) + (d1 * d1);
    const double b = 2.0 * ((r0 * d0) + (r1 * d1));
    const double c = ((r0 * r0) + (r1 * r1)) - (RADIUS * RADIUS);
    require_finite(a, "capsule quadratic a");
    require_finite(b, "capsule quadratic b");
    require_finite(c, "capsule quadratic c");

    const std::array<double, 3> end = {segment_end.x, segment_end.y, segment_end.z};
    const double axis_min = e[axis];
    const double axis_max = end[axis];
    if (c <= 0.0 && s[axis] > axis_min && s[axis] < axis_max) {
        std::array<double, 3> components = {0.0, 0.0, 0.0};
        components[perpendicular[0]] = r0;
        components[perpendicular[1]] = r1;
        const Vec3 normal = normalize(
            {components[0], components[1], components[2]},
            "capsule touching normal");
        if (meaningfully_inward(displacement, normal)) {
            select_hit(best, {0.0, normal, feature});
        }
    }
    if (a == 0.0) {
        return;
    }

    const double discriminant = (b * b) - (4.0 * a * c);
    require_finite(discriminant, "capsule discriminant");
    if (discriminant < 0.0) {
        return;
    }
    const double time = (-b - std::sqrt(discriminant)) / (2.0 * a);
    require_finite(time, "capsule hit time");
    if (time < 0.0 || time > 1.0) {
        return;
    }
    const double coordinate = s[axis] + (d[axis] * time);
    require_finite(coordinate, "capsule axial coordinate");
    if (coordinate <= axis_min || coordinate >= axis_max) {
        return;
    }
    std::array<double, 3> components = {0.0, 0.0, 0.0};
    components[perpendicular[0]] = r0 + (d0 * time);
    components[perpendicular[1]] = r1 + (d1 * time);
    const Vec3 normal = normalize(
        {components[0], components[1], components[2]},
        "capsule hit normal");
    if (meaningfully_inward(displacement, normal)) {
        select_hit(best, {time, normal, feature});
    }
}

void consider_sphere(
    std::optional<Hit> &best,
    Vec3 start,
    Vec3 displacement,
    Vec3 centre,
    std::uint32_t feature) {
    const Vec3 radial = subtract(start, centre);
    const double a = dot(displacement, displacement);
    const double b = 2.0 * dot(radial, displacement);
    const double c = dot(radial, radial) - (RADIUS * RADIUS);
    require_finite(a, "sphere quadratic a");
    require_finite(b, "sphere quadratic b");
    require_finite(c, "sphere quadratic c");
    if (c <= 0.0) {
        const Vec3 normal = normalize(radial, "sphere touching normal");
        if (meaningfully_inward(displacement, normal)) {
            select_hit(best, {0.0, normal, feature});
        }
    }
    if (a == 0.0) {
        return;
    }
    const double discriminant = (b * b) - (4.0 * a * c);
    require_finite(discriminant, "sphere discriminant");
    if (discriminant < 0.0) {
        return;
    }
    const double time = (-b - std::sqrt(discriminant)) / (2.0 * a);
    require_finite(time, "sphere hit time");
    if (time < 0.0 || time > 1.0) {
        return;
    }
    const Vec3 point = add(start, scale(displacement, time));
    const Vec3 normal = normalize(subtract(point, centre), "sphere hit normal");
    if (meaningfully_inward(displacement, normal)) {
        select_hit(best, {time, normal, feature});
    }
}

std::optional<Hit> earliest_internal_hit(Vec3 start, Vec3 displacement) {
    std::optional<Hit> best;
    if (displacement.x > 0.0 && start.x <= 1.0) {
        consider_face(best, start, displacement, 1.0 - RADIUS, {-1.0, 0.0, 0.0});
    }
    if (displacement.x < 0.0 && start.x >= 1.0) {
        consider_face(best, start, displacement, 1.0 + RADIUS, {1.0, 0.0, 0.0});
    }

    consider_capsule(best, start, displacement, {1.0, 0.2, 0.4}, {1.0, 0.2, 0.6}, 2, 17);
    consider_capsule(best, start, displacement, {1.0, 0.4, 0.4}, {1.0, 0.4, 0.6}, 2, 18);
    consider_capsule(best, start, displacement, {1.0, 0.2, 0.4}, {1.0, 0.4, 0.4}, 1, 19);
    consider_capsule(best, start, displacement, {1.0, 0.2, 0.6}, {1.0, 0.4, 0.6}, 1, 20);
    consider_sphere(best, start, displacement, {1.0, 0.2, 0.4}, 21);
    consider_sphere(best, start, displacement, {1.0, 0.2, 0.6}, 22);
    consider_sphere(best, start, displacement, {1.0, 0.4, 0.4}, 23);
    consider_sphere(best, start, displacement, {1.0, 0.4, 0.6}, 24);
    return best;
}

Projection project_internal(Vec3 start, Vec3 velocity) {
    Vec3 current = start;
    Vec3 remaining = scale(velocity, DT);
    Projection result;
    result.velocity = velocity;
    bool contacted = false;
    for (std::size_t iteration = 0; iteration < HIT_LIMIT; ++iteration) {
        const auto hit = earliest_internal_hit(current, remaining);
        if (!hit.has_value()) {
            current = add(current, remaining);
            require_finite(current, "internal final advance");
            remaining = {0.0, 0.0, 0.0};
            break;
        }
        current = add(current, scale(remaining, hit->time));
        const Vec3 tail = scale(remaining, 1.0 - hit->time);
        const double inward = dot(tail, hit->normal);
        require_finite(current, "internal impact advance");
        require_finite(tail, "internal tail");
        require_finite(inward, "internal inward component");
        if (inward >= 0.0) {
            current = add(current, tail);
            remaining = {0.0, 0.0, 0.0};
            break;
        }
        const Vec3 correction = scale(hit->normal, -inward);
        remaining = add(tail, correction);
        require_finite(remaining, "internal projected tail");
        result.feature_counts.at(hit->feature) += 1U;
        result.hit_order.push_back(hit->feature);
        contacted = true;
    }
    if (!is_zero(remaining)) {
        throw std::runtime_error("internal contact exceeded fixed eight-hit schedule");
    }
    result.end = current;
    if (contacted) {
        result.velocity = scale(subtract(current, start), INV_DT);
        require_finite(result.velocity, "internal published velocity");
    }
    return result;
}

double project_axis(double position, double velocity, double lower, double upper) {
    const double minimum_velocity = (lower - position) * INV_DT;
    const double maximum_velocity = (upper - position) * INV_DT;
    require_finite(minimum_velocity, "outer minimum velocity");
    require_finite(maximum_velocity, "outer maximum velocity");
    return std::max(minimum_velocity, std::min(velocity, maximum_velocity));
}

Projection project_outer(Vec3 start, Vec3 velocity) {
    Projection result;
    result.velocity = {
        project_axis(start.x, velocity.x, RADIUS, 1.0 - RADIUS),
        project_axis(start.y, velocity.y, RADIUS, 1.0 - RADIUS),
        project_axis(start.z, velocity.z, RADIUS, 1.0 - RADIUS),
    };
    const Vec3 delta = subtract(result.velocity, velocity);
    const std::array<double, 3> components = {delta.x, delta.y, delta.z};
    const std::array<std::uint32_t, 3> minimum_features = {0, 2, 4};
    const std::array<std::uint32_t, 3> maximum_features = {1, 3, 5};
    for (std::size_t axis = 0; axis < components.size(); ++axis) {
        if (components[axis] == 0.0) {
            continue;
        }
        const std::uint32_t feature =
            components[axis] > 0.0 ? minimum_features[axis] : maximum_features[axis];
        result.feature_counts.at(feature) += 1U;
        result.hit_order.push_back(feature);
    }
    result.end = add(start, scale(result.velocity, DT));
    require_finite(result.end, "outer accepted end");
    return result;
}

std::int64_t quantize(double value, double scale_factor, std::string_view phase) {
    const double scaled = value * scale_factor;
    require_finite(scaled, phase);
    const double rounded = std::nearbyint(scaled);
    if (rounded < static_cast<double>(std::numeric_limits<std::int64_t>::min())
        || rounded > static_cast<double>(std::numeric_limits<std::int64_t>::max())) {
        throw std::runtime_error(std::string(phase) + " exceeds i64");
    }
    return static_cast<std::int64_t>(rounded);
}

std::array<std::int64_t, 3> quantize_position(Vec3 value) {
    return {
        quantize(value.x, 1'000'000.0, "position x"),
        quantize(value.y, 1'000'000.0, "position y"),
        quantize(value.z, 1'000'000.0, "position z"),
    };
}

std::array<std::int64_t, 3> quantize_velocity(Vec3 value) {
    return {
        quantize(value.x, 1'000'000.0, "velocity x"),
        quantize(value.y, 1'000'000.0, "velocity y"),
        quantize(value.z, 1'000'000.0, "velocity z"),
    };
}

double point_segment_distance_squared(Vec3 point, Vec3 first, Vec3 second) {
    const Vec3 segment = subtract(second, first);
    const double denominator = dot(segment, segment);
    if (denominator <= 0.0) {
        throw std::runtime_error("validator segment has no extent");
    }
    const double raw = dot(subtract(point, first), segment) / denominator;
    const double parameter = std::max(0.0, std::min(raw, 1.0));
    const Vec3 nearest = add(first, scale(segment, parameter));
    const Vec3 radial = subtract(point, nearest);
    const double result = dot(radial, radial);
    require_finite(result, "validator segment distance");
    return result;
}

void validate_outer_end(Vec3 end, double x_max) {
    require_finite(end, "validator outer end");
    if (end.x < RADIUS || end.x > x_max - RADIUS || end.y < RADIUS
        || end.y > 1.0 - RADIUS || end.z < RADIUS || end.z > 1.0 - RADIUS) {
        throw std::runtime_error("accepted endpoint violates outer radius clearance");
    }
}

void validate_internal_end(Vec3 end) {
    validate_outer_end(end, 2.0);
    const bool in_closed_opening = end.y >= 0.2 && end.y <= 0.4 && end.z >= 0.4
        && end.z <= 0.6;
    double distance_squared = 0.0;
    if (in_closed_opening) {
        distance_squared = point_segment_distance_squared(
            end, {1.0, 0.2, 0.4}, {1.0, 0.2, 0.6});
        for (const auto &segment : std::array<std::array<Vec3, 2>, 3>{
                 std::array<Vec3, 2>{Vec3{1.0, 0.4, 0.4}, Vec3{1.0, 0.4, 0.6}},
                 std::array<Vec3, 2>{Vec3{1.0, 0.2, 0.4}, Vec3{1.0, 0.4, 0.4}},
                 std::array<Vec3, 2>{Vec3{1.0, 0.2, 0.6}, Vec3{1.0, 0.4, 0.6}},
             }) {
            distance_squared = std::min(
                distance_squared,
                point_segment_distance_squared(end, segment[0], segment[1]));
        }
    } else {
        const double distance = std::abs(end.x - 1.0);
        distance_squared = distance * distance;
    }
    const double required = RADIUS * RADIUS;
    if (distance_squared < required) {
        throw std::runtime_error("accepted endpoint violates internal radius clearance");
    }
}

void validate_chord(Vec3 start, Vec3 end) {
    const double start_side = start.x - 1.0;
    const double end_side = end.x - 1.0;
    if (!((start_side < 0.0 && end_side >= 0.0)
            || (start_side > 0.0 && end_side <= 0.0))) {
        return;
    }
    const double denominator = end.x - start.x;
    if (denominator == 0.0) {
        throw std::runtime_error("wall chord has zero denominator");
    }
    const double time = (1.0 - start.x) / denominator;
    const Vec3 crossing = add(start, scale(subtract(end, start), time));
    require_finite(crossing, "wall chord crossing");
    if (crossing.y < 0.2 + RADIUS || crossing.y > 0.4 - RADIUS
        || crossing.z < 0.4 + RADIUS || crossing.z > 0.6 - RADIUS) {
        throw std::runtime_error("accepted chord crosses outside radius-safe opening");
    }
}

void expect_vector(
    std::string_view id,
    const Projection &result,
    const std::array<std::int64_t, 3> &velocity,
    const std::array<std::int64_t, 3> &end,
    const std::vector<std::uint32_t> &hits) {
    if (quantize_velocity(result.velocity) != velocity) {
        throw std::runtime_error(std::string(id) + " velocity mismatch");
    }
    if (quantize_position(result.end) != end) {
        throw std::runtime_error(std::string(id) + " endpoint mismatch");
    }
    if (result.hit_order != hits) {
        throw std::runtime_error(std::string(id) + " hit order mismatch");
    }
    std::array<std::uint32_t, FEATURE_CAPACITY> expected_counts{};
    for (const std::uint32_t feature : hits) {
        expected_counts.at(feature) += 1U;
    }
    if (result.feature_counts != expected_counts) {
        throw std::runtime_error(std::string(id) + " feature count mismatch");
    }
}

std::string vector_string(const std::array<std::int64_t, 3> &value) {
    std::ostringstream output;
    output << value[0] << ',' << value[1] << ',' << value[2];
    return output.str();
}

std::string hit_string(const std::vector<std::uint32_t> &hits) {
    if (hits.empty()) {
        return "none";
    }
    std::ostringstream output;
    for (std::size_t index = 0; index < hits.size(); ++index) {
        if (index != 0) {
            output << ',';
        }
        output << hits[index];
    }
    return output.str();
}

std::string fixture_line(std::string_view id, const Projection &result) {
    std::ostringstream output;
    output << "fixture." << id << "=PASS;velocity_um_s="
           << vector_string(quantize_velocity(result.velocity)) << ";end_um="
           << vector_string(quantize_position(result.end)) << ";hits="
           << hit_string(result.hit_order) << '\n';
    return output.str();
}

std::string hex_bits(double value) {
    std::ostringstream output;
    output << "0x" << std::hex << std::setfill('0') << std::setw(16) << to_bits(value);
    return output.str();
}

std::string profile_projection(
    std::uint64_t radius_bits,
    const std::array<std::uint32_t, 9> &internal_order) {
    const std::array<std::pair<Vec3, Vec3>, 6> vectors = {
        std::pair<Vec3, Vec3>{{0.95, 0.5, 0.5}, {12.0, 0.0, 0.0}},
        std::pair<Vec3, Vec3>{{0.5, 0.5, 0.5}, {-300.0, 0.0, 0.0}},
        std::pair<Vec3, Vec3>{{0.9, 0.3, 0.5}, {30.0, 0.0, 0.0}},
        std::pair<Vec3, Vec3>{{0.9, 0.225, 0.5}, {30.0, 0.0, 0.0}},
        std::pair<Vec3, Vec3>{{0.9, 0.22, 0.5}, {30.0, 0.0, 0.0}},
        std::pair<Vec3, Vec3>{{0.9, 0.2, 0.4}, {30.0, 0.0, 0.0}},
    };
    const std::array<std::string_view, 6> ids = {
        "OUTER-FACE-INTERIOR",
        "OUTER-HIGH-SPEED",
        "APERTURE-PASS",
        "APERTURE-EDGE-GRAZE",
        "APERTURE-EDGE-IMPACT",
        "APERTURE-CORNER-SPHERE",
    };
    std::ostringstream output;
    output << "B4DR1B_CONTACT_PROFILE_V2_BEGIN\n"
           << "contract=" << CONTRACT_IDENTITY << '\n'
           << "parent=" << PARENT_IDENTITY << '\n'
           << "upstream_library=" << UPSTREAM_LIBRARY_SHA256 << '\n'
           << "geometry=outer-unit:[0,1]^3;orifice:[0,2]x[0,1]^2;wall-x=1;"
              "opening-y=.2:.4;opening-z=.4:.6\n"
           << "dt_bits=0x" << std::hex << std::setfill('0') << std::setw(16) << DT_BITS
           << ";inv_dt_bits=0x" << std::setw(16) << INV_DT_BITS << ";radius_bits=0x"
           << std::setw(16) << radius_bits << std::dec << '\n'
           << "guard=32eps;schedule=8;outer_order=0,1,2,3,4,5;internal_order=";
    for (std::size_t index = 0; index < internal_order.size(); ++index) {
        if (index != 0) {
            output << ',';
        }
        output << internal_order[index];
    }
    output << '\n';
    for (std::size_t index = 0; index < vectors.size(); ++index) {
        output << "vector." << ids[index] << ".start=" << hex_bits(vectors[index].first.x)
               << ',' << hex_bits(vectors[index].first.y) << ','
               << hex_bits(vectors[index].first.z) << ";velocity="
               << hex_bits(vectors[index].second.x) << ','
               << hex_bits(vectors[index].second.y) << ','
               << hex_bits(vectors[index].second.z) << '\n';
    }
    output << "sentinel.internal-face=(.9,.1,.5);(30,0,0)\n"
           << "sentinel.edge-restart=derived-contact-x-nextafter-axis;.22,.5;(30,0,0)\n"
           << "B4DR1B_CONTACT_PROFILE_V2_END\n";
    return output.str();
}

std::string preflight_failure() {
    const char *locale = std::setlocale(LC_ALL, "");
    if (locale == nullptr || std::string_view(locale) != "C") {
        return "LOCALE_NOT_C";
    }
    const char *locale_environment = std::getenv("LC_ALL");
    if (locale_environment == nullptr || std::string_view(locale_environment) != "C") {
        return "LC_ALL_NOT_EXACT_C";
    }
    if (std::fegetround() != FE_TONEAREST) {
        return "ROUNDING_NOT_NEAREST";
    }
    const std::uint32_t mxcsr = _mm_getcsr();
    if ((mxcsr & MXCSR_FTZ) != 0U) {
        return "FTZ_ENABLED";
    }
    if ((mxcsr & MXCSR_DAZ) != 0U) {
        return "DAZ_ENABLED";
    }
    const char *omp_threads = std::getenv("OMP_NUM_THREADS");
    if (omp_threads == nullptr || std::string_view(omp_threads) != "1") {
        return "OMP_NUM_THREADS_NOT_ONE";
    }
    const char *omp_dynamic = std::getenv("OMP_DYNAMIC");
    if (omp_dynamic == nullptr || std::string_view(omp_dynamic) != "FALSE") {
        return "OMP_DYNAMIC_NOT_FALSE";
    }
    if (omp_get_dynamic() != 0) {
        return "OPENMP_DYNAMIC_ACTIVE";
    }
    if (omp_get_max_threads() != 1) {
        return "OPENMP_MAX_THREADS_NOT_ONE";
    }
    int observed_team_size = 0;
#pragma omp parallel
    {
#pragma omp single
        observed_team_size = omp_get_num_threads();
    }
    if (observed_team_size != 1) {
        return "OPENMP_TEAM_SIZE_NOT_ONE";
    }
    if (sizeof(Real) != 8U || !std::numeric_limits<Real>::is_iec559
        || std::numeric_limits<Real>::radix != 2 || std::numeric_limits<Real>::digits != 53) {
        return "UPSTREAM_REAL_NOT_BINARY64";
    }
    if (sizeof(void *) != 8U) {
        return "POINTER_ABI_NOT_64_BIT";
    }
    const std::uint32_t endian_probe = UINT32_C(0x01020304);
    std::array<unsigned char, sizeof(endian_probe)> endian_bytes{};
    std::memcpy(endian_bytes.data(), &endian_probe, sizeof(endian_probe));
    if (endian_bytes[0] != 0x04U) {
        return "HOST_NOT_LITTLE_ENDIAN";
    }
    if (SPH::TimeStepDFSPH::METHOD_NAME != "DFSPH") {
        return "UPSTREAM_DFSPH_ANCHOR_MISMATCH";
    }
    return {};
}

std::string rejected_report(std::string_view reason) {
    std::ostringstream output;
    output << "schema=" << SCHEMA << '\n'
           << "contract_identity=" << CONTRACT_IDENTITY << '\n'
           << "status=REJECTED\n"
           << "reason=" << reason << '\n'
           << "contact_started=false\n"
           << "trajectory_started=false\n";
    return output.str();
}

} // namespace

AdapterRun reject_unknown_argument() {
    return {false, rejected_report("UNKNOWN_ARGUMENT")};
}

AdapterRun run_contact_adapter(PreflightMutation mutation) {
    if (mutation == PreflightMutation::RoundDown) {
        if (std::fesetround(FE_DOWNWARD) != 0) {
            return {false, rejected_report("ROUNDING_MUTATION_FAILED")};
        }
    } else if (mutation == PreflightMutation::FtzOn) {
        _mm_setcsr(_mm_getcsr() | MXCSR_FTZ);
    }

    const std::string preflight = preflight_failure();
    if (!preflight.empty()) {
        return {false, rejected_report(preflight)};
    }
    if (mutation != PreflightMutation::None) {
        return {false, rejected_report("NEGATIVE_MUTATION_NOT_REJECTED")};
    }

    try {
        const Vec3 outer_face_start = {0.95, 0.5, 0.5};
        const Projection outer_face = project_outer(outer_face_start, {12.0, 0.0, 0.0});
        expect_vector(
            "OUTER-FACE-INTERIOR",
            outer_face,
            {6'000'000, 0, 0},
            {975'000, 500'000, 500'000},
            {1});
        validate_outer_end(outer_face.end, 1.0);

        const Vec3 outer_fast_start = {0.5, 0.5, 0.5};
        const Projection outer_fast = project_outer(outer_fast_start, {-300.0, 0.0, 0.0});
        expect_vector(
            "OUTER-HIGH-SPEED",
            outer_fast,
            {-114'000'000, 0, 0},
            {25'000, 500'000, 500'000},
            {0});
        validate_outer_end(outer_fast.end, 1.0);

        const Vec3 pass_start = {0.9, 0.3, 0.5};
        const Projection pass = project_internal(pass_start, {30.0, 0.0, 0.0});
        expect_vector(
            "APERTURE-PASS",
            pass,
            {30'000'000, 0, 0},
            {1'025'000, 300'000, 500'000},
            {});
        validate_internal_end(pass.end);
        validate_chord(pass_start, pass.end);

        const Vec3 graze_start = {0.9, 0.225, 0.5};
        const Projection graze = project_internal(graze_start, {30.0, 0.0, 0.0});
        expect_vector(
            "APERTURE-EDGE-GRAZE",
            graze,
            {30'000'000, 0, 0},
            {1'025'000, 225'000, 500'000},
            {});
        validate_internal_end(graze.end);
        validate_chord(graze_start, graze.end);

        const Vec3 edge_start = {0.9, 0.22, 0.5};
        const Projection edge = project_internal(edge_start, {30.0, 0.0, 0.0});
        expect_vector(
            "APERTURE-EDGE-IMPACT",
            edge,
            {26'544'000, 4'608'000, 0},
            {1'010'600, 239'200, 500'000},
            {17});
        validate_internal_end(edge.end);
        validate_chord(edge_start, edge.end);

        const Vec3 corner_start = {0.9, 0.2, 0.4};
        const Projection corner = project_internal(corner_start, {30.0, 0.0, 0.0});
        expect_vector(
            "APERTURE-CORNER-SPHERE",
            corner,
            {18'000'000, 0, 0},
            {975'000, 200'000, 400'000},
            {21});
        validate_internal_end(corner.end);
        validate_chord(corner_start, corner.end);

        const Vec3 internal_face_start = {0.9, 0.1, 0.5};
        const Projection internal_face =
            project_internal(internal_face_start, {30.0, 0.0, 0.0});
        expect_vector(
            "INTERNAL-FACE-SENTINEL",
            internal_face,
            {18'000'000, 0, 0},
            {975'000, 100'000, 500'000},
            {16});
        validate_internal_end(internal_face.end);
        validate_chord(internal_face_start, internal_face.end);

        const double contact_x = 1.0 - std::sqrt(
            (RADIUS * RADIUS) - ((0.22 - 0.2) * (0.22 - 0.2)));
        const Vec3 restart_start = {
            std::nextafter(contact_x, 1.0),
            0.22,
            0.5,
        };
        const Projection restart = project_internal(restart_start, {30.0, 0.0, 0.0});
        expect_vector(
            "EDGE-T0-RESTART-SENTINEL",
            restart,
            {19'200'000, 14'400'000, 0},
            {1'065'000, 280'000, 500'000},
            {17});
        validate_internal_end(restart.end);
        validate_chord(restart_start, restart.end);

        const std::array<std::uint32_t, 9> base_order = {
            16, 17, 18, 19, 20, 21, 22, 23, 24,
        };
        auto order_mutation = base_order;
        std::swap(order_mutation[1], order_mutation[2]);
        const std::string profile_root =
            nextengine::nonlocal::sha256_hex(profile_projection(RADIUS_BITS, base_order));
        const std::string order_mutation_root =
            nextengine::nonlocal::sha256_hex(profile_projection(RADIUS_BITS, order_mutation));
        const std::string radius_mutation_root = nextengine::nonlocal::sha256_hex(
            profile_projection(RADIUS_BITS + UINT64_C(1), base_order));
        if (profile_root == order_mutation_root || profile_root == radius_mutation_root
            || order_mutation_root == radius_mutation_root) {
            throw std::runtime_error("contact profile mutation root collision");
        }

        std::ostringstream output;
        output << "schema=" << SCHEMA << '\n'
               << "contract_identity=" << CONTRACT_IDENTITY << '\n'
               << "parent_identity=" << PARENT_IDENTITY << '\n'
               << "upstream_library_sha256=" << UPSTREAM_LIBRARY_SHA256 << '\n'
               << "status=PASS\n"
               << "contact_started=true\n"
               << "trajectory_started=false\n"
               << "abi.real_bytes=" << sizeof(Real) << '\n'
               << "abi.real_iec559=" << std::boolalpha << std::numeric_limits<Real>::is_iec559
               << '\n'
               << "abi.real_digits=" << std::numeric_limits<Real>::digits << '\n'
               << "abi.pointer_bytes=" << sizeof(void *) << '\n'
               << "abi.endian=little\n"
               << "abi.dfsph_method=" << SPH::TimeStepDFSPH::METHOD_NAME << '\n'
               << "environment.locale=C\n"
               << "environment.rounding=nearest\n"
               << "environment.ftz=false\n"
               << "environment.daz=false\n"
               << "environment.omp_threads=1\n"
               << "environment.omp_dynamic=false\n"
               << fixture_line("OUTER-FACE-INTERIOR", outer_face)
               << fixture_line("OUTER-HIGH-SPEED", outer_fast)
               << fixture_line("APERTURE-PASS", pass)
               << fixture_line("APERTURE-EDGE-GRAZE", graze)
               << fixture_line("APERTURE-EDGE-IMPACT", edge)
               << fixture_line("APERTURE-CORNER-SPHERE", corner)
               << fixture_line("INTERNAL-FACE-SENTINEL", internal_face)
               << fixture_line("EDGE-T0-RESTART-SENTINEL", restart)
               << "profile_root=" << profile_root << '\n'
               << "order_mutation_root=" << order_mutation_root << '\n'
               << "radius_mutation_root=" << radius_mutation_root << '\n'
               << "r1c_design_authorized=true\n"
               << "b4e_design_authorized=false\n";
        return {true, output.str()};
    } catch (const std::exception &error) {
        std::ostringstream output;
        output << "schema=" << SCHEMA << '\n'
               << "contract_identity=" << CONTRACT_IDENTITY << '\n'
               << "status=FAIL\n"
               << "reason=" << error.what() << '\n'
               << "contact_started=true\n"
               << "trajectory_started=false\n";
        return {false, output.str()};
    }
}

} // namespace nextengine::nonlocal_reference
