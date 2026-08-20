#pragma once

#include "math.hpp"

#include <cstddef>
#include <string>
#include <vector>

namespace nextengine::nonlocal {

struct EnabledTerms {
    bool incompressibility = false;
    bool bulk_viscosity = false;
    bool shear_viscosity = false;
    bool surface_tension = false;
};

struct Tolerances {
    double density_absolute = 0.1;
    double density_relative = 1.0e-4;
    double normalized_absolute = 2.0e-5;
    double normalized_relative = 5.0e-5;
    double position_absolute = 2.0e-6;
    double velocity_absolute = 2.0e-3;
    double normalized_momentum_residual = 1.0e-5;
};

enum class InitializationOrder {
    Lexicographic,
    AffinePermutation,
};

struct Profile {
    int record_version = 0;
    std::string id;
    int lattice_x = 0;
    int lattice_y = 0;
    int lattice_z = 0;
    std::size_t samples = 0;
    Vec3 origin{};
    double rest_density = 1000.0;
    double spacing = 0.005;
    double mass = 0.000125;
    double horizon = 0.015;
    double time_step = 0.001;
    Vec3 gravity{0.0, -9.81, 0.0};
    double kappa = 0.0;
    double lambda = 0.0;
    double mu = 0.0;
    double gamma = 0.0;
    EnabledTerms terms;
    int fixed_iterations = 1;
    std::size_t max_samples = 0;
    std::size_t max_neighbors = 0;
    std::size_t max_directed_pairs = 0;
    std::string geometry;
    std::string boundary;
    Tolerances tolerances;
    InitializationOrder initialization_order = InitializationOrder::Lexicographic;
    std::size_t permutation_multiplier = 0;
    std::size_t permutation_offset = 0;
    bool advected = false;
    int trace_length = 1;
    double lattice_scale = 1.0;
    double grid_margin = 0.0;
};

const std::vector<Profile>& profiles();
const Profile& find_profile(const std::string& id);
std::string canonical_profile_json(const Profile& profile);
std::string described_profile_json(const Profile& profile);
std::string production_profile_audit_json();

} // namespace nextengine::nonlocal
