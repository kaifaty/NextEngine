#include "tiny_corpus.hpp"

#include "oracle.hpp"
#include "profiles.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <iomanip>
#include <limits>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

namespace nextengine::nonlocal {
namespace {

constexpr double PI = 3.141592653589793238462643383279502884;
constexpr double SPACING = 0.05;
constexpr double RADIUS = 0.025;
constexpr double MASS = 0.125;
constexpr double HORIZON = 0.10;
constexpr double TIME_STEP = 1.0 / 240.0;
constexpr double REST_DENSITY = 1000.0;
constexpr int ITERATIONS = 4;

struct Box {
    Vec3 minimum;
    Vec3 maximum;
    std::array<int, 3> cells;
};

struct SweepResult {
    Vec3 position;
    Vec3 velocity;
    Vec3 fluid_impulse;
    Vec3 reaction;
    std::vector<int> features;
    double maximum_penetration = 0.0;
};

struct CaseResult {
    bool passed = false;
    std::string first_failure;
    std::string json;
};

struct ProfileResult {
    std::string id;
    std::string profile_sha256;
    bool passed = false;
    std::string first_failure;
    CaseResult free_fall;
    CaseResult hydro;
    CaseResult reversible;
    CaseResult wall;
    double hydro_mean_positive_compression = 0.0;
};

Fixture base_fixture(const Profile& profile, std::string name) {
    Fixture fixture;
    fixture.name = std::move(name);
    fixture.rest_density = REST_DENSITY;
    fixture.spacing = SPACING;
    fixture.mass = MASS;
    fixture.horizon = HORIZON;
    fixture.time_step = TIME_STEP;
    fixture.gravity = profile.gravity;
    fixture.kappa = profile.kappa;
    fixture.lambda = profile.lambda;
    fixture.mu = profile.mu;
    fixture.gamma = profile.gamma;
    fixture.terms = profile.terms;
    fixture.iterations = ITERATIONS;
    return fixture;
}

std::size_t append_two_layer_complement(Fixture& fixture, const Box& box) {
    constexpr int layers = 2;
    const std::size_t before = fixture.particles.size();
    for (int x = -layers; x < box.cells[0] + layers; ++x) {
        for (int y = -layers; y < box.cells[1] + layers; ++y) {
            for (int z = -layers; z < box.cells[2] + layers; ++z) {
                if (x >= 0 && x < box.cells[0] && y >= 0 && y < box.cells[1]
                    && z >= 0 && z < box.cells[2]) {
                    continue;
                }
                fixture.particles.push_back({
                    {
                        box.minimum.x + RADIUS + x * SPACING,
                        box.minimum.y + RADIUS + y * SPACING,
                        box.minimum.z + RADIUS + z * SPACING,
                    },
                    {},
                    true,
                });
            }
        }
    }
    return fixture.particles.size() - before;
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
            [](Vec3 value) { return finite(value); });
}

double axis(Vec3 value, int component) {
    if (component == 0) {
        return value.x;
    }
    if (component == 1) {
        return value.y;
    }
    return value.z;
}

void set_axis(Vec3& value, int component, double scalar) {
    if (component == 0) {
        value.x = scalar;
    } else if (component == 1) {
        value.y = scalar;
    } else {
        value.z = scalar;
    }
}

SweepResult sweep_box(Vec3 start, Vec3 tentative, const Box& box) {
    SweepResult result;
    result.position = tentative;
    const Vec3 incoming_velocity = (tentative - start) / TIME_STEP;
    const std::array<double, 3> lower = {
        box.minimum.x + RADIUS,
        box.minimum.y + RADIUS,
        box.minimum.z + RADIUS,
    };
    const std::array<double, 3> upper = {
        box.maximum.x - RADIUS,
        box.maximum.y - RADIUS,
        box.maximum.z - RADIUS,
    };
    const std::array<int, 3> lower_feature = {0, 2, 4};
    const std::array<int, 3> upper_feature = {1, 3, 5};
    for (int component = 0; component < 3; ++component) {
        const double start_value = axis(start, component);
        const double tentative_value = axis(tentative, component);
        const double displacement = tentative_value - start_value;
        if (tentative_value < lower[component] && displacement < 0.0) {
            const double toi = (lower[component] - start_value) / displacement;
            if (!std::isfinite(toi) || toi < 0.0 || toi > 1.0) {
                throw std::runtime_error("invalid lower swept-box time of impact");
            }
            set_axis(result.position, component, lower[component]);
            result.features.push_back(lower_feature[component]);
        } else if (tentative_value > upper[component] && displacement > 0.0) {
            const double toi = (upper[component] - start_value) / displacement;
            if (!std::isfinite(toi) || toi < 0.0 || toi > 1.0) {
                throw std::runtime_error("invalid upper swept-box time of impact");
            }
            set_axis(result.position, component, upper[component]);
            result.features.push_back(upper_feature[component]);
        }
    }
    std::sort(result.features.begin(), result.features.end());
    result.velocity = (result.position - start) / TIME_STEP;
    result.fluid_impulse = (result.velocity - incoming_velocity) * MASS;
    result.reaction = result.fluid_impulse * -1.0;
    for (int component = 0; component < 3; ++component) {
        result.maximum_penetration = std::max(result.maximum_penetration,
            std::max(lower[component] - axis(result.position, component),
                axis(result.position, component) - upper[component]));
    }
    result.maximum_penetration = std::max(result.maximum_penetration, 0.0);
    return result;
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

double cubic_scale() {
    const int half_resolution = static_cast<int>(HORIZON / SPACING + 1.0);
    double total = 0.0;
    for (int x = -half_resolution; x <= half_resolution; ++x) {
        for (int y = -half_resolution; y <= half_resolution; ++y) {
            for (int z = -half_resolution; z <= half_resolution; ++z) {
                const Vec3 offset{x * SPACING, y * SPACING, z * SPACING};
                total += SPACING * SPACING * SPACING
                    * cubic_weight(norm(offset), HORIZON);
            }
        }
    }
    return 1.0 / total;
}

std::vector<double> independent_density(
    const std::vector<Particle>& particles,
    std::size_t fluid_count) {
    const double scale = cubic_scale();
    std::vector<double> density(fluid_count, 0.0);
    for (std::size_t i = 0; i < fluid_count; ++i) {
        for (const Particle& particle : particles) {
            density[i] += MASS
                * cubic_weight(norm(particles[i].position - particle.position), HORIZON)
                * scale;
        }
    }
    return density;
}

void append_vec3(std::ostringstream& output, Vec3 value) {
    output << '[' << value.x << ',' << value.y << ',' << value.z << ']';
}

void append_features(std::ostringstream& output, const std::vector<int>& features) {
    output << '[';
    for (std::size_t index = 0; index < features.size(); ++index) {
        if (index != 0U) {
            output << ',';
        }
        output << features[index];
    }
    output << ']';
}

CaseResult run_free_fall(const Profile& profile) {
    Particle particle{{0.0, 1.0, 0.0}, {}, false};
    Vec3 expected_position = particle.position;
    Vec3 expected_velocity{};
    double maximum_position_error = 0.0;
    double maximum_velocity_error = 0.0;
    bool finite_all = true;
    for (int step = 0; step < 16; ++step) {
        Fixture fixture = base_fixture(profile, "TPF-1");
        fixture.particles = {particle};
        const OracleResult result = run_cpu_gather_oracle(fixture);
        finite_all = finite_all && finite_result(result);
        expected_velocity += fixture.gravity * TIME_STEP;
        expected_position += expected_velocity * TIME_STEP;
        maximum_position_error = std::max(
            maximum_position_error, norm(result.next_position.front() - expected_position));
        maximum_velocity_error = std::max(
            maximum_velocity_error, norm(result.final_velocity.front() - expected_velocity));
        particle.position = result.next_position.front();
        particle.velocity = result.final_velocity.front();
    }
    const bool passed = finite_all && maximum_position_error <= 1.0e-12
        && maximum_velocity_error <= 1.0e-12;
    std::ostringstream json;
    json << std::setprecision(17)
         << "{\"case\":\"TPF-1\",\"passed\":" << (passed ? "true" : "false")
         << ",\"maximum_position_error_m\":" << maximum_position_error
         << ",\"maximum_velocity_error_m_s\":" << maximum_velocity_error
         << ",\"contact_count\":0}";
    return {passed, passed ? "" : "exact_free_fall", json.str()};
}

CaseResult run_hydro(const Profile& profile, double& mean_positive_compression) {
    const Box box{{0.0, 0.0, 0.0}, {0.1, 0.2, 0.1}, {2, 4, 2}};
    std::vector<Particle> fluid;
    for (int x = 0; x < 2; ++x) {
        for (int y = 0; y < 2; ++y) {
            for (int z = 0; z < 2; ++z) {
                fluid.push_back({
                    {RADIUS + x * SPACING, RADIUS + y * SPACING,
                        RADIUS + z * SPACING},
                    {},
                    false,
                });
            }
        }
    }
    const Vec3 initial_com{0.05, 0.05, 0.05};
    double maximum_fixed_displacement = 0.0;
    double maximum_penetration = 0.0;
    double maximum_speed = 0.0;
    bool finite_all = true;
    std::vector<Particle> final_all;
    for (int step = 0; step < 24; ++step) {
        Fixture fixture = base_fixture(profile, "TPH-1");
        fixture.particles = fluid;
        const std::size_t boundary_count = append_two_layer_complement(fixture, box);
        if (boundary_count != 272U || fixture.particles.size() != 280U) {
            throw std::runtime_error("TPH-1 support count mismatch");
        }
        const OracleResult tentative = run_cpu_gather_oracle(fixture);
        finite_all = finite_all && finite_result(tentative);
        for (std::size_t index = fluid.size(); index < fixture.particles.size(); ++index) {
            maximum_fixed_displacement = std::max(maximum_fixed_displacement,
                norm(tentative.next_position[index] - fixture.particles[index].position));
        }
        for (std::size_t index = 0; index < fluid.size(); ++index) {
            const SweepResult contact = sweep_box(
                fluid[index].position, tentative.next_position[index], box);
            maximum_penetration =
                std::max(maximum_penetration, contact.maximum_penetration);
            fluid[index].position = contact.position;
            fluid[index].velocity = contact.velocity;
            maximum_speed = std::max(maximum_speed, norm(contact.velocity));
        }
        final_all = fluid;
        Fixture support = base_fixture(profile, "TPH-1-density");
        support.particles = final_all;
        append_two_layer_complement(support, box);
        final_all = std::move(support.particles);
    }
    const std::vector<double> density = independent_density(final_all, fluid.size());
    double maximum_positive_compression = 0.0;
    mean_positive_compression = 0.0;
    for (double value : density) {
        const double positive = std::max(value / REST_DENSITY - 1.0, 0.0);
        mean_positive_compression += positive;
        maximum_positive_compression = std::max(maximum_positive_compression, positive);
    }
    mean_positive_compression /= static_cast<double>(density.size());
    Vec3 final_com{};
    for (const Particle& particle : fluid) {
        final_com += particle.position;
    }
    final_com = final_com / static_cast<double>(fluid.size());
    const double horizontal_drift = std::hypot(
        final_com.x - initial_com.x, final_com.z - initial_com.z);
    const bool passed = finite_all && fluid.size() == 8U
        && std::abs(static_cast<double>(fluid.size()) * MASS - 1.0) <= 1.0e-15
        && mean_positive_compression <= 1.0e-4
        && maximum_penetration <= 0.0025
        && horizontal_drift <= 1.0e-10
        && maximum_fixed_displacement <= 1.0e-15;
    std::string failure;
    if (!finite_all) {
        failure = "nonfinite";
    } else if (mean_positive_compression > 1.0e-4) {
        failure = "mean_positive_compression";
    } else if (maximum_penetration > 0.0025) {
        failure = "boundary_penetration";
    } else if (horizontal_drift > 1.0e-10) {
        failure = "horizontal_com_drift";
    } else if (maximum_fixed_displacement > 1.0e-15) {
        failure = "fixed_support_moved";
    }
    std::ostringstream json;
    json << std::setprecision(17)
         << "{\"case\":\"TPH-1\",\"passed\":" << (passed ? "true" : "false")
         << ",\"first_failure\":\"" << failure << "\",\"fluid_samples\":"
         << fluid.size() << ",\"fluid_mass_kg\":"
         << static_cast<double>(fluid.size()) * MASS
         << ",\"static_boundary_samples\":272,\"total_solver_samples\":280"
         << ",\"mean_positive_compression\":" << mean_positive_compression
         << ",\"maximum_positive_compression\":" << maximum_positive_compression
         << ",\"maximum_penetration_m\":" << maximum_penetration
         << ",\"horizontal_com_drift_m\":" << horizontal_drift
         << ",\"maximum_speed_m_s\":" << maximum_speed
         << ",\"maximum_fixed_displacement_m\":" << maximum_fixed_displacement
         << ",\"final_com_m\":";
    append_vec3(json, final_com);
    json << '}';
    return {passed, failure, json.str()};
}

CaseResult run_reversible(const Profile& profile) {
    const Vec3 uniform_velocity{0.2, -0.1, 0.15};
    std::vector<Particle> particles;
    for (int x = 0; x < 2; ++x) {
        for (int y = 0; y < 2; ++y) {
            for (int z = 0; z < 2; ++z) {
                particles.push_back({
                    {x * SPACING, y * SPACING, z * SPACING},
                    uniform_velocity,
                    false,
                });
            }
        }
    }
    const std::vector<Particle> initial = particles;
    Fixture forward = base_fixture(profile, "TPR-1-forward");
    forward.gravity = {};
    forward.particles = particles;
    const OracleResult first = run_cpu_gather_oracle(forward);
    Fixture reverse = base_fixture(profile, "TPR-1-reverse");
    reverse.gravity = {};
    for (std::size_t index = 0; index < particles.size(); ++index) {
        reverse.particles.push_back({
            first.next_position[index],
            first.final_velocity[index] * -1.0,
            false,
        });
    }
    const OracleResult second = run_cpu_gather_oracle(reverse);
    double maximum_position_error = 0.0;
    double maximum_velocity_error = 0.0;
    for (std::size_t index = 0; index < initial.size(); ++index) {
        maximum_position_error = std::max(maximum_position_error,
            norm(second.next_position[index] - initial[index].position) / SPACING);
        maximum_velocity_error = std::max(maximum_velocity_error,
            norm(second.final_velocity[index] + uniform_velocity)
                / norm(uniform_velocity));
    }
    const bool passed = finite_result(first) && finite_result(second)
        && maximum_position_error <= 1.0e-9
        && maximum_velocity_error <= 1.0e-9;
    std::ostringstream json;
    json << std::setprecision(17)
         << "{\"case\":\"TPR-1\",\"passed\":" << (passed ? "true" : "false")
         << ",\"maximum_return_position_over_spacing\":"
         << maximum_position_error
         << ",\"maximum_return_velocity_over_initial_speed\":"
         << maximum_velocity_error << '}';
    return {passed, passed ? "" : "reversible_rigid_mode", json.str()};
}

CaseResult run_wall(const Profile& profile) {
    const Box box{{0.0, 0.0, 0.0}, {0.1, 0.1, 0.1}, {2, 2, 2}};
    const std::array<Vec3, 2> starts = {
        Vec3{RADIUS, 0.075, RADIUS},
        Vec3{0.075, 0.075, 0.075},
    };
    const std::array<Vec3, 2> velocities = {
        Vec3{0.0, -120.0, 0.0},
        Vec3{-120.0, -120.0, -120.0},
    };
    const std::array<std::vector<int>, 2> expected = {
        std::vector<int>{2},
        std::vector<int>{0, 2, 4},
    };
    double maximum_penetration = 0.0;
    double maximum_impulse_closure = 0.0;
    double maximum_fixed_displacement = 0.0;
    bool exact_features = true;
    bool finite_all = true;
    std::ostringstream cases;
    cases << '[';
    for (std::size_t case_index = 0; case_index < starts.size(); ++case_index) {
        Fixture fixture = base_fixture(profile, "TPW-1");
        fixture.particles.push_back({starts[case_index], velocities[case_index], false});
        const std::size_t boundary_count = append_two_layer_complement(fixture, box);
        if (boundary_count != 208U || fixture.particles.size() != 209U) {
            throw std::runtime_error("TPW-1 support count mismatch");
        }
        const OracleResult tentative = run_cpu_gather_oracle(fixture);
        finite_all = finite_all && finite_result(tentative);
        for (std::size_t index = 1; index < fixture.particles.size(); ++index) {
            maximum_fixed_displacement = std::max(maximum_fixed_displacement,
                norm(tentative.next_position[index] - fixture.particles[index].position));
        }
        const SweepResult contact =
            sweep_box(starts[case_index], tentative.next_position.front(), box);
        maximum_penetration =
            std::max(maximum_penetration, contact.maximum_penetration);
        maximum_impulse_closure = std::max(
            maximum_impulse_closure, norm(contact.fluid_impulse + contact.reaction));
        exact_features = exact_features && contact.features == expected[case_index];
        if (case_index != 0U) {
            cases << ',';
        }
        cases << "{\"kind\":\"" << (case_index == 0U ? "face" : "corner")
              << "\",\"features\":";
        append_features(cases, contact.features);
        cases << ",\"accepted_position_m\":";
        append_vec3(cases, contact.position);
        cases << '}';
    }
    cases << ']';
    const bool passed = finite_all && exact_features
        && maximum_penetration <= 1.0e-12
        && maximum_impulse_closure <= 1.0e-12
        && maximum_fixed_displacement <= 1.0e-15;
    std::ostringstream json;
    json << std::setprecision(17)
         << "{\"case\":\"TPW-1\",\"passed\":" << (passed ? "true" : "false")
         << ",\"exact_features\":" << (exact_features ? "true" : "false")
         << ",\"maximum_penetration_m\":" << maximum_penetration
         << ",\"maximum_impulse_closure\":" << maximum_impulse_closure
         << ",\"maximum_fixed_displacement_m\":" << maximum_fixed_displacement
         << ",\"subcases\":" << cases.str() << '}';
    return {passed, passed ? "" : "face_or_corner_contact", json.str()};
}

ProfileResult evaluate_profile(const Profile& profile) {
    ProfileResult result;
    result.id = profile.id;
    result.profile_sha256 = sha256_hex(canonical_profile_json(profile));
    result.free_fall = run_free_fall(profile);
    result.hydro = run_hydro(profile, result.hydro_mean_positive_compression);
    result.reversible = run_reversible(profile);
    result.wall = run_wall(profile);
    const std::array<const CaseResult*, 4> cases = {
        &result.free_fall, &result.hydro, &result.reversible, &result.wall};
    result.passed = true;
    for (const CaseResult* item : cases) {
        result.passed = result.passed && item->passed;
        if (!item->passed && result.first_failure.empty()) {
            result.first_failure = item->first_failure;
        }
    }
    return result;
}

void append_profile(std::ostringstream& output, const ProfileResult& profile) {
    output << "{\"profile_id\":\"" << profile.id << "\",\"profile_sha256\":\""
           << profile.profile_sha256 << "\",\"passed\":"
           << (profile.passed ? "true" : "false") << ",\"first_failure\":\""
           << profile.first_failure << "\",\"cases\":[" << profile.free_fall.json << ','
           << profile.hydro.json << ',' << profile.reversible.json << ','
           << profile.wall.json << "]}";
}

} // namespace

CpuTinyCorpusReport run_cpu_tiny_physical_corpus() {
    const ProfileResult control =
        evaluate_profile(find_profile("nuv-basin-48k-static-support-control.v3"));
    const ProfileResult derived =
        evaluate_profile(find_profile("nuv-basin-48k-static-support-derived.v3"));

    std::string selected;
    std::string disposition;
    if (control.passed != derived.passed) {
        selected = control.passed ? control.id : derived.id;
        disposition = "NONLOCAL_PRODUCT_PROFILE_CANDIDATE";
    } else if (control.passed && derived.passed) {
        selected = derived.hydro_mean_positive_compression
                <= control.hydro_mean_positive_compression
            ? derived.id : control.id;
        disposition = "NONLOCAL_PRODUCT_PROFILE_CANDIDATE";
    } else {
        disposition = "PROFILE_RECLOSURE_REMEDIATION_1";
    }
    const bool command_passed = true;
    std::ostringstream root;
    root << std::setprecision(17)
         << control.profile_sha256 << '|' << control.passed << '|'
         << control.first_failure << '|' << control.hydro_mean_positive_compression << '|'
         << derived.profile_sha256 << '|' << derived.passed << '|'
         << derived.first_failure << '|' << derived.hydro_mean_positive_compression << '|'
         << disposition << '|' << selected;

    std::ostringstream output;
    output << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.cpu_tiny_physical_corpus.v1\""
           << ",\"corpus_identity\":\"npr0-tiny-physical-corpus-r0\""
           << ",\"status\":\"PASS\",\"command_passed\":true"
           << ",\"gates\":{\"free_fall_position_m\":1e-12"
           << ",\"free_fall_velocity_m_s\":1e-12"
           << ",\"hydro_mean_positive_compression\":0.0001"
           << ",\"hydro_maximum_penetration_m\":0.0025"
           << ",\"hydro_horizontal_com_drift_m\":1e-10"
           << ",\"reversible_position_over_spacing\":1e-9"
           << ",\"reversible_velocity_over_initial_speed\":1e-9"
           << ",\"wall_penetration_m\":1e-12"
           << ",\"wall_impulse_closure\":1e-12}"
           << ",\"profiles\":[";
    append_profile(output, control);
    output << ',';
    append_profile(output, derived);
    output << "],\"selection\":{\"disposition\":\"" << disposition
           << "\",\"selected_profile_id\":\"" << selected
           << "\",\"runtime_authority\":false,\"npr1_authorized\":"
           << (!selected.empty() ? "true" : "false") << "}"
           << ",\"result_sha256\":\"" << sha256_hex(root.str()) << "\"}";
    return {command_passed, output.str()};
}

} // namespace nextengine::nonlocal
