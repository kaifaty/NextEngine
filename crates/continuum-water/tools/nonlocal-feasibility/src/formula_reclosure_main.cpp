#include "boundary_reference.hpp"
#include "formula_discriminators.hpp"
#include "formula_reclosure.hpp"
#include "reference_attestation.hpp"
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
                         "--transactional-multistep-controller-self-test|"
                         "--fine-state-ownership-self-test|"
                         "--split-static-boundary-self-test|"
                         "--boundary-composition-smoke-self-test|"
                         "--boundary-reaction-accuracy-self-test|"
                         "--displacement-ownership-self-test|"
                         "--finite-precision-merit-self-test|"
                         "--floor-stationarity-trajectory-self-test|"
                         "--owned-gradient-self-test|"
                         "--owned-residual-trajectory-self-test|"
                         "--owned-boundary-composition-self-test|"
                         "--closed-box-eligibility-self-test|"
                         "--tiny-pressure-corpus-self-test|"
                         "--box-contact-kkt-self-test|"
                         "--box-contact-kkt-face-self-test|"
                         "--tiny-pressure-contact-kkt-self-test|"
                         "--contact-onset-forecast-self-test|"
                         "--tiny-pressure-contact-forecast-self-test|"
                         "--joint-neighborhood-self-test|"
                         "--joint-neighborhood-one-pass-self-test|"
                         "--joint-pressure-tape-self-test|"
                         "--joint-pressure-query-self-test|"
                         "--joint-pressure-controller-self-test|"
                         "--canonical-stage-self-test|"
                         "--canonical-conservation-self-test|"
                         "--canonical-stage-ledger-self-test|"
                         "--canonical-adaptive-self-test|"
                         "--canonical-adaptive-failure-probe|"
                         "--canonical-adaptive-ledger-probe|"
                         "--canonical-adaptive-p2-probe|"
                         "--canonical-adaptive-recovery-lanes-probe|"
                         "--canonical-adaptive-recovery-self-test|"
                         "--ledger-normalization-probe|"
                         "--ledger-normalization-self-test|"
                         "--kkt-scale-stage-ledger-probe|"
                         "--kkt-scale-stage-ledger-self-test|"
                         "--combined-adaptive-replay-probe|"
                         "--combined-adaptive-replay-self-test|"
                         "--fixed-canonical-reference-probe|"
                         "--fixed-canonical-reference-self-test|"
                         "--publication-cadence-probe|"
                         "--publication-cadence-self-test|"
                         "--publication-stability-probe|"
                         "--publication-stability-self-test|"
                         "--mixed-stability-budget-probe|"
                         "--mixed-stability-budget-self-test|"
                         "--macro-adaptive-transaction-probe|"
                         "--macro-adaptive-transaction-self-test|"
                         "--canonical-topology-probe|"
                         "--canonical-topology-self-test|"
                         "--macro-adaptive-replay-probe|"
                         "--macro-adaptive-replay-self-test|"
                         "--adaptive-fixed-diagnostic-probe|"
                         "--adaptive-fixed-diagnostic-self-test|"
                         "--adaptive-accuracy-budget-probe|"
                         "--adaptive-accuracy-budget-self-test|"
                         "--workspace-reuse-diagnostic-probe|"
                         "--workspace-reuse-diagnostic-self-test|"
                         "--reference-attestation-self-test|"
                         "--nominal-alignment-preflight|"
                         "--nominal-hydro-spectrum-probe|"
                         "--nominal-hydro-macro-probe|"
                         "--nominal-hydro-query-evidence-ablation|"
                         "--nominal-hydro-topology-reuse-audit|"
                         "--nominal-hydro-cached-topology-ablation|"
                         "--nominal-hydro-hvp-coefficient-ablation|"
                         "--nominal-hydro-evaluation-tape-dataflow-audit|"
                         "--nominal-hydro-fused-evaluation-tape-ablation|"
                         "--nominal-hydro-fused-phase-timing|"
                         "--nominal-hydro-owner-computes-dataflow-audit|"
                         "--nominal-hydro-owner-parallel-1|"
                         "--nominal-hydro-owner-parallel-2|"
                         "--nominal-hydro-owner-parallel-4|"
                         "--nominal-hydro-owner-parallel-8|"
                         "--nominal-hydro-owner-parallel-16|"
                         "--nominal-hydro-owner-parallel-phase-timing-8|"
                         "--nominal-hydro-masked-superset-plan-audit|"
                         "--nominal-hydro-masked-superset-plan-8|"
                         "--nominal-hydro-partitioned-active-plan-audit|"
                         "--nominal-hydro-split-incoming-plan-8|"
                         "--nominal-hydro-split-incoming-phase-timing-8|"
                         "--nominal-hydro-directed-scratch-audit|"
                         "--nominal-hydro-directed-scratch-reuse-8|"
                         "--nominal-hydro-directed-scratch-phase-timing-8|"
                         "--nominal-hydro-directed-scratch-setup-timing-8|"
                         "--nominal-hydro-directed-scratch-cpu-timing-8|"
                         "--nominal-hydro-directed-scratch-evaluation-buffer-"
                         "audit|"
                         "--nominal-hydro-topology-incoming-fusion-audit|"
                         "--nominal-dam-first-output|"
                         "--nominal-dam-reference-first-output|"
                         "--nominal-dam-step2-strain-refinement|"
                         "--nonlocal-pressure-state-formulation|"
                         "--nonlocal-al-path-oracle|"
                         "--nonlocal-al-dense-vector-oracle|"
                         "--nonlocal-al-dense-vector-stable-commit|"
                         "--nonlocal-al-inner-floor-diagnostic|"
                         "--nonlocal-al-topology-step-discriminator|"
                         "--nonlocal-al-topology-step-signed-zero-reclosure|"
                         "--nonlocal-al-globalization-policy-discriminator|"
                         "--nonlocal-al-step-norm-trust-inner|"
                         "--nonlocal-al-step-norm-private-outer|"
                         "--nonlocal-al-cap-accuracy-discriminator|"
                         "--nonlocal-al-inner-floor-mechanism-discriminator|"
                         "--nonlocal-al-extended-precision-energy-"
                         "discriminator|"
                         "--nonlocal-al-divided-difference-discriminator|"
                         "--nonlocal-al-divided-difference-reclosure|"
                         "--nonlocal-al-divided-difference-private-inner|"
                         "--nonlocal-al-binary128-accepted-sign-oracle|"
                         "--nonlocal-al-divided-private-outer-continuation|"
                         "--nonlocal-al-divided-full-private-transaction|"
                         "--nonlocal-al-sparse-workspace-equivalence|"
                         "--nonlocal-al-nominal-prerequisites|"
                         "--nonlocal-al-sparse-precision-transaction|"
                         "--nonlocal-al-nominal-substep-shadow|"
                         "--nonlocal-al-kappa-scaling-prerequisites|"
                         "--nonlocal-al-nondimensional-transaction|"
                         "--nonlocal-al-full-normalized-private-transaction|"
                         "--nonlocal-al-normalized-divided-precancellation-"
                         "replay|"
                         "--nonlocal-al-full-normalized-precancelled-private-"
                         "transaction|"
                         "--nonlocal-al-normalized-krylov-forcing-replay|"
                         "--nonlocal-al-full-normalized-dimensionless-"
                         "forcing-private-transaction|"
                         "--nonlocal-al-normalized-nominal-substep-shadow|"
                         "--nonlocal-al-topology-precision-shell-replay|"
                         "--nonlocal-al-binary64-topology-nominal-substep-"
                         "shadow|"
                         "--nonlocal-al-sixth-trust-recurrence-replay|"
                         "--nonlocal-al-krylov-model-image-discriminator|"
                         "--nonlocal-al-guarded-sixth-trial-shadow|"
                         "--nonlocal-al-guarded-residual-private-transaction|"
                         "--nonlocal-al-second-guard-boundary-replay|"
                         "--nonlocal-al-tiered-grace-model-image-"
                         "discriminator|"
                         "--nonlocal-al-tiered-grace-private-transaction|"
                         "--nonlocal-al-total-hvp-boundary-diagnostic|"
                         "--nonlocal-al-total-budget-offline-replay|"
                         "--nonlocal-al-total-budget-atomic-completion|"
                         "--nonlocal-al-post-acceptance-boundary|"
                         "--nonlocal-al-soft-cap-suspension-projection|"
                         "--nonlocal-al-token-completeness|"
                         "--nonlocal-al-v2-envelope-validation|"
                         "--nonlocal-al-owner-epoch-transition|"
                         "--nonlocal-al-shadow-outer6-resume|"
                         "--nonlocal-al-within-epoch-outer7-grant|"
                         "--nonlocal-al-shadow-outer7-execution|"
                         "--nonlocal-al-within-epoch-outer8-grant|"
                         "--nonlocal-al-shadow-outer8-execution|"
                         "--nonlocal-al-within-epoch-outer9-grant|"
                         "--nonlocal-al-shadow-outer9-execution|"
                         "--nonlocal-al-within-epoch-outer10-grant|"
                         "--nonlocal-al-shadow-outer10-execution|"
                         "--nonlocal-al-within-epoch-outer11-grant|"
                         "--nonlocal-al-shadow-outer11-execution|"
                         "--nonlocal-al-linearized-feasibility-operator|"
                         "--nonlocal-al-linearized-range-projection|"
                         "--nonlocal-al-scalar-feasibility-interval|"
                         "--nonlocal-al-all-inequality-cauchy-normal-step|"
                         "--nonlocal-al-iterated-all-inequality-normal-step|"
                         "--nonlocal-al-first-order-reference-extension|"
                         "--nonlocal-al-equal-work-generalized-hessian|"
                         "--nonlocal-al-equal-work-curvature-polish-hybrid|"
                         "--nonlocal-al-hybrid-polish-continuation|"
                         "--nonlocal-al-direction-memory-replay|"
                         "--nonlocal-al-guarded-hz-recurrence|"
                         "--nonlocal-al-nonlinear-normal-acceptance|"
                         "--nonlocal-al-support-crossing-contact-audit|"
                         "--nonlocal-al-stable-superset-relinearization|"
                         "--nonlocal-al-contact-tangent-normal-step|"
                         "--nonlocal-al-contact-feasible-common-descent|"
                         "--nonlocal-al-nonlinear-line-globalization|"
                         "--nonlocal-al-filter-globalization|"
                         "--nominal-dam-first-output-preflight|"
                         "--nominal-dam-reference-binary64-topology\n";
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
        if (command == "--fine-state-ownership-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::
                    run_fine_state_ownership_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--split-static-boundary-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::run_split_static_boundary_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--boundary-composition-smoke-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_boundary_composition_smoke_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--boundary-reaction-accuracy-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_boundary_reaction_accuracy_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--displacement-ownership-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_displacement_ownership_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--finite-precision-merit-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_finite_precision_merit_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--floor-stationarity-trajectory-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_floor_stationarity_trajectory_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--owned-gradient-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::run_owned_gradient_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--owned-residual-trajectory-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_owned_residual_trajectory_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--owned-boundary-composition-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_owned_boundary_composition_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--closed-box-eligibility-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_closed_box_eligibility_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--tiny-pressure-corpus-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_tiny_pressure_corpus_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--box-contact-kkt-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::run_box_contact_kkt_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--box-contact-kkt-face-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_box_contact_kkt_face_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--tiny-pressure-contact-kkt-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_tiny_pressure_contact_kkt_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--contact-onset-forecast-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_contact_onset_forecast_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--tiny-pressure-contact-forecast-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_tiny_pressure_contact_forecast_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--joint-neighborhood-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::run_joint_neighborhood_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--joint-neighborhood-one-pass-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_joint_neighborhood_one_pass_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--joint-pressure-tape-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::run_joint_pressure_tape_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--joint-pressure-query-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::run_joint_pressure_query_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--joint-pressure-controller-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_joint_pressure_controller_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--canonical-stage-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::run_canonical_stage_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--canonical-conservation-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::run_balanced_canonical_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--canonical-stage-ledger-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_balanced_stage_ledger_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--canonical-adaptive-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_canonical_adaptive_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--canonical-adaptive-failure-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_canonical_adaptive_failure_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--canonical-adaptive-ledger-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_canonical_adaptive_ledger_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--canonical-adaptive-p2-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_canonical_adaptive_p2_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--canonical-adaptive-recovery-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_canonical_adaptive_recovery_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--canonical-adaptive-recovery-lanes-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_canonical_adaptive_recovery_lanes_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--ledger-normalization-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_ledger_normalization_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--ledger-normalization-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_ledger_normalization_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--kkt-scale-stage-ledger-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_kkt_scale_stage_ledger_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--kkt-scale-stage-ledger-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_kkt_scale_stage_ledger_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--combined-adaptive-replay-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_combined_adaptive_replay_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--combined-adaptive-replay-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_combined_adaptive_replay_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--fixed-canonical-reference-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_fixed_canonical_reference_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--fixed-canonical-reference-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_fixed_canonical_reference_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--publication-cadence-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_publication_cadence_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--publication-cadence-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_publication_cadence_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--publication-stability-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_publication_stability_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--publication-stability-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_publication_stability_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--mixed-stability-budget-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_mixed_stability_budget_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--mixed-stability-budget-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_mixed_stability_budget_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--macro-adaptive-transaction-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_macro_adaptive_transaction_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--macro-adaptive-transaction-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_macro_adaptive_transaction_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--canonical-topology-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_canonical_topology_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--canonical-topology-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_canonical_topology_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--macro-adaptive-replay-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_macro_adaptive_replay_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--macro-adaptive-replay-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_macro_adaptive_replay_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--adaptive-fixed-diagnostic-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_adaptive_fixed_diagnostic_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--adaptive-fixed-diagnostic-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_adaptive_fixed_diagnostic_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--adaptive-accuracy-budget-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_adaptive_accuracy_budget_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--adaptive-accuracy-budget-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_adaptive_accuracy_budget_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--workspace-reuse-diagnostic-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_workspace_reuse_diagnostic_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--workspace-reuse-diagnostic-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_workspace_reuse_diagnostic_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--retained-workspace-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_retained_workspace_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--retained-workspace-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_retained_workspace_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--complete-retention-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_complete_retention_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--complete-retention-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_complete_retention_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--static-support-index-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_static_support_index_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--static-support-index-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_static_support_index_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--static-support-index-benchmark") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_static_support_timing_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--complete-static-support-index-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_complete_static_index_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--complete-static-support-index-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_complete_static_index_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--flat-adjacency-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_flat_adjacency_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--flat-adjacency-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_flat_adjacency_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--flat-adjacency-benchmark") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_flat_adjacency_timing_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--complete-flat-adjacency-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_complete_flat_adjacency_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--complete-flat-adjacency-self-test") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_complete_flat_adjacency_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--reference-attestation-self-test") {
            const nextengine::nonlocal::fcr::ReferenceAttestationReport report =
                nextengine::nonlocal::fcr::
                    run_reference_attestation_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-alignment-preflight") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_alignment_preflight_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-spectrum-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_spectrum_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-macro-probe") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_macro_probe_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-query-evidence-ablation") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_query_evidence_ablation_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-topology-reuse-audit") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_topology_reuse_audit_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-cached-topology-ablation") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_cached_topology_ablation_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-hvp-coefficient-ablation") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_hvp_coefficient_ablation_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-evaluation-tape-dataflow-audit") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_evaluation_tape_dataflow_audit_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-fused-evaluation-tape-ablation") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_fused_evaluation_tape_ablation_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-fused-phase-timing") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_fused_phase_timing_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-owner-computes-dataflow-audit") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_owner_computes_dataflow_audit_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        for (const int worker_count : {1, 2, 4, 8, 16}) {
            if (command == "--nominal-hydro-owner-parallel-"
                    + std::to_string(worker_count)) {
                const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                    nextengine::nonlocal::fcr::
                        run_nominal_hydro_owner_parallel_controls(
                            worker_count);
                std::cout << report.json << '\n';
                return report.passed ? 0 : 1;
            }
        }
        if (command == "--nominal-hydro-owner-parallel-phase-timing-8") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_owner_parallel_phase_timing_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-masked-superset-plan-audit") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_masked_superset_plan_audit_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-masked-superset-plan-8") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_masked_superset_plan_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-partitioned-active-plan-audit") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_partitioned_active_plan_audit_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-partitioned-active-plan-8") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_partitioned_active_plan_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-current-topology-plan-audit") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_current_topology_plan_audit_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-split-incoming-plan-audit") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_split_incoming_plan_audit_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-incoming-construction-audit") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_incoming_construction_audit_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-split-incoming-plan-8") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_split_incoming_plan_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-split-incoming-phase-timing-8") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_split_incoming_phase_timing_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-directed-scratch-audit") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_directed_scratch_audit_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-directed-scratch-reuse-8") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_directed_scratch_reuse_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-dam-first-output") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_dam_first_output_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-dam-reference-first-output") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_dam_reference_first_output_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-dam-step2-strain-refinement") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_dam_step2_strain_refinement_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-pressure-state-formulation") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_pressure_state_formulation_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-path-oracle") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::run_al_path_oracle_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-dense-vector-oracle") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_dense_vector_oracle_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-dense-vector-stable-commit") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_dense_vector_stable_commit_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-inner-floor-diagnostic") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_inner_floor_diagnostic_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-topology-step-discriminator") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_topology_step_discriminator_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-topology-step-signed-zero-reclosure") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_topology_step_signed_zero_reclosure_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-globalization-policy-discriminator") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_globalization_policy_discriminator_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-step-norm-trust-inner") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_step_norm_trust_inner_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-step-norm-private-outer") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_step_norm_private_outer_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-cap-accuracy-discriminator") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_cap_accuracy_discriminator_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-inner-floor-mechanism-discriminator") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_inner_floor_mechanism_discriminator_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-extended-precision-energy-discriminator") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_extended_precision_energy_discriminator_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-divided-difference-discriminator") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_divided_difference_discriminator_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-divided-difference-reclosure") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_divided_difference_reclosure_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-divided-difference-private-inner") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_divided_difference_private_inner_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-binary128-accepted-sign-oracle") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_binary128_accepted_sign_oracle_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-divided-private-outer-continuation") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_divided_private_outer_continuation_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-divided-full-private-transaction") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_divided_full_private_transaction_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-sparse-workspace-equivalence") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_sparse_workspace_equivalence_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-nominal-prerequisites") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_nominal_prerequisites_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-sparse-precision-transaction") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_sparse_precision_transaction_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-nominal-substep-shadow") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_nominal_substep_shadow_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-kappa-scaling-prerequisites") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_kappa_scaling_prerequisites_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-nondimensional-transaction") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_nondimensional_transaction_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-full-normalized-private-transaction") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_full_normalized_private_transaction_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-normalized-divided-precancellation-replay") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_normalized_divided_precancellation_replay_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-full-normalized-precancelled-private-"
               "transaction") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_full_normalized_precancelled_private_transaction_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-normalized-krylov-forcing-replay") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_normalized_krylov_forcing_replay_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-full-normalized-dimensionless-forcing-"
               "private-transaction") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_full_normalized_dimensionless_forcing_private_transaction_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-normalized-nominal-substep-shadow") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_normalized_nominal_substep_shadow_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-topology-precision-shell-replay") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_topology_precision_shell_replay_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-binary64-topology-nominal-substep-shadow") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_binary64_topology_nominal_substep_shadow_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-sixth-trust-recurrence-replay") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_sixth_trust_recurrence_replay_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-krylov-model-image-discriminator") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_krylov_model_image_discriminator_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-guarded-sixth-trial-shadow") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_guarded_sixth_trial_shadow_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-guarded-residual-private-transaction") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_guarded_residual_private_transaction_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-second-guard-boundary-replay") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_second_guard_boundary_replay_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-tiered-grace-model-image-discriminator") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_tiered_grace_model_image_discriminator_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-tiered-grace-private-transaction") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_tiered_grace_private_transaction_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-total-hvp-boundary-diagnostic") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_total_hvp_boundary_diagnostic_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-total-budget-offline-replay") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_total_budget_offline_replay_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-total-budget-atomic-completion") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_total_budget_atomic_completion_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-post-acceptance-boundary") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_post_acceptance_boundary_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-soft-cap-suspension-projection") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_soft_cap_suspension_projection_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-token-completeness") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_token_completeness_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-v2-envelope-validation") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_v2_envelope_validation_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-owner-epoch-transition") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_owner_epoch_transition_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-shadow-outer6-resume") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_shadow_outer6_resume_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-within-epoch-outer7-grant") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_within_epoch_outer7_grant_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-shadow-outer7-execution") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_shadow_outer7_execution_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-within-epoch-outer8-grant") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_within_epoch_outer8_grant_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-shadow-outer8-execution") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_shadow_outer8_execution_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-within-epoch-outer9-grant") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_within_epoch_outer9_grant_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-shadow-outer9-execution") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_shadow_outer9_execution_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-within-epoch-outer10-grant") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_within_epoch_outer10_grant_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-shadow-outer10-execution") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_shadow_outer10_execution_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-within-epoch-outer11-grant") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_within_epoch_outer11_grant_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-shadow-outer11-execution") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_shadow_outer11_execution_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-linearized-feasibility-operator") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_linearized_feasibility_operator_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-linearized-range-projection") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_linearized_range_projection_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-scalar-feasibility-interval") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_scalar_feasibility_interval_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-all-inequality-cauchy-normal-step") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_all_inequality_cauchy_step_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-iterated-all-inequality-normal-step") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_iterated_all_inequality_normal_step_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-first-order-reference-extension") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_first_order_reference_extension_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-equal-work-generalized-hessian") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_equal_work_generalized_hessian_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nonlocal-al-equal-work-curvature-polish-hybrid") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_equal_work_curvature_polish_hybrid_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-hybrid-polish-continuation") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_hybrid_polish_continuation_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-direction-memory-replay") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_direction_memory_replay_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-guarded-hz-recurrence") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_guarded_hz_recurrence_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-nonlinear-normal-acceptance") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_nonlinear_normal_acceptance_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-support-crossing-contact-audit") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_support_crossing_contact_audit_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-stable-superset-relinearization") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_stable_superset_relinearization_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-contact-tangent-normal-step") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_contact_tangent_normal_step_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-contact-feasible-common-descent") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_contact_feasible_common_descent_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-nonlinear-line-globalization") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_nonlinear_line_globalization_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nonlocal-al-filter-globalization") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_al_filter_globalization_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-dam-first-output-preflight") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_dam_first_output_preflight_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-dam-reference-binary64-topology") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_dam_reference_binary64_topology_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-directed-scratch-phase-timing-8") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_directed_scratch_phase_timing_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-directed-scratch-setup-timing-8") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_directed_scratch_setup_timing_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-directed-scratch-cpu-timing-8") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_directed_scratch_cpu_timing_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command
            == "--nominal-hydro-directed-scratch-evaluation-buffer-audit") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_directed_scratch_evaluation_buffer_audit_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--nominal-hydro-topology-incoming-fusion-audit") {
            const nextengine::nonlocal::fcr::SplitBoundaryReport report =
                nextengine::nonlocal::fcr::
                    run_nominal_hydro_topology_incoming_fusion_audit_controls();
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
                     "--transactional-multistep-controller-self-test|"
                     "--fine-state-ownership-self-test|"
                     "--split-static-boundary-self-test|"
                     "--boundary-composition-smoke-self-test|"
                     "--boundary-reaction-accuracy-self-test|"
                     "--displacement-ownership-self-test|"
                     "--finite-precision-merit-self-test|"
                     "--floor-stationarity-trajectory-self-test|"
                     "--owned-gradient-self-test|"
                     "--owned-residual-trajectory-self-test|"
                     "--owned-boundary-composition-self-test|"
                     "--closed-box-eligibility-self-test|"
                     "--tiny-pressure-corpus-self-test|"
                     "--box-contact-kkt-self-test|"
                     "--box-contact-kkt-face-self-test|"
                     "--tiny-pressure-contact-kkt-self-test|"
                     "--contact-onset-forecast-self-test|"
                     "--tiny-pressure-contact-forecast-self-test|"
                     "--joint-neighborhood-self-test|"
                     "--joint-neighborhood-one-pass-self-test|"
                     "--joint-pressure-tape-self-test|"
                     "--joint-pressure-query-self-test|"
                     "--joint-pressure-controller-self-test|"
                     "--canonical-stage-self-test|"
                     "--canonical-conservation-self-test|"
                     "--canonical-stage-ledger-self-test|"
                     "--canonical-adaptive-self-test|"
                     "--canonical-adaptive-failure-probe|"
                     "--canonical-adaptive-ledger-probe|"
                     "--canonical-adaptive-p2-probe|"
                     "--canonical-adaptive-recovery-lanes-probe|"
                     "--canonical-adaptive-recovery-self-test|"
                     "--ledger-normalization-probe|"
                     "--ledger-normalization-self-test|"
                     "--kkt-scale-stage-ledger-probe|"
                     "--kkt-scale-stage-ledger-self-test|"
                     "--combined-adaptive-replay-probe|"
                     "--combined-adaptive-replay-self-test|"
                     "--fixed-canonical-reference-probe|"
                     "--fixed-canonical-reference-self-test|"
                     "--publication-cadence-probe|"
                     "--publication-cadence-self-test|"
                     "--publication-stability-probe|"
                     "--publication-stability-self-test|"
                     "--mixed-stability-budget-probe|"
                     "--mixed-stability-budget-self-test|"
                     "--macro-adaptive-transaction-probe|"
                     "--macro-adaptive-transaction-self-test|"
                     "--canonical-topology-probe|"
                     "--canonical-topology-self-test|"
                     "--macro-adaptive-replay-probe|"
                     "--macro-adaptive-replay-self-test|"
                     "--adaptive-fixed-diagnostic-probe|"
                     "--adaptive-fixed-diagnostic-self-test|"
                     "--adaptive-accuracy-budget-probe|"
                     "--adaptive-accuracy-budget-self-test|"
                     "--retained-workspace-probe|"
                     "--retained-workspace-self-test|"
                     "--complete-retention-probe|"
                     "--complete-retention-self-test|"
                     "--static-support-index-probe|"
                     "--static-support-index-self-test|"
                     "--static-support-index-benchmark|"
                     "--complete-static-support-index-probe|"
                     "--complete-static-support-index-self-test|"
                     "--flat-adjacency-probe|"
                     "--flat-adjacency-self-test|"
                     "--flat-adjacency-benchmark|"
                     "--complete-flat-adjacency-probe|"
                     "--complete-flat-adjacency-self-test|"
                     "--workspace-reuse-diagnostic-probe|"
                     "--workspace-reuse-diagnostic-self-test|"
                     "--reference-attestation-self-test|"
                     "--nominal-alignment-preflight|"
                     "--nominal-hydro-spectrum-probe|"
                     "--nominal-hydro-macro-probe|"
                     "--nominal-hydro-query-evidence-ablation|"
                     "--nominal-hydro-topology-reuse-audit|"
                     "--nominal-hydro-cached-topology-ablation|"
                     "--nominal-hydro-hvp-coefficient-ablation|"
                     "--nominal-hydro-evaluation-tape-dataflow-audit|"
                     "--nominal-hydro-fused-evaluation-tape-ablation|"
                     "--nominal-hydro-fused-phase-timing|"
                     "--nominal-hydro-owner-computes-dataflow-audit|"
                     "--nominal-hydro-owner-parallel-1|"
                     "--nominal-hydro-owner-parallel-2|"
                     "--nominal-hydro-owner-parallel-4|"
                     "--nominal-hydro-owner-parallel-8|"
                     "--nominal-hydro-owner-parallel-16|"
                     "--nominal-hydro-owner-parallel-phase-timing-8|"
                     "--nominal-hydro-masked-superset-plan-audit|"
                     "--nominal-hydro-masked-superset-plan-8|"
                     "--nominal-hydro-partitioned-active-plan-audit|"
                     "--nominal-hydro-partitioned-active-plan-8|"
                     "--nominal-hydro-current-topology-plan-audit|"
                     "--nominal-hydro-split-incoming-plan-audit|"
                     "--nominal-hydro-incoming-construction-audit|"
                     "--nominal-hydro-split-incoming-plan-8|"
                     "--nominal-hydro-split-incoming-phase-timing-8|"
                     "--nominal-hydro-directed-scratch-audit|"
                     "--nominal-hydro-directed-scratch-reuse-8|"
                     "--nominal-hydro-directed-scratch-phase-timing-8|"
                     "--nominal-hydro-directed-scratch-setup-timing-8|"
                     "--nominal-hydro-directed-scratch-cpu-timing-8|"
                     "--nominal-hydro-directed-scratch-evaluation-buffer-"
                     "audit|"
                     "--nominal-hydro-topology-incoming-fusion-audit|"
                     "--nominal-dam-first-output|"
                     "--nominal-dam-reference-first-output|"
                     "--nominal-dam-step2-strain-refinement|"
                     "--nonlocal-pressure-state-formulation|"
                     "--nonlocal-al-path-oracle|"
                     "--nonlocal-al-dense-vector-oracle|"
                     "--nonlocal-al-dense-vector-stable-commit|"
                     "--nonlocal-al-inner-floor-diagnostic|"
                     "--nonlocal-al-topology-step-discriminator|"
                     "--nonlocal-al-topology-step-signed-zero-reclosure|"
                     "--nonlocal-al-globalization-policy-discriminator|"
                     "--nonlocal-al-step-norm-trust-inner|"
                     "--nominal-dam-first-output-preflight|"
                     "--nominal-dam-reference-binary64-topology\n";
        return 2;
    } catch (const std::exception& error) {
        std::cerr << "nonlocal-formula-reclosure: " << error.what() << '\n';
        return 1;
    }
}
