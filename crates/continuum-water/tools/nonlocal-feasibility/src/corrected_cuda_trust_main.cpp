#include "corrected_cuda_assembly.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <functional>
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
constexpr int OUTER_TRIALS = 8;
constexpr double INITIAL_RADIUS = 50.0e-6;
constexpr double MINIMUM_RADIUS = INITIAL_RADIUS / 1048576.0;
constexpr double MAXIMUM_RADIUS = 4.0 * INITIAL_RADIUS;
constexpr double ACCEPT_RATIO = 0.1;
constexpr double POSITION_GATE_UM = 5.0;

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

std::vector<double> initial_state(const AssemblyFixture& fixture) {
    std::vector<double> result;
    result.reserve(3U * fixture.samples.size());
    for (std::size_t index : canonical_indices(fixture)) {
        for (std::int64_t value : fixture.samples[index].current_um) {
            result.push_back(1.0e-6 * static_cast<double>(value));
        }
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
    static_assert(sizeof(bits) == sizeof(value), "NCGA4 binary64 layout");
    std::memcpy(&bits, &value, sizeof(bits));
    append_u64(bytes, bits);
}

std::string state_root(const std::vector<double>& state) {
    std::string bytes = "nextengine.nonlocal.ncga4.state.v1\0";
    append_u64(bytes, state.size());
    for (double value : state) append_double(bytes, value);
    return sha256_hex(bytes);
}

double dot(const std::vector<double>& lhs, const std::vector<double>& rhs) {
    if (lhs.size() != rhs.size()) throw std::runtime_error("NCGA4 dot size mismatch");
    double result = 0.0;
    for (std::size_t index = 0; index < lhs.size(); ++index) {
        result += lhs[index] * rhs[index];
    }
    return result;
}

double norm(const std::vector<double>& value) {
    return std::sqrt(std::max(dot(value, value), 0.0));
}

std::vector<double> matvec(const std::vector<double>& matrix,
    const std::vector<double>& value) {
    if (matrix.size() != value.size() * value.size()) {
        throw std::runtime_error("NCGA4 matvec size mismatch");
    }
    std::vector<double> result(value.size(), 0.0);
    for (std::size_t row = 0; row < value.size(); ++row) {
        double sum = 0.0;
        for (std::size_t column = 0; column < value.size(); ++column) {
            sum += matrix[row * value.size() + column] * value[column];
        }
        result[row] = sum;
    }
    return result;
}

bool finite_vector(const std::vector<double>& value) {
    return std::all_of(value.begin(), value.end(),
        [](double component) { return std::isfinite(component); });
}

double total_energy(const AssemblyResult& value) {
    double result = 0.0;
    for (double component : value.energy) result += component;
    return result;
}

struct TopologySignature {
    std::vector<std::uint8_t> support;
    std::vector<std::uint8_t> first_surface;
    std::vector<std::uint8_t> outer_surface;
};

TopologySignature topology_signature(
    const std::vector<double>& state, const AssemblyProfile& profile) {
    TopologySignature result;
    const std::size_t count = state.size() / 3U;
    result.support.reserve(count * (count - 1U) / 2U);
    result.first_surface.reserve(result.support.capacity());
    result.outer_surface.reserve(result.support.capacity());
    for (std::size_t first = 0; first < count; ++first) {
        for (std::size_t second = first + 1U; second < count; ++second) {
            double squared = 0.0;
            for (std::size_t axis = 0; axis < 3U; ++axis) {
                const double delta = state[3U * first + axis]
                    - state[3U * second + axis];
                squared += delta * delta;
            }
            const double radius = std::sqrt(squared);
            result.support.push_back(radius <= profile.horizon ? 1U : 0U);
            result.first_surface.push_back(radius <= profile.spacing ? 1U : 0U);
            result.outer_surface.push_back(
                radius < 3.0 * profile.spacing ? 1U : 0U);
        }
    }
    return result;
}

bool same_topology(
    const TopologySignature& lhs, const TopologySignature& rhs) {
    return lhs.support == rhs.support
        && lhs.first_surface == rhs.first_surface
        && lhs.outer_surface == rhs.outer_surface;
}

bool same_assembly_topology(
    const AssemblyResult& lhs, const AssemblyResult& rhs) {
    return lhs.owner_ids == rhs.owner_ids
        && lhs.current_offsets == rhs.current_offsets
        && lhs.current_neighbors == rhs.current_neighbors
        && lhs.reference_offsets == rhs.reference_offsets
        && lhs.reference_neighbors == rhs.reference_neighbors;
}

void add_work(AssemblyWorkReceipt& target, const AssemblyWorkReceipt& value) {
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
}

struct ControllerWork {
    std::uint64_t evaluator_calls = 0;
    std::uint64_t cg_matvecs = 0;
    std::uint64_t prediction_matvecs = 0;
    std::uint64_t dot_products = 0;
    std::uint64_t boundary_intersections = 0;
    std::uint64_t accepted_trials = 0;
    std::uint64_t rejected_trials = 0;
    std::uint64_t radius_shrinks = 0;
    std::uint64_t radius_expands = 0;
    std::uint64_t continuous_state_uploads = 0;
    std::uint64_t trial_state_f32_rounds = 0;
    std::uint64_t topology_pair_checks = 0;
    AssemblyWorkReceipt assembly;
};

struct InnerStep {
    std::vector<double> value;
    std::string reason;
    int iterations = 0;
    int hvp_calls = 0;
    bool boundary = false;
    bool finite = true;
};

double positive_boundary_intersection(const std::vector<double>& point,
    const std::vector<double>& direction, double radius,
    ControllerWork& work) {
    ++work.boundary_intersections;
    const double a = dot(direction, direction);
    const double b = 2.0 * dot(point, direction);
    const double c = dot(point, point) - radius * radius;
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
    std::vector<double> direction(residual.size(), 0.0);
    for (std::size_t index = 0; index < residual.size(); ++index) {
        direction[index] = -residual[index];
    }
    double residual_squared = dot(residual, residual);
    ++work.dot_products;
    const double gradient_norm = std::sqrt(residual_squared);
    const double forcing = std::min(0.5, std::sqrt(gradient_norm));
    const int maximum_iterations = static_cast<int>(3U * residual.size());
    for (int iteration = 0; iteration < maximum_iterations; ++iteration) {
        std::vector<double> hessian_direction = matvec(
            evaluation.hessian, direction);
        ++result.hvp_calls;
        ++work.cg_matvecs;
        if (invert_first_hvp) {
            for (double& value : hessian_direction) value = -value;
            invert_first_hvp = false;
        }
        const double curvature = dot(direction, hessian_direction);
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
        const double candidate_norm = norm(candidate);
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
        const double next_squared = dot(next_residual, next_residual);
        ++work.dot_products;
        if (std::sqrt(next_squared) <= forcing * gradient_norm) {
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
    double step_norm = 0.0;
    double actual_reduction = 0.0;
    double predicted_reduction = 0.0;
    double ratio = 0.0;
    double radius_before = 0.0;
    double radius_after = 0.0;
    std::string reason;
    int iterations = 0;
    int hvp_calls = 0;
    bool boundary = false;
    bool accepted = false;
    std::uint64_t active_count = 0;
    std::string active_root;
    std::string state_root;
    std::vector<double> state;
};

std::string active_root(const std::vector<std::uint8_t>& active) {
    std::string bytes = "nextengine.nonlocal.ncga4.active.v1\0";
    append_u64(bytes, active.size());
    bytes.append(reinterpret_cast<const char*>(active.data()), active.size());
    return sha256_hex(bytes);
}

struct SolveResult {
    bool finite = true;
    bool topology_safe = true;
    bool monotonic = true;
    std::string failure;
    double initial_objective = 0.0;
    double final_objective = 0.0;
    double minimum_radius = INITIAL_RADIUS;
    std::vector<double> state;
    std::vector<TrialRecord> trials;
    ControllerWork work;
};

using Evaluator = std::function<AssemblyResult(const std::vector<double>&)>;

SolveResult solve_prefix(const std::vector<double>& initial,
    const AssemblyProfile& profile,
    const TopologySignature& frozen_topology, const Evaluator& evaluator,
    bool round_accepted_to_f32, bool invert_first_hvp) {
    SolveResult result;
    result.state = initial;
    double radius = INITIAL_RADIUS;
    AssemblyResult current = evaluator(result.state);
    ++result.work.evaluator_calls;
    if (round_accepted_to_f32) {
        result.work.continuous_state_uploads += result.state.size();
    }
    add_work(result.work.assembly, current.work);
    if (current.failure != AssemblyFailure::None
        || !finite_vector(current.gradient) || !finite_vector(current.hessian)) {
        result.finite = false;
        result.failure = "INVALID_INITIAL_EVALUATION";
        return result;
    }
    result.initial_objective = total_energy(current);
    result.final_objective = result.initial_objective;
    if (!std::isfinite(result.initial_objective)) {
        result.finite = false;
        result.failure = "NONFINITE_INITIAL_OBJECTIVE";
        return result;
    }
    for (int outer = 0; outer < OUTER_TRIALS; ++outer) {
        TrialRecord record;
        record.objective = total_energy(current);
        record.gradient_norm = norm(current.gradient);
        ++result.work.dot_products;
        record.radius_before = radius;
        InnerStep step = truncated_cg(
            current, radius, invert_first_hvp, result.work);
        record.reason = step.reason;
        record.iterations = step.iterations;
        record.hvp_calls = step.hvp_calls;
        record.boundary = step.boundary;
        record.step_norm = norm(step.value);
        ++result.work.dot_products;
        if (!step.finite || !finite_vector(step.value)) {
            result.finite = false;
            result.failure = "INVALID_INNER_STEP";
            break;
        }
        const std::vector<double> hessian_step = matvec(current.hessian, step.value);
        ++result.work.prediction_matvecs;
        record.predicted_reduction = -(
            dot(current.gradient, step.value)
            + 0.5 * dot(step.value, hessian_step));
        result.work.dot_products += 2U;
        std::vector<double> trial = result.state;
        for (std::size_t index = 0; index < trial.size(); ++index) {
            trial[index] += step.value[index];
            if (round_accepted_to_f32) {
                trial[index] = static_cast<double>(static_cast<float>(trial[index]));
                ++result.work.trial_state_f32_rounds;
            }
        }
        result.work.topology_pair_checks += trial.size() / 3U
            * (trial.size() / 3U - 1U) / 2U;
        if (!same_topology(topology_signature(trial, profile),
                frozen_topology)) {
            result.topology_safe = false;
            result.failure = "TOPOLOGY_OR_BRANCH_CROSSING";
            break;
        }
        AssemblyResult trial_evaluation = evaluator(trial);
        ++result.work.evaluator_calls;
        if (round_accepted_to_f32) {
            result.work.continuous_state_uploads += trial.size();
        }
        add_work(result.work.assembly, trial_evaluation.work);
        const double trial_objective = total_energy(trial_evaluation);
        record.actual_reduction = record.objective - trial_objective;
        const bool valid = trial_evaluation.failure == AssemblyFailure::None
            && finite_vector(trial_evaluation.gradient)
            && finite_vector(trial_evaluation.hessian)
            && std::isfinite(trial_objective)
            && std::isfinite(record.actual_reduction)
            && std::isfinite(record.predicted_reduction)
            && record.actual_reduction > 0.0
            && record.predicted_reduction > 0.0;
        record.ratio = valid
            ? record.actual_reduction / record.predicted_reduction
            : -std::numeric_limits<double>::infinity();
        record.accepted = valid && std::isfinite(record.ratio)
            && record.ratio >= ACCEPT_RATIO;
        if (record.accepted) {
            result.monotonic = result.monotonic
                && trial_objective < record.objective;
            result.state = std::move(trial);
            current = std::move(trial_evaluation);
            result.final_objective = trial_objective;
            ++result.work.accepted_trials;
        } else {
            ++result.work.rejected_trials;
        }
        if (!valid || record.ratio < 0.25) {
            radius *= 0.25;
            ++result.work.radius_shrinks;
        } else if (record.ratio > 0.75 && record.boundary) {
            const double expanded = std::min(2.0 * radius, MAXIMUM_RADIUS);
            if (expanded != radius) ++result.work.radius_expands;
            radius = expanded;
        }
        result.minimum_radius = std::min(result.minimum_radius, radius);
        record.radius_after = radius;
        record.active_count = static_cast<std::uint64_t>(std::count(
            current.pressure_active.begin(), current.pressure_active.end(),
            static_cast<std::uint8_t>(1U)));
        record.active_root = active_root(current.pressure_active);
        record.state_root = state_root(result.state);
        record.state = result.state;
        result.trials.push_back(record);
        if (!(radius >= MINIMUM_RADIUS)) {
            result.failure = "MINIMUM_RADIUS";
            break;
        }
    }
    return result;
}

std::string signature_root(const SolveResult& result) {
    std::string bytes = "nextengine.nonlocal.ncga4.signature.v1\0";
    append_u64(bytes, result.trials.size());
    for (const TrialRecord& record : result.trials) {
        append_u64(bytes, record.accepted ? 1U : 0U);
        append_u64(bytes, record.boundary ? 1U : 0U);
        append_u64(bytes, static_cast<std::uint64_t>(record.iterations));
        append_u64(bytes, static_cast<std::uint64_t>(record.hvp_calls));
        append_u64(bytes, record.reason.size());
        bytes += record.reason;
        append_double(bytes, record.radius_before);
        append_double(bytes, record.radius_after);
        append_u64(bytes, record.active_count);
        bytes += record.active_root;
    }
    return sha256_hex(bytes);
}

std::string work_root(const ControllerWork& work) {
    std::string bytes = "nextengine.nonlocal.ncga4.work.v1\0";
    const std::array<std::uint64_t, 28> fields{
        work.evaluator_calls, work.cg_matvecs, work.prediction_matvecs,
        work.dot_products, work.boundary_intersections, work.accepted_trials,
        work.rejected_trials, work.radius_shrinks, work.radius_expands,
        work.continuous_state_uploads, work.trial_state_f32_rounds,
        work.topology_pair_checks,
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
        static_cast<std::uint64_t>(OUTER_TRIALS)};
    for (std::uint64_t field : fields) append_u64(bytes, field);
    return sha256_hex(bytes);
}

double maximum_particle_difference_um(
    const std::vector<double>& lhs, const std::vector<double>& rhs) {
    if (lhs.size() != rhs.size()) return std::numeric_limits<double>::infinity();
    double maximum = 0.0;
    for (std::size_t particle = 0; particle < lhs.size() / 3U; ++particle) {
        double squared = 0.0;
        for (std::size_t axis = 0; axis < 3U; ++axis) {
            const double delta = 1.0e6
                * (lhs[3U * particle + axis] - rhs[3U * particle + axis]);
            squared += delta * delta;
        }
        maximum = std::max(maximum, std::sqrt(squared));
    }
    return maximum;
}

bool same_controller_route(const SolveResult& lhs, const SolveResult& rhs) {
    if (lhs.trials.size() != rhs.trials.size()) return false;
    for (std::size_t index = 0; index < lhs.trials.size(); ++index) {
        const TrialRecord& a = lhs.trials[index];
        const TrialRecord& b = rhs.trials[index];
        if (a.accepted != b.accepted || a.boundary != b.boundary
            || a.reason != b.reason || a.iterations != b.iterations
            || a.hvp_calls != b.hvp_calls
            || a.radius_before != b.radius_before
            || a.radius_after != b.radius_after
            || a.active_count != b.active_count
            || a.active_root != b.active_root) return false;
    }
    return true;
}

double maximum_trace_drift_um(
    const SolveResult& reference, const SolveResult& candidate) {
    double result = maximum_particle_difference_um(
        reference.state, candidate.state);
    const std::size_t count = std::min(
        reference.trials.size(), candidate.trials.size());
    for (std::size_t index = 0; index < count; ++index) {
        if (reference.trials[index].accepted != candidate.trials[index].accepted) {
            return std::numeric_limits<double>::infinity();
        }
        result = std::max(result, maximum_particle_difference_um(
            reference.trials[index].state, candidate.trials[index].state));
    }
    return result;
}

std::string solve_root(const SolveResult& result) {
    std::string bytes = "nextengine.nonlocal.ncga4.solve.v1\0";
    bytes += signature_root(result);
    bytes += state_root(result.state);
    bytes += work_root(result.work);
    append_double(bytes, result.initial_objective);
    append_double(bytes, result.final_objective);
    append_u64(bytes, result.finite ? 1U : 0U);
    append_u64(bytes, result.topology_safe ? 1U : 0U);
    append_u64(bytes, result.monotonic ? 1U : 0U);
    return sha256_hex(bytes);
}

void emit_trials(std::ostringstream& output, const SolveResult& result) {
    output << '[';
    for (std::size_t index = 0; index < result.trials.size(); ++index) {
        if (index != 0U) output << ',';
        const TrialRecord& value = result.trials[index];
        output << "{\"outer\":" << index
               << ",\"objective\":" << value.objective
               << ",\"gradient_norm\":" << value.gradient_norm
               << ",\"step_norm\":" << value.step_norm
               << ",\"ared\":" << value.actual_reduction
               << ",\"pred\":" << value.predicted_reduction
               << ",\"rho\":" << value.ratio
               << ",\"radius_before\":" << value.radius_before
               << ",\"radius_after\":" << value.radius_after
               << ",\"reason\":\"" << value.reason << "\""
               << ",\"iterations\":" << value.iterations
               << ",\"hvp_calls\":" << value.hvp_calls
               << ",\"boundary\":" << (value.boundary ? "true" : "false")
               << ",\"accepted\":" << (value.accepted ? "true" : "false")
               << ",\"active_count\":" << value.active_count
               << ",\"active_root\":\"" << value.active_root << "\""
               << ",\"state_root\":\"" << value.state_root << "\"}";
    }
    output << ']';
}

void emit_work(std::ostringstream& output, const ControllerWork& work) {
    output << "{\"evaluator_calls\":" << work.evaluator_calls
           << ",\"cg_matvecs\":" << work.cg_matvecs
           << ",\"prediction_matvecs\":" << work.prediction_matvecs
           << ",\"dot_products\":" << work.dot_products
           << ",\"boundary_intersections\":" << work.boundary_intersections
           << ",\"accepted_trials\":" << work.accepted_trials
           << ",\"rejected_trials\":" << work.rejected_trials
           << ",\"radius_shrinks\":" << work.radius_shrinks
           << ",\"radius_expands\":" << work.radius_expands
           << ",\"continuous_state_uploads\":"
           << work.continuous_state_uploads
           << ",\"trial_state_f32_rounds\":"
           << work.trial_state_f32_rounds
           << ",\"topology_pair_checks\":" << work.topology_pair_checks
           << ",\"assembly\":{\"owner_sort_comparisons\":"
           << work.assembly.owner_sort_comparisons
           << ",\"graph_distance_predicates\":"
           << work.assembly.graph_distance_predicates
           << ",\"density_kernel_evaluations\":"
           << work.assembly.density_kernel_evaluations
           << ",\"energy_pair_visits\":"
           << work.assembly.energy_pair_visits
           << ",\"gradient_pair_visits\":"
           << work.assembly.gradient_pair_visits
           << ",\"active_pressure_centers\":"
           << work.assembly.active_pressure_centers
           << ",\"pressure_outer_products\":"
           << work.assembly.pressure_outer_products
           << ",\"pressure_geometric_blocks\":"
           << work.assembly.pressure_geometric_blocks
           << ",\"viscosity_curvature_blocks\":"
           << work.assembly.viscosity_curvature_blocks
           << ",\"surface_curvature_blocks\":"
           << work.assembly.surface_curvature_blocks
           << ",\"dense_entries_written\":"
           << work.assembly.dense_entries_written
           << ",\"hvp_products\":" << work.assembly.hvp_products
           << ",\"host_device_scalar_transfers\":"
           << work.assembly.host_device_scalar_transfers
           << ",\"compensated_additions\":"
           << work.assembly.compensated_additions
           << ",\"compensation_initializations\":"
           << work.assembly.compensation_initializations << "}}";
}

int run() {
    const AssemblyProfile profile;
    const AssemblyFixture fixture = make_combined(false);
    const AssemblyFixture permuted = make_combined(true);
    const std::vector<double> start = initial_state(fixture);
    const TopologySignature frozen_topology = topology_signature(start, profile);
    const AssemblyResult initial_reference = evaluate_reference_assembly_at(
        profile, fixture, start);
    const AssemblyResult initial_candidate = evaluate_gpu_assembly_at(
        profile, fixture, start, AssemblyVariant::Corrected);
    const bool apparatus = initial_reference.failure == AssemblyFailure::None
        && initial_candidate.failure == AssemblyFailure::None
        && same_assembly_topology(initial_reference, initial_candidate)
        && initial_reference.pressure_active == initial_candidate.pressure_active;

    const Evaluator reference_evaluator = [&profile, &fixture](
        const std::vector<double>& state) {
        return evaluate_reference_assembly_at(profile, fixture, state);
    };
    const Evaluator candidate_evaluator = [&profile, &fixture](
        const std::vector<double>& state) {
        return evaluate_gpu_assembly_at(
            profile, fixture, state, AssemblyVariant::Corrected);
    };
    const Evaluator permuted_evaluator = [&profile, &permuted](
        const std::vector<double>& state) {
        return evaluate_gpu_assembly_at(
            profile, permuted, state, AssemblyVariant::Corrected);
    };

    const SolveResult reference = solve_prefix(
        start, profile, frozen_topology, reference_evaluator, false, false);
    const SolveResult candidate = solve_prefix(
        start, profile, frozen_topology, candidate_evaluator, true, false);
    const SolveResult negative = solve_prefix(
        start, profile, frozen_topology, candidate_evaluator, true, true);
    const SolveResult permuted_result = solve_prefix(
        start, profile, frozen_topology, permuted_evaluator, true, false);

    const bool route_match = same_controller_route(reference, candidate);
    const double drift_um = maximum_trace_drift_um(reference, candidate);
    const double reference_reduction = reference.initial_objective
        - reference.final_objective;
    const double candidate_reduction = candidate.initial_objective
        - candidate.final_objective;
    const double reduction_difference = std::abs(
        candidate_reduction - reference_reduction)
        / std::max(std::abs(reference_reduction), 1.0);
    const bool permutation_exact = solve_root(candidate) == solve_root(permuted_result);
    const bool negative_rejected = signature_root(negative) != signature_root(candidate)
        || maximum_particle_difference_um(negative.state, candidate.state)
            > POSITION_GATE_UM;
    const bool positive = apparatus && reference.finite && candidate.finite
        && reference.topology_safe && candidate.topology_safe
        && reference.monotonic && candidate.monotonic
        && reference.trials.size() == static_cast<std::size_t>(OUTER_TRIALS)
        && candidate.trials.size() == static_cast<std::size_t>(OUTER_TRIALS)
        && reference.work.accepted_trials > 0U
        && candidate.work.accepted_trials > 0U
        && reference.minimum_radius >= MINIMUM_RADIUS
        && candidate.minimum_radius >= MINIMUM_RADIUS
        && route_match && drift_um <= POSITION_GATE_UM
        && reduction_difference <= 1.0e-3
        && negative_rejected && permutation_exact;

    std::string result = "INCONCLUSIVE";
    if (positive) {
        result = "GPU_ASSEMBLED_TRUST_PREFIX_SUPPORTED";
    } else if (apparatus && reference.finite && candidate.finite
        && reference.topology_safe && candidate.topology_safe) {
        if (!route_match && drift_um <= POSITION_GATE_UM) {
            bool reduction_sign_boundary = false;
            const std::size_t count = std::min(
                reference.trials.size(), candidate.trials.size());
            for (std::size_t index = 0; index < count; ++index) {
                reduction_sign_boundary = reduction_sign_boundary
                    || ((reference.trials[index].actual_reduction > 0.0)
                        != (candidate.trials[index].actual_reduction > 0.0));
            }
            result = reduction_sign_boundary
                ? "ENERGY_ROUNDING_GLOBALIZATION_BOUNDARY"
                : "KRYLOV_CONTROL_FLOW_BOUNDARY";
        } else if (drift_um > POSITION_GATE_UM
            || !candidate.monotonic || reduction_difference > 1.0e-3) {
            result = "F32_SOLVER_CONSEQUENCE_MATERIAL";
        }
    }

    std::string semantic = "nextengine.nonlocal.ncga4.semantic.v1\0";
    semantic += result;
    semantic += solve_root(reference);
    semantic += solve_root(candidate);
    semantic += solve_root(negative);
    append_double(semantic, drift_um);
    append_double(semantic, reduction_difference);
    append_u64(semantic, route_match ? 1U : 0U);
    append_u64(semantic, negative_rejected ? 1U : 0U);
    append_u64(semantic, permutation_exact ? 1U : 0U);

    std::ostringstream output;
    output << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.ncga4.v1\""
           << ",\"result\":\"" << result << "\""
           << ",\"apparatus_valid\":" << (apparatus ? "true" : "false")
           << ",\"route_match\":" << (route_match ? "true" : "false")
           << ",\"permutation_exact\":"
           << (permutation_exact ? "true" : "false")
           << ",\"negative_rejected\":"
           << (negative_rejected ? "true" : "false")
           << ",\"maximum_state_drift_um\":" << drift_um
           << ",\"reduction_difference\":" << reduction_difference
           << ",\"reference\":{\"initial_objective\":"
           << reference.initial_objective << ",\"final_objective\":"
           << reference.final_objective << ",\"signature_root\":\""
           << signature_root(reference) << "\",\"state_root\":\""
           << state_root(reference.state) << "\",\"work_root\":\""
           << work_root(reference.work) << "\",\"work\":";
    emit_work(output, reference.work);
    output << ",\"trials\":";
    emit_trials(output, reference);
    output << "},\"strict_f32\":{\"initial_objective\":"
           << candidate.initial_objective << ",\"final_objective\":"
           << candidate.final_objective << ",\"signature_root\":\""
           << signature_root(candidate) << "\",\"state_root\":\""
           << state_root(candidate.state) << "\",\"work_root\":\""
           << work_root(candidate.work) << "\",\"work\":";
    emit_work(output, candidate.work);
    output << ",\"trials\":";
    emit_trials(output, candidate);
    output << "},\"negative\":{\"signature_root\":\""
           << signature_root(negative) << "\",\"state_root\":\""
           << state_root(negative.state) << "\"}"
           << ",\"semantic_root\":\"" << sha256_hex(semantic) << "\""
           << ",\"gpu\":" << gpu_assembly_environment_json()
           << ",\"claim\":\"tiny_fixed_graph_gpu_assembled_host_controlled_prefix_only\""
           << ",\"full_solver_authority\":false"
           << ",\"trajectory_authority\":false"
           << ",\"performance_authority\":false"
           << ",\"runtime_authority\":false}\n";
    std::cout << output.str();
    return positive ? 0 : 1;
}

} // namespace
} // namespace nextengine::nonlocal::gpu_assembly_audit

int main() {
    try {
        return nextengine::nonlocal::gpu_assembly_audit::run();
    } catch (const std::exception& error) {
        std::cerr << "NCGA4 failure: " << error.what() << '\n';
        return 2;
    }
}
