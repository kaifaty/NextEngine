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

std::vector<Fixture> oracle_fixtures();
OracleResult run_cpu_oracle(const Fixture& fixture);
std::vector<OracleCaseReport> run_cpu_self_test();
std::string cpu_self_test_json(const std::vector<OracleCaseReport>& reports);

} // namespace nextengine::nonlocal
