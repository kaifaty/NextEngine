#![forbid(unsafe_code)]

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Serialize;

use crate::audit::{
    VolumeMapObservation, independent_volume_map_hydro_calibration,
    independent_volume_map_observation, production_volume_map_observation,
};
use crate::error::{
    AUDIT_INVALID, AUDIT_MISMATCH, REPORT_CAPACITY_EXCEEDED, SCENARIO_INVALID, WaterError,
};
use crate::hash::{self, FrozenRoots};
use crate::model::{AcceptedFrame, Geometry, StepSummary, StorageOrder, Vec3i};
use crate::oracle::command::{
    tool_commit, tool_tree_state, validate_output_path, validate_report_capacity,
};
use crate::{boundary, profile, scenario, solver};

use super::{HYDRO_SOAK_STEPS, HydroCalibrationTrace, first_mismatch};

pub(super) const CANDIDATE_ID: &str = "volume-map-box-bender2019-ref-v1";
const LOCAL_PARTITION_LIMIT_PPB: i64 = 100_000;

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
    source_basis: SourceBasis,
    frozen_operations: FrozenOperations,
    classification: String,
    corpus_credit: String,
    selection_status: String,
    local_disposition: String,
    comparison: String,
    first_mismatch: Option<String>,
    partition_status: String,
    local_partition_limit_ppb: i64,
    maximum_absolute_selected_partition_error_ppb: i64,
    feature_normal_status: String,
    conclusion: String,
    product_check: String,
    external_aggregate_comparison: String,
    fluid_sample_count: usize,
    production_trace: HydroCalibrationTrace,
    independent_trace: HydroCalibrationTrace,
    field_probes: Vec<FieldProbe>,
    normal_hydro_step: NormalHydroStep,
    original_ceiling_hydro_soak: HydroSoak,
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
struct SourceBasis {
    paper: String,
    paper_doi: String,
    reference_repository: String,
    reference_commit: String,
    analytical_substitution: String,
}

#[derive(Serialize)]
struct FrozenOperations {
    signed_distance: String,
    extension: String,
    quadrature: String,
    quadrature_degree: u8,
    quadrature_points_per_axis: usize,
    maximum_integrand_evaluations_per_query: usize,
    spherical_support_cutoff: String,
    reference_volume_scale_bits: String,
    virtual_sample: String,
    maximum_virtual_samples_per_fluid_row: usize,
    maximum_directed_virtual_samples: usize,
    continuation_state: String,
}

#[derive(Serialize)]
struct FieldProbe {
    label: String,
    comparison: String,
    production: VolumeMapObservation,
    independent: VolumeMapObservation,
}

#[derive(Serialize)]
struct NormalHydroStep {
    status: String,
    terminal_code: String,
    terminal_detail: String,
    summary: Option<StepSummary>,
}

#[derive(Serialize)]
struct HydroSoak {
    status: String,
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

#[derive(Serialize)]
struct FreefallControl {
    status: String,
    compared_frames: u32,
    first_mismatch_frame: Option<u32>,
    baseline_final_frame_root: Option<String>,
    candidate_final_frame_root: Option<String>,
    candidate_terminal_code: Option<String>,
    candidate_terminal_detail: Option<String>,
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

    let started = Instant::now();
    let production_trace = solver::production_volume_map_calibration(&samples, hydro.geometry)?;
    let independent_trace = independent_volume_map_hydro_calibration()?;
    let mut mismatch = first_mismatch(&production_trace, &independent_trace);
    let field_probes = field_probes(hydro.geometry, &mut mismatch)?;
    let maximum_partition_error = production_trace
        .contributions
        .iter()
        .map(|row| row.partition_error_ppb.saturating_abs())
        .max()
        .unwrap_or(i64::MAX);
    let partition_pass = maximum_partition_error <= LOCAL_PARTITION_LIMIT_PPB;
    let normal_status = feature_normal_status(&field_probes);
    let normal_hydro_step = normal_hydro_step(
        &samples,
        hydro.geometry,
        &roots.execution_profile,
        &hydro_root,
    )?;
    let original_ceiling_hydro_soak = run_hydro_soak(
        &samples,
        hydro.geometry,
        &roots.execution_profile,
        &hydro_root,
    )?;
    let freefall_control = compare_freefall(&freefall, &roots.execution_profile, &freefall_root)?;
    let elapsed = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let survived = mismatch.is_none()
        && partition_pass
        && normal_hydro_step.status == "STEP_PASS"
        && original_ceiling_hydro_soak.status == "SOAK_PASS"
        && freefall_control.status == "EXACT_MATCH";
    let local_disposition = if survived {
        "LOCAL_DISCRIMINATOR_SURVIVED"
    } else {
        "CANDIDATE_REJECTED"
    };
    let initial_tree_state = tool_tree_state(repository_root);
    let mut report = CandidateReport {
        report_schema: "nextengine.continuum-water.volume-map-boundary-candidate.v1".to_owned(),
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
        candidate_definition: "one immutable analytical volume-map query per boundary-adjacent fluid row for the closed axis-aligned box; no boundary particles, fitted multiplier, warm start, retained float state or public contract".to_owned(),
        source_basis: SourceBasis {
            paper: "Bender, Kugelstadt, Weiler, Koschier (2019), Volume Maps: An Implicit Boundary Representation for SPH".to_owned(),
            paper_doi: "10.1145/3359566.3360077".to_owned(),
            reference_repository: "InteractiveComputerGraphics/SPlisHSPlasH".to_owned(),
            reference_commit: "eccce86155776f6ac52d5080b1f720a52bf29450".to_owned(),
            analytical_substitution: "replace Discregrid cubic interpolation with the exact signed-distance field of the selected closed axis-aligned box; retain the pinned reference quadrature, 0.8 scale and virtual-point offset".to_owned(),
        },
        frozen_operations: FrozenOperations {
            signed_distance: "positive inside the fluid box, negative in the exterior solid; equal nearest-face normals use the normalized symmetric subgradient".to_owned(),
            extension: "1 in solid; CubicKernel::W(distance)/CubicKernel::W(0) for 0<distance<support; 0 otherwise".to_owned(),
            quadrature: "tensor Gauss-Legendre over [-support,+support]^3 in i-j-k order, discarding samples outside the spherical support".to_owned(),
            quadrature_degree: 30,
            quadrature_points_per_axis: 16,
            maximum_integrand_evaluations_per_query: 4_096,
            spherical_support_cutoff: format!("0x{:016x}", profile::H2.to_bits()),
            reference_volume_scale_bits: format!("0x{:016x}", 0.8_f64.to_bits()),
            virtual_sample: "distance=max(signed_distance+0.5*particle_radius,2*particle_radius); x*=x-distance*normal; density and gradient use the frozen cubic SPH kernel".to_owned(),
            maximum_virtual_samples_per_fluid_row: 1,
            maximum_directed_virtual_samples: profile::MAXIMUM_SAMPLES,
            continuation_state: "NONE; rebuilt from canonical integer position at every substep".to_owned(),
        },
        classification: "COUNTERFACTUAL_NONPARTICLE_BOUNDARY_CANDIDATE".to_owned(),
        corpus_credit: "NO_CORPUS_CREDIT".to_owned(),
        selection_status: "NOT_SELECTED".to_owned(),
        local_disposition: local_disposition.to_owned(),
        comparison: if mismatch.is_none() {
            "EXACT_MATCH".to_owned()
        } else {
            "MISMATCH".to_owned()
        },
        first_mismatch: mismatch.clone(),
        partition_status: if partition_pass {
            "PARTITION_PASS".to_owned()
        } else {
            "PARTITION_FAIL".to_owned()
        },
        local_partition_limit_ppb: LOCAL_PARTITION_LIMIT_PPB,
        maximum_absolute_selected_partition_error_ppb: maximum_partition_error,
        feature_normal_status: normal_status,
        conclusion: if survived {
            "the analytical volume-map candidate survives the bounded local discriminator; external aggregate comparison, successor-root closure and the full W1 corpus remain required".to_owned()
        } else if !partition_pass {
            "the pinned reference volume-map semantics overfill the selected one-radius-clearance boundary layer and fail before W1 promotion; no fitted rescaling or relaxed ceiling is authorized".to_owned()
        } else {
            "the analytical volume-map candidate fails at least one bounded local discriminator and must not be selected".to_owned()
        },
        product_check: "CONTINUUM-WATER-REF-P1=NOT_RUN".to_owned(),
        external_aggregate_comparison: if survived {
            "REQUIRED_BEFORE_SELECTION".to_owned()
        } else {
            "NOT_RUN_LOCAL_REJECTION".to_owned()
        },
        fluid_sample_count: samples.len(),
        production_trace,
        independent_trace,
        field_probes,
        normal_hydro_step,
        original_ceiling_hydro_soak,
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
            "partition_status": report.partition_status,
            "maximum_absolute_selected_partition_error_ppb": report.maximum_absolute_selected_partition_error_ppb,
            "feature_normal_status": report.feature_normal_status,
            "local_disposition": report.local_disposition,
            "selection_status": report.selection_status,
            "normal_hydro_step": report.normal_hydro_step.status,
            "original_ceiling_hydro_soak": report.original_ceiling_hydro_soak.status,
            "original_ceiling_completed_steps": report.original_ceiling_hydro_soak.completed_steps,
            "freefall_control": report.freefall_control.status,
            "product_check": report.product_check,
        }
    });
    serde_json::to_string(&command).map_err(|error| {
        WaterError::new(
            REPORT_CAPACITY_EXCEEDED,
            format!("cannot serialize volume-map candidate command report: {error}"),
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

fn field_probes(
    geometry: Geometry,
    mismatch: &mut Option<String>,
) -> Result<Vec<FieldProbe>, WaterError> {
    let definitions = [
        ("corner-grid", Vec3i::new(25_000, 25_000, 25_000)),
        ("edge-grid", Vec3i::new(25_000, 25_000, 525_000)),
        ("face-grid", Vec3i::new(25_000, 375_000, 525_000)),
        ("interior-grid", Vec3i::new(525_000, 375_000, 525_000)),
        ("face-off-grid", Vec3i::new(31_337, 376_111, 526_777)),
        ("edge-x-nearer", Vec3i::new(24_999, 25_001, 525_000)),
        ("edge-y-nearer", Vec3i::new(25_001, 24_999, 525_000)),
        ("support-inner", Vec3i::new(99_999, 375_000, 525_000)),
        ("support-boundary", Vec3i::new(100_000, 375_000, 525_000)),
    ];
    let mut probes = Vec::new();
    probes
        .try_reserve_exact(definitions.len())
        .map_err(|error| {
            WaterError::new(
                AUDIT_INVALID,
                format!("volume-map field probe allocation failed: {error}"),
            )
        })?;
    for (label, position) in definitions {
        let production = production_volume_map_observation(position, geometry)?;
        let independent = independent_volume_map_observation(position)?;
        let matched = production == independent;
        if !matched && mismatch.is_none() {
            *mismatch = Some(format!("volume-map field probe {label} differs"));
        }
        probes.push(FieldProbe {
            label: label.to_owned(),
            comparison: if matched {
                "EXACT_MATCH".to_owned()
            } else {
                "MISMATCH".to_owned()
            },
            production,
            independent,
        });
    }
    Ok(probes)
}

fn feature_normal_status(probes: &[FieldProbe]) -> String {
    let x_nearer = probes
        .iter()
        .find(|probe| probe.label == "edge-x-nearer")
        .and_then(|probe| probe.production.volume_gradient_bits.as_ref());
    let y_nearer = probes
        .iter()
        .find(|probe| probe.label == "edge-y-nearer")
        .and_then(|probe| probe.production.volume_gradient_bits.as_ref());
    if x_nearer.is_some() && y_nearer.is_some() && x_nearer != y_nearer {
        "EXPECTED_ANALYTICAL_EDGE_NORMAL_SWITCH_OBSERVED".to_owned()
    } else {
        "NO_EDGE_NORMAL_SWITCH_OBSERVED".to_owned()
    }
}

fn normal_hydro_step(
    samples: &[crate::model::CanonicalSample],
    geometry: Geometry,
    execution_root: &[u8; 32],
    scenario_root: &[u8; 32],
) -> Result<NormalHydroStep, WaterError> {
    let (initial, _) = solver::initial_frame_volume_map(
        samples.to_vec(),
        geometry,
        execution_root,
        scenario_root,
    )?;
    match solver::substep_volume_map(&initial, geometry, execution_root, scenario_root) {
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

fn run_hydro_soak(
    samples: &[crate::model::CanonicalSample],
    geometry: Geometry,
    execution_root: &[u8; 32],
    scenario_root: &[u8; 32],
) -> Result<HydroSoak, WaterError> {
    let (mut frame, _) = solver::initial_frame_volume_map(
        samples.to_vec(),
        geometry,
        execution_root,
        scenario_root,
    )?;
    let mut maximum_density_iterations = 0_u8;
    let mut maximum_density_error_ppb = 0_i64;
    let mut maximum_penetration_um = 0_i64;
    for step in 1..=HYDRO_SOAK_STEPS {
        let outcome =
            match solver::substep_volume_map(&frame, geometry, execution_root, scenario_root) {
                Ok(outcome) => outcome,
                Err(error) => {
                    return Ok(HydroSoak {
                        status: "CANDIDATE_REJECTED".to_owned(),
                        density_ceiling: 20,
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
        density_ceiling: 20,
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

fn compare_freefall(
    scenario: &crate::model::Scenario,
    execution_root: &[u8; 32],
    scenario_root: &[u8; 32],
) -> Result<FreefallControl, WaterError> {
    let samples = scenario::initial_samples(scenario, StorageOrder::Reverse)?;
    let baseline_boundary = boundary::build(scenario.geometry)?;
    let (mut baseline, _) = solver::initial_frame(
        samples.clone(),
        scenario.geometry,
        &baseline_boundary,
        execution_root,
        scenario_root,
    )?;
    let (mut candidate, _) = solver::initial_frame_volume_map(
        samples,
        scenario.geometry,
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
        candidate = match solver::substep_volume_map(
            &candidate,
            scenario.geometry,
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
            format!("cannot serialize volume-map candidate report: {error}"),
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
                    "cannot create volume-map candidate report {}: {error}",
                    output.display()
                ),
            )
        })?;
    file.write_all(&bytes).map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!(
                "cannot write volume-map candidate report {}: {error}",
                output.display()
            ),
        )
    })?;
    file.sync_all().map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!(
                "cannot sync volume-map candidate report {}: {error}",
                output.display()
            ),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_and_independent_field_probes_match_exactly() {
        let hydro = scenario::find("CW-HYDRO-001").unwrap();
        let mut mismatch = None;
        let probes = field_probes(hydro.geometry, &mut mismatch).unwrap();
        assert_eq!(mismatch, None);
        assert!(probes.iter().all(|probe| probe.comparison == "EXACT_MATCH"));
        assert_eq!(
            feature_normal_status(&probes),
            "EXPECTED_ANALYTICAL_EDGE_NORMAL_SWITCH_OBSERVED"
        );
    }
}
