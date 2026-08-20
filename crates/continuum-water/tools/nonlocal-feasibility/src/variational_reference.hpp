#pragma once

#include <string>

namespace nextengine::nonlocal::fcr {

struct ReferenceSolverReport {
    bool passed = false;
    std::string json;
};

ReferenceSolverReport run_reference_solver_controls();
ReferenceSolverReport run_conditioning_controls();
ReferenceSolverReport run_sissm_controls();
ReferenceSolverReport run_sissm_term_local_controls();
ReferenceSolverReport run_sissm_pressure_chebyshev_controls();
ReferenceSolverReport run_spectral_hvp_controls();
ReferenceSolverReport run_trust_region_controls();
ReferenceSolverReport run_block_preconditioner_controls();
ReferenceSolverReport run_scale_aware_block_preconditioner_controls();
ReferenceSolverReport run_neighborhood_hvp_controls();
ReferenceSolverReport run_neighborhood_trust_scaling_controls();
ReferenceSolverReport run_neighborhood_trust_rejection_trace_controls();
ReferenceSolverReport run_numerical_floor_stop_controls();
ReferenceSolverReport run_serial_cpu_baseline_controls();
ReferenceSolverReport run_hvp_workspace_stream_controls();
ReferenceSolverReport run_hessian_tape_controls();
ReferenceSolverReport run_dimensional_profile_controls();
ReferenceSolverReport run_normalized_kernel_reclosure_controls();
ReferenceSolverReport run_manufactured_multistep_controls();
ReferenceSolverReport run_temporal_stiffness_diagnostic_controls();
ReferenceSolverReport run_floor_limited_temporal_oracle_controls();
ReferenceSolverReport run_acoustic_substep_policy_controls();
ReferenceSolverReport run_pressure_tangent_spectrum_controls();
ReferenceSolverReport run_spectral_substep_policy_controls();

} // namespace nextengine::nonlocal::fcr
