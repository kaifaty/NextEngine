#include "corrected_cuda_full_step.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <limits>
#include <numeric>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace {

#ifndef NCGP11_CONTRACT_ROOT
#define NCGP11_CONTRACT_ROOT "unconfigured"
#endif
#ifndef NCGP11_SOURCE_ROOT
#define NCGP11_SOURCE_ROOT "unconfigured"
#endif
#ifndef NCGP11_SOURCE_COMMIT
#define NCGP11_SOURCE_COMMIT "unconfigured"
#endif
#ifndef NCGP11_SOURCE_TREE
#define NCGP11_SOURCE_TREE "unconfigured"
#endif
#ifndef NCGP11_COMPILER_FLAGS
#define NCGP11_COMPILER_FLAGS "unconfigured"
#endif

using namespace nextengine::nonlocal::gpu_full_step;

constexpr std::uint32_t kHvpBudget = 128U;
constexpr std::size_t kConditioningSamples = 16U;
constexpr std::size_t kWarmupSamples = 8U;
constexpr std::size_t kMeasuredSamples = 32U;

struct ProfileSpec {
    std::string_view id;
    std::uint32_t nx;
    std::uint32_t ny;
    std::uint32_t nz;
};

constexpr std::array<ProfileSpec, 3U> kProfiles{{
    {"cost-4k", 20U, 20U, 10U},
    {"cost-16k", 40U, 20U, 20U},
    {"cost-50k", 50U, 40U, 25U},
}};

struct TimingSample {
    double total_ms = 0.0;
    double graph_ms = 0.0;
    double evaluation_ms = 0.0;
    double hvp_ms = 0.0;
    double remainder_ms = 0.0;
    double reset_upload_ms = 0.0;
};

struct Distribution {
    double minimum = 0.0;
    double mean = 0.0;
    double p50 = 0.0;
    double p95 = 0.0;
    double p99 = 0.0;
    double maximum = 0.0;
};

struct Invocation {
    NonlocalGpuFailure upload_failure = NonlocalGpuFailure::None;
    NonlocalGpuStepResult step;
    TimingSample timing;
};

struct BenchmarkResult {
    std::string classification;
    std::string failed_phase;
    std::size_t failed_index = 0U;
    NonlocalGpuFailure failure = NonlocalGpuFailure::None;
    NonlocalGpuStepResult representative;
    std::vector<TimingSample> samples;
    std::vector<std::string> work_roots;
    std::vector<std::string> step_roots;
};

std::string file_root(const std::string& path) {
    std::ifstream stream(path, std::ios::binary);
    if (!stream) return {};
    std::ostringstream bytes;
    bytes << stream.rdbuf();
    if (!stream.good() && !stream.eof()) return {};
    return nextengine::nonlocal::sha256_hex(bytes.str());
}

std::string binary_root() { return file_root("/proc/self/exe"); }

std::string json_escape(std::string_view value) {
    std::ostringstream output;
    for (const char character : value) {
        switch (character) {
        case '\\': output << "\\\\"; break;
        case '"': output << "\\\""; break;
        case '\n': output << "\\n"; break;
        case '\r': output << "\\r"; break;
        case '\t': output << "\\t"; break;
        default: output << character; break;
        }
    }
    return output.str();
}

const ProfileSpec* find_profile(std::string_view id) {
    const auto iterator = std::find_if(kProfiles.begin(), kProfiles.end(),
        [&](const ProfileSpec& candidate) {
            return candidate.id == id || candidate.id.substr(5U) == id;
        });
    return iterator == kProfiles.end() ? nullptr : &*iterator;
}

std::size_t nearest_rank_index(
    std::size_t count, std::size_t numerator, std::size_t denominator) {
    if (count == 0U || numerator == 0U || numerator > denominator) {
        throw std::invalid_argument("invalid nearest-rank request");
    }
    return (numerator * count + denominator - 1U) / denominator - 1U;
}

Distribution distribution(std::vector<double> values) {
    if (values.size() != kMeasuredSamples) {
        throw std::invalid_argument("incomplete timing distribution");
    }
    const double sum = std::accumulate(values.begin(), values.end(), 0.0);
    std::sort(values.begin(), values.end());
    return {values.front(), sum / static_cast<double>(values.size()),
        values[nearest_rank_index(values.size(), 50U, 100U)],
        values[nearest_rank_index(values.size(), 95U, 100U)],
        values[nearest_rank_index(values.size(), 99U, 100U)], values.back()};
}

double finite_remainder(const NonlocalGpuTimings& timing) {
    const double accounted = static_cast<double>(timing.graph_ms)
        + static_cast<double>(timing.density_energy_gradient_ms)
        + static_cast<double>(timing.hvp_ms);
    return std::max(static_cast<double>(timing.total_ms) - accounted, 0.0);
}

Invocation invoke_once(NonlocalGpuWorkspace& workspace,
    const std::vector<NonlocalGpuSample>& state,
    const std::vector<NonlocalGpuGhost>& ghosts) {
    Invocation result;
    const auto reset_start = std::chrono::steady_clock::now();
    result.upload_failure = workspace.upload(state, ghosts, true);
    const auto reset_stop = std::chrono::steady_clock::now();
    result.timing.reset_upload_ms = std::chrono::duration<double, std::milli>(
        reset_stop - reset_start).count();
    if (result.upload_failure != NonlocalGpuFailure::None) {
        result.step.failure = result.upload_failure;
        return result;
    }
    result.step = workspace.step(kHvpBudget,
        NonlocalGpuSolverProfile::Unpreconditioned,
        NonlocalGpuVariant::CompensatedScalePressureF64, false, true, false);
    result.timing.total_ms = result.step.timing.total_ms;
    result.timing.graph_ms = result.step.timing.graph_ms;
    result.timing.evaluation_ms =
        result.step.timing.density_energy_gradient_ms;
    result.timing.hvp_ms = result.step.timing.hvp_ms;
    result.timing.remainder_ms = finite_remainder(result.step.timing);
    return result;
}

bool same_work_shape(const NonlocalGpuStepResult& lhs,
    const NonlocalGpuStepResult& rhs) {
    return lhs.failure == rhs.failure
        && lhs.maximum_directed_pairs == rhs.maximum_directed_pairs
        && lhs.maximum_degree == rhs.maximum_degree
        && lhs.hvp_budget == rhs.hvp_budget && lhs.hvp_used == rhs.hvp_used
        && lhs.outer_trials == rhs.outer_trials
        && lhs.work.graph_builds == rhs.work.graph_builds
        && lhs.work.hvp_applications == rhs.work.hvp_applications
        && lhs.work.accepted_trials == rhs.work.accepted_trials
        && lhs.work.rejected_trials == rhs.work.rejected_trials
        && lhs.work.allocated_device_bytes == rhs.work.allocated_device_bytes;
}

BenchmarkResult run_benchmark(NonlocalGpuWorkspace& workspace,
    const NonlocalGpuProfile& profile,
    const std::string& input_root,
    const std::vector<NonlocalGpuSample>& state,
    const std::vector<NonlocalGpuGhost>& ghosts) {
    BenchmarkResult result;
    const auto run_phase = [&](std::string_view phase, std::size_t count,
                               bool retain) -> bool {
        for (std::size_t index = 0U; index < count; ++index) {
            Invocation invocation = invoke_once(workspace, state, ghosts);
            if (invocation.upload_failure != NonlocalGpuFailure::None
                || invocation.step.failure != NonlocalGpuFailure::None) {
                result.classification =
                    "INVALID_PHYSICS_CAPACITY_OR_WORK_REFUTED";
                result.failed_phase = std::string(phase);
                result.failed_index = index;
                result.failure = invocation.step.failure;
                result.representative = std::move(invocation.step);
                return false;
            }
            if (result.representative.hvp_budget == 0U) {
                result.representative = invocation.step;
            } else if (!same_work_shape(result.representative, invocation.step)) {
                result.classification = "INVALID_PHYSICS_COST_INCONCLUSIVE";
                result.failed_phase = std::string(phase) + "-work-shape";
                result.failed_index = index;
                result.failure = NonlocalGpuFailure::InvalidState;
                result.representative = std::move(invocation.step);
                return false;
            }
            if (retain) {
                result.samples.push_back(invocation.timing);
                result.work_roots.push_back(
                    step_work_semantic_root(profile, invocation.step));
                result.step_roots.push_back(
                    step_semantic_root(profile, input_root, invocation.step));
            }
        }
        return true;
    };

    if (!run_phase("conditioning", kConditioningSamples, false)
        || !run_phase("warmup", kWarmupSamples, false)
        || !run_phase("measured", kMeasuredSamples, true)) {
        return result;
    }
    result.classification = "INVALID_PHYSICS_COST_COMPLETE_PROCESS";
    return result;
}

std::vector<double> select_values(const std::vector<TimingSample>& samples,
    double TimingSample::*member) {
    std::vector<double> values;
    values.reserve(samples.size());
    for (const TimingSample& sample : samples) values.push_back(sample.*member);
    return values;
}

void append_samples(std::ostream& output, const std::vector<TimingSample>& samples,
    double TimingSample::*member) {
    output << '[';
    for (std::size_t index = 0U; index < samples.size(); ++index) {
        if (index != 0U) output << ',';
        output << samples[index].*member;
    }
    output << ']';
}

void append_roots(std::ostream& output, const std::vector<std::string>& roots) {
    output << '[';
    for (std::size_t index = 0U; index < roots.size(); ++index) {
        if (index != 0U) output << ',';
        output << '"' << roots[index] << '"';
    }
    output << ']';
}

void append_distribution(std::ostream& output, const Distribution& value) {
    output << "{\"min\":" << value.minimum << ",\"mean\":" << value.mean
           << ",\"p50\":" << value.p50 << ",\"p95\":" << value.p95
           << ",\"p99\":" << value.p99 << ",\"max\":" << value.maximum
           << '}';
}

std::string result_material(const ProfileSpec& spec,
    const std::string& mode,
    const std::string& executable_root,
    const std::string& input_root,
    const std::string& environment,
    const BenchmarkResult& result) {
    std::ostringstream material;
    material << std::setprecision(std::numeric_limits<double>::max_digits10)
             << "NCGP11-invalid-physics-cost-only-v1\n"
             << NCGP11_CONTRACT_ROOT << '\n' << NCGP11_SOURCE_ROOT << '\n'
             << NCGP11_SOURCE_COMMIT << '\n' << NCGP11_SOURCE_TREE << '\n'
             << executable_root << '\n' << input_root << '\n' << environment
             << '\n' << spec.id << ':' << spec.nx << ':' << spec.ny << ':'
             << spec.nz << '\n' << mode << '\n' << result.classification << ':'
             << static_cast<std::uint32_t>(result.failure) << ':'
             << result.failed_phase << ':' << result.failed_index << '\n'
             << result.representative.maximum_directed_pairs << ':'
             << result.representative.maximum_degree << ':'
             << result.representative.hvp_budget << ':'
             << result.representative.hvp_used << ':'
             << result.representative.outer_trials << ':'
             << result.representative.work.allocated_device_bytes << '\n';
    for (std::size_t index = 0U; index < result.samples.size(); ++index) {
        const TimingSample& sample = result.samples[index];
        material << index << ':' << sample.total_ms << ':' << sample.graph_ms
                 << ':' << sample.evaluation_ms << ':' << sample.hvp_ms << ':'
                 << sample.remainder_ms << ':' << sample.reset_upload_ms << ':'
                 << result.work_roots[index] << ':' << result.step_roots[index]
                 << '\n';
    }
    return material.str();
}

void emit_result(const ProfileSpec& spec,
    const std::string& mode,
    const std::string& command,
    const std::string& executable_root,
    const std::string& input_root,
    const std::string& environment,
    std::size_t ghost_count,
    std::size_t allocated_bytes,
    const BenchmarkResult& result) {
    const std::string root = nextengine::nonlocal::sha256_hex(result_material(
        spec, mode, executable_root, input_root, environment, result));
    std::cout << std::setprecision(std::numeric_limits<double>::max_digits10)
              << "{\"research_id\":\"NCGP11\""
              << ",\"schema\":\"nonlocal-water-invalid-physics-cost-only-v1\""
              << ",\"status\":\"" << result.classification << "\""
              << ",\"claim_ceiling\":\"INVALID_PHYSICS_COST_ONLY_NO_WATER_QUALITY_CLAIM\""
              << ",\"mode\":\"" << mode << "\""
              << ",\"profile\":\"" << spec.id << "\""
              << ",\"lattice\":[" << spec.nx << ',' << spec.ny << ','
              << spec.nz << "]"
              << ",\"dynamic_samples\":"
              << static_cast<std::uint64_t>(spec.nx) * spec.ny * spec.nz
              << ",\"ghost_samples\":" << ghost_count
              << ",\"contract_root\":\"" << NCGP11_CONTRACT_ROOT << "\""
              << ",\"source_root\":\"" << NCGP11_SOURCE_ROOT << "\""
              << ",\"source_commit\":\"" << NCGP11_SOURCE_COMMIT << "\""
              << ",\"source_tree\":\"" << NCGP11_SOURCE_TREE << "\""
              << ",\"binary_root\":\"" << executable_root << "\""
              << ",\"input_root\":\"" << input_root << "\""
              << ",\"result_root\":\"" << root << "\""
              << ",\"compiler_flags\":\""
              << json_escape(NCGP11_COMPILER_FLAGS) << "\""
              << ",\"command\":\"" << json_escape(command) << "\""
              << ",\"environment\":" << environment
              << ",\"variant\":19,\"solver_profile\":0,\"hvp_budget\":"
              << kHvpBudget
              << ",\"allocated_device_bytes\":" << allocated_bytes
              << ",\"failure\":"
              << static_cast<std::uint32_t>(result.failure)
              << ",\"failed_phase\":\"" << json_escape(result.failed_phase)
              << "\",\"failed_index\":" << result.failed_index
              << ",\"work\":{\"directed_pairs\":"
              << result.representative.maximum_directed_pairs
              << ",\"maximum_degree\":" << result.representative.maximum_degree
              << ",\"hvp_used\":" << result.representative.hvp_used
              << ",\"outer_trials\":" << result.representative.outer_trials
              << ",\"accepted_trials\":"
              << result.representative.work.accepted_trials
              << ",\"rejected_trials\":"
              << result.representative.work.rejected_trials
              << ",\"graph_builds\":"
              << result.representative.work.graph_builds
              << ",\"hvp_applications\":"
              << result.representative.work.hvp_applications << '}';

    if (result.samples.size() == kMeasuredSamples) {
        const Distribution total = distribution(select_values(
            result.samples, &TimingSample::total_ms));
        const Distribution graph = distribution(select_values(
            result.samples, &TimingSample::graph_ms));
        const Distribution evaluation = distribution(select_values(
            result.samples, &TimingSample::evaluation_ms));
        const Distribution hvp = distribution(select_values(
            result.samples, &TimingSample::hvp_ms));
        const Distribution remainder = distribution(select_values(
            result.samples, &TimingSample::remainder_ms));
        const Distribution reset = distribution(select_values(
            result.samples, &TimingSample::reset_upload_ms));
        std::cout << ",\"sampling\":{\"conditioning\":"
                  << kConditioningSamples << ",\"warmup\":" << kWarmupSamples
                  << ",\"measured\":" << kMeasuredSamples
                  << ",\"nearest_rank_indices\":[15,30,31]}"
                  << ",\"distribution_ms\":{\"total\":";
        append_distribution(std::cout, total);
        std::cout << ",\"graph\":";
        append_distribution(std::cout, graph);
        std::cout << ",\"density_energy_gradient\":";
        append_distribution(std::cout, evaluation);
        std::cout << ",\"hvp\":";
        append_distribution(std::cout, hvp);
        std::cout << ",\"step_remainder\":";
        append_distribution(std::cout, remainder);
        std::cout << ",\"reset_upload_integration_tax\":";
        append_distribution(std::cout, reset);
        std::cout << "},\"raw_ms\":{\"total\":";
        append_samples(std::cout, result.samples, &TimingSample::total_ms);
        std::cout << ",\"graph\":";
        append_samples(std::cout, result.samples, &TimingSample::graph_ms);
        std::cout << ",\"density_energy_gradient\":";
        append_samples(std::cout, result.samples, &TimingSample::evaluation_ms);
        std::cout << ",\"hvp\":";
        append_samples(std::cout, result.samples, &TimingSample::hvp_ms);
        std::cout << ",\"step_remainder\":";
        append_samples(std::cout, result.samples, &TimingSample::remainder_ms);
        std::cout << ",\"reset_upload_integration_tax\":";
        append_samples(
            std::cout, result.samples, &TimingSample::reset_upload_ms);
        std::cout << "},\"work_roots\":";
        append_roots(std::cout, result.work_roots);
        std::cout << ",\"step_roots\":";
        append_roots(std::cout, result.step_roots);
    } else if (result.samples.size() == 1U) {
        const TimingSample& sample = result.samples.front();
        std::cout << ",\"probe_timing_ms\":{\"total\":" << sample.total_ms
                  << ",\"graph\":" << sample.graph_ms
                  << ",\"density_energy_gradient\":"
                  << sample.evaluation_ms << ",\"hvp\":" << sample.hvp_ms
                  << ",\"step_remainder\":" << sample.remainder_ms
                  << ",\"reset_upload_integration_tax\":"
                  << sample.reset_upload_ms << "},\"work_roots\":";
        append_roots(std::cout, result.work_roots);
        std::cout << ",\"step_roots\":";
        append_roots(std::cout, result.step_roots);
    }
    std::cout << ",\"window_includes\":[\"dynamic_graph\",\"density_active_energy_gradient\",\"all_hvp_reductions\",\"trust_region_control\",\"boundary_contact\",\"integration_publication\"]"
              << ",\"window_excludes\":[\"process_startup\",\"workspace_allocation\",\"reset_upload\",\"cpu_oracle\",\"trajectory_observers\",\"json\",\"renderer\",\"physx\"]}"
              << '\n';
}

BenchmarkResult run_probe(NonlocalGpuWorkspace& workspace,
    const NonlocalGpuProfile& profile,
    const std::string& input_root,
    const std::vector<NonlocalGpuSample>& state,
    const std::vector<NonlocalGpuGhost>& ghosts) {
    BenchmarkResult result;
    Invocation invocation = invoke_once(workspace, state, ghosts);
    result.representative = std::move(invocation.step);
    result.failure = result.representative.failure;
    if (invocation.upload_failure != NonlocalGpuFailure::None
        || result.failure != NonlocalGpuFailure::None) {
        result.classification = "INVALID_PHYSICS_CAPACITY_OR_WORK_REFUTED";
        result.failed_phase = "probe";
        return result;
    }
    result.classification = "INVALID_PHYSICS_PROBE_ONLY";
    result.samples.push_back(invocation.timing);
    result.work_roots.push_back(step_work_semantic_root(profile,
        result.representative));
    result.step_roots.push_back(step_semantic_root(profile, input_root,
        result.representative));
    return result;
}

int execute_profile(const ProfileSpec& spec,
    bool benchmark,
    const std::string& command,
    const std::string& executable_root) {
    const NonlocalGpuProfile profile = nonlocal_water_corrected_profile();
    const auto state = make_lattice_state(
        profile, spec.nx, spec.ny, spec.nz, false, false);
    const auto ghosts = canonicalize_ghosts_binary32(make_basin_ghosts(profile));
    const NonlocalGpuFailure admission = validate_nonlocal_input(
        profile, state, ghosts);
    if (admission != NonlocalGpuFailure::None) {
        BenchmarkResult result;
        result.classification = "INVALID_PHYSICS_CAPACITY_OR_WORK_REFUTED";
        result.failure = admission;
        result.representative.failure = admission;
        result.failed_phase = "host-admission";
        emit_result(spec, benchmark ? "benchmark" : "probe", command,
            executable_root, input_semantic_root(profile, state, ghosts), "{}",
            ghosts.size(), 0U, result);
        return 37;
    }
    NonlocalGpuWorkspace workspace(profile);
    const std::string environment = workspace.environment_json();
    const std::string input_root = input_semantic_root(profile, state, ghosts);
    BenchmarkResult result = benchmark
        ? run_benchmark(workspace, profile, input_root, state, ghosts)
        : run_probe(workspace, profile, input_root, state, ghosts);
    emit_result(spec, benchmark ? "benchmark" : "probe", command,
        executable_root, input_root, environment, ghosts.size(),
        workspace.allocated_device_bytes(), result);
    if (result.classification == "INVALID_PHYSICS_CAPACITY_OR_WORK_REFUTED") {
        return 37;
    }
    return result.classification == "INVALID_PHYSICS_COST_INCONCLUSIVE" ? 4 : 0;
}

} // namespace

int main(int argc, char** argv) {
    const std::string executable_root = binary_root();
    if (executable_root.size() != 64U
        || std::string_view(NCGP11_CONTRACT_ROOT).size() != 64U
        || std::string_view(NCGP11_SOURCE_ROOT).size() != 64U
        || std::string_view(NCGP11_SOURCE_COMMIT).size() != 40U
        || std::string_view(NCGP11_SOURCE_TREE).size() != 40U) {
        std::cerr << "NCGP11 identity closure failed\n";
        return 4;
    }
    std::ostringstream command;
    for (int index = 0; index < argc; ++index) {
        if (index != 0) command << ' ';
        command << argv[index];
    }
    try {
        if (argc == 2 && std::string_view(argv[1]) == "--probe-all") {
            int status = 0;
            for (const ProfileSpec& spec : kProfiles) {
                const int current = execute_profile(
                    spec, false, command.str(), executable_root);
                if (current != 0 && status == 0) status = current;
            }
            return status;
        }
        if (argc == 3 && std::string_view(argv[1]) == "--benchmark") {
            const ProfileSpec* spec = find_profile(argv[2]);
            if (spec == nullptr) {
                std::cerr << "unknown profile\n";
                return 2;
            }
            return execute_profile(*spec, true, command.str(), executable_root);
        }
        std::cerr << "usage: " << argv[0]
                  << " --probe-all | --benchmark {4k|16k|50k}\n";
        return 2;
    } catch (const std::exception& error) {
        std::cerr << "NCGP11 apparatus failure: " << error.what() << '\n';
        return 4;
    }
}
