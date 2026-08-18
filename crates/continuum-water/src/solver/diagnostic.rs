#![forbid(unsafe_code)]

use crate::audit::{
    AuditBoundaryInput, AuditComputation, BoundaryNeighborTrace, DensityInitialTrace,
    DensityIterationRowTrace, HydroAuditTrace, HydroRowTrace, SELECTED_ROWS,
    production_fluid_input, scalar_bits, vector_bits,
};
use crate::calibration::successor::{
    ContactFixtureInput, ContactFixtureObservation, DensityFixtureObservation, FeatureActivation,
};
use crate::calibration::{ContactProjectionCase, ContactProjectionProbe};
use crate::error::{AUDIT_INVALID, DENSITY_NONCONVERGENCE, WaterError};

use super::*;

mod calibration;

pub(crate) use calibration::{
    production_hydro_calibration, production_pressure_operator_probe,
    production_projected_pcg_first_step_probe, production_volume_map_calibration,
};

pub(crate) fn counterfactual_substep(
    prior: &AcceptedFrame,
    geometry: Geometry,
    boundary: &[BoundarySample],
    execution_profile_root: &[u8; 32],
    scenario_root: &[u8; 32],
    density_maximum_iterations: u8,
) -> Result<StepOutcome, WaterError> {
    if density_maximum_iterations < DENSITY_MIN_ITERATIONS {
        return Err(WaterError::new(
            AUDIT_INVALID,
            format!(
                "counterfactual density ceiling {density_maximum_iterations} is below the minimum"
            ),
        ));
    }
    substep_with_density_limit(
        prior,
        geometry,
        boundary,
        execution_profile_root,
        scenario_root,
        density_maximum_iterations,
    )
}

pub(crate) struct ContactConstrainedStepOutcome {
    pub(crate) outcome: StepOutcome,
    pub(crate) projection: VelocityProjectionResult,
    pub(crate) energy: StepEnergyTrace,
}

pub(crate) fn contact_constrained_substep_with_limit(
    prior: &AcceptedFrame,
    geometry: Geometry,
    boundary: &[BoundarySample],
    execution_profile_root: &[u8; 32],
    scenario_root: &[u8; 32],
    density_maximum_iterations: u8,
) -> Result<ContactConstrainedStepOutcome, WaterError> {
    if density_maximum_iterations < DENSITY_MIN_ITERATIONS {
        return Err(WaterError::new(
            AUDIT_INVALID,
            format!(
                "contact counterfactual density ceiling {density_maximum_iterations} is below the minimum"
            ),
        ));
    }
    if geometry.aperture.is_some() {
        return Err(WaterError::new(
            AUDIT_INVALID,
            "predictive outer-box contact does not define internal aperture contact",
        ));
    }
    let (outcome, projection, energy) = substep_with_boundary_projection_limit(
        prior,
        geometry,
        BoundaryInput::Particles(boundary),
        execution_profile_root,
        scenario_root,
        density_maximum_iterations,
        TerminalVelocityProjection::PredictiveOuterBox,
        DensitySolveMethod::RelaxedJacobi,
    )?;
    Ok(ContactConstrainedStepOutcome {
        outcome,
        projection,
        energy,
    })
}

pub(crate) fn contact_pcg_constrained_substep(
    prior: &AcceptedFrame,
    geometry: Geometry,
    boundary: &[BoundarySample],
    execution_profile_root: &[u8; 32],
    scenario_root: &[u8; 32],
    density_maximum_iterations: u8,
) -> Result<ContactConstrainedStepOutcome, WaterError> {
    if density_maximum_iterations < DENSITY_MIN_ITERATIONS {
        return Err(WaterError::new(
            AUDIT_INVALID,
            format!(
                "projected PCG density ceiling {density_maximum_iterations} is below the minimum"
            ),
        ));
    }
    if geometry.aperture.is_some() {
        return Err(WaterError::new(
            AUDIT_INVALID,
            "predictive outer-box contact does not define internal aperture contact",
        ));
    }
    let (outcome, projection, energy) = substep_with_boundary_projection_limit(
        prior,
        geometry,
        BoundaryInput::Particles(boundary),
        execution_profile_root,
        scenario_root,
        density_maximum_iterations,
        TerminalVelocityProjection::PredictiveOuterBox,
        DensitySolveMethod::ProjectedPreconditionedConjugateGradient,
    )?;
    Ok(ContactConstrainedStepOutcome {
        outcome,
        projection,
        energy,
    })
}

pub(crate) fn successor_pcg_constrained_substep(
    prior: &AcceptedFrame,
    geometry: Geometry,
    boundary: &[BoundarySample],
    execution_profile_root: &[u8; 32],
    scenario_root: &[u8; 32],
) -> Result<ContactConstrainedStepOutcome, WaterError> {
    let (outcome, projection, energy) = substep_with_boundary_projection_limit(
        prior,
        geometry,
        BoundaryInput::Particles(boundary),
        execution_profile_root,
        scenario_root,
        50,
        TerminalVelocityProjection::PredictiveGeometry,
        DensitySolveMethod::ProjectedPreconditionedConjugateGradient,
    )?;
    Ok(ContactConstrainedStepOutcome {
        outcome,
        projection,
        energy,
    })
}

pub(crate) fn production_successor_density_fixtures(
    samples: &[CanonicalSample],
    geometry: Geometry,
    boundary: &[BoundarySample],
    selections: &[(&str, u32)],
) -> Result<Vec<DensityFixtureObservation>, WaterError> {
    let state = decode(samples)?;
    let reconstruction = reconstruct(&state, geometry, boundary)?;
    let mut observations = Vec::new();
    observations
        .try_reserve_exact(selections.len())
        .map_err(audit_reserve_error)?;
    for (role, sample_id) in selections {
        let index = state
            .samples
            .binary_search_by_key(sample_id, |sample| sample.id)
            .map_err(|_| {
                WaterError::new(
                    AUDIT_INVALID,
                    format!("successor density fixture sample {sample_id} is missing"),
                )
            })?;
        let row = reconstruction.rows[index];
        let mut fluid_gradient = Vec3f::ZERO;
        for neighbor in &reconstruction.fluid[row.fluid_start..row.fluid_end] {
            fluid_gradient = fluid_gradient
                .add(
                    neighbor
                        .gradient
                        .scale(REST_VOLUME)
                        .checked("successor fixture fluid gradient")?,
                )
                .checked("successor fixture fluid gradient reduction")?;
        }
        let mut boundary_gradient = Vec3f::ZERO;
        for neighbor in &reconstruction.solid[row.solid_start..row.solid_end] {
            boundary_gradient = boundary_gradient
                .add(
                    neighbor
                        .gradient
                        .scale(neighbor.volume)
                        .checked("successor fixture boundary gradient")?,
                )
                .checked("successor fixture boundary gradient reduction")?;
        }
        let total_gradient = fluid_gradient
            .add(boundary_gradient)
            .checked("successor fixture total gradient")?;
        observations.push(DensityFixtureObservation {
            role: (*role).to_owned(),
            sample_id: *sample_id,
            position_um: state.samples[index].position_um,
            fluid_neighbor_count: row.fluid_end - row.fluid_start,
            boundary_neighbor_count: row.solid_end - row.solid_start,
            rho_ratio_bits: scalar_bits(reconstruction.rho_ratio[index]),
            alpha_bits: scalar_bits(reconstruction.alpha[index]),
            fluid_gradient_bits: vector_bits(fluid_gradient),
            boundary_gradient_bits: vector_bits(boundary_gradient),
            total_gradient_bits: vector_bits(total_gradient),
        });
    }
    Ok(observations)
}

pub(crate) fn production_successor_contact_fixtures(
    geometry: Geometry,
    inputs: &[ContactFixtureInput],
) -> Result<Vec<ContactFixtureObservation>, WaterError> {
    let mut observations = Vec::new();
    observations
        .try_reserve_exact(inputs.len())
        .map_err(audit_reserve_error)?;
    for input in inputs {
        let position = Vec3f::new(
            decode_micrometres(input.position_um.x)?,
            decode_micrometres(input.position_um.y)?,
            decode_micrometres(input.position_um.z)?,
        );
        let before = Vec3f::new(
            decode_velocity(input.velocity_um_s.x)?,
            decode_velocity(input.velocity_um_s.y)?,
            decode_velocity(input.velocity_um_s.z)?,
        );
        let mut velocities = [before];
        let projection = project_predictive_geometry(geometry, &[position], &mut velocities)?;
        let mut features = Vec::new();
        for (feature_id, active_constraints) in projection
            .feature_active_constraints
            .iter()
            .copied()
            .enumerate()
        {
            if active_constraints == 0 {
                continue;
            }
            features.push(FeatureActivation {
                feature_id: u32::try_from(feature_id).map_err(|_| {
                    WaterError::new(AUDIT_INVALID, "contact fixture feature ID overflow")
                })?,
                active_constraints,
                fluid_impulse_bits: vector_bits(projection.feature_fluid_impulses[feature_id]),
            });
        }
        observations.push(ContactFixtureObservation {
            id: input.id.to_owned(),
            position_um: input.position_um,
            velocity_before_um_s: input.velocity_um_s,
            velocity_after_bits: vector_bits(velocities[0]),
            fluid_impulse_bits: vector_bits(projection.fluid_impulse),
            active_rows: projection.active_rows,
            active_components: projection.active_components,
            features,
        });
    }
    Ok(observations)
}

pub(crate) fn production_contact_projection_probe() -> Result<ContactProjectionProbe, WaterError> {
    let geometry = Geometry {
        bounds: crate::model::Box3i {
            min: Vec3i::new(0, 0, 0),
            max: Vec3i::new(1_000_000, 1_000_000, 1_000_000),
        },
        aperture: None,
    };
    let inputs = [
        (
            "lower-face-inward",
            Vec3i::new(25_000, 500_000, 500_000),
            Vec3i::new(-200_000, 0, 0),
        ),
        (
            "lower-face-separating",
            Vec3i::new(25_000, 500_000, 500_000),
            Vec3i::new(200_000, 0, 0),
        ),
        (
            "upper-face-inward",
            Vec3i::new(500_000, 975_000, 500_000),
            Vec3i::new(0, 300_000, 0),
        ),
        (
            "lower-edge-inward",
            Vec3i::new(25_000, 25_000, 500_000),
            Vec3i::new(-100_000, -200_000, 50_000),
        ),
        (
            "mixed-corner-inward",
            Vec3i::new(975_000, 25_000, 975_000),
            Vec3i::new(100_000, -200_000, 300_000),
        ),
        (
            "admitted-shallow-position",
            Vec3i::new(24_000, 500_000, 500_000),
            Vec3i::new(0, 0, 0),
        ),
        (
            "interior-no-hit",
            Vec3i::new(30_000, 500_000, 500_000),
            Vec3i::new(-1_000_000, 0, 0),
        ),
        (
            "interior-predicted-hit",
            Vec3i::new(26_000, 500_000, 500_000),
            Vec3i::new(-1_000_000, 0, 0),
        ),
    ];
    let mut cases = Vec::new();
    cases
        .try_reserve_exact(inputs.len())
        .map_err(audit_reserve_error)?;
    for (id, position_um, velocity_um_s) in inputs {
        let position = Vec3f::new(
            decode_micrometres(position_um.x)?,
            decode_micrometres(position_um.y)?,
            decode_micrometres(position_um.z)?,
        );
        let before = Vec3f::new(
            decode_velocity(velocity_um_s.x)?,
            decode_velocity(velocity_um_s.y)?,
            decode_velocity(velocity_um_s.z)?,
        );
        let mut velocities = [before];
        let projection = project_predictive_outer_box(geometry, &[position], &mut velocities)?;
        cases.push(ContactProjectionCase {
            id,
            position_um,
            velocity_before_bits: vector_bits(before),
            velocity_after_bits: vector_bits(velocities[0]),
            fluid_impulse_bits: vector_bits(projection.fluid_impulse),
            active_components: projection.active_components,
        });
    }
    Ok(ContactProjectionProbe { cases })
}

pub(super) struct DensityRecorder {
    selected_indices: Vec<usize>,
    rows: Vec<HydroRowTrace>,
    error_ppb_by_iteration: Vec<i64>,
}

pub(super) struct IterationObservation {
    pub(super) multiplier_in: f64,
    pub(super) acceleration: Vec3f,
    pub(super) matrix_action: f64,
    pub(super) error: f64,
    pub(super) multiplier_out: f64,
}

impl DensityRecorder {
    fn new(
        state: &DecodedState,
        reconstruction: &Reconstruction,
        boundary: &[BoundarySample],
    ) -> Result<Self, WaterError> {
        let mut selected_indices = Vec::new();
        let mut rows = Vec::new();
        selected_indices
            .try_reserve_exact(SELECTED_ROWS.len())
            .map_err(audit_reserve_error)?;
        rows.try_reserve_exact(SELECTED_ROWS.len())
            .map_err(audit_reserve_error)?;
        for (sample_id, role) in SELECTED_ROWS {
            let index = state
                .samples
                .binary_search_by_key(&sample_id, |sample| sample.id)
                .map_err(|_| {
                    WaterError::new(
                        AUDIT_INVALID,
                        format!("selected hydro sample {sample_id} is missing"),
                    )
                })?;
            let row = reconstruction.rows[index];
            let mut fluid_neighbor_ids = Vec::new();
            fluid_neighbor_ids
                .try_reserve_exact(row.fluid_end - row.fluid_start)
                .map_err(audit_reserve_error)?;
            for neighbor in &reconstruction.fluid[row.fluid_start..row.fluid_end] {
                fluid_neighbor_ids.push(state.samples[neighbor.other].id);
            }
            let mut boundary_neighbors = Vec::new();
            boundary_neighbors
                .try_reserve_exact(row.solid_end - row.solid_start)
                .map_err(audit_reserve_error)?;
            for neighbor in &reconstruction.solid[row.solid_start..row.solid_end] {
                let sample = boundary[neighbor.boundary];
                boundary_neighbors.push(BoundaryNeighborTrace {
                    id: sample.id,
                    volume_bits: scalar_bits(sample.volume),
                });
            }
            let mut iterations = Vec::new();
            iterations
                .try_reserve_exact(usize::from(DENSITY_MAX_ITERATIONS))
                .map_err(audit_reserve_error)?;
            selected_indices.push(index);
            rows.push(HydroRowTrace {
                role: role.to_owned(),
                sample_id,
                position_um: state.samples[index].position_um,
                fluid_neighbor_ids,
                boundary_neighbors,
                initial: Some(DensityInitialTrace {
                    rho_ratio_bits: scalar_bits(reconstruction.rho_ratio[index]),
                    alpha_bits: scalar_bits(reconstruction.alpha[index]),
                    rho_adv_bits: String::new(),
                    factor_bits: String::new(),
                    multiplier_bits: String::new(),
                }),
                iterations,
            });
        }
        let mut error_ppb_by_iteration = Vec::new();
        error_ppb_by_iteration
            .try_reserve_exact(usize::from(DENSITY_MAX_ITERATIONS))
            .map_err(audit_reserve_error)?;
        Ok(Self {
            selected_indices,
            rows,
            error_ppb_by_iteration,
        })
    }

    pub(super) fn capture_initial(
        &mut self,
        rho_adv: &[f64],
        factor: &[f64],
        multiplier: &[f64],
    ) -> Result<(), WaterError> {
        for (slot, index) in self
            .rows
            .iter_mut()
            .zip(self.selected_indices.iter().copied())
        {
            let initial = slot.initial.as_mut().ok_or_else(|| {
                WaterError::new(
                    AUDIT_INVALID,
                    "selected row reconstruction trace is missing",
                )
            })?;
            initial.rho_adv_bits = scalar_bits(rho_adv[index]);
            initial.factor_bits = scalar_bits(factor[index]);
            initial.multiplier_bits = scalar_bits(multiplier[index]);
        }
        Ok(())
    }

    pub(super) fn capture_iteration_row(
        &mut self,
        iteration: u8,
        index: usize,
        observation: IterationObservation,
    ) -> Result<(), WaterError> {
        let Some(slot) = self
            .selected_indices
            .iter()
            .position(|selected| *selected == index)
        else {
            return Ok(());
        };
        self.rows[slot].iterations.push(DensityIterationRowTrace {
            iteration,
            multiplier_in_bits: scalar_bits(observation.multiplier_in),
            acceleration_bits: vector_bits(observation.acceleration),
            matrix_action_bits: scalar_bits(observation.matrix_action),
            error_bits: scalar_bits(observation.error),
            multiplier_out_bits: scalar_bits(observation.multiplier_out),
        });
        Ok(())
    }

    pub(super) fn capture_iteration_error(&mut self, error_ppb: i64) -> Result<(), WaterError> {
        self.error_ppb_by_iteration.push(error_ppb);
        Ok(())
    }

    fn finish(
        self,
        divergence: SolveResult,
        terminal_code: String,
        terminal_detail: String,
    ) -> HydroAuditTrace {
        HydroAuditTrace {
            divergence_iterations: divergence.iterations,
            divergence_error_ppb: divergence.error_ppb,
            density_error_ppb_by_iteration: self.error_ppb_by_iteration,
            density_terminal_code: terminal_code,
            density_terminal_detail: terminal_detail,
            rows: self.rows,
        }
    }
}

pub(crate) fn production_hydro_audit(
    samples: &[CanonicalSample],
    geometry: Geometry,
    boundary: &[BoundarySample],
) -> Result<AuditComputation, WaterError> {
    let mut state = decode(samples)?;
    let reconstruction = reconstruct(&state, geometry, boundary)?;
    let divergence = solve_divergence(&reconstruction, &mut state.velocities)?;
    for velocity in &mut state.velocities {
        velocity.y = checked_scalar(
            velocity.y + (DT * -GRAVITY_MAGNITUDE),
            "audit production gravity velocity y",
        )?;
    }
    let mut recorder = DensityRecorder::new(&state, &reconstruction, boundary)?;
    let density =
        solve_density_with_recorder(&reconstruction, &mut state.velocities, Some(&mut recorder));
    let (terminal_code, terminal_detail) = match density {
        Ok(result) => (
            "COMPLETED".to_owned(),
            format!(
                "converged at iteration {} with {} ppb",
                result.iterations, result.error_ppb
            ),
        ),
        Err(error) if error.code() == DENSITY_NONCONVERGENCE => {
            (error.code().to_owned(), error.detail().to_owned())
        }
        Err(error) => return Err(error),
    };

    let mut boundary_input = Vec::new();
    boundary_input
        .try_reserve_exact(boundary.len())
        .map_err(audit_reserve_error)?;
    for sample in boundary {
        boundary_input.push(AuditBoundaryInput {
            id: sample.id,
            position_um: sample.position_um,
            volume_bits: scalar_bits(sample.volume),
        });
    }
    let fluid = production_fluid_input(&state.samples)?;
    Ok(AuditComputation {
        fluid,
        boundary: boundary_input,
        trace: recorder.finish(divergence, terminal_code, terminal_detail),
    })
}

fn audit_reserve_error(error: std::collections::TryReserveError) -> WaterError {
    WaterError::new(
        AUDIT_INVALID,
        format!("hydro audit allocation failed: {error}"),
    )
}
