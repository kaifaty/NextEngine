#include "corrected_cuda_terms.hpp"

#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

namespace nextengine::nonlocal::gpu_audit {
namespace {

constexpr double MIXED_LIMIT = 2.0e-5;
constexpr double ZERO_LIMIT = 2.0e-7;
constexpr double ENERGY_DERIVATIVE_LIMIT = 2.0e-5;
constexpr int COLD_REPEATS = 10;

struct NamedInput {
    const char* name;
    TermInput input;
};

Vec3Input add(Vec3Input lhs, Vec3Input rhs) {
    return {lhs.x + rhs.x, lhs.y + rhs.y, lhs.z + rhs.z};
}

Vec3Input multiply(double scalar, Vec3Input value) {
    return {scalar * value.x, scalar * value.y, scalar * value.z};
}

double length(Vec3Input value) {
    return std::sqrt(value.x * value.x + value.y * value.y + value.z * value.z);
}

Vec3Input normalized(Vec3Input value) {
    return multiply(1.0 / length(value), value);
}

std::vector<NamedInput> make_inputs(const AuditProfile& profile) {
    std::vector<NamedInput> inputs;
    TermInput kernel_inner;
    kernel_inner.kind = TermKind::KernelGradient;
    kernel_inner.parameter = 0.03;
    inputs.push_back({"kernel_inner", kernel_inner});

    TermInput kernel_outer = kernel_inner;
    kernel_outer.parameter = 0.105;
    inputs.push_back({"kernel_outer", kernel_outer});

    TermInput compression;
    compression.kind = TermKind::Compression;
    compression.first = {0.0, 0.0, 0.0};
    compression.second = {0.061, 0.017, -0.009};
    compression.parameter = 1.1;
    inputs.push_back({"compression_above_rest", compression});
    compression.parameter = 0.9;
    inputs.push_back({"compression_below_rest", compression});

    TermInput viscosity;
    viscosity.reference_first = {-0.025, 0.015, -0.01};
    viscosity.reference_second = {0.055, -0.012, 0.018};
    const Vec3Input first_velocity{1.2, 0.4, -0.1};
    const Vec3Input second_velocity{-0.2, 0.1, 0.5};
    viscosity.first = add(
        viscosity.reference_first,
        multiply(profile.time_step, first_velocity));
    viscosity.second = add(
        viscosity.reference_second,
        multiply(profile.time_step, second_velocity));
    viscosity.kind = TermKind::BulkViscosity;
    inputs.push_back({"bulk_viscosity", viscosity});
    viscosity.kind = TermKind::ShearViscosity;
    inputs.push_back({"shear_viscosity", viscosity});

    const Vec3Input axis = normalized({0.91, -0.21, 0.356});
    for (const auto& surface : std::array<std::pair<const char*, double>, 3>{
             std::pair{"surface_repulsive", 0.8},
             std::pair{"surface_attractive", 1.7},
             std::pair{"surface_outside", 3.2},
         }) {
        TermInput input;
        input.kind = TermKind::Surface;
        input.first = {0.0, 0.0, 0.0};
        input.second = multiply(surface.second * profile.spacing, axis);
        input.parameter = surface.second;
        inputs.push_back({surface.first, input});
    }
    return inputs;
}

std::vector<TermInput> raw_inputs(const std::vector<NamedInput>& inputs) {
    std::vector<TermInput> result;
    result.reserve(inputs.size());
    for (const NamedInput& input : inputs) {
        result.push_back(input.input);
    }
    return result;
}

double mixed_error(double candidate, double reference) {
    return std::abs(candidate - reference)
        / std::max(1.0, std::abs(reference));
}

bool component_passes(double candidate, double reference) {
    if (reference == 0.0) {
        return std::abs(candidate) <= ZERO_LIMIT;
    }
    return mixed_error(candidate, reference) <= MIXED_LIMIT;
}

double maximum_output_error(
    const GpuTermOutput& candidate,
    const ReferenceTermOutput& reference) {
    double maximum = mixed_error(candidate.scalar, reference.scalar);
    const double first_reference[3] = {
        reference.first_force.x,
        reference.first_force.y,
        reference.first_force.z,
    };
    const double second_reference[3] = {
        reference.second_force.x,
        reference.second_force.y,
        reference.second_force.z,
    };
    for (int component = 0; component < 3; ++component) {
        maximum = std::max(maximum,
            mixed_error(candidate.first_force[component], first_reference[component]));
        maximum = std::max(maximum,
            mixed_error(candidate.second_force[component], second_reference[component]));
    }
    return maximum;
}

bool output_passes(
    const GpuTermOutput& candidate,
    const ReferenceTermOutput& reference) {
    if (candidate.finite == 0 || !reference.finite
        || !component_passes(candidate.scalar, reference.scalar)) {
        return false;
    }
    const double first_reference[3] = {
        reference.first_force.x,
        reference.first_force.y,
        reference.first_force.z,
    };
    const double second_reference[3] = {
        reference.second_force.x,
        reference.second_force.y,
        reference.second_force.z,
    };
    for (int component = 0; component < 3; ++component) {
        if (!component_passes(
                candidate.first_force[component], first_reference[component])
            || !component_passes(
                candidate.second_force[component], second_reference[component])) {
            return false;
        }
        const double closure = static_cast<double>(candidate.first_force[component])
            + static_cast<double>(candidate.second_force[component]);
        if (std::abs(closure) > ZERO_LIMIT) {
            return false;
        }
    }
    return true;
}

double energy_derivative_error(
    const GpuTermOutput& candidate,
    const EnergyDerivativeOutput& reference) {
    const double projection = candidate.first_force[0] * reference.direction.x
        + candidate.first_force[1] * reference.direction.y
        + candidate.first_force[2] * reference.direction.z;
    return std::abs(-projection - reference.directional_derivative)
        / std::max(1.0, std::abs(reference.directional_derivative));
}

double force_projection(
    const GpuTermOutput& candidate,
    const EnergyDerivativeOutput& reference) {
    return candidate.first_force[0] * reference.direction.x
        + candidate.first_force[1] * reference.direction.y
        + candidate.first_force[2] * reference.direction.z;
}

bool energy_derivative_passes(
    const GpuTermOutput& candidate,
    const EnergyDerivativeOutput& reference) {
    return candidate.finite != 0 && reference.finite
        && energy_derivative_error(candidate, reference)
            <= ENERGY_DERIVATIVE_LIMIT;
}

std::string fixture_material(
    const AuditProfile& profile,
    const std::vector<NamedInput>& inputs) {
    std::ostringstream output;
    output << std::setprecision(17) << profile.horizon << '|' << profile.spacing
           << '|' << profile.mass << '|' << profile.time_step << '|'
           << profile.rest_density << '|' << profile.kappa << '|'
           << profile.lambda << '|' << profile.mu << '|' << profile.gamma;
    for (const NamedInput& named : inputs) {
        const TermInput& input = named.input;
        output << '|' << named.name << ':' << static_cast<std::uint32_t>(input.kind)
               << ':' << input.first.x << ':' << input.first.y << ':' << input.first.z
               << ':' << input.second.x << ':' << input.second.y << ':' << input.second.z
               << ':' << input.reference_first.x << ':' << input.reference_first.y
               << ':' << input.reference_first.z << ':' << input.reference_second.x
               << ':' << input.reference_second.y << ':' << input.reference_second.z
               << ':' << input.parameter;
    }
    return output.str();
}

std::string result_material(const std::vector<GpuTermOutput>& output) {
    std::string bytes;
    bytes.resize(output.size() * sizeof(GpuTermOutput));
    std::memcpy(bytes.data(), output.data(), bytes.size());
    return bytes;
}

std::string energy_material(
    const std::vector<EnergyDerivativeOutput>& output) {
    std::ostringstream bytes;
    bytes << std::setprecision(17);
    for (const EnergyDerivativeOutput& value : output) {
        bytes << value.directional_derivative << ':' << value.direction.x << ':'
              << value.direction.y << ':' << value.direction.z << ':'
              << (value.finite ? 1 : 0) << '|';
    }
    return bytes.str();
}

int run() {
    const AuditProfile profile;
    const std::vector<NamedInput> named_inputs = make_inputs(profile);
    const std::vector<TermInput> inputs = raw_inputs(named_inputs);
    std::vector<ReferenceTermOutput> reference;
    std::vector<EnergyDerivativeOutput> energy_reference;
    reference.reserve(inputs.size());
    energy_reference.reserve(inputs.size());
    for (const TermInput& input : inputs) {
        reference.push_back(evaluate_reference_term(profile, input));
        energy_reference.push_back(
            evaluate_viscosity_energy_derivative(profile, input));
    }

    std::vector<std::vector<GpuTermOutput>> corrected_runs;
    corrected_runs.reserve(COLD_REPEATS);
    for (int repeat = 0; repeat < COLD_REPEATS; ++repeat) {
        corrected_runs.push_back(evaluate_gpu_terms(
            profile, inputs, GpuVariant::CorrectedFullPair));
    }
    bool repeat_exact = true;
    for (int repeat = 1; repeat < COLD_REPEATS; ++repeat) {
        repeat_exact = repeat_exact
            && corrected_runs[repeat].size() == corrected_runs.front().size()
            && std::memcmp(corrected_runs[repeat].data(),
                   corrected_runs.front().data(),
                   corrected_runs.front().size() * sizeof(GpuTermOutput))
                == 0;
    }

    const std::vector<GpuTermOutput>& corrected = corrected_runs.front();
    const std::vector<GpuTermOutput> source =
        evaluate_gpu_terms(profile, inputs, GpuVariant::SourceShapedGradient);
    const std::vector<GpuTermOutput> directed_edge = evaluate_gpu_terms(
        profile, inputs, GpuVariant::DirectedEdgeViscosity);
    bool corrected_passed = corrected.size() == reference.size();
    std::vector<bool> case_passed(inputs.size(), false);
    std::vector<double> case_errors(inputs.size(), 0.0);
    std::vector<bool> energy_passed(inputs.size(), false);
    std::vector<double> energy_errors(inputs.size(), 0.0);
    std::vector<double> energy_projections(inputs.size(), 0.0);
    for (std::size_t index = 0; index < inputs.size(); ++index) {
        case_passed[index] = output_passes(corrected[index], reference[index]);
        case_errors[index] = maximum_output_error(corrected[index], reference[index]);
        const bool viscosity = inputs[index].kind == TermKind::BulkViscosity
            || inputs[index].kind == TermKind::ShearViscosity;
        energy_passed[index] = !viscosity
            || energy_derivative_passes(corrected[index], energy_reference[index]);
        if (viscosity) {
            energy_errors[index] = energy_derivative_error(
                corrected[index], energy_reference[index]);
            energy_projections[index] = force_projection(
                corrected[index], energy_reference[index]);
        }
        case_passed[index] = case_passed[index] && energy_passed[index];
        corrected_passed = corrected_passed && case_passed[index];
    }

    const std::array<std::size_t, 3> required_negative = {0, 1, 2};
    bool negative_passed = true;
    std::vector<bool> source_rejected(inputs.size(), false);
    for (std::size_t index = 0; index < inputs.size(); ++index) {
        source_rejected[index] = !output_passes(source[index], reference[index]);
    }
    for (std::size_t index : required_negative) {
        negative_passed = negative_passed && source_rejected[index];
    }
    const std::array<std::size_t, 2> required_directed_edge_negative = {4, 5};
    bool directed_edge_negative_passed = true;
    std::vector<bool> directed_edge_rejected(inputs.size(), false);
    std::vector<double> directed_edge_energy_errors(inputs.size(), 0.0);
    for (std::size_t index : required_directed_edge_negative) {
        directed_edge_energy_errors[index] = energy_derivative_error(
            directed_edge[index], energy_reference[index]);
        directed_edge_rejected[index] =
            !output_passes(directed_edge[index], reference[index])
            && !energy_derivative_passes(
                directed_edge[index], energy_reference[index]);
        directed_edge_negative_passed = directed_edge_negative_passed
            && directed_edge_rejected[index];
    }

    const bool passed = corrected_passed && repeat_exact && negative_passed
        && directed_edge_negative_passed;
    const std::string fixture_root = sha256_hex(
        fixture_material(profile, named_inputs));
    const std::string corrected_root = sha256_hex(result_material(corrected));
    const std::string energy_root = sha256_hex(energy_material(energy_reference));
    const std::string source_root = sha256_hex(result_material(source));
    const std::string directed_edge_root =
        sha256_hex(result_material(directed_edge));
    std::ostringstream output;
    output << std::setprecision(17)
           << "{\"schema\":\"nextengine.nonlocal.corrected_cuda_terms.ncga0.v2\","
              "\"status\":\""
           << (passed ? "PASS" : "FAIL")
           << "\",\"claim\":\"CORRECTED_FULL_PAIR_TERM_CORRESPONDENCE_"
           << (passed ? "SUPPORTED_BOUNDED" : "REJECTED")
           << "\",\"fixture_root\":\"" << fixture_root
           << "\",\"corrected_root\":\"" << corrected_root
           << "\",\"energy_reference_root\":\"" << energy_root
           << "\",\"source_control_root\":\"" << source_root
           << "\",\"directed_edge_control_root\":\"" << directed_edge_root
           << "\",\"environment\":" << gpu_environment_json()
           << ",\"thresholds\":{\"mixed\":" << MIXED_LIMIT
           << ",\"zero\":" << ZERO_LIMIT
           << ",\"energy_derivative\":" << ENERGY_DERIVATIVE_LIMIT
           << "},\"cold_repeats\":" << COLD_REPEATS
           << ",\"repeat_exact\":" << (repeat_exact ? "true" : "false")
           << ",\"negative_control_passed\":"
           << (negative_passed ? "true" : "false")
           << ",\"directed_edge_negative_passed\":"
           << (directed_edge_negative_passed ? "true" : "false")
           << ",\"cases\":[";
    for (std::size_t index = 0; index < inputs.size(); ++index) {
        if (index != 0) {
            output << ',';
        }
        output << "{\"name\":\"" << named_inputs[index].name
               << "\",\"status\":\"" << (case_passed[index] ? "PASS" : "FAIL")
               << "\",\"maximum_mixed_error\":" << case_errors[index]
               << ",\"energy_derivative_status\":";
        if (inputs[index].kind == TermKind::BulkViscosity
            || inputs[index].kind == TermKind::ShearViscosity) {
            output << '"' << (energy_passed[index] ? "PASS" : "FAIL") << '"'
                   << ",\"energy_derivative_error\":" << energy_errors[index]
                   << ",\"energy_derivative_reference\":"
                   << energy_reference[index].directional_derivative
                   << ",\"candidate_force_projection\":"
                   << energy_projections[index]
                   << ",\"directed_edge_energy_error\":"
                   << directed_edge_energy_errors[index];
        } else {
            output << "null,\"energy_derivative_error\":null,"
                      "\"energy_derivative_reference\":null,"
                      "\"candidate_force_projection\":null,"
                      "\"directed_edge_energy_error\":null";
        }
        output << ",\"directed_edge_control_rejected\":"
               << (directed_edge_rejected[index] ? "true" : "false")
               << ",\"source_control_rejected\":"
               << (source_rejected[index] ? "true" : "false") << '}';
    }
    output << "]}";
    std::cout << output.str() << '\n';
    return passed ? 0 : 1;
}

} // namespace
} // namespace nextengine::nonlocal::gpu_audit

int main(int argc, char** argv) {
    try {
        if (argc != 2 || std::string(argv[1]) != "--self-test") {
            std::cerr << "usage: nonlocal-corrected-cuda-terms --self-test\n";
            return 2;
        }
        return nextengine::nonlocal::gpu_audit::run();
    } catch (const std::exception& error) {
        std::cerr << "nonlocal-corrected-cuda-terms: " << error.what() << '\n';
        return 1;
    }
}
