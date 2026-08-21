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

} // namespace nextengine::nonlocal::fcr
