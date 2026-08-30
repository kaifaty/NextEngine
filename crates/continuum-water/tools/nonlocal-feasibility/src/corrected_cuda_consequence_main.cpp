#include "corrected_cuda_assembly.hpp"
#include "corrected_cuda_consequence.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <limits>
#include <numeric>
#include <sstream>
#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

namespace nextengine::nonlocal::gpu_assembly_audit {
namespace {

constexpr std::size_t SAMPLE_COUNT = 100U;
constexpr std::size_t DIMENSION = 3U * SAMPLE_COUNT;
constexpr std::size_t FAILING_INDEX = 51472U;
constexpr std::size_t FAILING_ROW = 171U;
constexpr std::size_t FAILING_COLUMN = 172U;
constexpr double OPERATOR_BAND = 1.0e-3;
constexpr double COSINE_BAND = 1.0e-6;
constexpr double STEP_DRIFT_UM = 5.0;
constexpr double STEP_CAP_METRES = 50.0e-6;
constexpr int SEQUENCE_STEPS = 8;

AssemblySample sample(std::uint32_t id,
    std::array<std::int64_t, 3> reference,
    std::array<std::int64_t, 3> predicted,
    std::array<std::int64_t, 3> current,
    std::array<std::int32_t, 3> direction) {
    return {id, reference, predicted, current, direction};
}

AssemblyFixture make_combined(bool permuted) {
    AssemblyFixture fixture;
    fixture.name = permuted ? "combined_cluster_permuted" : "combined_cluster";
    fixture.terms = {true, true, true, true};
    fixture.samples.reserve(SAMPLE_COUNT);
    for (std::size_t input = 0; input < SAMPLE_COUNT; ++input) {
        const std::size_t logical = (37U * input + 11U) % SAMPLE_COUNT;
        const std::int64_t x = (static_cast<std::int64_t>(logical % 5U) - 2) * 1000;
        const std::int64_t y = (static_cast<std::int64_t>((logical / 5U) % 5U) - 2)
            * 1000;
        const std::int64_t z = (static_cast<std::int64_t>(logical / 25U) - 1) * 1000;
        const std::array<std::int64_t, 3> reference{x, y, z};
        std::array<std::int64_t, 3> predicted = reference;
        std::array<std::int64_t, 3> current = reference;
        predicted[0] += static_cast<std::int64_t>((17U * logical + 3U) % 41U) - 20;
        predicted[1] += static_cast<std::int64_t>((13U * logical + 7U) % 37U) - 18;
        predicted[2] += static_cast<std::int64_t>((19U * logical + 5U) % 43U) - 21;
        current[0] += static_cast<std::int64_t>((29U * logical + 11U) % 101U) - 50;
        current[1] += static_cast<std::int64_t>((31U * logical + 17U) % 97U) - 48;
        current[2] += static_cast<std::int64_t>((23U * logical + 19U) % 89U) - 44;
        const std::array<std::int32_t, 3> direction{
            static_cast<std::int32_t>((43U * logical + 5U) % 1701U) - 850,
            static_cast<std::int32_t>((47U * logical + 9U) % 1601U) - 800,
            static_cast<std::int32_t>((53U * logical + 13U) % 1801U) - 900};
        fixture.samples.push_back(sample(
            static_cast<std::uint32_t>(10000U + 17U * logical), reference,
            predicted, current, direction));
    }
    if (permuted) std::reverse(fixture.samples.begin(), fixture.samples.end());
    return fixture;
}

std::vector<std::size_t> canonical_indices(const AssemblyFixture& fixture) {
    std::vector<std::size_t> result(fixture.samples.size());
    std::iota(result.begin(), result.end(), 0U);
    std::sort(result.begin(), result.end(), [&fixture](std::size_t lhs, std::size_t rhs) {
        return fixture.samples[lhs].sample_id < fixture.samples[rhs].sample_id;
    });
    return result;
}

std::vector<double> frozen_direction(const AssemblyFixture& fixture) {
    const std::vector<std::size_t> order = canonical_indices(fixture);
    std::vector<double> result;
    result.reserve(3U * order.size());
    for (std::size_t index : order) {
        for (std::int32_t value : fixture.samples[index].direction_milli) {
            result.push_back(static_cast<double>(value) / 1000.0);
        }
    }
    return result;
}

void normalize(std::vector<double>& value) {
    long double squared = 0.0L;
    for (double component : value) squared += static_cast<long double>(component) * component;
    const double norm = std::sqrt(static_cast<double>(squared));
    if (!(norm > 0.0) || !std::isfinite(norm)) {
        throw std::runtime_error("NCGA3 zero/nonfinite probe");
    }
    for (double& component : value) component /= norm;
}

std::uint64_t splitmix64(std::uint64_t value) {
    value += UINT64_C(0x9e3779b97f4a7c15);
    value = (value ^ (value >> 30U)) * UINT64_C(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27U)) * UINT64_C(0x94d049bb133111eb);
    return value ^ (value >> 31U);
}

std::vector<std::pair<std::string, std::vector<double>>> make_probes(
    const AssemblyFixture& fixture, const AssemblyResult& reference) {
    std::vector<std::pair<std::string, std::vector<double>>> probes;
    std::vector<double> frozen = frozen_direction(fixture);
    normalize(frozen);
    probes.emplace_back("frozen_direction", std::move(frozen));
    std::vector<double> gradient = reference.gradient;
    for (double& value : gradient) value = -value;
    normalize(gradient);
    probes.emplace_back("negative_gradient", std::move(gradient));
    for (std::size_t coordinate : {FAILING_ROW, FAILING_COLUMN}) {
        std::vector<double> basis(DIMENSION, 0.0);
        basis[coordinate] = 1.0;
        probes.emplace_back("coordinate_" + std::to_string(coordinate), std::move(basis));
    }
    for (std::size_t probe = 0; probe < 6U; ++probe) {
        std::vector<double> value(DIMENSION, 0.0);
        for (std::size_t index = 0; index < DIMENSION; ++index) {
            const std::uint64_t seed = UINT64_C(0x4e43474133000000)
                + probe * DIMENSION + index;
            value[index] = (splitmix64(seed) & UINT64_C(1)) != 0U ? 1.0 : -1.0;
        }
        normalize(value);
        probes.emplace_back("rademacher_" + std::to_string(probe), std::move(value));
    }
    constexpr double pi = 3.141592653589793238462643383279502884;
    for (int period : {7, 11, 17}) {
        std::vector<double> value(DIMENSION, 0.0);
        for (std::size_t scalar = 0; scalar < DIMENSION; ++scalar) {
            const std::size_t particle = scalar / 3U;
            const std::size_t axis = scalar % 3U;
            value[scalar] = std::sin(2.0 * pi * static_cast<double>(particle + 1U)
                    / static_cast<double>(period)
                + static_cast<double>(axis) * pi / 3.0);
        }
        normalize(value);
        probes.emplace_back("smooth_" + std::to_string(period), std::move(value));
    }
    return probes;
}

std::vector<double> matvec(
    const std::vector<double>& matrix, const std::vector<double>& vector) {
    const std::size_t dimension = vector.size();
    if (matrix.size() != dimension * dimension) {
        throw std::runtime_error("NCGA3 matrix/vector size mismatch");
    }
    std::vector<double> result(dimension, 0.0);
    for (std::size_t row = 0; row < dimension; ++row) {
        long double sum = 0.0L;
        for (std::size_t column = 0; column < dimension; ++column) {
            sum += static_cast<long double>(matrix[row * dimension + column])
                * vector[column];
        }
        result[row] = static_cast<double>(sum);
    }
    return result;
}

double l2_norm(const std::vector<double>& value) {
    long double squared = 0.0L;
    for (double component : value) squared += static_cast<long double>(component) * component;
    return std::sqrt(static_cast<double>(squared));
}

double cosine_loss(
    const std::vector<double>& lhs, const std::vector<double>& rhs) {
    long double product = 0.0L;
    for (std::size_t index = 0; index < lhs.size(); ++index) {
        product += static_cast<long double>(lhs[index]) * rhs[index];
    }
    const double denominator = l2_norm(lhs) * l2_norm(rhs);
    if (!(denominator > 0.0)) return std::numeric_limits<double>::infinity();
    return 1.0 - static_cast<double>(product) / denominator;
}

struct ProbeSummary {
    bool finite = true;
    double maximum_relative_l2 = 0.0;
    double maximum_cosine_loss = 0.0;
    double maximum_component_error = 0.0;
    double maximum_component_mixed = 0.0;
    std::string worst_relative_probe;
    std::string worst_component_probe;
};

ProbeSummary compare_probes(const std::vector<double>& reference,
    const std::vector<double>& candidate,
    const std::vector<std::pair<std::string, std::vector<double>>>& probes) {
    ProbeSummary result;
    for (const auto& [name, probe] : probes) {
        const std::vector<double> expected = matvec(reference, probe);
        const std::vector<double> actual = matvec(candidate, probe);
        std::vector<double> delta(expected.size(), 0.0);
        double maximum_component = 0.0;
        double maximum_mixed = 0.0;
        for (std::size_t index = 0; index < expected.size(); ++index) {
            delta[index] = actual[index] - expected[index];
            maximum_component = std::max(maximum_component, std::abs(delta[index]));
            maximum_mixed = std::max(maximum_mixed,
                std::abs(delta[index]) / std::max(1.0, std::abs(expected[index])));
            result.finite = result.finite && std::isfinite(expected[index])
                && std::isfinite(actual[index]);
        }
        const double relative = l2_norm(delta) / std::max(1.0, l2_norm(expected));
        const double cosine = cosine_loss(expected, actual);
        if (relative > result.maximum_relative_l2) {
            result.maximum_relative_l2 = relative;
            result.worst_relative_probe = name;
        }
        if (maximum_component > result.maximum_component_error) {
            result.maximum_component_error = maximum_component;
            result.worst_component_probe = name;
        }
        result.maximum_component_mixed = std::max(
            result.maximum_component_mixed, maximum_mixed);
        result.maximum_cosine_loss = std::max(result.maximum_cosine_loss, cosine);
    }
    return result;
}

struct ElementSummary {
    bool finite = true;
    double maximum_mixed = 0.0;
    std::size_t index = 0U;
    double reference = 0.0;
    double candidate = 0.0;
};

ElementSummary compare_elements(
    const std::vector<double>& reference, const std::vector<double>& candidate) {
    ElementSummary result;
    if (reference.size() != candidate.size()) {
        result.finite = false;
        return result;
    }
    for (std::size_t index = 0; index < reference.size(); ++index) {
        result.finite = result.finite && std::isfinite(reference[index])
            && std::isfinite(candidate[index]);
        const double mixed = std::abs(candidate[index] - reference[index])
            / std::max(1.0, std::abs(reference[index]));
        if (mixed > result.maximum_mixed) {
            result.maximum_mixed = mixed;
            result.index = index;
            result.reference = reference[index];
            result.candidate = candidate[index];
        }
    }
    return result;
}

struct LinearStep {
    bool positive_definite = false;
    bool finite = false;
    double normalized_residual = std::numeric_limits<double>::infinity();
    double antisymmetric_relative_frobenius = 0.0;
    double reference_row_bound = 0.0;
    double regularization = 0.0;
    std::vector<double> value;
};

LinearStep regularized_step(const AssemblyProfile& profile,
    const std::vector<double>& reference_hessian,
    const std::vector<double>& hessian, const std::vector<double>& gradient) {
    LinearStep result;
    const std::size_t dimension = gradient.size();
    if (hessian.size() != dimension * dimension
        || reference_hessian.size() != dimension * dimension) return result;
    for (std::size_t row = 0; row < dimension; ++row) {
        long double row_sum = 0.0L;
        for (std::size_t column = 0; column < dimension; ++column) {
            const double value = 0.5
                * (reference_hessian[row * dimension + column]
                    + reference_hessian[column * dimension + row]);
            row_sum += std::abs(static_cast<long double>(value));
        }
        result.reference_row_bound = std::max(
            result.reference_row_bound, static_cast<double>(row_sum));
    }
    const double inertia = profile.mass / (profile.time_step * profile.time_step);
    result.regularization = result.reference_row_bound + inertia;
    std::vector<double> system(dimension * dimension, 0.0);
    long double anti_squared = 0.0L;
    long double matrix_squared = 0.0L;
    for (std::size_t row = 0; row < dimension; ++row) {
        for (std::size_t column = 0; column < dimension; ++column) {
            const double value = 0.5 * (hessian[row * dimension + column]
                + hessian[column * dimension + row]);
            system[row * dimension + column] = value;
            const double anti = 0.5 * (hessian[row * dimension + column]
                - hessian[column * dimension + row]);
            anti_squared += static_cast<long double>(anti) * anti;
            matrix_squared += static_cast<long double>(value) * value;
        }
    }
    for (std::size_t index = 0; index < dimension; ++index) {
        system[index * dimension + index] += result.regularization;
    }
    result.antisymmetric_relative_frobenius = std::sqrt(static_cast<double>(anti_squared))
        / std::max(1.0, std::sqrt(static_cast<double>(matrix_squared)));

    std::vector<double> lower(dimension * dimension, 0.0);
    for (std::size_t row = 0; row < dimension; ++row) {
        for (std::size_t column = 0; column <= row; ++column) {
            long double value = system[row * dimension + column];
            for (std::size_t inner = 0; inner < column; ++inner) {
                value -= static_cast<long double>(lower[row * dimension + inner])
                    * lower[column * dimension + inner];
            }
            if (row == column) {
                if (!(value > 0.0L) || !std::isfinite(static_cast<double>(value))) {
                    return result;
                }
                lower[row * dimension + column] = std::sqrt(static_cast<double>(value));
            } else {
                lower[row * dimension + column] = static_cast<double>(value)
                    / lower[column * dimension + column];
            }
        }
    }
    result.positive_definite = true;
    std::vector<double> intermediate(dimension, 0.0);
    for (std::size_t row = 0; row < dimension; ++row) {
        long double value = -gradient[row];
        for (std::size_t column = 0; column < row; ++column) {
            value -= static_cast<long double>(lower[row * dimension + column])
                * intermediate[column];
        }
        intermediate[row] = static_cast<double>(value)
            / lower[row * dimension + row];
    }
    result.value.assign(dimension, 0.0);
    for (std::size_t reverse = dimension; reverse-- > 0U;) {
        long double value = intermediate[reverse];
        for (std::size_t row = reverse + 1U; row < dimension; ++row) {
            value -= static_cast<long double>(lower[row * dimension + reverse])
                * result.value[row];
        }
        result.value[reverse] = static_cast<double>(value)
            / lower[reverse * dimension + reverse];
    }
    std::vector<double> residual(dimension, 0.0);
    for (std::size_t row = 0; row < dimension; ++row) {
        long double value = gradient[row];
        for (std::size_t column = 0; column < dimension; ++column) {
            value += static_cast<long double>(system[row * dimension + column])
                * result.value[column];
        }
        residual[row] = static_cast<double>(value);
    }
    result.normalized_residual = l2_norm(residual)
        / std::max(1.0, l2_norm(gradient));
    result.finite = std::all_of(result.value.begin(), result.value.end(),
        [](double value) { return std::isfinite(value); })
        && std::isfinite(result.normalized_residual);
    return result;
}

double objective(const AssemblyResult& result) {
    return std::accumulate(result.energy.begin(), result.energy.end(), 0.0);
}

AssemblyFixture apply_step(const AssemblyFixture& fixture,
    const std::vector<double>& step, double& scale, bool& changed) {
    AssemblyFixture result = fixture;
    const std::vector<std::size_t> order = canonical_indices(result);
    double maximum = 0.0;
    for (std::size_t particle = 0; particle < order.size(); ++particle) {
        double squared = 0.0;
        for (std::size_t axis = 0; axis < 3U; ++axis) {
            const double value = step[3U * particle + axis];
            squared += value * value;
        }
        maximum = std::max(maximum, std::sqrt(squared));
    }
    scale = maximum > STEP_CAP_METRES ? STEP_CAP_METRES / maximum : 1.0;
    changed = false;
    for (std::size_t particle = 0; particle < order.size(); ++particle) {
        for (std::size_t axis = 0; axis < 3U; ++axis) {
            const std::int64_t delta = static_cast<std::int64_t>(std::llround(
                step[3U * particle + axis] * scale * 1.0e6));
            result.samples[order[particle]].current_um[axis] += delta;
            changed = changed || delta != 0;
        }
    }
    return result;
}

bool same_topology(const AssemblyResult& lhs, const AssemblyResult& rhs) {
    return lhs.owner_ids == rhs.owner_ids
        && lhs.current_offsets == rhs.current_offsets
        && lhs.current_neighbors == rhs.current_neighbors
        && lhs.reference_offsets == rhs.reference_offsets
        && lhs.reference_neighbors == rhs.reference_neighbors
        && lhs.pressure_active == rhs.pressure_active;
}

enum class StepModel {
    Reference,
    StrictF32,
    MixedReduction,
    MixedProducts,
};

struct ModelEvaluation {
    bool valid = false;
    AssemblyResult truth;
    std::vector<double> hessian;
    std::vector<double> gradient;
};

ModelEvaluation evaluate_model(const AssemblyProfile& profile,
    const AssemblyFixture& fixture, StepModel model) {
    ModelEvaluation result;
    result.truth = evaluate_reference_assembly(profile, fixture);
    if (result.truth.failure != AssemblyFailure::None) return result;
    if (model == StepModel::Reference) {
        result.hessian = result.truth.hessian;
        result.gradient = result.truth.gradient;
        result.valid = true;
        return result;
    }
    const AssemblyResult strict = evaluate_gpu_assembly(
        profile, fixture, AssemblyVariant::Corrected);
    if (strict.failure != AssemblyFailure::None || !same_topology(result.truth, strict)) {
        return result;
    }
    result.gradient = strict.gradient;
    if (model == StepModel::StrictF32) {
        result.hessian = strict.hessian;
        result.valid = true;
        return result;
    }
    const PressureArithmetic arithmetic = model == StepModel::MixedReduction
        ? PressureArithmetic::F32ProductsF64Reduction
        : PressureArithmetic::F64PressureProducts;
    const MixedPressureResult mixed = evaluate_gpu_mixed_pressure_hessian(
        profile, fixture, arithmetic);
    if (mixed.failure != AssemblyFailure::None
        || mixed.pressure_active != strict.pressure_active) return result;
    result.hessian = mixed.hessian;
    result.valid = true;
    return result;
}

struct OneStepSummary {
    bool valid = false;
    bool descent = false;
    bool topology_unchanged = false;
    bool changed = false;
    double before_energy = 0.0;
    double after_energy = 0.0;
    double applied_scale = 0.0;
    LinearStep linear;
    AssemblyFixture next;
};

OneStepSummary take_one_step(const AssemblyProfile& profile,
    const AssemblyFixture& fixture, const AssemblyResult& initial_topology,
    StepModel model) {
    OneStepSummary result;
    const ModelEvaluation evaluated = evaluate_model(profile, fixture, model);
    if (!evaluated.valid) return result;
    result.before_energy = objective(evaluated.truth);
    result.linear = regularized_step(profile, evaluated.truth.hessian,
        evaluated.hessian, evaluated.gradient);
    if (!result.linear.positive_definite || !result.linear.finite
        || result.linear.normalized_residual > 1.0e-10) return result;
    result.next = apply_step(fixture, result.linear.value,
        result.applied_scale, result.changed);
    const AssemblyResult after = evaluate_reference_assembly(profile, result.next);
    if (after.failure != AssemblyFailure::None) return result;
    result.after_energy = objective(after);
    result.descent = result.changed && result.after_energy < result.before_energy;
    result.topology_unchanged = same_topology(initial_topology, after);
    result.valid = result.descent && result.topology_unchanged;
    return result;
}

struct SequenceSummary {
    bool valid = false;
    bool all_descent = true;
    bool topology_unchanged = true;
    double initial_energy = 0.0;
    double final_energy = 0.0;
    double maximum_residual = 0.0;
    double maximum_antisymmetric = 0.0;
    AssemblyFixture final_fixture;
};

SequenceSummary run_sequence(const AssemblyProfile& profile,
    const AssemblyFixture& initial, StepModel model) {
    SequenceSummary result;
    const AssemblyResult initial_truth = evaluate_reference_assembly(profile, initial);
    if (initial_truth.failure != AssemblyFailure::None) return result;
    result.initial_energy = objective(initial_truth);
    AssemblyFixture current = initial;
    for (int iteration = 0; iteration < SEQUENCE_STEPS; ++iteration) {
        const OneStepSummary step = take_one_step(
            profile, current, initial_truth, model);
        result.all_descent = result.all_descent && step.descent;
        result.topology_unchanged = result.topology_unchanged
            && step.topology_unchanged;
        result.maximum_residual = std::max(
            result.maximum_residual, step.linear.normalized_residual);
        result.maximum_antisymmetric = std::max(result.maximum_antisymmetric,
            step.linear.antisymmetric_relative_frobenius);
        if (!step.valid) return result;
        current = step.next;
    }
    result.final_fixture = current;
    const AssemblyResult final_truth = evaluate_reference_assembly(profile, current);
    if (final_truth.failure != AssemblyFailure::None) return result;
    result.final_energy = objective(final_truth);
    result.valid = result.all_descent && result.topology_unchanged
        && result.final_energy < result.initial_energy;
    return result;
}

double maximum_position_drift_um(
    const AssemblyFixture& lhs, const AssemblyFixture& rhs) {
    const std::vector<std::size_t> left_order = canonical_indices(lhs);
    const std::vector<std::size_t> right_order = canonical_indices(rhs);
    double maximum = 0.0;
    for (std::size_t particle = 0; particle < left_order.size(); ++particle) {
        long double squared = 0.0L;
        for (std::size_t axis = 0; axis < 3U; ++axis) {
            const long double delta = static_cast<long double>(
                lhs.samples[left_order[particle]].current_um[axis]
                - rhs.samples[right_order[particle]].current_um[axis]);
            squared += delta * delta;
        }
        maximum = std::max(maximum, std::sqrt(static_cast<double>(squared)));
    }
    return maximum;
}

struct StepComparison {
    double relative_l2 = std::numeric_limits<double>::infinity();
    double cosine = std::numeric_limits<double>::infinity();
    double maximum_particle_difference_um = std::numeric_limits<double>::infinity();
};

StepComparison compare_steps(
    const std::vector<double>& reference, const std::vector<double>& candidate) {
    StepComparison result;
    std::vector<double> delta(reference.size(), 0.0);
    for (std::size_t index = 0; index < reference.size(); ++index) {
        delta[index] = candidate[index] - reference[index];
    }
    result.relative_l2 = l2_norm(delta) / std::max(1.0e-12, l2_norm(reference));
    result.cosine = cosine_loss(reference, candidate);
    result.maximum_particle_difference_um = 0.0;
    for (std::size_t particle = 0; particle < reference.size() / 3U; ++particle) {
        double squared = 0.0;
        for (std::size_t axis = 0; axis < 3U; ++axis) {
            const double value = delta[3U * particle + axis] * 1.0e6;
            squared += value * value;
        }
        result.maximum_particle_difference_um = std::max(
            result.maximum_particle_difference_um, std::sqrt(squared));
    }
    return result;
}

void append_u64(std::string& bytes, std::uint64_t value) {
    for (unsigned int shift = 0; shift < 64U; shift += 8U) {
        bytes.push_back(static_cast<char>((value >> shift) & 0xffU));
    }
}

void append_double(std::string& bytes, double value) {
    std::uint64_t bits = 0U;
    static_assert(sizeof(bits) == sizeof(value), "NCGA3 binary64 layout");
    std::memcpy(&bits, &value, sizeof(bits));
    append_u64(bytes, bits);
}

std::string matrix_root(const std::vector<double>& matrix) {
    std::string bytes = "nextengine.nonlocal.ncga3.matrix.v1\0";
    append_u64(bytes, matrix.size());
    for (double value : matrix) append_double(bytes, value);
    return sha256_hex(bytes);
}

std::string work_root(const PressureArithmeticWork& work) {
    std::string bytes = "nextengine.nonlocal.ncga3.work.v1\0";
    append_u64(bytes, work.density_terms);
    append_u64(bytes, work.jacobian_terms);
    append_u64(bytes, work.pressure_outer_products);
    append_u64(bytes, work.pressure_geometric_products);
    append_u64(bytes, work.f32_completed_products);
    append_u64(bytes, work.f64_completed_products);
    append_u64(bytes, work.f64_accumulations);
    append_u64(bytes, work.output_rounds_to_f32);
    return sha256_hex(bytes);
}

bool same_work(
    const PressureArithmeticWork& lhs, const PressureArithmeticWork& rhs) {
    return work_root(lhs) == work_root(rhs);
}

int run() {
    const AssemblyProfile profile;
    const AssemblyFixture fixture = make_combined(false);
    const AssemblyFixture permuted = make_combined(true);
    const AssemblyResult reference = evaluate_reference_assembly(profile, fixture);
    const AssemblyResult strict = evaluate_gpu_assembly(
        profile, fixture, AssemblyVariant::Corrected);
    const AssemblyResult strict_permuted = evaluate_gpu_assembly(
        profile, permuted, AssemblyVariant::Corrected);
    if (reference.failure != AssemblyFailure::None
        || strict.failure != AssemblyFailure::None
        || !same_topology(reference, strict)) {
        throw std::runtime_error("NCGA3 baseline admission/topology failed");
    }
    const bool ncga2_reproduced = strict.hessian.size() > FAILING_INDEX
        && reference.hessian.size() > FAILING_INDEX
        && strict.hessian[FAILING_INDEX] == 20.029237747192383
        && std::abs(reference.hessian[FAILING_INDEX] - 20.023981996858438) <= 1.0e-14;

    const MixedPressureResult reduction = evaluate_gpu_mixed_pressure_hessian(
        profile, fixture, PressureArithmetic::F32ProductsF64Reduction);
    const MixedPressureResult products = evaluate_gpu_mixed_pressure_hessian(
        profile, fixture, PressureArithmetic::F64PressureProducts);
    const MixedPressureResult reduction_permuted = evaluate_gpu_mixed_pressure_hessian(
        profile, permuted, PressureArithmetic::F32ProductsF64Reduction);
    const MixedPressureResult products_permuted = evaluate_gpu_mixed_pressure_hessian(
        profile, permuted, PressureArithmetic::F64PressureProducts);
    const bool mixed_valid = reduction.failure == AssemblyFailure::None
        && products.failure == AssemblyFailure::None
        && reduction.pressure_active == strict.pressure_active
        && products.pressure_active == strict.pressure_active;
    const bool permutation_exact = strict.hessian == strict_permuted.hessian
        && reduction.hessian == reduction_permuted.hessian
        && products.hessian == products_permuted.hessian
        && same_work(reduction.work, reduction_permuted.work)
        && same_work(products.work, products_permuted.work);

    const auto probes = make_probes(fixture, reference);
    const ElementSummary strict_elements = compare_elements(
        reference.hessian, strict.hessian);
    const ElementSummary reduction_elements = compare_elements(
        reference.hessian, reduction.hessian);
    const ElementSummary product_elements = compare_elements(
        reference.hessian, products.hessian);
    const ProbeSummary strict_probes = compare_probes(
        reference.hessian, strict.hessian, probes);
    const ProbeSummary reduction_probes = compare_probes(
        reference.hessian, reduction.hessian, probes);
    const ProbeSummary product_probes = compare_probes(
        reference.hessian, products.hessian, probes);

    const AssemblyResult initial_topology = reference;
    const OneStepSummary reference_step = take_one_step(
        profile, fixture, initial_topology, StepModel::Reference);
    const OneStepSummary strict_step = take_one_step(
        profile, fixture, initial_topology, StepModel::StrictF32);
    const OneStepSummary reduction_step = take_one_step(
        profile, fixture, initial_topology, StepModel::MixedReduction);
    const OneStepSummary product_step = take_one_step(
        profile, fixture, initial_topology, StepModel::MixedProducts);
    const StepComparison strict_step_comparison = compare_steps(
        reference_step.linear.value, strict_step.linear.value);
    const StepComparison reduction_step_comparison = compare_steps(
        reference_step.linear.value, reduction_step.linear.value);
    const StepComparison product_step_comparison = compare_steps(
        reference_step.linear.value, product_step.linear.value);

    const bool strict_operator_pass = strict_probes.finite
        && strict_probes.maximum_relative_l2 <= OPERATOR_BAND
        && strict_probes.maximum_cosine_loss <= COSINE_BAND;
    const bool strict_step_pass = reference_step.valid && strict_step.valid
        && strict_step_comparison.relative_l2 <= OPERATOR_BAND
        && strict_step_comparison.cosine <= COSINE_BAND
        && strict_step_comparison.maximum_particle_difference_um <= STEP_DRIFT_UM;

    const SequenceSummary reference_sequence = run_sequence(
        profile, fixture, StepModel::Reference);
    const SequenceSummary strict_sequence = run_sequence(
        profile, fixture, StepModel::StrictF32);
    double strict_sequence_drift = std::numeric_limits<double>::infinity();
    double strict_sequence_energy_error = std::numeric_limits<double>::infinity();
    if (reference_sequence.valid && strict_sequence.valid) {
        strict_sequence_drift = maximum_position_drift_um(
            reference_sequence.final_fixture, strict_sequence.final_fixture);
        strict_sequence_energy_error = std::abs(strict_sequence.final_energy
                - reference_sequence.final_energy)
            / std::max(1.0, std::abs(reference_sequence.final_energy));
    }
    const bool strict_sequence_pass = reference_sequence.valid
        && strict_sequence.valid && strict_sequence_drift <= STEP_DRIFT_UM
        && strict_sequence_energy_error <= OPERATOR_BAND;

    SequenceSummary reduction_sequence;
    SequenceSummary product_sequence;
    double reduction_sequence_drift = 0.0;
    double product_sequence_drift = 0.0;
    bool reduction_sequence_pass = true;
    bool product_sequence_pass = true;
    if (!(strict_operator_pass && strict_step_pass && strict_sequence_pass)) {
        reduction_sequence = run_sequence(profile, fixture, StepModel::MixedReduction);
        product_sequence = run_sequence(profile, fixture, StepModel::MixedProducts);
        reduction_sequence_drift = reduction_sequence.valid
            ? maximum_position_drift_um(reference_sequence.final_fixture,
                reduction_sequence.final_fixture)
            : std::numeric_limits<double>::infinity();
        product_sequence_drift = product_sequence.valid
            ? maximum_position_drift_um(reference_sequence.final_fixture,
                product_sequence.final_fixture)
            : std::numeric_limits<double>::infinity();
        reduction_sequence_pass = reduction_sequence.valid
            && reduction_sequence_drift <= STEP_DRIFT_UM;
        product_sequence_pass = product_sequence.valid
            && product_sequence_drift <= STEP_DRIFT_UM;
    }

    std::vector<double> biased = strict.hessian;
    const double bias = 0.01 * reference.hessian[FAILING_INDEX];
    biased[FAILING_ROW * DIMENSION + FAILING_COLUMN] += bias;
    biased[FAILING_COLUMN * DIMENSION + FAILING_ROW] += bias;
    const ProbeSummary bias_control = compare_probes(
        reference.hessian, biased, probes);
    const bool bias_rejected = bias_control.maximum_component_mixed > OPERATOR_BAND;
    std::vector<double> expected_frozen = matvec(reference.hessian, probes.front().second);
    std::vector<double> flipped_frozen = expected_frozen;
    for (double& value : flipped_frozen) value = -value;
    const bool sign_flip_rejected = cosine_loss(expected_frozen, flipped_frozen)
        > COSINE_BAND;

    const bool apparatus_valid = ncga2_reproduced && mixed_valid
        && permutation_exact && bias_rejected && sign_flip_rejected
        && reference_step.valid && reduction_step.valid && product_step.valid;
    const bool strict_negligible = apparatus_valid && strict_operator_pass
        && strict_step_pass && strict_sequence_pass;
    const bool mixed_products_pass = product_elements.maximum_mixed <= 2.0e-4
        && product_probes.maximum_relative_l2 <= OPERATOR_BAND
        && product_probes.maximum_cosine_loss <= COSINE_BAND
        && product_step_comparison.relative_l2 <= OPERATOR_BAND
        && product_step_comparison.cosine <= COSINE_BAND
        && product_step_comparison.maximum_particle_difference_um <= STEP_DRIFT_UM
        && product_sequence_pass;
    std::string result = "INCONCLUSIVE";
    if (strict_negligible) result = "F32_CONSEQUENCE_NEGLIGIBLE_BOUNDED";
    else if (apparatus_valid && mixed_products_pass) {
        result = "MIXED_PRESSURE_REQUIRED_BOUNDED";
    } else if (apparatus_valid) {
        result = "F32_CONSEQUENCE_MATERIAL";
    }

    std::ostringstream output;
    output << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.ncga3.v1\""
           << ",\"result\":\"" << result << '"'
           << ",\"ncga2_reproduced\":" << (ncga2_reproduced ? "true" : "false")
           << ",\"permutation_exact\":" << (permutation_exact ? "true" : "false")
           << ",\"controls\":{\"bias_rejected\":"
           << (bias_rejected ? "true" : "false")
           << ",\"sign_flip_rejected\":"
           << (sign_flip_rejected ? "true" : "false") << "}"
           << ",\"strict_f32\":{\"element_mixed\":"
           << strict_elements.maximum_mixed << ",\"element_index\":"
           << strict_elements.index << ",\"probe_relative_l2\":"
           << strict_probes.maximum_relative_l2 << ",\"probe_cosine_loss\":"
           << strict_probes.maximum_cosine_loss << ",\"probe_component_mixed\":"
           << strict_probes.maximum_component_mixed << ",\"worst_probe\":\""
           << strict_probes.worst_relative_probe << "\",\"step_relative_l2\":"
           << strict_step_comparison.relative_l2 << ",\"step_cosine_loss\":"
           << strict_step_comparison.cosine << ",\"step_max_um\":"
           << strict_step_comparison.maximum_particle_difference_um
           << ",\"step_residual\":" << strict_step.linear.normalized_residual
           << ",\"antisymmetric_frobenius\":"
           << strict_step.linear.antisymmetric_relative_frobenius
           << ",\"reference_row_bound\":"
           << strict_step.linear.reference_row_bound
           << ",\"regularization\":" << strict_step.linear.regularization
           << ",\"step_descent\":" << (strict_step.descent ? "true" : "false")
           << ",\"sequence_pass\":" << (strict_sequence_pass ? "true" : "false")
           << ",\"sequence_drift_um\":" << strict_sequence_drift
           << ",\"sequence_energy_error\":" << strict_sequence_energy_error
           << ",\"matrix_root\":\"" << matrix_root(strict.hessian) << "\"}"
           << ",\"f32_products_f64_reduction\":{\"element_mixed\":"
           << reduction_elements.maximum_mixed << ",\"element_index\":"
           << reduction_elements.index << ",\"probe_relative_l2\":"
           << reduction_probes.maximum_relative_l2 << ",\"probe_cosine_loss\":"
           << reduction_probes.maximum_cosine_loss << ",\"step_relative_l2\":"
           << reduction_step_comparison.relative_l2 << ",\"step_max_um\":"
           << reduction_step_comparison.maximum_particle_difference_um
           << ",\"sequence_pass\":"
           << (reduction_sequence_pass ? "true" : "false")
           << ",\"sequence_drift_um\":" << reduction_sequence_drift
           << ",\"matrix_root\":\"" << matrix_root(reduction.hessian)
           << "\",\"work_root\":\"" << work_root(reduction.work)
           << "\",\"work\":{\"density_terms\":" << reduction.work.density_terms
           << ",\"jacobian_terms\":" << reduction.work.jacobian_terms
           << ",\"outer_products\":" << reduction.work.pressure_outer_products
           << ",\"geometric_products\":"
           << reduction.work.pressure_geometric_products
           << ",\"f32_products\":" << reduction.work.f32_completed_products
           << ",\"f64_products\":" << reduction.work.f64_completed_products
           << ",\"f64_accumulations\":" << reduction.work.f64_accumulations
           << ",\"f32_output_rounds\":" << reduction.work.output_rounds_to_f32
           << "}}"
           << ",\"f64_pressure_products\":{\"element_mixed\":"
           << product_elements.maximum_mixed << ",\"element_index\":"
           << product_elements.index << ",\"probe_relative_l2\":"
           << product_probes.maximum_relative_l2 << ",\"probe_cosine_loss\":"
           << product_probes.maximum_cosine_loss << ",\"step_relative_l2\":"
           << product_step_comparison.relative_l2 << ",\"step_max_um\":"
           << product_step_comparison.maximum_particle_difference_um
           << ",\"sequence_pass\":"
           << (product_sequence_pass ? "true" : "false")
           << ",\"sequence_drift_um\":" << product_sequence_drift
           << ",\"matrix_root\":\"" << matrix_root(products.hessian)
           << "\",\"work_root\":\"" << work_root(products.work)
           << "\",\"work\":{\"density_terms\":" << products.work.density_terms
           << ",\"jacobian_terms\":" << products.work.jacobian_terms
           << ",\"outer_products\":" << products.work.pressure_outer_products
           << ",\"geometric_products\":"
           << products.work.pressure_geometric_products
           << ",\"f32_products\":" << products.work.f32_completed_products
           << ",\"f64_products\":" << products.work.f64_completed_products
           << ",\"f64_accumulations\":" << products.work.f64_accumulations
           << ",\"f32_output_rounds\":" << products.work.output_rounds_to_f32
           << "}}"
           << ",\"reference\":{\"step_residual\":"
           << reference_step.linear.normalized_residual
           << ",\"antisymmetric_frobenius\":"
           << reference_step.linear.antisymmetric_relative_frobenius
           << ",\"initial_energy\":" << reference_sequence.initial_energy
           << ",\"final_energy\":" << reference_sequence.final_energy << "}"
           << ",\"gpu\":" << gpu_assembly_environment_json()
           << ",\"claim\":\"tiny_local_numerical_consequence_only\""
           << ",\"solver_authority\":false,\"performance_authority\":false"
           << ",\"runtime_authority\":false}\n";
    std::cout << output.str();
    return result == "INCONCLUSIVE" ? 2
        : (result == "F32_CONSEQUENCE_MATERIAL" ? 1 : 0);
}

} // namespace
} // namespace nextengine::nonlocal::gpu_assembly_audit

int main() {
    try {
        return nextengine::nonlocal::gpu_assembly_audit::run();
    } catch (const std::exception& error) {
        std::cerr << "NCGA3_ERROR:" << error.what() << '\n';
        return 3;
    }
}
