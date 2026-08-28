use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::super::{
    canonical_external_file, read_bounded_file, require_empty_output, resolve_cli_path,
    resolve_output_path, sha256_hex,
};
use super::model::{SolverResult, solve_fixture};
use super::{DataPolicy, Fixture, LevelReport, Solver};

const MANIFEST_SCHEMA: &str = "nextengine.experimental-physical-sound-bem-quadrature.manifest.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-bem-quadrature.report.v1";
const STUDY_ID: &str = "physical-sound-bem-panel-quadrature-discriminator";
const PROTOCOL_REVISION: &str = "pulsating-sphere-seven-point-quadrature-v1";
const MANIFEST_SHA256: &str = "b9ff02c9097893f9a568f25bc8bd9c4a32e6593e3a104ab114e888521c5dff47";
const SOURCE_MANIFEST_SHA256: &str =
    "9a26ca137681b319b2ffb490dd1e4b897aeaf4de19349c7cbb669c97669c8095";
const SOURCE_REPORT_SHA256: &str =
    "6f74a30988ff349c270d6cb0ba37cfbf905e1dc1b078868660d01fb1c622a689";
const MAX_MANIFEST_BYTES: usize = 128 * 1024;
const MAX_REPORT_BYTES: usize = 2 * 1024 * 1024;
static NEXT_STAGING: AtomicU64 = AtomicU64::new(0);

pub(super) fn run_cli(
    root: &Path,
    mut arguments: impl Iterator<Item = String>,
) -> Result<(), String> {
    let mut manifest = None;
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--manifest" => set_once(&mut manifest, PathBuf::from(value), &flag)?,
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            _ => return Err(format!("unexpected BEM quadrature argument: {flag}")),
        }
    }
    let manifest = manifest.ok_or_else(|| {
        "bem-feasibility quadrature requires --manifest <external-json>".to_owned()
    })?;
    let output = output.ok_or_else(|| {
        "bem-feasibility quadrature requires --output <external-empty-directory>".to_owned()
    })?;
    run(root, &manifest, &output)
}

fn set_once<T>(slot: &mut Option<T>, value: T, flag: &str) -> Result<(), String> {
    if slot.replace(value).is_some() {
        return Err(format!("duplicate argument: {flag}"));
    }
    Ok(())
}

fn run(root: &Path, manifest_argument: &Path, output_argument: &Path) -> Result<(), String> {
    let manifest_path = canonical_external_file(
        root,
        &resolve_cli_path(root, manifest_argument),
        "BEM quadrature manifest",
    )?;
    let manifest_bytes = read_bounded_file(
        &manifest_path,
        MAX_MANIFEST_BYTES,
        "BEM quadrature manifest",
    )?;
    require_hash(&manifest_bytes, MANIFEST_SHA256, "BEM quadrature manifest")?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse BEM quadrature manifest: {error}"))?;
    let base = manifest_path
        .parent()
        .ok_or_else(|| "BEM quadrature manifest has no parent".to_owned())?;
    let source_manifest_bytes = read_reference(
        root,
        base,
        &manifest.source_control.manifest_path,
        &manifest.source_control.manifest_sha256,
        MAX_MANIFEST_BYTES,
        "BEM source manifest",
    )?;
    let source_report_bytes = read_reference(
        root,
        base,
        &manifest.source_control.report_path,
        &manifest.source_control.report_sha256,
        MAX_REPORT_BYTES,
        "BEM source report",
    )?;
    let source_manifest: super::Manifest = serde_json::from_slice(&source_manifest_bytes)
        .map_err(|error| format!("parse BEM source manifest: {error}"))?;
    super::validate_manifest(&source_manifest)?;
    let source_report: Value = serde_json::from_slice(&source_report_bytes)
        .map_err(|error| format!("parse BEM source report: {error}"))?;
    validate_manifest(&manifest, &source_manifest, &source_report)?;

    let control_result = solve_fixture(&manifest.fixture, &manifest.control_solver)?;
    let regenerated_source_report = super::build_report(
        &source_manifest,
        solve_fixture(&manifest.fixture, &manifest.control_solver)?,
    )?;
    let regenerated_source_report = super::pretty_json(&regenerated_source_report)?;
    require_hash(
        &regenerated_source_report,
        SOURCE_REPORT_SHA256,
        "regenerated three-point V1 report",
    )?;
    let control = summarize(control_result, &manifest.fixture)?;
    let candidate = summarize(
        solve_fixture(&manifest.fixture, &manifest.candidate_solver)?,
        &manifest.fixture,
    )?;
    let comparison_ratio =
        candidate.fine.median_relative_complex_error / control.fine.median_relative_complex_error;
    if !comparison_ratio.is_finite() {
        return Err("BEM quadrature comparison ratio is non-finite".to_owned());
    }
    let gate = GateReport {
        fine_relative_complex_error_passed: candidate.fine.max_relative_complex_error
            <= manifest
                .admission_gates
                .candidate_fine_max_relative_complex_error,
        fine_magnitude_error_passed: candidate.fine.max_absolute_magnitude_error_db
            <= manifest
                .admission_gates
                .candidate_fine_max_absolute_magnitude_error_db,
        fine_phase_error_passed: candidate.fine.max_absolute_phase_error_degrees
            <= manifest
                .admission_gates
                .candidate_fine_max_absolute_phase_error_degrees,
        fine_direction_symmetry_passed: candidate.fine.max_direction_magnitude_span_db
            <= manifest
                .admission_gates
                .candidate_fine_max_direction_magnitude_span_db,
        refinement_passed: candidate.refinement.median_relative_complex_error_ratio
            <= manifest
                .admission_gates
                .candidate_fine_to_coarse_median_relative_complex_error_ratio,
        comparison_passed: comparison_ratio
            <= manifest
                .admission_gates
                .candidate_to_control_fine_median_relative_complex_error_ratio,
    };
    let passed = gate.all_passed();
    let report = Report {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: if passed {
            "SevenPointPanelQuadratureHypothesisSupported"
        } else {
            "SevenPointPanelQuadratureHypothesisRejected"
        },
        claim: "SYNTHETIC_PANEL_QUADRATURE_DISCRIMINATOR_ONLY / NO_CONVERGED_BEM_OR_REAL_OBJECT_MATERIAL_QUALITY_ADMISSION_RUNTIME_OR_REALIMPACT_CREDIT",
        study_id: STUDY_ID,
        protocol_revision: PROTOCOL_REVISION,
        manifest_sha256: MANIFEST_SHA256,
        source_manifest_sha256: SOURCE_MANIFEST_SHA256,
        source_report_sha256: SOURCE_REPORT_SHA256,
        hypotheses: &manifest.hypotheses,
        control,
        candidate,
        candidate_to_control_fine_median_relative_complex_error_ratio: comparison_ratio,
        thresholds: &manifest.admission_gates,
        gate,
        data_policy: &manifest.data_policy,
        next_action: if passed {
            "freeze an added-resolution or independent classical-solver check before any non-spherical surface mode or fresh REALIMPACT payload"
        } else {
            "reject quadrature-only remediation; keep REALIMPACT sealed and test planar geometry or the boundary formulation against an independent classical solver"
        },
    };
    let report_bytes = pretty_json(&report)?;
    let report_sha256 = sha256_hex(&report_bytes);
    let output = resolve_output_path(root, output_argument)?;
    require_empty_output(&output)?;
    publish(
        &output,
        &manifest_bytes,
        &source_manifest_bytes,
        &source_report_bytes,
        &report_bytes,
    )?;

    println!("physical sound BEM quadrature: {}", output.display());
    println!("manifest sha256: {MANIFEST_SHA256}");
    println!(
        "candidate fine median error: {:.9}",
        report.candidate.fine.median_relative_complex_error
    );
    println!(
        "candidate refinement ratio: {:.9}",
        report
            .candidate
            .refinement
            .median_relative_complex_error_ratio
    );
    println!("candidate/control ratio: {comparison_ratio:.9}");
    println!("decision: {}", report.decision);
    println!("report sha256: {report_sha256}");
    Ok(())
}

fn summarize(result: SolverResult, fixture: &Fixture) -> Result<SolverReport, String> {
    let coarse = super::aggregate_level(result.coarse, fixture)?;
    let fine = super::aggregate_level(result.fine, fixture)?;
    let ratio = fine.median_relative_complex_error / coarse.median_relative_complex_error;
    if !ratio.is_finite() {
        return Err("BEM solver refinement ratio is non-finite".to_owned());
    }
    Ok(SolverReport {
        coarse,
        fine,
        refinement: RefinementReport {
            median_relative_complex_error_ratio: ratio,
        },
    })
}

fn validate_manifest(
    manifest: &Manifest,
    source_manifest: &super::Manifest,
    source_report: &Value,
) -> Result<(), String> {
    let mut expected_candidate = manifest.control_solver.clone();
    expected_candidate.off_diagonal_panel_quadrature = "symmetric-seven-point-triangle".to_owned();
    if manifest.schema != MANIFEST_SCHEMA
        || manifest.study_id != STUDY_ID
        || manifest.protocol_revision != PROTOCOL_REVISION
        || manifest.source_control.manifest_path != "feasibility-manifest.json"
        || manifest.source_control.manifest_sha256 != SOURCE_MANIFEST_SHA256
        || manifest.source_control.report_path != "run-a/report.json"
        || manifest.source_control.report_sha256 != SOURCE_REPORT_SHA256
        || manifest.fixture != source_manifest.fixture
        || manifest.control_solver != source_manifest.solver
        || manifest.candidate_solver != expected_candidate
        || manifest
            .admission_gates
            .candidate_fine_max_relative_complex_error
            != 0.15
        || manifest
            .admission_gates
            .candidate_fine_max_absolute_magnitude_error_db
            != 0.75
        || manifest
            .admission_gates
            .candidate_fine_max_absolute_phase_error_degrees
            != 10.0
        || manifest
            .admission_gates
            .candidate_fine_max_direction_magnitude_span_db
            != 0.25
        || manifest
            .admission_gates
            .candidate_fine_to_coarse_median_relative_complex_error_ratio
            != 0.8
        || manifest
            .admission_gates
            .candidate_to_control_fine_median_relative_complex_error_ratio
            != 0.75
        || !manifest.admission_gates.repeat_report_bytes_identical
        || manifest.data_policy != source_manifest.data_policy
        || source_report.get("schema").and_then(Value::as_str) != Some(super::REPORT_SCHEMA)
        || source_report.get("decision").and_then(Value::as_str)
            != Some("ClassicalBoundarySolverAnalyticalControlRejected")
        || source_report
            .pointer("/gate/refinement_passed")
            .and_then(Value::as_bool)
            != Some(false)
    {
        return Err("BEM quadrature manifest or source lineage changed".to_owned());
    }
    Ok(())
}

fn read_reference(
    root: &Path,
    base: &Path,
    relative: &str,
    expected_sha256: &str,
    maximum_bytes: usize,
    role: &str,
) -> Result<Vec<u8>, String> {
    let path = canonical_external_file(root, &base.join(relative), role)?;
    let bytes = read_bounded_file(&path, maximum_bytes, role)?;
    require_hash(&bytes, expected_sha256, role)?;
    Ok(bytes)
}

fn require_hash(bytes: &[u8], expected: &str, role: &str) -> Result<(), String> {
    let actual = sha256_hex(bytes);
    if actual != expected {
        return Err(format!(
            "{role} hash changed: expected {expected}, got {actual}"
        ));
    }
    Ok(())
}

fn pretty_json(value: &impl Serialize) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("serialize BEM quadrature report: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn publish(
    output: &Path,
    manifest: &[u8],
    source_manifest: &[u8],
    source_report: &[u8],
    report: &[u8],
) -> Result<(), String> {
    let parent = output
        .parent()
        .ok_or_else(|| "BEM quadrature output has no parent".to_owned())?;
    let sequence = NEXT_STAGING.fetch_add(1, Ordering::Relaxed);
    let staging = parent.join(format!(
        ".nextengine-bem-quadrature-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&staging).map_err(|error| format!("create quadrature staging: {error}"))?;
    let result = (|| {
        for (name, bytes) in [
            ("manifest.json", manifest),
            ("source-manifest.json", source_manifest),
            ("source-report.json", source_report),
            ("report.json", report),
        ] {
            fs::write(staging.join(name), bytes)
                .map_err(|error| format!("write quadrature {name}: {error}"))?;
        }
        if output.exists() {
            fs::remove_dir(output)
                .map_err(|error| format!("remove confirmed-empty quadrature output: {error}"))?;
        }
        fs::rename(&staging, output)
            .map_err(|error| format!("publish BEM quadrature output: {error}"))
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: String,
    study_id: String,
    protocol_revision: String,
    hypotheses: Hypotheses,
    source_control: SourceControl,
    fixture: Fixture,
    control_solver: Solver,
    candidate_solver: Solver,
    admission_gates: AdmissionGates,
    data_policy: DataPolicy,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Hypotheses {
    candidate: String,
    control: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceControl {
    manifest_path: String,
    manifest_sha256: String,
    report_path: String,
    report_sha256: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AdmissionGates {
    candidate_fine_max_relative_complex_error: f64,
    candidate_fine_max_absolute_magnitude_error_db: f64,
    candidate_fine_max_absolute_phase_error_degrees: f64,
    candidate_fine_max_direction_magnitude_span_db: f64,
    candidate_fine_to_coarse_median_relative_complex_error_ratio: f64,
    candidate_to_control_fine_median_relative_complex_error_ratio: f64,
    repeat_report_bytes_identical: bool,
}

#[derive(Debug, Serialize)]
struct Report<'a> {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    study_id: &'static str,
    protocol_revision: &'static str,
    manifest_sha256: &'static str,
    source_manifest_sha256: &'static str,
    source_report_sha256: &'static str,
    hypotheses: &'a Hypotheses,
    control: SolverReport,
    candidate: SolverReport,
    candidate_to_control_fine_median_relative_complex_error_ratio: f64,
    thresholds: &'a AdmissionGates,
    gate: GateReport,
    data_policy: &'a DataPolicy,
    next_action: &'static str,
}

#[derive(Debug, Serialize)]
struct SolverReport {
    coarse: LevelReport,
    fine: LevelReport,
    refinement: RefinementReport,
}

#[derive(Debug, Serialize)]
struct RefinementReport {
    median_relative_complex_error_ratio: f64,
}

#[derive(Debug, Serialize)]
struct GateReport {
    fine_relative_complex_error_passed: bool,
    fine_magnitude_error_passed: bool,
    fine_phase_error_passed: bool,
    fine_direction_symmetry_passed: bool,
    refinement_passed: bool,
    comparison_passed: bool,
}

impl GateReport {
    fn all_passed(&self) -> bool {
        self.fine_relative_complex_error_passed
            && self.fine_magnitude_error_passed
            && self.fine_phase_error_passed
            && self.fine_direction_symmetry_passed
            && self.refinement_passed
            && self.comparison_passed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison_gate_is_conjunctive() {
        let mut gate = GateReport {
            fine_relative_complex_error_passed: true,
            fine_magnitude_error_passed: true,
            fine_phase_error_passed: true,
            fine_direction_symmetry_passed: true,
            refinement_passed: true,
            comparison_passed: true,
        };
        assert!(gate.all_passed());
        gate.comparison_passed = false;
        assert!(!gate.all_passed());
    }
}
