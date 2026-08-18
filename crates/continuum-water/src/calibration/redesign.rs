#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Serialize;

use crate::audit::{
    boundary_input_root, independent_contact_projection_probe,
    independent_support_complete_hydro_calibration, independent_support_complete_pressure_probe,
    independent_support_complete_projected_pcg_first_step_probe,
};
use crate::error::{AUDIT_INVALID, REPORT_CAPACITY_EXCEEDED, SCENARIO_INVALID, WaterError};
use crate::hash::{self, FrozenRoots};
use crate::model::{
    AcceptedFrame, CanonicalSample, Geometry, StepSummary, StorageOrder, Vec3f, Vec3i,
};
use crate::oracle::command::{tool_commit, tool_tree_state, validate_output_path};
use crate::profile::{
    DT, GRAVITY_MAGNITUDE, PARTICLE_RADIUS_UM, UNIFORM_MASS, decode_micrometres, decode_velocity,
    quantize_ppb, quantize_velocity,
};
use crate::{boundary, profile, scenario, solver};

use super::candidate::{audit_boundary_input, support_complete_lattice_complement};
use super::{
    ContactProjectionProbe, HydroCalibrationTrace, PressureOperatorProbe,
    ProjectedPcgFirstStepProbe,
};

mod report;
#[cfg(test)]
mod tests;

use report::write_report;

pub(crate) const CANDIDATE_ID: &str = "constraint-separated-support-pcg-v1";
const PCG_MAXIMUM_ITERATIONS: u8 = 50;
const LOCAL_HYDRO_STEPS: u32 = 24;
const EXTENDED_HYDRO_STEPS: u32 = 1_200;
const JACOBI_DIAGNOSTIC_ITERATIONS: u8 = 160;

#[derive(Serialize)]
struct RedesignEnvelope<'a> {
    schema_version: u32,
    status: &'a str,
    command: &'a str,
    details: &'a RedesignReport,
}

#[derive(Serialize)]
struct RedesignReport {
    report_schema: String,
    tool_commit: String,
    tool_tree_state: String,
    toolchain_target: String,
    build_profile: String,
    build_rustflags: String,
    roots: RedesignRoots,
    candidate_id: String,
    candidate_definition: String,
    classification: String,
    corpus_credit: String,
    selection_status: String,
    local_disposition: String,
    conclusion: String,
    product_check: String,
    initialization_contract: String,
    stabilization_contract: String,
    fluid_sample_count: usize,
    boundary_sample_count: usize,
    boundary_input_root: String,
    initial_boundary_comparison: String,
    initial_production_trace: HydroCalibrationTrace,
    initial_independent_trace: HydroCalibrationTrace,
    pressure_operator_comparison: String,
    production_pressure_operator: PressureOperatorProbe,
    independent_pressure_operator: PressureOperatorProbe,
    projected_pcg_first_step_comparison: String,
    production_projected_pcg_first_step: ProjectedPcgFirstStepProbe,
    independent_projected_pcg_first_step: ProjectedPcgFirstStepProbe,
    contact_operator_comparison: String,
    production_contact_operator: ContactProjectionProbe,
    independent_contact_operator: ContactProjectionProbe,
    contact_jacobi20_control: ControlSoak,
    contact_jacobi160_control: ControlSoak,
    pcg50_local_soak: ProfileSoak,
    pcg50_extended_soak: ProfileSoak,
    freefall_control: FreefallControl,
    timing_classification: String,
    wall_clock_nanoseconds: u64,
}

#[derive(Serialize)]
struct RedesignRoots {
    rejected_w0b_document_root: String,
    rejected_float_profile_root: String,
    rejected_corpus_root: String,
    rejected_execution_profile_root: String,
    rejected_hydro_scenario_root: String,
    rejected_freefall_scenario_root: String,
    successor_roots: String,
}

#[derive(Serialize)]
struct ControlSoak {
    role: String,
    density_method: String,
    density_ceiling: u8,
    requested_steps: u32,
    completed_steps: u32,
    status: String,
    terminal_code: String,
    terminal_detail: String,
    maximum_density_iterations: u8,
    maximum_density_error_ppb: i64,
    maximum_penetration_um: i64,
    maximum_contact_rows: usize,
    maximum_contact_components: usize,
    final_frame_root: String,
}

#[derive(Serialize)]
struct ProfileSoak {
    density_method: String,
    density_ceiling: u8,
    requested_steps: u32,
    completed_steps: u32,
    status: String,
    terminal_code: String,
    terminal_detail: String,
    maximum_density_iterations: u8,
    maximum_density_error_ppb: i64,
    maximum_divergence_iterations: u8,
    maximum_divergence_error_ppb: i64,
    maximum_penetration_um: i64,
    minimum_outer_clearance_um: i64,
    maximum_contact_rows: usize,
    maximum_contact_components: usize,
    maximum_contact_delta_velocity_um_s: i64,
    pressure_boundary_fluid_impulse_bits: [String; 3],
    contact_fluid_impulse_bits: [String; 3],
    maximum_momentum_residual_ppb: i64,
    maximum_energy_residual_ppb: i64,
    initial_centre_of_mass_um: Vec3i,
    final_centre_of_mass_um: Vec3i,
    maximum_centre_of_mass_drift_um: Vec3i,
    final_frame_root: String,
    checkpoints: Vec<SoakCheckpoint>,
}

#[derive(Serialize)]
struct SoakCheckpoint {
    step: u32,
    density_iterations: u8,
    density_error_ppb: i64,
    divergence_iterations: u8,
    divergence_error_ppb: i64,
    minimum_outer_clearance_um: i64,
    contact_rows: usize,
    contact_components: usize,
    contact_delta_velocity_um_s: i64,
    centre_of_mass_um: Vec3i,
    momentum_residual_ppb: i64,
    energy_residual_ppb: i64,
    frame_root: String,
}

#[derive(Serialize)]
struct FreefallControl {
    status: String,
    compared_frames: u32,
    first_mismatch_frame: Option<u32>,
    candidate_terminal_code: Option<String>,
    candidate_terminal_detail: Option<String>,
    active_contact_components: usize,
    maximum_density_iterations: u8,
    maximum_density_error_ppb: i64,
    baseline_final_frame_root: String,
    candidate_final_frame_root: String,
}

#[derive(Clone, Copy)]
struct PhysicalTotals {
    momentum: Vec3f,
    mechanical_energy: f64,
}

pub(crate) fn run_xtask(
    repository_root: &Path,
    mut arguments: impl Iterator<Item = String>,
) -> Result<String, WaterError> {
    let candidate_flag = arguments.next().ok_or_else(argument_error)?;
    let candidate = arguments.next().ok_or_else(argument_error)?;
    let output_flag = arguments.next().ok_or_else(argument_error)?;
    let output = arguments.next().ok_or_else(argument_error)?;
    if candidate_flag != "--candidate"
        || candidate != CANDIDATE_ID
        || output_flag != "--output"
        || arguments.next().is_some()
    {
        return Err(argument_error());
    }
    let output = PathBuf::from(output);
    validate_output_path(repository_root, &output)?;
    profile::validate_execution_profile()?;
    profile::validate_float_environment()?;
    let roots = FrozenRoots::verify(repository_root)?;
    let hydro = scenario::find("CW-HYDRO-001")?;
    let hydro_root = scenario::root_for(&hydro, &roots)?;
    let freefall = scenario::find("CW-FREEFALL-001")?;
    let freefall_root = scenario::root_for(&freefall, &roots)?;
    let samples = scenario::initial_samples(&hydro, StorageOrder::Reverse)?;
    let candidate_boundary = support_complete_lattice_complement(hydro.geometry)?;
    let boundary_input = audit_boundary_input(&candidate_boundary)?;

    let started = Instant::now();
    let initial_production_trace =
        solver::production_hydro_calibration(&samples, &candidate_boundary, hydro.geometry)?;
    let independent_initial = independent_support_complete_hydro_calibration()?;
    let initial_boundary_match = boundary_input == independent_initial.boundary
        && initial_production_trace == independent_initial.trace;
    let production_pressure_operator =
        solver::production_pressure_operator_probe(&samples, &candidate_boundary)?;
    let independent_pressure_operator = independent_support_complete_pressure_probe()?;
    let pressure_operator_match = production_pressure_operator == independent_pressure_operator;
    let production_projected_pcg_first_step =
        solver::production_projected_pcg_first_step_probe(&samples, &candidate_boundary)?;
    let independent_projected_pcg_first_step =
        independent_support_complete_projected_pcg_first_step_probe()?;
    let projected_pcg_first_step_match =
        production_projected_pcg_first_step == independent_projected_pcg_first_step;
    let production_contact_operator = solver::production_contact_projection_probe()?;
    let independent_contact_operator = independent_contact_projection_probe()?;
    let contact_operator_match = production_contact_operator == independent_contact_operator;
    let contact_jacobi20_control = run_control_soak(
        &samples,
        hydro.geometry,
        &candidate_boundary,
        &roots.execution_profile,
        &hydro_root,
        20,
        "NEGATIVE_CONTROL",
    )?;
    let contact_jacobi160_control = run_control_soak(
        &samples,
        hydro.geometry,
        &candidate_boundary,
        &roots.execution_profile,
        &hydro_root,
        JACOBI_DIAGNOSTIC_ITERATIONS,
        "CAUSAL_CONTROL_NO_SELECTION_CREDIT",
    )?;
    let pcg50_local_soak = run_profile_soak(
        &samples,
        hydro.geometry,
        &candidate_boundary,
        &roots.execution_profile,
        &hydro_root,
        LOCAL_HYDRO_STEPS,
    )?;
    let pcg50_extended_soak = run_profile_soak(
        &samples,
        hydro.geometry,
        &candidate_boundary,
        &roots.execution_profile,
        &hydro_root,
        EXTENDED_HYDRO_STEPS,
    )?;
    let freefall_control = compare_freefall(&freefall, &roots.execution_profile, &freefall_root)?;
    let elapsed = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);

    let survived = initial_boundary_match
        && pressure_operator_match
        && projected_pcg_first_step_match
        && contact_operator_match
        && contact_jacobi20_control.status == "EXPECTED_REJECTION"
        && contact_jacobi160_control.status == "CONTROL_PASS"
        && profile_soak_passes(&pcg50_local_soak)
        && profile_soak_passes(&pcg50_extended_soak)
        && freefall_control.status == "EXACT_MATCH";
    let local_disposition = if survived {
        "LOCAL_PROFILE_DISCRIMINATOR_SURVIVED"
    } else {
        "CANDIDATE_REJECTED"
    };
    let initial_tree_state = tool_tree_state(repository_root);
    let mut report = RedesignReport {
        report_schema: "nextengine.continuum-water.constraint-separated-redesign.v1".to_owned(),
        tool_commit: tool_commit(repository_root),
        tool_tree_state: initial_tree_state,
        toolchain_target: env!("WATER_BUILD_TARGET").to_owned(),
        build_profile: env!("WATER_BUILD_PROFILE").to_owned(),
        build_rustflags: env!("WATER_BUILD_RUSTFLAGS").replace('\u{1f}', " "),
        roots: RedesignRoots {
            rejected_w0b_document_root: hash::hex(&roots.document),
            rejected_float_profile_root: hash::hex(&roots.float_profile),
            rejected_corpus_root: hash::hex(&roots.corpus),
            rejected_execution_profile_root: hash::hex(&roots.execution_profile),
            rejected_hydro_scenario_root: hash::hex(&hydro_root),
            rejected_freefall_scenario_root: hash::hex(&freefall_root),
            successor_roots: "NOT_ISSUED".to_owned(),
        },
        candidate_id: CANDIDATE_ID.to_owned(),
        candidate_definition: "support-complete discrete density complement; projected diagonally preconditioned conjugate-gradient density solve with a 50-iteration ceiling and unchanged 100000 ppb threshold; analytical particle-radius outer-box velocity projection after pressure and before integration; cold regular-lattice initialization; no warm state, XSPH, viscosity, density map, retry or position repair".to_owned(),
        classification: "COUNTERFACTUAL_JOINT_PROFILE_REDESIGN".to_owned(),
        corpus_credit: "NO_CORPUS_CREDIT".to_owned(),
        selection_status: "NOT_SELECTED".to_owned(),
        local_disposition: local_disposition.to_owned(),
        conclusion: if survived {
            "the constraint-separated candidate survives independent operator checks, both causal controls, the 24-step and 1200-step hydro gates, reaction/work accounting and exact freefall; successor roots and the full W1 corpus are still required before selection".to_owned()
        } else {
            "the constraint-separated candidate fails at least one W0E gate and must not be selected".to_owned()
        },
        product_check: "CONTINUUM-WATER-REF-P1=NOT_RUN".to_owned(),
        initialization_contract: "canonical regular lattice and zero velocity; no retained pressure or settling output".to_owned(),
        stabilization_contract: "NONE; no XSPH, viscosity, surface tension or hidden damping".to_owned(),
        fluid_sample_count: samples.len(),
        boundary_sample_count: candidate_boundary.len(),
        boundary_input_root: boundary_input_root(&boundary_input),
        initial_boundary_comparison: comparison(initial_boundary_match),
        initial_production_trace,
        initial_independent_trace: independent_initial.trace,
        pressure_operator_comparison: comparison(pressure_operator_match),
        production_pressure_operator,
        independent_pressure_operator,
        projected_pcg_first_step_comparison: comparison(projected_pcg_first_step_match),
        production_projected_pcg_first_step,
        independent_projected_pcg_first_step,
        contact_operator_comparison: comparison(contact_operator_match),
        production_contact_operator,
        independent_contact_operator,
        contact_jacobi20_control,
        contact_jacobi160_control,
        pcg50_local_soak,
        pcg50_extended_soak,
        freefall_control,
        timing_classification: "DIAGNOSTIC_ONLY".to_owned(),
        wall_clock_nanoseconds: elapsed,
    };
    let ending_tree_state = tool_tree_state(repository_root);
    if ending_tree_state != report.tool_tree_state {
        report.tool_tree_state = format!("{}->{ending_tree_state}", report.tool_tree_state);
    }
    write_report(&output, &report, survived)?;
    let command = serde_json::json!({
        "schema_version": 1,
        "status": "REPORT_ONLY",
        "command": "continuum water evaluate-hydro-redesign",
        "details": {
            "report": output.display().to_string(),
            "candidate": CANDIDATE_ID,
            "local_disposition": local_disposition,
            "selection_status": "NOT_SELECTED",
            "initial_boundary_comparison": comparison(initial_boundary_match),
            "pressure_operator_comparison": comparison(pressure_operator_match),
            "projected_pcg_first_step_comparison": comparison(projected_pcg_first_step_match),
            "contact_operator_comparison": comparison(contact_operator_match),
            "jacobi20_control": report.contact_jacobi20_control.status,
            "jacobi160_control": report.contact_jacobi160_control.status,
            "local_soak": report.pcg50_local_soak.status,
            "extended_soak": report.pcg50_extended_soak.status,
            "freefall_control": report.freefall_control.status,
            "product_check": report.product_check,
        }
    });
    serde_json::to_string(&command).map_err(|error| {
        WaterError::new(
            REPORT_CAPACITY_EXCEEDED,
            format!("cannot serialize redesign command report: {error}"),
        )
    })
}

fn argument_error() -> WaterError {
    WaterError::new(
        SCENARIO_INVALID,
        format!(
            "evaluate-hydro-redesign requires --candidate {CANDIDATE_ID} --output <absolute-path>"
        ),
    )
}

fn comparison(matches: bool) -> String {
    if matches { "EXACT_MATCH" } else { "MISMATCH" }.to_owned()
}

fn profile_soak_passes(soak: &ProfileSoak) -> bool {
    soak.status == "SOAK_PASS"
        && soak.maximum_momentum_residual_ppb <= 10_000_000
        && soak.maximum_energy_residual_ppb <= 10_000_000
        && soak.minimum_outer_clearance_um >= PARTICLE_RADIUS_UM
}

#[allow(clippy::too_many_arguments)]
fn run_control_soak(
    samples: &[CanonicalSample],
    geometry: Geometry,
    boundary: &[crate::boundary::BoundarySample],
    execution_root: &[u8; 32],
    scenario_root: &[u8; 32],
    density_ceiling: u8,
    role: &str,
) -> Result<ControlSoak, WaterError> {
    let (mut frame, _) = solver::initial_frame(
        samples.to_vec(),
        geometry,
        boundary,
        execution_root,
        scenario_root,
    )?;
    let mut maximum_density_iterations = 0_u8;
    let mut maximum_density_error_ppb = 0_i64;
    let mut maximum_penetration_um = 0_i64;
    let mut maximum_contact_rows = 0_usize;
    let mut maximum_contact_components = 0_usize;
    for step in 1..=LOCAL_HYDRO_STEPS {
        let constrained = match solver::contact_constrained_substep_with_limit(
            &frame,
            geometry,
            boundary,
            execution_root,
            scenario_root,
            density_ceiling,
        ) {
            Ok(value) => value,
            Err(error) => {
                let expected = density_ceiling == 20
                    && step == 2
                    && error.code() == crate::error::DENSITY_NONCONVERGENCE;
                return Ok(ControlSoak {
                    role: role.to_owned(),
                    density_method: "RELAXED_JACOBI".to_owned(),
                    density_ceiling,
                    requested_steps: LOCAL_HYDRO_STEPS,
                    completed_steps: step - 1,
                    status: if expected {
                        "EXPECTED_REJECTION".to_owned()
                    } else {
                        "CONTROL_FAILED".to_owned()
                    },
                    terminal_code: error.code().to_owned(),
                    terminal_detail: error.detail().to_owned(),
                    maximum_density_iterations,
                    maximum_density_error_ppb,
                    maximum_penetration_um,
                    maximum_contact_rows,
                    maximum_contact_components,
                    final_frame_root: hash::hex(&frame.frame_root),
                });
            }
        };
        maximum_density_iterations =
            maximum_density_iterations.max(constrained.outcome.summary.density_iterations);
        maximum_density_error_ppb =
            maximum_density_error_ppb.max(constrained.outcome.summary.density_error_ppb);
        maximum_penetration_um =
            maximum_penetration_um.max(constrained.outcome.summary.maximum_penetration_um);
        maximum_contact_rows = maximum_contact_rows.max(constrained.projection.active_rows);
        maximum_contact_components =
            maximum_contact_components.max(constrained.projection.active_components);
        frame = constrained.outcome.frame;
    }
    Ok(ControlSoak {
        role: role.to_owned(),
        density_method: "RELAXED_JACOBI".to_owned(),
        density_ceiling,
        requested_steps: LOCAL_HYDRO_STEPS,
        completed_steps: LOCAL_HYDRO_STEPS,
        status: if density_ceiling == JACOBI_DIAGNOSTIC_ITERATIONS {
            "CONTROL_PASS".to_owned()
        } else {
            "UNEXPECTED_PASS".to_owned()
        },
        terminal_code: "COMPLETED".to_owned(),
        terminal_detail: format!("completed {LOCAL_HYDRO_STEPS} contact/Jacobi control steps"),
        maximum_density_iterations,
        maximum_density_error_ppb,
        maximum_penetration_um,
        maximum_contact_rows,
        maximum_contact_components,
        final_frame_root: hash::hex(&frame.frame_root),
    })
}

#[allow(clippy::too_many_arguments)]
fn run_profile_soak(
    samples: &[CanonicalSample],
    geometry: Geometry,
    boundary: &[crate::boundary::BoundarySample],
    execution_root: &[u8; 32],
    scenario_root: &[u8; 32],
    requested_steps: u32,
) -> Result<ProfileSoak, WaterError> {
    let (mut frame, initial_summary) = solver::initial_frame(
        samples.to_vec(),
        geometry,
        boundary,
        execution_root,
        scenario_root,
    )?;
    let baseline = physical_totals(&frame)?;
    let initial_centre = initial_summary.centre_of_mass_um;
    let mut final_centre = initial_centre;
    let mut maximum_drift = Vec3i::new(0, 0, 0);
    let mut gravity_impulse = Vec3f::ZERO;
    let mut pressure_impulse = Vec3f::ZERO;
    let mut contact_impulse = Vec3f::ZERO;
    let mut maximum_density_iterations = 0_u8;
    let mut maximum_density_error_ppb = 0_i64;
    let mut maximum_divergence_iterations = 0_u8;
    let mut maximum_divergence_error_ppb = 0_i64;
    let mut maximum_penetration_um = 0_i64;
    let mut minimum_clearance = minimum_outer_clearance(geometry, &frame.samples)?;
    let mut maximum_contact_rows = 0_usize;
    let mut maximum_contact_components = 0_usize;
    let mut maximum_contact_delta_velocity = 0.0_f64;
    let mut maximum_momentum_residual_ppb = 0_i64;
    let mut maximum_energy_residual_ppb = 0_i64;
    let mut checkpoints = Vec::new();
    checkpoints
        .try_reserve_exact(usize::try_from(requested_steps / LOCAL_HYDRO_STEPS).unwrap_or(0))
        .map_err(reserve_error)?;
    for step in 1..=requested_steps {
        let constrained = match solver::contact_pcg_constrained_substep(
            &frame,
            geometry,
            boundary,
            execution_root,
            scenario_root,
            PCG_MAXIMUM_ITERATIONS,
        ) {
            Ok(value) => value,
            Err(error) => {
                return failed_profile_soak(
                    requested_steps,
                    step - 1,
                    error,
                    &frame,
                    maximum_density_iterations,
                    maximum_density_error_ppb,
                    maximum_divergence_iterations,
                    maximum_divergence_error_ppb,
                    maximum_penetration_um,
                    minimum_clearance,
                    maximum_contact_rows,
                    maximum_contact_components,
                    maximum_contact_delta_velocity,
                    pressure_impulse,
                    contact_impulse,
                    maximum_momentum_residual_ppb,
                    maximum_energy_residual_ppb,
                    initial_centre,
                    final_centre,
                    maximum_drift,
                    checkpoints,
                );
            }
        };
        for _sample in &constrained.outcome.frame.samples {
            let gravity_row = Vec3f::new(
                0.0,
                checked(
                    UNIFORM_MASS * DT * -GRAVITY_MAGNITUDE,
                    "redesign gravity impulse",
                )?,
                0.0,
            );
            gravity_impulse = gravity_impulse
                .add(gravity_row)
                .checked("redesign gravity impulse reduction")?;
        }
        for impulse in constrained.outcome.boundary_impulses {
            pressure_impulse = pressure_impulse
                .add(impulse)
                .checked("redesign pressure impulse reduction")?;
        }
        contact_impulse = contact_impulse
            .add(constrained.projection.fluid_impulse)
            .checked("redesign contact impulse reduction")?;
        update_step_maxima(
            &constrained.outcome.summary,
            constrained.projection.active_rows,
            constrained.projection.active_components,
            constrained.projection.maximum_absolute_delta_velocity,
            &mut maximum_density_iterations,
            &mut maximum_density_error_ppb,
            &mut maximum_divergence_iterations,
            &mut maximum_divergence_error_ppb,
            &mut maximum_penetration_um,
            &mut maximum_contact_rows,
            &mut maximum_contact_components,
            &mut maximum_contact_delta_velocity,
        );
        let summary = constrained.outcome.summary;
        frame = constrained.outcome.frame;
        let step_clearance = minimum_outer_clearance(geometry, &frame.samples)?;
        minimum_clearance = minimum_clearance.min(step_clearance);
        final_centre = summary.centre_of_mass_um;
        maximum_drift = maximum_drift.max_abs_difference(initial_centre, final_centre)?;
        let (momentum_residual_ppb, energy_residual_ppb) = physical_residuals(
            &frame,
            baseline,
            gravity_impulse,
            pressure_impulse,
            contact_impulse,
        )?;
        maximum_momentum_residual_ppb = maximum_momentum_residual_ppb.max(momentum_residual_ppb);
        maximum_energy_residual_ppb = maximum_energy_residual_ppb.max(energy_residual_ppb);
        if step % LOCAL_HYDRO_STEPS == 0 {
            checkpoints.push(SoakCheckpoint {
                step,
                density_iterations: summary.density_iterations,
                density_error_ppb: summary.density_error_ppb,
                divergence_iterations: summary.divergence_iterations,
                divergence_error_ppb: summary.divergence_error_ppb,
                minimum_outer_clearance_um: step_clearance,
                contact_rows: constrained.projection.active_rows,
                contact_components: constrained.projection.active_components,
                contact_delta_velocity_um_s: quantize_velocity(
                    constrained.projection.maximum_absolute_delta_velocity,
                )?,
                centre_of_mass_um: summary.centre_of_mass_um,
                momentum_residual_ppb,
                energy_residual_ppb,
                frame_root: summary.frame_root,
            });
        }
    }
    Ok(ProfileSoak {
        density_method: "PROJECTED_DIAGONAL_PCG_ACTIVE_SET_RESTART".to_owned(),
        density_ceiling: PCG_MAXIMUM_ITERATIONS,
        requested_steps,
        completed_steps: requested_steps,
        status: "SOAK_PASS".to_owned(),
        terminal_code: "COMPLETED".to_owned(),
        terminal_detail: format!("completed {requested_steps} constraint-separated profile steps"),
        maximum_density_iterations,
        maximum_density_error_ppb,
        maximum_divergence_iterations,
        maximum_divergence_error_ppb,
        maximum_penetration_um,
        minimum_outer_clearance_um: minimum_clearance,
        maximum_contact_rows,
        maximum_contact_components,
        maximum_contact_delta_velocity_um_s: quantize_velocity(maximum_contact_delta_velocity)?,
        pressure_boundary_fluid_impulse_bits: crate::audit::vector_bits(pressure_impulse),
        contact_fluid_impulse_bits: crate::audit::vector_bits(contact_impulse),
        maximum_momentum_residual_ppb,
        maximum_energy_residual_ppb,
        initial_centre_of_mass_um: initial_centre,
        final_centre_of_mass_um: final_centre,
        maximum_centre_of_mass_drift_um: maximum_drift,
        final_frame_root: hash::hex(&frame.frame_root),
        checkpoints,
    })
}

#[allow(clippy::too_many_arguments)]
fn failed_profile_soak(
    requested_steps: u32,
    completed_steps: u32,
    error: WaterError,
    frame: &AcceptedFrame,
    maximum_density_iterations: u8,
    maximum_density_error_ppb: i64,
    maximum_divergence_iterations: u8,
    maximum_divergence_error_ppb: i64,
    maximum_penetration_um: i64,
    minimum_outer_clearance_um: i64,
    maximum_contact_rows: usize,
    maximum_contact_components: usize,
    maximum_contact_delta_velocity: f64,
    pressure_impulse: Vec3f,
    contact_impulse: Vec3f,
    maximum_momentum_residual_ppb: i64,
    maximum_energy_residual_ppb: i64,
    initial_centre: Vec3i,
    final_centre: Vec3i,
    maximum_drift: Vec3i,
    checkpoints: Vec<SoakCheckpoint>,
) -> Result<ProfileSoak, WaterError> {
    Ok(ProfileSoak {
        density_method: "PROJECTED_DIAGONAL_PCG_ACTIVE_SET_RESTART".to_owned(),
        density_ceiling: PCG_MAXIMUM_ITERATIONS,
        requested_steps,
        completed_steps,
        status: "CANDIDATE_REJECTED".to_owned(),
        terminal_code: error.code().to_owned(),
        terminal_detail: error.detail().to_owned(),
        maximum_density_iterations,
        maximum_density_error_ppb,
        maximum_divergence_iterations,
        maximum_divergence_error_ppb,
        maximum_penetration_um,
        minimum_outer_clearance_um,
        maximum_contact_rows,
        maximum_contact_components,
        maximum_contact_delta_velocity_um_s: quantize_velocity(maximum_contact_delta_velocity)?,
        pressure_boundary_fluid_impulse_bits: crate::audit::vector_bits(pressure_impulse),
        contact_fluid_impulse_bits: crate::audit::vector_bits(contact_impulse),
        maximum_momentum_residual_ppb,
        maximum_energy_residual_ppb,
        initial_centre_of_mass_um: initial_centre,
        final_centre_of_mass_um: final_centre,
        maximum_centre_of_mass_drift_um: maximum_drift,
        final_frame_root: hash::hex(&frame.frame_root),
        checkpoints,
    })
}

#[allow(clippy::too_many_arguments)]
fn update_step_maxima(
    summary: &StepSummary,
    contact_rows: usize,
    contact_components: usize,
    contact_delta_velocity: f64,
    maximum_density_iterations: &mut u8,
    maximum_density_error_ppb: &mut i64,
    maximum_divergence_iterations: &mut u8,
    maximum_divergence_error_ppb: &mut i64,
    maximum_penetration_um: &mut i64,
    maximum_contact_rows: &mut usize,
    maximum_contact_components: &mut usize,
    maximum_contact_delta_velocity: &mut f64,
) {
    *maximum_density_iterations = (*maximum_density_iterations).max(summary.density_iterations);
    *maximum_density_error_ppb = (*maximum_density_error_ppb).max(summary.density_error_ppb);
    *maximum_divergence_iterations =
        (*maximum_divergence_iterations).max(summary.divergence_iterations);
    *maximum_divergence_error_ppb =
        (*maximum_divergence_error_ppb).max(summary.divergence_error_ppb);
    *maximum_penetration_um = (*maximum_penetration_um).max(summary.maximum_penetration_um);
    *maximum_contact_rows = (*maximum_contact_rows).max(contact_rows);
    *maximum_contact_components = (*maximum_contact_components).max(contact_components);
    *maximum_contact_delta_velocity = (*maximum_contact_delta_velocity).max(contact_delta_velocity);
}

fn compare_freefall(
    scenario: &crate::model::Scenario,
    execution_root: &[u8; 32],
    scenario_root: &[u8; 32],
) -> Result<FreefallControl, WaterError> {
    let samples = scenario::initial_samples(scenario, StorageOrder::Reverse)?;
    let baseline_boundary = boundary::build(scenario.geometry)?;
    let candidate_boundary = support_complete_lattice_complement(scenario.geometry)?;
    let (mut baseline, _) = solver::initial_frame(
        samples.clone(),
        scenario.geometry,
        &baseline_boundary,
        execution_root,
        scenario_root,
    )?;
    let (mut candidate, _) = solver::initial_frame(
        samples,
        scenario.geometry,
        &candidate_boundary,
        execution_root,
        scenario_root,
    )?;
    let mut active_contact_components = 0_usize;
    let mut maximum_density_iterations = 0_u8;
    let mut maximum_density_error_ppb = 0_i64;
    if baseline.samples != candidate.samples || baseline.frame_root != candidate.frame_root {
        return Ok(freefall_mismatch(
            0,
            &baseline,
            &candidate,
            active_contact_components,
            maximum_density_iterations,
            maximum_density_error_ppb,
        ));
    }
    for step in 1..=scenario.steps {
        baseline = solver::substep(
            &baseline,
            scenario.geometry,
            &baseline_boundary,
            execution_root,
            scenario_root,
        )?
        .frame;
        let constrained = match solver::contact_pcg_constrained_substep(
            &candidate,
            scenario.geometry,
            &candidate_boundary,
            execution_root,
            scenario_root,
            PCG_MAXIMUM_ITERATIONS,
        ) {
            Ok(value) => value,
            Err(error) => {
                return Ok(FreefallControl {
                    status: "CANDIDATE_ERROR".to_owned(),
                    compared_frames: step,
                    first_mismatch_frame: Some(step),
                    candidate_terminal_code: Some(error.code().to_owned()),
                    candidate_terminal_detail: Some(error.detail().to_owned()),
                    active_contact_components,
                    maximum_density_iterations,
                    maximum_density_error_ppb,
                    baseline_final_frame_root: hash::hex(&baseline.frame_root),
                    candidate_final_frame_root: hash::hex(&candidate.frame_root),
                });
            }
        };
        active_contact_components = active_contact_components
            .checked_add(constrained.projection.active_components)
            .ok_or_else(|| WaterError::new(AUDIT_INVALID, "freefall contact count overflow"))?;
        maximum_density_iterations =
            maximum_density_iterations.max(constrained.outcome.summary.density_iterations);
        maximum_density_error_ppb =
            maximum_density_error_ppb.max(constrained.outcome.summary.density_error_ppb);
        candidate = constrained.outcome.frame;
        if baseline.samples != candidate.samples || baseline.frame_root != candidate.frame_root {
            return Ok(freefall_mismatch(
                step,
                &baseline,
                &candidate,
                active_contact_components,
                maximum_density_iterations,
                maximum_density_error_ppb,
            ));
        }
    }
    Ok(FreefallControl {
        status: if active_contact_components == 0
            && maximum_density_iterations == 2
            && maximum_density_error_ppb == 0
        {
            "EXACT_MATCH".to_owned()
        } else {
            "DIAGNOSTIC_MISMATCH".to_owned()
        },
        compared_frames: scenario.steps + 1,
        first_mismatch_frame: None,
        candidate_terminal_code: None,
        candidate_terminal_detail: None,
        active_contact_components,
        maximum_density_iterations,
        maximum_density_error_ppb,
        baseline_final_frame_root: hash::hex(&baseline.frame_root),
        candidate_final_frame_root: hash::hex(&candidate.frame_root),
    })
}

fn freefall_mismatch(
    step: u32,
    baseline: &AcceptedFrame,
    candidate: &AcceptedFrame,
    active_contact_components: usize,
    maximum_density_iterations: u8,
    maximum_density_error_ppb: i64,
) -> FreefallControl {
    FreefallControl {
        status: "MISMATCH".to_owned(),
        compared_frames: step + 1,
        first_mismatch_frame: Some(step),
        candidate_terminal_code: None,
        candidate_terminal_detail: None,
        active_contact_components,
        maximum_density_iterations,
        maximum_density_error_ppb,
        baseline_final_frame_root: hash::hex(&baseline.frame_root),
        candidate_final_frame_root: hash::hex(&candidate.frame_root),
    }
}

fn physical_totals(frame: &AcceptedFrame) -> Result<PhysicalTotals, WaterError> {
    let mut momentum = Vec3f::ZERO;
    let mut mechanical_energy = 0.0;
    for sample in &frame.samples {
        let velocity = Vec3f::new(
            decode_velocity(sample.velocity_um_s.x)?,
            decode_velocity(sample.velocity_um_s.y)?,
            decode_velocity(sample.velocity_um_s.z)?,
        );
        momentum = momentum
            .add(velocity.scale(UNIFORM_MASS))
            .checked("redesign momentum reduction")?;
        let kinetic = checked(
            0.5 * UNIFORM_MASS * velocity.dot(velocity),
            "redesign kinetic energy",
        )?;
        let potential = checked(
            UNIFORM_MASS * GRAVITY_MAGNITUDE * decode_micrometres(sample.position_um.y)?,
            "redesign potential energy",
        )?;
        mechanical_energy = checked(
            mechanical_energy + kinetic + potential,
            "redesign mechanical energy reduction",
        )?;
    }
    Ok(PhysicalTotals {
        momentum,
        mechanical_energy,
    })
}

fn physical_residuals(
    frame: &AcceptedFrame,
    baseline: PhysicalTotals,
    gravity_impulse: Vec3f,
    pressure_impulse: Vec3f,
    contact_impulse: Vec3f,
) -> Result<(i64, i64), WaterError> {
    let current = physical_totals(frame)?;
    let expected_momentum = baseline
        .momentum
        .add(gravity_impulse)
        .add(pressure_impulse)
        .add(contact_impulse)
        .checked("redesign expected momentum")?;
    let momentum_delta = current
        .momentum
        .sub(expected_momentum)
        .checked("redesign momentum residual")?;
    let momentum_denominator = (vector_norm(baseline.momentum)?
        + vector_norm(gravity_impulse)?
        + vector_norm(pressure_impulse)?
        + vector_norm(contact_impulse)?)
    .max(1.0);
    let momentum_relative = checked(
        vector_norm(momentum_delta)? / momentum_denominator,
        "redesign normalized momentum residual",
    )?;
    let energy_denominator = baseline.mechanical_energy.abs().max(1.0);
    let energy_relative = checked(
        (current.mechanical_energy - baseline.mechanical_energy).abs() / energy_denominator,
        "redesign normalized energy residual",
    )?;
    Ok((
        quantize_ppb(momentum_relative)?,
        quantize_ppb(energy_relative)?,
    ))
}

fn vector_norm(value: Vec3f) -> Result<f64, WaterError> {
    checked(value.dot(value).sqrt(), "redesign vector norm")
}

fn minimum_outer_clearance(
    geometry: Geometry,
    samples: &[CanonicalSample],
) -> Result<i64, WaterError> {
    let mut minimum = i64::MAX;
    for sample in samples {
        for clearance in [
            sample.position_um.x - geometry.bounds.min.x,
            geometry.bounds.max.x - sample.position_um.x,
            sample.position_um.y - geometry.bounds.min.y,
            geometry.bounds.max.y - sample.position_um.y,
            sample.position_um.z - geometry.bounds.min.z,
            geometry.bounds.max.z - sample.position_um.z,
        ] {
            minimum = minimum.min(clearance);
        }
    }
    if minimum == i64::MAX {
        Ok(0)
    } else {
        Ok(minimum)
    }
}

trait MaximumDrift {
    fn max_abs_difference(self, initial: Self, current: Self) -> Result<Self, WaterError>
    where
        Self: Sized;
}

impl MaximumDrift for Vec3i {
    fn max_abs_difference(self, initial: Self, current: Self) -> Result<Self, WaterError> {
        Ok(Vec3i::new(
            self.x.max(abs_difference(initial.x, current.x)?),
            self.y.max(abs_difference(initial.y, current.y)?),
            self.z.max(abs_difference(initial.z, current.z)?),
        ))
    }
}

fn abs_difference(left: i64, right: i64) -> Result<i64, WaterError> {
    let difference = i128::from(left) - i128::from(right);
    i64::try_from(difference.abs())
        .map_err(|_| WaterError::new(AUDIT_INVALID, "centre-of-mass drift overflow"))
}

fn checked(value: f64, phase: &str) -> Result<f64, WaterError> {
    crate::model::checked_scalar(value, phase)
}

fn reserve_error(error: std::collections::TryReserveError) -> WaterError {
    WaterError::new(
        AUDIT_INVALID,
        format!("redesign report allocation failed: {error}"),
    )
}
