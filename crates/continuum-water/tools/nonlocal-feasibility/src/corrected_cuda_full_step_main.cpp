#include "corrected_cuda_full_step.hpp"
#include "corrected_cuda_assembly.hpp"

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <iomanip>
#include <iostream>
#include <limits>
#include <stdexcept>
#include <string>
#include <vector>

namespace {

using namespace nextengine::nonlocal::gpu_full_step;

bool same_graph(const NonlocalGpuGraphResult& lhs,
    const NonlocalGpuGraphResult& rhs) {
    return lhs.failure == rhs.failure && lhs.owner_ids == rhs.owner_ids
        && lhs.offsets == rhs.offsets && lhs.neighbor_ids == rhs.neighbor_ids;
}

std::vector<NonlocalGpuSample> support_edge_fixture() {
    return {
        {90U, {0.0, 0.0, 0.0}, {0.0, 0.0, 0.0}, {}},
        {4U, {0.15, 0.0, 0.0}, {0.15, 0.0, 0.0}, {}},
        {55U, {0.150001, 0.0, 0.0}, {0.150001, 0.0, 0.0}, {}},
        {801U, {0.0, 0.150001, 0.0}, {0.0, 0.150001, 0.0}, {}},
    };
}

std::vector<NonlocalGpuSample> combined_cluster() {
    std::vector<NonlocalGpuSample> samples;
    samples.reserve(100U);
    for (std::uint32_t logical = 0U; logical < 100U; ++logical) {
        const std::uint32_t ix = logical % 5U;
        const std::uint32_t iy = (logical / 5U) % 5U;
        const std::uint32_t iz = logical / 25U;
        const Vec3d reference{(static_cast<double>(ix) - 2.0) * 0.001,
            (static_cast<double>(iy) - 2.0) * 0.001,
            (static_cast<double>(iz) - 1.5) * 0.001};
        const Vec3d displacement{
            static_cast<double>(static_cast<int>(logical % 7U) - 3) * 2.0e-6,
            static_cast<double>(static_cast<int>((logical * 3U) % 11U) - 5)
                * 1.0e-6,
            static_cast<double>(static_cast<int>((logical * 5U) % 13U) - 6)
                * 1.0e-6};
        const Vec3d current{reference.x + displacement.x,
            reference.y + displacement.y, reference.z + displacement.z};
        samples.push_back({1000U + 17U * logical, reference, current, {}});
    }
    return samples;
}

std::vector<Vec3d> smooth_direction(std::size_t count) {
    std::vector<Vec3d> result;
    result.reserve(count);
    double squared = 0.0;
    for (std::size_t index = 0U; index < count; ++index) {
        Vec3d value{std::sin(0.17 * static_cast<double>(index + 1U)),
            std::cos(0.11 * static_cast<double>(index + 3U)),
            std::sin(0.07 * static_cast<double>(index + 5U) + 0.3)};
        squared += value.x * value.x + value.y * value.y + value.z * value.z;
        result.push_back(value);
    }
    const double inverse = 1.0 / std::sqrt(squared);
    for (Vec3d& value : result) {
        value.x *= inverse;
        value.y *= inverse;
        value.z *= inverse;
    }
    return result;
}

double relative_vector_error(
    const std::vector<Vec3d>& candidate, const std::vector<Vec3d>& reference) {
    if (candidate.size() != reference.size()) return std::numeric_limits<double>::infinity();
    double difference = 0.0;
    double scale = 0.0;
    for (std::size_t index = 0U; index < candidate.size(); ++index) {
        const Vec3d delta{candidate[index].x - reference[index].x,
            candidate[index].y - reference[index].y,
            candidate[index].z - reference[index].z};
        difference += delta.x * delta.x + delta.y * delta.y + delta.z * delta.z;
        scale += reference[index].x * reference[index].x
            + reference[index].y * reference[index].y
            + reference[index].z * reference[index].z;
    }
    return std::sqrt(difference) / std::max(std::sqrt(scale), 1.0);
}

double cosine_loss(
    const std::vector<Vec3d>& candidate, const std::vector<Vec3d>& reference) {
    if (candidate.size() != reference.size()) return 2.0;
    double product = 0.0;
    double candidate_norm = 0.0;
    double reference_norm = 0.0;
    for (std::size_t index = 0U; index < candidate.size(); ++index) {
        product += candidate[index].x * reference[index].x
            + candidate[index].y * reference[index].y
            + candidate[index].z * reference[index].z;
        candidate_norm += candidate[index].x * candidate[index].x
            + candidate[index].y * candidate[index].y
            + candidate[index].z * candidate[index].z;
        reference_norm += reference[index].x * reference[index].x
            + reference[index].y * reference[index].y
            + reference[index].z * reference[index].z;
    }
    if (!(candidate_norm > 0.0) || !(reference_norm > 0.0)) return 2.0;
    return 1.0 - product / std::sqrt(candidate_norm * reference_norm);
}

bool negative_rejected(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& samples,
    NonlocalGpuVariant variant,
    bool compare_hvp) {
    const std::vector<NonlocalGpuGhost> no_ghosts;
    const auto direction = smooth_direction(samples.size());
    const auto reference = evaluate_reference(profile, samples, no_ghosts,
        compare_hvp ? &direction : nullptr, NonlocalGpuVariant::Corrected);
    NonlocalGpuWorkspace workspace(profile);
    if (workspace.upload(samples, no_ghosts) != NonlocalGpuFailure::None) return false;
    const auto candidate = workspace.evaluate(compare_hvp ? &direction : nullptr,
        variant, true, false);
    if (candidate.failure != NonlocalGpuFailure::None
        || reference.failure != NonlocalGpuFailure::None) return false;
    const double gradient_error = relative_vector_error(
        candidate.gradient, reference.gradient);
    const double hvp_error = compare_hvp
        ? relative_vector_error(candidate.hvp, reference.hvp) : 0.0;
    const double hvp_cosine = compare_hvp
        ? cosine_loss(candidate.hvp, reference.hvp) : 0.0;
    return gradient_error > 1.0e-3 || hvp_error > 1.0e-3
        || hvp_cosine > 1.0e-6;
}

nextengine::nonlocal::gpu_assembly_audit::AssemblyFixture dense_fixture(
    const std::vector<NonlocalGpuSample>& samples) {
    using namespace nextengine::nonlocal::gpu_assembly_audit;
    AssemblyFixture fixture;
    fixture.name = "ncgp1_independent_combined_cluster";
    fixture.terms = {true, true, true, true};
    fixture.samples.reserve(samples.size());
    const auto micrometres = [](double value) {
        return static_cast<std::int64_t>(std::llround(value * 1.0e6));
    };
    for (const auto& sample : samples) {
        AssemblySample converted;
        converted.sample_id = sample.sample_id;
        converted.reference_um = {micrometres(sample.reference.x),
            micrometres(sample.reference.y), micrometres(sample.reference.z)};
        converted.predicted_um = converted.reference_um;
        converted.current_um = {micrometres(sample.current.x),
            micrometres(sample.current.y), micrometres(sample.current.z)};
        fixture.samples.push_back(converted);
    }
    return fixture;
}

int run_operator_self_test() {
    NonlocalGpuProfile profile = nonlocal_water_profile();
    profile.gravity = {0.0, 0.0, 0.0};
    const auto samples = combined_cluster();
    const auto direction = smooth_direction(samples.size());
    const std::vector<NonlocalGpuGhost> no_ghosts;
    const auto reference = evaluate_reference(profile, samples, no_ghosts,
        &direction, NonlocalGpuVariant::Corrected);
    using namespace nextengine::nonlocal::gpu_assembly_audit;
    AssemblyProfile dense_profile;
    const AssemblyResult dense = evaluate_reference_assembly(
        dense_profile, dense_fixture(samples));
    if (dense.failure != AssemblyFailure::None
        || dense.gradient.size() != 3U * samples.size()
        || dense.hessian.size() != 9U * samples.size() * samples.size()) return 24;
    std::vector<Vec3d> dense_gradient(samples.size());
    std::vector<Vec3d> dense_hvp(samples.size());
    for (std::size_t row = 0U; row < samples.size(); ++row) {
        dense_gradient[row] = {dense.gradient[3U * row],
            dense.gradient[3U * row + 1U], dense.gradient[3U * row + 2U]};
        for (std::size_t axis = 0U; axis < 3U; ++axis) {
            double value = 0.0;
            const std::size_t scalar_row = 3U * row + axis;
            for (std::size_t column = 0U; column < samples.size(); ++column) {
                value += dense.hessian[scalar_row * 3U * samples.size()
                        + 3U * column]
                        * direction[column].x
                    + dense.hessian[scalar_row * 3U * samples.size()
                        + 3U * column + 1U]
                        * direction[column].y
                    + dense.hessian[scalar_row * 3U * samples.size()
                        + 3U * column + 2U]
                        * direction[column].z;
            }
            if (axis == 0U) dense_hvp[row].x = value;
            if (axis == 1U) dense_hvp[row].y = value;
            if (axis == 2U) dense_hvp[row].z = value;
        }
    }
    const double oracle_gradient_error = relative_vector_error(
        reference.gradient, dense_gradient);
    const double oracle_hvp_error = relative_vector_error(reference.hvp, dense_hvp);
    const double dense_energy = dense.energy[0] + dense.energy[1]
        + dense.energy[2] + dense.energy[3];
    const double oracle_energy_error = std::abs(reference.energy - dense_energy)
        / std::max(std::abs(dense_energy), 1.0);
    constexpr double derivative_step = 5.0e-5;
    auto plus = samples;
    auto minus = samples;
    for (std::size_t index = 0U; index < samples.size(); ++index) {
        plus[index].current.x += derivative_step * direction[index].x;
        plus[index].current.y += derivative_step * direction[index].y;
        plus[index].current.z += derivative_step * direction[index].z;
        minus[index].current.x -= derivative_step * direction[index].x;
        minus[index].current.y -= derivative_step * direction[index].y;
        minus[index].current.z -= derivative_step * direction[index].z;
    }
    const auto plus_energy = evaluate_reference(profile, plus, no_ghosts,
        nullptr, NonlocalGpuVariant::Corrected);
    const auto minus_energy = evaluate_reference(profile, minus, no_ghosts,
        nullptr, NonlocalGpuVariant::Corrected);
    double gradient_direction = 0.0;
    double hessian_direction = 0.0;
    for (std::size_t index = 0U; index < direction.size(); ++index) {
        gradient_direction += reference.gradient[index].x * direction[index].x
            + reference.gradient[index].y * direction[index].y
            + reference.gradient[index].z * direction[index].z;
        hessian_direction += reference.hvp[index].x * direction[index].x
            + reference.hvp[index].y * direction[index].y
            + reference.hvp[index].z * direction[index].z;
    }
    const double finite_first = (plus_energy.energy - minus_energy.energy)
        / (2.0 * derivative_step);
    const double finite_second = (plus_energy.energy - 2.0 * reference.energy
        + minus_energy.energy) / (derivative_step * derivative_step);
    const double first_derivative_error = std::abs(finite_first - gradient_direction)
        / std::max(std::abs(gradient_direction), 1.0);
    const double second_derivative_error = std::abs(finite_second - hessian_direction)
        / std::max(std::abs(hessian_direction), 1.0);
    NonlocalGpuWorkspace workspace(profile);
    if (workspace.upload(samples, no_ghosts) != NonlocalGpuFailure::None) return 20;
    const auto candidate = workspace.evaluate(
        &direction, NonlocalGpuVariant::Corrected, true, false);
    if (reference.failure != NonlocalGpuFailure::None
        || candidate.failure != NonlocalGpuFailure::None) return 21;
    const double energy_error = std::abs(candidate.energy - reference.energy)
        / std::max(std::abs(reference.energy), 1.0);
    const double gradient_error = relative_vector_error(
        candidate.gradient, reference.gradient);
    const double hvp_error = relative_vector_error(candidate.hvp, reference.hvp);
    const double hvp_cosine = cosine_loss(candidate.hvp, reference.hvp);
    if (candidate.active_pressure_centers != reference.active_pressure_centers
        || oracle_energy_error > 1.0e-12
        || oracle_gradient_error > 1.0e-10 || oracle_hvp_error > 1.0e-10
        || first_derivative_error > 2.0e-7
        || second_derivative_error > 2.0e-5
        || energy_error > 1.0e-4 || gradient_error > 1.0e-3
        || hvp_error > 1.0e-3 || hvp_cosine > 1.0e-6) {
        std::cerr << std::setprecision(17)
                  << "operator gate: energy=" << energy_error
                  << " gradient=" << gradient_error << " hvp=" << hvp_error
                  << " cosine=" << hvp_cosine
                  << " d1=" << first_derivative_error
                  << " d2=" << second_derivative_error
                  << " oracle_energy=" << oracle_energy_error
                  << " oracle_gradient=" << oracle_gradient_error
                  << " oracle_hvp=" << oracle_hvp_error << '\n';
        return 22;
    }

    auto viscous = std::vector<NonlocalGpuSample>{
        {1U, {-0.025, 0.0, 0.0}, {-0.024, 0.001, 0.0},
            {0.24, 0.24, 0.0}},
        {2U, {0.025, 0.0, 0.0}, {0.024, -0.001, 0.0},
            {-0.24, -0.24, 0.0}}};
    auto surface = std::vector<NonlocalGpuSample>{
        {1U, {-0.02, 0.0, 0.0}, {-0.02, 0.0, 0.0}, {}},
        {2U, {0.02, 0.0, 0.0}, {0.02, 0.0, 0.0}, {}}};
    auto crossing = std::vector<NonlocalGpuSample>{
        {1U, {-0.07, 0.0, 0.0}, {-0.08, 0.0, 0.0}, {-2.4, 0.0, 0.0}},
        {2U, {0.07, 0.0, 0.0}, {0.08, 0.0, 0.0}, {2.4, 0.0, 0.0}}};
    const bool missing_chain = negative_rejected(profile, samples,
        NonlocalGpuVariant::MissingKernelChain, true);
    const bool owner_pressure = negative_rejected(profile, samples,
        NonlocalGpuVariant::OwnerOnlyPressure, true);
    const bool sign_flip = negative_rejected(profile, samples,
        NonlocalGpuVariant::HvpSignFlip, true);
    const bool half_viscosity = negative_rejected(profile, viscous,
        NonlocalGpuVariant::HalfViscosity, false);
    const bool surface_sign = negative_rejected(profile, surface,
        NonlocalGpuVariant::WrongSurfaceSign, true);
    const bool graph_swap = negative_rejected(profile, crossing,
        NonlocalGpuVariant::CurrentReferenceSwap, false);
    if (!missing_chain || !owner_pressure || !sign_flip || !half_viscosity
        || !surface_sign || !graph_swap) return 23;

    std::cout << std::setprecision(12)
              << "{\"research_id\":\"NCGP1\",\"mode\":\"operator-self-test\""
              << ",\"status\":\"PASS\",\"energy_relative_error\":"
              << energy_error << ",\"gradient_relative_l2\":" << gradient_error
              << ",\"hvp_relative_l2\":" << hvp_error
              << ",\"hvp_cosine_loss\":" << hvp_cosine
              << ",\"first_derivative_error\":" << first_derivative_error
              << ",\"second_derivative_error\":" << second_derivative_error
              << ",\"oracle_energy_error\":" << oracle_energy_error
              << ",\"oracle_gradient_relative_l2\":" << oracle_gradient_error
              << ",\"oracle_hvp_relative_l2\":" << oracle_hvp_error
              << ",\"active_centers\":" << candidate.active_pressure_centers
              << ",\"work_root\":\"" << work_semantic_root(candidate.work)
              << "\",\"controls\":{\"missing_chain\":true"
              << ",\"owner_pressure\":true,\"hvp_sign\":true"
              << ",\"half_viscosity\":true,\"surface_sign\":true"
              << ",\"graph_swap\":true}}\n";
    return 0;
}

int run_graph_self_test() {
    const NonlocalGpuProfile profile = nonlocal_water_profile();
    const std::vector<NonlocalGpuGhost> no_ghosts;
    const auto edge = support_edge_fixture();
    const auto edge_reference = build_reference_graph(profile, edge, no_ghosts, false);
    NonlocalGpuWorkspace edge_workspace(profile);
    if (edge_workspace.upload(edge, no_ghosts) != NonlocalGpuFailure::None) return 2;
    const auto edge_gpu = edge_workspace.build_current_graph(
        NonlocalGpuVariant::Corrected, true, false);
    const auto edge_strict = edge_workspace.build_current_graph(
        NonlocalGpuVariant::StrictRadius, true, false);
    if (!same_graph(edge_reference, edge_gpu)
        || same_graph(edge_reference, edge_strict)) return 3;

    const auto lattice = make_lattice_state(profile, 4U, 4U, 4U, false, false);
    const auto permuted = make_lattice_state(profile, 4U, 4U, 4U, true, false);
    NonlocalGpuWorkspace canonical_workspace(profile);
    NonlocalGpuWorkspace permuted_workspace(profile);
    if (canonical_workspace.upload(lattice, no_ghosts) != NonlocalGpuFailure::None
        || permuted_workspace.upload(permuted, no_ghosts)
            != NonlocalGpuFailure::None) return 4;
    const auto canonical_graph = canonical_workspace.build_current_graph(
        NonlocalGpuVariant::Corrected, true, false);
    const auto permuted_graph = permuted_workspace.build_current_graph(
        NonlocalGpuVariant::Corrected, true, false);
    const auto reference_graph = build_reference_graph(
        profile, lattice, no_ghosts, false);
    if (!same_graph(reference_graph, canonical_graph)
        || !same_graph(canonical_graph, permuted_graph)) return 5;

    std::vector<NonlocalGpuSample> overflow;
    overflow.reserve(kMaximumNeighbors + 1U);
    for (std::uint32_t index = 0U; index <= kMaximumNeighbors; ++index) {
        overflow.push_back({100000U + index, {}, {}, {}});
    }
    NonlocalGpuWorkspace overflow_workspace(profile);
    if (overflow_workspace.upload(overflow, no_ghosts)
            != NonlocalGpuFailure::None
        || overflow_workspace.build_current_graph(
               NonlocalGpuVariant::Corrected, false, false)
               .failure
            != NonlocalGpuFailure::CapacityExceeded) return 6;

    const auto samples16 = make_lattice_state(profile, 40U, 20U, 20U, false, false);
    const auto samples50 = make_lattice_state(profile, 50U, 40U, 25U, false, false);
    const auto samples50_permuted = make_lattice_state(
        profile, 50U, 40U, 25U, true, false);
    const auto ghosts = make_basin_ghosts(profile);
    NonlocalGpuWorkspace workspace(profile);
    if (workspace.upload(samples16, ghosts) != NonlocalGpuFailure::None) return 7;
    const auto graph16 = workspace.build_current_graph(
        NonlocalGpuVariant::Corrected, false, false);
    if (graph16.failure != NonlocalGpuFailure::None
        || graph16.maximum_degree > kMaximumNeighbors) return 8;
    if (workspace.upload(samples50, ghosts) != NonlocalGpuFailure::None) return 9;
    const auto graph50 = workspace.build_current_graph(
        NonlocalGpuVariant::Corrected, true, false);
    if (graph50.failure != NonlocalGpuFailure::None
        || graph50.maximum_degree > kMaximumNeighbors) return 10;
    NonlocalGpuWorkspace permuted50_workspace(profile);
    if (permuted50_workspace.upload(samples50_permuted, ghosts)
            != NonlocalGpuFailure::None) return 11;
    const auto graph50_permuted = permuted50_workspace.build_current_graph(
        NonlocalGpuVariant::Corrected, true, false);
    if (!same_graph(graph50, graph50_permuted)) return 12;

    std::cout << std::setprecision(9)
              << "{\"research_id\":\"NCGP1\",\"mode\":\"graph-self-test\""
              << ",\"status\":\"PASS\",\"edge_root\":\""
              << graph_semantic_root(edge_gpu) << "\",\"lattice_root\":\""
              << graph_semantic_root(canonical_graph)
              << "\",\"graph16\":{\"pairs\":" << graph16.directed_pairs
              << ",\"max_degree\":" << graph16.maximum_degree
              << "},\"graph50\":{\"pairs\":" << graph50.directed_pairs
              << ",\"max_degree\":" << graph50.maximum_degree
              << ",\"root\":\"" << graph_semantic_root(graph50)
              << "\"},\"ghosts\":" << ghosts.size()
              << ",\"device_bytes\":" << workspace.allocated_device_bytes()
              << ",\"environment\":" << workspace.environment_json() << "}\n";
    return 0;
}

} // namespace

int main(int argc, char** argv) {
    try {
        if (argc == 2 && std::string(argv[1]) == "--graph-self-test") {
            return run_graph_self_test();
        }
        if (argc == 2 && std::string(argv[1]) == "--operator-self-test") {
            return run_operator_self_test();
        }
        std::cerr << "usage: nonlocal-corrected-cuda-full-step "
                     "--graph-self-test|--operator-self-test\n";
        return 64;
    } catch (const std::exception& error) {
        std::cerr << "NCGP1 failure: " << error.what() << '\n';
        return 1;
    }
}
