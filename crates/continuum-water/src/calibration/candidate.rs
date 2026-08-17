#![forbid(unsafe_code)]

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Serialize;

use crate::audit::{
    AuditBoundaryInput, boundary_input_root, fluid_input_root, independent_ghost_hydro_calibration,
    production_fluid_input, scalar_bits,
};
use crate::boundary::BoundarySample;
use crate::error::{
    AUDIT_INVALID, AUDIT_MISMATCH, BOUNDARY_CAPACITY_EXCEEDED, REPORT_CAPACITY_EXCEEDED,
    SCENARIO_INVALID, WaterError,
};
use crate::hash::{self, FrozenRoots};
use crate::model::{AcceptedFrame, Geometry, StepSummary, StorageOrder, Vec3i};
use crate::oracle::command::{
    tool_commit, tool_tree_state, validate_output_path, validate_report_capacity,
};
use crate::profile::{
    LATTICE_SPACING_UM, MAXIMUM_BOUNDARY_SAMPLES, PARTICLE_RADIUS_UM, REST_VOLUME,
};
use crate::{boundary, profile, scenario, solver};

use super::{CandidateCalibrationComputation, HydroCalibrationTrace, first_mismatch};

const CANDIDATE_ID: &str = "ghost-cell-shell-v1";
pub(super) const HYDRO_SOAK_STEPS: u32 = 24;

#[derive(Serialize)]
struct CandidateEnvelope<'a> {
    schema_version: u32,
    status: &'a str,
    command: &'a str,
    details: &'a CandidateReport,
}

#[derive(Serialize)]
struct CandidateReport {
    report_schema: String,
    tool_commit: String,
    tool_tree_state: String,
    toolchain_target: String,
    build_profile: String,
    build_rustflags: String,
    roots: CandidateRoots,
    candidate_id: String,
    candidate_definition: String,
    classification: String,
    corpus_credit: String,
    selection_status: String,
    comparison: String,
    first_mismatch: Option<String>,
    local_disposition: String,
    ceiling_counterfactual_id: String,
    ceiling_counterfactual_disposition: String,
    conclusion: String,
    product_check: String,
    fluid_sample_count: usize,
    candidate_boundary_sample_count: usize,
    fluid_input_root: String,
    candidate_boundary_input_root: String,
    production_trace: HydroCalibrationTrace,
    independent_trace: HydroCalibrationTrace,
    normal_hydro_step: NormalHydroStep,
    original_ceiling_hydro_soak: HydroSoak,
    max100_hydro_soak: HydroSoak,
    max160_diagnostic_soak: HydroSoak,
    freefall_control: FreefallControl,
    timing_classification: String,
    wall_clock_nanoseconds: u64,
}

#[derive(Serialize)]
struct CandidateRoots {
    original_w0b_document_root: String,
    original_float_profile_root: String,
    original_corpus_root: String,
    original_execution_profile_root: String,
    original_hydro_scenario_root: String,
    original_freefall_scenario_root: String,
}

#[derive(Serialize)]
struct NormalHydroStep {
    status: String,
    terminal_code: String,
    terminal_detail: String,
    summary: Option<StepSummary>,
}

#[derive(Serialize)]
pub(super) struct FreefallControl {
    pub(super) status: String,
    compared_frames: u32,
    first_mismatch_frame: Option<u32>,
    baseline_final_frame_root: Option<String>,
    candidate_final_frame_root: Option<String>,
    candidate_terminal_code: Option<String>,
    candidate_terminal_detail: Option<String>,
}

#[derive(Serialize)]
pub(super) struct HydroSoak {
    pub(super) status: String,
    density_ceiling: u8,
    requested_steps: u32,
    completed_steps: u32,
    terminal_code: String,
    terminal_detail: String,
    maximum_density_iterations: u8,
    maximum_density_error_ppb: i64,
    maximum_penetration_um: i64,
    final_frame_root: String,
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
    let candidate_boundary = ghost_cell_shell(hydro.geometry)?;
    let production_boundary = audit_boundary_input(&candidate_boundary)?;
    let mut fluid_input = production_fluid_input(&samples)?;
    fluid_input.sort_unstable_by_key(|sample| sample.id);

    let started = Instant::now();
    let production_trace =
        solver::production_hydro_calibration(&samples, &candidate_boundary, hydro.geometry)?;
    let independent = independent_ghost_hydro_calibration()?;
    let mismatch = candidate_mismatch(&production_boundary, &production_trace, &independent);
    let normal_hydro_step = normal_hydro_step(
        &samples,
        hydro.geometry,
        &candidate_boundary,
        &roots.execution_profile,
        &hydro_root,
    )?;
    let original_ceiling_hydro_soak = run_hydro_soak(
        &samples,
        hydro.geometry,
        &candidate_boundary,
        &roots.execution_profile,
        &hydro_root,
        20,
    )?;
    let max100_hydro_soak = run_hydro_soak(
        &samples,
        hydro.geometry,
        &candidate_boundary,
        &roots.execution_profile,
        &hydro_root,
        100,
    )?;
    let max160_diagnostic_soak = run_hydro_soak(
        &samples,
        hydro.geometry,
        &candidate_boundary,
        &roots.execution_profile,
        &hydro_root,
        160,
    )?;
    let freefall_control = compare_freefall(&freefall, &roots.execution_profile, &freefall_root)?;
    let elapsed = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let comparison = if mismatch.is_none() {
        "EXACT_MATCH"
    } else {
        "MISMATCH"
    };
    let survived = mismatch.is_none()
        && normal_hydro_step.status == "STEP_PASS"
        && original_ceiling_hydro_soak.status == "SOAK_PASS"
        && freefall_control.status == "EXACT_MATCH";
    let local_disposition = if survived {
        "LOCAL_DISCRIMINATOR_SURVIVED"
    } else {
        "CANDIDATE_REJECTED"
    };
    let combined_survived = mismatch.is_none()
        && max100_hydro_soak.status == "SOAK_PASS"
        && freefall_control.status == "EXACT_MATCH";
    let ceiling_counterfactual_disposition = if combined_survived {
        "BOUNDED_SOAK_SURVIVED"
    } else {
        "CANDIDATE_REJECTED"
    };
    let initial_tree_state = tool_tree_state(repository_root);
    let mut report = CandidateReport {
        report_schema: "nextengine.continuum-water.hydro-boundary-candidate.v1".to_owned(),
        tool_commit: tool_commit(repository_root),
        tool_tree_state: initial_tree_state,
        toolchain_target: env!("WATER_BUILD_TARGET").to_owned(),
        build_profile: env!("WATER_BUILD_PROFILE").to_owned(),
        build_rustflags: env!("WATER_BUILD_RUSTFLAGS").replace('\u{1f}', " "),
        roots: CandidateRoots {
            original_w0b_document_root: hash::hex(&roots.document),
            original_float_profile_root: hash::hex(&roots.float_profile),
            original_corpus_root: hash::hex(&roots.corpus),
            original_execution_profile_root: hash::hex(&roots.execution_profile),
            original_hydro_scenario_root: hash::hex(&hydro_root),
            original_freefall_scenario_root: hash::hex(&freefall_root),
        },
        candidate_id: CANDIDATE_ID.to_owned(),
        candidate_definition: "one REST_VOLUME sample at every exterior cell centre in the one-cell shell around the analytical box; lattice origin is min + PARTICLE_RADIUS and no fitted multiplier is used".to_owned(),
        classification: "COUNTERFACTUAL_BOUNDARY_CANDIDATE".to_owned(),
        corpus_credit: "NO_CORPUS_CREDIT".to_owned(),
        selection_status: "NOT_SELECTED".to_owned(),
        comparison: comparison.to_owned(),
        first_mismatch: mismatch.clone(),
        local_disposition: local_disposition.to_owned(),
        ceiling_counterfactual_id: "ghost-cell-shell-v1+max100".to_owned(),
        ceiling_counterfactual_disposition: ceiling_counterfactual_disposition.to_owned(),
        conclusion: if survived {
            "the cell-centred ghost-volume shell survives the bounded hydro step and exact free-fall control; full corpus, external aggregate comparison and successor-root closure are still required before selection".to_owned()
        } else if combined_survived {
            "the boundary-only candidate fails at the original ceiling, while the source-derived max100 counterfactual survives the bounded hydro soak and exact free-fall control; independent dynamic, full-corpus and external evidence are still required".to_owned()
        } else {
            "the cell-centred ghost-volume shell fails a local discriminator and must not be selected".to_owned()
        },
        product_check: "CONTINUUM-WATER-REF-P1=NOT_RUN".to_owned(),
        fluid_sample_count: fluid_input.len(),
        candidate_boundary_sample_count: production_boundary.len(),
        fluid_input_root: fluid_input_root(&fluid_input),
        candidate_boundary_input_root: boundary_input_root(&production_boundary),
        production_trace,
        independent_trace: independent.trace,
        normal_hydro_step,
        original_ceiling_hydro_soak,
        max100_hydro_soak,
        max160_diagnostic_soak,
        freefall_control,
        timing_classification: "DIAGNOSTIC_ONLY".to_owned(),
        wall_clock_nanoseconds: elapsed,
    };
    let ending_tree_state = tool_tree_state(repository_root);
    if ending_tree_state != report.tool_tree_state {
        report.tool_tree_state = format!("{}->{ending_tree_state}", report.tool_tree_state);
    }
    write_report(&output, &report, mismatch.is_none())?;
    if let Some(detail) = mismatch {
        return Err(WaterError::new(
            AUDIT_MISMATCH,
            format!("{detail}; candidate report written to {}", output.display()),
        ));
    }
    let command = serde_json::json!({
        "schema_version": 1,
        "status": "REPORT_ONLY",
        "command": "continuum water evaluate-hydro-candidate",
        "details": {
            "report": output.display().to_string(),
            "candidate": report.candidate_id,
            "comparison": report.comparison,
            "local_disposition": report.local_disposition,
            "selection_status": report.selection_status,
            "density_iterations": report.normal_hydro_step.summary.as_ref().map(|summary| summary.density_iterations),
            "density_error_ppb": report.normal_hydro_step.summary.as_ref().map(|summary| summary.density_error_ppb),
            "original_ceiling_hydro_soak": report.original_ceiling_hydro_soak.status,
            "original_ceiling_completed_steps": report.original_ceiling_hydro_soak.completed_steps,
            "max100_hydro_soak": report.max100_hydro_soak.status,
            "max100_completed_steps": report.max100_hydro_soak.completed_steps,
            "max160_diagnostic_soak": report.max160_diagnostic_soak.status,
            "max160_completed_steps": report.max160_diagnostic_soak.completed_steps,
            "ceiling_counterfactual_disposition": report.ceiling_counterfactual_disposition,
            "freefall_control": report.freefall_control.status,
            "product_check": report.product_check,
        }
    });
    serde_json::to_string(&command).map_err(|error| {
        WaterError::new(
            REPORT_CAPACITY_EXCEEDED,
            format!("cannot serialize candidate command report: {error}"),
        )
    })
}

fn argument_error() -> WaterError {
    WaterError::new(
        SCENARIO_INVALID,
        format!(
            "evaluate-hydro-candidate requires --candidate {CANDIDATE_ID} --output <absolute-path>"
        ),
    )
}

pub(super) fn ghost_cell_shell(geometry: Geometry) -> Result<Vec<BoundarySample>, WaterError> {
    if geometry.aperture.is_some() {
        return Err(WaterError::new(
            AUDIT_INVALID,
            "ghost-cell-shell-v1 does not define internal aperture sampling",
        ));
    }
    let counts = [
        cell_count(geometry.bounds.min.x, geometry.bounds.max.x)?,
        cell_count(geometry.bounds.min.y, geometry.bounds.max.y)?,
        cell_count(geometry.bounds.min.z, geometry.bounds.max.z)?,
    ];
    let expanded = counts
        .iter()
        .try_fold(1_usize, |product, count| {
            usize::try_from(*count + 2)
                .ok()
                .and_then(|value| product.checked_mul(value))
        })
        .ok_or_else(|| WaterError::new(BOUNDARY_CAPACITY_EXCEEDED, "ghost shell size overflow"))?;
    let interior = counts
        .iter()
        .try_fold(1_usize, |product, count| {
            usize::try_from(*count)
                .ok()
                .and_then(|value| product.checked_mul(value))
        })
        .ok_or_else(|| WaterError::new(BOUNDARY_CAPACITY_EXCEEDED, "ghost interior overflow"))?;
    let expected = expanded.checked_sub(interior).ok_or_else(|| {
        WaterError::new(
            BOUNDARY_CAPACITY_EXCEEDED,
            "ghost shell subtraction overflow",
        )
    })?;
    if expected > MAXIMUM_BOUNDARY_SAMPLES {
        return Err(WaterError::new(
            BOUNDARY_CAPACITY_EXCEEDED,
            format!("ghost boundary samples {expected} exceed capacity {MAXIMUM_BOUNDARY_SAMPLES}"),
        ));
    }
    let mut result = Vec::new();
    result.try_reserve_exact(expected).map_err(|error| {
        WaterError::new(
            BOUNDARY_CAPACITY_EXCEEDED,
            format!("ghost boundary allocation failed: {error}"),
        )
    })?;
    for ix in -1_i64..=counts[0] {
        for iy in -1_i64..=counts[1] {
            for iz in -1_i64..=counts[2] {
                if ix == -1
                    || ix == counts[0]
                    || iy == -1
                    || iy == counts[1]
                    || iz == -1
                    || iz == counts[2]
                {
                    let position_um = Vec3i::new(
                        ghost_coordinate(geometry.bounds.min.x, ix)?,
                        ghost_coordinate(geometry.bounds.min.y, iy)?,
                        ghost_coordinate(geometry.bounds.min.z, iz)?,
                    );
                    result.push(BoundarySample {
                        id: u32::try_from(result.len()).map_err(|_| {
                            WaterError::new(BOUNDARY_CAPACITY_EXCEEDED, "ghost id overflow")
                        })?,
                        position_um,
                        volume: REST_VOLUME,
                    });
                }
            }
        }
    }
    if result.len() != expected {
        return Err(WaterError::new(
            AUDIT_INVALID,
            format!(
                "ghost boundary generated {} samples, expected {expected}",
                result.len()
            ),
        ));
    }
    Ok(result)
}

fn cell_count(minimum: i64, maximum: i64) -> Result<i64, WaterError> {
    let span = maximum
        .checked_sub(minimum)
        .ok_or_else(|| WaterError::new(AUDIT_INVALID, "ghost axis span overflow"))?;
    if span <= 0 || span % LATTICE_SPACING_UM != 0 {
        return Err(WaterError::new(
            AUDIT_INVALID,
            format!("ghost axis {minimum}..{maximum} is not a positive lattice span"),
        ));
    }
    Ok(span / LATTICE_SPACING_UM)
}

fn ghost_coordinate(minimum: i64, index: i64) -> Result<i64, WaterError> {
    LATTICE_SPACING_UM
        .checked_mul(index)
        .and_then(|offset| PARTICLE_RADIUS_UM.checked_add(offset))
        .and_then(|offset| minimum.checked_add(offset))
        .ok_or_else(|| WaterError::new(AUDIT_INVALID, "ghost coordinate overflow"))
}

pub(super) fn audit_boundary_input(
    boundary: &[BoundarySample],
) -> Result<Vec<AuditBoundaryInput>, WaterError> {
    let mut result = Vec::new();
    result.try_reserve_exact(boundary.len()).map_err(|error| {
        WaterError::new(
            AUDIT_INVALID,
            format!("candidate boundary input allocation failed: {error}"),
        )
    })?;
    for sample in boundary {
        result.push(AuditBoundaryInput {
            id: sample.id,
            position_um: sample.position_um,
            volume_bits: scalar_bits(sample.volume),
        });
    }
    Ok(result)
}

fn candidate_mismatch(
    production_boundary: &[AuditBoundaryInput],
    production_trace: &HydroCalibrationTrace,
    independent: &CandidateCalibrationComputation,
) -> Option<String> {
    if production_boundary.len() != independent.boundary.len() {
        return Some(format!(
            "candidate boundary count {} != {}",
            production_boundary.len(),
            independent.boundary.len()
        ));
    }
    if let Some(index) = production_boundary
        .iter()
        .zip(&independent.boundary)
        .position(|(left, right)| left != right)
    {
        return Some(format!("candidate boundary differs at index {index}"));
    }
    first_mismatch(production_trace, &independent.trace)
}

fn normal_hydro_step(
    samples: &[crate::model::CanonicalSample],
    geometry: Geometry,
    boundary: &[BoundarySample],
    execution_root: &[u8; 32],
    scenario_root: &[u8; 32],
) -> Result<NormalHydroStep, WaterError> {
    let (initial, _) = solver::initial_frame(
        samples.to_vec(),
        geometry,
        boundary,
        execution_root,
        scenario_root,
    )?;
    match solver::substep(&initial, geometry, boundary, execution_root, scenario_root) {
        Ok(outcome) => Ok(NormalHydroStep {
            status: "STEP_PASS".to_owned(),
            terminal_code: "COMPLETED".to_owned(),
            terminal_detail: format!(
                "density converged at iteration {} with {} ppb",
                outcome.summary.density_iterations, outcome.summary.density_error_ppb
            ),
            summary: Some(outcome.summary),
        }),
        Err(error) => Ok(NormalHydroStep {
            status: "CANDIDATE_REJECTED".to_owned(),
            terminal_code: error.code().to_owned(),
            terminal_detail: error.detail().to_owned(),
            summary: None,
        }),
    }
}

pub(super) fn run_hydro_soak(
    samples: &[crate::model::CanonicalSample],
    geometry: Geometry,
    boundary: &[BoundarySample],
    execution_root: &[u8; 32],
    scenario_root: &[u8; 32],
    density_ceiling: u8,
) -> Result<HydroSoak, WaterError> {
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
    for step in 1..=HYDRO_SOAK_STEPS {
        let outcome = match if density_ceiling == 20 {
            solver::substep(&frame, geometry, boundary, execution_root, scenario_root)
        } else {
            solver::counterfactual_substep(
                &frame,
                geometry,
                boundary,
                execution_root,
                scenario_root,
                density_ceiling,
            )
        } {
            Ok(outcome) => outcome,
            Err(error) => {
                return Ok(HydroSoak {
                    status: "CANDIDATE_REJECTED".to_owned(),
                    density_ceiling,
                    requested_steps: HYDRO_SOAK_STEPS,
                    completed_steps: step - 1,
                    terminal_code: error.code().to_owned(),
                    terminal_detail: error.detail().to_owned(),
                    maximum_density_iterations,
                    maximum_density_error_ppb,
                    maximum_penetration_um,
                    final_frame_root: hash::hex(&frame.frame_root),
                });
            }
        };
        maximum_density_iterations =
            maximum_density_iterations.max(outcome.summary.density_iterations);
        maximum_density_error_ppb =
            maximum_density_error_ppb.max(outcome.summary.density_error_ppb);
        maximum_penetration_um = maximum_penetration_um.max(outcome.summary.maximum_penetration_um);
        frame = outcome.frame;
    }
    Ok(HydroSoak {
        status: "SOAK_PASS".to_owned(),
        density_ceiling,
        requested_steps: HYDRO_SOAK_STEPS,
        completed_steps: HYDRO_SOAK_STEPS,
        terminal_code: "COMPLETED".to_owned(),
        terminal_detail: format!("completed {HYDRO_SOAK_STEPS} candidate hydro steps"),
        maximum_density_iterations,
        maximum_density_error_ppb,
        maximum_penetration_um,
        final_frame_root: hash::hex(&frame.frame_root),
    })
}

pub(super) fn compare_freefall(
    scenario: &crate::model::Scenario,
    execution_root: &[u8; 32],
    scenario_root: &[u8; 32],
) -> Result<FreefallControl, WaterError> {
    let samples = scenario::initial_samples(scenario, StorageOrder::Reverse)?;
    let baseline_boundary = boundary::build(scenario.geometry)?;
    let candidate_boundary = ghost_cell_shell(scenario.geometry)?;
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
    if baseline.samples != candidate.samples || baseline.frame_root != candidate.frame_root {
        return Ok(freefall_mismatch(0, &baseline, &candidate));
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
        candidate = match solver::substep(
            &candidate,
            scenario.geometry,
            &candidate_boundary,
            execution_root,
            scenario_root,
        ) {
            Ok(outcome) => outcome.frame,
            Err(error) => {
                return Ok(FreefallControl {
                    status: "CANDIDATE_ERROR".to_owned(),
                    compared_frames: step,
                    first_mismatch_frame: Some(step),
                    baseline_final_frame_root: Some(hash::hex(&baseline.frame_root)),
                    candidate_final_frame_root: Some(hash::hex(&candidate.frame_root)),
                    candidate_terminal_code: Some(error.code().to_owned()),
                    candidate_terminal_detail: Some(error.detail().to_owned()),
                });
            }
        };
        if baseline.samples != candidate.samples || baseline.frame_root != candidate.frame_root {
            return Ok(freefall_mismatch(step, &baseline, &candidate));
        }
    }
    Ok(FreefallControl {
        status: "EXACT_MATCH".to_owned(),
        compared_frames: scenario.steps + 1,
        first_mismatch_frame: None,
        baseline_final_frame_root: Some(hash::hex(&baseline.frame_root)),
        candidate_final_frame_root: Some(hash::hex(&candidate.frame_root)),
        candidate_terminal_code: None,
        candidate_terminal_detail: None,
    })
}

fn freefall_mismatch(
    step: u32,
    baseline: &AcceptedFrame,
    candidate: &AcceptedFrame,
) -> FreefallControl {
    FreefallControl {
        status: "MISMATCH".to_owned(),
        compared_frames: step + 1,
        first_mismatch_frame: Some(step),
        baseline_final_frame_root: Some(hash::hex(&baseline.frame_root)),
        candidate_final_frame_root: Some(hash::hex(&candidate.frame_root)),
        candidate_terminal_code: None,
        candidate_terminal_detail: None,
    }
}

fn write_report(output: &Path, report: &CandidateReport, matched: bool) -> Result<(), WaterError> {
    let envelope = CandidateEnvelope {
        schema_version: 1,
        status: if matched { "REPORT_ONLY" } else { "FAIL" },
        command: "continuum water evaluate-hydro-candidate",
        details: report,
    };
    let mut bytes = serde_json::to_vec_pretty(&envelope).map_err(|error| {
        WaterError::new(
            AUDIT_INVALID,
            format!("cannot serialize hydro candidate report: {error}"),
        )
    })?;
    bytes.push(b'\n');
    validate_report_capacity(bytes.len())?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|error| {
            WaterError::new(
                SCENARIO_INVALID,
                format!(
                    "cannot create candidate report {}: {error}",
                    output.display()
                ),
            )
        })?;
    file.write_all(&bytes).map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!(
                "cannot write candidate report {}: {error}",
                output.display()
            ),
        )
    })?;
    file.sync_all().map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!("cannot sync candidate report {}: {error}", output.display()),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ghost_shell_is_a_bounded_cell_centred_outer_layer() {
        let hydro = scenario::find("CW-HYDRO-001").unwrap();
        let boundary = ghost_cell_shell(hydro.geometry).unwrap();
        assert_eq!(boundary.len(), 2_648);
        assert_eq!(
            boundary.first().unwrap().position_um,
            Vec3i::new(-25_000, -25_000, -25_000)
        );
        assert_eq!(
            boundary.last().unwrap().position_um,
            Vec3i::new(1_025_000, 1_025_000, 1_025_000)
        );
        assert!(
            boundary
                .iter()
                .all(|sample| sample.volume.to_bits() == REST_VOLUME.to_bits())
        );
    }

    #[test]
    fn ghost_candidate_matches_independent_but_fails_the_dynamic_soak() {
        let hydro = scenario::find("CW-HYDRO-001").unwrap();
        let samples = scenario::initial_samples(&hydro, StorageOrder::Reverse).unwrap();
        let boundary = ghost_cell_shell(hydro.geometry).unwrap();
        let boundary_input = audit_boundary_input(&boundary).unwrap();
        let production =
            solver::production_hydro_calibration(&samples, &boundary, hydro.geometry).unwrap();
        let independent = independent_ghost_hydro_calibration().unwrap();

        assert_eq!(
            candidate_mismatch(&boundary_input, &production, &independent),
            None
        );
        assert_eq!(
            boundary_input_root(&boundary_input),
            "64726ee4cebe64a393eb2476f932bb63c9c6aa601dc970d4211d3be0bcb21168"
        );
        assert_eq!(
            production
                .contributions
                .iter()
                .map(|row| row.partition_error_ppb)
                .collect::<Vec<_>>(),
            [-27_534; 4]
        );
        assert_eq!(production.first_original_threshold_iteration, Some(2));
        let checkpoint = production
            .first_original_threshold_checkpoint
            .as_ref()
            .unwrap();
        assert_eq!(checkpoint.density_error_ppb, 95_755);
        assert_eq!(checkpoint.prospective_maximum_penetration_um, 171);

        let original =
            run_hydro_soak(&samples, hydro.geometry, &boundary, &[0; 32], &[1; 32], 20).unwrap();
        let max100 =
            run_hydro_soak(&samples, hydro.geometry, &boundary, &[0; 32], &[1; 32], 100).unwrap();
        let max160 =
            run_hydro_soak(&samples, hydro.geometry, &boundary, &[0; 32], &[1; 32], 160).unwrap();
        assert_eq!(original.completed_steps, 1);
        assert_eq!(original.terminal_code, "WATER_DENSITY_NONCONVERGENCE");
        assert_eq!(max100.completed_steps, 3);
        assert_eq!(max100.maximum_density_iterations, 63);
        assert_eq!(max160.completed_steps, 4);
        assert_eq!(max160.maximum_density_iterations, 108);
    }

    #[test]
    fn candidate_cli_rejects_an_unregistered_candidate() {
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let error = crate::run_xtask(
            repository_root,
            [
                "water".to_owned(),
                "evaluate-hydro-candidate".to_owned(),
                "--candidate".to_owned(),
                "unknown".to_owned(),
                "--output".to_owned(),
                "/tmp/not-created.json".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap_err();
        assert!(error.starts_with("WATER_SCENARIO_INVALID:"));
    }
}
