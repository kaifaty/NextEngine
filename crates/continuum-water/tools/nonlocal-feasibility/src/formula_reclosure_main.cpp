#include "formula_discriminators.hpp"
#include "formula_reclosure.hpp"
#include "variational_reference.hpp"

#include <exception>
#include <iostream>
#include <string>

int main(int argc, char** argv) {
    try {
        if (argc != 2) {
            std::cerr << "usage: nonlocal-formula-reclosure "
                         "--self-test|--pair-pressure-self-test|"
                         "--reference-solver-self-test|--conditioning-self-test|"
                         "--sissm-self-test|--sissm-term-local-self-test|"
                         "--sissm-pressure-chebyshev-self-test|"
                         "--spectral-hvp-self-test|--trust-region-self-test|"
                         "--block-preconditioner-self-test|"
                         "--scale-aware-block-preconditioner-self-test|"
                         "--neighborhood-hvp-self-test|"
                         "--neighborhood-trust-scaling-self-test|"
                         "--neighborhood-trust-rejection-trace-self-test|"
                         "--numerical-floor-stop-self-test|"
                         "--serial-cpu-baseline|"
                         "--hvp-workspace-stream-benchmark|"
                         "--hessian-tape-benchmark|"
                         "--dimensional-profile-self-test|"
                         "--normalized-kernel-reclosure-self-test|"
                         "--manufactured-multistep-self-test|"
                         "--temporal-stiffness-diagnostic|"
                         "--floor-limited-temporal-oracle|"
                         "--acoustic-substep-policy-self-test|"
                         "--pressure-tangent-spectrum-self-test|"
                         "--spectral-substep-policy-self-test|"
                         "--embedded-spectral-error-controller-self-test|"
                         "--transactional-multistep-controller-self-test\n";
            return 2;
        }
        const std::string command = argv[1];
        if (command == "--self-test") {
            const nextengine::nonlocal::fcr::FormulaReport report =
                nextengine::nonlocal::fcr::run_formula_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--pair-pressure-self-test") {
            const nextengine::nonlocal::fcr::DiscriminatorReport report =
                nextengine::nonlocal::fcr::run_pair_pressure_discriminator();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--reference-solver-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::run_reference_solver_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--conditioning-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::run_conditioning_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--sissm-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::run_sissm_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--sissm-term-local-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::run_sissm_term_local_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--sissm-pressure-chebyshev-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::run_sissm_pressure_chebyshev_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--spectral-hvp-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::run_spectral_hvp_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--trust-region-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::run_trust_region_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--block-preconditioner-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::run_block_preconditioner_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--scale-aware-block-preconditioner-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::
                    run_scale_aware_block_preconditioner_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--neighborhood-hvp-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::run_neighborhood_hvp_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--neighborhood-trust-scaling-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::
                    run_neighborhood_trust_scaling_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--neighborhood-trust-rejection-trace-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::
                    run_neighborhood_trust_rejection_trace_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--numerical-floor-stop-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::
                    run_numerical_floor_stop_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--serial-cpu-baseline") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::run_serial_cpu_baseline_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--hvp-workspace-stream-benchmark") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::
                    run_hvp_workspace_stream_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--hessian-tape-benchmark") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::run_hessian_tape_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--dimensional-profile-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::run_dimensional_profile_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--normalized-kernel-reclosure-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::
                    run_normalized_kernel_reclosure_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--manufactured-multistep-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::
                    run_manufactured_multistep_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--temporal-stiffness-diagnostic") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::
                    run_temporal_stiffness_diagnostic_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--floor-limited-temporal-oracle") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::
                    run_floor_limited_temporal_oracle_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--acoustic-substep-policy-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::
                    run_acoustic_substep_policy_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--pressure-tangent-spectrum-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::
                    run_pressure_tangent_spectrum_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--spectral-substep-policy-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::
                    run_spectral_substep_policy_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--embedded-spectral-error-controller-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::
                    run_embedded_spectral_error_controller_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--transactional-multistep-controller-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::
                    run_transactional_multistep_controller_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        std::cerr << "usage: nonlocal-formula-reclosure "
                     "--self-test|--pair-pressure-self-test|"
                     "--reference-solver-self-test|--conditioning-self-test|"
                     "--sissm-self-test|--sissm-term-local-self-test|"
                     "--sissm-pressure-chebyshev-self-test|"
                     "--spectral-hvp-self-test|--trust-region-self-test|"
                     "--block-preconditioner-self-test|"
                     "--scale-aware-block-preconditioner-self-test|"
                     "--neighborhood-hvp-self-test|"
                     "--neighborhood-trust-scaling-self-test|"
                     "--neighborhood-trust-rejection-trace-self-test|"
                     "--numerical-floor-stop-self-test|"
                     "--serial-cpu-baseline|"
                     "--hvp-workspace-stream-benchmark|"
                     "--hessian-tape-benchmark|"
                     "--dimensional-profile-self-test|"
                     "--normalized-kernel-reclosure-self-test|"
                     "--manufactured-multistep-self-test|"
                     "--temporal-stiffness-diagnostic|"
                     "--floor-limited-temporal-oracle|"
                     "--acoustic-substep-policy-self-test|"
                     "--pressure-tangent-spectrum-self-test|"
                     "--spectral-substep-policy-self-test|"
                     "--embedded-spectral-error-controller-self-test|"
                     "--transactional-multistep-controller-self-test\n";
        return 2;
    } catch (const std::exception& error) {
        std::cerr << "nonlocal-formula-reclosure: " << error.what() << '\n';
        return 1;
    }
}
