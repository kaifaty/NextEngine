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

Profile performance_profile_v1(
    std::string id,
    int x,
    int y,
    int z,
    double kappa,
    double lambda,
    double mu,
    double gamma,
    EnabledTerms terms,
    int iterations,
    bool advected = false) {
    Profile profile = performance_profile(
        std::move(id), x, y, z, kappa, lambda, mu, gamma, terms, iterations);
    profile.record_version = 1;
    profile.advected = advected;
    profile.trace_length = advected ? 32 : 1;
    profile.lattice_scale = advected ? 0.99 : 1.0;
    profile.grid_margin = advected ? 0.025 : 0.0;
    if (advected) {
        profile.max_neighbors = 192;
        profile.max_directed_pairs = profile.samples * profile.max_neighbors;
        profile.geometry = "free_surface_scaled_rectangular_lattice_dynamic_seed_v1";
    }
    return profile;
}

Profile product_bridge_profile(
    std::string id,
    double horizon,
    double time_step,
    std::size_t max_neighbors,
    std::string geometry) {
    Profile profile = performance_profile_v1(
        std::move(id), 80, 15, 40, 1.0, 1.5, 0.0, 0.0,
        {true, true, false, false}, 5);
    profile.record_version = 2;
    profile.origin = {0.025, 0.025, 0.025};
    profile.spacing = 0.05;
    profile.mass = 0.125;
    profile.horizon = horizon;
    profile.time_step = time_step;
    profile.max_neighbors = max_neighbors;
    profile.max_directed_pairs = profile.samples * max_neighbors;
    profile.geometry = std::move(geometry);
    profile.boundary = "none_profile_bridge_only";
    return profile;
}

Profile product_static_support_profile(
    std::string id,
    double kappa,
    double lambda,
    std::string coefficient_identity) {
    Profile profile = performance_profile_v1(
        std::move(id), 80, 15, 40, kappa, lambda, 0.0, 0.0,
        {true, true, false, false}, 5);
    profile.record_version = 3;
    profile.origin = {-1.975, 0.025, -0.975};
    profile.spacing = 0.05;
    profile.mass = 0.125;
    profile.horizon = 0.10;
    profile.time_step = 1.0 / 240.0;
    profile.max_samples = 50000;
    profile.static_boundary_samples = 24704;
    profile.max_static_boundary_samples = 32768;
    profile.static_boundary_layers = 2;
    profile.basin_min = {-2.0, 0.0, -1.0};
    profile.basin_max = {2.0, 1.0, 1.0};
    profile.particle_radius = 0.025;
    profile.max_neighbors = 33;
    profile.max_directed_pairs =
        (profile.samples + profile.static_boundary_samples) * profile.max_neighbors;
    profile.geometry = "spec38_sealed_extent_two_layer_outer_lattice_"
        + std::move(coefficient_identity);
    profile.boundary = "two_layer_fixed_ghost_density_support_no_contact";
    profile.contact = "analytical_swept_sphere_external_not_in_gpu_preflight";
    return profile;
}

Profile product_h3_support_profile() {
    Profile profile = product_static_support_profile(
        "nuv-basin-48k-static-support-h3-physical.v4", 9196.875, 360.0,
        "hydro_head_h3_support_remediation");
    profile.record_version = 4;
    profile.horizon = 0.15;
    profile.fixed_iterations = 16;
    profile.static_boundary_samples = 38856;
    profile.max_static_boundary_samples = 38856;
    profile.static_boundary_layers = 3;
    profile.max_neighbors = 123;
    profile.max_directed_pairs =
        (profile.samples + profile.static_boundary_samples) * profile.max_neighbors;
    profile.geometry =
        "spec38_sealed_extent_three_layer_outer_lattice_h3_physical_candidate";
    profile.boundary = "three_layer_fixed_ghost_density_support_no_contact";
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
        performance_profile_v1(
            "nuv-water-50k-coherent.v1", 100, 25, 20, 1.0, 1.5, 0.0, 0.0,
            {true, true, false, false}, 5),
        [] {
            Profile profile = performance_profile_v1(
                "nuv-water-50k-permuted.v1", 100, 25, 20, 1.0, 1.5, 0.0, 0.0,
                {true, true, false, false}, 5);
            profile.initialization_order = InitializationOrder::AffinePermutation;
            profile.permutation_multiplier = 32749;
            profile.permutation_offset = 7919;
            profile.geometry = "free_surface_rectangular_lattice_affine_permuted_stable_ids";
            return profile;
        }(),
        performance_profile_v1(
            "nuv-water-50k-advected.v1", 100, 25, 20, 1.0, 1.5, 0.0, 0.0,
            {true, true, false, false}, 5, true),
        performance_profile_v1(
            "nuv-viscous-16k-advected.v1", 40, 20, 20, 1.0, 200.0, 1.0, 0.0,
            {true, true, true, false}, 20, true),
        performance_profile_v1(
            "nuv-surface-16k-advected.v1", 40, 20, 20, 1.0, 0.2, 0.0, 100.0,
            {true, false, false, true}, 20, true),
        performance_profile_v1(
            "nuv-surface-stiff-16k-i2.v1", 40, 20, 20, 1.0, 0.2, 0.0, 1000.0,
            {true, false, false, true}, 2),
        performance_profile_v1(
            "nuv-water-100k-report.v1", 100, 50, 20, 1.0, 1.5, 0.0, 0.0,
            {true, true, false, false}, 5),
        product_bridge_profile(
            "nuv-basin-48k-source-scale.v2", 0.15, 0.001, 123,
            "product_axis_basin_lattice_source_horizon_and_cadence"),
        product_bridge_profile(
            "nuv-basin-48k-cadence.v2", 0.15, 1.0 / 240.0, 123,
            "product_axis_basin_lattice_source_horizon_product_cadence"),
        product_bridge_profile(
            "nuv-basin-48k-spec-support.v2", 0.10, 1.0 / 240.0, 33,
            "product_axis_basin_lattice_spec_support_and_cadence"),
        product_static_support_profile(
            "nuv-basin-48k-static-support-control.v3", 1.0, 1.5,
            "source_coefficient_control"),
        product_static_support_profile(
            "nuv-basin-48k-static-support-derived.v3", 576.0, 360.0,
            "dimensionally_derived_coefficient_hypothesis"),
        product_h3_support_profile(),
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
    output << "{\"profile_id\":\"" << profile.id << "\",\"record_version\":"
           << profile.record_version
           << ",\"lattice\":[" << profile.lattice_x << ',' << profile.lattice_y << ','
           << profile.lattice_z << ']'
           << ",\"samples\":" << profile.samples
           << ",\"initialization_order\":\""
           << (profile.initialization_order == InitializationOrder::Lexicographic
                   ? "lexicographic_z_y_x" : "affine_permuted_stable_id")
           << "\"";
    if (profile.record_version >= 1) {
        output << ",\"generator\":{\"lattice_scale\":" << profile.lattice_scale
               << ",\"advected\":" << bool_json(profile.advected)
               << ",\"trace_length\":" << profile.trace_length
               << ",\"grid_margin_m\":" << profile.grid_margin;
        if (profile.initialization_order == InitializationOrder::AffinePermutation) {
            output << ",\"permutation\":{\"kind\":\"affine_mod_n\",\"multiplier\":"
                   << profile.permutation_multiplier << ",\"offset\":"
                   << profile.permutation_offset << '}';
        }
        output << '}';
    }
    output
           << ",\"geometry\":\"" << profile.geometry << "\""
           << ",\"origin_m\":[";
    if (profile.record_version >= 2) {
        output << profile.origin.x << ',' << profile.origin.y << ',' << profile.origin.z;
    } else {
        output << "0,0,0";
    }
    output << ']'
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
           << ",\"max_neighbors\":" << profile.max_neighbors;
    if (profile.record_version >= 3) {
        output << ",\"static_boundary_samples\":" << profile.static_boundary_samples
               << ",\"max_static_boundary_samples\":"
               << profile.max_static_boundary_samples
               << ",\"total_solver_samples\":"
               << profile.samples + profile.static_boundary_samples;
    }
    output
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
           << (profile.record_version == 0
                   ? "\"neighbor membership is frozen from the initial position\","
                   : "\"neighbor membership is rebuilt from each substep reference and frozen within its nonlinear solve\",")
           << "\"surface pairwise function is bidirectional\"]";
    if (profile.record_version >= 3) {
        output << ",\"static_boundary\":{\"layers\":"
               << profile.static_boundary_layers << ",\"basin_min_m\":["
               << profile.basin_min.x << ',' << profile.basin_min.y << ','
               << profile.basin_min.z << "],\"basin_max_m\":["
               << profile.basin_max.x << ',' << profile.basin_max.y << ','
               << profile.basin_max.z << "],\"particle_radius_m\":"
               << profile.particle_radius << ",\"contact\":\"" << profile.contact
               << "\"}";
    }
    output << '}';
    return output.str();
}

std::string described_profile_json(const Profile& profile) {
    const std::string canonical = canonical_profile_json(profile);
    std::ostringstream output;
    output << "{\"schema\":\"nextengine.nonlocal.profile.v" << profile.record_version
           << "\",\"profile_sha256\":\""
           << sha256_hex(canonical) << "\",\"profile\":" << canonical << '}';
    return output.str();
}

std::string production_profile_audit_json() {
    const std::string retained_48k_id = "nuv-water-48k.v0";
    const std::string retained_50k_id = "nuv-water-50k-coherent.v1";
    const std::string source_scale_id = "nuv-basin-48k-source-scale.v2";
    const std::string cadence_id = "nuv-basin-48k-cadence.v2";
    const std::string spec_support_id = "nuv-basin-48k-spec-support.v2";
    const std::string static_control_id = "nuv-basin-48k-static-support-control.v3";
    const std::string static_derived_id = "nuv-basin-48k-static-support-derived.v3";
    const std::string h3_candidate_id =
        "nuv-basin-48k-static-support-h3-physical.v4";
    const Profile& retained_48k = find_profile(retained_48k_id);
    const Profile& retained_50k = find_profile(retained_50k_id);
    const Profile& source_scale = find_profile(source_scale_id);
    const Profile& cadence = find_profile(cadence_id);
    const Profile& spec_support = find_profile(spec_support_id);
    const Profile& static_control = find_profile(static_control_id);
    const Profile& static_derived = find_profile(static_derived_id);
    const Profile& h3_candidate = find_profile(h3_candidate_id);

    const auto append_profile = [](std::ostringstream& output, const Profile& profile) {
        const std::string canonical = canonical_profile_json(profile);
        output << "{\"profile_id\":\"" << profile.id << "\",\"profile_sha256\":\""
               << sha256_hex(canonical) << "\",\"samples\":" << profile.samples
               << ",\"lattice\":[" << profile.lattice_x << ',' << profile.lattice_y << ','
               << profile.lattice_z << "],\"origin_m\":[" << profile.origin.x << ','
               << profile.origin.y << ',' << profile.origin.z << "],\"spacing_m\":"
               << profile.spacing << ",\"mass_kg\":" << profile.mass
               << ",\"horizon_m\":" << profile.horizon << ",\"horizon_over_spacing\":"
               << profile.horizon / profile.spacing << ",\"time_step_s\":"
               << profile.time_step << ",\"maximum_neighbors\":"
               << profile.max_neighbors << ",\"static_boundary_samples\":"
               << profile.static_boundary_samples << ",\"total_solver_samples\":"
               << profile.samples + profile.static_boundary_samples
               << ",\"boundary\":\"" << profile.boundary << "\",\"contact\":\""
               << profile.contact << "\"}";
    };

    std::ostringstream output;
    output << std::setprecision(17);
    output << "{\"schema\":\"nextengine.nonlocal.production-profile-audit.v2\""
           << ",\"command_status\":\"PASS\""
           << ",\"semantic_status\":\"H3_SUPPORT_REMEDIATION_CANDIDATE\""
           << ",\"retained_profiles\":[";
    append_profile(output, retained_48k);
    output << ',';
    append_profile(output, retained_50k);
    output << "],\"bridge_profiles\":[";
    append_profile(output, source_scale);
    output << ',';
    append_profile(output, cadence);
    output << ',';
    append_profile(output, spec_support);
    output << ',';
    append_profile(output, static_control);
    output << ',';
    append_profile(output, static_derived);
    output << ',';
    append_profile(output, h3_candidate);
    output << "]"
           << ",\"product_expectation\":{\"samples_nominal\":48000"
           << ",\"samples_hard_capacity\":50000,\"lattice\":[80,15,40]"
           << ",\"spacing_m\":0.050000000000000003,\"mass_kg\":0.125"
           << ",\"horizon_m\":0.10000000000000001,\"time_step_s\":"
           << 1.0 / 240.0
           << ",\"static_boundary_samples\":24704"
           << ",\"static_boundary_capacity\":32768"
           << ",\"boundary\":\"two_layer_support_plus_swept_contact\"}"
           << ",\"remediation_candidate\":{\"profile_id\":\""
           << h3_candidate.id << "\",\"horizon_m\":" << h3_candidate.horizon
           << ",\"horizon_over_spacing\":"
           << h3_candidate.horizon / h3_candidate.spacing
           << ",\"static_boundary_samples\":"
           << h3_candidate.static_boundary_samples
           << ",\"static_boundary_capacity\":"
           << h3_candidate.max_static_boundary_samples
           << ",\"total_solver_samples\":"
           << h3_candidate.samples + h3_candidate.static_boundary_samples
           << ",\"fixed_iterations\":" << h3_candidate.fixed_iterations
           << ",\"runtime_authority\":false,\"npr1_authorized\":false}"
           << ",\"mismatch\":{\"spacing_scale_from_retained_48k\":"
           << source_scale.spacing / retained_48k.spacing
           << ",\"mass_scale_from_retained_48k\":"
           << source_scale.mass / retained_48k.mass
           << ",\"time_step_scale_from_source\":"
           << cadence.time_step / source_scale.time_step
           << ",\"source_horizon_over_spacing\":"
           << source_scale.horizon / source_scale.spacing
           << ",\"product_horizon_over_spacing\":"
           << spec_support.horizon / spec_support.spacing
           << ",\"retained_48k_axis_order_matches_product\":false"
           << ",\"retained_50k_sample_count_matches_nominal\":false"
           << ",\"retained_boundary_matches_product\":false}"
           << ",\"open_gates\":{\"coefficient_scale_law_derived\":true"
           << ",\"coefficient_scale_law_physically_selected\":false"
           << ",\"static_density_support_selected\":true"
           << ",\"sealed_contact_implemented\":false"
           << ",\"canonical_publication_selected\":false"
           << ",\"physical_corpus_passed\":false"
           << ",\"authority_selected\":false}}";
    return output.str();
}

} // namespace nextengine::nonlocal
