#pragma once

#include "math.hpp"
#include "profiles.hpp"

#include <cstddef>
#include <string>
#include <vector>

namespace nextengine::nonlocal {

struct Particle {
    Vec3 position;
    Vec3 velocity;
    bool fixed = false;
};

struct Fixture {
    std::string name;
    std::vector<Particle> particles;
    double rest_density = 1000.0;
    double spacing = 0.005;
    double mass = 0.000125;
    double horizon = 0.015;
    double time_step = 0.001;
    Vec3 gravity{};
    double kappa = 0.0;
    double lambda = 0.0;
    double mu = 0.0;
    double gamma = 0.0;
    EnabledTerms terms;
    int iterations = 1;
    std::vector<int> lattice_index_by_sample;
    std::size_t pair_capacity = 0;
    double grid_margin = 0.0;
    bool analytic_box_contact = false;
    Vec3 contact_minimum{};
    Vec3 contact_maximum{};
    double particle_radius = 0.0;
    bool advected = false;
    /// NGQ7 revision 2: fixed boundary samples support density and the
    /// incompressibility term only; viscosity and surface terms skip them.
    bool boundary_density_only = false;
    int trace_length = 1;
    /// NGQ8: interior shelf + divider with one rectangular opening, applied
    /// after the outer-box clamp (positional, presentation candidate only).
    struct Spill {
        bool enabled = false;
        double shelf_top = 0.0;
        double wall_x0 = 0.0;
        double wall_x1 = 0.0;
        double opening_y0 = 0.0;
        double opening_y1 = 0.0;
        double opening_z0 = 0.0;
        double opening_z1 = 0.0;
        /// NGQ8 revision 3: `flush` clamps sample centres to the opening
        /// faces themselves instead of one radius inside them.
        bool flush = false;
        /// NGQ8 revision 3: leave the one-cell ring of divider cells around
        /// the opening without fixed density samples.
        bool open_ring = false;
        /// NGQ8 revision 9: under-floor pipe. The divider has no opening;
        /// a vertical shaft from the tank floor centre (`shaft_x0..x1`,
        /// `opening_z0..z1`) feeds a horizontal duct (`opening_y0..y1`,
        /// same z) that runs through the shelf and the divider and leaves
        /// inside a protruding pipe body ending at `pipe_x1`.
        bool under_floor = false;
        double shaft_x0 = 0.0;
        double shaft_x1 = 0.0;
        double pipe_x1 = 0.0;
        double pipe_wall = 0.0;

        double exit_x() const { return under_floor ? pipe_x1 : wall_x1; }
        double exit_centre_y() const { return 0.5 * (opening_y0 + opening_y1); }
    } spill;
};

struct EnergyComponents {
    double inertia = 0.0;
    double incompressibility = 0.0;
    double bulk_viscosity = 0.0;
    double shear_viscosity = 0.0;
    double surface_tension = 0.0;
};

struct OracleResult {
    std::vector<double> density;
    std::vector<Vec3> source;
    std::vector<Mat3> local_matrix;
    std::vector<Vec3> predicted_position;
    std::vector<Vec3> linearization_position;
    std::vector<Vec3> next_position;
    std::vector<Vec3> final_velocity;
    EnergyComponents energy;
    std::size_t directed_pairs = 0;
    std::size_t maximum_degree = 0;
    double normalized_momentum_residual = 0.0;
};

struct OracleCaseReport {
    std::string name;
    bool passed = false;
    std::vector<std::string> failures;
    OracleResult result;
};

struct CpuGatherSelfTestReport {
    bool passed = false;
    std::string json;
};

struct CpuScaleLawSelfTestReport {
    bool passed = false;
    std::string json;
};

std::vector<Fixture> oracle_fixtures();
OracleResult run_cpu_oracle(const Fixture& fixture);
OracleResult run_cpu_gather_oracle(const Fixture& fixture);
std::vector<OracleCaseReport> run_cpu_self_test();
std::string cpu_self_test_json(const std::vector<OracleCaseReport>& reports);
CpuGatherSelfTestReport run_cpu_gather_self_test();
CpuScaleLawSelfTestReport run_cpu_scale_law_self_test();

} // namespace nextengine::nonlocal
