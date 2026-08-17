#![forbid(unsafe_code)]

use crate::audit::{
    AuditBoundaryInput, AuditComputation, BoundaryNeighborTrace, DensityInitialTrace,
    DensityIterationRowTrace, HydroAuditTrace, HydroRowTrace, SELECTED_ROWS,
    production_fluid_input, scalar_bits, vector_bits,
};
use crate::error::{AUDIT_INVALID, DENSITY_NONCONVERGENCE, WaterError};

use super::*;

mod calibration;

pub(crate) use calibration::{production_hydro_calibration, production_volume_map_calibration};

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
    boundary: &[BoundarySample],
) -> Result<AuditComputation, WaterError> {
    let mut state = decode(samples)?;
    let reconstruction = reconstruct(&state, boundary)?;
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
