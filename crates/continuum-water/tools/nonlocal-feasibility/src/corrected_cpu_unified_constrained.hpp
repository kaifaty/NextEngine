#pragma once

#include <array>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <string>
#include <string_view>
#include <vector>

namespace nextengine::nonlocal::ncgp15 {

struct Vec3l {
    long double x = 0.0L;
    long double y = 0.0L;
    long double z = 0.0L;
};

inline Vec3l operator+(Vec3l lhs, Vec3l rhs) {
    return {lhs.x + rhs.x, lhs.y + rhs.y, lhs.z + rhs.z};
}
inline Vec3l operator-(Vec3l lhs, Vec3l rhs) {
    return {lhs.x - rhs.x, lhs.y - rhs.y, lhs.z - rhs.z};
}
inline Vec3l operator-(Vec3l value) {
    return {-value.x, -value.y, -value.z};
}
inline Vec3l operator*(Vec3l value, long double scale) {
    return {value.x * scale, value.y * scale, value.z * scale};
}
inline Vec3l operator*(long double scale, Vec3l value) {
    return value * scale;
}
inline Vec3l operator/(Vec3l value, long double scale) {
    return {value.x / scale, value.y / scale, value.z / scale};
}
inline Vec3l& operator+=(Vec3l& lhs, Vec3l rhs) {
    lhs = lhs + rhs;
    return lhs;
}
inline Vec3l& operator-=(Vec3l& lhs, Vec3l rhs) {
    lhs = lhs - rhs;
    return lhs;
}
inline Vec3l& operator*=(Vec3l& value, long double scale) {
    value = value * scale;
    return value;
}
inline long double dot15(Vec3l lhs, Vec3l rhs) {
    return lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z;
}
inline long double norm15(Vec3l value) {
    return std::sqrt(dot15(value, value));
}
inline bool finite15(Vec3l value) {
    return std::isfinite(value.x) && std::isfinite(value.y)
        && std::isfinite(value.z);
}

struct Gates15 {
    long double inner_rms_relative = 1.0e-9L;
    long double inner_max_relative = 1.0e-8L;
    long double density_maximum = 1.0e-3L;
    long double density_rms = 2.5e-4L;
    long double kkt_rms_m = 1.0e-6L;
    long double kkt_maximum_m = 5.0e-6L;
    long double complementarity = 1.0e-8L;
    long double multiplier_fixed_point = 1.0e-8L;
    long double density_relative_l2 = 2.0e-12L;
    long double gradient_relative_l2 = 1.0e-8L;
    long double gradient_maximum_n = 1.0e-7L;
    long double pair_position_m = 5.0e-6L;
    long double normal_decay_relative = 1.0e-10L;
    long double tangential_change_absolute = 1.0e-12L;
    long double center_of_mass_relative = 1.0e-12L;
    long double surface_energy_relative_excess = 1.0e-8L;
    long double surface_energy_absolute_excess_j = 1.0e-10L;
    long double reversible_energy_drift = 1.0e-2L;
    long double one_step_position_rms_m = 1.0e-6L;
    long double one_step_position_maximum_m = 5.0e-6L;
    long double trajectory_position_rms_m = 2.5e-3L;
    long double trajectory_position_maximum_m = 5.0e-3L;
    long double trajectory_velocity_rms_mps = 5.0e-2L;
    long double trajectory_velocity_maximum_mps = 1.0e-1L;
    long double momentum_residual = 1.0e-2L;
    long double energy_excess = 1.0e-2L;
    long double internal_force_closure = 1.0e-12L;
};

struct Profile15 {
    std::string id = "nonlocal-water-50k-v1";
    long double dt = static_cast<long double>(1.0 / 240.0);
    long double spacing = static_cast<long double>(0.05);
    long double horizon = static_cast<long double>(0.15);
    long double surface_r0 = static_cast<long double>(0.05);
    long double mass = static_cast<long double>(0.125);
    long double rho0 = static_cast<long double>(1000.0);
    long double kernel_scale =
        static_cast<long double>(7.985668078772472);
    long double beta = static_cast<long double>(1226.25);
    long double lambda_v =
        static_cast<long double>(1.4138231728735551e-5);
    long double mu_v = static_cast<long double>(0.0);
    long double gamma =
        static_cast<long double>(0.010664424039285813);
    Vec3l gravity{static_cast<long double>(0.0),
        static_cast<long double>(0.0), static_cast<long double>(-9.81)};
    Vec3l basin{static_cast<long double>(0.2),
        static_cast<long double>(0.2), static_cast<long double>(0.6)};
    std::uint32_t ghost_layers = 3U;
    std::uint32_t maximum_dynamic_samples = 50000U;
    std::uint32_t maximum_neighbors = 256U;
    std::uint32_t maximum_outer_updates = 64U;
    std::uint64_t maximum_accepted_inner_iterations = 8192U;
    std::uint32_t maximum_line_search_backtracks = 40U;
    long double armijo = 1.0e-4L;
    long double initial_alpha = 1.0L;
    long double line_search_shrink = 0.5L;
    long double finite_difference_scale = 0x1p-18L;
    Gates15 gates;
};

struct Dynamic15 {
    std::uint32_t id = 0U;
    Vec3l reference;
    Vec3l position;
    Vec3l velocity;
};

struct Ghost15 {
    std::uint32_t id = 0U;
    Vec3l position;
};

struct State15 {
    std::vector<Dynamic15> dynamic;
    std::vector<Ghost15> ghosts;
};

struct TermMask15 {
    bool density_constraint = true;
    bool normal_viscosity = true;
    bool tangential_viscosity = false;
    bool surface = true;
};

enum class GravityMode15 : std::uint8_t {
    Profile = 0U,
    Zero = 1U,
};

enum class BoundaryMode15 : std::uint8_t {
    AnalyticBox = 0U,
    UnboundedManufactured = 1U,
};

enum class Mutation15 : std::uint8_t {
    None = 0U,
    MissingKernelChain = 1U,
    HalfNormalViscosity = 2U,
    WrongSurfaceSign = 3U,
    MissingSurfaceFactorTwo = 4U,
    CurrentReferenceViscosityGraph = 5U,
    FinitePressurePenalty = 6U,
};

enum class MultiplierMode15 : std::uint8_t {
    AugmentedLagrangian = 0U,
    FinalKkt = 1U,
};

struct SolverOptions15 {
    Profile15 profile;
    TermMask15 terms;
    GravityMode15 gravity_mode = GravityMode15::Profile;
    BoundaryMode15 boundary_mode = BoundaryMode15::AnalyticBox;
    Mutation15 mutation = Mutation15::None;
};

struct Work15 {
    std::uint64_t fixture_records = 0U;
    std::uint64_t ghost_records = 0U;
    std::uint64_t graph_builds = 0U;
    std::uint64_t graph_candidates = 0U;
    std::uint64_t accepted_pairs = 0U;
    std::uint64_t density_pairs = 0U;
    std::uint64_t viscosity_pairs = 0U;
    std::uint64_t surface_pair_tests = 0U;
    std::uint64_t surface_active_pairs = 0U;
    std::uint64_t objective_evaluations = 0U;
    std::uint64_t gradient_evaluations = 0U;
    std::uint64_t jacobian_products = 0U;
    std::uint64_t projected_directions = 0U;
    std::uint64_t line_search_trials = 0U;
    std::uint64_t accepted_inner_iterations = 0U;
    std::uint64_t outer_multiplier_updates = 0U;
    std::uint64_t box_plane_tests = 0U;
    std::uint64_t contact_reaction_values = 0U;
    std::uint64_t oracle_pair_tests = 0U;
    std::uint64_t finite_difference_evaluations = 0U;
    std::uint64_t topology_edges = 0U;
    std::uint64_t scalar_predicates = 0U;
    std::uint64_t portable_fields = 0U;
    std::uint64_t hash_derivations = 0U;
};

inline std::array<std::uint64_t, 24> work_values15(const Work15& work) {
    return {work.fixture_records, work.ghost_records, work.graph_builds,
        work.graph_candidates, work.accepted_pairs, work.density_pairs,
        work.viscosity_pairs, work.surface_pair_tests,
        work.surface_active_pairs, work.objective_evaluations,
        work.gradient_evaluations, work.jacobian_products,
        work.projected_directions, work.line_search_trials,
        work.accepted_inner_iterations, work.outer_multiplier_updates,
        work.box_plane_tests, work.contact_reaction_values,
        work.oracle_pair_tests, work.finite_difference_evaluations,
        work.topology_edges, work.scalar_predicates, work.portable_fields,
        work.hash_derivations};
}

inline void add_work15(Work15& target, const Work15& value) {
    target.fixture_records += value.fixture_records;
    target.ghost_records += value.ghost_records;
    target.graph_builds += value.graph_builds;
    target.graph_candidates += value.graph_candidates;
    target.accepted_pairs += value.accepted_pairs;
    target.density_pairs += value.density_pairs;
    target.viscosity_pairs += value.viscosity_pairs;
    target.surface_pair_tests += value.surface_pair_tests;
    target.surface_active_pairs += value.surface_active_pairs;
    target.objective_evaluations += value.objective_evaluations;
    target.gradient_evaluations += value.gradient_evaluations;
    target.jacobian_products += value.jacobian_products;
    target.projected_directions += value.projected_directions;
    target.line_search_trials += value.line_search_trials;
    target.accepted_inner_iterations += value.accepted_inner_iterations;
    target.outer_multiplier_updates += value.outer_multiplier_updates;
    target.box_plane_tests += value.box_plane_tests;
    target.contact_reaction_values += value.contact_reaction_values;
    target.oracle_pair_tests += value.oracle_pair_tests;
    target.finite_difference_evaluations += value.finite_difference_evaluations;
    target.topology_edges += value.topology_edges;
    target.scalar_predicates += value.scalar_predicates;
    target.portable_fields += value.portable_fields;
    target.hash_derivations += value.hash_derivations;
}

inline bool equal_work15(const Work15& lhs, const Work15& rhs) {
    return work_values15(lhs) == work_values15(rhs);
}

struct Density15 {
    bool valid = false;
    bool capacity_overflow = false;
    std::string outcome = "NOT_RUN";
    std::vector<long double> rho;
    std::vector<long double> constraint;
    std::vector<std::uint32_t> active_ids;
    std::uint32_t maximum_degree = 0U;
    std::uint64_t accepted_pair_count = 0U;
};

struct EnergyGradient15 {
    bool valid = false;
    std::string outcome = "NOT_RUN";
    long double objective = 0.0L;
    long double nonpressure_energy = 0.0L;
    long double inertia_energy = 0.0L;
    long double viscosity_energy = 0.0L;
    long double surface_energy = 0.0L;
    Density15 density;
    std::vector<Vec3l> gradient;
    std::vector<long double> effective_multiplier;
    std::vector<Vec3l> dynamic_pressure_force;
    std::vector<Vec3l> ghost_pressure_force;
    std::vector<Vec3l> viscosity_force;
    std::vector<Vec3l> surface_force;
    std::uint64_t viscosity_pair_count = 0U;
    std::uint64_t surface_pair_count = 0U;
};

struct StepTrace15 {
    std::uint32_t outer_updates = 0U;
    std::uint64_t accepted_inner_iterations = 0U;
    std::vector<std::uint32_t> backtracks;
    std::vector<std::uint64_t> outer_inner_offsets;
    std::vector<long double> accepted_objectives;
    std::vector<std::vector<Vec3l>> evaluation_positions;
    std::vector<std::vector<long double>> evaluation_multipliers;
    std::vector<std::uint8_t> evaluation_modes;
    std::vector<std::uint8_t> evaluation_gradient;
    std::vector<std::string> reached_stages;
};

struct Contact15 {
    std::uint32_t id = 0U;
    std::uint8_t mask = 0U;
    Vec3l impulse;
};

struct StepMetrics15 {
    long double objective = 0.0L;
    long double nonpressure_energy = 0.0L;
    long double inertia_energy = 0.0L;
    long double viscosity_energy = 0.0L;
    long double surface_energy = 0.0L;
    long double maximum_positive_density_strain = 0.0L;
    long double rms_positive_density_strain = 0.0L;
    long double kkt_rms_m = 0.0L;
    long double kkt_maximum_m = 0.0L;
    long double complementarity = 0.0L;
    long double multiplier_fixed_point = 0.0L;
    std::uint64_t active_multiplier_count = 0U;
};

struct Step15 {
    std::string outcome = "NOT_RUN";
    bool apparatus_valid = false;
    bool work_ceiling = false;
    bool accepted = false;
    State15 state;
    std::vector<long double> pi;
    std::vector<long double> rho;
    std::vector<std::uint32_t> active_multiplier_ids;
    std::vector<Contact15> contact;
    std::vector<Vec3l> kkt_gradient;
    std::vector<Vec3l> kkt_direction;
    std::vector<Vec3l> dynamic_pressure_force;
    std::vector<Vec3l> ghost_pressure_force;
    std::vector<Vec3l> viscosity_force;
    std::vector<Vec3l> surface_force;
    StepTrace15 trace;
    StepMetrics15 metrics;
    std::string input_root;
    std::string previous_state_root;
    std::string state_root;
    std::string pressure_root;
    std::string contact_root;
    std::string graph_root;
    std::string work_root;
    std::string result_root;
    Work15 work;
    Work15 expected_work;
};

struct GradientCheck15 {
    bool valid = false;
    std::string outcome = "NOT_RUN";
    std::vector<Vec3l> finite_difference;
    long double relative_l2 = 0.0L;
    long double maximum_absolute_n = 0.0L;
    Work15 work;
    Work15 expected_work;
};

Profile15 frozen_profile15();

Density15 oracle_density15(const State15& accepted,
    const std::vector<Vec3l>& positions, const SolverOptions15& options,
    Work15& work);

Work15 oracle_expected_density_work15(const State15& accepted,
    const std::vector<Vec3l>& positions, const SolverOptions15& options);

EnergyGradient15 oracle_energy_gradient15(const State15& accepted,
    const std::vector<Vec3l>& positions,
    const std::vector<long double>& multipliers,
    const SolverOptions15& options, MultiplierMode15 multiplier_mode,
    bool need_gradient, Work15& work);

Step15 oracle_step15(const State15& accepted,
    const SolverOptions15& options);

GradientCheck15 oracle_central_difference_gradient15(
    const State15& accepted, const std::vector<Vec3l>& positions,
    const std::vector<long double>& multipliers,
    const SolverOptions15& options,
    const std::vector<Vec3l>& analytical_gradient);

long double oracle_surface_pair_root15(long double initial_separation,
    const SolverOptions15& options);

Density15 candidate_density15(const State15& accepted,
    const std::vector<Vec3l>& positions, const SolverOptions15& options,
    Work15& work);

Work15 candidate_expected_density_work15(const State15& accepted,
    const std::vector<Vec3l>& positions, const SolverOptions15& options);

EnergyGradient15 candidate_energy_gradient15(const State15& accepted,
    const std::vector<Vec3l>& positions,
    const std::vector<long double>& multipliers,
    const SolverOptions15& options, MultiplierMode15 multiplier_mode,
    bool need_gradient, Work15& work);

Work15 candidate_expected_energy_gradient_work15(
    const State15& accepted, const std::vector<Vec3l>& positions,
    const std::vector<long double>& multipliers,
    const SolverOptions15& options, bool need_gradient);

Step15 candidate_step15(const State15& accepted,
    const SolverOptions15& options);

std::string_view gravity_mode_name15(GravityMode15 mode);
std::string_view boundary_mode_name15(BoundaryMode15 mode);
std::string_view mutation_name15(Mutation15 mutation);

} // namespace nextengine::nonlocal::ncgp15
