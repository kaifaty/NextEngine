#include "corrected_cuda_full_step.hpp"
#include "corrected_cuda_assembly.hpp"

#include <algorithm>
#include <chrono>
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

double cubic_weight(double radius, double horizon) {
    const double pi = std::acos(-1.0);
    const double q = 2.0 * radius / horizon;
    const double alpha = 3.0 / (2.0 * pi * horizon * horizon * horizon);
    if (q < 1.0) return alpha * (2.0 / 3.0 - q * q + 0.5 * q * q * q);
    if (q <= 2.0) {
        const double tail = 2.0 - q;
        return alpha * tail * tail * tail / 6.0;
    }
    return 0.0;
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

int run_solver_self_test() {
    const NonlocalGpuProfile profile = nonlocal_water_profile();
    const std::vector<NonlocalGpuGhost> no_ghosts;
    const std::vector<NonlocalGpuSample> free_fall{
        {17U, {0.5, 0.5, 0.5}, {0.5, 0.5, 0.5}, {}}};
    const double expected_z = 0.5
        + profile.dt * profile.dt * profile.gravity.z;
    NonlocalGpuWorkspace unpreconditioned_workspace(profile);
    NonlocalGpuWorkspace jacobi_workspace(profile);
    if (unpreconditioned_workspace.upload(free_fall, no_ghosts)
            != NonlocalGpuFailure::None
        || jacobi_workspace.upload(free_fall, no_ghosts)
            != NonlocalGpuFailure::None) return 30;
    const auto unpreconditioned = unpreconditioned_workspace.step(32U,
        NonlocalGpuSolverProfile::Unpreconditioned,
        NonlocalGpuVariant::Corrected, true, false);
    const auto jacobi = jacobi_workspace.step(32U,
        NonlocalGpuSolverProfile::Jacobi,
        NonlocalGpuVariant::Corrected, true, false);
    const auto cpu = step_reference(profile, free_fall, no_ghosts, 32U,
        NonlocalGpuVariant::Corrected, true);
    if (unpreconditioned.failure != NonlocalGpuFailure::None
        || jacobi.failure != NonlocalGpuFailure::None
        || cpu.failure != NonlocalGpuFailure::None
        || unpreconditioned.state.size() != 1U || jacobi.state.size() != 1U
        || cpu.state.size() != 1U
        || unpreconditioned.work.accepted_trials == 0U
        || jacobi.work.accepted_trials == 0U
        || unpreconditioned.scaled_displacement_residual > 1.0e-5
        || jacobi.scaled_displacement_residual > 1.0e-5) return 31;
    const double unpreconditioned_error = std::abs(
        unpreconditioned.state[0].current.z - expected_z);
    const double jacobi_error = std::abs(jacobi.state[0].current.z - expected_z);
    const double correspondence = std::abs(unpreconditioned.state[0].current.z
        - jacobi.state[0].current.z);
    const double cpu_correspondence = std::abs(
        unpreconditioned.state[0].current.z - cpu.state[0].current.z);
    if (unpreconditioned_error > 5.0e-6 || jacobi_error > 5.0e-6
        || correspondence > 5.0e-6 || cpu_correspondence > 5.0e-6) return 32;

    const std::vector<NonlocalGpuSample> contact{
        {19U, {0.5, 0.5, 0.025}, {0.5, 0.5, 0.025}, {0.0, 0.0, -2.0}}};
    NonlocalGpuWorkspace contact_workspace(profile);
    NonlocalGpuWorkspace disabled_workspace(profile);
    if (contact_workspace.upload(contact, no_ghosts) != NonlocalGpuFailure::None
        || disabled_workspace.upload(contact, no_ghosts)
            != NonlocalGpuFailure::None) return 33;
    const auto bounded = contact_workspace.step(32U,
        NonlocalGpuSolverProfile::Jacobi, NonlocalGpuVariant::Corrected,
        true, false);
    const auto disabled = disabled_workspace.step(32U,
        NonlocalGpuSolverProfile::Jacobi, NonlocalGpuVariant::DisableBoundary,
        true, false);
    if (bounded.failure != NonlocalGpuFailure::None
        || bounded.state.size() != 1U
        || bounded.state[0].current.z != static_cast<double>(
            static_cast<float>(0.025))
        || disabled.failure == NonlocalGpuFailure::None) return 34;

    std::cout << std::setprecision(12)
              << "{\"research_id\":\"NCGP1\",\"mode\":\"solver-self-test\""
              << ",\"status\":\"PASS\",\"free_fall_error_m\":"
              << unpreconditioned_error
              << ",\"jacobi_error_m\":" << jacobi_error
              << ",\"profile_correspondence_m\":" << correspondence
              << ",\"cpu_correspondence_m\":" << cpu_correspondence
              << ",\"unpreconditioned_hvp\":" << unpreconditioned.hvp_used
              << ",\"jacobi_hvp\":" << jacobi.hvp_used
              << ",\"bounded_contact_z\":" << bounded.state[0].current.z
              << ",\"disabled_boundary_failure\":"
              << static_cast<std::uint32_t>(disabled.failure)
              << ",\"work_root\":\""
              << work_semantic_root(jacobi.work) << "\"}\n";
    return 0;
}

struct TinyCase {
    std::string name;
    NonlocalGpuProfile profile;
    std::vector<NonlocalGpuSample> state;
};

std::vector<TinyCase> tiny_cases() {
    std::vector<TinyCase> result;
    TinyCase pair;
    pair.name = "compressed-pair";
    pair.profile = nonlocal_water_profile();
    pair.profile.gravity = {};
    pair.profile.kappa = 500.0;
    pair.profile.rest_density = pair.profile.mass
        * (cubic_weight(0.0, pair.profile.horizon)
            + cubic_weight(0.05, pair.profile.horizon)) / 1.1;
    pair.state = {
        {101U, {0.725, 0.75, 0.75}, {0.725, 0.75, 0.75}, {}},
        {202U, {0.775, 0.75, 0.75}, {0.775, 0.75, 0.75}, {}}};
    result.push_back(pair);

    TinyCase tetra;
    tetra.name = "combined-tetrahedron";
    tetra.profile = nonlocal_water_profile();
    tetra.profile.gravity = {};
    tetra.profile.kappa = 200.0;
    tetra.profile.lambda = 20.0;
    tetra.profile.mu = 10.0;
    tetra.profile.gamma = 100.0;
    const std::array<Vec3d, 4> reference{{
        {0.75, 0.75, 0.75},
        {0.79, 0.75, 0.75},
        {0.77, 0.7846410161513775, 0.75},
        {0.77, 0.7615470053837925, 0.7826598632371090}}};
    const std::array<Vec3d, 4> velocity{{
        {0.4, -0.2, 0.1}, {-0.1, 0.3, -0.2},
        {0.2, 0.1, 0.3}, {-0.3, -0.2, -0.2}}};
    double density = tetra.profile.mass
        * cubic_weight(0.0, tetra.profile.horizon);
    for (std::size_t index = 1U; index < reference.size(); ++index) {
        const Vec3d delta{reference[0].x - reference[index].x,
            reference[0].y - reference[index].y,
            reference[0].z - reference[index].z};
        density += tetra.profile.mass * cubic_weight(std::sqrt(
            delta.x * delta.x + delta.y * delta.y + delta.z * delta.z),
            tetra.profile.horizon);
    }
    tetra.profile.rest_density = density / 1.1;
    for (std::size_t index = 0U; index < reference.size(); ++index) {
        const Vec3d current{reference[index].x
                + tetra.profile.dt * velocity[index].x,
            reference[index].y + tetra.profile.dt * velocity[index].y,
            reference[index].z + tetra.profile.dt * velocity[index].z};
        tetra.state.push_back({static_cast<std::uint32_t>(100U + 17U * index),
            reference[index], current, velocity[index]});
    }
    result.push_back(tetra);
    return result;
}

int run_tiny_solver_correspondence() {
    double corpus_maximum = 0.0;
    std::uint32_t gpu_hvp = 0U;
    std::uint32_t cpu_hvp = 0U;
    for (const TinyCase& input : tiny_cases()) {
        const std::vector<NonlocalGpuGhost> no_ghosts;
        NonlocalGpuWorkspace gpu_workspace(input.profile);
        if (gpu_workspace.upload(input.state, no_ghosts)
            != NonlocalGpuFailure::None) return 35;
        const auto gpu = gpu_workspace.step(128U,
            NonlocalGpuSolverProfile::Unpreconditioned,
            NonlocalGpuVariant::Corrected, true, false);
        const auto cpu = step_reference(input.profile, input.state,
            no_ghosts, 128U, NonlocalGpuVariant::Corrected, true);
        auto permuted = input.state;
        std::reverse(permuted.begin(), permuted.end());
        NonlocalGpuWorkspace permuted_workspace(input.profile);
        if (permuted_workspace.upload(permuted, no_ghosts)
            != NonlocalGpuFailure::None) return 36;
        const auto permuted_gpu = permuted_workspace.step(128U,
            NonlocalGpuSolverProfile::Unpreconditioned,
            NonlocalGpuVariant::Corrected, true, false);
        if (gpu.failure != NonlocalGpuFailure::None
            || cpu.failure != NonlocalGpuFailure::None
            || permuted_gpu.failure != NonlocalGpuFailure::None
            || gpu.state.size() != cpu.state.size()
            || gpu.state.size() != permuted_gpu.state.size()) {
            std::cerr << input.name << " gpu="
                      << static_cast<std::uint32_t>(gpu.failure)
                      << " cpu=" << static_cast<std::uint32_t>(cpu.failure)
                      << " permuted="
                      << static_cast<std::uint32_t>(permuted_gpu.failure)
                      << " gpu_hvp=" << gpu.hvp_used
                      << " cpu_hvp=" << cpu.hvp_used
                      << " gpu_outer=" << gpu.outer_trials
                      << " gpu_scaled=" << gpu.scaled_displacement_residual
                      << " gpu_gradient=" << gpu.gradient_norm
                      << " gpu_accept=" << gpu.work.accepted_trials
                      << " gpu_reject=" << gpu.work.rejected_trials << '\n';
            std::cout << std::setprecision(12)
                      << "{\"research_id\":\"NCGP1\",\"mode\":"
                         "\"tiny-solver-correspondence\",\"status\":"
                         "\"PHYSICS_REFUTED\",\"first_case\":\""
                      << input.name << "\",\"gpu_failure\":"
                      << static_cast<std::uint32_t>(gpu.failure)
                      << ",\"cpu_failure\":"
                      << static_cast<std::uint32_t>(cpu.failure)
                      << ",\"permuted_gpu_failure\":"
                      << static_cast<std::uint32_t>(permuted_gpu.failure)
                      << ",\"gpu_hvp\":" << gpu.hvp_used
                      << ",\"cpu_hvp\":" << cpu.hvp_used
                      << ",\"gpu_outer_trials\":" << gpu.outer_trials
                      << ",\"gpu_scaled_residual\":"
                      << gpu.scaled_displacement_residual
                      << ",\"gpu_gradient_norm\":" << gpu.gradient_norm
                      << ",\"gpu_accepted\":" << gpu.work.accepted_trials
                      << ",\"gpu_rejected\":" << gpu.work.rejected_trials
                      << ",\"gpu_work_root\":\""
                      << work_semantic_root(gpu.work) << "\"}\n";
            return 37;
        }
        for (std::size_t index = 0U; index < gpu.state.size(); ++index) {
            if (gpu.state[index].sample_id != cpu.state[index].sample_id
                || gpu.state[index].sample_id
                    != permuted_gpu.state[index].sample_id) return 38;
            const Vec3d difference{
                gpu.state[index].current.x - cpu.state[index].current.x,
                gpu.state[index].current.y - cpu.state[index].current.y,
                gpu.state[index].current.z - cpu.state[index].current.z};
            corpus_maximum = std::max(corpus_maximum, std::sqrt(
                difference.x * difference.x + difference.y * difference.y
                    + difference.z * difference.z));
            if (gpu.state[index].current.x != permuted_gpu.state[index].current.x
                || gpu.state[index].current.y
                    != permuted_gpu.state[index].current.y
                || gpu.state[index].current.z
                    != permuted_gpu.state[index].current.z) return 39;
            const bool gpu_active = gpu.density[index]
                > static_cast<double>(static_cast<float>(
                    input.profile.rest_density));
            const bool cpu_active = cpu.density[index]
                > input.profile.rest_density;
            if (gpu_active != cpu_active) return 42;
        }
        gpu_hvp = std::max(gpu_hvp, gpu.hvp_used);
        cpu_hvp = std::max(cpu_hvp, cpu.hvp_used);
    }
    const bool passed = corpus_maximum <= 5.0e-6;
    std::cout << std::setprecision(12)
              << "{\"research_id\":\"NCGP1\",\"mode\":\"tiny-solver-correspondence\""
              << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
              << ",\"maximum_position_error_m\":" << corpus_maximum
              << ",\"gpu_hvp_max\":" << gpu_hvp
              << ",\"cpu_hvp_max\":" << cpu_hvp
              << ",\"active_signature_exact\":true"
              << ",\"permutation_exact\":true}\n";
    return passed ? 0 : 43;
}

int run_step_probe(std::uint32_t budget, bool full, bool cpu) {
    const NonlocalGpuProfile profile = nonlocal_water_profile();
    const auto state = full
        ? make_lattice_state(profile, 50U, 40U, 25U, false, false)
        : make_lattice_state(profile, 20U, 20U, 10U, false, false);
    const auto ghosts = make_basin_ghosts(profile);
    const auto started = std::chrono::steady_clock::now();
    NonlocalGpuStepResult result;
    if (cpu) {
        result = step_reference(profile, state, ghosts, budget,
            NonlocalGpuVariant::Corrected, false);
    } else {
        NonlocalGpuWorkspace workspace(profile);
        if (workspace.upload(state, ghosts) != NonlocalGpuFailure::None) return 40;
        result = workspace.step(budget, NonlocalGpuSolverProfile::Jacobi,
            NonlocalGpuVariant::Corrected, false, true);
    }
    const double wall_ms = std::chrono::duration<double, std::milli>(
        std::chrono::steady_clock::now() - started).count();
    std::cout << std::setprecision(12)
              << "{\"research_id\":\"NCGP1\",\"mode\":\"step-probe-"
              << (full ? "50k" : "4k") << "\""
              << ",\"implementation\":\"" << (cpu ? "cpu" : "cuda") << "\""
              << ",\"failure\":" << static_cast<std::uint32_t>(result.failure)
              << ",\"budget\":" << budget << ",\"hvp_used\":"
              << result.hvp_used << ",\"outer_trials\":"
              << result.outer_trials << ",\"accepted\":"
              << result.work.accepted_trials << ",\"rejected\":"
              << result.work.rejected_trials << ",\"gradient_norm\":"
              << result.gradient_norm << ",\"scaled_residual\":"
              << result.scaled_displacement_residual
              << ",\"penetration_m\":" << result.maximum_penetration_m
              << ",\"timing_ms\":{\"total\":" << result.timing.total_ms
              << ",\"graph\":" << result.timing.graph_ms
              << ",\"evaluation\":"
              << result.timing.density_energy_gradient_ms
              << ",\"hvp\":" << result.timing.hvp_ms
              << ",\"control\":" << result.timing.solver_control_ms
              << "},\"wall_ms\":" << wall_ms << ",\"work_root\":\""
              << work_semantic_root(result.work)
              << "\"}\n";
    return result.failure == NonlocalGpuFailure::None ? 0 : 41;
}

int run_correspondence_4k(std::uint32_t budget) {
    const NonlocalGpuProfile profile = nonlocal_water_profile();
    const auto initial = make_lattice_state(
        profile, 20U, 20U, 10U, false, false);
    const auto ghosts = make_basin_ghosts(profile);
    NonlocalGpuWorkspace workspace(profile);
    if (workspace.upload(initial, ghosts) != NonlocalGpuFailure::None) return 50;
    const auto gpu = workspace.step(budget, NonlocalGpuSolverProfile::Jacobi,
        NonlocalGpuVariant::Corrected, true, false);
    const auto cpu = step_reference(profile, initial, ghosts, budget,
        NonlocalGpuVariant::Corrected, true);
    if (gpu.failure != NonlocalGpuFailure::None
        || cpu.failure != NonlocalGpuFailure::None
        || gpu.state.size() != cpu.state.size()
        || gpu.density.size() != cpu.density.size()
        || gpu.state.size() != gpu.density.size()) return 51;
    double position_squared = 0.0;
    double position_maximum = 0.0;
    double density_squared = 0.0;
    double density_maximum = 0.0;
    for (std::size_t index = 0U; index < gpu.state.size(); ++index) {
        if (gpu.state[index].sample_id != cpu.state[index].sample_id) return 52;
        const Vec3d delta{gpu.state[index].current.x - cpu.state[index].current.x,
            gpu.state[index].current.y - cpu.state[index].current.y,
            gpu.state[index].current.z - cpu.state[index].current.z};
        const double distance = std::sqrt(delta.x * delta.x
            + delta.y * delta.y + delta.z * delta.z);
        position_squared += distance * distance;
        position_maximum = std::max(position_maximum, distance);
        const double density_error = std::abs(gpu.density[index]
            - cpu.density[index]);
        density_squared += density_error * density_error;
        density_maximum = std::max(density_maximum, density_error);
    }
    const double position_rmse = std::sqrt(
        position_squared / static_cast<double>(gpu.state.size()));
    const double density_rmse = std::sqrt(
        density_squared / static_cast<double>(gpu.state.size()))
        / profile.rest_density;
    const double density_max = density_maximum / profile.rest_density;
    const bool passed = position_rmse <= 0.0025 && position_maximum <= 0.005
        && density_rmse <= 0.05 && density_max <= 0.10
        && gpu.active_pressure_centers == cpu.active_pressure_centers;
    std::cout << std::setprecision(12)
              << "{\"research_id\":\"NCGP1\",\"mode\":\"correspondence-4k\""
              << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << "\""
              << ",\"budget\":" << budget << ",\"position_rmse_m\":"
              << position_rmse << ",\"position_max_m\":" << position_maximum
              << ",\"density_rmse_fraction\":" << density_rmse
              << ",\"density_max_fraction\":" << density_max
              << ",\"gpu_active\":" << gpu.active_pressure_centers
              << ",\"cpu_active\":" << cpu.active_pressure_centers
              << ",\"gpu_hvp\":" << gpu.hvp_used
              << ",\"cpu_hvp\":" << cpu.hvp_used << "}\n";
    return passed ? 0 : 53;
}

std::vector<NonlocalGpuSample> trajectory_initial(
    const NonlocalGpuProfile& profile, const std::string& name) {
    if (name == "hydrostatic-hold") {
        return make_lattice_state(profile, 20U, 20U, 10U, false, false);
    }
    if (name == "dam-break") {
        return make_lattice_state(profile, 10U, 20U, 20U, false, false);
    }
    if (name == "orifice-jet") {
        auto result = make_lattice_state(
            profile, 20U, 20U, 10U, false, false);
        for (auto& sample : result) {
            const double dy = sample.current.y - 0.75;
            const double dz = sample.current.z - 0.25;
            if (dy * dy + dz * dz <= 0.15 * 0.15) {
                sample.velocity.x = 1.5;
            }
        }
        return result;
    }
    throw std::invalid_argument("unknown NCGP1 trajectory");
}

int run_trajectory_4k(const std::string& name,
    std::uint32_t steps,
    std::uint32_t gpu_budget) {
    const NonlocalGpuProfile profile = nonlocal_water_profile();
    const auto ghosts = make_basin_ghosts(profile);
    auto cpu_state = trajectory_initial(profile, name);
    const auto initial = cpu_state;
    NonlocalGpuWorkspace gpu_workspace(profile);
    if (gpu_workspace.upload(initial, ghosts) != NonlocalGpuFailure::None) return 60;
    double maximum_position_rmse = 0.0;
    double maximum_position_error = 0.0;
    double maximum_density_rmse = 0.0;
    double maximum_density_error = 0.0;
    std::uint32_t maximum_gpu_hvp = 0U;
    std::uint32_t maximum_cpu_hvp = 0U;
    std::uint64_t active_mismatches = 0U;
    const auto started = std::chrono::steady_clock::now();
    for (std::uint32_t step = 0U; step < steps; ++step) {
        const auto gpu = gpu_workspace.step(gpu_budget,
            NonlocalGpuSolverProfile::Jacobi, NonlocalGpuVariant::Corrected,
            true, false);
        const auto cpu = step_reference(profile, cpu_state, ghosts, 128U,
            NonlocalGpuVariant::Corrected, true);
        if (gpu.failure != NonlocalGpuFailure::None
            || cpu.failure != NonlocalGpuFailure::None
            || gpu.state.size() != cpu.state.size()
            || gpu.density.size() != cpu.density.size()) {
            std::cerr << "trajectory " << name << " failed at step " << step
                      << " gpu=" << static_cast<std::uint32_t>(gpu.failure)
                      << " cpu=" << static_cast<std::uint32_t>(cpu.failure)
                      << '\n';
            return 61;
        }
        double position_squared = 0.0;
        double density_squared = 0.0;
        for (std::size_t index = 0U; index < gpu.state.size(); ++index) {
            if (gpu.state[index].sample_id != cpu.state[index].sample_id) return 62;
            const Vec3d delta{
                gpu.state[index].current.x - cpu.state[index].current.x,
                gpu.state[index].current.y - cpu.state[index].current.y,
                gpu.state[index].current.z - cpu.state[index].current.z};
            const double distance = std::sqrt(delta.x * delta.x
                + delta.y * delta.y + delta.z * delta.z);
            position_squared += distance * distance;
            maximum_position_error = std::max(maximum_position_error, distance);
            const double density_error = std::abs(
                gpu.density[index] - cpu.density[index]) / profile.rest_density;
            density_squared += density_error * density_error;
            maximum_density_error = std::max(
                maximum_density_error, density_error);
        }
        maximum_position_rmse = std::max(maximum_position_rmse,
            std::sqrt(position_squared / static_cast<double>(gpu.state.size())));
        maximum_density_rmse = std::max(maximum_density_rmse,
            std::sqrt(density_squared / static_cast<double>(gpu.state.size())));
        active_mismatches += gpu.active_pressure_centers
            != cpu.active_pressure_centers ? 1U : 0U;
        maximum_gpu_hvp = std::max(maximum_gpu_hvp, gpu.hvp_used);
        maximum_cpu_hvp = std::max(maximum_cpu_hvp, cpu.hvp_used);
        cpu_state = cpu.state;
    }
    const double wall_seconds = std::chrono::duration<double>(
        std::chrono::steady_clock::now() - started).count();
    const bool passed = maximum_position_rmse <= 0.0025
        && maximum_position_error <= 0.005
        && maximum_density_rmse <= 0.05
        && maximum_density_error <= 0.10 && active_mismatches == 0U;
    std::cout << std::setprecision(12)
              << "{\"research_id\":\"NCGP1\",\"mode\":\"trajectory-4k\""
              << ",\"scenario\":\"" << name << "\",\"status\":\""
              << (passed ? "PASS" : "FAIL") << "\",\"steps\":" << steps
              << ",\"gpu_budget\":" << gpu_budget
              << ",\"position_rmse_max_m\":" << maximum_position_rmse
              << ",\"position_error_max_m\":" << maximum_position_error
              << ",\"density_rmse_max_fraction\":" << maximum_density_rmse
              << ",\"density_error_max_fraction\":" << maximum_density_error
              << ",\"active_mismatch_steps\":" << active_mismatches
              << ",\"gpu_hvp_max\":" << maximum_gpu_hvp
              << ",\"cpu_hvp_max\":" << maximum_cpu_hvp
              << ",\"wall_seconds\":" << wall_seconds << "}\n";
    return passed ? 0 : 63;
}

int run_gpu_trajectory_probe(const std::string& name,
    std::uint32_t steps,
    std::uint32_t budget) {
    const NonlocalGpuProfile profile = nonlocal_water_profile();
    const auto ghosts = make_basin_ghosts(profile);
    const auto initial = trajectory_initial(profile, name);
    NonlocalGpuWorkspace workspace(profile);
    if (workspace.upload(initial, ghosts) != NonlocalGpuFailure::None) return 70;
    for (std::uint32_t step = 0U; step < steps; ++step) {
        const auto result = workspace.step(budget,
            NonlocalGpuSolverProfile::Jacobi, NonlocalGpuVariant::Corrected,
            false, false);
        std::cout << std::setprecision(12)
                  << "{\"research_id\":\"NCGP1\",\"mode\":\"gpu-trajectory-probe\""
                  << ",\"scenario\":\"" << name << "\",\"step\":" << step
                  << ",\"failure\":" << static_cast<std::uint32_t>(result.failure)
                  << ",\"hvp\":" << result.hvp_used
                  << ",\"diagonal_probes\":" << result.work.diagonal_probes
                  << ",\"outer_trials\":" << result.outer_trials
                  << ",\"accepted\":" << result.work.accepted_trials
                  << ",\"rejected\":" << result.work.rejected_trials
                  << ",\"scaled_residual\":"
                  << result.scaled_displacement_residual
                  << ",\"gradient_norm\":" << result.gradient_norm
                  << ",\"penetration_m\":" << result.maximum_penetration_m
                  << ",\"work_root\":\"" << work_semantic_root(result.work)
                  << "\"}\n";
        if (result.failure != NonlocalGpuFailure::None) return 71;
    }
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
        if (argc == 2 && std::string(argv[1]) == "--solver-self-test") {
            return run_solver_self_test();
        }
        if (argc == 2 && std::string(argv[1])
                == "--tiny-solver-correspondence") {
            return run_tiny_solver_correspondence();
        }
        if (argc == 3 && (std::string(argv[1]) == "--step-probe-4k"
                || std::string(argv[1]) == "--step-probe-50k"
                || std::string(argv[1]) == "--cpu-step-probe-4k")) {
            return run_step_probe(static_cast<std::uint32_t>(
                std::stoul(argv[2])), std::string(argv[1]) == "--step-probe-50k",
                std::string(argv[1]) == "--cpu-step-probe-4k");
        }
        if (argc == 3 && std::string(argv[1]) == "--correspondence-4k") {
            return run_correspondence_4k(static_cast<std::uint32_t>(
                std::stoul(argv[2])));
        }
        if (argc == 5 && std::string(argv[1]) == "--trajectory-4k") {
            return run_trajectory_4k(argv[2], static_cast<std::uint32_t>(
                std::stoul(argv[3])), static_cast<std::uint32_t>(
                std::stoul(argv[4])));
        }
        if (argc == 5 && std::string(argv[1]) == "--gpu-trajectory-probe") {
            return run_gpu_trajectory_probe(argv[2],
                static_cast<std::uint32_t>(std::stoul(argv[3])),
                static_cast<std::uint32_t>(std::stoul(argv[4])));
        }
        std::cerr << "usage: nonlocal-corrected-cuda-full-step "
                     "--graph-self-test|--operator-self-test|"
                     "--solver-self-test|--step-probe-4k BUDGET|"
                     "--step-probe-50k BUDGET|--cpu-step-probe-4k BUDGET\n";
        return 64;
    } catch (const std::exception& error) {
        std::cerr << "NCGP1 failure: " << error.what() << '\n';
        return 1;
    }
}
