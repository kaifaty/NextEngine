#include "boundary_discriminator.hpp"

#include "oracle.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <cmath>
#include <iomanip>
#include <sstream>
#include <stdexcept>

namespace nextengine::nonlocal {
namespace {

constexpr double SPACING = 0.05;
constexpr double RADIUS = 0.025;
constexpr double TIME_STEP = 1.0 / 240.0;
constexpr double MASS = 0.125;
constexpr int OUTER_Y_MIN_FEATURE_ID = 2;

void append_independent_two_layer_complement(Fixture& fixture) {
    constexpr int cells = 2;
    constexpr int layers = 2;
    for (int x = -layers; x < cells + layers; ++x) {
        for (int y = -layers; y < cells + layers; ++y) {
            for (int z = -layers; z < cells + layers; ++z) {
                if (x >= 0 && x < cells && y >= 0 && y < cells
                    && z >= 0 && z < cells) {
                    continue;
                }
                fixture.particles.push_back({
                    {
                        RADIUS + x * SPACING,
                        RADIUS + y * SPACING,
                        RADIUS + z * SPACING,
                    },
                    {},
                    true,
                });
            }
        }
    }
}

struct ContactResult {
    Vec3 position;
    Vec3 velocity;
    Vec3 fluid_impulse;
    Vec3 boundary_reaction;
    double time_of_impact = 1.0;
    bool active = false;
};

ContactResult apply_bottom_swept_sphere(
    Vec3 start,
    Vec3 tentative,
    double time_step,
    double mass) {
    ContactResult result;
    result.position = tentative;
    result.velocity = (tentative - start) / time_step;
    const Vec3 incoming_velocity = result.velocity;
    const Vec3 displacement = tentative - start;
    if (tentative.y >= RADIUS || displacement.y >= 0.0) {
        return result;
    }
    const double time_of_impact = (RADIUS - start.y) / displacement.y;
    if (!std::isfinite(time_of_impact) || time_of_impact < 0.0 || time_of_impact > 1.0) {
        throw std::runtime_error("invalid swept-sphere time of impact");
    }
    const Vec3 hit = start + displacement * time_of_impact;
    Vec3 remainder = displacement * (1.0 - time_of_impact);
    remainder.y = std::max(remainder.y, 0.0);
    result.position = hit + remainder;
    result.velocity = (result.position - start) / time_step;
    result.fluid_impulse = (result.velocity - incoming_velocity) * mass;
    result.boundary_reaction = result.fluid_impulse * -1.0;
    result.time_of_impact = time_of_impact;
    result.active = true;
    return result;
}

void append_vec3(std::ostringstream& output, Vec3 value) {
    output << '[' << value.x << ',' << value.y << ',' << value.z << ']';
}

bool finite_result(const OracleResult& result) {
    return std::all_of(result.density.begin(), result.density.end(),
               [](double value) { return std::isfinite(value); })
        && std::all_of(result.source.begin(), result.source.end(),
            [](Vec3 value) { return finite(value); })
        && std::all_of(result.local_matrix.begin(), result.local_matrix.end(),
            [](const Mat3& value) { return finite(value); })
        && std::all_of(result.next_position.begin(), result.next_position.end(),
            [](Vec3 value) { return finite(value); })
        && std::all_of(result.final_velocity.begin(), result.final_velocity.end(),
            [](Vec3 value) { return finite(value); })
        && std::isfinite(result.normalized_momentum_residual);
}

} // namespace

CpuBoundaryDiscriminatorReport run_cpu_boundary_discriminator() {
    Fixture fixture;
    fixture.name = "npr0_tiny_two_layer_ghost_wall_crossing";
    fixture.rest_density = 1000.0;
    fixture.spacing = SPACING;
    fixture.mass = MASS;
    fixture.horizon = 0.10;
    fixture.time_step = TIME_STEP;
    fixture.gravity = {0.0, -9.81, 0.0};
    fixture.kappa = 576.0;
    fixture.lambda = 360.0;
    fixture.terms = {true, true, false, false};
    fixture.iterations = 4;
    fixture.particles.push_back({
        {RADIUS, 0.075, RADIUS},
        {0.0, -120.0, 0.0},
        false,
    });
    append_independent_two_layer_complement(fixture);
    constexpr std::size_t expected_boundary_samples = 208;
    if (fixture.particles.size() != expected_boundary_samples + 1U) {
        throw std::runtime_error("tiny independent boundary count mismatch");
    }

    const OracleResult ghost_only = run_cpu_gather_oracle(fixture);
    const Vec3 tentative = ghost_only.next_position.front();
    const ContactResult contact = apply_bottom_swept_sphere(
        fixture.particles.front().position, tentative, fixture.time_step, fixture.mass);
    double maximum_fixed_displacement = 0.0;
    for (std::size_t index = 1; index < fixture.particles.size(); ++index) {
        maximum_fixed_displacement = std::max(maximum_fixed_displacement,
            norm(ghost_only.next_position[index] - fixture.particles[index].position));
    }
    const double impulse_closure =
        norm(contact.fluid_impulse + contact.boundary_reaction);
    const bool ghost_support_only_penetrates = tentative.y < RADIUS;
    const bool contact_nonpenetrating = contact.active
        && contact.position.y >= RADIUS - 1.0e-15;
    const bool passed = finite_result(ghost_only)
        && ghost_only.directed_pairs > 0U
        && maximum_fixed_displacement <= 1.0e-15
        && ghost_support_only_penetrates
        && contact_nonpenetrating
        && impulse_closure <= 1.0e-15;

    std::ostringstream root_material;
    root_material << std::setprecision(17)
                  << fixture.particles.size() << '|' << ghost_only.directed_pairs << '|'
                  << tentative.x << ',' << tentative.y << ',' << tentative.z << '|'
                  << contact.position.x << ',' << contact.position.y << ','
                  << contact.position.z << '|' << contact.time_of_impact << '|'
                  << contact.fluid_impulse.x << ',' << contact.fluid_impulse.y << ','
                  << contact.fluid_impulse.z;

    std::ostringstream output;
    output << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.cpu_boundary_discriminator.v1\""
           << ",\"identity\":\"npr0-split-static-boundary-r0\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
           << ",\"semantic_outcome\":\""
           << (passed ? "SPLIT_BOUNDARY_REQUIRED" : "BOUNDARY_DISCRIMINATOR_INVALID")
           << "\",\"fixture\":{\"fluid_samples\":1,\"static_boundary_samples\":"
           << expected_boundary_samples << ",\"total_solver_samples\":"
           << fixture.particles.size() << ",\"spacing_m\":" << fixture.spacing
           << ",\"horizon_m\":" << fixture.horizon << ",\"time_step_s\":"
           << fixture.time_step << ",\"particle_radius_m\":" << RADIUS
           << ",\"initial_velocity_m_s\":";
    append_vec3(output, fixture.particles.front().velocity);
    output << "},\"ghost_support\":{\"identity\":\"two-layer-rest-volume-fixed-samples\""
           << ",\"directed_pairs\":" << ghost_only.directed_pairs
           << ",\"maximum_degree\":" << ghost_only.maximum_degree
           << ",\"maximum_fixed_displacement_m\":" << maximum_fixed_displacement
           << ",\"tentative_position_m\":";
    append_vec3(output, tentative);
    output << ",\"penetrates_bottom_wall\":"
           << (ghost_support_only_penetrates ? "true" : "false")
           << "},\"contact_counterfactual\":{\"identity\":"
              "\"spec38-static-swept-sphere-bottom-face-r0\""
           << ",\"feature_id\":" << OUTER_Y_MIN_FEATURE_ID
           << ",\"active\":" << (contact.active ? "true" : "false")
           << ",\"time_of_impact_fraction\":" << contact.time_of_impact
           << ",\"accepted_position_m\":";
    append_vec3(output, contact.position);
    output << ",\"accepted_velocity_m_s\":";
    append_vec3(output, contact.velocity);
    output << ",\"fluid_impulse_kg_m_s\":";
    append_vec3(output, contact.fluid_impulse);
    output << ",\"boundary_reaction_kg_m_s\":";
    append_vec3(output, contact.boundary_reaction);
    output << ",\"impulse_closure_absolute\":" << impulse_closure
           << ",\"nonpenetrating\":" << (contact_nonpenetrating ? "true" : "false")
           << "},\"result_sha256\":\"" << sha256_hex(root_material.str()) << "\""
           << ",\"claim\":\"fixed ghost samples restore support but do not seal the basin; "
              "a separately specified contact operation remains mandatory\"}";
    return {passed, output.str()};
}

} // namespace nextengine::nonlocal
