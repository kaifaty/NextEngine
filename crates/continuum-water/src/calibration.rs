#![forbid(unsafe_code)]

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Serialize;

use crate::audit::{
    AuditBoundaryInput, boundary_input_root, fluid_input_root, independent_hydro_calibration,
    production_fluid_input,
};
use crate::error::{
    AUDIT_INVALID, AUDIT_MISMATCH, REPORT_CAPACITY_EXCEEDED, SCENARIO_INVALID, WaterError,
};
use crate::hash::{self, FrozenRoots};
use crate::model::{StorageOrder, Vec3i};
use crate::oracle::command::{
    tool_commit, tool_tree_state, validate_output_path, validate_report_capacity,
};
use crate::{boundary, profile, scenario, solver};

mod candidate;

pub(crate) use candidate::run_xtask as run_candidate_xtask;

pub(crate) const DIAGNOSTIC_MAX_ITERATIONS: u16 = 320;
pub(crate) const DIAGNOSTIC_CHECKPOINTS: [u16; 5] = [20, 40, 80, 160, 320];

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct BoundaryFeatureContribution {
    pub(crate) face_count: usize,
    pub(crate) edge_count: usize,
    pub(crate) corner_count: usize,
    pub(crate) face_bits: String,
    pub(crate) edge_bits: String,
    pub(crate) corner_bits: String,
    pub(crate) total_bits: String,
    pub(crate) face_ppb: i64,
    pub(crate) edge_ppb: i64,
    pub(crate) corner_ppb: i64,
    pub(crate) total_ppb: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DensityContributionRow {
    pub(crate) role: String,
    pub(crate) sample_id: u32,
    pub(crate) position_um: Vec3i,
    pub(crate) self_bits: String,
    pub(crate) fluid_bits: String,
    pub(crate) self_plus_fluid_bits: String,
    pub(crate) boundary: BoundaryFeatureContribution,
    pub(crate) grouped_rho_ratio_bits: String,
    pub(crate) reconstructed_rho_ratio_bits: String,
    pub(crate) grouping_delta_bits: String,
    pub(crate) self_plus_fluid_ppb: i64,
    pub(crate) reconstructed_rho_ratio_ppb: i64,
    pub(crate) partition_error_ppb: i64,
    pub(crate) diagnostic_required_boundary_scale_bits: Option<String>,
    pub(crate) diagnostic_required_boundary_scale_ppb: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ExtendedDensityCheckpoint {
    pub(crate) iteration: u16,
    pub(crate) density_error_ppb: i64,
    pub(crate) original_threshold_met: bool,
    pub(crate) maximum_multiplier_bits: String,
    pub(crate) prospective_maximum_speed_um_s: i64,
    pub(crate) prospective_minimum_velocity_y_um_s: i64,
    pub(crate) prospective_maximum_velocity_y_um_s: i64,
    pub(crate) prospective_minimum_outer_clearance_um: i64,
    pub(crate) prospective_maximum_penetration_um: i64,
    pub(crate) prospective_outer_escape: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct HydroCalibrationTrace {
    pub(crate) contributions: Vec<DensityContributionRow>,
    pub(crate) density_error_ppb_by_iteration: Vec<i64>,
    pub(crate) first_original_threshold_iteration: Option<u16>,
    pub(crate) first_original_threshold_checkpoint: Option<ExtendedDensityCheckpoint>,
    pub(crate) checkpoints: Vec<ExtendedDensityCheckpoint>,
}

pub(crate) struct ExtendedDensityTrace {
    pub(crate) errors_ppb: Vec<i64>,
    pub(crate) first_original_threshold_iteration: Option<u16>,
    pub(crate) first_original_threshold_checkpoint: Option<ExtendedDensityCheckpoint>,
    pub(crate) checkpoints: Vec<ExtendedDensityCheckpoint>,
}

pub(crate) struct CandidateCalibrationComputation {
    pub(crate) boundary: Vec<AuditBoundaryInput>,
    pub(crate) trace: HydroCalibrationTrace,
}

#[derive(Serialize)]
struct DiagnosticEnvelope<'a> {
    schema_version: u32,
    status: &'a str,
    command: &'a str,
    details: &'a DiagnosticReport,
}

#[derive(Serialize)]
struct DiagnosticReport {
    report_schema: String,
    tool_commit: String,
    tool_tree_state: String,
    toolchain_target: String,
    build_profile: String,
    build_rustflags: String,
    roots: DiagnosticRoots,
    comparison: String,
    first_mismatch: Option<String>,
    classification: String,
    corpus_credit: String,
    conclusion: String,
    product_check: String,
    original_density_ceiling: u16,
    diagnostic_density_ceiling: u16,
    diagnostic_checkpoints: [u16; 5],
    fluid_sample_count: usize,
    boundary_sample_count: usize,
    fluid_input_root: String,
    boundary_input_root: String,
    production_trace: HydroCalibrationTrace,
    independent_trace: HydroCalibrationTrace,
    timing_classification: String,
    wall_clock_nanoseconds: u64,
}

#[derive(Serialize)]
struct DiagnosticRoots {
    w0b_document_root: String,
    float_profile_root: String,
    corpus_root: String,
    execution_profile_root: String,
    scenario_root: String,
}

pub(crate) fn run_xtask(
    repository_root: &Path,
    mut arguments: impl Iterator<Item = String>,
) -> Result<String, WaterError> {
    let output_flag = arguments.next().ok_or_else(argument_error)?;
    let output = arguments.next().ok_or_else(argument_error)?;
    if output_flag != "--output" || arguments.next().is_some() {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            "diagnose-hydro accepts exactly --output <absolute-path>",
        ));
    }
    let output = PathBuf::from(output);
    validate_output_path(repository_root, &output)?;
    profile::validate_execution_profile()?;
    profile::validate_float_environment()?;
    let roots = FrozenRoots::verify(repository_root)?;
    let scenario = scenario::find("CW-HYDRO-001")?;
    let scenario_root = scenario::root_for(&scenario, &roots)?;
    let samples = scenario::initial_samples(&scenario, StorageOrder::Reverse)?;
    let boundary = boundary::build(scenario.geometry)?;
    let mut fluid_input = production_fluid_input(&samples)?;
    fluid_input.sort_unstable_by_key(|sample| sample.id);
    let mut boundary_input = Vec::new();
    boundary_input
        .try_reserve_exact(boundary.len())
        .map_err(|error| {
            WaterError::new(
                AUDIT_INVALID,
                format!("hydro diagnostic boundary input allocation failed: {error}"),
            )
        })?;
    for sample in &boundary {
        boundary_input.push(AuditBoundaryInput {
            id: sample.id,
            position_um: sample.position_um,
            volume_bits: crate::audit::scalar_bits(sample.volume),
        });
    }

    let started = Instant::now();
    let production = solver::production_hydro_calibration(&samples, &boundary, scenario.geometry)?;
    let independent = independent_hydro_calibration()?;
    let elapsed = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let mismatch = first_mismatch(&production, &independent);
    let comparison = if mismatch.is_none() {
        "EXACT_MATCH"
    } else {
        "MISMATCH"
    };
    let initial_tree_state = tool_tree_state(repository_root);
    let mut report = DiagnosticReport {
        report_schema: "nextengine.continuum-water.hydro-calibration-diagnostic.v1".to_owned(),
        tool_commit: tool_commit(repository_root),
        tool_tree_state: initial_tree_state,
        toolchain_target: env!("WATER_BUILD_TARGET").to_owned(),
        build_profile: env!("WATER_BUILD_PROFILE").to_owned(),
        build_rustflags: env!("WATER_BUILD_RUSTFLAGS").replace('\u{1f}', " "),
        roots: DiagnosticRoots {
            w0b_document_root: hash::hex(&roots.document),
            float_profile_root: hash::hex(&roots.float_profile),
            corpus_root: hash::hex(&roots.corpus),
            execution_profile_root: hash::hex(&roots.execution_profile),
            scenario_root: hash::hex(&scenario_root),
        },
        comparison: comparison.to_owned(),
        first_mismatch: mismatch.clone(),
        classification: "COUNTERFACTUAL_DIAGNOSTIC".to_owned(),
        corpus_credit: "NO_CORPUS_CREDIT".to_owned(),
        conclusion: if mismatch.is_none() {
            "production and independent calculators agree on the W0B contribution decomposition and unchanged-profile extended curve; the report diagnoses candidates but cannot promote the original profile".to_owned()
        } else {
            "production and independent W0C diagnostics differ; no calibration inference is admissible until the mismatch is repaired".to_owned()
        },
        product_check: "CONTINUUM-WATER-REF-P1=NOT_RUN".to_owned(),
        original_density_ceiling: 20,
        diagnostic_density_ceiling: DIAGNOSTIC_MAX_ITERATIONS,
        diagnostic_checkpoints: DIAGNOSTIC_CHECKPOINTS,
        fluid_sample_count: fluid_input.len(),
        boundary_sample_count: boundary_input.len(),
        fluid_input_root: fluid_input_root(&fluid_input),
        boundary_input_root: boundary_input_root(&boundary_input),
        production_trace: production,
        independent_trace: independent,
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
            format!(
                "{detail}; diagnostic report written to {}",
                output.display()
            ),
        ));
    }
    let final_error = report
        .production_trace
        .density_error_ppb_by_iteration
        .last()
        .copied();
    let command = serde_json::json!({
        "schema_version": 1,
        "status": "REPORT_ONLY",
        "command": "continuum water diagnose-hydro",
        "details": {
            "report": output.display().to_string(),
            "comparison": "EXACT_MATCH",
            "classification": report.classification,
            "corpus_credit": report.corpus_credit,
            "first_original_threshold_iteration": report.production_trace.first_original_threshold_iteration,
            "iteration_320_density_error_ppb": final_error,
            "product_check": report.product_check,
        }
    });
    serde_json::to_string(&command).map_err(|error| {
        WaterError::new(
            REPORT_CAPACITY_EXCEEDED,
            format!("cannot serialize hydro diagnostic command report: {error}"),
        )
    })
}

fn argument_error() -> WaterError {
    WaterError::new(
        SCENARIO_INVALID,
        "diagnose-hydro requires --output <absolute-path>",
    )
}

pub(crate) fn first_mismatch(
    production: &HydroCalibrationTrace,
    independent: &HydroCalibrationTrace,
) -> Option<String> {
    if production.contributions.len() != independent.contributions.len() {
        return Some("density contribution row count differs".to_owned());
    }
    if let Some(index) = production
        .contributions
        .iter()
        .zip(&independent.contributions)
        .position(|(left, right)| left != right)
    {
        return Some(format!("density contribution row {index} differs"));
    }
    if production.density_error_ppb_by_iteration != independent.density_error_ppb_by_iteration {
        let index = production
            .density_error_ppb_by_iteration
            .iter()
            .zip(&independent.density_error_ppb_by_iteration)
            .position(|(left, right)| left != right)
            .unwrap_or_else(|| {
                production
                    .density_error_ppb_by_iteration
                    .len()
                    .min(independent.density_error_ppb_by_iteration.len())
            });
        return Some(format!(
            "extended density error differs at iteration {}",
            index + 1
        ));
    }
    if production.first_original_threshold_iteration
        != independent.first_original_threshold_iteration
    {
        return Some("first threshold iteration differs".to_owned());
    }
    if production.first_original_threshold_checkpoint
        != independent.first_original_threshold_checkpoint
    {
        return Some("first threshold checkpoint differs".to_owned());
    }
    if production.checkpoints.len() != independent.checkpoints.len() {
        return Some("extended checkpoint count differs".to_owned());
    }
    if let Some(index) = production
        .checkpoints
        .iter()
        .zip(&independent.checkpoints)
        .position(|(left, right)| left != right)
    {
        return Some(format!(
            "prospective checkpoint {} differs",
            production.checkpoints[index].iteration
        ));
    }
    None
}

fn write_report(output: &Path, report: &DiagnosticReport, matched: bool) -> Result<(), WaterError> {
    let envelope = DiagnosticEnvelope {
        schema_version: 1,
        status: if matched { "REPORT_ONLY" } else { "FAIL" },
        command: "continuum water diagnose-hydro",
        details: report,
    };
    let mut bytes = serde_json::to_vec_pretty(&envelope).map_err(|error| {
        WaterError::new(
            AUDIT_INVALID,
            format!("cannot serialize hydro diagnostic report: {error}"),
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
                    "cannot create hydro diagnostic report {}: {error}",
                    output.display()
                ),
            )
        })?;
    file.write_all(&bytes).map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!(
                "cannot write hydro diagnostic report {}: {error}",
                output.display()
            ),
        )
    })?;
    file.sync_all().map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!(
                "cannot sync hydro diagnostic report {}: {error}",
                output.display()
            ),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extended_diagnostic_matches_independent_calculator() {
        let scenario = scenario::find("CW-HYDRO-001").unwrap();
        let samples = scenario::initial_samples(&scenario, StorageOrder::Reverse).unwrap();
        let boundary = boundary::build(scenario.geometry).unwrap();
        let mut fluid_input = production_fluid_input(&samples).unwrap();
        fluid_input.sort_unstable_by_key(|sample| sample.id);
        let production =
            solver::production_hydro_calibration(&samples, &boundary, scenario.geometry).unwrap();
        let independent = independent_hydro_calibration().unwrap();

        assert_eq!(first_mismatch(&production, &independent), None);
        assert_eq!(
            fluid_input_root(&fluid_input),
            "bd18fe6e6a305ccc875ba12014e90f0cbca74a05068c7bd573ef95d3f3f51dbc"
        );
        assert_eq!(
            production.density_error_ppb_by_iteration.len(),
            usize::from(DIAGNOSTIC_MAX_ITERATIONS)
        );
        assert_eq!(production.checkpoints.len(), DIAGNOSTIC_CHECKPOINTS.len());
        assert_eq!(
            production.contributions.len(),
            crate::audit::SELECTED_ROWS.len()
        );
        assert_eq!(
            production
                .contributions
                .iter()
                .map(|row| row.partition_error_ppb)
                .collect::<Vec<_>>(),
            [798_771_849, 734_263_219, 544_669_732, -27_534]
        );
        assert_eq!(
            production
                .checkpoints
                .iter()
                .map(|checkpoint| checkpoint.density_error_ppb)
                .collect::<Vec<_>>(),
            [74_482_699, 51_452_219, 24_624_457, 5_741_077, 382_257]
        );
        assert_eq!(
            production
                .checkpoints
                .iter()
                .map(|checkpoint| checkpoint.prospective_minimum_outer_clearance_um)
                .collect::<Vec<_>>(),
            [-83_638, -110_060, -138_816, -159_135, -165_233]
        );
        assert_eq!(production.first_original_threshold_iteration, None);
        assert_eq!(production.first_original_threshold_checkpoint, None);

        let mut changed = independent.clone();
        changed.density_error_ppb_by_iteration[39] -= 1;
        assert_eq!(
            first_mismatch(&production, &changed).as_deref(),
            Some("extended density error differs at iteration 40")
        );
    }

    #[test]
    fn diagnostic_cli_requires_its_single_bounded_output_argument() {
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let error = crate::run_xtask(
            repository_root,
            ["water".to_owned(), "diagnose-hydro".to_owned()].into_iter(),
        )
        .unwrap_err();
        assert!(error.starts_with("WATER_SCENARIO_INVALID:"));
    }
}
