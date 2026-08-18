#![forbid(unsafe_code)]

use std::mem::size_of;

use crate::boundary::BoundarySample;
use crate::error::{
    DECODED_HEAP_CAPACITY_EXCEEDED, DENSITY_NONCONVERGENCE, DIVERGENCE_NONCONVERGENCE,
    DUPLICATE_SAMPLE_ID, NEIGHBOR_CAPACITY_EXCEEDED, NONFINITE_VALUE, NUMERIC_OVERFLOW, WaterError,
};
use crate::hash;
use crate::kernel;
use crate::model::{
    AcceptedFrame, CanonicalSample, Geometry, StepSummary, Vec3f, Vec3i, checked_scalar,
};
use crate::profile::{
    DT, DT2, GRAVITY_MAGNITUDE, INV_DT, INV_DT2, MAXIMUM_DECODED_HEAP_BYTES,
    MAXIMUM_DIRECTED_FLUID_NEIGHBORS, MAXIMUM_NEIGHBORS_PER_FLUID_ROW, RELAXATION, REST_VOLUME,
    RHO0, SOLVER_EPSILON, decode_micrometres, decode_velocity, quantize_micrometres, quantize_ppb,
    quantize_velocity,
};
use crate::scenario::{validate_capacity, validate_sample_identity};

mod constraint;
mod diagnostic;
mod energy;
mod neighborhood;
mod reconstruction;
mod statistics;

use constraint::{
    VelocityProjectionResult, checked_dot, density_pressure_operator, project_predictive_geometry,
    project_predictive_outer_box, solve_density_accelerated_projected_gradient,
    solve_density_projected_pcg,
};
pub(crate) use diagnostic::{
    ContactConstrainedStepOutcome, contact_constrained_substep_with_limit,
    contact_pcg_constrained_substep, counterfactual_substep,
    production_accelerated_pressure_first_step_probe, production_contact_projection_probe,
    production_hydro_audit, production_hydro_calibration, production_pressure_operator_probe,
    production_projected_pcg_first_step_probe, production_successor_contact_fixtures,
    production_successor_density_fixtures, production_volume_map_calibration,
    successor_accelerated_projected_gradient_substep, successor_pcg_constrained_substep,
};
pub(crate) use energy::StepEnergyTrace;
use energy::mechanical_energy;
use neighborhood::{admitted_boundary, admitted_fluid, build_boundary_grid, build_fluid_grid};
use statistics::{centre_of_mass, density_ratio_percentiles};

const DENSITY_MIN_ITERATIONS: u8 = 2;
const DENSITY_MAX_ITERATIONS: u8 = 20;
const DENSITY_THRESHOLD_PPB: i64 = 100_000;
const DIVERGENCE_MIN_ITERATIONS: u8 = 1;
const DIVERGENCE_MAX_ITERATIONS: u8 = 20;
const DIVERGENCE_THRESHOLD_PPB: i64 = 1_000_000;

#[derive(Clone, Copy, Debug)]
struct FluidNeighbor {
    other: usize,
    value: f64,
    gradient: Vec3f,
}

#[derive(Clone, Copy, Debug)]
struct SolidNeighbor {
    boundary: usize,
    volume: f64,
    value: f64,
    gradient: Vec3f,
    feature_rank: usize,
}

#[derive(Clone, Copy, Debug, Default)]
struct Row {
    fluid_start: usize,
    fluid_end: usize,
    solid_start: usize,
    solid_end: usize,
}

impl Row {
    fn neighbor_count(self) -> usize {
        (self.fluid_end - self.fluid_start) + (self.solid_end - self.solid_start)
    }
}

struct Reconstruction {
    rows: Vec<Row>,
    fluid: Vec<FluidNeighbor>,
    solid: Vec<SolidNeighbor>,
    rho_ratio: Vec<f64>,
    alpha: Vec<f64>,
}

struct DecodedState {
    samples: Vec<CanonicalSample>,
    positions: Vec<Vec3f>,
    velocities: Vec<Vec3f>,
}

#[derive(Clone, Copy, Debug)]
struct SolveResult {
    iterations: u8,
    error_ppb: i64,
    kkt_error_ppb: Option<i64>,
    maximum_multiplier_bits: u64,
    boundary_impulse: Vec3f,
}

struct AccelerationBatch {
    total: Vec<Vec3f>,
    boundary: Vec<Vec3f>,
}

pub(crate) struct StepOutcome {
    pub(crate) frame: AcceptedFrame,
    pub(crate) summary: StepSummary,
    pub(crate) boundary_impulses: [Vec3f; 2],
}

#[derive(Clone, Copy)]
enum TerminalVelocityProjection {
    None,
    PredictiveOuterBox,
    PredictiveGeometry,
}

#[derive(Clone, Copy)]
enum DensitySolveMethod {
    RelaxedJacobi,
    ProjectedPreconditionedConjugateGradient,
    AcceleratedProjectedGradient,
}

#[derive(Clone, Copy)]
enum BoundaryInput<'a> {
    Particles(&'a [BoundarySample]),
    VolumeMap(Geometry),
}

pub(crate) fn initial_frame(
    samples: Vec<CanonicalSample>,
    geometry: Geometry,
    boundary: &[BoundarySample],
    execution_profile_root: &[u8; 32],
    scenario_root: &[u8; 32],
) -> Result<(AcceptedFrame, StepSummary), WaterError> {
    initial_frame_with_boundary(
        samples,
        geometry,
        BoundaryInput::Particles(boundary),
        execution_profile_root,
        scenario_root,
    )
}

pub(crate) fn initial_frame_volume_map(
    samples: Vec<CanonicalSample>,
    geometry: Geometry,
    execution_profile_root: &[u8; 32],
    scenario_root: &[u8; 32],
) -> Result<(AcceptedFrame, StepSummary), WaterError> {
    initial_frame_with_boundary(
        samples,
        geometry,
        BoundaryInput::VolumeMap(geometry),
        execution_profile_root,
        scenario_root,
    )
}

fn initial_frame_with_boundary(
    mut samples: Vec<CanonicalSample>,
    geometry: Geometry,
    boundary: BoundaryInput<'_>,
    execution_profile_root: &[u8; 32],
    scenario_root: &[u8; 32],
) -> Result<(AcceptedFrame, StepSummary), WaterError> {
    samples.sort_unstable_by_key(|sample| sample.id);
    validate_sample_identity(&samples)?;
    validate_canonical_bounds(&samples)?;
    let decoded = decode(&samples)?;
    let reconstruction = reconstruct_boundary(&decoded, geometry, boundary)?;
    let density_percentiles = density_ratio_percentiles(&reconstruction.rho_ratio)?;
    let penetration = crate::boundary::validate_centres(geometry, &samples)?;
    let frame_root = hash::frame_root(execution_profile_root, scenario_root, 0, &samples)?;
    let centre_of_mass_um = centre_of_mass(&samples)?;
    Ok((
        AcceptedFrame {
            step: 0,
            samples,
            frame_root,
        },
        StepSummary {
            step: 0,
            frame_root: hash::hex(&frame_root),
            density_iterations: 0,
            density_error_ppb: 0,
            density_kkt_error_ppb: None,
            density_maximum_multiplier_bits: "0x0000000000000000".to_owned(),
            density_ratio_p50_ppb: density_percentiles[0],
            density_ratio_p95_ppb: density_percentiles[1],
            density_ratio_p99_ppb: density_percentiles[2],
            divergence_iterations: 0,
            divergence_error_ppb: 0,
            divergence_maximum_multiplier_bits: "0x0000000000000000".to_owned(),
            maximum_penetration_um: penetration,
            centre_of_mass_um,
        },
    ))
}

pub(crate) fn substep(
    prior: &AcceptedFrame,
    geometry: Geometry,
    boundary: &[BoundarySample],
    execution_profile_root: &[u8; 32],
    scenario_root: &[u8; 32],
) -> Result<StepOutcome, WaterError> {
    substep_with_density_limit(
        prior,
        geometry,
        boundary,
        execution_profile_root,
        scenario_root,
        DENSITY_MAX_ITERATIONS,
    )
}

pub(crate) fn substep_volume_map(
    prior: &AcceptedFrame,
    geometry: Geometry,
    execution_profile_root: &[u8; 32],
    scenario_root: &[u8; 32],
) -> Result<StepOutcome, WaterError> {
    substep_with_boundary_limit(
        prior,
        geometry,
        BoundaryInput::VolumeMap(geometry),
        execution_profile_root,
        scenario_root,
        DENSITY_MAX_ITERATIONS,
    )
}

fn substep_with_density_limit(
    prior: &AcceptedFrame,
    geometry: Geometry,
    boundary: &[BoundarySample],
    execution_profile_root: &[u8; 32],
    scenario_root: &[u8; 32],
    density_maximum_iterations: u8,
) -> Result<StepOutcome, WaterError> {
    substep_with_boundary_limit(
        prior,
        geometry,
        BoundaryInput::Particles(boundary),
        execution_profile_root,
        scenario_root,
        density_maximum_iterations,
    )
}

fn substep_with_boundary_limit(
    prior: &AcceptedFrame,
    geometry: Geometry,
    boundary: BoundaryInput<'_>,
    execution_profile_root: &[u8; 32],
    scenario_root: &[u8; 32],
    density_maximum_iterations: u8,
) -> Result<StepOutcome, WaterError> {
    substep_with_boundary_projection_limit(
        prior,
        geometry,
        boundary,
        execution_profile_root,
        scenario_root,
        density_maximum_iterations,
        TerminalVelocityProjection::None,
        DensitySolveMethod::RelaxedJacobi,
    )
    .map(|(outcome, _projection, _energy)| outcome)
}

#[allow(clippy::too_many_arguments)]
fn substep_with_boundary_projection_limit(
    prior: &AcceptedFrame,
    geometry: Geometry,
    boundary: BoundaryInput<'_>,
    execution_profile_root: &[u8; 32],
    scenario_root: &[u8; 32],
    density_maximum_iterations: u8,
    velocity_projection: TerminalVelocityProjection,
    density_method: DensitySolveMethod,
) -> Result<(StepOutcome, VelocityProjectionResult, StepEnergyTrace), WaterError> {
    let mut state = decode(&prior.samples)?;
    if state.samples.is_empty() {
        return publish_empty(prior.step, execution_profile_root, scenario_root, geometry).map(
            |outcome| {
                (
                    outcome,
                    VelocityProjectionResult::default(),
                    StepEnergyTrace::default(),
                )
            },
        );
    }
    let reconstruction = reconstruct_boundary(&state, geometry, boundary)?;
    let density_percentiles = density_ratio_percentiles(&reconstruction.rho_ratio)?;

    let mut energy = StepEnergyTrace {
        decoded: mechanical_energy(&state.positions, &state.velocities)?,
        ..StepEnergyTrace::default()
    };

    let divergence = solve_divergence(&reconstruction, &mut state.velocities)?;
    energy.after_divergence = mechanical_energy(&state.positions, &state.velocities)?;
    for (index, velocity) in state.velocities.iter_mut().enumerate() {
        velocity.y = checked_scalar(velocity.y + (DT * -GRAVITY_MAGNITUDE), "gravity velocity y")?;
        (*velocity).checked(&format!("gravity sample {}", state.samples[index].id))?;
    }
    energy.after_gravity = mechanical_energy(&state.positions, &state.velocities)?;
    let density = match density_method {
        DensitySolveMethod::RelaxedJacobi => solve_density_with_limit(
            &reconstruction,
            &mut state.velocities,
            None,
            density_maximum_iterations,
        )?,
        DensitySolveMethod::ProjectedPreconditionedConjugateGradient => {
            solve_density_projected_pcg(
                &reconstruction,
                &mut state.velocities,
                density_maximum_iterations,
            )?
        }
        DensitySolveMethod::AcceleratedProjectedGradient => {
            solve_density_accelerated_projected_gradient(
                &reconstruction,
                &mut state.velocities,
                density_maximum_iterations,
            )?
        }
    };
    energy.after_density = mechanical_energy(&state.positions, &state.velocities)?;

    let projection = match velocity_projection {
        TerminalVelocityProjection::None => VelocityProjectionResult::default(),
        TerminalVelocityProjection::PredictiveOuterBox => {
            project_predictive_outer_box(geometry, &state.positions, &mut state.velocities)?
        }
        TerminalVelocityProjection::PredictiveGeometry => {
            project_predictive_geometry(geometry, &state.positions, &mut state.velocities)?
        }
    };
    energy.after_contact = mechanical_energy(&state.positions, &state.velocities)?;

    for index in 0..state.samples.len() {
        let displacement = state.velocities[index]
            .scale(DT)
            .checked("position integration displacement")?;
        state.positions[index] = state.positions[index]
            .add(displacement)
            .checked("position integration")?;
    }
    energy.after_integration = mechanical_energy(&state.positions, &state.velocities)?;
    let next_step = prior
        .step
        .checked_add(1)
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "step number overflow"))?;
    let samples = publish(&state)?;
    let published_state = decode(&samples)?;
    energy.after_publication =
        mechanical_energy(&published_state.positions, &published_state.velocities)?;
    let maximum_penetration_um = crate::boundary::validate_centres(geometry, &samples)?;
    crate::boundary::validate_transition(geometry, &prior.samples, &samples)?;
    let frame_root = hash::frame_root(execution_profile_root, scenario_root, next_step, &samples)?;
    let centre_of_mass_um = centre_of_mass(&samples)?;
    Ok((
        StepOutcome {
            frame: AcceptedFrame {
                step: next_step,
                samples,
                frame_root,
            },
            summary: StepSummary {
                step: next_step,
                frame_root: hash::hex(&frame_root),
                density_iterations: density.iterations,
                density_error_ppb: density.error_ppb,
                density_kkt_error_ppb: density.kkt_error_ppb,
                density_maximum_multiplier_bits: format!(
                    "0x{:016x}",
                    density.maximum_multiplier_bits
                ),
                density_ratio_p50_ppb: density_percentiles[0],
                density_ratio_p95_ppb: density_percentiles[1],
                density_ratio_p99_ppb: density_percentiles[2],
                divergence_iterations: divergence.iterations,
                divergence_error_ppb: divergence.error_ppb,
                divergence_maximum_multiplier_bits: format!(
                    "0x{:016x}",
                    divergence.maximum_multiplier_bits
                ),
                maximum_penetration_um,
                centre_of_mass_um,
            },
            boundary_impulses: [divergence.boundary_impulse, density.boundary_impulse],
        },
        projection,
        energy,
    ))
}

fn publish_empty(
    prior_step: u32,
    execution_profile_root: &[u8; 32],
    scenario_root: &[u8; 32],
    geometry: Geometry,
) -> Result<StepOutcome, WaterError> {
    let step = prior_step
        .checked_add(1)
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "step number overflow"))?;
    let samples = Vec::new();
    let maximum_penetration_um = crate::boundary::validate_centres(geometry, &samples)?;
    let frame_root = hash::frame_root(execution_profile_root, scenario_root, step, &samples)?;
    Ok(StepOutcome {
        frame: AcceptedFrame {
            step,
            samples,
            frame_root,
        },
        summary: StepSummary {
            step,
            frame_root: hash::hex(&frame_root),
            density_iterations: DENSITY_MIN_ITERATIONS,
            density_error_ppb: 0,
            density_kkt_error_ppb: None,
            density_maximum_multiplier_bits: "0x0000000000000000".to_owned(),
            density_ratio_p50_ppb: 0,
            density_ratio_p95_ppb: 0,
            density_ratio_p99_ppb: 0,
            divergence_iterations: DIVERGENCE_MIN_ITERATIONS,
            divergence_error_ppb: 0,
            divergence_maximum_multiplier_bits: "0x0000000000000000".to_owned(),
            maximum_penetration_um,
            centre_of_mass_um: Vec3i::new(0, 0, 0),
        },
        boundary_impulses: [Vec3f::ZERO, Vec3f::ZERO],
    })
}

fn decode(samples: &[CanonicalSample]) -> Result<DecodedState, WaterError> {
    validate_sample_identity(samples)?;
    let mut sorted = Vec::new();
    sorted
        .try_reserve_exact(samples.len())
        .map_err(heap_error)?;
    sorted.extend_from_slice(samples);
    sorted.sort_unstable_by_key(|sample| sample.id);
    for pair in sorted.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(WaterError::new(
                DUPLICATE_SAMPLE_ID,
                format!("duplicate SampleId {}", pair[0].id),
            ));
        }
    }
    validate_canonical_bounds(&sorted)?;
    let mut positions = Vec::new();
    let mut velocities = Vec::new();
    positions
        .try_reserve_exact(sorted.len())
        .map_err(heap_error)?;
    velocities
        .try_reserve_exact(sorted.len())
        .map_err(heap_error)?;
    for sample in &sorted {
        positions.push(Vec3f::new(
            decode_micrometres(sample.position_um.x)?,
            decode_micrometres(sample.position_um.y)?,
            decode_micrometres(sample.position_um.z)?,
        ));
        velocities.push(Vec3f::new(
            decode_velocity(sample.velocity_um_s.x)?,
            decode_velocity(sample.velocity_um_s.y)?,
            decode_velocity(sample.velocity_um_s.z)?,
        ));
    }
    Ok(DecodedState {
        samples: sorted,
        positions,
        velocities,
    })
}

fn reconstruct_boundary(
    state: &DecodedState,
    geometry: Geometry,
    boundary: BoundaryInput<'_>,
) -> Result<Reconstruction, WaterError> {
    match boundary {
        BoundaryInput::Particles(samples) => reconstruct(state, geometry, samples),
        BoundaryInput::VolumeMap(geometry) => reconstruct_volume_map(state, geometry),
    }
}

fn reconstruct(
    state: &DecodedState,
    geometry: Geometry,
    boundary: &[BoundarySample],
) -> Result<Reconstruction, WaterError> {
    reconstruction::particles(state, geometry, boundary)
}

fn reconstruct_volume_map(
    state: &DecodedState,
    geometry: Geometry,
) -> Result<Reconstruction, WaterError> {
    reconstruction::volume_map(state, geometry)
}

fn solve_divergence(
    reconstruction: &Reconstruction,
    velocities: &mut [Vec3f],
) -> Result<SolveResult, WaterError> {
    let count = velocities.len();
    let mut source = filled_vec(count, 0.0)?;
    let mut factor = filled_vec(count, 0.0)?;
    let mut multiplier = filled_vec(count, 0.0)?;
    let mut next = filled_vec(count, 0.0)?;
    for index in 0..count {
        let row = reconstruction.rows[index];
        let mut d = divergence_source(index, row, reconstruction, velocities)?;
        if row.neighbor_count() < 20 {
            d = 0.0;
        }
        source[index] = checked_scalar(d, "divergence source")?;
        factor[index] = checked_scalar(reconstruction.alpha[index] * INV_DT, "divergence factor")?;
        let positive_d = if d > 0.0 { d } else { 0.0 };
        multiplier[index] = checked_scalar(positive_d * factor[index], "divergence initial k")?;
    }
    let mut accepted_acceleration = None;
    let mut error_ppb = i64::MAX;
    let mut accepted_iteration = 0_u8;
    for iteration in 1..=DIVERGENCE_MAX_ITERATIONS {
        let acceleration = pressure_acceleration(reconstruction, &multiplier)?;
        let matrix = matrix_action(reconstruction, &acceleration.total)?;
        let mut error_sum = 0.0;
        for index in 0..count {
            let s = checked_scalar(-source[index], "divergence s")?;
            let dt_a = checked_scalar(DT * matrix[index], "divergence dt A")?;
            let correction = checked_scalar((s - dt_a) * factor[index], "divergence correction")?;
            let candidate = checked_scalar(
                multiplier[index] - (RELAXATION * correction),
                "divergence next candidate",
            )?;
            next[index] = if candidate > 0.0 { candidate } else { 0.0 };
            let error = checked_scalar(source[index] + dt_a, "divergence error")?;
            let error = if error > 0.0 { error } else { 0.0 };
            error_sum = checked_scalar(error_sum + error, "divergence error reduction")?;
        }
        let mean = checked_scalar(error_sum / (count as f64), "divergence error mean")?;
        error_ppb = quantize_ppb(checked_scalar(DT * mean, "divergence branch metric")?)?;
        std::mem::swap(&mut multiplier, &mut next);
        if converged(
            iteration,
            DIVERGENCE_MIN_ITERATIONS,
            error_ppb,
            DIVERGENCE_THRESHOLD_PPB,
        ) {
            accepted_iteration = iteration;
            accepted_acceleration = Some(pressure_acceleration(reconstruction, &multiplier)?);
            break;
        }
    }
    if accepted_iteration == 0 {
        return Err(WaterError::new(
            DIVERGENCE_NONCONVERGENCE,
            format!("iteration 20 ended at {error_ppb} ppb"),
        ));
    }
    let boundary_impulse = apply_acceleration(
        velocities,
        &accepted_acceleration.ok_or_else(|| {
            WaterError::new(
                DIVERGENCE_NONCONVERGENCE,
                "accepted acceleration is missing",
            )
        })?,
    )?;
    let maximum_multiplier_bits = maximum_multiplier_bits(&multiplier)?;
    Ok(SolveResult {
        iterations: accepted_iteration,
        error_ppb,
        kkt_error_ppb: None,
        maximum_multiplier_bits,
        boundary_impulse,
    })
}

fn solve_density_with_recorder(
    reconstruction: &Reconstruction,
    velocities: &mut [Vec3f],
    recorder: Option<&mut diagnostic::DensityRecorder>,
) -> Result<SolveResult, WaterError> {
    solve_density_with_limit(reconstruction, velocities, recorder, DENSITY_MAX_ITERATIONS)
}

fn solve_density_with_limit(
    reconstruction: &Reconstruction,
    velocities: &mut [Vec3f],
    mut recorder: Option<&mut diagnostic::DensityRecorder>,
    maximum_iterations: u8,
) -> Result<SolveResult, WaterError> {
    let count = velocities.len();
    let mut rho_adv = filled_vec(count, 0.0)?;
    let mut factor = filled_vec(count, 0.0)?;
    let mut multiplier = filled_vec(count, 0.0)?;
    let mut next = filled_vec(count, 0.0)?;
    for index in 0..count {
        let delta = divergence_source(
            index,
            reconstruction.rows[index],
            reconstruction,
            velocities,
        )?;
        rho_adv[index] = checked_scalar(
            reconstruction.rho_ratio[index] + (DT * delta),
            "advected density ratio",
        )?;
        factor[index] = checked_scalar(reconstruction.alpha[index] * INV_DT2, "density factor")?;
        let error = checked_scalar(rho_adv[index] - 1.0, "density initial error")?;
        let positive = if error > 0.0 { error } else { 0.0 };
        multiplier[index] = checked_scalar(positive * factor[index], "density initial k")?;
    }
    if let Some(recorder) = recorder.as_deref_mut() {
        recorder.capture_initial(&rho_adv, &factor, &multiplier)?;
    }
    let mut accepted_acceleration = None;
    let mut error_ppb = i64::MAX;
    let mut accepted_iteration = 0_u8;
    for iteration in 1..=maximum_iterations {
        let acceleration = pressure_acceleration(reconstruction, &multiplier)?;
        let matrix = matrix_action(reconstruction, &acceleration.total)?;
        let mut error_sum = 0.0;
        for index in 0..count {
            let s = checked_scalar(1.0 - rho_adv[index], "density s")?;
            let dt2_a = checked_scalar(DT2 * matrix[index], "density dt2 A")?;
            let correction = checked_scalar((s - dt2_a) * factor[index], "density correction")?;
            let candidate = checked_scalar(
                multiplier[index] - (RELAXATION * correction),
                "density next candidate",
            )?;
            next[index] = if candidate > 0.0 { candidate } else { 0.0 };
            let error = checked_scalar((rho_adv[index] + dt2_a) - 1.0, "density error")?;
            let error = if error > 0.0 { error } else { 0.0 };
            error_sum = checked_scalar(error_sum + error, "density error reduction")?;
            if let Some(recorder) = recorder.as_deref_mut() {
                recorder.capture_iteration_row(
                    iteration,
                    index,
                    diagnostic::IterationObservation {
                        multiplier_in: multiplier[index],
                        acceleration: acceleration.total[index],
                        matrix_action: matrix[index],
                        error,
                        multiplier_out: next[index],
                    },
                )?;
            }
        }
        let mean = checked_scalar(error_sum / (count as f64), "density error mean")?;
        error_ppb = quantize_ppb(mean)?;
        if let Some(recorder) = recorder.as_deref_mut() {
            recorder.capture_iteration_error(error_ppb)?;
        }
        std::mem::swap(&mut multiplier, &mut next);
        if converged(
            iteration,
            DENSITY_MIN_ITERATIONS,
            error_ppb,
            DENSITY_THRESHOLD_PPB,
        ) {
            accepted_iteration = iteration;
            accepted_acceleration = Some(pressure_acceleration(reconstruction, &multiplier)?);
            break;
        }
    }
    if accepted_iteration == 0 {
        return Err(WaterError::new(
            DENSITY_NONCONVERGENCE,
            format!("iteration {maximum_iterations} ended at {error_ppb} ppb"),
        ));
    }
    let boundary_impulse = apply_acceleration(
        velocities,
        &accepted_acceleration.ok_or_else(|| {
            WaterError::new(DENSITY_NONCONVERGENCE, "accepted acceleration is missing")
        })?,
    )?;
    let maximum_multiplier_bits = maximum_multiplier_bits(&multiplier)?;
    Ok(SolveResult {
        iterations: accepted_iteration,
        error_ppb,
        kkt_error_ppb: None,
        maximum_multiplier_bits,
        boundary_impulse,
    })
}

fn divergence_source(
    index: usize,
    row: Row,
    reconstruction: &Reconstruction,
    velocities: &[Vec3f],
) -> Result<f64, WaterError> {
    let mut result = 0.0;
    for neighbor in &reconstruction.fluid[row.fluid_start..row.fluid_end] {
        let relative = velocities[index]
            .sub(velocities[neighbor.other])
            .checked("fluid relative velocity")?;
        let term = checked_scalar(
            REST_VOLUME * relative.dot(neighbor.gradient),
            "fluid divergence term",
        )?;
        result = checked_scalar(result + term, "fluid divergence reduction")?;
    }
    for neighbor in &reconstruction.solid[row.solid_start..row.solid_end] {
        let term = checked_scalar(
            neighbor.volume * velocities[index].dot(neighbor.gradient),
            "boundary divergence term",
        )?;
        result = checked_scalar(result + term, "boundary divergence reduction")?;
    }
    Ok(result)
}

fn pressure_acceleration(
    reconstruction: &Reconstruction,
    multiplier: &[f64],
) -> Result<AccelerationBatch, WaterError> {
    let mut total = Vec::new();
    let mut boundary_acceleration = Vec::new();
    total
        .try_reserve_exact(multiplier.len())
        .map_err(heap_error)?;
    boundary_acceleration
        .try_reserve_exact(multiplier.len())
        .map_err(heap_error)?;
    for (index, row) in reconstruction.rows.iter().copied().enumerate() {
        let mut value = Vec3f::ZERO;
        for neighbor in &reconstruction.fluid[row.fluid_start..row.fluid_end] {
            let pressure_sum = checked_scalar(
                multiplier[index] + multiplier[neighbor.other],
                "fluid pressure sum",
            )?;
            if pressure_sum.abs() > SOLVER_EPSILON {
                let scale = checked_scalar(
                    -(REST_VOLUME * pressure_sum),
                    "fluid pressure acceleration scale",
                )?;
                value = value
                    .add(neighbor.gradient.scale(scale))
                    .checked("fluid pressure acceleration reduction")?;
            }
        }
        let mut boundary_value = Vec3f::ZERO;
        if multiplier[index].abs() > SOLVER_EPSILON {
            for neighbor in &reconstruction.solid[row.solid_start..row.solid_end] {
                let scale = checked_scalar(
                    -(neighbor.volume * multiplier[index]),
                    "boundary pressure acceleration scale",
                )?;
                let term = neighbor.gradient.scale(scale);
                boundary_value = boundary_value
                    .add(term)
                    .checked("boundary pressure acceleration reduction")?;
            }
        }
        value = value
            .add(boundary_value)
            .checked("total pressure acceleration")?;
        total.push(value);
        boundary_acceleration.push(boundary_value);
    }
    Ok(AccelerationBatch {
        total,
        boundary: boundary_acceleration,
    })
}

fn matrix_action(
    reconstruction: &Reconstruction,
    acceleration: &[Vec3f],
) -> Result<Vec<f64>, WaterError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(acceleration.len())
        .map_err(heap_error)?;
    for (index, row) in reconstruction.rows.iter().copied().enumerate() {
        let mut value = 0.0;
        for neighbor in &reconstruction.fluid[row.fluid_start..row.fluid_end] {
            let relative = acceleration[index]
                .sub(acceleration[neighbor.other])
                .checked("matrix relative acceleration")?;
            let term = checked_scalar(
                REST_VOLUME * relative.dot(neighbor.gradient),
                "fluid matrix term",
            )?;
            value = checked_scalar(value + term, "fluid matrix reduction")?;
        }
        for neighbor in &reconstruction.solid[row.solid_start..row.solid_end] {
            let term = checked_scalar(
                neighbor.volume * acceleration[index].dot(neighbor.gradient),
                "boundary matrix term",
            )?;
            value = checked_scalar(value + term, "boundary matrix reduction")?;
        }
        result.push(value);
    }
    Ok(result)
}

fn apply_acceleration(
    velocities: &mut [Vec3f],
    acceleration: &AccelerationBatch,
) -> Result<Vec3f, WaterError> {
    let mut boundary_impulse = Vec3f::ZERO;
    for (index, velocity) in velocities.iter_mut().enumerate() {
        *velocity = velocity
            .add(acceleration.total[index].scale(DT))
            .checked("accepted pressure velocity")?;
        let impulse = acceleration.boundary[index]
            .scale(crate::profile::UNIFORM_MASS * DT)
            .checked("boundary impulse row")?;
        boundary_impulse = boundary_impulse
            .add(impulse)
            .checked("boundary impulse reduction")?;
    }
    Ok(boundary_impulse)
}

fn publish(state: &DecodedState) -> Result<Vec<CanonicalSample>, WaterError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(state.samples.len())
        .map_err(heap_error)?;
    for index in 0..state.samples.len() {
        let position = state.positions[index].checked("pre-publication position")?;
        let velocity = state.velocities[index].checked("pre-publication velocity")?;
        result.push(CanonicalSample {
            id: state.samples[index].id,
            position_um: Vec3i::new(
                quantize_micrometres(position.x)?,
                quantize_micrometres(position.y)?,
                quantize_micrometres(position.z)?,
            ),
            velocity_um_s: Vec3i::new(
                quantize_velocity(velocity.x)?,
                quantize_velocity(velocity.y)?,
                quantize_velocity(velocity.z)?,
            ),
        });
    }
    result.sort_unstable_by_key(|sample| sample.id);
    Ok(result)
}

fn validate_canonical_bounds(samples: &[CanonicalSample]) -> Result<(), WaterError> {
    for sample in samples {
        for value in [
            sample.position_um.x,
            sample.position_um.y,
            sample.position_um.z,
        ] {
            let _ = decode_micrometres(value)?;
        }
        for value in [
            sample.velocity_um_s.x,
            sample.velocity_um_s.y,
            sample.velocity_um_s.z,
        ] {
            let _ = decode_velocity(value)?;
        }
    }
    Ok(())
}

fn validate_heap_plan(
    sample_count: usize,
    fluid_neighbor_count: usize,
    solid_neighbor_count: usize,
) -> Result<(), WaterError> {
    let sample_bytes = sample_count
        .checked_mul(
            size_of::<CanonicalSample>()
                + (2 * size_of::<Vec3f>())
                + size_of::<Row>()
                + (8 * size_of::<f64>()),
        )
        .ok_or_else(|| {
            WaterError::new(DECODED_HEAP_CAPACITY_EXCEEDED, "sample heap plan overflow")
        })?;
    let fluid_bytes = fluid_neighbor_count
        .checked_mul(size_of::<FluidNeighbor>())
        .ok_or_else(|| {
            WaterError::new(DECODED_HEAP_CAPACITY_EXCEEDED, "fluid heap plan overflow")
        })?;
    let solid_bytes = solid_neighbor_count
        .checked_mul(size_of::<SolidNeighbor>())
        .ok_or_else(|| {
            WaterError::new(DECODED_HEAP_CAPACITY_EXCEEDED, "solid heap plan overflow")
        })?;
    let total = sample_bytes
        .checked_add(fluid_bytes)
        .and_then(|value| value.checked_add(solid_bytes))
        .ok_or_else(|| {
            WaterError::new(DECODED_HEAP_CAPACITY_EXCEEDED, "decoded heap plan overflow")
        })?;
    validate_capacity(
        total,
        MAXIMUM_DECODED_HEAP_BYTES,
        DECODED_HEAP_CAPACITY_EXCEEDED,
        "decoded heap bytes",
    )
}

fn heap_error(error: std::collections::TryReserveError) -> WaterError {
    WaterError::new(
        DECODED_HEAP_CAPACITY_EXCEEDED,
        format!("bounded oracle allocation failed: {error}"),
    )
}

fn filled_vec<T: Clone>(count: usize, value: T) -> Result<Vec<T>, WaterError> {
    let mut result = Vec::new();
    result.try_reserve_exact(count).map_err(heap_error)?;
    result.resize(count, value);
    Ok(result)
}

fn converged(iteration: u8, minimum: u8, value: i64, threshold: i64) -> bool {
    iteration >= minimum && value <= threshold
}

fn maximum_multiplier_bits(values: &[f64]) -> Result<u64, WaterError> {
    let mut maximum = 0.0;
    for value in values {
        if !value.is_finite() {
            return Err(WaterError::new(
                NONFINITE_VALUE,
                "nonfinite accepted pressure multiplier",
            ));
        }
        if *value > maximum {
            maximum = *value;
        }
    }
    Ok(maximum.to_bits())
}

#[cfg(test)]
mod tests;
