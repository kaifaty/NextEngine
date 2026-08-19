#include "profiles.hpp"

#include "sha256.hpp"

#include <iomanip>
#include <sstream>
#include <stdexcept>

namespace nextengine::nonlocal {
namespace {

constexpr const char* PAPER_DOI = "10.1145/3799902.3811196";
constexpr const char* PAPER_PDF_SHA256 =
    "610047ef32e895026c3661c57999f14ae550d8e2719ba2fcd95742371ad031c1";
constexpr const char* PERIDYNO_COMMIT = "1aa892bb296fe766d2f9249c881b8605af23a69b";

Profile performance_profile(
    std::string id,
    int x,
    int y,
    int z,
    double kappa,
    double lambda,
    double mu,
    double gamma,
    EnabledTerms terms,
    int iterations) {
    Profile profile;
    profile.id = std::move(id);
    profile.lattice_x = x;
    profile.lattice_y = y;
    profile.lattice_z = z;
    profile.samples = static_cast<std::size_t>(x) * static_cast<std::size_t>(y)
        * static_cast<std::size_t>(z);
    profile.kappa = kappa;
    profile.lambda = lambda;
    profile.mu = mu;
    profile.gamma = gamma;
    profile.terms = terms;
    profile.fixed_iterations = iterations;
    profile.max_samples = profile.samples;
    profile.max_neighbors = 123;
    profile.max_directed_pairs = profile.samples * profile.max_neighbors;
    profile.geometry = "free_surface_rectangular_lattice_lexicographic_z_y_x";
    profile.boundary = "none";
    return profile;
}

std::string bool_json(bool value) { return value ? "true" : "false"; }

} // namespace

const std::vector<Profile>& profiles() {
    static const std::vector<Profile> values = {
        [] {
            Profile profile;
            profile.id = "nuv-tiny-oracle.v0";
            profile.samples = 256;
            profile.kappa = 1.0;
            profile.lambda = 1.5;
            profile.mu = 1.0;
            profile.gamma = 1000.0;
            profile.terms = {true, true, true, true};
            profile.fixed_iterations = 4;
            profile.max_samples = 256;
            profile.max_neighbors = 256;
            profile.max_directed_pairs = 65536;
            profile.geometry = "named_deterministic_fixtures_1_through_256_samples";
            profile.boundary = "none_or_fixed_ghost_shell_per_named_fixture";
            return profile;
        }(),
        performance_profile(
            "nuv-water-16k.v0", 40, 20, 20, 1.0, 1.5, 0.0, 0.0,
            {true, true, false, false}, 5),
        performance_profile(
            "nuv-water-48k.v0", 80, 40, 15, 1.0, 1.5, 0.0, 0.0,
            {true, true, false, false}, 5),
        performance_profile(
            "nuv-viscous-16k.v0", 40, 20, 20, 1.0, 200.0, 1.0, 0.0,
            {true, true, true, false}, 20),
        performance_profile(
            "nuv-surface-16k.v0", 40, 20, 20, 1.0, 0.2, 0.0, 1000.0,
            {true, false, false, true}, 20),
    };
    return values;
}

const Profile& find_profile(const std::string& id) {
    for (const Profile& profile : profiles()) {
        if (profile.id == id) {
            return profile;
        }
    }
    throw std::invalid_argument("unknown profile: " + id);
}

std::string canonical_profile_json(const Profile& profile) {
    std::ostringstream output;
    output << std::setprecision(17);
    output << "{\"profile_id\":\"" << profile.id << "\",\"record_version\":0"
           << ",\"lattice\":[" << profile.lattice_x << ',' << profile.lattice_y << ','
           << profile.lattice_z << ']'
           << ",\"samples\":" << profile.samples
           << ",\"initialization_order\":\"lexicographic_z_y_x\""
           << ",\"geometry\":\"" << profile.geometry << "\""
           << ",\"spacing_m\":" << profile.spacing
           << ",\"mass_kg\":" << profile.mass
           << ",\"horizon_m\":" << profile.horizon
           << ",\"rest_density_kg_m3\":" << profile.rest_density
           << ",\"time_step_s\":" << profile.time_step
           << ",\"gravity_m_s2\":[" << profile.gravity.x << ',' << profile.gravity.y << ','
           << profile.gravity.z << ']'
           << ",\"boundary\":\"" << profile.boundary << "\""
           << ",\"coefficients\":{\"kappa\":" << profile.kappa
           << ",\"lambda\":" << profile.lambda << ",\"mu\":" << profile.mu
           << ",\"gamma\":" << profile.gamma << '}'
           << ",\"enabled_terms\":{\"incompressibility\":"
           << bool_json(profile.terms.incompressibility)
           << ",\"bulk_viscosity\":" << bool_json(profile.terms.bulk_viscosity)
           << ",\"shear_viscosity\":" << bool_json(profile.terms.shear_viscosity)
           << ",\"surface_tension\":" << bool_json(profile.terms.surface_tension) << '}'
           << ",\"fixed_iterations\":" << profile.fixed_iterations
           << ",\"capacity\":{\"max_samples\":" << profile.max_samples
           << ",\"max_neighbors\":" << profile.max_neighbors
           << ",\"max_directed_pairs\":" << profile.max_directed_pairs << '}'
           << ",\"numeric_modes\":{\"cpu_oracle\":\"ieee754_binary64\""
           << ",\"cuda_baseline\":\"ieee754_binary32\"}"
           << ",\"tolerances\":{\"density_absolute\":"
           << profile.tolerances.density_absolute << ",\"density_relative\":"
           << profile.tolerances.density_relative << ",\"normalized_absolute\":"
           << profile.tolerances.normalized_absolute << ",\"normalized_relative\":"
           << profile.tolerances.normalized_relative << ",\"position_absolute_m\":"
           << profile.tolerances.position_absolute << ",\"velocity_absolute_m_s\":"
           << profile.tolerances.velocity_absolute << ",\"normalized_momentum_residual\":"
           << profile.tolerances.normalized_momentum_residual << '}'
           << ",\"paper\":{\"doi\":\"" << PAPER_DOI << "\",\"pdf_sha256\":\""
           << PAPER_PDF_SHA256 << "\"}"
           << ",\"reference\":{\"repository\":\"PeriDyno\",\"commit\":\""
           << PERIDYNO_COMMIT << "\"}"
           << ",\"research_choices\":["
           << "\"performance blocks are boundary-free\","
           << "\"tiny closed-box boundary is a fixed ghost shell\","
           << "\"neighbor membership is frozen from the initial position\","
           << "\"surface pairwise function is bidirectional\"]}";
    return output.str();
}

std::string described_profile_json(const Profile& profile) {
    const std::string canonical = canonical_profile_json(profile);
    std::ostringstream output;
    output << "{\"schema\":\"nextengine.nonlocal.profile.v0\",\"profile_sha256\":\""
           << sha256_hex(canonical) << "\",\"profile\":" << canonical << '}';
    return output.str();
}

} // namespace nextengine::nonlocal
