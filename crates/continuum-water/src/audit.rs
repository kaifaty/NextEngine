#![forbid(unsafe_code)]

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::{
    AUDIT_INVALID, AUDIT_MISMATCH, REPORT_CAPACITY_EXCEEDED, SCENARIO_INVALID, WaterError,
};
use crate::hash::{self, FrozenRoots};
use crate::model::{CanonicalSample, StorageOrder, Vec3f, Vec3i};
use crate::oracle::command::{
    tool_commit, tool_tree_state, validate_output_path, validate_report_capacity,
};
use crate::volume_map::VolumeMapBoundary;
use crate::{boundary, profile, scenario, solver};

mod independent;

pub(crate) const SELECTED_ROWS: [(u32, &str); 4] = [
    (0, "corner-x0-y0-z0"),
    (200, "edge-x0-y0"),
    (3_000, "face-x0"),
    (3_010, "interior"),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuditFluidInput {
    pub(crate) id: u32,
    pub(crate) position_um: Vec3i,
    pub(crate) velocity_um_s: Vec3i,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuditBoundaryInput {
    pub(crate) id: u32,
    pub(crate) position_um: Vec3i,
    pub(crate) volume_bits: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct BoundaryNeighborTrace {
    pub(crate) id: u32,
    pub(crate) volume_bits: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DensityInitialTrace {
    pub(crate) rho_ratio_bits: String,
    pub(crate) alpha_bits: String,
    pub(crate) rho_adv_bits: String,
    pub(crate) factor_bits: String,
    pub(crate) multiplier_bits: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DensityIterationRowTrace {
    pub(crate) iteration: u8,
    pub(crate) multiplier_in_bits: String,
    pub(crate) acceleration_bits: [String; 3],
    pub(crate) matrix_action_bits: String,
    pub(crate) error_bits: String,
    pub(crate) multiplier_out_bits: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct HydroRowTrace {
    pub(crate) role: String,
    pub(crate) sample_id: u32,
    pub(crate) position_um: Vec3i,
    pub(crate) fluid_neighbor_ids: Vec<u32>,
    pub(crate) boundary_neighbors: Vec<BoundaryNeighborTrace>,
    pub(crate) initial: Option<DensityInitialTrace>,
    pub(crate) iterations: Vec<DensityIterationRowTrace>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct HydroAuditTrace {
    pub(crate) divergence_iterations: u8,
    pub(crate) divergence_error_ppb: i64,
    pub(crate) density_error_ppb_by_iteration: Vec<i64>,
    pub(crate) density_terminal_code: String,
    pub(crate) density_terminal_detail: String,
    pub(crate) rows: Vec<HydroRowTrace>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuditComputation {
    pub(crate) fluid: Vec<AuditFluidInput>,
    pub(crate) boundary: Vec<AuditBoundaryInput>,
    pub(crate) trace: HydroAuditTrace,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct VolumeMapObservation {
    pub(crate) position_um: Vec3i,
    pub(crate) present: bool,
    pub(crate) signed_distance_bits: Option<String>,
    pub(crate) volume_bits: Option<String>,
    pub(crate) virtual_distance_bits: Option<String>,
    pub(crate) displacement_bits: Option<[String; 3]>,
    pub(crate) kernel_value_bits: Option<String>,
    pub(crate) kernel_gradient_bits: Option<[String; 3]>,
    pub(crate) volume_gradient_bits: Option<[String; 3]>,
    pub(crate) density_contribution_bits: Option<String>,
    pub(crate) density_contribution_ppb: Option<i64>,
    pub(crate) feature_rank: Option<usize>,
}

#[derive(Serialize)]
struct AuditEnvelope<'a> {
    schema_version: u32,
    status: &'a str,
    command: &'a str,
    details: &'a AuditReport,
}

#[derive(Serialize)]
struct AuditReport {
    report_schema: String,
    tool_commit: String,
    tool_tree_state: String,
    toolchain_target: String,
    build_profile: String,
    build_rustflags: String,
    roots: AuditRoots,
    comparison: String,
    first_mismatch: Option<String>,
    conclusion: String,
    product_check: String,
    fluid_sample_count: usize,
    boundary_sample_count: usize,
    fluid_input_root: String,
    boundary_input_root: String,
    production_trace: HydroAuditTrace,
    independent_trace: HydroAuditTrace,
    timing_classification: String,
    wall_clock_nanoseconds: u64,
}

#[derive(Serialize)]
struct AuditRoots {
    w0b_document_root: String,
    float_profile_root: String,
    corpus_root: String,
    execution_profile_root: String,
    scenario_root: String,
}

pub(crate) fn scalar_bits(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

pub(crate) fn vector_bits(value: Vec3f) -> [String; 3] {
    [
        scalar_bits(value.x),
        scalar_bits(value.y),
        scalar_bits(value.z),
    ]
}

pub(crate) fn run_xtask(
    repository_root: &Path,
    mut arguments: impl Iterator<Item = String>,
) -> Result<String, WaterError> {
    let output_flag = arguments.next().ok_or_else(|| {
        WaterError::new(
            SCENARIO_INVALID,
            "audit-hydro requires --output <absolute-path>",
        )
    })?;
    let output = arguments.next().ok_or_else(|| {
        WaterError::new(
            SCENARIO_INVALID,
            "audit-hydro requires --output <absolute-path>",
        )
    })?;
    if output_flag != "--output" || arguments.next().is_some() {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            "audit-hydro accepts exactly --output <absolute-path>",
        ));
    }
    let output = PathBuf::from(output);
    validate_output_path(repository_root, &output)?;
    profile::validate_execution_profile()?;
    profile::validate_float_environment()?;
    let roots = FrozenRoots::verify(repository_root)?;
    let scenario = scenario::find("CW-HYDRO-001")?;
    let scenario_root = scenario::root_for(&scenario, &roots)?;
    let production_samples = scenario::initial_samples(&scenario, StorageOrder::Reverse)?;
    let production_boundary = boundary::build(scenario.geometry)?;
    let started = Instant::now();
    let production = solver::production_hydro_audit(
        &production_samples,
        scenario.geometry,
        &production_boundary,
    )?;
    let independent = independent::compute()?;
    let elapsed = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);

    let mismatch = first_mismatch(&production, &independent);
    let comparison = if mismatch.is_none() {
        "EXACT_MATCH"
    } else {
        "MISMATCH"
    };
    let fluid_input_root = fluid_input_root(&production.fluid);
    let boundary_input_root = boundary_input_root(&production.boundary);
    let initial_tree_state = tool_tree_state(repository_root);
    let mut report = AuditReport {
        report_schema: "nextengine.continuum-water.hydro-row-audit.v1".to_owned(),
        tool_commit: tool_commit(repository_root),
        tool_tree_state: initial_tree_state,
        toolchain_target: env!("WATER_BUILD_TARGET").to_owned(),
        build_profile: env!("WATER_BUILD_PROFILE").to_owned(),
        build_rustflags: env!("WATER_BUILD_RUSTFLAGS").replace('\u{1f}', " "),
        roots: AuditRoots {
            w0b_document_root: hash::hex(&roots.document),
            float_profile_root: hash::hex(&roots.float_profile),
            corpus_root: hash::hex(&roots.corpus),
            execution_profile_root: hash::hex(&roots.execution_profile),
            scenario_root: hash::hex(&scenario_root),
        },
        comparison: comparison.to_owned(),
        first_mismatch: mismatch.clone(),
        conclusion: if mismatch.is_none() {
            "the independent brute-force audit matches all inputs, boundary volumes, the global residual curve and selected rows bit-for-bit; W0B numerical calibration must be reopened".to_owned()
        } else {
            "the production and independent paths differ; repair W1 before changing W0B".to_owned()
        },
        product_check: "CONTINUUM-WATER-REF-P1=NOT_RUN".to_owned(),
        fluid_sample_count: production.fluid.len(),
        boundary_sample_count: production.boundary.len(),
        fluid_input_root,
        boundary_input_root,
        production_trace: production.trace,
        independent_trace: independent.trace,
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
            format!("{detail}; audit report written to {}", output.display()),
        ));
    }
    let command = serde_json::json!({
        "schema_version": 1,
        "status": "REPORT_ONLY",
        "command": "continuum water audit-hydro",
        "details": {
            "report": output.display().to_string(),
            "comparison": "EXACT_MATCH",
            "density_terminal": report.production_trace.density_terminal_code,
            "final_density_error_ppb": report.production_trace.density_error_ppb_by_iteration.last(),
            "product_check": report.product_check,
        }
    });
    serde_json::to_string(&command).map_err(|error| {
        WaterError::new(
            REPORT_CAPACITY_EXCEEDED,
            format!("cannot serialize audit command report: {error}"),
        )
    })
}

fn first_mismatch(production: &AuditComputation, independent: &AuditComputation) -> Option<String> {
    if production.fluid.len() != independent.fluid.len() {
        return Some(format!(
            "fluid count {} != {}",
            production.fluid.len(),
            independent.fluid.len()
        ));
    }
    if let Some(index) = production
        .fluid
        .iter()
        .zip(&independent.fluid)
        .position(|(left, right)| left != right)
    {
        return Some(format!("fluid input differs at sorted index {index}"));
    }
    if production.boundary.len() != independent.boundary.len() {
        return Some(format!(
            "boundary count {} != {}",
            production.boundary.len(),
            independent.boundary.len()
        ));
    }
    if let Some(index) = production
        .boundary
        .iter()
        .zip(&independent.boundary)
        .position(|(left, right)| left != right)
    {
        return Some(format!(
            "boundary input or volume differs at sorted index {index}"
        ));
    }
    trace_mismatch(&production.trace, &independent.trace)
}

fn trace_mismatch(production: &HydroAuditTrace, independent: &HydroAuditTrace) -> Option<String> {
    if production.divergence_iterations != independent.divergence_iterations
        || production.divergence_error_ppb != independent.divergence_error_ppb
    {
        return Some("divergence trace differs".to_owned());
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
            "global density error differs at iteration {}",
            index + 1
        ));
    }
    if production.density_terminal_code != independent.density_terminal_code
        || production.density_terminal_detail != independent.density_terminal_detail
    {
        return Some("density terminal result differs".to_owned());
    }
    if production.rows.len() != independent.rows.len() {
        return Some("selected row count differs".to_owned());
    }
    for (left, right) in production.rows.iter().zip(&independent.rows) {
        if left != right {
            return Some(row_mismatch(left, right));
        }
    }
    None
}

fn row_mismatch(left: &HydroRowTrace, right: &HydroRowTrace) -> String {
    if left.sample_id != right.sample_id || left.position_um != right.position_um {
        return format!("selected row identity differs for {}", left.role);
    }
    if left.fluid_neighbor_ids != right.fluid_neighbor_ids {
        return format!("fluid neighbors differ for sample {}", left.sample_id);
    }
    if left.boundary_neighbors != right.boundary_neighbors {
        return format!("boundary neighbors differ for sample {}", left.sample_id);
    }
    if left.initial != right.initial {
        return format!(
            "initial density fields differ for sample {}",
            left.sample_id
        );
    }
    let iteration = left
        .iterations
        .iter()
        .zip(&right.iterations)
        .position(|(a, b)| a != b)
        .map_or(0, |index| index + 1);
    format!(
        "density iteration {iteration} differs for sample {}",
        left.sample_id
    )
}

pub(crate) fn fluid_input_root(samples: &[AuditFluidInput]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"nextengine.continuum-water.audit-fluid-input.v1\0");
    for sample in samples {
        hasher.update(sample.id.to_le_bytes());
        for value in [
            sample.position_um.x,
            sample.position_um.y,
            sample.position_um.z,
            sample.velocity_um_s.x,
            sample.velocity_um_s.y,
            sample.velocity_um_s.z,
        ] {
            hasher.update(value.to_le_bytes());
        }
    }
    hash::hex(&hasher.finalize().into())
}

pub(crate) fn boundary_input_root(samples: &[AuditBoundaryInput]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"nextengine.continuum-water.audit-boundary-input.v1\0");
    for sample in samples {
        hasher.update(sample.id.to_le_bytes());
        for value in [
            sample.position_um.x,
            sample.position_um.y,
            sample.position_um.z,
        ] {
            hasher.update(value.to_le_bytes());
        }
        hasher.update(sample.volume_bits.as_bytes());
    }
    hash::hex(&hasher.finalize().into())
}

fn write_report(output: &Path, report: &AuditReport, matched: bool) -> Result<(), WaterError> {
    let envelope = AuditEnvelope {
        schema_version: 1,
        status: if matched { "REPORT_ONLY" } else { "FAIL" },
        command: "continuum water audit-hydro",
        details: report,
    };
    let mut bytes = serde_json::to_vec_pretty(&envelope).map_err(|error| {
        WaterError::new(
            AUDIT_INVALID,
            format!("cannot serialize hydro audit report: {error}"),
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
                format!("cannot create audit report {}: {error}", output.display()),
            )
        })?;
    file.write_all(&bytes).map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!("cannot write audit report {}: {error}", output.display()),
        )
    })?;
    file.sync_all().map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!("cannot sync audit report {}: {error}", output.display()),
        )
    })
}

pub(crate) fn production_fluid_input(
    samples: &[CanonicalSample],
) -> Result<Vec<AuditFluidInput>, WaterError> {
    let mut result = Vec::new();
    result.try_reserve_exact(samples.len()).map_err(|error| {
        WaterError::new(
            AUDIT_INVALID,
            format!("production audit input allocation failed: {error}"),
        )
    })?;
    for sample in samples {
        result.push(AuditFluidInput {
            id: sample.id,
            position_um: sample.position_um,
            velocity_um_s: sample.velocity_um_s,
        });
    }
    Ok(result)
}

pub(crate) fn independent_hydro_calibration()
-> Result<crate::calibration::HydroCalibrationTrace, WaterError> {
    independent::compute_calibration()
}

pub(crate) fn independent_ghost_hydro_calibration()
-> Result<crate::calibration::CandidateCalibrationComputation, WaterError> {
    independent::compute_ghost_calibration()
}

pub(crate) fn independent_support_complete_hydro_calibration()
-> Result<crate::calibration::CandidateCalibrationComputation, WaterError> {
    independent::compute_support_complete_calibration()
}

pub(crate) fn independent_support_complete_pressure_probe()
-> Result<crate::calibration::PressureOperatorProbe, WaterError> {
    independent::compute_support_complete_pressure_probe()
}

pub(crate) fn independent_support_complete_projected_pcg_first_step_probe()
-> Result<crate::calibration::ProjectedPcgFirstStepProbe, WaterError> {
    independent::compute_support_complete_projected_pcg_first_step_probe()
}

pub(crate) fn independent_contact_projection_probe()
-> Result<crate::calibration::ContactProjectionProbe, WaterError> {
    independent::compute_contact_projection_probe()
}

pub(crate) fn independent_volume_map_hydro_calibration()
-> Result<crate::calibration::HydroCalibrationTrace, WaterError> {
    independent::compute_volume_map_calibration()
}

pub(crate) fn production_volume_map_observation(
    position_um: Vec3i,
    geometry: crate::model::Geometry,
) -> Result<VolumeMapObservation, WaterError> {
    let position = Vec3f::new(
        profile::decode_micrometres(position_um.x)?,
        profile::decode_micrometres(position_um.y)?,
        profile::decode_micrometres(position_um.z)?,
    );
    let sample = VolumeMapBoundary::new(geometry)?.sample(position)?;
    let Some(sample) = sample else {
        return Ok(empty_volume_map_observation(position_um));
    };
    let contribution = crate::model::checked_scalar(
        sample.volume * sample.value,
        "volume-map observed density contribution",
    )?;
    let volume_gradient = sample
        .gradient
        .scale(sample.volume)
        .checked("volume-map observed volume gradient")?;
    Ok(VolumeMapObservation {
        position_um,
        present: true,
        signed_distance_bits: Some(scalar_bits(sample.signed_distance)),
        volume_bits: Some(scalar_bits(sample.volume)),
        virtual_distance_bits: Some(scalar_bits(sample.virtual_distance)),
        displacement_bits: Some(vector_bits(sample.displacement)),
        kernel_value_bits: Some(scalar_bits(sample.value)),
        kernel_gradient_bits: Some(vector_bits(sample.gradient)),
        volume_gradient_bits: Some(vector_bits(volume_gradient)),
        density_contribution_bits: Some(scalar_bits(contribution)),
        density_contribution_ppb: Some(profile::quantize_ppb(contribution)?),
        feature_rank: Some(sample.feature_rank),
    })
}

pub(crate) fn independent_volume_map_observation(
    position_um: Vec3i,
) -> Result<VolumeMapObservation, WaterError> {
    independent::observe_volume_map(position_um)
}

pub(crate) fn empty_volume_map_observation(position_um: Vec3i) -> VolumeMapObservation {
    VolumeMapObservation {
        position_um,
        present: false,
        signed_distance_bits: None,
        volume_bits: None,
        virtual_distance_bits: None,
        displacement_bits: None,
        kernel_value_bits: None,
        kernel_gradient_bits: None,
        volume_gradient_bits: None,
        density_contribution_bits: None,
        density_contribution_ppb: None,
        feature_rank: None,
    }
}

pub(crate) fn independent_zero_velocity_settling()
-> Result<crate::calibration::SettlingComputation, WaterError> {
    independent::compute_zero_velocity_settling()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DENSITY_NONCONVERGENCE;

    #[test]
    fn independent_hydro_audit_matches_production_at_every_observed_bit() {
        let scenario = scenario::find("CW-HYDRO-001").unwrap();
        let samples = scenario::initial_samples(&scenario, StorageOrder::Reverse).unwrap();
        let boundary = boundary::build(scenario.geometry).unwrap();
        let production =
            solver::production_hydro_audit(&samples, scenario.geometry, &boundary).unwrap();
        let independent = independent::compute().unwrap();

        assert_eq!(first_mismatch(&production, &independent), None);
        assert_eq!(
            fluid_input_root(&production.fluid),
            "bd18fe6e6a305ccc875ba12014e90f0cbca74a05068c7bd573ef95d3f3f51dbc"
        );
        assert_eq!(
            boundary_input_root(&production.boundary),
            "301246ba4e6efdf1b2f13e60605a2394aab46b22826f2083f67c6cce1d8df0a7"
        );
        assert_eq!(
            production.trace.density_error_ppb_by_iteration.first(),
            Some(&109_612_910)
        );
        assert_eq!(
            production.trace.density_error_ppb_by_iteration.last(),
            Some(&74_482_699)
        );
        assert_eq!(
            production.trace.density_terminal_code,
            DENSITY_NONCONVERGENCE
        );
        assert_eq!(production.trace.rows.len(), SELECTED_ROWS.len());
        assert_eq!(production.trace.rows[0].fluid_neighbor_ids.len(), 10);
        assert_eq!(production.trace.rows[0].boundary_neighbors.len(), 16);
        assert_eq!(production.trace.rows[3].fluid_neighbor_ids.len(), 32);
        assert!(production.trace.rows[3].boundary_neighbors.is_empty());
    }

    #[test]
    fn independent_volume_map_calibration_matches_production_exactly() {
        let scenario = scenario::find("CW-HYDRO-001").unwrap();
        let samples = scenario::initial_samples(&scenario, StorageOrder::Reverse).unwrap();
        let production =
            solver::production_volume_map_calibration(&samples, scenario.geometry).unwrap();
        let independent = independent_volume_map_hydro_calibration().unwrap();

        assert_eq!(production, independent);
    }

    #[test]
    fn comparison_reports_the_first_global_iteration_mismatch() {
        let mut left = empty_trace();
        let mut right = empty_trace();
        left.density_error_ppb_by_iteration = vec![10, 9, 8];
        right.density_error_ppb_by_iteration = vec![10, 7, 8];
        assert_eq!(
            trace_mismatch(&left, &right).as_deref(),
            Some("global density error differs at iteration 2")
        );
    }

    #[test]
    fn audit_cli_requires_its_single_bounded_output_argument() {
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let error = crate::run_xtask(
            repository_root,
            ["water".to_owned(), "audit-hydro".to_owned()].into_iter(),
        )
        .unwrap_err();
        assert!(error.starts_with("WATER_SCENARIO_INVALID:"));
    }

    fn empty_trace() -> HydroAuditTrace {
        HydroAuditTrace {
            divergence_iterations: 1,
            divergence_error_ppb: 0,
            density_error_ppb_by_iteration: Vec::new(),
            density_terminal_code: DENSITY_NONCONVERGENCE.to_owned(),
            density_terminal_detail: "iteration 20 ended at 1 ppb".to_owned(),
            rows: Vec::new(),
        }
    }
}
