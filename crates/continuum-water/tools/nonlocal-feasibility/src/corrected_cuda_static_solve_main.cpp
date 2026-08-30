#include "corrected_cuda_assembly.hpp"
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

constexpr int MAXIMUM_OUTER_TRIALS = 64;
constexpr double ACCEPT_RATIO = 0.1;
constexpr double RAW_GRADIENT_LIMIT = 1.0e-10;
constexpr double POSITION_LIMIT_UM = 5.0;
constexpr double OBJECTIVE_LIMIT = 1.0e-6;
constexpr double SUPPORTED_SCALED_RESIDUAL = 1.0e-7;
constexpr double FLOOR_SCALED_RESIDUAL = 1.0e-5;

struct StaticCase {
    std::string name;
    AssemblyProfile profile;
    AssemblyFixture fixture;
    AssemblyFixture permuted_fixture;
    AssemblyContinuousState initial;
    int expected_outer = 0;
    std::uint64_t expected_evaluations = 0;
    std::uint64_t expected_hvp = 0;
};

double cubic_weight_binary64(double radius, double horizon) {
    constexpr double pi = 3.141592653589793238462643383279502884;
    const double q = 2.0 * radius / horizon;
    const double alpha = 3.0 / (2.0 * pi * horizon * horizon * horizon);
    if (q > 2.0) return 0.0;
    if (q >= 1.0) {
        const double delta = 2.0 - q;
        return alpha * delta * delta * delta / 6.0;
    }
    return alpha * (2.0 / 3.0 - q * q + 0.5 * q * q * q);
}

std::int64_t to_um(double value) {
    return static_cast<std::int64_t>(std::llround(1.0e6 * value));
}

AssemblySample make_sample(std::uint32_t id, const std::array<double, 3>& reference,
    const std::array<double, 3>& predicted) {
    AssemblySample result;
    result.sample_id = id;
    for (std::size_t axis = 0; axis < 3U; ++axis) {
        result.reference_um[axis] = to_um(reference[axis]);
        result.predicted_um[axis] = to_um(predicted[axis]);
        result.current_um[axis] = result.predicted_um[axis];
    }
    return result;
}

void append_position(
    std::vector<double>& values, const std::array<double, 3>& position) {
    values.insert(values.end(), position.begin(), position.end());
}

StaticCase compressed_case() {
    StaticCase result;
    result.name = "compressed_pair";
    result.profile.kappa = 500.0;
    const std::array<double, 3> first{-0.025, 0.0, 0.0};
    const std::array<double, 3> second{0.025, 0.0, 0.0};
    const double density = result.profile.mass
        * (cubic_weight_binary64(0.0, result.profile.horizon)
            + cubic_weight_binary64(0.05, result.profile.horizon));
    result.profile.rest_density = density / 1.1;
    result.fixture.name = result.name;
    result.fixture.terms = {true, true, true, true};
    result.fixture.samples = {
        make_sample(101U, first, first), make_sample(202U, second, second)};
    result.permuted_fixture = result.fixture;
    result.permuted_fixture.name += "_permuted";
    std::reverse(result.permuted_fixture.samples.begin(),
        result.permuted_fixture.samples.end());
    append_position(result.initial.reference_m, first);
    append_position(result.initial.reference_m, second);
    result.initial.predicted_m = result.initial.reference_m;
    result.initial.current_m = result.initial.predicted_m;
    result.expected_outer = 4;
    result.expected_evaluations = 5U;
    result.expected_hvp = 8U;
    return result;
}

StaticCase combined_case() {
    StaticCase result;
    result.name = "combined_tetrahedron";
    result.profile.kappa = 200.0;
    result.profile.lambda = 20.0;
    result.profile.mu = 10.0;
    result.profile.gamma = 100.0;
    const std::array<std::array<double, 3>, 4> reference{{
        {{0.0, 0.0, 0.0}},
        {{0.04, 0.0, 0.0}},
        {{0.02, 0.0346410161513775, 0.0}},
        {{0.02, 0.0115470053837925, 0.0326598632371090}},
    }};
    const std::array<std::array<double, 3>, 4> velocity{{
        {{0.4, -0.2, 0.1}},
        {{-0.1, 0.3, -0.2}},
        {{0.2, 0.1, 0.3}},
        {{-0.3, -0.2, -0.2}},
    }};
    double density = result.profile.mass
        * cubic_weight_binary64(0.0, result.profile.horizon);
    for (std::size_t index = 1U; index < reference.size(); ++index) {
        double squared = 0.0;
        for (std::size_t axis = 0; axis < 3U; ++axis) {
            const double delta = reference[0][axis] - reference[index][axis];
            squared += delta * delta;
        }
        density += result.profile.mass * cubic_weight_binary64(
            std::sqrt(squared), result.profile.horizon);
    }
    result.profile.rest_density = density / 1.1;
    result.fixture.name = result.name;
    result.fixture.terms = {true, true, true, true};
    for (std::size_t index = 0; index < reference.size(); ++index) {
        std::array<double, 3> predicted{};
        for (std::size_t axis = 0; axis < 3U; ++axis) {
            predicted[axis] = reference[index][axis]
                + result.profile.time_step * velocity[index][axis];
        }
        result.fixture.samples.push_back(make_sample(
            static_cast<std::uint32_t>(100U + 17U * index),
            reference[index], predicted));
        append_position(result.initial.reference_m, reference[index]);
        append_position(result.initial.predicted_m, predicted);
    }
    result.initial.current_m = result.initial.predicted_m;
    result.permuted_fixture = result.fixture;
    result.permuted_fixture.name += "_permuted";
    std::reverse(result.permuted_fixture.samples.begin(),
        result.permuted_fixture.samples.end());
    result.expected_outer = 13;
    result.expected_evaluations = 14U;
    result.expected_hvp = 33U;
    return result;
}

void append_u64(std::string& bytes, std::uint64_t value) {
    for (unsigned int shift = 0U; shift < 64U; shift += 8U) {
        bytes.push_back(static_cast<char>((value >> shift) & 0xffU));
    }
}

void append_double(std::string& bytes, double value) {
    std::uint64_t bits = 0U;
    static_assert(sizeof(bits) == sizeof(value), "NCGA5 binary64 layout");
    std::memcpy(&bits, &value, sizeof(bits));
    append_u64(bytes, bits);
}

std::string vector_root(const std::string& domain,
    const std::vector<double>& values) {
    std::string bytes = domain;
    append_u64(bytes, values.size());
    for (double value : values) append_double(bytes, value);
    return sha256_hex(bytes);
}

double vector_dot(const std::vector<double>& lhs,
    const std::vector<double>& rhs) {
    if (lhs.size() != rhs.size()) throw std::runtime_error("NCGA5 dot size");
    double result = 0.0;
    for (std::size_t index = 0; index < lhs.size(); ++index) {
        result += lhs[index] * rhs[index];
    }
    return result;
}

double vector_norm(const std::vector<double>& values) {
    return std::sqrt(std::max(vector_dot(values, values), 0.0));
}

bool finite_vector(const std::vector<double>& values) {
    return std::all_of(values.begin(), values.end(),
        [](double value) { return std::isfinite(value); });
}

std::vector<double> matvec(const std::vector<double>& matrix,
    const std::vector<double>& values) {
    if (matrix.size() != values.size() * values.size()) {
        throw std::runtime_error("NCGA5 matvec size");
    }
    std::vector<double> result(values.size(), 0.0);
    for (std::size_t row = 0; row < values.size(); ++row) {
        for (std::size_t column = 0; column < values.size(); ++column) {
            result[row] += matrix[row * values.size() + column] * values[column];
        }
    }
    return result;
}

double total_energy(const AssemblyResult& result) {
    return std::accumulate(result.energy.begin(), result.energy.end(), 0.0);
}

std::string active_root(const std::vector<std::uint8_t>& active) {
    std::string bytes = "nextengine.nonlocal.ncga5.active.v1\0";
    append_u64(bytes, active.size());
    bytes.append(reinterpret_cast<const char*>(active.data()), active.size());
    return sha256_hex(bytes);
}

void add_assembly_work(AssemblyWorkReceipt& target,
    const AssemblyWorkReceipt& value) {
    target.owner_sort_comparisons += value.owner_sort_comparisons;
    target.graph_distance_predicates += value.graph_distance_predicates;
    target.density_kernel_evaluations += value.density_kernel_evaluations;
    target.energy_pair_visits += value.energy_pair_visits;
    target.gradient_pair_visits += value.gradient_pair_visits;
    target.active_pressure_centers += value.active_pressure_centers;
    target.pressure_outer_products += value.pressure_outer_products;
    target.pressure_geometric_blocks += value.pressure_geometric_blocks;
    target.viscosity_curvature_blocks += value.viscosity_curvature_blocks;
    target.surface_curvature_blocks += value.surface_curvature_blocks;
    target.dense_entries_written += value.dense_entries_written;
    target.hvp_products += value.hvp_products;
    target.host_device_scalar_transfers += value.host_device_scalar_transfers;
    target.compensated_additions += value.compensated_additions;
    target.compensation_initializations += value.compensation_initializations;
    target.f64_energy_density_terms += value.f64_energy_density_terms;
    target.f64_energy_particle_terms += value.f64_energy_particle_terms;
    target.f64_energy_viscosity_pairs += value.f64_energy_viscosity_pairs;
    target.f64_energy_surface_pairs += value.f64_energy_surface_pairs;
    target.f64_energy_component_writes += value.f64_energy_component_writes;
}

struct ControllerWork {
    std::uint64_t evaluator_calls = 0;
    std::uint64_t inner_hvp = 0;
    std::uint64_t prediction_hvp = 0;
    std::uint64_t dot_products = 0;
    std::uint64_t boundary_intersections = 0;
    std::uint64_t accepted_trials = 0;
    std::uint64_t rejected_trials = 0;
    std::uint64_t radius_shrinks = 0;
    std::uint64_t radius_expands = 0;
    std::uint64_t state_scalar_uploads = 0;
    std::uint64_t state_scalar_rounds = 0;
    std::uint64_t negative_curvature_stops = 0;
    std::uint64_t boundary_stops = 0;
    std::uint64_t residual_stops = 0;
    std::uint64_t dimension_stops = 0;
    AssemblyWorkReceipt assembly;
};

struct InnerStep {
    std::vector<double> value;
    std::string reason;
    int iterations = 0;
    int hvp = 0;
    bool boundary = false;
    bool finite = true;
};

double positive_boundary_intersection(const std::vector<double>& point,
    const std::vector<double>& direction, double radius,
    ControllerWork& work) {
    ++work.boundary_intersections;
    const double a = vector_dot(direction, direction);
    const double b = 2.0 * vector_dot(point, direction);
    const double c = vector_dot(point, point) - radius * radius;
    work.dot_products += 3U;
    if (!(a > 0.0) || !std::isfinite(a) || !std::isfinite(b)
        || !std::isfinite(c)) return std::numeric_limits<double>::quiet_NaN();
    const double discriminant = std::max(b * b - 4.0 * a * c, 0.0);
    return (-b + std::sqrt(discriminant)) / (2.0 * a);
}

InnerStep truncated_cg(const AssemblyResult& evaluation, double radius,
    bool& invert_first_hvp, ControllerWork& work) {
    InnerStep result;
    result.value.assign(evaluation.gradient.size(), 0.0);
    std::vector<double> residual = evaluation.gradient;
    std::vector<double> direction(residual.size());
    std::transform(residual.begin(), residual.end(), direction.begin(),
        [](double value) { return -value; });
    double residual_squared = vector_dot(residual, residual);
    ++work.dot_products;
    const double initial_norm = std::sqrt(residual_squared);
    const double forcing = std::min(0.5, std::sqrt(initial_norm));
    const int maximum_iterations = static_cast<int>(3U * residual.size());
    for (int iteration = 0; iteration < maximum_iterations; ++iteration) {
        std::vector<double> hessian_direction = matvec(
            evaluation.hessian, direction);
        ++result.hvp;
        ++work.inner_hvp;
        if (invert_first_hvp) {
            for (double& value : hessian_direction) value = -value;
            invert_first_hvp = false;
        }
        const double curvature = vector_dot(direction, hessian_direction);
        ++work.dot_products;
        if (!std::isfinite(curvature) || !finite_vector(hessian_direction)) {
            result.finite = false;
            result.reason = "NONFINITE";
            return result;
        }
        if (curvature <= 0.0) {
            const double tau = positive_boundary_intersection(
                result.value, direction, radius, work);
            if (!std::isfinite(tau)) {
                result.finite = false;
                result.reason = "NONFINITE";
                return result;
            }
            for (std::size_t index = 0; index < result.value.size(); ++index) {
                result.value[index] += tau * direction[index];
            }
            result.reason = "NEGATIVE_CURVATURE";
            result.boundary = true;
            result.iterations = iteration + 1;
            return result;
        }
        const double alpha = residual_squared / curvature;
        std::vector<double> candidate = result.value;
        for (std::size_t index = 0; index < candidate.size(); ++index) {
            candidate[index] += alpha * direction[index];
        }
        const double candidate_norm = vector_norm(candidate);
        ++work.dot_products;
        if (candidate_norm >= radius) {
            const double tau = positive_boundary_intersection(
                result.value, direction, radius, work);
            if (!std::isfinite(tau)) {
                result.finite = false;
                result.reason = "NONFINITE";
                return result;
            }
            for (std::size_t index = 0; index < result.value.size(); ++index) {
                result.value[index] += tau * direction[index];
            }
            result.reason = "BOUNDARY";
            result.boundary = true;
            result.iterations = iteration + 1;
            return result;
        }
        result.value = std::move(candidate);
        std::vector<double> next_residual = residual;
        for (std::size_t index = 0; index < next_residual.size(); ++index) {
            next_residual[index] += alpha * hessian_direction[index];
        }
        const double next_squared = vector_dot(next_residual, next_residual);
        ++work.dot_products;
        if (std::sqrt(next_squared) <= forcing * initial_norm) {
            result.reason = "RESIDUAL";
            result.iterations = iteration + 1;
            return result;
        }
        const double beta = next_squared / residual_squared;
        for (std::size_t index = 0; index < direction.size(); ++index) {
            direction[index] = -next_residual[index] + beta * direction[index];
        }
        residual = std::move(next_residual);
        residual_squared = next_squared;
    }
    result.reason = "DIMENSION_LIMIT";
    result.iterations = maximum_iterations;
    return result;
}

struct TrialRecord {
    double objective = 0.0;
    double gradient_norm = 0.0;
    double scaled_residual = 0.0;
    double step_norm = 0.0;
    double actual_reduction = 0.0;
    double predicted_reduction = 0.0;
    double ratio = 0.0;
    double radius_before = 0.0;
    double radius_after = 0.0;
    std::string reason;
    int iterations = 0;
    int hvp = 0;
    bool boundary = false;
    bool accepted = false;
    std::string state_root;
};

struct SolveResult {
    bool succeeded = false;
    bool finite = true;
    bool monotonic = true;
    bool saw_reduction_floor = false;
    std::string failure;
    std::string convergence_stop;
    double initial_objective = 0.0;
    double final_objective = 0.0;
    double final_gradient_norm = 0.0;
    double final_scaled_residual = 0.0;
    double minimum_radius = 0.0;
    std::uint64_t active_set_changes = 0;
    std::string final_active_root;
    std::vector<double> state;
    std::vector<TrialRecord> trials;
    ControllerWork work;
};

double scaled_residual(const AssemblyProfile& profile,
    const std::vector<double>& gradient) {
    double maximum = 0.0;
    for (std::size_t particle = 0; particle < gradient.size() / 3U; ++particle) {
        double squared = 0.0;
        for (std::size_t axis = 0; axis < 3U; ++axis) {
            const double value = gradient[3U * particle + axis];
            squared += value * value;
        }
        maximum = std::max(maximum, std::sqrt(squared));
    }
    return profile.time_step * profile.time_step / profile.mass
        * maximum / profile.spacing;
}

using EvaluationFunction = AssemblyResult (*)(const AssemblyProfile&,
    const AssemblyFixture&, const AssemblyContinuousState&, AssemblyVariant);

AssemblyResult host_evaluate(const AssemblyProfile& profile,
    const AssemblyFixture& fixture, const AssemblyContinuousState& state,
    AssemblyVariant) {
    return evaluate_reference_assembly_state(profile, fixture, state);
}

AssemblyResult gpu_evaluate(const AssemblyProfile& profile,
    const AssemblyFixture& fixture, const AssemblyContinuousState& state,
    AssemblyVariant variant) {
    return evaluate_gpu_assembly_state(profile, fixture, state, variant);
}

AssemblyResult gpu_evaluate_f64_energy_rounded(const AssemblyProfile& profile,
    const AssemblyFixture& fixture, const AssemblyContinuousState& state,
    AssemblyVariant) {
    AssemblyResult result = evaluate_gpu_assembly_state(
        profile, fixture, state, AssemblyVariant::F64Energy);
    for (double& component : result.energy) {
        component = static_cast<double>(static_cast<float>(component));
    }
    return result;
}

SolveResult solve(const StaticCase& input, const AssemblyFixture& fixture,
    EvaluationFunction evaluator, bool round_state_to_f32,
    bool invert_first_hvp,
    AssemblyVariant variant = AssemblyVariant::Corrected,
    double scale_aware_limit = 0.0) {
    SolveResult result;
    result.state = input.initial.current_m;
    const double minimum_radius = std::ldexp(input.profile.spacing, -40);
    const double maximum_radius = 4.0 * input.profile.spacing;
    double radius = input.profile.spacing;
    result.minimum_radius = radius;
    AssemblyContinuousState state = input.initial;
    state.current_m = result.state;
    AssemblyResult current = evaluator(
        input.profile, fixture, state, variant);
    ++result.work.evaluator_calls;
    result.work.state_scalar_uploads += 3U * result.state.size();
    add_assembly_work(result.work.assembly, current.work);
    if (current.failure != AssemblyFailure::None
        || !finite_vector(current.gradient) || !finite_vector(current.hessian)) {
        result.finite = false;
        result.failure = "INVALID_INITIAL_EVALUATION";
        return result;
    }
    result.initial_objective = total_energy(current);
    result.final_objective = result.initial_objective;
    std::string pressure_root = active_root(current.pressure_active);
    for (int outer = 0; outer < MAXIMUM_OUTER_TRIALS; ++outer) {
        const double gradient_norm = vector_norm(current.gradient);
        ++result.work.dot_products;
        const double current_scaled = scaled_residual(
            input.profile, current.gradient);
        if (gradient_norm <= RAW_GRADIENT_LIMIT
            || (scale_aware_limit > 0.0
                && current_scaled <= scale_aware_limit)) {
            result.succeeded = true;
            result.convergence_stop = gradient_norm <= RAW_GRADIENT_LIMIT
                ? "RAW_GRADIENT" : "SCALED_DISPLACEMENT";
            break;
        }
        TrialRecord record;
        record.objective = total_energy(current);
        record.gradient_norm = gradient_norm;
        record.scaled_residual = scaled_residual(input.profile, current.gradient);
        record.radius_before = radius;
        InnerStep step = truncated_cg(
            current, radius, invert_first_hvp, result.work);
        record.reason = step.reason;
        record.iterations = step.iterations;
        record.hvp = step.hvp;
        record.boundary = step.boundary;
        if (step.reason == "NEGATIVE_CURVATURE") {
            ++result.work.negative_curvature_stops;
        } else if (step.reason == "BOUNDARY") {
            ++result.work.boundary_stops;
        } else if (step.reason == "RESIDUAL") {
            ++result.work.residual_stops;
        } else if (step.reason == "DIMENSION_LIMIT") {
            ++result.work.dimension_stops;
        }
        record.step_norm = vector_norm(step.value);
        ++result.work.dot_products;
        if (!step.finite || !finite_vector(step.value)) {
            result.finite = false;
            result.failure = "INVALID_INNER_STEP";
            break;
        }
        const std::vector<double> hessian_step = matvec(
            current.hessian, step.value);
        ++result.work.prediction_hvp;
        record.predicted_reduction = -(
            vector_dot(current.gradient, step.value)
            + 0.5 * vector_dot(step.value, hessian_step));
        result.work.dot_products += 2U;
        bool valid_model = std::isfinite(record.predicted_reduction)
            && record.predicted_reduction > 0.0;
        AssemblyResult trial_evaluation;
        std::vector<double> trial = result.state;
        if (valid_model) {
            for (std::size_t index = 0; index < trial.size(); ++index) {
                trial[index] += step.value[index];
                if (round_state_to_f32) {
                    trial[index] = static_cast<double>(static_cast<float>(trial[index]));
                    ++result.work.state_scalar_rounds;
                }
            }
            state.current_m = trial;
            trial_evaluation = evaluator(
                input.profile, fixture, state, variant);
            ++result.work.evaluator_calls;
            result.work.state_scalar_uploads += 3U * trial.size();
            add_assembly_work(result.work.assembly, trial_evaluation.work);
            const double trial_objective = total_energy(trial_evaluation);
            record.actual_reduction = record.objective - trial_objective;
            record.ratio = record.actual_reduction / record.predicted_reduction;
            valid_model = trial_evaluation.failure == AssemblyFailure::None
                && finite_vector(trial_evaluation.gradient)
                && finite_vector(trial_evaluation.hessian)
                && std::isfinite(trial_objective)
                && std::isfinite(record.actual_reduction)
                && std::isfinite(record.ratio)
                && record.actual_reduction > 0.0;
            if (!valid_model && record.predicted_reduction > 0.0
                && record.actual_reduction <= 0.0) {
                result.saw_reduction_floor = true;
            }
        } else {
            record.ratio = -std::numeric_limits<double>::infinity();
        }
        record.accepted = valid_model && record.ratio >= ACCEPT_RATIO;
        if (record.accepted) {
            const double next_objective = total_energy(trial_evaluation);
            result.monotonic = result.monotonic
                && next_objective < record.objective;
            const std::string next_active = active_root(
                trial_evaluation.pressure_active);
            if (next_active != pressure_root) ++result.active_set_changes;
            pressure_root = next_active;
            result.state = std::move(trial);
            current = std::move(trial_evaluation);
            result.final_objective = next_objective;
            ++result.work.accepted_trials;
        } else {
            ++result.work.rejected_trials;
        }
        if (!valid_model || record.ratio < 0.25) {
            radius *= 0.25;
            ++result.work.radius_shrinks;
        } else if (record.ratio > 0.75 && record.boundary) {
            const double expanded = std::min(2.0 * radius, maximum_radius);
            if (expanded != radius) ++result.work.radius_expands;
            radius = expanded;
        }
        result.minimum_radius = std::min(result.minimum_radius, radius);
        record.radius_after = radius;
        record.state_root = vector_root(
            "nextengine.nonlocal.ncga5.state.v1\0", result.state);
        result.trials.push_back(record);
        if (radius < minimum_radius) {
            result.failure = "MINIMUM_TRUST_RADIUS";
            break;
        }
    }
    result.final_gradient_norm = vector_norm(current.gradient);
    ++result.work.dot_products;
    result.final_scaled_residual = scaled_residual(input.profile, current.gradient);
    result.final_active_root = active_root(current.pressure_active);
    if (!result.succeeded && result.final_gradient_norm <= RAW_GRADIENT_LIMIT
        && result.failure.empty()) {
        result.succeeded = true;
        result.convergence_stop = "RAW_GRADIENT";
    }
    if (!result.succeeded && scale_aware_limit > 0.0
        && result.final_scaled_residual <= scale_aware_limit
        && result.failure.empty()) {
        result.succeeded = true;
        result.convergence_stop = "SCALED_DISPLACEMENT";
    }
    if (!result.succeeded && result.failure.empty()) {
        result.failure = "OUTER_TRIAL_LIMIT";
    }
    result.succeeded = result.succeeded && result.finite && result.monotonic;
    return result;
}

double maximum_particle_drift_um(const std::vector<double>& lhs,
    const std::vector<double>& rhs) {
    if (lhs.size() != rhs.size()) return std::numeric_limits<double>::infinity();
    double result = 0.0;
    for (std::size_t particle = 0; particle < lhs.size() / 3U; ++particle) {
        double squared = 0.0;
        for (std::size_t axis = 0; axis < 3U; ++axis) {
            const double delta = 1.0e6
                * (lhs[3U * particle + axis] - rhs[3U * particle + axis]);
            squared += delta * delta;
        }
        result = std::max(result, std::sqrt(squared));
    }
    return result;
}

double objective_difference(const SolveResult& host,
    const SolveResult& candidate) {
    return std::abs(candidate.final_objective - host.final_objective)
        / std::max(std::abs(host.final_objective), 1.0);
}

std::string route_root(const SolveResult& result) {
    std::string bytes = "nextengine.nonlocal.ncga5.route.v1\0";
    append_u64(bytes, result.trials.size());
    for (const TrialRecord& record : result.trials) {
        append_u64(bytes, record.accepted ? 1U : 0U);
        append_u64(bytes, record.boundary ? 1U : 0U);
        append_u64(bytes, static_cast<std::uint64_t>(record.iterations));
        append_u64(bytes, static_cast<std::uint64_t>(record.hvp));
        append_u64(bytes, record.reason.size());
        bytes += record.reason;
        append_double(bytes, record.radius_before);
        append_double(bytes, record.radius_after);
    }
    bytes += result.failure;
    bytes += result.convergence_stop;
    return sha256_hex(bytes);
}

std::string work_root(const ControllerWork& work) {
    std::string bytes = "nextengine.nonlocal.ncga5.work.v1\0";
    const std::array<std::uint64_t, 31> fields{{
        work.evaluator_calls, work.inner_hvp, work.prediction_hvp,
        work.dot_products, work.boundary_intersections, work.accepted_trials,
        work.rejected_trials, work.radius_shrinks, work.radius_expands,
        work.state_scalar_uploads, work.state_scalar_rounds,
        work.negative_curvature_stops, work.boundary_stops,
        work.residual_stops, work.dimension_stops,
        work.assembly.owner_sort_comparisons,
        work.assembly.graph_distance_predicates,
        work.assembly.density_kernel_evaluations,
        work.assembly.energy_pair_visits,
        work.assembly.gradient_pair_visits,
        work.assembly.active_pressure_centers,
        work.assembly.pressure_outer_products,
        work.assembly.pressure_geometric_blocks,
        work.assembly.viscosity_curvature_blocks,
        work.assembly.surface_curvature_blocks,
        work.assembly.dense_entries_written,
        work.assembly.hvp_products,
        work.assembly.host_device_scalar_transfers,
        work.assembly.compensated_additions,
        work.assembly.compensation_initializations,
        static_cast<std::uint64_t>(MAXIMUM_OUTER_TRIALS),
    }};
    for (std::uint64_t field : fields) append_u64(bytes, field);
    return sha256_hex(bytes);
}

std::string solve_root(const SolveResult& result) {
    std::string bytes = "nextengine.nonlocal.ncga5.solve.v1\0";
    bytes += route_root(result);
    bytes += work_root(result.work);
    bytes += vector_root("nextengine.nonlocal.ncga5.state.v1\0", result.state);
    bytes += result.final_active_root;
    append_double(bytes, result.final_objective);
    append_double(bytes, result.final_gradient_norm);
    append_double(bytes, result.final_scaled_residual);
    append_u64(bytes, result.succeeded ? 1U : 0U);
    append_u64(bytes, result.monotonic ? 1U : 0U);
    return sha256_hex(bytes);
}

std::string mixed_work_root(const ControllerWork& work) {
    std::string bytes = "nextengine.nonlocal.ncga6.work.v1\0";
    bytes += work_root(work);
    const std::array<std::uint64_t, 5> fields{{
        work.assembly.f64_energy_density_terms,
        work.assembly.f64_energy_particle_terms,
        work.assembly.f64_energy_viscosity_pairs,
        work.assembly.f64_energy_surface_pairs,
        work.assembly.f64_energy_component_writes,
    }};
    for (std::uint64_t field : fields) append_u64(bytes, field);
    return sha256_hex(bytes);
}

std::string mixed_solve_root(const SolveResult& result) {
    std::string bytes = "nextengine.nonlocal.ncga6.solve.v1\0";
    bytes += route_root(result);
    bytes += mixed_work_root(result.work);
    bytes += vector_root("nextengine.nonlocal.ncga5.state.v1\0", result.state);
    bytes += result.final_active_root;
    append_double(bytes, result.final_objective);
    append_double(bytes, result.final_gradient_norm);
    append_double(bytes, result.final_scaled_residual);
    append_u64(bytes, result.succeeded ? 1U : 0U);
    append_u64(bytes, result.monotonic ? 1U : 0U);
    return sha256_hex(bytes);
}

bool host_shape_valid(const StaticCase& input, const SolveResult& result) {
    const bool common = result.finite && result.monotonic
        && result.active_set_changes == 0U
        && result.work.negative_curvature_stops == 0U
        && result.work.residual_stops > 0U;
    if (!common) return false;
    if (input.name == "compressed_pair") {
        return result.succeeded && result.failure.empty()
            && result.trials.size()
                == static_cast<std::size_t>(input.expected_outer)
            && result.work.evaluator_calls == input.expected_evaluations
            && result.work.inner_hvp + result.work.prediction_hvp
                == input.expected_hvp
            && result.work.rejected_trials == 0U
            && result.work.radius_shrinks == 0U
            && result.final_gradient_norm <= RAW_GRADIENT_LIMIT;
    }
    const bool raw = result.succeeded && result.failure.empty()
        && result.final_gradient_norm <= RAW_GRADIENT_LIMIT;
    const bool scale_aware_floor = !result.succeeded
        && result.saw_reduction_floor
        && (result.failure == "MINIMUM_TRUST_RADIUS"
            || result.failure == "OUTER_TRIAL_LIMIT")
        && result.final_scaled_residual <= 1.0e-8;
    const bool multi_hvp_residual = std::any_of(
        result.trials.begin(), result.trials.end(),
        [](const TrialRecord& trial) {
            return trial.reason == "RESIDUAL" && trial.hvp > 1;
        });
    return multi_hvp_residual && (raw || scale_aware_floor);
}

bool candidate_supported(const SolveResult& host, const SolveResult& candidate) {
    return candidate.succeeded && candidate.finite && candidate.monotonic
        && candidate.failure.empty() && candidate.active_set_changes == 0U
        && candidate.final_active_root == host.final_active_root
        && maximum_particle_drift_um(host.state, candidate.state)
            <= POSITION_LIMIT_UM
        && objective_difference(host, candidate) <= OBJECTIVE_LIMIT
        && candidate.final_scaled_residual <= SUPPORTED_SCALED_RESIDUAL;
}

bool candidate_at_floor(const SolveResult& host, const SolveResult& candidate) {
    return !candidate.succeeded && candidate.finite && candidate.monotonic
        && candidate.saw_reduction_floor && candidate.work.radius_shrinks > 0U
        && candidate.active_set_changes == 0U
        && candidate.final_active_root == host.final_active_root
        && maximum_particle_drift_um(host.state, candidate.state)
            <= POSITION_LIMIT_UM
        && objective_difference(host, candidate) <= OBJECTIVE_LIMIT
        && candidate.final_scaled_residual <= FLOOR_SCALED_RESIDUAL;
}

void emit_trials(std::ostringstream& output, const SolveResult& result) {
    output << '[';
    for (std::size_t index = 0; index < result.trials.size(); ++index) {
        if (index != 0U) output << ',';
        const TrialRecord& trial = result.trials[index];
        output << "{\"outer\":" << index
               << ",\"objective\":" << trial.objective
               << ",\"gradient_norm\":" << trial.gradient_norm
               << ",\"scaled_residual\":" << trial.scaled_residual
               << ",\"step_norm\":" << trial.step_norm
               << ",\"ared\":" << trial.actual_reduction
               << ",\"pred\":" << trial.predicted_reduction
               << ",\"rho\":" << trial.ratio
               << ",\"radius_before\":" << trial.radius_before
               << ",\"radius_after\":" << trial.radius_after
               << ",\"reason\":\"" << trial.reason << "\""
               << ",\"iterations\":" << trial.iterations
               << ",\"hvp\":" << trial.hvp
               << ",\"boundary\":" << (trial.boundary ? "true" : "false")
               << ",\"accepted\":" << (trial.accepted ? "true" : "false")
               << ",\"state_root\":\"" << trial.state_root << "\"}";
    }
    output << ']';
}

void emit_solve(std::ostringstream& output, const SolveResult& result) {
    output << "{\"succeeded\":" << (result.succeeded ? "true" : "false")
           << ",\"finite\":" << (result.finite ? "true" : "false")
           << ",\"monotonic\":" << (result.monotonic ? "true" : "false")
           << ",\"saw_reduction_floor\":"
           << (result.saw_reduction_floor ? "true" : "false")
           << ",\"failure\":\"" << result.failure << "\""
           << ",\"convergence_stop\":\"" << result.convergence_stop << "\""
           << ",\"initial_objective\":" << result.initial_objective
           << ",\"final_objective\":" << result.final_objective
           << ",\"final_gradient_norm\":" << result.final_gradient_norm
           << ",\"final_scaled_residual\":" << result.final_scaled_residual
           << ",\"minimum_radius\":" << result.minimum_radius
           << ",\"active_set_changes\":" << result.active_set_changes
           << ",\"final_active_root\":\"" << result.final_active_root << "\""
           << ",\"state_root\":\""
           << vector_root("nextengine.nonlocal.ncga5.state.v1\0", result.state)
           << "\",\"route_root\":\"" << route_root(result)
           << "\",\"work_root\":\"" << work_root(result.work)
           << "\",\"solve_root\":\"" << solve_root(result) << "\""
           << ",\"work\":{\"evaluator_calls\":" << result.work.evaluator_calls
           << ",\"inner_hvp\":" << result.work.inner_hvp
           << ",\"prediction_hvp\":" << result.work.prediction_hvp
           << ",\"dot_products\":" << result.work.dot_products
           << ",\"accepted_trials\":" << result.work.accepted_trials
           << ",\"rejected_trials\":" << result.work.rejected_trials
           << ",\"radius_shrinks\":" << result.work.radius_shrinks
           << ",\"radius_expands\":" << result.work.radius_expands
           << ",\"state_scalar_uploads\":" << result.work.state_scalar_uploads
           << ",\"state_scalar_rounds\":" << result.work.state_scalar_rounds
           << ",\"negative_curvature_stops\":"
           << result.work.negative_curvature_stops
           << ",\"boundary_stops\":" << result.work.boundary_stops
           << ",\"residual_stops\":" << result.work.residual_stops
           << ",\"dimension_stops\":" << result.work.dimension_stops
           << "},\"trials\":";
    emit_trials(output, result);
    output << '}';
}

int run() {
    const std::array<StaticCase, 2> cases{{compressed_case(), combined_case()}};
    std::array<SolveResult, 2> host_results;
    std::array<SolveResult, 2> gpu_results;
    std::array<SolveResult, 2> permuted_results;
    for (std::size_t index = 0; index < cases.size(); ++index) {
        host_results[index] = solve(
            cases[index], cases[index].fixture, host_evaluate, false, false);
    }
    for (std::size_t index = 0; index < cases.size(); ++index) {
        gpu_results[index] = solve(
            cases[index], cases[index].fixture, gpu_evaluate, true, false);
    }
    for (std::size_t index = 0; index < cases.size(); ++index) {
        permuted_results[index] = solve(cases[index],
            cases[index].permuted_fixture, gpu_evaluate, true, false);
    }

    SolveResult negative = solve(
        cases[1], cases[1].fixture, gpu_evaluate, true, true);
    AssemblyContinuousState invalid_state = cases[1].initial;
    invalid_state.current_m[0] = std::numeric_limits<double>::quiet_NaN();
    const AssemblyResult invalid = evaluate_gpu_assembly_state(cases[1].profile,
        cases[1].fixture, invalid_state, AssemblyVariant::Corrected);
    const AssemblyResult ownership_base = evaluate_reference_assembly_state(
        cases[1].profile, cases[1].fixture, cases[1].initial);
    AssemblyContinuousState predicted_control = cases[1].initial;
    predicted_control.predicted_m[0] += 0.001;
    const AssemblyResult predicted_changed = evaluate_reference_assembly_state(
        cases[1].profile, cases[1].fixture, predicted_control);
    AssemblyContinuousState reference_control = cases[1].initial;
    reference_control.reference_m[3] += 0.001;
    const AssemblyResult reference_changed = evaluate_reference_assembly_state(
        cases[1].profile, cases[1].fixture, reference_control);

    bool host_valid = true;
    bool permutation_exact = true;
    bool all_supported = true;
    bool all_supported_or_floor = true;
    bool any_floor = false;
    bool residual_exercised = false;
    for (std::size_t index = 0; index < cases.size(); ++index) {
        host_valid = host_valid && host_shape_valid(cases[index], host_results[index]);
        permutation_exact = permutation_exact
            && solve_root(gpu_results[index]) == solve_root(permuted_results[index]);
        const bool supported = candidate_supported(
            host_results[index], gpu_results[index]);
        const bool floor = candidate_at_floor(host_results[index], gpu_results[index]);
        all_supported = all_supported && supported;
        all_supported_or_floor = all_supported_or_floor && (supported || floor);
        any_floor = any_floor || floor;
        residual_exercised = residual_exercised
            || gpu_results[index].work.residual_stops > 0U;
    }
    const bool negative_rejected = solve_root(negative) != solve_root(gpu_results[1]);
    const bool state_controls = invalid.failure == AssemblyFailure::InvalidInput
        && total_energy(predicted_changed) != total_energy(ownership_base)
        && (total_energy(reference_changed) != total_energy(ownership_base)
            || reference_changed.gradient != ownership_base.gradient);
    const bool apparatus = host_valid && permutation_exact
        && negative_rejected && state_controls;

    std::string result = "INCONCLUSIVE";
    if (apparatus && all_supported && residual_exercised) {
        result = "GPU_STATIC_SOLVE_SUPPORTED";
    } else if (apparatus && all_supported_or_floor && any_floor
        && residual_exercised) {
        result = "STRICT_F32_CONVERGENCE_FLOOR";
    } else if (apparatus) {
        result = "F32_STATIC_SOLVE_MATERIAL";
    }

    std::string semantic = "nextengine.nonlocal.ncga5.semantic.v1\0";
    semantic += result;
    for (const SolveResult& value : host_results) semantic += solve_root(value);
    for (const SolveResult& value : gpu_results) semantic += solve_root(value);
    semantic += solve_root(negative);
    append_u64(semantic, host_valid ? 1U : 0U);
    append_u64(semantic, permutation_exact ? 1U : 0U);
    append_u64(semantic, negative_rejected ? 1U : 0U);
    append_u64(semantic, state_controls ? 1U : 0U);

    std::ostringstream output;
    output << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.ncga5.v1\""
           << ",\"result\":\"" << result << "\""
           << ",\"apparatus_valid\":" << (apparatus ? "true" : "false")
           << ",\"host_shape_valid\":" << (host_valid ? "true" : "false")
           << ",\"permutation_exact\":"
           << (permutation_exact ? "true" : "false")
           << ",\"negative_rejected\":"
           << (negative_rejected ? "true" : "false")
           << ",\"state_controls\":" << (state_controls ? "true" : "false")
           << ",\"environment\":" << gpu_assembly_environment_json()
           << ",\"cases\":[";
    for (std::size_t index = 0; index < cases.size(); ++index) {
        if (index != 0U) output << ',';
        output << "{\"name\":\"" << cases[index].name << "\""
               << ",\"maximum_state_drift_um\":"
               << maximum_particle_drift_um(
                    host_results[index].state, gpu_results[index].state)
               << ",\"objective_difference\":"
               << objective_difference(host_results[index], gpu_results[index])
               << ",\"host\":";
        emit_solve(output, host_results[index]);
        output << ",\"strict_f32\":";
        emit_solve(output, gpu_results[index]);
        output << '}';
    }
    output << "],\"negative_root\":\"" << solve_root(negative)
           << "\",\"semantic_root\":\"" << sha256_hex(semantic) << "\"}";
    std::cout << output.str() << '\n';
    return apparatus && result != "INCONCLUSIVE" ? 0 : 1;
}

bool accepted_where_strict_rejected(
    const SolveResult& strict, const SolveResult& mixed) {
    const std::size_t count = std::min(strict.trials.size(), mixed.trials.size());
    for (std::size_t index = 0; index < count; ++index) {
        if (!strict.trials[index].accepted && mixed.trials[index].accepted
            && strict.trials[index].actual_reduction <= 0.0) return true;
        if (strict.trials[index].accepted != mixed.trials[index].accepted) break;
    }
    return false;
}

int run_mixed_energy() {
    const std::array<StaticCase, 2> cases{{compressed_case(), combined_case()}};
    std::array<SolveResult, 2> host_results;
    std::array<SolveResult, 2> strict_results;
    std::array<SolveResult, 2> mixed_results;
    std::array<SolveResult, 2> permuted_results;
    std::array<SolveResult, 2> rounded_energy_results;
    for (std::size_t index = 0; index < cases.size(); ++index) {
        host_results[index] = solve(
            cases[index], cases[index].fixture, host_evaluate, false, false);
    }
    for (std::size_t index = 0; index < cases.size(); ++index) {
        strict_results[index] = solve(cases[index], cases[index].fixture,
            gpu_evaluate, true, false, AssemblyVariant::Corrected,
            SUPPORTED_SCALED_RESIDUAL);
    }
    for (std::size_t index = 0; index < cases.size(); ++index) {
        mixed_results[index] = solve(cases[index], cases[index].fixture,
            gpu_evaluate, true, false, AssemblyVariant::F64Energy,
            SUPPORTED_SCALED_RESIDUAL);
    }
    for (std::size_t index = 0; index < cases.size(); ++index) {
        permuted_results[index] = solve(cases[index],
            cases[index].permuted_fixture, gpu_evaluate, true, false,
            AssemblyVariant::F64Energy, SUPPORTED_SCALED_RESIDUAL);
        rounded_energy_results[index] = solve(cases[index], cases[index].fixture,
            gpu_evaluate_f64_energy_rounded, true, false,
            AssemblyVariant::F64Energy, SUPPORTED_SCALED_RESIDUAL);
    }
    const SolveResult negative = solve(cases[1], cases[1].fixture,
        gpu_evaluate, true, true, AssemblyVariant::F64Energy,
        SUPPORTED_SCALED_RESIDUAL);

    bool host_valid = true;
    bool strict_all_supported = true;
    bool mixed_all_supported = true;
    bool permutation_exact = true;
    bool rounded_rejected = false;
    bool causal_acceptance = false;
    bool residual_exercised = false;
    bool progress_changed = false;
    for (std::size_t index = 0; index < cases.size(); ++index) {
        host_valid = host_valid && host_shape_valid(cases[index], host_results[index]);
        strict_all_supported = strict_all_supported && candidate_supported(
            host_results[index], strict_results[index]);
        mixed_all_supported = mixed_all_supported && candidate_supported(
            host_results[index], mixed_results[index]);
        permutation_exact = permutation_exact
            && mixed_solve_root(mixed_results[index])
                == mixed_solve_root(permuted_results[index]);
        rounded_rejected = rounded_rejected
            || !candidate_supported(host_results[index], rounded_energy_results[index]);
        causal_acceptance = causal_acceptance || accepted_where_strict_rejected(
            strict_results[index], mixed_results[index]);
        residual_exercised = residual_exercised
            || std::any_of(mixed_results[index].trials.begin(),
                mixed_results[index].trials.end(), [](const TrialRecord& trial) {
                    return trial.reason == "RESIDUAL" && trial.hvp > 1;
                });
        progress_changed = progress_changed
            || mixed_results[index].work.accepted_trials
                > strict_results[index].work.accepted_trials
            || mixed_results[index].final_scaled_residual
                < strict_results[index].final_scaled_residual;
    }
    const bool negative_rejected = mixed_solve_root(negative)
        != mixed_solve_root(mixed_results[1]);
    AssemblyContinuousState invalid_state = cases[1].initial;
    invalid_state.current_m[0] = std::numeric_limits<double>::infinity();
    const AssemblyResult invalid = evaluate_gpu_assembly_state(cases[1].profile,
        cases[1].fixture, invalid_state, AssemblyVariant::F64Energy);
    const bool controls = permutation_exact && rounded_rejected
        && negative_rejected && invalid.failure == AssemblyFailure::InvalidInput;
    const bool apparatus = host_valid && controls;

    std::string result = "INCONCLUSIVE";
    if (apparatus && mixed_all_supported && !strict_all_supported
        && causal_acceptance && residual_exercised) {
        result = "GPU_F64_ENERGY_STATIC_SOLVE_SUPPORTED";
    } else if (apparatus && progress_changed) {
        result = "PRESSURE_OPERATOR_PRECISION_REQUIRED";
    } else if (apparatus) {
        result = "F64_ENERGY_NOT_CAUSAL";
    }

    std::string semantic = "nextengine.nonlocal.ncga6.semantic.v1\0";
    semantic += result;
    for (const SolveResult& value : host_results) semantic += solve_root(value);
    for (const SolveResult& value : strict_results) semantic += solve_root(value);
    for (const SolveResult& value : mixed_results) semantic += mixed_solve_root(value);
    semantic += mixed_solve_root(negative);
    append_u64(semantic, permutation_exact ? 1U : 0U);
    append_u64(semantic, rounded_rejected ? 1U : 0U);
    append_u64(semantic, causal_acceptance ? 1U : 0U);

    std::ostringstream output;
    output << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.ncga6.v1\""
           << ",\"result\":\"" << result << "\""
           << ",\"apparatus_valid\":" << (apparatus ? "true" : "false")
           << ",\"host_shape_valid\":" << (host_valid ? "true" : "false")
           << ",\"strict_all_supported\":"
           << (strict_all_supported ? "true" : "false")
           << ",\"mixed_all_supported\":"
           << (mixed_all_supported ? "true" : "false")
           << ",\"causal_acceptance\":"
           << (causal_acceptance ? "true" : "false")
           << ",\"permutation_exact\":"
           << (permutation_exact ? "true" : "false")
           << ",\"rounded_energy_rejected\":"
           << (rounded_rejected ? "true" : "false")
           << ",\"negative_rejected\":"
           << (negative_rejected ? "true" : "false")
           << ",\"environment\":" << gpu_assembly_environment_json()
           << ",\"cases\":[";
    for (std::size_t index = 0; index < cases.size(); ++index) {
        if (index != 0U) output << ',';
        output << "{\"name\":\"" << cases[index].name << "\""
               << ",\"strict_drift_um\":"
               << maximum_particle_drift_um(
                    host_results[index].state, strict_results[index].state)
               << ",\"mixed_drift_um\":"
               << maximum_particle_drift_um(
                    host_results[index].state, mixed_results[index].state)
               << ",\"strict_objective_difference\":"
               << objective_difference(host_results[index], strict_results[index])
               << ",\"mixed_objective_difference\":"
               << objective_difference(host_results[index], mixed_results[index])
               << ",\"host\":";
        emit_solve(output, host_results[index]);
        output << ",\"strict_f32\":";
        emit_solve(output, strict_results[index]);
        output << ",\"f64_energy\":";
        emit_solve(output, mixed_results[index]);
        output << ",\"f64_energy_work_root\":\""
               << mixed_work_root(mixed_results[index].work) << "\""
               << ",\"f64_energy_work\":{\"density_terms\":"
               << mixed_results[index].work.assembly.f64_energy_density_terms
               << ",\"particle_terms\":"
               << mixed_results[index].work.assembly.f64_energy_particle_terms
               << ",\"viscosity_pairs\":"
               << mixed_results[index].work.assembly.f64_energy_viscosity_pairs
               << ",\"surface_pairs\":"
               << mixed_results[index].work.assembly.f64_energy_surface_pairs
               << ",\"component_writes\":"
               << mixed_results[index].work.assembly.f64_energy_component_writes
               << "}}";
    }
    output << "],\"negative_root\":\"" << mixed_solve_root(negative)
           << "\",\"semantic_root\":\"" << sha256_hex(semantic) << "\"}";
    std::cout << output.str() << '\n';
    return apparatus && result != "INCONCLUSIVE" ? 0 : 1;
}

} // namespace
} // namespace nextengine::nonlocal::gpu_assembly_audit

int main(int argc, char** argv) {
    try {
        if (argc == 2 && std::string(argv[1]) == "--mixed-energy") {
            return nextengine::nonlocal::gpu_assembly_audit::run_mixed_energy();
        }
        if (argc != 1) {
            std::cerr << "usage: nonlocal-corrected-cuda-static-solve "
                         "[--mixed-energy]\n";
            return 2;
        }
        return nextengine::nonlocal::gpu_assembly_audit::run();
    } catch (const std::exception& error) {
        std::cerr << "NCGA5_UNEXPECTED:" << error.what() << '\n';
        return 2;
    }
}
