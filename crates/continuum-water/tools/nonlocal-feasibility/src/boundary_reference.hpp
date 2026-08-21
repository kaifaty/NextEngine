#pragma once

#include <string>

namespace nextengine::nonlocal::fcr {

struct SplitBoundaryReport {
    bool passed = false;
    std::string json;
};

SplitBoundaryReport run_split_static_boundary_controls();
SplitBoundaryReport run_boundary_composition_smoke_controls();
SplitBoundaryReport run_boundary_reaction_accuracy_controls();
SplitBoundaryReport run_displacement_ownership_controls();
SplitBoundaryReport run_finite_precision_merit_controls();
SplitBoundaryReport run_floor_stationarity_trajectory_controls();
SplitBoundaryReport run_owned_gradient_controls();
SplitBoundaryReport run_owned_residual_trajectory_controls();
SplitBoundaryReport run_owned_boundary_composition_controls();
SplitBoundaryReport run_closed_box_eligibility_controls();
SplitBoundaryReport run_tiny_pressure_corpus_controls();
SplitBoundaryReport run_box_contact_kkt_controls();
SplitBoundaryReport run_box_contact_kkt_face_controls();
SplitBoundaryReport run_tiny_pressure_contact_kkt_controls();
SplitBoundaryReport run_contact_onset_forecast_controls();
SplitBoundaryReport run_tiny_pressure_contact_forecast_controls();
SplitBoundaryReport run_joint_neighborhood_controls();
SplitBoundaryReport run_joint_neighborhood_one_pass_controls();
SplitBoundaryReport run_joint_pressure_tape_controls();
SplitBoundaryReport run_joint_pressure_query_controls();
SplitBoundaryReport run_joint_pressure_controller_controls();
SplitBoundaryReport run_canonical_stage_controls();
SplitBoundaryReport run_balanced_canonical_controls();
SplitBoundaryReport run_balanced_stage_ledger_controls();
SplitBoundaryReport run_canonical_adaptive_controls();
SplitBoundaryReport run_canonical_adaptive_failure_probe_controls();
SplitBoundaryReport run_canonical_adaptive_ledger_probe_controls();
SplitBoundaryReport run_canonical_adaptive_p2_probe_controls();
SplitBoundaryReport run_canonical_adaptive_recovery_lanes_probe_controls();
SplitBoundaryReport run_canonical_adaptive_recovery_controls();
SplitBoundaryReport run_ledger_normalization_probe_controls();
SplitBoundaryReport run_ledger_normalization_controls();
SplitBoundaryReport run_kkt_scale_stage_ledger_probe_controls();
SplitBoundaryReport run_kkt_scale_stage_ledger_controls();

} // namespace nextengine::nonlocal::fcr
