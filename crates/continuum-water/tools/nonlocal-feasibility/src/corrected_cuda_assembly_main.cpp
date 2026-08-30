#include "corrected_cuda_assembly.hpp"

#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <limits>
#include <sstream>
#include <string>
#include <utility>
#include <vector>

namespace nextengine::nonlocal::gpu_assembly_audit {
namespace {

constexpr int COLD_REPEATS = 10;

AssemblySample sample(std::uint32_t id,
    std::array<std::int64_t, 3> reference,
    std::array<std::int64_t, 3> predicted,
    std::array<std::int64_t, 3> current,
    std::array<std::int32_t, 3> direction) {
    return {id, reference, predicted, current, direction};
}

AssemblyFixture make_cluster(bool combined) {
    AssemblyFixture fixture;
    fixture.name = combined ? "combined_cluster" : "active_pressure_cluster";
    fixture.terms = combined
        ? AssemblyTerms{true, true, true, true}
        : AssemblyTerms{false, true, false, false};
    fixture.samples.reserve(100U);
    for (std::size_t input = 0; input < 100U; ++input) {
        const std::size_t logical = (37U * input + 11U) % 100U;
        const std::int64_t x = (static_cast<std::int64_t>(logical % 5U) - 2) * 1000;
        const std::int64_t y = (static_cast<std::int64_t>((logical / 5U) % 5U) - 2)
            * 1000;
        const std::int64_t z = (static_cast<std::int64_t>(logical / 25U) - 1) * 1000;
        const std::array<std::int64_t, 3> reference{x, y, z};
        std::array<std::int64_t, 3> predicted = reference;
        std::array<std::int64_t, 3> current = reference;
        if (combined) {
            predicted[0] += static_cast<std::int64_t>((17U * logical + 3U) % 41U) - 20;
            predicted[1] += static_cast<std::int64_t>((13U * logical + 7U) % 37U) - 18;
            predicted[2] += static_cast<std::int64_t>((19U * logical + 5U) % 43U) - 21;
            current[0] += static_cast<std::int64_t>((29U * logical + 11U) % 101U) - 50;
            current[1] += static_cast<std::int64_t>((31U * logical + 17U) % 97U) - 48;
            current[2] += static_cast<std::int64_t>((23U * logical + 19U) % 89U) - 44;
        }
        const std::array<std::int32_t, 3> direction{
            static_cast<std::int32_t>((43U * logical + 5U) % 1701U) - 850,
            static_cast<std::int32_t>((47U * logical + 9U) % 1601U) - 800,
            static_cast<std::int32_t>((53U * logical + 13U) % 1801U) - 900};
        fixture.samples.push_back(sample(
            static_cast<std::uint32_t>(10000U + 17U * logical),
            reference, predicted, current, direction));
    }
    return fixture;
}

std::vector<AssemblyFixture> make_fixtures() {
    std::vector<AssemblyFixture> fixtures;
    fixtures.push_back({"isolated_inertia", {true, false, false, false},
        {sample(42U, {0, 0, 0}, {1000, -500, 250}, {1400, -700, 900},
            {700, -300, 500})}});
    fixtures.push_back({"inactive_pressure_cloud", {false, true, false, false},
        {
            sample(90U, {0, 0, 0}, {0, 0, 0}, {0, 0, 0}, {500, -300, 200}),
            sample(4U, {62000, 7000, 1000}, {62000, 7000, 1000},
                {62000, 7000, 1000}, {-200, 700, 100}),
            sample(55U, {-81000, 13000, 4000}, {-81000, 13000, 4000},
                {-81000, 13000, 4000}, {300, 100, -600}),
            sample(801U, {9000, -93000, 17000}, {9000, -93000, 17000},
                {9000, -93000, 17000}, {-400, -200, 800}),
        }});
    fixtures.push_back({"reference_current_support_crossing",
        {false, false, true, true},
        {
            sample(700U, {0, 0, 0}, {0, 0, 0}, {0, 0, 0}, {600, -200, 300}),
            sample(31U, {149000, 0, 0}, {151000, 0, 0}, {151000, 0, 0},
                {-500, 400, 100}),
            sample(99U, {500000, 0, 0}, {500000, 0, 0}, {500000, 0, 0},
                {200, 700, -300}),
            sample(12U, {651000, 0, 0}, {649000, 0, 0}, {649000, 0, 0},
                {-300, -600, 500}),
        }});
    fixtures.push_back({"oblique_viscosity", {false, false, true, false},
        {
            sample(77U, {0, 0, 0}, {0, 0, 0}, {2500, -1700, 900},
                {700, -400, 500}),
            sample(9U, {70000, 40000, 30000}, {70000, 40000, 30000},
                {68400, 42100, 28700}, {-600, 800, 200}),
            sample(401U, {-50000, 65000, -20000}, {-50000, 65000, -20000},
                {-49200, 63600, -17800}, {300, 100, -700}),
        }});
    fixtures.push_back({"surface_two_branches", {false, false, false, true},
        {
            sample(60U, {0, 0, 0}, {0, 0, 0}, {0, 0, 0}, {500, 200, -300}),
            sample(5U, {40000, 0, 0}, {40000, 0, 0}, {40000, 0, 0},
                {-400, 700, 100}),
            sample(800U, {300000, 0, 0}, {300000, 0, 0}, {300000, 0, 0},
                {300, -600, 400}),
            sample(17U, {400000, 0, 0}, {400000, 0, 0}, {400000, 0, 0},
                {-200, -300, -500}),
        }});
    fixtures.push_back(make_cluster(false));
    fixtures.push_back(make_cluster(true));
    AssemblyFixture permuted = fixtures.back();
    permuted.name = "combined_cluster_permuted";
    std::reverse(permuted.samples.begin(), permuted.samples.end());
    fixtures.push_back(std::move(permuted));
    return fixtures;
}

void append_u32(std::string& bytes, std::uint32_t value) {
    for (unsigned int shift = 0; shift < 32U; shift += 8U) {
        bytes.push_back(static_cast<char>((value >> shift) & 0xffU));
    }
}

void append_u64(std::string& bytes, std::uint64_t value) {
    for (unsigned int shift = 0; shift < 64U; shift += 8U) {
        bytes.push_back(static_cast<char>((value >> shift) & 0xffU));
    }
}

void append_i64(std::string& bytes, std::int64_t value) {
    append_u64(bytes, static_cast<std::uint64_t>(value));
}

void append_double(std::string& bytes, double value) {
    std::uint64_t bits = 0U;
    static_assert(sizeof(bits) == sizeof(value), "NCGA2 binary64 layout");
    std::memcpy(&bits, &value, sizeof(bits));
    append_u64(bytes, bits);
}

std::string fixture_material(const std::vector<AssemblyFixture>& fixtures) {
    std::string bytes = "nextengine.nonlocal.ncga2.fixtures.v1\0";
    append_u32(bytes, static_cast<std::uint32_t>(fixtures.size()));
    for (const AssemblyFixture& fixture : fixtures) {
        append_u32(bytes, static_cast<std::uint32_t>(fixture.name.size()));
        bytes.append(fixture.name);
        append_u32(bytes, fixture.terms.inertia ? 1U : 0U);
        append_u32(bytes, fixture.terms.pressure ? 1U : 0U);
        append_u32(bytes, fixture.terms.viscosity ? 1U : 0U);
        append_u32(bytes, fixture.terms.surface ? 1U : 0U);
        append_u32(bytes, static_cast<std::uint32_t>(fixture.samples.size()));
        for (const AssemblySample& value : fixture.samples) {
            append_u32(bytes, value.sample_id);
            for (std::int64_t component : value.reference_um) append_i64(bytes, component);
            for (std::int64_t component : value.predicted_um) append_i64(bytes, component);
            for (std::int64_t component : value.current_um) append_i64(bytes, component);
            for (std::int32_t component : value.direction_milli) {
                append_u32(bytes, static_cast<std::uint32_t>(component));
            }
        }
    }
    return bytes;
}

std::string payload_material(const AssemblyResult& result) {
    std::string bytes = "nextengine.nonlocal.ncga2.payload.v1\0";
    append_u32(bytes, static_cast<std::uint32_t>(result.failure));
    const auto append_u32_vector = [&bytes](const auto& values) {
        append_u32(bytes, static_cast<std::uint32_t>(values.size()));
        for (const auto value : values) append_u32(bytes, static_cast<std::uint32_t>(value));
    };
    append_u32_vector(result.owner_ids);
    append_u32_vector(result.current_offsets);
    append_u32_vector(result.current_neighbors);
    append_u32_vector(result.reference_offsets);
    append_u32_vector(result.reference_neighbors);
    append_u32(bytes, static_cast<std::uint32_t>(result.density.size()));
    for (double value : result.density) append_double(bytes, value);
    for (double value : result.energy) append_double(bytes, value);
    for (const auto* values : {&result.gradient, &result.hessian,
             &result.diagonal_blocks, &result.hvp}) {
        append_u32(bytes, static_cast<std::uint32_t>(values->size()));
        for (double value : *values) append_double(bytes, value);
    }
    append_u32(bytes, static_cast<std::uint32_t>(result.pressure_active.size()));
    for (std::uint8_t value : result.pressure_active) append_u32(bytes, value);
    append_double(bytes, result.minimum_density_clamp_margin);
    append_i64(bytes, result.minimum_current_support_margin_um);
    append_i64(bytes, result.minimum_reference_support_margin_um);
    append_i64(bytes, result.minimum_surface_branch_margin_um);
    return bytes;
}

std::string work_material(const AssemblyWorkReceipt& work) {
    std::string bytes = "nextengine.nonlocal.ncga2.work.v1\0";
    append_u64(bytes, work.owner_sort_comparisons);
    append_u64(bytes, work.graph_distance_predicates);
    append_u64(bytes, work.density_kernel_evaluations);
    append_u64(bytes, work.energy_pair_visits);
    append_u64(bytes, work.gradient_pair_visits);
    append_u64(bytes, work.active_pressure_centers);
    append_u64(bytes, work.pressure_outer_products);
    append_u64(bytes, work.pressure_geometric_blocks);
    append_u64(bytes, work.viscosity_curvature_blocks);
    append_u64(bytes, work.surface_curvature_blocks);
    append_u64(bytes, work.dense_entries_written);
    append_u64(bytes, work.hvp_products);
    append_u64(bytes, work.host_device_scalar_transfers);
    append_u64(bytes, work.compensated_additions);
    append_u64(bytes, work.compensation_initializations);
    return bytes;
}

struct ErrorSummary {
    bool passed = true;
    double maximum_mixed = 0.0;
    double maximum_zero = 0.0;
    std::size_t mixed_index = 0U;
    std::size_t zero_index = 0U;
    double mixed_reference = 0.0;
    double mixed_candidate = 0.0;
};

void compare_values(const std::vector<double>& reference,
    const std::vector<double>& candidate, double zero_bound,
    ErrorSummary& summary) {
    if (reference.size() != candidate.size()) {
        summary.passed = false;
        return;
    }
    for (std::size_t index = 0; index < reference.size(); ++index) {
        if (!std::isfinite(reference[index]) || !std::isfinite(candidate[index])) {
            summary.passed = false;
            continue;
        }
        const double error = std::abs(candidate[index] - reference[index]);
        if (reference[index] == 0.0) {
            if (error > summary.maximum_zero) {
                summary.maximum_zero = error;
                summary.zero_index = index;
            }
            summary.passed = summary.passed && error <= zero_bound;
        } else {
            const double mixed = error / std::max(1.0, std::abs(reference[index]));
            if (mixed > summary.maximum_mixed) {
                summary.maximum_mixed = mixed;
                summary.mixed_index = index;
                summary.mixed_reference = reference[index];
                summary.mixed_candidate = candidate[index];
            }
            summary.passed = summary.passed && mixed <= 2.0e-4;
        }
    }
}

std::vector<double> direction_in_canonical_order(const AssemblyFixture& fixture) {
    std::vector<const AssemblySample*> ordered;
    ordered.reserve(fixture.samples.size());
    for (const AssemblySample& value : fixture.samples) ordered.push_back(&value);
    std::sort(ordered.begin(), ordered.end(), [](const AssemblySample* lhs,
        const AssemblySample* rhs) { return lhs->sample_id < rhs->sample_id; });
    std::vector<double> result;
    result.reserve(3U * ordered.size());
    for (const AssemblySample* value : ordered) {
        for (std::int32_t component : value->direction_milli) {
            result.push_back(static_cast<double>(component) / 1000.0);
        }
    }
    return result;
}

struct CaseReport {
    bool passed = false;
    bool graph_exact = false;
    bool derivatives_passed = false;
    bool hvp_consistent = false;
    bool symmetry_passed = false;
    bool cold_exact = false;
    ErrorSummary errors;
    ErrorSummary density_errors;
    ErrorSummary energy_errors;
    ErrorSummary gradient_errors;
    ErrorSummary hessian_errors;
    ErrorSummary block_errors;
    ErrorSummary hvp_value_errors;
    double first_derivative_error = 0.0;
    double second_derivative_error = 0.0;
    double hvp_error = 0.0;
    double symmetry_error = 0.0;
    std::string payload_root;
    std::string work_root;
};

CaseReport compare_case(const AssemblyProfile& profile,
    const AssemblyFixture& fixture, const AssemblyResult& reference,
    const AssemblyResult& candidate, bool run_cold) {
    CaseReport report;
    report.graph_exact = reference.failure == AssemblyFailure::None
        && candidate.failure == AssemblyFailure::None
        && reference.owner_ids == candidate.owner_ids
        && reference.current_offsets == candidate.current_offsets
        && reference.current_neighbors == candidate.current_neighbors
        && reference.reference_offsets == candidate.reference_offsets
        && reference.reference_neighbors == candidate.reference_neighbors
        && reference.pressure_active == candidate.pressure_active;
    compare_values(reference.density, candidate.density, 2.0e-6,
        report.density_errors);
    compare_values(std::vector<double>(reference.energy.begin(), reference.energy.end()),
        std::vector<double>(candidate.energy.begin(), candidate.energy.end()),
        2.0e-6, report.energy_errors);
    compare_values(reference.gradient, candidate.gradient, 2.0e-6,
        report.gradient_errors);
    compare_values(reference.hessian, candidate.hessian, 2.0e-5,
        report.hessian_errors);
    compare_values(reference.diagonal_blocks, candidate.diagonal_blocks,
        2.0e-5, report.block_errors);
    compare_values(reference.hvp, candidate.hvp, 2.0e-6,
        report.hvp_value_errors);
    for (const ErrorSummary* field : {&report.density_errors,
             &report.energy_errors, &report.gradient_errors,
             &report.hessian_errors, &report.block_errors,
             &report.hvp_value_errors}) {
        report.errors.passed = report.errors.passed && field->passed;
        report.errors.maximum_mixed = std::max(
            report.errors.maximum_mixed, field->maximum_mixed);
        report.errors.maximum_zero = std::max(
            report.errors.maximum_zero, field->maximum_zero);
    }

    const std::vector<double> direction = direction_in_canonical_order(fixture);
    const EnergyDerivativeResult derivative = evaluate_energy_derivatives(profile, fixture);
    double gradient_projection = 0.0;
    double hessian_projection = 0.0;
    for (std::size_t index = 0; index < direction.size(); ++index) {
        gradient_projection += reference.gradient[index] * direction[index];
        hessian_projection += direction[index] * reference.hvp[index];
    }
    report.first_derivative_error = std::abs(
        gradient_projection - derivative.first_directional)
        / std::max(1.0, std::abs(derivative.first_directional));
    report.second_derivative_error = std::abs(
        hessian_projection - derivative.second_directional)
        / std::max(1.0, std::abs(derivative.second_directional));
    report.derivatives_passed = derivative.finite && derivative.branch_safe
        && report.first_derivative_error <= 2.0e-7
        && report.second_derivative_error <= 2.0e-5;

    std::vector<double> dense_hvp(direction.size(), 0.0);
    for (std::size_t row = 0; row < direction.size(); ++row) {
        for (std::size_t column = 0; column < direction.size(); ++column) {
            dense_hvp[row] += candidate.hessian[row * direction.size() + column]
                * direction[column];
        }
        report.hvp_error = std::max(report.hvp_error,
            std::abs(dense_hvp[row] - candidate.hvp[row])
                / std::max(1.0, std::abs(dense_hvp[row])));
    }
    report.hvp_consistent = report.hvp_error <= 2.0e-4;
    double maximum_hessian = 0.0;
    for (double value : candidate.hessian) maximum_hessian = std::max(
        maximum_hessian, std::abs(value));
    for (std::size_t row = 0; row < direction.size(); ++row) {
        for (std::size_t column = row + 1U; column < direction.size(); ++column) {
            report.symmetry_error = std::max(report.symmetry_error,
                std::abs(candidate.hessian[row * direction.size() + column]
                    - candidate.hessian[column * direction.size() + row]));
        }
    }
    report.symmetry_passed = report.symmetry_error
        <= 2.0e-5 * std::max(1.0, maximum_hessian);
    report.payload_root = sha256_hex(payload_material(candidate));
    report.work_root = sha256_hex(work_material(candidate.work));
    report.cold_exact = true;
    if (run_cold) {
        for (int repeat = 1; repeat < COLD_REPEATS; ++repeat) {
            const AssemblyResult rerun = evaluate_gpu_assembly(
                profile, fixture, AssemblyVariant::Corrected);
            if (sha256_hex(payload_material(rerun)) != report.payload_root
                || sha256_hex(work_material(rerun.work)) != report.work_root) {
                report.cold_exact = false;
                break;
            }
        }
    }
    report.passed = report.graph_exact && report.errors.passed
        && report.derivatives_passed && report.hvp_consistent
        && report.symmetry_passed && report.cold_exact;
    return report;
}

bool admission_controls_pass(const AssemblyProfile& profile) {
    const AssemblyFixture empty{"empty", {}, {}};
    const AssemblyFixture duplicate{"duplicate", {}, {
        sample(7U, {0, 0, 0}, {0, 0, 0}, {0, 0, 0}, {1, 2, 3}),
        sample(7U, {1, 0, 0}, {1, 0, 0}, {1, 0, 0}, {3, 2, 1})}};
    AssemblyFixture excess{"excess", {}, {}};
    for (std::size_t index = 0; index <= ASSEMBLY_MAX_SAMPLES; ++index) {
        excess.samples.push_back(sample(static_cast<std::uint32_t>(1000U + index),
            {static_cast<std::int64_t>(index), 0, 0},
            {static_cast<std::int64_t>(index), 0, 0},
            {static_cast<std::int64_t>(index), 0, 0}, {1, 2, 3}));
    }
    const std::array<std::pair<AssemblyFixture, AssemblyFailure>, 3> controls{{
        {empty, AssemblyFailure::InvalidInput},
        {duplicate, AssemblyFailure::DuplicateSampleId},
        {excess, AssemblyFailure::CapacityExceeded},
    }};
    for (const auto& [fixture, expected] : controls) {
        if (evaluate_reference_assembly(profile, fixture).failure != expected
            || evaluate_gpu_assembly(profile, fixture,
                AssemblyVariant::Corrected).failure != expected) return false;
    }
    return true;
}

const char* variant_name(AssemblyVariant variant) {
    switch (variant) {
    case AssemblyVariant::Corrected: return "corrected";
    case AssemblyVariant::SourceShapedGradient: return "source_shaped_gradient";
    case AssemblyVariant::DirectedEdgeViscosity: return "directed_edge_viscosity";
    case AssemblyVariant::OwnerPressureOnly: return "owner_pressure_only";
    case AssemblyVariant::GaussNewtonPressureOnly: return "gauss_newton_pressure_only";
    case AssemblyVariant::SissmLocalMatrix: return "sissm_local_matrix";
    case AssemblyVariant::CurrentGraphViscosity: return "current_graph_viscosity";
    case AssemblyVariant::NaiveF32Pressure: return "naive_f32_pressure";
    case AssemblyVariant::F64Energy: return "f64_energy";
    case AssemblyVariant::F64PressureOperator: return "f64_pressure_operator";
    }
    return "unknown";
}

int run() {
    const AssemblyProfile profile;
    const std::vector<AssemblyFixture> fixtures = make_fixtures();
    std::vector<AssemblyResult> references;
    std::vector<AssemblyResult> candidates;
    std::vector<CaseReport> reports;
    references.reserve(fixtures.size());
    candidates.reserve(fixtures.size());
    reports.reserve(fixtures.size());
    bool positives_passed = true;
    for (std::size_t index = 0; index < fixtures.size(); ++index) {
        references.push_back(evaluate_reference_assembly(profile, fixtures[index]));
        candidates.push_back(evaluate_gpu_assembly(
            profile, fixtures[index], AssemblyVariant::Corrected));
        reports.push_back(compare_case(profile, fixtures[index], references.back(),
            candidates.back(), true));
        positives_passed = positives_passed && reports.back().passed;
    }
    const bool permutation_exact = reports[6].payload_root == reports[7].payload_root
        && reports[6].work_root == reports[7].work_root;
    positives_passed = positives_passed && permutation_exact;

    struct Control {
        AssemblyVariant variant;
        std::size_t fixture;
    };
    const std::array<Control, 7> controls{{
        {AssemblyVariant::SourceShapedGradient, 5U},
        {AssemblyVariant::DirectedEdgeViscosity, 3U},
        {AssemblyVariant::OwnerPressureOnly, 5U},
        {AssemblyVariant::GaussNewtonPressureOnly, 5U},
        {AssemblyVariant::SissmLocalMatrix, 6U},
        {AssemblyVariant::CurrentGraphViscosity, 2U},
        {AssemblyVariant::NaiveF32Pressure, 5U},
    }};
    std::array<bool, controls.size()> control_rejected{};
    bool negatives_passed = true;
    for (std::size_t index = 0; index < controls.size(); ++index) {
        const AssemblyResult wrong = evaluate_gpu_assembly(profile,
            fixtures[controls[index].fixture], controls[index].variant);
        const CaseReport compared = compare_case(profile,
            fixtures[controls[index].fixture], references[controls[index].fixture],
            wrong, false);
        control_rejected[index] = !compared.passed;
        negatives_passed = negatives_passed && control_rejected[index];
    }
    const bool admission_passed = admission_controls_pass(profile);
    const bool passed = positives_passed && negatives_passed && admission_passed;
    std::string first_failure;
    if (!positives_passed) first_failure = "NCGA2_POSITIVE_CORRESPONDENCE_FAILED";
    else if (!negatives_passed) first_failure = "NCGA2_NEGATIVE_IDENTITY_NOT_REJECTED";
    else if (!admission_passed) first_failure = "NCGA2_ADMISSION_FAILED";

    std::string payload_set = "nextengine.nonlocal.ncga2.payload-set.v1\0";
    std::string work_set = "nextengine.nonlocal.ncga2.work-set.v1\0";
    for (const CaseReport& report : reports) {
        append_u32(payload_set, static_cast<std::uint32_t>(report.payload_root.size()));
        payload_set.append(report.payload_root);
        append_u32(work_set, static_cast<std::uint32_t>(report.work_root.size()));
        work_set.append(report.work_root);
    }

    std::ostringstream output;
    output << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.ncga2.v1\""
           << ",\"status\":\"" << (passed ? "PASS" : "FAIL") << '"'
           << ",\"first_failure\":\"" << first_failure << '"'
           << ",\"fixture_root\":\"" << sha256_hex(fixture_material(fixtures)) << '"'
           << ",\"payload_root\":\"" << sha256_hex(payload_set) << '"'
           << ",\"work_root\":\"" << sha256_hex(work_set) << '"'
           << ",\"permutation_exact\":" << (permutation_exact ? "true" : "false")
           << ",\"cold_repeats\":" << COLD_REPEATS
           << ",\"admission_controls\":" << (admission_passed ? "true" : "false")
           << ",\"cases\":[";
    for (std::size_t index = 0; index < fixtures.size(); ++index) {
        if (index != 0U) output << ',';
        const CaseReport& report = reports[index];
        output << "{\"name\":\"" << fixtures[index].name << "\",\"pass\":"
               << (report.passed ? "true" : "false")
               << ",\"samples\":" << fixtures[index].samples.size()
               << ",\"current_directed\":" << candidates[index].current_neighbors.size()
               << ",\"reference_directed\":" << candidates[index].reference_neighbors.size()
               << ",\"active_pressure\":" << candidates[index].work.active_pressure_centers
               << ",\"max_mixed\":" << report.errors.maximum_mixed
               << ",\"max_zero\":" << report.errors.maximum_zero
               << ",\"density_mixed\":" << report.density_errors.maximum_mixed
               << ",\"energy_mixed\":" << report.energy_errors.maximum_mixed
               << ",\"gradient_mixed\":" << report.gradient_errors.maximum_mixed
               << ",\"hessian_mixed\":" << report.hessian_errors.maximum_mixed
               << ",\"block_mixed\":" << report.block_errors.maximum_mixed
               << ",\"hvp_value_mixed\":" << report.hvp_value_errors.maximum_mixed
               << ",\"hessian_zero\":" << report.hessian_errors.maximum_zero
               << ",\"gradient_max_index\":" << report.gradient_errors.mixed_index
               << ",\"gradient_max_reference\":"
               << report.gradient_errors.mixed_reference
               << ",\"gradient_max_candidate\":"
               << report.gradient_errors.mixed_candidate
               << ",\"hessian_max_index\":" << report.hessian_errors.mixed_index
               << ",\"hessian_max_reference\":"
               << report.hessian_errors.mixed_reference
               << ",\"hessian_max_candidate\":"
               << report.hessian_errors.mixed_candidate
               << ",\"first_derivative_error\":" << report.first_derivative_error
               << ",\"second_derivative_error\":" << report.second_derivative_error
               << ",\"hvp_error\":" << report.hvp_error
               << ",\"symmetry_error\":" << report.symmetry_error
               << ",\"payload_root\":\"" << report.payload_root << '"'
               << ",\"work_root\":\"" << report.work_root << "\"}";
    }
    output << "],\"negative_controls\":[";
    for (std::size_t index = 0; index < controls.size(); ++index) {
        if (index != 0U) output << ',';
        output << "{\"variant\":\"" << variant_name(controls[index].variant)
               << "\",\"fixture\":\"" << fixtures[controls[index].fixture].name
               << "\",\"rejected\":"
               << (control_rejected[index] ? "true" : "false") << '}';
    }
    output << "],\"gpu\":" << gpu_assembly_environment_json()
           << ",\"claim\":\"tiny_objective_assembly_correspondence_only\""
           << ",\"solver_authority\":false,\"performance_authority\":false"
           << ",\"runtime_authority\":false}\n";
    std::cout << output.str();
    return passed ? 0 : 1;
}

} // namespace
} // namespace nextengine::nonlocal::gpu_assembly_audit

int main() {
    try {
        return nextengine::nonlocal::gpu_assembly_audit::run();
    } catch (const std::exception& error) {
        std::cerr << "NCGA2_ERROR:" << error.what() << '\n';
        return 2;
    }
}
